// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Verification — Task #119 Task 4
// Replay protection, partition-aware suspicion, FTS integration

//! Neighbor attestation verification with replay protection and
//! partition-aware suspicion counters.
//!
//! Replay protection: monotonic sequence check + HPTP freshness (≤240s).
//! Suspicion: heartbeat-reachable but attestation-absent increments;
//! heartbeat-unreachable does NOT increment (partition awareness).
//! Threshold crossing triggers FTS Suspect transition.

use std::collections::HashMap;
use ternary_math::trit_int::TritInt;

use crate::cube_addr::CubeAddr;
use super::report::{SignedAttestationReport, ReportError, SCHEMA_VERSION};
use super::signing::AttestationSigningKey;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum freshness window: 2 × interval_max_s (default 120) = 240 seconds.
/// Expressed in femtoseconds for HPTP comparison.
const FRESHNESS_WINDOW_FS: u128 = 240 * 1_000_000_000_000_000;

/// Default suspicion threshold: 3 consecutive missed attestations.
const DEFAULT_SUSPICION_THRESHOLD: u32 = 3;

/// Minimum suspicion threshold floor.
const MIN_SUSPICION_THRESHOLD: u32 = 1;

/// Maximum suspicion threshold ceiling.
const MAX_SUSPICION_THRESHOLD: u32 = 100;

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR ATTESTATION STATE
// ═══════════════════════════════════════════════════════════════════════

/// Per-neighbor attestation verification state.
#[derive(Debug, Clone)]
pub struct NeighborAttestState {
    /// Last accepted sequence number. Volatile (not persisted).
    /// After receiver restart, resets to zero (freshness-only protection).
    pub last_sequence: TritInt,
    /// Consecutive missed attestation intervals while heartbeat-reachable.
    pub suspicion_counter: u32,
    /// Last received attestation timestamp (femtoseconds).
    pub last_attest_ts: Option<u128>,
    /// Whether this neighbor is heartbeat-reachable.
    pub heartbeat_reachable: bool,
}

impl NeighborAttestState {
    pub fn new() -> Self {
        Self {
            last_sequence: TritInt::zero(),
            suspicion_counter: 0,
            last_attest_ts: None,
            heartbeat_reachable: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// VERIFICATION RESULT
// ═══════════════════════════════════════════════════════════════════════

/// Result of verifying a received attestation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    /// Report accepted — valid signature, fresh, sequence advancing.
    Accepted,
    /// Signature verification failed.
    SignatureFailed,
    /// Sequence number ≤ last accepted (replay).
    SequenceReplay,
    /// Timestamp outside freshness window (>240s stale).
    StaleTimestamp,
    /// Unknown schema version — "unparseable", NOT suspicion increment.
    UnknownVersion(u16),
    /// Kernel hash or config fingerprint mismatch.
    ValueMismatch(String),
    /// Report from unknown address (not a registered neighbor).
    UnknownSender,
}

// ═══════════════════════════════════════════════════════════════════════
// SUSPICION OUTCOME
// ═══════════════════════════════════════════════════════════════════════

/// Outcome of a suspicion counter check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuspicionOutcome {
    /// No suspicion — attestation received or neighbor unreachable.
    None,
    /// Counter incremented but below threshold.
    Incremented(u32),
    /// Threshold crossed — trigger FTS Suspect transition.
    ThresholdCrossed { counter: u32, threshold: u32 },
}

// ═══════════════════════════════════════════════════════════════════════
// ATTESTATION VERIFIER
// ═══════════════════════════════════════════════════════════════════════

/// Verifies incoming attestation reports from neighbors.
pub struct AttestationVerifier {
    /// Per-neighbor state, keyed by Rep C address.
    neighbor_states: HashMap<CubeAddr, NeighborAttestState>,
    /// Suspicion threshold (configurable, TL-DSA-signed in PlenumConfig).
    suspicion_threshold: u32,
    /// Known schema versions (from TL-DSA-signed version registry).
    known_versions: Vec<u16>,
    /// Expected kernel hash (from PlenumConfig). Dual-hash during upgrades.
    expected_kernel_hashes: Vec<Vec<u8>>,
    /// Expected config fingerprint (from PlenumConfig).
    expected_config_fingerprints: Vec<Vec<u8>>,
    /// Whether an upgrade window is active (suppresses version mismatch alerts).
    upgrade_window_active: bool,
}

impl AttestationVerifier {
    /// Create a new verifier with default configuration.
    pub fn new() -> Self {
        Self {
            neighbor_states: HashMap::new(),
            suspicion_threshold: DEFAULT_SUSPICION_THRESHOLD,
            known_versions: vec![SCHEMA_VERSION],
            expected_kernel_hashes: Vec::new(),
            expected_config_fingerprints: Vec::new(),
            upgrade_window_active: false,
        }
    }

    /// Set the suspicion threshold (range 1–100, from TL-DSA-signed config).
    pub fn set_threshold(&mut self, threshold: u32) -> Result<(), &'static str> {
        if threshold < MIN_SUSPICION_THRESHOLD || threshold > MAX_SUSPICION_THRESHOLD {
            return Err("suspicion threshold out of range 1–100");
        }
        if threshold > 50 {
            // WARNING log — values >50 are not recommended
            tracing::warn!(
                threshold,
                "attestation suspicion threshold >50 is not recommended (range 3–10)"
            );
        }
        self.suspicion_threshold = threshold;
        Ok(())
    }

    /// Update expected kernel hashes (from PlenumConfig).
    /// During rolling upgrades, both old and new hashes are accepted.
    pub fn set_expected_kernel_hashes(&mut self, hashes: Vec<Vec<u8>>) {
        self.expected_kernel_hashes = hashes;
    }

    /// Update expected config fingerprints (from PlenumConfig).
    pub fn set_expected_config_fingerprints(&mut self, fingerprints: Vec<Vec<u8>>) {
        self.expected_config_fingerprints = fingerprints;
    }

    /// Set upgrade window state.
    pub fn set_upgrade_window(&mut self, active: bool) {
        self.upgrade_window_active = active;
    }

    /// Update heartbeat reachability for a neighbor.
    /// Called by FTS when heartbeat state changes.
    pub fn set_heartbeat_reachable(&mut self, neighbor: &CubeAddr, reachable: bool) {
        let state = self.neighbor_states.entry(neighbor.clone())
            .or_insert_with(NeighborAttestState::new);
        state.heartbeat_reachable = reachable;
    }

    /// Verify a received signed attestation report.
    ///
    /// Checks: signature, sequence monotonicity, HPTP freshness,
    /// schema version, kernel hash, config fingerprint.
    pub fn verify(
        &mut self,
        signed_report: &SignedAttestationReport,
        verifier_key: &[u8],
        current_time_fs: u128,
    ) -> VerifyResult {
        let report = &signed_report.report;
        let sender = &report.node_addr;

        // 1. Signature verification
        if !AttestationSigningKey::verify_report(verifier_key, signed_report) {
            return VerifyResult::SignatureFailed;
        }

        // 2. Schema version check
        let version = report.schema_version.to_u64().unwrap_or(0) as u16;
        if !self.known_versions.contains(&version) {
            return VerifyResult::UnknownVersion(version);
        }

        // 3. Sequence number monotonicity
        let state = self.neighbor_states.entry(sender.clone())
            .or_insert_with(NeighborAttestState::new);

        if report.sequence <= state.last_sequence && !state.last_sequence.is_zero() {
            return VerifyResult::SequenceReplay;
        }

        // 4. HPTP freshness window (≤240s from current time)
        let report_ts = report.timestamp.to_u128().unwrap_or(0);
        if current_time_fs > report_ts && (current_time_fs - report_ts) > FRESHNESS_WINDOW_FS {
            return VerifyResult::StaleTimestamp;
        }

        // 5. Kernel hash comparison (dual-expected-hash during upgrades)
        if !self.expected_kernel_hashes.is_empty()
            && !self.expected_kernel_hashes.contains(&report.kernel_hash)
        {
            return VerifyResult::ValueMismatch("kernel_hash".into());
        }

        // 6. Config fingerprint comparison
        if !self.expected_config_fingerprints.is_empty()
            && !self.expected_config_fingerprints.contains(&report.config_fingerprint)
        {
            return VerifyResult::ValueMismatch("config_fingerprint".into());
        }

        // All checks passed — update state
        state.last_sequence = report.sequence.clone();
        state.last_attest_ts = Some(report_ts);
        state.suspicion_counter = 0; // Reset on successful attestation

        VerifyResult::Accepted
    }

    /// Check suspicion for a neighbor that missed an attestation interval.
    /// ONLY increments if the neighbor is heartbeat-reachable.
    /// Returns the suspicion outcome.
    pub fn check_suspicion(&mut self, neighbor: &CubeAddr) -> SuspicionOutcome {
        let state = self.neighbor_states.entry(neighbor.clone())
            .or_insert_with(NeighborAttestState::new);

        // Partition awareness: heartbeat-unreachable → NOT attestation failure
        if !state.heartbeat_reachable {
            return SuspicionOutcome::None;
        }

        state.suspicion_counter += 1;

        if state.suspicion_counter >= self.suspicion_threshold {
            SuspicionOutcome::ThresholdCrossed {
                counter: state.suspicion_counter,
                threshold: self.suspicion_threshold,
            }
        } else {
            SuspicionOutcome::Incremented(state.suspicion_counter)
        }
    }

    /// Get the current suspicion counter for a neighbor.
    pub fn suspicion_counter(&self, neighbor: &CubeAddr) -> u32 {
        self.neighbor_states.get(neighbor)
            .map(|s| s.suspicion_counter)
            .unwrap_or(0)
    }

    /// Whether the upgrade window suppresses version mismatch alerts.
    pub fn is_upgrade_window_active(&self) -> bool {
        self.upgrade_window_active
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fts::NeighborState;
    use super::super::report::*;
    use super::super::signing::*;

    fn addr1() -> CubeAddr { CubeAddr::new([1; 13]) }
    fn addr2() -> CubeAddr { CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }

    fn make_signed(addr: &CubeAddr, seq: u64, ts: u128, secret: &[u8]) -> SignedAttestationReport {
        let key = AttestationSigningKey::derive(secret, addr);
        let report = AttestationReport {
            node_addr: addr.clone(),
            sequence: TritInt::from_u64(seq),
            timestamp: TritInt::from_u128(ts),
            schema_version: TritInt::from_u64(SCHEMA_VERSION as u64),
            boot_measurements: BootMeasurements {
                firmware_hash: vec![0xAB; 48],
                anti_rollback_counter: TritInt::from_u64(1),
            },
            kernel_hash: vec![0xCD; 48],
            puf_health: PufHealth::Healthy,
            fts_state: NeighborState::Up,
            config_fingerprint: vec![0xEF; 27],
            merkle_root: vec![0x12; 27],
        };
        key.sign_report(&report).unwrap()
    }

    #[test]
    fn accepts_valid_report() {
        let addr = addr1();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let now_fs: u128 = 100 * 1_000_000_000_000_000;
        let signed = make_signed(&addr, 1, now_fs, &secret);

        let mut verifier = AttestationVerifier::new();
        assert_eq!(
            verifier.verify(&signed, &key.key_material, now_fs + 1_000_000_000_000_000),
            VerifyResult::Accepted
        );
    }

    #[test]
    fn rejects_replay_sequence() {
        let addr = addr1();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let now_fs: u128 = 100 * 1_000_000_000_000_000;

        let mut verifier = AttestationVerifier::new();

        // Accept sequence 5
        let signed5 = make_signed(&addr, 5, now_fs, &secret);
        assert_eq!(verifier.verify(&signed5, &key.key_material, now_fs), VerifyResult::Accepted);

        // Reject sequence 5 again
        let signed5b = make_signed(&addr, 5, now_fs, &secret);
        assert_eq!(verifier.verify(&signed5b, &key.key_material, now_fs), VerifyResult::SequenceReplay);

        // Accept sequence 6
        let signed6 = make_signed(&addr, 6, now_fs, &secret);
        assert_eq!(verifier.verify(&signed6, &key.key_material, now_fs), VerifyResult::Accepted);
    }

    #[test]
    fn rejects_stale_timestamp() {
        let addr = addr1();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let old_ts: u128 = 100 * 1_000_000_000_000_000;
        let now_fs = old_ts + FRESHNESS_WINDOW_FS + 1_000_000_000_000_000; // >240s later

        let signed = make_signed(&addr, 1, old_ts, &secret);
        let mut verifier = AttestationVerifier::new();
        assert_eq!(verifier.verify(&signed, &key.key_material, now_fs), VerifyResult::StaleTimestamp);
    }

    #[test]
    fn suspicion_increments_when_reachable() {
        let nbr = addr2();
        let mut verifier = AttestationVerifier::new();
        verifier.set_heartbeat_reachable(&nbr, true);

        assert_eq!(verifier.check_suspicion(&nbr), SuspicionOutcome::Incremented(1));
        assert_eq!(verifier.check_suspicion(&nbr), SuspicionOutcome::Incremented(2));
        assert_eq!(verifier.check_suspicion(&nbr), SuspicionOutcome::ThresholdCrossed {
            counter: 3, threshold: 3
        });
    }

    #[test]
    fn suspicion_skips_when_unreachable() {
        let nbr = addr2();
        let mut verifier = AttestationVerifier::new();
        verifier.set_heartbeat_reachable(&nbr, false);

        // Partition awareness: unreachable → no increment
        assert_eq!(verifier.check_suspicion(&nbr), SuspicionOutcome::None);
        assert_eq!(verifier.check_suspicion(&nbr), SuspicionOutcome::None);
        assert_eq!(verifier.suspicion_counter(&nbr), 0);
    }

    #[test]
    fn suspicion_resets_on_valid_attestation() {
        let addr = addr1();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let now_fs: u128 = 100 * 1_000_000_000_000_000;

        let mut verifier = AttestationVerifier::new();
        verifier.set_heartbeat_reachable(&addr, true);

        // Increment suspicion twice
        verifier.check_suspicion(&addr);
        verifier.check_suspicion(&addr);
        assert_eq!(verifier.suspicion_counter(&addr), 2);

        // Valid attestation resets counter
        let signed = make_signed(&addr, 1, now_fs, &secret);
        assert_eq!(verifier.verify(&signed, &key.key_material, now_fs), VerifyResult::Accepted);
        assert_eq!(verifier.suspicion_counter(&addr), 0);
    }

    #[test]
    fn threshold_range_enforced() {
        let mut verifier = AttestationVerifier::new();
        assert!(verifier.set_threshold(0).is_err());
        assert!(verifier.set_threshold(101).is_err());
        assert!(verifier.set_threshold(1).is_ok());
        assert!(verifier.set_threshold(100).is_ok());
        assert!(verifier.set_threshold(3).is_ok());
    }

    #[test]
    fn receiver_restart_replay_blocked_by_freshness() {
        let addr = addr1();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let old_ts: u128 = 100 * 1_000_000_000_000_000;
        let now_fs = old_ts + FRESHNESS_WINDOW_FS + 1_000_000_000_000_000;

        // Fresh verifier (simulating restart — sequence counter at zero)
        let mut verifier = AttestationVerifier::new();

        // Replay old report — sequence 0 would normally be accepted post-restart,
        // but timestamp is >240s stale → rejected by freshness
        let signed = make_signed(&addr, 1, old_ts, &secret);
        assert_eq!(verifier.verify(&signed, &key.key_material, now_fs), VerifyResult::StaleTimestamp);
    }
}
