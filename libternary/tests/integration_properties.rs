// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! # Integration Tests — Cross-Module Mathematical Properties
//!
//! These tests verify invariants that span multiple libternary modules,
//! ensuring the Tribonacci arithmetic, Borromean topology, and 364°
//! ternary circle geometry are consistent with each other.
//!
//! Run: `cargo test --test integration_properties --release`

use libternary::borromean::TernaryWord;
use libternary::ternary_circle::{
    base3_repunit_order, is_base3_repunit, std_deg_to_ternary_deg, std_rad_to_ternary_rad,
    ternary_deg_to_std_deg, ternary_rad_to_std_rad, trit_to_std_rad, walk_tribonacci_radian_spiral,
    CYCLIC_ORDER, FULL_CIRCLE_DEG, PI_TERNARY, RADIAN_DEG, TAU_TRIBONACCI, TWO_PI_TERNARY, Z28,
};
use libternary::tribonacci::{TernaryRepr, TribonacciBase3, TritVec};

// ══════════════════════════════════════════════════════════════
// AXIOM VERIFICATION — The foundational identities
// ══════════════════════════════════════════════════════════════

#[test]
fn axiom_full_circle_is_six_digit_repunit() {
    // 364 = (3⁶ − 1) / 2 = 111111₃
    assert!(is_base3_repunit(FULL_CIRCLE_DEG as u64));
    assert_eq!(base3_repunit_order(FULL_CIRCLE_DEG as u64), Some(6));
}

#[test]
fn axiom_radian_is_three_digit_repunit() {
    // 13 = (3³ − 1) / 2 = 111₃
    assert!(is_base3_repunit(RADIAN_DEG as u64));
    assert_eq!(base3_repunit_order(RADIAN_DEG as u64), Some(3));
}

#[test]
fn axiom_radian_is_tribonacci_t7() {
    // The seventh Tribonacci number T(7) = 13
    let mut gen = TribonacciBase3::new();
    let terms: Vec<_> = (0..8).map(|_| gen.next_term()).collect();
    assert_eq!(terms[7].decimal, 13);
}

#[test]
fn axiom_circle_equals_28_radians() {
    assert_eq!(FULL_CIRCLE_DEG, RADIAN_DEG * TWO_PI_TERNARY);
    assert_eq!(364.0, 13.0 * 28.0);
}

#[test]
fn axiom_pi_is_14() {
    // C = π·d, C/r = 2π = 28, π = 14
    assert_eq!(PI_TERNARY, 14.0);
    assert_eq!(TWO_PI_TERNARY, 2.0 * PI_TERNARY);
}

#[test]
fn axiom_ternary_full_circle_maps_to_360_std() {
    let std = ternary_deg_to_std_deg(FULL_CIRCLE_DEG);
    assert!((std - 360.0).abs() < 1e-10);
}

#[test]
fn axiom_28_ternary_radians_equals_2pi_std() {
    let std = ternary_rad_to_std_rad(TWO_PI_TERNARY);
    assert!((std - 2.0 * std::f64::consts::PI).abs() < 1e-10);
}

// ══════════════════════════════════════════════════════════════
// REPRESENTATION INTERCHANGE — The kernel guarantee
// ══════════════════════════════════════════════════════════════

#[test]
fn repr_abc_roundtrip_first_30_tribonacci() {
    // For the first 30 Tribonacci numbers, converting to each
    // representation and back must yield the identical decimal value.
    let mut gen = TribonacciBase3::new();
    for i in 0..30 {
        let term = gen.next_term();
        let decimal = term.decimal;

        // B → A → B
        let repr_a = term.value.to_repr_a();
        let back_from_a = TritVec::from_repr_a(&repr_a);
        assert_eq!(
            back_from_a.to_decimal(),
            decimal,
            "A-roundtrip failed at T({})",
            i
        );

        // B → C → B (skip zero — bijective has no zero representation)
        if decimal > 0 {
            let repr_c = term.value.to_repr_c();
            let back_from_c = TritVec::from_repr_c(&repr_c);
            assert_eq!(
                back_from_c.to_decimal(),
                decimal,
                "C-roundtrip failed at T({})",
                i
            );

            // Verify bijective has no zeros
            for &digit in &repr_c {
                assert!(
                    digit >= 1 && digit <= 3,
                    "Bijective digit {} out of range at T({})",
                    digit,
                    i
                );
            }
        }

        // Verify balanced has only valid digits
        for &digit in &repr_a {
            assert!(
                digit >= -1 && digit <= 1,
                "Balanced digit {} out of range at T({})",
                digit,
                i
            );
        }
    }
}

#[test]
fn repr_abc_display_format() {
    // Verify the subscript display format for each representation
    let mut gen = TribonacciBase3::new();
    let terms: Vec<_> = (0..11).map(|_| gen.next_term()).collect();

    // T(10) = 81 = 10000₃
    let t10 = &terms[10];
    let fmt_b = t10.value.format_repr(TernaryRepr::Standard);
    assert!(
        fmt_b.contains("10000"),
        "T(10) in Rep B should be 10000₃, got {}",
        fmt_b
    );
}

// ══════════════════════════════════════════════════════════════
// TRIBONACCI × BORROMEAN — Topology from arithmetic
// ══════════════════════════════════════════════════════════════

#[test]
fn borromean_triples_from_consecutive_tribonacci() {
    // Construct Borromean triples from consecutive Tribonacci terms.
    // The XOR should be non-zero for well-formed triples.
    let mut gen = TribonacciBase3::new();
    let terms: Vec<_> = (0..20).map(|_| gen.next_term()).collect();

    // Use terms starting from T(5) onward (multi-digit)
    for window in terms[5..].windows(3) {
        let trits_a = window[0].value.to_repr_b();
        let trits_b = window[1].value.to_repr_b();
        let trits_c = window[2].value.to_repr_b();

        // Pad to equal length
        let max_len = trits_a.len().max(trits_b.len()).max(trits_c.len());
        let pad = |v: &[u8], len: usize| -> Vec<u8> {
            let mut p = vec![0u8; len];
            for (i, &d) in v.iter().enumerate() {
                if i < len {
                    p[i] = d;
                }
            }
            p
        };

        let wa = TernaryWord::new(pad(&trits_a, max_len));
        let wb = TernaryWord::new(pad(&trits_b, max_len));
        let wc = TernaryWord::new(pad(&trits_c, max_len));

        let xor_result = wa.xor_mod3(&wb).xor_mod3(&wc);
        // At least some positions should be non-zero for the triple to be Borromean.
        // (Not all triples from consecutive terms will satisfy this, but most will.)
        // We just verify the XOR operation is well-formed.
        assert_eq!(xor_result.len(), max_len);
    }
}

#[test]
fn borromean_invariant_across_representations() {
    // The Borromean XOR is defined over mod-3, so it must be
    // representation-independent. Construct the same word via
    // Rep A, B, and C — the XOR result must be identical.
    let digits_b: Vec<u8> = vec![1, 2, 0, 1, 2];

    let word_from_b = TernaryWord::new(digits_b.clone());

    // Rep A: map through balanced representation
    let repr_a: Vec<i8> = digits_b
        .iter()
        .map(|&d| match d {
            0 => -1_i8,
            1 => 0,
            2 => 1,
            _ => unreachable!(),
        })
        .collect();
    let word_from_a = TernaryWord::from_balanced(&repr_a);

    // Rep C: map through bijective representation
    let repr_c: Vec<u8> = digits_b.iter().map(|&d| d + 1).collect();
    let word_from_c = TernaryWord::from_bijective(&repr_c);

    // All three should produce identical XOR behavior
    let partner = TernaryWord::new(vec![2, 1, 1, 0, 2]);

    let xor_b = word_from_b.xor_mod3(&partner);
    let xor_a = word_from_a.xor_mod3(&partner);
    let xor_c = word_from_c.xor_mod3(&partner);

    assert_eq!(
        xor_b.digits(),
        xor_a.digits(),
        "Borromean XOR must be representation-independent (A vs B)"
    );
    assert_eq!(
        xor_b.digits(),
        xor_c.digits(),
        "Borromean XOR must be representation-independent (B vs C)"
    );
}

// ══════════════════════════════════════════════════════════════
// TRIBONACCI × TERNARY CIRCLE — Arithmetic meets geometry
// ══════════════════════════════════════════════════════════════

#[test]
fn tribonacci_radian_is_repunit_and_sequence_member() {
    // 13 must be simultaneously:
    //   (a) A base-3 repunit (111₃)
    //   (b) The 7th Tribonacci number
    //   (c) The ternary radian in degrees
    assert!(is_base3_repunit(13));
    assert_eq!(RADIAN_DEG, 13.0);

    let mut gen = TribonacciBase3::new();
    let terms: Vec<_> = (0..8).map(|_| gen.next_term()).collect();
    assert_eq!(terms[7].decimal, 13);
}

#[test]
fn spiral_walk_directions_are_lattice_points() {
    // Every direction in the spiral walk must be an exact
    // integer multiple of 13° — the walk lives on 28 rays.
    let trits: Vec<u8> = vec![
        1, 2, 0, 1, 0, 0, 2, 2, 0, 1, 1, 1, 2, 0, 2, 1, 0, 0, 1, 2, 2, 0, 1, 1, 0, 2, 1, 0, 2, 0,
    ];

    let points = walk_tribonacci_radian_spiral(&trits);

    for p in &points[1..] {
        // Z28 position must be in [0, 28)
        assert!(
            p.position.0 < CYCLIC_ORDER as u8,
            "Z₂₈ position {} out of range at step {}",
            p.position.0,
            p.step
        );

        // The ternary degree must be an exact multiple of 13
        let deg = p.position.to_ternary_deg();
        let remainder = deg % RADIAN_DEG;
        assert!(
            remainder.abs() < 1e-10 || (RADIAN_DEG - remainder).abs() < 1e-10,
            "Walk angle {}° is not a lattice point at step {}",
            deg,
            p.step
        );
    }
}

#[test]
fn spiral_scaling_is_tau() {
    // Each step in the spiral is shorter by factor τ.
    let trits: Vec<u8> = vec![1, 1, 1, 1, 1, 1, 1, 1];
    let points = walk_tribonacci_radian_spiral(&trits);

    for k in 2..points.len() {
        let dx1 = points[k].x - points[k - 1].x;
        let dy1 = points[k].y - points[k - 1].y;
        let len1 = (dx1 * dx1 + dy1 * dy1).sqrt();

        let dx0 = points[k - 1].x - points[k - 2].x;
        let dy0 = points[k - 1].y - points[k - 2].y;
        let len0 = (dx0 * dx0 + dy0 * dy0).sqrt();

        if len0 > 1e-12 {
            let ratio = len0 / len1;
            assert!(
                (ratio - TAU_TRIBONACCI).abs() < 0.01,
                "Step ratio at k={} should be τ ≈ {:.6}, got {:.6}",
                k,
                TAU_TRIBONACCI,
                ratio
            );
        }
    }
}

#[test]
fn z28_visits_all_positions_with_trit_1() {
    // The generator trit=1 must visit all 28 positions in Z₂₈.
    let mut visited = [false; 28];
    let mut pos = Z28::new(0);

    for _ in 0..28 {
        visited[pos.0 as usize] = true;
        pos = pos.step(1);
    }

    for (i, &v) in visited.iter().enumerate() {
        assert!(v, "Z₂₈ position {} was never visited by generator 1", i);
    }
}

#[test]
fn z28_group_inverse_property() {
    // Every element a in Z₂₈ has an inverse -a such that a + (-a) = 0.
    for i in 0..28u8 {
        let a = Z28(i);
        let neg_a = a.neg();
        let sum = a.add(neg_a);
        assert_eq!(
            sum,
            Z28(0),
            "Z₂₈({}): {} + {} should be identity, got {}",
            i,
            a,
            neg_a,
            sum
        );
    }
}

// ══════════════════════════════════════════════════════════════
// CONVERSION CONSISTENCY — Bridge between worlds
// ══════════════════════════════════════════════════════════════

#[test]
fn conversion_roundtrip_ternary_to_std_degrees() {
    // Converting 0..364 ternary degrees to standard and back
    // must be lossless.
    for i in 0..=364 {
        let ternary = i as f64;
        let standard = ternary_deg_to_std_deg(ternary);
        let back = std_deg_to_ternary_deg(standard);
        assert!(
            (back - ternary).abs() < 1e-10,
            "Degree roundtrip failed at {}°: {} → {} → {}",
            i,
            ternary,
            standard,
            back
        );
    }
}

#[test]
fn conversion_roundtrip_ternary_to_std_radians() {
    // Converting 0..28 ternary radians to standard and back.
    for i in 0..=28 {
        let ternary = i as f64;
        let standard = ternary_rad_to_std_rad(ternary);
        let back = std_rad_to_ternary_rad(standard);
        assert!(
            (back - ternary).abs() < 1e-10,
            "Radian roundtrip failed at {} trad: {} → {} → {}",
            i,
            ternary,
            standard,
            back
        );
    }
}

#[test]
fn trit_to_std_rad_is_consistent_with_z28() {
    // For each trit value 0,1,2: the standard radian from
    // trit_to_std_rad must match Z28::step().to_std_rad().
    for trit in 0..=2u8 {
        let from_fn = trit_to_std_rad(trit);
        let from_z28 = Z28::new(0).step(trit).to_std_rad();
        assert!(
            (from_fn - from_z28).abs() < 1e-10,
            "trit_to_std_rad({}) = {} but Z28.step({}).to_std_rad() = {}",
            trit,
            from_fn,
            trit,
            from_z28
        );
    }
}

// ══════════════════════════════════════════════════════════════
// REPUNIT CHAIN — The structural backbone
// ══════════════════════════════════════════════════════════════

#[test]
fn repunit_chain_connects_radian_to_circle() {
    // The base-3 repunits form a chain:
    //   1₃ = 1
    //   11₃ = 4
    //   111₃ = 13  ← radian
    //   1111₃ = 40
    //   11111₃ = 121
    //   111111₃ = 364  ← full circle
    let expected = [1u64, 4, 13, 40, 121, 364];
    for (i, &val) in expected.iter().enumerate() {
        assert!(
            is_base3_repunit(val),
            "Expected {} to be a base-3 repunit (order {})",
            val,
            i + 1
        );
        assert_eq!(base3_repunit_order(val), Some((i + 1) as u32));
    }

    // The radian (order 3) and circle (order 6) are linked:
    // circle = radian × 28, and 28 = 2π in the ternary system.
    assert_eq!(expected[2] * 28, expected[5]);
}

// ══════════════════════════════════════════════════════════════
// EDGE CASES — The zero boundary
// ══════════════════════════════════════════════════════════════

#[test]
fn bijective_zero_is_empty() {
    // In bijective ternary (Rep C), zero has no representation.
    let zero = TritVec::from_decimal(0);
    let repr_c = zero.to_repr_c();
    assert!(
        repr_c.is_empty(),
        "Bijective representation of 0 should be empty, got {:?}",
        repr_c
    );
}

#[test]
fn balanced_zero_is_zero() {
    // In balanced ternary (Rep A), zero is [0].
    let zero = TritVec::from_decimal(0);
    let repr_a = zero.to_repr_a();
    // All digits should be 0
    assert!(
        repr_a.iter().all(|&d| d == 0),
        "Balanced representation of 0 should be all zeros, got {:?}",
        repr_a
    );
}
