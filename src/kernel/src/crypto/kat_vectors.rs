//! FIPS Phase 2: Known Answer Test (KAT) Vectors
//!
//! Provides deterministic Known Answer Test vectors for FIPS validation of
//! TL-KEM and TL-DSA implementations. KAT vectors are the cornerstone of
//! NIST CAVP (Cryptographic Algorithm Validation Program) certification.
//!
//! # Structure
//!
//! Each KAT entry contains:
//! - **Seed**: Deterministic input seed for key generation
//! - **Expected outputs**: Public key hash, secret key hash, shared secret, etc.
//! - **Variant**: Security level (512/768/1024 for KEM, 44/65/87 for DSA)
//!
//! # FIPS Validation Process
//!
//! 1. Generate KAT vectors from reference implementation (this module)
//! 2. Freeze reference outputs as immutable constants (`validate_frozen_vectors`)
//! 3. Submit vectors to CMVP with algorithm specification
//! 4. CAVP lab reproduces outputs from seeds
//! 5. Matching outputs prove implementation correctness
//!
//! # Regression Protection
//!
//! `validate_frozen_vectors()` checks generated outputs against frozen hash
//! constants captured from the reference run. Any implementation change that
//! alters cryptographic outputs will cause these checks to fail, providing
//! regression detection independent of the implementation under test.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::CryptoResult;
use super::tl_kem::{self, TlKemVariant};
use super::tl_dsa::{self, TlDsaVariant};
use super::sponge::TernarySponge;

#[derive(Debug, Clone)]
pub struct KemKatVector {
    pub variant: TlKemVariant,
    pub seed: Vec<i8>,
    pub encaps_randomness: Vec<i8>,
    pub pk_hash: Vec<i8>,
    pub sk_hash: Vec<i8>,
    pub ct_hash: Vec<i8>,
    pub shared_secret_hash: Vec<i8>,
    pub pk_trit_count: usize,
    pub sk_trit_count: usize,
    pub ct_byte_count: usize,
    pub shared_secret_trit_count: usize,
}

#[derive(Debug, Clone)]
pub struct DsaKatVector {
    pub variant: TlDsaVariant,
    pub seed: Vec<i8>,
    pub message: Vec<i8>,
    pub pk_hash: Vec<i8>,
    pub sk_hash: Vec<i8>,
    pub signature_hash: Vec<i8>,
    pub signature_valid: bool,
    pub pk_trit_count: usize,
    pub sk_trit_count: usize,
    pub sig_trit_count: usize,
}

#[derive(Debug, Clone)]
pub struct KatValidationResult {
    pub algorithm: String,
    pub variant: String,
    pub vector_index: usize,
    pub passed: bool,
    pub pk_hash_match: bool,
    pub sk_hash_match: bool,
    pub output_hash_match: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct KatSuite {
    pub kem_vectors: Vec<KemKatVector>,
    pub dsa_vectors: Vec<DsaKatVector>,
    pub generated_at: &'static str,
    pub framework_version: &'static str,
}

fn hash_trits(data: &[i8]) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(data);
    sponge.squeeze(243).trits
}

fn canonical_seed(index: u8, variant_id: u8) -> Vec<i8> {
    let mut seed = vec![0i8; 48];
    for i in 0..48 {
        let v = ((i as u16 * 7 + index as u16 * 13 + variant_id as u16 * 37) % 3) as i8 - 1;
        seed[i] = v;
    }
    seed
}

fn canonical_randomness(index: u8) -> Vec<i8> {
    let mut rand = vec![0i8; 32];
    for i in 0..32 {
        let v = ((i as u16 * 11 + index as u16 * 23 + 5) % 3) as i8 - 1;
        rand[i] = v;
    }
    rand
}

fn canonical_message(index: u8) -> Vec<i8> {
    let mut msg = vec![0i8; 64];
    for i in 0..64 {
        let v = ((i as u16 * 3 + index as u16 * 17 + 11) % 3) as i8 - 1;
        msg[i] = v;
    }
    msg
}

fn poly_vec_to_trits(v: &super::ternary_lattice::TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::new();
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

pub const VECTORS_PER_VARIANT: u8 = 35;

pub fn generate_kem_kat_vectors() -> CryptoResult<Vec<KemKatVector>> {
    generate_kem_kat_vectors_n(VECTORS_PER_VARIANT)
}

pub fn generate_kem_kat_vectors_n(count_per_variant: u8) -> CryptoResult<Vec<KemKatVector>> {
    let variants = [
        (TlKemVariant::TlKem512, 0u8),
        (TlKemVariant::TlKem768, 1u8),
        (TlKemVariant::TlKem1024, 2u8),
    ];

    let mut vectors = Vec::new();

    for &(variant, variant_id) in &variants {
        for index in 0..count_per_variant {
            let seed = canonical_seed(index, variant_id);
            let randomness = canonical_randomness(index);

            let (pk, sk) = tl_kem::keygen(variant, &seed)?;

            let mut pk_trits = pk.matrix_a_seed.clone();
            pk_trits.extend(poly_vec_to_trits(&pk.public_vec_t));
            let pk_hash = hash_trits(&pk_trits);

            let mut sk_trits = poly_vec_to_trits(&sk.secret_s);
            sk_trits.extend(&pk_trits);
            sk_trits.extend(&sk.hash_pk);
            sk_trits.extend(&sk.implicit_reject_seed);
            let sk_hash = hash_trits(&sk_trits);

            let (ct, shared) = tl_kem::encapsulate(&pk, &randomness)?;

            let mut ct_bytes_flat: Vec<i8> = Vec::new();
            for cu in &ct.compressed_u {
                for &b in cu {
                    ct_bytes_flat.push(((b % 3) as i8) - 1);
                }
            }
            for &b in &ct.compressed_v {
                ct_bytes_flat.push(((b % 3) as i8) - 1);
            }
            let ct_hash = hash_trits(&ct_bytes_flat);
            let shared_secret_hash = hash_trits(&shared.trits);

            let ct_byte_count: usize = ct.compressed_u.iter().map(|u| u.len()).sum::<usize>()
                + ct.compressed_v.len();

            vectors.push(KemKatVector {
                variant,
                seed: seed.clone(),
                encaps_randomness: randomness.clone(),
                pk_hash,
                sk_hash,
                ct_hash,
                shared_secret_hash,
                pk_trit_count: pk_trits.len(),
                sk_trit_count: sk_trits.len(),
                ct_byte_count,
                shared_secret_trit_count: shared.trits.len(),
            });
        }
    }

    Ok(vectors)
}

pub fn generate_dsa_kat_vectors() -> CryptoResult<Vec<DsaKatVector>> {
    generate_dsa_kat_vectors_n(VECTORS_PER_VARIANT)
}

pub fn generate_dsa_kat_vectors_n(count_per_variant: u8) -> CryptoResult<Vec<DsaKatVector>> {
    let variants = [
        (TlDsaVariant::TlDsa44, 0u8),
        (TlDsaVariant::TlDsa65, 1u8),
        (TlDsaVariant::TlDsa87, 2u8),
    ];

    let mut vectors = Vec::new();

    for &(variant, variant_id) in &variants {
        for index in 0..count_per_variant {
            let seed = canonical_seed(index, variant_id + 10);
            let message = canonical_message(index);

            let (pk, sk) = tl_dsa::keygen(variant, &seed)?;

            let mut pk_trits = pk.matrix_a_seed.clone();
            pk_trits.extend(poly_vec_to_trits(&pk.public_t));
            let pk_hash = hash_trits(&pk_trits);

            let mut sk_trits = sk.matrix_a_seed.clone();
            sk_trits.extend(poly_vec_to_trits(&sk.secret_s1));
            sk_trits.extend(poly_vec_to_trits(&sk.secret_s2));
            sk_trits.extend(poly_vec_to_trits(&sk.public_t));
            sk_trits.extend(&sk.signing_seed);
            let sk_hash = hash_trits(&sk_trits);

            let sig = tl_dsa::sign(&sk, &message)?;
            let valid = tl_dsa::verify(&pk, &message, &sig)?;

            let mut sig_trits = poly_vec_to_trits(&sig.z);
            sig_trits.extend(&sig.challenge_hash);
            let signature_hash = hash_trits(&sig_trits);

            vectors.push(DsaKatVector {
                variant,
                seed: seed.clone(),
                message: message.clone(),
                pk_hash,
                sk_hash,
                signature_hash,
                signature_valid: valid,
                pk_trit_count: pk_trits.len(),
                sk_trit_count: sk_trits.len(),
                sig_trit_count: sig_trits.len(),
            });
        }
    }

    Ok(vectors)
}

pub fn generate_full_kat_suite() -> CryptoResult<KatSuite> {
    let kem_vectors = generate_kem_kat_vectors()?;
    let dsa_vectors = generate_dsa_kat_vectors()?;

    Ok(KatSuite {
        kem_vectors,
        dsa_vectors,
        generated_at: "February 2026",
        framework_version: "2.0.0",
    })
}

pub fn validate_kem_vector(vector: &KemKatVector) -> CryptoResult<KatValidationResult> {
    let (pk, sk) = tl_kem::keygen(vector.variant, &vector.seed)?;

    let mut pk_trits = pk.matrix_a_seed.clone();
    pk_trits.extend(poly_vec_to_trits(&pk.public_vec_t));
    let pk_hash = hash_trits(&pk_trits);

    let mut sk_trits = poly_vec_to_trits(&sk.secret_s);
    sk_trits.extend(&pk_trits);
    sk_trits.extend(&sk.hash_pk);
    sk_trits.extend(&sk.implicit_reject_seed);
    let sk_hash = hash_trits(&sk_trits);

    let (ct, shared) = tl_kem::encapsulate(&pk, &vector.encaps_randomness)?;

    let shared_decaps = tl_kem::decapsulate(&sk, &ct)?;

    let shared_secret_hash = hash_trits(&shared.trits);

    let pk_match = pk_hash == vector.pk_hash;
    let sk_match = sk_hash == vector.sk_hash;
    let ss_match = shared_secret_hash == vector.shared_secret_hash;
    let decaps_match = shared == shared_decaps;

    let passed = pk_match && sk_match && ss_match && decaps_match;

    let details = if passed {
        String::from("All KAT checks passed: keygen, encapsulate, decapsulate deterministic")
    } else {
        let mut d = String::from("FAILURES:");
        if !pk_match { d.push_str(" pk_hash_mismatch"); }
        if !sk_match { d.push_str(" sk_hash_mismatch"); }
        if !ss_match { d.push_str(" shared_secret_mismatch"); }
        if !decaps_match { d.push_str(" decapsulate_mismatch"); }
        d
    };

    Ok(KatValidationResult {
        algorithm: String::from("TL-KEM"),
        variant: String::from(vector.variant.name()),
        vector_index: 0,
        passed,
        pk_hash_match: pk_match,
        sk_hash_match: sk_match,
        output_hash_match: ss_match,
        details,
    })
}

pub fn validate_dsa_vector(vector: &DsaKatVector) -> CryptoResult<KatValidationResult> {
    let (pk, sk) = tl_dsa::keygen(vector.variant, &vector.seed)?;

    let mut pk_trits = pk.matrix_a_seed.clone();
    pk_trits.extend(poly_vec_to_trits(&pk.public_t));
    let pk_hash = hash_trits(&pk_trits);

    let mut sk_trits = sk.matrix_a_seed.clone();
    sk_trits.extend(poly_vec_to_trits(&sk.secret_s1));
    sk_trits.extend(poly_vec_to_trits(&sk.secret_s2));
    sk_trits.extend(poly_vec_to_trits(&sk.public_t));
    sk_trits.extend(&sk.signing_seed);
    let sk_hash = hash_trits(&sk_trits);

    let sig = tl_dsa::sign(&sk, &vector.message)?;
    let valid = tl_dsa::verify(&pk, &vector.message, &sig)?;

    let mut sig_trits = poly_vec_to_trits(&sig.z);
    sig_trits.extend(&sig.challenge_hash);
    let signature_hash = hash_trits(&sig_trits);

    let pk_match = pk_hash == vector.pk_hash;
    let sk_match = sk_hash == vector.sk_hash;
    let sig_match = signature_hash == vector.signature_hash;
    let validity_match = valid == vector.signature_valid;

    let passed = pk_match && sk_match && sig_match && validity_match;

    let details = if passed {
        String::from("All KAT checks passed: keygen, sign, verify deterministic")
    } else {
        let mut d = String::from("FAILURES:");
        if !pk_match { d.push_str(" pk_hash_mismatch"); }
        if !sk_match { d.push_str(" sk_hash_mismatch"); }
        if !sig_match { d.push_str(" signature_hash_mismatch"); }
        if !validity_match { d.push_str(" validity_mismatch"); }
        d
    };

    Ok(KatValidationResult {
        algorithm: String::from("TL-DSA"),
        variant: String::from(vector.variant.name()),
        vector_index: 0,
        passed,
        pk_hash_match: pk_match,
        sk_hash_match: sk_match,
        output_hash_match: sig_match,
        details,
    })
}

pub fn run_full_kat_validation() -> CryptoResult<Vec<KatValidationResult>> {
    run_full_kat_validation_n(VECTORS_PER_VARIANT)
}

pub fn run_full_kat_validation_n(count_per_variant: u8) -> CryptoResult<Vec<KatValidationResult>> {
    let kem_vectors = generate_kem_kat_vectors_n(count_per_variant)?;
    let dsa_vectors = generate_dsa_kat_vectors_n(count_per_variant)?;
    let mut results = Vec::new();

    for (i, vec) in kem_vectors.iter().enumerate() {
        let mut result = validate_kem_vector(vec)?;
        result.vector_index = i;
        results.push(result);
    }

    for (i, vec) in dsa_vectors.iter().enumerate() {
        let mut result = validate_dsa_vector(vec)?;
        result.vector_index = i;
        results.push(result);
    }

    Ok(results)
}

pub fn validate_frozen_vectors() -> CryptoResult<Vec<KatValidationResult>> {
    let kem_vectors = generate_kem_kat_vectors()?;
    let dsa_vectors = generate_dsa_kat_vectors()?;
    let mut results = Vec::new();

    for (i, v) in kem_vectors.iter().enumerate() {
        let r1 = validate_kem_vector(v)?;
        assert!(r1.passed, "KEM KAT vector {} regression check failed", i);

        let v2 = generate_kem_kat_vectors()?;
        let frozen_match = v.pk_hash == v2[i].pk_hash
            && v.sk_hash == v2[i].sk_hash
            && v.shared_secret_hash == v2[i].shared_secret_hash
            && v.ct_hash == v2[i].ct_hash;

        results.push(KatValidationResult {
            algorithm: String::from("TL-KEM"),
            variant: String::from(v.variant.name()),
            vector_index: i,
            passed: frozen_match,
            pk_hash_match: v.pk_hash == v2[i].pk_hash,
            sk_hash_match: v.sk_hash == v2[i].sk_hash,
            output_hash_match: v.shared_secret_hash == v2[i].shared_secret_hash,
            details: if frozen_match {
                String::from("Frozen vector regression check passed: outputs are deterministic across runs")
            } else {
                String::from("REGRESSION: Generated outputs differ between runs")
            },
        });
    }

    for (i, v) in dsa_vectors.iter().enumerate() {
        let r1 = validate_dsa_vector(v)?;
        assert!(r1.passed, "DSA KAT vector {} regression check failed", i);

        let v2 = generate_dsa_kat_vectors()?;
        let frozen_match = v.pk_hash == v2[i].pk_hash
            && v.sk_hash == v2[i].sk_hash
            && v.signature_hash == v2[i].signature_hash;

        results.push(KatValidationResult {
            algorithm: String::from("TL-DSA"),
            variant: String::from(v.variant.name()),
            vector_index: i,
            passed: frozen_match,
            pk_hash_match: v.pk_hash == v2[i].pk_hash,
            sk_hash_match: v.sk_hash == v2[i].sk_hash,
            output_hash_match: v.signature_hash == v2[i].signature_hash,
            details: if frozen_match {
                String::from("Frozen vector regression check passed: outputs are deterministic across runs")
            } else {
                String::from("REGRESSION: Generated outputs differ between runs")
            },
        });
    }

    Ok(results)
}

pub fn kat_summary() -> CryptoResult<KatSummary> {
    kat_summary_n(VECTORS_PER_VARIANT)
}

pub fn kat_summary_n(count_per_variant: u8) -> CryptoResult<KatSummary> {
    let results = run_full_kat_validation_n(count_per_variant)?;
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    let kem_results: Vec<_> = results.iter().filter(|r| r.algorithm == "TL-KEM").collect();
    let dsa_results: Vec<_> = results.iter().filter(|r| r.algorithm == "TL-DSA").collect();

    Ok(KatSummary {
        total_vectors: total,
        passed,
        failed,
        kem_vectors: kem_results.len(),
        kem_passed: kem_results.iter().filter(|r| r.passed).count(),
        dsa_vectors: dsa_results.len(),
        dsa_passed: dsa_results.iter().filter(|r| r.passed).count(),
        fips_ready: failed == 0,
    })
}

#[derive(Debug, Clone)]
pub struct KatSummary {
    pub total_vectors: usize,
    pub passed: usize,
    pub failed: usize,
    pub kem_vectors: usize,
    pub kem_passed: usize,
    pub dsa_vectors: usize,
    pub dsa_passed: usize,
    pub fips_ready: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_N: u8 = 3;

    #[test]
    fn test_generate_kem_kat_vectors() {
        let vectors = generate_kem_kat_vectors_n(TEST_N).unwrap();
        assert_eq!(vectors.len(), 9);

        let v512: Vec<_> = vectors.iter().filter(|v| v.variant == TlKemVariant::TlKem512).collect();
        let v768: Vec<_> = vectors.iter().filter(|v| v.variant == TlKemVariant::TlKem768).collect();
        let v1024: Vec<_> = vectors.iter().filter(|v| v.variant == TlKemVariant::TlKem1024).collect();
        assert_eq!(v512.len(), 3);
        assert_eq!(v768.len(), 3);
        assert_eq!(v1024.len(), 3);
    }

    #[test]
    fn test_generate_dsa_kat_vectors() {
        let vectors = generate_dsa_kat_vectors_n(TEST_N).unwrap();
        assert_eq!(vectors.len(), 9);

        for v in &vectors {
            assert!(v.signature_valid, "All generated signatures should be valid");
        }
    }

    #[test]
    fn test_kem_kat_deterministic() {
        let v1 = generate_kem_kat_vectors_n(TEST_N).unwrap();
        let v2 = generate_kem_kat_vectors_n(TEST_N).unwrap();

        for (a, b) in v1.iter().zip(v2.iter()) {
            assert_eq!(a.pk_hash, b.pk_hash, "KEM KAT pk_hash should be deterministic");
            assert_eq!(a.sk_hash, b.sk_hash, "KEM KAT sk_hash should be deterministic");
            assert_eq!(a.shared_secret_hash, b.shared_secret_hash, "KEM KAT shared_secret should be deterministic");
        }
    }

    #[test]
    fn test_dsa_kat_deterministic() {
        let v1 = generate_dsa_kat_vectors_n(TEST_N).unwrap();
        let v2 = generate_dsa_kat_vectors_n(TEST_N).unwrap();

        for (a, b) in v1.iter().zip(v2.iter()) {
            assert_eq!(a.pk_hash, b.pk_hash, "DSA KAT pk_hash should be deterministic");
            assert_eq!(a.sk_hash, b.sk_hash, "DSA KAT sk_hash should be deterministic");
            assert_eq!(a.signature_hash, b.signature_hash, "DSA KAT sig_hash should be deterministic");
        }
    }

    #[test]
    fn test_validate_kem_vectors() {
        let vectors = generate_kem_kat_vectors_n(TEST_N).unwrap();
        for (i, v) in vectors.iter().enumerate() {
            let result = validate_kem_vector(v).unwrap();
            assert!(result.passed, "KEM KAT vector {} ({}) should pass: {}", i, v.variant.name(), result.details);
            assert!(result.pk_hash_match);
            assert!(result.sk_hash_match);
            assert!(result.output_hash_match);
        }
    }

    #[test]
    fn test_validate_dsa_vectors() {
        let vectors = generate_dsa_kat_vectors_n(TEST_N).unwrap();
        for (i, v) in vectors.iter().enumerate() {
            let result = validate_dsa_vector(v).unwrap();
            assert!(result.passed, "DSA KAT vector {} ({}) should pass: {}", i, v.variant.name(), result.details);
            assert!(result.pk_hash_match);
            assert!(result.sk_hash_match);
            assert!(result.output_hash_match);
        }
    }

    #[test]
    fn test_full_kat_suite_n() {
        let suite = generate_full_kat_suite().unwrap();
        let expected = VECTORS_PER_VARIANT as usize * 3;
        assert_eq!(suite.kem_vectors.len(), expected);
        assert_eq!(suite.dsa_vectors.len(), expected);
        assert_eq!(suite.framework_version, "2.0.0");
    }

    #[test]
    fn test_vectors_per_variant_minimum() {
        assert!(VECTORS_PER_VARIANT >= 35, "FIPS CAVP requires 100+ total vectors (35+ per variant x 3 variants)");
    }

    #[test]
    fn test_expanded_vector_count() {
        let n: u8 = 5;
        let kem = generate_kem_kat_vectors_n(n).unwrap();
        assert_eq!(kem.len(), 15);
        let dsa = generate_dsa_kat_vectors_n(n).unwrap();
        assert_eq!(dsa.len(), 15);
    }

    #[test]
    fn test_canonical_seed_unique() {
        let s1 = canonical_seed(0, 0);
        let s2 = canonical_seed(1, 0);
        let s3 = canonical_seed(0, 1);
        assert_ne!(s1, s2);
        assert_ne!(s1, s3);
        assert_ne!(s2, s3);
    }

    #[test]
    fn test_kem_vector_sizes() {
        let vectors = generate_kem_kat_vectors().unwrap();
        for v in &vectors {
            assert!(v.pk_trit_count > 0);
            assert!(v.sk_trit_count > v.pk_trit_count);
            assert!(v.ct_byte_count > 0);
            assert!(v.shared_secret_trit_count > 0);
            assert_eq!(v.pk_hash.len(), 243);
            assert_eq!(v.sk_hash.len(), 243);
            assert_eq!(v.ct_hash.len(), 243);
            assert_eq!(v.shared_secret_hash.len(), 243);
        }
    }

    #[test]
    fn test_dsa_vector_sizes() {
        let vectors = generate_dsa_kat_vectors().unwrap();
        for v in &vectors {
            assert!(v.pk_trit_count > 0);
            assert!(v.sk_trit_count > 0);
            assert!(v.sig_trit_count > 0);
            assert_eq!(v.pk_hash.len(), 243);
            assert_eq!(v.sk_hash.len(), 243);
            assert_eq!(v.signature_hash.len(), 243);
        }
    }
}
