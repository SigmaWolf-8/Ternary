// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `triangular_numbers` — `T_n = n·(n+1)/2`; Triple Identity at R₃
//!
//! ## Invariants verified at compile time
//!
//! - **I-19 (verifiable form).** `R_3 = TRI(R_2) + (R_2 − 1)
//!   = 10 + 3 = 13`.

use crate::constants::{R_2_INT, R_3_INT};

/// `T_n = n · (n + 1) / 2`.
#[inline]
pub const fn tri(n: u64) -> u64 {
    n * (n + 1) / 2
}

const _: () = {
    // I-19 (verifiable form)
    assert!(R_3_INT == tri(R_2_INT) + (R_2_INT - 1));
    assert!(tri(R_2_INT) == 10);
};
