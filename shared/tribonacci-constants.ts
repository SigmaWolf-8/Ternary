/**
 * Centralized Tribonacci Constants
 *
 * Single source of truth for τ (Tribonacci constant) and derived values.
 * Used by both libternary and server/salvi-core to prevent drift.
 *
 * τ satisfies τ³ = τ² + τ + 1 and is derived from SO(8) quantum graph
 * stability in the Unified 13D Torsion Plenum Theory V9.8Rf.
 *
 * Full precision (50 digits): 1.83928675521416113255185256465328660042417874609759
 * OEIS A058265 — for Rust f128 / arbitrary precision, use the full value above
 * JS/TS IEEE 754 double is limited to ~17 significant digits
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

export const TAU = 1.8392867552141612;

export const verifyTau = (): { valid: boolean; lhs: number; rhs: number; error: number } => {
  const lhs = TAU ** 3;
  const rhs = TAU ** 2 + TAU + 1;
  const error = Math.abs(lhs - rhs);
  return {
    valid: error < 1e-10,
    lhs,
    rhs,
    error,
  };
};

export const TAU_POWERS = {
  TAU_1: TAU,
  TAU_2: TAU ** 2,
  TAU_3: TAU ** 3,
  TAU_5: TAU ** 5,
  TAU_7: TAU ** 7,
  TAU_13: TAU ** 13,
} as const;

export const DERIVED_CONSTANTS = {
  TAU_13: TAU_POWERS.TAU_13,
  S_INST: 2 * TAU_POWERS.TAU_7,
  DELTA_THETA_DEG: 9 / TAU_POWERS.TAU_5,
  M1_TEV: 1.30,
  M_T_GEV: 1.49e-42,
  D4_DIM: 28,
  FUNDAMENTAL_PERIOD_DAYS: 1152,
  P_SID_DAYS: 28 * (1 - TAU ** -6),
  LOG2_3: Math.log2(3),
  DENSITY_ADVANTAGE_PCT: (Math.log2(3) - 1) * 100,
} as const;

export const VM_CONSTANTS = {
  MAX_TRITS: 729,
  WORD_SIZE: 27,
  MAX_REGISTERS: 81,
  REGISTER_COUNT: 27,
  DEFAULT_STACK_SIZE: 4096,
  MAX_CYCLES: 1_000_000,
  HASH_SEED: Math.floor(TAU_POWERS.TAU_2 * 1e9),
  HASH_MIX: Math.floor(TAU_POWERS.TAU_7 * 1e6),
  HASH_ROUNDS: 13,
  MAX_ROUNDS: 13,
  GC_THRESHOLD: 243,
  GC_THRESHOLD_RATIO: TAU ** -2,
  INSTRUCTION_CACHE_SIZE: Math.floor(TAU_POWERS.TAU_5 * 4),
  TRIT_BUFFER_SIZE: Math.floor(TAU_POWERS.TAU_7 * 2),
} as const;

export const TRIBONACCI_SEQUENCE = [
  0, 0, 1, 1, 2, 4, 7, 13, 24, 44,
  81, 149, 274, 504, 927, 1705, 3136, 5768, 10609, 19513,
];

export const TRIBONACCI_RATIO_CONVERGENCE = TRIBONACCI_SEQUENCE
  .slice(3)
  .map((val, i) => ({
    n: i + 3,
    ratio: val / TRIBONACCI_SEQUENCE[i + 2],
    error: Math.abs(val / TRIBONACCI_SEQUENCE[i + 2] - TAU),
  }));

export const TERNARY_CIRCLE = {
  DEGREES: 364,
  PI: 14,
  RADIAN_DEG: 13,
  FULL_CIRCLE_RADIANS: 28,
  CYCLIC_GROUP_ORDER: 28,
  TRIBONACCI_GOLDEN_ANGLE_DEG: 364 / TAU_POWERS.TAU_3,
  TRANSLATED_GOLDEN_ANGLE_DEG: 2 * 7 * 13 * (3 - Math.sqrt(5)),
  TRIBONACCI_WORD_ANGLES: [0, 13, 26] as const,
} as const;

export const verifyTernaryCircle = (): { valid: boolean; checks: Record<string, boolean> } => {
  const c = TERNARY_CIRCLE;
  const checks = {
    degIsRepunit: c.DEGREES === (3 ** 6 - 1) / 2,
    radianIsT7: c.RADIAN_DEG === TRIBONACCI_SEQUENCE[7],
    fullCircle: c.FULL_CIRCLE_RADIANS === c.DEGREES / c.RADIAN_DEG,
    twoPi: c.FULL_CIRCLE_RADIANS === 2 * c.PI,
    radianIsRepunit: c.RADIAN_DEG === 1 + 3 + 9,
  };
  return {
    valid: Object.values(checks).every(Boolean),
    checks,
  };
};
