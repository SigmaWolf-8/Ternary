// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # PT26-DSA Walk Construction
//!
//! Builds secret walks through the 13D ternary hypercube using
//! the σ permutation schedule. The walk is the core structure
//! that the signer knows and the verifier checks.

use crate::cube_addr::CubeAddr;
use crate::plenum_square::{SIGMAS, WEIGHT_VECTOR, MAGIC_CONSTANT};
use crate::pt26_dsa::{
    DIMENSIONS, MAX_WALK_LENGTH, NUM_SIGMAS,
    SecretSchedule, STEP_COMMIT_LEN,
    compute_step_commit, hamming_distance,
};

// ═══════════════════════════════════════════════════════════════════════
// WALK STEP
// ═══════════════════════════════════════════════════════════════════════

/// A single step in a hypercube walk.
#[derive(Debug, Clone)]
pub struct WalkStep {
    /// Vertex before this step.
    pub from: CubeAddr,
    /// Vertex after this step.
    pub to: CubeAddr,
    /// Which dimension was fixed.
    pub dimension: usize,
    /// Which σ permutation was used.
    pub sigma_index: u8,
    /// Weight from the Plenum Square for this step.
    pub weight: u32,
    /// Step position in the walk (0-indexed).
    pub position: usize,
    /// The step commitment.
    pub commitment: [u8; STEP_COMMIT_LEN],
}

// ═══════════════════════════════════════════════════════════════════════
// WALK — Complete path through the hypercube
// ═══════════════════════════════════════════════════════════════════════

/// A complete walk from source to destination.
#[derive(Debug, Clone)]
pub struct Walk {
    /// Source vertex (signer's address).
    pub source: CubeAddr,
    /// Destination vertex (message-derived).
    pub destination: CubeAddr,
    /// The individual steps.
    pub steps: Vec<WalkStep>,
    /// Walk checksum mod 333.
    pub checksum: u32,
}

impl Walk {
    /// Walk length (= Hamming distance between source and destination).
    pub fn length(&self) -> usize {
        self.steps.len()
    }

    /// Total weight (sum of step weights, unreduced).
    pub fn total_weight(&self) -> u64 {
        self.steps.iter().map(|s| s.weight as u64).sum()
    }

    /// Get the dimension ordering used in this walk.
    pub fn dimension_order(&self) -> Vec<usize> {
        self.steps.iter().map(|s| s.dimension).collect()
    }

    /// Get the σ schedule used in this walk.
    pub fn sigma_schedule(&self) -> Vec<u8> {
        self.steps.iter().map(|s| s.sigma_index).collect()
    }

    /// Extract step commitments as a flat array.
    pub fn step_commitments(&self) -> Vec<[u8; STEP_COMMIT_LEN]> {
        self.steps.iter().map(|s| s.commitment).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WALK BUILDER
// ═══════════════════════════════════════════════════════════════════════

/// Construct a secret walk using the σ schedule.
///
/// This is the core signing operation: given a secret schedule,
/// build the walk from source to destination.
pub fn build_walk(
    source: &CubeAddr,
    destination: &CubeAddr,
    schedule: &SecretSchedule,
) -> Walk {
    let src_bytes = source.to_bytes();
    let dst_bytes = destination.to_bytes();
    let h = hamming_distance(source, destination);

    let mut dims_remaining: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| src_bytes[d] != dst_bytes[d])
        .collect();

    let mut current = source.clone();
    let mut steps = Vec::with_capacity(h);
    let mut checksum: u32 = 0;

    for step in 0..h {
        let sigma = &SIGMAS[schedule.sigma_index[step] as usize];

        // Select dimension using secret ordering
        let priority = (schedule.dim_order[step] as usize) % dims_remaining.len();
        let dim = dims_remaining.remove(priority);

        // Construct next vertex
        let mut next_trits = current.to_bytes();
        next_trits[dim] = dst_bytes[dim];
        let next = CubeAddr::new(next_trits);

        // Compute step weight
        let triplet_idx = (dim / 3).min(8);
        let weight = WEIGHT_VECTOR[sigma[triplet_idx]];

        // Compute commitment
        let commitment = compute_step_commit(
            &current, &next, weight, &schedule.weight_key, step,
        );

        // Update checksum
        let weight_idx = (schedule.sigma_index[step] as usize * 2 + step % 3) % 9;
        checksum = (checksum + WEIGHT_VECTOR[weight_idx]) % MAGIC_CONSTANT;

        steps.push(WalkStep {
            from: current.clone(),
            to: next.clone(),
            dimension: dim,
            sigma_index: schedule.sigma_index[step],
            weight,
            position: step,
            commitment,
        });

        current = next;
    }

    Walk {
        source: source.clone(),
        destination: destination.clone(),
        steps,
        checksum,
    }
}

/// Verify that a walk is structurally valid.
///
/// Checks: contiguous steps, correct start/end, each step fixes
/// exactly one dimension, no dimension fixed twice.
pub fn validate_walk(walk: &Walk) -> bool {
    if walk.steps.is_empty() {
        return walk.source == walk.destination;
    }

    // First step starts from source
    if walk.steps[0].from != walk.source {
        return false;
    }

    // Last step ends at destination
    if walk.steps.last().unwrap().to != walk.destination {
        return false;
    }

    // Steps are contiguous
    for i in 1..walk.steps.len() {
        if walk.steps[i].from != walk.steps[i - 1].to {
            return false;
        }
    }

    // Each step fixes exactly one dimension
    let mut dims_fixed = Vec::with_capacity(walk.steps.len());
    for step in &walk.steps {
        let from = step.from.to_bytes();
        let to = step.to.to_bytes();
        let changed: Vec<usize> = (0..DIMENSIONS)
            .filter(|&d| from[d] != to[d])
            .collect();
        if changed.len() != 1 {
            return false;
        }
        dims_fixed.push(changed[0]);
    }

    // No dimension fixed twice
    let mut seen = [false; DIMENSIONS];
    for &dim in &dims_fixed {
        if seen[dim] { return false; }
        seen[dim] = true;
    }

    // Checksum in range
    walk.checksum < MAGIC_CONSTANT

    // σ indices valid
    && walk.steps.iter().all(|s| (s.sigma_index as usize) < NUM_SIGMAS)
}

/// Compute the expected Hamming distance between two random Z₃¹³ vertices.
///
/// Each dimension independently has P(differ) = 2/3.
/// Expected distance = 13 × 2/3 ≈ 8.667.
pub fn expected_hamming_distance() -> f64 {
    DIMENSIONS as f64 * 2.0 / 3.0
}

/// Compute the expected signature size in bytes.
///
/// Sig = 64 + 48h bytes. E[h] = 13 × 2/3 ≈ 8.667.
/// E[sig_size] = 64 + 48 × 8.667 ≈ 480 bytes.
pub fn expected_signature_size() -> f64 {
    64.0 + (STEP_COMMIT_LEN as f64) * expected_hamming_distance()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr_a() -> CubeAddr { CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }
    fn addr_b() -> CubeAddr { CubeAddr::new([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]) }

    fn test_schedule() -> SecretSchedule {
        SecretSchedule::derive(&addr_a(), b"test-secret")
    }

    #[test]
    fn test_build_walk_correct_length() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert_eq!(walk.length(), 13); // Max distance
    }

    #[test]
    fn test_build_walk_starts_at_source() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert_eq!(walk.source, addr_a());
        assert_eq!(walk.steps[0].from, addr_a());
    }

    #[test]
    fn test_build_walk_ends_at_destination() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert_eq!(walk.destination, addr_b());
        assert_eq!(walk.steps.last().unwrap().to, addr_b());
    }

    #[test]
    fn test_build_walk_contiguous() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        for i in 1..walk.steps.len() {
            assert_eq!(walk.steps[i].from, walk.steps[i - 1].to);
        }
    }

    #[test]
    fn test_build_walk_each_step_fixes_one_dim() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        for step in &walk.steps {
            let from = step.from.to_bytes();
            let to = step.to.to_bytes();
            let changes: usize = (0..13).filter(|&d| from[d] != to[d]).count();
            assert_eq!(changes, 1);
        }
    }

    #[test]
    fn test_build_walk_no_duplicate_dims() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        let dims: Vec<usize> = walk.dimension_order();
        let mut seen = [false; 13];
        for d in dims {
            assert!(!seen[d], "Dimension {} fixed twice", d);
            seen[d] = true;
        }
    }

    #[test]
    fn test_validate_walk_valid() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert!(validate_walk(&walk));
    }

    #[test]
    fn test_walk_checksum_in_range() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert!(walk.checksum < MAGIC_CONSTANT);
    }

    #[test]
    fn test_walk_zero_distance() {
        let walk = build_walk(&addr_a(), &addr_a(), &test_schedule());
        assert_eq!(walk.length(), 0);
        assert!(validate_walk(&walk));
    }

    #[test]
    fn test_expected_hamming() {
        let e = expected_hamming_distance();
        assert!((e - 8.667).abs() < 0.01);
    }

    #[test]
    fn test_expected_sig_size() {
        let e = expected_signature_size();
        assert!(e > 400.0 && e < 500.0, "Expected ~480 bytes, got {}", e);
    }
}