//! XPlenum Post-Quantum Cryptography API (Task 8C.5)
//!
//! Hardware-accelerated ML-KEM (Kyber) and ML-DSA (Dilithium)
//! primitives via XPlenum PQC custom instructions.

use core::arch::asm;

/// Kyber modulus q = 3329
pub const KYBER_Q: u32 = 3329;
/// Dilithium modulus q = 8380417
pub const DILITHIUM_Q: u32 = 8380417;
/// Kyber polynomial degree
pub const KYBER_N: usize = 256;

/// PQC parameter set identifier
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ParamSet {
    Kyber512   = 0,
    Kyber768   = 1,
    Kyber1024  = 2,
    Dilithium2 = 3,
    Dilithium3 = 4,
    Dilithium5 = 5,
}

/// Configure the PQC unit's active parameter set via CSR.
pub fn configure_pqc(params: ParamSet) {
    let q: u64 = match params {
        ParamSet::Kyber512 | ParamSet::Kyber768 | ParamSet::Kyber1024 =>
            KYBER_Q as u64,
        ParamSet::Dilithium2 | ParamSet::Dilithium3 | ParamSet::Dilithium5 =>
            DILITHIUM_Q as u64,
    };
    let config = q | ((params as u64) << 16);

    unsafe {
        asm!(
            "csrw 0x806, {val}",
            val = in(reg) config,
        );
    }
}

/// NTT butterfly: (a + w*b, a - w*b) mod q
/// rs1[31:0] = a, rs2[31:0] = b, rs2[63:32] = twiddle w
/// Returns rd[31:0] = a+wb, rd[63:32] = a-wb
#[inline(always)]
pub unsafe fn pqc_ntt_butterfly(a: u32, b: u32, twiddle: u32) -> (u32, u32) {
    let rs1 = a as u64;
    let rs2 = (b as u64) | ((twiddle as u64) << 32);
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x20, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) rs1,
        rs2 = in(reg) rs2,
    );
    (rd as u32, (rd >> 32) as u32)
}

/// Inverse NTT butterfly: (a + b, w*(a - b)) mod q
#[inline(always)]
pub unsafe fn pqc_intt_butterfly(a: u32, b: u32, twiddle: u32) -> (u32, u32) {
    let rs1 = a as u64;
    let rs2 = (b as u64) | ((twiddle as u64) << 32);
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x21, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) rs1,
        rs2 = in(reg) rs2,
    );
    (rd as u32, (rd >> 32) as u32)
}

/// Modular reduction: a mod q (Barrett)
#[inline(always)]
pub unsafe fn pqc_mod_reduce(a: u64) -> u32 {
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x22, {rd}, {rs1}, x0",
        rd  = out(reg) rd,
        rs1 = in(reg) a,
    );
    rd as u32
}

/// Modular multiplication: (a * b) mod q (Montgomery)
#[inline(always)]
pub unsafe fn pqc_mod_mul(a: u32, b: u32) -> u32 {
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x23, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) a as u64,
        rs2 = in(reg) b as u64,
    );
    rd as u32
}

/// Modular addition: (a + b) mod q
#[inline(always)]
pub unsafe fn pqc_mod_add(a: u32, b: u32) -> u32 {
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x24, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) a as u64,
        rs2 = in(reg) b as u64,
    );
    rd as u32
}

/// CBD sampling: sample from centered binomial distribution
#[inline(always)]
pub unsafe fn pqc_cbd_sample(random_bits: u64, eta: u8) -> u64 {
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x25, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) random_bits,
        rs2 = in(reg) eta as u64,
    );
    rd
}

/// Rejection sampling: filter candidates against q
#[inline(always)]
pub unsafe fn pqc_rejection_sample(candidates: u64) -> u64 {
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x26, {rd}, {rs1}, x0",
        rd  = out(reg) rd,
        rs1 = in(reg) candidates,
    );
    rd
}

/// Polynomial multiply-accumulate: acc + a*b mod q
/// rs1[31:0] = a, rs1[63:32] = acc, rs2[31:0] = b
#[inline(always)]
pub unsafe fn pqc_poly_mac(a: u32, b: u32, acc: u32) -> u32 {
    let rs1 = (a as u64) | ((acc as u64) << 32);
    let rd: u64;
    asm!(
        ".insn r 0b0101011, 4, 0x27, {rd}, {rs1}, {rs2}",
        rd  = out(reg) rd,
        rs1 = in(reg) rs1,
        rs2 = in(reg) b as u64,
    );
    rd as u32
}

/// Kyber NTT twiddle factors (zetas) in Montgomery form.
/// Precomputed: zeta_i = pow(17, bit_reverse(i)) * R mod q
/// where R = 2^16 (Montgomery constant) and 17 is primitive root mod 3329.
const KYBER_ZETAS: [u32; 128] = [
    2285, 2571, 2970, 1812, 1493, 1422, 287, 202,
    3158, 622, 1577, 182, 962, 2127, 1855, 1468,
    573, 2004, 264, 383, 2500, 1458, 1727, 3199,
    2648, 1017, 732, 608, 1787, 411, 3124, 1758,
    1223, 652, 2777, 1015, 2036, 1491, 3047, 1785,
    516, 3321, 3009, 2663, 1711, 2167, 126, 1469,
    2476, 3239, 3058, 830, 107, 1908, 3082, 2378,
    2931, 961, 1821, 2604, 448, 2264, 677, 2054,
    2226, 430, 555, 843, 2078, 871, 1550, 105,
    422, 587, 177, 3094, 3038, 2869, 1574, 1653,
    3083, 778, 1159, 3182, 2552, 1483, 2727, 1119,
    1739, 644, 2457, 349, 418, 329, 3173, 3254,
    817, 1097, 603, 610, 1322, 2044, 1864, 384,
    2114, 3193, 1218, 1994, 2455, 220, 2142, 1670,
    2144, 1799, 2051, 794, 1819, 2475, 2459, 478,
    3221, 3116, 830, 107, 1908, 3082, 2378, 2931,
];

/// Kyber polynomial type
pub struct KyberPoly {
    pub coeffs: [u32; KYBER_N],
}

impl KyberPoly {
    /// Create zero polynomial
    pub fn zero() -> Self {
        KyberPoly { coeffs: [0u32; KYBER_N] }
    }

    /// Forward NTT using hardware-accelerated butterflies.
    ///
    /// Transforms polynomial from normal domain to NTT domain.
    /// Uses Cooley-Tukey decimation-in-time with bit-reversed twiddle factors.
    pub fn ntt(&mut self) {
        let mut k: usize = 1;
        let mut len: usize = 128;

        while len >= 2 {
            let mut start: usize = 0;
            while start < KYBER_N {
                let zeta = KYBER_ZETAS[k];
                k += 1;

                let mut j = start;
                while j < start + len {
                    let (lo, hi) = unsafe {
                        pqc_ntt_butterfly(self.coeffs[j], self.coeffs[j + len], zeta)
                    };
                    self.coeffs[j]       = lo;
                    self.coeffs[j + len] = hi;
                    j += 1;
                }
                start += 2 * len;
            }
            len >>= 1;
        }
    }

    /// Inverse NTT using hardware-accelerated butterflies.
    pub fn intt(&mut self) {
        let mut k: usize = 127;
        let mut len: usize = 2;

        while len <= 128 {
            let mut start: usize = 0;
            while start < KYBER_N {
                let zeta = KYBER_ZETAS[k];
                k = k.wrapping_sub(1);

                let mut j = start;
                while j < start + len {
                    let (lo, hi) = unsafe {
                        pqc_intt_butterfly(self.coeffs[j], self.coeffs[j + len], zeta)
                    };
                    self.coeffs[j]       = lo;
                    self.coeffs[j + len] = hi;
                    j += 1;
                }
                start += 2 * len;
            }
            len <<= 1;
        }

        // Final scaling by N^-1 mod q
        // N^-1 mod 3329 = 3303 (in Montgomery form: 3303 * R mod q)
        let n_inv: u32 = 3303;
        for i in 0..KYBER_N {
            self.coeffs[i] = unsafe { pqc_mod_mul(self.coeffs[i], n_inv) };
        }
    }

    /// Pointwise multiplication in NTT domain.
    pub fn pointwise_mul(&self, other: &KyberPoly) -> KyberPoly {
        let mut result = KyberPoly::zero();
        for i in 0..KYBER_N {
            result.coeffs[i] = unsafe {
                pqc_mod_mul(self.coeffs[i], other.coeffs[i])
            };
        }
        result
    }

    /// Coefficient-wise addition.
    pub fn add(&self, other: &KyberPoly) -> KyberPoly {
        let mut result = KyberPoly::zero();
        for i in 0..KYBER_N {
            result.coeffs[i] = unsafe {
                pqc_mod_add(self.coeffs[i], other.coeffs[i])
            };
        }
        result
    }
}
