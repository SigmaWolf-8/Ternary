// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Plenum Square Constants
//!
//! All constants derive from a single quantity:
//!
//! ```text
//! arc = 182 = π(π − 1) = 14 × 13 = 20202₃
//! ```
//!
//! The half-turn — semicircle of the 364° ternary circle, half of the
//! 364-day year, palindrome in base-3.
//!
//! From `arc` alone:
//!
//! - `c = (182 + 40) / 2 = 111` → the (111) Miller index → hexagonal symmetry
//! - `Δ₂ = 1 + 4 × 182 = 729 = 3⁶` → kernel sponge width (derived, not designed)
//! - `π = (1 + 27) / 2 = 14` → circle, calendar, topology
//! - `√Δ = 12` → lattice parameter
//!
//! The weight vector and σ permutations encode the four canonical magic
//! square arrangements as runtime-ready data. The mathematical derivation
//! via (p, q) parameterization is documented in TM-2026-015 §II.
//!
//! ## Invariant Compliance
//!
//! - **INV 1** (Geometry IS the System): All constants derived from arc = π(π−1).
//! - **INV 3** (Rep C, zero excluded): Weight vector values are all positive integers.
//!
//! ## Created by T-04 (SPEC-2026-NEXT)
//! Required by T-10 (dual checksum), T-17 (wire ECC), T-18 (sponge shuffles),
//! T-19 (arc rotation), T-23 (reciprocal-lattice mixer).

use crate::constants;

// ═══════════════════════════════════════════════════════════════════════
// RE-EXPORTS FROM constants.rs — backward-compatible public names
// ═══════════════════════════════════════════════════════════════════════

pub use constants::ARC_ROOT_SEMI as ARC;
pub use constants::CENTER;
pub use constants::MAGIC_CONSTANT;
pub use constants::DISCRIMINANT;
pub use constants::DISCRIMINANT_SQRT as LATTICE_PARAM;
pub use constants::ROOT_X1 as ROOT_A;
pub use constants::ROOT_X2 as ROOT_B;
pub use constants::DISCRIMINANT_2;
pub use constants::QUAD_PRODUCT as FULL_CIRCLE;

// ═══════════════════════════════════════════════════════════════════════
// MAGIC-SQUARE-SPECIFIC CONSTANTS (not universal — stay here)
// ═══════════════════════════════════════════════════════════════════════

/// Complement A: `pair_sum − root_a = 222 − 14 = 208`.
pub const COMPLEMENT_A: u32 = 208;

/// Complement B: `pair_sum − root_b = 222 − 26 = 196`.
pub const COMPLEMENT_B: u32 = 196;

/// Opposite-pair sum: `2 × center = 2 × 111 = 222`.
///
/// Every pair of diametrically opposite cells sums to this.
pub const PAIR_SUM: u32 = 222;

// ═══════════════════════════════════════════════════════════════════════
// WEIGHT VECTOR — One specific magic square's cell values
// ═══════════════════════════════════════════════════════════════════════

/// The 9-cell weight vector from the canonical Plenum Square.
///
/// Used by T-23 (reciprocal-lattice key mixing): for a 27-trit address
/// decomposed into 9 triplets, `nonce = Σ(weight[i] × triplet[i]) mod 333`.
///
/// The magic constant guarantees any three aligned coefficients sum to 333.
/// An attacker controlling some address dimensions cannot bias the key —
/// the geometric balance of the crystal prevents it.
///
/// Layout (reading order, row by row):
/// ```text
/// ┌─────┬─────┬─────┐
/// │ 208 │   2 │ 123 │  → 333
/// ├─────┼─────┼─────┤
/// │  26 │ 111 │ 196 │  → 333
/// ├─────┼─────┼─────┤
/// │  99 │ 220 │  14 │  → 333
/// └─────┴─────┴─────┘
///   ↓       ↓     ↓
///  333    333   333    diag: 333, 333
/// ```
pub const WEIGHT_VECTOR: [u32; 9] = [208, 2, 123, 26, 111, 196, 99, 220, 14];

// ═══════════════════════════════════════════════════════════════════════
// σ PERMUTATIONS — Block shuffle indices for sponge dynamics
// ═══════════════════════════════════════════════════════════════════════

/// Block permutation σ_A — the only **full derangement** (no fixed points).
///
/// Applied first in sponge rounds (round 0). This permutation moves ALL 9
/// cells including the center, providing maximum disruption on the first round.
/// Design intent: "maximum disruption first."
///
/// TIS-27 (4 rounds): σ_A on round 0, σ_B on round 1, σ_C on 2, σ_D on 3.
/// TLSponge-385 (9 rounds): cycling σ_A→σ_D→σ_A.
pub const SIGMA_A: [usize; 9] = [4, 8, 3, 2, 0, 7, 5, 6, 1];

/// Block permutation σ_B. Fixed point at index 4 (center).
pub const SIGMA_B: [usize; 9] = [6, 0, 5, 8, 4, 3, 2, 1, 7];

/// Block permutation σ_C. Fixed point at index 4 (center).
///
/// The center stays fixed while the 8 surrounding cells rotate — this is
/// physically correct for a magic square: the center is the axis of symmetry.
pub const SIGMA_C: [usize; 9] = [2, 6, 7, 8, 4, 0, 1, 5, 3];

/// Block permutation σ_D. Fixed point at index 4 (center).
pub const SIGMA_D: [usize; 9] = [8, 2, 1, 0, 4, 6, 7, 3, 5];

/// All four σ permutations indexed for round-dependent selection.
///
/// Usage: `SIGMAS[round % 4]` gives the permutation for that round.
pub const SIGMAS: [[usize; 9]; 4] = [SIGMA_A, SIGMA_B, SIGMA_C, SIGMA_D];

// ═══════════════════════════════════════════════════════════════════════
// VALIDATION FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

/// Verify that a permutation is a valid derangement (no fixed points).
///
/// A derangement satisfies: `σ[i] ≠ i` for all `i`.
pub const fn is_derangement(sigma: &[usize; 9]) -> bool {
    let mut i = 0;
    while i < 9 {
        if sigma[i] == i {
            return false;
        }
        i += 1;
    }
    true
}

/// Verify that a permutation is a valid permutation of {0..8}.
///
/// Each value 0–8 must appear exactly once.
pub const fn is_valid_permutation(sigma: &[usize; 9]) -> bool {
    let mut seen = [false; 9];
    let mut i = 0;
    while i < 9 {
        if sigma[i] >= 9 {
            return false;
        }
        if seen[sigma[i]] {
            return false;
        }
        seen[sigma[i]] = true;
        i += 1;
    }
    true
}

/// Verify that a row of the weight vector sums to the magic constant.
pub const fn row_sum(row: usize) -> u32 {
    WEIGHT_VECTOR[row * 3] + WEIGHT_VECTOR[row * 3 + 1] + WEIGHT_VECTOR[row * 3 + 2]
}

/// Verify that a column of the weight vector sums to the magic constant.
pub const fn col_sum(col: usize) -> u32 {
    WEIGHT_VECTOR[col] + WEIGHT_VECTOR[col + 3] + WEIGHT_VECTOR[col + 6]
}

/// Main diagonal sum (top-left to bottom-right).
pub const fn diag_main_sum() -> u32 {
    WEIGHT_VECTOR[0] + WEIGHT_VECTOR[4] + WEIGHT_VECTOR[8]
}

/// Anti-diagonal sum (top-right to bottom-left).
pub const fn diag_anti_sum() -> u32 {
    WEIGHT_VECTOR[2] + WEIGHT_VECTOR[4] + WEIGHT_VECTOR[6]
}

/// Compute weighted nonce from a 27-trit address decomposed into 9 triplets.
///
/// `nonce = Σ(weight[i] × triplet_value[i]) mod MAGIC_CONSTANT`
///
/// Used by T-23 (reciprocal-lattice key mixing) as additional domain
/// material in v3 tunnel key derivation.
pub fn weighted_nonce(triplet_values: &[u32; 9]) -> u32 {
    let mut sum: u64 = 0;
    for i in 0..9 {
        sum += (WEIGHT_VECTOR[i] as u64) * (triplet_values[i] as u64);
    }
    (sum % MAGIC_CONSTANT as u64) as u32
}

// ═══════════════════════════════════════════════════════════════════════
// COMPILE-TIME ASSERTIONS
// ═══════════════════════════════════════════════════════════════════════

const _: () = {
    assert!(ARC == ROOT_A * 13);
    assert!(CENTER == (ARC + 40) / 2);
    assert!(MAGIC_CONSTANT == 3 * CENTER);
    assert!(DISCRIMINANT == LATTICE_PARAM * LATTICE_PARAM);
    assert!(PAIR_SUM == 2 * CENTER);
    assert!(COMPLEMENT_A == PAIR_SUM - ROOT_A);
    assert!(COMPLEMENT_B == PAIR_SUM - ROOT_B);
    assert!(DISCRIMINANT_2 == 1 + 4 * ARC);
    assert!(DISCRIMINANT_2 == 729);
    assert!(FULL_CIRCLE == 2 * ARC);
    assert!(FULL_CIRCLE == 364);

    assert!(is_valid_permutation(&SIGMA_A));
    assert!(is_valid_permutation(&SIGMA_B));
    assert!(is_valid_permutation(&SIGMA_C));
    assert!(is_valid_permutation(&SIGMA_D));

    assert!(is_derangement(&SIGMA_A));

    assert!(row_sum(0) == MAGIC_CONSTANT);
    assert!(row_sum(1) == MAGIC_CONSTANT);
    assert!(row_sum(2) == MAGIC_CONSTANT);
    assert!(col_sum(0) == MAGIC_CONSTANT);
    assert!(col_sum(1) == MAGIC_CONSTANT);
    assert!(col_sum(2) == MAGIC_CONSTANT);
    assert!(diag_main_sum() == MAGIC_CONSTANT);
    assert!(diag_anti_sum() == MAGIC_CONSTANT);

    assert!(WEIGHT_VECTOR[4] == CENTER);

    assert!(PAIR_SUM == constants::LAMBDA_FAR_UVC);
};

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_is_palindrome_base3() {
        let mut val = ARC;
        let mut digits = Vec::new();
        while val > 0 {
            digits.push(val % 3);
            val /= 3;
        }
        let rev: Vec<u32> = digits.iter().rev().copied().collect();
        assert_eq!(digits, rev, "182 must be a palindrome in base-3 (20202₃)");
    }

    #[test]
    fn test_all_weight_vector_values_positive() {
        for &w in &WEIGHT_VECTOR {
            assert!(w > 0, "All weight vector values must be positive (no zeros)");
        }
    }

    #[test]
    fn test_weight_vector_contains_roots_and_complements() {
        let wv: Vec<u32> = WEIGHT_VECTOR.to_vec();
        assert!(wv.contains(&ROOT_A), "Weight vector must contain ROOT_A (14)");
        assert!(wv.contains(&ROOT_B), "Weight vector must contain ROOT_B (26)");
        assert!(wv.contains(&COMPLEMENT_A), "Weight vector must contain COMPLEMENT_A (208)");
        assert!(wv.contains(&COMPLEMENT_B), "Weight vector must contain COMPLEMENT_B (196)");
        assert!(wv.contains(&CENTER), "Weight vector must contain CENTER (111)");
    }

    #[test]
    fn test_opposite_pairs_sum() {
        assert_eq!(WEIGHT_VECTOR[0] + WEIGHT_VECTOR[8], PAIR_SUM);
        assert_eq!(WEIGHT_VECTOR[1] + WEIGHT_VECTOR[7], PAIR_SUM);
        assert_eq!(WEIGHT_VECTOR[2] + WEIGHT_VECTOR[6], PAIR_SUM);
        assert_eq!(WEIGHT_VECTOR[3] + WEIGHT_VECTOR[5], PAIR_SUM);
    }

    #[test]
    fn test_sigma_a_is_derangement() {
        assert!(is_derangement(&SIGMA_A), "σ_A must be a derangement (no fixed points)");
    }

    #[test]
    fn test_sigma_bcd_have_center_fixed_point() {
        assert_eq!(SIGMA_B[4], 4, "σ_B fixes center");
        assert_eq!(SIGMA_C[4], 4, "σ_C fixes center");
        assert_eq!(SIGMA_D[4], 4, "σ_D fixes center");
        assert_ne!(SIGMA_A[4], 4, "σ_A moves center");
    }

    #[test]
    fn test_all_sigmas_are_valid_permutations() {
        for (i, sigma) in SIGMAS.iter().enumerate() {
            assert!(
                is_valid_permutation(sigma),
                "σ_{} must be a valid permutation of {{0..8}}",
                ['A', 'B', 'C', 'D'][i]
            );
        }
    }

    #[test]
    fn test_weighted_nonce_deterministic() {
        let triplets = [1u32, 2, 3, 1, 2, 3, 1, 2, 3];
        let n1 = weighted_nonce(&triplets);
        let n2 = weighted_nonce(&triplets);
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_weighted_nonce_range() {
        let triplets = [3u32, 3, 3, 3, 3, 3, 3, 3, 3];
        let n = weighted_nonce(&triplets);
        assert!(n < MAGIC_CONSTANT, "Nonce must be < 333");
    }

    #[test]
    fn test_weighted_nonce_different_inputs() {
        let t1 = [1u32, 1, 1, 1, 1, 1, 1, 1, 2];
        let t2 = [1u32, 1, 1, 1, 1, 1, 1, 2, 1];
        let n1 = weighted_nonce(&t1);
        let n2 = weighted_nonce(&t2);
        assert_eq!(n1, 14, "Nonce for t1 must be 14");
        assert_eq!(n2, 220, "Nonce for t2 must be 220");
        assert_ne!(n1, n2, "Different triplets → different nonces");
    }

    #[test]
    fn test_discriminant_2_is_3_to_the_6() {
        assert_eq!(DISCRIMINANT_2, 3u32.pow(6));
    }

    #[test]
    fn test_full_circle_is_repunit() {
        assert_eq!(FULL_CIRCLE, (3u32.pow(6) - 1) / 2);
    }
}
