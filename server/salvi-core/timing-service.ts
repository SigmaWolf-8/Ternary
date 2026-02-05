/**
 * Femtosecond Timing Service API
 * HPTP (Hierarchical Precision Time Protocol) Implementation
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

/**
 * Clock Source Type
 */
export type ClockSourceType = 
  | 'optical_atomic_clock'
  | 'cesium_atomic_clock'
  | 'gps_disciplined'
  | 'rubidium_oscillator'
  | 'crystal_oscillator';

/**
 * Synchronization Status
 */
export type SyncStatus = 'synchronized' | 'syncing' | 'unsynchronized' | 'holdover';

/**
 * Certification Level
 */
export type CertificationLevel = 'financial' | 'regulatory' | 'standard' | 'best_effort';

/**
 * Timestamp Components
 */
export interface TimestampComponents {
  iso8601: string;
  unix_nanoseconds: number;
  femtoseconds: bigint;
  femtoseconds_since_epoch: bigint;
}

/**
 * Clock Source Information
 */
export interface ClockSourceInfo {
  type: ClockSourceType;
  id: string;
  accuracy_fs: number;
  uncertainty_fs: number;
  traceability: string;
}

/**
 * Synchronization Information
 */
export interface SynchronizationInfo {
  status: SyncStatus;
  offset_from_master_fs: number;
  drift_rate_fs_per_second: number;
  last_sync_at: string;
}

/**
 * Leap Second Information
 */
export interface LeapSecondInfo {
  current_offset: number;
  next_scheduled: string | null;
}

/**
 * Protocol Metadata
 */
export interface TimingProtocolMetadata {
  protocol: 'HPTP' | 'PTP' | 'NTP';
  version: string;
  leap_second_info: LeapSecondInfo;
}

/**
 * Current Timestamp Response
 * GET /api/timing/v1/current
 */
export interface CurrentTimestampResponse {
  timestamp: TimestampComponents;
  source: ClockSourceInfo;
  synchronization: SynchronizationInfo;
  metadata: TimingProtocolMetadata;
}

/**
 * Timing Event
 */
export interface TimingEvent {
  name: string;
  timestamp_fs: bigint;
  source: string;
  uncertainty_fs: number;
}

/**
 * Certification Requirements
 */
export interface CertificationRequirements {
  max_total_uncertainty_fs: number;
  required_sources: ClockSourceType[];
  certification_level: CertificationLevel;
}

/**
 * Timing Certification Request
 * POST /api/timing/v1/certify
 */
export interface TimingCertificationRequest {
  operation_id: string;
  events: TimingEvent[];
  requirements: CertificationRequirements;
}

/**
 * Certification Status
 */
export interface CertificationStatus {
  id: string;
  operation_id: string;
  status: 'certified' | 'pending' | 'rejected';
  level: CertificationLevel;
  valid_from: string;
  valid_until: string;
}

/**
 * Certification Metrics
 */
export interface CertificationMetrics {
  total_uncertainty_fs: number;
  worst_case_error_fs: number;
  meets_requirements: boolean;
}

/**
 * Timestamp Proof
 */
export interface TimestampProof {
  hedera_tx_id: string;
  consensus_timestamp: string;
}

/**
 * Certification Proof
 */
export interface CertificationProof {
  merkle_root: string;
  inclusion_proof: string[];
  timestamp_proof: TimestampProof;
}

/**
 * Certification Signature
 */
export interface CertificationSignature {
  signer: string;
  algorithm: 'Ed25519' | 'RSA' | 'ECDSA';
  signature: string;
  timestamp_fs: bigint;
}

/**
 * Timing Certification Response
 */
export interface TimingCertificationResponse {
  certification: CertificationStatus;
  metrics: CertificationMetrics;
  proof: CertificationProof;
  signatures: CertificationSignature[];
}

/**
 * Salvi Epoch Constants
 * Base epoch: April 1, 2025 00:00:00.000 UTC
 */
export const SALVI_EPOCH = {
  date: new Date('2025-04-01T00:00:00.000Z'),
  unix_ms: 1743465600000,
  unix_ns: 1743465600000000000n,
  unix_fs: 1743465600000000000000000n
} as const;

/**
 * Timing Constants
 */
export const TIMING_CONSTANTS = {
  NANOSECONDS_PER_SECOND: 1_000_000_000n,
  FEMTOSECONDS_PER_NANOSECOND: 1_000_000n,
  FEMTOSECONDS_PER_SECOND: 1_000_000_000_000_000n,
  MAX_CLOCK_DRIFT_FS_PER_SECOND: 1000,
  DEFAULT_SYNC_INTERVAL_MS: 100
} as const;

/**
 * Get current femtosecond timestamp
 * Note: JavaScript cannot achieve true femtosecond precision,
 * this is a simulation for API compatibility
 */
export function getCurrentFemtosecondTimestamp(): TimestampComponents {
  const now = Date.now();
  const unixNs = BigInt(now) * 1_000_000n;
  const unixFs = unixNs * 1_000_000n;
  const fsSinceEpoch = unixFs - SALVI_EPOCH.unix_fs;
  
  return {
    iso8601: new Date(now).toISOString(),
    unix_nanoseconds: Number(unixNs),
    femtoseconds: unixFs,
    femtoseconds_since_epoch: fsSinceEpoch
  };
}

/**
 * Calculate timing uncertainty
 */
export function calculateTotalUncertainty(events: TimingEvent[]): number {
  return events.reduce((sum, event) => sum + event.uncertainty_fs, 0);
}

/**
 * Check if timing meets certification requirements
 */
export function meetsRequirements(
  events: TimingEvent[],
  requirements: CertificationRequirements
): boolean {
  const totalUncertainty = calculateTotalUncertainty(events);
  return totalUncertainty <= requirements.max_total_uncertainty_fs;
}

/**
 * Create current timestamp response
 */
export function createCurrentTimestampResponse(
  sourceType: ClockSourceType = 'optical_atomic_clock'
): CurrentTimestampResponse {
  const timestamp = getCurrentFemtosecondTimestamp();
  
  return {
    timestamp,
    source: {
      type: sourceType,
      id: `clock_${sourceType}_001`,
      accuracy_fs: sourceType === 'optical_atomic_clock' ? 10 : 1000,
      uncertainty_fs: sourceType === 'optical_atomic_clock' ? 5 : 500,
      traceability: 'UTC(NIST) via GPS'
    },
    synchronization: {
      status: 'synchronized',
      offset_from_master_fs: 15,
      drift_rate_fs_per_second: 0.001,
      last_sync_at: new Date(Date.now() - 1000).toISOString()
    },
    metadata: {
      protocol: 'HPTP',
      version: '1.0',
      leap_second_info: {
        current_offset: 37,
        next_scheduled: '2026-06-30T23:59:60Z'
      }
    }
  };
}

/**
 * Create timing certification response
 */
export function createCertificationResponse(
  request: TimingCertificationRequest,
  hederaTxId: string
): TimingCertificationResponse {
  const totalUncertainty = calculateTotalUncertainty(request.events);
  const meetsReqs = totalUncertainty <= request.requirements.max_total_uncertainty_fs;
  
  const now = new Date();
  const validUntil = new Date(now.getTime() + 10000);
  
  return {
    certification: {
      id: `time_cert_${Date.now().toString(36)}`,
      operation_id: request.operation_id,
      status: meetsReqs ? 'certified' : 'rejected',
      level: request.requirements.certification_level,
      valid_from: now.toISOString(),
      valid_until: validUntil.toISOString()
    },
    metrics: {
      total_uncertainty_fs: totalUncertainty,
      worst_case_error_fs: totalUncertainty * 2,
      meets_requirements: meetsReqs
    },
    proof: {
      merkle_root: `proof_${Date.now().toString(16)}`,
      inclusion_proof: ['hash1', 'hash2', 'hash3'],
      timestamp_proof: {
        hedera_tx_id: hederaTxId,
        consensus_timestamp: now.toISOString()
      }
    },
    signatures: [{
      signer: 'salvi_timing_authority',
      algorithm: 'Ed25519',
      signature: `sig_${Date.now().toString(36)}`,
      timestamp_fs: getCurrentFemtosecondTimestamp().femtoseconds
    }]
  };
}

/**
 * Timing Service Interface
 */
export interface ITimingService {
  getCurrentTimestamp(): CurrentTimestampResponse;
  certifyTiming(request: TimingCertificationRequest): Promise<TimingCertificationResponse>;
  getSyncStatus(): SynchronizationInfo;
  calibrate(): Promise<boolean>;
}
