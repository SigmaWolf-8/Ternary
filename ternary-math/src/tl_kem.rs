// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
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
//! Ported from the Salvi kernel (`src/kernel/src/crypto/tl_kem.rs`)
//! to use `ternary-math`'s TL-Sponge-385 and lattice modules.
//!
//! The `SharedSecret` type provides a `to_bytes_32()` method that produces
//! a 32-byte shared secret compatible with `sponge385_derive_key`'s
//! `kem_shared_secret: &[u8; 32]` parameter.

use crate::ternary_lattice::{
    TernaryPolynomial, TernaryPolyVec, LatticeParams, LatticeError,
    sample_matrix, sample_noise_vec, compress_ternary, decompress_ternary,
};
use crate::tlsponge385::Sponge385Pub;

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

    fn tag_byte(&self) -> u8 {
        match self {
            TlKemVariant::TlKem512 => 0x01,
            TlKemVariant::TlKem768 => 0x02,
            TlKemVariant::TlKem1024 => 0x03,
        }
    }

    fn from_tag_byte(b: u8) -> Result<Self, TlKemError> {
        match b {
            0x01 => Ok(TlKemVariant::TlKem512),
            0x02 => Ok(TlKemVariant::TlKem768),
            0x03 => Ok(TlKemVariant::TlKem1024),
            _ => Err(TlKemError::InvalidFormat),
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

impl TlKemPublicKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.variant.tag_byte());
        let seed_bytes = trits_to_bytes(&self.matrix_a_seed);
        out.extend_from_slice(&(seed_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&seed_bytes);
        let t_trits = poly_vec_to_trits(&self.public_vec_t);
        let t_bytes = trits_to_bytes(&t_trits);
        out.extend_from_slice(&(t_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&t_bytes);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError> {
        if data.is_empty() { return Err(TlKemError::InvalidFormat); }
        let variant = TlKemVariant::from_tag_byte(data[0])?;
        let params = variant.params();
        let mut pos = 1;

        let seed_len = read_u32_le(data, &mut pos)? as usize;
        if pos + seed_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let seed_bytes = &data[pos..pos + seed_len];
        let matrix_a_seed = bytes_to_trits(seed_bytes, 243);
        pos += seed_len;

        let t_len = read_u32_le(data, &mut pos)? as usize;
        if pos + t_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let t_bytes = &data[pos..pos + t_len];
        let t_trits = bytes_to_trits(t_bytes, params.k * params.n);
        let mut polys = Vec::with_capacity(params.k);
        for i in 0..params.k {
            let start = i * params.n;
            let end = start + params.n;
            polys.push(TernaryPolynomial::from_coeffs_unchecked(t_trits[start..end].to_vec()));
        }
        Ok(TlKemPublicKey {
            variant,
            matrix_a_seed,
            public_vec_t: TernaryPolyVec { polys, n: params.n },
        })
    }
}

#[derive(Debug, Clone)]
pub struct TlKemSecretKey {
    pub variant: TlKemVariant,
    pub secret_s: TernaryPolyVec,
    pub public_key: TlKemPublicKey,
    pub hash_pk: Vec<i8>,
    pub implicit_reject_seed: Vec<i8>,
}

impl TlKemSecretKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.variant.tag_byte());
        let s_trits = poly_vec_to_trits(&self.secret_s);
        let s_bytes = trits_to_bytes(&s_trits);
        out.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&s_bytes);
        let pk_bytes = self.public_key.to_bytes();
        out.extend_from_slice(&(pk_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&pk_bytes);
        let hash_bytes = trits_to_bytes(&self.hash_pk);
        out.extend_from_slice(&(hash_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&hash_bytes);
        let reject_bytes = trits_to_bytes(&self.implicit_reject_seed);
        out.extend_from_slice(&(reject_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&reject_bytes);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError> {
        if data.is_empty() { return Err(TlKemError::InvalidFormat); }
        let variant = TlKemVariant::from_tag_byte(data[0])?;
        let params = variant.params();
        let mut pos = 1;

        let s_len = read_u32_le(data, &mut pos)? as usize;
        if pos + s_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let s_trits = bytes_to_trits(&data[pos..pos + s_len], params.k * params.n);
        let mut s_polys = Vec::with_capacity(params.k);
        for i in 0..params.k {
            let start = i * params.n;
            let end = start + params.n;
            s_polys.push(TernaryPolynomial::from_coeffs_unchecked(s_trits[start..end].to_vec()));
        }
        let secret_s = TernaryPolyVec { polys: s_polys, n: params.n };
        pos += s_len;

        let pk_len = read_u32_le(data, &mut pos)? as usize;
        if pos + pk_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let public_key = TlKemPublicKey::from_bytes(&data[pos..pos + pk_len])?;
        pos += pk_len;

        let hash_len = read_u32_le(data, &mut pos)? as usize;
        if pos + hash_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let hash_pk = bytes_to_trits(&data[pos..pos + hash_len], 243);
        pos += hash_len;

        let reject_len = read_u32_le(data, &mut pos)? as usize;
        if pos + reject_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let implicit_reject_seed = bytes_to_trits(&data[pos..pos + reject_len], 243);

        Ok(TlKemSecretKey { variant, secret_s, public_key, hash_pk, implicit_reject_seed })
    }
}

#[derive(Debug, Clone)]
pub struct TlKemCiphertext {
    pub variant: TlKemVariant,
    pub compressed_u: Vec<Vec<u8>>,
    pub compressed_v: Vec<u8>,
}

impl TlKemCiphertext {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.variant.tag_byte());
        out.push(self.compressed_u.len() as u8);
        for cu in &self.compressed_u {
            out.extend_from_slice(&(cu.len() as u32).to_le_bytes());
            out.extend_from_slice(cu);
        }
        out.extend_from_slice(&(self.compressed_v.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.compressed_v);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, TlKemError> {
        if data.len() < 2 { return Err(TlKemError::InvalidFormat); }
        let variant = TlKemVariant::from_tag_byte(data[0])?;
        let k = data[1] as usize;
        let mut pos = 2;
        let mut compressed_u = Vec::with_capacity(k);
        for _ in 0..k {
            let cu_len = read_u32_le(data, &mut pos)? as usize;
            if pos + cu_len > data.len() { return Err(TlKemError::InvalidFormat); }
            compressed_u.push(data[pos..pos + cu_len].to_vec());
            pos += cu_len;
        }
        let cv_len = read_u32_le(data, &mut pos)? as usize;
        if pos + cv_len > data.len() { return Err(TlKemError::InvalidFormat); }
        let compressed_v = data[pos..pos + cv_len].to_vec();
        Ok(TlKemCiphertext { variant, compressed_u, compressed_v })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecret {
    pub trits: Vec<i8>,
}

impl SharedSecret {
    pub fn to_bytes_32(&self) -> [u8; 32] {
        let domain = b"TL-KEM-SharedSecret-v1";
        let hash = crate::tlsponge385::derive_key(domain, &trits_to_bytes(&self.trits), 32);
        let mut result = [0u8; 32];
        let len = hash.len().min(32);
        result[..len].copy_from_slice(&hash[..len]);
        result
    }
}

#[derive(Debug)]
pub enum TlKemError {
    Lattice(LatticeError),
    InvalidSeed,
    InvalidFormat,
}

impl std::fmt::Display for TlKemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlKemError::Lattice(e) => write!(f, "Lattice error: {}", e),
            TlKemError::InvalidSeed => write!(f, "Invalid seed"),
            TlKemError::InvalidFormat => write!(f, "Invalid serialized format"),
        }
    }
}

impl std::error::Error for TlKemError {}

impl From<LatticeError> for TlKemError {
    fn from(e: LatticeError) -> Self {
        TlKemError::Lattice(e)
    }
}

fn trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for chunk in trits.chunks(5) {
        let mut val: u16 = 0;
        for (j, &t) in chunk.iter().enumerate() {
            let b_val = (t as i16 + 1) as u16;
            val += b_val * 3u16.pow(j as u32);
        }
        bytes.push(val as u8);
    }
    bytes
}

fn bytes_to_trits(bytes: &[u8], trit_count: usize) -> Vec<i8> {
    let mut trits = Vec::with_capacity(trit_count);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            if trits.len() >= trit_count { break; }
            let remainder = val % 3;
            trits.push(remainder as i8 - 1);
            val /= 3;
        }
    }
    trits.truncate(trit_count);
    trits
}

fn read_u32_le(data: &[u8], pos: &mut usize) -> Result<u32, TlKemError> {
    if *pos + 4 > data.len() { return Err(TlKemError::InvalidFormat); }
    let val = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    Ok(val)
}

fn kem_hash(inputs: &[&[i8]], output_len: usize) -> Vec<i8> {
    let mut sponge = Sponge385Pub::new();
    for input in inputs {
        sponge.absorb(input);
    }
    sponge.squeeze(output_len)
}

fn generate_message(seed: &[i8]) -> Vec<i8> {
    kem_hash(&[seed, &[0i8, 1, -1]], KEM_MESSAGE_TRITS)
}

fn serialize_ciphertext_bytes(ct: &TlKemCiphertext) -> Vec<u8> {
    let mut out = Vec::new();
    for cu in &ct.compressed_u {
        out.extend_from_slice(cu);
    }
    out.extend_from_slice(&ct.compressed_v);
    out
}

fn ciphertext_to_trits_for_kdf(ct_bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(ct_bytes.len());
    for &b in ct_bytes {
        trits.push(((b % 3) as i8) - 1);
    }
    trits
}

pub fn keygen(variant: TlKemVariant, seed: &[i8]) -> Result<(TlKemPublicKey, TlKemSecretKey), TlKemError> {
    let params = variant.params();
    let k = params.k;
    let n = params.n;

    let rho = kem_hash(&[seed, &[0i8]], 243);
    let sigma = kem_hash(&[seed, &[1i8]], 243);

    let matrix_a = sample_matrix(&rho, k, n);
    let secret_s = sample_noise_vec(&sigma, k, n, 0, params.eta1);

    let public_t = matrix_a.to_ntt().mul_vec(&secret_s)?;

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

pub fn encapsulate(pk: &TlKemPublicKey, randomness: &[i8]) -> Result<(TlKemCiphertext, SharedSecret), TlKemError> {
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

    let a_t = matrix_a.transpose();
    let u = a_t.to_ntt().mul_vec(&r)?;

    let t_dot_r = pk.public_vec_t.inner_product(&r)?;
    let msg_poly = message_to_polynomial(&message, n);
    let v = t_dot_r.add(&msg_poly)?;

    let compressed_u: Vec<Vec<u8>> = u.polys.iter()
        .map(|p| compress_ternary(p, params.du))
        .collect();
    let compressed_v = compress_ternary(&v, params.dv);

    let ct = TlKemCiphertext {
        variant: pk.variant,
        compressed_u,
        compressed_v,
    };

    let ct_bytes = serialize_ciphertext_bytes(&ct);
    let ct_trits = ciphertext_to_trits_for_kdf(&ct_bytes);
    let shared_trits = kem_hash(
        &[shared_key_seed, &ct_trits],
        pk.variant.shared_secret_trits(),
    );

    let shared = SharedSecret { trits: shared_trits };

    Ok((ct, shared))
}

pub fn decapsulate(sk: &TlKemSecretKey, ct: &TlKemCiphertext) -> Result<SharedSecret, TlKemError> {
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

    let ct_bytes = serialize_ciphertext_bytes(ct);
    let ct_prime_bytes = serialize_ciphertext_bytes(&ct_prime);

    let match_flag = ct_eq_byte_slices(&ct_bytes, &ct_prime_bytes);
    let match_bit = match_flag & 1;

    let ct_trits = ciphertext_to_trits_for_kdf(&ct_bytes);
    let ss_accept = kem_hash(
        &[shared_key_seed, &ct_trits],
        sk.variant.shared_secret_trits(),
    );
    let ss_reject = kem_hash(
        &[&sk.implicit_reject_seed, &ct_trits],
        sk.variant.shared_secret_trits(),
    );

    let shared_trits = ct_select_vec(match_bit, &ss_accept, &ss_reject);

    Ok(SharedSecret { trits: shared_trits })
}

fn encapsulate_inner(
    pk: &TlKemPublicKey,
    message: &[i8],
    coins: &[i8],
) -> Result<(TlKemCiphertext, Vec<i8>), TlKemError> {
    let params = pk.variant.params();
    let k = params.k;
    let n = params.n;

    let matrix_a = sample_matrix(&pk.matrix_a_seed, k, n);
    let r = sample_noise_vec(coins, k, n, 0, params.eta1);

    let a_t = matrix_a.transpose();
    let u = a_t.to_ntt().mul_vec(&r)?;

    let t_dot_r = pk.public_vec_t.inner_product(&r)?;
    let msg_poly = message_to_polynomial(message, n);
    let v = t_dot_r.add(&msg_poly)?;

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

#[inline(always)]
fn ct_eq_u8(a: u8, b: u8) -> u8 {
    let x = a ^ b;
    let x16 = x as u16;
    let neg = x16.wrapping_sub(1);
    (neg >> 8) as u8
}

fn ct_eq_byte_slices(a: &[u8], b: &[u8]) -> u8 {
    if a.len() != b.len() {
        return 0;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    ct_eq_u8(diff, 0)
}

fn ct_select_vec(condition: u8, if_true: &[i8], if_false: &[i8]) -> Vec<i8> {
    let len = if_true.len().min(if_false.len());
    let mask = 0u8.wrapping_sub(condition & 1) as i8;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        result.push((mask & if_true[i]) | (!mask & if_false[i]));
    }
    result
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

    #[test]
    fn test_shared_secret_to_bytes_32() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, _) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let (_, shared) = encapsulate(&pk, &randomness).unwrap();

        let bytes = shared.to_bytes_32();
        assert_eq!(bytes.len(), 32);

        let bytes2 = shared.to_bytes_32();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_shared_secret_to_bytes_32_different_secrets() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1];
        let (pk, _) = keygen(TlKemVariant::TlKem512, &seed).unwrap();

        let (_, s1) = encapsulate(&pk, &[0i8, 0, 0, 0, 0]).unwrap();
        let (_, s2) = encapsulate(&pk, &[1i8, 0, 0, 0, 0]).unwrap();

        assert_ne!(s1.to_bytes_32(), s2.to_bytes_32());
    }

    #[test]
    fn test_shared_secret_compat_sponge385_derive_key() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let (ct, shared1) = encapsulate(&pk, &randomness).unwrap();
        let shared2 = decapsulate(&sk, &ct).unwrap();

        let kem_secret = shared1.to_bytes_32();
        let kem_secret2 = shared2.to_bytes_32();
        assert_eq!(kem_secret, kem_secret2);

        let domain = b"PlenumNET-CON-v3.0";
        let addr_a = b"addr_a_test";
        let addr_b = b"addr_b_test";
        let epoch: u64 = 42;
        let key1 = crate::tlsponge385::sponge385_derive_key(domain, addr_a, addr_b, &kem_secret, epoch);
        let key2 = crate::tlsponge385::sponge385_derive_key(domain, addr_a, addr_b, &kem_secret2, epoch);
        assert_eq!(key1, key2);
        assert!(!key1.is_empty());
    }

    #[test]
    fn test_public_key_serialization_roundtrip() {
        for variant in [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024] {
            let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
            let (pk, _) = keygen(variant, &seed).unwrap();
            let bytes = pk.to_bytes();
            let pk2 = TlKemPublicKey::from_bytes(&bytes).unwrap();
            assert_eq!(pk.variant, pk2.variant);
            assert_eq!(pk.matrix_a_seed, pk2.matrix_a_seed);
            assert_eq!(
                poly_vec_to_trits(&pk.public_vec_t),
                poly_vec_to_trits(&pk2.public_vec_t)
            );
        }
    }

    #[test]
    fn test_secret_key_serialization_roundtrip() {
        for variant in [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024] {
            let seed = vec![1i8, 0, -1, 0, 1, -1, 0, 1];
            let (_, sk) = keygen(variant, &seed).unwrap();
            let bytes = sk.to_bytes();
            let sk2 = TlKemSecretKey::from_bytes(&bytes).unwrap();
            assert_eq!(sk.variant, sk2.variant);
            assert_eq!(
                poly_vec_to_trits(&sk.secret_s),
                poly_vec_to_trits(&sk2.secret_s)
            );
            assert_eq!(sk.hash_pk, sk2.hash_pk);
            assert_eq!(sk.implicit_reject_seed, sk2.implicit_reject_seed);
        }
    }

    #[test]
    fn test_ciphertext_serialization_roundtrip() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, _) = keygen(TlKemVariant::TlKem512, &seed).unwrap();
        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let (ct, _) = encapsulate(&pk, &randomness).unwrap();

        let bytes = ct.to_bytes();
        let ct2 = TlKemCiphertext::from_bytes(&bytes).unwrap();
        assert_eq!(ct.variant, ct2.variant);
        assert_eq!(ct.compressed_u, ct2.compressed_u);
        assert_eq!(ct.compressed_v, ct2.compressed_v);
    }

    #[test]
    fn test_serialized_key_encapsulate_decapsulate() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let (pk, sk) = keygen(TlKemVariant::TlKem512, &seed).unwrap();

        let pk_bytes = pk.to_bytes();
        let sk_bytes = sk.to_bytes();
        let pk2 = TlKemPublicKey::from_bytes(&pk_bytes).unwrap();
        let sk2 = TlKemSecretKey::from_bytes(&sk_bytes).unwrap();

        let randomness = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let (ct, shared1) = encapsulate(&pk2, &randomness).unwrap();

        let ct_bytes = ct.to_bytes();
        let ct2 = TlKemCiphertext::from_bytes(&ct_bytes).unwrap();

        let shared2 = decapsulate(&sk2, &ct2).unwrap();
        assert_eq!(shared1, shared2);
    }

    #[test]
    fn test_ct_eq_byte_slices() {
        let a: Vec<u8> = vec![0, 1, 2, 3, 4];
        let b: Vec<u8> = vec![0, 1, 2, 3, 4];
        assert_ne!(ct_eq_byte_slices(&a, &b), 0);

        let c: Vec<u8> = vec![0, 1, 2, 3, 5];
        assert_eq!(ct_eq_byte_slices(&a, &c), 0);

        let d: Vec<u8> = vec![0, 1, 2];
        assert_eq!(ct_eq_byte_slices(&a, &d), 0);
    }

    #[test]
    fn test_ct_select_vec() {
        let a = vec![1i8, 0, -1];
        let b = vec![-1i8, 1, 0];

        let r1 = ct_select_vec(1, &a, &b);
        assert_eq!(r1, a);

        let r0 = ct_select_vec(0, &a, &b);
        assert_eq!(r0, b);
    }

    #[test]
    fn test_invalid_format_deserialization() {
        assert!(TlKemPublicKey::from_bytes(&[]).is_err());
        assert!(TlKemPublicKey::from_bytes(&[0xFF]).is_err());
        assert!(TlKemSecretKey::from_bytes(&[]).is_err());
        assert!(TlKemCiphertext::from_bytes(&[]).is_err());
        assert!(TlKemCiphertext::from_bytes(&[0x01]).is_err());
    }

    #[test]
    fn test_trits_bytes_roundtrip() {
        let trits = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0];
        let bytes = trits_to_bytes(&trits);
        let recovered = bytes_to_trits(&bytes, trits.len());
        assert_eq!(trits, recovered);
    }
}
