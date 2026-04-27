// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `borromean` — ternary Borromean XOR invariant
//!
//! For three trits `(x, y, z)`, the Borromean XOR is
//!
//! ```text
//!     Bor(x, y, z) = x ⊕ y ⊕ z   (in GF(3))
//! ```
//!
//! and the **Borromean invariant** says that, on any cyclic permutation
//! of the three operands, `Bor` is preserved — a ternary echo of the
//! Borromean rings: removing or rotating any single ring/trit changes
//! nothing. Lifted element-wise to [`TritVec`], it gives a small,
//! pairing-free 3-fold XOR primitive used by the conservation laws and
//! by the Plenum Square σ-permutation tests.
//!
//! ## Invariants verified at compile time
//!
//! - **I-45.** Cyclic invariance: `Bor(x, y, z) = Bor(y, z, x) = Bor(z, x, y)`
//!   for all trits — a pure consequence of GF(3) associativity and
//!   commutativity of `+`, but proven here as a load-bearing identity.

use crate::trit::Trit;
use crate::tritvec::TritVec;

/// Borromean XOR on a single trit triple.
#[inline]
pub const fn bor(x: Trit, y: Trit, z: Trit) -> Trit {
    x.add(y).add(z)
}

/// Element-wise Borromean XOR on three TritVecs of equal length.
pub fn vec_bor(x: &TritVec, y: &TritVec, z: &TritVec) -> Option<TritVec> {
    if x.len() != y.len() || y.len() != z.len() {
        return None;
    }
    Some(TritVec::from_iter_msb(
        x.as_slice()
            .iter()
            .zip(y.as_slice().iter())
            .zip(z.as_slice().iter())
            .map(|((a, b), c)| bor(*a, *b, *c)),
    ))
}

// ════════════════════════════════════════════════════════════════════════
// I-45 — Borromean cyclic invariance (compile-time)
// ════════════════════════════════════════════════════════════════════════

const _: () = {
    // Walk the full 27-element cube of trit triples.
    let alphabet = [Trit::One, Trit::Two, Trit::Three];

    let mut i = 0;
    while i < 3 {
        let mut j = 0;
        while j < 3 {
            let mut k = 0;
            while k < 3 {
                let x = alphabet[i];
                let y = alphabet[j];
                let z = alphabet[k];
                let xyz = bor(x, y, z);
                let yzx = bor(y, z, x);
                let zxy = bor(z, x, y);
                assert!(xyz.value_a() == yzx.value_a());
                assert!(yzx.value_a() == zxy.value_a());
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
};
