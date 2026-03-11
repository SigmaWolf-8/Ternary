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
 * # Noether Symmetries for the Ternary VM
 *
 * Functional implementation of conserved quantities derived from
 * Noether's theorem applied to the SUFT Lagrangian's three symmetries:
 *
 * 1. **Ternary Gauge Symmetry**
 *    Invariance under t_α → t_α + θ_α with Σθ_α = 0.
 *    Conserved charge: Q = Σ π_{t_α} θ_α = 0 (SU(3)-like on branches).
 *
 * 2. **Reparametrization Symmetry**
 *    Invariance under τ → f(τ).
 *    Conserved quantity: H ≈ λΦ = 0 (on-shell "energy").
 *
 * 3. **Periodicity Symmetry**
 *    Discrete shift t_α → t_α + 364.
 *    Conserved: mod-364 equivalence (discrete Noether for CTCs).
 *
 * All functions are pure, exact (rational coefficients from SUFT constants),
 * and ternary-safe (mod-3). Designed for VM state validation hooks and
 * phase encryption invariant checks.
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  SUFT_RADIUS,
  SUFT_LUNAR_HARMONIC,
  SUFT_COSMIC_CIRCUMFERENCE,
  MASS_SHELL_RATIO,
} from './plenum-square';

import type { Trit } from './lagrangian-ternary-utils';

type TritTriple = [Trit, Trit, Trit];

/**
 * Checks the ternary gauge symmetry invariant: Σ θ_α = 0.
 *
 * For the SUFT Lagrangian, invariance under t_α → t_α + θ_α
 * requires that the branch shifts sum to zero.
 *
 * @param theta Branch shift trits [θ₋₁, θ₀, θ₊₁]
 * @returns True if gauge constraint satisfied (sum = 0)
 */
export function checkTernaryGaugeInvariant(theta: TritTriple): boolean {
  return theta[0] + theta[1] + theta[2] === 0;
}

/**
 * Applies a ternary gauge transformation to branch states.
 *
 * Transforms t_α → t_α + θ_α with mod-3 clamping to balanced ternary.
 * Throws if θ violates the gauge constraint (Σθ_α ≠ 0).
 *
 * @param state Current branch states [t₋₁, t₀, t₊₁]
 * @param theta Branch shifts (must satisfy Σθ_α = 0)
 * @returns Transformed states in balanced ternary
 * @throws Error if gauge constraint violated
 */
export function applyTernaryGaugeTransform(
  state: TritTriple,
  theta: TritTriple
): TritTriple {
  if (!checkTernaryGaugeInvariant(theta)) {
    throw new Error('Ternary gauge violation: shifts must sum to 0');
  }
  return state.map((s, i) => {
    const raw = s + theta[i];
    return (((raw % 3) + 4) % 3 - 1) as Trit;
  }) as TritTriple;
}

/**
 * Enumerates all valid gauge shifts for balanced ternary branches.
 *
 * Returns every [θ₋₁, θ₀, θ₊₁] ∈ {-1,0,+1}³ satisfying Σθ_α = 0.
 * There are exactly 7 valid shifts (including the trivial [0,0,0]).
 *
 * @returns Array of valid gauge shift triples
 */
export function enumerateGaugeShifts(): TritTriple[] {
  const trits: Trit[] = [-1, 0, 1];
  const valid: TritTriple[] = [];
  for (const a of trits) {
    for (const b of trits) {
      for (const c of trits) {
        if (a + b + c === 0) {
          valid.push([a, b, c]);
        }
      }
    }
  }
  return valid;
}

/**
 * Checks the reparametrization invariant (discrete version).
 *
 * On-shell, the Hamiltonian vanishes: H = λΦ = 0.
 * The mass-shell constraint is Φ = g^μν p_μ p_ν + (13/28) Σ E_α² + m² = 0.
 *
 * For discrete ternary states, we check λ·Φ = 0 using exact rational 13/28.
 *
 * @param lambda Constraint multiplier trit
 * @param p_mu Spatial momentum components (assumed flat metric g=1)
 * @param E_trits Branch energy trits [E₋₁, E₀, E₊₁]
 * @param m2 Squared mass parameter
 * @returns True if reparametrization invariant holds
 */
export function checkReparamInvariant(
  lambda: Trit,
  p_mu: number[],
  E_trits: TritTriple,
  m2: number
): boolean {
  const sumP2 = p_mu.reduce((sum, p) => sum + p * p, 0);
  const sumE2 = E_trits.reduce((sum: number, E: number) => sum + E * E, 0);
  const phi = sumP2 + MASS_SHELL_RATIO * sumE2 + m2;
  return Math.abs(lambda * phi) < 1e-12;
}

/**
 * Checks the periodicity symmetry invariant.
 *
 * Under the discrete shift t_α → t_α + 364, the SUFT Lagrangian's
 * periodicity constraint cos(2π t_α / 364) − 1 = 0 is preserved.
 *
 * @param t_alpha Branch coordinate value
 * @returns True if periodicity invariant holds (t_alpha ≡ 0 mod 364)
 */
export function checkPeriodicityInvariant(t_alpha: number): boolean {
  const residual = Math.cos(2 * Math.PI * t_alpha / SUFT_COSMIC_CIRCUMFERENCE) - 1;
  return Math.abs(residual) < 1e-12;
}

/**
 * Applies a periodicity shift (discrete Noether transformation for CTCs).
 *
 * Only multiples of 364 preserve the periodicity invariant.
 * Throws if the shift is not a valid multiple.
 *
 * @param t_alpha Current temporal coordinate
 * @param shift Shift amount (must be a multiple of 364)
 * @returns Shifted coordinate
 * @throws Error if shift is not a multiple of 364
 */
export function applyPeriodicityShift(t_alpha: number, shift: number): number {
  if (shift % SUFT_COSMIC_CIRCUMFERENCE !== 0) {
    throw new Error('Periodicity violation: shift must be multiple of 364');
  }
  return t_alpha + shift;
}

/**
 * Reduces a temporal coordinate to its canonical representative mod 364.
 *
 * Maps any integer t_alpha into [0, 363].
 *
 * @param t_alpha Temporal coordinate
 * @returns Canonical representative in [0, 363]
 */
export function canonicalPeriodicity(t_alpha: number): number {
  return ((t_alpha % SUFT_COSMIC_CIRCUMFERENCE) + SUFT_COSMIC_CIRCUMFERENCE) % SUFT_COSMIC_CIRCUMFERENCE;
}

/**
 * Computes the conserved Noether charge for ternary gauge symmetry.
 *
 * Q = Σ π_{t_α} · θ_α where Σθ_α = 0.
 * For the trivial shift θ = [1, -1, 0] (canonical generator),
 * Q = π₋₁ − π₀.
 *
 * In the mod-3 ternary domain, the charge is computed as (Σ π_α) mod 3.
 *
 * @param momenta Canonical momenta [π₋₁, π₀, π₊₁]
 * @returns Conserved charge mod 3 (0 if symmetry holds perfectly)
 */
export function noetherGaugeCharge(momenta: TritTriple): number {
  const sum = momenta[0] + momenta[1] + momenta[2];
  return ((sum % 3) + 3) % 3;
}

/**
 * Checks whether a gauge transformation preserves the Noether charge.
 *
 * Applies the gauge shift θ to state, recomputes momenta via simple
 * identity (since T(7)=13 ≡ 1 mod 3), and verifies charge conservation.
 *
 * @param momenta Current canonical momenta
 * @param theta Gauge shift (must satisfy Σθ_α = 0)
 * @returns True if charge is conserved under the transformation
 */
export function verifyGaugeChargeConservation(
  momenta: TritTriple,
  theta: TritTriple
): boolean {
  if (!checkTernaryGaugeInvariant(theta)) return false;
  const chargeBefore = noetherGaugeCharge(momenta);
  const shifted: TritTriple = momenta.map((p, i) => {
    const raw = p + theta[i];
    return (((raw % 3) + 4) % 3 - 1) as Trit;
  }) as TritTriple;
  const chargeAfter = noetherGaugeCharge(shifted);
  return chargeBefore === chargeAfter;
}

/**
 * Validates all three Noether invariants simultaneously on a VM state.
 *
 * The gauge charge is computed from canonical momenta if provided,
 * otherwise falls back to E_trits (which equal momenta when cross-coupling
 * vanishes, i.e., t_α = 0).
 *
 * @param state VM state snapshot
 * @returns Diagnostic with pass/fail for each invariant
 */
export function validateNoetherInvariants(state: {
  lambda: Trit;
  p_mu: number[];
  E_trits: TritTriple;
  canonicalMomenta?: TritTriple;
  t_epoch: number;
  m2: number;
}): {
  gaugeHolds: boolean;
  reparamHolds: boolean;
  periodicityHolds: boolean;
  allHold: boolean;
} {
  const momenta = state.canonicalMomenta ?? state.E_trits;
  const gaugeHolds = noetherGaugeCharge(momenta) === 0;
  const reparamHolds = checkReparamInvariant(state.lambda, state.p_mu, state.E_trits, state.m2);
  const periodicityHolds = checkPeriodicityInvariant(state.t_epoch);

  return {
    gaugeHolds,
    reparamHolds,
    periodicityHolds,
    allHold: gaugeHolds && reparamHolds && periodicityHolds,
  };
}
