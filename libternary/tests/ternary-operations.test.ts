/**
 * GF(3) Arithmetic Tests
 *
 * Validates the full 3x3 addition and multiplication tables for the
 * balanced ternary representation A = {-1, 0, +1}.
 *
 * Run: npx tsx libternary/tests/ternary-operations.test.ts
 */

import { TritA } from '../src/ternary-types';
import {
  ternaryAdd,
  ternaryMultiply,
  ternaryXor,
  calculateInformationDensity,
} from '../src/ternary-operations';

let passed = 0;
let failed = 0;

function assertEquals(actual: any, expected: any, label: string) {
  const ok = actual === expected;
  if (!ok) {
    console.error(`  FAIL: ${label} -- expected ${expected}, got ${actual}`);
    failed++;
  } else {
    passed++;
  }
}

console.log('=== GF(3) Addition Table (9 cases) ===');

const addTable: [TritA, TritA, number][] = [
  [-1, -1,  1],
  [-1,  0, -1],
  [-1,  1,  0],
  [ 0, -1, -1],
  [ 0,  0,  0],
  [ 0,  1,  1],
  [ 1, -1,  0],
  [ 1,  0,  1],
  [ 1,  1, -1],
];

for (const [a, b, expected] of addTable) {
  const result = ternaryAdd(a, b);
  assertEquals(result.result, expected, `(${a}) + (${b}) = ${expected}`);
}

console.log('=== GF(3) Multiplication Table (9 cases) ===');

const mulTable: [TritA, TritA, number][] = [
  [-1, -1,  1],
  [-1,  0,  0],
  [-1,  1, -1],
  [ 0, -1,  0],
  [ 0,  0,  0],
  [ 0,  1,  0],
  [ 1, -1, -1],
  [ 1,  0,  0],
  [ 1,  1,  1],
];

for (const [a, b, expected] of mulTable) {
  const result = ternaryMultiply(a, b);
  assertEquals(result.result, expected, `(${a}) * (${b}) = ${expected}`);
}

console.log('=== Ternary XOR Table (Kleene min) ===');

const xorTable: [TritA, TritA, number][] = [
  [-1, -1, -1],
  [-1,  0, -1],
  [-1,  1, -1],
  [ 0, -1, -1],
  [ 0,  0,  0],
  [ 0,  1,  0],
  [ 1, -1, -1],
  [ 1,  0,  0],
  [ 1,  1,  1],
];

for (const [a, b, expected] of xorTable) {
  const result = ternaryXor(a, b);
  assertEquals(result.result, expected, `(${a}) XOR (${b}) = ${expected}`);
}

console.log('=== Algebraic Properties ===');

const trits: TritA[] = [-1, 0, 1];

for (const a of trits) {
  assertEquals(ternaryAdd(a, 0 as TritA).result, a, `additive identity: ${a} + 0 = ${a}`);
  assertEquals(ternaryMultiply(a, 1 as TritA).result, a, `multiplicative identity: ${a} * 1 = ${a}`);
  assertEquals(ternaryMultiply(a, 0 as TritA).result, 0, `zero annihilator: ${a} * 0 = 0`);
}

assertEquals(ternaryAdd(-1 as TritA, 1 as TritA).result, 0, 'additive inverse: -1 + 1 = 0');
assertEquals(ternaryAdd(1 as TritA, -1 as TritA).result, 0, 'additive inverse: 1 + (-1) = 0');

for (const a of trits) {
  for (const b of trits) {
    assertEquals(
      ternaryAdd(a, b).result,
      ternaryAdd(b, a).result,
      `add commutativity: ${a}+${b} = ${b}+${a}`
    );
    assertEquals(
      ternaryMultiply(a, b).result,
      ternaryMultiply(b, a).result,
      `mul commutativity: ${a}*${b} = ${b}*${a}`
    );
  }
}

console.log('=== Information Density ===');

const density1 = calculateInformationDensity(1);
assertEquals(density1.trits, 1, 'density: trits = 1');
const bitsExpected = Math.log2(3);
const approxBits = Math.abs(density1.bitsEquivalent - bitsExpected);
if (approxBits < 0.01) { passed++; } else { failed++; console.error(`  FAIL: 1 trit ~${bitsExpected} bits (got ${density1.bitsEquivalent})`); }

console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
process.exit(failed > 0 ? 1 : 0);
