/**
 * Centralized Tribonacci Constants
 *
 * Single source of truth for τ (Tribonacci constant) and derived values.
 * Used by both libternary and server/salvi-core to prevent drift.
 *
 * τ satisfies τ³ = τ² + τ + 1 and is derived from SO(8) quantum graph
 * stability in the Unified 13D Torsion Plenum Theory V9.8Rf.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

export const TAU = 1.8392867552141612;

export const TAU_POWERS = {
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
  REGISTER_COUNT: 27,
  DEFAULT_STACK_SIZE: 4096,
  MAX_CYCLES: 1_000_000,
  HASH_SEED: Math.floor(TAU_POWERS.TAU_2 * 1e9),
  HASH_MIX: Math.floor(TAU_POWERS.TAU_7 * 1e6),
  HASH_ROUNDS: 13,
  GC_THRESHOLD_RATIO: TAU ** -2,
  INSTRUCTION_CACHE_SIZE: Math.floor(TAU_POWERS.TAU_5 * 4),
  TRIT_BUFFER_SIZE: Math.floor(TAU_POWERS.TAU_7 * 2),
} as const;
