// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

//! # PT26-DSA Walk Construction
//!
//! Builds secret walks through the 13D ternary hypercube using
//! the σ permutation schedule. The walk is the core structure
//! that the signer knows and the verifier checks.
//!
//! Updated for unified PT26-DSA: uses GF(3) step tokens instead
//! of per-step sponge commitments.

use crate::cube_addr::CubeAddr;
use crate::plenum_square::{MAGIC_CONSTANT};
use crate::pt26_dsa::{
    DIMENSIONS, Schedule, trit_diff, step_token, walk_token, walk_parity,
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
    /// Step token (GF(3) weighted triplet evaluation mod 333).
    pub token: u32,
    /// Step position in the walk (0-indexed).
    pub position: usize,
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
    /// Walk token (accumulated product mod 333).
    pub walk_tok: u32,
    /// Walk parity (8-trit ECC syndrome).
    pub parity: [u8; 8],
}

impl Walk {
    /// Walk length (= Hamming distance between source and destination).
    pub fn length(&self) -> usize {
        self.steps.len()
    }

    /// Get the dimension ordering used in this walk.
    pub fn dimension_order(&self) -> Vec<usize> {
        self.steps.iter().map(|s| s.dimension).collect()
    }

    /// Get the σ schedule used in this walk.
    pub fn sigma_schedule(&self) -> Vec<u8> {
        self.steps.iter().map(|s| s.sigma_index).collect()
    }

    /// Extract step tokens.
    pub fn step_tokens(&self) -> Vec<u32> {
        self.steps.iter().map(|s| s.token).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WALK BUILDER
// ═══════════════════════════════════════════════════════════════════════

/// Construct a secret walk using the σ schedule.
///
/// Uses GF(3) trit arithmetic for step tokens (sub-nanosecond).
/// Zero sponge calls.
pub fn build_walk(
    source: &CubeAddr,
    destination: &CubeAddr,
    schedule: &Schedule,
) -> Walk {
    let src_bytes = source.to_bytes();
    let dst_bytes = destination.to_bytes();
    let h = (0..DIMENSIONS).filter(|&d| src_bytes[d] != dst_bytes[d]).count();

    let mut dims_remaining: Vec<usize> = (0..DIMENSIONS)
        .filter(|&d| src_bytes[d] != dst_bytes[d])
        .collect();

    let mut current = source.clone();
    let mut steps = Vec::with_capacity(h);
    let mut tokens = Vec::with_capacity(h);

    for step in 0..h {
        let si = schedule.sigma[step] as usize;
        let priority = (schedule.dim_order[step] as usize) % dims_remaining.len();
        let dim = dims_remaining.remove(priority);

        let cur_bytes = current.to_bytes();
        let mut next_trits = cur_bytes;
        next_trits[dim] = dst_bytes[dim];
        let next = CubeAddr::new(next_trits);

        let delta = trit_diff(&next_trits, &cur_bytes);
        let tok = step_token(&delta, si, step);
        tokens.push(tok);

        steps.push(WalkStep {
            from: current.clone(),
            to: next.clone(),
            dimension: dim,
            sigma_index: schedule.sigma[step],
            token: tok,
            position: step,
        });

        current = next;
    }

    let wt = walk_token(&tokens);
    let par = walk_parity(&src_bytes, &dst_bytes, wt, &tokens);

    Walk {
        source: source.clone(),
        destination: destination.clone(),
        steps,
        walk_tok: wt,
        parity: par,
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

    if walk.steps[0].from != walk.source {
        return false;
    }

    if walk.steps.last().unwrap().to != walk.destination {
        return false;
    }

    for i in 1..walk.steps.len() {
        if walk.steps[i].from != walk.steps[i - 1].to {
            return false;
        }
    }

    let mut seen = [false; DIMENSIONS];
    for step in &walk.steps {
        let from = step.from.to_bytes();
        let to = step.to.to_bytes();
        let changed: Vec<usize> = (0..DIMENSIONS)
            .filter(|&d| from[d] != to[d])
            .collect();
        if changed.len() != 1 {
            return false;
        }
        let dim = changed[0];
        if seen[dim] { return false; }
        seen[dim] = true;
    }

    walk.walk_tok < MAGIC_CONSTANT
        && walk.steps.iter().all(|s| (s.sigma_index as usize) < 4)
}

/// Compute the expected Hamming distance between two random Z₃¹³ vertices.
///
/// Each dimension independently has P(differ) = 2/3.
/// Expected distance = 13 × 2/3 ≈ 8.667.
pub fn expected_hamming_distance() -> f64 {
    DIMENSIONS as f64 * 2.0 / 3.0
}

/// Compute the expected signature size (fixed at 71 bytes in unified PT26-DSA).
pub fn signature_size() -> usize {
    71
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr_a() -> CubeAddr { CubeAddr::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }
    fn addr_b() -> CubeAddr { CubeAddr::new([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]) }

    fn test_schedule() -> Schedule {
        Schedule::derive(&addr_a().to_bytes(), b"test-secret")
    }

    #[test]
    fn test_build_walk_correct_length() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert_eq!(walk.length(), 13);
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
    fn test_walk_token_in_range() {
        let walk = build_walk(&addr_a(), &addr_b(), &test_schedule());
        assert!(walk.walk_tok < MAGIC_CONSTANT);
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
    fn test_signature_size() {
        assert_eq!(signature_size(), 71);
    }
}
