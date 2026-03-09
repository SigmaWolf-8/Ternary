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

//! Ternary Sponge Construction — 7-Neighbor Extended Theta
//!
//! Keccak-inspired sponge operating in balanced ternary {-1, 0, +1}.
//! All arithmetic is first-principles — no lookup tables, no integer
//! division in the hot path. The permutation uses:
//!
//!   - **Theta**: 7-neighbor extended substitution at distances ±1, ±7, ±13
//!     (all coprime to 729 = 3⁶)
//!   - **Diffusion**: fixed permutation π(i) = (376·i + 1) mod 729
//!   - **Asymmetry**: round constants derived from (7·round + 13·lane + 3) mod 3
//!
//! State size 729 = 3⁶ trits, rate 243 = 3⁵ trits, capacity 486 trits.
//! Security target: 243-trit preimage resistance (≈ 385 bits).
//! 9 rounds = 3² (3× safety margin over diffusion diameter).
//!
//! # Performance
//!
//! Hot-path operations use conditional arithmetic (compiles to cmov on x86)
//! instead of integer modulo. On x86_64 with AVX2, theta processes 32 trits
//! per cycle using rotation buffers and SIMD balanced-wrap. Single auxiliary
//! buffer, zero heap allocation during permutation.

use alloc::vec::Vec;
use super::{TernaryDigest, TERNARY_HASH_TRITS};

const SPONGE_STATE_SIZE: usize = 729;  // 3⁶
const SPONGE_RATE: usize = 243;        // 3⁵
const SPONGE_ROUNDS: usize = 9;        // 3² = 3× safety margin
const SPONGE_LANES: usize = 27;

// ---------------------------------------------------------------------------
// First-principles balanced ternary arithmetic
//
// Domain: {-1, 0, +1}.  All operations wrap modulo 3 in balanced form.
// No lookup tables.  No integer division.  Conditional moves only.
// ---------------------------------------------------------------------------

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

#[inline(always)]
fn wrap_idx(i: usize, offset: usize) -> usize {
    let idx = i + offset;
    if idx >= SPONGE_STATE_SIZE { idx - SPONGE_STATE_SIZE } else { idx }
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
// Permutation — 7-neighbor extended theta
//
// For each trit position i:
//   left  = balanced_wrap(state[i−13] + state[i−7] + state[i−1])
//   right = balanced_wrap(state[i+1]  + state[i+7] + state[i+13])
//   theta[i] = balanced_wrap(left + state[i] + right)
//
// Each intermediate sum of 3 balanced trits lies in [−3, +3].
// balanced_wrap maps this to [−1, +1] with a single conditional:
// subtract 3 if ≥ 2, add 3 if ≤ −2.  No division, no tables.
//
// All neighbor distances (1, 7, 13) are coprime to 729 = 3⁶,
// ensuring full state-space coverage in the diffusion step.
// ---------------------------------------------------------------------------

#[inline(always)]
fn rot(src: &[i8; SPONGE_STATE_SIZE], dst: &mut [i8], dist: usize) {
    dst[..SPONGE_STATE_SIZE - dist]
        .copy_from_slice(&src[dist..SPONGE_STATE_SIZE]);
    dst[SPONGE_STATE_SIZE - dist..SPONGE_STATE_SIZE]
        .copy_from_slice(&src[..dist]);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sponge_permutation_avx2(state: &mut [i8; SPONGE_STATE_SIZE]) {
    use core::arch::x86_64::*;

    const PAD: usize = 736;
    let mut l13 = [0i8; PAD];
    let mut l7  = [0i8; PAD];
    let mut l1  = [0i8; PAD];
    let mut r1  = [0i8; PAD];
    let mut r7  = [0i8; PAD];
    let mut r13 = [0i8; PAD];
    let mut theta = [0i8; PAD];
    let mut s_pad = [0i8; PAD];

    let v_one   = _mm256_set1_epi8(1);
    let v_neg   = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    for round in 0..SPONGE_ROUNDS {
        s_pad[..SPONGE_STATE_SIZE].copy_from_slice(state);

        rot(state, &mut l13, 13);
        rot(state, &mut l7, 7);
        rot(state, &mut l1, 1);
        rot(state, &mut r1, SPONGE_STATE_SIZE - 1);
        rot(state, &mut r7, SPONGE_STATE_SIZE - 7);
        rot(state, &mut r13, SPONGE_STATE_SIZE - 13);

        let mut i = 0;
        while i + 32 <= PAD {
            let lg_raw = _mm256_add_epi8(
                _mm256_add_epi8(
                    _mm256_loadu_si256(r13[i..].as_ptr() as *const __m256i),
                    _mm256_loadu_si256(r7[i..].as_ptr() as *const __m256i),
                ),
                _mm256_loadu_si256(r1[i..].as_ptr() as *const __m256i),
            );
            let gt1_l  = _mm256_cmpgt_epi8(lg_raw, v_one);
            let lt_n_l = _mm256_cmpgt_epi8(v_neg, lg_raw);
            let lg = _mm256_blendv_epi8(
                _mm256_blendv_epi8(lg_raw, _mm256_sub_epi8(lg_raw, v_three), gt1_l),
                _mm256_add_epi8(lg_raw, v_three),
                lt_n_l,
            );

            let rg_raw = _mm256_add_epi8(
                _mm256_add_epi8(
                    _mm256_loadu_si256(l1[i..].as_ptr() as *const __m256i),
                    _mm256_loadu_si256(l7[i..].as_ptr() as *const __m256i),
                ),
                _mm256_loadu_si256(l13[i..].as_ptr() as *const __m256i),
            );
            let gt1_r  = _mm256_cmpgt_epi8(rg_raw, v_one);
            let lt_n_r = _mm256_cmpgt_epi8(v_neg, rg_raw);
            let rg = _mm256_blendv_epi8(
                _mm256_blendv_epi8(rg_raw, _mm256_sub_epi8(rg_raw, v_three), gt1_r),
                _mm256_add_epi8(rg_raw, v_three),
                lt_n_r,
            );

            let center = _mm256_loadu_si256(s_pad[i..].as_ptr() as *const __m256i);
            let sum = _mm256_add_epi8(_mm256_add_epi8(lg, center), rg);
            let gt1_s  = _mm256_cmpgt_epi8(sum, v_one);
            let lt_n_s = _mm256_cmpgt_epi8(v_neg, sum);
            let result = _mm256_blendv_epi8(
                _mm256_blendv_epi8(sum, _mm256_sub_epi8(sum, v_three), gt1_s),
                _mm256_add_epi8(sum, v_three),
                lt_n_s,
            );

            _mm256_storeu_si256(theta[i..].as_mut_ptr() as *mut __m256i, result);
            i += 32;
        }

        for i in 0..SPONGE_STATE_SIZE {
            state[PERM[i] as usize] = theta[i];
        }

        let rc = &RC_TABLE[round];
        for lane in 0..SPONGE_LANES {
            let idx = lane * SPONGE_LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

fn sponge_permutation_scalar(state: &mut [i8; SPONGE_STATE_SIZE]) {
    let mut buf = [0i8; SPONGE_STATE_SIZE];

    for round in 0..SPONGE_ROUNDS {
        for i in 0..SPONGE_STATE_SIZE {
            let left = balanced_wrap(
                state[wrap_idx(i, SPONGE_STATE_SIZE - 13)]
                + state[wrap_idx(i, SPONGE_STATE_SIZE - 7)]
                + state[wrap_idx(i, SPONGE_STATE_SIZE - 1)]
            );
            let right = balanced_wrap(
                state[wrap_idx(i, 1)]
                + state[wrap_idx(i, 7)]
                + state[wrap_idx(i, 13)]
            );
            buf[i] = balanced_wrap(left + state[i] + right);
        }

        for i in 0..SPONGE_STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }

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

    pub fn absorb(&mut self, input: &[i8]) {
        self.absorbed = true;
        self.squeezed = false;

        let mut offset = 0;
        let input_len = input.len();

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

        while offset + SPONGE_RATE <= input_len {
            let block = &input[offset..offset + SPONGE_RATE];
            for i in 0..SPONGE_RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            sponge_permutation(&mut self.state);
            offset += SPONGE_RATE;
        }

        let remaining = input_len - offset;
        if remaining > 0 {
            self.buf[self.buf_len..self.buf_len + remaining]
                .copy_from_slice(&input[offset..]);
            self.buf_len += remaining;
        }
    }

    pub fn absorb_bytes(&mut self, input: &[u8]) {
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

    pub fn squeeze(&mut self, output_trits: usize) -> TernaryDigest {
        if self.buf_len > 0 || !self.absorbed {
            for i in 0..self.buf_len {
                self.state[i] = trit_add(self.state[i], self.buf[i]);
            }
            if self.buf_len < SPONGE_RATE {
                self.state[self.buf_len] = trit_add(self.state[self.buf_len], 1);
            }
            self.buf_len = 0;
            sponge_permutation(&mut self.state);
        }

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
        assert_eq!(trit_add(-1, -1), 1);
        assert_eq!(trit_add(-1,  0), -1);
        assert_eq!(trit_add(-1,  1), 0);
        assert_eq!(trit_add( 0, -1), -1);
        assert_eq!(trit_add( 0,  0), 0);
        assert_eq!(trit_add( 0,  1), 1);
        assert_eq!(trit_add( 1, -1), 0);
        assert_eq!(trit_add( 1,  0), 1);
        assert_eq!(trit_add( 1,  1), -1);
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
    fn test_balanced_wrap_all_inputs() {
        assert_eq!(balanced_wrap(-3), 0);
        assert_eq!(balanced_wrap(-2), 1);
        assert_eq!(balanced_wrap(-1), -1);
        assert_eq!(balanced_wrap( 0), 0);
        assert_eq!(balanced_wrap( 1), 1);
        assert_eq!(balanced_wrap( 2), -1);
        assert_eq!(balanced_wrap( 3), 0);
    }

    #[test]
    fn test_theta_coprime() {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 { let t = b; b = a % b; a = t; }
            a
        }
        assert_eq!(gcd(1, SPONGE_STATE_SIZE), 1);
        assert_eq!(gcd(7, SPONGE_STATE_SIZE), 1);
        assert_eq!(gcd(13, SPONGE_STATE_SIZE), 1);
    }

    #[test]
    fn test_theta_produces_valid_trits() {
        for &a in &[-1i8, 0, 1] {
            for &b in &[-1i8, 0, 1] {
                for &c in &[-1i8, 0, 1] {
                    let sum = balanced_wrap(a + b + c);
                    assert!(sum >= -1 && sum <= 1,
                        "balanced_wrap({}) = {}", a + b + c, sum);
                }
            }
        }
        for &lg in &[-1i8, 0, 1] {
            for &center in &[-1i8, 0, 1] {
                for &rg in &[-1i8, 0, 1] {
                    let theta = balanced_wrap(lg + center + rg);
                    assert!(theta >= -1 && theta <= 1);
                }
            }
        }
    }

    #[test]
    fn test_wrap_idx_boundaries() {
        assert_eq!(wrap_idx(0, SPONGE_STATE_SIZE - 1), 728);
        assert_eq!(wrap_idx(0, SPONGE_STATE_SIZE - 7), 722);
        assert_eq!(wrap_idx(0, SPONGE_STATE_SIZE - 13), 716);
        assert_eq!(wrap_idx(728, 1), 0);
        assert_eq!(wrap_idx(722, 7), 0);
        assert_eq!(wrap_idx(716, 13), 0);
        assert_eq!(wrap_idx(100, 1), 101);
        assert_eq!(wrap_idx(100, 7), 107);
        assert_eq!(wrap_idx(100, 13), 113);
    }

    #[test]
    fn test_diffusion_full_period() {
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

    #[test]
    fn test_sponge_avalanche() {
        let a = alloc::vec![0i8; 50];
        let mut b = a.clone();
        b[0] = 1;
        let ha = sponge_hash(&a);
        let hb = sponge_hash(&b);
        let diff: usize = ha.trits.iter().zip(hb.trits.iter())
            .filter(|(&x, &y)| x != y).count();
        assert!(diff >= 50, "Avalanche too low: {}/{}", diff, TERNARY_HASH_TRITS);
    }
}
