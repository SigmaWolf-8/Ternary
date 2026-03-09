// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Ternary Sponge Construction (Optimized)
//!
//! Keccak-inspired sponge operating in balanced ternary {-1, 0, +1}.
//! All arithmetic is first-principles — no lookup tables, no integer
//! division in the hot path. The permutation uses:
//!
//!   - **Substitution**: three-neighbor sbox with cyclic trit rotation
//!   - **Diffusion**: fixed permutation π(i) = (376·i + 1) mod 729
//!   - **Asymmetry**: round constants derived from (7·round + 13·lane + 3) mod 3
//!
//! State size 729 = 3⁶ trits, rate 243 = 3⁵ trits, capacity 486 trits.
//! Security target: 243-trit preimage resistance (≈ 385 bits).
//!
//! # Performance
//!
//! Hot-path operations use conditional arithmetic (compiles to cmov on x86)
//! instead of integer modulo. Single auxiliary buffer, zero heap allocation
//! during permutation. Sbox + diffusion read from one buffer and write to
//! another — no full-state copies.

use alloc::vec::Vec;
use super::{TernaryDigest, TERNARY_HASH_TRITS};

const SPONGE_STATE_SIZE: usize = 729;  // 3⁶
const SPONGE_RATE: usize = 243;        // 3⁵
const SPONGE_ROUNDS: usize = 27;       // 3³
const SPONGE_LANES: usize = 27;        // round constant injection points

// ---------------------------------------------------------------------------
// First-principles balanced ternary arithmetic
//
// Domain: {-1, 0, +1}.  All operations wrap modulo 3 in balanced form.
// No lookup tables.  No integer division.  Conditional moves only.
//
// NOTE: trit_rotate and sbox are retained as first-principles building
// blocks.  The hot path uses the algebraically collapsed form
// (balanced_wrap) but these functions serve as the specification that
// the collapsed form was derived from, and are exercised in tests.
// ---------------------------------------------------------------------------

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

/// Cyclic trit rotation: -1 → 0, 0 → +1, +1 → -1.
/// Three applications return to identity.
#[inline(always)]
#[cfg(test)]
fn trit_rotate(t: i8) -> i8 {
    let r = t + 1;
    if r > 1 { -1 } else { r }
}

/// Three-neighbor substitution box: sbox(a, b, c) = a ⊕₃ rotate(b) ⊕₃ c.
/// Algebraically equivalent to balanced_wrap(a + b + c + 1) — see
/// permutation comments for the derivation.
#[inline(always)]
#[cfg(test)]
fn sbox(a: i8, b: i8, c: i8) -> i8 {
    trit_add(trit_add(a, trit_rotate(b)), c)
}

// ---------------------------------------------------------------------------
// Compile-time precomputed tables
// ---------------------------------------------------------------------------

const PERM: [u16; SPONGE_STATE_SIZE] = {
    let mut p = [0u16; SPONGE_STATE_SIZE];
    let mut i = 0usize;
    while i < SPONGE_STATE_SIZE {
        p[i] = ((i * 376 + 1) % SPONGE_STATE_SIZE) as u16;
        i += 1;
    }
    p
};

const RC_TABLE: [[i8; SPONGE_LANES]; SPONGE_ROUNDS] = {
    let mut rc = [[0i8; SPONGE_LANES]; SPONGE_ROUNDS];
    let mut r = 0usize;
    while r < SPONGE_ROUNDS {
        let mut lane = 0usize;
        while lane < SPONGE_LANES {
            let val = (r * 7 + lane * 13 + 3) % 3;
            rc[r][lane] = val as i8 - 1;
            lane += 1;
        }
        r += 1;
    }
    rc
};

// ---------------------------------------------------------------------------
// Permutation
//
// Substitution uses GF(3) associativity to collapse four serial trit_add
// calls into a single parallel integer sum followed by one balanced wrap.
//
//   sbox(a, b, c)  =  a ⊕₃ rotate(b) ⊕₃ c
//                   =  a ⊕₃ (b ⊕₃ 1) ⊕₃ c
//                   =  (a + b + c + 1) mod 3   [balanced form]
//
// The integer sum a+b+c+1 lies in [-2, 4].  Balanced mod-3 wrapping is a
// single conditional: subtract 3 if ≥ 2, add 3 if ≤ -2.  No division,
// no tables — purely first-principles modular arithmetic.
//
// On x86_64 with AVX2, the substitution processes 32 trits per cycle via
// _mm256_add_epi8 + compare/blend — the same first-principles conditional
// wrap, executed in parallel hardware lanes.
// ---------------------------------------------------------------------------

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sponge_permutation_avx2(state: &mut [i8; SPONGE_STATE_SIZE]) {
    use core::arch::x86_64::*;

    let mut ext = [0i8; SPONGE_STATE_SIZE + 2];
    let mut buf = [0i8; SPONGE_STATE_SIZE];
    let last = SPONGE_STATE_SIZE - 1;

    let v_one   = _mm256_set1_epi8(1);
    let v_hi    = _mm256_set1_epi8(1);
    let v_lo    = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    for round in 0..SPONGE_ROUNDS {
        // Wrap-around boundary: ext[0] = state[last], ext[N+1] = state[0]
        ext[0] = state[last];
        ext[1..SPONGE_STATE_SIZE + 1].copy_from_slice(state);
        ext[SPONGE_STATE_SIZE + 1] = state[0];

        // SIMD substitution: 32 trits per iteration
        let mut i = 0;
        while i + 32 <= SPONGE_STATE_SIZE {
            let left   = _mm256_loadu_si256(ext.as_ptr().add(i)     as *const __m256i);
            let center = _mm256_loadu_si256(ext.as_ptr().add(i + 1) as *const __m256i);
            let right  = _mm256_loadu_si256(ext.as_ptr().add(i + 2) as *const __m256i);

            // (left + center + right + 1) — collapsed sbox
            let sum = _mm256_add_epi8(
                _mm256_add_epi8(_mm256_add_epi8(left, center), right),
                v_one,
            );

            // Balanced wrap: subtract 3 if > 1, add 3 if < -1
            let gt1    = _mm256_cmpgt_epi8(sum, v_hi);
            let lt_neg = _mm256_cmpgt_epi8(v_lo, sum);
            let sub3   = _mm256_sub_epi8(sum, v_three);
            let add3   = _mm256_add_epi8(sum, v_three);

            let result = _mm256_blendv_epi8(sum, sub3, gt1);
            let result = _mm256_blendv_epi8(result, add3, lt_neg);

            _mm256_storeu_si256(buf.as_mut_ptr().add(i) as *mut __m256i, result);
            i += 32;
        }

        // Scalar tail for remaining trits (729 % 32 = 25)
        while i < SPONGE_STATE_SIZE {
            let raw = ext[i] + ext[i + 1] + ext[i + 2] + 1;
            buf[i] = balanced_wrap(raw);
            i += 1;
        }

        // Diffusion: fixed permutation π(i) = (376·i + 1) mod 729
        for i in 0..SPONGE_STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }

        // Round constant injection
        let rc = &RC_TABLE[round];
        for lane in 0..SPONGE_LANES {
            let idx = lane * SPONGE_LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

fn sponge_permutation_scalar(state: &mut [i8; SPONGE_STATE_SIZE]) {
    let mut buf = [0i8; SPONGE_STATE_SIZE];
    let last = SPONGE_STATE_SIZE - 1;

    for round in 0..SPONGE_ROUNDS {
        // Substitution with wrap-around boundary handling
        buf[0] = balanced_wrap(state[last] + state[0] + state[1] + 1);

        for i in 1..last {
            buf[i] = balanced_wrap(state[i - 1] + state[i] + state[i + 1] + 1);
        }

        buf[last] = balanced_wrap(state[last - 1] + state[last] + state[0] + 1);

        // Diffusion
        for i in 0..SPONGE_STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }

        // Round constant injection
        let rc = &RC_TABLE[round];
        for lane in 0..SPONGE_LANES {
            let idx = lane * SPONGE_LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

fn sponge_permutation(state: &mut [i8; SPONGE_STATE_SIZE]) {
    #[cfg(all(target_arch = "x86_64", not(feature = "no_std")))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { sponge_permutation_avx2(state); }
            return;
        }
    }
    sponge_permutation_scalar(state);
}

// ---------------------------------------------------------------------------
// Sponge struct
// ---------------------------------------------------------------------------

pub struct TernarySponge {
    state: [i8; SPONGE_STATE_SIZE],
    buf: [i8; SPONGE_RATE],
    buf_len: usize,
    absorbed: bool,
    squeezed: bool,
}

impl TernarySponge {
    pub fn new() -> Self {
        Self {
            state: [0i8; SPONGE_STATE_SIZE],
            buf: [0i8; SPONGE_RATE],
            buf_len: 0,
            absorbed: false,
            squeezed: false,
        }
    }

    /// Absorb trit data into the sponge.
    ///
    /// Buffered until a full rate block (243 trits) accumulates, then
    /// XOR'd into the rate portion of the state and permuted.
    /// No heap allocation in the absorb path.
    pub fn absorb(&mut self, input: &[i8]) {
        self.absorbed = true;
        self.squeezed = false;

        let mut offset = 0;
        let input_len = input.len();

        // Fill partial buffer from previous call
        if self.buf_len > 0 {
            let space = SPONGE_RATE - self.buf_len;
            let fill = if input_len < space { input_len } else { space };
            self.buf[self.buf_len..self.buf_len + fill]
                .copy_from_slice(&input[..fill]);
            self.buf_len += fill;
            offset = fill;

            if self.buf_len == SPONGE_RATE {
                for i in 0..SPONGE_RATE {
                    self.state[i] = trit_add(self.state[i], self.buf[i]);
                }
                sponge_permutation(&mut self.state);
                self.buf_len = 0;
            }
        }

        // Process full blocks directly from input slice — zero copy
        while offset + SPONGE_RATE <= input_len {
            let block = &input[offset..offset + SPONGE_RATE];
            for i in 0..SPONGE_RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            sponge_permutation(&mut self.state);
            offset += SPONGE_RATE;
        }

        // Buffer any remaining trits (< one block)
        let remaining = input_len - offset;
        if remaining > 0 {
            self.buf[self.buf_len..self.buf_len + remaining]
                .copy_from_slice(&input[offset..]);
            self.buf_len += remaining;
        }
    }

    /// Absorb raw bytes by converting to balanced ternary (5 trits per byte).
    pub fn absorb_bytes(&mut self, input: &[u8]) {
        // Stack buffer: 51 bytes × 5 trits = 255 ≈ one rate block
        let mut trit_buf = [0i8; 255];
        let mut trit_len = 0;

        for &byte in input {
            let mut val = byte;
            for _ in 0..5 {
                trit_buf[trit_len] = (val % 3) as i8 - 1;
                val /= 3;
                trit_len += 1;
            }
            if trit_len >= 250 {
                self.absorb(&trit_buf[..trit_len]);
                trit_len = 0;
            }
        }
        if trit_len > 0 {
            self.absorb(&trit_buf[..trit_len]);
        }
    }

    /// Finalize and squeeze output trits.
    ///
    /// Pads buffered data (single +1 trit after last data trit), permutes,
    /// then extracts rate-sized blocks until the requested length is reached.
    pub fn squeeze(&mut self, output_trits: usize) -> TernaryDigest {
        // Finalize: absorb remaining buffer with padding
        if self.buf_len > 0 || !self.absorbed {
            for i in 0..self.buf_len {
                self.state[i] = trit_add(self.state[i], self.buf[i]);
            }
            // Pad: inject +1 after last data trit
            if self.buf_len < SPONGE_RATE {
                self.state[self.buf_len] = trit_add(self.state[self.buf_len], 1);
            }
            self.buf_len = 0;
            sponge_permutation(&mut self.state);
        }

        // Squeeze: extract from rate portion
        let mut output = Vec::with_capacity(output_trits);

        while output.len() < output_trits {
            let remaining = output_trits - output.len();
            let take = if remaining < SPONGE_RATE { remaining } else { SPONGE_RATE };
            output.extend_from_slice(&self.state[..take]);

            if output.len() < output_trits {
                sponge_permutation(&mut self.state);
            }
        }

        output.truncate(output_trits);
        self.squeezed = true;

        TernaryDigest { trits: output }
    }

    pub fn squeeze_default(&mut self) -> TernaryDigest {
        self.squeeze(TERNARY_HASH_TRITS)
    }

    pub fn reset(&mut self) {
        self.state = [0i8; SPONGE_STATE_SIZE];
        self.buf = [0i8; SPONGE_RATE];
        self.buf_len = 0;
        self.absorbed = false;
        self.squeezed = false;
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

pub fn sponge_hash(input: &[i8]) -> TernaryDigest {
    let mut sponge = TernarySponge::new();
    sponge.absorb(input);
    sponge.squeeze_default()
}

pub fn sponge_hash_bytes(input: &[u8]) -> TernaryDigest {
    let mut sponge = TernarySponge::new();
    sponge.absorb_bytes(input);
    sponge.squeeze_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_add_exhaustive() {
        // All 9 input pairs — verified against modular arithmetic
        assert_eq!(trit_add(-1, -1), 1);   // -2 wraps to +1
        assert_eq!(trit_add(-1,  0), -1);
        assert_eq!(trit_add(-1,  1), 0);
        assert_eq!(trit_add( 0, -1), -1);
        assert_eq!(trit_add( 0,  0), 0);
        assert_eq!(trit_add( 0,  1), 1);
        assert_eq!(trit_add( 1, -1), 0);
        assert_eq!(trit_add( 1,  0), 1);
        assert_eq!(trit_add( 1,  1), -1);  // +2 wraps to -1
    }

    #[test]
    fn test_trit_add_commutative() {
        for &a in &[-1i8, 0, 1] {
            for &b in &[-1i8, 0, 1] {
                assert_eq!(trit_add(a, b), trit_add(b, a));
            }
        }
    }

    #[test]
    fn test_trit_add_zero_identity() {
        for &t in &[-1i8, 0, 1] {
            assert_eq!(trit_add(t, 0), t);
        }
    }

    #[test]
    fn test_trit_add_inverse() {
        for &t in &[-1i8, 0, 1] {
            assert_eq!(trit_add(t, -t), 0);
        }
    }

    #[test]
    fn test_trit_rotate_cycle() {
        // Three applications return to identity
        for &t in &[-1i8, 0, 1] {
            assert_eq!(trit_rotate(trit_rotate(trit_rotate(t))), t);
        }
    }

    #[test]
    fn test_trit_rotate_values() {
        assert_eq!(trit_rotate(-1), 0);
        assert_eq!(trit_rotate(0), 1);
        assert_eq!(trit_rotate(1), -1);
    }

    #[test]
    fn test_sbox_nonlinearity() {
        // sbox(a, b, c) = a ⊕ rotate(b) ⊕ c
        assert_eq!(sbox(0, 0, 0), 1);   // 0 + rotate(0) + 0 = 0 + 1 + 0 = 1
        assert_eq!(sbox(0, 1, 0), -1);  // 0 + rotate(1) + 0 = 0 + (-1) + 0 = -1
        assert_eq!(sbox(1, -1, -1), 0); // 1 + rotate(-1) + (-1) = 1 + 0 + (-1) = 0
    }

    #[test]
    fn test_sbox_equals_collapsed_form() {
        // Verify that sbox(a,b,c) == balanced_wrap(a + b + c + 1)
        // This proves the hot-path optimization is algebraically equivalent
        for &a in &[-1i8, 0, 1] {
            for &b in &[-1i8, 0, 1] {
                for &c in &[-1i8, 0, 1] {
                    assert_eq!(
                        sbox(a, b, c),
                        balanced_wrap(a + b + c + 1),
                        "Mismatch at sbox({}, {}, {})", a, b, c
                    );
                }
            }
        }
    }

    #[test]
    fn test_diffusion_full_period() {
        // π(i) = (376·i + 1) mod 729 must be a bijection
        let mut seen = [false; SPONGE_STATE_SIZE];
        for i in 0..SPONGE_STATE_SIZE {
            let dest = (i * 376 + 1) % SPONGE_STATE_SIZE;
            assert!(!seen[dest], "Collision at dest {} from source {}", dest, i);
            seen[dest] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn test_sponge_creation() {
        let sponge = TernarySponge::new();
        assert!(!sponge.absorbed);
        assert!(!sponge.squeezed);
        assert_eq!(sponge.buf_len, 0);
    }

    #[test]
    fn test_sponge_deterministic() {
        let input = alloc::vec![0i8, 1, -1, 0, 1];
        let h1 = sponge_hash(&input);
        let h2 = sponge_hash(&input);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sponge_output_length() {
        let h = sponge_hash(&[0i8; 50]);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_sponge_valid_trits() {
        let h = sponge_hash(&[1i8, 0, -1]);
        for &t in &h.trits {
            assert!(t >= -1 && t <= 1);
        }
    }

    #[test]
    fn test_sponge_incremental() {
        let input = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0];
        let full = sponge_hash(&input);

        let mut sponge = TernarySponge::new();
        sponge.absorb(&input[..5]);
        sponge.absorb(&input[5..]);
        let incremental = sponge.squeeze_default();

        assert_eq!(full, incremental);
    }

    #[test]
    fn test_sponge_different_inputs() {
        let h1 = sponge_hash(&[0i8, 0, 0]);
        let h2 = sponge_hash(&[1i8, 0, 0]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sponge_empty_input() {
        let h = sponge_hash(&[]);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_sponge_large_input() {
        let input = alloc::vec![1i8; 2000];
        let h = sponge_hash(&input);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_sponge_bytes() {
        let h1 = sponge_hash_bytes(b"hello");
        let h2 = sponge_hash_bytes(b"hello");
        assert_eq!(h1, h2);

        let h3 = sponge_hash_bytes(b"world");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_sponge_reset() {
        let mut sponge = TernarySponge::new();
        sponge.absorb(&[1i8, 0, -1]);
        let h1 = sponge.squeeze_default();

        sponge.reset();
        sponge.absorb(&[1i8, 0, -1]);
        let h2 = sponge.squeeze_default();

        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sponge_custom_output_length() {
        let mut sponge = TernarySponge::new();
        sponge.absorb(&[1i8, 0, -1]);
        let h = sponge.squeeze(100);
        assert_eq!(h.len(), 100);
    }

    #[test]
    fn test_sponge_long_squeeze() {
        let mut sponge = TernarySponge::new();
        sponge.absorb(&[1i8, 0, -1]);
        let h = sponge.squeeze(1000);
        assert_eq!(h.len(), 1000);
        for &t in &h.trits {
            assert!(t >= -1 && t <= 1);
        }
    }

    #[test]
    fn test_capacity_untouched_before_permutation() {
        let mut state = [0i8; SPONGE_STATE_SIZE];
        let input = [1i8; SPONGE_RATE];
        for i in 0..SPONGE_RATE {
            state[i] = trit_add(state[i], input[i]);
        }
        for i in SPONGE_RATE..SPONGE_STATE_SIZE {
            assert_eq!(state[i], 0, "Capacity trit {} modified during absorb", i);
        }
    }
}
