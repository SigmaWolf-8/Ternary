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

//! # Unified Public Constants — Salvi Framework
//!
//! The **singular set of shared constants** for all ternary mathematics modules.
//! Every module in `ternary-math` imports its fundamental values from here —
//! no module defines its own copy. This guarantees consistency across:
//!
//! - `ternary_circle` (Z₂₈ geometry)
//! - `tribonacci` (base-3 recurrence)
//! - `borromean` (three-party topology)
//! - `clifford` (Cl(3,0)/GF(3) gate compression)
//! - `torus` (ternary torus network topology)
//! - `plenum_square` (magic square / reciprocal-lattice mixer)
//!
//! ## Design Principle
//!
//! All constants derive from the circle quadratic **x² − 40x + 364 = 0**,
//! whose coefficients are base-3 repunits R₄ = 40 and R₆ = 364.
//!
//! Everything — π = 14, 1 radian = 13°, Z₂₈ cyclic group, coprime walk,
//! UV spectral partition, HModal signal — follows from this single equation.
//!
//! ## Verification Architecture
//!
//! - **Rule 1:** No `assert!` in production code paths. All verification is
//!   compile-time const assertions or `#[test]` functions.
//! - **Rule 2:** No f64 constants in the mathematical core. The only f64
//!   values are in the clearly marked conversion section at the bottom.

use std::f64::consts::PI as STD_PI;
use crate::trit_int::TritInt;

// ══════════════════════════════════════════════════════════════
// §0  TERNARY-NATIVE CONSTANTS (SOURCE OF TRUTH)
//
// Every mathematical constant in the Salvi Framework is a TritInt
// derived from the master formula x² − R₄·x + R₆ = 0.
// TritInt is the native numeric type. Binary values below (§1+)
// are BOUNDARY CROSSINGS for consumers not yet migrated.
//
// Derivation chain:
//   repunit(n)  →  circle quadratic  →  roots  →  arc equation  →  everything
//
// No decimal literal appears in this section. Every value is
// constructed from its trit representation or derived via const
// arithmetic from prior values.
// ══════════════════════════════════════════════════════════════

// ── Repunits: Rₙ = 111...1₃ (n ones) = (3ⁿ − 1)/2 ─────────

/// R₁ = 1₃. The unit.
pub const T_REPUNIT_1: TritInt = TritInt::repunit(1);
/// R₂ = 11₃. The quadratic root offset.
pub const T_REPUNIT_2: TritInt = TritInt::repunit(2);
/// R₃ = 111₃. The radian.
pub const T_REPUNIT_3: TritInt = TritInt::repunit(3);
/// R₄ = 1111₃. Sum of circle quadratic roots. Inline buffer width.
pub const T_REPUNIT_4: TritInt = TritInt::repunit(4);
/// R₅ = 11111₃.
pub const T_REPUNIT_5: TritInt = TritInt::repunit(5);
/// R₆ = 111111₃. The full circle. 364°.
pub const T_REPUNIT_6: TritInt = TritInt::repunit(6);

// ── Circle quadratic: x² − R₄·x + R₆ = 0 ──────────────────

/// Root x₁ = 14 = R₃ + 1 = 112₃.
pub const T_ROOT_X1: TritInt = TritInt::from_trits(&[2, 1, 1]);
/// Root x₂ = 26 = 2·R₃ = 222₃.
pub const T_ROOT_X2: TritInt = TritInt::from_trits(&[2, 2, 2]);
/// Discriminant Δ = R₄ − 4·R₆ ... = 144 = 12² = 12100₃.
pub const T_DISCRIMINANT: TritInt = TritInt::from_trits(&[0, 0, 1, 2, 1]);
/// √Δ = 12 = 110₃.
pub const T_DISCRIMINANT_SQRT: TritInt = TritInt::from_trits(&[0, 1, 1]);
/// Δ₂ = 729 = 3⁶ = 1000000₃.
pub const T_DISCRIMINANT_2: TritInt = TritInt::from_trits(&[0, 0, 0, 0, 0, 0, 1]);
/// √Δ₂ = 27 = 3³ = 1000₃.
pub const T_DISCRIMINANT_2_SQRT: TritInt = TritInt::from_trits(&[0, 0, 0, 1]);

// ── Arc equation: arc² − 832·arc + 118300 = 0 ──────────────

/// arc₁ = 182 = 2 × 7 × 13 = 20202₃. The half-turn (semi-arc).
pub const T_ARC_ROOT_SEMI: TritInt = TritInt::from_trits(&[2, 0, 2, 0, 2]);
/// arc₂ = 650 = 2 × 5² × 13 = 212202₃. The complementary arc.
pub const T_ARC_ROOT_COMP: TritInt = TritInt::from_trits(&[2, 0, 0, 0, 2, 2]);
/// Green arc (effective) = 286 = 2 × 11 × 13 = 101121₃.
pub const T_GREEN_ARC_EFF: TritInt = TritInt::from_trits(&[1, 2, 1, 1, 0, 1]);

// ── UV Spectral partition ───────────────────────────────────

/// λ_EUV = 91 = 7 × 13 = 10101₃.
pub const T_LAMBDA_EUV: TritInt = TritInt::from_trits(&[1, 0, 1, 0, 1]);
/// λ_UVC = 182 = 2 × 7 × 13 = T_ARC_ROOT_SEMI.
pub const T_LAMBDA_UVC: TritInt = T_ARC_ROOT_SEMI;
/// λ_UVB = 286 = 2 × 11 × 13 = T_GREEN_ARC_EFF.
pub const T_LAMBDA_UVB: TritInt = T_GREEN_ARC_EFF;
/// λ_UVA = 364 = R₆ = T_REPUNIT_6.
pub const T_LAMBDA_UVA: TritInt = T_REPUNIT_6;

// ── Polygon generators (coprime source set) ─────────────────

/// 3 = 10₃. Triangle.
pub const T_POLYGON_3: TritInt = TritInt::from_trits(&[0, 1]);
/// 4 = R₂ = 11₃. Square.
pub const T_POLYGON_4: TritInt = T_REPUNIT_2;
/// 5 = 12₃. Pentagon.
pub const T_POLYGON_5: TritInt = TritInt::from_trits(&[2, 1]);
/// 7 = 21₃. Heptagon.
pub const T_POLYGON_7: TritInt = TritInt::from_trits(&[1, 2]);
/// 8 = 22₃. Octagon.
pub const T_POLYGON_8: TritInt = TritInt::from_trits(&[2, 2]);
/// 9 = 100₃ = 3². Nonagon.
pub const T_POLYGON_9: TritInt = TritInt::from_trits(&[0, 0, 1]);
/// 11 = 102₃. Hendecagon. First coprime generator.
pub const T_POLYGON_11: TritInt = TritInt::from_trits(&[2, 0, 1]);
/// 13 = R₃ = 111₃. Tridecagon. The radian polygon.
pub const T_POLYGON_13: TritInt = T_REPUNIT_3;
/// 14 = 112₃ = x₁. Tetradecagon. The π-gon.
pub const T_POLYGON_14: TritInt = T_ROOT_X1;
/// 15 = 120₃ = 3 × 5. Pentadecagon. Bridge polygon.
pub const T_POLYGON_15: TritInt = TritInt::from_trits(&[0, 2, 1]);

// ── Plenum Square ───────────────────────────────────────────

/// Magic constant = 333 = 110100₃.
pub const T_MAGIC_CONSTANT: TritInt = TritInt::from_trits(&[0, 0, 1, 0, 1, 1]);
/// Center = 111 = 11010₃.
pub const T_CENTER: TritInt = TritInt::from_trits(&[0, 1, 0, 1, 1]);

// ── Key derived products ────────────────────────────────────

/// Z₂₈ group order = 28 = 1001₃.
pub const T_Z28_ORDER: TritInt = TritInt::from_trits(&[1, 0, 0, 1]);
/// Circumference = 1554 = 2010120₃.
pub const T_CIRCUMFERENCE: TritInt = TritInt::from_trits(&[0, 2, 1, 0, 1, 0, 2]);

// ── Compile-time derivation verification ────────────────────

const _: () = {
    // Repunit values: Rₙ = (3ⁿ − 1)/2
    assert!(T_REPUNIT_1.to_u32_const() == 1);
    assert!(T_REPUNIT_2.to_u32_const() == 4);
    assert!(T_REPUNIT_3.to_u32_const() == 13);
    assert!(T_REPUNIT_4.to_u32_const() == 40);
    assert!(T_REPUNIT_5.to_u32_const() == 121);
    assert!(T_REPUNIT_6.to_u32_const() == 364);

    // Circle quadratic: x₁ + x₂ = R₄, x₁ × x₂ = R₆
    assert!(T_ROOT_X1.const_add(T_ROOT_X2).to_u32_const() == T_REPUNIT_4.to_u32_const());
    assert!(T_ROOT_X1.const_mul(T_ROOT_X2).to_u32_const() == T_REPUNIT_6.to_u32_const());

    // Discriminant: Δ = R₄² − 4·R₆ = 1600 − 1456 = 144
    assert!(T_DISCRIMINANT.to_u32_const() == 144);
    assert!(T_DISCRIMINANT_SQRT.const_mul(T_DISCRIMINANT_SQRT).to_u32_const() == 144);

    // Δ₂ = 3⁶ = 729
    assert!(T_DISCRIMINANT_2.to_u32_const() == 729);
    assert!(T_DISCRIMINANT_2_SQRT.const_mul(T_DISCRIMINANT_2_SQRT).to_u32_const() == 729);

    // Arc semi: 182 = 2 × 7 × 13 = R₆/2
    assert!(T_ARC_ROOT_SEMI.to_u32_const() == 182);

    // UV spectral: λ_EUV × 2 = λ_UVC = arc_semi
    assert!(T_LAMBDA_EUV.const_mul(TritInt::from_trits(&[2])).to_u32_const()
            == T_LAMBDA_UVC.to_u32_const());

    // Polygon identities
    assert!(T_POLYGON_15.to_u32_const() == T_POLYGON_3.const_mul(T_POLYGON_5).to_u32_const());
    assert!(T_POLYGON_14.to_u32_const() == T_ROOT_X1.to_u32_const()); // 14-gon = π

    // Magic constant: 3 × center
    assert!(T_MAGIC_CONSTANT.to_u32_const()
            == T_CENTER.const_mul(TritInt::from_trits(&[0, 1])).to_u32_const());
};

// ══════════════════════════════════════════════════════════════
// §1+  u32 BOUNDARY CROSSINGS — BACKWARD COMPATIBILITY ONLY
//
// These exist for files NOT in this delivery (ternary_circle.rs,
// trit.rs, torus.rs, tribonacci.rs) that still import u32 constants.
// New code MUST cross through T_ constants at point of use:
//
//   const R3: usize = crate::constants::T_REPUNIT_3.to_u32_const() as usize;
//
// NOT:
//
//   const R3: usize = crate::constants::REPUNIT_3 as usize;
//
// Each u32 export below is derived from its T_ source of truth.
// Delete each one as its consumers are migrated.
// ══════════════════════════════════════════════════════════════

// ══════════════════════════════════════════════════════════════
// §1  REPUNIT FAMILY — BOUNDARY CROSSINGS
//     Derived from §0 TritInt source of truth. u32 for host use.
// ══════════════════════════════════════════════════════════════

/// The ternary radix — 3.
pub const TERNARY_BASE: u32 = T_REPUNIT_1.const_add(TritInt::from_trits(&[2])).to_u32_const();

/// R₁ — boundary crossing from T_REPUNIT_1.
pub const REPUNIT_1: u32 = T_REPUNIT_1.to_u32_const();
/// R₂ — boundary crossing from T_REPUNIT_2.
pub const REPUNIT_2: u32 = T_REPUNIT_2.to_u32_const();
/// R₃ — boundary crossing from T_REPUNIT_3. The radian.
pub const REPUNIT_3: u32 = T_REPUNIT_3.to_u32_const();
/// R₄ — boundary crossing from T_REPUNIT_4. Sum of circle quadratic roots.
pub const REPUNIT_4: u32 = T_REPUNIT_4.to_u32_const();
/// R₅ — boundary crossing from T_REPUNIT_5.
pub const REPUNIT_5: u32 = T_REPUNIT_5.to_u32_const();
/// R₆ — boundary crossing from T_REPUNIT_6. The full circle.
pub const REPUNIT_6: u32 = T_REPUNIT_6.to_u32_const();

/// Master repunit generating function: Rₙ = (3ⁿ − 1) / (3 − 1).
pub const fn repunit(n: u32) -> u32 {
    (TERNARY_BASE.pow(n) - 1) / (TERNARY_BASE - 1)
}

// ══════════════════════════════════════════════════════════════
// §2  CIRCLE QUADRATIC (TM-2026-017 §2.2)
//     x² − R₄·x + R₆ = 0  →  x² − 40x + 364 = 0
// ══════════════════════════════════════════════════════════════

/// Vieta sum x₁ + x₂ = R₄ — derived from T_REPUNIT_4.
pub const QUAD_SUM: u32 = REPUNIT_4;

/// Vieta product x₁ · x₂ = R₆ — derived from T_REPUNIT_6.
pub const QUAD_PRODUCT: u32 = REPUNIT_6;

/// Discriminant Δ = R₄² − 4·R₆ — derived from T_DISCRIMINANT.
pub const DISCRIMINANT: u32 = T_DISCRIMINANT.to_u32_const();

/// √Δ — derived from T_DISCRIMINANT_SQRT.
pub const DISCRIMINANT_SQRT: u32 = T_DISCRIMINANT_SQRT.to_u32_const();

/// Smaller root x₁ = π — derived from T_ROOT_X1.
pub const ROOT_X1: u32 = T_ROOT_X1.to_u32_const();

/// Larger root x₂ = R₆/π — derived from T_ROOT_X2.
pub const ROOT_X2: u32 = T_ROOT_X2.to_u32_const();

// ══════════════════════════════════════════════════════════════
// §3  UNIFIED EQUATION — BOUNDARY CROSSINGS
//     Derived from §0 TritInt or from §1/§2 boundary values.
// ══════════════════════════════════════════════════════════════

/// |linear coefficient| = R₄(R₄−1) − 2R₆.
pub const UNIFIED_LINEAR: u32 = REPUNIT_4 * (REPUNIT_4 - 1) - 2 * REPUNIT_6;

/// R₆ − R₄ + 1.
pub const UNIFIED_FACTOR: u32 = REPUNIT_6 - REPUNIT_4 + 1;

/// Constant term = R₆ × (R₆ − R₄ + 1).
pub const UNIFIED_CONSTANT: u32 = REPUNIT_6 * UNIFIED_FACTOR;

/// Δ_arc = UNIFIED_LINEAR² − 4·UNIFIED_CONSTANT.
pub const UNIFIED_DISC: u32 = UNIFIED_LINEAR * UNIFIED_LINEAR - 4 * UNIFIED_CONSTANT;

/// √Δ_arc — derived from T_ARC_ROOT_COMP − T_ARC_ROOT_SEMI.
pub const UNIFIED_DISC_SQRT: u32 = T_ARC_ROOT_COMP.to_u32_const() - T_ARC_ROOT_SEMI.to_u32_const();

/// Semicircle root — derived from T_ARC_ROOT_SEMI.
pub const ARC_ROOT_SEMI: u32 = T_ARC_ROOT_SEMI.to_u32_const();

/// Complementary root — derived from T_ARC_ROOT_COMP.
pub const ARC_ROOT_COMP: u32 = T_ARC_ROOT_COMP.to_u32_const();

/// Green arc effective — derived from T_GREEN_ARC_EFF.
pub const GREEN_ARC_EFF: u32 = T_GREEN_ARC_EFF.to_u32_const();

/// Center c = (arc + R₄)/2 — derived from T_CENTER.
pub const CENTER: u32 = T_CENTER.to_u32_const();

/// Radius numerator = CENTER.
pub const RADIUS_NUM: u32 = CENTER;

/// Radius denominator.
pub const RADIUS_DEN: u32 = 2;

/// Δ₂ — derived from T_DISCRIMINANT_2.
pub const DISCRIMINANT_2: u32 = T_DISCRIMINANT_2.to_u32_const();

/// √Δ₂ — derived from T_DISCRIMINANT_2_SQRT.
pub const DISCRIMINANT_2_SQRT: u32 = T_DISCRIMINANT_2_SQRT.to_u32_const();

/// Magic constant — derived from T_MAGIC_CONSTANT.
pub const MAGIC_CONSTANT: u32 = T_MAGIC_CONSTANT.to_u32_const();

/// Circumference — derived from T_CIRCUMFERENCE.
pub const CIRCUMFERENCE: u32 = T_CIRCUMFERENCE.to_u32_const();

// ══════════════════════════════════════════════════════════════
// §4  SQUARED CIRCLE (TM-2026-017 §3)
// ══════════════════════════════════════════════════════════════

/// Unit circle area = π = x₁ — derived from T_ROOT_X1.
pub const UNIT_CIRCLE_AREA: u32 = ROOT_X1;

/// Radian circle area = π × R₃ — derived from T_ARC_ROOT_SEMI.
pub const RADIAN_CIRCLE_AREA: u32 = ARC_ROOT_SEMI;

/// Side² of squared unit circle = π — same as ROOT_X1.
pub const SQUARED_SIDE_SQ_UNIT: u32 = ROOT_X1;

/// Side² of squared radian circle — same as ARC_ROOT_SEMI.
pub const SQUARED_SIDE_SQ_RADIAN: u32 = ARC_ROOT_SEMI;

// ══════════════════════════════════════════════════════════════
// §5  ANGULAR CONVERSION FACTOR
// ══════════════════════════════════════════════════════════════

/// Standard circle in degrees — external reference (not derived from ternary axiom).
pub const STD_CIRCLE_DEG: u32 = 360;

/// Angular conversion factor numerator: κ = R₆/360 = 91/90. Same as λ_EUV.
pub const ANGULAR_CONV_NUM: u32 = T_LAMBDA_EUV.to_u32_const();

/// Angular conversion factor denominator.
pub const ANGULAR_CONV_DEN: u32 = STD_CIRCLE_DEG / T_REPUNIT_2.to_u32_const();

// ══════════════════════════════════════════════════════════════
// §6  UV SPECTRAL WAVELENGTHS (TM-2026-017 §16, §2.5)
// ══════════════════════════════════════════════════════════════

/// Quarter-turn = 7 × 13 = 7 radians — derived from T_LAMBDA_EUV.
pub const LAMBDA_EUV: u32 = T_LAMBDA_EUV.to_u32_const();

/// Half-turn = 14 × 13 = π radians — same as ARC_ROOT_SEMI.
pub const LAMBDA_UVC: u32 = ARC_ROOT_SEMI;

/// Green arc effective = 22 × 13 = 22 radians — same as GREEN_ARC_EFF.
pub const LAMBDA_UVB: u32 = GREEN_ARC_EFF;

/// Full circle = 28 × 13 = 2π radians — same as REPUNIT_6.
pub const LAMBDA_UVA: u32 = REPUNIT_6;

/// Far-UVC = 2 × CENTER.
pub const LAMBDA_FAR_UVC: u32 = 2 * CENTER;

/// XeCl excimer = 4 × 7 × 11.
pub const LAMBDA_EXCIMER: u32 = T_REPUNIT_2.to_u32_const()
    * T_POLYGON_7.to_u32_const()
    * T_POLYGON_11.to_u32_const();

/// Narrowband UVB = e₂ = pq + pr + qr = 7×11 + 7×13 + 11×13.
pub const LAMBDA_NB_UVB: u32 = T_POLYGON_7.to_u32_const() * T_POLYGON_11.to_u32_const()
    + T_POLYGON_7.to_u32_const() * REPUNIT_3
    + T_POLYGON_11.to_u32_const() * REPUNIT_3;

/// EUV|UVC boundary = (λ_EUV + λ_UVC) / 2. Truncated half-integer.
pub const BOUNDARY_EUV_UVC: u32 = (LAMBDA_EUV + LAMBDA_UVC) / 2;

/// UVC|UVB boundary = (λ_UVC + λ_UVB) / 2.
pub const BOUNDARY_UVC_UVB: u32 = (LAMBDA_UVC + LAMBDA_UVB) / 2;

/// UVB|UVA boundary = (λ_UVB + λ_UVA) / 2 = UNIFIED_FACTOR.
pub const BOUNDARY_UVB_UVA: u32 = (LAMBDA_UVB + LAMBDA_UVA) / 2;

/// UV|Visible boundary — external reference.
pub const BOUNDARY_UV_VIS: u32 = 400;

/// Vacuum bias numerator: (1/R_H − 91)/91 ≈ 0.00193.
pub const VACUUM_BIAS_NUM: u32 = 193;

/// Vacuum bias denominator.
pub const VACUUM_BIAS_DEN: u32 = 100_000;

// ══════════════════════════════════════════════════════════════
// §7  COPRIME WALK LANDSCAPE (TM-2026-017 §10)
//
// All coprime groups derived from the 13 polygon set.
// Member arrays and LCM arrays are indexed together:
//   COPRIME_*S[i]  →  COPRIME_*_LCMS[i]
//
// Two paths through the landscape:
//
// COMPRESSION — pentadecagon (15) kept whole.
//   Source set: {7, 11, 13, 14, 15}.
//   Excluded: (7, 14) — gcd = 7.
//   Groups: pairs (9), triples (7), quadruples (2).
//
// EXPANSION — pentadecagon decomposed into factors 3 and 5.
//   Source set: {3, 4, 5, 7, 8, 9, 11, 13}.
//   Excluded: (3, 9) — gcd = 3; (4, 8) — gcd = 4.
//   Groups: quintuples (28 exist, 5 key stored), sextuples (4).
//
// Compression → expansion duality: decomposing 15 into {3, 5}
// unlocks 12× larger LCMs (max 30,030 → max 360,360).
//
// The 4 sextuples are the 4 ways to choose one from each
// expansion conflict: {3|9} × {4|8}, with {5, 7, 11, 13}
// always present. This is why exactly 4 exist and size 7
// is impossible.
// ══════════════════════════════════════════════════════════════

/// Pentadecagon — derived from T_POLYGON_15.
pub const PENTADECAGON: u32 = T_POLYGON_15.to_u32_const();

// Polygon generator aliases for table readability.
// Source of truth: §0 T_POLYGON_* TritInt constants.
const P3: u32 = T_POLYGON_3.to_u32_const();
const P4: u32 = T_POLYGON_4.to_u32_const();
const P5: u32 = T_POLYGON_5.to_u32_const();
const P7: u32 = T_POLYGON_7.to_u32_const();
const P8: u32 = T_POLYGON_8.to_u32_const();
const P9: u32 = T_POLYGON_9.to_u32_const();
const P11: u32 = T_POLYGON_11.to_u32_const();
const P13: u32 = T_POLYGON_13.to_u32_const();
const P14: u32 = T_POLYGON_14.to_u32_const();
const P15: u32 = T_POLYGON_15.to_u32_const();

/// Excluded pairs — gcd > 1 prevents coexistence.
pub const EXCLUDED_PAIRS: [(u32, u32); 3] = [(P7, P14), (P3, P9), (P4, P8)];

// ── COMPRESSION PATH ─────────────────────────────────────────
// Source set: {7, 11, 13, 14, 15}. Pentadecagon kept whole.

// ── Pairs (9, complete) ──────────────────────────────────────

/// 9 coprime pairs from {7, 11, 13, 14, 15} (TM-2026-017 §10.3).
pub const COPRIME_PAIRS: [(u32, u32); 9] = [
    (P7, P11), (P7, P13), (P7, P15),
    (P11, P13), (P11, P14), (P11, P15),
    (P13, P14), (P13, P15), (P14, P15),
];

/// LCMs of the 9 coprime pairs — products of coprime generators.
pub const COPRIME_PAIR_LCMS: [u32; 9] = [
    P7*P11, P7*P13, P7*P15,
    P11*P13, P11*P14, P11*P15,
    P13*P14, P13*P15, P14*P15,
];

// ── Triples (7, complete) ────────────────────────────────────

/// 7 coprime triples from {7, 11, 13, 14, 15} (TM-2026-017 §10.3).
/// COPRIME_TRIPLES[0] = [7, 11, 13] — the primary generators from the
/// arc factorizations: 182 = 2×7×13, 286 = 2×11×13 → ratio 7:11.
pub const COPRIME_TRIPLES: [[u32; 3]; 7] = [
    [P7, P11, P13], [P7, P11, P15], [P7, P13, P15],
    [P11, P13, P14], [P11, P13, P15], [P11, P14, P15], [P13, P14, P15],
];

/// LCMs of the 7 coprime triples — products of coprime generators.
pub const COPRIME_TRIPLE_LCMS: [u32; 7] = [
    P7*P11*P13, P7*P11*P15, P7*P13*P15,
    P11*P13*P14, P11*P13*P15, P11*P14*P15, P13*P14*P15,
];

// ── Quadruples (2, complete) ─────────────────────────────────

/// 2 compression-path coprime quadruples (TM-2026-017 §10.4).
pub const COPRIME_QUADRUPLES: [[u32; 4]; 2] = [
    [P7, P11, P13, P15],
    [P11, P13, P14, P15],
];

/// LCMs of the 2 compression-path quadruples — products of coprime generators.
pub const COPRIME_QUADRUPLE_LCMS: [u32; 2] = [P7*P11*P13*P15, P11*P13*P14*P15];

// ── EXPANSION PATH ───────────────────────────────────────────
// Pentadecagon decomposed: 15 → {3, 5}.
// Source set: {3, 4, 5, 7, 8, 9, 11, 13}.

// ── Quintuples (28 exist, 5 key stored) ──────────────────────

/// 5 key expansion-path coprime quintuples (TM-2026-017 §10.5.1).
/// 28 valid quintuples exist from the full polygon set; these are
/// the architecturally significant ones. The first two demonstrate
/// the decompression identity: quint[0] == quad[0] in LCM, and
/// quint[1] == quad[1] in LCM, because 3 × 5 = 15.
pub const COPRIME_QUINTUPLES: [[u32; 5]; 5] = [
    [P3, P5, P7, P11, P13],
    [P3, P5, P11, P13, P14],
    [P3, P4, P7, P11, P13],
    [P5, P7, P8, P11, P13],
    [P5, P7, P9, P11, P13],
];

/// LCMs of the 5 key quintuples — products of coprime generators.
pub const COPRIME_QUINTUPLE_LCMS: [u32; 5] = [
    P3*P5*P7*P11*P13, P3*P5*P11*P13*P14, P3*P4*P7*P11*P13,
    P5*P7*P8*P11*P13, P5*P7*P9*P11*P13,
];

// ── Sextuples (4, complete) ──────────────────────────────────

/// 4 expansion-path coprime sextuples (TM-2026-017 §10.5.2).
/// These are the 4 ways to choose one from each expansion conflict:
/// {3|9} × {4|8}, with {5, 7, 11, 13} always present.
/// No group of 7 exists — size 6 is the structural limit.
pub const COPRIME_SEXTUPLES: [[u32; 6]; 4] = [
    [P3, P4, P5, P7, P11, P13],
    [P3, P5, P7, P8, P11, P13],
    [P4, P5, P7, P9, P11, P13],
    [P5, P7, P8, P9, P11, P13],
];

/// LCMs of the 4 expansion-path sextuples — products of coprime generators.
pub const COPRIME_SEXTUPLE_LCMS: [u32; 4] = [
    P3*P4*P5*P7*P11*P13, P3*P5*P7*P8*P11*P13,
    P4*P5*P7*P9*P11*P13, P5*P7*P8*P9*P11*P13,
];

// ── Key aliases ──────────────────────────────────────────────
// Heavily cross-referenced in §8, §9, §10, §17.

/// LCM of primary coprime generators — same as COPRIME_TRIPLE_LCMS[0].
pub const LCM_PRIMARY: u32 = COPRIME_TRIPLE_LCMS[0];

/// Maximum sextuple LCM — same as COPRIME_SEXTUPLE_LCMS[3].
pub const LCM_SEXT_MAX: u32 = COPRIME_SEXTUPLE_LCMS[3];

/// Circle × coprime walk: R₆ × LCM_PRIMARY.
pub const GEOMETRIC_SPECTRAL_PRODUCT: u32 = REPUNIT_6 * LCM_PRIMARY;

// ── 3D position counts (× 729 = Δ₂ = 3⁶) ───────────────────

/// 3D positions for the primary triple: LCM_PRIMARY × Δ₂.
pub const POS_3D_PRIMARY: u64 = (LCM_PRIMARY as u64) * (DISCRIMINANT_2 as u64);

/// 3D positions for the 2 compression-path quadruples.
/// Indexed to match COPRIME_QUADRUPLES.
pub const POS_3D_QUADRUPLES: [u64; 2] = [
    (COPRIME_QUADRUPLE_LCMS[0] as u64) * (DISCRIMINANT_2 as u64),
    (COPRIME_QUADRUPLE_LCMS[1] as u64) * (DISCRIMINANT_2 as u64),
];

/// 3D positions for the 4 expansion-path sextuples.
/// Indexed to match COPRIME_SEXTUPLES.
pub const POS_3D_SEXTUPLES: [u64; 4] = [
    (COPRIME_SEXTUPLE_LCMS[0] as u64) * (DISCRIMINANT_2 as u64),
    (COPRIME_SEXTUPLE_LCMS[1] as u64) * (DISCRIMINANT_2 as u64),
    (COPRIME_SEXTUPLE_LCMS[2] as u64) * (DISCRIMINANT_2 as u64),
    (COPRIME_SEXTUPLE_LCMS[3] as u64) * (DISCRIMINANT_2 as u64),
];

// ══════════════════════════════════════════════════════════════
// §8  CCP BRIDGE ANALYSIS (Circle × Coprime Product deficit)
// ══════════════════════════════════════════════════════════════

/// GEOMETRIC_SPECTRAL_PRODUCT − LCM_SEXT_MAX. Null harmonic deficit.
pub const NULL_HARMONIC_DEFICIT: u32 = GEOMETRIC_SPECTRAL_PRODUCT - LCM_SEXT_MAX;

/// Bridge ratio — same as angular conversion factor κ = 91/90.
pub const BRIDGE_RATIO_NUM: u32 = ANGULAR_CONV_NUM;

/// Bridge ratio denominator.
pub const BRIDGE_RATIO_DEN: u32 = ANGULAR_CONV_DEN;

/// Deficit rate numerator: NULL_HARMONIC_DEFICIT / GEOMETRIC_SPECTRAL_PRODUCT = 1/91.
pub const DEFICIT_RATE_NUM: u32 = 1;

/// Deficit rate denominator = λ_EUV.
pub const DEFICIT_RATE_DEN: u32 = LAMBDA_EUV;

/// 3D geometric-spectral: GEOMETRIC_SPECTRAL_PRODUCT × Δ₂.
pub const POS_3D_GEOM_SPECTRAL: u64 = (GEOMETRIC_SPECTRAL_PRODUCT as u64) * (DISCRIMINANT_2 as u64);

/// 3D null deficit: NULL_HARMONIC_DEFICIT × Δ₂.
pub const POS_3D_NULL_DEFICIT: u64 = (NULL_HARMONIC_DEFICIT as u64) * (DISCRIMINANT_2 as u64);

// ══════════════════════════════════════════════════════════════
// §9  PERFECT HASH COEFFICIENTS (TM-2026-028a §2–§3)
// ══════════════════════════════════════════════════════════════

/// CRT coefficients for odd-prime quadruple (7, 11, 13, 15).
/// Tuple order: (modulus, coefficient). Each c satisfies gcd(c, m) = 1 and c mod m ≠ 0.
pub const HASH_COEFF_A: [(u32, u32); 4] = [(P7, 2), (P11, 3), (P13, 5), (P15, 7)];

/// CRT coefficients for π-gon quadruple.
pub const HASH_COEFF_B: [(u32, u32); 4] = [(P11, 2), (P13, 3), (P14, 5), (P15, 7)];

/// CRT coefficients for maximum sextuple.
pub const HASH_COEFF_SEXT: [(u32, u32); 6] = [(P5, 2), (P7, 3), (P8, 5), (P9, 7), (P11, 4), (P13, 6)];

/// HModal mixer constant A (multiply after shift-12). Odd → bijective mod 2⁶⁴.
pub const HMODAL_MIX_A: u64 = 0x91e3d5c9a3e5d1c3;

/// HModal mixer constant B (multiply after shift-25). Odd → bijective mod 2⁶⁴.
pub const HMODAL_MIX_B: u64 = 0x1001c4b5e9f7a2d1;

/// Mixer shift 1.
pub const HMODAL_MIX_SHIFT_1: u32 = 12;

/// Mixer shift 2.
pub const HMODAL_MIX_SHIFT_2: u32 = 25;

/// Mixer shift 3.
pub const HMODAL_MIX_SHIFT_3: u32 = 33;

// ══════════════════════════════════════════════════════════════
// §10  HMODAL SIGNAL CONSTANTS (TM-2026-028 §2, TM-2026-017 §17)
// ══════════════════════════════════════════════════════════════

/// Idle state numerator: α = R₆/Δ = 91/36. Numerator = λ_EUV.
pub const ALPHA_NUM: u32 = LAMBDA_EUV;

/// Idle state denominator: Δ/R₂.
pub const ALPHA_DEN: u32 = DISCRIMINANT / REPUNIT_2;

/// Dispatch state numerator: β = R₆/√Δ = 91/3. Numerator = λ_EUV.
pub const BETA_NUM: u32 = LAMBDA_EUV;

/// Dispatch state denominator = TERNARY_BASE.
pub const BETA_DEN: u32 = TERNARY_BASE;

/// Transition magnitude numerator: γ = β − α = LCM_PRIMARY / 36.
pub const GAMMA_NUM: u32 = LCM_PRIMARY;

/// Transition magnitude denominator — same as ALPHA_DEN.
pub const GAMMA_DEN: u32 = ALPHA_DEN;

/// Dispatch-to-idle time ratio = 1/TERNARY_BASE.
pub const DISPATCH_RATIO_NUM: u32 = 1;

/// Dispatch-to-idle time ratio denominator.
pub const DISPATCH_RATIO_DEN: u32 = TERNARY_BASE;

/// Duty cycle = 1/R₂.
pub const DUTY_NUM: u32 = 1;

/// Duty cycle denominator.
pub const DUTY_DEN: u32 = REPUNIT_2;

/// DC component numerator: ⟨H⟩ = α + γd = (R₆ + LCM_PRIMARY) / TERNARY_BASE.
pub const DC_NUM: u32 = (REPUNIT_6 + LCM_PRIMARY) / TERNARY_BASE;

/// DC component denominator: Δ / TERNARY_BASE.
pub const DC_DEN: u32 = DISCRIMINANT / TERNARY_BASE;

/// AC power numerator: P_AC = γ²·d(1−d). LCM_PRIMARY² × TERNARY_BASE.
pub const AC_POWER_NUM: u32 = LCM_PRIMARY * LCM_PRIMARY * TERNARY_BASE;

/// AC power denominator: ALPHA_DEN² × DUTY_DEN².
pub const AC_POWER_DEN: u32 = ALPHA_DEN * ALPHA_DEN * DUTY_DEN * DUTY_DEN;

/// HModal trit mapping — low (α state, 75% dwell).
pub const HMODAL_TRIT_LOW: i8 = -1;

/// HModal trit mapping — mid (transition, zero-crossing).
pub const HMODAL_TRIT_MID: i8 = 0;

/// Transition midpoint signal level numerator: (α+β)/2 = λ_EUV × R₃ / (2 × ALPHA_DEN).
pub const HMODAL_TRIT_MID_NUM: u32 = LAMBDA_EUV * REPUNIT_3;

/// Transition midpoint signal level denominator.
pub const HMODAL_TRIT_MID_DEN: u32 = 2 * ALPHA_DEN;

/// HModal trit mapping — high (β state, 25% dwell).
pub const HMODAL_TRIT_HIGH: i8 = 1;

// ══════════════════════════════════════════════════════════════
// §11  CHANNEL ARCHITECTURE (TM-2026-028 §3–§4)
// ══════════════════════════════════════════════════════════════

/// Null-channel modulus = R₂ = 4.
pub const NULL_CHANNEL_MOD: u32 = REPUNIT_2;

/// Period of |sin(πn/4)| = 2 × R₂.
pub const SIN_PERIOD: u32 = 2 * REPUNIT_2;

/// Phase step = 1/R₂.
pub const PHASE_STEP_NUM: u32 = 1;
pub const PHASE_STEP_DEN: u32 = REPUNIT_2;

// ══════════════════════════════════════════════════════════════
// §12  POLYGON GEOMETRY (TM-2026-017 §4–§5)
// ══════════════════════════════════════════════════════════════

/// Number of regular n-gons for n = 3..15 = R₃.
pub const POLYGON_COUNT: u32 = REPUNIT_3;

/// Central angle generating function: θ_n = R₆/n.
pub const fn central_angle(n: u32) -> (u32, u32) {
    (REPUNIT_6, n)
}

/// θ₃ = R₆/3.
pub const CENTRAL_ANGLE_TRIANGLE: (u32, u32) = (REPUNIT_6, P3);

/// θ₄ = R₆/R₂ = λ_EUV (exact integer).
pub const CENTRAL_ANGLE_SQUARE: u32 = REPUNIT_6 / REPUNIT_2;

/// θ₅ = R₆/5.
pub const CENTRAL_ANGLE_PENTAGON: (u32, u32) = (REPUNIT_6, P5);

/// θ₆ = R₆/6.
pub const CENTRAL_ANGLE_HEXAGON: (u32, u32) = (REPUNIT_6, 2 * P3);

/// θ₇ = R₆/7 (exact integer).
pub const CENTRAL_ANGLE_HEPTAGON: u32 = REPUNIT_6 / P7;

/// θ₈ = R₆/8.
pub const CENTRAL_ANGLE_OCTAGON: (u32, u32) = (REPUNIT_6, P8);

/// θ₉ = R₆/9.
pub const CENTRAL_ANGLE_ENNEAGON: (u32, u32) = (REPUNIT_6, P9);

/// θ₁₀ = R₆/10.
pub const CENTRAL_ANGLE_DECAGON: (u32, u32) = (REPUNIT_6, 2 * P5);

/// θ₁₁ = R₆/11.
pub const CENTRAL_ANGLE_HENDECAGON: (u32, u32) = (REPUNIT_6, P11);

/// θ₁₂ = R₆/12.
pub const CENTRAL_ANGLE_DODECAGON: (u32, u32) = (REPUNIT_6, 4 * P3);

/// θ₁₃ = R₆/R₃ = Z₂₈ order (exact integer).
pub const CENTRAL_ANGLE_TRIDECAGON: u32 = REPUNIT_6 / REPUNIT_3;

/// θ₁₄ = R₆/x₁ = x₂ (exact integer).
pub const CENTRAL_ANGLE_TETRADECAGON: u32 = ROOT_X2;

/// θ₁₅ = R₆/15.
pub const CENTRAL_ANGLE_PENTADECAGON: (u32, u32) = (REPUNIT_6, P15);

/// Bézier C₁₈₂ angle = λ_EUV.
pub const BEZIER_C182_ANGLE: u32 = LAMBDA_EUV;

/// Bézier C₆₅₀ angle = P11 × P13.
pub const BEZIER_C650_ANGLE: u32 = P11 * P13;

/// C₁₈₂ in custom radians = P7.
pub const BEZIER_C182_RADIANS: u32 = P7;

/// C₆₅₀ in custom radians = P11.
pub const BEZIER_C650_RADIANS: u32 = P11;

/// Arc convergence: 3 × θ₅ = 3 × R₆/5 = 1092/5.
pub const ARC_CONVERGENCE_NUM: u32 = 3 * REPUNIT_6;

/// Arc convergence denominator.
pub const ARC_CONVERGENCE_DEN: u32 = P5;

/// Rim vertices in the inscribed polygon overlay = 2 × Z₂₈ + 2.
pub const RIM_VERTICES: u32 = 2 * CENTRAL_ANGLE_TRIDECAGON + 2;

/// Interior intersections.
pub const INTERIOR_INTERSECTIONS: u32 = TOTAL_NODES - RIM_VERTICES;

/// Total nodes = 504 = R₆ + Δ = 364 + 140. Verify in const assertion.
pub const TOTAL_NODES: u32 = 504;

// ══════════════════════════════════════════════════════════════
// §13  SUPERHUB ZONES — integer data (TM-2026-017 §11.2–§11.5)
// ══════════════════════════════════════════════════════════════

/// Polygon membership for zones A & B: 7, 11, 12, 13.
/// 11, 12, 13 appear in ALL four zones; 4th switches between 7 (A/B) and 8 (C/D).
pub const SUPERHUB_AB_POLYGONS: [u32; 4] = [P7, P11, 12, P13];

/// Polygon membership for zones C & D: 8, 11, 12, 13.
pub const SUPERHUB_CD_POLYGONS: [u32; 4] = [P8, P11, 12, P13];

// ══════════════════════════════════════════════════════════════
// §14  TRIANGULAR NUMBER ANCHORS (TM-2026-017 §9)
// ══════════════════════════════════════════════════════════════

/// Tri(3) = P3 × (P3+1) / 2.
pub const TRI_3: u32 = P3 * (P3 + 1) / 2;

/// Tri(7) = P7 × (P7+1) / 2 = Z₂₈.
pub const TRI_7: u32 = P7 * (P7 + 1) / 2;

/// Tri(10) = 10 × 11 / 2.
pub const TRI_10: u32 = (2 * P5) * (P11) / 2;

/// Tri(R₃) = R₃ × (R₃+1) / 2 = λ_EUV.
pub const TRI_13: u32 = LAMBDA_EUV;

// ══════════════════════════════════════════════════════════════
// §15  TORUS KNOT PARAMETERS (TM-2026-017 §10.6–§10.8)
// ══════════════════════════════════════════════════════════════

/// Crossing number (11,14) = P11 × P13. Minimum crossings.
pub const CROSSING_11_14: u32 = P11 * P13;

/// Crossing number (13,14) = R₃² (radian squared).
pub const CROSSING_13_14: u32 = REPUNIT_3 * REPUNIT_3;

/// Crossing number (13,15) = R₃ × x₁ = ARC_ROOT_SEMI.
pub const CROSSING_13_15: u32 = ARC_ROOT_SEMI;

/// Crossing number (14,15) = x₁² (π squared).
pub const CROSSING_14_15: u32 = ROOT_X1 * ROOT_X1;

// ══════════════════════════════════════════════════════════════
// §17  PLENUM SQUARE SCALING (Lo Shu × 22 + CENTER)
// ══════════════════════════════════════════════════════════════

/// Lo Shu scaling factor = 2 × P11.
pub const PLENUM_SQUARE_STEP: u32 = 2 * P11;

/// Smallest magic square entry = CENTER − R₂ × STEP.
pub const PLENUM_SQUARE_MIN: u32 = CENTER - REPUNIT_2 * PLENUM_SQUARE_STEP;

// ══════════════════════════════════════════════════════════════
// §19a  FIBONACCI–PHYSICAL BRIDGE — integer core
//
// F(14) = 377 is the sole new integer constant in this section.
// It connects the Fibonacci sequence to the framework through
// π_framework = 14, and predicts the impedance of free space
// to 0.072% (validated against CODATA 2022).
//
// Integer identities (compile-time verified):
//   F(14) = R₆ + R₃          = 364 + 13      (circle + radian)
//   F(14) = R₃ × (Z₂₈ + 1)  = 13 × 29       (radian × shifted cyclic order)
//   F(14) = F(13) + F(12)    = 233 + Δ        (Fibonacci recurrence, F(12) = Δ = 144)
//   F(7)  = R₃               = 13             (radian appears at Fibonacci index 7)
// ══════════════════════════════════════════════════════════════

/// F(π) = F(14) = R₆ + R₃ = circle + radian.
pub const FIBONACCI_PI: u32 = REPUNIT_6 + REPUNIT_3;

/// F(12) = Δ (circle quadratic discriminant).
pub const FIBONACCI_12: u32 = DISCRIMINANT;

/// F(13) = F(14) − F(12) = (R₆ + R₃) − Δ.
pub const FIBONACCI_13: u32 = FIBONACCI_PI - FIBONACCI_12;

// ══════════════════════════════════════════════════════════════
// Z₂₈ CYCLIC GROUP
// ══════════════════════════════════════════════════════════════

/// Order of Z₂₈ = R₆/R₃ = θ₁₃ = 2π.
pub const CYCLIC_ORDER: u32 = CENTRAL_ANGLE_TRIDECAGON;

/// Radians per circle = Z₂₈ order.
pub const RADIANS_PER_CIRCLE: u32 = CYCLIC_ORDER;

/// Dimensions = Z₂₈ order.
pub const Z28_DIMENSIONS: u32 = CYCLIC_ORDER;

/// Generator of Z₂₈.
pub const Z28_GENERATOR: u32 = 1;

/// Co-generator = R₃. gcd(R₃, Z₂₈) = 1.
pub const Z28_CO_GENERATOR: u32 = REPUNIT_3;

// ══════════════════════════════════════════════════════════════
// TRIBONACCI CONSTANTS
// ══════════════════════════════════════════════════════════════

/// The Tribonacci constant τ ≈ 1.839286755214161.
///
/// Real root of x³ = x² + x + 1. The native irrational of ternary
/// recursion, analogous to φ in binary/Fibonacci systems.
pub const TAU_TRIBONACCI: f64 = 1.839286755214161;

/// τ² ≈ 3.38297576790891.
pub const TAU_SQUARED: f64 = 3.38297576790891;

/// τ³ = τ² + τ + 1 ≈ 6.22226252312307.
pub const TAU_CUBED: f64 = 6.22226252312307;

/// Tribonacci golden angle (native ternary): 364° / τ³ ≈ 58.50°.
pub const TRIBONACCI_GOLDEN_ANGLE_DEG: f64 = 58.50438656;

/// Classical golden angle translated into the ternary circle:
/// 364° / φ² where φ = (1+√5)/2.
pub const GOLDEN_ANGLE_TERNARY_DEG: f64 = 138.98056;

/// Maximum number of trits supported for Tribonacci computations.
pub const MAX_TRITS: usize = 128;

// ══════════════════════════════════════════════════════════════
// WALK INSTRUCTION SET ON Z₂₈
// ══════════════════════════════════════════════════════════════

/// Walk instruction for trit digit 0: **0 radians** (no turn, step forward).
pub const WALK_TURN_0: f64 = 0.0;

/// Walk instruction for trit digit 1: **1 ternary radian = 13°**.
pub const WALK_TURN_1: f64 = 13.0;

/// Walk instruction for trit digit 2: **2 ternary radians = 26°**.
pub const WALK_TURN_2: f64 = 26.0;

// ══════════════════════════════════════════════════════════════
// CLIFFORD ALGEBRA DIMENSIONAL CONSTANTS
// ══════════════════════════════════════════════════════════════

/// Number of basis vectors in Cl(3,0)/GF(3).
pub const CLIFFORD_GENERATORS: usize = 3;

/// Total dimension of the algebra: 2^3 = 8 components.
pub const CLIFFORD_DIM: usize = 8;

/// Number of even-grade basis elements (scalar + bivectors = 4).
pub const CLIFFORD_EVEN_DIM: usize = 4;

// ══════════════════════════════════════════════════════════════
// TORUS TOPOLOGY CONSTANTS
// ══════════════════════════════════════════════════════════════

/// The ternary radix for the torus k-ary n-cube (k=3).
pub const TORUS_RADIX: u32 = 3;

/// The number of neighbors per dimension on the torus (forward + backward).
pub const TORUS_NEIGHBORS_PER_DIM: usize = 2;

// ══════════════════════════════════════════════════════════════
// BORROMEAN TOPOLOGY CONSTANTS
// ══════════════════════════════════════════════════════════════

/// Number of rings in a Borromean link.
pub const BORROMEAN_RING_COUNT: usize = 3;

/// The modulus for the Borromean XOR invariant (Z/3Z).
pub const BORROMEAN_MODULUS: u8 = 3;

// ══════════════════════════════════════════════════════════════
// FRAMEWORK IDENTITY
// ══════════════════════════════════════════════════════════════

/// The Salvi Framework identifier.
pub const FRAMEWORK: &str = "Salvi Framework";

/// The division responsible for this codebase.
pub const DIVISION: &str = "Applied Physics Division";

/// The organization.
pub const ORG: &str = "Capomastro Holdings Ltd.";

// ══════════════════════════════════════════════════════════════
// CONVERSION SECTION — f64 values (NOT part of the mathematical core)
//
// These pre-date the integer formalization and persist as the
// interface between the integer framework and standard trig.
// ══════════════════════════════════════════════════════════════

/// Full circle in the ternary angular system: **364 degrees** (f64 form).
pub const FULL_CIRCLE_DEG: f64 = 364.0;

/// Full circle as a base-3 repunit string.
pub const FULL_CIRCLE_BASE3: &str = "111111";

/// π in the ternary circle system: **exactly 14** (f64 form).
pub const PI_TERNARY: f64 = 14.0;

/// 2π in the ternary system: **28** (f64 form).
pub const TWO_PI_TERNARY: f64 = 28.0;

/// One ternary radian in degrees: **exactly 13°** (f64 form).
pub const RADIAN_DEG: f64 = 13.0;

/// One ternary radian as a base-3 repunit string.
pub const RADIAN_BASE3: &str = "111";

// ══════════════════════════════════════════════════════════════
// §19b  CODATA 2022 PHYSICAL CONSTANTS BRIDGE
//
// These f64 values are NOT part of the mathematical core. They
// provide NIST/CODATA reference measurements that the framework's
// algebraic constants predict, enabling automated validation.
//
// Two Rydberg constants are relevant:
//   R_∞ = 10,973,731.568157 m⁻¹  (infinite nuclear mass)
//   R_H = R_∞ / (1 + mₑ/mₚ)     (hydrogen-specific, finite proton)
//
// Balmer's constant B and the Lyman limit 1/R_∞ both use R_∞.
// The vacuum bias (VACUUM_BIAS_NUM/DEN) uses R_H.
//
// Sources: CODATA 2022, NIST SP 961 (May 2024).
// ══════════════════════════════════════════════════════════════

/// Rydberg constant R_∞ (CODATA 2022), infinite nuclear mass.
pub const CODATA_RYDBERG_CONST: f64 = 10_973_731.568157; // m⁻¹

/// Electron-to-proton mass ratio mₑ/mₚ (CODATA 2022).
pub const CODATA_ME_OVER_MP: f64 = 5.446_170_214_889e-4;

/// Balmer's constant B = 4/R_∞ (CODATA 2022).
/// Framework prediction: QUAD_PRODUCT = R₆ = 364 → 364 nm.
/// Discrepancy: 0.139%.
pub const CODATA_BALMER_CONSTANT_NM: f64 = 364.506_82;

/// Inverse Rydberg constant 1/R_∞ = Lyman series limit (CODATA 2022).
/// Framework prediction: LAMBDA_EUV = R₆/4 = 91 → 91 nm.
/// Discrepancy: 0.139%.
pub const CODATA_LYMAN_LIMIT_NM: f64 = 91.126_705;

/// Lyman-alpha wavelength (n=2 → n=1 hydrogen transition, vacuum).
/// Framework prediction: QUAD_PRODUCT / 3 = R₆/3 ≈ 121.333 nm.
/// Discrepancy: 0.192%.
pub const CODATA_LYMAN_ALPHA_NM: f64 = 121.567_0;

/// H-alpha wavelength (n=3 → n=2 hydrogen transition, in air).
/// Framework prediction: ARC_ROOT_COMP = 650 → 650 nm.
/// Discrepancy: 0.957%. Residual ≈ 2π_conventional.
pub const CODATA_H_ALPHA_NM: f64 = 656.281;

/// Impedance of free space Z₀ = √(μ₀/ε₀) = μ₀c (CODATA 2022).
/// Framework prediction: FIBONACCI_PI = F(14) = R₆ + R₃ = 377 Ω.
/// Discrepancy: 0.072%.
pub const CODATA_Z0_OHM: f64 = 376.730_313_412;

/// Hartree energy in eV (CODATA 2022).
/// Framework prediction: DISCRIMINANT_2_SQRT = √Δ₂ = 27 → 27 eV.
/// Discrepancy: 0.777%.
pub const CODATA_HARTREE_EV: f64 = 27.211_386_245_981;

/// Rydberg energy in eV = R_∞ hc (CODATA 2022).
/// Framework prediction: REPUNIT_3 = R₃ = 13 → 13 eV.
/// Discrepancy: 4.45%.
pub const CODATA_RYDBERG_EV: f64 = 13.605_693_122_990;

/// Inverse fine-structure constant (CODATA 2022).
/// Framework proximity: BEZIER_C650_ANGLE = 11 × 13 = 143 ≈ 1/α + 6.
/// Tier 3 — suggestive, not predictive. No tolerance enforced.
pub const CODATA_INV_ALPHA: f64 = 137.035_999_177;

/// H-alpha residual: CODATA_H_ALPHA_NM − ARC_ROOT_COMP = 656.281 − 650 ≈ 6.28 ≈ 2π_conv.
pub const H_ALPHA_RESIDUAL: f64 = CODATA_H_ALPHA_NM - ARC_ROOT_COMP as f64;

/// §19c  TOLERANCE BOUNDS for CODATA validation tests.
///
/// Tier 1: spectral wavelengths derived from R₆ (< 0.2%).
pub const CODATA_TIER_1_TOLERANCE: f64 = 0.002;

/// Tier 2: energy levels and impedance (< 1%).
pub const CODATA_TIER_2_TOLERANCE: f64 = 0.01;

/// Tier 2b: Rydberg energy vs radian (< 5%).
pub const CODATA_TIER_2B_TOLERANCE: f64 = 0.05;

// ══════════════════════════════════════════════════════════════
// §13  SUPERHUB ZONE COORDINATES (approximate f64 — NOT mathematical core)
//
// Approximate intersection points of polygon edges on the unit circle,
// given to 4 decimal places (TM-2026-017 §11.2–§11.5, R2-A5-2).
// These are NOT exact rational numbers. Mirror symmetry is enforced
// by construction (shared X components, negated Y components).
// ══════════════════════════════════════════════════════════════

/// Superhub X coordinate (left pair, zones A & B).
pub const SUPERHUB_X_LEFT: f64 = -0.9005;

/// Superhub X coordinate (right pair, zones C & D).
pub const SUPERHUB_X_RIGHT: f64 = 0.7400;

/// Superhub Y magnitude (zones A & B).
pub const SUPERHUB_Y_AB: f64 = 0.3720;

/// Superhub Y magnitude (zones C & D).
pub const SUPERHUB_Y_CD: f64 = 0.6270;

/// Zone A: (SUPERHUB_X_LEFT, SUPERHUB_Y_AB) — 159.3° custom.
pub const SUPERHUB_A: (f64, f64) = (SUPERHUB_X_LEFT, SUPERHUB_Y_AB);

/// Zone B: (SUPERHUB_X_LEFT, -SUPERHUB_Y_AB) — 204.7° custom.
pub const SUPERHUB_B: (f64, f64) = (SUPERHUB_X_LEFT, -SUPERHUB_Y_AB);

/// Zone C: (SUPERHUB_X_RIGHT, SUPERHUB_Y_CD) — 40.8° custom.
pub const SUPERHUB_C: (f64, f64) = (SUPERHUB_X_RIGHT, SUPERHUB_Y_CD);

/// Zone D: (SUPERHUB_X_RIGHT, -SUPERHUB_Y_CD) — 323.2° custom.
pub const SUPERHUB_D: (f64, f64) = (SUPERHUB_X_RIGHT, -SUPERHUB_Y_CD);

// ══════════════════════════════════════════════════════════════
// §16  BRIDGE TO STANDARD TRIG (TM-2026-017 §1.1)
// ══════════════════════════════════════════════════════════════

/// Bridge coefficient: π_std / ROOT_X1 = π_std / 14.
/// Maps the integer system onto the transcendental one.
/// Sin₃₆₄(ρ) = sin_std(BRIDGE_COEFF × ρ).
/// Standard π is the conversion constant, not the fundamental constant.
pub const BRIDGE_COEFF: f64 = STD_PI / 14.0;

// ══════════════════════════════════════════════════════════════
// CONVERSION UTILITIES (pure functions of the constants above)
// ══════════════════════════════════════════════════════════════

/// Convert ternary degrees (364° full circle) to conventional degrees (360°).
#[inline]
pub fn ternary_deg_to_std_deg(ternary_deg: f64) -> f64 {
    ternary_deg * (360.0 / FULL_CIRCLE_DEG)
}

/// Convert conventional degrees (360° full circle) to ternary degrees (364°).
#[inline]
pub fn std_deg_to_ternary_deg(std_deg: f64) -> f64 {
    std_deg * (FULL_CIRCLE_DEG / 360.0)
}

/// Convert ternary radians (28 per circle) to standard radians (2π per circle).
#[inline]
pub fn ternary_rad_to_std_rad(ternary_rad: f64) -> f64 {
    ternary_rad * (2.0 * STD_PI / TWO_PI_TERNARY)
}

/// Convert standard radians to ternary radians.
#[inline]
pub fn std_rad_to_ternary_rad(std_rad: f64) -> f64 {
    std_rad * (TWO_PI_TERNARY / (2.0 * STD_PI))
}

/// Convert ternary degrees to ternary radians.
#[inline]
pub fn ternary_deg_to_ternary_rad(deg: f64) -> f64 {
    deg / RADIAN_DEG
}

/// Convert ternary radians to ternary degrees.
#[inline]
pub fn ternary_rad_to_ternary_deg(rad: f64) -> f64 {
    rad * RADIAN_DEG
}

/// Convert a trit digit (0, 1, or 2) to its walk angle in ternary degrees.
#[inline]
pub fn trit_to_walk_angle_deg(trit: u8) -> f64 {
    debug_assert!(trit <= 2, "Trit must be 0, 1, or 2; got {}", trit);
    trit as f64 * RADIAN_DEG
}

/// Convert a trit digit to its walk angle in standard radians.
#[inline]
pub fn trit_to_std_rad(trit: u8) -> f64 {
    ternary_rad_to_std_rad(trit as f64)
}


// ══════════════════════════════════════════════════════════════
// TIER 1 — COMPILE-TIME CONST ASSERTIONS
// Pure integer arithmetic. Fails the build if wrong.
// ══════════════════════════════════════════════════════════════

const _: () = {
    // §1 Repunit family
    assert!(TERNARY_BASE == TORUS_RADIX);
    assert!(REPUNIT_1 == repunit(1));
    assert!(REPUNIT_2 == repunit(2));
    assert!(REPUNIT_3 == repunit(3));
    assert!(REPUNIT_4 == repunit(4));
    assert!(REPUNIT_5 == repunit(5));
    assert!(REPUNIT_6 == repunit(6));
    assert!(REPUNIT_2 == TERNARY_BASE * REPUNIT_1 + 1);
    assert!(REPUNIT_3 == TERNARY_BASE * REPUNIT_2 + 1);
    assert!(REPUNIT_4 == TERNARY_BASE * REPUNIT_3 + 1);
    assert!(REPUNIT_5 == TERNARY_BASE * REPUNIT_4 + 1);
    assert!(REPUNIT_6 == TERNARY_BASE * REPUNIT_5 + 1);
    assert!(REPUNIT_3 as f64 == RADIAN_DEG);
    assert!(REPUNIT_6 as f64 == FULL_CIRCLE_DEG);

    // §2 Circle quadratic
    assert!(QUAD_SUM == REPUNIT_4);
    assert!(QUAD_PRODUCT == REPUNIT_6);
    assert!(DISCRIMINANT == QUAD_SUM * QUAD_SUM - 4 * QUAD_PRODUCT);
    assert!(DISCRIMINANT_SQRT * DISCRIMINANT_SQRT == DISCRIMINANT);
    assert!(ROOT_X1 + ROOT_X2 == QUAD_SUM);
    assert!(ROOT_X1 * ROOT_X2 == QUAD_PRODUCT);
    assert!(ROOT_X2 - ROOT_X1 == DISCRIMINANT_SQRT);
    assert!(ROOT_X1 == (QUAD_SUM - DISCRIMINANT_SQRT) / 2);
    assert!(ROOT_X2 == (QUAD_SUM + DISCRIMINANT_SQRT) / 2);
    assert!(ROOT_X1 * ROOT_X1 + QUAD_PRODUCT == QUAD_SUM * ROOT_X1);
    assert!(ROOT_X2 * ROOT_X2 + QUAD_PRODUCT == QUAD_SUM * ROOT_X2);
    assert!(ROOT_X1 as f64 == PI_TERNARY);

    // §3 Unified equation
    assert!(UNIFIED_LINEAR == QUAD_SUM * (QUAD_SUM - 1) - 2 * QUAD_PRODUCT);
    assert!(UNIFIED_CONSTANT == QUAD_PRODUCT * UNIFIED_FACTOR);
    assert!(UNIFIED_FACTOR == QUAD_PRODUCT - QUAD_SUM + 1);
    assert!(UNIFIED_DISC == UNIFIED_LINEAR * UNIFIED_LINEAR - 4 * UNIFIED_CONSTANT);
    assert!(UNIFIED_DISC_SQRT * UNIFIED_DISC_SQRT == UNIFIED_DISC);
    assert!(ARC_ROOT_SEMI == (UNIFIED_LINEAR - UNIFIED_DISC_SQRT) / 2);
    assert!(ARC_ROOT_COMP == (UNIFIED_LINEAR + UNIFIED_DISC_SQRT) / 2);
    assert!(ARC_ROOT_SEMI + ARC_ROOT_COMP == UNIFIED_LINEAR);
    assert!((ARC_ROOT_SEMI as u64) * (ARC_ROOT_COMP as u64) == UNIFIED_CONSTANT as u64);
    assert!((ARC_ROOT_SEMI as u64) * (ARC_ROOT_SEMI as u64)
        + (UNIFIED_CONSTANT as u64)
        == (UNIFIED_LINEAR as u64) * (ARC_ROOT_SEMI as u64));
    assert!((ARC_ROOT_COMP as u64) * (ARC_ROOT_COMP as u64)
        + (UNIFIED_CONSTANT as u64)
        == (UNIFIED_LINEAR as u64) * (ARC_ROOT_COMP as u64));
    assert!(GREEN_ARC_EFF == ARC_ROOT_COMP - QUAD_PRODUCT);
    assert!(ARC_ROOT_SEMI == QUAD_PRODUCT / 2);
    assert!(ARC_ROOT_SEMI == ROOT_X1 * (ROOT_X1 - 1));
    assert!(ARC_ROOT_SEMI == ROOT_X1 * REPUNIT_3);
    assert!(CENTER == (ARC_ROOT_SEMI + QUAD_SUM) / 2);
    assert!(MAGIC_CONSTANT == 3 * CENTER);
    assert!(CIRCUMFERENCE == ROOT_X1 * CENTER);
    assert!(CIRCUMFERENCE == CYCLIC_ORDER * RADIUS_NUM / RADIUS_DEN);
    assert!(DISCRIMINANT_2 == 1 + 4 * ARC_ROOT_SEMI);
    assert!(DISCRIMINANT_2 == TERNARY_BASE.pow(6));
    assert!(DISCRIMINANT_2_SQRT * DISCRIMINANT_2_SQRT == DISCRIMINANT_2);
    assert!(ROOT_X1 == (1 + DISCRIMINANT_2_SQRT) / 2);

    // §4 Squared circle — with coprime ties (ticket #99 Task 5)
    assert!(UNIT_CIRCLE_AREA == ROOT_X1);
    assert!(RADIAN_CIRCLE_AREA == ROOT_X1 * REPUNIT_3);
    assert!(RADIAN_CIRCLE_AREA == ARC_ROOT_SEMI);
    assert!(SQUARED_SIDE_SQ_RADIAN == SQUARED_SIDE_SQ_UNIT * REPUNIT_3);
    assert!(UNIT_CIRCLE_AREA == 2 * COPRIME_TRIPLES[0][0]);
    assert!(RADIAN_CIRCLE_AREA == COPRIME_PAIR_LCMS[6]);
    assert!(SQUARED_SIDE_SQ_RADIAN == COPRIME_PAIR_LCMS[6]);

    // §5 Angular conversion
    assert!(ANGULAR_CONV_NUM * STD_CIRCLE_DEG == ANGULAR_CONV_DEN * QUAD_PRODUCT);
    assert!(ANGULAR_CONV_NUM == LAMBDA_EUV);

    // §6 UV spectral wavelengths — with coprime pair ties (ticket #99 Task 5)
    assert!(LAMBDA_EUV % REPUNIT_3 == 0);
    assert!(LAMBDA_UVC % REPUNIT_3 == 0);
    assert!(LAMBDA_UVB % REPUNIT_3 == 0);
    assert!(LAMBDA_UVA % REPUNIT_3 == 0);
    assert!(LAMBDA_EUV == 7 * 13);
    assert!(LAMBDA_UVC == 14 * 13);
    assert!(LAMBDA_UVB == 22 * 13);
    assert!(LAMBDA_UVA == 28 * 13);
    assert!(LAMBDA_EUV / REPUNIT_3 == 7);
    assert!(LAMBDA_UVC / REPUNIT_3 == 14);
    assert!(LAMBDA_UVB / REPUNIT_3 == 22);
    assert!(LAMBDA_UVA / REPUNIT_3 == 28);
    assert!(LAMBDA_UVC / LAMBDA_EUV == 2);
    assert!(LAMBDA_UVB * 7 == LAMBDA_EUV * 22);
    assert!(LAMBDA_UVA / LAMBDA_EUV == 4);
    assert!(LAMBDA_UVB * 7 == LAMBDA_UVC * 11);
    assert!(LAMBDA_UVA * 11 == LAMBDA_UVB * 14);
    assert!(LAMBDA_UVC == 2 * LAMBDA_EUV);
    assert!(LAMBDA_UVA == 4 * LAMBDA_EUV);
    assert!(LAMBDA_UVA == 2 * LAMBDA_UVC);
    assert!(LAMBDA_FAR_UVC == 2 * CENTER);
    assert!(LAMBDA_EXCIMER == NULL_CHANNEL_MOD * COPRIME_PAIR_LCMS[0]);
    assert!(LAMBDA_NB_UVB == COPRIME_PAIR_LCMS[0] + COPRIME_PAIR_LCMS[1] + COPRIME_PAIR_LCMS[3]);
    // NB-UVB = 311 is prime: trial division up to √311 ≈ 17.6
    assert!(311 % 2 != 0 && 311 % 3 != 0 && 311 % 5 != 0
        && 311 % 7 != 0 && 311 % 11 != 0 && 311 % 13 != 0 && 311 % 17 != 0);
    assert!(BOUNDARY_UVB_UVA == UNIFIED_FACTOR);
    assert!(LAMBDA_UVA == QUAD_PRODUCT);
    assert!(LAMBDA_UVC == ARC_ROOT_SEMI);
    assert!(GREEN_ARC_EFF == LAMBDA_UVB);
    assert!(LAMBDA_UVB == 2 * COPRIME_PAIR_LCMS[3]);
    assert!(LAMBDA_UVA == 2 * COPRIME_PAIR_LCMS[6]);
    assert!(BOUNDARY_UVC_UVB == (LAMBDA_UVC + LAMBDA_UVB) / 2);
    assert!(BOUNDARY_UVB_UVA == (LAMBDA_UVB + LAMBDA_UVA) / 2);

    // §7 Coprime walk — arc factorizations via pair LCMs
    assert!(ARC_ROOT_SEMI == 2 * COPRIME_PAIR_LCMS[1]);
    assert!(GREEN_ARC_EFF == 2 * COPRIME_PAIR_LCMS[3]);
    assert!(LCM_PRIMARY == COPRIME_TRIPLES[0][0] * COPRIME_TRIPLES[0][1] * COPRIME_TRIPLES[0][2]);
    // §7 Coprime walk — pairs
    assert!(COPRIME_PAIR_LCMS[0] == 7 * 11);
    assert!(COPRIME_PAIR_LCMS[1] == 7 * 13);
    assert!(COPRIME_PAIR_LCMS[2] == 7 * 15);
    assert!(COPRIME_PAIR_LCMS[3] == 11 * 13);
    assert!(COPRIME_PAIR_LCMS[4] == 11 * 14);
    assert!(COPRIME_PAIR_LCMS[5] == 11 * 15);
    assert!(COPRIME_PAIR_LCMS[6] == 13 * 14);
    assert!(COPRIME_PAIR_LCMS[7] == 13 * 15);
    assert!(COPRIME_PAIR_LCMS[8] == 14 * 15);
    assert!(COPRIME_PAIR_LCMS[1] == LAMBDA_EUV);
    assert!(COPRIME_PAIR_LCMS[3] == BEZIER_C650_ANGLE);
    assert!(COPRIME_PAIR_LCMS[6] == ARC_ROOT_SEMI);
    // §7 Coprime walk — triples
    assert!(COPRIME_TRIPLE_LCMS[0] == 7 * 11 * 13);
    assert!(COPRIME_TRIPLE_LCMS[0] == LCM_PRIMARY);
    assert!(COPRIME_TRIPLE_LCMS[1] == 7 * 11 * 15);
    assert!(COPRIME_TRIPLE_LCMS[2] == 7 * 13 * 15);
    assert!(COPRIME_TRIPLE_LCMS[3] == 11 * 13 * 14);
    assert!(COPRIME_TRIPLE_LCMS[3] == 2 * LCM_PRIMARY);
    assert!(COPRIME_TRIPLE_LCMS[4] == 11 * 13 * 15);
    assert!(COPRIME_TRIPLE_LCMS[5] == 11 * 14 * 15);
    assert!(COPRIME_TRIPLE_LCMS[6] == 13 * 14 * 15);
    // §7 Coprime walk — quadruples
    assert!(COPRIME_QUADRUPLE_LCMS[0] == 3 * 5 * 7 * 11 * 13);
    assert!(COPRIME_QUADRUPLE_LCMS[0] == PENTADECAGON * LCM_PRIMARY);
    assert!(COPRIME_QUADRUPLE_LCMS[1] == 2 * COPRIME_QUADRUPLE_LCMS[0]);
    // §7 Coprime walk — quintuples
    assert!(COPRIME_QUINTUPLE_LCMS[0] == 3 * 5 * 7 * 11 * 13);
    assert!(COPRIME_QUINTUPLE_LCMS[0] == COPRIME_QUADRUPLE_LCMS[0]);
    assert!(COPRIME_QUINTUPLE_LCMS[1] == 3 * 5 * 11 * 13 * 14);
    assert!(COPRIME_QUINTUPLE_LCMS[1] == COPRIME_QUADRUPLE_LCMS[1]);
    assert!(COPRIME_QUINTUPLE_LCMS[2] == 3 * 4 * 7 * 11 * 13);
    assert!(COPRIME_QUINTUPLE_LCMS[3] == 5 * 7 * 8 * 11 * 13);
    assert!(COPRIME_QUINTUPLE_LCMS[4] == 5 * 7 * 9 * 11 * 13);
    // §7 Coprime walk — sextuples
    assert!(COPRIME_SEXTUPLE_LCMS[0] == 3 * 4 * 5 * 7 * 11 * 13);
    assert!(COPRIME_SEXTUPLE_LCMS[1] == 3 * 5 * 7 * 8 * 11 * 13);
    assert!(COPRIME_SEXTUPLE_LCMS[2] == 4 * 5 * 7 * 9 * 11 * 13);
    assert!(COPRIME_SEXTUPLE_LCMS[3] == 5 * 7 * 8 * 9 * 11 * 13);
    assert!(COPRIME_SEXTUPLE_LCMS[3] == LCM_SEXT_MAX);
    assert!(LCM_SEXT_MAX == STD_CIRCLE_DEG * LCM_PRIMARY);
    assert!(LCM_SEXT_MAX == 24 * COPRIME_QUADRUPLE_LCMS[0]);
    // §7 Coprime walk — geometric-spectral product
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == QUAD_PRODUCT * LCM_PRIMARY);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == 364 * 1000 + 364);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == 4 * 7 * 7 * 11 * 13 * 13);
    // §7 Coprime walk — 3D products
    assert!(POS_3D_PRIMARY == (LCM_PRIMARY as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_QUADRUPLES[0] == (COPRIME_QUADRUPLE_LCMS[0] as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_QUADRUPLES[1] == (COPRIME_QUADRUPLE_LCMS[1] as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_SEXTUPLES[0] == (COPRIME_SEXTUPLE_LCMS[0] as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_SEXTUPLES[1] == (COPRIME_SEXTUPLE_LCMS[1] as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_SEXTUPLES[2] == (COPRIME_SEXTUPLE_LCMS[2] as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_SEXTUPLES[3] == (COPRIME_SEXTUPLE_LCMS[3] as u64) * (DISCRIMINANT_2 as u64));

    // §8 CCP bridge analysis
    assert!(NULL_HARMONIC_DEFICIT == GEOMETRIC_SPECTRAL_PRODUCT - LCM_SEXT_MAX);
    assert!(NULL_HARMONIC_DEFICIT == NULL_CHANNEL_MOD * LCM_PRIMARY);
    assert!((GEOMETRIC_SPECTRAL_PRODUCT as u64) * (BRIDGE_RATIO_DEN as u64)
        == (LCM_SEXT_MAX as u64) * (BRIDGE_RATIO_NUM as u64));
    assert!(BRIDGE_RATIO_NUM == LAMBDA_EUV);
    assert!(DEFICIT_RATE_DEN == LAMBDA_EUV);
    assert!(BRIDGE_RATIO_NUM == TRI_13);
    assert!(DEFICIT_RATE_NUM * QUAD_PRODUCT == NULL_CHANNEL_MOD * DEFICIT_RATE_DEN);
    assert!(LCM_SEXT_MAX / LCM_PRIMARY == STD_CIRCLE_DEG);
    assert!(QUAD_PRODUCT - STD_CIRCLE_DEG == NULL_CHANNEL_MOD);
    assert!(POS_3D_GEOM_SPECTRAL == (GEOMETRIC_SPECTRAL_PRODUCT as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_NULL_DEFICIT == (NULL_HARMONIC_DEFICIT as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_GEOM_SPECTRAL - POS_3D_SEXTUPLES[3] == POS_3D_NULL_DEFICIT);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT - LCM_SEXT_MAX == NULL_CHANNEL_MOD * LCM_PRIMARY);
    assert!(BRIDGE_RATIO_NUM == ANGULAR_CONV_NUM && BRIDGE_RATIO_DEN == ANGULAR_CONV_DEN);

    // §9 Perfect hash — mixer oddness (bijective mod 2⁶⁴)
    assert!(HMODAL_MIX_A % 2 == 1);
    assert!(HMODAL_MIX_B % 2 == 1);
    // Modulus products — updated to use COPRIME_QUADRUPLE_LCMS (ticket #99 Task 5)
    assert!(7 * 11 * 13 * 15 == COPRIME_QUADRUPLE_LCMS[0]);
    assert!(11 * 13 * 14 * 15 == COPRIME_QUADRUPLE_LCMS[1]);
    assert!(5 * 7 * 8 * 9 * 11 * 13 == LCM_SEXT_MAX);

    // §10 HModal signal
    assert!(HMODAL_TRIT_LOW + HMODAL_TRIT_HIGH == 0);
    assert!(HMODAL_TRIT_LOW + 2 == 1);
    assert!(HMODAL_TRIT_MID + 2 == 2);
    assert!(HMODAL_TRIT_HIGH + 2 == 3);
    assert!((ALPHA_NUM as u64) * (DISCRIMINANT as u64) == (QUAD_PRODUCT as u64) * (ALPHA_DEN as u64));
    assert!((BETA_NUM as u64) * (DISCRIMINANT_SQRT as u64) == (QUAD_PRODUCT as u64) * (BETA_DEN as u64));
    assert!(BETA_NUM * ALPHA_DEN == ALPHA_NUM * BETA_DEN * DISCRIMINANT_SQRT);
    assert!(GAMMA_NUM * BETA_DEN * ALPHA_DEN
        == (BETA_NUM * ALPHA_DEN - ALPHA_NUM * BETA_DEN) * GAMMA_DEN);
    assert!(GAMMA_NUM == 7 * 11 * 13);
    assert!(GAMMA_NUM == LCM_PRIMARY);
    assert!(DUTY_NUM * (DISPATCH_RATIO_DEN + DISPATCH_RATIO_NUM) == DISPATCH_RATIO_NUM * DUTY_DEN);
    assert!(DC_NUM == 5 * 7 * 13);
    assert!((DC_NUM as u64) * (DISCRIMINANT as u64)
        == ((QUAD_PRODUCT as u64) + (GAMMA_NUM as u64)) * (DC_DEN as u64));
    assert!(AC_POWER_NUM == (GAMMA_NUM as u64 * GAMMA_NUM as u64 * 3) as u32);
    assert!(AC_POWER_DEN == GAMMA_DEN * GAMMA_DEN * 16);
    assert!(DC_NUM == 5 * LAMBDA_EUV);
    assert!(HMODAL_TRIT_MID_NUM == ALPHA_NUM * REPUNIT_3);
    assert!(2 * HMODAL_TRIT_MID_NUM * ALPHA_DEN * BETA_DEN
        == (ALPHA_NUM * BETA_DEN + BETA_NUM * ALPHA_DEN) * HMODAL_TRIT_MID_DEN);

    // §11 Channel architecture
    assert!(SIN_PERIOD == 2 * NULL_CHANNEL_MOD);
    assert!(PHASE_STEP_NUM == DUTY_NUM);
    assert!(PHASE_STEP_DEN == DUTY_DEN);

    // §12 Polygon geometry — updated generator ref (ticket #99 Task 5)
    assert!(CENTRAL_ANGLE_SQUARE == QUAD_PRODUCT / NULL_CHANNEL_MOD);
    assert!(CENTRAL_ANGLE_SQUARE == LAMBDA_EUV);
    assert!(CENTRAL_ANGLE_HEPTAGON == QUAD_PRODUCT / COPRIME_TRIPLES[0][0]);
    assert!(CENTRAL_ANGLE_HEPTAGON == NULL_CHANNEL_MOD * REPUNIT_3);
    assert!(CENTRAL_ANGLE_TRIDECAGON == QUAD_PRODUCT / REPUNIT_3);
    assert!(CENTRAL_ANGLE_TRIDECAGON == CYCLIC_ORDER);
    assert!(CENTRAL_ANGLE_TETRADECAGON == QUAD_PRODUCT / ROOT_X1);
    assert!(CENTRAL_ANGLE_TETRADECAGON == ROOT_X2);
    assert!(CENTRAL_ANGLE_TRIANGLE.0 == QUAD_PRODUCT && CENTRAL_ANGLE_TRIANGLE.1 == 3);
    assert!(CENTRAL_ANGLE_PENTAGON.0 * 5 == QUAD_PRODUCT * CENTRAL_ANGLE_PENTAGON.1);
    assert!(CENTRAL_ANGLE_PENTAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_HEXAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_OCTAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_ENNEAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_DECAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_HENDECAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_DODECAGON.0 == QUAD_PRODUCT);
    assert!(CENTRAL_ANGLE_PENTADECAGON.0 == QUAD_PRODUCT);
    assert!(BEZIER_C182_ANGLE == 7 * REPUNIT_3);
    assert!(BEZIER_C650_ANGLE == 11 * REPUNIT_3);
    assert!(BEZIER_C182_ANGLE == LAMBDA_EUV);
    assert!(POLYGON_COUNT == REPUNIT_3);
    assert!(RIM_VERTICES + INTERIOR_INTERSECTIONS == TOTAL_NODES);
    assert!(QUAD_PRODUCT / 4 == LAMBDA_EUV);
    assert!(QUAD_PRODUCT / 13 == 2 * ROOT_X1);
    assert!(QUAD_PRODUCT / 14 == ROOT_X2);
    assert!(ARC_CONVERGENCE_NUM == DISCRIMINANT_SQRT * LAMBDA_EUV);

    // §13 Superhub zones — dodecagon ties added (ticket #99 Task 5)
    assert!(SUPERHUB_AB_POLYGONS[2] == DISCRIMINANT_SQRT);
    assert!(SUPERHUB_CD_POLYGONS[2] == DISCRIMINANT_SQRT);

    // §14 Triangular numbers
    assert!(TRI_3 == 3 * 4 / 2);
    assert!(TRI_7 == 7 * 8 / 2);
    assert!(TRI_10 == 10 * 11 / 2);
    assert!(TRI_13 == 13 * 14 / 2);
    assert!(TRI_7 == CYCLIC_ORDER);
    assert!(TRI_13 == LAMBDA_EUV);
    assert!(2 * TRI_10 + 1 == RADIUS_NUM);

    // §15 Torus knot parameters — coprime pair ties added (ticket #99 Task 5)
    assert!(7 * (11 - 1) == 70);
    assert!(7 * (13 - 1) == 84);
    assert!(7 * (15 - 1) == 98);
    assert!(11 * (13 - 1) == 132);
    assert!(CROSSING_11_14 == 11 * (14 - 1));
    assert!(CROSSING_11_14 == BEZIER_C650_ANGLE);
    assert!(CROSSING_11_14 == COPRIME_PAIR_LCMS[3]);
    assert!(11 * (15 - 1) == 154);
    assert!(CROSSING_13_14 == 13 * (14 - 1));
    assert!(CROSSING_13_14 == REPUNIT_3 * REPUNIT_3);
    assert!(CROSSING_13_15 == 13 * (15 - 1));
    assert!(CROSSING_13_15 == ARC_ROOT_SEMI);
    assert!(CROSSING_13_15 == COPRIME_PAIR_LCMS[6]);
    assert!(CROSSING_14_15 == 14 * (15 - 1));
    assert!(CROSSING_14_15 == ROOT_X1 * ROOT_X1);

    // §17 Plenum square — updated generator ref (ticket #99 Task 5)
    assert!(PLENUM_SQUARE_STEP == 2 * COPRIME_TRIPLES[0][1]);
    assert!(PLENUM_SQUARE_STEP == LAMBDA_UVB / REPUNIT_3);
    assert!(PLENUM_SQUARE_STEP * NULL_CHANNEL_MOD == 88);
    assert!(PLENUM_SQUARE_MIN + PLENUM_SQUARE_STEP * NULL_CHANNEL_MOD == CENTER);
    assert!(PLENUM_SQUARE_MIN == ROOT_X2 - TERNARY_BASE);
    assert!(MAGIC_CONSTANT == CENTER + (CENTER - PLENUM_SQUARE_STEP) + (CENTER + PLENUM_SQUARE_STEP));

    // §19a Fibonacci bridge — pure integer identities
    assert!(FIBONACCI_PI == 377);
    assert!(FIBONACCI_PI == QUAD_PRODUCT + REPUNIT_3);
    assert!(FIBONACCI_PI == REPUNIT_3 * 29);
    assert!(FIBONACCI_PI == REPUNIT_3 * (CYCLIC_ORDER + 1));
    assert!(FIBONACCI_PI == LAMBDA_UVA + REPUNIT_3);
    assert!(FIBONACCI_12 == 144);
    assert!(FIBONACCI_12 == DISCRIMINANT);
    assert!(FIBONACCI_13 == 233);
    assert!(FIBONACCI_PI == FIBONACCI_13 + FIBONACCI_12);
};

// ══════════════════════════════════════════════════════════════
// §RepX  PHYSICS-ENGINE CONSTANTS (task-133)
//
// Values used by the executable physics engine in `repx.rs`.
// Each constant carries a source comment naming its derivation,
// an OT-ID, or its calibration anchor. Symbol Map entries are
// the canonical names referenced by `repx.rs`; bare numeric
// literals matching these values are forbidden inside `repx.rs`.
// ══════════════════════════════════════════════════════════════

// ── Pure-integer Symbol Map additions ──────────────────────

/// 36 = (P13 − P7)² = (13 − 7)² = 6². Used in TM-2026-017 §20.16
/// fine-structure α⁻¹ closed form. Canonical form is `(P13 − P7)²`
/// only — earlier `(R₃ − R₁)² − 1` (143) and `(P11 − P5)²` forms
/// are WITHDRAWN per QC-R1 R1-A3-2.
pub const T_36: u32 = ((P13 - P7) as u32).pow(2);
const _: () = assert!(T_36 == 36);

/// α⁻¹ stepping-stone integer 137,036 = 1000 · α⁻¹.
/// `α⁻¹ = (P11² + REPUNIT_2²) + (P13 − P7)² / (LCM_PRIMARY − 1)`
///       = 137 + 36/1000 = 137.036  (TM-2026-017 §20.16)
pub const ALPHA_INV_INT: u32 =
    (P11 * P11 + REPUNIT_2 * REPUNIT_2) * (LCM_PRIMARY - REPUNIT_1) + T_36;
const _: () = assert!(ALPHA_INV_INT == 137_036);

/// Rydberg-derived numerator used by the OT-1c residual EP κ_ep
/// derivation (TM-2026-017 §20.16 κ Rydberg prediction).
/// `κ_ep = 1 + RYDBERG_NUM_TM017 / ALPHA_INV_INT²`.
///
/// Closed form per Symbol Map row 26147:
/// `ROOT_X2 * (LCM_PRIMARY − REPUNIT_1) + TERNARY_BASE * P7 * P7`
///   = 26 · 1000 + 3 · 49 = 26 147.
///
/// With this value: κ_ep − 1 ≈ 26 147 / 137 036² ≈ 1.392×10⁻⁶,
/// and the OT-1c residual EP `Δa/a = α²·(κ_ep − 1) ≈ 7.41×10⁻¹¹`.
pub const RYDBERG_NUM_TM017: u32 =
    ROOT_X2 * (LCM_PRIMARY - REPUNIT_1) + TERNARY_BASE * P7 * P7;
const _: () = assert!(RYDBERG_NUM_TM017 == 26_147);

/// F8 fine-structure bridge coefficient numerator: `κ_bridge = 91.127/91`
/// (TM-2026-017 §18). DISTINCT from `kappa_ep()` used in OT-1c residual EP.
pub const KAPPA_BRIDGE_NUM: u32 = 91_127;
/// F8 fine-structure bridge coefficient denominator: `91.000`.
pub const KAPPA_BRIDGE_DEN: u32 = 91_000;

/// Brieskorn singularity Milnor number (referenced by the algebraic
/// surfaces underlying the (p, q, r) coprime triple).
pub const MILNOR_NUMBER: u32 = 720;

// ── OT-1c residual EP function ─────────────────────────────

/// κ_ep = 1 + RYDBERG_NUM_TM017 / ALPHA_INV_INT² — closed-form,
/// not a bare constant, because the derivation IS the closure (OT-1c).
/// MUST NOT be conflated with the F8 bridge `KAPPA_BRIDGE_NUM/DEN`.
#[inline]
pub fn kappa_ep() -> f64 {
    1.0 + (RYDBERG_NUM_TM017 as f64) / ((ALPHA_INV_INT as f64).powi(2))
}

// ── SI calibration anchors (f64) for the 39-register grid ──

/// Horn-radius density boundary RHO_0 = 1.15×10⁻² kg/m³, derived
/// from the OT-1g horn fixed-point boundary `f(1) = 1`. Replaces
/// the prior solar-wind matter-density calibration.
pub const RHO_0: f64 = 1.15e-2; // kg/m³ — SI-ANCHOR: FRAMEWORK-DERIVED (OT-1g)

/// Γ₀ ≡ GM_⊙ — SI calibration anchor for the Gaia #2 volumetric
/// downpull register. NOT a derivation; an SI bridge.
pub const GAMMA_0: f64 = 1.327_124_400_18e20; // m³/s² — SI-ANCHOR: IAU-2015

/// Γ₀,G ≡ GM_⊕ — SI calibration anchor for Earth-as-Gaia (Luna-Gaia
/// nested-fulcrum worked example). NOT a derivation; an SI bridge.
pub const GAMMA_0_G: f64 = 3.986_004_418e14; // m³/s² — SI-ANCHOR: IAU-2015

/// Speed of light c² ≡ π_fw under OT-1a horn fixed-point boundary;
/// SI bridge for the Moon #7 acoustic-impedance register.
pub const C_LIGHT: f64 = 2.997_924_58e8; // m/s — SI-ANCHOR: CODATA-2018 (exact)

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 { let t = b; b = a % b; a = t; } a
    }

    #[test] fn full_circle_bound() { assert_eq!(FULL_CIRCLE_DEG, RADIAN_DEG * TWO_PI_TERNARY); }
    #[test] fn pi_bound() { assert_eq!(PI_TERNARY, TWO_PI_TERNARY / 2.0); }
    #[test] fn cyclic_order_matches() { assert_eq!(CYCLIC_ORDER, RADIANS_PER_CIRCLE); assert_eq!(CYCLIC_ORDER, Z28_DIMENSIONS); }

    #[test]
    fn tau_identity() {
        let tau = TAU_TRIBONACCI;
        let tau_cubed = tau * tau * tau;
        let tau_sq_plus_tau_plus_1 = TAU_SQUARED + TAU_TRIBONACCI + 1.0;
        assert!((tau_cubed - tau_sq_plus_tau_plus_1).abs() < 1e-8);
    }

    #[test]
    fn conversion_round_trip_deg() {
        let std = ternary_deg_to_std_deg(FULL_CIRCLE_DEG);
        assert!((std - 360.0).abs() < 1e-10);
        assert!((std_deg_to_ternary_deg(std) - FULL_CIRCLE_DEG).abs() < 1e-10);
    }

    #[test]
    fn conversion_round_trip_rad() {
        let std = ternary_rad_to_std_rad(TWO_PI_TERNARY);
        assert!((std - 2.0 * STD_PI).abs() < 1e-10);
        assert!((std_rad_to_ternary_rad(std) - TWO_PI_TERNARY).abs() < 1e-10);
    }

    #[test] fn radian_conversion() { assert_eq!(ternary_rad_to_ternary_deg(1.0), RADIAN_DEG); assert_eq!(ternary_deg_to_ternary_rad(RADIAN_DEG), 1.0); }
    #[test] fn trit_walk_angles() { assert_eq!(trit_to_walk_angle_deg(0), WALK_TURN_0); assert_eq!(trit_to_walk_angle_deg(1), WALK_TURN_1); assert_eq!(trit_to_walk_angle_deg(2), WALK_TURN_2); }

    #[test]
    fn repunit_formula_verification() {
        for (n, expected) in [(1, 1), (2, 4), (3, 13), (4, 40), (5, 121), (6, 364)] {
            assert_eq!(repunit(n), expected, "repunit({}) failed", n);
        }
    }

    // ── Coprime tests (ticket #99 Task 6 — replaces old sextuple tests) ──

    #[test]
    fn excluded_pairs_not_coprime() {
        for &(a, b) in &EXCLUDED_PAIRS {
            assert_ne!(gcd(a, b), 1, "EXCLUDED_PAIRS: gcd({}, {}) should be > 1", a, b);
        }
    }

    #[test]
    fn coprime_pairs_all_coprime() {
        for &(a, b) in &COPRIME_PAIRS {
            assert_eq!(gcd(a, b), 1, "COPRIME_PAIRS: gcd({}, {}) != 1", a, b);
        }
    }

    #[test]
    fn coprime_triples_all_coprime() {
        for triple in &COPRIME_TRIPLES {
            for i in 0..3 { for j in (i + 1)..3 {
                assert_eq!(gcd(triple[i], triple[j]), 1,
                    "COPRIME_TRIPLES: gcd({}, {}) != 1", triple[i], triple[j]);
            }}
        }
    }

    #[test]
    fn coprime_quadruples_all_coprime() {
        for quad in &COPRIME_QUADRUPLES {
            for i in 0..4 { for j in (i + 1)..4 {
                assert_eq!(gcd(quad[i], quad[j]), 1,
                    "COPRIME_QUADRUPLES: gcd({}, {}) != 1", quad[i], quad[j]);
            }}
        }
    }

    #[test]
    fn coprime_quintuples_all_coprime() {
        for quint in &COPRIME_QUINTUPLES {
            for i in 0..5 { for j in (i + 1)..5 {
                assert_eq!(gcd(quint[i], quint[j]), 1,
                    "COPRIME_QUINTUPLES: gcd({}, {}) != 1", quint[i], quint[j]);
            }}
        }
    }

    #[test]
    fn coprime_sextuples_all_coprime() {
        for sext in &COPRIME_SEXTUPLES {
            for i in 0..6 { for j in (i + 1)..6 {
                assert_eq!(gcd(sext[i], sext[j]), 1,
                    "COPRIME_SEXTUPLES: gcd({}, {}) != 1", sext[i], sext[j]);
            }}
        }
    }

    #[test]
    fn far_uvc_equals_pair_sum() {
        assert_eq!(LAMBDA_FAR_UVC, crate::plenum_square::PAIR_SUM);
    }

    // ── Hash coefficient tests (unchanged) ──

    #[test]
    fn hash_coefficients_a_coprime() {
        for &(m, c) in &HASH_COEFF_A { assert_eq!(gcd(m, c), 1); assert_ne!(c % m, 0); }
    }
    #[test]
    fn hash_coefficients_b_coprime() {
        for &(m, c) in &HASH_COEFF_B { assert_eq!(gcd(m, c), 1); assert_ne!(c % m, 0); }
    }
    #[test]
    fn hash_coefficients_sext_coprime() {
        for &(m, c) in &HASH_COEFF_SEXT { assert_eq!(gcd(m, c), 1); assert_ne!(c % m, 0); }
    }

    #[test]
    fn gcd_of_primaries_is_radian() {
        let g = gcd(gcd(LAMBDA_EUV, LAMBDA_UVC), gcd(LAMBDA_UVB, LAMBDA_UVA));
        assert_eq!(g, 13);
    }

    #[test]
    fn superhub_near_unit_circle() {
        let dist_ab = (SUPERHUB_X_LEFT * SUPERHUB_X_LEFT + SUPERHUB_Y_AB * SUPERHUB_Y_AB).sqrt();
        assert!(dist_ab > 0.96 && dist_ab < 0.98);
        let dist_cd = (SUPERHUB_X_RIGHT * SUPERHUB_X_RIGHT + SUPERHUB_Y_CD * SUPERHUB_Y_CD).sqrt();
        assert!(dist_cd > 0.96 && dist_cd < 0.98);
    }
}

#[cfg(test)]
mod float_tests {
    use super::*;

    #[test]
    fn energy_recovery_from_master_formula() {
        let f = |n: u32| -> f64 {
            let x = STD_PI * n as f64 / 4.0;
            32.0 * x.sin().powi(2) / (3.0 * STD_PI.powi(2) * (n as f64).powi(2))
        };
        let mut cum = 0.0;
        for &(n, target) in &[(1u32, 0.540), (2, 0.811), (3, 0.871)] {
            cum += f(n);
            assert!((cum - target).abs() < 0.001);
        }
        for &n in &[5u32, 6, 7] { cum += f(n); }
        assert!((cum - 0.933).abs() < 0.001);
    }

    #[test]
    fn bridge_coeff_round_trip() {
        let bc = STD_PI / ROOT_X1 as f64;
        assert!((BRIDGE_COEFF - bc).abs() < 1e-15);
        assert!((BRIDGE_COEFF * CYCLIC_ORDER as f64 - 2.0 * STD_PI).abs() < 1e-10);
    }
}
// §19  CODATA VALIDATION TESTS
//
// Validates framework integer predictions against CODATA 2022
// experimental measurements. Organized by prediction tier:
//   Tier 1: < 0.2% (spectral wavelengths from R₆)
//   Tier 2: < 1.0% (impedance, Hartree, H-alpha)
//   Tier 2b: < 5.0% (Rydberg energy vs radian)
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod codata_tests {
    use super::*;

    fn rel_err(predicted: f64, measured: f64) -> f64 {
        (predicted - measured).abs() / measured
    }

    // ── Tier 1: spectral wavelengths (< 0.2%) ───────────────

    #[test]
    fn balmer_constant_matches_circle() {
        let err = rel_err(QUAD_PRODUCT as f64, CODATA_BALMER_CONSTANT_NM);
        assert!(err < CODATA_TIER_1_TOLERANCE,
            "Balmer constant: R₆={} vs B={} nm, error={:.4}%",
            QUAD_PRODUCT, CODATA_BALMER_CONSTANT_NM, err * 100.0);
    }

    #[test]
    fn lyman_limit_matches_euv() {
        let err = rel_err(LAMBDA_EUV as f64, CODATA_LYMAN_LIMIT_NM);
        assert!(err < CODATA_TIER_1_TOLERANCE,
            "Lyman limit: LAMBDA_EUV={} vs 1/R_∞={} nm, error={:.4}%",
            LAMBDA_EUV, CODATA_LYMAN_LIMIT_NM, err * 100.0);
    }

    #[test]
    fn lyman_alpha_matches_triangle_angle() {
        let predicted = QUAD_PRODUCT as f64 / 3.0;
        let err = rel_err(predicted, CODATA_LYMAN_ALPHA_NM);
        assert!(err < CODATA_TIER_1_TOLERANCE,
            "Lyman-α: R₆/3={:.3} vs {:.3} nm, error={:.4}%",
            predicted, CODATA_LYMAN_ALPHA_NM, err * 100.0);
    }

    #[test]
    fn balmer_limit_is_four_times_lyman_limit() {
        // Both derived from R_∞: B = 4/R_∞, L = 1/R_∞ → B/L = 4 exactly.
        let ratio = CODATA_BALMER_CONSTANT_NM / CODATA_LYMAN_LIMIT_NM;
        assert!((ratio - 4.0).abs() < 1e-6,
            "Balmer/Lyman ratio should be 4, got {}", ratio);
    }

    #[test]
    fn lyman_series_from_circle() {
        // Every Lyman line ≈ R₆ × n² / (4(n² − 1))
        let r6 = QUAD_PRODUCT as f64;
        let measured: [(u32, f64); 4] = [
            (2, 121.5670),   // Lyman-α
            (3, 102.5728),   // Lyman-β
            (4,  97.2537),   // Lyman-γ
            (5,  94.9743),   // Lyman-δ
        ];
        for &(n, lambda_measured) in &measured {
            let n2 = (n * n) as f64;
            let predicted = r6 * n2 / (4.0 * (n2 - 1.0));
            let err = rel_err(predicted, lambda_measured);
            assert!(err < CODATA_TIER_1_TOLERANCE,
                "Lyman n={}: predicted={:.3} vs measured={:.4}, error={:.4}%",
                n, predicted, lambda_measured, err * 100.0);
        }
    }

    #[test]
    fn balmer_series_from_circle() {
        // Every Balmer line ≈ R₆ × n² / (n² − 4)
        let r6 = QUAD_PRODUCT as f64;
        let measured: [(u32, f64); 4] = [
            (3, 656.281),    // H-α
            (4, 486.135),    // H-β
            (5, 434.047),    // H-γ
            (6, 410.174),    // H-δ
        ];
        for &(n, lambda_measured) in &measured {
            let n2 = (n * n) as f64;
            let predicted = r6 * n2 / (n2 - 4.0);
            let err = rel_err(predicted, lambda_measured);
            assert!(err < CODATA_TIER_2_TOLERANCE,
                "Balmer n={}: predicted={:.3} vs measured={:.3}, error={:.4}%",
                n, predicted, lambda_measured, err * 100.0);
        }
    }

    // ── Tier 2: impedance and energy (< 1%) ─────────────────

    #[test]
    fn z0_matches_fibonacci_pi() {
        let err = rel_err(FIBONACCI_PI as f64, CODATA_Z0_OHM);
        assert!(err < CODATA_TIER_2_TOLERANCE,
            "Z₀: F(π)={} vs {:.6} Ω, error={:.4}%",
            FIBONACCI_PI, CODATA_Z0_OHM, err * 100.0);
    }

    #[test]
    fn hartree_matches_z27() {
        let err = rel_err(DISCRIMINANT_2_SQRT as f64, CODATA_HARTREE_EV);
        assert!(err < CODATA_TIER_2_TOLERANCE,
            "Hartree: √Δ₂={} vs {:.6} eV, error={:.4}%",
            DISCRIMINANT_2_SQRT, CODATA_HARTREE_EV, err * 100.0);
    }

    #[test]
    fn h_alpha_matches_arc_root_comp() {
        let err = rel_err(ARC_ROOT_COMP as f64, CODATA_H_ALPHA_NM);
        assert!(err < CODATA_TIER_2_TOLERANCE,
            "H-α: ARC_ROOT_COMP={} vs {:.3} nm, error={:.4}%",
            ARC_ROOT_COMP, CODATA_H_ALPHA_NM, err * 100.0);
    }

    // ── Tier 2b: Rydberg energy (< 5%) ──────────────────────

    #[test]
    fn rydberg_matches_radian() {
        let err = rel_err(REPUNIT_3 as f64, CODATA_RYDBERG_EV);
        assert!(err < CODATA_TIER_2B_TOLERANCE,
            "Rydberg: R₃={} vs {:.6} eV, error={:.4}%",
            REPUNIT_3, CODATA_RYDBERG_EV, err * 100.0);
    }

    // ── Structural identities ────────────────────────────────

    #[test]
    fn hartree_is_double_rydberg() {
        let ratio = CODATA_HARTREE_EV / CODATA_RYDBERG_EV;
        assert!((ratio - 2.0).abs() < 1e-10,
            "Hartree/Rydberg should be exactly 2, got {}", ratio);
    }

    #[test]
    fn h_alpha_residual_near_two_pi_conventional() {
        let residual = CODATA_H_ALPHA_NM - ARC_ROOT_COMP as f64;
        let two_pi = 2.0 * STD_PI;
        let err = (residual - two_pi).abs() / two_pi;
        assert!(err < 0.005,
            "H-α residual: {} vs 2π={:.5}, error={:.4}%",
            residual, two_pi, err * 100.0);
    }

    #[test]
    fn vacuum_bias_consistent() {
        // R_H = R_∞ / (1 + mₑ/mₚ)
        let r_h = CODATA_RYDBERG_CONST / (1.0 + CODATA_ME_OVER_MP);
        let lyman_limit_h = 1e9 / r_h;
        let bias_computed = (lyman_limit_h - LAMBDA_EUV as f64) / LAMBDA_EUV as f64;
        let bias_stored = VACUUM_BIAS_NUM as f64 / VACUUM_BIAS_DEN as f64;
        assert!((bias_computed - bias_stored).abs() < 5e-4,
            "Vacuum bias: computed={:.6} vs stored={:.6}",
            bias_computed, bias_stored);
    }

    #[test]
    fn fibonacci_pi_decomposition() {
        assert_eq!(FIBONACCI_PI, QUAD_PRODUCT + REPUNIT_3);
        assert_eq!(FIBONACCI_PI, REPUNIT_3 * 29);
        assert_eq!(FIBONACCI_PI, FIBONACCI_13 + FIBONACCI_12);
        assert_eq!(FIBONACCI_12, DISCRIMINANT);
    }

    #[test]
    fn fibonacci_sequence_verification() {
        // Verify the Fibonacci sequence from F(0) to F(14)
        let mut fib = [0u32; 15];
        fib[0] = 0;
        fib[1] = 1;
        for i in 2..15 {
            fib[i] = fib[i - 1] + fib[i - 2];
        }
        assert_eq!(fib[7], REPUNIT_3, "F(7) should equal R₃ = 13");
        assert_eq!(fib[12], FIBONACCI_12, "F(12) should equal 144 = Δ");
        assert_eq!(fib[13], FIBONACCI_13, "F(13) should equal 233");
        assert_eq!(fib[14], FIBONACCI_PI, "F(14) should equal 377");
    }
}