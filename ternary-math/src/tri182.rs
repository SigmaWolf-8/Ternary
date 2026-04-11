// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// tri182.rs — 182° Triangle Geometry via ℤ[φ] on the Trit Type
//
// Everything that follows from triangles summing to 182°.
// The ℤ[φ] ring, R² = 14 + 5φ, Gauss-Bonnet closure at 364 faces,
// coprime walks on triangulated surfaces, vertex coordinates in ℤ[φ].
// The disdyakis triacontahedron is a test case that validates it.
//
// This module is a CONSUMER of the Trit type. It assigns meaning
// to v[0] and v[1]:
//
//   v[0] = a  (integer coefficient of 1)
//   v[1] = b  (integer coefficient of φ, the golden ratio)
//   v[2] = unused (additive identity)
//
// Sources:
//   TM-2026-031 v5.0 §1.4 (ℤ[φ] ring definition)
//   TM-2026-031 v5.0 §4.4 (ℤ[φ] as structural similarity)
//   Disdyakis triacontahedron analysis (R² = 14 + 5φ derivation)

use crate::trit::Trit;
use crate::trit_int::TritInt;

// ══════════════════════════════════════════════════════════════
// §1  CONSTRUCTORS
// ══════════════════════════════════════════════════════════════

/// Create a ℤ[φ] element from two TritInt values: a + bφ.
pub fn zphi(a: TritInt, b: TritInt) -> Trit {
    Trit::golden(a, b)
}

/// Create a ℤ[φ] element from a plain integer (b = 0).
pub fn tri182_int(a: TritInt) -> Trit {
    Trit::scalar(a)
}

/// Zero in ℤ[φ].
pub fn tri182_zero() -> Trit {
    Trit::zero()
}

/// One in ℤ[φ].
pub fn tri182_one() -> Trit {
    Trit::one()
}

/// φ itself: 0 + 1·φ.
pub fn tri182_phi() -> Trit {
    Trit::golden(TritInt::zero(), TritInt::one())
}

// ══════════════════════════════════════════════════════════════
// §2  FRAMEWORK CONSTANTS IN ℤ[φ]
// ══════════════════════════════════════════════════════════════

/// R² = 14 + 5φ. Norm = 241 (prime, irreducible in ℤ[φ]).
pub const R_SQUARED: Trit = Trit::golden(
    TritInt::from_trits(&[2, 1, 1]),  // 14
    TritInt::from_trits(&[2, 1]),     //  5
);

/// Icosahedron edge² (unit edge): 2.
pub const ICOSA_EDGE_SQ: Trit = Trit::scalar(TritInt::from_trits(&[2]));

/// Icosahedron circumradius² (unit edge): 2 + φ.
pub const ICOSA_CIRCUMRADIUS_SQ: Trit = Trit::golden(
    TritInt::from_trits(&[2]),
    TritInt::from_trits(&[1]),
);

/// φ² = 1 + φ.
pub const PHI_SQUARED: Trit = Trit::golden(
    TritInt::from_trits(&[1]),
    TritInt::from_trits(&[1]),
);

// ══════════════════════════════════════════════════════════════
// §3  ℤ[φ] ARITHMETIC (delegates to Trit methods)
// ══════════════════════════════════════════════════════════════

/// Add two ℤ[φ] elements.
pub fn tri182_add(x: &Trit, y: &Trit) -> Trit { Trit::add(x, y) }

/// Subtract two ℤ[φ] elements.
pub fn tri182_sub(x: &Trit, y: &Trit) -> Trit { Trit::sub(x, y) }

/// Multiply two ℤ[φ] elements. Applies φ² = φ + 1 reduction.
pub fn tri182_mul(x: &Trit, y: &Trit) -> Trit { x.mul_golden(y) }

/// Norm: N(a + bφ) = a² + ab − b².
pub fn tri182_norm(x: &Trit) -> Trit { x.norm_golden() }

/// Evaluate as f64 (BOUNDARY CROSSING).
pub fn tri182_to_f64(x: &Trit) -> f64 { x.to_f64() }

// ══════════════════════════════════════════════════════════════
// §4  COMPARISON
// ══════════════════════════════════════════════════════════════

/// True if non-zero. Assumes a ≥ 0 and b ≥ 0 (all framework values).
pub fn tri182_is_positive(x: &Trit) -> bool {
    !x.v[0].is_zero() || !x.v[1].is_zero()
}

// ══════════════════════════════════════════════════════════════
// §5  NORM DISTANCE
// ══════════════════════════════════════════════════════════════

/// |N(x − y)|. Structural similarity metric. 16 bytes vs 6,144.
pub fn tri182_norm_distance(x: &Trit, y: &Trit) -> Trit {
    tri182_norm(&tri182_sub(x, y))
}

// ══════════════════════════════════════════════════════════════
// §6  FIBONACCI-WEIGHTED PROJECTION
// ══════════════════════════════════════════════════════════════

/// Project 27-trit classification address into ℤ[φ] coordinates.
///
/// a = Σ trit[k] × fib(k),  b = Σ trit[k] × fib(k+1)
///
/// Natural because φⁿ = F(n−1) + F(n)·φ.
pub fn project_to_zphi(classification_trits: &[u8]) -> Trit {
    assert!(classification_trits.len() <= 27);

    let mut fib = Vec::with_capacity(28);
    fib.push(TritInt::one());
    fib.push(TritInt::one());
    for i in 2..28 {
        fib.push(TritInt::add(&fib[i - 1], &fib[i - 2]));
    }

    let mut a = TritInt::zero();
    let mut b = TritInt::zero();

    for (k, &trit) in classification_trits.iter().enumerate() {
        let trit_val = TritInt::from_trits(&[trit]);
        a = TritInt::add(&a, &TritInt::mul(&fib[k], &trit_val));
        b = TritInt::add(&b, &TritInt::mul(&fib[k + 1], &trit_val));
    }

    Trit::golden(a, b)
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_squared_is_phi_plus_one() {
        let phi_sq = tri182_mul(&tri182_phi(), &tri182_phi());
        assert_eq!(phi_sq.v[0].to_decimal(), 1);
        assert_eq!(phi_sq.v[1].to_decimal(), 1);
        assert_eq!(phi_sq, PHI_SQUARED);
        assert!(phi_sq.v[2].is_zero());
    }

    #[test]
    fn phi_cubed() {
        let phi = tri182_phi();
        let result = tri182_mul(&phi, &tri182_mul(&phi, &phi));
        assert_eq!(result.v[0].to_decimal(), 1);
        assert_eq!(result.v[1].to_decimal(), 2);
        assert!(result.v[2].is_zero());
    }

    #[test]
    fn fibonacci_powers_of_phi() {
        let phi = tri182_phi();
        let mut power = tri182_one();
        let expected: [(u64, u64); 6] = [
            (1, 0), (0, 1), (1, 1), (1, 2), (2, 3), (3, 5),
        ];
        for (n, (ea, eb)) in expected.iter().enumerate() {
            assert_eq!(power.v[0].to_decimal(), *ea, "φ^{}: a", n);
            assert_eq!(power.v[1].to_decimal(), *eb, "φ^{}: b", n);
            assert!(power.v[2].is_zero(), "φ^{}: v[2]", n);
            power = tri182_mul(&power, &phi);
        }
    }

    #[test]
    fn r_squared_value() {
        let val = tri182_to_f64(&R_SQUARED);
        let exact = 14.0 + 5.0 * (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((val - exact).abs() < 1e-10);
    }

    #[test]
    fn r_squared_integer_part_is_pi() {
        assert_eq!(R_SQUARED.v[0].to_decimal(), 14);
    }

    #[test]
    fn r_squared_norm_is_241() {
        let norm = tri182_norm(&R_SQUARED);
        assert_eq!(norm.integer_part().to_decimal(), 241);
        assert!(241 % 2 != 0 && 241 % 3 != 0 && 241 % 5 != 0
            && 241 % 7 != 0 && 241 % 11 != 0 && 241 % 13 != 0);
    }

    #[test]
    #[should_panic(expected = "negative norm")]
    fn norm_of_phi_panics() {
        // N(φ) = 0 + 0 − 1 = −1 → unsigned underflow → panic
        tri182_norm(&tri182_phi());
    }

    #[test]
    fn integer_embedding() {
        let val = tri182_int(TritInt::repunit(6));
        assert!(val.is_scalar());
        assert_eq!(val.v[0].to_decimal(), 364);
    }

    #[test]
    fn multiply_by_integer() {
        let three = tri182_int(TritInt::from_u64(3));
        let product = tri182_mul(&three, &R_SQUARED);
        assert_eq!(product.v[0].to_decimal(), 42);
        assert_eq!(product.v[1].to_decimal(), 15);
        assert!(product.v[2].is_zero());
    }

    #[test]
    fn v2_stays_zero() {
        let r = tri182_mul(&R_SQUARED, &ICOSA_CIRCUMRADIUS_SQ);
        assert!(r.v[2].is_zero());
    }

    #[test]
    fn add_commutative() {
        let a = tri182_add(&R_SQUARED, &ICOSA_CIRCUMRADIUS_SQ);
        let b = tri182_add(&ICOSA_CIRCUMRADIUS_SQ, &R_SQUARED);
        assert_eq!(a, b);
    }

    #[test]
    fn mul_commutative() {
        let a = tri182_mul(&R_SQUARED, &ICOSA_CIRCUMRADIUS_SQ);
        let b = tri182_mul(&ICOSA_CIRCUMRADIUS_SQ, &R_SQUARED);
        assert_eq!(a, b);
    }

    #[test]
    fn project_all_ones() {
        let trits = [1u8; 5];
        let r = project_to_zphi(&trits);
        assert_eq!(r.v[0].to_decimal(), 12); // sum fib(0..4) = 1+1+2+3+5
        assert_eq!(r.v[1].to_decimal(), 19); // sum fib(1..5) = 1+2+3+5+8
    }
}
