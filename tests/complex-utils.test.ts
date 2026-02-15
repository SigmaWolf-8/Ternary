/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { describe, it, expect } from 'vitest';
import {
  cx, cxFromPolar, cxAdd, cxSub, cxMul, cxDiv, cxConj,
  cxMag, cxMagSq, cxScale, cxNeg,
  CX_ZERO, CX_ONE, CX_I,
  cvNorm, cvNormSq, cvNormalize, cvAdd, cvSub, cvScale,
  cvInnerProduct,
  matVecMul, matMul, matDagger, identityMatrix,
  cxApproxEqual,
} from '../shared/complex-utils';

describe('Complex Utilities', () => {
  describe('Construction', () => {
    it('creates real complex number', () => {
      const z = cx(3);
      expect(z.re).toBe(3);
      expect(z.im).toBe(0);
    });

    it('creates complex number with imaginary part', () => {
      const z = cx(2, 5);
      expect(z.re).toBe(2);
      expect(z.im).toBe(5);
    });

    it('creates from polar form', () => {
      const z = cxFromPolar(1, Math.PI / 2);
      expect(Math.abs(z.re)).toBeLessThan(1e-10);
      expect(Math.abs(z.im - 1)).toBeLessThan(1e-10);
    });

    it('CX_ZERO is zero', () => {
      expect(CX_ZERO.re).toBe(0);
      expect(CX_ZERO.im).toBe(0);
    });

    it('CX_I is imaginary unit', () => {
      expect(CX_I.re).toBe(0);
      expect(CX_I.im).toBe(1);
    });
  });

  describe('Arithmetic', () => {
    it('adds two complex numbers', () => {
      const r = cxAdd(cx(1, 2), cx(3, 4));
      expect(r.re).toBe(4);
      expect(r.im).toBe(6);
    });

    it('subtracts two complex numbers', () => {
      const r = cxSub(cx(5, 3), cx(2, 1));
      expect(r.re).toBe(3);
      expect(r.im).toBe(2);
    });

    it('multiplies two complex numbers', () => {
      const r = cxMul(cx(1, 2), cx(3, 4));
      expect(r.re).toBe(-5);
      expect(r.im).toBe(10);
    });

    it('i * i = -1', () => {
      const r = cxMul(CX_I, CX_I);
      expect(r.re).toBeCloseTo(-1);
      expect(r.im).toBeCloseTo(0);
    });

    it('divides two complex numbers', () => {
      const r = cxDiv(cx(2, 4), cx(1, 1));
      expect(r.re).toBeCloseTo(3);
      expect(r.im).toBeCloseTo(1);
    });

    it('throws on division by zero', () => {
      expect(() => cxDiv(cx(1), CX_ZERO)).toThrow('Division by zero');
    });

    it('scales complex number', () => {
      const r = cxScale(cx(2, 3), 4);
      expect(r.re).toBe(8);
      expect(r.im).toBe(12);
    });

    it('negates complex number', () => {
      const r = cxNeg(cx(2, -3));
      expect(r.re).toBe(-2);
      expect(r.im).toBe(3);
    });
  });

  describe('Unary operations', () => {
    it('conjugates correctly', () => {
      const r = cxConj(cx(3, 4));
      expect(r.re).toBe(3);
      expect(r.im).toBe(-4);
    });

    it('computes magnitude', () => {
      expect(cxMag(cx(3, 4))).toBeCloseTo(5);
    });

    it('computes magnitude squared', () => {
      expect(cxMagSq(cx(3, 4))).toBe(25);
    });
  });

  describe('Vector operations', () => {
    it('computes norm squared', () => {
      expect(cvNormSq([cx(1), cx(0), cx(0)])).toBe(1);
    });

    it('computes norm', () => {
      expect(cvNorm([cx(3), cx(4)])).toBeCloseTo(5);
    });

    it('normalizes a vector', () => {
      const v = cvNormalize([cx(3), cx(4)]);
      expect(cvNorm(v)).toBeCloseTo(1);
    });

    it('throws normalizing zero vector', () => {
      expect(() => cvNormalize([CX_ZERO])).toThrow('Cannot normalize zero');
    });

    it('adds vectors', () => {
      const r = cvAdd([cx(1), cx(2)], [cx(3), cx(4)]);
      expect(r[0].re).toBe(4);
      expect(r[1].re).toBe(6);
    });

    it('subtracts vectors', () => {
      const r = cvSub([cx(5), cx(3)], [cx(2), cx(1)]);
      expect(r[0].re).toBe(3);
      expect(r[1].re).toBe(2);
    });

    it('scales vector', () => {
      const r = cvScale([cx(1, 2), cx(3, 4)], 2);
      expect(r[0].re).toBe(2);
      expect(r[1].im).toBe(8);
    });

    it('computes inner product', () => {
      const ip = cvInnerProduct([cx(1), cx(0)], [cx(0), cx(1)]);
      expect(ip.re).toBeCloseTo(0);
      expect(ip.im).toBeCloseTo(0);
    });

    it('inner product of same vector gives norm squared', () => {
      const v = [cx(1, 1), cx(2, -1)];
      const ip = cvInnerProduct(v, v);
      expect(ip.re).toBeCloseTo(cvNormSq(v));
      expect(Math.abs(ip.im)).toBeLessThan(1e-10);
    });

    it('throws on dimension mismatch', () => {
      expect(() => cvAdd([cx(1)], [cx(1), cx(2)])).toThrow('dimension mismatch');
    });
  });

  describe('Matrix operations', () => {
    it('identity matrix times vector is identity', () => {
      const I = identityMatrix(3);
      const v = [cx(1), cx(2), cx(3)];
      const r = matVecMul(I, v);
      expect(r[0].re).toBe(1);
      expect(r[1].re).toBe(2);
      expect(r[2].re).toBe(3);
    });

    it('multiplies matrices', () => {
      const A = [[cx(1), cx(0)], [cx(0), cx(1)]];
      const B = [[cx(2), cx(3)], [cx(4), cx(5)]];
      const R = matMul(A, B);
      expect(R[0][0].re).toBe(2);
      expect(R[1][1].re).toBe(5);
    });

    it('computes adjoint (dagger)', () => {
      const M = [[cx(1, 1), cx(2, 3)], [cx(4, 5), cx(6, 7)]];
      const D = matDagger(M);
      expect(D[0][0].re).toBe(1);
      expect(D[0][0].im).toBe(-1);
      expect(D[0][1].re).toBe(4);
      expect(D[0][1].im).toBe(-5);
    });

    it('I dagger = I', () => {
      const I = identityMatrix(3);
      const D = matDagger(I);
      for (let i = 0; i < 3; i++) {
        for (let j = 0; j < 3; j++) {
          expect(D[i][j].re).toBeCloseTo(i === j ? 1 : 0);
          expect(D[i][j].im).toBeCloseTo(0);
        }
      }
    });
  });

  describe('Approximate equality', () => {
    it('equal values are approximately equal', () => {
      expect(cxApproxEqual(cx(1, 2), cx(1, 2))).toBe(true);
    });

    it('close values within epsilon', () => {
      expect(cxApproxEqual(cx(1, 2), cx(1 + 1e-12, 2 - 1e-12))).toBe(true);
    });

    it('distant values not equal', () => {
      expect(cxApproxEqual(cx(1), cx(2))).toBe(false);
    });
  });
});
