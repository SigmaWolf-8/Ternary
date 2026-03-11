// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 N-API Native Addon
// Compiles the Rust sponge (with chi, precomputed CHI_MAP) as a Node.js
// native module for direct invocation from TypeScript. ~1,500× faster
// than the interpreted TypeScript fallback.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ternary_math::sponge::{Sponge385Pub, hash_hex, hash_hex_v1,
                            sponge_permutation, sponge_permutation_v1};

#[napi]
pub fn sponge_hash(input: Buffer) -> String {
    hash_hex(input.as_ref())
}

#[napi]
pub fn sponge_hash_v1(input: Buffer) -> String {
    hash_hex_v1(input.as_ref())
}

#[napi]
pub fn sponge_keystream(domain_input: Buffer, trit_count: u32) -> Buffer {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(domain_input.as_ref());
    let trits = sponge.squeeze(trit_count as usize);
    let bytes: Vec<u8> = trits.iter().map(|&t| (t + 1) as u8).collect();
    Buffer::from(bytes)
}

#[napi]
pub fn sponge_keystream_v1(domain_input: Buffer, trit_count: u32) -> Buffer {
    let mut sponge = Sponge385Pub::new_v1();
    sponge.absorb_bytes(domain_input.as_ref());
    let trits = sponge.squeeze(trit_count as usize);
    let bytes: Vec<u8> = trits.iter().map(|&t| (t + 1) as u8).collect();
    Buffer::from(bytes)
}

#[napi]
pub fn sponge_duplex_encrypt(
    domain_input: Buffer,
    keystream_trit_count: u32,
    ciphertext: Buffer,
    mac_trit_count: u32,
) -> Buffer {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(domain_input.as_ref());
    let keystream = sponge.squeeze(keystream_trit_count as usize);
    sponge.absorb_bytes(ciphertext.as_ref());
    let mac = sponge.squeeze(mac_trit_count as usize);

    let ks_len = keystream.len();
    let mut out = Vec::with_capacity(4 + ks_len + mac.len());
    out.extend_from_slice(&(ks_len as u32).to_le_bytes());
    out.extend(keystream.iter().map(|&t| (t + 1) as u8));
    out.extend(mac.iter().map(|&t| (t + 1) as u8));
    Buffer::from(out)
}

#[napi]
pub fn sponge_duplex_decrypt(
    domain_input: Buffer,
    keystream_trit_count: u32,
    ciphertext: Buffer,
    mac_trit_count: u32,
) -> Buffer {
    sponge_duplex_encrypt(domain_input, keystream_trit_count, ciphertext, mac_trit_count)
}

#[napi]
pub fn sponge_derive_key(context: Buffer, material: Buffer, key_len: u32) -> Buffer {
    let result = ternary_math::sponge::derive_key(context.as_ref(), material.as_ref(), key_len as usize);
    Buffer::from(result)
}

#[napi]
pub fn sponge_permute_v2(state_buf: Buffer) -> Buffer {
    let mut state = [0i8; 729];
    let src = state_buf.as_ref();
    for i in 0..729.min(src.len()) {
        state[i] = src[i] as i8;
    }
    sponge_permutation(&mut state);
    let out: Vec<u8> = state.iter().map(|&t| t as u8).collect();
    Buffer::from(out)
}

#[napi]
pub fn sponge_permute_v1(state_buf: Buffer) -> Buffer {
    let mut state = [0i8; 729];
    let src = state_buf.as_ref();
    for i in 0..729.min(src.len()) {
        state[i] = src[i] as i8;
    }
    sponge_permutation_v1(&mut state);
    let out: Vec<u8> = state.iter().map(|&t| t as u8).collect();
    Buffer::from(out)
}
