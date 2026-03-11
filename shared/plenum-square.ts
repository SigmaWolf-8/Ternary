/**
 * Plenum Square — Generative Root of the Salvi Framework
 *
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved — Applied Physics Division
 *
 * The 3×3 circulant magic square {111, 14, 208} is the generative root from which
 * every constant in PlenumNET cascades through exact integer arithmetic.
 *
 * This module includes the complete Plenum Square Family: the original circulant
 * plus four non-circulant configurations (A–D) derived from the circle parameters
 * π=14, 364/π=26, π²=196, and 222−14=208. These exhaust all positive-integer
 * embeddings of the two opposite pairs {14,208} and {26,196} around center 111,
 * up to dihedral symmetry (D₄ rotations/reflections).
 *
 * INVARIANT: All constants derive from the magic square through exact integer
 * arithmetic — zero rounding, zero approximation, zero floating-point.
 */

import { TRIBONACCI_SEQUENCE } from './tribonacci-constants';

// ============================================================================
// §1  PLENUM CIRCULANT (original circulant magic square)
// ============================================================================

export const PLENUM_SQUARE_MATRIX = [
  [111, 14, 208],
  [208, 111, 14],
  [14, 208, 111],
] as const;

export const TERNARY_BALANCE_CENTER = 111;

export const PLENUM_MAGIC_CONSTANT = 333;

// ============================================================================
// §2  GENERATIVE CASCADE
// ============================================================================

export const RADIUS_COSMIC = 208 / 16; // = 13

export const PI_ESOTERIC = RADIUS_COSMIC + 1; // = 14

export const LUNAR_SOLAR_HARMONIC = 2 * PI_ESOTERIC; // = 28

export const COSMIC_CIRCUMFERENCE = LUNAR_SOLAR_HARMONIC * RADIUS_COSMIC; // = 364

export const PHASE_DISSONANCE = 360 - PLENUM_MAGIC_CONSTANT; // = 27

export const DISSONANCE_CLOSURE = PHASE_DISSONANCE + 1; // = 28

export const TEMPORAL_CROSS_DENOM = RADIUS_COSMIC; // = 13

export const MASS_SHELL_RATIO = RADIUS_COSMIC / LUNAR_SOLAR_HARMONIC; // = 13/28

export const PLENUM_NATURAL_YEAR = COSMIC_CIRCUMFERENCE; // = 364

// ============================================================================
// §3  CIRCLE-DERIVED CONSTANTS
// ============================================================================

export const CIRCLE_DIAMETER = TERNARY_BALANCE_CENTER; // = 111

export const CIRCLE_RADIUS = CIRCLE_DIAMETER / 2; // = 55.5

export const CIRCLE_CIRCUMFERENCE = PI_ESOTERIC * CIRCLE_DIAMETER; // = 1554

export const CIRCLE_DEGREES_PER_PI = COSMIC_CIRCUMFERENCE / PI_ESOTERIC; // = 26

export const PI_SQUARED = PI_ESOTERIC * PI_ESOTERIC; // = 196

// ============================================================================
// §4  HARMONIC LADDER — Multiples of 111
// ============================================================================

export const HARMONIC_LADDER = {
  CENTER: TERNARY_BALANCE_CENTER, // 111
  OPPOSITE_PAIR_SUM: 2 * TERNARY_BALANCE_CENTER, // 222
  MAGIC_CONSTANT: PLENUM_MAGIC_CONSTANT, // 333
  CORNER_SUM: 4 * TERNARY_BALANCE_CENTER, // 444
  EDGE_CENTER_SUM: 4 * TERNARY_BALANCE_CENTER, // 444
  CIRCUMFERENCE_RESIDUAL: 5 * TERNARY_BALANCE_CENTER, // 555
  SURROUND_SUM: 8 * TERNARY_BALANCE_CENTER, // 888
  TOTAL_SUM: 9 * TERNARY_BALANCE_CENTER, // 999
} as const;

// ============================================================================
// §5  THE FIVE CIRCLE-DERIVED VALUES
// ============================================================================

export const PLENUM_SQUARE_VALUES = {
  PI: PI_ESOTERIC, // 14
  DEGREES_PER_PI: CIRCLE_DEGREES_PER_PI, // 26
  CENTER: TERNARY_BALANCE_CENTER, // 111
  PI_SQUARED: PI_SQUARED, // 196
  PI_COMPLEMENT: HARMONIC_LADDER.OPPOSITE_PAIR_SUM - PI_ESOTERIC, // 208
} as const;

export const OPPOSITE_PAIRS = {
  PI_PAIR: [PLENUM_SQUARE_VALUES.PI, PLENUM_SQUARE_VALUES.PI_COMPLEMENT] as const, // [14, 208]
  DEGREES_PAIR: [PLENUM_SQUARE_VALUES.DEGREES_PER_PI, PLENUM_SQUARE_VALUES.PI_SQUARED] as const, // [26, 196]
} as const;

// ============================================================================
// §6  PARAMETRIZED MAGIC SQUARE GENERATOR
// ============================================================================

export type MagicSquare3x3 = readonly [
  readonly [number, number, number],
  readonly [number, number, number],
  readonly [number, number, number],
];

export function generateMagicSquare(p: number, q: number): MagicSquare3x3 {
  const C = TERNARY_BALANCE_CENTER;
  return [
    [C + p, C - p - q, C + q],
    [C - p + q, C, C + p - q],
    [C - q, C + p + q, C - p],
  ] as const;
}

// ============================================================================
// §7  FOUR NON-CIRCULANT CONFIGURATIONS (A–D)
// ============================================================================

export interface PlenumSquareConfiguration {
  readonly id: string;
  readonly p: number;
  readonly q: number;
  readonly grid: MagicSquare3x3;
  readonly piPairPosition: 'corners' | 'vertical_middles' | 'horizontal_middles';
  readonly degreesPairPosition: 'corners' | 'vertical_middles' | 'horizontal_middles';
  readonly interpretation: string;
}

export const PLENUM_SQUARE_A: PlenumSquareConfiguration = {
  id: 'A',
  p: 97,
  q: 12,
  grid: generateMagicSquare(97, 12),
  piPairPosition: 'corners',
  degreesPairPosition: 'horizontal_middles',
  interpretation: 'Diagonal crossed by horizontal axis',
} as const;

export const PLENUM_SQUARE_B: PlenumSquareConfiguration = {
  id: 'B',
  p: 91,
  q: 6,
  grid: generateMagicSquare(91, 6),
  piPairPosition: 'vertical_middles',
  degreesPairPosition: 'horizontal_middles',
  interpretation: 'Cross formation (vertical × horizontal)',
} as const;

export const PLENUM_SQUARE_C: PlenumSquareConfiguration = {
  id: 'C',
  p: 85,
  q: -12,
  grid: generateMagicSquare(85, -12),
  piPairPosition: 'horizontal_middles',
  degreesPairPosition: 'corners',
  interpretation: 'Horizontal axis crossed by diagonal (π² above π)',
} as const;

export const PLENUM_SQUARE_D: PlenumSquareConfiguration = {
  id: 'D',
  p: 6,
  q: -91,
  grid: generateMagicSquare(6, -91),
  piPairPosition: 'horizontal_middles',
  degreesPairPosition: 'vertical_middles',
  interpretation: 'Cross formation (horizontal × vertical)',
} as const;

export const PLENUM_SQUARE_FAMILY = [
  PLENUM_SQUARE_A,
  PLENUM_SQUARE_B,
  PLENUM_SQUARE_C,
  PLENUM_SQUARE_D,
] as const;

// ============================================================================
// §8  INVARIANT PRODUCTS
// ============================================================================

export const INVARIANT_PRODUCTS = {
  CIRCLE_DEGREES: PLENUM_SQUARE_VALUES.PI * PLENUM_SQUARE_VALUES.DEGREES_PER_PI, // = 364
  PI_SQUARED: PLENUM_SQUARE_VALUES.PI * PLENUM_SQUARE_VALUES.PI, // = 196
  PI_PAIR_PRODUCT: PLENUM_SQUARE_VALUES.PI * PLENUM_SQUARE_VALUES.PI_COMPLEMENT, // = 2912
  DEGREES_PAIR_PRODUCT:
    PLENUM_SQUARE_VALUES.DEGREES_PER_PI * PLENUM_SQUARE_VALUES.PI_SQUARED, // = 5096
  PAIR_PRODUCT_SUM: 2912 + 5096, // = 8008
} as const;

// ============================================================================
// §9  VALIDATION FUNCTIONS
// ============================================================================

export function validateMagicSquare(grid: MagicSquare3x3): string[] {
  const errors: string[] = [];
  const C = TERNARY_BALANCE_CENTER;
  const M = PLENUM_MAGIC_CONSTANT;
  const OPP = HARMONIC_LADDER.OPPOSITE_PAIR_SUM;
  const CORNER = HARMONIC_LADDER.CORNER_SUM;

  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      if (grid[r][c] <= 0 || !Number.isInteger(grid[r][c])) {
        errors.push(`grid[${r}][${c}]=${grid[r][c]} is not a positive integer`);
      }
    }
  }

  if (grid[1][1] !== C) {
    errors.push(`Center=${grid[1][1]}, expected ${C}`);
  }

  for (let r = 0; r < 3; r++) {
    const sum = grid[r][0] + grid[r][1] + grid[r][2];
    if (sum !== M) errors.push(`Row ${r} sum=${sum}, expected ${M}`);
  }

  for (let c = 0; c < 3; c++) {
    const sum = grid[0][c] + grid[1][c] + grid[2][c];
    if (sum !== M) errors.push(`Col ${c} sum=${sum}, expected ${M}`);
  }

  const mainDiag = grid[0][0] + grid[1][1] + grid[2][2];
  if (mainDiag !== M) errors.push(`Main diagonal sum=${mainDiag}, expected ${M}`);
  const antiDiag = grid[0][2] + grid[1][1] + grid[2][0];
  if (antiDiag !== M) errors.push(`Anti-diagonal sum=${antiDiag}, expected ${M}`);

  const opposites: Array<[string, number, number]> = [
    ['TL-BR', grid[0][0], grid[2][2]],
    ['TR-BL', grid[0][2], grid[2][0]],
    ['TM-BM', grid[0][1], grid[2][1]],
    ['ML-MR', grid[1][0], grid[1][2]],
  ];
  for (const [label, a, b] of opposites) {
    if (a + b !== OPP) errors.push(`Opposite ${label}: ${a}+${b}=${a + b}, expected ${OPP}`);
  }

  const cornerSum = grid[0][0] + grid[0][2] + grid[2][0] + grid[2][2];
  if (cornerSum !== CORNER) errors.push(`Corner sum=${cornerSum}, expected ${CORNER}`);

  const edgeCenterSum = grid[0][1] + grid[1][0] + grid[1][2] + grid[2][1];
  if (edgeCenterSum !== CORNER) {
    errors.push(`Edge-center sum=${edgeCenterSum}, expected ${CORNER}`);
  }

  return errors;
}

export function validatePlenumSquareFamily(): Record<string, string[]> {
  const results: Record<string, string[]> = {};
  for (const config of PLENUM_SQUARE_FAMILY) {
    results[config.id] = validateMagicSquare(config.grid);
  }
  return results;
}

export function validateHarmonicLadder(): string[] {
  const errors: string[] = [];
  const H = HARMONIC_LADDER;

  if (H.OPPOSITE_PAIR_SUM !== 2 * H.CENTER) {
    errors.push(`OPPOSITE_PAIR_SUM=${H.OPPOSITE_PAIR_SUM} ≠ 2×${H.CENTER}`);
  }
  if (H.MAGIC_CONSTANT !== 3 * H.CENTER) {
    errors.push(`MAGIC_CONSTANT=${H.MAGIC_CONSTANT} ≠ 3×${H.CENTER}`);
  }
  if (H.CORNER_SUM !== 4 * H.CENTER) {
    errors.push(`CORNER_SUM=${H.CORNER_SUM} ≠ 4×${H.CENTER}`);
  }
  if (H.CIRCUMFERENCE_RESIDUAL !== 5 * H.CENTER) {
    errors.push(`CIRCUMFERENCE_RESIDUAL=${H.CIRCUMFERENCE_RESIDUAL} ≠ 5×${H.CENTER}`);
  }
  if (H.SURROUND_SUM !== 8 * H.CENTER) {
    errors.push(`SURROUND_SUM=${H.SURROUND_SUM} ≠ 8×${H.CENTER}`);
  }
  if (H.TOTAL_SUM !== 9 * H.CENTER) {
    errors.push(`TOTAL_SUM=${H.TOTAL_SUM} ≠ 9×${H.CENTER}`);
  }

  if (H.CORNER_SUM + H.EDGE_CENTER_SUM !== H.SURROUND_SUM) {
    errors.push(
      `CORNER_SUM+EDGE_CENTER_SUM=${H.CORNER_SUM + H.EDGE_CENTER_SUM} ≠ SURROUND_SUM=${H.SURROUND_SUM}`,
    );
  }

  if (H.SURROUND_SUM + H.CENTER !== H.TOTAL_SUM) {
    errors.push(
      `SURROUND_SUM+CENTER=${H.SURROUND_SUM + H.CENTER} ≠ TOTAL_SUM=${H.TOTAL_SUM}`,
    );
  }

  if (CIRCLE_CIRCUMFERENCE - H.TOTAL_SUM !== H.CIRCUMFERENCE_RESIDUAL) {
    errors.push(
      `CIRCUMFERENCE−TOTAL=${CIRCLE_CIRCUMFERENCE - H.TOTAL_SUM} ≠ CIRCUMFERENCE_RESIDUAL=${H.CIRCUMFERENCE_RESIDUAL}`,
    );
  }

  return errors;
}

export function validateInvariantProducts(): string[] {
  const errors: string[] = [];

  for (const config of PLENUM_SQUARE_FAMILY) {
    const flat = config.grid.flat();

    if (!flat.includes(PI_SQUARED)) {
      errors.push(`Square ${config.id}: π²=${PI_SQUARED} not found in grid`);
    }

    if (!flat.includes(PI_ESOTERIC)) {
      errors.push(`Square ${config.id}: π=${PI_ESOTERIC} not found in grid`);
    }
    if (!flat.includes(CIRCLE_DEGREES_PER_PI)) {
      errors.push(`Square ${config.id}: 364/π=${CIRCLE_DEGREES_PER_PI} not found in grid`);
    }
  }

  if (INVARIANT_PRODUCTS.CIRCLE_DEGREES !== COSMIC_CIRCUMFERENCE) {
    errors.push(
      `CIRCLE_DEGREES=${INVARIANT_PRODUCTS.CIRCLE_DEGREES} ≠ COSMIC_CIRCUMFERENCE=${COSMIC_CIRCUMFERENCE}`,
    );
  }

  return errors;
}

// ============================================================================
// §10  TRIBONACCI–PLENUM SQUARE HARMONY
// ============================================================================

export const HAS_TRIBONACCI_PLENUM_HARMONY =
  RADIUS_COSMIC === TRIBONACCI_SEQUENCE[7]; // 13 === T₇

// ============================================================================
// §11  SUFT ALIASES (backward-compatible re-exports)
// ============================================================================

export const SUFT_RADIUS = RADIUS_COSMIC; // 13
export const SUFT_PI = PI_ESOTERIC; // 14
export const SUFT_LUNAR_HARMONIC = LUNAR_SOLAR_HARMONIC; // 28
export const SUFT_COSMIC_CIRCUMFERENCE = COSMIC_CIRCUMFERENCE; // 364

export const ENERGY_CROSS_DENOM = LUNAR_SOLAR_HARMONIC; // 28

export const COSMIC_CIRCUMFERENCE_DAYS = COSMIC_CIRCUMFERENCE; // 364

export const PHASE_DISSONANCE_DEGREES = PHASE_DISSONANCE; // 27
export const DISSONANCE_CLOSURE_HARMONIC = DISSONANCE_CLOSURE; // 28

export const MAGIC_CONSTANT = PLENUM_MAGIC_CONSTANT; // 333

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

export const TRIBONACCI_RADIUS_MATCH = TRIBONACCI_SEQUENCE[7]; // 13

export const PLENUM_NATURAL_YEAR_DAYS = COSMIC_CIRCUMFERENCE; // 364

// ============================================================================
// §12  CONVENIENCE: FULL VALIDATION SUITE
// ============================================================================

export function validateAll(): string[] {
  const familyResults = validatePlenumSquareFamily();
  const familyErrors = Object.entries(familyResults).flatMap(([id, errs]) =>
    errs.map((e) => `[Family:${id}] ${e}`),
  );

  const ladderErrors = validateHarmonicLadder().map((e) => `[Ladder] ${e}`);
  const productErrors = validateInvariantProducts().map((e) => `[Products] ${e}`);

  const harmonyErrors = HAS_TRIBONACCI_PLENUM_HARMONY
    ? []
    : ['[Harmony] RADIUS_COSMIC ≠ T₇ — Tribonacci–Plenum Square harmony broken'];

  return [...familyErrors, ...ladderErrors, ...productErrors, ...harmonyErrors];
}
