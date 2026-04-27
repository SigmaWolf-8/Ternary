// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `milesian` — bijective base-`b³` over the Milesian glyph table
//!
//! The Milesian register has `b³ = 27` glyphs — one per Greek-alphabet
//! position **1..=27** with the three ghost letters reinstated
//! (digamma at position 6, qoppa at position 18, sampi at position 27).
//!
//! Per Spec v3.3.33 §4.5 the glyph string is the **bijective base-27
//! representation** of `N`. Bijective base-`b` uses digits `1..=b`
//! (no zero digit), so the natural numeral line is:
//!
//! ```text
//!     N = 0  → ""           (empty)
//!     N = 1  → α            (digit 1)
//!     N = 27 → ϡ            (digit 27 = sampi)
//!     N = 28 → αα           (digits LSB [1, 1])
//! ```
//!
//! The decomposition recurrence is the canonical bijective form:
//!
//! ```text
//!     while n > 0:
//!         (q, r) = divmod(n, b³)
//!         if r == 0:
//!             digit = b³          (= 27, ghost-letter ϡ)
//!             q     = q - 1       (carry the borrow)
//!         else:
//!             digit = r
//!         emit digit; n = q
//! ```
//!
//! At the boundary the engine emits a `&'static str` glyph from the
//! `b³`-glyph table. The boundary call `digit.to_index()` is the **one
//! permitted Rep-B-narrowing call** in this module (I-3).

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::arithmetic::{cmp, divmod, sub};
use crate::constants::{__tb, B3_INT};
use crate::tritvec::TritVec;

/// The b³-glyph table. 27 entries, indexed `0..=26` by `(position - 1)`.
/// Glyphs are the canonical Greek alphabet positions 1..=27 extended
/// with the three ghost letters: digamma (position 6), qoppa
/// (position 18), sampi (position 27).
pub const GLYPH_TABLE: [&str; 27] = [
    "α", "β", "γ", "δ", "ε", "ϛ", // position 6 = digamma (ghost)
    "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ϟ", // position 18 = qoppa (ghost)
    "ρ", "σ", "τ", "υ", "φ", "χ", "ψ", "ω", "ϡ", // position 27 = sampi (ghost)
];

/// One decoded Milesian digit.
///
/// `position` is the **1-indexed Milesian position**, in `1..=27`,
/// matching Spec v3.3.33 §1 (`MilesianGlyph` carries position 1..27).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilesianDigit {
    position: u8,
}

impl MilesianDigit {
    /// Construct from an in-range Milesian position `1..=27`.
    /// Returns `None` for `0` or anything `> 27`.
    #[inline]
    pub const fn new(position: u8) -> Option<Self> {
        if position >= 1 && position <= 27 {
            Some(Self { position })
        } else {
            None
        }
    }

    /// The 1..=27 Milesian-position integer.
    #[inline]
    pub const fn position(self) -> u8 {
        self.position
    }

    /// **The single permitted Rep-B-narrowing call** — turn a digit
    /// into its `usize` table index, `(position - 1)` (I-3).
    #[inline]
    pub const fn to_index(self) -> usize {
        (self.position - 1) as usize
    }

    /// Look up the glyph for this digit.
    #[inline]
    pub fn glyph(self) -> &'static str {
        GLYPH_TABLE[self.to_index()]
    }
}

/// `b³` as a TritVec (`[1, 0, 0, 0]` in Rep-B base-3).
fn b3_tv() -> TritVec {
    TritVec::from_trits(&[__tb(1), __tb(0), __tb(0), __tb(0)])
}

/// `1` as a TritVec (`[1]` in Rep-B base-3).
fn one_tv() -> TritVec {
    TritVec::from_trits(&[__tb(1)])
}

/// Decompose a non-negative TritVec into Milesian digits, **least-
/// significant first**. Each digit carries a 1..=27 Milesian position
/// (bijective base-27 per Spec v3.3.33 §4.5).
pub fn digits_lsb(n: &TritVec) -> Vec<MilesianDigit> {
    let mut out = Vec::new();
    let zero = TritVec::zeros(1);
    let base = b3_tv();
    let one = one_tv();
    let mut current = n.clone().trim_leading_zeros();

    while cmp(&current, &zero) == core::cmp::Ordering::Greater {
        let (mut q, r) = divmod(&current, &base).expect("b³ ≠ 0");
        // `r` is a TritVec representing 0..=26 in Rep-B base-3.
        let r_val = small_repb_to_u8(&r);
        // Bijective adjustment: r == 0 means "carry the borrow" — the
        // emitted digit is the full base (27 = sampi, ϡ) and q is
        // decremented by 1.
        let digit_val: u8 = if r_val == 0 {
            q = sub(&q, &one).expect("q ≥ 1 when r == 0 because n > 0");
            27
        } else {
            r_val as u8
        };
        // Boundary narrow: digit_val ∈ 1..=27, fits a u8.
        let d = MilesianDigit::new(digit_val).expect("1 ≤ digit ≤ 27");
        out.push(d);
        current = q;
    }
    out
}

/// Decompose into Milesian glyphs (most-significant first).
pub fn glyphs_msb(n: &TritVec) -> String {
    let mut digits = digits_lsb(n);
    digits.reverse();
    let mut s = String::new();
    for d in digits {
        s.push_str(d.glyph());
    }
    s
}

/// Internal: read a TritVec known to fit `< b³` as a `u32`. Relies
/// on the digit-bound guarantee for I-3 compliance.
fn small_repb_to_u8(t: &TritVec) -> u32 {
    let mut acc: u32 = 0;
    for &x in t.as_slice() {
        acc = acc * (B3_INT as u32 / 9) + x.value_b() as u32; // (B3_INT/9) = 3
    }
    acc
}

const _: () = {
    // 27 entries — one per Milesian position 1..=27.
    assert!(GLYPH_TABLE.len() == 27);
};
