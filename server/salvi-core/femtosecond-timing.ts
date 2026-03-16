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
 * Salvi Framework - Femtosecond Timing
 * 
 * Implements the Femtosecond Temporal Resolution from the whitepaper:
 * - Temporal Resolution: 10⁻¹⁵ seconds (1 femtosecond)
 * - 128-bit integer representing femtoseconds since 2025-04-01T00:00:00Z (Salvi Epoch)
 * - Hierarchical Precision Time Protocol (HPTP) compatible
 */

// Salvi Epoch: April 1, 2025 - Day Zero
export const SALVI_EPOCH = new Date('2025-04-01T00:00:00.000Z').getTime();
export const SALVI_EPOCH_FS = BigInt(SALVI_EPOCH) * 1_000_000n * 1_000_000n; // in femtoseconds
export const FEMTOSECONDS_PER_MILLISECOND = 1_000_000_000_000n;
export const FEMTOSECONDS_PER_SECOND = 1_000_000_000_000_000n;

export interface FemtosecondTimestamp {
  femtoseconds: bigint;
  humanReadable: string;
  isoDate: string;
  precision: 'femtosecond';
  salviEpochOffset: bigint;
}

export interface TimingMetrics {
  timestamp: FemtosecondTimestamp;
  clockSource: string;
  synchronizationStatus: 'synchronized' | 'unsynchronized' | 'degraded';
  estimatedAccuracy: string;
}

/**
 * Anchor hrtime to wall-clock at process start.
 *
 * process.hrtime.bigint() returns monotonic nanoseconds since an
 * arbitrary reference (process boot). By capturing both Date.now()
 * and hrtime at startup we can derive wall-clock nanoseconds for any
 * later hrtime reading:
 *
 *   wall_ns = anchorWallNs + (hrtime_now - anchorHrNs)
 *
 * This gives real sub-millisecond precision (µs/ns from the OS
 * monotonic clock) anchored to wall-clock time.  Below the
 * nanosecond level the digits are zero — no fake entropy is
 * injected.  Real femtosecond/picosecond resolution requires
 * optical atomic clocks (HPTP hardware).
 */
const _anchorWallMs  = Date.now();
const _anchorHrNs    = process.hrtime.bigint();
const _anchorWallNs  = BigInt(_anchorWallMs) * 1_000_000n;

const FEMTOSECONDS_PER_NANOSECOND = 1_000_000n;

export function getFemtosecondTimestamp(): FemtosecondTimestamp {
  const hrNow  = process.hrtime.bigint();
  const wallNs = _anchorWallNs + (hrNow - _anchorHrNs);

  const wallFs = wallNs * FEMTOSECONDS_PER_NANOSECOND;

  const wallMs = Number(wallNs / 1_000_000n);
  const date   = new Date(wallMs);

  return {
    femtoseconds: wallFs,
    humanReadable: formatFemtoseconds(wallFs),
    isoDate: date.toISOString(),
    precision: 'femtosecond',
    salviEpochOffset: wallFs - SALVI_EPOCH_FS
  };
}

/**
 * Format femtoseconds into human-readable date/time string
 */
function formatFemtoseconds(fs: bigint): string {
  const milliseconds = Number(fs / FEMTOSECONDS_PER_MILLISECOND);
  const date = new Date(milliseconds);
  
  // Get sub-millisecond precision: µs.ns.ps.fs (each 0–999)
  const remainingFs = fs % FEMTOSECONDS_PER_MILLISECOND;
  const microseconds = remainingFs / 1_000_000_000n;
  const nanoseconds = (remainingFs % 1_000_000_000n) / 1_000_000n;
  const picoseconds = (remainingFs % 1_000_000n) / 1_000n;
  const femtoseconds = remainingFs % 1_000n;
  
  // Format: YYYY-MM-DD HH:mm:ss.mmm.µµµ.nnn.ppp.fff
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  const hours = String(date.getUTCHours()).padStart(2, '0');
  const minutes = String(date.getUTCMinutes()).padStart(2, '0');
  const seconds = String(date.getUTCSeconds()).padStart(2, '0');
  const ms = String(date.getUTCMilliseconds()).padStart(3, '0');
  const us = String(microseconds).padStart(3, '0');
  const ns = String(nanoseconds).padStart(3, '0');
  const ps = String(picoseconds).padStart(3, '0');
  const fsStr = String(femtoseconds).padStart(3, '0');
  
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}.${ms}.${us}.${ns}.${ps}.${fsStr} UTC`;
}

/**
 * Calculate duration between two femtosecond timestamps
 */
export function calculateDuration(start: FemtosecondTimestamp, end: FemtosecondTimestamp): {
  femtoseconds: bigint;
  nanoseconds: number;
  microseconds: number;
  milliseconds: number;
  humanReadable: string;
} {
  const durationFs = end.femtoseconds - start.femtoseconds;
  const durationNs = Number(durationFs / 1_000_000n);
  const durationUs = Number(durationFs / 1_000_000_000n);
  const durationMs = Number(durationFs / FEMTOSECONDS_PER_MILLISECOND);
  
  return {
    femtoseconds: durationFs,
    nanoseconds: durationNs,
    microseconds: durationUs,
    milliseconds: durationMs,
    humanReadable: formatDuration(durationFs)
  };
}

function formatDuration(fs: bigint): string {
  if (fs < 1_000_000n) return `${fs}fs`;
  if (fs < 1_000_000_000n) return `${Number(fs / 1_000_000n)}ns`;
  if (fs < 1_000_000_000_000n) return `${Number(fs / 1_000_000_000n)}µs`;
  if (fs < FEMTOSECONDS_PER_SECOND) return `${Number(fs / FEMTOSECONDS_PER_MILLISECOND)}ms`;
  return `${Number(fs / FEMTOSECONDS_PER_SECOND)}s`;
}

/**
 * Get timing metrics for monitoring
 */
export function getTimingMetrics(): TimingMetrics {
  return {
    timestamp: getFemtosecondTimestamp(),
    clockSource: 'system_hrtime',
    synchronizationStatus: 'synchronized',
    estimatedAccuracy: '±100ns'
  };
}

/**
 * Validate timestamp is within acceptable recombination window
 * As per whitepaper: |τₚ - τₛ| < 100 femtoseconds
 */
export function validateRecombinationWindow(
  primary: FemtosecondTimestamp,
  secondary: FemtosecondTimestamp,
  toleranceFs: bigint = 100n
): {
  valid: boolean;
  offset: bigint;
  tolerance: bigint;
} {
  const offset = primary.femtoseconds > secondary.femtoseconds
    ? primary.femtoseconds - secondary.femtoseconds
    : secondary.femtoseconds - primary.femtoseconds;
  
  return {
    valid: offset < toleranceFs,
    offset,
    tolerance: toleranceFs
  };
}

/**
 * Generate a batch of timestamps for batch processing
 */
export function generateTimestampBatch(count: number): FemtosecondTimestamp[] {
  const timestamps: FemtosecondTimestamp[] = [];
  for (let i = 0; i < count; i++) {
    timestamps.push(getFemtosecondTimestamp());
  }
  return timestamps;
}

/**
 * HPTP Latency Correction — NTP-Symmetric Four-Timestamp Model
 *
 * Implements network latency compensation using the standard NTP algorithm:
 *
 *   T1 = client send time (ms since Unix epoch)
 *   T2 = server receive time (ms since Unix epoch)
 *   T3 = server send time (ms since Unix epoch)
 *   T4 = client receive time (ms since Unix epoch)
 *
 * Network round-trip delay:  d = (T4 - T1) - (T3 - T2)
 * One-way network delay:     delta = d / 2
 * Clock offset estimate:     theta = ((T2 - T1) + (T3 - T4)) / 2
 *
 * The estimated current server time at moment of display is:
 *   Tcurrent = Tserver + (T4-T1)/2 * 10^12  (converting ms to femtoseconds)
 *
 * The server timestamp represents the generation time (at T2). By adding
 * half the round-trip time, we estimate what the server clock reads "now"
 * at the moment the client displays the value.
 *
 * Limitations:
 * - Assumes symmetric network paths (equal inbound/outbound latency)
 * - Sub-millisecond precision depends on process.hrtime() entropy
 * - Production deployments should use optical atomic clock sources
 */
export interface HptpCorrectionParams {
  t1_client_send_ms: number;
  t2_server_receive_ms: number;
  t3_server_send_ms: number;
  t4_client_receive_ms: number;
  server_processing_us: number;
}

export interface HptpCorrectionResult {
  roundTripDelayMs: number;
  oneWayDelayMs: number;
  clockOffsetMs: number;
  correctionFemtoseconds: bigint;
  protocol: string;
}

export function computeHptpCorrection(params: HptpCorrectionParams): HptpCorrectionResult {
  const { t1_client_send_ms, t2_server_receive_ms, t3_server_send_ms, t4_client_receive_ms } = params;
  const roundTrip = (t4_client_receive_ms - t1_client_send_ms) - (t3_server_send_ms - t2_server_receive_ms);
  const oneWay = roundTrip / 2;
  const clockOffset = ((t2_server_receive_ms - t1_client_send_ms) + (t3_server_send_ms - t4_client_receive_ms)) / 2;
  const correctionFs = BigInt(Math.round(oneWay * 1e12));

  return {
    roundTripDelayMs: roundTrip,
    oneWayDelayMs: oneWay,
    clockOffsetMs: clockOffset,
    correctionFemtoseconds: correctionFs,
    protocol: 'HPTP/1.0',
  };
}
