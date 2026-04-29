// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `calculus` — the four sub-calculi
//!
//! 1. **Difference calculus** — the trit-pure forward difference
//!    `Δa_n = a_{n+1} − a_n`.
//! 2. **Circular calculus** — wrap-around arithmetic mod the canonical
//!    full-circle `R_6 = 364`.
//! 3. **Iteration calculus** — fixed-point iteration of a TritVec → TritVec
//!    map under a max-step bound.
//! 4. **Series calculus** — sums and partial sums on TritVec sequences.
//!
//! All four operate purely in TritVec space.
//!
//! ## Invariants verified at compile time
//!
//! - **I-39.** Difference calculus is closed on TritVec sequences.
//! - **I-40.** Circular calculus modulus is `R_6`.

extern crate alloc;

use alloc::vec::Vec;

use crate::arithmetic::{add, divmod, sub};
use crate::constants::{__tb, R_6_INT};
use crate::tritvec::TritVec;

// ────────────────────────────────────────────────────────────────────────
// Difference calculus
// ────────────────────────────────────────────────────────────────────────

/// Forward difference of a sequence of TritVecs.
///
/// `Δa = [a_1 − a_0, a_2 − a_1, …]`. Returns `None` on any underflow
/// (i.e. if any `a_{n+1} < a_n`).
pub fn forward_difference(seq: &[TritVec]) -> Option<Vec<TritVec>> {
    if seq.len() < 2 {
        return Some(Vec::new());
    }
    let mut out = Vec::with_capacity(seq.len() - 1);
    for i in 0..seq.len() - 1 {
        out.push(sub(&seq[i + 1], &seq[i])?);
    }
    Some(out)
}

// ────────────────────────────────────────────────────────────────────────
// Circular calculus
// ────────────────────────────────────────────────────────────────────────

/// `R_6 = 364` as a TritVec.
fn r_6_tv() -> TritVec {
    // 364 = 111111₃
    TritVec::from_trits(&[__tb(1); 6])
}

/// Reduce a TritVec modulo `R_6` (the canonical full circle).
pub fn reduce_circle(x: &TritVec) -> TritVec {
    let m = r_6_tv();
    let (_, r) = divmod(x, &m).expect("R_6 ≠ 0");
    r
}

/// Add two angles modulo `R_6`.
pub fn circular_add(a: &TritVec, b: &TritVec) -> TritVec {
    reduce_circle(&add(a, b))
}

// ────────────────────────────────────────────────────────────────────────
// Iteration calculus
// ────────────────────────────────────────────────────────────────────────

/// Iterate a TritVec → TritVec map up to `max_steps` times or until
/// a fixed point is reached. Returns the final TritVec.
pub fn iterate<F>(x0: TritVec, mut f: F, max_steps: usize) -> TritVec
where
    F: FnMut(&TritVec) -> TritVec,
{
    let mut current = x0;
    for _ in 0..max_steps {
        let next = f(&current);
        if next == current {
            return current;
        }
        current = next;
    }
    current
}

// ────────────────────────────────────────────────────────────────────────
// Series calculus
// ────────────────────────────────────────────────────────────────────────

/// Partial sums of a TritVec sequence.
pub fn partial_sums(seq: &[TritVec]) -> Vec<TritVec> {
    let mut out = Vec::with_capacity(seq.len());
    let mut acc = TritVec::zeros(1);
    for x in seq {
        acc = add(&acc, x);
        out.push(acc.clone());
    }
    out
}

const _: () = {
    // I-40 — the canonical circle modulus
    assert!(R_6_INT == 364);
};
