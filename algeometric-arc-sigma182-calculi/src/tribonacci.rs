// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `tribonacci` — base-`b` Tribonacci ladder
//!
//! `T_0 = 0, T_1 = 0, T_2 = 1, T_{n+3} = T_{n+2} + T_{n+1} + T_n`.
//! The Tribonacci ratio limit is the constant whose discrete
//! reciprocal underwrites the GAIT cumulative-delta identity
//! `Σ̃ = b³ · α⁻¹_int = 3699`.

use alloc::vec::Vec;

/// Compute the Tribonacci sequence up to index `n` inclusive.
pub fn tribonacci(n: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(n + 1);
    if n == 0 {
        out.push(0);
        return out;
    }
    out.push(0);
    out.push(0);
    if n == 1 {
        return out;
    }
    out.push(1);
    for i in 3..=n {
        let v = out[i - 1] + out[i - 2] + out[i - 3];
        out.push(v);
    }
    out
}
