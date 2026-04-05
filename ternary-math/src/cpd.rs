// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TM-2026-030: Coprime Periodic Detection — Stride Delta Preprocessing
// ternary-math/src/cpd.rs
//
// Integrates into TTC as DeltaFlag::STRIDE (7).
// Detects periodic structure via autocorrelation at coprime distances,
// delta-encodes at the detected stride. Stride stored as first 2 bytes
// of the delta data. Existing LZ77 + rANS handles entropy coding.

const COPRIME_STRIDES: [u16; 13] = [7,11,13,14,15,28, 77,91,105,143,154,165,182];
const MAX_SCAN: u16 = 256;
const AC_THRESHOLD: f64 = 0.25;
const H_THRESHOLD: f64 = 0.3;

fn autocorrelation(data: &[u8], s: usize) -> f64 {
    if s == 0 || s >= data.len() { return 0.0; }
    let mut h = 0u64;
    for i in s..data.len() { h += (data[i] == data[i-s]) as u64; }
    h as f64 / (data.len()-s) as f64
}

/// Detect best stride for delta encoding. Returns (stride, ac) or (0, 0.0).
pub fn detect_stride(data: &[u8]) -> (u16, f64) {
    if data.len() < 32 { return (0, 0.0); }
    let sample = &data[..data.len().min(4096)];
    let (mut bs, mut ba): (u16, f64) = (0, 0.0);
    for &s in &COPRIME_STRIDES {
        if (s as usize) >= sample.len()/2 { continue; }
        let ac = autocorrelation(sample, s as usize);
        if ac > ba { ba = ac; bs = s; }
    }
    for s in 2u16..=MAX_SCAN {
        if (s as usize) >= sample.len()/2 { break; }
        if COPRIME_STRIDES.contains(&s) { continue; }
        let ac = autocorrelation(sample, s as usize);
        if ac > ba { ba = ac; bs = s; }
    }
    if ba >= AC_THRESHOLD { (bs, ba) } else { (0, 0.0) }
}

/// Stride delta encode. Output: [stride u16 BE] [delta bytes].
pub fn stride_delta_encode(data: &[u8], stride: u16, out: &mut Vec<u8>) {
    out.clear(); out.reserve(2+data.len());
    out.push((stride>>8) as u8); out.push((stride&0xFF) as u8);
    let s = stride as usize;
    for i in 0..data.len() {
        out.push(if i < s { data[i] } else { data[i].wrapping_sub(data[i-s]) });
    }
}

/// Stride delta decode. Reads stride from first 2 bytes.
pub fn stride_delta_decode(enc: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if enc.len() < 2 { return; }
    let stride = ((enc[0] as u16)<<8) | enc[1] as u16;
    let s = stride as usize; let data = &enc[2..];
    out.reserve(data.len());
    for i in 0..data.len() {
        out.push(if i < s { data[i] } else { data[i].wrapping_add(out[i-s]) });
    }
}

/// Does stride delta beat order-1 delta on this data?
pub fn stride_beats_order1(data: &[u8], stride: u16) -> bool {
    if stride <= 1 { return false; }
    let sample = &data[..data.len().min(1024)];
    let s = stride as usize;
    let mut cs = [0u32;256]; let mut c1 = [0u32;256];
    c1[sample[0] as usize] += 1;
    for i in 0..sample.len() {
        cs[if i>=s { sample[i].wrapping_sub(sample[i-s]) } else { sample[i] } as usize] += 1;
        if i > 0 { c1[sample[i].wrapping_sub(sample[i-1]) as usize] += 1; }
    }
    let n = sample.len() as f64;
    let h = |c: &[u32;256]| -> f64 { c.iter().filter(|&&v|v>0).map(|&v|{let p=v as f64/n;-p*p.log2()}).sum() };
    h(&c1) - h(&cs) >= H_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn roundtrip_stride7() {
        let d: Vec<u8> = (0..4096).map(|i|((i%7)*30+i/7) as u8).collect();
        let mut e=Vec::new(); stride_delta_encode(&d,7,&mut e);
        let mut o=Vec::new(); stride_delta_decode(&e,&mut o);
        assert_eq!(d, o);
    }
    #[test] fn roundtrip_all_strides() {
        let d: Vec<u8> = (0..2048).map(|i|(i*7+13) as u8).collect();
        for s in [1u16,2,7,11,13,28,37,67,128] {
            let mut e=Vec::new(); stride_delta_encode(&d,s,&mut e);
            let mut o=Vec::new(); stride_delta_decode(&e,&mut o);
            assert_eq!(d, o, "stride {s}");
        }
    }
    #[test] fn detect_periodic7() {
        let d: Vec<u8> = (0..4096).map(|i|((i%7)*30) as u8).collect();
        let (s, ac) = detect_stride(&d);
        assert_eq!(s, 7); assert!(ac > 0.9);
    }
    #[test] fn detect_csv() {
        let row = b"2026-03-15,node_01,22.5,1013.2,45.1,OK\n";
        let d: Vec<u8> = row.iter().cycle().take(4096).cloned().collect();
        let (s, ac) = detect_stride(&d);
        assert!(ac > 0.9); assert_eq!(s, row.len() as u16);
    }
    #[test] fn beats_order1_csv() {
        let row = b"2026-03-15,node_01,22.5,1013.2,45.1,OK\n";
        let d: Vec<u8> = row.iter().cycle().take(4096).cloned().collect();
        let (s, _) = detect_stride(&d);
        assert!(stride_beats_order1(&d, s));
    }
    #[test] fn no_beat_ramp() {
        let d: Vec<u8> = (0..4096).map(|i|(i%256) as u8).collect();
        assert!(!stride_beats_order1(&d, 7));
    }
    #[test] fn embedded_stride() {
        let mut e=Vec::new(); stride_delta_encode(b"test",195,&mut e);
        assert_eq!(e[0], 0); assert_eq!(e[1], 195);
    }
}
