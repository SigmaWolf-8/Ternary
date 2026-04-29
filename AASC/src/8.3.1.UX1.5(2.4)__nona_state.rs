// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `nona_state` — the Nona State and the transfer equation
//!
//! The Nona State is the 9-cell magic-square state of the Plenum
//! Square, indexed by row-major position `0..=8`. The conservation
//! identity says any σ-permutation preserves the magic sum
//! `M_sq = R_2 · b = 12`.
//!
//! ## Invariants verified at compile time
//!
//! - **I-35.** Nine cells, magic sum `12`. (Sum closure was already
//!   verified in [`crate::plenum_square`]; this module re-pins the
//!   cardinality and the sum.)

use crate::constants::M_SQ_INT;
use crate::plenum_square::SUM;

/// Number of Nona-State cells.
pub const NONA_CELLS: usize = 9;

const _: () = {
    assert!(SUM == M_SQ_INT);
    assert!(NONA_CELLS == 9);
};
