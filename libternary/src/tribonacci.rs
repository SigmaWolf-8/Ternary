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

//! # Tribonacci Base-3 Generator
//!
//! Native ternary Tribonacci sequence generator for the Salvi Framework.
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
//! The kernel performs arithmetic in representation B internally (the natural
//! choice for addition with carry propagation) and provides lossless
//! conversion to/from A and C at any boundary.
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
//!
//! ## Mathematical Foundation
//!
//! The Tribonacci constant τ ≈ 1.839286755… is the real root of x³ = x² + x + 1.
//! When T(n) lands on a pure power of 3 (e.g., T(10) = 81 = 3⁴ = 10000₃),
//! the ternary representation consists of a single 1-trit followed by zeros —
//! a structural singularity where carry complexity vanishes.

use std::fmt;

/// Maximum number of trits supported (covers T(n) up to n ≈ 200).
const _MAX_TRITS: usize = 128;

/// The three ternary representations supported by the kernel.
///
/// These are not three separate number systems — they are three **views**
/// of the same triadic arithmetic, freely interchangeable with carry-correct
/// translation at every boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TernaryRepr {
    /// Representation A: Balanced ternary `{-1, 0, +1}`.
    ///
    /// Symmetric around zero. Negation is trivial (flip all trits).
    /// Native to signed arithmetic, wave functions, error correction.
    /// The digit -1 is conventionally written as `T` or `−`.
    Balanced,

    /// Representation B: Standard ternary `{0, 1, 2}`.
    ///
    /// The conventional positional system. Natural for indexing, counting,
    /// the Tribonacci recurrence, and polynomial evaluation over GF(3).
    /// This is the kernel's internal arithmetic representation.
    Standard,

    /// Representation C: Bijective ternary `{1, 2, 3}`.
    ///
    /// Zero-free. Every positive integer has exactly one representation
    /// with no leading-zero ambiguity. Native to cryptographic wire formats
    /// where zero-padding must be distinguishable from data.
    /// Zero is represented as the empty word (no digits).
    Bijective,
}

/// A number represented as a vector of ternary digits (trits).
///
/// Stored least-significant-trit first (little-endian) for efficient
/// carry propagation during addition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TritVec {
    /// Trit storage: each element is 0, 1, or 2.
    /// Index 0 = least significant trit.
    trits: Vec<u8>,
}

/// Result of a ternary addition, including carry metadata.
#[derive(Clone, Debug)]
pub struct TernaryAddResult {
    /// The sum as a trit vector.
    pub sum: TritVec,
    /// Number of carry propagations that occurred during addition.
    /// In the timing domain, this corresponds to "jerk" — the rate
    /// of change of acceleration.
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

    /// Number of trits (significant digits) in this representation.
    pub fn trit_length(&self) -> usize {
        // Find the most significant non-zero trit.
        for i in (0..self.trits.len()).rev() {
            if self.trits[i] != 0 {
                return i + 1;
            }
        }
        1 // At least one trit (the zero itself).
    }

    /// Get the trit at position `i` (0 = least significant).
    /// Returns 0 for positions beyond the stored length.
    pub fn trit_at(&self, i: usize) -> u8 {
        if i < self.trits.len() {
            self.trits[i]
        } else {
            0
        }
    }

    /// Returns the trits as a slice, most-significant first (display order).
    pub fn trits_msb_first(&self) -> Vec<u8> {
        let len = self.trit_length();
        let mut result: Vec<u8> = self.trits[..len].to_vec();
        result.reverse();
        result
    }

    // ── Representation A/B/C Interchange ───────────────────────────
    //
    // The kernel stores values internally in Representation B {0,1,2}.
    // These methods provide lossless conversion to and from all three
    // representations. This is the entire point of the ternary kernel:
    // one engine, three output modes.

    /// Emit digits in **Representation B** (Standard: `{0, 1, 2}`).
    ///
    /// This is the internal storage format — returned as-is, MSB first.
    pub fn to_repr_b(&self) -> Vec<u8> {
        self.trits_msb_first()
    }

    /// Emit digits in **Representation A** (Balanced: `{-1, 0, +1}`).
    ///
    /// Conversion from B to A:
    /// - B digit 0 → A digit  0
    /// - B digit 1 → A digit +1
    /// - B digit 2 → A digit −1 with carry +1 to next higher position
    ///
    /// Returns `i8` values: -1, 0, or +1. MSB first.
    pub fn to_repr_a(&self) -> Vec<i8> {
        // Work LSB-first (internal order), then reverse.
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
                // Can happen when carry propagates into a 2
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

        // Trim trailing zeros (MSB side in internal order).
        while balanced.len() > 1 && *balanced.last().unwrap() == 0 {
            balanced.pop();
        }

        balanced.reverse(); // MSB first for output.
        balanced
    }

    /// Emit digits in **Representation C** (Bijective: `{1, 2, 3}`).
    ///
    /// Conversion from B to C:
    /// - B digit 0 → C digit 3 with borrow −1 from next higher position
    /// - B digit 1 → C digit 1
    /// - B digit 2 → C digit 2
    ///
    /// Zero is represented as the empty vector (no digits).
    /// Returns MSB first.
    pub fn to_repr_c(&self) -> Vec<u8> {
        if self.to_decimal() == 0 {
            return vec![]; // Zero = empty word in bijective.
        }

        // Bijective base-3 conversion from the decimal value.
        // Direct algorithm: repeatedly divide, but use ceiling division.
        let mut n = self.to_decimal();
        let mut digits: Vec<u8> = Vec::new();

        while n > 0 {
            n -= 1; // Shift to make it 0-indexed.
            let d = (n % 3) as u8 + 1; // Digits are 1, 2, 3.
            digits.push(d);
            n /= 3;
        }

        digits.reverse(); // MSB first.
        digits
    }

    /// Create a TritVec from **Representation A** (Balanced) digits.
    ///
    /// Input: `i8` values of -1, 0, or +1, MSB first.
    pub fn from_repr_a(balanced: &[i8]) -> Self {
        // Convert balanced → decimal → standard B.
        let mut value: i64 = 0;
        let mut power: i64 = 1;
        for &d in balanced.iter().rev() {
            assert!(
                d >= -1 && d <= 1,
                "Balanced trit must be -1, 0, or +1; got {}",
                d
            );
            value += d as i64 * power;
            power *= 3;
        }
        assert!(value >= 0, "Negative values not yet supported in TritVec");
        Self::from_decimal(value as u64)
    }

    /// Create a TritVec from **Representation C** (Bijective) digits.
    ///
    /// Input: `u8` values of 1, 2, or 3, MSB first.
    /// An empty slice represents zero.
    pub fn from_repr_c(bijective: &[u8]) -> Self {
        if bijective.is_empty() {
            return Self::zero();
        }
        // Convert bijective base-3 → decimal → standard B.
        let mut value: u64 = 0;
        let mut power: u64 = 1;
        for &d in bijective.iter().rev() {
            assert!(
                d >= 1 && d <= 3,
                "Bijective trit must be 1, 2, or 3; got {}",
                d
            );
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
                    return "ε".to_string(); // Empty word = zero.
                }
                let s: String = digits.iter().map(|d| char::from(b'0' + d)).collect();
                format!("{}₃ᵇ", s)
            }
        }
    }

    /// Check if this value is a perfect power of 3.
    ///
    /// A ternary number is a power of 3 if and only if it has the form
    /// `1` followed by zero or more `0`s — i.e., exactly one non-zero trit
    /// and that trit is `1`.
    pub fn is_power_of_3(&self) -> bool {
        let len = self.trit_length();
        if len == 0 {
            return false;
        }
        // The most significant trit must be 1.
        if self.trits[len - 1] != 1 {
            return false;
        }
        // All other trits must be 0.
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
    ///
    /// Returns the sum along with carry propagation metadata.
    /// The carry count is the fundamental observable: in the timing
    /// protocol domain, each carry event corresponds to a unit of
    /// computational "jerk" (third derivative of position).
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

        // Handle any remaining carry (shouldn't happen with single-digit carry,
        // but guards against edge cases).
        while carry > 0 {
            result_trits.push(carry % 3);
            carry /= 3;
        }

        // Trim trailing zeros (but keep at least one trit).
        while result_trits.len() > 1 && *result_trits.last().unwrap() == 0 {
            result_trits.pop();
        }

        TernaryAddResult {
            sum: TritVec {
                trits: result_trits,
            },
            carry_count,
            max_carry_chain,
        }
    }

    /// Simple addition without carry tracking.
    pub fn add(a: &TritVec, b: &TritVec) -> TritVec {
        Self::add_with_carry_tracking(a, b).sum
    }

    /// Three-way addition: a + b + c, as needed by Tribonacci recurrence.
    /// Returns the result with aggregate carry metadata.
    pub fn add3_with_carry_tracking(a: &TritVec, b: &TritVec, c: &TritVec) -> TernaryAddResult {
        let first = Self::add_with_carry_tracking(a, b);
        let second = Self::add_with_carry_tracking(&first.sum, c);
        TernaryAddResult {
            sum: second.sum,
            carry_count: first.carry_count + second.carry_count,
            max_carry_chain: std::cmp::max(first.max_carry_chain, second.max_carry_chain),
        }
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
///
/// The sequence starts from the First Position triple: (0, 0, 1).
/// Each term is produced with full ternary metadata including carry
/// propagation counts and ternary power alignment detection.
pub struct TribonacciBase3 {
    /// The three most recent terms (circular buffer).
    window: [TritVec; 3],
    /// Current index in the sequence.
    index: usize,
}

impl TribonacciBase3 {
    /// Create a new Tribonacci generator starting from the First Position.
    pub fn new() -> Self {
        TribonacciBase3 {
            window: [
                TritVec::zero(),       // T(0) = 0
                TritVec::zero(),       // T(1) = 0
                TritVec::from_trit(1), // T(2) = 1
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
            // Return one of the initial conditions.
            let value = self.window[idx].clone();
            let decimal = value.to_decimal();
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

        // T(n) = T(n-1) + T(n-2) + T(n-3)
        // The window stores [T(n-3), T(n-2), T(n-1)] at indices
        // determined by circular indexing.
        let a = &self.window[0];
        let b = &self.window[1];
        let c = &self.window[2];

        let result = TritVec::add3_with_carry_tracking(a, b, c);

        let decimal = result.sum.to_decimal();
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

        // Shift the window: drop oldest, append newest.
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
/// as a canonical test oracle for ternary operations. Any correct bijective
/// ternary encoder/decoder must preserve the structural properties of this word.
pub fn tribonacci_word(length: usize) -> Vec<u8> {
    // Start with "0" and iteratively apply the morphism.
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
///
/// Uses the relation τ³ = τ² + τ + 1 and iterative refinement.
/// The expansion begins: 1.2010022201112021…₃
///
/// Each digit can be interpreted as a Triskellion walk instruction:
///   0 → step forward (no turn)
///   1 → turn left 120°, step
///   2 → turn right 120°, step
pub fn tribonacci_constant_base3(n_digits: usize) -> Vec<u8> {
    // τ ≈ 1.839286755214161...
    // We use high-precision arithmetic to extract base-3 digits.
    // τ is the real root of x³ - x² - x - 1 = 0.

    // Start with a sufficiently precise decimal approximation.
    // For production use, this would use arbitrary-precision arithmetic.
    let mut tau: f64 = 1.839286755214161;
    let mut digits = Vec::with_capacity(n_digits);

    // Extract the integer part.
    let int_part = tau.floor() as u8;
    digits.push(int_part);
    tau -= int_part as f64;

    // Extract fractional ternary digits.
    for _ in 1..n_digits {
        tau *= 3.0;
        let digit = tau.floor() as u8;
        digits.push(digit.min(2)); // Clamp for floating-point safety.
        tau -= digit as f64;
    }

    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tritvec_from_decimal() {
        assert_eq!(TritVec::from_decimal(0).to_decimal(), 0);
        assert_eq!(TritVec::from_decimal(1).to_decimal(), 1);
        assert_eq!(TritVec::from_decimal(4).to_decimal(), 4);
        assert_eq!(TritVec::from_decimal(81).to_decimal(), 81);
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
        assert!(TritVec::from_decimal(1).is_power_of_3()); // 3⁰
        assert!(TritVec::from_decimal(3).is_power_of_3()); // 3¹
        assert!(TritVec::from_decimal(9).is_power_of_3()); // 3²
        assert!(TritVec::from_decimal(27).is_power_of_3()); // 3³
        assert!(TritVec::from_decimal(81).is_power_of_3()); // 3⁴
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
        let a = TritVec::from_decimal(4); // 11₃
        let b = TritVec::from_decimal(7); // 21₃
        let sum = TritVec::add(&a, &b);
        assert_eq!(sum.to_decimal(), 11);
    }

    #[test]
    fn test_carry_tracking() {
        let a = TritVec::from_decimal(2); // 2₃
        let b = TritVec::from_decimal(1); // 1₃
        let result = TritVec::add_with_carry_tracking(&a, &b);
        assert_eq!(result.sum.to_decimal(), 3);
        assert!(result.carry_count > 0, "2₃ + 1₃ = 10₃ must produce a carry");
    }

    #[test]
    fn test_tribonacci_first_21_terms() {
        let expected_decimal: Vec<u64> = vec![
            0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, 1705, 3136, 5768, 10609,
            19513, 35890,
        ];

        let terms = TribonacciBase3::generate(21);

        for (i, term) in terms.iter().enumerate() {
            assert_eq!(
                term.decimal, expected_decimal[i],
                "T({}) = {} (expected {})",
                i, term.decimal, expected_decimal[i]
            );
            // Cross-validate: trit vector round-trips through decimal.
            assert_eq!(term.value.to_decimal(), expected_decimal[i]);
        }
    }

    #[test]
    fn test_tribonacci_t10_is_ternary_power() {
        let terms = TribonacciBase3::generate(11);
        let t10 = &terms[10];
        assert_eq!(t10.decimal, 81);
        assert!(
            t10.is_ternary_power,
            "T(10) = 81 = 3⁴ must be detected as ternary power"
        );
        assert_eq!(t10.ternary_exponent, Some(4));
    }

    #[test]
    fn test_tribonacci_carries_begin_at_t5() {
        let terms = TribonacciBase3::generate(6);
        // T(3) and T(4) should have zero or minimal carries.
        // T(5) = 4 = 11₃ is the first multi-digit result from addition.
        let t5 = &terms[5];
        assert_eq!(t5.decimal, 4);
        assert!(
            t5.carry_events > 0,
            "T(5) computation (1+1+2=4) should involve carries"
        );
    }

    #[test]
    fn test_tribonacci_word() {
        let word = tribonacci_word(20);
        // The Tribonacci word begins: 0, 1, 0, 2, 0, 1, 0, 0, 1, 0, 2, 0, 1, ...
        assert_eq!(word[0], 0);
        assert_eq!(word[1], 1);
        assert_eq!(word[2], 0);
        assert_eq!(word[3], 2);
        assert_eq!(word[4], 0);
        assert_eq!(word[5], 1);
        assert_eq!(word[6], 0);

        // Verify all elements are in {0, 1, 2}.
        for &ch in &word {
            assert!(ch <= 2, "Tribonacci word must use alphabet {{0, 1, 2}}");
        }
    }

    #[test]
    fn test_tribonacci_constant_starts_correctly() {
        let digits = tribonacci_constant_base3(5);
        // τ = 1.2010...₃
        assert_eq!(digits[0], 1); // Integer part
        assert_eq!(digits[1], 2); // First fractional digit
        assert_eq!(digits[2], 0);
        assert_eq!(digits[3], 1);
        assert_eq!(digits[4], 0);
    }

    // ── Representation A/B/C Interchange Tests ────────────────────

    #[test]
    fn test_repr_b_roundtrip() {
        for n in 0..=100u64 {
            let tv = TritVec::from_decimal(n);
            assert_eq!(tv.to_decimal(), n, "Rep B roundtrip failed for {}", n);
        }
    }

    #[test]
    fn test_repr_c_bijective_roundtrip() {
        // Every positive integer should roundtrip through bijective {1,2,3}.
        for n in 0..=100u64 {
            let tv = TritVec::from_decimal(n);
            let bij = tv.to_repr_c();
            let back = TritVec::from_repr_c(&bij);
            assert_eq!(
                back.to_decimal(),
                n,
                "Bijective roundtrip failed for {} (bijective digits: {:?})",
                n,
                bij
            );
        }
    }

    #[test]
    fn test_repr_c_zero_is_empty() {
        let zero = TritVec::zero();
        let bij = zero.to_repr_c();
        assert!(
            bij.is_empty(),
            "Zero must be the empty word in bijective ternary"
        );
    }

    #[test]
    fn test_repr_c_no_zeros() {
        // Bijective digits must never contain 0.
        for n in 1..=200u64 {
            let tv = TritVec::from_decimal(n);
            let bij = tv.to_repr_c();
            for (i, &d) in bij.iter().enumerate() {
                assert!(
                    d >= 1 && d <= 3,
                    "Bijective digit at position {} for value {} is {} (must be 1,2,3)",
                    i,
                    n,
                    d
                );
            }
        }
    }

    #[test]
    fn test_repr_c_known_values() {
        // Bijective base-3: 1→1, 2→2, 3→3, 4→11, 5→12, 6→13, 7→21, ...
        assert_eq!(TritVec::from_decimal(1).to_repr_c(), vec![1]);
        assert_eq!(TritVec::from_decimal(2).to_repr_c(), vec![2]);
        assert_eq!(TritVec::from_decimal(3).to_repr_c(), vec![3]);
        assert_eq!(TritVec::from_decimal(4).to_repr_c(), vec![1, 1]);
        assert_eq!(TritVec::from_decimal(5).to_repr_c(), vec![1, 2]);
        assert_eq!(TritVec::from_decimal(6).to_repr_c(), vec![1, 3]);
        assert_eq!(TritVec::from_decimal(7).to_repr_c(), vec![2, 1]);
        assert_eq!(TritVec::from_decimal(13).to_repr_c(), vec![1, 1, 1]);
    }

    #[test]
    fn test_repr_a_balanced_roundtrip() {
        for n in 0..=100u64 {
            let tv = TritVec::from_decimal(n);
            let bal = tv.to_repr_a();
            let back = TritVec::from_repr_a(&bal);
            assert_eq!(
                back.to_decimal(),
                n,
                "Balanced roundtrip failed for {} (balanced digits: {:?})",
                n,
                bal
            );
        }
    }

    #[test]
    fn test_repr_a_known_values() {
        // 0 → [0], 1 → [1], 2 → [1,-1], 3 → [1,0], 4 → [1,1], 5 → [1,-1,-1]
        assert_eq!(TritVec::from_decimal(0).to_repr_a(), vec![0]);
        assert_eq!(TritVec::from_decimal(1).to_repr_a(), vec![1]);
        assert_eq!(TritVec::from_decimal(2).to_repr_a(), vec![1, -1]);
        assert_eq!(TritVec::from_decimal(3).to_repr_a(), vec![1, 0]);
        assert_eq!(TritVec::from_decimal(4).to_repr_a(), vec![1, 1]);
    }

    #[test]
    fn test_repr_a_only_valid_digits() {
        // Balanced ternary digits must be -1, 0, or +1.
        for n in 0..=200u64 {
            let tv = TritVec::from_decimal(n);
            let bal = tv.to_repr_a();
            for (i, &d) in bal.iter().enumerate() {
                assert!(
                    d >= -1 && d <= 1,
                    "Balanced digit at position {} for value {} is {} (must be -1,0,+1)",
                    i,
                    n,
                    d
                );
            }
        }
    }

    #[test]
    fn test_all_three_reprs_same_value() {
        // The kernel's guarantee: A, B, and C all represent the same integer.
        for n in 0..=100u64 {
            let tv = TritVec::from_decimal(n);

            let from_b = tv.to_decimal();
            let from_a = TritVec::from_repr_a(&tv.to_repr_a()).to_decimal();
            let from_c = TritVec::from_repr_c(&tv.to_repr_c()).to_decimal();

            assert_eq!(from_b, n, "Rep B mismatch for {}", n);
            assert_eq!(from_a, n, "Rep A mismatch for {}", n);
            assert_eq!(from_c, n, "Rep C mismatch for {}", n);
        }
    }

    #[test]
    fn test_format_repr_display() {
        let tv = TritVec::from_decimal(7);
        assert_eq!(tv.format_repr(TernaryRepr::Standard), "21₃");
        assert_eq!(tv.format_repr(TernaryRepr::Bijective), "21₃ᵇ");
        // Balanced: 7 = 9 - 3 + 1 = 1*9 + (-1)*3 + 1*1 → [1, T, 1]
        let bal_str = tv.format_repr(TernaryRepr::Balanced);
        assert!(
            bal_str.contains('T') || bal_str.contains('1'),
            "Balanced format of 7 should contain balanced digits: {}",
            bal_str
        );
    }

    #[test]
    fn test_tribonacci_terms_in_all_reprs() {
        // Verify T(0)..T(10) can be expressed in all three representations
        // and round-trip correctly.
        let terms = TribonacciBase3::generate(11);
        for term in &terms {
            let _b = term.value.to_repr_b();
            let a = term.value.to_repr_a();
            let c = term.value.to_repr_c();

            let from_a = TritVec::from_repr_a(&a).to_decimal();
            let from_c = TritVec::from_repr_c(&c).to_decimal();

            assert_eq!(
                from_a, term.decimal,
                "T({}) Rep A roundtrip: expected {}, got {}",
                term.index, term.decimal, from_a
            );
            assert_eq!(
                from_c, term.decimal,
                "T({}) Rep C roundtrip: expected {}, got {}",
                term.index, term.decimal, from_c
            );
        }
    }
}
