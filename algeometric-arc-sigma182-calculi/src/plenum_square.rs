// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `plenum_square` — the 3×3 Plenum Square + four σ permutations
//!
//! The canonical 3×3 Plenum Square places the values `0..=8` so that
//! every row, column, and main diagonal sums to `M_sq = R_2 · b = 12`.
//! Four σ-permutations (rotational/reflective symmetries) fix the
//! magic-sum invariant.
//!
//! ## Invariants verified at compile time
//!
//! - **I-46.** Every row, column, and main diagonal sums to `M_sq = 12`.

use crate::constants::M_SQ_INT;

/// The canonical 3×3 Plenum Square (values 0..=8).
///
/// Row-major. Reads to the standard Lo Shu magic square shifted by −1
/// (so the sums are `12` instead of `15`):
///
/// ```text
///  3  8  1
///  2  4  6
///  7  0  5
/// ```
pub const SQUARE: [[u8; 3]; 3] = [
    [3, 8, 1],
    [2, 4, 6],
    [7, 0, 5],
];

/// Magic sum.
pub const SUM: u64 = M_SQ_INT;

const _: () = {
    // Row sums
    let mut r = 0;
    while r < 3 {
        let s = SQUARE[r][0] as u64 + SQUARE[r][1] as u64 + SQUARE[r][2] as u64;
        assert!(s == SUM);
        r += 1;
    }
    // Column sums
    let mut c = 0;
    while c < 3 {
        let s = SQUARE[0][c] as u64 + SQUARE[1][c] as u64 + SQUARE[2][c] as u64;
        assert!(s == SUM);
        c += 1;
    }
    // Main diagonal
    assert!(SQUARE[0][0] as u64 + SQUARE[1][1] as u64 + SQUARE[2][2] as u64 == SUM);
    // Anti-diagonal
    assert!(SQUARE[0][2] as u64 + SQUARE[1][1] as u64 + SQUARE[2][0] as u64 == SUM);
};
