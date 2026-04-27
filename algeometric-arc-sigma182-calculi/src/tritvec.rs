// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `tritvec` — MSB-first vector of trits
//!
//! `TritVec` is the variable-length sequence type of the engine. Trits
//! are stored **MSB-first**: index `0` holds the most significant trit.
//! This matches the human-readable wire format and the Notation table's
//! Rep-C convention.
//!
//! There is no `from_u64`, no host-binary smuggling. The only public
//! constructors are pure-trit: from a slice/iterator of [`Trit`], or
//! from a slice of Rep-C bytes (which validates each byte against the
//! `{1, 2, 3}` alphabet). Conversion to host integers happens **only**
//! at the [`bridge`] feature boundary.
//!
//! ## Storage
//!
//! `TritVec` is a thin newtype around `alloc::vec::Vec<Trit>`. The
//! `core` + `alloc` build is sufficient — `std` is never required.
//!
//! ## Invariants
//!
//! - **I-1.** Each cell holds one [`Trit`] (three symbols).
//! - **I-2.** No `from_u64`/`to_u64`/`from_bytes`/`to_bytes` in this
//!   module's public surface; all such conversions live in [`bridge`]
//!   under the gated `bridge` feature.
//! - **MSB-first.** `tv[0]` is the most significant trit.
//!
//! [`bridge`]: crate::bridge

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use core::fmt;
use core::ops::{Deref, Index, IndexMut};

use crate::trit::Trit;

// ════════════════════════════════════════════════════════════════════════
// TritVec
// ════════════════════════════════════════════════════════════════════════

/// MSB-first vector of trits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TritVec {
    /// Storage. Index 0 is the most significant trit.
    trits: Vec<Trit>,
}

impl TritVec {
    /// Construct an empty `TritVec`.
    #[inline]
    pub fn new() -> Self {
        Self { trits: Vec::new() }
    }

    /// Construct a `TritVec` of length `n` filled with the **Rep-B
    /// zero digit** (`Trit::One` — i.e. base-3 numeral `0`).
    ///
    /// This is the right zero for arithmetic and repunit/positional
    /// contexts. For the GF(3) additive identity, build a TritVec of
    /// `Trit::ZERO` (= `Trit::Two`) explicitly.
    #[inline]
    pub fn zeros(n: usize) -> Self {
        Self {
            trits: vec![Trit::One; n],
        }
    }

    /// Construct from a slice of [`Trit`]s, MSB-first.
    #[inline]
    pub fn from_trits(trits: &[Trit]) -> Self {
        Self { trits: trits.to_vec() }
    }

    /// Construct from an iterator of [`Trit`]s, MSB-first.
    pub fn from_iter_msb<I: IntoIterator<Item = Trit>>(iter: I) -> Self {
        Self { trits: iter.into_iter().collect() }
    }

    /// Construct from a slice of Rep-C bytes (`{1, 2, 3}`), MSB-first.
    /// Returns `None` if any byte is outside the alphabet.
    pub fn from_rep_c(bytes: &[u8]) -> Option<Self> {
        let mut out = Vec::with_capacity(bytes.len());
        for &b in bytes {
            out.push(Trit::from_c(b)?);
        }
        Some(Self { trits: out })
    }

    /// Construct from a slice of Rep-B bytes (`{0, 1, 2}`), MSB-first.
    /// Returns `None` if any byte is outside the alphabet.
    pub fn from_rep_b(bytes: &[u8]) -> Option<Self> {
        let mut out = Vec::with_capacity(bytes.len());
        for &b in bytes {
            out.push(Trit::from_b(b)?);
        }
        Some(Self { trits: out })
    }

    /// Construct from a slice of Rep-A `i8`s (`{-1, 0, +1}`), MSB-first.
    /// Returns `None` if any value is outside the alphabet.
    pub fn from_rep_a(values: &[i8]) -> Option<Self> {
        let mut out = Vec::with_capacity(values.len());
        for &v in values {
            out.push(Trit::from_a(v)?);
        }
        Some(Self { trits: out })
    }

    /// Length in trits.
    #[inline]
    pub fn len(&self) -> usize {
        self.trits.len()
    }

    /// True iff length is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.trits.is_empty()
    }

    /// True iff the TritVec encodes the integer **zero** in Rep-B
    /// base-3 — i.e. every trit is the Rep-B digit `0` (`Trit::One`).
    /// An empty TritVec is also considered zero.
    pub fn is_zero(&self) -> bool {
        self.trits.iter().all(|t| t.value_b() == 0)
    }

    /// Slice view of the trits, MSB-first.
    #[inline]
    pub fn as_slice(&self) -> &[Trit] {
        &self.trits
    }

    /// Mutable slice view.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Trit] {
        &mut self.trits
    }

    /// Drop leading **Rep-B zero digits** (`Trit::One`). Returns
    /// `self` for chaining. Always preserves at least one trit.
    pub fn trim_leading_zeros(mut self) -> Self {
        while self.trits.first().map_or(false, |t| t.value_b() == 0) && self.trits.len() > 1 {
            self.trits.remove(0);
        }
        self
    }

    /// Pad on the high (MSB) side with zero trits to reach `target` length.
    ///
    /// If `self.len() >= target` this is a no-op.
    pub fn pad_msb_to(mut self, target: usize) -> Self {
        let cur = self.trits.len();
        if cur >= target {
            return self;
        }
        let pad = target - cur;
        let mut new = Vec::with_capacity(target);
        for _ in 0..pad {
            // Rep-B zero digit
            new.push(Trit::One);
        }
        new.append(&mut self.trits);
        self.trits = new;
        self
    }

    /// Append one trit on the LSB end.
    #[inline]
    pub fn push_lsb(&mut self, t: Trit) {
        self.trits.push(t);
    }

    /// Append one trit on the MSB end.
    #[inline]
    pub fn push_msb(&mut self, t: Trit) {
        self.trits.insert(0, t);
    }

    /// Reverse the trit order in place (turns MSB-first into LSB-first
    /// and vice versa). Used internally by arithmetic that consumes
    /// LSB-first carry chains.
    #[inline]
    pub fn reverse(&mut self) {
        self.trits.reverse();
    }

    /// Cloned reverse — returns a new `TritVec` in the opposite order.
    pub fn reversed(&self) -> Self {
        let mut t = self.trits.clone();
        t.reverse();
        Self { trits: t }
    }

    /// Equality of *value* (after stripping leading zeros). Useful for
    /// comparing TritVecs of different physical lengths.
    pub fn equal_values(&self, other: &Self) -> bool {
        self.clone().trim_leading_zeros() == other.clone().trim_leading_zeros()
    }
}

// ════════════════════════════════════════════════════════════════════════
// Standard trait impls
// ════════════════════════════════════════════════════════════════════════

impl Default for TritVec {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for TritVec {
    type Target = [Trit];
    #[inline]
    fn deref(&self) -> &[Trit] {
        &self.trits
    }
}

impl Index<usize> for TritVec {
    type Output = Trit;
    #[inline]
    fn index(&self, i: usize) -> &Trit {
        &self.trits[i]
    }
}

impl IndexMut<usize> for TritVec {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Trit {
        &mut self.trits[i]
    }
}

impl fmt::Display for TritVec {
    /// Display formats as MSB-first Rep-C numerals (e.g. `20202` for
    /// `ARC = 182 = 20202₃`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.trits.is_empty() {
            return write!(f, "0");
        }
        for t in &self.trits {
            write!(f, "{}", t.value_c())?;
        }
        Ok(())
    }
}

impl FromIterator<Trit> for TritVec {
    fn from_iter<I: IntoIterator<Item = Trit>>(iter: I) -> Self {
        Self { trits: iter.into_iter().collect() }
    }
}
