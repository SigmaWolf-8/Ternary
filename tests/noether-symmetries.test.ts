/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  checkTernaryGaugeInvariant,
  applyTernaryGaugeTransform,
  enumerateGaugeShifts,
  checkReparamInvariant,
  checkPeriodicityInvariant,
  applyPeriodicityShift,
  canonicalPeriodicity,
  noetherGaugeCharge,
  verifyGaugeChargeConservation,
  validateNoetherInvariants,
} from '../shared/noether-symmetries-utils';
import type { Trit } from '../shared/lagrangian-ternary-utils';

type TritTriple = [Trit, Trit, Trit];

describe('Noether Symmetries — Ternary Gauge Invariant', () => {
  test('zero shift satisfies gauge constraint', () => {
    expect(checkTernaryGaugeInvariant([0, 0, 0])).toBe(true);
  });

  test('balanced shifts satisfy gauge constraint', () => {
    expect(checkTernaryGaugeInvariant([-1, 0, 1])).toBe(true);
    expect(checkTernaryGaugeInvariant([1, -1, 0])).toBe(true);
    expect(checkTernaryGaugeInvariant([0, 1, -1])).toBe(true);
    expect(checkTernaryGaugeInvariant([1, 1, -1])).toBe(false);
  });

  test('all-same shifts violate gauge constraint', () => {
    expect(checkTernaryGaugeInvariant([1, 1, 1])).toBe(false);
    expect(checkTernaryGaugeInvariant([-1, -1, -1])).toBe(false);
  });
});

describe('Noether Symmetries — Gauge Transform', () => {
  test('zero shift is identity', () => {
    expect(applyTernaryGaugeTransform([1, 0, -1], [0, 0, 0])).toEqual([1, 0, -1]);
  });

  test('balanced shift transforms correctly', () => {
    expect(applyTernaryGaugeTransform([-1, 0, 1], [1, 0, -1])).toEqual([0, 0, 0]);
  });

  test('transform wraps via mod-3', () => {
    const result = applyTernaryGaugeTransform([1, -1, 0], [-1, 0, 1]);
    expect(result.every(v => v >= -1 && v <= 1)).toBe(true);
  });

  test('throws on invalid gauge shift', () => {
    expect(() => applyTernaryGaugeTransform([0, 0, 0], [1, 1, 1])).toThrow('Ternary gauge violation');
  });
});

describe('Noether Symmetries — Gauge Shift Enumeration', () => {
  test('enumerates exactly 7 valid shifts', () => {
    const shifts = enumerateGaugeShifts();
    expect(shifts.length).toBe(7);
  });

  test('all enumerated shifts sum to zero', () => {
    const shifts = enumerateGaugeShifts();
    for (const s of shifts) {
      expect(s[0] + s[1] + s[2]).toBe(0);
    }
  });

  test('trivial shift [0,0,0] is included', () => {
    const shifts = enumerateGaugeShifts();
    expect(shifts).toContainEqual([0, 0, 0]);
  });

  test('all values are valid trits', () => {
    const shifts = enumerateGaugeShifts();
    for (const s of shifts) {
      for (const v of s) {
        expect(v).toBeGreaterThanOrEqual(-1);
        expect(v).toBeLessThanOrEqual(1);
      }
    }
  });
});

describe('Noether Symmetries — Reparametrization Invariant', () => {
  test('massless zero state satisfies reparam invariant for any lambda', () => {
    expect(checkReparamInvariant(0, [0, 0, 0, 0], [0, 0, 0], 0)).toBe(true);
    expect(checkReparamInvariant(1, [0, 0, 0, 0], [0, 0, 0], 0)).toBe(true);
    expect(checkReparamInvariant(-1, [0, 0, 0, 0], [0, 0, 0], 0)).toBe(true);
  });

  test('lambda=0 always satisfies reparam invariant', () => {
    expect(checkReparamInvariant(0, [1, 1, 1, 1], [1, 1, 1], 5)).toBe(true);
  });

  test('non-zero state with non-zero lambda can violate', () => {
    expect(checkReparamInvariant(1, [1, 0, 0, 0], [0, 0, 0], 0)).toBe(false);
  });
});

describe('Noether Symmetries — Periodicity', () => {
  test('zero satisfies periodicity invariant', () => {
    expect(checkPeriodicityInvariant(0)).toBe(true);
  });

  test('multiples of 364 satisfy periodicity', () => {
    expect(checkPeriodicityInvariant(364)).toBe(true);
    expect(checkPeriodicityInvariant(728)).toBe(true);
    expect(checkPeriodicityInvariant(-364)).toBe(true);
  });

  test('non-multiples of 364 violate periodicity', () => {
    expect(checkPeriodicityInvariant(1)).toBe(false);
    expect(checkPeriodicityInvariant(13)).toBe(false);
    expect(checkPeriodicityInvariant(365)).toBe(false);
  });

  test('applyPeriodicityShift with valid shift works', () => {
    expect(applyPeriodicityShift(0, 364)).toBe(364);
    expect(applyPeriodicityShift(100, 728)).toBe(828);
    expect(applyPeriodicityShift(50, 0)).toBe(50);
  });

  test('applyPeriodicityShift throws on invalid shift', () => {
    expect(() => applyPeriodicityShift(0, 100)).toThrow('Periodicity violation');
    expect(() => applyPeriodicityShift(0, 1)).toThrow('Periodicity violation');
  });

  test('canonicalPeriodicity reduces to [0, 363]', () => {
    expect(canonicalPeriodicity(0)).toBe(0);
    expect(canonicalPeriodicity(364)).toBe(0);
    expect(canonicalPeriodicity(365)).toBe(1);
    expect(canonicalPeriodicity(-1)).toBe(363);
    expect(canonicalPeriodicity(728)).toBe(0);
  });
});

describe('Noether Symmetries — Gauge Charge', () => {
  test('zero momenta give zero charge', () => {
    expect(noetherGaugeCharge([0, 0, 0])).toBe(0);
  });

  test('balanced momenta give zero charge', () => {
    expect(noetherGaugeCharge([1, -1, 0])).toBe(0);
    expect(noetherGaugeCharge([-1, 0, 1])).toBe(0);
  });

  test('all-positive momenta give zero charge (mod 3)', () => {
    expect(noetherGaugeCharge([1, 1, 1])).toBe(0);
  });

  test('unbalanced momenta give non-zero charge', () => {
    expect(noetherGaugeCharge([1, 0, 0])).toBe(1);
    expect(noetherGaugeCharge([-1, 0, 0])).toBe(2);
  });

  test('charge is always in {0, 1, 2}', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const a of trits) {
      for (const b of trits) {
        for (const c of trits) {
          const charge = noetherGaugeCharge([a, b, c]);
          expect(charge).toBeGreaterThanOrEqual(0);
          expect(charge).toBeLessThan(3);
        }
      }
    }
  });
});

describe('Noether Symmetries — Charge Conservation', () => {
  test('trivial shift preserves charge', () => {
    expect(verifyGaugeChargeConservation([1, 0, -1], [0, 0, 0])).toBe(true);
  });

  test('valid gauge shifts preserve charge', () => {
    const shifts = enumerateGaugeShifts();
    const momenta: TritTriple = [1, 0, -1];
    for (const theta of shifts) {
      expect(verifyGaugeChargeConservation(momenta, theta)).toBe(true);
    }
  });

  test('invalid shift returns false', () => {
    expect(verifyGaugeChargeConservation([0, 0, 0], [1, 1, 1])).toBe(false);
  });
});

describe('Noether Symmetries — Full Validation', () => {
  test('zero state satisfies all invariants', () => {
    const result = validateNoetherInvariants({
      lambda: 0,
      p_mu: [0, 0, 0, 0],
      E_trits: [0, 0, 0],
      t_epoch: 0,
      m2: 0,
    });
    expect(result.gaugeHolds).toBe(true);
    expect(result.reparamHolds).toBe(true);
    expect(result.periodicityHolds).toBe(true);
    expect(result.allHold).toBe(true);
  });

  test('non-zero epoch violates periodicity but gauge can hold', () => {
    const result = validateNoetherInvariants({
      lambda: 0,
      p_mu: [0, 0, 0, 0],
      E_trits: [1, -1, 0],
      t_epoch: 1,
      m2: 0,
    });
    expect(result.gaugeHolds).toBe(true);
    expect(result.periodicityHolds).toBe(false);
    expect(result.allHold).toBe(false);
  });

  test('epoch at 364 satisfies periodicity', () => {
    const result = validateNoetherInvariants({
      lambda: 0,
      p_mu: [0, 0, 0, 0],
      E_trits: [0, 0, 0],
      t_epoch: 364,
      m2: 0,
    });
    expect(result.periodicityHolds).toBe(true);
  });
});
