// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TM-2026-030: Coprime Periodic Detection — Stride Delta Preprocessing
// ternary-math/src/cpd.rs  (TTC v5.0.2)
//
// DeltaFlag::STRIDE (7). Detects periodic structure via autocorrelation
// at coprime distances, delta-encodes at detected stride.
// Uses dual-circle entropy estimate (α = Z₂₇/(Z₂₇+Z₂₈)) for stride comparison.

// Framework coprime strides — derived from polygon generators and pair LCMs.
// Source of truth: constants.rs §0 TritInt constants → §1+ boundary crossings.
const COPRIME_STRIDES: [u16; 13] = [
    crate::constants::T_POLYGON_7.host_u32() as u16,   // 7
    crate::constants::T_POLYGON_11.host_u32() as u16,  // 11
    crate::constants::REPUNIT_3 as u16,                      // 13 = R₃
    crate::constants::ROOT_X1 as u16,                        // 14 = x₁
    crate::constants::T_POLYGON_15.host_u32() as u16,  // 15
    crate::constants::T_Z28_ORDER.host_u32() as u16,   // 28 = Z₂₈
    // Coprime pair LCMs — from constants.rs COPRIME_PAIR_LCMS
    crate::constants::COPRIME_PAIR_LCMS[0] as u16,          // 77  = 7×11
    crate::constants::COPRIME_PAIR_LCMS[1] as u16,          // 91  = 7×13
    crate::constants::COPRIME_PAIR_LCMS[2] as u16,          // 105 = 7×15
    crate::constants::COPRIME_PAIR_LCMS[3] as u16,          // 143 = 11×13
    crate::constants::COPRIME_PAIR_LCMS[4] as u16,          // 154 = 11×14
    crate::constants::COPRIME_PAIR_LCMS[5] as u16,          // 165 = 11×15
    crate::constants::COPRIME_PAIR_LCMS[6] as u16,          // 182 = 13×14
];
const MAX_SCAN: u16 = 256;
const AC_THRESHOLD: f64 = 0.25;
const H_THRESHOLD: f64 = 0.3;

// Dual-circle blend: α = Z₂₇/(Z₂₇+Z₂₈) = 3³/(3³ + 3³+1) = 27/55
const ALPHA: f64 = 27.0 / 55.0;

fn autocorrelation(data: &[u8], s: usize) -> f64 {
    if s == 0 || s >= data.len() { return 0.0; }
    let mut h = 0u64;
    for i in s..data.len() { h += (data[i] == data[i - s]) as u64; }
    h as f64 / (data.len() - s) as f64
}

pub fn detect_stride(data: &[u8]) -> (u16, f64) {
    if data.len() < 32 { return (0, 0.0); }
    let sample = &data[..data.len().min(4096)];
    let (mut bs, mut ba): (u16, f64) = (0, 0.0);
    for &s in &COPRIME_STRIDES {
        if (s as usize) >= sample.len() / 2 { continue; }
        let ac = autocorrelation(sample, s as usize);
        if ac > ba { ba = ac; bs = s; }
    }
    for s in 2u16..=MAX_SCAN {
        if (s as usize) >= sample.len() / 2 { break; }
        if COPRIME_STRIDES.contains(&s) { continue; }
        let ac = autocorrelation(sample, s as usize);
        if ac > ba { ba = ac; bs = s; }
    }
    if ba >= AC_THRESHOLD { (bs, ba) } else { (0, 0.0) }
}

pub fn stride_delta_encode(data: &[u8], stride: u16, out: &mut Vec<u8>) {
    out.clear(); out.reserve(2 + data.len());
    out.push((stride >> 8) as u8); out.push((stride & 0xFF) as u8);
    let s = stride as usize;
    for i in 0..data.len() {
        out.push(if i < s { data[i] } else { data[i].wrapping_sub(data[i - s]) });
    }
}

pub fn stride_delta_decode(enc: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if enc.len() < 2 { return; }
    let stride = ((enc[0] as u16) << 8) | enc[1] as u16;
    let s = stride as usize; let data = &enc[2..];
    out.reserve(data.len());
    for i in 0..data.len() {
        out.push(if i < s { data[i] } else { data[i].wrapping_add(out[i - s]) });
    }
}

/// Dual-circle entropy (geometric-compression.md §9).
/// H = α·H_alg(Z₂₇) + (1−α)·H_geo(Z₂₈)
fn dual_circle_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let n = data.len();
    let mut counts = [0u32; 256];
    for &b in data { counts[b as usize] += 1; }
    let nf = n as f64;
    let h_alg: f64 = counts.iter().filter(|&&c| c > 0)
        .map(|&c| { let p = c as f64 / nf; -p * p.log2() }).sum();
    if n <= 28 { return h_alg; }
    let mut gc = [0u32; 256];
    for i in 28..n { gc[data[i].wrapping_sub(data[i - 28]) as usize] += 1; }
    let gn = (n - 28) as f64;
    let h_geo: f64 = gc.iter().filter(|&&c| c > 0)
        .map(|&c| { let p = c as f64 / gn; -p * p.log2() }).sum();
    ALPHA * h_alg + (1.0 - ALPHA) * h_geo
}

pub fn stride_beats_order1(data: &[u8], stride: u16) -> bool {
    if stride <= 1 { return false; }
    let sample = &data[..data.len().min(1024)];
    let s = stride as usize;
    let o1: Vec<u8> = sample.iter().enumerate()
        .map(|(i, &v)| if i > 0 { v.wrapping_sub(sample[i - 1]) } else { v }).collect();
    let sd: Vec<u8> = sample.iter().enumerate()
        .map(|(i, &v)| if i >= s { v.wrapping_sub(sample[i - s]) } else { v }).collect();
    dual_circle_entropy(&o1) - dual_circle_entropy(&sd) >= H_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip_stride7() {
        let d: Vec<u8> = (0..4096).map(|i| ((i%7)*30+i/7) as u8).collect();
        let mut e = Vec::new(); stride_delta_encode(&d, 7, &mut e);
        let mut o = Vec::new(); stride_delta_decode(&e, &mut o);
        assert_eq!(d, o);
    }
    #[test] fn roundtrip_all() {
        let d: Vec<u8> = (0..2048).map(|i| (i*7+13) as u8).collect();
        for s in [1u16,2,7,11,13,28,37,67,128] {
            let mut e = Vec::new(); stride_delta_encode(&d, s, &mut e);
            let mut o = Vec::new(); stride_delta_decode(&e, &mut o);
            assert_eq!(d, o, "stride {s}");
        }
    }
    #[test] fn detect_periodic7() {
        let d: Vec<u8> = (0..4096).map(|i| ((i%7)*30) as u8).collect();
        let (s, ac) = detect_stride(&d); assert_eq!(s, 7); assert!(ac > 0.9);
    }
    #[test] fn detect_csv() {
        let row = b"2026-03-15,node_01,22.5,1013.2,45.1,OK\n";
        let d: Vec<u8> = row.iter().cycle().take(4096).cloned().collect();
        let (s, ac) = detect_stride(&d); assert!(ac > 0.9); assert_eq!(s, row.len() as u16);
    }
    #[test] fn beats_order1_csv() {
        let row = b"2026-03-15,node_01,22.5,1013.2,45.1,OK\n";
        let d: Vec<u8> = row.iter().cycle().take(4096).cloned().collect();
        let (s, _) = detect_stride(&d); assert!(stride_beats_order1(&d, s));
    }
    #[test] fn no_beat_ramp() {
        let d: Vec<u8> = (0..4096).map(|i| (i%256) as u8).collect();
        assert!(!stride_beats_order1(&d, 7));
    }
    #[test] fn dual_circle_entropy_constant() {
        let h = dual_circle_entropy(&vec![42u8; 1024]);
        assert!(h < 0.01, "constant data entropy={h}");
    }
}
