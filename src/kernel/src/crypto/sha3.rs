//! SHA-3 Implementation (FIPS 202) — Internal Hardware Use Only
//!
//! Implements SHA3-384 and SHA3-512 using the Keccak-f[1600] permutation
//! for internal hardware integrity functions only (boot verification,
//! TPU FPGA integrity checks).
//!
//! # Scope Restriction
//! Per CNSA 2.0 guidance, SHA-3 is approved for internal hardware functions
//! only and is NOT intended for general cryptographic use. SHA-384/SHA-512
//! (FIPS 180-4, in sha2.rs) should be used for TLS, IPsec, and other
//! protocol-level operations.
//!
//! # Use Cases
//! - Kernel boot integrity verification
//! - TPU FPGA bitstream validation
//! - Hardware driver integrity checks
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;

const KECCAK_LANES: usize = 25;
const KECCAK_ROUNDS: usize = 24;
#[allow(dead_code)]
const KECCAK_STATE_BYTES: usize = 200;

const SHA3_384_RATE: usize = 104;
const SHA3_512_RATE: usize = 72;
const SHA3_384_DIGEST: usize = 48;
const SHA3_512_DIGEST: usize = 64;

static RC: [u64; KECCAK_ROUNDS] = [
    0x0000000000000001, 0x0000000000008082,
    0x800000000000808A, 0x8000000080008000,
    0x000000000000808B, 0x0000000080000001,
    0x8000000080008081, 0x8000000000008009,
    0x000000000000008A, 0x0000000000000088,
    0x0000000080008009, 0x000000008000000A,
    0x000000008000808B, 0x800000000000008B,
    0x8000000000008089, 0x8000000000008003,
    0x8000000000008002, 0x8000000000000080,
    0x000000000000800A, 0x800000008000000A,
    0x8000000080008081, 0x8000000000008080,
    0x0000000080000001, 0x8000000080008008,
];

static ROT_OFFSETS: [u32; KECCAK_LANES] = [
     0,  1, 62, 28, 27,
    36, 44,  6, 55, 20,
     3, 10, 43, 25, 39,
    41, 45, 15, 21,  8,
    18,  2, 61, 56, 14,
];

static PI_INDICES: [usize; KECCAK_LANES] = [
     0, 10, 20,  5, 15,
    16,  1, 11, 21,  6,
     7, 17,  2, 12, 22,
    23,  8, 18,  3, 13,
    14, 24,  9, 19,  4,
];

fn keccak_f1600(state: &mut [u64; KECCAK_LANES]) {
    for round in 0..KECCAK_ROUNDS {
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }

        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
        }

        for x in 0..5 {
            for y in 0..5 {
                state[x + 5 * y] ^= d[x];
            }
        }

        let mut b = [0u64; KECCAK_LANES];
        for i in 0..KECCAK_LANES {
            b[PI_INDICES[i]] = state[i].rotate_left(ROT_OFFSETS[i]);
        }

        for x in 0..5 {
            for y in 0..5 {
                state[x + 5 * y] = b[x + 5 * y] ^ (!b[(x + 1) % 5 + 5 * y] & b[(x + 2) % 5 + 5 * y]);
            }
        }

        state[0] ^= RC[round];
    }
}

#[allow(dead_code)]
fn keccak_absorb(state: &mut [u64; KECCAK_LANES], rate: usize, data: &[u8]) {
    let rate_lanes = rate / 8;

    for chunk in data.chunks(rate) {
        let mut block = [0u8; KECCAK_STATE_BYTES];
        block[..chunk.len()].copy_from_slice(chunk);

        for i in 0..rate_lanes {
            if i * 8 + 8 <= rate {
                state[i] ^= u64::from_le_bytes([
                    block[i * 8],
                    block[i * 8 + 1],
                    block[i * 8 + 2],
                    block[i * 8 + 3],
                    block[i * 8 + 4],
                    block[i * 8 + 5],
                    block[i * 8 + 6],
                    block[i * 8 + 7],
                ]);
            }
        }

        if chunk.len() == rate {
            keccak_f1600(state);
        }
    }
}

fn sha3_core(rate: usize, digest_len: usize, message: &[u8]) -> Vec<u8> {
    let mut state = [0u64; KECCAK_LANES];

    let full_blocks = message.len() / rate;
    let remaining = message.len() % rate;

    if full_blocks > 0 {
        let full_data = &message[..full_blocks * rate];
        for chunk in full_data.chunks_exact(rate) {
            let rate_lanes = rate / 8;
            for i in 0..rate_lanes {
                state[i] ^= u64::from_le_bytes([
                    chunk[i * 8],
                    chunk[i * 8 + 1],
                    chunk[i * 8 + 2],
                    chunk[i * 8 + 3],
                    chunk[i * 8 + 4],
                    chunk[i * 8 + 5],
                    chunk[i * 8 + 6],
                    chunk[i * 8 + 7],
                ]);
            }
            keccak_f1600(&mut state);
        }
    }

    let mut last_block = alloc::vec![0u8; rate];
    if remaining > 0 {
        last_block[..remaining].copy_from_slice(&message[full_blocks * rate..]);
    }
    last_block[remaining] = 0x06;
    last_block[rate - 1] |= 0x80;

    let rate_lanes = rate / 8;
    for i in 0..rate_lanes {
        state[i] ^= u64::from_le_bytes([
            last_block[i * 8],
            last_block[i * 8 + 1],
            last_block[i * 8 + 2],
            last_block[i * 8 + 3],
            last_block[i * 8 + 4],
            last_block[i * 8 + 5],
            last_block[i * 8 + 6],
            last_block[i * 8 + 7],
        ]);
    }
    keccak_f1600(&mut state);

    let mut digest = Vec::with_capacity(digest_len);
    let mut offset = 0;
    while digest.len() < digest_len {
        let lane_idx = offset / 8;
        if lane_idx < KECCAK_LANES {
            let bytes = state[lane_idx].to_le_bytes();
            let start = offset % 8;
            for j in start..8 {
                if digest.len() >= digest_len {
                    break;
                }
                digest.push(bytes[j]);
            }
        }
        offset = (lane_idx + 1) * 8;

        if offset >= rate && digest.len() < digest_len {
            keccak_f1600(&mut state);
            offset = 0;
        }
    }

    digest.truncate(digest_len);
    digest
}

pub fn sha3_384(message: &[u8]) -> [u8; SHA3_384_DIGEST] {
    let d = sha3_core(SHA3_384_RATE, SHA3_384_DIGEST, message);
    let mut result = [0u8; SHA3_384_DIGEST];
    result.copy_from_slice(&d);
    result
}

pub fn sha3_512(message: &[u8]) -> [u8; SHA3_512_DIGEST] {
    let d = sha3_core(SHA3_512_RATE, SHA3_512_DIGEST, message);
    let mut result = [0u8; SHA3_512_DIGEST];
    result.copy_from_slice(&d);
    result
}

pub fn sha3_384_boot_verify(image: &[u8], expected: &[u8; SHA3_384_DIGEST]) -> bool {
    let computed = sha3_384(image);
    let mut diff: u8 = 0;
    for i in 0..SHA3_384_DIGEST {
        diff |= computed[i] ^ expected[i];
    }
    diff == 0
}

pub fn sha3_512_fpga_verify(bitstream: &[u8], expected: &[u8; SHA3_512_DIGEST]) -> bool {
    let computed = sha3_512(bitstream);
    let mut diff: u8 = 0;
    for i in 0..SHA3_512_DIGEST {
        diff |= computed[i] ^ expected[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_512_empty() {
        let hash = sha3_512(b"");
        let expected: [u8; 64] = [
            0xa6, 0x9f, 0x73, 0xcc, 0xa2, 0x3a, 0x9a, 0xc5,
            0xc8, 0xb5, 0x67, 0xdc, 0x18, 0x5a, 0x75, 0x6e,
            0x97, 0xc9, 0x82, 0x16, 0x4f, 0xe2, 0x58, 0x59,
            0xe0, 0xd1, 0xdc, 0xc1, 0x47, 0x5c, 0x80, 0xa6,
            0x15, 0xb2, 0x12, 0x3a, 0xf1, 0xf5, 0xf9, 0x4c,
            0x11, 0xe3, 0xe9, 0x40, 0x2c, 0x3a, 0xc5, 0x58,
            0xf5, 0x00, 0x19, 0x9d, 0x95, 0xb6, 0xd3, 0xe3,
            0x01, 0x75, 0x85, 0x86, 0x28, 0x1d, 0xcd, 0x26,
        ];
        assert_eq!(hash, expected, "SHA3-512 empty string NIST vector failed");
    }

    #[test]
    fn test_sha3_384_empty() {
        let hash = sha3_384(b"");
        let expected: [u8; 48] = [
            0x0c, 0x63, 0xa7, 0x5b, 0x84, 0x5e, 0x4f, 0x7d,
            0x01, 0x10, 0x7d, 0x85, 0x2e, 0x4c, 0x24, 0x85,
            0xc5, 0x1a, 0x50, 0xaa, 0xaa, 0x94, 0xfc, 0x61,
            0x99, 0x5e, 0x71, 0xbb, 0xee, 0x98, 0x3a, 0x2a,
            0xc3, 0x71, 0x38, 0x31, 0x26, 0x4a, 0xdb, 0x47,
            0xfb, 0x6b, 0xd1, 0xe0, 0x58, 0xd5, 0xf0, 0x04,
        ];
        assert_eq!(hash, expected, "SHA3-384 empty string NIST vector failed");
    }

    #[test]
    fn test_sha3_512_abc() {
        let hash = sha3_512(b"abc");
        let expected: [u8; 64] = [
            0xb7, 0x51, 0x85, 0x0b, 0x1a, 0x57, 0x16, 0x8a,
            0x56, 0x93, 0xcd, 0x92, 0x4b, 0x6b, 0x09, 0x6e,
            0x08, 0xf6, 0x21, 0x82, 0x74, 0x44, 0xf7, 0x0d,
            0x88, 0x4f, 0x5d, 0x02, 0x40, 0xd2, 0x71, 0x2e,
            0x10, 0xe1, 0x16, 0xe9, 0x19, 0x2a, 0xf3, 0xc9,
            0x1a, 0x7e, 0xc5, 0x76, 0x47, 0xe3, 0x93, 0x40,
            0x57, 0x34, 0x0b, 0x4c, 0xf4, 0x08, 0xd5, 0xa5,
            0x65, 0x92, 0xf8, 0x27, 0x4e, 0xec, 0x53, 0xf0,
        ];
        assert_eq!(hash, expected, "SHA3-512 'abc' NIST vector failed");
    }

    #[test]
    fn test_sha3_384_abc() {
        let hash = sha3_384(b"abc");
        let expected: [u8; 48] = [
            0xec, 0x01, 0x49, 0x82, 0x88, 0x51, 0x6f, 0xc9,
            0x26, 0x45, 0x9f, 0x58, 0xe2, 0xc6, 0xad, 0x8d,
            0xf9, 0xb4, 0x73, 0xcb, 0x0f, 0xc0, 0x8c, 0x25,
            0x96, 0xda, 0x7c, 0xf0, 0xe4, 0x9b, 0xe4, 0xb2,
            0x98, 0xd8, 0x8c, 0xea, 0x92, 0x7a, 0xc7, 0xf5,
            0x39, 0xf1, 0xed, 0xf2, 0x28, 0x37, 0x6d, 0x25,
        ];
        assert_eq!(hash, expected, "SHA3-384 'abc' NIST vector failed");
    }

    #[test]
    fn test_sha3_512_deterministic() {
        let h1 = sha3_512(b"test");
        let h2 = sha3_512(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha3_384_deterministic() {
        let h1 = sha3_384(b"test");
        let h2 = sha3_384(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha3_512_different_inputs() {
        let h1 = sha3_512(b"hello");
        let h2 = sha3_512(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha3_512_long_message() {
        let msg = alloc::vec![0x61u8; 1000];
        let hash = sha3_512(&msg);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha3_384_long_message() {
        let msg = alloc::vec![0x61u8; 1000];
        let hash = sha3_384(&msg);
        assert_eq!(hash.len(), 48);
    }

    #[test]
    fn test_boot_verify_correct() {
        let image = b"kernel image data for testing";
        let hash = sha3_384(image);
        assert!(sha3_384_boot_verify(image, &hash));
    }

    #[test]
    fn test_boot_verify_tampered() {
        let image = b"kernel image data for testing";
        let hash = sha3_384(image);
        let tampered = b"kernel image data TAMPERED!!!!";
        assert!(!sha3_384_boot_verify(tampered, &hash));
    }

    #[test]
    fn test_fpga_verify_correct() {
        let bitstream = b"fpga bitstream content for verification";
        let hash = sha3_512(bitstream);
        assert!(sha3_512_fpga_verify(bitstream, &hash));
    }

    #[test]
    fn test_fpga_verify_tampered() {
        let bitstream = b"fpga bitstream content for verification";
        let hash = sha3_512(bitstream);
        let tampered = b"fpga bitstream MODIFIED content!!!!!!!";
        assert!(!sha3_512_fpga_verify(tampered, &hash));
    }

    #[test]
    fn test_keccak_permutation_not_identity() {
        let mut state = [0u64; KECCAK_LANES];
        state[0] = 1;
        let original = state;
        keccak_f1600(&mut state);
        assert_ne!(state, original);
    }
}
