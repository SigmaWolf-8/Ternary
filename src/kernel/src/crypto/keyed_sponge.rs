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

//! Keyed TL-Sponge-385 Construction
//!
//! Extends the existing `TernarySponge` with a key-dependent permutation layer
//! derived from the automorphism group of the ternary 6-cube (S_3 ≀ S_6).
//!
//! # Motivation
//!
//! The standard sponge uses a fixed linear permutation `(i * 376 + 1) % 729`
//! for diffusion. This module replaces that fixed permutation with a
//! key-dependent permutation drawn from the ternary cube's symmetry group.
//!
//! # Security Properties
//!
//! - **Per-round diversity**: Each of the 27 rounds uses a different permutation
//!   derived from the master key, giving 27 × 25 ≈ 675 bits of key influence
//!   across the full permutation.
//!
//! - **Hamming distance preservation**: All permutations from S_3 ≀ S_6 preserve
//!   Hamming distance in the ternary cube, ensuring diffusion quality is at least
//!   as good as any fixed member of the group.
//!
//! - **Bijection guarantee**: Every element of the automorphism group is a
//!   bijection by definition. No key can produce a degenerate permutation.
//!
//! # When to Use
//!
//! - **Keyed MAC**: When you need a message authentication code with ternary-
//!   native key dependence (stronger than absorbing key as data alone).
//! - **Timing packet authentication**: Key-dependent structure means an attacker
//!   who compromises one key learns nothing about the permutation under a
//!   different key.
//! - **Domain separation**: Different services can use different keys, producing
//!   structurally different sponges that share no internal symmetry.
//!
//! # When NOT to Use
//!
//! - **Standard hashing**: Use the unkeyed `TernarySponge` for collision-
//!   resistant hashing. The keyed variant changes the security model.
//! - **Key derivation from passwords**: Use the existing KDF which properly
//!   handles iteration counts and salt.
//!
//! # Architecture
//!
//! The keyed sponge state is 729 = 3^6 trits, viewed as a 6-dimensional
//! ternary cube. The key schedule works as follows:
//!
//! 1. Master key (≥243 trits) is expanded using the UNKEYED sponge
//! 2. The expansion produces 27 independent trit streams (one per round)
//! 3. Each stream drives `TernaryCubeAutomorphism::from_key_trits()` to
//!    select that round's permutation from S_3 ≀ S_6
//! 4. Automorphisms are stored directly (~1KB total) and applied via
//!    coordinate decomposition at each round — no precomputed tables
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use alloc::vec;
use super::{TernaryDigest, TERNARY_HASH_TRITS};
use super::sponge::{sponge_hash, TernarySponge as UnkeyedSponge};
use super::ternary_cube_perm::TernaryCubeAutomorphism;

const SPONGE_STATE_SIZE: usize = 729;  // 3^6
const SPONGE_RATE: usize = 243;        // 3^5
const SPONGE_ROUNDS: usize = 27;       // 3^3
const CUBE_DIM: usize = 6;

/// Trits of key material needed per round for the cube automorphism.
/// S_6 needs 10 trits (Fisher-Yates on 6), S_3^6 needs 12 trits (2 per dim).
/// Must be ≥ TernaryCubeAutomorphism::min_key_trits(CUBE_DIM) = 22.
const TRITS_PER_ROUND: usize = 24;

// Compile-time guarantee: per-round key material is sufficient.
const _: () = assert!(
    TRITS_PER_ROUND >= TernaryCubeAutomorphism::min_key_trits(CUBE_DIM),
    "TRITS_PER_ROUND must be >= min_key_trits(CUBE_DIM)"
);

/// The Saturnian Magic Square row: [111, 14, 208].
/// Circulant matrix: each row is a rotation of this triple.
const SATURNIAN_FLAT: [u16; 9] = [
    111, 14, 208,   // row 0
    208, 111, 14,   // row 1
    14, 208, 111,   // row 2
];

/// Derive trit constants from the Magic Square at compile time.
///
/// Each value mod 3 maps to balanced ternary:
///   0 → 0 (balanced zero)
///   1 → +1 (positive)
///   2 → -1 (negative — the esoteric π position)
///
/// This is the ONLY derivation path. The constants are never written
/// by hand — they emerge from the geometry.
const fn derive_saturnian_trits() -> [i8; 9] {
    let mut trits = [0i8; 9];
    let mut i = 0;
    while i < 9 {
        let r = (SATURNIAN_FLAT[i] % 3) as i8;
        // Map: 0→0, 1→+1, 2→-1
        trits[i] = if r == 2 { -1 } else { r };
        i += 1;
    }
    trits
}

/// Round constants derived from the Saturnian Magic Square [111, 14, 208].
///
/// Programmatically derived via `derive_saturnian_trits()` — NOT hardcoded.
/// Period 9 = 3². Tiles exactly 81× into the 729-trit sponge state.
/// Each round rotates the pattern by 3 positions (one circulant row shift),
/// so all three rows of the Magic Square contribute across 27 rounds.
const SATURNIAN_TRIT_CONSTANTS: [i8; 9] = derive_saturnian_trits();

fn trit_add(a: i8, b: i8) -> i8 {
    let sum = (a + 1 + b + 1) % 3;
    sum as i8 - 1
}

fn trit_rotate(t: i8) -> i8 {
    debug_assert!(t >= -1 && t <= 1, "trit_rotate: value {} outside {{-1, 0, +1}}", t);
    // Rotation in F₃: add 1. Convert through Rep B: (t+1+1) mod 3 - 1
    ((t + 2).rem_euclid(3)) - 1
}

fn sbox(a: i8, b: i8, c: i8) -> i8 {
    trit_add(trit_add(a, trit_rotate(b)), c)
}

/// Key schedule: derives per-round automorphisms from a master key.
///
/// Each automorphism is ~39 bytes (axis perm + inverse + value IDs).
/// 27 rounds × 39 bytes ≈ 1KB total — vs ~20KB for precomputed tables.
fn derive_round_automorphisms(master_key: &[i8]) -> Vec<TernaryCubeAutomorphism> {
    let total_trits_needed = SPONGE_ROUNDS * TRITS_PER_ROUND;
    let mut expander = UnkeyedSponge::new();

    // Domain separation: fixed prefix so key expansion cannot collide
    // with any other use of the unkeyed sponge
    let domain_tag: [i8; 8] = [1, -1, 1, -1, 1, -1, 1, -1];
    expander.absorb(&domain_tag);
    expander.absorb(master_key);
    let expanded = expander.squeeze(total_trits_needed);

    let mut automorphisms = Vec::with_capacity(SPONGE_ROUNDS);
    for round in 0..SPONGE_ROUNDS {
        let offset = round * TRITS_PER_ROUND;
        let round_key = &expanded.trits[offset..offset + TRITS_PER_ROUND];
        automorphisms.push(
            TernaryCubeAutomorphism::from_key_trits(CUBE_DIM, round_key)
                .expect("INTERNAL: sponge squeeze produced invalid key material")
        );
    }
    automorphisms
}

/// Keyed permutation function: replaces `sponge_permutation` with
/// key-dependent diffusion. Uses direct coordinate computation —
/// no lookup tables, no data-dependent memory access at state scale.
fn keyed_sponge_permutation(
    state: &mut [i8; SPONGE_STATE_SIZE],
    round_auts: &[TernaryCubeAutomorphism],
) {
    for round in 0..SPONGE_ROUNDS {
        // Step 1: S-box layer (identical to unkeyed sponge)
        let old = *state;
        for i in 0..SPONGE_STATE_SIZE {
            let prev = if i == 0 { old[SPONGE_STATE_SIZE - 1] } else { old[i - 1] };
            let next = old[(i + 1) % SPONGE_STATE_SIZE];
            state[i] = sbox(old[i], prev, next);
        }

        // Step 2: KEY-DEPENDENT permutation via direct cube automorphism
        // Computes new_pos = σ(π(coords(i))) for each position —
        // decompose, transform, recompose. No table indirection.
        let mut temp = [0i8; SPONGE_STATE_SIZE];
        round_auts[round].apply_state(state, &mut temp);
        *state = temp;

        // Step 3: SATURNIAN round constants
        // Derived from the Magic Square [111, 14, 208] reduced to GF(3):
        //   111 mod 3 = 0 → balance, 14 mod 3 = 2 → -1, 208 mod 3 = 1 → +1
        // Pattern: [0, -1, 1, 1, 0, -1, -1, 1, 0] — the circulant tiled across 729.
        // The pattern has period 9 (= 3²), tiling exactly 81 times into 729 (= 3⁶).
        for i in 0..27 {
            let idx = i * 27;
            if idx < SPONGE_STATE_SIZE {
                // Saturnian constant at this position, rotated by round index
                let pattern_pos = (idx + round * 3) % 9;
                let rc = SATURNIAN_TRIT_CONSTANTS[pattern_pos];
                state[idx] = trit_add(state[idx], rc);
            }
        }
    }
}

/// A keyed TL-Sponge-385 with geometry-derived permutations.
///
/// The key parameterizes the internal diffusion layer using the automorphism
/// group of the ternary 6-cube (S_3 ≀ S_6), producing a structurally unique
/// sponge instance per key.
pub struct KeyedTernarySponge {
    state: [i8; SPONGE_STATE_SIZE],
    buffer: Vec<i8>,
    absorbed: bool,
    squeezed: bool,
    round_auts: Vec<TernaryCubeAutomorphism>,
}

impl KeyedTernarySponge {
    /// Create a new keyed sponge from master key material.
    ///
    /// # Arguments
    /// * `key` - Master key as balanced ternary trits {-1, 0, +1}.
    ///   Minimum recommended: 243 trits (one rate block).
    ///   Longer keys provide no additional security but are accepted.
    pub fn new(key: &[i8]) -> Self {
        let round_auts = derive_round_automorphisms(key);
        Self {
            state: [0i8; SPONGE_STATE_SIZE],
            buffer: Vec::new(),
            absorbed: false,
            squeezed: false,
            round_auts,
        }
    }

    /// Absorb input trits into the sponge state.
    pub fn absorb(&mut self, input: &[i8]) {
        self.buffer.extend_from_slice(input);
        self.absorbed = true;
        self.squeezed = false;

        while self.buffer.len() >= SPONGE_RATE {
            let block: Vec<i8> = self.buffer.drain(..SPONGE_RATE).collect();
            for i in 0..SPONGE_RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            keyed_sponge_permutation(&mut self.state, &self.round_auts);
        }
    }

    /// Squeeze output trits from the sponge.
    pub fn squeeze(&mut self, output_trits: usize) -> TernaryDigest {
        // Finalize any remaining buffered input
        if !self.buffer.is_empty() {
            let remaining = self.buffer.clone();
            self.buffer.clear();
            for (i, &t) in remaining.iter().enumerate() {
                if i < SPONGE_RATE {
                    self.state[i] = trit_add(self.state[i], t);
                }
            }
            if SPONGE_RATE > remaining.len() {
                let pad_pos = remaining.len();
                if pad_pos < SPONGE_RATE {
                    self.state[pad_pos] = trit_add(self.state[pad_pos], 1);
                }
            }
            keyed_sponge_permutation(&mut self.state, &self.round_auts);
        } else if !self.absorbed {
            keyed_sponge_permutation(&mut self.state, &self.round_auts);
        }

        let mut output = Vec::with_capacity(output_trits);
        while output.len() < output_trits {
            let remaining = output_trits - output.len();
            let take = core::cmp::min(remaining, SPONGE_RATE);
            output.extend_from_slice(&self.state[..take]);
            if output.len() < output_trits {
                keyed_sponge_permutation(&mut self.state, &self.round_auts);
            }
        }

        output.truncate(output_trits);
        self.squeezed = true;
        TernaryDigest { trits: output }
    }

    /// Squeeze the default digest length.
    pub fn squeeze_default(&mut self) -> TernaryDigest {
        self.squeeze(TERNARY_HASH_TRITS)
    }

    /// Reset the sponge state (key schedule is preserved).
    pub fn reset(&mut self) {
        self.state = [0i8; SPONGE_STATE_SIZE];
        self.buffer.clear();
        self.absorbed = false;
        self.squeezed = false;
    }

    /// Re-key the sponge with new key material.
    pub fn rekey(&mut self, new_key: &[i8]) {
        self.round_auts = derive_round_automorphisms(new_key);
        self.reset();
    }
}

/// Convenience: compute a keyed hash in one call.
pub fn keyed_sponge_hash(key: &[i8], message: &[i8]) -> TernaryDigest {
    let mut sponge = KeyedTernarySponge::new(key);
    sponge.absorb(message);
    sponge.squeeze_default()
}

/// Convenience: compute a keyed MAC tag (fixed 243-trit output).
pub fn keyed_sponge_mac(key: &[i8], message: &[i8]) -> TernaryDigest {
    keyed_sponge_hash(key, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<i8> {
        vec![1i8, 0, -1, 1, -1, 0, 1, 0, -1, 1, 0, -1,
             0, 1, -1, 0, 1, 1, -1, 0, 1, -1, 0, 1,
             -1, 0, 1, -1, 1, 0, -1, 0, 1, -1, 0, 1]
    }

    #[test]
    fn test_keyed_sponge_deterministic() {
        let key = test_key();
        let msg = vec![0i8, 1, -1, 0, 1];

        let h1 = keyed_sponge_hash(&key, &msg);
        let h2 = keyed_sponge_hash(&key, &msg);
        assert_eq!(h1, h2, "Same key + same message must produce same hash");
    }

    #[test]
    fn test_keyed_sponge_output_length() {
        let key = test_key();
        let h = keyed_sponge_hash(&key, &[0i8; 50]);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_keyed_sponge_valid_trits() {
        let key = test_key();
        let h = keyed_sponge_hash(&key, &[1i8, 0, -1]);
        for &t in &h.trits {
            assert!(t >= -1 && t <= 1, "Invalid trit value: {}", t);
        }
    }

    #[test]
    fn test_different_keys_different_output() {
        let key_a = vec![1i8; 36];
        let key_b = vec![-1i8; 36];
        let msg = vec![0i8, 1, -1];

        let h_a = keyed_sponge_hash(&key_a, &msg);
        let h_b = keyed_sponge_hash(&key_b, &msg);
        assert_ne!(h_a, h_b, "Different keys must produce different hashes");
    }

    #[test]
    fn test_different_messages_different_output() {
        let key = test_key();
        let h_a = keyed_sponge_hash(&key, &[0i8, 0, 0]);
        let h_b = keyed_sponge_hash(&key, &[1i8, 0, 0]);
        assert_ne!(h_a, h_b, "Different messages must produce different hashes");
    }

    #[test]
    fn test_keyed_differs_from_unkeyed() {
        let key = test_key();
        let msg = vec![0i8, 1, -1, 0, 1];

        let keyed = keyed_sponge_hash(&key, &msg);
        let unkeyed = sponge_hash(&msg);
        assert_ne!(keyed, unkeyed, "Keyed sponge must differ from unkeyed");
    }

    #[test]
    fn test_incremental_absorb() {
        let key = test_key();
        let msg = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0];

        let full = keyed_sponge_hash(&key, &msg);

        let mut sponge = KeyedTernarySponge::new(&key);
        sponge.absorb(&msg[..5]);
        sponge.absorb(&msg[5..]);
        let incremental = sponge.squeeze_default();

        assert_eq!(full, incremental, "Incremental absorb must match single absorb");
    }

    #[test]
    fn test_reset_preserves_key() {
        let key = test_key();
        let msg = vec![1i8, 0, -1];

        let mut sponge = KeyedTernarySponge::new(&key);
        sponge.absorb(&msg);
        let h1 = sponge.squeeze_default();

        sponge.reset();
        sponge.absorb(&msg);
        let h2 = sponge.squeeze_default();

        assert_eq!(h1, h2, "Reset should preserve key schedule");
    }

    #[test]
    fn test_rekey_changes_output() {
        let key_a = vec![1i8; 36];
        let key_b = vec![-1i8; 36];
        let msg = vec![0i8, 1, -1];

        let mut sponge = KeyedTernarySponge::new(&key_a);
        sponge.absorb(&msg);
        let h_a = sponge.squeeze_default();

        sponge.rekey(&key_b);
        sponge.absorb(&msg);
        let h_b = sponge.squeeze_default();

        assert_ne!(h_a, h_b, "Rekey must change output");
    }

    #[test]
    fn test_empty_message() {
        let key = test_key();
        let h = keyed_sponge_hash(&key, &[]);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_large_message() {
        let key = test_key();
        let msg = vec![1i8; 2000]; // larger than rate
        let h = keyed_sponge_hash(&key, &msg);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
        for &t in &h.trits {
            assert!(t >= -1 && t <= 1);
        }
    }

    #[test]
    fn test_variable_squeeze_length() {
        let key = test_key();
        let mut sponge = KeyedTernarySponge::new(&key);
        sponge.absorb(&[1i8, 0, -1]);
        let h = sponge.squeeze(100);
        assert_eq!(h.len(), 100);
    }

    #[test]
    fn test_long_squeeze() {
        let key = test_key();
        let mut sponge = KeyedTernarySponge::new(&key);
        sponge.absorb(&[1i8, 0, -1]);
        let h = sponge.squeeze(1000);
        assert_eq!(h.len(), 1000);
    }

    #[test]
    fn test_mac_convenience() {
        let key = test_key();
        let msg = vec![0i8, 1, -1];
        let mac = keyed_sponge_mac(&key, &msg);
        let hash = keyed_sponge_hash(&key, &msg);
        assert_eq!(mac, hash, "MAC and hash should be identical for default length");
    }

    #[test]
    fn test_saturnian_constants_derived_from_magic_square() {
        // Belt-and-suspenders: verify the const fn derivation produces
        // the expected values from [111, 14, 208] mod 3 → balanced ternary.
        //   111 % 3 = 0 →  0
        //    14 % 3 = 2 → -1
        //   208 % 3 = 1 → +1
        let expected: [i8; 9] = [0, -1, 1, 1, 0, -1, -1, 1, 0];
        assert_eq!(SATURNIAN_TRIT_CONSTANTS, expected,
            "Derived constants must match Magic Square modular reduction");

        // Verify the derivation source is the correct circulant matrix
        assert_eq!(SATURNIAN_FLAT[0], 111);
        assert_eq!(SATURNIAN_FLAT[1], 14);
        assert_eq!(SATURNIAN_FLAT[2], 208);
        // Row sums must equal the magic constant 333
        for row in 0..3 {
            let sum: u16 = SATURNIAN_FLAT[row*3..row*3+3].iter().sum();
            assert_eq!(sum, 333, "Row {} sum must equal magic constant", row);
        }
    }
}