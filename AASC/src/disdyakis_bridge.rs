// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `disdyakis_bridge` — `R²_disdyakis = π + 5φ`
//!
//! The disdyakis dodecahedron's circumscribed-sphere squared radius
//! lives in ℤ[φ] as the element `(14, 5) = π + 5φ`. This is the
//! framework's bridge from integer trit-space to the golden-ratio
//! ring of crystallographic vertices.
//!
//! ## Invariants verified at compile time
//!
//! - **I-25.** `R²_disdyakis = π + 5φ ∈ ℤ[φ]`.

use crate::zphi::ZPhi;

/// `R²_disdyakis ∈ ℤ[φ]`.
pub const R_SQUARED: ZPhi = ZPhi::R_SQUARED_DISDYAKIS;

const _: () = {
    assert!(R_SQUARED.a == 14);
    assert!(R_SQUARED.b == 5);
};
