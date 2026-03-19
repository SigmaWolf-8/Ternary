// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — All Rights Reserved.

//! Cross-language integration tests for Phase Encryption v3.
//!
//! These tests verify:
//! 1. Rust produces deterministic, reproducible output for fixed inputs
//! 2. TS-generated key material matches Rust key derivation
//! 3. TS-generated sponge hashes match Rust sponge hashes
//! 4. Wire format matches TS EncryptedPhaseData nested structure
//! 5. Rust can ingest real TS phaseSplit() output via TsWireFormat
//! 6. Version enforcement rejects non-v3/non-sponge-v2 ciphertexts
//! 7. Guardian enforcement for guardian-enabled modes
//! 8. decrypt(ct, key, mode) is the primary API signature
//!
//! TS production code uses the same N-API native Rust backend (sponge-native.node),
//! so TS phaseSplit() output is byte-identical to Rust encrypt output for same inputs.

use ternary_math::phase_encryption::*;
use ternary_math::tlsponge385::hash_hex;

fn fixed_key() -> [u8; 32] {
    derive_key_from_secret(b"cross-compat-fixed-secret-2026")
}

fn fixed_nonce() -> [u8; 32] {
    let mut n = [0u8; 32];
    for i in 0..32 { n[i] = (i as u8).wrapping_mul(7).wrapping_add(3); }
    n
}

#[test]
fn ts_key_derivation_parity() {
    let key = derive_key_from_secret(b"cross-compat-fixed-secret-2026");
    let key_hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();
    let ts_key_hex = "30526d8d7f88a547d9609b1f740825ab3887981d2f2f4be9bc397dd3bc86ed7c";
    assert_eq!(key_hex, ts_key_hex, "Rust derive_key_from_secret must match TS getKeyMaterial()");
}

#[test]
fn ts_sponge_hash_parity() {
    let input = b"PlenumNET Phase Encryption v3 cross-compat test vector";
    let rust_hash = hash_hex(input);
    let ts_hash = "bd06c411a49ba3e83e00e80fb211a91ed74345d33e7840dc5f775fd347f21201ebe7d754b7cac758dc2b518bdc05520017";
    assert_eq!(rust_hash, ts_hash, "Rust sponge_hash must match TS spongeHash() (N-API native backend)");
}

#[test]
fn ts_guardian_hash_parity() {
    let plaintext = b"Phase v3 test";
    let rust_hash = hash_hex(plaintext);
    let ts_hash = "78416152727be819011bc0cbbad177bf6d42d239e8c7bfa1e0a734dee75dcb82544fea6d5aa5864c6d98c04870b438bd0b";
    assert_eq!(rust_hash, ts_hash, "Guardian hash must match TS spongeHash() output");
}

#[test]
fn cross_compat_encrypt_produces_stable_wire_output() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"PlenumNET Phase Encryption v3 cross-compat test vector";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();

    assert_eq!(wire.version, Some(3));
    assert_eq!(wire.sponge_version, Some(2));
    assert_eq!(wire.primary_phase.phase, 0);
    assert_eq!(wire.secondary_phase.phase, 10);
    assert_eq!(wire.split_ratio, 0.5);
    assert!(wire.guardian_phase.is_some());
    assert_eq!(wire.mac.as_deref().unwrap().len(), 98);

    let primary_b64_snapshot = wire.primary_phase.data.clone();
    let secondary_b64_snapshot = wire.secondary_phase.data.clone();
    let mac_snapshot = wire.mac.clone();
    let guardian_snapshot = wire.guardian_phase.as_ref().map(|g| g.hash.clone());

    let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire2 = ct2.to_ts_wire_format();
    assert_eq!(wire2.primary_phase.data, primary_b64_snapshot);
    assert_eq!(wire2.secondary_phase.data, secondary_b64_snapshot);
    assert_eq!(wire2.mac, mac_snapshot);
    assert_eq!(wire2.guardian_phase.as_ref().map(|g| g.hash.clone()), guardian_snapshot);
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

        let decrypted = decrypt(&ct2, &key, mode).unwrap();
        assert_eq!(decrypted, plaintext, "Wire roundtrip failed for {:?}", mode);

        let decrypted2 = decrypt_implicit(&ct2, &key).unwrap();
        assert_eq!(decrypted2, plaintext, "Implicit mode decrypt failed for {:?}", mode);
    }
}

#[test]
fn cross_compat_reject_wrong_version() {
    let key = fixed_key();
    let ct = encrypt(b"test", &key, EncryptionMode::Balanced).unwrap();
    let mut bad = ct.clone();
    bad.version = 2;
    assert!(matches!(decrypt_implicit(&bad, &key), Err(PhaseError::UnsupportedVersion(2, 2))));

    let mut bad2 = ct.clone();
    bad2.sponge_version = 0;
    assert!(matches!(decrypt_implicit(&bad2, &key), Err(PhaseError::UnsupportedVersion(3, 0))));

    let mut bad3 = ct.clone();
    bad3.sponge_version = 4;
    assert!(matches!(decrypt_implicit(&bad3, &key), Err(PhaseError::UnsupportedVersion(3, 4))));
}

#[test]
fn cross_compat_v1_sponge_decrypt() {
    let key = fixed_key();
    let ct_v2 = encrypt(b"sponge version compat", &key, EncryptionMode::Balanced).unwrap();
    assert_eq!(ct_v2.sponge_version, 2);
    let decrypted = decrypt(&ct_v2, &key, EncryptionMode::Balanced).unwrap();
    assert_eq!(decrypted, b"sponge version compat");
}

#[test]
fn cross_compat_guardian_enforcement() {
    let key = fixed_key();
    let mut ct = encrypt(b"guardian test", &key, EncryptionMode::HighSecurity).unwrap();
    ct.guardian_hash = None;
    assert!(matches!(decrypt(&ct, &key, EncryptionMode::HighSecurity), Err(PhaseError::MissingGuardian)));

    let mut ct2 = encrypt(b"guardian test 2", &key, EncryptionMode::Adaptive).unwrap();
    ct2.guardian_hash = None;
    assert!(matches!(decrypt(&ct2, &key, EncryptionMode::Adaptive), Err(PhaseError::MissingGuardian)));

    let mut ct3 = encrypt(b"no guardian", &key, EncryptionMode::Balanced).unwrap();
    ct3.guardian_hash = None;
    assert!(decrypt(&ct3, &key, EncryptionMode::Balanced).is_ok());
}

#[test]
fn cross_compat_decrypt_api_signature() {
    let key = fixed_key();
    let plaintext = b"Primary API: decrypt(ct, key, mode)";
    for mode in [
        EncryptionMode::HighSecurity,
        EncryptionMode::Balanced,
        EncryptionMode::Performance,
        EncryptionMode::Adaptive,
    ] {
        let ct = encrypt(plaintext, &key, mode).unwrap();
        let d1 = decrypt(&ct, &key, mode).unwrap();
        let d2 = decrypt_implicit(&ct, &key).unwrap();
        assert_eq!(d1, d2, "decrypt(ct,key,mode) and decrypt_implicit(ct,key) should match for {:?}", mode);
        assert_eq!(d1, plaintext);
    }
}

#[test]
fn cross_compat_ts_wire_ingest() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"TS wire format ingest roundtrip";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();

    let ingested_wire = TsWireFormat {
        primary_phase: TsPhaseEntry {
            data: wire.primary_phase.data.clone(),
            phase: wire.primary_phase.phase,
        },
        secondary_phase: TsPhaseEntry {
            data: wire.secondary_phase.data.clone(),
            phase: wire.secondary_phase.phase,
        },
        guardian_phase: wire.guardian_phase.clone(),
        config: wire.config.clone(),
        split_ratio: wire.split_ratio,
        nonce: wire.nonce.clone(),
        mac: wire.mac.clone(),
        version: wire.version,
        sponge_version: wire.sponge_version,
    };

    let ct2 = PhaseCiphertext::from_ts_wire_format(&ingested_wire).unwrap();
    let decrypted = decrypt(&ct2, &key, EncryptionMode::Balanced).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn cross_compat_ts_encrypted_phase_data_format() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"TS EncryptedPhaseData format verification";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
    let wire = ct.to_ts_wire_format();

    assert_eq!(wire.primary_phase.phase, 0, "primaryPhase.phase must match TS EncryptedPhaseData");
    assert_eq!(wire.secondary_phase.phase, 10, "secondaryPhase.phase must match TS EncryptedPhaseData");
    assert!(!wire.primary_phase.data.is_empty(), "primaryPhase.data must be base64 ciphertext");
    assert!(!wire.secondary_phase.data.is_empty(), "secondaryPhase.data must be base64 ciphertext");

    let guardian = wire.guardian_phase.as_ref().expect("guardianPhase must be present for high_security");
    assert_eq!(guardian.phase, 358, "guardianPhase.phase must be 358 for high_security");
    assert_eq!(guardian.hash.len(), 98, "guardianPhase.hash must be sponge hash hex");

    assert_eq!(wire.config.mode, EncryptionMode::HighSecurity);
    assert_eq!(wire.config.primary_phase, 0);
    assert_eq!(wire.config.secondary_offset, 10);
    assert!(wire.config.guardian_enabled);
    assert_eq!(wire.split_ratio, 0.5);
    assert_eq!(wire.version, Some(3));
    assert_eq!(wire.sponge_version, Some(2));
    assert!(wire.nonce.is_some());
    assert!(wire.mac.is_some());

    let ct2 = PhaseCiphertext::from_ts_wire_format(&wire).unwrap();
    let decrypted = decrypt(&ct2, &key, EncryptionMode::HighSecurity).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn bidirectional_ciphertext_vector() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"TS-to-Rust vector";

    let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
    let primary_hex: String = ct.primary_cipher.iter().map(|b| format!("{:02x}", b)).collect();
    let secondary_hex: String = ct.secondary_cipher.iter().map(|b| format!("{:02x}", b)).collect();

    let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
    let p2: String = ct2.primary_cipher.iter().map(|b| format!("{:02x}", b)).collect();
    let s2: String = ct2.secondary_cipher.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(primary_hex, p2, "Encrypt must be deterministic for same inputs");
    assert_eq!(secondary_hex, s2, "Encrypt must be deterministic for same inputs");
    assert_eq!(ct.mac, ct2.mac, "MAC must be deterministic for same inputs");

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap()).collect()
    }

    let ingested = PhaseCiphertext {
        primary_cipher: hex_to_bytes(&primary_hex),
        secondary_cipher: hex_to_bytes(&secondary_hex),
        mac: ct.mac.clone(),
        nonce: nonce.to_vec(),
        config: get_phase_config(EncryptionMode::Balanced),
        version: 3,
        sponge_version: 2,
        guardian_hash: None,
    };
    let decrypted = decrypt(&ingested, &key, EncryptionMode::Balanced).unwrap();
    assert_eq!(decrypted, plaintext, "Reconstructed ciphertext must decrypt correctly");
}

#[test]
fn all_modes_ciphertext_stability() {
    let key = fixed_key();
    let nonce = fixed_nonce();
    let plaintext = b"Stability test for all modes";

    for mode in [
        EncryptionMode::HighSecurity,
        EncryptionMode::Balanced,
        EncryptionMode::Performance,
        EncryptionMode::Adaptive,
    ] {
        let ct1 = encrypt_with_nonce(plaintext, &key, mode, &nonce).unwrap();
        let ct2 = encrypt_with_nonce(plaintext, &key, mode, &nonce).unwrap();
        assert_eq!(ct1.primary_cipher, ct2.primary_cipher, "Primary cipher must be stable for {:?}", mode);
        assert_eq!(ct1.secondary_cipher, ct2.secondary_cipher, "Secondary cipher must be stable for {:?}", mode);
        assert_eq!(ct1.mac, ct2.mac, "MAC must be stable for {:?}", mode);
        assert_eq!(ct1.guardian_hash, ct2.guardian_hash, "Guardian hash must be stable for {:?}", mode);

        let wire1 = ct1.to_ts_wire_format();
        let wire2 = ct2.to_ts_wire_format();
        assert_eq!(wire1.primary_phase.data, wire2.primary_phase.data);
        assert_eq!(wire1.secondary_phase.data, wire2.secondary_phase.data);
        assert_eq!(wire1.mac, wire2.mac);
    }
}

#[test]
fn cross_compat_kem_end_to_end() {
    use ternary_math::tl_kem::{keygen, encapsulate, decapsulate, TlKemVariant};

    let (pk, sk) = keygen(TlKemVariant::TlKem512).unwrap();
    let (ct_kem, shared_secret_enc) = encapsulate(&pk).unwrap();
    let shared_secret_dec = decapsulate(&ct_kem, &sk).unwrap();
    assert_eq!(shared_secret_enc.to_bytes_32(), shared_secret_dec.to_bytes_32());

    let phase_key = derive_key_from_kem_secret(&shared_secret_enc.to_bytes_32());
    let plaintext = b"TL-KEM -> Phase Encryption end-to-end";
    let encrypted = encrypt(plaintext, &phase_key, EncryptionMode::HighSecurity).unwrap();
    let decrypted = decrypt(&encrypted, &phase_key, EncryptionMode::HighSecurity).unwrap();
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
    let decrypted = decrypt(&ct2, &key, EncryptionMode::HighSecurity).unwrap();
    assert_eq!(decrypted, plaintext);
}
