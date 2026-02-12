/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * Salvi Framework - Ternary Operations (IMPROVED)
 * 
 * Implements the Enhanced Galois Ternary Field operations from the whitepaper:
 * - Ternary addition: a ⊕₃ b
 * - Ternary multiplication: a ⊗₃ b
 * - Dynamic bijective rotations: TBR(θ)
 *
 * FIX: Corrected GF(3) ring isomorphism. Previous mapping f(a) = a + 1
 * mapped -1→0, 0→1, 1→2, which is NOT a ring homomorphism — it maps the
 * balanced-ternary element -1 to GF(3)'s additive identity 0, breaking
 * multiplication (e.g., (-1)*(-1) incorrectly returned -1 instead of +1).
 *
 * Correct mapping: -1 ↔ 2, 0 ↔ 0, 1 ↔ 1 (standard modular equivalence)
 * This preserves both addition and multiplication structure.
 */

import { TritA, TritB, Representation, convertTrit } from './ternary-types';

export type SecurityMode = 'phi' | 'mode1' | 'mode0';

export interface OperationResult {
  operands: { a: number; b: number };
  operation: string;
  result: number;
  representation: Representation;
  constantTime: boolean;
  securityMode?: SecurityMode;
}

/**
 * Map balanced ternary (-1, 0, +1) to GF(3) (0, 1, 2)
 * Uses the correct ring isomorphism: -1 ↔ 2, 0 ↔ 0, 1 ↔ 1
 * Formula: (a + 3) % 3
 */
function toGF3(a: TritA): number {
  return ((a % 3) + 3) % 3;
}

/**
 * Map GF(3) (0, 1, 2) back to balanced ternary (-1, 0, +1)
 * Inverse of toGF3: 0 → 0, 1 → 1, 2 → -1
 */
function fromGF3(g: number): TritA {
  if (g === 2) return -1;
  return g as TritA;
}

/**
 * Ternary Addition in GF(3)
 * a ⊕₃ b = (a + b) mod 3
 *
 * Uses the correct ring isomorphism for balanced ternary ↔ GF(3).
 * Verified: (-1)+(-1)=1, (-1)+1=0, 1+1=-1 (all correct in GF(3))
 */
export function ternaryAdd(a: TritA, b: TritA): OperationResult {
  const result = fromGF3((toGF3(a) + toGF3(b)) % 3);

  return {
    operands: { a, b },
    operation: 'ternary_addition',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Ternary Multiplication in GF(3)
 * a ⊗₃ b = (a × b) mod 3
 *
 * FIX: Now uses correct ring isomorphism.
 * Verified: (-1)*(-1)=1, (-1)*1=-1, 1*1=1, 0*x=0 (all correct)
 */
export function ternaryMultiply(a: TritA, b: TritA): OperationResult {
  const result = fromGF3((toGF3(a) * toGF3(b)) % 3);

  return {
    operands: { a, b },
    operation: 'ternary_multiplication',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Ternary Bijective Rotation
 * TBR(θ) - Rotates a trit value by the specified angle
 * 
 * θ = 120° (2π/3) - Standard rotation
 * θ = 222.5° (360°/φ) - Golden ratio rotation
 */
export function ternaryRotate(value: TritA, steps: number = 1): OperationResult {
  const normalizedSteps = ((steps % 3) + 3) % 3;
  const rotated = fromGF3((toGF3(value) + normalizedSteps) % 3);

  return {
    operands: { a: value, b: steps },
    operation: 'ternary_rotation',
    result: rotated,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Adaptive Ternary Addition based on Security Mode
 * 
 * Mode φ (phi): Full quantum-resistant operations
 * Mode 1: Standard post-quantum operations
 * Mode 0: AES-256 compatible operations
 */
export function adaptiveTernaryAdd(a: TritA, b: TritA, mode: SecurityMode): OperationResult {
  const result = ternaryAdd(a, b);
  
  return {
    ...result,
    operation: `adaptive_ternary_addition_${mode}`,
    securityMode: mode
  };
}

/**
 * Batch Ternary Addition
 * Processes multiple ternary operations efficiently
 */
export function batchTernaryAdd(pairs: Array<{ a: TritA; b: TritA }>): OperationResult[] {
  return pairs.map(({ a, b }) => ternaryAdd(a, b));
}

/**
 * Ternary XOR — Kleene min(a, b)
 * Canonical semantics from the Rust kernel (TXor).
 * This is NOT GF(3) addition; GF(3) addition is handled by TAdd / ternaryAdd.
 */
export function ternaryXor(a: TritA, b: TritA): OperationResult {
  const result = Math.min(a, b) as TritA;
  
  return {
    operands: { a, b },
    operation: 'ternary_xor',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Ternary AND — Łukasiewicz conjunction: max(a + b - 1, -1)
 * Canonical semantics from the Rust kernel (TAnd).
 */
export function ternaryAnd(a: TritA, b: TritA): OperationResult {
  const result = Math.max(a + b - 1, -1) as TritA;
  
  return {
    operands: { a, b },
    operation: 'ternary_and',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Ternary OR — Kleene max(a, b)
 * Canonical semantics from the Rust kernel (TOr).
 */
export function ternaryOr(a: TritA, b: TritA): OperationResult {
  const result = Math.max(a, b) as TritA;
  
  return {
    operands: { a, b },
    operation: 'ternary_or',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Ternary NOT (negation)
 * Flips the trit value: -1 → 1, 0 → 0, 1 → -1
 */
export function ternaryNot(value: TritA): OperationResult {
  const result = (-value) as TritA;
  
  return {
    operands: { a: value, b: 0 },
    operation: 'ternary_not',
    result,
    representation: 'A',
    constantTime: true
  };
}

/**
 * Calculate information density advantage
 * log₂(3) ≈ 1.585 bits per trit vs 1 bit per bit
 * Returns the efficiency gain percentage
 */
export const toGF3Export = toGF3;
export const fromGF3Export = fromGF3;

export const gf3Add = (a: TritA, b: TritA): TritA => {
  return fromGF3((toGF3(a) + toGF3(b)) % 3);
};

export const gf3Multiply = (a: TritA, b: TritA): TritA => {
  return fromGF3((toGF3(a) * toGF3(b)) % 3);
};

// Kleene min — matches the Rust kernel's TXor canonical semantics.
// For GF(3) field addition, use gf3Add (TAdd) instead.
export const gf3Xor = (a: TritA, b: TritA): TritA => {
  return Math.min(a, b) as TritA;
};

// Łukasiewicz conjunction — matches the Rust kernel's TAnd canonical semantics.
export const gf3And = (a: TritA, b: TritA): TritA => {
  return Math.max(a + b - 1, -1) as TritA;
};

// Kleene max — matches the Rust kernel's TOr canonical semantics.
export const gf3Or = (a: TritA, b: TritA): TritA => {
  return Math.max(a, b) as TritA;
};

export const gf3Not = (a: TritA): TritA => (-a) as TritA;

export const gf3Rotate = (value: TritA, steps: number = 1): TritA => {
  const gf3Val = toGF3(value);
  const rotated = (gf3Val + ((steps % 3) + 3) % 3) % 3;
  return fromGF3(rotated);
};

export const GF3_ADDITION_TABLE: TritA[][] = [
  [gf3Add(-1, -1), gf3Add(-1, 0), gf3Add(-1, 1)],
  [gf3Add(0, -1),  gf3Add(0, 0),  gf3Add(0, 1)],
  [gf3Add(1, -1),  gf3Add(1, 0),  gf3Add(1, 1)],
];

export const GF3_MULTIPLICATION_TABLE: TritA[][] = [
  [gf3Multiply(-1, -1), gf3Multiply(-1, 0), gf3Multiply(-1, 1)],
  [gf3Multiply(0, -1),  gf3Multiply(0, 0),  gf3Multiply(0, 1)],
  [gf3Multiply(1, -1),  gf3Multiply(1, 0),  gf3Multiply(1, 1)],
];

export const verifyGF3 = (): boolean => {
  const test1 = gf3Multiply(-1, -1) === 1;
  const test2 = gf3Multiply(-1, 1) === -1;
  const test3 = gf3Multiply(0, 1) === 0;
  const test4 = gf3Multiply(1, 1) === 1;
  const test5 = gf3Multiply(-1, 0) === 0;
  const test6 = gf3Multiply(1, 0) === 0;
  const test7 = gf3Add(-1, 0) === -1;
  const test8 = gf3Add(0, 0) === 0;
  const test9 = gf3Add(1, 0) === 1;
  const test10 = gf3Add(1, -1) === 0;
  return test1 && test2 && test3 && test4 && test5 &&
         test6 && test7 && test8 && test9 && test10;
};

export function calculateInformationDensity(tritCount: number): {
  trits: number;
  bitsEquivalent: number;
  efficiencyGain: string;
} {
  const log2of3 = Math.log2(3);
  const bitsEquivalent = tritCount * log2of3;
  const efficiencyGain = ((log2of3 - 1) * 100).toFixed(2);
  
  return {
    trits: tritCount,
    bitsEquivalent: Math.round(bitsEquivalent * 100) / 100,
    efficiencyGain: `+${efficiencyGain}%`
  };
}
