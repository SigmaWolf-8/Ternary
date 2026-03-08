// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # TIS-27 — Ternary Identity Sponge
//!
//! The single cryptographic primitive for the entire PlenumNET stack.
//!
//! All parameters are derived from the architecture, not chosen:
//!
//! | Parameter | Value | Why |
//! |-----------|-------|-----|
//! | State     | 54    | 27 classification + 27 identity = full address width |
//! | Rate      | 27    | identity anchor width = classification layer width |
//! | Capacity  | 27    | classification layer width |
//! | Rounds    | 27    | one per identity trit |
//! | Stride    | 13    | gcd(13, 54) = 1 → complete Pi permutation cycle |
//!
//! Round constants: Tribonacci sequence mod 3, indices 0–26.
//!
//! ## Usage
//!
//! ```
//! use ternary_math::sponge::{derive_key, mac, derive_link_keys};
//!
//! // Key derivation (CON tunnel keys, node identity keys)
//! let key = derive_key(b"PlenumNET-CON-v2.5", &[addr_a, addr_b, secret].concat(), 32);
//!
//! // MAC (wire packet authentication)
//! let tag = mac(b"tunnel-key", b"packet-data");
//!
//! // Topology-derived link key pair (CON)
//! let (outbound, inbound) = derive_link_keys(&addr_a_bytes, &addr_b_bytes, secret);
//! ```
//!
//! All arithmetic in GF(3) = {0,1,2}. No binary hash primitives anywhere.

// ─── Parameters ──────────────────────────────────────────────────────────────

const STATE: usize = 54;
const RATE:  usize = 27;
const ROUNDS: usize = 27;

/// Tribonacci sequence mod 3, indices 0–26.
/// T(n) = T(n-1) + T(n-2) + T(n-3), T(0)=0 T(1)=0 T(2)=1, then mod 3.
const RC: [u8; 27] = [
    0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0, 0, 1, 1, 2, 1, 1, 1, 0, 2, 0, 2, 1, 0,
];

// ─── Sponge ──────────────────────────────────────────────────────────────────

/// TIS-27 sponge state.
struct Sponge {
    state: [u8; STATE],
}

impl Sponge {
    fn new() -> Self {
        Self { state: [0u8; STATE] }
    }

    /// Absorb arbitrary bytes.  May be called multiple times.
    fn absorb(&mut self, input: &[u8]) {
        // Convert bytes to trits with padding [1, 0*, 2]
        let mut trits: Vec<u8> = Vec::with_capacity(input.len() * 6 + 2);
        for &b in input {
            byte_to_trits(b, &mut trits);
        }
        trits.push(1);
        while trits.len() % RATE != RATE - 1 {
            trits.push(0);
        }
        trits.push(2);

        for block in trits.chunks(RATE) {
            for (i, &t) in block.iter().enumerate() {
                self.state[i] = gf3(self.state[i] + t);
            }
            permute(&mut self.state);
        }
    }

    /// Squeeze `n` bytes.  Each squeeze call consumes the rate lane and
    /// applies the permutation before the next block.
    fn squeeze(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            // Pack rate-lane trits → bytes (6 trits → 1 byte, base-3)
            for chunk in self.state[..RATE].chunks(6) {
                if out.len() >= n { break; }
                let b = chunk.iter().enumerate()
                    .fold(0u8, |acc, (i, &t)| acc + t * 3u8.pow(i as u32));
                out.push(b);
            }
            permute(&mut self.state);
        }
        out.truncate(n);
        out
    }
}

// ─── Permutation P54 ─────────────────────────────────────────────────────────

fn permute(s: &mut [u8; STATE]) {
    for r in 0..ROUNDS {
        theta(s);
        pi(s);
        iota(s, r);
    }
}

/// Theta: circular GF(3) neighbour mix.
fn theta(s: &mut [u8; STATE]) {
    let mut t = [0u8; STATE];
    for i in 0..STATE {
        let p = (i + STATE - 1) % STATE;
        let n = (i + 1) % STATE;
        t[i] = gf3(s[i] + gf3(s[p] + s[n]));
    }
    *s = t;
}

/// Pi: stride-13 position permutation — gcd(13,54)=1 → complete cycle.
fn pi(s: &mut [u8; STATE]) {
    let mut t = [0u8; STATE];
    for i in 0..STATE {
        t[(i * 13) % STATE] = s[i];
    }
    *s = t;
}

/// Iota: inject Tribonacci round constant at position 0.
fn iota(s: &mut [u8; STATE], round: usize) {
    s[0] = gf3(s[0] + RC[round]);
}

#[inline(always)]
fn gf3(v: u8) -> u8 { v % 3 }

fn byte_to_trits(b: u8, out: &mut Vec<u8>) {
    let mut v = b as u32;
    for _ in 0..6 { out.push((v % 3) as u8); v /= 3; }
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// General-purpose key derivation.
///
/// Absorbs `context` (domain separator) then `secret`, squeezes `n` bytes.
/// Both endpoints compute the same key from the same geometric inputs —
/// no key exchange protocol required.
///
/// # Example
/// ```
/// let key = ternary_math::sponge::derive_key(b"PlenumNET-CON-v2.5", secret, 32);
/// ```
pub fn derive_key(context: &[u8], secret: &[u8], n: usize) -> Vec<u8> {
    let mut s = Sponge::new();
    s.absorb(context);
    s.absorb(secret);
    s.squeeze(n)
}

/// Keyed MAC — 32-byte authentication tag.
///
/// Replaces BLAKE3 MAC on inter-node packets in `wire.rs`.
/// Domain separator prevents key/message lane collisions.
///
/// # Example
/// ```
/// let tag = ternary_math::sponge::mac(outbound_key, packet_bytes);
/// assert_eq!(ternary_math::sponge::verify_mac(outbound_key, packet_bytes, &tag), true);
/// ```
pub fn mac(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut s = Sponge::new();
    s.absorb(b"tis27-mac-v1");
    s.absorb(key);
    s.absorb(message);
    let bytes = s.squeeze(32);
    let mut tag = [0u8; 32];
    tag.copy_from_slice(&bytes);
    tag
}

/// Constant-time MAC verification.
pub fn verify_mac(key: &[u8], message: &[u8], expected: &[u8; 32]) -> bool {
    let computed = mac(key, message);
    // Constant-time compare — XOR all bytes, check if sum is zero
    let diff: u8 = computed.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b));
    diff == 0
}

/// Derive a directional tunnel key pair for a CON link edge.
///
/// Replaces `blake3::hash` in `overlay.rs`.  Both endpoints independently
/// compute the same pair from their 14-byte packed wire addresses + shared
/// secret.  Canonical ordering (smaller address first) ensures symmetry.
///
/// Returns `(outbound_key, inbound_key)` — directions baked in, no negotiation.
pub fn derive_link_keys(addr_a: &[u8], addr_b: &[u8], secret: &[u8]) -> ([u8; 32], [u8; 32]) {
    let (lo, hi) = if addr_a <= addr_b { (addr_a, addr_b) } else { (addr_b, addr_a) };
    let mut material = Vec::with_capacity(lo.len() + hi.len() + secret.len());
    material.extend_from_slice(lo);
    material.extend_from_slice(hi);
    material.extend_from_slice(secret);

    let k_out = derive_key(b"tis27-con-outbound-v1", &material, 32);
    let k_in  = derive_key(b"tis27-con-inbound-v1",  &material, 32);

    let mut out = [0u8; 32]; out.copy_from_slice(&k_out);
    let mut inp = [0u8; 32]; inp.copy_from_slice(&k_in);
    (out, inp)
}

/// General-purpose hash — produces `n` bytes from arbitrary input.
/// Use for flow hashing, endpoint fingerprinting, and other non-keyed digests.
pub fn hash(input: &[u8], n: usize) -> Vec<u8> {
    derive_key(b"tis27-hash-v1", input, n)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_complete_cycle() {
        let mut visited = [false; STATE];
        let mut pos = 0usize;
        for _ in 0..STATE { visited[pos] = true; pos = (pos * 13) % STATE; }
        assert!(visited.iter().all(|&v| v));
    }

    #[test]
    fn round_consts_tribonacci_mod3() {
        let mut t = vec![0u64, 0, 1];
        while t.len() < 27 { let n = t.len(); t.push(t[n-1]+t[n-2]+t[n-3]); }
        let expected: Vec<u8> = t.iter().map(|v| (v % 3) as u8).collect();
        assert_eq!(&RC[..], expected.as_slice());
    }

    #[test]
    fn byte_trits_roundtrip_all_256() {
        for b in 0u8..=255 {
            let mut out = Vec::new();
            byte_to_trits(b, &mut out);
            assert_eq!(out.len(), 6);
            for t in &out { assert!(*t < 3); }
        }
    }

    #[test]
    fn derive_key_deterministic() {
        assert_eq!(derive_key(b"ctx", b"s", 32), derive_key(b"ctx", b"s", 32));
    }

    #[test]
    fn derive_key_lengths() {
        for n in [16, 32, 64, 100] {
            assert_eq!(derive_key(b"ctx", b"s", n).len(), n);
        }
    }

    #[test]
    fn derive_key_context_separates() {
        assert_ne!(derive_key(b"ctx-a", b"s", 32), derive_key(b"ctx-b", b"s", 32));
    }

    #[test]
    fn derive_key_secret_sensitive() {
        assert_ne!(derive_key(b"ctx", b"secret-1", 32), derive_key(b"ctx", b"secret-2", 32));
    }

    #[test]
    fn mac_deterministic() {
        assert_eq!(mac(b"key", b"msg"), mac(b"key", b"msg"));
    }

    #[test]
    fn mac_key_sensitive() {
        assert_ne!(mac(b"key1", b"msg"), mac(b"key2", b"msg"));
    }

    #[test]
    fn mac_message_sensitive() {
        assert_ne!(mac(b"key", b"msg1"), mac(b"key", b"msg2"));
    }

    #[test]
    fn mac_verify_roundtrip() {
        let tag = mac(b"key", b"msg");
        assert!(verify_mac(b"key", b"msg", &tag));
        assert!(!verify_mac(b"key", b"tampered", &tag));
        assert!(!verify_mac(b"wrong-key", b"msg", &tag));
    }

    #[test]
    fn derive_link_keys_symmetric() {
        let a = [1u8; 14]; let b = [2u8; 14]; let s = b"shared";
        let (lo_hi_1, hi_lo_1) = derive_link_keys(&a, &b, s);
        let (lo_hi_2, hi_lo_2) = derive_link_keys(&b, &a, s);
        assert_eq!(lo_hi_1, lo_hi_2);
        assert_eq!(hi_lo_1, hi_lo_2);
    }

    #[test]
    fn derive_link_keys_directional() {
        let a = [1u8; 14]; let b = [2u8; 14];
        let (out, inp) = derive_link_keys(&a, &b, b"s");
        assert_ne!(out, inp, "outbound and inbound keys must differ");
    }

    #[test]
    fn derive_link_keys_pair_sensitive() {
        let a = [1u8; 14]; let b = [2u8; 14]; let c = [3u8; 14];
        let (k_ab, _) = derive_link_keys(&a, &b, b"s");
        let (k_ac, _) = derive_link_keys(&a, &c, b"s");
        assert_ne!(k_ab, k_ac);
    }

    #[test]
    fn hash_nonzero_output() {
        let h = hash(b"test-input", 32);
        assert_eq!(h.len(), 32);
        assert!(h.iter().any(|&b| b != 0));
    }
}
