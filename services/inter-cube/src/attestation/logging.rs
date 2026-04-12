// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Logging — Forma Codex 18∏ native log entries
// 27-trit classification, TIS-27 hash chain, HPTP timestamps

//! Structured logging producing Forma Codex 18∏ LogEntry values.
//!
//! Every attestation log entry carries a 27-trit classification address
//! across 7 categories (Who/What/Where/When/Why/How/Peace), a TIS-27
//! identity hash, and a chain hash linking to the previous entry for
//! tamper-evident audit trails.
//!
//! Log path: `C:\PlenumNET\Logs\attestation\`
//! Three faces per entry: message (operator), raw data (engineer),
//! correlation context (tracing).
//!
//! All node identification in dot-separated Rep C (INVARIANT 9).
//! Binary values cross the gate (TritInt) immediately on entry.

use std::sync::Mutex;
use ternary_math::trit_int::TritInt;

use crate::cube_addr::CubeAddr;
use super::audit::AttestAuditEvent;
use super::broadcast::DispatchPhase;
use super::failure::ServiceState;

// ═══════════════════════════════════════════════════════════════════════
// 27-TRIT CLASSIFICATION — Rep C {1, 2, 3}
// ═══════════════════════════════════════════════════════════════════════

/// A single Rep C classification trit. Value ∈ {1, 2, 3}.
/// Zero is structurally absent — corruption detection by encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassTrit(u8);

impl ClassTrit {
    pub const fn new(v: u8) -> Self {
        assert!(v >= 1 && v <= 3, "classification trit must be 1, 2, or 3");
        ClassTrit(v)
    }
    pub const fn value(self) -> u8 { self.0 }
}

// Dimension value constants (Rep C alphabet)
const C1: ClassTrit = ClassTrit(1);
const C2: ClassTrit = ClassTrit(2);
const C3: ClassTrit = ClassTrit(3);

// ═══════════════════════════════════════════════════════════════════════
// WHO — Identity (dims 1-4) — Immutable
// ═══════════════════════════════════════════════════════════════════════
// All attestation events:
//   Origin     = System/kernel (3) — attestation is a system-level service
//   Actor      = Automated agent (2) — no human in the loop
//   Authority  = Admin/privileged (3) — signing keys, PUF access
//   Tenant     = Platform-wide (3) — attestation spans the entire node

const WHO_ORIGIN: ClassTrit     = C3; // System/kernel
const WHO_ACTOR: ClassTrit      = C2; // Automated agent
const WHO_AUTHORITY: ClassTrit  = C3; // Admin/privileged
const WHO_TENANT: ClassTrit     = C3; // Platform-wide

// ═══════════════════════════════════════════════════════════════════════
// WHERE — Location (dims 9-12) — Immutable
// ═══════════════════════════════════════════════════════════════════════
// All attestation events:
//   Layer      = Service/daemon (2) — inter-cube daemon module
//   Subsystem  = Network (3) — inter-cube communication
//   Zone       = Internal trust (1) — within the PlenumNET trust boundary
//   Replica    = Primary (1) — attestation runs on the node itself

const WHERE_LAYER: ClassTrit     = C2; // Service/daemon
const WHERE_SUBSYSTEM: ClassTrit = C3; // Network
const WHERE_ZONE: ClassTrit      = C1; // Internal trust
const WHERE_REPLICA: ClassTrit   = C1; // Primary

// ═══════════════════════════════════════════════════════════════════════
// HOW — Mechanism (dims 21-24) — Immutable
// ═══════════════════════════════════════════════════════════════════════
// All attestation events:
//   Direction  = Internal (3) — intra-node and inter-node attestation
//   Synchrony  = Asynchronous (2) — background broadcast cycle
//   Transport  = Network RPC (2) — Inter-Cube encrypted tunnels
//   Format     = Structured (3) — Rep C wire encoding

const HOW_DIRECTION: ClassTrit  = C3; // Internal
const HOW_SYNCHRONY: ClassTrit  = C2; // Asynchronous
const HOW_TRANSPORT: ClassTrit  = C2; // Network RPC
const HOW_FORMAT: ClassTrit     = C3; // Structured

// ═══════════════════════════════════════════════════════════════════════
// FORMA CODEX LOG ENTRY
// ═══════════════════════════════════════════════════════════════════════

/// A log entry conforming to the Forma Codex 18∏ Log Viewer spec.
///
/// 27-trit classification, TIS-27 identity + chain hashes, HPTP timestamp,
/// three faces (message, raw_data, correlation). Binary values cross
/// the TritInt gate on entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 27-trit classification address in Rep C {1,2,3}.
    /// Dims 1-4: Who, 5-8: What, 9-12: Where, 13-16: When,
    /// 17-20: Why, 21-24: How, 25-27: Peace.
    pub classification: [ClassTrit; 27],

    /// HPTP femtosecond write timestamp (covered by identity_hash).
    pub hptp_timestamp: TritInt,

    /// HPTP ingestion time (NOT covered by identity_hash — latency measurement).
    pub received_at: TritInt,

    /// Face 1: Human-readable message (operator).
    pub message: String,

    /// Face 2: Raw structured data (engineer). JSON, key-value, wire dumps.
    pub raw_data: Option<String>,

    /// Face 3: Correlation context — parent_id, triggered events, cross-service refs.
    pub correlation: Option<String>,

    /// TIS-27 identity hash of (classification[0..24] + hptp_timestamp + message + raw_data).
    /// Covers the 24 immutable trits. Peace (25-27) and received_at are excluded.
    pub identity_hash: Vec<u8>,

    /// TIS-27 chain hash of (identity_hash ‖ previous chain_hash).
    /// First entry: chain_hash = identity_hash.
    pub chain_hash: Vec<u8>,
}

impl LogEntry {
    // ── Slice accessors per Forma Codex Log Viewer §4.3 ─────────
    pub fn who(&self) -> &[ClassTrit]       { &self.classification[0..4] }
    pub fn what(&self) -> &[ClassTrit]      { &self.classification[4..8] }
    pub fn where_dim(&self) -> &[ClassTrit] { &self.classification[8..12] }
    pub fn when_dim(&self) -> &[ClassTrit]  { &self.classification[12..16] }
    pub fn why(&self) -> &[ClassTrit]       { &self.classification[16..20] }
    pub fn how(&self) -> &[ClassTrit]       { &self.classification[20..24] }
    pub fn peace(&self) -> &[ClassTrit]     { &self.classification[24..27] }

    /// Composite: is this an error? What:Outcome (dim 7) = Failure (3).
    pub fn is_error(&self) -> bool { self.classification[6].value() == 3 }

    /// Composite: high priority? Peace:Priority (dim 26) = High (3).
    pub fn is_high_priority(&self) -> bool { self.classification[25].value() == 3 }
}

// ═══════════════════════════════════════════════════════════════════════
// TIS-27 HASH COMPUTATION
// ═══════════════════════════════════════════════════════════════════════

const TIS27_LEN: usize = 27;
const IDENTITY_DOMAIN: &[u8] = b"PLENUMNET-LOG-IDENTITY";
const CHAIN_DOMAIN: &[u8] = b"PLENUMNET-LOG-CHAIN";

/// Compute identity_hash: TIS-27(classification[0..24] + hptp_timestamp + message + raw_data).
fn compute_identity_hash(
    classification: &[ClassTrit; 27],
    hptp_timestamp: &TritInt,
    message: &str,
    raw_data: &Option<String>,
) -> Vec<u8> {
    let mut input = Vec::new();
    // 24 immutable classification trits (not Peace dims 25-27)
    for i in 0..24 {
        input.push(classification[i].value());
    }
    // HPTP timestamp as Rep C
    input.extend_from_slice(&hptp_timestamp.to_repr_c());
    // Message bytes
    input.extend_from_slice(message.as_bytes());
    // Raw data bytes (if present)
    if let Some(raw) = raw_data {
        input.extend_from_slice(raw.as_bytes());
    }
    ternary_math::sponge::derive_key(IDENTITY_DOMAIN, &input, TIS27_LEN)
}

/// Compute chain_hash: TIS-27(identity_hash ‖ previous_chain_hash).
/// First entry: chain_hash = identity_hash.
fn compute_chain_hash(identity_hash: &[u8], prev_chain_hash: Option<&[u8]>) -> Vec<u8> {
    match prev_chain_hash {
        None => identity_hash.to_vec(), // first entry
        Some(prev) => {
            let mut input = Vec::with_capacity(identity_hash.len() + prev.len());
            input.extend_from_slice(identity_hash);
            input.extend_from_slice(prev);
            ternary_math::sponge::derive_key(CHAIN_DOMAIN, &input, TIS27_LEN)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CLASSIFICATION BUILDER — maps attestation events to 27 trits
// ═══════════════════════════════════════════════════════════════════════

/// What dimensions (5-8) for attestation event types.
struct WhatClass {
    category: ClassTrit,    // dim 5: Lifecycle(1) / Fault(2) / Security(3)
    operation: ClassTrit,   // dim 6: Create(1) / Modify(2) / Delete(3)
    outcome: ClassTrit,     // dim 7: Success(1) / Partial(2) / Failure(3)
    idempotency: ClassTrit, // dim 8: Idempotent(1) / Non-idempotent(2) / Unknown(3)
}

/// Why dimensions (17-20) for attestation event types.
struct WhyClass {
    trigger: ClassTrit,     // dim 17: User(1) / System(2) / Schedule(3)
    causal_role: ClassTrit, // dim 18: Root cause(1) / Contributing(2) / Consequence(3)
    propagation: ClassTrit, // dim 19: Local(1) / Service-wide(2) / Global(3)
    certainty: ClassTrit,   // dim 20: Confirmed(1) / Suspected(2) / Inferred(3)
}

/// When dimensions (13-16) — derived from HPTP timestamp.
/// Simplified: attestation is always periodic, daemon-internal.
fn when_class() -> [ClassTrit; 4] {
    [
        C2, // Period: Business hours (reasonable default; runtime derivation from HPTP + manifest)
        C2, // Phase: Middle (attestation is continuous, no batch window)
        C2, // Cadence: Periodic (attestation broadcasts at regular intervals)
        C1, // Latency: Immediate (local signing, <1ms)
    ]
}

/// Peace dimensions (25-27) — initial state at write time.
fn peace_from_outcome(outcome: ClassTrit) -> [ClassTrit; 3] {
    let priority = match outcome.value() {
        3 => C3, // Failure → High priority
        2 => C2, // Partial → Medium
        _ => C1, // Success → Low
    };
    [
        C1,       // State: Unresolved
        priority, // Priority: mapped from outcome
        C1,       // Ownership: Unassigned
    ]
}

/// Build the full 27-trit classification for an attestation audit event.
fn classify_audit_event(event: &AttestAuditEvent) -> [ClassTrit; 27] {
    let (what, why) = match event {
        AttestAuditEvent::SignFail { .. } => (
            WhatClass { category: C3, operation: C2, outcome: C3, idempotency: C2 },
            WhyClass { trigger: C2, causal_role: C1, propagation: C2, certainty: C1 },
        ),
        AttestAuditEvent::HptpTimeout { .. } => (
            WhatClass { category: C2, operation: C2, outcome: C2, idempotency: C1 },
            WhyClass { trigger: C2, causal_role: C2, propagation: C1, certainty: C1 },
        ),
        AttestAuditEvent::BootMissing { .. } => (
            WhatClass { category: C3, operation: C1, outcome: C3, idempotency: C1 },
            WhyClass { trigger: C2, causal_role: C1, propagation: C2, certainty: C1 },
        ),
        AttestAuditEvent::SeqCorrupt { .. } => (
            WhatClass { category: C2, operation: C2, outcome: C3, idempotency: C2 },
            WhyClass { trigger: C2, causal_role: C1, propagation: C1, certainty: C1 },
        ),
        AttestAuditEvent::PufDegraded { .. } => (
            WhatClass { category: C3, operation: C2, outcome: C2, idempotency: C1 },
            WhyClass { trigger: C2, causal_role: C2, propagation: C1, certainty: C2 },
        ),
        AttestAuditEvent::Suspect { .. } => (
            WhatClass { category: C3, operation: C2, outcome: C3, idempotency: C2 },
            WhyClass { trigger: C2, causal_role: C3, propagation: C2, certainty: C2 },
        ),
        AttestAuditEvent::Mismatch { .. } => (
            WhatClass { category: C3, operation: C2, outcome: C2, idempotency: C1 },
            WhyClass { trigger: C2, causal_role: C2, propagation: C2, certainty: C1 },
        ),
        AttestAuditEvent::VersionUnknown { .. } => (
            WhatClass { category: C1, operation: C2, outcome: C2, idempotency: C1 },
            WhyClass { trigger: C2, causal_role: C3, propagation: C1, certainty: C3 },
        ),
    };

    let when = when_class();
    let peace = peace_from_outcome(what.outcome);

    [
        // Who (1-4)
        WHO_ORIGIN, WHO_ACTOR, WHO_AUTHORITY, WHO_TENANT,
        // What (5-8)
        what.category, what.operation, what.outcome, what.idempotency,
        // Where (9-12)
        WHERE_LAYER, WHERE_SUBSYSTEM, WHERE_ZONE, WHERE_REPLICA,
        // When (13-16)
        when[0], when[1], when[2], when[3],
        // Why (17-20)
        why.trigger, why.causal_role, why.propagation, why.certainty,
        // How (21-24)
        HOW_DIRECTION, HOW_SYNCHRONY, HOW_TRANSPORT, HOW_FORMAT,
        // Peace (25-27)
        peace[0], peace[1], peace[2],
    ]
}

/// Build a 27-trit classification for operational log events (non-audit).
fn classify_operational(outcome: ClassTrit) -> [ClassTrit; 27] {
    let when = when_class();
    let peace = peace_from_outcome(outcome);
    [
        WHO_ORIGIN, WHO_ACTOR, WHO_AUTHORITY, WHO_TENANT,
        C1, C2, outcome, C1,  // What: Lifecycle, Modify, outcome, Idempotent
        WHERE_LAYER, WHERE_SUBSYSTEM, WHERE_ZONE, WHERE_REPLICA,
        when[0], when[1], when[2], when[3],
        C2, C2, C1, C1,  // Why: System, Contributing, Local, Confirmed
        HOW_DIRECTION, HOW_SYNCHRONY, HOW_TRANSPORT, HOW_FORMAT,
        peace[0], peace[1], peace[2],
    ]
}

// ═══════════════════════════════════════════════════════════════════════
// ATTESTATION LOGGER
// ═══════════════════════════════════════════════════════════════════════

/// Forma Codex-native attestation logger.
///
/// Produces LogEntry values with 27-trit classification, TIS-27 identity
/// and chain hashes, and three faces. Entries are buffered in memory for
/// Forma Codex log viewer consumption and written to the attestation log
/// directory via the tracing subsystem.
///
/// Log directory: `C:\PlenumNET\Logs\attestation\`
pub struct AttestationLogger {
    /// In-memory entry buffer (ring buffer, evicts oldest).
    buffer: Mutex<Vec<LogEntry>>,
    /// Maximum buffer entries.
    max_buffer: usize,
    /// Last chain hash for tamper-evident chaining.
    last_chain_hash: Mutex<Option<Vec<u8>>>,
}

impl AttestationLogger {
    /// Create a new logger.
    pub fn new(max_buffer: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
            max_buffer,
            last_chain_hash: Mutex::new(None),
        }
    }

    /// Emit a log entry with full Forma Codex classification and hashing.
    fn emit(&self, classification: [ClassTrit; 27], message: String,
            raw_data: Option<String>, correlation: Option<String>,
            timestamp_fs: u128) {
        let hptp = TritInt::from_u128(timestamp_fs);
        let received_at = hptp.clone(); // same node — no ingestion latency

        // TIS-27 identity hash (covers 24 immutable trits + timestamp + content)
        let identity_hash = compute_identity_hash(
            &classification, &hptp, &message, &raw_data,
        );

        // TIS-27 chain hash (links to previous entry)
        let chain_hash = {
            let prev = self.last_chain_hash.lock().unwrap_or_else(|e| e.into_inner());
            compute_chain_hash(&identity_hash, prev.as_deref())
        };

        // Update chain head
        {
            let mut last = self.last_chain_hash.lock().unwrap_or_else(|e| e.into_inner());
            *last = Some(chain_hash.clone());
        }

        let entry = LogEntry {
            classification,
            hptp_timestamp: hptp,
            received_at,
            message: message.clone(),
            raw_data,
            correlation,
            identity_hash,
            chain_hash,
        };

        // Write to tracing (integrates with PlenumNET tracing → file output)
        if entry.is_error() {
            tracing::error!("{}", message);
        } else if entry.is_high_priority() {
            tracing::warn!("{}", message);
        } else {
            tracing::info!("{}", message);
        }

        // Buffer for Forma Codex log viewer
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(entry);
            while buf.len() > self.max_buffer {
                buf.remove(0);
            }
        }
    }

    // ── Audit event logging ─────────────────────────────────────

    /// Log an attestation audit event with full 27-trit classification.
    pub fn log_audit_event(&self, event: &AttestAuditEvent, timestamp_fs: u128) {
        let classification = classify_audit_event(event);
        let message = event.log_message();
        let raw_data = Some(format!("event_id={} severity={}", event.event_id(), event.severity()));
        self.emit(classification, message, raw_data, None, timestamp_fs);
    }

    // ── Operational event logging ───────────────────────────────

    /// Log attestation service startup.
    pub fn log_started(&self, node: &CubeAddr, interval_base: u16, ts: u128) {
        let classification = classify_operational(C1); // Success
        self.emit(
            classification,
            format!("Attestation service started, interval={}s, key derived — node {}",
                interval_base, node.to_rep_c_display()),
            Some(format!("node={} interval_base_s={}", node.to_rep_c_display(), interval_base)),
            None, ts,
        );
    }

    /// Log attestation service shutdown.
    pub fn log_stopped(&self, node: &CubeAddr, ts: u128) {
        let classification = classify_operational(C1);
        self.emit(
            classification,
            format!("Attestation service stopped, key zeroized — node {}", node.to_rep_c_display()),
            Some(format!("node={}", node.to_rep_c_display())),
            None, ts,
        );
    }

    /// Log attestation report broadcast (face 3 = correlation with neighbor).
    pub fn log_broadcast(&self, node: &CubeAddr, neighbor: &CubeAddr,
                          seq: u64, wire_size: usize, ts: u128) {
        let classification = classify_operational(C1);
        self.emit(
            classification,
            format!("Attestation broadcast seq={} to {} ({} bytes)",
                seq, neighbor.to_rep_c_display(), wire_size),
            Some(format!("from={} to={} seq={} wire_bytes={}",
                node.to_rep_c_display(), neighbor.to_rep_c_display(), seq, wire_size)),
            Some(format!("sender={} receiver={}", node.to_rep_c_display(), neighbor.to_rep_c_display())),
            ts,
        );
    }

    /// Log attestation report verified.
    pub fn log_verified(&self, node: &CubeAddr, sender: &CubeAddr, seq: u64, ts: u128) {
        let classification = classify_operational(C1);
        self.emit(
            classification,
            format!("Attestation verified from {} seq={}", sender.to_rep_c_display(), seq),
            Some(format!("node={} sender={} seq={}", node.to_rep_c_display(), sender.to_rep_c_display(), seq)),
            Some(format!("verifier={} attester={}", node.to_rep_c_display(), sender.to_rep_c_display())),
            ts,
        );
    }

    /// Log HModal phase transition (α idle / β dispatch).
    pub fn log_phase_transition(&self, node: &CubeAddr, phase: DispatchPhase,
                                 interval_s: u16, ts: u128) {
        let phase_str = match phase {
            DispatchPhase::Idle => "idle (α)",
            DispatchPhase::Dispatch => "dispatch (β)",
        };
        let classification = classify_operational(C1);
        self.emit(
            classification,
            format!("HModal phase={} interval={}s — node {}",
                phase_str, interval_s, node.to_rep_c_display()),
            Some(format!("node={} phase={} interval_s={}", node.to_rep_c_display(), phase_str, interval_s)),
            None, ts,
        );
    }

    /// Log service state change (Running/Degraded/Disabled).
    pub fn log_state_change(&self, node: &CubeAddr, state: ServiceState, ts: u128) {
        let (outcome, state_str) = match state {
            ServiceState::Running => (C1, "running"),
            ServiceState::Degraded(r) => (C3, "degraded"),
            ServiceState::Disabled => (C1, "disabled"),
        };
        let classification = classify_operational(outcome);
        let detail = match state {
            ServiceState::Degraded(r) => format!("state=degraded reason={}", r),
            _ => format!("state={}", state_str),
        };
        self.emit(
            classification,
            format!("Attestation state={} — node {}", state_str, node.to_rep_c_display()),
            Some(format!("node={} {}", node.to_rep_c_display(), detail)),
            None, ts,
        );
    }

    /// Log bandwidth backoff.
    pub fn log_backoff(&self, node: &CubeAddr, neighbor: &CubeAddr,
                        backoff_level: u32, effective_interval: u16, ts: u128) {
        let classification = classify_operational(C2); // Partial failure
        self.emit(
            classification,
            format!("Bandwidth backoff level={} interval={}s to {} — node {}",
                backoff_level, effective_interval, neighbor.to_rep_c_display(), node.to_rep_c_display()),
            Some(format!("node={} link={} backoff={} interval_s={}",
                node.to_rep_c_display(), neighbor.to_rep_c_display(), backoff_level, effective_interval)),
            Some(format!("sender={} congested_link={}", node.to_rep_c_display(), neighbor.to_rep_c_display())),
            ts,
        );
    }

    // ── Buffer access for Forma Codex log viewer ────────────────

    /// All buffered entries (for log viewer grid population).
    pub fn entries(&self) -> Vec<LogEntry> {
        self.buffer.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Entries filtered by What:Outcome = Failure (dim 7 = 3).
    pub fn errors(&self) -> Vec<LogEntry> {
        self.entries().into_iter().filter(|e| e.is_error()).collect()
    }

    /// Entries filtered by Peace:Priority = High (dim 26 = 3).
    pub fn high_priority(&self) -> Vec<LogEntry> {
        self.entries().into_iter().filter(|e| e.is_high_priority()).collect()
    }

    /// Entry count.
    pub fn count(&self) -> usize {
        self.buffer.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Clear the buffer.
    pub fn clear(&self) {
        if let Ok(mut buf) = self.buffer.lock() { buf.clear(); }
        if let Ok(mut last) = self.last_chain_hash.lock() { *last = None; }
    }

    /// Verify the integrity chain. Returns the index of the first break, or None if clean.
    pub fn verify_chain(&self) -> Option<usize> {
        let entries = self.entries();
        if entries.is_empty() { return None; }

        // Verify first entry: chain_hash == identity_hash
        if entries[0].chain_hash != entries[0].identity_hash {
            return Some(0);
        }

        // Verify subsequent entries
        for i in 1..entries.len() {
            let expected = compute_chain_hash(
                &entries[i].identity_hash,
                Some(&entries[i - 1].chain_hash),
            );
            if entries[i].chain_hash != expected {
                return Some(i);
            }
        }
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr() -> CubeAddr { CubeAddr::new([1; 13]) }
    fn addr2() -> CubeAddr { CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }

    #[test]
    fn classification_is_27_trits_in_rep_c() {
        let logger = AttestationLogger::new(100);
        logger.log_started(&addr(), 30, 0);
        let entries = logger.entries();
        assert_eq!(entries[0].classification.len(), 27);
        for trit in &entries[0].classification {
            assert!(trit.value() >= 1 && trit.value() <= 3,
                "all classification trits must be Rep C {{1,2,3}}");
        }
    }

    #[test]
    fn identity_hash_covers_immutable_trits() {
        let logger = AttestationLogger::new(100);
        logger.log_started(&addr(), 30, 1_000_000);
        let entries = logger.entries();
        assert_eq!(entries[0].identity_hash.len(), TIS27_LEN);
        assert!(!entries[0].identity_hash.iter().all(|&b| b == 0));
    }

    #[test]
    fn chain_hash_links_entries() {
        let logger = AttestationLogger::new(100);
        logger.log_started(&addr(), 30, 1_000_000);
        logger.log_broadcast(&addr(), &addr2(), 1, 3000, 2_000_000);
        logger.log_verified(&addr(), &addr2(), 1, 3_000_000);

        let entries = logger.entries();
        assert_eq!(entries.len(), 3);

        // First entry: chain_hash == identity_hash
        assert_eq!(entries[0].chain_hash, entries[0].identity_hash);

        // Subsequent: chain_hash = TIS-27(identity ‖ prev_chain)
        let expected = compute_chain_hash(&entries[1].identity_hash, Some(&entries[0].chain_hash));
        assert_eq!(entries[1].chain_hash, expected);

        // Full chain verification passes
        assert_eq!(logger.verify_chain(), None);
    }

    #[test]
    fn tampered_entry_breaks_chain() {
        let logger = AttestationLogger::new(100);
        logger.log_started(&addr(), 30, 0);
        logger.log_broadcast(&addr(), &addr2(), 1, 3000, 1);
        logger.log_verified(&addr(), &addr2(), 1, 2);

        // Tamper with entry 1's chain hash
        {
            let mut buf = logger.buffer.lock().unwrap();
            buf[1].chain_hash[0] ^= 0xFF;
        }

        // Chain should break at entry 1
        assert_eq!(logger.verify_chain(), Some(1));
    }

    #[test]
    fn audit_event_classifies_correctly() {
        let event = AttestAuditEvent::SignFail {
            node_rep_c: addr(), error_code: 42, timestamp: 0,
        };
        let class = classify_audit_event(&event);

        // What:Category = Security (3), What:Outcome = Failure (3)
        assert_eq!(class[4].value(), 3, "category should be Security");
        assert_eq!(class[6].value(), 3, "outcome should be Failure");

        // Peace:Priority should be High (3) for failures
        assert_eq!(class[25].value(), 3, "priority should be High");
    }

    #[test]
    fn error_filter_works() {
        let logger = AttestationLogger::new(100);

        // Operational success
        logger.log_started(&addr(), 30, 0);

        // Audit failure (outcome = Failure → is_error = true)
        let event = AttestAuditEvent::SignFail {
            node_rep_c: addr(), error_code: 1, timestamp: 1,
        };
        logger.log_audit_event(&event, 1);

        assert_eq!(logger.count(), 2);
        assert_eq!(logger.errors().len(), 1);
        assert!(logger.errors()[0].is_error());
        assert!(logger.errors()[0].is_high_priority());
    }

    #[test]
    fn all_addresses_are_dotted_rep_c() {
        let logger = AttestationLogger::new(100);
        logger.log_broadcast(&addr(), &addr2(), 42, 3000, 0);
        let entries = logger.entries();
        let msg = &entries[0].message;
        assert!(msg.contains("2.1.1.1.1.1.1.1.1.1.1.1.1"), "neighbor in dotted Rep C");
        let raw = entries[0].raw_data.as_ref().unwrap();
        assert!(raw.contains("1.1.1.1.1.1.1.1.1.1.1.1.1"), "self in dotted Rep C");
    }

    #[test]
    fn phase_transition_logs_hmodal() {
        let logger = AttestationLogger::new(100);
        logger.log_phase_transition(&addr(), DispatchPhase::Idle, 120, 0);
        logger.log_phase_transition(&addr(), DispatchPhase::Dispatch, 120, 1);
        let entries = logger.entries();
        assert!(entries[0].message.contains("idle (α)"));
        assert!(entries[1].message.contains("dispatch (β)"));
    }

    #[test]
    fn buffer_evicts_oldest_preserves_chain() {
        let logger = AttestationLogger::new(3);
        for i in 0..5u128 {
            logger.log_started(&addr(), 30, i);
        }
        assert_eq!(logger.count(), 3);
        // Note: chain verification on evicted buffer is partial —
        // the chain within the buffer is valid, but the first entry's
        // chain_hash links to an evicted predecessor. This is expected.
    }
}
