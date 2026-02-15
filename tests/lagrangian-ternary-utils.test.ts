/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  toTrit,
  ternaryMomentum,
  canonicalMomentumTMinus1,
  canonicalMomentumTPlus1,
  elUpdateEDotMinus1,
  elUpdateEDot0,
  elUpdateEDotPlus1,
  elSolveTDotMinus1,
  elSolveTDot0,
  elSolveTDotPlus1,
  ternaryCrossCoupling,
  checkMassShellConstraint,
  checkPeriodicityConstraint,
  noetherTernaryCharge,
  eulerLagrangeStep,
  type Trit,
} from '../shared/lagrangian-ternary-utils';

describe('Lagrangian Ternary Utils — Core Functions', () => {
  test('toTrit maps values to balanced ternary {-1, 0, +1}', () => {
    expect(toTrit(0)).toBe(0);
    expect(toTrit(1)).toBe(1);
    expect(toTrit(-1)).toBe(-1);
    expect(toTrit(3)).toBe(0);
    expect(toTrit(-3)).toBe(0);
    expect(toTrit(2)).toBe(-1);
    expect(toTrit(0.4)).toBe(0);
    expect(toTrit(0.6)).toBe(1);
  });

  test('toTrit output always in {-1, 0, 1}', () => {
    for (let i = -20; i <= 20; i++) {
      const t = toTrit(i);
      expect(t).toBeGreaterThanOrEqual(-1);
      expect(t).toBeLessThanOrEqual(1);
    }
  });

  test('ternaryMomentum preserves velocity (13 ≡ 1 mod 3)', () => {
    expect(ternaryMomentum(-1)).toBe(-1);
    expect(ternaryMomentum(0)).toBe(0);
    expect(ternaryMomentum(1)).toBe(1);
  });
});

describe('Lagrangian Ternary Utils — Canonical Momenta', () => {
  test('canonicalMomentumTMinus1 computes π = E₋₁ − (φ/26)·t₊₁', () => {
    const pi = canonicalMomentumTMinus1(1, 0);
    expect(pi).toBe(1);
    const pi2 = canonicalMomentumTMinus1(0, 0);
    expect(pi2).toBe(0);
  });

  test('canonicalMomentumTPlus1 computes π = E₊₁ + (φ/26)·t₋₁', () => {
    const pi = canonicalMomentumTPlus1(1, 0);
    expect(pi).toBe(1);
    const pi2 = canonicalMomentumTPlus1(0, 0);
    expect(pi2).toBe(0);
  });

  test('canonical momenta return valid trits for all input combinations', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const E of trits) {
      for (const t of trits) {
        const pm = canonicalMomentumTMinus1(E, t);
        const pp = canonicalMomentumTPlus1(E, t);
        expect(pm).toBeGreaterThanOrEqual(-1);
        expect(pm).toBeLessThanOrEqual(1);
        expect(pp).toBeGreaterThanOrEqual(-1);
        expect(pp).toBeLessThanOrEqual(1);
      }
    }
  });
});

describe('Lagrangian Ternary Utils — EL Updates', () => {
  test('elUpdateEDotMinus1 returns valid trit', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const v of trits) {
      for (const mu of trits) {
        for (const t of trits) {
          const result = elUpdateEDotMinus1(v, mu, t);
          expect(result).toBeGreaterThanOrEqual(-1);
          expect(result).toBeLessThanOrEqual(1);
        }
      }
    }
  });

  test('elUpdateEDot0 — zero multiplier gives zero update', () => {
    expect(elUpdateEDot0(0, 0)).toBe(0);
    expect(elUpdateEDot0(0, 1)).toBe(0);
    expect(elUpdateEDot0(0, -1)).toBe(0);
  });

  test('elUpdateEDotPlus1 returns valid trit', () => {
    const result = elUpdateEDotPlus1(0, 0, 0);
    expect(result).toBe(0);
    const result2 = elUpdateEDotPlus1(1, 1, 1);
    expect(result2).toBeGreaterThanOrEqual(-1);
    expect(result2).toBeLessThanOrEqual(1);
  });

  test('elSolveTDotMinus1 returns valid trit', () => {
    const result = elSolveTDotMinus1(0, 0, 0);
    expect(result).toBe(0);
  });

  test('elSolveTDot0 — zero lambda gives zero velocity', () => {
    expect(elSolveTDot0(0, 1)).toBe(0);
    expect(elSolveTDot0(0, -1)).toBe(0);
    expect(elSolveTDot0(0, 0)).toBe(0);
  });

  test('elSolveTDotPlus1 returns valid trit', () => {
    const result = elSolveTDotPlus1(0, 0, 0);
    expect(result).toBe(0);
  });
});

describe('Lagrangian Ternary Utils — Cross Coupling', () => {
  test('ternaryCrossCoupling is antisymmetric', () => {
    const c1 = ternaryCrossCoupling(1, -1);
    const c2 = ternaryCrossCoupling(-1, 1);
    expect(c1).toBeCloseTo(-c2, 10);
  });

  test('ternaryCrossCoupling vanishes for equal branches', () => {
    expect(ternaryCrossCoupling(0, 0)).toBe(0);
    expect(ternaryCrossCoupling(1, 1)).toBe(0);
    expect(ternaryCrossCoupling(-1, -1)).toBe(0);
  });

  test('ternaryCrossCoupling has expected scale ≈ φ/26', () => {
    const phi = (1 + Math.sqrt(5)) / 2;
    const expected = -2 * phi / 26;
    expect(ternaryCrossCoupling(1, -1)).toBeCloseTo(expected, 10);
  });
});

describe('Lagrangian Ternary Utils — Constraints', () => {
  test('mass-shell constraint vanishes for massless zero state', () => {
    const result = checkMassShellConstraint([0, 0, 0, 0], [0, 0, 0], 0);
    expect(result.constraint).toBe(0);
    expect(result.vanishes).toBe(true);
  });

  test('mass-shell constraint computes correctly', () => {
    const result = checkMassShellConstraint([1, 0, 0, 0], [0, 0, 0], 0);
    expect(result.constraint).toBe(-1);
  });

  test('mass-shell with energy terms uses 13/28 ratio', () => {
    const result = checkMassShellConstraint([0, 0, 0, 0], [1, 1, 1], 0);
    expect(result.constraint).toBeCloseTo(3 * 13 / 28, 10);
  });

  test('periodicity constraint satisfied at t = 0', () => {
    expect(checkPeriodicityConstraint(0)).toBe(true);
  });

  test('periodicity constraint satisfied at t = 364', () => {
    expect(checkPeriodicityConstraint(364)).toBe(true);
  });

  test('periodicity constraint NOT satisfied at t = 1', () => {
    expect(checkPeriodicityConstraint(1)).toBe(false);
  });

  test('periodicity constraint satisfied at multiples of 364', () => {
    expect(checkPeriodicityConstraint(728)).toBe(true);
    expect(checkPeriodicityConstraint(-364)).toBe(true);
  });
});

describe('Lagrangian Ternary Utils — Noether Charge', () => {
  test('zero momenta give zero charge', () => {
    expect(noetherTernaryCharge([0, 0, 0])).toBe(0);
  });

  test('balanced momenta give zero charge', () => {
    expect(noetherTernaryCharge([1, -1, 0])).toBe(0);
    expect(noetherTernaryCharge([-1, 0, 1])).toBe(0);
    expect(noetherTernaryCharge([1, 1, 1])).toBe(0);
  });

  test('unbalanced momenta give non-zero charge', () => {
    expect(noetherTernaryCharge([1, 0, 0])).toBe(1);
    expect(noetherTernaryCharge([-1, 0, 0])).toBe(2);
  });

  test('charge is always in {0, 1, 2}', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const a of trits) {
      for (const b of trits) {
        for (const c of trits) {
          const charge = noetherTernaryCharge([a, b, c]);
          expect(charge).toBeGreaterThanOrEqual(0);
          expect(charge).toBeLessThan(3);
        }
      }
    }
  });
});

describe('Lagrangian Ternary Utils — Full EL Step', () => {
  test('eulerLagrangeStep with zero state returns valid trits', () => {
    const result = eulerLagrangeStep({
      t: [0, 0, 0],
      E: [0, 0, 0],
      tDot: [0, 0, 0],
    });

    for (const arr of [result.t, result.E, result.tDot, result.EDot]) {
      for (const v of arr) {
        expect(v).toBeGreaterThanOrEqual(-1);
        expect(v).toBeLessThanOrEqual(1);
      }
    }
    expect(result.noetherCharge).toBeGreaterThanOrEqual(0);
    expect(result.noetherCharge).toBeLessThan(3);
  });

  test('eulerLagrangeStep with non-trivial state produces valid output', () => {
    const result = eulerLagrangeStep(
      {
        t: [1, 0, -1],
        E: [-1, 1, 0],
        tDot: [0, 1, -1],
      },
      1,
      [1, 0, -1]
    );

    for (const arr of [result.t, result.E, result.tDot, result.EDot]) {
      for (const v of arr) {
        expect(v).toBeGreaterThanOrEqual(-1);
        expect(v).toBeLessThanOrEqual(1);
      }
    }
  });

  test('eulerLagrangeStep is deterministic', () => {
    const state = {
      t: [1, -1, 0] as [Trit, Trit, Trit],
      E: [0, 1, -1] as [Trit, Trit, Trit],
      tDot: [-1, 0, 1] as [Trit, Trit, Trit],
    };

    const r1 = eulerLagrangeStep(state);
    const r2 = eulerLagrangeStep(state);
    expect(r1.t).toEqual(r2.t);
    expect(r1.E).toEqual(r2.E);
    expect(r1.tDot).toEqual(r2.tDot);
    expect(r1.EDot).toEqual(r2.EDot);
    expect(r1.noetherCharge).toBe(r2.noetherCharge);
  });

  test('eulerLagrangeStep produces valid results for all zero-velocity inputs', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const t0 of trits) {
      for (const E0 of trits) {
        const result = eulerLagrangeStep({
          t: [t0, 0, 0],
          E: [E0, 0, 0],
          tDot: [0, 0, 0],
        });
        for (const v of [...result.t, ...result.E, ...result.tDot, ...result.EDot]) {
          expect(v).toBeGreaterThanOrEqual(-1);
          expect(v).toBeLessThanOrEqual(1);
        }
      }
    }
  });
});
