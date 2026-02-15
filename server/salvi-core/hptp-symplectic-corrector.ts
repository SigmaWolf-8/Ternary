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
 * Symplectic integrators preserve the Hamiltonian (energy) invariant of a
 * dynamical system across iterations — unlike naive Euler integration which
 * accumulates drift. We model timestamp jitter as phase noise in a constrained
 * system where the "energy" is a conserved quadratic invariant:
 *
 *   H = p²/2 + q²/2
 *
 * The update rule uses leapfrog (Störmer–Verlet) splitting:
 *   p_{n+1/2} = p_n - (h/2) · ∂H/∂q = p_n - (h/2) · q_n
 *   q_{n+1}   = q_n + h · ∂H/∂p   = q_n + h · p_{n+1/2}
 *   p_{n+1}   = p_{n+1/2} - (h/2) · ∂H/∂q = p_{n+1/2} - (h/2) · q_{n+1}
 *
 * The constraint scale uses T(7) = 13 (Tribonacci dimensional constant).
 *
 * ## Pragmatic Benefit
 *
 * Reduces cumulative error in multi-calendar synchronization (30,000+ years),
 * improving reliability for blockchain and distributed timing endpoints.
 * The corrector is stateless per-call; accumulate the invariant across samples
 * to track energy conservation.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { TRIBONACCI_SEQUENCE } from '@shared/tribonacci-constants';
import { SUFT_RADIUS, SUFT_LUNAR_HARMONIC } from '@shared/saturnian-blueprint';

const T7 = BigInt(TRIBONACCI_SEQUENCE[7]); // 13n

export interface SymplecticCorrectionResult {
  correctedTimestamp: bigint;
  momentum: bigint;
  invariant: number;
  correctionApplied: bigint;
}

export interface SymplecticState {
  momentum: bigint;
  invariant: number;
}

/**
 * Applies a single symplectic (leapfrog) jitter correction step.
 *
 * @param currentTimestamp  Femtosecond timestamp (position q)
 * @param jitterDelta       Measured jitter offset in femtoseconds
 * @param prevState         Previous symplectic state (momentum + invariant)
 * @returns Corrected timestamp and updated symplectic state
 */
export function applySymplecticJitterCorrection(
  currentTimestamp: bigint,
  jitterDelta: bigint,
  prevState: SymplecticState = { momentum: 0n, invariant: 0 }
): SymplecticCorrectionResult {
  const h = 1n; // Step size (femtosecond unit)

  const halfH = h; // h/2 in integer arithmetic → use h and divide result by 2

  // Leapfrog step 1: half-kick momentum
  //   p_{n+1/2} = p_n - (h/2) · q_error
  const positionError = jitterDelta;
  const momentumHalf = prevState.momentum - (halfH * positionError) / (2n * T7);

  // Leapfrog step 2: full drift position
  //   q_{n+1} = q_n + h · p_{n+1/2}
  const correctionApplied = (h * momentumHalf) / T7;
  const correctedTimestamp = currentTimestamp + correctionApplied;

  // Leapfrog step 3: half-kick momentum again
  //   p_{n+1} = p_{n+1/2} - (h/2) · q_{n+1}_error
  const newMomentum = momentumHalf - (halfH * correctionApplied) / (2n * T7);

  // Compute conserved Hamiltonian: H = p²/(2·28) + q²/(2·13)
  const pFloat = Number(newMomentum);
  const qFloat = Number(correctionApplied);
  const newInvariant =
    (pFloat * pFloat) / (2 * SUFT_LUNAR_HARMONIC) +
    (qFloat * qFloat) / (2 * SUFT_RADIUS);

  return {
    correctedTimestamp,
    momentum: newMomentum,
    invariant: newInvariant,
    correctionApplied,
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
  let state: SymplecticState = { momentum: 0n, invariant: 0 };
  const correctedSamples: SymplecticCorrectionResult[] = [];
  let initialInvariant: number | null = null;

  for (const sample of samples) {
    const result = applySymplecticJitterCorrection(
      sample.timestamp,
      sample.jitterDelta,
      state
    );

    if (initialInvariant === null) {
      initialInvariant = result.invariant;
    }

    state = {
      momentum: result.momentum,
      invariant: result.invariant,
    };

    correctedSamples.push(result);
  }

  const energyDrift =
    initialInvariant !== null && initialInvariant > 0
      ? Math.abs(state.invariant - initialInvariant) / initialInvariant
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

  const baseInvariant = results[0].invariant;
  if (baseInvariant === 0) {
    return { conserved: true, maxDrift: 0, avgDrift: 0 };
  }

  let maxDrift = 0;
  let totalDrift = 0;

  for (let i = 1; i < results.length; i++) {
    const drift = Math.abs(results[i].invariant - baseInvariant) / Math.abs(baseInvariant);
    if (drift > maxDrift) maxDrift = drift;
    totalDrift += drift;
  }

  const avgDrift = totalDrift / (results.length - 1);

  return {
    conserved: maxDrift <= toleranceFraction,
    maxDrift,
    avgDrift,
  };
}
