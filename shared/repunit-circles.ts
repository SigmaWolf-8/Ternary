// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Repunit Circle Library
//
// Base-3 repunits R(n) = (3^n - 1) / 2 define the natural cycle hierarchy
// of the ternary circle geometry. These are GEOMETRIC constructs measured
// in circle-days (364-day rotations). They are NOT calendar years.
//
// The calendar year = 365 = 364 + DOT (Day Out of Time).
// Repunit circles never include DOT — they are pure geometry.
// To convert to calendar duration, DOT insertions must be computed separately.
//
// INVARIANT: These values derive from the repunit formula and
// the ternary circle identity 111111₃ = 364. No tuning parameters.

// ============================================================
// CORE: Repunit computation
// ============================================================

/**
 * Compute the n-th base-3 repunit: R(n) = (3^n - 1) / 2.
 * R(n) is the n-digit base-3 number consisting entirely of 1s.
 *
 * R(1) = 1, R(2) = 4, R(3) = 13, R(4) = 40, R(5) = 121,
 * R(6) = 364, R(7) = 1093, R(8) = 3280, R(9) = 9841
 *
 * @param n - Repunit length (number of 1-digits in base 3). Must be ≥ 1.
 */
export function repunit(n: number): number {
  if (n < 1 || !Number.isInteger(n)) {
    throw new Error(`Repunit index must be a positive integer, got ${n}`);
  }
  return (Math.pow(3, n) - 1) / 2;
}

/**
 * Repunit factorization identity: R(2n) = R(n) × (3^n + 1).
 * Returns the two factors.
 *
 * @param n - Half-index: returns factors of R(2n).
 */
export function repunitFactorization(n: number): {
  r2n: number;
  rn: number;
  cofactor: number; // 3^n + 1
} {
  const rn = repunit(n);
  const cofactor = Math.pow(3, n) + 1;
  return {
    r2n: rn * cofactor,
    rn,
    cofactor,
  };
}

// ============================================================
// CIRCLE DEFINITIONS
// ============================================================

export interface RepunitCircle {
  /** Repunit index n (number of base-3 digits) */
  index: number;
  /** Repunit value R(n) = (3^n - 1) / 2 */
  value: number;
  /** Circle-days: pure geometric duration (no DOT) */
  circleDays: number;
  /** Human-readable label */
  label: string;
  /** Base-3 repunit string (all 1s) */
  base3: string;
  /** Is R(n) prime? */
  isPrime: boolean;
}

/**
 * The repunit circle hierarchy.
 *
 * IMPORTANT: circleDays is a geometric measure, not a calendar measure.
 * To get calendar days, use `circleDaysToCalendarDays()`.
 */
export const REPUNIT_CIRCLES: RepunitCircle[] = [
  { index: 1, value: 1,    circleDays: 1,    label: "Unit",            base3: "1",       isPrime: false },
  { index: 2, value: 4,    circleDays: 4,    label: "Tetrad",          base3: "11",      isPrime: false },
  { index: 3, value: 13,   circleDays: 13,   label: "Radian",          base3: "111",     isPrime: true  },
  { index: 4, value: 40,   circleDays: 40,   label: "Minor Circle",    base3: "1111",    isPrime: false },
  { index: 5, value: 121,  circleDays: 121,  label: "Quarter Circle",  base3: "11111",   isPrime: false }, // 121 = 11²
  { index: 6, value: 364,  circleDays: 364,  label: "Full Circle",     base3: "111111",  isPrime: false }, // 364 = 2² × 7 × 13
  { index: 7, value: 1093, circleDays: 1093, label: "Triple Circle",   base3: "1111111", isPrime: true  },
  { index: 8, value: 3280, circleDays: 3280, label: "Ennead Circle",   base3: "11111111", isPrime: false },
  { index: 9, value: 9841, circleDays: 9841, label: "Grand Circle",    base3: "111111111", isPrime: false },
];

/**
 * Get a repunit circle by index.
 */
export function getRepunitCircle(n: number): RepunitCircle | undefined {
  return REPUNIT_CIRCLES.find(c => c.index === n);
}

// ============================================================
// CALENDAR CONVERSION (DOT-aware)
// ============================================================

/**
 * Full ternary circle in circle-days (R₆ = 364 = 111111₃).
 * This is the geometric constant. The calendar year is 364 + 1 DOT = 365.
 */
export const FULL_CIRCLE_DAYS = 364;

/**
 * Convert circle-days to calendar days by inserting DOT boundaries.
 *
 * The calendar year is 365 = 364 circle-days + 1 DOT.
 * DOT (Day Out of Time) occurs at the end of each full 364-day circle rotation,
 * specifically on November 11 (the Fibonacci partition point: 8 moons before, 5 after).
 *
 * For a span of `circleDays` geometric days:
 *   - completedCircles = floor(circleDays / 364)
 *   - each completed circle inserts 1 DOT
 *   - calendarDays = circleDays + completedCircles
 *
 * @param circleDays - Duration in geometric circle-days (no DOT).
 * @returns Duration in calendar days (with DOT insertions).
 */
export function circleDaysToCalendarDays(circleDays: number): number {
  if (circleDays < 0) throw new Error("Circle-days cannot be negative");
  const completedCircles = Math.floor(circleDays / FULL_CIRCLE_DAYS);
  return circleDays + completedCircles;
}

/**
 * Convert calendar days to circle-days by removing DOT boundaries.
 *
 * @param calendarDays - Duration in calendar days (with DOTs).
 * @returns Duration in geometric circle-days (pure geometry).
 */
export function calendarDaysToCircleDays(calendarDays: number): number {
  if (calendarDays < 0) throw new Error("Calendar days cannot be negative");
  // Each calendar year is 365 days = 364 circle-days + 1 DOT.
  // completedYears = floor(calendarDays / 365)
  // remaining = calendarDays - completedYears * 365
  // circleDays = completedYears * 364 + min(remaining, 364)
  const completedYears = Math.floor(calendarDays / 365);
  const remaining = calendarDays - completedYears * 365;
  return completedYears * FULL_CIRCLE_DAYS + Math.min(remaining, FULL_CIRCLE_DAYS);
}

// ============================================================
// PLATFORM CONSTANTS (add to shared/constants.ts → PLATFORM)
// ============================================================

/**
 * Repunit circle constants for PLATFORM object.
 */
export const REPUNIT_PLATFORM_CONSTANTS = {
  /** R₃ = 13 = T₇ = 1 radian = 111₃ */
  REPUNIT_R3: 13,
  /** R₄ = 40 = 1111₃ */
  REPUNIT_R4: 40,
  /** R₅ = 121 = 11111₃ = 11² */
  REPUNIT_R5: 121,
  /** R₆ = 364 = 111111₃ = full ternary circle */
  REPUNIT_R6: 364,
  /** R₇ = 1093 = 1111111₃ (prime) — key rotation circle */
  REPUNIT_R7: 1093,
  /** R₈ = 3280 = 11111111₃ — certificate expiry circle */
  REPUNIT_R8: 3280,
  /** R₉ = 9841 = 111111111₃ — archival circle */
  REPUNIT_R9: 9841,

  /** Checksum modulus: R₆ = 364 */
  REPUNIT_CHECKSUM_MODULUS: 364,
  /** Checksum trit count: 6 (digits in R₆ base-3) */
  REPUNIT_CHECKSUM_TRITS: 6,
} as const;

/**
 * Calendar equivalents for repunit circles (precomputed).
 *
 * These are approximate — the exact mapping depends on which DOTs
 * fall within the span relative to the Salvi Epoch start date.
 */
export const REPUNIT_CALENDAR_EQUIVALENTS = {
  R6: { circleDays: 364,  calendarDays: 365,  approxYears: 1 },
  R7: { circleDays: 1093, calendarDays: 1096, approxYears: 3 },   // floor(1093/364) = 3 DOTs
  R8: { circleDays: 3280, calendarDays: 3289, approxYears: 9 },   // 9 DOTs
  R9: { circleDays: 9841, calendarDays: 9868, approxYears: 27 },  // 27 DOTs
} as const;

// ============================================================
// USAGE CONTEXTS
// ============================================================

/**
 * Recommended repunit circle assignments for framework operations.
 *
 * These are geometric intervals — not calendar periods.
 * TSA and HPTP should reference circleDays, with calendar
 * equivalents computed on demand via circleDaysToCalendarDays().
 */
export const REPUNIT_USAGE = {
  /** Daily agent scheduling cycle */
  AGENT_MICRO: 'R3',     // 13 circle-days
  /** Key rotation interval (CNSA 2.0 lifecycle) */
  KEY_ROTATION: 'R7',    // 1093 circle-days
  /** Certificate expiry interval */
  CERT_EXPIRY: 'R8',     // 3280 circle-days
  /** Archival timestamp interval */
  ARCHIVAL: 'R9',        // 9841 circle-days
} as const;
