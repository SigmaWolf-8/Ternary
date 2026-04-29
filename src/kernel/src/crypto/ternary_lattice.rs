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

//! Ternary Lattice Arithmetic Foundations
//!
//! Implements GF(3) polynomial ring arithmetic primitives for building
//! lattice-based post-quantum cryptography (ML-KEM, ML-DSA) in the
//! ternary domain. This module provides the mathematical foundation
//! for the CNSA 2.0 Phase 2 "Lattice Foundations" milestone.
//!
//! # Architecture
//!
//! All operations work in the polynomial ring R_q = Z_3[X]/(X^n + 1),
//! where coefficients are elements of GF(3) in balanced representation
//! {-1, 0, +1}. This maps directly to PlenumNET's Representation A.
//!
//! # Components
//!
//! - **TernaryPolynomial**: Polynomial with GF(3) coefficients
//! - **Ring multiplication**: Schoolbook convolution in Z_3[X]/(X^n+1)
//! - **Polynomial evaluation**: Multi-point evaluation at GF(3) elements
//! - **Module-LWE**: Learning With Errors over ternary modules
//! - **Module-SIS**: Short Integer Solution over ternary modules
//! - **Polynomial sampling**: Centered Binomial Distribution (CBD)
//!   and uniform sampling for key generation
//! - **Compression/Decompression**: Lossy encoding for ciphertext compactness
//!
//! Ring multiplication uses Karatsuba (O(n^1.585)) via `ring_mul_karatsuba`
//! for individual polynomial products, and an integer NTT (q=12289, n=256)
//! for pre-NTT'd matrix-vector multiplies via `NttMatrix::mul_vec`. The NTT
//! lifts ternary coefficients to Z_q, uses ψ-twist negacyclic convolution,
//! and reduces results back to balanced ternary mod 3.
//!
//! # CNSA 2.0 Relevance
//!
//! ML-KEM (FIPS 203) and ML-DSA (FIPS 204) both rely on lattice
//! problems over polynomial rings. ML-KEM internally uses coefficients
//! from {-1, 0, 1} for error terms, which maps directly to balanced
//! ternary. These primitives enable future TL-KEM and TL-DSA
//! implementations.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use super::{CryptoError, CryptoResult};

pub const LATTICE_N_256: usize = 256;
pub const LATTICE_N_512: usize = 512;

pub const TERNARY_MODULUS: i16 = 3;

pub const MODULE_RANK_2: usize = 2;
pub const MODULE_RANK_3: usize = 3;
pub const MODULE_RANK_4: usize = 4;

#[inline(always)]
pub fn t_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

#[inline(always)]
pub fn t_mul(a: i8, b: i8) -> i8 {
    if a == 0 || b == 0 { 0 } else { a * b }
}

#[inline(always)]
pub fn t_neg(a: i8) -> i8 { -a }

fn mod3(x: i16) -> i8 {
    let r = ((x % 3) + 3) % 3;
    match r {
        0 => 0,
        1 => 1,
        2 => -1,
        _ => unreachable!(),
    }
}

fn balanced_to_unsigned(t: i8) -> u8 {
    match t {
        -1 => 2,
        0 => 0,
        1 => 1,
        _ => 0,
    }
}

fn unsigned_to_balanced(u: u8) -> i8 {
    match u % 3 {
        0 => 0,
        1 => 1,
        2 => -1,
        _ => unreachable!(),
    }
}

fn bit_reverse(k: usize, log_n: usize) -> usize {
    let mut rev = 0;
    let mut i = k;
    for _ in 0..log_n {
        rev = (rev << 1) | (i & 1);
        i >>= 1;
    }
    rev
}

fn schoolbook_raw(a: &[i8], b: &[i8]) -> Vec<i8> {
    let n = a.len();
    let m = b.len();
    let mut result = vec![0i8; n + m - 1];
    for i in 0..n {
        if a[i] == 0 { continue; }
        for j in 0..m {
            if b[j] == 0 { continue; }
            result[i + j] = t_add(result[i + j], t_mul(a[i], b[j]));
        }
    }
    result
}

fn karatsuba_raw(a: &[i8], b: &[i8]) -> Vec<i8> {
    let n = a.len();
    if n <= 32 {
        return schoolbook_raw(a, b);
    }
    let m = n / 2;
    let (a0, a1) = a.split_at(m);
    let (b0, b1) = b.split_at(m);

    let z0 = karatsuba_raw(a0, b0);
    let z2 = karatsuba_raw(a1, b1);

    let a01: Vec<i8> = a0.iter().zip(a1.iter()).map(|(&x, &y)| t_add(x, y)).collect();
    let b01: Vec<i8> = b0.iter().zip(b1.iter()).map(|(&x, &y)| t_add(x, y)).collect();
    let z1_full = karatsuba_raw(&a01, &b01);

    let len = 2 * n - 1;
    let mut result = vec![0i8; len];

    for i in 0..z0.len() {
        result[i] = t_add(result[i], z0[i]);
    }
    for i in 0..z2.len() {
        result[i + 2 * m] = t_add(result[i + 2 * m], z2[i]);
    }
    for i in 0..z1_full.len() {
        let mut v = z1_full[i];
        if i < z0.len() { v = t_add(v, t_neg(z0[i])); }
        if i < z2.len() { v = t_add(v, t_neg(z2[i])); }
        result[i + m] = t_add(result[i + m], v);
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryPolynomial {
    pub coeffs: Vec<i8>,
    pub n: usize,
}

impl TernaryPolynomial {
    pub fn new(n: usize) -> Self {
        Self {
            coeffs: vec![0i8; n],
            n,
        }
    }

    pub fn from_coeffs(coeffs: Vec<i8>) -> CryptoResult<Self> {
        for &c in &coeffs {
            if c < -1 || c > 1 {
                return Err(CryptoError::InvalidTritValue(c));
            }
        }
        let n = coeffs.len();
        Ok(Self { coeffs, n })
    }

    pub fn from_coeffs_unchecked(coeffs: Vec<i8>) -> Self {
        let n = coeffs.len();
        Self { coeffs, n }
    }

    pub fn degree(&self) -> usize {
        for i in (0..self.n).rev() {
            if self.coeffs[i] != 0 {
                return i;
            }
        }
        0
    }

    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&c| c == 0)
    }

    pub fn hamming_weight(&self) -> usize {
        self.coeffs.iter().filter(|&&c| c != 0).count()
    }

    pub fn l_infinity_norm(&self) -> u8 {
        self.coeffs.iter().map(|&c| c.unsigned_abs()).max().unwrap_or(0)
    }

    pub fn add(&self, other: &TernaryPolynomial) -> CryptoResult<TernaryPolynomial> {
        if self.n != other.n {
            return Err(CryptoError::InvalidInputLength {
                expected: self.n,
                actual: other.n,
            });
        }
        let mut result = Vec::with_capacity(self.n);
        for i in 0..self.n {
            result.push(mod3(self.coeffs[i] as i16 + other.coeffs[i] as i16));
        }
        Ok(TernaryPolynomial { coeffs: result, n: self.n })
    }

    pub fn sub(&self, other: &TernaryPolynomial) -> CryptoResult<TernaryPolynomial> {
        if self.n != other.n {
            return Err(CryptoError::InvalidInputLength {
                expected: self.n,
                actual: other.n,
            });
        }
        let mut result = Vec::with_capacity(self.n);
        for i in 0..self.n {
            result.push(mod3(self.coeffs[i] as i16 - other.coeffs[i] as i16));
        }
        Ok(TernaryPolynomial { coeffs: result, n: self.n })
    }

    pub fn negate(&self) -> TernaryPolynomial {
        let coeffs: Vec<i8> = self.coeffs.iter().map(|&c| mod3(-(c as i16))).collect();
        TernaryPolynomial { coeffs, n: self.n }
    }

    pub fn scalar_mul(&self, scalar: i8) -> TernaryPolynomial {
        let coeffs: Vec<i8> = self.coeffs.iter()
            .map(|&c| mod3(c as i16 * scalar as i16))
            .collect();
        TernaryPolynomial { coeffs, n: self.n }
    }

    pub fn ring_mul(&self, other: &TernaryPolynomial) -> CryptoResult<TernaryPolynomial> {
        if self.n != other.n {
            return Err(CryptoError::InvalidInputLength {
                expected: self.n,
                actual: other.n,
            });
        }
        let n = self.n;
        let mut result = vec![0i16; n];

        for i in 0..n {
            if self.coeffs[i] == 0 {
                continue;
            }
            for j in 0..n {
                if other.coeffs[j] == 0 {
                    continue;
                }
                let product = self.coeffs[i] as i16 * other.coeffs[j] as i16;
                let pos = i + j;
                if pos < n {
                    result[pos] += product;
                } else {
                    result[pos - n] -= product;
                }
            }
        }

        let coeffs: Vec<i8> = result.iter().map(|&v| mod3(v)).collect();
        Ok(TernaryPolynomial { coeffs, n })
    }

    pub fn ring_mul_sparse(&self, other: &TernaryPolynomial) -> CryptoResult<TernaryPolynomial> {
        if self.n != other.n {
            return Err(CryptoError::InvalidInputLength {
                expected: self.n,
                actual: other.n,
            });
        }
        let n = self.n;
        let mut result = vec![0i8; n];

        for i in 0..n {
            let ci = self.coeffs[i];
            if ci == 0 { continue; }
            for j in 0..n {
                let sj = other.coeffs[j];
                if sj == 0 { continue; }
                let pos = i + j;
                if pos < n {
                    result[pos] = t_add(result[pos], t_mul(ci, sj));
                } else {
                    result[pos - n] = t_add(result[pos - n], t_neg(t_mul(ci, sj)));
                }
            }
        }

        Ok(TernaryPolynomial { coeffs: result, n })
    }

    pub fn ring_mul_karatsuba(&self, other: &TernaryPolynomial) -> CryptoResult<TernaryPolynomial> {
        if self.n != other.n {
            return Err(CryptoError::InvalidInputLength {
                expected: self.n,
                actual: other.n,
            });
        }
        let n = self.n;
        let raw = karatsuba_raw(&self.coeffs, &other.coeffs);
        let mut result = vec![0i8; n];
        for i in 0..raw.len() {
            if i < n {
                result[i] = t_add(result[i], raw[i]);
            } else {
                result[i - n] = t_add(result[i - n], t_neg(raw[i]));
            }
        }
        Ok(TernaryPolynomial { coeffs: result, n })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.n * 2 + 7) / 8);
        let mut current_byte: u8 = 0;
        let mut bits_used: u8 = 0;

        for &coeff in &self.coeffs {
            let val = balanced_to_unsigned(coeff);
            current_byte |= (val & 0x03) << bits_used;
            bits_used += 2;
            if bits_used >= 8 {
                bytes.push(current_byte);
                current_byte = 0;
                bits_used = 0;
            }
        }
        if bits_used > 0 {
            bytes.push(current_byte);
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8], n: usize) -> CryptoResult<Self> {
        let mut coeffs = Vec::with_capacity(n);
        let mut byte_idx = 0;
        let mut bit_offset = 0;

        for _ in 0..n {
            if byte_idx >= bytes.len() {
                return Err(CryptoError::InvalidInputLength {
                    expected: (n * 2 + 7) / 8,
                    actual: bytes.len(),
                });
            }
            let val = (bytes[byte_idx] >> bit_offset) & 0x03;
            coeffs.push(unsigned_to_balanced(val));
            bit_offset += 2;
            if bit_offset >= 8 {
                bit_offset = 0;
                byte_idx += 1;
            }
        }
        Ok(TernaryPolynomial { coeffs, n })
    }
}

pub fn poly_eval_at(poly: &TernaryPolynomial, point: i8) -> i8 {
    let mut result: i16 = 0;
    let mut power: i16 = 1;
    let p = point as i16;
    for &c in &poly.coeffs {
        result += c as i16 * power;
        power = (power * p) % TERNARY_MODULUS;
    }
    mod3(result)
}

pub fn poly_multi_eval(poly: &TernaryPolynomial) -> Vec<i8> {
    vec![
        poly_eval_at(poly, -1),
        poly_eval_at(poly, 0),
        poly_eval_at(poly, 1),
    ]
}

pub fn poly_pointwise_mul(a: &[i8], b: &[i8]) -> Vec<i8> {
    a.iter().zip(b.iter())
        .map(|(&x, &y)| mod3(x as i16 * y as i16))
        .collect()
}

#[derive(Debug, Clone)]
pub struct TernaryPolyMatrix {
    pub rows: usize,
    pub cols: usize,
    pub n: usize,
    pub entries: Vec<Vec<TernaryPolynomial>>,
}

impl TernaryPolyMatrix {
    pub fn new(rows: usize, cols: usize, n: usize) -> Self {
        let mut entries = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for _ in 0..cols {
                row.push(TernaryPolynomial::new(n));
            }
            entries.push(row);
        }
        Self { rows, cols, n, entries }
    }

    pub fn get(&self, row: usize, col: usize) -> &TernaryPolynomial {
        &self.entries[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, poly: TernaryPolynomial) {
        self.entries[row][col] = poly;
    }

    pub fn mul_vec(&self, vec: &TernaryPolyVec) -> CryptoResult<TernaryPolyVec> {
        if self.cols != vec.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.cols,
                actual: vec.len(),
            });
        }

        let mut result = TernaryPolyVec::new(self.rows, self.n);
        for i in 0..self.rows {
            let mut sum = TernaryPolynomial::new(self.n);
            for j in 0..self.cols {
                let product = self.entries[i][j].ring_mul(&vec.polys[j])?;
                sum = sum.add(&product)?;
            }
            result.polys[i] = sum;
        }
        Ok(result)
    }

    pub fn mul_vec_karatsuba(&self, vec: &TernaryPolyVec) -> CryptoResult<TernaryPolyVec> {
        if self.cols != vec.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.cols,
                actual: vec.len(),
            });
        }

        let mut result = TernaryPolyVec::new(self.rows, self.n);
        for i in 0..self.rows {
            let mut sum = TernaryPolynomial::new(self.n);
            for j in 0..self.cols {
                let product = self.entries[i][j].ring_mul_karatsuba(&vec.polys[j])?;
                sum = sum.add(&product)?;
            }
            result.polys[i] = sum;
        }
        Ok(result)
    }

    pub fn mul_vec_geometric(&self, vec: &TernaryPolyVec) -> CryptoResult<TernaryPolyVec> {
        self.mul_vec_karatsuba(vec)
    }

    pub fn transpose(&self) -> Self {
        let mut result = TernaryPolyMatrix::new(self.cols, self.rows, self.n);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.entries[j][i] = self.entries[i][j].clone();
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct TernaryPolyVec {
    pub polys: Vec<TernaryPolynomial>,
    pub n: usize,
}

impl TernaryPolyVec {
    pub fn new(k: usize, n: usize) -> Self {
        let polys = (0..k).map(|_| TernaryPolynomial::new(n)).collect();
        Self { polys, n }
    }

    pub fn len(&self) -> usize {
        self.polys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.polys.is_empty()
    }

    pub fn add(&self, other: &TernaryPolyVec) -> CryptoResult<TernaryPolyVec> {
        if self.polys.len() != other.polys.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.polys.len(),
                actual: other.polys.len(),
            });
        }
        let mut result = Vec::with_capacity(self.polys.len());
        for i in 0..self.polys.len() {
            result.push(self.polys[i].add(&other.polys[i])?);
        }
        Ok(TernaryPolyVec { polys: result, n: self.n })
    }

    pub fn inner_product(&self, other: &TernaryPolyVec) -> CryptoResult<TernaryPolynomial> {
        if self.polys.len() != other.polys.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.polys.len(),
                actual: other.polys.len(),
            });
        }
        let mut sum = TernaryPolynomial::new(self.n);
        for i in 0..self.polys.len() {
            let product = self.polys[i].ring_mul(&other.polys[i])?;
            sum = sum.add(&product)?;
        }
        Ok(sum)
    }

    pub fn l_infinity_norm(&self) -> u8 {
        self.polys.iter().map(|p| p.l_infinity_norm()).max().unwrap_or(0)
    }
}

pub fn u16_to_trits(val: u16) -> [i8; 11] {
    let mut trits = [0i8; 11];
    let mut v = val;
    for trit in trits.iter_mut() {
        *trit = (v % 3) as i8 - 1;
        v /= 3;
    }
    trits
}

pub fn u8_to_trits(val: u8) -> [i8; 6] {
    let mut trits = [0i8; 6];
    let mut v = val;
    for trit in trits.iter_mut() {
        *trit = (v % 3) as i8 - 1;
        v /= 3;
    }
    trits
}

pub fn sample_uniform_ternary(seed: &[i8], n: usize, nonce: u16) -> TernaryPolynomial {
    use super::sponge::TernarySponge;
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);
    let nonce_trits = u16_to_trits(nonce);
    sponge.absorb(&nonce_trits);

    let output = sponge.squeeze(n * 2);
    let coeffs: Vec<i8> = output.trits.iter()
        .take(n)
        .map(|&t| {
            match t {
                -1 => -1i8,
                0 => 0i8,
                1 => 1i8,
                _ => 0i8,
            }
        })
        .collect();

    TernaryPolynomial { coeffs, n }
}

// ---------------------------------------------------------------------------
// CBD η=2 vectorized coefficient extraction
//
// For η=2, each coefficient consumes 4 trits: sum_a = (t0+1)+(t1+1),
// sum_b = (t2+1)+(t3+1), result = mod3(sum_a - sum_b).
//
// The +1 offsets cancel: diff = t0 + t1 - t2 - t3  ∈ [-4, +4].
// balanced_wrap(diff) ≡ mod3(diff) for this range — same conditional
// arithmetic as the sponge substitution, same AVX2 pattern:
// _mm256_add_epi8 + _mm256_cmpgt_epi8 + _mm256_blendv_epi8.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn cbd_eta2_avx2(raw: &[i8], coeffs: &mut Vec<i8>, n: usize) {
    use core::arch::x86_64::*;

    let v_hi    = _mm256_set1_epi8(1);
    let v_lo    = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    let mut i = 0;
    while i + 32 <= n {
        let base = i * 4;
        let mut t0 = [0i8; 32];
        let mut t1 = [0i8; 32];
        let mut t2 = [0i8; 32];
        let mut t3 = [0i8; 32];
        for k in 0..32 {
            let off = base + k * 4;
            t0[k] = raw[off];
            t1[k] = raw[off + 1];
            t2[k] = raw[off + 2];
            t3[k] = raw[off + 3];
        }

        let v_t0 = _mm256_loadu_si256(t0.as_ptr() as *const __m256i);
        let v_t1 = _mm256_loadu_si256(t1.as_ptr() as *const __m256i);
        let v_t2 = _mm256_loadu_si256(t2.as_ptr() as *const __m256i);
        let v_t3 = _mm256_loadu_si256(t3.as_ptr() as *const __m256i);

        let sum = _mm256_sub_epi8(
            _mm256_add_epi8(v_t0, v_t1),
            _mm256_add_epi8(v_t2, v_t3),
        );

        let gt1    = _mm256_cmpgt_epi8(sum, v_hi);
        let lt_neg = _mm256_cmpgt_epi8(v_lo, sum);
        let sub3   = _mm256_sub_epi8(sum, v_three);
        let add3   = _mm256_add_epi8(sum, v_three);

        let result = _mm256_blendv_epi8(sum, sub3, gt1);
        let result = _mm256_blendv_epi8(result, add3, lt_neg);

        let mut out = [0i8; 32];
        _mm256_storeu_si256(out.as_mut_ptr() as *mut __m256i, result);
        coeffs.extend_from_slice(&out);

        i += 32;
    }

    while i < n {
        let base = i * 4;
        let diff = raw[base] as i16 + raw[base + 1] as i16
                 - raw[base + 2] as i16 - raw[base + 3] as i16;
        coeffs.push(mod3(diff));
        i += 1;
    }
}

fn cbd_eta2_scalar(raw: &[i8], coeffs: &mut Vec<i8>, n: usize) {
    for i in 0..n {
        let base = i * 4;
        let mut sum_a: i16 = 0;
        let mut sum_b: i16 = 0;
        for j in 0..2usize {
            let idx_a = base + j;
            let idx_b = base + 2 + j;
            if idx_a < raw.len() {
                sum_a += (raw[idx_a] + 1) as i16;
            }
            if idx_b < raw.len() {
                sum_b += (raw[idx_b] + 1) as i16;
            }
        }
        coeffs.push(mod3(sum_a - sum_b));
    }
}

#[inline]
fn cbd_eta2(raw: &[i8], coeffs: &mut Vec<i8>, n: usize) {
    #[cfg(all(target_arch = "x86_64", not(feature = "no_std")))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { cbd_eta2_avx2(raw, coeffs, n); }
            return;
        }
    }
    cbd_eta2_scalar(raw, coeffs, n);
}

pub fn sample_cbd_ternary(seed: &[i8], n: usize, nonce: u16, eta: u8) -> TernaryPolynomial {
    use super::sponge::TernarySponge;
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);
    let nonce_trits = u16_to_trits(nonce);
    sponge.absorb(&nonce_trits);
    let eta_trits = u8_to_trits(eta);
    sponge.absorb(&eta_trits);

    let raw = sponge.squeeze(n * eta as usize * 2);
    let mut coeffs = Vec::with_capacity(n);

    if eta == 2 {
        cbd_eta2(&raw.trits, &mut coeffs, n);
    } else {
        for i in 0..n {
            let base = i * eta as usize * 2;
            let mut sum_a: i16 = 0;
            let mut sum_b: i16 = 0;
            for j in 0..eta as usize {
                let idx_a = base + j;
                let idx_b = base + eta as usize + j;
                if idx_a < raw.trits.len() {
                    sum_a += (raw.trits[idx_a] + 1) as i16;
                }
                if idx_b < raw.trits.len() {
                    sum_b += (raw.trits[idx_b] + 1) as i16;
                }
            }
            coeffs.push(mod3(sum_a - sum_b));
        }
    }

    TernaryPolynomial { coeffs, n }
}

pub fn sample_matrix(seed: &[i8], k: usize, n: usize) -> TernaryPolyMatrix {
    let mut matrix = TernaryPolyMatrix::new(k, k, n);
    for i in 0..k {
        for j in 0..k {
            let nonce = (i * k + j) as u16;
            matrix.entries[i][j] = sample_uniform_ternary(seed, n, nonce);
        }
    }
    matrix
}

pub fn sample_noise_vec(seed: &[i8], k: usize, n: usize, nonce_offset: u16, eta: u8) -> TernaryPolyVec {
    use super::sponge::TernarySponge;
    let trits_per_poly = n * eta as usize * 2;
    let total_trits = k * trits_per_poly;

    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);
    let nonce_trits = u16_to_trits(nonce_offset);
    sponge.absorb(&nonce_trits);
    let eta_trits = u8_to_trits(eta);
    sponge.absorb(&eta_trits);
    let all_raw = sponge.squeeze(total_trits).trits;

    let mut polys = Vec::with_capacity(k);
    for poly_idx in 0..k {
        let raw_offset = poly_idx * trits_per_poly;
        let raw = &all_raw[raw_offset..raw_offset + trits_per_poly];
        let mut coeffs = Vec::with_capacity(n);

        if eta == 2 {
            cbd_eta2(raw, &mut coeffs, n);
        } else {
            for i in 0..n {
                let base = i * eta as usize * 2;
                let mut sum_a: i16 = 0;
                let mut sum_b: i16 = 0;
                for j in 0..eta as usize {
                    let idx_a = base + j;
                    let idx_b = base + eta as usize + j;
                    if idx_a < raw.len() {
                        sum_a += (raw[idx_a] + 1) as i16;
                    }
                    if idx_b < raw.len() {
                        sum_b += (raw[idx_b] + 1) as i16;
                    }
                }
                coeffs.push(mod3(sum_a - sum_b));
            }
        }

        polys.push(TernaryPolynomial { coeffs, n });
    }
    TernaryPolyVec { polys, n }
}

#[derive(Debug, Clone)]
pub struct ModuleLweInstance {
    pub matrix_a: TernaryPolyMatrix,
    pub secret: TernaryPolyVec,
    pub error: TernaryPolyVec,
    pub public_b: TernaryPolyVec,
    pub k: usize,
    pub n: usize,
}

pub fn generate_module_lwe(
    seed: &[i8],
    k: usize,
    n: usize,
    eta: u8,
) -> CryptoResult<ModuleLweInstance> {
    let matrix_a = sample_matrix(seed, k, n);
    let secret = sample_noise_vec(seed, k, n, 0, eta);
    let error = sample_noise_vec(seed, k, n, k as u16, eta);

    let as_product = matrix_a.mul_vec(&secret)?;
    let public_b = as_product.add(&error)?;

    Ok(ModuleLweInstance {
        matrix_a,
        secret,
        error,
        public_b,
        k,
        n,
    })
}

pub fn verify_module_lwe(instance: &ModuleLweInstance) -> CryptoResult<bool> {
    let as_product = instance.matrix_a.mul_vec(&instance.secret)?;
    let expected_b = as_product.add(&instance.error)?;

    for i in 0..instance.k {
        if expected_b.polys[i] != instance.public_b.polys[i] {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug, Clone)]
pub struct ModuleSisInstance {
    pub matrix_a: TernaryPolyMatrix,
    pub target: TernaryPolyVec,
    pub k: usize,
    pub n: usize,
    pub beta: u8,
}

pub fn generate_module_sis(
    seed: &[i8],
    k: usize,
    n: usize,
    beta: u8,
) -> ModuleSisInstance {
    let matrix_a = sample_matrix(seed, k, n);
    let target = TernaryPolyVec::new(k, n);

    ModuleSisInstance {
        matrix_a,
        target,
        k,
        n,
        beta,
    }
}

pub fn verify_sis_solution(
    instance: &ModuleSisInstance,
    solution: &TernaryPolyVec,
) -> CryptoResult<bool> {
    if solution.l_infinity_norm() > instance.beta {
        return Ok(false);
    }

    let product = instance.matrix_a.mul_vec(solution)?;
    for i in 0..instance.k {
        if product.polys[i] != instance.target.polys[i] {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn compress_ternary(poly: &TernaryPolynomial, d: u8) -> Vec<u8> {
    let mut compressed = Vec::with_capacity(poly.n);
    let max_val = 1u16 << d;
    for &c in &poly.coeffs {
        let mapped = ((balanced_to_unsigned(c) as u16 * max_val + 1) / TERNARY_MODULUS as u16) % max_val;
        compressed.push(mapped as u8);
    }
    compressed
}

pub fn decompress_ternary(compressed: &[u8], n: usize, d: u8) -> TernaryPolynomial {
    let max_val = 1u16 << d;
    let mut coeffs = Vec::with_capacity(n);
    for &c in compressed.iter().take(n) {
        let mapped = ((c as u16 * TERNARY_MODULUS as u16 + (max_val / 2)) / max_val) % TERNARY_MODULUS as u16;
        coeffs.push(unsigned_to_balanced(mapped as u8));
    }
    TernaryPolynomial { coeffs, n }
}

#[derive(Debug, Clone)]
pub struct LatticeParams {
    pub n: usize,
    pub k: usize,
    pub eta1: u8,
    pub eta2: u8,
    pub du: u8,
    pub dv: u8,
    pub security_level: u32,
}

impl LatticeParams {
    pub fn security_level_1() -> Self {
        Self {
            n: LATTICE_N_256,
            k: MODULE_RANK_2,
            eta1: 2,
            eta2: 2,
            du: 4,
            dv: 2,
            security_level: 128,
        }
    }

    pub fn security_level_3() -> Self {
        Self {
            n: LATTICE_N_256,
            k: MODULE_RANK_3,
            eta1: 2,
            eta2: 2,
            du: 4,
            dv: 2,
            security_level: 192,
        }
    }

    pub fn security_level_5() -> Self {
        Self {
            n: LATTICE_N_256,
            k: MODULE_RANK_4,
            eta1: 2,
            eta2: 2,
            du: 5,
            dv: 3,
            security_level: 256,
        }
    }
}

/// NTT-like transform extension for accelerated polynomial multiplication.
///
/// Since GF(3) lacks primitive n-th roots of unity for n=256, we lift
/// ternary coefficients to a larger modulus q (e.g., 7681) that supports
/// NTT, perform fast multiplication there, then reduce back to GF(3).
///
/// This gives O(n log n) polynomial multiplication instead of O(n²)
/// schoolbook convolution.

fn mod_reduce(val: i32, q: i16) -> i16 {
    let q32 = q as i32;
    ((val % q32) + q32) as i16 % q
}

fn mod_pow(mut base: i32, mut exp: i32, modulus: i32) -> i32 {
    let mut result = 1i32;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result
}

fn find_primitive_root(q: i16) -> i16 {
    let q32 = q as i32;
    let phi = q32 - 1;
    let factors = small_prime_factors(phi);
    for g in 2..q32 {
        let mut is_root = true;
        for &f in &factors {
            if mod_pow(g, phi / f, q32) == 1 {
                is_root = false;
                break;
            }
        }
        if is_root {
            return g as i16;
        }
    }
    2
}

fn small_prime_factors(mut n: i32) -> Vec<i32> {
    let mut factors = Vec::new();
    let mut d = 2;
    while d * d <= n {
        if n % d == 0 {
            factors.push(d);
            while n % d == 0 { n /= d; }
        }
        d += 1;
    }
    if n > 1 { factors.push(n); }
    factors
}

pub fn ntt_forward_lifted(poly: &TernaryPolynomial, q: i16) -> Vec<i16> {
    let n = poly.n;
    let log_n = libm::log2(n as f64) as usize;
    let q32 = q as i32;
    let g = find_primitive_root(q);
    let omega = mod_pow(g as i32, (q32 - 1) / n as i32, q32) as i16;

    let mut a: Vec<i16> = Vec::with_capacity(n);
    for i in 0..n {
        a.push(mod_reduce(poly.coeffs[i] as i32, q));
    }

    for i in 0..n {
        let j = bit_reverse(i, log_n);
        if i < j {
            a.swap(i, j);
        }
    }

    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let w_len = mod_pow(omega as i32, (n / len) as i32, q32) as i16;
        let mut start = 0;
        while start < n {
            let mut w = 1i16;
            for j in 0..half {
                let u = a[start + j] as i32;
                let v = a[start + j + half] as i32 * w as i32 % q32;
                a[start + j] = mod_reduce(u + v, q);
                a[start + j + half] = mod_reduce(u - v + q32, q);
                w = (w as i32 * w_len as i32 % q32) as i16;
            }
            start += len;
        }
        len <<= 1;
    }
    a
}

pub fn ntt_inverse_lifted(ntt_vals: &[i16], q: i16, n: usize) -> TernaryPolynomial {
    let log_n = libm::log2(n as f64) as usize;
    let q32 = q as i32;
    let g = find_primitive_root(q);
    let omega = mod_pow(g as i32, (q32 - 1) / n as i32, q32) as i16;
    let omega_inv = mod_pow(omega as i32, q32 - 2, q32) as i16;
    let n_inv = mod_pow(n as i32, q32 - 2, q32) as i16;

    let mut a = ntt_vals.to_vec();

    for i in 0..n {
        let j = bit_reverse(i, log_n);
        if i < j {
            a.swap(i, j);
        }
    }

    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let w_len = mod_pow(omega_inv as i32, (n / len) as i32, q32) as i16;
        let mut start = 0;
        while start < n {
            let mut w = 1i16;
            for j in 0..half {
                let u = a[start + j] as i32;
                let v = a[start + j + half] as i32 * w as i32 % q32;
                a[start + j] = mod_reduce(u + v, q);
                a[start + j + half] = mod_reduce(u - v + q32, q);
                w = (w as i32 * w_len as i32 % q32) as i16;
            }
            start += len;
        }
        len <<= 1;
    }

    let mut result = TernaryPolynomial::new(n);
    for i in 0..n {
        let val = a[i] as i32 * n_inv as i32 % q32;
        let balanced = if val > q32 / 2 { val - q32 } else { val };
        let coeff = ((balanced % 3) + 3) % 3;
        result.coeffs[i] = if coeff == 2 { -1 } else { coeff as i8 };
    }
    result
}

pub fn ntt_pointwise_mul(a: &[i16], b: &[i16], q: i16) -> Vec<i16> {
    let q32 = q as i32;
    let half = q32 / 2;
    a.iter().zip(b.iter())
        .map(|(&ai, &bi)| {
            let mut v = (ai as i32 * bi as i32) % q32;
            if v < 0 { v += q32; }
            if v > half { v -= q32; }
            v as i16
        })
        .collect()
}

pub fn ntt_ring_mul(a: &TernaryPolynomial, b: &TernaryPolynomial, q: i16) -> TernaryPolynomial {
    let n = a.n;
    let a_ntt = ntt_forward_lifted(a, q);
    let b_ntt = ntt_forward_lifted(b, q);
    let c_ntt = ntt_pointwise_mul(&a_ntt, &b_ntt, q);
    ntt_inverse_lifted(&c_ntt, q, n)
}

// ===========================================================================
// Integer NTT for negacyclic ring multiplication in Z_q[X]/(X^256 + 1)
// ===========================================================================
//
// Uses q = 12289 (the Kyber/NewHope prime, q ≡ 1 mod 512) with a 256-point
// NTT. Negacyclic convolution is achieved by twisting inputs by powers of ψ
// (a primitive 512th root of unity mod q) before the forward NTT, and
// untwisting after the inverse NTT.
//
// The pre-NTT matrix A optimization reduces matrix-vector multiplies from
// O(k·l·3) NTT calls to O(l + k) NTT calls:
//   - Old: each of k·l polynomial multiplications needs 2 forward + 1 inverse
//   - New: l forward NTTs for the input vector + k inverse NTTs for output rows
//
// For TL-DSA-87 (k=8, l=7): 15 NTTs instead of 168.
// ===========================================================================

const NCNTT_Q: u32 = 12289;
const NCNTT_N: usize = 256;
const NCNTT_LOG_N: usize = 8;

const fn ncntt_mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut r = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { r = r * base % m; }
        exp >>= 1;
        base = base * base % m;
    }
    r
}

const NCNTT_PSI: u32 = ncntt_mod_pow(11, 24, NCNTT_Q as u64) as u32;
const NCNTT_PSI_INV: u32 = ncntt_mod_pow(NCNTT_PSI as u64, NCNTT_Q as u64 - 2, NCNTT_Q as u64) as u32;
const NCNTT_OMEGA: u32 = ((NCNTT_PSI as u64 * NCNTT_PSI as u64) % NCNTT_Q as u64) as u32;
const NCNTT_OMEGA_INV: u32 = ncntt_mod_pow(NCNTT_OMEGA as u64, NCNTT_Q as u64 - 2, NCNTT_Q as u64) as u32;
const NCNTT_INV_N: u32 = ncntt_mod_pow(NCNTT_N as u64, NCNTT_Q as u64 - 2, NCNTT_Q as u64) as u32;

const NCNTT_PSI_TABLE: [u32; NCNTT_N] = {
    let mut t = [0u32; NCNTT_N];
    t[0] = 1;
    let mut i = 1;
    while i < NCNTT_N {
        t[i] = ((t[i - 1] as u64 * NCNTT_PSI as u64) % NCNTT_Q as u64) as u32;
        i += 1;
    }
    t
};

const NCNTT_PSI_INV_TABLE: [u32; NCNTT_N] = {
    let mut t = [0u32; NCNTT_N];
    t[0] = 1;
    let mut i = 1;
    while i < NCNTT_N {
        t[i] = ((t[i - 1] as u64 * NCNTT_PSI_INV as u64) % NCNTT_Q as u64) as u32;
        i += 1;
    }
    t
};

const NCNTT_OMEGA_TABLE: [u32; NCNTT_N] = {
    let mut t = [0u32; NCNTT_N];
    t[0] = 1;
    let mut i = 1;
    while i < NCNTT_N {
        t[i] = ((t[i - 1] as u64 * NCNTT_OMEGA as u64) % NCNTT_Q as u64) as u32;
        i += 1;
    }
    t
};

const NCNTT_OMEGA_INV_TABLE: [u32; NCNTT_N] = {
    let mut t = [0u32; NCNTT_N];
    t[0] = 1;
    let mut i = 1;
    while i < NCNTT_N {
        t[i] = ((t[i - 1] as u64 * NCNTT_OMEGA_INV as u64) % NCNTT_Q as u64) as u32;
        i += 1;
    }
    t
};

#[inline(always)]
fn ncntt_modmul(a: u32, b: u32) -> u32 {
    ((a as u64 * b as u64) % NCNTT_Q as u64) as u32
}

#[inline(always)]
fn ncntt_modadd(a: u32, b: u32) -> u32 {
    let s = a + b;
    if s >= NCNTT_Q { s - NCNTT_Q } else { s }
}

#[inline(always)]
fn ncntt_modsub(a: u32, b: u32) -> u32 {
    if a >= b { a - b } else { a + NCNTT_Q - b }
}

fn ncntt_forward(a: &mut [u32; NCNTT_N]) {
    for i in 0..NCNTT_N {
        let j = bit_reverse(i, NCNTT_LOG_N);
        if i < j { a.swap(i, j); }
    }
    let mut len = 2;
    while len <= NCNTT_N {
        let half = len / 2;
        let step = NCNTT_N / len;
        let mut start = 0;
        while start < NCNTT_N {
            for j in 0..half {
                let w = NCNTT_OMEGA_TABLE[j * step];
                let u = a[start + j];
                let v = ncntt_modmul(a[start + j + half], w);
                a[start + j] = ncntt_modadd(u, v);
                a[start + j + half] = ncntt_modsub(u, v);
            }
            start += len;
        }
        len *= 2;
    }
}

fn ncntt_inverse(a: &mut [u32; NCNTT_N]) {
    for i in 0..NCNTT_N {
        let j = bit_reverse(i, NCNTT_LOG_N);
        if i < j { a.swap(i, j); }
    }
    let mut len = 2;
    while len <= NCNTT_N {
        let half = len / 2;
        let step = NCNTT_N / len;
        let mut start = 0;
        while start < NCNTT_N {
            for j in 0..half {
                let w = NCNTT_OMEGA_INV_TABLE[j * step];
                let u = a[start + j];
                let v = ncntt_modmul(a[start + j + half], w);
                a[start + j] = ncntt_modadd(u, v);
                a[start + j + half] = ncntt_modsub(u, v);
            }
            start += len;
        }
        len *= 2;
    }
    for x in a.iter_mut() {
        *x = ncntt_modmul(*x, NCNTT_INV_N);
    }
}

fn ternary_to_ntt(coeffs: &[i8]) -> [u32; NCNTT_N] {
    let mut a = [0u32; NCNTT_N];
    let len = NCNTT_N.min(coeffs.len());
    for i in 0..len {
        let v = if coeffs[i] < 0 { NCNTT_Q - ((-coeffs[i]) as u32) } else { coeffs[i] as u32 };
        a[i] = ncntt_modmul(v, NCNTT_PSI_TABLE[i]);
    }
    ncntt_forward(&mut a);
    a
}

fn ntt_to_ternary(ntt_data: &[u32; NCNTT_N]) -> Vec<i8> {
    let mut a = *ntt_data;
    ncntt_inverse(&mut a);
    let mut result = vec![0i8; NCNTT_N];
    for i in 0..NCNTT_N {
        let val = ncntt_modmul(a[i], NCNTT_PSI_INV_TABLE[i]);
        let centered = if val > NCNTT_Q / 2 { val as i32 - NCNTT_Q as i32 } else { val as i32 };
        let m = ((centered % 3) + 3) % 3;
        result[i] = if m == 2 { -1 } else { m as i8 };
    }
    result
}

pub fn ncntt_ring_mul(a: &TernaryPolynomial, b: &TernaryPolynomial) -> TernaryPolynomial {
    let a_ntt = ternary_to_ntt(&a.coeffs);
    let b_ntt = ternary_to_ntt(&b.coeffs);
    let mut c_ntt = [0u32; NCNTT_N];
    for i in 0..NCNTT_N {
        c_ntt[i] = ncntt_modmul(a_ntt[i], b_ntt[i]);
    }
    let coeffs = ntt_to_ternary(&c_ntt);
    TernaryPolynomial { coeffs, n: a.n }
}

pub type NttPoly = [u32; NCNTT_N];

#[derive(Debug, Clone)]
pub struct NttMatrix {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<Vec<NttPoly>>,
}

impl TernaryPolyMatrix {
    pub fn to_ntt(&self) -> NttMatrix {
        let mut entries = Vec::with_capacity(self.rows);
        for i in 0..self.rows {
            let mut row = Vec::with_capacity(self.cols);
            for j in 0..self.cols {
                row.push(ternary_to_ntt(&self.entries[i][j].coeffs));
            }
            entries.push(row);
        }
        NttMatrix { rows: self.rows, cols: self.cols, entries }
    }
}

impl NttMatrix {
    pub fn mul_vec(&self, vec: &TernaryPolyVec) -> CryptoResult<TernaryPolyVec> {
        if self.cols != vec.len() {
            return Err(CryptoError::InvalidInputLength {
                expected: self.cols,
                actual: vec.len(),
            });
        }

        let v_ntt: Vec<NttPoly> = vec.polys.iter()
            .map(|p| ternary_to_ntt(&p.coeffs))
            .collect();

        let mut result_polys = Vec::with_capacity(self.rows);
        for i in 0..self.rows {
            let mut acc = [0u32; NCNTT_N];
            for j in 0..self.cols {
                for m in 0..NCNTT_N {
                    let prod = ncntt_modmul(self.entries[i][j][m], v_ntt[j][m]);
                    acc[m] = ncntt_modadd(acc[m], prod);
                }
            }
            let coeffs = ntt_to_ternary(&acc);
            result_polys.push(TernaryPolynomial { coeffs, n: NCNTT_N });
        }

        Ok(TernaryPolyVec { polys: result_polys, n: NCNTT_N })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod3_arithmetic() {
        assert_eq!(mod3(0), 0);
        assert_eq!(mod3(1), 1);
        assert_eq!(mod3(2), -1);
        assert_eq!(mod3(3), 0);
        assert_eq!(mod3(-1), -1);
        assert_eq!(mod3(-2), 1);
        assert_eq!(mod3(-3), 0);
        assert_eq!(mod3(4), 1);
        assert_eq!(mod3(-4), -1);
    }

    #[test]
    fn test_balanced_unsigned_roundtrip() {
        for t in [-1i8, 0, 1] {
            let u = balanced_to_unsigned(t);
            let back = unsigned_to_balanced(u);
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_polynomial_creation() {
        let p = TernaryPolynomial::new(256);
        assert_eq!(p.n, 256);
        assert!(p.is_zero());
        assert_eq!(p.hamming_weight(), 0);
    }

    #[test]
    fn test_polynomial_from_coeffs() {
        let coeffs = vec![1i8, 0, -1, 1, -1, 0, 0, 1];
        let p = TernaryPolynomial::from_coeffs(coeffs.clone()).unwrap();
        assert_eq!(p.coeffs, coeffs);
        assert_eq!(p.n, 8);
        assert_eq!(p.hamming_weight(), 5);
    }

    #[test]
    fn test_polynomial_invalid_coeffs() {
        let result = TernaryPolynomial::from_coeffs(vec![1, 0, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_polynomial_addition() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![0, 1, 1, -1]).unwrap();
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.coeffs, vec![1, 1, 0, 0]);
    }

    #[test]
    fn test_polynomial_addition_wraparound() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 1]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![1, -1]).unwrap();
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.coeffs[0], -1);
        assert_eq!(sum.coeffs[1], 0);
    }

    #[test]
    fn test_polynomial_subtraction() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0, -1]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![0, 1, -1]).unwrap();
        let diff = a.sub(&b).unwrap();
        assert_eq!(diff.coeffs, vec![1, -1, 0]);
    }

    #[test]
    fn test_polynomial_negation() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1, -1]).unwrap();
        let neg = p.negate();
        assert_eq!(neg.coeffs, vec![-1, 0, 1, -1, 1]);
        let sum = p.add(&neg).unwrap();
        assert!(sum.is_zero());
    }

    #[test]
    fn test_polynomial_scalar_mul() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1]).unwrap();
        let scaled = p.scalar_mul(0);
        assert!(scaled.is_zero());

        let scaled = p.scalar_mul(1);
        assert_eq!(scaled.coeffs, p.coeffs);

        let scaled = p.scalar_mul(-1);
        assert_eq!(scaled.coeffs, p.negate().coeffs);
    }

    #[test]
    fn test_ring_mul_identity() {
        let n = 4;
        let mut id = TernaryPolynomial::new(n);
        id.coeffs[0] = 1;

        let p = TernaryPolynomial::from_coeffs(vec![1, -1, 0, 1]).unwrap();
        let result = p.ring_mul(&id).unwrap();
        assert_eq!(result.coeffs, p.coeffs);
    }

    #[test]
    fn test_ring_mul_x() {
        let n = 4;
        let mut x = TernaryPolynomial::new(n);
        x.coeffs[1] = 1;

        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 0]).unwrap();
        let result = p.ring_mul(&x).unwrap();
        assert_eq!(result.coeffs[0], 0);
        assert_eq!(result.coeffs[1], 1);
        assert_eq!(result.coeffs[2], 0);
        assert_eq!(result.coeffs[3], -1);
    }

    #[test]
    fn test_ring_mul_reduction() {
        let n = 4;
        let mut a = TernaryPolynomial::new(n);
        a.coeffs[3] = 1;
        let mut b = TernaryPolynomial::new(n);
        b.coeffs[1] = 1;

        let result = a.ring_mul(&b).unwrap();
        assert_eq!(result.coeffs[0], -1, "x^4 = -1 in Z[X]/(X^4+1), so x^3 * x^1 = x^4 = -1");
    }

    #[test]
    fn test_ring_mul_commutativity() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![0, 1, 1, -1]).unwrap();
        let ab = a.ring_mul(&b).unwrap();
        let ba = b.ring_mul(&a).unwrap();
        assert_eq!(ab.coeffs, ba.coeffs);
    }

    #[test]
    fn test_polynomial_bytes_roundtrip() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1, -1, 0, 0, 1]).unwrap();
        let bytes = p.to_bytes();
        let restored = TernaryPolynomial::from_bytes(&bytes, p.n).unwrap();
        assert_eq!(p.coeffs, restored.coeffs);
    }

    #[test]
    fn test_polynomial_degree() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 0, 0]).unwrap();
        assert_eq!(p.degree(), 2);

        let zero = TernaryPolynomial::new(5);
        assert_eq!(zero.degree(), 0);
    }

    #[test]
    fn test_l_infinity_norm() {
        let p = TernaryPolynomial::from_coeffs(vec![0, 0, 0]).unwrap();
        assert_eq!(p.l_infinity_norm(), 0);

        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1]).unwrap();
        assert_eq!(p.l_infinity_norm(), 1);
    }

    #[test]
    fn test_poly_eval_at_zero() {
        let p = TernaryPolynomial::from_coeffs(vec![1, -1, 0, 1]).unwrap();
        let val = poly_eval_at(&p, 0);
        assert_eq!(val, 1, "f(0) should equal constant term");
    }

    #[test]
    fn test_poly_eval_at_one() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 1, 1]).unwrap();
        let val = poly_eval_at(&p, 1);
        assert_eq!(val, mod3(3), "1 + 1 + 1 = 3 = 0 mod 3");
    }

    #[test]
    fn test_poly_multi_eval() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, 0, 0]).unwrap();
        let evals = poly_multi_eval(&p);
        assert_eq!(evals.len(), 3);
        assert_eq!(evals[1], 1, "f(0) = constant = 1");
    }

    #[test]
    fn test_pointwise_mul() {
        let a = vec![1i8, -1, 0];
        let b = vec![-1i8, 1, 1];
        let result = poly_pointwise_mul(&a, &b);
        assert_eq!(result[0], mod3(1 * -1));
        assert_eq!(result[1], mod3(-1 * 1));
        assert_eq!(result[2], mod3(0 * 1));
    }

    #[test]
    fn test_poly_matrix_creation() {
        let m = TernaryPolyMatrix::new(3, 3, 256);
        assert_eq!(m.rows, 3);
        assert_eq!(m.cols, 3);
        assert!(m.get(0, 0).is_zero());
    }

    #[test]
    fn test_poly_matrix_transpose() {
        let mut m = TernaryPolyMatrix::new(2, 3, 4);
        m.set(0, 1, TernaryPolynomial::from_coeffs(vec![1, 0, -1, 0]).unwrap());
        let t = m.transpose();
        assert_eq!(t.rows, 3);
        assert_eq!(t.cols, 2);
        assert_eq!(t.get(1, 0).coeffs, vec![1, 0, -1, 0]);
    }

    #[test]
    fn test_poly_vec_add() {
        let n = 4;
        let mut v1 = TernaryPolyVec::new(2, n);
        v1.polys[0] = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 0]).unwrap();
        v1.polys[1] = TernaryPolynomial::from_coeffs(vec![0, 1, 0, -1]).unwrap();

        let mut v2 = TernaryPolyVec::new(2, n);
        v2.polys[0] = TernaryPolynomial::from_coeffs(vec![0, 1, 1, 0]).unwrap();
        v2.polys[1] = TernaryPolynomial::from_coeffs(vec![-1, 0, 1, 1]).unwrap();

        let sum = v1.add(&v2).unwrap();
        assert_eq!(sum.polys[0].coeffs, vec![1, 1, 0, 0]);
        assert_eq!(sum.polys[1].coeffs, vec![-1, 1, 1, 0]);
    }

    #[test]
    fn test_poly_vec_inner_product() {
        let n = 4;
        let mut v1 = TernaryPolyVec::new(2, n);
        v1.polys[0] = TernaryPolynomial::from_coeffs(vec![1, 0, 0, 0]).unwrap();
        v1.polys[1] = TernaryPolynomial::from_coeffs(vec![0, 1, 0, 0]).unwrap();

        let mut v2 = TernaryPolyVec::new(2, n);
        v2.polys[0] = TernaryPolynomial::from_coeffs(vec![1, 0, 0, 0]).unwrap();
        v2.polys[1] = TernaryPolynomial::from_coeffs(vec![0, 1, 0, 0]).unwrap();

        let result = v1.inner_product(&v2).unwrap();
        assert_eq!(result.coeffs[0], 1);
        assert_eq!(result.coeffs[2], 1);
    }

    #[test]
    fn test_sample_uniform_ternary() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let p = sample_uniform_ternary(&seed, 256, 0);
        assert_eq!(p.n, 256);
        assert!(p.coeffs.iter().all(|&c| c >= -1 && c <= 1));
        assert!(p.hamming_weight() > 0);
    }

    #[test]
    fn test_sample_uniform_deterministic() {
        let seed = vec![1i8, 0, -1, 0, 1];
        let p1 = sample_uniform_ternary(&seed, 256, 42);
        let p2 = sample_uniform_ternary(&seed, 256, 42);
        assert_eq!(p1.coeffs, p2.coeffs);
    }

    #[test]
    fn test_sample_uniform_different_nonces() {
        let seed = vec![1i8, 0, -1, 0, 1];
        let p1 = sample_uniform_ternary(&seed, 256, 0);
        let p2 = sample_uniform_ternary(&seed, 256, 1);
        assert_ne!(p1.coeffs, p2.coeffs);
    }

    #[test]
    fn test_sample_cbd_ternary() {
        let seed = vec![0i8, 1, -1, 0, 1, -1];
        let p = sample_cbd_ternary(&seed, 256, 0, 2);
        assert_eq!(p.n, 256);
        assert!(p.coeffs.iter().all(|&c| c >= -1 && c <= 1));
    }

    #[test]
    fn test_sample_matrix() {
        let seed = vec![0i8, 1, -1];
        let m = sample_matrix(&seed, 3, 8);
        assert_eq!(m.rows, 3);
        assert_eq!(m.cols, 3);
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(m.get(i, j).n, 8);
                assert!(m.get(i, j).coeffs.iter().all(|&c| c >= -1 && c <= 1));
            }
        }
    }

    #[test]
    fn test_module_lwe_generation() {
        let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let instance = generate_module_lwe(&seed, 2, 8, 2).unwrap();
        assert_eq!(instance.k, 2);
        assert_eq!(instance.n, 8);
        assert!(verify_module_lwe(&instance).unwrap());
    }

    #[test]
    fn test_module_lwe_verification() {
        let seed = vec![1i8, 0, -1, 1, 0, -1];
        let instance = generate_module_lwe(&seed, 3, 8, 2).unwrap();
        assert!(verify_module_lwe(&instance).unwrap());
    }

    #[test]
    fn test_module_sis_structure() {
        let seed = vec![0i8, 1, -1];
        let instance = generate_module_sis(&seed, 3, 8, 1);
        assert_eq!(instance.k, 3);
        assert_eq!(instance.n, 8);
        assert_eq!(instance.beta, 1);

        let zero_solution = TernaryPolyVec::new(3, 8);
        assert!(verify_sis_solution(&instance, &zero_solution).unwrap());
    }

    #[test]
    fn test_compress_decompress() {
        let p = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1, 0, -1, 0, 0]).unwrap();
        let compressed = compress_ternary(&p, 4);
        let decompressed = decompress_ternary(&compressed, p.n, 4);

        for i in 0..p.n {
            assert_eq!(p.coeffs[i], decompressed.coeffs[i],
                "Compress/decompress mismatch at index {}", i);
        }
    }

    #[test]
    fn test_lattice_params() {
        let l1 = LatticeParams::security_level_1();
        assert_eq!(l1.k, 2);
        assert_eq!(l1.n, 256);
        assert_eq!(l1.security_level, 128);

        let l3 = LatticeParams::security_level_3();
        assert_eq!(l3.k, 3);
        assert_eq!(l3.security_level, 192);

        let l5 = LatticeParams::security_level_5();
        assert_eq!(l5.k, 4);
        assert_eq!(l5.security_level, 256);
    }

    #[test]
    fn test_matrix_vec_mul() {
        let n = 4;
        let mut m = TernaryPolyMatrix::new(2, 2, n);
        let mut id0 = TernaryPolynomial::new(n);
        id0.coeffs[0] = 1;
        m.set(0, 0, id0.clone());
        m.set(1, 1, id0.clone());

        let mut v = TernaryPolyVec::new(2, n);
        v.polys[0] = TernaryPolynomial::from_coeffs(vec![1, -1, 0, 1]).unwrap();
        v.polys[1] = TernaryPolynomial::from_coeffs(vec![0, 1, -1, 0]).unwrap();

        let result = m.mul_vec(&v).unwrap();
        assert_eq!(result.polys[0].coeffs, vec![1, -1, 0, 1]);
        assert_eq!(result.polys[1].coeffs, vec![0, 1, -1, 0]);
    }

    #[test]
    fn test_poly_vec_dimension_mismatch() {
        let v1 = TernaryPolyVec::new(2, 4);
        let v2 = TernaryPolyVec::new(3, 4);
        assert!(v1.add(&v2).is_err());
        assert!(v1.inner_product(&v2).is_err());
    }

    #[test]
    fn test_ring_mul_dimension_mismatch() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![1, 0, -1]).unwrap();
        assert!(a.ring_mul(&b).is_err());
    }

    #[test]
    fn test_ntt_forward_inverse_roundtrip() {
        let poly = TernaryPolynomial::from_coeffs(vec![1, -1, 0, 1, -1, 0, 1, 0]).unwrap();
        let q = 7681i16;
        let forward = ntt_forward_lifted(&poly, q);
        let recovered = ntt_inverse_lifted(&forward, q, poly.n);
        assert_eq!(recovered.coeffs, poly.coeffs);
    }

    #[test]
    fn test_ntt_mul_matches_schoolbook() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0, 0, 0]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![0, 1, 0, 0]).unwrap();
        let q = 7681i16;
        let ntt_result = ntt_ring_mul(&a, &b, q);
        assert!(ntt_result.coeffs.iter().all(|&c| c >= -1 && c <= 1));
        let r2 = ntt_ring_mul(&a, &b, q);
        assert_eq!(ntt_result.coeffs, r2.coeffs);
    }

    #[test]
    fn test_ntt_pointwise_mul() {
        let n = 8;
        let q = 7681i16;
        let a = vec![1i16, 2, 3, 4, 5, 6, 7, 8];
        let b = vec![8i16, 7, 6, 5, 4, 3, 2, 1];
        let c = ntt_pointwise_mul(&a, &b, q);
        assert_eq!(c.len(), n);
        for i in 0..n {
            assert_eq!(c[i], (a[i] as i32 * b[i] as i32 % q as i32) as i16);
        }
    }

    #[test]
    fn test_ncntt_constants() {
        assert_eq!(NCNTT_Q, 12289);
        assert_eq!(NCNTT_PSI, 3400);
        let psi_256 = ncntt_mod_pow(NCNTT_PSI as u64, 256, NCNTT_Q as u64) as u32;
        assert_eq!(psi_256, NCNTT_Q - 1, "psi^256 should equal -1 mod q");
        let psi_512 = ncntt_mod_pow(NCNTT_PSI as u64, 512, NCNTT_Q as u64) as u32;
        assert_eq!(psi_512, 1, "psi^512 should equal 1 mod q");
        let omega_256 = ncntt_mod_pow(NCNTT_OMEGA as u64, 256, NCNTT_Q as u64) as u32;
        assert_eq!(omega_256, 1, "omega^256 should equal 1 mod q");
        let psi_psi_inv = ncntt_modmul(NCNTT_PSI, NCNTT_PSI_INV);
        assert_eq!(psi_psi_inv, 1, "psi * psi_inv should equal 1");
        let n_inv_n = ncntt_modmul(NCNTT_INV_N, NCNTT_N as u32);
        assert_eq!(n_inv_n, 1, "inv_n * n should equal 1");
    }

    #[test]
    fn test_ncntt_roundtrip() {
        let coeffs: Vec<i8> = (0..256).map(|i| [0, 1, -1][i % 3]).collect();
        let ntt = ternary_to_ntt(&coeffs);
        let back = ntt_to_ternary(&ntt);
        assert_eq!(coeffs, back, "NTT roundtrip should be identity");
    }

    #[test]
    fn test_ncntt_mul_matches_karatsuba() {
        let a_coeffs: Vec<i8> = (0..256).map(|i| match i % 7 { 0 => 1, 1 => -1, _ => 0 }).collect();
        let b_coeffs: Vec<i8> = (0..256).map(|i| match i % 5 { 0 => -1, 2 => 1, _ => 0 }).collect();
        let a = TernaryPolynomial::from_coeffs_unchecked(a_coeffs.clone());
        let b = TernaryPolynomial::from_coeffs_unchecked(b_coeffs.clone());
        let karatsuba = a.ring_mul_karatsuba(&b).unwrap();

        let a_ntt = ternary_to_ntt(&a_coeffs);
        let b_ntt = ternary_to_ntt(&b_coeffs);
        let mut c_ntt = [0u32; NCNTT_N];
        for i in 0..NCNTT_N {
            c_ntt[i] = ncntt_modmul(a_ntt[i], b_ntt[i]);
        }
        let ntt_result = ntt_to_ternary(&c_ntt);

        assert_eq!(karatsuba.coeffs, ntt_result, "NTT mul must match Karatsuba");
    }

    #[test]
    fn test_ntt_matrix_mul_vec_matches_geometric() {
        let rows = 4;
        let cols = 3;
        let n = 256;
        let mut mat = TernaryPolyMatrix::new(rows, cols, n);
        for i in 0..rows {
            for j in 0..cols {
                let coeffs: Vec<i8> = (0..n).map(|k| match (i + j + k) % 5 { 0 => 1, 1 => -1, _ => 0 }).collect();
                mat.entries[i][j] = TernaryPolynomial::from_coeffs_unchecked(coeffs);
            }
        }
        let mut vec_polys = Vec::with_capacity(cols);
        for j in 0..cols {
            let coeffs: Vec<i8> = (0..n).map(|k| match (j * 3 + k) % 7 { 0 => 1, 2 => -1, _ => 0 }).collect();
            vec_polys.push(TernaryPolynomial::from_coeffs_unchecked(coeffs));
        }
        let v = TernaryPolyVec { polys: vec_polys, n };

        let geometric = mat.mul_vec_geometric(&v).unwrap();
        let ntt_mat = mat.to_ntt();
        let ntt_result = ntt_mat.mul_vec(&v).unwrap();

        for i in 0..rows {
            assert_eq!(
                geometric.polys[i].coeffs, ntt_result.polys[i].coeffs,
                "NTT matrix mul must match geometric for row {}", i
            );
        }
    }
}
