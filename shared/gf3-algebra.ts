// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// GF(3) Algebra — Division-Free Ternary-Native Operations
// Location: shared/gf3-algebra.ts
// Mirrors: ternary-math/src/gf3_algebra.rs
//
// Division-free: conditional subtract instead of % operator.
// Sponge code lives in tis-sponge.ts — not here.

// ── Division-free reduction ─────────────────────────────────────────

function mod3s(n: number): number { return n >= 3 ? n - 3 : n; }
function mod7s(n: number): number { if (n >= 14) n -= 14; if (n >= 7) n -= 7; return n; }

// ── GF(3) element operations ────────────────────────────────────────

export function gf3Add(a: number, b: number): number { return mod3s(a + b); }
export function gf3Sub(a: number, b: number): number { return mod3s(a + 3 - b); }
export function gf3Mul(a: number, b: number): number { return mod3s(a * b); }
export function gf3Neg(a: number): number { const s = 3 - a; return s >= 3 ? 0 : s; }
export function gf3Square(a: number): number { return mod3s(a * a); }

export function repCtoB(c: number): number { return c - 1; }
export function repBtoC(b: number): number { return b + 1; }

// ── Hamming distance: Σ(a-b)² mod 3 ────────────────────────────────

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

// ── Repunit checksum (Horner mod 364 — % stays, unbounded accumulator) ──

export function repunitChecksum(tritsRepC: number[]): number {
  let value = 0;
  for (let i = tritsRepC.length - 1; i >= 0; i--) value = (value * 3 + (tritsRepC[i] - 1)) % 364;
  return value;
}

// ── Derivation — INVARIANT 2 ────────────────────────────────────────

export function projectToGf3(k: number, n: number): number { return Math.min(Math.floor(3*k/n), 2); }
export function deriveTrit(k: number, n: number): number { return projectToGf3(k, n) + 1; }
