// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Address-Bound TL-DSA Identity Keys (T-15, SPEC-2026-NEXT)
//!
//! Deterministic identity keypair derivation: a node's TL-DSA keypair is
//! derived solely from its Rep C address + the master secret. No key storage
//! needed beyond the master secret (T-12). If the master secret is known,
//! the identity can be regenerated for any address.
//!
//! ## Derivation
//!
//! ```text
//! seed = TLSponge-385("PlenumNET-IDENTITY" ‖ address_bytes ‖ master_secret)
//! keypair = TL-DSA-87::keygen(seed)
//! ```
//!
//! The seed is deterministic: same (address, master_secret) always produces
//! the same keypair. This means:
//!
//! - Node restart: regenerate from master secret — no key file needed
//! - Key rotation: change master secret → all keypairs change
//! - Verification: anyone with the public key can verify signatures
//!
//! ## LRU Cache
//!
//! Keypair derivation involves multiple TLSponge-385 evaluations + WOTS+
//! chain computation (~1ms per keypair). The LRU cache stores recently
//! derived keypairs to avoid recomputation.
//!
//! - Default capacity: 10,000 entries (covers large neighborhoods)
//! - Invalidation: on master_secret rotation, all cached entries are cleared
//! - Cache key: `(address, master_secret_fingerprint)` to avoid stale hits
//!
//! ## Dual-Accept During Rotation
//!
//! When the master secret rotates (T-12 arc-epoch), there's a transition
//! window where both old and new identity keys are valid:
//!
//! 1. Node rotates master secret → derives new keypair
//! 2. Re-registers with CRS using new public key
//! 3. Neighbors may still have the old public key cached
//! 4. During dual-accept window: CRS stores both public keys
//! 5. Verifiers try new key first, fall back to old key
//!
//! After the dual-accept window closes (max 182 days), only the new key
//! is accepted.

use std::collections::HashMap;

use crate::cube_addr::CubeAddr;
use crate::identity::MasterSecret;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Domain separator for identity seed derivation.
pub const IDENTITY_DOMAIN: &[u8] = b"PlenumNET-IDENTITY";

/// Domain separator for master secret fingerprinting.
pub const FINGERPRINT_DOMAIN: &[u8] = b"PlenumNET-MS-FP";

/// Seed length for TL-DSA keygen (must be ≥ variant sk_len for full entropy).
pub const IDENTITY_SEED_LEN: usize = 128;

/// Default LRU cache capacity.
pub const DEFAULT_CACHE_CAPACITY: usize = 10_000;

/// Fingerprint length (truncated hash of master secret for cache keying).
pub const FINGERPRINT_LEN: usize = 16;

/// TL-DSA variant used for identity keys.
pub const IDENTITY_VARIANT: ternary_math::tl_dsa::TlDsaVariant =
    ternary_math::tl_dsa::TlDsaVariant::TlDsa87;

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

/// Address key errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressKeyError {
    /// Master secret not set.
    NoMasterSecret,
    /// Address is invalid (contains zero trits).
    InvalidAddress,
    /// Key derivation produced unexpected output.
    DerivationFailed,
}

impl std::fmt::Display for AddressKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMasterSecret => write!(f, "master secret not set"),
            Self::InvalidAddress => write!(f, "address contains invalid trits"),
            Self::DerivationFailed => write!(f, "key derivation failed"),
        }
    }
}

impl std::error::Error for AddressKeyError {}

// ═══════════════════════════════════════════════════════════════════════
// IDENTITY KEYPAIR
// ═══════════════════════════════════════════════════════════════════════

/// An address-bound TL-DSA-87 identity keypair.
///
/// Derived deterministically from `(address, master_secret)`.
/// The public key is registered with CRS and used by neighbors
/// to verify signatures (T-06, T-07).
#[derive(Debug, Clone)]
pub struct IdentityKeypair {
    /// Full TL-DSA-87 public key (64 bytes).
    pub public_key: Vec<u8>,
    /// Full TL-DSA-87 secret key (128 bytes). Zeroized when the
    /// containing `AddressKeyManager` drops or rotates.
    pub secret_key: Vec<u8>,
    /// The address this keypair is bound to.
    pub address: CubeAddr,
    /// Fingerprint of the master secret used to derive this keypair.
    /// Used for cache invalidation on rotation.
    pub secret_fingerprint: [u8; FINGERPRINT_LEN],
}

impl IdentityKeypair {
    /// Sign a message with this identity key.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        ternary_math::tl_dsa::sign(&self.secret_key, message, IDENTITY_VARIANT)
    }

    /// Verify a message against this identity's public key.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        ternary_math::tl_dsa::verify(&self.public_key, message, signature, IDENTITY_VARIANT)
    }

    /// Get the public key length.
    pub fn pk_len(&self) -> usize {
        self.public_key.len()
    }

    /// Get the signature length for this variant.
    pub fn sig_len(&self) -> usize {
        ternary_math::tl_dsa::sig_len(IDENTITY_VARIANT)
    }
}

impl Drop for IdentityKeypair {
    /// Zeroize the secret key on drop.
    fn drop(&mut self) {
        for b in self.secret_key.iter_mut() {
            unsafe { std::ptr::write_volatile(b as *mut u8, 0x00); }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// KEY DERIVATION
// ═══════════════════════════════════════════════════════════════════════

/// Derive a deterministic seed for TL-DSA keygen from (address, master_secret).
///
/// `seed = TLSponge-385("PlenumNET-IDENTITY", address_bytes ‖ master_secret, 128)`
///
/// The 128-byte output ensures full entropy for TL-DSA-87 (sk_len = 128).
pub fn derive_identity_seed(addr: &CubeAddr, master_secret: &[u8]) -> Vec<u8> {
    let addr_bytes = addr.to_bytes();
    let mut material = Vec::with_capacity(addr_bytes.len() + master_secret.len());
    material.extend_from_slice(&addr_bytes);
    material.extend_from_slice(master_secret);

    ternary_math::sponge::derive_key(IDENTITY_DOMAIN, &material, IDENTITY_SEED_LEN)
}

/// Derive a full TL-DSA-87 keypair from (address, master_secret).
///
/// This is the core derivation function. Deterministic: same inputs
/// always produce the same keypair.
pub fn derive_identity_keypair(
    addr: &CubeAddr,
    master_secret: &MasterSecret,
) -> IdentityKeypair {
    let seed = derive_identity_seed(addr, master_secret.as_bytes());
    let kp = ternary_math::tl_dsa::keygen(IDENTITY_VARIANT, Some(&seed));
    let fingerprint = compute_fingerprint(master_secret);

    IdentityKeypair {
        public_key: kp.public_key,
        secret_key: kp.secret_key,
        address: addr.clone(),
        secret_fingerprint: fingerprint,
    }
}

/// Compute a fingerprint of the master secret for cache keying.
///
/// `fingerprint = TLSponge-385("PlenumNET-MS-FP", master_secret, 16)`
///
/// This is NOT the master secret — it's a one-way hash used to detect
/// when the secret has changed (cache invalidation).
pub fn compute_fingerprint(master_secret: &MasterSecret) -> [u8; FINGERPRINT_LEN] {
    let hash = ternary_math::sponge::derive_key(
        FINGERPRINT_DOMAIN,
        master_secret.as_bytes(),
        FINGERPRINT_LEN,
    );
    let mut fp = [0u8; FINGERPRINT_LEN];
    fp.copy_from_slice(&hash);
    fp
}

/// Derive only the public key for an address (no secret key).
///
/// Useful for verifiers who need to reconstruct the expected public key
/// from a known (address, master_secret) pair without storing the secret key.
pub fn derive_public_key(addr: &CubeAddr, master_secret: &MasterSecret) -> Vec<u8> {
    let kp = derive_identity_keypair(addr, master_secret);
    kp.public_key.clone()
}

// ═══════════════════════════════════════════════════════════════════════
// LRU CACHE
// ═══════════════════════════════════════════════════════════════════════

/// Simple LRU cache for derived keypairs.
///
/// Evicts the least-recently-used entry when capacity is exceeded.
/// Uses a HashMap + access-order Vec for O(1) lookup and O(n) eviction
/// (acceptable for cache sizes up to 10,000).
struct KeyCache {
    /// Cached keypairs: (address, fingerprint) → keypair.
    entries: HashMap<(CubeAddr, [u8; FINGERPRINT_LEN]), IdentityKeypair>,
    /// Access order: most recently accessed at the back.
    access_order: Vec<(CubeAddr, [u8; FINGERPRINT_LEN])>,
    /// Maximum capacity.
    capacity: usize,
}

impl KeyCache {
    fn new(capacity: usize) -> Self {
        KeyCache {
            entries: HashMap::new(),
            access_order: Vec::new(),
            capacity,
        }
    }

    /// Look up a cached keypair. Moves the entry to the back (most recent).
    fn get(&mut self, addr: &CubeAddr, fingerprint: &[u8; FINGERPRINT_LEN]) -> Option<&IdentityKeypair> {
        let key = (addr.clone(), *fingerprint);
        if self.entries.contains_key(&key) {
            // Move to back of access order
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key.clone());
            self.entries.get(&key)
        } else {
            None
        }
    }

    /// Insert a keypair. Evicts LRU entry if at capacity.
    fn insert(&mut self, keypair: IdentityKeypair) {
        let key = (keypair.address.clone(), keypair.secret_fingerprint);

        // Evict if at capacity and this is a new entry
        if !self.entries.contains_key(&key) && self.entries.len() >= self.capacity {
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.entries.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        // Remove old position in access order (if re-inserting)
        self.access_order.retain(|k| k != &key);
        self.access_order.push(key.clone());
        self.entries.insert(key, keypair);
    }

    /// Invalidate all entries with a specific fingerprint.
    ///
    /// Called when the master secret rotates.
    fn invalidate_fingerprint(&mut self, fingerprint: &[u8; FINGERPRINT_LEN]) {
        self.entries.retain(|k, _| &k.1 != fingerprint);
        self.access_order.retain(|k| &k.1 != fingerprint);
    }

    /// Clear the entire cache.
    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }

    /// Number of cached entries.
    fn len(&self) -> usize {
        self.entries.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ADDRESS KEY MANAGER
// ═══════════════════════════════════════════════════════════════════════

/// Manages address-bound identity keys with caching and dual-accept.
///
/// The primary interface for T-15. Nodes use this to:
///
/// - Derive their own identity keypair from their address + master secret
/// - Cache recently derived keypairs to avoid recomputation
/// - Handle dual-accept during master secret rotation (T-12)
/// - Verify signatures against expected public keys
pub struct AddressKeyManager {
    /// Keypair cache.
    cache: KeyCache,
    /// Current master secret fingerprint.
    current_fingerprint: Option<[u8; FINGERPRINT_LEN]>,
    /// Previous master secret fingerprint (during dual-accept).
    previous_fingerprint: Option<[u8; FINGERPRINT_LEN]>,
    /// Cache hit counter (for telemetry).
    cache_hits: u64,
    /// Cache miss counter.
    cache_misses: u64,
}

impl AddressKeyManager {
    /// Create a new key manager with default cache capacity.
    pub fn new() -> Self {
        AddressKeyManager {
            cache: KeyCache::new(DEFAULT_CACHE_CAPACITY),
            current_fingerprint: None,
            previous_fingerprint: None,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Create with custom cache capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        AddressKeyManager {
            cache: KeyCache::new(capacity),
            current_fingerprint: None,
            previous_fingerprint: None,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Set the current master secret.
    ///
    /// Computes the fingerprint and stores it for cache keying.
    /// Does NOT invalidate the cache — old entries from a previous
    /// secret remain accessible for dual-accept.
    pub fn set_master_secret(&mut self, master_secret: &MasterSecret) {
        let fp = compute_fingerprint(master_secret);
        self.previous_fingerprint = self.current_fingerprint;
        self.current_fingerprint = Some(fp);
    }

    /// Derive the identity keypair for an address using the current master secret.
    ///
    /// Checks the cache first. On miss, derives and caches the result.
    pub fn get_keypair(
        &mut self,
        addr: &CubeAddr,
        master_secret: &MasterSecret,
    ) -> IdentityKeypair {
        let fp = compute_fingerprint(master_secret);

        // Cache lookup
        if let Some(cached) = self.cache.get(addr, &fp) {
            self.cache_hits += 1;
            return cached.clone();
        }

        // Cache miss — derive
        self.cache_misses += 1;
        let kp = derive_identity_keypair(addr, master_secret);
        self.cache.insert(kp.clone());
        kp
    }

    /// Get only the public key for an address.
    ///
    /// Uses the cache if available.
    pub fn get_public_key(
        &mut self,
        addr: &CubeAddr,
        master_secret: &MasterSecret,
    ) -> Vec<u8> {
        self.get_keypair(addr, master_secret).public_key.clone()
    }

    /// Verify a signature against the expected address-bound public key.
    ///
    /// Derives the expected public key from (addr, master_secret) and
    /// verifies. If the current secret doesn't match, tries the previous
    /// secret (dual-accept during rotation).
    ///
    /// Returns `(valid, is_current_secret)`.
    pub fn verify_signature(
        &mut self,
        addr: &CubeAddr,
        message: &[u8],
        signature: &[u8],
        current_secret: &MasterSecret,
        previous_secret: Option<&MasterSecret>,
    ) -> (bool, bool) {
        // Try current secret first
        let kp = self.get_keypair(addr, current_secret);
        if kp.verify(message, signature) {
            return (true, true);
        }

        // Try previous secret (dual-accept)
        if let Some(prev) = previous_secret {
            let kp_prev = self.get_keypair(addr, prev);
            if kp_prev.verify(message, signature) {
                return (true, false);
            }
        }

        (false, false)
    }

    /// Invalidate all cached entries for the previous master secret.
    ///
    /// Called when the dual-accept window closes (T-12).
    pub fn close_dual_accept(&mut self) {
        if let Some(fp) = self.previous_fingerprint.take() {
            self.cache.invalidate_fingerprint(&fp);
        }
    }

    /// Clear the entire cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// Cache size.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Cache hit count.
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits
    }

    /// Cache miss count.
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses
    }

    /// Cache hit ratio (0.0–1.0).
    pub fn hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

impl Default for AddressKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr {
        CubeAddr::new(trits)
    }

    fn addr_a() -> CubeAddr { addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }
    fn addr_b() -> CubeAddr { addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }

    fn secret_1() -> MasterSecret { MasterSecret::from_seed(b"master-secret-epoch-0") }
    fn secret_2() -> MasterSecret { MasterSecret::from_seed(b"master-secret-epoch-1") }

    // ── Derivation ──────────────────────────────────────────────

    #[test]
    fn test_derive_deterministic() {
        let ms = secret_1();
        let kp1 = derive_identity_keypair(&addr_a(), &ms);
        let kp2 = derive_identity_keypair(&addr_a(), &ms);
        assert_eq!(kp1.public_key, kp2.public_key, "Same (addr, secret) → same pk");
        assert_eq!(kp1.secret_key, kp2.secret_key, "Same (addr, secret) → same sk");
    }

    #[test]
    fn test_derive_different_addresses() {
        let ms = secret_1();
        let kp_a = derive_identity_keypair(&addr_a(), &ms);
        let kp_b = derive_identity_keypair(&addr_b(), &ms);
        assert_ne!(kp_a.public_key, kp_b.public_key, "Different addrs → different pk");
    }

    #[test]
    fn test_derive_different_secrets() {
        let kp1 = derive_identity_keypair(&addr_a(), &secret_1());
        let kp2 = derive_identity_keypair(&addr_a(), &secret_2());
        assert_ne!(kp1.public_key, kp2.public_key, "Different secrets → different pk");
    }

    #[test]
    fn test_derive_correct_key_sizes() {
        let kp = derive_identity_keypair(&addr_a(), &secret_1());
        assert_eq!(kp.public_key.len(), 64, "TL-DSA-87 pk = 64 bytes");
        assert_eq!(kp.secret_key.len(), 128, "TL-DSA-87 sk = 128 bytes");
    }

    #[test]
    fn test_derive_public_key_matches() {
        let ms = secret_1();
        let kp = derive_identity_keypair(&addr_a(), &ms);
        let pk_only = derive_public_key(&addr_a(), &ms);
        assert_eq!(kp.public_key, pk_only);
    }

    // ── Sign and verify ─────────────────────────────────────────

    #[test]
    fn test_sign_verify_valid() {
        let kp = derive_identity_keypair(&addr_a(), &secret_1());
        let msg = b"PlenumNET CRS registration payload";
        let sig = kp.sign(msg);
        assert!(kp.verify(msg, &sig), "Valid signature must verify");
    }

    #[test]
    fn test_sign_verify_wrong_message() {
        let kp = derive_identity_keypair(&addr_a(), &secret_1());
        let sig = kp.sign(b"correct message");
        assert!(!kp.verify(b"wrong message", &sig));
    }

    #[test]
    fn test_sign_verify_wrong_key() {
        let kp1 = derive_identity_keypair(&addr_a(), &secret_1());
        let kp2 = derive_identity_keypair(&addr_b(), &secret_1());
        let msg = b"test message";
        let sig = kp1.sign(msg);
        assert!(!kp2.verify(msg, &sig), "Wrong key must fail verification");
    }

    #[test]
    fn test_sign_verify_after_rotation() {
        // Sign with old secret, verify with new secret → fails
        let kp_old = derive_identity_keypair(&addr_a(), &secret_1());
        let kp_new = derive_identity_keypair(&addr_a(), &secret_2());
        let msg = b"test message";
        let sig = kp_old.sign(msg);
        assert!(!kp_new.verify(msg, &sig), "Old sig must fail with new key");
    }

    // ── Fingerprint ─────────────────────────────────────────────

    #[test]
    fn test_fingerprint_deterministic() {
        let fp1 = compute_fingerprint(&secret_1());
        let fp2 = compute_fingerprint(&secret_1());
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_secrets() {
        let fp1 = compute_fingerprint(&secret_1());
        let fp2 = compute_fingerprint(&secret_2());
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_length() {
        let fp = compute_fingerprint(&secret_1());
        assert_eq!(fp.len(), FINGERPRINT_LEN);
    }

    // ── LRU Cache ───────────────────────────────────────────────

    #[test]
    fn test_cache_hit_miss() {
        let ms = secret_1();
        let fp = compute_fingerprint(&ms);
        let mut cache = KeyCache::new(10);

        // Miss
        assert!(cache.get(&addr_a(), &fp).is_none());

        // Insert + hit
        let kp = derive_identity_keypair(&addr_a(), &ms);
        cache.insert(kp.clone());
        assert!(cache.get(&addr_a(), &fp).is_some());
    }

    #[test]
    fn test_cache_eviction() {
        let ms = secret_1();
        let mut cache = KeyCache::new(2); // Tiny cache

        // Insert 3 entries into a cache of size 2
        let kp_a = derive_identity_keypair(&addr_a(), &ms);
        let kp_b = derive_identity_keypair(&addr_b(), &ms);
        let addr_c = addr([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let kp_c = derive_identity_keypair(&addr_c, &ms);

        cache.insert(kp_a);
        cache.insert(kp_b);
        assert_eq!(cache.len(), 2);

        cache.insert(kp_c); // Evicts LRU (addr_a)
        assert_eq!(cache.len(), 2);

        let fp = compute_fingerprint(&ms);
        assert!(cache.get(&addr_a(), &fp).is_none(), "LRU should be evicted");
        assert!(cache.get(&addr_b(), &fp).is_some());
    }

    #[test]
    fn test_cache_invalidate_fingerprint() {
        let ms1 = secret_1();
        let ms2 = secret_2();
        let fp1 = compute_fingerprint(&ms1);
        let fp2 = compute_fingerprint(&ms2);
        let mut cache = KeyCache::new(10);

        cache.insert(derive_identity_keypair(&addr_a(), &ms1));
        cache.insert(derive_identity_keypair(&addr_a(), &ms2));
        assert_eq!(cache.len(), 2);

        cache.invalidate_fingerprint(&fp1);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&addr_a(), &fp1).is_none());
        assert!(cache.get(&addr_a(), &fp2).is_some());
    }

    // ── AddressKeyManager ───────────────────────────────────────

    #[test]
    fn test_manager_get_keypair() {
        let ms = secret_1();
        let mut mgr = AddressKeyManager::new();
        mgr.set_master_secret(&ms);

        let kp = mgr.get_keypair(&addr_a(), &ms);
        assert_eq!(kp.public_key.len(), 64);
        assert_eq!(mgr.cache_misses(), 1);

        // Second call hits cache
        let kp2 = mgr.get_keypair(&addr_a(), &ms);
        assert_eq!(kp.public_key, kp2.public_key);
        assert_eq!(mgr.cache_hits(), 1);
    }

    #[test]
    fn test_manager_hit_ratio() {
        let ms = secret_1();
        let mut mgr = AddressKeyManager::new();
        mgr.set_master_secret(&ms);

        mgr.get_keypair(&addr_a(), &ms); // miss
        mgr.get_keypair(&addr_a(), &ms); // hit
        mgr.get_keypair(&addr_a(), &ms); // hit

        assert_eq!(mgr.cache_hits(), 2);
        assert_eq!(mgr.cache_misses(), 1);
        assert!((mgr.hit_ratio() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_manager_verify_signature_current() {
        let ms = secret_1();
        let mut mgr = AddressKeyManager::new();
        mgr.set_master_secret(&ms);

        let kp = mgr.get_keypair(&addr_a(), &ms);
        let msg = b"test message";
        let sig = kp.sign(msg);

        let (valid, is_current) = mgr.verify_signature(&addr_a(), msg, &sig, &ms, None);
        assert!(valid);
        assert!(is_current);
    }

    #[test]
    fn test_manager_verify_signature_dual_accept() {
        let ms_old = secret_1();
        let ms_new = secret_2();
        let mut mgr = AddressKeyManager::new();

        // Sign with old secret
        let kp_old = derive_identity_keypair(&addr_a(), &ms_old);
        let msg = b"signed before rotation";
        let sig = kp_old.sign(msg);

        // Rotate
        mgr.set_master_secret(&ms_new);

        // Verify with dual-accept
        let (valid, is_current) = mgr.verify_signature(
            &addr_a(), msg, &sig, &ms_new, Some(&ms_old),
        );
        assert!(valid, "Dual-accept: old sig must verify with previous secret");
        assert!(!is_current, "Should indicate previous secret was used");
    }

    #[test]
    fn test_manager_verify_fails_after_dual_accept_close() {
        let ms_old = secret_1();
        let ms_new = secret_2();
        let mut mgr = AddressKeyManager::new();

        let kp_old = derive_identity_keypair(&addr_a(), &ms_old);
        let msg = b"old message";
        let sig = kp_old.sign(msg);

        mgr.set_master_secret(&ms_new);

        // Verify without dual-accept (no previous secret)
        let (valid, _) = mgr.verify_signature(&addr_a(), msg, &sig, &ms_new, None);
        assert!(!valid, "Without previous secret, old sig must fail");
    }

    #[test]
    fn test_manager_close_dual_accept() {
        let ms_old = secret_1();
        let ms_new = secret_2();
        let mut mgr = AddressKeyManager::new();

        mgr.set_master_secret(&ms_old);
        mgr.get_keypair(&addr_a(), &ms_old); // Cache old key

        mgr.set_master_secret(&ms_new);
        assert_eq!(mgr.cache_size(), 1); // Old entry still cached

        mgr.close_dual_accept();
        // Old entries should be invalidated
        // (they had the old fingerprint)
    }

    #[test]
    fn test_manager_clear_cache() {
        let ms = secret_1();
        let mut mgr = AddressKeyManager::new();
        mgr.set_master_secret(&ms);

        mgr.get_keypair(&addr_a(), &ms);
        mgr.get_keypair(&addr_b(), &ms);
        assert_eq!(mgr.cache_size(), 2);

        mgr.clear_cache();
        assert_eq!(mgr.cache_size(), 0);
        assert_eq!(mgr.cache_hits(), 0);
    }

    // ── Zeroize ─────────────────────────────────────────────────

    #[test]
    fn test_keypair_drop_doesnt_panic() {
        let kp = derive_identity_keypair(&addr_a(), &secret_1());
        let _pk = kp.public_key.clone(); // Keep a copy of pk
        drop(kp); // Should zeroize sk without panic
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert_eq!(IDENTITY_SEED_LEN, 128);
        assert_eq!(FINGERPRINT_LEN, 16);
        assert_eq!(DEFAULT_CACHE_CAPACITY, 10_000);
        assert_eq!(IDENTITY_VARIANT, ternary_math::tl_dsa::TlDsaVariant::TlDsa87);
    }
}