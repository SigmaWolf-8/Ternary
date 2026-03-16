// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
// Author: RSalvi@Salvigroup.com
//
// TM-2026-017: Tribonacci Ternary Compression (TTC) Protocol v2.0
// Production implementation — ternary-math/src/ttc.rs
// Revision: v4.2 — chunk size scaling, GURFT fast-path, compact freq tables,
//                   ternary rANS compliance, scratch buffer reuse

//! # TTC v2.0 — Tribonacci Ternary Compression Engine
//!
//! Native PlenumNET compression service implementing the TTC v2.0 protocol
//! (TM-2026-017). Nine compression levels (TTC1/TTC2/TTC3), four serialization
//! modes (STORED, COMPRESSED, TERNARY_ENHANCED, TERNARY_ANS), adaptive GF(3)
//! representation switching, domain-specific preprocessing (AUDIO/IMAGE/GENOMIC/
//! SOURCE/LOG/STRUCTURED), GURFT adaptive base selection, and beam-search
//! optimal parsing.
//!
//! ## v4.2 Changes (pure ternary architecture + optimization pass)
//!
//! - **Pure ternary window/chunk sizes**: All window and chunk sizes are now
//!   pure powers of 3 (3^8 through 3^16). Zero binary multipliers. Window and
//!   chunk boundaries are structurally unified with the ANS state machine
//!   (L4 chunk = 3^11 = TANS_L) and the hypercube geometry (L7 chunk = 3^13).
//!   L5 chunk = 3^12 = 531,441 bytes — most documents fit in 1 chunk.
//! - **GURFT entropy fast-path**: Skips torsion/delta/periodicity (~13K trig
//!   evaluations) when entropy > 6.0 bits/byte (base-3 guaranteed optimal).
//! - **Compact frequency table**: Delta-varint encoding, ~50-60% smaller.
//! - **Pre-allocations**: symbols, distances, trits, tokens vectors sized upfront.
//! - **Symbol range fix**: Runs 256-511, matches 512-767 (no overlap at 511).
//!
//! ## Inter-Cube Parallel Dispatch (§4.8)
//!
//! The 26-tunnel Inter-Cube model is implemented natively via `rayon`. Two modes:
//!
//! - **Independent chunks**: Full parallel across 26 tunnels per round.
//! - **Dependent chunks**: 13+13 pipelined — Phase 1 parallel, Phase 2 sequential.
//!
//! Runtime gated: parallel dispatch only when `rayon::current_num_threads() > 1`
//! AND chunk count ≥ threshold. Falls back to sequential automatically on
//! single-threaded runtimes.
//!
//! ## Cargo.toml dependency
//!
//! ```toml
//! [dependencies]
//! rayon = "1.8"
//! libm = "0.2"
//! ```

use rayon::prelude::*;

// ─── Constants ──────────────────────────────────────────────────────────────

/// TTC1 magic bytes: "TTC1" = 0x54544331
pub const MAGIC_TTC1: [u8; 4] = [0x54, 0x54, 0x43, 0x31];
/// TTCM magic bytes: "TTCM" = 0x5454434D
pub const MAGIC_TTCM: [u8; 4] = [0x54, 0x54, 0x43, 0x4D];
pub const VERSION_V2: u8 = 0x03;
pub const VERSION_V1: u8 = 0x02;
pub const HEADER_SIZE: usize = 96;
pub const CHUNK_MAP_ENTRY_SIZE: usize = 16;
pub const TAU: f64 = 1.839_286_755_214_161_1;
pub const GOLDEN_ANGLE: f64 = 139.035_628;
pub const PHI: f64 = 1.618_033_988_749_895;
pub const PHASE_DRIFT_RATE: f64 = 3.956;
pub const LOG2_3: f64 = 1.584_962_500_7;
pub const TUNNEL_COUNT: usize = 26;
/// Pure ternary power constants — all window/chunk sizes derive from these.
/// No binary multipliers. Every structural boundary is 3^k.
pub const T3_8: usize = 6_561;         // 3^8  — L1 window
pub const T3_9: usize = 19_683;        // 3^9  — L1 chunk, L2 window
pub const T3_10: usize = 59_049;       // 3^10 — L3 window+chunk
pub const T3_11: usize = 177_147;      // 3^11 — L4 window+chunk = TANS_L
pub const T3_12: usize = 531_441;      // 3^12 — L5 window+chunk (document sweet spot)
pub const T3_13: usize = 1_594_323;    // 3^13 — hypercube vertices, L7+ chunk
pub const T3_14: usize = 4_782_969;    // 3^14 — L7 window
pub const T3_15: usize = 14_348_907;   // 3^15 — L8 window
pub const T3_16: usize = 43_046_721;   // 3^16 — L9 window
/// tANS table size: L = 3^11 = 177,147 — structurally unified with L4 chunk size.
pub const TANS_L: u32 = 177_147;
pub const TANS_EOB: u16 = 1023;
pub const TANS_ALPHABET: usize = 1024;
pub const BEAM_WIDTH: usize = 8;
pub const TAU_HARMONIC: f64 = 0.72;
pub const TAU_HOLOGRAPHIC: f64 = 0.80;
pub const TAU_RESONANCE: f64 = 0.95;
pub const DELTA_HOLOGRAPHIC: f64 = 0.80;
pub const ENTROPY_GATE: f64 = 7.5;
pub const MODE_PRUNE_ENTROPY: f64 = 7.0;

const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0u32;
    while n < 256 {
        let mut c = n;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 { c = 0xEDB8_8320 ^ (c >> 1); } else { c >>= 1; }
            k += 1;
        }
        table[n as usize] = c;
        n += 1;
    }
    table
};

/// Tribonacci sequence T(0)..T(29). Index-preserving: TRIBONACCI_SEQ[7] = 13 = T₇.
/// T(n) = T(n-1) + T(n-2) + T(n-3), seeded T(0)=0, T(1)=0, T(2)=1.
const TRIBONACCI_SEQ: [u64; 30] = [
    0, 0, 1, 1, 2, 4, 7, 13, 24, 44,
    81, 149, 274, 504, 927, 1705, 3136, 5768, 10609, 19513,
    35890, 66012, 121415, 223317, 410744, 755476, 1389537, 2555757, 4700770, 8646064,
];

// ─── Error Type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TtcError {
    InvalidMagic, UnsupportedVersion(u8), InvalidMode(u8), InvalidLevel(u8),
    InvalidDeltaFlag(u8), InvalidDomainTransform(u8),
    TruncatedHeader, TruncatedChunkMap, TruncatedPayload,
    Crc32Mismatch { expected: u32, computed: u32 },
    ImageWidthRequired, DecompressionError(String), SerializationError(String),
}

impl core::fmt::Display for TtcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid archive format. Expected TTC1 or TTCM magic."),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported version byte: 0x{v:02X}"),
            Self::InvalidMode(m) => write!(f, "mode must be 0–7, got {m}"),
            Self::InvalidLevel(l) => write!(f, "level must be 1–9 or a valid tier name (TTC1-1 through TTC3-3), got {l}"),
            Self::InvalidDeltaFlag(d) => write!(f, "Reserved delta_flag 0b111 encountered: {d}"),
            Self::InvalidDomainTransform(d) => write!(f, "Reserved domain transform 0b111: {d}"),
            Self::TruncatedHeader => write!(f, "Archive truncated: header < 96 bytes"),
            Self::TruncatedChunkMap => write!(f, "Archive truncated: chunk map incomplete"),
            Self::TruncatedPayload => write!(f, "Archive truncated: payload incomplete"),
            Self::Crc32Mismatch { expected, computed } => write!(f, "CRC32 mismatch: expected 0x{expected:08X}, computed 0x{computed:08X}"),
            Self::ImageWidthRequired => write!(f, "imageWidth required for IMAGE mode with MED predictor"),
            Self::DecompressionError(s) => write!(f, "Decompression error: {s}"),
            Self::SerializationError(s) => write!(f, "Serialization error: {s}"),
        }
    }
}

pub type TtcResult<T> = Result<T, TtcError>;

// ─── Enumerations ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMode {
    Basic = 0, Temporal = 1, Image = 2, Audio = 3,
    Genomic = 4, Source = 5, Log = 6, Structured = 7,
}

impl CompressionMode {
    #[inline]
    pub fn from_u8(v: u8) -> TtcResult<Self> {
        match v {
            0 => Ok(Self::Basic), 1 => Ok(Self::Temporal), 2 => Ok(Self::Image),
            3 => Ok(Self::Audio), 4 => Ok(Self::Genomic), 5 => Ok(Self::Source),
            6 => Ok(Self::Log), 7 => Ok(Self::Structured),
            _ => Err(TtcError::InvalidMode(v)),
        }
    }
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "BASIC", Self::Temporal => "TEMPORAL", Self::Image => "IMAGE",
            Self::Audio => "AUDIO", Self::Genomic => "GENOMIC", Self::Source => "SOURCE",
            Self::Log => "LOG", Self::Structured => "STRUCTURED",
        }
    }
    #[inline]
    pub fn allowed_bases(self) -> &'static [u16] {
        match self {
            Self::Basic => &[3], Self::Temporal => &[3, 13, 28, 70, 364],
            Self::Image | Self::Genomic | Self::Source | Self::Structured => &[3, 13],
            Self::Audio | Self::Log => &[3, 13, 28],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChunkMode { Stored = 0, Compressed = 1, TernaryEnhanced = 2, TernaryAns = 3 }

impl ChunkMode {
    #[inline]
    pub fn from_u8(v: u8) -> TtcResult<Self> {
        match v { 0 => Ok(Self::Stored), 1 => Ok(Self::Compressed), 2 => Ok(Self::TernaryEnhanced),
            3 => Ok(Self::TernaryAns), _ => Err(TtcError::DecompressionError(format!("Unknown chunk mode: {v}"))) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfRep { A, B, C }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parsing { Greedy, Lazy, BeamOptimal }

#[derive(Debug, Clone)]
pub enum Token { Literal(u8), Run { byte: u8, length: usize }, Match { dist: usize, length: usize } }

// ─── Level Configuration (§4.2) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LevelConfig {
    pub level: u8, pub tier_name: &'static str, pub window_size: usize,
    pub min_match: usize, pub min_run: usize, pub skip_gurft: bool,
    pub chunk_size: usize, pub chain_depth: usize, pub parsing: Parsing, pub candidates: usize,
}

static LEVEL_CONFIGS: [LevelConfig; 9] = [
    // TTC1: Speed tier.
    // L1: 8-trit window (3^8 = 6,561), 9-trit chunk (3^9 = 19,683)
    LevelConfig { level: 1, tier_name: "TTC1-1", window_size: 6_561,       min_match: 8, min_run: 6, skip_gurft: true,  chunk_size: 19_683,      chain_depth: 8,   parsing: Parsing::Greedy,      candidates: 2 },
    // L2: 9-trit window (3^9 = 19,683), 10-trit chunk (3^10 = 59,049)
    LevelConfig { level: 2, tier_name: "TTC1-2", window_size: 19_683,      min_match: 6, min_run: 5, skip_gurft: true,  chunk_size: 59_049,      chain_depth: 16,  parsing: Parsing::Lazy,        candidates: 3 },
    // L3: 10-trit window = chunk (3^10 = 59,049)
    LevelConfig { level: 3, tier_name: "TTC1-3", window_size: 59_049,      min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 59_049,      chain_depth: 32,  parsing: Parsing::Lazy,        candidates: 4 },
    // TTC2: Document tier.
    // L4: 11-trit window = chunk (3^11 = 177,147 = TANS_L). ANS table and chunk are the same object.
    LevelConfig { level: 4, tier_name: "TTC2-1", window_size: 177_147,     min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 177_147,     chain_depth: 32,  parsing: Parsing::Lazy,        candidates: 4 },
    // L5: 12-trit window = chunk (3^12 = 531,441). Most documents fit in 1 chunk.
    LevelConfig { level: 5, tier_name: "TTC2-2", window_size: 531_441,     min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 531_441,     chain_depth: 64,  parsing: Parsing::Lazy,        candidates: 4 },
    // L6: 13-trit window (3^13 = 1,594,323 = hypercube), 12-trit chunk (3^12)
    LevelConfig { level: 6, tier_name: "TTC2-3", window_size: 1_594_323,   min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 531_441,     chain_depth: 128, parsing: Parsing::Lazy,        candidates: 4 },
    // TTC3: Maximum ratio tier. Lazy parsing — BeamOptimal cannot cover 3^13
    // positions with BEAM_WIDTH=8 (good paths pruned, never recover).
    // L7: 14-trit window (3^14), 13-trit chunk (3^13 = hypercube). Single chunk for most documents.
    LevelConfig { level: 7, tier_name: "TTC3-1", window_size: 4_782_969,   min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 1_594_323,   chain_depth: 128, parsing: Parsing::Lazy, candidates: 4 },
    // L8: 15-trit window (3^15), 13-trit chunk (3^13)
    LevelConfig { level: 8, tier_name: "TTC3-2", window_size: 14_348_907,  min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 1_594_323,   chain_depth: 192, parsing: Parsing::Lazy, candidates: 4 },
    // L9: 16-trit window (3^16), 13-trit chunk (3^13)
    LevelConfig { level: 9, tier_name: "TTC3-3", window_size: 43_046_721,  min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 1_594_323,   chain_depth: 256, parsing: Parsing::Lazy, candidates: 4 },
];

#[inline]
pub fn level_config(level: u8) -> TtcResult<&'static LevelConfig> {
    if level >= 1 && level <= 9 { Ok(&LEVEL_CONFIGS[(level - 1) as usize]) }
    else { Err(TtcError::InvalidLevel(level)) }
}

pub fn parse_level(s: &str) -> TtcResult<u8> {
    if let Ok(n) = s.parse::<u8>() { if (1..=9).contains(&n) { return Ok(n); } }
    match s {
        "TTC1-1" => Ok(1), "TTC1-2" => Ok(2), "TTC1-3" => Ok(3),
        "TTC2-1" => Ok(4), "TTC2-2" => Ok(5), "TTC2-3" => Ok(6),
        "TTC3-1" => Ok(7), "TTC3-2" => Ok(8), "TTC3-3" => Ok(9),
        _ => Err(TtcError::InvalidLevel(0)),
    }
}

// ─── CRC32 — Hardware-accelerated on SSE4.2, software fallback ──────────────

/// Software CRC32 (table-based, all platforms)
#[inline]
fn crc32_software(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data { crc = CRC32_TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8); }
    crc ^ 0xFFFF_FFFF
}

/// Hardware CRC32 using SSE4.2 instruction (x86/x86_64 only)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.2")]
unsafe fn crc32_hw(data: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::_mm_crc32_u8;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::_mm_crc32_u8;
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = _mm_crc32_u8(crc, b);
    }
    crc ^ 0xFFFF_FFFF
}

/// Public CRC32: dispatches to hardware path when SSE4.2 available
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse4.2") {
            return unsafe { crc32_hw(data) };
        }
    }
    crc32_software(data)
}

#[inline] pub fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut counts = [0u64; 256];
    for &b in data { counts[b as usize] += 1; }
    let n = data.len() as f64;
    let mut h = 0.0f64;
    for &c in &counts { if c > 0 { let p = c as f64 / n; h -= p * libm::log2(p); } }
    h
}

// ─── GF(3) Delta Encoding (§3.1) ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaFlag(pub u8);
impl DeltaFlag {
    pub const NONE: Self = Self(0b000);
    pub const ORDER1_B: Self = Self(0b001);
    pub const ORDER1_A: Self = Self(0b010);
    pub const ORDER1_C: Self = Self(0b011);
    pub const ORDER2_B: Self = Self(0b100);
    pub const ORDER2_A: Self = Self(0b101);
    pub const ORDER2_C: Self = Self(0b110);
    #[inline] pub fn order(self) -> u8 { match self.0 { 0 => 0, 1..=3 => 1, 4..=6 => 2, _ => 0 } }
    #[inline] pub fn rep(self) -> Option<GfRep> { match self.0 { 0 => None, 1|4 => Some(GfRep::B), 2|5 => Some(GfRep::A), 3|6 => Some(GfRep::C), _ => None } }
    #[inline] pub fn rep_name(self) -> &'static str { match self.rep() { None => "none", Some(GfRep::A) => "A", Some(GfRep::B) => "B", Some(GfRep::C) => "C" } }
}

#[inline] pub fn delta_encode_b(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { out.push(data[i].wrapping_sub(data[i - 1])); }
}
#[inline] pub fn delta_decode_b(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { out.push(out[i - 1].wrapping_add(data[i])); }
}
#[inline] pub fn delta_encode_a(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { out.push(data[i].wrapping_sub(data[i - 1]).wrapping_add(128)); }
}
#[inline] pub fn delta_decode_a(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { let d: i16 = data[i] as i16 - 128; out.push(((out[i-1] as i16 + d + 256) % 256) as u8); }
}
#[inline] pub fn delta_encode_c(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { let d = data[i].wrapping_sub(data[i-1]); out.push(if d == 0 { 255 } else { d.wrapping_sub(1) }); }
}
#[inline] pub fn delta_decode_c(data: &[u8], out: &mut Vec<u8>) {
    out.clear(); if data.is_empty() { return; } out.reserve(data.len()); out.push(data[0]);
    for i in 1..data.len() { let d = (data[i] as u16 + 1) & 0xFF; out.push(((out[i-1] as u16 + d) & 0xFF) as u8); }
}

/// Apply delta encoding, writing result into `out`. `scratch` is reusable
/// workspace for order-2 deltas. Both buffers are pre-allocated by caller
/// with `chunk.len()` capacity and reused across calls — zero reallocation.
fn apply_delta_encode(data: &[u8], flag: DeltaFlag, out: &mut Vec<u8>, scratch: &mut Vec<u8>) {
    match flag.0 {
        0 => { out.clear(); out.extend_from_slice(data); }
        1 => delta_encode_b(data, out),
        2 => delta_encode_a(data, out),
        3 => delta_encode_c(data, out),
        4 => { delta_encode_b(data, scratch); delta_encode_b(scratch, out); }
        5 => { delta_encode_a(data, scratch); delta_encode_a(scratch, out); }
        6 => { delta_encode_c(data, scratch); delta_encode_c(scratch, out); }
        _ => { out.clear(); out.extend_from_slice(data); }
    }
}

fn apply_delta_decode(data: &[u8], flag: DeltaFlag) -> TtcResult<Vec<u8>> {
    let mut buf = Vec::with_capacity(data.len()); let mut buf2 = Vec::with_capacity(data.len());
    match flag.0 {
        0 => Ok(data.to_vec()),
        1 => { delta_decode_b(data, &mut buf); Ok(buf) } 2 => { delta_decode_a(data, &mut buf); Ok(buf) }
        3 => { delta_decode_c(data, &mut buf); Ok(buf) }
        4 => { delta_decode_b(data, &mut buf); delta_decode_b(&buf, &mut buf2); Ok(buf2) }
        5 => { delta_decode_a(data, &mut buf); delta_decode_a(&buf, &mut buf2); Ok(buf2) }
        6 => { delta_decode_c(data, &mut buf); delta_decode_c(&buf, &mut buf2); Ok(buf2) }
        7 => Err(TtcError::InvalidDeltaFlag(7)), _ => Err(TtcError::InvalidDeltaFlag(flag.0)),
    }
}

fn select_delta(chunk: &[u8], mode: CompressionMode) -> DeltaFlag {
    let sample_len = chunk.len().min(512); let sample = &chunk[..sample_len];
    let h_raw = compute_entropy(sample);
    let mut best_flag = DeltaFlag::NONE; let mut best_h = h_raw;
    let mut buf = Vec::with_capacity(sample_len); let mut buf2 = Vec::with_capacity(sample_len);
    for &(flag, enc) in &[(DeltaFlag::ORDER1_B, delta_encode_b as fn(&[u8], &mut Vec<u8>)),
        (DeltaFlag::ORDER1_A, delta_encode_a as fn(&[u8], &mut Vec<u8>)),
        (DeltaFlag::ORDER1_C, delta_encode_c as fn(&[u8], &mut Vec<u8>))] {
        enc(sample, &mut buf); let h = compute_entropy(&buf);
        if h < best_h { best_h = h; best_flag = flag; }
    }
    if matches!(mode, CompressionMode::Audio | CompressionMode::Image) {
        for &(flag, enc) in &[(DeltaFlag::ORDER2_B, delta_encode_b as fn(&[u8], &mut Vec<u8>)),
            (DeltaFlag::ORDER2_A, delta_encode_a as fn(&[u8], &mut Vec<u8>)),
            (DeltaFlag::ORDER2_C, delta_encode_c as fn(&[u8], &mut Vec<u8>))] {
            enc(sample, &mut buf); enc(&buf, &mut buf2);
            let h = compute_entropy(&buf2); if h < best_h { best_h = h; best_flag = flag; }
        }
    }
    best_flag
}

// ─── Trit Encoding (§3.2) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct TritDigits { pub digits: [u8; 6], pub len: u8 }
#[derive(Debug, Clone, Copy)]
pub struct BalancedTritDigits { pub digits: [i8; 6], pub len: u8 }

#[inline] pub fn byte_to_bijective(byte: u8) -> TritDigits {
    let mut n = byte as u32 + 1; let mut buf = [0u8; 6]; let mut len = 0u8;
    while n > 0 { let mut r = n % 3; if r == 0 { r = 3; n = n/3 - 1; } else { n /= 3; } buf[len as usize] = r as u8; len += 1; }
    if len == 0 { buf[0] = 1; len = 1; }
    let (mut i, mut j) = (0usize, (len-1) as usize); while i < j { buf.swap(i,j); i+=1; j-=1; }
    TritDigits { digits: buf, len }
}
#[inline] pub fn bijective_to_byte(td: &TritDigits) -> u8 {
    let mut r = 0u32; for i in 0..(td.len as usize) { r = r*3 + td.digits[i] as u32; } ((r-1) & 0xFF) as u8
}
#[inline] pub fn byte_to_standard(byte: u8) -> TritDigits {
    if byte == 0 { return TritDigits { digits: [0;6], len: 1 }; }
    let mut n = byte as u32; let mut buf = [0u8; 6]; let mut len = 0u8;
    while n > 0 { buf[len as usize] = (n%3) as u8; n /= 3; len += 1; }
    let (mut i, mut j) = (0usize, (len-1) as usize); while i < j { buf.swap(i,j); i+=1; j-=1; }
    TritDigits { digits: buf, len }
}
#[inline] pub fn standard_to_byte(td: &TritDigits) -> u8 {
    let mut r = 0u32; for i in 0..(td.len as usize) { r = r*3 + td.digits[i] as u32; } (r & 0xFF) as u8
}
#[inline] pub fn byte_to_balanced(byte: u8) -> BalancedTritDigits {
    let signed: i16 = if byte <= 127 { byte as i16 } else { byte as i16 - 256 };
    if signed == 0 { return BalancedTritDigits { digits: [0;6], len: 1 }; }
    let mut value = signed; let mut buf = [0i8; 6]; let mut len = 0u8; let mut iters = 0u8;
    while value != 0 && iters < 100 {
        let rem = ((value % 3) + 3) % 3;
        match rem { 0 => { buf[len as usize] = 0; value /= 3; }
            1 => { buf[len as usize] = 1; value = (value-1)/3; }
            2 => { buf[len as usize] = -1; value = (value+1)/3; } _ => unreachable!() }
        len += 1; iters += 1;
    }
    let (mut i, mut j) = (0usize, (len-1) as usize); while i < j { buf.swap(i,j); i+=1; j-=1; }
    BalancedTritDigits { digits: buf, len }
}
#[inline] pub fn balanced_to_byte(td: &BalancedTritDigits) -> u8 {
    let mut v: i16 = 0; let mut m: i16 = 1;
    for k in (0..(td.len as usize)).rev() { v += td.digits[k] as i16 * m; m *= 3; }
    ((v + 256) % 256) as u8
}
#[inline] pub fn trit_count_c(byte: u8) -> u8 { byte_to_bijective(byte).len }
#[inline] pub fn trit_count_b(byte: u8) -> u8 { byte_to_standard(byte).len }
#[inline] pub fn trit_count_a(byte: u8) -> u8 { byte_to_balanced(byte).len }

pub struct TritCostTables { pub rep_a: [u8; 256], pub rep_b: [u8; 256], pub rep_c: [u8; 256] }
impl TritCostTables {
    pub fn new() -> Self {
        let mut t = Self { rep_a: [0;256], rep_b: [0;256], rep_c: [0;256] };
        for b in 0..=255u8 { t.rep_a[b as usize] = trit_count_a(b); t.rep_b[b as usize] = trit_count_b(b); t.rep_c[b as usize] = trit_count_c(b); }
        t
    }
    #[inline] pub fn cost(&self, byte: u8, rep: GfRep) -> u8 {
        match rep { GfRep::A => self.rep_a[byte as usize], GfRep::B => self.rep_b[byte as usize], GfRep::C => self.rep_c[byte as usize] }
    }
    #[inline] pub fn avg_cost(&self, group: &[u8], rep: GfRep) -> f64 {
        if group.is_empty() { return 0.0; }
        group.iter().map(|&b| self.cost(b, rep) as u32).sum::<u32>() as f64 / group.len() as f64
    }
    #[inline] pub fn best_rep(&self, group: &[u8]) -> GfRep {
        let (ca, cb, cc) = (self.avg_cost(group, GfRep::A), self.avg_cost(group, GfRep::B), self.avg_cost(group, GfRep::C));
        if ca <= cb && ca <= cc { GfRep::A } else if cb <= cc { GfRep::B } else { GfRep::C }
    }
}

/// Global singleton trit cost tables — computed once, zero allocation on
/// subsequent calls. Eliminates the ~400 µs cold-start penalty.
fn trit_cost_tables() -> &'static TritCostTables {
    use std::sync::OnceLock;
    static TABLES: OnceLock<TritCostTables> = OnceLock::new();
    TABLES.get_or_init(TritCostTables::new)
}

// ─── Bit I/O ────────────────────────────────────────────────────────────────

pub struct BitWriter { buffer: Vec<u8>, current: u8, bit_pos: u8 }
impl BitWriter {
    #[inline] pub fn new() -> Self { Self { buffer: Vec::new(), current: 0, bit_pos: 0 } }
    #[inline] pub fn with_capacity(cap: usize) -> Self { Self { buffer: Vec::with_capacity(cap), current: 0, bit_pos: 0 } }
    #[inline] pub fn write(&mut self, value: u32, count: u8) {
        for i in (0..count).rev() { self.current |= (((value >> i) & 1) as u8) << (7 - self.bit_pos);
            self.bit_pos += 1; if self.bit_pos == 8 { self.buffer.push(self.current); self.current = 0; self.bit_pos = 0; } }
    }
    #[inline] pub fn write_bit(&mut self, bit: bool) { self.write(bit as u32, 1); }
    #[inline] pub fn bit_count(&self) -> usize { self.buffer.len() * 8 + self.bit_pos as usize }
    pub fn finish(mut self) -> Vec<u8> { if self.bit_pos > 0 { self.buffer.push(self.current); } self.buffer }
    pub fn finish_with_header(self) -> Vec<u8> {
        let bc = self.bit_count() as u32; let bytes = self.finish();
        let mut out = Vec::with_capacity(4 + bytes.len()); out.extend_from_slice(&bc.to_be_bytes()); out.extend_from_slice(&bytes); out
    }
}

pub struct BitReader<'a> { data: &'a [u8], byte_pos: usize, bit_pos: u8 }
impl<'a> BitReader<'a> {
    #[inline] pub fn new(data: &'a [u8]) -> Self { Self { data, byte_pos: 0, bit_pos: 0 } }
    #[inline] pub fn read(&mut self, count: u8) -> u32 {
        let mut v = 0u32; for _ in 0..count { if self.byte_pos >= self.data.len() { return v; }
            let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1; v = (v << 1) | bit as u32;
            self.bit_pos += 1; if self.bit_pos == 8 { self.bit_pos = 0; self.byte_pos += 1; } } v
    }
    #[inline] pub fn read_bit(&mut self) -> bool { self.read(1) != 0 }
    #[inline] pub fn count_leading_zeros(&mut self) -> u32 {
        let mut c = 0u32; while !self.is_exhausted() { if self.read_bit() { return c; } c += 1; } c
    }
    #[inline] pub fn is_exhausted(&self) -> bool { self.byte_pos >= self.data.len() }
}

// ─── Trit Stream I/O (§3.7 — ternary ANS wire format) ──────────────────────
//
// Packs trits 5-per-byte: byte = t₀·81 + t₁·27 + t₂·9 + t₃·3 + t₄
// where each tᵢ ∈ {0, 1, 2}. 3⁵ = 243 ≤ 255, so every packed byte is valid.
// Wire format: [4-byte total trit count (BE)] [packed trit bytes]

/// Divisor table for unpacking 5 trits from a byte: byte / DIV[i] % 3
const TRIT_PACK_DIV: [u8; 5] = [81, 27, 9, 3, 1];

struct TritStreamWriter {
    buffer: Vec<u8>,
    pending: [u8; 5],
    count: u8,
    total_trits: u32,
}

impl TritStreamWriter {
    #[inline]
    fn new() -> Self { Self { buffer: Vec::new(), pending: [0; 5], count: 0, total_trits: 0 } }

    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self { buffer: Vec::with_capacity(cap), pending: [0; 5], count: 0, total_trits: 0 }
    }

    #[inline(always)]
    fn write_trit(&mut self, t: u8) {
        debug_assert!(t < 3, "trit must be 0, 1, or 2");
        self.pending[self.count as usize] = t;
        self.count += 1;
        self.total_trits += 1;
        if self.count == 5 {
            self.flush_group();
        }
    }

    #[inline]
    fn flush_group(&mut self) {
        let packed = self.pending[0] * 81
                   + self.pending[1] * 27
                   + self.pending[2] * 9
                   + self.pending[3] * 3
                   + self.pending[4];
        self.buffer.push(packed);
        self.count = 0;
        self.pending = [0; 5];
    }

    /// Finalize: returns [4-byte total_trits (BE)][packed trit bytes]
    fn finish(mut self) -> Vec<u8> {
        // Flush any remaining trits (padded with 0)
        if self.count > 0 {
            let mut packed = 0u8;
            for i in 0..self.count as usize {
                packed += self.pending[i] * TRIT_PACK_DIV[i];
            }
            self.buffer.push(packed);
        }
        let mut out = Vec::with_capacity(4 + self.buffer.len());
        out.extend_from_slice(&self.total_trits.to_be_bytes());
        out.extend_from_slice(&self.buffer);
        out
    }
}

struct TritStreamReader<'a> {
    data: &'a [u8],
    total_trits: u32,
    byte_pos: usize,
    trit_pos: u8,
    trits_read: u32,
}

impl<'a> TritStreamReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        if data.len() < 4 {
            return Self { data: &[], total_trits: 0, byte_pos: 0, trit_pos: 0, trits_read: 0 };
        }
        let total = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        Self { data: &data[4..], total_trits: total, byte_pos: 0, trit_pos: 0, trits_read: 0 }
    }

    #[inline(always)]
    fn read_trit(&mut self) -> u8 {
        if self.trits_read >= self.total_trits || self.byte_pos >= self.data.len() {
            return 0;
        }
        let byte = self.data[self.byte_pos];
        let trit = (byte / TRIT_PACK_DIV[self.trit_pos as usize]) % 3;
        self.trit_pos += 1;
        self.trits_read += 1;
        if self.trit_pos >= 5 {
            self.trit_pos = 0;
            self.byte_pos += 1;
        }
        trit
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.trits_read >= self.total_trits
    }
}

// ─── Prefix Codes (§2.2, §3.4, §3.5, §3.6) ────────────────────────────────

/// Encoding basis: unique positive Tribonacci values for Zeckendorf representation.
/// Derived from TRIBONACCI_SEQ by skipping T(0)=T(1)=0 and duplicate T(3)=T(2)=1.
/// Used by encode/decode_tribonacci for prefix codes.
const TRIBONACCI_BASIS: [u64; 27] = [
    1, 2, 4, 7, 13, 24, 44, 81, 149, 274,
    504, 927, 1705, 3136, 5768, 10609, 19513, 35890, 66012, 121415,
    223317, 410744, 755476, 1389537, 2555757, 4700770, 8646064,
];

#[inline] pub fn encode_tribonacci(n: u64) -> Vec<bool> {
    if n == 0 { return vec![false]; }
    let mut top = 0; for (i, &v) in TRIBONACCI_BASIS.iter().enumerate() { if v <= n { top = i; } }
    let mut bits = vec![false; top + 1]; let mut rem = n;
    for i in (0..=top).rev() { if TRIBONACCI_BASIS[i] <= rem { bits[i] = true; rem -= TRIBONACCI_BASIS[i]; if rem == 0 { break; } } }
    let msb = bits.iter().rposition(|&b| b).unwrap_or(0); bits[..=msb].to_vec()
}
#[inline] pub fn decode_tribonacci(bits: &[bool]) -> u64 {
    let mut s = 0u64; for (i, &b) in bits.iter().enumerate() { if b && i < TRIBONACCI_BASIS.len() { s += TRIBONACCI_BASIS[i]; } } s
}
#[inline] fn encode_hybrid_prefix(w: &mut BitWriter, value: u64) {
    if value == 0 { w.write(0b00, 2); } else if value <= 3 { w.write(0b01, 2); w.write((value-1) as u32, 2); }
    else if value <= 15 { w.write(0b10, 2); w.write((value-4) as u32, 4); }
    else { w.write(0b11, 2); let code = encode_tribonacci(value); let len = code.len().min(31) as u32;
        w.write(len, 5); for i in 0..(len as usize) { w.write_bit(code[i]); } }
}
#[inline] fn decode_hybrid_prefix(r: &mut BitReader) -> u64 {
    match r.read(2) { 0b00 => 0, 0b01 => r.read(2) as u64 + 1, 0b10 => r.read(4) as u64 + 4,
        0b11 => { let len = r.read(5) as usize; let mut bits = Vec::with_capacity(len);
            for _ in 0..len { bits.push(r.read_bit()); } decode_tribonacci(&bits) } _ => 0 }
}
#[inline] fn encode_elias_gamma(w: &mut BitWriter, n: u64) {
    let value = (n+1).max(1); let bits = 64 - value.leading_zeros();
    for _ in 0..(bits-1) { w.write_bit(false); } w.write(value as u32, bits as u8);
}
#[inline] fn decode_elias_gamma(r: &mut BitReader) -> u64 {
    let zeros = r.count_leading_zeros(); let lower = r.read(zeros as u8); ((1u64 << zeros) | lower as u64) - 1
}
#[inline(always)] fn encode_rice(w: &mut BitWriter, n: u64, m: u8) {
    let q = n >> m; for _ in 0..q { w.write_bit(true); } w.write_bit(false);
    w.write((n & ((1u64 << m) - 1)) as u32, m);
}
#[inline(always)] fn decode_rice(r: &mut BitReader, m: u8) -> u64 {
    let mut q = 0u64; while r.read_bit() { q += 1; } let rem = r.read(m) as u64; (q << m) | rem
}
fn compute_initial_rice_m(tokens: &[Token]) -> u8 {
    let mut sum = 0u64; let mut count = 0u32;
    for t in tokens { if let Token::Match { dist, .. } = t { sum += *dist as u64; count += 1; if count >= 128 { break; } } }
    if count == 0 { return 4; } let mean = sum / count as u64;
    if mean == 0 { return 1; } ((64 - mean.leading_zeros()).saturating_sub(1) as u8).clamp(1, 8)
}

// ─── Ternary rANS + Hybrid Rice (§3.7, TM-2026-017) ────────────────────────
//
// CORRECTED in v4.1: Uses rANS encoding formula with base-3 I/O.
//
// The v4 tANS approach had a mathematical flaw: after base-3 normalization
// (while state >= 3·fs), state ∈ [fs, 3·fs−1], giving offset ∈ [0, 2·fs−1].
// But the symbol position table only has fs entries. When offset >= fs, the
// encoding step was silently skipped — corrupting the stream on real data.
//
// Root cause: The tANS position-table pattern assumes offset range == table
// size, which only holds for base-2 (offset ∈ [0, fs−1] after `while >= 2·fs`).
// For base-3, the offset range is 2× the table size.
//
// Fix: Replace tANS position-table encoding with the rANS encoding formula,
// which is provably correct for any radix:
//
//   Normalize: while state >= 3·fs { emit trit(state mod 3); state /= 3 }
//   Encode:    state = (state / fs) × L + cum[s] + (state mod fs)
//   Decode:    slot = state mod L; s = spread[slot];
//              state = fs × (state / L) + slot − cum[s]
//   Renorm:    while state < L { trit = read(); state = state × 3 + trit }
//
// Proof of correctness:
//   After normalize: state ∈ [fs, 3·fs − 1]
//   After encode:    state/fs ∈ [1, 2], so
//                    new_state ∈ [L + cum[s], 2·L + cum[s] + fs − 1]
//                    ⊆ [L, 3·L − 1]  ✓  (operating range)
//   After decode:    state/L ∈ [1, 2], slot ∈ [cum[s], cum[s]+fs−1], so
//                    new_state ∈ [fs, 3·fs − 1]  ✓  (pre-renorm range)
//   After renorm:    state ∈ [L, 3·L − 1]  ✓
//
// The trit stream is packed 5-per-byte (3⁵=243 ≤ 255) for wire efficiency.
// L = 3¹¹ = 177,147 remains naturally ternary-aligned.
//
// Hybrid architecture: literals, runs, match lengths → ternary rANS;
// exact distances → lossless Rice side channel (binary, per §3.6).

struct TansFreqTable {
    entries: Vec<(u16, u32)>,
    fnorm_lookup: [u32; TANS_ALPHABET],
    cum: [u32; TANS_ALPHABET],
}

impl TansFreqTable {
    fn build(tokens: &[Token], _window_size: usize) -> Self {
        let mut counts = [0u32; TANS_ALPHABET];
        for tok in tokens {
            match tok {
                Token::Literal(v) => counts[*v as usize] += 1,
                Token::Run { byte, length } => {
                    counts[*byte as usize] += 1;
                    let sym = (256 + (*length).min(255)) as usize;
                    if sym < TANS_ALPHABET { counts[sym] += 1; }
                }
                Token::Match { length, .. } => {
                    let len_sym = (512 + (*length).min(255)) as usize;
                    if len_sym < TANS_ALPHABET { counts[len_sym] += 1; }
                }
            }
        }
        counts[TANS_EOB as usize] = counts[TANS_EOB as usize].max(1);

        let total_raw: u64 = counts.iter().map(|&c| c as u64).sum();
        let l = TANS_L as u64;
        let mut fnorm = [0u32; TANS_ALPHABET];
        let mut sum = 0u32;
        for i in 0..TANS_ALPHABET {
            if counts[i] > 0 { fnorm[i] = ((counts[i] as u64 * l / total_raw.max(1)) as u32).max(1); sum += fnorm[i]; }
        }
        let target = TANS_L;
        while sum != target {
            if sum > target { let mut mi = 0; let mut mv = 0u32;
                for i in 0..TANS_ALPHABET { if fnorm[i] > mv && fnorm[i] > 1 { mv = fnorm[i]; mi = i; } }
                fnorm[mi] -= 1; sum -= 1;
            } else { let mut mi = 0; let mut mv = u32::MAX;
                for i in 0..TANS_ALPHABET { if fnorm[i] > 0 && fnorm[i] < mv { mv = fnorm[i]; mi = i; } }
                fnorm[mi] += 1; sum += 1;
            }
        }
        let mut entries: Vec<(u16, u32)> = Vec::new();
        for i in 0..TANS_ALPHABET { if fnorm[i] > 0 { entries.push((i as u16, fnorm[i])); } }
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        // Cumulative frequencies: sorted by symbol ID (ascending) for rANS.
        // Each symbol s occupies the contiguous range [cum[s], cum[s]+fs-1].
        let mut cum = [0u32; TANS_ALPHABET];
        let mut c = 0u32;
        for i in 0..TANS_ALPHABET {
            cum[i] = c;
            c += fnorm[i];
        }
        Self { entries, fnorm_lookup: fnorm, cum }
    }
    #[inline] fn fnorm(&self, sym: u16) -> u32 { self.fnorm_lookup[sym as usize] }
}

/// Build contiguous-range spread table for rANS decode.
/// spread[slot] = symbol s such that cum[s] <= slot < cum[s] + fs.
fn rans_build_spread(freq: &TansFreqTable) -> Vec<u16> {
    let l = TANS_L as usize;
    let mut spread = vec![0u16; l];
    for i in 0..TANS_ALPHABET {
        let fs = freq.fnorm_lookup[i];
        let start = freq.cum[i] as usize;
        for j in 0..fs as usize {
            if start + j < l {
                spread[start + j] = i as u16;
            }
        }
    }
    spread
}

/// Ternary rANS encode (TM-2026-017 §3.7 compliant, v4.1 corrected).
///
/// Uses rANS encoding formula with base-3 normalization:
///   Normalize: while state >= 3·fs { emit trit(state mod 3); state /= 3 }
///   Encode:    state = (state / fs) × L + cum[s] + (state mod fs)
///
/// Returns (final_state, packed_trit_stream, exact_distances).
fn tans_encode(tokens: &[Token], freq: &TansFreqTable, _window_size: usize) -> (u32, Vec<u8>, Vec<usize>) {
    let l = TANS_L as usize;
    let mut symbols: Vec<u16> = Vec::with_capacity(tokens.len() * 2 + 1);
    let mut distances: Vec<usize> = Vec::with_capacity(tokens.len());

    for tok in tokens {
        match tok {
            Token::Literal(v) => symbols.push(*v as u16),
            Token::Run { byte, length } => {
                symbols.push(*byte as u16);
                symbols.push((256 + (*length).min(255)) as u16);
            }
            Token::Match { dist, length } => {
                symbols.push((512 + (*length).min(255)) as u16);
                distances.push(*dist);
            }
        }
    }
    symbols.push(TANS_EOB);

    // Encode symbols in REVERSE so decoder produces forward order.
    let mut state: usize = l; // initial state = L
    let mut trits: Vec<u8> = Vec::with_capacity(symbols.len() * 3);

    for &s in symbols.iter().rev() {
        let fs = freq.fnorm(s) as usize;
        if fs == 0 { continue; }
        let cum_s = freq.cum[s as usize] as usize;

        // Ternary normalization: emit trits while state >= 3·fs
        // After loop: state ∈ [fs, 3·fs − 1]
        while state >= 3 * fs {
            trits.push((state % 3) as u8);
            state /= 3;
        }

        // rANS encoding formula (provably correct for any radix):
        // state = (state / fs) × L + cum[s] + (state mod fs)
        // After: state ∈ [L, 3·L − 1]
        state = (state / fs) * l + cum_s + (state % fs);
    }

    trits.reverse();

    // Pack trits 5-per-byte using TritStreamWriter
    let mut writer = TritStreamWriter::with_capacity((trits.len() + 4) / 5);
    for &t in &trits {
        writer.write_trit(t);
    }
    let packed = writer.finish();

    (state as u32, packed, distances)
}

/// Ternary rANS decode (TM-2026-017 §3.7 compliant, v4.1 corrected).
///
/// Uses rANS decoding formula:
///   slot  = state mod L;  s = spread[slot]
///   state = fs × (state / L) + slot − cum[s]
///   Renormalize: while state < L { trit = read(); state = state × 3 + trit }
fn tans_decode(
    freq: &TansFreqTable, spread: &[u16],
    initial_state: u32, packed_trits: &[u8], distances: &[usize],
) -> Vec<Token> {
    let mut trit_reader = TritStreamReader::new(packed_trits);
    let l = TANS_L as usize;

    let mut state = initial_state as usize;
    let mut tokens: Vec<Token> = Vec::new();
    let mut dist_idx = 0usize;

    loop {
        // Decode symbol from state
        let slot = state % l;
        let s = spread[slot];
        if s == TANS_EOB { break; }

        let fs = freq.fnorm(s) as usize;
        let cum_s = freq.cum[s as usize] as usize;

        // rANS decode formula: new_state = fs × (state / L) + slot − cum[s]
        state = fs * (state / l) + slot - cum_s;

        // Ternary renormalization: read trits until state >= L
        while state < l {
            let t = trit_reader.read_trit() as usize;
            state = state * 3 + t;
        }

        // Emit token
        if s <= 255 {
            tokens.push(Token::Literal(s as u8));
        } else if s >= 256 && s <= 511 {
            let run_len = (s - 256) as usize;
            if let Some(Token::Literal(b)) = tokens.last() {
                let byte = *b; tokens.pop();
                tokens.push(Token::Run { byte, length: run_len });
            }
        } else if s >= 512 && s <= 767 {
            let match_len = (s - 512) as usize;
            let dist = if dist_idx < distances.len() { distances[dist_idx] } else { 0 };
            dist_idx += 1;
            tokens.push(Token::Match { dist, length: match_len });
        }
    }
    tokens
}

/// Serialize hybrid ternary rANS: compact freq table + trit stream + Rice side channel.
/// v4.2: Frequency table uses delta-varint encoding (50-60% smaller than fixed-width).
fn serialize_tans(tokens: &[Token], window_size: usize) -> Vec<u8> {
    let freq = TansFreqTable::build(tokens, window_size);
    let (state, packed_trits, distances) = tans_encode(tokens, &freq, window_size);

    let mut out = Vec::with_capacity(
        2 + freq.entries.len() * 3 + 3 + 4 + packed_trits.len() + 4 + distances.len() * 4
    );
    // Compact frequency table: [2-byte count][delta-varint sym, varint freq pairs]
    let s = freq.entries.len() as u16;
    out.extend_from_slice(&s.to_be_bytes());
    let mut prev_sym: u16 = 0;
    for &(sym, fnorm) in &freq.entries {
        encode_varint(&mut out, sym.wrapping_sub(prev_sym) as u64);
        encode_varint(&mut out, fnorm as u64);
        prev_sym = sym;
    }
    // Initial state (3 bytes)
    out.push((state >> 16) as u8); out.push((state >> 8) as u8); out.push(state as u8);
    // Ternary rANS trit stream length + data
    out.extend_from_slice(&(packed_trits.len() as u32).to_be_bytes());
    out.extend_from_slice(&packed_trits);
    // Rice side channel
    out.extend_from_slice(&(distances.len() as u32).to_be_bytes());
    if !distances.is_empty() {
        let mean_dist: u64 = distances.iter().map(|&d| d as u64).sum::<u64>() / distances.len().max(1) as u64;
        let rice_m = if mean_dist == 0 { 1u8 } else { ((64 - mean_dist.leading_zeros()).saturating_sub(1) as u8).clamp(1, 8) };
        out.push(rice_m);
        let mut w = BitWriter::with_capacity(distances.len() * 4);
        for &d in &distances { encode_rice(&mut w, d as u64, rice_m); }
        let rice_payload = w.finish_with_header();
        out.extend_from_slice(&rice_payload);
    }
    out
}

/// Deserialize hybrid ternary rANS: compact freq table + trit stream + Rice side channel.
/// v4.2: Frequency table uses delta-varint encoding matching serialize_tans.
fn deserialize_tans(payload: &[u8], _window_size: usize) -> TtcResult<Vec<Token>> {
    if payload.len() < 2 { return Err(TtcError::DecompressionError("tANS payload too short".into())); }
    let s = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let mut pos = 2;

    // Read compact frequency table (delta-varint encoded)
    let mut entries = Vec::with_capacity(s);
    let mut fnorm = [0u32; TANS_ALPHABET];
    let mut prev_sym: u16 = 0;
    for _ in 0..s {
        if pos >= payload.len() { return Err(TtcError::TruncatedPayload); }
        let (sym_delta, br1) = decode_varint(&payload[pos..]); pos += br1;
        let sym = prev_sym.wrapping_add(sym_delta as u16);
        prev_sym = sym;
        if pos >= payload.len() { return Err(TtcError::TruncatedPayload); }
        let (f, br2) = decode_varint(&payload[pos..]); pos += br2;
        fnorm[sym as usize] = f as u32;
        entries.push((sym, f as u32));
    }

    if pos + 3 > payload.len() { return Err(TtcError::TruncatedPayload); }
    let init = (payload[pos] as u32) << 16 | (payload[pos+1] as u32) << 8 | payload[pos+2] as u32;
    pos += 3;

    // Read ternary rANS trit stream
    if pos + 4 > payload.len() { return Err(TtcError::TruncatedPayload); }
    let trits_len = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]) as usize;
    pos += 4;
    if pos + trits_len > payload.len() { return Err(TtcError::TruncatedPayload); }
    let packed_trits = &payload[pos..pos + trits_len];
    pos += trits_len;

    // Read Rice distance side channel
    let mut distances: Vec<usize> = Vec::new();
    if pos + 4 <= payload.len() {
        let dist_count = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]) as usize;
        pos += 4;
        if dist_count > 0 && pos + 1 <= payload.len() {
            let rice_m = payload[pos]; pos += 1;
            if pos + 4 <= payload.len() {
                let _bit_count = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]);
                pos += 4;
                let mut reader = BitReader::new(&payload[pos..]);
                distances.reserve(dist_count);
                for _ in 0..dist_count {
                    distances.push(decode_rice(&mut reader, rice_m) as usize);
                }
            }
        }
    }

    // Rebuild cumulative frequencies and spread for decode
    let mut cum = [0u32; TANS_ALPHABET];
    let mut c = 0u32;
    for i in 0..TANS_ALPHABET { cum[i] = c; c += fnorm[i]; }

    let freq = TansFreqTable { entries, fnorm_lookup: fnorm, cum };
    let spread = rans_build_spread(&freq);
    Ok(tans_decode(&freq, &spread, init, packed_trits, &distances))
}


// ─── Pre-Compressed Detection (§4.3) ────────────────────────────────────────

fn is_pre_compressed(data: &[u8]) -> bool {
    if data.len() < 5 { return false; }
    let sample_len = data.len().min(1024);
    if compute_entropy(&data[..sample_len]) > ENTROPY_GATE { return true; }
    if data.starts_with(b"%PDF-") { return true; }
    if data.starts_with(&[0x50, 0x4B]) { return true; }
    if data.starts_with(&[0x1F, 0x8B]) { return true; }
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) { return true; }
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF { return true; }
    if data.starts_with(&[0x37, 0x7A, 0xBC, 0xAF]) { return true; }
    if data.starts_with(b"Rar!") { return true; }
    if data.starts_with(b"ID3") { return true; }
    if data.len() >= 12 && &data[4..12] == b"ftypavif" { return true; }
    false
}

// ─── GURFT Adaptive Engine (§6) ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GurftResult { pub tau: f64, pub delta: f64, pub entropy: f64, pub periodicity: f64, pub salvi_resonance: bool }
impl Default for GurftResult { fn default() -> Self { Self { tau: 0.0, delta: 0.0, entropy: 0.0, periodicity: 0.0, salvi_resonance: false } } }

fn compute_torsion_region(data: &[u8]) -> f64 {
    let n = data.len(); if n == 0 { return 0.0; }
    let mut total = 0.0f64;
    for k in 1..=13u32 { let mut rs = 0.0f64; let mut is_v = 0.0f64;
        let freq = 2.0 * core::f64::consts::PI * (k as f64 / 13.0);
        for (i, &b) in data.iter().enumerate() { let angle = freq * i as f64;
            rs += b as f64 * libm::cos(angle); is_v += b as f64 * libm::sin(angle); }
        let norm = libm::sqrt(rs * rs + is_v * is_v) / (n as f64 * 128.0);
        total += if norm > 1.0 { 1.0 } else { norm }; }
    total / 13.0
}
fn compute_delta_region(data: &[u8]) -> f64 {
    let n = data.len().min(512); if n < 14 { return 0.0; }
    let mut cross = 0.0f64; let mut sq = 0.0f64;
    for i in 0..(n-13) { cross += data[i] as f64 * data[i+13] as f64; sq += data[i] as f64 * data[i] as f64; }
    (cross / (sq + 1e-10)).clamp(0.0, 1.0)
}
fn compute_periodicity(data: &[u8]) -> f64 {
    let n = data.len().min(512); let mut best = 0.0f64;
    for &p in &[28usize, 364] { if n <= p { continue; }
        let cnt = (0..(n-p)).filter(|&i| (data[i] as i16 - data[i+p] as i16).unsigned_abs() < 16).count();
        let s = cnt as f64 / n as f64; if s > best { best = s; } } best
}
fn compute_salvi_resonance(data: &[u8]) -> bool {
    let n = data.len().min(512);
    for &p in &[5usize, 25, 125] { if n <= p { continue; }
        let cnt = (0..(n-p)).filter(|&i| (data[i] as i16 - data[i+p] as i16).unsigned_abs() < 8).count();
        if cnt as f64 / n as f64 > 0.80 { return true; } } false
}
/// GURFT analysis with entropy-only fast-path.
/// If entropy > 6.0 bits/byte, base-3 is always optimal — skip the expensive
/// torsion/delta/periodicity computation (~13,000 trig calls per region).
pub fn gurft_analyze(data: &[u8]) -> GurftResult {
    let sample = &data[..data.len().min(512)];
    let h = compute_entropy(sample);
    // Fast-path: high entropy → base-3 guaranteed, skip trig-heavy analysis
    if h > 6.0 {
        return GurftResult { tau: 0.0, delta: 0.0, entropy: h,
            periodicity: 0.0, salvi_resonance: false };
    }
    if data.len() < 1024 {
        return GurftResult { tau: compute_torsion_region(sample), delta: compute_delta_region(sample),
            entropy: h, periodicity: compute_periodicity(sample), salvi_resonance: compute_salvi_resonance(sample) };
    }
    let mid = data.len() / 2; let ra = &data[..512];
    let rb = &data[(mid.saturating_sub(256))..(mid+256).min(data.len())]; let rc = &data[data.len().saturating_sub(512)..];
    GurftResult { tau: 0.3*compute_torsion_region(ra) + 0.4*compute_torsion_region(rb) + 0.3*compute_torsion_region(rc),
        delta: 0.3*compute_delta_region(ra) + 0.4*compute_delta_region(rb) + 0.3*compute_delta_region(rc),
        entropy: 0.3*compute_entropy(ra) + 0.4*compute_entropy(rb) + 0.3*compute_entropy(rc),
        periodicity: compute_periodicity(data), salvi_resonance: compute_salvi_resonance(data) }
}
fn select_base(g: &GurftResult, mode: CompressionMode) -> u16 {
    if mode == CompressionMode::Basic { return 3; } if g.tau < TAU_HARMONIC { return 3; }
    let allowed = mode.allowed_bases();
    let cand = if g.tau >= TAU_HARMONIC && g.delta < DELTA_HOLOGRAPHIC { 13u16 }
        else if g.tau >= TAU_HOLOGRAPHIC && g.delta >= DELTA_HOLOGRAPHIC {
            if g.salvi_resonance && g.tau > TAU_RESONANCE { 70 } else if g.periodicity > 0.7 { 364 } else { 28 }
        } else { 3 };
    if allowed.contains(&cand) { cand } else { 3 }
}

// ─── Base-N Packing (§4.11) ────────────────────────────────────────────────

fn pack_bytes_tribonacci(data: &[u8]) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(data.len() * 2);
    for &b in data { let td = byte_to_bijective(b);
        for i in 0..(td.len as usize) { w.write(match td.digits[i] { 1=>0b00u32, 2=>0b01, 3=>0b10, _=>0b11 }, 2); } }
    w.finish_with_header()
}
fn pack_bytes_base_n(data: &[u8], base: u16) -> Vec<u8> {
    if data.is_empty() { return vec![0,0,0,0]; }
    let mut big = data.to_vec(); let mut digits = Vec::new();
    while !big.is_empty() && !(big.len() == 1 && big[0] == 0) {
        let mut rem = 0u32; let mut new_big = Vec::new();
        for &byte in &big { let val = rem * 256 + byte as u32; let q = val / base as u32; rem = val % base as u32;
            if !new_big.is_empty() || q > 0 { new_big.push(q as u8); } }
        digits.push(rem as u16); big = new_big; }
    digits.reverse(); let bpd = (16 - (base-1).leading_zeros()) as u8;
    let mut w = BitWriter::with_capacity(digits.len()*2); encode_hybrid_prefix(&mut w, digits.len() as u64);
    for &d in &digits { w.write(d as u32, bpd); } w.finish_with_header()
}
fn try_and_compare_base(chunk: &[u8], g: &GurftResult, mode: CompressionMode) -> u16 {
    let cand = select_base(g, mode); if cand == 3 { return 3; }
    let sample = &chunk[..chunk.len().min(512)];
    if pack_bytes_tribonacci(sample).len() <= pack_bytes_base_n(sample, cand).len() { 3 } else { cand }
}

// ─── Domain Preprocessing (§3.8, §3.9) ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainTransform(pub u8);
impl DomainTransform {
    pub const NONE: Self = Self(0); pub const AUDIO_LP: Self = Self(1); pub const IMAGE_MED: Self = Self(2);
    pub const GENOMIC: Self = Self(3); pub const SOURCE: Self = Self(4); pub const LOG: Self = Self(5); pub const STRUCTURED: Self = Self(6);
}

fn audio_lp_encode(data: &[u8]) -> (Vec<u8>, [i16; 4]) {
    const P: usize = 4;
    if data.len() <= P { return (data.to_vec(), [0i16; 4]); }
    let slen = data.len().min(1024); let samples: Vec<f64> = data[..slen].iter().map(|&b| b as f64).collect();
    let mut r = [0.0f64; P+1]; for lag in 0..=P { for i in lag..slen { r[lag] += samples[i] * samples[i-lag]; } }
    let mut a = [0.0f64; P]; let mut e = r[0]; if e <= 0.0 { return (data.to_vec(), [0;4]); }
    for i in 0..P { let mut lambda = 0.0; for j in 0..i { lambda += a[j] * r[i-j]; }
        lambda = (r[i+1] - lambda) / e; let mut a_new = a; a_new[i] = lambda;
        for j in 0..i { a_new[j] = a[j] - lambda * a[i-1-j]; } a = a_new; e *= 1.0 - lambda*lambda; if e <= 0.0 { break; } }
    let coeffs = [(a[0]*32767.0).clamp(-32768.0,32767.0) as i16, (a[1]*32767.0).clamp(-32768.0,32767.0) as i16,
        (a[2]*32767.0).clamp(-32768.0,32767.0) as i16, (a[3]*32767.0).clamp(-32768.0,32767.0) as i16];
    let af = [coeffs[0] as f64/32767.0, coeffs[1] as f64/32767.0, coeffs[2] as f64/32767.0, coeffs[3] as f64/32767.0];
    let mut res = Vec::with_capacity(data.len()); for i in 0..P { res.push(data[i]); }
    for i in P..data.len() { let pred = af[0]*data[i-1] as f64 + af[1]*data[i-2] as f64 + af[2]*data[i-3] as f64 + af[3]*data[i-4] as f64;
        res.push((data[i] as i16).wrapping_sub(pred.round() as i16) as u8); } (res, coeffs)
}
fn audio_lp_decode(res: &[u8], coeffs: &[i16; 4]) -> Vec<u8> {
    const P: usize = 4; if res.len() <= P { return res.to_vec(); }
    let af = [coeffs[0] as f64/32767.0, coeffs[1] as f64/32767.0, coeffs[2] as f64/32767.0, coeffs[3] as f64/32767.0];
    let mut out = Vec::with_capacity(res.len()); for i in 0..P { out.push(res[i]); }
    for i in P..res.len() { let pred = af[0]*out[i-1] as f64 + af[1]*out[i-2] as f64 + af[2]*out[i-3] as f64 + af[3]*out[i-4] as f64;
        out.push((res[i] as i16).wrapping_add(pred.round() as i16) as u8); } out
}
fn image_med_encode(data: &[u8], width: usize) -> Vec<u8> {
    if width == 0 || data.is_empty() { return data.to_vec(); } let height = data.len() / width;
    let mut res = Vec::with_capacity(data.len());
    for r in 0..height { for c in 0..width { let idx = r*width+c;
        if r == 0 || c == 0 { res.push(if idx == 0 { data[0] } else { data[idx].wrapping_sub(data[idx-1]) }); }
        else { let a = data[r*width+c-1] as i16; let b = data[(r-1)*width+c] as i16; let cc = data[(r-1)*width+c-1] as i16;
            let p = if cc >= a.max(b) { a.min(b) } else if cc <= a.min(b) { a.max(b) } else { a+b-cc };
            res.push((data[idx] as i16 - p) as u8); } } }
    for i in (height*width)..data.len() { res.push(data[i]); } res
}
fn image_med_decode(res: &[u8], width: usize) -> Vec<u8> {
    if width == 0 || res.is_empty() { return res.to_vec(); } let height = res.len() / width;
    let mut out: Vec<u8> = Vec::with_capacity(res.len());
    for r in 0..height { for c in 0..width { let idx = r*width+c;
        if r == 0 || c == 0 { out.push(if idx == 0 { res[0] } else { out[idx-1].wrapping_add(res[idx]) }); }
        else { let a = out[r*width+c-1] as i16; let b = out[(r-1)*width+c] as i16; let cc = out[(r-1)*width+c-1] as i16;
            let p = if cc >= a.max(b) { a.min(b) } else if cc <= a.min(b) { a.max(b) } else { a+b-cc };
            out.push((res[idx] as i16 + p) as u8); } } }
    for i in (height*width)..res.len() { out.push(res[i]); } out
}
fn genomic_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()/4+data.len()); let mut i = 0;
    while i < data.len() { let b = data[i].to_ascii_uppercase();
        let c0 = match b { b'A'=>0u8, b'C'=>1, b'G'=>2, b'T'=>3, _ => { out.push(0xFF); out.push(data[i]); i+=1; continue; } };
        let mut packed = c0 << 6; let mut cnt = 1;
        while cnt < 4 && i+cnt < data.len() { match data[i+cnt].to_ascii_uppercase() {
            b'A' => packed |= 0 << (6-cnt*2), b'C' => packed |= 1 << (6-cnt*2),
            b'G' => packed |= 2 << (6-cnt*2), b'T' => packed |= 3 << (6-cnt*2), _ => break }; cnt += 1; }
        if cnt == 4 { out.push(packed); i += 4; } else { out.push(0xFF); out.push(data[i]); i += 1; } } out
}
fn genomic_decode(data: &[u8]) -> Vec<u8> {
    let map = [b'A', b'C', b'G', b'T']; let mut out = Vec::with_capacity(data.len()*4); let mut i = 0;
    while i < data.len() { if data[i] == 0xFF { i+=1; if i<data.len() { out.push(data[i]); i+=1; } }
        else { let p = data[i]; out.push(map[((p>>6)&3) as usize]); out.push(map[((p>>4)&3) as usize]);
            out.push(map[((p>>2)&3) as usize]); out.push(map[(p&3) as usize]); i+=1; } } out
}

const SOURCE_KEYWORDS: &[&[u8]] = &[
    b"function", b"return", b"if", b"else", b"for", b"while", b"do", b"class", b"import", b"export",
    b"const", b"let", b"var", b"true", b"false", b"null", b"void", b"int", b"string", b"bool",
    b"float", b"double", b"char", b"byte", b"long", b"short", b"unsigned", b"public", b"private",
    b"protected", b"static", b"final", b"abstract", b"interface", b"enum", b"struct", b"type",
    b"trait", b"impl", b"fn", b"pub", b"mod", b"use", b"crate", b"self", b"super", b"match",
    b"case", b"switch", b"break", b"continue", b"default", b"try", b"catch", b"throw", b"throws",
    b"finally", b"async", b"await", b"yield", b"new", b"delete", b"typeof", b"instanceof", b"in",
    b"of", b"from", b"as", b"with", b"this", b"package", b"extends", b"implements", b"override",
    b"virtual", b"const_cast", b"template", b"namespace", b"using", b"include", b"define", b"ifdef",
    b"ifndef", b"endif", b"pragma", b"extern", b"volatile", b"register", b"inline", b"goto",
    b"sizeof", b"nullptr", b"auto", b"decltype", b"constexpr", b"noexcept", b"lambda", b"def",
    b"elif", b"pass", b"raise", b"except", b"print", b"println", b"printf", b"sprintf", b"fprintf",
    b"malloc", b"free", b"realloc", b"calloc",
    b"->", b"=>", b"==", b"!=", b"<=", b">=", b"&&", b"||", b"<<", b">>", b"++", b"--",
    b"+=", b"-=", b"*=", b"/=", b"{", b"}", b"(", b")", b"[", b"]", b";", b":", b",",
];
fn source_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()); let mut i = 0;
    'outer: while i < data.len() { for (idx, kw) in SOURCE_KEYWORDS.iter().enumerate() { if idx >= 127 { break; }
        if data[i..].starts_with(kw) { let after = i+kw.len();
            let is_word = kw.len() <= 2 || after >= data.len() || !(data[after].is_ascii_alphanumeric() || data[after] == b'_');
            if is_word { out.push((idx+1) as u8); i += kw.len(); continue 'outer; } } }
        out.push(0x80); out.push(data[i]); i += 1; } out
}
fn source_decode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()*4); let mut i = 0;
    while i < data.len() { if data[i] == 0x80 { i+=1; if i<data.len() { out.push(data[i]); i+=1; } }
        else if data[i] >= 0x01 && data[i] <= 0x7F { let idx = (data[i]-1) as usize;
            if idx < SOURCE_KEYWORDS.len() { out.extend_from_slice(SOURCE_KEYWORDS[idx]); } i+=1; }
        else { out.push(data[i]); i+=1; } } out
}

fn log_encode(data: &[u8]) -> Vec<u8> {
    let text = data; let mut out = Vec::with_capacity(data.len());
    let mut prev_ts: u64 = 0; let mut service_dict: Vec<Vec<u8>> = Vec::new();
    let mut line_start = 0;
    while line_start < text.len() {
        let line_end = text[line_start..].iter().position(|&b| b == b'\n').map(|p| line_start+p).unwrap_or(text.len());
        let line = &text[line_start..line_end];
        if line.is_empty() { if line_end < text.len() { out.push(0x04); out.push(0); out.push(0); }
            line_start = line_end + 1; continue; }
        let (ts_end, ts_value) = detect_timestamp(line);
        if ts_end > 0 { out.push(0x01);
            if prev_ts == 0 { out.extend_from_slice(&ts_value.to_be_bytes()); }
            else { encode_varint(&mut out, ts_value.saturating_sub(prev_ts)); } prev_ts = ts_value; }
        let remaining = if ts_end > 0 && ts_end < line.len() { &line[ts_end..] } else { line };
        let remaining = trim_leading_space(remaining);
        let (level_code, after_level) = detect_log_level(remaining);
        if level_code > 0 { out.push(0x02); out.push(level_code); }
        let rest = if level_code > 0 { trim_leading_space(after_level) } else { remaining };
        let (service_idx, after_service) = detect_service_name(rest, &mut service_dict);
        if let Some(idx) = service_idx { out.push(0x03); out.extend_from_slice(&(idx as u16).to_be_bytes()); }
        let msg = if service_idx.is_some() { after_service } else { rest };
        if !msg.is_empty() { out.push(0x04); out.extend_from_slice(&(msg.len() as u16).to_be_bytes()); out.extend_from_slice(msg); }
        line_start = if line_end < text.len() { line_end + 1 } else { text.len() };
    }
    let mut result = Vec::with_capacity(4 + service_dict.iter().map(|s| 2+s.len()).sum::<usize>() + out.len());
    result.extend_from_slice(&(service_dict.len() as u16).to_be_bytes());
    for svc in &service_dict { result.extend_from_slice(&(svc.len() as u16).to_be_bytes()); result.extend_from_slice(svc); }
    result.extend_from_slice(&out); result
}
fn log_decode(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return data.to_vec(); } let mut pos = 0;
    let svc_count = u16::from_be_bytes([data[pos], data[pos+1]]) as usize; pos += 2;
    let mut services: Vec<Vec<u8>> = Vec::with_capacity(svc_count);
    for _ in 0..svc_count { if pos+2 > data.len() { break; }
        let slen = u16::from_be_bytes([data[pos], data[pos+1]]) as usize; pos += 2;
        if pos+slen > data.len() { break; } services.push(data[pos..pos+slen].to_vec()); pos += slen; }
    let mut out = Vec::with_capacity(data.len()*2); let mut prev_ts: u64 = 0; let mut line_has_content = false;
    while pos < data.len() { let marker = data[pos]; pos += 1;
        match marker {
            0x01 => { if prev_ts == 0 { if pos+8>data.len() { break; }
                prev_ts = u64::from_be_bytes([data[pos],data[pos+1],data[pos+2],data[pos+3],data[pos+4],data[pos+5],data[pos+6],data[pos+7]]); pos+=8;
                } else { let (delta, br) = decode_varint(&data[pos..]); pos += br; prev_ts += delta; }
                write_timestamp(&mut out, prev_ts); out.push(b' '); line_has_content = true; }
            0x02 => { if pos >= data.len() { break; } let level = data[pos]; pos += 1;
                let name = match level { 1=>b"DEBUG" as &[u8], 2=>b"INFO", 3=>b"WARN", 4=>b"ERROR", 5=>b"FATAL", _=>b"UNKNOWN" };
                out.extend_from_slice(name); out.push(b' '); line_has_content = true; }
            0x03 => { if pos+2 > data.len() { break; } let idx = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
                if idx < services.len() { out.extend_from_slice(&services[idx]); } out.extend_from_slice(b": "); line_has_content = true; }
            0x04 => { if pos+2 > data.len() { break; } let mlen = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
                if mlen == 0 { if line_has_content { out.push(b'\n'); } out.push(b'\n'); line_has_content = false; }
                else { let end = (pos+mlen).min(data.len()); out.extend_from_slice(&data[pos..end]); out.push(b'\n'); pos = end; line_has_content = false; } }
            0x05 => { out.extend_from_slice(&data[pos..]); pos = data.len(); }
            _ => {} } } out
}
fn detect_timestamp(line: &[u8]) -> (usize, u64) {
    if line.len() >= 19 && line[4]==b'-' && line[7]==b'-' && (line[10]==b'T'||line[10]==b' ') && line[13]==b':' && line[16]==b':' {
        if let Ok(s) = core::str::from_utf8(&line[..19]) {
            let year = s[0..4].parse::<u64>().unwrap_or(2026); let month = s[5..7].parse::<u64>().unwrap_or(1);
            let day = s[8..10].parse::<u64>().unwrap_or(1); let hour = s[11..13].parse::<u64>().unwrap_or(0);
            let min = s[14..16].parse::<u64>().unwrap_or(0); let sec = s[17..19].parse::<u64>().unwrap_or(0);
            let ms = ((year-1970)*31536000+(month-1)*2592000+(day-1)*86400+hour*3600+min*60+sec)*1000;
            let end = if line.len()>19 && line[19]==b'.' { 23.min(line.len()) } else { 19 }; return (end, ms); } }
    if line.len() >= 10 && line[..10].iter().all(|&b| b.is_ascii_digit()) {
        if let Ok(s) = core::str::from_utf8(&line[..10]) { if let Ok(epoch) = s.parse::<u64>() {
            let end = line.iter().position(|&b| !b.is_ascii_digit() && b!=b'.').unwrap_or(line.len()); return (end, epoch*1000); } } }
    (0, 0)
}
fn detect_log_level(data: &[u8]) -> (u8, &[u8]) {
    for &(prefix, code) in &[(b"DEBUG" as &[u8], 1u8), (b"INFO",2), (b"WARN",3), (b"WARNING",3), (b"ERROR",4), (b"FATAL",5), (b"CRITICAL",5)] {
        if data.starts_with(prefix) { let after = &data[prefix.len()..];
            if after.is_empty() || after[0]==b' ' || after[0]==b']' || after[0]==b':' { return (code, after); } } }
    if data.starts_with(b"[") { if let Some(end) = data.iter().position(|&b| b==b']') {
        let (code, _) = detect_log_level(&data[1..end]); if code > 0 { return (code, &data[end+1..]); } } }
    (0, data)
}
fn detect_service_name<'a>(data: &'a [u8], dict: &mut Vec<Vec<u8>>) -> (Option<usize>, &'a [u8]) {
    let trimmed = trim_leading_space(data);
    if let Some(cp) = trimmed.iter().position(|&b| b==b':') { if cp > 0 && cp <= 64 {
        let name = &trimmed[..cp];
        if name.iter().all(|&b| b.is_ascii_alphanumeric() || b==b'_' || b==b'-' || b==b'.') {
            let nv = name.to_vec(); let idx = if let Some(i) = dict.iter().position(|s| s==&nv) { i } else { dict.push(nv); dict.len()-1 };
            return (Some(idx), &trimmed[cp+1..]); } } } (None, trimmed)
}
fn trim_leading_space(data: &[u8]) -> &[u8] { &data[data.iter().position(|&b| b!=b' ' && b!=b'\t').unwrap_or(data.len())..] }
fn encode_varint(out: &mut Vec<u8>, mut value: u64) { loop { let mut byte = (value & 0x7F) as u8; value >>= 7;
    if value > 0 { byte |= 0x80; } out.push(byte); if value == 0 { break; } } }
fn decode_varint(data: &[u8]) -> (u64, usize) { let mut r = 0u64; let mut s = 0u32;
    for (i, &byte) in data.iter().enumerate() { r |= ((byte & 0x7F) as u64) << s; s += 7;
        if byte & 0x80 == 0 { return (r, i+1); } if s >= 64 { return (r, i+1); } } (r, data.len()) }
fn write_timestamp(out: &mut Vec<u8>, ms: u64) { let secs = ms/1000; let mut buf = [0u8;20]; let mut n = secs; let mut len = 0;
    if n == 0 { out.push(b'0'); return; } while n > 0 { buf[len] = b'0'+(n%10) as u8; n /= 10; len += 1; }
    for i in (0..len).rev() { out.push(buf[i]); } }

fn structured_encode(data: &[u8]) -> Vec<u8> {
    let trimmed = trim_leading_space(data);
    if trimmed.starts_with(b"{") || trimmed.starts_with(b"[") { structured_encode_json(data) }
    else if looks_like_csv(data) { structured_encode_csv(data) }
    else if trimmed.starts_with(b"<") { structured_encode_xml(data) }
    else { let mut out = Vec::with_capacity(1+data.len()); out.push(0x00); out.extend_from_slice(data); out }
}
fn structured_decode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() { return Vec::new(); }
    match data[0] { 0x00 => data[1..].to_vec(), 0x01 => structured_decode_json(&data[1..]),
        0x02 => structured_decode_csv(&data[1..]), 0x03 => structured_decode_xml(&data[1..]), _ => data.to_vec() }
}
fn structured_encode_json(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()); out.push(0x01);
    let mut keys: Vec<Vec<u8>> = Vec::new(); let mut i = 0;
    while i < data.len() { if data[i]==b'"' { let start=i+1;
        let end = data[start..].iter().position(|&b| b==b'"').map(|p| start+p).unwrap_or(data.len());
        let after = data[end+1..].iter().position(|&b| b!=b' '&&b!=b'\t').map(|p| end+1+p);
        if let Some(cp) = after { if cp < data.len() && data[cp]==b':' { let key = data[start..end].to_vec();
            if !keys.contains(&key) && keys.len() < 65535 { keys.push(key); } } } i = end+1; } else { i+=1; } }
    out.extend_from_slice(&(keys.len() as u16).to_be_bytes());
    for k in &keys { out.extend_from_slice(&(k.len() as u16).to_be_bytes()); out.extend_from_slice(k); }
    i = 0; while i < data.len() { if data[i]==b'"' { let start=i+1;
        let end = data[start..].iter().position(|&b| b==b'"').map(|p| start+p).unwrap_or(data.len());
        let key = &data[start..end]; let aq = end+1;
        let nw = data[aq..].iter().position(|&b| b!=b' '&&b!=b'\t').map(|p| aq+p);
        if let Some(np) = nw { if np < data.len() && data[np]==b':' { if let Some(kid) = keys.iter().position(|k| k==key) {
            out.push(0xFE); out.extend_from_slice(&(kid as u16).to_be_bytes()); i=np+1; continue; } } }
        out.push(b'"'); out.extend_from_slice(&data[start..end]); out.push(b'"'); i=end+1;
    } else { out.push(data[i]); i+=1; } } out
}
fn structured_decode_json(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return Vec::new(); } let mut pos = 0;
    let kc = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(kc);
    for _ in 0..kc { if pos+2>data.len() { break; } let kl = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
        if pos+kl>data.len() { break; } keys.push(data[pos..pos+kl].to_vec()); pos+=kl; }
    let mut out = Vec::with_capacity(data.len()*2);
    while pos < data.len() { if data[pos]==0xFE { pos+=1; if pos+2>data.len() { break; }
        let kid = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
        out.push(b'"'); if kid<keys.len() { out.extend_from_slice(&keys[kid]); } out.push(b'"'); out.push(b':');
    } else { out.push(data[pos]); pos+=1; } } out
}
fn structured_encode_csv(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()); out.push(0x02);
    let fnl = data.iter().position(|&b| b==b'\n').unwrap_or(data.len()); let header = &data[..fnl];
    out.extend_from_slice(&(header.len() as u16).to_be_bytes()); out.extend_from_slice(header);
    let cols: Vec<&[u8]> = header.split(|&b| b==b',').collect(); let cc = cols.len();
    out.extend_from_slice(&(cc as u16).to_be_bytes());
    let body = if fnl < data.len() { &data[fnl+1..] } else { &[] };
    let mut prev = vec![0i64; cc];
    for line in body.split(|&b| b==b'\n') { if line.is_empty() { continue; }
        let fields: Vec<&[u8]> = line.split(|&b| b==b',').collect();
        for (ci, field) in fields.iter().enumerate() { if ci >= cc { break; }
            if let Ok(s) = core::str::from_utf8(field) { if let Ok(n) = s.trim().parse::<i64>() {
                let delta = n - prev[ci]; prev[ci] = n; out.push(0x01); encode_varint_signed(&mut out, delta); continue; } }
            out.push(0x02); out.extend_from_slice(&(*field).len().to_be_bytes()[6..8]); out.extend_from_slice(field); }
        out.push(0x00); } out
}
fn structured_decode_csv(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 { return Vec::new(); } let mut pos = 0;
    let hl = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
    if pos+hl > data.len() { return Vec::new(); } let header = &data[pos..pos+hl]; pos+=hl;
    if pos+2 > data.len() { return Vec::new(); }
    let cc = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
    let mut out = Vec::with_capacity(data.len()*2); out.extend_from_slice(header); out.push(b'\n');
    let mut prev = vec![0i64; cc]; let mut ci = 0;
    while pos < data.len() { let m = data[pos]; pos+=1; match m {
        0x00 => { if out.last()==Some(&b',') { out.pop(); } out.push(b'\n'); ci=0; }
        0x01 => { let (d, br) = decode_varint_signed(&data[pos..]); pos+=br;
            prev[ci.min(cc-1)] += d; write_i64(&mut out, prev[ci.min(cc-1)]); out.push(b','); ci+=1; }
        0x02 => { if pos+2>data.len() { break; } let sl = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
            let end = (pos+sl).min(data.len()); out.extend_from_slice(&data[pos..end]); out.push(b','); pos=end; ci+=1; }
        _ => {} } } out
}
fn structured_encode_xml(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len()); out.push(0x03);
    let mut tags: Vec<Vec<u8>> = Vec::new(); let mut i = 0;
    while i < data.len() { if data[i]==b'<' && i+1<data.len() && data[i+1]!=b'/' && data[i+1]!=b'!' && data[i+1]!=b'?' {
        let start=i+1; let end = data[start..].iter().position(|&b| b==b' '||b==b'>'||b==b'/').map(|p| start+p).unwrap_or(data.len());
        let tag = data[start..end].to_vec(); if !tag.is_empty() && !tags.contains(&tag) && tags.len()<65535 { tags.push(tag); } } i+=1; }
    out.extend_from_slice(&(tags.len() as u16).to_be_bytes());
    for t in &tags { out.extend_from_slice(&(t.len() as u16).to_be_bytes()); out.extend_from_slice(t); }
    i = 0; while i < data.len() { if data[i]==b'<' { let is_closing = i+1<data.len() && data[i+1]==b'/';
        let ts = if is_closing { i+2 } else { i+1 };
        if ts < data.len() && data[ts]!=b'!' && data[ts]!=b'?' {
            let end = data[ts..].iter().position(|&b| b==b' '||b==b'>'||b==b'/').map(|p| ts+p).unwrap_or(data.len());
            let tag = &data[ts..end]; if let Some(tid) = tags.iter().position(|t| t==tag) {
                out.push(if is_closing { 0xFD } else { 0xFE }); out.extend_from_slice(&(tid as u16).to_be_bytes());
                let gt = data[i..].iter().position(|&b| b==b'>').map(|p| i+p+1).unwrap_or(data.len());
                if end < gt.saturating_sub(1) { let attrs = &data[end..gt-1]; out.push(0xFC);
                    out.extend_from_slice(&(attrs.len() as u16).to_be_bytes()); out.extend_from_slice(attrs); }
                i = gt; continue; } } } out.push(data[i]); i+=1; } out
}
fn structured_decode_xml(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return Vec::new(); } let mut pos = 0;
    let tc = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
    let mut tags: Vec<Vec<u8>> = Vec::with_capacity(tc);
    for _ in 0..tc { if pos+2>data.len() { break; } let tl = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
        if pos+tl>data.len() { break; } tags.push(data[pos..pos+tl].to_vec()); pos+=tl; }
    let mut out = Vec::with_capacity(data.len()*2);
    while pos < data.len() { match data[pos] {
        0xFE => { pos+=1; if pos+2>data.len() { break; } let tid = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
            out.push(b'<'); if tid<tags.len() { out.extend_from_slice(&tags[tid]); }
            if pos<data.len() && data[pos]==0xFC { pos+=1; if pos+2<=data.len() {
                let al = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
                let end = (pos+al).min(data.len()); out.extend_from_slice(&data[pos..end]); pos=end; } } out.push(b'>'); }
        0xFD => { pos+=1; if pos+2>data.len() { break; } let tid = u16::from_be_bytes([data[pos],data[pos+1]]) as usize; pos+=2;
            out.extend_from_slice(b"</"); if tid<tags.len() { out.extend_from_slice(&tags[tid]); } out.push(b'>'); }
        _ => { out.push(data[pos]); pos+=1; } } } out
}
fn looks_like_csv(data: &[u8]) -> bool { let fnl = data.iter().position(|&b| b==b'\n').unwrap_or(data.len());
    data[..fnl].iter().filter(|&&b| b==b',').count() >= 2 && fnl < data.len() }
fn encode_varint_signed(out: &mut Vec<u8>, value: i64) { encode_varint(out, ((value<<1)^(value>>63)) as u64); }
fn decode_varint_signed(data: &[u8]) -> (i64, usize) { let (zz, b) = decode_varint(data); (((zz>>1) as i64) ^ -((zz&1) as i64), b) }
fn write_i64(out: &mut Vec<u8>, value: i64) { if value < 0 { out.push(b'-'); write_u64(out, (-value) as u64); } else { write_u64(out, value as u64); } }
fn write_u64(out: &mut Vec<u8>, value: u64) { if value == 0 { out.push(b'0'); return; }
    let mut buf = [0u8;20]; let mut n = value; let mut len = 0;
    while n > 0 { buf[len] = b'0'+(n%10) as u8; n /= 10; len += 1; } for i in (0..len).rev() { out.push(buf[i]); } }

fn apply_domain_preprocess(data: &[u8], mode: CompressionMode, image_width: Option<usize>) -> (Vec<u8>, DomainTransform, Option<[i16; 4]>, Option<u64>) {
    match mode {
        CompressionMode::Audio => { let (r,c) = audio_lp_encode(data); (r, DomainTransform::AUDIO_LP, Some(c), None) }
        CompressionMode::Image => { if let Some(w) = image_width { (image_med_encode(data,w), DomainTransform::IMAGE_MED, None, Some(w as u64)) }
            else { (data.to_vec(), DomainTransform::NONE, None, None) } }
        CompressionMode::Genomic => (genomic_encode(data), DomainTransform::GENOMIC, None, None),
        CompressionMode::Source => (source_encode(data), DomainTransform::SOURCE, None, None),
        CompressionMode::Log => (log_encode(data), DomainTransform::LOG, None, None),
        CompressionMode::Structured => (structured_encode(data), DomainTransform::STRUCTURED, None, None),
        _ => (data.to_vec(), DomainTransform::NONE, None, None),
    }
}
fn reverse_domain_preprocess(data: &[u8], xform: DomainTransform, coeffs: Option<&[i16; 4]>, iw: Option<u64>) -> TtcResult<Vec<u8>> {
    match xform.0 { 0 => Ok(data.to_vec()),
        1 => Ok(audio_lp_decode(data, coeffs.ok_or_else(|| TtcError::DecompressionError("Missing LP coefficients".into()))?)),
        2 => Ok(image_med_decode(data, iw.ok_or(TtcError::ImageWidthRequired)? as usize)),
        3 => Ok(genomic_decode(data)), 4 => Ok(source_decode(data)), 5 => Ok(log_decode(data)), 6 => Ok(structured_decode(data)),
        7 => Err(TtcError::InvalidDomainTransform(7)), _ => Err(TtcError::InvalidDomainTransform(xform.0)) }
}

// ─── LZ77 (§4) ─────────────────────────────────────────────────────────────

const INVALID_POS: u32 = u32::MAX;
struct Lz77Engine { window_size: usize, min_match: usize, #[allow(dead_code)] min_run: usize, chain_depth: usize, head: Vec<u32>, chain: Vec<u32> }
impl Lz77Engine {
    fn new(cfg: &LevelConfig) -> Self { Self { window_size: cfg.window_size, min_match: cfg.min_match, min_run: cfg.min_run,
        chain_depth: cfg.chain_depth, head: vec![INVALID_POS; cfg.window_size], chain: vec![INVALID_POS; cfg.window_size] } }
    #[inline] fn hash(&self, data: &[u8], i: usize) -> usize { if i+2 >= data.len() { return 0; }
        ((data[i] as usize).wrapping_mul(65521) ^ (data[i+1] as usize).wrapping_mul(257) ^ data[i+2] as usize) % self.window_size }
    fn find_best_match(&self, data: &[u8], pos: usize) -> Option<(usize, usize)> {
        if pos+2 >= data.len() { return None; } let h = self.hash(data, pos);
        let mut j = self.head[h]; let mut bl = 0usize; let mut bd = 0usize; let mut steps = 0;
        let min_pos = pos.saturating_sub(self.window_size);
        while j != INVALID_POS && steps < self.chain_depth { let jj = j as usize;
            if jj < min_pos || jj >= pos { j = self.chain[jj % self.window_size]; steps += 1; continue; }
            if data[jj]==data[pos] && data[jj+1]==data[pos+1] && data[jj+2]==data[pos+2] {
                let ml = 255.min(data.len()-pos); let mut len = 3;
                while len < ml && jj+len < data.len() && data[jj+len]==data[pos+len] { len += 1; }
                if len > bl { bl = len; bd = pos - jj; } }
            j = self.chain[jj % self.window_size]; steps += 1; }
        if bl >= self.min_match { Some((bd, bl)) } else { None }
    }
    #[inline] fn update(&mut self, data: &[u8], pos: usize) { if pos+2 >= data.len() { return; }
        let h = self.hash(data, pos); let old = self.head[h]; self.head[h] = pos as u32; self.chain[pos % self.window_size] = old; }
    #[inline] fn count_run(&self, data: &[u8], pos: usize) -> usize { if pos >= data.len() { return 0; }
        let byte = data[pos]; let mut len = 1; while pos+len < data.len() && data[pos+len]==byte && len < 255 { len += 1; } len }
}

fn tokenize_greedy_lazy(data: &[u8], hist_off: usize, cfg: &LevelConfig) -> Vec<Token> {
    let mut eng = Lz77Engine::new(cfg);
    // Pre-allocate: typical compression produces ~1 token per 4 bytes
    let mut tokens = Vec::with_capacity((data.len().saturating_sub(hist_off)) / 4 + 16);
    for j in 0..hist_off.min(data.len()) { eng.update(data, j); } let mut i = hist_off;
    while i < data.len() {
        let run = eng.count_run(data, i);
        if run >= cfg.min_run { tokens.push(Token::Run { byte: data[i], length: run });
            for k in 0..run { eng.update(data, i+k); } i += run; continue; }
        if let Some((dist, len)) = eng.find_best_match(data, i) {
            if cfg.parsing == Parsing::Lazy && len < 255 { eng.update(data, i);
                if let Some((dist1, len1)) = eng.find_best_match(data, i+1) { if len1 > len+1 {
                    tokens.push(Token::Literal(data[i])); i+=1;
                    tokens.push(Token::Match { dist: dist1, length: len1 });
                    for k in 0..len1 { eng.update(data, i+k); } i += len1; continue; } }
            } else { eng.update(data, i); }
            tokens.push(Token::Match { dist, length: len });
            for k in 1..len { eng.update(data, i+k); } i += len; continue; }
        tokens.push(Token::Literal(data[i])); eng.update(data, i); i += 1; } tokens
}

fn tokenize_beam(data: &[u8], hist_off: usize, cfg: &LevelConfig, cost_mode: ChunkMode) -> Vec<Token> {
    let chunk_len = data.len() - hist_off; if chunk_len == 0 { return Vec::new(); }
    let mut eng = Lz77Engine::new(cfg);
    for j in 0..data.len().min(hist_off+chunk_len) { eng.update(data, j); }
    let lit_bits: u64 = match cost_mode { ChunkMode::Stored=>8, ChunkMode::Compressed=>10, ChunkMode::TernaryEnhanced=>7, ChunkMode::TernaryAns=>8 };
    let match_overhead: u64 = match cost_mode { ChunkMode::Stored=>64, ChunkMode::Compressed=>6, ChunkMode::TernaryEnhanced=>6, ChunkMode::TernaryAns=>4 };
    #[derive(Clone)] struct Node { cost: u64, token: Option<Token>, prev: u32 }
    let estimated_nodes = chunk_len * 3 + 1;
    let mut nodes: Vec<Node> = Vec::with_capacity(estimated_nodes.min(1 << 20));
    nodes.push(Node { cost: 0, token: None, prev: u32::MAX });
    let mut beam: Vec<Vec<u32>> = (0..=chunk_len).map(|_| Vec::with_capacity(BEAM_WIDTH)).collect();
    beam[0].push(0);
    for pos in 0..chunk_len { if beam[pos].is_empty() { continue; }
        let mut current: Vec<u32> = beam[pos].clone(); current.sort_by_key(|&idx| nodes[idx as usize].cost);
        current.truncate(BEAM_WIDTH); let abs_pos = hist_off + pos;
        for &ni in &current { let base = nodes[ni as usize].cost;
            let lc = base + lit_bits; let li = nodes.len() as u32;
            nodes.push(Node { cost: lc, token: Some(Token::Literal(data[abs_pos])), prev: ni });
            if pos+1 <= chunk_len { beam[pos+1].push(li); }
            if let Some((dist, len)) = eng.find_best_match(data, abs_pos) {
                let db = if dist==0{1} else {(64-(dist as u64).leading_zeros()) as u64};
                let lb = if len==0{1} else {(64-(len as u64).leading_zeros()) as u64};
                let mc = base + match_overhead + db + lb; let mi = nodes.len() as u32;
                nodes.push(Node { cost: mc, token: Some(Token::Match { dist, length: len }), prev: ni });
                beam[(pos+len).min(chunk_len)].push(mi); }
            let run = eng.count_run(data, abs_pos);
            if run >= cfg.min_run { let rc = base + 12; let ri = nodes.len() as u32;
                nodes.push(Node { cost: rc, token: Some(Token::Run { byte: data[abs_pos], length: run }), prev: ni });
                beam[(pos+run).min(chunk_len)].push(ri); } } }
    if beam[chunk_len].is_empty() { return tokenize_greedy_lazy(data, hist_off, cfg); }
    let best_end = *beam[chunk_len].iter().min_by_key(|&&i| nodes[i as usize].cost).unwrap();
    let mut tokens = Vec::new(); let mut idx = best_end;
    while idx != 0 && idx != u32::MAX { if let Some(ref tok) = nodes[idx as usize].token { tokens.push(tok.clone()); } idx = nodes[idx as usize].prev; }
    tokens.reverse(); tokens
}

fn tokenize_chunk(data: &[u8], hist_len: usize, cfg: &LevelConfig, cost_mode: ChunkMode) -> Vec<Token> {
    match cfg.parsing { Parsing::BeamOptimal => tokenize_beam(data, hist_len, cfg, cost_mode), _ => tokenize_greedy_lazy(data, hist_len, cfg) }
}
fn decompress_tokens(tokens: &[Token], history: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = history.to_vec();
    for tok in tokens { match tok { Token::Literal(b) => out.push(*b),
        Token::Run { byte, length } => { for _ in 0..*length { out.push(*byte); } }
        Token::Match { dist, length } => { for _ in 0..*length { let b = out[out.len()-dist]; out.push(b); } } } } out
}

// ─── Token Serialization (§5) ───────────────────────────────────────────────

fn serialize_compressed(tokens: &[Token], initial_m: u8) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(tokens.len()*2); encode_elias_gamma(&mut w, tokens.len() as u64);
    w.write(initial_m as u32, 8); let mut m = initial_m; let mut mc = 0u32; let mut ms = 0u64; let mut i = 0;
    while i < tokens.len() { match &tokens[i] {
        Token::Literal(_) => { let start = i; while i < tokens.len() && matches!(tokens[i], Token::Literal(_)) { i+=1; }
            w.write(0b00, 2); encode_elias_gamma(&mut w, (i-start) as u64);
            for j in start..i { if let Token::Literal(b) = tokens[j] { w.write(b as u32, 8); } } }
        Token::Run { byte, length } => { w.write(0b01, 2); w.write(*byte as u32, 8); encode_elias_gamma(&mut w, *length as u64); i+=1; }
        Token::Match { dist, length } => { ms += *dist as u64; mc += 1;
            if mc % 128 == 0 && mc > 0 { let mean = ms/mc as u64;
                let nm = if mean==0{1} else {((64-mean.leading_zeros()).saturating_sub(1) as u8).clamp(1,8)};
                if nm != m { let delta = (nm as i8 - m as i8).clamp(-4,3); w.write(0b11,2); w.write((delta as u8 & 0x07) as u32,3); m = (m as i8+delta) as u8; } }
            w.write(0b10, 2); encode_rice(&mut w, *dist as u64, m); encode_elias_gamma(&mut w, *length as u64); i+=1; } } }
    w.finish_with_header()
}
fn deserialize_compressed(payload: &[u8]) -> TtcResult<Vec<Token>> {
    if payload.len() < 4 { return Err(TtcError::DecompressionError("Payload too short".into())); }
    let mut r = BitReader::new(&payload[4..]); let tc = decode_elias_gamma(&mut r) as usize;
    let mut m = r.read(8) as u8; let mut tokens = Vec::with_capacity(tc); let mut decoded = 0;
    while decoded < tc && !r.is_exhausted() { match r.read(2) {
        0b00 => { let cnt = decode_elias_gamma(&mut r) as usize;
            for _ in 0..cnt { tokens.push(Token::Literal(r.read(8) as u8)); decoded+=1; } }
        0b01 => { let byte = r.read(8) as u8; let len = decode_elias_gamma(&mut r) as usize;
            tokens.push(Token::Run { byte, length: len }); decoded+=1; }
        0b10 => { let dist = decode_rice(&mut r, m) as usize; let len = decode_elias_gamma(&mut r) as usize;
            tokens.push(Token::Match { dist, length: len }); decoded+=1; }
        0b11 => { let dr = r.read(3) as i8; let d = if dr>3{dr-8}else{dr}; m = ((m as i8)+d).clamp(1,8) as u8; }
        _ => {} } } Ok(tokens)
}

fn serialize_ternary_enhanced(tokens: &[Token], initial_m: u8, tc: &TritCostTables) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(tokens.len()*3); encode_hybrid_prefix(&mut w, tokens.len() as u64);
    let m = initial_m; let mut i = 0;
    while i < tokens.len() { match &tokens[i] {
        Token::Literal(_) => { let start = i; while i < tokens.len() && matches!(tokens[i], Token::Literal(_)) { i+=1; }
            let group: Vec<u8> = (start..i).filter_map(|j| if let Token::Literal(b) = tokens[j] { Some(b) } else { None }).collect();
            let best = tc.best_rep(&group); w.write(0b00, 2);
            w.write(match best { GfRep::C=>0b00, GfRep::B=>0b01, GfRep::A=>0b10 }, 2);
            encode_hybrid_prefix(&mut w, group.len() as u64); for &b in &group { write_trit_encoded(&mut w, b, best); } }
        Token::Run { byte, length } => { let best = tc.best_rep(&[*byte]); w.write(0b01, 2);
            w.write(match best { GfRep::C=>0b00, GfRep::B=>0b01, GfRep::A=>0b10 }, 2);
            write_trit_encoded(&mut w, *byte, best); encode_hybrid_prefix(&mut w, *length as u64); i+=1; }
        Token::Match { dist, length } => { w.write(0b10, 2); encode_rice(&mut w, *dist as u64, m);
            encode_hybrid_prefix(&mut w, *length as u64); i+=1; } } } w.finish_with_header()
}
#[inline(always)]
fn write_trit_encoded(w: &mut BitWriter, byte: u8, rep: GfRep) { match rep {
    GfRep::C => { let td = byte_to_bijective(byte); w.write(td.len as u32, 3);
        for k in 0..(td.len as usize) { w.write(match td.digits[k] { 1=>0b00, 2=>0b01, 3=>0b10, _=>0b11 }, 2); } }
    GfRep::B => { let td = byte_to_standard(byte); w.write(td.len as u32, 3);
        for k in 0..(td.len as usize) { w.write(match td.digits[k] { 0=>0b00, 1=>0b01, 2=>0b10, _=>0b11 }, 2); } }
    GfRep::A => { let td = byte_to_balanced(byte); w.write(td.len as u32, 3);
        for k in 0..(td.len as usize) { w.write(match td.digits[k] { -1=>0b10, 0=>0b00, 1=>0b01, _=>0b11 } as u32, 2); } } } }

/// Deserialize ternary enhanced — with fast-path Rep C optimization.
/// When adaptive_rep == false, skip the 2-bit rep selector and assume Rep C.
fn deserialize_ternary_enhanced(payload: &[u8], adaptive_rep: bool) -> TtcResult<Vec<Token>> {
    if payload.len() < 4 { return Err(TtcError::DecompressionError("Payload too short".into())); }
    let mut r = BitReader::new(&payload[4..]); let tc = decode_hybrid_prefix(&mut r) as usize;
    let mut tokens = Vec::with_capacity(tc); let mut decoded = 0; let m: u8 = 4;
    while decoded < tc && !r.is_exhausted() { match r.read(2) {
        0b00 => {
            // OPTIMIZATION: fast-path when adaptive_rep is false → always Rep C
            if !adaptive_rep {
                let _ = r.read(2); // consume rep selector (ignored, force Rep C)
                let cnt = decode_hybrid_prefix(&mut r) as usize;
                for _ in 0..cnt {
                    tokens.push(Token::Literal(read_trit_decoded(&mut r, GfRep::C)));
                    decoded += 1;
                }
            } else {
                let rep = match r.read(2) { 0b00=>GfRep::C, 0b01=>GfRep::B, _=>GfRep::A };
                let cnt = decode_hybrid_prefix(&mut r) as usize;
                for _ in 0..cnt { tokens.push(Token::Literal(read_trit_decoded(&mut r, rep))); decoded+=1; }
            }
        }
        0b01 => {
            let rep = if !adaptive_rep { let _ = r.read(2); GfRep::C }
                      else { match r.read(2) { 0b00=>GfRep::C, 0b01=>GfRep::B, _=>GfRep::A } };
            let byte = read_trit_decoded(&mut r, rep); let len = decode_hybrid_prefix(&mut r) as usize;
            tokens.push(Token::Run { byte, length: len }); decoded+=1;
        }
        0b10 => { let dist = decode_rice(&mut r, m) as usize; let len = decode_hybrid_prefix(&mut r) as usize;
            tokens.push(Token::Match { dist, length: len }); decoded+=1; }
        0b11 => { let _ = r.read(3); } _ => {} } } Ok(tokens)
}
#[inline(always)]
fn read_trit_decoded(r: &mut BitReader, rep: GfRep) -> u8 { let dc = r.read(3) as usize; match rep {
    GfRep::C => { let mut td = TritDigits { digits: [0;6], len: dc as u8 };
        for k in 0..dc { td.digits[k] = match r.read(2) { 0b00=>1, 0b01=>2, 0b10=>3, _=>1 }; } bijective_to_byte(&td) }
    GfRep::B => { let mut td = TritDigits { digits: [0;6], len: dc as u8 };
        for k in 0..dc { td.digits[k] = match r.read(2) { 0b00=>0, 0b01=>1, 0b10=>2, _=>0 }; } standard_to_byte(&td) }
    GfRep::A => { let mut td = BalancedTritDigits { digits: [0;6], len: dc as u8 };
        for k in 0..dc { td.digits[k] = match r.read(2) { 0b10=> -1i8, 0b00=>0, 0b01=>1, _=>0 }; } balanced_to_byte(&td) } } }

// ─── Phase 1 + Phase 2 (deduplicated compress path) ────────────────────────

#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub index: usize, pub original_size: u32, pub compressed_size: u32,
    pub base: u16, pub tau: f64, pub delta: f64, pub mode: ChunkMode,
    pub rice_m: u8, pub delta_flag: DeltaFlag, pub domain_transform: DomainTransform, pub payload: Vec<u8>,
}

fn make_stored_payload(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5+data.len()); out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.push(0x00); out.extend_from_slice(data); out
}
fn make_mode_payload(data: &[u8], mode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5+payload.len()); out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.push(mode); out.extend_from_slice(payload); out
}

/// Phase 1: read-only analysis. No history dependency. Fully parallelizable.
#[derive(Debug, Clone)]
struct Phase1Result {
    chunk_index: usize, pre_compressed: bool, gurft: GurftResult,
    base: u16, delta_flag: DeltaFlag, delta_data: Vec<u8>, h_chunk: f64,
}

fn phase1_analyze(chunk: &[u8], chunk_index: usize, cfg: &LevelConfig, mode: CompressionMode) -> Phase1Result {
    if is_pre_compressed(chunk) {
        return Phase1Result { chunk_index, pre_compressed: true, gurft: GurftResult::default(),
            base: 3, delta_flag: DeltaFlag::NONE, delta_data: Vec::new(), h_chunk: 8.0 };
    }
    // Small-chunk fast-path — skip GURFT, delta, base selection for tiny chunks.
    if chunk.len() < 256 {
        return Phase1Result { chunk_index, pre_compressed: false, gurft: GurftResult::default(),
            base: 3, delta_flag: DeltaFlag::NONE, delta_data: chunk.to_vec(),
            h_chunk: compute_entropy(chunk) };
    }
    let gurft = if cfg.skip_gurft { GurftResult::default() } else { gurft_analyze(chunk) };
    let base = try_and_compare_base(chunk, &gurft, mode);
    let delta_flag = select_delta(chunk, mode);
    let mut delta_data = Vec::with_capacity(chunk.len());
    let mut scratch = Vec::with_capacity(chunk.len());
    apply_delta_encode(chunk, delta_flag, &mut delta_data, &mut scratch);
    let h_chunk = compute_entropy(chunk);
    Phase1Result { chunk_index, pre_compressed: false, gurft, base, delta_flag, delta_data, h_chunk }
}

/// Phase 2: tokenization + serialization. Needs history for dependent mode.
/// Takes Phase1Result by VALUE — avoids cloning delta_data.
fn phase2_compress(
    chunk: &[u8], p1: Phase1Result, history: &[u8],
    cfg: &LevelConfig, independent: bool, tc: &TritCostTables, dom_xform: DomainTransform,
) -> ChunkResult {
    let orig_size = chunk.len() as u32;
    if p1.pre_compressed {
        return ChunkResult { index: p1.chunk_index, original_size: orig_size,
            compressed_size: (chunk.len()+5) as u32, base: 3, tau: 0.0, delta: 0.0,
            mode: ChunkMode::Stored, rice_m: 0, delta_flag: DeltaFlag::NONE,
            domain_transform: dom_xform, payload: make_stored_payload(chunk) };
    }

    // Use owned delta_data directly — no clone needed (p1 consumed by value)
    // Move delta_data directly in independent case (p1 is consumed by value).
    // For dependent mode, prepend history window and extend with delta_data.
    let (vdata, hlen) = if independent || history.is_empty() {
        (p1.delta_data, 0) // zero-copy move — no clone needed
    } else {
        let h = &history[history.len().saturating_sub(cfg.window_size)..];
        let mut vd = Vec::with_capacity(h.len() + p1.delta_data.len());
        vd.extend_from_slice(h);
        vd.extend_from_slice(&p1.delta_data);
        let hl = h.len();
        (vd, hl)
    };

    let mut candidates: Vec<(ChunkMode, Vec<u8>)> = vec![(ChunkMode::Stored, make_stored_payload(chunk))];

    if p1.h_chunk <= MODE_PRUNE_ENTROPY {
        let best_cost_mode = if p1.h_chunk < 4.0 { ChunkMode::TernaryAns }
            else if chunk.len() <= 16384 { ChunkMode::TernaryEnhanced }
            else { ChunkMode::Compressed };

        let tokens = tokenize_chunk(&vdata, hlen, cfg, best_cost_mode);
        let rice_m = compute_initial_rice_m(&tokens);

        // Mode 1: Compressed
        let comp = serialize_compressed(&tokens, rice_m);
        candidates.push((ChunkMode::Compressed, make_mode_payload(chunk, 1, &comp)));

        // Mode 3: Ternary ANS (now spec-compliant base-3 state machine)
        let tans = serialize_tans(&tokens, cfg.window_size);
        candidates.push((ChunkMode::TernaryAns, make_mode_payload(chunk, 3, &tans)));

        // Mode 2: TernaryEnhanced (≤16 KB chunks only)
        if chunk.len() <= 16384 {
            let enh = serialize_ternary_enhanced(&tokens, rice_m, tc);
            candidates.push((ChunkMode::TernaryEnhanced, make_mode_payload(chunk, 2, &enh)));
        }

        // Early exit: compressed >= 98% of raw → STORED
        if let Some((_, ref c)) = candidates.iter().find(|(m, _)| *m == ChunkMode::Compressed) {
            if c.len() >= chunk.len() * 98 / 100 {
                return ChunkResult { index: p1.chunk_index, original_size: orig_size,
                    compressed_size: (chunk.len()+5) as u32, base: p1.base, tau: p1.gurft.tau,
                    delta: p1.gurft.delta, mode: ChunkMode::Stored, rice_m: 0,
                    delta_flag: DeltaFlag::NONE, domain_transform: dom_xform,
                    payload: make_stored_payload(chunk) };
            }
        }
    }

    let (best_mode, best_payload) = candidates.into_iter().min_by_key(|(_, p)| p.len()).unwrap();
    ChunkResult { index: p1.chunk_index, original_size: orig_size, compressed_size: best_payload.len() as u32,
        base: p1.base, tau: p1.gurft.tau, delta: p1.gurft.delta, mode: best_mode,
        rice_m: compute_initial_rice_m(&[]), delta_flag: p1.delta_flag,
        domain_transform: dom_xform, payload: best_payload }
}

/// Thin wrapper — delegates to phase1 + phase2.
#[allow(dead_code)]
fn compress_chunk(
    chunk: &[u8], history: &[u8], idx: usize, cfg: &LevelConfig,
    mode: CompressionMode, independent: bool, tc: &TritCostTables,
) -> ChunkResult {
    let p1 = phase1_analyze(chunk, idx, cfg, mode);
    phase2_compress(chunk, p1, history, cfg, independent, tc, DomainTransform::NONE)
}

// ─── Inter-Cube Parallel Dispatch (§4.8) ────────────────────────────────────

const PARALLEL_CHUNK_THRESHOLD_MIN: usize = 2;

fn dispatch_independent_parallel(cs: &[&[u8]], cfg: &LevelConfig, mode: CompressionMode, tc: &TritCostTables, dx: DomainTransform) -> Vec<ChunkResult> {
    let cc = cs.len(); let rounds = (cc + TUNNEL_COUNT - 1) / TUNNEL_COUNT;
    let mut results: Vec<Option<ChunkResult>> = vec![None; cc];
    for round in 0..rounds { let start = round*TUNNEL_COUNT; let end = (start+TUNNEL_COUNT).min(cc);
        let batch: Vec<ChunkResult> = (start..end).into_par_iter().map(|idx| {
            let p1 = phase1_analyze(cs[idx], idx, cfg, mode); phase2_compress(cs[idx], p1, &[], cfg, true, tc, dx)
        }).collect();
        for cr in batch { let idx = cr.index; results[idx] = Some(cr); } }
    results.into_iter().map(|o| o.expect("All chunks filled")).collect()
}

fn dispatch_dependent_pipelined(cs: &[&[u8]], cfg: &LevelConfig, mode: CompressionMode, tc: &TritCostTables, dx: DomainTransform) -> Vec<ChunkResult> {
    let cc = cs.len(); let bs = 13; let tb = (cc+bs-1)/bs;
    let mut results: Vec<ChunkResult> = Vec::with_capacity(cc); let mut history: Vec<u8> = Vec::new();
    let fbe = bs.min(cc);
    let mut acache: Vec<Phase1Result> = (0..fbe).into_par_iter().map(|idx| phase1_analyze(cs[idx], idx, cfg, mode)).collect();
    for batch in 0..tb { let bstart = batch*bs; let bend = (bstart+bs).min(cc);
        let nstart = bend; let nend = (nstart+bs).min(cc);
        let current_cache = core::mem::take(&mut acache);
        if nstart < cc { let next_slices: Vec<(usize, &[u8])> = (nstart..nend).map(|i| (i, cs[i])).collect();
            let (p2r, p1r) = rayon::join(|| {
                let mut br = Vec::with_capacity(bend-bstart);
                for (li, p1) in current_cache.into_iter().enumerate() { let gi = bstart+li; if gi >= cc { break; }
                    let cr = phase2_compress(cs[gi], p1, &history, cfg, false, tc, dx);
                    history.extend_from_slice(cs[gi]);
                    if history.len() > cfg.window_size { let t = history.len()-cfg.window_size; history.drain(..t); }
                    br.push(cr); } br },
                || next_slices.par_iter().map(|&(idx, chunk)| phase1_analyze(chunk, idx, cfg, mode)).collect::<Vec<_>>());
            results.extend(p2r); acache = p1r;
        } else { for (li, p1) in current_cache.into_iter().enumerate() { let gi = bstart+li; if gi >= cc { break; }
            let cr = phase2_compress(cs[gi], p1, &history, cfg, false, tc, dx);
            history.extend_from_slice(cs[gi]);
            if history.len() > cfg.window_size { let t = history.len()-cfg.window_size; history.drain(..t); }
            results.push(cr); } } } results
}

fn dispatch_sequential(cs: &[&[u8]], cfg: &LevelConfig, mode: CompressionMode, independent: bool, tc: &TritCostTables, dx: DomainTransform) -> Vec<ChunkResult> {
    let mut results = Vec::with_capacity(cs.len()); let mut history: Vec<u8> = Vec::new();
    for (i, chunk) in cs.iter().enumerate() {
        let p1 = phase1_analyze(chunk, i, cfg, mode);
        let hr = if independent { &[] as &[u8] } else { &history };
        let mut cr = phase2_compress(chunk, p1, hr, cfg, independent, tc, dx);
        cr.domain_transform = dx;
        if !independent { history.extend_from_slice(chunk);
            if history.len() > cfg.window_size { let t = history.len()-cfg.window_size; history.drain(..t); } }
        results.push(cr); } results
}

fn dispatch_chunks(cs: &[&[u8]], cfg: &LevelConfig, mode: CompressionMode, independent: bool, tc: &TritCostTables, dx: DomainTransform) -> Vec<ChunkResult> {
    let cc = cs.len();
    let thread_count = rayon::current_num_threads();
    let threshold = PARALLEL_CHUNK_THRESHOLD_MIN.max(thread_count);
    if thread_count > 1 && cc >= threshold {
        if independent { return dispatch_independent_parallel(cs, cfg, mode, tc, dx); }
        else { return dispatch_dependent_pipelined(cs, cfg, mode, tc, dx); }
    }
    dispatch_sequential(cs, cfg, mode, independent, tc, dx)
}

// ─── Container (§9), Filename (§9.3), Fibonacci (§10) ──────────────────────

fn build_container(chunks: &[ChunkResult], orig_size: u64, crc: u32, mode: CompressionMode,
    level: u8, independent: bool, adaptive_rep: bool, avg_tau: f64, avg_delta: f64,
    predominant_base: u16, lp_coeffs: Option<&[i16; 4]>, image_width: Option<u64>,
    fib_computed: bool, has_filename: bool) -> Vec<u8> {
    let chunk_count = chunks.len() as u16; let cm_off = HEADER_SIZE as u64;
    let cm_size = chunk_count as usize * CHUNK_MAP_ENTRY_SIZE;
    let total_payload: usize = chunks.iter().map(|c| c.payload.len()).sum();
    let comp_size = (HEADER_SIZE + cm_size + total_payload) as u64;
    let mut out = Vec::with_capacity(comp_size as usize);
    out.extend_from_slice(&MAGIC_TTC1); out.push(VERSION_V2); out.push(mode as u8);
    out.extend_from_slice(&orig_size.to_be_bytes()); out.extend_from_slice(&comp_size.to_be_bytes());
    out.extend_from_slice(&crc.to_be_bytes()); out.extend_from_slice(&predominant_base.to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes());
    match mode { CompressionMode::Audio => { if let Some(c) = lp_coeffs { for &v in c { out.extend_from_slice(&v.to_be_bytes()); } }
        else { out.extend_from_slice(&[0u8;8]); } }
        CompressionMode::Image => { out.extend_from_slice(&image_width.unwrap_or(0).to_be_bytes()); }
        _ => out.extend_from_slice(&[0u8;8]) }
    let mut flags = 0u8;
    if chunks.iter().any(|c| c.base != 3) { flags |= 0x01; }
    if chunks.len() > 1 && chunks.iter().any(|c| c.base != chunks[0].base) { flags |= 0x02; }
    if independent { flags |= 0x04; } if adaptive_rep { flags |= 0x08; }
    if fib_computed { flags |= 0x10; } if has_filename { flags |= 0x20; }
    out.push(flags); out.push(level); out.extend_from_slice(&chunk_count.to_be_bytes());
    out.extend_from_slice(&((avg_tau*1_000_000.0) as u32).to_be_bytes());
    out.extend_from_slice(&((avg_delta*1_000_000.0) as u32).to_be_bytes());
    out.extend_from_slice(&[0u8;4]); out.extend_from_slice(&cm_off.to_be_bytes());
    out.extend_from_slice(&[0u8;16]); out.extend_from_slice(&[0u8;16]);
    debug_assert_eq!(out.len(), HEADER_SIZE);
    for c in chunks { out.extend_from_slice(&c.original_size.to_be_bytes()); out.extend_from_slice(&c.compressed_size.to_be_bytes());
        out.extend_from_slice(&c.base.to_be_bytes()); out.extend_from_slice(&((c.tau*1000.0) as u16).to_be_bytes());
        out.extend_from_slice(&((c.delta*1000.0) as u16).to_be_bytes()); out.push(c.rice_m);
        out.push((c.delta_flag.0 << 5) | (c.domain_transform.0 & 0x07)); }
    for c in chunks { out.extend_from_slice(&c.payload); } out
}

fn embed_filename(content: &[u8], filename: &str) -> Vec<u8> {
    let nb = filename.as_bytes(); let mut out = Vec::with_capacity(2+nb.len()+content.len());
    out.extend_from_slice(&(nb.len() as u16).to_be_bytes()); out.extend_from_slice(nb); out.extend_from_slice(content); out
}
fn extract_filename(data: &[u8]) -> (String, &[u8]) {
    if data.len() < 2 { return (String::new(), data); }
    let nl = u16::from_be_bytes([data[0],data[1]]) as usize;
    if data.len() < 2+nl { return (String::new(), data); }
    (sanitize_filename(&String::from_utf8_lossy(&data[2..2+nl])), &data[2+nl..])
}
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name.chars().filter(|c| *c != '\0').collect();
    let safe: String = cleaned
        .split(|c: char| c == '/' || c == '\\')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("_");
    let trimmed = safe.trim_start_matches('.');
    if trimmed.is_empty() { "unnamed".to_string() } else { trimmed.to_string() }
}

#[derive(Debug, Clone)]
pub struct FibonacciAnalysis { pub arb_weight: f64, pub aligned_terms: Vec<u64>, pub optimal_ratio: f64, pub phase_delta: f64, pub resonance_band: String }
pub fn fibonacci_analysis(data_len: usize) -> FibonacciAnalysis {
    let fibs: [u64;25] = [1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765,10946,17711,28657,46368,75025];
    let dl = data_len as u64; let mut aligned = Vec::new(); let mut arb_sum = 0.0f64;
    for &hz in &fibs { if hz==0{continue;} if dl%hz==0||hz%8==0 { aligned.push(hz);
        let rd = libm::fabs((dl as f64*GOLDEN_ANGLE)%364.0 - (dl as f64*GOLDEN_ANGLE)%360.0);
        let gain = if rd<1.875{0.0035} else if rd<3.125{0.0069} else if rd<4.375{0.0104} else if rd<5.625{0.0139} else {0.0174};
        arb_sum += 1.0+gain; } }
    let arb = if aligned.is_empty(){1.0} else {arb_sum/aligned.len() as f64};
    let pd = libm::fabs((dl as f64*GOLDEN_ANGLE)%364.0 - (dl as f64*GOLDEN_ANGLE)%360.0);
    let band = if arb>1.015{"HIGH"} else if arb>1.005{"MEDIUM"} else {"LOW"};
    FibonacciAnalysis { arb_weight: arb, aligned_terms: aligned, optimal_ratio: arb, phase_delta: pd, resonance_band: band.into() }
}

// ─── Main API ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CompressOptions { pub mode: CompressionMode, pub level: u8, pub independent_chunks: bool,
    pub compute_fibonacci: bool, pub image_width: Option<usize>, pub filename: Option<String> }
impl Default for CompressOptions { fn default() -> Self {
    Self { mode: CompressionMode::Temporal, level: 5, independent_chunks: false,
        compute_fibonacci: false, image_width: None, filename: None } } }

#[derive(Debug, Clone)]
pub struct CompressionResult { pub compressed: Vec<u8>, pub original_size: u64, pub compressed_size: u64,
    pub compression_ratio: f64, pub crc32: u32, pub mode: u8, pub mode_name: String, pub version: String,
    pub level: u8, pub level_name: String, pub chunks: Vec<ChunkDescriptor>,
    pub avg_tau: f64, pub avg_delta: f64, pub base_distribution: BaseDistribution,
    pub predominant_base: u16, pub independent_chunks: bool, pub adaptive_rep_used: bool,
    pub fibonacci_analysis: Option<FibonacciAnalysis> }
#[derive(Debug, Clone)]
pub struct ChunkDescriptor { pub index: usize, pub original_size: u32, pub compressed_size: u32,
    pub base: u16, pub tau: f64, pub delta: f64, pub mode: u8, pub rice_m: u8,
    pub delta_flag: u8, pub delta_order: u8, pub delta_rep: String, pub domain_transform: u8 }
#[derive(Debug, Clone, Default)]
pub struct BaseDistribution { pub base_3: u32, pub base_13: u32, pub base_28: u32, pub base_70: u32, pub base_364: u32 }

pub fn ttc_compress(data: &[u8], opts: &CompressOptions) -> TtcResult<CompressionResult> {
    let cfg = level_config(opts.level)?;
    let tc = trit_cost_tables();
    let crc = crc32(data);
    let input = if let Some(ref name) = opts.filename { embed_filename(data, name) } else { data.to_vec() };
    let (preprocessed, dom_xform, lp_coeffs, iw) = apply_domain_preprocess(&input, opts.mode, opts.image_width);
    let cs = cfg.chunk_size; let cc = (preprocessed.len()+cs-1)/cs;
    let chunk_slices: Vec<&[u8]> = (0..cc).map(|i| { let start=i*cs; let end=(start+cs).min(preprocessed.len()); &preprocessed[start..end] }).collect();
    let mut chunks = dispatch_chunks(&chunk_slices, cfg, opts.mode, opts.independent_chunks, tc, dom_xform);
    for c in &mut chunks { c.domain_transform = dom_xform; }
    let adaptive_rep = chunks.iter().any(|c| c.delta_flag.rep().map_or(false, |r| r != GfRep::C));
    let mut bd = BaseDistribution::default(); let (mut ts, mut ds) = (0.0f64, 0.0f64);
    for c in &chunks { match c.base { 3=>bd.base_3+=1, 13=>bd.base_13+=1, 28=>bd.base_28+=1, 70=>bd.base_70+=1, 364=>bd.base_364+=1, _=>bd.base_3+=1 }
        ts+=c.tau; ds+=c.delta; }
    let n = chunks.len().max(1) as f64; let (at,ad) = (ts/n, ds/n);
    let pb = if bd.base_364>0{364} else if bd.base_70>0{70} else if bd.base_28>0{28} else if bd.base_13>0{13} else {3};
    let compressed = build_container(&chunks, data.len() as u64, crc, opts.mode, opts.level,
        opts.independent_chunks, adaptive_rep, at, ad, pb, lp_coeffs.as_ref(), iw, opts.compute_fibonacci, opts.filename.is_some());
    let fib = if opts.compute_fibonacci { Some(fibonacci_analysis(data.len())) } else { None };
    let descs: Vec<ChunkDescriptor> = chunks.iter().map(|c| ChunkDescriptor {
        index: c.index, original_size: c.original_size, compressed_size: c.compressed_size,
        base: c.base, tau: c.tau, delta: c.delta, mode: c.mode as u8, rice_m: c.rice_m,
        delta_flag: c.delta_flag.0, delta_order: c.delta_flag.order(), delta_rep: c.delta_flag.rep_name().into(),
        domain_transform: c.domain_transform.0 }).collect();
    let csz = compressed.len() as u64;
    Ok(CompressionResult { compressed, original_size: data.len() as u64, compressed_size: csz,
        compression_ratio: if csz>0{data.len() as f64/csz as f64}else{1.0}, crc32: crc,
        mode: opts.mode as u8, mode_name: opts.mode.name().into(), version: "2.0".into(),
        level: opts.level, level_name: cfg.tier_name.into(), chunks: descs,
        avg_tau: at, avg_delta: ad, base_distribution: bd, predominant_base: pb,
        independent_chunks: opts.independent_chunks, adaptive_rep_used: adaptive_rep, fibonacci_analysis: fib })
}

// ─── Decompression (§8) ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DecompressionResult { pub data: Vec<u8>, pub original_file_name: Option<String>,
    pub original_size: u64, pub compressed_size: u64, pub version: String,
    pub level: Option<u8>, pub level_name: Option<String>, pub crc32_verified: bool }

pub fn ttc_decompress(compressed: &[u8]) -> TtcResult<DecompressionResult> {
    if compressed.len() < HEADER_SIZE { return Err(TtcError::TruncatedHeader); }
    if compressed[0..4] != MAGIC_TTC1 { return Err(TtcError::InvalidMagic); }
    let version = compressed[0x04];
    if version != VERSION_V2 && version != VERSION_V1 { return Err(TtcError::UnsupportedVersion(version)); }
    let mode = CompressionMode::from_u8(compressed[0x05])?;
    let orig_size = u64::from_be_bytes(compressed[0x06..0x0E].try_into().unwrap());
    let stored_crc = u32::from_be_bytes(compressed[0x16..0x1A].try_into().unwrap());
    let aflags = compressed[0x28]; let level = compressed[0x29];
    let chunk_count = u16::from_be_bytes(compressed[0x2A..0x2C].try_into().unwrap()) as usize;
    let cm_off = u64::from_be_bytes(compressed[0x38..0x40].try_into().unwrap()) as usize;
    let independent = aflags & 0x04 != 0; let adaptive_rep = aflags & 0x08 != 0;
    let has_filename = aflags & 0x20 != 0;
    let lp_coeffs: Option<[i16;4]> = if mode==CompressionMode::Audio { Some([
        i16::from_be_bytes([compressed[0x20],compressed[0x21]]), i16::from_be_bytes([compressed[0x22],compressed[0x23]]),
        i16::from_be_bytes([compressed[0x24],compressed[0x25]]), i16::from_be_bytes([compressed[0x26],compressed[0x27]])]) } else { None };
    let iw: Option<u64> = if mode==CompressionMode::Image { Some(u64::from_be_bytes(compressed[0x20..0x28].try_into().unwrap())) } else { None };
    let cm_end = cm_off + chunk_count * CHUNK_MAP_ENTRY_SIZE;
    if compressed.len() < cm_end { return Err(TtcError::TruncatedChunkMap); }
    struct CME { comp: u32, dflag: DeltaFlag, dxform: DomainTransform }
    let mut entries = Vec::with_capacity(chunk_count);
    for i in 0..chunk_count { let o = cm_off+i*CHUNK_MAP_ENTRY_SIZE;
        let comp = u32::from_be_bytes(compressed[o+4..o+8].try_into().unwrap());
        let pk = compressed[o+15]; let df = (pk>>5)&0x07; let dt = pk&0x07;
        if df == 7 { return Err(TtcError::InvalidDeltaFlag(df)); }
        if dt == 7 { return Err(TtcError::InvalidDomainTransform(dt)); }
        entries.push(CME { comp, dflag: DeltaFlag(df), dxform: DomainTransform(dt) }); }
    let ws = if level>=1&&level<=9 { level_config(level).ok().map(|c| c.window_size) } else { None }.unwrap_or(243*1024);
    let mut poff = cm_end; let mut output = Vec::with_capacity(orig_size as usize); let mut history: Vec<u8> = Vec::new();
    for entry in &entries { let pe = poff + entry.comp as usize;
        if compressed.len() < pe { return Err(TtcError::TruncatedPayload); }
        let payload = &compressed[poff..pe]; poff = pe;
        if payload.len() < 5 { return Err(TtcError::DecompressionError("Chunk payload too short".into())); }
        let cm = ChunkMode::from_u8(payload[4])?; let cp = &payload[5..];
        let chunk_bytes = match cm {
            ChunkMode::Stored => cp.to_vec(),
            ChunkMode::Compressed => { let toks = deserialize_compressed(cp)?;
                let hr = if independent{&[]}else{&history[..]}; let full = decompress_tokens(&toks, hr);
                if independent{full}else{full[hr.len()..].to_vec()} }
            ChunkMode::TernaryEnhanced => { let toks = deserialize_ternary_enhanced(cp, adaptive_rep)?;
                let hr = if independent{&[]}else{&history[..]}; let full = decompress_tokens(&toks, hr);
                if independent{full}else{full[hr.len()..].to_vec()} }
            ChunkMode::TernaryAns => { let toks = deserialize_tans(cp, ws)?;
                let hr = if independent{&[]}else{&history[..]}; let full = decompress_tokens(&toks, hr);
                if independent{full}else{full[hr.len()..].to_vec()} } };
        let decoded = apply_delta_decode(&chunk_bytes, entry.dflag)?;
        if !independent { history.extend_from_slice(&decoded);
            if history.len() > ws { let t = history.len()-ws; history.drain(..t); } }
        output.extend_from_slice(&decoded); }
    let gdt = entries.first().map(|e| e.dxform).unwrap_or(DomainTransform::NONE);
    let final_data = reverse_domain_preprocess(&output, gdt, lp_coeffs.as_ref(), iw)?;
    let (actual_data, original_file_name) = if has_filename {
        let (fname, content) = extract_filename(&final_data);
        if !fname.is_empty() { (content.to_vec(), Some(fname)) } else { (final_data, None) }
    } else { (final_data, None) };
    let computed_crc = crc32(&actual_data);
    Ok(DecompressionResult { data: actual_data, original_file_name, original_size: orig_size,
        compressed_size: compressed.len() as u64, version: if version==VERSION_V2{"2.0"}else{"1.1"}.into(),
        level: if version==VERSION_V2{Some(level)}else{None},
        level_name: if version==VERSION_V2{level_config(level).ok().map(|c| c.tier_name.into())}else{None},
        crc32_verified: computed_crc == stored_crc })
}

// ─── Multi-File (§9.2) ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MultiFileResult { pub compressed: Vec<u8>, pub total_original_size: u64, pub total_compressed_size: u64,
    pub compression_ratio: f64, pub file_count: usize, pub files: Vec<FileEntry>,
    pub avg_tau: f64, pub avg_delta: f64, pub base_distribution: BaseDistribution,
    pub predominant_base: u16, pub mode_name: String, pub version: String,
    pub level: u8, pub level_name: String, pub adaptive_rep_used: bool,
    pub fibonacci_analysis: Option<FibonacciAnalysis> }
#[derive(Debug, Clone)]
pub struct FileEntry { pub name: String, pub original_size: u64, pub compressed_size: u64, pub ratio: f64 }

pub fn ttc_compress_multi(files: &[(&str, &[u8])], opts: &CompressOptions) -> TtcResult<MultiFileResult> {
    let cfg = level_config(opts.level)?;
    let mut archives: Vec<(String, Vec<u8>, u64, u64)> = Vec::new();
    let (mut to, mut tc_sum) = (0u64, 0u64); let (mut ts, mut ds) = (0.0f64, 0.0f64);
    let mut bd = BaseDistribution::default(); let mut any_ar = false;
    for &(name, data) in files { let mut fo = opts.clone(); fo.filename = Some(name.to_string());
        let r = ttc_compress(data, &fo)?; to+=r.original_size; tc_sum+=r.compressed_size; ts+=r.avg_tau; ds+=r.avg_delta;
        bd.base_3+=r.base_distribution.base_3; bd.base_13+=r.base_distribution.base_13;
        bd.base_28+=r.base_distribution.base_28; bd.base_70+=r.base_distribution.base_70; bd.base_364+=r.base_distribution.base_364;
        if r.adaptive_rep_used { any_ar = true; }
        archives.push((name.into(), r.compressed, r.original_size, r.compressed_size)); }
    let mut out = Vec::new(); out.extend_from_slice(&MAGIC_TTCM); out.extend_from_slice(&(files.len() as u32).to_be_bytes());
    for (name, _, orig, comp) in &archives { let nb = name.as_bytes();
        out.extend_from_slice(&(nb.len() as u16).to_be_bytes()); out.extend_from_slice(nb);
        out.extend_from_slice(&(*orig as u32).to_be_bytes()); out.extend_from_slice(&(*comp as u32).to_be_bytes()); }
    for (_, arc, _, _) in &archives { out.extend_from_slice(arc); }
    let n = files.len().max(1) as f64;
    let pb = if bd.base_364>0{364} else if bd.base_70>0{70} else if bd.base_28>0{28} else if bd.base_13>0{13} else {3};
    Ok(MultiFileResult { total_compressed_size: out.len() as u64, compressed: out, total_original_size: to,
        compression_ratio: if tc_sum>0{to as f64/tc_sum as f64}else{1.0}, file_count: files.len(),
        files: archives.iter().map(|(n,_,o,c)| FileEntry { name: n.clone(), original_size: *o, compressed_size: *c,
            ratio: if *c>0{*o as f64/ *c as f64}else{1.0} }).collect(),
        avg_tau: ts/n, avg_delta: ds/n, base_distribution: bd, predominant_base: pb,
        mode_name: opts.mode.name().into(), version: "2.0".into(), level: opts.level, level_name: cfg.tier_name.into(),
        adaptive_rep_used: any_ar, fibonacci_analysis: if opts.compute_fibonacci{Some(fibonacci_analysis(to as usize))}else{None} })
}
pub fn ttc_decompress_multi(compressed: &[u8]) -> TtcResult<Vec<(String, Vec<u8>)>> {
    if compressed.len() < 8 { return Err(TtcError::TruncatedHeader); }
    if compressed[0..4] != MAGIC_TTCM { return Err(TtcError::InvalidMagic); }
    let fc = u32::from_be_bytes(compressed[4..8].try_into().unwrap()) as usize; let mut pos = 8;
    let mut ft: Vec<(String, u32)> = Vec::with_capacity(fc);
    for _ in 0..fc { if pos+2>compressed.len(){return Err(TtcError::TruncatedHeader);}
        let nl = u16::from_be_bytes([compressed[pos],compressed[pos+1]]) as usize; pos+=2;
        if pos+nl+8>compressed.len(){return Err(TtcError::TruncatedHeader);}
        let name = sanitize_filename(&String::from_utf8_lossy(&compressed[pos..pos+nl])); pos+=nl;
        let _orig = u32::from_be_bytes(compressed[pos..pos+4].try_into().unwrap()); pos+=4;
        let comp = u32::from_be_bytes(compressed[pos..pos+4].try_into().unwrap()); pos+=4;
        ft.push((name, comp)); }
    let mut results = Vec::with_capacity(fc);
    for (name, comp) in &ft { let ae = pos+*comp as usize;
        if ae>compressed.len(){return Err(TtcError::TruncatedPayload);}
        let r = ttc_decompress(&compressed[pos..ae])?; pos = ae;
        results.push((r.original_file_name.unwrap_or_else(|| name.clone()), r.data)); } Ok(results)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType { Single, Multi, Unknown }
#[inline] pub fn detect_archive_type(data: &[u8]) -> ArchiveType {
    if data.len() >= 4 { if data[0..4]==MAGIC_TTC1{return ArchiveType::Single;} if data[0..4]==MAGIC_TTCM{return ArchiveType::Multi;} }
    ArchiveType::Unknown
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_crc32_known_vectors() {
        assert_eq!(crc32_software(b""), 0x0000_0000);
        assert_eq!(crc32_software(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32_software(b"The quick brown fox jumps over the lazy dog"), 0x414F_A339);
    }
    #[test] fn test_tribonacci_codec_round_trip() {
        for n in 0..2000u64 { let enc = encode_tribonacci(n); let dec = decode_tribonacci(&enc);
            assert_eq!(n, dec, "Tribonacci round-trip failed for n={n}"); } }
    #[test] fn test_elias_gamma_round_trip() {
        for n in 0..500u64 { let mut w = BitWriter::new(); encode_elias_gamma(&mut w, n); let d = w.finish();
            let mut r = BitReader::new(&d); assert_eq!(n, decode_elias_gamma(&mut r)); } }
    #[test] fn test_rice_round_trip() {
        for m in 1..=8u8 { for n in 0..200u64 { let mut w = BitWriter::new(); encode_rice(&mut w, n, m); let d = w.finish();
            let mut r = BitReader::new(&d); assert_eq!(n, decode_rice(&mut r, m)); } } }
    #[test] fn test_hybrid_prefix_round_trip() {
        for n in 0..500u64 { let mut w = BitWriter::new(); encode_hybrid_prefix(&mut w, n); let d = w.finish();
            let mut r = BitReader::new(&d); assert_eq!(n, decode_hybrid_prefix(&mut r)); } }
    #[test] fn test_bijective_ternary_round_trip() {
        for b in 0..=255u8 { let td = byte_to_bijective(b); assert!(td.len>=1&&td.len<=6);
            assert_eq!(b, bijective_to_byte(&td)); } }
    #[test] fn test_standard_ternary_round_trip() {
        for b in 0..=255u8 { assert_eq!(b, standard_to_byte(&byte_to_standard(b))); } }
    #[test] fn test_balanced_ternary_round_trip() {
        for b in 0..=255u8 { assert_eq!(b, balanced_to_byte(&byte_to_balanced(b))); } }
    #[test] fn test_delta_all_flags_round_trip() {
        let data: Vec<u8> = (0..256).map(|i| (i&0xFF) as u8).collect();
        let mut out = Vec::with_capacity(data.len()); let mut scratch = Vec::with_capacity(data.len());
        for flag in 0..=6u8 { let df = DeltaFlag(flag);
            apply_delta_encode(&data, df, &mut out, &mut scratch);
            let dec = apply_delta_decode(&out, df).unwrap();
            assert_eq!(data, dec, "Delta round-trip failed for flag={flag}"); } }
    #[test] fn test_delta_flag_7_rejected() { assert!(apply_delta_decode(&[0u8;10], DeltaFlag(7)).is_err()); }
    #[test] fn test_entropy_bounds() {
        let uniform: Vec<u8> = (0..=255).collect(); assert!((compute_entropy(&uniform)-8.0).abs()<0.01);
        assert!(compute_entropy(&vec![42u8;1000]) < 0.01); }
    #[test] fn test_level_configs() {
        for l in 1..=9u8 { let c = level_config(l).unwrap(); assert_eq!(c.level,l); assert!(c.window_size>0); }
        assert!(level_config(0).is_err()); assert!(level_config(10).is_err()); }
    #[test] fn test_parse_level_aliases() { assert_eq!(parse_level("5").unwrap(),5);
        assert_eq!(parse_level("TTC2-2").unwrap(),5); assert_eq!(parse_level("TTC3-3").unwrap(),9);
        assert!(parse_level("invalid").is_err()); }
    #[test] fn test_genomic_round_trip() {
        let dna = b"ACGTACGTACGTACGT"; assert_eq!(genomic_decode(&genomic_encode(dna)), dna.to_vec()); }
    #[test] fn test_source_round_trip() {
        let src = b"function main() { return 42; }"; assert_eq!(source_decode(&source_encode(src)), src.to_vec()); }
    #[test] fn test_log_round_trip_basic() {
        let log = b"2026-03-15T10:30:00 INFO server: Request processed\n2026-03-15T10:30:01 DEBUG server: Cache hit\n";
        let dec = log_decode(&log_encode(log)); assert!(!dec.is_empty());
        assert!(dec.windows(7).any(|w| w==b"Request")); assert!(dec.windows(5).any(|w| w==b"Cache")); }
    #[test] fn test_structured_json_round_trip() {
        let json = br#"{"name":"alice","age":30,"name":"bob","age":25}"#;
        let dec = structured_decode(&structured_encode(json));
        assert!(dec.windows(4).any(|w| w==b"name")); assert!(dec.windows(3).any(|w| w==b"age")); }
    #[test] fn test_structured_csv_round_trip() {
        let csv = b"name,age,score\nalice,30,95\nbob,25,88\n";
        let dec = structured_decode(&structured_encode(csv));
        assert!(dec.windows(4).any(|w| w==b"name")); assert!(dec.windows(5).any(|w| w==b"alice")); }
    #[test] fn test_filename_embed_extract() {
        let embedded = embed_filename(b"hello world", "test.txt");
        let (name, content) = extract_filename(&embedded);
        assert_eq!(name, "test.txt"); assert_eq!(content, b"hello world"); }
    #[test] fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("foo/../bar"), "foo_bar");
        assert_eq!(sanitize_filename("...."), "unnamed");
        assert_eq!(sanitize_filename("/"), "unnamed");
        assert_eq!(sanitize_filename("a/b\\c"), "a_b_c"); }

    // ─── Trit Stream I/O tests (NEW for v4) ────────────────────────────────

    #[test] fn test_trit_stream_round_trip() {
        let test_trits: Vec<u8> = vec![0, 1, 2, 0, 1, 2, 1, 0, 2, 2, 1, 0, 0];
        let mut writer = TritStreamWriter::new();
        for &t in &test_trits { writer.write_trit(t); }
        let packed = writer.finish();
        let mut reader = TritStreamReader::new(&packed);
        let mut decoded = Vec::new();
        while !reader.is_exhausted() { decoded.push(reader.read_trit()); }
        assert_eq!(test_trits, decoded, "Trit stream round-trip failed");
    }

    #[test] fn test_trit_stream_packing_efficiency() {
        // 5 trits per byte: 243 values fit in 1 byte
        let mut writer = TritStreamWriter::new();
        for _ in 0..50 { writer.write_trit(1); }
        let packed = writer.finish();
        // 50 trits = 10 full bytes + 4-byte header
        assert_eq!(packed.len(), 14, "50 trits should pack into 10 bytes + 4-byte header");
    }

    #[test] fn test_trit_stream_remainder_handling() {
        // 7 trits = 1 full group of 5 + 2 remainder
        let mut writer = TritStreamWriter::new();
        for &t in &[2u8, 1, 0, 2, 1, 0, 2] { writer.write_trit(t); }
        let packed = writer.finish();
        let mut reader = TritStreamReader::new(&packed);
        let decoded: Vec<u8> = (0..7).map(|_| reader.read_trit()).collect();
        assert_eq!(decoded, vec![2, 1, 0, 2, 1, 0, 2]);
    }

    // ─── Ternary ANS compliance tests (NEW for v4) ─────────────────────────

    #[test] fn test_tans_standalone_ternary_round_trip() {
        let cfg = level_config(3).unwrap();
        let data: Vec<u8> = (0..1024).map(|i| (i % 91 + 32) as u8).collect();
        let tokens = tokenize_greedy_lazy(&data, 0, cfg);
        eprintln!("tANS ternary standalone: {} tokens from {} bytes", tokens.len(), data.len());
        let serialized = serialize_tans(&tokens, cfg.window_size);
        eprintln!("  serialized: {} bytes", serialized.len());
        let decoded_tokens = deserialize_tans(&serialized, cfg.window_size).unwrap();
        eprintln!("  decoded: {} tokens", decoded_tokens.len());
        let reconstructed = decompress_tokens(&decoded_tokens, &[]);
        eprintln!("  reconstructed: {} bytes", reconstructed.len());
        assert_eq!(reconstructed, data, "Ternary ANS standalone round-trip failed");
    }

    #[test] fn test_tans_ternary_with_matches() {
        // Ensure ternary ANS handles match tokens (distances via Rice side channel)
        let cfg = level_config(3).unwrap();
        let text = b"The ternary hypercube provides 26 tunnels. The ternary hypercube provides 26 tunnels.";
        let tokens = tokenize_greedy_lazy(text, 0, cfg);
        let has_matches = tokens.iter().any(|t| matches!(t, Token::Match { .. }));
        eprintln!("tANS ternary with matches: {} tokens, has_matches={}", tokens.len(), has_matches);
        let serialized = serialize_tans(&tokens, cfg.window_size);
        let decoded_tokens = deserialize_tans(&serialized, cfg.window_size).unwrap();
        let reconstructed = decompress_tokens(&decoded_tokens, &[]);
        assert_eq!(reconstructed, text.to_vec(), "Ternary ANS with matches round-trip failed");
    }

    #[test] fn test_rans_stress_diverse_patterns() {
        // Fuzz-style stress test: exercises rANS state machine with diverse data
        // patterns that explore different frequency distributions and state ranges.
        // This catches the v4 bug where offset >= fs caused silent corruption.
        let cfg = level_config(3).unwrap();
        let patterns: Vec<(&str, Vec<u8>)> = vec![
            // All same byte → extreme frequency skew (one symbol dominates)
            ("constant", vec![0xAA; 512]),
            // Two-byte alternating → balanced two-symbol distribution
            ("alternating", (0..512).map(|i| if i % 2 == 0 { 0x00 } else { 0xFF }).collect()),
            // Sequential ramp → all 256 symbols present
            ("ramp", (0..512).map(|i| (i & 0xFF) as u8).collect()),
            // Pseudo-random (LCG) → broad distribution, tests many state values
            ("random_lcg", {
                let mut v = Vec::with_capacity(2048); let mut s = 0x1337u32;
                for _ in 0..2048 { s = s.wrapping_mul(1103515245).wrapping_add(12345);
                    v.push((s >> 16) as u8); } v }),
            // Repeated short pattern → high match density
            ("repeating_short", b"ABCABC".repeat(200)),
            // Nearly incompressible (uniform) → stress ANS with high entropy
            ("high_entropy", (0..1024).map(|i| ((i*7+13) % 256) as u8).collect()),
            // Very small (edge case) → few tokens
            ("tiny", b"Hi!".to_vec()),
            // Power-of-3 aligned (Tribonacci resonance)
            ("trib_aligned", vec![0u8; 729]),
            // Sparse (mostly zeros with occasional spikes)
            ("sparse", (0..1024).map(|i| if i % 13 == 0 { 0xFF } else { 0x00 }).collect()),
        ];
        for (name, data) in &patterns {
            let tokens = tokenize_greedy_lazy(data, 0, cfg);
            if tokens.is_empty() { continue; }
            let serialized = serialize_tans(&tokens, cfg.window_size);
            let decoded = deserialize_tans(&serialized, cfg.window_size)
                .unwrap_or_else(|e| panic!("rANS deser failed for pattern '{name}': {e}"));
            let reconstructed = decompress_tokens(&decoded, &[]);
            assert_eq!(reconstructed, *data,
                "rANS round-trip failed for pattern '{name}' ({} bytes, {} tokens)",
                data.len(), tokens.len());
        }
        eprintln!("rANS stress test: all {} patterns passed", patterns.len());
    }

    // ─── Full round-trip tests ──────────────────────────────────────────────

    #[test] fn test_compress_decompress_round_trip() {
        let data = b"Hello, World! This is a test of TTC v2.0 compression. \
            The quick brown fox jumps over the lazy dog. Repeated: \
            The quick brown fox jumps over the lazy dog. \
            PlenumNET 13-dimensional hypercube with 26 tunnels and 364 degrees.";
        let opts = CompressOptions { mode: CompressionMode::Temporal, level: 3, independent_chunks: true,
            filename: Some("test.txt".into()), ..Default::default() };
        let result = ttc_compress(data, &opts).unwrap();
        assert!(result.compressed_size > 0); assert_eq!(result.crc32, crc32(data));
        let dec = ttc_decompress(&result.compressed).unwrap();
        assert_eq!(dec.data, data.to_vec()); assert_eq!(dec.original_file_name, Some("test.txt".into()));
        assert!(dec.crc32_verified); }
    #[test] fn test_stored_mode_constant_data() {
        let data = vec![0u8; 200];
        let opts = CompressOptions { mode: CompressionMode::Basic, level: 1, independent_chunks: true, ..Default::default() };
        let dec = ttc_decompress(&ttc_compress(&data, &opts).unwrap().compressed).unwrap();
        assert_eq!(dec.data, data); }
    #[test] fn test_multi_file_round_trip() {
        let f1 = b"First file content for multi-file testing.";
        let f2 = b"Second file with different content to verify.";
        let result = ttc_compress_multi(&[("test1.txt", f1.as_slice()), ("test2.txt", f2.as_slice())], &CompressOptions::default()).unwrap();
        assert_eq!(result.file_count, 2);
        let dec = ttc_decompress_multi(&result.compressed).unwrap();
        assert_eq!(dec[0].1, f1.to_vec()); assert_eq!(dec[1].1, f2.to_vec()); }
    #[test] fn test_archive_detection() {
        assert_eq!(detect_archive_type(&MAGIC_TTC1), ArchiveType::Single);
        assert_eq!(detect_archive_type(&MAGIC_TTCM), ArchiveType::Multi);
        assert_eq!(detect_archive_type(b"XXXX"), ArchiveType::Unknown); }
    #[test] fn test_trit_cost_tables() {
        let t = TritCostTables::new(); assert_eq!(t.cost(0, GfRep::C), 1); assert_eq!(t.cost(0, GfRep::B), 1);
        assert!(t.cost(255, GfRep::B) <= 6); }
    #[test] fn test_gurft_constant_data() { assert!(gurft_analyze(&vec![128u8;2048]).entropy < 0.01); }
    #[test] fn test_varint_round_trip() {
        for &v in &[0u64,1,127,128,16383,16384,1_000_000,u64::MAX/2] { let mut buf=Vec::new();
            encode_varint(&mut buf, v); assert_eq!(v, decode_varint(&buf).0); } }
    #[test] fn test_varint_signed_round_trip() {
        for &v in &[0i64,1,-1,127,-128,10000,-10000,i64::MAX/2,i64::MIN/2] { let mut buf=Vec::new();
            encode_varint_signed(&mut buf, v); assert_eq!(v, decode_varint_signed(&buf).0); } }
    #[test] fn test_pre_compressed_detection() {
        assert!(is_pre_compressed(b"\x89PNG\r\n\x1a\nmore data here"));
        assert!(!is_pre_compressed(b"Hello, this is plain text content for testing")); }
    #[test] fn test_all_levels_compress_decompress() {
        let data = b"Tribonacci ternary compression test across all nine levels. \
            The 13-dimensional hypercube geometry provides 26 tunnels. \
            Repeated content: 13-dimensional hypercube geometry 26 tunnels.";
        for level in 1..=9u8 { let opts = CompressOptions { mode: CompressionMode::Temporal, level,
            independent_chunks: true, ..Default::default() };
            let dec = ttc_decompress(&ttc_compress(data, &opts).unwrap().compressed).unwrap();
            assert_eq!(dec.data, data.to_vec(), "Round-trip failed at level {level}"); } }
    #[test] fn test_compression_mode_names() {
        for m in 0..=7u8 { let mode = CompressionMode::from_u8(m).unwrap();
            assert!(!mode.name().is_empty()); assert!(!mode.allowed_bases().is_empty()); }
        assert!(CompressionMode::from_u8(8).is_err()); }
    #[test] fn test_phase1_phase2_split() {
        let data = b"Test data for phase split verification. Repeated: phase split verification.";
        let cfg = level_config(3).unwrap(); let tc = trit_cost_tables();
        let p1 = phase1_analyze(data, 0, cfg, CompressionMode::Temporal);
        let cr = phase2_compress(data, p1, &[], cfg, true, tc, DomainTransform::NONE);
        assert!(cr.compressed_size > 0); assert_eq!(cr.index, 0); }
    #[test] fn test_dispatch_sequential() {
        let data = b"Dispatch test data for independent mode. Repeated chunk content.";
        let chunks: Vec<&[u8]> = vec![data.as_slice(), data.as_slice()];
        let tc = trit_cost_tables();
        let results = dispatch_sequential(&chunks, level_config(3).unwrap(), CompressionMode::Temporal, true, tc, DomainTransform::NONE);
        assert_eq!(results.len(), 2); assert_eq!(results[0].index, 0); assert_eq!(results[1].index, 1); }
    #[test] fn test_dispatch_threshold() {
        let chunks: Vec<&[u8]> = vec![b"Small" as &[u8]; 2]; let tc = trit_cost_tables();
        let results = dispatch_chunks(&chunks, level_config(1).unwrap(), CompressionMode::Basic, true, tc, DomainTransform::NONE);
        assert_eq!(results.len(), 2); }
    #[test] fn test_full_round_trip_via_dispatch() {
        let data: Vec<u8> = (0..4000).map(|i| ((i*7+13)%256) as u8).collect();
        for &indep in &[true, false] { let opts = CompressOptions { mode: CompressionMode::Temporal, level: 3,
            independent_chunks: indep, ..Default::default() };
            let dec = ttc_decompress(&ttc_compress(&data, &opts).unwrap().compressed).unwrap();
            assert_eq!(dec.data, data, "Round-trip failed with independent={indep}"); } }
    #[test] fn test_fibonacci_analysis_runs() {
        let fa = fibonacci_analysis(1024); assert!(fa.arb_weight >= 1.0); assert!(!fa.resonance_band.is_empty()); }
    #[test] fn test_oncelock_singleton() {
        let t1 = trit_cost_tables() as *const TritCostTables;
        let t2 = trit_cost_tables() as *const TritCostTables;
        assert_eq!(t1, t2, "OnceLock must return the same instance"); }

    #[test] fn test_16kb_level1_roundtrip() {
        let text = b"The 13-dimensional ternary hypercube geometry provides exactly 26 neighbor tunnels per node. ";
        let size = 16384usize;
        let data: Vec<u8> = (0..size).map(|i| text[i % text.len()]).collect();
        let opts = CompressOptions { mode: CompressionMode::Basic, level: 1, independent_chunks: true, ..Default::default() };
        let result = ttc_compress(&data, &opts).unwrap();
        let dec = ttc_decompress(&result.compressed).unwrap();
        assert_eq!(dec.data.len(), data.len(), "Decompressed size mismatch");
        assert_eq!(dec.data, data, "Round-trip data mismatch at 16KB L1");
    }

    #[test] fn bench_ttc_compress_decompress() {
        use std::time::Instant;
        let text = b"The 13-dimensional ternary hypercube geometry provides exactly 26 neighbor tunnels per node. \
            PlenumNET leverages post-quantum cryptographic primitives including TL-Sponge-385, TL-DSA-87, \
            and TL-KEM for secure tunnel establishment. Each populated cube contains 20,726,199 unique \
            PQ-encrypted tunnels computed as 26 times 3^13 divided by 2. The Tribonacci constant governs \
            structural resonance across the ternary addressing lattice. ";
        // Each level benchmarked at its own chunk size (1 chunk) and 3× chunk size (3 chunks).
        // Sizes are pure 3^k — matching the ternary architecture exactly.
        // Levels tested in dependent mode (default for document compression).
        let cases: &[(usize, &str, u8, &str)] = &[
            // (data_size, size_label, level, trit_label)
            // ── TTC1: speed tier ──
            (19_683,       "3^9=19.2K",  1, "9-trit"),   // L1: 1 chunk
            (59_049,       "3^10=57.7K", 1, "10-trit"),  // L1: 3 chunks
            (59_049,       "3^10=57.7K", 2, "10-trit"),  // L2: 1 chunk
            (177_147,      "3^11=173K",  2, "11-trit"),  // L2: 3 chunks
            (59_049,       "3^10=57.7K", 3, "10-trit"),  // L3: 1 chunk
            (177_147,      "3^11=173K",  3, "11-trit"),  // L3: 3 chunks
            // ── TTC2: document tier ──
            (177_147,      "3^11=173K",  4, "11-trit"),  // L4: 1 chunk = TANS_L
            (531_441,      "3^12=519K",  4, "12-trit"),  // L4: 3 chunks
            (531_441,      "3^12=519K",  5, "12-trit"),  // L5: 1 chunk (document sweet spot)
            (1_594_323,    "3^13=1.52M", 5, "13-trit"),  // L5: 3 chunks
            (531_441,      "3^12=519K",  6, "12-trit"),  // L6: 1 chunk
            (1_594_323,    "3^13=1.52M", 6, "13-trit"),  // L6: 3 chunks
            // ── TTC3: max ratio (skip L8-L9 in bench — too slow for CI) ──
            (1_594_323,    "3^13=1.52M", 7, "13-trit"),  // L7: 1 chunk = hypercube
        ];
        eprintln!("\n=== TTC v2.0+v4.2 Benchmark (dependent mode, ternary rANS, pure 3^k sizes) ===");
        eprintln!("{:<14} {:<7} {:<8} {:>10} {:>10} {:>10} {:>8} {:>8} {:<7} {:<6}",
            "Size", "Trits", "Level", "Comp(us)", "Dec(us)", "Total(us)", "Ratio", "Saved%", "Mode", "Chunks");
        eprintln!("{}", "-".repeat(100));
        for &(size, label, level, trit_label) in cases {
            let data: Vec<u8> = (0..size).map(|i| text[i % text.len()]).collect();
            let opts = CompressOptions { mode: CompressionMode::Basic, level,
                independent_chunks: false, ..Default::default() };
            let iters = if size <= 59_049 { 10 } else if size <= 531_441 { 3 } else { 1 };
            let t0 = Instant::now();
            let mut result = ttc_compress(&data, &opts).unwrap();
            for _ in 1..iters { result = ttc_compress(&data, &opts).unwrap(); }
            let comp_us = t0.elapsed().as_micros() as f64 / iters as f64;
            let t1 = Instant::now();
            let mut dec = ttc_decompress(&result.compressed).unwrap();
            for _ in 1..iters { dec = ttc_decompress(&result.compressed).unwrap(); }
            let dec_us = t1.elapsed().as_micros() as f64 / iters as f64;
            assert_eq!(dec.data, data, "round-trip failed: size={} level={}", size, level);
            let ratio = data.len() as f64 / result.compressed_size as f64;
            let saved = (1.0 - result.compressed_size as f64 / data.len() as f64) * 100.0;
            let mode = result.chunks.first().map(|c| c.mode).unwrap_or(0);
            let mn = match mode { 0=>"Stored", 1=>"Comp", 2=>"TernEnh", 3=>"rANS/3", _=>"?" };
            eprintln!("{:<14} {:<7} L{:<7} {:>10.1} {:>10.1} {:>10.1} {:>8.2}x {:>6.1}% {:<7} {}",
                label, trit_label, level, comp_us, dec_us, comp_us+dec_us, ratio, saved, mn, result.chunks.len());
        }
        eprintln!(); }
}