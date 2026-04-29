// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `dual_circle` — `Z_{Z_dual} ≅ Z_{b³} × Z_{2π}` CRT bijection
//!
//! The dual-circle structure says that the cyclic group on
//! `Z_dual = b³ · 2π = 27 · 28 = 756` positions decomposes as the
//! direct product `Z_{b³} × Z_{2π}` because `gcd(b³, 2π) = 1`.
//!
//! In framework terms: the `b³` Milesian register and the `2π`
//! radian-step register live on **independent** axes; CRT lets us
//! recover any position from its (Milesian, radian) residue pair.
//!
//! ## Invariants verified at compile time
//!
//! - **I-21.** `gcd(b³, 2π) = 1`, so `Z_{b³ · 2π} ≅ Z_{b³} × Z_{2π}`.

use crate::constants::{gcd_const, B3_INT, TWO_PI_INT};

/// Cardinality of the dual circle.
pub const Z_DUAL: u64 = B3_INT * TWO_PI_INT;

/// Project a dual position onto its `(milesian, radian)` residue pair.
pub const fn project(z: u64) -> (u64, u64) {
    (z % B3_INT, z % TWO_PI_INT)
}

const _: () = {
    // I-21
    assert!(gcd_const(B3_INT, TWO_PI_INT) == 1);
    assert!(Z_DUAL == 756);
    // Sanity: a few projections
    let (m, r) = project(0);
    assert!(m == 0 && r == 0);
    let (m, r) = project(Z_DUAL - 1);
    assert!(m == (Z_DUAL - 1) % B3_INT);
    assert!(r == (Z_DUAL - 1) % TWO_PI_INT);
};
