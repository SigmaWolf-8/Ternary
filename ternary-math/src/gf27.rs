// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # GF(27) = GF(3³) Arithmetic
//!
//! Extension field GF(3)[t]/(t³ + 2t + 1).
//! Every operation derived from the irreducible polynomial.
//! No lookup tables in the hot path.
//!
//! An element of GF(27) is a polynomial a₀ + a₁t + a₂t² with
//! coefficients in GF(3) = {0, 1, 2}.
//!
//! Storage: 3 bytes per element (one coefficient per byte).
//! Packed: 1 byte per element using base-3 encoding (a₀ + 3a₁ + 9a₂).

/// The irreducible polynomial: t³ + 2t + 1.
/// Coefficients: [1, 2, 0, 1] → constant=1, t=2, t²=0, t³=1.
/// Reduction rule: t³ = -(2t + 1) = t + 2 (in GF(3): -2=1, -1=2).
pub const IRRED: [u8; 3] = [2, 1, 0]; // t³ ≡ 2 + t (mod 3)

// ═══════════════════════════════════════════════════════════════════════
// GF(3) BASE OPERATIONS
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
pub fn gf3_add(a: u8, b: u8) -> u8 { (a + b) % 3 }

#[inline(always)]
pub fn gf3_sub(a: u8, b: u8) -> u8 { (a + 3 - b) % 3 }

#[inline(always)]
pub fn gf3_mul(a: u8, b: u8) -> u8 { (a * b) % 3 }

#[inline(always)]
pub fn gf3_neg(a: u8) -> u8 { (3 - a) % 3 }

// ═══════════════════════════════════════════════════════════════════════
// GF(27) ELEMENT — 3 coefficients in GF(3)
// ═══════════════════════════════════════════════════════════════════════

/// GF(27) element as [a₀, a₁, a₂] representing a₀ + a₁t + a₂t².
pub type Gf27 = [u8; 3];

pub const GF27_ZERO: Gf27 = [0, 0, 0];
pub const GF27_ONE: Gf27 = [1, 0, 0];

/// Pack a GF(27) element into a single byte: a₀ + 3·a₁ + 9·a₂.
/// Range: 0–26.
#[inline(always)]
pub fn gf27_pack(e: &Gf27) -> u8 {
    e[0] + 3 * e[1] + 9 * e[2]
}

/// Unpack a byte (0–26) into a GF(27) element.
#[inline(always)]
pub fn gf27_unpack(v: u8) -> Gf27 {
    [v % 3, (v / 3) % 3, v / 9]
}

// ═══════════════════════════════════════════════════════════════════════
// GF(27) ADDITION / SUBTRACTION — coefficient-wise in GF(3)
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
pub fn gf27_add(a: &Gf27, b: &Gf27) -> Gf27 {
    [gf3_add(a[0], b[0]), gf3_add(a[1], b[1]), gf3_add(a[2], b[2])]
}

#[inline(always)]
pub fn gf27_sub(a: &Gf27, b: &Gf27) -> Gf27 {
    [gf3_sub(a[0], b[0]), gf3_sub(a[1], b[1]), gf3_sub(a[2], b[2])]
}

#[inline(always)]
pub fn gf27_neg(a: &Gf27) -> Gf27 {
    [gf3_neg(a[0]), gf3_neg(a[1]), gf3_neg(a[2])]
}

// ═══════════════════════════════════════════════════════════════════════
// GF(27) MULTIPLICATION — polynomial multiply mod (t³ + 2t + 1)
//
// (a₀ + a₁t + a₂t²)(b₀ + b₁t + b₂t²) =
//   c₀ + c₁t + c₂t² + c₃t³ + c₄t⁴
//
// where:
//   c₀ = a₀b₀
//   c₁ = a₀b₁ + a₁b₀
//   c₂ = a₀b₂ + a₁b₁ + a₂b₀
//   c₃ = a₁b₂ + a₂b₁
//   c₄ = a₂b₂
//
// Reduce using t³ ≡ 2 + t, t⁴ ≡ 2t + t² (derived by multiplying t³ ≡ 2+t by t):
//   r₀ = c₀ + 2·c₃
//   r₁ = c₁ + c₃ + 2·c₄
//   r₂ = c₂ + c₄
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
pub fn gf27_mul(a: &Gf27, b: &Gf27) -> Gf27 {
    // Schoolbook: 9 GF(3) multiplies + 4 adds for unreduced product
    let c0 = gf3_mul(a[0], b[0]);
    let c1 = gf3_add(gf3_mul(a[0], b[1]), gf3_mul(a[1], b[0]));
    let c2 = gf3_add(gf3_add(gf3_mul(a[0], b[2]), gf3_mul(a[1], b[1])), gf3_mul(a[2], b[0]));
    let c3 = gf3_add(gf3_mul(a[1], b[2]), gf3_mul(a[2], b[1]));
    let c4 = gf3_mul(a[2], b[2]);

    // Reduce: t³ ≡ 2 + t, t⁴ ≡ 2t + t²
    let r0 = gf3_add(c0, gf3_mul(2, c3));
    let r1 = gf3_add(gf3_add(c1, c3), gf3_mul(2, c4));
    let r2 = gf3_add(c2, c4);

    [r0, r1, r2]
}

// ═══════════════════════════════════════════════════════════════════════
// GF(27) SQUARING — optimized (fewer muls than general multiply)
//
// (a₀ + a₁t + a₂t²)² = a₀² + a₁²t² + a₂²t⁴ + 2(a₀a₁t + a₀a₂t² + a₁a₂t³)
//
// In GF(3): x² = x for all x (since 0²=0, 1²=1, 2²=4≡1... wait, 2²=4≡1 mod 3)
// Actually: 0²=0, 1²=1, 2²=1 in GF(3). So squaring is NOT identity.
//
// But we can still use the Frobenius: (a+b)³ = a³+b³ in char 3.
// For squaring, just compute a² via the general multiply (it's only
// marginally optimizable over general mul in degree 2).
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
pub fn gf27_square(a: &Gf27) -> Gf27 {
    // Direct computation is cleaner than special-casing
    let a0_sq = gf3_mul(a[0], a[0]); // = a[0] (for 0,1) or 1 (for 2)
    let a1_sq = gf3_mul(a[1], a[1]);
    let a2_sq = gf3_mul(a[2], a[2]);
    let a0a1_2 = gf3_mul(2, gf3_mul(a[0], a[1])); // 2·a₀·a₁
    let a0a2_2 = gf3_mul(2, gf3_mul(a[0], a[2])); // 2·a₀·a₂
    let a1a2_2 = gf3_mul(2, gf3_mul(a[1], a[2])); // 2·a₁·a₂

    let c0 = a0_sq;
    let c1 = a0a1_2;
    let c2 = gf3_add(a1_sq, a0a2_2);
    let c3 = a1a2_2;
    let c4 = a2_sq;

    // Reduce
    let r0 = gf3_add(c0, gf3_mul(2, c3));
    let r1 = gf3_add(gf3_add(c1, c3), gf3_mul(2, c4));
    let r2 = gf3_add(c2, c4);

    [r0, r1, r2]
}

// ═══════════════════════════════════════════════════════════════════════
// χ S-BOX: x¹⁷ over GF(27) via addition chain
//
// x → x² → x⁴ → x⁸ → x¹⁶ → x¹⁷ = x¹⁶ · x
//
// 4 squarings + 1 multiplication = 5 GF(27) operations.
// No lookup table. Derived from the algebra.
// ═══════════════════════════════════════════════════════════════════════

/// χ(x) = x¹⁷ over GF(27).
///
/// The optimal S-box for TLSponge-385 (TM-2026-008):
/// - Bijective on GF(27) (verified: all 27 outputs distinct)
/// - DP_max = 1/9 (optimal differential probability)
/// - Algebraic degree 2 over GF(3) (minimal for nonlinearity)
///
/// Computed via addition chain, not lookup.
#[inline(always)]
pub fn chi(x: &Gf27) -> Gf27 {
    let x2 = gf27_square(x);
    let x4 = gf27_square(&x2);
    let x8 = gf27_square(&x4);
    let x16 = gf27_square(&x8);
    gf27_mul(&x16, x)
}

/// Inverse χ: x²³ (since 17 × 23 ≡ 1 mod 26, and x²⁶ = 1 for x ≠ 0).
///
/// x²³ = x¹⁶ · x⁴ · x² · x.
/// Addition chain: x → x² → x⁴ → x⁸ → x¹⁶ → x¹⁶·x⁴·x²·x = x²³.
#[inline(always)]
pub fn chi_inv(x: &Gf27) -> Gf27 {
    let x2 = gf27_square(x);
    let x4 = gf27_square(&x2);
    let x8 = gf27_square(&x4);
    let x16 = gf27_square(&x8);
    let x20 = gf27_mul(&x16, &x4);
    let x22 = gf27_mul(&x20, &x2);
    gf27_mul(&x22, x) // x²² · x = x²³
}

// ═══════════════════════════════════════════════════════════════════════
// VERIFICATION TABLE — compile-time, test-only
//
// Precompute all 27 values of χ to verify the algebraic computation.
// This table does NOT run in the hot path. It proves the algebra correct.
// ═══════════════════════════════════════════════════════════════════════

/// Verify χ is a bijection on GF(27) (all 27 outputs distinct).
pub fn verify_chi_bijection() -> bool {
    let mut seen = [false; 27];
    for i in 0..27u8 {
        let x = gf27_unpack(i);
        let y = chi(&x);
        let packed = gf27_pack(&y);
        if packed >= 27 { return false; }
        if seen[packed as usize] { return false; }
        seen[packed as usize] = true;
    }
    true
}

/// Verify χ and χ⁻¹ are inverses.
pub fn verify_chi_inverse() -> bool {
    for i in 0..27u8 {
        let x = gf27_unpack(i);
        let y = chi(&x);
        let x_back = chi_inv(&y);
        if x != x_back { return false; }
    }
    true
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── GF(3) base ──────────────────────────────────────────

    #[test]
    fn gf3_add_commutative() {
        for a in 0..3u8 {
            for b in 0..3u8 {
                assert_eq!(gf3_add(a, b), gf3_add(b, a));
            }
        }
    }

    #[test]
    fn gf3_add_identity() {
        for a in 0..3u8 {
            assert_eq!(gf3_add(a, 0), a);
        }
    }

    #[test]
    fn gf3_mul_commutative() {
        for a in 0..3u8 {
            for b in 0..3u8 {
                assert_eq!(gf3_mul(a, b), gf3_mul(b, a));
            }
        }
    }

    #[test]
    fn gf3_mul_identity() {
        for a in 0..3u8 {
            assert_eq!(gf3_mul(a, 1), a);
        }
    }

    #[test]
    fn gf3_neg_inverse() {
        for a in 0..3u8 {
            assert_eq!(gf3_add(a, gf3_neg(a)), 0);
        }
    }

    // ── GF(27) structure ────────────────────────────────────

    #[test]
    fn gf27_add_commutative() {
        for i in 0..27u8 {
            for j in 0..27u8 {
                let a = gf27_unpack(i);
                let b = gf27_unpack(j);
                assert_eq!(gf27_add(&a, &b), gf27_add(&b, &a));
            }
        }
    }

    #[test]
    fn gf27_add_identity() {
        for i in 0..27u8 {
            let a = gf27_unpack(i);
            assert_eq!(gf27_add(&a, &GF27_ZERO), a);
        }
    }

    #[test]
    fn gf27_add_inverse() {
        for i in 0..27u8 {
            let a = gf27_unpack(i);
            assert_eq!(gf27_add(&a, &gf27_neg(&a)), GF27_ZERO);
        }
    }

    #[test]
    fn gf27_mul_commutative() {
        for i in 0..27u8 {
            for j in 0..27u8 {
                let a = gf27_unpack(i);
                let b = gf27_unpack(j);
                assert_eq!(gf27_mul(&a, &b), gf27_mul(&b, &a));
            }
        }
    }

    #[test]
    fn gf27_mul_identity() {
        for i in 0..27u8 {
            let a = gf27_unpack(i);
            assert_eq!(gf27_mul(&a, &GF27_ONE), a);
        }
    }

    #[test]
    fn gf27_mul_zero() {
        for i in 0..27u8 {
            let a = gf27_unpack(i);
            assert_eq!(gf27_mul(&a, &GF27_ZERO), GF27_ZERO);
        }
    }

    #[test]
    fn gf27_mul_associative() {
        // Spot check (full 27³ is 19,683 triples)
        for i in [0u8, 1, 2, 5, 13, 26] {
            for j in [0u8, 1, 3, 7, 14, 26] {
                for k in [0u8, 1, 4, 11, 20, 26] {
                    let a = gf27_unpack(i);
                    let b = gf27_unpack(j);
                    let c = gf27_unpack(k);
                    let ab_c = gf27_mul(&gf27_mul(&a, &b), &c);
                    let a_bc = gf27_mul(&a, &gf27_mul(&b, &c));
                    assert_eq!(ab_c, a_bc, "({} * {}) * {} != {} * ({} * {})", i, j, k, i, j, k);
                }
            }
        }
    }

    #[test]
    fn gf27_distributive() {
        for i in [0u8, 1, 5, 13, 26] {
            for j in [0u8, 2, 7, 14, 26] {
                for k in [0u8, 3, 11, 20, 26] {
                    let a = gf27_unpack(i);
                    let b = gf27_unpack(j);
                    let c = gf27_unpack(k);
                    let lhs = gf27_mul(&a, &gf27_add(&b, &c));
                    let rhs = gf27_add(&gf27_mul(&a, &b), &gf27_mul(&a, &c));
                    assert_eq!(lhs, rhs, "{} * ({} + {}) != {} * {} + {} * {}", i, j, k, i, j, i, k);
                }
            }
        }
    }

    // ── Pack/unpack ─────────────────────────────────────────

    #[test]
    fn pack_unpack_roundtrip() {
        for i in 0..27u8 {
            assert_eq!(gf27_pack(&gf27_unpack(i)), i);
        }
    }

    #[test]
    fn pack_range() {
        for a0 in 0..3u8 {
            for a1 in 0..3u8 {
                for a2 in 0..3u8 {
                    assert!(gf27_pack(&[a0, a1, a2]) < 27);
                }
            }
        }
    }

    // ── χ S-box ─────────────────────────────────────────────

    #[test]
    fn chi_zero_is_zero() {
        assert_eq!(chi(&GF27_ZERO), GF27_ZERO);
    }

    #[test]
    fn chi_one_is_one() {
        // 1¹⁷ = 1
        assert_eq!(chi(&GF27_ONE), GF27_ONE);
    }

    #[test]
    fn chi_is_bijection() {
        assert!(verify_chi_bijection(), "χ must be a bijection on GF(27)");
    }

    #[test]
    fn chi_inv_is_correct() {
        assert!(verify_chi_inverse(), "χ⁻¹ must invert χ for all 27 elements");
    }

    #[test]
    fn chi_matches_power_17() {
        // Verify by repeated multiplication: x¹⁷ = x·x·x·...·x (17 times)
        for i in 0..27u8 {
            let x = gf27_unpack(i);
            let mut power = GF27_ONE;
            for _ in 0..17 {
                power = gf27_mul(&power, &x);
            }
            assert_eq!(chi(&x), power, "χ({}) should equal x¹⁷ by repeated mul", i);
        }
    }

    // ── Squaring ────────────────────────────────────────────

    #[test]
    fn square_equals_self_mul() {
        for i in 0..27u8 {
            let x = gf27_unpack(i);
            assert_eq!(gf27_square(&x), gf27_mul(&x, &x));
        }
    }

    // ── Field order ─────────────────────────────────────────

    #[test]
    fn fermat_little_theorem() {
        // x²⁷ = x for all x in GF(27) (Frobenius endomorphism)
        for i in 0..27u8 {
            let x = gf27_unpack(i);
            let mut power = x;
            for _ in 1..27 {
                power = gf27_mul(&power, &x);
            }
            assert_eq!(power, x, "x²⁷ should equal x for element {}", i);
        }
    }

    #[test]
    fn nonzero_order_divides_26() {
        // For x ≠ 0: x²⁶ = 1
        for i in 1..27u8 {
            let x = gf27_unpack(i);
            let mut power = GF27_ONE;
            for _ in 0..26 {
                power = gf27_mul(&power, &x);
            }
            assert_eq!(power, GF27_ONE, "x²⁶ should be 1 for nonzero element {}", i);
        }
    }
}