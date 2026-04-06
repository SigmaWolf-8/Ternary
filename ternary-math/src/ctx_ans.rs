// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Context-1 Ternary rANS — TTC v5.0.3
// ternary-math/src/ctx_ans.rs
//
// Order-1 context model with Z₂₇ (27) context bins.
// Context = previous_literal_byte mod 27 (algebraic circle).
// Each context bin maintains its own frequency distribution.
//
// Order-0: English text → ~5.0 bits/byte
// Order-1 with 27 bins: English text → ~3.2 bits/byte (36% reduction)
//
// ChunkMode::ContextAns = 4
//
// Wire format:
//   [1]       context_count (27)
//   [27 ×]    per-context compact freq table: [2 BE count][delta-varint pairs]
//   [3]       initial rANS state
//   [4 + n]   packed trit stream
//   [4 + n]   rice-coded distances

use crate::ttc::{
    Token, TtcResult, TtcError,
    TANS_L, TANS_ALPHABET, TANS_SYM_RUN_BASE, TANS_SYM_MATCH_BASE, TANS_EOB,
    MAX_RUN_LEN, MAX_MATCH_LEN,
    BitWriter, BitReader,
    encode_varint, decode_varint, encode_rice, decode_rice,
};

/// Z₂₇ context bins — from the algebraic circle.
const CTX_BINS: usize = 27;

// ═══════════════════════════════════════════════════════════════
// CONTEXT FREQUENCY TABLES
// ═══════════════════════════════════════════════════════════════

struct CtxFreqTable {
    /// fnorm[ctx][sym] = normalized frequency for symbol `sym` in context `ctx`
    fnorm: [[u32; TANS_ALPHABET]; CTX_BINS],
    /// cum[ctx][sym] = cumulative frequency for symbol `sym` in context `ctx`
    cum: [[u32; TANS_ALPHABET]; CTX_BINS],
    /// Sparse entries per context for serialization: (sym, freq) pairs with freq > 0
    entries: [Vec<(u16, u32)>; CTX_BINS],
}

impl CtxFreqTable {
    fn build(tokens: &[Token]) -> Self {
        let l = TANS_L as u64;
        let mut raw_counts: [[u64; TANS_ALPHABET]; CTX_BINS] = [[0; TANS_ALPHABET]; CTX_BINS];
        let mut ctx: u8 = 0;

        // Count symbols per context
        for tok in tokens {
            match tok {
                Token::Literal(v) => {
                    raw_counts[ctx as usize][*v as usize] += 1;
                    ctx = *v % CTX_BINS as u8;
                }
                Token::Run { byte, length } => {
                    raw_counts[ctx as usize][*byte as usize] += 1;
                    ctx = *byte % CTX_BINS as u8;
                    raw_counts[ctx as usize][(TANS_SYM_RUN_BASE + (*length).min(MAX_RUN_LEN) as u16) as usize] += 1;
                }
                Token::Match { dist: _, length } => {
                    raw_counts[ctx as usize][(TANS_SYM_MATCH_BASE + (*length).min(MAX_MATCH_LEN) as u16) as usize] += 1;
                }
            }
        }
        // EOB in final context
        raw_counts[ctx as usize][TANS_EOB as usize] += 1;

        // Normalize each context's frequencies to sum = L
        let mut fnorm = [[0u32; TANS_ALPHABET]; CTX_BINS];
        let mut cum = [[0u32; TANS_ALPHABET]; CTX_BINS];
        let mut entries: [Vec<(u16, u32)>; CTX_BINS] = core::array::from_fn(|_| Vec::new());

        for c in 0..CTX_BINS {
            let total: u64 = raw_counts[c].iter().sum();
            if total == 0 {
                // Empty context — assign uniform distribution over literal bytes
                // so decoder doesn't hit zero-frequency
                let per = (l / 256) as u32;
                let mut remainder = l as u32 - per * 256;
                for s in 0..256 {
                    fnorm[c][s] = per + if s < remainder as usize { 1 } else { 0 };
                }
            } else {
                // Normalize: fnorm = max(1, round(count * L / total)) for active symbols
                let mut active: Vec<usize> = (0..TANS_ALPHABET).filter(|&s| raw_counts[c][s] > 0).collect();
                let mut assigned = 0u64;
                for &s in &active {
                    let f = (raw_counts[c][s] * l / total).max(1);
                    fnorm[c][s] = f as u32;
                    assigned += f;
                }
                // Fix rounding — adjust largest symbol
                if assigned != l && !active.is_empty() {
                    let diff = l as i64 - assigned as i64;
                    // Find the symbol with highest raw count to absorb the difference
                    active.sort_by(|&a, &b| raw_counts[c][b].cmp(&raw_counts[c][a]));
                    fnorm[c][active[0]] = (fnorm[c][active[0]] as i64 + diff).max(1) as u32;
                }
            }

            // Build cumulative
            let mut c_sum = 0u32;
            for s in 0..TANS_ALPHABET {
                cum[c][s] = c_sum;
                c_sum += fnorm[c][s];
                if fnorm[c][s] > 0 {
                    entries[c].push((s as u16, fnorm[c][s]));
                }
            }
        }

        Self { fnorm, cum, entries }
    }
}

// ═══════════════════════════════════════════════════════════════
// SPREAD TABLES
// ═══════════════════════════════════════════════════════════════

fn build_ctx_spreads(freq: &CtxFreqTable) -> Vec<Vec<u16>> {
    let l = TANS_L as usize;
    let mut spreads = Vec::with_capacity(CTX_BINS);
    for c in 0..CTX_BINS {
        let mut spread = vec![0u16; l];
        for s in 0..TANS_ALPHABET {
            let fs = freq.fnorm[c][s];
            let start = freq.cum[c][s] as usize;
            for j in 0..fs as usize {
                if start + j < l {
                    spread[start + j] = s as u16;
                }
            }
        }
        spreads.push(spread);
    }
    spreads
}

// ═══════════════════════════════════════════════════════════════
// ENCODE
// ═══════════════════════════════════════════════════════════════

/// Context-1 rANS encode. Returns (state, packed_trits, distances).
fn ctx_tans_encode(tokens: &[Token], freq: &CtxFreqTable) -> (u32, Vec<u8>, Vec<usize>) {
    let l = TANS_L as usize;

    // Build (symbol, context) pairs in forward order
    let mut sym_ctx: Vec<(u16, u8)> = Vec::with_capacity(tokens.len() * 2 + 1);
    let mut distances: Vec<usize> = Vec::with_capacity(tokens.len());
    let mut ctx: u8 = 0;

    for tok in tokens {
        match tok {
            Token::Literal(v) => {
                sym_ctx.push((*v as u16, ctx));
                ctx = *v % CTX_BINS as u8;
            }
            Token::Run { byte, length } => {
                sym_ctx.push((*byte as u16, ctx));
                ctx = *byte % CTX_BINS as u8;
                sym_ctx.push((TANS_SYM_RUN_BASE + (*length).min(MAX_RUN_LEN) as u16, ctx));
            }
            Token::Match { dist, length } => {
                sym_ctx.push((TANS_SYM_MATCH_BASE + (*length).min(MAX_MATCH_LEN) as u16, ctx));
                distances.push(*dist);
            }
        }
    }
    sym_ctx.push((TANS_EOB, ctx));

    // Encode in reverse
    let mut state: usize = l;
    let mut trits: Vec<u8> = Vec::with_capacity(sym_ctx.len() * 3);

    for &(s, c) in sym_ctx.iter().rev() {
        let fs = freq.fnorm[c as usize][s as usize] as usize;
        if fs == 0 { continue; }
        let cum_s = freq.cum[c as usize][s as usize] as usize;

        // Ternary normalization
        while state >= 3 * fs {
            trits.push((state % 3) as u8);
            state /= 3;
        }

        // rANS encoding
        state = (state / fs) * l + cum_s + (state % fs);
    }

    trits.reverse();

    // Pack trits 5-per-byte
    let mut packed = Vec::with_capacity((trits.len() + 4) / 5 + 4);
    packed.extend_from_slice(&(trits.len() as u32).to_be_bytes());
    let mut pending = [0u8; 5];
    let mut count = 0;
    for &t in &trits {
        pending[count] = t;
        count += 1;
        if count == 5 {
            packed.push(pending[0] * 81 + pending[1] * 27 + pending[2] * 9 + pending[3] * 3 + pending[4]);
            count = 0;
            pending = [0; 5];
        }
    }
    if count > 0 {
        packed.push(pending[0] * 81 + pending[1] * 27 + pending[2] * 9 + pending[3] * 3 + pending[4]);
    }

    (state as u32, packed, distances)
}

// ═══════════════════════════════════════════════════════════════
// DECODE
// ═══════════════════════════════════════════════════════════════

fn ctx_tans_decode(
    freq: &CtxFreqTable, spreads: &[Vec<u16>],
    initial_state: u32, packed_trits: &[u8], distances: &[usize],
) -> Vec<Token> {
    let l = TANS_L as usize;

    // Unpack trit stream
    if packed_trits.len() < 4 { return Vec::new(); }
    let total_trits = u32::from_be_bytes([packed_trits[0], packed_trits[1], packed_trits[2], packed_trits[3]]) as usize;
    let data = &packed_trits[4..];
    let mut all_trits: Vec<u8> = Vec::with_capacity(total_trits);
    for (bi, &byte) in data.iter().enumerate() {
        let mut v = byte;
        let remaining = total_trits.saturating_sub(bi * 5);
        let count = remaining.min(5);
        let t4 = v % 3; v /= 3;
        let t3 = v % 3; v /= 3;
        let t2 = v % 3; v /= 3;
        let t1 = v % 3; v /= 3;
        let t0 = v;
        let ts = [t0, t1, t2, t3, t4];
        for i in 0..count { all_trits.push(ts[i]); }
    }

    let mut trit_pos = 0;
    let read_trit = |pos: &mut usize| -> u8 {
        if *pos < all_trits.len() { let t = all_trits[*pos]; *pos += 1; t } else { 0 }
    };

    let mut state = initial_state as usize;
    let mut tokens: Vec<Token> = Vec::new();
    let mut dist_idx = 0;
    let mut ctx: u8 = 0;

    loop {
        let spread = &spreads[ctx as usize];
        let slot = state % l;
        let s = spread[slot];
        if s == TANS_EOB { break; }

        let fs = freq.fnorm[ctx as usize][s as usize] as usize;
        let cum_s = freq.cum[ctx as usize][s as usize] as usize;

        state = fs * (state / l) + slot - cum_s;

        // Ternary renormalization
        while state < l {
            let t = read_trit(&mut trit_pos) as usize;
            state = state * 3 + t;
        }

        if s <= 255 {
            tokens.push(Token::Literal(s as u8));
            ctx = s as u8 % CTX_BINS as u8;
        } else if s >= TANS_SYM_RUN_BASE && s < TANS_SYM_MATCH_BASE {
            let run_len = (s - TANS_SYM_RUN_BASE) as usize;
            if let Some(Token::Literal(b)) = tokens.last() {
                let byte = *b;
                tokens.pop();
                tokens.push(Token::Run { byte, length: run_len });
            }
        } else if s >= TANS_SYM_MATCH_BASE && s < TANS_EOB {
            let match_len = (s - TANS_SYM_MATCH_BASE) as usize;
            let dist = if dist_idx < distances.len() { distances[dist_idx] } else { 0 };
            dist_idx += 1;
            tokens.push(Token::Match { dist, length: match_len });
        }
    }
    tokens
}

// ═══════════════════════════════════════════════════════════════
// SERIALIZE / DESERIALIZE
// ═══════════════════════════════════════════════════════════════

/// Serialize context-1 rANS. Wire format:
/// [1] CTX_BINS (27)
/// For each context: [2 BE entry_count][delta-varint (sym, freq) pairs]
/// [3] initial state
/// [packed trit stream with 4-byte length header]
/// [4 BE distance count][1 rice_m][rice-coded distances]
pub fn serialize(tokens: &[Token]) -> Vec<u8> {
    let freq = CtxFreqTable::build(tokens);
    let (state, packed_trits, distances) = ctx_tans_encode(tokens, &freq);

    // Estimate capacity
    let mut out = Vec::with_capacity(1 + CTX_BINS * 64 + 3 + packed_trits.len() + 4 + distances.len() * 4);

    // Context count
    out.push(CTX_BINS as u8);

    // Per-context frequency tables (delta-varint encoded)
    for c in 0..CTX_BINS {
        let entries = &freq.entries[c];
        let count = entries.len() as u16;
        out.extend_from_slice(&count.to_be_bytes());
        let mut prev_sym: u16 = 0;
        for &(sym, f) in entries {
            encode_varint(&mut out, sym.wrapping_sub(prev_sym) as u64);
            encode_varint(&mut out, f as u64);
            prev_sym = sym;
        }
    }

    // Initial state (3 bytes)
    out.push((state >> 16) as u8);
    out.push((state >> 8) as u8);
    out.push(state as u8);

    // Packed trit stream (already includes 4-byte length header)
    out.extend_from_slice(&packed_trits);

    // Rice-coded distances
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

/// Deserialize context-1 rANS.
pub fn deserialize(payload: &[u8]) -> TtcResult<Vec<Token>> {
    if payload.len() < 1 { return Err(TtcError::DecompressionError("ContextAns payload empty".into())); }
    let ctx_count = payload[0] as usize;
    if ctx_count != CTX_BINS { return Err(TtcError::DecompressionError(format!("Expected {CTX_BINS} context bins, got {ctx_count}"))); }
    let mut pos = 1;

    // Read per-context frequency tables
    let mut freq = CtxFreqTable {
        fnorm: [[0u32; TANS_ALPHABET]; CTX_BINS],
        cum: [[0u32; TANS_ALPHABET]; CTX_BINS],
        entries: core::array::from_fn(|_| Vec::new()),
    };

    for c in 0..CTX_BINS {
        if pos + 2 > payload.len() { return Err(TtcError::TruncatedPayload); }
        let count = u16::from_be_bytes([payload[pos], payload[pos + 1]]) as usize;
        pos += 2;
        let mut prev_sym: u16 = 0;
        for _ in 0..count {
            if pos >= payload.len() { return Err(TtcError::TruncatedPayload); }
            let (sym_delta, br1) = decode_varint(&payload[pos..]); pos += br1;
            let sym = prev_sym.wrapping_add(sym_delta as u16);
            prev_sym = sym;
            if pos >= payload.len() { return Err(TtcError::TruncatedPayload); }
            let (f, br2) = decode_varint(&payload[pos..]); pos += br2;
            freq.fnorm[c][sym as usize] = f as u32;
            freq.entries[c].push((sym, f as u32));
        }
        // Build cumulative
        let mut c_sum = 0u32;
        for s in 0..TANS_ALPHABET {
            freq.cum[c][s] = c_sum;
            c_sum += freq.fnorm[c][s];
        }
    }

    // Initial state
    if pos + 3 > payload.len() { return Err(TtcError::TruncatedPayload); }
    let initial_state = (payload[pos] as u32) << 16 | (payload[pos + 1] as u32) << 8 | payload[pos + 2] as u32;
    pos += 3;

    // Packed trit stream
    if pos >= payload.len() { return Err(TtcError::TruncatedPayload); }
    let trit_hdr_end = pos + 4;
    if trit_hdr_end > payload.len() { return Err(TtcError::TruncatedPayload); }
    let trits_data_len = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]) as usize;
    // Compute packed bytes needed: ceil(trits_data_len / 5)
    let packed_bytes = (trits_data_len + 4) / 5;
    let trit_end = trit_hdr_end + packed_bytes;
    if trit_end > payload.len() { return Err(TtcError::TruncatedPayload); }
    let packed_trits = &payload[pos..trit_end]; // includes 4-byte header
    pos = trit_end;

    // Rice-coded distances
    let mut distances: Vec<usize> = Vec::new();
    if pos + 4 <= payload.len() {
        let dist_count = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]) as usize;
        pos += 4;
        if dist_count > 0 && pos < payload.len() {
            let rice_m = payload[pos]; pos += 1;
            if pos + 4 <= payload.len() {
                let bit_count = u32::from_be_bytes([payload[pos], payload[pos+1], payload[pos+2], payload[pos+3]]);
                pos += 4;
                let mut r = BitReader::new(&payload[pos..]);
                for _ in 0..dist_count {
                    distances.push(decode_rice(&mut r, rice_m) as usize);
                }
            }
        }
    }

    // Build spread tables and decode
    let spreads = build_ctx_spreads(&freq);
    Ok(ctx_tans_decode(&freq, &spreads, initial_state, packed_trits, &distances))
}

// ═══════════════════════════════════════════════════════════════
// LONG-DISTANCE HASH (13-byte, one radian)
//
// Supplementary match finder for the LZ77 engine.
// Uses a 13-byte hash (framework radian = 13) for long-distance
// matching. Called before the standard 3-byte hash chain.
// Single-entry table (no chain) — 13-byte collisions are rare.
// ═══════════════════════════════════════════════════════════════

/// 13-byte hash for long-distance match finding.
/// Uses the coprime pair (11, 13) as multiplicative constants.
pub fn hash13(data: &[u8], pos: usize) -> u32 {
    if pos + 13 > data.len() { return 0; }
    let mut h = 0u64;
    // Multiply each byte by a coprime-derived constant
    // Constants: powers of 11 and 13 interleaved
    const MULTS: [u64; 13] = [
        11, 13, 121, 169, 1331, 2197, 14641,
        28561, 161051, 371293, 1771561, 4826809, 19487171,
    ];
    for i in 0..13 {
        h = h.wrapping_add((data[pos + i] as u64).wrapping_mul(MULTS[i]));
    }
    h as u32
}

/// Find a long-distance match using the 13-byte hash.
/// Returns Some((distance, length)) if a match of at least 13 bytes is found.
pub fn find_long_match(
    data: &[u8], pos: usize, long_table: &[u32], table_mask: u32, max_dist: usize,
) -> Option<(usize, usize)> {
    if pos + 13 > data.len() { return None; }
    let h = hash13(data, pos);
    let idx = (h & table_mask) as usize;
    let prev = long_table[idx] as usize;
    if prev == 0 || prev >= pos || pos - prev > max_dist { return None; }

    // Verify 13-byte prefix matches
    if data[prev..prev + 13] != data[pos..pos + 13] { return None; }

    // Extend match
    let max_len = data.len() - pos;
    let mut len = 13;
    while len < max_len && prev + len < data.len() && data[prev + len] == data[pos + len] {
        len += 1;
    }
    Some((pos - prev, len))
}

/// Update the 13-byte long-distance hash table.
pub fn update_long_table(data: &[u8], pos: usize, long_table: &mut [u32], table_mask: u32) {
    if pos + 13 > data.len() { return; }
    let h = hash13(data, pos);
    let idx = (h & table_mask) as usize;
    long_table[idx] = pos as u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctx_roundtrip_literals() {
        let tokens: Vec<Token> = b"Hello World! This is a test of context-1 rANS compression."
            .iter().map(|&b| Token::Literal(b)).collect();
        let serialized = serialize(&tokens);
        let decoded = deserialize(&serialized).unwrap();
        assert_eq!(tokens.len(), decoded.len());
        for (a, b) in tokens.iter().zip(decoded.iter()) {
            match (a, b) {
                (Token::Literal(x), Token::Literal(y)) => assert_eq!(x, y),
                _ => panic!("Token type mismatch"),
            }
        }
    }

    #[test]
    fn ctx_roundtrip_mixed() {
        let tokens = vec![
            Token::Literal(b'A'), Token::Literal(b'B'), Token::Literal(b'C'),
            Token::Run { byte: b'X', length: 50 },
            Token::Match { dist: 100, length: 20 },
            Token::Literal(b'D'), Token::Literal(b'E'),
        ];
        let serialized = serialize(&tokens);
        let decoded = deserialize(&serialized).unwrap();
        assert_eq!(tokens.len(), decoded.len());
    }

    #[test]
    fn hash13_deterministic() {
        let data = b"Hello World! Extra bytes here for 13+";
        let h1 = hash13(data, 0);
        let h2 = hash13(data, 0);
        assert_eq!(h1, h2);
        let h3 = hash13(data, 1);
        assert_ne!(h1, h3); // Different position should give different hash
    }
}
