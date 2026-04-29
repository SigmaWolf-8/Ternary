// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `cone_point` — the three cone-point lifts at Gabriel's Horn
//!
//! Canonical map address: **5.H.1.UX5.2** (algebra ring discrete panel).
//! Spec source: Inertissimum §3.6 (last paragraph).
//!
//! All numeric constants live in [`crate::constants`]:
//! `LIFT_UNIT_INT`, `LIFT_FINE_STRUCTURE_INT`, `LIFT_TRIT_BOUNDARY_INT`,
//! `PQR_INT`, `B_INT`.

use crate::constants::{
    LIFT_FINE_STRUCTURE_INT, LIFT_TRIT_BOUNDARY_INT, LIFT_UNIT_INT, PQR_INT,
};

/// Enumerated face of the cone-point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiftFace {
    /// `+1` — the bijective-numeration shift.
    Unit,
    /// `+36` — the fine-structure (algebraic-register) shift.
    FineStructure,
    /// `+8` — the trit-boundary (ghost-letter) shift.
    TritBoundary,
}

impl LiftFace {
    /// All three lift faces in canonical order.
    pub const ALL: [LiftFace; 3] = [
        LiftFace::Unit,
        LiftFace::FineStructure,
        LiftFace::TritBoundary,
    ];

    /// The integer magnitude of this face.
    #[inline]
    pub const fn value(self) -> u64 {
        match self {
            LiftFace::Unit => LIFT_UNIT_INT,
            LiftFace::FineStructure => LIFT_FINE_STRUCTURE_INT,
            LiftFace::TritBoundary => LIFT_TRIT_BOUNDARY_INT,
        }
    }

    /// Apply this lift to a base value: returns `base + value()`.
    #[inline]
    pub const fn lift(self, base: u64) -> u64 {
        base + self.value()
    }
}

const _: () = {
    assert!(LIFT_UNIT_INT == 1);
    assert!(LIFT_FINE_STRUCTURE_INT == 36);
    assert!(LIFT_TRIT_BOUNDARY_INT == 8);
    assert!(LIFT_UNIT_INT != LIFT_FINE_STRUCTURE_INT);
    assert!(LIFT_UNIT_INT != LIFT_TRIT_BOUNDARY_INT);
    assert!(LIFT_FINE_STRUCTURE_INT != LIFT_TRIT_BOUNDARY_INT);
    assert!(LiftFace::Unit.lift(PQR_INT - 1) == PQR_INT);
    assert!(LiftFace::Unit.lift(720) == 721);
    assert!(LiftFace::Unit.lift(12_012) == 12_013);
    assert!(LiftFace::FineStructure.lift(297) == 333);
    assert!(LiftFace::FineStructure.lift(720) == 756);
    assert!(LiftFace::FineStructure.lift(999) == 1035);
    assert!(LiftFace::FineStructure.lift(1053) == 1089);
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::B_INT;

    #[test]
    fn three_faces_have_canonical_values() {
        assert_eq!(LiftFace::Unit.value(), 1);
        assert_eq!(LiftFace::FineStructure.value(), 36);
        assert_eq!(LiftFace::TritBoundary.value(), 8);
    }

    #[test]
    fn faces_array_lists_all_three() {
        assert_eq!(LiftFace::ALL.len(), 3);
        assert!(LiftFace::ALL.contains(&LiftFace::Unit));
        assert!(LiftFace::ALL.contains(&LiftFace::FineStructure));
        assert!(LiftFace::ALL.contains(&LiftFace::TritBoundary));
    }

    #[test]
    fn unit_lift_produces_walk_length() {
        assert_eq!(LiftFace::Unit.lift(PQR_INT - 1), PQR_INT);
        assert_eq!(LiftFace::Unit.lift(PQR_INT - 1), 1001);
    }

    #[test]
    fn fine_structure_lift_lands_on_frame_square_anchors() {
        assert_eq!(LiftFace::FineStructure.lift(297), 333);
        assert_eq!(LiftFace::FineStructure.lift(720), 756);
        assert_eq!(LiftFace::FineStructure.lift(999), 1035);
        assert_eq!(LiftFace::FineStructure.lift(1053), 1089);
    }

    #[test]
    fn trit_boundary_lift_is_b_minus_one_cubed() {
        assert_eq!(LIFT_TRIT_BOUNDARY_INT, (B_INT - 1).pow(3));
    }
}
