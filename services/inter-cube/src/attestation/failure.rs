// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Failure Handling — Task #119 Task 6

//! Failure mode handling for all 7 attestation dependencies.
//! Each failure has a defined outcome (block/retry/degrade),
//! operator-visible error message, and structured audit event.

use crate::cube_addr::CubeAddr;
use super::audit::{AttestAuditEvent, AttestSeverity};

// ═══════════════════════════════════════════════════════════════════════
// SERVICE STATE
// ═══════════════════════════════════════════════════════════════════════

/// Attestation service operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Normal operation — signing and broadcasting attestation reports.
    Running,
    /// Degraded — one or more dependencies failed. Attestation blocked.
    /// Node continues heartbeats but does not broadcast attestation reports.
    Degraded(DegradedReason),
    /// Disabled — attestation.enabled = false in PlenumConfig.
    Disabled,
}

/// Reason for entering degraded state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradedReason {
    /// PUF hardware unavailable or key corruption.
    PufUnavailable,
    /// Boot measurement data unavailable.
    BootMissing,
    /// Sequence persistence corrupt.
    SeqCorrupt,
    /// HPTP timestamp service unreachable after retries.
    HptpUnreachable,
    /// Attestation public key not yet published to PlenumConfig.
    KeyNotPublished,
}

impl std::fmt::Display for DegradedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PufUnavailable => write!(f, "PUF hardware not responding"),
            Self::BootMissing => write!(f, "boot measurement data unavailable"),
            Self::SeqCorrupt => write!(f, "sequence persistence corrupt"),
            Self::HptpUnreachable => write!(f, "HPTP timestamp service unreachable"),
            Self::KeyNotPublished => write!(f, "attestation public key not published to PlenumConfig"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FAILURE OUTCOMES
// ═══════════════════════════════════════════════════════════════════════

/// Defined outcome for each dependency failure.
#[derive(Debug, Clone)]
pub struct FailureOutcome {
    /// Whether attestation broadcast is blocked.
    pub blocks_attestation: bool,
    /// Operator-visible error message (what/why/action).
    pub operator_message: String,
    /// Audit event to emit.
    pub audit_event: AttestAuditEvent,
    /// Resulting service state.
    pub service_state: ServiceState,
}

/// Handle TL-DSA signing failure (PUF unavailable or key corruption).
pub fn handle_sign_failure(node: &CubeAddr, error_code: u32, ts: u128) -> FailureOutcome {
    FailureOutcome {
        blocks_attestation: true,
        operator_message: "Attestation signing failed: PUF hardware not responding. \
            Verify PUF hardware is seated correctly; check Windows Device Manager \
            for PUF device status. Error details in attestation log.".into(),
        audit_event: AttestAuditEvent::SignFail {
            node_rep_c: node.clone(), error_code, timestamp: ts,
        },
        service_state: ServiceState::Degraded(DegradedReason::PufUnavailable),
    }
}

/// Handle HPTP timestamp service unreachable (after 3 retries).
pub fn handle_hptp_timeout(node: &CubeAddr, retry_count: u8, ts: u128) -> FailureOutcome {
    FailureOutcome {
        blocks_attestation: true,
        operator_message: format!(
            "Attestation delayed: HPTP timestamp service unreachable after {} retries. \
            Attestation will resume when HPTP service recovers. Check HPTP service status.",
            retry_count
        ),
        audit_event: AttestAuditEvent::HptpTimeout {
            node_rep_c: node.clone(), retry_count, timestamp: ts,
        },
        service_state: ServiceState::Degraded(DegradedReason::HptpUnreachable),
    }
}

/// Handle boot measurement data unavailable.
pub fn handle_boot_missing(node: &CubeAddr, ts: u128) -> FailureOutcome {
    FailureOutcome {
        blocks_attestation: true,
        operator_message: "Attestation blocked: boot measurement data unavailable. \
            This may indicate firmware tampering or boot sequence failure. \
            Investigate immediately.".into(),
        audit_event: AttestAuditEvent::BootMissing {
            node_rep_c: node.clone(), timestamp: ts,
        },
        service_state: ServiceState::Degraded(DegradedReason::BootMissing),
    }
}

/// Handle sequence persistence corruption.
pub fn handle_seq_corrupt(node: &CubeAddr, ts: u128) -> FailureOutcome {
    FailureOutcome {
        blocks_attestation: true,
        operator_message: "Attestation blocked: sequence number persistence store corrupt \
            or unreadable. Service cannot resume safely. Check persistence.rs data directory.".into(),
        audit_event: AttestAuditEvent::SeqCorrupt {
            node_rep_c: node.clone(), timestamp: ts,
        },
        service_state: ServiceState::Degraded(DegradedReason::SeqCorrupt),
    }
}

/// Handle PUF self-test degraded (fuzzy extractor health check failed).
/// Does NOT block attestation — continues broadcasting so neighbors see the failure.
pub fn handle_puf_degraded(node: &CubeAddr, fuzzy_health: u8, ts: u128) -> FailureOutcome {
    FailureOutcome {
        blocks_attestation: false, // continues broadcasting
        operator_message: format!(
            "PUF self-test degraded: fuzzy extractor health check returned error ({}). \
            Node continues attesting but neighbors will flag this node as suspect. \
            Contact support if condition persists.",
            fuzzy_health
        ),
        audit_event: AttestAuditEvent::PufDegraded {
            node_rep_c: node.clone(), fuzzy_health, timestamp: ts,
        },
        service_state: ServiceState::Running, // still running
    }
}

/// Handle Inter-Cube tunnel unavailable to specific neighbor.
/// Per-link failure — does NOT block attestation to other neighbors.
pub fn handle_tunnel_unavailable(neighbor: &CubeAddr) -> String {
    format!(
        "Attestation to neighbor [{}] queued: Inter-Cube tunnel unavailable. \
        Will retry when tunnel re-establishes.",
        neighbor.to_dotted()
    )
}

/// Handle empty Merkle tree (no heartbeat challenges received).
/// Does NOT increment suspicion counter.
pub fn handle_empty_merkle() -> String {
    "Attestation liveness: no heartbeat challenges received in last interval. \
    Liveness proof is empty. Check heartbeat subsystem connectivity.".into()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr() -> CubeAddr { CubeAddr::new([1; 13]) }

    #[test]
    fn sign_failure_blocks_attestation() {
        let outcome = handle_sign_failure(&addr(), 1, 0);
        assert!(outcome.blocks_attestation);
        assert_eq!(outcome.service_state, ServiceState::Degraded(DegradedReason::PufUnavailable));
        assert_eq!(outcome.audit_event.severity(), AttestSeverity::Critical);
    }

    #[test]
    fn puf_degraded_does_not_block() {
        let outcome = handle_puf_degraded(&addr(), 1, 0);
        assert!(!outcome.blocks_attestation);
        assert_eq!(outcome.service_state, ServiceState::Running);
        assert_eq!(outcome.audit_event.severity(), AttestSeverity::Warning);
    }

    #[test]
    fn all_blocking_failures_produce_critical() {
        let blocking = vec![
            handle_sign_failure(&addr(), 0, 0),
            handle_boot_missing(&addr(), 0),
            handle_seq_corrupt(&addr(), 0),
        ];
        for outcome in &blocking {
            assert!(outcome.blocks_attestation);
            assert_eq!(outcome.audit_event.severity(), AttestSeverity::Critical);
        }
    }

    #[test]
    fn hptp_timeout_blocks_after_retries() {
        let outcome = handle_hptp_timeout(&addr(), 3, 0);
        assert!(outcome.blocks_attestation);
        assert!(outcome.operator_message.contains("3 retries"));
    }

    #[test]
    fn tunnel_message_uses_dotted_repc() {
        let nbr = CubeAddr::new([2, 1, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let msg = handle_tunnel_unavailable(&nbr);
        assert!(msg.contains("2.1.3.1.2.3.1.2.3.1.2.3.1"));
    }
}
