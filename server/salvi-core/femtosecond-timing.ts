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
 * Salvi Framework — High-Precision Timing
 *
 * Clock hierarchy (HPTP spec):
 *   Tier 0  Optical atomic clock       → femtosecond  (10⁻¹⁵ s)
 *   Tier 1  PTP / White-Rabbit         → nanosecond   (10⁻⁹ s)
 *   Tier 2  OS monotonic + wall anchor → nanosecond   (10⁻⁹ s, ±µs jitter)
 *
 * This software implementation is Tier 2:
 *   • process.hrtime.bigint() provides monotonic nanoseconds
 *   • Anchored to wall-clock (Date.now()) at process start
 *   • Real precision: nanosecond.  Practical jitter: ~1–50 µs (OS scheduler)
 *   • Sub-nanosecond tiers (ps, fs) are ZERO — no fake entropy injected
 *
 * The response explicitly declares the clock tier and measured precision
 * so consumers know what they're getting vs. what the full HPTP spec targets.
 */

// Salvi Epoch: April 1, 2025 — Day Zero
export const SALVI_EPOCH = new Date('2025-04-01T00:00:00.000Z').getTime();
export const SALVI_EPOCH_NS = BigInt(SALVI_EPOCH) * 1_000_000n;

export const NANOSECONDS_PER_SECOND  = 1_000_000_000n;
export const NANOSECONDS_PER_MS      = 1_000_000n;

// Legacy export kept for any callers using the femtosecond constant
export const FEMTOSECONDS_PER_SECOND = 1_000_000_000_000_000n;

export interface HighPrecisionTimestamp {
  nanoseconds: bigint;
  unix_seconds: number;
  sub_second: {
    milliseconds: number;
    microseconds: number;
    nanoseconds: number;
  };
  humanReadable: string;
  isoDate: string;
  salviEpochOffset_ns: bigint;
  clockTier: number;
  measuredPrecision: string;
}

export interface TimingMetrics {
  timestamp: HighPrecisionTimestamp;
  clockSource: string;
  clockTier: number;
  synchronizationStatus: 'free-running' | 'ntp-disciplined';
  estimatedJitter: string;
}

/**
 * Anchor hrtime to wall-clock at process start.
 *
 * process.hrtime.bigint() returns monotonic nanoseconds since an
 * arbitrary reference (process boot). By capturing both Date.now()
 * and hrtime at startup we derive wall-clock nanoseconds for any
 * later hrtime reading:
 *
 *   wall_ns = anchorWallNs + (hrtime_now − anchorHrNs)
 *
 * Sub-millisecond digits (µs, ns) come from the monotonic clock.
 * Below nanosecond the digits are zero — no fake data.
 */
const _anchorWallMs  = Date.now();
const _anchorHrNs    = process.hrtime.bigint();
const _anchorWallNs  = BigInt(_anchorWallMs) * NANOSECONDS_PER_MS;

export function getHighPrecisionTimestamp(): HighPrecisionTimestamp {
  const hrNow  = process.hrtime.bigint();
  const wallNs = _anchorWallNs + (hrNow - _anchorHrNs);

  const unixSec = Number(wallNs / NANOSECONDS_PER_SECOND);
  const subNs   = wallNs % NANOSECONDS_PER_SECOND;
  const ms = Number(subNs / NANOSECONDS_PER_MS);
  const us = Number((subNs % NANOSECONDS_PER_MS) / 1_000n);
  const ns = Number(subNs % 1_000n);

  const wallMs = Number(wallNs / NANOSECONDS_PER_MS);
  const date   = new Date(wallMs);

  return {
    nanoseconds: wallNs,
    unix_seconds: unixSec,
    sub_second: { milliseconds: ms, microseconds: us, nanoseconds: ns },
    humanReadable: formatNanoseconds(wallNs),
    isoDate: date.toISOString(),
    salviEpochOffset_ns: wallNs - SALVI_EPOCH_NS,
    clockTier: 2,
    measuredPrecision: 'nanosecond',
  };
}

// Keep old name as alias so existing internal callers don't break
export function getFemtosecondTimestamp(): HighPrecisionTimestamp {
  return getHighPrecisionTimestamp();
}

function formatNanoseconds(totalNs: bigint): string {
  const wallMs = Number(totalNs / NANOSECONDS_PER_MS);
  const date = new Date(wallMs);

  const subNs = totalNs % NANOSECONDS_PER_SECOND;
  const ms = Number(subNs / NANOSECONDS_PER_MS);
  const us = Number((subNs % NANOSECONDS_PER_MS) / 1_000n);
  const ns = Number(subNs % 1_000n);

  const y   = date.getUTCFullYear();
  const mon = String(date.getUTCMonth() + 1).padStart(2, '0');
  const d   = String(date.getUTCDate()).padStart(2, '0');
  const h   = String(date.getUTCHours()).padStart(2, '0');
  const min = String(date.getUTCMinutes()).padStart(2, '0');
  const s   = String(date.getUTCSeconds()).padStart(2, '0');

  return `${y}-${mon}-${d} ${h}:${min}:${s}.${String(ms).padStart(3,'0')}.${String(us).padStart(3,'0')}.${String(ns).padStart(3,'0')} UTC`;
}

/**
 * Calculate duration between two timestamps
 */
export function calculateDuration(start: HighPrecisionTimestamp, end: HighPrecisionTimestamp): {
  nanoseconds: bigint;
  microseconds: number;
  milliseconds: number;
  humanReadable: string;
} {
  const durationNs = end.nanoseconds - start.nanoseconds;
  return {
    nanoseconds: durationNs,
    microseconds: Number(durationNs / 1_000n),
    milliseconds: Number(durationNs / NANOSECONDS_PER_MS),
    humanReadable: formatDuration(durationNs),
  };
}

function formatDuration(ns: bigint): string {
  if (ns < 1_000n)                return `${ns}ns`;
  if (ns < NANOSECONDS_PER_MS)    return `${Number(ns / 1_000n)}µs`;
  if (ns < NANOSECONDS_PER_SECOND) return `${Number(ns / NANOSECONDS_PER_MS)}ms`;
  return `${Number(ns / NANOSECONDS_PER_SECOND)}s`;
}

export function getTimingMetrics(): TimingMetrics {
  return {
    timestamp: getHighPrecisionTimestamp(),
    clockSource: 'process.hrtime.bigint()',
    clockTier: 2,
    synchronizationStatus: 'free-running',
    estimatedJitter: '±1–50µs (OS scheduler dependent)',
  };
}

/**
 * Validate timestamp pair is within acceptable recombination window
 */
export function validateRecombinationWindow(
  primary: HighPrecisionTimestamp,
  secondary: HighPrecisionTimestamp,
  toleranceNs: bigint = 100_000n
): {
  valid: boolean;
  offset_ns: bigint;
  tolerance_ns: bigint;
} {
  const offset = primary.nanoseconds > secondary.nanoseconds
    ? primary.nanoseconds - secondary.nanoseconds
    : secondary.nanoseconds - primary.nanoseconds;

  return {
    valid: offset < toleranceNs,
    offset_ns: offset,
    tolerance_ns: toleranceNs,
  };
}

export function generateTimestampBatch(count: number): HighPrecisionTimestamp[] {
  const timestamps: HighPrecisionTimestamp[] = [];
  for (let i = 0; i < count; i++) {
    timestamps.push(getHighPrecisionTimestamp());
  }
  return timestamps;
}

/**
 * HPTP Latency Correction — NTP-Symmetric Four-Timestamp Model
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
  correctionNanoseconds: bigint;
  protocol: string;
}

export function computeHptpCorrection(params: HptpCorrectionParams): HptpCorrectionResult {
  const { t1_client_send_ms, t2_server_receive_ms, t3_server_send_ms, t4_client_receive_ms } = params;
  const roundTrip = (t4_client_receive_ms - t1_client_send_ms) - (t3_server_send_ms - t2_server_receive_ms);
  const oneWay = roundTrip / 2;
  const clockOffset = ((t2_server_receive_ms - t1_client_send_ms) + (t3_server_send_ms - t4_client_receive_ms)) / 2;
  const correctionNs = BigInt(Math.round(oneWay * 1_000_000));

  return {
    roundTripDelayMs: roundTrip,
    oneWayDelayMs: oneWay,
    clockOffsetMs: clockOffset,
    correctionNanoseconds: correctionNs,
    protocol: 'HPTP/1.0',
  };
}

export function getSalviEpochAnchorPoints() {
  const now = getHighPrecisionTimestamp();
  return {
    salviEpoch: new Date(SALVI_EPOCH).toISOString(),
    currentTime: now.isoDate,
    offsetFromEpoch_ns: now.salviEpochOffset_ns.toString(),
    clockTier: 2,
  };
}
