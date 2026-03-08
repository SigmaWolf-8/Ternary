// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// CRT Fast Path — TypeScript Implementation
// Mirrors: src/kernel/src/hptp/crt_fast_path.rs
//
// Dual-projection timing accelerator using 364 = 13 × 28 CRT decomposition.
// The fast 28-path resolves in O(1) binary-friendly ops; the precise 13-path
// requires full modular division. Progressive refinement: act on the fast
// answer first, confirm with the precise answer.

// ============================================================
// CRT CONSTANTS (precomputed, immutable)
// ============================================================

/** Full ternary circle: 364 = 111111₃ = 13 × 28 */
export const FULL_CIRCLE = 364;

/** Moon-axis modulus */
export const MOD_MOON = 13;

/** Day-axis modulus */
export const MOD_DAY = 28;

/** 13⁻¹ mod 28 = 13 (self-inverse: 13×13 = 169 = 6×28 + 1) */
export const INV_13_MOD_28 = 13;

/** 28⁻¹ mod 13 = 7 (since 28 mod 13 = 2, 2×7 = 14 = 13 + 1) */
export const INV_28_MOD_13 = 7;

/** CRT reconstruction coefficient for fine (moon) component: 28 × 7 = 196 */
export const COEFF_FINE = MOD_DAY * INV_28_MOD_13; // 196

/** CRT reconstruction coefficient for fast (day) component: 13 × 13 = 169 */
export const COEFF_FAST = MOD_MOON * INV_13_MOD_28; // 169

/** Femtoseconds per circle-day (86,400 seconds) */
export const FEMTOSECONDS_PER_CIRCLE_DAY = 86_400_000_000_000_000_000n;

// ============================================================
// FAST PATH: mod-28 (binary-friendly)
// ============================================================

/**
 * Fast day-component extraction via mod-28 decomposition.
 *
 * 28 = 4 × 7:
 *   mod 4 → bitwise AND (instant)
 *   mod 7 → direct modulo (JS engines optimize small-constant modulo)
 *   CRT reconstruct → (21 × r4 + 8 × r7) mod 28
 *
 * In V8, this compiles to ~3 machine instructions for small integers.
 */
export function fastDayComponent(position: number): number {
  const r4 = position & 0x03;     // mod 4: bitmask
  const r7 = position % 7;        // mod 7: JS engine optimizes this
  return (21 * r4 + 8 * r7) % 28; // CRT reconstruct mod 28
}

// ============================================================
// PRECISE PATH: mod-13
// ============================================================

/**
 * Exact moon-sector determination (the slow path).
 * 13 is prime with no power-of-2 factor — no binary shortcut exists.
 */
export function fineMoonComponent(position: number): number {
  return position % MOD_MOON;
}

// ============================================================
// CRT RECONSTRUCTION
// ============================================================

/**
 * Reconstruct full circle position from CRT components.
 *
 * @param fine - Moon component (0–12), from fineMoonComponent()
 * @param fast - Day component (0–27), from fastDayComponent()
 * @returns Circle position (0–363)
 */
export function reconstruct(fine: number, fast: number): number {
  return (COEFF_FINE * fine + COEFF_FAST * fast) % FULL_CIRCLE;
}

// ============================================================
// PROGRESSIVE REFINEMENT
// ============================================================

export interface CoarseDecision {
  /** Day-within-moon (0–27), resolved in ~3 ops */
  dayComponent: number;
  /** Approximate trit dimensions resolved (~3, since 28 ≈ 3³) */
  resolvedTrits: number;
}

export interface FineDecision {
  /** Moon-sector (0–12), resolved via full mod-13 */
  moonComponent: number;
  /** Full circle position, CRT-reconstructed */
  circlePosition: number;
}

/**
 * Progressive route: fast path fires first, precise path confirms.
 *
 * In HFT usage:
 *   1. Read `coarse` immediately → begin pre-positioning
 *   2. Read `fine` when available → verify or micro-correct
 *
 * @param circlePosition - Position on the 364-point circle (0–363)
 */
export function progressiveRoute(circlePosition: number): {
  coarse: CoarseDecision;
  fine: FineDecision;
} {
  const pos = circlePosition % FULL_CIRCLE;

  // FAST PATH
  const day = fastDayComponent(pos);
  const coarse: CoarseDecision = {
    dayComponent: day,
    resolvedTrits: 3,
  };

  // PRECISE PATH
  const moon = fineMoonComponent(pos);
  const reconstructed = reconstruct(moon, day);

  const fine: FineDecision = {
    moonComponent: moon,
    circlePosition: reconstructed,
  };

  return { coarse, fine };
}

// ============================================================
// TIMESTAMP INTEGRATION
// ============================================================

/**
 * Convert HPTP femtosecond timestamp to circle position.
 *
 * @param femtosecondsSinceEpoch - 128-bit timestamp as bigint
 * @returns Circle position (0–363)
 */
export function timestampToCirclePosition(femtosecondsSinceEpoch: bigint): number {
  const dayIndex = femtosecondsSinceEpoch / FEMTOSECONDS_PER_CIRCLE_DAY;
  return Number(dayIndex % BigInt(FULL_CIRCLE));
}

/**
 * Full progressive route from HPTP timestamp.
 */
export function routeFromTimestamp(femtosecondsSinceEpoch: bigint): {
  coarse: CoarseDecision;
  fine: FineDecision;
} {
  const pos = timestampToCirclePosition(femtosecondsSinceEpoch);
  return progressiveRoute(pos);
}

// ============================================================
// CLOCK SOURCE INTEGRATION
// ============================================================

/**
 * The mod-28 fast path produces mod-7 as a sub-step.
 * With 7 HPTP clock sources (prime), mod-7 directly indexes the
 * active source — the timing and routing layers share arithmetic.
 */
export function fastClockSourceIndex(circlePosition: number): number {
  return (circlePosition % FULL_CIRCLE) % 7;
}
