// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `coprime_polygon_pair` — Generator Duality at (11, 13)
//!
//! Two regular polygons sit at the centre of the framework's
//! "Generator Duality Theorem": the 11-gon (`p_h`) and the 13-gon
//! (`R₃`). Their vertex sets, when interleaved palindromically,
//! produce **23 combined vertices** — exactly the I-22 number
//! `n − φ(n)` for `n = 11·13 = 143`.
//!
//! ## Invariants verified at compile time
//!
//! - **I-20.** `p_h` and `R_3` are pairwise coprime distinct primes.
//! - **I-22.** `n − φ(n) = p_h + R_3 − 1 = 23 = COMBINED_VERTICES`.
//! - **I-23.** Bézout: `p_h · 6 − R_3 · 5 = 1`.

use crate::constants::{COMBINED_VERTICES_INT, P_H_INT, R_1_INT, R_3_INT};
use crate::coprime::coprime;

/// Number of distinct vertices of the polygon-pair after interleaving.
pub const COMBINED_VERTICES: u64 = COMBINED_VERTICES_INT;

/// Bézout coefficients `(s, t)` so that `s·p_h + t·R_3 = 1`.
pub const fn bezout_coefficients_p_h_r_3() -> (i64, i64) {
    // s = 6, t = -5  (verified at compile time below)
    (6, -5)
}

/// Compute the combined-vertex count from primes `(p, q)`.
pub fn combined_vertices(p: u64, q: u64) -> Option<u64> {
    if !coprime(p, q) {
        return None;
    }
    Some(p + q - 1)
}

const _: () = {
    // I-20 — distinct, both > 1
    assert!(P_H_INT == 11);
    assert!(R_3_INT == 13);
    assert!(P_H_INT != R_3_INT);

    // I-22 — combined-vertex identity
    assert!(P_H_INT + R_3_INT - R_1_INT == COMBINED_VERTICES_INT);
    assert!(COMBINED_VERTICES_INT == 23);

    // I-23 — Bézout
    let (s, t) = bezout_coefficients_p_h_r_3();
    assert!(s * (P_H_INT as i64) + t * (R_3_INT as i64) == 1);
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coprime::totient;
    #[test]
    fn totient_at_143_is_120() {
        assert_eq!(totient(P_H_INT * R_3_INT), 120);
    }
}
