/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  SATURNIAN_MATRIX,
  MAGIC_CONSTANT,
  TERNARY_BALANCE_CENTER,
  PI_ESOTERIC,
  RADIUS_COSMIC,
  LUNAR_SOLAR_HARMONIC,
  COSMIC_CIRCUMFERENCE_DAYS,
  PHASE_DISSONANCE_DEGREES,
  DISSONANCE_CLOSURE_HARMONIC,
  TRIBONACCI_EXACT,
  TRIBONACCI_RADIUS_MATCH,
  HAS_TRIBONACCI_SATURNIAN_HARMONY,
  SUFT_RADIUS,
  SUFT_PI,
  SUFT_LUNAR_HARMONIC,
  SUFT_COSMIC_CIRCUMFERENCE,
  TEMPORAL_CROSS_DENOM,
  ENERGY_CROSS_DENOM,
  MASS_SHELL_RATIO,
  SATURNIAN_NATURAL_YEAR_DAYS,
} from '../shared/saturnian-blueprint';
import {
  getSaturnianFlattened,
  getSaturnianCyclic,
  getTernarySaturnianWeight,
  isSaturnianMagic,
  isCirculant,
} from '../shared/saturnian-matrix-utils';
import { TRIBONACCI_SEQUENCE } from '../shared/tribonacci-constants';
import { FULL_CIRCLE_DEG, PI_TERNARY, TWO_PI_TERNARY, RADIAN_DEG } from '../shared/ternary-circle';

describe('Saturnian Magic Square Blueprint', () => {
  test('matrix has correct magic constant (333)', () => {
    expect(isSaturnianMagic(SATURNIAN_MATRIX)).toBe(true);
    expect(MAGIC_CONSTANT).toBe(333);
  });

  test('matrix is circulant', () => {
    expect(isCirculant(SATURNIAN_MATRIX)).toBe(true);
  });

  test('matrix center is 111 (ternary balance)', () => {
    expect(SATURNIAN_MATRIX[1][1]).toBe(111);
    expect(TERNARY_BALANCE_CENTER).toBe(111);
  });

  test('all rows sum to 333', () => {
    for (const row of SATURNIAN_MATRIX) {
      expect(row[0] + row[1] + row[2]).toBe(333);
    }
  });

  test('all columns sum to 333', () => {
    for (let c = 0; c < 3; c++) {
      expect(SATURNIAN_MATRIX[0][c] + SATURNIAN_MATRIX[1][c] + SATURNIAN_MATRIX[2][c]).toBe(333);
    }
  });

  test('both diagonals sum to 333', () => {
    expect(SATURNIAN_MATRIX[0][0] + SATURNIAN_MATRIX[1][1] + SATURNIAN_MATRIX[2][2]).toBe(333);
    expect(SATURNIAN_MATRIX[0][2] + SATURNIAN_MATRIX[1][1] + SATURNIAN_MATRIX[2][0]).toBe(333);
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
    expect(SATURNIAN_NATURAL_YEAR_DAYS).toBe(364);
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

describe('Tribonacci–Saturnian Bridge', () => {
  test('T(7) = 13 aligns exactly with RADIUS_COSMIC', () => {
    expect(TRIBONACCI_SEQUENCE[7]).toBe(13);
    expect(TRIBONACCI_RADIUS_MATCH).toBe(13);
    expect(TRIBONACCI_RADIUS_MATCH).toBe(RADIUS_COSMIC);
  });

  test('PI_ESOTERIC = T(7) + T(3) = 13 + 1 = 14', () => {
    expect(TRIBONACCI_SEQUENCE[7] + TRIBONACCI_SEQUENCE[3]).toBe(14);
    expect(PI_ESOTERIC).toBe(14);
  });

  test('full Tribonacci-Saturnian harmony holds', () => {
    expect(HAS_TRIBONACCI_SATURNIAN_HARMONY).toBe(true);
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
  test('Saturnian COSMIC_CIRCUMFERENCE matches ternary full circle', () => {
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
    const flat = getSaturnianFlattened();
    expect(flat).toEqual([111, 14, 208, 208, 111, 14, 14, 208, 111]);
    expect(flat.length).toBe(9);
  });

  test('cyclic shift 0 returns original matrix', () => {
    const m = getSaturnianCyclic(0);
    expect(m[0]).toEqual([111, 14, 208]);
    expect(m[1]).toEqual([208, 111, 14]);
    expect(m[2]).toEqual([14, 208, 111]);
  });

  test('cyclic shift 1 rotates rows', () => {
    const m = getSaturnianCyclic(1);
    expect(m[0]).toEqual([208, 111, 14]);
    expect(m[1]).toEqual([14, 208, 111]);
    expect(m[2]).toEqual([111, 14, 208]);
  });

  test('cyclic shift 2 rotates rows twice', () => {
    const m = getSaturnianCyclic(2);
    expect(m[0]).toEqual([14, 208, 111]);
    expect(m[1]).toEqual([111, 14, 208]);
    expect(m[2]).toEqual([208, 111, 14]);
  });

  test('cyclic shift 0 (identity) preserves full magic property', () => {
    expect(isSaturnianMagic(getSaturnianCyclic(0))).toBe(true);
  });

  test('all cyclic shifts preserve row and column sums', () => {
    for (const shift of [0, 1, 2] as const) {
      const m = getSaturnianCyclic(shift);
      for (let i = 0; i < 3; i++) {
        expect(m[i][0] + m[i][1] + m[i][2]).toBe(333);
        expect(m[0][i] + m[1][i] + m[2][i]).toBe(333);
      }
    }
  });

  test('ternary weights return canonical row values', () => {
    expect(getTernarySaturnianWeight(0)).toBe(111);
    expect(getTernarySaturnianWeight(1)).toBe(14);
    expect(getTernarySaturnianWeight(2)).toBe(208);
  });

  test('isSaturnianMagic rejects invalid matrices', () => {
    expect(isSaturnianMagic([[1, 2, 3], [4, 5, 6], [7, 8, 9]])).toBe(false);
    expect(isSaturnianMagic([[111, 111, 111], [111, 111, 111], [111, 111, 111]])).toBe(true);
  });

  test('isCirculant rejects non-circulant magic squares', () => {
    const nonCirculant = [[111, 111, 111], [111, 111, 111], [111, 111, 111]];
    expect(isCirculant(nonCirculant)).toBe(true);
    const broken = [[111, 14, 208], [14, 208, 111], [208, 111, 14]];
    expect(isCirculant(broken)).toBe(false);
  });
});
