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

// ══════════════════════════════════════════════════════════════
// §1  REPUNIT FAMILY (TM-2026-017 §2.1)
// ══════════════════════════════════════════════════════════════

/// The ternary radix — foundation of the number system.
/// Used in all repunit formulas, Δ₂ = 3⁶, and recurrences.
pub const TERNARY_BASE: u32 = 3;

/// R₁ = (3¹ − 1)/2 = 1.
pub const REPUNIT_1: u32 = 1;

/// R₂ = (3² − 1)/2 = 4.
pub const REPUNIT_2: u32 = 4;

/// R₃ = (3³ − 1)/2 = 13. The radian unit.
pub const REPUNIT_3: u32 = 13;

/// R₄ = (3⁴ − 1)/2 = 40. Sum of circle quadratic roots.
pub const REPUNIT_4: u32 = 40;

/// R₅ = (3⁵ − 1)/2 = 121.
pub const REPUNIT_5: u32 = 121;

/// R₆ = (3⁶ − 1)/2 = 364. The full circle.
pub const REPUNIT_6: u32 = 364;

/// Master repunit generating function: Rₙ = (3ⁿ − 1) / (3 − 1).
pub const fn repunit(n: u32) -> u32 {
    (TERNARY_BASE.pow(n) - 1) / (TERNARY_BASE - 1)
}

// ══════════════════════════════════════════════════════════════
// §2  CIRCLE QUADRATIC (TM-2026-017 §2.2)
//     x² − R₄·x + R₆ = 0  →  x² − 40x + 364 = 0
// ══════════════════════════════════════════════════════════════

/// Vieta sum x₁ + x₂ = R₄ = 40.
pub const QUAD_SUM: u32 = 40;

/// Vieta product x₁ · x₂ = R₆ = 364.
pub const QUAD_PRODUCT: u32 = 364;

/// Discriminant Δ = R₄² − 4·R₆ = 1600 − 1456 = 144.
pub const DISCRIMINANT: u32 = 144;

/// √Δ = 12. The root spread and lattice parameter.
pub const DISCRIMINANT_SQRT: u32 = 12;

/// Smaller root x₁ = π = (R₄ − √Δ)/2 = (40 − 12)/2 = 14.
pub const ROOT_X1: u32 = 14;

/// Larger root x₂ = R₆/π = (R₄ + √Δ)/2 = (40 + 12)/2 = 26.
pub const ROOT_X2: u32 = 26;

// ══════════════════════════════════════════════════════════════
// §3  UNIFIED EQUATION (TM-2026-017 §2.3–§2.7)
//     arc² − 832·arc + 118,300 = 0
// ══════════════════════════════════════════════════════════════

/// |linear coefficient| = R₄(R₄−1) − 2R₆ = 40×39 − 728 = 832.
pub const UNIFIED_LINEAR: u32 = 832;

/// Constant term = R₆ · (R₆ − R₄ + 1) = 364 × 325 = 118,300.
pub const UNIFIED_CONSTANT: u32 = 118_300;

/// R₆ − R₄ + 1 = 364 − 40 + 1 = 325.
pub const UNIFIED_FACTOR: u32 = 325;

/// Δ_arc = 832² − 4·118300 = 219,024.
pub const UNIFIED_DISC: u32 = 219_024;

/// √Δ_arc = 468.
pub const UNIFIED_DISC_SQRT: u32 = 468;

/// Semicircle root = (832 − 468)/2 = 182.
pub const ARC_ROOT_SEMI: u32 = 182;

/// Complementary root = (832 + 468)/2 = 650.
pub const ARC_ROOT_COMP: u32 = 650;

/// Green arc effective span = 650 − 364 = 286.
pub const GREEN_ARC_EFF: u32 = 286;

/// Center c = (arc + R₄)/2 = (182 + 40)/2 = 111.
/// In TM-2026-017 §2.7 notation, d = c = 111 and r = d/2 = 55.5.
pub const CENTER: u32 = 111;

/// Radius numerator: r = d/2 = 111/2. Stored as numerator over denominator.
pub const RADIUS_NUM: u32 = 111;

/// Radius denominator.
pub const RADIUS_DEN: u32 = 2;

/// Δ₂ = 1 + 4·arc = 1 + 4·182 = 729 = 3⁶ = 27².
/// The kernel sponge state width of TLSponge-385.
pub const DISCRIMINANT_2: u32 = 729;

/// √Δ₂ = 27.
pub const DISCRIMINANT_2_SQRT: u32 = 27;

/// Hexagon perimeter = 6r = 3d = 3 × 111 = 333 (d = CENTER = 111).
pub const MAGIC_CONSTANT: u32 = 333;

/// Circumference πd = 14 × 111 = 1554 = 28r = 2πr (d = CENTER = 111).
pub const CIRCUMFERENCE: u32 = 1554;

// ══════════════════════════════════════════════════════════════
// §4  SQUARED CIRCLE (TM-2026-017 §3)
// ══════════════════════════════════════════════════════════════

/// Unit circle area = πr² = 14 × 1² = π = 14.
pub const UNIT_CIRCLE_AREA: u32 = 14;

/// Radian circle area = πr² = 14 × 13 = 182 (r = √13).
pub const RADIAN_CIRCLE_AREA: u32 = 182;

/// Side² of squared unit circle = π = 14.
pub const SQUARED_SIDE_SQ_UNIT: u32 = 14;

/// Side² of squared r=√13 circle = 182.
pub const SQUARED_SIDE_SQ_RADIAN: u32 = 182;

// ══════════════════════════════════════════════════════════════
// §5  ANGULAR CONVERSION FACTOR
// ══════════════════════════════════════════════════════════════

/// Standard circle in degrees — external reference (not derived from ternary axiom).
pub const STD_CIRCLE_DEG: u32 = 360;

/// Angular conversion factor numerator: κ = R₆/360 = 364/360 = 91/90.
pub const ANGULAR_CONV_NUM: u32 = 91;

/// Angular conversion factor denominator.
pub const ANGULAR_CONV_DEN: u32 = 90;

// ══════════════════════════════════════════════════════════════
// §6  UV SPECTRAL WAVELENGTHS (TM-2026-017 §16, §2.5)
// ══════════════════════════════════════════════════════════════

/// Quarter-turn = 7 × 13 = 7 radians. Lyman anchor.
pub const LAMBDA_EUV: u32 = 91;

/// Half-turn = 14 × 13 = π radians. O₂ absorption wall.
pub const LAMBDA_UVC: u32 = 182;

/// Green arc effective = 22 × 13 = 22 radians. Ozone bridge.
pub const LAMBDA_UVB: u32 = 286;

/// Full circle = 28 × 13 = 2π radians. Full transmission.
pub const LAMBDA_UVA: u32 = 364;

/// Far-UVC = 2 × CENTER = 2 × 111 = 222.
pub const LAMBDA_FAR_UVC: u32 = 222;

/// XeCl excimer = 4 × 7 × 11 = 308.
pub const LAMBDA_EXCIMER: u32 = 308;

/// Narrowband UVB = e₂ = pq + pr + qr = 7×11 + 7×13 + 11×13 = 311.
pub const LAMBDA_NB_UVB: u32 = 311;

/// EUV|UVC boundary = floor((91 + 182) / 2) = 136.
/// Note: exact midpoint is 136.5 nm — half-integer truncated to u32.
pub const BOUNDARY_EUV_UVC: u32 = 136;

/// UVC|UVB boundary = (182 + 286) / 2 = 234 (exact integer).
pub const BOUNDARY_UVC_UVB: u32 = 234;

/// UVB|UVA boundary = (286 + 364) / 2 = 325 (exact integer).
/// Structural tie: this equals UNIFIED_FACTOR = R₆ − R₄ + 1 = 325.
/// The spectral partition boundary and the algebraic factor that generates
/// the unified equation's constant term (118,300 = 364 × 325) are the same number.
pub const BOUNDARY_UVB_UVA: u32 = 325;

/// UV|Visible boundary.
pub const BOUNDARY_UV_VIS: u32 = 400;

/// Vacuum bias numerator: (1/R_H − 91)/91 ≈ 0.00193.
pub const VACUUM_BIAS_NUM: u32 = 193;

/// Vacuum bias denominator.
pub const VACUUM_BIAS_DEN: u32 = 100_000;

// ══════════════════════════════════════════════════════════════
// §7  COPRIME WALK LANDSCAPE (TM-2026-017 §10)
// ══════════════════════════════════════════════════════════════

/// Primary coprime generators from the arc factorizations.
/// 182 = 2×7×13 and 286 = 2×11×13 → reduced ratio 7:11.
pub const COPRIME_TRIPLE: [u32; 3] = [7, 11, 13];

/// Pentadecagon = 15 = 3 × 5. Bridges triangle and pentagon families.
pub const PENTADECAGON: u32 = 15;

/// The (7, 14) exclusion — gcd(7, 14) = 7 ≠ 1.
/// The only excluded pair from key polygon set {7, 11, 13, 14, 15}.
/// 14 = 2 × 7 = 2 × COPRIME_TRIPLE[0].
pub const EXCLUDED_PAIR: (u32, u32) = (7, 14);

/// Sextuple A: [3, 4, 5, 7, 11, 13] → LCM = 60,060.
pub const SEXTUPLE_A: [u32; 6] = [3, 4, 5, 7, 11, 13];

/// Sextuple B: [3, 5, 7, 8, 11, 13] → LCM = 120,120.
pub const SEXTUPLE_B: [u32; 6] = [3, 5, 7, 8, 11, 13];

/// Sextuple C: [4, 5, 7, 9, 11, 13] → LCM = 180,180.
pub const SEXTUPLE_C: [u32; 6] = [4, 5, 7, 9, 11, 13];

/// Maximum pairwise coprime sextuple: [5, 7, 8, 9, 11, 13] → LCM = 360,360.
/// No group of 7 exists from the polygon set — this is the structural limit.
pub const SEXTUPLE_MAX: [u32; 6] = [5, 7, 8, 9, 11, 13];

/// LCM of primary coprime triple: 7 × 11 × 13 = 1,001.
pub const LCM_PRIMARY: u32 = 1_001;

/// LCM of odd-prime quadruple: 3 × 5 × 7 × 11 × 13 = 15 × 1001 = 15,015.
pub const LCM_QUAD_ODD: u32 = 15_015;

/// LCM of π-gon quadruple: 2 × 3 × 5 × 7 × 11 × 13 = 2 × 15015 = 30,030.
pub const LCM_QUAD_PI: u32 = 30_030;

/// LCM of sextuple A: 3 × 4 × 5 × 7 × 11 × 13 = 60,060.
pub const LCM_SEXT_A: u32 = 60_060;

/// LCM of sextuple B: 3 × 5 × 7 × 8 × 11 × 13 = 120,120.
pub const LCM_SEXT_B: u32 = 120_120;

/// LCM of sextuple C: 4 × 5 × 7 × 9 × 11 × 13 = 180,180.
pub const LCM_SEXT_C: u32 = 180_180;

/// LCM of maximum sextuple: 360 × 1,001 = 360,360.
pub const LCM_SEXT_MAX: u32 = 360_360;

/// 3D position count (primary): 1,001 × 729.
pub const POS_3D_PRIMARY: u64 = 729_729;

/// 3D position count (odd-prime quadruple): 15,015 × 729.
pub const POS_3D_QUAD_ODD: u64 = 10_945_935;

/// 3D position count (π-gon quadruple): 30,030 × 729.
pub const POS_3D_QUAD_PI: u64 = 21_891_870;

/// 3D position count (maximum sextuple): 360,360 × 729.
pub const POS_3D_SEXT_MAX: u64 = 262_702_440;

// ══════════════════════════════════════════════════════════════
// §8  CCP BRIDGE CONSTANTS (Circle × Coprime Product)
// ══════════════════════════════════════════════════════════════

/// 364 × 1,001 = 364,364. Circle × coprime walk. Repdigit structure: 364 repeats.
/// Factorization: 2² × 7² × 11 × 13².
pub const GEOMETRIC_SPECTRAL_PRODUCT: u32 = 364_364;

/// 364,364 − 360,360 = 4,004 = 4 × 1,001. Null harmonic deficit.
pub const NULL_HARMONIC_DEFICIT: u32 = 4_004;

/// Bridge ratio numerator: 364,364 / 360,360 = 91/90.
pub const BRIDGE_RATIO_NUM: u32 = 91;

/// Bridge ratio denominator.
pub const BRIDGE_RATIO_DEN: u32 = 90;

/// Deficit rate numerator: 4,004 / 364,364 = 4/364 = 1/91.
pub const DEFICIT_RATE_NUM: u32 = 1;

/// Deficit rate denominator.
pub const DEFICIT_RATE_DEN: u32 = 91;

/// 3D geometric-spectral: 364,364 × 729.
pub const POS_3D_GEOM_SPECTRAL: u64 = 265_621_356;

/// 3D null deficit: 4,004 × 729.
pub const POS_3D_NULL_DEFICIT: u64 = 2_918_916;

// ══════════════════════════════════════════════════════════════
// §9  PERFECT HASH COEFFICIENTS (TM-2026-028a §2–§3)
// ══════════════════════════════════════════════════════════════

/// CRT coefficients for odd-prime quadruple (7, 11, 13, 15).
/// Tuple order: (modulus, coefficient). Each c satisfies gcd(c, m) = 1 and c mod m ≠ 0.
pub const HASH_COEFF_A: [(u32, u32); 4] = [(7, 2), (11, 3), (13, 5), (15, 7)];

/// CRT coefficients for π-gon quadruple (11, 13, 14, 15).
/// Tuple order: (modulus, coefficient). Each c satisfies gcd(c, m) = 1 and c mod m ≠ 0.
pub const HASH_COEFF_B: [(u32, u32); 4] = [(11, 2), (13, 3), (14, 5), (15, 7)];

/// CRT coefficients for maximum sextuple (5, 7, 8, 9, 11, 13).
/// Tuple order: (modulus, coefficient). Each c satisfies gcd(c, m) = 1 and c mod m ≠ 0.
/// Note: original TM-2026-028a §3.2 uses (11,11) and (13,13), but c mod m = 0
/// destroys bijectivity. Using c₁₁ = 4 and c₁₃ = 6 instead (known erratum).
pub const HASH_COEFF_SEXT: [(u32, u32); 6] = [(5, 2), (7, 3), (8, 5), (9, 7), (11, 4), (13, 6)];

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

/// Idle state numerator: α = R₆/Δ = 364/144 = 91/36.
pub const ALPHA_NUM: u32 = 91;

/// Idle state denominator.
pub const ALPHA_DEN: u32 = 36;

/// Dispatch state numerator: β = R₆/√Δ = 364/12 = 91/3.
pub const BETA_NUM: u32 = 91;

/// Dispatch state denominator.
pub const BETA_DEN: u32 = 3;

/// Transition magnitude numerator: γ = β − α = 1001/36.
/// 1001 = 7 × 11 × 13 — the coprime walk product appears in the transition.
pub const GAMMA_NUM: u32 = 1001;

/// Transition magnitude denominator.
pub const GAMMA_DEN: u32 = 36;

/// Dispatch-to-idle time ratio numerator = 1/3.
pub const DISPATCH_RATIO_NUM: u32 = 1;

/// Dispatch-to-idle time ratio denominator.
pub const DISPATCH_RATIO_DEN: u32 = 3;

/// Duty cycle numerator: d = 1/4.
/// Derivation: d = (1/3)/(1 + 1/3) = (1/3)/(4/3) = 1/4.
pub const DUTY_NUM: u32 = 1;

/// Duty cycle denominator.
pub const DUTY_DEN: u32 = 4;

/// DC component numerator: ⟨H⟩ = α + γd = 455/48.
/// 455 = 5 × 7 × 13 — the factor 5 emerges uninvited.
pub const DC_NUM: u32 = 455;

/// DC component denominator.
pub const DC_DEN: u32 = 48;

/// AC power numerator: P_AC = γ²·d(1−d) = 3,006,003/20,736.
pub const AC_POWER_NUM: u32 = 3_006_003;

/// AC power denominator.
pub const AC_POWER_DEN: u32 = 20_736;

/// HModal trit mapping — low (α state, 75% dwell).
pub const HMODAL_TRIT_LOW: i8 = -1;

/// HModal trit mapping — mid (transition, zero-crossing).
pub const HMODAL_TRIT_MID: i8 = 0;

/// Transition midpoint signal level numerator: (α+β)/2 = 1183/72.
/// 1183 = 91 × 13 = ALPHA_NUM × REPUNIT_3.
pub const HMODAL_TRIT_MID_NUM: u32 = 1183;

/// Transition midpoint signal level denominator.
pub const HMODAL_TRIT_MID_DEN: u32 = 72;

/// HModal trit mapping — high (β state, 25% dwell).
pub const HMODAL_TRIT_HIGH: i8 = 1;

// ══════════════════════════════════════════════════════════════
// §11  CHANNEL ARCHITECTURE (TM-2026-028 §3–§4)
// ══════════════════════════════════════════════════════════════

/// Null-channel modulus: A_n = 0 exactly when n ≡ 0 (mod 4).
pub const NULL_CHANNEL_MOD: u32 = 4;

/// Period of |sin(πn/4)| = 2 × NULL_CHANNEL_MOD = 8.
pub const SIN_PERIOD: u32 = 8;

/// Phase step numerator: each harmonic rotates by π/4 (fraction of π).
pub const PHASE_STEP_NUM: u32 = 1;

/// Phase step denominator.
pub const PHASE_STEP_DEN: u32 = 4;

// ══════════════════════════════════════════════════════════════
// §12  POLYGON GEOMETRY (TM-2026-017 §4–§5)
// ══════════════════════════════════════════════════════════════

/// Number of regular n-gons for n = 3..15 = radian unit.
pub const POLYGON_COUNT: u32 = 13;

/// Central angle generating function: θ_n = 364/n.
pub const fn central_angle(n: u32) -> (u32, u32) {
    (QUAD_PRODUCT, n)
}

/// θ₃ = 364/3 (rational).
pub const CENTRAL_ANGLE_TRIANGLE: (u32, u32) = (364, 3);

/// θ₄ = 364/4 = 91° (exact integer).
pub const CENTRAL_ANGLE_SQUARE: u32 = 91;

/// θ₅ = 364/5 (rational).
pub const CENTRAL_ANGLE_PENTAGON: (u32, u32) = (364, 5);

/// θ₆ = 364/6 (rational).
pub const CENTRAL_ANGLE_HEXAGON: (u32, u32) = (364, 6);

/// θ₇ = 364/7 = 52° (exact integer).
pub const CENTRAL_ANGLE_HEPTAGON: u32 = 52;

/// θ₈ = 364/8 (rational).
pub const CENTRAL_ANGLE_OCTAGON: (u32, u32) = (364, 8);

/// θ₉ = 364/9 (rational).
pub const CENTRAL_ANGLE_ENNEAGON: (u32, u32) = (364, 9);

/// θ₁₀ = 364/10 (rational).
pub const CENTRAL_ANGLE_DECAGON: (u32, u32) = (364, 10);

/// θ₁₁ = 364/11 (rational).
pub const CENTRAL_ANGLE_HENDECAGON: (u32, u32) = (364, 11);

/// θ₁₂ = 364/12 (rational).
pub const CENTRAL_ANGLE_DODECAGON: (u32, u32) = (364, 12);

/// θ₁₃ = 364/13 = 28° (exact integer).
pub const CENTRAL_ANGLE_TRIDECAGON: u32 = 28;

/// θ₁₄ = 364/14 = 26° (exact integer). The π-gon's central angle = x₂.
pub const CENTRAL_ANGLE_TETRADECAGON: u32 = 26;

/// θ₁₅ = 364/15 (rational).
pub const CENTRAL_ANGLE_PENTADECAGON: (u32, u32) = (364, 15);

/// Bézier C₁₈₂ control point angle = 91° = 7 custom radians. Coordinates (0, 1).
pub const BEZIER_C182_ANGLE: u32 = 91;

/// Bézier C₆₅₀ control point angle = 143° = 11 custom radians.
pub const BEZIER_C650_ANGLE: u32 = 143;

/// C₁₈₂ in custom radians.
pub const BEZIER_C182_RADIANS: u32 = 7;

/// C₆₅₀ in custom radians.
pub const BEZIER_C650_RADIANS: u32 = 11;

/// Arc convergence numerator: 218.4° = 1092/5 (3 × pentagon central angle).
pub const ARC_CONVERGENCE_NUM: u32 = 1092;

/// Arc convergence denominator.
pub const ARC_CONVERGENCE_DEN: u32 = 5;

/// Rim vertices in the inscribed polygon overlay (TM-2026-017 §11.1).
pub const RIM_VERTICES: u32 = 58;

/// Interior intersections.
pub const INTERIOR_INTERSECTIONS: u32 = 446;

/// Total nodes.
pub const TOTAL_NODES: u32 = 504;

// ══════════════════════════════════════════════════════════════
// §13  SUPERHUB ZONES — integer data (TM-2026-017 §11.2–§11.5)
// ══════════════════════════════════════════════════════════════

/// Polygon membership for zones A & B: 7, 11, 12, 13.
/// 11, 12, 13 appear in ALL four zones; 4th switches between 7 (A/B) and 8 (C/D).
pub const SUPERHUB_AB_POLYGONS: [u32; 4] = [7, 11, 12, 13];

/// Polygon membership for zones C & D: 8, 11, 12, 13.
pub const SUPERHUB_CD_POLYGONS: [u32; 4] = [8, 11, 12, 13];

// ══════════════════════════════════════════════════════════════
// §14  TRIANGULAR NUMBER ANCHORS (TM-2026-017 §9)
// ══════════════════════════════════════════════════════════════

/// Tri(3) = 6. Hexagon sides.
pub const TRI_3: u32 = 6;

/// Tri(7) = 28 = 2π. Full circle in radians.
pub const TRI_7: u32 = 28;

/// Tri(10) = 55. Radius = Tri(10) + ½ = 55.5.
pub const TRI_10: u32 = 55;

/// Tri(13) = 91. Quarter-turn.
pub const TRI_13: u32 = 91;

// ══════════════════════════════════════════════════════════════
// §15  TORUS KNOT PARAMETERS (TM-2026-017 §10.6–§10.8)
// ══════════════════════════════════════════════════════════════

/// Crossing number (11,14) = 11 × 13 = 143 = BEZIER_C650_ANGLE.
pub const CROSSING_11_14: u32 = 143;

/// Crossing number (13,14) = 13 × 13 = 169 = REPUNIT_3² (radian squared).
pub const CROSSING_13_14: u32 = 169;

/// Crossing number (13,15) = 13 × 14 = 182 = ARC_ROOT_SEMI (semicircle).
pub const CROSSING_13_15: u32 = 182;

/// Crossing number (14,15) = 14 × 14 = 196 = ROOT_X1² (π squared).
/// Note: TM-2026-017 v6.0 erroneously states 195 — correct value is 14 × (15−1) = 196.
pub const CROSSING_14_15: u32 = 196;

// ══════════════════════════════════════════════════════════════
// §17  PLENUM SQUARE SCALING (Lo Shu × 22 + CENTER)
// ══════════════════════════════════════════════════════════════

/// Lo Shu scaling factor. All nine entries are CENTER ± k × 22 for k ∈ {0,1,2,3,4}.
/// The 11 → 22 → 88 → 23 chain: hendecagon (11) doubles to step (22),
/// null period (4) scales step to 88, CENTER minus 88 is the smallest entry (23 = x₂ − 3).
pub const PLENUM_SQUARE_STEP: u32 = 22;

/// Smallest magic square entry = CENTER − 4 × STEP = 111 − 88 = 23.
pub const PLENUM_SQUARE_MIN: u32 = 23;

// ══════════════════════════════════════════════════════════════
// Z₂₈ CYCLIC GROUP
// ══════════════════════════════════════════════════════════════

/// Order of the cyclic group Z₂₈ generated by the ternary radian.
pub const CYCLIC_ORDER: u32 = 28;

/// Number of ternary radians in a full circle (= CYCLIC_ORDER).
pub const RADIANS_PER_CIRCLE: u32 = 28;

/// Number of dimensions in the Tribonacci 28-Dimension Symmetry.
pub const Z28_DIMENSIONS: u32 = 28;

/// The generator of Z₂₈ — the angular step of 1 ternary radian (13°).
pub const Z28_GENERATOR: u32 = 1;

/// The co-generator: 13 radians maps back to 1° short of full coverage,
/// and 13 is itself the radian value. gcd(13, 28) = 1 so 13 also generates Z₂₈.
pub const Z28_CO_GENERATOR: u32 = 13;

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

    // §4 Squared circle
    assert!(UNIT_CIRCLE_AREA == ROOT_X1);
    assert!(RADIAN_CIRCLE_AREA == ROOT_X1 * REPUNIT_3);
    assert!(RADIAN_CIRCLE_AREA == ARC_ROOT_SEMI);
    assert!(SQUARED_SIDE_SQ_RADIAN == SQUARED_SIDE_SQ_UNIT * REPUNIT_3);
    assert!(RADIAN_CIRCLE_AREA == 2 * 7 * 13);

    // §5 Angular conversion
    assert!(ANGULAR_CONV_NUM * STD_CIRCLE_DEG == ANGULAR_CONV_DEN * QUAD_PRODUCT);
    assert!(ANGULAR_CONV_NUM == LAMBDA_EUV);

    // §6 UV spectral wavelengths
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
    assert!(LAMBDA_EXCIMER == 4 * 7 * 11);
    assert!(LAMBDA_NB_UVB == 7 * 11 + 7 * 13 + 11 * 13);
    // NB-UVB = 311 is prime: trial division up to √311 ≈ 17.6
    assert!(311 % 2 != 0 && 311 % 3 != 0 && 311 % 5 != 0
        && 311 % 7 != 0 && 311 % 11 != 0 && 311 % 13 != 0 && 311 % 17 != 0);
    assert!(BOUNDARY_UVB_UVA == UNIFIED_FACTOR);
    assert!(LAMBDA_UVA == QUAD_PRODUCT);
    assert!(LAMBDA_UVC == ARC_ROOT_SEMI);
    assert!(GREEN_ARC_EFF == LAMBDA_UVB);

    // §7 Coprime walk
    assert!(ARC_ROOT_SEMI == 2 * COPRIME_TRIPLE[0] * COPRIME_TRIPLE[2]);
    assert!(GREEN_ARC_EFF == 2 * COPRIME_TRIPLE[1] * COPRIME_TRIPLE[2]);
    assert!(LCM_PRIMARY == 7 * 11 * 13);
    assert!(LCM_QUAD_ODD == 15 * LCM_PRIMARY);
    assert!(LCM_QUAD_ODD == 3 * 5 * 7 * 11 * 13);
    assert!(LCM_QUAD_PI == 2 * LCM_QUAD_ODD);
    assert!(LCM_SEXT_MAX == 360 * LCM_PRIMARY);
    assert!(LCM_SEXT_MAX == 24 * LCM_QUAD_ODD);
    assert!(LCM_SEXT_MAX == 5 * 7 * 8 * 9 * 11 * 13);
    assert!(LCM_SEXT_A == 3 * 4 * 5 * 7 * 11 * 13);
    assert!(LCM_SEXT_B == 3 * 5 * 7 * 8 * 11 * 13);
    assert!(LCM_SEXT_C == 4 * 5 * 7 * 9 * 11 * 13);
    assert!(POS_3D_PRIMARY == (LCM_PRIMARY as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_QUAD_ODD == (LCM_QUAD_ODD as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_QUAD_PI == (LCM_QUAD_PI as u64) * (DISCRIMINANT_2 as u64));
    assert!(POS_3D_SEXT_MAX == (LCM_SEXT_MAX as u64) * (DISCRIMINANT_2 as u64));

    // §8 CCP bridge
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == QUAD_PRODUCT * LCM_PRIMARY);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == 364 * 1000 + 364);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT == 4 * 7 * 7 * 11 * 13 * 13);
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
    assert!(POS_3D_GEOM_SPECTRAL - POS_3D_SEXT_MAX == POS_3D_NULL_DEFICIT);
    assert!(GEOMETRIC_SPECTRAL_PRODUCT - LCM_SEXT_MAX == NULL_CHANNEL_MOD * LCM_PRIMARY);
    assert!(BRIDGE_RATIO_NUM == ANGULAR_CONV_NUM && BRIDGE_RATIO_DEN == ANGULAR_CONV_DEN);

    // §9 Perfect hash — mixer oddness (bijective mod 2⁶⁴)
    assert!(HMODAL_MIX_A % 2 == 1);
    assert!(HMODAL_MIX_B % 2 == 1);
    // Modulus products
    assert!(7 * 11 * 13 * 15 == LCM_QUAD_ODD);
    assert!(11 * 13 * 14 * 15 == LCM_QUAD_PI);
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

    // §12 Polygon geometry
    assert!(CENTRAL_ANGLE_SQUARE == QUAD_PRODUCT / NULL_CHANNEL_MOD);
    assert!(CENTRAL_ANGLE_SQUARE == LAMBDA_EUV);
    assert!(CENTRAL_ANGLE_HEPTAGON == QUAD_PRODUCT / COPRIME_TRIPLE[0]);
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

    // §14 Triangular numbers
    assert!(TRI_3 == 3 * 4 / 2);
    assert!(TRI_7 == 7 * 8 / 2);
    assert!(TRI_10 == 10 * 11 / 2);
    assert!(TRI_13 == 13 * 14 / 2);
    assert!(TRI_7 == CYCLIC_ORDER);
    assert!(TRI_13 == LAMBDA_EUV);
    assert!(2 * TRI_10 + 1 == RADIUS_NUM);

    // §15 Torus knot parameters
    assert!(7 * (11 - 1) == 70);
    assert!(7 * (13 - 1) == 84);
    assert!(7 * (15 - 1) == 98);
    assert!(11 * (13 - 1) == 132);
    assert!(CROSSING_11_14 == 11 * (14 - 1));
    assert!(CROSSING_11_14 == BEZIER_C650_ANGLE);
    assert!(11 * (15 - 1) == 154);
    assert!(CROSSING_13_14 == 13 * (14 - 1));
    assert!(CROSSING_13_14 == REPUNIT_3 * REPUNIT_3);
    assert!(CROSSING_13_15 == 13 * (15 - 1));
    assert!(CROSSING_13_15 == ARC_ROOT_SEMI);
    assert!(CROSSING_14_15 == 14 * (15 - 1));
    assert!(CROSSING_14_15 == ROOT_X1 * ROOT_X1);
    // Excluded pair: gcd(7, 14) = 7 ≠ 1 → (7,14) is NOT a valid torus knot.

    // §17 Plenum square scaling
    assert!(PLENUM_SQUARE_STEP == 2 * COPRIME_TRIPLE[1]);
    assert!(PLENUM_SQUARE_STEP == LAMBDA_UVB / REPUNIT_3);
    assert!(PLENUM_SQUARE_STEP * NULL_CHANNEL_MOD == 88);
    assert!(PLENUM_SQUARE_MIN + PLENUM_SQUARE_STEP * NULL_CHANNEL_MOD == CENTER);
    assert!(PLENUM_SQUARE_MIN == ROOT_X2 - TERNARY_BASE);
    assert!(MAGIC_CONSTANT == CENTER + (CENTER - PLENUM_SQUARE_STEP) + (CENTER + PLENUM_SQUARE_STEP));
};

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    #[test]
    fn full_circle_bound() {
        assert_eq!(FULL_CIRCLE_DEG, RADIAN_DEG * TWO_PI_TERNARY);
        assert_eq!(364.0, 13.0 * 28.0);
    }

    #[test]
    fn pi_bound() {
        assert_eq!(PI_TERNARY, TWO_PI_TERNARY / 2.0);
    }

    #[test]
    fn cyclic_order_matches() {
        assert_eq!(CYCLIC_ORDER, RADIANS_PER_CIRCLE);
        assert_eq!(CYCLIC_ORDER, Z28_DIMENSIONS);
    }

    #[test]
    fn tau_identity() {
        let tau = TAU_TRIBONACCI;
        let tau_cubed = tau * tau * tau;
        let tau_sq_plus_tau_plus_1 = TAU_SQUARED + TAU_TRIBONACCI + 1.0;
        assert!((tau_cubed - tau_sq_plus_tau_plus_1).abs() < 1e-8,
            "τ³ should equal τ² + τ + 1");
    }

    #[test]
    fn conversion_round_trip_deg() {
        let std = ternary_deg_to_std_deg(FULL_CIRCLE_DEG);
        assert!((std - 360.0).abs() < 1e-10);
        let back = std_deg_to_ternary_deg(std);
        assert!((back - FULL_CIRCLE_DEG).abs() < 1e-10);
    }

    #[test]
    fn conversion_round_trip_rad() {
        let std = ternary_rad_to_std_rad(TWO_PI_TERNARY);
        assert!((std - 2.0 * STD_PI).abs() < 1e-10);
        let back = std_rad_to_ternary_rad(std);
        assert!((back - TWO_PI_TERNARY).abs() < 1e-10);
    }

    #[test]
    fn radian_conversion() {
        assert_eq!(ternary_rad_to_ternary_deg(1.0), RADIAN_DEG);
        assert_eq!(ternary_deg_to_ternary_rad(RADIAN_DEG), 1.0);
    }

    #[test]
    fn trit_walk_angles() {
        assert_eq!(trit_to_walk_angle_deg(0), WALK_TURN_0);
        assert_eq!(trit_to_walk_angle_deg(1), WALK_TURN_1);
        assert_eq!(trit_to_walk_angle_deg(2), WALK_TURN_2);
    }

    #[test]
    fn repunit_formula_verification() {
        for (n, expected) in [(1, 1), (2, 4), (3, 13), (4, 40), (5, 121), (6, 364)] {
            assert_eq!(repunit(n), expected, "repunit({}) failed", n);
        }
    }

    #[test]
    fn coprime_sextuple_max_pairwise() {
        let s = SEXTUPLE_MAX;
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_eq!(gcd(s[i], s[j]), 1,
                    "SEXTUPLE_MAX: gcd({}, {}) != 1", s[i], s[j]);
            }
        }
    }

    #[test]
    fn coprime_sextuple_a_pairwise() {
        let s = SEXTUPLE_A;
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_eq!(gcd(s[i], s[j]), 1,
                    "SEXTUPLE_A: gcd({}, {}) != 1", s[i], s[j]);
            }
        }
    }

    #[test]
    fn coprime_sextuple_b_pairwise() {
        let s = SEXTUPLE_B;
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_eq!(gcd(s[i], s[j]), 1,
                    "SEXTUPLE_B: gcd({}, {}) != 1", s[i], s[j]);
            }
        }
    }

    #[test]
    fn coprime_sextuple_c_pairwise() {
        let s = SEXTUPLE_C;
        for i in 0..6 {
            for j in (i + 1)..6 {
                assert_eq!(gcd(s[i], s[j]), 1,
                    "SEXTUPLE_C: gcd({}, {}) != 1", s[i], s[j]);
            }
        }
    }

    #[test]
    fn hash_coefficients_a_coprime() {
        for &(m, c) in &HASH_COEFF_A {
            assert_eq!(gcd(m, c), 1, "HASH_COEFF_A: gcd({}, {}) != 1", m, c);
            assert_ne!(c % m, 0, "HASH_COEFF_A: {} % {} == 0", c, m);
        }
    }

    #[test]
    fn hash_coefficients_b_coprime() {
        for &(m, c) in &HASH_COEFF_B {
            assert_eq!(gcd(m, c), 1, "HASH_COEFF_B: gcd({}, {}) != 1", m, c);
            assert_ne!(c % m, 0, "HASH_COEFF_B: {} % {} == 0", c, m);
        }
    }

    #[test]
    fn hash_coefficients_sext_coprime() {
        for &(m, c) in &HASH_COEFF_SEXT {
            assert_eq!(gcd(m, c), 1, "HASH_COEFF_SEXT: gcd({}, {}) != 1", m, c);
            assert_ne!(c % m, 0, "HASH_COEFF_SEXT: {} % {} == 0", c, m);
        }
    }

    #[test]
    fn gcd_of_primaries_is_radian() {
        let g = gcd(gcd(LAMBDA_EUV, LAMBDA_UVC), gcd(LAMBDA_UVB, LAMBDA_UVA));
        assert_eq!(g, 13, "GCD of primary UV wavelengths must be 13 (radian)");
    }

    #[test]
    fn superhub_near_unit_circle() {
        let dist_ab = (SUPERHUB_X_LEFT * SUPERHUB_X_LEFT + SUPERHUB_Y_AB * SUPERHUB_Y_AB).sqrt();
        assert!(dist_ab > 0.96 && dist_ab < 0.98,
            "Zone A/B distance from origin: {}", dist_ab);
        let dist_cd = (SUPERHUB_X_RIGHT * SUPERHUB_X_RIGHT + SUPERHUB_Y_CD * SUPERHUB_Y_CD).sqrt();
        assert!(dist_cd > 0.96 && dist_cd < 0.98,
            "Zone C/D distance from origin: {}", dist_cd);
    }

    #[test]
    fn excluded_pair_not_coprime() {
        assert_eq!(gcd(EXCLUDED_PAIR.0, EXCLUDED_PAIR.1), 7);
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
            assert!((cum - target).abs() < 0.001,
                "Cumulative energy at n={}: {} vs target {}", n, cum, target);
        }
        // Extended: n=1..7 skipping n=4 null
        for &n in &[5u32, 6, 7] {
            cum += f(n);
        }
        assert!((cum - 0.933).abs() < 0.001,
            "Cumulative energy at n=1..7: {} vs target 0.933", cum);
    }

    #[test]
    fn bridge_coeff_round_trip() {
        let bc = STD_PI / ROOT_X1 as f64;
        assert!((BRIDGE_COEFF - bc).abs() < 1e-15);
        assert!((BRIDGE_COEFF * CYCLIC_ORDER as f64 - 2.0 * STD_PI).abs() < 1e-10);
    }
}
