// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// Tier-2 invariant suite. One named test per invariant. The const
// identity blocks already prove the algebraic identities at compile
// time; this suite re-asserts them at runtime as ground-truth pins
// and exercises the runtime constructors.

use aasc::arithmetic::{add, mul, sub, divmod};
use aasc::borromean::bor;
use aasc::constants::*;
use aasc::coprime::{gcd, totient};
use aasc::repx::{from_bijective, to_bijective};
use aasc::trit::Trit;
use aasc::tritvec::TritVec;

// ── I-1 ─────────────────────────────────────────────────────────────────

#[test]
fn i1_three_symbols() {
    let alpha = [Trit::One, Trit::Two, Trit::Three];
    assert_eq!(alpha.len(), 3);
}

// ── Dual-zero contract (Rep-A additive identity ≠ Rep-B numeric zero) ──
//
// Pinning regression: prevents future drift between the two distinct
// "zeros" the crate carries. `Trit::ZERO` is the GF(3) additive
// identity (Rep-A 0 = Rep-B value 1 = `Trit::Two`); the *positional
// numeric zero* used by base-3 arithmetic and `TritVec::zeros` is
// `Trit::One` (Rep-B value 0). Conflating them is what previously
// broke `sub`, `mul` trim, and `repx`.

#[test]
fn dual_zero_contract() {
    let additive_identity = Trit::ZERO;
    let numeric_zero_digit = TritVec::zeros(1).as_slice()[0];
    assert_eq!(additive_identity, Trit::Two);
    assert_eq!(numeric_zero_digit, Trit::One);
    assert_ne!(additive_identity, numeric_zero_digit);
    assert_eq!(additive_identity.value_b(), 1);
    assert_eq!(numeric_zero_digit.value_b(), 0);
    assert!(TritVec::zeros(5).is_zero());
}

// ── I-2 ─────────────────────────────────────────────────────────────────

#[test]
fn i2_three_pure_constructors() {
    assert!(Trit::from_a(-1).is_some());
    assert!(Trit::from_b(0).is_some());
    assert!(Trit::from_c(1).is_some());
    assert!(Trit::from_a(2).is_none());
    assert!(Trit::from_b(3).is_none());
    assert!(Trit::from_c(0).is_none());
}

// ── I-5 ─────────────────────────────────────────────────────────────────

#[test]
fn i5_gf3_field_axioms_runtime() {
    // x + 0 = x
    for &t in &[Trit::One, Trit::Two, Trit::Three] {
        assert_eq!(t.add(Trit::ZERO).value_a(), t.value_a());
    }
    // x · 1 = x
    for &t in &[Trit::One, Trit::Two, Trit::Three] {
        assert_eq!(t.mul(Trit::ONE).value_a(), t.value_a());
    }
}

// ── I-6 ─────────────────────────────────────────────────────────────────

#[test]
fn i6_not_is_additive_inverse() {
    for &t in &[Trit::One, Trit::Two, Trit::Three] {
        assert_eq!(t.add(t.not()).value_a(), 0);
    }
}

// ── I-7 ─────────────────────────────────────────────────────────────────

#[test]
fn i7_arithmetic_closed_on_tritvec() {
    let a = TritVec::from_rep_b(&[1, 0, 2]).unwrap(); // 11
    let b = TritVec::from_rep_b(&[2, 1]).unwrap();    //  7
    let s = add(&a, &b);                              // 18 = 200₃
    assert_eq!(s, TritVec::from_rep_b(&[2, 0, 0]).unwrap());
    let d = sub(&a, &b).unwrap();                     //  4 = 11₃
    assert_eq!(d, TritVec::from_rep_b(&[1, 1]).unwrap());
    let p = mul(&a, &b);                              // 77 = 2212₃
    assert_eq!(p, TritVec::from_rep_b(&[2, 2, 1, 2]).unwrap());
    let (q, r) = divmod(&a, &b).unwrap();             // 11/7 = 1r4
    assert_eq!(q, TritVec::from_rep_b(&[1]).unwrap());
    assert_eq!(r, TritVec::from_rep_b(&[1, 1]).unwrap());
}

// ── I-8/I-9 ─────────────────────────────────────────────────────────────

#[test]
fn i8_repunit_recurrence() {
    assert_eq!(R_2_INT, B_INT * R_1_INT + R_1_INT);
    assert_eq!(R_6_INT, B_INT * R_5_INT + R_1_INT);
}

#[test]
fn i9_repunit_closed_form() {
    assert_eq!(R_6_INT, (3u64.pow(6) - 1) / 2);
}

// ── I-11 ────────────────────────────────────────────────────────────────

#[test]
fn i11_arc_five_forms() {
    assert_eq!(ARC_INT, PI_INT * (PI_INT - 1));
    assert_eq!(ARC_INT, R_6_INT / 2);
    assert_eq!(ARC_INT, 2 * P_INT * R_3_INT);
    assert_eq!(ARC_INT, 2 * LAMBDA_EUV_INT);
    assert_eq!(ARC_INT, 182);
}

// ── I-12 ────────────────────────────────────────────────────────────────

#[test]
fn i12_sponge_discriminant() {
    assert_eq!(DELTA_SPONGE_INT, 1 + 4 * ARC_INT);
    assert_eq!(DELTA_SPONGE_INT, B6_INT);
    assert_eq!(B3_INT * B3_INT, B6_INT);
}

// ── I-14 ────────────────────────────────────────────────────────────────

#[test]
fn i14_pqr_pairwise_coprime() {
    assert_eq!(gcd(P_INT, Q_INT), 1);
    assert_eq!(gcd(P_INT, R_INT), 1);
    assert_eq!(gcd(Q_INT, R_INT), 1);
    assert_eq!(PQR_INT, P_INT * Q_INT * R_INT);
    assert_eq!(PQR_INT, 1001);
}

// ── I-22 ────────────────────────────────────────────────────────────────

#[test]
fn i22_inclusion_exclusion_at_143() {
    let n = P_H_INT * R_3_INT;
    assert_eq!(n - totient(n), P_H_INT + R_3_INT - 1);
    assert_eq!(n - totient(n), 23);
}

// ── I-24 ────────────────────────────────────────────────────────────────

#[test]
fn i24_plenum_color_closure() {
    assert_eq!(ARC_BLUE_INT, B5_INT - B_INT);
    assert_eq!(SQRT_DELTA_ARC_INT, ARC_RED_INT + ARC_COPRIME_INT);
    assert_eq!(SQRT_DELTA_ARC_INT, 36 * R_3_INT);
    assert_eq!(ARC_GREEN_INT, R_6_INT + ARC_COPRIME_INT);
    assert_eq!(ARC_COPRIME_INT - ARC_BLUE_INT, 2 * COMBINED_VERTICES_INT);
}

// ── I-31 ────────────────────────────────────────────────────────────────

#[test]
fn i31_uv_chain_pinned_values() {
    assert_eq!(LAMBDA_LYMAN_INT, 91);
    assert_eq!(LAMBDA_UVC_INT, 182);
    assert_eq!(LAMBDA_UVB_INT, 286);
    assert_eq!(LAMBDA_UVA_INT, 364);
}

// ── I-37 ────────────────────────────────────────────────────────────────

#[test]
fn i37_repx_is_bijection() {
    // Round-trip every value 0..1000 through standard ↔ bijective.
    for n in 0..1001u64 {
        let std = u64_to_tv(n);
        let bij = to_bijective(&std);
        let back = from_bijective(&bij);
        let back_val = tv_to_u64(&back);
        assert_eq!(back_val, n, "failed at n = {}", n);
    }
}

// ── I-44 ────────────────────────────────────────────────────────────────

#[test]
fn i44_cumulative_delta() {
    assert_eq!(SIGMA_TILDE_INT, 27 * 137);
    assert_eq!(SIGMA_TILDE_INT, 3699);
}

// ── I-45 ────────────────────────────────────────────────────────────────

#[test]
fn i45_borromean_cyclic_invariance() {
    for x in [Trit::One, Trit::Two, Trit::Three] {
        for y in [Trit::One, Trit::Two, Trit::Three] {
            for z in [Trit::One, Trit::Two, Trit::Three] {
                let xyz = bor(x, y, z);
                let yzx = bor(y, z, x);
                assert_eq!(xyz.value_a(), yzx.value_a());
            }
        }
    }
}

// ── I-46 ────────────────────────────────────────────────────────────────

#[test]
fn i46_magic_square_sum() {
    assert_eq!(M_SQ_INT, R_2_INT * B_INT);
    assert_eq!(M_SQ_INT, 12);
}

// ════════════════════════════════════════════════════════════════════════
// Test-only helpers (test crate, not library — host integers OK here).
// ════════════════════════════════════════════════════════════════════════

fn u64_to_tv(mut n: u64) -> TritVec {
    if n == 0 {
        return TritVec::from_rep_b(&[0]).unwrap();
    }
    let mut digits = alloc::vec::Vec::new();
    while n > 0 {
        digits.push((n % 3) as u8);
        n /= 3;
    }
    digits.reverse();
    TritVec::from_rep_b(&digits).unwrap()
}

fn tv_to_u64(t: &TritVec) -> u64 {
    let mut acc: u64 = 0;
    for x in t.as_slice() {
        acc = acc * 3 + x.value_b() as u64;
    }
    acc
}

extern crate alloc;
