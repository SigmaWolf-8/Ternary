// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Attestation Report — Wire-format data structure
//!
//! The attestation report carries all 10 mandatory fields defined in
//! Task #119 §Attestation Report Structure. All integer fields use
//! TritInt (INVARIANT 8). Wire serialization uses Rep C encoding
//! (INVARIANT 3). Operator-facing displays use dot-separated Rep C.

use ternary_math::trit_int::TritInt;
use zeroize::Zeroize;

use crate::cube_addr::CubeAddr;
use crate::fts::NeighborState;

// ═══════════════════════════════════════════════════════════════════════
// SCHEMA VERSION
// ═══════════════════════════════════════════════════════════════════════

/// Current attestation report schema version.
/// Monotonic u16, initial value 1. Compared as unsigned integer (no semver).
pub const SCHEMA_VERSION: u16 = 1;

// ═══════════════════════════════════════════════════════════════════════
// BOOT MEASUREMENTS
// ═══════════════════════════════════════════════════════════════════════

/// Boot measurement data from firmware_sign.rs measured boot.
/// All values stored as TritInt; carried in Rep C on the wire.
#[derive(Debug, Clone, PartialEq)]
pub struct BootMeasurements {
    /// Firmware hash from measured boot (TLSponge-385 output).
    pub firmware_hash: Vec<u8>,
    /// Anti-rollback counter value.
    pub anti_rollback_counter: TritInt,
}

// ═══════════════════════════════════════════════════════════════════════
// PUF SELF-TEST RESULT
// ═══════════════════════════════════════════════════════════════════════

/// PUF self-test result including fuzzy extractor health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PufHealth {
    /// PUF healthy — fuzzy extractor operating within parameters.
    Healthy = 0,
    /// PUF degraded — fuzzy extractor health check returned warning.
    Degraded = 1,
    /// PUF failed — fuzzy extractor health check returned error.
    Failed = 2,
    /// PUF unavailable — hardware not responding.
    Unavailable = 3,
}

// ═══════════════════════════════════════════════════════════════════════
// ATTESTATION REPORT
// ═══════════════════════════════════════════════════════════════════════

/// A signed attestation report broadcast to geometric neighbors.
///
/// All 10 mandatory fields per Task #119 §Attestation Report Structure.
/// All integer fields use TritInt (INVARIANT 8 enforced by type).
/// Wire serialization uses Rep C encoding (INVARIANT 3).
///
/// Maximum report size: ≤ 4KB Rep C-encoded.
#[derive(Debug, Clone, PartialEq)]
pub struct AttestationReport {
    // ── Field 1: Attesting node Rep C address ────────────────────
    /// 13-trit Rep C address of the attesting node.
    /// Mandatory signed field. Verifiers check this matches the
    /// expected neighbor and the signing key's registered address.
    /// Binary encoding for wire and signing; dot-separated for display.
    pub node_addr: CubeAddr,

    // ── Field 2: Monotonic sequence number ───────────────────────
    /// Per-sender monotonic sequence number. TritInt auto-sizes —
    /// no fixed trit width, no overflow concern. Write-ahead persisted
    /// via persistence.rs. Receivers reject seq ≤ last accepted.
    pub sequence: TritInt,

    // ── Field 3: HPTP timestamp ──────────────────────────────────
    /// Femtoseconds since the Salvi Epoch (2025-04-01T00:00:00Z).
    /// TritInt auto-sizes to exact precision.
    pub timestamp: TritInt,

    // ── Field 4: Schema version ──────────────────────────────────
    /// Monotonic schema version (u16 value, carried as TritInt).
    pub schema_version: TritInt,

    // ── Field 5: Boot measurements ───────────────────────────────
    /// Firmware hashes and anti-rollback counters from measured boot.
    pub boot_measurements: BootMeasurements,

    // ── Field 6: Kernel integrity hash ───────────────────────────
    /// TLSponge-385 hash over the kernel binary image.
    /// Security-critical — forged hash = attestation bypass.
    /// Computed at build time, deterministic.
    pub kernel_hash: Vec<u8>,

    // ── Field 7: PUF self-test result ────────────────────────────
    /// PUF health status including fuzzy extractor state.
    pub puf_health: PufHealth,

    // ── Field 8: FTS healing state ───────────────────────────────
    /// Currently active FTS state for this node.
    pub fts_state: NeighborState,

    // ── Field 9: FTS/GLB configuration fingerprint ───────────────
    /// TIS-27 hash over canonical JSON serialization of FTS/GLB config.
    /// Canonical: sorted keys, no whitespace, UTF-8, integer/ASCII only.
    pub config_fingerprint: Vec<u8>,

    // ── Field 10: Liveness proof Merkle root ─────────────────────
    /// O(1) constant-size root of the rolling Merkle tree.
    /// TIS-27 hashed with domain separation (leaf=0, internal=1).
    pub merkle_root: Vec<u8>,
}

/// TL-DSA signature over an attestation report.
#[derive(Debug, Clone)]
pub struct SignedAttestationReport {
    /// The attestation report payload.
    pub report: AttestationReport,
    /// TL-DSA signature over the Rep C-encoded report.
    /// Context: "PLENUMNET-ATTEST-v1.0" ‖ signer_rep_c_addr.
    pub signature: Vec<u8>,
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE FORMAT: Rep C encoding (INVARIANT 3)
// ═══════════════════════════════════════════════════════════════════════

impl AttestationReport {
    /// Serialize the report to Rep C wire format for signing and transmission.
    /// All fields are included in the signed payload.
    pub fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(3072); // ~3KB estimate

        // Field 1: Node address (13 bytes, Rep C)
        out.extend_from_slice(&self.node_addr.to_bytes());

        // Field 2: Sequence number (TritInt → Rep C)
        let seq_rc = self.sequence.to_repr_c();
        encode_length_prefixed(&mut out, &seq_rc);

        // Field 3: Timestamp (TritInt → Rep C)
        let ts_rc = self.timestamp.to_repr_c();
        encode_length_prefixed(&mut out, &ts_rc);

        // Field 4: Schema version (TritInt → Rep C)
        let ver_rc = self.schema_version.to_repr_c();
        encode_length_prefixed(&mut out, &ver_rc);

        // Field 5: Boot measurements
        encode_length_prefixed(&mut out, &self.boot_measurements.firmware_hash);
        let arc_rc = self.boot_measurements.anti_rollback_counter.to_repr_c();
        encode_length_prefixed(&mut out, &arc_rc);

        // Field 6: Kernel integrity hash
        encode_length_prefixed(&mut out, &self.kernel_hash);

        // Field 7: PUF health (single byte)
        out.push(self.puf_health as u8);

        // Field 8: FTS state (single byte)
        out.push(fts_state_to_wire(self.fts_state));

        // Field 9: Config fingerprint
        encode_length_prefixed(&mut out, &self.config_fingerprint);

        // Field 10: Merkle root
        encode_length_prefixed(&mut out, &self.merkle_root);

        out
    }

    /// Deserialize from Rep C wire format.
    pub fn from_wire(data: &[u8]) -> Result<Self, ReportError> {
        let mut pos = 0;

        // Field 1: Node address (13 bytes)
        if pos + 13 > data.len() { return Err(ReportError::TooShort); }
        let node_addr = CubeAddr::try_from_bytes(&data[pos..pos + 13])
            .ok_or(ReportError::InvalidAddress)?;
        pos += 13;

        // Field 2: Sequence
        let (seq_rc, n) = decode_length_prefixed(data, pos)?;
        pos += n;
        let sequence = TritInt::try_from_repr_c(&seq_rc)
            .map_err(|_| ReportError::InvalidTritInt("sequence"))?;

        // Field 3: Timestamp
        let (ts_rc, n) = decode_length_prefixed(data, pos)?;
        pos += n;
        let timestamp = TritInt::try_from_repr_c(&ts_rc)
            .map_err(|_| ReportError::InvalidTritInt("timestamp"))?;

        // Field 4: Schema version
        let (ver_rc, n) = decode_length_prefixed(data, pos)?;
        pos += n;
        let schema_version = TritInt::try_from_repr_c(&ver_rc)
            .map_err(|_| ReportError::InvalidTritInt("schema_version"))?;

        // Field 5: Boot measurements
        let (fw_hash, n) = decode_length_prefixed(data, pos)?;
        pos += n;
        let (arc_rc, n) = decode_length_prefixed(data, pos)?;
        pos += n;
        let anti_rollback_counter = TritInt::try_from_repr_c(&arc_rc)
            .map_err(|_| ReportError::InvalidTritInt("anti_rollback_counter"))?;

        // Field 6: Kernel hash
        let (kernel_hash, n) = decode_length_prefixed(data, pos)?;
        pos += n;

        // Field 7: PUF health
        if pos >= data.len() { return Err(ReportError::TooShort); }
        let puf_health = match data[pos] {
            0 => PufHealth::Healthy,
            1 => PufHealth::Degraded,
            2 => PufHealth::Failed,
            3 => PufHealth::Unavailable,
            v => return Err(ReportError::InvalidPufHealth(v)),
        };
        pos += 1;

        // Field 8: FTS state
        if pos >= data.len() { return Err(ReportError::TooShort); }
        let fts_state = fts_state_from_wire(data[pos])?;
        pos += 1;

        // Field 9: Config fingerprint
        let (config_fingerprint, n) = decode_length_prefixed(data, pos)?;
        pos += n;

        // Field 10: Merkle root
        let (merkle_root, _n) = decode_length_prefixed(data, pos)?;

        Ok(AttestationReport {
            node_addr,
            sequence,
            timestamp,
            schema_version,
            boot_measurements: BootMeasurements {
                firmware_hash: fw_hash,
                anti_rollback_counter,
            },
            kernel_hash,
            puf_health,
            fts_state,
            config_fingerprint,
            merkle_root,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WIRE HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Encode a variable-length field with 2-byte BE length prefix.
fn encode_length_prefixed(out: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as u16;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(data);
}

/// Decode a length-prefixed field. Returns (data, bytes_consumed).
fn decode_length_prefixed(buf: &[u8], pos: usize) -> Result<(Vec<u8>, usize), ReportError> {
    if pos + 2 > buf.len() { return Err(ReportError::TooShort); }
    let len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
    let start = pos + 2;
    if start + len > buf.len() { return Err(ReportError::TooShort); }
    Ok((buf[start..start + len].to_vec(), 2 + len))
}

/// Map FTS NeighborState to wire byte.
fn fts_state_to_wire(state: NeighborState) -> u8 {
    match state {
        NeighborState::Up => 0,
        NeighborState::Suspect => 1,
        NeighborState::Down => 2,
        NeighborState::Recovering => 3,
    }
}

/// Map wire byte to FTS NeighborState.
fn fts_state_from_wire(b: u8) -> Result<NeighborState, ReportError> {
    match b {
        0 => Ok(NeighborState::Up),
        1 => Ok(NeighborState::Suspect),
        2 => Ok(NeighborState::Down),
        3 => Ok(NeighborState::Recovering),
        v => Err(ReportError::InvalidFtsState(v)),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

/// Errors during attestation report parsing or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportError {
    /// Report too short to contain required fields.
    TooShort,
    /// Invalid Rep C address (forgery: zero digit detected).
    InvalidAddress,
    /// Invalid TritInt encoding in named field.
    InvalidTritInt(&'static str),
    /// Invalid PUF health byte.
    InvalidPufHealth(u8),
    /// Invalid FTS state byte.
    InvalidFtsState(u8),
    /// Report exceeds 4KB wire size limit.
    OversizedReport(usize),
    /// Schema version 0 is reserved/invalid.
    InvalidSchemaVersion,
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "attestation report too short"),
            Self::InvalidAddress => write!(f, "invalid Rep C address in attestation report"),
            Self::InvalidTritInt(field) => write!(f, "invalid TritInt in field: {field}"),
            Self::InvalidPufHealth(v) => write!(f, "invalid PUF health byte: {v}"),
            Self::InvalidFtsState(v) => write!(f, "invalid FTS state byte: {v}"),
            Self::OversizedReport(sz) => write!(f, "attestation report exceeds 4KB: {sz} bytes"),
            Self::InvalidSchemaVersion => write!(f, "schema version 0 is invalid"),
        }
    }
}

impl std::error::Error for ReportError {}

// ═══════════════════════════════════════════════════════════════════════
// DISPLAY: dot-separated Rep C for operator-facing surfaces
// ═══════════════════════════════════════════════════════════════════════

impl std::fmt::Display for AttestationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AttestReport[node={}, seq={}, ver={}, puf={:?}, fts={}]",
            self.node_addr.to_rep_c_display(),
            self.sequence,
            self.schema_version,
            self.puf_health,
            self.fts_state,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════
// VALIDATION
// ═══════════════════════════════════════════════════════════════════════

impl AttestationReport {
    /// Validate report constraints before signing or transmission.
    pub fn validate(&self) -> Result<(), ReportError> {
        // Schema version must be ≥ 1
        if self.schema_version.is_zero() {
            return Err(ReportError::InvalidSchemaVersion);
        }
        // Wire size check
        let wire = self.to_wire();
        if wire.len() > 4096 {
            return Err(ReportError::OversizedReport(wire.len()));
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr() -> CubeAddr {
        CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    fn test_report() -> AttestationReport {
        AttestationReport {
            node_addr: test_addr(),
            sequence: TritInt::from_host_u64(42),
            timestamp: TritInt::from_host_u128(1_000_000_000_000_000),
            schema_version: TritInt::from_host_u64(SCHEMA_VERSION as u64),
            boot_measurements: BootMeasurements {
                firmware_hash: vec![0xAB; 48],
                anti_rollback_counter: TritInt::from_host_u64(7),
            },
            kernel_hash: vec![0xCD; 48],
            puf_health: PufHealth::Healthy,
            fts_state: NeighborState::Up,
            config_fingerprint: vec![0xEF; 27],
            merkle_root: vec![0x12; 27],
        }
    }

    #[test]
    fn wire_roundtrip() {
        let report = test_report();
        let wire = report.to_wire();
        assert!(wire.len() < 4096, "report should be under 4KB: {} bytes", wire.len());

        let decoded = AttestationReport::from_wire(&wire).unwrap();
        assert_eq!(decoded.node_addr, report.node_addr);
        assert_eq!(decoded.sequence, report.sequence);
        assert_eq!(decoded.timestamp, report.timestamp);
        assert_eq!(decoded.schema_version, report.schema_version);
        assert_eq!(decoded.kernel_hash, report.kernel_hash);
        assert_eq!(decoded.puf_health, report.puf_health);
        assert_eq!(decoded.fts_state, report.fts_state);
        assert_eq!(decoded.config_fingerprint, report.config_fingerprint);
        assert_eq!(decoded.merkle_root, report.merkle_root);
    }

    #[test]
    fn validates_schema_version() {
        let mut report = test_report();
        report.schema_version = TritInt::zero();
        assert_eq!(report.validate(), Err(ReportError::InvalidSchemaVersion));
    }

    #[test]
    fn display_uses_dotted_repc() {
        let report = test_report();
        let s = format!("{}", report);
        assert!(s.contains("1.1.1.1.1.1.1.1.1.1.1.1.1"), "display should use dot-separated Rep C");
    }

    #[test]
    fn rejects_invalid_address() {
        let mut wire = test_report().to_wire();
        // Corrupt first byte to 0 (forgery)
        wire[0] = 0;
        assert_eq!(
            AttestationReport::from_wire(&wire),
            Err(ReportError::InvalidAddress)
        );
    }

    #[test]
    fn rejects_truncated_report() {
        let wire = test_report().to_wire();
        assert!(AttestationReport::from_wire(&wire[..5]).is_err());
    }
}
