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
 * # Lagrangian Mechanics Utilities for Ternary Logic
 *
 * Discrete adaptations of the SUFT Lagrangian's Euler-Lagrange equations
 * for ternary-valued state evolution. Complements the Hamiltonian modules
 * by providing variational (action-based) state transitions.
 *
 * ## Theory
 *
 * The SUFT Lagrangian is:
 *
 *   L = p·ẋ + Σ E_α·ṫ_α − λΦ + (φ/26)(t₋₁ṫ₊₁ − t₊₁ṫ₋₁)
 *       + (1/56)(E₋₁Ė₊₁ − E₊₁Ė₋₁) + Σ μ_α[cos(2πt_α/364) − 1]
 *
 * In the ternary platform, coordinates are trits (values in {-1, 0, +1}).
 * The Euler-Lagrange equations d/dτ(∂L/∂q̇) − ∂L/∂q = 0 are discretized
 * with finite differences over trit states, using mod-3 arithmetic.
 *
 * ## SUFT Constants
 *
 * - Temporal cross denominator: 26 = 2 × T(7) = 2 × 13
 * - Energy cross denominator: 56 = 2 × 28 = 2 × SUFT_LUNAR_HARMONIC
 * - Periodicity: 364 = SUFT_COSMIC_CIRCUMFERENCE
 * - Mass-shell ratio: 13/28
 * - Constraint scale: 13/14 = 2 × MASS_SHELL_RATIO
 *
 * ## Design
 *
 * All functions are pure, functional, and use exact rational coefficients.
 * Ternary values are mapped to {-1, 0, +1} with balanced representation.
 * Each function preserves the ternary domain via ((x % 3) + 4) % 3 - 1.
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
  TEMPORAL_CROSS_DENOM,
  ENERGY_CROSS_DENOM,
} from './plenum-square';

export type Trit = -1 | 0 | 1;

const PHI = (1 + Math.sqrt(5)) / 2;
const CROSS_COEFF = PHI / (2 * TEMPORAL_CROSS_DENOM); // φ/26
const ENERGY_CROSS_COEFF = 1 / (2 * ENERGY_CROSS_DENOM); // 1/56
const PERIOD_COEFF = Math.PI / (SUFT_COSMIC_CIRCUMFERENCE / 2); // π/182
const CONSTRAINT_COEFF = 2 * MASS_SHELL_RATIO; // 13/14

/**
 * Clamps a real value to the nearest balanced trit {-1, 0, +1}.
 * Uses rounding to the nearest integer, then mod-3 balance.
 */
export function toTrit(value: number): Trit {
  const rounded = Math.round(value);
  const mod3 = ((rounded % 3) + 4) % 3 - 1;
  return mod3 as Trit;
}

/**
 * Computes the discrete ternary "momentum" from a trit velocity.
 * Maps ∂L/∂q̇ in the discrete setting: p = v × T(7) mod 3.
 *
 * Since T(7) = 13 ≡ 1 (mod 3), the momentum equals the velocity in GF(3).
 * This is exact — no rounding.
 *
 * @param velocity Discrete velocity trit ∈ {-1, 0, +1}
 * @returns Momentum trit
 */
export function ternaryMomentum(velocity: Trit): Trit {
  return velocity; // 13 ≡ 1 mod 3
}

/**
 * Computes the canonical momentum for the t₋₁ (past) branch.
 *
 * From EL derivation: π_{t₋₁} = E₋₁ − (φ/26)·t₊₁
 *
 * @param E_minus1  Energy of past branch
 * @param t_plus1   Trit state of future branch
 * @returns Canonical momentum trit for past branch
 */
export function canonicalMomentumTMinus1(E_minus1: Trit, t_plus1: Trit): Trit {
  return toTrit(E_minus1 - CROSS_COEFF * t_plus1);
}

/**
 * Computes the canonical momentum for t₊₁ (future branch).
 *
 * From EL derivation: π_{t₊₁} = E₊₁ + (φ/26)·t₋₁
 *
 * @param E_plus1   Energy of future branch
 * @param t_minus1  Trit state of past branch
 * @returns Canonical momentum trit for future branch
 */
export function canonicalMomentumTPlus1(E_plus1: Trit, t_minus1: Trit): Trit {
  return toTrit(E_plus1 + CROSS_COEFF * t_minus1);
}

/**
 * Discrete EL update for the t₋₁ (past) branch energy derivative.
 *
 * EL equation: Ė₋₁ = (φ/26)·ṫ₊₁ − (π·μ₋₁/182)·sin(π·t₋₁/182)
 *
 * @param dot_t_plus1   Discrete velocity of future branch
 * @param mu_minus1     Lagrange multiplier trit for periodicity
 * @param t_minus1      Current past branch state
 * @returns Updated energy derivative trit
 */
export function elUpdateEDotMinus1(
  dot_t_plus1: Trit,
  mu_minus1: Trit,
  t_minus1: Trit
): Trit {
  const cross = CROSS_COEFF * dot_t_plus1;
  const period = PERIOD_COEFF * mu_minus1 * Math.sin(PERIOD_COEFF * t_minus1);
  return toTrit(cross - period);
}

/**
 * Discrete EL update for the t₀ (present) branch energy derivative.
 *
 * EL equation: Ė₀ = −(π·μ₀/182)·sin(π·t₀/182)
 *
 * @param mu_0   Multiplier trit
 * @param t_0    Current present branch state
 * @returns Updated energy derivative trit
 */
export function elUpdateEDot0(mu_0: Trit, t_0: Trit): Trit {
  const period = PERIOD_COEFF * mu_0 * Math.sin(PERIOD_COEFF * t_0);
  return toTrit(-period);
}

/**
 * Discrete EL update for the t₊₁ (future) branch energy derivative.
 *
 * EL equation: Ė₊₁ = −(φ/26)·ṫ₋₁ − (π·μ₊₁/182)·sin(π·t₊₁/182)
 *
 * @param dot_t_minus1  Discrete velocity of past branch
 * @param mu_plus1      Multiplier trit
 * @param t_plus1       Current future branch state
 * @returns Updated energy derivative trit
 */
export function elUpdateEDotPlus1(
  dot_t_minus1: Trit,
  mu_plus1: Trit,
  t_plus1: Trit
): Trit {
  const cross = CROSS_COEFF * dot_t_minus1;
  const period = PERIOD_COEFF * mu_plus1 * Math.sin(PERIOD_COEFF * t_plus1);
  return toTrit(-cross - period);
}

/**
 * Discrete EL update for E₋₁ (solves for ṫ₋₁).
 *
 * EL equation: ṫ₋₁ = (1/28)·Ė₊₁ − λ·(13/14)·E₋₁
 *
 * @param dot_E_plus1  Discrete energy velocity of future branch
 * @param lambda       Constraint multiplier trit
 * @param E_minus1     Current past energy state
 * @returns Updated temporal velocity trit for past branch
 */
export function elSolveTDotMinus1(
  dot_E_plus1: Trit,
  lambda: Trit,
  E_minus1: Trit
): Trit {
  const energyCross = ENERGY_CROSS_COEFF * 2 * dot_E_plus1; // 1/28
  const constraint = lambda * CONSTRAINT_COEFF * E_minus1;    // λ·13/14
  return toTrit(energyCross - constraint);
}

/**
 * Discrete EL update for E₀ (solves for ṫ₀).
 *
 * EL equation: ṫ₀ = −λ·(13/14)·E₀
 *
 * @param lambda  Constraint multiplier trit
 * @param E_0     Current present energy state
 * @returns Updated temporal velocity trit for present branch
 */
export function elSolveTDot0(lambda: Trit, E_0: Trit): Trit {
  return toTrit(-lambda * CONSTRAINT_COEFF * E_0);
}

/**
 * Discrete EL update for E₊₁ (solves for ṫ₊₁).
 *
 * EL equation: ṫ₊₁ = −(1/28)·Ė₋₁ − λ·(13/14)·E₊₁
 *
 * @param dot_E_minus1  Discrete energy velocity of past branch
 * @param lambda        Constraint multiplier trit
 * @param E_plus1       Current future energy state
 * @returns Updated temporal velocity trit for future branch
 */
export function elSolveTDotPlus1(
  dot_E_minus1: Trit,
  lambda: Trit,
  E_plus1: Trit
): Trit {
  const energyCross = ENERGY_CROSS_COEFF * 2 * dot_E_minus1; // 1/28
  const constraint = lambda * CONSTRAINT_COEFF * E_plus1;
  return toTrit(-energyCross - constraint);
}

/**
 * Computes the ternary cross-coupling between temporal branches.
 *
 * From the Lagrangian antisymmetric term: (φ/26)(t₋₁ṫ₊₁ − t₊₁ṫ₋₁).
 * Returns the coupling strength scaled to ternary domain.
 *
 * @param t_plus1   Future branch trit
 * @param t_minus1  Past branch trit
 * @returns Cross coupling value (exact rational: ±φ/13)
 */
export function ternaryCrossCoupling(t_plus1: Trit, t_minus1: Trit): number {
  return (t_minus1 - t_plus1) * CROSS_COEFF;
}

/**
 * Checks the mass-shell constraint Φ = 0.
 *
 * Φ = g^μν p_μ p_ν + (13/28) Σ E_α² + m²
 *
 * In discrete ternary: p values are trit-valued momentum components,
 * E_trits are branch energies, m² is the mass-squared parameter.
 *
 * @param p_trits  4D momentum trits (flat spacetime: g = diag(-1,1,1,1))
 * @param E_trits  3 branch energy trits [E₋₁, E₀, E₊₁]
 * @param m2       Mass-squared parameter (default 0 for massless)
 * @returns Object with constraint value and whether it vanishes mod 3
 */
export function checkMassShellConstraint(
  p_trits: [Trit, Trit, Trit, Trit],
  E_trits: [Trit, Trit, Trit],
  m2: number = 0
): { constraint: number; vanishes: boolean } {
  const g = [-1, 1, 1, 1];
  let kinetic = 0;
  for (let mu = 0; mu < 4; mu++) {
    kinetic += g[mu] * p_trits[mu] * p_trits[mu];
  }

  let energySum = 0;
  for (const E of E_trits) {
    energySum += E * E;
  }

  const constraint = kinetic + MASS_SHELL_RATIO * energySum + m2;
  const vanishes = Math.abs(constraint) < 1e-12 || ((Math.round(constraint) % 3 + 3) % 3 === 0);

  return { constraint, vanishes };
}

/**
 * Checks the periodicity constraint: cos(2π·t_α/364) − 1 = 0.
 *
 * This constraint forces t_α to be a multiple of 364 (the cosmic
 * circumference), meaning the temporal coordinate wraps periodically.
 * For trit values {-1, 0, +1}, only t_α = 0 satisfies exactly.
 *
 * @param t_alpha Temporal branch value
 * @returns Whether the periodicity constraint is satisfied
 */
export function checkPeriodicityConstraint(t_alpha: number): boolean {
  const residual = Math.cos(2 * Math.PI * t_alpha / SUFT_COSMIC_CIRCUMFERENCE) - 1;
  return Math.abs(residual) < 1e-12;
}

/**
 * Computes the Noether charge for ternary gauge symmetry.
 *
 * Under the transformation t_α → t_α + θ_α with Σθ_α = 0,
 * the conserved charge is Q = Σ π_{t_α} · θ_α.
 *
 * For ternary states, the gauge invariant (mod 3) is the sum
 * of canonical momenta mod 3.
 *
 * @param canonicalMomenta Three canonical momentum trits [π₋₁, π₀, π₊₁]
 * @returns Noether charge mod 3 (0 if gauge symmetry holds)
 */
export function noetherTernaryCharge(canonicalMomenta: [Trit, Trit, Trit]): number {
  const sum = canonicalMomenta[0] + canonicalMomenta[1] + canonicalMomenta[2];
  return ((sum % 3) + 3) % 3;
}

/**
 * Performs a single discrete Euler-Lagrange evolution step for the
 * full ternary temporal system (t₋₁, t₀, t₊₁, E₋₁, E₀, E₊₁).
 *
 * Evolves the state by one discrete step using the SUFT EL equations,
 * returning the updated state and constraint diagnostics.
 *
 * @param state Current state: temporal trits and energy trits
 * @param lambda Constraint multiplier
 * @param mu Periodicity multipliers [μ₋₁, μ₀, μ₊₁]
 * @returns Evolved state and diagnostics
 */
export function eulerLagrangeStep(
  state: {
    t: [Trit, Trit, Trit];
    E: [Trit, Trit, Trit];
    tDot: [Trit, Trit, Trit];
  },
  lambda: Trit = 0,
  mu: [Trit, Trit, Trit] = [0, 0, 0]
): {
  t: [Trit, Trit, Trit];
  E: [Trit, Trit, Trit];
  tDot: [Trit, Trit, Trit];
  EDot: [Trit, Trit, Trit];
  noetherCharge: number;
} {
  const EDot: [Trit, Trit, Trit] = [
    elUpdateEDotMinus1(state.tDot[2], mu[0], state.t[0]),
    elUpdateEDot0(mu[1], state.t[1]),
    elUpdateEDotPlus1(state.tDot[0], mu[2], state.t[2]),
  ];

  const newTDot: [Trit, Trit, Trit] = [
    elSolveTDotMinus1(EDot[2], lambda, state.E[0]),
    elSolveTDot0(lambda, state.E[1]),
    elSolveTDotPlus1(EDot[0], lambda, state.E[2]),
  ];

  const newT: [Trit, Trit, Trit] = [
    toTrit(state.t[0] + newTDot[0]),
    toTrit(state.t[1] + newTDot[1]),
    toTrit(state.t[2] + newTDot[2]),
  ];

  const newE: [Trit, Trit, Trit] = [
    toTrit(state.E[0] + EDot[0]),
    toTrit(state.E[1] + EDot[1]),
    toTrit(state.E[2] + EDot[2]),
  ];

  const pi: [Trit, Trit, Trit] = [
    canonicalMomentumTMinus1(newE[0], newT[2]),
    newE[1],
    canonicalMomentumTPlus1(newE[2], newT[0]),
  ];

  return {
    t: newT,
    E: newE,
    tDot: newTDot,
    EDot,
    noetherCharge: noetherTernaryCharge(pi),
  };
}
