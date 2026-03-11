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
 * # Lagrangian Qutrit Utilities
 *
 * Discrete Euler-Lagrange equations for qutrit state evolution, integrated
 * with SUFT dynamics and Tribonacci variational potentials.
 *
 * Extends the classical Lagrangian ternary utils to quantum amplitudes:
 * qutrit branches carry complex amplitudes while the discrete EL updates
 * operate on their real projections (balanced trits) for ternary consistency.
 *
 * ## Key Functions
 *
 * - quantumElBranchUpdate: EL evolution of 3 qutrit branches
 * - triboQutritPotential: Tribonacci-weighted harmonic potential V_Trib
 * - discreteQutritAction: Full SUFT + Tribonacci per-site Lagrangian
 *
 * All coefficients derive from SUFT constants via plenum-square.ts.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  type Complex,
  cx, cxAdd, cxSub, cxMagSq,
} from './complex-utils';

import {
  SUFT_RADIUS,
  SUFT_COSMIC_CIRCUMFERENCE,
  MASS_SHELL_RATIO,
  TEMPORAL_CROSS_DENOM,
  ENERGY_CROSS_DENOM,
} from './plenum-square';

import { TRIBONACCI_SEQUENCE } from './tribonacci-constants';

import {
  toTrit,
  elUpdateEDotMinus1,
  elUpdateEDot0,
  elUpdateEDotPlus1,
} from './lagrangian-ternary-utils';

import type { Trit } from './lagrangian-ternary-utils';

const PHI = (1 + Math.sqrt(5)) / 2;
const CROSS_COEFF = PHI / (2 * TEMPORAL_CROSS_DENOM);
const ENERGY_CROSS_COEFF = 1 / (2 * ENERGY_CROSS_DENOM);

export interface QutritBranch {
  t_alpha: Complex;
  E_alpha: Complex;
}

export function quantumElBranchUpdate(
  branches: [QutritBranch, QutritBranch, QutritBranch],
  mu_trits: [Trit, Trit, Trit],
  dot_trits: [Complex, Complex, Complex]
): [QutritBranch, QutritBranch, QutritBranch] {
  const dot_t_minus1 = toTrit(Math.round(dot_trits[0].re)) as Trit;
  const dot_t_plus1 = toTrit(Math.round(dot_trits[2].re)) as Trit;

  const t_minus1 = toTrit(Math.round(branches[0].t_alpha.re)) as Trit;
  const t_0 = toTrit(Math.round(branches[1].t_alpha.re)) as Trit;
  const t_plus1 = toTrit(Math.round(branches[2].t_alpha.re)) as Trit;

  const E_minus1 = toTrit(Math.round(branches[0].E_alpha.re)) as Trit;
  const E_0 = toTrit(Math.round(branches[1].E_alpha.re)) as Trit;
  const E_plus1 = toTrit(Math.round(branches[2].E_alpha.re)) as Trit;

  const updatedE_minus1 = elUpdateEDotMinus1(dot_t_plus1, mu_trits[0], t_minus1);
  const updatedE_0 = elUpdateEDot0(mu_trits[1], t_0);
  const updatedE_plus1 = elUpdateEDotPlus1(dot_t_minus1, mu_trits[2], t_plus1);

  return [
    { t_alpha: branches[0].t_alpha, E_alpha: cx(updatedE_minus1) },
    { t_alpha: branches[1].t_alpha, E_alpha: cx(updatedE_0) },
    { t_alpha: branches[2].t_alpha, E_alpha: cx(updatedE_plus1) },
  ];
}

export function triboQutritPotential(
  n: number,
  branchAmplitudes: [Complex, Complex, Complex]
): number {
  if (n < 0) throw new Error('Site index n must be non-negative');
  const seqLen = TRIBONACCI_SEQUENCE.length;
  const T_n = TRIBONACCI_SEQUENCE[n % seqLen];
  const sumReal = branchAmplitudes.reduce((s, amp) => s + amp.re, 0);
  return 0.5 * (T_n - sumReal) ** 2;
}

export function discreteQutritAction(
  state_n: [Complex, Complex, Complex],
  state_n1: [Complex, Complex, Complex],
  lambda_n: number,
  mu_n: [number, number, number],
  n: number,
  m2: number = 0
): number {
  const diff: [number, number, number] = [
    state_n1[0].re - state_n[0].re,
    state_n1[1].re - state_n[1].re,
    state_n1[2].re - state_n[2].re,
  ];

  const crossTerm = CROSS_COEFF * (state_n[0].re * diff[2] - state_n[2].re * diff[0]);

  const E_n: [number, number, number] = [state_n[0].im, state_n[1].im, state_n[2].im];
  const dE: [number, number, number] = [
    state_n1[0].im - E_n[0],
    state_n1[1].im - E_n[1],
    state_n1[2].im - E_n[2],
  ];
  const energyCross = ENERGY_CROSS_COEFF * (E_n[0] * dE[2] - E_n[2] * dE[0]);

  const normSq = state_n.reduce((s, amp) => s + cxMagSq(amp), 0);
  const constraint = lambda_n * (MASS_SHELL_RATIO * normSq + m2);

  const periodTerm = mu_n.reduce(
    (sum, mu, i) =>
      sum + mu * (Math.cos((2 * Math.PI * state_n[i].re) / SUFT_COSMIC_CIRCUMFERENCE) - 1),
    0
  );

  const V_trib = triboQutritPotential(n, state_n);

  return crossTerm + energyCross - constraint + periodTerm + V_trib;
}

export function discreteQutritActionSum(
  trajectory: [Complex, Complex, Complex][],
  lambdas: number[],
  mus: [number, number, number][],
  m2: number = 0
): number {
  if (trajectory.length < 2) throw new Error('Trajectory must have at least 2 sites');
  if (lambdas.length !== trajectory.length - 1) throw new Error('Lambda count mismatch');
  if (mus.length !== trajectory.length - 1) throw new Error('Mu count mismatch');

  let total = 0;
  for (let n = 0; n < trajectory.length - 1; n++) {
    total += discreteQutritAction(trajectory[n], trajectory[n + 1], lambdas[n], mus[n], n, m2);
  }
  return total;
}

export function qutritBranchRealProjection(
  branches: [QutritBranch, QutritBranch, QutritBranch]
): [Trit, Trit, Trit] {
  return [
    toTrit(Math.round(branches[0].t_alpha.re)),
    toTrit(Math.round(branches[1].t_alpha.re)),
    toTrit(Math.round(branches[2].t_alpha.re)),
  ];
}

export function qutritEnergyProjection(
  branches: [QutritBranch, QutritBranch, QutritBranch]
): [Trit, Trit, Trit] {
  return [
    toTrit(Math.round(branches[0].E_alpha.re)),
    toTrit(Math.round(branches[1].E_alpha.re)),
    toTrit(Math.round(branches[2].E_alpha.re)),
  ];
}
