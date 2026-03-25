// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Cube Registration Service (CRS) — Service 3
//!
//! Allows new cubes to join the network, assigns them a unique Rep C address,
//! and provides the physical endpoint information they need to establish
//! overlay tunnels to their geometric neighbors.
//!
//! ## T-06 (SPEC-2026-NEXT): Signed CRS Registrations
//!
//! Every registration payload now includes a TL-DSA signature computed by
//! the registering node. CRS stores the signature alongside the record.
//! Querying nodes verify the signature before establishing tunnels (T-07).
//!
//! ### Signature Construction
//!
//! Canonical byte concatenation:
//! `address.to_wire() ‖ endpoint.as_bytes() ‖ public_key ‖ kem_public_key ‖ timestamp_le`
//!
//! Domain separator: `"PlenumNET-CRS-REG-v1"` (via TLSponge-385).
//! Signature algorithm: TL-DSA-87 (WOTS+ over TLSponge-385).
//!
//! ### Timestamp Policy
//!
//! - `u128` femtoseconds since Salvi Epoch (HPTP-synchronized, not NTP)
//! - Replay window: 30 seconds (`REGISTRATION_MAX_AGE_FS`)
//! - Future tolerance: 1 second (`TIMESTAMP_FUTURE_TOLERANCE_FS`)
//! - Re-registration requires strictly newer timestamp
//!
//! ### Feature Flag
//!
//! When `PlenumConfig.require_signature == false` (default), unsigned
//! registrations via `register()` are still accepted. When `true`, only
//! `register_signed()` succeeds.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cube_addr::{CubeAddr, RepCTrit, DIMENSIONS, NEIGHBORS_PER_CUBE, TOTAL_VERTICES};
use crate::wire::{
    pack_addr, WIRE_ADDR_SIZE,
    REGISTRATION_MAX_AGE_FS, TIMESTAMP_FUTURE_TOLERANCE_FS,
};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default heartbeat interval expected from registered cubes.
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 30_000; // 30 seconds

/// Default grace period after deregistration before address reuse.
const DEFAULT_GRACE_PERIOD_SECS: u64 = 86_400; // 24 hours

/// Default offline threshold: if no heartbeat for this long, mark offline.
const DEFAULT_OFFLINE_THRESHOLD_SECS: u64 = 120; // 2 minutes

/// Domain separator for CRS registration signatures.
/// Used as context for TLSponge-385 domain separation in TL-DSA.
pub const CRS_REG_DOMAIN: &[u8] = b"PlenumNET-CRS-REG-v1";

/// TL-DSA variant used for CRS signatures.
/// TL-DSA-87 = Level 5 security (post-quantum).
pub const CRS_SIG_VARIANT: u8 = 87;

// ═══════════════════════════════════════════════════════════════════════
// CUBE STATUS
// ═══════════════════════════════════════════════════════════════════════

/// Status of a registered cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CubeStatus {
    /// Sending heartbeats, tunnels expected to be active.
    Active,
    /// Shutting down gracefully — stop sending new traffic.
    Draining,
    /// Missed heartbeats — neighbors have been notified.
    Offline,
}

impl std::fmt::Display for CubeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CubeStatus::Active => write!(f, "active"),
            CubeStatus::Draining => write!(f, "draining"),
            CubeStatus::Offline => write!(f, "offline"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE RECORD — The only stored state in the system
// ═══════════════════════════════════════════════════════════════════════

/// Registration record for a cube in the network.
/// This is the Registry Database entry — the only persistent state
/// in the entire inter-cube infrastructure.
#[derive(Debug, Clone)]
pub struct CubeRecord {
    /// Primary key: 13-trit Rep C address (no zeros, guaranteed).
    pub addr: CubeAddr,
    /// Physical IP:port of the cube's gateway nodes.
    pub endpoints: Vec<SocketAddr>,
    /// Identity public key for tunnel authentication (TL-DSA-87, 64 bytes).
    pub public_key: Vec<u8>,
    /// TL-KEM public key for key exchange (T-15 address-bound keys).
    pub kem_public_key: Option<Vec<u8>>,
    /// Current status.
    pub status: CubeStatus,
    /// Last heartbeat timestamp (monotonic, for health checks).
    pub last_heartbeat: Instant,
    /// When this cube was first registered (monotonic, for uptime).
    pub registered_at: Instant,
    /// Registration timestamp in femtoseconds since Salvi Epoch (HPTP).
    /// Used for replay protection: re-registration must have a strictly
    /// newer timestamp than this value.
    pub registered_at_fs: u128,
    /// TL-DSA registration signature (T-06).
    ///
    /// Signature over canonical message:
    /// `address.to_wire() ‖ endpoint ‖ public_key ‖ kem_public_key ‖ timestamp_le`
    ///
    /// Stored so querying nodes can verify the record was created by the
    /// holder of the signing key (T-07 neighbor-side verification).
    /// `None` for legacy unsigned registrations.
    pub reg_signature: Option<Vec<u8>>,
    /// Whether this record uses a legacy (unbound) key.
    /// Set to `true` for pre-T-15 registrations. T-15 address-bound keys
    /// will set this to `false`.
    pub legacy_key: bool,
    /// Hierarchical level this CRS manages (0 = root).
    pub level: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNED REGISTRATION — T-06 payload
// ═══════════════════════════════════════════════════════════════════════

/// A signed CRS registration payload (T-06).
///
/// Contains all fields needed for registration plus the TL-DSA signature.
/// The signature covers the canonical concatenation of all other fields.
///
/// `public_key` is the full-length TL-DSA-87 public key (64 bytes) used
/// for signature verification. The CRS stores a 32-byte truncation in
/// `CubeRecord.public_key` as the identity key for tunnel authentication.
#[derive(Debug, Clone)]
pub struct SignedRegistration {
    /// Desired address (or None for auto-allocation).
    pub address: Option<CubeAddr>,
    /// Physical endpoint (IP:port).
    pub endpoint: SocketAddr,
    /// Full TL-DSA-87 public key for signature verification (64 bytes).
    /// A 32-byte truncation is stored in CubeRecord as the identity key.
    pub public_key: Vec<u8>,
    /// TL-KEM public key for key exchange (optional, for T-15).
    pub kem_public_key: Option<Vec<u8>>,
    /// Femtosecond timestamp since Salvi Epoch (HPTP-synchronized).
    pub timestamp_fs: u128,
    /// TL-DSA-87 signature over the canonical message.
    pub signature: Vec<u8>,
}

impl SignedRegistration {
    /// Construct the canonical message that was signed.
    ///
    /// Format: `domain ‖ address.to_wire() ‖ endpoint ‖ public_key ‖ kem_public_key ‖ timestamp_le`
    ///
    /// The domain separator `"PlenumNET-CRS-REG-v1"` is prepended to prevent
    /// cross-protocol signature reuse.
    pub fn canonical_message(&self, assigned_addr: &CubeAddr) -> Vec<u8> {
        build_registration_message(
            assigned_addr,
            &self.endpoint,
            &self.public_key,
            self.kem_public_key.as_deref(),
            self.timestamp_fs,
        )
    }

}

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR INFO — Returned to joining cubes
// ═══════════════════════════════════════════════════════════════════════

/// Information about a geometric neighbor, returned during registration.
#[derive(Debug, Clone)]
pub struct NeighborInfo {
    /// The neighbor's Rep C cube address (computed from trit flips).
    pub addr: CubeAddr,
    /// Physical endpoint (if the neighbor is registered).
    pub endpoint: Option<SocketAddr>,
    /// Public key (if the neighbor is registered, full TL-DSA-87 key).
    pub public_key: Option<Vec<u8>>,
    /// Status (if registered).
    pub status: Option<CubeStatus>,
    /// Registration signature (T-06).
    /// Included so querying nodes can verify the record (T-07).
    pub reg_signature: Option<Vec<u8>>,
    /// KEM public key (if available, for T-15 key exchange).
    pub kem_public_key: Option<Vec<u8>>,
}

// ═══════════════════════════════════════════════════════════════════════
// REGISTRATION RESULT
// ═══════════════════════════════════════════════════════════════════════

/// Result of a successful cube registration.
#[derive(Debug, Clone)]
pub struct RegistrationResult {
    /// The assigned Rep C address.
    pub address: CubeAddr,
    /// Computed neighbors with their endpoint info.
    pub neighbors: Vec<NeighborInfo>,
}

/// Errors during registration.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationError {
    /// Address space is full.
    AddressSpaceExhausted,
    /// Requested address is already in use.
    AddressInUse,
    /// Requested address contains zero (invalid Rep C).
    InvalidAddress,
    /// Public key is required.
    MissingPublicKey,
    /// TL-DSA signature verification failed (T-06).
    InvalidSignature,
    /// Timestamp is outside the acceptable window (T-06).
    /// Either too old (> 30s) or too far in the future (> 1s).
    StaleTimestamp,
    /// Re-registration attempted with a timestamp ≤ the existing record (T-06).
    /// Prevents replay of old registration payloads.
    ReplayDetected,
    /// Signed registration required but unsigned payload received.
    /// Only when `PlenumConfig.require_signature == true`.
    SignatureRequired,
    /// Address not found in registry (for key update).
    AddressNotFound,
}

// ═══════════════════════════════════════════════════════════════════════
// ADDRESS ALLOCATOR — Manages the Rep C address space
// ═══════════════════════════════════════════════════════════════════════

/// Bitmap-based address allocator for the 3¹³ = 1,594,323 address space.
///
/// Guarantees all allocated addresses are valid Rep C (no zeros).
/// By construction of the flat_index ↔ CubeAddr bijection, every index
/// maps to a valid Rep C address — zero cannot appear.
struct AddressAllocator {
    /// Bitmap: true = in use.
    used: Vec<bool>,
    /// Next allocation hint (sequential scan optimization).
    next_hint: u64,
    /// Count of used addresses.
    used_count: u64,
    /// Addresses in grace period (recently deregistered).
    grace_period: HashMap<u64, Instant>,
    /// Grace period duration.
    grace_duration: Duration,
}

impl AddressAllocator {
    fn new() -> Self {
        AddressAllocator {
            used: vec![false; TOTAL_VERTICES as usize],
            next_hint: 0,
            used_count: 0,
            grace_period: HashMap::new(),
            grace_duration: Duration::from_secs(DEFAULT_GRACE_PERIOD_SECS),
        }
    }

    /// Allocate the next available address.
    fn allocate(&mut self) -> Option<CubeAddr> {
        let now = Instant::now();
        let total = TOTAL_VERTICES as usize;

        // Scan from hint position
        for offset in 0..total {
            let idx = ((self.next_hint as usize) + offset) % total;
            if !self.used[idx] {
                // Check grace period
                if let Some(released_at) = self.grace_period.get(&(idx as u64)) {
                    if now.duration_since(*released_at) < self.grace_duration {
                        continue; // Still in grace period
                    }
                    self.grace_period.remove(&(idx as u64));
                }

                self.used[idx] = true;
                self.used_count += 1;
                self.next_hint = ((idx + 1) % total) as u64;
                return CubeAddr::from_flat_index(idx as u64);
            }
        }
        None // Address space exhausted
    }

    /// Allocate a specific requested address.
    fn allocate_specific(&mut self, addr: &CubeAddr) -> bool {
        let idx = addr.flat_index() as usize;
        if self.used[idx] {
            return false;
        }
        if let Some(released_at) = self.grace_period.get(&(idx as u64)) {
            if Instant::now().duration_since(*released_at) < self.grace_duration {
                return false; // Still in grace period
            }
            self.grace_period.remove(&(idx as u64));
        }
        self.used[idx] = true;
        self.used_count += 1;
        true
    }

    /// Release an address (enters grace period).
    fn release(&mut self, addr: &CubeAddr) {
        let idx = addr.flat_index() as usize;
        if self.used[idx] {
            self.used[idx] = false;
            self.used_count -= 1;
            self.grace_period.insert(idx as u64, Instant::now());
        }
    }

    /// Number of used addresses.
    fn count(&self) -> u64 {
        self.used_count
    }

    /// Number of available addresses (excluding grace period).
    fn available(&self) -> u64 {
        TOTAL_VERTICES - self.used_count
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SIGNATURE VERIFICATION — T-06
// ═══════════════════════════════════════════════════════════════════════

/// Verify a TL-DSA-87 signature on a signed registration payload.
///
/// Reconstructs the canonical message from the payload fields and verifies
/// using the public key. Does NOT require the secret key.
///
/// Returns `Ok(())` on valid signature, or the appropriate
/// `RegistrationError` on failure.
fn verify_registration_signature(
    reg: &SignedRegistration,
    assigned_addr: &CubeAddr,
) -> Result<(), RegistrationError> {
    let canonical_msg = reg.canonical_message(assigned_addr);

    let variant = ternary_math::tl_dsa::TlDsaVariant::from_u32(CRS_SIG_VARIANT as u32)
        .ok_or(RegistrationError::InvalidSignature)?;

    let valid = ternary_math::tl_dsa::verify(
        &reg.public_key,
        &canonical_msg,
        &reg.signature,
        variant,
    );

    if valid {
        Ok(())
    } else {
        Err(RegistrationError::InvalidSignature)
    }
}

/// Validate a registration timestamp against the current time.
///
/// Checks:
/// 1. Timestamp is not too far in the future (> 1s = TIMESTAMP_FUTURE_TOLERANCE_FS)
/// 2. Timestamp is not too old (> 30s = REGISTRATION_MAX_AGE_FS)
/// 3. If re-registering, timestamp is strictly newer than existing record
fn validate_timestamp(
    timestamp_fs: u128,
    now_fs: u128,
    existing_ts: Option<u128>,
) -> Result<(), RegistrationError> {
    // Future check
    if timestamp_fs > now_fs + TIMESTAMP_FUTURE_TOLERANCE_FS {
        return Err(RegistrationError::StaleTimestamp);
    }

    // Staleness check
    if now_fs > timestamp_fs && (now_fs - timestamp_fs) > REGISTRATION_MAX_AGE_FS {
        return Err(RegistrationError::StaleTimestamp);
    }

    // Replay check: must be strictly newer than existing record
    if let Some(existing) = existing_ts {
        if timestamp_fs <= existing {
            return Err(RegistrationError::ReplayDetected);
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// CANONICAL MESSAGE CONSTRUCTION — For callers who need to sign
// ═══════════════════════════════════════════════════════════════════════

/// Construct the canonical message for a CRS registration signature.
///
/// This is the message that the registering node signs with TL-DSA-87.
/// Both the signer and verifier must construct the same message.
///
/// Format: `domain ‖ address_wire ‖ endpoint_string ‖ public_key ‖ kem_public_key ‖ timestamp_le`
pub fn build_registration_message(
    addr: &CubeAddr,
    endpoint: &SocketAddr,
    public_key: &[u8],
    kem_public_key: Option<&[u8]>,
    timestamp_fs: u128,
) -> Vec<u8> {
    let addr_wire = pack_addr(addr).unwrap_or([0u8; WIRE_ADDR_SIZE]);
    let endpoint_str = endpoint.to_string();
    let kem_bytes = kem_public_key.unwrap_or(&[]);
    let ts_bytes = timestamp_fs.to_le_bytes();

    let mut msg = Vec::with_capacity(
        CRS_REG_DOMAIN.len()
            + WIRE_ADDR_SIZE
            + endpoint_str.len()
            + public_key.len()
            + kem_bytes.len()
            + 16,
    );
    msg.extend_from_slice(CRS_REG_DOMAIN);
    msg.extend_from_slice(&addr_wire);
    msg.extend_from_slice(endpoint_str.as_bytes());
    msg.extend_from_slice(public_key);
    msg.extend_from_slice(kem_bytes);
    msg.extend_from_slice(&ts_bytes);
    msg
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE REGISTRATION SERVICE
// ═══════════════════════════════════════════════════════════════════════

/// The Cube Registration Service coordinator.
///
/// Manages address allocation, endpoint registry, and neighbor computation.
/// In production, runs as a 3–5 node Raft cluster for fault tolerance.
pub struct CubeRegistrationService {
    /// Address allocator (bitmap over 3¹³ space).
    allocator: AddressAllocator,
    /// Registry database: addr → CubeRecord.
    registry: HashMap<CubeAddr, CubeRecord>,
    /// Expected heartbeat interval.
    heartbeat_interval: Duration,
    /// Offline threshold.
    offline_threshold: Duration,
    /// Hierarchical level this CRS manages (0 = root).
    level: usize,
}

impl CubeRegistrationService {
    /// Create a new CRS coordinator.
    pub fn new() -> Self {
        CubeRegistrationService {
            allocator: AddressAllocator::new(),
            registry: HashMap::new(),
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            offline_threshold: Duration::from_secs(DEFAULT_OFFLINE_THRESHOLD_SECS),
            level: 0,
        }
    }

    /// Create for a specific hierarchy level.
    pub fn at_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // REGISTRATION — Legacy unsigned path
    // ═══════════════════════════════════════════════════════════════

    /// Register a new cube (legacy unsigned path).
    ///
    /// Allocates an address (or uses the requested one) and returns the
    /// address along with neighbor endpoint information.
    ///
    /// When `PlenumConfig.require_signature == true`, this function should
    /// NOT be called — use `register_signed()` instead. The API layer
    /// (`api.rs`) enforces this gate.
    pub fn register(
        &mut self,
        endpoint: SocketAddr,
        public_key: Vec<u8>,
        desired_address: Option<CubeAddr>,
    ) -> Result<RegistrationResult, RegistrationError> {
        let now = Instant::now();

        // Allocate address
        let addr = if let Some(desired) = desired_address {
            // Validate desired address is valid Rep C
            let bytes = desired.to_bytes();
            for &b in &bytes {
                if b < 1 || b > 3 {
                    return Err(RegistrationError::InvalidAddress);
                }
            }
            if !self.allocator.allocate_specific(&desired) {
                return Err(RegistrationError::AddressInUse);
            }
            desired
        } else {
            self.allocator
                .allocate()
                .ok_or(RegistrationError::AddressSpaceExhausted)?
        };

        // Create registry record (unsigned — legacy)
        let record = CubeRecord {
            addr: addr.clone(),
            endpoints: vec![endpoint],
            public_key,
            kem_public_key: None,
            status: CubeStatus::Active,
            last_heartbeat: now,
            registered_at: now,
            registered_at_fs: 0,
            reg_signature: None,
            legacy_key: true,
            level: self.level,
        };
        self.registry.insert(addr.clone(), record);

        // Compute 26 neighbors and look up their endpoints
        let neighbors = self.compute_neighbor_info(&addr);

        Ok(RegistrationResult {
            address: addr,
            neighbors,
        })
    }

    // ═══════════════════════════════════════════════════════════════
    // REGISTRATION — Signed path (T-06)
    // ═══════════════════════════════════════════════════════════════

    /// Register a new cube with a TL-DSA-87 signed payload (T-06).
    ///
    /// Verifies the signature, validates the timestamp, allocates the
    /// address, and stores the signature alongside the record so querying
    /// nodes can verify it (T-07 neighbor-side verification).
    ///
    /// **Signature verification uses only the public key** — CRS does not
    /// need the registrant's secret key.
    ///
    /// ## Replay Protection
    ///
    /// The femtosecond timestamp is part of the signed message. CRS enforces:
    /// - Maximum registration age: 30s (`REGISTRATION_MAX_AGE_FS`)
    /// - Future tolerance: 1s (`TIMESTAMP_FUTURE_TOLERANCE_FS`)
    /// - Re-registration at the same address requires strictly newer timestamp
    ///
    /// ## Sybil Cost
    ///
    /// Each fake identity requires: 1 TL-DSA keypair + valid signature +
    /// 26 authenticated tunnels + 26 heartbeat responses per interval.
    /// Structural Sybil resistance from the geometry itself.
    pub fn register_signed(
        &mut self,
        reg: &SignedRegistration,
        now_fs: u128,
    ) -> Result<RegistrationResult, RegistrationError> {
        let now = Instant::now();

        // Step 1: Allocate or claim the address
        let addr = if let Some(ref desired) = reg.address {
            let bytes = desired.to_bytes();
            for &b in &bytes {
                if b < 1 || b > 3 {
                    return Err(RegistrationError::InvalidAddress);
                }
            }

            // Check for re-registration (same address, newer timestamp)
            let existing_ts = self.registry.get(desired).map(|r| r.registered_at_fs);

            // Validate timestamp (staleness, future, replay)
            validate_timestamp(reg.timestamp_fs, now_fs, existing_ts)?;

            // If re-registering, release the old allocation first
            if self.registry.contains_key(desired) {
                self.registry.remove(desired);
                // Don't release from allocator — we're reclaiming the same address
            } else {
                if !self.allocator.allocate_specific(desired) {
                    return Err(RegistrationError::AddressInUse);
                }
            }
            desired.clone()
        } else {
            // Auto-allocation: validate timestamp (no replay check for new address)
            validate_timestamp(reg.timestamp_fs, now_fs, None)?;

            self.allocator
                .allocate()
                .ok_or(RegistrationError::AddressSpaceExhausted)?
        };

        // Step 2: Verify the TL-DSA signature
        // The signature covers: domain ‖ address ‖ endpoint ‖ pk ‖ kem_pk ‖ timestamp
        verify_registration_signature(reg, &addr)?;

        // Step 3: Create the signed registry record
        let record = CubeRecord {
            addr: addr.clone(),
            endpoints: vec![reg.endpoint],
            public_key: reg.public_key.clone(),
            kem_public_key: reg.kem_public_key.clone(),
            status: CubeStatus::Active,
            last_heartbeat: now,
            registered_at: now,
            registered_at_fs: reg.timestamp_fs,
            reg_signature: Some(reg.signature.clone()),
            legacy_key: false,
            level: self.level,
        };
        self.registry.insert(addr.clone(), record);

        // Step 4: Compute neighbors
        let neighbors = self.compute_neighbor_info(&addr);

        Ok(RegistrationResult {
            address: addr,
            neighbors,
        })
    }

    // ═══════════════════════════════════════════════════════════════
    // NEIGHBOR COMPUTATION
    // ═══════════════════════════════════════════════════════════════

    /// Compute neighbor info for a cube address.
    ///
    /// This is the pure math part: flip each of the 13 trits to its
    /// 2 alternative values → 26 neighbors. Then look up endpoints.
    /// Computed on every call — not stored.
    ///
    /// T-06 addition: includes `reg_signature` and `kem_public_key`
    /// in the neighbor info for T-07 neighbor-side verification.
    pub fn compute_neighbor_info(&self, addr: &CubeAddr) -> Vec<NeighborInfo> {
        let mut neighbors = Vec::with_capacity(NEIGHBORS_PER_CUBE);
        for dim in 0..DIMENSIONS {
            for alt in addr.trit(dim).alternatives() {
                let mut nbr_addr = addr.clone();
                nbr_addr.set_trit(dim, alt);

                let (endpoint, public_key, status, reg_sig, kem_pk) =
                    if let Some(record) = self.registry.get(&nbr_addr) {
                        (
                            record.endpoints.first().copied(),
                            Some(record.public_key.clone()),
                            Some(record.status),
                            record.reg_signature.clone(),
                            record.kem_public_key.clone(),
                        )
                    } else {
                        (None, None, None, None, None)
                    };

                neighbors.push(NeighborInfo {
                    addr: nbr_addr,
                    endpoint,
                    public_key,
                    status,
                    reg_signature: reg_sig,
                    kem_public_key: kem_pk,
                });
            }
        }
        neighbors
    }

    // ═══════════════════════════════════════════════════════════════
    // LOOKUP
    // ═══════════════════════════════════════════════════════════════

    /// Look up a cube's registration record.
    pub fn lookup(&self, addr: &CubeAddr) -> Option<&CubeRecord> {
        self.registry.get(addr)
    }

    /// Look up just the endpoint for a cube address.
    pub fn lookup_endpoint(&self, addr: &CubeAddr) -> Option<SocketAddr> {
        self.registry
            .get(addr)
            .and_then(|r| r.endpoints.first().copied())
    }

    // ═══════════════════════════════════════════════════════════════
    // HEARTBEAT
    // ═══════════════════════════════════════════════════════════════

    /// Process a heartbeat from a registered cube.
    pub fn heartbeat(&mut self, addr: &CubeAddr, endpoint: SocketAddr) -> bool {
        if let Some(record) = self.registry.get_mut(addr) {
            record.last_heartbeat = Instant::now();
            // Update endpoint if it changed (mobile cubes)
            if !record.endpoints.contains(&endpoint) {
                record.endpoints.push(endpoint);
                // Keep only the 3 most recent endpoints
                if record.endpoints.len() > 3 {
                    record.endpoints.remove(0);
                }
            }
            if record.status == CubeStatus::Offline {
                record.status = CubeStatus::Active; // Recovery
            }
            true
        } else {
            false // Unknown cube
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // DEREGISTRATION
    // ═══════════════════════════════════════════════════════════════

    /// Deregister a cube. The address enters a grace period before reuse.
    pub fn deregister(&mut self, addr: &CubeAddr) -> bool {
        if self.registry.remove(addr).is_some() {
            self.allocator.release(addr);
            true
        } else {
            false
        }
    }

    /// Mark a cube as draining (graceful shutdown).
    pub fn drain(&mut self, addr: &CubeAddr) -> bool {
        if let Some(record) = self.registry.get_mut(addr) {
            record.status = CubeStatus::Draining;
            true
        } else {
            false
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // HEALTH CHECK — Detect offline cubes
    // ═══════════════════════════════════════════════════════════════

    /// Scan all registered cubes and mark those with expired heartbeats
    /// as offline. Returns the list of newly offline cubes.
    pub fn check_heartbeats(&mut self) -> Vec<CubeAddr> {
        let now = Instant::now();
        let threshold = self.offline_threshold;
        let mut newly_offline = Vec::new();

        for (addr, record) in self.registry.iter_mut() {
            if record.status == CubeStatus::Active
                && now.duration_since(record.last_heartbeat) > threshold
            {
                record.status = CubeStatus::Offline;
                newly_offline.push(addr.clone());
            }
        }

        newly_offline
    }

    // ═══════════════════════════════════════════════════════════════
    // STATISTICS
    pub fn update_public_key(
        &mut self,
        addr: &CubeAddr,
        new_public_key: Vec<u8>,
    ) -> Result<(), RegistrationError> {
        match self.registry.get_mut(addr) {
            Some(record) => {
                record.public_key = new_public_key;
                record.last_heartbeat = std::time::Instant::now();
                Ok(())
            }
            None => Err(RegistrationError::AddressNotFound),
        }
    }

    // ═══════════════════════════════════════════════════════════════

    /// Number of registered cubes.
    pub fn registered_count(&self) -> usize {
        self.registry.len()
    }

    /// Number of available addresses.
    pub fn available_addresses(&self) -> u64 {
        self.allocator.available()
    }

    /// Number of active cubes.
    pub fn active_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.status == CubeStatus::Active)
            .count()
    }

    /// Number of offline cubes.
    pub fn offline_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.status == CubeStatus::Offline)
            .count()
    }

    /// Number of signed registrations (non-legacy).
    pub fn signed_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.reg_signature.is_some())
            .count()
    }

    /// Number of legacy unsigned registrations.
    pub fn legacy_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.legacy_key)
            .count()
    }

    /// Get all registered cube addresses.
    pub fn all_addresses(&self) -> Vec<&CubeAddr> {
        self.registry.keys().collect()
    }

    /// The hierarchy level this CRS manages.
    pub fn level(&self) -> usize {
        self.level
    }
}

impl Default for CubeRegistrationService {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SERVICE SLOT REGISTRY (V3 — Array3 Node Cluster)
// ═══════════════════════════════════════════════════════════════════════

/// 3-trit Rep C slot address within a node's 27-slot cube.
/// Index 0 = plane (1=Data, 2=Control, 3=Management),
/// 1 = role, 2 = instance.
pub type SlotAddr = [u8; 3];

/// Registered service slot with identity, capabilities, and port.
#[derive(Debug, Clone)]
pub struct ServiceSlot {
    pub slot_addr: SlotAddr,
    pub node_id: u8,         // Rep C {1, 2, 3}
    pub port: u16,           // Computed from port formula
    pub identity: Vec<u8>,   // TL-DSA public key (32 bytes)
    pub capabilities: [bool; DIMENSIONS], // 13 capability flags
    pub classification: [u8; 27],        // Original 27-trit input
    pub registered_at: Instant,
}

/// Error during service slot registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotError {
    ZeroNodeId,
    NodeIdOutOfRange(u8),
    InvalidClassification,
    SlotOccupied(SlotAddr),
    ProjectionFailed,
}

impl std::fmt::Display for SlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotError::ZeroNodeId =>
                write!(f, "FATAL: node_id=0 — zero-sentinel forgery"),
            SlotError::NodeIdOutOfRange(id) =>
                write!(f, "FATAL: node_id={} exceeds MAX_NODES=3", id),
            SlotError::InvalidClassification =>
                write!(f, "classification contains non-Rep-C values"),
            SlotError::SlotOccupied(addr) =>
                write!(f, "slot ({},{},{}) already occupied", addr[0], addr[1], addr[2]),
            SlotError::ProjectionFailed =>
                write!(f, "27→3 projection failed"),
        }
    }
}

/// Result of slot-to-node routing resolution.
#[derive(Debug, Clone)]
pub struct SlotRoute {
    pub node_id: u8,
    pub port: u16,
    pub identity: Vec<u8>,
}

/// Registry for service slots within an Array3 cluster.
///
/// Manages the 81-slot space (3 nodes × 27 slots each).
/// Provides register_service() (T-14) and resolve_slot() (T-15).
pub struct ServiceSlotRegistry {
    slots: HashMap<(u8, SlotAddr), ServiceSlot>,
}

impl ServiceSlotRegistry {
    pub fn new() -> Self {
        ServiceSlotRegistry {
            slots: HashMap::new(),
        }
    }

    /// Register a service: 27 classification trits → ServiceSlot.
    ///
    /// Performs 27→3 projection, computes port, stores identity and capabilities.
    /// Returns the computed ServiceSlot on success.
    pub fn register_service(
        &mut self,
        node_id: u8,
        classification: &[u8; 27],
        identity: Vec<u8>,
        capabilities: [bool; DIMENSIONS],
    ) -> Result<ServiceSlot, SlotError> {
        if node_id == 0 {
            return Err(SlotError::ZeroNodeId);
        }
        if node_id > 3 {
            return Err(SlotError::NodeIdOutOfRange(node_id));
        }

        // Validate all trits are Rep C {1, 2, 3}
        for &t in classification.iter() {
            if t < 1 || t > 3 {
                return Err(SlotError::InvalidClassification);
            }
        }

        // 27→3 projection using same GF(3) quantization as plenumlan
        let slot_addr = self.project_27_to_3(classification)
            .ok_or(SlotError::ProjectionFailed)?;

        // Check for conflicts
        let key = (node_id, slot_addr);
        if self.slots.contains_key(&key) {
            return Err(SlotError::SlotOccupied(slot_addr));
        }

        // Compute port: BASE_PORT + (node_id-1)*27 + offset
        let base_port: u16 = 11111;
        let offset = ((slot_addr[0] as u16 - 1) * 9)
            + ((slot_addr[1] as u16 - 1) * 3)
            + (slot_addr[2] as u16 - 1);
        let port = base_port + ((node_id as u16 - 1) * 27) + offset;

        let slot = ServiceSlot {
            slot_addr,
            node_id,
            port,
            identity,
            capabilities,
            classification: *classification,
            registered_at: Instant::now(),
        };

        self.slots.insert(key, slot.clone());
        Ok(slot)
    }

    /// Resolve a 3-trit slot address to the owning node and port (T-15).
    ///
    /// Searches across all nodes for a registered service at the given slot.
    pub fn resolve_slot(&self, slot_addr: &SlotAddr) -> Option<SlotRoute> {
        for node_id in 1..=3u8 {
            if let Some(slot) = self.slots.get(&(node_id, *slot_addr)) {
                return Some(SlotRoute {
                    node_id: slot.node_id,
                    port: slot.port,
                    identity: slot.identity.clone(),
                });
            }
        }
        None
    }

    /// Number of registered service slots across all nodes.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// 27→3 projection (mirrors plenumlan/src/cube/projection.rs).
    ///
    /// Uses the same polarity tables and GF(3) quantization.
    fn project_27_to_3(&self, classification: &[u8; 27]) -> Option<SlotAddr> {
        // Polarity tables (same as Rust plenumlan)
        // Plane: D1+, D2−, D3+, D9−, D10+, D17+, D19+, D25+, D26+
        let plane_dims: [(usize, bool); 9] = [
            (0, true), (1, false), (2, true), (8, false), (9, true),
            (16, true), (18, true), (24, true), (25, true),
        ];
        // Role: D5+, D6+, D7+, D8+, D12+, D18+, D22−, D23+, D24−
        let role_dims: [(usize, bool); 9] = [
            (4, true), (5, true), (6, true), (7, true), (11, true),
            (17, true), (21, false), (22, true), (23, false),
        ];
        // Instance: D4+, D11+, D13+, D14+, D15+, D16+, D20+, D21+, D27+
        let inst_dims: [(usize, bool); 9] = [
            (3, true), (10, true), (12, true), (13, true), (14, true),
            (15, true), (19, true), (20, true), (26, true),
        ];

        fn count_high(class: &[u8; 27], dims: &[(usize, bool); 9]) -> u64 {
            let mut k = 0u64;
            for &(idx, positive) in dims {
                let raw = class[idx];
                let adjusted = if positive { raw } else {
                    match raw { 1 => 3, 3 => 1, _ => 2 }
                };
                if adjusted == 3 { k += 1; }
            }
            k
        }

        fn project_to_gf3(k: u64, n: u64) -> u8 {
            std::cmp::min((3 * k / n) as u8, 2)
        }

        let plane_k = count_high(classification, &plane_dims);
        let role_k = count_high(classification, &role_dims);
        let inst_k = count_high(classification, &inst_dims);

        let n = 9u64; // DIMS_PER_GROUP

        let plane = project_to_gf3(plane_k, n) + 1;
        let role = project_to_gf3(role_k, n) + 1;
        let instance = project_to_gf3(inst_k, n) + 1;

        Some([plane, role, instance])
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

    fn test_endpoint() -> SocketAddr {
        "127.0.0.1:51820".parse().unwrap()
    }

    fn test_key() -> Vec<u8> {
        vec![0xAB; 32]
    }

    // ── Legacy (unsigned) registration tests ────────────────────

    #[test]
    fn test_register_auto_address() {
        let mut crs = CubeRegistrationService::new();
        let result = crs.register(test_endpoint(), test_key(), None).unwrap();
        let bytes = result.address.to_bytes();
        for &b in &bytes {
            assert!(b >= 1 && b <= 3, "Address must be Rep C");
        }
        assert_eq!(result.neighbors.len(), NEIGHBORS_PER_CUBE);
    }

    #[test]
    fn test_register_specific_address() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]);
        let result = crs
            .register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        assert_eq!(result.address, desired);
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        let result = crs.register(test_endpoint(), test_key(), Some(desired));
        assert_eq!(result.unwrap_err(), RegistrationError::AddressInUse);
    }

    #[test]
    fn test_neighbor_info_includes_registered() {
        let mut crs = CubeRegistrationService::new();
        let addr_a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        crs.register(
            "10.0.0.1:51820".parse().unwrap(),
            vec![0x11; 32],
            Some(addr_a.clone()),
        )
        .unwrap();

        let addr_b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let result = crs
            .register(
                "10.0.0.2:51820".parse().unwrap(),
                vec![0x22; 32],
                Some(addr_b.clone()),
            )
            .unwrap();

        let a_info = result.neighbors.iter().find(|n| n.addr == addr_a).unwrap();
        assert!(a_info.endpoint.is_some());
        assert_eq!(a_info.status, Some(CubeStatus::Active));
    }

    #[test]
    fn test_lookup() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();

        let record = crs.lookup(&desired).unwrap();
        assert_eq!(record.status, CubeStatus::Active);
        assert_eq!(record.public_key, test_key());
        assert!(record.reg_signature.is_none(), "Legacy registration has no signature");
        assert!(record.legacy_key, "Legacy registration is marked legacy");
    }

    #[test]
    fn test_heartbeat() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        assert!(crs.heartbeat(&desired, test_endpoint()));
        let unknown = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        assert!(!crs.heartbeat(&unknown, test_endpoint()));
    }

    #[test]
    fn test_deregister() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        assert_eq!(crs.registered_count(), 1);
        assert!(crs.deregister(&desired));
        assert_eq!(crs.registered_count(), 0);
    }

    #[test]
    fn test_multiple_registrations() {
        let mut crs = CubeRegistrationService::new();
        for i in 0u8..10 {
            crs.register(
                format!("10.0.0.{}:51820", i).parse().unwrap(),
                vec![i; 32],
                None,
            )
            .unwrap();
        }
        assert_eq!(crs.registered_count(), 10);
        assert_eq!(crs.active_count(), 10);
        assert_eq!(crs.legacy_count(), 10);
    }

    #[test]
    fn test_address_never_contains_zero() {
        let mut crs = CubeRegistrationService::new();
        for i in 0..100 {
            let result = crs
                .register(
                    format!("10.0.0.{}:{}", i % 256, 51820 + i / 256)
                        .parse()
                        .unwrap(),
                    vec![i as u8; 32],
                    None,
                )
                .unwrap();
            for b in result.address.to_bytes() {
                assert!(b >= 1 && b <= 3);
            }
        }
    }

    #[test]
    fn test_recursive_levels() {
        let root_crs = CubeRegistrationService::new();
        let inner_crs = CubeRegistrationService::new().at_level(1);
        assert_eq!(root_crs.level(), 0);
        assert_eq!(inner_crs.level(), 1);
    }

    // ── Signed registration tests (T-06) ────────────────────────

    #[test]
    fn test_signed_registration_valid() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-for-signed-crs-registration";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let desired = addr([2, 1, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let endpoint: SocketAddr = "10.0.0.1:51820".parse().unwrap();
        let now_fs: u128 = 100 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let msg = build_registration_message(&desired, &endpoint, &pk, None, now_fs);
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &msg, variant);

        let reg = SignedRegistration {
            address: Some(desired.clone()),
            endpoint,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig,
        };

        let result = crs.register_signed(&reg, now_fs).unwrap();
        assert_eq!(result.address, desired);

        let record = crs.lookup(&desired).unwrap();
        assert!(record.reg_signature.is_some());
        assert!(!record.legacy_key);
        assert_eq!(record.registered_at_fs, now_fs);
    }

    #[test]
    fn test_signed_registration_wrong_signature() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-wrong-sig";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let desired = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let endpoint: SocketAddr = "10.0.0.2:51820".parse().unwrap();
        let now_fs: u128 = 100 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let wrong_msg = build_registration_message(
            &desired,
            &"10.0.0.99:9999".parse().unwrap(),
            &pk,
            None,
            now_fs,
        );
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &wrong_msg, variant);

        let reg = SignedRegistration {
            address: Some(desired),
            endpoint,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig,
        };

        let err = crs.register_signed(&reg, now_fs).unwrap_err();
        assert_eq!(err, RegistrationError::InvalidSignature);
    }

    #[test]
    fn test_signed_registration_stale_timestamp() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-stale";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let desired = addr([3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3]);
        let endpoint: SocketAddr = "10.0.0.3:51820".parse().unwrap();
        let now_fs: u128 = 1000 * crate::wire::FS_PER_SECOND;
        let old_ts = now_fs - 60 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let msg = build_registration_message(&desired, &endpoint, &pk, None, old_ts);
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &msg, variant);

        let reg = SignedRegistration {
            address: Some(desired),
            endpoint,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: old_ts,
            signature: sig,
        };

        let err = crs.register_signed(&reg, now_fs).unwrap_err();
        assert_eq!(err, RegistrationError::StaleTimestamp);
    }

    #[test]
    fn test_signed_registration_replay_detected() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-replay";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let desired = addr([1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1]);
        let endpoint: SocketAddr = "10.0.0.4:51820".parse().unwrap();
        let now_fs: u128 = 500 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let msg1 = build_registration_message(&desired, &endpoint, &pk, None, now_fs);
        let sig1 = ternary_math::tl_dsa::sign(&kp.secret_key, &msg1, variant);

        let reg1 = SignedRegistration {
            address: Some(desired.clone()),
            endpoint,
            public_key: pk.clone(),
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig1,
        };
        crs.register_signed(&reg1, now_fs).unwrap();

        let msg2 = build_registration_message(&desired, &endpoint, &pk, None, now_fs);
        let sig2 = ternary_math::tl_dsa::sign(&kp.secret_key, &msg2, variant);

        let reg2 = SignedRegistration {
            address: Some(desired),
            endpoint,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig2,
        };

        let err = crs.register_signed(&reg2, now_fs).unwrap_err();
        assert_eq!(err, RegistrationError::ReplayDetected);
    }

    #[test]
    fn test_signed_re_registration_newer_timestamp() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-rereg";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let desired = addr([2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2]);
        let endpoint1: SocketAddr = "10.0.0.5:51820".parse().unwrap();
        let endpoint2: SocketAddr = "10.0.0.6:51820".parse().unwrap();
        let ts1: u128 = 500 * crate::wire::FS_PER_SECOND;
        let ts2: u128 = 510 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let msg1 = build_registration_message(&desired, &endpoint1, &pk, None, ts1);
        let sig1 = ternary_math::tl_dsa::sign(&kp.secret_key, &msg1, variant);
        let reg1 = SignedRegistration {
            address: Some(desired.clone()),
            endpoint: endpoint1,
            public_key: pk.clone(),
            kem_public_key: None,
            timestamp_fs: ts1,
            signature: sig1,
        };
        crs.register_signed(&reg1, ts1).unwrap();

        let msg2 = build_registration_message(&desired, &endpoint2, &pk, None, ts2);
        let sig2 = ternary_math::tl_dsa::sign(&kp.secret_key, &msg2, variant);
        let reg2 = SignedRegistration {
            address: Some(desired.clone()),
            endpoint: endpoint2,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: ts2,
            signature: sig2,
        };
        let result = crs.register_signed(&reg2, ts2).unwrap();
        assert_eq!(result.address, desired);

        let record = crs.lookup(&desired).unwrap();
        assert_eq!(record.endpoints[0], endpoint2);
        assert_eq!(record.registered_at_fs, ts2);
    }

    #[test]
    fn test_signed_registration_wrong_key() {
        let mut crs = CubeRegistrationService::new();

        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp_signer = ternary_math::tl_dsa::keygen(variant, Some(b"signer-seed"));
        let kp_imposter = ternary_math::tl_dsa::keygen(variant, Some(b"imposter-seed"));

        let desired = addr([3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3]);
        let endpoint: SocketAddr = "10.0.0.7:51820".parse().unwrap();
        let now_fs: u128 = 200 * crate::wire::FS_PER_SECOND;

        let pk_imposter = kp_imposter.public_key.clone();

        let msg = build_registration_message(&desired, &endpoint, &pk_imposter, None, now_fs);
        let sig = ternary_math::tl_dsa::sign(&kp_signer.secret_key, &msg, variant);

        let reg = SignedRegistration {
            address: Some(desired),
            endpoint,
            public_key: pk_imposter,
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig,
        };

        let err = crs.register_signed(&reg, now_fs).unwrap_err();
        assert_eq!(err, RegistrationError::InvalidSignature);
    }

    #[test]
    fn test_neighbor_info_includes_signature() {
        let mut crs = CubeRegistrationService::new();

        let seed = b"test-seed-nbr-sig";
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(seed));

        let addr_a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let endpoint: SocketAddr = "10.0.0.10:51820".parse().unwrap();
        let now_fs: u128 = 300 * crate::wire::FS_PER_SECOND;

        let pk = kp.public_key.clone();

        let msg = build_registration_message(&addr_a, &endpoint, &pk, None, now_fs);
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &msg, variant);

        let reg = SignedRegistration {
            address: Some(addr_a.clone()),
            endpoint,
            public_key: pk,
            kem_public_key: None,
            timestamp_fs: now_fs,
            signature: sig,
        };
        crs.register_signed(&reg, now_fs).unwrap();

        let addr_b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        crs.register(
            "10.0.0.11:51820".parse().unwrap(),
            vec![0x22; 32],
            Some(addr_b.clone()),
        )
        .unwrap();

        let nbrs = crs.compute_neighbor_info(&addr_b);
        let a_info = nbrs.iter().find(|n| n.addr == addr_a).unwrap();
        assert!(
            a_info.reg_signature.is_some(),
            "Neighbor info must include the registration signature for T-07 verification"
        );
    }

    // ── Timestamp validation unit tests ─────────────────────────

    #[test]
    fn test_validate_timestamp_current() {
        let now = 1000 * crate::wire::FS_PER_SECOND;
        assert!(validate_timestamp(now, now, None).is_ok());
    }

    #[test]
    fn test_validate_timestamp_slightly_old() {
        let now = 1000 * crate::wire::FS_PER_SECOND;
        let ts = now - 10 * crate::wire::FS_PER_SECOND;
        assert!(validate_timestamp(ts, now, None).is_ok());
    }

    #[test]
    fn test_validate_timestamp_too_old() {
        let now = 1000 * crate::wire::FS_PER_SECOND;
        let ts = now - 60 * crate::wire::FS_PER_SECOND;
        assert_eq!(validate_timestamp(ts, now, None).unwrap_err(), RegistrationError::StaleTimestamp);
    }

    #[test]
    fn test_validate_timestamp_replay() {
        let now = 1000 * crate::wire::FS_PER_SECOND;
        let existing = now - 5 * crate::wire::FS_PER_SECOND;
        assert_eq!(
            validate_timestamp(existing, now, Some(existing)).unwrap_err(),
            RegistrationError::ReplayDetected
        );
    }

    #[test]
    fn test_validate_timestamp_newer_than_existing() {
        let now = 1000 * crate::wire::FS_PER_SECOND;
        let existing = now - 10 * crate::wire::FS_PER_SECOND;
        let newer = now - 5 * crate::wire::FS_PER_SECOND;
        assert!(validate_timestamp(newer, now, Some(existing)).is_ok());
    }

    // ── Statistics tests ────────────────────────────────────────

    #[test]
    fn test_signed_vs_legacy_counts() {
        let mut crs = CubeRegistrationService::new();

        for i in 0..3 {
            crs.register(
                format!("10.0.0.{}:51820", i).parse().unwrap(),
                vec![i as u8; 32],
                None,
            )
            .unwrap();
        }

        assert_eq!(crs.registered_count(), 3);
        assert_eq!(crs.legacy_count(), 3);
        assert_eq!(crs.signed_count(), 0);
    }

    // ── Service Slot tests (V3) ─────────────────────────────────

    #[test]
    fn test_service_slot_registration() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27]; // center classification
        let identity = vec![0xABu8; 32];
        let capabilities = [false; 13];

        let result = registry.register_service(1, &classification, identity.clone(), capabilities);
        assert!(result.is_ok());
        let slot = result.unwrap();
        assert!(slot.port >= 11111 && slot.port <= 11137);
    }

    #[test]
    fn test_service_slot_rejects_zero_node_id() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27];
        let result = registry.register_service(0, &classification, vec![0xAB; 32], [false; 13]);
        assert!(result.is_err());
    }

    #[test]
    fn test_slot_to_node_routing() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27];
        let identity = vec![0xABu8; 32];
        let slot = registry.register_service(1, &classification, identity.clone(), [false; 13]).unwrap();

        let route = registry.resolve_slot(&slot.slot_addr);
        assert!(route.is_some());
        let route = route.unwrap();
        assert_eq!(route.node_id, 1);
        assert_eq!(route.port, slot.port);
        assert_eq!(route.identity, identity);
    }

    #[test]
    fn test_slot_registry_duplicate_rejected() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27];
        registry.register_service(1, &classification, vec![0xAB; 32], [false; 13]).unwrap();
        let result = registry.register_service(1, &classification, vec![0xCD; 32], [false; 13]);
        assert!(matches!(result, Err(SlotError::SlotOccupied(_))));
    }

    #[test]
    fn test_slot_registry_same_slot_different_nodes() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27];
        registry.register_service(1, &classification, vec![0xAB; 32], [false; 13]).unwrap();
        let result = registry.register_service(2, &classification, vec![0xCD; 32], [false; 13]);
        assert!(result.is_ok());
        assert_eq!(registry.slot_count(), 2);
    }

    #[test]
    fn test_slot_registry_node_id_out_of_range() {
        let mut registry = ServiceSlotRegistry::new();
        let classification = [2u8; 27];
        let result = registry.register_service(4, &classification, vec![0xAB; 32], [false; 13]);
        assert_eq!(result.unwrap_err(), SlotError::NodeIdOutOfRange(4));
    }

    #[test]
    fn test_slot_registry_invalid_classification() {
        let mut registry = ServiceSlotRegistry::new();
        let mut classification = [2u8; 27];
        classification[0] = 0; // zero = Rep C violation
        let result = registry.register_service(1, &classification, vec![0xAB; 32], [false; 13]);
        assert_eq!(result.unwrap_err(), SlotError::InvalidClassification);
    }

    #[test]
    fn test_slot_port_ranges_by_node() {
        let mut registry = ServiceSlotRegistry::new();
        let c1 = [1u8; 27]; // all-1s classification
        let c2 = [3u8; 27]; // all-3s classification

        let s1 = registry.register_service(1, &c1, vec![0x11; 32], [false; 13]).unwrap();
        assert!(s1.port >= 11111 && s1.port <= 11137, "Node 1 port {} out of range", s1.port);

        let s2 = registry.register_service(2, &c2, vec![0x22; 32], [false; 13]).unwrap();
        assert!(s2.port >= 11138 && s2.port <= 11164, "Node 2 port {} out of range", s2.port);

        let s3_class = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3];
        let s3 = registry.register_service(3, &s3_class, vec![0x33; 32], [false; 13]).unwrap();
        assert!(s3.port >= 11165 && s3.port <= 11191, "Node 3 port {} out of range", s3.port);
    }

    #[test]
    fn test_resolve_slot_not_found() {
        let registry = ServiceSlotRegistry::new();
        assert!(registry.resolve_slot(&[1, 1, 1]).is_none());
    }
}
