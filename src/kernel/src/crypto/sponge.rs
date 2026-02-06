//! Ternary Sponge Construction
//!
//! Keccak-inspired sponge construction operating in the ternary domain.
//! Provides an incremental hashing interface where data can be absorbed
//! in multiple calls before squeezing out the digest.

use alloc::vec::Vec;
use super::{CryptoResult, TernaryDigest, TERNARY_HASH_TRITS};

const SPONGE_STATE_SIZE: usize = 729;
const SPONGE_RATE: usize = 243;
const SPONGE_ROUNDS: usize = 27;

fn trit_add(a: i8, b: i8) -> i8 {
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
    trit_add(trit_add(a, trit_rotate(b)), c)
}

fn sponge_permutation(state: &mut [i8; SPONGE_STATE_SIZE]) {
    for round in 0..SPONGE_ROUNDS {
        let old = *state;
        for i in 0..SPONGE_STATE_SIZE {
            let prev = if i == 0 { old[SPONGE_STATE_SIZE - 1] } else { old[i - 1] };
            let next = old[(i + 1) % SPONGE_STATE_SIZE];
            state[i] = sbox(old[i], prev, next);
        }

        let mut temp = [0i8; SPONGE_STATE_SIZE];
        for i in 0..SPONGE_STATE_SIZE {
            let new_pos = (i * 376 + 1) % SPONGE_STATE_SIZE;
            temp[new_pos] = state[i];
        }
        *state = temp;

        for i in 0..27 {
            let idx = i * 27;
            if idx < SPONGE_STATE_SIZE {
                let rc = (((round * 7 + i * 13 + 3) % 3) as i8) - 1;
                state[idx] = trit_add(state[idx], rc);
            }
        }
    }
}

pub struct TernarySponge {
    state: [i8; SPONGE_STATE_SIZE],
    buffer: Vec<i8>,
    absorbed: bool,
    squeezed: bool,
}

impl TernarySponge {
    pub fn new() -> Self {
        Self {
            state: [0i8; SPONGE_STATE_SIZE],
            buffer: Vec::new(),
            absorbed: false,
            squeezed: false,
        }
    }

    pub fn absorb(&mut self, input: &[i8]) {
        self.buffer.extend_from_slice(input);
        self.absorbed = true;
        self.squeezed = false;

        while self.buffer.len() >= SPONGE_RATE {
            let block: Vec<i8> = self.buffer.drain(..SPONGE_RATE).collect();
            for i in 0..SPONGE_RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            sponge_permutation(&mut self.state);
        }
    }

    pub fn absorb_bytes(&mut self, input: &[u8]) {
        let trits = bytes_to_trits(input);
        self.absorb(&trits);
    }

    pub fn squeeze(&mut self, output_trits: usize) -> TernaryDigest {
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
            sponge_permutation(&mut self.state);
        } else if !self.absorbed {
            sponge_permutation(&mut self.state);
        }

        let mut output = Vec::with_capacity(output_trits);

        while output.len() < output_trits {
            let remaining = output_trits - output.len();
            let take = core::cmp::min(remaining, SPONGE_RATE);
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
        self.buffer.clear();
        self.absorbed = false;
        self.squeezed = false;
    }
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sponge_creation() {
        let sponge = TernarySponge::new();
        assert!(!sponge.absorbed);
        assert!(!sponge.squeezed);
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
}
