// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Audit Events — integration with security-audit.service.ts

//! Attestation-specific audit event types for the security audit service.
//! All node identification uses Rep C dot-separated format (INVARIANT 9).

use crate::cube_addr::CubeAddr;

/// Attestation audit event severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AttestSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Attestation audit event types per Task #119 §Attestation Audit Events.
#[derive(Debug, Clone)]
pub enum AttestAuditEvent {
    /// ATTEST_SIGN_FAIL: TL-DSA signing failure. CRITICAL.
    SignFail {
        node_rep_c: CubeAddr,
        error_code: u32,
        timestamp: u128,
    },
    /// ATTEST_HPTP_TIMEOUT: HPTP timestamp unreachable. WARNING.
    HptpTimeout {
        node_rep_c: CubeAddr,
        retry_count: u8,
        timestamp: u128,
    },
    /// ATTEST_BOOT_MISSING: Boot measurement unavailable. CRITICAL.
    BootMissing {
        node_rep_c: CubeAddr,
        timestamp: u128,
    },
    /// ATTEST_SEQ_CORRUPT: Sequence persistence corrupt. CRITICAL.
    SeqCorrupt {
        node_rep_c: CubeAddr,
        timestamp: u128,
    },
    /// ATTEST_PUF_DEGRADED: PUF self-test failure. WARNING.
    PufDegraded {
        node_rep_c: CubeAddr,
        fuzzy_health: u8,
        timestamp: u128,
    },
    /// ATTEST_SUSPECT: Suspicion threshold crossed. WARNING.
    Suspect {
        reporter_rep_c: CubeAddr,
        suspect_rep_c: CubeAddr,
        counter: u32,
        threshold: u32,
        timestamp: u128,
    },
    /// ATTEST_MISMATCH: Hash/fingerprint mismatch. WARNING.
    Mismatch {
        reporter_rep_c: CubeAddr,
        attester_rep_c: CubeAddr,
        field: String,
        timestamp: u128,
    },
    /// ATTEST_VERSION_UNKNOWN: Unknown schema version received. INFO.
    VersionUnknown {
        reporter_rep_c: CubeAddr,
        attester_rep_c: CubeAddr,
        version: u16,
        timestamp: u128,
    },
}

impl AttestAuditEvent {
    /// Event ID string for log correlation.
    pub fn event_id(&self) -> &'static str {
        match self {
            Self::SignFail { .. } => "ATTEST_SIGN_FAIL",
            Self::HptpTimeout { .. } => "ATTEST_HPTP_TIMEOUT",
            Self::BootMissing { .. } => "ATTEST_BOOT_MISSING",
            Self::SeqCorrupt { .. } => "ATTEST_SEQ_CORRUPT",
            Self::PufDegraded { .. } => "ATTEST_PUF_DEGRADED",
            Self::Suspect { .. } => "ATTEST_SUSPECT",
            Self::Mismatch { .. } => "ATTEST_MISMATCH",
            Self::VersionUnknown { .. } => "ATTEST_VERSION_UNKNOWN",
        }
    }

    /// Severity level for this event.
    pub fn severity(&self) -> AttestSeverity {
        match self {
            Self::SignFail { .. } | Self::BootMissing { .. } | Self::SeqCorrupt { .. } => {
                AttestSeverity::Critical
            }
            Self::HptpTimeout { .. } | Self::PufDegraded { .. } | Self::Suspect { .. } | Self::Mismatch { .. } => {
                AttestSeverity::Warning
            }
            Self::VersionUnknown { .. } => AttestSeverity::Info,
        }
    }

    /// Format the event for log output. All addresses in dot-separated Rep C.
    pub fn log_message(&self) -> String {
        match self {
            Self::SignFail { node_rep_c, error_code, .. } => {
                format!("[{}] {} node={} error_code={}",
                    self.severity(), self.event_id(), node_rep_c.to_rep_c_display(), error_code)
            }
            Self::HptpTimeout { node_rep_c, retry_count, .. } => {
                format!("[{}] {} node={} retries={}",
                    self.severity(), self.event_id(), node_rep_c.to_rep_c_display(), retry_count)
            }
            Self::BootMissing { node_rep_c, .. } => {
                format!("[{}] {} node={}",
                    self.severity(), self.event_id(), node_rep_c.to_rep_c_display())
            }
            Self::SeqCorrupt { node_rep_c, .. } => {
                format!("[{}] {} node={}",
                    self.severity(), self.event_id(), node_rep_c.to_rep_c_display())
            }
            Self::PufDegraded { node_rep_c, fuzzy_health, .. } => {
                format!("[{}] {} node={} fuzzy_health={}",
                    self.severity(), self.event_id(), node_rep_c.to_rep_c_display(), fuzzy_health)
            }
            Self::Suspect { reporter_rep_c, suspect_rep_c, counter, threshold, .. } => {
                format!("[{}] {} reporter={} suspect={} counter={} threshold={}",
                    self.severity(), self.event_id(),
                    reporter_rep_c.to_rep_c_display(), suspect_rep_c.to_rep_c_display(), counter, threshold)
            }
            Self::Mismatch { reporter_rep_c, attester_rep_c, field, .. } => {
                format!("[{}] {} reporter={} attester={} field={}",
                    self.severity(), self.event_id(),
                    reporter_rep_c.to_rep_c_display(), attester_rep_c.to_rep_c_display(), field)
            }
            Self::VersionUnknown { reporter_rep_c, attester_rep_c, version, .. } => {
                format!("[{}] {} reporter={} attester={} version={}",
                    self.severity(), self.event_id(),
                    reporter_rep_c.to_rep_c_display(), attester_rep_c.to_rep_c_display(), version)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr() -> CubeAddr {
        CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    #[test]
    fn event_ids_unique() {
        let events = vec![
            AttestAuditEvent::SignFail { node_rep_c: addr(), error_code: 1, timestamp: 0 },
            AttestAuditEvent::HptpTimeout { node_rep_c: addr(), retry_count: 3, timestamp: 0 },
            AttestAuditEvent::BootMissing { node_rep_c: addr(), timestamp: 0 },
            AttestAuditEvent::SeqCorrupt { node_rep_c: addr(), timestamp: 0 },
            AttestAuditEvent::PufDegraded { node_rep_c: addr(), fuzzy_health: 1, timestamp: 0 },
            AttestAuditEvent::Suspect {
                reporter_rep_c: addr(), suspect_rep_c: addr(),
                counter: 3, threshold: 3, timestamp: 0,
            },
            AttestAuditEvent::Mismatch {
                reporter_rep_c: addr(), attester_rep_c: addr(),
                field: "kernel_hash".into(), timestamp: 0,
            },
            AttestAuditEvent::VersionUnknown {
                reporter_rep_c: addr(), attester_rep_c: addr(),
                version: 99, timestamp: 0,
            },
        ];
        let ids: Vec<&str> = events.iter().map(|e| e.event_id()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "all event IDs must be unique");
    }

    #[test]
    fn log_messages_use_dotted_repc() {
        let event = AttestAuditEvent::Suspect {
            reporter_rep_c: addr(),
            suspect_rep_c: CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            counter: 3,
            threshold: 3,
            timestamp: 0,
        };
        let msg = event.log_message();
        assert!(msg.contains("1.1.1.1.1.1.1.1.1.1.1.1.1"), "reporter should be dotted");
        assert!(msg.contains("2.1.1.1.1.1.1.1.1.1.1.1.1"), "suspect should be dotted");
        assert!(!msg.contains("hostname"), "no hostnames in attestation logs");
    }

    #[test]
    fn severity_levels_correct() {
        assert_eq!(
            AttestAuditEvent::SignFail { node_rep_c: addr(), error_code: 0, timestamp: 0 }.severity(),
            AttestSeverity::Critical
        );
        assert_eq!(
            AttestAuditEvent::Suspect {
                reporter_rep_c: addr(), suspect_rep_c: addr(),
                counter: 3, threshold: 3, timestamp: 0,
            }.severity(),
            AttestSeverity::Warning
        );
        assert_eq!(
            AttestAuditEvent::VersionUnknown {
                reporter_rep_c: addr(), attester_rep_c: addr(),
                version: 2, timestamp: 0,
            }.severity(),
            AttestSeverity::Info
        );
    }
}
