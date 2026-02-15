/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * # Lightweight Complex Number Utilities
 *
 * Self-contained complex arithmetic for quantum ternary simulations.
 * No external dependencies — all operations are pure and exact where
 * the underlying IEEE 754 representation permits.
 *
 * Supports: construction, addition, subtraction, multiplication, division,
 * conjugation, magnitude, polar form, and matrix–vector products.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

export interface Complex {
  re: number;
  im: number;
}

export function cx(re: number, im: number = 0): Complex {
  return { re, im };
}

export function cxFromPolar(r: number, theta: number): Complex {
  return { re: r * Math.cos(theta), im: r * Math.sin(theta) };
}

export function cxAdd(a: Complex, b: Complex): Complex {
  return { re: a.re + b.re, im: a.im + b.im };
}

export function cxSub(a: Complex, b: Complex): Complex {
  return { re: a.re - b.re, im: a.im - b.im };
}

export function cxMul(a: Complex, b: Complex): Complex {
  return {
    re: a.re * b.re - a.im * b.im,
    im: a.re * b.im + a.im * b.re,
  };
}

export function cxDiv(a: Complex, b: Complex): Complex {
  const denom = b.re * b.re + b.im * b.im;
  if (denom === 0) throw new Error('Division by zero in complex division');
  return {
    re: (a.re * b.re + a.im * b.im) / denom,
    im: (a.im * b.re - a.re * b.im) / denom,
  };
}

export function cxConj(a: Complex): Complex {
  return { re: a.re, im: -a.im };
}

export function cxMag(a: Complex): number {
  return Math.sqrt(a.re * a.re + a.im * a.im);
}

export function cxMagSq(a: Complex): number {
  return a.re * a.re + a.im * a.im;
}

export function cxScale(a: Complex, s: number): Complex {
  return { re: a.re * s, im: a.im * s };
}

export function cxNeg(a: Complex): Complex {
  return { re: -a.re, im: -a.im };
}

export const CX_ZERO: Complex = { re: 0, im: 0 };
export const CX_ONE: Complex = { re: 1, im: 0 };
export const CX_I: Complex = { re: 0, im: 1 };

export type ComplexVector = Complex[];
export type ComplexMatrix = Complex[][];

export function cvNormSq(v: ComplexVector): number {
  return v.reduce((sum, amp) => sum + cxMagSq(amp), 0);
}

export function cvNorm(v: ComplexVector): number {
  return Math.sqrt(cvNormSq(v));
}

export function cvNormalize(v: ComplexVector): ComplexVector {
  const n = cvNorm(v);
  if (n === 0) throw new Error('Cannot normalize zero vector');
  return v.map(amp => cxScale(amp, 1 / n));
}

export function cvAdd(a: ComplexVector, b: ComplexVector): ComplexVector {
  if (a.length !== b.length) throw new Error('Vector dimension mismatch');
  return a.map((ai, i) => cxAdd(ai, b[i]));
}

export function cvSub(a: ComplexVector, b: ComplexVector): ComplexVector {
  if (a.length !== b.length) throw new Error('Vector dimension mismatch');
  return a.map((ai, i) => cxSub(ai, b[i]));
}

export function cvScale(v: ComplexVector, s: number): ComplexVector {
  return v.map(amp => cxScale(amp, s));
}

export function cvInnerProduct(a: ComplexVector, b: ComplexVector): Complex {
  if (a.length !== b.length) throw new Error('Vector dimension mismatch');
  return a.reduce((sum, ai, i) => cxAdd(sum, cxMul(cxConj(ai), b[i])), CX_ZERO);
}

export function matVecMul(M: ComplexMatrix, v: ComplexVector): ComplexVector {
  if (M.length === 0 || M[0].length !== v.length) throw new Error('Matrix-vector dimension mismatch');
  return M.map(row =>
    row.reduce((sum, mij, j) => cxAdd(sum, cxMul(mij, v[j])), CX_ZERO)
  );
}

export function matMul(A: ComplexMatrix, B: ComplexMatrix): ComplexMatrix {
  const rows = A.length;
  const cols = B[0]?.length ?? 0;
  const inner = B.length;
  if (A[0]?.length !== inner) throw new Error('Matrix dimension mismatch');
  const result: ComplexMatrix = [];
  for (let i = 0; i < rows; i++) {
    const row: ComplexVector = [];
    for (let j = 0; j < cols; j++) {
      let sum = CX_ZERO;
      for (let k = 0; k < inner; k++) {
        sum = cxAdd(sum, cxMul(A[i][k], B[k][j]));
      }
      row.push(sum);
    }
    result.push(row);
  }
  return result;
}

export function matDagger(M: ComplexMatrix): ComplexMatrix {
  const rows = M.length;
  const cols = M[0]?.length ?? 0;
  const result: ComplexMatrix = [];
  for (let j = 0; j < cols; j++) {
    const row: ComplexVector = [];
    for (let i = 0; i < rows; i++) {
      row.push(cxConj(M[i][j]));
    }
    result.push(row);
  }
  return result;
}

export function identityMatrix(d: number): ComplexMatrix {
  const result: ComplexMatrix = [];
  for (let i = 0; i < d; i++) {
    const row: ComplexVector = [];
    for (let j = 0; j < d; j++) {
      row.push(i === j ? CX_ONE : CX_ZERO);
    }
    result.push(row);
  }
  return result;
}

export function cxApproxEqual(a: Complex, b: Complex, eps: number = 1e-10): boolean {
  return Math.abs(a.re - b.re) < eps && Math.abs(a.im - b.im) < eps;
}
