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

//! # Trit — Three-Vector Algebraic Value Type
//!
//! The foundational value type for the Salvi Framework. Wraps three
//! [`TritInt`] vectors representing an element of the form `a + bφ + cω`:
//!
//! - `v[0]` — coefficient of **1** (integer ground ring ℤ)
//! - `v[1]` — coefficient of **φ** (golden ratio, ℤ\[φ\] extension)
//! - `v[2]` — coefficient of **ω** (cube root of unity, ℤ\[ω\] extension)
//!
//! All three vectors are always live — no caps, no sentinels, no modes.
//! For a plain integer, `v[1]` and `v[2]` hold the additive identity.
//!
//! **Multiplication is algebra-specific:** `mul_golden` applies φ² = φ + 1,
//! `mul_scalar` operates on `v[0]` only. Eisenstein multiplication
//! (ω² = −1 − ω) is deferred to Phase 3 (requires signed arithmetic).
//!
//! **Position in the type chain:**
//! - `TritInt` — one ternary integer (Phase 1)
//! - `Trit` — three TritInts: v\[0\] = ℤ, v\[1\] = φ, v\[2\] = ω (this module)
//! - `[Trit; 3]` — one vertex coordinate
//! - Triangles, meshes, manifolds — built on Trit

use crate::trit_int::{TritInt, Overflow};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, AddAssign, SubAssign};

// ══════════════════════════════════════════════════════════════
// TYPE DEFINITION
// ══════════════════════════════════════════════════════════════

/// A three-vector algebraic value: `a + bφ + cω`.
///
/// `v[0]` = integer part (ℤ), `v[1]` = golden ratio coefficient (φ),
/// `v[2]` = cube root of unity coefficient (ω).
///
/// Three vectors because the system is base 3. Three algebraic components
/// because the framework operates in three algebraic domains.
pub struct Trit {
    pub v: [TritInt; 3],
}

// ══════════════════════════════════════════════════════════════
// CONSTRUCTORS (all const fn)
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// Zero: the additive identity in all three components.
    pub const fn zero() -> Self {
        Trit { v: [TritInt::zero(), TritInt::zero(), TritInt::zero()] }
    }

    /// One: the multiplicative identity (1 + 0φ + 0ω).
    pub const fn one() -> Self {
        Trit { v: [TritInt::one(), TritInt::zero(), TritInt::zero()] }
    }

    /// Scalar: a plain integer (a + 0φ + 0ω).
    pub const fn scalar(a: TritInt) -> Self {
        Trit { v: [a, TritInt::zero(), TritInt::zero()] }
    }

    /// Golden: a ℤ[φ] element (a + bφ + 0ω).
    pub const fn golden(a: TritInt, b: TritInt) -> Self {
        Trit { v: [a, b, TritInt::zero()] }
    }

    /// Eisenstein: a ℤ[ω] element (a + 0φ + cω).
    /// Note: mul_eisenstein is deferred to Phase 3 (requires signed arithmetic).
    /// This constructor is provided so the slot is addressable.
    pub const fn eisenstein(a: TritInt, c: TritInt) -> Self {
        Trit { v: [a, TritInt::zero(), c] }
    }

    /// Full: all three components live (a + bφ + cω).
    pub const fn new(a: TritInt, b: TritInt, c: TritInt) -> Self {
        Trit { v: [a, b, c] }
    }

    /// Construct a scalar Trit from a trit slice. Rep B {0,1,2}, LSB-first.
    /// Populates v[0] only. For multi-component values, use `golden`, `eisenstein`, or `new`.
    pub const fn from_trits(trits: &[u8]) -> Self {
        Self::scalar(TritInt::from_trits(trits))
    }

    /// Scalar repunit: R_n = 111...1₃ (n ones) + 0φ + 0ω.
    pub const fn repunit(n: usize) -> Self {
        Self::scalar(TritInt::repunit(n))
    }

    /// Scalar from u64 (BOUNDARY CROSSING: binary → ternary).
    pub const fn from_u64(val: u64) -> Self {
        Self::scalar(TritInt::from_u64(val))
    }

    // ── Const accessors (take self by value for const fn compatibility) ──

    /// Extract v[0] (integer part). Const fn — takes self by value.
    pub const fn v0(self) -> TritInt {
        let Trit { v: [a, _b, _c] } = self;
        a
    }

    /// Extract v[1] (φ-coefficient). Const fn — takes self by value.
    pub const fn v1(self) -> TritInt {
        let Trit { v: [_a, b, _c] } = self;
        b
    }

    /// Extract v[2] (ω-coefficient). Const fn — takes self by value.
    pub const fn v2(self) -> TritInt {
        let Trit { v: [_a, _b, c] } = self;
        c
    }
}

// ══════════════════════════════════════════════════════════════
// FRAMEWORK CONSTANTS
// ══════════════════════════════════════════════════════════════

/// Archimedean solid circumradius squared: R² = 14 + 5φ.
/// Integer part (14) IS π_Salvi. φ-coefficient (5) IS the pentagon generator.
/// Norm: N(14, 5) = 14² + 14×5 − 5² = 196 + 70 − 25 = 241 (prime, irreducible).
pub const R_SQUARED_T: Trit = Trit::golden(TritInt::from_u64(14), TritInt::from_u64(5));

/// Full circle: R₆ = 364 = 111111₃ (scalar).
pub const FULL_CIRCLE_T: Trit = Trit::repunit(6);

/// Pi: π = 14 = 112₃ (scalar).
pub const PI_T: Trit = Trit::scalar(TritInt::from_trits(&[2, 1, 1]));

/// Half-turn: 182 = π × R₃ (scalar).
pub const HALF_TURN_T: Trit = Trit::scalar(TritInt::from_u64(182));

/// φ² = 1 + φ = (1, 1) — the defining relation as a Trit constant.
pub const PHI_SQUARED_T: Trit = Trit::golden(TritInt::one(), TritInt::one());

// ══════════════════════════════════════════════════════════════
// COMPONENT-WISE ARITHMETIC (always valid, algebra-independent)
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// Component-wise addition. Always valid regardless of which algebra is active.
    pub fn add(&self, other: &Trit) -> Trit {
        Trit {
            v: [
                TritInt::add(&self.v[0], &other.v[0]),
                TritInt::add(&self.v[1], &other.v[1]),
                TritInt::add(&self.v[2], &other.v[2]),
            ]
        }
    }

    /// Component-wise subtraction. Panics if any component of other exceeds
    /// the corresponding component of self (unsigned underflow). Phase 3
    /// provides signed arithmetic for operations requiring negative intermediates.
    pub fn sub(&self, other: &Trit) -> Trit {
        Trit {
            v: [
                TritInt::sub(&self.v[0], &other.v[0]),
                TritInt::sub(&self.v[1], &other.v[1]),
                TritInt::sub(&self.v[2], &other.v[2]),
            ]
        }
    }

    /// Multiply all three vectors by a TritInt scalar.
    /// Does NOT require the Trit to be scalar — scales any element uniformly.
    pub fn scale(&self, scalar: &TritInt) -> Trit {
        Trit {
            v: [
                TritInt::mul(&self.v[0], scalar),
                TritInt::mul(&self.v[1], scalar),
                TritInt::mul(&self.v[2], scalar),
            ]
        }
    }
}

// ══════════════════════════════════════════════════════════════
// ALGEBRA-SPECIFIC MULTIPLICATION
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// ℤ[φ] multiplication. Applies the reduction rule φ² = φ + 1.
    ///
    /// (a₁ + b₁φ)(a₂ + b₂φ) = (a₁a₂ + b₁b₂) + (a₁b₂ + a₂b₁ + b₁b₂)φ
    ///
    /// Uses 4 TritInt multiplications and 3 TritInt additions.
    /// The reduction is addition-only, so unsigned TritInt handles it cleanly.
    ///
    /// Panics if either operand has a non-zero ω-component (v[2] ≠ 0).
    pub fn mul_golden(&self, other: &Trit) -> Trit {
        assert!(self.v[2].is_zero(), "mul_golden: self has non-zero ω-component");
        assert!(other.v[2].is_zero(), "mul_golden: other has non-zero ω-component");

        let a1a2 = TritInt::mul(&self.v[0], &other.v[0]);
        let b1b2 = TritInt::mul(&self.v[1], &other.v[1]);
        let a1b2 = TritInt::mul(&self.v[0], &other.v[1]);
        let a2b1 = TritInt::mul(&other.v[0], &self.v[1]);

        Trit {
            v: [
                // Integer part: a₁a₂ + b₁b₂ (from φ² = φ + 1 reduction)
                TritInt::add(&a1a2, &b1b2),
                // φ-coefficient: a₁b₂ + a₂b₁ + b₁b₂
                TritInt::add(&TritInt::add(&a1b2, &a2b1), &b1b2),
                // ω-component stays zero
                TritInt::zero(),
            ]
        }
    }

    /// Plain integer multiplication. Multiplies v[0] components only.
    ///
    /// Panics if either operand has non-zero φ or ω components.
    /// For multiplying any Trit by a TritInt without this restriction, use `scale`.
    pub fn mul_scalar(&self, other: &Trit) -> Trit {
        assert!(self.is_scalar(), "mul_scalar: self is not scalar");
        assert!(other.is_scalar(), "mul_scalar: other is not scalar");
        Trit::scalar(TritInt::mul(&self.v[0], &other.v[0]))
    }
}

// ══════════════════════════════════════════════════════════════
// NORMS
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// ℤ[φ] norm: N(a + bφ) = a² + ab − b².
    ///
    /// Returns a scalar Trit. The norm maps ℤ[φ] → ℤ.
    ///
    /// Panics if:
    /// - The ω-component is non-zero (v[2] ≠ 0)
    /// - The norm is negative (b² > a² + ab), which occurs for elements
    ///   with large φ-coefficient relative to integer part.
    ///   All framework constants produce positive norms.
    pub fn norm_golden(&self) -> Trit {
        assert!(self.v[2].is_zero(), "norm_golden: non-zero ω-component");

        let a = &self.v[0];
        let b = &self.v[1];
        let a_sq = TritInt::mul(a, a);
        let ab = TritInt::mul(a, b);
        let b_sq = TritInt::mul(b, b);

        let sum = TritInt::add(&a_sq, &ab); // a² + ab
        assert!(sum >= b_sq, "norm_golden: negative norm (b² > a² + ab)");

        Trit::scalar(TritInt::sub(&sum, &b_sq)) // a² + ab − b²
    }
}

// ══════════════════════════════════════════════════════════════
// ACCESSORS
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// True if all three components are zero.
    pub fn is_zero(&self) -> bool {
        self.v[0].is_zero() && self.v[1].is_zero() && self.v[2].is_zero()
    }

    /// True if v[1] and v[2] are both zero (pure integer).
    pub fn is_scalar(&self) -> bool {
        self.v[1].is_zero() && self.v[2].is_zero()
    }

    /// True if v[1] is non-zero and v[2] is zero (ℤ[φ] element with live φ part).
    pub fn is_golden(&self) -> bool {
        !self.v[1].is_zero() && self.v[2].is_zero()
    }

    /// True if v[2] is non-zero and v[1] is zero (ℤ[ω] element with live ω part).
    pub fn is_eisenstein(&self) -> bool {
        self.v[1].is_zero() && !self.v[2].is_zero()
    }

    /// Reference to v[0] (integer part).
    pub fn integer_part(&self) -> &TritInt { &self.v[0] }

    /// Reference to v[1] (φ-coefficient).
    pub fn golden_part(&self) -> &TritInt { &self.v[1] }

    /// Reference to v[2] (ω-coefficient).
    pub fn eisenstein_part(&self) -> &TritInt { &self.v[2] }

    /// Reference to any component by index.
    pub fn vector(&self, i: usize) -> &TritInt { &self.v[i] }
}

// ══════════════════════════════════════════════════════════════
// BOUNDARY CROSSINGS
// ══════════════════════════════════════════════════════════════

impl Trit {
    /// Evaluate as f64: a + b·φ + c·ω_real.
    ///
    /// BOUNDARY CROSSING — exits exact ternary arithmetic into floating point.
    /// φ = (1+√5)/2, ω_real = Re(ω) = −1/2. Both derived from their defining
    /// algebraic relations (φ²=φ+1 and ω²+ω+1=0), independent of the 364° circle.
    pub fn to_f64(&self) -> f64 {
        const PHI: f64 = 1.618_033_988_749_895;
        const OMEGA_REAL: f64 = -0.5;
        let a = self.v[0].to_decimal() as f64;
        let b = self.v[1].to_decimal() as f64;
        let c = self.v[2].to_decimal() as f64;
        a + b * PHI + c * OMEGA_REAL
    }

    /// Convert a scalar Trit to u64. Returns Err if the Trit is not scalar
    /// (has non-zero φ or ω components).
    pub fn to_u64(&self) -> Result<u64, Overflow> {
        if !self.is_scalar() {
            return Err(Overflow(0)); // 0 signals non-scalar, not a bit-width
        }
        self.v[0].to_u64()
    }
}

// ══════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ══════════════════════════════════════════════════════════════

// ── Clone ───────────────────────────────────────────────────

impl Clone for Trit {
    fn clone(&self) -> Self {
        Trit {
            v: [self.v[0].clone(), self.v[1].clone(), self.v[2].clone()]
        }
    }
}

// ── Display ─────────────────────────────────────────────────

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0₃");
        }

        let mut wrote_term = false;

        // Integer part
        if !self.v[0].is_zero() {
            write!(f, "{}", self.v[0])?;
            wrote_term = true;
        }

        // φ-coefficient
        if !self.v[1].is_zero() {
            if wrote_term { write!(f, " + ")?; }
            write!(f, "{}φ", self.v[1])?;
            wrote_term = true;
        }

        // ω-coefficient
        if !self.v[2].is_zero() {
            if wrote_term { write!(f, " + ")?; }
            write!(f, "{}ω", self.v[2])?;
        }

        Ok(())
    }
}

impl fmt::Debug for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trit({})", self)
    }
}

// ── PartialEq, Eq ───────────────────────────────────────────

impl PartialEq for Trit {
    fn eq(&self, other: &Self) -> bool {
        self.v[0] == other.v[0] && self.v[1] == other.v[1] && self.v[2] == other.v[2]
    }
}

impl Eq for Trit {}

// ── Hash ────────────────────────────────────────────────────

impl Hash for Trit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.v[0].hash(state);
        self.v[1].hash(state);
        self.v[2].hash(state);
    }
}

// ── Add / Sub operator traits ───────────────────────────────

impl Add for Trit {
    type Output = Trit;
    fn add(self, rhs: Trit) -> Trit { Trit::add(&self, &rhs) }
}

impl Add for &Trit {
    type Output = Trit;
    fn add(self, rhs: &Trit) -> Trit { Trit::add(self, rhs) }
}

impl Sub for Trit {
    type Output = Trit;
    fn sub(self, rhs: Trit) -> Trit { Trit::sub(&self, &rhs) }
}

impl Sub for &Trit {
    type Output = Trit;
    fn sub(self, rhs: &Trit) -> Trit { Trit::sub(self, rhs) }
}

impl AddAssign for Trit {
    fn add_assign(&mut self, rhs: Trit) { *self = Trit::add(self, &rhs); }
}

impl SubAssign for Trit {
    fn sub_assign(&mut self, rhs: Trit) { *self = Trit::sub(self, &rhs); }
}

// ══════════════════════════════════════════════════════════════
// COMPILE-TIME CONST ASSERTIONS
// ══════════════════════════════════════════════════════════════

const _: () = {
    // R² = 14 + 5φ — integer part is π
    assert!(R_SQUARED_T.v0().to_u32_const() == 14);
    assert!(R_SQUARED_T.v1().to_u32_const() == 5);
    assert!(R_SQUARED_T.v2().to_u32_const() == 0);

    // Full circle = 364 (scalar)
    assert!(FULL_CIRCLE_T.v0().to_u32_const() == 364);
    assert!(FULL_CIRCLE_T.v1().to_u32_const() == 0);

    // Pi = 14
    assert!(PI_T.v0().to_u32_const() == 14);

    // Half-turn = 182
    assert!(HALF_TURN_T.v0().to_u32_const() == 182);

    // φ² = 1 + φ
    assert!(PHI_SQUARED_T.v0().to_u32_const() == 1);
    assert!(PHI_SQUARED_T.v1().to_u32_const() == 1);
};

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────

    /// Create a golden Trit from decimal values for concise tests.
    fn g(a: u64, b: u64) -> Trit {
        Trit::golden(TritInt::from_u64(a), TritInt::from_u64(b))
    }

    /// Create a scalar Trit from a decimal value.
    fn s(val: u64) -> Trit {
        Trit::from_u64(val)
    }

    /// φ as a Trit value: 0 + 1·φ.
    fn phi() -> Trit {
        Trit::golden(TritInt::zero(), TritInt::one())
    }

    // ── Constructors ────────────────────────────────────────

    #[test]
    fn zero_is_zero() {
        let z = Trit::zero();
        assert!(z.is_zero());
        assert!(z.is_scalar());
        assert_eq!(z.v[0].to_decimal(), 0);
    }

    #[test]
    fn one_is_scalar_one() {
        let o = Trit::one();
        assert!(!o.is_zero());
        assert!(o.is_scalar());
        assert_eq!(o.v[0].to_decimal(), 1);
    }

    #[test]
    fn scalar_constructor() {
        let t = Trit::scalar(TritInt::from_u64(364));
        assert!(t.is_scalar());
        assert_eq!(t.v[0].to_decimal(), 364);
        assert!(t.v[1].is_zero());
        assert!(t.v[2].is_zero());
    }

    #[test]
    fn golden_constructor() {
        let t = g(14, 5);
        assert!(t.is_golden());
        assert!(!t.is_scalar());
        assert_eq!(t.v[0].to_decimal(), 14);
        assert_eq!(t.v[1].to_decimal(), 5);
        assert!(t.v[2].is_zero());
    }

    #[test]
    fn eisenstein_constructor() {
        let t = Trit::eisenstein(TritInt::from_u64(3), TritInt::from_u64(2));
        assert!(t.is_eisenstein());
        assert_eq!(t.v[0].to_decimal(), 3);
        assert!(t.v[1].is_zero());
        assert_eq!(t.v[2].to_decimal(), 2);
    }

    #[test]
    fn new_full_constructor() {
        let t = Trit::new(TritInt::from_u64(1), TritInt::from_u64(2), TritInt::from_u64(3));
        assert!(!t.is_scalar());
        assert!(!t.is_golden());
        assert!(!t.is_eisenstein());
        assert_eq!(t.v[0].to_decimal(), 1);
        assert_eq!(t.v[1].to_decimal(), 2);
        assert_eq!(t.v[2].to_decimal(), 3);
    }

    #[test]
    fn from_trits_makes_scalar() {
        let t = Trit::from_trits(&[2, 1, 1]); // 14
        assert!(t.is_scalar());
        assert_eq!(t.v[0].to_decimal(), 14);
    }

    #[test]
    fn repunit_makes_scalar() {
        let t = Trit::repunit(6);
        assert!(t.is_scalar());
        assert_eq!(t.v[0].to_decimal(), 364);
    }

    #[test]
    fn from_u64_makes_scalar() {
        let t = Trit::from_u64(182);
        assert!(t.is_scalar());
        assert_eq!(t.v[0].to_decimal(), 182);
    }

    // ── Component-wise add / sub ────────────────────────────

    #[test]
    fn add_scalars() {
        let result = Trit::add(&s(13), &s(1));
        assert_eq!(result.v[0].to_decimal(), 14);
        assert!(result.is_scalar());
    }

    #[test]
    fn add_golden_values() {
        let a = g(14, 5);
        let b = g(3, 2);
        let result = Trit::add(&a, &b);
        assert_eq!(result.v[0].to_decimal(), 17);
        assert_eq!(result.v[1].to_decimal(), 7);
        assert!(result.v[2].is_zero());
    }

    #[test]
    fn sub_scalars() {
        let result = Trit::sub(&s(182), &s(14));
        assert_eq!(result.v[0].to_decimal(), 168);
    }

    #[test]
    fn add_sub_identity() {
        let a = g(14, 5);
        let zero = Trit::zero();
        assert_eq!(Trit::add(&a, &zero), a);
        assert_eq!(Trit::sub(&a, &zero), a);
    }

    // ── Scale ───────────────────────────────────────────────

    #[test]
    fn scale_golden_by_integer() {
        let t = g(2, 3);
        let scalar = TritInt::from_u64(5);
        let result = t.scale(&scalar);
        assert_eq!(result.v[0].to_decimal(), 10);
        assert_eq!(result.v[1].to_decimal(), 15);
        assert!(result.v[2].is_zero());
    }

    #[test]
    fn scale_by_zero() {
        let t = g(14, 5);
        let result = t.scale(&TritInt::zero());
        assert!(result.is_zero());
    }

    // ── mul_golden: φ² = φ + 1 ──────────────────────────────

    #[test]
    fn phi_squared_is_phi_plus_one() {
        // φ × φ = (0 + 1φ) × (0 + 1φ) = (0·0 + 1·1) + (0·1 + 0·1 + 1·1)φ = 1 + φ
        let result = phi().mul_golden(&phi());
        assert_eq!(result.v[0].to_decimal(), 1);
        assert_eq!(result.v[1].to_decimal(), 1);
        assert!(result.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    #[test]
    fn phi_cubed_is_one_plus_two_phi() {
        // φ³ = φ · φ² = φ · (1 + φ) = φ + φ² = φ + 1 + φ = 1 + 2φ
        let phi_sq = phi().mul_golden(&phi());
        let result = phi().mul_golden(&phi_sq);
        assert_eq!(result.v[0].to_decimal(), 1);
        assert_eq!(result.v[1].to_decimal(), 2);
        assert!(result.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    #[test]
    fn fibonacci_powers_of_phi() {
        // φⁿ = F(n-1) + F(n)·φ  where F is the Fibonacci sequence
        let expected: [(u64, u64); 6] = [
            (1, 0),  // φ⁰ = 1
            (0, 1),  // φ¹ = φ
            (1, 1),  // φ² = 1 + φ
            (1, 2),  // φ³ = 1 + 2φ
            (2, 3),  // φ⁴ = 2 + 3φ
            (3, 5),  // φ⁵ = 3 + 5φ
        ];

        let mut power = Trit::one();
        for (n, (exp_a, exp_b)) in expected.iter().enumerate() {
            assert_eq!(power.v[0].to_decimal(), *exp_a, "φ^{}: integer part wrong", n);
            assert_eq!(power.v[1].to_decimal(), *exp_b, "φ^{}: φ-coefficient wrong", n);
            assert!(power.v[2].is_zero(), "φ^{}: v[2] must stay zero", n);
            power = power.mul_golden(&phi());
        }
    }

    #[test]
    fn mul_golden_by_one_is_identity() {
        let one = Trit::one();
        let r_sq = g(14, 5);
        let result = r_sq.mul_golden(&one);
        assert_eq!(result, r_sq);
        assert!(result.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    #[test]
    fn mul_golden_by_scalar() {
        // 3 × (14 + 5φ) = 42 + 15φ
        let three = s(3);
        let result = three.mul_golden(&g(14, 5));
        assert_eq!(result.v[0].to_decimal(), 42);
        assert_eq!(result.v[1].to_decimal(), 15);
        assert!(result.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    #[test]
    fn mul_golden_commutativity() {
        let a = g(14, 5);
        let b = g(3, 2);
        let ab = a.mul_golden(&b);
        let ba = b.mul_golden(&a);
        assert_eq!(ab, ba);
        assert!(ab.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    #[test]
    fn mul_golden_associativity() {
        let a = g(2, 1);
        let b = g(3, 1);
        let c = g(1, 2);
        let ab_c = a.mul_golden(&b).mul_golden(&c);
        let a_bc = a.mul_golden(&b.mul_golden(&c));
        assert_eq!(ab_c, a_bc);
        assert!(ab_c.v[2].is_zero(), "v[2] must stay zero through mul_golden");
    }

    // ── mul_scalar ──────────────────────────────────────────

    #[test]
    fn mul_scalar_basic() {
        let result = s(14).mul_scalar(&s(13));
        assert_eq!(result.v[0].to_decimal(), 182);
        assert!(result.is_scalar());
    }

    // ── norm_golden ─────────────────────────────────────────

    #[test]
    fn norm_golden_r_squared() {
        // N(14 + 5φ) = 14² + 14×5 − 5² = 196 + 70 − 25 = 241
        let norm = g(14, 5).norm_golden();
        assert!(norm.is_scalar());
        assert_eq!(norm.v[0].to_decimal(), 241);
    }

    #[test]
    fn norm_golden_of_phi() {
        // N(0 + 1φ) = 0 + 0 − 1 = −1 → should panic (negative norm)
        // Tested separately with should_panic
    }

    #[test]
    fn norm_golden_of_one() {
        // N(1 + 0φ) = 1 + 0 − 0 = 1
        let norm = Trit::one().norm_golden();
        assert_eq!(norm.v[0].to_decimal(), 1);
    }

    #[test]
    fn norm_golden_of_integer() {
        // N(7 + 0φ) = 49 + 0 − 0 = 49
        let norm = s(7).norm_golden();
        assert_eq!(norm.v[0].to_decimal(), 49);
    }

    #[test]
    fn norm_golden_241_is_prime() {
        // N(R²) = 241 — verify primality via trial division
        let n: u64 = 241;
        let is_prime = (2..16).all(|d| n % d != 0);
        assert!(is_prime, "241 should be prime");
    }

    // ── Accessors ───────────────────────────────────────────

    #[test]
    fn accessor_integer_part() {
        let t = g(14, 5);
        assert_eq!(t.integer_part().to_decimal(), 14);
    }

    #[test]
    fn accessor_golden_part() {
        let t = g(14, 5);
        assert_eq!(t.golden_part().to_decimal(), 5);
    }

    #[test]
    fn accessor_eisenstein_part() {
        let t = Trit::eisenstein(TritInt::from_u64(3), TritInt::from_u64(7));
        assert_eq!(t.eisenstein_part().to_decimal(), 7);
    }

    #[test]
    fn accessor_vector_indexing() {
        let t = Trit::new(TritInt::from_u64(1), TritInt::from_u64(2), TritInt::from_u64(3));
        assert_eq!(t.vector(0).to_decimal(), 1);
        assert_eq!(t.vector(1).to_decimal(), 2);
        assert_eq!(t.vector(2).to_decimal(), 3);
    }

    // ── Boundary crossings ──────────────────────────────────

    #[test]
    fn to_f64_golden() {
        // 1 + 1φ = 1 + (1+√5)/2 = (3+√5)/2 ≈ 2.618
        let t = g(1, 1);
        let val = t.to_f64();
        assert!((val - 2.618_033_988_749_895).abs() < 1e-10);
    }

    #[test]
    fn to_f64_r_squared() {
        // R² = 14 + 5φ ≈ 22.090
        let val = R_SQUARED_T.to_f64();
        assert!((val - 22.090_169_943_749_47).abs() < 1e-10);
    }

    #[test]
    fn to_u64_scalar() {
        assert_eq!(s(364).to_u64().unwrap(), 364);
    }

    #[test]
    fn to_u64_non_scalar_errors() {
        assert!(g(14, 5).to_u64().is_err());
    }

    // ── Display ─────────────────────────────────────────────

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", Trit::zero()), "0₃");
    }

    #[test]
    fn display_scalar() {
        assert_eq!(format!("{}", s(14)), "112₃");
    }

    #[test]
    fn display_golden() {
        let t = g(14, 5);
        assert_eq!(format!("{}", t), "112₃ + 12₃φ");
    }

    #[test]
    fn display_pure_phi() {
        let t = phi();
        assert_eq!(format!("{}", t), "1₃φ");
    }

    #[test]
    fn display_eisenstein() {
        let t = Trit::eisenstein(TritInt::from_u64(7), TritInt::from_u64(2));
        assert_eq!(format!("{}", t), "21₃ + 2₃ω");
    }

    // ── Operator traits ─────────────────────────────────────

    #[test]
    fn operator_add() {
        let c = s(13) + s(1);
        assert_eq!(c.v[0].to_decimal(), 14);
    }

    #[test]
    fn operator_sub() {
        let c = s(182) - s(40);
        assert_eq!(c.v[0].to_decimal(), 142);
    }

    #[test]
    fn operator_add_ref() {
        let a = s(13);
        let b = s(1);
        let c = &a + &b;
        assert_eq!(c.v[0].to_decimal(), 14);
    }

    #[test]
    fn operator_add_assign() {
        let mut a = s(13);
        a += s(1);
        assert_eq!(a.v[0].to_decimal(), 14);
    }

    // ── Equality and hashing ────────────────────────────────

    #[test]
    fn equality_same_values() {
        assert_eq!(g(14, 5), g(14, 5));
        assert_ne!(g(14, 5), g(14, 6));
        assert_ne!(g(14, 5), s(14));
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(g(14, 5));
        assert!(set.contains(&g(14, 5)));
        assert!(!set.contains(&g(14, 6)));
    }

    // ── Should-panic tests ──────────────────────────────────

    #[test]
    #[should_panic(expected = "negative norm")]
    fn norm_golden_negative_panics() {
        // N(0 + 1φ) = 0 + 0 − 1 = −1 → negative, panic
        phi().norm_golden();
    }

    #[test]
    #[should_panic(expected = "not scalar")]
    fn mul_scalar_non_scalar_panics() {
        g(14, 5).mul_scalar(&s(3));
    }

    #[test]
    #[should_panic(expected = "non-zero ω-component")]
    fn mul_golden_with_omega_panics() {
        let with_omega = Trit::new(TritInt::one(), TritInt::one(), TritInt::one());
        with_omega.mul_golden(&Trit::one());
    }

    #[test]
    #[should_panic(expected = "non-zero ω-component")]
    fn norm_golden_with_omega_panics() {
        let with_omega = Trit::eisenstein(TritInt::from_u64(3), TritInt::from_u64(2));
        with_omega.norm_golden();
    }
}
