/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { describe, it, expect } from 'vitest';
import {
  suftBasisState,
  qutritFromAmplitudes,
  normalizeQutrit,
  qutritNormSq,
  bornProbabilities,
  expectationValue,
  suftPhaseGate,
  applyGate,
  qutritShiftOperator,
  qutritClockOperator,
  gellMannLambda,
  isUnitaryQutrit,
  qutritOverlap,
  suftBranchPhases,
  QUTRIT_IDENTITY,
  QUTRIT_REVIVAL_PERIOD,
} from '../shared/qutrit-basics';
import { cx, cxMag, cxMagSq, cvNormSq } from '../shared/complex-utils';

describe('Qutrit Basics', () => {
  describe('Basis states', () => {
    it('past branch |−1⟩ = [1,0,0]', () => {
      const s = suftBasisState(-1);
      expect(s[0].re).toBe(1);
      expect(s[1].re).toBe(0);
      expect(s[2].re).toBe(0);
    });

    it('present branch |0⟩ = [0,1,0]', () => {
      const s = suftBasisState(0);
      expect(s[1].re).toBe(1);
    });

    it('future branch |+1⟩ = [0,0,1]', () => {
      const s = suftBasisState(1);
      expect(s[2].re).toBe(1);
    });

    it('all basis states are normalized', () => {
      for (const alpha of [-1, 0, 1] as const) {
        expect(qutritNormSq(suftBasisState(alpha))).toBeCloseTo(1);
      }
    });
  });

  describe('Construction and normalization', () => {
    it('creates from amplitudes', () => {
      const s = qutritFromAmplitudes(cx(1), cx(0), cx(0));
      expect(s[0].re).toBe(1);
    });

    it('normalizes non-unit state', () => {
      const s = normalizeQutrit([cx(2), cx(0), cx(0)]);
      expect(qutritNormSq(s)).toBeCloseTo(1);
    });

    it('throws normalizing zero state', () => {
      expect(() => normalizeQutrit([cx(0), cx(0), cx(0)])).toThrow();
    });
  });

  describe('Born probabilities', () => {
    it('basis state has single probability 1', () => {
      const probs = bornProbabilities(suftBasisState(0));
      expect(probs[0]).toBeCloseTo(0);
      expect(probs[1]).toBeCloseTo(1);
      expect(probs[2]).toBeCloseTo(0);
    });

    it('equal superposition has equal probabilities', () => {
      const s = normalizeQutrit([cx(1), cx(1), cx(1)]);
      const probs = bornProbabilities(s);
      for (const p of probs) {
        expect(p).toBeCloseTo(1 / 3);
      }
    });

    it('probabilities sum to 1 for normalized state', () => {
      const s = normalizeQutrit([cx(1, 1), cx(2, -1), cx(0, 3)]);
      const probs = bornProbabilities(s);
      expect(probs.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
    });
  });

  describe('SUFT phase gate', () => {
    it('theta=0 gives identity', () => {
      const gate = suftPhaseGate(0);
      expect(isUnitaryQutrit(gate)).toBe(true);
      const s = applyGate(gate, suftBasisState(0));
      expect(cxMag(s[1])).toBeCloseTo(1);
    });

    it('is unitary for arbitrary theta', () => {
      expect(isUnitaryQutrit(suftPhaseGate(Math.PI / 4))).toBe(true);
      expect(isUnitaryQutrit(suftPhaseGate(1.5))).toBe(true);
    });

    it('preserves norm', () => {
      const s = normalizeQutrit([cx(1, 1), cx(2), cx(0, 1)]);
      const gate = suftPhaseGate(0.7);
      const result = applyGate(gate, s);
      expect(cvNormSq(result)).toBeCloseTo(1);
    });

    it('present branch is fixed (eigenstate)', () => {
      const gate = suftPhaseGate(Math.PI);
      const s = suftBasisState(0);
      const result = applyGate(gate, s);
      expect(cxMag(result[1])).toBeCloseTo(1);
    });
  });

  describe('Shift and clock operators', () => {
    it('shift is unitary', () => {
      expect(isUnitaryQutrit(qutritShiftOperator())).toBe(true);
    });

    it('clock is unitary', () => {
      expect(isUnitaryQutrit(qutritClockOperator())).toBe(true);
    });

    it('shift cycles |−1⟩ → |0⟩ → |+1⟩ → |−1⟩', () => {
      const X = qutritShiftOperator();
      const s0 = applyGate(X, suftBasisState(-1));
      expect(cxMag(s0[1])).toBeCloseTo(1);
      const s1 = applyGate(X, s0);
      expect(cxMag(s1[2])).toBeCloseTo(1);
      const s2 = applyGate(X, s1);
      expect(cxMag(s2[0])).toBeCloseTo(1);
    });

    it('shift^3 = identity', () => {
      const X = qutritShiftOperator();
      let s = suftBasisState(-1);
      for (let i = 0; i < 3; i++) s = applyGate(X, s);
      expect(qutritOverlap(s, suftBasisState(-1))).toBeCloseTo(1);
    });
  });

  describe('Gell-Mann matrices', () => {
    it('all 8 lambda matrices are 3x3', () => {
      for (let i = 1; i <= 8; i++) {
        const lam = gellMannLambda(i as 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8);
        expect(lam.length).toBe(3);
        expect(lam[0].length).toBe(3);
      }
    });

    it('lambda_1 swaps |−1⟩ and |0⟩', () => {
      const l1 = gellMannLambda(1);
      const s = applyGate(l1, suftBasisState(-1));
      expect(cxMag(s[1])).toBeCloseTo(1);
    });

    it('lambda_3 is diagonal', () => {
      const l3 = gellMannLambda(3);
      expect(l3[0][0].re).toBe(1);
      expect(l3[1][1].re).toBe(-1);
      expect(l3[2][2].re).toBe(0);
    });

    it('lambda_8 has trace zero (traceless)', () => {
      const l8 = gellMannLambda(8);
      const trace = l8[0][0].re + l8[1][1].re + l8[2][2].re;
      expect(trace).toBeCloseTo(0);
    });
  });

  describe('Overlap and fidelity', () => {
    it('same state has overlap 1', () => {
      const s = suftBasisState(0);
      expect(qutritOverlap(s, s)).toBeCloseTo(1);
    });

    it('orthogonal states have overlap 0', () => {
      expect(qutritOverlap(suftBasisState(-1), suftBasisState(0))).toBeCloseTo(0);
      expect(qutritOverlap(suftBasisState(0), suftBasisState(1))).toBeCloseTo(0);
    });
  });

  describe('SUFT branch phases', () => {
    it('zero phases give unit amplitudes', () => {
      const s = suftBranchPhases(0, 0, 0);
      for (const amp of s) {
        expect(cxMag(amp)).toBeCloseTo(1);
      }
    });
  });

  describe('Constants', () => {
    it('identity is 3x3', () => {
      expect(QUTRIT_IDENTITY.length).toBe(3);
    });

    it('revival period is 56', () => {
      expect(QUTRIT_REVIVAL_PERIOD).toBe(56);
    });
  });

  describe('Expectation value', () => {
    it('identity gives norm squared', () => {
      const s = suftBasisState(0);
      const ev = expectationValue(s, QUTRIT_IDENTITY);
      expect(ev.re).toBeCloseTo(1);
    });
  });
});
