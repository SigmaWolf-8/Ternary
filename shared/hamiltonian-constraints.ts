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
 * # Hamiltonian Constraint System for Ternary VM
 *
 * Enforces conservation laws on the ternary virtual machine's register state,
 * inspired by Hamiltonian mechanics. In a Hamiltonian system, the constraint
 * surface Φ ≈ 0 (mass shell) must be preserved through all state transitions.
 *
 * ## Mapping to Ternary VM
 *
 * The VM has 27 registers (3³) grouped into 3 banks of 9. We define a
 * "Hamiltonian" invariant over the register state:
 *
 *   H(regs) = Σ(reg_i²) mod (T(7) × T(8))   [mod 312]
 *
 * The constraint Φ checks whether H is preserved (within tolerance) across
 * opcode execution boundaries. Violations indicate invalid state transitions
 * — useful for formal verification and FIPS/CMVP compliance evidence.
 *
 * ## SUFT Alignment
 *
 * - Register grouping: 27 = 3³ (ternary cubic)
 * - Constraint modulus: T(7) × T(8) = 13 × 24 = 312
 * - Mass-shell ratio: 13/28 (SUFT coefficient from Saturnian blueprint)
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { TRIBONACCI_SEQUENCE } from './tribonacci-constants';
import { SUFT_RADIUS, SUFT_LUNAR_HARMONIC, MASS_SHELL_RATIO } from './saturnian-blueprint';

const T7 = TRIBONACCI_SEQUENCE[7];  // 13
const T8 = TRIBONACCI_SEQUENCE[8];  // 24
const CONSTRAINT_MODULUS = T7 * T8; // 312

export const VM_REGISTER_COUNT = 27;      // 3³
export const VM_REGISTER_BANKS = 3;
export const VM_REGISTERS_PER_BANK = 9;   // 3²

export interface HamiltonianState {
  energy: number;
  bankEnergies: [number, number, number];
  constraintValue: number;
}

export interface ConstraintCheckResult {
  valid: boolean;
  currentEnergy: number;
  previousEnergy: number;
  drift: number;
  tolerance: number;
  constraintSurface: number;
}

/**
 * Computes the Hamiltonian "energy" of a 27-register state.
 *
 * H = Σ(reg_i²) mod 312
 *
 * @param registers Array of 27 register values
 * @returns Hamiltonian state with total and per-bank energies
 */
export function computeHamiltonianState(registers: number[]): HamiltonianState {
  if (registers.length !== VM_REGISTER_COUNT) {
    throw new Error(`Expected ${VM_REGISTER_COUNT} registers, got ${registers.length}`);
  }

  let totalEnergy = 0;
  const bankEnergies: [number, number, number] = [0, 0, 0];

  for (let i = 0; i < VM_REGISTER_COUNT; i++) {
    const contribution = (registers[i] * registers[i]) >>> 0;
    totalEnergy += contribution;
    bankEnergies[Math.floor(i / VM_REGISTERS_PER_BANK)] += contribution;
  }

  const energy = ((totalEnergy % CONSTRAINT_MODULUS) + CONSTRAINT_MODULUS) % CONSTRAINT_MODULUS;

  const constraintValue = energy * MASS_SHELL_RATIO;

  return { energy, bankEnergies, constraintValue };
}

/**
 * Checks the Hamiltonian constraint between two register states.
 * Returns whether the energy invariant is preserved within tolerance.
 *
 * @param prevRegisters  Register state before opcode execution
 * @param currRegisters  Register state after opcode execution
 * @param tolerance      Maximum allowed energy drift (default: T(7) = 13)
 * @returns Constraint check result
 */
export function checkHamiltonianConstraint(
  prevRegisters: number[],
  currRegisters: number[],
  tolerance: number = T7
): ConstraintCheckResult {
  const prevState = computeHamiltonianState(prevRegisters);
  const currState = computeHamiltonianState(currRegisters);

  const drift = Math.abs(currState.energy - prevState.energy);

  const constraintSurface = currState.constraintValue;

  return {
    valid: drift <= tolerance,
    currentEnergy: currState.energy,
    previousEnergy: prevState.energy,
    drift,
    tolerance,
    constraintSurface,
  };
}

/**
 * Validates a sequence of register snapshots for constraint preservation.
 * Useful for verifying opcode chains maintain invariants.
 *
 * @param snapshots Array of 27-element register arrays
 * @param tolerance Maximum allowed per-step energy drift
 * @returns Summary of constraint violations across the sequence
 */
export function validateOpcodeSequence(
  snapshots: number[][],
  tolerance: number = T7
): {
  valid: boolean;
  totalSteps: number;
  violations: number;
  maxDrift: number;
  violationIndices: number[];
} {
  if (snapshots.length < 2) {
    return { valid: true, totalSteps: 0, violations: 0, maxDrift: 0, violationIndices: [] };
  }

  let violations = 0;
  let maxDrift = 0;
  const violationIndices: number[] = [];

  for (let i = 1; i < snapshots.length; i++) {
    const result = checkHamiltonianConstraint(snapshots[i - 1], snapshots[i], tolerance);
    if (!result.valid) {
      violations++;
      violationIndices.push(i);
    }
    if (result.drift > maxDrift) {
      maxDrift = result.drift;
    }
  }

  return {
    valid: violations === 0,
    totalSteps: snapshots.length - 1,
    violations,
    maxDrift,
    violationIndices,
  };
}

/**
 * Computes the ternary parity invariant for a register bank.
 * Each register value is reduced mod 3, and the bank parity is the
 * sum mod 3 — a GF(3) conservation check.
 *
 * @param registers Full 27-register array
 * @param bank Bank index (0, 1, or 2)
 * @returns Ternary parity (0, 1, or 2)
 */
export function computeBankTernaryParity(registers: number[], bank: 0 | 1 | 2): number {
  if (registers.length !== VM_REGISTER_COUNT) {
    throw new Error(`Expected ${VM_REGISTER_COUNT} registers, got ${registers.length}`);
  }

  const start = bank * VM_REGISTERS_PER_BANK;
  let sum = 0;
  for (let i = start; i < start + VM_REGISTERS_PER_BANK; i++) {
    sum += ((registers[i] % 3) + 3) % 3;
  }
  return sum % 3;
}
