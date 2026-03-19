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
//! lattice-based post-quantum cryptography (TL-KEM) in the ternary domain.
//!
//! All operations work in the polynomial ring R_q = Z_3[X]/(X^n + 1),
//! where coefficients are elements of GF(3) in balanced representation
//! {-1, 0, +1}. This maps directly to PlenumNET's Representation A.
//!
//! Ported from the Salvi kernel (`src/kernel/src/crypto/ternary_lattice.rs`)
//! to use std and `ternary-math`'s TL-Sponge-385 as the hash primitive.

use crate::tlsponge385::Sponge385Pub;

pub const LATTICE_N_256: usize = 256;

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

    pub fn from_coeffs(coeffs: Vec<i8>) -> Result<Self, LatticeError> {
        for &c in &coeffs {
            if c < -1 || c > 1 {
                return Err(LatticeError::InvalidTritValue(c));
            }
        }
        let n = coeffs.len();
        Ok(Self { coeffs, n })
    }

    pub fn from_coeffs_unchecked(coeffs: Vec<i8>) -> Self {
        let n = coeffs.len();
        Self { coeffs, n }
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

    pub fn add(&self, other: &TernaryPolynomial) -> Result<TernaryPolynomial, LatticeError> {
        if self.n != other.n {
            return Err(LatticeError::DimensionMismatch { expected: self.n, actual: other.n });
        }
        let mut result = Vec::with_capacity(self.n);
        for i in 0..self.n {
            result.push(mod3(self.coeffs[i] as i16 + other.coeffs[i] as i16));
        }
        Ok(TernaryPolynomial { coeffs: result, n: self.n })
    }

    pub fn sub(&self, other: &TernaryPolynomial) -> Result<TernaryPolynomial, LatticeError> {
        if self.n != other.n {
            return Err(LatticeError::DimensionMismatch { expected: self.n, actual: other.n });
        }
        let mut result = Vec::with_capacity(self.n);
        for i in 0..self.n {
            result.push(mod3(self.coeffs[i] as i16 - other.coeffs[i] as i16));
        }
        Ok(TernaryPolynomial { coeffs: result, n: self.n })
    }

    pub fn ring_mul_karatsuba(&self, other: &TernaryPolynomial) -> Result<TernaryPolynomial, LatticeError> {
        if self.n != other.n {
            return Err(LatticeError::DimensionMismatch { expected: self.n, actual: other.n });
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

    pub fn from_bytes(bytes: &[u8], n: usize) -> Result<Self, LatticeError> {
        let mut coeffs = Vec::with_capacity(n);
        let mut byte_idx = 0;
        let mut bit_offset = 0;

        for _ in 0..n {
            if byte_idx >= bytes.len() {
                return Err(LatticeError::DimensionMismatch {
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

    pub fn transpose(&self) -> Self {
        let mut result = TernaryPolyMatrix::new(self.cols, self.rows, self.n);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.entries[j][i] = self.entries[i][j].clone();
            }
        }
        result
    }

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

    pub fn add(&self, other: &TernaryPolyVec) -> Result<TernaryPolyVec, LatticeError> {
        if self.polys.len() != other.polys.len() {
            return Err(LatticeError::DimensionMismatch {
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

    pub fn inner_product(&self, other: &TernaryPolyVec) -> Result<TernaryPolynomial, LatticeError> {
        if self.polys.len() != other.polys.len() {
            return Err(LatticeError::DimensionMismatch {
                expected: self.polys.len(),
                actual: other.polys.len(),
            });
        }
        let mut sum = TernaryPolynomial::new(self.n);
        for i in 0..self.polys.len() {
            let product = self.polys[i].ring_mul_karatsuba(&other.polys[i])?;
            sum = sum.add(&product)?;
        }
        Ok(sum)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        Self { n: LATTICE_N_256, k: MODULE_RANK_2, eta1: 2, eta2: 2, du: 4, dv: 2, security_level: 128 }
    }

    pub fn security_level_3() -> Self {
        Self { n: LATTICE_N_256, k: MODULE_RANK_3, eta1: 2, eta2: 2, du: 4, dv: 2, security_level: 192 }
    }

    pub fn security_level_5() -> Self {
        Self { n: LATTICE_N_256, k: MODULE_RANK_4, eta1: 2, eta2: 2, du: 5, dv: 3, security_level: 256 }
    }
}

#[derive(Debug)]
pub enum LatticeError {
    InvalidTritValue(i8),
    DimensionMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for LatticeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LatticeError::InvalidTritValue(v) => write!(f, "Invalid trit value: {}", v),
            LatticeError::DimensionMismatch { expected, actual } =>
                write!(f, "Dimension mismatch: expected {}, got {}", expected, actual),
        }
    }
}

impl std::error::Error for LatticeError {}

fn u16_to_trits(val: u16) -> [i8; 11] {
    let mut trits = [0i8; 11];
    let mut v = val;
    for trit in trits.iter_mut() {
        *trit = (v % 3) as i8 - 1;
        v /= 3;
    }
    trits
}

fn u8_to_trits(val: u8) -> [i8; 6] {
    let mut trits = [0i8; 6];
    let mut v = val;
    for trit in trits.iter_mut() {
        *trit = (v % 3) as i8 - 1;
        v /= 3;
    }
    trits
}

fn sponge_hash(inputs: &[&[i8]], output_len: usize) -> Vec<i8> {
    let mut sponge = Sponge385Pub::new();
    for input in inputs {
        sponge.absorb(input);
    }
    sponge.squeeze(output_len)
}

pub fn sample_uniform_ternary(seed: &[i8], n: usize, nonce: u16) -> TernaryPolynomial {
    let nonce_trits = u16_to_trits(nonce);
    let output = sponge_hash(&[seed, &nonce_trits], n * 2);
    let coeffs: Vec<i8> = output.iter()
        .take(n)
        .map(|&t| match t { -1 => -1i8, 0 => 0i8, 1 => 1i8, _ => 0i8 })
        .collect();
    TernaryPolynomial { coeffs, n }
}

pub fn sample_cbd_ternary(seed: &[i8], n: usize, nonce: u16, eta: u8) -> TernaryPolynomial {
    let nonce_trits = u16_to_trits(nonce);
    let eta_trits = u8_to_trits(eta);
    let raw = sponge_hash(&[seed, &nonce_trits, &eta_trits], n * eta as usize * 2);
    let mut coeffs = Vec::with_capacity(n);

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
    let trits_per_poly = n * eta as usize * 2;
    let total_trits = k * trits_per_poly;

    let nonce_trits = u16_to_trits(nonce_offset);
    let eta_trits = u8_to_trits(eta);
    let all_raw = sponge_hash(&[seed, &nonce_trits, &eta_trits], total_trits);

    let mut polys = Vec::with_capacity(k);
    for poly_idx in 0..k {
        let raw_offset = poly_idx * trits_per_poly;
        let raw = &all_raw[raw_offset..raw_offset + trits_per_poly];
        let mut coeffs = Vec::with_capacity(n);

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

        polys.push(TernaryPolynomial { coeffs, n });
    }
    TernaryPolyVec { polys, n }
}

pub fn compress_ternary(poly: &TernaryPolynomial, d: u8) -> Vec<u8> {
    let mut compressed = Vec::with_capacity(poly.n);
    let max_val = 1u16 << d;
    for &c in &poly.coeffs {
        let mapped = ((balanced_to_unsigned(c) as u16 * max_val + 1) / 3) % max_val;
        compressed.push(mapped as u8);
    }
    compressed
}

pub fn decompress_ternary(compressed: &[u8], n: usize, d: u8) -> TernaryPolynomial {
    let max_val = 1u16 << d;
    let mut coeffs = Vec::with_capacity(n);
    for &c in compressed.iter().take(n) {
        let mapped = ((c as u16 * 3 + (max_val / 2)) / max_val) % 3;
        coeffs.push(unsigned_to_balanced(mapped as u8));
    }
    TernaryPolynomial { coeffs, n }
}

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

pub type NttPoly = [u32; NCNTT_N];

#[derive(Debug, Clone)]
pub struct NttMatrix {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<Vec<NttPoly>>,
}

impl NttMatrix {
    pub fn mul_vec(&self, vec: &TernaryPolyVec) -> Result<TernaryPolyVec, LatticeError> {
        if self.cols != vec.len() {
            return Err(LatticeError::DimensionMismatch {
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
    fn test_ring_mul_identity() {
        let n = 4;
        let mut id = TernaryPolynomial::new(n);
        id.coeffs[0] = 1;
        let p = TernaryPolynomial::from_coeffs(vec![1, -1, 0, 1]).unwrap();
        let result = p.ring_mul_karatsuba(&id).unwrap();
        assert_eq!(result.coeffs, p.coeffs);
    }

    #[test]
    fn test_ring_mul_reduction() {
        let n = 4;
        let mut a = TernaryPolynomial::new(n);
        a.coeffs[3] = 1;
        let mut b = TernaryPolynomial::new(n);
        b.coeffs[1] = 1;
        let result = a.ring_mul_karatsuba(&b).unwrap();
        assert_eq!(result.coeffs[0], -1, "x^4 = -1 in Z[X]/(X^4+1)");
    }

    #[test]
    fn test_ring_mul_commutativity() {
        let a = TernaryPolynomial::from_coeffs(vec![1, 0, -1, 1]).unwrap();
        let b = TernaryPolynomial::from_coeffs(vec![0, 1, 1, -1]).unwrap();
        let ab = a.ring_mul_karatsuba(&b).unwrap();
        let ba = b.ring_mul_karatsuba(&a).unwrap();
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
                assert_eq!(m.entries[i][j].n, 8);
                assert!(m.entries[i][j].coeffs.iter().all(|&c| c >= -1 && c <= 1));
            }
        }
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
    fn test_ncntt_constants() {
        assert_eq!(NCNTT_Q, 12289);
        assert_eq!(NCNTT_PSI, 3400);
        let psi_256 = ncntt_mod_pow(NCNTT_PSI as u64, 256, NCNTT_Q as u64) as u32;
        assert_eq!(psi_256, NCNTT_Q - 1);
        let psi_512 = ncntt_mod_pow(NCNTT_PSI as u64, 512, NCNTT_Q as u64) as u32;
        assert_eq!(psi_512, 1);
    }

    #[test]
    fn test_ncntt_roundtrip() {
        let coeffs: Vec<i8> = (0..256).map(|i| [0, 1, -1][i % 3]).collect();
        let ntt = ternary_to_ntt(&coeffs);
        let back = ntt_to_ternary(&ntt);
        assert_eq!(coeffs, back);
    }

    #[test]
    fn test_ncntt_mul_matches_karatsuba() {
        let a_coeffs: Vec<i8> = (0..256).map(|i| match i % 7 { 0 => 1, 1 => -1, _ => 0 }).collect();
        let b_coeffs: Vec<i8> = (0..256).map(|i| match i % 5 { 0 => -1, 2 => 1, _ => 0 }).collect();
        let a = TernaryPolynomial::from_coeffs_unchecked(a_coeffs);
        let b = TernaryPolynomial::from_coeffs_unchecked(b_coeffs);
        let karatsuba = a.ring_mul_karatsuba(&b).unwrap();

        let a_ntt = ternary_to_ntt(&a.coeffs);
        let b_ntt = ternary_to_ntt(&b.coeffs);
        let mut c_ntt = [0u32; NCNTT_N];
        for i in 0..NCNTT_N {
            c_ntt[i] = ncntt_modmul(a_ntt[i], b_ntt[i]);
        }
        let ntt_result = ntt_to_ternary(&c_ntt);
        assert_eq!(karatsuba.coeffs, ntt_result);
    }

    #[test]
    fn test_ntt_matrix_mul_vec() {
        let rows = 2;
        let cols = 2;
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

        let karatsuba_result = {
            let mut result = TernaryPolyVec::new(rows, n);
            for i in 0..rows {
                let mut sum = TernaryPolynomial::new(n);
                for j in 0..cols {
                    let product = mat.entries[i][j].ring_mul_karatsuba(&v.polys[j]).unwrap();
                    sum = sum.add(&product).unwrap();
                }
                result.polys[i] = sum;
            }
            result
        };

        let ntt_mat = mat.to_ntt();
        let ntt_result = ntt_mat.mul_vec(&v).unwrap();

        for i in 0..rows {
            assert_eq!(
                karatsuba_result.polys[i].coeffs, ntt_result.polys[i].coeffs,
                "NTT matrix mul must match Karatsuba for row {}", i
            );
        }
    }
}
