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
 * # Generalized Qudit Basics — Higher-Dimensional Quantum States
 *
 * Extends qutrit (d=3) to arbitrary dimension d ≥ 2. A single qudit
 * encodes log₂(d) bits, offering denser information encoding, improved
 * fault tolerance, and compact multi-controlled gates.
 *
 * ## Key Operations
 *
 * - Arbitrary-d basis states and normalization
 * - Generalized shift (X_d) and clock (Z_d) operators
 * - Higher-d error simulation (phase, leakage, depolarizing)
 * - SUFT-scaled stabilizer code parameters [[9m−k, k, 2]]_d
 * - Born probabilities and overlap (fidelity)
 *
 * Hardware mapping: d=3 recovers qutrit; d=4 ← ququart; d=5 ← ququint.
 * For d=3, use the specialized qutrit-basics.ts module instead.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  type Complex,
  type ComplexVector,
  type ComplexMatrix,
  cx, cxFromPolar, cxAdd, cxMul, cxConj, cxScale,
  cxMagSq, cvNorm, cvNormSq, cvNormalize,
  cvInnerProduct, matVecMul,
  CX_ZERO, CX_ONE,
  identityMatrix,
} from './complex-utils';

import { SUFT_RADIUS, SUFT_COSMIC_CIRCUMFERENCE } from './saturnian-blueprint';

export type QuditState = ComplexVector;

export function quditBasisState(level: number, dimension: number): QuditState {
  if (dimension < 2) throw new Error('Dimension must be >= 2');
  if (level < 0 || level >= dimension) throw new Error(`Level must be in [0, ${dimension - 1}]`);
  const state: QuditState = Array(dimension).fill(null).map(() => CX_ZERO);
  state[level] = CX_ONE;
  return state;
}

export function quditFromAmplitudes(amplitudes: Complex[]): QuditState {
  if (amplitudes.length < 2) throw new Error('Need at least 2 amplitudes');
  return [...amplitudes];
}

export function normalizeQudit(state: QuditState): QuditState {
  return cvNormalize(state);
}

export function quditNormSq(state: QuditState): number {
  return cvNormSq(state);
}

export function quditBornProbabilities(state: QuditState): number[] {
  return state.map(amp => cxMagSq(amp));
}

export function quditShiftOperator(d: number): ComplexMatrix {
  if (d < 2) throw new Error('Dimension must be >= 2');
  const M: ComplexMatrix = [];
  for (let i = 0; i < d; i++) {
    const row: ComplexVector = Array(d).fill(null).map(() => CX_ZERO);
    row[((i - 1) % d + d) % d] = CX_ONE;
    M.push(row);
  }
  return M;
}

export function quditClockOperator(d: number): ComplexMatrix {
  if (d < 2) throw new Error('Dimension must be >= 2');
  const M: ComplexMatrix = [];
  for (let i = 0; i < d; i++) {
    const row: ComplexVector = Array(d).fill(null).map(() => CX_ZERO);
    row[i] = cxFromPolar(1, (2 * Math.PI * i) / d);
    M.push(row);
  }
  return M;
}

export function applyQuditGate(gate: ComplexMatrix, state: QuditState): QuditState {
  return matVecMul(gate, state);
}

export type QuditErrorType = 'none' | 'phase' | 'leak' | 'depolarize';

export function applyQuditError(state: QuditState, errorType: QuditErrorType): QuditState {
  const d = state.length;

  switch (errorType) {
    case 'none':
      return [...state];

    case 'phase': {
      return state.map((amp, i) => {
        const phase = cxFromPolar(1, (2 * Math.PI * i) / d);
        return cxMul(amp, phase);
      });
    }

    case 'leak': {
      const factor = cx(1 / Math.sqrt(2));
      const result = state.map(amp => cxMul(amp, factor));
      result[d - 1] = cxAdd(result[d - 1], cx(1 / Math.sqrt(2)));
      return result;
    }

    case 'depolarize': {
      const uniform = cx(1 / Math.sqrt(d));
      return state.map(() => uniform);
    }
  }
}

export function quditOverlap(a: QuditState, b: QuditState): number {
  if (a.length !== b.length) throw new Error('Dimension mismatch');
  const inner = cvInnerProduct(a, b);
  return cxMagSq(inner);
}

export function quditFidelity(state: QuditState, target: QuditState): number {
  return quditOverlap(state, target);
}

export function quditDepolarizingChannel(
  state: QuditState,
  errorRate: number
): QuditState {
  const d = state.length;
  if (errorRate < 0 || errorRate > 1) throw new Error('Error rate must be in [0, 1]');

  const retainFactor = 1 - errorRate;
  const mixFactor = errorRate / d;

  const result = state.map(amp =>
    cxAdd(cxScale(amp, Math.sqrt(retainFactor)), cx(Math.sqrt(mixFactor)))
  );

  const norm = cvNorm(result);
  if (norm > 0) return result.map(a => cxScale(a, 1 / norm));
  return result;
}

export function quditCodeParameters(m: number, d: number): {
  n: number;
  k: number;
  dist: number;
  overhead: number;
  yieldGamma: number;
} {
  if (m < 1) throw new Error('Code parameter m must be >= 1');
  if (d < 2) throw new Error('Dimension must be >= 2');

  const k = (d * m) - (d - 1);
  const n = (d * d * m) - k;
  const dist = 2;
  const overhead = n / k;
  const yieldGamma = Math.log2(2 + (2 * d) / k);

  return { n, k, dist, overhead, yieldGamma };
}

export function suftScaledQuditPhaseGate(d: number, theta: number): ComplexMatrix {
  if (d < 2) throw new Error('Dimension must be >= 2');
  const PHI = (1 + Math.sqrt(5)) / 2;
  const scaledTheta = (PHI / SUFT_RADIUS) * theta;

  const M: ComplexMatrix = [];
  for (let i = 0; i < d; i++) {
    const row: ComplexVector = Array(d).fill(null).map(() => CX_ZERO);
    const levelPhase = scaledTheta * (i - Math.floor(d / 2));
    row[i] = cxFromPolar(1, levelPhase);
    M.push(row);
  }
  return M;
}

export function quditDimension(state: QuditState): number {
  return state.length;
}

export function isValidQuditState(state: QuditState, eps: number = 1e-10): boolean {
  if (state.length < 2) return false;
  return Math.abs(cvNormSq(state) - 1) < eps;
}

export function quditMaxEntangled(d: number): QuditState {
  if (d < 2) throw new Error('Dimension must be >= 2');
  const amp = cx(1 / Math.sqrt(d));
  return Array(d).fill(null).map(() => amp);
}

export const SUPPORTED_DIMENSIONS = [2, 3, 4, 5, 7, 8, 9, 13] as const;

export function infoDensityBits(d: number): number {
  return Math.log2(d);
}
