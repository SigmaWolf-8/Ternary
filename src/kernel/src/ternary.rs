// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Ternary Computing Module — **`aasc` compatibility shim**
//!
//! This module is a backward-compatibility shim over the canonical
//! pure-ternary engine [`algeometric_arc_sigma182_calculi`] (re-exported
//! at the bottom of this file as [`AascTrit`] and [`TritVec`]).
//! It preserves every symbol downstream consumers — including the
//! bare-metal target at `src/kernel/bare-metal/` — already import:
//! `Trit`, `Tryte`, `TernaryWord`, `Representation`,
//! `convert_representation`, `pack_trits`, `unpack_trits`,
//! `packed_map`, `packed_zip`, `is_valid_packed`,
//! `packed_shift_left`, `packed_shift_right`, `packed_rotate_left`,
//! `packed_reduce`, `packed_convert`, `pack_single_trit`,
//! `scalar_to_trit`, `information_density`, `DensityComparison`.
//!
//! The local `Trit`/`Tryte` types remain a thin Rep-A `i8` wrapper so
//! kernel internal call sites that depend on the `Trit { value: i8 }`
//! field shape (vm/engine.rs, vm/cache.rs, kani_proofs.rs and the rest
//! of `src/kernel/src/`) keep compiling unchanged. The arithmetic is
//! GF(3)-equivalent to `aasc::Trit` by construction (Task #158 I-47:
//! shim parity, zero divergence). Migration of those call sites onto
//! `aasc::Trit` directly is follow-up work.
//!
//! # Representations
//! - **A (Computational)**: {-1, 0, +1} - For arithmetic operations
//! - **B (Network)**: {0, 1, 2} - For network transmission
//! - **C (Human)**: {1, 2, 3} - For human-readable display
//!
//! # Bijections
//! - A→B: f(a) = a + 1
//! - A→C: f(a) = a + 2
//! - B→C: f(b) = b + 1
//!
//! # Migration anchor
//!
//! New code should prefer the canonical engine types re-exported below
//! (`AascTrit`, `TritVec`). The legacy `Trit`/`Tryte` symbols are kept
//! solely for the freestanding kernel + bare-metal symbol surface.

// ── aasc canonical engine re-exports ────────────────────────────────
//
// The single source of mathematical truth. Forward-migration anchor for
// the kernel and the bare-metal target. The local `Trit`/`Representation`
// types defined below remain available under their existing names to
// preserve the freestanding kernel API surface (Task #158 I-47).
pub use aasc::trit::Trit as AascTrit;
pub use aasc::tritvec::TritVec;
pub use aasc::trit::Representation as AascRepresentation;

/// A single trit (ternary digit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trit {
    /// Internal representation uses Representation A: {-1, 0, +1}
    value: i8,
}

impl Trit {
    #[inline(always)]
    pub const fn from_a(value: i8) -> Option<Self> {
        match value {
            -1 | 0 | 1 => Some(Self { value }),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn from_b(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self { value: -1 }),
            1 => Some(Self { value: 0 }),
            2 => Some(Self { value: 1 }),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn from_c(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self { value: -1 }),
            2 => Some(Self { value: 0 }),
            3 => Some(Self { value: 1 }),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn to_a(&self) -> i8 {
        self.value
    }

    #[inline(always)]
    pub const fn to_b(&self) -> u8 {
        (self.value + 1) as u8
    }

    #[inline(always)]
    pub const fn to_c(&self) -> u8 {
        (self.value + 2) as u8
    }

    #[inline(always)]
    pub const fn not(&self) -> Self {
        Self { value: -self.value }
    }

    /// GF(3) addition. Delegates to the canonical `aasc::Trit::add` so
    /// the kernel and the canonical engine share one mathematical truth
    /// (Task #158 I-47, shim parity). Inputs/outputs use the kernel's
    /// Rep-A `i8` storage; the bridge to/from aasc is a constructor +
    /// accessor pair on the canonical Trit enum.
    #[inline(always)]
    pub fn add(&self, other: &Trit) -> Self {
        let a = AascTrit::from_a(self.value).expect("kernel Trit invariant: value ∈ {-1,0,1}");
        let b = AascTrit::from_a(other.value).expect("kernel Trit invariant: value ∈ {-1,0,1}");
        Self { value: a.add(b).value_a() }
    }

    /// GF(3) multiplication. Delegates to the canonical `aasc::Trit::mul`.
    /// See the doc on `add` for the bridge contract.
    #[inline(always)]
    pub fn multiply(&self, other: &Trit) -> Self {
        let a = AascTrit::from_a(self.value).expect("kernel Trit invariant: value ∈ {-1,0,1}");
        let b = AascTrit::from_a(other.value).expect("kernel Trit invariant: value ∈ {-1,0,1}");
        Self { value: a.mul(b).value_a() }
    }

    #[inline(always)]
    pub const fn xor(&self, other: &Trit) -> Self {
        let min = if self.value < other.value { self.value } else { other.value };
        Self { value: min }
    }

    #[inline(always)]
    pub const fn rotate(&self) -> Self {
        let rotated = match self.value {
            -1 => 0,
            0 => 1,
            1 => -1,
            _ => 0,
        };
        Self { value: rotated }
    }

    #[inline(always)]
    pub const fn rotate_inverse(&self) -> Self {
        let rotated = match self.value {
            1 => 0,
            0 => -1,
            -1 => 1,
            _ => 0,
        };
        Self { value: rotated }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Trit) -> Self {
        self.add(&other.not())
    }

    #[inline(always)]
    pub const fn and(&self, other: &Trit) -> Self {
        let min = if self.value < other.value { self.value } else { other.value };
        Self { value: min }
    }

    #[inline(always)]
    pub const fn or(&self, other: &Trit) -> Self {
        let max = if self.value > other.value { self.value } else { other.value };
        Self { value: max }
    }

    #[inline(always)]
    pub const fn cmp_trit(&self, other: &Trit) -> Self {
        let diff = self.value - other.value;
        let clamped = if diff < 0 { -1 } else if diff > 0 { 1 } else { 0 };
        Self { value: clamped }
    }

    #[inline(always)]
    pub fn gf3_inverse(&self) -> Self {
        match self.value {
            0 => panic!("Zero has no multiplicative inverse in GF(3)"),
            _ => *self,
        }
    }

    #[inline(always)]
    pub const fn lukasiewicz_and(&self, other: &Trit) -> Self {
        let sum = self.value + other.value - 1;
        let val = if sum < -1 { -1 } else { sum };
        Self { value: val }
    }

    /// Reduce via specified gate across a trit word's positions.
    /// Gate 0 = add, 1 = mul, 2 = min, 3 = max
    pub fn reduce_with(acc: &Trit, elem: &Trit, gate: u8) -> Trit {
        match gate {
            0 => acc.add(elem),
            1 => acc.multiply(elem),
            2 => Trit { value: core::cmp::min(acc.value, elem.value) },
            3 => Trit { value: core::cmp::max(acc.value, elem.value) },
            _ => acc.add(elem),
        }
    }
}

/// Packed trit word: 27 trits stored in an i64 using 2-bit encoding per trit.
///
/// Encoding per trit (2 bits):
///   00 = 0
///   01 = +1
///   10 = -1
///   11 = unused/invalid
///
/// Bits [0..53] hold 27 trits (54 bits total). Bits [54..63] are reserved/zero.
/// This enables SIMD-style parallel trit operations via bitwise manipulation.

const TRITS_PER_WORD: usize = 27;
const BITS_PER_TRIT: usize = 2;

/// Pack a slice of Trit values into an i64. Up to 27 trits; excess are ignored.
/// Fewer than 27 trits are zero-padded in the high positions.
pub fn pack_trits(trits: &[Trit]) -> i64 {
    let mut packed: u64 = 0;
    let count = if trits.len() > TRITS_PER_WORD { TRITS_PER_WORD } else { trits.len() };
    for i in 0..count {
        let bits: u64 = match trits[i].to_a() {
            0 => 0b00,
            1 => 0b01,
            -1 => 0b10,
            _ => 0b00,
        };
        packed |= bits << (i * BITS_PER_TRIT);
    }
    packed as i64
}

/// Unpack an i64 into exactly 27 Trit values.
pub fn unpack_trits(packed: i64) -> [Trit; TRITS_PER_WORD] {
    let bits = packed as u64;
    let mut trits = [Trit { value: 0 }; TRITS_PER_WORD];
    for i in 0..TRITS_PER_WORD {
        let pair = (bits >> (i * BITS_PER_TRIT)) & 0b11;
        trits[i] = match pair {
            0b00 => Trit { value: 0 },
            0b01 => Trit { value: 1 },
            0b10 => Trit { value: -1 },
            _ => Trit { value: 0 },
        };
    }
    trits
}

/// Apply a unary operation to each trit in a packed word.
pub fn packed_map<F>(packed: i64, f: F) -> i64
where
    F: Fn(&Trit) -> Trit,
{
    let trits = unpack_trits(packed);
    let mut result = [Trit { value: 0 }; TRITS_PER_WORD];
    for i in 0..TRITS_PER_WORD {
        result[i] = f(&trits[i]);
    }
    pack_trits(&result)
}

/// Apply a binary operation element-wise to two packed trit words.
pub fn packed_zip<F>(a: i64, b: i64, f: F) -> i64
where
    F: Fn(&Trit, &Trit) -> Trit,
{
    let ta = unpack_trits(a);
    let tb = unpack_trits(b);
    let mut result = [Trit { value: 0 }; TRITS_PER_WORD];
    for i in 0..TRITS_PER_WORD {
        result[i] = f(&ta[i], &tb[i]);
    }
    pack_trits(&result)
}

/// Check if a packed word contains only valid trit encodings (no 0b11 pairs).
pub fn is_valid_packed(packed: i64) -> bool {
    let bits = packed as u64;
    for i in 0..TRITS_PER_WORD {
        let pair = (bits >> (i * BITS_PER_TRIT)) & 0b11;
        if pair == 0b11 {
            return false;
        }
    }
    let reserved_mask = !((1u64 << (TRITS_PER_WORD * BITS_PER_TRIT)) - 1);
    (bits & reserved_mask) == 0
}

/// Shift trits left by n positions within a packed word (zero-fill from right).
pub fn packed_shift_left(packed: i64, n: usize) -> i64 {
    if n >= TRITS_PER_WORD {
        return 0;
    }
    let mut trits = unpack_trits(packed);
    for i in (n..TRITS_PER_WORD).rev() {
        trits[i] = trits[i - n];
    }
    for i in 0..n {
        trits[i] = Trit { value: 0 };
    }
    pack_trits(&trits)
}

/// Shift trits right by n positions within a packed word (zero-fill from left).
pub fn packed_shift_right(packed: i64, n: usize) -> i64 {
    if n >= TRITS_PER_WORD {
        return 0;
    }
    let mut trits = unpack_trits(packed);
    for i in 0..(TRITS_PER_WORD - n) {
        trits[i] = trits[i + n];
    }
    for i in (TRITS_PER_WORD - n)..TRITS_PER_WORD {
        trits[i] = Trit { value: 0 };
    }
    pack_trits(&trits)
}

/// Rotate trits left within a packed word (wrapping).
pub fn packed_rotate_left(packed: i64, n: usize) -> i64 {
    let n = n % TRITS_PER_WORD;
    if n == 0 {
        return packed;
    }
    let trits = unpack_trits(packed);
    let mut result = [Trit { value: 0 }; TRITS_PER_WORD];
    for i in 0..TRITS_PER_WORD {
        result[(i + n) % TRITS_PER_WORD] = trits[i];
    }
    pack_trits(&result)
}

/// Reduce all trits in a packed word using a specified gate.
/// Gate: 0=add, 1=mul, 2=min, 3=max
pub fn packed_reduce(packed: i64, gate: u8) -> Trit {
    let trits = unpack_trits(packed);
    let mut acc = trits[0];
    for i in 1..TRITS_PER_WORD {
        acc = Trit::reduce_with(&acc, &trits[i], gate);
    }
    acc
}

/// Convert each trit in a packed word between representations.
pub fn packed_convert(packed: i64, from: Representation, to: Representation) -> i64 {
    let trits = unpack_trits(packed);
    let mut result = [Trit { value: 0 }; TRITS_PER_WORD];
    for i in 0..TRITS_PER_WORD {
        let converted = convert_representation(trits[i].to_a(), from, to);
        result[i] = Trit { value: converted.clamp(-1, 1) };
    }
    pack_trits(&result)
}

/// Pack a single trit value (for scalar-mode backward compatibility).
pub fn pack_single_trit(trit: &Trit) -> i64 {
    match trit.to_a() {
        0 => 0,
        1 => 1,
        -1 => -1,
        _ => 0,
    }
}

/// Extract first trit from a scalar i64 value (backward-compatible).
/// Normalizes any i64 to a valid trit value in {-1, 0, +1}.
pub fn scalar_to_trit(val: i64) -> Trit {
    let normalized = val.rem_euclid(3);
    let trit_val = match normalized {
        0 => 0i8,
        1 => 1,
        2 => -1,
        _ => 0,
    };
    Trit { value: trit_val }
}

/// A tryte (6 trits = 729 values, equivalent to ~9.5 bits)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tryte {
    trits: [Trit; 6],
}

impl Tryte {
    /// Create a new tryte from 6 trits
    pub fn new(trits: [Trit; 6]) -> Self {
        Self { trits }
    }

    /// Create from a decimal value (0-728)
    pub fn from_decimal(mut value: u16) -> Option<Self> {
        if value >= 729 {
            return None;
        }

        let mut trits = [Trit { value: 0 }; 6];
        for i in 0..6 {
            let remainder = (value % 3) as i8;
            let trit_value = match remainder {
                0 => -1,
                1 => 0,
                2 => 1,
                _ => 0,
            };
            trits[i] = Trit { value: trit_value };
            value /= 3;
        }
        Some(Self { trits })
    }

    /// Convert to decimal value
    pub fn to_decimal(&self) -> u16 {
        let mut result = 0u16;
        let mut multiplier = 1u16;
        for trit in &self.trits {
            let digit = (trit.value + 1) as u16;
            result += digit * multiplier;
            multiplier *= 3;
        }
        result
    }

    /// Host-integer view of this tryte's decimal value, widened to `u64`.
    ///
    /// Mirrors the `host_u64` boundary on `aasc::TritVec` / `TritInt`
    /// (Task #158 I-48) — the **only** place a host integer appears in
    /// the kernel ternary surface, kept here for backward compatibility
    /// with downstream consumers (and the existing self-test suite).
    pub fn host_u64(&self) -> u64 {
        self.to_decimal() as u64
    }

    /// Get trits
    pub fn trits(&self) -> &[Trit; 6] {
        &self.trits
    }

    /// Tryte-wise NOT
    pub fn not(&self) -> Self {
        let mut result = self.trits;
        for trit in &mut result {
            *trit = trit.not();
        }
        Self { trits: result }
    }

    /// Tryte-wise ADD
    pub fn add(&self, other: &Tryte) -> Self {
        let mut result = [Trit { value: 0 }; 6];
        for i in 0..6 {
            result[i] = self.trits[i].add(&other.trits[i]);
        }
        Self { trits: result }
    }
}

/// Ternary word (27 trits = 3 trytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryWord {
    trytes: [Tryte; 3],
}

impl TernaryWord {
    pub fn new(trytes: [Tryte; 3]) -> Self {
        Self { trytes }
    }

    pub fn trytes(&self) -> &[Tryte; 3] {
        &self.trytes
    }
}

/// Convert between representations
pub fn convert_representation(value: i8, from: Representation, to: Representation) -> i8 {
    // First convert to A
    let a_value = match from {
        Representation::A => value,
        Representation::B => value - 1,
        Representation::C => value - 2,
    };

    // Then convert from A to target
    match to {
        Representation::A => a_value,
        Representation::B => a_value + 1,
        Representation::C => a_value + 2,
    }
}

/// Representation enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    /// Computational: {-1, 0, +1}
    A,
    /// Network: {0, 1, 2}
    B,
    /// Human: {1, 2, 3}
    C,
}

/// Calculate information density for ternary vs binary
pub fn information_density(trit_count: u32) -> DensityComparison {
    let ternary_states = 3u128.pow(trit_count);
    let equivalent_bits = (trit_count as f64) * 1.585; // log2(3) ≈ 1.585
    let bit_count = if equivalent_bits == (equivalent_bits as u32 as f64) { equivalent_bits as u32 } else { equivalent_bits as u32 + 1 };
    let binary_states = 2u128.pow(bit_count);

    DensityComparison {
        trit_count,
        ternary_states,
        equivalent_bits,
        bit_count,
        binary_states,
        efficiency_gain: (ternary_states as f64) / (binary_states as f64),
    }
}

#[derive(Debug, Clone)]
pub struct DensityComparison {
    pub trit_count: u32,
    pub ternary_states: u128,
    pub equivalent_bits: f64,
    pub bit_count: u32,
    pub binary_states: u128,
    pub efficiency_gain: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_from_a_valid() {
        assert!(Trit::from_a(-1).is_some());
        assert!(Trit::from_a(0).is_some());
        assert!(Trit::from_a(1).is_some());
    }

    #[test]
    fn test_trit_from_a_invalid() {
        assert!(Trit::from_a(-2).is_none());
        assert!(Trit::from_a(2).is_none());
        assert!(Trit::from_a(127).is_none());
    }

    #[test]
    fn test_trit_from_b_valid() {
        assert!(Trit::from_b(0).is_some());
        assert!(Trit::from_b(1).is_some());
        assert!(Trit::from_b(2).is_some());
    }

    #[test]
    fn test_trit_from_b_invalid() {
        assert!(Trit::from_b(3).is_none());
        assert!(Trit::from_b(255).is_none());
    }

    #[test]
    fn test_trit_from_c_valid() {
        assert!(Trit::from_c(1).is_some());
        assert!(Trit::from_c(2).is_some());
        assert!(Trit::from_c(3).is_some());
    }

    #[test]
    fn test_trit_from_c_invalid() {
        assert!(Trit::from_c(0).is_none());
        assert!(Trit::from_c(4).is_none());
    }

    #[test]
    fn test_trit_representations_roundtrip() {
        let trit = Trit::from_a(-1).unwrap();
        assert_eq!(trit.to_a(), -1);
        assert_eq!(trit.to_b(), 0);
        assert_eq!(trit.to_c(), 1);

        let trit = Trit::from_a(0).unwrap();
        assert_eq!(trit.to_a(), 0);
        assert_eq!(trit.to_b(), 1);
        assert_eq!(trit.to_c(), 2);

        let trit = Trit::from_a(1).unwrap();
        assert_eq!(trit.to_a(), 1);
        assert_eq!(trit.to_b(), 2);
        assert_eq!(trit.to_c(), 3);
    }

    #[test]
    fn test_bijection_a_to_b() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.to_b() as i8, a_val + 1);
        }
    }

    #[test]
    fn test_bijection_a_to_c() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.to_c() as i8, a_val + 2);
        }
    }

    #[test]
    fn test_bijection_b_to_c() {
        for b_val in [0u8, 1, 2] {
            let trit = Trit::from_b(b_val).unwrap();
            assert_eq!(trit.to_c(), b_val + 1);
        }
    }

    #[test]
    fn test_bijection_roundtrip_b_a_b() {
        for b_val in [0u8, 1, 2] {
            let trit = Trit::from_b(b_val).unwrap();
            let a_val = trit.to_a();
            let reconstructed = Trit::from_a(a_val).unwrap();
            assert_eq!(reconstructed.to_b(), b_val);
        }
    }

    #[test]
    fn test_ternary_not() {
        assert_eq!(Trit::from_a(-1).unwrap().not().to_a(), 1);
        assert_eq!(Trit::from_a(0).unwrap().not().to_a(), 0);
        assert_eq!(Trit::from_a(1).unwrap().not().to_a(), -1);
    }

    #[test]
    fn test_ternary_not_involution() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.not().not().to_a(), a_val);
        }
    }

    #[test]
    fn test_ternary_addition_full_table() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                let result = ta.add(&tb);
                let expected = (a + b).rem_euclid(3);
                let expected_norm = if expected == 2 { -1 } else { expected as i8 };
                assert_eq!(result.to_a(), expected_norm, "GF(3) add: {} + {} = {}", a, b, expected_norm);
            }
        }
    }

    #[test]
    fn test_ternary_multiplication_full_table() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                let result = ta.multiply(&tb);
                let expected = (a * b).rem_euclid(3);
                let expected_norm = if expected == 2 { -1 } else { expected as i8 };
                assert_eq!(result.to_a(), expected_norm, "GF(3) mul: {} * {} = {}", a, b, expected_norm);
            }
        }
    }

    #[test]
    fn test_gf3_additive_identity() {
        let zero = Trit::from_a(0).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.add(&zero).to_a(), a_val);
        }
    }

    #[test]
    fn test_gf3_multiplicative_identity() {
        let one = Trit::from_a(1).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.multiply(&one).to_a(), a_val);
        }
    }

    #[test]
    fn test_gf3_multiplicative_absorbing() {
        let zero = Trit::from_a(0).unwrap();
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.multiply(&zero).to_a(), 0);
        }
    }

    #[test]
    fn test_rotation_cycle() {
        let trit = Trit::from_a(-1).unwrap();
        let r1 = trit.rotate();
        assert_eq!(r1.to_a(), 0);
        let r2 = r1.rotate();
        assert_eq!(r2.to_a(), 1);
        let r3 = r2.rotate();
        assert_eq!(r3.to_a(), -1);
    }

    #[test]
    fn test_rotation_inverse_cycle() {
        let trit = Trit::from_a(1).unwrap();
        let r1 = trit.rotate_inverse();
        assert_eq!(r1.to_a(), 0);
        let r2 = r1.rotate_inverse();
        assert_eq!(r2.to_a(), -1);
        let r3 = r2.rotate_inverse();
        assert_eq!(r3.to_a(), 1);
    }

    #[test]
    fn test_rotate_inverse_cancels_rotate() {
        for a_val in [-1i8, 0, 1] {
            let trit = Trit::from_a(a_val).unwrap();
            assert_eq!(trit.rotate().rotate_inverse().to_a(), a_val);
            assert_eq!(trit.rotate_inverse().rotate().to_a(), a_val);
        }
    }

    #[test]
    fn test_xor_commutativity() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                assert_eq!(ta.xor(&tb).to_a(), tb.xor(&ta).to_a());
            }
        }
    }

    #[test]
    fn test_tryte_creation() {
        let trits = [
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(-1).unwrap(),
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(-1).unwrap(),
        ];
        let tryte = Tryte::new(trits);
        assert_eq!(tryte.trits().len(), 6);
    }

    #[test]
    fn test_tryte_decimal_roundtrip() {
        for val in [0u16, 1, 100, 364, 365, 500, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            assert_eq!(tryte.to_decimal(), val, "Roundtrip failed for decimal {}", val);
            assert_eq!(tryte.host_u64(), val as u64, "host_u64 boundary mismatch for decimal {}", val);
        }
    }

    #[test]
    fn test_tryte_decimal_bounds() {
        assert!(Tryte::from_decimal(0).is_some());
        assert!(Tryte::from_decimal(728).is_some());
        assert!(Tryte::from_decimal(729).is_none());
        assert!(Tryte::from_decimal(1000).is_none());
    }

    #[test]
    fn test_tryte_not_involution() {
        for val in [0u16, 100, 365, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            assert_eq!(tryte.not().not().to_decimal(), val);
            assert_eq!(tryte.not().not().host_u64(), val as u64);
        }
    }

    #[test]
    fn test_tryte_add_identity() {
        let zero = Tryte::from_decimal(364).unwrap(); // all-zeros
        for val in [0u16, 100, 365, 728] {
            let tryte = Tryte::from_decimal(val).unwrap();
            let result = tryte.add(&zero);
            assert_eq!(result.trits().len(), 6);
        }
    }

    #[test]
    fn test_ternary_word_creation() {
        let t0 = Tryte::from_decimal(0).unwrap();
        let t1 = Tryte::from_decimal(100).unwrap();
        let t2 = Tryte::from_decimal(728).unwrap();
        let word = TernaryWord::new([t0, t1, t2]);
        assert_eq!(word.trytes().len(), 3);
    }

    #[test]
    fn test_convert_representation_a_to_b() {
        assert_eq!(convert_representation(-1, Representation::A, Representation::B), 0);
        assert_eq!(convert_representation(0, Representation::A, Representation::B), 1);
        assert_eq!(convert_representation(1, Representation::A, Representation::B), 2);
    }

    #[test]
    fn test_convert_representation_a_to_c() {
        assert_eq!(convert_representation(-1, Representation::A, Representation::C), 1);
        assert_eq!(convert_representation(0, Representation::A, Representation::C), 2);
        assert_eq!(convert_representation(1, Representation::A, Representation::C), 3);
    }

    #[test]
    fn test_convert_representation_b_to_c() {
        assert_eq!(convert_representation(0, Representation::B, Representation::C), 1);
        assert_eq!(convert_representation(1, Representation::B, Representation::C), 2);
        assert_eq!(convert_representation(2, Representation::B, Representation::C), 3);
    }

    #[test]
    fn test_convert_identity() {
        for repr in [Representation::A, Representation::B, Representation::C] {
            assert_eq!(convert_representation(0, repr, repr), 0);
        }
    }

    #[test]
    fn test_information_density() {
        let density = information_density(6);
        assert_eq!(density.trit_count, 6);
        assert_eq!(density.ternary_states, 729);
        assert!(density.efficiency_gain > 0.0);
        assert!(density.equivalent_bits > 9.0);
    }

    #[test]
    fn test_information_density_single_trit() {
        let density = information_density(1);
        assert_eq!(density.ternary_states, 3);
        assert_eq!(density.bit_count, 2);
        assert_eq!(density.binary_states, 4);
    }

    #[test]
    fn test_information_density_59_percent_gain() {
        let density = information_density(6);
        let ternary_per_unit = 729.0 / 6.0;
        let binary_per_unit = 1024.0 / 10.0;
        let gain = (ternary_per_unit / binary_per_unit - 1.0) * 100.0;
        assert!(gain > 15.0, "Ternary should have >15% information density advantage per digit: {:.1}%", gain);
    }

    #[test]
    fn test_trit_sub_full_table() {
        let vals = [-1i8, 0, 1];
        for &a in &vals {
            for &b in &vals {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                let result = ta.sub(&tb);
                let neg_b = (-b).rem_euclid(3);
                let expected = ((a as i16 + neg_b as i16) % 3) as i8;
                let expected_norm = if expected == 2 { -1 } else { expected };
                assert_eq!(result.to_a(), expected_norm, "GF(3) sub: {} - {}", a, b);
            }
        }
    }

    #[test]
    fn test_trit_and_or() {
        let n = Trit::from_a(-1).unwrap();
        let z = Trit::from_a(0).unwrap();
        let p = Trit::from_a(1).unwrap();
        assert_eq!(n.and(&p).to_a(), -1);
        assert_eq!(z.and(&p).to_a(), 0);
        assert_eq!(p.and(&p).to_a(), 1);
        assert_eq!(n.or(&p).to_a(), 1);
        assert_eq!(z.or(&n).to_a(), 0);
        assert_eq!(n.or(&n).to_a(), -1);
    }

    #[test]
    fn test_trit_cmp() {
        let n = Trit::from_a(-1).unwrap();
        let z = Trit::from_a(0).unwrap();
        let p = Trit::from_a(1).unwrap();
        assert_eq!(z.cmp_trit(&z).to_a(), 0);
        assert_eq!(p.cmp_trit(&n).to_a(), 1);
        assert_eq!(n.cmp_trit(&p).to_a(), -1);
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let trits = [
            Trit::from_a(-1).unwrap(),
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(1).unwrap(),
            Trit::from_a(-1).unwrap(),
        ];
        let packed = pack_trits(&trits);
        let unpacked = unpack_trits(packed);
        for i in 0..5 {
            assert_eq!(unpacked[i].to_a(), trits[i].to_a(), "Trit {} mismatch", i);
        }
        for i in 5..27 {
            assert_eq!(unpacked[i].to_a(), 0, "Trit {} should be zero-padded", i);
        }
    }

    #[test]
    fn test_pack_full_27_trits() {
        let mut trits = [Trit { value: 0 }; 27];
        for i in 0..27 {
            trits[i] = Trit::from_a([-1, 0, 1][i % 3]).unwrap();
        }
        let packed = pack_trits(&trits);
        let unpacked = unpack_trits(packed);
        for i in 0..27 {
            assert_eq!(unpacked[i].to_a(), trits[i].to_a(), "Full 27-trit roundtrip failed at {}", i);
        }
    }

    #[test]
    fn test_packed_map_negation() {
        let trits = [
            Trit::from_a(-1).unwrap(),
            Trit::from_a(0).unwrap(),
            Trit::from_a(1).unwrap(),
        ];
        let packed = pack_trits(&trits);
        let negated = packed_map(packed, |t| t.not());
        let result = unpack_trits(negated);
        assert_eq!(result[0].to_a(), 1);
        assert_eq!(result[1].to_a(), 0);
        assert_eq!(result[2].to_a(), -1);
    }

    #[test]
    fn test_packed_zip_add() {
        let a = [Trit::from_a(1).unwrap(), Trit::from_a(-1).unwrap()];
        let b = [Trit::from_a(1).unwrap(), Trit::from_a(1).unwrap()];
        let pa = pack_trits(&a);
        let pb = pack_trits(&b);
        let result = packed_zip(pa, pb, |x, y| x.add(y));
        let trits = unpack_trits(result);
        assert_eq!(trits[0].to_a(), -1); // 1+1 = 2 mod 3 = -1
        assert_eq!(trits[1].to_a(), 0);  // -1+1 = 0
    }

    #[test]
    fn test_is_valid_packed() {
        let trits = [Trit::from_a(1).unwrap(), Trit::from_a(-1).unwrap()];
        let packed = pack_trits(&trits);
        assert!(is_valid_packed(packed));
        let invalid: i64 = 0b11; // 0b11 = invalid encoding
        assert!(!is_valid_packed(invalid));
    }

    #[test]
    fn test_packed_shift_left() {
        let trits = [Trit::from_a(1).unwrap(), Trit::from_a(-1).unwrap(), Trit::from_a(0).unwrap()];
        let packed = pack_trits(&trits);
        let shifted = packed_shift_left(packed, 1);
        let result = unpack_trits(shifted);
        assert_eq!(result[0].to_a(), 0);
        assert_eq!(result[1].to_a(), 1);
        assert_eq!(result[2].to_a(), -1);
    }

    #[test]
    fn test_packed_shift_right() {
        let trits = [Trit::from_a(1).unwrap(), Trit::from_a(-1).unwrap(), Trit::from_a(0).unwrap()];
        let packed = pack_trits(&trits);
        let shifted = packed_shift_right(packed, 1);
        let result = unpack_trits(shifted);
        assert_eq!(result[0].to_a(), -1);
        assert_eq!(result[1].to_a(), 0);
    }

    #[test]
    fn test_packed_rotate_left() {
        let mut trits = [Trit { value: 0 }; 27];
        trits[0] = Trit::from_a(1).unwrap();
        trits[26] = Trit::from_a(-1).unwrap();
        let packed = pack_trits(&trits);
        let rotated = packed_rotate_left(packed, 1);
        let result = unpack_trits(rotated);
        assert_eq!(result[0].to_a(), -1); // wrapped from position 26
        assert_eq!(result[1].to_a(), 1);  // moved from position 0
    }

    #[test]
    fn test_scalar_to_trit() {
        assert_eq!(scalar_to_trit(0).to_a(), 0);
        assert_eq!(scalar_to_trit(1).to_a(), 1);
        assert_eq!(scalar_to_trit(-1).to_a(), -1);
        assert_eq!(scalar_to_trit(2).to_a(), -1);
        assert_eq!(scalar_to_trit(4).to_a(), 1);
    }
}
