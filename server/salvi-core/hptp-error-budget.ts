import { getFemtosecondTimestamp, FemtosecondTimestamp } from './femtosecond-timing';

interface DriftSample {
  timestamp: number;
  hrtimeDeltaNs: number;
  wallDeltaMs: number;
  driftPpm: number;
  monotonic: boolean;
  jitterFs: number;
}

interface ErrorBudgetState {
  started: number;
  samples: DriftSample[];
  violations: DriftViolation[];
  lastSampleTime: number;
  cumulativeDriftPpm: number;
  maxDriftPpm: number;
  totalSamples: number;
  monotonicViolations: number;
}

interface DriftViolation {
  timestamp: number;
  driftPpm: number;
  threshold: string;
  severity: 'warning' | 'critical';
}

const FINRA_613_THRESHOLD_US = 50;
const MIFID_II_HFT_THRESHOLD_US = 100;
const MIFID_II_STANDARD_THRESHOLD_MS = 1;

const MAX_DRIFT_PPM_WARNING = 10;
const MAX_DRIFT_PPM_CRITICAL = 50;

const MAX_SAMPLES_RETAINED = 2880;
const SAMPLE_INTERVAL_MS = 60_000;

let state: ErrorBudgetState = {
  started: Date.now(),
  samples: [],
  violations: [],
  lastSampleTime: 0,
  cumulativeDriftPpm: 0,
  maxDriftPpm: 0,
  totalSamples: 0,
  monotonicViolations: 0,
};

let intervalHandle: ReturnType<typeof setInterval> | null = null;

let prevHrtime: bigint | null = null;
let prevWall: number | null = null;

function collectDriftSample(): DriftSample {
  const wallNow = Date.now();
  const hrNow = process.hrtime.bigint();

  let driftPpm = 0;
  let hrtimeDeltaNs = 0;
  let wallDeltaMs = 0;

  if (prevHrtime !== null && prevWall !== null) {
    hrtimeDeltaNs = Number(hrNow - prevHrtime);
    wallDeltaMs = wallNow - prevWall;
    const wallDeltaNs = wallDeltaMs * 1_000_000;

    if (wallDeltaNs > 1_000_000 && hrtimeDeltaNs > 1_000_000) {
      const ratio = hrtimeDeltaNs / wallDeltaNs;
      driftPpm = Math.abs(ratio - 1.0) * 1_000_000;
    }
  }

  prevHrtime = hrNow;
  prevWall = wallNow;

  const sampleCount = 100;
  const timestamps: bigint[] = [];
  for (let i = 0; i < sampleCount; i++) {
    const ts = getFemtosecondTimestamp();
    timestamps.push(ts.femtoseconds);
  }

  let monotonic = true;
  let maxJitter = 0n;
  const deltas: bigint[] = [];
  for (let i = 1; i < sampleCount; i++) {
    const delta = timestamps[i] - timestamps[i - 1];
    deltas.push(delta);
    if (delta < 0n) {
      monotonic = false;
    }
  }

  const nonZeroDeltas = deltas.filter(d => d > 0n);
  if (nonZeroDeltas.length > 0) {
    const sorted = [...nonZeroDeltas].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
    const p99Index = Math.min(Math.floor(sorted.length * 0.99), sorted.length - 1);
    maxJitter = sorted[p99Index];
  }

  return {
    timestamp: wallNow,
    hrtimeDeltaNs,
    wallDeltaMs,
    driftPpm: Math.round(driftPpm * 100) / 100,
    monotonic,
    jitterFs: Number(maxJitter),
  };
}

function recordSample() {
  const sample = collectDriftSample();

  state.samples.push(sample);
  if (state.samples.length > MAX_SAMPLES_RETAINED) {
    state.samples = state.samples.slice(-MAX_SAMPLES_RETAINED);
  }

  state.totalSamples++;
  state.lastSampleTime = sample.timestamp;

  if (sample.driftPpm > state.maxDriftPpm) {
    state.maxDriftPpm = sample.driftPpm;
  }

  const recentSamples = state.samples.slice(-60);
  state.cumulativeDriftPpm = recentSamples.reduce((sum, s) => sum + s.driftPpm, 0) / recentSamples.length;

  if (!sample.monotonic) {
    state.monotonicViolations++;
  }

  if (sample.driftPpm > MAX_DRIFT_PPM_CRITICAL) {
    state.violations.push({
      timestamp: sample.timestamp,
      driftPpm: sample.driftPpm,
      threshold: 'CRITICAL (>50 ppm)',
      severity: 'critical',
    });
  } else if (sample.driftPpm > MAX_DRIFT_PPM_WARNING) {
    state.violations.push({
      timestamp: sample.timestamp,
      driftPpm: sample.driftPpm,
      threshold: 'WARNING (>10 ppm)',
      severity: 'warning',
    });
  }

  if (state.violations.length > 500) {
    state.violations = state.violations.slice(-500);
  }
}

export function startErrorBudgetMonitor(intervalMs: number = SAMPLE_INTERVAL_MS) {
  if (intervalHandle !== null) {
    return;
  }

  recordSample();

  intervalHandle = setInterval(() => {
    try {
      recordSample();
    } catch (err) {
      console.error('[HPTP Error Budget] Sample collection failed:', err);
    }
  }, intervalMs);

  if (typeof intervalHandle === 'object' && 'unref' in intervalHandle) {
    (intervalHandle as any).unref();
  }

  console.log(`[HPTP Error Budget] Monitor started, sampling every ${intervalMs / 1000}s`);
}

export function stopErrorBudgetMonitor() {
  if (intervalHandle !== null) {
    clearInterval(intervalHandle);
    intervalHandle = null;
    console.log('[HPTP Error Budget] Monitor stopped');
  }
}

export function getErrorBudgetReport() {
  const now = Date.now();
  const uptimeMs = now - state.started;
  const uptimeHours = Math.round(uptimeMs / 3600000 * 100) / 100;

  const recentSamples = state.samples.slice(-60);
  const last24h = state.samples.filter(s => now - s.timestamp < 86400000);

  const avgDriftPpm = recentSamples.length > 0
    ? Math.round(recentSamples.reduce((s, d) => s + d.driftPpm, 0) / recentSamples.length * 100) / 100
    : 0;

  const avgJitterFs = recentSamples.length > 0
    ? Math.round(recentSamples.reduce((s, d) => s + d.jitterFs, 0) / recentSamples.length)
    : 0;

  const maxJitterFs = recentSamples.length > 0
    ? Math.max(...recentSamples.map(s => s.jitterFs))
    : 0;

  const driftTrendPpm = calculateDriftTrend(last24h);

  const avgDriftUs = avgDriftPpm * SAMPLE_INTERVAL_MS / 1000;
  const finra613 = avgDriftUs < FINRA_613_THRESHOLD_US;
  const mifidHft = avgDriftUs < MIFID_II_HFT_THRESHOLD_US;
  const mifidStandard = avgDriftUs < (MIFID_II_STANDARD_THRESHOLD_MS * 1000);

  const recentViolations = state.violations.filter(v => now - v.timestamp < 3600000);
  const criticalCount = recentViolations.filter(v => v.severity === 'critical').length;
  const warningCount = recentViolations.filter(v => v.severity === 'warning').length;

  let budgetStatus: 'GREEN' | 'YELLOW' | 'RED';
  if (criticalCount > 0 || state.monotonicViolations > 0) {
    budgetStatus = 'RED';
  } else if (warningCount > 2 || avgDriftPpm > MAX_DRIFT_PPM_WARNING) {
    budgetStatus = 'YELLOW';
  } else {
    budgetStatus = 'GREEN';
  }

  return {
    status: budgetStatus,
    monitoring: {
      active: intervalHandle !== null,
      uptimeHours,
      totalSamples: state.totalSamples,
      sampleIntervalMs: SAMPLE_INTERVAL_MS,
      retainedSamples: state.samples.length,
    },
    drift: {
      currentAvgPpm: avgDriftPpm,
      maxObservedPpm: Math.round(state.maxDriftPpm * 100) / 100,
      trendPpmPerHour: driftTrendPpm,
      cumulativeAvgPpm: Math.round(state.cumulativeDriftPpm * 100) / 100,
    },
    jitter: {
      currentAvgFs: avgJitterFs,
      maxObservedFs: maxJitterFs,
      avgDescription: avgJitterFs > 0
        ? `${avgJitterFs} fs (${(avgJitterFs / 1e15).toExponential(2)} s)`
        : 'sub-sample',
    },
    monotonicity: {
      violations: state.monotonicViolations,
      allMonotonic: state.monotonicViolations === 0,
    },
    compliance: {
      finra613: {
        threshold: `${FINRA_613_THRESHOLD_US} µs synchronization`,
        status: finra613 ? 'COMPLIANT' : 'AT_RISK',
        marginUs: Math.round((FINRA_613_THRESHOLD_US - avgDriftUs) * 100) / 100,
      },
      mifidII_hft: {
        threshold: `${MIFID_II_HFT_THRESHOLD_US} µs HFT gateway`,
        status: mifidHft ? 'COMPLIANT' : 'AT_RISK',
        marginUs: Math.round((MIFID_II_HFT_THRESHOLD_US - avgDriftUs) * 100) / 100,
      },
      mifidII_standard: {
        threshold: `${MIFID_II_STANDARD_THRESHOLD_MS} ms standard trading`,
        status: mifidStandard ? 'COMPLIANT' : 'AT_RISK',
        marginMs: Math.round((MIFID_II_STANDARD_THRESHOLD_MS * 1000 - avgDriftUs) / 1000 * 100) / 100,
      },
    },
    errorBudget: {
      thresholds: {
        warningPpm: MAX_DRIFT_PPM_WARNING,
        criticalPpm: MAX_DRIFT_PPM_CRITICAL,
      },
      recentViolations: {
        lastHour: { warnings: warningCount, criticals: criticalCount },
        total: state.violations.length,
      },
      last5Violations: state.violations.slice(-5).map(v => ({
        time: new Date(v.timestamp).toISOString(),
        driftPpm: v.driftPpm,
        severity: v.severity,
      })),
    },
    recentSamples: recentSamples.slice(-10).map(s => ({
      time: new Date(s.timestamp).toISOString(),
      driftPpm: s.driftPpm,
      jitterFs: s.jitterFs,
      monotonic: s.monotonic,
    })),
  };
}

function calculateDriftTrend(samples: DriftSample[]): number {
  if (samples.length < 10) return 0;

  const n = samples.length;
  let sumX = 0, sumY = 0, sumXY = 0, sumXX = 0;
  const startTime = samples[0].timestamp;

  for (let i = 0; i < n; i++) {
    const x = (samples[i].timestamp - startTime) / 3600000;
    const y = samples[i].driftPpm;
    sumX += x;
    sumY += y;
    sumXY += x * y;
    sumXX += x * x;
  }

  const denominator = n * sumXX - sumX * sumX;
  if (Math.abs(denominator) < 1e-10) return 0;

  const slope = (n * sumXY - sumX * sumY) / denominator;
  return Math.round(slope * 1000) / 1000;
}

export function resetErrorBudget() {
  state = {
    started: Date.now(),
    samples: [],
    violations: [],
    lastSampleTime: 0,
    cumulativeDriftPpm: 0,
    maxDriftPpm: 0,
    totalSamples: 0,
    monotonicViolations: 0,
  };
}
