// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Continuous Attestation Service (Task #119, Gap 9)
//!
//! Cryptographic integrity layer answering "has this node been tampered with,
//! even if it's still running?" Complements operational health monitoring.
//!
//! ## Modules
//!
//! | Module | Task | Purpose |
//! |--------|------|---------|
//! | report | 1 | AttestationReport (10 fields, TritInt, Rep C wire) |
//! | signing | 1 | TL-DSA key derivation, context strings, Zeroize |
//! | merkle | 2 | Rolling Merkle tree (TIS-27, domain separation) |
//! | audit | 1 | 8 audit event types with severity |
//! | broadcast | 3 | HModal dispatch, HPTP jitter, per-link bandwidth backoff |
//! | verify | 4 | Replay protection, suspicion counters, FTS integration |
//! | versioning | 5 | Schema registry, upgrade window (4h auto-expiry) |
//! | failure | 6 | 7 failure modes, operator messages, service state |
//! | logging | — | Structured log output for Forma Codex 18∏ consumption |
//!
//! ## Signal Model
//!
//! The broadcast service uses HModal signal dispatch (TM-2026-028):
//! - duty = 1/R₂ = 1/4 → 25% dispatch (β), 75% idle (α)
//! - Idle phase: collect heartbeat challenges, build Merkle tree
//! - Dispatch phase: sign and broadcast attestation report
//!
//! ## Implementation Language
//!
//! Pure Rust. No TypeScript, no Node.js, no JavaScript in the attestation path.

pub mod report;
pub mod signing;
pub mod merkle;
pub mod audit;
pub mod broadcast;
pub mod verify;
pub mod versioning;
pub mod failure;
pub mod logging;

pub use report::{
    AttestationReport, SignedAttestationReport,
    BootMeasurements, PufHealth, ReportError, SCHEMA_VERSION,
};
pub use signing::{
    AttestationSigningKey,
    SIGNING_CONTEXT_PREFIX, KEY_DERIVATION_CONTEXT,
};
pub use merkle::{RollingMerkleTree, empty_tree_constant};
pub use audit::{AttestAuditEvent, AttestSeverity};
pub use broadcast::{
    BroadcastConfig, BroadcastState, LinkBandwidthState,
    HModalTiming, DispatchPhase,
};
pub use verify::{
    AttestationVerifier, NeighborAttestState,
    VerifyResult, SuspicionOutcome,
};
pub use versioning::{VersionRegistry, UpgradeWindow, SchemaVersionEntry};
pub use failure::{ServiceState, DegradedReason, FailureOutcome};
pub use logging::{AttestationLogger, LogEntry, ClassTrit};
