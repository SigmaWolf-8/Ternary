//! Ternary Hash Function
//!
//! Implements a hash function operating on trit-based data using
//! a substitution-permutation network in GF(3). Produces 243-trit
//! (48-byte equivalent) digests suitable for the Salvi Framework's
//! post-quantum security requirements.

use alloc::vec::Vec;
use super::{TernaryDigest, TERNARY_HASH_TRITS};

const ROUNDS: usize = 27;
const STATE_SIZE: usize = 729;
const RATE: usize = 243;
const CAPACITY: usize = STATE_SIZE - RATE;

fn trit_add_gf3(a: i8, b: i8) -> i8 {
    let sum = (a + 1 + b + 1) % 3;
    sum as i8 - 1
}

fn trit_rotate(t: i8) -> i8 {
    match t {
        -1 => 0,
        0 => 1,
        1 => -1,
        _ => 0,
    }
}

fn sbox(a: i8, b: i8, c: i8) -> i8 {
    let rotated = trit_rotate(b);
    let mixed = trit_add_gf3(a, rotated);
    trit_add_gf3(mixed, c)
}

fn substitution(state: &mut [i8; STATE_SIZE]) {
    let old = *state;
    for i in 0..STATE_SIZE {
        let prev = if i == 0 { old[STATE_SIZE - 1] } else { old[i - 1] };
        let next = old[(i + 1) % STATE_SIZE];
        state[i] = sbox(old[i], prev, next);
    }
}

fn permutation(state: &mut [i8; STATE_SIZE]) {
    let mut temp = [0i8; STATE_SIZE];
    for i in 0..STATE_SIZE {
        let new_pos = (i * 376 + 1) % STATE_SIZE;
        temp[new_pos] = state[i];
    }
    *state = temp;
}

fn round_constant(state: &mut [i8; STATE_SIZE], round: usize) {
    for i in 0..27 {
        let idx = i * 27;
        if idx < STATE_SIZE {
            let rc = (((round * 7 + i * 13 + 3) % 3) as i8) - 1;
            state[idx] = trit_add_gf3(state[idx], rc);
        }
    }
}

fn absorb_block(state: &mut [i8; STATE_SIZE], block: &[i8]) {
    let len = core::cmp::min(block.len(), RATE);
    for i in 0..len {
        state[i] = trit_add_gf3(state[i], block[i]);
    }

    for round in 0..ROUNDS {
        substitution(state);
        permutation(state);
        round_constant(state, round);
    }
}

fn squeeze(state: &[i8; STATE_SIZE], output_trits: usize) -> Vec<i8> {
    state[..output_trits].to_vec()
}

pub fn ternary_hash(input: &[i8]) -> TernaryDigest {
    let mut state = [0i8; STATE_SIZE];

    for chunk in input.chunks(RATE) {
        absorb_block(&mut state, chunk);
    }

    let output = squeeze(&state, TERNARY_HASH_TRITS);
    TernaryDigest { trits: output }
}

pub fn ternary_hash_bytes(input: &[u8]) -> TernaryDigest {
    let trits = bytes_to_trits(input);
    ternary_hash(&trits)
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            let remainder = val % 3;
            trits.push(remainder as i8 - 1);
            val /= 3;
        }
    }
    trits
}

pub fn verify_hash(input: &[i8], expected: &TernaryDigest) -> bool {
    let computed = ternary_hash(input);
    computed == *expected
}

pub fn verify_hash_bytes(input: &[u8], expected: &TernaryDigest) -> bool {
    let computed = ternary_hash_bytes(input);
    computed == *expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_hash_deterministic() {
        let input = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let h1 = ternary_hash(&input);
        let h2 = ternary_hash(&input);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_ternary_hash_output_length() {
        let input = alloc::vec![0i8; 100];
        let h = ternary_hash(&input);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_ternary_hash_valid_trits() {
        let input = alloc::vec![1i8, 0, -1, 1, 0];
        let h = ternary_hash(&input);
        for &t in &h.trits {
            assert!(t >= -1 && t <= 1, "Invalid trit in hash output: {}", t);
        }
    }

    #[test]
    fn test_ternary_hash_different_inputs() {
        let h1 = ternary_hash(&[0i8, 0, 0]);
        let h2 = ternary_hash(&[1i8, 0, 0]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_ternary_hash_empty_input() {
        let h = ternary_hash(&[]);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_ternary_hash_large_input() {
        let input = alloc::vec![1i8; 1000];
        let h = ternary_hash(&input);
        assert_eq!(h.len(), TERNARY_HASH_TRITS);
    }

    #[test]
    fn test_ternary_hash_bytes_deterministic() {
        let input = b"hello world";
        let h1 = ternary_hash_bytes(input);
        let h2 = ternary_hash_bytes(input);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_ternary_hash_bytes_different() {
        let h1 = ternary_hash_bytes(b"hello");
        let h2 = ternary_hash_bytes(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_verify_hash() {
        let input = alloc::vec![0i8, 1, -1, 0, 1];
        let expected = ternary_hash(&input);
        assert!(verify_hash(&input, &expected));

        let wrong_input = alloc::vec![1i8, 0, 0, 0, 0];
        assert!(!verify_hash(&wrong_input, &expected));
    }

    #[test]
    fn test_verify_hash_bytes() {
        let input = b"test data";
        let expected = ternary_hash_bytes(input);
        assert!(verify_hash_bytes(input, &expected));
        assert!(!verify_hash_bytes(b"other data", &expected));
    }

    #[test]
    fn test_bytes_to_trits() {
        let bytes = &[0u8, 255u8];
        let trits = bytes_to_trits(bytes);
        assert_eq!(trits.len(), 10);
        for &t in &trits {
            assert!(t >= -1 && t <= 1);
        }
    }

    #[test]
    fn test_gf3_operations() {
        for a in [-1i8, 0, 1] {
            assert_eq!(trit_add_gf3(a, -1), a, "additive identity (-1 maps to 0 in B-rep)");
        }

        assert_eq!(trit_add_gf3(0, 0), 1);
        assert_eq!(trit_add_gf3(1, 1), 0);
        assert_eq!(trit_add_gf3(-1, -1), -1);
    }
}
