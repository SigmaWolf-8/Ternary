// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `pqr_asymmetry` — the Forge triple `(p, q, r) = (7, 11, 13)`
//!
//! Canonical map address: **7.5.1.UX4.1** (SE quadrant).
//! Spec source: Inertissimum §0 (Forge triple) and §3.7 (asymmetry).
//!
//! All numeric constants live in [`crate::constants`]:
//! `P_INT`, `Q_INT`, `R_INT`, `LAMBDA_EUV_INT`, `PQR_INT`,
//! `WALK_CLOCK_INT`, `PQR_ASYMMETRY_INT`, `PQR_ASYMMETRY_LOWER_INT`,
//! `PQR_ASYMMETRY_UPPER_INT`, `PQR_ASYMMETRY_SQ_INT`, `R_2_INT`.

use crate::constants::{
    LAMBDA_EUV_INT, PQR_ASYMMETRY_INT, PQR_ASYMMETRY_LOWER_INT,
    PQR_ASYMMETRY_SQ_INT, PQR_ASYMMETRY_UPPER_INT, PQR_INT, P_INT, Q_INT,
    R_2_INT, R_INT, WALK_CLOCK_INT,
};

/// The Forge triple as an ordered tuple `(p, q, r)`.
#[inline]
pub const fn forge_triple() -> (u64, u64, u64) {
    (P_INT, Q_INT, R_INT)
}

const _: () = {
    assert!(P_INT < Q_INT);
    assert!(Q_INT < R_INT);
    assert!(LAMBDA_EUV_INT == P_INT * R_INT);
    assert!(LAMBDA_EUV_INT == 91);
    assert!(PQR_INT == P_INT * Q_INT * R_INT);
    assert!(PQR_INT == 1001);
    assert!(WALK_CLOCK_INT == 1000);
    assert!(PQR_ASYMMETRY_INT == 6);
    assert!(PQR_ASYMMETRY_SQ_INT == 36);
    assert!(PQR_ASYMMETRY_LOWER_INT == R_2_INT);
    assert!(PQR_ASYMMETRY_UPPER_INT == 2);
    assert!(PQR_ASYMMETRY_LOWER_INT + PQR_ASYMMETRY_UPPER_INT == PQR_ASYMMETRY_INT);
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
        assert_eq!(LAMBDA_EUV_INT, 91);
    }

    #[test]
    fn walk_length_is_one_thousand_one() {
        assert_eq!(PQR_INT, 1001);
    }

    #[test]
    fn fine_structure_lift_is_thirty_six() {
        assert_eq!(PQR_ASYMMETRY_SQ_INT, 36);
    }

    #[test]
    fn asymmetries_partition_full_gap() {
        assert_eq!(PQR_ASYMMETRY_LOWER_INT + PQR_ASYMMETRY_UPPER_INT, PQR_ASYMMETRY_INT);
    }
}
