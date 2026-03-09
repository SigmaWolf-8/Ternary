// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TIS Sponge — SIMD GF(3), zero heap, zero waste in hot path.
// All buffers allocated once. CPUID checked once. Pi tables static.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct TisParams {
    pub state_width: usize,
    pub rate: usize,
    pub capacity: usize,
    pub stride: usize,
    pub rounds: usize,
    pub pad: usize,
}

pub const TIS27: TisParams = TisParams { state_width: 54, rate: 27, capacity: 27, stride: 13, rounds: 4, pad: 64 };
pub const TIS81: TisParams = TisParams { state_width: 243, rate: 81, capacity: 162, stride: 13, rounds: 4, pad: 256 };

const RC_BASE: [u8; 27] = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

#[inline(always)]
fn gf3_add(a: u8, b: u8) -> u8 { let s = a + b; if s >= 3 { s - 3 } else { s } }

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

// ── Static pi tables ───────────────────────────────────────────────

static PI27: [u8; 54] = {
    let mut t = [0u8; 54];
    let mut i = 0;
    while i < 54 { t[i] = ((i * 13) % 54) as u8; i += 1; }
    t
};

static PI81: [u16; 243] = {
    let mut t = [0u16; 243];
    let mut i = 0;
    while i < 243 { t[i] = ((i * 13) % 243) as u16; i += 1; }
    t
};

// ── Static RC schedules ────────────────────────────────────────────

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

static RC4_81: [[u8; 96]; 4] = {
    let mut rcs = [[0u8; 96]; 4];
    let mut r = 0;
    while r < 4 {
        let mut i = 0;
        while i < 81 {
            let mut x = i + r;
            while x >= 27 { x -= 27; }
            rcs[r][i] = RC_BASE[x];
            i += 1;
        }
        r += 1;
    }
    rcs
};

// ── SIMD theta + pi + rc pass (generic over buffer size) ───────────

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn simd_theta_pass(
    state: &[u8], theta: &mut [u8],
    l13: &mut [u8], l7: &mut [u8], l1: &mut [u8],
    r1: &mut [u8], r7: &mut [u8], r13: &mut [u8],
    w: usize, pad: usize,
) {
    rot(state, l13, 13, w);
    rot(state, l7, 7, w);
    rot(state, l1, 1, w);
    rot(state, r1, w - 1, w);
    rot(state, r7, w - 7, w);
    rot(state, r13, w - 13, w);

    let mut i = 0;
    while i + 16 <= pad {
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
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn simd_rc_add(state: &mut [u8], rc: &[u8], stop: usize) {
    let mut i = 0;
    while i < stop {
        let sv = _mm_loadu_si128(state[i..].as_ptr() as *const _);
        let rv = _mm_loadu_si128(rc[i..].as_ptr() as *const _);
        _mm_storeu_si128(state[i..].as_mut_ptr() as *mut _, smod3(_mm_add_epi8(sv, rv)));
        i += 16;
    }
}

// ── TIS-27 ─────────────────────────────────────────────────────────

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
        simd_theta_pass(&state, &mut theta, &mut l13, &mut l7, &mut l1,
                        &mut r1, &mut r7, &mut r13, 54, 64);
        for i in 0..54 { pibuf[i] = theta[PI27[i] as usize]; }
        simd_rc_add(&mut pibuf, &RC4_27[r], 32);
        state[..54].copy_from_slice(&pibuf[..54]);
        state[54] = 0; state[55] = 0; state[56] = 0; state[57] = 0;
        state[58] = 0; state[59] = 0; state[60] = 0; state[61] = 0;
        state[62] = 0; state[63] = 0;
    }

    let take = std::cmp::min(27, output_len);
    state[..take].to_vec()
}

pub fn tis27_hash(input: &[u8], output_len: usize) -> Vec<u8> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { tis27_hash_simd(input, output_len) };
        }
    }
    tis27_hash_scalar(input, output_len)
}

fn tis27_hash_scalar(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; 54];
    let block = std::cmp::min(27, input.len());
    for i in 0..block { state[i] = input[i]; }
    for r in 0..4 {
        let mut t = [0u8; 54];
        for i in 0..54 {
            let left = { let mut n = state[(i+54-13)%54]+state[(i+54-7)%54]+state[(i+54-1)%54]; if n>=3{n-=3;} if n>=3{n-=3;} n };
            let right = { let mut n = state[(i+1)%54]+state[(i+7)%54]+state[(i+13)%54]; if n>=3{n-=3;} if n>=3{n-=3;} n };
            let mut n = left + state[i] + right; if n>=3{n-=3;} if n>=3{n-=3;} t[i] = n;
        }
        let mut p = [0u8; 54];
        for i in 0..54 { p[i] = t[PI27[i] as usize]; }
        for i in 0..27 { p[i] = gf3_add(p[i], RC4_27[r][i]); }
        state = p;
    }
    state[..std::cmp::min(27, output_len)].to_vec()
}

// ── TIS-81 ─────────────────────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn tis81_hash_simd(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; 256];
    let mut theta = [0u8; 256];
    let mut pibuf = [0u8; 256];
    let mut l13 = [0u8; 256]; let mut l7 = [0u8; 256]; let mut l1 = [0u8; 256];
    let mut r1 = [0u8; 256]; let mut r7 = [0u8; 256]; let mut r13 = [0u8; 256];

    let block = std::cmp::min(81, input.len());
    for i in 0..block { state[i] = input[i]; }

    for r in 0..4 {
        simd_theta_pass(&state, &mut theta, &mut l13, &mut l7, &mut l1,
                        &mut r1, &mut r7, &mut r13, 243, 256);
        for i in 0..243 { pibuf[i] = theta[PI81[i] as usize]; }
        simd_rc_add(&mut pibuf, &RC4_81[r], 96);
        state[..243].copy_from_slice(&pibuf[..243]);
        for i in 243..256 { state[i] = 0; }
    }

    let take = std::cmp::min(81, output_len);
    state[..take].to_vec()
}

pub fn tis81_hash(input: &[u8], output_len: usize) -> Vec<u8> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { tis81_hash_simd(input, output_len) };
        }
    }
    tis81_hash_scalar(input, output_len)
}

fn tis81_hash_scalar(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; 243];
    let block = std::cmp::min(81, input.len());
    for i in 0..block { state[i] = input[i]; }
    for r in 0..4 {
        let mut t = [0u8; 243];
        for i in 0..243 {
            let left = { let mut n = state[(i+243-13)%243]+state[(i+243-7)%243]+state[(i+243-1)%243]; if n>=3{n-=3;} if n>=3{n-=3;} n };
            let right = { let mut n = state[(i+1)%243]+state[(i+7)%243]+state[(i+13)%243]; if n>=3{n-=3;} if n>=3{n-=3;} n };
            let mut n = left + state[i] + right; if n>=3{n-=3;} if n>=3{n-=3;} t[i] = n;
        }
        let mut p = [0u8; 243];
        for i in 0..243 { p[i] = t[PI81[i] as usize]; }
        for i in 0..81 { let mut x = i+r; while x>=27{x-=27;} p[i] = gf3_add(p[i], RC_BASE[x]); }
        state = p;
    }
    state[..std::cmp::min(81, output_len)].to_vec()
}

// ── Key derivation ─────────────────────────────────────────────────

pub fn tis27_derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context); input.extend_from_slice(material);
    tis27_hash(&input, key_len)
}
pub fn tis81_derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context); input.extend_from_slice(material);
    tis81_hash(&input, key_len)
}
pub fn tis_hash(input: &[u8], output_len: usize, params: &TisParams) -> Vec<u8> {
    if params.state_width == 54 { tis27_hash(input, output_len) } else { tis81_hash(input, output_len) }
}
pub fn tis_derive_key(context: &[u8], material: &[u8], key_len: usize, params: &TisParams) -> Vec<u8> {
    if params.state_width == 54 { tis27_derive_key(context, material, key_len) } else { tis81_derive_key(context, material, key_len) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn gcd(mut a: usize, mut b: usize) -> usize { while b != 0 { let t = b; b = a % b; a = t; } a }
    #[test] fn test_coprime() { assert_eq!(gcd(13,54),1); assert_eq!(gcd(13,243),1); assert_eq!(gcd(7,54),1); assert_eq!(gcd(7,243),1); }
    #[test] fn test_pq() { assert!((TIS81.capacity as f64) * 3.0_f64.log2() >= 256.0); }
    #[test] fn test_det27() { let i: Vec<u8>=(0..27).map(|x|(x%3)as u8).collect(); assert_eq!(tis27_hash(&i,27),tis27_hash(&i,27)); }
    #[test] fn test_det81() { let i: Vec<u8>=(0..81).map(|x|(x%3)as u8).collect(); assert_eq!(tis81_hash(&i,81),tis81_hash(&i,81)); }
    #[test] fn test_gf3_27() { for &t in &tis27_hash(&[1,2,0,1,2,0,1],27) { assert!(t<=2); } }
    #[test] fn test_gf3_81() { for &t in &tis81_hash(&[1,2,0],81) { assert!(t<=2); } }
    #[test] fn test_aval27() { let a=vec![0u8;27]; let mut b=a.clone(); b[0]=1;
        let d:usize=tis27_hash(&a,27).iter().zip(tis27_hash(&b,27).iter()).filter(|(&x,&y)|x!=y).count();
        assert!(d>=10,"aval:{}/27",d); }
    #[test] fn test_aval81() { let a=vec![0u8;81]; let mut b=a.clone(); b[0]=1;
        let d:usize=tis81_hash(&a,81).iter().zip(tis81_hash(&b,81).iter()).filter(|(&x,&y)|x!=y).count();
        assert!(d>=30,"aval:{}/81",d); }
}
