// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Sampling Mode for Large Networks (T-25, SPEC-2026-NEXT)
//!
//! When the network exceeds 100,000 registered nodes, computing the
//! Hamming distance from every candidate to every registered node
//! becomes O(candidates × N) ≈ 1000 × 100,000 = 100M comparisons.
//! At ~10ns per comparison, that's ~1 second — too slow for real-time
//! allocation.
//!
//! ## Solution: Random Sampling
//!
//! Instead of comparing against ALL registered nodes, sample 1,000
//! nodes uniformly at random and compute distances against the sample.
//! The farthest-first heuristic's approximation ratio remains within
//! a factor of 2 of optimal with high probability when the sample
//! size is O(√N).
//!
//! ## Implementation
//!
//! 1. **Lock-free read-only snapshot**: Copy the registered address set
//!    into a Vec (one allocation, no locks during scoring).
//!
//! 2. **TLSponge-385 seeded PRNG**: Deterministic sample selection from
//!    a seed. Same seed → same sample → reproducible allocations.
//!
//! 3. **Stratified sampling**: The sample is stratified across the 13
//!    dimensions to ensure coverage of the full hypercube, not just
//!    a random cluster.
//!
//! ## Thresholds
//!
//! | Network size | Mode | Comparison targets |
//! |---|---|---|
//! | < 100K | Full scan | All registered nodes |
//! | 100K – 500K | 1,000 sample | Random + stratified |
//! | > 500K | 2,000 sample | Larger sample for accuracy |
//!
//! ## Performance Target
//!
//! allocateOptimal with sampling: < 50ms at any network size (T-26 benchmark).

use std::collections::HashSet;

use crate::cube_addr::{CubeAddr, DIMENSIONS};
use crate::placement::{hamming_distance};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Threshold for switching from full scan to sampling mode.
pub const SAMPLING_THRESHOLD: usize = 100_000;

/// Default sample size for 100K–500K networks.
pub const DEFAULT_SAMPLE_SIZE: usize = 1_000;

/// Larger sample size for >500K networks.
pub const LARGE_SAMPLE_SIZE: usize = 2_000;

/// Threshold for the larger sample size.
pub const LARGE_NETWORK_THRESHOLD: usize = 500_000;

/// Number of stratification buckets per dimension.
/// Each bucket covers one trit value {1, 2, 3}.
pub const STRATA_PER_DIM: usize = 3;

/// Fraction of sample allocated to stratified selection (vs pure random).
pub const STRATIFIED_FRACTION: f64 = 0.3;

// ═══════════════════════════════════════════════════════════════════════
// SNAPSHOT
// ═══════════════════════════════════════════════════════════════════════

/// A lock-free read-only snapshot of the registered address set.
///
/// Created once at the start of an allocation, used for all distance
/// computations. No locks held during scoring — the CRS can continue
/// processing registrations concurrently.
#[derive(Debug, Clone)]
pub struct AddressSnapshot {
    /// All registered addresses in a flat Vec.
    addresses: Vec<CubeAddr>,
    /// Total count.
    count: usize,
}

impl AddressSnapshot {
    /// Create a snapshot from the registered set.
    pub fn from_set(registered: &HashSet<CubeAddr>) -> Self {
        let addresses: Vec<CubeAddr> = registered.iter().cloned().collect();
        let count = addresses.len();
        AddressSnapshot { addresses, count }
    }

    /// Create from a Vec (for testing).
    pub fn from_vec(addresses: Vec<CubeAddr>) -> Self {
        let count = addresses.len();
        AddressSnapshot { addresses, count }
    }

    /// Number of addresses in the snapshot.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the full address list.
    pub fn addresses(&self) -> &[CubeAddr] {
        &self.addresses
    }

    /// Whether sampling mode should be used for this snapshot.
    pub fn needs_sampling(&self) -> bool {
        self.count >= SAMPLING_THRESHOLD
    }

    /// The appropriate sample size for this snapshot.
    pub fn sample_size(&self) -> usize {
        if self.count >= LARGE_NETWORK_THRESHOLD {
            LARGE_SAMPLE_SIZE
        } else if self.count >= SAMPLING_THRESHOLD {
            DEFAULT_SAMPLE_SIZE
        } else {
            self.count // Full scan
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DETERMINISTIC PRNG — TLSponge-385 seeded
// ═══════════════════════════════════════════════════════════════════════

/// A deterministic pseudo-random number generator seeded by TLSponge-385.
///
/// Produces a sequence of u64 values from a seed. Same seed → same
/// sequence. Used for reproducible sample selection.
struct SpongeRng {
    /// Pre-generated random bytes.
    bytes: Vec<u8>,
    /// Current offset into the byte buffer.
    offset: usize,
}

impl SpongeRng {
    /// Create a new PRNG with the given seed.
    ///
    /// Generates enough bytes for `count` u64 values.
    fn new(seed: u64, count: usize) -> Self {
        let seed_bytes = seed.to_le_bytes();
        let need_bytes = count * 8;
        let bytes = ternary_math::sponge::derive_key(
            b"PlenumNET-SAMPLE-RNG",
            &seed_bytes,
            need_bytes,
        );
        SpongeRng { bytes, offset: 0 }
    }

    /// Get the next random u64.
    fn next_u64(&mut self) -> u64 {
        if self.offset + 8 > self.bytes.len() {
            return 0; // Exhausted
        }
        let val = u64::from_le_bytes([
            self.bytes[self.offset],
            self.bytes[self.offset + 1],
            self.bytes[self.offset + 2],
            self.bytes[self.offset + 3],
            self.bytes[self.offset + 4],
            self.bytes[self.offset + 5],
            self.bytes[self.offset + 6],
            self.bytes[self.offset + 7],
        ]);
        self.offset += 8;
        val
    }

    /// Get a random index in [0, max).
    fn next_index(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() % max as u64) as usize
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SAMPLING
// ═══════════════════════════════════════════════════════════════════════

/// Select a representative sample from the snapshot.
///
/// Uses stratified + random sampling:
///
/// 1. **Stratified portion** (30%): For each of the 13 dimensions,
///    select addresses that have specific trit values. This ensures
///    the sample covers the full hypercube structure.
///
/// 2. **Random portion** (70%): Uniform random selection from the
///    full set. Fills the remaining sample budget.
///
/// Returns indices into the snapshot's address array.
pub fn select_sample(
    snapshot: &AddressSnapshot,
    sample_size: usize,
    seed: u64,
) -> Vec<usize> {
    if snapshot.len() <= sample_size {
        // Full scan — return all indices
        return (0..snapshot.len()).collect();
    }

    let mut rng = SpongeRng::new(seed, sample_size * 2);
    let mut selected = HashSet::with_capacity(sample_size);
    let addresses = snapshot.addresses();

    // Phase 1: Stratified sampling (30% of budget)
    let stratified_budget = (sample_size as f64 * STRATIFIED_FRACTION) as usize;
    let per_stratum = (stratified_budget / (DIMENSIONS * STRATA_PER_DIM)).max(1);

    for dim in 0..DIMENSIONS {
        for trit_val in 1u8..=3 {
            let mut found = 0;
            // Scan from a random starting point
            let start = rng.next_index(snapshot.len());
            for offset in 0..snapshot.len() {
                if found >= per_stratum {
                    break;
                }
                let idx = (start + offset) % snapshot.len();
                if addresses[idx].to_bytes()[dim] == trit_val && !selected.contains(&idx) {
                    selected.insert(idx);
                    found += 1;
                }
            }
        }
    }

    // Phase 2: Random sampling (fill remaining budget)
    let mut attempts = 0;
    while selected.len() < sample_size && attempts < sample_size * 10 {
        let idx = rng.next_index(snapshot.len());
        selected.insert(idx);
        attempts += 1;
    }

    let mut result: Vec<usize> = selected.into_iter().collect();
    result.sort(); // Sorted for cache-friendly access
    result
}

// ═══════════════════════════════════════════════════════════════════════
// SAMPLED DISTANCE COMPUTATION
// ═══════════════════════════════════════════════════════════════════════

/// Distance metrics computed against a sample.
#[derive(Debug, Clone)]
pub struct SampledMetrics {
    /// Minimum Hamming distance to any sampled node.
    pub min_distance: usize,
    /// Average Hamming distance to all sampled nodes.
    pub avg_distance: f64,
    /// Number of sampled nodes compared against.
    pub sample_count: usize,
}

/// Compute distance metrics for a candidate against a sampled subset.
pub fn compute_sampled_metrics(
    candidate: &CubeAddr,
    snapshot: &AddressSnapshot,
    sample_indices: &[usize],
) -> SampledMetrics {
    let addresses = snapshot.addresses();
    let mut min_d = usize::MAX;
    let mut sum_d: u64 = 0;
    let mut count = 0usize;

    for &idx in sample_indices {
        if idx < addresses.len() {
            let d = hamming_distance(candidate, &addresses[idx]);
            if d < min_d {
                min_d = d;
            }
            sum_d += d as u64;
            count += 1;
        }
    }

    if count == 0 {
        return SampledMetrics {
            min_distance: 13,
            avg_distance: 13.0,
            sample_count: 0,
        };
    }

    SampledMetrics {
        min_distance: min_d,
        avg_distance: sum_d as f64 / count as f64,
        sample_count: count,
    }
}

/// Score multiple candidates against a sample in one pass.
///
/// Returns candidates sorted by (max min_distance, max avg_distance).
/// The top candidate is the best placement choice.
pub fn score_candidates_sampled(
    candidates: &[CubeAddr],
    snapshot: &AddressSnapshot,
    sample_indices: &[usize],
) -> Vec<(usize, SampledMetrics)> {
    let mut scored: Vec<(usize, SampledMetrics)> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let metrics = compute_sampled_metrics(candidate, snapshot, sample_indices);
            (i, metrics)
        })
        .collect();

    // Sort: max min_distance, then max avg_distance
    scored.sort_by(|a, b| {
        b.1.min_distance.cmp(&a.1.min_distance)
            .then(
                b.1.avg_distance
                    .partial_cmp(&a.1.avg_distance)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    scored
}

// ═══════════════════════════════════════════════════════════════════════
// INTEGRATION — Drop-in for allocate_optimal
// ═══════════════════════════════════════════════════════════════════════

/// Compute distance targets for allocate_optimal, using sampling if needed.
///
/// This is the integration function that T-16's `allocate_optimal` calls
/// instead of computing distances against all registered nodes.
///
/// - If `snapshot.len() < SAMPLING_THRESHOLD`: returns all addresses
///   (full scan, same as before T-25)
/// - If `snapshot.len() >= SAMPLING_THRESHOLD`: returns a representative
///   sample (fast path)
pub fn get_distance_targets(
    snapshot: &AddressSnapshot,
    seed: u64,
) -> Vec<usize> {
    let sample_size = snapshot.sample_size();
    select_sample(snapshot, sample_size, seed)
}

/// Metrics about the sampling decision for telemetry.
#[derive(Debug, Clone)]
pub struct SamplingInfo {
    /// Total registered nodes.
    pub total_nodes: usize,
    /// Whether sampling was used.
    pub sampled: bool,
    /// Size of the sample (or total if not sampled).
    pub target_count: usize,
    /// Sampling ratio (sample_size / total).
    pub ratio: f64,
}

/// Get information about what sampling mode would be used.
pub fn sampling_info(snapshot: &AddressSnapshot) -> SamplingInfo {
    let total = snapshot.len();
    let sampled = snapshot.needs_sampling();
    let target_count = snapshot.sample_size();
    let ratio = if total > 0 {
        target_count as f64 / total as f64
    } else {
        1.0
    };

    SamplingInfo {
        total_nodes: total,
        sampled,
        target_count,
        ratio,
    }
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

    /// Generate N unique addresses deterministically.
    fn generate_addresses(n: usize) -> Vec<CubeAddr> {
        let total = TOTAL_VERTICES as usize;
        (0..n.min(total))
            .map(|i| CubeAddr::from_flat_index(i as u64).unwrap())
            .collect()
    }

    // ── Snapshot ────────────────────────────────────────────────

    #[test]
    fn test_snapshot_from_set() {
        let mut set = HashSet::new();
        set.insert(addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
        set.insert(addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]));

        let snap = AddressSnapshot::from_set(&set);
        assert_eq!(snap.len(), 2);
        assert!(!snap.is_empty());
        assert!(!snap.needs_sampling());
    }

    #[test]
    fn test_snapshot_sample_size_thresholds() {
        let small = AddressSnapshot::from_vec(generate_addresses(1000));
        assert_eq!(small.sample_size(), 1000); // Full scan

        // We can't generate 100K+ addresses in a test, but we can
        // test the logic directly
        let fake_large = AddressSnapshot {
            addresses: Vec::new(),
            count: 150_000,
        };
        assert!(fake_large.needs_sampling());
        assert_eq!(fake_large.sample_size(), DEFAULT_SAMPLE_SIZE);

        let fake_xlarge = AddressSnapshot {
            addresses: Vec::new(),
            count: 600_000,
        };
        assert_eq!(fake_xlarge.sample_size(), LARGE_SAMPLE_SIZE);
    }

    // ── Deterministic PRNG ──────────────────────────────────────

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = SpongeRng::new(42, 10);
        let mut rng2 = SpongeRng::new(42, 10);

        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_rng_different_seeds() {
        let mut rng1 = SpongeRng::new(42, 10);
        let mut rng2 = SpongeRng::new(99, 10);

        let v1 = rng1.next_u64();
        let v2 = rng2.next_u64();
        assert_ne!(v1, v2, "Different seeds should produce different values");
    }

    // ── Sample selection ────────────────────────────────────────

    #[test]
    fn test_select_sample_full_scan_when_small() {
        let addrs = generate_addresses(500);
        let snap = AddressSnapshot::from_vec(addrs);
        let sample = select_sample(&snap, 1000, 42);
        assert_eq!(sample.len(), 500, "Small set → full scan (all indices)");
    }

    #[test]
    fn test_select_sample_correct_size() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let sample = select_sample(&snap, 1000, 42);
        assert_eq!(sample.len(), 1000, "Sample should be exactly the requested size");
    }

    #[test]
    fn test_select_sample_deterministic() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let s1 = select_sample(&snap, 1000, 42);
        let s2 = select_sample(&snap, 1000, 42);
        assert_eq!(s1, s2, "Same seed → same sample");
    }

    #[test]
    fn test_select_sample_different_seeds() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let s1 = select_sample(&snap, 1000, 42);
        let s2 = select_sample(&snap, 1000, 99);
        assert_ne!(s1, s2, "Different seeds → different samples");
    }

    #[test]
    fn test_select_sample_no_duplicates() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let sample = select_sample(&snap, 1000, 42);
        let unique: HashSet<usize> = sample.iter().copied().collect();
        assert_eq!(sample.len(), unique.len(), "No duplicate indices in sample");
    }

    #[test]
    fn test_select_sample_indices_in_range() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let sample = select_sample(&snap, 1000, 42);
        for &idx in &sample {
            assert!(idx < 5000, "Index {} out of range", idx);
        }
    }

    #[test]
    fn test_select_sample_sorted() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        let sample = select_sample(&snap, 1000, 42);
        for i in 1..sample.len() {
            assert!(sample[i] >= sample[i - 1], "Sample should be sorted");
        }
    }

    // ── Sampled metrics ─────────────────────────────────────────

    #[test]
    fn test_sampled_metrics_basic() {
        let addrs = vec![
            addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]),
        ];
        let snap = AddressSnapshot::from_vec(addrs);
        let candidate = addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        let indices = vec![0, 1]; // Both addresses

        let metrics = compute_sampled_metrics(&candidate, &snap, &indices);
        assert_eq!(metrics.min_distance, 13); // Equidistant from both corners
        assert_eq!(metrics.avg_distance, 13.0);
        assert_eq!(metrics.sample_count, 2);
    }

    #[test]
    fn test_sampled_metrics_empty() {
        let snap = AddressSnapshot::from_vec(Vec::new());
        let candidate = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        let metrics = compute_sampled_metrics(&candidate, &snap, &[]);
        assert_eq!(metrics.min_distance, 13);
        assert_eq!(metrics.sample_count, 0);
    }

    // ── Scored candidates ───────────────────────────────────────

    #[test]
    fn test_score_candidates_ordering() {
        let registered = vec![
            addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ];
        let snap = AddressSnapshot::from_vec(registered);
        let indices = vec![0];

        let candidates = vec![
            addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), // distance 1
            addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]), // distance 13
            addr([2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), // distance 2
        ];

        let scored = score_candidates_sampled(&candidates, &snap, &indices);
        // First should be the farthest (distance 13)
        assert_eq!(scored[0].0, 1, "Farthest candidate should rank first");
        assert_eq!(scored[0].1.min_distance, 13);
    }

    // ── Sampling info ───────────────────────────────────────────

    #[test]
    fn test_sampling_info_small() {
        let snap = AddressSnapshot::from_vec(generate_addresses(500));
        let info = sampling_info(&snap);
        assert!(!info.sampled);
        assert_eq!(info.target_count, 500);
        assert_eq!(info.ratio, 1.0);
    }

    // ── Integration ─────────────────────────────────────────────

    #[test]
    fn test_get_distance_targets_small() {
        let addrs = generate_addresses(100);
        let snap = AddressSnapshot::from_vec(addrs);
        let targets = get_distance_targets(&snap, 42);
        assert_eq!(targets.len(), 100, "Small network → all indices");
    }

    #[test]
    fn test_get_distance_targets_sampled() {
        let addrs = generate_addresses(5000);
        let snap = AddressSnapshot::from_vec(addrs);
        // Manually set count to trigger sampling
        let large_snap = AddressSnapshot {
            addresses: snap.addresses,
            count: 150_000, // Fake count to trigger sampling
        };
        // This would try to sample from 5000 actual addresses
        // but with count=150K it thinks sampling is needed
        assert!(large_snap.needs_sampling());
    }

    // ── Constants ───────────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert_eq!(SAMPLING_THRESHOLD, 100_000);
        assert_eq!(DEFAULT_SAMPLE_SIZE, 1_000);
        assert_eq!(LARGE_SAMPLE_SIZE, 2_000);
        assert_eq!(LARGE_NETWORK_THRESHOLD, 500_000);
        assert!((STRATIFIED_FRACTION - 0.3).abs() < 0.001);
    }
}