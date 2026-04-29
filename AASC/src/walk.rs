// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `walk` — coprime walk + CRT recovery
//!
//! A *Walk* over modulus `m` with stride `s` (both coprime) visits
//! every residue exactly once in `m` steps. Combined across the
//! coprime triple `(p, q, r)` and CRT-recovered, a triple of walks
//! yields a permutation of `0..p·q·r = 1001`.
//!
//! ## Invariants verified at compile time
//!
//! - **I-38.** `gcd(p, q) = gcd(p, r) = gcd(q, r) = 1` (re-asserted).

extern crate alloc;

use alloc::vec::Vec;

use crate::constants::{P_INT, Q_INT, R_INT};
use crate::coprime::{coprime, crt3, gcd, walk};

/// A triple-walk over the (p, q, r) coprime triple.
#[derive(Debug, Clone)]
pub struct TripleWalk {
    pub p_walk: Vec<u64>,
    pub q_walk: Vec<u64>,
    pub r_walk: Vec<u64>,
}

impl TripleWalk {
    /// Run the canonical triple-walk: stride 1 over each modulus,
    /// `m` steps per axis.
    pub fn canonical() -> Self {
        Self {
            p_walk: walk(P_INT, 1, P_INT as usize),
            q_walk: walk(Q_INT, 1, Q_INT as usize),
            r_walk: walk(R_INT, 1, R_INT as usize),
        }
    }

    /// CRT-recover the unique value `0 ≤ x < p·q·r` from a triple of
    /// residues.
    pub fn crt_recover(x_p: u64, x_q: u64, x_r: u64) -> Option<u64> {
        crt3(x_p, P_INT, x_q, Q_INT, x_r, R_INT)
    }
}

const _: () = {
    // I-38
    assert!(gcd(P_INT, Q_INT) == 1);
    assert!(gcd(P_INT, R_INT) == 1);
    assert!(gcd(Q_INT, R_INT) == 1);
    // strides 1 are coprime to the moduli trivially
    let _ = coprime;
};
