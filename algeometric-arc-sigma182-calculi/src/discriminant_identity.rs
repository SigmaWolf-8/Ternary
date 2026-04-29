// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `discriminant_identity` — `(pqr − 1)² + 4·(p − r)² = 1 000 144`
//!
//! Canonical map address: **9.8.1.UX4.1** (NW quadrant).
//! Spec source: Inertissimum §3.7, eq 403 and eq 408 (`K = 2⁴·17·3677`).
//!
//! All numeric constants live in [`crate::constants`]:
//! `K_DISCRIMINANT_IDENTITY_INT`, `MASS_AXIS_PRIMITIVE_INT`,
//! `RESIDUAL_PRIME_INT`, `WALK_CLOCK_INT`, `PQR_ASYMMETRY_SQ_INT`,
//! `DISCRIMINANT_INT`, `B_INT`, `PI_INT`, `R_2_INT`, `R_3_INT`.

use crate::constants::{
    B_INT, DISCRIMINANT_INT, K_DISCRIMINANT_IDENTITY_INT,
    MASS_AXIS_PRIMITIVE_INT, PI_INT, PQR_ASYMMETRY_SQ_INT,
    RESIDUAL_PRIME_INT, R_2_INT, R_3_INT, WALK_CLOCK_INT,
};

/// The three-factor decomposition `(2⁴, 17, 3677)` of `K`.
#[inline]
pub const fn factorization() -> (u64, u64, u64) {
    (16, MASS_AXIS_PRIMITIVE_INT, RESIDUAL_PRIME_INT)
}

/// Const trial-division primality test, bounded for `RESIDUAL_PRIME_INT`.
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
    assert!(K_DISCRIMINANT_IDENTITY_INT == 1_000_144);
    assert!(K_DISCRIMINANT_IDENTITY_INT == WALK_CLOCK_INT * WALK_CLOCK_INT + 4 * PQR_ASYMMETRY_SQ_INT);
    assert!(K_DISCRIMINANT_IDENTITY_INT == WALK_CLOCK_INT * WALK_CLOCK_INT + DISCRIMINANT_INT);
    let (a, b, c) = factorization();
    assert!(a * b * c == K_DISCRIMINANT_IDENTITY_INT);
    assert!(a == 16);
    assert!(b == MASS_AXIS_PRIMITIVE_INT);
    assert!(c == RESIDUAL_PRIME_INT);
    assert!(MASS_AXIS_PRIMITIVE_INT == PI_INT + B_INT);
    assert!(MASS_AXIS_PRIMITIVE_INT == R_3_INT + R_2_INT);
    assert!(is_prime_const(RESIDUAL_PRIME_INT));
    assert!(is_prime_const(MASS_AXIS_PRIMITIVE_INT));
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_is_one_million_one_hundred_forty_four() {
        assert_eq!(K_DISCRIMINANT_IDENTITY_INT, 1_000_144);
    }

    #[test]
    fn factorization_multiplies_to_k() {
        let (a, b, c) = factorization();
        assert_eq!(a * b * c, K_DISCRIMINANT_IDENTITY_INT);
    }

    #[test]
    fn factorization_components_are_canonical() {
        assert_eq!(factorization(), (16, 17, 3677));
    }

    #[test]
    fn mass_axis_primitive_two_readings() {
        assert_eq!(MASS_AXIS_PRIMITIVE_INT, PI_INT + B_INT);
        assert_eq!(MASS_AXIS_PRIMITIVE_INT, R_3_INT + R_2_INT);
    }

    #[test]
    fn proton_mass_root_is_triangular_of_mass_axis() {
        let t17 = MASS_AXIS_PRIMITIVE_INT * (MASS_AXIS_PRIMITIVE_INT + 1) / 2;
        assert_eq!(t17, 153);
    }

    #[test]
    fn residual_prime_is_prime() {
        let n = RESIDUAL_PRIME_INT;
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
        assert_eq!(K_DISCRIMINANT_IDENTITY_INT, WALK_CLOCK_INT * WALK_CLOCK_INT + DISCRIMINANT_INT);
    }
}
