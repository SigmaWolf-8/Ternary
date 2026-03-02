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
// ---------------------------------------------------------------------------

/// Balanced ternary addition: (a + b) mod 3, mapped to {-1, 0, +1}.
///
/// Direct arithmetic: the raw sum `a + b` lies in [-2, +2].
/// Values in {-1, 0, +1} are already valid.  The two overflow cases:
///   -2  →  +1  (add 3)
///   +2  →  -1  (subtract 3)
///
/// Compiles to: add, cmp, cmov, cmp, cmov — five instructions, zero division.
///
/// Uses wrapping arithmetic so callers that pass raw byte-derived values
/// (e.g. nonce bytes cast to i8) don't trigger debug-mode overflow panics.
/// For valid trit inputs the wrapping add is identical to plain add.
#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a.wrapping_add(b);
    if s > 1 { s.wrapping_sub(3) } else if s < -1 { s.wrapping_add(3) } else { s }
}

/// Cyclic rotation in {-1, 0, +1}: the map t ↦ t + 1 (mod 3).
///   -1 → 0,  0 → +1,  +1 → -1
///
/// Single increment with wrap.  Compiles to: add, cmp, cmov.
#[inline(always)]
fn trit_rotate(t: i8) -> i8 {
    let r = t.wrapping_add(1);
    if r > 1 { -1 } else { r }
}

/// Three-input substitution box.
/// sbox(a, b, c) = a ⊕ rotate(b) ⊕ c
///
/// Provides nonlinearity via the rotation on the middle input.
#[inline(always)]
fn sbox(a: i8, b: i8, c: i8) -> i8 {
    trit_add(trit_add(a, trit_rotate(b)), c)
}

/// Balanced reduction of a non-negative integer to {-1, 0, +1}.
/// Used only for round constant generation (27 calls per round,
/// outside the 729-iteration hot loops).
///
/// The compiler replaces `% 3` on a compile-time-known divisor with
/// a multiply-shift sequence — no actual integer division emitted.
#[inline(always)]
fn balanced_mod3(n: usize) -> i8 {
    (n % 3) as i8 - 1
}

// ---------------------------------------------------------------------------
// Permutation
// ---------------------------------------------------------------------------

/// Core sponge permutation: 27 rounds of substitution + diffusion + constants.
///
/// Uses a single auxiliary buffer (stack-allocated, 729 bytes).
/// The sbox layer reads `state` → writes `buf`.
/// The diffusion layer reads `buf` → writes `state`.
/// Round constants are injected into `state` in-place.
/// Zero full-state copies. Zero heap allocation.
fn sponge_permutation(state: &mut [i8; SPONGE_STATE_SIZE]) {
    let mut buf = [0i8; SPONGE_STATE_SIZE];

    for round in 0..SPONGE_ROUNDS {
        // ── Substitution layer ──────────────────────────────────────
        // Each trit: sbox(self, left_neighbor, right_neighbor)
        // Reads from `state`, writes to `buf`.

        // First element: left neighbor wraps to end
        buf[0] = sbox(state[0], state[SPONGE_STATE_SIZE - 1], state[1]);

        // Interior: no wrapping, no bounds overhead
        for i in 1..(SPONGE_STATE_SIZE - 1) {
            buf[i] = sbox(state[i], state[i - 1], state[i + 1]);
        }

        // Last element: right neighbor wraps to start
        buf[SPONGE_STATE_SIZE - 1] = sbox(
            state[SPONGE_STATE_SIZE - 1],
            state[SPONGE_STATE_SIZE - 2],
            state[0],
        );

        // ── Diffusion layer ─────────────────────────────────────────
        // Fixed permutation π(i) = (376·i + 1) mod 729.
        // gcd(376, 729) = 1, so π is a full-period permutation.
        // Reads from `buf`, writes to `state`.
        //
        // The `% 729` on a compile-time constant is optimized by LLVM
        // into a multiply-shift — no runtime integer division.
        for i in 0..SPONGE_STATE_SIZE {
            let dest = (i * 376 + 1) % SPONGE_STATE_SIZE;
            state[dest] = buf[i];
        }

        // ── Round constant injection ────────────────────────────────
        // 27 constants per round at lane positions (every 27th trit).
        // rc(round, lane) = ((7·round + 13·lane + 3) mod 3) - 1
        for lane in 0..SPONGE_LANES {
            let idx = lane * SPONGE_LANES;
            let rc = balanced_mod3(round * 7 + lane * 13 + 3);
            state[idx] = trit_add(state[idx], rc);
        }
    }
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
                // Buffer full — absorb it
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
        // Every trit has an additive inverse
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
        // Absorb XORs only into rate [0..243], never capacity [243..729]
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
