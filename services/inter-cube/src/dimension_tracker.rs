// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Dimension Density Tracking (T-22, SPEC-2026-NEXT)
//!
//! Wraps T-16's `DimensionDensity` with CRS lifecycle integration,
//! API-ready queries, and enhanced tiebreaker logic for `allocateOptimal()`.
//!
//! ## Design
//!
//! The 13×3 density array is maintained as a first-class CRS field.
//! Every registration and deregistration updates the density. The
//! data is exposed via API endpoints for monitoring and the
//! `allocateOptimal` tiebreaker uses it to prefer sparse regions.
//!
//! ## Queries
//!
//! - **Per-dimension distribution**: How evenly are trits spread in each dim?
//! - **Imbalance score**: Single number measuring overall placement quality
//! - **Hotspots**: Dimensions where one trit value dominates (>60%)
//! - **Cold spots**: Dimension-value pairs with zero registrations
//! - **Allocation recommendation**: Which trit values should the next node prefer?
//!
//! ## Tiebreaker Logic
//!
//! When two candidate addresses have equal `(min_distance, avg_distance)`,
//! the density tiebreaker selects the candidate that fills the most
//! under-represented dimension-value combinations. This is already the
//! tertiary sort key in T-16's `allocate_optimal()` — T-22 provides
//! the richer analysis that feeds into it.

use crate::cube_addr::{CubeAddr, DIMENSIONS};
use crate::placement::{DimensionDensity, TRIT_VALUES};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Threshold for a dimension-value to be considered a "hotspot".
/// If one trit value holds > 60% of registrations in a dimension,
/// it's a hotspot — placement should avoid adding more to that value.
pub const HOTSPOT_THRESHOLD: f64 = 0.60;

/// Minimum registrations before density analysis is meaningful.
/// Below this, the network is too sparse for statistics.
pub const MIN_MEANINGFUL_REGISTRATIONS: u32 = 10;

// ═══════════════════════════════════════════════════════════════════════
// PER-DIMENSION DISTRIBUTION
// ═══════════════════════════════════════════════════════════════════════

/// Distribution of trit values within a single dimension.
#[derive(Debug, Clone)]
pub struct DimensionDistribution {
    /// Dimension index (0–12).
    pub dimension: usize,
    /// Count for trit value 1.
    pub count_1: u32,
    /// Count for trit value 2.
    pub count_2: u32,
    /// Count for trit value 3.
    pub count_3: u32,
    /// Total registrations (should equal count_1 + count_2 + count_3).
    pub total: u32,
    /// Entropy (0.0 = all same value, 1.585 = perfectly balanced).
    /// Uses log₃ so max entropy = 1.0 for 3 equiprobable values.
    pub entropy: f64,
    /// Whether any value exceeds HOTSPOT_THRESHOLD.
    pub has_hotspot: bool,
    /// The least-populated trit value (1, 2, or 3).
    pub least_populated: u8,
    /// The most-populated trit value.
    pub most_populated: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// HOTSPOT / COLD SPOT DETECTION
// ═══════════════════════════════════════════════════════════════════════

/// A detected density hotspot.
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// Dimension with the imbalance.
    pub dimension: usize,
    /// The over-represented trit value.
    pub trit_value: u8,
    /// What fraction of the dimension this value holds (0.0–1.0).
    pub fraction: f64,
    /// Count of registrations for this value.
    pub count: u32,
}

/// A dimension-value pair with zero registrations.
#[derive(Debug, Clone)]
pub struct ColdSpot {
    /// Dimension.
    pub dimension: usize,
    /// Trit value with zero registrations.
    pub trit_value: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// OVERALL METRICS
// ═══════════════════════════════════════════════════════════════════════

/// Aggregate density metrics for the entire 13D space.
#[derive(Debug, Clone)]
pub struct DensityMetrics {
    /// Total registered nodes.
    pub total_registrations: u32,
    /// Per-dimension distributions (13 entries).
    pub dimensions: Vec<DimensionDistribution>,
    /// Average entropy across all dimensions (0.0–1.0).
    pub avg_entropy: f64,
    /// Dimensions with hotspots.
    pub hotspots: Vec<Hotspot>,
    /// Dimension-value pairs with zero registrations.
    pub cold_spots: Vec<ColdSpot>,
    /// Global imbalance score (lower = more balanced).
    /// Sum of per-dimension max-min spread.
    pub global_imbalance: u32,
    /// Whether the density is meaningful (>= MIN_MEANINGFUL_REGISTRATIONS).
    pub is_meaningful: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// DIMENSION TRACKER — The CRS integration layer
// ═══════════════════════════════════════════════════════════════════════

/// CRS-integrated dimension density tracker.
///
/// Wraps `DimensionDensity` (T-16) with lifecycle hooks, rich queries,
/// and the API surface for the `/topology/density` endpoint.
pub struct DimensionTracker {
    /// The underlying 13×3 density array.
    density: DimensionDensity,
    /// Running history of imbalance scores (for trend analysis).
    imbalance_history: Vec<(u32, u32)>, // (registration_count, imbalance)
    /// Maximum history entries.
    max_history: usize,
}

impl DimensionTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        DimensionTracker {
            density: DimensionDensity::new(),
            imbalance_history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Get a reference to the underlying density (for T-16 allocateOptimal).
    pub fn density(&self) -> &DimensionDensity {
        &self.density
    }

    /// Get a mutable reference (for T-16 allocateOptimal).
    pub fn density_mut(&mut self) -> &mut DimensionDensity {
        &mut self.density
    }

    // ═══════════════════════════════════════════════════════════════
    // LIFECYCLE HOOKS — Called by CRS on registration/deregistration
    // ═══════════════════════════════════════════════════════════════

    /// Record a new registration.
    pub fn on_register(&mut self, addr: &CubeAddr) {
        self.density.register(addr);
        self.record_imbalance();
    }

    /// Record a deregistration.
    pub fn on_deregister(&mut self, addr: &CubeAddr) {
        self.density.deregister(addr);
        self.record_imbalance();
    }

    /// Record the current imbalance for trend tracking.
    fn record_imbalance(&mut self) {
        let imbalance = self.compute_global_imbalance();
        let total = self.density.total();
        self.imbalance_history.push((total, imbalance));
        if self.imbalance_history.len() > self.max_history {
            self.imbalance_history.remove(0);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // QUERIES
    // ═══════════════════════════════════════════════════════════════

    /// Compute the full density metrics.
    ///
    /// This is the primary query — returns everything the API needs.
    pub fn compute_metrics(&self) -> DensityMetrics {
        let total = self.density.total();
        let is_meaningful = total >= MIN_MEANINGFUL_REGISTRATIONS;

        let mut dimensions = Vec::with_capacity(DIMENSIONS);
        let mut hotspots = Vec::new();
        let mut cold_spots = Vec::new();
        let mut entropy_sum = 0.0f64;

        for dim in 0..DIMENSIONS {
            let c1 = self.density.count(dim, 1);
            let c2 = self.density.count(dim, 2);
            let c3 = self.density.count(dim, 3);
            let dim_total = c1 + c2 + c3;

            let entropy = if dim_total > 0 {
                compute_entropy_log3(c1, c2, c3, dim_total)
            } else {
                0.0
            };
            entropy_sum += entropy;

            let counts = [c1, c2, c3];
            let _min_c = *counts.iter().min().unwrap();
            let max_c = *counts.iter().max().unwrap();

            let least_pop = if c1 <= c2 && c1 <= c3 {
                1
            } else if c2 <= c3 {
                2
            } else {
                3
            };

            let most_pop = if c1 >= c2 && c1 >= c3 {
                1
            } else if c2 >= c3 {
                2
            } else {
                3
            };

            let has_hotspot = dim_total > 0
                && (max_c as f64 / dim_total as f64) > HOTSPOT_THRESHOLD;

            // Detect hotspots
            if has_hotspot && is_meaningful {
                for (val_idx, &count) in counts.iter().enumerate() {
                    let fraction = count as f64 / dim_total as f64;
                    if fraction > HOTSPOT_THRESHOLD {
                        hotspots.push(Hotspot {
                            dimension: dim,
                            trit_value: (val_idx + 1) as u8,
                            fraction,
                            count,
                        });
                    }
                }
            }

            // Detect cold spots
            for (val_idx, &count) in counts.iter().enumerate() {
                if count == 0 && dim_total > 0 {
                    cold_spots.push(ColdSpot {
                        dimension: dim,
                        trit_value: (val_idx + 1) as u8,
                    });
                }
            }

            dimensions.push(DimensionDistribution {
                dimension: dim,
                count_1: c1,
                count_2: c2,
                count_3: c3,
                total: dim_total,
                entropy,
                has_hotspot,
                least_populated: least_pop,
                most_populated: most_pop,
            });
        }

        let avg_entropy = if DIMENSIONS > 0 {
            entropy_sum / DIMENSIONS as f64
        } else {
            0.0
        };

        let global_imbalance = self.compute_global_imbalance();

        DensityMetrics {
            total_registrations: total,
            dimensions,
            avg_entropy,
            hotspots,
            cold_spots,
            global_imbalance,
            is_meaningful,
        }
    }

    /// Compute the per-dimension distribution for a single dimension.
    pub fn dimension_distribution(&self, dim: usize) -> Option<DimensionDistribution> {
        if dim >= DIMENSIONS {
            return None;
        }
        let metrics = self.compute_metrics();
        metrics.dimensions.into_iter().nth(dim)
    }

    /// Get the recommended trit values for the next allocation.
    ///
    /// Returns a 13-element array where each element is the trit value
    /// (1, 2, or 3) that would most balance that dimension.
    pub fn recommended_values(&self) -> [u8; DIMENSIONS] {
        let mut rec = [0u8; DIMENSIONS];
        for dim in 0..DIMENSIONS {
            rec[dim] = self.density.least_populated_value(dim);
        }
        rec
    }

    /// Compute the imbalance score for a specific candidate address.
    ///
    /// Lower score = candidate fills under-populated regions.
    /// This is the tiebreaker for T-16's `allocate_optimal()`.
    pub fn tiebreaker_score(&self, candidate: &CubeAddr) -> u32 {
        self.density.imbalance_score(candidate)
    }

    /// Compute the global imbalance: sum of (max - min) across all dimensions.
    ///
    /// A perfectly balanced network has imbalance = 0.
    /// Maximum imbalance = total_registrations × 13 (all in one value per dim).
    pub fn compute_global_imbalance(&self) -> u32 {
        let mut imbalance = 0u32;
        for dim in 0..DIMENSIONS {
            let c1 = self.density.count(dim, 1);
            let c2 = self.density.count(dim, 2);
            let c3 = self.density.count(dim, 3);
            let max_c = c1.max(c2).max(c3);
            let min_c = c1.min(c2).min(c3);
            imbalance += max_c - min_c;
        }
        imbalance
    }

    /// Get the imbalance history for trend analysis.
    pub fn imbalance_history(&self) -> &[(u32, u32)] {
        &self.imbalance_history
    }

    /// Total registrations tracked.
    pub fn total(&self) -> u32 {
        self.density.total()
    }

    /// The raw 13×3 array.
    pub fn raw_array(&self) -> &[[u32; TRIT_VALUES]; DIMENSIONS] {
        self.density.as_array()
    }
}

impl Default for DimensionTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ENTROPY COMPUTATION
// ═══════════════════════════════════════════════════════════════════════

/// Compute entropy in base-3 for a 3-value distribution.
///
/// Returns a value in [0.0, 1.0]:
/// - 0.0 = all registrations on one value
/// - 1.0 = perfectly uniform (1/3 each)
///
/// Uses: H₃ = -Σ p_i × log₃(p_i)
fn compute_entropy_log3(c1: u32, c2: u32, c3: u32, total: u32) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let t = total as f64;
    let log3 = 3.0f64.ln();
    let mut entropy = 0.0f64;

    for &c in &[c1, c2, c3] {
        if c > 0 {
            let p = c as f64 / t;
            entropy -= p * (p.ln() / log3);
        }
    }
    entropy
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

    // ── Basic lifecycle ─────────────────────────────────────────

    #[test]
    fn test_empty_tracker() {
        let tracker = DimensionTracker::new();
        assert_eq!(tracker.total(), 0);
        let metrics = tracker.compute_metrics();
        assert_eq!(metrics.total_registrations, 0);
        assert!(!metrics.is_meaningful);
        assert!(metrics.hotspots.is_empty());
    }

    #[test]
    fn test_register_updates_density() {
        let mut tracker = DimensionTracker::new();
        tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        assert_eq!(tracker.total(), 1);

        let raw = tracker.raw_array();
        for dim in 0..DIMENSIONS {
            assert_eq!(raw[dim][0], 1, "Dim {} val 1 should be 1", dim);
            assert_eq!(raw[dim][1], 0);
            assert_eq!(raw[dim][2], 0);
        }
    }

    #[test]
    fn test_deregister_updates_density() {
        let mut tracker = DimensionTracker::new();
        let a = addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        tracker.on_register(&a);
        assert_eq!(tracker.total(), 1);

        tracker.on_deregister(&a);
        assert_eq!(tracker.total(), 0);
    }

    // ── Entropy ─────────────────────────────────────────────────

    #[test]
    fn test_entropy_uniform() {
        // Perfectly balanced: entropy should be ~1.0
        let e = compute_entropy_log3(100, 100, 100, 300);
        assert!((e - 1.0).abs() < 0.001, "Uniform distribution should have entropy ~1.0");
    }

    #[test]
    fn test_entropy_concentrated() {
        // All on one value: entropy = 0
        let e = compute_entropy_log3(300, 0, 0, 300);
        assert!(e.abs() < 0.001, "Concentrated distribution should have entropy ~0.0");
    }

    #[test]
    fn test_entropy_partial() {
        // 200, 100, 0 — some imbalance
        let e = compute_entropy_log3(200, 100, 0, 300);
        assert!(e > 0.0 && e < 1.0, "Partial distribution entropy should be in (0, 1)");
    }

    // ── Hotspot detection ───────────────────────────────────────

    #[test]
    fn test_hotspot_detection() {
        let mut tracker = DimensionTracker::new();
        // Register 20 nodes all with val=1 in dim 0
        for i in 0..20 {
            let mut trits = [1u8; 13];
            trits[1] = ((i % 3) + 1) as u8; // Vary other dims
            tracker.on_register(&CubeAddr::new(trits));
        }

        let metrics = tracker.compute_metrics();
        assert!(metrics.is_meaningful);

        // Dim 0 should be a hotspot (all nodes have val=1)
        let dim0_hotspots: Vec<_> = metrics.hotspots.iter()
            .filter(|h| h.dimension == 0)
            .collect();
        assert!(!dim0_hotspots.is_empty(), "Dim 0 should have a hotspot");
        assert_eq!(dim0_hotspots[0].trit_value, 1);
    }

    // ── Cold spot detection ─────────────────────────────────────

    #[test]
    fn test_cold_spot_detection() {
        let mut tracker = DimensionTracker::new();
        // Register only val=1 and val=2 in all dims — val=3 is cold everywhere
        for _ in 0..5 {
            tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
            tracker.on_register(&addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]));
        }

        let metrics = tracker.compute_metrics();
        let val3_cold: Vec<_> = metrics.cold_spots.iter()
            .filter(|c| c.trit_value == 3)
            .collect();
        assert_eq!(val3_cold.len(), 13, "All 13 dimensions should have val=3 as cold spot");
    }

    // ── Recommendations ─────────────────────────────────────────

    #[test]
    fn test_recommended_values() {
        let mut tracker = DimensionTracker::new();
        // Heavy registration of val=1
        for _ in 0..10 {
            tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        }

        let rec = tracker.recommended_values();
        // Val 2 or 3 should be recommended (both have 0 registrations)
        for dim in 0..DIMENSIONS {
            assert_ne!(rec[dim], 1,
                "Dim {} should NOT recommend val=1 (most populated)", dim);
        }
    }

    // ── Tiebreaker score ────────────────────────────────────────

    #[test]
    fn test_tiebreaker_prefers_sparse() {
        let mut tracker = DimensionTracker::new();
        for _ in 0..10 {
            tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        }

        let dense = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let sparse = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);

        let score_dense = tracker.tiebreaker_score(&dense);
        let score_sparse = tracker.tiebreaker_score(&sparse);

        assert!(score_sparse < score_dense,
            "Sparse candidate should have lower (better) tiebreaker score");
    }

    // ── Global imbalance ────────────────────────────────────────

    #[test]
    fn test_global_imbalance_balanced() {
        let mut tracker = DimensionTracker::new();
        // Register one of each value — perfectly balanced
        tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        tracker.on_register(&addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]));
        tracker.on_register(&addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]));

        assert_eq!(tracker.compute_global_imbalance(), 0,
            "1 of each value per dim = zero imbalance");
    }

    #[test]
    fn test_global_imbalance_unbalanced() {
        let mut tracker = DimensionTracker::new();
        for _ in 0..10 {
            tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        }

        let imbalance = tracker.compute_global_imbalance();
        // Each dim: max=10, min=0, spread=10. 13 dims × 10 = 130
        assert_eq!(imbalance, 130);
    }

    // ── History tracking ────────────────────────────────────────

    #[test]
    fn test_imbalance_history() {
        let mut tracker = DimensionTracker::new();
        tracker.on_register(&addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        tracker.on_register(&addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]));

        let history = tracker.imbalance_history();
        assert_eq!(history.len(), 2);
        // First entry: 1 registration, imbalance = 13 (max-min=1 per dim)
        assert_eq!(history[0].0, 1); // total after first reg
        assert_eq!(history[0].1, 13);
        // Second entry: 2 registrations, imbalance = 13 (val3 still empty)
        assert_eq!(history[1].0, 2);
        assert_eq!(history[1].1, 13);
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert!((HOTSPOT_THRESHOLD - 0.60).abs() < 0.001);
        assert_eq!(MIN_MEANINGFUL_REGISTRATIONS, 10);
    }
}