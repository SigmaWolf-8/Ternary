// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `milesian` — `divmod(b³)` over the b³-glyph table
//!
//! The Milesian register has `b³ = 27` glyphs — one per Greek-alphabet
//! position (with the three ghost letters reinstated). Any TritVec
//! decomposes into a sequence of Milesian glyphs by repeated division
//! by `b³`:
//!
//! ```text
//!     digit_i = n  mod b³
//!     n       = n  div b³
//! ```
//!
//! At the boundary the engine emits a `&'static str` glyph from the
//! `b³`-glyph table. The boundary call `digit.to_index()` is the **one
//! permitted Rep-B-narrowing call** in this module (I-3).

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::arithmetic::{cmp, divmod};
use crate::constants::{__tb, B3_INT};
use crate::tritvec::TritVec;

/// The b³-glyph table. 27 entries, indexed `0..=26` by Milesian
/// position. Glyphs are the canonical Greek alphabet positions
/// extended with the three ghost letters at positions 6 (digamma),
/// 18 (qoppa), and 26 (sampi).
pub const GLYPH_TABLE: [&str; 27] = [
    "α", "β", "γ", "δ", "ε", "ϛ", // 6 = digamma (ghost)
    "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ϟ", // 18 = qoppa (ghost)
    "ρ", "σ", "τ", "υ", "φ", "χ", "ψ", "ω", "ϡ", // 27 = sampi (ghost)
];

/// One decoded Milesian digit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilesianDigit {
    /// Position in the b³ register, `0..=26`.
    position: u8,
}

impl MilesianDigit {
    /// Construct from an in-range position. Returns `None` outside
    /// `0..27`.
    #[inline]
    pub const fn new(position: u8) -> Option<Self> {
        if position < 27 {
            Some(Self { position })
        } else {
            None
        }
    }

    /// **The single permitted Rep-B-narrowing call** — turn a digit
    /// into its `usize` table index (I-3).
    #[inline]
    pub const fn to_index(self) -> usize {
        self.position as usize
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

/// Decompose a non-negative TritVec into Milesian digits, **least-
/// significant first**. Each digit is a value in `0..b³`.
pub fn digits_lsb(n: &TritVec) -> Vec<MilesianDigit> {
    let mut out = Vec::new();
    let zero = TritVec::zeros(1);
    let base = b3_tv();
    let mut current = n.clone().trim_leading_zeros();

    while cmp(&current, &zero) == core::cmp::Ordering::Greater {
        let (q, r) = divmod(&current, &base).expect("b³ ≠ 0");
        // r is a TritVec representing 0..=26 in Rep-B base-3.
        // Convert via the at-most-3-trit window to a u8 ≤ 26.
        let r_val = small_repb_to_u8(&r);
        // Boundary narrow: r_val < b³, fits a u8.
        let d = MilesianDigit::new(r_val as u8).expect("r < b³");
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
    // 27 entries
    assert!(GLYPH_TABLE.len() == 27);
};
