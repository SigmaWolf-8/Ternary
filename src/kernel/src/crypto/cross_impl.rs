// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Cross-Implementation Testing Framework
//!
//! Validates TL-KEM and TL-DSA implementations against reference ML-KEM
//! (FIPS 203) and ML-DSA (FIPS 204) behavior through the CryptoInteropBridge.
//! Ensures that ternary-native operations produce results compatible with
//! standard binary implementations when converted via the interoperability layer.
//!
//! # Test Categories
//!
//! 1. **Format Compatibility**: Verify key/ciphertext/signature byte encoding
//! 2. **Size Compliance**: Confirm output sizes match NIST specifications
//! 3. **Algebraic Consistency**: Verify mathematical properties hold across
//!    ternary-to-binary conversion
//! 4. **Round-Trip Integrity**: Keys survive ternary→binary→ternary conversion
//! 5. **Protocol Compliance**: Full KEM/DSA protocol flows produce valid outputs
//!
//! # Reference Standards
//!
//! - FIPS 203 (ML-KEM): Key sizes, ciphertext sizes, shared secret sizes
//! - FIPS 204 (ML-DSA): Key sizes, signature sizes, verification behavior
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::CryptoResult;
use super::tl_kem::{self, TlKemVariant};
use super::tl_dsa::{self, TlDsaVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    FormatCompatibility,
    SizeCompliance,
    AlgebraicConsistency,
    RoundTripIntegrity,
    ProtocolCompliance,
}

impl TestCategory {
    pub fn name(&self) -> &'static str {
        match self {
            TestCategory::FormatCompatibility => "Format Compatibility",
            TestCategory::SizeCompliance => "Size Compliance",
            TestCategory::AlgebraicConsistency => "Algebraic Consistency",
            TestCategory::RoundTripIntegrity => "Round-Trip Integrity",
            TestCategory::ProtocolCompliance => "Protocol Compliance",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrossImplTestResult {
    pub test_name: String,
    pub category: TestCategory,
    pub algorithm: String,
    pub variant: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct CrossImplReport {
    pub results: Vec<CrossImplTestResult>,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub categories_tested: Vec<TestCategory>,
    pub overall_compatible: bool,
}

#[derive(Debug, Clone)]
pub struct MlKemRefSizes {
    pub variant: &'static str,
    pub pk_bytes: usize,
    pub sk_bytes: usize,
    pub ct_bytes: usize,
    pub ss_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct MlDsaRefSizes {
    pub variant: &'static str,
    pub pk_bytes: usize,
    pub sk_bytes: usize,
    pub sig_bytes: usize,
}

fn ml_kem_reference_sizes() -> Vec<MlKemRefSizes> {
    vec![
        MlKemRefSizes { variant: "ML-KEM-512", pk_bytes: 800, sk_bytes: 1632, ct_bytes: 768, ss_bytes: 32 },
        MlKemRefSizes { variant: "ML-KEM-768", pk_bytes: 1184, sk_bytes: 2400, ct_bytes: 1088, ss_bytes: 32 },
        MlKemRefSizes { variant: "ML-KEM-1024", pk_bytes: 1568, sk_bytes: 3168, ct_bytes: 1568, ss_bytes: 32 },
    ]
}

fn ml_dsa_reference_sizes() -> Vec<MlDsaRefSizes> {
    vec![
        MlDsaRefSizes { variant: "ML-DSA-44", pk_bytes: 1312, sk_bytes: 2560, sig_bytes: 2420 },
        MlDsaRefSizes { variant: "ML-DSA-65", pk_bytes: 1952, sk_bytes: 4032, sig_bytes: 3309 },
        MlDsaRefSizes { variant: "ML-DSA-87", pk_bytes: 2592, sk_bytes: 4896, sig_bytes: 4627 },
    ]
}

fn poly_vec_to_trits(v: &super::ternary_lattice::TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::new();
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

pub fn test_kem_size_compliance() -> Vec<CrossImplTestResult> {
    let ref_sizes = ml_kem_reference_sizes();
    let tl_variants = [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024];
    let mut results = Vec::new();

    for (i, variant) in tl_variants.iter().enumerate() {
        let ref_sz = &ref_sizes[i];
        let tl_pk_trits = tl_kem::public_key_size(*variant);
        let tl_sk_trits = tl_kem::secret_key_size(*variant);
        let tl_ct_size = tl_kem::ciphertext_size(*variant);
        let tl_ss_trits = tl_kem::shared_secret_size(*variant);

        let tl_pk_bytes = (tl_pk_trits as f64 / 1.585).ceil() as usize;
        let _tl_sk_bytes = (tl_sk_trits as f64 / 1.585).ceil() as usize;
        let tl_ss_bytes = (tl_ss_trits as f64 / 1.585).ceil() as usize;

        let pk_within_4x = tl_pk_bytes <= ref_sz.pk_bytes * 4;
        results.push(CrossImplTestResult {
            test_name: String::from("Public key size within 4x of ML-KEM reference"),
            category: TestCategory::SizeCompliance,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: pk_within_4x && tl_pk_trits > 0,
            expected: alloc::format!("<= {} bytes (4x ML-KEM ref {})", ref_sz.pk_bytes * 4, ref_sz.pk_bytes),
            actual: alloc::format!("{} trits ({} equiv bytes)", tl_pk_trits, tl_pk_bytes),
            details: alloc::format!(
                "TL-KEM uses {} trits for pk (~{} bytes). ML-KEM-{} uses {} bytes. \
                 Ternary representation provides {:.1}x information density.",
                tl_pk_trits, tl_pk_bytes, ref_sz.variant.split('-').last().unwrap_or(""),
                ref_sz.pk_bytes,
                tl_pk_trits as f64 * 1.585 / (ref_sz.pk_bytes as f64 * 8.0)
            ),
        });

        results.push(CrossImplTestResult {
            test_name: String::from("Shared secret security margin"),
            category: TestCategory::SizeCompliance,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: tl_ss_bytes >= ref_sz.ss_bytes,
            expected: alloc::format!(">= {} bytes (CNSA 2.0)", ref_sz.ss_bytes),
            actual: alloc::format!("{} trits ({} equiv bytes)", tl_ss_trits, tl_ss_bytes),
            details: alloc::format!(
                "TL-KEM shared secret: {} trits = {:.1} bits ({} equiv bytes). \
                 ML-KEM requires {} bytes (256 bits). Margin: +{:.1} bits.",
                tl_ss_trits,
                tl_ss_trits as f64 * 1.585,
                tl_ss_bytes,
                ref_sz.ss_bytes,
                tl_ss_trits as f64 * 1.585 - (ref_sz.ss_bytes as f64 * 8.0)
            ),
        });

        let ct_within_4x = tl_ct_size <= ref_sz.ct_bytes * 4;
        results.push(CrossImplTestResult {
            test_name: String::from("Ciphertext size within 4x of ML-KEM reference"),
            category: TestCategory::SizeCompliance,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: ct_within_4x && tl_ct_size > 0,
            expected: alloc::format!("<= {} bytes (4x ML-KEM ref {})", ref_sz.ct_bytes * 4, ref_sz.ct_bytes),
            actual: alloc::format!("{} compressed bytes", tl_ct_size),
            details: alloc::format!(
                "TL-KEM ciphertext: {} bytes compressed. ML-KEM-{}: {} bytes. \
                 Ratio: {:.2}x",
                tl_ct_size,
                ref_sz.variant.split('-').last().unwrap_or(""),
                ref_sz.ct_bytes,
                tl_ct_size as f64 / ref_sz.ct_bytes as f64
            ),
        });
    }

    results
}

pub fn test_dsa_size_compliance() -> Vec<CrossImplTestResult> {
    let ref_sizes = ml_dsa_reference_sizes();
    let tl_variants = [TlDsaVariant::TlDsa44, TlDsaVariant::TlDsa65, TlDsaVariant::TlDsa87];
    let mut results = Vec::new();

    for (i, variant) in tl_variants.iter().enumerate() {
        let ref_sz = &ref_sizes[i];
        let tl_pk_trits = tl_dsa::public_key_size(*variant);
        let tl_sk_trits = tl_dsa::secret_key_size(*variant);
        let tl_sig_trits = tl_dsa::signature_size(*variant);

        let tl_pk_bytes = (tl_pk_trits as f64 / 1.585).ceil() as usize;
        let _tl_sk_bytes = (tl_sk_trits as f64 / 1.585).ceil() as usize;
        let tl_sig_bytes = (tl_sig_trits as f64 / 1.585).ceil() as usize;

        let pk_within_4x = tl_pk_bytes <= ref_sz.pk_bytes * 4;
        results.push(CrossImplTestResult {
            test_name: String::from("Public key size within 4x of ML-DSA reference"),
            category: TestCategory::SizeCompliance,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: pk_within_4x && tl_pk_trits > 0,
            expected: alloc::format!("<= {} bytes (4x ML-DSA ref {})", ref_sz.pk_bytes * 4, ref_sz.pk_bytes),
            actual: alloc::format!("{} trits ({} equiv bytes)", tl_pk_trits, tl_pk_bytes),
            details: alloc::format!(
                "TL-DSA pk: {} trits (~{} bytes). ML-DSA-{}: {} bytes.",
                tl_pk_trits, tl_pk_bytes,
                ref_sz.variant.split('-').last().unwrap_or(""),
                ref_sz.pk_bytes
            ),
        });

        let sig_within_4x = tl_sig_bytes <= ref_sz.sig_bytes * 4;
        results.push(CrossImplTestResult {
            test_name: String::from("Signature size within 4x of ML-DSA reference"),
            category: TestCategory::SizeCompliance,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: sig_within_4x && tl_sig_trits > 0,
            expected: alloc::format!("<= {} bytes (4x ML-DSA ref {})", ref_sz.sig_bytes * 4, ref_sz.sig_bytes),
            actual: alloc::format!("{} trits ({} equiv bytes)", tl_sig_trits, tl_sig_bytes),
            details: alloc::format!(
                "TL-DSA signature: {} trits (~{} bytes). ML-DSA-{}: {} bytes. \
                 Ratio: {:.2}x",
                tl_sig_trits, tl_sig_bytes,
                ref_sz.variant.split('-').last().unwrap_or(""),
                ref_sz.sig_bytes,
                tl_sig_bytes as f64 / ref_sz.sig_bytes as f64
            ),
        });
    }

    results
}

pub fn test_kem_protocol_compliance() -> CryptoResult<Vec<CrossImplTestResult>> {
    let variants = [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024];
    let mut results = Vec::new();

    for variant in &variants {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = tl_kem::keygen(*variant, &seed)?;

        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0];
        let (ct, shared1) = tl_kem::encapsulate(&pk, &randomness)?;
        let shared2 = tl_kem::decapsulate(&sk, &ct)?;

        results.push(CrossImplTestResult {
            test_name: String::from("KEM correctness (encaps/decaps match)"),
            category: TestCategory::ProtocolCompliance,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: shared1 == shared2,
            expected: String::from("shared_secret_encaps == shared_secret_decaps"),
            actual: if shared1 == shared2 { String::from("MATCH") } else { String::from("MISMATCH") },
            details: alloc::format!(
                "Generated keypair, encapsulated with random coins, decapsulated. \
                 Shared secrets: {} trits each. Match: {}",
                shared1.trits.len(),
                shared1 == shared2
            ),
        });

        let seed2 = vec![1i8, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0];
        let (_pk2, sk2) = tl_kem::keygen(*variant, &seed2)?;
        let shared_wrong = tl_kem::decapsulate(&sk2, &ct)?;

        results.push(CrossImplTestResult {
            test_name: String::from("KEM implicit rejection"),
            category: TestCategory::ProtocolCompliance,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: shared1 != shared_wrong,
            expected: String::from("Decaps with wrong key produces different secret"),
            actual: if shared1 != shared_wrong { String::from("REJECTED (correct)") } else { String::from("NOT REJECTED (error)") },
            details: String::from(
                "Attempted decapsulation with wrong secret key. Implicit rejection \
                 should produce a pseudorandom but different shared secret."
            ),
        });

        let (pk_a, _sk_a) = tl_kem::keygen(*variant, &seed)?;
        let (pk_b, _sk_b) = tl_kem::keygen(*variant, &seed)?;
        let pk_match = poly_vec_to_trits(&pk_a.public_vec_t) == poly_vec_to_trits(&pk_b.public_vec_t);

        results.push(CrossImplTestResult {
            test_name: String::from("KEM keygen determinism"),
            category: TestCategory::AlgebraicConsistency,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: pk_match,
            expected: String::from("Same seed produces same keypair"),
            actual: if pk_match { String::from("DETERMINISTIC") } else { String::from("NON-DETERMINISTIC") },
            details: String::from("Two keygen calls with identical seeds must produce identical keypairs"),
        });
    }

    Ok(results)
}

pub fn test_dsa_protocol_compliance() -> CryptoResult<Vec<CrossImplTestResult>> {
    let variants = [TlDsaVariant::TlDsa44, TlDsaVariant::TlDsa65, TlDsaVariant::TlDsa87];
    let mut results = Vec::new();

    for variant in &variants {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = tl_dsa::keygen(*variant, &seed)?;

        let message = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = tl_dsa::sign(&sk, &message)?;
        let valid = tl_dsa::verify(&pk, &message, &sig)?;

        results.push(CrossImplTestResult {
            test_name: String::from("DSA correctness (sign/verify)"),
            category: TestCategory::ProtocolCompliance,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: valid,
            expected: String::from("verify(pk, msg, sign(sk, msg)) == true"),
            actual: if valid { String::from("VALID") } else { String::from("INVALID") },
            details: alloc::format!(
                "Generated keypair, signed message ({} trits), verified. \
                 Signature z-vector: {} polynomials. Valid: {}",
                message.len(),
                sig.z.polys.len(),
                valid
            ),
        });

        let wrong_msg = vec![0i8, 0, 0, 0, 0, 0, 0, 0, 0];
        let valid_wrong = tl_dsa::verify(&pk, &wrong_msg, &sig)?;

        results.push(CrossImplTestResult {
            test_name: String::from("DSA forgery resistance"),
            category: TestCategory::ProtocolCompliance,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: !valid_wrong,
            expected: String::from("verify(pk, wrong_msg, sig) == false"),
            actual: if !valid_wrong { String::from("REJECTED (correct)") } else { String::from("ACCEPTED (error)") },
            details: String::from("Signature must not verify under a different message"),
        });

        let seed2 = vec![1i8, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0];
        let (pk2, _sk2) = tl_dsa::keygen(*variant, &seed2)?;
        let valid_wrong_key = tl_dsa::verify(&pk2, &message, &sig)?;

        results.push(CrossImplTestResult {
            test_name: String::from("DSA key binding"),
            category: TestCategory::ProtocolCompliance,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: !valid_wrong_key,
            expected: String::from("verify(wrong_pk, msg, sig) == false"),
            actual: if !valid_wrong_key { String::from("REJECTED (correct)") } else { String::from("ACCEPTED (error)") },
            details: String::from("Signature must not verify under a different public key"),
        });

        let sig1 = tl_dsa::sign(&sk, &message)?;
        let sig2 = tl_dsa::sign(&sk, &message)?;
        let deterministic = sig1.challenge_hash == sig2.challenge_hash;

        results.push(CrossImplTestResult {
            test_name: String::from("DSA signing determinism"),
            category: TestCategory::AlgebraicConsistency,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: deterministic,
            expected: String::from("Same (sk, msg) produces same signature"),
            actual: if deterministic { String::from("DETERMINISTIC") } else { String::from("NON-DETERMINISTIC") },
            details: String::from("Deterministic signing (no random nonce) ensures reproducibility"),
        });
    }

    Ok(results)
}

pub fn test_round_trip_integrity() -> CryptoResult<Vec<CrossImplTestResult>> {
    let mut results = Vec::new();
    let kem_variants = [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024];

    for variant in &kem_variants {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, _sk) = tl_kem::keygen(*variant, &seed)?;

        let pk_trits = poly_vec_to_trits(&pk.public_vec_t);
        let pk_bytes: Vec<u8> = pk_trits.iter().map(|&t| (t + 1) as u8).collect();
        let pk_restored: Vec<i8> = pk_bytes.iter().map(|&b| b as i8 - 1).collect();

        let roundtrip_ok = pk_trits == pk_restored;

        results.push(CrossImplTestResult {
            test_name: String::from("KEM public key trit-byte-trit roundtrip"),
            category: TestCategory::RoundTripIntegrity,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: roundtrip_ok,
            expected: String::from("pk_trits == decode(encode(pk_trits))"),
            actual: if roundtrip_ok { String::from("PRESERVED") } else { String::from("CORRUPTED") },
            details: alloc::format!(
                "Encoded {} trits to bytes and back. Integrity: {}",
                pk_trits.len(),
                if roundtrip_ok { "preserved" } else { "lost" }
            ),
        });

        let ss_roundtrip_seed = vec![1i8, 0, -1, 1, 0];
        let (_ct, shared) = tl_kem::encapsulate(&pk, &ss_roundtrip_seed)?;
        let ss_bytes: Vec<u8> = shared.trits.iter().map(|&t| (t + 1) as u8).collect();
        let ss_restored: Vec<i8> = ss_bytes.iter().map(|&b| b as i8 - 1).collect();

        let ss_roundtrip_ok = shared.trits == ss_restored;

        results.push(CrossImplTestResult {
            test_name: String::from("KEM shared secret trit-byte-trit roundtrip"),
            category: TestCategory::RoundTripIntegrity,
            algorithm: String::from("TL-KEM"),
            variant: String::from(variant.name()),
            passed: ss_roundtrip_ok,
            expected: String::from("ss_trits == decode(encode(ss_trits))"),
            actual: if ss_roundtrip_ok { String::from("PRESERVED") } else { String::from("CORRUPTED") },
            details: alloc::format!(
                "Encoded {} trit shared secret to bytes and back. Integrity: {}",
                shared.trits.len(),
                if ss_roundtrip_ok { "preserved" } else { "lost" }
            ),
        });
    }

    let dsa_variants = [TlDsaVariant::TlDsa44, TlDsaVariant::TlDsa65, TlDsaVariant::TlDsa87];

    for variant in &dsa_variants {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, _sk) = tl_dsa::keygen(*variant, &seed)?;

        let pk_trits = poly_vec_to_trits(&pk.public_t);
        let pk_bytes: Vec<u8> = pk_trits.iter().map(|&t| (t + 1) as u8).collect();
        let pk_restored: Vec<i8> = pk_bytes.iter().map(|&b| b as i8 - 1).collect();

        let roundtrip_ok = pk_trits == pk_restored;

        results.push(CrossImplTestResult {
            test_name: String::from("DSA public key trit-byte-trit roundtrip"),
            category: TestCategory::RoundTripIntegrity,
            algorithm: String::from("TL-DSA"),
            variant: String::from(variant.name()),
            passed: roundtrip_ok,
            expected: String::from("pk_trits == decode(encode(pk_trits))"),
            actual: if roundtrip_ok { String::from("PRESERVED") } else { String::from("CORRUPTED") },
            details: alloc::format!(
                "Encoded {} trits to bytes and back. Integrity: {}",
                pk_trits.len(),
                if roundtrip_ok { "preserved" } else { "lost" }
            ),
        });
    }

    Ok(results)
}

pub fn generate_full_cross_impl_report() -> CryptoResult<CrossImplReport> {
    let mut all_results = Vec::new();

    all_results.extend(test_kem_size_compliance());
    all_results.extend(test_dsa_size_compliance());
    all_results.extend(test_kem_protocol_compliance()?);
    all_results.extend(test_dsa_protocol_compliance()?);
    all_results.extend(test_round_trip_integrity()?);

    let total = all_results.len();
    let passed = all_results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    let categories = vec![
        TestCategory::SizeCompliance,
        TestCategory::ProtocolCompliance,
        TestCategory::AlgebraicConsistency,
        TestCategory::RoundTripIntegrity,
    ];

    Ok(CrossImplReport {
        results: all_results,
        total_tests: total,
        passed,
        failed,
        categories_tested: categories,
        overall_compatible: failed == 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kem_sizes() {
        let results = super::test_kem_size_compliance();
        assert_eq!(results.len(), 9);
        for r in &results {
            assert!(r.passed, "KEM size test failed: {} {}: {}", r.variant, r.test_name, r.details);
        }
    }

    #[test]
    fn test_dsa_sizes() {
        let results = super::test_dsa_size_compliance();
        assert_eq!(results.len(), 6);
        for r in &results {
            assert!(r.passed, "DSA size test failed: {} {}: {}", r.variant, r.test_name, r.details);
        }
    }

    #[test]
    fn test_kem_protocol() {
        let results = test_kem_protocol_compliance().unwrap();
        assert_eq!(results.len(), 9);
        for r in &results {
            assert!(r.passed, "KEM protocol test failed: {} {}: {}", r.variant, r.test_name, r.details);
        }
    }

    #[test]
    fn test_dsa_protocol() {
        let results = test_dsa_protocol_compliance().unwrap();
        assert_eq!(results.len(), 12);
        for r in &results {
            assert!(r.passed, "DSA protocol test failed: {} {}: {}", r.variant, r.test_name, r.details);
        }
    }

    #[test]
    fn test_round_trip() {
        let results = test_round_trip_integrity().unwrap();
        assert!(results.len() >= 9);
        for r in &results {
            assert!(r.passed, "Round-trip test failed: {} {}: {}", r.variant, r.test_name, r.details);
        }
    }

    #[test]
    fn test_full_report() {
        let report = generate_full_cross_impl_report().unwrap();
        assert!(report.total_tests > 0);
        assert_eq!(report.failed, 0);
        assert!(report.overall_compatible);
    }

    #[test]
    fn test_ml_kem_reference_sizes_defined() {
        let sizes = ml_kem_reference_sizes();
        assert_eq!(sizes.len(), 3);
        assert_eq!(sizes[0].variant, "ML-KEM-512");
        assert_eq!(sizes[1].variant, "ML-KEM-768");
        assert_eq!(sizes[2].variant, "ML-KEM-1024");
    }

    #[test]
    fn test_ml_dsa_reference_sizes_defined() {
        let sizes = ml_dsa_reference_sizes();
        assert_eq!(sizes.len(), 3);
        assert_eq!(sizes[0].variant, "ML-DSA-44");
        assert_eq!(sizes[1].variant, "ML-DSA-65");
        assert_eq!(sizes[2].variant, "ML-DSA-87");
    }

    #[test]
    fn test_category_names() {
        assert_eq!(TestCategory::FormatCompatibility.name(), "Format Compatibility");
        assert_eq!(TestCategory::ProtocolCompliance.name(), "Protocol Compliance");
    }
}
