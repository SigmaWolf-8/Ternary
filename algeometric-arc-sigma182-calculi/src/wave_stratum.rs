// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `wave_stratum` — vacuum impedance Z₀ and the dual-layer encoding gap
//!
//! `Z_0 = 377` (vacuum impedance, framework units). The TL-Sponge
//! security parameter is `385 = 377 + 8`. The "wave–sponge gap" is
//! reduced to `Δ_wave_sponge = b = 3` after the dual-layer encoding
//! collapses two of the eight binary trims onto one trit.
//!
//! ## Invariants verified at compile time
//!
//! - **I-33.** `Z_0 = 377` and `Δ_wave_sponge = b`.

use crate::constants::{B_INT, DELTA_WAVE_SPONGE_INT, Z0_INT};

/// Vacuum impedance Ω, framework units.
pub const Z_0: u64 = Z0_INT;

/// The wave–sponge gap.
pub const DELTA_WAVE_SPONGE: u64 = DELTA_WAVE_SPONGE_INT;

const _: () = {
    assert!(Z_0 == 377);
    assert!(DELTA_WAVE_SPONGE == B_INT);
};
