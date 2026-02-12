/**
 * # Ternary Circle — TypeScript Module
 *
 * Standalone TypeScript implementation of the canonical ternary circle
 * constants, Z₂₈ cyclic group operations, and conversion functions for
 * the Salvi Framework frontend.
 *
 * ## The Axiom
 *
 * A full circle is **364 degrees** = `111111₃` (a base-3 repunit of six 1's).
 * π = 14. One radian = 13° = `111₃` = T₇ (the seventh Tribonacci number).
 *
 * These are not independent choices. They are bound:
 *   - C = πd = 14d, so C/r = 28
 *   - Full circle = 28 radians = 364°
 *   - 1 radian = 364/28 = 13°
 *
 * ## Migration Guide: 120° → 364°
 *
 * The old "trinary" 120° symmetry is replaced by the canonical 364° circle
 * with 28-fold rotational symmetry on Z₂₈.
 *
 * | Old (120° system)          | New (364° system)                      |
 * |----------------------------|----------------------------------------|
 * | Full circle = 360°         | Full circle = 364° = 111111₃           |
 * | π ≈ 3.14159…               | π = 14 (exact)                         |
 * | 1 radian ≈ 57.2958°        | 1 radian = 13° = 111₃ (exact)          |
 * | 3-fold symmetry (120°)     | 28-fold symmetry (Z₂₈)                |
 * | Walk turns: 0°, 120°, 240° | Walk turns: 0°, 13°, 26° (in ternary)  |
 * | φ-scaling (golden ratio)   | τ-scaling (Tribonacci constant)        |
 *
 * To convert existing code:
 *   1. Replace `360` with `FULL_CIRCLE_DEG` (364)
 *   2. Replace `Math.PI` with `PI_TERNARY` (14) for ternary geometry
 *   3. Replace `2 * Math.PI` with `TWO_PI_TERNARY` (28) for full rotations
 *   4. Use `ternaryDegToStdDeg()` when passing angles to `Math.cos()`/`Math.sin()`
 *   5. Use `Z28` for discrete angular positions instead of raw degree arithmetic
 */

// ══════════════════════════════════════════════════════════════
// TERNARY CIRCLE CONSTANTS
// ══════════════════════════════════════════════════════════════

/** Full circle in the ternary angular system: **364 degrees**.
 *  In base 3: `111111₃` = (3⁶ − 1) / 2 = 364. */
export const FULL_CIRCLE_DEG = 364;

/** Full circle as a base-3 repunit string. */
export const FULL_CIRCLE_BASE3 = "111111";

/** π in the ternary circle system: **exactly 14**.
 *  Ratio of circumference to diameter. 14 = 112₃. */
export const PI_TERNARY = 14;

/** 2π in the ternary system: **28**.
 *  A full circle is 28 ternary radians. 28 = 1001₃. */
export const TWO_PI_TERNARY = 28;

/** One ternary radian in degrees: **exactly 13°**.
 *  13 = `111₃`, a three-digit base-3 repunit.
 *  13 is also T₇, the seventh Tribonacci number. */
export const RADIAN_DEG = 13;

/** One ternary radian as a base-3 repunit string. */
export const RADIAN_BASE3 = "111";

/** Order of the cyclic group Z₂₈. */
export const CYCLIC_ORDER = 28;

/** Number of ternary radians in a full circle. */
export const RADIANS_PER_CIRCLE = 28;

// ══════════════════════════════════════════════════════════════
// TRIBONACCI CONSTANTS IN THE TERNARY CIRCLE
// ══════════════════════════════════════════════════════════════

/** The Tribonacci constant τ ≈ 1.839286755214161.
 *  Real root of x³ = x² + x + 1. */
export const TAU_TRIBONACCI = 1.839286755214161;

/** τ² ≈ 3.38297576790891. */
export const TAU_SQUARED = 3.38297576790891;

/** τ³ = τ² + τ + 1 ≈ 6.22226252312307. */
export const TAU_CUBED = 6.22226252312307;

/** Tribonacci golden angle: 364° / τ³ ≈ 58.50°. */
export const TRIBONACCI_GOLDEN_ANGLE_DEG = 58.50438656;

/** Classical golden angle in the ternary circle: 364° / φ². */
export const GOLDEN_ANGLE_TERNARY_DEG = 138.98056;

// ══════════════════════════════════════════════════════════════
// WALK INSTRUCTION SET ON Z₂₈
// ══════════════════════════════════════════════════════════════

/** Walk instruction for trit digit 0: 0 radians (no turn). */
export const WALK_TURN_0 = 0;

/** Walk instruction for trit digit 1: 1 ternary radian = 13°. */
export const WALK_TURN_1 = 13;

/** Walk instruction for trit digit 2: 2 ternary radians = 26°. */
export const WALK_TURN_2 = 26;

// ══════════════════════════════════════════════════════════════
// CONVERSION FUNCTIONS
// ══════════════════════════════════════════════════════════════

/** Convert ternary degrees (364° full circle) to conventional degrees (360°). */
export function ternaryDegToStdDeg(ternaryDeg: number): number {
  return ternaryDeg * (360 / FULL_CIRCLE_DEG);
}

/** Convert conventional degrees (360° full circle) to ternary degrees (364°). */
export function stdDegToTernaryDeg(stdDeg: number): number {
  return stdDeg * (FULL_CIRCLE_DEG / 360);
}

/** Convert ternary radians (28 per circle) to standard radians (2π per circle). */
export function ternaryRadToStdRad(ternaryRad: number): number {
  return ternaryRad * ((2 * Math.PI) / TWO_PI_TERNARY);
}

/** Convert standard radians to ternary radians. */
export function stdRadToTernaryRad(stdRad: number): number {
  return stdRad * (TWO_PI_TERNARY / (2 * Math.PI));
}

/** Convert ternary degrees to ternary radians. */
export function ternaryDegToTernaryRad(deg: number): number {
  return deg / RADIAN_DEG;
}

/** Convert ternary radians to ternary degrees. */
export function ternaryRadToTernaryDeg(rad: number): number {
  return rad * RADIAN_DEG;
}

/** Convert a trit digit (0, 1, or 2) to its walk angle in ternary degrees. */
export function tritToWalkAngleDeg(trit: number): number {
  if (trit < 0 || trit > 2) throw new Error(`Trit must be 0, 1, or 2; got ${trit}`);
  return trit * RADIAN_DEG;
}

/** Convert a trit digit to its walk angle in standard radians. */
export function tritToStdRad(trit: number): number {
  return ternaryRadToStdRad(trit);
}

// ══════════════════════════════════════════════════════════════
// Z₂₈ CYCLIC GROUP
// ══════════════════════════════════════════════════════════════

/**
 * A position in the cyclic group Z₂₈.
 *
 * Represents one of the 28 discrete angular positions in the ternary
 * circle, separated by 13° each.
 */
export class Z28 {
  public readonly value: number;

  constructor(val: number) {
    this.value = ((val % 28) + 28) % 28;
  }

  /** The identity element (0 position). */
  static zero(): Z28 {
    return new Z28(0);
  }

  /** Create from a raw value, reducing modulo 28. */
  static from(val: number): Z28 {
    return new Z28(val);
  }

  /** Add two elements in Z₂₈ (group operation). */
  add(other: Z28): Z28 {
    return new Z28(this.value + other.value);
  }

  /** Subtract (inverse addition) in Z₂₈. */
  sub(other: Z28): Z28 {
    return new Z28(this.value - other.value + 28);
  }

  /** Negate (additive inverse) in Z₂₈. */
  neg(): Z28 {
    return this.value === 0 ? new Z28(0) : new Z28(28 - this.value);
  }

  /** The ternary degree value of this position. */
  toTernaryDeg(): number {
    return this.value * RADIAN_DEG;
  }

  /** The standard radian value (for trigonometric functions). */
  toStdRad(): number {
    return ternaryRadToStdRad(this.value);
  }

  /** Advance by a trit instruction (0, 1, or 2 ternary radians). */
  step(trit: number): Z28 {
    if (trit < 0 || trit > 2) throw new Error(`Trit must be 0, 1, or 2; got ${trit}`);
    return this.add(new Z28(trit));
  }

  /** Check equality with another Z₂₈ element. */
  equals(other: Z28): boolean {
    return this.value === other.value;
  }

  /** String representation. */
  toString(): string {
    return `Z₂₈(${this.value})`;
  }
}

// ══════════════════════════════════════════════════════════════
// SPIRAL WALK ENGINE
// ══════════════════════════════════════════════════════════════

/** A point in the Tribonacci radian spiral. */
export interface SpiralPoint {
  x: number;
  y: number;
  position: Z28;
  trit: number;
  step: number;
}

/**
 * Walk the Tribonacci radian spiral.
 *
 * Given a sequence of trit digits, compute the spiral path:
 *   z_n = Σ(k=1..n) e^(i · w_k · 13°) / τ^k
 *
 * where w_k ∈ {0, 1, 2} is the k-th trit, 13° is the ternary radian,
 * and τ is the Tribonacci constant (radial scaling).
 */
export function walkTribonacciRadianSpiral(trits: number[]): SpiralPoint[] {
  const points: SpiralPoint[] = [];
  let x = 0;
  let y = 0;
  let direction = Z28.zero();
  let tauPower = 1;

  points.push({ x: 0, y: 0, position: direction, trit: 0, step: 0 });

  for (let k = 0; k < trits.length; k++) {
    const trit = trits[k];
    if (trit < 0 || trit > 2) throw new Error(`Trit must be 0, 1, or 2; got ${trit}`);

    direction = direction.step(trit);
    tauPower *= TAU_TRIBONACCI;

    const angleStdRad = direction.toStdRad();
    const stepLen = 1 / tauPower;
    x += Math.cos(angleStdRad) * stepLen;
    y += Math.sin(angleStdRad) * stepLen;

    points.push({ x, y, position: direction, trit, step: k + 1 });
  }

  return points;
}

// ══════════════════════════════════════════════════════════════
// REPUNIT VERIFICATION
// ══════════════════════════════════════════════════════════════

/** Verify that a number is a base-3 repunit (all 1's in base 3). */
export function isBase3Repunit(n: number): boolean {
  if (n <= 0 || !Number.isInteger(n)) return false;
  let m = 2 * n + 1;
  while (m > 1) {
    if (m % 3 !== 0) return false;
    m = Math.floor(m / 3);
  }
  return m === 1;
}

/** Return the repunit order (number of 1's) if the value is a base-3 repunit. */
export function base3RepunitOrder(n: number): number | null {
  if (!isBase3Repunit(n)) return null;
  let m = 2 * n + 1;
  let order = 0;
  while (m > 1) {
    m = Math.floor(m / 3);
    order++;
  }
  return order;
}

// ══════════════════════════════════════════════════════════════
// TRIBONACCI WORD GENERATOR
// ══════════════════════════════════════════════════════════════

/**
 * Generate the Tribonacci word — the fixed point of the morphism:
 *   0 → 01, 1 → 02, 2 → 0
 *
 * This is a 3-automatic sequence over {0, 1, 2}.
 */
export function tribonacciWord(length: number): number[] {
  let word = [0];

  while (word.length < length) {
    const next: number[] = [];
    for (const ch of word) {
      if (ch === 0) { next.push(0, 1); }
      else if (ch === 1) { next.push(0, 2); }
      else { next.push(0); }
    }
    word = next;
  }

  return word.slice(0, length);
}
