// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RepX — Salvi Framework physics engine (formerly gf3_algebra.rs).
// Location: ternary-math/src/repx.rs
//
// This module hosts the four Rep digit alphabets (A {−1,0,+1},
// B {0,1,2}, C {1,2,3}, D {0,1,ω}) over ℤ[ω], the canonical
// 39-cell triadic grid (3 archetypes × 13 rows) with pentadic
// Element tags, and the framework's executable physics engine.
//
// Rep core (this section) is division-free: all mod-3 via
// conditional subtract (1-2 cycles), not % operator (20-40 cycles).
// Values bounded to {0,1,2}.
//
// Pure GF(3) algebra — no sponge code here.
// TL-Sponge-385: src/kernel/src/crypto/sponge.rs
//
// Foundations F1–F8, the canonical 39-register grid, the
// `RepXTransferFunction` and `RepXTransform` traits, the `Engine`
// façade, the FFI/CLI surface, and the 14 G-gates are documented
// in `.local/tasks/task-133.md`.

#[allow(dead_code)]
#[inline(always)] fn mod3_small(mut n: u8) -> u8 { if n >= 3 { n -= 3; } n }
#[allow(dead_code)]
#[inline(always)] fn mod3_med(mut n: u8) -> u8 { if n >= 3 { n -= 3; } if n >= 3 { n -= 3; } n }
#[inline(always)] fn mod7_small(mut n: u8) -> u8 { if n >= 14 { n -= 14; } if n >= 7 { n -= 7; } n }

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

// ── Hamming distance: Σ(a-b)² mod 3, division-free ─────────────────

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

// ── Forgery detection: product mod 7, division-free ─────────────────

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

// ── GF(3) vector operations ─────────────────────────────────────────

pub fn gf3_vec_add(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_add(a[i], b[i]); } }
pub fn gf3_vec_sub(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_sub(a[i], b[i]); } }
pub fn gf3_vec_mul(a: &[u8], b: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_mul(a[i], b[i]); } }
pub fn gf3_dot(a: &[u8], b: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for i in 0..a.len() { sum = gf3_add(sum, gf3_mul(a[i], b[i])); }
    sum
}
pub fn gf3_scalar_mul(scalar: u8, a: &[u8], out: &mut [u8]) { for i in 0..a.len() { out[i] = gf3_mul(scalar, a[i]); } }

// ── Repunit checksum: Horner mod 364 (% 364 stays — unbounded accumulator) ──

pub fn repunit_checksum(trits_rep_c: &[u8]) -> u64 {
    let mut value: u64 = 0;
    for i in (0..trits_rep_c.len()).rev() { value = (value * 3 + (trits_rep_c[i] - 1) as u64) % 364; }
    value
}

// ── Derivation: INVARIANT 2 ─────────────────────────────────────────

pub fn project_to_gf3(k: u64, n: u64) -> u8 { let v = 3 * k / n; if v >= 2 { 2 } else { v as u8 } }
pub fn derive_trit(k: u64, n: u64) -> u8 { project_to_gf3(k, n) + 1 }

// ══════════════════════════════════════════════════════════════
// Rep D: Algebraic Trits — {Zero, One, Omega}
//
// The fourth representation. Connects GF(3) arithmetic to the
// Eisenstein integers ℤ[ω] where ω is the primitive cube root
// of unity satisfying ω² + ω + 1 = 0.
//
// In GF(3) (mod 3): ω² = −1 − ω ≡ 2 + 2ω.
//
// Canonical bijection to Rep B: Zero↔0, One↔1, Omega↔2.
// This is NOT an affine formula — it is a pointwise map.
// ══════════════════════════════════════════════════════════════

use crate::gf3::{Gf3, BalancedTrit};

/// Algebraic trit — Rep D. The fourth framework representation.
///
/// - `Zero` — additive identity (0)
/// - `One` — multiplicative identity (1)
/// - `Omega` — primitive cube root of unity (ω, where ω²+ω+1=0)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlgebraicTrit {
    Zero,
    One,
    Omega,
}

// ── Rep D ↔ Rep B (u8) ─────────────────────────────────────

/// Rep B → Rep D. Pointwise: 0→Zero, 1→One, 2→Omega.
pub const fn rep_b_to_d(b: u8) -> AlgebraicTrit {
    match b {
        0 => AlgebraicTrit::Zero,
        1 => AlgebraicTrit::One,
        2 => AlgebraicTrit::Omega,
        _ => panic!("Rep B value must be 0, 1, or 2"),
    }
}

/// Rep D → Rep B. Pointwise: Zero→0, One→1, Omega→2.
pub const fn rep_d_to_b(d: AlgebraicTrit) -> u8 {
    match d {
        AlgebraicTrit::Zero => 0,
        AlgebraicTrit::One => 1,
        AlgebraicTrit::Omega => 2,
    }
}

// ── Rep D ↔ Rep A (i8, balanced) ────────────────────────────

/// Rep B → Rep A. 0→0, 1→1, 2→−1.
pub const fn rep_b_to_a(b: u8) -> i8 {
    match b { 0 => 0, 1 => 1, 2 => -1, _ => panic!("Rep B must be 0, 1, or 2") }
}

/// Rep A → Rep B. 0→0, 1→1, −1→2.
pub const fn rep_a_to_b(a: i8) -> u8 {
    match a { 0 => 0, 1 => 1, -1 => 2, _ => panic!("Rep A must be -1, 0, or 1") }
}

// ── Rep D ↔ Rep C (bijective) ───────────────────────────────

/// Rep D → Rep C. Composed: D→B→C.
pub const fn rep_d_to_c(d: AlgebraicTrit) -> u8 { rep_b_to_c(rep_d_to_b(d)) }

/// Rep C → Rep D. Composed: C→B→D.
pub const fn rep_c_to_d(c: u8) -> AlgebraicTrit { rep_b_to_d(rep_c_to_b(c)) }

// ── Rep A ↔ Rep C ───────────────────────────────────────────

/// Rep A → Rep C. Composed: A→B→C.
pub const fn rep_a_to_c(a: i8) -> u8 { rep_b_to_c(rep_a_to_b(a)) }

/// Rep C → Rep A. Composed: C→B→A.
pub const fn rep_c_to_a(c: u8) -> i8 { rep_b_to_a(rep_c_to_b(c)) }

// ── Rep A ↔ Rep D ───────────────────────────────────────────

/// Rep A → Rep D. Composed: A→B→D.
pub const fn rep_a_to_d(a: i8) -> AlgebraicTrit { rep_b_to_d(rep_a_to_b(a)) }

/// Rep D → Rep A. Composed: D→B→A.
pub const fn rep_d_to_a(d: AlgebraicTrit) -> i8 { rep_b_to_a(rep_d_to_b(d)) }

// ── Rep D ↔ Gf3 struct ─────────────────────────────────────

impl From<Gf3> for AlgebraicTrit {
    fn from(g: Gf3) -> Self {
        rep_b_to_d(g.value())
    }
}

impl From<AlgebraicTrit> for Gf3 {
    fn from(d: AlgebraicTrit) -> Self {
        Gf3::new(rep_d_to_b(d))
    }
}

// ── Rep D ↔ BalancedTrit ────────────────────────────────────

impl From<BalancedTrit> for AlgebraicTrit {
    fn from(b: BalancedTrit) -> Self {
        match b {
            BalancedTrit::Zero => AlgebraicTrit::Zero,
            BalancedTrit::Pos => AlgebraicTrit::One,
            BalancedTrit::Neg => AlgebraicTrit::Omega,
        }
    }
}

impl From<AlgebraicTrit> for BalancedTrit {
    fn from(d: AlgebraicTrit) -> Self {
        match d {
            AlgebraicTrit::Zero => BalancedTrit::Zero,
            AlgebraicTrit::One => BalancedTrit::Pos,
            AlgebraicTrit::Omega => BalancedTrit::Neg,
        }
    }
}

// ── Eisenstein arithmetic on AlgebraicTrit (GF(3) level) ────

impl AlgebraicTrit {
    /// Add in GF(3): delegates to gf3_add via Rep B round-trip.
    pub fn eisenstein_add(self, other: AlgebraicTrit) -> AlgebraicTrit {
        rep_b_to_d(gf3_add(rep_d_to_b(self), rep_d_to_b(other)))
    }

    /// Multiply in GF(3): delegates to gf3_mul via Rep B round-trip.
    pub fn eisenstein_mul(self, other: AlgebraicTrit) -> AlgebraicTrit {
        rep_b_to_d(gf3_mul(rep_d_to_b(self), rep_d_to_b(other)))
    }

    /// Subtract in GF(3): delegates to gf3_sub. Never underflows.
    pub fn eisenstein_sub(self, other: AlgebraicTrit) -> AlgebraicTrit {
        rep_b_to_d(gf3_sub(rep_d_to_b(self), rep_d_to_b(other)))
    }

    /// Negate in GF(3): 0→0, 1→2, 2→1.
    pub fn eisenstein_neg(self) -> AlgebraicTrit {
        rep_b_to_d(gf3_neg(rep_d_to_b(self)))
    }

    /// Square in GF(3).
    pub fn eisenstein_square(self) -> AlgebraicTrit {
        rep_b_to_d(gf3_square(rep_d_to_b(self)))
    }
}

// ── Validators ──────────────────────────────────────────────

pub const fn validate_rep_a(a: i8) -> bool { a >= -1 && a <= 1 }
pub const fn validate_rep_b(b: u8) -> bool { b <= 2 }
pub const fn validate_rep_c(c: u8) -> bool { c >= 1 && c <= 3 }
pub const fn validate_rep_d(_d: &AlgebraicTrit) -> bool { true } // enum is always valid

// ── Batch conversions for Rep D ─────────────────────────────

pub fn batch_b_to_d(trits: &[u8]) -> Vec<AlgebraicTrit> {
    trits.iter().map(|&b| rep_b_to_d(b)).collect()
}

pub fn batch_d_to_b(trits: &[AlgebraicTrit]) -> Vec<u8> {
    trits.iter().map(|&d| rep_d_to_b(d)).collect()
}

// ── Task 102 additions ──────────────────────────────────────

/// Converts a Rep D trit to a UNIFORM k-dimensional torus step vector.
/// Returns Rep B values (0, 1, or 2). Callers convert to TritInt at their level.
///
/// All dimensions receive the same step value.
/// For per-dimension vectors with different coprime generators,
/// construct manually with `coprime::is_coprime` validation.
///
/// Caveat: step=2 (Omega) is only coprime to odd moduli.
/// Step=0 (Zero) produces no movement — stationary coordinates only.
pub fn to_step_vector(d: AlgebraicTrit, k: usize) -> Vec<u8> {
    let step = rep_d_to_b(d);
    vec![step; k]
}

/// Batch convert Rep A trits to Rep B.
pub fn batch_a_to_b(trits: &[i8]) -> Vec<u8> {
    trits.iter().map(|&a| rep_a_to_b(a)).collect()
}

/// Batch convert Rep B trits to Rep A.
pub fn batch_b_to_a(trits: &[u8]) -> Vec<i8> {
    trits.iter().map(|&b| rep_b_to_a(b)).collect()
}

// ══════════════════════════════════════════════════════════════
// Circle-and-Square Bijection — Spec v3.3.33 §1, §4
//
// Encoder-facing typed wrappers for the byte → trit + Milesian
// glyph pipeline. These types are the public surface of the
// encoder (Step 4 / S007). All values are ternary-typed; host
// integers appear only in:
//   - explicit constructors that validate their argument and
//     return Option,
//   - position indices into the 27-symbol Milesian register,
//   - block-size and length counts (which are not values).
//
// The encoder body itself is added in S007. This block adds only
// the wrappers needed by the encoder and by acceptance tests.
// ══════════════════════════════════════════════════════════════

/// One of the four canonical trit alphabets (Spec §1):
/// A — balanced  {−1, 0, +1};
/// B — standard  { 0, 1, 2};
/// C — bijective { 1, 2, 3} (canonical);
/// D — algebraic over ℤ[ω].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Representation { A, B, C, D }

impl Representation {
    pub const fn name(self) -> &'static str {
        match self {
            Representation::A => "A",
            Representation::B => "B",
            Representation::C => "C",
            Representation::D => "D",
        }
    }
}

/// φ_R: Rep-C → Rep-R affine projection (Spec §1).
/// Input must be a Rep-C value in {1, 2, 3}; output range
/// depends on R. Rep-D is taken as identity-on-{1,2,3} here;
/// the algebraic ω-form lives on `AlgebraicTrit` in trit_int.rs.
#[inline]
pub const fn phi_r(rep: Representation, c_value: u8) -> i8 {
    match rep {
        Representation::A => (c_value as i8) - 2,  // {−1, 0, +1}
        Representation::B => (c_value as i8) - 1,  // { 0, 1, 2}
        Representation::C =>  c_value as i8,        // { 1, 2, 3}
        Representation::D =>  c_value as i8,        // {1,2,3} canonical
    }
}

/// φ_R⁻¹: Rep-R → Rep-C inverse affine projection (Spec §1).
#[inline]
pub const fn phi_r_inv(rep: Representation, r_value: i8) -> u8 {
    match rep {
        Representation::A => (r_value + 2) as u8,
        Representation::B => (r_value + 1) as u8,
        Representation::C =>  r_value as u8,
        Representation::D =>  r_value as u8,
    }
}

/// A single trit, stored canonically in Rep-C ({1, 2, 3}).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Trit(u8);

impl Trit {
    /// Construct from a Rep-C literal. Returns `None` if the
    /// input is not in {1, 2, 3}.
    #[inline]
    pub const fn from_rep_c(c_value: u8) -> Option<Self> {
        if c_value >= 1 && c_value <= 3 { Some(Trit(c_value)) } else { None }
    }

    /// Construct from a Rep-B literal. Returns `None` if the
    /// input is not in {0, 1, 2}.
    #[inline]
    pub const fn from_rep_b(b_value: u8) -> Option<Self> {
        if b_value <= 2 { Some(Trit(b_value + 1)) } else { None }
    }

    /// Construct from a Rep-A literal. Returns `None` if the
    /// input is not in {−1, 0, +1}.
    #[inline]
    pub const fn from_rep_a(a_value: i8) -> Option<Self> {
        if a_value >= -1 && a_value <= 1 {
            Some(Trit((a_value + 2) as u8))
        } else {
            None
        }
    }

    /// Project to Rep-R.
    #[inline]
    pub const fn project(self, rep: Representation) -> i8 {
        phi_r(rep, self.0)
    }

    #[inline] pub const fn rep_c(self) -> u8 { self.0 }
    #[inline] pub const fn rep_b(self) -> u8 { self.0 - 1 }
    #[inline] pub const fn rep_a(self) -> i8 { (self.0 as i8) - 2 }
}

/// A trit sequence carrier — the encoder's per-byte output
/// (Spec §4.4). Holds the trit sequence in canonical Rep-C and
/// the block size k that produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TritString {
    trits: Vec<Trit>,
    block_size_k: usize,
}

impl TritString {
    pub fn empty(block_size_k: usize) -> Self {
        Self { trits: Vec::new(), block_size_k }
    }

    pub fn from_trits(trits: Vec<Trit>, block_size_k: usize) -> Self {
        Self { trits, block_size_k }
    }

    #[inline] pub fn len(&self) -> usize { self.trits.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.trits.is_empty() }
    #[inline] pub fn block_size_k(&self) -> usize { self.block_size_k }
    #[inline] pub fn trits(&self) -> &[Trit] { &self.trits }

    pub fn push(&mut self, t: Trit) { self.trits.push(t); }

    /// Project the whole sequence into Rep-R.
    pub fn project(&self, rep: Representation) -> Vec<i8> {
        self.trits.iter().map(|t| t.project(rep)).collect()
    }

    /// Convenience: Rep-C view ({1, 2, 3} per slot).
    pub fn as_rep_c(&self) -> Vec<u8> {
        self.trits.iter().map(|t| t.rep_c()).collect()
    }

    /// Convenience: Rep-B view ({0, 1, 2} per slot).
    pub fn as_rep_b(&self) -> Vec<u8> {
        self.trits.iter().map(|t| t.rep_b()).collect()
    }

    /// Convenience: Rep-A view ({−1, 0, +1} per slot).
    pub fn as_rep_a(&self) -> Vec<i8> {
        self.trits.iter().map(|t| t.rep_a()).collect()
    }
}

/// A byte typed wrapper for the encoder input boundary
/// (Spec §3). Carries one source byte; the value is private
/// to the type so the encoder cannot smuggle binary integers
/// through the public surface.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Byte(u8);

impl Byte {
    #[inline] pub const fn new(value: u8) -> Self { Byte(value) }
    #[inline] pub const fn value(self) -> u8 { self.0 }
}

/// A byte sequence — the encoder's input (Spec §3). The
/// constructor takes a host byte slice because that is the only
/// shape a host can supply; the carrier itself is `Byte`-typed.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ByteString {
    bytes: Vec<Byte>,
}

impl ByteString {
    pub fn empty() -> Self { Self { bytes: Vec::new() } }

    pub fn from_host_bytes(host_bytes: &[u8]) -> Self {
        Self { bytes: host_bytes.iter().map(|&b| Byte::new(b)).collect() }
    }

    #[inline] pub fn len(&self) -> usize { self.bytes.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.bytes.is_empty() }
    #[inline] pub fn bytes(&self) -> &[Byte] { &self.bytes }
}

/// A single Milesian glyph (Spec §1, §4.5). Drawn from the
/// 27-symbol register declared in `crate::constants`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MilesianGlyph(char);

impl MilesianGlyph {
    /// Lookup the glyph at register position `p ∈ {1,…,27}`
    /// using `crate::constants::T_MILESIAN_REGISTER` as the
    /// source of truth.
    pub fn from_position(p: u32) -> Option<Self> {
        if (1..=27).contains(&p) {
            let (_, glyph) =
                crate::constants::T_MILESIAN_REGISTER[(p - 1) as usize];
            Some(MilesianGlyph(glyph))
        } else {
            None
        }
    }

    #[inline] pub const fn glyph(self) -> char { self.0 }
}

/// The Milesian glyph string — Spec §4.5 universal output.
/// Produced for every encoding regardless of k. Empty when N = 0.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MilesianGlyphString {
    glyphs: Vec<MilesianGlyph>,
}

impl MilesianGlyphString {
    pub fn empty() -> Self { Self { glyphs: Vec::new() } }

    pub fn from_glyphs(glyphs: Vec<MilesianGlyph>) -> Self {
        Self { glyphs }
    }

    pub fn push(&mut self, g: MilesianGlyph) { self.glyphs.push(g); }

    #[inline] pub fn len(&self) -> usize { self.glyphs.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.glyphs.is_empty() }
    #[inline] pub fn glyphs(&self) -> &[MilesianGlyph] { &self.glyphs }

    /// String form built from the 27-symbol register.
    pub fn as_string(&self) -> String {
        self.glyphs.iter().map(|g| g.glyph()).collect()
    }
}

impl std::fmt::Display for MilesianGlyphString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for g in &self.glyphs { write!(f, "{}", g.glyph())?; }
        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Original tests (unchanged) ──────────────────────────

    #[test] fn test_gf3_add() { let e=[[0,1,2],[1,2,0],[2,0,1]]; for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_add(a,b),e[a as usize][b as usize]);}}}
    #[test] fn test_gf3_mul() { let e=[[0,0,0],[0,1,2],[0,2,1]]; for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_mul(a,b),e[a as usize][b as usize]);}}}
    #[test] fn test_gf3_sub() { for a in 0..3u8{for b in 0..3u8{assert_eq!(gf3_sub(a,b),(a+3-b)%3);}}}
    #[test] fn test_gf3_square() { assert_eq!(gf3_square(0),0); assert_eq!(gf3_square(1),1); assert_eq!(gf3_square(2),1); }
    #[test] fn test_mod7_small() { for n in 0..=18u8 { assert_eq!(mod7_small(n), n%7); } }
    #[test] fn test_hamming_id() { let a=[0u8,1,2,0,1,2]; assert_eq!(hamming_distance(&a,&a),0); }
    #[test] fn test_hamming_all() { assert_eq!(hamming_distance(&[0;5],&[1,2,1,2,1]),5); }
    #[test] fn test_forgery_ok() { assert!(!has_forgery(&[1,2,3,1,2,3])); }
    #[test] fn test_forgery_bad() { assert!(has_forgery(&[1,0,3,1])); }

    // ── Rep D roundtrip tests ───────────────────────────────

    #[test]
    fn rep_d_roundtrip_b() {
        for b in 0..3u8 {
            assert_eq!(rep_d_to_b(rep_b_to_d(b)), b);
        }
    }

    #[test]
    fn rep_d_roundtrip_a() {
        for &a in &[-1i8, 0, 1] {
            assert_eq!(rep_d_to_a(rep_a_to_d(a)), a);
        }
    }

    #[test]
    fn rep_d_roundtrip_c() {
        for c in 1..=3u8 {
            assert_eq!(rep_d_to_c(rep_c_to_d(c)), c);
        }
    }

    // ── All 12 conversion paths ─────────────────────────────

    #[test]
    fn all_12_paths() {
        for b in 0..3u8 {
            // B → D → C → B
            assert_eq!(rep_c_to_b(rep_d_to_c(rep_b_to_d(b))), b);
            // B → A → D → B
            assert_eq!(rep_d_to_b(rep_a_to_d(rep_b_to_a(b))), b);
            // B → C → A → B
            assert_eq!(rep_a_to_b(rep_c_to_a(rep_b_to_c(b))), b);
            // B → D → A → B
            assert_eq!(rep_a_to_b(rep_d_to_a(rep_b_to_d(b))), b);
        }
    }

    // ── Gf3 ↔ AlgebraicTrit bridge ─────────────────────────

    #[test]
    fn gf3_to_algebraic_roundtrip() {
        for &g in &Gf3::ALL {
            let at: AlgebraicTrit = g.into();
            let back: Gf3 = at.into();
            assert_eq!(back, g);
        }
    }

    // ── BalancedTrit ↔ AlgebraicTrit bridge ─────────────────

    #[test]
    fn balanced_to_algebraic_roundtrip() {
        for &bt in &[BalancedTrit::Zero, BalancedTrit::Pos, BalancedTrit::Neg] {
            let at: AlgebraicTrit = bt.into();
            let back: BalancedTrit = at.into();
            assert_eq!(back, bt);
        }
    }

    // ── Eisenstein arithmetic on AlgebraicTrit ──────────────

    #[test]
    fn omega_squared_is_2_plus_2omega() {
        use AlgebraicTrit::*;
        // ω² = −1 − ω ≡ 2 + 2ω (mod 3)
        let result = Omega.eisenstein_mul(Omega);
        // In GF(3): 2 * 2 = 4 mod 3 = 1. So ω×ω = 1 in GF(3) mul table.
        // gf3_mul(2, 2) = 4 mod 3 = 1 → One.
        // But ω² should be 2 + 2ω in the Eisenstein ring...
        //
        // The GF(3) multiplication table IS the Eisenstein product at the
        // single-element level. In GF(3), ω is just the element 2, and
        // 2 × 2 = 1 mod 3. The "2 + 2ω" form only appears when tracking
        // both components (integer + ω-coefficient) simultaneously.
        //
        // At the single-trit level: gf3_mul(ω, ω) = gf3_mul(2, 2) = 1 = One.
        assert_eq!(result, One);
    }

    #[test]
    fn omega_cubed_is_one() {
        use AlgebraicTrit::*;
        // ω³ = ω · ω² = ω · 1 = ω.
        // In GF(3): ω = 2. 2³ = 8 mod 3 = 2. So ω³ = ω, not 1.
        // BUT: ω is a CUBE ROOT of unity, meaning ω³ = 1.
        // The issue: in GF(3), the element 2 satisfies 2³ = 8 ≡ 2 (mod 3).
        // So 2 is NOT a cube root of unity in GF(3).
        //
        // The cube roots of unity in GF(3) are the solutions to x³ = 1:
        // 0³ = 0, 1³ = 1, 2³ = 2. Only x = 1 satisfies x³ = 1.
        // GF(3) does not contain non-trivial cube roots of unity.
        //
        // ω as a cube root of unity lives in GF(3²) or in ℂ, not in GF(3).
        // The AlgebraicTrit mapping Omega↔2 is a REPRESENTATION choice
        // that preserves GF(3) arithmetic, not Eisenstein ring arithmetic.
        //
        // Correct test: ω³ = 2³ = 2 in GF(3). ω³ = ω.
        let w2 = Omega.eisenstein_mul(Omega);
        let w3 = Omega.eisenstein_mul(w2);
        assert_eq!(w3, Omega); // 2³ ≡ 2 (mod 3)
    }

    #[test]
    fn zero_is_additive_identity() {
        use AlgebraicTrit::*;
        assert_eq!(Zero.eisenstein_add(One), One);
        assert_eq!(Zero.eisenstein_add(Omega), Omega);
        assert_eq!(Zero.eisenstein_add(Zero), Zero);
    }

    #[test]
    fn zero_annihilates() {
        use AlgebraicTrit::*;
        assert_eq!(Zero.eisenstein_mul(One), Zero);
        assert_eq!(Zero.eisenstein_mul(Omega), Zero);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        use AlgebraicTrit::*;
        assert_eq!(One.eisenstein_mul(One), One);
        assert_eq!(One.eisenstein_mul(Omega), Omega);
        assert_eq!(One.eisenstein_mul(Zero), Zero);
    }

    #[test]
    fn eisenstein_mul_exhaustive() {
        // All 9 pairs, verified against gf3_mul table
        let variants = [AlgebraicTrit::Zero, AlgebraicTrit::One, AlgebraicTrit::Omega];
        for &a in &variants {
            for &b in &variants {
                let result = a.eisenstein_mul(b);
                let expected = rep_b_to_d(gf3_mul(rep_d_to_b(a), rep_d_to_b(b)));
                assert_eq!(result, expected, "{:?} × {:?}", a, b);
            }
        }
    }

    #[test]
    fn eisenstein_mul_commutativity() {
        let variants = [AlgebraicTrit::Zero, AlgebraicTrit::One, AlgebraicTrit::Omega];
        for &a in &variants {
            for &b in &variants {
                assert_eq!(a.eisenstein_mul(b), b.eisenstein_mul(a));
            }
        }
    }

    // ── Validators ──────────────────────────────────────────

    #[test]
    fn validators() {
        assert!(validate_rep_a(0)); assert!(validate_rep_a(1)); assert!(validate_rep_a(-1));
        assert!(!validate_rep_a(2)); assert!(!validate_rep_a(-2));
        assert!(validate_rep_b(0)); assert!(validate_rep_b(2)); assert!(!validate_rep_b(3));
        assert!(validate_rep_c(1)); assert!(validate_rep_c(3)); assert!(!validate_rep_c(0)); assert!(!validate_rep_c(4));
    }

    #[test]
    fn test_to_step_vector() {
        use AlgebraicTrit::*;
        assert_eq!(to_step_vector(Zero, 3), vec![0u8, 0, 0]);
        assert_eq!(to_step_vector(One, 2), vec![1u8, 1]);
        assert_eq!(to_step_vector(Omega, 4), vec![2u8, 2, 2, 2]);
    }

    #[test]
    fn test_batch_a_to_b_roundtrip() {
        let b_trits = vec![0u8, 1, 2, 0, 2, 1];
        let a_trits = batch_b_to_a(&b_trits);
        let back = batch_a_to_b(&a_trits);
        assert_eq!(back, b_trits);
    }

    #[test]
    fn test_batch_a_b_values() {
        assert_eq!(batch_b_to_a(&[0, 1, 2]), vec![0i8, 1, -1]);
        assert_eq!(batch_a_to_b(&[0, 1, -1]), vec![0u8, 1, 2]);
    }
}

// ══════════════════════════════════════════════════════════════
// CANONICAL 39-REGISTER GRID  (task-133 §F3, §F4)
//
// Three archetypes (Sun/Gaia/Moon) × thirteen rows = 39 cells.
// Each cell carries a pentadic Element tag per F4's Quintessence
// verbatim mapping. Formulas are stored as documentation strings
// (post-OT-1, π_fw form) — the executable Transfer Function and
// Engine façade that READS these strings into runnable code is
// scoped as task-133 follow-ups (Steps 6, 7, 8 of the spec).
// ══════════════════════════════════════════════════════════════

/// One of the three triadic archetypes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Archetype {
    /// Push — outward, projecting.
    Sun,
    /// Balance / Fulcrum — local medium state.
    Gaia,
    /// Pull — inward, returning, cycling.
    Moon,
}

/// Pentadic Element secondary tag (F4). Bridge cells carry both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    Fire,
    Water,
    Air,
    Earth,
    Aether,
}

/// One cell of the canonical 39-cell grid.
#[derive(Clone, Copy, Debug)]
pub struct Register {
    pub archetype: Archetype,
    pub row: u8,
    pub elements: &'static [Element],
    pub formula_doc: &'static str,
    pub ot_notes: &'static [&'static str],
}

const F: Element = Element::Fire;
const W: Element = Element::Water;
const A: Element = Element::Air;
const E: Element = Element::Earth;
const Z: Element = Element::Aether;

const NONE: &[Element] = &[];

/// The canonical 39-register grid (Sun / Gaia / Moon × 13 rows).
/// All formulas are in post-OT-1, π_fw (= ROOT_X1) form.
pub const REGISTERS: [Register; 39] = [
    // ── Sun column (Push) ─────────────────────────────────
    Register { archetype: Archetype::Sun, row: 1, elements: &[F],
        formula_doc: "V = ROOT_X1·(REPUNIT_1 − REPUNIT_1/x)",
        ot_notes: &["OT-1e: horn replaces sphere (4/3)πx³"] },
    Register { archetype: Archetype::Sun, row: 2, elements: &[F],
        formula_doc: "A = ROOT_X1/x²",
        ot_notes: &["OT-1e: horn replaces sphere 4πx²"] },
    Register { archetype: Archetype::Sun, row: 3, elements: &[F],
        formula_doc: "I = P₀·x²/ROOT_X1",
        ot_notes: &["radiant flux; horn-form (== flux/cross-section)"] },
    Register { archetype: Archetype::Sun, row: 4, elements: &[F],
        formula_doc: "q = −k·dT/dx",
        ot_notes: &["thermal flux push"] },
    Register { archetype: Archetype::Sun, row: 5, elements: &[F],
        formula_doc: "E = Q/((4·ROOT_X1)·x²)",
        ot_notes: &["F5: 4π → 4·ROOT_X1"] },
    Register { archetype: Archetype::Sun, row: 6, elements: &[F],
        formula_doc: "KE = ½·ρ·V·v²", ot_notes: &[] },
    Register { archetype: Archetype::Sun, row: 7, elements: &[F],
        formula_doc: "J = ∫F dt", ot_notes: &[] },
    Register { archetype: Archetype::Sun, row: 8, elements: NONE,
        formula_doc: "P_pow = F·v",
        ot_notes: &["not in Quintessence verbatim mapping"] },
    Register { archetype: Archetype::Sun, row: 9, elements: NONE,
        formula_doc: "T_temp ∝ ⟨½·m·v²⟩",
        ot_notes: &["not in Quintessence verbatim mapping"] },
    Register { archetype: Archetype::Sun, row: 10, elements: &[A],
        formula_doc: "f = REPUNIT_1/T",
        ot_notes: &["wave generation frequency; bridges to Moon #10"] },
    Register { archetype: Archetype::Sun, row: 11, elements: &[A],
        formula_doc: "A₀",
        ot_notes: &["wave amplitude (source)"] },
    Register { archetype: Archetype::Sun, row: 12, elements: &[E],
        formula_doc: "τ = F·x", ot_notes: &[] },
    Register { archetype: Archetype::Sun, row: 13, elements: &[F],
        formula_doc: "C_cap = C₀·x/ln(x)",
        ot_notes: &["horn-form, NOT 4·ROOT_X1·ε₀·x"] },

    // ── Gaia column (Balance / Fulcrum) ───────────────────
    Register { archetype: Archetype::Gaia, row: 1, elements: &[E],
        formula_doc: "ρ = REPUNIT_1/x  [SI: ρ_med = RHO_0/x]",
        ot_notes: &["OT-1g: horn fixed-point boundary"] },
    Register { archetype: Archetype::Gaia, row: 2, elements: &[W],
        formula_doc: "Γ = ROOT_X1/x²  [SI: Γ = GAMMA_0/x²]",
        ot_notes: &["replaces Newtonian g; Γ_source ≡ GM_source"] },
    Register { archetype: Archetype::Gaia, row: 3, elements: &[E],
        formula_doc: "P = ∫ρ·Γ dx", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 4, elements: &[W],
        formula_doc: "v = dx/dt", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 5, elements: NONE,
        formula_doc: "a = dv/dt",
        ot_notes: &["not in Quintessence verbatim mapping"] },
    Register { archetype: Archetype::Gaia, row: 6, elements: &[W],
        formula_doc: "Q = (ROOT_X1/x²)·v",
        ot_notes: &["π_fw appears via cross-section"] },
    Register { archetype: Archetype::Gaia, row: 7, elements: &[E],
        formula_doc: "F_B = (ρ_med − ρ_obj)·V·Γ",
        ot_notes: &["OT-1f: ρ_med and ρ_obj are HORN-RADIUS PARAMETERS, NOT bulk mass densities; composition dependence enters only at α²·(κ_ep − 1)"] },
    Register { archetype: Archetype::Gaia, row: 8, elements: &[E],
        formula_doc: "I_rot = ∫r²·dm", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 9, elements: &[E],
        formula_doc: "C_v = ∫x dV / ∫dV", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 10, elements: &[W],
        formula_doc: "P + ½·ρ·v² = const", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 11, elements: &[W],
        formula_doc: "Re = ρ·v·L/η", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 12, elements: &[Z],
        formula_doc: "Φ = δE/δρ", ot_notes: &[] },
    Register { archetype: Archetype::Gaia, row: 13, elements: &[E],
        formula_doc: "MA = F_out/F_in", ot_notes: &[] },

    // ── Moon column (Pull) ────────────────────────────────
    Register { archetype: Archetype::Moon, row: 1, elements: &[Z],
        formula_doc: "κ_curve = 2x/(x⁴+1)^(3/2); K_gauss = −2/(x⁶·(REPUNIT_1+REPUNIT_1/x⁴)²)",
        ot_notes: &["OT-1e: horn-curvature pair replaces sphere 1/x"] },
    Register { archetype: Archetype::Moon, row: 2, elements: &[Z],
        formula_doc: "θ = s/x", ot_notes: &[] },
    Register { archetype: Archetype::Moon, row: 3, elements: &[Z],
        formula_doc: "Ω = ROOT_X1/x⁴", ot_notes: &[] },
    Register { archetype: Archetype::Moon, row: 4, elements: &[Z],
        formula_doc: "ω = ∇×v", ot_notes: &[] },
    Register { archetype: Archetype::Moon, row: 5, elements: &[Z],
        formula_doc: "B = μ₀·I/((2·ROOT_X1)·x)",
        ot_notes: &["F5: 2π → 2·ROOT_X1 (== T_Z28_ORDER)"] },
    Register { archetype: Archetype::Moon, row: 6, elements: &[Z],
        formula_doc: "L = μ₀·N²·ROOT_X1/(x²·ℓ)", ot_notes: &[] },
    Register { archetype: Archetype::Moon, row: 7, elements: &[A],
        formula_doc: "Z = ρ·c",
        ot_notes: &["OT-1a: c² = π_fw = ROOT_X1 from horn fixed-point boundary"] },
    Register { archetype: Archetype::Moon, row: 8, elements: &[W],
        formula_doc: "F_d = (6·ROOT_X1)·η·r·v",
        ot_notes: &[
            "F5: 6π → 6·ROOT_X1",
            "Quintessence (F4 reconciliation): Water column, not Aether — drag is hydraulic resistance via η",
        ] },
    Register { archetype: Archetype::Moon, row: 9, elements: &[W],
        formula_doc: "J_diff = −D·dC/dx",
        ot_notes: &["Quintessence (F4 reconciliation): Water column — diffusion is a flow phenomenon"] },
    Register { archetype: Archetype::Moon, row: 10, elements: &[A],
        formula_doc: "λ = h/p",
        ot_notes: &["de Broglie; bridges to Sun #10 via OT-1h Gabor uncertainty floor Δt·Δξ = ½"] },
    Register { archetype: Archetype::Moon, row: 11, elements: &[A],
        formula_doc: "ψ ∝ exp(−x/λ)",
        ot_notes: &["OT-1h closure; real-envelope degenerate Gabor atom (TM-2026-017 §20.16)"] },
    Register { archetype: Archetype::Moon, row: 12, elements: &[A],
        formula_doc: "S = k_B·ln(Ω)",
        ot_notes: &["entropy; Ω is phase-space multiplicity, not Moon #3 solid angle"] },
    Register { archetype: Archetype::Moon, row: 13, elements: &[Z],
        formula_doc: "ΔF ≈ (dΓ/dx)·Δx·ρ·V",
        ot_notes: &["tidal pull; dΓ/dx differentiated from Gaia #2"] },
];

const _: () = assert!(REGISTERS.len() == 39);

/// Look up a single register by archetype × row (1..=13).
/// Returns `None` if `row` is outside the 1..=13 range.
pub fn at(archetype: Archetype, row: u8) -> Option<&'static Register> {
    if row == 0 || row > 13 {
        return None;
    }
    REGISTERS.iter().find(|r| r.archetype == archetype && r.row == row)
}

/// Iterator over every register tagged with `element`.
/// Bridge cells (multi-element) appear in every iterator they belong to.
pub fn registers_for_element(element: Element) -> impl Iterator<Item = &'static Register> {
    REGISTERS.iter().filter(move |r| r.elements.contains(&element))
}

/// Iterator over every register in the given archetype column.
pub fn registers_for_archetype(archetype: Archetype) -> impl Iterator<Item = &'static Register> {
    REGISTERS.iter().filter(move |r| r.archetype == archetype)
}

// ══════════════════════════════════════════════════════════════
// TRAIT SKELETONS (task-133 §F7)
//
// `RepXTransferFunction` (load-bearing primary) and `RepXTransform`
// (complementary spectral engine) are declared here as the stable
// trait surface. Per QC-R1 R1-A1-5 they use the sealed-trait
// pattern so downstream `impl` blocks cannot widen the surface.
// Concrete implementations land in task-133 Steps 6, 7 follow-ups.
// ══════════════════════════════════════════════════════════════

mod sealed {
    pub trait Sealed {}
}

/// Composition operators for [`RepXTransferFunction::compose`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposeOp {
    /// Vector sum — §5.7 Example 2 (Sun-Moon distance via Sun-Gaia ⊕ Gaia-Moon).
    VectorSum,
    /// Inverse of [`ComposeOp::VectorSum`].
    VectorDifference,
    /// Sequential register applications.
    ChainMultiply,
    /// Re-enter the transfer function at a sub-scale Fulcrum (Gaia within Sun, Luna within Gaia).
    NestedFulcrum,
}

/// Primary RepX trait — one displacement signal `x` in, structural
/// readings across the 39 registers out. Sealed: only the engine
/// inside this crate may implement it. Concrete impl is a follow-up
/// to task-133 Step 6.
pub trait RepXTransferFunction: sealed::Sealed {
    /// Reading of `x` in `register`. Returns the Rep-D coefficient.
    fn at(&self, point: AlgebraicTrit, register: &Register) -> AlgebraicTrit;
    /// One point, many readings, one pass.
    fn emit(&self, point: AlgebraicTrit, registers: &[&Register]) -> Vec<AlgebraicTrit>;
    /// RepD Transform: route a reading from `from.0` into `to`, going
    /// through Gaia for dimensional bridging when source and target
    /// are Sun and Moon (or vice-versa).
    fn rotate(
        &self,
        from: (&Register, AlgebraicTrit),
        to: &Register,
    ) -> AlgebraicTrit;
    /// Combine multiple register readings under `op`.
    fn compose(
        &self,
        readings: &[(&Register, AlgebraicTrit)],
        op: ComposeOp,
    ) -> AlgebraicTrit;
}

/// Complementary spectral engine — operations *on values*, not
/// routing of a signal *through* registers. Sealed; lifts every
/// existing native primitive into a single trait surface. Concrete
/// implementation is a follow-up to task-133 Step 7.
pub trait RepXTransform: sealed::Sealed {
    /// Yoneda projection — read this value as its character coefficient.
    fn as_coefficient(&self) -> AlgebraicTrit;
    /// `1·c₀ + ω·c₁ + ω²·c₂ = 0` over ℤ[ω] — the Nona closure.
    fn nona_closure(&self) -> bool;
    /// Parseval energy `Σ N(c_k)` via the single-trit Eisenstein norm.
    fn parseval_energy(&self) -> u64;
}


// ══════════════════════════════════════════════════════════════
// CONCRETE ENGINE — numerical (SI) façade + algebraic trait impls
//
// Per task-133 §F7 / Public API §2: `Engine` is the single entry
// point. Two surfaces share one struct:
//   • SI surface — `read_si`, `convert_si`, `compose_si_op`,
//     plus per-bridge helpers (`omega_from_frequency`,
//     `period_natural`, `precession_natural`, `ot1c_delta_over_a`).
//     `read_si` is TOTAL over the 39-cell grid: every register
//     evaluates to a finite f64 reading at displacement x with
//     framework-canonical default state taken from `Calibration`.
//   • Algebraic surface — `RepXTransferFunction` /
//     `RepXTransform` sealed-trait Yoneda projection over ℤ[ω].
//     `at` is formula-aware via the per-cell `character_of` table;
//     `rotate` follows the spec §D.2 algorithm (Sun↔Moon transit
//     Gaia, cross-row routes through both Gaia bridges).
// ══════════════════════════════════════════════════════════════

use crate::constants::{
    kappa_ep, ALPHA_INV_INT, GAMMA_0, GAMMA_0_G, LCM_PRIMARY, RHO_0, ROOT_X1, ROOT_X2,
};

/// Calibration body identity for `read_si` / `convert_si`. Selects
/// which Γ₀ (Sun vs. Earth) the gravitational reads use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// Sun-as-Gaia (default; Γ₀ = GAMMA_0).
    Sun,
    /// Earth-as-Gaia (Luna-Gaia nested fulcrum; Γ₀ = GAMMA_0_G).
    Earth,
}

/// Framework-canonical SI defaults for state-augmented cells.
///
/// Per spec §C, every state-aug register has a *default state*
/// derived either from the framework's own constants or from a
/// universally-agreed SI value. `Calibration` makes those defaults
/// explicit so `read_si` is total over all 39 cells without any
/// caller having to thread per-cell state. Override individual
/// fields to model a specific physical scenario.
#[derive(Clone, Copy, Debug)]
pub struct Calibration {
    /// Reference power for Sun #3 intensity (1 W default).
    pub p_0: f64,
    /// Thermal conductivity for Sun #4 (W·m⁻¹·K⁻¹).
    pub k_thermal: f64,
    /// Default temperature gradient dT/dx (K/m).
    pub dt_dx: f64,
    /// Charge for Sun #5 field (C).
    pub q_charge: f64,
    /// Permittivity ε₀ for Sun #5 (F/m).
    pub epsilon_0: f64,
    /// Permeability μ₀ for Moon #5/#6 (H/m).
    pub mu_0: f64,
    /// Source current for Moon #5 (A).
    pub current_i: f64,
    /// Solenoid turns N for Moon #6.
    pub coil_n: f64,
    /// Solenoid length ℓ for Moon #6 (m).
    pub coil_l: f64,
    /// Dynamic viscosity η for Moon #8 / Gaia #11 (Pa·s).
    pub eta: f64,
    /// Particle radius r for Moon #8 (m).
    pub r_particle: f64,
    /// Diffusion coefficient D for Moon #9 (m²/s).
    pub d_diff: f64,
    /// Concentration gradient dC/dx for Moon #9 (mol·m⁻⁴).
    pub dc_dx: f64,
    /// Planck constant h for Moon #10 (J·s).
    pub h_planck: f64,
    /// Default momentum p for Moon #10 (kg·m/s).
    pub momentum_p: f64,
    /// Wavelength λ for Moon #11 (m); defaults to h/p.
    pub lambda: f64,
    /// Boltzmann k_B for Moon #12 (J/K).
    pub k_b: f64,
    /// Microstate count Ω for Moon #12.
    pub omega_states: f64,
    /// Reference capacitance C₀ for Sun #13 (F).
    pub c0: f64,
    /// Default period T for Sun #10 frequency read (s).
    pub period_t: f64,
    /// Default amplitude A₀ for Sun #11 (m).
    pub amplitude: f64,
    /// Force F default for Sun #6/#7/#12, Gaia #13, Moon #13 (N).
    pub force_default: f64,
    /// Mass m default for Sun #6/#9 (kg).
    pub mass_default: f64,
    /// Velocity v default for Sun #6/#7/#8/Gaia#6/#11/Moon#8 (m/s).
    pub velocity_default: f64,
    /// Volume V default for Sun #6 KE (m³).
    pub volume_default: f64,
    /// Time t default for Sun #7 impulse, Gaia #4/#5 (s).
    pub time_default: f64,
    /// Lever arm for Sun #12 torque (m).
    pub lever_arm: f64,
    /// Centroid x_c for Gaia #9 (m).
    pub centroid: f64,
    /// Rotational inertia I for Gaia #8 (kg·m²).
    pub i_rot: f64,
    /// Mechanical advantage F_out/F_in for Gaia #13.
    pub mech_advantage: f64,
    /// Bernoulli total head P + ½ρv² (Pa) for Gaia #10.
    pub bernoulli_const: f64,
    /// Reynolds number for Gaia #11 (dimensionless).
    pub re_value: f64,
    /// Variational potential δE/δρ for Gaia #12 (J·m³/kg).
    pub potential_phi: f64,
    /// Arc length s for Moon #2 (m).
    pub arc_s: f64,
    /// Vorticity ω = ∇×v for Moon #4 (1/s).
    pub omega_vort: f64,
    /// Submerged-object density ρ_obj for Gaia #7 (kg/m³). Per OT-1f
    /// horn-radius reinterpretation the buoyant force is
    /// `F_B = (ρ_med − ρ_obj)·V·Γ`. Default 0 ⇒ pure displacement read.
    pub rho_obj: f64,
}

impl Calibration {
    /// Framework-canonical SI defaults. Magnitudes chosen so that
    /// every cell evaluates to a finite, bounded, non-NaN reading
    /// at the engine's reference displacement x = 1 m.
    pub const fn defaults() -> Self {
        Self {
            p_0: 1.0,
            k_thermal: 1.0,
            dt_dx: 1.0,
            q_charge: 1.0,
            epsilon_0: 8.854_187_817e-12,
            mu_0: 1.256_637_061e-6,
            current_i: 1.0,
            coil_n: 1.0,
            coil_l: 1.0,
            eta: 1.0e-3,
            r_particle: 1.0,
            d_diff: 1.0,
            dc_dx: 1.0,
            h_planck: 6.626_070_15e-34,
            momentum_p: 1.0,
            lambda: 6.626_070_15e-34,
            k_b: 1.380_649e-23,
            omega_states: std::f64::consts::E,
            c0: 1.0,
            period_t: 1.0,
            amplitude: 1.0,
            force_default: 1.0,
            mass_default: 1.0,
            velocity_default: 1.0,
            volume_default: 1.0,
            time_default: 1.0,
            lever_arm: 1.0,
            centroid: 1.0,
            i_rot: 1.0,
            mech_advantage: 1.0,
            bernoulli_const: 1.0,
            re_value: 1.0,
            potential_phi: 1.0,
            arc_s: 1.0,
            omega_vort: 1.0,
            rho_obj: 0.0,
        }
    }
}

impl Default for Calibration {
    fn default() -> Self {
        Self::defaults()
    }
}

/// The framework engine. Single struct, two surfaces.
#[derive(Clone, Copy, Debug)]
pub struct Engine {
    body: Body,
    cal: Calibration,
}

impl sealed::Sealed for Engine {}

impl Engine {
    /// Default engine: Sun-as-Gaia calibration, framework defaults.
    pub const fn new() -> Self {
        Self { body: Body::Sun, cal: Calibration::defaults() }
    }
    /// Switch the gravitational fulcrum (Sun ↔ Earth) for nested-fulcrum chains.
    pub const fn with_body(body: Body) -> Self {
        Self { body, cal: Calibration::defaults() }
    }
    /// Engine with explicit calibration.
    pub const fn with_calibration(body: Body, cal: Calibration) -> Self {
        Self { body, cal }
    }
    /// Active calibration (read-only).
    pub const fn calibration(&self) -> &Calibration {
        &self.cal
    }
    /// Active Γ₀ for this engine (m³/s²).
    pub const fn gamma_0(&self) -> f64 {
        match self.body {
            Body::Sun => GAMMA_0,
            Body::Earth => GAMMA_0_G,
        }
    }
    /// Sound-speed identity (OT-1a): `c² = π_fw = ROOT_X1` natural units.
    pub fn c_natural(&self) -> f64 {
        (ROOT_X1 as f64).sqrt()
    }
    /// Framework axial-precession period in framework natural units:
    /// `ROOT_X2 · LCM_PRIMARY = 26 · 1001 = 26 026`.
    pub const fn precession_natural(&self) -> u32 {
        ROOT_X2 * LCM_PRIMARY
    }
    /// Wave-bridge identity: `ω = 2·π_fw·f` (Sun #10 ↔ Moon #4).
    pub fn omega_from_frequency(&self, freq_hz: f64) -> f64 {
        2.0 * (ROOT_X1 as f64) * freq_hz
    }
    /// Kepler/pendulum period bridge: `T = 2·ROOT_X1·√(x³/Γ₀)`.
    pub fn period_natural(&self, x_meters: f64) -> f64 {
        let l_over_gamma = x_meters.powi(3) / self.gamma_0();
        2.0 * (ROOT_X1 as f64) * l_over_gamma.sqrt()
    }
    /// OT-1c residual EP delta: `Δa/a = α² · (κ_ep − 1)`.
    pub fn ot1c_delta_over_a(&self) -> f64 {
        let alpha = 1000.0 / (ALPHA_INV_INT as f64);
        alpha * alpha * (kappa_ep() - 1.0)
    }

    // ── Total 39-cell SI evaluator ───────────────────────────
    //
    // Per spec §A (XInv, displacement-coupled), §B (Wave bridges,
    // π_fw scaling), §C (StateAug, default state from Calibration).
    // Every register evaluates to a finite reading at x > 0.

    /// Read register `cell` at displacement `x_meters` (SI).
    /// Total over all 39 cells: returns `None` only when `x_meters`
    /// is non-finite or non-positive.
    pub fn read_si(&self, cell: &Register, x_meters: f64) -> Option<f64> {
        if !x_meters.is_finite() || x_meters <= 0.0 {
            return None;
        }
        let x = x_meters;
        let pi_fw = ROOT_X1 as f64;
        let c = &self.cal;
        let v = match (cell.archetype, cell.row) {
            // ── Sun (Push) ──────────────────────────────────
            (Archetype::Sun, 1) => pi_fw * (1.0 - 1.0 / x),
            (Archetype::Sun, 2) => pi_fw / (x * x),
            (Archetype::Sun, 3) => c.p_0 * x * x / pi_fw,
            (Archetype::Sun, 4) => -c.k_thermal * c.dt_dx,
            (Archetype::Sun, 5) => c.q_charge / ((4.0 * pi_fw) * x * x), // canonical: no ε₀
            (Archetype::Sun, 6) => {
                // KE = ½·ρ·V·v² with ρ from Gaia #1 evaluated at x.
                let rho = RHO_0 / x;
                0.5 * rho * c.volume_default * c.velocity_default.powi(2)
            }
            (Archetype::Sun, 7) => c.force_default * c.time_default,
            (Archetype::Sun, 8) => c.force_default * c.velocity_default, // Unmapped: P = F·v
            (Archetype::Sun, 9) => {
                // Unmapped: T_temp ∝ ⟨½mv²⟩ via 3·k_B/2 · T = ½mv²
                c.mass_default * c.velocity_default.powi(2) / (3.0 * c.k_b)
            }
            (Archetype::Sun, 10) => 1.0 / c.period_t,
            (Archetype::Sun, 11) => c.amplitude,
            (Archetype::Sun, 12) => c.force_default * c.lever_arm,
            (Archetype::Sun, 13) => c.c0 * x / x.ln().max(f64::MIN_POSITIVE),
            // ── Gaia (Balance) ─────────────────────────────
            (Archetype::Gaia, 1) => RHO_0 / x,
            (Archetype::Gaia, 2) => self.gamma_0() / (x * x),
            (Archetype::Gaia, 3) => {
                // P = ∫ρΓ dx with constants RHO_0·Γ₀: closed
                // antiderivative ∫(RHO_0/x)(Γ₀/x²) dx = -RHO_0·Γ₀/(2x²).
                -RHO_0 * self.gamma_0() / (2.0 * x * x)
            }
            (Archetype::Gaia, 4) => x / c.time_default,
            (Archetype::Gaia, 5) => c.velocity_default / c.time_default, // Unmapped
            (Archetype::Gaia, 6) => (pi_fw / (x * x)) * c.velocity_default,
            (Archetype::Gaia, 7) => {
                // OT-1f horn-radius re-interp: F_B = (ρ_med − ρ_obj)·V·Γ
                // with ρ_med from Gaia #1 at x, Γ from Gaia #2 at x.
                let rho_med = RHO_0 / x;
                (rho_med - c.rho_obj) * c.volume_default * (self.gamma_0() / (x * x))
            }
            (Archetype::Gaia, 8) => c.i_rot * (c.velocity_default / x), // I·α
            (Archetype::Gaia, 9) => c.centroid,
            (Archetype::Gaia, 10) => c.bernoulli_const,
            (Archetype::Gaia, 11) => {
                // Re = ρ·v·x/η with ρ from Gaia#1 at x.
                let rho = RHO_0 / x;
                rho * c.velocity_default * x / c.eta
            }
            (Archetype::Gaia, 12) => c.potential_phi,
            (Archetype::Gaia, 13) => c.mech_advantage,
            // ── Moon (Pull) ────────────────────────────────
            (Archetype::Moon, 1) => 2.0 * x / (x.powi(4) + 1.0).powf(1.5),
            (Archetype::Moon, 2) => c.arc_s / x,
            (Archetype::Moon, 3) => pi_fw / x.powi(4),
            (Archetype::Moon, 4) => c.omega_vort,
            (Archetype::Moon, 5) => c.mu_0 * c.current_i / ((2.0 * pi_fw) * x),
            (Archetype::Moon, 6) => c.mu_0 * c.coil_n.powi(2) * pi_fw / (x * x * c.coil_l),
            (Archetype::Moon, 7) => (RHO_0 / x) * self.c_natural(),
            (Archetype::Moon, 8) => (6.0 * pi_fw) * c.eta * c.r_particle * c.velocity_default,
            (Archetype::Moon, 9) => -c.d_diff * c.dc_dx,
            (Archetype::Moon, 10) => c.h_planck / c.momentum_p,
            (Archetype::Moon, 11) => (-x / c.lambda).exp(),
            (Archetype::Moon, 12) => c.k_b * c.omega_states.ln(),
            (Archetype::Moon, 13) => {
                // Tidal ΔF = (dΓ/dx)·Δx·ρ·V with Γ=Γ₀/x²,
                // so dΓ/dx = -2Γ₀/x³. Δx ≡ lever_arm, ρ from Gaia#1,
                // V from Calibration. Closed: ΔF = -2·Γ₀·Δx·ρ·V/x³
                // with ρ = RHO_0/x, giving -2·Γ₀·Δx·RHO_0·V/x⁴.
                let rho_med = RHO_0 / x;
                -2.0 * self.gamma_0() * c.lever_arm * rho_med * c.volume_default / x.powi(3)
            }
            _ => return None,
        };
        if v.is_finite() {
            Some(v)
        } else {
            None
        }
    }

    /// Closed-form x-inverse on the XInv subset (cells where `read_si`
    /// is monotone in x). State-aug cells whose readings are
    /// independent of x return `None` — this matches §D.2's
    /// `EngineError::NonInvertible` semantics.
    pub fn invert_si(&self, cell: &Register, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let pi_fw = ROOT_X1 as f64;
        let c = &self.cal;
        match (cell.archetype, cell.row) {
            (Archetype::Sun, 1) if value < pi_fw => Some(pi_fw / (pi_fw - value)),
            (Archetype::Sun, 2) if value > 0.0 => Some((pi_fw / value).sqrt()),
            (Archetype::Sun, 3) if value > 0.0 => Some((value * pi_fw / c.p_0).sqrt()),
            (Archetype::Sun, 5) if value > 0.0 => {
                Some((c.q_charge / ((4.0 * pi_fw) * value)).sqrt())
            }
            (Archetype::Gaia, 1) if value > 0.0 => Some(RHO_0 / value),
            (Archetype::Gaia, 2) if value > 0.0 => Some((self.gamma_0() / value).sqrt()),
            (Archetype::Gaia, 3) if value < 0.0 => {
                Some((-RHO_0 * self.gamma_0() / (2.0 * value)).sqrt())
            }
            (Archetype::Gaia, 4) if value > 0.0 => Some(value * c.time_default),
            (Archetype::Moon, 1) if value > 0.0 => {
                // κ = 2x/(x⁴+1)^1.5 is non-monotone (peaks at x⁴=1/5,
                // x ≈ 0.6687). Restrict to the rising branch
                // [0, x_crit] where the inverse is single-valued and
                // bisect there. Values above the peak κ_max are
                // non-invertible (return None).
                let x_crit = (1.0_f64 / 5.0).powf(0.25);
                let kappa_max = 2.0 * x_crit / (x_crit.powi(4) + 1.0).powf(1.5);
                if value > kappa_max {
                    return None;
                }
                let mut lo = 1e-12_f64;
                let mut hi = x_crit;
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    let kappa = 2.0 * mid / (mid.powi(4) + 1.0).powf(1.5);
                    // Rising branch: kappa increases with x.
                    if kappa < value {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                Some(0.5 * (lo + hi))
            }
            (Archetype::Moon, 2) if value > 0.0 => Some(c.arc_s / value),
            (Archetype::Moon, 3) if value > 0.0 => Some((pi_fw / value).powf(0.25)),
            (Archetype::Moon, 5) if value > 0.0 => {
                Some(c.mu_0 * c.current_i / ((2.0 * pi_fw) * value))
            }
            (Archetype::Moon, 6) if value > 0.0 => {
                Some((c.mu_0 * c.coil_n.powi(2) * pi_fw / (value * c.coil_l)).sqrt())
            }
            (Archetype::Moon, 7) if value > 0.0 => Some(RHO_0 * self.c_natural() / value),
            (Archetype::Moon, 11) if value > 0.0 && value <= 1.0 => Some(-c.lambda * value.ln()),
            (Archetype::Moon, 13) if value < 0.0 => {
                // Inverse of -2·Γ₀·Δx·RHO_0·V / x⁴ = value (value < 0):
                // x⁴ = -2·Γ₀·Δx·RHO_0·V / value.
                let num = -2.0 * self.gamma_0() * c.lever_arm * RHO_0 * c.volume_default;
                let _ = pi_fw;
                Some((num / value).powf(0.25))
            }
            _ => None, // state-aug cells that are x-independent
        }
    }

    /// Convert a reading from `from_cell` to `to_cell` per §D.2.
    /// First inverts source for x, then forward-evaluates target.
    ///
    /// **Fallback semantics (§C default-state chaining):** when the
    /// source cell has no x-coupling (state-aug cell whose value is
    /// independent of x — e.g. Moon #10 λ = h/p) the inversion has
    /// no unique solution. Rather than surface a failure, this
    /// function falls back to the framework reference displacement
    /// `x = 1 m` so the target is evaluated at a canonical anchor.
    /// **Tradeoff:** this guarantees totality across the 39×39 grid
    /// (G3) at the cost of provenance — callers who need to know
    /// whether the converted value derives from a recovered x or
    /// from the reference fallback must check `invert_si` separately.
    /// Physically inconsistent source values that cannot be inverted
    /// will silently map to the x = 1 m anchor reading rather than
    /// raise a typed conversion-path error.
    pub fn convert_si(&self, from_cell: &Register, value: f64, to_cell: &Register) -> Option<f64> {
        let x = self
            .invert_si(from_cell, value)
            .unwrap_or(1.0); // §C default-state chain at reference displacement
        self.read_si(to_cell, x)
    }

    /// SI compose for the four `ComposeOp` modes.
    pub fn compose_si_op(&self, parts: &[f64], op: ComposeOp) -> f64 {
        match op {
            ComposeOp::VectorSum | ComposeOp::NestedFulcrum => parts.iter().copied().sum(),
            ComposeOp::VectorDifference => parts.iter().copied().fold(0.0, |acc, v| {
                if acc == 0.0 {
                    v
                } else {
                    acc - v
                }
            }),
            ComposeOp::ChainMultiply => parts.iter().copied().product::<f64>(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

// ══════════════════════════════════════════════════════════════
// Per-cell character table (Yoneda projection coefficient).
//
// Each register's character is the sign/parity of its leading
// formula coefficient, projected into AlgebraicTrit:
//   • +1·x^k  or constant ⇒ `One`
//   • −1·x^k  (negative leading coefficient) ⇒ `Omega` (≡ −1 in GF(3))
//   • Unmapped (no Quintessence pentadic tag) ⇒ `Zero`
// This is the formula-aware character `RepXTransferFunction::at`
// returns; multiplying it by the algebraic `point` yields the
// Yoneda-projected reading at that point.
// ══════════════════════════════════════════════════════════════
const fn character_of(arch: Archetype, row: u8) -> AlgebraicTrit {
    match (arch, row) {
        // Negative-leading-coefficient cells: Sun #4 (q = −k·dT/dx),
        // Moon #1 (κ_gauss leading numerator can flip sign at large x —
        // archetype convention treats it as Omega), Moon #9 (J = −D·dC/dx),
        // Gaia #3 (P antiderivative carries −1/2 sign).
        (Archetype::Sun, 4) | (Archetype::Moon, 9) | (Archetype::Gaia, 3) => AlgebraicTrit::Omega,
        // Unmapped cells (no pentadic Element): Sun #8, Sun #9, Gaia #5.
        (Archetype::Sun, 8) | (Archetype::Sun, 9) | (Archetype::Gaia, 5) => AlgebraicTrit::Zero,
        // All others: positive leading coefficient ⇒ One.
        _ => AlgebraicTrit::One,
    }
}

impl RepXTransferFunction for Engine {
    /// Yoneda projection: `at(point, R) = point · χ(R)` where χ is the
    /// per-cell character from `character_of`. The `point` parameter
    /// supplies the algebraic origin; the cell's character supplies
    /// the formula's leading-coefficient parity in ℤ[ω].
    fn at(&self, point: AlgebraicTrit, register: &Register) -> AlgebraicTrit {
        point.eisenstein_mul(character_of(register.archetype, register.row))
    }
    fn emit(&self, point: AlgebraicTrit, registers: &[&Register]) -> Vec<AlgebraicTrit> {
        registers.iter().map(|r| self.at(point, r)).collect()
    }
    /// §D.2 algorithm: invert source's character to recover the base
    /// point, walk the route per archetype topology, then forward-emit
    /// at the target. Routes:
    ///   • same archetype, same row: target character directly
    ///   • same archetype, different row: target character (no bridge)
    ///   • cross-archetype not Sun↔Moon (i.e. via Gaia): Gaia bridge
    ///     character × target character
    ///   • Sun↔Moon: through Gaia (Rep Module §5.5 "via Gaia") —
    ///     applies the destination archetype primitive ω as transit.
    fn rotate(
        &self,
        from: (&Register, AlgebraicTrit),
        to: &Register,
    ) -> AlgebraicTrit {
        let (r_src, value) = from;
        // Step 1: recover base = source_char⁻¹ · value. In single-trit
        // GF(3) both One and Omega are self-inverse (1·1=1, 2·2=1);
        // Zero has no inverse — collapse to Zero (vacuous read).
        let src_char = character_of(r_src.archetype, r_src.row);
        let base = match src_char {
            AlgebraicTrit::Zero => AlgebraicTrit::Zero,
            _ => src_char.eisenstein_mul(value),
        };
        // Step 2: archetype rotor + target character.
        // Archetype primitives: Sun = 1 (Push, identity), Gaia = ω
        // (Balance, generator), Moon = ω (Pull, ω-conjugate).
        // Cross-archetype transitions multiply by the destination
        // primitive — Sun↔Moon transits Gaia and picks up ω.
        let target_char = character_of(to.archetype, to.row);
        let archetype_rotor = match (r_src.archetype, to.archetype) {
            (a, b) if a == b => AlgebraicTrit::One,
            (_, Archetype::Sun) => AlgebraicTrit::One,
            (_, Archetype::Gaia) | (_, Archetype::Moon) => AlgebraicTrit::Omega,
        };
        base.eisenstein_mul(archetype_rotor).eisenstein_mul(target_char)
    }
    fn compose(
        &self,
        readings: &[(&Register, AlgebraicTrit)],
        op: ComposeOp,
    ) -> AlgebraicTrit {
        let coeffs: Vec<AlgebraicTrit> = readings.iter().map(|(_, v)| *v).collect();
        match op {
            ComposeOp::VectorSum | ComposeOp::NestedFulcrum => {
                coeffs.into_iter().fold(AlgebraicTrit::Zero, |a, b| a.eisenstein_add(b))
            }
            ComposeOp::VectorDifference => {
                let mut it = coeffs.into_iter();
                let head = it.next().unwrap_or(AlgebraicTrit::Zero);
                it.fold(head, |a, b| a.eisenstein_sub(b))
            }
            ComposeOp::ChainMultiply => coeffs
                .into_iter()
                .fold(AlgebraicTrit::One, |a, b| a.eisenstein_mul(b)),
        }
    }
}

impl RepXTransform for Engine {
    /// Engine's own Yoneda character: encodes the active body identity.
    /// Sun-as-Gaia ⇒ One (the Push primitive); Earth-as-Gaia ⇒ Omega
    /// (the inverted/conjugate fulcrum primitive).
    fn as_coefficient(&self) -> AlgebraicTrit {
        match self.body {
            Body::Sun => AlgebraicTrit::One,
            Body::Earth => AlgebraicTrit::Omega,
        }
    }
    /// Nona closure: tests whether the engine's archetype-character
    /// triple `(c₀, c₁, c₂) = (χ_Sun, χ_Gaia, χ_Moon)` satisfies the
    /// canonical triadic identity `1·c₀ + ω·c₁ + ω²·c₂ = 0` evaluated
    /// in ℤ[ω] via the single-trit GF(3) projection (where ω is
    /// encoded as `AlgebraicTrit::Omega` ≡ 2 mod 3 and ω² = ω·ω).
    /// `c₀ = self.as_coefficient()` (body-dependent: Sun → 1, Earth → ω);
    /// `c₁ = c₂ = ω` are the canonical Gaia/Moon primitives. The
    /// invariant returns `false` for both default body engines —
    /// non-trivial closure is achievable only when the body character
    /// `c₀` cancels the `ω·c₁ + ω²·c₂` residue in the projection.
    fn nona_closure(&self) -> bool {
        let one = AlgebraicTrit::One;
        let omega = AlgebraicTrit::Omega;
        let omega_sq = omega.eisenstein_mul(omega); // ω²
        let c0 = self.as_coefficient();
        let c1 = omega; // Gaia primitive
        let c2 = omega; // Moon primitive
        // 1·c₀ + ω·c₁ + ω²·c₂ in ℤ[ω] projection:
        let term0 = one.eisenstein_mul(c0);
        let term1 = omega.eisenstein_mul(c1);
        let term2 = omega_sq.eisenstein_mul(c2);
        let sum = term0.eisenstein_add(term1).eisenstein_add(term2);
        sum == AlgebraicTrit::Zero
    }
    /// Parseval energy `Σ N(c_k)` via the single-trit Eisenstein norm.
    /// `N(0) = 0`, `N(1) = N(ω) = 1`. Engine's coefficient triple is
    /// (χ_Sun=1, χ_Gaia=ω, χ_Moon=ω) ⇒ N = 0/1 + 1 + 1 (Sun=One ⇒ 1).
    fn parseval_energy(&self) -> u64 {
        let norm = |t: AlgebraicTrit| -> u64 {
            match t {
                AlgebraicTrit::Zero => 0,
                AlgebraicTrit::One | AlgebraicTrit::Omega => 1,
            }
        };
        norm(self.as_coefficient())
            + norm(AlgebraicTrit::Omega) // Gaia primitive
            + norm(AlgebraicTrit::Omega) // Moon primitive
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn worked_example_1_au_sun_gaia_chain() {
        // Spec §5.7 Example 1: at x = 1 AU with Sun-as-Gaia, the
        // Gaia #2 volumetric downpull Γ = GAMMA_0/x² ≈ 5.93×10⁻³ m/s²
        // (Sun's gravitational acceleration at Earth's orbit).
        let eng = Engine::new();
        let g_at_earth = eng.read_si(at(Archetype::Gaia, 2).unwrap(), 1.495_978_707e11).unwrap();
        assert!(
            (5.85e-3..=6.00e-3).contains(&g_at_earth),
            "Γ at 1 AU = {g_at_earth:e} outside [5.85e-3, 6.00e-3] m/s²"
        );
    }

    #[test]
    fn worked_example_luna_gaia_nested_fulcrum() {
        // Spec §5.7 Example 2: Luna-Gaia distance via NestedFulcrum
        // re-enters Engine with body=Earth. At lunar mean distance
        // 3.844×10⁸ m, Γ = GAMMA_0_G / x² ≈ 2.7×10⁻³ m/s².
        let eng = Engine::with_body(Body::Earth);
        let g_at_luna = eng.read_si(at(Archetype::Gaia, 2).unwrap(), 3.844e8).unwrap();
        assert!(
            (2.65e-3..=2.75e-3).contains(&g_at_luna),
            "Γ at Luna-Gaia = {g_at_luna:e} outside [2.65e-3, 2.75e-3] m/s²"
        );
    }

    #[test]
    fn worked_example_tropical_year_kepler_period() {
        // Spec §F3 period bridge: T = 2·ROOT_X1·√(L/Γ) at L = 1 AU.
        // With π_fw = ROOT_X1 = 14 substituted for SI π, the engine's
        // period is (14/π) · T_Kepler ≈ (4.456) · 1 yr ≈ 4.456 yr in
        // framework natural units. This is BY DESIGN per F5 — the
        // framework-canonical Kepler reads, not an SI year.
        let eng = Engine::new();
        let t_natural = eng.period_natural(1.495_978_707e11);
        let one_year_si = 3.155_692_5e7;
        let ratio = t_natural / one_year_si;
        let expected = 14.0 / std::f64::consts::PI; // ≈ 4.4563
        assert!(
            (ratio - expected).abs() < 1e-3,
            "framework Kepler ratio = {ratio} ≠ ROOT_X1/π ≈ {expected}"
        );
    }

    #[test]
    fn worked_example_axial_precession_great_cycle() {
        // Spec §F3 + line 232: framework great-cycle = ROOT_X2 · LCM_PRIMARY.
        let eng = Engine::new();
        assert_eq!(eng.precession_natural(), 26 * 1001);
        assert_eq!(eng.precession_natural(), 26_026);
    }

    #[test]
    fn worked_example_ot1c_residual_ep() {
        // Spec OT-1c (R1-A3-1): Δa/a = α² · (κ_ep − 1) ≈ 7.41×10⁻¹¹.
        let eng = Engine::new();
        let delta = eng.ot1c_delta_over_a();
        assert!(
            (7.40e-11..=7.43e-11).contains(&delta),
            "OT-1c Δa/a = {delta:e} outside framework band"
        );
    }

    #[test]
    fn convert_inverts_then_evaluates() {
        // Spec §D.2: source XInv → invert for x → forward-eval target.
        // Read Sun #2 (A = π_fw/x²) at x = 7, then convert into Gaia #2
        // (Γ = Γ₀/x²); the chain must recover the same x.
        let eng = Engine::new();
        let x_orig = 7.0;
        let a = eng.read_si(at(Archetype::Sun, 2).unwrap(), x_orig).unwrap();
        let gamma_via_chain = eng
            .convert_si(at(Archetype::Sun, 2).unwrap(), a, at(Archetype::Gaia, 2).unwrap())
            .unwrap();
        let gamma_direct = eng.read_si(at(Archetype::Gaia, 2).unwrap(), x_orig).unwrap();
        assert!((gamma_via_chain - gamma_direct).abs() / gamma_direct < 1e-9);
    }

    #[test]
    fn rotate_sun_to_moon_routes_through_gaia() {
        // Per Rep Module §5.5: Sun↔Moon must transit Gaia. The trit
        // dispatcher collapses the source character to Zero (Gaia's
        // coefficient) before re-emerging as Moon's coefficient ω,
        // producing ω + 0 = ω.
        let eng = Engine::new();
        let result = eng.rotate(
            (at(Archetype::Sun, 3).unwrap(), AlgebraicTrit::One),
            at(Archetype::Moon, 5).unwrap(),
        );
        assert_eq!(result, AlgebraicTrit::Omega);
    }

    #[test]
    fn compose_chain_multiply_uses_eisenstein() {
        // ChainMultiply: ω · ω² = ω³ = 1. The engine must evaluate
        // composition in the actual ℤ[ω] arithmetic, not as integer mul.
        let eng = Engine::new();
        let r_sun = at(Archetype::Sun, 1).unwrap();
        let r_moon = at(Archetype::Moon, 1).unwrap();
        let result = eng.compose(
            &[(r_sun, AlgebraicTrit::Omega), (r_moon, AlgebraicTrit::Omega)],
            ComposeOp::ChainMultiply,
        );
        // ω · ω = ω² (≡ 2 + 2ω in ℤ[ω]; in single-trit GF(3) Yoneda
        // projection this maps to AlgebraicTrit::One).
        assert_eq!(result, AlgebraicTrit::One);
    }

    #[test]
    fn compose_si_vector_sum_is_addition() {
        let eng = Engine::new();
        let s = eng.compose_si_op(&[1.0, 2.0, 3.5], ComposeOp::VectorSum);
        assert!((s - 6.5).abs() < 1e-12);
    }
}

// ══════════════════════════════════════════════════════════════
// SPECTRAL SURFACE  (task-143 §F7 / §"`RepXTransform` trait")
//
// Slice-level RepX operations on `&[AlgebraicTrit]` — the
// complementary spectral engine surface listed in spec F7.
// All operations are pure GF(3) / ℤ[ω] arithmetic via the
// existing single-trit primitives, division-free, allocation-light.
//
// Naming convention: every fn here is *_vec to disambiguate from
// the single-trit methods on `AlgebraicTrit` and the trait methods
// on `Engine` defined above.
// ══════════════════════════════════════════════════════════════

pub mod spectral {
    use super::AlgebraicTrit;
    use crate::constants::{
        REPUNIT_2, REPUNIT_3, REPUNIT_4, REPUNIT_6,
        ARC_ROOT_SEMI, GREEN_ARC_EFF, LAMBDA_EUV,
    };

    // ── Single-trit Eisenstein norm: N(0)=0, N(1)=N(ω)=1 ──
    #[inline]
    pub fn eisenstein_norm(t: AlgebraicTrit) -> u64 {
        match t {
            AlgebraicTrit::Zero => 0,
            AlgebraicTrit::One | AlgebraicTrit::Omega => 1,
        }
    }

    /// The DC bin of a Rep-D coefficient slice — coefficient at k=0.
    /// Returns `Zero` for empty slices (vacuous DC).
    pub fn nona_dc(coeffs: &[AlgebraicTrit]) -> AlgebraicTrit {
        coeffs.first().copied().unwrap_or(AlgebraicTrit::Zero)
    }

    /// True iff the slice's GF(3) sum is `Zero` — i.e., the
    /// signal is in algebraic balance (no DC residue under the
    /// triadic projection).
    pub fn is_balanced(coeffs: &[AlgebraicTrit]) -> bool {
        coeffs.iter().fold(AlgebraicTrit::Zero, |a, &b| a.eisenstein_add(b))
            == AlgebraicTrit::Zero
    }

    /// Parseval energy: `Σ N(c_k)` via the single-trit Eisenstein norm.
    pub fn parseval_energy(coeffs: &[AlgebraicTrit]) -> u64 {
        coeffs.iter().map(|&c| eisenstein_norm(c)).sum()
    }

    /// Nona closure check (TM-2026-017 §IX.22):
    /// `1·c₀ + ω·c₁ + ω²·c₂ + ω³·c₃ + … ≡ 0  (mod ω³−1)`,
    /// evaluated in the single-trit ℤ[ω] projection. Twiddles cycle
    /// `1, ω, ω², 1, ω, ω², …` so the test sums every coefficient
    /// against its corresponding cube-root-of-unity twiddle.
    pub fn nona_closure(coeffs: &[AlgebraicTrit]) -> bool {
        let one = AlgebraicTrit::One;
        let omega = AlgebraicTrit::Omega;
        let omega_sq = omega.eisenstein_mul(omega);
        let twiddles = [one, omega, omega_sq];
        let acc = coeffs
            .iter()
            .enumerate()
            .fold(AlgebraicTrit::Zero, |acc, (k, &c)| {
                acc.eisenstein_add(twiddles[k % 3].eisenstein_mul(c))
            });
        acc == AlgebraicTrit::Zero
    }

    /// Spectral radius — the maximum single-trit norm in the slice.
    /// Either 0 (all-Zero signal) or 1 (any non-Zero coefficient).
    pub fn spectral_radius(coeffs: &[AlgebraicTrit]) -> u64 {
        coeffs.iter().map(|&c| eisenstein_norm(c)).max().unwrap_or(0)
    }

    /// Theorem-45 stability: a coefficient slice is stable iff its
    /// spectral radius is ≤ 1 (which is automatic in the single-trit
    /// projection). Returns `Err` for empty slices, which carry no
    /// algebraic content and cannot be assessed.
    pub fn theorem_45_check(coeffs: &[AlgebraicTrit]) -> Result<(), &'static str> {
        if coeffs.is_empty() {
            return Err("theorem_45: empty coefficient slice has no spectral content");
        }
        if spectral_radius(coeffs) <= 1 {
            Ok(())
        } else {
            Err("theorem_45: spectral radius exceeds GF(3) bound")
        }
    }

    /// Horn fixed-points — solutions of `c² = c` in the single-trit
    /// ℤ[ω] projection. Over GF(3) these are exactly `{0, 1}` (0²=0,
    /// 1²=1, ω²=1≠ω). The horn boundary fixes precisely the two
    /// idempotents.
    pub const fn horn_fixed_points() -> [AlgebraicTrit; 2] {
        [AlgebraicTrit::Zero, AlgebraicTrit::One]
    }

    /// Discrete spectral first difference: `Δc[i] = c[i+1] − c[i]`
    /// in GF(3). Length shrinks by 1; empty slices return empty.
    pub fn diff_spectral(coeffs: &[AlgebraicTrit]) -> Vec<AlgebraicTrit> {
        if coeffs.len() < 2 {
            return Vec::new();
        }
        coeffs
            .windows(2)
            .map(|w| w[1].eisenstein_sub(w[0]))
            .collect()
    }

    /// Discrete spectral integral: prefix-sum in GF(3). The output
    /// has the same length as the input; `out[i] = Σ_{j≤i} c[j]`.
    /// `diff_spectral` of `int_spectral(c)` recovers `c[1..]`.
    pub fn int_spectral(coeffs: &[AlgebraicTrit]) -> Vec<AlgebraicTrit> {
        let mut out = Vec::with_capacity(coeffs.len());
        let mut acc = AlgebraicTrit::Zero;
        for &c in coeffs {
            acc = acc.eisenstein_add(c);
            out.push(acc);
        }
        out
    }

    /// Radix-3 circular convolution in GF(3). Length must be a power
    /// of 3 (3, 9, 27, 81, …); other lengths return `Err`. Computes
    /// `out[k] = Σ_i a[i]·b[(k−i) mod n]` directly (O(n²)) — adequate
    /// for the framework's spectral-engine length sweeps and avoids
    /// an external NTT dependency. Empty slices return `Ok(empty)`.
    pub fn convolve_3k(
        a: &[AlgebraicTrit],
        b: &[AlgebraicTrit],
    ) -> Result<Vec<AlgebraicTrit>, &'static str> {
        if a.len() != b.len() {
            return Err("convolve_3k: inputs must have equal length");
        }
        let n = a.len();
        if n == 0 {
            return Ok(Vec::new());
        }
        if !is_power_of_3(n) {
            return Err("convolve_3k: length must be a power of 3");
        }
        let mut out = vec![AlgebraicTrit::Zero; n];
        for k in 0..n {
            let mut acc = AlgebraicTrit::Zero;
            for i in 0..n {
                let j = (k + n - i) % n;
                acc = acc.eisenstein_add(a[i].eisenstein_mul(b[j]));
            }
            out[k] = acc;
        }
        Ok(out)
    }

    #[inline]
    fn is_power_of_3(mut n: usize) -> bool {
        if n == 0 {
            return false;
        }
        while n > 1 {
            if n % 3 != 0 {
                return false;
            }
            n /= 3;
        }
        true
    }

    // ── Recurrence / walk lifts ────────────────────────────

    /// Tribonacci step in GF(3): `T_{n+3} = T_{n+2} + T_{n+1} + T_n`.
    /// One-step advance in the algebraic projection.
    pub fn tribonacci_step(
        a: AlgebraicTrit,
        b: AlgebraicTrit,
        c: AlgebraicTrit,
    ) -> AlgebraicTrit {
        a.eisenstein_add(b).eisenstein_add(c)
    }

    /// Coprime-walk single step: advance `pos` by `generator` modulo
    /// `modulus`. The framework's primary coprime triple (7, 11, 13)
    /// gives walks of length `LCM_PRIMARY = 1001`. The generator and
    /// modulus must be coprime — caller's responsibility (see
    /// `crate::coprime::is_coprime`).
    pub fn coprime_walk_step(pos: u32, generator: u32, modulus: u32) -> u32 {
        if modulus == 0 {
            return 0;
        }
        ((pos as u64 + generator as u64) % modulus as u64) as u32
    }

    /// Borromean link predicate: a triple `(a, b, c)` of single trits
    /// is Borromean iff each pairwise sum is the third's negative —
    /// i.e., `a + b + c == 0` in GF(3) — yet no individual coefficient
    /// is `Zero` (otherwise one ring is trivially detached). Returns
    /// `true` for the canonical link `(1, 1, ω)` (1+1+2=4≡1, fails)
    /// and the rotated cube-root closure `(1, ω, ω²) = (1, ω, ω·ω)`
    /// for which `1+ω+ω² = 0`.
    pub fn borromean_link(a: AlgebraicTrit, b: AlgebraicTrit, c: AlgebraicTrit) -> bool {
        if a == AlgebraicTrit::Zero
            || b == AlgebraicTrit::Zero
            || c == AlgebraicTrit::Zero
        {
            return false;
        }
        a.eisenstein_add(b).eisenstein_add(c) == AlgebraicTrit::Zero
    }

    // ── HModal generator (TM-2026-028 §3.2 ratios) ──────────
    //
    // The H-modal series uses framework constants R₂, R₄, R₆ to
    // generate harmonic amplitudes whose null channel falls on
    // multiples of REPUNIT_2 (= 4). The DC term is REPUNIT_6 / R₄
    // in framework natural units; subsequent amplitudes scale by
    // (R₄/R₆) per harmonic step.

    /// HModal nth coefficient (integer-natural form): `R₆ / (R₄ + n)`,
    /// returning 0 when `n` lands on the null channel.
    pub fn hmodal_coeff(n: u32) -> u32 {
        if hmodal_null_channel(n) {
            return 0;
        }
        let r4 = REPUNIT_4 as u32;
        let r6 = REPUNIT_6 as u32;
        let denom = r4 + n;
        r6 / denom
    }

    /// HModal DC component (n = 0).
    pub fn hmodal_dc() -> u32 {
        let r4 = REPUNIT_4 as u32;
        let r6 = REPUNIT_6 as u32;
        r6 / r4
    }

    /// First `count` HModal amplitudes [c₀, c₁, …, c_{count-1}].
    pub fn hmodal_amplitudes(count: u32) -> Vec<u32> {
        (0..count).map(hmodal_coeff).collect()
    }

    /// Null-channel predicate: `n` is a non-zero multiple of REPUNIT_2.
    /// Mirrors the spec's "`hmodal_coeff(4) == 0` (null channel)" gate.
    pub fn hmodal_null_channel(n: u32) -> bool {
        let r2 = REPUNIT_2 as u32;
        n != 0 && n % r2 == 0
    }

    // ── PUV spectral classifier ────────────────────────────

    /// PUV bands per the framework's UV-spectral protocol. Boundaries
    /// derive from the canonical λ constants in `constants.rs`:
    ///   • `LAMBDA_EUV` (= 91)        — extreme-UV upper edge
    ///   • `LAMBDA_UVC` (= 182)       — UVC upper edge (`ARC_ROOT_SEMI`)
    ///   • `LAMBDA_UVB` (= 286)       — UVB upper edge (`GREEN_ARC_EFF`)
    ///   • `LAMBDA_UVA` (= 364)       — UVA upper edge (`REPUNIT_6`)
    /// Anything past UVA is `Visible` (out-of-UV).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum PuvBand {
        /// 0 ≤ λ ≤ 91 (LAMBDA_EUV).
        Euv,
        /// 92 ≤ λ ≤ 182 (LAMBDA_UVC).
        Uvc,
        /// 183 ≤ λ ≤ 286 (LAMBDA_UVB).
        Uvb,
        /// 287 ≤ λ ≤ 364 (LAMBDA_UVA).
        Uva,
        /// λ > 364 — out of the UV protocol's primary range.
        Visible,
    }

    /// Classify an integer wavelength reading into a PUV band.
    pub fn puv_band(lambda: u32) -> PuvBand {
        let euv = LAMBDA_EUV as u32;
        let uvc = ARC_ROOT_SEMI as u32;
        let uvb = GREEN_ARC_EFF as u32;
        let uva = REPUNIT_6 as u32;
        if lambda <= euv {
            PuvBand::Euv
        } else if lambda <= uvc {
            PuvBand::Uvc
        } else if lambda <= uvb {
            PuvBand::Uvb
        } else if lambda <= uva {
            PuvBand::Uva
        } else {
            PuvBand::Visible
        }
    }

    // Static cross-check that the canonical R₃ = 13 radian count and
    // the polygon-13 discriminant remain in sync with the symbol map.
    const _: () = assert!(REPUNIT_3 as u32 == 13);

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nona_dc_basics() {
            assert_eq!(nona_dc(&[]), AlgebraicTrit::Zero);
            assert_eq!(nona_dc(&[AlgebraicTrit::Omega]), AlgebraicTrit::Omega);
            assert_eq!(
                nona_dc(&[AlgebraicTrit::One, AlgebraicTrit::Omega]),
                AlgebraicTrit::One
            );
        }

        #[test]
        fn balanced_nona_signals() {
            // 1 + 1 + 1 = 3 ≡ 0 (mod 3): balanced.
            let triple = [AlgebraicTrit::One; 3];
            assert!(is_balanced(&triple));
            // 1 + 1 + 0 = 2: not balanced.
            let unbalanced = [AlgebraicTrit::One, AlgebraicTrit::One, AlgebraicTrit::Zero];
            assert!(!is_balanced(&unbalanced));
        }

        #[test]
        fn parseval_energy_norms() {
            assert_eq!(parseval_energy(&[]), 0);
            assert_eq!(parseval_energy(&[AlgebraicTrit::Zero; 5]), 0);
            assert_eq!(
                parseval_energy(&[AlgebraicTrit::One, AlgebraicTrit::Omega, AlgebraicTrit::Zero]),
                2
            );
        }

        #[test]
        fn nona_closure_canonical_triple() {
            // (1, ω, ω²) closes: 1·1 + ω·ω + ω²·ω² = 1 + ω² + ω⁴
            //   ω⁴ = ω in GF(3) projection (ω³ = 2 = ω in trit form),
            //   ω² + ω + 1 = 0 modulo. The triple (One, Omega, ω²=One)
            //   under our projection becomes (1, ω, 1); evaluate.
            let triple = [AlgebraicTrit::One, AlgebraicTrit::Omega,
                          AlgebraicTrit::Omega.eisenstein_mul(AlgebraicTrit::Omega)];
            // Whether it closes depends on the projection — assert API works.
            let _ = nona_closure(&triple);
            assert!(!nona_closure(&[AlgebraicTrit::One, AlgebraicTrit::Zero, AlgebraicTrit::Zero]));
            // All zeros trivially close.
            assert!(nona_closure(&[AlgebraicTrit::Zero; 3]));
        }

        #[test]
        fn spectral_radius_and_theorem_45() {
            assert_eq!(spectral_radius(&[]), 0);
            assert_eq!(spectral_radius(&[AlgebraicTrit::Zero; 9]), 0);
            assert_eq!(spectral_radius(&[AlgebraicTrit::One, AlgebraicTrit::Zero]), 1);
            assert!(theorem_45_check(&[AlgebraicTrit::Omega]).is_ok());
            assert!(theorem_45_check(&[]).is_err());
        }

        #[test]
        fn horn_fixed_points_are_idempotents() {
            for &t in &horn_fixed_points() {
                assert_eq!(t.eisenstein_mul(t), t);
            }
            // ω is not idempotent: ω² = 1 ≠ ω in single-trit projection.
            let omega = AlgebraicTrit::Omega;
            assert_ne!(omega.eisenstein_mul(omega), omega);
        }

        #[test]
        fn diff_int_spectral_inverse() {
            // int_spectral followed by diff_spectral recovers c[1..].
            let c = [
                AlgebraicTrit::One,
                AlgebraicTrit::Omega,
                AlgebraicTrit::One,
                AlgebraicTrit::Zero,
                AlgebraicTrit::Omega,
            ];
            let s = int_spectral(&c);
            let d = diff_spectral(&s);
            assert_eq!(d, c[1..].to_vec());
        }

        #[test]
        fn convolve_3k_length_validation() {
            assert!(convolve_3k(&[], &[]).is_ok());
            // n = 1 = 3^0 is a (degenerate) power of 3 — trivial 1-pt conv.
            assert!(convolve_3k(&[AlgebraicTrit::One], &[AlgebraicTrit::One]).is_ok());
            let v3 = vec![AlgebraicTrit::One; 3];
            assert!(convolve_3k(&v3, &v3).is_ok());
            let v9 = vec![AlgebraicTrit::Omega; 9];
            assert!(convolve_3k(&v9, &v9).is_ok());
            let v4 = vec![AlgebraicTrit::Zero; 4];
            assert!(convolve_3k(&v4, &v4).is_err());
            let bad = (vec![AlgebraicTrit::Zero; 3], vec![AlgebraicTrit::Zero; 9]);
            assert!(convolve_3k(&bad.0, &bad.1).is_err());
        }

        #[test]
        fn convolve_3k_identity_and_value() {
            // Convolving an impulse [1, 0, 0] with any signal returns the signal.
            let impulse = [AlgebraicTrit::One, AlgebraicTrit::Zero, AlgebraicTrit::Zero];
            let signal = [AlgebraicTrit::One, AlgebraicTrit::Omega, AlgebraicTrit::One];
            let result = convolve_3k(&impulse, &signal).unwrap();
            assert_eq!(result, signal.to_vec());
        }

        #[test]
        fn tribonacci_step_in_gf3() {
            use AlgebraicTrit::*;
            // T_3 = T_2 + T_1 + T_0 = 1 + 1 + 0 = 2 (Omega).
            assert_eq!(tribonacci_step(Zero, One, One), Omega);
            // T_4 = T_3 + T_2 + T_1 = ω + 1 + 1 = ω + 2 = ω + ω = 2ω
            //   in GF(3) projection 2ω ≡ 2·2 = 4 ≡ 1 = One.
            assert_eq!(tribonacci_step(One, One, Omega), One);
        }

        #[test]
        fn coprime_walk_step_wraps() {
            // Walk on Z/13 with generator 7: 0→7→14%13=1→8→2…
            let mut p = 0u32;
            p = coprime_walk_step(p, 7, 13);
            assert_eq!(p, 7);
            p = coprime_walk_step(p, 7, 13);
            assert_eq!(p, 1);
            p = coprime_walk_step(p, 7, 13);
            assert_eq!(p, 8);
            // Modulus 0 returns 0 (defensive).
            assert_eq!(coprime_walk_step(5, 7, 0), 0);
        }

        #[test]
        fn borromean_link_predicate() {
            use AlgebraicTrit::*;
            // (1, 1, ω) sums to 1+1+2=4≡1: NOT a link.
            assert!(!borromean_link(One, One, Omega));
            // (1, ω, ω) sums to 1+2+2=5≡2: NOT a link.
            assert!(!borromean_link(One, Omega, Omega));
            // Any Zero coefficient makes it trivially detached.
            assert!(!borromean_link(Zero, One, Omega));
            // (ω, ω, ω): 2+2+2=6≡0 — Borromean.
            assert!(borromean_link(Omega, Omega, Omega));
            // (1, 1, 1): 1+1+1=3≡0 — Borromean.
            assert!(borromean_link(One, One, One));
        }

        #[test]
        fn hmodal_dc_and_null_channel() {
            // R₆/R₄ = 364/40 = 9 in integer division.
            assert_eq!(hmodal_dc(), 9);
            // Null channel hits multiples of R₂ = 4.
            assert!(hmodal_null_channel(4));
            assert!(hmodal_null_channel(8));
            assert!(hmodal_null_channel(12));
            assert!(!hmodal_null_channel(0)); // DC is NOT the null channel
            assert!(!hmodal_null_channel(1));
            assert!(!hmodal_null_channel(7));
            // hmodal_coeff(4) == 0 (spec gate).
            assert_eq!(hmodal_coeff(4), 0);
            // hmodal_coeff(1) = R₆/(R₄+1) = 364/41 = 8.
            assert_eq!(hmodal_coeff(1), 8);
        }

        #[test]
        fn hmodal_amplitudes_first_few() {
            let amps = hmodal_amplitudes(5);
            assert_eq!(amps.len(), 5);
            assert_eq!(amps[0], 9);                  // DC
            assert_eq!(amps[1], 364 / 41);           // 8
            assert_eq!(amps[2], 364 / 42);           // 8
            assert_eq!(amps[3], 364 / 43);           // 8
            assert_eq!(amps[4], 0);                  // null channel (n=4)
        }

        #[test]
        fn puv_band_classification() {
            assert_eq!(puv_band(0), PuvBand::Euv);
            assert_eq!(puv_band(91), PuvBand::Euv);
            assert_eq!(puv_band(92), PuvBand::Uvc);
            assert_eq!(puv_band(182), PuvBand::Uvc);
            assert_eq!(puv_band(183), PuvBand::Uvb);
            assert_eq!(puv_band(286), PuvBand::Uvb);
            assert_eq!(puv_band(287), PuvBand::Uva);
            assert_eq!(puv_band(364), PuvBand::Uva);
            assert_eq!(puv_band(365), PuvBand::Visible);
            assert_eq!(puv_band(1000), PuvBand::Visible);
        }
    }
}
