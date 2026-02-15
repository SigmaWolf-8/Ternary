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
 * # HPTP Symplectic Jitter Corrector
 *
 * Applies Hamiltonian-inspired symplectic correction to femtosecond timestamps,
 * reducing cumulative drift in HPTP timing over long observation windows.
 *
 * ## Theory
 *
 * We model jitter correction as a harmonic oscillator in phase space (q, p):
 *   - q = cumulative position error (femtoseconds)
 *   - p = correction momentum (femtoseconds/step)
 *
 * Hamiltonian: H(q, p) = p²/2 + (ω²·q²)/2
 * where ω = 1/T(7) = 1/13 is the constraint frequency.
 *
 * The leapfrog (Störmer–Verlet) integrator preserves H to O(h⁴):
 *   p_{n+1/2} = p_n     - (h/2) · ω² · q_n
 *   q_{n+1}   = q_n     + h · p_{n+1/2}
 *   p_{n+1}   = p_{n+1/2} - (h/2) · ω² · q_{n+1}
 *
 * All arithmetic uses floating-point scaled values to avoid integer truncation
 * that would break the symplectic property. The corrected timestamp is
 * computed from the updated position error q.
 *
 * ## Pragmatic Benefit
 *
 * Reduces cumulative error in multi-calendar synchronization (30,000+ years),
 * improving reliability for blockchain and distributed timing endpoints.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { TRIBONACCI_SEQUENCE } from '@shared/tribonacci-constants';

const OMEGA = 1 / TRIBONACCI_SEQUENCE[7]; // 1/13 — constraint frequency
const OMEGA_SQ = OMEGA * OMEGA;           // ω²

export interface SymplecticCorrectionResult {
  correctedTimestamp: bigint;
  position: number;
  momentum: number;
  invariant: number;
  correctionApplied: bigint;
}

export interface SymplecticState {
  position: number;
  momentum: number;
}

/**
 * Computes the Hamiltonian: H = p²/2 + ω²·q²/2
 */
export function computeHamiltonian(q: number, p: number): number {
  return (p * p) / 2 + (OMEGA_SQ * q * q) / 2;
}

/**
 * Applies a single symplectic (leapfrog) jitter correction step.
 *
 * The jitter delta is added to the position error, then the leapfrog
 * integrator evolves (q, p) one step. The correction applied to the
 * timestamp is the negative of the new position error.
 *
 * @param currentTimestamp  Femtosecond timestamp
 * @param jitterDelta       Measured jitter offset in femtoseconds
 * @param prevState         Previous symplectic state
 * @param h                 Step size (default 1.0)
 * @returns Corrected timestamp and updated symplectic state
 */
export function applySymplecticJitterCorrection(
  currentTimestamp: bigint,
  jitterDelta: bigint,
  prevState: SymplecticState = { position: 0, momentum: 0 },
  h: number = 1.0
): SymplecticCorrectionResult {
  let q = prevState.position + Number(jitterDelta);
  let p = prevState.momentum;

  // Leapfrog step 1: half-kick momentum
  p = p - (h / 2) * OMEGA_SQ * q;

  // Leapfrog step 2: full drift position
  q = q + h * p;

  // Leapfrog step 3: half-kick momentum
  p = p - (h / 2) * OMEGA_SQ * q;

  const correctionFs = BigInt(Math.round(-q));
  const correctedTimestamp = currentTimestamp + correctionFs;
  const invariant = computeHamiltonian(q, p);

  return {
    correctedTimestamp,
    position: q,
    momentum: p,
    invariant,
    correctionApplied: correctionFs,
  };
}

/**
 * Applies symplectic correction to a batch of jitter samples.
 * Tracks energy conservation across the full sequence.
 *
 * @param samples Array of { timestamp, jitterDelta } pairs
 * @returns Corrected samples and final symplectic state
 */
export function correctJitterBatch(
  samples: Array<{ timestamp: bigint; jitterDelta: bigint }>
): {
  correctedSamples: SymplecticCorrectionResult[];
  finalState: SymplecticState;
  energyDrift: number;
} {
  let state: SymplecticState = { position: 0, momentum: 0 };
  const correctedSamples: SymplecticCorrectionResult[] = [];
  let initialInvariant: number | null = null;

  for (const sample of samples) {
    const result = applySymplecticJitterCorrection(
      sample.timestamp,
      sample.jitterDelta,
      state
    );

    if (initialInvariant === null && result.invariant > 0) {
      initialInvariant = result.invariant;
    }

    state = {
      position: result.position,
      momentum: result.momentum,
    };

    correctedSamples.push(result);
  }

  const energyDrift =
    initialInvariant !== null && initialInvariant > 0
      ? Math.abs(state.position !== 0 || state.momentum !== 0
          ? Math.abs(computeHamiltonian(state.position, state.momentum) - initialInvariant) / initialInvariant
          : 0)
      : 0;

  return { correctedSamples, finalState: state, energyDrift };
}

/**
 * Verifies that the symplectic integrator conserves energy within tolerance.
 * For a true symplectic map, energy drift should remain bounded (not grow linearly).
 *
 * @param results Sequence of correction results
 * @param toleranceFraction Maximum allowed relative energy drift (default 0.01 = 1%)
 * @returns Whether energy is conserved within tolerance
 */
export function verifyEnergyConservation(
  results: SymplecticCorrectionResult[],
  toleranceFraction: number = 0.01
): { conserved: boolean; maxDrift: number; avgDrift: number } {
  if (results.length < 2) {
    return { conserved: true, maxDrift: 0, avgDrift: 0 };
  }

  const nonZeroResults = results.filter(r => r.invariant > 0);
  if (nonZeroResults.length < 2) {
    return { conserved: true, maxDrift: 0, avgDrift: 0 };
  }

  const baseInvariant = nonZeroResults[0].invariant;
  let maxDrift = 0;
  let totalDrift = 0;

  for (let i = 1; i < nonZeroResults.length; i++) {
    const drift = Math.abs(nonZeroResults[i].invariant - baseInvariant) / Math.abs(baseInvariant);
    if (drift > maxDrift) maxDrift = drift;
    totalDrift += drift;
  }

  const avgDrift = totalDrift / (nonZeroResults.length - 1);

  return {
    conserved: maxDrift <= toleranceFraction,
    maxDrift,
    avgDrift,
  };
}
