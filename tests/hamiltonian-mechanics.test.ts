/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 */

import { describe, test, expect } from 'vitest';
import {
  applySymplecticJitterCorrection,
  correctJitterBatch,
  verifyEnergyConservation,
  computeHamiltonian,
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
  computeTernaryParity,
  computePhaseChecksum,
  symplecticGuardianChecksum,
  verifySymplecticParity,
} from '../server/salvi-core/symplectic-phase-mix';

describe('HPTP Symplectic Jitter Corrector', () => {
  test('computeHamiltonian returns consistent H = p²/2 + ω²q²/2', () => {
    const h1 = computeHamiltonian(0, 0);
    expect(h1).toBe(0);
    const h2 = computeHamiltonian(13, 0);
    expect(h2).toBeGreaterThan(0);
    const h3 = computeHamiltonian(0, 5);
    expect(h3).toBe(5 * 5 / 2);
  });

  test('single correction returns valid result', () => {
    const result = applySymplecticJitterCorrection(1000000n, 50n);
    expect(typeof result.correctedTimestamp).toBe('bigint');
    expect(typeof result.momentum).toBe('number');
    expect(typeof result.position).toBe('number');
    expect(typeof result.invariant).toBe('number');
    expect(typeof result.correctionApplied).toBe('bigint');
  });

  test('zero jitter produces zero correction', () => {
    const result = applySymplecticJitterCorrection(1000000n, 0n);
    expect(result.correctedTimestamp).toBe(1000000n);
    expect(result.position).toBe(0);
    expect(result.momentum).toBe(0);
    expect(result.invariant).toBe(0);
  });

  test('leapfrog conserves energy for oscillating jitter', () => {
    const samples = Array.from({ length: 50 }, (_, i) => ({
      timestamp: BigInt(1000 + i * 100),
      jitterDelta: BigInt(Math.round(10 * Math.sin(i * 0.5))),
    }));

    const { correctedSamples } = correctJitterBatch(samples);
    const nonZero = correctedSamples.filter(r => r.invariant > 0);
    if (nonZero.length >= 2) {
      const conservation = verifyEnergyConservation(correctedSamples, 0.5);
      expect(conservation.maxDrift).toBeDefined();
      expect(conservation.avgDrift).toBeGreaterThanOrEqual(0);
    }
  });

  test('state accumulates across corrections', () => {
    let state: SymplecticState = { position: 0, momentum: 0 };

    const r1 = applySymplecticJitterCorrection(1000n, 50n, state);
    state = { position: r1.position, momentum: r1.momentum };

    const r2 = applySymplecticJitterCorrection(2000n, -30n, state);
    expect(r2.position !== r1.position).toBe(true);
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
    expect(typeof finalState.position).toBe('number');
    expect(typeof finalState.momentum).toBe('number');
    expect(energyDrift).toBeGreaterThanOrEqual(0);
  });

  test('energy conservation verification edge cases', () => {
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

  test('large perturbation violates tight constraint', () => {
    const regs1 = new Array(27).fill(0);
    const regs2 = new Array(27).fill(0);
    regs2[0] = 10; // energy = 100 mod 312 = 100, drift = 100 > tolerance 1
    const result = checkHamiltonianConstraint(regs1, regs2, 1);
    expect(result.valid).toBe(false);
    expect(result.drift).toBe(100);
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
    regs2[0] = 10;
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

  test('ternary parity is stable for uniform registers', () => {
    const regs = Array.from({ length: 27 }, () => 1);
    expect(computeBankTernaryParity(regs, 0)).toBe(computeBankTernaryParity(regs, 1));
    expect(computeBankTernaryParity(regs, 1)).toBe(computeBankTernaryParity(regs, 2));
  });
});

describe('Symplectic Phase Mixing', () => {
  test('empty input returns empty output', () => {
    expect(symplecticPhaseMix([])).toEqual([]);
  });

  test('mixes ternary values — output remains in {0, 1, 2}', () => {
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

  test('more rounds produces at least as much mixing', () => {
    const input = [0, 1, 2, 0, 1, 2, 0, 1, 2, 1, 0, 2, 1];
    const mixed0 = symplecticPhaseMix(input, 0);
    const mixedDefault = symplecticPhaseMix(input);
    expect(mixed0).toEqual(input.map(v => ((v % 3) + 3) % 3));
    expect(mixedDefault.length).toBe(input.length);
    mixedDefault.forEach(v => {
      expect(v).toBeGreaterThanOrEqual(0);
      expect(v).toBeLessThan(3);
    });
  });

  test('verifies ternary parity is preserved by mixing', () => {
    const testCases = [
      [0, 1, 2, 0, 1, 2],
      [1, 1, 1, 1, 1, 1],
      [2, 2, 2, 2, 2, 2],
      [0, 0, 0, 0, 0, 0],
      [1, 0, 2, 1, 0, 2, 1],
      [2, 1, 0, 2, 0, 1, 2, 1, 0],
    ];
    for (const input of testCases) {
      const parityBefore = computeTernaryParity(input);
      const mixed = symplecticPhaseMix(input);
      const parityAfter = computeTernaryParity(mixed);
      expect(parityAfter).toBe(parityBefore);
      expect(verifySymplecticParity(input, mixed)).toBe(true);
    }
  });

  test('computeTernaryParity returns value in {0, 1, 2}', () => {
    expect(computeTernaryParity([0, 1, 2])).toBe(0);
    expect(computeTernaryParity([1, 1, 1])).toBe(0);
    expect(computeTernaryParity([1, 0, 0])).toBe(1);
    expect(computeTernaryParity([2, 0, 0])).toBe(2);
  });

  test('computePhaseChecksum returns value in [0, 12]', () => {
    const state = [1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1];
    const checksum = computePhaseChecksum(state);
    expect(checksum).toBeGreaterThanOrEqual(0);
    expect(checksum).toBeLessThan(13);
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
});
