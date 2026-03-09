// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// GF(3) Algebra — Division-Free Ternary-Native Operations
// Location: shared/gf3-algebra.ts
// Mirrors: ternary-math/src/gf3_algebra.rs
//
// All GF(3) arithmetic uses conditional subtract instead of % operator.
// Values are bounded to {0,1,2} — general division is never needed.

// ── Division-free reduction ─────────────────────────────────────────

/** Reduce [0,4] → GF(3). One conditional subtract. */
function mod3s(n: number): number { return n >= 3 ? n - 3 : n; }

/** Reduce [0,6] → GF(3). Two conditional subtracts (for 3-input sums). */
function mod3m(n: number): number { if (n >= 3) n -= 3; if (n >= 3) n -= 3; return n; }

/** Reduce [0,18] → mod 7 (for forgery product). */
function mod7s(n: number): number { if (n >= 14) n -= 14; if (n >= 7) n -= 7; return n; }

/** Reduce [0,52] → mod 27 (for round constant indexing). */
function mod27s(n: number): number { return n >= 27 ? n - 27 : n; }

// ── GF(3) element operations ────────────────────────────────────────

export function gf3Add(a: number, b: number): number { return mod3s(a + b); }
export function gf3Sub(a: number, b: number): number { return mod3s(a + 3 - b); }
export function gf3Mul(a: number, b: number): number { return mod3s(a * b); }
export function gf3Neg(a: number): number { const s = 3 - a; return s >= 3 ? 0 : s; }
export function gf3Square(a: number): number { return mod3s(a * a); }

export function repCtoB(c: number): number { return c - 1; }
export function repBtoC(b: number): number { return b + 1; }

// ── Hamming distance: Σ(aᵢ-bᵢ)² mod 3 ─────────────────────────────

export function hammingDistance(a: number[], b: number[]): number {
  let dist = 0;
  for (let i = 0; i < a.length; i++) dist += gf3Square(gf3Sub(a[i], b[i]));
  return dist;
}

export function hammingDistanceRepC(a: number[], b: number[]): number {
  let dist = 0;
  for (let i = 0; i < a.length; i++) dist += gf3Square(gf3Sub(repCtoB(a[i]), repCtoB(b[i])));
  return dist;
}

// ── Forgery detection: product mod 7 ────────────────────────────────

export function hasForgery(tritsRepC: number[]): boolean {
  let product = 1;
  for (let i = 0; i < tritsRepC.length; i++) {
    product = mod7s(product * tritsRepC[i]);
    if (product === 0) return true;
  }
  return false;
}

export function findForgeries(tritsRepC: number[]): number[] {
  return tritsRepC.map((t, i) => t === 0 ? i : -1).filter(i => i >= 0);
}

// ── GF(3) vector operations ─────────────────────────────────────────

export function gf3VecAdd(a: number[], b: number[]): number[] { return a.map((v, i) => gf3Add(v, b[i])); }
export function gf3VecSub(a: number[], b: number[]): number[] { return a.map((v, i) => gf3Sub(v, b[i])); }
export function gf3VecMul(a: number[], b: number[]): number[] { return a.map((v, i) => gf3Mul(v, b[i])); }
export function gf3Dot(a: number[], b: number[]): number {
  let sum = 0; for (let i = 0; i < a.length; i++) sum = gf3Add(sum, gf3Mul(a[i], b[i])); return sum;
}
export function gf3ScalarMul(scalar: number, a: number[]): number[] { return a.map(v => gf3Mul(scalar, v)); }

// ── TIS-27 Sponge — division-free ───────────────────────────────────

export const TIS27_STATE_WIDTH = 54;
export const TIS27_RATE = 27;
export const TIS27_ROUNDS = 27;
export const TIS27_STRIDE = 13;

export const PI_TABLE = [
   0,13,26,39,52,11,24,37,50, 9,22,35,48, 7,20,33,46, 5,
  18,31,44, 3,16,29,42, 1,14,27,40,53,12,25,38,51,10,23,
  36,49, 8,21,34,47, 6,19,32,45, 4,17,30,43, 2,15,28,41
];

export const TIS27_ROUND_CONSTANTS = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

export function spongeTheta(state: number[]): number[] {
  const w = state.length;
  const out = new Array(w);
  out[0] = mod3m(state[w-1] + state[0] + state[1]);
  for (let i = 1; i < w - 1; i++) out[i] = mod3m(state[i-1] + state[i] + state[i+1]);
  out[w-1] = mod3m(state[w-2] + state[w-1] + state[0]);
  return out;
}

export function spongePi(state: number[]): number[] {
  return PI_TABLE.map(idx => state[idx]);
}

export function tis27Round(state: number[], round: number): void {
  const afterTheta = spongeTheta(state);
  const afterPi = PI_TABLE.map(idx => afterTheta[idx]);
  const off = round >= 27 ? round - 27 : round;
  for (let i = 0; i < TIS27_RATE; i++) {
    afterPi[i] = gf3Add(afterPi[i], TIS27_ROUND_CONSTANTS[mod27s(i + off)]);
  }
  for (let i = 0; i < TIS27_STATE_WIDTH; i++) state[i] = afterPi[i];
}

export function tis27Sponge(inputTrits: number[], outputLen: number): number[] {
  const state = new Array(TIS27_STATE_WIDTH).fill(0);
  let offset = 0;
  while (offset < inputTrits.length) {
    const blockLen = Math.min(TIS27_RATE, inputTrits.length - offset);
    for (let i = 0; i < blockLen; i++) state[i] = gf3Add(state[i], inputTrits[offset + i]);
    for (let round = 0; round < TIS27_ROUNDS; round++) tis27Round(state, round);
    offset += TIS27_RATE;
  }
  const output: number[] = [];
  while (output.length < outputLen) {
    output.push(...state.slice(0, Math.min(TIS27_RATE, outputLen - output.length)));
    if (output.length < outputLen) for (let r = 0; r < TIS27_ROUNDS; r++) tis27Round(state, r);
  }
  return output.slice(0, outputLen);
}

// ── Repunit checksum (Horner mod 364 — this % stays, values are unbounded) ──

export function repunitChecksum(tritsRepC: number[]): number {
  let value = 0;
  for (let i = tritsRepC.length - 1; i >= 0; i--) value = (value * 3 + (tritsRepC[i] - 1)) % 364;
  return value;
}

// ── Derivation — INVARIANT 2 ────────────────────────────────────────

export function projectToGf3(k: number, n: number): number { return Math.min(Math.floor(3*k/n), 2); }
export function deriveTrit(k: number, n: number): number { return projectToGf3(k, n) + 1; }
