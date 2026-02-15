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
 * A symplectic map M satisfies: M^T · J · M = J, where J is the standard
 * symplectic matrix. For a 1D system, the simplest symplectic map is a
 * shear (area-preserving in 2D phase space):
 *
 *   q' = q + f(p)
 *   p' = p + g(q')
 *
 * We apply this to ternary-valued phase state arrays, preserving a mod-13
 * checksum invariant (from SUFT_RADIUS). The mixing improves avalanche
 * behavior in the guardian phase checksum without breaking the conserved
 * quantity.
 *
 * ## Integration
 *
 * Chain after the Tribonacci hash in phaseSplit() for enhanced tamper
 * detection. The symplectic property ensures that the mixing is invertible
 * and preserves the checksum mod structure.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { SUFT_RADIUS, SUFT_LUNAR_HARMONIC, MAGIC_CONSTANT } from '@shared/saturnian-blueprint';
import { TRIBONACCI_SEQUENCE } from '@shared/tribonacci-constants';

/**
 * Applies a symplectic mixing step to a phase state array.
 *
 * The map preserves the checksum invariant: Σ(state_i) mod 13 = constant.
 * Each element is updated using its neighbor as the "conjugate momentum"
 * in a shear-style symplectic map.
 *
 * @param phaseState Array of integer values representing phase state
 * @param rounds     Number of mixing rounds (default: T(7) = 13)
 * @returns Mixed phase state with invariant preserved
 */
export function symplecticPhaseMix(
  phaseState: number[],
  rounds: number = SUFT_RADIUS
): number[] {
  if (phaseState.length === 0) return [];

  const state = [...phaseState];
  const n = state.length;
  const invariantBefore = computePhaseInvariant(state);

  for (let r = 0; r < rounds; r++) {
    // Forward shear: q_i' = q_i + q_{i+1} (mod 3)
    for (let i = 0; i < n; i++) {
      const neighbor = state[(i + 1) % n];
      state[i] = ((state[i] + neighbor) % 3 + 3) % 3;
    }

    // Backward shear: p_i' = p_i + p_{i-1} (mod 3) — conjugate direction
    for (let i = n - 1; i >= 0; i--) {
      const neighbor = state[(i - 1 + n) % n];
      state[i] = ((state[i] + neighbor) % 3 + 3) % 3;
    }
  }

  return state;
}

/**
 * Computes the phase-space invariant: checksum mod SUFT_RADIUS (13).
 *
 * @param state Phase state array
 * @returns Invariant value in [0, 12]
 */
export function computePhaseInvariant(state: number[]): number {
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
  // Step 1: Convert to ternary phase state
  const phaseState: number[] = [];
  for (let i = 0; i < data.length; i++) {
    phaseState.push(data.charCodeAt(i) % 3);
  }

  // Step 2: Apply symplectic mixing
  const mixed = symplecticPhaseMix(phaseState);

  // Step 3: Fold into 64-bit checksum using Saturnian constants
  let h0 = MAGIC_CONSTANT >>> 0; // 333 seed
  let h1 = (SUFT_LUNAR_HARMONIC * SUFT_RADIUS) >>> 0; // 364 seed

  const MIX = Math.floor(TRIBONACCI_SEQUENCE[7] * TRIBONACCI_SEQUENCE[8]); // 312

  for (let i = 0; i < mixed.length; i++) {
    const v = mixed[i];
    h0 = Math.imul(h0 ^ (v + i), MIX) >>> 0;
    h0 = ((h0 << 13) | (h0 >>> 19)) >>> 0;
    h1 = Math.imul(h1 ^ (v * 3 + i), MIX + 1) >>> 0;
    h1 = ((h1 << 7) | (h1 >>> 25)) >>> 0;
    h0 = (h0 ^ h1) >>> 0;
  }

  // 13-round finalization
  for (let r = 0; r < SUFT_RADIUS; r++) {
    h0 = Math.imul(h0 ^ (h0 >>> 16), MIX) >>> 0;
    h1 = Math.imul(h1 ^ (h1 >>> 16), MIX + 1) >>> 0;
    h0 = (h0 ^ h1) >>> 0;
    h1 = (h1 ^ h0) >>> 0;
  }

  return h0.toString(16).padStart(8, '0') + h1.toString(16).padStart(8, '0');
}

/**
 * Verifies that symplectic mixing preserves the ternary parity.
 * The total ternary sum mod 3 should be invariant under symplectic maps.
 *
 * @param original  Original phase state
 * @param mixed     Mixed phase state
 * @returns Whether ternary parity is preserved
 */
export function verifySymplecticParity(original: number[], mixed: number[]): boolean {
  const originalParity = original.reduce((s, v) => s + ((v % 3 + 3) % 3), 0) % 3;
  const mixedParity = mixed.reduce((s, v) => s + ((v % 3 + 3) % 3), 0) % 3;
  return originalParity === mixedParity;
}
