/**
 * libternary - Tribonacci Constants & Sequences
 *
 * Implements the Tribonacci mathematics from the Unified 13D Torsion Plenum Theory V9.8Rf.
 * τ (Tribonacci constant) is DERIVED from pre-geometric SO(8) quantum graph stability,
 * satisfying τ³ = τ² + τ + 1.
 *
 * These constants appear throughout PlenumNET's applied physics implementations
 * and connect the theoretical framework to production engineering.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

/**
 * The Tribonacci constant τ — real root of τ³ = τ² + τ + 1
 * Derived from SO(8) quantum graph stability (V9.8Rf)
 */
export const TAU = 1.8392867552141612;

/**
 * Pre-computed powers of τ used throughout the theory
 */
export const TAU_POWERS = {
  TAU_2: TAU ** 2,
  TAU_3: TAU ** 3,
  TAU_5: TAU ** 5,
  TAU_7: TAU ** 7,
  TAU_13: TAU ** 13,
} as const;

/**
 * Key physical constants derived from τ
 */
export const DERIVED_CONSTANTS = {
  /** τ¹³ = 2757.038 — fundamental period constant */
  TAU_13: TAU_POWERS.TAU_13,

  /** S_inst = 2τ⁷ = 142.42 — instanton action for dark energy suppression */
  S_INST: 2 * TAU_POWERS.TAU_7,

  /** Δθ = 9/τ⁵ = 0.4275° — CMB birefringence prediction (confirmed by observation) */
  DELTA_THETA_DEG: 9 / TAU_POWERS.TAU_5,

  /** M₁ = 1.30 TeV — first LHC resonance mass */
  M1_TEV: 1.30,

  /** m_T ≈ 1.49×10⁻⁴² GeV — torsion mass scale */
  M_T_GEV: 1.49e-42,

  /** D₄ gauge group dimension = 28 */
  D4_DIM: 28,

  /** Fundamental period = 1152 days (harmonic alignment) */
  FUNDAMENTAL_PERIOD_DAYS: 1152,

  /** P_sid = 28(1 - τ⁻⁶) = 27.28 days — lunar sidereal period */
  P_SID_DAYS: 28 * (1 - TAU ** -6),

  /** log₂(3) ≈ 1.585 — information density advantage per trit */
  LOG2_3: Math.log2(3),

  /** 59% density advantage: (log₂(3) - 1) × 100 */
  DENSITY_ADVANTAGE_PCT: (Math.log2(3) - 1) * 100,
} as const;

/**
 * Generate Tribonacci sequence T(n) where T(n) = T(n-1) + T(n-2) + T(n-3)
 * Starting values: T(0)=0, T(1)=0, T(2)=1
 *
 * Key values used in the theory:
 *   T(4)=2, T(5)=4, T(6)=7, T(7)=13, T(8)=24, T(9)=44, T(10)=81
 *   T(7)=13 links the 7-cycle to 13 dimensions
 */
export function tribonacci(n: number): number {
  if (n < 0) throw new Error('Tribonacci index must be non-negative');
  if (n === 0 || n === 1) return 0;
  if (n === 2) return 1;

  let a = 0, b = 0, c = 1;
  for (let i = 3; i <= n; i++) {
    const next = a + b + c;
    a = b;
    b = c;
    c = next;
  }
  return c;
}

/**
 * Generate Tribonacci sequence up to index n (inclusive)
 * Returns array [T(0), T(1), ..., T(n)]
 */
export function tribonacciSequence(n: number): number[] {
  if (n < 0) return [];
  const seq: number[] = [0];
  if (n === 0) return seq;
  seq.push(0);
  if (n === 1) return seq;
  seq.push(1);

  for (let i = 3; i <= n; i++) {
    seq.push(seq[i - 1] + seq[i - 2] + seq[i - 3]);
  }
  return seq;
}

/**
 * Calculate LHC resonance mass M_n using the theory's formula:
 * M_n = M₁ × T(n+6) / T(7)
 *
 * Where M₁ = 1.30 TeV and T(7) = 13
 */
export function resonanceMass(n: number): { n: number; mass_TeV: number; T_index: number; T_value: number } {
  const T7 = tribonacci(7);
  const T_index = n + 6;
  const T_value = tribonacci(T_index);
  const mass = DERIVED_CONSTANTS.M1_TEV * T_value / T7;

  return {
    n,
    mass_TeV: Math.round(mass * 1000) / 1000,
    T_index,
    T_value
  };
}

/**
 * Compute QGN (Quantum Graphitic Noise) decoherence rate
 * Γ ≈ τ⁻⁵ × E² / M_P
 *
 * @param E_GeV Energy in GeV
 * @param M_P_GeV Planck mass in GeV (default: 1.22e19)
 * @returns Decoherence rate in GeV
 */
export function qgnDecoherence(E_GeV: number, M_P_GeV: number = 1.22e19): {
  E_GeV: number;
  gamma_GeV: number;
  tau_minus_5: number;
} {
  const tau_minus_5 = TAU ** -5;
  const gamma = tau_minus_5 * (E_GeV ** 2) / M_P_GeV;

  return {
    E_GeV,
    gamma_GeV: gamma,
    tau_minus_5
  };
}

/**
 * Verify that τ satisfies the defining equation τ³ = τ² + τ + 1
 * Useful for validation and testing
 */
export function verifyTau(): { valid: boolean; lhs: number; rhs: number; error: number } {
  const lhs = TAU ** 3;
  const rhs = TAU ** 2 + TAU + 1;
  const error = Math.abs(lhs - rhs);

  return {
    valid: error < 1e-10,
    lhs,
    rhs,
    error
  };
}
