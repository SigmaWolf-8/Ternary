//! Security Audit Trail
//!
//! Records all security-relevant events for compliance and forensics.
//! Supports FINRA 613 and MiFID II audit requirements with femtosecond
//! precision timestamps.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::process::ProcessId;
use crate::timing::FemtosecondTimestamp;
use super::{Action, DomainId, ResourceId, SecurityResult};
use super::capability::TokenId;

const DEFAULT_MAX_ENTRIES: usize = 16384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
    Violation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEventKind {
    AccessGranted,
    AccessDenied,
    CapabilityCreated,
    CapabilityRevoked,
    CapabilityDelegated,
    DomainTransition,
    DomainTransitionDenied,
    PolicyChange,
    SecurityModeChange,
    ProcessCreated,
    ProcessTerminated,
    IpcEvent,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub sequence: u64,
    pub timestamp: FemtosecondTimestamp,
    pub kind: AuditEventKind,
    pub severity: AuditSeverity,
    pub subject: ProcessId,
    pub subject_mode: SecurityMode,
    pub resource: Option<ResourceId>,
    pub action: Option<Action>,
    pub domain: Option<DomainId>,
    pub token: Option<TokenId>,
    pub detail: String,
    pub checksum: u32,
}

impl AuditEntry {
    fn compute_checksum(
        sequence: u64,
        timestamp: u128,
        kind: AuditEventKind,
        subject: ProcessId,
    ) -> u32 {
        let mut hash: u32 = 0x5A1F1;
        hash = hash.wrapping_mul(31).wrapping_add(sequence as u32);
        hash = hash.wrapping_mul(31).wrapping_add((timestamp & 0xFFFFFFFF) as u32);
        hash = hash.wrapping_mul(31).wrapping_add(kind as u32);
        hash = hash.wrapping_mul(31).wrapping_add(subject as u32);
        hash
    }

    pub fn verify_integrity(&self) -> bool {
        let expected = Self::compute_checksum(
            self.sequence,
            self.timestamp.femtoseconds,
            self.kind,
            self.subject,
        );
        self.checksum == expected
    }
}

pub struct AuditLog {
    entries: VecDeque<AuditEntry>,
    max_entries: usize,
    next_sequence: u64,
    total_logged: u64,
    total_violations: u64,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: DEFAULT_MAX_ENTRIES,
            next_sequence: 0,
            total_logged: 0,
            total_violations: 0,
        }
    }

    pub fn with_capacity(max: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: max,
            next_sequence: 0,
            total_logged: 0,
            total_violations: 0,
        }
    }

    pub fn log(
        &mut self,
        kind: AuditEventKind,
        severity: AuditSeverity,
        subject: ProcessId,
        subject_mode: SecurityMode,
        timestamp: FemtosecondTimestamp,
        detail: String,
    ) -> SecurityResult<u64> {
        self.log_full(kind, severity, subject, subject_mode, timestamp, None, None, None, None, detail)
    }

    pub fn log_full(
        &mut self,
        kind: AuditEventKind,
        severity: AuditSeverity,
        subject: ProcessId,
        subject_mode: SecurityMode,
        timestamp: FemtosecondTimestamp,
        resource: Option<ResourceId>,
        action: Option<Action>,
        domain: Option<DomainId>,
        token: Option<TokenId>,
        detail: String,
    ) -> SecurityResult<u64> {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }

        let seq = self.next_sequence;
        self.next_sequence += 1;
        self.total_logged += 1;

        if severity == AuditSeverity::Violation {
            self.total_violations += 1;
        }

        let checksum = AuditEntry::compute_checksum(
            seq, timestamp.femtoseconds, kind, subject,
        );

        let entry = AuditEntry {
            sequence: seq,
            timestamp,
            kind,
            severity,
            subject,
            subject_mode,
            resource,
            action,
            domain,
            token,
            detail,
            checksum,
        };

        self.entries.push_back(entry);
        Ok(seq)
    }

    pub fn get(&self, sequence: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.sequence == sequence)
    }

    pub fn recent(&self, count: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    pub fn by_process(&self, pid: ProcessId) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.subject == pid).collect()
    }

    pub fn by_severity(&self, severity: AuditSeverity) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.severity == severity).collect()
    }

    pub fn by_kind(&self, kind: AuditEventKind) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.kind == kind).collect()
    }

    pub fn violations(&self) -> Vec<&AuditEntry> {
        self.by_severity(AuditSeverity::Violation)
    }

    pub fn verify_chain_integrity(&self) -> bool {
        self.entries.iter().all(|e| e.verify_integrity())
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_logged(&self) -> u64 {
        self.total_logged
    }

    pub fn total_violations(&self) -> u64 {
        self.total_violations
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts() -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(1_000_000)
    }

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new();
        assert_eq!(log.count(), 0);
        assert_eq!(log.total_logged(), 0);
    }

    #[test]
    fn test_audit_log_entry() {
        let mut log = AuditLog::new();
        let seq = log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(),
            String::from("Read access to resource 10"),
        ).unwrap();

        assert_eq!(seq, 0);
        assert_eq!(log.count(), 1);
    }

    #[test]
    fn test_audit_entry_integrity() {
        let mut log = AuditLog::new();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(),
            String::from("test"),
        ).unwrap();

        let entry = log.get(0).unwrap();
        assert!(entry.verify_integrity());
    }

    #[test]
    fn test_audit_chain_integrity() {
        let mut log = AuditLog::new();
        for i in 0..5 {
            log.log(
                AuditEventKind::AccessGranted, AuditSeverity::Info,
                i, SecurityMode::ModeOne, make_ts(),
                String::from("test"),
            ).unwrap();
        }
        assert!(log.verify_chain_integrity());
    }

    #[test]
    fn test_audit_by_process() {
        let mut log = AuditLog::new();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(), String::from("a"),
        ).unwrap();
        log.log(
            AuditEventKind::AccessDenied, AuditSeverity::Warning,
            2, SecurityMode::ModeZero, make_ts(), String::from("b"),
        ).unwrap();
        log.log(
            AuditEventKind::CapabilityCreated, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(), String::from("c"),
        ).unwrap();

        let entries = log.by_process(1);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_audit_violations() {
        let mut log = AuditLog::new();
        log.log(
            AuditEventKind::AccessDenied, AuditSeverity::Violation,
            1, SecurityMode::ModeZero, make_ts(),
            String::from("unauthorized access attempt"),
        ).unwrap();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            2, SecurityMode::ModeOne, make_ts(),
            String::from("normal access"),
        ).unwrap();

        assert_eq!(log.total_violations(), 1);
        assert_eq!(log.violations().len(), 1);
    }

    #[test]
    fn test_audit_capacity_eviction() {
        let mut log = AuditLog::with_capacity(3);
        for i in 0..5 {
            log.log(
                AuditEventKind::AccessGranted, AuditSeverity::Info,
                i, SecurityMode::ModeOne, make_ts(),
                String::from("test"),
            ).unwrap();
        }

        assert_eq!(log.count(), 3);
        assert_eq!(log.total_logged(), 5);
        assert!(log.get(0).is_none());
        assert!(log.get(2).is_some());
    }

    #[test]
    fn test_audit_recent() {
        let mut log = AuditLog::new();
        for i in 0..5 {
            log.log(
                AuditEventKind::AccessGranted, AuditSeverity::Info,
                i, SecurityMode::ModeOne, make_ts(),
                String::from("test"),
            ).unwrap();
        }

        let recent = log.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].sequence, 4);
        assert_eq!(recent[1].sequence, 3);
    }

    #[test]
    fn test_audit_by_kind() {
        let mut log = AuditLog::new();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(), String::from("a"),
        ).unwrap();
        log.log(
            AuditEventKind::DomainTransition, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(), String::from("b"),
        ).unwrap();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            2, SecurityMode::ModeOne, make_ts(), String::from("c"),
        ).unwrap();

        let granted = log.by_kind(AuditEventKind::AccessGranted);
        assert_eq!(granted.len(), 2);
    }

    #[test]
    fn test_audit_full_entry() {
        let mut log = AuditLog::new();
        log.log_full(
            AuditEventKind::CapabilityCreated, AuditSeverity::Info,
            1, SecurityMode::ModePhi, make_ts(),
            Some(ResourceId(42)), Some(Action::Read),
            Some(DomainId(0)), Some(TokenId(1)),
            String::from("granted read capability"),
        ).unwrap();

        let entry = log.get(0).unwrap();
        assert_eq!(entry.resource, Some(ResourceId(42)));
        assert!(entry.verify_integrity());
    }

    #[test]
    fn test_audit_clear() {
        let mut log = AuditLog::new();
        log.log(
            AuditEventKind::AccessGranted, AuditSeverity::Info,
            1, SecurityMode::ModeOne, make_ts(), String::from("test"),
        ).unwrap();
        log.clear();
        assert_eq!(log.count(), 0);
        assert_eq!(log.total_logged(), 1);
    }
}
