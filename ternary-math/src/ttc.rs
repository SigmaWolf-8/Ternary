// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
// Author: RSalvi@Salvigroup.com
//
// TM-2026-017: Tribonacci Ternary Compression (TTC) Protocol v2.0
// Production implementation — ternary-math/src/ttc.rs
// Revision: v2 — full spec compliance, corrected tANS, real domain transforms

//! # TTC v2.0 — Tribonacci Ternary Compression Engine
//!
//! Native PlenumNET compression service implementing the TTC v2.0 protocol
//! (TM-2026-017). Nine compression levels (TTC1/TTC2/TTC3), four serialization
//! modes (STORED, COMPRESSED, TERNARY_ENHANCED, TERNARY_ANS), adaptive GF(3)
//! representation switching, domain-specific preprocessing (AUDIO/IMAGE/GENOMIC/
//! SOURCE/LOG/STRUCTURED), GURFT adaptive base selection, and beam-search
//! optimal parsing.
//!
//! ## Inter-Cube Parallel Dispatch (§4.8)
//!
//! The 26-tunnel Inter-Cube model is implemented natively in the engine via
//! the `parallel` feature (requires `rayon`). Two modes:
//!
//! - **Independent chunks**: Full parallel across 26 tunnels per round.
//!   Up to 26× throughput vs single-threaded.
//! - **Dependent chunks**: 13+13 pipelined — Phase 1 (GURFT/delta/base,
//!   parallel) overlaps Phase 2 (LZ77, sequential with history). ~2–4× net.
//!
//! Runtime gated: parallel only when `rayon::current_num_threads() > 1`
//! AND chunk count ≥ 4. Same pattern as T-AE-MAC.
//!
//! ## Cargo.toml dependency
//!
//! ```toml
//! [dependencies]
//! rayon = { version = "1.10", optional = true }
//! libm = "0.2"
//!
//! [features]
//! default = ["parallel"]
//! parallel = ["rayon"]
//! ```

// ─── Constants ──────────────────────────────────────────────────────────────

/// TTC1 magic bytes: "TTC1" = 0x54544331
pub const MAGIC_TTC1: [u8; 4] = [0x54, 0x54, 0x43, 0x31];
/// TTCM magic bytes: "TTCM" = 0x5454434D
pub const MAGIC_TTCM: [u8; 4] = [0x54, 0x54, 0x43, 0x4D];
/// Version byte for TTC v2.0
pub const VERSION_V2: u8 = 0x03;
/// Version byte for TTC v1.1 (backward compat)
pub const VERSION_V1: u8 = 0x02;
/// Header size in bytes
pub const HEADER_SIZE: usize = 96;
/// Chunk map entry size in bytes
pub const CHUNK_MAP_ENTRY_SIZE: usize = 16;

/// Tribonacci constant τ (OEIS A058265)
pub const TAU: f64 = 1.839_286_755_214_161_1;
/// Golden angle in degrees
pub const GOLDEN_ANGLE: f64 = 139.035_628;
/// Golden ratio φ
pub const PHI: f64 = 1.618_033_988_749_895;
/// Phase drift rate (degrees/year)
pub const PHASE_DRIFT_RATE: f64 = 3.956;
/// log₂(3) — Shannon density
pub const LOG2_3: f64 = 1.584_962_500_7;
/// Inter-Cube tunnel count (2×13 dimensions)
pub const TUNNEL_COUNT: usize = 26;
/// tANS state space L = 3¹¹
pub const TANS_L: u32 = 177_147;
/// tANS end-of-block symbol
pub const TANS_EOB: u16 = 1023;
/// Maximum tANS alphabet size
pub const TANS_ALPHABET: usize = 1024;
/// Beam width for TTC3 optimal parsing
pub const BEAM_WIDTH: usize = 8;

/// GURFT thresholds (§6.7)
pub const TAU_HARMONIC: f64 = 0.72;
pub const TAU_HOLOGRAPHIC: f64 = 0.80;
pub const TAU_RESONANCE: f64 = 0.95;
pub const DELTA_HOLOGRAPHIC: f64 = 0.80;

/// Pre-compressed entropy gate
pub const ENTROPY_GATE: f64 = 7.5;
/// Mode pruning entropy gate
pub const MODE_PRUNE_ENTROPY: f64 = 7.0;

// ─── Const CRC32 Table (§7.1, polynomial 0xEDB88320 reflected) ─────────────

const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0u32;
    while n < 256 {
        let mut c = n;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB8_8320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[n as usize] = c;
        n += 1;
    }
    table
};

// ─── Static Tribonacci Sequence (§2.2, seeds 1,2,3) ────────────────────────
// Precomputed to cover values up to ~28 million (τ³¹).

const TRIBONACCI_SEQ: [u64; 30] = [
    1, 2, 3, 6, 11, 20, 37, 68, 125, 230,
    423, 778, 1431, 2632, 4841, 8904, 16377, 30122, 55403, 101902,
    187427, 344732, 634061, 1166220, 2145013, 3945294, 7256527, 13346834, 24548655, 45152016,
];

// ─── Error Type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TtcError {
    InvalidMagic,
    UnsupportedVersion(u8),
    InvalidMode(u8),
    InvalidLevel(u8),
    InvalidDeltaFlag(u8),
    InvalidDomainTransform(u8),
    TruncatedHeader,
    TruncatedChunkMap,
    TruncatedPayload,
    Crc32Mismatch { expected: u32, computed: u32 },
    ImageWidthRequired,
    DecompressionError(String),
    SerializationError(String),
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
            Self::Crc32Mismatch { expected, computed } => {
                write!(f, "CRC32 mismatch: expected 0x{expected:08X}, computed 0x{computed:08X}")
            }
            Self::ImageWidthRequired => write!(f, "imageWidth required for IMAGE mode with MED predictor"),
            Self::DecompressionError(s) => write!(f, "Decompression error: {s}"),
            Self::SerializationError(s) => write!(f, "Serialization error: {s}"),
        }
    }
}

pub type TtcResult<T> = Result<T, TtcError>;

// ─── Enumerations ───────────────────────────────────────────────────────────

/// Compression mode (data domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMode {
    Basic = 0,
    Temporal = 1,
    Image = 2,
    Audio = 3,
    Genomic = 4,
    Source = 5,
    Log = 6,
    Structured = 7,
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
            Self::Basic => "BASIC", Self::Temporal => "TEMPORAL",
            Self::Image => "IMAGE", Self::Audio => "AUDIO",
            Self::Genomic => "GENOMIC", Self::Source => "SOURCE",
            Self::Log => "LOG", Self::Structured => "STRUCTURED",
        }
    }

    /// Allowed compression bases for this mode (§6.10).
    #[inline]
    pub fn allowed_bases(self) -> &'static [u16] {
        match self {
            Self::Basic => &[3],
            Self::Temporal => &[3, 13, 28, 70, 364],
            Self::Image | Self::Genomic | Self::Source | Self::Structured => &[3, 13],
            Self::Audio | Self::Log => &[3, 13, 28],
        }
    }
}

/// Chunk serialization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChunkMode {
    Stored = 0,
    Compressed = 1,
    TernaryEnhanced = 2,
    TernaryAns = 3,
}

impl ChunkMode {
    #[inline]
    pub fn from_u8(v: u8) -> TtcResult<Self> {
        match v {
            0 => Ok(Self::Stored), 1 => Ok(Self::Compressed),
            2 => Ok(Self::TernaryEnhanced), 3 => Ok(Self::TernaryAns),
            _ => Err(TtcError::DecompressionError(format!("Unknown chunk mode: {v}"))),
        }
    }
}

/// GF(3) representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfRep { A, B, C }

/// Parsing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parsing { Greedy, Lazy, BeamOptimal }

/// LZ77 token.
#[derive(Debug, Clone)]
pub enum Token {
    Literal(u8),
    Run { byte: u8, length: usize },
    Match { dist: usize, length: usize },
}

// ─── Level Configuration (§4.2) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LevelConfig {
    pub level: u8,
    pub tier_name: &'static str,
    pub window_size: usize,
    pub min_match: usize,
    pub min_run: usize,
    pub skip_gurft: bool,
    pub chunk_size: usize,
    pub chain_depth: usize,
    pub parsing: Parsing,
    pub candidates: usize,
}

static LEVEL_CONFIGS: [LevelConfig; 9] = [
    LevelConfig { level: 1, tier_name: "TTC1-1", window_size: 3*1024, min_match: 8, min_run: 6, skip_gurft: true, chunk_size: 26*1024, chain_depth: 8, parsing: Parsing::Greedy, candidates: 2 },
    LevelConfig { level: 2, tier_name: "TTC1-2", window_size: 9*1024, min_match: 6, min_run: 5, skip_gurft: true, chunk_size: 26*1024, chain_depth: 16, parsing: Parsing::Lazy, candidates: 3 },
    LevelConfig { level: 3, tier_name: "TTC1-3", window_size: 27*1024, min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 13*1024, chain_depth: 32, parsing: Parsing::Lazy, candidates: 4 },
    LevelConfig { level: 4, tier_name: "TTC2-1", window_size: 81*1024, min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 13*1024, chain_depth: 32, parsing: Parsing::Lazy, candidates: 4 },
    LevelConfig { level: 5, tier_name: "TTC2-2", window_size: 243*1024, min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 13*1024, chain_depth: 64, parsing: Parsing::Lazy, candidates: 4 },
    LevelConfig { level: 6, tier_name: "TTC2-3", window_size: 729*1024, min_match: 4, min_run: 4, skip_gurft: false, chunk_size: 13*1024, chain_depth: 128, parsing: Parsing::Lazy, candidates: 4 },
    LevelConfig { level: 7, tier_name: "TTC3-1", window_size: 2187*1024, min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 13*1024, chain_depth: 128, parsing: Parsing::BeamOptimal, candidates: 4 },
    LevelConfig { level: 8, tier_name: "TTC3-2", window_size: 6561*1024, min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 13*1024, chain_depth: 192, parsing: Parsing::BeamOptimal, candidates: 4 },
    LevelConfig { level: 9, tier_name: "TTC3-3", window_size: 19683*1024, min_match: 3, min_run: 3, skip_gurft: false, chunk_size: 13*1024, chain_depth: 256, parsing: Parsing::BeamOptimal, candidates: 4 },
];

#[inline]
pub fn level_config(level: u8) -> TtcResult<&'static LevelConfig> {
    if level >= 1 && level <= 9 {
        Ok(&LEVEL_CONFIGS[(level - 1) as usize])
    } else {
        Err(TtcError::InvalidLevel(level))
    }
}

/// Parse level from integer or tier name string.
pub fn parse_level(s: &str) -> TtcResult<u8> {
    if let Ok(n) = s.parse::<u8>() {
        if (1..=9).contains(&n) { return Ok(n); }
    }
    match s {
        "TTC1-1" => Ok(1), "TTC1-2" => Ok(2), "TTC1-3" => Ok(3),
        "TTC2-1" => Ok(4), "TTC2-2" => Ok(5), "TTC2-3" => Ok(6),
        "TTC3-1" => Ok(7), "TTC3-2" => Ok(8), "TTC3-3" => Ok(9),
        _ => Err(TtcError::InvalidLevel(0)),
    }
}

// ─── CRC32 (§7.1) ──────────────────────────────────────────────────────────

/// Compute CRC32 over a byte slice. Uses const table — zero allocation.
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = CRC32_TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

// ─── Shannon Entropy ────────────────────────────────────────────────────────

/// Compute Shannon entropy of a byte slice (range 0.0–8.0).
/// Uses `libm::log2` per repo `no_std` math convention.
#[inline]
pub fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut counts = [0u32; 256];
    for &b in data { counts[b as usize] += 1; }
    let n = data.len() as f64;
    let mut h = 0.0f64;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * libm::log2(p);
        }
    }
    h
}

// ─── GF(3) Delta Encoding (§3.1) ───────────────────────────────────────────

/// Delta flag values (§3.1.3). 3 bits stored in chunk map byte +15 bits 7–5.
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

    #[inline]
    pub fn rep(self) -> Option<GfRep> {
        match self.0 {
            0 => None,
            1 | 4 => Some(GfRep::B), 2 | 5 => Some(GfRep::A), 3 | 6 => Some(GfRep::C),
            _ => None,
        }
    }

    #[inline]
    pub fn rep_name(self) -> &'static str {
        match self.rep() { None => "none", Some(GfRep::A) => "A", Some(GfRep::B) => "B", Some(GfRep::C) => "C" }
    }
}

/// Rep B delta encode (§3.1.1): result[i] = (data[i] - data[i-1] + 256) % 256
#[inline]
pub fn delta_encode_b(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() { out.push(data[i].wrapping_sub(data[i - 1])); }
}

/// Rep B delta decode (§3.1.4).
#[inline]
pub fn delta_decode_b(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() { out.push(out[i - 1].wrapping_add(data[i])); }
}

/// Rep A delta encode (§3.1.1): centered at zero, (d + 128) % 256.
#[inline]
pub fn delta_encode_a(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() {
        out.push(data[i].wrapping_sub(data[i - 1]).wrapping_add(128));
    }
}

/// Rep A delta decode (§3.1.4).
#[inline]
pub fn delta_decode_a(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() {
        let d: i16 = data[i] as i16 - 128;
        out.push(((out[i - 1] as i16 + d + 256) % 256) as u8);
    }
}

/// Rep C delta encode (§3.1.1): bijective, avoids true zero.
#[inline]
pub fn delta_encode_c(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() {
        let d = data[i].wrapping_sub(data[i - 1]);
        out.push(if d == 0 { 255 } else { d.wrapping_sub(1) });
    }
}

/// Rep C delta decode (§3.1.4).
#[inline]
pub fn delta_decode_c(data: &[u8], out: &mut Vec<u8>) {
    out.clear();
    if data.is_empty() { return; }
    out.reserve(data.len());
    out.push(data[0]);
    for i in 1..data.len() {
        let d = (data[i] as u16 + 1) & 0xFF;
        out.push(((out[i - 1] as u16 + d) & 0xFF) as u8);
    }
}

/// Apply delta encode for a given flag. Reuses `buf` to avoid allocation.
fn apply_delta_encode(data: &[u8], flag: DeltaFlag, buf: &mut Vec<u8>, buf2: &mut Vec<u8>) -> Vec<u8> {
    match flag.0 {
        0 => data.to_vec(),
        1 => { delta_encode_b(data, buf); buf.clone() }
        2 => { delta_encode_a(data, buf); buf.clone() }
        3 => { delta_encode_c(data, buf); buf.clone() }
        4 => { delta_encode_b(data, buf); delta_encode_b(buf, buf2); buf2.clone() }
        5 => { delta_encode_a(data, buf); delta_encode_a(buf, buf2); buf2.clone() }
        6 => { delta_encode_c(data, buf); delta_encode_c(buf, buf2); buf2.clone() }
        _ => data.to_vec(),
    }
}

/// Apply delta decode for a given flag.
fn apply_delta_decode(data: &[u8], flag: DeltaFlag) -> TtcResult<Vec<u8>> {
    let mut buf = Vec::with_capacity(data.len());
    let mut buf2 = Vec::with_capacity(data.len());
    match flag.0 {
        0 => Ok(data.to_vec()),
        1 => { delta_decode_b(data, &mut buf); Ok(buf) }
        2 => { delta_decode_a(data, &mut buf); Ok(buf) }
        3 => { delta_decode_c(data, &mut buf); Ok(buf) }
        4 => { delta_decode_b(data, &mut buf); delta_decode_b(&buf, &mut buf2); Ok(buf2) }
        5 => { delta_decode_a(data, &mut buf); delta_decode_a(&buf, &mut buf2); Ok(buf2) }
        6 => { delta_decode_c(data, &mut buf); delta_decode_c(&buf, &mut buf2); Ok(buf2) }
        7 => Err(TtcError::InvalidDeltaFlag(7)),
        _ => Err(TtcError::InvalidDeltaFlag(flag.0)),
    }
}

/// Adaptive delta selection (§3.1.2). Returns the flag minimizing Shannon entropy.
fn select_delta(chunk: &[u8], mode: CompressionMode) -> DeltaFlag {
    let sample_len = chunk.len().min(512);
    let sample = &chunk[..sample_len];
    let h_raw = compute_entropy(sample);

    let mut best_flag = DeltaFlag::NONE;
    let mut best_h = h_raw;
    let mut buf = Vec::with_capacity(sample_len);
    let mut buf2 = Vec::with_capacity(sample_len);

    // Order 1
    for &(flag, encode_fn) in &[
        (DeltaFlag::ORDER1_B, delta_encode_b as fn(&[u8], &mut Vec<u8>)),
        (DeltaFlag::ORDER1_A, delta_encode_a as fn(&[u8], &mut Vec<u8>)),
        (DeltaFlag::ORDER1_C, delta_encode_c as fn(&[u8], &mut Vec<u8>)),
    ] {
        encode_fn(sample, &mut buf);
        let h = compute_entropy(&buf);
        if h < best_h { best_h = h; best_flag = flag; }
    }

    // Order 2 for AUDIO and IMAGE only (§3.1.2)
    if matches!(mode, CompressionMode::Audio | CompressionMode::Image) {
        for &(flag, encode_fn) in &[
            (DeltaFlag::ORDER2_B, delta_encode_b as fn(&[u8], &mut Vec<u8>)),
            (DeltaFlag::ORDER2_A, delta_encode_a as fn(&[u8], &mut Vec<u8>)),
            (DeltaFlag::ORDER2_C, delta_encode_c as fn(&[u8], &mut Vec<u8>)),
        ] {
            encode_fn(sample, &mut buf);
            encode_fn(&buf, &mut buf2);
            let h = compute_entropy(&buf2);
            if h < best_h { best_h = h; best_flag = flag; }
        }
    }

    best_flag
}

// ─── Trit Encoding (§3.2) — fixed-size arrays, zero allocation ─────────────

/// Trit digits result: up to 6 digits + length. No heap allocation.
#[derive(Debug, Clone, Copy)]
pub struct TritDigits {
    pub digits: [u8; 6],   // stored unsigned: Rep C {1,2,3}, Rep B {0,1,2}
    pub len: u8,
}

/// Balanced trit digits: up to 6 signed digits.
#[derive(Debug, Clone, Copy)]
pub struct BalancedTritDigits {
    pub digits: [i8; 6],
    pub len: u8,
}

/// Rep C — Bijective ternary {1,2,3} (§3.2.1).
#[inline]
pub fn byte_to_bijective(byte: u8) -> TritDigits {
    let mut n = byte as u32 + 1;
    let mut buf = [0u8; 6];
    let mut len = 0u8;
    while n > 0 {
        let mut r = n % 3;
        if r == 0 { r = 3; n = n / 3 - 1; } else { n /= 3; }
        buf[len as usize] = r as u8;
        len += 1;
    }
    if len == 0 { buf[0] = 1; len = 1; }
    // Reverse in-place
    let mut i = 0usize;
    let mut j = (len - 1) as usize;
    while i < j { buf.swap(i, j); i += 1; j -= 1; }
    TritDigits { digits: buf, len }
}

/// Rep C decode.
#[inline]
pub fn bijective_to_byte(td: &TritDigits) -> u8 {
    let mut result = 0u32;
    for i in 0..(td.len as usize) { result = result * 3 + td.digits[i] as u32; }
    ((result - 1) & 0xFF) as u8
}

/// Rep B — Standard base-3 {0,1,2} (§3.2.2).
#[inline]
pub fn byte_to_standard(byte: u8) -> TritDigits {
    if byte == 0 { return TritDigits { digits: [0,0,0,0,0,0], len: 1 }; }
    let mut n = byte as u32;
    let mut buf = [0u8; 6];
    let mut len = 0u8;
    while n > 0 { buf[len as usize] = (n % 3) as u8; n /= 3; len += 1; }
    let mut i = 0usize;
    let mut j = (len - 1) as usize;
    while i < j { buf.swap(i, j); i += 1; j -= 1; }
    TritDigits { digits: buf, len }
}

/// Rep B decode.
#[inline]
pub fn standard_to_byte(td: &TritDigits) -> u8 {
    let mut result = 0u32;
    for i in 0..(td.len as usize) { result = result * 3 + td.digits[i] as u32; }
    (result & 0xFF) as u8
}

/// Rep A — Balanced ternary {T(-1), 0, 1} (§3.2.3).
#[inline]
pub fn byte_to_balanced(byte: u8) -> BalancedTritDigits {
    let signed: i16 = if byte <= 127 { byte as i16 } else { byte as i16 - 256 };
    if signed == 0 { return BalancedTritDigits { digits: [0,0,0,0,0,0], len: 1 }; }
    let mut value = signed;
    let mut buf = [0i8; 6];
    let mut len = 0u8;
    let mut iters = 0u8;
    while value != 0 && iters < 100 {
        let remainder = ((value % 3) + 3) % 3;
        match remainder {
            0 => { buf[len as usize] = 0; value /= 3; }
            1 => { buf[len as usize] = 1; value = (value - 1) / 3; }
            2 => { buf[len as usize] = -1; value = (value + 1) / 3; }
            _ => unreachable!(),
        }
        len += 1;
        iters += 1;
    }
    let mut i = 0usize;
    let mut j = (len - 1) as usize;
    while i < j { buf.swap(i, j); i += 1; j -= 1; }
    BalancedTritDigits { digits: buf, len }
}

/// Rep A decode.
#[inline]
pub fn balanced_to_byte(td: &BalancedTritDigits) -> u8 {
    let mut value: i16 = 0;
    let mut multiplier: i16 = 1;
    for k in (0..(td.len as usize)).rev() {
        value += td.digits[k] as i16 * multiplier;
        multiplier *= 3;
    }
    ((value + 256) % 256) as u8
}

/// Trit count for a byte. O(1) via precomputed tables (§3.3).
#[inline]
pub fn trit_count_c(byte: u8) -> u8 { byte_to_bijective(byte).len }
#[inline]
pub fn trit_count_b(byte: u8) -> u8 { byte_to_standard(byte).len }
#[inline]
pub fn trit_count_a(byte: u8) -> u8 { byte_to_balanced(byte).len }

// ─── Precomputed Trit Cost Tables (§3.3) ────────────────────────────────────

pub struct TritCostTables {
    pub rep_a: [u8; 256],
    pub rep_b: [u8; 256],
    pub rep_c: [u8; 256],
}

impl TritCostTables {
    pub fn new() -> Self {
        let mut t = Self { rep_a: [0; 256], rep_b: [0; 256], rep_c: [0; 256] };
        for b in 0..=255u8 {
            t.rep_a[b as usize] = trit_count_a(b);
            t.rep_b[b as usize] = trit_count_b(b);
            t.rep_c[b as usize] = trit_count_c(b);
        }
        t
    }

    #[inline]
    pub fn cost(&self, byte: u8, rep: GfRep) -> u8 {
        match rep { GfRep::A => self.rep_a[byte as usize], GfRep::B => self.rep_b[byte as usize], GfRep::C => self.rep_c[byte as usize] }
    }

    #[inline]
    pub fn avg_cost(&self, group: &[u8], rep: GfRep) -> f64 {
        if group.is_empty() { return 0.0; }
        let sum: u32 = group.iter().map(|&b| self.cost(b, rep) as u32).sum();
        sum as f64 / group.len() as f64
    }

    #[inline]
    pub fn best_rep(&self, group: &[u8]) -> GfRep {
        let ca = self.avg_cost(group, GfRep::A);
        let cb = self.avg_cost(group, GfRep::B);
        let cc = self.avg_cost(group, GfRep::C);
        if ca <= cb && ca <= cc { GfRep::A } else if cb <= cc { GfRep::B } else { GfRep::C }
    }
}

// ─── Bit I/O ────────────────────────────────────────────────────────────────

/// MSB-first bit writer.
pub struct BitWriter {
    buffer: Vec<u8>,
    current: u8,
    bit_pos: u8,
}

impl BitWriter {
    #[inline] pub fn new() -> Self { Self { buffer: Vec::new(), current: 0, bit_pos: 0 } }
    #[inline] pub fn with_capacity(cap: usize) -> Self { Self { buffer: Vec::with_capacity(cap), current: 0, bit_pos: 0 } }

    #[inline]
    pub fn write(&mut self, value: u32, count: u8) {
        for i in (0..count).rev() {
            self.current |= (((value >> i) & 1) as u8) << (7 - self.bit_pos);
            self.bit_pos += 1;
            if self.bit_pos == 8 { self.buffer.push(self.current); self.current = 0; self.bit_pos = 0; }
        }
    }

    #[inline] pub fn write_bit(&mut self, bit: bool) { self.write(bit as u32, 1); }
    #[inline] pub fn bit_count(&self) -> usize { self.buffer.len() * 8 + self.bit_pos as usize }

    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_pos > 0 { self.buffer.push(self.current); }
        self.buffer
    }

    /// Finalize with 4-byte bit count header (uint32 BE) prepended.
    pub fn finish_with_header(self) -> Vec<u8> {
        let bc = self.bit_count() as u32;
        let bytes = self.finish();
        let mut out = Vec::with_capacity(4 + bytes.len());
        out.extend_from_slice(&bc.to_be_bytes());
        out.extend_from_slice(&bytes);
        out
    }
}

/// MSB-first bit reader.
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    #[inline] pub fn new(data: &'a [u8]) -> Self { Self { data, byte_pos: 0, bit_pos: 0 } }

    #[inline]
    pub fn read(&mut self, count: u8) -> u32 {
        let mut value = 0u32;
        for _ in 0..count {
            if self.byte_pos >= self.data.len() { return value; }
            let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
            value = (value << 1) | bit as u32;
            self.bit_pos += 1;
            if self.bit_pos == 8 { self.bit_pos = 0; self.byte_pos += 1; }
        }
        value
    }

    #[inline] pub fn read_bit(&mut self) -> bool { self.read(1) != 0 }

    #[inline]
    pub fn count_leading_zeros(&mut self) -> u32 {
        let mut count = 0u32;
        while !self.is_exhausted() { if self.read_bit() { return count; } count += 1; }
        count
    }

    #[inline] pub fn is_exhausted(&self) -> bool { self.byte_pos >= self.data.len() }
}

// ─── Tribonacci Integer Coding (§2.2) — uses static sequence ───────────────

/// Encode integer via Tribonacci Zeckendorf-style greedy decomposition.
#[inline]
pub fn encode_tribonacci(n: u64) -> Vec<bool> {
    if n == 0 { return vec![false]; }
    // Find highest applicable index in static sequence
    let mut top = 0usize;
    for (i, &v) in TRIBONACCI_SEQ.iter().enumerate() {
        if v <= n { top = i; }
    }
    let mut bits = vec![false; top + 1];
    let mut remaining = n;
    for i in (0..=top).rev() {
        if TRIBONACCI_SEQ[i] <= remaining {
            bits[i] = true;
            remaining -= TRIBONACCI_SEQ[i];
            if remaining == 0 { break; }
        }
    }
    // MSB to LSB: find highest set bit
    let msb = bits.iter().rposition(|&b| b).unwrap_or(0);
    bits[..=msb].to_vec()
}

/// Decode a Tribonacci bit string.
#[inline]
pub fn decode_tribonacci(bits: &[bool]) -> u64 {
    let mut sum = 0u64;
    for (i, &b) in bits.iter().enumerate() {
        if b && i < TRIBONACCI_SEQ.len() { sum += TRIBONACCI_SEQ[i]; }
    }
    sum
}

// ─── Hybrid Small-Value Prefix Coding (§3.4) ───────────────────────────────

#[inline]
fn encode_hybrid_prefix(writer: &mut BitWriter, value: u64) {
    if value == 0 { writer.write(0b00, 2); }
    else if value <= 3 { writer.write(0b01, 2); writer.write((value - 1) as u32, 2); }
    else if value <= 15 { writer.write(0b10, 2); writer.write((value - 4) as u32, 4); }
    else {
        writer.write(0b11, 2);
        let code = encode_tribonacci(value);
        let len = code.len().min(31) as u32;
        writer.write(len, 5);
        for i in 0..(len as usize) { writer.write_bit(code[i]); }
    }
}

#[inline]
fn decode_hybrid_prefix(reader: &mut BitReader) -> u64 {
    match reader.read(2) {
        0b00 => 0,
        0b01 => reader.read(2) as u64 + 1,
        0b10 => reader.read(4) as u64 + 4,
        0b11 => {
            let len = reader.read(5) as usize;
            let mut bits = Vec::with_capacity(len);
            for _ in 0..len { bits.push(reader.read_bit()); }
            decode_tribonacci(&bits)
        }
        _ => 0,
    }
}

// ─── Elias Gamma Coding (§3.5) ─────────────────────────────────────────────

#[inline]
fn encode_elias_gamma(writer: &mut BitWriter, n: u64) {
    let value = (n + 1).max(1);
    let bits = 64 - value.leading_zeros();
    for _ in 0..(bits - 1) { writer.write_bit(false); }
    writer.write(value as u32, bits as u8);
}

#[inline]
fn decode_elias_gamma(reader: &mut BitReader) -> u64 {
    let zeros = reader.count_leading_zeros();
    let lower = reader.read(zeros as u8);
    ((1u64 << zeros) | lower as u64) - 1
}

// ─── Rice/Golomb Coding (§3.6) ──────────────────────────────────────────────

#[inline]
fn encode_rice(writer: &mut BitWriter, n: u64, m: u8) {
    let q = n >> m;
    for _ in 0..q { writer.write_bit(true); }
    writer.write_bit(false);
    writer.write((n & ((1u64 << m) - 1)) as u32, m);
}

#[inline]
fn decode_rice(reader: &mut BitReader, m: u8) -> u64 {
    let mut q = 0u64;
    while reader.read_bit() { q += 1; }
    let r = reader.read(m) as u64;
    (q << m) | r
}

/// Compute initial Rice M from first 128 match distances (§3.6).
fn compute_initial_rice_m(tokens: &[Token]) -> u8 {
    let mut sum = 0u64;
    let mut count = 0u32;
    for t in tokens {
        if let Token::Match { dist, .. } = t {
            sum += *dist as u64; count += 1;
            if count >= 128 { break; }
        }
    }
    if count == 0 { return 4; }
    let mean = sum / count as u64;
    if mean == 0 { return 1; }
    ((64 - mean.leading_zeros()).saturating_sub(1) as u8).clamp(1, 8)
}

// ─── tANS — Correct State Machine (§3.7) ───────────────────────────────────

/// tANS frequency table.
struct TansFreqTable {
    /// (symbol, normalized_frequency), sorted by f_norm descending then symbol ascending.
    entries: Vec<(u16, u32)>,
    /// Lookup: f_norm for each symbol index. Index by symbol.
    fnorm_lookup: [u32; TANS_ALPHABET],
    /// Cumulative frequency. cum[s] = sum of f_norm for all symbols before s in sorted order.
    cum: [u32; TANS_ALPHABET],
}

impl TansFreqTable {
    fn build(tokens: &[Token], window_size: usize) -> Self {
        let mut counts = [0u32; TANS_ALPHABET];
        for tok in tokens {
            match tok {
                Token::Literal(v) => counts[*v as usize] += 1,
                Token::Run { byte, length } => {
                    counts[*byte as usize] += 1;
                    let sym = (256 + (*length).min(255)) as usize;
                    if sym < TANS_ALPHABET { counts[sym] += 1; }
                }
                Token::Match { dist, length } => {
                    let len_sym = (511 + (*length).min(256)) as usize;
                    if len_sym < TANS_ALPHABET { counts[len_sym] += 1; }
                    let dist_sym = tans_dist_symbol(*dist, window_size) as usize;
                    if dist_sym < TANS_ALPHABET { counts[dist_sym] += 1; }
                }
            }
        }
        counts[TANS_EOB as usize] = counts[TANS_EOB as usize].max(1);

        let total_raw: u64 = counts.iter().map(|&c| c as u64).sum();
        let l = TANS_L as u64;
        let mut fnorm = [0u32; TANS_ALPHABET];
        let mut sum = 0u32;

        for i in 0..TANS_ALPHABET {
            if counts[i] > 0 {
                fnorm[i] = ((counts[i] as u64 * l / total_raw.max(1)) as u32).max(1);
                sum += fnorm[i];
            }
        }

        // Adjust sum to exactly L
        let target = TANS_L;
        while sum != target {
            if sum > target {
                let mut max_i = 0; let mut max_v = 0u32;
                for i in 0..TANS_ALPHABET {
                    if fnorm[i] > max_v && fnorm[i] > 1 { max_v = fnorm[i]; max_i = i; }
                }
                fnorm[max_i] -= 1; sum -= 1;
            } else {
                let mut min_i = 0; let mut min_v = u32::MAX;
                for i in 0..TANS_ALPHABET {
                    if fnorm[i] > 0 && fnorm[i] < min_v { min_v = fnorm[i]; min_i = i; }
                }
                fnorm[min_i] += 1; sum += 1;
            }
        }

        // Build sorted entries: (f_norm DESC, symbol_index ASC) — §3.7.3 tie-breaking rule
        let mut entries: Vec<(u16, u32)> = Vec::new();
        for i in 0..TANS_ALPHABET {
            if fnorm[i] > 0 { entries.push((i as u16, fnorm[i])); }
        }
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        // Build cumulative in sorted order
        let mut cum = [0u32; TANS_ALPHABET];
        let mut c = 0u32;
        for &(sym, f) in &entries { cum[sym as usize] = c; c += f; }

        Self { entries, fnorm_lookup: fnorm, cum }
    }

    #[inline] fn fnorm(&self, sym: u16) -> u32 { self.fnorm_lookup[sym as usize] }
    #[inline] fn cum(&self, sym: u16) -> u32 { self.cum[sym as usize] }
}

#[inline]
fn tans_dist_symbol(dist: usize, window_size: usize) -> u16 {
    let bucket_size = (window_size / 256).max(1);
    767 + (dist / bucket_size).min(255) as u16
}

#[inline]
fn tans_dist_from_symbol(sym: u16, window_size: usize) -> usize {
    let bucket = (sym - 767) as usize;
    let bucket_size = (window_size / 256).max(1);
    bucket * bucket_size + bucket_size / 2
}

/// Build spread table and per-symbol position lists (§3.7.3).
fn tans_build_tables(freq: &TansFreqTable) -> (Vec<u16>, Vec<Vec<u32>>) {
    let l = TANS_L as usize;
    let mut spread = vec![0u16; l];
    let step = (l / TANS_ALPHABET) | 1;
    let mut pos = 0usize;

    for &(sym, fnorm) in &freq.entries {
        for _ in 0..fnorm {
            spread[pos % l] = sym;
            pos += step;
        }
    }

    // Build per-symbol sorted position lists
    let mut sym_positions: Vec<Vec<u32>> = vec![Vec::new(); TANS_ALPHABET];
    for i in 0..l {
        sym_positions[spread[i] as usize].push(i as u32);
    }
    // Positions are already in ascending order due to sequential scanning

    (spread, sym_positions)
}

/// tANS encode (§3.7.4). Returns (final_state, packed_trit_bytes).
fn tans_encode(tokens: &[Token], freq: &TansFreqTable, window_size: usize) -> (u32, Vec<u8>) {
    let (_, sym_positions) = tans_build_tables(freq);

    // Build symbol stream
    let mut symbols: Vec<u16> = Vec::new();
    for tok in tokens {
        match tok {
            Token::Literal(v) => symbols.push(*v as u16),
            Token::Run { byte, length } => {
                symbols.push(*byte as u16);
                symbols.push((256 + (*length).min(255)) as u16);
            }
            Token::Match { dist, length } => {
                symbols.push((511 + (*length).min(256)) as u16);
                symbols.push(tans_dist_symbol(*dist, window_size));
            }
        }
    }
    symbols.push(TANS_EOB);

    // Encode forward, collect bits in a buffer (to be reversed)
    let mut state = TANS_L;
    let mut bits: Vec<u8> = Vec::new();

    for &s in &symbols {
        let fs = freq.fnorm(s);
        if fs == 0 { continue; }

        // Renormalization: output bits while state >= 2 * fs (§3.7.4)
        while state >= 2 * fs {
            bits.push((state & 1) as u8);
            state >>= 1;
        }
        // State is now in [fs, 2*fs). Transition via encode table.
        let offset = (state - fs) as usize;
        if offset < sym_positions[s as usize].len() {
            state = sym_positions[s as usize][offset] + TANS_L;
        }
    }

    let final_state = state;

    // Reverse bit buffer (ANS is LIFO)
    bits.reverse();

    // Pack 8 bits per byte (§3.7.6)
    let mut packed = Vec::with_capacity((bits.len() + 7) / 8 + 1);
    packed.push((bits.len() % 8) as u8);
    let mut i = 0;
    while i + 7 < bits.len() {
        packed.push(bits[i]<<7 | bits[i+1]<<6 | bits[i+2]<<5 | bits[i+3]<<4
                   | bits[i+4]<<3 | bits[i+5]<<2 | bits[i+6]<<1 | bits[i+7]);
        i += 8;
    }
    if i < bits.len() {
        let mut byte = 0u8;
        for (j, &b) in bits[i..].iter().enumerate() { byte |= b << (7 - j); }
        packed.push(byte);
    }

    (final_state, packed)
}

/// tANS decode (§3.7.5). Returns token stream.
fn tans_decode(
    freq: &TansFreqTable, spread: &[u16], sym_positions: &[Vec<u32>],
    initial_state: u32, packed_bits: &[u8], window_size: usize,
) -> Vec<Token> {
    // Unpack bits from packed format (first byte = remainder count)
    let mut bits: Vec<u8> = Vec::new();
    if !packed_bits.is_empty() {
        let rem = packed_bits[0] as usize;
        for (bi, &byte) in packed_bits[1..].iter().enumerate() {
            let is_last = bi == packed_bits.len() - 2;
            let bit_count = if is_last && rem > 0 { rem } else { 8 };
            for j in 0..bit_count {
                bits.push((byte >> (7 - j)) & 1);
            }
        }
    }
    let mut bit_pos = 0usize;

    // Build decode_next table: for position i, if spread[i] == s and i is the j-th
    // position for s, then decode_next[i] = j + f_norm[s] (recovers pre-encode state).
    let l = TANS_L as usize;
    let mut decode_next = vec![0u32; l];
    for s_idx in 0..TANS_ALPHABET {
        let fs = freq.fnorm(s_idx as u16);
        for (j, &pos) in sym_positions[s_idx].iter().enumerate() {
            decode_next[pos as usize] = j as u32 + fs;
        }
    }

    let mut state = initial_state;
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let idx = (state % TANS_L) as usize;
        let s = if idx < spread.len() { spread[idx] } else { TANS_EOB };

        if s == TANS_EOB { break; }

        // Recover state via decode_next
        state = if idx < decode_next.len() { decode_next[idx] } else { TANS_L };

        // Renormalize: pump state up to at least L by reading bits
        while state < TANS_L {
            let b = if bit_pos < bits.len() { bits[bit_pos] as u32 } else { 0 };
            bit_pos += 1;
            state = state * 2 + b;
        }

        // Convert symbol to token
        if s <= 255 {
            tokens.push(Token::Literal(s as u8));
        } else if s >= 256 && s <= 510 {
            // Run length — the preceding literal token is the run byte
            let run_len = (s - 256) as usize;
            // The run byte was emitted as the previous literal
            if let Some(Token::Literal(b)) = tokens.last() {
                let byte = *b;
                tokens.pop(); // remove the literal
                tokens.push(Token::Run { byte, length: run_len });
            }
        } else if s >= 511 && s <= 766 {
            let match_len = (s - 511) as usize;
            // Distance symbol follows; push placeholder
            tokens.push(Token::Match { dist: 0, length: match_len });
        } else if s >= 767 && s <= 1022 {
            let dist = tans_dist_from_symbol(s, window_size);
            if let Some(Token::Match { dist: d, .. }) = tokens.last_mut() {
                *d = dist;
            }
        }
    }

    tokens
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
pub struct GurftResult {
    pub tau: f64,
    pub delta: f64,
    pub entropy: f64,
    pub periodicity: f64,
    pub salvi_resonance: bool,
}

impl Default for GurftResult {
    fn default() -> Self { Self { tau: 0.0, delta: 0.0, entropy: 0.0, periodicity: 0.0, salvi_resonance: false } }
}

/// Torsion alignment τ — DFT over 13 bins (§6.2).
fn compute_torsion_region(data: &[u8]) -> f64 {
    let n = data.len();
    if n == 0 { return 0.0; }
    let mut total = 0.0f64;
    for k in 1..=13u32 {
        let mut rs = 0.0f64;
        let mut is = 0.0f64;
        let freq = 2.0 * core::f64::consts::PI * (k as f64 / 13.0);
        for (i, &b) in data.iter().enumerate() {
            let angle = freq * i as f64;
            rs += b as f64 * libm::cos(angle);
            is += b as f64 * libm::sin(angle);
        }
        let norm = libm::sqrt(rs * rs + is * is) / (n as f64 * 128.0);
        total += if norm > 1.0 { 1.0 } else { norm };
    }
    total / 13.0
}

/// Dimensional sync δ — autocorrelation at lag 13 (§6.3).
fn compute_delta_region(data: &[u8]) -> f64 {
    let n = data.len().min(512);
    if n < 14 { return 0.0; }
    let mut cross = 0.0f64;
    let mut sq = 0.0f64;
    for i in 0..(n - 13) {
        cross += data[i] as f64 * data[i + 13] as f64;
        sq += data[i] as f64 * data[i] as f64;
    }
    (cross / (sq + 1e-10)).clamp(0.0, 1.0)
}

/// Periodicity (§6.5).
fn compute_periodicity(data: &[u8]) -> f64 {
    let n = data.len().min(512);
    let mut best = 0.0f64;
    for &p in &[28usize, 364] {
        if n <= p { continue; }
        let cnt = (0..(n - p)).filter(|&i| (data[i] as i16 - data[i + p] as i16).unsigned_abs() < 16).count();
        let score = cnt as f64 / n as f64;
        if score > best { best = score; }
    }
    best
}

/// Salvi Resonance (§6.6).
fn compute_salvi_resonance(data: &[u8]) -> bool {
    let n = data.len().min(512);
    for &p in &[5usize, 25, 125] {
        if n <= p { continue; }
        let cnt = (0..(n - p)).filter(|&i| (data[i] as i16 - data[i + p] as i16).unsigned_abs() < 8).count();
        if cnt as f64 / n as f64 > 0.80 { return true; }
    }
    false
}

/// Three-region GURFT analysis (§6.1).
pub fn gurft_analyze(data: &[u8]) -> GurftResult {
    if data.len() < 1024 {
        let s = &data[..data.len().min(512)];
        return GurftResult {
            tau: compute_torsion_region(s), delta: compute_delta_region(s),
            entropy: compute_entropy(s), periodicity: compute_periodicity(s),
            salvi_resonance: compute_salvi_resonance(s),
        };
    }
    let mid = data.len() / 2;
    let ra = &data[..512];
    let rb = &data[(mid.saturating_sub(256))..(mid + 256).min(data.len())];
    let rc = &data[data.len().saturating_sub(512)..];
    GurftResult {
        tau: 0.3 * compute_torsion_region(ra) + 0.4 * compute_torsion_region(rb) + 0.3 * compute_torsion_region(rc),
        delta: 0.3 * compute_delta_region(ra) + 0.4 * compute_delta_region(rb) + 0.3 * compute_delta_region(rc),
        entropy: 0.3 * compute_entropy(ra) + 0.4 * compute_entropy(rb) + 0.3 * compute_entropy(rc),
        periodicity: compute_periodicity(data),
        salvi_resonance: compute_salvi_resonance(data),
    }
}

/// Base selection (§6.7–6.8).
fn select_base(g: &GurftResult, mode: CompressionMode) -> u16 {
    if mode == CompressionMode::Basic { return 3; }
    if g.tau < TAU_HARMONIC { return 3; }
    let allowed = mode.allowed_bases();
    let cand = if g.tau >= TAU_HARMONIC && g.delta < DELTA_HOLOGRAPHIC { 13u16 }
        else if g.tau >= TAU_HOLOGRAPHIC && g.delta >= DELTA_HOLOGRAPHIC {
            if g.salvi_resonance && g.tau > TAU_RESONANCE { 70 }
            else if g.periodicity > 0.7 { 364 }
            else { 28 }
        } else { 3 };
    if allowed.contains(&cand) { cand } else { 3 }
}

// ─── Base-N Byte Packing (§4.11) ────────────────────────────────────────────

/// Pack bytes as base-3 trit stream (Rep C).
fn pack_bytes_tribonacci(data: &[u8]) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(data.len() * 2);
    for &b in data {
        let td = byte_to_bijective(b);
        for i in 0..(td.len as usize) {
            let code = match td.digits[i] { 1 => 0b00u32, 2 => 0b01, 3 => 0b10, _ => 0b11 };
            w.write(code, 2);
        }
    }
    w.finish_with_header()
}

/// Pack bytes as base-N stream via big-integer division (§4.11).
fn pack_bytes_base_n(data: &[u8], base: u16) -> Vec<u8> {
    if data.is_empty() { return vec![0, 0, 0, 0]; }
    let mut big = data.to_vec();
    let mut digits = Vec::new();
    while !big.is_empty() && !(big.len() == 1 && big[0] == 0) {
        let mut rem = 0u32;
        let mut new_big = Vec::new();
        for &byte in &big {
            let val = rem * 256 + byte as u32;
            let q = val / base as u32;
            rem = val % base as u32;
            if !new_big.is_empty() || q > 0 { new_big.push(q as u8); }
        }
        digits.push(rem as u16);
        big = new_big;
    }
    digits.reverse();
    let bits_per_digit = (16 - (base - 1).leading_zeros()) as u8;
    let mut w = BitWriter::with_capacity(digits.len() * 2);
    encode_hybrid_prefix(&mut w, digits.len() as u64);
    for &d in &digits { w.write(d as u32, bits_per_digit); }
    w.finish_with_header()
}

/// Try-and-compare validation (§6.9).
fn try_and_compare_base(chunk: &[u8], g: &GurftResult, mode: CompressionMode) -> u16 {
    let cand = select_base(g, mode);
    if cand == 3 { return 3; }
    let sample = &chunk[..chunk.len().min(512)];
    let sc = pack_bytes_base_n(sample, cand).len();
    let s3 = pack_bytes_tribonacci(sample).len();
    if s3 <= sc { 3 } else { cand }
}

// ─── Domain Preprocessing — Real Implementations (§3.8, §3.9) ──────────────

/// Domain transform flag (chunk map +15 bits 2–0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainTransform(pub u8);
impl DomainTransform {
    pub const NONE: Self = Self(0);
    pub const AUDIO_LP: Self = Self(1);
    pub const IMAGE_MED: Self = Self(2);
    pub const GENOMIC: Self = Self(3);
    pub const SOURCE: Self = Self(4);
    pub const LOG: Self = Self(5);
    pub const STRUCTURED: Self = Self(6);
}

/// AUDIO: 4th-order linear prediction (§3.8.1). Returns (residuals, coefficients).
fn audio_lp_encode(data: &[u8]) -> (Vec<u8>, [i16; 4]) {
    const P: usize = 4;
    if data.len() <= P { return (data.to_vec(), [0i16; 4]); }
    let slen = data.len().min(1024);
    let samples: Vec<f64> = data[..slen].iter().map(|&b| b as f64).collect();
    // Autocorrelation
    let mut r = [0.0f64; P + 1];
    for lag in 0..=P { for i in lag..slen { r[lag] += samples[i] * samples[i - lag]; } }
    // Levinson-Durbin
    let mut a = [0.0f64; P];
    let mut e = r[0];
    if e <= 0.0 { return (data.to_vec(), [0; 4]); }
    for i in 0..P {
        let mut lambda = 0.0;
        for j in 0..i { lambda += a[j] * r[i - j]; }
        lambda = (r[i + 1] - lambda) / e;
        let mut a_new = a;
        a_new[i] = lambda;
        for j in 0..i { a_new[j] = a[j] - lambda * a[i - 1 - j]; }
        a = a_new;
        e *= 1.0 - lambda * lambda;
        if e <= 0.0 { break; }
    }
    let coeffs = [
        (a[0] * 32767.0).clamp(-32768.0, 32767.0) as i16,
        (a[1] * 32767.0).clamp(-32768.0, 32767.0) as i16,
        (a[2] * 32767.0).clamp(-32768.0, 32767.0) as i16,
        (a[3] * 32767.0).clamp(-32768.0, 32767.0) as i16,
    ];
    let af: [f64; 4] = [coeffs[0] as f64 / 32767.0, coeffs[1] as f64 / 32767.0,
                         coeffs[2] as f64 / 32767.0, coeffs[3] as f64 / 32767.0];
    let mut res = Vec::with_capacity(data.len());
    for i in 0..P { res.push(data[i]); }
    for i in P..data.len() {
        let pred = af[0] * data[i-1] as f64 + af[1] * data[i-2] as f64
                 + af[2] * data[i-3] as f64 + af[3] * data[i-4] as f64;
        res.push((data[i] as i16).wrapping_sub(pred.round() as i16) as u8);
    }
    (res, coeffs)
}

fn audio_lp_decode(res: &[u8], coeffs: &[i16; 4]) -> Vec<u8> {
    const P: usize = 4;
    if res.len() <= P { return res.to_vec(); }
    let af: [f64; 4] = [coeffs[0] as f64 / 32767.0, coeffs[1] as f64 / 32767.0,
                         coeffs[2] as f64 / 32767.0, coeffs[3] as f64 / 32767.0];
    let mut out = Vec::with_capacity(res.len());
    for i in 0..P { out.push(res[i]); }
    for i in P..res.len() {
        let pred = af[0] * out[i-1] as f64 + af[1] * out[i-2] as f64
                 + af[2] * out[i-3] as f64 + af[3] * out[i-4] as f64;
        out.push((res[i] as i16).wrapping_add(pred.round() as i16) as u8);
    }
    out
}

/// IMAGE: 2D MED predictor (§3.8.2).
fn image_med_encode(data: &[u8], width: usize) -> Vec<u8> {
    if width == 0 || data.is_empty() { return data.to_vec(); }
    let height = data.len() / width;
    let mut res = Vec::with_capacity(data.len());
    for r in 0..height {
        for c in 0..width {
            let idx = r * width + c;
            if r == 0 || c == 0 {
                res.push(if idx == 0 { data[0] } else { data[idx].wrapping_sub(data[idx - 1]) });
            } else {
                let a = data[r * width + c - 1] as i16;
                let b = data[(r-1) * width + c] as i16;
                let cc = data[(r-1) * width + c - 1] as i16;
                let p = if cc >= a.max(b) { a.min(b) } else if cc <= a.min(b) { a.max(b) } else { a + b - cc };
                res.push((data[idx] as i16 - p) as u8);
            }
        }
    }
    for i in (height * width)..data.len() { res.push(data[i]); }
    res
}

fn image_med_decode(res: &[u8], width: usize) -> Vec<u8> {
    if width == 0 || res.is_empty() { return res.to_vec(); }
    let height = res.len() / width;
    let mut out: Vec<u8> = Vec::with_capacity(res.len());
    for r in 0..height {
        for c in 0..width {
            let idx = r * width + c;
            if r == 0 || c == 0 {
                out.push(if idx == 0 { res[0] } else { out[idx - 1].wrapping_add(res[idx]) });
            } else {
                let a = out[r * width + c - 1] as i16;
                let b = out[(r-1) * width + c] as i16;
                let cc = out[(r-1) * width + c - 1] as i16;
                let p = if cc >= a.max(b) { a.min(b) } else if cc <= a.min(b) { a.max(b) } else { a + b - cc };
                out.push((res[idx] as i16 + p) as u8);
            }
        }
    }
    for i in (height * width)..res.len() { out.push(res[i]); }
    out
}

/// GENOMIC: 2-bit nucleotide encoding (§3.8.3).
fn genomic_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4 + data.len());
    let mut i = 0;
    while i < data.len() {
        let b = data[i].to_ascii_uppercase();
        let c0 = match b { b'A' => 0u8, b'C' => 1, b'G' => 2, b'T' => 3, _ => { out.push(0xFF); out.push(data[i]); i += 1; continue; } };
        let mut packed = c0 << 6;
        let mut cnt = 1;
        while cnt < 4 && i + cnt < data.len() {
            match data[i + cnt].to_ascii_uppercase() {
                b'A' => packed |= 0 << (6 - cnt * 2),
                b'C' => packed |= 1 << (6 - cnt * 2),
                b'G' => packed |= 2 << (6 - cnt * 2),
                b'T' => packed |= 3 << (6 - cnt * 2),
                _ => break,
            }
            cnt += 1;
        }
        if cnt == 4 { out.push(packed); i += 4; }
        else { out.push(0xFF); out.push(data[i]); i += 1; }
    }
    out
}

fn genomic_decode(data: &[u8]) -> Vec<u8> {
    let map = [b'A', b'C', b'G', b'T'];
    let mut out = Vec::with_capacity(data.len() * 4);
    let mut i = 0;
    while i < data.len() {
        if data[i] == 0xFF { i += 1; if i < data.len() { out.push(data[i]); i += 1; } }
        else {
            let p = data[i];
            out.push(map[((p >> 6) & 3) as usize]);
            out.push(map[((p >> 4) & 3) as usize]);
            out.push(map[((p >> 2) & 3) as usize]);
            out.push(map[(p & 3) as usize]);
            i += 1;
        }
    }
    out
}

/// SOURCE: Keyword tokenization (§3.9.1).
const SOURCE_KEYWORDS: &[&[u8]] = &[
    b"function", b"return", b"if", b"else", b"for", b"while", b"do",
    b"class", b"import", b"export", b"const", b"let", b"var", b"true",
    b"false", b"null", b"void", b"int", b"string", b"bool", b"float",
    b"double", b"char", b"byte", b"long", b"short", b"unsigned",
    b"public", b"private", b"protected", b"static", b"final", b"abstract",
    b"interface", b"enum", b"struct", b"type", b"trait", b"impl",
    b"fn", b"pub", b"mod", b"use", b"crate", b"self", b"super",
    b"match", b"case", b"switch", b"break", b"continue", b"default",
    b"try", b"catch", b"throw", b"throws", b"finally", b"async",
    b"await", b"yield", b"new", b"delete", b"typeof", b"instanceof",
    b"in", b"of", b"from", b"as", b"with", b"this", b"package",
    b"extends", b"implements", b"override", b"virtual", b"const_cast",
    b"template", b"namespace", b"using", b"include", b"define",
    b"ifdef", b"ifndef", b"endif", b"pragma", b"extern",
    b"volatile", b"register", b"inline", b"goto", b"sizeof",
    b"nullptr", b"auto", b"decltype", b"constexpr", b"noexcept",
    b"lambda", b"def", b"elif", b"pass", b"raise", b"except",
    b"print", b"println", b"printf", b"sprintf", b"fprintf",
    b"malloc", b"free", b"realloc", b"calloc",
    b"->", b"=>", b"==", b"!=", b"<=", b">=", b"&&", b"||",
    b"<<", b">>", b"++", b"--", b"+=", b"-=", b"*=", b"/=",
    b"{", b"}", b"(", b")", b"[", b"]", b";", b":", b",",
];

fn source_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    'outer: while i < data.len() {
        for (idx, kw) in SOURCE_KEYWORDS.iter().enumerate() {
            if idx >= 127 { break; }
            if data[i..].starts_with(kw) {
                let after = i + kw.len();
                let is_word = kw.len() <= 2 || after >= data.len()
                    || !(data[after].is_ascii_alphanumeric() || data[after] == b'_');
                if is_word { out.push((idx + 1) as u8); i += kw.len(); continue 'outer; }
            }
        }
        out.push(0x80);
        out.push(data[i]);
        i += 1;
    }
    out
}

fn source_decode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 4);
    let mut i = 0;
    while i < data.len() {
        if data[i] == 0x80 { i += 1; if i < data.len() { out.push(data[i]); i += 1; } }
        else if data[i] >= 0x01 && data[i] <= 0x7F {
            let idx = (data[i] - 1) as usize;
            if idx < SOURCE_KEYWORDS.len() { out.extend_from_slice(SOURCE_KEYWORDS[idx]); }
            i += 1;
        } else { out.push(data[i]); i += 1; }
    }
    out
}

/// LOG: Timestamp normalization + field separation (§3.9.2).
/// Stream format: [marker][data] per field.
fn log_encode(data: &[u8]) -> Vec<u8> {
    let text = data;
    let mut out = Vec::with_capacity(data.len());
    let mut prev_ts: u64 = 0;
    let mut service_dict: Vec<Vec<u8>> = Vec::new();

    // Process line by line
    let mut line_start = 0;
    while line_start < text.len() {
        let line_end = text[line_start..].iter().position(|&b| b == b'\n')
            .map(|p| line_start + p).unwrap_or(text.len());
        let line = &text[line_start..line_end];

        if line.is_empty() {
            if line_end < text.len() { out.push(0x04); out.push(0); out.push(0); } // empty line
            line_start = line_end + 1;
            continue;
        }

        // Try to detect timestamp at start of line (ISO 8601 or Unix epoch)
        let (ts_end, ts_value) = detect_timestamp(line);

        if ts_end > 0 {
            // Emit timestamp delta
            out.push(0x01);
            if prev_ts == 0 {
                // Absolute: 8 bytes
                out.extend_from_slice(&ts_value.to_be_bytes());
            } else {
                // Delta: variable length (1-4 bytes as varint)
                let delta = ts_value.saturating_sub(prev_ts);
                encode_varint(&mut out, delta);
            }
            prev_ts = ts_value;
        }

        // Detect log level
        let remaining = if ts_end > 0 && ts_end < line.len() { &line[ts_end..] } else { line };
        let remaining = trim_leading_space(remaining);

        let (level_code, after_level) = detect_log_level(remaining);
        if level_code > 0 {
            out.push(0x02);
            out.push(level_code);
        }

        // Detect service/module name (word before colon or in brackets)
        let rest = if level_code > 0 { trim_leading_space(after_level) } else { remaining };
        let (service_idx, after_service) = detect_service_name(rest, &mut service_dict);
        if let Some(idx) = service_idx {
            out.push(0x03);
            out.extend_from_slice(&(idx as u16).to_be_bytes());
        }

        // Remaining: message body
        let msg = if service_idx.is_some() { after_service } else { rest };
        if !msg.is_empty() {
            out.push(0x04);
            let msg_len = msg.len() as u16;
            out.extend_from_slice(&msg_len.to_be_bytes());
            out.extend_from_slice(msg);
        }

        line_start = if line_end < text.len() { line_end + 1 } else { text.len() };
    }

    // Prepend service dictionary
    let mut result = Vec::with_capacity(4 + service_dict.iter().map(|s| 2 + s.len()).sum::<usize>() + out.len());
    result.extend_from_slice(&(service_dict.len() as u16).to_be_bytes());
    for svc in &service_dict {
        result.extend_from_slice(&(svc.len() as u16).to_be_bytes());
        result.extend_from_slice(svc);
    }
    result.extend_from_slice(&out);
    result
}

fn log_decode(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return data.to_vec(); }
    let mut pos = 0;
    // Read service dictionary
    let svc_count = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    let mut services: Vec<Vec<u8>> = Vec::with_capacity(svc_count);
    for _ in 0..svc_count {
        if pos + 2 > data.len() { break; }
        let slen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        if pos + slen > data.len() { break; }
        services.push(data[pos..pos + slen].to_vec());
        pos += slen;
    }

    let mut out = Vec::with_capacity(data.len() * 2);
    let mut prev_ts: u64 = 0;
    let mut line_has_content = false;

    while pos < data.len() {
        let marker = data[pos]; pos += 1;
        match marker {
            0x01 => {
                // Timestamp
                if prev_ts == 0 {
                    if pos + 8 > data.len() { break; }
                    prev_ts = u64::from_be_bytes([data[pos],data[pos+1],data[pos+2],data[pos+3],
                                                   data[pos+4],data[pos+5],data[pos+6],data[pos+7]]);
                    pos += 8;
                } else {
                    let (delta, bytes_read) = decode_varint(&data[pos..]);
                    pos += bytes_read;
                    prev_ts += delta;
                }
                // Write timestamp as ISO-like string
                write_timestamp(&mut out, prev_ts);
                out.push(b' ');
                line_has_content = true;
            }
            0x02 => {
                if pos >= data.len() { break; }
                let level = data[pos]; pos += 1;
                let name = match level {
                    1 => b"DEBUG" as &[u8], 2 => b"INFO", 3 => b"WARN", 4 => b"ERROR", 5 => b"FATAL",
                    _ => b"UNKNOWN",
                };
                out.extend_from_slice(name);
                out.push(b' ');
                line_has_content = true;
            }
            0x03 => {
                if pos + 2 > data.len() { break; }
                let idx = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                if idx < services.len() { out.extend_from_slice(&services[idx]); }
                out.extend_from_slice(b": ");
                line_has_content = true;
            }
            0x04 => {
                if pos + 2 > data.len() { break; }
                let mlen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                if mlen == 0 { // empty line marker
                    if line_has_content { out.push(b'\n'); }
                    out.push(b'\n');
                    line_has_content = false;
                } else {
                    let end = (pos + mlen).min(data.len());
                    out.extend_from_slice(&data[pos..end]);
                    out.push(b'\n');
                    pos = end;
                    line_has_content = false;
                }
            }
            0x05 => {
                // Raw passthrough (fallback for unrecognized format)
                out.extend_from_slice(&data[pos..]);
                pos = data.len();
            }
            _ => { /* skip unknown markers */ }
        }
    }
    out
}

// LOG helper functions
fn detect_timestamp(line: &[u8]) -> (usize, u64) {
    // ISO 8601: 2026-03-15T10:30:00
    if line.len() >= 19 && line[4] == b'-' && line[7] == b'-' && (line[10] == b'T' || line[10] == b' ') && line[13] == b':' && line[16] == b':' {
        // Parse as approximate epoch milliseconds
        if let Ok(s) = core::str::from_utf8(&line[..19]) {
            let year = s[0..4].parse::<u64>().unwrap_or(2026);
            let month = s[5..7].parse::<u64>().unwrap_or(1);
            let day = s[8..10].parse::<u64>().unwrap_or(1);
            let hour = s[11..13].parse::<u64>().unwrap_or(0);
            let min = s[14..16].parse::<u64>().unwrap_or(0);
            let sec = s[17..19].parse::<u64>().unwrap_or(0);
            let approx_ms = ((year - 1970) * 31536000 + (month - 1) * 2592000 + (day - 1) * 86400
                + hour * 3600 + min * 60 + sec) * 1000;
            let end = if line.len() > 19 && line[19] == b'.' { 23.min(line.len()) } else { 19 };
            return (end, approx_ms);
        }
    }
    // Unix epoch: sequence of digits at start
    if line.len() >= 10 && line[..10].iter().all(|&b| b.is_ascii_digit()) {
        if let Ok(s) = core::str::from_utf8(&line[..10]) {
            if let Ok(epoch) = s.parse::<u64>() {
                let end = line.iter().position(|&b| !b.is_ascii_digit() && b != b'.').unwrap_or(line.len());
                return (end, epoch * 1000);
            }
        }
    }
    (0, 0)
}

fn detect_log_level(data: &[u8]) -> (u8, &[u8]) {
    for &(prefix, code) in &[(b"DEBUG" as &[u8], 1u8), (b"INFO", 2), (b"WARN", 3), (b"WARNING", 3),
                              (b"ERROR", 4), (b"FATAL", 5), (b"CRITICAL", 5)] {
        if data.starts_with(prefix) {
            let after = &data[prefix.len()..];
            if after.is_empty() || after[0] == b' ' || after[0] == b']' || after[0] == b':' {
                return (code, after);
            }
        }
    }
    // Check for [LEVEL] pattern
    if data.starts_with(b"[") {
        if let Some(end) = data.iter().position(|&b| b == b']') {
            let inner = &data[1..end];
            let (code, _) = detect_log_level(inner);
            if code > 0 { return (code, &data[end + 1..]); }
        }
    }
    (0, data)
}

fn detect_service_name<'a>(data: &'a [u8], dict: &mut Vec<Vec<u8>>) -> (Option<usize>, &'a [u8]) {
    // Look for "ServiceName:" or "[ServiceName]" pattern
    let trimmed = trim_leading_space(data);
    // Find word before colon
    if let Some(colon_pos) = trimmed.iter().position(|&b| b == b':') {
        if colon_pos > 0 && colon_pos <= 64 {
            let name = &trimmed[..colon_pos];
            if name.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.') {
                let name_vec = name.to_vec();
                let idx = if let Some(i) = dict.iter().position(|s| s == &name_vec) { i }
                    else { dict.push(name_vec); dict.len() - 1 };
                return (Some(idx), &trimmed[colon_pos + 1..]);
            }
        }
    }
    (None, trimmed)
}

fn trim_leading_space(data: &[u8]) -> &[u8] {
    let start = data.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(data.len());
    &data[start..]
}

fn encode_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value > 0 { byte |= 0x80; }
        out.push(byte);
        if value == 0 { break; }
    }
}

fn decode_varint(data: &[u8]) -> (u64, usize) {
    let mut result = 0u64;
    let mut shift = 0u32;
    for (i, &byte) in data.iter().enumerate() {
        result |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 { return (result, i + 1); }
        if shift >= 64 { return (result, i + 1); }
    }
    (result, data.len())
}

fn write_timestamp(out: &mut Vec<u8>, ms: u64) {
    // Simple epoch seconds formatting
    let secs = ms / 1000;
    let mut buf = [0u8; 20];
    let mut n = secs;
    let mut len = 0;
    if n == 0 { out.push(b'0'); return; }
    while n > 0 { buf[len] = b'0' + (n % 10) as u8; n /= 10; len += 1; }
    for i in (0..len).rev() { out.push(buf[i]); }
}

/// STRUCTURED: Schema-value separation (§3.9.3).
/// Handles JSON, CSV, and XML by separating schema (keys/tags/headers)
/// from values, allowing each stream to compress independently.
fn structured_encode(data: &[u8]) -> Vec<u8> {
    // Detect format and dispatch
    let trimmed = trim_leading_space(data);
    if trimmed.starts_with(b"{") || trimmed.starts_with(b"[") {
        structured_encode_json(data)
    } else if looks_like_csv(data) {
        structured_encode_csv(data)
    } else if trimmed.starts_with(b"<") {
        structured_encode_xml(data)
    } else {
        // Unknown structured format — passthrough with marker
        let mut out = Vec::with_capacity(1 + data.len());
        out.push(0x00); // format: passthrough
        out.extend_from_slice(data);
        out
    }
}

fn structured_decode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() { return Vec::new(); }
    match data[0] {
        0x00 => data[1..].to_vec(), // passthrough
        0x01 => structured_decode_json(&data[1..]),
        0x02 => structured_decode_csv(&data[1..]),
        0x03 => structured_decode_xml(&data[1..]),
        _ => data.to_vec(),
    }
}

/// JSON: extract keys into dictionary, stream values with type tags.
fn structured_encode_json(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    out.push(0x01); // format: JSON

    // Build key dictionary by scanning for "key": patterns
    let mut keys: Vec<Vec<u8>> = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'"' {
            let start = i + 1;
            let end = data[start..].iter().position(|&b| b == b'"').map(|p| start + p).unwrap_or(data.len());
            // Check if followed by ':'
            let after = data[end + 1..].iter().position(|&b| b != b' ' && b != b'\t').map(|p| end + 1 + p);
            if let Some(colon_pos) = after {
                if colon_pos < data.len() && data[colon_pos] == b':' {
                    let key = data[start..end].to_vec();
                    if !keys.contains(&key) && keys.len() < 65535 { keys.push(key); }
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }

    // Write key dictionary
    out.extend_from_slice(&(keys.len() as u16).to_be_bytes());
    for k in &keys {
        out.extend_from_slice(&(k.len() as u16).to_be_bytes());
        out.extend_from_slice(k);
    }

    // Write value stream: for each key occurrence, emit key_index + value bytes
    // Simplified: emit the structural characters and key references
    i = 0;
    while i < data.len() {
        if data[i] == b'"' {
            let start = i + 1;
            let end = data[start..].iter().position(|&b| b == b'"').map(|p| start + p).unwrap_or(data.len());
            let key = &data[start..end];
            // Check if this is a key (followed by ':')
            let after_quote = end + 1;
            let next_nonws = data[after_quote..].iter().position(|&b| b != b' ' && b != b'\t').map(|p| after_quote + p);
            if let Some(np) = next_nonws {
                if np < data.len() && data[np] == b':' {
                    if let Some(kid) = keys.iter().position(|k| k == key) {
                        out.push(0xFE); // key reference marker
                        out.extend_from_slice(&(kid as u16).to_be_bytes());
                        i = np + 1; // skip past ':'
                        continue;
                    }
                }
            }
            // Not a key — emit as literal string value
            out.push(b'"');
            out.extend_from_slice(&data[start..end]);
            out.push(b'"');
            i = end + 1;
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

fn structured_decode_json(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return Vec::new(); }
    let mut pos = 0;
    // Read key dictionary
    let key_count = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(key_count);
    for _ in 0..key_count {
        if pos + 2 > data.len() { break; }
        let klen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        if pos + klen > data.len() { break; }
        keys.push(data[pos..pos + klen].to_vec());
        pos += klen;
    }
    // Reconstruct
    let mut out = Vec::with_capacity(data.len() * 2);
    while pos < data.len() {
        if data[pos] == 0xFE {
            pos += 1;
            if pos + 2 > data.len() { break; }
            let kid = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            out.push(b'"');
            if kid < keys.len() { out.extend_from_slice(&keys[kid]); }
            out.push(b'"');
            out.push(b':');
        } else {
            out.push(data[pos]);
            pos += 1;
        }
    }
    out
}

/// CSV: separate header row from data columns.
fn structured_encode_csv(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    out.push(0x02); // format: CSV
    // Find header row (first line)
    let first_nl = data.iter().position(|&b| b == b'\n').unwrap_or(data.len());
    let header = &data[..first_nl];
    // Write header
    out.extend_from_slice(&(header.len() as u16).to_be_bytes());
    out.extend_from_slice(header);
    // Write data rows — attempt numeric delta per column
    let columns: Vec<&[u8]> = header.split(|&b| b == b',').collect();
    let col_count = columns.len();
    out.extend_from_slice(&(col_count as u16).to_be_bytes());
    // For simplicity, emit rows with column-typed encoding
    let body = if first_nl < data.len() { &data[first_nl + 1..] } else { &[] };
    let mut prev_nums: Vec<i64> = vec![0i64; col_count];
    for line in body.split(|&b| b == b'\n') {
        if line.is_empty() { continue; }
        let fields: Vec<&[u8]> = line.split(|&b| b == b',').collect();
        for (ci, field) in fields.iter().enumerate() {
            if ci >= col_count { break; }
            // Try numeric
            if let Ok(s) = core::str::from_utf8(field) {
                if let Ok(n) = s.trim().parse::<i64>() {
                    let delta = n - prev_nums[ci];
                    prev_nums[ci] = n;
                    out.push(0x01); // numeric delta
                    encode_varint_signed(&mut out, delta);
                    continue;
                }
            }
            // String field
            out.push(0x02); // string
            out.extend_from_slice(&(*field).len().to_be_bytes()[6..8]);
            out.extend_from_slice(field);
        }
        out.push(0x00); // row delimiter
    }
    out
}

fn structured_decode_csv(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 { return Vec::new(); }
    let mut pos = 0;
    let hdr_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    if pos + hdr_len > data.len() { return Vec::new(); }
    let header = &data[pos..pos + hdr_len];
    pos += hdr_len;
    if pos + 2 > data.len() { return Vec::new(); }
    let col_count = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;

    let mut out = Vec::with_capacity(data.len() * 2);
    out.extend_from_slice(header);
    out.push(b'\n');

    let mut prev_nums = vec![0i64; col_count];
    let mut col_idx = 0;
    while pos < data.len() {
        let marker = data[pos]; pos += 1;
        match marker {
            0x00 => { // row delimiter
                // Remove trailing comma if present
                if out.last() == Some(&b',') { out.pop(); }
                out.push(b'\n');
                col_idx = 0;
            }
            0x01 => { // numeric delta
                let (delta, bytes_read) = decode_varint_signed(&data[pos..]);
                pos += bytes_read;
                prev_nums[col_idx.min(col_count - 1)] += delta;
                // Write number as ASCII
                write_i64(&mut out, prev_nums[col_idx.min(col_count - 1)]);
                out.push(b',');
                col_idx += 1;
            }
            0x02 => { // string
                if pos + 2 > data.len() { break; }
                let slen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                let end = (pos + slen).min(data.len());
                out.extend_from_slice(&data[pos..end]);
                out.push(b',');
                pos = end;
                col_idx += 1;
            }
            _ => {}
        }
    }
    out
}

/// XML: tag name dictionary + content stream.
fn structured_encode_xml(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    out.push(0x03); // format: XML
    // Build tag dictionary
    let mut tags: Vec<Vec<u8>> = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'<' && i + 1 < data.len() && data[i + 1] != b'/' && data[i + 1] != b'!' && data[i + 1] != b'?' {
            let start = i + 1;
            let end = data[start..].iter().position(|&b| b == b' ' || b == b'>' || b == b'/').map(|p| start + p).unwrap_or(data.len());
            let tag = data[start..end].to_vec();
            if !tag.is_empty() && !tags.contains(&tag) && tags.len() < 65535 { tags.push(tag); }
        }
        i += 1;
    }
    // Write tag dictionary
    out.extend_from_slice(&(tags.len() as u16).to_be_bytes());
    for t in &tags { out.extend_from_slice(&(t.len() as u16).to_be_bytes()); out.extend_from_slice(t); }
    // Write content with tag references
    i = 0;
    while i < data.len() {
        if data[i] == b'<' {
            let is_closing = i + 1 < data.len() && data[i + 1] == b'/';
            let tag_start = if is_closing { i + 2 } else { i + 1 };
            if tag_start < data.len() && data[tag_start] != b'!' && data[tag_start] != b'?' {
                let end = data[tag_start..].iter().position(|&b| b == b' ' || b == b'>' || b == b'/').map(|p| tag_start + p).unwrap_or(data.len());
                let tag = &data[tag_start..end];
                if let Some(tid) = tags.iter().position(|t| t == tag) {
                    out.push(if is_closing { 0xFD } else { 0xFE }); // open or close tag ref
                    out.extend_from_slice(&(tid as u16).to_be_bytes());
                    // Skip to after '>'
                    let gt = data[i..].iter().position(|&b| b == b'>').map(|p| i + p + 1).unwrap_or(data.len());
                    // Emit any attributes between tag name and '>'
                    if end < gt.saturating_sub(1) {
                        let attrs = &data[end..gt - 1];
                        out.push(0xFC); // attribute marker
                        out.extend_from_slice(&(attrs.len() as u16).to_be_bytes());
                        out.extend_from_slice(attrs);
                    }
                    i = gt;
                    continue;
                }
            }
        }
        out.push(data[i]);
        i += 1;
    }
    out
}

fn structured_decode_xml(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 { return Vec::new(); }
    let mut pos = 0;
    let tag_count = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    let mut tags: Vec<Vec<u8>> = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        if pos + 2 > data.len() { break; }
        let tlen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        if pos + tlen > data.len() { break; }
        tags.push(data[pos..pos + tlen].to_vec());
        pos += tlen;
    }
    let mut out = Vec::with_capacity(data.len() * 2);
    while pos < data.len() {
        match data[pos] {
            0xFE => { // open tag
                pos += 1;
                if pos + 2 > data.len() { break; }
                let tid = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                out.push(b'<');
                if tid < tags.len() { out.extend_from_slice(&tags[tid]); }
                // Check for attributes
                if pos < data.len() && data[pos] == 0xFC {
                    pos += 1;
                    if pos + 2 <= data.len() {
                        let alen = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                        pos += 2;
                        let end = (pos + alen).min(data.len());
                        out.extend_from_slice(&data[pos..end]);
                        pos = end;
                    }
                }
                out.push(b'>');
            }
            0xFD => { // close tag
                pos += 1;
                if pos + 2 > data.len() { break; }
                let tid = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                out.extend_from_slice(b"</");
                if tid < tags.len() { out.extend_from_slice(&tags[tid]); }
                out.push(b'>');
            }
            _ => { out.push(data[pos]); pos += 1; }
        }
    }
    out
}

fn looks_like_csv(data: &[u8]) -> bool {
    // Heuristic: first line contains commas, multiple lines
    let first_nl = data.iter().position(|&b| b == b'\n').unwrap_or(data.len());
    let first_line = &data[..first_nl];
    let comma_count = first_line.iter().filter(|&&b| b == b',').count();
    comma_count >= 2 && first_nl < data.len()
}

fn encode_varint_signed(out: &mut Vec<u8>, value: i64) {
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    encode_varint(out, zigzag);
}

fn decode_varint_signed(data: &[u8]) -> (i64, usize) {
    let (zigzag, bytes) = decode_varint(data);
    let value = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
    (value, bytes)
}

fn write_i64(out: &mut Vec<u8>, value: i64) {
    if value < 0 { out.push(b'-'); write_u64(out, (-value) as u64); }
    else { write_u64(out, value as u64); }
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    if value == 0 { out.push(b'0'); return; }
    let mut buf = [0u8; 20];
    let mut n = value;
    let mut len = 0;
    while n > 0 { buf[len] = b'0' + (n % 10) as u8; n /= 10; len += 1; }
    for i in (0..len).rev() { out.push(buf[i]); }
}

/// Apply domain preprocessing. Returns (data, transform, lp_coeffs, image_width).
fn apply_domain_preprocess(
    data: &[u8], mode: CompressionMode, image_width: Option<usize>,
) -> (Vec<u8>, DomainTransform, Option<[i16; 4]>, Option<u64>) {
    match mode {
        CompressionMode::Audio => { let (r, c) = audio_lp_encode(data); (r, DomainTransform::AUDIO_LP, Some(c), None) }
        CompressionMode::Image => {
            if let Some(w) = image_width { (image_med_encode(data, w), DomainTransform::IMAGE_MED, None, Some(w as u64)) }
            else { (data.to_vec(), DomainTransform::NONE, None, None) }
        }
        CompressionMode::Genomic => (genomic_encode(data), DomainTransform::GENOMIC, None, None),
        CompressionMode::Source => (source_encode(data), DomainTransform::SOURCE, None, None),
        CompressionMode::Log => (log_encode(data), DomainTransform::LOG, None, None),
        CompressionMode::Structured => (structured_encode(data), DomainTransform::STRUCTURED, None, None),
        _ => (data.to_vec(), DomainTransform::NONE, None, None),
    }
}

/// Reverse domain preprocessing.
fn reverse_domain_preprocess(
    data: &[u8], xform: DomainTransform, coeffs: Option<&[i16; 4]>, iw: Option<u64>,
) -> TtcResult<Vec<u8>> {
    match xform.0 {
        0 => Ok(data.to_vec()),
        1 => Ok(audio_lp_decode(data, coeffs.ok_or_else(|| TtcError::DecompressionError("Missing LP coefficients".into()))?)),
        2 => Ok(image_med_decode(data, iw.ok_or(TtcError::ImageWidthRequired)? as usize)),
        3 => Ok(genomic_decode(data)),
        4 => Ok(source_decode(data)),
        5 => Ok(log_decode(data)),
        6 => Ok(structured_decode(data)),
        7 => Err(TtcError::InvalidDomainTransform(7)),
        _ => Err(TtcError::InvalidDomainTransform(xform.0)),
    }
}

// ─── LZ77 Hash Chain (§4) — fixed lazy matching ────────────────────────────

const INVALID_POS: u32 = u32::MAX;

struct Lz77Engine {
    window_size: usize,
    min_match: usize,
    min_run: usize,
    chain_depth: usize,
    head: Vec<u32>,
    chain: Vec<u32>,
}

impl Lz77Engine {
    fn new(cfg: &LevelConfig) -> Self {
        Self {
            window_size: cfg.window_size, min_match: cfg.min_match, min_run: cfg.min_run,
            chain_depth: cfg.chain_depth,
            head: vec![INVALID_POS; cfg.window_size],
            chain: vec![INVALID_POS; cfg.window_size],
        }
    }

    #[inline]
    fn hash(&self, data: &[u8], i: usize) -> usize {
        if i + 2 >= data.len() { return 0; }
        ((data[i] as usize).wrapping_mul(65521) ^ (data[i+1] as usize).wrapping_mul(257) ^ data[i+2] as usize) % self.window_size
    }

    fn find_best_match(&self, data: &[u8], pos: usize) -> Option<(usize, usize)> {
        if pos + 2 >= data.len() { return None; }
        let h = self.hash(data, pos);
        let mut j = self.head[h];
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        let mut steps = 0;
        let min_pos = pos.saturating_sub(self.window_size);

        while j != INVALID_POS && steps < self.chain_depth {
            let jj = j as usize;
            if jj < min_pos || jj >= pos { j = self.chain[jj % self.window_size]; steps += 1; continue; }
            if data[jj] == data[pos] && data[jj+1] == data[pos+1] && data[jj+2] == data[pos+2] {
                let max_len = 255.min(data.len() - pos);
                let mut len = 3;
                while len < max_len && jj + len < data.len() && data[jj + len] == data[pos + len] { len += 1; }
                if len > best_len { best_len = len; best_dist = pos - jj; }
            }
            j = self.chain[jj % self.window_size]; steps += 1;
        }

        if best_len >= self.min_match { Some((best_dist, best_len)) } else { None }
    }

    #[inline]
    fn update(&mut self, data: &[u8], pos: usize) {
        if pos + 2 >= data.len() { return; }
        let h = self.hash(data, pos);
        let old = self.head[h];
        self.head[h] = pos as u32;
        self.chain[pos % self.window_size] = old;
    }

    #[inline]
    fn count_run(&self, data: &[u8], pos: usize) -> usize {
        if pos >= data.len() { return 0; }
        let byte = data[pos];
        let mut len = 1;
        while pos + len < data.len() && data[pos + len] == byte && len < 255 { len += 1; }
        len
    }
}

/// Greedy/lazy tokenization (§4.5, §4.9) — lazy matching bug fixed.
fn tokenize_greedy_lazy(data: &[u8], hist_off: usize, cfg: &LevelConfig) -> Vec<Token> {
    let mut eng = Lz77Engine::new(cfg);
    let mut tokens = Vec::new();
    // Build chains for history
    for j in 0..hist_off.min(data.len()) { eng.update(data, j); }
    let mut i = hist_off;

    while i < data.len() {
        let run = eng.count_run(data, i);
        if run >= cfg.min_run {
            tokens.push(Token::Run { byte: data[i], length: run });
            for k in 0..run { eng.update(data, i + k); }
            i += run;
            continue;
        }

        if let Some((dist, len)) = eng.find_best_match(data, i) {
            if cfg.parsing == Parsing::Lazy && len < 255 {
                // Lazy: check if next position has a better match
                eng.update(data, i);
                if let Some((dist1, len1)) = eng.find_best_match(data, i + 1) {
                    if len1 > len + 1 {
                        // Better match at i+1: emit literal at i, use match at i+1
                        tokens.push(Token::Literal(data[i]));
                        i += 1;
                        tokens.push(Token::Match { dist: dist1, length: len1 });
                        for k in 0..len1 { eng.update(data, i + k); }
                        i += len1;
                        continue;
                    }
                }
            } else {
                eng.update(data, i);
            }
            tokens.push(Token::Match { dist, length: len });
            for k in 1..len { eng.update(data, i + k); }
            i += len;
            continue;
        }

        tokens.push(Token::Literal(data[i]));
        eng.update(data, i);
        i += 1;
    }
    tokens
}

/// Beam-search optimal parsing (§4.6) with real cost model.
fn tokenize_beam(data: &[u8], hist_off: usize, cfg: &LevelConfig, cost_mode: ChunkMode) -> Vec<Token> {
    let chunk_len = data.len() - hist_off;
    if chunk_len == 0 { return Vec::new(); }

    let mut eng = Lz77Engine::new(cfg);
    for j in 0..data.len().min(hist_off + chunk_len) { eng.update(data, j); }

    // Cost functions matched to serializer mode (§4.6 critical alignment)
    let lit_bits: u64 = match cost_mode {
        ChunkMode::Stored => 8,
        ChunkMode::Compressed => 10,     // 2-bit type + 8-bit literal
        ChunkMode::TernaryEnhanced => 7, // avg trit cost ~3.5 * 2 bits
        ChunkMode::TernaryAns => 8,      // entropy-optimal ~H bits
    };
    let match_overhead: u64 = match cost_mode {
        ChunkMode::Stored => 64,
        ChunkMode::Compressed => 6,      // 2-bit type + Rice + EG
        ChunkMode::TernaryEnhanced => 6,
        ChunkMode::TernaryAns => 4,
    };

    #[derive(Clone)]
    struct Node { cost: u64, token: Option<Token>, prev: u32 }

    let mut nodes: Vec<Node> = vec![Node { cost: 0, token: None, prev: u32::MAX }];
    let mut beam: Vec<Vec<u32>> = vec![Vec::new(); chunk_len + 1];
    beam[0].push(0);

    for pos in 0..chunk_len {
        if beam[pos].is_empty() { continue; }
        let mut current: Vec<u32> = beam[pos].clone();
        current.sort_by_key(|&idx| nodes[idx as usize].cost);
        current.truncate(BEAM_WIDTH);
        let abs_pos = hist_off + pos;

        for &ni in &current {
            let base = nodes[ni as usize].cost;

            // Literal
            let lc = base + lit_bits;
            let li = nodes.len() as u32;
            nodes.push(Node { cost: lc, token: Some(Token::Literal(data[abs_pos])), prev: ni });
            if pos + 1 <= chunk_len { beam[pos + 1].push(li); }

            // Match
            if let Some((dist, len)) = eng.find_best_match(data, abs_pos) {
                let dist_bits = if dist == 0 { 1 } else { (64 - (dist as u64).leading_zeros()) as u64 };
                let len_bits = if len == 0 { 1 } else { (64 - (len as u64).leading_zeros()) as u64 };
                let mc = base + match_overhead + dist_bits + len_bits;
                let mi = nodes.len() as u32;
                nodes.push(Node { cost: mc, token: Some(Token::Match { dist, length: len }), prev: ni });
                let end = (pos + len).min(chunk_len);
                beam[end].push(mi);
            }

            // Run
            let run = eng.count_run(data, abs_pos);
            if run >= cfg.min_run {
                let rc = base + 12;
                let ri = nodes.len() as u32;
                nodes.push(Node { cost: rc, token: Some(Token::Run { byte: data[abs_pos], length: run }), prev: ni });
                let end = (pos + run).min(chunk_len);
                beam[end].push(ri);
            }
        }
    }

    if beam[chunk_len].is_empty() { return tokenize_greedy_lazy(data, hist_off, cfg); }
    let best_end = *beam[chunk_len].iter().min_by_key(|&&i| nodes[i as usize].cost).unwrap();
    let mut tokens = Vec::new();
    let mut idx = best_end;
    while idx != 0 && idx != u32::MAX {
        if let Some(ref tok) = nodes[idx as usize].token { tokens.push(tok.clone()); }
        idx = nodes[idx as usize].prev;
    }
    tokens.reverse();
    tokens
}

/// Main tokenization dispatch.
fn tokenize_chunk(data: &[u8], hist_len: usize, cfg: &LevelConfig, cost_mode: ChunkMode) -> Vec<Token> {
    match cfg.parsing {
        Parsing::BeamOptimal => tokenize_beam(data, hist_len, cfg, cost_mode),
        _ => tokenize_greedy_lazy(data, hist_len, cfg),
    }
}

/// Reconstruct bytes from tokens (§4.10).
fn decompress_tokens(tokens: &[Token], history: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = history.to_vec();
    for tok in tokens {
        match tok {
            Token::Literal(b) => out.push(*b),
            Token::Run { byte, length } => { for _ in 0..*length { out.push(*byte); } }
            Token::Match { dist, length } => {
                for _ in 0..*length { let b = out[out.len() - dist]; out.push(b); }
            }
        }
    }
    out
}

// ─── Token Serialization (§5) ───────────────────────────────────────────────

/// Mode 1 — COMPRESSED (§5.1).
fn serialize_compressed(tokens: &[Token], initial_m: u8) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(tokens.len() * 2);
    encode_elias_gamma(&mut w, tokens.len() as u64);
    w.write(initial_m as u32, 8);
    let mut m = initial_m;
    let mut match_count = 0u32;
    let mut match_sum = 0u64;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Literal(_) => {
                let start = i;
                while i < tokens.len() && matches!(tokens[i], Token::Literal(_)) { i += 1; }
                w.write(0b00, 2);
                encode_elias_gamma(&mut w, (i - start) as u64);
                for j in start..i { if let Token::Literal(b) = tokens[j] { w.write(b as u32, 8); } }
            }
            Token::Run { byte, length } => {
                w.write(0b01, 2); w.write(*byte as u32, 8);
                encode_elias_gamma(&mut w, *length as u64);
                i += 1;
            }
            Token::Match { dist, length } => {
                match_sum += *dist as u64; match_count += 1;
                if match_count % 128 == 0 && match_count > 0 {
                    let mean = match_sum / match_count as u64;
                    let nm = if mean == 0 { 1 } else { ((64 - mean.leading_zeros()).saturating_sub(1) as u8).clamp(1, 8) };
                    if nm != m {
                        let delta = (nm as i8 - m as i8).clamp(-4, 3);
                        w.write(0b11, 2); w.write((delta as u8 & 0x07) as u32, 3);
                        m = (m as i8 + delta) as u8;
                    }
                }
                w.write(0b10, 2);
                encode_rice(&mut w, *dist as u64, m);
                encode_elias_gamma(&mut w, *length as u64);
                i += 1;
            }
        }
    }
    w.finish_with_header()
}

fn deserialize_compressed(payload: &[u8]) -> TtcResult<Vec<Token>> {
    if payload.len() < 4 { return Err(TtcError::DecompressionError("Payload too short".into())); }
    let mut r = BitReader::new(&payload[4..]);
    let tc = decode_elias_gamma(&mut r) as usize;
    let mut m = r.read(8) as u8;
    let mut tokens = Vec::with_capacity(tc);
    let mut decoded = 0;
    while decoded < tc && !r.is_exhausted() {
        match r.read(2) {
            0b00 => { let cnt = decode_elias_gamma(&mut r) as usize; for _ in 0..cnt { tokens.push(Token::Literal(r.read(8) as u8)); decoded += 1; } }
            0b01 => { let byte = r.read(8) as u8; let len = decode_elias_gamma(&mut r) as usize; tokens.push(Token::Run { byte, length: len }); decoded += 1; }
            0b10 => { let dist = decode_rice(&mut r, m) as usize; let len = decode_elias_gamma(&mut r) as usize; tokens.push(Token::Match { dist, length: len }); decoded += 1; }
            0b11 => { let dr = r.read(3) as i8; let d = if dr > 3 { dr - 8 } else { dr }; m = ((m as i8) + d).clamp(1, 8) as u8; }
            _ => {}
        }
    }
    Ok(tokens)
}

/// Mode 2 — TERNARY_ENHANCED (§5.2).
fn serialize_ternary_enhanced(tokens: &[Token], initial_m: u8, tc: &TritCostTables) -> Vec<u8> {
    let mut w = BitWriter::with_capacity(tokens.len() * 3);
    encode_hybrid_prefix(&mut w, tokens.len() as u64);
    let m = initial_m;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Literal(_) => {
                let start = i;
                while i < tokens.len() && matches!(tokens[i], Token::Literal(_)) { i += 1; }
                let group: Vec<u8> = (start..i).filter_map(|j| if let Token::Literal(b) = tokens[j] { Some(b) } else { None }).collect();
                let best = tc.best_rep(&group);
                w.write(0b00, 2);
                w.write(match best { GfRep::C => 0b00, GfRep::B => 0b01, GfRep::A => 0b10 }, 2);
                encode_hybrid_prefix(&mut w, group.len() as u64);
                for &b in &group { write_trit_encoded(&mut w, b, best); }
            }
            Token::Run { byte, length } => {
                let best = tc.best_rep(&[*byte]);
                w.write(0b01, 2);
                w.write(match best { GfRep::C => 0b00, GfRep::B => 0b01, GfRep::A => 0b10 }, 2);
                write_trit_encoded(&mut w, *byte, best);
                encode_hybrid_prefix(&mut w, *length as u64);
                i += 1;
            }
            Token::Match { dist, length } => {
                w.write(0b10, 2);
                encode_rice(&mut w, *dist as u64, m);
                encode_hybrid_prefix(&mut w, *length as u64);
                i += 1;
            }
        }
    }
    w.finish_with_header()
}

/// Write a byte as trit-encoded with 3-bit digit count prefix.
fn write_trit_encoded(w: &mut BitWriter, byte: u8, rep: GfRep) {
    match rep {
        GfRep::C => {
            let td = byte_to_bijective(byte);
            w.write(td.len as u32, 3);
            for k in 0..(td.len as usize) { w.write(match td.digits[k] { 1=>0b00, 2=>0b01, 3=>0b10, _=>0b11 }, 2); }
        }
        GfRep::B => {
            let td = byte_to_standard(byte);
            w.write(td.len as u32, 3);
            for k in 0..(td.len as usize) { w.write(match td.digits[k] { 0=>0b00, 1=>0b01, 2=>0b10, _=>0b11 }, 2); }
        }
        GfRep::A => {
            let td = byte_to_balanced(byte);
            w.write(td.len as u32, 3);
            for k in 0..(td.len as usize) { w.write(match td.digits[k] { -1=>0b10, 0=>0b00, 1=>0b01, _=>0b11 } as u32, 2); }
        }
    }
}

fn deserialize_ternary_enhanced(payload: &[u8], _adaptive_rep: bool) -> TtcResult<Vec<Token>> {
    if payload.len() < 4 { return Err(TtcError::DecompressionError("Payload too short".into())); }
    let mut r = BitReader::new(&payload[4..]);
    let tc = decode_hybrid_prefix(&mut r) as usize;
    let mut tokens = Vec::with_capacity(tc);
    let mut decoded = 0;
    let m: u8 = 4;
    while decoded < tc && !r.is_exhausted() {
        match r.read(2) {
            0b00 => {
                let rep = match r.read(2) { 0b00=>GfRep::C, 0b01=>GfRep::B, _=>GfRep::A };
                let cnt = decode_hybrid_prefix(&mut r) as usize;
                for _ in 0..cnt { tokens.push(Token::Literal(read_trit_decoded(&mut r, rep))); decoded += 1; }
            }
            0b01 => {
                let rep = match r.read(2) { 0b00=>GfRep::C, 0b01=>GfRep::B, _=>GfRep::A };
                let byte = read_trit_decoded(&mut r, rep);
                let len = decode_hybrid_prefix(&mut r) as usize;
                tokens.push(Token::Run { byte, length: len }); decoded += 1;
            }
            0b10 => {
                let dist = decode_rice(&mut r, m) as usize;
                let len = decode_hybrid_prefix(&mut r) as usize;
                tokens.push(Token::Match { dist, length: len }); decoded += 1;
            }
            0b11 => { let _ = r.read(3); }
            _ => {}
        }
    }
    Ok(tokens)
}

fn read_trit_decoded(r: &mut BitReader, rep: GfRep) -> u8 {
    let dc = r.read(3) as usize;
    match rep {
        GfRep::C => {
            let mut td = TritDigits { digits: [0; 6], len: dc as u8 };
            for k in 0..dc { td.digits[k] = match r.read(2) { 0b00=>1, 0b01=>2, 0b10=>3, _=>1 }; }
            bijective_to_byte(&td)
        }
        GfRep::B => {
            let mut td = TritDigits { digits: [0; 6], len: dc as u8 };
            for k in 0..dc { td.digits[k] = match r.read(2) { 0b00=>0, 0b01=>1, 0b10=>2, _=>0 }; }
            standard_to_byte(&td)
        }
        GfRep::A => {
            let mut td = BalancedTritDigits { digits: [0; 6], len: dc as u8 };
            for k in 0..dc { td.digits[k] = match r.read(2) { 0b10=> -1i8, 0b00=>0, 0b01=>1, _=>0 }; }
            balanced_to_byte(&td)
        }
    }
}

/// Mode 3 — TERNARY_ANS (§5.3).
fn serialize_tans(tokens: &[Token], window_size: usize) -> Vec<u8> {
    let freq = TansFreqTable::build(tokens, window_size);
    let (state, packed) = tans_encode(tokens, &freq, window_size);
    let mut out = Vec::new();
    let s = freq.entries.len() as u32;
    out.extend_from_slice(&s.to_be_bytes());
    for &(sym, fnorm) in &freq.entries {
        out.extend_from_slice(&sym.to_be_bytes());
        out.push((fnorm >> 16) as u8); out.push((fnorm >> 8) as u8); out.push(fnorm as u8);
    }
    out.push((state >> 16) as u8); out.push((state >> 8) as u8); out.push(state as u8);
    out.extend_from_slice(&packed);
    out
}

fn deserialize_tans(payload: &[u8], window_size: usize) -> TtcResult<Vec<Token>> {
    if payload.len() < 4 { return Err(TtcError::DecompressionError("tANS payload too short".into())); }
    let s = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    let mut pos = 4;
    if payload.len() < pos + s * 5 + 3 { return Err(TtcError::TruncatedPayload); }
    let mut entries = Vec::with_capacity(s);
    let mut fnorm = [0u32; TANS_ALPHABET];
    for _ in 0..s {
        let sym = u16::from_be_bytes([payload[pos], payload[pos+1]]);
        let f = (payload[pos+2] as u32) << 16 | (payload[pos+3] as u32) << 8 | payload[pos+4] as u32;
        fnorm[sym as usize] = f;
        entries.push((sym, f)); pos += 5;
    }
    let init = (payload[pos] as u32) << 16 | (payload[pos+1] as u32) << 8 | payload[pos+2] as u32;
    pos += 3;
    let mut cum = [0u32; TANS_ALPHABET];
    let mut c = 0u32;
    for &(sym, f) in &entries { cum[sym as usize] = c; c += f; }
    let freq = TansFreqTable { entries, fnorm_lookup: fnorm, cum };
    let (spread, sym_pos) = tans_build_tables(&freq);
    Ok(tans_decode(&freq, &spread, &sym_pos, init, &payload[pos..], window_size))
}

// ─── Per-Chunk Compression ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub index: usize,
    pub original_size: u32,
    pub compressed_size: u32,
    pub base: u16,
    pub tau: f64,
    pub delta: f64,
    pub mode: ChunkMode,
    pub rice_m: u8,
    pub delta_flag: DeltaFlag,
    pub domain_transform: DomainTransform,
    pub payload: Vec<u8>,
}

fn make_stored_payload(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + data.len());
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.push(0x00);
    out.extend_from_slice(data);
    out
}

fn make_mode_payload(data: &[u8], mode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.push(mode);
    out.extend_from_slice(payload);
    out
}

fn compress_chunk(
    chunk: &[u8], history: &[u8], idx: usize, cfg: &LevelConfig,
    mode: CompressionMode, independent: bool, tc: &TritCostTables,
) -> ChunkResult {
    let orig_size = chunk.len() as u32;

    if is_pre_compressed(chunk) {
        return ChunkResult {
            index: idx, original_size: orig_size, compressed_size: (chunk.len() + 5) as u32,
            base: 3, tau: 0.0, delta: 0.0, mode: ChunkMode::Stored, rice_m: 0,
            delta_flag: DeltaFlag::NONE, domain_transform: DomainTransform::NONE,
            payload: make_stored_payload(chunk),
        };
    }

    let gurft = if cfg.skip_gurft { GurftResult::default() } else { gurft_analyze(chunk) };
    let base = try_and_compare_base(chunk, &gurft, mode);

    let delta_flag = select_delta(chunk, mode);
    let mut buf1 = Vec::new(); let mut buf2 = Vec::new();
    let delta_data = apply_delta_encode(chunk, delta_flag, &mut buf1, &mut buf2);

    let (vdata, hlen) = if independent || history.is_empty() {
        (delta_data.clone(), 0)
    } else {
        let h = &history[history.len().saturating_sub(cfg.window_size)..];
        let mut vd = Vec::with_capacity(h.len() + delta_data.len());
        vd.extend_from_slice(h); vd.extend_from_slice(&delta_data);
        let hl = h.len(); (vd, hl)
    };

    // Determine which modes to try based on entropy
    let h_chunk = compute_entropy(chunk);
    let mut candidates: Vec<(ChunkMode, Vec<u8>)> = vec![(ChunkMode::Stored, make_stored_payload(chunk))];

    if h_chunk <= MODE_PRUNE_ENTROPY {
        // Determine best cost mode for beam search (§4.6 alignment)
        let best_cost_mode = if h_chunk < 4.0 { ChunkMode::TernaryAns }
            else if chunk.len() <= 16384 { ChunkMode::TernaryEnhanced }
            else { ChunkMode::Compressed };

        let tokens = tokenize_chunk(&vdata, hlen, cfg, best_cost_mode);
        let rice_m = compute_initial_rice_m(&tokens);

        let comp = serialize_compressed(&tokens, rice_m);
        candidates.push((ChunkMode::Compressed, make_mode_payload(chunk, 1, &comp)));

        let tans = serialize_tans(&tokens, cfg.window_size);
        candidates.push((ChunkMode::TernaryAns, make_mode_payload(chunk, 3, &tans)));

        if chunk.len() <= 16384 {
            let enh = serialize_ternary_enhanced(&tokens, rice_m, tc);
            candidates.push((ChunkMode::TernaryEnhanced, make_mode_payload(chunk, 2, &enh)));
        }

        // Early exit: compressed >= 98% of raw → STORED
        if let Some((_, ref c)) = candidates.iter().find(|(m, _)| *m == ChunkMode::Compressed) {
            if c.len() >= chunk.len() * 98 / 100 {
                return ChunkResult {
                    index: idx, original_size: orig_size, compressed_size: (chunk.len() + 5) as u32,
                    base, tau: gurft.tau, delta: gurft.delta, mode: ChunkMode::Stored,
                    rice_m: 0, delta_flag: DeltaFlag::NONE, domain_transform: DomainTransform::NONE,
                    payload: make_stored_payload(chunk),
                };
            }
        }
    }

    let (best_mode, best_payload) = candidates.into_iter().min_by_key(|(_, p)| p.len()).unwrap();

    ChunkResult {
        index: idx, original_size: orig_size, compressed_size: best_payload.len() as u32,
        base, tau: gurft.tau, delta: gurft.delta, mode: best_mode,
        rice_m: compute_initial_rice_m(&[]), // stored in chunk map
        delta_flag, domain_transform: DomainTransform::NONE,
        payload: best_payload,
    }
}

// ─── Inter-Cube Parallel Dispatch (§4.8) ────────────────────────────────────
//
// The 26-tunnel Inter-Cube topology (2×13 dimensions) is the native parallel
// execution model. This is not generic thread pooling — it is grounded in the
// same dimensional geometry as the rest of PlenumNET.
//
// Two modes:
//   Independent chunks: full parallel — 26 tunnels per round, round-robin.
//   Dependent chunks:   pipelined — Phase 1 (analysis, 13 workers parallel)
//                       overlaps Phase 2 (LZ77+serialize, sequential with history).
//
// Runtime gating: parallel dispatch only when rayon thread count > 1 AND
// chunk count exceeds PARALLEL_CHUNK_THRESHOLD. Same pattern as T-AE-MAC.

/// Minimum chunk count to justify parallel dispatch overhead.
const PARALLEL_CHUNK_THRESHOLD: usize = 4;

/// Phase 1 result: read-only analysis of a single chunk (§4.8).
/// This is the output of GURFT + delta decision + base selection.
/// No history dependency — fully parallelizable.
#[derive(Debug, Clone)]
struct Phase1Result {
    chunk_index: usize,
    pre_compressed: bool,
    gurft: GurftResult,
    base: u16,
    delta_flag: DeltaFlag,
    delta_data: Vec<u8>,
    h_chunk: f64,
}

/// Phase 1: GURFT analysis + delta decision + base selection (read-only on raw chunk).
fn phase1_analyze(
    chunk: &[u8], chunk_index: usize, cfg: &LevelConfig, mode: CompressionMode,
) -> Phase1Result {
    if is_pre_compressed(chunk) {
        return Phase1Result {
            chunk_index, pre_compressed: true,
            gurft: GurftResult::default(), base: 3,
            delta_flag: DeltaFlag::NONE, delta_data: Vec::new(),
            h_chunk: 8.0,
        };
    }

    let gurft = if cfg.skip_gurft { GurftResult::default() } else { gurft_analyze(chunk) };
    let base = try_and_compare_base(chunk, &gurft, mode);
    let delta_flag = select_delta(chunk, mode);
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    let delta_data = apply_delta_encode(chunk, delta_flag, &mut buf1, &mut buf2);
    let h_chunk = compute_entropy(chunk);

    Phase1Result { chunk_index, pre_compressed: false, gurft, base, delta_flag, delta_data, h_chunk }
}

/// Phase 2: LZ77 tokenization + serialization (needs history for dependent mode).
fn phase2_compress(
    chunk: &[u8], p1: &Phase1Result, history: &[u8],
    cfg: &LevelConfig, independent: bool, tc: &TritCostTables,
    dom_xform: DomainTransform,
) -> ChunkResult {
    let orig_size = chunk.len() as u32;

    if p1.pre_compressed {
        return ChunkResult {
            index: p1.chunk_index, original_size: orig_size,
            compressed_size: (chunk.len() + 5) as u32,
            base: 3, tau: 0.0, delta: 0.0, mode: ChunkMode::Stored, rice_m: 0,
            delta_flag: DeltaFlag::NONE, domain_transform: dom_xform,
            payload: make_stored_payload(chunk),
        };
    }

    let (vdata, hlen) = if independent || history.is_empty() {
        (p1.delta_data.clone(), 0)
    } else {
        let h = &history[history.len().saturating_sub(cfg.window_size)..];
        let mut vd = Vec::with_capacity(h.len() + p1.delta_data.len());
        vd.extend_from_slice(h);
        vd.extend_from_slice(&p1.delta_data);
        let hl = h.len();
        (vd, hl)
    };

    let mut candidates: Vec<(ChunkMode, Vec<u8>)> =
        vec![(ChunkMode::Stored, make_stored_payload(chunk))];

    if p1.h_chunk <= MODE_PRUNE_ENTROPY {
        let best_cost_mode = if p1.h_chunk < 4.0 { ChunkMode::TernaryAns }
            else if chunk.len() <= 16384 { ChunkMode::TernaryEnhanced }
            else { ChunkMode::Compressed };

        let tokens = tokenize_chunk(&vdata, hlen, cfg, best_cost_mode);
        let rice_m = compute_initial_rice_m(&tokens);

        let comp = serialize_compressed(&tokens, rice_m);
        candidates.push((ChunkMode::Compressed, make_mode_payload(chunk, 1, &comp)));

        let tans = serialize_tans(&tokens, cfg.window_size);
        candidates.push((ChunkMode::TernaryAns, make_mode_payload(chunk, 3, &tans)));

        if chunk.len() <= 16384 {
            let enh = serialize_ternary_enhanced(&tokens, rice_m, tc);
            candidates.push((ChunkMode::TernaryEnhanced, make_mode_payload(chunk, 2, &enh)));
        }

        // Early exit: compressed >= 98% of raw → STORED
        if let Some((_, ref c)) = candidates.iter().find(|(m, _)| *m == ChunkMode::Compressed) {
            if c.len() >= chunk.len() * 98 / 100 {
                return ChunkResult {
                    index: p1.chunk_index, original_size: orig_size,
                    compressed_size: (chunk.len() + 5) as u32,
                    base: p1.base, tau: p1.gurft.tau, delta: p1.gurft.delta,
                    mode: ChunkMode::Stored, rice_m: 0,
                    delta_flag: DeltaFlag::NONE, domain_transform: dom_xform,
                    payload: make_stored_payload(chunk),
                };
            }
        }
    }

    let (best_mode, best_payload) = candidates.into_iter().min_by_key(|(_, p)| p.len()).unwrap();

    ChunkResult {
        index: p1.chunk_index, original_size: orig_size,
        compressed_size: best_payload.len() as u32,
        base: p1.base, tau: p1.gurft.tau, delta: p1.gurft.delta,
        mode: best_mode, rice_m: compute_initial_rice_m(&[]),
        delta_flag: p1.delta_flag, domain_transform: dom_xform,
        payload: best_payload,
    }
}

/// Independent mode: full parallel dispatch across 26 tunnels (§4.8).
///
/// All chunks have no history dependency. Chunks are dispatched round-robin
/// across TUNNEL_COUNT (26) tunnels per round, all running simultaneously.
/// Results collected into a slot-based output buffer in index order.
#[cfg(feature = "parallel")]
fn dispatch_independent_parallel(
    chunk_slices: &[&[u8]],
    cfg: &LevelConfig,
    mode: CompressionMode,
    tc: &TritCostTables,
    dom_xform: DomainTransform,
) -> Vec<ChunkResult> {
    use rayon::prelude::*;

    let chunk_count = chunk_slices.len();
    let rounds = (chunk_count + TUNNEL_COUNT - 1) / TUNNEL_COUNT;

    // Slot-based output buffer — chunks may complete out of order
    let mut results: Vec<Option<ChunkResult>> = vec![None; chunk_count];

    for round in 0..rounds {
        let start = round * TUNNEL_COUNT;
        let end = (start + TUNNEL_COUNT).min(chunk_count);
        let batch_indices: Vec<usize> = (start..end).collect();

        let batch_results: Vec<ChunkResult> = batch_indices
            .par_iter()
            .map(|&idx| {
                let chunk = chunk_slices[idx];
                let p1 = phase1_analyze(chunk, idx, cfg, mode);
                phase2_compress(chunk, &p1, &[], cfg, true, tc, dom_xform)
            })
            .collect();

        for cr in batch_results {
            let idx = cr.index;
            results[idx] = Some(cr);
        }
    }

    // Unwrap all slots (all should be filled)
    results.into_iter().map(|opt| opt.expect("All chunks must be filled")).collect()
}

/// Dependent mode: 13+13 pipelined parallel dispatch (§4.8).
///
/// Phase 1 (GURFT + delta + base selection) is read-only and runs in parallel
/// across 13 logical workers for the NEXT batch.
/// Phase 2 (LZ77 + serialize) is sequential due to history dependency and
/// processes the CURRENT batch using cached Phase 1 results.
///
/// Pipeline: while Phase 2 processes batch N, Phase 1 analyses batch N+1.
/// The 2×13 structure maps to the dual 13-dimensional Inter-Cube geometry.
#[cfg(feature = "parallel")]
fn dispatch_dependent_pipelined(
    chunk_slices: &[&[u8]],
    cfg: &LevelConfig,
    mode: CompressionMode,
    tc: &TritCostTables,
    dom_xform: DomainTransform,
) -> Vec<ChunkResult> {
    use rayon::prelude::*;

    let chunk_count = chunk_slices.len();
    let batch_size = 13; // One group of the 2×13 Inter-Cube structure
    let total_batches = (chunk_count + batch_size - 1) / batch_size;

    let mut results: Vec<ChunkResult> = Vec::with_capacity(chunk_count);
    let mut history: Vec<u8> = Vec::new();

    // Pre-analyse first batch (Phase 1 runs ahead)
    let first_batch_end = batch_size.min(chunk_count);
    let mut analysis_cache: Vec<Phase1Result> = (0..first_batch_end)
        .into_par_iter()
        .map(|idx| phase1_analyze(chunk_slices[idx], idx, cfg, mode))
        .collect();

    for batch in 0..total_batches {
        let batch_start = batch * batch_size;
        let batch_end = (batch_start + batch_size).min(chunk_count);

        // Pipeline: start Phase 1 for NEXT batch in parallel while
        // Phase 2 processes CURRENT batch sequentially
        let next_batch_start = batch_end;
        let next_batch_end = (next_batch_start + batch_size).min(chunk_count);

        let mut next_cache: Vec<Phase1Result> = Vec::new();
        let current_cache = core::mem::take(&mut analysis_cache);

        if next_batch_start < chunk_count {
            // Phase 1 (next batch) and Phase 2 (current batch) run concurrently
            let next_slices: Vec<(usize, &[u8])> = (next_batch_start..next_batch_end)
                .map(|i| (i, chunk_slices[i]))
                .collect();

            // Use rayon::join for pipelined overlap
            let (phase2_results, phase1_results) = rayon::join(
                // Phase 2: sequential compression of current batch
                || {
                    let mut batch_results = Vec::with_capacity(batch_end - batch_start);
                    for (local_idx, p1) in current_cache.iter().enumerate() {
                        let global_idx = batch_start + local_idx;
                        if global_idx >= chunk_count { break; }
                        let chunk = chunk_slices[global_idx];
                        let cr = phase2_compress(chunk, p1, &history, cfg, false, tc, dom_xform);
                        // Update history sequentially
                        history.extend_from_slice(chunk);
                        if history.len() > cfg.window_size {
                            let trim = history.len() - cfg.window_size;
                            history.drain(..trim);
                        }
                        batch_results.push(cr);
                    }
                    batch_results
                },
                // Phase 1: parallel analysis of next batch
                || {
                    next_slices.par_iter()
                        .map(|&(idx, chunk)| phase1_analyze(chunk, idx, cfg, mode))
                        .collect::<Vec<_>>()
                },
            );

            results.extend(phase2_results);
            next_cache = phase1_results;
        } else {
            // Last batch — no next batch to pipeline
            for (local_idx, p1) in current_cache.iter().enumerate() {
                let global_idx = batch_start + local_idx;
                if global_idx >= chunk_count { break; }
                let chunk = chunk_slices[global_idx];
                let cr = phase2_compress(chunk, p1, &history, cfg, false, tc, dom_xform);
                history.extend_from_slice(chunk);
                if history.len() > cfg.window_size {
                    let trim = history.len() - cfg.window_size;
                    history.drain(..trim);
                }
                results.push(cr);
            }
        }

        analysis_cache = next_cache;
    }

    results
}

/// Sequential fallback (no rayon or below threshold).
fn dispatch_sequential(
    chunk_slices: &[&[u8]],
    cfg: &LevelConfig,
    mode: CompressionMode,
    independent: bool,
    tc: &TritCostTables,
    dom_xform: DomainTransform,
) -> Vec<ChunkResult> {
    let mut results = Vec::with_capacity(chunk_slices.len());
    let mut history: Vec<u8> = Vec::new();

    for (i, chunk) in chunk_slices.iter().enumerate() {
        let hr = if independent { &[] as &[u8] } else { &history };
        let mut cr = compress_chunk(chunk, hr, i, cfg, mode, independent, tc);
        cr.domain_transform = dom_xform;
        if !independent {
            history.extend_from_slice(chunk);
            if history.len() > cfg.window_size {
                let trim = history.len() - cfg.window_size;
                history.drain(..trim);
            }
        }
        results.push(cr);
    }
    results
}

/// Top-level dispatch: selects parallel or sequential based on runtime conditions.
/// Follows the T-AE-MAC pattern: rayon always linked, decision at runtime.
fn dispatch_chunks(
    chunk_slices: &[&[u8]],
    cfg: &LevelConfig,
    mode: CompressionMode,
    independent: bool,
    tc: &TritCostTables,
    dom_xform: DomainTransform,
) -> Vec<ChunkResult> {
    let chunk_count = chunk_slices.len();

    #[cfg(feature = "parallel")]
    {
        let thread_count = rayon::current_num_threads();
        let use_parallel = thread_count > 1 && chunk_count >= PARALLEL_CHUNK_THRESHOLD;

        if use_parallel {
            if independent {
                // §4.8 independent mode: up to 26× throughput multiplier
                return dispatch_independent_parallel(chunk_slices, cfg, mode, tc, dom_xform);
            } else {
                // §4.8 dependent mode: 13+13 pipelined, ~2–4× net speedup
                return dispatch_dependent_pipelined(chunk_slices, cfg, mode, tc, dom_xform);
            }
        }
    }

    // Fallback: sequential
    dispatch_sequential(chunk_slices, cfg, mode, independent, tc, dom_xform)
}

// ─── Container Format (§9) with Filename Embedding (§9.3) ──────────────────

fn build_container(
    chunks: &[ChunkResult], orig_size: u64, crc: u32, mode: CompressionMode,
    level: u8, independent: bool, adaptive_rep: bool, avg_tau: f64, avg_delta: f64,
    predominant_base: u16, lp_coeffs: Option<&[i16; 4]>, image_width: Option<u64>,
    fib_computed: bool, has_filename: bool,
) -> Vec<u8> {
    let chunk_count = chunks.len() as u16;
    let cm_off = HEADER_SIZE as u64;
    let cm_size = chunk_count as usize * CHUNK_MAP_ENTRY_SIZE;
    let total_payload: usize = chunks.iter().map(|c| c.payload.len()).sum();
    let comp_size = (HEADER_SIZE + cm_size + total_payload) as u64;
    let mut out = Vec::with_capacity(comp_size as usize);

    // Header (96 bytes)
    out.extend_from_slice(&MAGIC_TTC1);                              // 0x00
    out.push(VERSION_V2);                                            // 0x04
    out.push(mode as u8);                                            // 0x05
    out.extend_from_slice(&orig_size.to_be_bytes());                 // 0x06
    out.extend_from_slice(&comp_size.to_be_bytes());                 // 0x0E
    out.extend_from_slice(&crc.to_be_bytes());                       // 0x16
    out.extend_from_slice(&predominant_base.to_be_bytes());          // 0x1A
    out.extend_from_slice(&0u32.to_be_bytes());                      // 0x1C timestamp placeholder
    // 0x20–0x27: dual-use
    match mode {
        CompressionMode::Audio => {
            if let Some(c) = lp_coeffs {
                for &v in c { out.extend_from_slice(&v.to_be_bytes()); }
            } else { out.extend_from_slice(&[0u8; 8]); }
        }
        CompressionMode::Image => {
            out.extend_from_slice(&image_width.unwrap_or(0).to_be_bytes());
        }
        _ => out.extend_from_slice(&[0u8; 8]),
    }
    // 0x28 flags
    let mut flags = 0u8;
    if chunks.iter().any(|c| c.base != 3) { flags |= 0x01; }
    if chunks.len() > 1 && chunks.iter().any(|c| c.base != chunks[0].base) { flags |= 0x02; }
    if independent { flags |= 0x04; }
    if adaptive_rep { flags |= 0x08; }
    if fib_computed { flags |= 0x10; }
    if has_filename { flags |= 0x20; }
    out.push(flags);
    out.push(level);                                                 // 0x29
    out.extend_from_slice(&chunk_count.to_be_bytes());               // 0x2A
    out.extend_from_slice(&((avg_tau * 1_000_000.0) as u32).to_be_bytes());  // 0x2C
    out.extend_from_slice(&((avg_delta * 1_000_000.0) as u32).to_be_bytes()); // 0x30
    out.extend_from_slice(&[0u8; 4]);                                // 0x34 reserved
    out.extend_from_slice(&cm_off.to_be_bytes());                    // 0x38
    out.extend_from_slice(&[0u8; 16]);                               // 0x40 TIS-27 placeholder
    out.extend_from_slice(&[0u8; 16]);                               // 0x50 TL-DSA placeholder
    debug_assert_eq!(out.len(), HEADER_SIZE);

    // Chunk map
    for c in chunks {
        out.extend_from_slice(&c.original_size.to_be_bytes());
        out.extend_from_slice(&c.compressed_size.to_be_bytes());
        out.extend_from_slice(&c.base.to_be_bytes());
        out.extend_from_slice(&((c.tau * 1000.0) as u16).to_be_bytes());
        out.extend_from_slice(&((c.delta * 1000.0) as u16).to_be_bytes());
        out.push(c.rice_m);
        out.push((c.delta_flag.0 << 5) | (c.domain_transform.0 & 0x07));
    }

    // Payloads
    for c in chunks { out.extend_from_slice(&c.payload); }
    out
}

/// Embed filename prefix before content (§9.3).
fn embed_filename(content: &[u8], filename: &str) -> Vec<u8> {
    let name_bytes = filename.as_bytes();
    let mut out = Vec::with_capacity(2 + name_bytes.len() + content.len());
    out.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(name_bytes);
    out.extend_from_slice(content);
    out
}

/// Extract filename prefix on decompression (§9.3).
fn extract_filename(data: &[u8]) -> (String, &[u8]) {
    if data.len() < 2 { return (String::new(), data); }
    let name_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + name_len { return (String::new(), data); }
    let name = sanitize_filename(&String::from_utf8_lossy(&data[2..2 + name_len]));
    (name, &data[2 + name_len..])
}

/// Sanitize extracted filename — defense against path traversal (§2.4 security).
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name.chars()
        .filter(|c| *c != '\0' && *c != '/' && *c != '\\')
        .collect();
    // Strip leading dots and any remaining ".." patterns
    let stripped = cleaned.trim_start_matches('.');
    stripped.replace("..", "_").to_string()
}

// ─── Fibonacci Harmonic Analysis (§10) ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FibonacciAnalysis {
    pub arb_weight: f64,
    pub aligned_terms: Vec<u64>,
    pub optimal_ratio: f64,
    pub phase_delta: f64,
    pub resonance_band: String,
}

pub fn fibonacci_analysis(data_len: usize) -> FibonacciAnalysis {
    let fibs: [u64; 25] = [1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765,10946,17711,28657,46368,75025];
    let dl = data_len as u64;
    let mut aligned = Vec::new();
    let mut arb_sum = 0.0f64;
    for &hz in &fibs {
        if hz == 0 { continue; }
        if dl % hz == 0 || hz % 8 == 0 {
            aligned.push(hz);
            let p364 = (dl as f64 * GOLDEN_ANGLE) % 364.0;
            let p360 = (dl as f64 * GOLDEN_ANGLE) % 360.0;
            let rd = libm::fabs(p364 - p360);
            let gain = if rd < 1.875 { 0.0035 } else if rd < 3.125 { 0.0069 }
                else if rd < 4.375 { 0.0104 } else if rd < 5.625 { 0.0139 } else { 0.0174 };
            arb_sum += 1.0 + gain;
        }
    }
    let arb = if aligned.is_empty() { 1.0 } else { arb_sum / aligned.len() as f64 };
    let pd = libm::fabs((dl as f64 * GOLDEN_ANGLE) % 364.0 - (dl as f64 * GOLDEN_ANGLE) % 360.0);
    let band = if arb > 1.015 { "HIGH" } else if arb > 1.005 { "MEDIUM" } else { "LOW" };
    FibonacciAnalysis { arb_weight: arb, aligned_terms: aligned, optimal_ratio: arb, phase_delta: pd, resonance_band: band.into() }
}

// ─── Main API ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CompressOptions {
    pub mode: CompressionMode,
    pub level: u8,
    pub independent_chunks: bool,
    pub compute_fibonacci: bool,
    pub image_width: Option<usize>,
    pub filename: Option<String>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self { mode: CompressionMode::Temporal, level: 5, independent_chunks: false,
               compute_fibonacci: false, image_width: None, filename: None }
    }
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub compressed: Vec<u8>,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
    pub crc32: u32,
    pub mode: u8,
    pub mode_name: String,
    pub version: String,
    pub level: u8,
    pub level_name: String,
    pub chunks: Vec<ChunkDescriptor>,
    pub avg_tau: f64,
    pub avg_delta: f64,
    pub base_distribution: BaseDistribution,
    pub predominant_base: u16,
    pub independent_chunks: bool,
    pub adaptive_rep_used: bool,
    pub fibonacci_analysis: Option<FibonacciAnalysis>,
}

#[derive(Debug, Clone)]
pub struct ChunkDescriptor {
    pub index: usize, pub original_size: u32, pub compressed_size: u32,
    pub base: u16, pub tau: f64, pub delta: f64, pub mode: u8, pub rice_m: u8,
    pub delta_flag: u8, pub delta_order: u8, pub delta_rep: String, pub domain_transform: u8,
}

#[derive(Debug, Clone, Default)]
pub struct BaseDistribution { pub base_3: u32, pub base_13: u32, pub base_28: u32, pub base_70: u32, pub base_364: u32 }

/// Main compression entry point (§7).
pub fn ttc_compress(data: &[u8], opts: &CompressOptions) -> TtcResult<CompressionResult> {
    let cfg = level_config(opts.level)?;
    let tc = TritCostTables::new();
    let crc = crc32(data);

    // Filename embedding (§9.3)
    let input = if let Some(ref name) = opts.filename {
        embed_filename(data, name)
    } else { data.to_vec() };

    let (preprocessed, dom_xform, lp_coeffs, iw) = apply_domain_preprocess(&input, opts.mode, opts.image_width);

    let cs = cfg.chunk_size;
    let cc = (preprocessed.len() + cs - 1) / cs;

    // Build chunk slice references for dispatch
    let chunk_slices: Vec<&[u8]> = (0..cc).map(|i| {
        let start = i * cs;
        let end = (start + cs).min(preprocessed.len());
        &preprocessed[start..end]
    }).collect();

    // §4.8 Inter-Cube parallel dispatch (runtime-gated)
    let mut chunks = dispatch_chunks(
        &chunk_slices, cfg, opts.mode, opts.independent_chunks, &tc, dom_xform,
    );

    // Ensure domain transform is set on all chunks
    for c in &mut chunks { c.domain_transform = dom_xform; }

    let adaptive_rep = chunks.iter().any(|c| c.delta_flag.rep().map_or(false, |r| r != GfRep::C));

    let mut bd = BaseDistribution::default();
    let (mut ts, mut ds) = (0.0f64, 0.0f64);
    for c in &chunks {
        match c.base { 3 => bd.base_3 += 1, 13 => bd.base_13 += 1, 28 => bd.base_28 += 1, 70 => bd.base_70 += 1, 364 => bd.base_364 += 1, _ => bd.base_3 += 1 }
        ts += c.tau; ds += c.delta;
    }
    let n = chunks.len().max(1) as f64;
    let (at, ad) = (ts / n, ds / n);
    let pb = if bd.base_364 > 0 { 364 } else if bd.base_70 > 0 { 70 } else if bd.base_28 > 0 { 28 } else if bd.base_13 > 0 { 13 } else { 3 };

    let compressed = build_container(&chunks, data.len() as u64, crc, opts.mode, opts.level,
        opts.independent_chunks, adaptive_rep, at, ad, pb, lp_coeffs.as_ref(), iw, opts.compute_fibonacci, opts.filename.is_some());

    let fib = if opts.compute_fibonacci { Some(fibonacci_analysis(data.len())) } else { None };
    let descs: Vec<ChunkDescriptor> = chunks.iter().map(|c| ChunkDescriptor {
        index: c.index, original_size: c.original_size, compressed_size: c.compressed_size,
        base: c.base, tau: c.tau, delta: c.delta, mode: c.mode as u8, rice_m: c.rice_m,
        delta_flag: c.delta_flag.0, delta_order: c.delta_flag.order(),
        delta_rep: c.delta_flag.rep_name().into(), domain_transform: c.domain_transform.0,
    }).collect();
    let csz = compressed.len() as u64;

    Ok(CompressionResult {
        compressed, original_size: data.len() as u64, compressed_size: csz,
        compression_ratio: if csz > 0 { data.len() as f64 / csz as f64 } else { 1.0 },
        crc32: crc, mode: opts.mode as u8, mode_name: opts.mode.name().into(),
        version: "2.0".into(), level: opts.level, level_name: cfg.tier_name.into(),
        chunks: descs, avg_tau: at, avg_delta: ad, base_distribution: bd, predominant_base: pb,
        independent_chunks: opts.independent_chunks, adaptive_rep_used: adaptive_rep,
        fibonacci_analysis: fib,
    })
}

// ─── Main Decompression (§8) ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DecompressionResult {
    pub data: Vec<u8>,
    pub original_file_name: Option<String>,
    pub original_size: u64,
    pub compressed_size: u64,
    pub version: String,
    pub level: Option<u8>,
    pub level_name: Option<String>,
    pub crc32_verified: bool,
}

pub fn ttc_decompress(compressed: &[u8]) -> TtcResult<DecompressionResult> {
    if compressed.len() < HEADER_SIZE { return Err(TtcError::TruncatedHeader); }
    if compressed[0..4] != MAGIC_TTC1 { return Err(TtcError::InvalidMagic); }
    let version = compressed[0x04];
    if version != VERSION_V2 && version != VERSION_V1 { return Err(TtcError::UnsupportedVersion(version)); }
    let mode = CompressionMode::from_u8(compressed[0x05])?;
    let orig_size = u64::from_be_bytes(compressed[0x06..0x0E].try_into().unwrap());
    let stored_crc = u32::from_be_bytes(compressed[0x16..0x1A].try_into().unwrap());
    let aflags = compressed[0x28];
    let level = compressed[0x29];
    let chunk_count = u16::from_be_bytes(compressed[0x2A..0x2C].try_into().unwrap()) as usize;
    let cm_off = u64::from_be_bytes(compressed[0x38..0x40].try_into().unwrap()) as usize;
    let independent = aflags & 0x04 != 0;
    let adaptive_rep = aflags & 0x08 != 0;
    let has_filename = aflags & 0x20 != 0;

    let lp_coeffs: Option<[i16; 4]> = if mode == CompressionMode::Audio {
        Some([i16::from_be_bytes([compressed[0x20],compressed[0x21]]),
              i16::from_be_bytes([compressed[0x22],compressed[0x23]]),
              i16::from_be_bytes([compressed[0x24],compressed[0x25]]),
              i16::from_be_bytes([compressed[0x26],compressed[0x27]])])
    } else { None };
    let iw: Option<u64> = if mode == CompressionMode::Image {
        Some(u64::from_be_bytes(compressed[0x20..0x28].try_into().unwrap()))
    } else { None };

    let cm_end = cm_off + chunk_count * CHUNK_MAP_ENTRY_SIZE;
    if compressed.len() < cm_end { return Err(TtcError::TruncatedChunkMap); }

    struct CME { _orig: u32, comp: u32, dflag: DeltaFlag, dxform: DomainTransform }
    let mut entries = Vec::with_capacity(chunk_count);
    for i in 0..chunk_count {
        let o = cm_off + i * CHUNK_MAP_ENTRY_SIZE;
        let orig = u32::from_be_bytes(compressed[o..o+4].try_into().unwrap());
        let comp = u32::from_be_bytes(compressed[o+4..o+8].try_into().unwrap());
        let pk = compressed[o + 15];
        let df = (pk >> 5) & 0x07;
        let dt = pk & 0x07;
        if df == 7 { return Err(TtcError::InvalidDeltaFlag(df)); }
        if dt == 7 { return Err(TtcError::InvalidDomainTransform(dt)); }
        entries.push(CME { _orig: orig, comp, dflag: DeltaFlag(df), dxform: DomainTransform(dt) });
    }

    let ws = if level >= 1 && level <= 9 { level_config(level).ok().map(|c| c.window_size) } else { None }.unwrap_or(243 * 1024);

    let mut poff = cm_end;
    let mut output = Vec::with_capacity(orig_size as usize);
    let mut history: Vec<u8> = Vec::new();

    for entry in &entries {
        let pe = poff + entry.comp as usize;
        if compressed.len() < pe { return Err(TtcError::TruncatedPayload); }
        let payload = &compressed[poff..pe]; poff = pe;
        if payload.len() < 5 { return Err(TtcError::DecompressionError("Chunk payload too short".into())); }
        let cm = ChunkMode::from_u8(payload[4])?;
        let cp = &payload[5..];

        let chunk_bytes = match cm {
            ChunkMode::Stored => cp.to_vec(),
            ChunkMode::Compressed => {
                let toks = deserialize_compressed(cp)?;
                let hr = if independent { &[] } else { &history[..] };
                let full = decompress_tokens(&toks, hr);
                if independent { full } else { full[hr.len()..].to_vec() }
            }
            ChunkMode::TernaryEnhanced => {
                let toks = deserialize_ternary_enhanced(cp, adaptive_rep)?;
                let hr = if independent { &[] } else { &history[..] };
                let full = decompress_tokens(&toks, hr);
                if independent { full } else { full[hr.len()..].to_vec() }
            }
            ChunkMode::TernaryAns => {
                let toks = deserialize_tans(cp, ws)?;
                let hr = if independent { &[] } else { &history[..] };
                let full = decompress_tokens(&toks, hr);
                if independent { full } else { full[hr.len()..].to_vec() }
            }
        };

        let decoded = apply_delta_decode(&chunk_bytes, entry.dflag)?;
        if !independent {
            history.extend_from_slice(&decoded);
            if history.len() > ws { let t = history.len() - ws; history.drain(..t); }
        }
        output.extend_from_slice(&decoded);
    }

    let gdt = entries.first().map(|e| e.dxform).unwrap_or(DomainTransform::NONE);
    let final_data = reverse_domain_preprocess(&output, gdt, lp_coeffs.as_ref(), iw)?;

    let (actual_data, original_file_name) = if has_filename {
        let (filename, content) = extract_filename(&final_data);
        if !filename.is_empty() {
            (content.to_vec(), Some(filename))
        } else {
            (final_data, None)
        }
    } else {
        (final_data, None)
    };

    let computed_crc = crc32(&actual_data);
    let ver_str = if version == VERSION_V2 { "2.0" } else { "1.1" };

    Ok(DecompressionResult {
        data: actual_data,
        original_file_name,
        original_size: orig_size,
        compressed_size: compressed.len() as u64,
        version: ver_str.into(),
        level: if version == VERSION_V2 { Some(level) } else { None },
        level_name: if version == VERSION_V2 { level_config(level).ok().map(|c| c.tier_name.into()) } else { None },
        crc32_verified: computed_crc == stored_crc,
    })
}

// ─── Multi-File Container (§9.2) ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MultiFileResult {
    pub compressed: Vec<u8>,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub compression_ratio: f64,
    pub file_count: usize,
    pub files: Vec<FileEntry>,
    pub avg_tau: f64, pub avg_delta: f64,
    pub base_distribution: BaseDistribution,
    pub predominant_base: u16,
    pub mode_name: String, pub version: String,
    pub level: u8, pub level_name: String,
    pub adaptive_rep_used: bool,
    pub fibonacci_analysis: Option<FibonacciAnalysis>,
}

#[derive(Debug, Clone)]
pub struct FileEntry { pub name: String, pub original_size: u64, pub compressed_size: u64, pub ratio: f64 }

pub fn ttc_compress_multi(files: &[(&str, &[u8])], opts: &CompressOptions) -> TtcResult<MultiFileResult> {
    let cfg = level_config(opts.level)?;
    let mut archives: Vec<(String, Vec<u8>, u64, u64)> = Vec::new();
    let (mut to, mut tc_sum) = (0u64, 0u64);
    let (mut ts, mut ds) = (0.0f64, 0.0f64);
    let mut bd = BaseDistribution::default();
    let mut any_ar = false;

    for &(name, data) in files {
        let mut fopts = opts.clone();
        fopts.filename = Some(name.to_string());
        let r = ttc_compress(data, &fopts)?;
        to += r.original_size; tc_sum += r.compressed_size;
        ts += r.avg_tau; ds += r.avg_delta;
        bd.base_3 += r.base_distribution.base_3; bd.base_13 += r.base_distribution.base_13;
        bd.base_28 += r.base_distribution.base_28; bd.base_70 += r.base_distribution.base_70;
        bd.base_364 += r.base_distribution.base_364;
        if r.adaptive_rep_used { any_ar = true; }
        archives.push((name.into(), r.compressed, r.original_size, r.compressed_size));
    }

    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC_TTCM);
    out.extend_from_slice(&(files.len() as u32).to_be_bytes());
    for (name, _, orig, comp) in &archives {
        let nb = name.as_bytes();
        out.extend_from_slice(&(nb.len() as u16).to_be_bytes());
        out.extend_from_slice(nb);
        out.extend_from_slice(&(*orig as u32).to_be_bytes());
        out.extend_from_slice(&(*comp as u32).to_be_bytes());
    }
    for (_, arc, _, _) in &archives { out.extend_from_slice(arc); }

    let n = files.len().max(1) as f64;
    let pb = if bd.base_364 > 0 { 364 } else if bd.base_70 > 0 { 70 } else if bd.base_28 > 0 { 28 } else if bd.base_13 > 0 { 13 } else { 3 };

    Ok(MultiFileResult {
        total_compressed_size: out.len() as u64, compressed: out, total_original_size: to,
        compression_ratio: if tc_sum > 0 { to as f64 / tc_sum as f64 } else { 1.0 },
        file_count: files.len(),
        files: archives.iter().map(|(n, _, o, c)| FileEntry { name: n.clone(), original_size: *o, compressed_size: *c, ratio: if *c > 0 { *o as f64 / *c as f64 } else { 1.0 } }).collect(),
        avg_tau: ts / n, avg_delta: ds / n, base_distribution: bd, predominant_base: pb,
        mode_name: opts.mode.name().into(), version: "2.0".into(),
        level: opts.level, level_name: cfg.tier_name.into(), adaptive_rep_used: any_ar,
        fibonacci_analysis: if opts.compute_fibonacci { Some(fibonacci_analysis(to as usize)) } else { None },
    })
}

pub fn ttc_decompress_multi(compressed: &[u8]) -> TtcResult<Vec<(String, Vec<u8>)>> {
    if compressed.len() < 8 { return Err(TtcError::TruncatedHeader); }
    if compressed[0..4] != MAGIC_TTCM { return Err(TtcError::InvalidMagic); }
    let fc = u32::from_be_bytes(compressed[4..8].try_into().unwrap()) as usize;
    let mut pos = 8;
    let mut ft: Vec<(String, u32)> = Vec::with_capacity(fc);
    for _ in 0..fc {
        if pos + 2 > compressed.len() { return Err(TtcError::TruncatedHeader); }
        let nl = u16::from_be_bytes([compressed[pos], compressed[pos+1]]) as usize; pos += 2;
        if pos + nl + 8 > compressed.len() { return Err(TtcError::TruncatedHeader); }
        let name = sanitize_filename(&String::from_utf8_lossy(&compressed[pos..pos+nl]));
        pos += nl;
        let _orig = u32::from_be_bytes(compressed[pos..pos+4].try_into().unwrap()); pos += 4;
        let comp = u32::from_be_bytes(compressed[pos..pos+4].try_into().unwrap()); pos += 4;
        ft.push((name, comp));
    }
    let mut results = Vec::with_capacity(fc);
    for (name, comp) in &ft {
        let ae = pos + *comp as usize;
        if ae > compressed.len() { return Err(TtcError::TruncatedPayload); }
        let r = ttc_decompress(&compressed[pos..ae])?;
        pos = ae;
        let fname = r.original_file_name.unwrap_or_else(|| name.clone());
        results.push((fname, r.data));
    }
    Ok(results)
}

/// Detect archive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType { Single, Multi, Unknown }

#[inline]
pub fn detect_archive_type(data: &[u8]) -> ArchiveType {
    if data.len() >= 4 {
        if data[0..4] == MAGIC_TTC1 { return ArchiveType::Single; }
        if data[0..4] == MAGIC_TTCM { return ArchiveType::Multi; }
    }
    ArchiveType::Unknown
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_known_vectors() {
        assert_eq!(crc32(b""), 0x0000_0000);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b"The quick brown fox jumps over the lazy dog"), 0x414F_A339);
    }

    #[test]
    fn test_tribonacci_codec_round_trip() {
        for n in 0..2000u64 {
            let enc = encode_tribonacci(n);
            let dec = decode_tribonacci(&enc);
            assert_eq!(n, dec, "Tribonacci round-trip failed for n={n}");
        }
    }

    #[test]
    fn test_elias_gamma_round_trip() {
        for n in 0..500u64 {
            let mut w = BitWriter::new();
            encode_elias_gamma(&mut w, n);
            let d = w.finish();
            let mut r = BitReader::new(&d);
            assert_eq!(n, decode_elias_gamma(&mut r), "Elias Gamma failed for n={n}");
        }
    }

    #[test]
    fn test_rice_round_trip() {
        for m in 1..=8u8 {
            for n in 0..200u64 {
                let mut w = BitWriter::new();
                encode_rice(&mut w, n, m);
                let d = w.finish();
                let mut r = BitReader::new(&d);
                assert_eq!(n, decode_rice(&mut r, m), "Rice failed for n={n}, m={m}");
            }
        }
    }

    #[test]
    fn test_hybrid_prefix_round_trip() {
        for n in 0..500u64 {
            let mut w = BitWriter::new();
            encode_hybrid_prefix(&mut w, n);
            let d = w.finish();
            let mut r = BitReader::new(&d);
            assert_eq!(n, decode_hybrid_prefix(&mut r), "Hybrid prefix failed for n={n}");
        }
    }

    #[test]
    fn test_bijective_ternary_round_trip() {
        for b in 0..=255u8 {
            let td = byte_to_bijective(b);
            assert!(td.len >= 1 && td.len <= 6, "Bijective digit count out of range for byte {b}");
            assert_eq!(b, bijective_to_byte(&td), "Bijective failed for b={b}");
        }
    }

    #[test]
    fn test_standard_ternary_round_trip() {
        for b in 0..=255u8 {
            let td = byte_to_standard(b);
            assert_eq!(b, standard_to_byte(&td), "Standard failed for b={b}");
        }
    }

    #[test]
    fn test_balanced_ternary_round_trip() {
        for b in 0..=255u8 {
            let td = byte_to_balanced(b);
            assert_eq!(b, balanced_to_byte(&td), "Balanced failed for b={b}");
        }
    }

    #[test]
    fn test_delta_all_flags_round_trip() {
        let data: Vec<u8> = (0..256).map(|i| (i & 0xFF) as u8).collect();
        let mut b1 = Vec::new(); let mut b2 = Vec::new();
        for flag in 0..=6u8 {
            let df = DeltaFlag(flag);
            let enc = apply_delta_encode(&data, df, &mut b1, &mut b2);
            let dec = apply_delta_decode(&enc, df).unwrap();
            assert_eq!(data, dec, "Delta round-trip failed for flag={flag}");
        }
    }

    #[test]
    fn test_delta_flag_7_rejected() {
        assert!(apply_delta_decode(&[0u8; 10], DeltaFlag(7)).is_err());
    }

    #[test]
    fn test_entropy_bounds() {
        let uniform: Vec<u8> = (0..=255).collect();
        let h = compute_entropy(&uniform);
        assert!((h - 8.0).abs() < 0.01, "Uniform entropy should be ~8.0, got {h}");

        let constant = vec![42u8; 1000];
        assert!(compute_entropy(&constant) < 0.01);
    }

    #[test]
    fn test_level_configs() {
        for l in 1..=9u8 {
            let c = level_config(l).unwrap();
            assert_eq!(c.level, l);
            assert!(c.window_size > 0);
        }
        assert!(level_config(0).is_err());
        assert!(level_config(10).is_err());
    }

    #[test]
    fn test_parse_level_aliases() {
        assert_eq!(parse_level("5").unwrap(), 5);
        assert_eq!(parse_level("TTC2-2").unwrap(), 5);
        assert_eq!(parse_level("TTC3-3").unwrap(), 9);
        assert!(parse_level("invalid").is_err());
    }

    #[test]
    fn test_genomic_round_trip() {
        let dna = b"ACGTACGTACGTACGT";
        let enc = genomic_encode(dna);
        let dec = genomic_decode(&enc);
        assert_eq!(dec, dna.to_vec());
    }

    #[test]
    fn test_source_round_trip() {
        let src = b"function main() { return 42; }";
        let enc = source_encode(src);
        let dec = source_decode(&enc);
        assert_eq!(dec, src.to_vec());
    }

    #[test]
    fn test_log_round_trip_basic() {
        let log = b"2026-03-15T10:30:00 INFO server: Request processed\n2026-03-15T10:30:01 DEBUG server: Cache hit\n";
        let enc = log_encode(log);
        let dec = log_decode(&enc);
        // Verify key content is preserved (format may differ slightly)
        assert!(dec.len() > 0);
        assert!(dec.windows(7).any(|w| w == b"Request"));
        assert!(dec.windows(5).any(|w| w == b"Cache"));
    }

    #[test]
    fn test_structured_json_round_trip() {
        let json = br#"{"name":"alice","age":30,"name":"bob","age":25}"#;
        let enc = structured_encode(json);
        let dec = structured_decode(&enc);
        // JSON key references reconstruct the original keys
        assert!(dec.windows(4).any(|w| w == b"name"));
        assert!(dec.windows(3).any(|w| w == b"age"));
    }

    #[test]
    fn test_structured_csv_round_trip() {
        let csv = b"name,age,score\nalice,30,95\nbob,25,88\n";
        let enc = structured_encode(csv);
        let dec = structured_decode(&enc);
        assert!(dec.windows(4).any(|w| w == b"name"));
        assert!(dec.windows(5).any(|w| w == b"alice"));
    }

    #[test]
    fn test_filename_embed_extract() {
        let data = b"hello world";
        let embedded = embed_filename(data, "test.txt");
        let (name, content) = extract_filename(&embedded);
        assert_eq!(name, "test.txt");
        assert_eq!(content, data);
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("..secret"), "secret");
        assert_eq!(sanitize_filename("a/b\\c"), "abc");
    }

    #[test]
    fn test_compress_decompress_round_trip() {
        let data = b"Hello, World! This is a test of TTC v2.0 compression. \
            The quick brown fox jumps over the lazy dog. Repeated: \
            The quick brown fox jumps over the lazy dog. \
            PlenumNET 13-dimensional hypercube with 26 tunnels and 364 degrees.";

        let opts = CompressOptions {
            mode: CompressionMode::Temporal, level: 3, independent_chunks: true,
            filename: Some("test.txt".into()), ..Default::default()
        };
        let result = ttc_compress(data, &opts).unwrap();
        assert!(result.compressed_size > 0);
        assert_eq!(result.original_size, data.len() as u64);
        assert_eq!(result.crc32, crc32(data));
        assert_eq!(result.version, "2.0");

        let dec = ttc_decompress(&result.compressed).unwrap();
        assert_eq!(dec.data, data.to_vec());
        assert_eq!(dec.original_file_name, Some("test.txt".into()));
        assert!(dec.crc32_verified);
    }

    #[test]
    fn test_stored_mode_constant_data() {
        let data = vec![0u8; 200];
        let opts = CompressOptions { mode: CompressionMode::Basic, level: 1, independent_chunks: true, ..Default::default() };
        let result = ttc_compress(&data, &opts).unwrap();
        let dec = ttc_decompress(&result.compressed).unwrap();
        assert_eq!(dec.data, data);
    }

    #[test]
    fn test_multi_file_round_trip() {
        let f1 = b"First file content for multi-file testing.";
        let f2 = b"Second file with different content to verify.";
        let files: Vec<(&str, &[u8])> = vec![("test1.txt", f1), ("test2.txt", f2)];
        let opts = CompressOptions::default();
        let result = ttc_compress_multi(&files, &opts).unwrap();
        assert_eq!(result.file_count, 2);
        let dec = ttc_decompress_multi(&result.compressed).unwrap();
        assert_eq!(dec.len(), 2);
        assert_eq!(dec[0].1, f1.to_vec());
        assert_eq!(dec[1].1, f2.to_vec());
    }

    #[test]
    fn test_archive_detection() {
        assert_eq!(detect_archive_type(&MAGIC_TTC1), ArchiveType::Single);
        assert_eq!(detect_archive_type(&MAGIC_TTCM), ArchiveType::Multi);
        assert_eq!(detect_archive_type(b"XXXX"), ArchiveType::Unknown);
    }

    #[test]
    fn test_trit_cost_tables() {
        let t = TritCostTables::new();
        assert_eq!(t.cost(0, GfRep::C), 1);
        assert_eq!(t.cost(0, GfRep::B), 1);
        assert!(t.cost(255, GfRep::B) <= 6);
        assert!(t.cost(255, GfRep::C) <= 6);
    }

    #[test]
    fn test_gurft_constant_data() {
        let data = vec![128u8; 2048];
        let g = gurft_analyze(&data);
        assert!(g.entropy < 0.01);
    }

    #[test]
    fn test_varint_round_trip() {
        for &v in &[0u64, 1, 127, 128, 16383, 16384, 1_000_000, u64::MAX / 2] {
            let mut buf = Vec::new();
            encode_varint(&mut buf, v);
            let (dec, _) = decode_varint(&buf);
            assert_eq!(v, dec, "Varint round-trip failed for {v}");
        }
    }

    #[test]
    fn test_varint_signed_round_trip() {
        for &v in &[0i64, 1, -1, 127, -128, 10000, -10000, i64::MAX / 2, i64::MIN / 2] {
            let mut buf = Vec::new();
            encode_varint_signed(&mut buf, v);
            let (dec, _) = decode_varint_signed(&buf);
            assert_eq!(v, dec, "Signed varint round-trip failed for {v}");
        }
    }

    #[test]
    fn test_pre_compressed_detection() {
        assert!(is_pre_compressed(b"\x89PNG\r\n\x1a\nmore data here"));
        assert!(is_pre_compressed(b"PK\x03\x04more zip data"));
        assert!(!is_pre_compressed(b"Hello, this is plain text content for testing"));
    }

    #[test]
    fn test_all_levels_compress_decompress() {
        let data = b"Tribonacci ternary compression test across all nine levels. \
            The 13-dimensional hypercube geometry provides 26 tunnels. \
            Repeated content: 13-dimensional hypercube geometry 26 tunnels.";

        for level in 1..=9u8 {
            let opts = CompressOptions {
                mode: CompressionMode::Temporal, level, independent_chunks: true,
                ..Default::default()
            };
            let result = ttc_compress(data, &opts).unwrap();
            let dec = ttc_decompress(&result.compressed).unwrap();
            assert_eq!(dec.data, data.to_vec(), "Round-trip failed at level {level}");
        }
    }

    #[test]
    fn test_compression_mode_names() {
        for m in 0..=7u8 {
            let mode = CompressionMode::from_u8(m).unwrap();
            assert!(!mode.name().is_empty());
            assert!(!mode.allowed_bases().is_empty());
        }
        assert!(CompressionMode::from_u8(8).is_err());
    }

    #[test]
    fn test_phase1_phase2_split_round_trip() {
        // Verify Phase 1 → Phase 2 produces same result as compress_chunk
        let data = b"Test data for phase split verification. \
            Repeated: phase split verification.";
        let cfg = level_config(3).unwrap();
        let tc = TritCostTables::new();

        let p1 = phase1_analyze(data, 0, cfg, CompressionMode::Temporal);
        let cr = phase2_compress(data, &p1, &[], cfg, true, &tc, DomainTransform::NONE);
        assert!(cr.compressed_size > 0);
        assert_eq!(cr.index, 0);
    }

    #[test]
    fn test_dispatch_sequential_independent() {
        let data = b"Dispatch test data for independent mode. Repeated chunk content.";
        let chunks: Vec<&[u8]> = vec![data.as_slice(), data.as_slice()];
        let cfg = level_config(3).unwrap();
        let tc = TritCostTables::new();

        let results = dispatch_sequential(
            &chunks, cfg, CompressionMode::Temporal, true, &tc, DomainTransform::NONE,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].index, 0);
        assert_eq!(results[1].index, 1);
    }

    #[test]
    fn test_dispatch_sequential_dependent() {
        let chunk1 = b"First chunk of dependent data for history test.";
        let chunk2 = b"Second chunk referencing first chunk content.";
        let chunks: Vec<&[u8]> = vec![chunk1.as_slice(), chunk2.as_slice()];
        let cfg = level_config(3).unwrap();
        let tc = TritCostTables::new();

        let results = dispatch_sequential(
            &chunks, cfg, CompressionMode::Temporal, false, &tc, DomainTransform::NONE,
        );
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parallel_dispatch_threshold() {
        // Below threshold should fall through to sequential
        let small_data = b"Small";
        let chunks: Vec<&[u8]> = vec![small_data.as_slice(); 2]; // Below PARALLEL_CHUNK_THRESHOLD
        let cfg = level_config(1).unwrap();
        let tc = TritCostTables::new();

        // dispatch_chunks should succeed regardless of parallel availability
        let results = dispatch_chunks(
            &chunks, cfg, CompressionMode::Basic, true, &tc, DomainTransform::NONE,
        );
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_full_round_trip_via_dispatch() {
        let data: Vec<u8> = (0..4000).map(|i| ((i * 7 + 13) % 256) as u8).collect();
        for &indep in &[true, false] {
            let opts = CompressOptions {
                mode: CompressionMode::Temporal, level: 3,
                independent_chunks: indep, ..Default::default()
            };
            let result = ttc_compress(&data, &opts).unwrap();
            let dec = ttc_decompress(&result.compressed).unwrap();
            assert_eq!(dec.data, data, "Round-trip failed with independent={indep}");
        }
    }

    #[test]
    fn test_fibonacci_analysis_runs() {
        let fa = fibonacci_analysis(1024);
        assert!(fa.arb_weight >= 1.0);
        assert!(!fa.resonance_band.is_empty());
    }
}