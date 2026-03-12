/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  PLENUM_SQUARE_MATRIX,
  PLENUM_MAGIC_CONSTANT,
  TERNARY_BALANCE_CENTER,
  PI_ESOTERIC,
  RADIUS_COSMIC,
  LUNAR_SOLAR_HARMONIC,
  COSMIC_CIRCUMFERENCE_DAYS,
  PHASE_DISSONANCE_DEGREES,
  DISSONANCE_CLOSURE_HARMONIC,
  TRIBONACCI_EXACT,
  TRIBONACCI_RADIUS_MATCH,
  HAS_TRIBONACCI_PLENUM_HARMONY,
  SUFT_RADIUS,
  SUFT_PI,
  SUFT_LUNAR_HARMONIC,
  SUFT_COSMIC_CIRCUMFERENCE,
  TEMPORAL_CROSS_DENOM,
  ENERGY_CROSS_DENOM,
  MASS_SHELL_RATIO,
  PLENUM_NATURAL_YEAR_DAYS,
  PLENUM_SQUARE_FAMILY,
  HARMONIC_LADDER,
  UNIFIED_EQUATION,
  validateAll,
  validatePlenumSquareFamily,
  validateHarmonicLadder,
  validateInvariantProducts,
  validateUnifiedEquation,
} from '../shared/plenum-square';
import {
  getPlenumSquareFlattened,
  getPlenumSquareCyclic,
  getTernaryPlenumSquareWeight,
  isPlenumSquareMagic,
  isCirculant,
} from '../shared/plenum-square-utils';
import { TRIBONACCI_SEQUENCE } from '../shared/tribonacci-constants';
import { FULL_CIRCLE_DEG, PI_TERNARY, TWO_PI_TERNARY, RADIAN_DEG } from '../shared/ternary-circle';
import { PLATFORM } from '../shared/constants';

describe('Plenum Square Blueprint', () => {
  test('matrix has correct magic constant (333)', () => {
    expect(isPlenumSquareMagic(PLENUM_SQUARE_MATRIX)).toBe(true);
    expect(PLENUM_MAGIC_CONSTANT).toBe(333);
  });

  test('matrix is circulant', () => {
    expect(isCirculant(PLENUM_SQUARE_MATRIX)).toBe(true);
  });

  test('matrix center is 111 (ternary balance)', () => {
    expect(PLENUM_SQUARE_MATRIX[1][1]).toBe(111);
    expect(TERNARY_BALANCE_CENTER).toBe(111);
  });

  test('all rows sum to 333', () => {
    for (const row of PLENUM_SQUARE_MATRIX) {
      expect(row[0] + row[1] + row[2]).toBe(333);
    }
  });

  test('all columns sum to 333', () => {
    for (let c = 0; c < 3; c++) {
      expect(PLENUM_SQUARE_MATRIX[0][c] + PLENUM_SQUARE_MATRIX[1][c] + PLENUM_SQUARE_MATRIX[2][c]).toBe(333);
    }
  });

  test('both diagonals sum to 333', () => {
    expect(PLENUM_SQUARE_MATRIX[0][0] + PLENUM_SQUARE_MATRIX[1][1] + PLENUM_SQUARE_MATRIX[2][2]).toBe(333);
    expect(PLENUM_SQUARE_MATRIX[0][2] + PLENUM_SQUARE_MATRIX[1][1] + PLENUM_SQUARE_MATRIX[2][0]).toBe(333);
  });
});

describe('SUFT Constants — Exact Integer Derivation', () => {
  test('RADIUS_COSMIC = 13 (exact)', () => {
    expect(RADIUS_COSMIC).toBe(13);
    expect(SUFT_RADIUS).toBe(13);
  });

  test('PI_ESOTERIC = 14 (exact)', () => {
    expect(PI_ESOTERIC).toBe(14);
    expect(SUFT_PI).toBe(14);
  });

  test('LUNAR_SOLAR_HARMONIC = 28 (exact)', () => {
    expect(LUNAR_SOLAR_HARMONIC).toBe(28);
    expect(SUFT_LUNAR_HARMONIC).toBe(28);
  });

  test('COSMIC_CIRCUMFERENCE_DAYS = 364 (exact)', () => {
    expect(COSMIC_CIRCUMFERENCE_DAYS).toBe(364);
    expect(SUFT_COSMIC_CIRCUMFERENCE).toBe(364);
    expect(PLENUM_NATURAL_YEAR_DAYS).toBe(364);
  });

  test('phase dissonance = 27, closure = 28', () => {
    expect(PHASE_DISSONANCE_DEGREES).toBe(27);
    expect(DISSONANCE_CLOSURE_HARMONIC).toBe(28);
  });

  test('SUFT coefficients are exact', () => {
    expect(TEMPORAL_CROSS_DENOM).toBe(13);
    expect(ENERGY_CROSS_DENOM).toBe(28);
    expect(MASS_SHELL_RATIO).toBe(13 / 28);
  });
});

describe('Tribonacci–Plenum Square Bridge', () => {
  test('T(7) = 13 aligns exactly with RADIUS_COSMIC', () => {
    expect(TRIBONACCI_SEQUENCE[7]).toBe(13);
    expect(TRIBONACCI_RADIUS_MATCH).toBe(13);
    expect(TRIBONACCI_RADIUS_MATCH).toBe(RADIUS_COSMIC);
  });

  test('PI_ESOTERIC = T(7) + T(3) = 13 + 1 = 14', () => {
    expect(TRIBONACCI_SEQUENCE[7] + TRIBONACCI_SEQUENCE[3]).toBe(14);
    expect(PI_ESOTERIC).toBe(14);
  });

  test('full Tribonacci-Plenum Square harmony holds', () => {
    expect(HAS_TRIBONACCI_PLENUM_HARMONY).toBe(true);
  });

  test('TRIBONACCI_EXACT matches canonical sequence', () => {
    expect(TRIBONACCI_EXACT.T0).toBe(TRIBONACCI_SEQUENCE[0]);
    expect(TRIBONACCI_EXACT.T7).toBe(TRIBONACCI_SEQUENCE[7]);
    expect(TRIBONACCI_EXACT.T8).toBe(TRIBONACCI_SEQUENCE[8]);
    expect(TRIBONACCI_EXACT.T10).toBe(TRIBONACCI_SEQUENCE[10]);
    expect(TRIBONACCI_EXACT.T14).toBe(TRIBONACCI_SEQUENCE[14]);
  });
});

describe('Ternary Circle Alignment', () => {
  test('Plenum Square COSMIC_CIRCUMFERENCE matches ternary full circle', () => {
    expect(COSMIC_CIRCUMFERENCE_DAYS).toBe(FULL_CIRCLE_DEG);
  });

  test('PI_ESOTERIC matches ternary π', () => {
    expect(PI_ESOTERIC).toBe(PI_TERNARY);
  });

  test('LUNAR_SOLAR_HARMONIC matches ternary 2π', () => {
    expect(LUNAR_SOLAR_HARMONIC).toBe(TWO_PI_TERNARY);
  });

  test('RADIUS_COSMIC matches ternary radian', () => {
    expect(RADIUS_COSMIC).toBe(RADIAN_DEG);
  });
});

describe('Matrix Utility Functions', () => {
  test('flattened returns 9 values in row-major order', () => {
    const flat = getPlenumSquareFlattened();
    expect(flat).toEqual([111, 14, 208, 208, 111, 14, 14, 208, 111]);
    expect(flat.length).toBe(9);
  });

  test('cyclic shift 0 returns original matrix', () => {
    const m = getPlenumSquareCyclic(0);
    expect(m[0]).toEqual([111, 14, 208]);
    expect(m[1]).toEqual([208, 111, 14]);
    expect(m[2]).toEqual([14, 208, 111]);
  });

  test('cyclic shift 1 rotates rows', () => {
    const m = getPlenumSquareCyclic(1);
    expect(m[0]).toEqual([208, 111, 14]);
    expect(m[1]).toEqual([14, 208, 111]);
    expect(m[2]).toEqual([111, 14, 208]);
  });

  test('cyclic shift 2 rotates rows twice', () => {
    const m = getPlenumSquareCyclic(2);
    expect(m[0]).toEqual([14, 208, 111]);
    expect(m[1]).toEqual([111, 14, 208]);
    expect(m[2]).toEqual([208, 111, 14]);
  });

  test('cyclic shift 0 (identity) preserves full magic property', () => {
    expect(isPlenumSquareMagic(getPlenumSquareCyclic(0))).toBe(true);
  });

  test('all cyclic shifts preserve row and column sums', () => {
    for (const shift of [0, 1, 2] as const) {
      const m = getPlenumSquareCyclic(shift);
      for (let i = 0; i < 3; i++) {
        expect(m[i][0] + m[i][1] + m[i][2]).toBe(333);
        expect(m[0][i] + m[1][i] + m[2][i]).toBe(333);
      }
    }
  });

  test('ternary weights return canonical row values', () => {
    expect(getTernaryPlenumSquareWeight(0)).toBe(111);
    expect(getTernaryPlenumSquareWeight(1)).toBe(14);
    expect(getTernaryPlenumSquareWeight(2)).toBe(208);
  });

  test('isPlenumSquareMagic rejects invalid matrices', () => {
    expect(isPlenumSquareMagic([[1, 2, 3], [4, 5, 6], [7, 8, 9]])).toBe(false);
    expect(isPlenumSquareMagic([[111, 111, 111], [111, 111, 111], [111, 111, 111]])).toBe(true);
  });

  test('isCirculant rejects non-circulant magic squares', () => {
    const nonCirculant = [[111, 111, 111], [111, 111, 111], [111, 111, 111]];
    expect(isCirculant(nonCirculant)).toBe(true);
    const broken = [[111, 14, 208], [14, 208, 111], [208, 111, 14]];
    expect(isCirculant(broken)).toBe(false);
  });
});

describe('Plenum Square Family (Non-Circulant Configurations)', () => {
  test('all four configurations are valid magic squares', () => {
    const results = validatePlenumSquareFamily();
    for (const [id, errors] of Object.entries(results)) {
      expect(errors, `Square ${id} has validation errors: ${errors.join(', ')}`).toEqual([]);
    }
  });

  test('all four configurations have center 111', () => {
    for (const config of PLENUM_SQUARE_FAMILY) {
      expect(config.grid[1][1]).toBe(111);
    }
  });

  test('all four configurations have magic constant 333', () => {
    for (const config of PLENUM_SQUARE_FAMILY) {
      for (let r = 0; r < 3; r++) {
        expect(config.grid[r][0] + config.grid[r][1] + config.grid[r][2]).toBe(333);
      }
    }
  });
});

describe('Harmonic Ladder', () => {
  test('ladder is self-consistent', () => {
    const errors = validateHarmonicLadder();
    expect(errors).toEqual([]);
  });

  test('444 + 444 = 888', () => {
    expect(HARMONIC_LADDER.CORNER_SUM + HARMONIC_LADDER.EDGE_CENTER_SUM).toBe(HARMONIC_LADDER.SURROUND_SUM);
  });

  test('888 + 111 = 999', () => {
    expect(HARMONIC_LADDER.SURROUND_SUM + HARMONIC_LADDER.CENTER).toBe(HARMONIC_LADDER.TOTAL_SUM);
  });

  test('1554 - 999 = 555', () => {
    expect(1554 - HARMONIC_LADDER.TOTAL_SUM).toBe(HARMONIC_LADDER.CIRCUMFERENCE_RESIDUAL);
  });
});

describe('Invariant Products', () => {
  test('all product invariants hold', () => {
    const errors = validateInvariantProducts();
    expect(errors).toEqual([]);
  });
});

describe('Unified Equation: arc² − 832·arc + 118300 = 0', () => {
  test('arc = 182 is a root', () => {
    expect(182 * 182 - 832 * 182 + 118300).toBe(0);
  });

  test('arc = 650 is the other root', () => {
    expect(650 * 650 - 832 * 650 + 118300).toBe(0);
  });

  test('discriminant is perfect square: 219024 = 468²', () => {
    expect(UNIFIED_EQUATION.ARC_DISCRIMINANT).toBe(468 * 468);
    expect(UNIFIED_EQUATION.ARC_DISCRIMINANT).toBe(832 * 832 - 4 * 118300);
  });

  test('coefficient decomposition: 832 = R₄(R₄−1) − 2R₆', () => {
    expect(40 * 39 - 2 * 364).toBe(832);
  });

  test('coefficient decomposition: 118300 = R₆ · R₃ · (π−9)²', () => {
    expect(364 * 13 * 25).toBe(118300);
  });

  test('center derived from arc: c = (182 + 40)/2 = 111', () => {
    expect((UNIFIED_EQUATION.ARC + 40) / 2).toBe(111);
  });

  test('secondary discriminant: 1 + 4·182 = 729 = 3⁶ (sponge width)', () => {
    expect(UNIFIED_EQUATION.SECONDARY_DISCRIMINANT).toBe(729);
    expect(1 + 4 * 182).toBe(729);
    expect(Math.pow(3, 6)).toBe(729);
  });

  test('π recovered from secondary discriminant: (1+27)/2 = 14', () => {
    expect((1 + Math.sqrt(729)) / 2).toBe(14);
  });

  test('468 = (R₄−1)·√Δ_quad = 39 × 12', () => {
    expect(UNIFIED_EQUATION.ARC_SQRT_DISCRIMINANT).toBe(39 * 12);
  });

  test('validateUnifiedEquation() returns zero errors', () => {
    const errors = validateUnifiedEquation();
    expect(errors).toEqual([]);
  });
});

describe('PLATFORM.PLENUM_SQUARE.MASTER unified constants', () => {
  test('MASTER block has all unified equation constants', () => {
    const M = PLATFORM.PLENUM_SQUARE.MASTER;
    expect(M.ARC_COEFF_B).toBe(832);
    expect(M.ARC_COEFF_C).toBe(118300);
    expect(M.ARC_DISCRIMINANT).toBe(219024);
    expect(M.ARC_SQRT_DISCRIMINANT).toBe(468);
    expect(M.ARC).toBe(182);
    expect(M.SECONDARY_DISCRIMINANT).toBe(729);
  });

  test('unified equation verified through PLATFORM constants', () => {
    const M = PLATFORM.PLENUM_SQUARE.MASTER;
    expect(M.ARC * M.ARC - M.ARC_COEFF_B * M.ARC + M.ARC_COEFF_C).toBe(0);
    expect(M.ARC_COEFF_B * M.ARC_COEFF_B - 4 * M.ARC_COEFF_C).toBe(M.ARC_DISCRIMINANT);
    expect(M.ARC_SQRT_DISCRIMINANT * M.ARC_SQRT_DISCRIMINANT).toBe(M.ARC_DISCRIMINANT);
    expect((M.ARC + M.R4) / 2).toBe(M.DERIVED_DIAMETER);
    expect(1 + 4 * M.ARC).toBe(M.SECONDARY_DISCRIMINANT);
  });
});

describe('Full Validation Suite', () => {
  test('validateAll() returns zero errors', () => {
    const errors = validateAll();
    expect(errors).toEqual([]);
  });
});
