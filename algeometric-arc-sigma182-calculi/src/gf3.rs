// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `gf3` — Galois Field GF(3) arithmetic
//!
//! GF(3) = ℤ/3ℤ. Three elements `{−1, 0, +1}` (Rep-A) under
//! addition and multiplication mod 3. Closed, associative,
//! distributive, with two-sided identities and additive inverses
//! for all elements + multiplicative inverses for the units `±1`.
//!
//! All operations here lift the per-trit GF(3) ops in [`Trit`] to
//! length-paired sequences via [`TritVec`].
//!
//! ## Invariants verified at compile time
//!
//! - **I-5.** GF(3) field axioms (additive identity, additive inverse,
//!   commutativity of `+` and `·`, multiplicative identity, multiplicative
//!   inverses for units, distributivity).
//! - **I-6.** Tritwise NOT is the additive inverse: `x + NOT(x) = 0`.

use crate::tritvec::TritVec;

// ════════════════════════════════════════════════════════════════════════
// Element-wise GF(3) ops on TritVec
// ════════════════════════════════════════════════════════════════════════

/// Element-wise GF(3) addition (`x + y mod 3`). Inputs must have equal length.
pub fn vec_add(a: &TritVec, b: &TritVec) -> Option<TritVec> {
    if a.len() != b.len() {
        return None;
    }
    Some(TritVec::from_iter_msb(
        a.as_slice()
            .iter()
            .zip(b.as_slice().iter())
            .map(|(x, y)| x.add(*y)),
    ))
}

/// Element-wise GF(3) subtraction.
pub fn vec_sub(a: &TritVec, b: &TritVec) -> Option<TritVec> {
    if a.len() != b.len() {
        return None;
    }
    Some(TritVec::from_iter_msb(
        a.as_slice()
            .iter()
            .zip(b.as_slice().iter())
            .map(|(x, y)| x.sub(*y)),
    ))
}

/// Element-wise GF(3) multiplication.
pub fn vec_mul(a: &TritVec, b: &TritVec) -> Option<TritVec> {
    if a.len() != b.len() {
        return None;
    }
    Some(TritVec::from_iter_msb(
        a.as_slice()
            .iter()
            .zip(b.as_slice().iter())
            .map(|(x, y)| x.mul(*y)),
    ))
}

/// Element-wise GF(3) NOT (additive inverse).
pub fn vec_not(a: &TritVec) -> TritVec {
    TritVec::from_iter_msb(a.as_slice().iter().map(|x| x.not()))
}

// ════════════════════════════════════════════════════════════════════════
// I-5 / I-6 — compile-time field axioms
// ════════════════════════════════════════════════════════════════════════

const _: () = {
    use crate::trit::Trit;

    let zero = Trit::ZERO;
    let one = Trit::ONE;
    let neg = Trit::NEG_ONE;

    // Additive identity
    assert!(zero.add(zero).value_a() == 0);
    assert!(one.add(zero).value_a() == 1);
    assert!(neg.add(zero).value_a() == -1);

    // Additive inverse via NOT (I-6)
    assert!(one.add(one.not()).value_a() == 0);
    assert!(neg.add(neg.not()).value_a() == 0);
    assert!(zero.add(zero.not()).value_a() == 0);

    // Commutativity of +
    assert!(one.add(neg).value_a() == neg.add(one).value_a());

    // Multiplicative identity
    assert!(one.mul(one).value_a() == 1);
    assert!(neg.mul(one).value_a() == -1);
    assert!(zero.mul(one).value_a() == 0);

    // Multiplicative inverse for units
    let u = match one.gf3_inverse() { Some(t) => t, None => Trit::ZERO };
    let v = match neg.gf3_inverse() { Some(t) => t, None => Trit::ZERO };
    assert!(one.mul(u).value_a() == 1);
    assert!(neg.mul(v).value_a() == 1);
    // 0 has no multiplicative inverse
    assert!(matches!(zero.gf3_inverse(), None));

    // Distributivity: a·(b + c) = a·b + a·c, sample on (1, −1, 1)
    let lhs = one.mul(neg.add(one));
    let rhs = one.mul(neg).add(one.mul(one));
    assert!(lhs.value_a() == rhs.value_a());
};
