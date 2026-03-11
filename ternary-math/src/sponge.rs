// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 — Standalone Post-Quantum Sponge Construction (v3 — Chi Layer)
// Location: ternary-math/src/sponge.rs
//
// Mirrors the kernel sponge (src/kernel/src/crypto/sponge.rs) as a
// standalone std-compatible implementation for use by inter-cube and
// other non-kernel crates that need 385-bit PQ security.
//
// AVX2/NEON SIMD permutations ported from the kernel crate with runtime
// feature detection. Scalar fallback for non-SIMD targets.
//
// Sponge version history:
//   v1: 3-neighbor theta, 27 rounds (deprecated)
//   v2: 7-neighbor theta (±1,±7,±13), 9 rounds, no chi
//   v3: Chi layer χ(x) = x¹⁷ over GF(27) added before theta
//
// Round function (v3):
//   1. Chi — χ(x) = x¹⁷ over GF(27) on each 3-trit block (243 blocks)
//   2. Theta — 7-neighbor extended substitution (±1, ±7, ±13)
//   3. Pi — stride-376 fixed permutation
//   4. Round constants
//
// Parameters:
//   State: 729 trits | Rate: 243 | Capacity: 486 (385-bit PQ security)
//   Rounds: 9 (3² — 3× safety margin over 3-round full diffusion)
//   Theta: 7-neighbor extended (±1, ±7, ±13) — all coprime to 729
//   Diffusion: π(i) = (376·i + 1) mod 729 — full-period permutation
//   Round constants: (7·round + 13·lane + 3) mod 3 − 1

const STATE_SIZE: usize = 729;
const RATE: usize = 243;
const ROUNDS: usize = 9;
const LANES: usize = 27;
const CHI_BLOCKS: usize = 243;

pub const SPONGE_VERSION: u8 = 2;

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

#[inline(always)]
const fn gf3_mul(a: u8, b: u8) -> u8 {
    (a * b) % 3
}

#[inline(always)]
const fn gf3_add(a: u8, b: u8) -> u8 {
    (a + b) % 3
}

#[inline(always)]
const fn gf27_mul(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    let c0 = gf3_mul(a[0], b[0]);
    let c1 = gf3_add(gf3_mul(a[0], b[1]), gf3_mul(a[1], b[0]));
    let c2 = gf3_add(gf3_add(gf3_mul(a[0], b[2]), gf3_mul(a[1], b[1])), gf3_mul(a[2], b[0]));
    let c3 = gf3_add(gf3_mul(a[1], b[2]), gf3_mul(a[2], b[1]));
    let c4 = gf3_mul(a[2], b[2]);

    let r0 = gf3_add(c0, gf3_mul(2, c3));
    let r1 = gf3_add(gf3_add(c1, c3), gf3_mul(2, c4));
    let r2 = gf3_add(c2, c4);

    [r0, r1, r2]
}

#[inline(always)]
const fn gf27_pow17(x: [u8; 3]) -> [u8; 3] {
    let x2 = gf27_mul(x, x);
    let x4 = gf27_mul(x2, x2);
    let x8 = gf27_mul(x4, x4);
    let x16 = gf27_mul(x8, x8);
    gf27_mul(x16, x)
}

static CHI_MAP: [[i8; 3]; 27] = {
    let mut map = [[0i8; 3]; 27];
    let mut idx = 0usize;
    while idx < 27 {
        let g0 = (idx % 3) as u8;
        let g1 = ((idx / 3) % 3) as u8;
        let g2 = (idx / 9) as u8;
        let [r0, r1, r2] = gf27_pow17([g0, g1, g2]);
        map[idx] = [r0 as i8 - 1, r1 as i8 - 1, r2 as i8 - 1];
        idx += 1;
    }
    map
};

static CHI_MAP_T0: [i8; 32] = {
    let mut t = [0i8; 32];
    let mut i = 0usize;
    while i < 27 { t[i] = CHI_MAP[i][0]; i += 1; }
    t
};
static CHI_MAP_T1: [i8; 32] = {
    let mut t = [0i8; 32];
    let mut i = 0usize;
    while i < 27 { t[i] = CHI_MAP[i][1]; i += 1; }
    t
};
static CHI_MAP_T2: [i8; 32] = {
    let mut t = [0i8; 32];
    let mut i = 0usize;
    while i < 27 { t[i] = CHI_MAP[i][2]; i += 1; }
    t
};

fn chi_layer(state: &mut [i8; STATE_SIZE]) {
    let mut block = 0;
    while block < STATE_SIZE {
        let idx = (state[block] + 1) as usize
            + (state[block + 1] + 1) as usize * 3
            + (state[block + 2] + 1) as usize * 9;
        let r = CHI_MAP[idx];
        state[block]     = r[0];
        state[block + 1] = r[1];
        state[block + 2] = r[2];
        block += 3;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn chi_layer_avx2(state: &mut [i8; STATE_SIZE]) {
    use std::arch::x86_64::*;

    let t0_lo_128 = _mm_loadu_si128(CHI_MAP_T0.as_ptr() as *const __m128i);
    let t0_hi_128 = _mm_loadu_si128(CHI_MAP_T0.as_ptr().add(16) as *const __m128i);
    let map_t0_lo = _mm256_broadcastsi128_si256(t0_lo_128);
    let map_t0_hi = _mm256_broadcastsi128_si256(t0_hi_128);

    let t1_lo_128 = _mm_loadu_si128(CHI_MAP_T1.as_ptr() as *const __m128i);
    let t1_hi_128 = _mm_loadu_si128(CHI_MAP_T1.as_ptr().add(16) as *const __m128i);
    let map_t1_lo = _mm256_broadcastsi128_si256(t1_lo_128);
    let map_t1_hi = _mm256_broadcastsi128_si256(t1_hi_128);

    let t2_lo_128 = _mm_loadu_si128(CHI_MAP_T2.as_ptr() as *const __m128i);
    let t2_hi_128 = _mm_loadu_si128(CHI_MAP_T2.as_ptr().add(16) as *const __m128i);
    let map_t2_lo = _mm256_broadcastsi128_si256(t2_lo_128);
    let map_t2_hi = _mm256_broadcastsi128_si256(t2_hi_128);

    let v_sixteen = _mm256_set1_epi8(16);
    let v_fifteen = _mm256_set1_epi8(15);

    let mut indices = [0u8; CHI_BLOCKS];
    for b in 0..CHI_BLOCKS {
        let base = b * 3;
        indices[b] = ((state[base] + 1) as u8)
            + ((state[base + 1] + 1) as u8) * 3
            + ((state[base + 2] + 1) as u8) * 9;
    }

    let mut i = 0;
    while i + 32 <= CHI_BLOCKS {
        let idx_vec = _mm256_loadu_si256(indices.as_ptr().add(i) as *const __m256i);
        let idx_hi  = _mm256_sub_epi8(idx_vec, v_sixteen);
        let mask_hi = _mm256_cmpgt_epi8(idx_vec, v_fifteen);

        let lo0 = _mm256_shuffle_epi8(map_t0_lo, idx_vec);
        let hi0 = _mm256_shuffle_epi8(map_t0_hi, idx_hi);
        let r0  = _mm256_blendv_epi8(lo0, hi0, mask_hi);

        let lo1 = _mm256_shuffle_epi8(map_t1_lo, idx_vec);
        let hi1 = _mm256_shuffle_epi8(map_t1_hi, idx_hi);
        let r1  = _mm256_blendv_epi8(lo1, hi1, mask_hi);

        let lo2 = _mm256_shuffle_epi8(map_t2_lo, idx_vec);
        let hi2 = _mm256_shuffle_epi8(map_t2_hi, idx_hi);
        let r2  = _mm256_blendv_epi8(lo2, hi2, mask_hi);

        let mut out0 = [0i8; 32];
        let mut out1 = [0i8; 32];
        let mut out2 = [0i8; 32];
        _mm256_storeu_si256(out0.as_mut_ptr() as *mut __m256i, r0);
        _mm256_storeu_si256(out1.as_mut_ptr() as *mut __m256i, r1);
        _mm256_storeu_si256(out2.as_mut_ptr() as *mut __m256i, r2);

        for j in 0..32 {
            let base = (i + j) * 3;
            state[base]     = out0[j];
            state[base + 1] = out1[j];
            state[base + 2] = out2[j];
        }

        i += 32;
    }

    while i < CHI_BLOCKS {
        let base = i * 3;
        let idx = indices[i] as usize;
        let r = CHI_MAP[idx];
        state[base]     = r[0];
        state[base + 1] = r[1];
        state[base + 2] = r[2];
        i += 1;
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn chi_layer_neon(state: &mut [i8; STATE_SIZE]) {
    use std::arch::aarch64::*;

    let map_t0 = vld1q_s8(CHI_MAP_T0.as_ptr());
    let map_t1 = vld1q_s8(CHI_MAP_T1.as_ptr());
    let map_t2 = vld1q_s8(CHI_MAP_T2.as_ptr());
    let map_t0_hi = vld1q_s8(CHI_MAP_T0.as_ptr().add(16));
    let map_t1_hi = vld1q_s8(CHI_MAP_T1.as_ptr().add(16));
    let map_t2_hi = vld1q_s8(CHI_MAP_T2.as_ptr().add(16));

    let mut indices = [0u8; CHI_BLOCKS];
    for b in 0..CHI_BLOCKS {
        let base = b * 3;
        indices[b] = ((state[base] + 1) as u8)
            + ((state[base + 1] + 1) as u8) * 3
            + ((state[base + 2] + 1) as u8) * 9;
    }

    let v_16 = vdupq_n_u8(16);

    let mut i = 0;
    while i + 16 <= CHI_BLOCKS {
        let idx_vec = vld1q_u8(indices.as_ptr().add(i));

        let lo_mask = vcltq_u8(idx_vec, v_16);
        let hi_idx = vsubq_u8(idx_vec, v_16);

        let r0_lo = vqtbl1q_s8(map_t0, vreinterpretq_u8_s8(vreinterpretq_s8_u8(idx_vec)));
        let r0_hi = vqtbl1q_s8(map_t0_hi, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi_idx)));
        let r0 = vbslq_s8(lo_mask, r0_lo, r0_hi);

        let r1_lo = vqtbl1q_s8(map_t1, vreinterpretq_u8_s8(vreinterpretq_s8_u8(idx_vec)));
        let r1_hi = vqtbl1q_s8(map_t1_hi, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi_idx)));
        let r1 = vbslq_s8(lo_mask, r1_lo, r1_hi);

        let r2_lo = vqtbl1q_s8(map_t2, vreinterpretq_u8_s8(vreinterpretq_s8_u8(idx_vec)));
        let r2_hi = vqtbl1q_s8(map_t2_hi, vreinterpretq_u8_s8(vreinterpretq_s8_u8(hi_idx)));
        let r2 = vbslq_s8(lo_mask, r2_lo, r2_hi);

        let mut out0 = [0i8; 16];
        let mut out1 = [0i8; 16];
        let mut out2 = [0i8; 16];
        vst1q_s8(out0.as_mut_ptr(), r0);
        vst1q_s8(out1.as_mut_ptr(), r1);
        vst1q_s8(out2.as_mut_ptr(), r2);

        for j in 0..16 {
            let base = (i + j) * 3;
            state[base]     = out0[j];
            state[base + 1] = out1[j];
            state[base + 2] = out2[j];
        }

        i += 16;
    }

    while i < CHI_BLOCKS {
        let base = i * 3;
        let idx = indices[i] as usize;
        let r = CHI_MAP[idx];
        state[base]     = r[0];
        state[base + 1] = r[1];
        state[base + 2] = r[2];
        i += 1;
    }
}

static PERM: [u16; STATE_SIZE] = {
    let mut p = [0u16; STATE_SIZE];
    let mut i = 0usize;
    while i < STATE_SIZE {
        p[i] = ((i * 376 + 1) % STATE_SIZE) as u16;
        i += 1;
    }
    p
};

static RC_TABLE: [[i8; LANES]; ROUNDS] = {
    let mut rc = [[0i8; LANES]; ROUNDS];
    let mut r = 0usize;
    while r < ROUNDS {
        let mut lane = 0usize;
        while lane < LANES {
            let val = (r * 7 + lane * 13 + 3) % 3;
            rc[r][lane] = val as i8 - 1;
            lane += 1;
        }
        r += 1;
    }
    rc
};

fn theta_pi_rc(state: &mut [i8; STATE_SIZE], buf: &mut [i8; STATE_SIZE], round: usize) {
    let w = STATE_SIZE;
    for i in 0..w {
        let left = balanced_wrap(
            state[(i + w - 13) % w] +
            state[(i + w - 7) % w] +
            state[(i + w - 1) % w]
        );
        let right = balanced_wrap(
            state[(i + 1) % w] +
            state[(i + 7) % w] +
            state[(i + 13) % w]
        );
        buf[i] = balanced_wrap(left + state[i] + right + 1);
    }

    for i in 0..STATE_SIZE {
        state[PERM[i] as usize] = buf[i];
    }

    let rc = &RC_TABLE[round];
    for lane in 0..LANES {
        let idx = lane * LANES;
        state[idx] = balanced_wrap(state[idx] + rc[lane]);
    }
}

fn sponge_permutation_v2_scalar(state: &mut [i8; STATE_SIZE]) {
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..ROUNDS {
        chi_layer(state);
        theta_pi_rc(state, &mut buf, round);
    }
}

fn sponge_permutation_v1_scalar(state: &mut [i8; STATE_SIZE]) {
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..ROUNDS {
        theta_pi_rc(state, &mut buf, round);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sponge_permutation_v2_avx2(state: &mut [i8; STATE_SIZE]) {
    use std::arch::x86_64::*;

    let mut ext = [0i8; STATE_SIZE + 26];
    let mut buf = [0i8; STATE_SIZE];

    let v_one   = _mm256_set1_epi8(1);
    let v_hi    = _mm256_set1_epi8(1);
    let v_lo    = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    for round in 0..ROUNDS {
        chi_layer_avx2(state);

        ext[..13].copy_from_slice(&state[STATE_SIZE - 13..]);
        ext[13..13 + STATE_SIZE].copy_from_slice(state);
        ext[13 + STATE_SIZE..].copy_from_slice(&state[..13]);

        let mut i = 0;
        while i + 32 <= STATE_SIZE {
            let ei = i + 13;

            let l13 = _mm256_loadu_si256(ext.as_ptr().add(ei - 13) as *const __m256i);
            let l7  = _mm256_loadu_si256(ext.as_ptr().add(ei - 7)  as *const __m256i);
            let l1  = _mm256_loadu_si256(ext.as_ptr().add(ei - 1)  as *const __m256i);
            let lsum = _mm256_add_epi8(_mm256_add_epi8(l13, l7), l1);

            let lgt = _mm256_cmpgt_epi8(lsum, v_hi);
            let llt = _mm256_cmpgt_epi8(v_lo, lsum);
            let lwrap = _mm256_blendv_epi8(lsum, _mm256_sub_epi8(lsum, v_three), lgt);
            let lwrap = _mm256_blendv_epi8(lwrap, _mm256_add_epi8(lsum, v_three), llt);

            let r1  = _mm256_loadu_si256(ext.as_ptr().add(ei + 1)  as *const __m256i);
            let r7  = _mm256_loadu_si256(ext.as_ptr().add(ei + 7)  as *const __m256i);
            let r13 = _mm256_loadu_si256(ext.as_ptr().add(ei + 13) as *const __m256i);
            let rsum = _mm256_add_epi8(_mm256_add_epi8(r1, r7), r13);

            let rgt = _mm256_cmpgt_epi8(rsum, v_hi);
            let rlt = _mm256_cmpgt_epi8(v_lo, rsum);
            let rwrap = _mm256_blendv_epi8(rsum, _mm256_sub_epi8(rsum, v_three), rgt);
            let rwrap = _mm256_blendv_epi8(rwrap, _mm256_add_epi8(rsum, v_three), rlt);

            let center = _mm256_loadu_si256(ext.as_ptr().add(ei) as *const __m256i);
            let total = _mm256_add_epi8(
                _mm256_add_epi8(_mm256_add_epi8(lwrap, center), rwrap),
                v_one,
            );

            let fgt = _mm256_cmpgt_epi8(total, v_hi);
            let flt = _mm256_cmpgt_epi8(v_lo, total);
            let result = _mm256_blendv_epi8(total, _mm256_sub_epi8(total, v_three), fgt);
            let result = _mm256_blendv_epi8(result, _mm256_add_epi8(total, v_three), flt);

            _mm256_storeu_si256(buf.as_mut_ptr().add(i) as *mut __m256i, result);
            i += 32;
        }

        while i < STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
        }

        for i in 0..STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }

        let rc = &RC_TABLE[round];
        for lane in 0..LANES {
            let idx = lane * LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sponge_permutation_v1_avx2(state: &mut [i8; STATE_SIZE]) {
    use std::arch::x86_64::*;

    let mut ext = [0i8; STATE_SIZE + 26];
    let mut buf = [0i8; STATE_SIZE];

    let v_one   = _mm256_set1_epi8(1);
    let v_hi    = _mm256_set1_epi8(1);
    let v_lo    = _mm256_set1_epi8(-1);
    let v_three = _mm256_set1_epi8(3);

    for round in 0..ROUNDS {
        ext[..13].copy_from_slice(&state[STATE_SIZE - 13..]);
        ext[13..13 + STATE_SIZE].copy_from_slice(state);
        ext[13 + STATE_SIZE..].copy_from_slice(&state[..13]);

        let mut i = 0;
        while i + 32 <= STATE_SIZE {
            let ei = i + 13;
            let l13 = _mm256_loadu_si256(ext.as_ptr().add(ei - 13) as *const __m256i);
            let l7  = _mm256_loadu_si256(ext.as_ptr().add(ei - 7)  as *const __m256i);
            let l1  = _mm256_loadu_si256(ext.as_ptr().add(ei - 1)  as *const __m256i);
            let lsum = _mm256_add_epi8(_mm256_add_epi8(l13, l7), l1);
            let lgt = _mm256_cmpgt_epi8(lsum, v_hi);
            let llt = _mm256_cmpgt_epi8(v_lo, lsum);
            let lwrap = _mm256_blendv_epi8(lsum, _mm256_sub_epi8(lsum, v_three), lgt);
            let lwrap = _mm256_blendv_epi8(lwrap, _mm256_add_epi8(lsum, v_three), llt);
            let r1  = _mm256_loadu_si256(ext.as_ptr().add(ei + 1)  as *const __m256i);
            let r7  = _mm256_loadu_si256(ext.as_ptr().add(ei + 7)  as *const __m256i);
            let r13 = _mm256_loadu_si256(ext.as_ptr().add(ei + 13) as *const __m256i);
            let rsum = _mm256_add_epi8(_mm256_add_epi8(r1, r7), r13);
            let rgt = _mm256_cmpgt_epi8(rsum, v_hi);
            let rlt = _mm256_cmpgt_epi8(v_lo, rsum);
            let rwrap = _mm256_blendv_epi8(rsum, _mm256_sub_epi8(rsum, v_three), rgt);
            let rwrap = _mm256_blendv_epi8(rwrap, _mm256_add_epi8(rsum, v_three), rlt);
            let center = _mm256_loadu_si256(ext.as_ptr().add(ei) as *const __m256i);
            let total = _mm256_add_epi8(
                _mm256_add_epi8(_mm256_add_epi8(lwrap, center), rwrap),
                v_one,
            );
            let fgt = _mm256_cmpgt_epi8(total, v_hi);
            let flt = _mm256_cmpgt_epi8(v_lo, total);
            let result = _mm256_blendv_epi8(total, _mm256_sub_epi8(total, v_three), fgt);
            let result = _mm256_blendv_epi8(result, _mm256_add_epi8(total, v_three), flt);
            _mm256_storeu_si256(buf.as_mut_ptr().add(i) as *mut __m256i, result);
            i += 32;
        }
        while i < STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
        }
        for i in 0..STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }
        let rc = &RC_TABLE[round];
        for lane in 0..LANES {
            let idx = lane * LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sponge_permutation_v2_neon(state: &mut [i8; STATE_SIZE]) {
    use std::arch::aarch64::*;

    let mut ext = [0i8; STATE_SIZE + 26];
    let mut buf = [0i8; STATE_SIZE];

    let v_one   = vdupq_n_s8(1);
    let v_hi    = vdupq_n_s8(1);
    let v_lo    = vdupq_n_s8(-1);
    let v_three = vdupq_n_s8(3);
    let v_neg3  = vdupq_n_s8(-3);

    for round in 0..ROUNDS {
        chi_layer_neon(state);

        ext[..13].copy_from_slice(&state[STATE_SIZE - 13..]);
        ext[13..13 + STATE_SIZE].copy_from_slice(state);
        ext[13 + STATE_SIZE..].copy_from_slice(&state[..13]);

        let mut i = 0;
        while i + 16 <= STATE_SIZE {
            let ei = i + 13;

            let l13 = vld1q_s8(ext.as_ptr().add(ei - 13));
            let l7  = vld1q_s8(ext.as_ptr().add(ei - 7));
            let l1  = vld1q_s8(ext.as_ptr().add(ei - 1));
            let lsum = vaddq_s8(vaddq_s8(l13, l7), l1);

            let lgt = vcgtq_s8(lsum, v_hi);
            let llt = vcltq_s8(lsum, v_lo);
            let lwrap = vbslq_s8(lgt, vaddq_s8(lsum, v_neg3), lsum);
            let lwrap = vbslq_s8(llt, vaddq_s8(lsum, v_three), lwrap);

            let r1  = vld1q_s8(ext.as_ptr().add(ei + 1));
            let r7  = vld1q_s8(ext.as_ptr().add(ei + 7));
            let r13 = vld1q_s8(ext.as_ptr().add(ei + 13));
            let rsum = vaddq_s8(vaddq_s8(r1, r7), r13);

            let rgt = vcgtq_s8(rsum, v_hi);
            let rlt = vcltq_s8(rsum, v_lo);
            let rwrap = vbslq_s8(rgt, vaddq_s8(rsum, v_neg3), rsum);
            let rwrap = vbslq_s8(rlt, vaddq_s8(rsum, v_three), rwrap);

            let center = vld1q_s8(ext.as_ptr().add(ei));
            let total = vaddq_s8(
                vaddq_s8(vaddq_s8(lwrap, center), rwrap),
                v_one,
            );

            let fgt = vcgtq_s8(total, v_hi);
            let flt = vcltq_s8(total, v_lo);
            let result = vbslq_s8(fgt, vaddq_s8(total, v_neg3), total);
            let result = vbslq_s8(flt, vaddq_s8(total, v_three), result);

            vst1q_s8(buf.as_mut_ptr().add(i), result);
            i += 16;
        }

        while i < STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
        }

        for i in 0..STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }

        let rc = &RC_TABLE[round];
        for lane in 0..LANES {
            let idx = lane * LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sponge_permutation_v1_neon(state: &mut [i8; STATE_SIZE]) {
    use std::arch::aarch64::*;

    let mut ext = [0i8; STATE_SIZE + 26];
    let mut buf = [0i8; STATE_SIZE];

    let v_one   = vdupq_n_s8(1);
    let v_hi    = vdupq_n_s8(1);
    let v_lo    = vdupq_n_s8(-1);
    let v_three = vdupq_n_s8(3);
    let v_neg3  = vdupq_n_s8(-3);

    for round in 0..ROUNDS {
        ext[..13].copy_from_slice(&state[STATE_SIZE - 13..]);
        ext[13..13 + STATE_SIZE].copy_from_slice(state);
        ext[13 + STATE_SIZE..].copy_from_slice(&state[..13]);

        let mut i = 0;
        while i + 16 <= STATE_SIZE {
            let ei = i + 13;
            let l13 = vld1q_s8(ext.as_ptr().add(ei - 13));
            let l7  = vld1q_s8(ext.as_ptr().add(ei - 7));
            let l1  = vld1q_s8(ext.as_ptr().add(ei - 1));
            let lsum = vaddq_s8(vaddq_s8(l13, l7), l1);
            let lgt = vcgtq_s8(lsum, v_hi);
            let llt = vcltq_s8(lsum, v_lo);
            let lwrap = vbslq_s8(lgt, vaddq_s8(lsum, v_neg3), lsum);
            let lwrap = vbslq_s8(llt, vaddq_s8(lsum, v_three), lwrap);
            let r1  = vld1q_s8(ext.as_ptr().add(ei + 1));
            let r7  = vld1q_s8(ext.as_ptr().add(ei + 7));
            let r13 = vld1q_s8(ext.as_ptr().add(ei + 13));
            let rsum = vaddq_s8(vaddq_s8(r1, r7), r13);
            let rgt = vcgtq_s8(rsum, v_hi);
            let rlt = vcltq_s8(rsum, v_lo);
            let rwrap = vbslq_s8(rgt, vaddq_s8(rsum, v_neg3), rsum);
            let rwrap = vbslq_s8(rlt, vaddq_s8(rsum, v_three), rwrap);
            let center = vld1q_s8(ext.as_ptr().add(ei));
            let total = vaddq_s8(
                vaddq_s8(vaddq_s8(lwrap, center), rwrap),
                v_one,
            );
            let fgt = vcgtq_s8(total, v_hi);
            let flt = vcltq_s8(total, v_lo);
            let result = vbslq_s8(fgt, vaddq_s8(total, v_neg3), total);
            let result = vbslq_s8(flt, vaddq_s8(total, v_three), result);
            vst1q_s8(buf.as_mut_ptr().add(i), result);
            i += 16;
        }
        while i < STATE_SIZE {
            let ei = i + 13;
            let left  = balanced_wrap(ext[ei-13] + ext[ei-7] + ext[ei-1]);
            let right = balanced_wrap(ext[ei+1]  + ext[ei+7] + ext[ei+13]);
            buf[i] = balanced_wrap(left + ext[ei] + right + 1);
            i += 1;
        }
        for i in 0..STATE_SIZE {
            state[PERM[i] as usize] = buf[i];
        }
        let rc = &RC_TABLE[round];
        for lane in 0..LANES {
            let idx = lane * LANES;
            state[idx] = balanced_wrap(state[idx] + rc[lane]);
        }
    }
}

pub fn sponge_permutation(state: &mut [i8; STATE_SIZE]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { sponge_permutation_v2_avx2(state); }
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe { sponge_permutation_v2_neon(state); }
            return;
        }
    }
    sponge_permutation_v2_scalar(state);
}

pub fn sponge_permutation_v1(state: &mut [i8; STATE_SIZE]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { sponge_permutation_v1_avx2(state); }
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe { sponge_permutation_v1_neon(state); }
            return;
        }
    }
    sponge_permutation_v1_scalar(state);
}

pub fn bytes_to_trits_pub(bytes: &[u8]) -> Vec<i8> {
    bytes_to_trits(bytes)
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

fn trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((trits.len() + 4) / 5);
    let mut i = 0;
    while i < trits.len() {
        let mut val: u8 = 0;
        let mut pow: u8 = 1;
        for j in 0..5 {
            if i + j < trits.len() {
                let t = (trits[i + j] + 1) as u8;
                val += t * pow;
            }
            pow = pow.wrapping_mul(3);
        }
        bytes.push(val);
        i += 5;
    }
    bytes
}

pub struct Sponge385Pub {
    state: [i8; STATE_SIZE],
    buf: [i8; RATE],
    buf_len: usize,
    absorbed: bool,
    use_v1: bool,
}

impl Sponge385Pub {
    pub fn new() -> Self {
        Self {
            state: [0i8; STATE_SIZE],
            buf: [0i8; RATE],
            buf_len: 0,
            absorbed: false,
            use_v1: false,
        }
    }

    pub fn new_v1() -> Self {
        Self {
            state: [0i8; STATE_SIZE],
            buf: [0i8; RATE],
            buf_len: 0,
            absorbed: false,
            use_v1: true,
        }
    }

    fn permute(&mut self) {
        if self.use_v1 {
            sponge_permutation_v1(&mut self.state);
        } else {
            sponge_permutation(&mut self.state);
        }
    }

    pub fn absorb(&mut self, input: &[i8]) {
        if input.is_empty() { return; }
        self.absorbed = true;
        let mut offset = 0;
        let input_len = input.len();

        if self.buf_len > 0 {
            let space = RATE - self.buf_len;
            let fill = if input_len < space { input_len } else { space };
            self.buf[self.buf_len..self.buf_len + fill]
                .copy_from_slice(&input[..fill]);
            self.buf_len += fill;
            offset = fill;

            if self.buf_len == RATE {
                for i in 0..RATE {
                    self.state[i] = trit_add(self.state[i], self.buf[i]);
                }
                self.permute();
                self.buf_len = 0;
            }
        }

        while offset + RATE <= input_len {
            let block = &input[offset..offset + RATE];
            for i in 0..RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            self.permute();
            offset += RATE;
        }

        let remaining = input_len - offset;
        if remaining > 0 {
            self.buf[self.buf_len..self.buf_len + remaining]
                .copy_from_slice(&input[offset..]);
            self.buf_len += remaining;
        }
    }

    pub fn absorb_bytes(&mut self, input: &[u8]) {
        let trits = bytes_to_trits(input);
        self.absorb(&trits);
    }

    pub fn squeeze(&mut self, output_trits: usize) -> Vec<i8> {
        if self.buf_len > 0 || !self.absorbed {
            for i in 0..self.buf_len {
                self.state[i] = trit_add(self.state[i], self.buf[i]);
            }
            if self.buf_len < RATE {
                self.state[self.buf_len] = trit_add(self.state[self.buf_len], 1);
            }
            self.buf_len = 0;
            self.permute();
        }

        let mut output = Vec::with_capacity(output_trits);
        while output.len() < output_trits {
            let remaining = output_trits - output.len();
            let take = if remaining < RATE { remaining } else { RATE };
            output.extend_from_slice(&self.state[..take]);
            if output.len() < output_trits {
                self.permute();
            }
        }
        output.truncate(output_trits);
        output
    }
}

pub fn hash(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(input);
    let output_trits = output_len * 5;
    let trits = sponge.squeeze(output_trits);
    let bytes = trits_to_bytes(&trits);
    bytes[..output_len].to_vec()
}

pub fn hash_v1(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut sponge = Sponge385Pub::new_v1();
    sponge.absorb_bytes(input);
    let output_trits = output_len * 5;
    let trits = sponge.squeeze(output_trits);
    let bytes = trits_to_bytes(&trits);
    bytes[..output_len].to_vec()
}

pub fn hash_hex(input: &[u8]) -> String {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(input);
    let trits = sponge.squeeze(243);
    let bytes = trits_to_bytes(&trits);
    let out = &bytes[..49];
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hash_hex_v1(input: &[u8]) -> String {
    let mut sponge = Sponge385Pub::new_v1();
    sponge.absorb_bytes(input);
    let trits = sponge.squeeze(243);
    let bytes = trits_to_bytes(&trits);
    let out = &bytes[..49];
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context);
    input.extend_from_slice(material);
    hash(&input, key_len)
}

pub fn sponge385_derive_key(
    domain: &[u8],
    addr_a: &[u8],
    addr_b: &[u8],
    kem_shared_secret: &[u8; 32],
    epoch: u64,
) -> Vec<u8> {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(domain);
    sponge.absorb_bytes(addr_a);
    sponge.absorb_bytes(addr_b);
    sponge.absorb_bytes(kem_shared_secret);
    sponge.absorb_bytes(&epoch.to_le_bytes());
    let trits = sponge.squeeze(RATE);
    let bytes = trits_to_bytes(&trits);
    bytes[..32].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let a = hash(b"hello world", 32);
        let b = hash(b"hello world", 32);
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = hash(b"hello", 32);
        let b = hash(b"world", 32);
        assert_ne!(a, b);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let a = derive_key(b"ctx", b"material", 32);
        let b = derive_key(b"ctx", b"material", 32);
        assert_eq!(a, b);
    }

    #[test]
    fn test_derive_key_length() {
        let k = derive_key(b"ctx", b"material", 32);
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn test_sponge385_derive_key_symmetric() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret = [42u8; 32];
        let epoch = 100u64;
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, epoch,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, epoch,
        );
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_sponge385_derive_key_different_secrets() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret1 = [42u8; 32];
        let secret2 = [99u8; 32];
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret1, 100,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret2, 100,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_sponge385_derive_key_different_epochs() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret = [42u8; 32];
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, 100,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, 200,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_perm_full_period() {
        let mut seen = [false; STATE_SIZE];
        for i in 0..STATE_SIZE {
            let dest = PERM[i] as usize;
            assert!(!seen[dest]);
            seen[dest] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn test_neighbor_distances_coprime() {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 { let t = b; b = a % b; a = t; } a
        }
        assert_eq!(gcd(1, STATE_SIZE), 1);
        assert_eq!(gcd(7, STATE_SIZE), 1);
        assert_eq!(gcd(13, STATE_SIZE), 1);
    }

    #[test]
    fn test_chi_is_bijection() {
        let mut outputs = Vec::new();
        for a0 in 0u8..3 {
            for a1 in 0u8..3 {
                for a2 in 0u8..3 {
                    let result = gf27_pow17([a0, a1, a2]);
                    outputs.push(result);
                }
            }
        }
        assert_eq!(gf27_pow17([0, 0, 0]), [0, 0, 0], "0 must map to 0");
        outputs.sort();
        outputs.dedup();
        assert_eq!(outputs.len(), 27, "chi must be a bijection over all 27 GF(27) elements");
    }

    #[test]
    fn test_chi_v2_differs_from_v1() {
        let mut s1 = [0i8; STATE_SIZE];
        let mut s2 = [0i8; STATE_SIZE];
        s1[0] = 1; s1[1] = -1; s1[3] = 1;
        s2 = s1;
        sponge_permutation_v1(&mut s1);
        sponge_permutation(&mut s2);
        assert_ne!(s1, s2, "v2 (with chi) must produce different output from v1");
    }

    #[test]
    fn test_simd_matches_scalar() {
        let mut s_simd = [0i8; STATE_SIZE];
        let mut s_scalar = [0i8; STATE_SIZE];
        for i in 0..STATE_SIZE {
            let v = ((i * 7 + 3) % 3) as i8 - 1;
            s_simd[i] = v;
            s_scalar[i] = v;
        }
        sponge_permutation(&mut s_simd);
        sponge_permutation_v2_scalar(&mut s_scalar);
        assert_eq!(s_simd, s_scalar, "SIMD v2 must match scalar v2");
    }

    #[test]
    fn test_simd_v1_matches_scalar() {
        let mut s_simd = [0i8; STATE_SIZE];
        let mut s_scalar = [0i8; STATE_SIZE];
        for i in 0..STATE_SIZE {
            let v = ((i * 7 + 3) % 3) as i8 - 1;
            s_simd[i] = v;
            s_scalar[i] = v;
        }
        sponge_permutation_v1(&mut s_simd);
        sponge_permutation_v1_scalar(&mut s_scalar);
        assert_eq!(s_simd, s_scalar, "SIMD v1 must match scalar v1");
    }
}
