// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Attestation Signing — TL-DSA key derivation and report signing
//!
//! The attestation signing key is derived from the PUF root secret via
//! TLSponge-385 with domain separation. The PUF root is used ONLY for
//! this one-time derivation, not for direct signing.
//!
//! Key lifecycle:
//! - Derived once on service start from PUF root secret
//! - Held in a Zeroize-implementing struct for the service lifetime
//! - Zeroized on service shutdown or key rotation
//! - Re-derived from PUF on service restart
//!
//! Context strings are registered in the canonical context string registry.

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::cube_addr::CubeAddr;
use super::report::{AttestationReport, SignedAttestationReport};

// ═══════════════════════════════════════════════════════════════════════
// CONTEXT STRINGS — registered in canonical context string registry
// ═══════════════════════════════════════════════════════════════════════

/// TL-DSA signing context: "PLENUMNET-ATTEST-v1.0" ‖ signer_rep_c_addr.
/// Verifiers reconstruct this from the sender's known Rep C address.
pub const SIGNING_CONTEXT_PREFIX: &[u8] = b"PLENUMNET-ATTEST-v1.0";

/// Key derivation context: domain separation from other PUF-derived keys.
/// e.g. CON tunnel keys use "PlenumNET-CON-v3.0".
pub const KEY_DERIVATION_CONTEXT: &[u8] = b"PLENUMNET-ATTEST-KEY-v1.0";

/// Attestation signing key length (bytes). Matches TLSponge-385 security level.
const SIGNING_KEY_LEN: usize = 48;

// ═══════════════════════════════════════════════════════════════════════
// ATTESTATION SIGNING KEY — Zeroize on drop
// ═══════════════════════════════════════════════════════════════════════

/// Holds the derived attestation signing key material.
/// Implements Zeroize + ZeroizeOnDrop — memory is zeroed when the
/// struct goes out of scope or is explicitly zeroized on shutdown.
///
/// The key is derived once from the PUF root secret and held for
/// the service lifetime. NOT zeroed per-operation.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AttestationSigningKey {
    /// The derived signing key material.
    key_material: Vec<u8>,
}

impl AttestationSigningKey {
    /// Public accessor for verification — returns a reference to the raw key bytes.
    /// Tests and verifiers use this to feed into `verify()`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_material
    }
}

impl AttestationSigningKey {
    /// Derive the attestation signing key from PUF root secret.
    ///
    /// `key = TLSponge-385(context: KEY_DERIVATION_CONTEXT, input: puf_root ‖ node_addr)`
    ///
    /// This is called once on service start. The PUF root secret is
    /// NOT retained — only the derived subkey is held.
    pub fn derive(puf_root_secret: &[u8], node_addr: &CubeAddr) -> Self {
        let addr_bytes = node_addr.to_bytes();
        let mut material = Vec::with_capacity(puf_root_secret.len() + addr_bytes.len());
        material.extend_from_slice(puf_root_secret);
        material.extend_from_slice(&addr_bytes);

        let key_material = ternary_math::sponge::derive_key(
            KEY_DERIVATION_CONTEXT,
            &material,
            SIGNING_KEY_LEN,
        );

        // Zeroize the intermediate material
        material.zeroize();

        AttestationSigningKey { key_material }
    }

    /// Build the full signing context for a specific node address.
    ///
    /// Context = "PLENUMNET-ATTEST-v1.0" ‖ node_rep_c_addr_bytes
    ///
    /// Both signer and verifier construct the same context from the
    /// node's known Rep C address.
    pub fn signing_context(node_addr: &CubeAddr) -> Vec<u8> {
        let addr_bytes = node_addr.to_bytes();
        let mut ctx = Vec::with_capacity(SIGNING_CONTEXT_PREFIX.len() + addr_bytes.len());
        ctx.extend_from_slice(SIGNING_CONTEXT_PREFIX);
        ctx.extend_from_slice(&addr_bytes);
        ctx
    }

    /// Sign an attestation report using TL-DSA.
    ///
    /// The report is serialized to Rep C wire format, then signed with
    /// the derived attestation subkey and the node-specific context string.
    ///
    /// Returns None if signing fails (PUF unavailable, key corruption).
    pub fn sign_report(&self, report: &AttestationReport) -> Option<SignedAttestationReport> {
        let wire_payload = report.to_wire();
        let context = Self::signing_context(&report.node_addr);

        // TL-DSA sign: key_material + context + payload → signature
        // In production, this calls through the Rust kernel bridge.
        // For now, use TLSponge-385 keyed hash as the signing primitive
        // (the real TL-DSA lattice signature replaces this when integrated).
        let signature = ternary_math::sponge::derive_key(
            &context,
            &[self.key_material.as_slice(), wire_payload.as_slice()].concat(),
            // TL-DSA-87 signature size — placeholder until real lattice sig
            64,
        );

        Some(SignedAttestationReport {
            report: report.clone(),
            signature,
        })
    }

    /// Verify a signed attestation report.
    ///
    /// Reconstructs the expected context from the report's node address,
    /// recomputes the signature, and compares in constant time.
    ///
    /// `verifier_key` is the attestation public key fetched from
    /// PlenumConfig for the attesting node's Rep C address.
    pub fn verify_report(
        verifier_key: &[u8],
        signed_report: &SignedAttestationReport,
    ) -> bool {
        let wire_payload = signed_report.report.to_wire();
        let context = Self::signing_context(&signed_report.report.node_addr);

        // Recompute expected signature
        let expected = ternary_math::sponge::derive_key(
            &context,
            &[verifier_key, wire_payload.as_slice()].concat(),
            64,
        );

        // Constant-time comparison
        if expected.len() != signed_report.signature.len() {
            return false;
        }
        let mut diff: u8 = 0;
        for (&a, &b) in expected.iter().zip(signed_report.signature.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }
}

impl std::fmt::Debug for AttestationSigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose key material in debug output
        write!(f, "AttestationSigningKey([REDACTED, {} bytes])", self.key_material.len())
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
    use ternary_math::trit_int::TritInt;

    fn test_addr() -> CubeAddr {
        CubeAddr::new([2, 1, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1])
    }

    fn test_report(addr: &CubeAddr) -> AttestationReport {
        AttestationReport {
            node_addr: addr.clone(),
            sequence: TritInt::from_u64(1),
            timestamp: TritInt::from_u128(1_000_000_000_000_000),
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
        }
    }

    #[test]
    fn key_derivation_deterministic() {
        let addr = test_addr();
        let secret = vec![0x42u8; 32];
        let key1 = AttestationSigningKey::derive(&secret, &addr);
        let key2 = AttestationSigningKey::derive(&secret, &addr);
        // Same input → same key
        assert_eq!(key1.key_material, key2.key_material);
    }

    #[test]
    fn key_derivation_domain_separated() {
        let addr = test_addr();
        let secret = vec![0x42u8; 32];
        let attest_key = AttestationSigningKey::derive(&secret, &addr);

        // Derive a key with a different context (simulating CON key derivation)
        let con_key = ternary_math::sponge::derive_key(
            b"PlenumNET-CON-v3.0",
            &[&secret[..], &addr.to_bytes()].concat(),
            48,
        );

        // Keys MUST be different — domain separation
        assert_ne!(attest_key.key_material, con_key);
    }

    #[test]
    fn different_addresses_different_keys() {
        let addr1 = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let addr2 = CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = vec![0x42u8; 32];
        let key1 = AttestationSigningKey::derive(&secret, &addr1);
        let key2 = AttestationSigningKey::derive(&secret, &addr2);
        assert_ne!(key1.key_material, key2.key_material);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let addr = test_addr();
        let secret = vec![0x42u8; 32];
        let key = AttestationSigningKey::derive(&secret, &addr);
        let report = test_report(&addr);

        let signed = key.sign_report(&report).unwrap();
        assert!(!signed.signature.is_empty());

        // Verify with same key
        assert!(AttestationSigningKey::verify_report(&key.key_material, &signed));
    }

    #[test]
    fn wrong_key_rejects() {
        let addr = test_addr();
        let key = AttestationSigningKey::derive(&vec![0x42u8; 32], &addr);
        let wrong_key = AttestationSigningKey::derive(&vec![0x99u8; 32], &addr);
        let report = test_report(&addr);

        let signed = key.sign_report(&report).unwrap();
        assert!(!AttestationSigningKey::verify_report(&wrong_key.key_material, &signed));
    }

    #[test]
    fn signing_context_includes_address() {
        let addr = test_addr();
        let ctx = AttestationSigningKey::signing_context(&addr);
        assert!(ctx.starts_with(SIGNING_CONTEXT_PREFIX));
        assert_eq!(ctx.len(), SIGNING_CONTEXT_PREFIX.len() + 13);
    }

    #[test]
    fn debug_redacts_key_material() {
        let addr = test_addr();
        let key = AttestationSigningKey::derive(&vec![0x42u8; 32], &addr);
        let debug = format!("{:?}", key);
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("42")); // no raw key bytes
    }
}
