// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `wieferich_register` — first base-3 Wieferich prime is `p_h = 11`
//!
//! Wieferich primes in base `b` are primes `p` such that
//! `b^(p−1) ≡ 1 (mod p²)`. In base 3, the smallest known
//! such prime is `p_h = 11`. The framework register pins this fact.
//!
//! ## Invariants verified at compile time
//!
//! - **I-42.** `3^10 ≡ 1 (mod 121)`.

use crate::constants::P_H_INT;

const _: () = {
    let p = P_H_INT;
    let p2 = p * p;
    // Compute 3^(p-1) mod p² by repeated squaring.
    let mut acc: u64 = 1;
    let base: u64 = 3;
    let mut exp: u64 = p - 1;
    let mut b = base % p2;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = (acc * b) % p2;
        }
        exp >>= 1;
        b = (b * b) % p2;
    }
    assert!(acc == 1);
};
