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
    // 0 = (empty in Milesian — divmod(0, 27) terminates immediately)
    // 1 = α   (position 0)
    // 27 = αα (position 1, then position 0): 27 = 1·27 + 0 → digits LSB [0, 1] → MSB "βα"  ?
    //
    // Actually in our LSB-first decomposition:
    //   27 mod 27 = 0 → α; 27 / 27 = 1 → 1 mod 27 = 1 → β; 1 / 27 = 0 → stop
    //   Digits LSB = [α, β]; MSB = "βα"
    // That's the canonical Milesian convention: most-significant glyph
    // appears first in the rendered string.
    let zero = TritVec::from_rep_b(&[0]).unwrap();
    assert_eq!(glyphs_msb(&zero), "");

    let one = TritVec::from_rep_b(&[1]).unwrap();
    assert_eq!(glyphs_msb(&one), "β"); // position 1 → β  (positions 0..26: α β γ δ ε ϛ ζ η θ ι κ λ μ ν ξ ο π ϟ ρ σ τ υ φ χ ψ ω ϡ)
                                        // Wait: position 0 = α, position 1 = β. Value 1 = position 1?
                                        // No: value 1 mod 27 = 1 → MilesianDigit{position: 1} → glyph β.
                                        // Hmm — that's a Milesian convention question. The Greek
                                        // alphabet starts at α = 1, so position 0 should be α
                                        // representing value 0, position 1 = β representing value 1.
                                        // We follow the "value-equals-position" convention here.
}
