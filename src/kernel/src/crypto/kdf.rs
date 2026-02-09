//! Ternary Key Derivation Function
//!
//! Derives cryptographic keys from passwords, seeds, or other key material.
//! Uses iterated ternary HMAC with salt for key stretching.

use alloc::vec::Vec;
use super::{CryptoError, CryptoResult, TernaryDigest, TERNARY_KEY_TRITS};
use super::hmac::ternary_hmac;
use super::sponge::TernarySponge;

const DEFAULT_ITERATIONS: u32 = 1000;
const MIN_SALT_TRITS: usize = 27;

pub struct KdfParams {
    pub iterations: u32,
    pub output_trits: usize,
    pub salt: Vec<i8>,
}

impl KdfParams {
    pub fn new(salt: Vec<i8>, iterations: u32, output_trits: usize) -> CryptoResult<Self> {
        if salt.len() < MIN_SALT_TRITS {
            return Err(CryptoError::InvalidInputLength {
                expected: MIN_SALT_TRITS,
                actual: salt.len(),
            });
        }
        for &t in &salt {
            if t < -1 || t > 1 {
                return Err(CryptoError::InvalidTritValue(t));
            }
        }
        Ok(Self { iterations, output_trits, salt })
    }

    pub fn default_with_salt(salt: Vec<i8>) -> CryptoResult<Self> {
        Self::new(salt, DEFAULT_ITERATIONS, TERNARY_KEY_TRITS)
    }
}

pub fn derive_key(password: &[i8], params: &KdfParams) -> TernaryDigest {
    let mut combined = Vec::with_capacity(password.len() + params.salt.len());
    combined.extend_from_slice(password);
    combined.extend_from_slice(&params.salt);

    let mut result = ternary_hmac(&params.salt, &combined);

    for _ in 1..params.iterations {
        result = ternary_hmac(&params.salt, &result.trits);
    }

    if params.output_trits != result.len() {
        let mut sponge = TernarySponge::new();
        sponge.absorb(&result.trits);
        result = sponge.squeeze(params.output_trits);
    }

    result
}

pub fn derive_key_bytes(password: &[u8], salt: &[u8], iterations: u32) -> TernaryDigest {
    let pwd_trits = bytes_to_trits(password);
    let salt_trits = bytes_to_trits(salt);

    let effective_salt = if salt_trits.len() < MIN_SALT_TRITS {
        let mut padded = salt_trits;
        padded.resize(MIN_SALT_TRITS, 0);
        padded
    } else {
        salt_trits
    };

    let params = KdfParams {
        iterations,
        output_trits: TERNARY_KEY_TRITS,
        salt: effective_salt,
    };

    derive_key(&pwd_trits, &params)
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

pub fn derive_subkey(master_key: &TernaryDigest, context: &[i8], index: u32) -> TernaryDigest {
    let mut input = Vec::with_capacity(master_key.len() + context.len() + 4);
    input.extend_from_slice(&master_key.trits);
    input.extend_from_slice(context);

    let idx_trits = [
        ((index % 3) as i8 - 1),
        (((index / 3) % 3) as i8 - 1),
        (((index / 9) % 3) as i8 - 1),
        (((index / 27) % 3) as i8 - 1),
    ];
    input.extend_from_slice(&idx_trits);

    ternary_hmac(&master_key.trits, &input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_salt() -> Vec<i8> {
        alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1]
    }

    #[test]
    fn test_kdf_params_creation() {
        let params = KdfParams::new(make_salt(), 100, 243);
        assert!(params.is_ok());
    }

    #[test]
    fn test_kdf_params_short_salt() {
        let short_salt = alloc::vec![0i8; 10];
        let params = KdfParams::new(short_salt, 100, 243);
        assert!(params.is_err());
    }

    #[test]
    fn test_kdf_params_invalid_trit() {
        let mut salt = make_salt();
        salt[0] = 2;
        let params = KdfParams::new(salt, 100, 243);
        assert!(params.is_err());
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = alloc::vec![0i8, 1, -1, 0, 1];
        let params = KdfParams::default_with_salt(make_salt()).unwrap();
        let k1 = derive_key(&password, &params);
        let k2 = derive_key(&password, &params);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let params = KdfParams::default_with_salt(make_salt()).unwrap();
        let k1 = derive_key(&[0i8, 0, 0], &params);
        let k2 = derive_key(&[1i8, 0, 0], &params);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = alloc::vec![0i8, 1, -1];
        let salt1 = make_salt();
        let mut salt2 = make_salt();
        salt2[0] = 1;

        let p1 = KdfParams::default_with_salt(salt1).unwrap();
        let p2 = KdfParams::default_with_salt(salt2).unwrap();

        let k1 = derive_key(&password, &p1);
        let k2 = derive_key(&password, &p2);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_derive_key_output_length() {
        let password = alloc::vec![0i8, 1, -1];
        let params = KdfParams::new(make_salt(), 10, 100).unwrap();
        let key = derive_key(&password, &params);
        assert_eq!(key.len(), 100);
    }

    #[test]
    fn test_derive_key_bytes() {
        let k1 = derive_key_bytes(b"password", b"saltsaltsalt", 10);
        let k2 = derive_key_bytes(b"password", b"saltsaltsalt", 10);
        assert_eq!(k1, k2);

        let k3 = derive_key_bytes(b"different", b"saltsaltsalt", 10);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_derive_subkey() {
        let master = TernaryDigest::zero(TERNARY_KEY_TRITS);
        let context = alloc::vec![1i8, 0, -1];

        let sub0 = derive_subkey(&master, &context, 0);
        let sub1 = derive_subkey(&master, &context, 1);
        assert_ne!(sub0, sub1);

        let sub0_again = derive_subkey(&master, &context, 0);
        assert_eq!(sub0, sub0_again);
    }

    #[test]
    fn test_derive_subkey_different_context() {
        let master = TernaryDigest::zero(TERNARY_KEY_TRITS);
        let ctx_a = alloc::vec![1i8, 0, -1];
        let ctx_b = alloc::vec![0i8, 1, -1];

        let k1 = derive_subkey(&master, &ctx_a, 0);
        let k2 = derive_subkey(&master, &ctx_b, 0);
        assert_ne!(k1, k2);
    }
}
