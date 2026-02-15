/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * # Saturnian Magic Square Blueprint
 *
 * Static constants derived from the 3×3 Saturnian Magic Square and bridged
 * to the existing Tribonacci sequence (shared/tribonacci-constants.ts).
 *
 * ## The Matrix
 *
 *   | 111 |  14 | 208 |
 *   | 208 | 111 |  14 |
 *   |  14 | 208 | 111 |
 *
 * Every row, column, and diagonal sums to **333** (the magic constant).
 * The matrix is a circulant — each row is a cyclic permutation of [111, 14, 208].
 *
 * ## Tribonacci Alignment (Exact Integer)
 *
 * - RADIUS_COSMIC = 208 / 16 = **13** = T(7) (seventh Tribonacci number)
 * - PI_ESOTERIC = **14** = T(7) + T(3) = 13 + 1 (matches ternary π)
 * - LUNAR_SOLAR_HARMONIC = 2 × 14 = **28** (matches Z₂₈ cyclic order)
 * - COSMIC_CIRCUMFERENCE = 28 × 13 = **364** (matches ternary full circle)
 *
 * These are exact integer identities — no rounding or floating-point involved.
 *
 * ## Design
 *
 * All values are compile-time constants or trivial static arithmetic.
 * Zero runtime overhead. Import and use directly in timing, crypto seeding,
 * VM register weighting, or calendar modules.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { TRIBONACCI_SEQUENCE } from './tribonacci-constants';

export const SATURNIAN_MATRIX = [
  [111, 14, 208],
  [208, 111, 14],
  [14, 208, 111],
] as const;

export const MAGIC_CONSTANT = 333;

export const TERNARY_BALANCE_CENTER = 111;

export const PI_ESOTERIC = 14;

export const RADIUS_COSMIC = 208 / 16; // → 13

export const LUNAR_SOLAR_HARMONIC = 2 * PI_ESOTERIC; // → 28

export const COSMIC_CIRCUMFERENCE_DAYS = LUNAR_SOLAR_HARMONIC * RADIUS_COSMIC; // → 364

export const PHASE_DISSONANCE_DEGREES = 360 - MAGIC_CONSTANT; // → 27

export const DISSONANCE_CLOSURE_HARMONIC = PHASE_DISSONANCE_DEGREES + 1; // → 28

export const TRIBONACCI_EXACT = {
  T0: 0,
  T1: 0,
  T2: 1,
  T3: 1,
  T4: 2,
  T5: 4,
  T6: 7,
  T7: 13,
  T8: 24,
  T9: 44,
  T10: 81,
  T11: 149,
  T12: 274,
  T13: 504,
  T14: 927,
} as const;

export const TRIBONACCI_RADIUS_MATCH = TRIBONACCI_SEQUENCE[7]; // 13 — aligns exactly with RADIUS_COSMIC

export const HAS_TRIBONACCI_SATURNIAN_HARMONY =
  RADIUS_COSMIC === TRIBONACCI_SEQUENCE[7] &&
  PI_ESOTERIC === TRIBONACCI_SEQUENCE[7] + TRIBONACCI_SEQUENCE[3] &&
  LUNAR_SOLAR_HARMONIC === 28 &&
  COSMIC_CIRCUMFERENCE_DAYS === 364;

export const SUFT_RADIUS = TRIBONACCI_EXACT.T7;                               // 13 exact
export const SUFT_PI = SUFT_RADIUS + TRIBONACCI_EXACT.T3;                     // 14 exact
export const SUFT_LUNAR_HARMONIC = 2 * SUFT_PI;                               // 28 exact
export const SUFT_COSMIC_CIRCUMFERENCE = SUFT_LUNAR_HARMONIC * SUFT_RADIUS;   // 364 exact

export const TEMPORAL_CROSS_DENOM = SUFT_RADIUS;                              // 13
export const ENERGY_CROSS_DENOM = SUFT_LUNAR_HARMONIC;                        // 28
export const MASS_SHELL_RATIO = SUFT_RADIUS / SUFT_LUNAR_HARMONIC;            // 13/28

export const SATURNIAN_NATURAL_YEAR_DAYS = COSMIC_CIRCUMFERENCE_DAYS;          // 364
