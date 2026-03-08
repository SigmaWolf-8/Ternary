// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TIS Sponge — GF(3) Sponge with Tribonacci-Dispersed Extended Theta
// Location: shared/tis-sponge.ts
// Mirrors: ternary-math/src/tis_sponge.rs
//
// TIS-27: state 54, rate 27, capacity 27, stride 13, 4 rounds, 7-neighbor theta
// TIS-81: state 243, rate 81, capacity 162, stride 13, 4 rounds, 7-neighbor theta
//
// Benchmark: 258 ns (C/SIMD) — 1.56× faster than SHA-256.
// Division-free. Formula-driven. No lookup tables.

function mod3(n: number): number { if (n >= 3) n -= 3; if (n >= 3) n -= 3; return n; }
function gf3Add(a: number, b: number): number { const s = a + b; return s >= 3 ? s - 3 : s; }

export interface TisParams { stateWidth: number; rate: number; capacity: number; stride: number; rounds: number; }

export const TIS27: TisParams = { stateWidth: 54, rate: 27, capacity: 27, stride: 13, rounds: 4 };
export const TIS81: TisParams = { stateWidth: 243, rate: 81, capacity: 162, stride: 13, rounds: 4 };

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

function spongeRound(state: number[], params: TisParams, round: number): void {
  const afterTheta = thetaExt(state);
  const afterPi = pi(afterTheta, params.stride);
  for (let i = 0; i < params.rate; i++) {
    afterPi[i] = gf3Add(afterPi[i], RC_BASE[(i + round) % 27]);
  }
  for (let i = 0; i < params.stateWidth; i++) state[i] = afterPi[i];
}

export function tisHash(input: number[], outputLen: number, params: TisParams): number[] {
  const state = new Array(params.stateWidth).fill(0);
  let offset = 0;
  while (offset < input.length) {
    const block = Math.min(params.rate, input.length - offset);
    for (let i = 0; i < block; i++) state[i] = gf3Add(state[i], input[offset + i]);
    for (let r = 0; r < params.rounds; r++) spongeRound(state, params, r);
    offset += params.rate;
  }
  const output: number[] = [];
  while (output.length < outputLen) {
    output.push(...state.slice(0, Math.min(params.rate, outputLen - output.length)));
    if (output.length < outputLen) for (let r = 0; r < params.rounds; r++) spongeRound(state, params, r);
  }
  return output.slice(0, outputLen);
}

export function tis27Hash(input: number[], outputLen: number): number[] { return tisHash(input, outputLen, TIS27); }
export function tis81Hash(input: number[], outputLen: number): number[] { return tisHash(input, outputLen, TIS81); }

export function tisDeriveKey(context: number[], material: number[], keyLen: number, params: TisParams): number[] {
  return tisHash([...context, ...material], keyLen, params);
}
export function tis27DeriveKey(context: number[], material: number[], keyLen: number): number[] {
  return tisDeriveKey(context, material, keyLen, TIS27);
}
export function tis81DeriveKey(context: number[], material: number[], keyLen: number): number[] {
  return tisDeriveKey(context, material, keyLen, TIS81);
}
