const STATE_SIZE = 729;
const RATE = 243;
const ROUNDS = 9;
const LANES = 27;

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

function gf27Affine(a0: number, a1: number, a2: number): [number, number, number] {
  const [p0, p1, p2] = gf27Pow17(a0, a1, a2);
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

const _ext = new Int8Array(STATE_SIZE + 26);
const _buf = new Int8Array(STATE_SIZE);

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

function bytesToBalancedTrits(input: Uint8Array): Int8Array {
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
      state[i] = TA[state[i] + inputTrits[offset + i] + 2];
    }
    spongePermutationV2(state);
    offset += RATE;
  }
  const remaining = inputTrits.length - offset;
  for (let i = 0; i < remaining; i++) {
    state[i] = TA[state[i] + inputTrits[offset + i] + 2];
  }
  if (remaining < RATE) {
    state[remaining] = TA[state[remaining] + 1 + 2];
  }
  spongePermutationV2(state);
  const output = new Int8Array(outputTrits);
  let written = 0;
  while (written < outputTrits) {
    const take = Math.min(RATE, outputTrits - written);
    output.set(state.subarray(0, take), written);
    written += take;
    if (written < outputTrits) {
      spongePermutationV2(state);
    }
  }
  return output;
}

function tritsToBytes(trits: Int8Array, byteLen: number): Uint8Array {
  const out = new Uint8Array(byteLen);
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

function bytesToHex(bytes: Uint8Array): string {
  let hex = '';
  for (const b of bytes) {
    hex += b.toString(16).padStart(2, '0');
  }
  return hex;
}

export function tis27Hash(input: string): string {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(input);
  const inputTrits = bytesToBalancedTrits(bytes);
  const outputTrits = spongeAbsorbAndSqueeze(inputTrits, 160);
  const outBytes = tritsToBytes(outputTrits, 32);
  return bytesToHex(outBytes.subarray(0, 16));
}
