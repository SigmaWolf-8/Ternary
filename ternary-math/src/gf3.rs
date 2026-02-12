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

//! # GF(3) — Galois Field of Order 3
//!
//! The atomic arithmetic type for all ternary computation in PlenumNET.
//! Elements are {0, 1, 2} under modular arithmetic.
//!
//! This module also provides balanced ternary representation {-1, 0, +1}
//! via the `BalancedTrit` type, with verified isomorphism to `Gf3`.

use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// An element of GF(3) = {0, 1, 2}.
///
/// Stored as a u8, invariant: value < 3.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gf3(u8);

impl Gf3 {
    pub const ZERO: Gf3 = Gf3(0);
    pub const ONE: Gf3 = Gf3(1);
    pub const TWO: Gf3 = Gf3(2);

    /// All elements of the field, for exhaustive iteration.
    pub const ALL: [Gf3; 3] = [Gf3::ZERO, Gf3::ONE, Gf3::TWO];

    /// Create from raw value. Panics if value >= 3.
    #[inline]
    pub fn new(value: u8) -> Self {
        assert!(value < 3, "GF(3) element must be 0, 1, or 2, got {value}");
        Gf3(value)
    }

    /// Create from raw value, reducing mod 3.
    #[inline]
    pub fn from_mod(value: u8) -> Self {
        Gf3(value % 3)
    }

    /// The raw underlying value.
    #[inline]
    pub fn value(self) -> u8 {
        self.0
    }

    /// Multiplicative inverse. Panics on zero (which has no inverse).
    /// In GF(3): 1⁻¹ = 1, 2⁻¹ = 2 (since 2×2 = 4 ≡ 1 mod 3).
    #[inline]
    pub fn inv(self) -> Self {
        match self.0 {
            0 => panic!("Zero has no multiplicative inverse in GF(3)"),
            1 => Gf3::ONE,
            2 => Gf3::TWO,
            _ => unreachable!(),
        }
    }

    /// Additive inverse (negation). 0→0, 1→2, 2→1.
    #[inline]
    pub fn neg(self) -> Self {
        Gf3((3 - self.0) % 3)
    }

    /// Convert to balanced ternary representation.
    #[inline]
    pub fn to_balanced(self) -> BalancedTrit {
        match self.0 {
            0 => BalancedTrit::Zero,
            1 => BalancedTrit::Pos,
            2 => BalancedTrit::Neg,
            _ => unreachable!(),
        }
    }

    /// Check if this is the additive identity.
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for Gf3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Gf3((self.0 + rhs.0) % 3)
    }
}

impl Sub for Gf3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        // (self + neg(rhs)) mod 3
        Gf3((self.0 + 3 - rhs.0) % 3)
    }
}

impl Mul for Gf3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Gf3((self.0 * rhs.0) % 3)
    }
}

impl Neg for Gf3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Gf3::neg(self)
    }
}

impl fmt::Debug for Gf3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gf3({})", self.0)
    }
}

impl fmt::Display for Gf3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -- Balanced Ternary ---------------------------------------------------------

/// Balanced ternary representation: {-1, 0, +1}.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BalancedTrit {
    Neg = 2, // stored as its GF(3) equivalent
    Zero = 0,
    Pos = 1,
}

impl BalancedTrit {
    /// Convert to GF(3) standard form.
    #[inline]
    pub fn to_gf3(self) -> Gf3 {
        Gf3(self as u8)
    }

    /// Signed integer value.
    #[inline]
    pub fn to_i8(self) -> i8 {
        match self {
            BalancedTrit::Neg => -1,
            BalancedTrit::Zero => 0,
            BalancedTrit::Pos => 1,
        }
    }

    /// Negate: +1 ↔ -1, 0 stays 0. No carry propagation.
    #[inline]
    pub fn negate(self) -> Self {
        match self {
            BalancedTrit::Neg => BalancedTrit::Pos,
            BalancedTrit::Zero => BalancedTrit::Zero,
            BalancedTrit::Pos => BalancedTrit::Neg,
        }
    }
}

impl fmt::Display for BalancedTrit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BalancedTrit::Neg => write!(f, "T"), // T for negative, Knuth convention
            BalancedTrit::Zero => write!(f, "0"),
            BalancedTrit::Pos => write!(f, "1"),
        }
    }
}

// -- Ternary VM Opcodes (GF(3) operations) ------------------------------------

/// Ternary NOT: 0→0, 1→2, 2→1. Equivalent to additive inverse.
#[inline]
pub fn tnot(a: Gf3) -> Gf3 {
    -a
}

/// Ternary MIN under natural ordering 0 < 1 < 2.
#[inline]
pub fn tmin(a: Gf3, b: Gf3) -> Gf3 {
    Gf3(a.value().min(b.value()))
}

/// Ternary MAX under natural ordering 0 < 1 < 2.
#[inline]
pub fn tmax(a: Gf3, b: Gf3) -> Gf3 {
    Gf3(a.value().max(b.value()))
}

/// Consensus: if a == b then a, else 0.
#[inline]
pub fn consensus(a: Gf3, b: Gf3) -> Gf3 {
    if a == b { a } else { Gf3::ZERO }
}

/// Any: if a != 0 then a, else b.
#[inline]
pub fn any(a: Gf3, b: Gf3) -> Gf3 {
    if !a.is_zero() { a } else { b }
}

/// Ternary multiply-accumulate: a * b + c (single fused operation).
#[inline]
pub fn tmac(a: Gf3, b: Gf3, c: Gf3) -> Gf3 {
    a * b + c
}

// -- GF(3) Vector (trit word) -------------------------------------------------

/// A fixed-width vector over GF(3). Used for VM registers, addresses, etc.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Gf3Vec {
    trits: Vec<Gf3>,
}

impl Gf3Vec {
    pub fn new(trits: Vec<Gf3>) -> Self {
        Gf3Vec { trits }
    }

    pub fn zeros(n: usize) -> Self {
        Gf3Vec {
            trits: vec![Gf3::ZERO; n],
        }
    }

    pub fn len(&self) -> usize {
        self.trits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trits.is_empty()
    }

    pub fn get(&self, i: usize) -> Gf3 {
        self.trits[i]
    }

    /// Component-wise addition in GF(3)^n.
    pub fn add(&self, other: &Gf3Vec) -> Gf3Vec {
        assert_eq!(self.len(), other.len(), "Vector dimension mismatch");
        Gf3Vec {
            trits: self
                .trits
                .iter()
                .zip(other.trits.iter())
                .map(|(&a, &b)| a + b)
                .collect(),
        }
    }

    /// Component-wise subtraction in GF(3)^n.
    pub fn sub(&self, other: &Gf3Vec) -> Gf3Vec {
        assert_eq!(self.len(), other.len(), "Vector dimension mismatch");
        Gf3Vec {
            trits: self
                .trits
                .iter()
                .zip(other.trits.iter())
                .map(|(&a, &b)| a - b)
                .collect(),
        }
    }

    /// Interpret as unsigned integer (standard ternary).
    pub fn to_u64(&self) -> u64 {
        let mut result = 0u64;
        let mut place = 1u64;
        for &trit in &self.trits {
            result += trit.value() as u64 * place;
            place *= 3;
        }
        result
    }

    /// Interpret as signed integer (balanced ternary).
    pub fn to_i64_balanced(&self) -> i64 {
        let mut result = 0i64;
        let mut place = 1i64;
        for &trit in &self.trits {
            result += trit.to_balanced().to_i8() as i64 * place;
            place *= 3;
        }
        result
    }
}

impl fmt::Debug for Gf3Vec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, t) in self.trits.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{t}")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // EXHAUSTIVE FIELD AXIOM VERIFICATION
    // These are not samples — they check every possible input combination.
    // =========================================================================

    #[test]
    fn axiom_addition_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                let c = a + b;
                assert!(c.value() < 3, "Addition closure violated: {a} + {b} = {c}");
            }
        }
    }

    #[test]
    fn axiom_addition_associativity() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    assert_eq!(
                        (a + b) + c,
                        a + (b + c),
                        "Associativity violated: ({a}+{b})+{c} ≠ {a}+({b}+{c})"
                    );
                }
            }
        }
    }

    #[test]
    fn axiom_addition_identity() {
        for &a in &Gf3::ALL {
            assert_eq!(a + Gf3::ZERO, a, "Additive identity violated for {a}");
            assert_eq!(Gf3::ZERO + a, a, "Additive identity violated for {a}");
        }
    }

    #[test]
    fn axiom_addition_inverses() {
        for &a in &Gf3::ALL {
            let neg_a = -a;
            assert_eq!(
                a + neg_a,
                Gf3::ZERO,
                "Additive inverse violated: {a} + {neg_a} ≠ 0"
            );
        }
    }

    #[test]
    fn axiom_addition_commutativity() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert_eq!(
                    a + b,
                    b + a,
                    "Commutativity violated: {a}+{b} ≠ {b}+{a}"
                );
            }
        }
    }

    #[test]
    fn axiom_multiplication_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                let c = a * b;
                assert!(
                    c.value() < 3,
                    "Multiplication closure violated: {a} × {b} = {c}"
                );
            }
        }
    }

    #[test]
    fn axiom_multiplication_associativity() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    assert_eq!(
                        (a * b) * c,
                        a * (b * c),
                        "Mul assoc violated: ({a}×{b})×{c} ≠ {a}×({b}×{c})"
                    );
                }
            }
        }
    }

    #[test]
    fn axiom_multiplication_identity() {
        for &a in &Gf3::ALL {
            assert_eq!(a * Gf3::ONE, a, "Multiplicative identity violated for {a}");
            assert_eq!(Gf3::ONE * a, a, "Multiplicative identity violated for {a}");
        }
    }

    #[test]
    fn axiom_multiplication_inverses() {
        for &a in &Gf3::ALL {
            if !a.is_zero() {
                let inv = a.inv();
                assert_eq!(
                    a * inv,
                    Gf3::ONE,
                    "Multiplicative inverse violated: {a} × {inv} ≠ 1"
                );
            }
        }
    }

    #[test]
    fn axiom_multiplication_commutativity() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert_eq!(
                    a * b,
                    b * a,
                    "Mul commutativity violated: {a}×{b} ≠ {b}×{a}"
                );
            }
        }
    }

    #[test]
    fn axiom_distributivity() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    assert_eq!(
                        a * (b + c),
                        a * b + a * c,
                        "Distributivity violated: {a}×({b}+{c}) ≠ {a}×{b}+{a}×{c}"
                    );
                }
            }
        }
    }

    // =========================================================================
    // OPCODE CLOSURE VERIFICATION
    // Every opcode must map GF(3) inputs to GF(3) outputs.
    // =========================================================================

    #[test]
    fn opcode_tnot_closure() {
        for &a in &Gf3::ALL {
            let r = tnot(a);
            assert!(r.value() < 3);
        }
    }

    #[test]
    fn opcode_tnot_involution() {
        for &a in &Gf3::ALL {
            assert_eq!(tnot(tnot(a)), a, "TNOT is not an involution for {a}");
        }
    }

    #[test]
    fn opcode_tmin_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert!(tmin(a, b).value() < 3);
            }
        }
    }

    #[test]
    fn opcode_tmax_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert!(tmax(a, b).value() < 3);
            }
        }
    }

    #[test]
    fn opcode_consensus_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert!(consensus(a, b).value() < 3);
            }
        }
    }

    #[test]
    fn opcode_any_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                assert!(any(a, b).value() < 3);
            }
        }
    }

    #[test]
    fn opcode_tmac_closure() {
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    assert!(tmac(a, b, c).value() < 3);
                }
            }
        }
    }

    // =========================================================================
    // BALANCED TERNARY ISOMORPHISM
    // =========================================================================

    #[test]
    fn balanced_roundtrip() {
        for &a in &Gf3::ALL {
            assert_eq!(a.to_balanced().to_gf3(), a, "Balanced roundtrip failed for {a}");
        }
    }

    #[test]
    fn balanced_negation_matches() {
        for &a in &Gf3::ALL {
            let neg_via_gf3 = (-a).to_balanced();
            let neg_via_balanced = a.to_balanced().negate();
            assert_eq!(neg_via_gf3, neg_via_balanced);
        }
    }
}
