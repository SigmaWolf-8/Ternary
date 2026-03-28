// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use ternary_math::tl_dsa::{self, TlDsaVariant, TlDsaKeyPair};

pub const VARIANT: TlDsaVariant = TlDsaVariant::TlDsa87;

pub fn generate_keypair(seed: &[u8]) -> TlDsaKeyPair {
    tl_dsa::keygen(VARIANT, Some(seed))
}

pub fn sign(secret_key: &[u8], payload: &[u8]) -> Vec<u8> {
    tl_dsa::sign(secret_key, payload, VARIANT)
}

pub fn verify(public_key: &[u8], payload: &[u8], signature: &[u8]) -> bool {
    tl_dsa::verify(public_key, payload, signature, VARIANT)
}

pub fn export_pubkey_b64(public_key: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(public_key)
}

pub fn fingerprint(public_key: &[u8]) -> String {
    let hash = ternary_math::sponge::derive_key(b"NinjaExec-FP", public_key, 16);
    hash.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(":")
}

#[allow(dead_code)]
pub fn pk_len() -> usize {
    tl_dsa::pk_len(VARIANT)
}

#[allow(dead_code)]
pub fn sk_len() -> usize {
    tl_dsa::sk_len(VARIANT)
}

#[allow(dead_code)]
pub fn sig_len() -> usize {
    tl_dsa::sig_len(VARIANT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_sign_verify() {
        let kp = generate_keypair(b"ninja-exec-test-seed-roundtrip");
        let msg = b"test payload for NinjaExec signing";
        let sig = sign(&kp.secret_key, msg);
        assert!(verify(&kp.public_key, msg, &sig));
    }

    #[test]
    fn test_tampered_payload_rejected() {
        let kp = generate_keypair(b"ninja-exec-test-seed-tamper");
        let msg = b"original payload";
        let sig = sign(&kp.secret_key, msg);
        assert!(!verify(&kp.public_key, b"tampered payload", &sig));
    }

    #[test]
    fn test_export_pubkey_b64() {
        let kp = generate_keypair(b"ninja-exec-test-seed-export");
        let b64 = export_pubkey_b64(&kp.public_key);
        assert!(!b64.is_empty());
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        assert_eq!(decoded, kp.public_key);
    }

    #[test]
    fn test_fingerprint_deterministic() {
        let kp = generate_keypair(b"ninja-exec-test-seed-fp");
        let fp1 = fingerprint(&kp.public_key);
        let fp2 = fingerprint(&kp.public_key);
        assert_eq!(fp1, fp2);
        assert!(fp1.contains(':'));
    }

    #[test]
    fn test_key_sizes() {
        assert_eq!(pk_len(), 64);
        assert_eq!(sk_len(), 128);
        assert_eq!(sig_len(), 3168);
    }
}
