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
//! Note: GF(3) lacks primitive roots of unity for n=256, so standard
//! NTT-based fast multiplication is not available. Ring multiplication
//! uses schoolbook convolution with X^n+1 reduction. Future work may
//! explore lifting to a larger modulus q with NTT support.
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

pub fn sample_uniform_ternary(seed: &[i8], n: usize, nonce: u16) -> TernaryPolynomial {
    use super::sponge::TernarySponge;
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);
    sponge.absorb(&[(nonce & 0xFF) as i8, ((nonce >> 8) & 0xFF) as i8]);

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

pub fn sample_cbd_ternary(seed: &[i8], n: usize, nonce: u16, eta: u8) -> TernaryPolynomial {
    use super::sponge::TernarySponge;
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);
    sponge.absorb(&[(nonce & 0xFF) as i8, ((nonce >> 8) & 0xFF) as i8, eta as i8]);

    let raw = sponge.squeeze(n * eta as usize * 2);
    let mut coeffs = Vec::with_capacity(n);

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
    let mut polys = Vec::with_capacity(k);
    for i in 0..k {
        polys.push(sample_cbd_ternary(seed, n, nonce_offset + i as u16, eta));
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
}
