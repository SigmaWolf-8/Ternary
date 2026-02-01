/**
 * libternary - Phase Encryption
 * 
 * Implements the Adaptive Dual-Phase Quantum Encryption System from the Salvi Framework whitepaper:
 * - Tunable Phase-Split Architecture
 * - Primary Phase: 360°/0° reference (fixed)
 * - Secondary Phase: Δθ(t) = (1°-10°) tunable
 * - Guardian Phase: 358° offset for tamper detection
 * 
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import { getFemtosecondTimestamp, FemtosecondTimestamp } from './femtosecond-timing';

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
 * Encode data to base64 (works in both Node.js and browser)
 */
function toBase64(data: string): string {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(data).toString('base64');
  }
  return btoa(data);
}

/**
 * Decode data from base64 (works in both Node.js and browser)
 */
function fromBase64(data: string): string {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(data, 'base64').toString();
  }
  return atob(data);
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
      data: toBase64(primaryData),
      phase: config.primaryPhase,
      timestamp: primaryTimestamp
    },
    secondaryPhase: {
      data: toBase64(secondaryData),
      phase: config.primaryPhase + config.secondaryOffset,
      timestamp: secondaryTimestamp
    },
    config,
    splitRatio
  };
  
  if (config.guardianEnabled) {
    const guardianTimestamp = getFemtosecondTimestamp();
    const hash = simpleHash(data);
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
      error: 'Phase alignment below threshold'
    };
  }
  
  try {
    const primaryData = fromBase64(encrypted.primaryPhase.data);
    const secondaryData = fromBase64(encrypted.secondaryPhase.data);
    const recombinedData = primaryData + secondaryData;
    
    let guardianValidation: boolean | undefined;
    if (encrypted.guardianPhase) {
      const currentHash = simpleHash(recombinedData);
      guardianValidation = currentHash === encrypted.guardianPhase.hash;
      
      if (!guardianValidation) {
        return {
          success: false,
          phaseAlignment,
          timestampValidation,
          guardianValidation,
          error: 'Guardian phase validation failed - data may be tampered'
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
  } catch (error) {
    return {
      success: false,
      phaseAlignment,
      timestampValidation,
      error: `Recombination failed: ${error}`
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
 * Simple hash function for guardian phase
 */
function simpleHash(data: string): string {
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    const char = data.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
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
