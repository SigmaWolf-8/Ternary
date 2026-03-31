// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Multi-Generator Agent Scheduling
//
// Extends the 28-agent array to support all 12 generators of Z₂₈.
// The coprime walk theorem guarantees: step `a` visits all 28 positions
// iff gcd(a, 28) = 1. There are exactly φ(28) = 12 such generators.
//
// Usage:
//   import { getAgentWalk, Z28_GENERATORS, Z28_GENERATOR_INVERSES } from './agent-generators';
//   const walk = getAgentWalk(13);  // default: step 13 (T₇)
//   const reverseWalk = getAgentWalk(Z28_GENERATOR_INVERSES[13]); // same as forward (13 is self-inverse)

// ============================================================
// PLATFORM CONSTANTS — add to shared/constants.ts → PLATFORM
// ============================================================

/**
 * All 12 generators (units) of Z₂₈.
 * Each generates a complete walk visiting all 28 positions exactly once.
 * φ(28) = φ(4) × φ(7) = 2 × 6 = 12.
 */
export const Z28_GENERATORS = [1, 3, 5, 9, 11, 13, 15, 17, 19, 23, 25, 27] as const;

/**
 * Multiplicative inverse of each generator mod 28.
 * For generator g: g × g⁻¹ ≡ 1 (mod 28).
 * The inverse walk visits positions in reverse order.
 *
 * Self-inverse generators (g = g⁻¹): 1, 13, 15, 27
 * Inverse pairs: 3↔19, 5↔17, 9↔25, 11↔23
 */
export const Z28_GENERATOR_INVERSES: Record<number, number> = {
  1: 1,    // 1 × 1 = 1 ≡ 1 (mod 28)  [self-inverse]
  3: 19,   // 3 × 19 = 57 = 2×28 + 1    [pair]
  5: 17,   // 5 × 17 = 85 = 3×28 + 1    [pair]
  9: 25,   // 9 × 25 = 225 = 8×28 + 1   [pair]
  11: 23,  // 11 × 23 = 253 = 9×28 + 1  [pair]
  13: 13,  // 13 × 13 = 169 = 6×28 + 1  [self-inverse]
  15: 15,  // 15 × 15 = 225 = 8×28 + 1  [self-inverse]
  17: 5,   // inverse of 5               [pair]
  19: 3,   // inverse of 3               [pair]
  23: 11,  // inverse of 11              [pair]
  25: 9,   // inverse of 9               [pair]
  27: 27,  // 27 × 27 = 729 = 26×28 + 1 [self-inverse]
};

/**
 * The canonical generator — T₇ = 13 = 111₃ = 1 ternary radian.
 * Default for all scheduling unless explicitly overridden.
 */
export const Z28_CANONICAL_GENERATOR = 13;

/**
 * Z₂₈ agent array size — 2π radians.
 */
export const Z28_SIZE = 28;

// ============================================================
// GCD utility (constant-time for small values)
// ============================================================

/**
 * Greatest common divisor via Euclidean algorithm.
 */
export function gcd(a: number, b: number): number {
  a = Math.abs(a);
  b = Math.abs(b);
  while (b !== 0) {
    const t = b;
    b = a % b;
    a = t;
  }
  return a;
}

/**
 * Check if a step is a valid generator of Z_m.
 */
export function isGenerator(step: number, modulus: number): boolean {
  return step > 0 && step < modulus && gcd(step, modulus) === 1;
}

// ============================================================
// Walk generation
// ============================================================

/**
 * Generate the complete walk sequence for a given generator.
 *
 * @param generator - Step size (must be coprime to 28). Default: 13.
 * @returns Array of 28 positions in walk order, starting from 0.
 * @throws If generator is not coprime to 28.
 *
 * @example
 *   getAgentWalk(13)  → [0, 13, 26, 11, 24, 9, 22, 7, 20, 5, 18, 3, 16, 1, ...]
 *   getAgentWalk(3)   → [0, 3, 6, 9, 12, 15, 18, 21, 24, 27, 2, 5, 8, 11, ...]
 *   getAgentWalk(19)  → reverse of step-3 walk
 */
export function getAgentWalk(generator: number = Z28_CANONICAL_GENERATOR): number[] {
  if (!isGenerator(generator, Z28_SIZE)) {
    throw new Error(
      `Generator ${generator} is not coprime to ${Z28_SIZE} (gcd=${gcd(generator, Z28_SIZE)}). ` +
      `Valid generators: [${Z28_GENERATORS.join(', ')}]`
    );
  }

  const walk: number[] = new Array(Z28_SIZE);
  for (let k = 0; k < Z28_SIZE; k++) {
    walk[k] = (k * generator) % Z28_SIZE;
  }
  return walk;
}

/**
 * Get the agent position at step k for a given generator.
 *
 * @param k - Step index (0-based).
 * @param generator - Walk generator (default: 13).
 * @returns Position in Z₂₈.
 */
export function getAgentPosition(k: number, generator: number = Z28_CANONICAL_GENERATOR): number {
  return (k * generator) % Z28_SIZE;
}

/**
 * Get the reverse walk (using the multiplicative inverse generator).
 * The reverse walk visits the same positions in opposite order.
 *
 * @param generator - Original generator (default: 13).
 * @returns Array of 28 positions in reverse walk order.
 */
export function getReverseWalk(generator: number = Z28_CANONICAL_GENERATOR): number[] {
  const inverse = Z28_GENERATOR_INVERSES[generator];
  if (inverse === undefined) {
    throw new Error(`No known inverse for generator ${generator}`);
  }
  return getAgentWalk(inverse);
}

/**
 * Generate parallel schedule assignments for fault-tolerant monitoring.
 *
 * Assigns each agent to multiple schedules using different generators.
 * If one schedule's heartbeat is missed, another covers the same agent.
 *
 * @param generatorCount - How many parallel schedules (1-12). Default: 3.
 * @returns Array of walks, one per generator selected.
 */
export function getParallelSchedules(generatorCount: number = 3): {
  generator: number;
  walk: number[];
}[] {
  const count = Math.min(Math.max(1, generatorCount), Z28_GENERATORS.length);

  // Select generators that are maximally spread across the set.
  // Strategy: pick every (12/count)-th generator from the sorted list.
  const selected: number[] = [];
  const step = Z28_GENERATORS.length / count;
  for (let i = 0; i < count; i++) {
    selected.push(Z28_GENERATORS[Math.floor(i * step)]);
  }

  return selected.map(gen => ({
    generator: gen,
    walk: getAgentWalk(gen),
  }));
}

// ============================================================
// Convolution kernel (preserved from original)
// ============================================================

/**
 * Tribonacci convolution kernel: [T₇, T₈, T₉] = [13, 24, 44].
 * Three consecutive Tribonacci numbers used for agent response weighting.
 */
export const TRIBONACCI_KERNEL = [13, 24, 44] as const;

/**
 * The calendar identity: 13 × 28 = 364 = 111111₃ = full ternary circle.
 */
export const CALENDAR_IDENTITY = Z28_CANONICAL_GENERATOR * Z28_SIZE; // 364

export const Z364_CANONICAL_GENERATOR = 11;

export const DUAL_GENERATOR_PAIR = {
  primary: {
    stride: 13,
    role: 'radian_cycle' as const,
    group: 'Z28' as const,
    inverseStride: 13,
    generatesZ364: false,
  },
  secondary: {
    stride: 11,
    role: 'full_circle' as const,
    group: 'Z364' as const,
    inverseStride: 23,
    generatesZ364: true,
  },
  arc: 143,
  combinedVertices: 23,
  eulerTotient: 120,
  interleave: [1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1] as const,
  bezout: [6, -5] as const,
} as const;

export function getStride11Walk(): number[] {
  return Array.from({ length: 28 }, (_, k) => (11 * k) % 28);
}

export function getDualGeneratorSchedule(): {
  primaryWalk: number[];
  secondaryWalk: number[];
  interleave: readonly number[];
  combinedSchedule: Array<{ position: number; source: 'primary' | 'secondary' }>;
} {
  const primaryWalk = Array.from({ length: 28 }, (_, k) => (13 * k) % 28);
  const secondaryWalk = Array.from({ length: 28 }, (_, k) => (11 * k) % 28);
  const combined: Array<{ position: number; source: 'primary' | 'secondary' }> = [];
  const il = DUAL_GENERATOR_PAIR.interleave;
  let pi = 0, si = 0;

  for (let i = 0; i < il.length && si < 28; i++) {
    for (let j = 0; j < il[i] && pi < 28; j++) {
      combined.push({ position: primaryWalk[pi++], source: 'primary' });
    }
    if (si < 28) combined.push({ position: secondaryWalk[si++], source: 'secondary' });
  }
  while (pi < 28) combined.push({ position: primaryWalk[pi++], source: 'primary' });
  while (si < 28) combined.push({ position: secondaryWalk[si++], source: 'secondary' });

  return { primaryWalk, secondaryWalk, interleave: il, combinedSchedule: combined };
}
