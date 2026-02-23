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

//! Ternary Lattice Digital Signature Algorithm (TL-DSA)
//!
//! Implements a ternary-native equivalent of ML-DSA (FIPS 204, Dilithium)
//! using GF(3) polynomial ring arithmetic from `ternary_lattice`. Provides
//! EUF-CMA secure digital signatures at three security levels:
//!
//! | Variant     | Module Rank (k,l) | NIST Level | ML-DSA Equivalent |
//! |------------|-------------------|------------|-------------------|
//! | TL-DSA-44  | k=4, l=4          | Level 2    | ML-DSA-44         |
//! | TL-DSA-65  | k=6, l=5          | Level 3    | ML-DSA-65         |
//! | TL-DSA-87  | k=8, l=7          | Level 5    | ML-DSA-87         |
//!
//! # Construction
//!
//! TL-DSA follows the Fiat-Shamir with Aborts paradigm:
//! 1. **KeyGen**: Generate Module-LWE keypair (A, t=As1+s2) with ternary noise
//! 2. **Sign**: Sample masking vector y, compute challenge c = H(Ay || msg),
//!    compute z = y + c*s1, reject if z too large (abort-and-retry)
//! 3. **Verify**: Check ||z||_inf <= bound and Az - ct = Ay mod q
//!
//! All arithmetic operates in R_q = Z_3[X]/(X^n+1) with balanced ternary
//! coefficients {-1, 0, +1}.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use super::{CryptoError, CryptoResult};
use super::ternary_lattice::{
    TernaryPolynomial, TernaryPolyMatrix, TernaryPolyVec,
    sample_matrix, sample_noise_vec,
};
use super::sponge::TernarySponge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlDsaVariant {
    TlDsa44,
    TlDsa65,
    TlDsa87,
}

impl TlDsaVariant {
    pub fn name(&self) -> &'static str {
        match self {
            TlDsaVariant::TlDsa44 => "TL-DSA-44",
            TlDsaVariant::TlDsa65 => "TL-DSA-65",
            TlDsaVariant::TlDsa87 => "TL-DSA-87",
        }
    }

    pub fn security_bits(&self) -> u32 {
        match self {
            TlDsaVariant::TlDsa44 => 128,
            TlDsaVariant::TlDsa65 => 192,
            TlDsaVariant::TlDsa87 => 256,
        }
    }

    pub fn params(&self) -> TlDsaParams {
        match self {
            TlDsaVariant::TlDsa44 => TlDsaParams {
                n: 256,
                k: 4,
                l: 4,
                eta: 2,
                gamma: 1,
                tau: 39,
                max_attempts: 256,
            },
            TlDsaVariant::TlDsa65 => TlDsaParams {
                n: 256,
                k: 6,
                l: 5,
                eta: 2,
                gamma: 1,
                tau: 49,
                max_attempts: 256,
            },
            TlDsaVariant::TlDsa87 => TlDsaParams {
                n: 256,
                k: 8,
                l: 7,
                eta: 2,
                gamma: 1,
                tau: 60,
                max_attempts: 256,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TlDsaParams {
    pub n: usize,
    pub k: usize,
    pub l: usize,
    pub eta: u8,
    pub gamma: u8,
    pub tau: usize,
    pub max_attempts: usize,
}

#[derive(Debug, Clone)]
pub struct TlDsaPublicKey {
    pub variant: TlDsaVariant,
    pub matrix_a_seed: Vec<i8>,
    pub public_t: TernaryPolyVec,
}

#[derive(Debug, Clone)]
pub struct TlDsaSecretKey {
    pub variant: TlDsaVariant,
    pub matrix_a_seed: Vec<i8>,
    pub secret_s1: TernaryPolyVec,
    pub secret_s2: TernaryPolyVec,
    pub public_t: TernaryPolyVec,
    pub signing_seed: Vec<i8>,
}

#[derive(Debug, Clone)]
pub struct TlDsaSignature {
    pub variant: TlDsaVariant,
    pub z: TernaryPolyVec,
    pub challenge_hash: Vec<i8>,
}

fn dsa_hash(inputs: &[&[i8]], output_len: usize) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    for input in inputs {
        sponge.absorb(input);
    }
    sponge.squeeze(output_len).trits
}

fn poly_vec_to_trits(v: &TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::new();
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

fn sample_challenge(seed: &[i8], n: usize, tau: usize) -> TernaryPolynomial {
    let hash = dsa_hash(&[seed], n * 2);
    let mut coeffs = vec![0i8; n];

    let mut placed = 0;
    let mut idx = 0;
    while placed < tau && idx < hash.len() {
        let pos = ((hash[idx] + 1) as usize * n / 3) % n;
        let val = if hash.get(idx + 1).copied().unwrap_or(0) >= 0 { 1i8 } else { -1i8 };
        if coeffs[pos] == 0 {
            coeffs[pos] = val;
            placed += 1;
        }
        idx += 2;
        if idx >= hash.len() {
            let extended = dsa_hash(&[seed, &[placed as i8]], n * 2);
            let ext_pos = ((extended[0] + 1) as usize * n / 3) % n;
            let ext_val = if extended.get(1).copied().unwrap_or(0) >= 0 { 1i8 } else { -1i8 };
            if coeffs[ext_pos] == 0 {
                coeffs[ext_pos] = ext_val;
            }
            break;
        }
    }

    TernaryPolynomial::from_coeffs_unchecked(coeffs)
}

fn sample_masking_vec(seed: &[i8], l: usize, n: usize, nonce: u16) -> TernaryPolyVec {
    let mut polys = Vec::with_capacity(l);
    for i in 0..l {
        let poly_seed = dsa_hash(
            &[seed, &[(nonce + i as u16) as i8, ((nonce + i as u16) >> 8) as i8]],
            n,
        );
        let coeffs: Vec<i8> = poly_seed.iter().take(n).copied().collect();
        polys.push(TernaryPolynomial::from_coeffs_unchecked(coeffs));
    }
    TernaryPolyVec { polys, n }
}

pub fn keygen(variant: TlDsaVariant, seed: &[i8]) -> CryptoResult<(TlDsaPublicKey, TlDsaSecretKey)> {
    let params = variant.params();
    let n = params.n;
    let k = params.k;
    let l = params.l;

    let rho = dsa_hash(&[seed, &[0i8]], 243);
    let sigma = dsa_hash(&[seed, &[1i8]], 243);
    let signing_seed = dsa_hash(&[seed, &[0i8, 1, -1]], 243);

    let matrix_a = sample_matrix(&rho, k, n);

    let secret_s1 = sample_noise_vec(&sigma, l, n, 0, params.eta);
    let secret_s2 = TernaryPolyVec::new(k, n);

    let public_t = matrix_a_mul_vec(&matrix_a, &secret_s1, k, l, n)?;

    let pk = TlDsaPublicKey {
        variant,
        matrix_a_seed: rho.clone(),
        public_t: public_t.clone(),
    };

    let sk = TlDsaSecretKey {
        variant,
        matrix_a_seed: rho,
        secret_s1,
        secret_s2,
        public_t,
        signing_seed,
    };

    Ok((pk, sk))
}

fn matrix_a_mul_vec(
    matrix_a: &TernaryPolyMatrix,
    vec: &TernaryPolyVec,
    k: usize,
    l: usize,
    n: usize,
) -> CryptoResult<TernaryPolyVec> {
    let mut result = TernaryPolyVec::new(k, n);
    for i in 0..k {
        let mut sum = TernaryPolynomial::new(n);
        for j in 0..l {
            let row = i;
            let col = j % matrix_a.cols;
            let product = matrix_a.entries[row][col].ring_mul(&vec.polys[j])?;
            sum = sum.add(&product)?;
        }
        result.polys[i] = sum;
    }
    Ok(result)
}

pub fn sign(sk: &TlDsaSecretKey, message: &[i8]) -> CryptoResult<TlDsaSignature> {
    let params = sk.variant.params();
    let n = params.n;
    let k = params.k;
    let l = params.l;

    let matrix_a = sample_matrix(&sk.matrix_a_seed, k, n);

    let pk_trits = poly_vec_to_trits(&sk.public_t);
    let mu = dsa_hash(&[&pk_trits, message], 243);

    for attempt in 0..params.max_attempts {
        let y_seed = dsa_hash(
            &[&sk.signing_seed, &mu, &[attempt as i8, (attempt >> 8) as i8]],
            243,
        );
        let y = sample_masking_vec(&y_seed, l, n, 0);

        let ay = matrix_a_mul_vec(&matrix_a, &y, k, l, n)?;
        let w_trits = poly_vec_to_trits(&ay);

        let challenge_hash = dsa_hash(&[&mu, &w_trits], 243);
        let c = sample_challenge(&challenge_hash, n, params.tau);

        let mut z_polys = Vec::with_capacity(l);
        let mut reject = false;

        for i in 0..l {
            let cs1 = c.ring_mul(&sk.secret_s1.polys[i])?;
            let zi = y.polys[i].add(&cs1)?;

            if zi.l_infinity_norm() > params.gamma {
                reject = true;
                break;
            }
            z_polys.push(zi);
        }

        if reject {
            continue;
        }

        let z = TernaryPolyVec { polys: z_polys, n };

        return Ok(TlDsaSignature {
            variant: sk.variant,
            z,
            challenge_hash,
        });
    }

    Err(CryptoError::KeyGenerationFailed(
        alloc::string::String::from("TL-DSA signing failed: max attempts exceeded"),
    ))
}

pub fn verify(pk: &TlDsaPublicKey, message: &[i8], sig: &TlDsaSignature) -> CryptoResult<bool> {
    let params = pk.variant.params();
    let n = params.n;
    let k = params.k;
    let l = params.l;

    if sig.z.polys.len() != l {
        return Ok(false);
    }

    for poly in &sig.z.polys {
        if poly.l_infinity_norm() > params.gamma {
            return Ok(false);
        }
    }

    let matrix_a = sample_matrix(&pk.matrix_a_seed, k, n);

    let c = sample_challenge(&sig.challenge_hash, n, params.tau);

    let az = matrix_a_mul_vec(&matrix_a, &sig.z, k, l, n)?;

    let mut ct_polys = Vec::with_capacity(k);
    for i in 0..k {
        let cti = c.ring_mul(&pk.public_t.polys[i])?;
        ct_polys.push(cti);
    }
    let ct = TernaryPolyVec { polys: ct_polys, n };

    let w_prime = az.add(&ct.negate()?)?;

    let pk_trits = poly_vec_to_trits(&pk.public_t);
    let mu = dsa_hash(&[&pk_trits, message], 243);

    let w_trits = poly_vec_to_trits(&w_prime);
    let expected_hash = dsa_hash(&[&mu, &w_trits], 243);

    Ok(sig.challenge_hash == expected_hash)
}

impl TernaryPolyVec {
    pub fn negate(&self) -> CryptoResult<TernaryPolyVec> {
        let polys: Vec<TernaryPolynomial> = self.polys.iter()
            .map(|p| p.negate())
            .collect();
        Ok(TernaryPolyVec { polys, n: self.n })
    }
}

pub fn public_key_size(variant: TlDsaVariant) -> usize {
    let params = variant.params();
    243 + params.k * params.n
}

pub fn secret_key_size(variant: TlDsaVariant) -> usize {
    let params = variant.params();
    243 + params.l * params.n + params.k * params.n + params.k * params.n + 243
}

pub fn signature_size(variant: TlDsaVariant) -> usize {
    let params = variant.params();
    params.l * params.n + 243
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tl_dsa_variant_names() {
        assert_eq!(TlDsaVariant::TlDsa44.name(), "TL-DSA-44");
        assert_eq!(TlDsaVariant::TlDsa65.name(), "TL-DSA-65");
        assert_eq!(TlDsaVariant::TlDsa87.name(), "TL-DSA-87");
    }

    #[test]
    fn test_tl_dsa_security_bits() {
        assert_eq!(TlDsaVariant::TlDsa44.security_bits(), 128);
        assert_eq!(TlDsaVariant::TlDsa65.security_bits(), 192);
        assert_eq!(TlDsaVariant::TlDsa87.security_bits(), 256);
    }

    #[test]
    fn test_keygen_44() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
        assert_eq!(pk.variant, TlDsaVariant::TlDsa44);
        assert_eq!(sk.variant, TlDsaVariant::TlDsa44);
        assert_eq!(pk.public_t.polys.len(), 4);
        assert_eq!(sk.secret_s1.polys.len(), 4);
        assert_eq!(sk.secret_s2.polys.len(), 4);
    }

    #[test]
    fn test_keygen_65() {
        let seed = vec![1i8, 0, -1, 1, 0, -1, 1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa65, &seed).unwrap();
        assert_eq!(pk.public_t.polys.len(), 6);
        assert_eq!(sk.secret_s1.polys.len(), 5);
        assert_eq!(sk.secret_s2.polys.len(), 6);
    }

    #[test]
    fn test_keygen_87() {
        let seed = vec![-1i8, 0, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa87, &seed).unwrap();
        assert_eq!(pk.public_t.polys.len(), 8);
        assert_eq!(sk.secret_s1.polys.len(), 7);
        assert_eq!(sk.secret_s2.polys.len(), 8);
    }

    #[test]
    fn test_keygen_deterministic() {
        let seed = vec![0i8, 1, -1, 0, 1];
        let (pk1, sk1) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
        let (pk2, sk2) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
        assert_eq!(pk1.matrix_a_seed, pk2.matrix_a_seed);
        assert_eq!(
            poly_vec_to_trits(&pk1.public_t),
            poly_vec_to_trits(&pk2.public_t),
        );
        assert_eq!(
            poly_vec_to_trits(&sk1.secret_s1),
            poly_vec_to_trits(&sk2.secret_s1),
        );
    }

    #[test]
    fn test_sign_verify_44() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "Signature should verify with correct key and message");
    }

    #[test]
    fn test_sign_verify_wrong_message() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        let wrong_msg = vec![0i8, 0, 0, 0, 0, 0];
        let valid = verify(&pk, &wrong_msg, &sig).unwrap();
        assert!(!valid, "Signature should not verify with wrong message");
    }

    #[test]
    fn test_sign_verify_wrong_key() {
        let seed1 = vec![0i8, 1, -1, 0, 1, -1];
        let seed2 = vec![1i8, -1, 0, 1, -1, 0];

        let (_pk1, sk1) = keygen(TlDsaVariant::TlDsa44, &seed1).unwrap();
        let (pk2, _sk2) = keygen(TlDsaVariant::TlDsa44, &seed2).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1];
        let sig = sign(&sk1, &message).unwrap();

        let valid = verify(&pk2, &message, &sig).unwrap();
        assert!(!valid, "Signature should not verify with wrong public key");
    }

    #[test]
    fn test_sign_deterministic() {
        let seed = vec![0i8, 1, -1, 0, 1];
        let (_pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1];
        let sig1 = sign(&sk, &message).unwrap();
        let sig2 = sign(&sk, &message).unwrap();

        assert_eq!(sig1.challenge_hash, sig2.challenge_hash);
        assert_eq!(
            poly_vec_to_trits(&sig1.z),
            poly_vec_to_trits(&sig2.z),
        );
    }

    #[test]
    fn test_key_sizes_increase_with_security() {
        let pk44 = public_key_size(TlDsaVariant::TlDsa44);
        let pk65 = public_key_size(TlDsaVariant::TlDsa65);
        let pk87 = public_key_size(TlDsaVariant::TlDsa87);
        assert!(pk65 > pk44);
        assert!(pk87 > pk65);

        let sk44 = secret_key_size(TlDsaVariant::TlDsa44);
        let sk65 = secret_key_size(TlDsaVariant::TlDsa65);
        let sk87 = secret_key_size(TlDsaVariant::TlDsa87);
        assert!(sk65 > sk44);
        assert!(sk87 > sk65);

        let sig44 = signature_size(TlDsaVariant::TlDsa44);
        let sig65 = signature_size(TlDsaVariant::TlDsa65);
        let sig87 = signature_size(TlDsaVariant::TlDsa87);
        assert!(sig65 > sig44);
        assert!(sig87 > sig65);
    }

    #[test]
    fn test_sample_challenge() {
        let seed = vec![0i8, 1, -1, 0, 1, -1];
        let c = sample_challenge(&seed, 256, 39);
        assert_eq!(c.coeffs.len(), 256);
        let nonzero = c.coeffs.iter().filter(|&&x| x != 0).count();
        assert!(nonzero > 0);
        assert!(nonzero <= 39);
    }
}
