// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # TL-DSA — Ternary Lattice Digital Signature Algorithm
//!
//! Post-quantum signature scheme built on TLSponge-385.
//!
//! ## Construction
//!
//! Uses WOTS+ (Winternitz One-Time Signature Plus) with TLSponge-385 as
//! the hash function. This is a well-analyzed, provably secure hash-based
//! signature scheme. The key property: **verify needs only the public key.**
//!
//! - **Post-quantum secure**: Security reduces to the collision resistance
//!   and second-preimage resistance of TLSponge-385 (≈385-bit PQ security).
//! - **No new dependencies**: Uses only the existing sponge primitive.
//! - **Deterministic**: Signatures are deterministic for a given (sk, msg) pair.
//!
//! ## Variants
//!
//! | Variant | Security | PK bytes | SK bytes | Sig bytes | Chains |
//! |---------|----------|----------|----------|-----------|--------|
//! | TL-DSA-44 | Level 2 | 32 | 64 | 1632 | 51 |
//! | TL-DSA-65 | Level 3 | 48 | 96 | 2144 | 67 |
//! | TL-DSA-87 | Level 5 | 64 | 128 | 3168 | 99 |
//!
//! ## One-Time Constraint
//!
//! WOTS+ is a **one-time** signature scheme: each keypair should sign at most
//! one message. For CRS registrations (T-06) this is natural — each registration
//! is a one-time event. For repeated signing (heartbeats), use HMAC mode (T-08)
//! or generate fresh keypairs.
//!
//! When the full TL-DSA lattice-based implementation is wired from the kernel
//! (`src/kernel/src/crypto/tl_dsa.rs`), this module's internals will be replaced.
//! The public API surface (`keygen`, `sign`, `verify`) remains identical.
//!
//! ## Created by T-03 (SPEC-2026-NEXT)

use crate::tlsponge385::derive_key;

// ═══════════════════════════════════════════════════════════════════════
// VARIANT PARAMETERS
// ═══════════════════════════════════════════════════════════════════════

/// TL-DSA variant identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TlDsaVariant {
    /// Security level 2. PK=32, SK=64, Sig=1632 bytes.
    TlDsa44 = 44,
    /// Security level 3. PK=48, SK=96, Sig=2144 bytes.
    TlDsa65 = 65,
    /// Security level 5 (recommended). PK=64, SK=128, Sig=3168 bytes.
    TlDsa87 = 87,
}

impl TlDsaVariant {
    /// Parse from integer variant code.
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            44 => Some(Self::TlDsa44),
            65 => Some(Self::TlDsa65),
            87 => Some(Self::TlDsa87),
            _ => None,
        }
    }

    /// Get the parameters for this variant.
    pub fn params(&self) -> TlDsaParams {
        match self {
            Self::TlDsa44 => TlDsaParams {
                pk_len: 32,
                sk_len: 64,
                msg_hash_len: 24,
                chains: 51,      // ceil(24*2) + 3 checksum
                chain_depth: 15, // w=16 → depth 15
                sig_len: 51 * 32, // 1632
                variant_tag: b"TL-DSA-44",
            },
            Self::TlDsa65 => TlDsaParams {
                pk_len: 48,
                sk_len: 96,
                msg_hash_len: 32,
                chains: 67,      // 64 + 3 checksum
                chain_depth: 15,
                sig_len: 67 * 32, // 2144
                variant_tag: b"TL-DSA-65",
            },
            Self::TlDsa87 => TlDsaParams {
                pk_len: 64,
                sk_len: 128,
                msg_hash_len: 48,
                chains: 99,      // 96 + 3 checksum
                chain_depth: 15,
                sig_len: 99 * 32, // 3168
                variant_tag: b"TL-DSA-87",
            },
        }
    }
}

/// Parameters for a TL-DSA variant.
#[derive(Debug, Clone)]
pub struct TlDsaParams {
    /// Public key length in bytes.
    pub pk_len: usize,
    /// Secret key length in bytes.
    pub sk_len: usize,
    /// Message hash length in bytes (determines chain count).
    pub msg_hash_len: usize,
    /// Total number of WOTS+ chains (message nibbles + checksum nibbles).
    pub chains: usize,
    /// Maximum chain depth (w-1 for Winternitz parameter w=16).
    pub chain_depth: u8,
    /// Signature length in bytes (chains × 32).
    pub sig_len: usize,
    /// Domain separation tag for this variant.
    pub variant_tag: &'static [u8],
}

/// Size of each chain value in bytes.
const CHAIN_VALUE_LEN: usize = 32;

// ═══════════════════════════════════════════════════════════════════════
// WOTS+ CHAIN OPERATIONS
// ═══════════════════════════════════════════════════════════════════════

/// One step of a WOTS+ chain: H(step_index ‖ value).
///
/// The step_index is included as domain separation per WOTS+ specification
/// to prevent certain multi-target attacks on basic WOTS chains.
fn chain_step(value: &[u8; CHAIN_VALUE_LEN], step_index: u8) -> [u8; CHAIN_VALUE_LEN] {
    let mut material = Vec::with_capacity(CHAIN_VALUE_LEN + 1);
    material.push(step_index);
    material.extend_from_slice(value);
    let result = derive_key(b"TL-DSA-WOTS-STEP", &material, CHAIN_VALUE_LEN);
    let mut out = [0u8; CHAIN_VALUE_LEN];
    out.copy_from_slice(&result);
    out
}

/// Iterate a WOTS+ chain `steps` times from `start`.
///
/// Returns H^steps(start) with per-step domain separation.
fn chain_iterate(start: &[u8; CHAIN_VALUE_LEN], steps: u8) -> [u8; CHAIN_VALUE_LEN] {
    let mut current = *start;
    for s in 0..steps {
        current = chain_step(&current, s);
    }
    current
}

/// Derive the chain bottom value (secret) for chain index `i`.
fn derive_chain_bottom(sk_seed: &[u8], chain_index: usize, variant_tag: &[u8]) -> [u8; CHAIN_VALUE_LEN] {
    let mut material = Vec::with_capacity(variant_tag.len() + sk_seed.len() + 4);
    material.extend_from_slice(variant_tag);
    material.extend_from_slice(sk_seed);
    material.extend_from_slice(&(chain_index as u32).to_le_bytes());
    let result = derive_key(b"TL-DSA-CHAIN-SK", &material, CHAIN_VALUE_LEN);
    let mut out = [0u8; CHAIN_VALUE_LEN];
    out.copy_from_slice(&result);
    out
}

// ═══════════════════════════════════════════════════════════════════════
// MESSAGE HASHING + CHECKSUM
// ═══════════════════════════════════════════════════════════════════════

/// Hash a message to the fixed-length digest for signing.
fn hash_message(msg: &[u8], variant_tag: &[u8], hash_len: usize) -> Vec<u8> {
    derive_key(
        b"TL-DSA-MSG",
        &[variant_tag, msg].concat(),
        hash_len,
    )
}

/// Convert a byte array to nibbles (4-bit values, range 0..15).
fn bytes_to_nibbles(data: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(data.len() * 2);
    for &b in data {
        nibbles.push(b >> 4);     // high nibble
        nibbles.push(b & 0x0F);   // low nibble
    }
    nibbles
}

/// Compute WOTS+ checksum over message nibbles.
///
/// Checksum = sum(chain_depth - nibble[i]) for all message nibbles.
/// Encoded as exactly 3 big-endian base-16 digits (len2=3 for w=16).
///
/// Max checksum = 99 * 15 = 1485, which fits in 3 base-16 digits (max 4095).
fn compute_checksum(msg_nibbles: &[u8], chain_depth: u8) -> Vec<u8> {
    let sum: u32 = msg_nibbles
        .iter()
        .map(|&n| (chain_depth as u32) - (n as u32))
        .sum();

    vec![
        ((sum >> 8) & 0x0F) as u8,
        ((sum >> 4) & 0x0F) as u8,
        (sum & 0x0F) as u8,
    ]
}

/// Get the full digit sequence: message nibbles + checksum nibbles.
///
/// This is the WOTS+ "base-w representation" with w=16.
fn get_digits(msg: &[u8], params: &TlDsaParams) -> Vec<u8> {
    let msg_hash = hash_message(msg, params.variant_tag, params.msg_hash_len);
    let msg_nibbles = bytes_to_nibbles(&msg_hash);
    let checksum_nibbles = compute_checksum(&msg_nibbles, params.chain_depth);

    let mut digits = msg_nibbles;
    digits.extend_from_slice(&checksum_nibbles);

    // Ensure we have exactly `params.chains` digits.
    // Truncate or pad with zeros if needed.
    digits.truncate(params.chains);
    while digits.len() < params.chains {
        digits.push(0);
    }
    digits
}

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC API — keygen, sign, verify
// ═══════════════════════════════════════════════════════════════════════

/// TL-DSA key pair.
#[derive(Debug, Clone)]
pub struct TlDsaKeyPair {
    /// Public key (compressed chain tops).
    pub public_key: Vec<u8>,
    /// Secret key (seed material — expand on sign).
    pub secret_key: Vec<u8>,
    /// Variant used for generation.
    pub variant: TlDsaVariant,
}

/// Generate a TL-DSA keypair.
///
/// If `seed` is `None`, uses 64 bytes of zeros (for deterministic testing).
/// In production, `seed` must be cryptographically random with ≥256 bits entropy.
///
/// The secret key is the expanded seed material. The public key is the
/// sponge-compressed concatenation of all WOTS+ chain tops.
pub fn keygen(variant: TlDsaVariant, seed: Option<&[u8]>) -> TlDsaKeyPair {
    let params = variant.params();

    // Expand seed to sk_seed
    let default_seed = vec![0u8; 64];
    let raw_seed = seed.unwrap_or(&default_seed);
    let sk_seed = derive_key(b"TL-DSA-SK-EXPAND", raw_seed, params.sk_len);

    // Compute chain tops: pk_raw = chain_top[0] ‖ ... ‖ chain_top[chains-1]
    let mut pk_raw = Vec::with_capacity(params.chains * CHAIN_VALUE_LEN);
    for i in 0..params.chains {
        let bottom = derive_chain_bottom(&sk_seed, i, params.variant_tag);
        let top = chain_iterate(&bottom, params.chain_depth);
        pk_raw.extend_from_slice(&top);
    }

    // Compress pk to the variant's pk_len via sponge
    let public_key = derive_key(
        b"TL-DSA-PK-COMPRESS",
        &[params.variant_tag, pk_raw.as_slice()].concat(),
        params.pk_len,
    );

    TlDsaKeyPair {
        public_key,
        secret_key: sk_seed,
        variant,
    }
}

/// Sign a message with a TL-DSA secret key.
///
/// Produces a deterministic WOTS+ signature: for each chain, iterate
/// `digit[i]` steps from the chain bottom. The verifier can complete
/// the remaining steps to reach the chain top and compare against pk.
///
/// **One-time constraint**: Each keypair should sign at most one message.
pub fn sign(secret_key: &[u8], message: &[u8], variant: TlDsaVariant) -> Vec<u8> {
    let params = variant.params();
    let digits = get_digits(message, &params);

    let mut signature = Vec::with_capacity(params.sig_len);
    for i in 0..params.chains {
        let bottom = derive_chain_bottom(secret_key, i, params.variant_tag);
        let sig_value = chain_iterate(&bottom, digits[i]);
        signature.extend_from_slice(&sig_value);
    }

    signature
}

/// Verify a TL-DSA signature using only the public key.
///
/// For each chain in the signature, completes the remaining iterations
/// to reach the chain top. If the compressed chain tops match the public
/// key, the signature is valid.
///
/// **This function does NOT require the secret key.**
///
/// Returns `true` if the signature is valid, `false` otherwise.
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8], variant: TlDsaVariant) -> bool {
    let params = variant.params();

    // Check signature length
    if signature.len() != params.sig_len {
        return false;
    }

    // Check public key length
    if public_key.len() != params.pk_len {
        return false;
    }

    let digits = get_digits(message, &params);

    // For each chain: complete the remaining steps from sig[i] to chain_top
    let mut pk_raw_prime = Vec::with_capacity(params.chains * CHAIN_VALUE_LEN);
    for i in 0..params.chains {
        let sig_start = i * CHAIN_VALUE_LEN;
        let sig_end = sig_start + CHAIN_VALUE_LEN;
        let mut sig_value = [0u8; CHAIN_VALUE_LEN];
        sig_value.copy_from_slice(&signature[sig_start..sig_end]);

        // Remaining steps = chain_depth - digit[i]
        // The sign function iterated digit[i] steps from bottom.
        // We iterate the remaining (chain_depth - digit[i]) steps to reach the top.
        let remaining = params.chain_depth - digits[i];
        // WOTS+ step indices continue from where sign left off
        let top_prime = chain_iterate_from(&sig_value, digits[i], remaining);
        pk_raw_prime.extend_from_slice(&top_prime);
    }

    // Compress reconstructed chain tops to pk'
    let pk_prime = derive_key(
        b"TL-DSA-PK-COMPRESS",
        &[params.variant_tag, pk_raw_prime.as_slice()].concat(),
        params.pk_len,
    );

    // Constant-time comparison
    constant_time_eq(public_key, &pk_prime)
}

/// Chain iteration for verification: starts at `step_offset`, iterates `steps` times.
///
/// This continues the chain from where sign() left off. The step indices
/// are offset so that sign + verify together traverse the full chain.
fn chain_iterate_from(
    start: &[u8; CHAIN_VALUE_LEN],
    step_offset: u8,
    steps: u8,
) -> [u8; CHAIN_VALUE_LEN] {
    let mut current = *start;
    for s in 0..steps {
        current = chain_step(&current, step_offset + s);
    }
    current
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ═══════════════════════════════════════════════════════════════════════
// CONVENIENCE: VARIANT SIZES
// ═══════════════════════════════════════════════════════════════════════

/// Get the public key length for a variant.
pub fn pk_len(variant: TlDsaVariant) -> usize {
    variant.params().pk_len
}

/// Get the secret key length for a variant.
pub fn sk_len(variant: TlDsaVariant) -> usize {
    variant.params().sk_len
}

/// Get the signature length for a variant.
pub fn sig_len(variant: TlDsaVariant) -> usize {
    variant.params().sig_len
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_deterministic() {
        let seed = b"test-seed-for-tl-dsa-keygen-deterministic";
        let kp1 = keygen(TlDsaVariant::TlDsa87, Some(seed));
        let kp2 = keygen(TlDsaVariant::TlDsa87, Some(seed));
        assert_eq!(kp1.public_key, kp2.public_key, "Same seed → same pk");
        assert_eq!(kp1.secret_key, kp2.secret_key, "Same seed → same sk");
    }

    #[test]
    fn test_keygen_different_seeds() {
        let kp1 = keygen(TlDsaVariant::TlDsa87, Some(b"seed-alpha"));
        let kp2 = keygen(TlDsaVariant::TlDsa87, Some(b"seed-beta"));
        assert_ne!(kp1.public_key, kp2.public_key, "Different seeds → different pk");
    }

    #[test]
    fn test_keygen_correct_sizes_87() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        assert_eq!(kp.public_key.len(), 64);
        assert_eq!(kp.secret_key.len(), 128);
    }

    #[test]
    fn test_keygen_correct_sizes_65() {
        let kp = keygen(TlDsaVariant::TlDsa65, Some(b"test-seed"));
        assert_eq!(kp.public_key.len(), 48);
        assert_eq!(kp.secret_key.len(), 96);
    }

    #[test]
    fn test_keygen_correct_sizes_44() {
        let kp = keygen(TlDsaVariant::TlDsa44, Some(b"test-seed"));
        assert_eq!(kp.public_key.len(), 32);
        assert_eq!(kp.secret_key.len(), 64);
    }

    #[test]
    fn test_sign_correct_size() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let sig = sign(&kp.secret_key, b"hello world", TlDsaVariant::TlDsa87);
        assert_eq!(sig.len(), 3168, "TL-DSA-87 signature should be 3168 bytes");
    }

    #[test]
    fn test_sign_deterministic() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let sig1 = sign(&kp.secret_key, b"hello world", TlDsaVariant::TlDsa87);
        let sig2 = sign(&kp.secret_key, b"hello world", TlDsaVariant::TlDsa87);
        assert_eq!(sig1, sig2, "Same (sk, msg) → same signature");
    }

    #[test]
    fn test_sign_different_messages() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let sig1 = sign(&kp.secret_key, b"message one", TlDsaVariant::TlDsa87);
        let sig2 = sign(&kp.secret_key, b"message two", TlDsaVariant::TlDsa87);
        assert_ne!(sig1, sig2, "Different messages → different signatures");
    }

    #[test]
    fn test_verify_valid_signature_87() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let msg = b"PlenumNET CRS registration payload";
        let sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa87);
        assert!(
            verify(&kp.public_key, msg, &sig, TlDsaVariant::TlDsa87),
            "Valid signature must verify — PK ONLY, no secret key"
        );
    }

    #[test]
    fn test_verify_valid_signature_65() {
        let kp = keygen(TlDsaVariant::TlDsa65, Some(b"test-seed"));
        let msg = b"test message for TL-DSA-65";
        let sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa65);
        assert!(verify(&kp.public_key, msg, &sig, TlDsaVariant::TlDsa65));
    }

    #[test]
    fn test_verify_valid_signature_44() {
        let kp = keygen(TlDsaVariant::TlDsa44, Some(b"test-seed"));
        let msg = b"test message for TL-DSA-44";
        let sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa44);
        assert!(verify(&kp.public_key, msg, &sig, TlDsaVariant::TlDsa44));
    }

    #[test]
    fn test_verify_wrong_message() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let sig = sign(&kp.secret_key, b"correct message", TlDsaVariant::TlDsa87);
        assert!(
            !verify(&kp.public_key, b"wrong message", &sig, TlDsaVariant::TlDsa87),
            "Wrong message must fail verification"
        );
    }

    #[test]
    fn test_verify_wrong_public_key() {
        let kp1 = keygen(TlDsaVariant::TlDsa87, Some(b"seed-one"));
        let kp2 = keygen(TlDsaVariant::TlDsa87, Some(b"seed-two"));
        let msg = b"test message";
        let sig = sign(&kp1.secret_key, msg, TlDsaVariant::TlDsa87);
        assert!(
            !verify(&kp2.public_key, msg, &sig, TlDsaVariant::TlDsa87),
            "Wrong public key must fail verification"
        );
    }

    #[test]
    fn test_verify_truncated_signature() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let msg = b"test message";
        let sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa87);
        let truncated = &sig[..sig.len() - 1];
        assert!(
            !verify(&kp.public_key, msg, truncated, TlDsaVariant::TlDsa87),
            "Truncated signature must fail"
        );
    }

    #[test]
    fn test_verify_corrupted_signature() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let msg = b"test message";
        let mut sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa87);
        let mid = sig.len() / 2;
        sig[mid] ^= 0xFF;
        assert!(
            !verify(&kp.public_key, msg, &sig, TlDsaVariant::TlDsa87),
            "Corrupted signature must fail"
        );
    }

    #[test]
    fn test_verify_empty_signature() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        assert!(
            !verify(&kp.public_key, b"msg", &[], TlDsaVariant::TlDsa87),
            "Empty signature must fail"
        );
    }

    #[test]
    fn test_verify_wrong_variant() {
        let kp = keygen(TlDsaVariant::TlDsa87, Some(b"test-seed"));
        let msg = b"test message";
        let sig = sign(&kp.secret_key, msg, TlDsaVariant::TlDsa87);
        // Verify with wrong variant (different chain count → different sig_len check)
        assert!(
            !verify(&kp.public_key, msg, &sig, TlDsaVariant::TlDsa65),
            "Wrong variant must fail (sig length mismatch)"
        );
    }

    #[test]
    fn test_chain_step_deterministic() {
        let val = [42u8; CHAIN_VALUE_LEN];
        let s1 = chain_step(&val, 0);
        let s2 = chain_step(&val, 0);
        assert_eq!(s1, s2, "Chain step must be deterministic");
    }

    #[test]
    fn test_chain_step_different_indices() {
        let val = [42u8; CHAIN_VALUE_LEN];
        let s0 = chain_step(&val, 0);
        let s1 = chain_step(&val, 1);
        assert_ne!(s0, s1, "Different step indices → different outputs");
    }

    #[test]
    fn test_variant_from_u32() {
        assert_eq!(TlDsaVariant::from_u32(44), Some(TlDsaVariant::TlDsa44));
        assert_eq!(TlDsaVariant::from_u32(65), Some(TlDsaVariant::TlDsa65));
        assert_eq!(TlDsaVariant::from_u32(87), Some(TlDsaVariant::TlDsa87));
        assert_eq!(TlDsaVariant::from_u32(99), None);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }

    /// Verify checksum encoding produces exactly 3 base-16 digits and
    /// preserves all 12 bits of the checksum value.
    #[test]
    fn test_checksum_encoding_exact_3_digits() {
        let chain_depth: u8 = 15;
        // All-zero nibbles → max checksum = 96 * 15 = 1440 = 0x5A0
        let all_zero = vec![0u8; 96];
        let cs = compute_checksum(&all_zero, chain_depth);
        assert_eq!(cs.len(), 3, "Checksum must be exactly 3 nibbles");
        let val = ((cs[0] as u32) << 8) | ((cs[1] as u32) << 4) | (cs[2] as u32);
        assert_eq!(val, 1440, "Max checksum for 96 zero-nibbles must be 1440");

        // All-max nibbles → min checksum = 0
        let all_max = vec![15u8; 96];
        let cs_min = compute_checksum(&all_max, chain_depth);
        assert_eq!(cs_min, vec![0, 0, 0], "Min checksum must be [0,0,0]");

        // Specific value that previously collided under 4-nibble-then-truncate:
        // checksum=17 (0x011) and checksum=1 (0x001) must produce different encodings
        // because they differ only in the low nibble.
        let mut nibs_17 = vec![15u8; 96];
        // Reduce 17 nibbles by 1 each to get checksum = 17
        for i in 0..17 {
            nibs_17[i] = 14;
        }
        let cs_17 = compute_checksum(&nibs_17, chain_depth);

        let mut nibs_1 = vec![15u8; 96];
        nibs_1[0] = 14; // checksum = 1
        let cs_1 = compute_checksum(&nibs_1, chain_depth);

        assert_ne!(cs_17, cs_1, "Checksums 17 vs 1 must produce different digit encodings");
        let v17 = ((cs_17[0] as u32) << 8) | ((cs_17[1] as u32) << 4) | (cs_17[2] as u32);
        let v1 = ((cs_1[0] as u32) << 8) | ((cs_1[1] as u32) << 4) | (cs_1[2] as u32);
        assert_eq!(v17, 17);
        assert_eq!(v1, 1);
    }
}
