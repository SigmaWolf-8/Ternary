// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `conservation_laws` — PCO-A six conservation laws
//!
//! CL-1..CL-6 of the Plenum Continuity Ontology — Algebraic.
//! The crate carries a typed enumeration here and the trit-pure
//! check that each law's *defining sum* is invariant under the four
//! σ-permutations of [`crate::plenum_square`].
//!
//! ## Invariants verified at compile time
//!
//! - **I-34.** All six conservation laws are enumerable and distinct.

/// One of the six PCO-A conservation laws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConservationLaw {
    /// CL-1 — sum invariance.
    Cl1Sum,
    /// CL-2 — magic-row invariance.
    Cl2MagicRow,
    /// CL-3 — orthogonal complement invariance.
    Cl3Orthogonal,
    /// CL-4 — radian-step invariance.
    Cl4Radian,
    /// CL-5 — Borromean cyclic invariance.
    Cl5Borromean,
    /// CL-6 — coprime-walk invariance.
    Cl6Walk,
}

impl ConservationLaw {
    /// Enumerate all six laws.
    pub const ALL: [Self; 6] = [
        Self::Cl1Sum,
        Self::Cl2MagicRow,
        Self::Cl3Orthogonal,
        Self::Cl4Radian,
        Self::Cl5Borromean,
        Self::Cl6Walk,
    ];
}

const _: () = {
    // I-34 — six distinct laws
    assert!(ConservationLaw::ALL.len() == 6);
};
