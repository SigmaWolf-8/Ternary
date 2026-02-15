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
 * # Qutrit Fault Tolerance
 *
 * Classical simulation of qutrit fault-tolerant protocols for the
 * Ternary Computing Platform. Includes stabilizer codes, error
 * operators, triorthogonal magic state distillation, and syndrome
 * decoding — all scaled by SUFT constants.
 *
 * ## Protocols
 *
 * 1. **Error Operators**: Phase flip, leakage, and depolarizing noise
 *    on individual qutrits.
 *
 * 2. **[[3,1,2]]_3 Stabilizer Code**: Encodes 1 logical qutrit in 3
 *    physical qutrits. Detects single errors via syndrome measurement.
 *
 * 3. **Triorthogonal Distillation**: Simulates [[9m-k,k,2]]_3 codes
 *    for magic state preparation, scaled by SUFT_RADIUS (13).
 *
 * 4. **Syndrome Decoding**: Determines error position from measurement
 *    syndrome and applies correction.
 *
 * 27 registers = 9 qutrits (3 branches × 3 levels). All operations
 * are pure and exact. No external dependencies.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  type Complex,
  type ComplexVector,
  cx, cxMul, cxAdd, cxScale, cxConj, cxMagSq, cxFromPolar,
  cvNormSq, cvNormalize,
  CX_ZERO, CX_ONE, CX_I,
} from './complex-utils';

import { SUFT_RADIUS } from './saturnian-blueprint';

export type QutritState = [Complex, Complex, Complex];

export type ErrorType = 'none' | 'phase' | 'leak' | 'depolarize';

export function applyQutritError(state: QutritState, errorType: ErrorType): QutritState {
  switch (errorType) {
    case 'none':
      return [...state] as QutritState;

    case 'phase': {
      const omega = cxFromPolar(1, (2 * Math.PI) / 3);
      const omega2 = cxMul(omega, omega);
      return [state[0], cxMul(state[1], omega), cxMul(state[2], omega2)];
    }

    case 'leak': {
      const factor = cx(1 / Math.sqrt(2));
      const leakAmp = cx(1 / Math.sqrt(2));
      return [
        cxMul(state[0], factor),
        cxMul(state[1], factor),
        cxAdd(cxMul(state[2], factor), leakAmp),
      ];
    }

    case 'depolarize': {
      const mixed = cx(1 / Math.sqrt(3));
      return [mixed, mixed, mixed];
    }
  }
}

export function encodeQutritStabilizer(logical: QutritState): [QutritState, QutritState, QutritState] {
  return [
    [...logical] as QutritState,
    [...logical] as QutritState,
    [...logical] as QutritState,
  ];
}

export function measureSyndrome(
  encoded: [QutritState, QutritState, QutritState]
): [number, number] {
  let s1 = 0;
  let s2 = 0;

  for (let i = 0; i < 3; i++) {
    const a0 = encoded[0][i];
    const a1 = encoded[1][i];
    const a2 = encoded[2][i];
    s1 += cxMagSq({ re: a0.re - a1.re, im: a0.im - a1.im });
    s2 += cxMagSq({ re: a1.re - a2.re, im: a1.im - a2.im });
  }

  return [s1, s2];
}

export function correctQutritStabilizer(
  encoded: [QutritState, QutritState, QutritState]
): { corrected: QutritState; errorPosition: number | null } {
  const [s1, s2] = measureSyndrome(encoded);

  const eps = 1e-10;
  if (s1 < eps && s2 < eps) {
    return { corrected: [...encoded[0]] as QutritState, errorPosition: null };
  }

  if (s1 > eps && s2 < eps) {
    return { corrected: [...encoded[1]] as QutritState, errorPosition: 0 };
  }

  if (s1 < eps && s2 > eps) {
    return { corrected: [...encoded[0]] as QutritState, errorPosition: 2 };
  }

  const avg: QutritState = [CX_ZERO, CX_ZERO, CX_ZERO];
  for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 3; j++) {
      avg[j] = cxAdd(avg[j], encoded[i][j]);
    }
  }
  const result = avg.map(a => cxScale(a, 1 / 3)) as QutritState;
  return { corrected: result, errorPosition: 1 };
}

export function simulateTriorthogonalDistillation(
  m: number,
  inputStates: QutritState[]
): { distilled: QutritState; yieldGamma: number; codeParams: { n: number; k: number; d: number } } {
  if (m < 1) throw new Error('Code parameter m must be >= 1');
  if (inputStates.length === 0) throw new Error('Need at least one input state');

  const k = 3 * m - 2;
  const n = 9 * m - k;
  const d = 2;
  const yieldGamma = Math.log2(2 + 6 / k);

  const accumulated: QutritState = [CX_ZERO, CX_ZERO, CX_ZERO];
  for (const state of inputStates) {
    for (let j = 0; j < 3; j++) {
      accumulated[j] = cxAdd(accumulated[j], state[j]);
    }
  }

  const scaleFactor = 1 / (inputStates.length * SUFT_RADIUS);
  const distilled = accumulated.map(a => cxScale(a, scaleFactor)) as QutritState;

  const normSq = cvNormSq(distilled);
  const normalized = normSq > 0
    ? (cvNormalize(distilled) as QutritState)
    : distilled;

  return {
    distilled: normalized,
    yieldGamma,
    codeParams: { n, k, d },
  };
}

export function qutritFidelity(
  state: QutritState,
  target: QutritState
): number {
  let overlap = CX_ZERO;
  for (let i = 0; i < 3; i++) {
    overlap = cxAdd(overlap, cxMul(cxConj(target[i]), state[i]));
  }
  return cxMagSq(overlap);
}

export function detectError(
  encoded: [QutritState, QutritState, QutritState]
): { hasError: boolean; syndrome: [number, number]; estimatedPosition: number | null } {
  const [s1, s2] = measureSyndrome(encoded);
  const eps = 1e-10;

  if (s1 < eps && s2 < eps) {
    return { hasError: false, syndrome: [s1, s2], estimatedPosition: null };
  }

  let estimatedPosition: number | null = null;
  if (s1 > eps && s2 < eps) estimatedPosition = 0;
  else if (s1 < eps && s2 > eps) estimatedPosition = 2;
  else estimatedPosition = 1;

  return { hasError: true, syndrome: [s1, s2], estimatedPosition };
}

export function qutritDepolarizingChannel(
  state: QutritState,
  errorRate: number
): QutritState {
  if (errorRate < 0 || errorRate > 1) throw new Error('Error rate must be in [0, 1]');

  const retainFactor = 1 - errorRate;
  const mixFactor = errorRate / 3;

  const result: QutritState = [CX_ZERO, CX_ZERO, CX_ZERO];
  for (let i = 0; i < 3; i++) {
    result[i] = cxAdd(
      cxScale(state[i], Math.sqrt(retainFactor)),
      cx(Math.sqrt(mixFactor))
    );
  }

  const norm = Math.sqrt(cvNormSq(result));
  if (norm > 0) {
    return result.map(a => cxScale(a, 1 / norm)) as QutritState;
  }
  return result;
}

export function codeDistance(m: number): { n: number; k: number; d: number; overhead: number } {
  if (m < 1) throw new Error('Code parameter m must be >= 1');
  const k = 3 * m - 2;
  const n = 9 * m - k;
  return { n, k, d: 2, overhead: n / k };
}
