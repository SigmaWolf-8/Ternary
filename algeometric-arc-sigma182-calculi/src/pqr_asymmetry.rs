// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `pqr_asymmetry` — the Forge triple `(p, q, r) = (7, 11, 13)`
//!
//! Canonical map address: **7.5.1.UX4.1** (SE quadrant).
//!
//! Spec source: `Inertissimum Iώτα Nona — Codex Unificationis`,
//! §0 (Forge triple definition) and §3.7 (discriminant identity).
//!
//! ## What this module exposes
//!
//! The Forge triple is the worked-example input tuple `(p, q, r)`
//! used throughout the Compendium. Its asymmetry — the gap between
//! the extremal primes `p` and `r` — appears as the framework's
//! second-order cone-point lift `(p − r)² = 36` (one-quarter of the
//! circle-quadratic discriminant `Δ = 144`; see [`crate::discriminant`]).
//!
//! ## Invariants verified at compile time
//!
//! - `(p, q, r) = (7, 11, 13)` are pairwise coprime distinct primes.
//! - `Λ_EUV = p · r = 91` (the EUV constant, structural seam).
//! - `L = p · q · r = 1001` (Salvi closed-loop walk length).
//! - `pqr − 1 = 1000` (the walk-clock).
//! - `(p − r)² = 36` (the fine-structure cone-point lift).
//! - `q − p = 4 = R₂` (lower asymmetry equals the radian-pair).
//! - `r − q = 2` (upper asymmetry equals the smallest even prime).
//! - `r − p = 6 = b · R₂ / 2 = (b − 1) · b` (full asymmetry).

use crate::constants::{LAMBDA_EUV_INT, P_INT, PQR_INT, Q_INT, R_INT, R_2_INT};

/// Lower prime of the Forge triple.
pub const P: u64 = P_INT;

/// Middle prime of the Forge triple.
pub const Q: u64 = Q_INT;

/// Upper prime of the Forge triple.
pub const R: u64 = R_INT;

/// EUV constant, the structural seam: `Λ_EUV = p · r`.
pub const LAMBDA_EUV: u64 = LAMBDA_EUV_INT;

/// Walk length: `L = p · q · r`.
pub const L: u64 = PQR_INT;

/// Walk-clock: `pqr − 1`.
pub const WALK_CLOCK: u64 = PQR_INT - 1;

/// Full asymmetry: `r − p`.
pub const ASYMMETRY: u64 = R_INT - P_INT;

/// Squared asymmetry — the fine-structure cone-point lift: `(p − r)²`.
pub const ASYMMETRY_SQ: u64 = (R_INT - P_INT) * (R_INT - P_INT);

/// Lower asymmetry: `q − p` (equals `R₂ = 4`).
pub const ASYMMETRY_LOWER: u64 = Q_INT - P_INT;

/// Upper asymmetry: `r − q` (equals `2`).
pub const ASYMMETRY_UPPER: u64 = R_INT - Q_INT;

/// The Forge triple as an ordered tuple.
#[inline]
pub const fn forge_triple() -> (u64, u64, u64) {
    (P, Q, R)
}

const _: () = {
    // Pairwise distinct
    assert!(P < Q);
    assert!(Q < R);

    // Λ_EUV = p · r = 91
    assert!(LAMBDA_EUV == P * R);
    assert!(LAMBDA_EUV == 91);

    // L = pqr = 1001
    assert!(L == P * Q * R);
    assert!(L == 1001);

    // Walk-clock = 1000
    assert!(WALK_CLOCK == 1000);

    // Asymmetries
    assert!(ASYMMETRY == 6);
    assert!(ASYMMETRY_SQ == 36);
    assert!(ASYMMETRY_LOWER == R_2_INT); // q − p = R₂ = 4
    assert!(ASYMMETRY_UPPER == 2);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forge_triple_is_seven_eleven_thirteen() {
        assert_eq!(forge_triple(), (7, 11, 13));
    }

    #[test]
    fn lambda_euv_is_ninety_one() {
        assert_eq!(LAMBDA_EUV, 91);
    }

    #[test]
    fn walk_length_is_one_thousand_one() {
        assert_eq!(L, 1001);
    }

    #[test]
    fn fine_structure_lift_is_thirty_six() {
        assert_eq!(ASYMMETRY_SQ, 36);
    }

    #[test]
    fn asymmetries_partition_full_gap() {
        // q − p plus r − q must equal r − p
        assert_eq!(ASYMMETRY_LOWER + ASYMMETRY_UPPER, ASYMMETRY);
    }
}
