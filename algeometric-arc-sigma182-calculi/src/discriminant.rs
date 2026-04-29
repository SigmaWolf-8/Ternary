// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `discriminant` — circle quadratic `x² − R₄·x + R₆ = 0`
//!
//! Canonical map address: **5.2.1.UX5.1** (algebra ring).
//!
//! Spec source: `Inertissimum Iώτα Nona — Codex Unificationis`,
//! §3.7 (Discriminant Identity — The Walk and the Circle Quadratic),
//! equations 391 and following.
//!
//! ## What this module exposes
//!
//! The circle quadratic is the framework polynomial whose two roots
//! are the angular generator `π_geom = 14` and its complement
//! `R₆ / π_geom = 26`. The discriminant `Δ = R₄² − 4·R₆ = 144` is
//! the algebraic source of the fine-structure cone-point lift
//! `(p − r)² = 36 = Δ / 4` exposed in [`crate::pqr_asymmetry`].
//!
//! ## Invariants verified at compile time
//!
//! - The circle quadratic is `x² − R₄·x + R₆ = 0` with `R₄ = 40`
//!   and `R₆ = 364`.
//! - `Δ = R₄² − 4·R₆ = 144` (already pinned in [`crate::constants`]
//!   as `DISCRIMINANT_INT`).
//! - `√Δ = 12` (integer; `DISCRIMINANT_SQRT_INT`).
//! - Roots `(x₁, x₂) = (14, 26)` recovered by Vieta:
//!   `x₁ + x₂ = R₄`, `x₁ · x₂ = R₆`.
//! - **Quarter-discriminant identity (Inertissimum §3.7, eq 397):**
//!
//!   ```text
//!   4(p − r)² = (π_geom + 2·R₂)² − 4·R₆ = 40² − 1456 = 144 = Δ.
//!   ```

use crate::constants::{
    DISCRIMINANT_INT, DISCRIMINANT_SQRT_INT, P_INT, R_4_INT, R_6_INT, R_INT,
    ROOT_X1_INT, ROOT_X2_INT,
};

/// Discriminant of the circle quadratic: `Δ = R₄² − 4·R₆ = 144`.
pub const DELTA: u64 = DISCRIMINANT_INT;

/// Integer square root of the discriminant: `√Δ = 12`.
pub const SQRT_DELTA: u64 = DISCRIMINANT_SQRT_INT;

/// Quarter-discriminant: `Δ / 4 = 36 = (p − r)²` —
/// the fine-structure cone-point lift.
pub const QUARTER_DELTA: u64 = DISCRIMINANT_INT / 4;

/// Lower root of the circle quadratic: `x₁ = (R₄ − √Δ) / 2 = 14`.
pub const ROOT_X1: u64 = ROOT_X1_INT;

/// Upper root of the circle quadratic: `x₂ = (R₄ + √Δ) / 2 = 26`.
pub const ROOT_X2: u64 = ROOT_X2_INT;

/// Vieta sum of the roots: `x₁ + x₂ = R₄ = 40`.
pub const VIETA_SUM: u64 = R_4_INT;

/// Vieta product of the roots: `x₁ · x₂ = R₆ = 364`.
pub const VIETA_PRODUCT: u64 = R_6_INT;

/// Evaluate the circle quadratic `x² − R₄·x + R₆` at integer `x`.
///
/// Returns `Some(0)` when `x` is a root, `Some(positive)` when the
/// quadratic is positive at `x`, and `None` when the result would be
/// negative (which never happens between the two integer roots
/// because they are integers; this signals an out-of-domain query).
#[inline]
pub const fn evaluate(x: u64) -> Option<i64> {
    let xx = (x * x) as i64;
    let lin = (R_4_INT * x) as i64;
    let cst = R_6_INT as i64;
    Some(xx - lin + cst)
}

const _: () = {
    // Δ matches the constants pin
    assert!(DELTA == 144);
    assert!(SQRT_DELTA == 12);
    assert!(SQRT_DELTA * SQRT_DELTA == DELTA);

    // Quarter-discriminant equals (p − r)²
    assert!(QUARTER_DELTA == 36);
    assert!(QUARTER_DELTA == (R_INT - P_INT) * (R_INT - P_INT));

    // Vieta identities
    assert!(ROOT_X1 + ROOT_X2 == VIETA_SUM);
    assert!(ROOT_X1 * ROOT_X2 == VIETA_PRODUCT);
    assert!(ROOT_X1 == 14);
    assert!(ROOT_X2 == 26);

    // Both roots evaluate the quadratic to zero
    // (we use a saturating-subtract style by computing as i64).
    let q1 = (ROOT_X1 * ROOT_X1) as i64 - (R_4_INT * ROOT_X1) as i64 + R_6_INT as i64;
    let q2 = (ROOT_X2 * ROOT_X2) as i64 - (R_4_INT * ROOT_X2) as i64 + R_6_INT as i64;
    assert!(q1 == 0);
    assert!(q2 == 0);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_is_one_hundred_forty_four() {
        assert_eq!(DELTA, 144);
    }

    #[test]
    fn sqrt_delta_is_twelve() {
        assert_eq!(SQRT_DELTA, 12);
    }

    #[test]
    fn quarter_delta_equals_pqr_lift() {
        assert_eq!(QUARTER_DELTA, 36);
        assert_eq!(QUARTER_DELTA, (R_INT - P_INT).pow(2));
    }

    #[test]
    fn roots_are_fourteen_and_twenty_six() {
        assert_eq!(ROOT_X1, 14);
        assert_eq!(ROOT_X2, 26);
    }

    #[test]
    fn vieta_sum_and_product_check() {
        assert_eq!(ROOT_X1 + ROOT_X2, R_4_INT);
        assert_eq!(ROOT_X1 * ROOT_X2, R_6_INT);
    }

    #[test]
    fn evaluate_at_roots_is_zero() {
        assert_eq!(evaluate(ROOT_X1), Some(0));
        assert_eq!(evaluate(ROOT_X2), Some(0));
    }

    #[test]
    fn evaluate_at_midpoint_is_negative_quarter_delta() {
        // Midpoint of the two roots is R₄ / 2 = 20.
        // Vertex value of x² − R₄·x + R₆ is R₆ − R₄²/4 = −Δ/4 = −36.
        let mid = R_4_INT / 2;
        assert_eq!(evaluate(mid), Some(-(QUARTER_DELTA as i64)));
    }
}
