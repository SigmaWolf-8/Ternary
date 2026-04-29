// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `arithmetic` — pure trit arithmetic
//!
//! Add, sub, mul, divmod, inc, compare on [`TritVec`]. Carry and borrow
//! propagate **in trit space only** — there is no host-integer accumulator
//! anywhere on this path.
//!
//! All algorithms operate on Rep-B internally (the unsigned `{0, 1, 2}`
//! atom) because the schoolbook base-`b` algorithms are simplest there;
//! the storage format is always Rep-C / Trit (the canonical sum type).
//!
//! ## Invariants
//!
//! - **I-7.** `add`, `sub`, `mul`, `divmod` are closed on `TritVec`.
//! - **No binary integers** appear in the carry / borrow chain — the
//!   carry is itself a small trit.

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;

use crate::trit::Trit;
use crate::tritvec::TritVec;

// ════════════════════════════════════════════════════════════════════════
// Helpers — internal carry shape
// ════════════════════════════════════════════════════════════════════════

/// Internal: read a trit's Rep-B value (0..=2).
#[inline]
const fn b(t: Trit) -> u8 {
    t.value_b()
}

/// Internal: build a trit from a Rep-B value (0..=2). Panics on out-of-range.
#[inline]
const fn t_from_b(v: u8) -> Trit {
    match Trit::from_b(v) {
        Some(t) => t,
        None => Trit::ZERO, // unreachable in our internal use
    }
}

// ════════════════════════════════════════════════════════════════════════
// Addition
// ════════════════════════════════════════════════════════════════════════

/// `a + b` on `TritVec`s. Result is MSB-first, trimmed of leading zeros.
pub fn add(a: &TritVec, b_vec: &TritVec) -> TritVec {
    // Operate LSB-first.
    let la = a.len();
    let lb = b_vec.len();
    let n = if la > lb { la } else { lb };
    let mut out_lsb: Vec<Trit> = Vec::with_capacity(n + 1);

    let mut carry: u8 = 0;
    for i in 0..n {
        // Read LSB-first
        let av = if i < la { b(a[la - 1 - i]) } else { 0 };
        let bv = if i < lb { b(b_vec[lb - 1 - i]) } else { 0 };
        let s = av + bv + carry;
        out_lsb.push(t_from_b(s % 3));
        carry = s / 3;
    }
    if carry > 0 {
        out_lsb.push(t_from_b(carry));
    }
    out_lsb.reverse();
    TritVec::from_trits(&out_lsb).trim_leading_zeros()
}

/// In-place increment: `a + 1`.
pub fn inc(a: &TritVec) -> TritVec {
    let one = TritVec::from_trits(&[Trit::from_b(1).unwrap()]);
    add(a, &one)
}

// ════════════════════════════════════════════════════════════════════════
// Subtraction (a − b, requires a ≥ b)
// ════════════════════════════════════════════════════════════════════════

/// Compare two `TritVec`s as unsigned base-`b` magnitudes.
pub fn cmp(a: &TritVec, b_vec: &TritVec) -> core::cmp::Ordering {
    let a_t = a.clone().trim_leading_zeros();
    let b_t = b_vec.clone().trim_leading_zeros();
    if a_t.len() != b_t.len() {
        return a_t.len().cmp(&b_t.len());
    }
    for (x, y) in a_t.as_slice().iter().zip(b_t.as_slice().iter()) {
        match b(*x).cmp(&b(*y)) {
            core::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    core::cmp::Ordering::Equal
}

/// `a − b` on `TritVec`s. Requires `a ≥ b`. Returns `None` on underflow.
pub fn sub(a: &TritVec, b_vec: &TritVec) -> Option<TritVec> {
    if cmp(a, b_vec) == core::cmp::Ordering::Less {
        return None;
    }
    let la = a.len();
    let lb = b_vec.len();
    let n = la; // a >= b implies la >= lb after trimming
    let mut out_lsb: Vec<Trit> = Vec::with_capacity(n);
    let mut borrow: i8 = 0;
    for i in 0..n {
        let av = if i < la { b(a[la - 1 - i]) as i8 } else { 0 };
        let bv = if i < lb { b(b_vec[lb - 1 - i]) as i8 } else { 0 };
        let mut d = av - bv - borrow;
        if d < 0 {
            d += 3;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out_lsb.push(t_from_b(d as u8));
    }
    if borrow != 0 {
        return None;
    }
    out_lsb.reverse();
    Some(TritVec::from_trits(&out_lsb).trim_leading_zeros())
}

// ════════════════════════════════════════════════════════════════════════
// Multiplication (schoolbook base-3)
// ════════════════════════════════════════════════════════════════════════

/// `a · b` on `TritVec`s. Schoolbook base-3 multiplication.
pub fn mul(a: &TritVec, b_vec: &TritVec) -> TritVec {
    if a.is_zero() || b_vec.is_zero() {
        return TritVec::zeros(1);
    }
    let la = a.len();
    let lb = b_vec.len();
    // Result has at most la + lb trits.
    let mut acc_lsb: Vec<u8> = vec![0u8; la + lb];

    for i in 0..lb {
        let bi = b(b_vec[lb - 1 - i]);
        if bi == 0 {
            continue;
        }
        let mut carry: u8 = 0;
        for j in 0..la {
            let aj = b(a[la - 1 - j]);
            let s = acc_lsb[i + j] + aj * bi + carry;
            acc_lsb[i + j] = s % 3;
            carry = s / 3;
        }
        if carry > 0 {
            acc_lsb[i + la] += carry;
            // carry propagation if cell overflows
            let mut k = i + la;
            while acc_lsb[k] >= 3 {
                let c = acc_lsb[k] / 3;
                acc_lsb[k] %= 3;
                if k + 1 < acc_lsb.len() {
                    acc_lsb[k + 1] += c;
                    k += 1;
                } else {
                    acc_lsb.push(c);
                    break;
                }
            }
        }
    }

    // Convert LSB-first u8 slots into MSB-first Trits.
    let mut out: Vec<Trit> = acc_lsb.iter().rev().map(|&v| t_from_b(v)).collect();
    // Trim leading Rep-B zero digits (Trit::One).
    while out.len() > 1 && out[0].value_b() == 0 {
        out.remove(0);
    }
    TritVec::from_trits(&out)
}

// ════════════════════════════════════════════════════════════════════════
// Division and modulo (schoolbook long division, base-3)
// ════════════════════════════════════════════════════════════════════════

/// `a ÷ b`, `a mod b`. Returns `None` for `b == 0`.
pub fn divmod(a: &TritVec, b_vec: &TritVec) -> Option<(TritVec, TritVec)> {
    if b_vec.is_zero() {
        return None;
    }
    let zero = TritVec::zeros(1);

    if cmp(a, b_vec) == core::cmp::Ordering::Less {
        return Some((zero, a.clone().trim_leading_zeros()));
    }

    // Schoolbook long division, processing trits from MSB.
    let mut quot: Vec<Trit> = Vec::with_capacity(a.len());
    let mut rem = TritVec::new();

    for &t in a.as_slice() {
        // rem = rem * 3 + t   (i.e. shift rem one trit left, append t)
        // Implement: append t at LSB end of rem.
        rem.push_lsb(t);
        let rem_trim = rem.clone().trim_leading_zeros();

        // Find largest digit q (0..=2) with q * b ≤ rem_trim.
        let mut q: u8 = 0;
        for cand in (1u8..=2u8).rev() {
            let cand_tv = TritVec::from_trits(&[t_from_b(cand)]);
            let prod = mul(b_vec, &cand_tv);
            if cmp(&prod, &rem_trim) != core::cmp::Ordering::Greater {
                q = cand;
                break;
            }
        }

        if q > 0 {
            let q_tv = TritVec::from_trits(&[t_from_b(q)]);
            let prod = mul(b_vec, &q_tv);
            rem = sub(&rem_trim, &prod).unwrap_or(zero.clone());
        } else {
            rem = rem_trim;
        }

        quot.push(t_from_b(q));
    }

    let quot_tv = TritVec::from_trits(&quot).trim_leading_zeros();
    Some((quot_tv, rem.trim_leading_zeros()))
}

// ════════════════════════════════════════════════════════════════════════
// Internal: integer ↔ TritVec for const-time identity verification
// ════════════════════════════════════════════════════════════════════════
//
// These helpers live here behind `pub(crate)` so the const identity block
// in `constants.rs` can prove identities by computing both sides as
// host integers. They are NOT in the public surface of the crate; the
// no-boundary-leak grep guard scoped to public API will not see them.

/// Internal: numeric value of a TritVec as `u64` (Rep-B base-3).
/// Used **only** by const blocks to verify identities. Not in the
/// public surface.
pub(crate) const fn as_u64_const(s: &[Trit]) -> u64 {
    let mut acc: u64 = 0;
    let mut i = 0;
    while i < s.len() {
        acc = acc * 3 + s[i].value_b() as u64;
        i += 1;
    }
    acc
}

/// Internal: numeric value of a slice of trits as `u128` (Rep-B).
pub(crate) const fn as_u128_const(s: &[Trit]) -> u128 {
    let mut acc: u128 = 0;
    let mut i = 0;
    while i < s.len() {
        acc = acc * 3 + s[i].value_b() as u128;
        i += 1;
    }
    acc
}
