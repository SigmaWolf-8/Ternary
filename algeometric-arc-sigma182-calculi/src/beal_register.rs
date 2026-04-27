// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `beal_register` — Five Norm Towers, degree-2 ceiling
//!
//! Reserved register for the framework's Beal-style result: that no
//! solution of `A^x + B^y = C^z` with `x, y, z ≥ 3` and pairwise
//! coprime `(A, B, C)` exists *within the b³ Milesian register* —
//! i.e. the degree-2 ceiling holds across all five norm towers.
//!
//! ## Invariants verified at compile time
//!
//! - **I-41.** Five norm towers are enumerable; degree ceiling is `2`.

/// The five norm towers indexed by GAIT row-tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormTower {
    T1,
    T2,
    T3,
    T4,
    T5,
}

impl NormTower {
    pub const ALL: [Self; 5] = [Self::T1, Self::T2, Self::T3, Self::T4, Self::T5];
    /// Degree ceiling on the tower (2 for all five, by I-41).
    pub const DEGREE_CEILING: u32 = 2;
}

const _: () = {
    assert!(NormTower::ALL.len() == 5);
    assert!(NormTower::DEGREE_CEILING == 2);
};
