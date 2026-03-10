// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// WASM exports for TL-Sponge-385 — binds the Rust sponge to JavaScript
// via wasm-bindgen for use in server/crypto/sponge-wasm-bridge.ts.

use wasm_bindgen::prelude::*;
use crate::sponge;

#[wasm_bindgen]
pub fn sponge_hash(input: &[u8], output_len: usize) -> Vec<u8> {
    sponge::hash(input, output_len)
}

#[wasm_bindgen]
pub fn sponge_derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    sponge::derive_key(context, material, key_len)
}

#[wasm_bindgen]
pub fn sponge_keystream(domain: &[u8], trit_count: usize) -> Vec<i8> {
    let trits = sponge::bytes_to_trits_pub(domain);
    let mut s = sponge::Sponge385Pub::new();
    s.absorb(&trits);
    s.squeeze(trit_count)
}

#[wasm_bindgen]
pub fn sponge_duplex_encrypt(
    domain: &[u8],
    keystream_len: usize,
    switch_marker: &[u8],
    keystream2_len: usize,
    cipher1: &[u8],
    cipher2: &[u8],
    mac_trits: usize,
) -> Vec<i8> {
    let domain_trits = sponge::bytes_to_trits_pub(domain);
    let switch_trits = sponge::bytes_to_trits_pub(switch_marker);
    let cipher1_trits = sponge::bytes_to_trits_pub(cipher1);
    let cipher2_trits = sponge::bytes_to_trits_pub(cipher2);

    let mut s = sponge::Sponge385Pub::new();
    s.absorb(&domain_trits);
    let ks1 = s.squeeze(keystream_len);

    s.absorb(&switch_trits);
    let ks2 = s.squeeze(keystream2_len);

    s.absorb(&cipher1_trits);
    s.absorb(&cipher2_trits);
    let mac = s.squeeze(mac_trits);

    let header_bytes: [u8; 12] = {
        let mut h = [0u8; 12];
        h[0..4].copy_from_slice(&(ks1.len() as u32).to_le_bytes());
        h[4..8].copy_from_slice(&(ks2.len() as u32).to_le_bytes());
        h[8..12].copy_from_slice(&(mac.len() as u32).to_le_bytes());
        h
    };
    let mut result: Vec<i8> = Vec::with_capacity(12 + ks1.len() + ks2.len() + mac.len());
    for &b in &header_bytes {
        result.push(b as i8);
    }
    result.extend(ks1);
    result.extend(ks2);
    result.extend(mac);
    result
}
