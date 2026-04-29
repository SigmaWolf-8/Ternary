// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `bridge` — bytes ↔ TritVec (gated `feature = "bridge"`)
//!
//! **The single legal trit ↔ binary boundary in the crate.** Everything
//! that converts between an `&[u8]` byte buffer and a [`TritVec`] lives
//! here. The rest of the crate operates on TritVec exclusively.
//!
//! Two encodings are provided:
//!
//! - **Rep-B byte encoding.** Each byte must be in `{0, 1, 2}`; one
//!   trit per byte, MSB-first.
//! - **Packed `b³` encoding.** Each byte holds three trits packed in
//!   Rep-B (`d_2 + 3·d_1 + 9·d_0`, big-endian within the byte), giving
//!   a 3:1 compression. Lengths are not multiples-of-3 padded on the
//!   MSB end with zero trits (the canonical convention).
//!
//! ## Invariants verified by tests
//!
//! - **I-3.** This module is the only file outside `host_*` shims to
//!   import `u8` for boundary I/O. The static guard
//!   `tests/no_boundary_leak.rs` enforces the rule.

extern crate alloc;

use alloc::vec::Vec;

use crate::trit::Trit;
use crate::tritvec::TritVec;

// ════════════════════════════════════════════════════════════════════════
// Rep-B byte encoding (one trit per byte)
// ════════════════════════════════════════════════════════════════════════

/// Encode a [`TritVec`] as a `Vec<u8>` of Rep-B bytes (`{0, 1, 2}`).
pub fn to_bytes_repb(t: &TritVec) -> Vec<u8> {
    t.as_slice().iter().map(|x| x.value_b()).collect()
}

/// Decode a slice of Rep-B bytes into a [`TritVec`]. Returns `None` if
/// any byte is outside `{0, 1, 2}`.
pub fn from_bytes_repb(bytes: &[u8]) -> Option<TritVec> {
    TritVec::from_rep_b(bytes)
}

// ════════════════════════════════════════════════════════════════════════
// Packed b³ encoding (three trits per byte)
// ════════════════════════════════════════════════════════════════════════

/// Encode a [`TritVec`] in packed `b³` form: 3 trits per byte,
/// big-endian within the byte. Pads on the MSB end with zero trits to
/// reach a multiple of 3.
pub fn to_bytes_packed(t: &TritVec) -> Vec<u8> {
    let pad = (3 - (t.len() % 3)) % 3;
    let mut padded = TritVec::zeros(pad);
    for &x in t.as_slice() {
        padded.push_lsb(x);
    }
    let mut out = Vec::with_capacity(padded.len() / 3);
    let mut i = 0;
    while i + 3 <= padded.len() {
        let d0 = padded.as_slice()[i].value_b() as u16;
        let d1 = padded.as_slice()[i + 1].value_b() as u16;
        let d2 = padded.as_slice()[i + 2].value_b() as u16;
        let byte = (d0 * 9 + d1 * 3 + d2) as u8;
        out.push(byte);
        i += 3;
    }
    out
}

/// Decode a packed-`b³` byte stream back to a [`TritVec`]. Each input
/// byte must be `< 27`; otherwise returns `None`. The high zero-pad
/// is preserved (caller may strip via [`TritVec::trim_leading_zeros`]).
pub fn from_bytes_packed(bytes: &[u8]) -> Option<TritVec> {
    let mut out: Vec<Trit> = Vec::with_capacity(bytes.len() * 3);
    for &b in bytes {
        if b >= 27 {
            return None;
        }
        let d0 = b / 9;
        let d1 = (b / 3) % 3;
        let d2 = b % 3;
        out.push(Trit::from_b(d0)?);
        out.push(Trit::from_b(d1)?);
        out.push(Trit::from_b(d2)?);
    }
    Some(TritVec::from_trits(&out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repb_roundtrip() {
        let t = TritVec::from_rep_b(&[2, 0, 2, 0, 2]).unwrap();
        let bytes = to_bytes_repb(&t);
        let back = from_bytes_repb(&bytes).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn packed_roundtrip_arc() {
        // ARC = 182 = 20202₃ MSB-first. 5 trits → padded to 6 with one
        // leading zero → 2 bytes.
        let t = TritVec::from_rep_b(&[2, 0, 2, 0, 2]).unwrap();
        let bytes = to_bytes_packed(&t);
        assert_eq!(bytes.len(), 2);
        let back = from_bytes_packed(&bytes).unwrap();
        // After trimming the leading-zero pad, we get the original.
        assert_eq!(back.trim_leading_zeros(), t);
    }

    #[test]
    fn packed_rejects_invalid_byte() {
        // Any byte ≥ 27 is invalid in packed b³.
        assert!(from_bytes_packed(&[27]).is_none());
        assert!(from_bytes_packed(&[255]).is_none());
    }

    #[test]
    fn repb_rejects_invalid_byte() {
        assert!(from_bytes_repb(&[3]).is_none());
    }
}
