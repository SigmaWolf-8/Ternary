// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 N-API Native Addon
// Compiles the Rust sponge (with chi, precomputed CHI_MAP) as a Node.js
// native module for direct invocation from TypeScript.
//
// phase_duplex_encrypt / phase_duplex_decrypt execute the ENTIRE phase
// encryption duplex flow in a single FFI crossing — zero intermediate
// boundary overhead.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ternary_math::sponge::{Sponge385Pub, hash_hex, hash_hex_v1,
                            sponge_permutation, sponge_permutation_v1};

const MAX_TRIT_COUNT: u32 = 1_000_000;
const MAX_PLAIN_BYTES: usize = 1_048_576;
const TRITS_PER_BYTE: usize = 6;

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

#[inline(always)]
fn trit_sub(a: i8, b: i8) -> i8 {
    let s = a - b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

fn bytes_to_balanced_trits6(input: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(input.len() * TRITS_PER_BYTE);
    for &b in input {
        let mut v = b as i32;
        for _ in 0..TRITS_PER_BYTE {
            trits.push((v % 3) as i8 - 1);
            v /= 3;
        }
    }
    trits
}

fn balanced_trits6_to_bytes(trits: &[i8], byte_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; byte_len];
    let mut trit_idx = 0;
    for b in 0..byte_len {
        let mut val: i32 = 0;
        let mut mul: i32 = 1;
        for _ in 0..TRITS_PER_BYTE {
            if trit_idx < trits.len() {
                val += (trits[trit_idx] as i32 + 1) * mul;
                mul *= 3;
                trit_idx += 1;
            }
        }
        out[b] = val as u8;
    }
    out
}

fn cipher_trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let pack = 5;
    let byte_len = (trits.len() + pack - 1) / pack;
    let mut out = vec![0u8; byte_len];
    let mut trit_idx = 0;
    for b in 0..byte_len {
        let mut idx: i32 = 0;
        let mut mul: i32 = 1;
        for _ in 0..pack {
            if trit_idx < trits.len() {
                idx += (trits[trit_idx] as i32 + 1) * mul;
                mul *= 3;
                trit_idx += 1;
            }
        }
        out[b] = (idx & 0xFF) as u8;
    }
    out
}

fn cipher_bytes_to_trits(input: &[u8], trit_count: usize) -> Vec<i8> {
    let mut trits = vec![0i8; trit_count];
    let mut idx = 0;
    for &b in input {
        if idx >= trit_count { break; }
        let val = if b < 243 { b } else { 0 };
        let mut v = val as i32;
        for _ in 0..5 {
            if idx >= trit_count { break; }
            trits[idx] = (v % 3) as i8 - 1;
            v /= 3;
            idx += 1;
        }
    }
    trits
}

fn trits_to_hex(trits: &[i8]) -> String {
    let pack = 5;
    let byte_len = (trits.len() + pack - 1) / pack;
    let mut bytes = Vec::with_capacity(byte_len);
    let mut i = 0;
    while i < trits.len() {
        let mut val: u32 = 0;
        let mut mul: u32 = 1;
        for _ in 0..pack {
            if i < trits.len() {
                val += (trits[i] as u32 + 1) * mul;
                mul *= 3;
                i += 1;
            }
        }
        bytes.push(val as u8);
    }
    hex::encode(&bytes)
}

#[napi]
pub fn sponge_hash(input: Buffer) -> String {
    hash_hex(input.as_ref())
}

#[napi]
pub fn sponge_hash_v1(input: Buffer) -> String {
    hash_hex_v1(input.as_ref())
}

#[napi]
pub fn sponge_keystream(domain_input: Buffer, trit_count: u32) -> napi::Result<Buffer> {
    if trit_count > MAX_TRIT_COUNT {
        return Err(napi::Error::from_reason(
            format!("trit_count {} exceeds maximum {}", trit_count, MAX_TRIT_COUNT)
        ));
    }
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(domain_input.as_ref());
    let trits = sponge.squeeze(trit_count as usize);
    let bytes: Vec<u8> = trits.iter().map(|&t| (t + 1) as u8).collect();
    Ok(Buffer::from(bytes))
}

#[napi]
pub fn sponge_keystream_v1(domain_input: Buffer, trit_count: u32) -> napi::Result<Buffer> {
    if trit_count > MAX_TRIT_COUNT {
        return Err(napi::Error::from_reason(
            format!("trit_count {} exceeds maximum {}", trit_count, MAX_TRIT_COUNT)
        ));
    }
    let mut sponge = Sponge385Pub::new_v1();
    sponge.absorb_bytes(domain_input.as_ref());
    let trits = sponge.squeeze(trit_count as usize);
    let bytes: Vec<u8> = trits.iter().map(|&t| (t + 1) as u8).collect();
    Ok(Buffer::from(bytes))
}

#[napi]
pub fn sponge_derive_key(context: Buffer, material: Buffer, key_len: u32) -> napi::Result<Buffer> {
    if key_len > MAX_TRIT_COUNT {
        return Err(napi::Error::from_reason("key_len exceeds maximum".to_string()));
    }
    let result = ternary_math::sponge::derive_key(context.as_ref(), material.as_ref(), key_len as usize);
    Ok(Buffer::from(result))
}

#[napi]
pub fn sponge_permute_v2(state_buf: Buffer) -> napi::Result<Buffer> {
    let src = state_buf.as_ref();
    if src.len() != 729 {
        return Err(napi::Error::from_reason(
            format!("state must be exactly 729 bytes, got {}", src.len())
        ));
    }
    let mut state = [0i8; 729];
    for i in 0..729 {
        let v = src[i] as i8;
        if v < -1 || v > 1 {
            return Err(napi::Error::from_reason(
                format!("invalid trit value {} at index {}", v, i)
            ));
        }
        state[i] = v;
    }
    sponge_permutation(&mut state);
    let out: Vec<u8> = state.iter().map(|&t| t as u8).collect();
    Ok(Buffer::from(out))
}

#[napi]
pub fn sponge_permute_v1(state_buf: Buffer) -> napi::Result<Buffer> {
    let src = state_buf.as_ref();
    if src.len() != 729 {
        return Err(napi::Error::from_reason(
            format!("state must be exactly 729 bytes, got {}", src.len())
        ));
    }
    let mut state = [0i8; 729];
    for i in 0..729 {
        let v = src[i] as i8;
        if v < -1 || v > 1 {
            return Err(napi::Error::from_reason(
                format!("invalid trit value {} at index {}", v, i)
            ));
        }
        state[i] = v;
    }
    sponge_permutation_v1(&mut state);
    let out: Vec<u8> = state.iter().map(|&t| t as u8).collect();
    Ok(Buffer::from(out))
}

/// Full phase encryption duplex — single FFI crossing.
///
/// Replicates the exact TypeScript duplexEncrypt flow:
///   1. absorb domain_input
///   2. squeeze ks1 (primary_trit_count trits)
///   3. GF(3) add: cipher1_trits = primary_plain_trits + ks1
///   4. pack cipher1_trits → cipher1_bytes (5 trits/byte)
///   5. absorb switch_marker
///   6. squeeze ks2 (secondary_trit_count trits)
///   7. GF(3) add: cipher2_trits = secondary_plain_trits + ks2
///   8. pack cipher2_trits → cipher2_bytes (5 trits/byte)
///   9. build header1(8) + header2(8)
///  10. absorb header1 ‖ cipher1 ‖ header2 ‖ cipher2 → squeeze MAC (mac_trit_count)
///
/// Output layout:
///   [4B primary_cipher_len] [primary_header(8) + cipher1_bytes]
///   [4B secondary_cipher_len] [secondary_header(8) + cipher2_bytes]
///   [mac_hex as UTF-8 bytes]
#[napi]
pub fn phase_duplex_encrypt(
    domain_input: Buffer,
    primary_plain_bytes: Buffer,
    switch_marker: Buffer,
    secondary_plain_bytes: Buffer,
    mac_trit_count: u32,
    sponge_version: u32,
) -> napi::Result<Buffer> {
    let p1 = primary_plain_bytes.as_ref();
    let p2 = secondary_plain_bytes.as_ref();
    if p1.len() > MAX_PLAIN_BYTES || p2.len() > MAX_PLAIN_BYTES {
        return Err(napi::Error::from_reason("plaintext exceeds maximum size".to_string()));
    }
    if mac_trit_count > MAX_TRIT_COUNT {
        return Err(napi::Error::from_reason("mac_trit_count exceeds maximum".to_string()));
    }

    let primary_trits = bytes_to_balanced_trits6(p1);
    let secondary_trits = bytes_to_balanced_trits6(p2);

    let mut sponge = if sponge_version >= 2 {
        Sponge385Pub::new()
    } else {
        Sponge385Pub::new_v1()
    };

    sponge.absorb_bytes(domain_input.as_ref());

    let ks1 = sponge.squeeze(primary_trits.len());
    let mut cipher1_trits = vec![0i8; primary_trits.len()];
    for i in 0..primary_trits.len() {
        cipher1_trits[i] = trit_add(primary_trits[i], ks1[i]);
    }
    let cipher1_bytes = cipher_trits_to_bytes(&cipher1_trits);

    sponge.absorb_bytes(switch_marker.as_ref());

    let ks2 = sponge.squeeze(secondary_trits.len());
    let mut cipher2_trits = vec![0i8; secondary_trits.len()];
    for i in 0..secondary_trits.len() {
        cipher2_trits[i] = trit_add(secondary_trits[i], ks2[i]);
    }
    let cipher2_bytes = cipher_trits_to_bytes(&cipher2_trits);

    let mut header1 = [0u8; 8];
    header1[0..4].copy_from_slice(&(p1.len() as u32).to_be_bytes());
    header1[4..8].copy_from_slice(&(primary_trits.len() as u32).to_be_bytes());

    let mut header2 = [0u8; 8];
    header2[0..4].copy_from_slice(&(p2.len() as u32).to_be_bytes());
    header2[4..8].copy_from_slice(&(secondary_trits.len() as u32).to_be_bytes());

    sponge.absorb_bytes(&header1);
    sponge.absorb_bytes(&cipher1_bytes);
    sponge.absorb_bytes(&header2);
    sponge.absorb_bytes(&cipher2_bytes);
    let mac_trits = sponge.squeeze(mac_trit_count as usize);
    let mac_hex = trits_to_hex(&mac_trits);

    let full_cipher1 = [&header1[..], &cipher1_bytes[..]].concat();
    let full_cipher2 = [&header2[..], &cipher2_bytes[..]].concat();

    let mut out = Vec::with_capacity(
        4 + full_cipher1.len() + 4 + full_cipher2.len() + mac_hex.len()
    );
    out.extend_from_slice(&(full_cipher1.len() as u32).to_le_bytes());
    out.extend_from_slice(&full_cipher1);
    out.extend_from_slice(&(full_cipher2.len() as u32).to_le_bytes());
    out.extend_from_slice(&full_cipher2);
    out.extend_from_slice(mac_hex.as_bytes());

    Ok(Buffer::from(out))
}

/// Full phase decryption duplex — single FFI crossing.
///
/// Replicates the exact TypeScript duplexDecrypt flow:
///   1. Parse headers + ciphertext from base64-decoded buffers
///   2. absorb domain_input → squeeze ks1
///   3. absorb switch_marker → squeeze ks2
///   4. absorb headers + ciphertexts → squeeze MAC → verify
///   5. GF(3) sub: plain = cipher - keystream
///
/// Output layout:
///   [4B primary_plain_len] [primary_plain_bytes]
///   [4B secondary_plain_len] [secondary_plain_bytes]
///
/// Returns empty buffer if MAC verification fails.
#[napi]
pub fn phase_duplex_decrypt(
    domain_input: Buffer,
    primary_cipher_raw: Buffer,
    switch_marker: Buffer,
    secondary_cipher_raw: Buffer,
    expected_mac_hex: String,
    mac_trit_count: u32,
    sponge_version: u32,
) -> napi::Result<Buffer> {
    if mac_trit_count > MAX_TRIT_COUNT {
        return Err(napi::Error::from_reason("mac_trit_count exceeds maximum".to_string()));
    }

    let raw1 = primary_cipher_raw.as_ref();
    let raw2 = secondary_cipher_raw.as_ref();
    if raw1.len() < 8 || raw2.len() < 8 {
        return Err(napi::Error::from_reason("cipher data too short".to_string()));
    }

    let original_byte_len1 = u32::from_be_bytes([raw1[0], raw1[1], raw1[2], raw1[3]]) as usize;
    let trit_count1 = u32::from_be_bytes([raw1[4], raw1[5], raw1[6], raw1[7]]) as usize;
    let cipher1_bytes = &raw1[8..];

    let original_byte_len2 = u32::from_be_bytes([raw2[0], raw2[1], raw2[2], raw2[3]]) as usize;
    let trit_count2 = u32::from_be_bytes([raw2[4], raw2[5], raw2[6], raw2[7]]) as usize;
    let cipher2_bytes = &raw2[8..];

    if trit_count1 > MAX_TRIT_COUNT as usize || trit_count2 > MAX_TRIT_COUNT as usize {
        return Err(napi::Error::from_reason("trit count in header exceeds maximum".to_string()));
    }

    let cipher1_trits = cipher_bytes_to_trits(cipher1_bytes, trit_count1);
    let cipher2_trits = cipher_bytes_to_trits(cipher2_bytes, trit_count2);

    let mut sponge = if sponge_version >= 2 {
        Sponge385Pub::new()
    } else {
        Sponge385Pub::new_v1()
    };

    sponge.absorb_bytes(domain_input.as_ref());
    let ks1 = sponge.squeeze(trit_count1);

    sponge.absorb_bytes(switch_marker.as_ref());
    let ks2 = sponge.squeeze(trit_count2);

    let re_header1 = &raw1[0..8];
    let re_header2 = &raw2[0..8];
    sponge.absorb_bytes(re_header1);
    sponge.absorb_bytes(cipher1_bytes);
    sponge.absorb_bytes(re_header2);
    sponge.absorb_bytes(cipher2_bytes);
    let mac_trits = sponge.squeeze(mac_trit_count as usize);
    let computed_mac = trits_to_hex(&mac_trits);

    if computed_mac.len() != expected_mac_hex.len() {
        return Ok(Buffer::from(vec![]));
    }
    let a = computed_mac.as_bytes();
    let b = expected_mac_hex.as_bytes();
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    if diff != 0 {
        return Ok(Buffer::from(vec![]));
    }

    let mut plain1_trits = vec![0i8; trit_count1];
    for i in 0..trit_count1 {
        plain1_trits[i] = trit_sub(cipher1_trits[i], ks1[i]);
    }
    let plain1 = balanced_trits6_to_bytes(&plain1_trits, original_byte_len1);

    let mut plain2_trits = vec![0i8; trit_count2];
    for i in 0..trit_count2 {
        plain2_trits[i] = trit_sub(cipher2_trits[i], ks2[i]);
    }
    let plain2 = balanced_trits6_to_bytes(&plain2_trits, original_byte_len2);

    let mut out = Vec::with_capacity(8 + plain1.len() + plain2.len());
    out.extend_from_slice(&(plain1.len() as u32).to_le_bytes());
    out.extend_from_slice(&plain1);
    out.extend_from_slice(&(plain2.len() as u32).to_le_bytes());
    out.extend_from_slice(&plain2);

    Ok(Buffer::from(out))
}
