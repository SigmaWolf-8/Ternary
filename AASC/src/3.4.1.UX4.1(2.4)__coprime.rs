// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `coprime` — coprimality, totient, walks, CRT
//!
//! Pure number-theoretic primitives over framework integers. The
//! engine carries three pairwise-coprime moduli — `p = 7`, `q = 11`,
//! `r = R₃ = 13` — whose product `PQR = 1001` enables Chinese
//! Remainder Theorem (CRT) recovery from a triple of residues.
//!
//! Functions in this module operate on `u64` residues for arithmetic
//! density. The TritVec wrappers in [`crate::walk`] convert at the
//! caller's request.
//!
//! ## Invariants verified at compile time
//!
//! - **I-14.** `gcd(p,q) = gcd(p,r) = gcd(q,r) = 1`.
//! - **I-22.** `n − φ(n) = p + q − 1` for `n = p·q` with `p, q` distinct
//!   primes (verified at the (11, 13) point).

use crate::constants::{P_H_INT, P_INT, Q_INT, R_3_INT, R_INT};

/// Euclidean GCD.
pub const fn gcd(a: u64, b: u64) -> u64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}

/// True iff `gcd(a, b) == 1`.
#[inline]
pub const fn coprime(a: u64, b: u64) -> bool {
    gcd(a, b) == 1
}

/// Euler totient `φ(n)` for `n` in `1..=u32::MAX`. Trial-division
/// implementation — adequate for framework moduli (≤ a few thousand).
pub fn totient(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut result = n;
    let mut nn = n;
    let mut p: u64 = 2;
    while p * p <= nn {
        if nn % p == 0 {
            while nn % p == 0 {
                nn /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if nn > 1 {
        result -= result / nn;
    }
    result
}

/// Naive CRT recovery from a triple of residues `(x_p, x_q, x_r)`
/// with coprime moduli `(p, q, r)`. Returns the unique `0 ≤ x < p·q·r`.
pub fn crt3(x_p: u64, p: u64, x_q: u64, q: u64, x_r: u64, r: u64) -> Option<u64> {
    if !coprime(p, q) || !coprime(p, r) || !coprime(q, r) {
        return None;
    }
    let n = p * q * r;
    // Brute-force search: for the framework moduli (n = 1001) this is
    // perfectly fast and avoids any extended-Euclid complexity.
    for x in 0..n {
        if x % p == x_p % p && x % q == x_q % q && x % r == x_r % r {
            return Some(x);
        }
    }
    None
}

/// Coprime walk of length `n` over modulus `m` with stride `s`.
///
/// Yields `[0, s mod m, 2s mod m, …, (n−1)s mod m]`. The walk visits
/// all `m` residues exactly once iff `gcd(s, m) = 1`.
pub fn walk(m: u64, s: u64, n: usize) -> alloc::vec::Vec<u64> {
    let mut out = alloc::vec::Vec::with_capacity(n);
    let mut acc: u64 = 0;
    for _ in 0..n {
        out.push(acc);
        acc = (acc + s) % m;
    }
    out
}

// ════════════════════════════════════════════════════════════════════════
// Compile-time invariants
// ════════════════════════════════════════════════════════════════════════

const _: () = {
    // I-14 — pairwise coprimality of (p, q, r)
    assert!(gcd(P_INT, Q_INT) == 1);
    assert!(gcd(P_INT, R_INT) == 1);
    assert!(gcd(Q_INT, R_INT) == 1);

    // I-22 — for n = p_h · R_3, n − φ(n) = p_h + R_3 − 1 = 23
    let n: u64 = P_H_INT * R_3_INT;
    let phi_n: u64 = (P_H_INT - 1) * (R_3_INT - 1); // both prime
    assert!(n - phi_n == P_H_INT + R_3_INT - 1);
    assert!(n - phi_n == 23);
};
