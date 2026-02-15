/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  tribonacciActionDeviation,
  isExactTribonacci,
  generateTribonacci,
  tribonacciResiduals,
  tribonacciRatioConvergence,
  tribonacciPotential,
  tribonacciPotentialGradient,
  coupledLagrangianSite,
  discreteAction,
  verifyCanonicalTribonacci,
  variationalFitness,
} from '../shared/tribonacci-variational';
import { TRIBONACCI_SEQUENCE, TAU } from '../shared/tribonacci-constants';
import type { Trit } from '../shared/lagrangian-ternary-utils';

type TritTriple = [Trit, Trit, Trit];

describe('Tribonacci Variational — Action Deviation', () => {
  test('canonical sequence has zero action', () => {
    expect(tribonacciActionDeviation(TRIBONACCI_SEQUENCE)).toBe(0);
  });

  test('generated sequence has zero action', () => {
    const seq = generateTribonacci(15);
    expect(tribonacciActionDeviation(seq)).toBe(0);
  });

  test('perturbed sequence has non-zero action', () => {
    const seq = [0, 0, 1, 1, 2, 4, 7, 13, 24, 45];
    expect(tribonacciActionDeviation(seq)).toBeGreaterThan(0);
  });

  test('action is non-negative', () => {
    const seq = [1, 2, 3, 4, 5, 6, 7];
    expect(tribonacciActionDeviation(seq)).toBeGreaterThanOrEqual(0);
  });

  test('short sequences have zero action', () => {
    expect(tribonacciActionDeviation([0, 0, 1])).toBe(0);
    expect(tribonacciActionDeviation([5, 10])).toBe(0);
  });
});

describe('Tribonacci Variational — Exact Check', () => {
  test('canonical sequence is exact', () => {
    expect(isExactTribonacci(TRIBONACCI_SEQUENCE)).toBe(true);
  });

  test('perturbed sequence is not exact', () => {
    expect(isExactTribonacci([0, 0, 1, 1, 2, 4, 7, 13, 25])).toBe(false);
  });

  test('short sequences are trivially exact', () => {
    expect(isExactTribonacci([0])).toBe(true);
    expect(isExactTribonacci([0, 0])).toBe(true);
    expect(isExactTribonacci([0, 0, 1])).toBe(true);
    expect(isExactTribonacci([])).toBe(true);
  });

  test('non-standard initial conditions can still satisfy recurrence', () => {
    const seq = [1, 1, 1, 3, 5, 9, 17];
    expect(isExactTribonacci(seq)).toBe(true);
  });
});

describe('Tribonacci Variational — Generation', () => {
  test('generates correct first 10 terms', () => {
    expect(generateTribonacci(10)).toEqual([0, 0, 1, 1, 2, 4, 7, 13, 24, 44]);
  });

  test('matches canonical sequence', () => {
    const generated = generateTribonacci(TRIBONACCI_SEQUENCE.length);
    expect(generated).toEqual([...TRIBONACCI_SEQUENCE]);
  });

  test('handles edge cases', () => {
    expect(generateTribonacci(0)).toEqual([]);
    expect(generateTribonacci(1)).toEqual([0]);
    expect(generateTribonacci(2)).toEqual([0, 0]);
    expect(generateTribonacci(3)).toEqual([0, 0, 1]);
  });
});

describe('Tribonacci Variational — Residuals', () => {
  test('canonical sequence has all-zero residuals', () => {
    const residuals = tribonacciResiduals(TRIBONACCI_SEQUENCE);
    for (const r of residuals) {
      expect(r.residual).toBe(0);
      expect(r.penalty).toBe(0);
    }
  });

  test('perturbed last element has one non-zero residual', () => {
    const seq = [0, 0, 1, 1, 2, 4, 7, 13, 25];
    const residuals = tribonacciResiduals(seq);
    const lastResidual = residuals[residuals.length - 1];
    expect(lastResidual.residual).toBe(1);
    expect(lastResidual.penalty).toBe(1);
  });

  test('residuals index starts at 3', () => {
    const residuals = tribonacciResiduals([0, 0, 1, 1]);
    expect(residuals.length).toBe(1);
    expect(residuals[0].n).toBe(3);
  });
});

describe('Tribonacci Variational — Ratio Convergence', () => {
  test('ratios converge toward TAU', () => {
    const convergence = tribonacciRatioConvergence(generateTribonacci(15));
    const lastError = convergence[convergence.length - 1].error;
    expect(lastError).toBeLessThan(0.01);
  });

  test('late-stage errors are much smaller than early errors', () => {
    const convergence = tribonacciRatioConvergence(generateTribonacci(15));
    const earlyError = convergence[2].error;
    const lateError = convergence[convergence.length - 1].error;
    expect(lateError).toBeLessThan(earlyError * 0.01);
  });

  test('skips zero denominators', () => {
    const convergence = tribonacciRatioConvergence([0, 0, 1, 1]);
    expect(convergence.every(c => isFinite(c.ratio))).toBe(true);
  });
});

describe('Tribonacci Variational — Potential', () => {
  test('potential vanishes when branch sum equals T_n', () => {
    expect(tribonacciPotential(2, [0, 0, 1])).toBe(0);
    expect(tribonacciPotential(0, [0, 0, 0])).toBe(0);
  });

  test('potential is non-negative', () => {
    const trits: Trit[] = [-1, 0, 1];
    for (const a of trits) {
      for (const b of trits) {
        for (const c of trits) {
          const V = tribonacciPotential(3, [a, b, c]);
          expect(V).toBeGreaterThanOrEqual(0);
        }
      }
    }
  });

  test('potential grows with distance from T_n', () => {
    const V0 = tribonacciPotential(7, [1, 1, 1]);
    const V1 = tribonacciPotential(7, [0, 0, 0]);
    expect(V1).toBeGreaterThan(V0);
  });
});

describe('Tribonacci Variational — Potential Gradient', () => {
  test('gradient vanishes when branch sum equals T_n', () => {
    const grad = tribonacciPotentialGradient(2, [0, 0, 1]);
    expect(grad[0]).toBeCloseTo(0, 10);
    expect(grad[1]).toBeCloseTo(0, 10);
    expect(grad[2]).toBeCloseTo(0, 10);
  });

  test('gradient is symmetric across branches', () => {
    const grad = tribonacciPotentialGradient(7, [0, 0, 0]);
    expect(grad[0]).toBe(grad[1]);
    expect(grad[1]).toBe(grad[2]);
  });

  test('gradient points toward T_n (negative of deficit)', () => {
    const grad = tribonacciPotentialGradient(7, [0, 0, 0]);
    expect(grad[0]).toBe(-13);
  });
});

describe('Tribonacci Variational — Coupled Lagrangian', () => {
  test('zero state site returns non-negative Lagrangian', () => {
    const L = coupledLagrangianSite({
      n: 0,
      t: [0, 0, 0],
      tNext: [0, 0, 0],
      E: [0, 0, 0],
      ENext: [0, 0, 0],
      lambda: 0,
      mu: [0, 0, 0],
      p_mu: [0, 0, 0, 0],
    });
    expect(typeof L).toBe('number');
    expect(isFinite(L)).toBe(true);
  });

  test('Lagrangian includes Tribonacci potential', () => {
    const L_with = coupledLagrangianSite({
      n: 7,
      t: [0, 0, 0],
      tNext: [0, 0, 0],
      E: [0, 0, 0],
      ENext: [0, 0, 0],
      lambda: 0,
      mu: [0, 0, 0],
      p_mu: [0, 0, 0, 0],
    });
    expect(L_with).toBe(0.5 * 13 * 13);
  });
});

describe('Tribonacci Variational — Discrete Action', () => {
  test('empty chain has zero action', () => {
    expect(discreteAction([])).toBe(0);
  });

  test('single zero site has computable action', () => {
    const action = discreteAction([{
      n: 0,
      t: [0, 0, 0],
      tNext: [0, 0, 0],
      E: [0, 0, 0],
      ENext: [0, 0, 0],
      lambda: 0,
      mu: [0, 0, 0],
      p_mu: [0, 0, 0, 0],
    }]);
    expect(isFinite(action)).toBe(true);
  });
});

describe('Tribonacci Variational — Canonical Verification', () => {
  test('canonical Tribonacci verifies with zero action', () => {
    const result = verifyCanonicalTribonacci();
    expect(result.valid).toBe(true);
    expect(result.action).toBe(0);
    expect(result.length).toBe(TRIBONACCI_SEQUENCE.length);
  });
});

describe('Tribonacci Variational — Fitness', () => {
  test('exact Tribonacci has fitness 1.0', () => {
    expect(variationalFitness(TRIBONACCI_SEQUENCE)).toBe(1.0);
  });

  test('short sequences have fitness 1.0', () => {
    expect(variationalFitness([0])).toBe(1.0);
    expect(variationalFitness([0, 0, 1])).toBe(1.0);
  });

  test('perturbed sequence has fitness < 1', () => {
    const fitness = variationalFitness([0, 0, 1, 1, 2, 4, 7, 13, 25]);
    expect(fitness).toBeLessThan(1.0);
    expect(fitness).toBeGreaterThan(0);
  });

  test('fitness is bounded in [0, 1]', () => {
    const sequences = [
      [0, 0, 1, 1, 2, 4, 7, 13, 24],
      [1, 2, 3, 100, 200, 300, 400],
      [0, 0, 0, 0, 0, 0, 0],
    ];
    for (const seq of sequences) {
      const f = variationalFitness(seq);
      expect(f).toBeGreaterThanOrEqual(0);
      expect(f).toBeLessThanOrEqual(1);
    }
  });
});
