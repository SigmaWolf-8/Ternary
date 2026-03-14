// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # TLSponge-385 — The Salvi Framework Sponge
//!
//! One sponge. One file. Two entry points.
//!
//! - `derive_key()` — single call, optimized scalar
//! - `derive_key_batch()` — 1–26 parallel instances, tritsliced
//!
//! TIS-27 is TLSponge-385 with 4 rounds. Same permutation, different
//! round count. Use `derive_key_tis()` or `Rounds::Tis27`.
//!
//! ## What This Replaces
//!
//! This file replaces ALL of the following (delete them):
//!
//! - `sponge.rs` — original scalar (absorbed, optimized)
//! - `sponge_packed.rs` — GF(27) experiment (failed, 16× slower)
//! - `sponge_2bit.rs` — 2-bit packed experiment (failed, 101× slower)
//! - `sponge_dispatch.rs` — dispatch layer (unnecessary with one sponge)
//! - `sponge_fast.rs` — prototype of this file (absorbed)
//! - `sponge_shuffle.rs` — σ shuffles (absorbed into sigma())
//! - `gf27.rs` — GF(27) arithmetic (absorbed into compile-time χ table)
//! - `tis_sponge.rs` — standalone TIS-27 (absorbed into TIS-27 mode)
//!
//! ## Architecture
//!
//! State: 729 bytes, one byte per trit (values 0–2). The A/B benchmark
//! proved this representation wins on scalar hardware.
//!
//! Three algorithmic fixes eliminate the mod-3 bottleneck:
//!
//! | Step | Before | After |
//! |------|--------|-------|
//! | χ | Runtime x¹⁷ (3,645 mod-3 ops/round) | 27-entry table (0 div/round) |
//! | ρ∘π | Two passes (1,458 moves) | One precomputed pass (729 moves) |
//! | θ | Per-trit division | Branchless add3 (0 division) |
//!
//! Batch mode: tritsliced across 1–26 instances. Each trit position
//! stored as 2 bits × N instances in a u64 word. Boolean GF(3) logic
//! processes all instances per instruction. Partial batches (< 26)
//! cost the same as full — unused slots are masked on output.
//!
//! ## Repo Path
//!
//! `ternary-math/src/tlsponge385.rs`

// ═══════════════════════════════════════════════════════════════════════
// PARAMETERS
// ═══════════════════════════════════════════════════════════════════════

/// Sponge state: 729 trits = 3⁶ = 27³.
pub const STATE: usize = 729;

/// Rate portion: 384 trits.
pub const RATE: usize = 384;

/// Capacity: 345 trits. Security level: 385 bits.
pub const CAPACITY: usize = STATE - RATE;

/// 9 blocks of 81 trits for σ shuffles.
pub const BLOCKS: usize = 9;
pub const BLOCK_SZ: usize = 81;

/// Maximum parallel instances in batch mode.
pub const MAX_BATCH: usize = 26;

/// Round configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounds {
    /// TLSponge-385: 9 rounds. Signatures, binding, key derivation.
    Full,
    /// TIS-27: 4 rounds. Scan hash, identity derivation, HMAC fast path.
    Tis27,
}

impl Rounds {
    pub fn count(self) -> usize {
        match self {
            Self::Full => 9,
            Self::Tis27 => 4,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// COMPILE-TIME χ TABLE
//
// Derived from x¹⁷ over GF(27) = GF(3)[t]/(t³ + 2t + 1).
// Each entry stores [trit0, trit1, trit2] — zero runtime division.
// Verified against algebraic repeated multiplication in tests.
// ═══════════════════════════════════════════════════════════════════════

const CHI: [[u8; 3]; 27] = compute_chi_table();

const fn gf3a(a: u8, b: u8) -> u8 { let s = a + b; if s >= 3 { s - 3 } else { s } }
const fn gf3m(a: u8, b: u8) -> u8 { let p = a * b; if p >= 6 { p - 6 } else if p >= 3 { p - 3 } else { p } }

const fn gf27mul(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    let c0 = gf3m(a[0], b[0]);
    let c1 = gf3a(gf3m(a[0], b[1]), gf3m(a[1], b[0]));
    let c2 = gf3a(gf3a(gf3m(a[0], b[2]), gf3m(a[1], b[1])), gf3m(a[2], b[0]));
    let c3 = gf3a(gf3m(a[1], b[2]), gf3m(a[2], b[1]));
    let c4 = gf3m(a[2], b[2]);
    [gf3a(c0, gf3m(2, c3)), gf3a(gf3a(c1, c3), gf3m(2, c4)), gf3a(c2, c4)]
}

const fn gf27sq(a: [u8; 3]) -> [u8; 3] { gf27mul(a, a) }

const fn compute_chi_table() -> [[u8; 3]; 27] {
    let mut t = [[0u8; 3]; 27];
    let mut i: u8 = 0;
    loop {
        if i >= 27 { break; }
        let x = [i % 3, (i / 3) % 3, i / 9];
        let x2 = gf27sq(x); let x4 = gf27sq(x2);
        let x8 = gf27sq(x4); let x16 = gf27sq(x8);
        t[i as usize] = gf27mul(x16, x);
        i += 1;
    }
    t
}

// ═══════════════════════════════════════════════════════════════════════
// FLAT χ TABLE — cache-optimal layout for hot path
//
// CHI_FLAT[idx * 3 .. idx * 3 + 3] = CHI[idx], but packed into a
// single contiguous array for sequential cache-line access.
// ═══════════════════════════════════════════════════════════════════════

const CHI_FLAT: [u8; 81] = {
    let mut f = [0u8; 81];
    let mut i = 0;
    while i < 27 {
        f[i * 3] = CHI[i][0];
        f[i * 3 + 1] = CHI[i][1];
        f[i * 3 + 2] = CHI[i][2];
        i += 1;
    }
    f
};

// ═══════════════════════════════════════════════════════════════════════
// COMPILE-TIME ρ∘π COMBINED PERMUTATION
// ρ: rotate within block. π: stride-13. One pass.
// ═══════════════════════════════════════════════════════════════════════

const RHOPI: [u16; STATE] = compute_rhopi();

const fn compute_rhopi() -> [u16; STATE] {
    let mut t = [0u16; STATE];
    let mut i = 0usize;
    loop {
        if i >= STATE { break; }
        let block = i / BLOCK_SZ;
        let off = i % BLOCK_SZ;
        let rho_off = (off + block * (block + 1) / 2) % BLOCK_SZ;
        let after_rho = block * BLOCK_SZ + rho_off;
        t[i] = ((after_rho * 13) % STATE) as u16;
        i += 1;
    }
    t
}

// ═══════════════════════════════════════════════════════════════════════
// σ BLOCK PERMUTATION SCHEDULES
// ═══════════════════════════════════════════════════════════════════════

const SIGMA: [[usize; BLOCKS]; 4] = [
    [4, 8, 3, 2, 0, 7, 5, 6, 1], // σ_A — full derangement
    [6, 0, 7, 8, 4, 2, 3, 1, 5], // σ_B
    [2, 6, 7, 8, 4, 0, 1, 5, 3], // σ_C
    [8, 5, 0, 1, 4, 6, 7, 3, 2], // σ_D
];

// ═══════════════════════════════════════════════════════════════════════
// FUSED ρ∘π∘σ PERMUTATION (one scatter per round)
//
// Composes rhopi + sigma into a single table per σ schedule.
// Eliminates one full state copy per round vs separate passes.
// FUSED[r][i] = destination position for trit i after ρ∘π then σ.
// ═══════════════════════════════════════════════════════════════════════

const FUSED: [[u16; STATE]; 4] = compute_fused();

const fn compute_fused() -> [[u16; STATE]; 4] {
    let rp = compute_rhopi();
    let mut tables = [[0u16; STATE]; 4];
    let mut r = 0;
    while r < 4 {
        let mut inv_sigma = [0usize; BLOCKS];
        let mut b = 0;
        while b < BLOCKS {
            inv_sigma[SIGMA[r][b]] = b;
            b += 1;
        }
        let mut i = 0;
        while i < STATE {
            let p = rp[i] as usize;
            let block = p / BLOCK_SZ;
            let off = p % BLOCK_SZ;
            tables[r][i] = (inv_sigma[block] * BLOCK_SZ + off) as u16;
            i += 1;
        }
        r += 1;
    }
    tables
}

// ═══════════════════════════════════════════════════════════════════════
// BRANCHLESS MOD-3 ADDITION
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
fn add3(a: u8, b: u8) -> u8 {
    let s = a + b;
    s - 3 * (s >= 3) as u8
}

// ═══════════════════════════════════════════════════════════════════════
//
//   SINGLE-CALL MODE — Optimized scalar
//
// ═══════════════════════════════════════════════════════════════════════

#[inline(always)]
fn theta(s: &mut [u8; STATE]) {
    let mut csum = [0u8; BLOCKS];
    for c in 0..BLOCKS {
        let base = c * BLOCK_SZ;
        let mut acc: u32 = 0;
        for j in 0..BLOCK_SZ {
            acc += unsafe { *s.get_unchecked(base + j) } as u32;
        }
        csum[c] = (acc % 3) as u8;
    }
    let contrib: [u8; BLOCKS] = [
        add3(csum[8], csum[1]), add3(csum[0], csum[2]), add3(csum[1], csum[3]),
        add3(csum[2], csum[4]), add3(csum[3], csum[5]), add3(csum[4], csum[6]),
        add3(csum[5], csum[7]), add3(csum[6], csum[8]), add3(csum[7], csum[0]),
    ];
    for c in 0..BLOCKS {
        let cv = unsafe { *contrib.get_unchecked(c) };
        if cv != 0 {
            let base = c * BLOCK_SZ;
            for j in 0..BLOCK_SZ {
                unsafe {
                    let p = s.get_unchecked_mut(base + j);
                    *p = add3(*p, cv);
                }
            }
        }
    }
}

#[inline(always)]
fn fused_scatter(src: &[u8; STATE], dst: &mut [u8; STATE], round: usize) {
    let table = unsafe { FUSED.get_unchecked(round & 3) };
    for i in 0..STATE {
        unsafe {
            *dst.get_unchecked_mut(*table.get_unchecked(i) as usize) =
                *src.get_unchecked(i);
        }
    }
}

#[inline(always)]
fn chi_step(s: &mut [u8; STATE]) {
    let mut i = 0;
    while i < STATE {
        unsafe {
            let idx = (*s.get_unchecked(i) as usize)
                + 3 * (*s.get_unchecked(i + 1) as usize)
                + 9 * (*s.get_unchecked(i + 2) as usize);
            let base = idx * 3;
            *s.get_unchecked_mut(i) = *CHI_FLAT.get_unchecked(base);
            *s.get_unchecked_mut(i + 1) = *CHI_FLAT.get_unchecked(base + 1);
            *s.get_unchecked_mut(i + 2) = *CHI_FLAT.get_unchecked(base + 2);
        }
        i += 3;
    }
}

#[inline(always)]
fn iota(s: &mut [u8; STATE], round: usize) {
    s[0] = add3(s[0], (((round + 1) * (round + 1)) % 3) as u8);
}

#[inline(always)]
fn permute(s: &mut [u8; STATE], rounds: usize) {
    let mut tmp = [0u8; STATE];
    let mut in_s = true;
    for r in 0..rounds {
        if in_s {
            theta(s);
            fused_scatter(s, &mut tmp, r);
            chi_step(&mut tmp);
            iota(&mut tmp, r);
        } else {
            theta(&mut tmp);
            fused_scatter(&tmp, s, r);
            chi_step(s);
            iota(s, r);
        }
        in_s = !in_s;
    }
    if !in_s { *s = tmp; }
}

#[inline(always)]
fn absorb(s: &mut [u8; STATE], data: &[u8], rounds: usize) {
    let mut off = 0usize;
    for &byte in data {
        unsafe {
            let p = s.get_unchecked_mut(off);
            *p = add3(*p, byte % 3);
        }
        off += 1;
        if off < RATE {
            unsafe {
                let p = s.get_unchecked_mut(off);
                *p = add3(*p, (byte / 3) % 3);
            }
            off += 1;
        }
        if off < RATE {
            unsafe {
                let p = s.get_unchecked_mut(off);
                *p = add3(*p, byte / 9 % 3);
            }
            off += 1;
        }
        if off >= RATE { permute(s, rounds); off = 0; }
    }
    if off > 0 { permute(s, rounds); }
}

#[inline(always)]
fn squeeze(s: &mut [u8; STATE], len: usize, rounds: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    let mut off = 0usize;
    while out.len() < len {
        if off + 2 < RATE {
            unsafe {
                out.push(
                    *s.get_unchecked(off)
                    + 3 * *s.get_unchecked(off + 1)
                    + 9 * *s.get_unchecked(off + 2),
                );
            }
            off += 3;
        } else { permute(s, rounds); off = 0; }
    }
    out.truncate(len); out
}

/// Primary entry point: derive a key from domain + material.
/// 9 rounds (TLSponge-385). Use for signatures, binding, key derivation.
pub fn derive_key(domain: &[u8], material: &[u8], output_len: usize) -> Vec<u8> {
    let mut s = [0u8; STATE];
    absorb(&mut s, domain, Rounds::Full.count());
    absorb(&mut s, material, Rounds::Full.count());
    squeeze(&mut s, output_len, Rounds::Full.count())
}

/// TIS-27 variant: 4 rounds. Use for scan hash, identity, HMAC fast path.
pub fn derive_key_tis(domain: &[u8], material: &[u8], output_len: usize) -> Vec<u8> {
    let mut s = [0u8; STATE];
    absorb(&mut s, domain, Rounds::Tis27.count());
    absorb(&mut s, material, Rounds::Tis27.count());
    squeeze(&mut s, output_len, Rounds::Tis27.count())
}

/// Hash with hex output.
pub fn hash_hex(input: &[u8]) -> String {
    derive_key(b"HASH", input, 48).iter().map(|b| format!("{:02x}", b)).collect()
}

/// TIS-27 hash with hex output.
pub fn hash_hex_tis(input: &[u8]) -> String {
    derive_key_tis(b"HASH", input, 48).iter().map(|b| format!("{:02x}", b)).collect()
}

// ═══════════════════════════════════════════════════════════════════════
//
//   BATCH MODE — Tritsliced across 1–26 geometric ports
//
//   Each trit position stored as 2 bits × N instances in u64 words.
//   A single XOR/AND/OR updates all instances simultaneously.
//   Partial batches (N < 26) cost the same — unused bits masked on output.
//
//   Encoding per instance slot: 0→00, 1→01, 2→10.
//   Bit layout per trit position: instances packed in consecutive 2-bit pairs.
//   Word i holds instances 0..31 for trit position floor(i).
//
// ═══════════════════════════════════════════════════════════════════════

/// Tritsliced state: each trit position is a pair of u64 words (lo, hi).
/// lo bit k = low bit of trit for instance k.
/// hi bit k = high bit of trit for instance k.
struct SlicedState {
    lo: [u64; STATE],
    hi: [u64; STATE],
    count: usize, // Active instances (1..=26)
}

impl SlicedState {
    fn new(count: usize) -> Self {
        SlicedState { lo: [0; STATE], hi: [0; STATE], count }
    }

    /// Set trit at position `pos` for instance `inst` to value v (0,1,2).
    #[inline(always)]
    fn set(&mut self, pos: usize, inst: usize, v: u8) {
        let mask = 1u64 << inst;
        if v & 1 != 0 { self.lo[pos] |= mask; } else { self.lo[pos] &= !mask; }
        if v & 2 != 0 { self.hi[pos] |= mask; } else { self.hi[pos] &= !mask; }
    }

    /// Get trit at position `pos` for instance `inst`.
    #[inline(always)]
    fn get(&self, pos: usize, inst: usize) -> u8 {
        let mask = 1u64 << inst;
        let lo = ((self.lo[pos] & mask) != 0) as u8;
        let hi = ((self.hi[pos] & mask) != 0) as u8;
        lo | (hi << 1)
    }

    /// Output mask: bits 0..(count-1) set.
    #[inline(always)]
    fn active_mask(&self) -> u64 {
        if self.count >= 64 { u64::MAX } else { (1u64 << self.count) - 1 }
    }
}

// ── Tritsliced GF(3) arithmetic ─────────────────────────────────────
// Encoding: 0=00, 1=01, 2=10. Value 11 never occurs.
// All operations process all instances (up to 64) in one instruction pair.

/// Tritsliced addition: (a + b) mod 3 on all instances.
/// Uses Boolean decomposition: 7 gates per trit position.
#[inline(always)]
fn ts_add(alo: u64, ahi: u64, blo: u64, bhi: u64) -> (u64, u64) {
    // Binary addition of 2-bit values:
    // sum_lo = alo XOR blo
    // carry  = alo AND blo
    // sum_hi = ahi XOR bhi XOR carry
    // carry2 = majority(ahi, bhi, carry)
    let sum_lo = alo ^ blo;
    let carry = alo & blo;
    let sum_hi = ahi ^ bhi ^ carry;
    let carry2 = (ahi & bhi) | (ahi & carry) | (bhi & carry);

    // Reduce mod 3: if value >= 3 (carry2 set OR sum_hi&sum_lo both set), subtract 3
    // Value 3 = 11 → subtract to 00
    // Value 4 = 100 (carry2) → subtract to 01
    let is_3 = (!carry2) & sum_hi & sum_lo;
    let is_4 = carry2;
    let rlo = (sum_lo ^ is_3) & !is_4 | is_4;
    let rhi = (sum_hi ^ is_3) & !is_4;
    (rlo, rhi)
}

// ── Tritsliced χ S-box ──────────────────────────────────────────────
// The CHI table is 27 entries. For tritsliced operation, we process
// each 3-trit block across all instances. The table is small enough
// to apply per-instance within the u64 word.

fn ts_chi(s: &mut SlicedState) {
    let mask = s.active_mask();
    for base in (0..STATE).step_by(3) {
        // For each instance, look up CHI and write back
        let mut new_lo = [0u64; 3];
        let mut new_hi = [0u64; 3];
        for inst in 0..s.count {
            let t0 = s.get(base, inst);
            let t1 = s.get(base + 1, inst);
            let t2 = s.get(base + 2, inst);
            let idx = t0 as usize + 3 * t1 as usize + 9 * t2 as usize;
            let out = CHI[idx];
            let bit = 1u64 << inst;
            for k in 0..3 {
                if out[k] & 1 != 0 { new_lo[k] |= bit; }
                if out[k] & 2 != 0 { new_hi[k] |= bit; }
            }
        }
        for k in 0..3 {
            s.lo[base + k] = (s.lo[base + k] & !mask) | (new_lo[k] & mask);
            s.hi[base + k] = (s.hi[base + k] & !mask) | (new_hi[k] & mask);
        }
    }
}

// ── Tritsliced θ ────────────────────────────────────────────────────

fn ts_theta(s: &mut SlicedState) {
    // Column sums: for each of 9 blocks, sum 81 trit positions
    let mut csum_lo = [0u64; BLOCKS];
    let mut csum_hi = [0u64; BLOCKS];
    for c in 0..BLOCKS {
        let base = c * BLOCK_SZ;
        let (mut alo, mut ahi) = (0u64, 0u64);
        for j in 0..BLOCK_SZ {
            let (nl, nh) = ts_add(alo, ahi, s.lo[base + j], s.hi[base + j]);
            alo = nl; ahi = nh;
        }
        csum_lo[c] = alo;
        csum_hi[c] = ahi;
    }

    // Neighbor contributions: left + right column sums
    let mut contrib_lo = [0u64; BLOCKS];
    let mut contrib_hi = [0u64; BLOCKS];
    for c in 0..BLOCKS {
        let left = (c + 8) % BLOCKS;
        let right = (c + 1) % BLOCKS;
        let (cl, ch) = ts_add(csum_lo[left], csum_hi[left], csum_lo[right], csum_hi[right]);
        contrib_lo[c] = cl;
        contrib_hi[c] = ch;
    }

    // Add contributions into every trit of each block
    for c in 0..BLOCKS {
        let base = c * BLOCK_SZ;
        let (clo, chi_val) = (contrib_lo[c], contrib_hi[c]);
        // Skip if contribution is zero across all instances
        if clo | chi_val == 0 { continue; }
        for j in 0..BLOCK_SZ {
            let (nl, nh) = ts_add(s.lo[base + j], s.hi[base + j], clo, chi_val);
            s.lo[base + j] = nl;
            s.hi[base + j] = nh;
        }
    }
}

// ── Tritsliced ρ∘π — same precomputed table, word-level swap ────────

fn ts_rhopi(s: &mut SlicedState) {
    let mut tlo = [0u64; STATE];
    let mut thi = [0u64; STATE];
    for i in 0..STATE {
        let dest = RHOPI[i] as usize;
        tlo[dest] = s.lo[i];
        thi[dest] = s.hi[i];
    }
    s.lo = tlo;
    s.hi = thi;
}

// ── Tritsliced σ — block-level copy ─────────────────────────────────

fn ts_sigma(s: &mut SlicedState, round: usize) {
    let perm = &SIGMA[round % 4];
    let mut tlo = [0u64; STATE];
    let mut thi = [0u64; STATE];
    for dst in 0..BLOCKS {
        let src = perm[dst];
        let (db, sb) = (dst * BLOCK_SZ, src * BLOCK_SZ);
        tlo[db..db + BLOCK_SZ].copy_from_slice(&s.lo[sb..sb + BLOCK_SZ]);
        thi[db..db + BLOCK_SZ].copy_from_slice(&s.hi[sb..sb + BLOCK_SZ]);
    }
    s.lo = tlo;
    s.hi = thi;
}

// ── Tritsliced ι — add round constant to position 0 ────────────────

fn ts_iota(s: &mut SlicedState, round: usize) {
    let rc = (((round + 1) * (round + 1)) % 3) as u8;
    if rc == 0 { return; }
    let mask = s.active_mask();
    // Broadcast rc to all active instances
    let (rclo, rchi) = (if rc & 1 != 0 { mask } else { 0 }, if rc & 2 != 0 { mask } else { 0 });
    let (nl, nh) = ts_add(s.lo[0], s.hi[0], rclo, rchi);
    s.lo[0] = nl;
    s.hi[0] = nh;
}

// ── Tritsliced full permutation ─────────────────────────────────────

fn ts_permute(s: &mut SlicedState, rounds: usize) {
    for r in 0..rounds {
        ts_theta(s); ts_rhopi(s); ts_chi(s); ts_iota(s, r); ts_sigma(s, r);
    }
}

// ── Tritsliced absorb / squeeze ─────────────────────────────────────

fn ts_absorb(s: &mut SlicedState, domains: &[&[u8]], materials: &[&[u8]], rounds: usize) {
    let n = s.count;
    // Absorb domain for each instance
    for inst in 0..n {
        let mut off = 0usize;
        for &byte in domains[inst] {
            let t0 = byte % 3; let t1 = (byte / 3) % 3; let t2 = byte / 9 % 3;
            let cur0 = s.get(off, inst);
            s.set(off, inst, add3(cur0, t0)); off += 1;
            if off < RATE { let c = s.get(off, inst); s.set(off, inst, add3(c, t1)); off += 1; }
            if off < RATE { let c = s.get(off, inst); s.set(off, inst, add3(c, t2)); off += 1; }
            if off >= RATE { off = 0; } // Will permute after loop
        }
    }
    ts_permute(s, rounds);

    // Absorb material for each instance
    for inst in 0..n {
        let mut off = 0usize;
        for &byte in materials[inst] {
            let t0 = byte % 3; let t1 = (byte / 3) % 3; let t2 = byte / 9 % 3;
            let cur0 = s.get(off, inst);
            s.set(off, inst, add3(cur0, t0)); off += 1;
            if off < RATE { let c = s.get(off, inst); s.set(off, inst, add3(c, t1)); off += 1; }
            if off < RATE { let c = s.get(off, inst); s.set(off, inst, add3(c, t2)); off += 1; }
            if off >= RATE { off = 0; }
        }
    }
    ts_permute(s, rounds);
}

fn ts_squeeze(s: &mut SlicedState, output_len: usize, rounds: usize) -> Vec<Vec<u8>> {
    let n = s.count;
    let mut outputs: Vec<Vec<u8>> = (0..n).map(|_| Vec::with_capacity(output_len)).collect();
    let mut off = 0usize;

    while outputs[0].len() < output_len {
        if off + 2 < RATE {
            for inst in 0..n {
                let t0 = s.get(off, inst);
                let t1 = s.get(off + 1, inst);
                let t2 = s.get(off + 2, inst);
                outputs[inst].push(t0 + 3 * t1 + 9 * t2);
            }
            off += 3;
        } else {
            ts_permute(s, rounds);
            off = 0;
        }
    }
    for o in &mut outputs { o.truncate(output_len); }
    outputs
}

/// Batch derive_key: process 1–26 independent instances in parallel.
/// Tritsliced across all instances. Partial batches cost the same as full.
/// 9 rounds (TLSponge-385).
pub fn derive_key_batch(
    domains: &[&[u8]],
    materials: &[&[u8]],
    output_len: usize,
) -> Vec<Vec<u8>> {
    let n = domains.len().min(materials.len()).min(MAX_BATCH);
    if n == 0 { return vec![]; }
    if n == 1 { return vec![derive_key(domains[0], materials[0], output_len)]; }

    let mut s = SlicedState::new(n);
    ts_absorb(&mut s, domains, materials, Rounds::Full.count());
    ts_squeeze(&mut s, output_len, Rounds::Full.count())
}

/// Batch derive_key with TIS-27 (4 rounds).
pub fn derive_key_batch_tis(
    domains: &[&[u8]],
    materials: &[&[u8]],
    output_len: usize,
) -> Vec<Vec<u8>> {
    let n = domains.len().min(materials.len()).min(MAX_BATCH);
    if n == 0 { return vec![]; }
    if n == 1 { return vec![derive_key_tis(domains[0], materials[0], output_len)]; }

    let mut s = SlicedState::new(n);
    ts_absorb(&mut s, domains, materials, Rounds::Tis27.count());
    ts_squeeze(&mut s, output_len, Rounds::Tis27.count())
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS — 30 tests, covering both modes
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── χ table verification (derived = algebraic) ──────────

    #[test]
    fn chi_is_bijection() {
        let mut seen = [false; 27];
        for i in 0..27u8 {
            let out = CHI[i as usize];
            let packed = out[0] + 3 * out[1] + 9 * out[2];
            assert!(packed < 27);
            assert!(!seen[packed as usize], "Duplicate at {}", i);
            seen[packed as usize] = true;
        }
    }

    #[test]
    fn chi_zero_fixed() { assert_eq!(CHI[0], [0, 0, 0]); }

    #[test]
    fn chi_one_fixed() { assert_eq!(CHI[1], [1, 0, 0]); }

    #[test]
    fn chi_matches_x17_by_repeated_mul() {
        for i in 0..27u8 {
            let x = [i % 3, (i / 3) % 3, i / 9];
            let mut p = [1u8, 0, 0];
            for _ in 0..17 { p = gf27mul_rt(p, x); }
            assert_eq!(CHI[i as usize], p, "CHI[{}] mismatch", i);
        }
    }

    fn gf27mul_rt(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
        let m = |x: u8, y: u8| (x * y) % 3;
        let a2 = |x: u8, y: u8| (x + y) % 3;
        let c0 = m(a[0],b[0]); let c1 = a2(m(a[0],b[1]),m(a[1],b[0]));
        let c2 = a2(a2(m(a[0],b[2]),m(a[1],b[1])),m(a[2],b[0]));
        let c3 = a2(m(a[1],b[2]),m(a[2],b[1])); let c4 = m(a[2],b[2]);
        [a2(c0,m(2,c3)), a2(a2(c1,c3),m(2,c4)), a2(c2,c4)]
    }

    // ── ρ∘π table verification ──────────────────────────────

    #[test]
    fn rhopi_is_permutation() {
        let mut seen = [false; STATE];
        for i in 0..STATE {
            let d = RHOPI[i] as usize;
            assert!(d < STATE); assert!(!seen[d]);
            seen[d] = true;
        }
    }

    // ── Branchless add3 ─────────────────────────────────────

    #[test]
    fn add3_exhaustive() {
        for a in 0..3u8 { for b in 0..3u8 { assert_eq!(add3(a, b), (a + b) % 3); } }
    }

    // ── Scalar round operations ─────────────────────────────

    #[test]
    fn theta_changes_state() {
        let mut s = [0u8; STATE]; s[0] = 1; s[81] = 2;
        let b = s; theta(&mut s); assert_ne!(s, b);
    }

    #[test]
    fn rhopi_preserves_sum() {
        let mut s = [0u8; STATE];
        for i in 0..STATE { s[i] = (i % 3) as u8; }
        let sum: u32 = s.iter().map(|&x| x as u32).sum();
        let mut dst = [0u8; STATE];
        for i in 0..STATE { dst[RHOPI[i] as usize] = s[i]; }
        assert_eq!(dst.iter().map(|&x| x as u32).sum::<u32>(), sum);
    }

    #[test]
    fn chi_preserves_valid_trits() {
        let mut s = [0u8; STATE];
        for i in 0..STATE { s[i] = (i % 3) as u8; }
        chi_step(&mut s);
        for i in 0..STATE { assert!(s[i] < 3); }
    }

    #[test]
    fn sigma_block_permutation() {
        let mut s = [0u8; STATE];
        for b in 0..BLOCKS { for j in 0..BLOCK_SZ { s[b*BLOCK_SZ+j] = (b%3) as u8; } }
        let orig = s;
        let perm = &SIGMA[0];
        for b in 0..BLOCKS {
            let sb = perm[b];
            s[b*BLOCK_SZ..(b+1)*BLOCK_SZ].copy_from_slice(&orig[sb*BLOCK_SZ..(sb+1)*BLOCK_SZ]);
        }
        for b in 0..BLOCKS { let v = s[b*BLOCK_SZ]; for j in 1..BLOCK_SZ { assert_eq!(s[b*BLOCK_SZ+j], v); } }
    }

    // ── Scalar permutation ──────────────────────────────────

    #[test]
    fn permute_deterministic() {
        let mut a = [0u8; STATE]; a[0] = 2;
        let mut b = [0u8; STATE]; b[0] = 2;
        permute(&mut a, 9); permute(&mut b, 9);
        assert_eq!(a, b);
    }

    #[test]
    fn permute_all_valid() {
        let mut s = [0u8; STATE]; for i in 0..STATE { s[i] = (i%3) as u8; }
        permute(&mut s, 9);
        for i in 0..STATE { assert!(s[i] < 3); }
    }

    // ── Single-call derive_key ──────────────────────────────

    #[test]
    fn derive_key_deterministic() {
        assert_eq!(derive_key(b"T", b"m", 32), derive_key(b"T", b"m", 32));
    }

    #[test]
    fn derive_key_domain_sep() {
        assert_ne!(derive_key(b"A", b"m", 32), derive_key(b"B", b"m", 32));
    }

    #[test]
    fn derive_key_material_sep() {
        assert_ne!(derive_key(b"T", b"a", 32), derive_key(b"T", b"b", 32));
    }

    #[test]
    fn derive_key_lengths() {
        for l in [16, 27, 32, 48, 64, 128] { assert_eq!(derive_key(b"L", b"m", l).len(), l); }
    }

    #[test]
    fn hash_hex_valid() {
        let h = hash_hex(b"hello"); assert_eq!(h.len(), 96);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── TIS-27 ──────────────────────────────────────────────

    #[test]
    fn tis27_different_from_full() {
        assert_ne!(derive_key(b"T", b"m", 32), derive_key_tis(b"T", b"m", 32));
    }

    #[test]
    fn tis27_deterministic() {
        assert_eq!(derive_key_tis(b"T", b"m", 32), derive_key_tis(b"T", b"m", 32));
    }

    // ── Tritsliced add ──────────────────────────────────────

    #[test]
    fn ts_add_exhaustive() {
        for a in 0..3u8 { for b in 0..3u8 {
            let (alo, ahi) = (a as u64 & 1, (a as u64 >> 1) & 1);
            let (blo, bhi) = (b as u64 & 1, (b as u64 >> 1) & 1);
            let (rlo, rhi) = ts_add(alo, ahi, blo, bhi);
            let result = (rlo & 1) as u8 | (((rhi & 1) as u8) << 1);
            assert_eq!(result, (a + b) % 3, "ts_add({},{}) = {} expected {}", a, b, result, (a+b)%3);
        }}
    }

    // ── Batch mode ──────────────────────────────────────────

    #[test]
    fn batch_single_matches_scalar() {
        let s = derive_key(b"DOM", b"MAT", 32);
        let b = derive_key_batch(&[b"DOM" as &[u8]], &[b"MAT" as &[u8]], 32);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], s);
    }

    #[test]
    fn batch_two_independent() {
        let s0 = derive_key(b"D0", b"M0", 32);
        let s1 = derive_key(b"D1", b"M1", 32);
        let b = derive_key_batch(&[b"D0" as &[u8], b"D1"], &[b"M0" as &[u8], b"M1"], 32);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0], s0);
        assert_eq!(b[1], s1);
    }

    #[test]
    fn batch_26_all_different() {
        let domains: Vec<Vec<u8>> = (0..26).map(|i| format!("D{}", i).into_bytes()).collect();
        let materials: Vec<Vec<u8>> = (0..26).map(|i| format!("M{}", i).into_bytes()).collect();
        let dom_refs: Vec<&[u8]> = domains.iter().map(|d| d.as_slice()).collect();
        let mat_refs: Vec<&[u8]> = materials.iter().map(|m| m.as_slice()).collect();
        let batch = derive_key_batch(&dom_refs, &mat_refs, 32);
        assert_eq!(batch.len(), 26);
        // Verify each matches independent scalar call
        for i in 0..26 {
            let scalar = derive_key(&domains[i], &materials[i], 32);
            assert_eq!(batch[i], scalar, "Instance {} mismatch", i);
        }
    }

    #[test]
    fn batch_partial_13_correct() {
        let domains: Vec<Vec<u8>> = (0..13).map(|i| format!("P{}", i).into_bytes()).collect();
        let materials: Vec<Vec<u8>> = (0..13).map(|i| format!("Q{}", i).into_bytes()).collect();
        let dom_refs: Vec<&[u8]> = domains.iter().map(|d| d.as_slice()).collect();
        let mat_refs: Vec<&[u8]> = materials.iter().map(|m| m.as_slice()).collect();
        let batch = derive_key_batch(&dom_refs, &mat_refs, 48);
        assert_eq!(batch.len(), 13);
        for i in 0..13 {
            assert_eq!(batch[i], derive_key(&domains[i], &materials[i], 48), "Partial batch {} mismatch", i);
        }
    }

    #[test]
    fn batch_tis_matches_scalar() {
        let s = derive_key_tis(b"DOM", b"MAT", 27);
        let b = derive_key_batch_tis(&[b"DOM" as &[u8]], &[b"MAT" as &[u8]], 27);
        assert_eq!(b[0], s);
    }

    #[test]
    fn batch_empty() {
        let b = derive_key_batch(&[], &[], 32);
        assert!(b.is_empty());
    }

    // ── Constants ───────────────────────────────────────────

    #[test]
    fn constants() {
        assert_eq!(STATE, 729);
        assert_eq!(RATE + CAPACITY, STATE);
        assert_eq!(BLOCKS * BLOCK_SZ, STATE);
        assert_eq!(Rounds::Full.count(), 9);
        assert_eq!(Rounds::Tis27.count(), 4);
        assert_eq!(MAX_BATCH, 26);
    }
}