/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { describe, it, expect } from 'vitest';
import {
  quantumElBranchUpdate,
  triboQutritPotential,
  discreteQutritAction,
  discreteQutritActionSum,
  qutritBranchRealProjection,
  qutritEnergyProjection,
  type QutritBranch,
} from '../shared/lagrangian-qutrit-utils';
import { cx, CX_ZERO } from '../shared/complex-utils';
import { TRIBONACCI_SEQUENCE } from '../shared/tribonacci-constants';

describe('Lagrangian Qutrit Utilities', () => {
  const zeroBranch: QutritBranch = { t_alpha: cx(0), E_alpha: cx(0) };
  const neutralBranches: [QutritBranch, QutritBranch, QutritBranch] = [
    { ...zeroBranch },
    { ...zeroBranch },
    { ...zeroBranch },
  ];
  const zeroDots: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(0), cx(0), cx(0)];
  const zeroMu: [0, 0, 0] = [0, 0, 0];

  describe('quantumElBranchUpdate', () => {
    it('neutral case preserves zero energy', () => {
      const updated = quantumElBranchUpdate(neutralBranches, zeroMu, zeroDots);
      for (const branch of updated) {
        expect(branch.E_alpha.re).toBe(0);
      }
    });

    it('preserves branch positions (t_alpha unchanged)', () => {
      const branches: [QutritBranch, QutritBranch, QutritBranch] = [
        { t_alpha: cx(-1), E_alpha: cx(0) },
        { t_alpha: cx(0), E_alpha: cx(1) },
        { t_alpha: cx(1), E_alpha: cx(-1) },
      ];
      const updated = quantumElBranchUpdate(branches, zeroMu, zeroDots);
      expect(updated[0].t_alpha.re).toBe(-1);
      expect(updated[1].t_alpha.re).toBe(0);
      expect(updated[2].t_alpha.re).toBe(1);
    });

    it('returns trit values for energy', () => {
      const updated = quantumElBranchUpdate(neutralBranches, [1, 0, -1], zeroDots);
      for (const branch of updated) {
        expect([-1, 0, 1]).toContain(branch.E_alpha.re);
      }
    });

    it('responds to non-zero dot_trits', () => {
      const dots: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(0), cx(1)];
      const updated = quantumElBranchUpdate(neutralBranches, zeroMu, dots);
      expect(typeof updated[0].E_alpha.re).toBe('number');
    });
  });

  describe('triboQutritPotential', () => {
    it('zero for branches matching T(n)', () => {
      const T_0 = TRIBONACCI_SEQUENCE[0]; // 0
      const amps: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(0), cx(0), cx(0)];
      const V = triboQutritPotential(0, amps);
      expect(V).toBeCloseTo(0.5 * (T_0 - 0) ** 2);
    });

    it('positive for mismatched branches', () => {
      const amps: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(5), cx(5), cx(5)];
      const V = triboQutritPotential(7, amps); // T(7)=13, sum=15
      expect(V).toBeGreaterThan(0);
    });

    it('exact at T(7)=13 with matching branches', () => {
      const amps: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(4), cx(4), cx(5)];
      const V = triboQutritPotential(7, amps); // sum=13=T(7)
      expect(V).toBeCloseTo(0);
    });

    it('throws for negative n', () => {
      expect(() => triboQutritPotential(-1, [cx(0), cx(0), cx(0)])).toThrow('non-negative');
    });

    it('wraps around Tribonacci sequence', () => {
      const seqLen = TRIBONACCI_SEQUENCE.length;
      const V1 = triboQutritPotential(0, [cx(0), cx(0), cx(0)]);
      const V2 = triboQutritPotential(seqLen, [cx(0), cx(0), cx(0)]);
      expect(V1).toBeCloseTo(V2);
    });
  });

  describe('discreteQutritAction', () => {
    it('returns finite value for neutral states', () => {
      const s0: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(0), cx(0), cx(0)];
      const s1: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(0), cx(0), cx(0)];
      const action = discreteQutritAction(s0, s1, 0, [0, 0, 0], 0);
      expect(isFinite(action)).toBe(true);
    });

    it('includes cross-coupling term', () => {
      const s0: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(0), cx(0)];
      const s1: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(0), cx(1)];
      const action = discreteQutritAction(s0, s1, 0, [0, 0, 0], 0);
      expect(action).not.toBeCloseTo(0);
    });

    it('constraint term scales with lambda', () => {
      const s0: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(1), cx(1)];
      const s1: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(1), cx(1)];
      const a1 = discreteQutritAction(s0, s1, 0, [0, 0, 0], 0);
      const a2 = discreteQutritAction(s0, s1, 1, [0, 0, 0], 0);
      expect(a1).not.toBeCloseTo(a2);
    });

    it('includes m2 mass term', () => {
      const s0: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(0), cx(0)];
      const s1: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(1), cx(0), cx(0)];
      const a1 = discreteQutritAction(s0, s1, 1, [0, 0, 0], 0, 0);
      const a2 = discreteQutritAction(s0, s1, 1, [0, 0, 0], 0, 2);
      expect(a1).not.toBeCloseTo(a2);
    });

    it('periodicity term responds to mu', () => {
      const s0: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(91), cx(0), cx(0)];
      const s1: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO] = [cx(91), cx(0), cx(0)];
      const a1 = discreteQutritAction(s0, s1, 0, [0, 0, 0], 0);
      const a2 = discreteQutritAction(s0, s1, 0, [10, 0, 0], 0);
      expect(Math.abs(a1 - a2)).toBeGreaterThan(1);
    });
  });

  describe('discreteQutritActionSum', () => {
    it('sums action over trajectory', () => {
      const traj: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO][] = [
        [cx(0), cx(0), cx(0)],
        [cx(0), cx(0), cx(0)],
        [cx(0), cx(0), cx(0)],
      ];
      const total = discreteQutritActionSum(traj, [0, 0], [[0, 0, 0], [0, 0, 0]]);
      expect(isFinite(total)).toBe(true);
    });

    it('throws for trajectory shorter than 2', () => {
      expect(() => discreteQutritActionSum([[cx(0), cx(0), cx(0)]], [], [])).toThrow('at least 2');
    });

    it('throws for lambda count mismatch', () => {
      const traj: [typeof CX_ZERO, typeof CX_ZERO, typeof CX_ZERO][] = [
        [cx(0), cx(0), cx(0)],
        [cx(0), cx(0), cx(0)],
      ];
      expect(() => discreteQutritActionSum(traj, [0, 0], [[0, 0, 0]])).toThrow('Lambda count');
    });
  });

  describe('Projections', () => {
    it('projects branches to trits', () => {
      const branches: [QutritBranch, QutritBranch, QutritBranch] = [
        { t_alpha: cx(-1), E_alpha: cx(0) },
        { t_alpha: cx(0), E_alpha: cx(1) },
        { t_alpha: cx(1), E_alpha: cx(-1) },
      ];
      const trits = qutritBranchRealProjection(branches);
      expect(trits).toEqual([-1, 0, 1]);
    });

    it('projects energies to trits', () => {
      const branches: [QutritBranch, QutritBranch, QutritBranch] = [
        { t_alpha: cx(0), E_alpha: cx(1) },
        { t_alpha: cx(0), E_alpha: cx(-1) },
        { t_alpha: cx(0), E_alpha: cx(0) },
      ];
      const trits = qutritEnergyProjection(branches);
      expect(trits).toEqual([1, -1, 0]);
    });
  });
});
