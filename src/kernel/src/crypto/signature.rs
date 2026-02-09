//! Post-Quantum Signature Stubs
//!
//! Implements a proper Lamport-style one-time signature (OTS) scheme for
//! ternary data. For each trit position in the message digest, three secret
//! values are generated (one per trit value: -1, 0, +1). The public key
//! contains hashes of all secrets. Signing reveals only the secret
//! corresponding to each message-digest trit.
//!
//! **WARNING**: This is a one-time signature scheme. Each key pair MUST only
//! be used to sign a single message. Reusing a key pair compromises security.
//!
//! Production implementations will integrate lattice-based schemes
//! (CRYSTALS-Dilithium, FALCON) when available.

use alloc::string::String;
use alloc::vec::Vec;
use super::{CryptoError, CryptoResult, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;

const SIGN_DIGEST_TRITS: usize = 81;
const SECRET_ELEMENT_TRITS: usize = TERNARY_HASH_TRITS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScheme {
    TernaryLamport,
    Dilithium,
    Falcon,
    SphincsPlus,
}

impl SignatureScheme {
    pub fn is_available(&self) -> bool {
        matches!(self, SignatureScheme::TernaryLamport)
    }

    pub fn security_level(&self) -> u32 {
        match self {
            SignatureScheme::TernaryLamport => 128,
            SignatureScheme::Dilithium => 256,
            SignatureScheme::Falcon => 256,
            SignatureScheme::SphincsPlus => 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SigningKey {
    pub scheme: SignatureScheme,
    pub secrets: Vec<Vec<i8>>,
    pub used: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyingKey {
    pub scheme: SignatureScheme,
    pub public: Vec<Vec<i8>>,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub scheme: SignatureScheme,
    pub revealed: Vec<Vec<i8>>,
}

fn hash_element(data: &[i8]) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(data);
    sponge.squeeze(SECRET_ELEMENT_TRITS).trits
}

fn hash_message(message: &[i8]) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(message);
    sponge.squeeze(SIGN_DIGEST_TRITS).trits
}

fn derive_secret(seed: &[i8], index: usize, trit_val: i8) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);

    let idx_trits = [
        ((index % 3) as i8 - 1),
        (((index / 3) % 3) as i8 - 1),
        (((index / 9) % 3) as i8 - 1),
        (((index / 27) % 3) as i8 - 1),
        (((index / 81) % 3) as i8 - 1),
        (((index / 243) % 3) as i8 - 1),
        trit_val,
    ];
    sponge.absorb(&idx_trits);
    sponge.squeeze(SECRET_ELEMENT_TRITS).trits
}

fn trit_to_index(t: i8) -> usize {
    match t {
        -1 => 0,
        0 => 1,
        1 => 2,
        _ => 1,
    }
}

pub fn generate_keypair(scheme: SignatureScheme, seed: &[i8]) -> CryptoResult<(SigningKey, VerifyingKey)> {
    match scheme {
        SignatureScheme::TernaryLamport => generate_lamport_keypair(seed),
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn generate_lamport_keypair(seed: &[i8]) -> CryptoResult<(SigningKey, VerifyingKey)> {
    let total_secrets = SIGN_DIGEST_TRITS * 3;
    let mut secrets = Vec::with_capacity(total_secrets);
    let mut public = Vec::with_capacity(total_secrets);

    for pos in 0..SIGN_DIGEST_TRITS {
        for trit_val in [-1i8, 0, 1] {
            let secret = derive_secret(seed, pos, trit_val);
            let pub_hash = hash_element(&secret);
            secrets.push(secret);
            public.push(pub_hash);
        }
    }

    Ok((
        SigningKey {
            scheme: SignatureScheme::TernaryLamport,
            secrets,
            used: false,
        },
        VerifyingKey {
            scheme: SignatureScheme::TernaryLamport,
            public,
        },
    ))
}

pub fn sign(key: &mut SigningKey, message: &[i8]) -> CryptoResult<Signature> {
    if key.used {
        return Err(CryptoError::KeyGenerationFailed(
            String::from("One-time signing key already used"),
        ));
    }
    match key.scheme {
        SignatureScheme::TernaryLamport => {
            let sig = lamport_sign(key, message)?;
            key.used = true;
            Ok(sig)
        }
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn lamport_sign(key: &SigningKey, message: &[i8]) -> CryptoResult<Signature> {
    let msg_hash = hash_message(message);

    let mut revealed = Vec::with_capacity(SIGN_DIGEST_TRITS);

    for (pos, &trit) in msg_hash.iter().enumerate() {
        let secret_idx = pos * 3 + trit_to_index(trit);
        revealed.push(key.secrets[secret_idx].clone());
    }

    Ok(Signature {
        scheme: SignatureScheme::TernaryLamport,
        revealed,
    })
}

pub fn verify(key: &VerifyingKey, message: &[i8], signature: &Signature) -> CryptoResult<bool> {
    match key.scheme {
        SignatureScheme::TernaryLamport => lamport_verify(key, message, signature),
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn lamport_verify(key: &VerifyingKey, message: &[i8], signature: &Signature) -> CryptoResult<bool> {
    let msg_hash = hash_message(message);

    if signature.revealed.len() != SIGN_DIGEST_TRITS {
        return Ok(false);
    }

    for (pos, &trit) in msg_hash.iter().enumerate() {
        let pub_idx = pos * 3 + trit_to_index(trit);
        let revealed_hash = hash_element(&signature.revealed[pos]);

        let expected = &key.public[pub_idx];
        if expected.len() != revealed_hash.len() {
            return Ok(false);
        }

        let mut diff: u8 = 0;
        for (a, b) in expected.iter().zip(revealed_hash.iter()) {
            diff |= (*a as u8) ^ (*b as u8);
        }
        if diff != 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn signature_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * SECRET_ELEMENT_TRITS,
        SignatureScheme::Dilithium => 3293,
        SignatureScheme::Falcon => 1280,
        SignatureScheme::SphincsPlus => 49856,
    }
}

pub fn public_key_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS,
        _ => 0,
    }
}

pub fn secret_key_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_availability() {
        assert!(SignatureScheme::TernaryLamport.is_available());
        assert!(!SignatureScheme::Dilithium.is_available());
        assert!(!SignatureScheme::Falcon.is_available());
        assert!(!SignatureScheme::SphincsPlus.is_available());
    }

    #[test]
    fn test_scheme_security_level() {
        assert_eq!(SignatureScheme::TernaryLamport.security_level(), 128);
        assert_eq!(SignatureScheme::Dilithium.security_level(), 256);
    }

    #[test]
    fn test_generate_keypair() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        assert_eq!(sk.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(vk.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(sk.secrets.len(), SIGN_DIGEST_TRITS * 3);
        assert_eq!(vk.public.len(), SIGN_DIGEST_TRITS * 3);
        assert!(!sk.used);
    }

    #[test]
    fn test_generate_keypair_deterministic() {
        let seed = alloc::vec![1i8, 0, -1, 1, 0];
        let (sk1, vk1) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let (sk2, vk2) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        assert_eq!(sk1.secrets, sk2.secrets);
        assert_eq!(vk1.public, vk2.public);
    }

    #[test]
    fn test_generate_keypair_different_seeds() {
        let (_, vk1) = generate_keypair(SignatureScheme::TernaryLamport, &[0i8, 0, 0]).unwrap();
        let (_, vk2) = generate_keypair(SignatureScheme::TernaryLamport, &[1i8, 0, 0]).unwrap();
        assert_ne!(vk1.public, vk2.public);
    }

    #[test]
    fn test_sign() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1];
        let (mut sk, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        assert_eq!(sig.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(sig.revealed.len(), SIGN_DIGEST_TRITS);
    }

    #[test]
    fn test_sign_deterministic() {
        let seed = alloc::vec![0i8, 1, -1];
        let message = alloc::vec![1i8, 0, -1];

        let (mut sk1, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let sig1 = sign(&mut sk1, &message).unwrap();

        let (mut sk2, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let sig2 = sign(&mut sk2, &message).unwrap();

        assert_eq!(sig1.revealed, sig2.revealed);
    }

    #[test]
    fn test_sign_verify() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (mut sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        let valid = verify(&vk, &message, &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_sign_verify_wrong_message() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (mut sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        let wrong_msg = alloc::vec![0i8, 0, 0, 0, 0, 0, 0, 0, 0];
        let valid = verify(&vk, &wrong_msg, &sig).unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_one_time_guard() {
        let seed = alloc::vec![0i8, 1, -1];
        let (mut sk, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1];
        sign(&mut sk, &message).unwrap();
        assert!(sk.used);

        let result = sign(&mut sk, &message);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_scheme() {
        let result = generate_keypair(SignatureScheme::Dilithium, &[0i8]);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_sizes() {
        assert_eq!(signature_size(SignatureScheme::TernaryLamport), SIGN_DIGEST_TRITS * SECRET_ELEMENT_TRITS);
        assert!(signature_size(SignatureScheme::Dilithium) > 0);
        assert!(signature_size(SignatureScheme::Falcon) > 0);
        assert!(signature_size(SignatureScheme::SphincsPlus) > 0);
    }

    #[test]
    fn test_key_sizes() {
        let pk_size = public_key_size(SignatureScheme::TernaryLamport);
        let sk_size = secret_key_size(SignatureScheme::TernaryLamport);
        assert_eq!(pk_size, SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS);
        assert_eq!(sk_size, pk_size);
    }
}
