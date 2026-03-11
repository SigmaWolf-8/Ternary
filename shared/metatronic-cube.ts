/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * See LICENSE in the repository root for full terms.
 */

/**
 * # Saturnian Tesseract Metatron Ternary Cube
 *
 * TypeScript parallel of `metatronic_cube.rs`. The 13-dimensional ternary
 * cube viewed through the Metatronic geometry and Saturnian Black Cube
 * tradition. Connects to the existing modules:
 *
 * - `plenum-square.ts` → Plenum Square, Tribonacci alignment
 * - `ternary-circle.ts` → Z₂₈, 364°, π = 14, radian = 13° = T₇
 * - `topology/index.ts` → Toroidal addressing, GF(3) operations
 *
 * GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.
 *
 * @license All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026
 */

import {
  PLENUM_SQUARE_MATRIX,
  MAGIC_CONSTANT,
  TERNARY_BALANCE_CENTER,
  PI_ESOTERIC,
  RADIUS_COSMIC,
  PLENUM_NATURAL_YEAR_DAYS,
} from './plenum-square';

import {
  FULL_CIRCLE_DEG,
  PI_TERNARY,
  TWO_PI_TERNARY,
  RADIAN_DEG,
  CYCLIC_ORDER,
  Z28,
} from './ternary-circle';

import { type Trit, gf3Add, gf3Neg } from './topology';

// ══════════════════════════════════════════════════════════════
// CONSTANTS
// ══════════════════════════════════════════════════════════════

/** Total dimensions of the Metatronic Cube. */
export const METATRONIC_DIM = 13;

/** Total vertices: 3^13 = 1,594,323. */
export const METATRONIC_VERTICES = 1_594_323;

/** Vertices per shell: 3^12 = 531,441. */
export const SHELL_VERTICES = 531_441;

/** Depth axis index (0-indexed INTERNAL representation). */
export const DEPTH_AXIS = 12;

/**
 * Depth axis in Rep C (1-based bijective): **13** = T₇ = one ternary radian.
 * In the ternary computational domain, axis identifiers are 1-based
 * so that zero becomes a sentinel — just like trit Rep C {1,2,3}.
 */
export const DEPTH_AXIS_RC = 13;

// ── Bijective Axis Numbering (Rep C for axes) ──────────────────────

/**
 * Convert an internal 0-indexed axis index to Rep C (1-based bijective).
 *
 * Internal 0 → RC 1, ..., Internal 12 → RC 13.
 * Returns null if axis ≥ 13.
 */
export function axisToRepC(internal: number): number | null {
  if (internal >= 0 && internal < METATRONIC_DIM) {
    return internal + 1;
  }
  return null;
}

/**
 * Convert a Rep C axis identifier (1..13) to internal 0-indexed.
 *
 * **Zero rejection**: RC axis 0 is structurally impossible — the sentinel
 * property extends from trit values to axis identifiers. A zero axis in
 * a ternary wire packet proves corruption or forgery.
 */
export function axisFromRepC(rc: number): number | null {
  if (rc >= 1 && rc <= METATRONIC_DIM) {
    return rc - 1;
  }
  return null; // 0 = sentinel-invalid; >13 = out of range
}

/** Validate a Rep C axis identifier. True if rc ∈ {1..13}. */
export function axisRcValid(rc: number): boolean {
  return rc >= 1 && rc <= METATRONIC_DIM;
}

/** The Saturnian weight for each of the 13 axes. */
export const SATURNIAN_WEIGHTS: readonly number[] = [
  TERNARY_BALANCE_CENTER, // axis 0: Central = 111
  PI_ESOTERIC,            // axis 1: Inner = 14
  PI_ESOTERIC,            // axis 2: Inner = 14
  PI_ESOTERIC,            // axis 3: Inner = 14
  PI_ESOTERIC,            // axis 4: Inner = 14
  PI_ESOTERIC,            // axis 5: Inner = 14
  PI_ESOTERIC,            // axis 6: Inner = 14
  208,                    // axis 7: Outer (COSMIC_WEIGHT from blueprint)
  208,                    // axis 8: Outer
  208,                    // axis 9: Outer
  208,                    // axis 10: Outer
  208,                    // axis 11: Outer
  MAGIC_CONSTANT,         // axis 12: Depth = 333
] as const;

/** Sum of all axis weights: 111 + 6×14 + 5×208 + 333 = 1568. */
export const TOTAL_SATURNIAN_WEIGHT = SATURNIAN_WEIGHTS.reduce((a, b) => a + b, 0);

/**
 * Saturnian trit constants — the Magic Square flattened and reduced to GF(3).
 *
 * [111, 14, 208, 208, 111, 14, 14, 208, 111] mod 3 →
 * [0, -1, 1, 1, 0, -1, -1, 1, 0]
 *
 * Period 9 = 3². Tiles 81× into the 729-trit sponge state.
 */
export const SATURNIAN_TRIT_CONSTANTS: readonly Trit[] = [
  0, -1, 1, 1, 0, -1, -1, 1, 0,
] as const;

/**
 * Number of ternary tesseract families: C(13,4) = 715.
 * Each family has 3^9 = 19,683 distinct embeddings.
 * Total: 715 × 19,683 = 14,073,405.
 */
export const TESSERACT_FAMILIES = 715;
export const TESSERACT_TOTAL = 14_073_405;

// ══════════════════════════════════════════════════════════════
// METATRONIC CIRCLE / AXIS IDENTITY
// ══════════════════════════════════════════════════════════════

/** The domain groups of the Metatronic axes. */
export type MetatronicDomain =
  | 'foundation'     // axis 0
  | 'manifestation'  // axes 1–6
  | 'transcendence'  // axes 7–11
  | 'shell_boundary'; // axis 12

/** A Metatronic circle assignment for one axis. */
export interface MetatronicCircle {
  /** Axis index in **internal** (0-indexed) representation. */
  readonly axis: number;
  /** Axis index in **Rep C** (1-based bijective) for wire encoding. */
  readonly axisRc: number;
  /** The circle name. */
  readonly name: string;
  /** Domain group. */
  readonly domain: MetatronicDomain;
  /** Saturnian weight from the Magic Square. */
  readonly weight: number;
}

/** All 13 Metatronic circles in axis order. */
export const METATRONIC_CIRCLES: readonly MetatronicCircle[] = [
  { axis: 0,  axisRc: 1,  name: 'Central',  domain: 'foundation',     weight: TERNARY_BALANCE_CENTER },
  { axis: 1,  axisRc: 2,  name: 'Inner 1',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 2,  axisRc: 3,  name: 'Inner 2',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 3,  axisRc: 4,  name: 'Inner 3',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 4,  axisRc: 5,  name: 'Inner 4',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 5,  axisRc: 6,  name: 'Inner 5',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 6,  axisRc: 7,  name: 'Inner 6',  domain: 'manifestation',  weight: PI_ESOTERIC },
  { axis: 7,  axisRc: 8,  name: 'Outer 1',  domain: 'transcendence',  weight: 208 },
  { axis: 8,  axisRc: 9,  name: 'Outer 2',  domain: 'transcendence',  weight: 208 },
  { axis: 9,  axisRc: 10, name: 'Outer 3',  domain: 'transcendence',  weight: 208 },
  { axis: 10, axisRc: 11, name: 'Outer 4',  domain: 'transcendence',  weight: 208 },
  { axis: 11, axisRc: 12, name: 'Outer 5',  domain: 'transcendence',  weight: 208 },
  { axis: 12, axisRc: 13, name: 'Depth',    domain: 'shell_boundary',  weight: MAGIC_CONSTANT },
] as const;

/**
 * Get the MetatronicCircle for a given **internal** (0-indexed) axis.
 * Use `getCircleRc()` for Rep C (1-based) input.
 */
export function getCircle(axis: number): MetatronicCircle | undefined {
  return METATRONIC_CIRCLES[axis];
}

/**
 * Get the MetatronicCircle for a given **Rep C** (1-based bijective) axis.
 * Returns undefined if rc is 0 (sentinel violation) or > 13.
 */
export function getCircleRc(rc: number): MetatronicCircle | undefined {
  const internal = axisFromRepC(rc);
  return internal !== null ? METATRONIC_CIRCLES[internal] : undefined;
}

// ══════════════════════════════════════════════════════════════
// THREE SATURNIAN SHELLS
// ══════════════════════════════════════════════════════════════

/** The three shells of the Saturnian Ternary Cube. */
export type SaturnianShell = 'inner' | 'void' | 'outer';

/** Shell from the depth-axis trit value. */
export function shellFromDepthTrit(t: Trit): SaturnianShell {
  switch (t) {
    case -1: return 'inner';
    case 0:  return 'void';
    case 1:  return 'outer';
    default: return 'void';
  }
}

/** Depth trit from shell name. */
export function depthTritFromShell(shell: SaturnianShell): Trit {
  switch (shell) {
    case 'inner': return -1;
    case 'void':  return 0;
    case 'outer': return 1;
  }
}

/** The opposing shell. */
export function mirrorShell(shell: SaturnianShell): SaturnianShell {
  switch (shell) {
    case 'inner': return 'outer';
    case 'outer': return 'inner';
    case 'void':  return 'void';
  }
}

// ══════════════════════════════════════════════════════════════
// METATRONIC VERTEX
// ══════════════════════════════════════════════════════════════

/** Balanced ternary coordinate type: Rep A {-1, 0, +1}. */
export type RepA = readonly Trit[];

/** Wire encoding type: Rep B {0, 1, 2}. */
export type RepB = readonly number[];

/** Bijective ternary type: Rep C {1, 2, 3}. */
export type RepC = readonly number[];

/** A vertex in the 13D Metatronic Cube. */
export class MetatronicVertex {
  /** Rep A coordinates {-1, 0, +1}. */
  public readonly coords: readonly Trit[];

  private constructor(coords: Trit[]) {
    this.coords = Object.freeze(coords);
  }

  /** Create from Rep A {-1, 0, +1}. */
  static fromRepA(coords: Trit[]): MetatronicVertex | null {
    if (coords.length !== METATRONIC_DIM) return null;
    if (!coords.every(c => c >= -1 && c <= 1)) return null;
    return new MetatronicVertex([...coords]);
  }

  /** Create from Rep B {0, 1, 2}. */
  static fromRepB(coords: number[]): MetatronicVertex | null {
    if (coords.length !== METATRONIC_DIM) return null;
    const repA = coords.map(c => {
      if (c < 0 || c > 2) return null;
      return (c - 1) as Trit;
    });
    if (repA.some(c => c === null)) return null;
    return new MetatronicVertex(repA as Trit[]);
  }

  /** Create from Rep C {1, 2, 3}. Returns null if any digit is 0 (sentinel). */
  static fromRepC(coords: number[]): MetatronicVertex | null {
    if (coords.length !== METATRONIC_DIM) return null;
    const repA = coords.map(c => {
      if (c < 1 || c > 3) return null;
      return (c - 2) as Trit;
    });
    if (repA.some(c => c === null)) return null;
    return new MetatronicVertex(repA as Trit[]);
  }

  /** Create from linear index (0..1,594,322). */
  static fromLinearIndex(idx: number): MetatronicVertex {
    const coords: Trit[] = [];
    let n = idx;
    for (let i = 0; i < METATRONIC_DIM; i++) {
      coords.push(((n % 3) - 1) as Trit);
      n = Math.floor(n / 3);
    }
    return new MetatronicVertex(coords);
  }

  /** The origin vertex (all zeros, Void shell). */
  static origin(): MetatronicVertex {
    return new MetatronicVertex(new Array(METATRONIC_DIM).fill(0) as Trit[]);
  }

  /** Convert to Rep B {0, 1, 2}. */
  toRepB(): number[] {
    return this.coords.map(c => c + 1);
  }

  /** Convert to Rep C {1, 2, 3}. */
  toRepC(): number[] {
    return this.coords.map(c => c + 2);
  }

  /** Convert to linear index. */
  toLinearIndex(): number {
    let idx = 0;
    let power = 1;
    for (let i = 0; i < METATRONIC_DIM; i++) {
      idx += (this.coords[i] + 1) * power;
      power *= 3;
    }
    return idx;
  }

  /** Which Saturnian shell this vertex inhabits. */
  shell(): SaturnianShell {
    return shellFromDepthTrit(this.coords[DEPTH_AXIS]);
  }

  /** The 12D intra-shell coordinates (axes 0..11). */
  shellCoords(): Trit[] {
    return [...this.coords.slice(0, 12)] as Trit[];
  }

  /** The Metatronic circle for a given **internal** (0-indexed) axis. */
  circleAt(axis: number): MetatronicCircle | undefined {
    return METATRONIC_CIRCLES[axis];
  }

  /** The Metatronic circle for a given **Rep C** (1-based) axis. */
  circleAtRc(rc: number): MetatronicCircle | undefined {
    return getCircleRc(rc);
  }

  /**
   * Serialize as (Rep C axis, Rep C trit) pairs for wire encoding.
   *
   * No zeros appear in either axis identifiers (1..13) or trit values (1..3).
   * Any zero in the stream is a sentinel violation.
   */
  toWirePairs(): Array<[number, number]> {
    const tritRc = this.toRepC();
    return tritRc.map((t, i) => [(i + 1), t] as [number, number]);
  }

  /**
   * Deserialize from wire pairs (Rep C axis, Rep C trit).
   * Returns null if any axis or trit is 0 (sentinel violation).
   */
  static fromWirePairs(pairs: Array<[number, number]>): MetatronicVertex | null {
    if (pairs.length !== METATRONIC_DIM) return null;
    const repA = new Array<Trit>(METATRONIC_DIM).fill(0 as Trit);
    for (const [axisRc, tritRc] of pairs) {
      const axis = axisFromRepC(axisRc);
      if (axis === null) return null; // sentinel violation
      if (tritRc < 1 || tritRc > 3) return null; // 0 = sentinel, >3 = invalid
      repA[axis] = (tritRc - 2) as Trit;
    }
    return MetatronicVertex.fromRepA(repA);
  }

  /** Saturnian-weighted norm: Σ |xᵢ| × weight(axis_i). */
  saturnianNorm(): number {
    return this.coords.reduce((sum, c, i) =>
      sum + Math.abs(c) * SATURNIAN_WEIGHTS[i], 0 as number);
  }

  /** Hamming distance (number of differing coordinates). */
  hammingDistance(other: MetatronicVertex): number {
    return this.coords.reduce((count, c, i) =>
      count + (c !== other.coords[i] ? 1 : 0), 0 as number);
  }

  /** Saturnian-weighted distance. */
  saturnianDistance(other: MetatronicVertex): number {
    return this.coords.reduce((dist, c, i) =>
      dist + (c !== other.coords[i] ? SATURNIAN_WEIGHTS[i] : 0), 0 as number);
  }

  /** Mirror vertex (opposite shell, same intra-shell coords). */
  mirrorVertex(): MetatronicVertex {
    const coords = [...this.coords] as Trit[];
    coords[DEPTH_AXIS] = (-coords[DEPTH_AXIS]) as Trit;
    return new MetatronicVertex(coords);
  }

  /** Rep C sentinel check: valid if constructed through fromRepC(). */
  repCValid(): boolean {
    return this.toRepC().every(c => c >= 1 && c <= 3);
  }
}

// ══════════════════════════════════════════════════════════════
// CORRESPONDENCE EDGES
// ══════════════════════════════════════════════════════════════

/** A correspondence edge between shells. */
export interface CorrespondenceEdge {
  /** Shared 12D coordinates. */
  shellCoords: Trit[];
  /** Source shell. */
  from: SaturnianShell;
  /** Target shell. */
  to: SaturnianShell;
}

/** Whether a correspondence edge is direct (adjacent shells). */
export function isDirectCorrespondence(edge: CorrespondenceEdge): boolean {
  const { from, to } = edge;
  return (
    (from === 'inner' && to === 'void') ||
    (from === 'void' && to === 'outer') ||
    (from === 'void' && to === 'inner') ||
    (from === 'outer' && to === 'void')
  );
}

// ══════════════════════════════════════════════════════════════
// Z₂₈ ANGULAR RELATIONSHIPS ON THE CUBE
// ══════════════════════════════════════════════════════════════

/**
 * Map a pair of Metatronic axes to an angular position in Z₂₈.
 *
 * Accepts **internal** (0-indexed) axis indices.
 * Use `axisAngleRc()` for Rep C (1-based) input.
 */
export function axisAngle(axisA: number, axisB: number): Z28 {
  const wa = SATURNIAN_WEIGHTS[axisA] ?? 0;
  const wb = SATURNIAN_WEIGHTS[axisB] ?? 0;
  return Z28.from((wa + wb) % CYCLIC_ORDER);
}

/**
 * Map a pair of Metatronic axes (in **Rep C**, 1-based) to Z₂₈.
 * Returns null if either axis is 0 (sentinel violation) or > 13.
 */
export function axisAngleRc(aRc: number, bRc: number): Z28 | null {
  const a = axisFromRepC(aRc);
  const b = axisFromRepC(bRc);
  if (a === null || b === null) return null;
  return axisAngle(a, b);
}

/**
 * The Z₂₈ angular signature of a vertex: sum of all non-zero axis
 * weights, mod 28. Gives a cyclic "fingerprint" of the vertex's position.
 */
export function vertexAngularSignature(v: MetatronicVertex): Z28 {
  const total = v.coords.reduce((sum, c, i) =>
    sum + (c !== 0 ? SATURNIAN_WEIGHTS[i] : 0), 0 as number);
  return Z28.from(total % CYCLIC_ORDER);
}

// ══════════════════════════════════════════════════════════════
// SATURNIAN ROUND CONSTANTS
// ══════════════════════════════════════════════════════════════

/**
 * The Saturnian trit constant for a given sponge-state position and round.
 *
 * Pattern [0, -1, 1, 1, 0, -1, -1, 1, 0] from the Magic Square,
 * rotated by 3 positions per round (one circulant row shift).
 */
export function saturnianRoundConstant(position: number, round: number): Trit {
  const patternPos = (position + round * 3) % 9;
  return SATURNIAN_TRIT_CONSTANTS[patternPos];
}

/**
 * Expand Saturnian trit constants to fill a 729-element sponge state.
 * Round index determines the circulant rotation.
 */
export function saturnianSpongeConstants(round: number = 0): Trit[] {
  const constants: Trit[] = new Array(729);
  for (let i = 0; i < 729; i++) {
    constants[i] = SATURNIAN_TRIT_CONSTANTS[(i + round * 3) % 9];
  }
  return constants;
}

// ══════════════════════════════════════════════════════════════
// SPONGE STATE ↔ METATRONIC EMBEDDING
// ══════════════════════════════════════════════════════════════

/**
 * Map a sponge state index (0..728) to a Metatronic vertex.
 *
 * The 6 sponge dimensions embed into inner-ring axes (1..6).
 * All other coordinates are zero → Void shell, Central origin.
 */
export function spongeToMetatronic(spongeIndex: number): MetatronicVertex {
  const coords: Trit[] = new Array(METATRONIC_DIM).fill(0) as Trit[];
  let idx = spongeIndex;
  for (let axis = 1; axis <= 6; axis++) {
    coords[axis] = ((idx % 3) - 1) as Trit;
    idx = Math.floor(idx / 3);
  }
  return MetatronicVertex.fromRepA(coords)!;
}

/**
 * Map a Metatronic vertex back to a sponge index (inner-ring axes only).
 */
export function metatronicToSponge(v: MetatronicVertex): number {
  let idx = 0;
  let power = 1;
  for (let axis = 1; axis <= 6; axis++) {
    idx += (v.coords[axis] + 1) * power;
    power *= 3;
  }
  return idx;
}

// ══════════════════════════════════════════════════════════════
// EMBEDDED POLYTOPE ENUMERATION
// ══════════════════════════════════════════════════════════════

/**
 * Enumerate all C(13,4) = 715 axis-selections for ternary tesseracts.
 * Each selection defines a "family" of 3^9 = 19,683 tesseracts.
 * Returns **internal** (0-indexed) axis indices.
 * Use `tesseractFamilyRc()` to convert individual families to Rep C.
 */
export function enumerateTesseractFamilies(): number[][] {
  const families: number[][] = [];
  for (let a = 0; a < 10; a++) {
    for (let b = a + 1; b < 11; b++) {
      for (let c = b + 1; c < 12; c++) {
        for (let d = c + 1; d < 13; d++) {
          families.push([a, b, c, d]);
        }
      }
    }
  }
  return families;
}

/**
 * Convert a tesseract family's axis indices to **Rep C** (1-based bijective).
 * Zero axis IDs cannot appear — sentinel property upheld.
 */
export function tesseractFamilyRc(family: number[]): number[] {
  return family.map(ax => axisToRepC(ax)!);
}

/**
 * Whether a tesseract family spans multiple shells.
 * True if the depth axis (internal 12 / RC 13) is one of the free axes.
 * Accepts **internal** (0-indexed) axis indices.
 */
export function isTransShellFamily(freeAxes: number[]): boolean {
  return freeAxes.includes(DEPTH_AXIS);
}

/**
 * Count trans-shell tesseract families: C(12,3) = 220.
 */
export function countTransShellFamilies(): number {
  return enumerateTesseractFamilies().filter(isTransShellFamily).length;
}

// ══════════════════════════════════════════════════════════════
// STRUCTURED PROJECTION MATRIX (12D → 3D)
// ══════════════════════════════════════════════════════════════

/**
 * Structured orthonormal projection matrix for visualizing the 12D
 * intra-shell point cloud with hexagonal (6-fold) Metatronic symmetry.
 *
 * Replaces the random projection `np.random.seed(42)` used in the
 * original Python code. Designed to preserve:
 * - 6-fold rotational symmetry (inner ring's hexagonal structure)
 * - Block ±1 opposition (Saturn's hexagonal polar vortex)
 * - Periodic harmonic content near 6 cycles per 12 elements
 *
 * Row 0: 6-vs-6 block (strong ± grouping)
 * Row 1: Sinusoidal pattern (~6-fold harmonic)
 * Row 2: Complementary orthogonal direction
 *
 * All rows are unit-norm, mutual dot products ≈ 0.
 */
export const METATRONIC_PROJECTION_MATRIX: readonly (readonly number[])[] = [
  // Row 0: 6 vs 6 block
  [
    0.40824829,  0.40824829,  0.40824829,  0.40824829,  0.40824829,  0.40824829,
   -0.40824829, -0.40824829, -0.40824829, -0.40824829, -0.40824829, -0.40824829,
  ],
  // Row 1: sinusoidal (~6-fold harmonic content)
  [
    0.02086713,  0.40873551,  0.34311983, -0.09926404, -0.40119415, -0.20966289,
    0.20966289,  0.40119415,  0.09926404, -0.34311983, -0.40873551, -0.02086713,
  ],
  // Row 2: complementary orthogonal
  [
    0.39223227,  0.16293917, -0.25685751, -0.37634411, -0.05582047,  0.32996678,
    0.32996678, -0.05582047, -0.37634411, -0.25685751,  0.16293917,  0.39223227,
  ],
] as const;

/**
 * Project a 12D point (intra-shell coords of a vertex) to 3D
 * using the structured Metatronic projection matrix.
 *
 * @param coords12d The 12 intra-shell coordinates (axes 0..11), as numbers.
 * @returns [x, y, z] in projected space.
 */
export function projectTo3D(coords12d: number[]): [number, number, number] {
  if (coords12d.length !== 12) {
    throw new Error(`Expected 12 coordinates, got ${coords12d.length}`);
  }
  const result: [number, number, number] = [0, 0, 0];
  for (let row = 0; row < 3; row++) {
    let dot = 0;
    for (let col = 0; col < 12; col++) {
      dot += METATRONIC_PROJECTION_MATRIX[row][col] * coords12d[col];
    }
    result[row] = dot;
  }
  return result;
}

/**
 * Project a MetatronicVertex to 3D using perspective scaling by shell.
 *
 * Inner shell (x₁₂ = -1): scale = 1 / (1 + depth_scale)
 * Void shell  (x₁₂ = 0):  scale = 1
 * Outer shell (x₁₂ = +1): scale = 1 + depth_scale
 *
 * @param v The vertex to project.
 * @param depthScale How much the depth axis affects size (default 0.5).
 */
export function projectVertex(
  v: MetatronicVertex,
  depthScale: number = 0.5
): [number, number, number] {
  const shellCoords = v.shellCoords().map(c => c as number);
  const [x, y, z] = projectTo3D(shellCoords);

  const depthTrit = v.coords[DEPTH_AXIS];
  const scale = 1 + depthTrit * depthScale;

  return [x * scale, y * scale, z * scale];
}

// ══════════════════════════════════════════════════════════════
// VERIFICATION CONSTANTS
// ══════════════════════════════════════════════════════════════

/**
 * Structural identities that must hold. Import and call in tests.
 */
export const METATRONIC_IDENTITIES = {
  /** 3^13 = total vertices */
  totalVertices: Math.pow(3, METATRONIC_DIM) === METATRONIC_VERTICES,
  /** 3^12 = shell vertices */
  shellVertices: Math.pow(3, 12) === SHELL_VERTICES,
  /** 3 shells × shell_size = total */
  shellPartition: 3 * SHELL_VERTICES === METATRONIC_VERTICES,
  /** Radian × Z₂₈ order = full circle */
  circleIdentity: RADIAN_DEG * CYCLIC_ORDER === FULL_CIRCLE_DEG,
  /** π in ternary circle */
  piTernary: PI_TERNARY === PI_ESOTERIC,
  /** Saturnian magic sum */
  magicSum: PLENUM_SQUARE_MATRIX[0].reduce((a, b) => a + b, 0) === MAGIC_CONSTANT,
  /** Weight sum */
  weightSum: TOTAL_SATURNIAN_WEIGHT === 1568,
  /** Trit constants sum to zero (balanced) */
  tritBalance: SATURNIAN_TRIT_CONSTANTS.reduce((a: number, b: number) => a + b, 0) === 0,
  /** Tesseract count */
  tesseractCount: TESSERACT_FAMILIES * 19_683 === TESSERACT_TOTAL,
  /** Year days from Saturnian constants */
  yearDays: PLENUM_NATURAL_YEAR_DAYS === FULL_CIRCLE_DEG,
  /** Depth axis RC = T₇ = one ternary radian = 13 */
  depthAxisRcIsRadian: DEPTH_AXIS_RC === RADIAN_DEG,
  /** All circle axisRc fields are 1-based and match axis + 1 */
  circleRcConsistent: METATRONIC_CIRCLES.every(c => c.axisRc === c.axis + 1),
  /** No zero axis RC in any circle (sentinel property) */
  circleRcNoZero: METATRONIC_CIRCLES.every(c => c.axisRc >= 1),
} as const;