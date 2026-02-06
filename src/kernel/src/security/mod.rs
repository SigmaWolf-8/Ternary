//! PlenumNET Modal Security System
//!
//! Implements the three-mode security architecture from the Salvi Framework:
//! - **ModePhi** (Φ): Maximum privilege - kernel operations, cryptographic key management
//! - **ModeOne** (1): Standard operation - user processes, normal I/O
//! - **ModeZero** (0): Restricted/quarantine - untrusted code, sandboxed execution
//!
//! The security system provides:
//! - Capability-based access control with fine-grained tokens
//! - Security domain isolation with controlled inter-domain transitions
//! - Audit trail for all security-relevant events
//! - Policy engine for configurable access rules
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod domain;
pub mod capability;
pub mod audit;
pub mod policy;

use alloc::string::String;
use crate::memory::SecurityMode;
use crate::process::ProcessId;
use crate::timing::FemtosecondTimestamp;

#[derive(Debug, Clone)]
pub enum SecurityError {
    AccessDenied { subject: ProcessId, resource: ResourceId, action: Action },
    InsufficientCapability { required: CapabilityKind, held: Option<CapabilityKind> },
    InvalidTransition { from: SecurityMode, to: SecurityMode },
    CapabilityExpired { token_id: u64 },
    CapabilityRevoked { token_id: u64 },
    DomainNotFound(DomainId),
    PolicyViolation(String),
    AuditFull,
    InvalidToken,
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::AccessDenied { subject, resource, action } => {
                write!(f, "Access denied: process {} cannot {:?} resource {}", subject, action, resource.0)
            }
            SecurityError::InsufficientCapability { required, .. } => {
                write!(f, "Insufficient capability: {:?} required", required)
            }
            SecurityError::InvalidTransition { from, to } => {
                write!(f, "Invalid security transition: {:?} -> {:?}", from, to)
            }
            SecurityError::CapabilityExpired { token_id } => {
                write!(f, "Capability token {} expired", token_id)
            }
            SecurityError::CapabilityRevoked { token_id } => {
                write!(f, "Capability token {} revoked", token_id)
            }
            SecurityError::DomainNotFound(id) => {
                write!(f, "Security domain {} not found", id.0)
            }
            SecurityError::PolicyViolation(msg) => {
                write!(f, "Policy violation: {}", msg)
            }
            SecurityError::AuditFull => write!(f, "Audit log full"),
            SecurityError::InvalidToken => write!(f, "Invalid capability token"),
        }
    }
}

pub type SecurityResult<T> = core::result::Result<T, SecurityError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomainId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Execute,
    TernaryCompute,
    PhaseEncrypt,
    PhaseDecrypt,
    TimingAccess,
    IpcSend,
    IpcReceive,
    ModifyPolicy,
    CreateProcess,
    TerminateProcess,
    AllocateMemory,
    DeallocateMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    Memory,
    Process,
    Ipc,
    TernaryUnit,
    PhaseEncryption,
    Timing,
    Security,
    Io,
    Audit,
}

impl CapabilityKind {
    pub fn minimum_mode(&self) -> SecurityMode {
        match self {
            CapabilityKind::Security | CapabilityKind::Audit => SecurityMode::ModePhi,
            CapabilityKind::PhaseEncryption | CapabilityKind::Timing => SecurityMode::ModeOne,
            _ => SecurityMode::ModeZero,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_kind_minimum_mode() {
        assert_eq!(CapabilityKind::Security.minimum_mode(), SecurityMode::ModePhi);
        assert_eq!(CapabilityKind::Audit.minimum_mode(), SecurityMode::ModePhi);
        assert_eq!(CapabilityKind::PhaseEncryption.minimum_mode(), SecurityMode::ModeOne);
        assert_eq!(CapabilityKind::Timing.minimum_mode(), SecurityMode::ModeOne);
        assert_eq!(CapabilityKind::Memory.minimum_mode(), SecurityMode::ModeZero);
        assert_eq!(CapabilityKind::Process.minimum_mode(), SecurityMode::ModeZero);
    }

    #[test]
    fn test_resource_id() {
        let r1 = ResourceId(1);
        let r2 = ResourceId(2);
        assert_ne!(r1, r2);
        assert_eq!(r1, ResourceId(1));
    }

    #[test]
    fn test_domain_id() {
        let d1 = DomainId(0);
        let d2 = DomainId(1);
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::AccessDenied {
            subject: 1,
            resource: ResourceId(42),
            action: Action::Read,
        };
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("Access denied"));
        assert!(msg.contains("42"));
    }
}
