// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Cube Address — Rep C 13-Trit Bijective Ternary Address
//!
//! The fundamental addressing type for PlenumNET's 13-dimensional Metatronic Cube.
//!
//! ## Design
//!
//! Every cube in the network has a 13-trit address in **Rep C** (digits {1, 2, 3}).
//! Zero never appears — its presence is structural proof of forgery.
//!
//! The 13D cube has 3¹³ = 1,594,323 vertices. Each vertex is a routable address.
//! Neighbor relationships, routing decisions, path counts, and security shell
//! membership are all **computed** from the trit coordinates — never stored.
//!
//! ## Integration with ternary-math
//!
//! `CubeAddr` wraps a `[RepCTrit; 13]` array. Conversion to/from `Gf3Vec`
//! (Rep B: {0,1,2}) uses the standard bijection `C→B: f(c) = c - 1`.
//! The torus topology in `ternary_math::torus` operates in Rep B; all
//! inter-cube services operate in Rep C at the boundary.

use std::fmt;
use std::hash::{Hash, Hasher};
use ternary_math::gf3::{Gf3, Gf3Vec};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Number of dimensions in the Metatronic Cube.
pub const DIMENSIONS: usize = 13;

/// Total vertices in the 13D ternary cube: 3¹³ = 1,594,323.
pub const TOTAL_VERTICES: u64 = 1_594_323;

/// Neighbors per cube: 13 dimensions × 2 alternative values = 26.
pub const NEIGHBORS_PER_CUBE: usize = 26;

/// Maximum Hamming distance (worst case hop count): 13.
pub const MAX_HAMMING_DISTANCE: usize = DIMENSIONS;

/// Valid Rep C digit values.
pub const VALID_DIGITS: [u8; 3] = [1, 2, 3];

// ═══════════════════════════════════════════════════════════════════════
// Rep C Trit — single digit {1, 2, 3}
// ═══════════════════════════════════════════════════════════════════════

/// A single Rep C trit. Invariant: value ∈ {1, 2, 3}.
/// Zero is structurally impossible by construction.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RepCTrit(u8);

impl RepCTrit {
    pub const ONE: RepCTrit = RepCTrit(1);
    pub const TWO: RepCTrit = RepCTrit(2);
    pub const THREE: RepCTrit = RepCTrit(3);

    /// All valid Rep C trit values, for exhaustive iteration.
    pub const ALL: [RepCTrit; 3] = [RepCTrit::ONE, RepCTrit::TWO, RepCTrit::THREE];

    /// Create from raw value. Panics if value ∉ {1, 2, 3}.
    #[inline]
    pub fn new(value: u8) -> Self {
        assert!(
            value >= 1 && value <= 3,
            "Rep C trit must be 1, 2, or 3, got {value}"
        );
        RepCTrit(value)
    }

    /// Create from raw value, returning None if invalid.
    /// This is the sentinel check: if this returns None, the input
    /// contained a zero (or >3) and is therefore forged.
    #[inline]
    pub fn try_new(value: u8) -> Option<Self> {
        if value >= 1 && value <= 3 {
            Some(RepCTrit(value))
        } else {
            None
        }
    }

    /// Raw value.
    #[inline]
    pub fn value(self) -> u8 {
        self.0
    }

    /// Convert Rep C → Rep B: f(c) = c - 1.
    /// Bijection: {1,2,3} → {0,1,2}.
    #[inline]
    pub fn to_gf3(self) -> Gf3 {
        Gf3::new(self.0 - 1)
    }

    /// Convert Rep B → Rep C: f(b) = b + 1.
    /// Bijection: {0,1,2} → {1,2,3}.
    #[inline]
    pub fn from_gf3(g: Gf3) -> Self {
        RepCTrit(g.value() + 1)
    }

    /// The two alternative values for this trit (the "other" neighbors).
    /// For trit value v, returns the two elements of {1,2,3} \ {v}.
    #[inline]
    pub fn alternatives(self) -> [RepCTrit; 2] {
        match self.0 {
            1 => [RepCTrit::TWO, RepCTrit::THREE],
            2 => [RepCTrit::ONE, RepCTrit::THREE],
            3 => [RepCTrit::ONE, RepCTrit::TWO],
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for RepCTrit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for RepCTrit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CubeAddr — 13-trit Rep C address
// ═══════════════════════════════════════════════════════════════════════

/// A 13-trit Rep C address in the Metatronic Cube.
///
/// This is the primary addressing type for all inter-cube infrastructure.
/// Zero cannot appear in any position — its detection is instant proof
/// of forgery. Validation is constant-time (no branching on content).
#[derive(Clone, PartialEq, Eq)]
pub struct CubeAddr {
    trits: [RepCTrit; DIMENSIONS],
}

impl CubeAddr {
    /// Create a new CubeAddr from 13 Rep C trits.
    /// Panics if any trit is not in {1, 2, 3}.
    pub fn new(trits: [u8; DIMENSIONS]) -> Self {
        let mut addr = [RepCTrit::ONE; DIMENSIONS];
        for (i, &t) in trits.iter().enumerate() {
            addr[i] = RepCTrit::new(t);
        }
        CubeAddr { trits: addr }
    }

    /// Attempt to create from raw bytes. Returns None if any byte
    /// is zero or > 3 — this IS the forgery detection.
    ///
    /// Validation is constant-time: accumulates validity across all
    /// trits without early exit, preventing timing side-channels.
    pub fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != DIMENSIONS {
            return None;
        }
        let mut addr = [RepCTrit::ONE; DIMENSIONS];
        let mut valid: u8 = 1;
        for (i, &b) in bytes.iter().enumerate() {
            // Constant-time validity check: accumulate AND of (b >= 1 && b <= 3)
            let in_range = ((b.wrapping_sub(1)) < 3) as u8;
            valid &= in_range;
            // Safe because we check `valid` below — but fill the array regardless
            // to avoid branching on secret data.
            addr[i] = RepCTrit(if b >= 1 && b <= 3 { b } else { 1 });
        }
        if valid == 1 {
            Some(CubeAddr { trits: addr })
        } else {
            None // Zero detected — forgery or corruption
        }
    }

    /// Access a single trit by dimension index (0..12).
    #[inline]
    pub fn trit(&self, dim: usize) -> RepCTrit {
        self.trits[dim]
    }

    /// Set a single trit at the given dimension.
    #[inline]
    pub fn set_trit(&mut self, dim: usize, value: RepCTrit) {
        self.trits[dim] = value;
    }

    /// Get the raw trit array.
    #[inline]
    pub fn trits(&self) -> &[RepCTrit; DIMENSIONS] {
        &self.trits
    }

    /// Export as raw byte array.
    pub fn to_bytes(&self) -> [u8; DIMENSIONS] {
        let mut out = [0u8; DIMENSIONS];
        for (i, t) in self.trits.iter().enumerate() {
            out[i] = t.value();
        }
        out
    }

    /// Convert to GF(3) vector (Rep B) for interop with ternary-math torus.
    pub fn to_gf3_vec(&self) -> Gf3Vec {
        let v: Vec<Gf3> = self.trits.iter().map(|t| t.to_gf3()).collect();
        Gf3Vec::new(v)
    }

    /// Create from GF(3) vector (Rep B → Rep C).
    pub fn from_gf3_vec(gv: &Gf3Vec) -> Option<Self> {
        if gv.len() != DIMENSIONS {
            return None;
        }
        let mut addr = [RepCTrit::ONE; DIMENSIONS];
        for i in 0..DIMENSIONS {
            addr[i] = RepCTrit::from_gf3(gv.get(i));
        }
        Some(CubeAddr { trits: addr })
    }

    // ═══════════════════════════════════════════════════════════════
    // PURE MATH ROUTING — No Routing Tables
    // ═══════════════════════════════════════════════════════════════

    /// Compute the delta set: dimensions where `self` and `other` differ.
    /// This IS the routing information. |delta| = Hamming distance = hop count.
    pub fn delta(&self, other: &CubeAddr) -> Vec<usize> {
        let mut d = Vec::with_capacity(DIMENSIONS);
        for i in 0..DIMENSIONS {
            if self.trits[i] != other.trits[i] {
                d.push(i);
            }
        }
        d
    }

    /// Hamming distance — the number of hops between two cubes.
    /// Equivalent to `delta().len()` but avoids allocation.
    #[inline]
    pub fn hamming_distance(&self, other: &CubeAddr) -> usize {
        let mut d = 0usize;
        for i in 0..DIMENSIONS {
            if self.trits[i] != other.trits[i] {
                d += 1;
            }
        }
        d
    }

    /// Compute the next hop toward `destination` by fixing dimension `dim`.
    /// Returns a new CubeAddr with `self[dim]` replaced by `destination[dim]`.
    /// This is one step of the pure-math routing algorithm.
    pub fn step_toward(&self, destination: &CubeAddr, dim: usize) -> CubeAddr {
        let mut next = self.clone();
        next.trits[dim] = destination.trits[dim];
        next
    }

    /// Number of shortest paths between `self` and `other`: delta! (factorial).
    /// For d differing dimensions, there are d! orderings of corrections.
    pub fn shortest_path_count(&self, other: &CubeAddr) -> u64 {
        let d = self.hamming_distance(other);
        factorial(d as u64)
    }

    // ═══════════════════════════════════════════════════════════════
    // NEIGHBOR COMPUTATION — Pure Arithmetic
    // ═══════════════════════════════════════════════════════════════

    /// Compute all 26 geometric neighbors.
    /// For each of 13 dimensions, flip the trit to its 2 alternative values.
    /// No storage — computed fresh each call.
    pub fn neighbors(&self) -> Vec<CubeAddr> {
        let mut nbrs = Vec::with_capacity(NEIGHBORS_PER_CUBE);
        for dim in 0..DIMENSIONS {
            for alt in self.trits[dim].alternatives() {
                let mut neighbor = self.clone();
                neighbor.trits[dim] = alt;
                nbrs.push(neighbor);
            }
        }
        nbrs
    }

    /// Compute the single neighbor at a specific dimension and alternative value.
    #[inline]
    pub fn neighbor_at(&self, dim: usize, alt: RepCTrit) -> CubeAddr {
        debug_assert!(alt != self.trits[dim], "Alternative must differ from current");
        let mut n = self.clone();
        n.trits[dim] = alt;
        n
    }

    // ═══════════════════════════════════════════════════════════════
    // SECURITY SHELL CLASSIFICATION
    // ═══════════════════════════════════════════════════════════════

    /// The Hamming weight — how many trits differ from "all ones" (the inner corner).
    /// Used for Inner / Void / Outer shell classification.
    pub fn hamming_weight_from_inner(&self) -> usize {
        let inner = CubeAddr::inner_corner();
        self.hamming_distance(&inner)
    }

    /// The inner corner address: all 1s (the geometric center).
    pub fn inner_corner() -> CubeAddr {
        CubeAddr {
            trits: [RepCTrit::ONE; DIMENSIONS],
        }
    }

    /// The outer corner address: all 3s (the geometric extremity).
    pub fn outer_corner() -> CubeAddr {
        CubeAddr {
            trits: [RepCTrit::THREE; DIMENSIONS],
        }
    }

    /// Flat index: convert the 13-trit address to a scalar in [0, 3¹³).
    /// Used for bitmap indexing in the CRS address allocator.
    /// Computed as: Σ (trit[i] - 1) × 3^i for i in 0..12.
    pub fn flat_index(&self) -> u64 {
        let mut idx: u64 = 0;
        let mut power: u64 = 1;
        for i in 0..DIMENSIONS {
            idx += (self.trits[i].value() as u64 - 1) * power;
            power *= 3;
        }
        idx
    }

    /// Reconstruct from flat index.
    pub fn from_flat_index(mut idx: u64) -> Option<CubeAddr> {
        if idx >= TOTAL_VERTICES {
            return None;
        }
        let mut trits = [RepCTrit::ONE; DIMENSIONS];
        for i in 0..DIMENSIONS {
            let digit = (idx % 3) as u8 + 1; // Rep C: offset by 1
            trits[i] = RepCTrit::new(digit);
            idx /= 3;
        }
        Some(CubeAddr { trits })
    }
}

impl Hash for CubeAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for t in &self.trits {
            t.value().hash(state);
        }
    }
}

impl fmt::Debug for CubeAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C[")?;
        for (i, t) in self.trits.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", t.value())?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for CubeAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.trits {
            write!(f, "{}", t.value())?;
        }
        Ok(())
    }
}

impl CubeAddr {
    /// Dotted notation: 111.111.111.111.2 (groups of 3 + final trit)
    pub fn to_dotted(&self) -> String {
        let flat: String = self.trits.iter().map(|t| char::from(b'0' + t.value())).collect();
        format!("{}.{}.{}.{}.{}",
            &flat[0..3], &flat[3..6], &flat[6..9], &flat[9..12], &flat[12..13])
    }

    /// Per-digit dot-separated Rep C: 1.1.1.1.1.1.1.1.1.1.1.1.1
    /// Used by attestation logs and operator-facing surfaces for maximum clarity.
    pub fn to_rep_c_display(&self) -> String {
        self.trits
            .iter()
            .map(|t| String::from(char::from(b'0' + t.value())))
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Parse from either flat (1111111111112) or dotted (111.111.111.111.2) notation.
    /// Returns None if length is wrong or any trit is outside {1,2,3}.
    pub fn parse(s: &str) -> Option<Self> {
        let flat: String = s.chars().filter(|c| *c != '.').collect();
        if flat.len() != DIMENSIONS {
            return None;
        }
        let bytes: Vec<u8> = flat
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u8))
            .collect::<Option<Vec<u8>>>()?;
        Self::try_from_bytes(&bytes)
    }
}


// ═══════════════════════════════════════════════════════════════════════
// MULTI-LEVEL ADDRESSING — Cube of Cubes
// ═══════════════════════════════════════════════════════════════════════

/// A multi-level address: concatenated 13-trit segments.
///
/// Level 1: [13 trits]                         → 1.59M nodes
/// Level 2: [13 trits : 13 trits]              → 2.54 trillion nodes
/// Level 3: [13 trits : 13 trits : 13 trits]   → 4.05 × 10¹⁸ nodes
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MultiLevelAddr {
    levels: Vec<CubeAddr>,
}

impl MultiLevelAddr {
    /// Create a single-level address.
    pub fn single(addr: CubeAddr) -> Self {
        MultiLevelAddr {
            levels: vec![addr],
        }
    }

    /// Create a multi-level address from concatenated cube addresses.
    pub fn new(levels: Vec<CubeAddr>) -> Self {
        assert!(!levels.is_empty(), "At least one level required");
        MultiLevelAddr { levels }
    }

    /// Number of levels in the hierarchy.
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// Get the address at a specific level (0 = outermost cube).
    pub fn level(&self, idx: usize) -> &CubeAddr {
        &self.levels[idx]
    }

    /// The outermost (cube-level) address — used for inter-cube routing.
    pub fn outer(&self) -> &CubeAddr {
        &self.levels[0]
    }

    /// The innermost (node-level) address — used for intra-cube routing.
    pub fn inner(&self) -> &CubeAddr {
        &self.levels[self.levels.len() - 1]
    }

    /// Multi-level routing: compare outer trits first.
    /// Returns the first level at which addresses differ.
    pub fn routing_level(&self, other: &MultiLevelAddr) -> Option<usize> {
        let depth = self.depth().min(other.depth());
        for lvl in 0..depth {
            if self.levels[lvl] != other.levels[lvl] {
                return Some(lvl);
            }
        }
        None // Same address at all levels
    }
}

impl fmt::Display for MultiLevelAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, level) in self.levels.iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            write!(f, "{}", level)?;
        }
        Ok(())
    }
}

impl fmt::Debug for MultiLevelAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ML[")?;
        for (i, level) in self.levels.iter().enumerate() {
            if i > 0 {
                write!(f, " : ")?;
            }
            write!(f, "{:?}", level)?;
        }
        write!(f, "]")
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Factorial for path count computation. Caps at 13! (fits in u64).
fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        2 => 2,
        3 => 6,
        4 => 24,
        5 => 120,
        6 => 720,
        7 => 5_040,
        8 => 40_320,
        9 => 362_880,
        10 => 3_628_800,
        11 => 39_916_800,
        12 => 479_001_600,
        13 => 6_227_020_800,
        _ => {
            let mut f: u64 = 1;
            for i in 2..=n {
                f = f.saturating_mul(i);
            }
            f
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repc_trit_valid() {
        assert!(RepCTrit::try_new(1).is_some());
        assert!(RepCTrit::try_new(2).is_some());
        assert!(RepCTrit::try_new(3).is_some());
    }

    #[test]
    fn test_repc_trit_zero_rejected() {
        assert!(RepCTrit::try_new(0).is_none(), "Zero must be rejected in Rep C");
    }

    #[test]
    fn test_repc_trit_four_rejected() {
        assert!(RepCTrit::try_new(4).is_none());
    }

    #[test]
    fn test_repc_gf3_roundtrip() {
        for &v in &VALID_DIGITS {
            let rc = RepCTrit::new(v);
            let gf = rc.to_gf3();
            let back = RepCTrit::from_gf3(gf);
            assert_eq!(rc, back, "Rep C ↔ GF(3) roundtrip failed for {v}");
        }
    }

    #[test]
    fn test_alternatives() {
        assert_eq!(RepCTrit::ONE.alternatives(), [RepCTrit::TWO, RepCTrit::THREE]);
        assert_eq!(RepCTrit::TWO.alternatives(), [RepCTrit::ONE, RepCTrit::THREE]);
        assert_eq!(RepCTrit::THREE.alternatives(), [RepCTrit::ONE, RepCTrit::TWO]);
    }

    #[test]
    fn test_cube_addr_new() {
        let addr = CubeAddr::new([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        assert_eq!(addr.trit(0).value(), 1);
        assert_eq!(addr.trit(12).value(), 1);
    }

    #[test]
    fn test_zero_in_address_detected() {
        let result = CubeAddr::try_from_bytes(&[1, 2, 0, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        assert!(result.is_none(), "Zero in address must be rejected");
    }

    #[test]
    fn test_neighbor_count() {
        let addr = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(addr.neighbors().len(), NEIGHBORS_PER_CUBE);
    }

    #[test]
    fn test_hamming_distance_self() {
        let addr = CubeAddr::new([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]);
        assert_eq!(addr.hamming_distance(&addr), 0);
    }

    #[test]
    fn test_hamming_distance_one() {
        let a = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(a.hamming_distance(&b), 1);
    }

    #[test]
    fn test_hamming_distance_max() {
        let a = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = CubeAddr::new([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        assert_eq!(a.hamming_distance(&b), 13);
    }

    #[test]
    fn test_shortest_path_count() {
        let a = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = CubeAddr::new([2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        // d=3 → 3! = 6 shortest paths
        assert_eq!(a.shortest_path_count(&b), 6);
    }

    #[test]
    fn test_step_toward() {
        let src = CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dst = CubeAddr::new([3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let hop = src.step_toward(&dst, 0);
        assert_eq!(hop.trit(0).value(), 3);
        assert_eq!(hop.trit(1).value(), 1); // unchanged
    }

    #[test]
    fn test_flat_index_roundtrip() {
        let addr = CubeAddr::new([2, 3, 1, 1, 2, 3, 2, 1, 3, 1, 2, 2, 3]);
        let idx = addr.flat_index();
        let back = CubeAddr::from_flat_index(idx).unwrap();
        assert_eq!(addr, back);
    }

    #[test]
    fn test_flat_index_inner_corner() {
        let inner = CubeAddr::inner_corner();
        assert_eq!(inner.flat_index(), 0); // all 1s → all zeros in Rep B → index 0
    }

    #[test]
    fn test_multi_level_routing() {
        let a = MultiLevelAddr::new(vec![
            CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            CubeAddr::new([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]),
        ]);
        let b = MultiLevelAddr::new(vec![
            CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            CubeAddr::new([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]),
        ]);
        // Same outer cube → routing happens at level 1 (intra-cube)
        assert_eq!(a.routing_level(&b), Some(1));
    }

    #[test]
    fn test_gf3_vec_roundtrip() {
        let addr = CubeAddr::new([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let gv = addr.to_gf3_vec();
        let back = CubeAddr::from_gf3_vec(&gv).unwrap();
        assert_eq!(addr, back);
    }
}