// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Authenticated Deregistration (T-21, SPEC-2026-NEXT)
//!
//! Prevents unauthorized removal of registrations. A cube can only be
//! deregistered by the holder of the signing key — the same key that
//! signed the original registration (T-06).
//!
//! ## Signature Construction
//!
//! ```text
//! message = "PlenumNET-CRS-DEREG-v1" ‖ address_wire ‖ timestamp_le ‖ reason_bytes
//! signature = TL-DSA-87::sign(secret_key, message)
//! ```
//!
//! The domain separator `"PlenumNET-CRS-DEREG-v1"` prevents cross-protocol
//! signature reuse — a registration signature cannot be replayed as a
//! deregistration and vice versa.
//!
//! ## Timestamp Policy
//!
//! Same as T-06 registration:
//! - `u128` femtoseconds since Salvi Epoch
//! - Replay window: 30 seconds
//! - Future tolerance: 1 second
//! - Must be strictly newer than the registration timestamp
//!
//! ## Deregistration Reasons
//!
//! The `reason` field is included in the signed message, preventing
//! an attacker from changing the reason after capture. Reasons:
//!
//! - `Graceful`: Node is shutting down cleanly
//! - `Maintenance`: Temporary removal, will re-register
//! - `KeyCompromise`: Emergency — revoke all dependent keys
//! - `Migration`: Moving to a different address
//!
//! ## Integration
//!
//! CRS calls `verify_deregistration()` before removing the record.
//! On success, the address enters the grace period (T-01) before reuse.

use crate::cube_addr::CubeAddr;
use crate::wire::{
    pack_addr, WIRE_ADDR_SIZE,
    REGISTRATION_MAX_AGE_FS, TIMESTAMP_FUTURE_TOLERANCE_FS,
};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Domain separator for deregistration signatures.
pub const CRS_DEREG_DOMAIN: &[u8] = b"PlenumNET-CRS-DEREG-v1";

/// TL-DSA variant for deregistration signatures (matches T-06).
pub const DEREG_SIG_VARIANT: u8 = 87;

// ═══════════════════════════════════════════════════════════════════════
// DEREGISTRATION REASON
// ═══════════════════════════════════════════════════════════════════════

/// Why a node is deregistering.
///
/// Included in the signed message to prevent reason tampering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeregReason {
    /// Clean shutdown — node is going offline.
    Graceful = 0x01,
    /// Temporary removal — will re-register soon.
    Maintenance = 0x02,
    /// Emergency key revocation — all dependent keys compromised.
    KeyCompromise = 0x03,
    /// Moving to a different address (re-register at new location).
    Migration = 0x04,
}

impl DeregReason {
    /// Parse from u8.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Graceful),
            0x02 => Some(Self::Maintenance),
            0x03 => Some(Self::KeyCompromise),
            0x04 => Some(Self::Migration),
            _ => None,
        }
    }

    /// Encode to bytes for inclusion in signed message.
    pub fn as_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }
}

impl std::fmt::Display for DeregReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graceful => write!(f, "graceful"),
            Self::Maintenance => write!(f, "maintenance"),
            Self::KeyCompromise => write!(f, "key_compromise"),
            Self::Migration => write!(f, "migration"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

/// Deregistration errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeregError {
    /// TL-DSA signature verification failed.
    InvalidSignature,
    /// Timestamp outside acceptable window.
    StaleTimestamp,
    /// Timestamp not newer than the registration.
    ReplayDetected,
    /// Address not found in the registry.
    AddressNotFound,
    /// The signing key doesn't match the registration's public key.
    KeyMismatch,
    /// Invalid deregistration reason code.
    InvalidReason(u8),
}

impl std::fmt::Display for DeregError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "deregistration signature invalid"),
            Self::StaleTimestamp => write!(f, "deregistration timestamp outside window"),
            Self::ReplayDetected => write!(f, "deregistration timestamp not newer than registration"),
            Self::AddressNotFound => write!(f, "address not found in registry"),
            Self::KeyMismatch => write!(f, "signing key doesn't match registration"),
            Self::InvalidReason(r) => write!(f, "invalid deregistration reason: 0x{:02X}", r),
        }
    }
}

impl std::error::Error for DeregError {}

// ═══════════════════════════════════════════════════════════════════════
// SIGNED DEREGISTRATION PAYLOAD
// ═══════════════════════════════════════════════════════════════════════

/// A signed deregistration request.
#[derive(Debug, Clone)]
pub struct SignedDeregistration {
    /// Address to deregister.
    pub address: CubeAddr,
    /// Public key of the deregistering node (must match registration).
    pub public_key: Vec<u8>,
    /// Femtosecond timestamp since Salvi Epoch.
    pub timestamp_fs: u128,
    /// Why the node is deregistering.
    pub reason: DeregReason,
    /// TL-DSA-87 signature over the canonical message.
    pub signature: Vec<u8>,
}

impl SignedDeregistration {
    /// Construct the canonical message that was signed.
    ///
    /// Format: `domain ‖ address_wire ‖ timestamp_le ‖ reason`
    pub fn canonical_message(&self) -> Vec<u8> {
        let addr_wire = pack_addr(&self.address).unwrap_or([0u8; WIRE_ADDR_SIZE]);
        let ts_bytes = self.timestamp_fs.to_le_bytes();
        let reason_bytes = self.reason.as_bytes();

        let mut msg = Vec::with_capacity(
            CRS_DEREG_DOMAIN.len() + WIRE_ADDR_SIZE + 16 + 1,
        );
        msg.extend_from_slice(CRS_DEREG_DOMAIN);
        msg.extend_from_slice(&addr_wire);
        msg.extend_from_slice(&ts_bytes);
        msg.extend_from_slice(&reason_bytes);
        msg
    }
}

/// Construct a deregistration message for signing.
///
/// Public helper so the deregistering node can build the canonical
/// message before signing it.
pub fn build_deregistration_message(
    addr: &CubeAddr,
    timestamp_fs: u128,
    reason: DeregReason,
) -> Vec<u8> {
    let addr_wire = pack_addr(addr).unwrap_or([0u8; WIRE_ADDR_SIZE]);
    let ts_bytes = timestamp_fs.to_le_bytes();
    let reason_bytes = reason.as_bytes();

    let mut msg = Vec::with_capacity(
        CRS_DEREG_DOMAIN.len() + WIRE_ADDR_SIZE + 16 + 1,
    );
    msg.extend_from_slice(CRS_DEREG_DOMAIN);
    msg.extend_from_slice(&addr_wire);
    msg.extend_from_slice(&ts_bytes);
    msg.extend_from_slice(&reason_bytes);
    msg
}

// ═══════════════════════════════════════════════════════════════════════
// VERIFICATION
// ═══════════════════════════════════════════════════════════════════════

/// Verify a signed deregistration request.
///
/// Checks:
/// 1. Timestamp is within the 30s window (not stale, not future)
/// 2. Timestamp is strictly newer than the registration's timestamp
/// 3. Public key matches the registration's public key
/// 4. TL-DSA-87 signature is valid over the canonical message
///
/// Returns `Ok(reason)` on success for the caller to decide how to
/// handle the deregistration (e.g., KeyCompromise triggers alerts).
pub fn verify_deregistration(
    dereg: &SignedDeregistration,
    registration_pk: &[u8],
    registration_ts: u128,
    now_fs: u128,
) -> Result<DeregReason, DeregError> {
    // 1. Timestamp window check
    if dereg.timestamp_fs > now_fs + TIMESTAMP_FUTURE_TOLERANCE_FS {
        return Err(DeregError::StaleTimestamp);
    }
    if now_fs > dereg.timestamp_fs
        && (now_fs - dereg.timestamp_fs) > REGISTRATION_MAX_AGE_FS
    {
        return Err(DeregError::StaleTimestamp);
    }

    // 2. Must be newer than the registration
    if dereg.timestamp_fs <= registration_ts {
        return Err(DeregError::ReplayDetected);
    }

    // 3. Public key must match the registration
    if dereg.public_key != registration_pk {
        return Err(DeregError::KeyMismatch);
    }

    // 4. Verify TL-DSA-87 signature
    let canonical_msg = dereg.canonical_message();
    let variant = ternary_math::tl_dsa::TlDsaVariant::from_u32(DEREG_SIG_VARIANT as u32)
        .ok_or(DeregError::InvalidSignature)?;

    let valid = ternary_math::tl_dsa::verify(
        &dereg.public_key,
        &canonical_msg,
        &dereg.signature,
        variant,
    );

    if !valid {
        return Err(DeregError::InvalidSignature);
    }

    Ok(dereg.reason)
}

/// Result of a successful deregistration.
#[derive(Debug, Clone)]
pub struct DeregResult {
    /// The address that was deregistered.
    pub address: CubeAddr,
    /// The reason given.
    pub reason: DeregReason,
    /// Whether this was a key compromise (triggers additional alerts).
    pub is_key_compromise: bool,
    /// Timestamp of the deregistration.
    pub timestamp_fs: u128,
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

    fn test_addr() -> CubeAddr {
        addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2])
    }

    /// Helper: generate a keypair and signed deregistration.
    fn make_signed_dereg(
        address: &CubeAddr,
        reason: DeregReason,
        reg_ts: u128,
        dereg_ts: u128,
    ) -> (Vec<u8>, SignedDeregistration) {
        let variant = ternary_math::tl_dsa::TlDsaVariant::TlDsa87;
        let kp = ternary_math::tl_dsa::keygen(variant, Some(b"test-dereg-key"));

        let msg = build_deregistration_message(address, dereg_ts, reason);
        let sig = ternary_math::tl_dsa::sign(&kp.secret_key, &msg, variant);

        let dereg = SignedDeregistration {
            address: address.clone(),
            public_key: kp.public_key.clone(),
            timestamp_fs: dereg_ts,
            reason,
            signature: sig,
        };

        (kp.public_key, dereg)
    }

    // ── Reason parsing ──────────────────────────────────────────

    #[test]
    fn test_reason_from_u8() {
        assert_eq!(DeregReason::from_u8(0x01), Some(DeregReason::Graceful));
        assert_eq!(DeregReason::from_u8(0x02), Some(DeregReason::Maintenance));
        assert_eq!(DeregReason::from_u8(0x03), Some(DeregReason::KeyCompromise));
        assert_eq!(DeregReason::from_u8(0x04), Some(DeregReason::Migration));
        assert_eq!(DeregReason::from_u8(0xFF), None);
    }

    #[test]
    fn test_reason_display() {
        assert_eq!(format!("{}", DeregReason::Graceful), "graceful");
        assert_eq!(format!("{}", DeregReason::KeyCompromise), "key_compromise");
    }

    // ── Canonical message ───────────────────────────────────────

    #[test]
    fn test_canonical_message_deterministic() {
        let ts = 100 * crate::wire::FS_PER_SECOND;
        let m1 = build_deregistration_message(&test_addr(), ts, DeregReason::Graceful);
        let m2 = build_deregistration_message(&test_addr(), ts, DeregReason::Graceful);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_canonical_message_different_reasons() {
        let ts = 100 * crate::wire::FS_PER_SECOND;
        let m1 = build_deregistration_message(&test_addr(), ts, DeregReason::Graceful);
        let m2 = build_deregistration_message(&test_addr(), ts, DeregReason::KeyCompromise);
        assert_ne!(m1, m2, "Different reasons must produce different messages");
    }

    #[test]
    fn test_canonical_message_differs_from_registration() {
        let ts = 100 * crate::wire::FS_PER_SECOND;
        let dereg_msg = build_deregistration_message(&test_addr(), ts, DeregReason::Graceful);
        let endpoint: std::net::SocketAddr = "10.0.0.1:51820".parse().unwrap();
        let pk = vec![0u8; 64];
        let reg_msg = crate::crs::build_registration_message(
            &test_addr(), &endpoint, &pk, None, ts,
        );
        assert_ne!(dereg_msg, reg_msg,
            "Dereg and reg messages must differ (different domain separators)");
    }

    // ── Verification: valid ─────────────────────────────────────

    #[test]
    fn test_verify_valid_deregistration() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 110 * fs;

        let (pk, dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        let result = verify_deregistration(&dereg, &pk, reg_ts, now_fs);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DeregReason::Graceful);
    }

    #[test]
    fn test_verify_key_compromise() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 110 * fs;

        let (pk, dereg) = make_signed_dereg(&test_addr(), DeregReason::KeyCompromise, reg_ts, dereg_ts);
        let result = verify_deregistration(&dereg, &pk, reg_ts, now_fs);

        assert_eq!(result.unwrap(), DeregReason::KeyCompromise);
    }

    // ── Verification: failures ──────────────────────────────────

    #[test]
    fn test_verify_stale_timestamp() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 200 * fs; // 90s later — beyond 30s window

        let (pk, dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        let err = verify_deregistration(&dereg, &pk, reg_ts, now_fs).unwrap_err();
        assert_eq!(err, DeregError::StaleTimestamp);
    }

    #[test]
    fn test_verify_replay_timestamp() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 110 * fs;
        let dereg_ts = 100 * fs; // OLDER than registration
        let now_fs = 110 * fs;

        let (pk, dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        let err = verify_deregistration(&dereg, &pk, reg_ts, now_fs).unwrap_err();
        assert_eq!(err, DeregError::ReplayDetected);
    }

    #[test]
    fn test_verify_key_mismatch() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 110 * fs;

        let (_, dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        let wrong_pk = vec![0xFFu8; 64]; // Different key
        let err = verify_deregistration(&dereg, &wrong_pk, reg_ts, now_fs).unwrap_err();
        assert_eq!(err, DeregError::KeyMismatch);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 110 * fs;

        let (pk, mut dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        // Corrupt the signature
        if let Some(b) = dereg.signature.get_mut(10) {
            *b ^= 0xFF;
        }
        let err = verify_deregistration(&dereg, &pk, reg_ts, now_fs).unwrap_err();
        assert_eq!(err, DeregError::InvalidSignature);
    }

    #[test]
    fn test_verify_wrong_reason_in_signature() {
        let fs = crate::wire::FS_PER_SECOND;
        let reg_ts = 100 * fs;
        let dereg_ts = 110 * fs;
        let now_fs = 110 * fs;

        // Sign with Graceful but claim KeyCompromise
        let (pk, mut dereg) = make_signed_dereg(&test_addr(), DeregReason::Graceful, reg_ts, dereg_ts);
        dereg.reason = DeregReason::KeyCompromise; // Tamper with reason
        let err = verify_deregistration(&dereg, &pk, reg_ts, now_fs).unwrap_err();
        assert_eq!(err, DeregError::InvalidSignature,
            "Changing reason after signing must invalidate the signature");
    }

    // ── DeregResult ─────────────────────────────────────────────

    #[test]
    fn test_dereg_result_key_compromise_flag() {
        let result = DeregResult {
            address: test_addr(),
            reason: DeregReason::KeyCompromise,
            is_key_compromise: true,
            timestamp_fs: 100,
        };
        assert!(result.is_key_compromise);
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_domain_differs_from_registration() {
        assert_ne!(
            CRS_DEREG_DOMAIN,
            crate::crs::CRS_REG_DOMAIN,
            "Dereg domain must differ from reg domain"
        );
    }
}