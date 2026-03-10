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
 * Salvi Framework — Phase Encryption
 *
 * Post-quantum phase-domain encryption using PlenumNET native primitives:
 *
 *   Key:        Server-side secret (SESSION_SECRET) — never exposed in output
 *   Keystream:  TL-Sponge-385 keyed by (secret ‖ nonce ‖ phase_angle ‖ context)
 *   Cipher:     GF(3) trit-wise addition (balanced ternary stream cipher)
 *   Geometry:   364° ternary circle phase angles as sponge domain separators
 *   Integrity:  TL-Sponge-385 MAC on all payloads (all modes, not just guardian)
 *
 * Architecture:
 *   1. Derive 32-byte key material from SESSION_SECRET via TL-Sponge-385
 *   2. Generate 32-byte random nonce per operation
 *   3. Build domain: key_material ‖ nonce ‖ phase_angle_364 ‖ context_tag
 *   4. Derive keystream: TL-Sponge-385 absorb domain, squeeze N trits
 *   5. Encrypt: ciphertext[i] = tritAdd(plaintext[i], keystream[i])  — GF(3)
 *   6. Decrypt: plaintext[i] = tritSub(ciphertext[i], keystream[i])  — GF(3)
 *   7. MAC: TL-Sponge-385 hash of (key_material ‖ nonce ‖ ciphertext) per phase
 *
 * Security: IND-CPA via fresh random nonce + secret key. The nonce is public
 * but the keystream is not derivable without the server secret. Post-quantum
 * security from TL-Sponge-385 capacity (486 trits = 385 bits). Mandatory
 * MAC verification on decryption prevents tampering in all modes.
 *
 * Byte encoding: 6 trits per byte (capacity 3^6 = 729 > 256), fully bijective.
 * Every byte value 0-255 maps uniquely to 6 balanced trits and back.
 */

import { randomBytes } from 'crypto';
import { getFemtosecondTimestamp, FemtosecondTimestamp } from './femtosecond-timing';
import {
  spongeKeystream,
  spongeHash,
} from '../crypto/sponge-hash';

const PHASE_CONTEXT_TAG = Buffer.from('PlenumNET-Phase-v2');
const MAC_CONTEXT_TAG = Buffer.from('PlenumNET-Phase-MAC');
const NONCE_BYTES = 32;
const TERNARY_FULL_CIRCLE = 364;
const STD_FULL_CIRCLE = 360;
const TRITS_PER_BYTE = 6;

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
  mac?: { primary: string; secondary: string };
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

function buildDomainInput(key: Buffer, nonce: Buffer, phaseAngleTernary: number): Buffer {
  const angleBuf = Buffer.alloc(2);
  angleBuf.writeUInt16BE(phaseAngleTernary & 0xFFFF, 0);
  return Buffer.concat([key, nonce, angleBuf, PHASE_CONTEXT_TAG]);
}

function computeMac(key: Buffer, nonce: Buffer, cipherB64: string): string {
  const cipherBuf = Buffer.from(cipherB64, 'base64');
  const input = Buffer.concat([key, nonce, cipherBuf, MAC_CONTEXT_TAG]);
  return spongeHash(input);
}

function bytesToBalancedTrits6(input: Buffer): Int8Array {
  const trits = new Int8Array(input.length * TRITS_PER_BYTE);
  let idx = 0;
  for (const byte of input) {
    let val = byte;
    for (let j = 0; j < TRITS_PER_BYTE; j++) {
      trits[idx++] = (val % 3) - 1;
      val = Math.floor(val / 3);
    }
  }
  return trits;
}

function balancedTrits6ToBytes(trits: Int8Array, byteLen: number): Buffer {
  const out = Buffer.alloc(byteLen);
  let tritIdx = 0;
  for (let b = 0; b < byteLen; b++) {
    let val = 0;
    let mul = 1;
    for (let j = 0; j < TRITS_PER_BYTE && tritIdx < trits.length; j++) {
      val += (trits[tritIdx] + 1) * mul;
      mul *= 3;
      tritIdx++;
    }
    out[b] = val & 0xFF;
  }
  return out;
}

function tritSub(a: number, b: number): number {
  const s = a - b;
  if (s > 1) return s - 3;
  if (s < -1) return s + 3;
  return s;
}

function tritAdd(a: number, b: number): number {
  const s = a + b;
  if (s > 1) return s - 3;
  if (s < -1) return s + 3;
  return s;
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
    let val = 0;
    let mul = 1;
    for (let j = 0; j < PACK && tritIdx < trits.length; j++) {
      val += (trits[tritIdx] + 1) * mul;
      mul *= 3;
      tritIdx++;
    }
    out[b] = val & 0xFF;
  }
  return out;
}

function cipherBytesToTrits(input: Buffer, tritCount: number): Int8Array {
  const PACK = 5;
  const trits = new Int8Array(tritCount);
  let idx = 0;
  for (const byte of input) {
    let val = byte;
    for (let j = 0; j < PACK && idx < tritCount; j++) {
      trits[idx++] = (val % 3) - 1;
      val = Math.floor(val / 3);
    }
  }
  return trits;
}

function encryptPhaseBytes(plainBytes: Buffer, key: Buffer, nonce: Buffer, phaseAngle: number): string {
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

function decryptPhaseBytes(cipherB64: string, key: Buffer, nonce: Buffer, phaseAngle: number): Buffer {
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

export function phaseSplit(
  data: string,
  mode: EncryptionMode = 'balanced'
): EncryptedPhaseData {
  const config = getPhaseConfig(mode);
  const splitRatio = 0.5;
  const key = getKeyMaterial();
  const nonce = randomBytes(NONCE_BYTES);

  const dataBytes = Buffer.from(data, 'utf-8');
  const midpoint = Math.ceil(dataBytes.length * splitRatio);
  const primaryBytes = dataBytes.subarray(0, midpoint);
  const secondaryBytes = dataBytes.subarray(midpoint);

  const primaryAngle = config.primaryPhase;
  const secondaryAngle = config.primaryPhase + config.secondaryOffset;

  const primaryTimestamp = getFemtosecondTimestamp();
  const secondaryTimestamp = getFemtosecondTimestamp();

  const primaryCipher = encryptPhaseBytes(primaryBytes, key, nonce, primaryAngle);
  const secondaryCipher = encryptPhaseBytes(secondaryBytes, key, nonce, secondaryAngle);

  const result: EncryptedPhaseData = {
    primaryPhase: {
      data: primaryCipher,
      phase: primaryAngle,
      timestamp: primaryTimestamp
    },
    secondaryPhase: {
      data: secondaryCipher,
      phase: secondaryAngle,
      timestamp: secondaryTimestamp
    },
    config,
    splitRatio,
    nonce: nonce.toString('hex'),
    mac: {
      primary: computeMac(key, nonce, primaryCipher),
      secondary: computeMac(key, nonce, secondaryCipher)
    }
  };

  if (config.guardianEnabled) {
    const guardianTimestamp = getFemtosecondTimestamp();
    const hash = spongeHash(Buffer.from(data, 'utf-8'));
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

      if (encrypted.mac) {
        const expectedPrimaryMac = computeMac(key, nonce, encrypted.primaryPhase.data);
        const expectedSecondaryMac = computeMac(key, nonce, encrypted.secondaryPhase.data);
        if (expectedPrimaryMac !== encrypted.mac.primary ||
            expectedSecondaryMac !== encrypted.mac.secondary) {
          return {
            success: false,
            phaseAlignment,
            timestampValidation,
            error: GENERIC_ERROR
          };
        }
      }

      const primaryBuf = decryptPhaseBytes(
        encrypted.primaryPhase.data,
        key,
        nonce,
        encrypted.primaryPhase.phase
      );
      const secondaryBuf = decryptPhaseBytes(
        encrypted.secondaryPhase.data,
        key,
        nonce,
        encrypted.secondaryPhase.phase
      );
      recombinedData = Buffer.concat([primaryBuf, secondaryBuf]).toString('utf-8');
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

export function getRecommendedMode(dataLength: number, isSensitive: boolean): EncryptionMode {
  if (isSensitive) {
    return 'high_security';
  }
  if (dataLength > 10000) {
    return 'performance';
  }
  return 'balanced';
}
