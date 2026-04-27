// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `hydrogen_spectral` — `R₆/4` bridge to `1/R∞`
//!
//! The framework Lyman boundary `λ_LYMAN = R₆/4 = 91` corresponds, in
//! continuous physics, to the Lyman series limit at `1/R∞` ≈ 91.18 nm.
//! The discrete framework value is the integer cousin of the
//! Rydberg-inverse limit.
//!
//! ## Invariants verified at compile time
//!
//! - **I-30.** Lyman boundary equals `R₆/4 = 91 = 7·R_3`.

use crate::constants::{LAMBDA_LYMAN_INT, R_3_INT, R_6_INT};

/// The Lyman boundary in framework units.
pub const LYMAN_BOUNDARY: u64 = LAMBDA_LYMAN_INT;

const _: () = {
    assert!(LYMAN_BOUNDARY == R_6_INT / 4);
    assert!(LYMAN_BOUNDARY == 7 * R_3_INT);
    assert!(LYMAN_BOUNDARY == 91);
};
