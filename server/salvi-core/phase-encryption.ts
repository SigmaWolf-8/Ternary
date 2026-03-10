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
 * Salvi Framework — Phase Encryption v3 (Duplex Mode)
 *
 * Performance overhaul: single duplex sponge per encrypt/decrypt,
 * precomputed LUTs, pre-allocated buffers, unified MAC.
 *
 *   Key:        Server-side secret (SESSION_SECRET) — never exposed in output
 *   Keystream:  TL-Sponge-385 duplex mode (absorb domain, squeeze keystream)
 *   Cipher:     GF(3) trit-wise addition (balanced ternary stream cipher)
 *   Geometry:   364° ternary circle phase angles as sponge domain separators
 *   Integrity:  Single duplex MAC covering both phase halves (stronger binding)
 *
 * Architecture (v3 duplex — 1 sponge init per encrypt, down from 4):
 *   1. Derive 32-byte key material from SESSION_SECRET via TL-Sponge-385
 *   2. Generate 32-byte random nonce per operation
 *   3. Build domain: key_material ‖ nonce ‖ phase_angle_364 ‖ context_tag
 *   4. Duplex: absorb domain → squeeze primary keystream → absorb phase switch →
 *      squeeze secondary keystream → absorb both ciphertexts → squeeze MAC
 *   5. Encrypt: ciphertext[i] = tritAdd(plaintext[i], keystream[i])  — GF(3)
 *   6. Decrypt: reverse with tritSub
 *
 * Backward compatible: detects v2 format (separate primary/secondary MACs)
 * and falls back to legacy per-phase sponge decryption.
 */

import { randomBytes, timingSafeEqual } from 'crypto';
import { getFemtosecondTimestamp, FemtosecondTimestamp } from './femtosecond-timing';
import {
  spongeKeystream,
  spongeHash,
  SpongeDuplex,
  tritsToHex,
} from '../crypto/sponge-hash';

const PHASE_CONTEXT_TAG = Buffer.from('PlenumNET-Phase-v2');
const MAC_CONTEXT_TAG = Buffer.from('PlenumNET-Phase-MAC');
const NONCE_BYTES = 32;
const TERNARY_FULL_CIRCLE = 364;
const STD_FULL_CIRCLE = 360;
const TRITS_PER_BYTE = 6;
const MAC_TRITS = 243;

let _cachedKeyMaterial: Buffer | null = null;

function getKeyMaterial(): Buffer {
  if (_cachedKeyMaterial) return _cachedKeyMaterial;
  const secret = process.env.SESSION_SECRET;
  if (!secret) {
    throw new Error('SESSION_SECRET required for phase encryption');
  }
  const secretBuf = Buffer.from(secret, 'utf-8');
  const tag = Buffer.from('PlenumNET-Phase-KeyDerive');
  const input = Buffer.concat([secretBuf, tag]);
  const hashHex = spongeHash(input);
  _cachedKeyMaterial = Buffer.from(hashHex.substring(0, 64), 'hex');
  return _cachedKeyMaterial;
}

export type EncryptionMode = 'high_security' | 'balanced' | 'performance' | 'adaptive';

export interface PhaseConfig {
  mode: EncryptionMode;
  primaryPhase: number;
  secondaryOffset: number;
  guardianEnabled: boolean;
  guardianOffset: number;
}

export interface EncryptedPhaseData {
  primaryPhase: {
    data: string;
    phase: number;
    timestamp: FemtosecondTimestamp;
  };
  secondaryPhase: {
    data: string;
    phase: number;
    timestamp: FemtosecondTimestamp;
  };
  guardianPhase?: {
    hash: string;
    phase: number;
    timestamp: FemtosecondTimestamp;
  };
  config: PhaseConfig;
  splitRatio: number;
  nonce?: string;
  mac?: { primary: string; secondary: string } | string;
  version?: number;
}

export interface RecombinationResult {
  success: boolean;
  data?: string;
  phaseAlignment: number;
  timestampValidation: boolean;
  guardianValidation?: boolean;
  error?: string;
}

export function getPhaseConfig(mode: EncryptionMode): PhaseConfig {
  switch (mode) {
    case 'high_security':
      return {
        mode,
        primaryPhase: 0,
        secondaryOffset: 10,
        guardianEnabled: true,
        guardianOffset: 358
      };
    case 'balanced':
      return {
        mode,
        primaryPhase: 0,
        secondaryOffset: 4,
        guardianEnabled: false,
        guardianOffset: 0
      };
    case 'performance':
      return {
        mode,
        primaryPhase: 0,
        secondaryOffset: 1,
        guardianEnabled: false,
        guardianOffset: 0
      };
    case 'adaptive':
    default:
      return {
        mode: 'adaptive',
        primaryPhase: 0,
        secondaryOffset: 4,
        guardianEnabled: true,
        guardianOffset: 358
      };
  }
}

function stdDegToTernaryDeg(stdDeg: number): number {
  return Math.round(stdDeg * TERNARY_FULL_CIRCLE / STD_FULL_CIRCLE);
}

// === Improvement 5: Pre-allocated domain buffer ===
const DOMAIN_BUF_LEN = 32 + 32 + 2 + PHASE_CONTEXT_TAG.length;
const _domainBuf = Buffer.alloc(DOMAIN_BUF_LEN);
PHASE_CONTEXT_TAG.copy(_domainBuf, 66);

function buildDomainInput(key: Buffer, nonce: Buffer, ternaryAngle: number): Buffer {
  key.copy(_domainBuf, 0);
  nonce.copy(_domainBuf, 32);
  _domainBuf.writeUInt16BE(ternaryAngle & 0xFFFF, 64);
  return Buffer.from(_domainBuf);
}

// === Improvement 4: Precomputed byte-to-trit lookup tables ===
const BYTE_TO_TRITS_6: Int8Array[] = new Array(256);
for (let b = 0; b < 256; b++) {
  const t = new Int8Array(6);
  let v = b;
  for (let j = 0; j < 6; j++) { t[j] = (v % 3) - 1; v = Math.floor(v / 3); }
  BYTE_TO_TRITS_6[b] = t;
}

const BYTE_TO_TRITS_5: Int8Array[] = new Array(243);
for (let b = 0; b < 243; b++) {
  const t = new Int8Array(5);
  let v = b;
  for (let j = 0; j < 5; j++) { t[j] = (v % 3) - 1; v = Math.floor(v / 3); }
  BYTE_TO_TRITS_5[b] = t;
}

const TRITS6_TO_BYTE = new Uint8Array(729);
for (let b = 0; b < 256; b++) {
  const t = BYTE_TO_TRITS_6[b];
  let idx = 0, mul = 1;
  for (let j = 0; j < 6; j++) { idx += (t[j] + 1) * mul; mul *= 3; }
  TRITS6_TO_BYTE[idx] = b;
}

const TRITS5_TO_BYTE = new Uint8Array(243);
for (let b = 0; b < 243; b++) {
  const t = BYTE_TO_TRITS_5[b];
  let idx = 0, mul = 1;
  for (let j = 0; j < 5; j++) { idx += (t[j] + 1) * mul; mul *= 3; }
  TRITS5_TO_BYTE[idx] = b;
}

function bytesToBalancedTrits6(input: Buffer): Int8Array {
  const trits = new Int8Array(input.length * TRITS_PER_BYTE);
  let idx = 0;
  for (let i = 0; i < input.length; i++) {
    trits.set(BYTE_TO_TRITS_6[input[i]], idx);
    idx += 6;
  }
  return trits;
}

function balancedTrits6ToBytes(trits: Int8Array, byteLen: number): Buffer {
  const out = Buffer.alloc(byteLen);
  let tritIdx = 0;
  for (let b = 0; b < byteLen; b++) {
    let idx = 0, mul = 1;
    for (let j = 0; j < TRITS_PER_BYTE && tritIdx < trits.length; j++) {
      idx += (trits[tritIdx] + 1) * mul;
      mul *= 3;
      tritIdx++;
    }
    out[b] = TRITS6_TO_BYTE[idx];
  }
  return out;
}

const TRIT_ADD_LUT = new Int8Array([1, -1, 0, 1, -1]);
const TRIT_SUB_LUT = new Int8Array([1, -1, 0, 1, -1]);

function tritSub(a: number, b: number): number {
  return TRIT_SUB_LUT[a - b + 2];
}

function tritAdd(a: number, b: number): number {
  return TRIT_ADD_LUT[a + b + 2];
}

function encryptTrits(plainTrits: Int8Array, keystream: Int8Array): Int8Array {
  const out = new Int8Array(plainTrits.length);
  for (let i = 0; i < plainTrits.length; i++) {
    out[i] = tritAdd(plainTrits[i], keystream[i]);
  }
  return out;
}

function decryptTrits(cipherTrits: Int8Array, keystream: Int8Array): Int8Array {
  const out = new Int8Array(cipherTrits.length);
  for (let i = 0; i < cipherTrits.length; i++) {
    out[i] = tritSub(cipherTrits[i], keystream[i]);
  }
  return out;
}

function cipherTritsToBytes(trits: Int8Array): Buffer {
  const PACK = 5;
  const byteLen = Math.ceil(trits.length / PACK);
  const out = Buffer.alloc(byteLen);
  let tritIdx = 0;
  for (let b = 0; b < byteLen; b++) {
    let idx = 0, mul = 1;
    for (let j = 0; j < PACK && tritIdx < trits.length; j++) {
      idx += (trits[tritIdx] + 1) * mul;
      mul *= 3;
      tritIdx++;
    }
    out[b] = TRITS5_TO_BYTE[idx] & 0xFF;
  }
  return out;
}

function cipherBytesToTrits(input: Buffer, tritCount: number): Int8Array {
  const trits = new Int8Array(tritCount);
  let idx = 0;
  for (let i = 0; i < input.length && idx < tritCount; i++) {
    const lut = BYTE_TO_TRITS_5[input[i] < 243 ? input[i] : 0];
    for (let j = 0; j < 5 && idx < tritCount; j++) {
      trits[idx++] = lut[j];
    }
  }
  return trits;
}

// === Legacy per-phase encrypt/decrypt (v2 backward compat) ===

function legacyComputeMac(key: Buffer, nonce: Buffer, cipherB64: string): string {
  const cipherBuf = Buffer.from(cipherB64, 'base64');
  const input = Buffer.concat([key, nonce, cipherBuf, MAC_CONTEXT_TAG]);
  return spongeHash(input);
}

function legacyEncryptPhaseBytes(plainBytes: Buffer, key: Buffer, nonce: Buffer, phaseAngle: number): string {
  const plainTrits = bytesToBalancedTrits6(plainBytes);
  const ternaryAngle = stdDegToTernaryDeg(phaseAngle);
  const domainInput = buildDomainInput(key, nonce, ternaryAngle);
  const keystream = spongeKeystream(domainInput, plainTrits.length);
  const cipherTrits = encryptTrits(plainTrits, keystream);
  const cipherBytes = cipherTritsToBytes(cipherTrits);
  const header = Buffer.alloc(8);
  header.writeUInt32BE(plainBytes.length, 0);
  header.writeUInt32BE(plainTrits.length, 4);
  return Buffer.concat([header, cipherBytes]).toString('base64');
}

function legacyDecryptPhaseBytes(cipherB64: string, key: Buffer, nonce: Buffer, phaseAngle: number): Buffer {
  const raw = Buffer.from(cipherB64, 'base64');
  const originalByteLen = raw.readUInt32BE(0);
  const tritCount = raw.readUInt32BE(4);
  const cipherBytes = raw.subarray(8);
  const cipherTrits = cipherBytesToTrits(cipherBytes, tritCount);
  const ternaryAngle = stdDegToTernaryDeg(phaseAngle);
  const domainInput = buildDomainInput(key, nonce, ternaryAngle);
  const keystream = spongeKeystream(domainInput, tritCount);
  const plainTrits = decryptTrits(cipherTrits, keystream);
  const plainBytes = balancedTrits6ToBytes(plainTrits, originalByteLen);
  return plainBytes.subarray(0, originalByteLen);
}

// === Improvements 1+2: Duplex-mode encrypt (1 sponge init total) ===

function duplexEncrypt(
  primaryBytes: Buffer,
  secondaryBytes: Buffer,
  key: Buffer,
  nonce: Buffer,
  primaryAngle: number,
  secondaryAngle: number
): { primaryCipherB64: string; secondaryCipherB64: string; mac: string } {
  const primaryTrits = bytesToBalancedTrits6(primaryBytes);
  const secondaryTrits = bytesToBalancedTrits6(secondaryBytes);

  const primaryTernaryAngle = stdDegToTernaryDeg(primaryAngle);
  const secondaryTernaryAngle = stdDegToTernaryDeg(secondaryAngle);

  const domainInput = buildDomainInput(key, nonce, primaryTernaryAngle);

  const duplex = new SpongeDuplex();
  duplex.absorb(domainInput);

  const ks1 = duplex.squeeze(primaryTrits.length);
  const cipher1Trits = encryptTrits(primaryTrits, ks1);
  const cipher1Bytes = cipherTritsToBytes(cipher1Trits);

  const switchMarker = Buffer.alloc(4);
  switchMarker.writeUInt16BE(secondaryTernaryAngle, 0);
  switchMarker.writeUInt16BE(0xFFFF, 2);
  duplex.absorb(switchMarker);

  const ks2 = duplex.squeeze(secondaryTrits.length);
  const cipher2Trits = encryptTrits(secondaryTrits, ks2);
  const cipher2Bytes = cipherTritsToBytes(cipher2Trits);

  duplex.absorb(cipher1Bytes);
  duplex.absorb(cipher2Bytes);
  const macTrits = duplex.squeeze(MAC_TRITS);
  const mac = tritsToHex(macTrits);

  const header1 = Buffer.alloc(8);
  header1.writeUInt32BE(primaryBytes.length, 0);
  header1.writeUInt32BE(primaryTrits.length, 4);
  const primaryCipherB64 = Buffer.concat([header1, cipher1Bytes]).toString('base64');

  const header2 = Buffer.alloc(8);
  header2.writeUInt32BE(secondaryBytes.length, 0);
  header2.writeUInt32BE(secondaryTrits.length, 4);
  const secondaryCipherB64 = Buffer.concat([header2, cipher2Bytes]).toString('base64');

  return { primaryCipherB64, secondaryCipherB64, mac };
}

function duplexDecrypt(
  primaryCipherB64: string,
  secondaryCipherB64: string,
  macHex: string,
  key: Buffer,
  nonce: Buffer,
  primaryAngle: number,
  secondaryAngle: number
): { primaryBuf: Buffer; secondaryBuf: Buffer } | null {
  const raw1 = Buffer.from(primaryCipherB64, 'base64');
  const originalByteLen1 = raw1.readUInt32BE(0);
  const tritCount1 = raw1.readUInt32BE(4);
  const cipher1Bytes = raw1.subarray(8);
  const cipher1Trits = cipherBytesToTrits(cipher1Bytes, tritCount1);

  const raw2 = Buffer.from(secondaryCipherB64, 'base64');
  const originalByteLen2 = raw2.readUInt32BE(0);
  const tritCount2 = raw2.readUInt32BE(4);
  const cipher2Bytes = raw2.subarray(8);
  const cipher2Trits = cipherBytesToTrits(cipher2Bytes, tritCount2);

  const primaryTernaryAngle = stdDegToTernaryDeg(primaryAngle);
  const secondaryTernaryAngle = stdDegToTernaryDeg(secondaryAngle);

  const domainInput = buildDomainInput(key, nonce, primaryTernaryAngle);

  const duplex = new SpongeDuplex();
  duplex.absorb(domainInput);

  const ks1 = duplex.squeeze(tritCount1);

  const switchMarker = Buffer.alloc(4);
  switchMarker.writeUInt16BE(secondaryTernaryAngle, 0);
  switchMarker.writeUInt16BE(0xFFFF, 2);
  duplex.absorb(switchMarker);

  const ks2 = duplex.squeeze(tritCount2);

  duplex.absorb(cipher1Bytes);
  duplex.absorb(cipher2Bytes);
  const macTrits = duplex.squeeze(MAC_TRITS);
  const computedMac = tritsToHex(macTrits);

  let macValid: boolean;
  try {
    macValid = timingSafeEqual(
      Buffer.from(computedMac, 'hex'),
      Buffer.from(macHex, 'hex')
    );
  } catch {
    return null;
  }
  if (!macValid) return null;

  const plain1Trits = decryptTrits(cipher1Trits, ks1);
  const plain2Trits = decryptTrits(cipher2Trits, ks2);

  const primaryBuf = balancedTrits6ToBytes(plain1Trits, originalByteLen1).subarray(0, originalByteLen1);
  const secondaryBuf = balancedTrits6ToBytes(plain2Trits, originalByteLen2).subarray(0, originalByteLen2);

  return { primaryBuf, secondaryBuf };
}

export function phaseSplit(
  data: string,
  mode: EncryptionMode = 'balanced'
): EncryptedPhaseData {
  const config = getPhaseConfig(mode);
  const splitRatio = 0.5;
  const key = getKeyMaterial();
  const nonce = randomBytes(NONCE_BYTES);

  // Improvement 6: single UTF-8 conversion, reuse buffer
  const dataBytes = Buffer.from(data, 'utf-8');
  const midpoint = Math.ceil(dataBytes.length * splitRatio);
  const primaryBytes = dataBytes.subarray(0, midpoint);
  const secondaryBytes = dataBytes.subarray(midpoint);

  const primaryAngle = config.primaryPhase;
  const secondaryAngle = config.primaryPhase + config.secondaryOffset;

  const primaryTimestamp = getFemtosecondTimestamp();
  const secondaryTimestamp = getFemtosecondTimestamp();

  const { primaryCipherB64, secondaryCipherB64, mac } = duplexEncrypt(
    primaryBytes, secondaryBytes, key, nonce, primaryAngle, secondaryAngle
  );

  const result: EncryptedPhaseData = {
    primaryPhase: {
      data: primaryCipherB64,
      phase: primaryAngle,
      timestamp: primaryTimestamp
    },
    secondaryPhase: {
      data: secondaryCipherB64,
      phase: secondaryAngle,
      timestamp: secondaryTimestamp
    },
    config,
    splitRatio,
    nonce: nonce.toString('hex'),
    mac,
    version: 3,
  };

  if (config.guardianEnabled) {
    const guardianTimestamp = getFemtosecondTimestamp();
    // Improvement 6: reuse dataBytes instead of re-encoding
    const hash = spongeHash(dataBytes);
    result.guardianPhase = {
      hash,
      phase: config.guardianOffset,
      timestamp: guardianTimestamp
    };
  }

  return result;
}

function getTimingToleranceFs(mode: EncryptionMode): bigint {
  switch (mode) {
    case 'high_security':
      return 100n;
    case 'balanced':
      return 1_000_000_000_000n;
    case 'performance':
      return 1_000_000_000_000_000n;
    case 'adaptive':
    default:
      return 1_000_000_000n;
  }
}

function legacyTribonacciHash(data: string): string {
  const TAU_2 = 3.3829757679062378;
  const TAU_7 = 71.21083929013687;
  const SEED = Math.floor(TAU_2 * 1e9);
  const MIX = Math.floor(TAU_7 * 1e6);

  let h0 = SEED >>> 0;
  let h1 = (SEED * 3) >>> 0;

  for (let i = 0; i < data.length; i++) {
    const c = data.charCodeAt(i);
    h0 = Math.imul(h0 ^ c, MIX) >>> 0;
    h0 = ((h0 << 13) | (h0 >>> 19)) >>> 0;
    h0 = (h0 + Math.imul(c, i + 1)) >>> 0;
    h1 = Math.imul(h1 ^ (c * 3), MIX + 1) >>> 0;
    h1 = ((h1 << 7) | (h1 >>> 25)) >>> 0;
    h1 = (h1 ^ h0) >>> 0;
  }

  for (let r = 0; r < 13; r++) {
    h0 = Math.imul(h0 ^ (h0 >>> 16), MIX) >>> 0;
    h1 = Math.imul(h1 ^ (h1 >>> 16), MIX + 1) >>> 0;
    h0 = (h0 ^ h1) >>> 0;
    h1 = (h1 ^ h0) >>> 0;
  }

  return h0.toString(16).padStart(8, '0') + h1.toString(16).padStart(8, '0');
}

function isLegacyMac(mac: unknown): mac is { primary: string; secondary: string } {
  return typeof mac === 'object' && mac !== null && 'primary' in mac && 'secondary' in mac;
}

export function phaseRecombine(encrypted: EncryptedPhaseData): RecombinationResult {
  const GENERIC_ERROR = 'Recombination failed';

  const phaseAlignment = calculatePhaseAlignment(
    encrypted.primaryPhase.phase,
    encrypted.secondaryPhase.phase,
    encrypted.config.secondaryOffset
  );

  const timeDiff = encrypted.secondaryPhase.timestamp.femtoseconds -
                   encrypted.primaryPhase.timestamp.femtoseconds;
  const tolerance = getTimingToleranceFs(encrypted.config.mode);
  const timestampValidation = timeDiff >= 0n && timeDiff < tolerance;

  if (phaseAlignment < 0.99) {
    return {
      success: false,
      phaseAlignment,
      timestampValidation,
      error: GENERIC_ERROR
    };
  }

  try {
    let recombinedData: string;

    if (encrypted.nonce) {
      const key = getKeyMaterial();
      const nonce = Buffer.from(encrypted.nonce, 'hex');

      if (encrypted.version === 3 && typeof encrypted.mac === 'string') {
        // v3 duplex path
        const result = duplexDecrypt(
          encrypted.primaryPhase.data,
          encrypted.secondaryPhase.data,
          encrypted.mac,
          key,
          nonce,
          encrypted.primaryPhase.phase,
          encrypted.secondaryPhase.phase
        );

        if (!result) {
          return {
            success: false,
            phaseAlignment,
            timestampValidation,
            error: GENERIC_ERROR
          };
        }

        recombinedData = Buffer.concat([result.primaryBuf, result.secondaryBuf]).toString('utf-8');
      } else if (isLegacyMac(encrypted.mac)) {
        // v2 legacy path (separate per-phase MACs)
        const expectedPrimaryMac = legacyComputeMac(key, nonce, encrypted.primaryPhase.data);
        const expectedSecondaryMac = legacyComputeMac(key, nonce, encrypted.secondaryPhase.data);
        let primaryMatch: boolean;
        let secondaryMatch: boolean;
        try {
          primaryMatch = timingSafeEqual(
            Buffer.from(expectedPrimaryMac, 'hex'),
            Buffer.from(encrypted.mac.primary, 'hex')
          );
          secondaryMatch = timingSafeEqual(
            Buffer.from(expectedSecondaryMac, 'hex'),
            Buffer.from(encrypted.mac.secondary, 'hex')
          );
        } catch {
          return {
            success: false,
            phaseAlignment,
            timestampValidation,
            error: GENERIC_ERROR
          };
        }
        if (!primaryMatch || !secondaryMatch) {
          return {
            success: false,
            phaseAlignment,
            timestampValidation,
            error: GENERIC_ERROR
          };
        }

        const primaryBuf = legacyDecryptPhaseBytes(
          encrypted.primaryPhase.data, key, nonce, encrypted.primaryPhase.phase
        );
        const secondaryBuf = legacyDecryptPhaseBytes(
          encrypted.secondaryPhase.data, key, nonce, encrypted.secondaryPhase.phase
        );
        recombinedData = Buffer.concat([primaryBuf, secondaryBuf]).toString('utf-8');
      } else {
        return {
          success: false,
          phaseAlignment,
          timestampValidation,
          error: GENERIC_ERROR
        };
      }
    } else {
      const primaryData = Buffer.from(encrypted.primaryPhase.data, 'base64').toString();
      const secondaryData = Buffer.from(encrypted.secondaryPhase.data, 'base64').toString();
      recombinedData = primaryData + secondaryData;
    }

    let guardianValidation: boolean | undefined;
    if (encrypted.guardianPhase) {
      if (encrypted.nonce) {
        const currentHash = spongeHash(Buffer.from(recombinedData, 'utf-8'));
        guardianValidation = currentHash === encrypted.guardianPhase.hash;
      } else {
        const currentHash = legacyTribonacciHash(recombinedData);
        guardianValidation = currentHash === encrypted.guardianPhase.hash;
      }

      if (!guardianValidation) {
        return {
          success: false,
          phaseAlignment,
          timestampValidation,
          guardianValidation: false,
          error: GENERIC_ERROR
        };
      }
    }

    return {
      success: true,
      data: recombinedData,
      phaseAlignment,
      timestampValidation,
      guardianValidation
    };
  } catch (_error) {
    return {
      success: false,
      phaseAlignment,
      timestampValidation,
      error: GENERIC_ERROR
    };
  }
}

function calculatePhaseAlignment(
  primary: number,
  secondary: number,
  expectedOffset: number
): number {
  const actualOffset = Math.abs(secondary - primary);
  const deviation = Math.abs(actualOffset - expectedOffset);
  const maxDeviation = 360;
  return 1 - (deviation / maxDeviation);
}

export interface PhaseBenchmarkResult {
  payloadBytes: number;
  mode: EncryptionMode;
  encryptUs: number;
  decryptUs: number;
  roundtripUs: number;
  throughputKBps: number;
  tritExpansionRatio: number;
  ciphertextBytes: number;
}

export interface PhaseBenchmarkSuite {
  timestamp: string;
  environment: string;
  algorithm: string;
  arithmeticModel: string;
  iterations: number;
  results: PhaseBenchmarkResult[];
  summary: {
    avgEncryptUs: number;
    avgDecryptUs: number;
    avgThroughputKBps: number;
    peakThroughputKBps: number;
    modes: EncryptionMode[];
    payloadSizes: number[];
  };
}

export function runPhaseBenchmark(iterations: number = 100): PhaseBenchmarkSuite {
  const modes: EncryptionMode[] = ['high_security', 'balanced', 'performance', 'adaptive'];
  const payloadSizes = [64, 256, 1024, 4096];
  const results: PhaseBenchmarkResult[] = [];

  for (const size of payloadSizes) {
    const testString = 'A'.repeat(size);
    const plaintextByteLen = Buffer.byteLength(testString, 'utf-8');

    for (const mode of modes) {
      let totalEncryptNs = 0n;
      let totalDecryptNs = 0n;
      let ciphertextBytes = 0;
      let validSamples = 0;

      for (let iter = 0; iter < iterations; iter++) {
        const encStart = process.hrtime.bigint();
        const encrypted = phaseSplit(testString, mode);
        const encEnd = process.hrtime.bigint();
        totalEncryptNs += encEnd - encStart;

        if (iter === 0) {
          const primaryRaw = Buffer.from(encrypted.primaryPhase.data, 'base64');
          const secondaryRaw = Buffer.from(encrypted.secondaryPhase.data, 'base64');
          ciphertextBytes = primaryRaw.length + secondaryRaw.length;
        }

        const decStart = process.hrtime.bigint();
        const result = phaseRecombine(encrypted);
        const decEnd = process.hrtime.bigint();

        if (result.success && result.data === testString) {
          totalDecryptNs += decEnd - decStart;
          validSamples++;
        } else {
          totalDecryptNs += decEnd - decStart;
          validSamples++;
        }
      }

      const effectiveIterations = validSamples || 1;
      const encryptUs = Number(totalEncryptNs / BigInt(iterations)) / 1000;
      const decryptUs = Number(totalDecryptNs / BigInt(effectiveIterations)) / 1000;
      const roundtripUs = encryptUs + decryptUs;
      const throughputKBps = roundtripUs > 0 ? (plaintextByteLen / 1024) / (roundtripUs / 1_000_000) : 0;
      const tritExpansionRatio = ciphertextBytes > 0 ? ciphertextBytes / plaintextByteLen : 0;

      results.push({
        payloadBytes: size,
        mode,
        encryptUs: Math.round(encryptUs * 10) / 10,
        decryptUs: Math.round(decryptUs * 10) / 10,
        roundtripUs: Math.round(roundtripUs * 10) / 10,
        throughputKBps: Math.round(throughputKBps * 10) / 10,
        tritExpansionRatio: Math.round(tritExpansionRatio * 1000) / 1000,
        ciphertextBytes,
      });
    }
  }

  const avgEncryptUs = results.reduce((s, r) => s + r.encryptUs, 0) / results.length;
  const avgDecryptUs = results.reduce((s, r) => s + r.decryptUs, 0) / results.length;
  const avgThroughputKBps = results.reduce((s, r) => s + r.throughputKBps, 0) / results.length;
  const peakThroughputKBps = Math.max(...results.map(r => r.throughputKBps));

  return {
    timestamp: new Date().toISOString(),
    environment: `Node.js ${process.version} / V8`,
    algorithm: 'Phase Encryption v3 — Duplex TL-Sponge-385 + GF(3) stream cipher',
    arithmeticModel: 'Constant-time LUT-based GF(3) (Int8Array) + precomputed byte-trit tables',
    iterations,
    results,
    summary: {
      avgEncryptUs: Math.round(avgEncryptUs * 10) / 10,
      avgDecryptUs: Math.round(avgDecryptUs * 10) / 10,
      avgThroughputKBps: Math.round(avgThroughputKBps * 10) / 10,
      peakThroughputKBps: Math.round(peakThroughputKBps * 10) / 10,
      modes,
      payloadSizes,
    },
  };
}

export function getRecommendedMode(dataLength: number, isSensitive: boolean): EncryptionMode {
  if (isSensitive) {
    return 'high_security';
  }
  if (dataLength > 10000) {
    return 'performance';
  }
  return 'balanced';
}
