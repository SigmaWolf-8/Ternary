/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { describe, it, expect } from 'vitest';
import {
  applyQutritError,
  encodeQutritStabilizer,
  measureSyndrome,
  correctQutritStabilizer,
  simulateTriorthogonalDistillation,
  qutritFidelity,
  detectError,
  qutritDepolarizingChannel,
  codeDistance,
  type QutritState,
} from '../shared/qutrit-fault-tolerance';
import { cx, cvNormSq, cxMag } from '../shared/complex-utils';

describe('Qutrit Fault Tolerance', () => {
  const basis0: QutritState = [cx(0), cx(1), cx(0)];
  const basis1: QutritState = [cx(1), cx(0), cx(0)];
  const superpos: QutritState = [
    cx(1 / Math.sqrt(3)),
    cx(1 / Math.sqrt(3)),
    cx(1 / Math.sqrt(3)),
  ];

  describe('Error operators', () => {
    it('none leaves state unchanged', () => {
      const result = applyQutritError(basis0, 'none');
      expect(result[0].re).toBe(0);
      expect(result[1].re).toBe(1);
    });

    it('phase applies ω rotation', () => {
      const result = applyQutritError(basis0, 'phase');
      expect(cxMag(result[1])).toBeCloseTo(1);
      expect(result[1].im).not.toBeCloseTo(0);
    });

    it('leak modifies amplitudes', () => {
      const result = applyQutritError(basis0, 'leak');
      expect(result[2].re).not.toBe(0);
    });

    it('depolarize creates maximally mixed state', () => {
      const result = applyQutritError(basis0, 'depolarize');
      expect(cxMag(result[0])).toBeCloseTo(cxMag(result[1]));
      expect(cxMag(result[1])).toBeCloseTo(cxMag(result[2]));
    });
  });

  describe('Stabilizer encoding', () => {
    it('encodes logical into 3 physical qutrits', () => {
      const encoded = encodeQutritStabilizer(basis0);
      expect(encoded.length).toBe(3);
      for (const phys of encoded) {
        expect(phys[1].re).toBe(1);
      }
    });

    it('preserves logical state in all copies', () => {
      const encoded = encodeQutritStabilizer(superpos);
      for (const phys of encoded) {
        expect(cxMag(phys[0])).toBeCloseTo(1 / Math.sqrt(3));
      }
    });
  });

  describe('Syndrome measurement', () => {
    it('zero syndrome for no error', () => {
      const encoded = encodeQutritStabilizer(basis0);
      const [s1, s2] = measureSyndrome(encoded);
      expect(s1).toBeCloseTo(0);
      expect(s2).toBeCloseTo(0);
    });

    it('non-zero syndrome when error on first qutrit', () => {
      const encoded = encodeQutritStabilizer(basis0);
      encoded[0] = applyQutritError(encoded[0], 'phase');
      const [s1, s2] = measureSyndrome(encoded);
      expect(s1).toBeGreaterThan(0);
    });

    it('non-zero syndrome for phase-only error (complex)', () => {
      const encoded = encodeQutritStabilizer(basis0);
      encoded[0] = applyQutritError(encoded[0], 'phase');
      const [s1] = measureSyndrome(encoded);
      expect(s1).toBeGreaterThan(1e-6);
    });
  });

  describe('Error correction', () => {
    it('corrects no-error case', () => {
      const encoded = encodeQutritStabilizer(basis0);
      const { corrected, errorPosition } = correctQutritStabilizer(encoded);
      expect(errorPosition).toBeNull();
      expect(corrected[1].re).toBe(1);
    });

    it('detects error on first qutrit', () => {
      const encoded = encodeQutritStabilizer(basis0);
      encoded[0] = [cx(1), cx(0), cx(0)];
      const { errorPosition } = correctQutritStabilizer(encoded);
      expect(errorPosition).toBe(0);
    });

    it('detects error on third qutrit', () => {
      const encoded = encodeQutritStabilizer(basis0);
      encoded[2] = [cx(1), cx(0), cx(0)];
      const { errorPosition } = correctQutritStabilizer(encoded);
      expect(errorPosition).toBe(2);
    });
  });

  describe('Triorthogonal distillation', () => {
    it('returns valid distilled state', () => {
      const inputs = [basis0, basis0, basis0];
      const { distilled, yieldGamma, codeParams } = simulateTriorthogonalDistillation(1, inputs);
      expect(cvNormSq(distilled)).toBeCloseTo(1);
      expect(yieldGamma).toBeGreaterThan(0);
      expect(codeParams.d).toBe(2);
    });

    it('code parameters scale with m', () => {
      const r1 = simulateTriorthogonalDistillation(1, [basis0]);
      const r2 = simulateTriorthogonalDistillation(2, [basis0]);
      expect(r2.codeParams.n).toBeGreaterThan(r1.codeParams.n);
    });

    it('throws for m < 1', () => {
      expect(() => simulateTriorthogonalDistillation(0, [basis0])).toThrow('m must be >= 1');
    });

    it('throws for empty inputs', () => {
      expect(() => simulateTriorthogonalDistillation(1, [])).toThrow('at least one');
    });

    it('yield gamma decreases with larger m', () => {
      const r1 = simulateTriorthogonalDistillation(1, [basis0]);
      const r2 = simulateTriorthogonalDistillation(3, [basis0]);
      expect(r2.yieldGamma).toBeLessThan(r1.yieldGamma);
    });
  });

  describe('Fidelity', () => {
    it('same state has fidelity 1', () => {
      expect(qutritFidelity(basis0, basis0)).toBeCloseTo(1);
    });

    it('orthogonal states have fidelity 0', () => {
      expect(qutritFidelity(basis0, basis1)).toBeCloseTo(0);
    });

    it('fidelity is in [0, 1]', () => {
      const f = qutritFidelity(superpos, basis0);
      expect(f).toBeGreaterThanOrEqual(0);
      expect(f).toBeLessThanOrEqual(1 + 1e-10);
    });
  });

  describe('Error detection', () => {
    it('no error detected in clean encoding', () => {
      const encoded = encodeQutritStabilizer(basis0);
      const { hasError } = detectError(encoded);
      expect(hasError).toBe(false);
    });

    it('detects error after corruption', () => {
      const encoded = encodeQutritStabilizer(basis0);
      encoded[1] = [cx(1), cx(0), cx(0)];
      const { hasError } = detectError(encoded);
      expect(hasError).toBe(true);
    });
  });

  describe('Depolarizing channel', () => {
    it('rate 0 preserves state', () => {
      const result = qutritDepolarizingChannel(basis0, 0);
      expect(cvNormSq(result)).toBeCloseTo(1);
    });

    it('throws for invalid rate', () => {
      expect(() => qutritDepolarizingChannel(basis0, -0.1)).toThrow('Error rate');
      expect(() => qutritDepolarizingChannel(basis0, 1.5)).toThrow('Error rate');
    });

    it('output is normalized', () => {
      const result = qutritDepolarizingChannel(basis0, 0.3);
      expect(cvNormSq(result)).toBeCloseTo(1, 5);
    });
  });

  describe('Code distance', () => {
    it('m=1 gives correct parameters', () => {
      const cd = codeDistance(1);
      expect(cd.k).toBe(1);
      expect(cd.d).toBe(2);
      expect(cd.n).toBeGreaterThan(cd.k);
    });

    it('overhead decreases with m', () => {
      const cd1 = codeDistance(1);
      const cd3 = codeDistance(3);
      expect(cd3.overhead).toBeLessThan(cd1.overhead);
    });

    it('throws for m < 1', () => {
      expect(() => codeDistance(0)).toThrow('m must be >= 1');
    });
  });
});
