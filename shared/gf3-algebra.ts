// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// GF(3) Algebra — Closed-Form Ternary-Native Operations
// Location: shared/gf3-algebra.ts
// Mirrors: ternary-math/src/gf3_algebra.rs
//
// Replaces trit-by-trit loops with algebraic formulas.
// The ternary math IS the optimization — no binary encoding tricks.

// ============================================================
// GF(3) ELEMENT OPERATIONS
// ============================================================

/** GF(3) addition: (a + b) mod 3 */
export function gf3Add(a: number, b: number): number {
  return (a + b) % 3;
}

/** GF(3) subtraction: (a - b + 3) mod 3 */
export function gf3Sub(a: number, b: number): number {
  return (a + 3 - b) % 3;
}

/** GF(3) multiplication: (a × b) mod 3 */
export function gf3Mul(a: number, b: number): number {
  return (a * b) % 3;
}

/** GF(3) negation: 0→0, 1→2, 2→1 */
export function gf3Neg(a: number): number {
  return (3 - a) % 3;
}

/**
 * GF(3) square: a² mod 3. This is the Hamming indicator.
 * 0→0 (same), 1→1 (different), 2→1 (different).
 */
export function gf3Square(a: number): number {
  return (a * a) % 3;
}

// ============================================================
// REP CONVERSIONS
// ============================================================

/** Rep C {1,2,3} → Rep B {0,1,2} */
export function repCtoB(c: number): number { return c - 1; }

/** Rep B {0,1,2} → Rep C {1,2,3} */
export function repBtoC(b: number): number { return b + 1; }

// ============================================================
// HAMMING DISTANCE — Σ (aᵢ - bᵢ)² mod 3
// ============================================================
//
// In GF(3): (a-b)² = 0 if a=b, 1 if a≠b.
// Proof: 0²=0, 1²=1, 2²=4≡1 mod 3.
// Sum of squared differences = count of mismatches.
// Pure algebra, no branching, no comparison operators.

/**
 * Hamming distance between two Rep B trit vectors.
 * Formula: d = Σ (aᵢ - bᵢ)² mod 3
 */
export function hammingDistance(a: number[], b: number[]): number {
  let dist = 0;
  for (let i = 0; i < a.length; i++) {
    const diff = (a[i] + 3 - b[i]) % 3;
    dist += (diff * diff) % 3;
  }
  return dist;
}

/**
 * Hamming distance between two Rep C trit vectors.
 * The mod-3 arithmetic absorbs the Rep C offset naturally:
 * (a - b + 3) % 3 works identically for Rep C inputs because
 * the +1 offsets cancel in the subtraction.
 */
export function hammingDistanceRepC(a: number[], b: number[]): number {
  let dist = 0;
  for (let i = 0; i < a.length; i++) {
    const diff = (a[i] + 3 - b[i]) % 3;
    dist += (diff * diff) % 3;
  }
  return dist;
}

// ============================================================
// FORGERY DETECTION — Multiplicative Closure
// ============================================================
//
// Rep C valid trits: {1, 2, 3}. All nonzero, all coprime to 7.
// Product mod 7 of valid trits is always nonzero.
// If any trit is 0 (forged), product mod 7 becomes 0.

/**
 * Check for forgery in a Rep C trit vector.
 * Returns true if any trit is 0 (zero = forgery in Rep C).
 * Uses running product mod 7 with early exit.
 */
export function hasForgery(tritsRepC: number[]): boolean {
  let product = 1;
  for (let i = 0; i < tritsRepC.length; i++) {
    product = (product * tritsRepC[i]) % 7;
    if (product === 0) return true;
  }
  return false;
}

/** Locate all forged positions (where trit = 0). */
export function findForgeries(tritsRepC: number[]): number[] {
  return tritsRepC
    .map((t, i) => t === 0 ? i : -1)
    .filter(i => i >= 0);
}

// ============================================================
// BATCH GF(3) VECTOR OPERATIONS
// ============================================================

/** Element-wise GF(3) addition */
export function gf3VecAdd(a: number[], b: number[]): number[] {
  return a.map((v, i) => (v + b[i]) % 3);
}

/** Element-wise GF(3) subtraction */
export function gf3VecSub(a: number[], b: number[]): number[] {
  return a.map((v, i) => (v + 3 - b[i]) % 3);
}

/** Element-wise GF(3) multiplication */
export function gf3VecMul(a: number[], b: number[]): number[] {
  return a.map((v, i) => (v * b[i]) % 3);
}

/** GF(3) dot product: Σ (aᵢ × bᵢ) mod 3 */
export function gf3Dot(a: number[], b: number[]): number {
  let sum = 0;
  for (let i = 0; i < a.length; i++) {
    sum = (sum + a[i] * b[i]) % 3;
  }
  return sum;
}

/** Scalar × vector in GF(3) */
export function gf3ScalarMul(scalar: number, a: number[]): number[] {
  return a.map(v => (scalar * v) % 3);
}

// ============================================================
// SPONGE PERMUTATION — Index Remap (Zero Data Movement)
// ============================================================
//
// tisPi: new[i] = old[(i × stride) mod width]
// INVARIANT 10: gcd(stride, width) = 1 for complete cycle.
// Data doesn't move — the read pattern changes.

/**
 * Apply stride-s permutation. out[i] = state[(i × stride) mod W].
 */
export function spongePermute(state: number[], stride: number): number[] {
  const w = state.length;
  return Array.from({ length: w }, (_, i) => state[(i * stride) % w]);
}

/**
 * Precompute permutation indices for repeated use.
 */
export function precomputePermutation(stride: number, width: number): number[] {
  return Array.from({ length: width }, (_, i) => (i * stride) % width);
}

/**
 * Apply precomputed permutation.
 */
export function applyPermutation(state: number[], perm: number[]): number[] {
  return perm.map(idx => state[idx]);
}

// ============================================================
// SPONGE DIFFUSION — Circulant Neighbor Sum
// ============================================================
//
// tisTheta: out[i] = (left + center + right) mod 3
// Circular: state[-1] = state[W-1], state[W] = state[0].

/**
 * Theta diffusion: out[i] = (state[i-1] + state[i] + state[i+1]) mod 3
 */
export function spongeTheta(state: number[]): number[] {
  const w = state.length;
  return Array.from({ length: w }, (_, i) => {
    const left   = state[(i + w - 1) % w];
    const center = state[i];
    const right  = state[(i + 1) % w];
    return (left + center + right) % 3;
  });
}

/**
 * Theta with round constant addition.
 */
export function spongeThetaWithConstant(state: number[], constants: number[]): number[] {
  const w = state.length;
  return Array.from({ length: w }, (_, i) => {
    const left   = state[(i + w - 1) % w];
    const center = state[i];
    const right  = state[(i + 1) % w];
    return (left + center + right + constants[i % constants.length]) % 3;
  });
}

// ============================================================
// TIS-27 SPONGE
// ============================================================

export const TIS27_ROUND_CONSTANTS = [
  0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0
];

export const TIS27_STATE_WIDTH = 54;
export const TIS27_RATE = 27;
export const TIS27_ROUNDS = 27;
export const TIS27_STRIDE = 13;

/** Precomputed stride-13 permutation for width 54 */
const TIS27_PERM = precomputePermutation(TIS27_STRIDE, TIS27_STATE_WIDTH);

/**
 * One TIS-27 round: theta → pi → round constant addition.
 * Mutates state in place.
 */
export function tis27Round(state: number[], round: number): void {
  // Step 1: Theta
  const afterTheta = spongeTheta(state);

  // Step 2: Pi (stride-13 permutation via precomputed indices)
  const afterPi = applyPermutation(afterTheta, TIS27_PERM);

  // Step 3: Round constant addition (rate portion)
  const rcOffset = round % TIS27_ROUND_CONSTANTS.length;
  for (let i = 0; i < TIS27_RATE; i++) {
    afterPi[i] = (afterPi[i] + TIS27_ROUND_CONSTANTS[(i + rcOffset) % TIS27_ROUND_CONSTANTS.length]) % 3;
  }

  // Write back
  for (let i = 0; i < TIS27_STATE_WIDTH; i++) {
    state[i] = afterPi[i];
  }
}

/**
 * Full TIS-27 sponge: absorb + squeeze.
 *
 * @param inputTrits - GF(3) trit values to absorb
 * @param outputLen - Number of output trits to squeeze
 */
export function tis27Sponge(inputTrits: number[], outputLen: number): number[] {
  const state = new Array(TIS27_STATE_WIDTH).fill(0);

  // Absorb
  let offset = 0;
  while (offset < inputTrits.length) {
    const blockLen = Math.min(TIS27_RATE, inputTrits.length - offset);
    for (let i = 0; i < blockLen; i++) {
      state[i] = (state[i] + inputTrits[offset + i]) % 3;
    }
    for (let round = 0; round < TIS27_ROUNDS; round++) {
      tis27Round(state, round);
    }
    offset += TIS27_RATE;
  }

  // Squeeze
  const output: number[] = [];
  while (output.length < outputLen) {
    const take = Math.min(TIS27_RATE, outputLen - output.length);
    output.push(...state.slice(0, take));
    if (output.length < outputLen) {
      for (let round = 0; round < TIS27_ROUNDS; round++) {
        tis27Round(state, round);
      }
    }
  }
  return output.slice(0, outputLen);
}

// ============================================================
// REPUNIT CHECKSUM — Horner's mod R₆ = 364
// ============================================================

/**
 * Repunit checksum: interpret Rep C trits as base-3 (Rep B),
 * evaluate via Horner's method with running mod-364 reduction.
 * 3⁶ ≡ 1 (mod 364) gives natural period-6 wrapping.
 */
export function repunitChecksum(tritsRepC: number[]): number {
  let value = 0;
  for (let i = tritsRepC.length - 1; i >= 0; i--) {
    value = (value * 3 + (tritsRepC[i] - 1)) % 364;
  }
  return value;
}

// ============================================================
// DERIVATION — INVARIANT 2
// ============================================================

/**
 * Universal derivation: signal count → GF(3) value.
 * gf3 = min(floor(3k / N), 2)
 * Boundaries at N/3 and 2N/3. No tuning parameters.
 */
export function projectToGf3(k: number, n: number): number {
  return Math.min(Math.floor(3 * k / n), 2);
}

/** Derivation with lift to Rep C. */
export function deriveTrit(k: number, n: number): number {
  return projectToGf3(k, n) + 1;
}
