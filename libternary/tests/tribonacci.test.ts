/**
 * Tribonacci Module Tests
 *
 * Validates Tribonacci sequence values, τ verification, resonance mass,
 * QGN decoherence, and derived constants.
 *
 * Run: npx tsx libternary/tests/tribonacci.test.ts
 */

import {
  TAU,
  TAU_POWERS,
  DERIVED_CONSTANTS,
  VM_CONSTANTS,
  tribonacci,
  tribonacciSequence,
  resonanceMass,
  qgnDecoherence,
  verifyTau,
} from '../src/tribonacci';

let passed = 0;
let failed = 0;

function assert(condition: boolean, label: string) {
  if (condition) {
    passed++;
  } else {
    failed++;
    console.error(`  FAIL: ${label}`);
  }
}

function assertEquals(actual: any, expected: any, label: string) {
  const ok = actual === expected;
  if (!ok) {
    console.error(`  FAIL: ${label} — expected ${expected}, got ${actual}`);
    failed++;
  } else {
    passed++;
  }
}

function assertApprox(actual: number, expected: number, epsilon: number, label: string) {
  const ok = Math.abs(actual - expected) < epsilon;
  if (!ok) {
    console.error(`  FAIL: ${label} — expected ~${expected}, got ${actual} (diff: ${Math.abs(actual - expected)})`);
    failed++;
  } else {
    passed++;
  }
}

console.log('=== τ Verification ===');

const tau = verifyTau();
assert(tau.valid, 'τ³ = τ² + τ + 1 holds');
assert(tau.error < 1e-10, `τ equation error < 1e-10 (actual: ${tau.error})`);
assertApprox(TAU, 1.8392867552141612, 1e-15, 'TAU value correct');

console.log('=== Tribonacci Sequence T(0)–T(15) ===');

const knownValues: [number, number][] = [
  [0, 0],
  [1, 0],
  [2, 1],
  [3, 1],
  [4, 2],
  [5, 4],
  [6, 7],
  [7, 13],
  [8, 24],
  [9, 44],
  [10, 81],
  [11, 149],
  [12, 274],
  [13, 504],
  [14, 927],
  [15, 1705],
];

for (const [n, expected] of knownValues) {
  assertEquals(tribonacci(n), expected, `T(${n}) = ${expected}`);
}

console.log('=== Tribonacci Sequence Generator ===');

const seq = tribonacciSequence(10);
assertEquals(seq.length, 11, 'tribonacciSequence(10) has 11 elements');
assertEquals(seq[0], 0, 'seq[0] = 0');
assertEquals(seq[1], 0, 'seq[1] = 0');
assertEquals(seq[2], 1, 'seq[2] = 1');
assertEquals(seq[7], 13, 'seq[7] = 13');
assertEquals(seq[10], 81, 'seq[10] = 81');

const emptySeq = tribonacciSequence(-1);
assertEquals(emptySeq.length, 0, 'tribonacciSequence(-1) is empty');

console.log('=== Negative Index Error ===');

let threw = false;
try { tribonacci(-1); } catch { threw = true; }
assert(threw, 'tribonacci(-1) throws');

console.log('=== Resonance Mass ===');

const r1 = resonanceMass(1);
assertEquals(r1.n, 1, 'resonanceMass(1).n = 1');
assertEquals(r1.T_index, 7, 'resonanceMass(1) uses T(7)');
assertEquals(r1.T_value, 13, 'T(7) = 13');
assertApprox(r1.mass_TeV, 1.30, 0.01, 'M₁ ≈ 1.30 TeV');

const r2 = resonanceMass(2);
assertEquals(r2.T_index, 8, 'resonanceMass(2) uses T(8)');
assertEquals(r2.T_value, 24, 'T(8) = 24');
assertApprox(r2.mass_TeV, 2.40, 0.01, 'M₂ ≈ 2.40 TeV');

const r3 = resonanceMass(3);
assertEquals(r3.T_index, 9, 'resonanceMass(3) uses T(9)');
assertApprox(r3.mass_TeV, 4.40, 0.01, 'M₃ ≈ 4.40 TeV');

console.log('=== QGN Decoherence ===');

const qgn = qgnDecoherence(1000);
assert(qgn.E_GeV === 1000, 'QGN input energy = 1000 GeV');
assertApprox(qgn.tau_minus_5, TAU ** -5, 1e-10, 'τ⁻⁵ correct');
assert(qgn.gamma_GeV > 0, 'decoherence rate is positive');
assert(qgn.gamma_GeV < 1, 'decoherence rate < 1 GeV at 1 TeV');

const qgn2 = qgnDecoherence(2000);
assertApprox(qgn2.gamma_GeV / qgn.gamma_GeV, 4, 0.001, 'Γ scales as E² (2x energy → 4x rate)');

console.log('=== TAU_POWERS ===');

assertApprox(TAU_POWERS.TAU_2, TAU ** 2, 1e-10, 'TAU_2 = τ²');
assertApprox(TAU_POWERS.TAU_3, TAU ** 3, 1e-10, 'TAU_3 = τ³');
assertApprox(TAU_POWERS.TAU_5, TAU ** 5, 1e-10, 'TAU_5 = τ⁵');
assertApprox(TAU_POWERS.TAU_7, TAU ** 7, 1e-10, 'TAU_7 = τ⁷');
assertApprox(TAU_POWERS.TAU_13, TAU ** 13, 1e-6, 'TAU_13 = τ¹³');

console.log('=== DERIVED_CONSTANTS ===');

assertApprox(DERIVED_CONSTANTS.LOG2_3, Math.log2(3), 1e-10, 'log₂(3) ≈ 1.585');
assertApprox(DERIVED_CONSTANTS.DENSITY_ADVANTAGE_PCT, 58.496, 0.01, '59% density advantage');
assertApprox(DERIVED_CONSTANTS.S_INST, 2 * TAU_POWERS.TAU_7, 1e-6, 'S_inst = 2τ⁷');
assertApprox(DERIVED_CONSTANTS.DELTA_THETA_DEG, 9 / TAU_POWERS.TAU_5, 1e-6, 'Δθ = 9/τ⁵');
assertEquals(DERIVED_CONSTANTS.D4_DIM, 28, 'D₄ dimension = 28');
assertEquals(DERIVED_CONSTANTS.FUNDAMENTAL_PERIOD_DAYS, 1152, 'fundamental period = 1152');

console.log('=== VM_CONSTANTS ===');

assertEquals(VM_CONSTANTS.REGISTER_COUNT, 27, 'VM register count = 27 (3³)');
assertEquals(VM_CONSTANTS.DEFAULT_STACK_SIZE, 4096, 'default stack = 4096');
assertEquals(VM_CONSTANTS.HASH_ROUNDS, 13, 'hash rounds = 13 (T(7))');
assert(VM_CONSTANTS.HASH_SEED > 0, 'hash seed is positive');
assert(VM_CONSTANTS.HASH_MIX > 0, 'hash mix is positive');
assert(VM_CONSTANTS.GC_THRESHOLD_RATIO > 0 && VM_CONSTANTS.GC_THRESHOLD_RATIO < 1, 'GC threshold ratio in (0,1)');

console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
process.exit(failed > 0 ? 1 : 0);
