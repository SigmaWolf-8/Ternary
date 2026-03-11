// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! SHA-384 / SHA-512 Implementation (FIPS 180-4)
//!
//! Binary-compatible SHA-2 family implementation for CNSA 2.0 compliance.
//! Provides FIPS 180-4 compliant SHA-384 and SHA-512 hash functions alongside
//! HMAC-SHA-384 and HMAC-SHA-512 for TLS/IPsec key derivation.
//!
//! This module provides interoperability with external systems requiring
//! byte-identical SHA-2 output, complementing TL-Sponge-385
//! used internally by the Salvi Framework.
//!
//! # CNSA 2.0 Requirement
//! SHA-384 is mandatory for TLS 1.3 key derivation (HKDF-SHA-384).
//! SHA-512 is approved for general integrity.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;

const SHA512_BLOCK_SIZE: usize = 128;
const SHA512_DIGEST_SIZE: usize = 64;
const SHA384_DIGEST_SIZE: usize = 48;

static K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

const SHA512_IV: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

const SHA384_IV: [u64; 8] = [
    0xcbbb9d5dc1059ed8, 0x629a292a367cd507,
    0x9159015a3070dd17, 0x152fecd8f70e5939,
    0x67332667ffc00b31, 0x8eb44a8768581511,
    0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4,
];

fn ch(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (!x & z)
}

fn maj(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn big_sigma0(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

fn big_sigma1(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

fn small_sigma0(x: u64) -> u64 {
    x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
}

fn small_sigma1(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
}

fn compress(state: &mut [u64; 8], block: &[u8; SHA512_BLOCK_SIZE]) {
    let mut w = [0u64; 80];

    for i in 0..16 {
        w[i] = u64::from_be_bytes([
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

    for i in 16..80 {
        w[i] = small_sigma1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(small_sigma0(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for i in 0..80 {
        let t1 = h
            .wrapping_add(big_sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

fn pad_message(message: &[u8]) -> Vec<u8> {
    let msg_len = message.len();
    let bit_len = (msg_len as u128) * 8;

    let mut padded = Vec::from(message);
    padded.push(0x80);

    let pad_len = (SHA512_BLOCK_SIZE - ((msg_len + 17) % SHA512_BLOCK_SIZE)) % SHA512_BLOCK_SIZE;
    padded.extend(core::iter::repeat(0u8).take(pad_len));

    padded.extend_from_slice(&bit_len.to_be_bytes());

    padded
}

fn sha2_core(iv: &[u64; 8], message: &[u8], digest_len: usize) -> Vec<u8> {
    let padded = pad_message(message);
    let mut state = *iv;

    for chunk in padded.chunks_exact(SHA512_BLOCK_SIZE) {
        let mut block = [0u8; SHA512_BLOCK_SIZE];
        block.copy_from_slice(chunk);
        compress(&mut state, &block);
    }

    let mut digest = Vec::with_capacity(digest_len);
    for &word in state.iter() {
        digest.extend_from_slice(&word.to_be_bytes());
        if digest.len() >= digest_len {
            break;
        }
    }
    digest.truncate(digest_len);
    digest
}

pub fn sha512(message: &[u8]) -> [u8; SHA512_DIGEST_SIZE] {
    let d = sha2_core(&SHA512_IV, message, SHA512_DIGEST_SIZE);
    let mut result = [0u8; SHA512_DIGEST_SIZE];
    result.copy_from_slice(&d);
    result
}

pub fn sha384(message: &[u8]) -> [u8; SHA384_DIGEST_SIZE] {
    let d = sha2_core(&SHA384_IV, message, SHA384_DIGEST_SIZE);
    let mut result = [0u8; SHA384_DIGEST_SIZE];
    result.copy_from_slice(&d);
    result
}

fn hmac_sha2(key: &[u8], message: &[u8], iv: &[u64; 8], digest_len: usize) -> Vec<u8> {
    let block_key = if key.len() > SHA512_BLOCK_SIZE {
        let hashed = sha2_core(iv, key, digest_len);
        let mut padded = hashed;
        padded.resize(SHA512_BLOCK_SIZE, 0);
        padded
    } else {
        let mut padded = Vec::from(key);
        padded.resize(SHA512_BLOCK_SIZE, 0);
        padded
    };

    let mut ipad = Vec::with_capacity(SHA512_BLOCK_SIZE + message.len());
    for &b in &block_key {
        ipad.push(b ^ 0x36);
    }
    ipad.extend_from_slice(message);
    let inner_hash = sha2_core(iv, &ipad, digest_len);

    let mut opad = Vec::with_capacity(SHA512_BLOCK_SIZE + digest_len);
    for &b in &block_key {
        opad.push(b ^ 0x5c);
    }
    opad.extend_from_slice(&inner_hash);
    sha2_core(iv, &opad, digest_len)
}

pub fn hmac_sha512(key: &[u8], message: &[u8]) -> [u8; SHA512_DIGEST_SIZE] {
    let d = hmac_sha2(key, message, &SHA512_IV, SHA512_DIGEST_SIZE);
    let mut result = [0u8; SHA512_DIGEST_SIZE];
    result.copy_from_slice(&d);
    result
}

pub fn hmac_sha384(key: &[u8], message: &[u8]) -> [u8; SHA384_DIGEST_SIZE] {
    let d = hmac_sha2(key, message, &SHA384_IV, SHA384_DIGEST_SIZE);
    let mut result = [0u8; SHA384_DIGEST_SIZE];
    result.copy_from_slice(&d);
    result
}

pub fn hkdf_sha384_extract(salt: &[u8], ikm: &[u8]) -> [u8; SHA384_DIGEST_SIZE] {
    let effective_salt = if salt.is_empty() {
        [0u8; SHA384_DIGEST_SIZE].to_vec()
    } else {
        salt.to_vec()
    };
    hmac_sha384(&effective_salt, ikm)
}

pub fn hkdf_sha384_expand(prk: &[u8; SHA384_DIGEST_SIZE], info: &[u8], length: usize) -> Vec<u8> {
    let n = (length + SHA384_DIGEST_SIZE - 1) / SHA384_DIGEST_SIZE;
    let mut okm = Vec::with_capacity(n * SHA384_DIGEST_SIZE);
    let mut t = Vec::new();

    for i in 1..=n {
        let mut input = Vec::with_capacity(t.len() + info.len() + 1);
        input.extend_from_slice(&t);
        input.extend_from_slice(info);
        input.push(i as u8);
        let block = hmac_sha384(prk, &input);
        t = block.to_vec();
        okm.extend_from_slice(&t);
    }

    okm.truncate(length);
    okm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha512_empty() {
        let hash = sha512(b"");
        let expected: [u8; 64] = [
            0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd,
            0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d, 0x80, 0x07,
            0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc,
            0x83, 0xf4, 0xa9, 0x21, 0xd3, 0x6c, 0xe9, 0xce,
            0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0,
            0xff, 0x83, 0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f,
            0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81,
            0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e,
        ];
        assert_eq!(hash, expected, "SHA-512 empty string CAVP vector failed");
    }

    #[test]
    fn test_sha512_abc() {
        let hash = sha512(b"abc");
        let expected: [u8; 64] = [
            0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba,
            0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31,
            0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2,
            0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a,
            0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8,
            0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd,
            0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
            0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
        ];
        assert_eq!(hash, expected, "SHA-512 'abc' CAVP vector failed");
    }

    #[test]
    fn test_sha384_empty() {
        let hash = sha384(b"");
        let expected: [u8; 48] = [
            0x38, 0xb0, 0x60, 0xa7, 0x51, 0xac, 0x96, 0x38,
            0x4c, 0xd9, 0x32, 0x7e, 0xb1, 0xb1, 0xe3, 0x6a,
            0x21, 0xfd, 0xb7, 0x11, 0x14, 0xbe, 0x07, 0x43,
            0x4c, 0x0c, 0xc7, 0xbf, 0x63, 0xf6, 0xe1, 0xda,
            0x27, 0x4e, 0xde, 0xbf, 0xe7, 0x6f, 0x65, 0xfb,
            0xd5, 0x1a, 0xd2, 0xf1, 0x48, 0x98, 0xb9, 0x5b,
        ];
        assert_eq!(hash, expected, "SHA-384 empty string CAVP vector failed");
    }

    #[test]
    fn test_sha384_abc() {
        let hash = sha384(b"abc");
        let expected: [u8; 48] = [
            0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b,
            0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6, 0x50, 0x07,
            0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63,
            0x1a, 0x8b, 0x60, 0x5a, 0x43, 0xff, 0x5b, 0xed,
            0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23,
            0x58, 0xba, 0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7,
        ];
        assert_eq!(hash, expected, "SHA-384 'abc' CAVP vector failed");
    }

    #[test]
    fn test_sha512_deterministic() {
        let h1 = sha512(b"test message");
        let h2 = sha512(b"test message");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha512_different_inputs() {
        let h1 = sha512(b"hello");
        let h2 = sha512(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha384_deterministic() {
        let h1 = sha384(b"test");
        let h2 = sha384(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha512_long_message() {
        let msg = alloc::vec![0x61u8; 1000];
        let hash = sha512(&msg);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha384_long_message() {
        let msg = alloc::vec![0x61u8; 1000];
        let hash = sha384(&msg);
        assert_eq!(hash.len(), 48);
    }

    #[test]
    fn test_hmac_sha512_basic() {
        let key = b"secret key";
        let msg = b"message";
        let mac1 = hmac_sha512(key, msg);
        let mac2 = hmac_sha512(key, msg);
        assert_eq!(mac1, mac2);

        let mac3 = hmac_sha512(b"different key", msg);
        assert_ne!(mac1, mac3);
    }

    #[test]
    fn test_hmac_sha384_basic() {
        let key = b"secret key";
        let msg = b"message";
        let mac1 = hmac_sha384(key, msg);
        let mac2 = hmac_sha384(key, msg);
        assert_eq!(mac1, mac2);
    }

    #[test]
    fn test_hmac_sha512_rfc4231_test1() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha512(&key, data);
        let expected: [u8; 64] = [
            0x87, 0xaa, 0x7c, 0xde, 0xa5, 0xef, 0x61, 0x9d,
            0x4f, 0xf0, 0xb4, 0x24, 0x1a, 0x1d, 0x6c, 0xb0,
            0x23, 0x79, 0xf4, 0xe2, 0xce, 0x4e, 0xc2, 0x78,
            0x7a, 0xd0, 0xb3, 0x05, 0x45, 0xe1, 0x7c, 0xde,
            0xda, 0xa8, 0x33, 0xb7, 0xd6, 0xb8, 0xa7, 0x02,
            0x03, 0x8b, 0x27, 0x4e, 0xae, 0xa3, 0xf4, 0xe4,
            0xbe, 0x9d, 0x91, 0x4e, 0xeb, 0x61, 0xf1, 0x70,
            0x2e, 0x69, 0x6c, 0x20, 0x3a, 0x12, 0x68, 0x54,
        ];
        assert_eq!(mac, expected, "HMAC-SHA-512 RFC 4231 Test Case 1 failed");
    }

    #[test]
    fn test_hmac_sha384_rfc4231_test1() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha384(&key, data);
        let expected: [u8; 48] = [
            0xaf, 0xd0, 0x39, 0x44, 0xd8, 0x48, 0x95, 0x62,
            0x6b, 0x08, 0x25, 0xf4, 0xab, 0x46, 0x90, 0x7f,
            0x15, 0xf9, 0xda, 0xdb, 0xe4, 0x10, 0x1e, 0xc6,
            0x82, 0xaa, 0x03, 0x4c, 0x7c, 0xeb, 0xc5, 0x9c,
            0xfa, 0xea, 0x9e, 0xa9, 0x07, 0x6e, 0xde, 0x7f,
            0x4a, 0xf1, 0x52, 0xe8, 0xb2, 0xfa, 0x9c, 0xb6,
        ];
        assert_eq!(mac, expected, "HMAC-SHA-384 RFC 4231 Test Case 1 failed");
    }

    #[test]
    fn test_hmac_sha512_different_messages() {
        let key = b"key";
        let mac1 = hmac_sha512(key, b"msg1");
        let mac2 = hmac_sha512(key, b"msg2");
        assert_ne!(mac1, mac2);
    }

    #[test]
    fn test_hmac_long_key() {
        let long_key = alloc::vec![0x42u8; 200];
        let mac = hmac_sha512(&long_key, b"test");
        assert_eq!(mac.len(), 64);
    }

    #[test]
    fn test_hkdf_sha384_extract() {
        let salt = b"salt value";
        let ikm = b"input key material";
        let prk = hkdf_sha384_extract(salt, ikm);
        assert_eq!(prk.len(), 48);

        let prk2 = hkdf_sha384_extract(salt, ikm);
        assert_eq!(prk, prk2);
    }

    #[test]
    fn test_hkdf_sha384_expand() {
        let prk = hkdf_sha384_extract(b"salt", b"ikm");
        let okm = hkdf_sha384_expand(&prk, b"info", 64);
        assert_eq!(okm.len(), 64);

        let okm2 = hkdf_sha384_expand(&prk, b"info", 64);
        assert_eq!(okm, okm2);
    }

    #[test]
    fn test_hkdf_sha384_different_info() {
        let prk = hkdf_sha384_extract(b"salt", b"ikm");
        let okm1 = hkdf_sha384_expand(&prk, b"info1", 32);
        let okm2 = hkdf_sha384_expand(&prk, b"info2", 32);
        assert_ne!(okm1, okm2);
    }

    #[test]
    fn test_hkdf_sha384_different_lengths() {
        let prk = hkdf_sha384_extract(b"salt", b"ikm");
        let okm32 = hkdf_sha384_expand(&prk, b"info", 32);
        let okm48 = hkdf_sha384_expand(&prk, b"info", 48);
        assert_eq!(&okm48[..32], &okm32[..]);
    }
}
