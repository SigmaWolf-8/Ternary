/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TL-SPONGE — TypeScript port of TL-Sponge-385 (Rust: TernarySponge).
 * @version 2.1.0
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
 *   Chi:      S(x) = M·x¹⁷+c over GF(27), affine-composed  [v2]
 *   Theta:    7-neighbor substitution at distances ±1, ±7, ±13
 *   Pi:       stride 376, offset +1: π(i) = (376·i + 1) mod 729
 *
 * Arithmetic: balanced ternary {-1, 0, +1}. All operations wrap mod 3.
 * Output: 243 trits squeezed → packed to 49 bytes (5 trits/byte, lossless) hex.
 *
 * Versioning:
 *   v1 = theta → pi → round constants (no chi — backward compat)
 *   v2 = chi → theta → pi → round constants (current default)
 *
 * Native acceleration: when sponge-native.node (Rust N-API addon) is
 * available, hash and keystream functions dispatch to compiled native
 * code automatically. TypeScript implementation is the fallback.
 */

import { createRequire as _createRequire } from 'module';
import { fileURLToPath as _fileURLToPath } from 'url';
import { dirname as _dirname, resolve as _resolve } from 'path';
import { execFileSync as _execFileSync } from 'child_process';
import { existsSync as _existsSync } from 'fs';

function _resolveNativePath(): string {
  if (typeof __filename !== 'undefined') {
    return _resolve(_dirname(__filename), 'sponge-native.node');
  } else if (typeof import.meta?.url !== 'undefined') {
    const _f = _fileURLToPath(import.meta.url);
    return _resolve(_dirname(_f), 'sponge-native.node');
  }
  return _resolve(process.cwd(), 'server/crypto/sponge-native.node');
}

function _getRequire(): NodeRequire {
  if (typeof require !== 'undefined') return require;
  return _createRequire(import.meta.url);
}

function _probeNativeAddon(addonPath: string): boolean {
  try {
    const result = _execFileSync(process.execPath, [
      '-e',
      `process.dlopen(module,${JSON.stringify(addonPath)});process.exit(0);`
    ], { timeout: 5000, stdio: 'pipe' });
    return true;
  } catch {
    return false;
  }
}

let _native: any = null;
let _useNative = false;
try {
  const _nativePath = _resolveNativePath();
  if (_existsSync(_nativePath)) {
    if (_probeNativeAddon(_nativePath)) {
      const _req = _getRequire();
      _native = _req(_nativePath);
      _useNative = true;
      console.log('[sponge] Native Rust N-API backend loaded — TL-Sponge-385');
    } else {
      console.log('[sponge] Native addon probe failed (SIGILL/incompatible) — using TypeScript backend');
    }
  } else {
    console.log('[sponge] Native addon not found — using TypeScript backend');
  }
} catch (e: any) {
  console.log('[sponge] Using TypeScript sponge backend');
  if (process.env.NODE_ENV === 'development') {
    console.log('[sponge] Native load error:', e?.message?.substring(0, 200));
  }
}

export function isNativeAvailable(): boolean {
  return _useNative;
}

export function getNativeModule(): any {
  return _native;
}

export const SPONGE_VERSION = 2;

const STATE_SIZE = 729;
const RATE = 243;
const ROUNDS = 9;
const LANES = 27;
const CHI_BLOCKS = 243;

const PERM: number[] = (() => {
  const p = new Array<number>(STATE_SIZE);
  for (let i = 0; i < STATE_SIZE; i++) {
    p[i] = (i * 376 + 1) % STATE_SIZE;
  }
  return p;
})();

const INV_PERM: Int32Array = (() => {
  const p = new Int32Array(STATE_SIZE);
  for (let i = 0; i < STATE_SIZE; i++) {
    p[(i * 376 + 1) % STATE_SIZE] = i;
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

const RC_INDICES: Int32Array = (() => {
  const idx = new Int32Array(LANES);
  for (let i = 0; i < LANES; i++) idx[i] = i * LANES;
  return idx;
})();

const BW = new Int8Array([-1, 0, 1, -1, 0, 1, -1, 0, 1]);

const TA = new Int8Array([1, -1, 0, 1, -1]);

function tritAdd(a: number, b: number): number {
  return TA[a + b + 2];
}

const _ext = new Int8Array(STATE_SIZE + 26);
const _buf = new Int8Array(STATE_SIZE);

const GF3_MUL: Uint8Array = (() => {
  const t = new Uint8Array(9);
  for (let a = 0; a < 3; a++)
    for (let b = 0; b < 3; b++)
      t[a * 3 + b] = (a * b) % 3;
  return t;
})();

const GF3_ADD: Uint8Array = (() => {
  const t = new Uint8Array(9);
  for (let a = 0; a < 3; a++)
    for (let b = 0; b < 3; b++)
      t[a * 3 + b] = (a + b) % 3;
  return t;
})();

function gf27Mul(a0: number, a1: number, a2: number,
                 b0: number, b1: number, b2: number): [number, number, number] {
  const c0 = GF3_MUL[a0 * 3 + b0];
  const c1 = GF3_ADD[GF3_MUL[a0 * 3 + b1] * 3 + GF3_MUL[a1 * 3 + b0]];
  const c2 = GF3_ADD[GF3_ADD[GF3_MUL[a0 * 3 + b2] * 3 + GF3_MUL[a1 * 3 + b1]] * 3 + GF3_MUL[a2 * 3 + b0]];
  const c3 = GF3_ADD[GF3_MUL[a1 * 3 + b2] * 3 + GF3_MUL[a2 * 3 + b1]];
  const c4 = GF3_MUL[a2 * 3 + b2];

  const r0 = GF3_ADD[c0 * 3 + GF3_MUL[2 * 3 + c3]];
  const r1 = GF3_ADD[GF3_ADD[c1 * 3 + c3] * 3 + GF3_MUL[2 * 3 + c4]];
  const r2 = GF3_ADD[c2 * 3 + c4];

  return [r0, r1, r2];
}

function gf27Pow17(a0: number, a1: number, a2: number): [number, number, number] {
  let [s0, s1, s2] = gf27Mul(a0, a1, a2, a0, a1, a2);
  let [q0, q1, q2] = gf27Mul(s0, s1, s2, s0, s1, s2);
  let [e0, e1, e2] = gf27Mul(q0, q1, q2, q0, q1, q2);
  let [x0, x1, x2] = gf27Mul(e0, e1, e2, e0, e1, e2);
  return gf27Mul(x0, x1, x2, a0, a1, a2);
}

/**
 * Affine-composed chi S-box: S(x) = M · x^17 + c over GF(27).
 * M = circulant [1,1,2] over GF(3)³, det=1, bn=3 (max over GF(3)).
 * c = [1, 0, 2] — eliminates zero fixed point.
 * DP_max = LP_max = 1/9 (preserved). Algebraic degree = 5 (preserved).
 */
function gf27Affine(a0: number, a1: number, a2: number): [number, number, number] {
  const [p0, p1, p2] = gf27Pow17(a0, a1, a2);
  // M · p + c where M = [[1,1,2],[2,1,1],[1,2,1]], c = [1,0,2]
  const r0 = GF3_ADD[GF3_ADD[p0 * 3 + GF3_ADD[p1 * 3 + GF3_MUL[2 * 3 + p2]]] * 3 + 1];
  const r1 = GF3_ADD[GF3_ADD[GF3_MUL[2 * 3 + p0] * 3 + p1] * 3 + p2];
  const r2 = GF3_ADD[GF3_ADD[p0 * 3 + GF3_ADD[GF3_MUL[2 * 3 + p1] * 3 + p2]] * 3 + 2];
  return [r0, r1, r2];
}

const CHI_MAP: Int8Array = (() => {
  const map = new Int8Array(27 * 3);
  for (let idx = 0; idx < 27; idx++) {
    const g0 = idx % 3;
    const g1 = Math.floor(idx / 3) % 3;
    const g2 = Math.floor(idx / 9);
    const [r0, r1, r2] = gf27Affine(g0, g1, g2);
    map[idx * 3]     = r0 - 1;
    map[idx * 3 + 1] = r1 - 1;
    map[idx * 3 + 2] = r2 - 1;
  }
  return map;
})();

function chiLayer(state: Int8Array): void {
  for (let block = 0; block < STATE_SIZE; block += 3) {
    const idx = ((state[block] + 1) + (state[block + 1] + 1) * 3 + (state[block + 2] + 1) * 9) * 3;
    state[block]     = CHI_MAP[idx];
    state[block + 1] = CHI_MAP[idx + 1];
    state[block + 2] = CHI_MAP[idx + 2];
  }
}

function thetaPiRc(state: Int8Array): void {
  _ext.set(state.subarray(STATE_SIZE - 13), 0);
  _ext.set(state, 13);
  _ext.set(state.subarray(0, 13), STATE_SIZE + 13);

  for (let i = 0; i < STATE_SIZE; i++) {
    const ei = i + 13;
    const left  = BW[_ext[ei - 13] + _ext[ei - 7] + _ext[ei - 1] + 4];
    const right = BW[_ext[ei + 1]  + _ext[ei + 7]  + _ext[ei + 13] + 4];
    _buf[i] = BW[left + _ext[ei] + right + 5];
  }

  for (let i = 0; i < STATE_SIZE; i++) {
    state[i] = _buf[INV_PERM[i]];
  }
}

function spongePermutationV1(state: Int8Array): void {
  for (let round = 0; round < ROUNDS; round++) {
    thetaPiRc(state);

    const rc = RC_TABLE[round];
    for (let lane = 0; lane < LANES; lane++) {
      const idx = RC_INDICES[lane];
      state[idx] = BW[state[idx] + rc[lane] + 4];
    }
  }
}

function spongePermutationV2(state: Int8Array): void {
  for (let round = 0; round < ROUNDS; round++) {
    chiLayer(state);
    thetaPiRc(state);

    const rc = RC_TABLE[round];
    for (let lane = 0; lane < LANES; lane++) {
      const idx = RC_INDICES[lane];
      state[idx] = BW[state[idx] + rc[lane] + 4];
    }
  }
}

function spongePermutation(state: Int8Array): void {
  spongePermutationV2(state);
}

function spongePermutationVersioned(state: Int8Array, version: number): void {
  if (version >= 2) {
    spongePermutationV2(state);
  } else {
    spongePermutationV1(state);
  }
}

export function bytesToBalancedTrits(input: Buffer | Uint8Array): Int8Array {
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

function _nativePermV2(state: Int8Array): void {
  const buf = Buffer.from(state.buffer, state.byteOffset, state.byteLength);
  const result: Buffer = _native.spongePermuteV2(buf);
  state.set(new Int8Array(result.buffer, result.byteOffset, result.byteLength));
}

function _nativePermV1(state: Int8Array): void {
  const buf = Buffer.from(state.buffer, state.byteOffset, state.byteLength);
  const result: Buffer = _native.spongePermuteV1(buf);
  state.set(new Int8Array(result.buffer, result.byteOffset, result.byteLength));
}

function spongeAbsorbAndSqueezeVersioned(inputTrits: Int8Array, outputTrits: number, version: number): Int8Array {
  const state = new Int8Array(STATE_SIZE);
  let permFn: (state: Int8Array) => void;
  if (_useNative) {
    permFn = version >= 2 ? _nativePermV2 : _nativePermV1;
  } else {
    permFn = version >= 2 ? spongePermutationV2 : spongePermutationV1;
  }

  let offset = 0;
  while (offset + RATE <= inputTrits.length) {
    for (let i = 0; i < RATE; i++) {
      state[i] = TA[state[i] + inputTrits[offset + i] + 2];
    }
    permFn(state);
    offset += RATE;
  }

  const remaining = inputTrits.length - offset;
  for (let i = 0; i < remaining; i++) {
    state[i] = TA[state[i] + inputTrits[offset + i] + 2];
  }
  if (remaining < RATE) {
    state[remaining] = TA[state[remaining] + 1 + 2];
  }
  permFn(state);

  const output = new Int8Array(outputTrits);
  let written = 0;
  while (written < outputTrits) {
    const take = Math.min(RATE, outputTrits - written);
    output.set(state.subarray(0, take), written);
    written += take;
    if (written < outputTrits) {
      permFn(state);
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

function _nativeBufferToTrits(buf: Buffer): Int8Array {
  const trits = new Int8Array(buf.length);
  for (let i = 0; i < buf.length; i++) trits[i] = buf[i] - 1;
  return trits;
}

export function spongeKeystream(domainInput: Buffer | Uint8Array, outputTritCount: number): Int8Array {
  if (_useNative) {
    const buf = Buffer.isBuffer(domainInput) ? domainInput : Buffer.from(domainInput);
    return _nativeBufferToTrits(_native.spongeKeystream(buf, outputTritCount));
  }
  const inputTrits = bytesToBalancedTrits(domainInput);
  return spongeAbsorbAndSqueezeVersioned(inputTrits, outputTritCount, 2);
}

export function spongeKeystreamV1(domainInput: Buffer | Uint8Array, outputTritCount: number): Int8Array {
  if (_useNative) {
    const buf = Buffer.isBuffer(domainInput) ? domainInput : Buffer.from(domainInput);
    return _nativeBufferToTrits(_native.spongeKeystreamV1(buf, outputTritCount));
  }
  const inputTrits = bytesToBalancedTrits(domainInput);
  return spongeAbsorbAndSqueezeVersioned(inputTrits, outputTritCount, 1);
}

export function spongeHash(input: Buffer | Uint8Array): string {
  if (_useNative) {
    const buf = Buffer.isBuffer(input) ? input : Buffer.from(input);
    return _native.spongeHash(buf);
  }
  const inputTrits = bytesToBalancedTrits(input);
  const outputTrits = spongeAbsorbAndSqueezeVersioned(inputTrits, 243, 2);
  const bytes = tritsToBytes(outputTrits, 49);
  return bytes.toString('hex');
}

export function spongeHashV1(input: Buffer | Uint8Array): string {
  if (_useNative) {
    const buf = Buffer.isBuffer(input) ? input : Buffer.from(input);
    return _native.spongeHashV1(buf);
  }
  const inputTrits = bytesToBalancedTrits(input);
  const outputTrits = spongeAbsorbAndSqueezeVersioned(inputTrits, 243, 1);
  const bytes = tritsToBytes(outputTrits, 49);
  return bytes.toString('hex');
}

export function spongeHashTrits(input: Buffer | Uint8Array): Int8Array {
  const inputTrits = bytesToBalancedTrits(input);
  return spongeAbsorbAndSqueezeVersioned(inputTrits, 243, 2);
}

export class SpongeDuplex {
  private state: Int8Array;
  private buf: Int8Array;
  private bufLen: number;
  private needsFinalize: boolean;
  private permFn: (state: Int8Array) => void;

  constructor(version: number = 2) {
    this.state = new Int8Array(STATE_SIZE);
    this.buf = new Int8Array(RATE);
    this.bufLen = 0;
    this.needsFinalize = true;
    if (_useNative) {
      this.permFn = version >= 2 ? _nativePermV2 : _nativePermV1;
    } else {
      this.permFn = version >= 2 ? spongePermutationV2 : spongePermutationV1;
    }
  }

  absorbTrits(trits: Int8Array): void {
    this._absorbRaw(trits);
  }

  absorb(input: Buffer | Uint8Array): void {
    const inputTrits = bytesToBalancedTrits(input);
    this._absorbRaw(inputTrits);
  }

  private _absorbRaw(inputTrits: Int8Array): void {
    this.needsFinalize = true;
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
          this.state[i] = TA[this.state[i] + this.buf[i] + 2];
        }
        this.permFn(this.state);
        this.bufLen = 0;
      }
    }

    while (offset + RATE <= inputLen) {
      for (let i = 0; i < RATE; i++) {
        this.state[i] = TA[this.state[i] + inputTrits[offset + i] + 2];
      }
      this.permFn(this.state);
      offset += RATE;
    }

    const remaining = inputLen - offset;
    if (remaining > 0) {
      this.buf.set(inputTrits.subarray(offset, offset + remaining), this.bufLen);
      this.bufLen += remaining;
    }
  }

  squeeze(tritCount: number): Int8Array {
    if (this.needsFinalize) {
      for (let i = 0; i < this.bufLen; i++) {
        this.state[i] = TA[this.state[i] + this.buf[i] + 2];
      }
      if (this.bufLen < RATE) {
        this.state[this.bufLen] = TA[this.state[this.bufLen] + 1 + 2];
      }
      this.bufLen = 0;
      this.needsFinalize = false;
      this.permFn(this.state);
    }

    const output = new Int8Array(tritCount);
    let written = 0;
    while (written < tritCount) {
      const take = Math.min(RATE, tritCount - written);
      output.set(this.state.subarray(0, take), written);
      written += take;
      if (written < tritCount) {
        this.permFn(this.state);
      }
    }
    return output;
  }

  reset(): void {
    this.state.fill(0);
    this.buf.fill(0);
    this.bufLen = 0;
    this.needsFinalize = true;
  }
}

export function tritsToHex(trits: Int8Array): string {
  const byteLen = Math.ceil(trits.length / 5);
  const bytes = tritsToBytes(trits, byteLen);
  return bytes.toString('hex');
}

export function tis27Hash(input: Buffer | Uint8Array): string {
  const inputTrits = bytesToBalancedTrits(Buffer.isBuffer(input) ? input : Buffer.from(input));
  const outputTrits = spongeAbsorbAndSqueezeVersioned(inputTrits, 160, 2);
  const bytes = tritsToBytes(outputTrits, 32);
  return bytes.subarray(0, 16).toString('hex');
}

export const TL_SPONGE_HASH_BYTES = 49;
export const TL_SPONGE_HASH_HEX_LEN = 98;
export const TL_SPONGE_HASH_TRITS = 243;
export const TL_SPONGE_SECURITY_BITS = 385;
export const TL_SPONGE_OID = '1.3.6.1.4.1.0.100.3.1';
export const TL_SPONGE_ALGORITHM_NAME = 'tl-sponge';