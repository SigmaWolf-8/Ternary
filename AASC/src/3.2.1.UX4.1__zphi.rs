// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `zphi` — ℤ[φ] golden-ratio integer ring
//!
//! Integer combinations `a + b·φ` with the minimal polynomial
//! `φ² = φ + 1`. The disdyakis bridge identity
//!
//! ```text
//!     R²_disdyakis = π + 5φ
//! ```
//!
//! lives in this ring (with `R²` interpreted as the framework
//! squared-radius constant). The ring's host integers `(a, b)` are
//! kept here as a deliberate, narrow boundary — this module is the
//! one place the framework's algebra crosses into `i64` for ℤ[φ]
//! arithmetic — but every element exposed in the public API has a
//! TritVec mirror for compositional use upstream.
//!
//! ## Invariants verified at compile time
//!
//! - **I-25.** Disdyakis identity `R² = π + 5φ` realises in ℤ[φ] as
//!   the element `(14, 5)`.
//! - **I-26.** Multiplication closure: `(a + bφ)·(c + dφ) =
//!   (ac + bd) + (ad + bc + bd)φ`.

/// An element of ℤ[φ]: `a + b·φ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZPhi {
    /// Rational integer part.
    pub a: i64,
    /// Coefficient of φ.
    pub b: i64,
}

impl ZPhi {
    /// Construct `a + b·φ`.
    #[inline]
    pub const fn new(a: i64, b: i64) -> Self {
        Self { a, b }
    }

    /// Additive identity.
    pub const ZERO: Self = Self::new(0, 0);
    /// Multiplicative identity.
    pub const ONE: Self = Self::new(1, 0);
    /// φ itself.
    pub const PHI: Self = Self::new(0, 1);

    /// Addition.
    #[inline]
    pub const fn add(self, other: Self) -> Self {
        Self::new(self.a + other.a, self.b + other.b)
    }

    /// Subtraction.
    #[inline]
    pub const fn sub(self, other: Self) -> Self {
        Self::new(self.a - other.a, self.b - other.b)
    }

    /// Multiplication using `φ² = φ + 1`.
    #[inline]
    pub const fn mul(self, other: Self) -> Self {
        // (a + bφ)(c + dφ) = ac + (ad+bc)φ + bd·φ² = ac + (ad+bc)φ + bd(φ+1)
        //                  = (ac + bd) + (ad + bc + bd) φ
        Self::new(
            self.a * other.a + self.b * other.b,
            self.a * other.b + self.b * other.a + self.b * other.b,
        )
    }

    /// Galois norm `N(a + bφ) = a² + ab − b²`.
    #[inline]
    pub const fn norm(self) -> i64 {
        self.a * self.a + self.a * self.b - self.b * self.b
    }

    /// The disdyakis squared-radius element: `π + 5φ` with `π = 14`.
    pub const R_SQUARED_DISDYAKIS: Self = Self::new(14, 5);
}

// ════════════════════════════════════════════════════════════════════════
// I-25 / I-26 — compile-time identities
// ════════════════════════════════════════════════════════════════════════

const _: () = {
    use crate::constants::PI_INT;

    // I-25 — element shape
    let r2 = ZPhi::R_SQUARED_DISDYAKIS;
    assert!(r2.a == PI_INT as i64);
    assert!(r2.b == 5);

    // I-26 — multiplication via φ² = φ + 1
    // (1 + 1·φ) · (1 + 1·φ) = (1·1 + 1·1) + (1·1 + 1·1 + 1·1)φ = 2 + 3φ
    let phi_plus_1 = ZPhi::new(1, 1);
    let p = phi_plus_1.mul(phi_plus_1);
    assert!(p.a == 2);
    assert!(p.b == 3);

    // φ · φ = φ + 1   (the minimal polynomial)
    let phi_sq = ZPhi::PHI.mul(ZPhi::PHI);
    assert!(phi_sq.a == 1);
    assert!(phi_sq.b == 1);

    // Norm of φ is −1
    assert!(ZPhi::PHI.norm() == -1);
};
