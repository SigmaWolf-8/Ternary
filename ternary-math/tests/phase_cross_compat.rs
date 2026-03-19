// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — All Rights Reserved.

//! Cross-language integration tests for Phase Encryption v3.
//!
//! These tests verify that the Rust Phase Encryption module produces
//! byte-identical output to the TypeScript implementation for fixed
//! inputs (key, nonce, plaintext, mode). They also verify that Rust
//! can decrypt ciphertext produced by the TS wire format.

use ternary_math::phase_encryption::*;

fn fixed_key() -> [u8; 32] {
    derive_key_from_secret(b"cross-compat-fixed-secret-2026")
}

fn fixed_nonce() -> [u8; 32] {
    let mut n = [0u8; 32];
    for i in 0..32 { n[i] = (i as u8).wrapping_mul(7).wrapping_add(3); }
    n
}

#[test]
fn cross_compat_encrypt_produces_stable_wire_output() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"PlenumNET Phase Encryption v3 cross-compat test vector";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();

    assert_eq!(wire.version, 3);
    assert_eq!(wire.sponge_version, 2);
    assert_eq!(wire.primary_phase, 0);
    assert_eq!(wire.secondary_phase, 10);
    assert_eq!(wire.split_ratio, 0.5);
    assert!(wire.guardian_hash.is_some());
    assert_eq!(wire.mac.len(), 98);

    let primary_b64_snapshot = wire.primary_data_b64.clone();
    let secondary_b64_snapshot = wire.secondary_data_b64.clone();
    let mac_snapshot = wire.mac.clone();
    let guardian_snapshot = wire.guardian_hash.clone();

    let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire2 = ct2.to_ts_wire_format();
    assert_eq!(wire2.primary_data_b64, primary_b64_snapshot);
    assert_eq!(wire2.secondary_data_b64, secondary_b64_snapshot);
    assert_eq!(wire2.mac, mac_snapshot);
    assert_eq!(wire2.guardian_hash, guardian_snapshot);
}

#[test]
fn cross_compat_wire_format_roundtrip_all_modes() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"Roundtrip verification across modes and wire serialization";

    for mode in [
        EncryptionMode::HighSecurity,
        EncryptionMode::Balanced,
        EncryptionMode::Performance,
        EncryptionMode::Adaptive,
    ] {
        let ct = encrypt_with_nonce(plaintext, &key, mode, &nonce).unwrap();

        let wire = ct.to_ts_wire_format();
        let ct2 = PhaseCiphertext::from_ts_wire_format(&wire).unwrap();

        let decrypted = decrypt_with_mode(&ct2, &key, mode).unwrap();
        assert_eq!(decrypted, plaintext, "Wire roundtrip failed for {:?}", mode);

        let decrypted2 = decrypt(&ct2, &key).unwrap();
        assert_eq!(decrypted2, plaintext, "Implicit mode decrypt failed for {:?}", mode);
    }
}

#[test]
fn cross_compat_reject_wrong_version() {
    let key = fixed_key();
    let ct = encrypt(b"test", &key, EncryptionMode::Balanced).unwrap();
    let mut bad = ct.clone();
    bad.version = 2;
    assert!(matches!(decrypt(&bad, &key), Err(PhaseError::InvalidCiphertext)));

    let mut bad2 = ct.clone();
    bad2.sponge_version = 1;
    assert!(matches!(decrypt(&bad2, &key), Err(PhaseError::InvalidCiphertext)));
}

#[test]
fn cross_compat_decrypt_with_mode_matches_embedded() {
    let key = fixed_key();
    let plaintext = b"Mode override test: decrypt_with_mode matches decrypt";
    for mode in [
        EncryptionMode::HighSecurity,
        EncryptionMode::Balanced,
        EncryptionMode::Performance,
        EncryptionMode::Adaptive,
    ] {
        let ct = encrypt(plaintext, &key, mode).unwrap();
        let d1 = decrypt(&ct, &key).unwrap();
        let d2 = decrypt_with_mode(&ct, &key, mode).unwrap();
        assert_eq!(d1, d2, "decrypt and decrypt_with_mode should match for {:?}", mode);
    }
}

#[test]
fn cross_compat_ts_wire_ingest_simulated() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"Simulated TS wire format ingest";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();

    let ingested_wire = TsWireFormat {
        primary_data_b64: wire.primary_data_b64.clone(),
        primary_phase: wire.primary_phase,
        secondary_data_b64: wire.secondary_data_b64.clone(),
        secondary_phase: wire.secondary_phase,
        config: wire.config.clone(),
        split_ratio: wire.split_ratio,
        nonce_hex: wire.nonce_hex.clone(),
        mac: wire.mac.clone(),
        version: wire.version,
        sponge_version: wire.sponge_version,
        guardian_hash: wire.guardian_hash.clone(),
        guardian_phase: wire.guardian_phase,
    };

    let ct2 = PhaseCiphertext::from_ts_wire_format(&ingested_wire).unwrap();
    let decrypted = decrypt(&ct2, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn cross_compat_kem_key_wire_roundtrip() {
    let kem_secret = [0x55u8; 32];
    let key = derive_key_from_kem_secret(&kem_secret);
    let nonce = fixed_nonce();
    let plaintext = b"KEM-derived key with wire format serialization";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();
    let ct2 = PhaseCiphertext::from_ts_wire_format(&wire).unwrap();
    let decrypted = decrypt(&ct2, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}
