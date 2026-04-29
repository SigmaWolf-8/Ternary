// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Metatronic Bridge
//!
//! Connects the `MetatronicVertex` (from `metatronic_cube.rs`) to the
//! existing kernel torsion network (`network::torus`).
//!
//! ## What This Does
//!
//! The kernel's `TorusTopology` uses `TorusCoordinate` (Rep B: `Vec<u16>`,
//! values in {0, 1, 2}) with generic `TorsionCoefficient` weights and
//! `DimensionType` labels. This bridge:
//!
//! 1. Converts `MetatronicVertex` ↔ `TorusCoordinate` losslessly
//! 2. Initializes a 13D `TorusTopology` with Metatronic axis assignments:
//!    - Saturnian weights as torsion coefficients
//!    - `DimensionType` mapped from `MetatronicDomain`
//! 3. Provides shell-aware routing helpers
//!
//! ## Placement
//!
//! `src/kernel/src/network/metatronic_bridge.rs`
//!
//! Add to `src/kernel/src/network/mod.rs`:
//! ```rust
//! pub mod metatronic_bridge;
//! ```

extern crate alloc;

use alloc::vec::Vec;

// These imports reference the existing kernel modules.
// In the actual codebase, adjust paths to match `use super::` or `use crate::`.
//
// use crate::network::{NetworkResult, NodeId};
// use crate::network::torus::{TorusCoordinate, TorusTopology, TorsionCoefficient, DimensionType};
// use crate::crypto::metatronic_cube::*;

// ══════════════════════════════════════════════════════════════
// RE-EXPORTS from metatronic_cube (for standalone compilation)
// In the real codebase, remove these and import from the module.
// ══════════════════════════════════════════════════════════════

const METATRONIC_DIM: usize = 13;

const SATURNIAN_WEIGHTS: [u32; 13] = [
    111,  // axis 0 / RC 1:  Central
    14,   // axis 1 / RC 2:  Inner
    14,   // axis 2 / RC 3:  Inner
    14,   // axis 3 / RC 4:  Inner
    14,   // axis 4 / RC 5:  Inner
    14,   // axis 5 / RC 6:  Inner
    14,   // axis 6 / RC 7:  Inner
    208,  // axis 7 / RC 8:  Outer
    208,  // axis 8 / RC 9:  Outer
    208,  // axis 9 / RC 10: Outer
    208,  // axis 10 / RC 11: Outer
    208,  // axis 11 / RC 12: Outer
    333,  // axis 12 / RC 13: Depth (= T₇ = one ternary radian)
];

/// Maps MetatronicDomain → DimensionType from the existing kernel.
///
/// | Metatronic Domain | DimensionType | Rationale |
/// |-------------------|---------------|-----------|
/// | Foundation (0)    | Ternary       | Central anchor of the ternary system |
/// | Manifestation(1–6)| Phase         | Inner ring: phase/timing dimensions |
/// | Transcendence(7–11)| Angular      | Outer ring: angular/spatial dimensions |
/// | ShellBoundary (12)| Security      | Depth axis: trust domain boundary |
///
/// These assignments are semantically motivated but can be refined
/// as the domain model evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetatronicDimType {
    Foundation,     // → DimensionType::Ternary
    Manifestation,  // → DimensionType::Phase
    Transcendence,  // → DimensionType::Angular
    ShellBoundary,  // → DimensionType::Security
}

impl MetatronicDimType {
    /// Get the MetatronicDimType for a given axis index.
    /// Accepts **internal** (0-indexed) axis indices.
    /// Use `from_axis_rc()` for Rep C (1-based bijective) input.
    pub fn from_axis(axis: usize) -> Option<Self> {
        match axis {
            0 => Some(MetatronicDimType::Foundation),
            1..=6 => Some(MetatronicDimType::Manifestation),
            7..=11 => Some(MetatronicDimType::Transcendence),
            12 => Some(MetatronicDimType::ShellBoundary),
            _ => None,
        }
    }

    /// Get the MetatronicDimType for a **Rep C** (1-based bijective) axis.
    /// Returns `None` if `rc` is 0 (sentinel violation) or > 13.
    pub fn from_axis_rc(rc: u8) -> Option<Self> {
        if rc == 0 || rc > 13 { return None; }
        Self::from_axis((rc - 1) as usize)
    }

    /// Convert to the kernel's DimensionType enum.
    ///
    /// In the actual codebase, return `DimensionType` directly:
    /// ```rust
    /// pub fn to_kernel_type(&self) -> DimensionType {
    ///     match self {
    ///         MetatronicDimType::Foundation => DimensionType::Ternary,
    ///         MetatronicDimType::Manifestation => DimensionType::Phase,
    ///         MetatronicDimType::Transcendence => DimensionType::Angular,
    ///         MetatronicDimType::ShellBoundary => DimensionType::Security,
    ///     }
    /// }
    /// ```
    pub fn kernel_type_name(&self) -> &'static str {
        match self {
            MetatronicDimType::Foundation => "Ternary",
            MetatronicDimType::Manifestation => "Phase",
            MetatronicDimType::Transcendence => "Angular",
            MetatronicDimType::ShellBoundary => "Security",
        }
    }

    /// The Saturnian weight for this dimension type.
    pub fn saturnian_weight(&self) -> u32 {
        match self {
            MetatronicDimType::Foundation => 111,
            MetatronicDimType::Manifestation => 14,
            MetatronicDimType::Transcendence => 208,
            MetatronicDimType::ShellBoundary => 333,
        }
    }
}

// ══════════════════════════════════════════════════════════════
// COORDINATE CONVERSION
// ══════════════════════════════════════════════════════════════

/// Convert a MetatronicVertex (Rep A: {-1, 0, +1}) to a TorusCoordinate
/// (Rep B: {0, 1, 2}).
///
/// In the actual codebase:
/// ```rust
/// pub fn vertex_to_torus(v: &MetatronicVertex) -> TorusCoordinate {
///     TorusCoordinate::new(v.to_rep_b().iter().map(|&b| b as u16).collect())
/// }
/// ```
pub fn rep_a_to_torus_coords(rep_a: &[i8; METATRONIC_DIM]) -> Vec<u16> {
    rep_a.iter().map(|&c| (c + 1) as u16).collect()
}

/// Convert a TorusCoordinate (Rep B: Vec<u16>) back to Rep A coords.
///
/// Returns None if any coordinate is outside {0, 1, 2} or wrong dimension.
pub fn torus_coords_to_rep_a(coords: &[u16]) -> Option<[i8; METATRONIC_DIM]> {
    if coords.len() != METATRONIC_DIM {
        return None;
    }
    let mut rep_a = [0i8; METATRONIC_DIM];
    for i in 0..METATRONIC_DIM {
        if coords[i] > 2 { return None; }
        rep_a[i] = coords[i] as i8 - 1;
    }
    Some(rep_a)
}

/// Convert Rep A to Rep C {1, 2, 3} for sentinel validation.
pub fn rep_a_to_rep_c(rep_a: &[i8; METATRONIC_DIM]) -> [u8; METATRONIC_DIM] {
    let mut rep_c = [0u8; METATRONIC_DIM];
    for i in 0..METATRONIC_DIM {
        rep_c[i] = (rep_a[i] + 2) as u8;
    }
    rep_c
}

/// Validate a Rep C address: all digits must be in {1, 2, 3}.
/// Returns the index of the first violation (0 digit), or None if valid.
pub fn rep_c_sentinel_check(rep_c: &[u8; METATRONIC_DIM]) -> Option<usize> {
    // Constant-time: check all positions, report first violation
    let mut first_bad: Option<usize> = None;
    for i in 0..METATRONIC_DIM {
        if rep_c[i] == 0 || rep_c[i] > 3 {
            if first_bad.is_none() {
                first_bad = Some(i);
            }
        }
    }
    first_bad
}

/// Convert a linear node index (0..1,594,323) to Rep A coordinates.
pub fn linear_to_rep_a(mut idx: usize) -> [i8; METATRONIC_DIM] {
    let mut coords = [0i8; METATRONIC_DIM];
    for i in 0..METATRONIC_DIM {
        coords[i] = (idx % 3) as i8 - 1;
        idx /= 3;
    }
    coords
}

/// Convert Rep A coordinates to a linear index.
pub fn rep_a_to_linear(coords: &[i8; METATRONIC_DIM]) -> usize {
    let mut idx = 0usize;
    let mut power = 1usize;
    for i in 0..METATRONIC_DIM {
        idx += ((coords[i] + 1) as usize) * power;
        power *= 3;
    }
    idx
}

// ══════════════════════════════════════════════════════════════
// TOPOLOGY INITIALIZATION
// ══════════════════════════════════════════════════════════════

/// The side lengths for a 13D ternary torus: all 3.
pub const METATRONIC_SIDE_LENGTHS: [u16; METATRONIC_DIM] = [3; METATRONIC_DIM];

/// Initialize a TorusTopology with Metatronic axis assignments.
///
/// This is the integration function — call it once at startup to create
/// a 13D torus with Saturnian weights and domain-typed dimensions.
///
/// In the actual codebase:
/// ```rust
/// pub fn create_metatronic_topology() -> NetworkResult<TorusTopology> {
///     let side_lengths = vec![3u16; METATRONIC_DIM];
///     let mut topo = TorusTopology::new(side_lengths)?;
///
///     for axis in 0..METATRONIC_DIM {
///         let dim_type = MetatronicDimType::from_axis(axis).unwrap();
///         topo.set_coefficient(
///             axis,
///             dim_type.saturnian_weight(),
///             dim_type.to_kernel_type(),
///         )?;
///     }
///
///     Ok(topo)
/// }
/// ```
pub fn metatronic_topology_config() -> Vec<(usize, u32, &'static str)> {
    let mut config = Vec::with_capacity(METATRONIC_DIM);
    for axis in 0..METATRONIC_DIM {
        let dt = MetatronicDimType::from_axis(axis).unwrap();
        config.push((axis, dt.saturnian_weight(), dt.kernel_type_name()));
    }
    config
}

// ══════════════════════════════════════════════════════════════
// SHELL-AWARE ROUTING
// ══════════════════════════════════════════════════════════════

/// Which Saturnian shell a node inhabits, given its Rep A coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Inner,  // depth = -1
    Void,   // depth = 0
    Outer,  // depth = +1
}

impl Shell {
    pub fn from_rep_a(coords: &[i8; METATRONIC_DIM]) -> Self {
        match coords[12] {
            -1 => Shell::Inner,
            0 => Shell::Void,
            1 => Shell::Outer,
            _ => Shell::Void, // fallback
        }
    }

    pub fn from_torus_coord(depth_val: u16) -> Self {
        match depth_val {
            0 => Shell::Inner,
            1 => Shell::Void,
            2 => Shell::Outer,
            _ => Shell::Void,
        }
    }
}

/// Classify a routing hop based on shell transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopType {
    /// Both source and destination in the same shell.
    IntraShell,
    /// Adjacent shells (Inner↔Void or Void↔Outer).
    DirectCorrespondence,
    /// Inner↔Outer (passes through Void logically).
    LongCorrespondence,
}

/// Classify the hop between two nodes.
pub fn classify_hop(src: &[i8; METATRONIC_DIM], dst: &[i8; METATRONIC_DIM]) -> HopType {
    let src_shell = Shell::from_rep_a(src);
    let dst_shell = Shell::from_rep_a(dst);

    if src_shell == dst_shell {
        HopType::IntraShell
    } else {
        match (src_shell, dst_shell) {
            (Shell::Inner, Shell::Outer) | (Shell::Outer, Shell::Inner) => {
                HopType::LongCorrespondence
            }
            _ => HopType::DirectCorrespondence,
        }
    }
}

/// Saturnian-weighted distance between two nodes.
///
/// Unlike the uniform Hamming distance, this weights each differing
/// axis by its Saturnian significance:
///   Central (111) > Outer (208) > Depth (333) > Inner (14)
///
/// The depth axis crossing (weight 333) is the most expensive single hop,
/// reflecting the security boundary between shells.
pub fn saturnian_distance(a: &[i8; METATRONIC_DIM], b: &[i8; METATRONIC_DIM]) -> u32 {
    let mut dist = 0u32;
    for i in 0..METATRONIC_DIM {
        if a[i] != b[i] {
            dist += SATURNIAN_WEIGHTS[i];
        }
    }
    dist
}

/// Saturnian-weighted torus distance (with wraparound).
///
/// On the torus, the distance along axis i is min(|a-b|, 3-|a-b|).
/// Since side length is 3, the max distance per axis is 1.
/// So torus distance equals Hamming distance for side=3 —
/// wraparound doesn't change the count, only the direction.
pub fn saturnian_torus_distance(a: &[u16; METATRONIC_DIM], b: &[u16; METATRONIC_DIM]) -> u32 {
    let mut dist = 0u32;
    for i in 0..METATRONIC_DIM {
        let diff = if a[i] > b[i] { a[i] - b[i] } else { b[i] - a[i] };
        let wrap = 3 - diff;
        let d = core::cmp::min(diff, wrap);
        if d > 0 {
            dist += SATURNIAN_WEIGHTS[i];
        }
    }
    dist
}

// ══════════════════════════════════════════════════════════════
// SPONGE STATE ↔ METATRONIC EMBEDDING
// ══════════════════════════════════════════════════════════════

/// Map a sponge state index (0..729) to a point in the Metatronic Cube.
///
/// The 6 sponge dimensions embed into inner-ring axes (1..6).
/// All other coordinates are zero → Void shell, Central origin, neutral Outer.
pub fn sponge_index_to_metatronic(sponge_idx: usize) -> [i8; METATRONIC_DIM] {
    let mut coords = [0i8; METATRONIC_DIM];
    let mut idx = sponge_idx;
    for axis in 1..=6 {
        coords[axis] = (idx % 3) as i8 - 1;
        idx /= 3;
    }
    coords
}

/// Map a Metatronic vertex back to a sponge index (inner-ring axes only).
pub fn metatronic_to_sponge_index(coords: &[i8; METATRONIC_DIM]) -> usize {
    let mut idx = 0usize;
    let mut power = 1usize;
    for axis in 1..=6 {
        idx += ((coords[axis] + 1) as usize) * power;
        power *= 3;
    }
    idx
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rep_a_to_torus_roundtrip() {
        let rep_a = [1i8, -1, 0, 1, 0, -1, 1, 0, -1, 1, 0, -1, 1];
        let torus = rep_a_to_torus_coords(&rep_a);
        assert_eq!(torus.len(), 13);
        assert_eq!(torus[0], 2); // 1 + 1
        assert_eq!(torus[1], 0); // -1 + 1
        assert_eq!(torus[2], 1); // 0 + 1

        let back = torus_coords_to_rep_a(&torus).unwrap();
        assert_eq!(back, rep_a);
    }

    #[test]
    fn test_rep_c_sentinel() {
        let good: [u8; 13] = [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert!(rep_c_sentinel_check(&good).is_none());

        let bad: [u8; 13] = [1, 2, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1];
        assert_eq!(rep_c_sentinel_check(&bad), Some(2));
    }

    #[test]
    fn test_linear_roundtrip() {
        for idx in [0, 1, 728, 1000, 531_440, 1_594_322] {
            let coords = linear_to_rep_a(idx);
            let back = rep_a_to_linear(&coords);
            assert_eq!(back, idx);
        }
    }

    #[test]
    fn test_shell_classification() {
        let inner = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1];
        assert_eq!(Shell::from_rep_a(&inner), Shell::Inner);

        let void = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(Shell::from_rep_a(&void), Shell::Void);

        let outer = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        assert_eq!(Shell::from_rep_a(&outer), Shell::Outer);
    }

    #[test]
    fn test_hop_classification() {
        let inner = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1];
        let void = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let outer = [0i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

        assert_eq!(classify_hop(&inner, &inner), HopType::IntraShell);
        assert_eq!(classify_hop(&inner, &void), HopType::DirectCorrespondence);
        assert_eq!(classify_hop(&void, &outer), HopType::DirectCorrespondence);
        assert_eq!(classify_hop(&inner, &outer), HopType::LongCorrespondence);
    }

    #[test]
    fn test_saturnian_distance() {
        let a = [0i8; 13];

        // Differ on Central (axis 0): weight 111
        let mut b = [0i8; 13];
        b[0] = 1;
        assert_eq!(saturnian_distance(&a, &b), 111);

        // Differ on Inner (axis 3): weight 14
        let mut c = [0i8; 13];
        c[3] = 1;
        assert_eq!(saturnian_distance(&a, &c), 14);

        // Differ on Outer (axis 9): weight 208
        let mut d = [0i8; 13];
        d[9] = 1;
        assert_eq!(saturnian_distance(&a, &d), 208);

        // Differ on Depth (axis 12): weight 333
        let mut e = [0i8; 13];
        e[12] = 1;
        assert_eq!(saturnian_distance(&a, &e), 333);
    }

    #[test]
    fn test_topology_config() {
        let config = metatronic_topology_config();
        assert_eq!(config.len(), 13);

        // Axis 0: Central → Ternary, weight 111
        assert_eq!(config[0], (0, 111, "Ternary"));

        // Axis 3: Inner → Phase, weight 14
        assert_eq!(config[3], (3, 14, "Phase"));

        // Axis 9: Outer → Angular, weight 208
        assert_eq!(config[9], (9, 208, "Angular"));

        // Axis 12: Depth → Security, weight 333
        assert_eq!(config[12], (12, 333, "Security"));
    }

    #[test]
    fn test_sponge_embedding_roundtrip() {
        for idx in [0, 1, 100, 364, 728] {
            let coords = sponge_index_to_metatronic(idx);
            let back = metatronic_to_sponge_index(&coords);
            assert_eq!(back, idx, "Sponge roundtrip failed for {}", idx);
        }
    }

    #[test]
    fn test_sponge_embedding_lands_in_void() {
        for idx in 0..729 {
            let coords = sponge_index_to_metatronic(idx);
            assert_eq!(Shell::from_rep_a(&coords), Shell::Void);
            assert_eq!(coords[0], 0);  // Central neutral
            assert_eq!(coords[12], 0); // Depth neutral (Void)
            // Outer ring neutral
            for ax in 7..=11 {
                assert_eq!(coords[ax], 0);
            }
        }
    }

    #[test]
    fn test_saturnian_weights_sum() {
        let total: u32 = SATURNIAN_WEIGHTS.iter().sum();
        // 111 + 6×14 + 5×208 + 333 = 111 + 84 + 1040 + 333 = 1568
        assert_eq!(total, 1568);
    }

    #[test]
    fn test_metatronic_dim_types_complete() {
        for axis in 0..13 {
            assert!(MetatronicDimType::from_axis(axis).is_some());
        }
        assert!(MetatronicDimType::from_axis(13).is_none());
    }
}