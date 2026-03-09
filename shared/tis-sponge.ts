// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TIS-27 Fast Ternary Integrity Function
// Location: shared/tis-sponge.ts
// Mirrors: ternary-math/src/tis_sponge.rs
//
// ┌─────────────────────────────────────────────────────────────────┐
// │  THIS IS NOT A CRYPTOGRAPHIC HASH.                              │
// │                                                                 │
// │  TIS-27 provides fast corruption detection for wire packets.    │
// │  27-trit capacity = 43 bits — insufficient for cryptographic    │
// │  security. For cryptographic operations, use TL-Sponge.         │
// └─────────────────────────────────────────────────────────────────┘
//
// Use: wire integrity, scan hashing on authenticated channels
// NOT for: signing, key derivation, identity binding, registration

function mod3(n: number): number { if (n >= 3) n -= 3; if (n >= 3) n -= 3; return n; }
function gf3Add(a: number, b: number): number { const s = a + b; return s >= 3 ? s - 3 : s; }

const RC_BASE = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

function thetaExt(state: number[]): number[] {
  const w = state.length;
  const out = new Array(w);
  for (let i = 0; i < w; i++) {
    const left = mod3(state[(i + w - 13) % w] + state[(i + w - 7) % w] + state[(i + w - 1) % w]);
    const right = mod3(state[(i + 1) % w] + state[(i + 7) % w] + state[(i + 13) % w]);
    out[i] = mod3(left + state[i] + right);
  }
  return out;
}

function pi(state: number[], stride: number): number[] {
  const w = state.length;
  return Array.from({ length: w }, (_, i) => state[(i * stride) % w]);
}

function spongeRound(state: number[], round: number): void {
  const afterTheta = thetaExt(state);
  const afterPi = pi(afterTheta, 13);
  for (let i = 0; i < 27; i++) {
    let x = i + round;
    if (x >= 27) x -= 27;
    afterPi[i] = gf3Add(afterPi[i], RC_BASE[x]);
  }
  for (let i = 0; i < 54; i++) state[i] = afterPi[i];
}

/**
 * TIS-27 fast integrity hash. NOT cryptographic.
 * For cryptographic hashing, use TL-Sponge via API.
 */
export function tis27Hash(input: number[], outputLen: number): number[] {
  const state = new Array(54).fill(0);
  const block = Math.min(27, input.length);
  for (let i = 0; i < block; i++) state[i] = gf3Add(state[i], input[i]);
  for (let r = 0; r < 4; r++) spongeRound(state, r);
  return state.slice(0, Math.min(27, outputLen));
}

/**
 * Derive integrity key. For wire context only.
 * NOT for cryptographic key derivation.
 */
export function tis27DeriveKey(context: number[], material: number[], keyLen: number): number[] {
  return tis27Hash([...context, ...material], keyLen);
}
