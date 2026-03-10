/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TL-SPONGE — TypeScript port of TL-Sponge-385 (Rust: TernarySponge).
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

const WRAP_TABLE = new Int8Array([
  -1, 0, 1, -1, 0, 1, -1, 0, 1
]);

function balancedWrap(s: number): number {
  return WRAP_TABLE[s + 4];
}

const TRIT_ADD_TABLE = new Int8Array([
  1, -1, 0, 1, -1
]);

function tritAdd(a: number, b: number): number {
  return TRIT_ADD_TABLE[a + b + 2];
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

export function spongeKeystream(domainInput: Buffer | Uint8Array, outputTritCount: number): Int8Array {
  const inputTrits = bytesToBalancedTrits(domainInput);
  return spongeAbsorbAndSqueeze(inputTrits, outputTritCount);
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

export class SpongeDuplex {
  private state: Int8Array;
  private buf: Int8Array;
  private bufLen: number;
  private absorbed: boolean;

  constructor() {
    this.state = new Int8Array(STATE_SIZE);
    this.buf = new Int8Array(RATE);
    this.bufLen = 0;
    this.absorbed = false;
  }

  absorbTrits(trits: Int8Array): void {
    this._absorbRaw(trits);
  }

  absorb(input: Buffer | Uint8Array): void {
    const inputTrits = bytesToBalancedTrits(input);
    this._absorbRaw(inputTrits);
  }

  private _absorbRaw(inputTrits: Int8Array): void {

    this.absorbed = true;
    let offset = 0;
    const inputLen = inputTrits.length;

    if (this.bufLen > 0) {
      const space = RATE - this.bufLen;
      const fill = Math.min(inputLen, space);
      this.buf.set(inputTrits.subarray(0, fill), this.bufLen);
      this.bufLen += fill;
      offset = fill;

      if (this.bufLen === RATE) {
        for (let i = 0; i < RATE; i++) {
          this.state[i] = tritAdd(this.state[i], this.buf[i]);
        }
        spongePermutation(this.state);
        this.bufLen = 0;
      }
    }

    while (offset + RATE <= inputLen) {
      for (let i = 0; i < RATE; i++) {
        this.state[i] = tritAdd(this.state[i], inputTrits[offset + i]);
      }
      spongePermutation(this.state);
      offset += RATE;
    }

    const remaining = inputLen - offset;
    if (remaining > 0) {
      this.buf.set(inputTrits.subarray(offset, offset + remaining), this.bufLen);
      this.bufLen += remaining;
    }
  }

  squeeze(tritCount: number): Int8Array {
    if (this.bufLen > 0 || !this.absorbed) {
      for (let i = 0; i < this.bufLen; i++) {
        this.state[i] = tritAdd(this.state[i], this.buf[i]);
      }
      if (this.bufLen < RATE) {
        this.state[this.bufLen] = tritAdd(this.state[this.bufLen], 1);
      }
      this.bufLen = 0;
      this.absorbed = false;
      spongePermutation(this.state);
    }

    const output = new Int8Array(tritCount);
    let written = 0;
    while (written < tritCount) {
      const take = Math.min(RATE, tritCount - written);
      output.set(this.state.subarray(0, take), written);
      written += take;
      if (written < tritCount) {
        spongePermutation(this.state);
      }
    }
    return output;
  }

  reset(): void {
    this.state.fill(0);
    this.buf.fill(0);
    this.bufLen = 0;
    this.absorbed = false;
  }
}

export function tritsToHex(trits: Int8Array): string {
  const byteLen = Math.ceil(trits.length / 5);
  const bytes = tritsToBytes(trits, byteLen);
  return bytes.toString('hex');
}

export const TL_SPONGE_HASH_BYTES = 49;
export const TL_SPONGE_HASH_HEX_LEN = 98;
export const TL_SPONGE_HASH_TRITS = 243;
export const TL_SPONGE_SECURITY_BITS = 385;
export const TL_SPONGE_OID = '1.3.6.1.4.1.0.100.3.1';
export const TL_SPONGE_ALGORITHM_NAME = 'tl-sponge';
