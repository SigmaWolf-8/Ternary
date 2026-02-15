/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { describe, it, expect } from 'vitest';
import {
  quditBasisState,
  quditFromAmplitudes,
  normalizeQudit,
  quditNormSq,
  quditBornProbabilities,
  quditShiftOperator,
  quditClockOperator,
  applyQuditGate,
  applyQuditError,
  quditOverlap,
  quditFidelity,
  quditDepolarizingChannel,
  quditCodeParameters,
  suftScaledQuditPhaseGate,
  quditDimension,
  isValidQuditState,
  quditMaxEntangled,
  infoDensityBits,
  SUPPORTED_DIMENSIONS,
} from '../shared/qudit-basics';
import { cx, cxMag, cvNormSq, matVecMul } from '../shared/complex-utils';

describe('Qudit Basics', () => {
  describe('Basis states', () => {
    it('creates d=3 basis', () => {
      const s = quditBasisState(1, 3);
      expect(s.length).toBe(3);
      expect(s[1].re).toBe(1);
      expect(s[0].re).toBe(0);
    });

    it('creates d=5 basis', () => {
      const s = quditBasisState(3, 5);
      expect(s.length).toBe(5);
      expect(s[3].re).toBe(1);
    });

    it('all basis states are normalized', () => {
      for (const d of [2, 3, 4, 5]) {
        for (let i = 0; i < d; i++) {
          expect(quditNormSq(quditBasisState(i, d))).toBeCloseTo(1);
        }
      }
    });

    it('throws for invalid level', () => {
      expect(() => quditBasisState(3, 3)).toThrow();
      expect(() => quditBasisState(-1, 3)).toThrow();
    });

    it('throws for d < 2', () => {
      expect(() => quditBasisState(0, 1)).toThrow('Dimension must be >= 2');
    });
  });

  describe('Construction and normalization', () => {
    it('creates from amplitudes', () => {
      const s = quditFromAmplitudes([cx(1), cx(0), cx(0), cx(0)]);
      expect(s.length).toBe(4);
    });

    it('normalizes', () => {
      const s = normalizeQudit([cx(2), cx(0), cx(0)]);
      expect(quditNormSq(s)).toBeCloseTo(1);
    });

    it('throws for too few amplitudes', () => {
      expect(() => quditFromAmplitudes([cx(1)])).toThrow('at least 2');
    });
  });

  describe('Born probabilities', () => {
    it('basis state has single prob = 1', () => {
      const probs = quditBornProbabilities(quditBasisState(2, 4));
      expect(probs[2]).toBeCloseTo(1);
      expect(probs[0]).toBeCloseTo(0);
    });

    it('probabilities sum to 1', () => {
      const s = normalizeQudit([cx(1), cx(1), cx(1), cx(1)]);
      const probs = quditBornProbabilities(s);
      expect(probs.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
    });
  });

  describe('Shift operator', () => {
    it('d=3 cycles through levels', () => {
      const X = quditShiftOperator(3);
      let s = quditBasisState(0, 3);
      s = applyQuditGate(X, s);
      expect(cxMag(s[1])).toBeCloseTo(1);
    });

    it('d=4 shift^4 = identity', () => {
      const X = quditShiftOperator(4);
      let s = quditBasisState(0, 4);
      for (let i = 0; i < 4; i++) s = applyQuditGate(X, s);
      expect(quditOverlap(s, quditBasisState(0, 4))).toBeCloseTo(1);
    });

    it('throws for d < 2', () => {
      expect(() => quditShiftOperator(1)).toThrow();
    });
  });

  describe('Clock operator', () => {
    it('d=3 applies phases', () => {
      const Z = quditClockOperator(3);
      const s = applyQuditGate(Z, quditBasisState(0, 3));
      expect(cxMag(s[0])).toBeCloseTo(1);
    });

    it('is diagonal', () => {
      const Z = quditClockOperator(4);
      for (let i = 0; i < 4; i++) {
        for (let j = 0; j < 4; j++) {
          if (i !== j) expect(cxMag(Z[i][j])).toBeCloseTo(0);
        }
      }
    });
  });

  describe('Error simulation', () => {
    it('none preserves state', () => {
      const s = quditBasisState(1, 4);
      const r = applyQuditError(s, 'none');
      expect(quditOverlap(r, s)).toBeCloseTo(1);
    });

    it('phase applies d-dependent phases', () => {
      const s = quditBasisState(1, 4);
      const r = applyQuditError(s, 'phase');
      expect(cxMag(r[1])).toBeCloseTo(1);
    });

    it('leak modifies last level', () => {
      const s = quditBasisState(0, 5);
      const r = applyQuditError(s, 'leak');
      expect(r[4].re).not.toBeCloseTo(0);
    });

    it('depolarize creates uniform amplitudes', () => {
      const r = applyQuditError(quditBasisState(0, 4), 'depolarize');
      const mag = cxMag(r[0]);
      for (const amp of r) {
        expect(cxMag(amp)).toBeCloseTo(mag);
      }
    });
  });

  describe('Overlap and fidelity', () => {
    it('same state has overlap 1', () => {
      const s = quditBasisState(2, 5);
      expect(quditOverlap(s, s)).toBeCloseTo(1);
    });

    it('orthogonal states have overlap 0', () => {
      expect(quditOverlap(quditBasisState(0, 4), quditBasisState(1, 4))).toBeCloseTo(0);
    });

    it('fidelity equals overlap', () => {
      const a = quditBasisState(0, 3);
      const b = normalizeQudit([cx(1), cx(1), cx(1)]);
      expect(quditFidelity(a, b)).toBeCloseTo(quditOverlap(a, b));
    });

    it('throws on dimension mismatch', () => {
      expect(() => quditOverlap(quditBasisState(0, 3), quditBasisState(0, 4))).toThrow('mismatch');
    });
  });

  describe('Depolarizing channel', () => {
    it('rate 0 preserves', () => {
      const result = quditDepolarizingChannel(quditBasisState(0, 4), 0);
      expect(cvNormSq(result)).toBeCloseTo(1);
    });

    it('throws for invalid rate', () => {
      expect(() => quditDepolarizingChannel(quditBasisState(0, 3), -0.1)).toThrow();
      expect(() => quditDepolarizingChannel(quditBasisState(0, 3), 1.5)).toThrow();
    });
  });

  describe('Code parameters', () => {
    it('d=3 m=1 gives valid params', () => {
      const cp = quditCodeParameters(1, 3);
      expect(cp.k).toBeGreaterThan(0);
      expect(cp.n).toBeGreaterThan(cp.k);
      expect(cp.dist).toBe(2);
    });

    it('overhead decreases with m', () => {
      const cp1 = quditCodeParameters(1, 3);
      const cp3 = quditCodeParameters(3, 3);
      expect(cp3.overhead).toBeLessThan(cp1.overhead);
    });

    it('higher d yields more info density', () => {
      expect(infoDensityBits(4)).toBeGreaterThan(infoDensityBits(3));
      expect(infoDensityBits(5)).toBeGreaterThan(infoDensityBits(4));
    });

    it('throws for d < 2', () => {
      expect(() => quditCodeParameters(1, 1)).toThrow();
    });
  });

  describe('SUFT phase gate', () => {
    it('theta=0 gives identity for any d', () => {
      for (const d of [3, 4, 5]) {
        const gate = suftScaledQuditPhaseGate(d, 0);
        const s = quditBasisState(0, d);
        const r = applyQuditGate(gate, s);
        expect(cxMag(r[0])).toBeCloseTo(1);
      }
    });

    it('preserves norm', () => {
      const s = normalizeQudit([cx(1), cx(1), cx(1), cx(1)]);
      const gate = suftScaledQuditPhaseGate(4, 0.5);
      const r = applyQuditGate(gate, s);
      expect(cvNormSq(r)).toBeCloseTo(1);
    });
  });

  describe('Utility functions', () => {
    it('quditDimension returns correct d', () => {
      expect(quditDimension(quditBasisState(0, 7))).toBe(7);
    });

    it('isValidQuditState checks normalization', () => {
      expect(isValidQuditState(quditBasisState(0, 3))).toBe(true);
      expect(isValidQuditState([cx(2), cx(0), cx(0)])).toBe(false);
    });

    it('maxEntangled has equal amplitudes', () => {
      const s = quditMaxEntangled(4);
      expect(quditNormSq(s)).toBeCloseTo(1);
      for (const amp of s) {
        expect(cxMag(amp)).toBeCloseTo(1 / 2);
      }
    });

    it('SUPPORTED_DIMENSIONS includes key values', () => {
      expect(SUPPORTED_DIMENSIONS).toContain(3);
      expect(SUPPORTED_DIMENSIONS).toContain(13);
    });
  });
});
