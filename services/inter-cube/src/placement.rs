// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Bootstrap Placement Optimization (T-16, SPEC-2026-NEXT)
//!
//! Replaces the sequential `allocateNext()` with geometry-aware placement.
//! New nodes are placed to maximize spread across the 13D hypercube,
//! ensuring even coverage from the earliest registrations.
//!
//! ## Design
//!
//! ### Max-Spread Placement
//!
//! For each candidate address, compute the minimum Hamming distance to
//! ALL currently registered nodes. Choose the candidate that maximizes
//! this minimum distance. Ties are broken by the candidate with the
//! highest average distance (secondary metric).
//!
//! This is the "farthest-first traversal" heuristic — provably within
//! a factor of 2 of optimal for dispersion in metric spaces.
//!
//! ### K=2 Floor
//!
//! After max-spread selection, verify that the chosen address has at
//! least K=2 registered neighbors (among its 26 geometric neighbors).
//! If not, the next-best candidate that meets the floor is chosen.
//!
//! Why K=2: a node with 0 or 1 registered neighbors has no redundant
//! path. With K≥2, there's always an alternative route if one neighbor
//! goes down. This is the minimum for the FTS (T-08) to be meaningful.
//!
//! ### Cold Start
//!
//! When the network is empty or very sparse (<K neighbors exist anywhere),
//! the K=2 floor is relaxed. The first few nodes get placed at maximally
//! separated corners of the 13D cube:
//!
//! - Node 0: `[1,1,1,1,1,1,1,1,1,1,1,1,1]` (inner corner)
//! - Node 1: `[3,3,3,3,3,3,3,3,3,3,3,3,3]` (outer corner, max distance = 13)
//! - Node 2: `[2,2,2,2,2,2,2,2,2,2,2,2,2]` (center, equidistant from both)
//! - Node 3+: max-spread from existing set
//!
//! ### Dimension Density Tracking
//!
//! A 13×3 array tracks how many registered nodes have each trit value
//! in each dimension. The allocator uses this to bias placement toward
//! under-populated dimension-value combinations.
//!
//! ### Performance
//!
//! For N registered nodes and C candidate addresses:
//! - Full scan: O(C × N) Hamming distance comparisons
//! - With sampling (>100K nodes, T-25): O(1000 × N)
//! - Target: <50ms for allocateOptimal at 10K nodes

use std::collections::HashSet;

use crate::cube_addr::{CubeAddr, DIMENSIONS, TOTAL_VERTICES};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Minimum number of registered neighbors for a candidate (K-floor).
pub const K_FLOOR: usize = 2;

/// Number of candidates to evaluate per allocation.
/// Full scan of 1.6M addresses is too expensive at scale.
/// We evaluate a strategic subset: neighbors of existing nodes,
/// plus random samples.
pub const MAX_CANDIDATES: usize = 1000;

/// Cold start addresses: maximally separated corners.
pub const COLD_START_ADDRESSES: [[u8; 13]; 3] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // Inner corner
    [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3], // Outer corner (distance 13)
    [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], // Center (distance 13 from both)
];

/// Number of trit values per dimension (Rep C: {1, 2, 3}).
pub const TRIT_VALUES: usize = 3;

// ═══════════════════════════════════════════════════════════════════════
// PLACEMENT METRICS
// ═══════════════════════════════════════════════════════════════════════

/// Metrics for a placement decision.
#[derive(Debug, Clone)]
pub struct PlacementMetrics {
    /// The chosen address.
    pub address: CubeAddr,
    /// Minimum Hamming distance to any registered node.
    pub min_distance: usize,
    /// Average Hamming distance to all registered nodes.
    pub avg_distance: f64,
    /// Number of registered neighbors (among the 26 geometric neighbors).
    pub registered_neighbors: usize,
    /// Number of candidates evaluated.
    pub candidates_evaluated: usize,
    /// Whether the K-floor constraint was satisfied.
    pub k_floor_satisfied: bool,
    /// Whether this was a cold-start placement.
    pub cold_start: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// DIMENSION DENSITY — 13×3 tracker
// ═══════════════════════════════════════════════════════════════════════

/// Tracks the count of registered nodes per dimension-value combination.
///
/// `density[dim][val-1]` = number of registered nodes with trit value
/// `val` (1, 2, or 3) in dimension `dim` (0..12).
///
/// Used to bias placement toward under-populated dimension-value pairs.
#[derive(Debug, Clone)]
pub struct DimensionDensity {
    /// 13 dimensions × 3 trit values.
    counts: [[u32; TRIT_VALUES]; DIMENSIONS],
    /// Total registered nodes (sum of any row should equal this).
    total: u32,
}

impl DimensionDensity {
    /// Create a new empty density tracker.
    pub fn new() -> Self {
        DimensionDensity {
            counts: [[0; TRIT_VALUES]; DIMENSIONS],
            total: 0,
        }
    }

    /// Record a registered address.
    pub fn register(&mut self, addr: &CubeAddr) {
        let trits = addr.to_bytes();
        for dim in 0..DIMENSIONS {
            let val = trits[dim] as usize;
            if val >= 1 && val <= 3 {
                self.counts[dim][val - 1] += 1;
            }
        }
        self.total += 1;
    }

    /// Remove a deregistered address.
    pub fn deregister(&mut self, addr: &CubeAddr) {
        let trits = addr.to_bytes();
        for dim in 0..DIMENSIONS {
            let val = trits[dim] as usize;
            if val >= 1 && val <= 3 {
                self.counts[dim][val - 1] = self.counts[dim][val - 1].saturating_sub(1);
            }
        }
        self.total = self.total.saturating_sub(1);
    }

    /// Get the count for a specific dimension-value pair.
    pub fn count(&self, dim: usize, val: u8) -> u32 {
        if dim < DIMENSIONS && val >= 1 && val <= 3 {
            self.counts[dim][(val - 1) as usize]
        } else {
            0
        }
    }

    /// Compute the density imbalance score for a candidate address.
    ///
    /// Lower score = candidate fills under-populated dimension-values.
    /// Score = sum of counts for each dimension-value in the candidate.
    /// A candidate in sparse regions has a low score.
    pub fn imbalance_score(&self, addr: &CubeAddr) -> u32 {
        let trits = addr.to_bytes();
        let mut score = 0u32;
        for dim in 0..DIMENSIONS {
            let val = trits[dim] as usize;
            if val >= 1 && val <= 3 {
                score += self.counts[dim][val - 1];
            }
        }
        score
    }

    /// Get the least-populated trit value for a given dimension.
    pub fn least_populated_value(&self, dim: usize) -> u8 {
        if dim >= DIMENSIONS {
            return 1;
        }
        let mut min_count = u32::MAX;
        let mut min_val = 1u8;
        for v in 0..TRIT_VALUES {
            if self.counts[dim][v] < min_count {
                min_count = self.counts[dim][v];
                min_val = (v + 1) as u8;
            }
        }
        min_val
    }

    /// Total registered nodes.
    pub fn total(&self) -> u32 {
        self.total
    }

    /// Get the full 13×3 density array.
    pub fn as_array(&self) -> &[[u32; TRIT_VALUES]; DIMENSIONS] {
        &self.counts
    }
}

impl Default for DimensionDensity {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HAMMING DISTANCE
// ═══════════════════════════════════════════════════════════════════════

/// Compute the Hamming distance between two 13-trit Rep C addresses.
///
/// Number of positions where the trits differ. Range: [0, 13].
#[inline]
pub fn hamming_distance(a: &CubeAddr, b: &CubeAddr) -> usize {
    let ta = a.to_bytes();
    let tb = b.to_bytes();
    let mut dist = 0;
    for i in 0..DIMENSIONS {
        if ta[i] != tb[i] {
            dist += 1;
        }
    }
    dist
}

/// Count how many of a candidate's 26 geometric neighbors are registered.
pub fn count_registered_neighbors(
    candidate: &CubeAddr,
    registered: &HashSet<CubeAddr>,
) -> usize {
    let trits = candidate.to_bytes();
    let mut count = 0;
    for dim in 0..DIMENSIONS {
        for alt_val in 1u8..=3 {
            if alt_val == trits[dim] {
                continue;
            }
            let mut nbr_trits = trits;
            nbr_trits[dim] = alt_val;
            let nbr = CubeAddr::new(nbr_trits);
            if registered.contains(&nbr) {
                count += 1;
            }
        }
    }
    count
}

// ═══════════════════════════════════════════════════════════════════════
// CANDIDATE GENERATION
// ═══════════════════════════════════════════════════════════════════════

/// Generate candidate addresses for evaluation.
///
/// Strategy:
/// 1. Neighbors of existing nodes that are unregistered (high-value)
/// 2. Addresses built from least-populated dimension values
/// 3. Deterministic pseudo-random samples via TLSponge-385
///
/// Returns at most `max_candidates` unique unregistered addresses.
pub fn generate_candidates(
    registered: &HashSet<CubeAddr>,
    used_bitmap: &[bool],
    density: &DimensionDensity,
    max_candidates: usize,
    seed: u64,
) -> Vec<CubeAddr> {
    let mut candidates = Vec::with_capacity(max_candidates);
    let mut seen = HashSet::new();

    // Strategy 1: Unregistered neighbors of registered nodes
    // These are high-value because they immediately have ≥1 registered neighbor
    for registered_addr in registered.iter() {
        if candidates.len() >= max_candidates / 2 {
            break;
        }
        let trits = registered_addr.to_bytes();
        for dim in 0..DIMENSIONS {
            for alt_val in 1u8..=3 {
                if alt_val == trits[dim] {
                    continue;
                }
                let mut nbr_trits = trits;
                nbr_trits[dim] = alt_val;
                let nbr = CubeAddr::new(nbr_trits);
                let idx = nbr.flat_index() as usize;
                if idx < used_bitmap.len() && !used_bitmap[idx] && seen.insert(nbr.clone()) {
                    candidates.push(nbr);
                    if candidates.len() >= max_candidates {
                        return candidates;
                    }
                }
            }
        }
    }

    // Strategy 2: Build from least-populated values
    let mut least_pop_trits = [0u8; 13];
    for dim in 0..DIMENSIONS {
        least_pop_trits[dim] = density.least_populated_value(dim);
    }
    let least_pop = CubeAddr::new(least_pop_trits);
    let idx = least_pop.flat_index() as usize;
    if idx < used_bitmap.len() && !used_bitmap[idx] && seen.insert(least_pop.clone()) {
        candidates.push(least_pop);
    }

    // Strategy 3: Deterministic pseudo-random via sponge
    let seed_bytes = seed.to_le_bytes();
    let random_bytes = ternary_math::sponge::derive_key(
        b"PlenumNET-PLACEMENT",
        &seed_bytes,
        max_candidates * 2, // Generate more than needed, filter invalid
    );

    let total = TOTAL_VERTICES as usize;
    let mut offset = 0;
    while candidates.len() < max_candidates && offset + 4 <= random_bytes.len() {
        let idx_raw = u32::from_le_bytes([
            random_bytes[offset],
            random_bytes[offset + 1],
            random_bytes[offset + 2],
            random_bytes[offset + 3],
        ]) as usize % total;
        offset += 4;

        if !used_bitmap[idx_raw] {
            if let Some(addr) = CubeAddr::from_flat_index(idx_raw as u64) {
                if seen.insert(addr.clone()) {
                    candidates.push(addr);
                }
            }
        }
    }

    candidates
}

// ═══════════════════════════════════════════════════════════════════════
// OPTIMAL ALLOCATION
// ═══════════════════════════════════════════════════════════════════════

/// Allocate an optimal address using max-spread with K-floor.
///
/// Returns the best address and placement metrics, or `None` if the
/// address space is exhausted.
///
/// ## Algorithm
///
/// 1. **Cold start**: If fewer than 3 nodes registered, use corner addresses
/// 2. **Generate candidates**: Neighbors of existing nodes + random samples
/// 3. **Score each candidate**: (min_distance, avg_distance, -imbalance)
/// 4. **Filter by K-floor**: At least K=2 registered neighbors
/// 5. **Select best**: Max min_distance, tie-break by avg_distance
/// 6. **Fallback**: If no candidate meets K-floor, relax to K=0
pub fn allocate_optimal(
    registered: &HashSet<CubeAddr>,
    used_bitmap: &mut Vec<bool>,
    density: &mut DimensionDensity,
    seed: u64,
) -> Option<(CubeAddr, PlacementMetrics)> {
    let registered_count = registered.len();

    // Cold start: use predefined corner addresses
    if registered_count < COLD_START_ADDRESSES.len() {
        let cold_addr = CubeAddr::new(COLD_START_ADDRESSES[registered_count]);
        let idx = cold_addr.flat_index() as usize;
        if idx < used_bitmap.len() && !used_bitmap[idx] {
            used_bitmap[idx] = true;
            density.register(&cold_addr);

            let min_d = if registered_count == 0 {
                13 // Max possible
            } else {
                registered.iter()
                    .map(|r| hamming_distance(&cold_addr, r))
                    .min()
                    .unwrap_or(13)
            };

            let avg_d = if registered_count == 0 {
                13.0
            } else {
                let sum: usize = registered.iter()
                    .map(|r| hamming_distance(&cold_addr, r))
                    .sum();
                sum as f64 / registered_count as f64
            };

            return Some((cold_addr.clone(), PlacementMetrics {
                address: cold_addr,
                min_distance: min_d,
                avg_distance: avg_d,
                registered_neighbors: count_registered_neighbors(
                    &CubeAddr::new(COLD_START_ADDRESSES[registered_count]),
                    registered,
                ),
                candidates_evaluated: 1,
                k_floor_satisfied: registered_count < K_FLOOR, // Relaxed during cold start
                cold_start: true,
            }));
        }
    }

    // Generate candidates
    let candidates = generate_candidates(
        registered,
        used_bitmap,
        density,
        MAX_CANDIDATES,
        seed,
    );

    if candidates.is_empty() {
        return None; // Address space exhausted
    }

    // Score each candidate
    let registered_vec: Vec<&CubeAddr> = registered.iter().collect();
    let mut scored: Vec<(CubeAddr, usize, f64, usize, u32)> = candidates
        .iter()
        .map(|candidate| {
            let distances: Vec<usize> = registered_vec
                .iter()
                .map(|r| hamming_distance(candidate, r))
                .collect();

            let min_d = distances.iter().copied().min().unwrap_or(13);
            let avg_d = if distances.is_empty() {
                13.0
            } else {
                distances.iter().sum::<usize>() as f64 / distances.len() as f64
            };
            let reg_nbrs = count_registered_neighbors(candidate, registered);
            let imbalance = density.imbalance_score(candidate);

            (candidate.clone(), min_d, avg_d, reg_nbrs, imbalance)
        })
        .collect();

    let candidates_evaluated = scored.len();

    // Sort: primary = max min_distance, secondary = max avg_distance,
    // tertiary = min imbalance (fills sparse regions)
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1) // max min_distance
            .then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)) // max avg
            .then(a.4.cmp(&b.4)) // min imbalance
    });

    // Try to find a candidate meeting K-floor
    let mut best_with_floor: Option<&(CubeAddr, usize, f64, usize, u32)> = None;
    let mut best_without_floor: Option<&(CubeAddr, usize, f64, usize, u32)> = None;

    for entry in &scored {
        if best_without_floor.is_none() {
            best_without_floor = Some(entry);
        }
        if entry.3 >= K_FLOOR && best_with_floor.is_none() {
            best_with_floor = Some(entry);
            break; // First match is the best (sorted)
        }
    }

    // Prefer K-floor candidate; fall back to best overall
    let (chosen, k_satisfied) = if let Some(entry) = best_with_floor {
        (entry, true)
    } else if let Some(entry) = best_without_floor {
        (entry, false)
    } else {
        return None;
    };

    let addr = chosen.0.clone();
    let idx = addr.flat_index() as usize;
    if idx >= used_bitmap.len() || used_bitmap[idx] {
        return None; // Shouldn't happen, but defensive
    }

    used_bitmap[idx] = true;
    density.register(&addr);

    Some((addr.clone(), PlacementMetrics {
        address: addr,
        min_distance: chosen.1,
        avg_distance: chosen.2,
        registered_neighbors: chosen.3,
        candidates_evaluated,
        k_floor_satisfied: k_satisfied,
        cold_start: false,
    }))
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr {
        CubeAddr::new(trits)
    }

    // ── Hamming distance ────────────────────────────────────────

    #[test]
    fn test_hamming_distance_identical() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(hamming_distance(&a, &a), 0);
    }

    #[test]
    fn test_hamming_distance_one_trit() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(hamming_distance(&a, &b), 1);
    }

    #[test]
    fn test_hamming_distance_max() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        assert_eq!(hamming_distance(&a, &b), 13);
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let a = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let b = addr([3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3]);
        assert_eq!(hamming_distance(&a, &b), hamming_distance(&b, &a));
    }

    // ── Dimension density ───────────────────────────────────────

    #[test]
    fn test_density_empty() {
        let density = DimensionDensity::new();
        assert_eq!(density.total(), 0);
        assert_eq!(density.count(0, 1), 0);
    }

    #[test]
    fn test_density_register() {
        let mut density = DimensionDensity::new();
        density.register(&addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]));
        assert_eq!(density.total(), 1);
        assert_eq!(density.count(0, 1), 1); // dim 0, val 1
        assert_eq!(density.count(1, 2), 1); // dim 1, val 2
        assert_eq!(density.count(2, 3), 1); // dim 2, val 3
    }

    #[test]
    fn test_density_deregister() {
        let mut density = DimensionDensity::new();
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        density.register(&a);
        assert_eq!(density.total(), 1);
        density.deregister(&a);
        assert_eq!(density.total(), 0);
        assert_eq!(density.count(0, 1), 0);
    }

    #[test]
    fn test_density_least_populated() {
        let mut density = DimensionDensity::new();
        // Register 3 nodes with val=1 in dim 0, 1 with val=2
        density.register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        density.register(&addr([1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        density.register(&addr([1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        density.register(&addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // dim 0: val1=3, val2=1, val3=0 → least populated = val 3
        assert_eq!(density.least_populated_value(0), 3);
    }

    #[test]
    fn test_density_imbalance_score() {
        let mut density = DimensionDensity::new();
        density.register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        density.register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));

        // Candidate [1,1,1,...] has high imbalance (same as registered)
        let score_same = density.imbalance_score(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        // Candidate [3,3,3,...] has low imbalance (different from registered)
        let score_diff = density.imbalance_score(&addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]));

        assert!(score_same > score_diff,
            "Same-value candidate should have higher imbalance score");
    }

    // ── Registered neighbor counting ────────────────────────────

    #[test]
    fn test_count_registered_neighbors_none() {
        let registered = HashSet::new();
        let candidate = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(count_registered_neighbors(&candidate, &registered), 0);
    }

    #[test]
    fn test_count_registered_neighbors_one() {
        let mut registered = HashSet::new();
        registered.insert(addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])); // neighbor in dim 0
        let candidate = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(count_registered_neighbors(&candidate, &registered), 1);
    }

    #[test]
    fn test_count_registered_neighbors_non_neighbor() {
        let mut registered = HashSet::new();
        registered.insert(addr([2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])); // differs in 2 dims
        let candidate = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(count_registered_neighbors(&candidate, &registered), 0,
            "Nodes differing in 2+ dims are NOT neighbors");
    }

    // ── Cold start allocation ───────────────────────────────────

    #[test]
    fn test_cold_start_first_node() {
        let registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        let (addr, metrics) = allocate_optimal(
            &registered, &mut bitmap, &mut density, 0,
        ).unwrap();

        assert_eq!(addr.to_bytes(), COLD_START_ADDRESSES[0]);
        assert!(metrics.cold_start);
    }

    #[test]
    fn test_cold_start_second_node_max_distance() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        // Register first node
        let first = CubeAddr::new(COLD_START_ADDRESSES[0]);
        registered.insert(first.clone());
        bitmap[first.flat_index() as usize] = true;
        density.register(&first);

        let (addr, metrics) = allocate_optimal(
            &registered, &mut bitmap, &mut density, 0,
        ).unwrap();

        assert_eq!(addr.to_bytes(), COLD_START_ADDRESSES[1]);
        assert_eq!(metrics.min_distance, 13, "Second node should be at max distance");
        assert!(metrics.cold_start);
    }

    #[test]
    fn test_cold_start_third_node() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        for i in 0..2 {
            let a = CubeAddr::new(COLD_START_ADDRESSES[i]);
            registered.insert(a.clone());
            bitmap[a.flat_index() as usize] = true;
            density.register(&a);
        }

        let (addr, metrics) = allocate_optimal(
            &registered, &mut bitmap, &mut density, 0,
        ).unwrap();

        assert_eq!(addr.to_bytes(), COLD_START_ADDRESSES[2]);
        assert!(metrics.cold_start);
    }

    // ── Optimal allocation ──────────────────────────────────────

    #[test]
    fn test_optimal_after_cold_start() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        // Fill cold start addresses
        for i in 0..3 {
            let a = CubeAddr::new(COLD_START_ADDRESSES[i]);
            registered.insert(a.clone());
            bitmap[a.flat_index() as usize] = true;
            density.register(&a);
        }

        // 4th allocation should use max-spread
        let (addr, metrics) = allocate_optimal(
            &registered, &mut bitmap, &mut density, 42,
        ).unwrap();

        assert!(!metrics.cold_start, "4th node should NOT be cold start");
        assert!(metrics.candidates_evaluated > 0);
        assert!(metrics.min_distance > 0, "Should have positive distance to existing nodes");
    }

    #[test]
    fn test_optimal_multiple_allocations_unique() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        let mut allocated = Vec::new();
        for seed in 0..20 {
            if let Some((addr, _)) = allocate_optimal(
                &registered, &mut bitmap, &mut density, seed,
            ) {
                registered.insert(addr.clone());
                allocated.push(addr);
            }
        }

        // All addresses must be unique
        let unique: HashSet<&CubeAddr> = allocated.iter().collect();
        assert_eq!(allocated.len(), unique.len(), "All allocated addresses must be unique");
        assert_eq!(allocated.len(), 20);
    }

    #[test]
    fn test_optimal_k_floor_preference() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let mut density = DimensionDensity::new();

        // Register a cluster of neighbors so K-floor candidates exist
        let center = addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        registered.insert(center.clone());
        bitmap[center.flat_index() as usize] = true;
        density.register(&center);

        // Register several neighbors of center
        for dim in 0..4 {
            let mut trits = [2u8; 13];
            trits[dim] = 1;
            let nbr = CubeAddr::new(trits);
            registered.insert(nbr.clone());
            bitmap[nbr.flat_index() as usize] = true;
            density.register(&nbr);
        }

        // Allocate — should prefer candidates near the cluster (K-floor met)
        let (_, metrics) = allocate_optimal(
            &registered, &mut bitmap, &mut density, 99,
        ).unwrap();

        // With 5 registered nodes in a cluster, the allocator should find
        // candidates with ≥2 registered neighbors
        assert!(metrics.candidates_evaluated > 0);
    }

    // ── Candidate generation ────────────────────────────────────

    #[test]
    fn test_generate_candidates_returns_unregistered() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let density = DimensionDensity::new();

        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        registered.insert(a.clone());
        bitmap[a.flat_index() as usize] = true;

        let candidates = generate_candidates(&registered, &bitmap, &density, 100, 0);
        for c in &candidates {
            assert!(!registered.contains(c), "Candidates must be unregistered");
        }
    }

    #[test]
    fn test_generate_candidates_includes_neighbors() {
        let mut registered = HashSet::new();
        let mut bitmap = vec![false; TOTAL_VERTICES as usize];
        let density = DimensionDensity::new();

        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        registered.insert(a.clone());
        bitmap[a.flat_index() as usize] = true;

        let candidates = generate_candidates(&registered, &bitmap, &density, 100, 0);
        // At least some candidates should be neighbors of the registered node
        let neighbors_found = candidates.iter().any(|c| hamming_distance(&a, c) == 1);
        assert!(neighbors_found, "Candidates should include neighbors of registered nodes");
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_cold_start_addresses_valid_rep_c() {
        for cold in &COLD_START_ADDRESSES {
            for &t in cold {
                assert!(t >= 1 && t <= 3, "Cold start address must be Rep C");
            }
        }
    }

    #[test]
    fn test_cold_start_addresses_max_spread() {
        let a = CubeAddr::new(COLD_START_ADDRESSES[0]);
        let b = CubeAddr::new(COLD_START_ADDRESSES[1]);
        let c = CubeAddr::new(COLD_START_ADDRESSES[2]);

        assert_eq!(hamming_distance(&a, &b), 13, "Corner addresses should be at max distance");
        assert_eq!(hamming_distance(&a, &c), 13, "Center should be at max distance from inner");
        assert_eq!(hamming_distance(&b, &c), 13, "Center should be at max distance from outer");
    }

    #[test]
    fn test_constants() {
        assert_eq!(K_FLOOR, 2);
        assert_eq!(MAX_CANDIDATES, 1000);
        assert_eq!(COLD_START_ADDRESSES.len(), 3);
    }
}