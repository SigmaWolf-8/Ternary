// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `cone_point` — the three cone-point lifts at Gabriel's Horn
//!
//! Canonical map address: **5.H.1.UX5.2** (algebra ring discrete panel).
//!
//! Spec source: `Inertissimum Iώτα Nona — Codex Unificationis`,
//! §3.6, last paragraph (the three cone-point lifts).
//!
//! ## What this module exposes
//!
//! The Compendium identifies **three structural lifts** at the
//! Gabriel's-Horn cone-point — three faces of "the same no-thing,
//! the zero that cannot be held":
//!
//! | Lift | Value | Reading |
//! |------|-------|---------|
//! | Unit cone-point | `+1` | the fundamental shift of bijective numeration that closes a lattice into a cycle (e.g. walk → `pqr`, koppa → `Λ_EUV`, `720 → 721`, `12 012 → 12 013`) |
//! | Fine-structure cone-point | `+(p − r)² = +36` | the square of the gap between the extremal Forge primes; arises as the quarter-discriminant of the circle quadratic (see [`crate::discriminant`]); the minimal lift in the algebraic register |
//! | Trit-boundary cone-point | `+(b − 1)³ = +8` | the trit-boundary cube — a third-order `+1` born from the ghost-letter structure |
//!
//! ## Invariants verified at compile time
//!
//! - `LIFT_UNIT = 1`.
//! - `LIFT_FINE_STRUCTURE = (p − r)² = 36`.
//! - `LIFT_TRIT_BOUNDARY = (b − 1)³ = 8`.
//! - The three lifts are pairwise distinct.
//! - Worked instances of `+1`:
//!   - walk-clock `pqr − 1 = 1000` lifts to walk length `pqr = 1001`;
//!   - the `720 → 721` lift (`Δ · F(5)` lifts to the κ-bridge numerator);
//!   - the `12 012 → 12 013` lift (`12 · pqr` lifts to a structural prime).
//! - Worked instance of `+36`:
//!   - `(bq)² = 1089` is the frame square; subtracting `+36` from
//!     each of the four anchors (`333`, `756`, `1035`, `1089`) gives
//!     the four pre-lift bases listed in Inertissimum §3.6.
//! - Worked instance of `+8`:
//!   - `b³ − 1 + 1 = 27` (the algebraic cube); `+8` is its trit-boundary
//!     counterpart.

use crate::constants::{B_INT, P_INT, PQR_INT, R_INT};

/// First face of the cone-point: the unit lift `+1`.
pub const LIFT_UNIT: u64 = 1;

/// Second face: the fine-structure lift `(p − r)² = 36`.
pub const LIFT_FINE_STRUCTURE: u64 = (R_INT - P_INT) * (R_INT - P_INT);

/// Third face: the trit-boundary lift `(b − 1)³ = 8`.
pub const LIFT_TRIT_BOUNDARY: u64 = (B_INT - 1) * (B_INT - 1) * (B_INT - 1);

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
    /// The integer magnitude of this face.
    #[inline]
    pub const fn value(self) -> u64 {
        match self {
            LiftFace::Unit => LIFT_UNIT,
            LiftFace::FineStructure => LIFT_FINE_STRUCTURE,
            LiftFace::TritBoundary => LIFT_TRIT_BOUNDARY,
        }
    }

    /// Apply this lift to a base value: returns `base + value()`.
    #[inline]
    pub const fn lift(self, base: u64) -> u64 {
        base + self.value()
    }
}

/// All three lift faces in canonical order.
pub const FACES: [LiftFace; 3] = [
    LiftFace::Unit,
    LiftFace::FineStructure,
    LiftFace::TritBoundary,
];

const _: () = {
    // Magnitudes
    assert!(LIFT_UNIT == 1);
    assert!(LIFT_FINE_STRUCTURE == 36);
    assert!(LIFT_TRIT_BOUNDARY == 8);

    // Pairwise distinct
    assert!(LIFT_UNIT != LIFT_FINE_STRUCTURE);
    assert!(LIFT_UNIT != LIFT_TRIT_BOUNDARY);
    assert!(LIFT_FINE_STRUCTURE != LIFT_TRIT_BOUNDARY);

    // Worked +1 instance: walk-clock to walk length
    assert!(LiftFace::Unit.lift(PQR_INT - 1) == PQR_INT);

    // Worked +1 instance: 720 → 721 (κ-bridge)
    assert!(LiftFace::Unit.lift(720) == 721);

    // Worked +1 instance: 12 012 → 12 013
    assert!(LiftFace::Unit.lift(12_012) == 12_013);

    // Worked +36 instance: each frame-square anchor lifted from its base
    // (Inertissimum §3.6 table)
    assert!(LiftFace::FineStructure.lift(297) == 333); //   b³·q + 36 = Plenum magic
    assert!(LiftFace::FineStructure.lift(720) == 756); //   Δ·F(5) + 36 = Grand Z₂₈
    assert!(LiftFace::FineStructure.lift(999) == 1035); //  pqr − 2 + 36 = triangular anchor
    assert!(LiftFace::FineStructure.lift(1053) == 1089); // b⁴·r + 36 = frame self
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_faces_have_canonical_values() {
        assert_eq!(LiftFace::Unit.value(), 1);
        assert_eq!(LiftFace::FineStructure.value(), 36);
        assert_eq!(LiftFace::TritBoundary.value(), 8);
    }

    #[test]
    fn faces_array_lists_all_three() {
        assert_eq!(FACES.len(), 3);
        assert!(FACES.contains(&LiftFace::Unit));
        assert!(FACES.contains(&LiftFace::FineStructure));
        assert!(FACES.contains(&LiftFace::TritBoundary));
    }

    #[test]
    fn unit_lift_produces_walk_length() {
        assert_eq!(LiftFace::Unit.lift(PQR_INT - 1), PQR_INT);
        assert_eq!(LiftFace::Unit.lift(PQR_INT - 1), 1001);
    }

    #[test]
    fn fine_structure_lift_lands_on_frame_square_anchors() {
        // The four anchors from Inertissimum §3.6
        assert_eq!(LiftFace::FineStructure.lift(297), 333);
        assert_eq!(LiftFace::FineStructure.lift(720), 756);
        assert_eq!(LiftFace::FineStructure.lift(999), 1035);
        assert_eq!(LiftFace::FineStructure.lift(1053), 1089);
    }

    #[test]
    fn trit_boundary_lift_is_b_minus_one_cubed() {
        assert_eq!(LiftFace::TritBoundary.value(), (B_INT - 1).pow(3));
    }
}
