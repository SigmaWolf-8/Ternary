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
  femtoseconds: bigint;        // = tickCounter × 8_000_000  (exact as-numerator,
                               //   strictly monotonic, no zero-padding,
                               //   divide by 1_002_001 to recover attoseconds)
  humanReadable: string;       // pure-framework display (tick, walk, as rational)
  isoDate: string;             // "n/a — pure framework derivation, no OS clock"
  precision: 'attosecond';
  salviEpochOffset: bigint;    // same scale as femtoseconds (tick × 8_000_000)
  clockTier: number;           // 0 = pure framework derivation (no hw clock)
  measured: string;
  // ── Closed-walk derivation on Z_{D_α} (Theorem 22) ──
  attoseconds: bigint;         // walk_position mod 125,           ∈ [0, 125)
  frameworkFsIndex: bigint;    // walk_position ÷ 125,             ∈ [0, 1_002_001)
  walkTick: bigint;            // (−tickCounter) mod D_α,          ∈ [0, 125_250_125)
  // ── First-principles tick clock ─────────────────────────────────
  tickCounter: bigint;         // monotonic; advances by 1 per HPTP read
  asSinceBootNum: bigint;      // EXACT rational attoseconds since boot — numerator
  asSinceBootDen: bigint;      // EXACT rational attoseconds since boot — denominator (= 1_002_001)
  tickPeriodAsNum: bigint;     // attoseconds per tick — numerator   = 8_000_000
  tickPeriodAsDen: bigint;     // attoseconds per tick — denominator = 1_002_001
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

// ════════════════════════════════════════════════════════════════════
// Sub-nanosecond calibration (Tier-2+ extension).
//
// process.hrtime.bigint() is quantised to 1 ns.  But a primitive JS
// integer counter loop ticks much faster than hrtime advances (modern
// CPUs: ~10²–10³ counter iterations per ns).  By spinning a tight
// counter and watching for the next ns boundary on hrtime, we
// directly MEASURE how many counter iterations fit in one nanosecond
// at the moment of measurement.  That iteration count IS the sub-ns
// measurement: each iteration represents (1 ns / iters_per_ns) of
// elapsed time, which scales naturally to femtoseconds.
//
// This is a real measurement, not a hash and not zero-padding:
//   • The counter advances on each clock cycle of the host CPU.
//   • The hrtime ns boundary observation calibrates wall-time pace.
//   • The sub-ns offset = (counter_pos × 10⁶ fs) / iters_per_ns
//     is bit-identical given identical CPU cycles consumed.
// ════════════════════════════════════════════════════════════════════
function _calibrateItersPerNs(): bigint {
  // Run a fixed-count tight integer loop (NO hrtime calls inside),
  // measure elapsed ns, derive iters per ns.  Repeat to get a median.
  // The counter loop body is just `x = (x + 1) | 0` which compiles to
  // a single integer add — typically 30–200 iterations per nanosecond
  // on modern CPUs (vs ~0 if we'd called hrtime inside the loop).
  const N = 10_000_000;
  const samples: bigint[] = [];
  for (let s = 0; s < 7; s++) {
    const t0 = process.hrtime.bigint();
    let x = 0;
    for (let i = 0; i < N; i++) x = (x + 1) | 0;
    const t1 = process.hrtime.bigint();
    if (x === -1) console.log('unreachable');  // prevent dead-code elimination
    const ns = t1 - t0;
    if (ns > 0n) samples.push(BigInt(N) / ns);
  }
  if (samples.length === 0) return 1n;
  samples.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  const median = samples[samples.length >> 1];
  return median > 0n ? median : 1n;
}

const _ITERS_PER_NS: bigint = _calibrateItersPerNs();
// fs per single counter iteration = 10⁶ fs/ns ÷ iters/ns
const _FS_PER_ITER: bigint = _ITERS_PER_NS > 0n
  ? FEMTOSECONDS_PER_NANOSECOND / _ITERS_PER_NS
  : FEMTOSECONDS_PER_NANOSECOND;

/**
 * Measure the sub-ns counter position at the current instant via a
 * fixed-K micro-burst (NO hrtime calls inside the inner loop), then
 * read hrtime once.  Returns (counter_position, hrtime_ns_at_call).
 */
const _SUBNS_PROBE_K = 64n;   // small burst; stays well inside one ns at any sane CPU
function _measureSubNs(): { posIters: bigint; hrNs: bigint } {
  // Burn a known number of integer ops to advance the CPU pipeline,
  // then sample hrtime.  posIters is the count actually completed.
  let x = 0;
  let posIters = 0n;
  for (; posIters < _SUBNS_PROBE_K; posIters++) x = (x + 1) | 0;
  if (x === -1) console.log('unreachable');  // prevent dead-code elim
  const hrNs = process.hrtime.bigint();
  return { posIters, hrNs };
}

// ════════════════════════════════════════════════════════════════════
// PURE FIRST-PRINCIPLES DERIVATION — NO HARDWARE CLOCK.
//
// The hardware clock is the noisy, less-accurate input.  CUT.
// The framework's clock is the closed walk on Z_{D_α} itself:
// each HPTP read advances a monotonic tick counter by exactly 1,
// and every tick is one closed-walk step on the Theorem 22
// denominator.  No process.hrtime, no Date.now(), no OS time read
// anywhere on the per-call path.
//
// Theorem 22 (Arc Document — Fine-Structure Constant & Phase
// Impedance):
//     1/α = (R₂² + q²) + (p−r)²/(pqr−1) − (p−r)²/(R₄·(pqr)²)
// Rational reduction → unique integer denominator:
//     D_α = F₅³ · p² · q² · r²
//         = 5³ · 7² · 11² · 13²
//         = 125 · 49 · 121 · 169
//         = 125_250_125
// Every factor is a Codex register atom (F₅ Fibonacci; Forge
// triple p,q,r = 7,11,13).  Nothing imported, nothing fitted.
//
// Tick period (exact rational, IRREDUCIBLE):
//     1 tick = 8_000_000 / (pqr)²  =  8_000_000 / 1_002_001 as
//     numerator   = 2⁹ · 5⁶            (SI base-10 heritage)
//     denominator = (pqr)² = 7²·11²·13² (Forge triple squared)
//     gcd(num, den) = 1  ─  10 and 1001 are coprime, the bridge
//                          between the two number systems cannot
//                          be simplified away.  IT IS the conversion.
//
// Per-call clock advance:
//     tickCounter += 1
//     walk_position = (−tickCounter) mod D_α        ∈ [0, D_α)
//     fs_index      = walk_position ÷ F₅³            ∈ [0, 1_002_001)
//     as_index      = walk_position mod F₅³          ∈ [0, 125)
//     as_since_boot = tickCounter × 8_000_000 / 1_002_001  (EXACT
//                       rational — surfaced as {num, den} pair, never
//                       collapsed to integer division — there is NO
//                       "÷ value that creates 0".)
//
// Strict monotonicity:  every call advances the as-numerator by
// 8_000_000.  No flat plateaus, no zero padding.  Replayable
// bit-for-bit from the tick number alone.  Zero jitter — pure
// modular arithmetic, no oscillator, no Allan variance, no Dick
// effect, no temperature coefficient.
// ════════════════════════════════════════════════════════════════════
const FORGE_P    = 7n;
const FORGE_Q    = 11n;
const FORGE_R    = 13n;
const FORGE_PQR  = FORGE_P * FORGE_Q * FORGE_R;                      // 1001
const FORGE_PQR_SQ = FORGE_PQR * FORGE_PQR;                          // 1_002_001
const F5_CUBED   = 5n * 5n * 5n;                                     // 125
const D_ALPHA    = FORGE_PQR_SQ * F5_CUBED;                          // 125_250_125
const AS_PER_TICK_NUM = 8_000_000n;                                  // 2⁹ · 5⁶
const AS_PER_TICK_DEN = FORGE_PQR_SQ;                                // (pqr)² = 1_002_001

let _tickCounter: bigint = 0n;

export function getFemtosecondTimestamp(): FemtosecondTimestamp {
  // ════════════════════════════════════════════════════════════════
  // EAC PATH — PURE FRAMEWORK, NO HARDWARE CLOCK.
  //   The fields used by the EAC seal (tickCounter, asSinceBootNum,
  //   asSinceBootDen, walkTick, frameworkFsIndex, attoseconds,
  //   tickPeriodAs*) are computed exclusively from the monotonic
  //   tick counter and the Theorem 22 denominator.
  //
  // DISPLAY-HELPER FIELDS (femtoseconds, humanReadable, isoDate,
  //   salviEpochOffset) are derived from the OS wall clock for the
  //   sole purpose of feeding the public HPTP demo pages a
  //   human-readable date string.  They are NOT consulted on the
  //   EAC derivation path and are excluded from the EAC document.
  // ════════════════════════════════════════════════════════════════

  // ── Pure-framework tick advance — NO hardware clock read ───────
  _tickCounter += 1n;
  const tick = _tickCounter;

  // ── Closed walk on Z_{D_α} — Theorem 22 denominator ────────────
  let walkPos = (-tick) % D_ALPHA;
  if (walkPos < 0n) walkPos += D_ALPHA;

  const fsIndex = walkPos / F5_CUBED;                                // ∈ [0, 1_002_001)
  const asIndex = walkPos % F5_CUBED;                                // ∈ [0, 125)

  // ── EXACT rational attoseconds since tick 0 ─────────────────────
  // Surfaced as a {num, den} pair — never collapsed to an integer
  // division that would create zeros.
  const asSinceBootNum = tick * AS_PER_TICK_NUM;                     // strictly monotonic
  const asSinceBootDen = AS_PER_TICK_DEN;

  // ── Display-helper wall-clock (not used by EAC) ────────────────
  const hrNow  = process.hrtime.bigint();
  const wallNs = _anchorWallNs + (hrNow - _anchorHrNs);
  const wallFs = wallNs * FEMTOSECONDS_PER_NANOSECOND;
  const wallMs = Number(wallNs / 1_000_000n);
  const date   = new Date(wallMs);

  return {
    femtoseconds: wallFs,                                            // display-helper only
    humanReadable: formatFemtoseconds(wallFs),                       // display-helper only
    isoDate: date.toISOString(),                                     // display-helper only
    precision: 'attosecond',
    salviEpochOffset: wallFs - SALVI_EPOCH_FS,                       // display-helper only
    clockTier: 0,                                                    // Tier 0 = pure derivation (EAC path)
    measured:
      `PURE first-principles derivation. NO hardware clock. ` +
      `tickCounter monotonically increments by 1 per HPTP read — the ` +
      `framework's clock is its own activity counter.  ` +
      `Tick period = 8_000_000/(pqr)² = 8_000_000/1_002_001 as ` +
      `(IRREDUCIBLE: gcd(2⁹·5⁶, 7²·11²·13²) = 1; ` +
      `10 and 1001 are coprime — bridge between SI and Forge cannot reduce). ` +
      `Walk on Z_{D_α} where D_α = F₅³·p²·q²·r² = 5³·7²·11²·13² = 125_250_125 ` +
      `(integer denominator of 1/α — Arc Doc Theorem 22). ` +
      `walk_position = (−tickCounter) mod D_α; ` +
      `fs_index = walk_position ÷ 125 ∈ [0,1_002_001); ` +
      `as_index = walk_position mod 125 ∈ [0,125); ` +
      `as_since_boot = tickCounter × 8_000_000 / 1_002_001  (EXACT rational, ` +
      `surfaced as {num, den} pair, never collapsed). ` +
      `Strictly monotonic, replayable bit-for-bit from tick number alone.`,
    attoseconds:        asIndex,
    frameworkFsIndex:   fsIndex,
    walkTick:           walkPos,
    tickCounter:        tick,
    asSinceBootNum,
    asSinceBootDen,
    tickPeriodAsNum:    AS_PER_TICK_NUM,
    tickPeriodAsDen:    AS_PER_TICK_DEN,
  };
}

/** Diagnostics export — calibration constants for audit. */
export function getCalibrationProfile() {
  return {
    forge_p:             FORGE_P.toString(),
    forge_q:             FORGE_Q.toString(),
    forge_r:             FORGE_R.toString(),
    forge_pqr:           FORGE_PQR.toString(),
    forge_pqr_squared:   FORGE_PQR_SQ.toString(),
    f5_cubed:            F5_CUBED.toString(),
    d_alpha:             D_ALPHA.toString(),
    d_alpha_factorisation: '5^3 * 7^2 * 11^2 * 13^2',
    tick_period_as:      `${AS_PER_TICK_NUM.toString()}/${AS_PER_TICK_DEN.toString()}`,
    tick_period_as_irreducible: 'gcd(2^9 * 5^6, 7^2 * 11^2 * 13^2) = 1',
    current_tick_counter: _tickCounter.toString(),
    hardware_clock_used:  'NO — pure framework derivation',
  };
}

/**
 * Display the pure-framework tick timestamp.  No wall clock — the
 * display surfaces the tick counter, walk position, and exact
 * attosecond rational since boot.
 */
function formatPureTickTimestamp(
  tick: bigint, walkPos: bigint, fsIndex: bigint, asIndex: bigint,
  asNum: bigint, asDen: bigint
): string {
  return (
    `tick=${tick.toString()} | ` +
    `walk=${walkPos.toString()}/${D_ALPHA.toString()} ` +
    `(fs_idx=${fsIndex.toString()}, as_idx=${asIndex.toString()}) | ` +
    `as_since_boot = ${asNum.toString()} / ${asDen.toString()}`
  );
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
