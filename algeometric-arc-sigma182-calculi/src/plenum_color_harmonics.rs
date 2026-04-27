// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `plenum_color_harmonics` — the four harmonic arcs
//!
//! ARC_RED, ARC_BLUE, ARC_COPRIME, √Δ_arc, ARC_GREEN. Each is a
//! function of the (`π`, `R₃`, `p_h`, `b⁵`) generators.
//!
//! ## Invariants verified at compile time
//!
//! - **I-24.** The four PlenumColor harmonic equalities:
//!   - (a) `ARC_BLUE = 2·φ(p_h·R_3) = b⁵ − b`
//!   - (b) `√Δ_arc = ARC_RED + ARC_COPRIME = 36·R_3`
//!   - (c) `ARC_GREEN = R_6 + ARC_COPRIME = ARC_RED + √Δ_arc`
//!   - (d) `ARC_COPRIME − ARC_BLUE = 2·COMBINED_VERTICES = 46`

use crate::constants::{
    ARC_BLUE_INT, ARC_COPRIME_INT, ARC_GREEN_INT, ARC_RED_INT, B5_INT, B_INT,
    COMBINED_VERTICES_INT, R_3_INT, R_6_INT, SQRT_DELTA_ARC_INT,
};

/// The five harmonic arcs of the PlenumColor system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlenumColors {
    pub red: u64,
    pub blue: u64,
    pub coprime: u64,
    pub sqrt_delta: u64,
    pub green: u64,
}

impl PlenumColors {
    /// The canonical, framework-pinned PlenumColors.
    pub const CANONICAL: Self = Self {
        red: ARC_RED_INT,
        blue: ARC_BLUE_INT,
        coprime: ARC_COPRIME_INT,
        sqrt_delta: SQRT_DELTA_ARC_INT,
        green: ARC_GREEN_INT,
    };
}

const _: () = {
    // I-24 — the four equalities
    // (a) ARC_BLUE = b⁵ − b
    assert!(ARC_BLUE_INT == B5_INT - B_INT);
    // (b) √Δ_arc = ARC_RED + ARC_COPRIME = 36·R_3
    assert!(SQRT_DELTA_ARC_INT == ARC_RED_INT + ARC_COPRIME_INT);
    assert!(SQRT_DELTA_ARC_INT == 36 * R_3_INT);
    // (c) ARC_GREEN closure
    assert!(ARC_GREEN_INT == R_6_INT + ARC_COPRIME_INT);
    assert!(ARC_GREEN_INT == ARC_RED_INT + SQRT_DELTA_ARC_INT);
    // (d) ARC_COPRIME − ARC_BLUE = 2·COMBINED_VERTICES
    assert!(ARC_COPRIME_INT - ARC_BLUE_INT == 2 * COMBINED_VERTICES_INT);
    assert!(2 * COMBINED_VERTICES_INT == 46);
};
