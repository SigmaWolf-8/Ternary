/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TL-SPONGE — TypeScript port of TL-Sponge (Rust: TernarySponge).
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/crypto/sponge-hash.ts
 * Source:     src/kernel/src/crypto/sponge.rs
 *
 * 729-trit (3⁶) sponge construction for cryptographic document hashing.
 * Parameters mirror the Rust kernel exactly:
 *   State:    729 trits  (3⁶)
 *   Rate:     243 trits  (3⁵)
 *   Capacity: 486 trits  → 385-bit post-quantum security
 *   Rounds:   9          (3² — 3× safety margin over 3-round full diffusion)
 *   Theta:    7-neighbor substitution at distances ±1, ±7, ±13
 *   Pi:       stride 376, offset +1: π(i) = (376·i + 1) mod 729
 *
 * Arithmetic: balanced ternary {-1, 0, +1}. All operations wrap mod 3.
 * Output: 243 trits squeezed → packed to 49 bytes (5 trits/byte, lossless) hex.
 */

const STATE_SIZE = 729;
const RATE = 243;
const ROUNDS = 9;
const LANES = 27;

const PERM: number[] = (() => {
  const p = new Array<number>(STATE_SIZE);
  for (let i = 0; i < STATE_SIZE; i++) {
    p[i] = (i * 376 + 1) % STATE_SIZE;
  }
  return p;
})();

const RC_TABLE: Int8Array[] = (() => {
  const rcs: Int8Array[] = [];
  for (let r = 0; r < ROUNDS; r++) {
    const row = new Int8Array(LANES);
    for (let lane = 0; lane < LANES; lane++) {
      row[lane] = ((r * 7 + lane * 13 + 3) % 3) - 1;
    }
    rcs.push(row);
  }
  return rcs;
})();

function balancedWrap(s: number): number {
  if (s >= 2) return s - 3;
  if (s <= -2) return s + 3;
  return s;
}

function tritAdd(a: number, b: number): number {
  const s = a + b;
  if (s > 1) return s - 3;
  if (s < -1) return s + 3;
  return s;
}

function spongePermutation(state: Int8Array): void {
  const ext = new Int8Array(STATE_SIZE + 26);
  const buf = new Int8Array(STATE_SIZE);

  for (let round = 0; round < ROUNDS; round++) {
    ext.set(state.subarray(STATE_SIZE - 13), 0);
    ext.set(state, 13);
    ext.set(state.subarray(0, 13), 13 + STATE_SIZE);

    for (let i = 0; i < STATE_SIZE; i++) {
      const ei = i + 13;
      const left = balancedWrap(ext[ei - 13] + ext[ei - 7] + ext[ei - 1]);
      const right = balancedWrap(ext[ei + 1] + ext[ei + 7] + ext[ei + 13]);
      buf[i] = balancedWrap(left + ext[ei] + right + 1);
    }

    for (let i = 0; i < STATE_SIZE; i++) {
      state[PERM[i]] = buf[i];
    }

    const rc = RC_TABLE[round];
    for (let lane = 0; lane < LANES; lane++) {
      const idx = lane * LANES;
      state[idx] = balancedWrap(state[idx] + rc[lane]);
    }
  }
}

function bytesToBalancedTrits(input: Buffer | Uint8Array): Int8Array {
  const trits: number[] = [];
  for (const byte of input) {
    let val = byte;
    for (let j = 0; j < 5; j++) {
      trits.push((val % 3) - 1);
      val = Math.floor(val / 3);
    }
  }
  return new Int8Array(trits);
}

function spongeAbsorbAndSqueeze(inputTrits: Int8Array, outputTrits: number): Int8Array {
  const state = new Int8Array(STATE_SIZE);

  let offset = 0;
  while (offset + RATE <= inputTrits.length) {
    for (let i = 0; i < RATE; i++) {
      state[i] = tritAdd(state[i], inputTrits[offset + i]);
    }
    spongePermutation(state);
    offset += RATE;
  }

  const remaining = inputTrits.length - offset;
  for (let i = 0; i < remaining; i++) {
    state[i] = tritAdd(state[i], inputTrits[offset + i]);
  }
  if (remaining < RATE) {
    state[remaining] = tritAdd(state[remaining], 1);
  }
  spongePermutation(state);

  const output = new Int8Array(outputTrits);
  let written = 0;
  while (written < outputTrits) {
    const take = Math.min(RATE, outputTrits - written);
    output.set(state.subarray(0, take), written);
    written += take;
    if (written < outputTrits) {
      spongePermutation(state);
    }
  }

  return output;
}

function tritsToBytes(trits: Int8Array, byteLen: number): Buffer {
  const out = Buffer.alloc(byteLen);
  let tritIdx = 0;
  for (let b = 0; b < byteLen; b++) {
    let val = 0;
    let mul = 1;
    for (let j = 0; j < 5 && tritIdx < trits.length; j++) {
      val += (trits[tritIdx] + 1) * mul;
      mul *= 3;
      tritIdx++;
    }
    out[b] = val & 0xff;
  }
  return out;
}

export function spongeHash(input: Buffer | Uint8Array): string {
  const inputTrits = bytesToBalancedTrits(input);
  const outputTrits = spongeAbsorbAndSqueeze(inputTrits, 243);
  const bytes = tritsToBytes(outputTrits, 49);
  return bytes.toString('hex');
}

export function spongeHashTrits(input: Buffer | Uint8Array): Int8Array {
  const inputTrits = bytesToBalancedTrits(input);
  return spongeAbsorbAndSqueeze(inputTrits, 243);
}

export const TL_SPONGE_HASH_BYTES = 49;
export const TL_SPONGE_HASH_HEX_LEN = 98;
export const TL_SPONGE_HASH_TRITS = 243;
export const TL_SPONGE_SECURITY_BITS = 385;
export const TL_SPONGE_OID = '1.3.6.1.4.1.0.100.3.1';
export const TL_SPONGE_ALGORITHM_NAME = 'tl-sponge';
