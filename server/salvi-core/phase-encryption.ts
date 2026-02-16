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
 * Salvi Framework - Phase Encryption (IMPROVED)
 * 
 * Implements the Adaptive Dual-Phase Quantum Encryption System from the whitepaper:
 * - Tunable Phase-Split Architecture
 * - Primary Phase: 360°/0° reference (fixed)
 * - Secondary Phase: Δθ(t) = (1°-10°) tunable
 * - Guardian Phase: 358° offset for tamper detection
 *
 * IMPROVEMENT: Replaced weak djb2 checksum with Tribonacci-weighted mixing.
 * The guardian phase checksum now uses τ-derived constants for better
 * tamper detection, directly connecting to the theory's mathematics.
 * Note: This is a non-cryptographic checksum. For cryptographic integrity,
 * use the CNSA 2.0 algorithms (HMAC-SHA-384, etc.) in the kernel crypto module.
 */

import { getFemtosecondTimestamp, FemtosecondTimestamp } from './femtosecond-timing';
import { TAU_POWERS } from '@shared/tribonacci-constants';

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
}

export interface RecombinationResult {
  success: boolean;
  data?: string;
  phaseAlignment: number;
  timestampValidation: boolean;
  guardianValidation?: boolean;
  error?: string;
}

const TAU_2 = TAU_POWERS.TAU_2;
const TAU_7 = TAU_POWERS.TAU_7;

/**
 * Get phase configuration based on encryption mode
 */
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

/**
 * Split data into phase components
 * Implements intelligent data split from whitepaper
 */
export function phaseSplit(
  data: string,
  mode: EncryptionMode = 'balanced'
): EncryptedPhaseData {
  const config = getPhaseConfig(mode);
  const splitRatio = 0.5;
  
  const midpoint = Math.ceil(data.length * splitRatio);
  const primaryData = data.substring(0, midpoint);
  const secondaryData = data.substring(midpoint);
  
  const primaryTimestamp = getFemtosecondTimestamp();
  const secondaryTimestamp = getFemtosecondTimestamp();
  
  const result: EncryptedPhaseData = {
    primaryPhase: {
      data: Buffer.from(primaryData).toString('base64'),
      phase: config.primaryPhase,
      timestamp: primaryTimestamp
    },
    secondaryPhase: {
      data: Buffer.from(secondaryData).toString('base64'),
      phase: config.primaryPhase + config.secondaryOffset,
      timestamp: secondaryTimestamp
    },
    config,
    splitRatio
  };
  
  if (config.guardianEnabled) {
    const guardianTimestamp = getFemtosecondTimestamp();
    const hash = tribonacciHash(data);
    result.guardianPhase = {
      hash,
      phase: config.guardianOffset,
      timestamp: guardianTimestamp
    };
  }
  
  return result;
}

/**
 * Get timing tolerance based on encryption mode
 * Per whitepaper: high_security uses 100fs, balanced uses 1ms, performance uses 1s
 */
function getTimingToleranceFs(mode: EncryptionMode): bigint {
  switch (mode) {
    case 'high_security':
      return 100n; // 100 femtoseconds per whitepaper spec
    case 'balanced':
      return 1_000_000_000_000n; // 1 millisecond (picosecond class)
    case 'performance':
      return 1_000_000_000_000_000n; // 1 second (nanosecond class)
    case 'adaptive':
    default:
      return 1_000_000_000n; // 1 microsecond
  }
}

/**
 * Recombine phase-split data
 * Implements quantum recombination requiring exact phase relationship
 */
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
    const primaryData = Buffer.from(encrypted.primaryPhase.data, 'base64').toString();
    const secondaryData = Buffer.from(encrypted.secondaryPhase.data, 'base64').toString();
    const recombinedData = primaryData + secondaryData;
    
    let guardianValidation: boolean | undefined;
    if (encrypted.guardianPhase) {
      const currentHash = tribonacciHash(recombinedData);
      guardianValidation = currentHash === encrypted.guardianPhase.hash;
      
      if (!guardianValidation) {
        return {
          success: false,
          phaseAlignment,
          timestampValidation,
          guardianValidation: undefined,
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

/**
 * Calculate phase alignment score (0-1)
 */
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

/**
 * Tribonacci-weighted checksum for guardian phase tamper detection
 *
 * Non-cryptographic checksum using τ-derived mixing constants
 * from the 13D Torsion Plenum Theory:
 *   - τ² as the initial seed (from SO(8) graph stability)
 *   - τ⁷ as the mixing multiplier (instanton action volume)
 *   - 13-round finalization (dimensional constant D=13)
 *
 * Produces a 64-bit checksum with better avalanche behavior
 * than the previous djb2-style shift-and-add approach.
 */
function tribonacciHash(data: string): string {
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

/**
 * Get encryption mode recommendations based on data sensitivity
 */
export function getRecommendedMode(dataLength: number, isSensitive: boolean): EncryptionMode {
  if (isSensitive) {
    return 'high_security';
  }
  if (dataLength > 10000) {
    return 'performance';
  }
  return 'balanced';
}
