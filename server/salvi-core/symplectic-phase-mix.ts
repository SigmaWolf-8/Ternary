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
 * # Symplectic Phase Mixing for Phase Encryption
 *
 * Adds a structure-preserving mixing step to the phase encryption pipeline.
 * Symplectic maps preserve phase-space volume (Liouville's theorem), which
 * maps to checksum invariant conservation in our context.
 *
 * ## Theory
 *
 * For ternary-valued state arrays (values in {0, 1, 2}), the natural
 * conserved quantity under GF(3) operations is the **ternary parity**:
 *
 *   P(state) = Σ(state_i) mod 3
 *
 * The mixing uses XOR-like operations in GF(3) that preserve this parity.
 * Additionally, a mod-13 checksum is computed for SUFT alignment but is
 * not claimed to be preserved by the mixing itself — the ternary parity
 * (mod 3) is the true symplectic invariant.
 *
 * ## Integration
 *
 * The `symplecticGuardianChecksum` function can be used alongside or
 * in place of the existing Tribonacci hash in the guardian phase of
 * phase encryption. It applies symplectic mixing before folding into
 * a 64-bit checksum.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { SUFT_RADIUS, SUFT_LUNAR_HARMONIC, MAGIC_CONSTANT } from '@shared/saturnian-blueprint';
import { TRIBONACCI_SEQUENCE } from '@shared/tribonacci-constants';

/**
 * Applies a symplectic mixing step to a ternary phase state array.
 *
 * The map preserves the ternary parity invariant: Σ(state_i) mod 3 = constant.
 * Each round applies pairwise transfers: for each adjacent pair (i, j),
 * a delta is transferred from j to i (add delta to i, subtract delta from j).
 * This guarantees the total mod-3 sum is conserved because each transfer
 * is zero-sum in GF(3).
 *
 * @param phaseState Array of integer values (will be reduced mod 3)
 * @param rounds     Number of mixing rounds (default: T(7) = 13)
 * @returns Mixed phase state with ternary parity preserved
 */
export function symplecticPhaseMix(
  phaseState: number[],
  rounds: number = SUFT_RADIUS
): number[] {
  if (phaseState.length === 0) return [];

  const n = phaseState.length;
  const state = phaseState.map(v => ((v % 3) + 3) % 3);

  for (let r = 0; r < rounds; r++) {
    // Forward pass: pairwise zero-sum transfers on even-indexed pairs
    for (let i = 0; i + 1 < n; i += 2) {
      const j = i + 1;
      const delta = (state[i] * (r + 1)) % 3;
      state[i] = (state[i] + delta) % 3;
      state[j] = ((state[j] - delta) % 3 + 3) % 3;
    }

    // Backward pass: pairwise zero-sum transfers on odd-indexed pairs
    for (let i = 1; i + 1 < n; i += 2) {
      const j = i + 1;
      const delta = (state[j] * (r + 1)) % 3;
      state[i] = (state[i] + delta) % 3;
      state[j] = ((state[j] - delta) % 3 + 3) % 3;
    }
  }

  return state;
}

/**
 * Computes the ternary parity invariant: Σ(state_i) mod 3.
 * This is the quantity preserved by symplecticPhaseMix.
 *
 * @param state Phase state array
 * @returns Parity value in {0, 1, 2}
 */
export function computeTernaryParity(state: number[]): number {
  let sum = 0;
  for (const val of state) {
    sum += ((val % 3) + 3) % 3;
  }
  return sum % 3;
}

/**
 * Computes a mod-13 phase checksum (SUFT-aligned).
 * Note: This is NOT preserved by symplectic mixing — use computeTernaryParity
 * for the conserved invariant. This is a supplementary diagnostic.
 *
 * @param state Phase state array
 * @returns Checksum in [0, 12]
 */
export function computePhaseChecksum(state: number[]): number {
  let sum = 0;
  for (const val of state) {
    sum += ((val % SUFT_RADIUS) + SUFT_RADIUS) % SUFT_RADIUS;
  }
  return sum % SUFT_RADIUS;
}

/**
 * Computes a symplectic-enhanced checksum for guardian phase tamper detection.
 *
 * Extends the existing Tribonacci hash by:
 * 1. Converting input to ternary-valued phase state
 * 2. Applying symplectic mixing (structure-preserving)
 * 3. Folding the mixed state into a 64-bit checksum
 *
 * @param data Input string to checksum
 * @returns 16-character hex checksum
 */
export function symplecticGuardianChecksum(data: string): string {
  const phaseState: number[] = [];
  for (let i = 0; i < data.length; i++) {
    phaseState.push(data.charCodeAt(i) % 3);
  }

  const mixed = symplecticPhaseMix(phaseState);

  let h0 = MAGIC_CONSTANT >>> 0; // 333 seed
  let h1 = (SUFT_LUNAR_HARMONIC * SUFT_RADIUS) >>> 0; // 364 seed

  const MIX = TRIBONACCI_SEQUENCE[7] * TRIBONACCI_SEQUENCE[8]; // 312

  for (let i = 0; i < mixed.length; i++) {
    const v = mixed[i];
    h0 = Math.imul(h0 ^ (v + i), MIX) >>> 0;
    h0 = ((h0 << 13) | (h0 >>> 19)) >>> 0;
    h1 = Math.imul(h1 ^ (v * 3 + i), MIX + 1) >>> 0;
    h1 = ((h1 << 7) | (h1 >>> 25)) >>> 0;
    h0 = (h0 ^ h1) >>> 0;
  }

  for (let r = 0; r < SUFT_RADIUS; r++) {
    h0 = Math.imul(h0 ^ (h0 >>> 16), MIX) >>> 0;
    h1 = Math.imul(h1 ^ (h1 >>> 16), MIX + 1) >>> 0;
    h0 = (h0 ^ h1) >>> 0;
    h1 = (h1 ^ h0) >>> 0;
  }

  return h0.toString(16).padStart(8, '0') + h1.toString(16).padStart(8, '0');
}

/**
 * Verifies that symplectic mixing preserved the ternary parity invariant.
 *
 * @param original  Original phase state
 * @param mixed     Mixed phase state (output of symplecticPhaseMix)
 * @returns Whether ternary parity (mod 3 sum) is preserved
 */
export function verifySymplecticParity(original: number[], mixed: number[]): boolean {
  return computeTernaryParity(original) === computeTernaryParity(mixed);
}
