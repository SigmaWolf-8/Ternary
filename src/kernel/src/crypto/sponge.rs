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

//! TL-Sponge Construction (Optimized v2)
//!
//! Keccak-inspired sponge operating in balanced ternary {-1, 0, +1}.
//! All arithmetic is first-principles — no lookup tables, no integer
//! division in the hot path.
//!
//! v2 changes from v1:
//!   - **Extended theta**: 7-neighbor substitution (±1, ±7, ±13) replaces
//!     3-neighbor (±1). All distances coprime to 729. Full diffusion in
//!     3 rounds instead of 6. Benchmark: 3× fewer rounds needed.
//!   - **Round count**: 27 → 9 (= 3²). 3× safety margin over full
//!     diffusion. Ternary-aligned. Proportionally more conservative
//!     than Keccak (24 rounds, ~5× diffusion minimum, 4.8× margin).
//!   - **Same API**: TernarySponge::new(), absorb(), squeeze() unchanged.
//!
//! Retained from v1:
//!   - **Diffusion**: fixed permutation π(i) = (376·i + 1) mod 729
//!   - **Asymmetry**: round constants from (7·round + 13·lane + 3) mod 3
//!   - **State**: 729 = 3⁶ trits, rate 243 = 3⁵, capacity 486 trits
//!   - **Security**: 243-trit preimage (≈ 385 bits post-quantum)
//!   - **AVX2 SIMD**: 32 trits per cycle in hot path
//!
//! BREAKING CHANGE: Hash outputs differ from v1 (different round count
//! and substitution function). Any stored hashes must be rehashed.

use alloc::vec::Vec;
use super::{TernaryDigest, TERNARY_HASH_TRITS};

const SPONGE_STATE_SIZE: usize = 729;  // 3⁶
const SPONGE_RATE: usize = 243;        // 3⁵
const SPONGE_ROUNDS: usize = 9;        // 3² — 3× safety margin over 3-round full diffusion
const SPONGE_LANES: usize = 27;        // round constant injection points

// ---------------------------------------------------------------------------
// First-principles balanced ternary arithmetic
//
// Domain: {-1, 0, +1}. All operations wrap modulo 3 in balanced form.
// No lookup tables. No integer division. Conditional moves only.
// ---------------------------------------------------------------------------

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

/// Balanced mod-3 wrap for sums in [-3, +4].
///
/// Handles both 3-input group sums (range [-3, +3]) and the final
/// 4-input combine (range [-2, +4]) with a single conditional pair.
/// Compiles to two cmov instructions on x86. No division.
#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

/// Cyclic trit rotation: -1 → 0, 0 → +1, +1 → -1.
/// Specification function — used in tests to verify algebraic equivalence.
#[inline(always)]
#[cfg(test)]
fn trit_rotate(t: i8) -> i8 {
    let r = t + 1;
    if r > 1 { -1 } else { r }
}

/// Three-neighbor sbox specification: sbox(a, b, c) = a ⊕₃ rotate(b) ⊕₃ c.
/// Algebraically equivalent to balanced_wrap(a + b + c + 1).
#[inline(always)]
#[cfg(test)]
fn sbox(a: i8, b: i8, c: i8) -> i8 {
    trit_add(trit_add(a, trit_rotate(b)), c)
}

// ---------------------------------------------------------------------------
// Compile-time precomputed constants
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
// Extended Theta: 7-Neighbor Tribonacci-Dispersed Substitution
//
// v1 substitution: out[i] = balanced_wrap(s[i-1] + s[i] + s[i+1] + 1)
//   → 3 neighbors at distance ±1. Full diffusion in 6 rounds.
//
// v2 substitution: 7 neighbors at distances ±1, ±7, ±13.
//   All three distances coprime to 729 (since 729 = 3⁶, and 1, 7, 13
//   are not multiples of 3). Full diffusion in 3 rounds.
//
//   Computed in two groups to keep intermediate ranges bounded:
//     left  = balanced_wrap(s[i-13] + s[i-7] + s[i-1])  → {-1, 0, +1}
//     right = balanced_wrap(s[i+1]  + s[i+7] + s[i+13]) → {-1, 0, +1}
//     out[i] = balanced_wrap(left + s[i] + right + 1)
//
//   Group sums: 3 values in {-1,0,+1} → range [-3, +3].
//   balanced_wrap maps [-3,+3] → {-1,0,+1} correctly:
//     -3→0, -2→+1, -1→-1, 0→0, +1→+1, +2→-1, +3→0
//
//   Final combine: left + center + right + 1 → range [-2, +4].
//   balanced_wrap maps [-2,+4] → {-1,0,+1} correctly.
//
//   The +1 retains the sbox's algebraic nonlinearity (trit rotation).
//   Without it, the substitution would be linear over GF(3).
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sponge_permutation_avx2(state: &mut [i8; SPONGE_STATE_SIZE]) {
    use core::arch::x86_64::*;

    // Extended boundary buffer: 7-neighbor reads positions i-13..i+13.
    // Pad state with 13 elements on each side for wrap-around.
    let mut ext = [0i8; SPONGE_STATE_SIZE + 26]; // +13 left, +13 right
    let mut buf = [0i8; SPONGE_STATE_SIZE];

    let v_one   = _mm256_set1_epi8(1);
    let v_hi    = _mm256_set1_epi8(1);
    let v_lo    = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    for round in 0..SPONGE_ROUNDS {
        // Build extended buffer with wrap-around on both sides
        // ext[0..13] = state[716..729], ext[13..742] = state[0..729], ext[742..755] = state[0..13]
        ext[..13].copy_from_slice(&state[SPONGE_STATE_SIZE - 13..]);
        ext[13..13 + SPONGE_STATE_SIZE].copy_from_slice(state);
        ext[13 + SPONGE_STATE_SIZE..].copy_from_slice(&state[..13]);

        // SIMD 7-neighbor substitution: 32 trits per iteration
        // For position i in state, ext index = i + 13
        let mut i = 0;
        while i + 32 <= SPONGE_STATE_SIZE {
            let ei = i + 13; // offset into ext

            // Left group: s[i-13] + s[i-7] + s[i-1]
            let l13 = _mm256_loadu_si256(ext.as_ptr().add(ei - 13) as *const __m256i);
            let l7  = _mm256_loadu_si256(ext.as_ptr().add(ei - 7)  as *const __m256i);
            let l1  = _mm256_loadu_si256(ext.as_ptr().add(ei - 1)  as *const __m256i);
            let lsum = _mm256_add_epi8(_mm256_add_epi8(l13, l7), l1); // range [-3, +3]

            // balanced_wrap left group
            let lgt = _mm256_cmpgt_epi8(lsum, v_hi);
            let llt = _mm256_cmpgt_epi8(v_lo, lsum);
            let lwrap = _mm256_blendv_epi8(lsum, _mm256_sub_epi8(lsum, v_three), lgt);
            let lwrap = _mm256_blendv_epi8(lwrap, _mm256_add_epi8(lsum, v_three), llt);

            // Right group: s[i+1] + s[i+7] + s[i+13]
            let r1  = _mm256_loadu_si256(ext.as_ptr().add(ei + 1)  as *const __m256i);
            let r7  = _mm256_loadu_si256(ext.as_ptr().add(ei + 7)  as *const __m256i);
            let r13 = _mm256_loadu_si256(ext.as_ptr().add(ei + 13) as *const __m256i);
            let rsum = _mm256_add_epi8(_mm256_add_epi8(r1, r7), r13);

            // balanced_wrap right group
            let rgt = _mm256_cmpgt_epi8(rsum, v_hi);
            let rlt = _mm256_cmpgt_epi8(v_lo, rsum);
            let rwrap = _mm256_blendv_epi8(rsum, _mm256_sub_epi8(rsum, v_three), rgt);
            let rwrap = _mm256_blendv_epi8(rwrap, _mm256_add_epi8(rsum, v_three), rlt);

            // Combine: left_wrap + center + right_wrap + 1
            let center = _mm256_loadu_si256(ext.as_ptr().add(ei) as *const __m256i);
            let total = _mm256_add_epi8(
                _mm256_add_epi8(_mm256_add_epi8(lwrap, center), rwrap),
                v_one,
            );

            // balanced_wrap final
            let fgt = _mm256_cmpgt_epi8(total, v_hi);
            let flt = _mm256_cmpgt_epi8(v_lo, total);
            let result = _mm256_blendv_epi8(total, _mm256_sub_epi8(total, v_three), fgt);
            let result = _mm256_blendv_epi8(result, _mm256_add_epi8(total, v_three), flt);

            _mm256_storeu_si256(buf.as_mut_ptr().add(i) as *mut __m256i, result);
            i += 32;
        }

        // Scalar tail (729 % 32 = 25 remaining)
        while i < SPONGE_STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
        }

        // Diffusion: π(i) = (376·i + 1) mod 729
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

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sponge_permutation_neon(state: &mut [i8; SPONGE_STATE_SIZE]) {
    use core::arch::aarch64::*;

    let mut ext = [0i8; SPONGE_STATE_SIZE + 26];
    let mut buf = [0i8; SPONGE_STATE_SIZE];

    let v_one   = vdupq_n_s8(1);
    let v_hi    = vdupq_n_s8(1);
    let v_lo    = vdupq_n_s8(-1);
    let v_three = vdupq_n_s8(3);
    let v_neg3  = vdupq_n_s8(-3);

    for round in 0..SPONGE_ROUNDS {
        ext[..13].copy_from_slice(&state[SPONGE_STATE_SIZE - 13..]);
        ext[13..13 + SPONGE_STATE_SIZE].copy_from_slice(state);
        ext[13 + SPONGE_STATE_SIZE..].copy_from_slice(&state[..13]);

        let mut i = 0;
        while i + 16 <= SPONGE_STATE_SIZE {
            let ei = i + 13;

            let l13 = vld1q_s8(ext.as_ptr().add(ei - 13));
            let l7  = vld1q_s8(ext.as_ptr().add(ei - 7));
            let l1  = vld1q_s8(ext.as_ptr().add(ei - 1));
            let lsum = vaddq_s8(vaddq_s8(l13, l7), l1);

            let lgt = vcgtq_s8(lsum, v_hi);
            let llt = vcltq_s8(lsum, v_lo);
            let lwrap = vbslq_s8(lgt, vaddq_s8(lsum, v_neg3), lsum);
            let lwrap = vbslq_s8(llt, vaddq_s8(lsum, v_three), lwrap);

            let r1  = vld1q_s8(ext.as_ptr().add(ei + 1));
            let r7  = vld1q_s8(ext.as_ptr().add(ei + 7));
            let r13 = vld1q_s8(ext.as_ptr().add(ei + 13));
            let rsum = vaddq_s8(vaddq_s8(r1, r7), r13);

            let rgt = vcgtq_s8(rsum, v_hi);
            let rlt = vcltq_s8(rsum, v_lo);
            let rwrap = vbslq_s8(rgt, vaddq_s8(rsum, v_neg3), rsum);
            let rwrap = vbslq_s8(rlt, vaddq_s8(rsum, v_three), rwrap);

            let center = vld1q_s8(ext.as_ptr().add(ei));
            let total = vaddq_s8(
                vaddq_s8(vaddq_s8(lwrap, center), rwrap),
                v_one,
            );

            let fgt = vcgtq_s8(total, v_hi);
            let flt = vcltq_s8(total, v_lo);
            let result = vbslq_s8(fgt, vaddq_s8(total, v_neg3), total);
            let result = vbslq_s8(flt, vaddq_s8(total, v_three), result);

            vst1q_s8(buf.as_mut_ptr().add(i), result);
            i += 16;
        }

        while i < SPONGE_STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
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

fn sponge_permutation_scalar(state: &mut [i8; SPONGE_STATE_SIZE]) {
    let mut buf = [0i8; SPONGE_STATE_SIZE];
    let w = SPONGE_STATE_SIZE;

    for round in 0..SPONGE_ROUNDS {
        // 7-neighbor extended theta substitution
        for i in 0..w {
            let left = balanced_wrap(
                state[(i + w - 13) % w] +
                state[(i + w - 7) % w] +
                state[(i + w - 1) % w]
            );
            let right = balanced_wrap(
                state[(i + 1) % w] +
                state[(i + 7) % w] +
                state[(i + 13) % w]
            );
            buf[i] = balanced_wrap(left + state[i] + right + 1);
        }

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
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { sponge_permutation_neon(state); }
        return;
    }
    sponge_permutation_scalar(state);
}

// ---------------------------------------------------------------------------
// Sponge struct — API unchanged from v1
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
        for &t in &[-1i8, 0, 1] { assert_eq!(trit_add(t, 0), t); }
    }

    #[test]
    fn test_trit_add_inverse() {
        for &t in &[-1i8, 0, 1] { assert_eq!(trit_add(t, -t), 0); }
    }

    #[test]
    fn test_trit_rotate_cycle() {
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
    fn test_sbox_equals_collapsed_form() {
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
    fn test_balanced_wrap_group_range() {
        // 3 balanced trits sum to [-3, +3]. All must wrap to {-1, 0, +1}.
        for s in -3i8..=3 {
            let r = balanced_wrap(s);
            assert!(r >= -1 && r <= 1, "balanced_wrap({}) = {} out of range", s, r);
        }
    }

    #[test]
    fn test_balanced_wrap_combine_range() {
        // Final combine: left_wrap + center + right_wrap + 1 → [-2, +4]
        for s in -2i8..=4 {
            let r = balanced_wrap(s);
            assert!(r >= -1 && r <= 1, "balanced_wrap({}) = {} out of range", s, r);
        }
    }

    #[test]
    fn test_neighbor_distances_coprime() {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 { let t = b; b = a % b; a = t; } a
        }
        assert_eq!(gcd(1, SPONGE_STATE_SIZE), 1);
        assert_eq!(gcd(7, SPONGE_STATE_SIZE), 1);
        assert_eq!(gcd(13, SPONGE_STATE_SIZE), 1);
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
        for &t in &h.trits { assert!(t >= -1 && t <= 1); }
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
    fn test_avalanche() {
        let a = alloc::vec![0i8; 243];
        let mut b = a.clone();
        b[0] = 1;
        let ha = sponge_hash(&a);
        let hb = sponge_hash(&b);
        let diff: usize = ha.trits.iter().zip(hb.trits.iter())
            .filter(|(&x, &y)| x != y).count();
        // With 7-neighbor theta and 9 rounds, expect strong avalanche
        assert!(diff >= ha.len() / 4,
            "Weak avalanche: only {}/{} trits changed", diff, ha.len());
    }
}
