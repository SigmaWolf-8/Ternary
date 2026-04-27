// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `arc182` — the Σ-182 axis
//!
//! Five equivalent characterisations of the central framework number
//! `ARC = 182`:
//!
//! 1. `ARC = π · (π − 1) = 14·13`
//! 2. `ARC = R₆ / 2` (half the canonical circle)
//! 3. `ARC = 2 · p · R₃` (twice the prime-pair product)
//! 4. `ARC = 2 · Λ_EUV` (twice the EUV wavelength)
//! 5. `4·ARC + 1 = b⁶ = Δ_sponge` (sponge discriminant)
//!
//! ## Invariants verified at compile time
//!
//! - **I-11.** All five characterisations produce the same value.
//! - **I-12.** `1 + 4·ARC = b⁶`.

use crate::constants::{
    ARC_INT, B6_INT, DELTA_SPONGE_INT, LAMBDA_EUV_INT, P_INT, PI_INT, R_3_INT, R_6_INT,
};

/// The Σ-182 number itself.
pub const ARC: u64 = ARC_INT;

/// Five equivalent characterisations as a static `[u64; 5]`.
pub const FIVE_FORMS: [u64; 5] = [
    PI_INT * (PI_INT - 1),
    R_6_INT / 2,
    2 * P_INT * R_3_INT,
    2 * LAMBDA_EUV_INT,
    (B6_INT - 1) / 4,
];

const _: () = {
    // I-11 — all five forms equal ARC
    let i = 0;
    assert!(FIVE_FORMS[i] == ARC);
    assert!(FIVE_FORMS[1] == ARC);
    assert!(FIVE_FORMS[2] == ARC);
    assert!(FIVE_FORMS[3] == ARC);
    assert!(FIVE_FORMS[4] == ARC);
    // I-12 sponge discriminant
    assert!(1 + 4 * ARC == DELTA_SPONGE_INT);
    assert!(DELTA_SPONGE_INT == B6_INT);
};
