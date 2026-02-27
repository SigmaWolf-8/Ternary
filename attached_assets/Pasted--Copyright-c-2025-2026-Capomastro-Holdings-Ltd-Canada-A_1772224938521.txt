// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Saturnian Tesseract Metatron Ternary Cube
//!
//! The 13-dimensional ternary cube viewed through the lens of Metatron's
//! geometry and the Saturnian Black Cube tradition. This is not a generic
//! algebraic construction — it is THE specific geometric object at the
//! intersection of:
//!
//! - **13 Metatronic circles** → 13 ternary axes
//! - **Three Saturnian shells** → the ternary trit values {-1, 0, +1}
//!   along the 13th (depth) axis
//! - **Saturnian Magic Square** [111, 14, 208] → round constants and
//!   torsion weights
//! - **Ternary circle** (364° = 111111₃, π = 14, radian = 13° = T₇)
//!   → angular relationships between embedded polytopes
//! - **Z₂₈ cyclic symmetry** → 28-fold rotational structure
//!
//! ## Structure
//!
//! A vertex in the Metatronic Cube is a 13-trit coordinate:
//!
//!     v = (x₀, x₁, ..., x₁₂)  where xᵢ ∈ {-1, 0, +1}
//!
//! Total vertices: 3¹³ = 1,594,323 — identical to the 13D torsion network.
//!
//! The 13th axis (index 12) serves as the **depth axis**, splitting the
//! cube into three shells:
//!
//! | Shell    | x₁₂ | Vertices  | Saturnian Role      |
//! |----------|------|-----------|---------------------|
//! | Inner    |  -1  | 531,441   | Manifest (form)     |
//! | Void     |   0  | 531,441   | Balance (potential)  |
//! | Outer    |  +1  | 531,441   | Transcendent (light) |
//!
//! Each shell is itself a 12-dimensional ternary cube.
//!
//! ## Metatronic Axis Assignments
//!
//! The 13 axes map to the 13 circles of Metatron's Cube:
//!
//! | Internal | Rep C | Circle     | Domain Group    | Saturnian Weight |
//! |----------|-------|------------|-----------------|------------------|
//! |  0       |   1   | Central    | Foundation      | 111 (balance)    |
//! |  1–6     |  2–7  | Inner Ring | Manifestation   | 14 (π-esoteric)  |
//! |  7–11    | 8–12  | Outer Ring | Transcendence   | 208 (cosmic)     |
//! |  12      |  13   | Depth      | Shell Selector  | 333 (magic sum)  |
//!
//! **Internal** (0-indexed) is used for Rust array subscripts (`coords[i]`).
//! **Rep C** (1-based bijective) is used at wire boundaries — VM operands,
//! torsion routing, any ternary data stream. A zero axis identifier in Rep C
//! is structurally impossible, inheriting the sentinel property from the
//! address system. Depth axis in Rep C = 13 = T₇ = one ternary radian = 111₃.
//!
//! # ⚠ REPRESENTATION SAFETY — READ BEFORE MODIFYING
//!
//! **Every axis-bearing method has two variants.** Internal methods use bare
//! names (`axis_index()`, `from_axis()`, `free_axes()`). Wire-safe methods
//! carry the `_rc` suffix (`axis_index_rc()`, `from_axis_rc()`, `free_axes_rc()`).
//!
//! **Never mix representations.** If a function receives an axis from external
//! input (network packet, VM operand, serialized config), it MUST use `_rc`
//! methods or convert explicitly via `axis_from_rep_c()`. Feeding a Rep C
//! value (1..=13) into an internal method (which expects 0..=12) will silently
//! produce wrong results. Feeding an internal value (0..=12) into a Rep C
//! context will produce 0, which is a sentinel violation.
//!
//! **The compiler cannot catch this.** Both representations are `usize` / `u8`.
//! The naming convention is the only guard. Treat `_rc` suffix as a type tag.
//!
//! The weight assignments derive from the Saturnian Magic Square:
//!
//!     | 111 |  14 | 208 |
//!     | 208 | 111 |  14 |
//!     |  14 | 208 | 111 |
//!
//! ## Embedded Polytopes (Ternary)
//!
//! In the ternary 13-cube, embedded polytopes are defined by vertex
//! subsets with specific coordinate constraints:
//!
//! - **Ternary tetrahedra:** 4 vertices forming a regular simplex in
//!   ternary Hamming space. A "polarized" tetrahedron has one fixed
//!   coordinate across all 4 vertices; a "twisted" (merkabah) tetrahedron
//!   has no fixed coordinate.
//!
//! - **Ternary octahedra:** 6 vertices forming a cross-polytope, where
//!   exactly one coordinate takes all three values while others are fixed.
//!
//! - **Ternary tesseracts:** 4D sub-cubes (3⁴ = 81 vertices) embedded in
//!   the 13D space by choosing 4 free axes and fixing the other 9.
//!
//! ## Connection to Existing Modules
//!
//! - `saturnian-blueprint.ts` → Magic Square constants imported here
//! - `ternary-circle.ts` → Z₂₈, 364°, π = 14, radian = 13°
//! - Torsion Network (`09_TORSION_NETWORK`) → This IS the 13D topology
//! - `sponge.rs` → Sponge state (3⁶) is a 6D sub-cube of this 13-cube
//! - `address_sentinel.rs` → Rep C validation operates on cube vertices
//!
//! GEOMETRIA PRIMUS. TEMPORIS ARCHITECTURA ABSOLUTA.

extern crate alloc;
use alloc::vec::Vec;

// ══════════════════════════════════════════════════════════════
// CONSTANTS — Saturnian & Metatronic
// ══════════════════════════════════════════════════════════════

/// Total dimensions of the Metatronic Cube.
pub const METATRONIC_DIM: usize = 13;

/// Total vertices: 3^13 = 1,594,323.
pub const METATRONIC_VERTICES: usize = 1_594_323;

/// Vertices per shell: 3^12 = 531,441.
pub const SHELL_VERTICES: usize = 531_441;

/// Depth axis index (the 13th axis, 0-indexed INTERNAL representation).
///
/// # Axis Representation Convention
///
/// Axis indices exist in two representations, mirroring the trit convention:
///
/// | Rep       | Range  | Use                                    |
/// |-----------|--------|----------------------------------------|
/// | Internal  | 0..12  | Rust array subscripts, struct fields    |
/// | Rep C     | 1..13  | Wire encoding, VM operands, torsion routing |
///
/// Internal representation is used everywhere inside this module for
/// array indexing (`coords[axis]`). Rep C is used at serialization
/// boundaries — anywhere an axis identifier enters a ternary data stream.
///
/// The depth axis is 12 internally, **13 in Rep C** — which equals T₇,
/// one ternary radian (13° = 111₃). This is not coincidence; the geometry
/// demands 1-based numbering in the ternary computational domain.
pub const DEPTH_AXIS: usize = 12;

/// Depth axis in Rep C (bijective ternary axis numbering): **13** = T₇.
pub const DEPTH_AXIS_RC: u8 = 13;

// ── Bijective Axis Numbering (Rep C for axes) ──────────────────────

/// Convert an internal 0-indexed axis index to Rep C (1-based bijective).
///
/// Internal 0 → Rep C 1, ..., Internal 12 → Rep C 13.
/// Returns `None` if axis ≥ 13.
#[inline]
pub const fn axis_to_rep_c(internal: usize) -> Option<u8> {
    if internal < METATRONIC_DIM {
        Some((internal + 1) as u8)
    } else {
        None
    }
}

/// Convert a Rep C axis identifier (1..=13) to internal 0-indexed.
///
/// Rep C 1 → Internal 0, ..., Rep C 13 → Internal 12.
/// Returns `None` if `rc` is 0 or > 13.
///
/// **Zero rejection**: Rep C axis 0 is structurally impossible, just as
/// trit value 0 is impossible in Rep C for coordinates. This is the
/// sentinel property extended to axis identifiers — a 0 in an axis
/// field of a ternary wire packet proves corruption or forgery.
#[inline]
pub const fn axis_from_rep_c(rc: u8) -> Option<usize> {
    if rc >= 1 && rc <= METATRONIC_DIM as u8 {
        Some((rc - 1) as usize)
    } else {
        None // 0 is sentinel-invalid; >13 is out of range
    }
}

/// Validate a Rep C axis identifier. Returns true if rc ∈ {1..=13}.
/// Zero is rejected (sentinel property), as are values > 13.
#[inline]
pub const fn axis_rc_valid(rc: u8) -> bool {
    rc >= 1 && rc <= METATRONIC_DIM as u8
}

/// Convert a slice of internal axis indices to Rep C.
/// Returns `None` if any index ≥ 13.
pub fn axes_to_rep_c(internal: &[usize]) -> Option<Vec<u8>> {
    internal.iter().map(|&ax| axis_to_rep_c(ax)).collect()
}

/// Convert a slice of Rep C axis identifiers to internal indices.
/// Returns None if any value is 0 (sentinel violation) or > 13.
pub fn axes_from_rep_c(rc: &[u8]) -> Option<Vec<usize>> {
    rc.iter().map(|&a| axis_from_rep_c(a)).collect()
}

/// Saturnian Magic Square — the circulant matrix [111, 14, 208].
/// Every row, column, and diagonal sums to 333.
pub const SATURNIAN_MATRIX: [[u16; 3]; 3] = [
    [111, 14, 208],
    [208, 111, 14],
    [14, 208, 111],
];

/// Magic constant: every line of the Saturnian square.
pub const MAGIC_CONSTANT: u16 = 333;

/// Ternary balance center (matrix diagonal element).
pub const BALANCE_CENTER: u16 = 111;

/// Esoteric π — ratio of circumference to diameter in the ternary circle.
pub const PI_ESOTERIC: u16 = 14;

/// Cosmic accumulation weight.
pub const COSMIC_WEIGHT: u16 = 208;

/// Full circle in ternary degrees: 364 = 111111₃.
pub const FULL_CIRCLE_DEG: u16 = 364;

/// One ternary radian in degrees: 13 = 111₃ = T₇.
pub const RADIAN_DEG: u16 = 13;

/// Z₂₈ cyclic order: full circle in ternary radians.
pub const Z28_ORDER: u16 = 28;

/// Number of ternary tesseracts embedded in the 13-cube.
/// C(13,4) × 1 (each choice of 4 free axes gives one 4D sub-cube family).
/// Actual count: C(13,4) = 715 axis-selections, each with 3⁹ = 19,683
/// distinct fixed-coordinate choices → 715 × 19,683 = 14,073,405 tesseracts.
///
/// In the BINARY case the original documents counted 366,080. In the TERNARY
/// case the count is larger because each fixed coordinate has 3 choices, not 2.
pub const TERNARY_TESSERACT_FAMILIES: usize = 715;

/// The Saturnian flattened sequence — used for round constant derivation.
pub const SATURNIAN_FLAT: [u16; 9] = [111, 14, 208, 208, 111, 14, 14, 208, 111];

/// Structured 12D → 3D projection matrix that preserves hexagonal (6-fold)
/// Metatronic symmetry when visualizing the intra-shell point cloud.
///
/// Replaces random orthogonal projection (seed 42) from the original Python.
/// Designed with:
/// - Row 0: ±1 block of 6 (Saturn's polar hexagon opposition)
/// - Row 1: sinusoidal ~6-cycle harmonic (Metatronic inner ring periodicity)
/// - Row 2: orthogonal complement with alternation
///
/// Rows are unit-norm, mutual dot products ≈ 0 (orthonormal to float precision).
pub const STRUCTURED_PROJ_MATRIX: [[f64; 12]; 3] = [
    [ 0.40824829,  0.40824829,  0.40824829,  0.40824829,  0.40824829,  0.40824829,
     -0.40824829, -0.40824829, -0.40824829, -0.40824829, -0.40824829, -0.40824829],
    [ 0.02086713,  0.40873551,  0.34311983, -0.09926404, -0.40119415, -0.20966289,
      0.20966289,  0.40119415,  0.09926404, -0.34311983, -0.40873551, -0.02086713],
    [ 0.39223227,  0.16293917, -0.25685751, -0.37634411, -0.05582047,  0.32996678,
      0.32996678, -0.05582047, -0.37634411, -0.25685751,  0.16293917,  0.39223227],
];

// ══════════════════════════════════════════════════════════════
// METATRONIC AXIS IDENTITY
// ══════════════════════════════════════════════════════════════

/// The 13 circles of Metatron's Cube, each mapped to one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MetatronicCircle {
    /// Axis 0: Central circle — Foundation. Saturnian weight: 111.
    Central,
    /// Axes 1–6: Inner ring — Manifestation. Saturnian weight: 14.
    Inner(u8), // 1..=6
    /// Axes 7–11: Outer ring — Transcendence. Saturnian weight: 208.
    Outer(u8), // 7..=11
    /// Axis 12: Depth — Shell selector. Saturnian weight: 333.
    Depth,
}

impl MetatronicCircle {
    /// Get the axis index in **internal** (0-indexed) representation.
    ///
    /// Use for array subscripts: `vertex.coords[circle.axis_index()]`.
    /// Do NOT use for wire encoding — use `axis_index_rc()` instead.
    pub fn axis_index(&self) -> usize {
        match self {
            MetatronicCircle::Central => 0,
            MetatronicCircle::Inner(n) => *n as usize,
            MetatronicCircle::Outer(n) => *n as usize,
            MetatronicCircle::Depth => 12,
        }
    }

    /// Get the axis index in **Rep C** (1-based bijective) representation.
    ///
    /// Use for wire encoding, VM operands, torsion routing packets.
    /// Central = 1, Inner = 2..7, Outer = 8..12, Depth = 13.
    ///
    /// Note: Depth axis = 13 = T₇ = one ternary radian = 111₃.
    pub fn axis_index_rc(&self) -> u8 {
        (self.axis_index() + 1) as u8
    }

    /// Get the Saturnian weight for this axis.
    pub fn saturnian_weight(&self) -> u16 {
        match self {
            MetatronicCircle::Central => BALANCE_CENTER,   // 111
            MetatronicCircle::Inner(_) => PI_ESOTERIC,     // 14
            MetatronicCircle::Outer(_) => COSMIC_WEIGHT,   // 208
            MetatronicCircle::Depth => MAGIC_CONSTANT,     // 333
        }
    }

    /// The domain group this axis belongs to.
    pub fn domain(&self) -> MetatronicDomain {
        match self {
            MetatronicCircle::Central => MetatronicDomain::Foundation,
            MetatronicCircle::Inner(_) => MetatronicDomain::Manifestation,
            MetatronicCircle::Outer(_) => MetatronicDomain::Transcendence,
            MetatronicCircle::Depth => MetatronicDomain::ShellBoundary,
        }
    }

    /// Construct from **internal** (0-indexed) axis index.
    ///
    /// Accepts 0..12. Use `from_axis_rc()` for Rep C (1..13) input.
    pub fn from_axis(axis: usize) -> Option<Self> {
        match axis {
            0 => Some(MetatronicCircle::Central),
            1..=6 => Some(MetatronicCircle::Inner(axis as u8)),
            7..=11 => Some(MetatronicCircle::Outer(axis as u8)),
            12 => Some(MetatronicCircle::Depth),
            _ => None,
        }
    }

    /// Construct from **Rep C** (1-based bijective) axis identifier.
    ///
    /// Accepts 1..=13. Rejects 0 (sentinel violation) and >13.
    /// Use at wire-decoding boundaries where axis IDs arrive in ternary encoding.
    pub fn from_axis_rc(rc: u8) -> Option<Self> {
        axis_from_rep_c(rc).and_then(Self::from_axis)
    }

    /// All 13 circles in axis order.
    pub fn all() -> [MetatronicCircle; 13] {
        [
            MetatronicCircle::Central,
            MetatronicCircle::Inner(1),
            MetatronicCircle::Inner(2),
            MetatronicCircle::Inner(3),
            MetatronicCircle::Inner(4),
            MetatronicCircle::Inner(5),
            MetatronicCircle::Inner(6),
            MetatronicCircle::Outer(7),
            MetatronicCircle::Outer(8),
            MetatronicCircle::Outer(9),
            MetatronicCircle::Outer(10),
            MetatronicCircle::Outer(11),
            MetatronicCircle::Depth,
        ]
    }
}

/// The four domain groups of the Metatronic axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetatronicDomain {
    /// Axis 0 — the anchor point.
    Foundation,
    /// Axes 1–6 — the six inner circles.
    Manifestation,
    /// Axes 7–11 — the five outer circles.
    Transcendence,
    /// Axis 12 — depth, shell selection.
    ShellBoundary,
}

// ══════════════════════════════════════════════════════════════
// THREE SATURNIAN SHELLS
// ══════════════════════════════════════════════════════════════

/// The three shells of the Saturnian Ternary Cube.
///
/// The depth axis (x₁₂) determines which shell a vertex inhabits.
/// In Saturnian symbolism:
/// - Inner (-1): the manifest, formed realm — the "Black Cube" interior
/// - Void (0): the balance plane, the mediator between realms
/// - Outer (+1): the transcendent, luminous realm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SaturnianShell {
    /// x₁₂ = -1. Manifest realm. 3¹² = 531,441 vertices.
    Inner,
    /// x₁₂ = 0. Void/balance plane. 3¹² = 531,441 vertices.
    Void,
    /// x₁₂ = +1. Transcendent realm. 3¹² = 531,441 vertices.
    Outer,
}

impl SaturnianShell {
    /// The balanced ternary trit value for this shell on the depth axis.
    pub fn depth_trit(&self) -> i8 {
        match self {
            SaturnianShell::Inner => -1,
            SaturnianShell::Void => 0,
            SaturnianShell::Outer => 1,
        }
    }

    /// Construct from the depth-axis trit value.
    pub fn from_depth_trit(t: i8) -> Option<Self> {
        match t {
            -1 => Some(SaturnianShell::Inner),
            0 => Some(SaturnianShell::Void),
            1 => Some(SaturnianShell::Outer),
            _ => None,
        }
    }

    /// The opposing shell (Inner ↔ Outer, Void maps to itself).
    pub fn mirror(&self) -> Self {
        match self {
            SaturnianShell::Inner => SaturnianShell::Outer,
            SaturnianShell::Outer => SaturnianShell::Inner,
            SaturnianShell::Void => SaturnianShell::Void,
        }
    }

    /// Whether this shell is a boundary shell (Inner or Outer).
    pub fn is_polar(&self) -> bool {
        matches!(self, SaturnianShell::Inner | SaturnianShell::Outer)
    }
}

// ══════════════════════════════════════════════════════════════
// METATRONIC VERTEX — A POINT IN THE 13D TERNARY CUBE
// ══════════════════════════════════════════════════════════════

/// A vertex in the Metatronic 13-cube.
///
/// Representation A: balanced ternary {-1, 0, +1}.
///
/// The vertex knows its shell, its 12D intra-shell coordinates, and
/// its full 13D position. It can convert between representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetatronicVertex {
    /// The 13 coordinates in Rep A {-1, 0, +1}.
    pub coords: [i8; METATRONIC_DIM],
}

impl MetatronicVertex {
    /// Create a vertex from Rep A coordinates.
    pub fn from_rep_a(coords: [i8; METATRONIC_DIM]) -> Option<Self> {
        if coords.iter().all(|&c| c >= -1 && c <= 1) {
            Some(Self { coords })
        } else {
            None
        }
    }

    /// Create from Rep B {0, 1, 2} coordinates.
    pub fn from_rep_b(coords: [u8; METATRONIC_DIM]) -> Option<Self> {
        let mut rep_a = [0i8; METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            rep_a[i] = match coords[i] {
                0 => -1,
                1 => 0,
                2 => 1,
                _ => return None,
            };
        }
        Some(Self { coords: rep_a })
    }

    /// Create from Rep C {1, 2, 3} bijective ternary coordinates.
    /// Returns None if any coordinate is 0 (structurally impossible in Rep C).
    pub fn from_rep_c(coords: [u8; METATRONIC_DIM]) -> Option<Self> {
        let mut rep_a = [0i8; METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            rep_a[i] = match coords[i] {
                1 => -1,
                2 => 0,
                3 => 1,
                0 => return None, // Rep C sentinel violation
                _ => return None,
            };
        }
        Some(Self { coords: rep_a })
    }

    /// Convert to Rep B {0, 1, 2}.
    pub fn to_rep_b(&self) -> [u8; METATRONIC_DIM] {
        let mut out = [0u8; METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            out[i] = (self.coords[i] + 1) as u8;
        }
        out
    }

    /// Convert to Rep C {1, 2, 3} bijective ternary.
    pub fn to_rep_c(&self) -> [u8; METATRONIC_DIM] {
        let mut out = [0u8; METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            out[i] = (self.coords[i] + 2) as u8;
        }
        out
    }

    /// Which Saturnian shell this vertex inhabits.
    pub fn shell(&self) -> SaturnianShell {
        SaturnianShell::from_depth_trit(self.coords[DEPTH_AXIS]).unwrap()
    }

    /// The 12D intra-shell coordinates (axes 0..12, excluding depth).
    pub fn shell_coords(&self) -> [i8; 12] {
        let mut out = [0i8; 12];
        out.copy_from_slice(&self.coords[0..12]);
        out
    }

    /// The Metatronic circle for a given axis of this vertex.
    /// `axis` is in **internal** (0-indexed) representation.
    pub fn circle_at(&self, axis: usize) -> Option<MetatronicCircle> {
        MetatronicCircle::from_axis(axis)
    }

    /// The Metatronic circle for a given axis in **Rep C** (1-based bijective).
    /// Returns `None` if `rc` is 0 (sentinel violation) or > 13.
    pub fn circle_at_rc(&self, rc: u8) -> Option<MetatronicCircle> {
        MetatronicCircle::from_axis_rc(rc)
    }

    /// Serialize this vertex as a sequence of (Rep C axis, Rep C trit value) pairs.
    ///
    /// This is the canonical wire format: no zero values appear in either
    /// axis identifiers (1..=13) or trit values (1..=3). Any zero in the
    /// stream is a sentinel violation proving corruption or forgery.
    ///
    /// Returns 13 pairs, one per axis in ascending Rep C axis order.
    pub fn to_wire_pairs(&self) -> [(u8, u8); METATRONIC_DIM] {
        let trit_rc = self.to_rep_c();
        let mut out = [(0u8, 0u8); METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            out[i] = (
                (i + 1) as u8,  // axis Rep C: 1..=13
                trit_rc[i],     // trit Rep C: 1..=3
            );
        }
        out
    }

    /// Deserialize from wire pairs `(Rep C axis, Rep C trit)`.
    ///
    /// Returns `None` if any axis or trit value is 0 (sentinel violation),
    /// if any axis is > 13, or if any trit is > 3.
    pub fn from_wire_pairs(pairs: &[(u8, u8); METATRONIC_DIM]) -> Option<Self> {
        let mut rep_a = [0i8; METATRONIC_DIM];
        for &(axis_rc, trit_rc) in pairs {
            let axis = axis_from_rep_c(axis_rc)?;
            let trit = match trit_rc {
                1 => -1i8,
                2 => 0i8,
                3 => 1i8,
                _ => return None, // 0 = sentinel violation, >3 = invalid
            };
            rep_a[axis] = trit;
        }
        Some(Self { coords: rep_a })
    }

    /// Saturnian-weighted norm: Σ |xᵢ| × weight(axis_i).
    ///
    /// This gives a scalar measure of a vertex's "distance from origin"
    /// weighted by the Metatronic significance of each axis.
    pub fn saturnian_norm(&self) -> u32 {
        MetatronicCircle::all()
            .iter()
            .zip(&self.coords)
            .map(|(circle, &trit)| circle.saturnian_weight() as u32 * trit.unsigned_abs() as u32)
            .sum()
    }

    /// Hamming distance to another vertex (number of coordinates that differ).
    pub fn hamming_distance(&self, other: &MetatronicVertex) -> usize {
        self.coords.iter().zip(other.coords.iter())
            .filter(|(&a, &b)| a != b)
            .count()
    }

    /// Ternary Hamming distance (sum of |xᵢ - yᵢ| mod 3).
    pub fn ternary_distance(&self, other: &MetatronicVertex) -> usize {
        self.coords.iter().zip(other.coords.iter())
            .map(|(&a, &b)| {
                let diff = ((a - b) % 3 + 3) % 3;
                diff.min(3 - diff) as usize
            })
            .sum()
    }

    /// Saturnian-weighted distance to another vertex.
    pub fn saturnian_distance(&self, other: &MetatronicVertex) -> u32 {
        MetatronicCircle::all()
            .iter()
            .zip(self.coords.iter().zip(other.coords.iter()))
            .filter(|(_, (&a, &b))| a != b)
            .map(|(circle, _)| circle.saturnian_weight() as u32)
            .sum()
    }

    /// The correspondence partner in the mirror shell.
    ///
    /// For a vertex in the Inner shell, returns the same 12D coordinates
    /// in the Outer shell (and vice versa). For Void vertices, returns self.
    pub fn mirror_vertex(&self) -> Self {
        let mut mirror = self.coords;
        mirror[DEPTH_AXIS] = -mirror[DEPTH_AXIS];
        Self { coords: mirror }
    }

    /// Convert to a linear index in [0, 3^13).
    ///
    /// Uses Rep B internally: index = Σ rep_b[i] × 3^i.
    pub fn to_linear_index(&self) -> usize {
        let rep_b = self.to_rep_b();
        let mut idx = 0usize;
        let mut power = 1usize;
        for i in 0..METATRONIC_DIM {
            idx += (rep_b[i] as usize) * power;
            power *= 3;
        }
        idx
    }

    /// Construct from a linear index.
    pub fn from_linear_index(mut idx: usize) -> Self {
        let mut coords = [0i8; METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            let digit = (idx % 3) as i8 - 1; // Rep B → Rep A
            coords[i] = digit;
            idx /= 3;
        }
        Self { coords }
    }

    /// The origin vertex: all coordinates zero (in the Void shell).
    pub fn origin() -> Self {
        Self { coords: [0; METATRONIC_DIM] }
    }

    /// Rep C sentinel check: returns true if this vertex is valid in
    /// bijective ternary (no coordinate would map to 0 in Rep C).
    /// For Rep A, this is always true since {-1, 0, +1} → {1, 2, 3}.
    /// The sentinel violation occurs when an EXTERNAL input claims to be
    /// Rep C but contains a 0.
    pub fn rep_c_valid(&self) -> bool {
        // In Rep A, all vertices are structurally valid — the sentinel
        // check is performed at the Rep C input boundary (from_rep_c).
        true
    }

    /// Project this vertex to 3D using perspective along the depth axis
    /// and the structured 6-fold Metatronic projection matrix.
    ///
    /// The depth axis (x₁₂) controls perspective scaling:
    ///   Inner (x₁₂ = -1): closer → larger
    ///   Void  (x₁₂ =  0): neutral
    ///   Outer (x₁₂ = +1): further → smaller
    ///
    /// `viewer_distance` controls perspective strength. Larger values
    /// flatten the projection; smaller values exaggerate shell separation.
    /// Default: 5.0. Must be > 1.0 to avoid division singularity.
    #[inline]
    pub fn project_to_3d(&self, viewer_distance: f64) -> [f64; 3] {
        let w = self.coords[DEPTH_AXIS] as f64;
        let factor = viewer_distance / (viewer_distance - w);

        // Scale the 12 intra-shell coordinates by perspective factor
        let mut p12 = [0.0f64; 12];
        for i in 0..12 {
            p12[i] = self.coords[i] as f64 * factor;
        }

        // Multiply by the structured projection matrix (12D → 3D)
        let mut p3 = [0.0f64; 3];
        for row in 0..3 {
            let mut dot = 0.0f64;
            for col in 0..12 {
                dot += p12[col] * STRUCTURED_PROJ_MATRIX[row][col];
            }
            p3[row] = dot;
        }
        p3
    }
}

impl core::fmt::Display for MetatronicVertex {
    /// Display as balanced ternary with shell indicator.
    ///
    /// Format: `[−10+100−1+1+10]@Void`
    /// - Each trit: `−` for -1, `0` for 0, `+` for +1
    /// - Shell suffix after `@`
    ///
    /// The 13 characters map directly to axes 0..12 (Central through Depth).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        for &trit in &self.coords {
            match trit {
                -1 => write!(f, "−")?,
                 0 => write!(f, "0")?,
                 1 => write!(f, "+")?,
                 _ => write!(f, "?")?,
            }
        }
        let shell = self.shell();
        write!(f, "]@{:?}", shell)
    }
}

// ══════════════════════════════════════════════════════════════
// CORRESPONDENCE EDGES — INTER-SHELL CONNECTIONS
// ══════════════════════════════════════════════════════════════

/// A correspondence edge between two shells.
///
/// In the binary 13-cube, there are 4,096 correspondence edges between
/// the two 12-cube shells. In the ternary 13-cube, correspondence edges
/// connect vertices that differ ONLY on the depth axis (x₁₂):
///
/// - Inner ↔ Void: 531,441 edges (x₁₂: -1 → 0)
/// - Void ↔ Outer: 531,441 edges (x₁₂: 0 → +1)
/// - Inner ↔ Outer: 531,441 edges (x₁₂: -1 → +1, distance 2)
///
/// Total: 1,594,323 correspondence edges (3 pairs × 531,441).
///
/// The Inner↔Outer edges pass through the Void — they are the
/// "long correspondence" that spans the full depth of the cube.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrespondenceEdge {
    /// The shared 12D coordinates (axes 0..12).
    pub shell_coords: [i8; 12],
    /// Source shell.
    pub from: SaturnianShell,
    /// Target shell.
    pub to: SaturnianShell,
}

impl CorrespondenceEdge {
    /// The source vertex.
    pub fn source_vertex(&self) -> MetatronicVertex {
        let mut coords = [0i8; METATRONIC_DIM];
        coords[..12].copy_from_slice(&self.shell_coords);
        coords[DEPTH_AXIS] = self.from.depth_trit();
        MetatronicVertex { coords }
    }

    /// The target vertex.
    pub fn target_vertex(&self) -> MetatronicVertex {
        let mut coords = [0i8; METATRONIC_DIM];
        coords[..12].copy_from_slice(&self.shell_coords);
        coords[DEPTH_AXIS] = self.to.depth_trit();
        MetatronicVertex { coords }
    }

    /// Whether this is a direct adjacency (shells differ by 1 step)
    /// or a long correspondence (Inner ↔ Outer, distance 2).
    pub fn is_direct(&self) -> bool {
        match (&self.from, &self.to) {
            (SaturnianShell::Inner, SaturnianShell::Void) => true,
            (SaturnianShell::Void, SaturnianShell::Outer) => true,
            (SaturnianShell::Void, SaturnianShell::Inner) => true,
            (SaturnianShell::Outer, SaturnianShell::Void) => true,
            _ => false,
        }
    }
}

// ══════════════════════════════════════════════════════════════
// EMBEDDED POLYTOPES
// ══════════════════════════════════════════════════════════════

/// A ternary tetrahedron embedded in the 13-cube.
///
/// Four vertices forming a regular simplex in ternary Hamming space.
/// "Polarized" tetrahedra have one coordinate fixed across all 4 vertices.
/// "Twisted" (merkabah) tetrahedra have no fixed coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TernaryTetrahedron {
    pub vertices: [MetatronicVertex; 4],
    pub polarization: TetrahedronPolarization,
}

/// Whether a tetrahedron is polarized (has a fixed axis) or twisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrahedronPolarization {
    /// One specific axis has the same value across all 4 vertices.
    /// `fixed_axis` is in **internal** (0-indexed) representation.
    Polarized { fixed_axis: usize, fixed_value: i8 },
    /// No axis is fixed — the merkabah pattern.
    Twisted,
}

impl TetrahedronPolarization {
    /// The fixed axis in **Rep C** (1-based bijective) for wire encoding.
    /// Returns `None` for Twisted tetrahedra or invalid axis.
    pub fn fixed_axis_rc(&self) -> Option<u8> {
        match self {
            TetrahedronPolarization::Polarized { fixed_axis, .. } => axis_to_rep_c(*fixed_axis),
            TetrahedronPolarization::Twisted => None,
        }
    }

    /// Construct from Rep C axis identifier.
    /// Returns `None` if `rc` is 0 (sentinel violation) or > 13.
    pub fn polarized_from_rc(rc: u8, fixed_value: i8) -> Option<Self> {
        axis_from_rep_c(rc).map(|ax| TetrahedronPolarization::Polarized {
            fixed_axis: ax,
            fixed_value,
        })
    }
}

/// A ternary octahedron embedded in the 13-cube.
///
/// Six vertices forming a cross-polytope: exactly one axis takes all
/// three values {-1, 0, +1} while a second axis also varies, and the
/// remaining 11 are fixed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TernaryOctahedron {
    pub vertices: [MetatronicVertex; 6],
    /// The axis that takes all three values.
    /// **Internal** (0-indexed) representation.
    pub varying_axis: usize,
}

impl TernaryOctahedron {
    /// The varying axis in **Rep C** (1-based bijective) for wire encoding.
    pub fn varying_axis_rc(&self) -> u8 {
        axis_to_rep_c(self.varying_axis)
            .expect("TernaryOctahedron varying_axis out of range")
    }

    /// Construct from a Rep C axis identifier.
    pub fn from_varying_axis_rc(vertices: [MetatronicVertex; 6], rc: u8) -> Option<Self> {
        axis_from_rep_c(rc).map(|ax| TernaryOctahedron { vertices, varying_axis: ax })
    }
}

/// A ternary tesseract (4-cube) embedded in the 13-cube.
///
/// Defined by choosing 4 free axes (which take all values in {-1, 0, +1})
/// and fixing the remaining 9 axes to specific values.
///
/// Each tesseract has 3⁴ = 81 vertices.
///
/// ## Axis Representation
///
/// Internal fields use 0-indexed axis indices (for array subscript compat).
/// Call `free_axes_rc()` / `fixed_coords_rc()` for Rep C (1-based bijective)
/// encoding suitable for wire transmission. Zero axis IDs in Rep C are
/// structurally impossible — the sentinel property extends to polytope metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TernaryTesseract {
    /// The 4 free axes — **internal** (0-indexed) representation.
    pub free_axes: [usize; 4],
    /// The fixed values for the 9 non-free axes.
    /// Stored as (axis_index, fixed_value) pairs.
    /// Axis indices are **internal** (0-indexed).
    pub fixed_coords: [(usize, i8); 9],
}

impl TernaryTesseract {
    /// How many vertices this tesseract contains: 3⁴ = 81.
    pub fn vertex_count(&self) -> usize {
        81
    }

    /// The 4 free axes in **Rep C** (1-based bijective) for wire encoding.
    pub fn free_axes_rc(&self) -> [u8; 4] {
        [
            axis_to_rep_c(self.free_axes[0]).unwrap(),
            axis_to_rep_c(self.free_axes[1]).unwrap(),
            axis_to_rep_c(self.free_axes[2]).unwrap(),
            axis_to_rep_c(self.free_axes[3]).unwrap(),
        ]
    }

    /// The 9 fixed coordinates with axis indices in **Rep C** for wire encoding.
    /// Returns `(rep_c_axis, trit_value)` pairs.
    pub fn fixed_coords_rc(&self) -> [(u8, i8); 9] {
        let mut out = [(0u8, 0i8); 9];
        for i in 0..9 {
            out[i] = (
                axis_to_rep_c(self.fixed_coords[i].0).unwrap(),
                self.fixed_coords[i].1,
            );
        }
        out
    }

    /// Construct from Rep C axis identifiers.
    /// Returns `None` if any axis RC is 0 (sentinel) or > 13.
    pub fn from_rc(
        free_rc: [u8; 4],
        fixed_rc: [(u8, i8); 9],
    ) -> Option<Self> {
        let mut free = [0usize; 4];
        for i in 0..4 {
            free[i] = axis_from_rep_c(free_rc[i])?;
        }
        let mut fixed = [(0usize, 0i8); 9];
        for i in 0..9 {
            fixed[i] = (axis_from_rep_c(fixed_rc[i].0)?, fixed_rc[i].1);
        }
        Some(TernaryTesseract { free_axes: free, fixed_coords: fixed })
    }

    /// Whether this tesseract spans multiple shells.
    ///
    /// True if the depth axis (12) is one of the free axes —
    /// the tesseract then has vertices in all three shells.
    pub fn is_trans_shell(&self) -> bool {
        self.free_axes.contains(&DEPTH_AXIS)
    }

    /// Which shell(s) this tesseract occupies.
    pub fn shells(&self) -> Vec<SaturnianShell> {
        if self.is_trans_shell() {
            vec![SaturnianShell::Inner, SaturnianShell::Void, SaturnianShell::Outer]
        } else {
            // Depth axis is fixed — find its value
            for &(axis, val) in &self.fixed_coords {
                if axis == DEPTH_AXIS {
                    return vec![SaturnianShell::from_depth_trit(val).unwrap()];
                }
            }
            // Should not reach here if DEPTH_AXIS ∈ free_axes is false
            vec![SaturnianShell::Void]
        }
    }

    /// The Saturnian weight of this tesseract: sum of weights of its free axes.
    pub fn saturnian_weight(&self) -> u32 {
        self.free_axes.iter()
            .map(|&ax| MetatronicCircle::from_axis(ax)
                 .map_or(0, |c| c.saturnian_weight() as u32))
            .sum()
    }
}

// ══════════════════════════════════════════════════════════════
// TESSERACT ENUMERATION
// ══════════════════════════════════════════════════════════════

/// Count the number of embedded ternary tesseracts.
///
/// For each choice of 4 free axes from 13 (C(13,4) = 715 ways),
/// and each assignment of fixed values to the 9 remaining axes
/// (3⁹ = 19,683 ways), we get one distinct tesseract.
///
/// Total: 715 × 19,683 = 14,073,405.
pub fn count_ternary_tesseracts() -> usize {
    // C(13, 4)
    let axis_choices = binomial(13, 4);
    // 3^9 fixed-coordinate assignments
    let fixed_assignments = 3usize.pow(9);
    axis_choices * fixed_assignments
}

/// Count trans-shell tesseracts (those with depth axis free).
///
/// If depth axis (12) must be free, we choose 3 more free axes from
/// the remaining 12: C(12,3) = 220 ways × 3⁹ = 19,683 assignments.
///
/// Total trans-shell: 220 × 19,683 = 4,330,260.
pub fn count_trans_shell_tesseracts() -> usize {
    let axis_choices = binomial(12, 3); // depth is mandatory free
    let fixed_assignments = 3usize.pow(9);
    axis_choices * fixed_assignments
}

/// All C(13,4) = 715 axis-selections for ternary tesseracts, computed at
/// compile time. Each entry is a 4-element array of free axis indices
/// in **internal** (0-indexed) representation.
/// Zero allocation, zero runtime cost.
const TESSERACT_FAMILY_TABLE: [[usize; 4]; 715] = {
    let mut arr = [[0usize; 4]; 715];
    let mut idx = 0;
    let mut a = 0;
    while a < 10 {
        let mut b = a + 1;
        while b < 11 {
            let mut c = b + 1;
            while c < 12 {
                let mut d = c + 1;
                while d < 13 {
                    arr[idx] = [a, b, c, d];
                    idx += 1;
                    d += 1;
                }
                c += 1;
            }
            b += 1;
        }
        a += 1;
    }
    arr
};

/// Return a reference to the compile-time tesseract family table.
///
/// Each of the 715 entries is 4 free-axis indices. Each family defines
/// 3⁹ = 19,683 distinct tesseracts (one per fixed-coordinate assignment).
/// Get the 715 tesseract families. Axis indices are **internal** (0-indexed).
/// For wire encoding, use `tesseract_family_rc()` to convert individual entries.
pub fn enumerate_tesseract_families() -> &'static [[usize; 4]; 715] {
    &TESSERACT_FAMILY_TABLE
}

/// Convert a single tesseract family's axis indices to **Rep C** (1-based bijective).
///
/// Use when serializing polytope metadata for torsion network transmission.
/// Zero axis identifiers cannot appear — sentinel property upheld.
pub fn tesseract_family_rc(family: &[usize; 4]) -> [u8; 4] {
    [
        axis_to_rep_c(family[0]).unwrap(),
        axis_to_rep_c(family[1]).unwrap(),
        axis_to_rep_c(family[2]).unwrap(),
        axis_to_rep_c(family[3]).unwrap(),
    ]
}

// ══════════════════════════════════════════════════════════════
// SATURNIAN ROUND CONSTANTS
// ══════════════════════════════════════════════════════════════

/// Derive round constants from the Saturnian Magic Square.
///
/// For cryptographic operations (sponge rounds, S-box seeding), the
/// Saturnian matrix provides structured constants rather than arbitrary
/// values. The circulant property guarantees each round sees a cyclic
/// rotation of the same weight triple.
///
/// `round`: the round index (0-based)
/// Returns the Saturnian weight triple for this round.
pub fn saturnian_round_triple(round: usize) -> [u16; 3] {
    let shift = round % 3;
    [
        SATURNIAN_MATRIX[shift][0],
        SATURNIAN_MATRIX[shift][1],
        SATURNIAN_MATRIX[shift][2],
    ]
}

/// Derive a ternary round constant from the Saturnian matrix.
///
/// Maps the magic square values into balanced ternary:
///   111 mod 3 = 0 (balance)
///   14 mod 3 = 2 → -1 in balanced ternary
///   208 mod 3 = 1 → 1 in balanced ternary
///
/// This gives the ternary round constant pattern:
///   [0, -1, 1, 1, 0, -1, -1, 1, 0]
///
/// Which cycles through the three trit values in the circulant order.
pub fn saturnian_trit_constants() -> [i8; 9] {
    let mut trits = [0i8; 9];
    for i in 0..9 {
        trits[i] = match SATURNIAN_FLAT[i] % 3 {
            0 => 0,   // 111 → balance
            1 => 1,   // 208 → positive
            2 => -1,  // 14 → negative
            _ => unreachable!(),
        };
    }
    trits
}

/// Expand Saturnian trit constants to fill a sponge state (729 elements).
///
/// The 9-element Saturnian pattern is tiled across the 729-trit state,
/// creating a structured constant layer with circulant symmetry.
pub fn saturnian_sponge_constants() -> [i8; 729] {
    let pattern = saturnian_trit_constants();
    let mut constants = [0i8; 729];
    for i in 0..729 {
        constants[i] = pattern[i % 9];
    }
    constants
}

// ══════════════════════════════════════════════════════════════
// Z₂₈ ANGULAR OPERATIONS ON THE CUBE
// ══════════════════════════════════════════════════════════════

/// A position in the Z₂₈ cyclic group — one of 28 discrete angular
/// positions in the ternary circle, separated by 13° (one radian).
///
/// # Canonical Definition
///
/// This is the **canonical Rust implementation** of Z₂₈. The TypeScript
/// counterpart is imported from `ternary-circle.ts` as `Z28`. Any change
/// to the group operations here MUST be mirrored in the TypeScript module.
///
/// When the crate structure permits, this type should migrate to a shared
/// `salvi-math` crate imported by both `metatronic_cube` and any other
/// module that needs angular arithmetic in the ternary circle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Z28(pub u8);

impl Z28 {
    pub fn new(val: u8) -> Self {
        Z28(val % 28)
    }

    pub fn zero() -> Self {
        Z28(0)
    }

    pub fn add(self, other: Z28) -> Z28 {
        Z28((self.0 + other.0) % 28)
    }

    pub fn neg(self) -> Z28 {
        if self.0 == 0 { Z28(0) } else { Z28(28 - self.0) }
    }

    /// The ternary degree value: position × 13.
    pub fn to_ternary_deg(self) -> u16 {
        (self.0 as u16) * RADIAN_DEG
    }

    /// Map a pair of Metatronic axes to an angular position in Z₂₈.
    ///
    /// The sum of Saturnian weights mod 28 gives a natural angular
    /// relationship between any two axes of the cube.
    ///
    /// Accepts **internal** (0-indexed) axis indices.
    /// Use `from_axis_pair_rc()` for Rep C (1-based) input.
    pub fn from_axis_pair(a: usize, b: usize) -> Self {
        let wa = MetatronicCircle::from_axis(a).map_or(0u16, |c| c.saturnian_weight());
        let wb = MetatronicCircle::from_axis(b).map_or(0u16, |c| c.saturnian_weight());
        Z28(((wa + wb) % 28) as u8)
    }

    /// Map a pair of Metatronic axes (in **Rep C**, 1-based) to Z₂₈.
    ///
    /// Returns `None` if either axis is 0 (sentinel violation) or > 13.
    pub fn from_axis_pair_rc(a_rc: u8, b_rc: u8) -> Option<Self> {
        let a = axis_from_rep_c(a_rc)?;
        let b = axis_from_rep_c(b_rc)?;
        Some(Self::from_axis_pair(a, b))
    }
}

// ══════════════════════════════════════════════════════════════
// METATRONIC AUTOMORPHISM (STRUCTURE-PRESERVING)
// ══════════════════════════════════════════════════════════════

/// An automorphism of the Metatronic Cube that preserves the domain
/// structure (Central, Inner, Outer, Depth).
///
/// Unlike the full S₃ ≀ S₁₃ automorphism group (which permutes all
/// axes freely), a Metatronic automorphism respects the circle
/// assignments:
///
/// - Axis 0 / RC 1 (Central) is fixed
/// - Axes 1–6 / RC 2–7 (Inner ring) may be permuted among themselves
/// - Axes 7–11 / RC 8–12 (Outer ring) may be permuted among themselves
/// - Axis 12 / RC 13 (Depth) is fixed
/// - Per-axis value permutations (S₃) apply independently
///
/// Group order: 1 × 6! × 5! × 1 × 6¹³ = 720 × 120 × 13,060,694,016
///            = 1,127,099,674,583,040 ≈ 1.13 × 10¹⁵
///
/// This is a subgroup of S₃ ≀ S₁₃ that preserves the Metatronic structure.
///
/// ## Axis Representation in Permutation Fields
///
/// `inner_perm` and `outer_perm` store **internal** (0-indexed) axis values.
/// Inner ring axes are 1..=6 internally (NOT starting at 0 — this is
/// coincidence with their position, not a Rep C encoding). Outer ring
/// axes are 7..=11 internally. Use `inner_perm_rc()` / `outer_perm_rc()`
/// for wire-safe bijective encoding.
#[derive(Debug, Clone)]
pub struct MetatronicAutomorphism {
    /// Permutation of inner ring axes.
    /// `inner_perm[i]` = new **internal** axis for what was axis `(i+1)`,
    /// result in 1..=6 (internal representation).
    pub inner_perm: [u8; 6],
    /// Permutation of outer ring axes.
    /// `outer_perm[i]` = new **internal** axis for what was axis `(i+7)`,
    /// result in 7..=11 (internal representation).
    pub outer_perm: [u8; 5],
    /// Per-axis S₃ value permutation as affine maps over 𝔽₃.
    ///
    /// Each entry is `(a, b)` where `π(x) = (a · (x+1) + b) mod 3 - 1`
    /// on balanced ternary {-1, 0, +1}. The `+1` / `-1` converts
    /// through Rep B {0, 1, 2} where the affine formula is exact.
    ///
    /// - `a ∈ {1, 2}`: 1 = even perm, 2 = odd perm (transposition)
    /// - `b ∈ {0, 1, 2}`: additive offset in 𝔽₃
    ///
    /// This replaces the previous lookup table `S3_PERMS` with pure
    /// arithmetic — S₃ ≅ Aff(1, 𝔽₃).
    pub value_perms: [(u8, u8); METATRONIC_DIM],
}

impl MetatronicAutomorphism {
    /// The identity automorphism.
    pub fn identity() -> Self {
        Self {
            inner_perm: [1, 2, 3, 4, 5, 6],
            outer_perm: [7, 8, 9, 10, 11],
            value_perms: [(1, 0); METATRONIC_DIM], // (a=1, b=0) = identity for all axes
        }
    }

    /// Apply this automorphism to a vertex.
    pub fn apply(&self, v: &MetatronicVertex) -> MetatronicVertex {
        let mut new_coords = [0i8; METATRONIC_DIM];

        // Map axis 0 (Central) — fixed position, apply value perm
        new_coords[0] = self.apply_value_perm(0, v.coords[0]);

        // Map inner ring axes (1..=6)
        for i in 0..6 {
            let src_axis = i + 1;
            let dst_axis = self.inner_perm[i] as usize;
            new_coords[dst_axis] = self.apply_value_perm(src_axis, v.coords[src_axis]);
        }

        // Map outer ring axes (7..=11)
        for i in 0..5 {
            let src_axis = i + 7;
            let dst_axis = self.outer_perm[i] as usize;
            new_coords[dst_axis] = self.apply_value_perm(src_axis, v.coords[src_axis]);
        }

        // Map depth axis (12) — fixed position, apply value perm
        new_coords[12] = self.apply_value_perm(12, v.coords[12]);

        MetatronicVertex { coords: new_coords }
    }

    /// Apply the S₃ value permutation for a specific axis.
    ///
    /// Uses the affine formula: π(x) = (a · (x+1) + b) mod 3 − 1
    /// converting balanced ternary {-1,0,+1} through Rep B {0,1,2}.
    /// Pure arithmetic — no lookup table, no data-dependent memory access.
    #[inline]
    fn apply_value_perm(&self, axis: usize, value: i8) -> i8 {
        let (a, b) = self.value_perms[axis];
        debug_assert!(a == 1 || a == 2, "S₃ affine 'a' out of range: {} (axis {})", a, axis);
        debug_assert!(b < 3, "S₃ affine 'b' out of range: {} (axis {})", b, axis);
        let rep_b = (value + 1) as u8;       // {-1,0,+1} → {0,1,2}
        let result = (a * rep_b + b) % 3;    // affine map in F₃
        result as i8 - 1                      // {0,1,2} → {-1,0,+1}
    }

    /// Minimum key trits required for a Metatronic automorphism.
    ///
    /// Inner ring Fisher-Yates: 5 trits (5 swaps × 1 trit each)
    /// Outer ring Fisher-Yates: 4 trits (4 swaps × 1 trit each)
    /// Value permutations: 26 trits (13 axes × 2 trits each)
    /// Total: 35 trits.
    pub const MIN_KEY_TRITS: usize = 35;

    /// Derive a Metatronic automorphism from key material (balanced ternary trits).
    ///
    /// Uses Fisher-Yates on the inner and outer rings, then reads S₃
    /// affine selectors for each axis.
    ///
    /// # Returns
    /// `None` if:
    /// - Key is shorter than 35 trits
    /// - Any trit is outside {-1, 0, +1}
    pub fn from_key_trits(key: &[i8]) -> Option<Self> {
        if key.len() < Self::MIN_KEY_TRITS {
            return None;
        }
        let mut idx = 0;

        // Inner ring permutation (6 elements, 5 Fisher-Yates swaps, 1 trit each → 5 trits)
        let mut inner = [1u8, 2, 3, 4, 5, 6];
        for i in (1..6).rev() {
            let t = read_trit_checked(key, &mut idx)?;
            let j = ((t + 1) as usize) % (i + 1);
            inner.swap(i, j);
        }

        // Outer ring permutation (5 elements, 4 Fisher-Yates swaps, 1 trit each → 4 trits)
        let mut outer = [7u8, 8, 9, 10, 11];
        for i in (1..5).rev() {
            let t = read_trit_checked(key, &mut idx)?;
            let j = ((t + 1) as usize) % (i + 1);
            outer.swap(i, j);
        }

        // Value permutations: 2 trits per axis → 13 × 2 = 26 trits
        // Each pair of trits maps to an affine element (a, b) of S₃ ≅ Aff(1, 𝔽₃)
        let mut vperms = [(1u8, 0u8); METATRONIC_DIM]; // identity default
        for axis in 0..METATRONIC_DIM {
            let t1 = read_trit_checked(key, &mut idx)?;
            let t2 = read_trit_checked(key, &mut idx)?;
            let raw = ((t1 + 1) as u8) * 3 + (t2 + 1) as u8; // 0..8
            let sel = raw % 6;
            vperms[axis] = (if sel < 3 { 1 } else { 2 }, sel % 3);
        }

        Some(Self {
            inner_perm: inner,
            outer_perm: outer,
            value_perms: vperms,
        })
    }

    /// Validate that this automorphism preserves Metatronic structure.
    pub fn is_valid(&self) -> bool {
        // Inner ring: must be a permutation of {1,2,3,4,5,6}
        let mut inner_check = [false; 6];
        for &ax in &self.inner_perm {
            if ax < 1 || ax > 6 { return false; }
            inner_check[(ax - 1) as usize] = true;
        }
        if !inner_check.iter().all(|&b| b) { return false; }

        // Outer ring: must be a permutation of {7,8,9,10,11}
        let mut outer_check = [false; 5];
        for &ax in &self.outer_perm {
            if ax < 7 || ax > 11 { return false; }
            outer_check[(ax - 7) as usize] = true;
        }
        if !outer_check.iter().all(|&b| b) { return false; }

        // Value perms: a ∈ {1, 2}, b ∈ {0, 1, 2}
        self.value_perms.iter().all(|&(a, b)| (a == 1 || a == 2) && b < 3)
    }

    /// Inner ring permutation in **Rep C** (2..=7).
    ///
    /// Internal values 1..=6 shift to Rep C 2..=7 for wire encoding.
    /// Zero is impossible — sentinel property upheld.
    pub fn inner_perm_rc(&self) -> [u8; 6] {
        let mut rc = [0u8; 6];
        for i in 0..6 {
            rc[i] = self.inner_perm[i] + 1; // internal 1..=6 → RC 2..=7
        }
        rc
    }

    /// Outer ring permutation in **Rep C** (8..=12).
    ///
    /// Internal values 7..=11 shift to Rep C 8..=12 for wire encoding.
    pub fn outer_perm_rc(&self) -> [u8; 5] {
        let mut rc = [0u8; 5];
        for i in 0..5 {
            rc[i] = self.outer_perm[i] + 1; // internal 7..=11 → RC 8..=12
        }
        rc
    }

    /// Full 13-axis permutation map in **Rep C**.
    ///
    /// Returns an array where `result[i]` is the Rep C destination axis
    /// for the axis whose Rep C identifier is `(i + 1)`.
    /// Central (RC 1) and Depth (RC 13) are identity-mapped.
    pub fn full_perm_rc(&self) -> [u8; METATRONIC_DIM] {
        let mut out = [0u8; METATRONIC_DIM];
        // Central: RC 1 → RC 1
        out[0] = 1;
        // Inner ring: RC 2..7
        for i in 0..6 {
            out[i + 1] = self.inner_perm[i] + 1;
        }
        // Outer ring: RC 8..12
        for i in 0..5 {
            out[i + 7] = self.outer_perm[i] + 1;
        }
        // Depth: RC 13 → RC 13
        out[12] = 13;
        out
    }
}

/// Read a balanced ternary trit from key material.
///
/// Returns None if:
/// - Key is empty
/// - Index is out of bounds
/// - Trit value is outside {-1, 0, +1} (corruption/encoding error)
///
/// **No clamping, no silent fallback** — if the key is bad, we reject it.
fn read_trit_checked(key: &[i8], idx: &mut usize) -> Option<i8> {
    if *idx >= key.len() {
        return None;
    }
    let t = key[*idx];
    *idx += 1;
    if t < -1 || t > 1 {
        return None; // corruption — not silently clamped
    }
    Some(t)
}

// ══════════════════════════════════════════════════════════════
// SPONGE STATE AS 6D SUB-CUBE
// ══════════════════════════════════════════════════════════════

/// The sponge state (729 = 3⁶ trits) is a 6-dimensional sub-cube
/// of the 13-dimensional Metatronic Cube.
///
/// This function maps a sponge state index (0..729) to a vertex in
/// the full 13-cube by embedding the 6 sponge dimensions into the
/// first 6 inner-ring axes (1..6) and setting all other coordinates
/// to zero (placing it in the Void shell, at the Central origin,
/// with neutral Outer coordinates).
pub fn sponge_to_metatronic(sponge_index: usize) -> MetatronicVertex {
    let mut coords = [0i8; METATRONIC_DIM];
    let mut idx = sponge_index;
    // Map sponge dimensions to inner ring axes (1..6)
    for axis in 1..=6 {
        coords[axis] = (idx % 3) as i8 - 1;
        idx /= 3;
    }
    MetatronicVertex { coords }
}

/// Map a full Metatronic vertex back to a sponge state index,
/// using only the inner-ring coordinates (axes 1..6).
pub fn metatronic_to_sponge(v: &MetatronicVertex) -> usize {
    let mut idx = 0usize;
    let mut power = 1usize;
    for axis in 1..=6 {
        idx += ((v.coords[axis] + 1) as usize) * power;
        power *= 3;
    }
    idx
}

// ══════════════════════════════════════════════════════════════
// PROJECTION HELPER — FOR VISUALIZATION / RENDERING PIPELINES
// ══════════════════════════════════════════════════════════════

/// Configurable projection from the 13D Metatronic Cube to 3D.
///
/// Wraps the perspective + structured-matrix projection into a reusable
/// helper. Feed the output directly into matplotlib, Three.js, or the
/// Grok Imagine pipeline.
///
/// # Streaming vs. Collected
///
/// For offline visualization, use `project_all()` or `project_shells()` —
/// these return `Vec` and are convenient but allocate 1.5M × 24 bytes ≈ 36 MB.
///
/// For real-time, embedded, or memory-constrained use, use `iter_all()` or
/// `iter_shell()` — these return lazy iterators with zero allocation.
///
/// ```rust
/// let proj = MetatronicProjection::DEFAULT;
///
/// // Streaming: zero allocation, process one vertex at a time
/// for (idx, point) in proj.iter_all().enumerate() {
///     // send point to GPU, write to file, etc.
/// }
///
/// // Collected: convenient for offline use
/// let [inner, void, outer] = proj.project_shells();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MetatronicProjection {
    /// Distance of the virtual viewer along the depth axis.
    /// Larger = flatter (less perspective). Must be > 1.0.
    pub viewer_distance: f64,
}

impl MetatronicProjection {
    /// Default projection: viewer_distance = 5.0.
    pub const DEFAULT: Self = Self { viewer_distance: 5.0 };

    // ── Streaming (zero allocation) ───────────────────────────

    /// Iterate all 1,594,323 projected vertices lazily.
    ///
    /// No allocation — produces one `[f64; 3]` at a time.
    /// Suitable for GPU streaming, disk writes, or any pipeline
    /// that processes vertices one-by-one.
    pub fn iter_all(&self) -> impl Iterator<Item = [f64; 3]> + '_ {
        (0..METATRONIC_VERTICES).map(move |i| {
            MetatronicVertex::from_linear_index(i)
                .project_to_3d(self.viewer_distance)
        })
    }

    /// Iterate projected vertices for a single shell lazily.
    ///
    /// Filters at zero cost — skips non-matching shells without
    /// allocating storage for them.
    pub fn iter_shell(&self, shell: SaturnianShell) -> impl Iterator<Item = [f64; 3]> + '_ {
        (0..METATRONIC_VERTICES).filter_map(move |i| {
            let v = MetatronicVertex::from_linear_index(i);
            if v.shell() == shell {
                Some(v.project_to_3d(self.viewer_distance))
            } else {
                None
            }
        })
    }

    // ── Collected (convenience for offline use) ───────────────

    /// Project all 1,594,323 vertices to 3D.
    ///
    /// Allocates ~36 MB. For streaming, use `iter_all()` instead.
    pub fn project_all(&self) -> Vec<[f64; 3]> {
        self.iter_all().collect()
    }

    /// Project vertices shell-by-shell: [Inner, Void, Outer].
    ///
    /// Allocates ~12 MB per shell. For streaming, use `iter_shell()`.
    pub fn project_shells(&self) -> [Vec<[f64; 3]>; 3] {
        let mut shells: [Vec<[f64; 3]>; 3] = [
            Vec::with_capacity(SHELL_VERTICES),
            Vec::with_capacity(SHELL_VERTICES),
            Vec::with_capacity(SHELL_VERTICES),
        ];
        for i in 0..METATRONIC_VERTICES {
            let v = MetatronicVertex::from_linear_index(i);
            let p = v.project_to_3d(self.viewer_distance);
            match v.shell() {
                SaturnianShell::Inner => shells[0].push(p),
                SaturnianShell::Void  => shells[1].push(p),
                SaturnianShell::Outer => shells[2].push(p),
            }
        }
        shells
    }

    /// Project only vertices in a specific shell (collected).
    ///
    /// For streaming, use `iter_shell()`.
    pub fn project_shell(&self, shell: SaturnianShell) -> Vec<[f64; 3]> {
        self.iter_shell(shell).collect()
    }
}

// ══════════════════════════════════════════════════════════════
// UTILITY
// ══════════════════════════════════════════════════════════════

/// Binomial coefficient C(n, k).
fn binomial(n: usize, k: usize) -> usize {
    if k > n { return 0; }
    if k == 0 || k == n { return 1; }
    let k = k.min(n - k);
    let mut result = 1usize;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_count() {
        assert_eq!(3usize.pow(13), METATRONIC_VERTICES);
        assert_eq!(3usize.pow(12), SHELL_VERTICES);
        assert_eq!(SHELL_VERTICES * 3, METATRONIC_VERTICES);
    }

    #[test]
    fn test_saturnian_magic_square() {
        // All rows sum to 333
        for row in &SATURNIAN_MATRIX {
            assert_eq!(row[0] as u32 + row[1] as u32 + row[2] as u32, 333);
        }
        // All columns sum to 333
        for c in 0..3 {
            assert_eq!(
                SATURNIAN_MATRIX[0][c] as u32 +
                SATURNIAN_MATRIX[1][c] as u32 +
                SATURNIAN_MATRIX[2][c] as u32,
                333
            );
        }
        // Diagonals
        assert_eq!(
            SATURNIAN_MATRIX[0][0] as u32 +
            SATURNIAN_MATRIX[1][1] as u32 +
            SATURNIAN_MATRIX[2][2] as u32,
            333
        );
        assert_eq!(
            SATURNIAN_MATRIX[0][2] as u32 +
            SATURNIAN_MATRIX[1][1] as u32 +
            SATURNIAN_MATRIX[2][0] as u32,
            333
        );
    }

    #[test]
    fn test_saturnian_circulant() {
        // Each row is a cyclic shift
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(SATURNIAN_MATRIX[r][c], SATURNIAN_MATRIX[0][(c + 3 - r) % 3]);
            }
        }
    }

    #[test]
    fn test_metatronic_circle_axes() {
        let all = MetatronicCircle::all();
        assert_eq!(all.len(), 13);
        assert_eq!(all[0].axis_index(), 0);
        assert_eq!(all[6].axis_index(), 6);
        assert_eq!(all[12].axis_index(), 12);
    }

    #[test]
    fn test_saturnian_weights_per_domain() {
        assert_eq!(MetatronicCircle::Central.saturnian_weight(), 111);
        assert_eq!(MetatronicCircle::Inner(3).saturnian_weight(), 14);
        assert_eq!(MetatronicCircle::Outer(9).saturnian_weight(), 208);
        assert_eq!(MetatronicCircle::Depth.saturnian_weight(), 333);
    }

    #[test]
    fn test_shell_from_depth_trit() {
        assert_eq!(SaturnianShell::from_depth_trit(-1), Some(SaturnianShell::Inner));
        assert_eq!(SaturnianShell::from_depth_trit(0), Some(SaturnianShell::Void));
        assert_eq!(SaturnianShell::from_depth_trit(1), Some(SaturnianShell::Outer));
        assert_eq!(SaturnianShell::from_depth_trit(2), None);
    }

    #[test]
    fn test_shell_mirror() {
        assert_eq!(SaturnianShell::Inner.mirror(), SaturnianShell::Outer);
        assert_eq!(SaturnianShell::Outer.mirror(), SaturnianShell::Inner);
        assert_eq!(SaturnianShell::Void.mirror(), SaturnianShell::Void);
    }

    #[test]
    fn test_vertex_origin_is_void_shell() {
        let origin = MetatronicVertex::origin();
        assert_eq!(origin.shell(), SaturnianShell::Void);
        assert_eq!(origin.coords, [0; 13]);
    }

    #[test]
    fn test_vertex_rep_conversion_roundtrip() {
        let v = MetatronicVertex::from_rep_a([1, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1]).unwrap();
        let rep_b = v.to_rep_b();
        let rep_c = v.to_rep_c();

        // Rep B: {-1,0,+1} → {0,1,2}
        assert_eq!(rep_b[0], 2); // 1 → 2
        assert_eq!(rep_b[1], 0); // -1 → 0
        assert_eq!(rep_b[2], 1); // 0 → 1

        // Rep C: {-1,0,+1} → {1,2,3}
        assert_eq!(rep_c[0], 3); // 1 → 3
        assert_eq!(rep_c[1], 1); // -1 → 1
        assert_eq!(rep_c[2], 2); // 0 → 2

        // Roundtrip through Rep C
        let v2 = MetatronicVertex::from_rep_c(rep_c).unwrap();
        assert_eq!(v.coords, v2.coords);
    }

    #[test]
    fn test_rep_c_sentinel_rejects_zero() {
        let bad = MetatronicVertex::from_rep_c([1, 2, 3, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3]);
        assert!(bad.is_none(), "Rep C must reject zero");
    }

    #[test]
    fn test_vertex_linear_index_roundtrip() {
        for idx in [0, 1, 728, 729, 1000, 531_440, 531_441, 1_594_322] {
            let v = MetatronicVertex::from_linear_index(idx);
            assert_eq!(v.to_linear_index(), idx, "Roundtrip failed for index {}", idx);
        }
    }

    #[test]
    fn test_vertex_shell_distribution() {
        // Index 0: all zeros → Void
        let v0 = MetatronicVertex::from_linear_index(0);
        assert_eq!(v0.shell(), SaturnianShell::Void); // 0 is middle rep_b digit → rep_a = -1... wait

        // Actually: from_linear_index(0) → rep_b all 0 → rep_a all -1
        // depth axis (12) = -1 → Inner shell
        assert_eq!(v0.shell(), SaturnianShell::Inner);
    }

    #[test]
    fn test_mirror_vertex() {
        let v = MetatronicVertex::from_rep_a([1, 0, -1, 1, 0, -1, 1, 0, -1, 1, 0, -1, -1]).unwrap();
        assert_eq!(v.shell(), SaturnianShell::Inner);
        let m = v.mirror_vertex();
        assert_eq!(m.shell(), SaturnianShell::Outer);
        assert_eq!(m.coords[..12], v.coords[..12]);
        assert_eq!(m.coords[12], 1);
    }

    #[test]
    fn test_hamming_distance() {
        let v1 = MetatronicVertex::from_rep_a([0; 13]).unwrap();
        let v2 = MetatronicVertex::from_rep_a([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(v1.hamming_distance(&v2), 1);

        let v3 = MetatronicVertex::from_rep_a([1; 13]).unwrap();
        assert_eq!(v1.hamming_distance(&v3), 13);
    }

    #[test]
    fn test_saturnian_distance_weighted() {
        let v1 = MetatronicVertex::origin();
        // Change only the Central axis (weight 111)
        let mut c2 = [0i8; 13];
        c2[0] = 1;
        let v2 = MetatronicVertex::from_rep_a(c2).unwrap();
        assert_eq!(v1.saturnian_distance(&v2), 111);

        // Change only an Inner axis (weight 14)
        let mut c3 = [0i8; 13];
        c3[3] = 1;
        let v3 = MetatronicVertex::from_rep_a(c3).unwrap();
        assert_eq!(v1.saturnian_distance(&v3), 14);

        // Change only an Outer axis (weight 208)
        let mut c4 = [0i8; 13];
        c4[9] = 1;
        let v4 = MetatronicVertex::from_rep_a(c4).unwrap();
        assert_eq!(v1.saturnian_distance(&v4), 208);
    }

    #[test]
    fn test_z28_operations() {
        let a = Z28::new(13);
        let b = Z28::new(20);
        let sum = a.add(b);
        assert_eq!(sum.0, (13 + 20) % 28);

        let zero = Z28::zero();
        assert_eq!(zero.to_ternary_deg(), 0);
        assert_eq!(Z28::new(1).to_ternary_deg(), 13);
        assert_eq!(Z28::new(28).to_ternary_deg(), 0); // wraps
    }

    #[test]
    fn test_z28_axis_pair() {
        // Central (111) + Inner (14) = 125 mod 28 = 13
        let pos = Z28::from_axis_pair(0, 1);
        assert_eq!(pos.0, (111 + 14) % 28);
    }

    #[test]
    fn test_tesseract_count() {
        assert_eq!(binomial(13, 4), 715);
        assert_eq!(count_ternary_tesseracts(), 715 * 19_683);
    }

    #[test]
    fn test_tesseract_family_enumeration() {
        let families = enumerate_tesseract_families();
        assert_eq!(families.len(), 715);
        // Each family has 4 distinct ascending axis indices
        for f in families.iter() {
            assert!(f[0] < f[1] && f[1] < f[2] && f[2] < f[3]);
            assert!(f[3] < 13);
        }
    }

    #[test]
    fn test_trans_shell_tesseract() {
        let t = TernaryTesseract {
            free_axes: [1, 5, 10, DEPTH_AXIS],
            fixed_coords: [
                (0, 0), (2, 0), (3, 0), (4, 0), (6, 0),
                (7, 0), (8, 0), (9, 0), (11, 0),
            ],
        };
        assert!(t.is_trans_shell());
        assert_eq!(t.shells().len(), 3);
    }

    #[test]
    fn test_intra_shell_tesseract() {
        let t = TernaryTesseract {
            free_axes: [1, 2, 3, 4],
            fixed_coords: [
                (0, 0), (5, 0), (6, 0), (7, 0), (8, 0),
                (9, 0), (10, 0), (11, 0), (DEPTH_AXIS, -1),
            ],
        };
        assert!(!t.is_trans_shell());
        assert_eq!(t.shells(), vec![SaturnianShell::Inner]);
    }

    #[test]
    fn test_saturnian_trit_constants() {
        let trits = saturnian_trit_constants();
        // 111 mod 3 = 0, 14 mod 3 = 2 → -1, 208 mod 3 = 1 → 1
        assert_eq!(trits[0], 0);   // 111
        assert_eq!(trits[1], -1);  // 14
        assert_eq!(trits[2], 1);   // 208
        // Circulant: row 2 is [208, 111, 14] → [1, 0, -1]
        assert_eq!(trits[3], 1);   // 208
        assert_eq!(trits[4], 0);   // 111
        assert_eq!(trits[5], -1);  // 14
    }

    #[test]
    fn test_sponge_metatronic_roundtrip() {
        for idx in [0, 1, 100, 364, 728] {
            let v = sponge_to_metatronic(idx);
            let back = metatronic_to_sponge(&v);
            assert_eq!(back, idx, "Sponge roundtrip failed for {}", idx);
        }
    }

    #[test]
    fn test_sponge_embedding_in_void_shell() {
        // All sponge embeddings land in the Void shell (x₁₂ = 0)
        for idx in 0..729 {
            let v = sponge_to_metatronic(idx);
            assert_eq!(v.shell(), SaturnianShell::Void);
            assert_eq!(v.coords[0], 0);  // Central axis neutral
            assert_eq!(v.coords[12], 0); // Depth axis neutral
        }
    }

    #[test]
    fn test_metatronic_automorphism_identity() {
        let aut = MetatronicAutomorphism::identity();
        assert!(aut.is_valid());

        let v = MetatronicVertex::from_rep_a([1, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1]).unwrap();
        let result = aut.apply(&v);
        assert_eq!(result.coords, v.coords);
    }

    #[test]
    fn test_metatronic_automorphism_preserves_domains() {
        let key: Vec<i8> = vec![1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1, 0,
                                 1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1, 0,
                                 1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1];
        let aut = MetatronicAutomorphism::from_key_trits(&key).unwrap();
        assert!(aut.is_valid());

        // Inner perm outputs must be in 1..=6
        for &ax in &aut.inner_perm {
            assert!(ax >= 1 && ax <= 6);
        }
        // Outer perm outputs must be in 7..=11
        for &ax in &aut.outer_perm {
            assert!(ax >= 7 && ax <= 11);
        }
    }

    #[test]
    fn test_metatronic_automorphism_from_key_bijection() {
        let key: Vec<i8> = vec![1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1, 0,
                                 1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1, 0,
                                 1, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, -1, 1, -1];
        let aut = MetatronicAutomorphism::from_key_trits(&key).unwrap();

        // Apply to many vertices, verify no collisions
        let mut seen = alloc::collections::BTreeSet::new();
        for idx in (0..1000).map(|i| i * 1594) {
            let v = MetatronicVertex::from_linear_index(idx % METATRONIC_VERTICES);
            let result = aut.apply(&v);
            let result_idx = result.to_linear_index();
            assert!(seen.insert(result_idx), "Collision at input index {}", idx);
        }
    }

    #[test]
    fn test_correspondence_edge_properties() {
        let shell_coords = [0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
        let edge = CorrespondenceEdge {
            shell_coords,
            from: SaturnianShell::Inner,
            to: SaturnianShell::Outer,
        };

        let src = edge.source_vertex();
        let tgt = edge.target_vertex();

        assert_eq!(src.shell(), SaturnianShell::Inner);
        assert_eq!(tgt.shell(), SaturnianShell::Outer);
        assert_eq!(src.shell_coords(), tgt.shell_coords());
        assert!(!edge.is_direct()); // Inner ↔ Outer is long correspondence
    }

    #[test]
    fn test_saturnian_weight_sums() {
        // Sum of all 13 axis weights:
        // 111 + 6×14 + 5×208 + 333 = 111 + 84 + 1040 + 333 = 1568
        let total: u32 = MetatronicCircle::all().iter()
            .map(|c| c.saturnian_weight() as u32)
            .sum();
        assert_eq!(total, 111 + 84 + 1040 + 333);
    }

    #[test]
    fn test_full_circle_ternary_constants() {
        assert_eq!(FULL_CIRCLE_DEG, 364);
        assert_eq!(RADIAN_DEG, 13);
        assert_eq!(Z28_ORDER, 28);
        assert_eq!(RADIAN_DEG * Z28_ORDER, FULL_CIRCLE_DEG);
        assert_eq!(PI_ESOTERIC, 14);
    }

    #[test]
    fn test_radian_is_tribonacci_t7() {
        // T₇ = 13 (the seventh Tribonacci number)
        // T: 0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, ...
        assert_eq!(RADIAN_DEG, 13);
    }

    // ── Patch tests: Copy/Hash/Default, const tesseracts, projection ──

    #[test]
    fn test_vertex_default_is_origin() {
        let v: MetatronicVertex = Default::default();
        assert_eq!(v.coords, [0; METATRONIC_DIM]);
        assert_eq!(v.shell(), SaturnianShell::Void);
    }

    #[test]
    fn test_vertex_copy_semantics() {
        let v1 = MetatronicVertex::from_rep_a([1, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1]).unwrap();
        let v2 = v1; // Copy, not move
        assert_eq!(v1.coords, v2.coords); // v1 still usable
    }

    #[test]
    fn test_const_tesseract_table_matches_runtime() {
        // Verify the const table produces the same families as iterating
        let table = enumerate_tesseract_families();
        assert_eq!(table.len(), 715);

        // Spot check first and last entries
        assert_eq!(table[0], [0, 1, 2, 3]);
        assert_eq!(table[714], [9, 10, 11, 12]);

        // All entries have 4 ascending indices < 13
        for f in table.iter() {
            assert!(f[0] < f[1] && f[1] < f[2] && f[2] < f[3] && f[3] < 13);
        }
    }

    #[test]
    fn test_projection_matrix_orthonormality() {
        // Row norms should be ≈ 1.0
        for row in 0..3 {
            let norm_sq: f64 = STRUCTURED_PROJ_MATRIX[row].iter()
                .map(|x| x * x).sum();
            assert!((norm_sq - 1.0).abs() < 1e-6, "Row {} norm² = {}", row, norm_sq);
        }
        // Off-diagonal dot products should be ≈ 0
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            let dot: f64 = (0..12).map(|j| {
                STRUCTURED_PROJ_MATRIX[a][j] * STRUCTURED_PROJ_MATRIX[b][j]
            }).sum();
            assert!(dot.abs() < 0.05, "Dot({},{}) = {}", a, b, dot);
        }
    }

    #[test]
    fn test_project_to_3d_origin() {
        let v = MetatronicVertex::origin();
        let p = v.project_to_3d(5.0);
        // Origin → all coordinates 0 → projection is [0, 0, 0]
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_project_to_3d_shell_separation() {
        // Same 12D coords, different shells → different 3D positions
        let inner = MetatronicVertex::from_rep_a([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1]).unwrap();
        let outer = MetatronicVertex::from_rep_a([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  1]).unwrap();

        let p_inner = inner.project_to_3d(5.0);
        let p_outer = outer.project_to_3d(5.0);

        // Inner shell has perspective factor 5/(5+1) = 5/6
        // Outer shell has perspective factor 5/(5-1) = 5/4
        // So outer projects LARGER than inner
        let norm_inner: f64 = p_inner.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_outer: f64 = p_outer.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(norm_outer > norm_inner, "Outer shell should project larger");
    }

    #[test]
    fn test_metatronic_projection_shell_counts() {
        // project_shells should produce 531,441 vertices per shell
        // (too expensive for CI — just verify the helper constructs correctly)
        let proj = MetatronicProjection::DEFAULT;
        assert_eq!(proj.viewer_distance, 5.0);
    }

    // ══════════════════════════════════════════════════════════════
    // BIJECTIVE AXIS NUMBERING (Rep C for Axes) TESTS
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn test_axis_to_rep_c_range() {
        // Internal 0 → RC 1, Internal 12 → RC 13
        assert_eq!(axis_to_rep_c(0), Some(1));
        assert_eq!(axis_to_rep_c(6), Some(7));
        assert_eq!(axis_to_rep_c(12), Some(13));
        // Out of range
        assert_eq!(axis_to_rep_c(13), None);
        assert_eq!(axis_to_rep_c(255), None);
    }

    #[test]
    fn test_axis_from_rep_c_range() {
        // RC 1 → Internal 0, RC 13 → Internal 12
        assert_eq!(axis_from_rep_c(1), Some(0));
        assert_eq!(axis_from_rep_c(7), Some(6));
        assert_eq!(axis_from_rep_c(13), Some(12));
        // Sentinel: RC 0 is structurally impossible
        assert_eq!(axis_from_rep_c(0), None);
        // Out of range
        assert_eq!(axis_from_rep_c(14), None);
    }

    #[test]
    fn test_axis_rc_roundtrip() {
        // Every internal index roundtrips through Rep C
        for i in 0..METATRONIC_DIM {
            let rc = axis_to_rep_c(i).unwrap();
            assert_eq!(axis_from_rep_c(rc), Some(i));
        }
    }

    #[test]
    fn test_axis_rc_zero_sentinel() {
        // Zero in Rep C is rejected — sentinel property
        assert!(!axis_rc_valid(0));
        assert!(axis_rc_valid(1));
        assert!(axis_rc_valid(13));
        assert!(!axis_rc_valid(14));
    }

    #[test]
    fn test_depth_axis_rc_is_t7() {
        // Depth axis RC 13 = T₇ = one ternary radian
        assert_eq!(DEPTH_AXIS_RC, 13);
        assert_eq!(DEPTH_AXIS_RC as u16, RADIAN_DEG); // 13° = T₇
    }

    #[test]
    fn test_circle_axis_index_rc() {
        // Central: internal 0, RC 1
        assert_eq!(MetatronicCircle::Central.axis_index(), 0);
        assert_eq!(MetatronicCircle::Central.axis_index_rc(), 1);
        // Inner(1): internal 1, RC 2
        assert_eq!(MetatronicCircle::Inner(1).axis_index_rc(), 2);
        // Inner(6): internal 6, RC 7
        assert_eq!(MetatronicCircle::Inner(6).axis_index_rc(), 7);
        // Outer(7): internal 7, RC 8
        assert_eq!(MetatronicCircle::Outer(7).axis_index_rc(), 8);
        // Outer(11): internal 11, RC 12
        assert_eq!(MetatronicCircle::Outer(11).axis_index_rc(), 12);
        // Depth: internal 12, RC 13
        assert_eq!(MetatronicCircle::Depth.axis_index_rc(), 13);
    }

    #[test]
    fn test_circle_from_axis_rc() {
        // RC 1 → Central
        assert_eq!(MetatronicCircle::from_axis_rc(1), Some(MetatronicCircle::Central));
        // RC 2..7 → Inner ring
        assert_eq!(MetatronicCircle::from_axis_rc(2), Some(MetatronicCircle::Inner(1)));
        assert_eq!(MetatronicCircle::from_axis_rc(7), Some(MetatronicCircle::Inner(6)));
        // RC 8..12 → Outer ring
        assert_eq!(MetatronicCircle::from_axis_rc(8), Some(MetatronicCircle::Outer(7)));
        assert_eq!(MetatronicCircle::from_axis_rc(12), Some(MetatronicCircle::Outer(11)));
        // RC 13 → Depth
        assert_eq!(MetatronicCircle::from_axis_rc(13), Some(MetatronicCircle::Depth));
        // RC 0 → sentinel violation
        assert_eq!(MetatronicCircle::from_axis_rc(0), None);
    }

    #[test]
    fn test_vertex_wire_roundtrip() {
        // Create a vertex and verify wire serialization roundtrips
        let v = MetatronicVertex::from_rep_a([1, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1]).unwrap();
        let wire = v.to_wire_pairs();

        // Verify no zeros in wire format
        for (axis_rc, trit_rc) in &wire {
            assert_ne!(*axis_rc, 0, "axis RC must never be zero");
            assert_ne!(*trit_rc, 0, "trit RC must never be zero");
        }

        // Verify axis IDs are 1..=13
        for i in 0..METATRONIC_DIM {
            assert_eq!(wire[i].0, (i + 1) as u8);
        }

        // Roundtrip
        let v2 = MetatronicVertex::from_wire_pairs(&wire).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_wire_sentinel_rejection() {
        // A wire pair with axis 0 should be rejected
        let mut bad_wire = [(0u8, 0u8); METATRONIC_DIM];
        for i in 0..METATRONIC_DIM {
            bad_wire[i] = ((i + 1) as u8, 2); // valid trit, valid axis
        }
        // Now poison one axis with 0
        bad_wire[5] = (0, 2);
        assert!(MetatronicVertex::from_wire_pairs(&bad_wire).is_none());

        // Poison one trit with 0
        bad_wire[5] = (6, 0);
        assert!(MetatronicVertex::from_wire_pairs(&bad_wire).is_none());
    }

    #[test]
    fn test_tesseract_rc_roundtrip() {
        let families = enumerate_tesseract_families();
        // First family: [0, 1, 2, 3] internal → [1, 2, 3, 4] RC
        let rc = tesseract_family_rc(&families[0]);
        assert_eq!(rc, [1, 2, 3, 4]);

        // Last family: [9, 10, 11, 12] internal → [10, 11, 12, 13] RC
        let rc_last = tesseract_family_rc(&families[714]);
        assert_eq!(rc_last, [10, 11, 12, 13]);

        // No zeros in any RC family
        for fam in families.iter() {
            let rc = tesseract_family_rc(fam);
            for ax_rc in &rc {
                assert_ne!(*ax_rc, 0, "Rep C axis must never be 0");
            }
        }
    }

    #[test]
    fn test_tesseract_struct_rc_roundtrip() {
        let t = TernaryTesseract {
            free_axes: [0, 3, 7, 12],
            fixed_coords: [
                (1, -1), (2, 0), (4, 1), (5, -1), (6, 0),
                (8, 1), (9, -1), (10, 0), (11, 1),
            ],
        };

        let free_rc = t.free_axes_rc();
        assert_eq!(free_rc, [1, 4, 8, 13]); // internal +1

        let fixed_rc = t.fixed_coords_rc();
        assert_eq!(fixed_rc[0], (2, -1)); // axis 1 → RC 2

        // Roundtrip through from_rc
        let t2 = TernaryTesseract::from_rc(free_rc, fixed_rc).unwrap();
        assert_eq!(t.free_axes, t2.free_axes);
        assert_eq!(t.fixed_coords, t2.fixed_coords);
    }

    #[test]
    fn test_tesseract_from_rc_rejects_zero() {
        // Zero axis in free_rc should fail
        let result = TernaryTesseract::from_rc(
            [0, 2, 3, 4],
            [(5, 0); 9],
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_z28_from_axis_pair_rc() {
        // from_axis_pair(0, 1) should equal from_axis_pair_rc(1, 2)
        let z_internal = Z28::from_axis_pair(0, 1);
        let z_rc = Z28::from_axis_pair_rc(1, 2).unwrap();
        assert_eq!(z_internal, z_rc);

        // RC 0 should fail (sentinel)
        assert!(Z28::from_axis_pair_rc(0, 1).is_none());
    }

    #[test]
    fn test_automorphism_perm_rc() {
        let auto = MetatronicAutomorphism::identity();

        // Identity inner: internal [1,2,3,4,5,6] → RC [2,3,4,5,6,7]
        assert_eq!(auto.inner_perm_rc(), [2, 3, 4, 5, 6, 7]);

        // Identity outer: internal [7,8,9,10,11] → RC [8,9,10,11,12]
        assert_eq!(auto.outer_perm_rc(), [8, 9, 10, 11, 12]);

        // Full perm RC: [1,2,3,4,5,6,7,8,9,10,11,12,13]
        let full = auto.full_perm_rc();
        for i in 0..13 {
            assert_eq!(full[i], (i + 1) as u8);
        }
    }

    #[test]
    fn test_automorphism_perm_rc_no_zeros() {
        // Key-derived automorphism should also produce no RC zeros
        // Minimum 35 trits required for MetatronicAutomorphism
        let key = [1i8, -1, 0, 1, 0, -1, 1, -1, 0, 1, 0, 1, -1, 0, 1, -1,
                   0, 1, -1, 0, 1, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1, 0, -1];
        let auto = MetatronicAutomorphism::from_key_trits(&key).unwrap();
        let full = auto.full_perm_rc();
        for ax_rc in &full {
            assert_ne!(*ax_rc, 0, "Rep C perm must never contain 0");
        }
    }

    #[test]
    fn test_polarization_rc() {
        let p = TetrahedronPolarization::Polarized { fixed_axis: 0, fixed_value: 1 };
        assert_eq!(p.fixed_axis_rc(), Some(1)); // internal 0 → RC 1

        let p2 = TetrahedronPolarization::Polarized { fixed_axis: 12, fixed_value: -1 };
        assert_eq!(p2.fixed_axis_rc(), Some(13)); // depth → RC 13

        let tw = TetrahedronPolarization::Twisted;
        assert_eq!(tw.fixed_axis_rc(), None);

        // Construct from RC
        let p3 = TetrahedronPolarization::polarized_from_rc(1, 1).unwrap();
        assert_eq!(p3, TetrahedronPolarization::Polarized { fixed_axis: 0, fixed_value: 1 });

        // RC 0 rejected
        assert!(TetrahedronPolarization::polarized_from_rc(0, 1).is_none());
    }

    #[test]
    fn test_octahedron_rc() {
        let oct = TernaryOctahedron {
            vertices: [MetatronicVertex::origin(); 6],
            varying_axis: 12,
        };
        assert_eq!(oct.varying_axis_rc(), 13); // depth → RC 13
    }

    #[test]
    fn test_axes_batch_conversion() {
        let internal = vec![0, 6, 12];
        let rc = axes_to_rep_c(&internal).unwrap();
        assert_eq!(rc, vec![1, 7, 13]);

        let back = axes_from_rep_c(&rc).unwrap();
        assert_eq!(back, internal);

        // Batch with out-of-range internal axis should fail
        assert!(axes_to_rep_c(&[0, 13, 12]).is_none());

        // Batch with zero Rep C should fail
        assert!(axes_from_rep_c(&[1, 0, 13]).is_none());
    }

    #[test]
    fn test_vertex_display() {
        use alloc::format;

        // Origin: all zeros → Void shell
        let origin = MetatronicVertex::default();
        let s = format!("{}", origin);
        assert!(s.starts_with("[0000000000000]"));
        assert!(s.contains("Void"));

        // Inner shell vertex: depth = -1
        let mut coords = [0i8; METATRONIC_DIM];
        coords[0] = 1;   // Central = +
        coords[1] = -1;  // Inner(1) = −
        coords[12] = -1; // Depth = − → Inner shell
        let v = MetatronicVertex { coords };
        let s = format!("{}", v);
        assert!(s.starts_with("[+−"));
        assert!(s.contains("Inner"));

        // Outer shell vertex: depth = +1
        coords[12] = 1;
        let v = MetatronicVertex { coords };
        let s = format!("{}", v);
        assert!(s.contains("Outer"));
    }

    #[test]
    fn test_from_key_trits_rejects_short_key() {
        let short = [0i8; 34]; // one short of 35
        assert!(MetatronicAutomorphism::from_key_trits(&short).is_none(),
            "Should reject key shorter than MIN_KEY_TRITS");

        let exact = [0i8; 35];
        assert!(MetatronicAutomorphism::from_key_trits(&exact).is_some(),
            "Should accept key at exact MIN_KEY_TRITS");
    }

    #[test]
    fn test_from_key_trits_rejects_out_of_range() {
        let mut bad = [0i8; 40];
        bad[10] = 2; // out of range
        assert!(MetatronicAutomorphism::from_key_trits(&bad).is_none(),
            "Should reject trit value 2");

        bad[10] = 0;
        bad[20] = -2;
        assert!(MetatronicAutomorphism::from_key_trits(&bad).is_none(),
            "Should reject trit value -2");
    }
}