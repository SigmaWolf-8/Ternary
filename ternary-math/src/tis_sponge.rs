// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TIS-27 Fast Ternary Integrity Function
// Location: ternary-math/src/tis_sponge.rs
//
// ┌─────────────────────────────────────────────────────────────────┐
// │  THIS IS NOT A CRYPTOGRAPHIC HASH.                              │
// │                                                                 │
// │  TIS-27 provides fast corruption detection for wire packets     │
// │  and data integrity checks on already-authenticated channels.   │
// │  27-trit capacity = 43 bits — insufficient for cryptographic    │
// │  security against a deliberate adversary.                       │
// │                                                                 │
// │  For cryptographic operations (signing, key derivation,         │
// │  identity binding, TDNS registration), use TL-Sponge:           │
// │  src/kernel/src/crypto/sponge.rs (385-bit post-quantum).        │
// └─────────────────────────────────────────────────────────────────┘
//
// Use cases:
//   - Wire packet integrity (fast corruption detection, 303 ns)
//   - Scan hashing on authenticated connections
//   - Data integrity checks where the channel is already secured
//   - Internal consistency verification
//
// NOT for:
//   - TDNS registration or identity binding (use TL-Sponge)
//   - Document signing or notarization (use TL-Sponge)
//   - Key derivation (use TL-Sponge)
//   - Any operation requiring collision resistance against an adversary
//
// Architecture:
//   State: 54 trits (GF(3), unsigned {0,1,2})
//   Rate: 27 trits | Capacity: 27 trits (43 bits — non-cryptographic)
//   Rounds: 4 | Theta: 7-neighbor (±1, ±7, ±13) | Pi: stride-13
//   SIMD: SSE2 on x86_64 | Division-free GF(3) arithmetic

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// ── Division-free GF(3) ────────────────────────────────────────────

#[inline(always)]
fn gf3_add(a: u8, b: u8) -> u8 { let s = a + b; if s >= 3 { s - 3 } else { s } }

const RC_BASE: [u8; 27] = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

// ── Static tables (compile-time, formula-derived) ──────────────────

static PI27: [u8; 54] = {
    let mut t = [0u8; 54];
    let mut i = 0;
    while i < 54 { t[i] = ((i * 13) % 54) as u8; i += 1; }
    t
};

static RC4_27: [[u8; 32]; 4] = {
    let mut rcs = [[0u8; 32]; 4];
    let mut r = 0;
    while r < 4 {
        let mut i = 0;
        while i < 27 {
            let mut x = i + r;
            if x >= 27 { x -= 27; }
            rcs[r][i] = RC_BASE[x];
            i += 1;
        }
        r += 1;
    }
    rcs
};

// ── SIMD helpers ───────────────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn smod3(v: __m128i) -> __m128i {
    let three = _mm_set1_epi8(3);
    let two = _mm_set1_epi8(2);
    let mut r = v;
    r = _mm_sub_epi8(r, _mm_and_si128(_mm_cmpgt_epi8(r, two), three));
    r = _mm_sub_epi8(r, _mm_and_si128(_mm_cmpgt_epi8(r, two), three));
    r
}

#[inline(always)]
fn rot(src: &[u8], dst: &mut [u8], dist: usize, w: usize) {
    dst[..w - dist].copy_from_slice(&src[dist..w]);
    dst[w - dist..w].copy_from_slice(&src[..dist]);
}

// ── TIS-27 SIMD implementation ─────────────────────────────────────

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn tis27_hash_simd(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; 64];
    let mut theta = [0u8; 64];
    let mut pibuf = [0u8; 64];
    let mut l13 = [0u8; 64]; let mut l7 = [0u8; 64]; let mut l1 = [0u8; 64];
    let mut r1 = [0u8; 64]; let mut r7 = [0u8; 64]; let mut r13 = [0u8; 64];

    let block = std::cmp::min(27, input.len());
    for i in 0..block { state[i] = input[i]; }

    for r in 0..4 {
        rot(&state, &mut l13, 13, 54); rot(&state, &mut l7, 7, 54); rot(&state, &mut l1, 1, 54);
        rot(&state, &mut r1, 53, 54); rot(&state, &mut r7, 47, 54); rot(&state, &mut r13, 41, 54);

        let mut i = 0;
        while i < 64 {
            let lg = smod3(_mm_add_epi8(_mm_add_epi8(
                _mm_loadu_si128(r13[i..].as_ptr() as *const _),
                _mm_loadu_si128(r7[i..].as_ptr() as *const _)),
                _mm_loadu_si128(r1[i..].as_ptr() as *const _)));
            let rg = smod3(_mm_add_epi8(_mm_add_epi8(
                _mm_loadu_si128(l1[i..].as_ptr() as *const _),
                _mm_loadu_si128(l7[i..].as_ptr() as *const _)),
                _mm_loadu_si128(l13[i..].as_ptr() as *const _)));
            let c = _mm_loadu_si128(state[i..].as_ptr() as *const _);
            _mm_storeu_si128(theta[i..].as_mut_ptr() as *mut _,
                smod3(_mm_add_epi8(_mm_add_epi8(lg, c), rg)));
            i += 16;
        }

        pibuf = [0u8; 64];
        for i in 0..54 { pibuf[i] = theta[PI27[i] as usize]; }

        // RC addition
        let mut j = 0;
        while j < 32 {
            let sv = _mm_loadu_si128(pibuf[j..].as_ptr() as *const _);
            let rv = _mm_loadu_si128(RC4_27[r][j..].as_ptr() as *const _);
            _mm_storeu_si128(pibuf[j..].as_mut_ptr() as *mut _, smod3(_mm_add_epi8(sv, rv)));
            j += 16;
        }

        state[..54].copy_from_slice(&pibuf[..54]);
        for i in 54..64 { state[i] = 0; }
    }

    let take = std::cmp::min(27, output_len);
    state[..take].to_vec()
}

// ── Scalar fallback ────────────────────────────────────────────────

fn tis27_hash_scalar(input: &[u8], output_len: usize) -> Vec<u8> {
    #[inline(always)]
    fn mod3(mut n: u8) -> u8 { if n >= 3 { n -= 3; } if n >= 3 { n -= 3; } n }

    let mut state = [0u8; 54];
    let block = std::cmp::min(27, input.len());
    for i in 0..block { state[i] = input[i]; }

    for r in 0..4 {
        let mut t = [0u8; 54];
        for i in 0..54 {
            let left = mod3(state[(i+54-13)%54] + state[(i+54-7)%54] + state[(i+54-1)%54]);
            let right = mod3(state[(i+1)%54] + state[(i+7)%54] + state[(i+13)%54]);
            let mut n = left + state[i] + right; if n >= 3 { n -= 3; } if n >= 3 { n -= 3; }
            t[i] = n;
        }
        let mut p = [0u8; 54];
        for i in 0..54 { p[i] = t[PI27[i] as usize]; }
        for i in 0..27 { p[i] = gf3_add(p[i], RC4_27[r][i]); }
        state = p;
    }

    let take = std::cmp::min(27, output_len);
    state[..take].to_vec()
}

// ── Public API ─────────────────────────────────────────────────────

/// Hash input trits (GF(3), values {0,1,2}) into output trits.
///
/// This is a FAST INTEGRITY FUNCTION, not a cryptographic hash.
/// For cryptographic hashing, use `crypto::sponge::sponge_hash`.
pub fn tis27_hash(input: &[u8], output_len: usize) -> Vec<u8> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { tis27_hash_simd(input, output_len) };
        }
    }
    tis27_hash_scalar(input, output_len)
}

/// Derive a key using TIS-27. For wire integrity context only.
///
/// NOT for cryptographic key derivation — use kernel KDF for that.
pub fn tis27_derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context);
    input.extend_from_slice(material);
    tis27_hash(&input, key_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let input: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();
        assert_eq!(tis27_hash(&input, 27), tis27_hash(&input, 27));
    }

    #[test]
    fn test_gf3_range() {
        let input = vec![1u8, 2, 0, 1, 2, 0, 1];
        for &t in &tis27_hash(&input, 27) { assert!(t <= 2); }
    }

    #[test]
    fn test_avalanche() {
        let a = vec![0u8; 27];
        let mut b = a.clone(); b[0] = 1;
        let ha = tis27_hash(&a, 27);
        let hb = tis27_hash(&b, 27);
        let diff: usize = ha.iter().zip(hb.iter()).filter(|(&x, &y)| x != y).count();
        assert!(diff >= 10, "avalanche: {}/27", diff);
    }

    #[test]
    fn test_pi_coprime() {
        fn gcd(mut a: usize, mut b: usize) -> usize { while b != 0 { let t = b; b = a % b; a = t; } a }
        assert_eq!(gcd(13, 54), 1);
        assert_eq!(gcd(7, 54), 1);
    }
}
