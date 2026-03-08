// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Repunit Checksum — TypeScript Implementation
// Mirrors: ternary-math/src/repunit_checksum.rs
//
// Lightweight 6-trit integrity check for 27-trit classification addresses.
// Uses mod R₆ = 364 = 111111₃ (full ternary circle).
// All arithmetic in GF(3) — no domain crossing.

/** Full ternary circle: R₆ = (3⁶ - 1) / 2 = 364 */
export const REPUNIT_R6 = 364;

/** Checksum output width: 6 trits (digits in 364 base-3) */
export const CHECKSUM_TRIT_COUNT = 6;

/**
 * Compute the repunit checksum of a 27-trit classification address.
 *
 * @param trits - Array of 27 trit values in Rep C {1, 2, 3}.
 * @returns Array of 6 trit values in Rep C {1, 2, 3}.
 * @throws If any trit is outside Rep C range (zero = forgery).
 */
export function computeChecksumRepC(trits: number[]): number[] {
  if (trits.length !== 27) {
    throw new Error(`Expected 27 trits, got ${trits.length}`);
  }

  // Validate Rep C: no zeros allowed
  for (let i = 0; i < 27; i++) {
    if (trits[i] < 1 || trits[i] > 3) {
      throw new Error(
        `Trit ${i} has invalid Rep C value ${trits[i]}: must be 1, 2, or 3 (zero = forgery)`
      );
    }
  }

  // Horner's method with mod reduction (MSB-first → index 26 down to 0)
  let value = 0;
  for (let i = 26; i >= 0; i--) {
    const tritB = trits[i] - 1; // Rep C → Rep B
    value = (value * 3 + tritB) % REPUNIT_R6;
  }

  // Decompose into 6 Rep C trits (LSB first)
  const checksum: number[] = new Array(CHECKSUM_TRIT_COUNT);
  let remaining = value;
  for (let i = 0; i < CHECKSUM_TRIT_COUNT; i++) {
    checksum[i] = (remaining % 3) + 1; // Rep B → Rep C
    remaining = Math.floor(remaining / 3);
  }

  return checksum;
}

/**
 * Verify a 27-trit address against its 6-trit checksum.
 */
export function verifyChecksum(trits: number[], expectedChecksum: number[]): boolean {
  const computed = computeChecksumRepC(trits);
  if (computed.length !== expectedChecksum.length) return false;
  return computed.every((v, i) => v === expectedChecksum[i]);
}

/**
 * Compute raw checksum value (0-363) without trit decomposition.
 */
export function computeChecksumRaw(trits: number[]): number {
  if (trits.length !== 27) {
    throw new Error(`Expected 27 trits, got ${trits.length}`);
  }
  for (let i = 0; i < 27; i++) {
    if (trits[i] < 1 || trits[i] > 3) {
      throw new Error(`Trit ${i} invalid: ${trits[i]}`);
    }
  }

  let value = 0;
  for (let i = 26; i >= 0; i--) {
    value = (value * 3 + (trits[i] - 1)) % REPUNIT_R6;
  }
  return value;
}
