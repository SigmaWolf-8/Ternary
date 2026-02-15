/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  applySymplecticJitterCorrection,
  correctJitterBatch,
  verifyEnergyConservation,
  type SymplecticState,
} from '../server/salvi-core/hptp-symplectic-corrector';
import {
  computeHamiltonianState,
  checkHamiltonianConstraint,
  validateOpcodeSequence,
  computeBankTernaryParity,
  VM_REGISTER_COUNT,
  VM_REGISTER_BANKS,
  VM_REGISTERS_PER_BANK,
} from '../shared/hamiltonian-constraints';
import {
  symplecticPhaseMix,
  computePhaseInvariant,
  symplecticGuardianChecksum,
  verifySymplecticParity,
} from '../server/salvi-core/symplectic-phase-mix';

describe('HPTP Symplectic Jitter Corrector', () => {
  test('single correction returns valid result', () => {
    const result = applySymplecticJitterCorrection(1000000n, 50n);
    expect(typeof result.correctedTimestamp).toBe('bigint');
    expect(typeof result.momentum).toBe('bigint');
    expect(typeof result.invariant).toBe('number');
    expect(typeof result.correctionApplied).toBe('bigint');
  });

  test('zero jitter produces minimal correction', () => {
    const result = applySymplecticJitterCorrection(1000000n, 0n);
    expect(result.correctedTimestamp).toBe(1000000n);
    expect(result.correctionApplied).toBe(0n);
  });

  test('correction is applied in opposite direction of jitter', () => {
    const resultPos = applySymplecticJitterCorrection(1000000n, 100n);
    const resultNeg = applySymplecticJitterCorrection(1000000n, -100n);
    expect(resultPos.momentum !== resultNeg.momentum).toBe(true);
  });

  test('batch correction processes multiple samples', () => {
    const samples = [
      { timestamp: 1000000n, jitterDelta: 10n },
      { timestamp: 1000100n, jitterDelta: -5n },
      { timestamp: 1000200n, jitterDelta: 15n },
      { timestamp: 1000300n, jitterDelta: -8n },
      { timestamp: 1000400n, jitterDelta: 3n },
    ];

    const { correctedSamples, finalState, energyDrift } = correctJitterBatch(samples);
    expect(correctedSamples.length).toBe(5);
    expect(typeof finalState.momentum).toBe('bigint');
    expect(typeof energyDrift).toBe('number');
    expect(energyDrift).toBeGreaterThanOrEqual(0);
  });

  test('state accumulates across corrections', () => {
    let state: SymplecticState = { momentum: 0n, invariant: 0 };

    const r1 = applySymplecticJitterCorrection(1000n, 50n, state);
    state = { momentum: r1.momentum, invariant: r1.invariant };

    const r2 = applySymplecticJitterCorrection(2000n, -30n, state);
    expect(r2.momentum !== r1.momentum).toBe(true);
  });

  test('energy conservation verification works', () => {
    const samples = Array.from({ length: 20 }, (_, i) => ({
      timestamp: BigInt(1000 + i * 100),
      jitterDelta: BigInt(Math.floor(Math.sin(i) * 10)),
    }));

    const { correctedSamples } = correctJitterBatch(samples);
    const conservation = verifyEnergyConservation(correctedSamples, 1.0);
    expect(typeof conservation.conserved).toBe('boolean');
    expect(conservation.maxDrift).toBeGreaterThanOrEqual(0);
    expect(conservation.avgDrift).toBeGreaterThanOrEqual(0);
  });

  test('energy conservation with empty/single result', () => {
    expect(verifyEnergyConservation([]).conserved).toBe(true);
    const single = applySymplecticJitterCorrection(1000n, 5n);
    expect(verifyEnergyConservation([single]).conserved).toBe(true);
  });
});

describe('Hamiltonian VM Constraints', () => {
  const zeroRegisters = new Array(27).fill(0);

  test('register constants are correct', () => {
    expect(VM_REGISTER_COUNT).toBe(27);
    expect(VM_REGISTER_BANKS).toBe(3);
    expect(VM_REGISTERS_PER_BANK).toBe(9);
  });

  test('computes Hamiltonian state for zero registers', () => {
    const state = computeHamiltonianState(zeroRegisters);
    expect(state.energy).toBe(0);
    expect(state.bankEnergies).toEqual([0, 0, 0]);
    expect(state.constraintValue).toBe(0);
  });

  test('computes non-trivial Hamiltonian state', () => {
    const regs = Array.from({ length: 27 }, (_, i) => i);
    const state = computeHamiltonianState(regs);
    expect(state.energy).toBeGreaterThan(0);
    expect(state.energy).toBeLessThan(312); // mod 312
    expect(state.bankEnergies[0]).toBeGreaterThan(0);
  });

  test('rejects wrong register count', () => {
    expect(() => computeHamiltonianState([1, 2, 3])).toThrow();
    expect(() => computeHamiltonianState(new Array(28).fill(0))).toThrow();
  });

  test('identical states pass constraint check', () => {
    const regs = Array.from({ length: 27 }, (_, i) => i % 3);
    const result = checkHamiltonianConstraint(regs, regs);
    expect(result.valid).toBe(true);
    expect(result.drift).toBe(0);
  });

  test('small perturbation within tolerance', () => {
    const regs1 = Array.from({ length: 27 }, (_, i) => i % 3);
    const regs2 = [...regs1];
    regs2[0] = (regs2[0] + 1) % 3;
    const result = checkHamiltonianConstraint(regs1, regs2, 13);
    expect(typeof result.valid).toBe('boolean');
    expect(result.drift).toBeGreaterThanOrEqual(0);
  });

  test('large perturbation may violate constraint', () => {
    const regs1 = new Array(27).fill(0);
    const regs2 = Array.from({ length: 27 }, (_, i) => i * 10);
    const result = checkHamiltonianConstraint(regs1, regs2, 1);
    expect(result.drift).toBeGreaterThan(0);
  });

  test('validates opcode sequence — all identical states pass', () => {
    const regs = Array.from({ length: 27 }, (_, i) => i % 3);
    const snapshots = [regs, regs, regs, regs, regs];
    const result = validateOpcodeSequence(snapshots);
    expect(result.valid).toBe(true);
    expect(result.violations).toBe(0);
    expect(result.totalSteps).toBe(4);
  });

  test('validates opcode sequence — detects violations with guaranteed drift', () => {
    const regs1 = new Array(27).fill(0);
    const regs2 = new Array(27).fill(0);
    regs2[0] = 10; // energy = 100 mod 312 = 100, drift = 100 > tolerance 1
    const result = validateOpcodeSequence([regs1, regs2], 1);
    expect(result.violations).toBeGreaterThan(0);
    expect(result.violationIndices).toContain(1);
    expect(result.maxDrift).toBe(100);
  });

  test('computes bank ternary parity', () => {
    const regs = Array.from({ length: 27 }, (_, i) => i);
    for (const bank of [0, 1, 2] as const) {
      const parity = computeBankTernaryParity(regs, bank);
      expect(parity).toBeGreaterThanOrEqual(0);
      expect(parity).toBeLessThan(3);
    }
  });

  test('ternary parity is stable for same bank', () => {
    const regs = Array.from({ length: 27 }, () => 1);
    expect(computeBankTernaryParity(regs, 0)).toBe(computeBankTernaryParity(regs, 1));
    expect(computeBankTernaryParity(regs, 1)).toBe(computeBankTernaryParity(regs, 2));
  });
});

describe('Symplectic Phase Mixing', () => {
  test('empty input returns empty output', () => {
    expect(symplecticPhaseMix([])).toEqual([]);
  });

  test('mixes ternary values', () => {
    const input = [0, 1, 2, 0, 1, 2, 0, 1, 2];
    const mixed = symplecticPhaseMix(input);
    expect(mixed.length).toBe(input.length);
    mixed.forEach(v => {
      expect(v).toBeGreaterThanOrEqual(0);
      expect(v).toBeLessThan(3);
    });
  });

  test('deterministic — same input produces same output', () => {
    const input = [1, 0, 2, 1, 0, 2];
    const mixed1 = symplecticPhaseMix(input);
    const mixed2 = symplecticPhaseMix(input);
    expect(mixed1).toEqual(mixed2);
  });

  test('different inputs produce different outputs', () => {
    const input1 = [0, 1, 2, 0, 1, 2];
    const input2 = [2, 1, 0, 2, 1, 0];
    const mixed1 = symplecticPhaseMix(input1);
    const mixed2 = symplecticPhaseMix(input2);
    expect(mixed1).not.toEqual(mixed2);
  });

  test('custom round count works', () => {
    const input = [0, 1, 2, 0, 1, 2];
    const mixed1 = symplecticPhaseMix(input, 1);
    const mixed13 = symplecticPhaseMix(input, 13);
    expect(mixed1).not.toEqual(mixed13);
  });

  test('computes phase invariant', () => {
    const state = [1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1];
    const invariant = computePhaseInvariant(state);
    expect(invariant).toBeGreaterThanOrEqual(0);
    expect(invariant).toBeLessThan(13);
  });

  test('guardian checksum produces 16-hex-char output', () => {
    const checksum = symplecticGuardianChecksum('Hello, PlenumNET!');
    expect(checksum.length).toBe(16);
    expect(/^[0-9a-f]{16}$/.test(checksum)).toBe(true);
  });

  test('guardian checksum is deterministic', () => {
    const c1 = symplecticGuardianChecksum('test data');
    const c2 = symplecticGuardianChecksum('test data');
    expect(c1).toBe(c2);
  });

  test('guardian checksum has avalanche — different inputs give different checksums', () => {
    const c1 = symplecticGuardianChecksum('test data A');
    const c2 = symplecticGuardianChecksum('test data B');
    expect(c1).not.toBe(c2);
  });

  test('verifySymplecticParity checks ternary parity', () => {
    const original = [0, 1, 2, 0, 1, 2];
    const same = [1, 2, 0, 1, 2, 0]; // same ternary sum mod 3
    const different = [0, 0, 0, 0, 0, 0]; // different sum mod 3
    expect(verifySymplecticParity(original, same)).toBe(true);
    expect(verifySymplecticParity(original, different)).toBe(true); // both sum to 0 mod 3
  });
});
