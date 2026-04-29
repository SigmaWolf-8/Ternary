// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `discriminant` — circle quadratic `x² − R₄·x + R₆ = 0`
//!
//! Canonical map address: **5.2.1.UX5.1** (algebra ring).
//! Spec source: Inertissimum §3.7, eq 391/397.
//!
//! Quarter-discriminant identity (Inertissimum §3.7, eq 397):
//!
//! ```text
//! 4(p − r)² = (π_geom + 2·R₂)² − 4·R₆ = 40² − 1456 = 144 = Δ.
//! ```
//!
//! All numeric constants live in [`crate::constants`]:
//! `DISCRIMINANT_INT`, `DISCRIMINANT_SQRT_INT`, `QUARTER_DISCRIMINANT_INT`,
//! `ROOT_X1_INT`, `ROOT_X2_INT`, `R_4_INT`, `R_6_INT`,
//! `PQR_ASYMMETRY_SQ_INT`, `P_INT`, `R_INT`.

use crate::constants::{
    DISCRIMINANT_INT, DISCRIMINANT_SQRT_INT, PQR_ASYMMETRY_SQ_INT,
    QUARTER_DISCRIMINANT_INT, ROOT_X1_INT, ROOT_X2_INT, R_4_INT, R_6_INT,
};

/// Evaluate the circle quadratic `x² − R₄·x + R₆` at integer `x`.
///
/// Returns the value as `i64`; the value is `0` at the two roots
/// `ROOT_X1_INT = 14` and `ROOT_X2_INT = 26`, and reaches its minimum
/// `−Δ/4 = −36` at the midpoint `R₄/2 = 20`.
#[inline]
pub const fn evaluate(x: u64) -> i64 {
    let xx = (x * x) as i64;
    let lin = (R_4_INT * x) as i64;
    let cst = R_6_INT as i64;
    xx - lin + cst
}

const _: () = {
    assert!(DISCRIMINANT_INT == 144);
    assert!(DISCRIMINANT_SQRT_INT == 12);
    assert!(DISCRIMINANT_SQRT_INT * DISCRIMINANT_SQRT_INT == DISCRIMINANT_INT);
    assert!(QUARTER_DISCRIMINANT_INT == 36);
    assert!(QUARTER_DISCRIMINANT_INT == PQR_ASYMMETRY_SQ_INT);
    assert!(ROOT_X1_INT + ROOT_X2_INT == R_4_INT);
    assert!(ROOT_X1_INT * ROOT_X2_INT == R_6_INT);
    assert!(ROOT_X1_INT == 14);
    assert!(ROOT_X2_INT == 26);
    let q1 = (ROOT_X1_INT * ROOT_X1_INT) as i64
        - (R_4_INT * ROOT_X1_INT) as i64
        + R_6_INT as i64;
    let q2 = (ROOT_X2_INT * ROOT_X2_INT) as i64
        - (R_4_INT * ROOT_X2_INT) as i64
        + R_6_INT as i64;
    assert!(q1 == 0);
    assert!(q2 == 0);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_is_one_hundred_forty_four() {
        assert_eq!(DISCRIMINANT_INT, 144);
    }

    #[test]
    fn sqrt_delta_is_twelve() {
        assert_eq!(DISCRIMINANT_SQRT_INT, 12);
    }

    #[test]
    fn quarter_delta_equals_pqr_lift() {
        assert_eq!(QUARTER_DISCRIMINANT_INT, 36);
        assert_eq!(QUARTER_DISCRIMINANT_INT, PQR_ASYMMETRY_SQ_INT);
    }

    #[test]
    fn roots_are_fourteen_and_twenty_six() {
        assert_eq!(ROOT_X1_INT, 14);
        assert_eq!(ROOT_X2_INT, 26);
    }

    #[test]
    fn vieta_sum_and_product_check() {
        assert_eq!(ROOT_X1_INT + ROOT_X2_INT, R_4_INT);
        assert_eq!(ROOT_X1_INT * ROOT_X2_INT, R_6_INT);
    }

    #[test]
    fn evaluate_at_roots_is_zero() {
        assert_eq!(evaluate(ROOT_X1_INT), 0);
        assert_eq!(evaluate(ROOT_X2_INT), 0);
    }

    #[test]
    fn evaluate_at_midpoint_is_negative_quarter_delta() {
        let mid = R_4_INT / 2;
        assert_eq!(evaluate(mid), -(QUARTER_DISCRIMINANT_INT as i64));
    }
}
