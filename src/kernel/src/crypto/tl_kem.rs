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

//! Ternary Lattice Key Encapsulation Mechanism (TL-KEM)
//!
//! Implements a ternary-native equivalent of ML-KEM (FIPS 203) using
//! GF(3) polynomial ring arithmetic from `ternary_lattice`. Provides
//! IND-CCA2 secure key encapsulation at three security levels:
//!
//! | Variant      | Module Rank (k) | NIST Level | ML-KEM Equivalent |
//! |-------------|-----------------|------------|-------------------|
//! | TL-KEM-512  | k=2             | Level 1    | ML-KEM-512        |
//! | TL-KEM-768  | k=3             | Level 3    | ML-KEM-768        |
//! | TL-KEM-1024 | k=4             | Level 5    | ML-KEM-1024       |
//!
//! # Construction
//!
//! TL-KEM follows the Fujisaki-Okamoto (FO) transform pattern:
//! 1. **KeyGen**: Generate Module-LWE keypair (A, t=As+e) with ternary noise
//! 2. **Encapsulate**: Encrypt random message m under public key, derive
//!    shared secret K = H(m || c) where c is the ciphertext
//! 3. **Decapsulate**: Decrypt ciphertext, re-encrypt to verify, derive K
//!
//! All arithmetic operates in R_q = Z_3[X]/(X^n+1) with balanced ternary
//! coefficients {-1, 0, +1}, mapping directly to PlenumNET Representation A.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use super::CryptoResult;
use super::ternary_lattice::{
    TernaryPolynomial, TernaryPolyVec, LatticeParams,
    sample_matrix, sample_noise_vec, compress_ternary, decompress_ternary,
};
use super::sponge::TernarySponge;
use super::ct_utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlKemVariant {
    TlKem512,
    TlKem768,
    TlKem1024,
}

impl TlKemVariant {
    pub fn name(&self) -> &'static str {
        match self {
            TlKemVariant::TlKem512 => "TL-KEM-512",
            TlKemVariant::TlKem768 => "TL-KEM-768",
            TlKemVariant::TlKem1024 => "TL-KEM-1024",
        }
    }

    pub fn params(&self) -> LatticeParams {
        match self {
            TlKemVariant::TlKem512 => LatticeParams::security_level_1(),
            TlKemVariant::TlKem768 => LatticeParams::security_level_3(),
            TlKemVariant::TlKem1024 => LatticeParams::security_level_5(),
        }
    }

    pub fn security_bits(&self) -> u32 {
        match self {
            TlKemVariant::TlKem512 => 128,
            TlKemVariant::TlKem768 => 192,
            TlKemVariant::TlKem1024 => 256,
        }
    }

    pub fn shared_secret_trits(&self) -> usize {
        match self {
            TlKemVariant::TlKem512 => 243,
            TlKemVariant::TlKem768 => 243,
            TlKemVariant::TlKem1024 => 486,
        }
    }
}

const KEM_MESSAGE_TRITS: usize = 243;

#[derive(Debug, Clone)]
pub struct TlKemPublicKey {
    pub variant: TlKemVariant,
    pub matrix_a_seed: Vec<i8>,
    pub public_vec_t: TernaryPolyVec,
}

#[derive(Debug, Clone)]
pub struct TlKemSecretKey {
    pub variant: TlKemVariant,
    pub secret_s: TernaryPolyVec,
    pub public_key: TlKemPublicKey,
    pub hash_pk: Vec<i8>,
    pub implicit_reject_seed: Vec<i8>,
}

#[derive(Debug, Clone)]
pub struct TlKemCiphertext {
    pub variant: TlKemVariant,
    pub compressed_u: Vec<Vec<u8>>,
    pub compressed_v: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecret {
    pub trits: Vec<i8>,
}

fn kem_hash(inputs: &[&[i8]], output_len: usize) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    for input in inputs {
        sponge.absorb(input);
    }
    sponge.squeeze(output_len).trits
}

fn generate_message(seed: &[i8]) -> Vec<i8> {
    kem_hash(&[seed, &[0i8, 1, -1]], KEM_MESSAGE_TRITS)
}

pub fn keygen(variant: TlKemVariant, seed: &[i8]) -> CryptoResult<(TlKemPublicKey, TlKemSecretKey)> {
    let params = variant.params();
    let k = params.k;
    let n = params.n;

    let rho = kem_hash(&[seed, &[0i8]], 243);
    let sigma = kem_hash(&[seed, &[1i8]], 243);

    let matrix_a = sample_matrix(&rho, k, n);
    let secret_s = sample_noise_vec(&sigma, k, n, 0, params.eta1);
    let error_e = sample_noise_vec(&sigma, k, n, k as u16, params.eta1);

    let as_product = matrix_a.mul_vec(&secret_s)?;
    let public_t = as_product.add(&error_e)?;

    let hash_pk = kem_hash(&[&rho, &poly_vec_to_trits(&public_t)], 243);
    let reject_seed = kem_hash(&[seed, &[0i8, 0, -1]], 243);

    let pk = TlKemPublicKey {
        variant,
        matrix_a_seed: rho,
        public_vec_t: public_t,
    };

    let sk = TlKemSecretKey {
        variant,
        secret_s,
        public_key: pk.clone(),
        hash_pk,
        implicit_reject_seed: reject_seed,
    };

    Ok((pk, sk))
}

pub fn encapsulate(pk: &TlKemPublicKey, randomness: &[i8]) -> CryptoResult<(TlKemCiphertext, SharedSecret)> {
    let params = pk.variant.params();
    let k = params.k;
    let n = params.n;

    let message = generate_message(randomness);

    let pk_hash = kem_hash(&[&pk.matrix_a_seed, &poly_vec_to_trits(&pk.public_vec_t)], 243);
    let combined = kem_hash(&[&message, &pk_hash], 486);
    let shared_key_seed = &combined[..243];
    let encaps_coins = &combined[243..];

    let matrix_a = sample_matrix(&pk.matrix_a_seed, k, n);
    let r = sample_noise_vec(encaps_coins, k, n, 0, params.eta1);
    let e1 = sample_noise_vec(encaps_coins, k, n, k as u16, params.eta2);
    let e2_poly = super::ternary_lattice::sample_cbd_ternary(encaps_coins, n, (2 * k) as u16, params.eta2);

    let a_t = matrix_a.transpose();
    let u = a_t.mul_vec(&r)?.add(&e1)?;

    let t_dot_r = pk.public_vec_t.inner_product(&r)?;
    let msg_poly = message_to_polynomial(&message, n);
    let v = t_dot_r.add(&e2_poly)?.add(&msg_poly)?;

    let compressed_u: Vec<Vec<u8>> = u.polys.iter()
        .map(|p| compress_ternary(p, params.du))
        .collect();
    let compressed_v = compress_ternary(&v, params.dv);

    let ct_trits = ciphertext_to_trits(&compressed_u, &compressed_v);
    let shared_trits = kem_hash(
        &[shared_key_seed, &ct_trits],
        pk.variant.shared_secret_trits(),
    );

    let ct = TlKemCiphertext {
        variant: pk.variant,
        compressed_u,
        compressed_v,
    };

    let shared = SharedSecret { trits: shared_trits };

    Ok((ct, shared))
}

pub fn decapsulate(sk: &TlKemSecretKey, ct: &TlKemCiphertext) -> CryptoResult<SharedSecret> {
    let params = sk.variant.params();
    let n = params.n;

    let u_polys: Vec<TernaryPolynomial> = ct.compressed_u.iter()
        .map(|cu| decompress_ternary(cu, n, params.du))
        .collect();
    let u = TernaryPolyVec { polys: u_polys, n };

    let v = decompress_ternary(&ct.compressed_v, n, params.dv);

    let s_dot_u = sk.secret_s.inner_product(&u)?;
    let m_prime = v.sub(&s_dot_u)?;
    let message = polynomial_to_message(&m_prime, KEM_MESSAGE_TRITS);

    let pk_hash = kem_hash(
        &[&sk.public_key.matrix_a_seed, &poly_vec_to_trits(&sk.public_key.public_vec_t)],
        243,
    );
    let combined = kem_hash(&[&message, &pk_hash], 486);
    let shared_key_seed = &combined[..243];
    let encaps_coins = &combined[243..];

    let (ct_prime, _) = encapsulate_inner(&sk.public_key, &message, encaps_coins)?;

    let ct_trits = ciphertext_to_trits(&ct.compressed_u, &ct.compressed_v);
    let ct_prime_trits = ciphertext_to_trits(&ct_prime.compressed_u, &ct_prime.compressed_v);

    let match_flag = ct_utils::ct_eq_slices(&ct_trits, &ct_prime_trits);
    let match_bit = match_flag & 1;

    let ss_accept = kem_hash(
        &[shared_key_seed, &ct_trits],
        sk.variant.shared_secret_trits(),
    );
    let ss_reject = kem_hash(
        &[&sk.implicit_reject_seed, &ct_trits],
        sk.variant.shared_secret_trits(),
    );

    let shared_trits = ct_utils::ct_select_vec(match_bit, &ss_accept, &ss_reject);

    Ok(SharedSecret { trits: shared_trits })
}

fn encapsulate_inner(
    pk: &TlKemPublicKey,
    message: &[i8],
    coins: &[i8],
) -> CryptoResult<(TlKemCiphertext, Vec<i8>)> {
    let params = pk.variant.params();
    let k = params.k;
    let n = params.n;

    let matrix_a = sample_matrix(&pk.matrix_a_seed, k, n);
    let r = sample_noise_vec(coins, k, n, 0, params.eta1);
    let e1 = sample_noise_vec(coins, k, n, k as u16, params.eta2);
    let e2_poly = super::ternary_lattice::sample_cbd_ternary(coins, n, (2 * k) as u16, params.eta2);

    let a_t = matrix_a.transpose();
    let u = a_t.mul_vec(&r)?.add(&e1)?;

    let t_dot_r = pk.public_vec_t.inner_product(&r)?;
    let msg_poly = message_to_polynomial(message, n);
    let v = t_dot_r.add(&e2_poly)?.add(&msg_poly)?;

    let compressed_u: Vec<Vec<u8>> = u.polys.iter()
        .map(|p| compress_ternary(p, params.du))
        .collect();
    let compressed_v = compress_ternary(&v, params.dv);

    let ct = TlKemCiphertext {
        variant: pk.variant,
        compressed_u,
        compressed_v,
    };

    Ok((ct, message.to_vec()))
}

fn message_to_polynomial(message: &[i8], n: usize) -> TernaryPolynomial {
    let mut coeffs = vec![0i8; n];
    for (i, &t) in message.iter().enumerate() {
        if i < n {
            coeffs[i] = t;
        }
    }
    TernaryPolynomial::from_coeffs_unchecked(coeffs)
}

fn polynomial_to_message(poly: &TernaryPolynomial, msg_len: usize) -> Vec<i8> {
    let mut message = Vec::with_capacity(msg_len);
    for i in 0..msg_len {
        if i < poly.coeffs.len() {
            let c = poly.coeffs[i];
            let decoded = if c == 0 { 0i8 } else if c == 1 { 1i8 } else { -1i8 };
            message.push(decoded);
        } else {
            message.push(0);
        }
    }
    message
}

fn poly_vec_to_trits(v: &TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::new();
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

fn ciphertext_to_trits(compressed_u: &[Vec<u8>], compressed_v: &[u8]) -> Vec<i8> {
    let mut trits = Vec::new();
    for cu in compressed_u {
        for &b in cu {
            trits.push(((b % 3) as i8) - 1);
        }
    }
    for &b in compressed_v {
        trits.push(((b % 3) as i8) - 1);
    }
    trits
}

pub fn public_key_size(variant: TlKemVariant) -> usize {
    let params = variant.params();
    let seed_trits = 243;
    let vec_trits = params.k * params.n;
    seed_trits + vec_trits
}

pub fn secret_key_size(variant: TlKemVariant) -> usize {
    let params = variant.params();
    let s_trits = params.k * params.n;
    let pk_size = public_key_size(variant);
    let hash_trits = 243;
    let reject_seed_trits = 243;
    s_trits + pk_size + hash_trits + reject_seed_trits
}

pub fn ciphertext_size(variant: TlKemVariant) -> usize {
    let params = variant.params();
    let u_bytes = params.k * params.n;
    let v_bytes = params.n;
    u_bytes + v_bytes
}

pub fn shared_secret_size(variant: TlKemVariant) -> usize {
    variant.shared_secret_trits()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tl_kem_variant_names() {
        assert_eq!(TlKemVariant::TlKem512.name(), "TL-KEM-512");
        assert_eq!(TlKemVariant::TlKem768.name(), "TL-KEM-768");
        assert_eq!(TlKemVariant::TlKem1024.name(), "TL-KEM-1024");
    }

    #[test]
    fn test_tl_kem_security_bits() {
        assert_eq!(TlKemVariant::TlKem512.security_bits(), 128);
        assert_eq!(TlKemVariant::TlKem768.security_bits(), 192);
        assert_eq!(TlKemVariant::TlKem1024.security_bits(), 256);
    }

    #[test]
    fn test_keygen_512() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        assert_eq!(pk.variant, TlKemVariant::TlKem512);
        assert_eq!(sk.variant, TlKemVariant::TlKem512);
        assert_eq!(pk.public_vec_t.polys.len(), 2);
        assert_eq!(sk.secret_s.polys.len(), 2);
        assert_eq!(pk.matrix_a_seed.len(), 243);
    }

    #[test]
    fn test_keygen_768() {
        let seed = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 0, 1];
        let (pk, sk) = keygen(TlKemVariant::TlKem768, &seed).unwrap();
        assert_eq!(pk.public_vec_t.polys.len(), 3);
        assert_eq!(sk.secret_s.polys.len(), 3);
    }

    #[test]
    fn test_keygen_1024() {
        let seed = vec![-1i8, 0, 1, -1, 0, 1, -1, 0, 1, 0, -1, 1];
        let (pk, sk) = keygen(TlKemVariant::TlKem1024, &seed).unwrap();
        assert_eq!(pk.public_vec_t.polys.len(), 4);
        assert_eq!(sk.secret_s.polys.len(), 4);
    }

    #[test]
    fn test_keygen_deterministic() {
        let seed = vec![0i8, 1, -1, 0, 1];
        let (pk1, sk1) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        let (pk2, sk2) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        assert_eq!(pk1.matrix_a_seed, pk2.matrix_a_seed);
        assert_eq!(
            poly_vec_to_trits(&pk1.public_vec_t),
            poly_vec_to_trits(&pk2.public_vec_t)
        );
        assert_eq!(
            poly_vec_to_trits(&sk1.secret_s),
            poly_vec_to_trits(&sk2.secret_s)
        );
    }

    #[test]
    fn test_encapsulate_decapsulate_512() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlKemVariant::TlKem512, &seed).unwrap();

        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let (ct, shared1) = encapsulate(&pk, &randomness).unwrap();
        let shared2 = decapsulate(&sk, &ct).unwrap();

        assert_eq!(shared1.trits.len(), 243);
        assert_eq!(shared2.trits.len(), 243);
        assert_eq!(shared1, shared2);
    }

    #[test]
    fn test_encapsulate_decapsulate_768() {
        let seed = vec![1i8, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlKemVariant::TlKem768, &seed).unwrap();

        let randomness = vec![-1i8, 0, 1, -1, 0, 1];
        let (ct, shared1) = encapsulate(&pk, &randomness).unwrap();
        let shared2 = decapsulate(&sk, &ct).unwrap();

        assert_eq!(shared1, shared2);
    }

    #[test]
    fn test_encapsulate_decapsulate_1024() {
        let seed = vec![0i8, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlKemVariant::TlKem1024, &seed).unwrap();

        let randomness = vec![1i8, 1, -1, 0, -1, 1, 0];
        let (ct, shared1) = encapsulate(&pk, &randomness).unwrap();
        let shared2 = decapsulate(&sk, &ct).unwrap();

        assert_eq!(shared1.trits.len(), 486);
        assert_eq!(shared2.trits.len(), 486);
        assert_eq!(shared1, shared2);
    }

    #[test]
    fn test_different_randomness_different_secrets() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1];
        let (pk, _) = keygen(TlKemVariant::TlKem512, &seed).unwrap();

        let r1 = vec![0i8, 0, 0, 0, 0];
        let r2 = vec![1i8, 0, 0, 0, 0];

        let (_, shared1) = encapsulate(&pk, &r1).unwrap();
        let (_, shared2) = encapsulate(&pk, &r2).unwrap();

        assert_ne!(shared1, shared2);
    }

    #[test]
    fn test_wrong_secret_key_decapsulation() {
        let seed1 = vec![0i8, 1, -1, 0, 1, -1];
        let seed2 = vec![1i8, -1, 0, 1, -1, 0];

        let (pk1, _sk1) = keygen(TlKemVariant::TlKem512, &seed1).unwrap();
        let (_pk2, sk2) = keygen(TlKemVariant::TlKem512, &seed2).unwrap();

        let randomness = vec![0i8, 1, -1, 0, 1];
        let (ct, shared_encaps) = encapsulate(&pk1, &randomness).unwrap();
        let shared_decaps = decapsulate(&sk2, &ct).unwrap();

        assert_ne!(shared_encaps, shared_decaps);
    }

    #[test]
    fn test_implicit_reject() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlKemVariant::TlKem512, &seed).unwrap();

        let randomness = vec![0i8, 1, -1];
        let (mut ct, _) = encapsulate(&pk, &randomness).unwrap();

        if let Some(first_u) = ct.compressed_u.first_mut() {
            if let Some(byte) = first_u.first_mut() {
                *byte = byte.wrapping_add(1);
            }
        }

        let shared_reject = decapsulate(&sk, &ct).unwrap();
        assert!(!shared_reject.trits.is_empty());
    }

    #[test]
    fn test_message_polynomial_roundtrip() {
        let msg = vec![1i8, 0, -1, 1, -1, 0, 0, 1, -1, 0];
        let poly = message_to_polynomial(&msg, 256);
        let recovered = polynomial_to_message(&poly, msg.len());
        assert_eq!(msg, recovered);
    }

    #[test]
    fn test_key_sizes() {
        assert!(public_key_size(TlKemVariant::TlKem512) > 0);
        assert!(secret_key_size(TlKemVariant::TlKem512) > public_key_size(TlKemVariant::TlKem512));
        assert!(ciphertext_size(TlKemVariant::TlKem512) > 0);

        assert!(public_key_size(TlKemVariant::TlKem768) > public_key_size(TlKemVariant::TlKem512));
        assert!(public_key_size(TlKemVariant::TlKem1024) > public_key_size(TlKemVariant::TlKem768));
    }

    #[test]
    fn test_shared_secret_sizes() {
        assert_eq!(shared_secret_size(TlKemVariant::TlKem512), 243);
        assert_eq!(shared_secret_size(TlKemVariant::TlKem768), 243);
        assert_eq!(shared_secret_size(TlKemVariant::TlKem1024), 486);
    }
}
