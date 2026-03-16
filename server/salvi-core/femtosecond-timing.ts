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
 * Salvi Framework — Femtosecond Timing
 *
 * All values resolve to femtosecond (10⁻¹⁵ s) mathematical precision
 * so the data structures are ready for Tier 0 atomic clock sources.
 *
 * Clock hierarchy (HPTP spec):
 *   Tier 0  Optical atomic clock       → fs measured   (target)
 *   Tier 1  PTP / White-Rabbit         → ns measured, ps/fs = 0
 *   Tier 2  OS monotonic + wall anchor → ns measured, ps/fs = 0
 *
 * Current implementation: Tier 2
 *   • process.hrtime.bigint() → monotonic nanoseconds
 *   • Anchored to wall-clock (Date.now()) at startup
 *   • Measured tiers: ms ✓  µs ✓  ns ✓
 *   • Awaiting hardware: ps ·  fs ·  (zero until clock source provides them)
 *
 * When a Tier 0/1 clock source is paired, only the clock-read function
 * changes — all downstream math, formats, and APIs already resolve
 * to femtosecond granularity.
 */

// Salvi Epoch: April 1, 2025 — Day Zero
export const SALVI_EPOCH = new Date('2025-04-01T00:00:00.000Z').getTime();
export const SALVI_EPOCH_FS = BigInt(SALVI_EPOCH) * 1_000_000_000_000n;

export const FEMTOSECONDS_PER_MILLISECOND = 1_000_000_000_000n;
export const FEMTOSECONDS_PER_SECOND      = 1_000_000_000_000_000n;
export const FEMTOSECONDS_PER_NANOSECOND  = 1_000_000n;

export interface FemtosecondTimestamp {
  femtoseconds: bigint;
  humanReadable: string;
  isoDate: string;
  precision: 'femtosecond';
  salviEpochOffset: bigint;
  clockTier: number;
  measured: string;
}

export interface TimingMetrics {
  timestamp: FemtosecondTimestamp;
  clockSource: string;
  clockTier: number;
  synchronizationStatus: string;
  estimatedAccuracy: string;
  measuredTiers: string;
}

/**
 * Anchor hrtime to wall-clock at process start.
 *
 * process.hrtime.bigint() → monotonic nanoseconds since process boot.
 * We capture Date.now() + hrtime together at startup so any later
 * hrtime reading maps to wall-clock nanoseconds:
 *
 *   wall_ns = anchorWallNs + (hrtime_now − anchorHrNs)
 *
 * Then scale to femtoseconds:  wall_fs = wall_ns × 10⁶
 *
 * The ns→fs multiplication preserves the measurement — it does NOT
 * inject fake sub-nanosecond data. The ps/fs digits are 0 until a
 * Tier 0 clock source provides them.
 */
const _anchorWallMs  = Date.now();
const _anchorHrNs    = process.hrtime.bigint();
const _anchorWallNs  = BigInt(_anchorWallMs) * 1_000_000n;

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
    salviEpochOffset: wallFs - SALVI_EPOCH_FS,
    clockTier: 2,
    measured: 'ms.µs.ns (ps.fs awaiting Tier 0 clock)',
  };
}

function formatFemtoseconds(fs: bigint): string {
  const milliseconds = Number(fs / FEMTOSECONDS_PER_MILLISECOND);
  const date = new Date(milliseconds);

  const remainingFs = fs % FEMTOSECONDS_PER_MILLISECOND;
  const microseconds = remainingFs / 1_000_000_000n;
  const nanoseconds  = (remainingFs % 1_000_000_000n) / 1_000_000n;
  const picoseconds  = (remainingFs % 1_000_000n) / 1_000n;
  const femtoseconds = remainingFs % 1_000n;

  const y   = date.getUTCFullYear();
  const mon = String(date.getUTCMonth() + 1).padStart(2, '0');
  const d   = String(date.getUTCDate()).padStart(2, '0');
  const h   = String(date.getUTCHours()).padStart(2, '0');
  const min = String(date.getUTCMinutes()).padStart(2, '0');
  const s   = String(date.getUTCSeconds()).padStart(2, '0');
  const msS = String(date.getUTCMilliseconds()).padStart(3, '0');
  const usS = String(microseconds).padStart(3, '0');
  const nsS = String(nanoseconds).padStart(3, '0');
  const psS = String(picoseconds).padStart(3, '0');
  const fsS = String(femtoseconds).padStart(3, '0');

  return `${y}-${mon}-${d} ${h}:${min}:${s}.${msS}.${usS}.${nsS}.${psS}.${fsS} UTC`;
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
  const durationNs = Number(durationFs / FEMTOSECONDS_PER_NANOSECOND);
  const durationUs = Number(durationFs / 1_000_000_000n);
  const durationMs = Number(durationFs / FEMTOSECONDS_PER_MILLISECOND);

  return {
    femtoseconds: durationFs,
    nanoseconds: durationNs,
    microseconds: durationUs,
    milliseconds: durationMs,
    humanReadable: formatDuration(durationFs),
  };
}

function formatDuration(fs: bigint): string {
  if (fs < FEMTOSECONDS_PER_NANOSECOND) return `${fs}fs`;
  if (fs < 1_000_000_000n)              return `${Number(fs / FEMTOSECONDS_PER_NANOSECOND)}ns`;
  if (fs < FEMTOSECONDS_PER_MILLISECOND) return `${Number(fs / 1_000_000_000n)}µs`;
  if (fs < FEMTOSECONDS_PER_SECOND)      return `${Number(fs / FEMTOSECONDS_PER_MILLISECOND)}ms`;
  return `${Number(fs / FEMTOSECONDS_PER_SECOND)}s`;
}

export function getTimingMetrics(): TimingMetrics {
  return {
    timestamp: getFemtosecondTimestamp(),
    clockSource: 'process.hrtime.bigint() + Date.now() anchor',
    clockTier: 2,
    synchronizationStatus: 'free-running (Tier 2 — OS monotonic)',
    estimatedAccuracy: '±1–50µs (OS scheduler jitter)',
    measuredTiers: 'ms ✓ | µs ✓ | ns ✓ | ps · | fs · (awaiting Tier 0)',
  };
}

/**
 * Validate timestamp pair is within acceptable recombination window
 * As per whitepaper: |τₚ - τₛ| < tolerance
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
    tolerance: toleranceFs,
  };
}

export function generateTimestampBatch(count: number): FemtosecondTimestamp[] {
  const timestamps: FemtosecondTimestamp[] = [];
  for (let i = 0; i < count; i++) {
    timestamps.push(getFemtosecondTimestamp());
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

export function getSalviEpochAnchorPoints() {
  const now = getFemtosecondTimestamp();
  return {
    salviEpoch: new Date(SALVI_EPOCH).toISOString(),
    currentTime: now.isoDate,
    offsetFromEpoch_fs: now.salviEpochOffset.toString(),
    clockTier: now.clockTier,
    measured: now.measured,
  };
}
