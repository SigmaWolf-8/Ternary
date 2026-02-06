//! Ternary HMAC (Hash-based Message Authentication Code)
//!
//! Implements HMAC using the ternary sponge hash for keyed message
//! authentication. Follows the standard HMAC construction:
//!   HMAC(K, M) = H((K ^ opad) || H((K ^ ipad) || M))

use alloc::vec::Vec;
use super::{TernaryDigest, TERNARY_KEY_TRITS, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;

const IPAD_TRIT: i8 = 1;
const OPAD_TRIT: i8 = -1;

fn trit_xor(a: i8, b: i8) -> i8 {
    let sum = (a + 1 + b + 1) % 3;
    sum as i8 - 1
}

fn pad_key(key: &[i8]) -> Vec<i8> {
    let mut padded = Vec::with_capacity(TERNARY_KEY_TRITS);
    if key.len() > TERNARY_KEY_TRITS {
        let mut sponge = TernarySponge::new();
        sponge.absorb(key);
        let hashed = sponge.squeeze(TERNARY_KEY_TRITS);
        padded.extend_from_slice(&hashed.trits);
    } else {
        padded.extend_from_slice(key);
        padded.resize(TERNARY_KEY_TRITS, 0);
    }
    padded
}

pub fn ternary_hmac(key: &[i8], message: &[i8]) -> TernaryDigest {
    let padded_key = pad_key(key);

    let ikey: Vec<i8> = padded_key.iter().map(|&k| trit_xor(k, IPAD_TRIT)).collect();
    let okey: Vec<i8> = padded_key.iter().map(|&k| trit_xor(k, OPAD_TRIT)).collect();

    let mut inner_sponge = TernarySponge::new();
    inner_sponge.absorb(&ikey);
    inner_sponge.absorb(message);
    let inner_hash = inner_sponge.squeeze_default();

    let mut outer_sponge = TernarySponge::new();
    outer_sponge.absorb(&okey);
    outer_sponge.absorb(&inner_hash.trits);
    outer_sponge.squeeze_default()
}

pub fn ternary_hmac_bytes(key: &[u8], message: &[u8]) -> TernaryDigest {
    let key_trits = bytes_to_trits(key);
    let msg_trits = bytes_to_trits(message);
    ternary_hmac(&key_trits, &msg_trits)
}

pub fn verify_hmac(key: &[i8], message: &[i8], expected: &TernaryDigest) -> bool {
    let computed = ternary_hmac(key, message);
    constant_time_compare(&computed.trits, &expected.trits)
}

pub fn verify_hmac_bytes(key: &[u8], message: &[u8], expected: &TernaryDigest) -> bool {
    let computed = ternary_hmac_bytes(key, message);
    constant_time_compare(&computed.trits, &expected.trits)
}

fn constant_time_compare(a: &[i8], b: &[i8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= (*x as u8) ^ (*y as u8);
    }
    diff == 0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_deterministic() {
        let key = alloc::vec![0i8, 1, -1, 0, 1];
        let msg = alloc::vec![1i8, 0, -1, 1, 0];
        let h1 = ternary_hmac(&key, &msg);
        let h2 = ternary_hmac(&key, &msg);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hmac_output_length() {
        let key = alloc::vec![0i8; 10];
        let msg = alloc::vec![1i8; 20];
        let h = ternary_hmac(&key, &msg);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_hmac_different_keys() {
        let msg = alloc::vec![0i8, 1, -1];
        let h1 = ternary_hmac(&[0i8, 0, 0], &msg);
        let h2 = ternary_hmac(&[1i8, 0, 0], &msg);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hmac_different_messages() {
        let key = alloc::vec![0i8, 1, -1];
        let h1 = ternary_hmac(&key, &[0i8, 0, 0]);
        let h2 = ternary_hmac(&key, &[1i8, 0, 0]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hmac_verify() {
        let key = alloc::vec![0i8, 1, -1, 0, 1];
        let msg = alloc::vec![1i8, 0, -1, 1, 0];
        let mac = ternary_hmac(&key, &msg);
        assert!(verify_hmac(&key, &msg, &mac));
    }

    #[test]
    fn test_hmac_verify_wrong_message() {
        let key = alloc::vec![0i8, 1, -1, 0, 1];
        let msg = alloc::vec![1i8, 0, -1, 1, 0];
        let mac = ternary_hmac(&key, &msg);
        assert!(!verify_hmac(&key, &[0i8, 0, 0, 0, 0], &mac));
    }

    #[test]
    fn test_hmac_verify_wrong_key() {
        let key = alloc::vec![0i8, 1, -1, 0, 1];
        let msg = alloc::vec![1i8, 0, -1, 1, 0];
        let mac = ternary_hmac(&key, &msg);
        assert!(!verify_hmac(&[1i8, 1, 1, 1, 1], &msg, &mac));
    }

    #[test]
    fn test_hmac_bytes() {
        let h1 = ternary_hmac_bytes(b"secret", b"message");
        let h2 = ternary_hmac_bytes(b"secret", b"message");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_verify_hmac_bytes() {
        let mac = ternary_hmac_bytes(b"key", b"data");
        assert!(verify_hmac_bytes(b"key", b"data", &mac));
        assert!(!verify_hmac_bytes(b"wrong", b"data", &mac));
    }

    #[test]
    fn test_hmac_long_key() {
        let long_key = alloc::vec![1i8; 500];
        let msg = alloc::vec![0i8; 10];
        let h = ternary_hmac(&long_key, &msg);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_constant_time_compare() {
        let a = alloc::vec![0i8, 1, -1];
        let b = alloc::vec![0i8, 1, -1];
        assert!(constant_time_compare(&a, &b));

        let c = alloc::vec![1i8, 1, -1];
        assert!(!constant_time_compare(&a, &c));
    }

    #[test]
    fn test_constant_time_compare_length_mismatch() {
        let a = alloc::vec![0i8, 1];
        let b = alloc::vec![0i8, 1, -1];
        assert!(!constant_time_compare(&a, &b));
    }
}
