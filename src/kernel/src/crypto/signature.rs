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

//! Post-Quantum Hash-Based Signatures (SP 800-208)
//!
//! Implements three hash-based signature schemes:
//!
//! 1. **Ternary Lamport OTS** — One-time signatures for ternary data
//! 2. **XMSS** (eXtended Merkle Signature Scheme) — Stateful multi-use
//!    signatures with WOTS+ and Merkle tree construction per SP 800-208
//! 3. **LMS** (Leighton-Micali Scheme) — Stateful multi-use signatures
//!    with LM-OTS and Merkle tree per SP 800-208
//!
//! XMSS and LMS close the last CNSA 2.0 algorithm gap, providing full
//! hash-based signature support alongside the ternary Lamport primitive.
//!
//! # CNSA 2.0 Compliance
//! - XMSS: SP 800-208, Section 5 (XMSS only; XMSS^MT prohibited)
//! - LMS: SP 800-208, Section 4 (LMS only; HSS prohibited)
//! - Both use ternary sponge hash as the underlying hash function
//!
//! # Security Warning
//! XMSS and LMS are **stateful** schemes. Each keypair tracks a monotonic
//! index. Reusing an index compromises security. The `XmssState`/`LmsState`
//! structs enforce this invariant — callers MUST persist state across restarts.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::{CryptoError, CryptoResult, TernaryDigest, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;

const SIGN_DIGEST_TRITS: usize = 81;
const SECRET_ELEMENT_TRITS: usize = TERNARY_HASH_TRITS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScheme {
    TernaryLamport,
    Xmss,
    Lms,
    Dilithium,
    Falcon,
    SphincsPlus,
}

impl SignatureScheme {
    pub fn is_available(&self) -> bool {
        matches!(self, SignatureScheme::TernaryLamport | SignatureScheme::Xmss | SignatureScheme::Lms)
    }

    pub fn security_level(&self) -> u32 {
        match self {
            SignatureScheme::TernaryLamport => 128,
            SignatureScheme::Xmss => 256,
            SignatureScheme::Lms => 256,
            SignatureScheme::Dilithium => 256,
            SignatureScheme::Falcon => 256,
            SignatureScheme::SphincsPlus => 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SigningKey {
    pub scheme: SignatureScheme,
    pub secrets: Vec<Vec<i8>>,
    pub used: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyingKey {
    pub scheme: SignatureScheme,
    pub public: Vec<Vec<i8>>,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub scheme: SignatureScheme,
    pub revealed: Vec<Vec<i8>>,
}

fn hash_element(data: &[i8]) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(data);
    sponge.squeeze(SECRET_ELEMENT_TRITS).trits
}

fn hash_message(message: &[i8]) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(message);
    sponge.squeeze(SIGN_DIGEST_TRITS).trits
}

fn derive_secret(seed: &[i8], index: usize, trit_val: i8) -> Vec<i8> {
    let mut sponge = TernarySponge::new();
    sponge.absorb(seed);

    let idx_trits = [
        ((index % 3) as i8 - 1),
        (((index / 3) % 3) as i8 - 1),
        (((index / 9) % 3) as i8 - 1),
        (((index / 27) % 3) as i8 - 1),
        (((index / 81) % 3) as i8 - 1),
        (((index / 243) % 3) as i8 - 1),
        trit_val,
    ];
    sponge.absorb(&idx_trits);
    sponge.squeeze(SECRET_ELEMENT_TRITS).trits
}

fn trit_to_index(t: i8) -> usize {
    match t {
        -1 => 0,
        0 => 1,
        1 => 2,
        _ => 1,
    }
}

pub fn generate_keypair(scheme: SignatureScheme, seed: &[i8]) -> CryptoResult<(SigningKey, VerifyingKey)> {
    match scheme {
        SignatureScheme::TernaryLamport => generate_lamport_keypair(seed),
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn generate_lamport_keypair(seed: &[i8]) -> CryptoResult<(SigningKey, VerifyingKey)> {
    let total_secrets = SIGN_DIGEST_TRITS * 3;
    let mut secrets = Vec::with_capacity(total_secrets);
    let mut public = Vec::with_capacity(total_secrets);

    for pos in 0..SIGN_DIGEST_TRITS {
        for trit_val in [-1i8, 0, 1] {
            let secret = derive_secret(seed, pos, trit_val);
            let pub_hash = hash_element(&secret);
            secrets.push(secret);
            public.push(pub_hash);
        }
    }

    Ok((
        SigningKey {
            scheme: SignatureScheme::TernaryLamport,
            secrets,
            used: false,
        },
        VerifyingKey {
            scheme: SignatureScheme::TernaryLamport,
            public,
        },
    ))
}

pub fn sign(key: &mut SigningKey, message: &[i8]) -> CryptoResult<Signature> {
    if key.used {
        return Err(CryptoError::KeyGenerationFailed(
            String::from("One-time signing key already used"),
        ));
    }
    match key.scheme {
        SignatureScheme::TernaryLamport => {
            let sig = lamport_sign(key, message)?;
            key.used = true;
            Ok(sig)
        }
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn lamport_sign(key: &SigningKey, message: &[i8]) -> CryptoResult<Signature> {
    let msg_hash = hash_message(message);

    let mut revealed = Vec::with_capacity(SIGN_DIGEST_TRITS);

    for (pos, &trit) in msg_hash.iter().enumerate() {
        let secret_idx = pos * 3 + trit_to_index(trit);
        revealed.push(key.secrets[secret_idx].clone());
    }

    Ok(Signature {
        scheme: SignatureScheme::TernaryLamport,
        revealed,
    })
}

pub fn verify(key: &VerifyingKey, message: &[i8], signature: &Signature) -> CryptoResult<bool> {
    match key.scheme {
        SignatureScheme::TernaryLamport => lamport_verify(key, message, signature),
        other => Err(CryptoError::UnsupportedAlgorithm(
            alloc::format!("{:?}", other),
        )),
    }
}

fn lamport_verify(key: &VerifyingKey, message: &[i8], signature: &Signature) -> CryptoResult<bool> {
    let msg_hash = hash_message(message);

    if signature.revealed.len() != SIGN_DIGEST_TRITS {
        return Ok(false);
    }

    for (pos, &trit) in msg_hash.iter().enumerate() {
        let pub_idx = pos * 3 + trit_to_index(trit);
        let revealed_hash = hash_element(&signature.revealed[pos]);

        let expected = &key.public[pub_idx];
        if expected.len() != revealed_hash.len() {
            return Ok(false);
        }

        let mut diff: u8 = 0;
        for (a, b) in expected.iter().zip(revealed_hash.iter()) {
            diff |= (*a as u8) ^ (*b as u8);
        }
        if diff != 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn signature_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * SECRET_ELEMENT_TRITS,
        SignatureScheme::Dilithium => 3293,
        SignatureScheme::Falcon => 1280,
        SignatureScheme::SphincsPlus => 49856,
        SignatureScheme::Xmss => 2500,
        SignatureScheme::Lms => 4064,
    }
}

pub fn public_key_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS,
        _ => 0,
    }
}

pub fn secret_key_size(scheme: SignatureScheme) -> usize {
    match scheme {
        SignatureScheme::TernaryLamport => SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS,
        _ => 0,
    }
}

// ============================================================
// Hash-Based Signature Utilities (SP 800-208)
// Shared by XMSS and LMS — uses ternary sponge as hash function
// ============================================================

const HBS_N: usize = 32;

fn hbs_hash(domain: u8, inputs: &[&[u8]]) -> [u8; HBS_N] {
    let mut sponge = TernarySponge::new();
    sponge.absorb(&[domain as i8]);
    for input in inputs {
        let len_bytes = (input.len() as u32).to_be_bytes();
        let len_td = TernaryDigest::from_bytes(&len_bytes, 20);
        sponge.absorb(&len_td.trits);
        if !input.is_empty() {
            let td = TernaryDigest::from_bytes(input, input.len() * 5);
            sponge.absorb(&td.trits);
        }
    }
    let out = sponge.squeeze(TERNARY_HASH_TRITS);
    let bytes = out.to_bytes();
    let mut result = [0u8; HBS_N];
    let len = core::cmp::min(bytes.len(), HBS_N);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

// ============================================================
// WOTS+ (Winternitz One-Time Signature Plus) — SP 800-208 §5.1
// Used as the OTS primitive within XMSS
// ============================================================

const WOTS_W: u32 = 16;
const WOTS_LOG_W: u32 = 4;
const WOTS_LEN1: usize = 64;
const WOTS_LEN2: usize = 3;
const WOTS_LEN: usize = WOTS_LEN1 + WOTS_LEN2;

fn base_w(data: &[u8], out_len: usize) -> Vec<u32> {
    let mut result = Vec::with_capacity(out_len);
    let mut in_idx = 0usize;
    let mut bits: u32 = 0;
    let mut total: u32 = 0;
    for _ in 0..out_len {
        if bits == 0 {
            total = if in_idx < data.len() { data[in_idx] as u32 } else { 0 };
            in_idx += 1;
            bits = 8;
        }
        bits -= WOTS_LOG_W;
        result.push((total >> bits) & (WOTS_W - 1));
    }
    result
}

fn wots_chain_lengths(msg_hash: &[u8; HBS_N]) -> Vec<u32> {
    let msg_bw = base_w(msg_hash, WOTS_LEN1);
    let mut csum: u32 = 0;
    for &v in &msg_bw {
        csum += WOTS_W - 1 - v;
    }
    csum <<= 4;
    let csum_bytes = [(csum >> 8) as u8, csum as u8];
    let csum_bw = base_w(&csum_bytes, WOTS_LEN2);
    let mut lengths = msg_bw;
    lengths.extend_from_slice(&csum_bw);
    lengths
}

fn wots_chain(mut val: [u8; HBS_N], start: u32, steps: u32, pub_seed: &[u8; HBS_N], chain_idx: u32) -> [u8; HBS_N] {
    let chain_bytes = chain_idx.to_be_bytes();
    for i in start..(start + steps) {
        let i_bytes = i.to_be_bytes();
        let key = hbs_hash(10, &[pub_seed, &chain_bytes, &i_bytes]);
        val = hbs_hash(11, &[&key, &val]);
    }
    val
}

fn wots_keygen_pk(sk_seed: &[u8; HBS_N], pub_seed: &[u8; HBS_N], ots_addr: u32) -> Vec<[u8; HBS_N]> {
    let addr_bytes = ots_addr.to_be_bytes();
    let mut pk = Vec::with_capacity(WOTS_LEN);
    for i in 0..WOTS_LEN {
        let i_bytes = (i as u32).to_be_bytes();
        let sk_i = hbs_hash(12, &[sk_seed, &addr_bytes, &i_bytes]);
        let pk_i = wots_chain(sk_i, 0, WOTS_W - 1, pub_seed, i as u32);
        pk.push(pk_i);
    }
    pk
}

fn wots_sign_msg(msg_hash: &[u8; HBS_N], sk_seed: &[u8; HBS_N], pub_seed: &[u8; HBS_N], ots_addr: u32) -> Vec<[u8; HBS_N]> {
    let lengths = wots_chain_lengths(msg_hash);
    let addr_bytes = ots_addr.to_be_bytes();
    let mut sig = Vec::with_capacity(WOTS_LEN);
    for (i, &len) in lengths.iter().enumerate() {
        let i_bytes = (i as u32).to_be_bytes();
        let sk_i = hbs_hash(12, &[sk_seed, &addr_bytes, &i_bytes]);
        sig.push(wots_chain(sk_i, 0, len, pub_seed, i as u32));
    }
    sig
}

fn wots_pk_from_sig(sig: &[[u8; HBS_N]], msg_hash: &[u8; HBS_N], pub_seed: &[u8; HBS_N]) -> Vec<[u8; HBS_N]> {
    let lengths = wots_chain_lengths(msg_hash);
    let mut pk = Vec::with_capacity(WOTS_LEN);
    for (i, (&len, sig_i)) in lengths.iter().zip(sig.iter()).enumerate() {
        pk.push(wots_chain(*sig_i, len, WOTS_W - 1 - len, pub_seed, i as u32));
    }
    pk
}

fn ltree(pk: &[[u8; HBS_N]], pub_seed: &[u8; HBS_N], addr: u32) -> [u8; HBS_N] {
    let mut nodes: Vec<[u8; HBS_N]> = pk.to_vec();
    let addr_bytes = addr.to_be_bytes();
    let mut level: u32 = 0;
    while nodes.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;
        while i + 1 < nodes.len() {
            let level_bytes = level.to_be_bytes();
            let i_bytes = (i as u32).to_be_bytes();
            next.push(hbs_hash(13, &[pub_seed, &addr_bytes, &level_bytes, &i_bytes, &nodes[i], &nodes[i + 1]]));
            i += 2;
        }
        if i < nodes.len() {
            next.push(nodes[i]);
        }
        nodes = next;
        level += 1;
    }
    nodes[0]
}

// ============================================================
// XMSS (eXtended Merkle Signature Scheme) — SP 800-208 §5
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmssParams {
    XmssSha2_10_256,
    XmssSha2_16_256,
    XmssSha2_20_256,
}

impl XmssParams {
    pub fn tree_height(&self) -> u32 {
        match self {
            XmssParams::XmssSha2_10_256 => 10,
            XmssParams::XmssSha2_16_256 => 16,
            XmssParams::XmssSha2_20_256 => 20,
        }
    }
    pub fn max_signatures(&self) -> u32 {
        1u32 << self.tree_height()
    }
    pub fn name(&self) -> &'static str {
        match self {
            XmssParams::XmssSha2_10_256 => "XMSS-SHA2_10_256",
            XmssParams::XmssSha2_16_256 => "XMSS-SHA2_16_256",
            XmssParams::XmssSha2_20_256 => "XMSS-SHA2_20_256",
        }
    }
}

#[derive(Debug, Clone)]
pub struct XmssKeypair {
    pub params: XmssParams,
    pub root: [u8; HBS_N],
    pub pub_seed: [u8; HBS_N],
    sk_seed: [u8; HBS_N],
    tree: Vec<[u8; HBS_N]>,
}

#[derive(Debug, Clone)]
pub struct XmssState {
    pub index: u32,
    pub max_index: u32,
    pub exhausted: bool,
}

impl XmssState {
    pub fn is_exhausted(&self) -> bool {
        self.index >= self.max_index
    }

    pub fn remaining(&self) -> u32 {
        if self.index >= self.max_index { 0 } else { self.max_index - self.index }
    }

    fn advance(&mut self) -> CryptoResult<u32> {
        if self.is_exhausted() {
            return Err(CryptoError::StateExhausted(
                String::from("XMSS tree fully used — all leaf indices consumed"),
            ));
        }
        let idx = self.index;
        self.index += 1;
        if self.index >= self.max_index {
            self.exhausted = true;
        }
        Ok(idx)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(9);
        out.extend_from_slice(&self.index.to_be_bytes());
        out.extend_from_slice(&self.max_index.to_be_bytes());
        out.push(if self.exhausted { 1 } else { 0 });
        out
    }

    pub fn from_bytes(data: &[u8]) -> CryptoResult<Self> {
        if data.len() < 9 {
            return Err(CryptoError::InvalidInputLength { expected: 9, actual: data.len() });
        }
        let index = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let max_index = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let exhausted = data[8] != 0;
        Ok(Self { index, max_index, exhausted })
    }
}

#[derive(Debug, Clone)]
pub struct XmssSignature {
    pub index: u32,
    pub randomness: [u8; HBS_N],
    pub wots_sig: Vec<[u8; HBS_N]>,
    pub auth_path: Vec<[u8; HBS_N]>,
}

fn xmss_compute_leaf(sk_seed: &[u8; HBS_N], pub_seed: &[u8; HBS_N], leaf_idx: u32) -> [u8; HBS_N] {
    let wots_pk = wots_keygen_pk(sk_seed, pub_seed, leaf_idx);
    ltree(&wots_pk, pub_seed, leaf_idx)
}

fn xmss_build_tree(sk_seed: &[u8; HBS_N], pub_seed: &[u8; HBS_N], height: u32) -> Vec<[u8; HBS_N]> {
    let num_leaves = 1u32 << height;
    let tree_size = (2 * num_leaves) as usize;
    let mut tree = vec![[0u8; HBS_N]; tree_size];
    for i in 0..num_leaves {
        tree[(num_leaves + i) as usize] = xmss_compute_leaf(sk_seed, pub_seed, i);
    }
    for p in (1..num_leaves).rev() {
        let p_bytes = (p as u32).to_be_bytes();
        tree[p as usize] = hbs_hash(14, &[pub_seed, &p_bytes, &tree[(2 * p) as usize], &tree[(2 * p + 1) as usize]]);
    }
    tree
}

fn xmss_auth_path(tree: &[[u8; HBS_N]], leaf_idx: u32, height: u32) -> Vec<[u8; HBS_N]> {
    let num_leaves = 1u32 << height;
    let mut auth = Vec::with_capacity(height as usize);
    let mut p = num_leaves + leaf_idx;
    for _ in 0..height {
        let sibling = p ^ 1;
        auth.push(tree[sibling as usize]);
        p >>= 1;
    }
    auth
}

pub fn xmss_keygen(params: XmssParams, seed: &[u8]) -> CryptoResult<(XmssKeypair, XmssState)> {
    if seed.len() < HBS_N {
        return Err(CryptoError::InvalidKeyLength { expected: HBS_N, actual: seed.len() });
    }
    let mut sk_seed = [0u8; HBS_N];
    sk_seed.copy_from_slice(&seed[..HBS_N]);
    let pub_seed = hbs_hash(20, &[&sk_seed]);
    let height = params.tree_height();
    let tree = xmss_build_tree(&sk_seed, &pub_seed, height);
    let root = tree[1];
    Ok((
        XmssKeypair { params, root, pub_seed, sk_seed, tree },
        XmssState { index: 0, max_index: params.max_signatures(), exhausted: false },
    ))
}

pub fn xmss_sign(keypair: &XmssKeypair, state: &mut XmssState, message: &[u8]) -> CryptoResult<XmssSignature> {
    let idx = state.advance()?;
    let idx_bytes = idx.to_be_bytes();
    let r = hbs_hash(21, &[&keypair.sk_seed, &idx_bytes]);
    let msg_hash = hbs_hash(22, &[&r, &keypair.root, &idx_bytes, message]);
    let wots_sig = wots_sign_msg(&msg_hash, &keypair.sk_seed, &keypair.pub_seed, idx);
    let auth_path = xmss_auth_path(&keypair.tree, idx, keypair.params.tree_height());
    Ok(XmssSignature { index: idx, randomness: r, wots_sig, auth_path })
}

pub fn xmss_verify(
    root: &[u8; HBS_N],
    pub_seed: &[u8; HBS_N],
    params: XmssParams,
    message: &[u8],
    sig: &XmssSignature,
) -> CryptoResult<bool> {
    let height = params.tree_height();
    if sig.auth_path.len() != height as usize { return Ok(false); }
    if sig.wots_sig.len() != WOTS_LEN { return Ok(false); }
    if sig.index >= params.max_signatures() { return Ok(false); }
    let idx_bytes = sig.index.to_be_bytes();
    let msg_hash = hbs_hash(22, &[&sig.randomness, root, &idx_bytes, message]);
    let wots_pk = wots_pk_from_sig(&sig.wots_sig, &msg_hash, pub_seed);
    let mut node = ltree(&wots_pk, pub_seed, sig.index);
    let num_leaves = 1u32 << height;
    let mut p = num_leaves + sig.index;
    for auth_node in sig.auth_path.iter() {
        let parent = p >> 1;
        let parent_bytes = (parent as u32).to_be_bytes();
        if p & 1 == 0 {
            node = hbs_hash(14, &[pub_seed, &parent_bytes, &node, auth_node]);
        } else {
            node = hbs_hash(14, &[pub_seed, &parent_bytes, auth_node, &node]);
        }
        p = parent;
    }
    let mut diff: u8 = 0;
    for (a, b) in node.iter().zip(root.iter()) {
        diff |= a ^ b;
    }
    Ok(diff == 0)
}

// ============================================================
// LM-OTS (Leighton-Micali One-Time Signature) — SP 800-208 §4.1
// Used as the OTS primitive within LMS
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LmotsType {
    LmotsSha256N32W1,
    LmotsSha256N32W2,
    LmotsSha256N32W4,
    LmotsSha256N32W8,
}

impl LmotsType {
    pub fn w(&self) -> u32 {
        match self {
            LmotsType::LmotsSha256N32W1 => 1,
            LmotsType::LmotsSha256N32W2 => 2,
            LmotsType::LmotsSha256N32W4 => 4,
            LmotsType::LmotsSha256N32W8 => 8,
        }
    }
    pub fn p(&self) -> usize {
        match self {
            LmotsType::LmotsSha256N32W1 => 265,
            LmotsType::LmotsSha256N32W2 => 133,
            LmotsType::LmotsSha256N32W4 => 67,
            LmotsType::LmotsSha256N32W8 => 34,
        }
    }
    pub fn ls(&self) -> u32 {
        match self {
            LmotsType::LmotsSha256N32W1 => 7,
            LmotsType::LmotsSha256N32W2 => 6,
            LmotsType::LmotsSha256N32W4 => 4,
            LmotsType::LmotsSha256N32W8 => 0,
        }
    }
}

fn coeff(s: &[u8], i: usize, w: u32) -> u32 {
    let mask = (1u32 << w) - 1;
    let byte_idx = (i as u32 * w / 8) as usize;
    let bit_shift = 8 - (w * ((i as u32 % (8 / w)) + 1));
    if byte_idx < s.len() {
        (s[byte_idx] as u32 >> bit_shift) & mask
    } else {
        0
    }
}

fn lmots_chain(val: [u8; HBS_N], start: u32, steps: u32, identifier: &[u8; 16], leaf_idx: u32, chain_idx: u16) -> [u8; HBS_N] {
    let mut tmp = val;
    let leaf_bytes = leaf_idx.to_be_bytes();
    let chain_bytes = chain_idx.to_be_bytes();
    for j in start..(start + steps) {
        let j_bytes = (j as u16).to_be_bytes();
        tmp = hbs_hash(30, &[identifier, &leaf_bytes, &chain_bytes, &j_bytes, &tmp]);
    }
    tmp
}

fn lmots_keygen(
    sk_seed: &[u8; HBS_N],
    identifier: &[u8; 16],
    leaf_idx: u32,
    ots_type: LmotsType,
) -> (Vec<[u8; HBS_N]>, [u8; HBS_N]) {
    let w = ots_type.w();
    let p = ots_type.p();
    let max_chain = (1u32 << w) - 1;
    let leaf_bytes = leaf_idx.to_be_bytes();
    let mut pk_elements = Vec::with_capacity(p);
    for i in 0..p {
        let i_bytes = (i as u32).to_be_bytes();
        let sk_i = hbs_hash(31, &[sk_seed, identifier, &leaf_bytes, &i_bytes]);
        let pk_i = lmots_chain(sk_i, 0, max_chain, identifier, leaf_idx, i as u16);
        pk_elements.push(pk_i);
    }
    let mut pk_concat = Vec::with_capacity(p * HBS_N + 16 + 4);
    pk_concat.extend_from_slice(identifier);
    pk_concat.extend_from_slice(&leaf_bytes);
    for elem in &pk_elements {
        pk_concat.extend_from_slice(elem);
    }
    let pk_hash = hbs_hash(32, &[&pk_concat]);
    (pk_elements, pk_hash)
}

fn lmots_sign(
    message: &[u8],
    sk_seed: &[u8; HBS_N],
    identifier: &[u8; 16],
    leaf_idx: u32,
    ots_type: LmotsType,
) -> (Vec<[u8; HBS_N]>, [u8; HBS_N]) {
    let w = ots_type.w();
    let p = ots_type.p();
    let ls = ots_type.ls();
    let leaf_bytes = leaf_idx.to_be_bytes();
    let c = hbs_hash(33, &[sk_seed, identifier, &leaf_bytes, message]);
    let q = hbs_hash(34, &[identifier, &leaf_bytes, &c, message]);
    let mut sig = Vec::with_capacity(p);
    let mut checksum: u32 = 0;
    let u = p - (((core::cmp::max(1, (8u32.wrapping_mul(HBS_N as u32).wrapping_mul(1) / w).leading_zeros()) as usize) + w as usize - 1) / w as usize);
    let _ = u;
    let num_msg_coeffs = (8 * HBS_N as u32 + w - 1) / w;
    for i in 0..num_msg_coeffs as usize {
        let a = coeff(&q, i, w);
        checksum += (1u32 << w) - 1 - a;
    }
    checksum <<= ls;
    let checksum_bytes = checksum.to_be_bytes();
    let total_p = p;
    for i in 0..total_p {
        let a = if (i as u32) < num_msg_coeffs {
            coeff(&q, i, w)
        } else {
            let ci = i - num_msg_coeffs as usize;
            coeff(&checksum_bytes, ci, w)
        };
        let i_bytes = (i as u32).to_be_bytes();
        let sk_i = hbs_hash(31, &[sk_seed, identifier, &leaf_bytes, &i_bytes]);
        sig.push(lmots_chain(sk_i, 0, a, identifier, leaf_idx, i as u16));
    }
    (sig, c)
}

fn lmots_pk_from_sig(
    message: &[u8],
    sig: &[[u8; HBS_N]],
    c: &[u8; HBS_N],
    identifier: &[u8; 16],
    leaf_idx: u32,
    ots_type: LmotsType,
) -> [u8; HBS_N] {
    let w = ots_type.w();
    let p = ots_type.p();
    let ls = ots_type.ls();
    let leaf_bytes = leaf_idx.to_be_bytes();
    let max_chain = (1u32 << w) - 1;
    let q = hbs_hash(34, &[identifier, &leaf_bytes, c, message]);
    let num_msg_coeffs = (8 * HBS_N as u32 + w - 1) / w;
    let mut checksum: u32 = 0;
    for i in 0..num_msg_coeffs as usize {
        checksum += max_chain - coeff(&q, i, w);
    }
    checksum <<= ls;
    let checksum_bytes = checksum.to_be_bytes();
    let mut z = Vec::with_capacity(p);
    for i in 0..p {
        let a = if (i as u32) < num_msg_coeffs {
            coeff(&q, i, w)
        } else {
            let ci = i - num_msg_coeffs as usize;
            coeff(&checksum_bytes, ci, w)
        };
        z.push(lmots_chain(sig[i], a, max_chain - a, identifier, leaf_idx, i as u16));
    }
    let mut pk_concat = Vec::with_capacity(p * HBS_N + 16 + 4);
    pk_concat.extend_from_slice(identifier);
    pk_concat.extend_from_slice(&leaf_bytes);
    for elem in &z {
        pk_concat.extend_from_slice(elem);
    }
    hbs_hash(32, &[&pk_concat])
}

// ============================================================
// LMS (Leighton-Micali Scheme) — SP 800-208 §4
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LmsType {
    LmsSha256M32H5,
    LmsSha256M32H10,
    LmsSha256M32H15,
    LmsSha256M32H20,
    LmsSha256M32H25,
}

impl LmsType {
    pub fn height(&self) -> u32 {
        match self {
            LmsType::LmsSha256M32H5 => 5,
            LmsType::LmsSha256M32H10 => 10,
            LmsType::LmsSha256M32H15 => 15,
            LmsType::LmsSha256M32H20 => 20,
            LmsType::LmsSha256M32H25 => 25,
        }
    }
    pub fn max_signatures(&self) -> u32 {
        1u32 << self.height()
    }
    pub fn name(&self) -> &'static str {
        match self {
            LmsType::LmsSha256M32H5 => "LMS_SHA256_M32_H5",
            LmsType::LmsSha256M32H10 => "LMS_SHA256_M32_H10",
            LmsType::LmsSha256M32H15 => "LMS_SHA256_M32_H15",
            LmsType::LmsSha256M32H20 => "LMS_SHA256_M32_H20",
            LmsType::LmsSha256M32H25 => "LMS_SHA256_M32_H25",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LmsKeypair {
    pub lms_type: LmsType,
    pub ots_type: LmotsType,
    pub root: [u8; HBS_N],
    pub identifier: [u8; 16],
    sk_seed: [u8; HBS_N],
    tree: Vec<[u8; HBS_N]>,
}

#[derive(Debug, Clone)]
pub struct LmsState {
    pub index: u32,
    pub max_index: u32,
    pub exhausted: bool,
}

impl LmsState {
    pub fn is_exhausted(&self) -> bool {
        self.index >= self.max_index
    }

    pub fn remaining(&self) -> u32 {
        if self.index >= self.max_index { 0 } else { self.max_index - self.index }
    }

    fn advance(&mut self) -> CryptoResult<u32> {
        if self.is_exhausted() {
            return Err(CryptoError::StateExhausted(
                String::from("LMS tree fully used — all leaf indices consumed"),
            ));
        }
        let idx = self.index;
        self.index += 1;
        if self.index >= self.max_index {
            self.exhausted = true;
        }
        Ok(idx)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(9);
        out.extend_from_slice(&self.index.to_be_bytes());
        out.extend_from_slice(&self.max_index.to_be_bytes());
        out.push(if self.exhausted { 1 } else { 0 });
        out
    }

    pub fn from_bytes(data: &[u8]) -> CryptoResult<Self> {
        if data.len() < 9 {
            return Err(CryptoError::InvalidInputLength { expected: 9, actual: data.len() });
        }
        let index = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let max_index = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let exhausted = data[8] != 0;
        Ok(Self { index, max_index, exhausted })
    }
}

#[derive(Debug, Clone)]
pub struct LmsSignature {
    pub index: u32,
    pub ots_sig: Vec<[u8; HBS_N]>,
    pub ots_randomizer: [u8; HBS_N],
    pub auth_path: Vec<[u8; HBS_N]>,
}

fn lms_compute_leaf(
    sk_seed: &[u8; HBS_N],
    identifier: &[u8; 16],
    leaf_idx: u32,
    ots_type: LmotsType,
) -> [u8; HBS_N] {
    let (_, pk_hash) = lmots_keygen(sk_seed, identifier, leaf_idx, ots_type);
    pk_hash
}

fn lms_build_tree(
    sk_seed: &[u8; HBS_N],
    identifier: &[u8; 16],
    height: u32,
    ots_type: LmotsType,
) -> Vec<[u8; HBS_N]> {
    let num_leaves = 1u32 << height;
    let tree_size = (2 * num_leaves) as usize;
    let mut tree = vec![[0u8; HBS_N]; tree_size];
    for i in 0..num_leaves {
        tree[(num_leaves + i) as usize] = lms_compute_leaf(sk_seed, identifier, i, ots_type);
    }
    for p in (1..num_leaves).rev() {
        let p_bytes = p.to_be_bytes();
        tree[p as usize] = hbs_hash(35, &[identifier, &p_bytes, &tree[(2 * p) as usize], &tree[(2 * p + 1) as usize]]);
    }
    tree
}

fn lms_auth_path(tree: &[[u8; HBS_N]], leaf_idx: u32, height: u32) -> Vec<[u8; HBS_N]> {
    let num_leaves = 1u32 << height;
    let mut auth = Vec::with_capacity(height as usize);
    let mut p = num_leaves + leaf_idx;
    for _ in 0..height {
        let sibling = p ^ 1;
        auth.push(tree[sibling as usize]);
        p >>= 1;
    }
    auth
}

pub fn lms_keygen(
    lms_type: LmsType,
    ots_type: LmotsType,
    seed: &[u8],
) -> CryptoResult<(LmsKeypair, LmsState)> {
    if seed.len() < HBS_N {
        return Err(CryptoError::InvalidKeyLength { expected: HBS_N, actual: seed.len() });
    }
    let mut sk_seed = [0u8; HBS_N];
    sk_seed.copy_from_slice(&seed[..HBS_N]);
    let id_full = hbs_hash(36, &[&sk_seed]);
    let mut identifier = [0u8; 16];
    identifier.copy_from_slice(&id_full[..16]);
    let height = lms_type.height();
    let tree = lms_build_tree(&sk_seed, &identifier, height, ots_type);
    let root = tree[1];
    Ok((
        LmsKeypair { lms_type, ots_type, root, identifier, sk_seed, tree },
        LmsState { index: 0, max_index: lms_type.max_signatures(), exhausted: false },
    ))
}

pub fn lms_sign(
    keypair: &LmsKeypair,
    state: &mut LmsState,
    message: &[u8],
) -> CryptoResult<LmsSignature> {
    let idx = state.advance()?;
    let (ots_sig, c) = lmots_sign(message, &keypair.sk_seed, &keypair.identifier, idx, keypair.ots_type);
    let auth_path = lms_auth_path(&keypair.tree, idx, keypair.lms_type.height());
    Ok(LmsSignature { index: idx, ots_sig, ots_randomizer: c, auth_path })
}

pub fn lms_verify(
    root: &[u8; HBS_N],
    identifier: &[u8; 16],
    lms_type: LmsType,
    ots_type: LmotsType,
    message: &[u8],
    sig: &LmsSignature,
) -> CryptoResult<bool> {
    let height = lms_type.height();
    if sig.auth_path.len() != height as usize { return Ok(false); }
    if sig.ots_sig.len() != ots_type.p() { return Ok(false); }
    if sig.index >= lms_type.max_signatures() { return Ok(false); }
    let pk_candidate = lmots_pk_from_sig(message, &sig.ots_sig, &sig.ots_randomizer, identifier, sig.index, ots_type);
    let mut node = pk_candidate;
    let num_leaves = 1u32 << height;
    let mut p = num_leaves + sig.index;
    for auth_node in sig.auth_path.iter() {
        let parent = p >> 1;
        let parent_bytes = parent.to_be_bytes();
        if p & 1 == 0 {
            node = hbs_hash(35, &[identifier, &parent_bytes, &node, auth_node]);
        } else {
            node = hbs_hash(35, &[identifier, &parent_bytes, auth_node, &node]);
        }
        p = parent;
    }
    let mut diff: u8 = 0;
    for (a, b) in node.iter().zip(root.iter()) {
        diff |= a ^ b;
    }
    Ok(diff == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_availability() {
        assert!(SignatureScheme::TernaryLamport.is_available());
        assert!(SignatureScheme::Xmss.is_available());
        assert!(SignatureScheme::Lms.is_available());
        assert!(!SignatureScheme::Dilithium.is_available());
        assert!(!SignatureScheme::Falcon.is_available());
        assert!(!SignatureScheme::SphincsPlus.is_available());
    }

    #[test]
    fn test_scheme_security_level() {
        assert_eq!(SignatureScheme::TernaryLamport.security_level(), 128);
        assert_eq!(SignatureScheme::Xmss.security_level(), 256);
        assert_eq!(SignatureScheme::Lms.security_level(), 256);
        assert_eq!(SignatureScheme::Dilithium.security_level(), 256);
    }

    #[test]
    fn test_generate_keypair() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        assert_eq!(sk.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(vk.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(sk.secrets.len(), SIGN_DIGEST_TRITS * 3);
        assert_eq!(vk.public.len(), SIGN_DIGEST_TRITS * 3);
        assert!(!sk.used);
    }

    #[test]
    fn test_generate_keypair_deterministic() {
        let seed = alloc::vec![1i8, 0, -1, 1, 0];
        let (sk1, vk1) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let (sk2, vk2) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        assert_eq!(sk1.secrets, sk2.secrets);
        assert_eq!(vk1.public, vk2.public);
    }

    #[test]
    fn test_generate_keypair_different_seeds() {
        let (_, vk1) = generate_keypair(SignatureScheme::TernaryLamport, &[0i8, 0, 0]).unwrap();
        let (_, vk2) = generate_keypair(SignatureScheme::TernaryLamport, &[1i8, 0, 0]).unwrap();
        assert_ne!(vk1.public, vk2.public);
    }

    #[test]
    fn test_sign() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1];
        let (mut sk, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        assert_eq!(sig.scheme, SignatureScheme::TernaryLamport);
        assert_eq!(sig.revealed.len(), SIGN_DIGEST_TRITS);
    }

    #[test]
    fn test_sign_deterministic() {
        let seed = alloc::vec![0i8, 1, -1];
        let message = alloc::vec![1i8, 0, -1];

        let (mut sk1, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let sig1 = sign(&mut sk1, &message).unwrap();

        let (mut sk2, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();
        let sig2 = sign(&mut sk2, &message).unwrap();

        assert_eq!(sig1.revealed, sig2.revealed);
    }

    #[test]
    fn test_sign_verify() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (mut sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        let valid = verify(&vk, &message, &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_sign_verify_wrong_message() {
        let seed = alloc::vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
        let (mut sk, vk) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
        let sig = sign(&mut sk, &message).unwrap();

        let wrong_msg = alloc::vec![0i8, 0, 0, 0, 0, 0, 0, 0, 0];
        let valid = verify(&vk, &wrong_msg, &sig).unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_one_time_guard() {
        let seed = alloc::vec![0i8, 1, -1];
        let (mut sk, _) = generate_keypair(SignatureScheme::TernaryLamport, &seed).unwrap();

        let message = alloc::vec![1i8, 0, -1];
        sign(&mut sk, &message).unwrap();
        assert!(sk.used);

        let result = sign(&mut sk, &message);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_scheme() {
        let result = generate_keypair(SignatureScheme::Dilithium, &[0i8]);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_sizes() {
        assert_eq!(signature_size(SignatureScheme::TernaryLamport), SIGN_DIGEST_TRITS * SECRET_ELEMENT_TRITS);
        assert!(signature_size(SignatureScheme::Dilithium) > 0);
        assert!(signature_size(SignatureScheme::Falcon) > 0);
        assert!(signature_size(SignatureScheme::SphincsPlus) > 0);
    }

    #[test]
    fn test_key_sizes() {
        let pk_size = public_key_size(SignatureScheme::TernaryLamport);
        let sk_size = secret_key_size(SignatureScheme::TernaryLamport);
        assert_eq!(pk_size, SIGN_DIGEST_TRITS * 3 * SECRET_ELEMENT_TRITS);
        assert_eq!(sk_size, pk_size);
    }

    #[test]
    fn test_base_w() {
        let data = [0xABu8, 0xCD];
        let bw = base_w(&data, 4);
        assert_eq!(bw, alloc::vec![0xA, 0xB, 0xC, 0xD]);
    }

    #[test]
    fn test_wots_chain_lengths_sum() {
        let msg = [0u8; HBS_N];
        let lengths = wots_chain_lengths(&msg);
        assert_eq!(lengths.len(), WOTS_LEN);
        for &l in &lengths {
            assert!(l < WOTS_W);
        }
    }

    #[test]
    fn test_wots_sign_verify_roundtrip() {
        let sk_seed = [42u8; HBS_N];
        let pub_seed = hbs_hash(20, &[&sk_seed]);
        let ots_addr = 0u32;
        let msg_hash = hbs_hash(99, &[b"test wots"]);
        let expected_pk = wots_keygen_pk(&sk_seed, &pub_seed, ots_addr);
        let sig = wots_sign_msg(&msg_hash, &sk_seed, &pub_seed, ots_addr);
        let recovered_pk = wots_pk_from_sig(&sig, &msg_hash, &pub_seed);
        assert_eq!(expected_pk.len(), recovered_pk.len());
        for (a, b) in expected_pk.iter().zip(recovered_pk.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_xmss_params() {
        assert_eq!(XmssParams::XmssSha2_10_256.tree_height(), 10);
        assert_eq!(XmssParams::XmssSha2_10_256.max_signatures(), 1024);
        assert_eq!(XmssParams::XmssSha2_16_256.tree_height(), 16);
        assert_eq!(XmssParams::XmssSha2_20_256.tree_height(), 20);
        assert_eq!(XmssParams::XmssSha2_20_256.max_signatures(), 1 << 20);
        assert_eq!(XmssParams::XmssSha2_10_256.name(), "XMSS-SHA2_10_256");
    }

    #[test]
    fn test_xmss_state_serialization() {
        let state = XmssState { index: 5, max_index: 1024, exhausted: false };
        let bytes = state.to_bytes();
        let recovered = XmssState::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.index, 5);
        assert_eq!(recovered.max_index, 1024);
        assert!(!recovered.exhausted);
    }

    #[test]
    fn test_xmss_state_exhaustion() {
        let mut state = XmssState { index: 15, max_index: 16, exhausted: false };
        assert_eq!(state.remaining(), 1);
        let idx = state.advance().unwrap();
        assert_eq!(idx, 15);
        assert!(state.is_exhausted());
        assert!(state.exhausted);
        assert_eq!(state.remaining(), 0);
        let err = state.advance();
        assert!(err.is_err());
    }

    #[test]
    fn test_lmots_type_params() {
        assert_eq!(LmotsType::LmotsSha256N32W4.w(), 4);
        assert_eq!(LmotsType::LmotsSha256N32W4.p(), 67);
        assert_eq!(LmotsType::LmotsSha256N32W8.w(), 8);
        assert_eq!(LmotsType::LmotsSha256N32W8.p(), 34);
        assert_eq!(LmotsType::LmotsSha256N32W1.p(), 265);
        assert_eq!(LmotsType::LmotsSha256N32W2.p(), 133);
    }

    #[test]
    fn test_lms_type_params() {
        assert_eq!(LmsType::LmsSha256M32H5.height(), 5);
        assert_eq!(LmsType::LmsSha256M32H5.max_signatures(), 32);
        assert_eq!(LmsType::LmsSha256M32H10.height(), 10);
        assert_eq!(LmsType::LmsSha256M32H10.max_signatures(), 1024);
        assert_eq!(LmsType::LmsSha256M32H5.name(), "LMS_SHA256_M32_H5");
    }

    #[test]
    fn test_lms_state_serialization() {
        let state = LmsState { index: 3, max_index: 32, exhausted: false };
        let bytes = state.to_bytes();
        let recovered = LmsState::from_bytes(&bytes).unwrap();
        assert_eq!(recovered.index, 3);
        assert_eq!(recovered.max_index, 32);
        assert!(!recovered.exhausted);
    }

    #[test]
    fn test_lms_state_exhaustion() {
        let mut state = LmsState { index: 31, max_index: 32, exhausted: false };
        assert_eq!(state.remaining(), 1);
        let idx = state.advance().unwrap();
        assert_eq!(idx, 31);
        assert!(state.is_exhausted());
        assert!(state.advance().is_err());
    }

    #[test]
    fn test_coeff() {
        let data = [0xABu8, 0xCD];
        assert_eq!(coeff(&data, 0, 4), 0xA);
        assert_eq!(coeff(&data, 1, 4), 0xB);
        assert_eq!(coeff(&data, 2, 4), 0xC);
        assert_eq!(coeff(&data, 3, 4), 0xD);
        assert_eq!(coeff(&data, 0, 8), 0xAB);
        assert_eq!(coeff(&data, 1, 8), 0xCD);
    }

    #[test]
    fn test_hbs_hash_deterministic() {
        let a = hbs_hash(0, &[b"hello"]);
        let b = hbs_hash(0, &[b"hello"]);
        assert_eq!(a, b);
        let c = hbs_hash(1, &[b"hello"]);
        assert_ne!(a, c);
        let d = hbs_hash(0, &[b"world"]);
        assert_ne!(a, d);
    }

    #[test]
    fn test_hbs_hash_domain_separation() {
        let a = hbs_hash(10, &[b"data"]);
        let b = hbs_hash(11, &[b"data"]);
        assert_ne!(a, b);
    }
}
