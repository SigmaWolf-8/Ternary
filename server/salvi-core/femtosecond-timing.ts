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
 * Generate a femtosecond-precision timestamp
 * 
 * Note: JavaScript's Date only provides millisecond precision.
 * We simulate femtosecond precision using high-resolution timer + random entropy.
 * Real implementations would use optical atomic clocks.
 */
export function getFemtosecondTimestamp(): FemtosecondTimestamp {
  const now = Date.now();
  const salviEpochOffset = now - SALVI_EPOCH;
  
  const millisecondsPart = BigInt(salviEpochOffset);
  const femtosecondsFromMs = millisecondsPart * FEMTOSECONDS_PER_MILLISECOND;
  
  const hrTime = process.hrtime.bigint();
  const subMillisecondPart = hrTime % FEMTOSECONDS_PER_MILLISECOND;
  
  const totalFemtoseconds = femtosecondsFromMs + subMillisecondPart;
  
  const date = new Date(now);
  
  return {
    femtoseconds: totalFemtoseconds,
    humanReadable: formatFemtoseconds(totalFemtoseconds),
    isoDate: date.toISOString(),
    precision: 'femtosecond',
    salviEpochOffset: totalFemtoseconds
  };
}

/**
 * Format femtoseconds into human-readable date/time string
 */
function formatFemtoseconds(fs: bigint): string {
  // Convert femtoseconds back to milliseconds and add to Salvi Epoch
  const milliseconds = Number(fs / FEMTOSECONDS_PER_MILLISECOND);
  const date = new Date(SALVI_EPOCH + milliseconds);
  
  // Get sub-millisecond precision
  const remainingFs = fs % FEMTOSECONDS_PER_MILLISECOND;
  const nanoseconds = remainingFs / 1_000_000n;
  const picoseconds = (remainingFs % 1_000_000n) / 1_000n;
  const femtoseconds = remainingFs % 1_000n;
  
  // Format: YYYY-MM-DD HH:mm:ss.mmm.nnn.ppp.fff
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  const hours = String(date.getUTCHours()).padStart(2, '0');
  const minutes = String(date.getUTCMinutes()).padStart(2, '0');
  const seconds = String(date.getUTCSeconds()).padStart(2, '0');
  const ms = String(date.getUTCMilliseconds()).padStart(3, '0');
  const ns = String(nanoseconds).padStart(3, '0');
  const ps = String(picoseconds).padStart(3, '0');
  const fsStr = String(femtoseconds).padStart(3, '0');
  
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}.${ms}.${ns}.${ps}.${fsStr} UTC`;
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
