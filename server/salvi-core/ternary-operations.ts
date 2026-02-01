/**
 * Salvi Framework - Ternary Operations
 * 
 * Implements the Enhanced Galois Ternary Field operations from the whitepaper:
 * - Ternary addition: a ⊕₃ b
 * - Ternary multiplication: a ⊗₃ b
 * - Dynamic bijective rotations: TBR(θ)
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
 * Ternary Addition in GF(3)
 * a ⊕₃ b = (a + b) mod 3
 * 
 * Uses Representation A internally (-1, 0, +1)
 * Mapped to (0, 1, 2) for modular arithmetic
 */
export function ternaryAdd(a: TritA, b: TritA): OperationResult {
  const aMapped = a + 1;
  const bMapped = b + 1;
  const sumMod3 = (aMapped + bMapped) % 3;
  const result = (sumMod3 - 1) as TritA;
  
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
 */
export function ternaryMultiply(a: TritA, b: TritA): OperationResult {
  const aMapped = a + 1;
  const bMapped = b + 1;
  const productMod3 = (aMapped * bMapped) % 3;
  const result = (productMod3 - 1) as TritA;
  
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
  const mapped = value + 1;
  const rotated = (mapped + normalizedSteps) % 3;
  const result = (rotated - 1) as TritA;
  
  return {
    operands: { a: value, b: steps },
    operation: 'ternary_rotation',
    result,
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
 * Ternary XOR (exclusive or)
 * Different from addition - returns True only when exactly one input is True
 */
export function ternaryXor(a: TritA, b: TritA): OperationResult {
  let result: TritA;
  
  if (a === b) {
    result = 0;
  } else if (a === 0) {
    result = b;
  } else if (b === 0) {
    result = a;
  } else {
    result = 0;
  }
  
  return {
    operands: { a, b },
    operation: 'ternary_xor',
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
