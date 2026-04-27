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

//! # Tribonacci Base-3 Generator
//!
//! Native ternary Tribonacci sequence generator for the Salvi Framework.
//!
//! All shared constants are imported from [`crate::constants`] — this module
//! does not define its own copy of any shared value.
//!
//! The Tribonacci recurrence `T(n) = T(n-1) + T(n-2) + T(n-3)` is computed
//! entirely in base-3 trit-vector arithmetic — no decimal intermediaries.
//!
//! ## Representations A, B, C
//!
//! The ternary kernel is **representation-agnostic**. All arithmetic operates
//! on the same underlying integer values, but the digit encoding can be
//! freely switched between three representations:
//!
//! - **A (Balanced):** `{-1, 0, +1}` — symmetric, native to signed arithmetic
//! - **B (Standard):** `{0, 1, 2}` — conventional positional ternary
//! - **C (Bijective):** `{1, 2, 3}` — zero-free, unique representation
//!
//! ## Key Features
//!
//! - **Native trit-vector arithmetic**: Addition with carry propagation tracked
//!   as a first-class operation (carry events map to "jerk" in timing protocols).
//! - **Ternary power alignment detection**: Identifies when T(n) = 3^k, which
//!   produces zero carry propagation and marks optimization windows.
//! - **Representation interchange**: Any TritVec can emit its digits in A, B, or C.
//! - **Tribonacci word generation**: The 3-automatic sequence from the morphism
//!   `0→01, 1→02, 2→0`, used as a canonical test oracle.

use std::fmt;
use crate::constants::TAU_TRIBONACCI;

/// The three ternary representations supported by the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TernaryRepr {
    /// Representation A: Balanced ternary `{-1, 0, +1}`.
    Balanced,
    /// Representation B: Standard ternary `{0, 1, 2}`.
    Standard,
    /// Representation C: Bijective ternary `{1, 2, 3}`.
    Bijective,
}

/// A number represented as a vector of ternary digits (trits).
///
/// Stored least-significant-trit first (little-endian) for efficient
/// carry propagation during addition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TritVec {
    trits: Vec<u8>,
}

/// Result of a ternary addition, including carry metadata.
#[derive(Clone, Debug)]
pub struct TernaryAddResult {
    /// The sum as a trit vector.
    pub sum: TritVec,
    /// Number of carry propagations that occurred during addition.
    pub carry_count: u32,
    /// Maximum carry chain length (consecutive carries without rest).
    pub max_carry_chain: u32,
}

/// Metadata about a Tribonacci term's ternary structure.
#[derive(Clone, Debug)]
pub struct TribonacciTerm {
    /// The index n in the sequence.
    pub index: usize,
    /// The value T(n) as a trit vector in base 3.
    pub value: TritVec,
    /// The decimal value (for reference / validation).
    pub decimal: u64,
    /// True if T(n) is a perfect power of 3 (i.e., 3^k for some k).
    pub is_ternary_power: bool,
    /// If `is_ternary_power`, the exponent k such that T(n) = 3^k.
    pub ternary_exponent: Option<u32>,
    /// Number of carry events from the addition that produced this term.
    pub carry_events: u32,
    /// Number of distinct trit values present in the representation.
    pub trit_diversity: u8,
}

impl TritVec {
    /// Create the zero trit vector.
    pub fn zero() -> Self {
        TritVec { trits: vec![0] }
    }

    /// Create a trit vector from a single trit value (0, 1, or 2).
    pub fn from_trit(t: u8) -> Self {
        assert!(t <= 2, "Trit must be 0, 1, or 2; got {}", t);
        TritVec { trits: vec![t] }
    }

    /// Create a trit vector from a decimal value.
    pub fn from_decimal(mut n: u64) -> Self {
        if n == 0 {
            return Self::zero();
        }
        let mut trits = Vec::new();
        while n > 0 {
            trits.push((n % 3) as u8);
            n /= 3;
        }
        TritVec { trits }
    }

    /// Convert back to decimal for validation.
    pub fn to_decimal(&self) -> u64 {
        let mut result: u64 = 0;
        let mut power: u64 = 1;
        for &trit in &self.trits {
            result += trit as u64 * power;
            power *= 3;
        }
        result
    }

    /// Host-u64 boundary alias on `TritVec`.  Mirrors the
    /// `TritInt::host_u64` boundary naming so call sites can rely on a
    /// single accessor name across the two ternary carriers.
    #[inline]
    pub fn host_u64(&self) -> u64 { self.to_decimal() }

    /// Number of trits (significant digits) in this representation.
    pub fn trit_length(&self) -> usize {
        for i in (0..self.trits.len()).rev() {
            if self.trits[i] != 0 {
                return i + 1;
            }
        }
        1
    }

    /// Get the trit at position `i` (0 = least significant).
    pub fn trit_at(&self, i: usize) -> u8 {
        if i < self.trits.len() { self.trits[i] } else { 0 }
    }

    /// Returns the trits as a slice, most-significant first (display order).
    pub fn trits_msb_first(&self) -> Vec<u8> {
        let len = self.trit_length();
        let mut result: Vec<u8> = self.trits[..len].to_vec();
        result.reverse();
        result
    }

    /// Emit digits in **Representation B** (Standard: `{0, 1, 2}`).
    pub fn to_repr_b(&self) -> Vec<u8> {
        self.trits_msb_first()
    }

    /// Emit digits in **Representation A** (Balanced: `{-1, 0, +1}`).
    pub fn to_repr_a(&self) -> Vec<i8> {
        let len = self.trit_length();
        let mut balanced: Vec<i8> = Vec::with_capacity(len + 1);
        let mut carry: i8 = 0;

        for i in 0..len {
            let d = self.trits[i] as i8 + carry;
            carry = 0;

            let bal = if d == 0 {
                0
            } else if d == 1 {
                1
            } else if d == 2 {
                carry = 1;
                -1
            } else if d == 3 {
                carry = 1;
                0
            } else {
                unreachable!("Unexpected digit value: {}", d)
            };

            balanced.push(bal);
        }

        if carry > 0 {
            balanced.push(carry);
        }

        while balanced.len() > 1 && *balanced.last().unwrap() == 0 {
            balanced.pop();
        }

        balanced.reverse();
        balanced
    }

    /// Emit digits in **Representation C** (Bijective: `{1, 2, 3}`).
    pub fn to_repr_c(&self) -> Vec<u8> {
        if self.host_u64() == 0 {
            return vec![];
        }

        let mut n = self.host_u64();
        let mut digits: Vec<u8> = Vec::new();

        while n > 0 {
            n -= 1;
            let d = (n % 3) as u8 + 1;
            digits.push(d);
            n /= 3;
        }

        digits.reverse();
        digits
    }

    /// Create a TritVec from **Representation A** (Balanced) digits.
    pub fn from_repr_a(balanced: &[i8]) -> Self {
        let mut value: i64 = 0;
        let mut power: i64 = 1;
        for &d in balanced.iter().rev() {
            assert!(d >= -1 && d <= 1, "Balanced trit must be -1, 0, or +1; got {}", d);
            value += d as i64 * power;
            power *= 3;
        }
        assert!(value >= 0, "Negative values not yet supported in TritVec");
        Self::from_decimal(value as u64)
    }

    /// Create a TritVec from **Representation C** (Bijective) digits.
    pub fn from_repr_c(bijective: &[u8]) -> Self {
        if bijective.is_empty() {
            return Self::zero();
        }
        let mut value: u64 = 0;
        let mut power: u64 = 1;
        for &d in bijective.iter().rev() {
            assert!(d >= 1 && d <= 3, "Bijective trit must be 1, 2, or 3; got {}", d);
            value += d as u64 * power;
            power *= 3;
        }
        Self::from_decimal(value)
    }

    /// Get the active representation's digit set label.
    pub fn repr_label(repr: TernaryRepr) -> &'static str {
        match repr {
            TernaryRepr::Balanced => "A {−1,0,+1}",
            TernaryRepr::Standard => "B {0,1,2}",
            TernaryRepr::Bijective => "C {1,2,3}",
        }
    }

    /// Format this value in a specific representation as a string.
    pub fn format_repr(&self, repr: TernaryRepr) -> String {
        match repr {
            TernaryRepr::Standard => {
                let digits = self.to_repr_b();
                let s: String = digits.iter().map(|d| char::from(b'0' + d)).collect();
                format!("{}₃", s)
            }
            TernaryRepr::Balanced => {
                let digits = self.to_repr_a();
                let s: String = digits
                    .iter()
                    .map(|&d| match d {
                        -1 => 'T',
                        0 => '0',
                        1 => '1',
                        _ => unreachable!(),
                    })
                    .collect();
                format!("{}₃", s)
            }
            TernaryRepr::Bijective => {
                let digits = self.to_repr_c();
                if digits.is_empty() {
                    return "ε".to_string();
                }
                let s: String = digits.iter().map(|d| char::from(b'0' + d)).collect();
                format!("{}₃ᵇ", s)
            }
        }
    }

    /// Check if this value is a perfect power of 3.
    pub fn is_power_of_3(&self) -> bool {
        let len = self.trit_length();
        if len == 0 {
            return false;
        }
        if self.trits[len - 1] != 1 {
            return false;
        }
        for i in 0..(len - 1) {
            if self.trits[i] != 0 {
                return false;
            }
        }
        true
    }

    /// If this is a power of 3, return the exponent k where self = 3^k.
    pub fn ternary_exponent(&self) -> Option<u32> {
        if self.is_power_of_3() {
            Some((self.trit_length() - 1) as u32)
        } else {
            None
        }
    }

    /// Count the number of distinct trit values (0, 1, 2) present.
    pub fn trit_diversity(&self) -> u8 {
        let mut seen = [false; 3];
        let len = self.trit_length();
        for i in 0..len {
            seen[self.trits[i] as usize] = true;
        }
        seen.iter().filter(|&&s| s).count() as u8
    }

    /// Add two trit vectors in native base-3 arithmetic.
    pub fn add_with_carry_tracking(a: &TritVec, b: &TritVec) -> TernaryAddResult {
        let max_len = std::cmp::max(a.trit_length(), b.trit_length());
        let mut result_trits = Vec::with_capacity(max_len + 1);
        let mut carry: u8 = 0;
        let mut carry_count: u32 = 0;
        let mut current_chain: u32 = 0;
        let mut max_carry_chain: u32 = 0;

        for i in 0..=max_len {
            let sum = a.trit_at(i) + b.trit_at(i) + carry;
            let digit = sum % 3;
            let new_carry = sum / 3;

            if new_carry > 0 {
                carry_count += 1;
                current_chain += 1;
                if current_chain > max_carry_chain {
                    max_carry_chain = current_chain;
                }
            } else {
                current_chain = 0;
            }

            carry = new_carry;
            result_trits.push(digit);
        }

        while carry > 0 {
            result_trits.push(carry % 3);
            carry /= 3;
        }

        while result_trits.len() > 1 && *result_trits.last().unwrap() == 0 {
            result_trits.pop();
        }

        TernaryAddResult {
            sum: TritVec { trits: result_trits },
            carry_count,
            max_carry_chain,
        }
    }

    /// Simple addition without carry tracking.
    pub fn add(a: &TritVec, b: &TritVec) -> TritVec {
        Self::add_with_carry_tracking(a, b).sum
    }

    /// Three-way addition: a + b + c, as needed by Tribonacci recurrence.
    pub fn add3_with_carry_tracking(
        a: &TritVec,
        b: &TritVec,
        c: &TritVec,
    ) -> TernaryAddResult {
        let first = Self::add_with_carry_tracking(a, b);
        let second = Self::add_with_carry_tracking(&first.sum, c);
        TernaryAddResult {
            sum: second.sum,
            carry_count: first.carry_count + second.carry_count,
            max_carry_chain: std::cmp::max(first.max_carry_chain, second.max_carry_chain),
        }
    }

    /// Return raw trits slice (LSB first, internal order).
    pub fn raw_trits(&self) -> &[u8] {
        &self.trits
    }
}

impl fmt::Display for TritVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msb = self.trits_msb_first();
        for trit in msb {
            write!(f, "{}", trit)?;
        }
        write!(f, "₃")
    }
}

/// Iterator over the Tribonacci sequence, computed natively in base-3.
pub struct TribonacciBase3 {
    window: [TritVec; 3],
    index: usize,
}

impl TribonacciBase3 {
    /// Create a new Tribonacci generator starting from the First Position.
    pub fn new() -> Self {
        TribonacciBase3 {
            window: [
                TritVec::zero(),
                TritVec::zero(),
                TritVec::from_trit(1),
            ],
            index: 0,
        }
    }

    /// Generate the first `n` terms with full metadata.
    pub fn generate(n: usize) -> Vec<TribonacciTerm> {
        let mut gen = Self::new();
        (0..n).map(|_| gen.next_term()).collect()
    }

    /// Produce the next term in the sequence with full ternary metadata.
    pub fn next_term(&mut self) -> TribonacciTerm {
        let idx = self.index;

        if idx < 3 {
            let value = self.window[idx].clone();
            let decimal = value.host_u64();
            let is_power = value.is_power_of_3();
            let exponent = value.ternary_exponent();
            let diversity = value.trit_diversity();

            self.index += 1;
            return TribonacciTerm {
                index: idx,
                value,
                decimal,
                is_ternary_power: is_power,
                ternary_exponent: exponent,
                carry_events: 0,
                trit_diversity: diversity,
            };
        }

        let a = &self.window[0];
        let b = &self.window[1];
        let c = &self.window[2];

        let result = TritVec::add3_with_carry_tracking(a, b, c);

        let decimal = result.sum.host_u64();
        let is_power = result.sum.is_power_of_3();
        let exponent = result.sum.ternary_exponent();
        let diversity = result.sum.trit_diversity();

        let term = TribonacciTerm {
            index: idx,
            value: result.sum.clone(),
            decimal,
            is_ternary_power: is_power,
            ternary_exponent: exponent,
            carry_events: result.carry_count,
            trit_diversity: diversity,
        };

        self.window[0] = self.window[1].clone();
        self.window[1] = self.window[2].clone();
        self.window[2] = result.sum;
        self.index += 1;

        term
    }
}

impl Default for TribonacciBase3 {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate the Tribonacci word — the fixed point of the morphism:
///   0 → 01
///   1 → 02
///   2 → 0
///
/// This is a 3-automatic sequence over the alphabet {0, 1, 2} and serves
/// as a canonical test oracle for ternary operations.
pub fn tribonacci_word(length: usize) -> Vec<u8> {
    let mut word: Vec<u8> = vec![0];

    while word.len() < length {
        let mut next = Vec::with_capacity(word.len() * 2);
        for &ch in &word {
            match ch {
                0 => {
                    next.push(0);
                    next.push(1);
                }
                1 => {
                    next.push(0);
                    next.push(2);
                }
                2 => {
                    next.push(0);
                }
                _ => unreachable!(),
            }
        }
        word = next;
    }

    word.truncate(length);
    word
}

/// Compute the first `n_digits` ternary digits of the Tribonacci constant
/// τ ≈ 1.839286755… in base 3.
pub fn tribonacci_constant_base3(n_digits: usize) -> Vec<u8> {
    let mut tau: f64 = TAU_TRIBONACCI;
    let mut digits = Vec::with_capacity(n_digits);

    let int_part = tau.floor() as u8;
    digits.push(int_part);
    tau -= int_part as f64;

    for _ in 1..n_digits {
        tau *= 3.0;
        let digit = tau.floor() as u8;
        digits.push(digit.min(2));
        tau -= digit as f64;
    }

    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tritvec_from_decimal() {
        assert_eq!(TritVec::from_decimal(0).host_u64(), 0);
        assert_eq!(TritVec::from_decimal(1).host_u64(), 1);
        assert_eq!(TritVec::from_decimal(4).host_u64(), 4);
        assert_eq!(TritVec::from_decimal(81).host_u64(), 81);
    }

    #[test]
    fn test_tritvec_display() {
        assert_eq!(format!("{}", TritVec::from_decimal(0)), "0₃");
        assert_eq!(format!("{}", TritVec::from_decimal(1)), "1₃");
        assert_eq!(format!("{}", TritVec::from_decimal(4)), "11₃");
        assert_eq!(format!("{}", TritVec::from_decimal(7)), "21₃");
        assert_eq!(format!("{}", TritVec::from_decimal(13)), "111₃");
        assert_eq!(format!("{}", TritVec::from_decimal(81)), "10000₃");
    }

    #[test]
    fn test_power_of_3_detection() {
        assert!(TritVec::from_decimal(1).is_power_of_3());
        assert!(TritVec::from_decimal(3).is_power_of_3());
        assert!(TritVec::from_decimal(9).is_power_of_3());
        assert!(TritVec::from_decimal(27).is_power_of_3());
        assert!(TritVec::from_decimal(81).is_power_of_3());
        assert!(!TritVec::from_decimal(0).is_power_of_3());
        assert!(!TritVec::from_decimal(2).is_power_of_3());
        assert!(!TritVec::from_decimal(4).is_power_of_3());
        assert!(!TritVec::from_decimal(82).is_power_of_3());
    }

    #[test]
    fn test_ternary_exponent() {
        assert_eq!(TritVec::from_decimal(1).ternary_exponent(), Some(0));
        assert_eq!(TritVec::from_decimal(3).ternary_exponent(), Some(1));
        assert_eq!(TritVec::from_decimal(81).ternary_exponent(), Some(4));
        assert_eq!(TritVec::from_decimal(7).ternary_exponent(), None);
    }

    #[test]
    fn test_addition() {
        let a = TritVec::from_decimal(4);
        let b = TritVec::from_decimal(7);
        let sum = TritVec::add(&a, &b);
        assert_eq!(sum.host_u64(), 11);
    }

    #[test]
    fn test_carry_tracking() {
        let a = TritVec::from_decimal(2);
        let b = TritVec::from_decimal(1);
        let result = TritVec::add_with_carry_tracking(&a, &b);
        assert_eq!(result.sum.host_u64(), 3);
        assert!(result.carry_count > 0);
    }

    #[test]
    fn test_tribonacci_first_21_terms() {
        let expected_decimal: Vec<u64> = vec![
            0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81,
            149, 274, 504, 927, 1705, 3136, 5768, 10609, 19513, 35890,
        ];

        let terms = TribonacciBase3::generate(21);

        for (i, term) in terms.iter().enumerate() {
            assert_eq!(term.decimal, expected_decimal[i],
                "T({}) = {} (expected {})", i, term.decimal, expected_decimal[i]);
            assert_eq!(term.value.host_u64(), expected_decimal[i]);
        }
    }

    #[test]
    fn test_tribonacci_t10_is_ternary_power() {
        let terms = TribonacciBase3::generate(11);
        let t10 = &terms[10];
        assert_eq!(t10.decimal, 81);
        assert!(t10.is_ternary_power);
        assert_eq!(t10.ternary_exponent, Some(4));
    }

    #[test]
    fn test_tribonacci_carries_begin_at_t5() {
        let terms = TribonacciBase3::generate(6);
        let t5 = &terms[5];
        assert_eq!(t5.decimal, 4);
        assert!(t5.carry_events > 0);
    }

    #[test]
    fn test_tribonacci_word() {
        let word = tribonacci_word(20);
        assert_eq!(word[0], 0);
        assert_eq!(word[1], 1);
        assert_eq!(word[2], 0);
        assert_eq!(word[3], 2);

        for &ch in &word {
            assert!(ch <= 2, "Tribonacci word should only contain 0, 1, 2");
        }
    }

    #[test]
    fn test_tribonacci_word_morphism_property() {
        let word = tribonacci_word(100);
        let counts = word.iter().fold([0usize; 3], |mut acc, &ch| {
            acc[ch as usize] += 1;
            acc
        });
        assert!(counts[0] > counts[1], "Trit 0 should dominate");
        assert!(counts[1] > counts[2], "Trit 1 should exceed trit 2");
    }

    #[test]
    fn test_tribonacci_constant_base3() {
        let digits = tribonacci_constant_base3(10);
        assert_eq!(digits[0], 1, "τ starts with 1");
        assert_eq!(digits.len(), 10);
    }

    #[test]
    fn test_repr_a_roundtrip() {
        for n in [0u64, 1, 4, 13, 81, 364] {
            let v = TritVec::from_decimal(n);
            let balanced = v.to_repr_a();
            let roundtrip = TritVec::from_repr_a(&balanced);
            assert_eq!(roundtrip.host_u64(), n,
                "Repr A roundtrip failed for {}", n);
        }
    }

    #[test]
    fn test_repr_c_roundtrip() {
        for n in [1u64, 2, 3, 4, 13, 81, 364] {
            let v = TritVec::from_decimal(n);
            let bijective = v.to_repr_c();
            let roundtrip = TritVec::from_repr_c(&bijective);
            assert_eq!(roundtrip.host_u64(), n,
                "Repr C roundtrip failed for {}", n);
        }
    }
}
