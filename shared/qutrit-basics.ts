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
 * # Qutrit Basics — SUFT-Coupled Quantum Ternary States
 *
 * Defines qutrit (3-level quantum system) states and operations coupled
 * to the SUFT ternary branches (Moon.Ra past, Amun.Ra present, SUN.Ra future).
 *
 * Basis states: |−1⟩ = [1,0,0], |0⟩ = [0,1,0], |+1⟩ = [0,0,1]
 *
 * Includes:
 * - SUFT branch basis states
 * - Normalization (exact, symbolic)
 * - SUFT ternary phase gate (e^{i φ/13 θ} on branches)
 * - Gell-Mann generators (SU(3) basis for qutrit unitaries)
 * - Qutrit shift (X) and clock (Z) operators
 * - Born probabilities and expectation values
 *
 * All operations use the lightweight complex utilities from complex-utils.ts.
 * No external dependencies. 27 registers ↔ 9 qutrits (3 branches × 3 levels).
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  type Complex,
  type ComplexVector,
  type ComplexMatrix,
  cx, cxFromPolar, cxAdd, cxSub, cxMul, cxConj, cxScale,
  cxMag, cxMagSq, cvNorm, cvNormSq, cvNormalize,
  cvInnerProduct, matVecMul, matDagger,
  CX_ZERO, CX_ONE, CX_I,
  identityMatrix,
} from './complex-utils';

import {
  SUFT_RADIUS,
} from './plenum-square';

const PHI = (1 + Math.sqrt(5)) / 2;

export type QutritState = [Complex, Complex, Complex];

export function suftBasisState(alpha: -1 | 0 | 1): QutritState {
  if (alpha === -1) return [cx(1), cx(0), cx(0)];
  if (alpha === 0) return [cx(0), cx(1), cx(0)];
  return [cx(0), cx(0), cx(1)];
}

export function qutritFromAmplitudes(a0: Complex, a1: Complex, a2: Complex): QutritState {
  return [a0, a1, a2];
}

export function normalizeQutrit(state: QutritState): QutritState {
  const n = cvNorm(state);
  if (n === 0) throw new Error('Cannot normalize zero qutrit state');
  return [
    cxScale(state[0], 1 / n),
    cxScale(state[1], 1 / n),
    cxScale(state[2], 1 / n),
  ];
}

export function qutritNormSq(state: QutritState): number {
  return cvNormSq(state);
}

export function bornProbabilities(state: QutritState): [number, number, number] {
  return [
    cxMagSq(state[0]),
    cxMagSq(state[1]),
    cxMagSq(state[2]),
  ];
}

export function expectationValue(state: QutritState, operator: ComplexMatrix): Complex {
  const applied = matVecMul(operator, state) as QutritState;
  return cvInnerProduct(state, applied);
}

export function suftPhaseGate(theta: number): ComplexMatrix {
  const phase = cxFromPolar(1, (PHI / SUFT_RADIUS) * theta);
  const phaseConj = cxConj(phase);
  return [
    [phase,    CX_ZERO, CX_ZERO],
    [CX_ZERO,  CX_ONE,  CX_ZERO],
    [CX_ZERO,  CX_ZERO, phaseConj],
  ];
}

export function applyGate(gate: ComplexMatrix, state: QutritState): QutritState {
  const result = matVecMul(gate, state);
  return [result[0], result[1], result[2]];
}

export function qutritShiftOperator(): ComplexMatrix {
  return [
    [CX_ZERO, CX_ZERO, CX_ONE],
    [CX_ONE,  CX_ZERO, CX_ZERO],
    [CX_ZERO, CX_ONE,  CX_ZERO],
  ];
}

export function qutritClockOperator(): ComplexMatrix {
  const omega = cxFromPolar(1, (2 * Math.PI) / 3);
  const omega2 = cxMul(omega, omega);
  return [
    [CX_ONE,  CX_ZERO, CX_ZERO],
    [CX_ZERO, omega,   CX_ZERO],
    [CX_ZERO, CX_ZERO, omega2],
  ];
}

export function gellMannLambda(index: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8): ComplexMatrix {
  const Z = CX_ZERO;
  const O = CX_ONE;
  const nO = cx(-1);
  const I = CX_I;
  const nI = cx(0, -1);
  const inv3 = cx(1 / Math.sqrt(3));
  const n2inv3 = cx(-2 / Math.sqrt(3));

  switch (index) {
    case 1: return [[Z, O, Z], [O, Z, Z], [Z, Z, Z]];
    case 2: return [[Z, nI, Z], [I, Z, Z], [Z, Z, Z]];
    case 3: return [[O, Z, Z], [Z, nO, Z], [Z, Z, Z]];
    case 4: return [[Z, Z, O], [Z, Z, Z], [O, Z, Z]];
    case 5: return [[Z, Z, nI], [Z, Z, Z], [I, Z, Z]];
    case 6: return [[Z, Z, Z], [Z, Z, O], [Z, O, Z]];
    case 7: return [[Z, Z, Z], [Z, Z, nI], [Z, I, Z]];
    case 8: return [[inv3, Z, Z], [Z, inv3, Z], [Z, Z, n2inv3]];
  }
}

export function isUnitaryQutrit(gate: ComplexMatrix, eps: number = 1e-10): boolean {
  const dag = matDagger(gate);
  const id = identityMatrix(3);
  for (let i = 0; i < 3; i++) {
    const row: Complex[] = [];
    for (let j = 0; j < 3; j++) {
      let sum = CX_ZERO;
      for (let k = 0; k < 3; k++) {
        sum = cxAdd(sum, cxMul(gate[i][k], dag[k][j]));
      }
      if (cxMag(cxSub(sum, id[i][j])) > eps) return false;
    }
  }
  return true;
}

export function qutritOverlap(a: QutritState, b: QutritState): number {
  const inner = cvInnerProduct(a, b);
  return cxMagSq(inner);
}

export function suftBranchPhases(t_past: number, t_present: number, t_future: number): QutritState {
  const phiFactor = PHI / SUFT_RADIUS;
  return [
    cxFromPolar(1, phiFactor * t_past),
    cxFromPolar(1, phiFactor * t_present),
    cxFromPolar(1, phiFactor * t_future),
  ];
}

export const QUTRIT_IDENTITY: ComplexMatrix = identityMatrix(3);

export const QUTRIT_REVIVAL_PERIOD = 56; // φ^k seconds qutrit revival period
