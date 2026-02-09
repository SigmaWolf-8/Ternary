//! Cryptographic Interoperability Layer
//!
//! Provides bidirectional conversion between standard binary ML-KEM/ML-DSA
//! formats and PlenumNET's native ternary TL-KEM/TL-DSA representations.
//! This enables hybrid deployments where ternary-native systems interoperate
//! with standard NIST post-quantum implementations.
//!
//! # Supported Conversions
//!
//! | Binary Standard | Ternary Equivalent | Direction |
//! |----------------|-------------------|-----------|
//! | ML-KEM-512 | TL-KEM-512 | Binary -> Ternary -> Binary |
//! | ML-KEM-768 | TL-KEM-768 | Binary -> Ternary -> Binary |
//! | ML-KEM-1024 | TL-KEM-1024 | Binary -> Ternary -> Binary |
//! | ML-DSA-44 | TL-DSA-44 | Binary -> Ternary -> Binary |
//! | ML-DSA-65 | TL-DSA-65 | Binary -> Ternary -> Binary |
//! | ML-DSA-87 | TL-DSA-87 | Binary -> Ternary -> Binary |
//!
//! # Architecture
//!
//! The interop layer sits between the binary compatibility gateway
//! (`BinaryTernaryGateway`) and the ternary crypto modules (`tl_kem`,
//! `tl_dsa`), providing structured encoding/decoding of cryptographic
//! parameters, keys, ciphertexts, and signatures.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use super::{CompatError, CompatResult};
use super::gateway::{binary_bytes_to_ternary, ternary_to_binary_bytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteropAlgorithm {
    MlKem512,
    MlKem768,
    MlKem1024,
    MlDsa44,
    MlDsa65,
    MlDsa87,
}

impl InteropAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            InteropAlgorithm::MlKem512 => "ML-KEM-512",
            InteropAlgorithm::MlKem768 => "ML-KEM-768",
            InteropAlgorithm::MlKem1024 => "ML-KEM-1024",
            InteropAlgorithm::MlDsa44 => "ML-DSA-44",
            InteropAlgorithm::MlDsa65 => "ML-DSA-65",
            InteropAlgorithm::MlDsa87 => "ML-DSA-87",
        }
    }

    pub fn ternary_equivalent(&self) -> &'static str {
        match self {
            InteropAlgorithm::MlKem512 => "TL-KEM-512",
            InteropAlgorithm::MlKem768 => "TL-KEM-768",
            InteropAlgorithm::MlKem1024 => "TL-KEM-1024",
            InteropAlgorithm::MlDsa44 => "TL-DSA-44",
            InteropAlgorithm::MlDsa65 => "TL-DSA-65",
            InteropAlgorithm::MlDsa87 => "TL-DSA-87",
        }
    }

    pub fn is_kem(&self) -> bool {
        matches!(self, InteropAlgorithm::MlKem512 | InteropAlgorithm::MlKem768 | InteropAlgorithm::MlKem1024)
    }

    pub fn is_dsa(&self) -> bool {
        !self.is_kem()
    }

    pub fn security_level(&self) -> u32 {
        match self {
            InteropAlgorithm::MlKem512 => 1,
            InteropAlgorithm::MlKem768 | InteropAlgorithm::MlDsa65 => 3,
            InteropAlgorithm::MlKem1024 | InteropAlgorithm::MlDsa87 => 5,
            InteropAlgorithm::MlDsa44 => 2,
        }
    }

    pub fn security_bits(&self) -> u32 {
        match self {
            InteropAlgorithm::MlKem512 | InteropAlgorithm::MlDsa44 => 128,
            InteropAlgorithm::MlKem768 | InteropAlgorithm::MlDsa65 => 192,
            InteropAlgorithm::MlKem1024 | InteropAlgorithm::MlDsa87 => 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryKeyMaterial {
    pub algorithm: InteropAlgorithm,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TernaryKeyMaterial {
    pub algorithm: InteropAlgorithm,
    pub trits: Vec<i8>,
}

#[derive(Debug, Clone)]
pub struct BinaryCiphertext {
    pub algorithm: InteropAlgorithm,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TernaryCiphertext {
    pub algorithm: InteropAlgorithm,
    pub trits: Vec<i8>,
}

#[derive(Debug, Clone)]
pub struct BinarySignature {
    pub algorithm: InteropAlgorithm,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TernarySignature {
    pub algorithm: InteropAlgorithm,
    pub trits: Vec<i8>,
}

#[derive(Debug, Clone)]
pub struct InteropStats {
    pub keys_converted: u64,
    pub ciphertexts_converted: u64,
    pub signatures_converted: u64,
    pub bytes_processed: u64,
    pub trits_processed: u64,
}

pub struct CryptoInteropBridge {
    stats: InteropStats,
}

impl CryptoInteropBridge {
    pub fn new() -> Self {
        Self {
            stats: InteropStats {
                keys_converted: 0,
                ciphertexts_converted: 0,
                signatures_converted: 0,
                bytes_processed: 0,
                trits_processed: 0,
            },
        }
    }

    pub fn key_to_ternary(&mut self, key: &BinaryKeyMaterial) -> CompatResult<TernaryKeyMaterial> {
        if key.data.is_empty() {
            return Err(CompatError::InvalidBinaryData);
        }

        let trits = binary_bytes_to_ternary(&key.data);
        self.stats.keys_converted += 1;
        self.stats.bytes_processed += key.data.len() as u64;
        self.stats.trits_processed += trits.len() as u64;

        Ok(TernaryKeyMaterial {
            algorithm: key.algorithm,
            trits,
        })
    }

    pub fn key_to_binary(&mut self, key: &TernaryKeyMaterial) -> CompatResult<BinaryKeyMaterial> {
        if key.trits.is_empty() {
            return Err(CompatError::InvalidTernaryData);
        }

        let padded = pad_trits_to_6(&key.trits);
        let data = ternary_to_binary_bytes(&padded)?;
        self.stats.keys_converted += 1;
        self.stats.bytes_processed += data.len() as u64;
        self.stats.trits_processed += key.trits.len() as u64;

        Ok(BinaryKeyMaterial {
            algorithm: key.algorithm,
            data,
        })
    }

    pub fn ciphertext_to_ternary(&mut self, ct: &BinaryCiphertext) -> CompatResult<TernaryCiphertext> {
        if ct.data.is_empty() {
            return Err(CompatError::InvalidBinaryData);
        }

        let trits = binary_bytes_to_ternary(&ct.data);
        self.stats.ciphertexts_converted += 1;
        self.stats.bytes_processed += ct.data.len() as u64;
        self.stats.trits_processed += trits.len() as u64;

        Ok(TernaryCiphertext {
            algorithm: ct.algorithm,
            trits,
        })
    }

    pub fn ciphertext_to_binary(&mut self, ct: &TernaryCiphertext) -> CompatResult<BinaryCiphertext> {
        if ct.trits.is_empty() {
            return Err(CompatError::InvalidTernaryData);
        }

        let padded = pad_trits_to_6(&ct.trits);
        let data = ternary_to_binary_bytes(&padded)?;
        self.stats.ciphertexts_converted += 1;
        self.stats.bytes_processed += data.len() as u64;
        self.stats.trits_processed += ct.trits.len() as u64;

        Ok(BinaryCiphertext {
            algorithm: ct.algorithm,
            data,
        })
    }

    pub fn signature_to_ternary(&mut self, sig: &BinarySignature) -> CompatResult<TernarySignature> {
        if sig.data.is_empty() {
            return Err(CompatError::InvalidBinaryData);
        }

        let trits = binary_bytes_to_ternary(&sig.data);
        self.stats.signatures_converted += 1;
        self.stats.bytes_processed += sig.data.len() as u64;
        self.stats.trits_processed += trits.len() as u64;

        Ok(TernarySignature {
            algorithm: sig.algorithm,
            trits,
        })
    }

    pub fn signature_to_binary(&mut self, sig: &TernarySignature) -> CompatResult<BinarySignature> {
        if sig.trits.is_empty() {
            return Err(CompatError::InvalidTernaryData);
        }

        let padded = pad_trits_to_6(&sig.trits);
        let data = ternary_to_binary_bytes(&padded)?;
        self.stats.signatures_converted += 1;
        self.stats.bytes_processed += data.len() as u64;
        self.stats.trits_processed += sig.trits.len() as u64;

        Ok(BinarySignature {
            algorithm: sig.algorithm,
            data,
        })
    }

    pub fn seed_to_ternary(&mut self, binary_seed: &[u8], algorithm: InteropAlgorithm) -> CompatResult<Vec<i8>> {
        if binary_seed.is_empty() {
            return Err(CompatError::InvalidBinaryData);
        }
        let trits = binary_bytes_to_ternary(binary_seed);
        self.stats.bytes_processed += binary_seed.len() as u64;
        self.stats.trits_processed += trits.len() as u64;
        let _ = algorithm;
        Ok(trits)
    }

    pub fn shared_secret_to_binary(&mut self, ternary_secret: &[i8], algorithm: InteropAlgorithm) -> CompatResult<Vec<u8>> {
        if ternary_secret.is_empty() {
            return Err(CompatError::InvalidTernaryData);
        }
        let padded = pad_trits_to_6(ternary_secret);
        let bytes = ternary_to_binary_bytes(&padded)?;
        self.stats.bytes_processed += bytes.len() as u64;
        self.stats.trits_processed += ternary_secret.len() as u64;
        let _ = algorithm;
        Ok(bytes)
    }

    pub fn stats(&self) -> &InteropStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = InteropStats {
            keys_converted: 0,
            ciphertexts_converted: 0,
            signatures_converted: 0,
            bytes_processed: 0,
            trits_processed: 0,
        };
    }
}

fn pad_trits_to_6(trits: &[i8]) -> Vec<i8> {
    let remainder = trits.len() % 6;
    if remainder == 0 {
        return trits.to_vec();
    }
    let padding = 6 - remainder;
    let mut padded = trits.to_vec();
    padded.extend(vec![0i8; padding]);
    padded
}

#[derive(Debug, Clone)]
pub struct InteropCapability {
    pub algorithm: InteropAlgorithm,
    pub key_encoding: bool,
    pub ciphertext_encoding: bool,
    pub signature_encoding: bool,
    pub seed_conversion: bool,
    pub shared_secret_conversion: bool,
}

pub fn list_capabilities() -> Vec<InteropCapability> {
    let algorithms = [
        InteropAlgorithm::MlKem512,
        InteropAlgorithm::MlKem768,
        InteropAlgorithm::MlKem1024,
        InteropAlgorithm::MlDsa44,
        InteropAlgorithm::MlDsa65,
        InteropAlgorithm::MlDsa87,
    ];

    algorithms.iter().map(|&alg| {
        InteropCapability {
            algorithm: alg,
            key_encoding: true,
            ciphertext_encoding: alg.is_kem(),
            signature_encoding: alg.is_dsa(),
            seed_conversion: true,
            shared_secret_conversion: alg.is_kem(),
        }
    }).collect()
}

pub fn validate_interop_readiness() -> InteropReadinessReport {
    let capabilities = list_capabilities();
    let total = capabilities.len();
    let ready = capabilities.iter().filter(|c| c.key_encoding && c.seed_conversion).count();

    InteropReadinessReport {
        total_algorithms: total,
        ready_algorithms: ready,
        readiness_percentage: if total > 0 { (ready * 100) / total } else { 0 },
        capabilities,
    }
}

#[derive(Debug, Clone)]
pub struct InteropReadinessReport {
    pub total_algorithms: usize,
    pub ready_algorithms: usize,
    pub readiness_percentage: usize,
    pub capabilities: Vec<InteropCapability>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_names() {
        assert_eq!(InteropAlgorithm::MlKem512.name(), "ML-KEM-512");
        assert_eq!(InteropAlgorithm::MlKem768.name(), "ML-KEM-768");
        assert_eq!(InteropAlgorithm::MlKem1024.name(), "ML-KEM-1024");
        assert_eq!(InteropAlgorithm::MlDsa44.name(), "ML-DSA-44");
        assert_eq!(InteropAlgorithm::MlDsa65.name(), "ML-DSA-65");
        assert_eq!(InteropAlgorithm::MlDsa87.name(), "ML-DSA-87");
    }

    #[test]
    fn test_ternary_equivalents() {
        assert_eq!(InteropAlgorithm::MlKem512.ternary_equivalent(), "TL-KEM-512");
        assert_eq!(InteropAlgorithm::MlDsa44.ternary_equivalent(), "TL-DSA-44");
    }

    #[test]
    fn test_algorithm_classification() {
        assert!(InteropAlgorithm::MlKem512.is_kem());
        assert!(!InteropAlgorithm::MlKem512.is_dsa());
        assert!(InteropAlgorithm::MlDsa44.is_dsa());
        assert!(!InteropAlgorithm::MlDsa44.is_kem());
    }

    #[test]
    fn test_security_levels() {
        assert_eq!(InteropAlgorithm::MlKem512.security_level(), 1);
        assert_eq!(InteropAlgorithm::MlKem768.security_level(), 3);
        assert_eq!(InteropAlgorithm::MlKem1024.security_level(), 5);
        assert_eq!(InteropAlgorithm::MlDsa44.security_level(), 2);
        assert_eq!(InteropAlgorithm::MlDsa65.security_level(), 3);
        assert_eq!(InteropAlgorithm::MlDsa87.security_level(), 5);
    }

    #[test]
    fn test_security_bits() {
        assert_eq!(InteropAlgorithm::MlKem512.security_bits(), 128);
        assert_eq!(InteropAlgorithm::MlKem768.security_bits(), 192);
        assert_eq!(InteropAlgorithm::MlKem1024.security_bits(), 256);
    }

    #[test]
    fn test_key_roundtrip_kem() {
        let mut bridge = CryptoInteropBridge::new();
        let binary_key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem768,
            data: vec![42u8, 0, 255, 128, 1, 73],
        };

        let ternary = bridge.key_to_ternary(&binary_key).unwrap();
        assert_eq!(ternary.algorithm, InteropAlgorithm::MlKem768);
        assert!(!ternary.trits.is_empty());

        let back = bridge.key_to_binary(&ternary).unwrap();
        assert_eq!(back.data, binary_key.data);
        assert_eq!(back.algorithm, InteropAlgorithm::MlKem768);
    }

    #[test]
    fn test_key_roundtrip_dsa() {
        let mut bridge = CryptoInteropBridge::new();
        let binary_key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlDsa65,
            data: vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };

        let ternary = bridge.key_to_ternary(&binary_key).unwrap();
        let back = bridge.key_to_binary(&ternary).unwrap();
        assert_eq!(back.data, binary_key.data);
    }

    #[test]
    fn test_ciphertext_roundtrip() {
        let mut bridge = CryptoInteropBridge::new();
        let ct = BinaryCiphertext {
            algorithm: InteropAlgorithm::MlKem512,
            data: vec![100u8, 200, 50, 25, 0, 255],
        };

        let ternary = bridge.ciphertext_to_ternary(&ct).unwrap();
        assert_eq!(ternary.algorithm, InteropAlgorithm::MlKem512);

        let back = bridge.ciphertext_to_binary(&ternary).unwrap();
        assert_eq!(back.data, ct.data);
    }

    #[test]
    fn test_signature_roundtrip() {
        let mut bridge = CryptoInteropBridge::new();
        let sig = BinarySignature {
            algorithm: InteropAlgorithm::MlDsa87,
            data: vec![10u8, 20, 30, 40, 50, 60, 70, 80, 90],
        };

        let ternary = bridge.signature_to_ternary(&sig).unwrap();
        assert_eq!(ternary.algorithm, InteropAlgorithm::MlDsa87);

        let back = bridge.signature_to_binary(&ternary).unwrap();
        assert_eq!(back.data, sig.data);
    }

    #[test]
    fn test_seed_conversion() {
        let mut bridge = CryptoInteropBridge::new();
        let seed = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                        16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let trits = bridge.seed_to_ternary(&seed, InteropAlgorithm::MlKem768).unwrap();
        assert_eq!(trits.len(), seed.len() * 6);
    }

    #[test]
    fn test_shared_secret_conversion() {
        let mut bridge = CryptoInteropBridge::new();
        let trits = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let bytes = bridge.shared_secret_to_binary(&trits, InteropAlgorithm::MlKem512).unwrap();
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn test_empty_key_error() {
        let mut bridge = CryptoInteropBridge::new();
        let empty_key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem512,
            data: vec![],
        };
        assert!(bridge.key_to_ternary(&empty_key).is_err());
    }

    #[test]
    fn test_empty_ternary_key_error() {
        let mut bridge = CryptoInteropBridge::new();
        let empty_key = TernaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem512,
            trits: vec![],
        };
        assert!(bridge.key_to_binary(&empty_key).is_err());
    }

    #[test]
    fn test_stats_tracking() {
        let mut bridge = CryptoInteropBridge::new();

        let key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem512,
            data: vec![42u8, 73, 99, 0, 128, 255],
        };
        let _ = bridge.key_to_ternary(&key).unwrap();

        let ct = BinaryCiphertext {
            algorithm: InteropAlgorithm::MlKem512,
            data: vec![1u8, 2, 3],
        };
        let _ = bridge.ciphertext_to_ternary(&ct).unwrap();

        let sig = BinarySignature {
            algorithm: InteropAlgorithm::MlDsa44,
            data: vec![10u8, 20, 30],
        };
        let _ = bridge.signature_to_ternary(&sig).unwrap();

        let stats = bridge.stats();
        assert_eq!(stats.keys_converted, 1);
        assert_eq!(stats.ciphertexts_converted, 1);
        assert_eq!(stats.signatures_converted, 1);
        assert!(stats.bytes_processed > 0);
        assert!(stats.trits_processed > 0);
    }

    #[test]
    fn test_stats_reset() {
        let mut bridge = CryptoInteropBridge::new();
        let key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem512,
            data: vec![42u8, 73, 99],
        };
        let _ = bridge.key_to_ternary(&key).unwrap();
        bridge.reset_stats();
        assert_eq!(bridge.stats().keys_converted, 0);
        assert_eq!(bridge.stats().bytes_processed, 0);
    }

    #[test]
    fn test_pad_trits_to_6() {
        assert_eq!(pad_trits_to_6(&[]).len(), 0);
        assert_eq!(pad_trits_to_6(&[0]).len(), 6);
        assert_eq!(pad_trits_to_6(&[0, 1]).len(), 6);
        assert_eq!(pad_trits_to_6(&[0, 1, -1, 0, 1, -1]).len(), 6);
        assert_eq!(pad_trits_to_6(&[0, 1, -1, 0, 1, -1, 0]).len(), 12);
    }

    #[test]
    fn test_list_capabilities() {
        let caps = list_capabilities();
        assert_eq!(caps.len(), 6);

        let kem_caps: Vec<_> = caps.iter().filter(|c| c.algorithm.is_kem()).collect();
        assert_eq!(kem_caps.len(), 3);
        for cap in &kem_caps {
            assert!(cap.key_encoding);
            assert!(cap.ciphertext_encoding);
            assert!(!cap.signature_encoding);
            assert!(cap.seed_conversion);
            assert!(cap.shared_secret_conversion);
        }

        let dsa_caps: Vec<_> = caps.iter().filter(|c| c.algorithm.is_dsa()).collect();
        assert_eq!(dsa_caps.len(), 3);
        for cap in &dsa_caps {
            assert!(cap.key_encoding);
            assert!(!cap.ciphertext_encoding);
            assert!(cap.signature_encoding);
            assert!(cap.seed_conversion);
            assert!(!cap.shared_secret_conversion);
        }
    }

    #[test]
    fn test_validate_interop_readiness() {
        let report = validate_interop_readiness();
        assert_eq!(report.total_algorithms, 6);
        assert_eq!(report.ready_algorithms, 6);
        assert_eq!(report.readiness_percentage, 100);
    }

    #[test]
    fn test_large_key_roundtrip() {
        let mut bridge = CryptoInteropBridge::new();
        let large_key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem1024,
            data: (0..=255).collect::<Vec<u8>>().repeat(6),
        };
        let ternary = bridge.key_to_ternary(&large_key).unwrap();
        let back = bridge.key_to_binary(&ternary).unwrap();
        assert_eq!(back.data, large_key.data);
    }

    #[test]
    fn test_all_byte_values_roundtrip() {
        let mut bridge = CryptoInteropBridge::new();
        let all_bytes: Vec<u8> = (0..=255).collect();
        let key = BinaryKeyMaterial {
            algorithm: InteropAlgorithm::MlKem512,
            data: all_bytes.clone(),
        };
        let ternary = bridge.key_to_ternary(&key).unwrap();
        let back = bridge.key_to_binary(&ternary).unwrap();

        for (i, (&original, &recovered)) in all_bytes.iter().zip(back.data.iter()).enumerate() {
            assert_eq!(original, recovered, "Byte value {} failed roundtrip at index {}", original, i);
        }
    }
}
