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
 * # Variational Methods Applied to the Tribonacci Sequence
 *
 * Constructs discrete functionals whose stationary conditions reproduce
 * the Tribonacci recurrence T_n = T_{n-1} + T_{n-2} + T_{n-3} exactly.
 *
 * ## Discrete Variational Principle
 *
 * The action functional S = Σ L(n) with penalty Lagrangian
 *   L(n) = ½(T_n − T_{n-1} − T_{n-2} − T_{n-3})²
 * has stationarity condition δS/δT_n = 0 iff T_n satisfies the recurrence.
 * Zero action ↔ exact Tribonacci.
 *
 * ## Coupled SUFT-Tribonacci Action
 *
 * The full discrete Lagrangian per site n integrates the SUFT dynamics
 * with a Tribonacci-weighted potential:
 *   V_Trib(n) = ½(T_n − (t₊₁ + t₀ + t₋₁))²
 * pulling branch sums toward Tribonacci values at each step.
 *
 * ## Tribonacci Constant τ
 *
 * The characteristic equation r³ − r² − r − 1 = 0 has dominant root
 * τ ≈ 1.8392867552141611 (OEIS A058265). Asymptotically T_n ∝ τ^n.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { TRIBONACCI_SEQUENCE, TAU } from './tribonacci-constants';
import {
  SUFT_COSMIC_CIRCUMFERENCE,
  MASS_SHELL_RATIO,
  TEMPORAL_CROSS_DENOM,
  ENERGY_CROSS_DENOM,
} from './saturnian-blueprint';
import type { Trit } from './lagrangian-ternary-utils';

const PHI = (1 + Math.sqrt(5)) / 2;
const CROSS_COEFF = PHI / (2 * TEMPORAL_CROSS_DENOM);    // φ/26, from SUFT_RADIUS=13
const ENERGY_CROSS_COEFF = 1 / (2 * ENERGY_CROSS_DENOM); // 1/56, from SUFT_LUNAR_HARMONIC=28

type TritTriple = [Trit, Trit, Trit];

/**
 * Computes the Tribonacci action deviation (penalty functional).
 *
 * S = Σ_{n=startIndex}^{N-1} ½(seq[n] − seq[n-1] − seq[n-2] − seq[n-3])²
 *
 * Zero iff the sequence satisfies the exact Tribonacci recurrence.
 * Non-zero values measure "distance" from the Tribonacci geodesic.
 *
 * @param sequence Array of numbers [T_0, T_1, ..., T_N]
 * @param startIndex First index to check (must be ≥ 3)
 * @returns Total squared deviation (action value)
 */
export function tribonacciActionDeviation(
  sequence: number[],
  startIndex: number = 3
): number {
  let deviation = 0;
  for (let n = startIndex; n < sequence.length; n++) {
    const predicted = sequence[n - 1] + sequence[n - 2] + sequence[n - 3];
    deviation += (sequence[n] - predicted) ** 2;
  }
  return deviation;
}

/**
 * Checks whether a sequence is an exact variational minimum (zero action).
 *
 * @param sequence Array of numbers to validate
 * @returns True iff sequence satisfies T_n = T_{n-1} + T_{n-2} + T_{n-3}
 */
export function isExactTribonacci(sequence: number[]): boolean {
  if (sequence.length < 4) return true;
  return tribonacciActionDeviation(sequence) === 0;
}

/**
 * Generates the canonical Tribonacci sequence up to index n.
 *
 * Uses the standard initial conditions T_0 = 0, T_1 = 0, T_2 = 1.
 *
 * @param n Number of terms to generate (minimum 3)
 * @returns Array of Tribonacci numbers [T_0, ..., T_{n-1}]
 */
export function generateTribonacci(n: number): number[] {
  if (n <= 0) return [];
  if (n === 1) return [0];
  if (n === 2) return [0, 0];
  const seq = [0, 0, 1];
  for (let i = 3; i < n; i++) {
    seq.push(seq[i - 1] + seq[i - 2] + seq[i - 3]);
  }
  return seq;
}

/**
 * Computes the per-site deviation from the Tribonacci recurrence.
 *
 * Returns the residual r_n = T_n − T_{n-1} − T_{n-2} − T_{n-3} at each site,
 * along with the squared penalty.
 *
 * @param sequence Sequence to analyze
 * @returns Array of per-site diagnostics
 */
export function tribonacciResiduals(
  sequence: number[]
): Array<{ n: number; residual: number; penalty: number }> {
  const results: Array<{ n: number; residual: number; penalty: number }> = [];
  for (let n = 3; n < sequence.length; n++) {
    const predicted = sequence[n - 1] + sequence[n - 2] + sequence[n - 3];
    const residual = sequence[n] - predicted;
    results.push({ n, residual, penalty: residual ** 2 });
  }
  return results;
}

/**
 * Computes the Tribonacci ratio convergence toward τ.
 *
 * For large n, T_n / T_{n-1} → τ ≈ 1.8392867552141611.
 * Returns the ratio and absolute error at each step.
 *
 * @param sequence Tribonacci sequence (or candidate)
 * @returns Array of ratio convergence data
 */
export function tribonacciRatioConvergence(
  sequence: number[]
): Array<{ n: number; ratio: number; error: number }> {
  const results: Array<{ n: number; ratio: number; error: number }> = [];
  for (let n = 1; n < sequence.length; n++) {
    if (sequence[n - 1] === 0) continue;
    const ratio = sequence[n] / sequence[n - 1];
    results.push({ n, ratio, error: Math.abs(ratio - TAU) });
  }
  return results;
}

/**
 * Computes the Tribonacci-weighted potential V_Trib for the coupled
 * SUFT-Tribonacci discrete action.
 *
 * V_Trib(n, t) = ½(T_n − (t₊₁ + t₀ + t₋₁))²
 *
 * This harmonic potential pulls the branch sum toward the n-th
 * Tribonacci number, embedding the recurrence into SUFT dynamics.
 *
 * @param n Discrete step index (for T_n lookup)
 * @param branches Branch states [t₋₁, t₀, t₊₁]
 * @returns Potential energy value
 */
export function tribonacciPotential(n: number, branches: TritTriple): number {
  const Tn = n < TRIBONACCI_SEQUENCE.length ? TRIBONACCI_SEQUENCE[n] : generateTribonacci(n + 1)[n];
  const branchSum = branches[0] + branches[1] + branches[2];
  return 0.5 * (Tn - branchSum) ** 2;
}

/**
 * Computes the gradient of the Tribonacci potential w.r.t. branch states.
 *
 * ∂V_Trib/∂t_α = −(T_n − Σt_α) for each α.
 * All three branches receive the same gradient (symmetric coupling).
 *
 * @param n Step index
 * @param branches Branch states [t₋₁, t₀, t₊₁]
 * @returns Gradient triple [∂V/∂t₋₁, ∂V/∂t₀, ∂V/∂t₊₁]
 */
export function tribonacciPotentialGradient(
  n: number,
  branches: TritTriple
): [number, number, number] {
  const Tn = n < TRIBONACCI_SEQUENCE.length ? TRIBONACCI_SEQUENCE[n] : generateTribonacci(n + 1)[n];
  const branchSum = branches[0] + branches[1] + branches[2];
  const grad = -(Tn - branchSum);
  return [grad, grad, grad];
}

/**
 * Evaluates the full coupled SUFT-Tribonacci discrete Lagrangian at site n.
 *
 * L_n = Σ E_α · Δt_α − λΦ + (φ/26)(cross terms) + (1/56)(energy cross)
 *       + Σ μ_α[cos(2πt_α/364) − 1] + V_Trib(n, t)
 *
 * @param site Discrete site data
 * @returns Per-site Lagrangian value
 */
export function coupledLagrangianSite(site: {
  n: number;
  t: TritTriple;
  tNext: TritTriple;
  E: TritTriple;
  ENext: TritTriple;
  lambda: Trit;
  mu: TritTriple;
  p_mu: number[];
  m2?: number;
}): number {
  const dt: [number, number, number] = [
    site.tNext[0] - site.t[0],
    site.tNext[1] - site.t[1],
    site.tNext[2] - site.t[2],
  ];
  const dE: [number, number, number] = [
    site.ENext[0] - site.E[0],
    site.ENext[1] - site.E[1],
    site.ENext[2] - site.E[2],
  ];

  let kineticTerm = 0;
  for (let alpha = 0; alpha < 3; alpha++) {
    kineticTerm += site.E[alpha] * dt[alpha];
  }

  const sumP2 = site.p_mu.reduce((s, p) => s + p * p, 0);
  const sumE2 = site.E.reduce((s: number, E: number) => s + E * E, 0);
  const m2 = site.m2 ?? 0;
  const massShell = sumP2 + MASS_SHELL_RATIO * sumE2 + m2;
  const constraintTerm = -site.lambda * massShell;

  const temporalCross = CROSS_COEFF * (site.t[0] * dt[2] - site.t[2] * dt[0]);

  const energyCross = ENERGY_CROSS_COEFF * (site.E[0] * dE[2] - site.E[2] * dE[0]);

  let periodicityTerm = 0;
  for (let alpha = 0; alpha < 3; alpha++) {
    periodicityTerm += site.mu[alpha] * (Math.cos(2 * Math.PI * site.t[alpha] / SUFT_COSMIC_CIRCUMFERENCE) - 1);
  }

  const vTrib = tribonacciPotential(site.n, site.t);

  return kineticTerm + constraintTerm + temporalCross + energyCross + periodicityTerm + vTrib;
}

/**
 * Computes the total discrete action S = Σ L_n over a chain of sites.
 *
 * @param sites Array of site data (ordered by n)
 * @returns Total action value
 */
export function discreteAction(
  sites: Array<{
    n: number;
    t: TritTriple;
    tNext: TritTriple;
    E: TritTriple;
    ENext: TritTriple;
    lambda: Trit;
    mu: TritTriple;
    p_mu: number[];
  }>
): number {
  let action = 0;
  for (const site of sites) {
    action += coupledLagrangianSite(site);
  }
  return action;
}

/**
 * Verifies that the canonical Tribonacci sequence (from shared constants)
 * has zero variational action, confirming it lies on the geodesic.
 *
 * @returns Diagnostic with action value and validity
 */
export function verifyCanonicalTribonacci(): {
  valid: boolean;
  action: number;
  length: number;
} {
  const action = tribonacciActionDeviation(TRIBONACCI_SEQUENCE);
  return {
    valid: action === 0,
    action,
    length: TRIBONACCI_SEQUENCE.length,
  };
}

/**
 * Computes the "variational fitness" of a candidate sequence.
 *
 * Returns a normalized score in [0, 1] where 1 = exact Tribonacci.
 * For sequences with all-zero deviations, returns 1.0.
 * Otherwise, fitness = 1 / (1 + action).
 *
 * @param sequence Candidate sequence
 * @returns Fitness score in [0, 1]
 */
export function variationalFitness(sequence: number[]): number {
  if (sequence.length < 4) return 1.0;
  const action = tribonacciActionDeviation(sequence);
  if (action === 0) return 1.0;
  return 1 / (1 + action);
}
