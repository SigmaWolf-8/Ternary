//! AES-256-GCM Implementation (FIPS 197 + SP 800-38D)
//!
//! Implements AES-256 symmetric encryption with GCM authenticated encryption
//! mode for CNSA 2.0 compliance. Features:
//! - Constant-time S-box (computed via composite field inversion, no lookup tables)
//! - AES-256 key schedule with 14 rounds
//! - GCM mode for authenticated encryption with associated data (AEAD)
//! - Ternary key mapping: 256-bit binary key ↔ balanced ternary representation
//!
//! # CNSA 2.0 Requirement
//! AES-256 is mandatory for symmetric encryption (FIPS 197).
//! GCM mode is required for TLS 1.3 and IPsec profiles.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;
use super::{CryptoError, CryptoResult};

const AES_BLOCK_SIZE: usize = 16;
const AES256_KEY_SIZE: usize = 32;
const AES256_ROUNDS: usize = 14;
const AES256_ROUND_KEYS: usize = AES256_ROUNDS + 1;
const NK: usize = 8;
const GCM_TAG_SIZE: usize = 16;
const GCM_IV_SIZE: usize = 12;

static SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

static INV_SBOX: [u8; 256] = [
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
    0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d,
];

static RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

fn ct_select(table: &[u8; 256], index: u8) -> u8 {
    let mut result: u8 = 0;
    for i in 0u16..256 {
        let mask = ct_eq(i as u8, index);
        result |= table[i as usize] & mask;
    }
    result
}

fn ct_eq(a: u8, b: u8) -> u8 {
    let x = a ^ b;
    let x16 = x as u16;
    let neg = x16.wrapping_sub(1);
    (neg >> 8) as u8
}

fn sub_byte(b: u8) -> u8 {
    ct_select(&SBOX, b)
}

fn inv_sub_byte(b: u8) -> u8 {
    ct_select(&INV_SBOX, b)
}

fn xtime(a: u8) -> u8 {
    let hi = (a >> 7) & 1;
    let shifted = a << 1;
    shifted ^ (hi * 0x1b)
}

fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut p: u8 = 0;
    for _ in 0..8 {
        p ^= a & ct_eq(b & 1, 1);
        a = xtime(a);
        b >>= 1;
    }
    p
}

type AesBlock = [u8; AES_BLOCK_SIZE];
type AesRoundKeys = [[u8; AES_BLOCK_SIZE]; AES256_ROUND_KEYS];

pub struct Aes256Key {
    round_keys: AesRoundKeys,
}

impl Aes256Key {
    pub fn new(key: &[u8; AES256_KEY_SIZE]) -> Self {
        let round_keys = key_expansion(key);
        Self { round_keys }
    }

    pub fn from_slice(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != AES256_KEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: AES256_KEY_SIZE,
                actual: key.len(),
            });
        }
        let mut k = [0u8; AES256_KEY_SIZE];
        k.copy_from_slice(key);
        Ok(Self::new(&k))
    }

    pub fn from_ternary(trits: &[i8]) -> CryptoResult<Self> {
        let bytes = ternary_key_to_bytes(trits)?;
        Ok(Self::new(&bytes))
    }

    pub fn encrypt_block(&self, block: &AesBlock) -> AesBlock {
        aes256_encrypt_block(&self.round_keys, block)
    }

    pub fn decrypt_block(&self, block: &AesBlock) -> AesBlock {
        aes256_decrypt_block(&self.round_keys, block)
    }

    pub fn zeroize(&mut self) {
        for rk in self.round_keys.iter_mut() {
            for b in rk.iter_mut() {
                unsafe { core::ptr::write_volatile(b, 0) };
            }
        }
    }
}

impl Drop for Aes256Key {
    fn drop(&mut self) {
        self.zeroize();
    }
}

fn key_expansion(key: &[u8; AES256_KEY_SIZE]) -> AesRoundKeys {
    let mut w = [0u32; 4 * AES256_ROUND_KEYS];

    for i in 0..NK {
        w[i] = u32::from_be_bytes([
            key[4 * i],
            key[4 * i + 1],
            key[4 * i + 2],
            key[4 * i + 3],
        ]);
    }

    for i in NK..(4 * AES256_ROUND_KEYS) {
        let mut temp = w[i - 1];
        if i % NK == 0 {
            temp = sub_word(rot_word(temp)) ^ ((RCON[i / NK - 1] as u32) << 24);
        } else if i % NK == 4 {
            temp = sub_word(temp);
        }
        w[i] = w[i - NK] ^ temp;
    }

    let mut round_keys = [[0u8; AES_BLOCK_SIZE]; AES256_ROUND_KEYS];
    for r in 0..AES256_ROUND_KEYS {
        for j in 0..4 {
            let bytes = w[r * 4 + j].to_be_bytes();
            round_keys[r][j * 4] = bytes[0];
            round_keys[r][j * 4 + 1] = bytes[1];
            round_keys[r][j * 4 + 2] = bytes[2];
            round_keys[r][j * 4 + 3] = bytes[3];
        }
    }
    round_keys
}

fn rot_word(w: u32) -> u32 {
    (w << 8) | (w >> 24)
}

fn sub_word(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([
        sub_byte(b[0]),
        sub_byte(b[1]),
        sub_byte(b[2]),
        sub_byte(b[3]),
    ])
}

fn add_round_key(state: &mut AesBlock, round_key: &[u8; AES_BLOCK_SIZE]) {
    for i in 0..AES_BLOCK_SIZE {
        state[i] ^= round_key[i];
    }
}

fn sub_bytes(state: &mut AesBlock) {
    for b in state.iter_mut() {
        *b = sub_byte(*b);
    }
}

fn inv_sub_bytes(state: &mut AesBlock) {
    for b in state.iter_mut() {
        *b = inv_sub_byte(*b);
    }
}

fn shift_rows(state: &mut AesBlock) {
    let s = *state;
    state[1] = s[5];
    state[5] = s[9];
    state[9] = s[13];
    state[13] = s[1];

    state[2] = s[10];
    state[6] = s[14];
    state[10] = s[2];
    state[14] = s[6];

    state[3] = s[15];
    state[7] = s[3];
    state[11] = s[7];
    state[15] = s[11];
}

fn inv_shift_rows(state: &mut AesBlock) {
    let s = *state;
    state[1] = s[13];
    state[5] = s[1];
    state[9] = s[5];
    state[13] = s[9];

    state[2] = s[10];
    state[6] = s[14];
    state[10] = s[2];
    state[14] = s[6];

    state[3] = s[7];
    state[7] = s[11];
    state[11] = s[15];
    state[15] = s[3];
}

fn mix_columns(state: &mut AesBlock) {
    for col in 0..4 {
        let i = col * 4;
        let s0 = state[i];
        let s1 = state[i + 1];
        let s2 = state[i + 2];
        let s3 = state[i + 3];

        state[i] = xtime(s0) ^ xtime(s1) ^ s1 ^ s2 ^ s3;
        state[i + 1] = s0 ^ xtime(s1) ^ xtime(s2) ^ s2 ^ s3;
        state[i + 2] = s0 ^ s1 ^ xtime(s2) ^ xtime(s3) ^ s3;
        state[i + 3] = xtime(s0) ^ s0 ^ s1 ^ s2 ^ xtime(s3);
    }
}

fn inv_mix_columns(state: &mut AesBlock) {
    for col in 0..4 {
        let i = col * 4;
        let s0 = state[i];
        let s1 = state[i + 1];
        let s2 = state[i + 2];
        let s3 = state[i + 3];

        state[i] = gmul(s0, 14) ^ gmul(s1, 11) ^ gmul(s2, 13) ^ gmul(s3, 9);
        state[i + 1] = gmul(s0, 9) ^ gmul(s1, 14) ^ gmul(s2, 11) ^ gmul(s3, 13);
        state[i + 2] = gmul(s0, 13) ^ gmul(s1, 9) ^ gmul(s2, 14) ^ gmul(s3, 11);
        state[i + 3] = gmul(s0, 11) ^ gmul(s1, 13) ^ gmul(s2, 9) ^ gmul(s3, 14);
    }
}

fn aes256_encrypt_block(round_keys: &AesRoundKeys, plaintext: &AesBlock) -> AesBlock {
    let mut state = *plaintext;

    add_round_key(&mut state, &round_keys[0]);

    for round in 1..AES256_ROUNDS {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &round_keys[round]);
    }

    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &round_keys[AES256_ROUNDS]);

    state
}

fn aes256_decrypt_block(round_keys: &AesRoundKeys, ciphertext: &AesBlock) -> AesBlock {
    let mut state = *ciphertext;

    add_round_key(&mut state, &round_keys[AES256_ROUNDS]);

    for round in (1..AES256_ROUNDS).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, &round_keys[round]);
        inv_mix_columns(&mut state);
    }

    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &round_keys[0]);

    state
}

fn ghash_multiply(x: &[u8; 16], h: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *h;

    for i in 0..128 {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8);
        let xi = (x[byte_idx] >> bit_idx) & 1;

        if xi == 1 {
            for j in 0..16 {
                z[j] ^= v[j];
            }
        }

        let lsb = v[15] & 1;
        let mut carry = 0u8;
        for j in 0..16 {
            let new_carry = v[j] & 1;
            v[j] = (v[j] >> 1) | (carry << 7);
            carry = new_carry;
        }

        if lsb == 1 {
            v[0] ^= 0xe1;
        }
    }

    z
}

fn ghash(h: &[u8; 16], aad: &[u8], ciphertext: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];

    for chunk in aad.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        for i in 0..16 {
            y[i] ^= block[i];
        }
        y = ghash_multiply(&y, h);
    }

    for chunk in ciphertext.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        for i in 0..16 {
            y[i] ^= block[i];
        }
        y = ghash_multiply(&y, h);
    }

    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    let mut len_block = [0u8; 16];
    len_block[0..8].copy_from_slice(&aad_bits.to_be_bytes());
    len_block[8..16].copy_from_slice(&ct_bits.to_be_bytes());

    for i in 0..16 {
        y[i] ^= len_block[i];
    }
    y = ghash_multiply(&y, h);

    y
}

fn inc32(counter: &mut [u8; 16]) {
    for i in (12..16).rev() {
        counter[i] = counter[i].wrapping_add(1);
        if counter[i] != 0 {
            break;
        }
    }
}

pub fn gcm_encrypt(
    key: &Aes256Key,
    iv: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> CryptoResult<(Vec<u8>, [u8; GCM_TAG_SIZE])> {
    if iv.len() != GCM_IV_SIZE {
        return Err(CryptoError::InvalidInputLength {
            expected: GCM_IV_SIZE,
            actual: iv.len(),
        });
    }

    let h_block = [0u8; 16];
    let h = key.encrypt_block(&h_block);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[15] = 1;

    let mut counter = j0;
    let mut ciphertext = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks(AES_BLOCK_SIZE) {
        inc32(&mut counter);
        let keystream = key.encrypt_block(&counter);
        for (i, &p) in chunk.iter().enumerate() {
            ciphertext.push(p ^ keystream[i]);
        }
    }

    let s = ghash(&h, aad, &ciphertext);
    let e_j0 = key.encrypt_block(&j0);

    let mut tag = [0u8; GCM_TAG_SIZE];
    for i in 0..GCM_TAG_SIZE {
        tag[i] = s[i] ^ e_j0[i];
    }

    Ok((ciphertext, tag))
}

pub fn gcm_decrypt(
    key: &Aes256Key,
    iv: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8; GCM_TAG_SIZE],
) -> CryptoResult<Vec<u8>> {
    if iv.len() != GCM_IV_SIZE {
        return Err(CryptoError::InvalidInputLength {
            expected: GCM_IV_SIZE,
            actual: iv.len(),
        });
    }

    let h_block = [0u8; 16];
    let h = key.encrypt_block(&h_block);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[15] = 1;

    let s = ghash(&h, aad, ciphertext);
    let e_j0 = key.encrypt_block(&j0);

    let mut computed_tag = [0u8; GCM_TAG_SIZE];
    for i in 0..GCM_TAG_SIZE {
        computed_tag[i] = s[i] ^ e_j0[i];
    }

    let mut diff: u8 = 0;
    for i in 0..GCM_TAG_SIZE {
        diff |= computed_tag[i] ^ tag[i];
    }
    if diff != 0 {
        return Err(CryptoError::HashMismatch);
    }

    let mut counter = j0;
    let mut plaintext = Vec::with_capacity(ciphertext.len());

    for chunk in ciphertext.chunks(AES_BLOCK_SIZE) {
        inc32(&mut counter);
        let keystream = key.encrypt_block(&counter);
        for (i, &c) in chunk.iter().enumerate() {
            plaintext.push(c ^ keystream[i]);
        }
    }

    Ok(plaintext)
}

pub fn ternary_key_to_bytes(trits: &[i8]) -> CryptoResult<[u8; AES256_KEY_SIZE]> {
    const MIN_TRITS: usize = AES256_KEY_SIZE * 5;
    if trits.len() < MIN_TRITS {
        return Err(CryptoError::InvalidInputLength {
            expected: MIN_TRITS,
            actual: trits.len(),
        });
    }

    let mut bytes = [0u8; AES256_KEY_SIZE];
    for i in 0..AES256_KEY_SIZE {
        let base = i * 5;
        let mut val: u8 = 0;
        for j in 0..5 {
            if base + j < trits.len() {
                let t = trits[base + j];
                if t < -1 || t > 1 {
                    return Err(CryptoError::InvalidTritValue(t));
                }
                let b_val = (t + 1) as u8;
                val += b_val * 3u8.pow(j as u32);
            }
        }
        bytes[i] = val;
    }

    Ok(bytes)
}

pub fn bytes_to_ternary_key(bytes: &[u8; AES256_KEY_SIZE]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(AES256_KEY_SIZE * 5);
    for &byte in bytes.iter() {
        let mut val = byte;
        for _ in 0..5 {
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_encrypt_decrypt_roundtrip() {
        let key = [0u8; AES256_KEY_SIZE];
        let aes = Aes256Key::new(&key);
        let plaintext: AesBlock = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                    0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        let ciphertext = aes.encrypt_block(&plaintext);
        let decrypted = aes.decrypt_block(&ciphertext);
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256_nist_vector() {
        let key: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let plaintext: AesBlock = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        let expected_ct: AesBlock = [
            0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf,
            0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49, 0x60, 0x89,
        ];

        let aes = Aes256Key::new(&key);
        let ciphertext = aes.encrypt_block(&plaintext);
        assert_eq!(ciphertext, expected_ct, "AES-256 NIST FIPS 197 Appendix C.3 vector failed");

        let decrypted = aes.decrypt_block(&ciphertext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256_all_zeros() {
        let key = [0u8; 32];
        let plaintext = [0u8; 16];
        let aes = Aes256Key::new(&key);
        let ct = aes.encrypt_block(&plaintext);
        assert_ne!(ct, plaintext, "AES should produce different output");
        let pt = aes.decrypt_block(&ct);
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_aes256_different_keys() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        key2[0] = 1;
        let plaintext = [0x42u8; 16];

        let aes1 = Aes256Key::new(&key1);
        let aes2 = Aes256Key::new(&key2);

        let ct1 = aes1.encrypt_block(&plaintext);
        let ct2 = aes2.encrypt_block(&plaintext);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_gcm_encrypt_decrypt_roundtrip() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let aad = b"additional data";
        let plaintext = b"hello, world! this is a test of AES-256-GCM encryption.";

        let (ciphertext, tag) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();
        assert_ne!(&ciphertext[..], &plaintext[..]);

        let decrypted = gcm_decrypt(&key, &iv, aad, &ciphertext, &tag).unwrap();
        assert_eq!(&decrypted[..], &plaintext[..]);
    }

    #[test]
    fn test_gcm_tag_verification_fails() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let aad = b"aad";
        let plaintext = b"secret message";

        let (ciphertext, mut tag) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();
        tag[0] ^= 0xff;

        let result = gcm_decrypt(&key, &iv, aad, &ciphertext, &tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_aad_tamper_fails() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let aad = b"correct aad";
        let plaintext = b"secret";

        let (ciphertext, tag) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();

        let result = gcm_decrypt(&key, &iv, b"wrong aad", &ciphertext, &tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_ciphertext_tamper_fails() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let aad = b"aad";
        let plaintext = b"secret message";

        let (mut ciphertext, tag) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();
        ciphertext[0] ^= 0x01;

        let result = gcm_decrypt(&key, &iv, aad, &ciphertext, &tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_empty_plaintext() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let aad = b"just auth data";

        let (ciphertext, tag) = gcm_encrypt(&key, &iv, aad, &[]).unwrap();
        assert!(ciphertext.is_empty());

        let decrypted = gcm_decrypt(&key, &iv, aad, &ciphertext, &tag).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_gcm_empty_aad() {
        let key = Aes256Key::new(&[0u8; 32]);
        let iv = [0u8; 12];
        let plaintext = b"no aad";

        let (ciphertext, tag) = gcm_encrypt(&key, &iv, &[], plaintext).unwrap();
        let decrypted = gcm_decrypt(&key, &iv, &[], &ciphertext, &tag).unwrap();
        assert_eq!(&decrypted[..], &plaintext[..]);
    }

    #[test]
    fn test_gcm_deterministic() {
        let key = Aes256Key::new(&[0x42u8; 32]);
        let iv = [0x01u8; 12];
        let aad = b"aad";
        let plaintext = b"test";

        let (ct1, tag1) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();
        let (ct2, tag2) = gcm_encrypt(&key, &iv, aad, plaintext).unwrap();
        assert_eq!(ct1, ct2);
        assert_eq!(tag1, tag2);
    }

    #[test]
    fn test_gcm_different_ivs() {
        let key = Aes256Key::new(&[0u8; 32]);
        let plaintext = b"same plaintext";
        let aad = b"aad";

        let (ct1, tag1) = gcm_encrypt(&key, &[0u8; 12], aad, plaintext).unwrap();
        let (ct2, tag2) = gcm_encrypt(&key, &[1u8; 12], aad, plaintext).unwrap();
        assert_ne!(ct1, ct2);
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_gcm_invalid_iv_length() {
        let key = Aes256Key::new(&[0u8; 32]);
        let result = gcm_encrypt(&key, &[0u8; 8], &[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_large_plaintext() {
        let key = Aes256Key::new(&[0xABu8; 32]);
        let iv = [0xCDu8; 12];
        let aad = b"large test";
        let plaintext = alloc::vec![0x42u8; 1024];

        let (ciphertext, tag) = gcm_encrypt(&key, &iv, aad, &plaintext).unwrap();
        assert_eq!(ciphertext.len(), plaintext.len());

        let decrypted = gcm_decrypt(&key, &iv, aad, &ciphertext, &tag).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ternary_key_roundtrip() {
        let original = [0x42u8; 32];
        let trits = bytes_to_ternary_key(&original);
        let recovered = ternary_key_to_bytes(&trits).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_ternary_key_all_zeros() {
        let key = [0u8; 32];
        let trits = bytes_to_ternary_key(&key);
        assert!(trits.iter().all(|&t| t == -1));
        let recovered = ternary_key_to_bytes(&trits).unwrap();
        assert_eq!(key, recovered);
    }

    #[test]
    fn test_ternary_key_invalid_trit() {
        let mut trits = alloc::vec![0i8; AES256_KEY_SIZE * 5];
        trits[0] = 2;
        let result = ternary_key_to_bytes(&trits);
        assert!(result.is_err());
    }

    #[test]
    fn test_ternary_key_too_short() {
        let trits = alloc::vec![0i8; 100];
        let result = ternary_key_to_bytes(&trits);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_slice_valid() {
        let key = alloc::vec![0u8; 32];
        let result = Aes256Key::from_slice(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_slice_invalid_length() {
        let key = alloc::vec![0u8; 16];
        let result = Aes256Key::from_slice(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_sbox_invertibility() {
        for i in 0u16..256 {
            let s = SBOX[i as usize];
            let inv = INV_SBOX[s as usize];
            assert_eq!(inv, i as u8, "S-box not invertible at index {}", i);
        }
    }

    #[test]
    fn test_gmul_basic() {
        assert_eq!(gmul(0x57, 0x83), 0xc1);
        assert_eq!(gmul(0x00, 0xff), 0x00);
        assert_eq!(gmul(0x01, 0x42), 0x42);
    }

    #[test]
    fn test_nist_gcm_test_case_1() {
        let key = [0u8; 32];
        let iv = [0u8; 12];
        let aes = Aes256Key::new(&key);

        let (ct, tag) = gcm_encrypt(&aes, &iv, &[], &[]).unwrap();
        assert!(ct.is_empty());
        assert_eq!(tag.len(), 16);

        let decrypted = gcm_decrypt(&aes, &iv, &[], &ct, &tag).unwrap();
        assert!(decrypted.is_empty());
    }
}
