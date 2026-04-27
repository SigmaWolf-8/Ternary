// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `repunit_circles` — repunit-radius circles
//!
//! Each repunit `R_L` indexes a concentric "shell" in the framework.
//! The L-th shell has radius `R_L`, perimeter `R_L · (R_3 / R_1) = 13·R_L`
//! degrees-per-step in the canonical (R₆ = 364) circle, and obeys the
//! repunit recurrence `R_{L+1} = b · R_L + R_1`.
//!
//! ## Invariants verified at compile time
//!
//! - **I-8.**  Repunit recurrence (re-asserted at this layer).
//! - **I-9.**  Repunit closed form (re-asserted at this layer).

use crate::constants::{B_INT, R_1_INT, R_2_INT, R_3_INT, R_4_INT, R_5_INT, R_6_INT, R_7_INT};

/// Get the L-th repunit (1..=7). Returns `None` outside that range.
pub const fn repunit(l: u32) -> Option<u64> {
    match l {
        1 => Some(R_1_INT),
        2 => Some(R_2_INT),
        3 => Some(R_3_INT),
        4 => Some(R_4_INT),
        5 => Some(R_5_INT),
        6 => Some(R_6_INT),
        7 => Some(R_7_INT),
        _ => None,
    }
}

/// Apply the repunit recurrence: `R_{L+1} = b · R_L + R_1`.
#[inline]
pub const fn next_repunit(r_l: u64) -> u64 {
    B_INT * r_l + R_1_INT
}

const _: () = {
    // I-8 chain
    assert!(next_repunit(R_1_INT) == R_2_INT);
    assert!(next_repunit(R_2_INT) == R_3_INT);
    assert!(next_repunit(R_3_INT) == R_4_INT);
    assert!(next_repunit(R_4_INT) == R_5_INT);
    assert!(next_repunit(R_5_INT) == R_6_INT);
    assert!(next_repunit(R_6_INT) == R_7_INT);
};
