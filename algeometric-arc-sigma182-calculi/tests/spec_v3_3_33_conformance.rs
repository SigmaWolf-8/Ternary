// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// Spec v3.3.33 conformance fixture. Worked examples for the bijective
// base-3 (repx) converter and the Milesian glyph stringer.
//
// The fixture is a tiny TSV table embedded as a string literal; no
// external parser dep needed.

use aasc::milesian::glyphs_msb;
use aasc::repx::{from_bijective, to_bijective};
use aasc::tritvec::TritVec;

extern crate alloc;
use alloc::vec::Vec;

/// Spec v3.3.33 §4 worked examples — `(decimal, std-base-3-MSB,
/// bijective-base-3-MSB-as-Rep-C)`.
const REPX_FIXTURES: &[(u64, &str, &str)] = &[
    // Trivial single-digit values
    (1, "1", "1"),
    (2, "2", "2"),
    (3, "10", "3"),
    (4, "11", "11"),
    (5, "12", "12"),
    (6, "20", "13"),
    (7, "21", "21"),
    (8, "22", "22"),
    (9, "100", "23"),
    (10, "101", "31"),
    // The Σ-182 axis pin
    (182, "20202", "13132"),
    // The half-circle PQR product
    (1001, "1101002", "323232"),
];

fn parse_repb(s: &str) -> TritVec {
    let bytes: Vec<u8> = s.bytes().map(|c| c - b'0').collect();
    TritVec::from_rep_b(&bytes).unwrap()
}

fn parse_repc(s: &str) -> TritVec {
    let bytes: Vec<u8> = s.bytes().map(|c| c - b'0').collect();
    TritVec::from_rep_c(&bytes).unwrap()
}

fn render_repc(t: &TritVec) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for x in t.as_slice() {
        s.push((b'0' + x.value_c()) as char);
    }
    s
}

fn render_repb(t: &TritVec) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for x in t.as_slice() {
        s.push((b'0' + x.value_b()) as char);
    }
    s
}

#[test]
fn spec_v3_3_33_repx_to_bijective() {
    for &(decimal, std_str, bij_str) in REPX_FIXTURES {
        let std = parse_repb(std_str);
        let got_bij = to_bijective(&std);
        let want_bij = parse_repc(bij_str);
        assert_eq!(
            render_repc(&got_bij),
            render_repc(&want_bij),
            "to_bijective({}) — std = `{}`, expected bijective `{}`, got `{}`",
            decimal,
            std_str,
            bij_str,
            render_repc(&got_bij),
        );
    }
}

#[test]
fn spec_v3_3_33_repx_from_bijective() {
    for &(decimal, std_str, bij_str) in REPX_FIXTURES {
        let bij = parse_repc(bij_str);
        let got_std = from_bijective(&bij);
        let want_std = parse_repb(std_str);
        assert_eq!(
            render_repb(&got_std.clone().trim_leading_zeros()),
            render_repb(&want_std),
            "from_bijective({}) — bij = `{}`, expected std `{}`, got `{}`",
            decimal,
            bij_str,
            std_str,
            render_repb(&got_std),
        );
    }
}

#[test]
fn spec_v3_3_33_milesian_first_few_glyphs() {
    // Spec v3.3.33 §4.5 — the glyph string is the bijective base-27
    // representation of N over the Milesian alphabet (positions 1..27,
    // with ghosts at 6 = digamma, 18 = qoppa, 27 = sampi).
    //
    //   N = 0  → ""           (no digits)
    //   N = 1  → "α"          (position 1)
    //   N = 6  → "ϛ"          (position 6 = digamma, ghost)
    //   N = 18 → "ϟ"          (position 18 = qoppa, ghost)
    //   N = 26 → "ω"          (position 26)
    //   N = 27 → "ϡ"          (position 27 = sampi, ghost)
    //   N = 28 → "αα"         (28 = 1·27 + 1 → bijective digits LSB [1,1])
    //   N = 54 → "αϡ"         (54 = 1·27 + 27 → bijective digits LSB [27,1])
    let cases: &[(u8, &str)] = &[
        (0, ""),
        (1, "α"),
        (6, "ϛ"),
        (18, "ϟ"),
        (26, "ω"),
        (27, "ϡ"),
        (28, "αα"),
        (54, "αϡ"),
    ];
    for &(n, expected) in cases {
        let v = TritVec::from_rep_b(&decimal_to_repb(n as u64)).unwrap();
        assert_eq!(
            glyphs_msb(&v),
            expected,
            "milesian glyph mismatch for N = {n}: expected `{expected}`",
        );
    }
}

/// Render a `u64` as the MSB-first base-3 (Rep-B) digit sequence used
/// by `TritVec::from_rep_b`. `0` → `[0]`.
fn decimal_to_repb(mut n: u64) -> Vec<u8> {
    if n == 0 {
        return alloc::vec![0];
    }
    let mut digits: Vec<u8> = Vec::new();
    while n > 0 {
        digits.push((n % 3) as u8);
        n /= 3;
    }
    digits.reverse();
    digits
}
