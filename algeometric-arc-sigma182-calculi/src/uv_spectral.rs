// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `uv_spectral` — UV system wavelength chain
//!
//! Four wavelengths anchor the framework's UV chain — Λ_EUV, λ_UVC,
//! λ_UVB, λ_UVA — each a multiple of `R_3 = 13` and each related to
//! `R_6 = 364` by simple integer factors.
//!
//! ## Invariants verified at compile time
//!
//! - **I-29.** Each UV band is a multiple of `R_3`.
//! - **I-31.** Specific values: `λ_LYMAN = R_6/4 = 91`, `λ_UVC = ARC = 182`,
//!   `λ_UVB = ARC_COPRIME = 286`, `λ_UVA = R_6 = 364`.

use crate::constants::{
    ARC_COPRIME_INT, ARC_INT, LAMBDA_LYMAN_INT, LAMBDA_UVA_INT, LAMBDA_UVB_INT, LAMBDA_UVC_INT,
    R_3_INT, R_6_INT,
};

/// The UV chain as a static array `[Λ_EUV, λ_UVC, λ_UVB, λ_UVA]`.
pub const UV_CHAIN: [u64; 4] = [
    LAMBDA_LYMAN_INT,
    LAMBDA_UVC_INT,
    LAMBDA_UVB_INT,
    LAMBDA_UVA_INT,
];

const _: () = {
    // I-29 — all multiples of R_3
    let mut i = 0;
    while i < 4 {
        assert!(UV_CHAIN[i] % R_3_INT == 0);
        i += 1;
    }
    // I-31 — specific pinned values
    assert!(LAMBDA_LYMAN_INT == R_6_INT / 4);
    assert!(LAMBDA_UVC_INT == ARC_INT);
    assert!(LAMBDA_UVB_INT == ARC_COPRIME_INT);
    assert!(LAMBDA_UVA_INT == R_6_INT);
};
