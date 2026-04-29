// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `repx` — bijective base-`b` converter (Spec v3.3.33 §4)
//!
//! Bijective base-`b` numeration uses the digit alphabet
//! `{1, 2, …, b}` instead of the standard `{0, 1, …, b−1}`. For
//! `b = 3` the alphabet is exactly `{1, 2, 3}` — and *that is the
//! Rep-C trit alphabet itself*. So a TritVec read as **Rep-C** is a
//! bijective base-3 number; a TritVec read as **Rep-B** is a standard
//! base-3 number. `repx` converts between the two without ever
//! leaving trit space.
//!
//! ## Algorithm — standard → bijective (LSB-first construction)
//!
//! Repeat while `n > 0`:
//!
//! ```text
//!     d = ((n − 1) mod b) + 1
//!     emit  d
//!     n = (n − d) / b
//! ```
//!
//! Both `mod`, `sub`, and `div` are pure TritVec operations from
//! [`crate::arithmetic`].
//!
//! ## Invariants verified by tests
//!
//! - **I-37.** `to_bijective` and `from_bijective` are mutual inverses
//!   on every non-negative integer ≤ a few thousand (covered by the
//!   spec-conformance fixture and unit tests).

extern crate alloc;

use alloc::vec::Vec;

use crate::arithmetic::{add, cmp, divmod, mul, sub};
use crate::constants::__tb;
use crate::trit::Trit;
use crate::tritvec::TritVec;

/// The base, as a single-trit `TritVec` `[3]` in Rep-C terms — but in
/// **Rep-B integer arithmetic** (which is what `arithmetic` uses) it's
/// `[1, 0]` because base-3 of 3 is "10".
fn base_tv() -> TritVec {
    // 3 in Rep-B base-3 = "10"
    TritVec::from_trits(&[__tb(1), __tb(0)])
}

/// One in standard form.
fn one_tv() -> TritVec {
    TritVec::from_trits(&[__tb(1)])
}

/// Convert a standard-base-3 [`TritVec`] (Rep-B digits `{0, 1, 2}`) into
/// the bijective-base-3 form (Rep-C digits `{1, 2, 3}`), MSB-first.
///
/// Returns the empty TritVec when the input value is zero — bijective
/// numeration has no canonical representation of zero.
pub fn to_bijective(n: &TritVec) -> TritVec {
    if n.is_zero() {
        return TritVec::new();
    }
    let mut digits_lsb: Vec<Trit> = Vec::new();
    let mut current = n.clone().trim_leading_zeros();
    let zero = TritVec::zeros(1);
    let one = one_tv();
    let base = base_tv();

    while cmp(&current, &zero) == core::cmp::Ordering::Greater {
        // d = ((n − 1) mod b) + 1   ∈ {1, 2, 3}
        let nm1 = sub(&current, &one).unwrap_or_else(|| zero.clone());
        let (_, rem) = divmod(&nm1, &base).expect("base ≠ 0");

        // rem is a single Rep-B trit value 0, 1, or 2.
        let d_minus_1: u8 = if rem.is_empty() {
            0
        } else {
            // Single-trit value (rem has at most 1 trit since base = 3).
            rem.as_slice().last().map(|t| t.value_b()).unwrap_or(0)
        };
        let d_b: u8 = d_minus_1 + 1; // 1, 2, or 3 — Rep-C atom

        // Emit Rep-C digit. Storage is Rep-C ⇒ Trit::from_c(d_b).
        let trit_c = Trit::from_c(d_b).expect("d_b ∈ {1,2,3}");
        digits_lsb.push(trit_c);

        // n ← (n − d) / b   where d here means the integer d_b.
        // Build d as a single-trit Rep-B TritVec (value d_b ∈ 1..=3).
        let d_int_tv: TritVec = if d_b <= 2 {
            TritVec::from_trits(&[Trit::from_b(d_b).expect("≤ 2")])
        } else {
            // 3 in Rep-B = "10"
            base_tv()
        };
        let n_minus_d = sub(&current, &d_int_tv).unwrap_or_else(|| zero.clone());
        let (q, _) = divmod(&n_minus_d, &base).expect("base ≠ 0");
        current = q;
    }

    digits_lsb.reverse();
    TritVec::from_trits(&digits_lsb)
}

/// Convert a bijective-base-3 [`TritVec`] (digits stored as Rep-C
/// trits, MSB-first) back into the standard-base-3 form.
pub fn from_bijective(b: &TritVec) -> TritVec {
    if b.is_empty() {
        return TritVec::zeros(1);
    }
    let base = base_tv();
    let mut acc = TritVec::zeros(1);
    for &t in b.as_slice() {
        // acc = acc · 3
        acc = mul(&acc, &base);
        // d = Rep-C value (1..=3) as a Rep-B integer TritVec
        let d_c = t.value_c();
        let d_tv: TritVec = if d_c <= 2 {
            TritVec::from_trits(&[Trit::from_b(d_c).expect("≤ 2")])
        } else {
            base_tv()
        };
        acc = add(&acc, &d_tv);
    }
    acc.trim_leading_zeros()
}
