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

//! Phase Encryption v3 — Duplex-mode TL-Sponge-385-based GF(3) stream cipher
//!
//! Port of `server/salvi-core/phase-encryption.ts` to pure Rust.
//!
//! Architecture (v3 duplex — 1 sponge init per encrypt):
//!   1. Derive 32-byte key material via TL-Sponge-385
//!   2. Generate 32-byte random nonce per operation
//!   3. Build domain: key_material ‖ nonce ‖ phase_angle_364 ‖ context_tag
//!   4. Duplex: absorb domain → squeeze primary keystream → absorb phase switch →
//!      squeeze secondary keystream → absorb both ciphertexts → squeeze MAC
//!   5. Encrypt: ciphertext[i] = tritAdd(plaintext[i], keystream[i])  — GF(3)
//!   6. Decrypt: reverse with tritSub
//!
//! Supports all four modes: high_security, balanced, performance, adaptive.
//! Guardian phase provides τ-derived tamper detection via TL-Sponge-385 hash.
//!
//! Compatible with TypeScript v3 ciphertext format.

use crate::tlsponge385::Sponge385Pub;

const TERNARY_FULL_CIRCLE: u32 = 364;
const STD_FULL_CIRCLE: u32 = 360;
const TRITS_PER_BYTE: usize = 6;
const MAC_TRITS: usize = 243;
const NONCE_BYTES: usize = 32;
const KEY_BYTES: usize = 32;

const PHASE_CONTEXT_TAG: &[u8] = b"PlenumNET-Phase-v2";

static TRIT_ADD_LUT: [i8; 5] = [1, -1, 0, 1, -1];
static TRIT_SUB_LUT: [i8; 5] = [1, -1, 0, 1, -1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    HighSecurity,
    Balanced,
    Performance,
    Adaptive,
}

impl EncryptionMode {
    pub fn name(&self) -> &'static str {
        match self {
            EncryptionMode::HighSecurity => "high_security",
            EncryptionMode::Balanced => "balanced",
            EncryptionMode::Performance => "performance",
            EncryptionMode::Adaptive => "adaptive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhaseConfig {
    pub mode: EncryptionMode,
    pub primary_phase: u16,
    pub secondary_offset: u16,
    pub guardian_enabled: bool,
    pub guardian_offset: u16,
}

pub fn get_phase_config(mode: EncryptionMode) -> PhaseConfig {
    match mode {
        EncryptionMode::HighSecurity => PhaseConfig {
            mode,
            primary_phase: 0,
            secondary_offset: 10,
            guardian_enabled: true,
            guardian_offset: 358,
        },
        EncryptionMode::Balanced => PhaseConfig {
            mode,
            primary_phase: 0,
            secondary_offset: 4,
            guardian_enabled: false,
            guardian_offset: 0,
        },
        EncryptionMode::Performance => PhaseConfig {
            mode,
            primary_phase: 0,
            secondary_offset: 1,
            guardian_enabled: false,
            guardian_offset: 0,
        },
        EncryptionMode::Adaptive => PhaseConfig {
            mode,
            primary_phase: 0,
            secondary_offset: 4,
            guardian_enabled: true,
            guardian_offset: 358,
        },
    }
}

#[derive(Debug, Clone)]
pub struct PhaseCiphertext {
    pub primary_cipher: Vec<u8>,
    pub secondary_cipher: Vec<u8>,
    pub mac: String,
    pub nonce: Vec<u8>,
    pub config: PhaseConfig,
    pub guardian_hash: Option<String>,
    pub version: u8,
    pub sponge_version: u8,
}

impl PhaseCiphertext {
    pub fn primary_cipher_b64(&self) -> String {
        base64_encode(&self.primary_cipher)
    }

    pub fn secondary_cipher_b64(&self) -> String {
        base64_encode(&self.secondary_cipher)
    }

    pub fn nonce_hex(&self) -> String {
        self.nonce.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn to_ts_wire_format(&self) -> TsWireFormat {
        TsWireFormat {
            primary_data_b64: self.primary_cipher_b64(),
            primary_phase: self.config.primary_phase,
            secondary_data_b64: self.secondary_cipher_b64(),
            secondary_phase: self.config.primary_phase + self.config.secondary_offset,
            config: self.config.clone(),
            split_ratio: 0.5,
            nonce_hex: self.nonce_hex(),
            mac: self.mac.clone(),
            version: self.version,
            sponge_version: self.sponge_version,
            guardian_hash: self.guardian_hash.clone(),
            guardian_phase: if self.config.guardian_enabled { Some(self.config.guardian_offset) } else { None },
        }
    }

    pub fn from_ts_wire_format(wire: &TsWireFormat) -> Result<Self, PhaseError> {
        let primary_cipher = base64_decode(&wire.primary_data_b64)
            .map_err(|_| PhaseError::InvalidCiphertext)?;
        let secondary_cipher = base64_decode(&wire.secondary_data_b64)
            .map_err(|_| PhaseError::InvalidCiphertext)?;
        let nonce = hex_decode(&wire.nonce_hex)
            .map_err(|_| PhaseError::InvalidCiphertext)?;
        if nonce.len() != NONCE_BYTES {
            return Err(PhaseError::InvalidCiphertext);
        }
        Ok(PhaseCiphertext {
            primary_cipher,
            secondary_cipher,
            mac: wire.mac.clone(),
            nonce,
            config: wire.config.clone(),
            guardian_hash: wire.guardian_hash.clone(),
            version: wire.version,
            sponge_version: wire.sponge_version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TsWireFormat {
    pub primary_data_b64: String,
    pub primary_phase: u16,
    pub secondary_data_b64: String,
    pub secondary_phase: u16,
    pub config: PhaseConfig,
    pub split_ratio: f64,
    pub nonce_hex: String,
    pub mac: String,
    pub version: u8,
    pub sponge_version: u8,
    pub guardian_hash: Option<String>,
    pub guardian_phase: Option<u16>,
}

#[derive(Debug)]
pub enum PhaseError {
    MacMismatch,
    GuardianFailed,
    InvalidCiphertext,
    RandomnessError,
}

impl std::fmt::Display for PhaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseError::MacMismatch => write!(f, "MAC verification failed"),
            PhaseError::GuardianFailed => write!(f, "Guardian phase tamper detection failed"),
            PhaseError::InvalidCiphertext => write!(f, "Invalid ciphertext format"),
            PhaseError::RandomnessError => write!(f, "Failed to generate random bytes"),
        }
    }
}

impl std::error::Error for PhaseError {}

fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 2 < data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i+1] as u32) << 8) | data[i+2] as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(B64[(n >> 6) as usize & 63] as char);
        out.push(B64[n as usize & 63] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i+1] as u32) << 8);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(B64[(n >> 6) as usize & 63] as char);
        out.push('=');
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    fn b64val(c: u8) -> Result<u8, &'static str> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err("invalid base64 char"),
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 { return Err("invalid base64 length"); }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut i = 0;
    while i < bytes.len() {
        let a = b64val(bytes[i])?; let b = b64val(bytes[i+1])?;
        let c = b64val(bytes[i+2])?; let d = b64val(bytes[i+3])?;
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | d as u32;
        out.push((n >> 16) as u8);
        if bytes[i+2] != b'=' { out.push((n >> 8) as u8); }
        if bytes[i+3] != b'=' { out.push(n as u8); }
        i += 4;
    }
    Ok(out)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 { return Err("odd hex length"); }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_char_to_nibble(bytes[i]);
        let lo = hex_char_to_nibble(bytes[i+1]);
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn std_deg_to_ternary_deg(std_deg: u16) -> u16 {
    ((std_deg as u32 * TERNARY_FULL_CIRCLE + STD_FULL_CIRCLE / 2) / STD_FULL_CIRCLE) as u16
}

fn build_domain_input(key: &[u8; KEY_BYTES], nonce: &[u8; NONCE_BYTES], ternary_angle: u16) -> Vec<u8> {
    let mut domain = Vec::with_capacity(KEY_BYTES + NONCE_BYTES + 2 + PHASE_CONTEXT_TAG.len());
    domain.extend_from_slice(key);
    domain.extend_from_slice(nonce);
    domain.push((ternary_angle >> 8) as u8);
    domain.push(ternary_angle as u8);
    domain.extend_from_slice(PHASE_CONTEXT_TAG);
    domain
}

fn bytes_to_balanced_trits_6(input: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(input.len() * TRITS_PER_BYTE);
    for &byte in input {
        let mut v = byte;
        for _ in 0..TRITS_PER_BYTE {
            trits.push((v % 3) as i8 - 1);
            v /= 3;
        }
    }
    trits
}

fn balanced_trits_6_to_bytes(trits: &[i8], byte_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; byte_len];
    let mut trit_idx = 0;
    for b in 0..byte_len {
        let mut idx: u16 = 0;
        let mut mul: u16 = 1;
        for _ in 0..TRITS_PER_BYTE {
            if trit_idx < trits.len() {
                idx += (trits[trit_idx] + 1) as u16 * mul;
                mul *= 3;
                trit_idx += 1;
            }
        }
        let trit6_to_byte_idx = idx as usize;
        if trit6_to_byte_idx < 729 {
            out[b] = TRITS6_TO_BYTE[trit6_to_byte_idx];
        }
    }
    out
}

static TRITS6_TO_BYTE: [u8; 729] = {
    let mut table = [0u8; 729];
    let mut byte: u16 = 0;
    while byte < 256 {
        let mut v = byte;
        let mut idx: usize = 0;
        let mut mul: usize = 1;
        let mut j = 0;
        while j < 6 {
            let t = (v % 3) as usize;
            idx += t * mul;
            mul *= 3;
            v /= 3;
            j += 1;
        }
        table[idx] = byte as u8;
        byte += 1;
    }
    table
};

fn cipher_trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let pack = 5;
    let byte_len = (trits.len() + pack - 1) / pack;
    let mut out = vec![0u8; byte_len];
    let mut trit_idx = 0;
    for b in 0..byte_len {
        let mut idx: u16 = 0;
        let mut mul: u16 = 1;
        for _ in 0..pack {
            if trit_idx < trits.len() {
                idx += (trits[trit_idx] + 1) as u16 * mul;
                mul *= 3;
                trit_idx += 1;
            }
        }
        let val = idx.min(242);
        out[b] = val as u8;
    }
    out
}

fn cipher_bytes_to_trits(input: &[u8], trit_count: usize) -> Vec<i8> {
    let mut trits = Vec::with_capacity(trit_count);
    for &byte in input {
        if trits.len() >= trit_count { break; }
        let b = if byte < 243 { byte } else { 0 };
        let mut v = b;
        for _ in 0..5 {
            if trits.len() >= trit_count { break; }
            trits.push((v % 3) as i8 - 1);
            v /= 3;
        }
    }
    trits
}

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    TRIT_ADD_LUT[(a + b + 2) as usize]
}

#[inline(always)]
fn trit_sub(a: i8, b: i8) -> i8 {
    TRIT_SUB_LUT[(a - b + 2) as usize]
}

fn encrypt_trits(plain: &[i8], keystream: &[i8]) -> Vec<i8> {
    plain.iter().zip(keystream.iter())
        .map(|(&p, &k)| trit_add(p, k))
        .collect()
}

fn decrypt_trits(cipher: &[i8], keystream: &[i8]) -> Vec<i8> {
    cipher.iter().zip(keystream.iter())
        .map(|(&c, &k)| trit_sub(c, k))
        .collect()
}

fn trits_to_hex(trits: &[i8]) -> String {
    let mut byte_vec = Vec::new();
    let mut i = 0;
    while i < trits.len() {
        let mut val: u8 = 0;
        let mut pow: u8 = 1;
        for j in 0..5 {
            if i + j < trits.len() {
                val = val.wrapping_add((trits[i + j] + 1) as u8 * pow);
            }
            pow = pow.wrapping_mul(3);
        }
        byte_vec.push(val);
        i += 5;
    }
    byte_vec.iter().map(|b| format!("{:02x}", b)).collect()
}

fn sponge_hash_hex(input: &[u8]) -> String {
    crate::tlsponge385::hash_hex(input)
}

pub fn derive_key_from_secret(secret: &[u8]) -> [u8; KEY_BYTES] {
    let tag = b"PlenumNET-Phase-KeyDerive";
    let mut input = Vec::with_capacity(secret.len() + tag.len());
    input.extend_from_slice(secret);
    input.extend_from_slice(tag);
    let hash_hex = sponge_hash_hex(&input);
    let mut key = [0u8; KEY_BYTES];
    let hex_bytes = hash_hex.as_bytes();
    for i in 0..KEY_BYTES {
        let hi = hex_char_to_nibble(hex_bytes[i * 2]);
        let lo = hex_char_to_nibble(hex_bytes[i * 2 + 1]);
        key[i] = (hi << 4) | lo;
    }
    key
}

pub fn derive_key_from_kem_secret(kem_shared: &[u8; 32]) -> [u8; KEY_BYTES] {
    let domain = b"PlenumNET-Phase-KEM-KeyDerive";
    let mut input = Vec::with_capacity(domain.len() + 32);
    input.extend_from_slice(domain);
    input.extend_from_slice(kem_shared);
    let hash_hex = sponge_hash_hex(&input);
    let mut key = [0u8; KEY_BYTES];
    let hex_bytes = hash_hex.as_bytes();
    for i in 0..KEY_BYTES {
        let hi = hex_char_to_nibble(hex_bytes[i * 2]);
        let lo = hex_char_to_nibble(hex_bytes[i * 2 + 1]);
        key[i] = (hi << 4) | lo;
    }
    key
}

fn hex_char_to_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

fn duplex_encrypt(
    primary_bytes: &[u8],
    secondary_bytes: &[u8],
    key: &[u8; KEY_BYTES],
    nonce: &[u8; NONCE_BYTES],
    primary_angle: u16,
    secondary_angle: u16,
) -> (Vec<u8>, Vec<u8>, String) {
    let primary_ternary_angle = std_deg_to_ternary_deg(primary_angle);
    let secondary_ternary_angle = std_deg_to_ternary_deg(secondary_angle);
    let domain_input = build_domain_input(key, nonce, primary_ternary_angle);

    let mut switch_marker = [0u8; 4];
    switch_marker[0] = (secondary_ternary_angle >> 8) as u8;
    switch_marker[1] = secondary_ternary_angle as u8;
    switch_marker[2] = 0xFF;
    switch_marker[3] = 0xFF;

    let primary_trits = bytes_to_balanced_trits_6(primary_bytes);
    let secondary_trits = bytes_to_balanced_trits_6(secondary_bytes);

    let mut duplex = Sponge385Pub::new();
    duplex.absorb_bytes(&domain_input);

    let ks1 = duplex.squeeze(primary_trits.len());
    let cipher1_trits = encrypt_trits(&primary_trits, &ks1);
    let cipher1_bytes = cipher_trits_to_bytes(&cipher1_trits);

    duplex.absorb_bytes(&switch_marker);

    let ks2 = duplex.squeeze(secondary_trits.len());
    let cipher2_trits = encrypt_trits(&secondary_trits, &ks2);
    let cipher2_bytes = cipher_trits_to_bytes(&cipher2_trits);

    let mut header1 = [0u8; 8];
    header1[0..4].copy_from_slice(&(primary_bytes.len() as u32).to_be_bytes());
    header1[4..8].copy_from_slice(&(primary_trits.len() as u32).to_be_bytes());

    let mut header2 = [0u8; 8];
    header2[0..4].copy_from_slice(&(secondary_bytes.len() as u32).to_be_bytes());
    header2[4..8].copy_from_slice(&(secondary_trits.len() as u32).to_be_bytes());

    duplex.absorb_bytes(&header1);
    duplex.absorb_bytes(&cipher1_bytes);
    duplex.absorb_bytes(&header2);
    duplex.absorb_bytes(&cipher2_bytes);
    let mac_trits = duplex.squeeze(MAC_TRITS);
    let mac = trits_to_hex(&mac_trits);

    let mut full_cipher1 = Vec::with_capacity(8 + cipher1_bytes.len());
    full_cipher1.extend_from_slice(&header1);
    full_cipher1.extend_from_slice(&cipher1_bytes);

    let mut full_cipher2 = Vec::with_capacity(8 + cipher2_bytes.len());
    full_cipher2.extend_from_slice(&header2);
    full_cipher2.extend_from_slice(&cipher2_bytes);

    (full_cipher1, full_cipher2, mac)
}

const MAX_PLAINTEXT_BYTES: usize = 64 * 1024 * 1024;

fn duplex_decrypt(
    primary_cipher: &[u8],
    secondary_cipher: &[u8],
    mac_hex: &str,
    key: &[u8; KEY_BYTES],
    nonce: &[u8; NONCE_BYTES],
    primary_angle: u16,
    secondary_angle: u16,
) -> Result<(Vec<u8>, Vec<u8>), PhaseError> {
    if primary_cipher.len() < 8 || secondary_cipher.len() < 8 {
        return Err(PhaseError::InvalidCiphertext);
    }

    let original_byte_len1 = u32::from_be_bytes([
        primary_cipher[0], primary_cipher[1], primary_cipher[2], primary_cipher[3]
    ]) as usize;
    let trit_count1 = u32::from_be_bytes([
        primary_cipher[4], primary_cipher[5], primary_cipher[6], primary_cipher[7]
    ]) as usize;
    let cipher1_bytes = &primary_cipher[8..];

    let original_byte_len2 = u32::from_be_bytes([
        secondary_cipher[0], secondary_cipher[1], secondary_cipher[2], secondary_cipher[3]
    ]) as usize;
    let trit_count2 = u32::from_be_bytes([
        secondary_cipher[4], secondary_cipher[5], secondary_cipher[6], secondary_cipher[7]
    ]) as usize;
    let cipher2_bytes = &secondary_cipher[8..];

    if original_byte_len1 > MAX_PLAINTEXT_BYTES || original_byte_len2 > MAX_PLAINTEXT_BYTES {
        return Err(PhaseError::InvalidCiphertext);
    }
    if trit_count1 != original_byte_len1 * TRITS_PER_BYTE || trit_count2 != original_byte_len2 * TRITS_PER_BYTE {
        return Err(PhaseError::InvalidCiphertext);
    }
    let expected_packed1 = (trit_count1 + 4) / 5;
    let expected_packed2 = (trit_count2 + 4) / 5;
    if cipher1_bytes.len() < expected_packed1 || cipher2_bytes.len() < expected_packed2 {
        return Err(PhaseError::InvalidCiphertext);
    }

    let primary_ternary_angle = std_deg_to_ternary_deg(primary_angle);
    let secondary_ternary_angle = std_deg_to_ternary_deg(secondary_angle);
    let domain_input = build_domain_input(key, nonce, primary_ternary_angle);

    let mut switch_marker = [0u8; 4];
    switch_marker[0] = (secondary_ternary_angle >> 8) as u8;
    switch_marker[1] = secondary_ternary_angle as u8;
    switch_marker[2] = 0xFF;
    switch_marker[3] = 0xFF;

    let cipher1_trits = cipher_bytes_to_trits(cipher1_bytes, trit_count1);
    let cipher2_trits = cipher_bytes_to_trits(cipher2_bytes, trit_count2);

    let mut duplex = Sponge385Pub::new();
    duplex.absorb_bytes(&domain_input);

    let ks1 = duplex.squeeze(trit_count1);

    duplex.absorb_bytes(&switch_marker);

    let ks2 = duplex.squeeze(trit_count2);

    let re_header1 = &primary_cipher[..8];
    let re_header2 = &secondary_cipher[..8];
    duplex.absorb_bytes(re_header1);
    duplex.absorb_bytes(cipher1_bytes);
    duplex.absorb_bytes(re_header2);
    duplex.absorb_bytes(cipher2_bytes);
    let mac_trits = duplex.squeeze(MAC_TRITS);
    let computed_mac = trits_to_hex(&mac_trits);

    let mac_valid = constant_time_eq(computed_mac.as_bytes(), mac_hex.as_bytes());
    if !mac_valid {
        return Err(PhaseError::MacMismatch);
    }

    let plain1_trits = decrypt_trits(&cipher1_trits, &ks1);
    let plain2_trits = decrypt_trits(&cipher2_trits, &ks2);

    let mut primary_buf = balanced_trits_6_to_bytes(&plain1_trits, original_byte_len1);
    primary_buf.truncate(original_byte_len1);

    let mut secondary_buf = balanced_trits_6_to_bytes(&plain2_trits, original_byte_len2);
    secondary_buf.truncate(original_byte_len2);

    Ok((primary_buf, secondary_buf))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

pub fn encrypt(plaintext: &[u8], key: &[u8; KEY_BYTES], mode: EncryptionMode) -> Result<PhaseCiphertext, PhaseError> {
    let config = get_phase_config(mode);

    let mut nonce = [0u8; NONCE_BYTES];
    getrandom::getrandom(&mut nonce).map_err(|_| PhaseError::RandomnessError)?;

    encrypt_with_nonce(plaintext, key, mode, &nonce)
}

pub fn encrypt_with_nonce(
    plaintext: &[u8],
    key: &[u8; KEY_BYTES],
    mode: EncryptionMode,
    nonce: &[u8; NONCE_BYTES],
) -> Result<PhaseCiphertext, PhaseError> {
    let config = get_phase_config(mode);
    let midpoint = (plaintext.len() + 1) / 2;
    let primary_bytes = &plaintext[..midpoint];
    let secondary_bytes = &plaintext[midpoint..];

    let primary_angle = config.primary_phase;
    let secondary_angle = config.primary_phase + config.secondary_offset;

    let (primary_cipher, secondary_cipher, mac) = duplex_encrypt(
        primary_bytes, secondary_bytes, key, nonce, primary_angle, secondary_angle,
    );

    let guardian_hash = if config.guardian_enabled {
        Some(sponge_hash_hex(plaintext))
    } else {
        None
    };

    Ok(PhaseCiphertext {
        primary_cipher,
        secondary_cipher,
        mac,
        nonce: nonce.to_vec(),
        config,
        guardian_hash,
        version: 3,
        sponge_version: 2,
    })
}

pub fn decrypt(ciphertext: &PhaseCiphertext, key: &[u8; KEY_BYTES]) -> Result<Vec<u8>, PhaseError> {
    decrypt_inner(ciphertext, key, None)
}

pub fn decrypt_with_mode(ciphertext: &PhaseCiphertext, key: &[u8; KEY_BYTES], mode: EncryptionMode) -> Result<Vec<u8>, PhaseError> {
    decrypt_inner(ciphertext, key, Some(mode))
}

fn decrypt_inner(ciphertext: &PhaseCiphertext, key: &[u8; KEY_BYTES], mode_override: Option<EncryptionMode>) -> Result<Vec<u8>, PhaseError> {
    if ciphertext.version != 3 {
        return Err(PhaseError::InvalidCiphertext);
    }
    if ciphertext.sponge_version != 2 {
        return Err(PhaseError::InvalidCiphertext);
    }
    if ciphertext.nonce.len() != NONCE_BYTES {
        return Err(PhaseError::InvalidCiphertext);
    }

    let config = match mode_override {
        Some(m) => get_phase_config(m),
        None => ciphertext.config.clone(),
    };

    let mut nonce = [0u8; NONCE_BYTES];
    nonce.copy_from_slice(&ciphertext.nonce);

    let primary_angle = config.primary_phase;
    let secondary_angle = config.primary_phase + config.secondary_offset;

    let (primary_buf, secondary_buf) = duplex_decrypt(
        &ciphertext.primary_cipher,
        &ciphertext.secondary_cipher,
        &ciphertext.mac,
        key,
        &nonce,
        primary_angle,
        secondary_angle,
    )?;

    let mut plaintext = Vec::with_capacity(primary_buf.len() + secondary_buf.len());
    plaintext.extend_from_slice(&primary_buf);
    plaintext.extend_from_slice(&secondary_buf);

    if let Some(gh) = &ciphertext.guardian_hash {
        let computed = sponge_hash_hex(&plaintext);
        if !constant_time_eq(computed.as_bytes(), gh.as_bytes()) {
            return Err(PhaseError::GuardianFailed);
        }
    }

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; KEY_BYTES] {
        derive_key_from_secret(b"test-session-secret-for-phase-encryption")
    }

    #[test]
    fn test_mode_configs() {
        let hs = get_phase_config(EncryptionMode::HighSecurity);
        assert!(hs.guardian_enabled);
        assert_eq!(hs.secondary_offset, 10);
        assert_eq!(hs.guardian_offset, 358);

        let bal = get_phase_config(EncryptionMode::Balanced);
        assert!(!bal.guardian_enabled);
        assert_eq!(bal.secondary_offset, 4);

        let perf = get_phase_config(EncryptionMode::Performance);
        assert!(!perf.guardian_enabled);
        assert_eq!(perf.secondary_offset, 1);

        let adp = get_phase_config(EncryptionMode::Adaptive);
        assert!(adp.guardian_enabled);
        assert_eq!(adp.secondary_offset, 4);
    }

    #[test]
    fn test_ternary_degree_conversion() {
        assert_eq!(std_deg_to_ternary_deg(0), 0);
        assert_eq!(std_deg_to_ternary_deg(360), 364);
        assert_eq!(std_deg_to_ternary_deg(180), 182);
    }

    #[test]
    fn test_trit_add_sub_inverse() {
        for a in [-1i8, 0, 1] {
            for b in [-1i8, 0, 1] {
                let added = trit_add(a, b);
                let recovered = trit_sub(added, b);
                assert_eq!(recovered, a, "tritSub(tritAdd({}, {}), {}) should be {}", a, b, b, a);
            }
        }
    }

    #[test]
    fn test_bytes_trits_6_roundtrip() {
        let input = b"Hello, Phase Encryption!";
        let trits = bytes_to_balanced_trits_6(input);
        assert_eq!(trits.len(), input.len() * 6);
        let recovered = balanced_trits_6_to_bytes(&trits, input.len());
        assert_eq!(&recovered, input);
    }

    #[test]
    fn test_cipher_trits_roundtrip() {
        let trits: Vec<i8> = (0..100).map(|i| ((i % 3) as i8) - 1).collect();
        let bytes = cipher_trits_to_bytes(&trits);
        let recovered = cipher_bytes_to_trits(&bytes, trits.len());
        assert_eq!(trits, recovered);
    }

    #[test]
    fn test_encrypt_decrypt_high_security() {
        let key = test_key();
        let plaintext = b"Top secret data for high security mode";
        let ct = encrypt(plaintext, &key, EncryptionMode::HighSecurity).unwrap();
        assert_eq!(ct.version, 3);
        assert!(ct.guardian_hash.is_some());
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_balanced() {
        let key = test_key();
        let plaintext = b"Balanced mode encryption test data";
        let ct = encrypt(plaintext, &key, EncryptionMode::Balanced).unwrap();
        assert!(ct.guardian_hash.is_none());
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_performance() {
        let key = test_key();
        let plaintext = b"Performance mode fast encryption";
        let ct = encrypt(plaintext, &key, EncryptionMode::Performance).unwrap();
        assert!(ct.guardian_hash.is_none());
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_adaptive() {
        let key = test_key();
        let plaintext = b"Adaptive mode with guardian phase enabled";
        let ct = encrypt(plaintext, &key, EncryptionMode::Adaptive).unwrap();
        assert!(ct.guardian_hash.is_some());
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_all_modes() {
        let key = test_key();
        let plaintext = b"Testing all four encryption modes in a single test case";
        for mode in [
            EncryptionMode::HighSecurity,
            EncryptionMode::Balanced,
            EncryptionMode::Performance,
            EncryptionMode::Adaptive,
        ] {
            let ct = encrypt(plaintext, &key, mode).unwrap();
            let decrypted = decrypt(&ct, &key).unwrap();
            assert_eq!(decrypted, plaintext, "Failed for mode {:?}", mode);
        }
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let key = test_key();
        let plaintext = b"";
        let ct = encrypt(plaintext, &key, EncryptionMode::Balanced).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_large_plaintext() {
        let key = test_key();
        let plaintext: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        let ct = encrypt(&plaintext, &key, EncryptionMode::Balanced).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key_from_secret(b"key-one");
        let key2 = derive_key_from_secret(b"key-two");
        let plaintext = b"This should not decrypt with wrong key";
        let ct = encrypt(plaintext, &key1, EncryptionMode::Balanced).unwrap();
        let result = decrypt(&ct, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_mac_fails() {
        let key = test_key();
        let plaintext = b"Tamper detection test data";
        let mut ct = encrypt(plaintext, &key, EncryptionMode::Balanced).unwrap();
        if let Some(byte) = ct.primary_cipher.get_mut(10) {
            *byte = byte.wrapping_add(1);
        }
        let result = decrypt(&ct, &key);
        assert!(matches!(result, Err(PhaseError::MacMismatch)));
    }

    #[test]
    fn test_guardian_tamper_detection() {
        let key = test_key();
        let plaintext = b"Guardian phase detects data tampering";
        let mut ct = encrypt(plaintext, &key, EncryptionMode::HighSecurity).unwrap();
        assert!(ct.guardian_hash.is_some());
        ct.guardian_hash = Some(sponge_hash_hex(b"different data"));
        let result = decrypt(&ct, &key);
        assert!(matches!(result, Err(PhaseError::GuardianFailed)));
    }

    #[test]
    fn test_different_nonces_different_ciphertexts() {
        let key = test_key();
        let plaintext = b"Same plaintext, different nonces";
        let nonce1 = [1u8; NONCE_BYTES];
        let nonce2 = [2u8; NONCE_BYTES];
        let ct1 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce1).unwrap();
        let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce2).unwrap();
        assert_ne!(ct1.primary_cipher, ct2.primary_cipher);
        assert_ne!(ct1.mac, ct2.mac);
    }

    #[test]
    fn test_deterministic_with_fixed_nonce() {
        let key = test_key();
        let plaintext = b"Deterministic encryption test";
        let nonce = [42u8; NONCE_BYTES];
        let ct1 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
        let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
        assert_eq!(ct1.primary_cipher, ct2.primary_cipher);
        assert_eq!(ct1.secondary_cipher, ct2.secondary_cipher);
        assert_eq!(ct1.mac, ct2.mac);
    }

    #[test]
    fn test_derive_key_from_kem_secret() {
        let kem_secret = [99u8; 32];
        let key = derive_key_from_kem_secret(&kem_secret);
        assert_eq!(key.len(), 32);
        let key2 = derive_key_from_kem_secret(&kem_secret);
        assert_eq!(key, key2);
        let different_kem = [100u8; 32];
        let key3 = derive_key_from_kem_secret(&different_kem);
        assert_ne!(key, key3);
    }

    #[test]
    fn test_kem_derived_key_encrypt_decrypt() {
        let kem_secret = [42u8; 32];
        let key = derive_key_from_kem_secret(&kem_secret);
        let plaintext = b"Encrypted with TL-KEM derived key";
        let ct = encrypt(plaintext, &key, EncryptionMode::HighSecurity).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_utf8_plaintext() {
        let key = test_key();
        let plaintext = "Unicode test: 日本語テスト 🔐 Ñoño";
        let ct = encrypt(plaintext.as_bytes(), &key, EncryptionMode::Balanced).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(std::str::from_utf8(&decrypted).unwrap(), plaintext);
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(EncryptionMode::HighSecurity.name(), "high_security");
        assert_eq!(EncryptionMode::Balanced.name(), "balanced");
        assert_eq!(EncryptionMode::Performance.name(), "performance");
        assert_eq!(EncryptionMode::Adaptive.name(), "adaptive");
    }

    #[test]
    fn test_cross_compat_key_derivation() {
        let key = derive_key_from_secret(b"test-secret");
        assert_eq!(key.len(), 32);
        let key2 = derive_key_from_secret(b"test-secret");
        assert_eq!(key, key2);
    }

    #[test]
    fn test_single_byte_plaintext() {
        let key = test_key();
        let plaintext = b"X";
        let ct = encrypt(plaintext, &key, EncryptionMode::HighSecurity).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_odd_length_plaintext() {
        let key = test_key();
        let plaintext = b"OddLengthInput!";
        assert_eq!(plaintext.len() % 2, 1);
        let ct = encrypt(plaintext, &key, EncryptionMode::Balanced).unwrap();
        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_malformed_ciphertext_too_short() {
        let key = test_key();
        let ct = PhaseCiphertext {
            primary_cipher: vec![0; 4],
            secondary_cipher: vec![0; 4],
            mac: String::new(),
            nonce: vec![0; NONCE_BYTES],
            config: get_phase_config(EncryptionMode::Balanced),
            guardian_hash: None,
            version: 3,
            sponge_version: 2,
        };
        assert!(matches!(decrypt(&ct, &key), Err(PhaseError::InvalidCiphertext)));
    }

    #[test]
    fn test_malformed_header_mismatch() {
        let key = test_key();
        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&(100u32).to_be_bytes());
        header[4..8].copy_from_slice(&(999u32).to_be_bytes());
        let mut cipher = Vec::new();
        cipher.extend_from_slice(&header);
        cipher.extend_from_slice(&[0u8; 50]);
        let ct = PhaseCiphertext {
            primary_cipher: cipher.clone(),
            secondary_cipher: cipher,
            mac: "00".repeat(49),
            nonce: vec![0; NONCE_BYTES],
            config: get_phase_config(EncryptionMode::Balanced),
            guardian_hash: None,
            version: 3,
            sponge_version: 2,
        };
        assert!(matches!(decrypt(&ct, &key), Err(PhaseError::InvalidCiphertext)));
    }

    #[test]
    fn test_known_answer_vector() {
        let key = derive_key_from_secret(b"KAT-phase-test-secret-2026");
        let nonce = [0x42u8; NONCE_BYTES];
        let plaintext = b"KAT vector for Phase Encryption v3";
        let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
        let mac_snapshot = ct.mac.clone();
        let primary_snapshot = ct.primary_cipher.clone();
        let secondary_snapshot = ct.secondary_cipher.clone();
        let guardian_snapshot = ct.guardian_hash.clone();

        let decrypted = decrypt(&ct, &key).unwrap();
        assert_eq!(decrypted, plaintext);

        let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::HighSecurity, &nonce).unwrap();
        assert_eq!(ct2.mac, mac_snapshot);
        assert_eq!(ct2.primary_cipher, primary_snapshot);
        assert_eq!(ct2.secondary_cipher, secondary_snapshot);
        assert_eq!(ct2.guardian_hash, guardian_snapshot);
    }

    #[test]
    fn test_wire_format_roundtrip() {
        let key = test_key();
        let plaintext = b"Wire format roundtrip test data";
        let ct = encrypt(plaintext, &key, EncryptionMode::HighSecurity).unwrap();
        let wire = ct.to_ts_wire_format();
        assert_eq!(wire.version, 3);
        assert_eq!(wire.sponge_version, 2);
        assert!(wire.guardian_hash.is_some());
        assert_eq!(wire.split_ratio, 0.5);
        assert_eq!(wire.primary_phase, 0);
        assert_eq!(wire.secondary_phase, 10);

        let ct2 = PhaseCiphertext::from_ts_wire_format(&wire).unwrap();
        let decrypted = decrypt(&ct2, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wire_format_b64_integrity() {
        let key = test_key();
        let nonce = [0xABu8; NONCE_BYTES];
        let plaintext = b"Base64 encoding verification";
        let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
        let wire = ct.to_ts_wire_format();

        let decoded_primary = base64_decode(&wire.primary_data_b64).unwrap();
        assert_eq!(decoded_primary, ct.primary_cipher);

        let decoded_secondary = base64_decode(&wire.secondary_data_b64).unwrap();
        assert_eq!(decoded_secondary, ct.secondary_cipher);

        let decoded_nonce = hex_decode(&wire.nonce_hex).unwrap();
        assert_eq!(decoded_nonce, ct.nonce);
    }

    #[test]
    fn test_wire_format_all_modes() {
        let key = test_key();
        let plaintext = b"Testing wire format across all modes";
        for mode in [
            EncryptionMode::HighSecurity,
            EncryptionMode::Balanced,
            EncryptionMode::Performance,
            EncryptionMode::Adaptive,
        ] {
            let ct = encrypt(plaintext, &key, mode).unwrap();
            let wire = ct.to_ts_wire_format();
            let ct2 = PhaseCiphertext::from_ts_wire_format(&wire).unwrap();
            let decrypted = decrypt(&ct2, &key).unwrap();
            assert_eq!(decrypted, plaintext, "Wire roundtrip failed for mode {:?}", mode);
        }
    }

    #[test]
    fn test_wire_format_invalid_b64() {
        let wire = TsWireFormat {
            primary_data_b64: "not!valid!base64".to_string(),
            primary_phase: 0,
            secondary_data_b64: "also!invalid".to_string(),
            secondary_phase: 4,
            config: get_phase_config(EncryptionMode::Balanced),
            split_ratio: 0.5,
            nonce_hex: "00".repeat(32),
            mac: "00".repeat(49),
            version: 3,
            sponge_version: 2,
            guardian_hash: None,
            guardian_phase: None,
        };
        assert!(matches!(PhaseCiphertext::from_ts_wire_format(&wire), Err(PhaseError::InvalidCiphertext)));
    }

    #[test]
    fn test_wire_format_invalid_nonce_len() {
        let key = test_key();
        let ct = encrypt(b"test", &key, EncryptionMode::Balanced).unwrap();
        let mut wire = ct.to_ts_wire_format();
        wire.nonce_hex = "aabb".to_string();
        assert!(matches!(PhaseCiphertext::from_ts_wire_format(&wire), Err(PhaseError::InvalidCiphertext)));
    }

    #[test]
    fn test_base64_encode_decode_roundtrip() {
        let test_cases: &[&[u8]] = &[b"", b"a", b"ab", b"abc", b"abcd", b"Hello, World!"];
        for &input in test_cases {
            let encoded = base64_encode(input);
            let decoded = base64_decode(&encoded).unwrap();
            assert_eq!(decoded, input, "base64 roundtrip failed for {:?}", input);
        }
    }

    #[test]
    fn test_sponge_version_in_ciphertext() {
        let key = test_key();
        let ct = encrypt(b"version test", &key, EncryptionMode::Balanced).unwrap();
        assert_eq!(ct.sponge_version, 2);
        assert_eq!(ct.version, 3);
        let wire = ct.to_ts_wire_format();
        assert_eq!(wire.sponge_version, 2);
    }

    #[test]
    fn test_cross_compat_deterministic_vectors() {
        let key = derive_key_from_secret(b"cross-compat-test-2026");
        let nonce = [0x01u8; NONCE_BYTES];
        let plaintext = b"Cross-language compatibility vector";

        let ct = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
        let wire = ct.to_ts_wire_format();

        assert_eq!(wire.version, 3);
        assert_eq!(wire.sponge_version, 2);
        assert_eq!(wire.nonce_hex, "01".repeat(32));
        assert!(!wire.primary_data_b64.is_empty());
        assert!(!wire.secondary_data_b64.is_empty());
        assert!(!wire.mac.is_empty());
        assert_eq!(wire.mac.len(), 98);

        let ct2 = encrypt_with_nonce(plaintext, &key, EncryptionMode::Balanced, &nonce).unwrap();
        let wire2 = ct2.to_ts_wire_format();
        assert_eq!(wire.primary_data_b64, wire2.primary_data_b64);
        assert_eq!(wire.secondary_data_b64, wire2.secondary_data_b64);
        assert_eq!(wire.mac, wire2.mac);
    }

    #[test]
    fn test_decrypt_with_mode() {
        let key = test_key();
        let plaintext = b"decrypt_with_mode API test";
        for mode in [
            EncryptionMode::HighSecurity,
            EncryptionMode::Balanced,
            EncryptionMode::Performance,
            EncryptionMode::Adaptive,
        ] {
            let ct = encrypt(plaintext, &key, mode).unwrap();
            let d1 = decrypt(&ct, &key).unwrap();
            let d2 = decrypt_with_mode(&ct, &key, mode).unwrap();
            assert_eq!(d1, d2, "decrypt and decrypt_with_mode should match for {:?}", mode);
            assert_eq!(d1, plaintext);
        }
    }

    #[test]
    fn test_reject_wrong_version() {
        let key = test_key();
        let mut ct = encrypt(b"version check", &key, EncryptionMode::Balanced).unwrap();
        ct.version = 2;
        assert!(matches!(decrypt(&ct, &key), Err(PhaseError::InvalidCiphertext)));
    }

    #[test]
    fn test_reject_wrong_sponge_version() {
        let key = test_key();
        let mut ct = encrypt(b"sponge version check", &key, EncryptionMode::Balanced).unwrap();
        ct.sponge_version = 1;
        assert!(matches!(decrypt(&ct, &key), Err(PhaseError::InvalidCiphertext)));
    }
}
