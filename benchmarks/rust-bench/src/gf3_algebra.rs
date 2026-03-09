// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// GF(3) Algebra — Division-Free Ternary-Native Operations
// Location: ternary-math/src/gf3_algebra.rs
//
// GF(3) elements are bounded to {0,1,2}. Modular reduction uses
// conditional subtract (1-2 cycles), NOT the % operator (20-40 cycles).
// Benchmark: 2.1× sponge speedup (5494 → 2616 ns).

#[inline(always)] fn mod3_small(mut n: u8) -> u8 { if n >= 3 { n -= 3; } n }
#[inline(always)] fn mod3_med(mut n: u8) -> u8 { if n >= 3 { n -= 3; } if n >= 3 { n -= 3; } n }
#[inline(always)] fn mod7_small(mut n: u8) -> u8 { if n >= 14 { n -= 14; } if n >= 7 { n -= 7; } n }
#[inline(always)] fn mod27_small(mut n: u8) -> u8 { if n >= 27 { n -= 27; } n }

#[inline(always)] pub const fn gf3_add(a: u8, b: u8) -> u8 { let s = a + b; if s >= 3 { s - 3 } else { s } }
#[inline(always)] pub const fn gf3_sub(a: u8, b: u8) -> u8 { let s = a + 3 - b; if s >= 3 { s - 3 } else { s } }
#[inline(always)] pub const fn gf3_mul(a: u8, b: u8) -> u8 { let p = a * b; if p >= 3 { p - 3 } else { p } }
#[inline(always)] pub const fn gf3_neg(a: u8) -> u8 { let s = 3 - a; if s >= 3 { 0 } else { s } }
#[inline(always)] pub const fn gf3_square(a: u8) -> u8 { let p = a * a; if p >= 3 { p - 3 } else { p } }
#[inline(always)] pub const fn gf3_inv(a: u8) -> u8 { assert!(a != 0, "no inverse for 0"); a }

#[inline(always)] pub const fn rep_c_to_b(c: u8) -> u8 { c - 1 }
#[inline(always)] pub const fn rep_b_to_c(b: u8) -> u8 { b + 1 }
pub fn batch_c_to_b(trits: &mut [u8]) { for t in trits.iter_mut() { *t -= 1; } }
pub fn batch_b_to_c(trits: &mut [u8]) { for t in trits.iter_mut() { *t += 1; } }

pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    let mut dist: u32 = 0;
    for i in 0..a.len() { dist += gf3_square(gf3_sub(a[i], b[i])) as u32; }
    dist
}
pub fn hamming_distance_rep_c(a: &[u8], b: &[u8]) -> u32 {
    let mut dist: u32 = 0;
    for i in 0..a.len() { dist += gf3_square(gf3_sub(rep_c_to_b(a[i]), rep_c_to_b(b[i]))) as u32; }
    dist
}
pub fn hamming_distance_27(a: &[u8; 27], b: &[u8; 27]) -> u32 {
    let mut dist: u32 = 0;
    for i in 0..27 { dist += gf3_square(gf3_sub(a[i], b[i])) as u32; }
    dist
}

pub fn has_forgery(trits_rep_c: &[u8]) -> bool {
    let mut product: u8 = 1;
    for &t in trits_rep_c {
        product = mod7_small(product * t);
        if product == 0 { return true; }
    }
    false
}
pub fn find_forgeries(trits_rep_c: &[u8]) -> Vec<usize> {
    trits_rep_c.iter().enumerate().filter(|(_, &t)| t == 0).map(|(i, _)| i).collect()
}

pub fn gf3_vec_add(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_add(a[i], b[i]); } }
pub fn gf3_vec_sub(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_sub(a[i], b[i]); } }
pub fn gf3_vec_mul(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_mul(a[i], b[i]); } }
pub fn gf3_dot(a: &[u8], b: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for i in 0..a.len() { sum = gf3_add(sum, gf3_mul(a[i], b[i])); }
    sum
}
pub fn gf3_scalar_mul(scalar: u8, a: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_mul(scalar, a[i]); } }

pub const TIS27_STATE_WIDTH: usize = 54;
pub const TIS27_RATE: usize = 27;
pub const TIS27_ROUNDS: usize = 27;
pub const TIS27_STRIDE: usize = 13;

pub const PI_TABLE: [u8; 54] = [
     0, 13, 26, 39, 52, 11, 24, 37, 50,  9, 22, 35, 48,  7, 20, 33, 46,  5,
    18, 31, 44,  3, 16, 29, 42,  1, 14, 27, 40, 53, 12, 25, 38, 51, 10, 23,
    36, 49,  8, 21, 34, 47,  6, 19, 32, 45,  4, 17, 30, 43,  2, 15, 28, 41
];
pub const TIS27_ROUND_CONSTANTS: [u8; 27] = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

pub fn sponge_theta(state: &[u8; TIS27_STATE_WIDTH], out: &mut [u8; TIS27_STATE_WIDTH]) {
    out[0] = mod3_med(state[53] + state[0] + state[1]);
    for i in 1..53 { out[i] = mod3_med(state[i-1] + state[i] + state[i+1]); }
    out[53] = mod3_med(state[52] + state[53] + state[0]);
}

pub fn sponge_pi(state: &[u8; TIS27_STATE_WIDTH], out: &mut [u8; TIS27_STATE_WIDTH]) {
    for i in 0..TIS27_STATE_WIDTH { out[i] = state[PI_TABLE[i] as usize]; }
}

pub fn tis27_round(state: &mut [u8; TIS27_STATE_WIDTH], round: usize) {
    let mut temp = [0u8; TIS27_STATE_WIDTH];
    sponge_theta(state, &mut temp);
    sponge_pi(&temp, state);
    let off = if round >= 27 { round - 27 } else { round };
    for i in 0..TIS27_RATE {
        let rc_idx = mod27_small((i + off) as u8) as usize;
        state[i] = gf3_add(state[i], TIS27_ROUND_CONSTANTS[rc_idx]);
    }
}

pub fn tis27_sponge(input_trits: &[u8], output_len: usize) -> Vec<u8> {
    let mut state = [0u8; TIS27_STATE_WIDTH];
    let mut offset = 0;
    while offset < input_trits.len() {
        let block_len = std::cmp::min(TIS27_RATE, input_trits.len() - offset);
        for i in 0..block_len { state[i] = gf3_add(state[i], input_trits[offset + i]); }
        for round in 0..TIS27_ROUNDS { tis27_round(&mut state, round); }
        offset += TIS27_RATE;
    }
    let mut output = Vec::with_capacity(output_len);
    while output.len() < output_len {
        let take = std::cmp::min(TIS27_RATE, output_len - output.len());
        output.extend_from_slice(&state[..take]);
        if output.len() < output_len { for round in 0..TIS27_ROUNDS { tis27_round(&mut state, round); } }
    }
    output.truncate(output_len);
    output
}

pub fn repunit_checksum(trits_rep_c: &[u8]) -> u64 {
    let mut value: u64 = 0;
    for i in (0..trits_rep_c.len()).rev() { value = (value * 3 + (trits_rep_c[i] - 1) as u64) % 364; }
    value
}

pub fn project_to_gf3(k: u64, n: u64) -> u8 { let v = 3 * k / n; if v >= 2 { 2 } else { v as u8 } }
pub fn derive_trit(k: u64, n: u64) -> u8 { project_to_gf3(k, n) + 1 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_gf3_add() { let e=[[0,1,2],[1,2,0],[2,0,1]]; for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_add(a,b),e[a as usize][b as usize]);}}}
    #[test] fn test_gf3_mul() { let e=[[0,0,0],[0,1,2],[0,2,1]]; for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_mul(a,b),e[a as usize][b as usize]);}}}
    #[test] fn test_gf3_sub() { for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_sub(a,b),(a+3-b)%3);}}}
    #[test] fn test_gf3_square() { assert_eq!(gf3_square(0),0); assert_eq!(gf3_square(1),1); assert_eq!(gf3_square(2),1); }
    #[test] fn test_mod3_med() { for n in 0..=6u8 { assert_eq!(mod3_med(n), n%3); } }
    #[test] fn test_mod7_small() { for n in 0..=18u8 { assert_eq!(mod7_small(n), n%7); } }
    #[test] fn test_pi_table() { for i in 0..54{assert_eq!(PI_TABLE[i],((i*13)%54)as u8);} }
    #[test] fn test_hamming_id() { let a=[0u8,1,2,0,1,2]; assert_eq!(hamming_distance(&a,&a),0); }
    #[test] fn test_hamming_all() { assert_eq!(hamming_distance(&[0;5],&[1,2,1,2,1]),5); }
    #[test] fn test_forgery_ok() { assert!(!has_forgery(&[1,2,3,1,2,3])); }
    #[test] fn test_forgery_bad() { assert!(has_forgery(&[1,0,3,1])); }
    #[test] fn test_theta_zeros() { let s=[0u8;54]; let mut o=[0u8;54]; sponge_theta(&s,&mut o); assert!(o.iter().all(|&v|v==0)); }
    #[test] fn test_theta_ones() { let s=[1u8;54]; let mut o=[0u8;54]; sponge_theta(&s,&mut o); assert!(o.iter().all(|&v|v==0)); }
}
