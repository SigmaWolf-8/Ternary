// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `gait` — Greek Atomic Invariant Tracker
//!
//! GAIT operates on the b³-symbol Milesian register. Each Milesian
//! glyph carries a *Greek alphabet position*, a *numeric value*
//! (Milesian), and an *atomic number* — a triple whose sum,
//! across the register, yields the **cumulative delta**
//!
//! ```text
//!     Σ̃ = b³ · α⁻¹_int = 27 · 137 = 3699
//! ```
//!
//! This module exposes the value-type of the entry and the cumulative
//! delta constant for the const identity block.
//!
//! ## Invariants verified at compile time
//!
//! - **I-44.** `Σ̃ = b³ · α⁻¹_int = 3699`.

use crate::constants::{ALPHA_INV_INT, B3_INT, SIGMA_TILDE_INT};

/// One row of the GAIT register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GaitEntry {
    /// Milesian glyph position in the b³ register, `0..=26`.
    pub position: u8,
    /// Greek-alphabet position (1..=27 with ghost letters reinstated).
    pub greek_position: u8,
    /// Milesian numeric value (1..=900 across the three b-decades).
    pub milesian: u32,
    /// Atomic number (1..=118 for the analogue mapping).
    pub atomic: u8,
}

/// Cumulative delta: `Σ̃ = b³ · α⁻¹_int`.
pub const SIGMA_TILDE: u128 = SIGMA_TILDE_INT;

/// Inverse fine-structure integer used by GAIT (`α⁻¹_int = 137`).
pub const ALPHA_INV: u128 = ALPHA_INV_INT;

const _: () = {
    // I-44
    assert!(SIGMA_TILDE == (B3_INT as u128) * ALPHA_INV);
    assert!(SIGMA_TILDE == 3699);
};
