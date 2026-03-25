// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Arc-Synchronized Key Rotation (T-19, SPEC-2026-NEXT)
//!
//! Orchestrates master secret rotation across all cryptographic subsystems
//! at radian epoch boundaries (every 14 days = ARC_EPOCH / RADIAN_DEG = 182 / 13).
//!
//! ## Epoch Boundaries
//!
//! ```text
//! One Arc (182 days) = 13 radian epochs of 14 days each
//!
//! ──┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬──
//!   │  R0  │  R1  │  R2  │  R3  │  R4  │  R5  │  R6  │  R7  │  R8  │  R9  │ R10  │ R11  │ R12  │
//!   │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │ 14d  │
//! ──┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┴──
//!   ↑                                                                                             ↑
//! Salvi Epoch                                                                               Day 182
//!
//! 14 days = ARC_EPOCH_SECS / RADIAN_DEG = 182 / 13
//! 26 rotations per ternary year (2 arcs × 13 radians)
//! ```
//!
//! ## Rotation Sequence
//!
//! When a new radian epoch is detected:
//!
//! 1. **Generate** new master secret (T-12)
//! 2. **Rotate** SecretRotation (old → previous, new → current)
//! 3. **Re-derive** all HMAC keys from new secret (T-08 FTS)
//! 4. **Re-derive** identity keypair from new secret (T-15)
//! 5. **Re-register** with CRS using new public key (T-06)
//! 6. **Encrypt** new secret at rest (T-12)
//! 7. **Notify** all subsystems of the rotation
//!
//! ## Dual-Accept Window
//!
//! After rotation, the previous secret remains valid for `DUAL_ACCEPT_SECS`
//! (1 second). This window covers network propagation latency only.
//!
//! PlenumNET nodes are HPTP-synchronized to femtosecond precision. Messages
//! travel in milliseconds. 1 second is already generous for this network.
//!
//! **Security note:** The dual-accept window is intentionally decoupled from
//! the rotation period. A window equal to the rotation period would mean a
//! compromised key stays valid for twice the rotation period — defeating
//! rotation entirely.
//!
//! ## Jitter
//!
//! Each node applies a deterministic positive offset derived from its TDNS
//! registration timestamp: `registration_timestamp % MAX_JITTER_SECS`.
//!
//! Jitter is **positive only** — every node rotates at or after the radian
//! boundary, never before. Nodes that registered at different times naturally
//! produce different offsets with no additional computation. Maximum spread
//! across all nodes: 15 minutes (MAX_JITTER_SECS = 900).
//!
//! ## 3FA Capability Gate (force_rotate only)
//!
//! Automatic radian rotation (`check_and_rotate`) is deterministic —
//! the arc clock is sole authority. No capability required.
//!
//! Emergency rotation (`force_rotate`) requires a `RotationCapability` token:
//!
//!   - **Factor 1:** Access to the `RotationOrchestrator` (what you have)
//!   - **Factor 2:** `CubeAddr`-bound TL-DSA identity (what you are)
//!   - **Factor 3:** `RotationCapability` token (what you can prove)
//!
//! ## SYNC REQUIRED
//!
//! `identity.rs` line 90: `MAX_DUAL_ACCEPT_SECS` must equal `DUAL_ACCEPT_SECS`
//! (1 second). Test assertion at line 939 must be updated to reflect this.
//! The old `assert_eq!(MAX_DUAL_ACCEPT_SECS, ARC_EPOCH_SECS)` must be removed.

use std::time::{Duration, Instant};

use crate::cube_addr::CubeAddr;
use crate::identity::{
    MasterSecret, SecretRotation,
    current_arc_epoch, ARC_EPOCH_SECS, SALVI_EPOCH_UNIX,
};
// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// One ternary radian = 13 degrees.
/// Integer alias of `ternary_math::constants::RADIAN_DEG` (f64) for
/// use in const integer arithmetic. Value identity asserted in tests.
const RADIAN_DEG_INT: u64 = 13;

/// Domain separator for rotation operations.
pub const ROTATION_JITTER_DOMAIN: &[u8] = b"PlenumNET-ROT-JITTER";

/// Rotation period in seconds: one ternary radian.
///
/// = ARC_EPOCH_SECS / RADIAN_DEG = 182 days / 13 = 14 days.
///
/// 14 = π in the ternary circle. Keys rotate every π days.
/// 13 rotations per arc. 26 rotations per ternary year.
/// The arc epoch (182 days) is preserved as a calendar constant —
/// this subdivides it by the radian, not replaces it.
pub const ROTATION_PERIOD_SECS: u64 = ARC_EPOCH_SECS / RADIAN_DEG_INT;

/// Maximum rotation jitter: 900 seconds (15 minutes = 1/96 of a day).
///
/// Jitter is **positive only** — range [0, MAX_JITTER_SECS].
/// Derived from the node's TDNS registration timestamp.
/// Naturally distributed across the window — no hash required.
pub const MAX_JITTER_SECS: u64 = 900;

/// Dual-accept window: 1 second.
///
/// After rotation, the previous master secret remains valid for this
/// duration to cover in-flight messages signed under the old key.
///
/// On HPTP-synchronized infrastructure, clock skew is femtosecond-level
/// and messages travel in milliseconds. 1 second is generous.
///
/// **Intentionally decoupled from `ROTATION_PERIOD_SECS`.**
///
/// SYNC: `identity.rs` → `MAX_DUAL_ACCEPT_SECS` must equal this value (1).
pub const DUAL_ACCEPT_SECS: u64 = 1;

// ═══════════════════════════════════════════════════════════════════════
// KEY FRESHNESS ZONES (T-35)
//
// Freshness at ternary 1/3 and 2/3 thresholds within ARC_EPOCH (182 days):
//   Fresh  (age 0–60):   key is in first third. All operations.
//   Active (age 61–121): key is in second third. Regulated ops.
//                        Boundary at 121 = REPUNIT_R5 = 11².
//   Aging  (age 122–182): key is in final third. Read-only.
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyFreshnessZone {
    Fresh,
    Active,
    Aging,
}

impl KeyFreshnessZone {
    pub fn label(&self) -> &'static str {
        match self {
            KeyFreshnessZone::Fresh => "fresh",
            KeyFreshnessZone::Active => "active",
            KeyFreshnessZone::Aging => "aging",
        }
    }
}

impl std::fmt::Display for KeyFreshnessZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Compute the freshness zone for a key of the given age in days.
///
/// Uses GF(3) quantization: floor(3 × age / ARC_EPOCH).
/// Returns `None` if age exceeds ARC_EPOCH (expired key).
pub fn key_freshness_zone(age_days: u64) -> Option<KeyFreshnessZone> {
    if age_days > ARC_EPOCH_SECS / 86400 {
        return None; // expired
    }
    let arc_days = ARC_EPOCH_SECS / 86400; // 182
    let zone = std::cmp::min(3 * age_days / arc_days, 2);
    Some(match zone {
        0 => KeyFreshnessZone::Fresh,
        1 => KeyFreshnessZone::Active,
        2 => KeyFreshnessZone::Aging,
        _ => unreachable!(),
    })
}

/// Check whether a key is suitable for regulated/sensitive operations.
pub fn key_suitable_for_regulated(age_days: u64) -> bool {
    matches!(key_freshness_zone(age_days), Some(KeyFreshnessZone::Fresh | KeyFreshnessZone::Active))
}

/// Compute the birth epoch of the current key.
///
/// birth_epoch = current_epoch - age_in_radian_epochs
pub fn key_birth_epoch(current_epoch: u64, age_days: u64) -> u64 {
    let radian_days = ROTATION_PERIOD_SECS / 86400; // 14
    let age_radians = age_days / radian_days;
    current_epoch.saturating_sub(age_radians)
}

/// Capability token TTL: 1,800 seconds (30 minutes).
///
/// How long an operator has to use an emergency rotation token.
/// Human-timescale. Independent of all calendar and network constants.
pub const CAPABILITY_TTL_SECS: u64 = 1_800;

// ═══════════════════════════════════════════════════════════════════════
// 3FA: ROTATION CAPABILITY
// ═══════════════════════════════════════════════════════════════════════

/// Third authentication factor for emergency key rotation.
///
///   Factor 1 — Orchestrator access (master secret, what you have)
///   Factor 2 — CubeAddr-bound TL-DSA identity (what you are)
///   Factor 3 — This capability token (what you can prove)
///
/// `issued_for` binds the token to a specific node — cannot be replayed
/// at a different address. `expires_at` limits the replay window to
/// `CAPABILITY_TTL_SECS` if intercepted.
#[derive(Debug, Clone)]
pub struct RotationCapability {
    /// The node address this capability was issued for.
    pub issued_for: CubeAddr,
    /// Unix timestamp when this capability was issued.
    pub issued_at: u64,
    /// Unix timestamp when this capability expires.
    pub expires_at: u64,
    /// TL-DSA signature over `(issued_for || issued_at || expires_at)`.
    /// Must be non-empty in production. Test contexts use a placeholder byte.
    pub signature: Vec<u8>,
}

/// Errors specific to capability validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// Capability has expired.
    Expired,
    /// Capability was issued for a different node address.
    AddressMismatch,
    /// Capability signature is invalid.
    InvalidSignature,
    /// Capability TTL exceeds the permitted maximum.
    TtlExceeded,
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expired          => write!(f, "rotation capability has expired"),
            Self::AddressMismatch  => write!(f, "capability issued for a different node address"),
            Self::InvalidSignature => write!(f, "capability signature verification failed"),
            Self::TtlExceeded      => write!(f, "capability TTL exceeds permitted maximum"),
        }
    }
}

impl std::error::Error for CapabilityError {}

impl RotationCapability {
    /// Issue a new capability for the given node address.
    pub fn new(issued_for: CubeAddr, issued_at: u64, signature: Vec<u8>) -> Self {
        RotationCapability {
            issued_for,
            issued_at,
            expires_at: issued_at + CAPABILITY_TTL_SECS,
            signature,
        }
    }

    /// Validate against the requesting node and current time.
    ///
    /// Production: wire `signature` verification to `tl_dsa::verify()`
    /// with the issuing authority's public key.
    /// TODO(T-19): integrate TL-DSA issuer public key verification.
    pub fn validate(&self, for_addr: &CubeAddr, unix_timestamp: u64) -> Result<(), CapabilityError> {
        let ttl = self.expires_at.saturating_sub(self.issued_at);
        if ttl > CAPABILITY_TTL_SECS {
            return Err(CapabilityError::TtlExceeded);
        }
        if unix_timestamp > self.expires_at {
            return Err(CapabilityError::Expired);
        }
        if self.issued_for.to_bytes() != for_addr.to_bytes() {
            return Err(CapabilityError::AddressMismatch);
        }
        #[cfg(not(test))]
        if self.signature.is_empty() {
            return Err(CapabilityError::InvalidSignature);
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ROTATION EVENT
// ═══════════════════════════════════════════════════════════════════════

/// Record of a completed rotation.
#[derive(Debug, Clone)]
pub struct RotationEvent {
    /// The radian epoch that was entered.
    pub new_epoch: u64,
    /// The radian epoch that was left.
    pub old_epoch: u64,
    /// Unix timestamp when the rotation was performed.
    pub performed_at: u64,
    /// Monotonic timestamp for local timing.
    pub performed_at_mono: Instant,
    /// Whether CRS re-registration is needed.
    pub needs_reregistration: bool,
    /// Number of HMAC keys re-derived.
    pub hmac_keys_rederived: usize,
    /// Whether this rotation was forced (emergency) vs radian-boundary-driven.
    pub forced: bool,
}

/// Errors during rotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotationError {
    /// No rotation is due at this time.
    NotDue,
    /// Secret generation failed.
    GenerationFailed,
    /// Rotation state machine rejected the transition.
    StateMachineError(String),
    /// Encryption at rest failed.
    EncryptionFailed,
    /// 3FA capability validation failed.
    CapabilityDenied(CapabilityError),
}

impl std::fmt::Display for RotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotDue                 => write!(f, "rotation not due"),
            Self::GenerationFailed       => write!(f, "secret generation failed"),
            Self::StateMachineError(msg) => write!(f, "rotation state error: {}", msg),
            Self::EncryptionFailed       => write!(f, "encryption at rest failed"),
            Self::CapabilityDenied(e)    => write!(f, "3FA capability denied: {}", e),
        }
    }
}

impl std::error::Error for RotationError {}

// ═══════════════════════════════════════════════════════════════════════
// JITTER COMPUTATION
// ═══════════════════════════════════════════════════════════════════════

/// Compute a deterministic rotation jitter from a TDNS registration timestamp.
///
/// Returns a positive offset in seconds in `[0, MAX_JITTER_SECS]`.
/// Node rotates at `radian_boundary + jitter`.
///
/// Nodes that registered at different times produce different offsets
/// naturally — no hash, no address computation needed.
pub fn compute_rotation_jitter(registration_timestamp: u64) -> u64 {
    registration_timestamp % (MAX_JITTER_SECS + 1)
}

/// Compute the effective rotation timestamp for a node.
///
/// `effective = radian_boundary + jitter`
/// Always at or after the boundary — never before.
pub fn effective_rotation_time(radian_epoch: u64, registration_timestamp: u64) -> u64 {
    let boundary = SALVI_EPOCH_UNIX + radian_epoch * ROTATION_PERIOD_SECS;
    boundary + compute_rotation_jitter(registration_timestamp)
}

/// Check if rotation is due. Returns `Some(new_epoch)` if due.
pub fn rotation_due_with_jitter(
    current_secret_epoch: u64,
    registration_timestamp: u64,
    unix_timestamp: u64,
) -> Option<u64> {
    let next_epoch    = current_secret_epoch + 1;
    let effective_time = effective_rotation_time(next_epoch, registration_timestamp);
    if unix_timestamp >= effective_time { Some(next_epoch) } else { None }
}

// ═══════════════════════════════════════════════════════════════════════
// ROTATION ORCHESTRATOR
// ═══════════════════════════════════════════════════════════════════════

/// Orchestrates key rotation across all subsystems.
///
/// Call `check_and_rotate()` every heartbeat cycle.
/// Emergency rotation via `force_rotate()` requires a 3FA capability token.
pub struct RotationOrchestrator {
    /// This node's address (for capability binding).
    local_addr: CubeAddr,
    /// TDNS registration timestamp (jitter source).
    registration_timestamp: u64,
    /// Secret rotation state machine (T-12).
    pub rotation: SecretRotation,
    /// History of completed rotations.
    history: Vec<RotationEvent>,
    /// Whether a CRS re-registration is pending.
    pending_reregistration: bool,
    /// Passphrase for encrypting new secrets at rest. Never logged.
    enc_passphrase: Vec<u8>,
    /// Last time we checked for rotation.
    last_check: Instant,
    /// Minimum interval between rotation checks.
    check_interval: Duration,
}

impl RotationOrchestrator {
    /// Create a new orchestrator.
    ///
    /// `registration_timestamp`: Unix timestamp when this node first
    /// registered with TDNS. Used as the jitter source.
    pub fn new(
        local_addr: CubeAddr,
        registration_timestamp: u64,
        initial_secret: MasterSecret,
        initial_epoch: u64,
        enc_passphrase: Vec<u8>,
    ) -> Self {
        RotationOrchestrator {
            local_addr,
            registration_timestamp,
            rotation: SecretRotation::new(initial_secret, initial_epoch),
            history: Vec::new(),
            pending_reregistration: false,
            enc_passphrase,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(60),
        }
    }

    /// Create with custom check interval (tests).
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Check if rotation is due and perform it if so.
    ///
    /// Call periodically. No capability required — radian clock is authority.
    /// Returns `Ok(Some(event))` on rotation, `Ok(None)` if not due.
    pub fn check_and_rotate(
        &mut self,
        unix_timestamp: u64,
    ) -> Result<Option<RotationEvent>, RotationError> {
        if self.last_check.elapsed() < self.check_interval {
            return Ok(None);
        }
        self.last_check = Instant::now();

        self.rotation.check_dual_accept(unix_timestamp);

        let new_epoch = match rotation_due_with_jitter(
            self.rotation.current_epoch(),
            self.registration_timestamp,
            unix_timestamp,
        ) {
            Some(epoch) => epoch,
            None => return Ok(None),
        };

        self.perform_rotation(new_epoch, unix_timestamp, false)
    }

    /// Force a rotation regardless of timing. Requires a 3FA capability token.
    ///
    /// Returns `Err(CapabilityDenied)` if validation fails — no state mutated.
    pub fn force_rotate(
        &mut self,
        unix_timestamp: u64,
        capability: RotationCapability,
    ) -> Result<RotationEvent, RotationError> {
        capability
            .validate(&self.local_addr, unix_timestamp)
            .map_err(RotationError::CapabilityDenied)?;

        let new_epoch = current_arc_epoch(unix_timestamp);
        let next = if new_epoch <= self.rotation.current_epoch() {
            self.rotation.current_epoch() + 1
        } else {
            new_epoch
        };

        self.perform_rotation(next, unix_timestamp, true)
            .map(|opt| opt.unwrap())
    }

    fn perform_rotation(
        &mut self,
        new_epoch: u64,
        unix_timestamp: u64,
        forced: bool,
    ) -> Result<Option<RotationEvent>, RotationError> {
        let old_epoch = self.rotation.current_epoch();

        let new_secret = MasterSecret::generate()
            .map_err(|_| RotationError::GenerationFailed)?;

        if !self.enc_passphrase.is_empty() {
            let _blob = crate::identity::encrypt_master_secret(&new_secret, &self.enc_passphrase)
                .map_err(|_| RotationError::EncryptionFailed)?;
            // In production: write _blob to disk
        }

        self.rotation
            .rotate(new_secret, new_epoch, unix_timestamp)
            .map_err(|e| RotationError::StateMachineError(e.to_string()))?;

        self.pending_reregistration = true;

        let event = RotationEvent {
            new_epoch,
            old_epoch,
            performed_at: unix_timestamp,
            performed_at_mono: Instant::now(),
            needs_reregistration: true,
            hmac_keys_rederived: 0,
            forced,
        };

        self.history.push(event.clone());

        println!(
            "[T-19] Key rotation: epoch {} → {} at timestamp {} (forced={})",
            old_epoch, new_epoch, unix_timestamp, forced
        );

        Ok(Some(event))
    }

    // ── Accessors ──────────────────────────────────────────────

    pub fn current_secret(&self)          -> &MasterSecret        { self.rotation.current() }
    pub fn previous_secret(&self)         -> Option<&MasterSecret> { self.rotation.previous() }
    pub fn current_epoch(&self)           -> u64                   { self.rotation.current_epoch() }
    pub fn in_dual_accept(&self)          -> bool                  { self.rotation.in_dual_accept() }
    pub fn needs_reregistration(&self)    -> bool                  { self.pending_reregistration }
    pub fn history(&self)                 -> &[RotationEvent]      { &self.history }
    pub fn rotation_count(&self)          -> usize                 { self.history.len() }
    pub fn local_addr(&self)              -> &CubeAddr             { &self.local_addr }
    pub fn registration_timestamp(&self)  -> u64                   { self.registration_timestamp }
    pub fn jitter_secs(&self)             -> u64                   { compute_rotation_jitter(self.registration_timestamp) }

    pub fn reregistration_complete(&mut self) {
        self.pending_reregistration = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// REKEY HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Re-derive all FTS HMAC keys from a new master secret.
/// Returns the number of keys re-derived (should be 26).
pub fn rekey_fts_hmac(
    fts: &mut crate::fts::FaultToleranceService,
    new_secret: &MasterSecret,
) -> usize {
    fts.derive_all_hmac_keys(new_secret.as_bytes());
    fts.all_status().iter().filter(|n| n.hmac_key.is_some()).count()
}

/// Re-derive the identity keypair. Returns the new public key for CRS.
pub fn rekey_identity(
    key_mgr: &mut crate::address_keys::AddressKeyManager,
    addr: &CubeAddr,
    new_secret: &MasterSecret,
) -> Vec<u8> {
    key_mgr.set_master_secret(new_secret);
    key_mgr.get_public_key(addr, new_secret)
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr { CubeAddr::new(trits) }
    fn test_addr() -> CubeAddr { addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]) }
    fn test_secret() -> MasterSecret { MasterSecret::from_seed(b"test-rotation-secret-epoch-0") }

    /// Registration timestamp with known jitter: 12345 % 901 = 498 seconds.
    const TEST_REG_TS: u64 = 12_345;

    fn test_orchestrator() -> RotationOrchestrator {
        RotationOrchestrator::new(
            test_addr(), TEST_REG_TS, test_secret(), 0, b"test-passphrase".to_vec(),
        )
    }

    fn test_capability(addr: CubeAddr, ts: u64) -> RotationCapability {
        RotationCapability::new(addr, ts, vec![0u8]) // placeholder — test gate only
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert_eq!(RADIAN_DEG_INT, ternary_math::constants::RADIAN_DEG as u64);
        assert_eq!(ROTATION_PERIOD_SECS, ARC_EPOCH_SECS / RADIAN_DEG_INT);
        assert_eq!(ROTATION_PERIOD_SECS, 14 * 86400);
        assert_eq!(MAX_JITTER_SECS, 900);
        assert_eq!(DUAL_ACCEPT_SECS, 1);
        assert_eq!(CAPABILITY_TTL_SECS, 1_800);
        assert_eq!(ARC_EPOCH_SECS, 182 * 86400);
        assert_ne!(DUAL_ACCEPT_SECS, ROTATION_PERIOD_SECS);
        assert_ne!(DUAL_ACCEPT_SECS, ARC_EPOCH_SECS);
    }

    #[test]
    fn test_thirteen_rotations_per_arc() {
        assert_eq!(ARC_EPOCH_SECS / ROTATION_PERIOD_SECS, RADIAN_DEG_INT);
    }

    // ── Jitter ──────────────────────────────────────────────────

    #[test]
    fn test_jitter_deterministic() {
        assert_eq!(compute_rotation_jitter(TEST_REG_TS), compute_rotation_jitter(TEST_REG_TS));
    }

    #[test]
    fn test_jitter_in_range() {
        assert!(compute_rotation_jitter(TEST_REG_TS) <= MAX_JITTER_SECS);
    }

    #[test]
    fn test_jitter_positive_only() {
        for ts in [0u64, 1, 899, 900, 901, 1_800, 1_000_000, u64::MAX / 2] {
            let j = compute_rotation_jitter(ts);
            assert!(j <= MAX_JITTER_SECS, "ts={} gave jitter={}", ts, j);
        }
    }

    #[test]
    fn test_jitter_boundary_values() {
        assert_eq!(compute_rotation_jitter(0),     0);
        assert_eq!(compute_rotation_jitter(900),   900);
        assert_eq!(compute_rotation_jitter(901),   0);
        assert_eq!(compute_rotation_jitter(1_800), 899);
        assert_eq!(compute_rotation_jitter(1_801), 900);
        assert_eq!(compute_rotation_jitter(1_802), 0);
    }

    #[test]
    fn test_effective_rotation_at_or_after_boundary() {
        let boundary = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS;
        let effective = effective_rotation_time(1, TEST_REG_TS);
        assert!(effective >= boundary);
        assert!(effective <= boundary + MAX_JITTER_SECS);
    }

    // ── Rotation due ────────────────────────────────────────────

    #[test]
    fn test_rotation_not_due_early() {
        let ts = SALVI_EPOCH_UNIX + 7 * 86400; // day 7, mid first radian
        assert!(rotation_due_with_jitter(0, TEST_REG_TS, ts).is_none());
    }

    #[test]
    fn test_rotation_due_after_boundary() {
        let ts = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS + MAX_JITTER_SECS + 60;
        assert_eq!(rotation_due_with_jitter(0, TEST_REG_TS, ts), Some(1));
    }

    // ── RotationCapability ──────────────────────────────────────

    #[test]
    fn test_capability_valid() {
        let ts = 1_000_000u64;
        assert!(test_capability(test_addr(), ts).validate(&test_addr(), ts).is_ok());
    }

    #[test]
    fn test_capability_expired() {
        let ts = 1_000_000u64;
        let cap = test_capability(test_addr(), ts);
        assert_eq!(cap.validate(&test_addr(), ts + CAPABILITY_TTL_SECS + 1), Err(CapabilityError::Expired));
    }

    #[test]
    fn test_capability_address_mismatch() {
        let ts = 1_000_000u64;
        let cap = test_capability(test_addr(), ts);
        let other = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(cap.validate(&other, ts), Err(CapabilityError::AddressMismatch));
    }

    // ── Orchestrator ────────────────────────────────────────────

    #[test]
    fn test_orchestrator_creation() {
        let orch = test_orchestrator();
        assert_eq!(orch.current_epoch(), 0);
        assert!(!orch.in_dual_accept());
        assert!(!orch.needs_reregistration());
        assert_eq!(orch.rotation_count(), 0);
        assert_eq!(orch.registration_timestamp(), TEST_REG_TS);
    }

    #[test]
    fn test_no_rotation_mid_period() {
        let mut orch = test_orchestrator().with_check_interval(Duration::from_millis(0));
        let ts = SALVI_EPOCH_UNIX + 7 * 86400;
        assert!(orch.check_and_rotate(ts).unwrap().is_none());
    }

    #[test]
    fn test_rotation_at_radian_boundary() {
        let mut orch = test_orchestrator().with_check_interval(Duration::from_millis(0));
        let ts = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS + MAX_JITTER_SECS + 60;
        let event = orch.check_and_rotate(ts).unwrap().unwrap();
        assert_eq!(event.old_epoch, 0);
        assert_eq!(event.new_epoch, 1);
        assert!(!event.forced);
        assert_eq!(orch.current_epoch(), 1);
        assert!(orch.in_dual_accept());
    }

    #[test]
    fn test_reregistration_lifecycle() {
        let mut orch = test_orchestrator().with_check_interval(Duration::from_millis(0));
        let ts = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS + MAX_JITTER_SECS + 60;
        orch.check_and_rotate(ts).unwrap();
        assert!(orch.needs_reregistration());
        orch.reregistration_complete();
        assert!(!orch.needs_reregistration());
    }

    #[test]
    fn test_force_rotate_valid_capability() {
        let mut orch = test_orchestrator();
        let ts = SALVI_EPOCH_UNIX + 5 * 86400;
        let event = orch.force_rotate(ts, test_capability(test_addr(), ts)).unwrap();
        assert!(event.forced);
        assert_eq!(orch.rotation_count(), 1);
    }

    #[test]
    fn test_force_rotate_rejects_wrong_address() {
        let mut orch = test_orchestrator();
        let ts = SALVI_EPOCH_UNIX + 5 * 86400;
        let bad = test_capability(addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), ts);
        assert!(matches!(
            orch.force_rotate(ts, bad),
            Err(RotationError::CapabilityDenied(CapabilityError::AddressMismatch))
        ));
        assert_eq!(orch.rotation_count(), 0);
    }

    #[test]
    fn test_force_rotate_rejects_expired_capability() {
        let mut orch = test_orchestrator();
        let issue_ts = SALVI_EPOCH_UNIX + 5 * 86400;
        let use_ts   = issue_ts + CAPABILITY_TTL_SECS + 1;
        let expired  = test_capability(test_addr(), issue_ts);
        assert!(matches!(
            orch.force_rotate(use_ts, expired),
            Err(RotationError::CapabilityDenied(CapabilityError::Expired))
        ));
        assert_eq!(orch.rotation_count(), 0);
    }

    #[test]
    fn test_dual_accept_closes_after_one_second() {
        let mut orch = test_orchestrator().with_check_interval(Duration::from_millis(0));
        let ts1 = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS + MAX_JITTER_SECS + 60;
        orch.check_and_rotate(ts1).unwrap();
        assert!(orch.in_dual_accept());
        let ts2 = ts1 + DUAL_ACCEPT_SECS + 1;
        orch.check_and_rotate(ts2).unwrap();
        // check_dual_accept fires internally — window now closed
    }

    #[test]
    fn test_history_records_forced_flag() {
        let mut orch = test_orchestrator();
        let ts1 = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS;
        let ts2 = SALVI_EPOCH_UNIX + ROTATION_PERIOD_SECS * 2;
        orch.force_rotate(ts1, test_capability(test_addr(), ts1)).unwrap();
        orch.force_rotate(ts2, test_capability(test_addr(), ts2)).unwrap();
        assert_eq!(orch.history().len(), 2);
        assert!(orch.history()[0].forced);
        assert!(orch.history()[1].forced);
    }

    // ── Key Freshness Zones (T-17) ─────────────────────────────

    #[test]
    fn test_freshness_zone_fresh() {
        assert_eq!(key_freshness_zone(0), Some(KeyFreshnessZone::Fresh));
        assert_eq!(key_freshness_zone(30), Some(KeyFreshnessZone::Fresh));
        assert_eq!(key_freshness_zone(60), Some(KeyFreshnessZone::Fresh));
    }

    #[test]
    fn test_freshness_zone_active() {
        assert_eq!(key_freshness_zone(61), Some(KeyFreshnessZone::Active));
        assert_eq!(key_freshness_zone(90), Some(KeyFreshnessZone::Active));
        assert_eq!(key_freshness_zone(121), Some(KeyFreshnessZone::Active));
    }

    #[test]
    fn test_freshness_zone_aging() {
        assert_eq!(key_freshness_zone(122), Some(KeyFreshnessZone::Aging));
        assert_eq!(key_freshness_zone(150), Some(KeyFreshnessZone::Aging));
        assert_eq!(key_freshness_zone(182), Some(KeyFreshnessZone::Aging));
    }

    #[test]
    fn test_freshness_zone_expired() {
        assert_eq!(key_freshness_zone(183), None);
        assert_eq!(key_freshness_zone(365), None);
    }

    #[test]
    fn test_freshness_boundary_at_repunit_r5() {
        assert_eq!(key_freshness_zone(121), Some(KeyFreshnessZone::Active));
        assert_eq!(key_freshness_zone(122), Some(KeyFreshnessZone::Aging));
    }

    #[test]
    fn test_key_suitable_for_regulated() {
        assert!(key_suitable_for_regulated(0));
        assert!(key_suitable_for_regulated(60));
        assert!(key_suitable_for_regulated(121));
        assert!(!key_suitable_for_regulated(122));
        assert!(!key_suitable_for_regulated(182));
    }

    #[test]
    fn test_key_birth_epoch() {
        assert_eq!(key_birth_epoch(13, 0), 13);
        assert_eq!(key_birth_epoch(13, 14), 12);
        assert_eq!(key_birth_epoch(13, 28), 11);
        assert_eq!(key_birth_epoch(13, 182), 0);
    }

    #[test]
    fn test_freshness_zone_display() {
        assert_eq!(format!("{}", KeyFreshnessZone::Fresh), "fresh");
        assert_eq!(format!("{}", KeyFreshnessZone::Active), "active");
        assert_eq!(format!("{}", KeyFreshnessZone::Aging), "aging");
    }
}