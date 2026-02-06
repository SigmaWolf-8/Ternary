//! PlenumNET Cryptographic Primitives
//!
//! Provides ternary-aware cryptographic operations for the Salvi Framework:
//! - **Ternary Hashing**: Hash functions operating on trit-based data
//! - **Sponge Construction**: Keccak-inspired sponge for ternary domains
//! - **HMAC**: Keyed message authentication using ternary hash
//! - **Key Derivation**: KDF for generating keys from passwords/seeds
//! - **Post-Quantum Signatures**: Stub interface for lattice-based signatures
//!
//! All operations work with both binary byte arrays and ternary trit arrays,
//! with automatic conversion between representations.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod hash;
pub mod sponge;
pub mod hmac;
pub mod kdf;
pub mod signature;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidKeyLength { expected: usize, actual: usize },
    InvalidInputLength { expected: usize, actual: usize },
    InvalidTritValue(i8),
    HashMismatch,
    SignatureInvalid,
    KeyGenerationFailed(String),
    UnsupportedAlgorithm(String),
}

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::InvalidKeyLength { expected, actual } => {
                write!(f, "Invalid key length: expected {}, got {}", expected, actual)
            }
            CryptoError::InvalidInputLength { expected, actual } => {
                write!(f, "Invalid input length: expected {}, got {}", expected, actual)
            }
            CryptoError::InvalidTritValue(v) => {
                write!(f, "Invalid trit value: {}", v)
            }
            CryptoError::HashMismatch => write!(f, "Hash mismatch"),
            CryptoError::SignatureInvalid => write!(f, "Signature verification failed"),
            CryptoError::KeyGenerationFailed(msg) => {
                write!(f, "Key generation failed: {}", msg)
            }
            CryptoError::UnsupportedAlgorithm(alg) => {
                write!(f, "Unsupported algorithm: {}", alg)
            }
        }
    }
}

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

pub const TERNARY_HASH_TRITS: usize = 243;
pub const TERNARY_HASH_BYTES: usize = 48;
pub const TERNARY_KEY_TRITS: usize = 243;
pub const TERNARY_KEY_BYTES: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryDigest {
    pub trits: Vec<i8>,
}

impl TernaryDigest {
    pub fn new(trits: Vec<i8>) -> CryptoResult<Self> {
        for &t in &trits {
            if t < -1 || t > 1 {
                return Err(CryptoError::InvalidTritValue(t));
            }
        }
        Ok(Self { trits })
    }

    pub fn zero(len: usize) -> Self {
        Self { trits: alloc::vec![0i8; len] }
    }

    pub fn len(&self) -> usize {
        self.trits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trits.is_empty()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for chunk in self.trits.chunks(5) {
            let mut val: u8 = 0;
            for (i, &t) in chunk.iter().enumerate() {
                let b_val = (t + 1) as u8;
                val += b_val * 3u8.pow(i as u32);
            }
            bytes.push(val);
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8], trit_count: usize) -> Self {
        let mut trits = Vec::with_capacity(trit_count);
        for &byte in bytes {
            let mut val = byte;
            for _ in 0..5 {
                if trits.len() >= trit_count {
                    break;
                }
                let remainder = val % 3;
                let trit = remainder as i8 - 1;
                trits.push(trit);
                val /= 3;
            }
        }
        trits.truncate(trit_count);
        Self { trits }
    }

    pub fn xor(&self, other: &TernaryDigest) -> CryptoResult<Self> {
        if self.trits.len() != other.trits.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.trits.len(),
                actual: other.trits.len(),
            });
        }
        let result: Vec<i8> = self.trits.iter()
            .zip(other.trits.iter())
            .map(|(&a, &b)| {
                let sum = (a + 1 + b + 1) % 3;
                sum as i8 - 1
            })
            .collect();
        Ok(Self { trits: result })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_digest_creation() {
        let d = TernaryDigest::new(alloc::vec![0, 1, -1, 0, 1]).unwrap();
        assert_eq!(d.len(), 5);
    }

    #[test]
    fn test_ternary_digest_invalid_trit() {
        let result = TernaryDigest::new(alloc::vec![0, 1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ternary_digest_zero() {
        let d = TernaryDigest::zero(10);
        assert_eq!(d.len(), 10);
        assert!(d.trits.iter().all(|&t| t == 0));
    }

    #[test]
    fn test_ternary_digest_bytes_roundtrip() {
        let original = TernaryDigest::new(alloc::vec![0, 1, -1, 0, 1, -1, 0, 1, -1, 0]).unwrap();
        let bytes = original.to_bytes();
        let restored = TernaryDigest::from_bytes(&bytes, 10);
        assert_eq!(original.trits, restored.trits);
    }

    #[test]
    fn test_ternary_digest_xor() {
        let a = TernaryDigest::new(alloc::vec![0, 1, -1]).unwrap();
        let b = TernaryDigest::new(alloc::vec![1, -1, 0]).unwrap();
        let result = a.xor(&b).unwrap();
        assert_eq!(result.len(), 3);
        for &t in &result.trits {
            assert!(t >= -1 && t <= 1);
        }
    }

    #[test]
    fn test_ternary_digest_xor_length_mismatch() {
        let a = TernaryDigest::new(alloc::vec![0, 1]).unwrap();
        let b = TernaryDigest::new(alloc::vec![0, 1, -1]).unwrap();
        assert!(a.xor(&b).is_err());
    }

    #[test]
    fn test_crypto_error_display() {
        let err = CryptoError::InvalidKeyLength { expected: 48, actual: 32 };
        let msg = alloc::format!("{}", err);
        assert!(msg.contains("48"));
        assert!(msg.contains("32"));
    }
}
