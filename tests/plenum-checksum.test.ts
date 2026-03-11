/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved — Applied Physics Division
 */
import { describe, it, expect } from 'vitest';
import {
  computePlenumChecksum,
  computeRepunitChecksum,
  computeDualChecksum,
  verifyPlenumChecksum,
  verifyDualChecksum,
  verifyAddressIntegrity,
  isValidRepC,
  PLENUM_MODULUS,
  REPUNIT_MODULUS,
  DUAL_DETECTION_SPACE,
  CLASSIFICATION_TRITS,
  CHECKSUM_TRITS,
} from '../shared/plenum-checksum';

const GOOGLE_ADDR = [
  2, 3, 2, 3,
  1, 1, 3, 3,
  3, 1, 3, 1,
  1, 3, 2, 2,
  2, 3, 3, 1,
  1, 2, 1, 2,
  3, 1, 3,
];

const PPTPRO_ADDR = [
  2, 3, 3, 3,
  2, 3, 3, 3,
  2, 2, 2, 2,
  3, 3, 3, 3,
  1, 2, 2, 1,
  2, 1, 3, 3,
  3, 3, 2,
];

function gcd(a: number, b: number): number {
  while (b !== 0) { const t = b; b = a % b; a = t; }
  return a;
}

describe('Plenum Checksum — Dual-Modulus Address Integrity', () => {
  it('constants are correct', () => {
    expect(PLENUM_MODULUS).toBe(333);
    expect(REPUNIT_MODULUS).toBe(364);
    expect(DUAL_DETECTION_SPACE).toBe(121212);
    expect(CLASSIFICATION_TRITS).toBe(27);
    expect(CHECKSUM_TRITS).toBe(6);
  });

  it('333 and 364 are coprime (gcd = 1)', () => {
    expect(gcd(333, 364)).toBe(1);
  });

  it('Google fixture — Plenum checksum', () => {
    expect(computePlenumChecksum(GOOGLE_ADDR)).toEqual([2, 1, 1, 2, 1, 3]);
  });

  it('Google fixture — Repunit checksum', () => {
    expect(computeRepunitChecksum(GOOGLE_ADDR)).toEqual([2, 1, 2, 3, 2, 1]);
  });

  it('dual checksum matches individual computations', () => {
    const dual = computeDualChecksum(GOOGLE_ADDR);
    const singleRepunit = computeRepunitChecksum(GOOGLE_ADDR);
    const singlePlenum = computePlenumChecksum(GOOGLE_ADDR);
    expect(dual.repunit).toEqual(singleRepunit);
    expect(dual.plenum).toEqual(singlePlenum);
  });

  it('round-trip verification passes', () => {
    const dual = computeDualChecksum(GOOGLE_ADDR);
    expect(verifyDualChecksum(GOOGLE_ADDR, dual.repunit, dual.plenum)).toBe(true);
    expect(verifyPlenumChecksum(GOOGLE_ADDR, dual.plenum)).toBe(true);
  });

  it('tamper detection — single trit flip detected', () => {
    const dual = computeDualChecksum(GOOGLE_ADDR);
    const tampered = [...GOOGLE_ADDR];
    tampered[13] = tampered[13] === 1 ? 2 : 1;
    expect(verifyDualChecksum(tampered, dual.repunit, dual.plenum)).toBe(false);
  });

  it('all-ones boundary → [1,1,1,1,1,1] for both', () => {
    const allOnes = new Array(27).fill(1);
    const pCk = computePlenumChecksum(allOnes);
    const rCk = computeRepunitChecksum(allOnes);
    expect(pCk).toEqual([1, 1, 1, 1, 1, 1]);
    expect(rCk).toEqual([1, 1, 1, 1, 1, 1]);
  });

  it('PPTPro fixture — HPTP-mandatory address round-trips', () => {
    const dual = computeDualChecksum(PPTPRO_ADDR);
    expect(verifyDualChecksum(PPTPRO_ADDR, dual.repunit, dual.plenum)).toBe(true);
    expect(PPTPRO_ADDR[14]).toBe(3);
    expect(PPTPRO_ADDR[15]).toBe(3);
  });

  it('forgery detection — zero in Rep C fails integrity', () => {
    const dual = computeDualChecksum(GOOGLE_ADDR);
    const forged = [...GOOGLE_ADDR];
    forged[0] = 0;
    const result = verifyAddressIntegrity(forged, [...dual.repunit], [...dual.plenum]);
    expect(result.valid).toBe(false);
    expect(result.repC).toBe(false);
  });

  it('wrong length throws', () => {
    expect(() => computePlenumChecksum([1, 2, 3])).toThrow();
    expect(() => computeRepunitChecksum(new Array(10).fill(1))).toThrow();
    expect(() => computeDualChecksum(new Array(28).fill(1))).toThrow();
  });

  it('Rep C validation — valid and invalid arrays', () => {
    expect(isValidRepC([1, 2, 3, 1, 2, 3])).toBe(true);
    expect(isValidRepC([0, 2, 3])).toBe(false);
    expect(isValidRepC([1, 4, 3])).toBe(false);
    expect(isValidRepC([])).toBe(true);
  });

  it('all 54 single-trit flips detected by dual check', () => {
    const dual = computeDualChecksum(GOOGLE_ADDR);
    for (let i = 0; i < 27; i++) {
      for (const flip of [1, 2, 3]) {
        if (flip === GOOGLE_ADDR[i]) continue;
        const tampered = [...GOOGLE_ADDR];
        tampered[i] = flip;
        expect(verifyDualChecksum(tampered, dual.repunit, dual.plenum)).toBe(false);
      }
    }
  });
});
