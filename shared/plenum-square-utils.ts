/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * # Plenum Square Utilities
 *
 * Pure, lightweight helpers for the Plenum Square.
 * No runtime computation beyond basic static arithmetic.
 * All functions are deterministic and side-effect-free.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  PLENUM_SQUARE_MATRIX,
  PLENUM_MAGIC_CONSTANT,
} from './plenum-square';

/**
 * Returns a flattened version of the Plenum Square matrix.
 * Useful for seeding, indexing, or mixing into crypto round constants.
 *
 * Result: [111, 14, 208, 208, 111, 14, 14, 208, 111]
 */
export function getPlenumSquareFlattened(): readonly number[] {
  return PLENUM_SQUARE_MATRIX.flat();
}

/**
 * Returns one of the three cyclic permutations of the matrix rows.
 * The Plenum Square matrix is circulant — each row is a cyclic shift.
 *
 * @param shift 0, 1, or 2 — corresponds to ternary gauge-like rotation
 */
export function getPlenumSquareCyclic(shift: 0 | 1 | 2 = 0): readonly (readonly number[])[] {
  const rows = PLENUM_SQUARE_MATRIX;
  return [
    rows[(0 + shift) % 3],
    rows[(1 + shift) % 3],
    rows[(2 + shift) % 3],
  ] as const;
}

/**
 * Returns a ternary-weighted value from the Plenum Square matrix row.
 *
 * @param tritIndex 0 | 1 | 2 — selects weight from the canonical row [111, 14, 208]
 *   - 0 → 111 (balance center, Amun.Ra void anchor)
 *   - 1 → 14  (esoteric π, future-initiating)
 *   - 2 → 208 (cosmic accumulation, past-heavy)
 */
export function getTernaryPlenumSquareWeight(tritIndex: 0 | 1 | 2): number {
  return PLENUM_SQUARE_MATRIX[0][tritIndex];
}

/**
 * Validates that a 3×3 matrix has the Plenum Square magic property.
 * All rows, columns, and both diagonals must sum to 333.
 *
 * Pure function — for tests or compile-time verification.
 */
export function isPlenumSquareMagic(matrix: readonly (readonly number[])[]): boolean {
  if (matrix.length !== 3 || matrix.some(row => row.length !== 3)) return false;

  const sums = [
    matrix[0][0] + matrix[0][1] + matrix[0][2],
    matrix[1][0] + matrix[1][1] + matrix[1][2],
    matrix[2][0] + matrix[2][1] + matrix[2][2],
    matrix[0][0] + matrix[1][0] + matrix[2][0],
    matrix[0][1] + matrix[1][1] + matrix[2][1],
    matrix[0][2] + matrix[1][2] + matrix[2][2],
    matrix[0][0] + matrix[1][1] + matrix[2][2],
    matrix[0][2] + matrix[1][1] + matrix[2][0],
  ];

  return sums.every(sum => sum === PLENUM_MAGIC_CONSTANT);
}

/**
 * Validates that a matrix is circulant — each row is a cyclic shift of the first.
 */
export function isCirculant(matrix: readonly (readonly number[])[]): boolean {
  if (matrix.length !== 3 || matrix.some(row => row.length !== 3)) return false;

  const row0 = matrix[0];
  for (let r = 1; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      if (matrix[r][c] !== row0[(c - r + 3) % 3]) return false;
    }
  }
  return true;
}
