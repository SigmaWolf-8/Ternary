// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TLSponge-385 — The Salvi Framework Sponge
// Location: ternary-math/src/tlsponge385.rs
//
// One sponge. One file.
//
// Round function (per round, 9 rounds):
//   1. Chi  — χ(x) = x¹⁷ over GF(27), compile-time 27-entry table
//   2. Theta-Pi-RC — fused 7-neighbor diffusion (±1,±7,±13) +
//                     stride-376 scatter + round constants
//
// This IS the old sponge.rs permutation with three improvements:
//   - Precomputed THETA_IDX eliminates mod arithmetic in scalar fallback
//   - TIS-27 mode (4 rounds) for scan hash / identity / HMAC fast path
//   - Batch API for heartbeat×26 (sequential now, tritsliced when ready)
//
// State: 729 balanced trits (i8, values -1/0/+1)
// Rate: 243 | Capacity: 486 | Security: 385-bit PQ
// Rounds: 9 full (TLSponge-385), 4 fast (TIS-27)

const STATE_SIZE: usize = 729;
const RATE: usize = 243;
const RATE_BULK: usize = 486;
const ROUNDS: usize = 9;
const ROUNDS_TIS: usize = 4;
const LANES: usize = 27;
const CHI_BLOCKS: usize = 243;

pub const MAX_BATCH: usize = 26;
pub const SPONGE_VERSION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundMode {
    Full,
    Tis27,
}
impl RoundMode {
    #[inline(always)]
    pub fn count(self) -> usize {
        match self { Self::Full => ROUNDS, Self::Tis27 => ROUNDS_TIS }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// BALANCED TRIT ARITHMETIC
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

// ═══════════════════════════════════════════════════════════════════════
// COMPILE-TIME TABLES
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
const fn gf3_mul(a: u8, b: u8) -> u8 { (a * b) % 3 }
#[inline(always)]
const fn gf3_add(a: u8, b: u8) -> u8 { (a + b) % 3 }
#[inline(always)]
const fn gf27_mul(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    let c0 = gf3_mul(a[0], b[0]);
    let c1 = gf3_add(gf3_mul(a[0], b[1]), gf3_mul(a[1], b[0]));
    let c2 = gf3_add(gf3_add(gf3_mul(a[0], b[2]), gf3_mul(a[1], b[1])), gf3_mul(a[2], b[0]));
    let c3 = gf3_add(gf3_mul(a[1], b[2]), gf3_mul(a[2], b[1]));
    let c4 = gf3_mul(a[2], b[2]);
    [gf3_add(c0, gf3_mul(2, c3)), gf3_add(gf3_add(c1, c3), gf3_mul(2, c4)), gf3_add(c2, c4)]
}
#[inline(always)]
const fn gf27_pow17(x: [u8; 3]) -> [u8; 3] {
    let x2 = gf27_mul(x, x); let x4 = gf27_mul(x2, x2);
    let x8 = gf27_mul(x4, x4); let x16 = gf27_mul(x8, x8);
    gf27_mul(x16, x)
}

// Chi table: balanced trit output. Index = (t0+1) + (t1+1)*3 + (t2+1)*9
static CHI_MAP: [[i8; 3]; 27] = {
    let mut map = [[0i8; 3]; 27];
    let mut idx = 0usize;
    while idx < 27 {
        let [r0, r1, r2] = gf27_pow17([(idx % 3) as u8, ((idx / 3) % 3) as u8, (idx / 9) as u8]);
        map[idx] = [r0 as i8 - 1, r1 as i8 - 1, r2 as i8 - 1];
        idx += 1;
    }
    map
};

// SoA for SIMD chi (padded to 32 for AVX2 vpshufb)
static CHI_MAP_T0: [i8; 32] = { let mut t = [0i8; 32]; let mut i = 0; while i < 27 { t[i] = CHI_MAP[i][0]; i += 1; } t };
static CHI_MAP_T1: [i8; 32] = { let mut t = [0i8; 32]; let mut i = 0; while i < 27 { t[i] = CHI_MAP[i][1]; i += 1; } t };
static CHI_MAP_T2: [i8; 32] = { let mut t = [0i8; 32]; let mut i = 0; while i < 27 { t[i] = CHI_MAP[i][2]; i += 1; } t };

// Pi permutation: π(i) = (376*i + 1) mod 729
static PERM: [u16; STATE_SIZE] = {
    let mut p = [0u16; STATE_SIZE];
    let mut i = 0usize;
    while i < STATE_SIZE { p[i] = ((i * 376 + 1) % STATE_SIZE) as u16; i += 1; }
    p
};

// Round constants: (7*round + 13*lane + 3) mod 3 - 1
static RC_TABLE: [[i8; LANES]; ROUNDS] = {
    let mut rc = [[0i8; LANES]; ROUNDS];
    let mut r = 0usize;
    while r < ROUNDS {
        let mut lane = 0usize;
        while lane < LANES { rc[r][lane] = ((r * 7 + lane * 13 + 3) % 3) as i8 - 1; lane += 1; }
        r += 1;
    }
    rc
};

// Precomputed theta neighbor indices — eliminates mod in scalar path
#[derive(Copy, Clone)]
struct ThetaNeighbors { left: [u16; 3], right: [u16; 3] }
static THETA_IDX: [ThetaNeighbors; STATE_SIZE] = {
    let mut t = [ThetaNeighbors { left: [0; 3], right: [0; 3] }; STATE_SIZE];
    let w = STATE_SIZE;
    let mut i = 0;
    while i < w {
        t[i] = ThetaNeighbors {
            left: [((i+w-13)%w) as u16, ((i+w-7)%w) as u16, ((i+w-1)%w) as u16],
            right: [((i+1)%w) as u16, ((i+7)%w) as u16, ((i+13)%w) as u16],
        };
        i += 1;
    }
    t
};

// ═══════════════════════════════════════════════════════════════════════
// CHI LAYER — scalar + AVX2 + NEON
// ═══════════════════════════════════════════════════════════════════════

fn chi_layer(state: &mut [i8; STATE_SIZE]) {
    let mut block = 0;
    while block < STATE_SIZE {
        let idx = (state[block]+1) as usize + (state[block+1]+1) as usize * 3 + (state[block+2]+1) as usize * 9;
        let r = CHI_MAP[idx];
        state[block]=r[0]; state[block+1]=r[1]; state[block+2]=r[2];
        block += 3;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn chi_layer_avx2(state: &mut [i8; STATE_SIZE]) {
    use std::arch::x86_64::*;
    let t0_lo = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T0.as_ptr() as *const __m128i));
    let t0_hi = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T0.as_ptr().add(16) as *const __m128i));
    let t1_lo = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T1.as_ptr() as *const __m128i));
    let t1_hi = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T1.as_ptr().add(16) as *const __m128i));
    let t2_lo = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T2.as_ptr() as *const __m128i));
    let t2_hi = _mm256_broadcastsi128_si256(_mm_loadu_si128(CHI_MAP_T2.as_ptr().add(16) as *const __m128i));
    let v16 = _mm256_set1_epi8(16); let v15 = _mm256_set1_epi8(15);
    let mut indices = [0u8; CHI_BLOCKS];
    for b in 0..CHI_BLOCKS { let base=b*3; indices[b]=((state[base]+1) as u8)+((state[base+1]+1) as u8)*3+((state[base+2]+1) as u8)*9; }
    let mut i = 0;
    while i+32 <= CHI_BLOCKS {
        let iv = _mm256_loadu_si256(indices.as_ptr().add(i) as *const __m256i);
        let ih = _mm256_sub_epi8(iv, v16);
        let mh = _mm256_cmpgt_epi8(iv, v15);
        let r0 = _mm256_blendv_epi8(_mm256_shuffle_epi8(t0_lo,iv), _mm256_shuffle_epi8(t0_hi,ih), mh);
        let r1 = _mm256_blendv_epi8(_mm256_shuffle_epi8(t1_lo,iv), _mm256_shuffle_epi8(t1_hi,ih), mh);
        let r2 = _mm256_blendv_epi8(_mm256_shuffle_epi8(t2_lo,iv), _mm256_shuffle_epi8(t2_hi,ih), mh);
        let mut o0=[0i8;32]; let mut o1=[0i8;32]; let mut o2=[0i8;32];
        _mm256_storeu_si256(o0.as_mut_ptr() as *mut __m256i, r0);
        _mm256_storeu_si256(o1.as_mut_ptr() as *mut __m256i, r1);
        _mm256_storeu_si256(o2.as_mut_ptr() as *mut __m256i, r2);
        for j in 0..32 { let b=(i+j)*3; state[b]=o0[j]; state[b+1]=o1[j]; state[b+2]=o2[j]; }
        i+=32;
    }
    while i < CHI_BLOCKS { let b=i*3; let idx=indices[i] as usize; let r=CHI_MAP[idx]; state[b]=r[0]; state[b+1]=r[1]; state[b+2]=r[2]; i+=1; }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn chi_layer_neon(state: &mut [i8; STATE_SIZE]) {
    use std::arch::aarch64::*;
    let mt0 = vld1q_s8(CHI_MAP_T0.as_ptr()); let mt0h = vld1q_s8(CHI_MAP_T0.as_ptr().add(16));
    let mt1 = vld1q_s8(CHI_MAP_T1.as_ptr()); let mt1h = vld1q_s8(CHI_MAP_T1.as_ptr().add(16));
    let mt2 = vld1q_s8(CHI_MAP_T2.as_ptr()); let mt2h = vld1q_s8(CHI_MAP_T2.as_ptr().add(16));
    let mut indices = [0u8; CHI_BLOCKS];
    for b in 0..CHI_BLOCKS { let base=b*3; indices[b]=((state[base]+1) as u8)+((state[base+1]+1) as u8)*3+((state[base+2]+1) as u8)*9; }
    let v16 = vdupq_n_u8(16);
    let mut i = 0;
    while i+16 <= CHI_BLOCKS {
        let iv = vld1q_u8(indices.as_ptr().add(i));
        let lm = vcltq_u8(iv, v16); let hi = vsubq_u8(iv, v16);
        let r0 = vbslq_s8(lm, vqtbl1q_s8(mt0, vreinterpretq_u8_s8(vreinterpretq_s8_u8(iv))), vqtbl1q_s8(mt0h, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi))));
        let r1 = vbslq_s8(lm, vqtbl1q_s8(mt1, vreinterpretq_u8_s8(vreinterpretq_s8_u8(iv))), vqtbl1q_s8(mt1h, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi))));
        let r2 = vbslq_s8(lm, vqtbl1q_s8(mt2, vreinterpretq_u8_s8(vreinterpretq_s8_u8(iv))), vqtbl1q_s8(mt2h, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi))));
        let mut o0=[0i8;16]; let mut o1=[0i8;16]; let mut o2=[0i8;16];
        vst1q_s8(o0.as_mut_ptr(),r0); vst1q_s8(o1.as_mut_ptr(),r1); vst1q_s8(o2.as_mut_ptr(),r2);
        for j in 0..16 { let b=(i+j)*3; state[b]=o0[j]; state[b+1]=o1[j]; state[b+2]=o2[j]; }
        i+=16;
    }
    while i < CHI_BLOCKS { let b=i*3; let idx=indices[i] as usize; let r=CHI_MAP[idx]; state[b]=r[0]; state[b+1]=r[1]; state[b+2]=r[2]; i+=1; }
}

// ═══════════════════════════════════════════════════════════════════════
// FUSED THETA-PI-RC — scalar (THETA_IDX) + AVX2 + NEON
// ═══════════════════════════════════════════════════════════════════════

fn theta_pi_rc(state: &mut [i8; STATE_SIZE], buf: &mut [i8; STATE_SIZE], round: usize) {
    for i in 0..STATE_SIZE {
        let n = &THETA_IDX[i];
        let left = balanced_wrap(state[n.left[0] as usize] + state[n.left[1] as usize] + state[n.left[2] as usize]);
        let right = balanced_wrap(state[n.right[0] as usize] + state[n.right[1] as usize] + state[n.right[2] as usize]);
        buf[i] = balanced_wrap(left + state[i] + right + 1);
    }
    for i in 0..STATE_SIZE { state[PERM[i] as usize] = buf[i]; }
    let rc = &RC_TABLE[round];
    for lane in 0..LANES { let idx = lane*LANES; state[idx] = balanced_wrap(state[idx] + rc[lane]); }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn theta_pi_rc_avx2(state: &mut [i8; STATE_SIZE], ext: &mut [i8; STATE_SIZE+26], buf: &mut [i8; STATE_SIZE], round: usize) {
    use std::arch::x86_64::*;
    let v1=_mm256_set1_epi8(1); let vhi=_mm256_set1_epi8(1); let vlo=_mm256_set1_epi8(-1); let v3=_mm256_set1_epi8(3);
    ext[..13].copy_from_slice(&state[STATE_SIZE-13..]);
    ext[13..13+STATE_SIZE].copy_from_slice(state);
    ext[13+STATE_SIZE..].copy_from_slice(&state[..13]);
    let mut i=0;
    while i+32<=STATE_SIZE {
        let ei=i+13;
        let l13=_mm256_loadu_si256(ext.as_ptr().add(ei-13) as *const __m256i);
        let l7=_mm256_loadu_si256(ext.as_ptr().add(ei-7) as *const __m256i);
        let l1=_mm256_loadu_si256(ext.as_ptr().add(ei-1) as *const __m256i);
        let ls=_mm256_add_epi8(_mm256_add_epi8(l13,l7),l1);
        let lw=_mm256_blendv_epi8(_mm256_blendv_epi8(ls,_mm256_sub_epi8(ls,v3),_mm256_cmpgt_epi8(ls,vhi)),_mm256_add_epi8(ls,v3),_mm256_cmpgt_epi8(vlo,ls));
        let r1v=_mm256_loadu_si256(ext.as_ptr().add(ei+1) as *const __m256i);
        let r7=_mm256_loadu_si256(ext.as_ptr().add(ei+7) as *const __m256i);
        let r13=_mm256_loadu_si256(ext.as_ptr().add(ei+13) as *const __m256i);
        let rs=_mm256_add_epi8(_mm256_add_epi8(r1v,r7),r13);
        let rw=_mm256_blendv_epi8(_mm256_blendv_epi8(rs,_mm256_sub_epi8(rs,v3),_mm256_cmpgt_epi8(rs,vhi)),_mm256_add_epi8(rs,v3),_mm256_cmpgt_epi8(vlo,rs));
        let c=_mm256_loadu_si256(ext.as_ptr().add(ei) as *const __m256i);
        let t=_mm256_add_epi8(_mm256_add_epi8(_mm256_add_epi8(lw,c),rw),v1);
        let res=_mm256_blendv_epi8(_mm256_blendv_epi8(t,_mm256_sub_epi8(t,v3),_mm256_cmpgt_epi8(t,vhi)),_mm256_add_epi8(t,v3),_mm256_cmpgt_epi8(vlo,t));
        _mm256_storeu_si256(buf.as_mut_ptr().add(i) as *mut __m256i, res);
        i+=32;
    }
    while i<STATE_SIZE { let ei=i+13; let left=balanced_wrap(ext[ei-13]+ext[ei-7]+ext[ei-1]); let right=balanced_wrap(ext[ei+1]+ext[ei+7]+ext[ei+13]); buf[i]=balanced_wrap(left+ext[ei]+right+1); i+=1; }
    for i in 0..STATE_SIZE { state[PERM[i] as usize]=buf[i]; }
    let rc=&RC_TABLE[round]; for lane in 0..LANES { let idx=lane*LANES; state[idx]=balanced_wrap(state[idx]+rc[lane]); }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn theta_pi_rc_neon(state: &mut [i8; STATE_SIZE], ext: &mut [i8; STATE_SIZE+26], buf: &mut [i8; STATE_SIZE], round: usize) {
    use std::arch::aarch64::*;
    let v1=vdupq_n_s8(1); let vhi=vdupq_n_s8(1); let vlo=vdupq_n_s8(-1); let v3=vdupq_n_s8(3); let vn3=vdupq_n_s8(-3);
    ext[..13].copy_from_slice(&state[STATE_SIZE-13..]);
    ext[13..13+STATE_SIZE].copy_from_slice(state);
    ext[13+STATE_SIZE..].copy_from_slice(&state[..13]);
    let mut i=0;
    while i+16<=STATE_SIZE {
        let ei=i+13;
        let l13=vld1q_s8(ext.as_ptr().add(ei-13)); let l7=vld1q_s8(ext.as_ptr().add(ei-7)); let l1=vld1q_s8(ext.as_ptr().add(ei-1));
        let ls=vaddq_s8(vaddq_s8(l13,l7),l1);
        let lw=vbslq_s8(vcltq_s8(ls,vlo),vaddq_s8(ls,v3),vbslq_s8(vcgtq_s8(ls,vhi),vaddq_s8(ls,vn3),ls));
        let r1v=vld1q_s8(ext.as_ptr().add(ei+1)); let r7=vld1q_s8(ext.as_ptr().add(ei+7)); let r13=vld1q_s8(ext.as_ptr().add(ei+13));
        let rs=vaddq_s8(vaddq_s8(r1v,r7),r13);
        let rw=vbslq_s8(vcltq_s8(rs,vlo),vaddq_s8(rs,v3),vbslq_s8(vcgtq_s8(rs,vhi),vaddq_s8(rs,vn3),rs));
        let c=vld1q_s8(ext.as_ptr().add(ei));
        let t=vaddq_s8(vaddq_s8(vaddq_s8(lw,c),rw),v1);
        let res=vbslq_s8(vcltq_s8(t,vlo),vaddq_s8(t,v3),vbslq_s8(vcgtq_s8(t,vhi),vaddq_s8(t,vn3),t));
        vst1q_s8(buf.as_mut_ptr().add(i),res);
        i+=16;
    }
    while i<STATE_SIZE { let ei=i+13; let left=balanced_wrap(ext[ei-13]+ext[ei-7]+ext[ei-1]); let right=balanced_wrap(ext[ei+1]+ext[ei+7]+ext[ei+13]); buf[i]=balanced_wrap(left+ext[ei]+right+1); i+=1; }
    for i in 0..STATE_SIZE { state[PERM[i] as usize]=buf[i]; }
    let rc=&RC_TABLE[round]; for lane in 0..LANES { let idx=lane*LANES; state[idx]=balanced_wrap(state[idx]+rc[lane]); }
}

// ═══════════════════════════════════════════════════════════════════════
// PERMUTATION DISPATCH
// ═══════════════════════════════════════════════════════════════════════

fn permute_n(state: &mut [i8; STATE_SIZE], rounds: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            let mut ext = [0i8; STATE_SIZE + 26];
            let mut buf = [0i8; STATE_SIZE];
            for round in 0..rounds {
                unsafe { chi_layer_avx2(state); theta_pi_rc_avx2(state, &mut ext, &mut buf, round); }
            }
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            let mut ext = [0i8; STATE_SIZE + 26];
            let mut buf = [0i8; STATE_SIZE];
            for round in 0..rounds {
                unsafe { chi_layer_neon(state); theta_pi_rc_neon(state, &mut ext, &mut buf, round); }
            }
            return;
        }
    }
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..rounds { chi_layer(state); theta_pi_rc(state, &mut buf, round); }
}

pub fn sponge_permutation(state: &mut [i8; STATE_SIZE]) { permute_n(state, ROUNDS); }
pub fn sponge_permutation_v1(state: &mut [i8; STATE_SIZE]) { permute_n(state, ROUNDS); }

// ═══════════════════════════════════════════════════════════════════════
// TRIT / BYTE CONVERSION
// ═══════════════════════════════════════════════════════════════════════

pub fn bytes_to_trits_pub(bytes: &[u8]) -> Vec<i8> { bytes_to_trits(bytes) }

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes { let mut v = byte; for _ in 0..5 { trits.push((v%3) as i8 - 1); v/=3; } }
    trits
}

fn trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((trits.len()+4)/5);
    let mut i = 0;
    while i < trits.len() {
        let mut val: u8 = 0; let mut pow: u8 = 1;
        for j in 0..5 { if i+j < trits.len() { val += (trits[i+j]+1) as u8 * pow; } pow = pow.wrapping_mul(3); }
        bytes.push(val); i += 5;
    }
    bytes
}

// ═══════════════════════════════════════════════════════════════════════
// SPONGE STRUCT — N-API compatible (Clone for tree-parallel squeeze)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct Sponge385Pub {
    state: [i8; STATE_SIZE],
    buf: [i8; RATE],
    buf_len: usize,
    absorbed: bool,
    rounds: usize,
}

impl Sponge385Pub {
    pub fn new() -> Self { Self { state: [0i8; STATE_SIZE], buf: [0i8; RATE], buf_len: 0, absorbed: false, rounds: ROUNDS } }
    pub fn new_tis() -> Self { Self { state: [0i8; STATE_SIZE], buf: [0i8; RATE], buf_len: 0, absorbed: false, rounds: ROUNDS_TIS } }
    pub fn new_v1() -> Self { Self::new() }

    fn do_permute(&mut self) { permute_n(&mut self.state, self.rounds); }

    pub fn absorb(&mut self, input: &[i8]) {
        if input.is_empty() { return; }
        self.absorbed = true;
        let mut offset = 0; let input_len = input.len();
        if self.buf_len > 0 {
            let fill = input_len.min(RATE - self.buf_len);
            self.buf[self.buf_len..self.buf_len+fill].copy_from_slice(&input[..fill]);
            self.buf_len += fill; offset = fill;
            if self.buf_len == RATE {
                for i in 0..RATE { self.state[i] = trit_add(self.state[i], self.buf[i]); }
                self.do_permute(); self.buf_len = 0;
            }
        }
        while offset + RATE <= input_len {
            for i in 0..RATE { self.state[i] = trit_add(self.state[i], input[offset+i]); }
            self.do_permute(); offset += RATE;
        }
        let remaining = input_len - offset;
        if remaining > 0 { self.buf[self.buf_len..self.buf_len+remaining].copy_from_slice(&input[offset..]); self.buf_len += remaining; }
    }

    pub fn absorb_bytes(&mut self, input: &[u8]) { self.absorb(&bytes_to_trits(input)); }

    fn finalize_absorb(&mut self) {
        for i in 0..self.buf_len { self.state[i] = trit_add(self.state[i], self.buf[i]); }
        if self.buf_len < RATE { self.state[self.buf_len] = trit_add(self.state[self.buf_len], 1); }
        self.buf_len = 0; self.do_permute();
    }

    pub fn squeeze(&mut self, output_trits: usize) -> Vec<i8> {
        if self.buf_len > 0 || !self.absorbed { self.finalize_absorb(); }
        let mut output = Vec::with_capacity(output_trits);
        while output.len() < output_trits {
            let take = (output_trits - output.len()).min(RATE);
            output.extend_from_slice(&self.state[..take]);
            if output.len() < output_trits { self.do_permute(); }
        }
        output.truncate(output_trits); output
    }

    pub fn squeeze_bulk(&mut self, output_trits: usize) -> Vec<i8> {
        if self.buf_len > 0 || !self.absorbed { self.finalize_absorb(); }
        let mut output = Vec::with_capacity(output_trits);
        while output.len() < output_trits {
            let take = (output_trits - output.len()).min(RATE_BULK);
            output.extend_from_slice(&self.state[..take]);
            if output.len() < output_trits { self.do_permute(); }
        }
        output.truncate(output_trits); output
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════

pub fn hash(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut s = Sponge385Pub::new(); s.absorb_bytes(input);
    trits_to_bytes(&s.squeeze(output_len * 5))[..output_len].to_vec()
}
pub fn hash_hex(input: &[u8]) -> String {
    let mut s = Sponge385Pub::new(); s.absorb_bytes(input);
    trits_to_bytes(&s.squeeze(243))[..49].iter().map(|b| format!("{:02x}", b)).collect()
}
pub fn hash_hex_tis(input: &[u8]) -> String {
    let mut s = Sponge385Pub::new_tis(); s.absorb_bytes(input);
    trits_to_bytes(&s.squeeze(243))[..49].iter().map(|b| format!("{:02x}", b)).collect()
}
pub fn derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context); input.extend_from_slice(material);
    hash(&input, key_len)
}
pub fn derive_key_tis(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut s = Sponge385Pub::new_tis();
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context); input.extend_from_slice(material);
    s.absorb_bytes(&input);
    trits_to_bytes(&s.squeeze(key_len * 5))[..key_len].to_vec()
}
pub fn sponge385_derive_key(domain: &[u8], addr_a: &[u8], addr_b: &[u8], kem_shared_secret: &[u8; 32], epoch: u64) -> Vec<u8> {
    let mut s = Sponge385Pub::new();
    s.absorb_bytes(domain); s.absorb_bytes(addr_a); s.absorb_bytes(addr_b);
    s.absorb_bytes(kem_shared_secret); s.absorb_bytes(&epoch.to_le_bytes());
    trits_to_bytes(&s.squeeze(RATE))[..32].to_vec()
}
pub fn derive_key_batch(domains: &[&[u8]], materials: &[&[u8]], output_len: usize) -> Vec<Vec<u8>> {
    let n = domains.len().min(materials.len()).min(MAX_BATCH);
    (0..n).map(|i| derive_key(domains[i], materials[i], output_len)).collect()
}
pub fn derive_key_batch_tis(domains: &[&[u8]], materials: &[&[u8]], output_len: usize) -> Vec<Vec<u8>> {
    let n = domains.len().min(materials.len()).min(MAX_BATCH);
    (0..n).map(|i| derive_key_tis(domains[i], materials[i], output_len)).collect()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn chi_bijection() {
        let mut seen = [false; 27];
        for i in 0..27 { let o=CHI_MAP[i]; let p=(o[0]+1) as usize+(o[1]+1) as usize*3+(o[2]+1) as usize*9; assert!(!seen[p]); seen[p]=true; }
    }
    #[test] fn chi_zero_fixed() {
        // Index 0 = balanced (-1,-1,-1) = GF(3) (0,0,0). 0^17 = 0. Balanced: [-1,-1,-1].
        assert_eq!(CHI_MAP[0], [-1,-1,-1]);
    }
    #[test] fn perm_full_period() {
        let mut seen = [false; STATE_SIZE];
        for i in 0..STATE_SIZE { let d=PERM[i] as usize; assert!(!seen[d]); seen[d]=true; }
    }
    #[test] fn coprime_neighbors() {
        fn gcd(mut a: usize, mut b: usize) -> usize { while b!=0 { let t=b; b=a%b; a=t; } a }
        assert_eq!(gcd(1,STATE_SIZE),1); assert_eq!(gcd(7,STATE_SIZE),1); assert_eq!(gcd(13,STATE_SIZE),1);
    }
    #[test] fn hash_deterministic() { assert_eq!(hash(b"hello world",32), hash(b"hello world",32)); }
    #[test] fn hash_different() { assert_ne!(hash(b"hello",32), hash(b"world",32)); }
    #[test] fn derive_key_det() { assert_eq!(derive_key(b"c",b"m",32), derive_key(b"c",b"m",32)); }
    #[test] fn derive_key_len() { assert_eq!(derive_key(b"c",b"m",32).len(), 32); }
    #[test] fn derive_key_sep() { assert_ne!(derive_key(b"A",b"m",32), derive_key(b"B",b"m",32)); }
    #[test] fn tis27_different() { assert_ne!(derive_key(b"T",b"m",32), derive_key_tis(b"T",b"m",32)); }
    #[test] fn tis27_det() { assert_eq!(derive_key_tis(b"T",b"m",32), derive_key_tis(b"T",b"m",32)); }
    #[test] fn hash_hex_valid() {
        let h = hash_hex(b"hello"); assert_eq!(h.len(), 98);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
    #[test] fn simd_matches_scalar() {
        let mut a = [0i8; STATE_SIZE]; let mut b = [0i8; STATE_SIZE];
        for i in 0..STATE_SIZE { let v=((i*7+3)%3) as i8 - 1; a[i]=v; b[i]=v; }
        permute_n(&mut a, ROUNDS);
        let mut buf = [0i8; STATE_SIZE];
        for round in 0..ROUNDS { chi_layer(&mut b); theta_pi_rc(&mut b, &mut buf, round); }
        assert_eq!(a, b);
    }
    #[test] fn clone_identical() {
        let mut s1 = Sponge385Pub::new(); s1.absorb_bytes(b"test");
        let mut s2 = s1.clone();
        assert_eq!(s1.squeeze(500), s2.squeeze(500));
    }
    #[test] fn batch_matches() {
        let s0=derive_key(b"D0",b"M0",32); let s1=derive_key(b"D1",b"M1",32);
        let b=derive_key_batch(&[b"D0" as &[u8],b"D1"], &[b"M0" as &[u8],b"M1"], 32);
        assert_eq!(b[0],s0); assert_eq!(b[1],s1);
    }
    #[test] fn batch_empty() { assert!(derive_key_batch(&[],&[],32).is_empty()); }
    #[test] fn constants() {
        assert_eq!(STATE_SIZE,729); assert_eq!(RATE+486,STATE_SIZE);
        assert_eq!(RoundMode::Full.count(),9); assert_eq!(RoundMode::Tis27.count(),4);
    }
}