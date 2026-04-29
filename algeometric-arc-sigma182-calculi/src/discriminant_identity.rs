// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `discriminant_identity` — `(pqr − 1)² + 4·(p − r)² = 1 000 144`
//!
//! Canonical map address: **9.8.1.UX4.1** (NW quadrant, spectral band).
//!
//! Spec source: `Inertissimum Iώτα Nona — Codex Unificationis`,
//! §3.7 (Discriminant Identity), equation 403.
//!
//! ## What this module exposes
//!
//! The Walk departure squared plus four times the fine-structure
//! residue fuse into a single constant:
//!
//! ```text
//! (pqr − 1)² + 4·(p − r)²  =  1000² + 4·36  =  1 000 144
//! ```
//!
//! and that constant has the prime factorization
//!
//! ```text
//! 1 000 144  =  2⁴ · 17 · 3677
//! ```
//!
//! The factor `17 = π_geom + b = R₃ + R₂` is the mass-axis primitive
//! (its triangular number `T(17) = 153` is the proton-mass root cited
//! throughout the Compendium). The factor `3677` is prime and openly
//! noted; it is the residual structural prime of this identity.
//!
//! ## Invariants verified at compile time
//!
//! - `K = (pqr − 1)² + 4·(p − r)² = 1 000 144` (Inertissimum eq 403).
//! - `K = 2⁴ · 17 · 3677` (Inertissimum eq 408).
//! - `K = 1000² + 144` (Walk-clock squared plus the discriminant).
//! - `17 = π_geom + b = R₃ + R₂` (mass-axis primitive).
//! - `3677` is prime (verified by trial division up to `⌊√3677⌋ = 60`).
//! - The identity is **independent of the fine-structure constant**;
//!   it is a pure consequence of the closed walk and the circle
//!   geometry.

use crate::constants::{B_INT, P_INT, PI_INT, PQR_INT, R_2_INT, R_3_INT, R_INT};

/// The discriminant-identity constant: `(pqr − 1)² + 4·(p − r)² = 1 000 144`.
pub const K: u64 = (PQR_INT - 1) * (PQR_INT - 1) + 4 * (R_INT - P_INT) * (R_INT - P_INT);

/// First prime factor exponent: `K = 2⁴ · ...`.
pub const FACTOR_2_EXPONENT: u32 = 4;

/// Mass-axis primitive: `17 = π_geom + b = R₃ + R₂`.
pub const MASS_AXIS_PRIMITIVE: u64 = 17;

/// Residual structural prime of the identity: `3677`.
pub const RESIDUAL_PRIME: u64 = 3677;

/// The three-factor decomposition `(2⁴, 17, 3677)`.
#[inline]
pub const fn factorization() -> (u64, u64, u64) {
    (1u64 << FACTOR_2_EXPONENT, MASS_AXIS_PRIMITIVE, RESIDUAL_PRIME)
}

/// Const trial-division primality test (bounded; intended for the
/// small residual prime `3677`).
#[allow(dead_code)]
const fn is_prime_const(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut d: u64 = 3;
    while d * d <= n {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

const _: () = {
    // Eq 403: the identity itself
    assert!(K == 1_000_144);

    // Decomposition into the walk-clock-squared + Δ form
    assert!(K == 1000 * 1000 + 144);

    // Eq 408: prime factorization 2⁴ · 17 · 3677
    let (a, b, c) = factorization();
    assert!(a * b * c == K);
    assert!(a == 16);
    assert!(b == MASS_AXIS_PRIMITIVE);
    assert!(c == RESIDUAL_PRIME);

    // Mass-axis primitive identities: 17 = π_geom + b = R₃ + R₂
    assert!(MASS_AXIS_PRIMITIVE == PI_INT + B_INT);
    assert!(MASS_AXIS_PRIMITIVE == R_3_INT + R_2_INT);

    // Residual prime is in fact prime
    assert!(is_prime_const(RESIDUAL_PRIME));

    // Mass-axis primitive is prime (sanity)
    assert!(is_prime_const(MASS_AXIS_PRIMITIVE));
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_is_one_million_one_hundred_forty_four() {
        assert_eq!(K, 1_000_144);
    }

    #[test]
    fn factorization_multiplies_to_k() {
        let (a, b, c) = factorization();
        assert_eq!(a * b * c, K);
    }

    #[test]
    fn factorization_components_are_canonical() {
        assert_eq!(factorization(), (16, 17, 3677));
    }

    #[test]
    fn mass_axis_primitive_two_readings() {
        assert_eq!(MASS_AXIS_PRIMITIVE, PI_INT + B_INT);
        assert_eq!(MASS_AXIS_PRIMITIVE, R_3_INT + R_2_INT);
    }

    #[test]
    fn proton_mass_root_is_triangular_of_mass_axis() {
        // T(17) = 17·18/2 = 153 — the proton-mass root cited
        // throughout the Compendium.
        let t17 = MASS_AXIS_PRIMITIVE * (MASS_AXIS_PRIMITIVE + 1) / 2;
        assert_eq!(t17, 153);
    }

    #[test]
    fn residual_prime_is_prime() {
        // Runtime check (mirrors the const-eval check above).
        let n = RESIDUAL_PRIME;
        let mut d = 3u64;
        let mut prime = n >= 2 && (n == 2 || n == 3 || n % 2 == 1);
        while prime && d * d <= n {
            if n % d == 0 {
                prime = false;
            }
            d += 2;
        }
        assert!(prime);
    }

    #[test]
    fn identity_is_walk_clock_squared_plus_delta() {
        // K = 1000² + 144 — the walk-clock squared plus the discriminant.
        let walk_clock = PQR_INT - 1;
        let delta = 144u64;
        assert_eq!(K, walk_clock * walk_clock + delta);
    }
}
