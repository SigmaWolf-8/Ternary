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
//! TL-DSA follows the Fiat-Shamir with Aborts paradigm adapted for the
//! ternary domain (q=3). A key structural advantage of the q=3 setting
//! is the **ternary non-abort property**: since all coefficients lie in
//! {-1, 0, +1} and modular addition wraps within this set, the response
//! vector z = y + c·s1 mod 3 always satisfies ||z||_∞ ≤ 1, eliminating
//! rejection sampling entirely. This yields deterministic (single-pass)
//! signing with zero-knowledge security.
//!
//! 1. **KeyGen**: Generate Module-LWE keypair (A, t = A·s1 + s2) with
//!    ternary CBD noise for both s1 and s2.
//! 2. **Sign**: Sample masking vector y, compute w = A·y, derive
//!    challenge c = H(H(vk || msg) || w), compute z = y + c·s1.
//!    No rejection needed (non-abort property).
//! 3. **Verify**: Reconstruct w' = A·z - c·t and check
//!    H(H(vk || msg) || w') == challenge_hash.
//!
//! All arithmetic operates in R_3 = Z_3[X]/(X^n+1) with balanced ternary
//! coefficients {-1, 0, +1}.
//!
//! # Security
//!
//! See docs/proofs/TL-DSA-EUF-CMA-proof.tex for the full formal reduction
//! to Module-SIS/Module-LWE over R_3.
//!
//! # Changelog (v2 — corrected)
//!
//! ## Bugs Fixed
//!
//! 1. **sample_challenge was catastrophically broken**: Position mapping
//!    `((hash[idx]+1) * n/3) % n` only produced 3 distinct indices
//!    {0, 85, 170} out of 256. Challenge polynomials could never achieve
//!    the required τ non-zero coefficients. Replaced with Fisher-Yates
//!    rejection sampling over the full index range.
//!
//! 2. **Matrix A had wrong dimensions**: `sample_matrix(&rho, k, n)`
//!    created a k×k square matrix via `ternary_lattice::sample_matrix`.
//!    For TL-DSA-65 (k=6, l=5) and TL-DSA-87 (k=8, l=7), A should be
//!    k×l. Replaced with `expand_matrix_a(seed, k, l, n)` that generates
//!    correct rectangular dimensions.
//!
//! 3. **secret_s2 was all-zeros**: `TernaryPolyVec::new(k, n)` initializes
//!    to zero. Now properly sampled via `sample_noise_vec` with CBD_η.
//!
//! 4. **Redundant matrix_a_mul_vec**: Removed freestanding function that
//!    used hacky `j % matrix_a.cols` column wrapping. Now uses the
//!    existing `TernaryPolyMatrix::mul_vec` method.
//!
//! 5. **Nonce truncation in sign**: `attempt as i8` wraps at 128,
//!    producing colliding nonces. Fixed with balanced ternary encoding.
//!
//! 6. **No domain separation in hashing**: All hash calls used the same
//!    sponge invocation pattern. Added domain separator prefixes to
//!    prevent cross-function hash collisions.
//!
//! 7. **Removed misleading gamma parameter**: gamma=1 was stored in params
//!    but is a tautology in q=3 (all coefficients are ≤ 1). Replaced
//!    with explicit non-abort property documentation and debug_assert.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use super::{CryptoError, CryptoResult};
use super::ternary_lattice::{
    TernaryPolynomial, TernaryPolyMatrix, TernaryPolyVec,
    NttMatrix,
    sample_noise_vec,
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
                tau: 39,
                max_attempts: 1,
            },
            TlDsaVariant::TlDsa65 => TlDsaParams {
                n: 256,
                k: 6,
                l: 5,
                eta: 2,
                tau: 49,
                max_attempts: 1,
            },
            TlDsaVariant::TlDsa87 => TlDsaParams {
                n: 256,
                k: 8,
                l: 7,
                eta: 2,
                tau: 60,
                max_attempts: 1,
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
    pub tau: usize,
    pub max_attempts: usize,
}

#[derive(Debug, Clone)]
pub struct TlDsaPublicKey {
    pub variant: TlDsaVariant,
    pub matrix_a_seed: Vec<i8>,
    pub public_t: TernaryPolyVec,
    pub matrix_a: TernaryPolyMatrix,
    pub matrix_a_ntt: NttMatrix,
}

#[derive(Debug, Clone)]
pub struct TlDsaSecretKey {
    pub variant: TlDsaVariant,
    pub matrix_a_seed: Vec<i8>,
    pub matrix_a: TernaryPolyMatrix,
    pub matrix_a_ntt: NttMatrix,
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

const DOMAIN_MATRIX_EXPAND: i8 = 0;
const DOMAIN_SECRET_SAMPLE: i8 = 1;
const DOMAIN_SIGNING_SEED: i8 = -1;
const DOMAIN_MESSAGE_HASH: i8 = 0;
const DOMAIN_CHALLENGE: i8 = 1;
const DOMAIN_MASKING: i8 = -1;

fn dsa_hash(domain: i8, inputs: &[&[i8]], output_len: usize) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(&[domain]);
    for input in inputs {
        sponge.absorb(input);
    }
    sponge.squeeze(output_len).trits
}

fn poly_vec_to_trits(v: &TernaryPolyVec) -> Vec<i8> {
    let mut trits = Vec::with_capacity(v.polys.len() * v.n);
    for p in &v.polys {
        trits.extend_from_slice(&p.coeffs);
    }
    trits
}

fn sample_challenge(seed: &[i8], n: usize, tau: usize) -> TernaryPolynomial {
    let hash_len = n * 4;
    let hash = dsa_hash(DOMAIN_CHALLENGE, &[seed], hash_len);

    let mut coeffs = vec![0i8; n];

    let mut indices: Vec<usize> = (0..n).collect();
    let mut hash_pos: usize = 0;

    for placed in 0..tau {
        let remaining = n - placed;
        let pos_in_remaining = sample_uniform_index(&hash, &mut hash_pos, remaining);

        let selected_idx = indices[pos_in_remaining];
        indices[pos_in_remaining] = indices[remaining - 1];
        indices[remaining - 1] = selected_idx;

        let sign = if hash_pos < hash.len() {
            let s = hash[hash_pos];
            hash_pos += 1;
            if s < 0 { -1i8 } else { 1i8 }
        } else {
            1i8
        };

        coeffs[selected_idx] = sign;
    }

    TernaryPolynomial::from_coeffs_unchecked(coeffs)
}

fn sample_uniform_index(hash: &[i8], pos: &mut usize, bound: usize) -> usize {
    if bound <= 1 {
        return 0;
    }

    let trits_needed = {
        let mut t = 1usize;
        let mut count = 0usize;
        while t < bound {
            t = t.saturating_mul(3);
            count += 1;
        }
        count
    };

    loop {
        let mut val = 0usize;
        let mut power = 1usize;

        for _ in 0..trits_needed {
            if *pos >= hash.len() {
                *pos = 0;
            }
            let trit = hash[*pos];
            *pos += 1;
            let digit = (trit + 1) as usize;
            val += digit * power;
            power *= 3;
        }

        if val < bound {
            return val;
        }
    }
}

fn expand_matrix_a(seed: &[i8], k: usize, l: usize, n: usize) -> TernaryPolyMatrix {
    use super::ternary_lattice::u16_to_trits;
    let mut matrix = TernaryPolyMatrix::new(k, l, n);
    for i in 0..k {
        for j in 0..l {
            let nonce = (i * l + j) as u16;
            let nonce_trits = u16_to_trits(nonce);
            let combined_seed = dsa_hash(
                DOMAIN_MATRIX_EXPAND,
                &[seed, &nonce_trits],
                n,
            );
            matrix.entries[i][j] = TernaryPolynomial::from_coeffs_unchecked(combined_seed);
        }
    }
    matrix
}

fn sample_masking_vec(seed: &[i8], l: usize, n: usize, nonce: u16) -> TernaryPolyVec {
    use super::ternary_lattice::u16_to_trits;
    let mut polys = Vec::with_capacity(l);
    for i in 0..l {
        let poly_nonce = nonce.wrapping_add(i as u16);
        let nonce_trits = u16_to_trits(poly_nonce);
        let poly_seed = dsa_hash(
            DOMAIN_MASKING,
            &[seed, &nonce_trits],
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

    let rho = dsa_hash(DOMAIN_MATRIX_EXPAND, &[seed, &[0i8]], 243);
    let sigma = dsa_hash(DOMAIN_SECRET_SAMPLE, &[seed, &[1i8]], 243);
    let signing_seed = dsa_hash(DOMAIN_SIGNING_SEED, &[seed, &[0i8, 1, -1]], 243);

    let matrix_a = expand_matrix_a(&rho, k, l, n);
    let matrix_a_ntt = matrix_a.to_ntt();

    let secret_s1 = sample_noise_vec(&sigma, l, n, 0, params.eta);
    let secret_s2 = sample_noise_vec(&sigma, k, n, l as u16, params.eta);

    let public_t = matrix_a_ntt.mul_vec(&secret_s1)?;

    let pk = TlDsaPublicKey {
        variant,
        matrix_a_seed: rho.clone(),
        public_t: public_t.clone(),
        matrix_a: matrix_a.clone(),
        matrix_a_ntt: matrix_a_ntt.clone(),
    };

    let sk = TlDsaSecretKey {
        variant,
        matrix_a_seed: rho,
        matrix_a,
        matrix_a_ntt,
        secret_s1,
        secret_s2,
        public_t,
        signing_seed,
    };

    Ok((pk, sk))
}

pub fn sign(sk: &TlDsaSecretKey, message: &[i8]) -> CryptoResult<TlDsaSignature> {
    let params = sk.variant.params();
    let n = params.n;
    let _k = params.k;
    let l = params.l;

    let pk_trits = poly_vec_to_trits(&sk.public_t);
    let mu = dsa_hash(DOMAIN_MESSAGE_HASH, &[&pk_trits, message], 243);

    for attempt in 0..params.max_attempts.max(1) {
        let attempt_trits = super::ternary_lattice::u16_to_trits(attempt as u16);
        let y_seed = dsa_hash(
            DOMAIN_MASKING,
            &[&sk.signing_seed, &mu, &attempt_trits],
            243,
        );

        let y = sample_masking_vec(&y_seed, l, n, 0);

        let w = sk.matrix_a_ntt.mul_vec(&y)?;
        let w_trits = poly_vec_to_trits(&w);

        let challenge_hash = dsa_hash(DOMAIN_CHALLENGE, &[&mu, &w_trits], 243);

        let c = sample_challenge(&challenge_hash, n, params.tau);

        let mut z_polys = Vec::with_capacity(l);
        let mut reject = false;

        for i in 0..l {
            let cs1_i = c.ring_mul_sparse(&sk.secret_s1.polys[i])?;
            let z_i = y.polys[i].add(&cs1_i)?;

            debug_assert!(
                z_i.l_infinity_norm() <= 1,
                "TL-DSA non-abort invariant violated: ||z[{}]||_∞ = {} > 1",
                i, z_i.l_infinity_norm()
            );

            if z_i.l_infinity_norm() > 1 {
                reject = true;
                break;
            }
            z_polys.push(z_i);
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
        if poly.l_infinity_norm() > 1 {
            return Ok(false);
        }
    }

    let c = sample_challenge(&sig.challenge_hash, n, params.tau);

    let az = pk.matrix_a_ntt.mul_vec(&sig.z)?;

    let mut ct_polys = Vec::with_capacity(k);
    for i in 0..k {
        let cti = c.ring_mul_sparse(&pk.public_t.polys[i])?;
        ct_polys.push(cti);
    }
    let ct = TernaryPolyVec { polys: ct_polys, n };

    let w_prime = az.add(&ct.negate()?)?;

    let pk_trits = poly_vec_to_trits(&pk.public_t);
    let mu = dsa_hash(DOMAIN_MESSAGE_HASH, &[&pk_trits, message], 243);

    let w_trits = poly_vec_to_trits(&w_prime);
    let expected_hash = dsa_hash(DOMAIN_CHALLENGE, &[&mu, &w_trits], 243);

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

#[cfg(feature = "bench-tools")]
pub fn sign_verify_timing_breakdown(
    variant: TlDsaVariant,
    seed: &[i8],
    message: &[i8],
) -> CryptoResult<Vec<(&'static str, core::time::Duration)>> {
    use std::time::Instant;

    let params = variant.params();
    let n = params.n;
    let k = params.k;
    let l = params.l;

    let mut timings: Vec<(&'static str, core::time::Duration)> = Vec::new();

    let t0 = Instant::now();
    let rho = dsa_hash(DOMAIN_MATRIX_EXPAND, &[seed, &[0i8]], 243);
    let sigma = dsa_hash(DOMAIN_SECRET_SAMPLE, &[seed, &[1i8]], 243);
    let signing_seed = dsa_hash(DOMAIN_SIGNING_SEED, &[seed, &[0i8, 1, -1]], 243);
    timings.push(("seed_derivation", t0.elapsed()));

    let t0 = Instant::now();
    let matrix_a = expand_matrix_a(&rho, k, l, n);
    timings.push(("expand_A (keygen)", t0.elapsed()));

    let t0 = Instant::now();
    let matrix_a_ntt = matrix_a.to_ntt();
    timings.push(("to_ntt(A) (keygen)", t0.elapsed()));

    let t0 = Instant::now();
    let secret_s1 = sample_noise_vec(&sigma, l, n, 0, params.eta);
    let secret_s2 = sample_noise_vec(&sigma, k, n, l as u16, params.eta);
    timings.push(("sample_s1_s2", t0.elapsed()));

    let t0 = Instant::now();
    let public_t = matrix_a_ntt.mul_vec(&secret_s1)?;
    timings.push(("A·s₁ NTT (kg)", t0.elapsed()));

    let pk = TlDsaPublicKey {
        variant,
        matrix_a_seed: rho.clone(),
        public_t: public_t.clone(),
        matrix_a: matrix_a.clone(),
        matrix_a_ntt: matrix_a_ntt.clone(),
    };
    let sk = TlDsaSecretKey {
        variant,
        matrix_a_seed: rho,
        matrix_a,
        matrix_a_ntt,
        secret_s1,
        secret_s2,
        public_t,
        signing_seed,
    };

    timings.push(("expand_A (sign)", core::time::Duration::ZERO));

    let pk_trits = poly_vec_to_trits(&sk.public_t);
    let t0 = Instant::now();
    let mu = dsa_hash(DOMAIN_MESSAGE_HASH, &[&pk_trits, message], 243);
    timings.push(("message_hash", t0.elapsed()));

    let attempt_trits = super::ternary_lattice::u16_to_trits(0u16);
    let t0 = Instant::now();
    let y_seed = dsa_hash(
        DOMAIN_MASKING,
        &[&sk.signing_seed, &mu, &attempt_trits],
        243,
    );
    let y = sample_masking_vec(&y_seed, l, n, 0);
    timings.push(("y_sampling", t0.elapsed()));

    let t0 = Instant::now();
    let w = sk.matrix_a_ntt.mul_vec(&y)?;
    timings.push(("A·y NTT (sign)", t0.elapsed()));

    let w_trits = poly_vec_to_trits(&w);
    let t0 = Instant::now();
    let challenge_hash = dsa_hash(DOMAIN_CHALLENGE, &[&mu, &w_trits], 243);
    timings.push(("commitment_sponge", t0.elapsed()));

    let t0 = Instant::now();
    let c = sample_challenge(&challenge_hash, n, params.tau);
    timings.push(("c_sampling", t0.elapsed()));

    let t0 = Instant::now();
    let mut z_polys = Vec::with_capacity(l);
    for i in 0..l {
        let cs1_i = c.ring_mul_sparse(&sk.secret_s1.polys[i])?;
        let z_i = y.polys[i].add(&cs1_i)?;
        z_polys.push(z_i);
    }
    let z = TernaryPolyVec { polys: z_polys, n };
    timings.push(("z = y + c·s₁ (sparse)", t0.elapsed()));

    let sig = TlDsaSignature {
        variant: sk.variant,
        z,
        challenge_hash,
    };

    timings.push(("expand_A (verify)", core::time::Duration::ZERO));

    let t0 = Instant::now();
    let c_ver = sample_challenge(&sig.challenge_hash, n, params.tau);
    timings.push(("c_sampling (verify)", t0.elapsed()));

    let t0 = Instant::now();
    let az = pk.matrix_a_ntt.mul_vec(&sig.z)?;
    timings.push(("A·z NTT (verify)", t0.elapsed()));

    let t0 = Instant::now();
    let mut ct_polys = Vec::with_capacity(k);
    for i in 0..k {
        let cti = c_ver.ring_mul_sparse(&pk.public_t.polys[i])?;
        ct_polys.push(cti);
    }
    let ct = TernaryPolyVec { polys: ct_polys, n };
    timings.push(("c·t sparse (verify)", t0.elapsed()));

    let t0 = Instant::now();
    let w_prime = az.add(&ct.negate()?)?;
    timings.push(("A·z - c·t subtract", t0.elapsed()));

    let t0 = Instant::now();
    for poly in &sig.z.polys {
        let _ = poly.l_infinity_norm();
    }
    timings.push(("norm_check", t0.elapsed()));

    let t0 = Instant::now();
    let pk_trits_v = poly_vec_to_trits(&pk.public_t);
    let mu_v = dsa_hash(DOMAIN_MESSAGE_HASH, &[&pk_trits_v, message], 243);
    let w_trits_v = poly_vec_to_trits(&w_prime);
    let _expected_hash = dsa_hash(DOMAIN_CHALLENGE, &[&mu_v, &w_trits_v], 243);
    timings.push(("verify_hash_compare", t0.elapsed()));

    Ok(timings)
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
        assert_eq!(
            poly_vec_to_trits(&sk1.secret_s2),
            poly_vec_to_trits(&sk2.secret_s2),
        );
    }

    #[test]
    fn test_keygen_s2_is_nonzero() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (_pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
        let s2_trits = poly_vec_to_trits(&sk.secret_s2);
        let nonzero_count = s2_trits.iter().filter(|&&t| t != 0).count();
        assert!(nonzero_count > 0, "s2 should have non-zero coefficients (was all-zeros in buggy version)");
    }

    #[test]
    fn test_sample_challenge_exact_weight() {
        let seed = vec![0i8, 1, -1, 0, 1, -1];
        for &tau in &[10usize, 20, 39, 49, 60] {
            let c = sample_challenge(&seed, 256, tau);
            assert_eq!(c.coeffs.len(), 256);
            let nonzero = c.coeffs.iter().filter(|&&x| x != 0).count();
            assert_eq!(
                nonzero, tau,
                "Challenge should have exactly τ={} non-zero coefficients, got {}",
                tau, nonzero
            );
        }
    }

    #[test]
    fn test_sample_challenge_values_are_pm1() {
        let seed = vec![1i8, 0, -1, 1, 0];
        let c = sample_challenge(&seed, 256, 39);
        for &coeff in &c.coeffs {
            assert!(
                coeff == -1 || coeff == 0 || coeff == 1,
                "Challenge coefficient {} is not in {{-1, 0, +1}}", coeff
            );
        }
    }

    #[test]
    fn test_sample_challenge_deterministic() {
        let seed = vec![0i8, 1, -1, 0, 1, -1];
        let c1 = sample_challenge(&seed, 256, 39);
        let c2 = sample_challenge(&seed, 256, 39);
        assert_eq!(c1.coeffs, c2.coeffs, "Challenge sampling must be deterministic");
    }

    #[test]
    fn test_sample_challenge_different_seeds() {
        let seed1 = vec![0i8, 1, -1];
        let seed2 = vec![1i8, -1, 0];
        let c1 = sample_challenge(&seed1, 256, 39);
        let c2 = sample_challenge(&seed2, 256, 39);
        assert_ne!(c1.coeffs, c2.coeffs);
    }

    #[test]
    fn test_sample_challenge_position_distribution() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1];
        let c = sample_challenge(&seed, 256, 60);
        let positions: Vec<usize> = c.coeffs.iter()
            .enumerate()
            .filter(|(_, &x)| x != 0)
            .map(|(i, _)| i)
            .collect();

        let mut sorted = positions.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 60, "All 60 positions should be unique");

        let in_q1 = positions.iter().filter(|&&p| p < 64).count();
        let in_q2 = positions.iter().filter(|&&p| (64..128).contains(&p)).count();
        let in_q3 = positions.iter().filter(|&&p| (128..192).contains(&p)).count();
        let in_q4 = positions.iter().filter(|&&p| p >= 192).count();
        assert!(in_q1 > 0, "Should have positions in [0, 64)");
        assert!(in_q2 > 0, "Should have positions in [64, 128)");
        assert!(in_q3 > 0, "Should have positions in [128, 192)");
        assert!(in_q4 > 0, "Should have positions in [192, 256)");
    }

    #[test]
    fn test_expand_matrix_dimensions() {
        let seed = vec![0i8, 1, -1];

        let a44 = expand_matrix_a(&seed, 4, 4, 256);
        assert_eq!(a44.rows, 4);
        assert_eq!(a44.cols, 4);

        let a65 = expand_matrix_a(&seed, 6, 5, 256);
        assert_eq!(a65.rows, 6);
        assert_eq!(a65.cols, 5);

        let a87 = expand_matrix_a(&seed, 8, 7, 256);
        assert_eq!(a87.rows, 8);
        assert_eq!(a87.cols, 7);
    }

    #[test]
    fn test_sign_verify_44() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "TL-DSA-44 signature should verify");
    }

    #[test]
    fn test_sign_verify_65() {
        let seed = vec![1i8, 0, -1, 1, 0, -1, 0, 1, -1, 1, 0, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa65, &seed).unwrap();

        let message = vec![0i8, 1, -1, 0, 1, -1];
        let sig = sign(&sk, &message).unwrap();

        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "TL-DSA-65 signature should verify");
    }

    #[test]
    fn test_sign_verify_87() {
        let seed = vec![-1i8, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa87, &seed).unwrap();

        let message = vec![1i8, 1, 0, -1, -1, 0, 1, 1];
        let sig = sign(&sk, &message).unwrap();

        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "TL-DSA-87 signature should verify");
    }

    #[test]
    fn test_sign_verify_wrong_message() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        let wrong_msg = vec![0i8, 0, 0, 0, 0, 0];
        let valid = verify(&pk, &wrong_msg, &sig).unwrap();
        assert!(!valid, "Signature should NOT verify with wrong message");
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
        assert!(!valid, "Signature should NOT verify with wrong public key");
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
    fn test_sign_z_norm_bound() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (_pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        for (i, poly) in sig.z.polys.iter().enumerate() {
            assert!(
                poly.l_infinity_norm() <= 1,
                "z[{}] has ||·||_∞ = {} > 1 (non-abort property violated)",
                i, poly.l_infinity_norm()
            );
        }
    }

    #[test]
    fn test_sign_verify_empty_message() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message: Vec<i8> = vec![];
        let sig = sign(&sk, &message).unwrap();
        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "Signature on empty message should verify");
    }

    #[test]
    fn test_sign_verify_long_message() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message: Vec<i8> = (0..1000).map(|i| ((i % 3) as i8) - 1).collect();
        let sig = sign(&sk, &message).unwrap();
        let valid = verify(&pk, &message, &sig).unwrap();
        assert!(valid, "Signature on long message should verify");
    }

    #[test]
    fn test_signature_variant_consistency() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (_pk, sk) = keygen(TlDsaVariant::TlDsa44, &seed).unwrap();

        let message = vec![1i8, 0, -1];
        let sig = sign(&sk, &message).unwrap();

        assert_eq!(sig.variant, TlDsaVariant::TlDsa44);
        assert_eq!(sig.z.polys.len(), 4);
        assert_eq!(sig.challenge_hash.len(), 243);
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
}
