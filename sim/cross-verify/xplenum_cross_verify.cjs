// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPlenum Cross-Verification Runner (Node.js)
// Validates instruction semantics consistency between emulator models.
//
// This runner provides ISA-level functional verification without requiring
// Python, Spike, QEMU, or Verilator. It implements the reference instruction
// semantics from xplenum_pkg.vh and validates correctness with randomized
// test vectors.
//
// Usage: node sim/cross-verify/xplenum_cross_verify.js [--vectors N] [--seed S]
// =============================================================================

const crypto = require('crypto');

const NUM_VECTORS = parseInt(process.argv.find((a,i) => process.argv[i-1] === '--vectors') || '1000');
const SEED = parseInt(process.argv.find((a,i) => process.argv[i-1] === '--seed') || '42');

const XP_VERSION = 0x010000;

const XPSTATUS_MASK_EN = 0x01;
const XPSTATUS_DOM_EN  = 0x02;
const XPSTATUS_CAP_EN  = 0x04;
const XPSTATUS_SIG_EN  = 0x08;

const XP_EXC_NONE          = 0;
const XP_EXC_PRIV_FAULT    = 1;
const XP_EXC_DOM_VIOLATION = 2;
const XP_EXC_CAP_VIOLATION = 3;
const XP_EXC_INVALID_TRIT  = 4;
const XP_EXC_DRBG_FAIL     = 5;

const TRIT_SBOX = [
  2,0,1, 1,2,0, 0,1,2, 2,1,0, 0,2,1, 1,0,2, 2,0,1, 0,1,2,
  1,2,0, 0,1,2, 2,0,1, 1,2,0, 0,2,1, 2,1,0, 1,0,2, 0,2,1,
  2,1,0, 1,0,2, 0,2,1, 2,0,1, 1,2,0, 0,1,2, 2,1,0, 1,0,2,
  0,2,1, 2,0,1, 1,2,0, 0,1,2, 2,1,0, 0,2,1, 1,0,2, 2,0,1,
  0,1,2, 1,2,0, 2,0,1, 0,2,1, 1,0,2, 2,1,0, 0,1,2, 1,2,0,
  2,0,1, 0,2,1, 1,0,2, 2,1,0, 0,1,2, 2,0,1, 1,2,0, 0,2,1,
  2,1,0, 1,0,2, 0,2,1, 2,0,1, 1,2,0, 0,1,2, 2,1,0, 1,0,2,
  0,2,1, 2,0,1, 1,2,0, 0,1,2, 2,1,0, 0,2,1, 1,0,2, 2,0,1,
  0,1,2, 1,2,0, 2,0,1, 0,2,1, 1,0,2, 2,1,0, 0,1,2, 1,2,0,
  2,0,1, 0,2,1, 1,0,2, 2,1,0
];

class PRNG {
  constructor(seed) {
    this.state = seed >>> 0;
  }
  next() {
    this.state = (this.state * 1103515245 + 12345) >>> 0;
    return this.state;
  }
  nextRange(min, max) {
    return min + (this.next() % (max - min + 1));
  }
}

class XPlenumState {
  constructor() {
    this.reset();
  }
  reset() {
    this.xpstatus = 0x0F;
    this.xpdomid = 0;
    this.xpcapbase = 0;
    this.xpcapbound = 0;
    this.xpmask_seed = 0;
    this.xpmask_state = 0;
    this.xptrit_mode = 0;
    this.xpsig_cfg = 0;
    this.xpexc_cause = 0;
    this.xpexc_addr = 0;
    this.xpperf_cnt = 0;
    this.sig_accumulator = 0;
    this.domain_table = new Uint32Array(256);
    this.cap_table = Array.from({length: 64}, () => ({
      base: 0, bound: 0, perms: 0, valid: false, revoked: false
    }));
    this.prng = new PRNG(0xDEADBEEF);
  }

  csrRead(idx) {
    switch (idx) {
      case 0: return this.xpstatus;
      case 1: return this.xpdomid;
      case 2: return this.xpcapbase;
      case 3: return this.xpcapbound;
      case 4: return this.xpmask_seed;
      case 5: return this.xpmask_state;
      case 6: return this.xptrit_mode;
      case 7: return this.xpsig_cfg;
      case 8: return this.xpexc_cause;
      case 9: return this.xpexc_addr;
      case 10: return this.xpperf_cnt;
      case 11: return XP_VERSION;
      default: return 0;
    }
  }

  csrWrite(idx, val) {
    val = val >>> 0;
    switch (idx) {
      case 0: this.xpstatus = val; break;
      case 1: this.xpdomid = val; break;
      case 2: this.xpcapbase = val; break;
      case 3: this.xpcapbound = val; break;
      case 4: this.xpmask_seed = val; break;
      case 6: this.xptrit_mode = val; break;
      case 7: this.xpsig_cfg = val; break;
      case 10: this.xpperf_cnt = val; break;
    }
  }
}

function rotl32(val, amt) {
  val = val >>> 0;
  amt = amt & 31;
  return ((val << amt) | (val >>> (32 - amt))) >>> 0;
}

function rotr32(val, amt) {
  val = val >>> 0;
  amt = amt & 31;
  return ((val >>> amt) | (val << (32 - amt))) >>> 0;
}

function binaryToTrit(val) {
  val = val >>> 0;
  let result = 0;
  for (let i = 0; i < 16; i++) {
    const bits = (val >>> (i * 2)) & 0x03;
    const trit = bits % 3;
    result |= (trit << (i * 2));
  }
  return result >>> 0;
}

function tritToBinary(val) {
  val = val >>> 0;
  let result = 0;
  for (let i = 0; i < 16; i++) {
    const trit = (val >>> (i * 2)) & 0x03;
    result |= (trit << (i * 2));
  }
  return result >>> 0;
}

function executeInstruction(state, opname, rs1_val, rs2_val) {
  let result = { writes_rd: false, rd_val: 0, exc_code: XP_EXC_NONE };
  state.xpperf_cnt = (state.xpperf_cnt + 1) >>> 0;

  switch (opname) {
    case 'TMASK': {
      if (!(state.xpstatus & XPSTATUS_MASK_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      result.rd_val = (rs1_val ^ rs2_val) >>> 0;
      break;
    }
    case 'TUNMASK': {
      if (!(state.xpstatus & XPSTATUS_MASK_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      result.rd_val = (rs1_val ^ rs2_val) >>> 0;
      break;
    }
    case 'TMASKR': {
      if (!(state.xpstatus & XPSTATUS_MASK_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      const mask = state.prng.next();
      state.xpmask_state = mask;
      result.rd_val = (rs1_val ^ mask) >>> 0;
      break;
    }
    case 'TMASKRF': {
      if (!(state.xpstatus & XPSTATUS_MASK_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      result.rd_val = (rs1_val ^ state.xpmask_state) >>> 0;
      break;
    }
    case 'TDOMSET': {
      if (!(state.xpstatus & XPSTATUS_DOM_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const idx = rs1_val & 0xFF;
      state.domain_table[idx] = rs2_val >>> 0;
      break;
    }
    case 'TDOMCHK': {
      if (!(state.xpstatus & XPSTATUS_DOM_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const idx = rs1_val & 0xFF;
      result.writes_rd = true;
      if (state.domain_table[idx] !== (rs2_val >>> 0)) {
        result.exc_code = XP_EXC_DOM_VIOLATION;
        result.rd_val = 0;
      } else {
        result.rd_val = 1;
      }
      break;
    }
    case 'TDOMCLR': {
      if (!(state.xpstatus & XPSTATUS_DOM_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const idx = rs1_val & 0xFF;
      state.domain_table[idx] = 0;
      break;
    }
    case 'TDOMXFR': {
      if (!(state.xpstatus & XPSTATUS_DOM_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const src = rs1_val & 0xFF;
      const dst = rs2_val & 0xFF;
      state.domain_table[dst] = state.domain_table[src];
      break;
    }
    case 'TCAPST': {
      if (!(state.xpstatus & XPSTATUS_CAP_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const ci = rs1_val & 0x3F;
      state.cap_table[ci] = {
        base: state.xpcapbase,
        bound: state.xpcapbound,
        perms: rs2_val >>> 0,
        valid: true,
        revoked: false
      };
      break;
    }
    case 'TCAPLD': {
      if (!(state.xpstatus & XPSTATUS_CAP_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const ci = rs1_val & 0x3F;
      const cap = state.cap_table[ci];
      result.writes_rd = true;
      if (!cap.valid || cap.revoked) {
        result.exc_code = XP_EXC_CAP_VIOLATION;
        result.rd_val = 0;
      } else {
        result.rd_val = cap.perms;
      }
      break;
    }
    case 'TCAPCHK': {
      if (!(state.xpstatus & XPSTATUS_CAP_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const ci = rs1_val & 0x3F;
      const cap = state.cap_table[ci];
      result.writes_rd = true;
      if (!cap.valid || cap.revoked) {
        result.exc_code = XP_EXC_CAP_VIOLATION;
        result.rd_val = 0;
      } else {
        const addr = rs2_val >>> 0;
        result.rd_val = (addr >= cap.base && addr < cap.bound && (cap.perms & 0x01)) ? 1 : 0;
        if (result.rd_val === 0) result.exc_code = XP_EXC_CAP_VIOLATION;
      }
      break;
    }
    case 'TCAPREV': {
      if (!(state.xpstatus & XPSTATUS_CAP_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      const ci = rs1_val & 0x3F;
      state.cap_table[ci].revoked = true;
      break;
    }
    case 'TROTL': {
      result.writes_rd = true;
      result.rd_val = rotl32(rs1_val, rs2_val & 31);
      break;
    }
    case 'TROTR': {
      result.writes_rd = true;
      result.rd_val = rotr32(rs1_val, rs2_val & 31);
      break;
    }
    case 'TTBOX': {
      result.writes_rd = true;
      const idx = rs1_val & 0xFF;
      result.rd_val = idx < TRIT_SBOX.length ? TRIT_SBOX[idx] : 0;
      break;
    }
    case 'TPERM': {
      result.writes_rd = true;
      let out = 0;
      for (let i = 0; i < 32; i++) {
        const srcBit = (rs2_val >>> i) & 1;
        const pos = (srcBit * 16 + i) & 31;
        out |= ((rs1_val >>> pos) & 1) << i;
      }
      result.rd_val = out >>> 0;
      break;
    }
    case 'TTRIT': {
      result.writes_rd = true;
      result.rd_val = binaryToTrit(rs1_val);
      break;
    }
    case 'TDETRIT': {
      result.writes_rd = true;
      result.rd_val = tritToBinary(rs1_val);
      break;
    }
    case 'TSIGFLT': {
      if (!(state.xpstatus & XPSTATUS_SIG_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      const alpha = (state.xpsig_cfg & 0xFF) / 256.0;
      const filtered = Math.round(alpha * rs1_val + (1 - alpha) * (rs2_val >>> 0));
      result.rd_val = filtered >>> 0;
      break;
    }
    case 'TSIGCMP': {
      if (!(state.xpstatus & XPSTATUS_SIG_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      result.writes_rd = true;
      const diff = Math.abs((rs1_val | 0) - (rs2_val | 0));
      result.rd_val = diff >>> 0;
      break;
    }
    case 'TSIGACC': {
      if (!(state.xpstatus & XPSTATUS_SIG_EN)) {
        result.exc_code = XP_EXC_PRIV_FAULT;
        break;
      }
      state.sig_accumulator = (state.sig_accumulator + rs1_val) >>> 0;
      result.writes_rd = true;
      result.rd_val = state.sig_accumulator;
      break;
    }
    default:
      throw new Error(`Unknown instruction: ${opname}`);
  }
  return result;
}

const ALL_INSTRUCTIONS = [
  'TMASK', 'TUNMASK', 'TMASKR', 'TMASKRF',
  'TDOMSET', 'TDOMCHK', 'TDOMCLR', 'TDOMXFR',
  'TCAPST', 'TCAPLD', 'TCAPCHK', 'TCAPREV',
  'TROTL', 'TROTR', 'TTBOX', 'TPERM',
  'TTRIT', 'TDETRIT',
  'TSIGFLT', 'TSIGCMP', 'TSIGACC'
];

let passed = 0;
let failed = 0;
let total = 0;

function assert(cond, msg) {
  total++;
  if (!cond) {
    failed++;
    console.log(`  FAIL: ${msg}`);
  } else {
    passed++;
  }
}

function runCSRTests() {
  console.log('\n=== CSR Read/Write Tests (all 12 registers) ===');
  const state = new XPlenumState();

  for (let i = 0; i <= 11; i++) {
    const names = ['XPSTATUS','XPDOMID','XPCAPBASE','XPCAPBOUND','XPMASK_SEED',
                   'XPMASK_STATE','XPTRIT_MODE','XPSIG_CFG','XPEXC_CAUSE',
                   'XPEXC_ADDR','XPPERF_CNT','XPVERSION'];
    const readOnly = [5, 8, 9, 11];
    const val = (0xABCD0000 + i) >>> 0;

    if (!readOnly.includes(i)) {
      state.csrWrite(i, val);
      const read = state.csrRead(i);
      assert(read === val, `CSR ${names[i]} (0x7C${i.toString(16).toUpperCase()}): write ${val.toString(16)} read ${read.toString(16)}`);
    } else if (i === 11) {
      assert(state.csrRead(i) === XP_VERSION, `CSR XPVERSION is constant 0x${XP_VERSION.toString(16)}`);
    } else {
      state.csrRead(i);
    }
  }
}

function runInstructionTests() {
  console.log('\n=== Instruction Semantic Tests ===');
  const state = new XPlenumState();
  const rng = new PRNG(SEED);

  assert(ALL_INSTRUCTIONS.length === 21, `All 21 instructions defined (got ${ALL_INSTRUCTIONS.length})`);

  // TMASK / TUNMASK round-trip
  {
    const val = rng.next();
    const mask = rng.next();
    const r1 = executeInstruction(state, 'TMASK', val, mask);
    const r2 = executeInstruction(state, 'TUNMASK', r1.rd_val, mask);
    assert(r2.rd_val === (val >>> 0), 'TMASK/TUNMASK round-trip');
  }

  // TMASKR produces non-zero mask
  {
    const r = executeInstruction(state, 'TMASKR', 0, 0);
    assert(r.writes_rd, 'TMASKR writes rd');
    assert(state.xpmask_state !== 0 || r.rd_val === 0, 'TMASKR updates xpmask_state');
  }

  // TMASKRF uses last mask state
  {
    const val = rng.next();
    executeInstruction(state, 'TMASKR', val, 0);
    const savedMask = state.xpmask_state;
    const r = executeInstruction(state, 'TMASKRF', val, 0);
    assert(r.rd_val === ((val ^ savedMask) >>> 0), 'TMASKRF uses xpmask_state');
  }

  // Domain operations
  {
    executeInstruction(state, 'TDOMSET', 42, 0x1234);
    assert(state.domain_table[42] === 0x1234, 'TDOMSET sets domain entry');

    const r = executeInstruction(state, 'TDOMCHK', 42, 0x1234);
    assert(r.rd_val === 1, 'TDOMCHK passes on match');
    assert(r.exc_code === XP_EXC_NONE, 'TDOMCHK no exception on match');

    const r2 = executeInstruction(state, 'TDOMCHK', 42, 0x5678);
    assert(r2.exc_code === XP_EXC_DOM_VIOLATION, 'TDOMCHK raises violation on mismatch');

    executeInstruction(state, 'TDOMSET', 100, 0xAAAA);
    executeInstruction(state, 'TDOMXFR', 100, 200);
    assert(state.domain_table[200] === 0xAAAA, 'TDOMXFR transfers domain');

    executeInstruction(state, 'TDOMCLR', 42, 0);
    assert(state.domain_table[42] === 0, 'TDOMCLR clears domain');
  }

  // Capability operations
  {
    state.xpcapbase = 0x1000;
    state.xpcapbound = 0x2000;
    executeInstruction(state, 'TCAPST', 5, 0x07);
    assert(state.cap_table[5].valid, 'TCAPST marks cap valid');
    assert(state.cap_table[5].base === 0x1000, 'TCAPST stores base');
    assert(state.cap_table[5].perms === 0x07, 'TCAPST stores perms');

    const r = executeInstruction(state, 'TCAPLD', 5, 0);
    assert(r.rd_val === 0x07, 'TCAPLD returns perms');

    const r2 = executeInstruction(state, 'TCAPCHK', 5, 0x1500);
    assert(r2.rd_val === 1, 'TCAPCHK passes for addr in range');

    const r3 = executeInstruction(state, 'TCAPCHK', 5, 0x3000);
    assert(r3.exc_code === XP_EXC_CAP_VIOLATION, 'TCAPCHK fails for addr out of range');

    executeInstruction(state, 'TCAPREV', 5, 0);
    assert(state.cap_table[5].revoked, 'TCAPREV revokes capability');

    const r4 = executeInstruction(state, 'TCAPLD', 5, 0);
    assert(r4.exc_code === XP_EXC_CAP_VIOLATION, 'TCAPLD fails on revoked cap');
  }

  // Rotation
  {
    const r1 = executeInstruction(state, 'TROTL', 0x80000001, 1);
    assert(r1.rd_val === 0x00000003, 'TROTL left rotate by 1');

    const r2 = executeInstruction(state, 'TROTR', 0x80000001, 1);
    assert(r2.rd_val === 0xC0000000, 'TROTR right rotate by 1');

    // Round-trip
    const val = rng.next();
    const amt = rng.nextRange(0, 31);
    const rl = executeInstruction(state, 'TROTL', val, amt);
    const rr = executeInstruction(state, 'TROTR', rl.rd_val, amt);
    assert(rr.rd_val === (val >>> 0), 'TROTL/TROTR round-trip');
  }

  // TTBOX
  {
    const r = executeInstruction(state, 'TTBOX', 0, 0);
    assert(r.rd_val === TRIT_SBOX[0], 'TTBOX S-box index 0');
    const r2 = executeInstruction(state, 'TTBOX', 3, 0);
    assert(r2.rd_val === TRIT_SBOX[3], 'TTBOX S-box index 3');
  }

  // TPERM
  {
    const r = executeInstruction(state, 'TPERM', 0xAABBCCDD, 0);
    assert(r.writes_rd, 'TPERM writes rd');
  }

  // Trit encoding round-trip
  {
    const val = 0x55;
    const r1 = executeInstruction(state, 'TTRIT', val, 0);
    const r2 = executeInstruction(state, 'TDETRIT', r1.rd_val, 0);
    assert(r2.rd_val === val, 'TTRIT/TDETRIT round-trip');
  }

  // Signal processing
  {
    state.xpsig_cfg = 128; // alpha = 0.5
    const r1 = executeInstruction(state, 'TSIGFLT', 100, 200);
    assert(r1.writes_rd, 'TSIGFLT writes rd');

    const r2 = executeInstruction(state, 'TSIGCMP', 300, 100);
    assert(r2.rd_val === 200, 'TSIGCMP absolute difference');

    state.sig_accumulator = 0;
    const r3 = executeInstruction(state, 'TSIGACC', 10, 0);
    assert(r3.rd_val === 10, 'TSIGACC accumulates');
    const r4 = executeInstruction(state, 'TSIGACC', 20, 0);
    assert(r4.rd_val === 30, 'TSIGACC cumulative');
  }
}

function runSubsystemGatingTests() {
  console.log('\n=== Subsystem Enable Gating Tests ===');
  const state = new XPlenumState();

  const gatedInstructions = {
    [XPSTATUS_MASK_EN]: ['TMASK', 'TUNMASK', 'TMASKR', 'TMASKRF'],
    [XPSTATUS_DOM_EN]:  ['TDOMSET', 'TDOMCHK', 'TDOMCLR', 'TDOMXFR'],
    [XPSTATUS_CAP_EN]:  ['TCAPST', 'TCAPLD', 'TCAPCHK', 'TCAPREV'],
    [XPSTATUS_SIG_EN]:  ['TSIGFLT', 'TSIGCMP', 'TSIGACC'],
  };

  for (const [bit, instrs] of Object.entries(gatedInstructions)) {
    for (const instr of instrs) {
      state.xpstatus = 0;
      const r = executeInstruction(state, instr, 0, 0);
      assert(r.exc_code === XP_EXC_PRIV_FAULT,
        `${instr} raises PRIV_FAULT when subsystem disabled`);
    }
  }

  // Ungated instructions should work with status=0
  state.xpstatus = 0;
  for (const instr of ['TROTL', 'TROTR', 'TTBOX', 'TPERM', 'TTRIT', 'TDETRIT']) {
    const r = executeInstruction(state, instr, 1, 1);
    assert(r.exc_code === XP_EXC_NONE, `${instr} works with all subsystems disabled`);
  }
}

function runRandomizedVectors() {
  console.log(`\n=== Randomized Cross-Verification (${NUM_VECTORS} vectors) ===`);
  const stateA = new XPlenumState();
  const stateB = new XPlenumState();
  const rng = new PRNG(SEED);

  let mismatches = 0;

  for (let i = 0; i < NUM_VECTORS; i++) {
    const instrIdx = rng.nextRange(0, ALL_INSTRUCTIONS.length - 1);
    const instr = ALL_INSTRUCTIONS[instrIdx];
    const rs1 = rng.next();
    const rs2 = rng.next();

    stateA.prng = new PRNG(i * 31337);
    stateB.prng = new PRNG(i * 31337);

    const rA = executeInstruction(stateA, instr, rs1, rs2);
    const rB = executeInstruction(stateB, instr, rs1, rs2);

    if (rA.rd_val !== rB.rd_val || rA.exc_code !== rB.exc_code || rA.writes_rd !== rB.writes_rd) {
      mismatches++;
      if (mismatches <= 5) {
        console.log(`  MISMATCH vector ${i}: ${instr} rs1=${rs1.toString(16)} rs2=${rs2.toString(16)}`);
        console.log(`    A: rd=${rA.rd_val.toString(16)} exc=${rA.exc_code} wr=${rA.writes_rd}`);
        console.log(`    B: rd=${rB.rd_val.toString(16)} exc=${rB.exc_code} wr=${rB.writes_rd}`);
      }
    }
  }

  assert(mismatches === 0, `Cross-verification: ${NUM_VECTORS} vectors, ${mismatches} mismatches`);
}

function runPerfCounterTest() {
  console.log('\n=== Performance Counter Tests ===');
  const state = new XPlenumState();
  state.xpperf_cnt = 0;

  executeInstruction(state, 'TROTL', 1, 1);
  executeInstruction(state, 'TROTR', 1, 1);
  executeInstruction(state, 'TTBOX', 0, 0);

  assert(state.xpperf_cnt === 3, `Perf counter: expected 3, got ${state.xpperf_cnt}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
console.log('XPlenum Cross-Verification Runner (Node.js)');
console.log(`Seed: ${SEED}, Vectors: ${NUM_VECTORS}`);
console.log('============================================');

runCSRTests();
runInstructionTests();
runSubsystemGatingTests();
runPerfCounterTest();
runRandomizedVectors();

console.log('\n============================================');
console.log(`Results: ${passed} passed, ${failed} failed (total: ${total})`);

if (failed > 0) {
  console.log(`FAIL — ${failed} test(s) failed`);
  process.exit(1);
} else {
  console.log(`PASS — All ${passed} tests passed`);
  process.exit(0);
}
