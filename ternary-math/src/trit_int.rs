// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! # TritInt — Ternary-Native Scalar Storage Primitive
//!
//! A ternary integer: one whole number stored in base 3 with variable
//! precision. Small values (≤ R₄ = 40 trits) use an inline buffer.
//! Large values use the heap (Phase 6 — not yet implemented).
//!
//! **Packing:** 5 trits per byte (3⁵ = 243 < 256 = 2⁸). This is where
//! ternary arithmetic meets binary hardware — the packing ratio is forced
//! by the host, not chosen.
//!
//! **Trit ordering:** Least significant trit first. Trit 0 is the least
//! significant, stored at packed\[0\] position 0.
//!
//! **Internal representation:** Rep B {0, 1, 2}. Zero is valid internally
//! as the additive identity. Rep C {1, 2, 3} is the wire format — conversion
//! happens at the boundary via `to_repr_c()` / `from_repr_c()` (Phase 3).
//!
//! **Position in the type chain:**
//! - `TritInt` — one ternary integer (this module)
//! - `Trit` — three TritInts: v\[0\] = ℤ, v\[1\] = φ, v\[2\] = ω (Phase 2)
//! - `[Trit; 3]` — one vertex coordinate in ℤ\[φ,ω\]
//! - Triangles, meshes, manifolds — built on Trit

use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use std::ops::{Add, Sub, Mul, Div, Rem, AddAssign, SubAssign, MulAssign};
use crate::gf3_algebra::AlgebraicTrit;
use zeroize::Zeroize;

// ══════════════════════════════════════════════════════════════
// TYPE DEFINITIONS
// ══════════════════════════════════════════════════════════════

/// Error returned when a TritInt value exceeds the target binary width.
/// The field holds the target bit width (32, 64, or 128).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overflow(pub u32);

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TritInt value exceeds u{} range", self.0)
    }
}

/// Result of addition with carry metadata.
pub struct TritIntAddResult {
    pub value: TritInt,
    pub carry_count: u32,
    pub max_carry_chain: u32,
}

/// Maximum trit count for the inline path.
/// R₄ = (3⁴ − 1)/2 = 40 — the four-digit repunit.
const MAX_INLINE_TRITS: u8 = 40;

/// Maximum trit count for heap allocation from untrusted input.
/// R₉ = (3⁹ − 1)/2 = 9,841 — the nine-digit repunit.
/// Framework-derived upper bound: ≥100× any anticipated production value,
/// ~2 KB maximum allocation per TritInt. Prevents allocation-exhaustion.
const MAX_HEAP_TRITS: u32 = 9841;

/// Powers of 3 within a single byte: 3⁰ through 3⁵.
/// Used for trit packing/unpacking. 3⁵ = 243 < 256 fits in u8.
const POW3: [u8; 6] = [1, 3, 9, 27, 81, 243];

/// Storage backend for TritInt.
enum TritIntStorage {
    /// Inline: R₄/5 = 40/5 = 8 bytes, derived from trit capacity and packing ratio.
    /// trit_count is u8 — the smallest host integer type that holds R₄ = 40.
    Inline {
        packed: [u8; 8],
        trit_count: u8,
    },
    /// Heap: for values exceeding R₄ = 40 trits. Same 5-trits-per-byte packing.
    /// ManuallyDrop keeps the enum const-compatible (no implicit Drop on TritInt).
    /// Cleanup: call .zeroize() or .drop_heap() before going out of scope.
    /// Framework constants are always Inline — no heap leak risk for const items.
    /// trit_count is u32 — host bookkeeping field (not a mathematical value).
    Heap {
        packed: ManuallyDrop<Vec<u8>>,
        trit_count: u32,
    },
}

impl Clone for TritIntStorage {
    fn clone(&self) -> Self {
        match self {
            TritIntStorage::Inline { packed, trit_count } =>
                TritIntStorage::Inline { packed: *packed, trit_count: *trit_count },
            TritIntStorage::Heap { packed, trit_count } =>
                TritIntStorage::Heap {
                    packed: ManuallyDrop::new((**packed).clone()),
                    trit_count: *trit_count,
                },
        }
    }
}

/// A ternary integer — one whole number stored in base 3.
///
/// Packing: 5 trits per byte. `byte_value = t₀ + 3·t₁ + 9·t₂ + 27·t₃ + 81·t₄`.
/// Least significant trit first. Internal representation is Rep B {0, 1, 2}.
#[derive(Clone)]
pub struct TritInt {
    storage: TritIntStorage,
}

// ══════════════════════════════════════════════════════════════
// INTERNAL CONST HELPERS
//
// These operate on raw packed arrays and counts. They exist because
// const fn on Rust 1.77 cannot take &mut parameters — all mutation
// must be on local variables. The public const methods on TritInt
// extract arrays, call these helpers, and wrap the result.
// ══════════════════════════════════════════════════════════════

/// Extract a single trit from a packed array.
const fn trit_at_packed(packed: [u8; 8], pos: u8) -> u8 {
    let byte_idx = (pos / 5) as usize;
    let trit_idx = (pos % 5) as usize;
    (packed[byte_idx] / POW3[trit_idx]) % 3
}

/// Normalize: strip leading zero trits and clear garbage beyond trit_count.
/// Returns (packed, count) in canonical form. Two TritInts with the same
/// mathematical value always produce identical (packed, count) after normalization.
const fn normalize(mut packed: [u8; 8], count: u8) -> ([u8; 8], u8) {
    // Strip leading zeros
    let mut c = count;
    while c > 0 && trit_at_packed(packed, c - 1) == 0 {
        c -= 1;
    }

    // Clear bytes beyond the last used byte
    let last_byte = if c == 0 { 0 } else { ((c - 1) / 5) as usize + 1 };
    let mut i = last_byte;
    while i < 8 {
        packed[i] = 0;
        i += 1;
    }

    // Mask unused trits in the last used byte
    if c > 0 {
        let trits_in_last = ((c - 1) % 5) + 1;
        let byte_idx = ((c - 1) / 5) as usize;
        packed[byte_idx] = packed[byte_idx] % POW3[trits_in_last as usize];
    } else {
        packed[0] = 0;
    }

    (packed, c)
}

/// Pack a TritInt from normalized packed array and count.
const fn make_inline(packed: [u8; 8], trit_count: u8) -> TritInt {
    let (p, c) = normalize(packed, trit_count);
    TritInt { storage: TritIntStorage::Inline { packed: p, trit_count: c } }
}

/// Add two packed arrays, returning (result_packed, result_count).
const fn add_packed(
    a: [u8; 8], a_count: u8,
    b: [u8; 8], b_count: u8,
) -> ([u8; 8], u8) {
    let max = if a_count > b_count { a_count } else { b_count };
    let mut result = [0u8; 8];
    let mut carry: u8 = 0;
    let mut i: u8 = 0;
    let mut current_byte: u8 = 0;
    let mut trits_in_byte: u8 = 0;
    let mut byte_idx: usize = 0;

    while i < max || carry > 0 {
        let at = if i < a_count { trit_at_packed(a, i) } else { 0 };
        let bt = if i < b_count { trit_at_packed(b, i) } else { 0 };
        let sum = at + bt + carry;
        current_byte += (sum % 3) * POW3[trits_in_byte as usize];
        carry = sum / 3;
        trits_in_byte += 1;
        if trits_in_byte == 5 {
            result[byte_idx] = current_byte;
            current_byte = 0;
            trits_in_byte = 0;
            byte_idx += 1;
        }
        i += 1;
    }
    if trits_in_byte > 0 {
        result[byte_idx] = current_byte;
    }

    assert!(i <= MAX_INLINE_TRITS, "add result exceeds R4 = 40 trits");
    normalize(result, i)
}

/// Subtract b from a (a ≥ b required), returning (result_packed, result_count).
const fn sub_packed(
    a: [u8; 8], a_count: u8,
    b: [u8; 8], b_count: u8,
) -> ([u8; 8], u8) {
    let max = if a_count > b_count { a_count } else { b_count };
    let mut result = [0u8; 8];
    let mut borrow: u8 = 0;
    let mut i: u8 = 0;
    let mut current_byte: u8 = 0;
    let mut trits_in_byte: u8 = 0;
    let mut byte_idx: usize = 0;

    while i < max {
        let at = if i < a_count { trit_at_packed(a, i) } else { 0 };
        let bt = if i < b_count { trit_at_packed(b, i) } else { 0 };
        let bt_plus_borrow = bt + borrow;

        let (digit, new_borrow) = if at >= bt_plus_borrow {
            (at - bt_plus_borrow, 0u8)
        } else {
            (at + 3 - bt_plus_borrow, 1u8)
        };
        borrow = new_borrow;

        current_byte += digit * POW3[trits_in_byte as usize];
        trits_in_byte += 1;
        if trits_in_byte == 5 {
            result[byte_idx] = current_byte;
            current_byte = 0;
            trits_in_byte = 0;
            byte_idx += 1;
        }
        i += 1;
    }
    if trits_in_byte > 0 {
        result[byte_idx] = current_byte;
    }

    assert!(borrow == 0, "const_sub underflow: subtrahend > minuend");
    normalize(result, max)
}

/// Multiply two packed arrays using schoolbook ternary long multiplication.
const fn mul_packed(
    a: [u8; 8], a_count: u8,
    b: [u8; 8], b_count: u8,
) -> ([u8; 8], u8) {
    // Product of a_count and b_count trit numbers has at most a_count + b_count trits
    let max_trits = a_count + b_count;
    assert!(max_trits <= MAX_INLINE_TRITS, "const_mul result exceeds R4 = 40 trits");

    // Work on unpacked trit array (max 80 trits for safety, though we assert ≤ 40)
    let mut result_trits = [0u8; 80];

    let mut i: u8 = 0;
    while i < a_count {
        let a_trit = trit_at_packed(a, i);
        if a_trit != 0 {
            let mut carry: u8 = 0;
            let mut j: u8 = 0;
            while j < b_count {
                let b_trit = trit_at_packed(b, j);
                let pos = (i + j) as usize;
                let sum = a_trit * b_trit + result_trits[pos] + carry;
                result_trits[pos] = sum % 3;
                carry = sum / 3;
                j += 1;
            }
            // Propagate remaining carry
            let mut k = (i + b_count) as usize;
            while carry > 0 {
                let sum = result_trits[k] + carry;
                result_trits[k] = sum % 3;
                carry = sum / 3;
                k += 1;
            }
        }
        i += 1;
    }

    // Find actual trit count
    let mut count: u8 = max_trits;
    while count > 0 && result_trits[(count - 1) as usize] == 0 {
        count -= 1;
    }

    // Pack into bytes
    let mut packed = [0u8; 8];
    let mut t: u8 = 0;
    while t < count {
        let byte_idx = (t / 5) as usize;
        let trit_idx = (t % 5) as usize;
        packed[byte_idx] += result_trits[t as usize] * POW3[trit_idx];
        t += 1;
    }

    normalize(packed, count)
}

/// Compare two packed values: true if a < b.
const fn lt_packed(a: [u8; 8], a_count: u8, b: [u8; 8], b_count: u8) -> bool {
    // Different lengths (after normalization): shorter is smaller
    if a_count != b_count {
        return a_count < b_count;
    }
    // Same length: compare from MSB
    if a_count == 0 {
        return false; // both zero
    }
    let mut i = a_count;
    while i > 0 {
        i -= 1;
        let at = trit_at_packed(a, i);
        let bt = trit_at_packed(b, i);
        if at < bt { return true; }
        if at > bt { return false; }
    }
    false // equal
}

/// Compare two packed values for equality.
const fn eq_packed(a: [u8; 8], a_count: u8, b: [u8; 8], b_count: u8) -> bool {
    if a_count != b_count { return false; }
    let mut i: u8 = 0;
    while i < a_count {
        if trit_at_packed(a, i) != trit_at_packed(b, i) { return false; }
        i += 1;
    }
    true
}

/// Convert packed trits to u64. Panics on overflow.
const fn to_u64_raw(packed: [u8; 8], count: u8) -> u64 {
    let mut result: u64 = 0;
    let mut power: u64 = 1;
    let mut i: u8 = 0;
    while i < count {
        result += trit_at_packed(packed, i) as u64 * power;
        power *= 3;
        i += 1;
    }
    result
}

/// Convert u64 to packed trits.
const fn from_u64_raw(mut val: u64) -> ([u8; 8], u8) {
    if val == 0 {
        return ([0u8; 8], 0);
    }
    let mut packed = [0u8; 8];
    let mut count: u8 = 0;
    while val > 0 {
        let trit = (val % 3) as u8;
        let byte_idx = (count / 5) as usize;
        let trit_idx = (count % 5) as usize;
        packed[byte_idx] += trit * POW3[trit_idx];
        count += 1;
        val /= 3;
    }
    assert!(count <= MAX_INLINE_TRITS, "from_u64: value exceeds R4 = 40 trits inline capacity");
    (packed, count)
}

// ══════════════════════════════════════════════════════════════
// RUNTIME DYNAMIC HELPERS (Phase 6)
//
// Slice-based versions of the const helpers for heap-sized values.
// Same packing (5 trits/byte, LSB-first), but operate on &[u8]
// slices of arbitrary length with u32 positions.
//
// All runtime arithmetic goes through these. The const helpers
// above remain for compile-time computation (const_add etc.).
// ══════════════════════════════════════════════════════════════

/// Extract a trit from a packed slice. Returns 0 for out-of-bounds.
fn trit_at_gen(packed: &[u8], pos: u32) -> u8 {
    let byte_idx = (pos / 5) as usize;
    let trit_idx = (pos % 5) as usize;
    if byte_idx >= packed.len() { return 0; }
    (packed[byte_idx] / POW3[trit_idx]) % 3
}

/// Pack individual trits (LSB-first) into bytes.
fn pack_trits(trits: &[u8]) -> Vec<u8> {
    let byte_count = (trits.len() + 4) / 5;
    let mut packed = vec![0u8; byte_count];
    for (i, &t) in trits.iter().enumerate() {
        packed[i / 5] += t * POW3[i % 5];
    }
    packed
}

/// Number of bytes needed for n trits.
fn bytes_for_trits(n: u32) -> usize { ((n as usize) + 4) / 5 }

/// Build a TritInt from packed bytes and trit count. Auto-selects inline or heap.
/// Normalizes (strips leading zero trits).
fn make_from_packed(mut packed: Vec<u8>, mut count: u32) -> TritInt {
    // Strip leading zeros
    while count > 0 && trit_at_gen(&packed, count - 1) == 0 { count -= 1; }

    if count <= MAX_INLINE_TRITS as u32 {
        let mut inline = [0u8; 8];
        let copy_len = std::cmp::min(packed.len(), 8);
        inline[..copy_len].copy_from_slice(&packed[..copy_len]);
        // Clear bytes beyond last used
        let last_byte = if count == 0 { 0 } else { ((count - 1) / 5) as usize + 1 };
        for i in last_byte..8 { inline[i] = 0; }
        if count > 0 {
            let trits_in_last = ((count - 1) % 5) + 1;
            inline[last_byte - 1] %= POW3[trits_in_last as usize];
        }
        TritInt { storage: TritIntStorage::Inline { packed: inline, trit_count: count as u8 } }
    } else {
        packed.truncate(bytes_for_trits(count));
        TritInt { storage: TritIntStorage::Heap { packed: ManuallyDrop::new(packed), trit_count: count } }
    }
}

/// Build from individual trit values (LSB-first).
fn make_from_trits(trits: &[u8]) -> TritInt {
    let packed = pack_trits(trits);
    make_from_packed(packed, trits.len() as u32)
}

/// General addition on packed slices. No size limit.
fn add_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    let max = std::cmp::max(ac, bc);
    let mut result = Vec::with_capacity((max + 2) as usize);
    let mut carry: u8 = 0;
    for i in 0..max {
        let sum = trit_at_gen(a, i) + trit_at_gen(b, i) + carry;
        result.push(sum % 3);
        carry = sum / 3;
    }
    if carry > 0 { result.push(carry); }
    make_from_trits(&result)
}

/// General subtraction on packed slices. Panics on underflow.
fn sub_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    let max = std::cmp::max(ac, bc);
    let mut result = Vec::with_capacity(max as usize);
    let mut borrow: u8 = 0;
    for i in 0..max {
        let at = trit_at_gen(a, i);
        let bt = trit_at_gen(b, i) + borrow;
        let (digit, new_borrow) = if at >= bt {
            (at - bt, 0u8)
        } else {
            (at + 3 - bt, 1u8)
        };
        borrow = new_borrow;
        result.push(digit);
    }
    assert!(borrow == 0, "const_sub underflow: subtrahend > minuend");
    make_from_trits(&result)
}

/// General multiplication on packed slices. No size limit.
fn mul_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    if ac == 0 || bc == 0 { return TritInt::zero(); }
    let max_trits = (ac + bc) as usize;
    let mut result = vec![0u8; max_trits];
    for i in 0..ac {
        let a_trit = trit_at_gen(a, i);
        if a_trit == 0 { continue; }
        let mut carry: u8 = 0;
        for j in 0..bc {
            let pos = (i + j) as usize;
            let sum = a_trit * trit_at_gen(b, j) + result[pos] + carry;
            result[pos] = sum % 3;
            carry = sum / 3;
        }
        let mut k = (i + bc) as usize;
        while carry > 0 {
            let sum = result[k] + carry;
            result[k] = sum % 3;
            carry = sum / 3;
            k += 1;
        }
    }
    make_from_trits(&result)
}

/// General less-than on packed slices.
fn lt_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> bool {
    if ac != bc { return ac < bc; }
    if ac == 0 { return false; }
    let mut i = ac;
    while i > 0 {
        i -= 1;
        let at = trit_at_gen(a, i);
        let bt = trit_at_gen(b, i);
        if at < bt { return true; }
        if at > bt { return false; }
    }
    false
}

/// General equality on packed slices.
fn eq_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> bool {
    if ac != bc { return false; }
    for i in 0..ac {
        if trit_at_gen(a, i) != trit_at_gen(b, i) { return false; }
    }
    true
}

/// General conversion to u64. Returns None if value exceeds u64::MAX.
fn to_u64_gen(packed: &[u8], count: u32) -> Option<u64> {
    // 3^40 ≈ 1.22 × 10^19 < u64::MAX ≈ 1.84 × 10^19
    // Values > 40 trits always exceed u64::MAX (3^41 > u64::MAX)
    if count > 40 { return None; }
    let mut result: u64 = 0;
    let mut power: u64 = 1;
    for i in 0..count {
        result += trit_at_gen(packed, i) as u64 * power;
        if i < count - 1 { power *= 3; }
    }
    Some(result)
}

/// Shift trits left by `shift` positions (multiply by 3^shift) on packed slices.
fn trit_shift_left_gen(packed: &[u8], count: u32, shift: u32) -> (Vec<u8>, u32) {
    let new_count = count + shift;
    if count == 0 { return (vec![0u8; bytes_for_trits(new_count)], 0); }
    let mut result = vec![0u8; bytes_for_trits(new_count)];
    for i in 0..count {
        let trit = trit_at_gen(packed, i);
        let new_pos = (i + shift) as usize;
        result[new_pos / 5] += trit * POW3[new_pos % 5];
    }
    (result, new_count)
}

// ══════════════════════════════════════════════════════════════
// CONSTRUCTORS (all const fn)
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// The additive identity: zero.
    pub const fn zero() -> Self {
        TritInt { storage: TritIntStorage::Inline { packed: [0u8; 8], trit_count: 0 } }
    }

    /// The multiplicative identity: one = 1₃.
    pub const fn one() -> Self {
        TritInt { storage: TritIntStorage::Inline { packed: [1, 0, 0, 0, 0, 0, 0, 0], trit_count: 1 } }
    }

    /// Repunit R_n = 111...1₃ (n ones). R_n = (3ⁿ − 1) / 2.
    ///
    /// Every trit is 1. Packed: each complete group of 5 ones encodes as
    /// 1 + 3 + 9 + 27 + 81 = 121.
    pub const fn repunit(n: usize) -> Self {
        assert!(n as u8 <= MAX_INLINE_TRITS, "repunit: n exceeds R4 = 40 trits");
        let mut packed = [0u8; 8];
        let full_bytes = n / 5;
        let remaining = n % 5;
        let mut i = 0;
        while i < full_bytes {
            packed[i] = 121; // 1 + 3 + 9 + 27 + 81
            i += 1;
        }
        if remaining > 0 {
            let mut val: u8 = 0;
            let mut j = 0;
            while j < remaining {
                val += POW3[j];
                j += 1;
            }
            packed[full_bytes] = val;
        }
        TritInt { storage: TritIntStorage::Inline { packed, trit_count: n as u8 } }
    }

    /// Construct from a trit slice. Rep B {0, 1, 2}, least significant trit first.
    /// Panics if any trit value ≥ 3.
    pub const fn from_trits(trits: &[u8]) -> Self {
        let count = trits.len();
        assert!(count <= MAX_INLINE_TRITS as usize, "from_trits: exceeds R4 = 40 trits");

        let mut packed = [0u8; 8];
        let mut i = 0;
        while i < count {
            assert!(trits[i] < 3, "from_trits: trit value must be 0, 1, or 2 (Rep B)");
            let byte_idx = i / 5;
            let trit_idx = i % 5;
            packed[byte_idx] += trits[i] * POW3[trit_idx];
            i += 1;
        }
        let (p, c) = normalize(packed, count as u8);
        TritInt { storage: TritIntStorage::Inline { packed: p, trit_count: c } }
    }

    /// Construct from a u64 value (BOUNDARY CROSSING: binary → ternary).
    /// Converts by repeated division by 3.
    pub const fn from_u64(val: u64) -> Self {
        let (packed, count) = from_u64_raw(val);
        TritInt { storage: TritIntStorage::Inline { packed, trit_count: count } }
    }

    /// Construct from a u32 value (BOUNDARY CROSSING: binary → ternary).
    pub const fn from_u32(val: u32) -> Self {
        Self::from_u64(val as u64)
    }

    /// Construct from a u128 value (BOUNDARY CROSSING: binary → ternary).
    pub const fn from_u128(val: u128) -> Self {
        // u128 max = 3.4 × 10³⁸ ≈ 3⁸¹ — exceeds 40 trits for very large values.
        // Inline path handles values up to 3⁴⁰ ≈ 1.22 × 10¹⁹.
        if val > u64::MAX as u128 {
            // For Phase 1 (inline only), values > u64::MAX always exceed 40 trits.
            // This is conservative — some values between u64::MAX and 3⁴⁰ fit,
            // but that range is empty (3⁴⁰ < u64::MAX), so this is exact.
            panic!("from_u128: value exceeds R4 = 40 trit inline capacity");
        }
        Self::from_u64(val as u64)
    }

    // ── Internal extractors ─────────────────────────────────

    /// Extract packed array and count (const fn, inline-only, takes self by value).
    /// Panics on Heap — const fn cannot handle heap allocation.
    const fn into_parts(self) -> ([u8; 8], u8) {
        match self.storage {
            TritIntStorage::Inline { packed, trit_count } => (packed, trit_count),
            TritIntStorage::Heap { .. } => panic!("into_parts() called on heap TritInt"),
        }
    }

    /// Extract packed array and count from an inline value (runtime).
    /// Panics on Heap — use packed_slice()/count() for heap-compatible access.
    fn parts(&self) -> ([u8; 8], u8) {
        match &self.storage {
            TritIntStorage::Inline { packed, trit_count } => (*packed, *trit_count),
            TritIntStorage::Heap { .. } => panic!("parts() called on heap TritInt — use packed_slice()/count()"),
        }
    }

    /// Slice view of the packed bytes. Works for both inline and heap.
    fn packed_slice(&self) -> &[u8] {
        match &self.storage {
            TritIntStorage::Inline { packed, .. } => &packed[..],
            TritIntStorage::Heap { packed, .. } => &packed[..],
        }
    }

    /// Trit count as u32. Works for both inline and heap.
    fn count(&self) -> u32 {
        match &self.storage {
            TritIntStorage::Inline { trit_count, .. } => *trit_count as u32,
            TritIntStorage::Heap { trit_count, .. } => *trit_count,
        }
    }
}

// ══════════════════════════════════════════════════════════════
// CONST COMPARISON
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Const equality comparison.
    pub const fn const_eq(self, other: TritInt) -> bool {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        eq_packed(a, ac, b, bc)
    }

    /// Const less-than comparison. Compares trit-by-trit from MSB.
    pub const fn const_lt(self, other: TritInt) -> bool {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        lt_packed(a, ac, b, bc)
    }

    /// Const greater-than comparison.
    pub const fn const_gt(self, other: TritInt) -> bool {
        other.const_lt(self)
    }
}

// ══════════════════════════════════════════════════════════════
// CONST ARITHMETIC
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Const addition. Pure base-3 ripple carry. Panics if result > R₄ trits.
    pub const fn const_add(self, other: TritInt) -> TritInt {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        let (packed, count) = add_packed(a, ac, b, bc);
        make_inline(packed, count)
    }

    /// Const subtraction. Panics on underflow (other > self).
    pub const fn const_sub(self, other: TritInt) -> TritInt {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        let (packed, count) = sub_packed(a, ac, b, bc);
        make_inline(packed, count)
    }

    /// Const multiplication. Ternary long multiplication. Panics if result > R₄ trits.
    /// All framework constant derivations produce results under 25 trits.
    pub const fn const_mul(self, other: TritInt) -> TritInt {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        let (packed, count) = mul_packed(a, ac, b, bc);
        make_inline(packed, count)
    }
}

// ══════════════════════════════════════════════════════════════
// BOUNDARY CROSSINGS (the ONLY binary ↔ ternary interface)
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Convert to u32. Returns Err(Overflow(32)) if value > u32::MAX.
    pub fn to_u32(&self) -> Result<u32, Overflow> {
        match to_u64_gen(self.packed_slice(), self.count()) {
            Some(val) if val <= u32::MAX as u64 => Ok(val as u32),
            Some(_) => Err(Overflow(32)),
            None => Err(Overflow(32)),
        }
    }

    /// Convert to u64. Returns Err(Overflow(64)) if value > u64::MAX.
    pub fn to_u64(&self) -> Result<u64, Overflow> {
        to_u64_gen(self.packed_slice(), self.count()).ok_or(Overflow(64))
    }

    /// Convert to u128. Returns Err(Overflow(128)) if value exceeds range.
    pub fn to_u128(&self) -> Result<u128, Overflow> {
        // 3^80 ≈ 1.5 × 10^38 < u128::MAX ≈ 3.4 × 10^38
        // Values > 80 trits always exceed u128::MAX
        if self.count() > 80 { return Err(Overflow(128)); }
        let mut result: u128 = 0;
        let mut power: u128 = 1;
        for i in 0..self.count() {
            result += trit_at_gen(self.packed_slice(), i) as u128 * power;
            if i < self.count() - 1 {
                power = power.checked_mul(3).ok_or(Overflow(128))?;
            }
        }
        Ok(result)
    }

    /// Const conversion to u32. Panics on overflow. Inline-only.
    pub const fn to_u32_const(self) -> u32 {
        let (packed, count) = self.into_parts();
        let val = to_u64_raw(packed, count);
        assert!(val <= u32::MAX as u64, "to_u32_const: value exceeds u32::MAX");
        val as u32
    }

    /// Const conversion to u64. Inline-only.
    pub const fn to_u64_const(self) -> u64 {
        let (packed, count) = self.into_parts();
        to_u64_raw(packed, count)
    }

    /// Convenience: convert to u64, panic on overflow.
    pub fn to_decimal(&self) -> u64 {
        to_u64_gen(self.packed_slice(), self.count())
            .expect("to_decimal: value exceeds u64 range (use to_u128 for larger values)")
    }
}

// ══════════════════════════════════════════════════════════════
// RUNTIME ARITHMETIC
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Add two TritInts. Auto-promotes to heap if result > R₄ = 40 trits.
    pub fn add(&self, other: &TritInt) -> TritInt {
        add_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count())
    }

    /// Subtract other from self. Panics if other > self (unsigned underflow).
    pub fn sub(&self, other: &TritInt) -> TritInt {
        sub_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count())
    }

    /// Multiply two TritInts. Auto-promotes to heap if result > R₄ = 40 trits.
    pub fn mul(&self, other: &TritInt) -> TritInt {
        mul_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count())
    }

    /// Division with remainder: returns (quotient, remainder).
    /// Panics if divisor is zero. Works with heap-sized operands.
    pub fn div_mod(&self, divisor: &TritInt) -> (TritInt, TritInt) {
        assert!(!divisor.is_zero(), "div_mod: division by zero");

        if self.is_zero() {
            return (TritInt::zero(), TritInt::zero());
        }

        let ac = self.count();
        let bc = divisor.count();

        if lt_gen(self.packed_slice(), ac, divisor.packed_slice(), bc) {
            return (TritInt::zero(), self.clone());
        }

        let shift = (ac as i64) - (bc as i64);
        let mut remainder = self.clone();
        let mut quotient_trits = vec![0u8; (shift + 1) as usize];
        let mut q_count: u32 = 0;

        let mut i = shift;
        while i >= 0 {
            let pos = i as u32;
            let (sp, sc) = trit_shift_left_gen(divisor.packed_slice(), bc, pos);
            let doubled = add_gen(&sp, sc, &sp, sc);

            let rp = remainder.packed_slice();
            let rc = remainder.count();

            if !lt_gen(rp, rc, doubled.packed_slice(), doubled.count()) {
                quotient_trits[pos as usize] = 2;
                remainder = sub_gen(rp, rc, doubled.packed_slice(), doubled.count());
            } else if !lt_gen(rp, rc, &sp, sc) {
                quotient_trits[pos as usize] = 1;
                remainder = sub_gen(rp, rc, &sp, sc);
            }

            if quotient_trits[pos as usize] != 0 && pos >= q_count {
                q_count = pos + 1;
            }

            i -= 1;
        }

        let q = make_from_trits(&quotient_trits[..q_count as usize]);
        (q, remainder)
    }

    /// Exponentiation by repeated squaring.
    pub fn pow(&self, mut exp: u32) -> TritInt {
        if exp == 0 { return TritInt::one(); }
        let mut base = self.clone();
        let mut result = TritInt::one();
        while exp > 0 {
            if exp % 2 == 1 {
                result = TritInt::mul(&result, &base);
            }
            exp /= 2;
            if exp > 0 {
                base = TritInt::mul(&base, &base);
            }
        }
        result
    }

    /// Greatest common divisor (Euclidean algorithm).
    pub fn gcd(a: &TritInt, b: &TritInt) -> TritInt {
        let mut x = a.clone();
        let mut y = b.clone();
        while !y.is_zero() {
            let (_, rem) = x.div_mod(&y);
            x = y;
            y = rem;
        }
        x
    }

    /// Extended GCD: returns (gcd, x, y) where a*x - b*y = gcd or b*y - a*x = gcd.
    ///
    /// Since TritInt is unsigned, the Bézout coefficients are returned as positive
    /// values with an implicit sign. For the primary use case (CRT with coprime inputs
    /// where gcd = 1), x = modular inverse of a mod b.
    pub fn extended_gcd(a: &TritInt, b: &TritInt) -> (TritInt, TritInt, TritInt) {
        if b.is_zero() {
            return (a.clone(), TritInt::one(), TritInt::zero());
        }

        // Iterative extended GCD tracking sign separately
        let mut old_r = a.clone();
        let mut r = b.clone();
        let mut old_s = TritInt::one();
        let mut s = TritInt::zero();
        let mut old_s_neg = false;
        let mut s_neg = false;

        while !r.is_zero() {
            let (quotient, remainder) = old_r.div_mod(&r);

            old_r = r.clone();
            r = remainder;

            // new_s = old_s - quotient * s (with sign tracking)
            let qs = TritInt::mul(&quotient, &s);
            let (new_s, new_s_neg) = signed_sub(old_s, old_s_neg, qs, s_neg);
            old_s = s;
            old_s_neg = s_neg;
            s = new_s;
            s_neg = new_s_neg;
        }

        // Compute y = (gcd - a * x) / b (if gcd = a*x + b*y)
        // For coprime case (gcd=1): if x is positive, y = (a*x - 1) / b
        // For simplicity, compute y from the relation
        let ax = TritInt::mul(a, &old_s);
        let y = if !old_s_neg {
            if !ax.is_zero() && !old_r.is_zero() {
                let (q, rem) = TritInt::sub(&ax, &old_r).div_mod(b);
                if rem.is_zero() { q } else { TritInt::zero() }
            } else {
                TritInt::zero()
            }
        } else {
            let (q, rem) = TritInt::add(&old_r, &ax).div_mod(b);
            if rem.is_zero() { q } else { TritInt::zero() }
        };

        let final_x = if old_s_neg && !b.is_zero() {
            TritInt::sub(b, &old_s)
        } else {
            old_s
        };

        (old_r, final_x, y)
    }

    /// Addition with carry tracking metadata.
    pub fn add_with_carry(&self, other: &TritInt) -> TritIntAddResult {
        let ac = self.count();
        let bc = other.count();
        let max = std::cmp::max(ac, bc);

        let mut result_trits = Vec::with_capacity((max + 2) as usize);
        let mut carry: u8 = 0;
        let mut carry_count: u32 = 0;
        let mut max_carry_chain: u32 = 0;
        let mut current_chain: u32 = 0;

        let mut i: u32 = 0;
        while i < max || carry > 0 {
            let at = trit_at_gen(self.packed_slice(), i);
            let bt = trit_at_gen(other.packed_slice(), i);
            let sum = at + bt + carry;
            let new_carry = sum / 3;

            if new_carry > 0 {
                carry_count += 1;
                current_chain += 1;
                if current_chain > max_carry_chain {
                    max_carry_chain = current_chain;
                }
            } else {
                current_chain = 0;
            }
            carry = new_carry;
            result_trits.push(sum % 3);
            i += 1;
        }

        TritIntAddResult {
            value: make_from_trits(&result_trits),
            carry_count,
            max_carry_chain,
        }
    }
}

// ── Signed subtraction helper for extended_gcd ──────────────

/// Compute (a_val, a_neg) - (b_val, b_neg) with sign tracking.
fn signed_sub(a: TritInt, a_neg: bool, b: TritInt, b_neg: bool) -> (TritInt, bool) {
    // a - b where a has sign a_neg and b has sign b_neg
    // Negate b's sign: we compute a + (-b)
    let b_neg_flipped = !b_neg;
    signed_add(a, a_neg, b, b_neg_flipped)
}

/// Compute (a_val, a_neg) + (b_val, b_neg) with sign tracking.
fn signed_add(a: TritInt, a_neg: bool, b: TritInt, b_neg: bool) -> (TritInt, bool) {
    if a_neg == b_neg {
        (TritInt::add(&a, &b), a_neg)
    } else {
        if lt_gen(a.packed_slice(), a.count(), b.packed_slice(), b.count()) {
            (TritInt::sub(&b, &a), b_neg)
        } else if lt_gen(b.packed_slice(), b.count(), a.packed_slice(), a.count()) {
            (TritInt::sub(&a, &b), a_neg)
        } else {
            (TritInt::zero(), false)
        }
    }
}

/// Shift trits left by `shift` positions (multiply by 3^shift).
/// Inserts `shift` zero trits at the LSB end.
fn trit_shift_left(packed: [u8; 8], count: u8, shift: u8) -> ([u8; 8], u8) {
    let new_count = count + shift;
    assert!(new_count <= MAX_INLINE_TRITS, "trit_shift_left: result exceeds R4 = 40 trits");
    if count == 0 { return ([0u8; 8], 0); }

    let mut result = [0u8; 8];
    let mut i: u8 = 0;
    while i < count {
        let trit = trit_at_packed(packed, i);
        let new_pos = i + shift;
        let byte_idx = (new_pos / 5) as usize;
        let trit_idx = (new_pos % 5) as usize;
        result[byte_idx] += trit * POW3[trit_idx];
        i += 1;
    }
    (result, new_count)
}

// ══════════════════════════════════════════════════════════════
// ACCESSORS
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// True if the value is zero (no trits, or all trits are 0).
    pub fn is_zero(&self) -> bool {
        self.count() == 0
    }

    /// Number of significant trits (excluding leading zeros).
    pub fn trit_length(&self) -> usize {
        self.count() as usize
    }

    /// Get the trit at position i (0 = LSB). Returns 0 for positions beyond trit_length.
    pub fn trit_at(&self, i: usize) -> u8 {
        if i >= self.count() as usize { return 0; }
        trit_at_gen(self.packed_slice(), i as u32)
    }

    /// How many distinct trit values {0, 1, 2} appear in this number.
    pub fn trit_diversity(&self) -> u8 {
        let mut seen = [false; 3];
        for i in 0..self.count() {
            seen[trit_at_gen(self.packed_slice(), i) as usize] = true;
        }
        seen[0] as u8 + seen[1] as u8 + seen[2] as u8
    }

    /// True if all trits are 1 (repunit). Zero is not a repunit.
    pub fn is_repunit(&self) -> bool {
        if self.count() == 0 { return false; }
        for i in 0..self.count() {
            if trit_at_gen(self.packed_slice(), i) != 1 { return false; }
        }
        true
    }

    /// True if the value is a power of 3 (exactly one non-zero trit, which is 1).
    pub fn is_power_of_3(&self) -> bool {
        let c = self.count();
        if c == 0 { return false; }
        if trit_at_gen(self.packed_slice(), c - 1) != 1 { return false; }
        for i in 0..c - 1 {
            if trit_at_gen(self.packed_slice(), i) != 0 { return false; }
        }
        true
    }

    /// If this value is a power of 3, return the exponent. Otherwise None.
    pub fn ternary_exponent(&self) -> Option<u32> {
        if self.is_power_of_3() {
            Some(self.count() - 1)
        } else {
            None
        }
    }

    /// Extract all trits as a Vec, least significant first. Rep B {0, 1, 2}.
    pub fn to_trits(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.count() as usize);
        for i in 0..self.count() {
            result.push(trit_at_gen(self.packed_slice(), i));
        }
        result
    }

    /// Extract all trits as a Vec, most significant first. Rep B {0, 1, 2}.
    pub fn trits_msb_first(&self) -> Vec<u8> {
        let mut trits = self.to_trits();
        trits.reverse();
        trits
    }
}

// ══════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ══════════════════════════════════════════════════════════════

// ── Display: "210₃" ─────────────────────────────────────────

impl fmt::Display for TritInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self.count();
        if c == 0 {
            return write!(f, "0₃");
        }
        let mut i = c;
        while i > 0 {
            i -= 1;
            write!(f, "{}", trit_at_gen(self.packed_slice(), i))?;
        }
        write!(f, "₃")
    }
}

impl fmt::Debug for TritInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count() <= MAX_INLINE_TRITS as u32 {
            write!(f, "TritInt({} = {})", self, self.to_decimal())
        } else {
            write!(f, "TritInt({} [{} trits, heap])", self, self.count())
        }
    }
}

// ── PartialEq, Eq ───────────────────────────────────────────

impl PartialEq for TritInt {
    fn eq(&self, other: &Self) -> bool {
        eq_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count())
    }
}

impl Eq for TritInt {}

// ── PartialOrd, Ord ─────────────────────────────────────────

impl PartialOrd for TritInt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TritInt {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if eq_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count()) {
            std::cmp::Ordering::Equal
        } else if lt_gen(self.packed_slice(), self.count(), other.packed_slice(), other.count()) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

// ── Hash ────────────────────────────────────────────────────

impl Hash for TritInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let c = self.count();
        c.hash(state);
        let used_bytes = if c == 0 { 0 } else { ((c - 1) / 5) as usize + 1 };
        self.packed_slice()[..used_bytes].hash(state);
    }
}

// ── Operator traits ─────────────────────────────────────────

impl Add for TritInt {
    type Output = TritInt;
    fn add(self, rhs: TritInt) -> TritInt { TritInt::add(&self, &rhs) }
}

impl Add for &TritInt {
    type Output = TritInt;
    fn add(self, rhs: &TritInt) -> TritInt { TritInt::add(self, rhs) }
}

impl Sub for TritInt {
    type Output = TritInt;
    fn sub(self, rhs: TritInt) -> TritInt { TritInt::sub(&self, &rhs) }
}

impl Sub for &TritInt {
    type Output = TritInt;
    fn sub(self, rhs: &TritInt) -> TritInt { TritInt::sub(self, rhs) }
}

impl Mul for TritInt {
    type Output = TritInt;
    fn mul(self, rhs: TritInt) -> TritInt { TritInt::mul(&self, &rhs) }
}

impl Mul for &TritInt {
    type Output = TritInt;
    fn mul(self, rhs: &TritInt) -> TritInt { TritInt::mul(self, rhs) }
}

impl Div for TritInt {
    type Output = TritInt;
    fn div(self, rhs: TritInt) -> TritInt { TritInt::div_mod(&self, &rhs).0 }
}

impl Div for &TritInt {
    type Output = TritInt;
    fn div(self, rhs: &TritInt) -> TritInt { TritInt::div_mod(self, rhs).0 }
}

impl Rem for TritInt {
    type Output = TritInt;
    fn rem(self, rhs: TritInt) -> TritInt { TritInt::div_mod(&self, &rhs).1 }
}

impl Rem for &TritInt {
    type Output = TritInt;
    fn rem(self, rhs: &TritInt) -> TritInt { TritInt::div_mod(self, rhs).1 }
}

impl AddAssign for TritInt {
    fn add_assign(&mut self, rhs: TritInt) { *self = TritInt::add(self, &rhs); }
}

impl SubAssign for TritInt {
    fn sub_assign(&mut self, rhs: TritInt) { *self = TritInt::sub(self, &rhs); }
}

impl MulAssign for TritInt {
    fn mul_assign(&mut self, rhs: TritInt) { *self = TritInt::mul(self, &rhs); }
}

// ══════════════════════════════════════════════════════════════
// COMPILE-TIME CONST ASSERTIONS
//
// These verify the full axiom→constant derivation chain at build time.
// If any assertion fails, the build fails — no runtime check needed.
// ══════════════════════════════════════════════════════════════

const _: () = {
    // (a) R₆ = 364 = full circle
    assert!(TritInt::repunit(6).to_u32_const() == 364);

    // (b) π = 14 from Rep B LSB-first trits: 2×3⁰ + 1×3¹ + 1×3² = 14
    assert!(TritInt::from_trits(&[2, 1, 1]).to_u32_const() == 14);

    // (c) R₃ + 1 = π — verifies const_add + repunit + boundary crossing
    assert!(TritInt::repunit(3).const_add(TritInt::one()).to_u32_const() == 14);

    // (d) π × R₃ = half-turn — verifies const_mul + from_trits
    assert!(TritInt::from_trits(&[2, 1, 1]).const_mul(TritInt::repunit(3)).to_u32_const() == 182);

    // (e) 1 + 4 × 182 = 729 = 3⁶ = Δ₂ — discriminant derivation chain
    assert!(
        TritInt::one()
            .const_add(TritInt::from_u64(4).const_mul(TritInt::from_u64(182)))
            .to_u32_const() == 729
    );

    // (f) Verify all repunits R₁ through R₆ via recurrence R(n) = 3·R(n−1) + 1
    assert!(TritInt::repunit(1).to_u32_const() == 1);
    assert!(TritInt::repunit(2).to_u32_const() == 4);
    assert!(TritInt::repunit(3).to_u32_const() == 13);
    assert!(TritInt::repunit(4).to_u32_const() == 40);
    assert!(TritInt::repunit(5).to_u32_const() == 121);
    assert!(TritInt::repunit(6).to_u32_const() == 364);

    // Verify recurrence: R(n) = 3·R(n-1) + 1
    assert!(TritInt::from_u64(3).const_mul(TritInt::repunit(1)).const_add(TritInt::one()).to_u32_const() == 4);
    assert!(TritInt::from_u64(3).const_mul(TritInt::repunit(2)).const_add(TritInt::one()).to_u32_const() == 13);
    assert!(TritInt::from_u64(3).const_mul(TritInt::repunit(3)).const_add(TritInt::one()).to_u32_const() == 40);
    assert!(TritInt::from_u64(3).const_mul(TritInt::repunit(4)).const_add(TritInt::one()).to_u32_const() == 121);
    assert!(TritInt::from_u64(3).const_mul(TritInt::repunit(5)).const_add(TritInt::one()).to_u32_const() == 364);

    // (g) Negative path: R₅ ≠ R₆
    assert!(TritInt::repunit(5).to_u32_const() != 364);

    // Additional derivation chains from the manifold spec:

    // ARC_ROOT_SEMI = 182 = 14 × 13 = π × R₃
    assert!(TritInt::from_u64(14).const_mul(TritInt::from_u64(13)).to_u32_const() == 182);

    // ARC_ROOT_COMP = 650 = 832 − 182 (via subtraction)
    assert!(TritInt::from_u64(832).const_sub(TritInt::from_u64(182)).to_u32_const() == 650);

    // Vieta: 182 + 650 = 832
    assert!(TritInt::from_u64(182).const_add(TritInt::from_u64(650)).to_u32_const() == 832);

    // LCM_PRIMARY = 7 × 11 × 13 = 1001
    assert!(
        TritInt::from_u64(7)
            .const_mul(TritInt::from_u64(11))
            .const_mul(TritInt::from_u64(13))
            .to_u32_const() == 1001
    );

    // CENTER = (182 + 40) / 2 = 111 — verify via multiplication: 111 × 2 = 222 = 182 + 40
    assert!(
        TritInt::from_u64(182).const_add(TritInt::from_u64(40)).to_u32_const() == 222
    );

    // MAGIC_CONSTANT = 3 × 111 = 333
    assert!(TritInt::from_u64(3).const_mul(TritInt::from_u64(111)).to_u32_const() == 333);

    // 91 = 7 × 13 (ionization threshold = product of coprime generators)
    assert!(TritInt::from_u64(7).const_mul(TritInt::from_u64(13)).to_u32_const() == 91);

    // Const comparison verification
    assert!(TritInt::repunit(3).const_lt(TritInt::repunit(4)));
    assert!(!TritInt::repunit(4).const_lt(TritInt::repunit(3)));
    assert!(TritInt::repunit(3).const_eq(TritInt::from_u64(13)));
    assert!(TritInt::zero().const_eq(TritInt::zero()));
    assert!(!TritInt::zero().const_eq(TritInt::one()));

    // Phase 6: verify MAX_HEAP_TRITS is framework-derived (R₉ = (3⁹−1)/2)
    // Cannot use u32::pow in const on Rust 1.77, so verify via literal.
    // 3⁹ = 19683, (19683 - 1) / 2 = 9841.
    assert!(MAX_HEAP_TRITS == 9841);
};

// ══════════════════════════════════════════════════════════════
// REPRESENTATION CONVERSIONS (Phase 3)
//
// All four representations: A (balanced), B (standard), C (bijective),
// D (algebraic). All to_repr_* produce MSB-first output (wire convention).
// All from_repr_* accept MSB-first input. This is distinct from
// to_trits/from_trits which use LSB-first (internal convention).
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Convert to Rep A (balanced ternary, {−1, 0, +1}), MSB-first.
    ///
    /// Algorithm (LSB to MSB): for each digit + carry:
    /// 0→0 carry 0, 1→1 carry 0, 2→−1 carry 1, 3→0 carry 1.
    /// Zero-valued TritInt produces empty output.
    pub fn to_repr_a(&self) -> Vec<i8> {
        let c = self.count();
        if c == 0 { return Vec::new(); }

        let mut balanced = Vec::with_capacity(c as usize + 1);
        let mut carry: u8 = 0;

        for i in 0..c {
            let digit = trit_at_gen(self.packed_slice(), i) + carry;
            match digit {
                0 => { balanced.push(0i8); carry = 0; }
                1 => { balanced.push(1i8); carry = 0; }
                2 => { balanced.push(-1i8); carry = 1; }
                3 => { balanced.push(0i8); carry = 1; }
                _ => unreachable!(),
            }
        }
        if carry > 0 {
            balanced.push(1i8);
        }

        // Strip trailing zeros (which become leading zeros after reverse)
        while balanced.last() == Some(&0) {
            balanced.pop();
        }
        balanced.reverse(); // LSB-first → MSB-first
        balanced
    }

    /// Convert to Rep B (standard, {0, 1, 2}), MSB-first.
    ///
    /// Wire convention output. Distinct from `to_trits()` which is LSB-first.
    /// Zero-valued TritInt produces empty output.
    pub fn to_repr_b(&self) -> Vec<u8> {
        self.trits_msb_first()
    }

    /// Convert to Rep C (bijective, {1, 2, 3}), MSB-first.
    ///
    /// Each digit = Rep B digit + 1. Zero-valued TritInt produces empty output.
    /// No zero digits ever appear in the output — Rep C wire safety.
    pub fn to_repr_c(&self) -> Vec<u8> {
        let c = self.count();
        if c == 0 { return Vec::new(); }

        let mut result = Vec::with_capacity(c as usize);
        let mut i = c;
        while i > 0 {
            i -= 1;
            result.push(trit_at_gen(self.packed_slice(), i) + 1);
        }
        result
    }

    /// Convert to Rep D (algebraic, {Zero, One, Omega}), MSB-first.
    ///
    /// Per-digit mapping: Rep B 0→Zero, 1→One, 2→Omega.
    /// Zero-valued TritInt produces empty output.
    pub fn to_repr_d(&self) -> Vec<AlgebraicTrit> {
        let c = self.count();
        if c == 0 { return Vec::new(); }

        let mut result = Vec::with_capacity(c as usize);
        let mut i = c;
        while i > 0 {
            i -= 1;
            result.push(match trit_at_gen(self.packed_slice(), i) {
                0 => AlgebraicTrit::Zero,
                1 => AlgebraicTrit::One,
                2 => AlgebraicTrit::Omega,
                _ => unreachable!(),
            });
        }
        result
    }

    /// Construct from Rep A (balanced, {−1, 0, +1}), MSB-first input.
    ///
    /// Panics if any digit is outside {−1, 0, +1}.
    pub fn from_repr_a(balanced: &[i8]) -> Self {
        if balanced.is_empty() { return TritInt::zero(); }

        // Validate
        for &d in balanced {
            assert!(d >= -1 && d <= 1, "from_repr_a: digit must be -1, 0, or +1, got {}", d);
        }

        // Reverse to LSB-first and convert balanced → standard with carry
        let mut lsb_first: Vec<i8> = balanced.to_vec();
        lsb_first.reverse();

        let mut rep_b = Vec::with_capacity(lsb_first.len() + 1);
        let mut carry: i8 = 0;

        for &digit in &lsb_first {
            let val = digit + carry;
            if val < 0 {
                rep_b.push((val + 3) as u8);
                carry = -1;
            } else if val > 2 {
                rep_b.push((val - 3) as u8);
                carry = 1;
            } else {
                rep_b.push(val as u8);
                carry = 0;
            }
        }
        if carry > 0 {
            rep_b.push(carry as u8);
        } else if carry < 0 {
            // This shouldn't happen for valid balanced inputs, but handle defensively
            panic!("from_repr_a: conversion produced negative carry — invalid input");
        }

        TritInt::from_trits(&rep_b) // from_trits expects LSB-first Rep B
    }

    /// Construct from Rep B (standard, {0, 1, 2}), MSB-first input.
    ///
    /// The MSB-first counterpart of `from_trits` (which is LSB-first).
    /// Panics if any digit ≥ 3.
    pub fn from_repr_b(standard: &[u8]) -> Self {
        if standard.is_empty() { return TritInt::zero(); }

        for &d in standard {
            assert!(d < 3, "from_repr_b: digit must be 0, 1, or 2, got {}", d);
        }

        let mut lsb_first: Vec<u8> = standard.to_vec();
        lsb_first.reverse();
        TritInt::from_trits(&lsb_first)
    }

    /// Construct from Rep C (bijective, {1, 2, 3}), MSB-first input.
    ///
    /// Panics on any digit = 0 (forgery detection) or digit > 3.
    /// This is a structural validity check — impossible digit values indicate
    /// corrupted or forged input.
    pub fn from_repr_c(bijective: &[u8]) -> Self {
        if bijective.is_empty() { return TritInt::zero(); }

        for &d in bijective {
            assert!(d >= 1 && d <= 3, "from_repr_c: digit must be 1, 2, or 3, got {} — zero = forgery", d);
        }

        let mut lsb_first: Vec<u8> = bijective.iter().map(|&c| c - 1).collect();
        lsb_first.reverse();
        TritInt::from_trits(&lsb_first)
    }

    /// Construct from Rep D (algebraic, {Zero, One, Omega}), MSB-first input.
    pub fn from_repr_d(algebraic: &[AlgebraicTrit]) -> Self {
        if algebraic.is_empty() { return TritInt::zero(); }

        let mut lsb_first: Vec<u8> = algebraic.iter().map(|d| match d {
            AlgebraicTrit::Zero => 0u8,
            AlgebraicTrit::One => 1u8,
            AlgebraicTrit::Omega => 2u8,
        }).collect();
        lsb_first.reverse();
        TritInt::from_trits(&lsb_first)
    }
}

// ══════════════════════════════════════════════════════════════
// DIV_REPUNIT AND MOD_POW (Phase 3)
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Division by repunit R_n = 111...1₃ (n ones).
    ///
    /// Exploits repunit structure: 3ⁿ ≡ 1 (mod R_n), so the remainder
    /// equals (sum of n-trit chunks) mod R_n. The quotient is computed
    /// from the chunk processing with carry propagation.
    ///
    /// Returns (quotient, remainder).
    pub fn div_repunit(&self, n: usize) -> (TritInt, TritInt) {
        assert!(n > 0, "div_repunit: n must be > 0");

        if self.is_zero() {
            return (TritInt::zero(), TritInt::zero());
        }

        let divisor = TritInt::repunit(n);

        if *self < divisor {
            return (TritInt::zero(), self.clone());
        }

        // Chunk-based: split into n-trit groups from LSB, sum chunks for remainder.
        // Since 3^n ≡ 1 (mod R_n), value mod R_n = (sum of chunks) mod R_n.
        let trits = self.to_trits(); // LSB-first
        let num_chunks = (trits.len() + n - 1) / n;

        // Sum all n-trit chunks
        let mut chunk_sum = TritInt::zero();
        for c in 0..num_chunks {
            let start = c * n;
            let end = std::cmp::min(start + n, trits.len());
            let chunk = TritInt::from_trits(&trits[start..end]);
            chunk_sum = TritInt::add(&chunk_sum, &chunk);
        }

        // Reduce chunk_sum mod R_n (recursive — chunk_sum might be > R_n)
        let (_, remainder) = if chunk_sum >= divisor {
            chunk_sum.div_mod(&divisor)
        } else {
            (TritInt::zero(), chunk_sum)
        };

        // Compute quotient: (self - remainder) / R_n
        // This division is exact (remainder was computed mod R_n).
        let numerator = TritInt::sub(self, &remainder);
        let (quotient, check_rem) = numerator.div_mod(&divisor);
        assert!(check_rem.is_zero(), "div_repunit: internal error — inexact quotient");

        (quotient, remainder)
    }

    /// Modular exponentiation: self^exp mod modulus.
    ///
    /// Standard square-and-multiply. NOT constant-time — must NOT be used
    /// for cryptographic operations (all crypto uses TLSponge-385 or TL-DSA).
    pub fn mod_pow(&self, exp: &TritInt, modulus: &TritInt) -> TritInt {
        assert!(!modulus.is_zero(), "mod_pow: modulus must be non-zero");

        if exp.is_zero() {
            // x^0 mod m = 1 (for m > 1)
            return if *modulus > TritInt::one() {
                TritInt::one()
            } else {
                TritInt::zero() // x^0 mod 1 = 0
            };
        }

        // Convert exponent to binary for square-and-multiply
        // (exponent bit extraction is a binary operation — this is
        // a boundary crossing, justified because the algorithm itself
        // is binary in structure regardless of the number base)
        let exp_val = exp.to_decimal();
        let mut result = TritInt::one();
        let mut base = self.div_mod(modulus).1; // reduce base mod modulus

        let mut e = exp_val;
        while e > 0 {
            if e % 2 == 1 {
                result = TritInt::mul(&result, &base).div_mod(modulus).1;
            }
            e /= 2;
            if e > 0 {
                base = TritInt::mul(&base, &base).div_mod(modulus).1;
            }
        }

        result
    }
}

// ══════════════════════════════════════════════════════════════
// PHASE 5: TritIntError + try_from_repr_c
//
// Result-based Rep C parsing for untrusted wire input. Deferred
// from Phase 3 (which uses panic-based from_repr_c for internal
// trust boundaries). This is the single enforcement point for
// input validation — both Serde and WASM callers go through here.
// ══════════════════════════════════════════════════════════════

/// Error type for TritInt parsing operations.
///
/// Separate from `Overflow` (which is arithmetic — value exceeds target
/// binary width). TritIntError covers input validation — malformed or
/// oversized trit data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TritIntError {
    /// A digit in the Rep C input is invalid (0 = forgery, or > 3).
    InvalidDigit(u8),
    /// The input exceeds the inline capacity (R₄ = 40 trits).
    /// Phase 6 heap path will accept larger inputs.
    TooLong,
}

impl fmt::Display for TritIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TritIntError::InvalidDigit(d) => write!(f, "invalid Rep C digit: {} (must be 1, 2, or 3; zero = forgery)", d),
            TritIntError::TooLong => write!(f, "input exceeds TritInt maximum capacity (R₉ = {} trits)", MAX_HEAP_TRITS),
        }
    }
}

impl std::error::Error for TritIntError {}

impl TritInt {
    /// Parse Rep C input (MSB-first) into a TritInt, returning Result.
    ///
    /// This is the single enforcement point for untrusted input validation.
    /// Both Serde and WASM callers go through this function.
    ///
    /// - Empty input → Ok(TritInt::zero())
    /// - Zero digit → Err(InvalidDigit(0)) — forgery detection
    /// - Digit > 3 → Err(InvalidDigit(d))
    /// - Input > R₉ = 9,841 trits → Err(TooLong)
    pub fn try_from_repr_c(bijective: &[u8]) -> Result<Self, TritIntError> {
        if bijective.is_empty() {
            return Ok(TritInt::zero());
        }

        if bijective.len() > MAX_HEAP_TRITS as usize {
            return Err(TritIntError::TooLong);
        }

        for &d in bijective {
            if d == 0 || d > 3 {
                return Err(TritIntError::InvalidDigit(d));
            }
        }

        // Valid — convert Rep C MSB-first to Rep B LSB-first
        let mut lsb_first: Vec<u8> = bijective.iter().map(|&c| c - 1).collect();
        lsb_first.reverse();

        if lsb_first.len() <= MAX_INLINE_TRITS as usize {
            Ok(TritInt::from_trits(&lsb_first))
        } else {
            Ok(make_from_trits(&lsb_first))
        }
    }
}

// ══════════════════════════════════════════════════════════════
// PHASE 5: ZEROIZE
//
// Cryptographic erasure for TritInt values that may contain key
// material. Zeros the packed buffer and resets the trit count.
// Heap Vec<u8> is deallocated on drop; call .zeroize() explicitly for sensitive erasure.
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Explicitly drop a heap-allocated TritInt, zeroing and deallocating the buffer.
    /// No-op for inline values. Must be called before heap TritInts go out of scope
    /// to prevent memory leaks (ManuallyDrop suppresses automatic Drop for const compatibility).
    pub fn drop_heap(&mut self) {
        if matches!(self.storage, TritIntStorage::Heap { .. }) {
            if let TritIntStorage::Heap { packed, trit_count } = &mut self.storage {
                packed.zeroize();
                *trit_count = 0;
                unsafe { ManuallyDrop::drop(packed); }
            }
            self.storage = TritIntStorage::Inline { packed: [0u8; 8], trit_count: 0 };
        }
    }
}

impl Zeroize for TritInt {
    fn zeroize(&mut self) {
        let was_heap = matches!(self.storage, TritIntStorage::Heap { .. });
        match &mut self.storage {
            TritIntStorage::Inline { packed, trit_count } => {
                packed.zeroize();
                *trit_count = 0;
            }
            TritIntStorage::Heap { packed, trit_count } => {
                packed.zeroize();
                *trit_count = 0;
                unsafe { ManuallyDrop::drop(packed); }
            }
        }
        if was_heap {
            self.storage = TritIntStorage::Inline { packed: [0u8; 8], trit_count: 0 };
        }
    }
}


// ══════════════════════════════════════════════════════════════
// PHASE 5: SERDE (behind #[cfg(feature = "serde")])
//
// TritInt serializes as a Rep C array (MSB-first u8 values).
// Zero-valued TritInt → empty array [].
// Deserialization uses try_from_repr_c — never panics.
// ══════════════════════════════════════════════════════════════

#[cfg(feature = "serde")]
impl serde::Serialize for TritInt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_repr_c().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TritInt {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr_c: Vec<u8> = <Vec<u8> as serde::Deserialize>::deserialize(deserializer)?;
        TritInt::try_from_repr_c(&repr_c).map_err(serde::de::Error::custom)
    }
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Packing round-trips ─────────────────────────────────

    #[test]
    fn pack_unpack_single_trits() {
        for trit in 0..3u8 {
            let t = TritInt::from_trits(&[trit]);
            assert_eq!(t.trit_at(0), trit);
            assert_eq!(t.trit_length(), if trit == 0 { 0 } else { 1 });
        }
    }

    #[test]
    fn pack_unpack_full_byte() {
        // 5 trits pack into one byte
        let trits = [2, 1, 0, 2, 1]; // 2 + 3 + 0 + 54 + 81 = 140
        let t = TritInt::from_trits(&trits);
        for i in 0..5 {
            assert_eq!(t.trit_at(i), trits[i], "trit {} mismatch", i);
        }
        assert_eq!(t.trit_length(), 5);
    }

    #[test]
    fn pack_unpack_multi_byte() {
        // 7 trits span two bytes
        let trits = [1, 2, 0, 1, 2, 1, 1];
        let t = TritInt::from_trits(&trits);
        for i in 0..7 {
            assert_eq!(t.trit_at(i), trits[i], "trit {} mismatch", i);
        }
    }

    #[test]
    fn pack_unpack_all_243_byte_values() {
        // Exhaustive: every possible 5-trit combination packs/unpacks correctly
        for val in 0..243u8 {
            let t0 = val % 3;
            let t1 = (val / 3) % 3;
            let t2 = (val / 9) % 3;
            let t3 = (val / 27) % 3;
            let t4 = (val / 81) % 3;
            let trits = [t0, t1, t2, t3, t4];
            let t = TritInt::from_trits(&trits);
            for i in 0..5 {
                assert_eq!(t.trit_at(i), trits[i], "byte {} trit {} failed", val, i);
            }
        }
    }

    #[test]
    fn pack_unpack_max_inline() {
        // 40 trits (max inline)
        let mut trits = [0u8; 40];
        for i in 0..40 {
            trits[i] = (i % 3) as u8;
        }
        let t = TritInt::from_trits(&trits);
        for i in 0..40 {
            assert_eq!(t.trit_at(i), trits[i], "trit {} mismatch", i);
        }
    }

    // ── Constructors ────────────────────────────────────────

    #[test]
    fn zero_is_zero() {
        let z = TritInt::zero();
        assert!(z.is_zero());
        assert_eq!(z.trit_length(), 0);
        assert_eq!(z.to_decimal(), 0);
    }

    #[test]
    fn one_is_one() {
        let o = TritInt::one();
        assert!(!o.is_zero());
        assert_eq!(o.trit_length(), 1);
        assert_eq!(o.trit_at(0), 1);
        assert_eq!(o.to_decimal(), 1);
    }

    #[test]
    fn repunits_match_formula() {
        // R_n = (3^n - 1) / 2
        let mut power: u64 = 1;
        for n in 1..=12 {
            power *= 3;
            let expected = (power - 1) / 2;
            let r = TritInt::repunit(n);
            assert_eq!(r.to_decimal(), expected, "repunit({}) wrong", n);
            assert!(r.is_repunit(), "repunit({}) should be repunit", n);
        }
    }

    #[test]
    fn repunit_trits_are_all_ones() {
        for n in 1..=8 {
            let r = TritInt::repunit(n);
            for i in 0..n {
                assert_eq!(r.trit_at(i), 1, "repunit({}) trit {} should be 1", n, i);
            }
        }
    }

    // ── Boundary crossing round-trips ───────────────────────

    #[test]
    fn u64_round_trip() {
        let test_values: [u64; 10] = [0, 1, 2, 3, 13, 14, 91, 182, 364, 118_300];
        for &val in &test_values {
            let t = TritInt::from_u64(val);
            assert_eq!(t.to_decimal(), val, "round-trip failed for {}", val);
        }
    }

    #[test]
    fn u32_round_trip() {
        let t = TritInt::from_u32(364);
        assert_eq!(t.to_u32().unwrap(), 364);
    }

    #[test]
    fn u128_round_trip() {
        let t = TritInt::from_u128(1001);
        assert_eq!(t.to_u128().unwrap(), 1001);
    }

    #[test]
    fn overflow_u32() {
        let big = TritInt::from_u64(u32::MAX as u64 + 1);
        assert!(big.to_u32().is_err());
        assert_eq!(big.to_u32().unwrap_err(), Overflow(32));
    }

    #[test]
    fn framework_constants_boundary() {
        // Every constant in constants.rs fits in inline storage
        let constants: [(u64, &str); 10] = [
            (1, "REPUNIT_1"), (4, "REPUNIT_2"), (13, "REPUNIT_3"),
            (40, "REPUNIT_4"), (121, "REPUNIT_5"), (364, "REPUNIT_6"),
            (14, "ROOT_X1"), (26, "ROOT_X2"), (182, "ARC_ROOT_SEMI"),
            (650, "ARC_ROOT_COMP"),
        ];
        for &(val, name) in &constants {
            let t = TritInt::from_u64(val);
            assert_eq!(t.to_decimal(), val, "{} round-trip failed", name);
            assert!(t.trit_length() <= 40, "{} exceeds inline", name);
        }
    }

    // ── Runtime arithmetic ──────────────────────────────────

    #[test]
    fn add_basic() {
        let a = TritInt::from_u64(13);
        let b = TritInt::from_u64(1);
        assert_eq!((&a + &b).to_decimal(), 14);
    }

    #[test]
    fn add_with_carry_propagation() {
        // 2₃ + 1₃ = 10₃ (carry)
        let a = TritInt::from_trits(&[2]);
        let b = TritInt::from_trits(&[1]);
        let result = &a + &b;
        assert_eq!(result.to_decimal(), 3);
        assert_eq!(result.trit_at(0), 0);
        assert_eq!(result.trit_at(1), 1);
    }

    #[test]
    fn sub_basic() {
        let a = TritInt::from_u64(182);
        let b = TritInt::from_u64(14);
        assert_eq!((&a - &b).to_decimal(), 168);
    }

    #[test]
    fn sub_to_zero() {
        let a = TritInt::from_u64(364);
        let b = TritInt::from_u64(364);
        assert!((&a - &b).is_zero());
    }

    #[test]
    fn mul_basic() {
        let a = TritInt::from_u64(14);
        let b = TritInt::from_u64(13);
        assert_eq!((&a * &b).to_decimal(), 182);
    }

    #[test]
    fn mul_by_zero() {
        let a = TritInt::from_u64(364);
        let b = TritInt::zero();
        assert!((&a * &b).is_zero());
    }

    #[test]
    fn mul_by_one() {
        let a = TritInt::from_u64(364);
        let b = TritInt::one();
        assert_eq!((&a * &b).to_decimal(), 364);
    }

    #[test]
    fn mul_framework_products() {
        // 7 × 11 × 13 = 1001
        let result = &(&TritInt::from_u64(7) * &TritInt::from_u64(11)) * &TritInt::from_u64(13);
        assert_eq!(result.to_decimal(), 1001);

        // 182 × 650 = 118300
        let result = &TritInt::from_u64(182) * &TritInt::from_u64(650);
        assert_eq!(result.to_decimal(), 118_300);
    }

    #[test]
    fn div_mod_basic() {
        let a = TritInt::from_u64(182);
        let b = TritInt::from_u64(13);
        let (q, r) = a.div_mod(&b);
        assert_eq!(q.to_decimal(), 14);
        assert_eq!(r.to_decimal(), 0);
    }

    #[test]
    fn div_mod_with_remainder() {
        let a = TritInt::from_u64(365);
        let b = TritInt::from_u64(364);
        let (q, r) = a.div_mod(&b);
        assert_eq!(q.to_decimal(), 1);
        assert_eq!(r.to_decimal(), 1);
    }

    #[test]
    fn div_mod_framework_values() {
        // 118300 / 182 = 650
        let (q, r) = TritInt::from_u64(118_300).div_mod(&TritInt::from_u64(182));
        assert_eq!(q.to_decimal(), 650);
        assert_eq!(r.to_decimal(), 0);

        // 118300 / 650 = 182
        let (q, r) = TritInt::from_u64(118_300).div_mod(&TritInt::from_u64(650));
        assert_eq!(q.to_decimal(), 182);
        assert_eq!(r.to_decimal(), 0);

        // 364 / 4 = 91
        let (q, r) = TritInt::from_u64(364).div_mod(&TritInt::from_u64(4));
        assert_eq!(q.to_decimal(), 91);
        assert_eq!(r.to_decimal(), 0);
    }

    #[test]
    fn div_mod_quotient_remainder_identity() {
        // For all a, b: a = q*b + r
        let pairs: [(u64, u64); 8] = [
            (364, 13), (729, 27), (1001, 7), (1001, 11),
            (15015, 143), (100, 3), (17, 5), (1, 1),
        ];
        for &(a, b) in &pairs {
            let ta = TritInt::from_u64(a);
            let tb = TritInt::from_u64(b);
            let (q, r) = ta.div_mod(&tb);
            let reconstructed = &(&q * &tb) + &r;
            assert_eq!(reconstructed.to_decimal(), a,
                "identity failed: {} / {} = {} rem {}", a, b, q.to_decimal(), r.to_decimal());
        }
    }

    #[test]
    fn pow_basic() {
        assert_eq!(TritInt::from_u64(3).pow(0).to_decimal(), 1);
        assert_eq!(TritInt::from_u64(3).pow(1).to_decimal(), 3);
        assert_eq!(TritInt::from_u64(3).pow(6).to_decimal(), 729);
        assert_eq!(TritInt::from_u64(2).pow(10).to_decimal(), 1024);
    }

    #[test]
    fn gcd_framework_values() {
        // gcd(182, 650) — 182 = 2×7×13, 650 = 2×5²×13 → gcd = 2×13 = 26
        let g = TritInt::gcd(&TritInt::from_u64(182), &TritInt::from_u64(650));
        assert_eq!(g.to_decimal(), 26);

        // gcd(7, 13) = 1 (coprime)
        let g = TritInt::gcd(&TritInt::from_u64(7), &TritInt::from_u64(13));
        assert_eq!(g.to_decimal(), 1);

        // gcd(11, 13) = 1 (AGS seed coprime)
        let g = TritInt::gcd(&TritInt::from_u64(11), &TritInt::from_u64(13));
        assert_eq!(g.to_decimal(), 1);

        // gcd(364, 1001) = 91 (364 = 2²×7×13, 1001 = 7×11×13 → gcd = 7×13 = 91)
        let g = TritInt::gcd(&TritInt::from_u64(364), &TritInt::from_u64(1001));
        assert_eq!(g.to_decimal(), 91);
    }

    #[test]
    fn extended_gcd_coprime() {
        // For coprime (11, 13): gcd = 1
        let (g, x, _y) = TritInt::extended_gcd(&TritInt::from_u64(11), &TritInt::from_u64(13));
        assert_eq!(g.to_decimal(), 1);
        // x is the modular inverse of 11 mod 13
        // 11 * x ≡ 1 (mod 13) → x = 6 (since 11*6 = 66 = 5*13 + 1)
        assert_eq!((&TritInt::from_u64(11) * &x).div_mod(&TritInt::from_u64(13)).1.to_decimal(), 1,
            "11 * {} mod 13 should be 1", x.to_decimal());
    }

    #[test]
    fn add_with_carry_metadata() {
        // 222₃ + 111₃ = 1110₃ (carries at every position)
        let a = TritInt::from_trits(&[2, 2, 2]);
        let b = TritInt::repunit(3);
        let result = a.add_with_carry(&b);
        assert_eq!(result.value.to_decimal(), 26 + 13); // 39
        assert!(result.carry_count > 0);
    }

    // ── Accessors ───────────────────────────────────────────

    #[test]
    fn trit_diversity_tests() {
        assert_eq!(TritInt::zero().trit_diversity(), 0);          // no trits
        assert_eq!(TritInt::repunit(5).trit_diversity(), 1);      // all 1s
        assert_eq!(TritInt::from_trits(&[1, 2]).trit_diversity(), 2); // 1 and 2
        assert_eq!(TritInt::from_trits(&[0, 1, 2]).trit_diversity(), 3); // all three
    }

    #[test]
    fn is_repunit_tests() {
        assert!(!TritInt::zero().is_repunit());
        assert!(TritInt::one().is_repunit());
        assert!(TritInt::repunit(6).is_repunit());
        assert!(!TritInt::from_u64(14).is_repunit()); // 112₃
    }

    #[test]
    fn is_power_of_3_tests() {
        assert!(TritInt::one().is_power_of_3());        // 3⁰ = 1
        assert!(TritInt::from_u64(3).is_power_of_3());  // 3¹
        assert!(TritInt::from_u64(9).is_power_of_3());  // 3²
        assert!(TritInt::from_u64(27).is_power_of_3()); // 3³
        assert!(TritInt::from_u64(729).is_power_of_3()); // 3⁶ = Δ₂
        assert!(!TritInt::from_u64(2).is_power_of_3());
        assert!(!TritInt::from_u64(14).is_power_of_3());
    }

    #[test]
    fn ternary_exponent_tests() {
        assert_eq!(TritInt::one().ternary_exponent(), Some(0));
        assert_eq!(TritInt::from_u64(3).ternary_exponent(), Some(1));
        assert_eq!(TritInt::from_u64(729).ternary_exponent(), Some(6));
        assert_eq!(TritInt::from_u64(14).ternary_exponent(), None);
    }

    #[test]
    fn to_trits_round_trip() {
        let original = [2, 1, 1, 0, 2];
        let t = TritInt::from_trits(&original);
        let recovered = t.to_trits();
        // Leading zero stripped: [2, 1, 1, 0, 2] → trit_count = 5 (MSB is 2, no strip)
        assert_eq!(recovered, original);
    }

    #[test]
    fn trits_msb_first() {
        let t = TritInt::from_trits(&[2, 1, 1]); // LSB-first: 112₃ = 14
        let msb = t.trits_msb_first();
        assert_eq!(msb, vec![1, 1, 2]); // MSB-first: 112₃
    }

    // ── Display ─────────────────────────────────────────────

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", TritInt::zero()), "0₃");
        assert_eq!(format!("{}", TritInt::one()), "1₃");
        assert_eq!(format!("{}", TritInt::from_u64(14)), "112₃");
        assert_eq!(format!("{}", TritInt::repunit(6)), "111111₃");
    }

    // ── Comparison and ordering ─────────────────────────────

    #[test]
    fn equality() {
        assert_eq!(TritInt::from_u64(14), TritInt::from_trits(&[2, 1, 1]));
        assert_eq!(TritInt::repunit(3), TritInt::from_u64(13));
        assert_ne!(TritInt::from_u64(13), TritInt::from_u64(14));
    }

    #[test]
    fn ordering() {
        assert!(TritInt::zero() < TritInt::one());
        assert!(TritInt::from_u64(13) < TritInt::from_u64(14));
        assert!(TritInt::from_u64(182) < TritInt::from_u64(650));
        assert!(TritInt::repunit(5) < TritInt::repunit(6));
    }

    #[test]
    fn leading_zero_normalization() {
        // from_trits with leading zeros should normalize
        let a = TritInt::from_trits(&[2, 1, 1, 0, 0]);
        let b = TritInt::from_trits(&[2, 1, 1]);
        assert_eq!(a, b);
        assert_eq!(a.trit_length(), 3);
    }

    // ── Operator traits ─────────────────────────────────────

    #[test]
    fn operator_add() {
        let a = TritInt::from_u64(13);
        let b = TritInt::from_u64(1);
        let c = a + b;
        assert_eq!(c.to_decimal(), 14);
    }

    #[test]
    fn operator_sub() {
        let a = TritInt::from_u64(182);
        let b = TritInt::from_u64(40);
        let c = a - b;
        assert_eq!(c.to_decimal(), 142);
    }

    #[test]
    fn operator_mul() {
        let c = TritInt::from_u64(7) * TritInt::from_u64(13);
        assert_eq!(c.to_decimal(), 91);
    }

    #[test]
    fn operator_div() {
        let c = TritInt::from_u64(364) / TritInt::from_u64(4);
        assert_eq!(c.to_decimal(), 91);
    }

    #[test]
    fn operator_rem() {
        let c = TritInt::from_u64(365) % TritInt::from_u64(364);
        assert_eq!(c.to_decimal(), 1);
    }

    #[test]
    fn operator_add_assign() {
        let mut a = TritInt::from_u64(13);
        a += TritInt::one();
        assert_eq!(a.to_decimal(), 14);
    }

    // ── Edge cases ──────────────────────────────────────────

    #[test]
    fn zero_arithmetic() {
        let z = TritInt::zero();
        assert_eq!((&z + &z).to_decimal(), 0);
        assert_eq!((&z * &TritInt::from_u64(100)).to_decimal(), 0);
        assert!(z.is_zero());
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TritInt::from_u64(14));
        assert!(set.contains(&TritInt::from_trits(&[2, 1, 1])));
    }

    // ── Should-panic tests ──────────────────────────────────

    #[test]
    #[should_panic(expected = "underflow")]
    fn sub_underflow_panics() {
        let a = TritInt::from_u64(5);
        let b = TritInt::from_u64(10);
        let _ = TritInt::sub(&a, &b);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn div_by_zero_panics() {
        let _ = TritInt::from_u64(14).div_mod(&TritInt::zero());
    }

    #[test]
    #[should_panic(expected = "trit value must be 0, 1, or 2")]
    fn invalid_trit_panics() {
        let _ = TritInt::from_trits(&[3]);
    }

    // ── Phase 3: Representation conversion tests ────────────

    #[test]
    fn repr_b_roundtrip() {
        let test_vals: [u64; 6] = [0, 1, 14, 182, 364, 729];
        for &val in &test_vals {
            let t = TritInt::from_u64(val);
            let repr = t.to_repr_b();
            let recovered = TritInt::from_repr_b(&repr);
            assert_eq!(recovered.to_decimal(), val, "Rep B roundtrip failed for {}", val);
        }
    }

    #[test]
    fn repr_a_roundtrip() {
        let test_vals: [u64; 6] = [0, 1, 14, 182, 364, 729];
        for &val in &test_vals {
            let t = TritInt::from_u64(val);
            let repr = t.to_repr_a();
            let recovered = TritInt::from_repr_a(&repr);
            assert_eq!(recovered.to_decimal(), val, "Rep A roundtrip failed for {}", val);
        }
    }

    #[test]
    fn repr_c_roundtrip() {
        let test_vals: [u64; 6] = [0, 1, 14, 182, 364, 729];
        for &val in &test_vals {
            let t = TritInt::from_u64(val);
            let repr = t.to_repr_c();
            let recovered = TritInt::from_repr_c(&repr);
            assert_eq!(recovered.to_decimal(), val, "Rep C roundtrip failed for {}", val);
        }
    }

    #[test]
    fn repr_d_roundtrip() {
        let test_vals: [u64; 6] = [0, 1, 14, 182, 364, 729];
        for &val in &test_vals {
            let t = TritInt::from_u64(val);
            let repr = t.to_repr_d();
            let recovered = TritInt::from_repr_d(&repr);
            assert_eq!(recovered.to_decimal(), val, "Rep D roundtrip failed for {}", val);
        }
    }

    #[test]
    fn repr_c_no_zero_digits() {
        let t = TritInt::from_u64(364);
        let repr = t.to_repr_c();
        for &digit in &repr {
            assert!(digit >= 1 && digit <= 3, "Rep C produced zero digit");
        }
    }

    #[test]
    fn repr_c_zero_produces_empty() {
        assert!(TritInt::zero().to_repr_c().is_empty());
    }

    #[test]
    fn repr_a_balanced_values() {
        // 14 = 112₃. Balanced: 1×9 + 1×3 + 2×1 = 14.
        // But 2 in balanced is T(−1) with carry: so 112₃ → 1 1 T₃ → carry propagation.
        // 2→T carry 1. Next: 1+1=2→T carry 1. Next: 1+1=2→T carry 1. Next: carry 1→1.
        // Result balanced: 1 T T T (MSB first) = 1×27 − 1×9 − 1×3 − 1×1 = 27−13 = 14. ✓
        let repr = TritInt::from_u64(14).to_repr_a();
        assert_eq!(repr, vec![1, -1, -1, -1]);
    }

    #[test]
    fn repr_b_msb_first() {
        // 14 = 112₃ → MSB-first: [1, 1, 2]
        let repr = TritInt::from_u64(14).to_repr_b();
        assert_eq!(repr, vec![1, 1, 2]);
    }

    #[test]
    fn repr_c_values() {
        // 14 = 112₃ → Rep C: [2, 2, 3]
        let repr = TritInt::from_u64(14).to_repr_c();
        assert_eq!(repr, vec![2, 2, 3]);
    }

    #[test]
    fn repr_d_values() {
        use crate::gf3_algebra::AlgebraicTrit::*;
        // 14 = 112₃ → Rep D MSB-first: [One, One, Omega]
        let repr = TritInt::from_u64(14).to_repr_d();
        assert_eq!(repr, vec![One, One, Omega]);
    }

    // ── Cross-representation 12-path round-trips ────────────

    #[test]
    fn all_12_paths_on_framework_constant() {
        // Test on 364 (R₆ = 111111₃)
        let t = TritInt::from_u64(364);
        let val = 364u64;

        // A→B→A
        let a = t.to_repr_a();
        assert_eq!(TritInt::from_repr_a(&a).to_decimal(), val);

        // B→C→B
        let b = t.to_repr_b();
        let c: Vec<u8> = b.iter().map(|&x| x + 1).collect();
        let b_back: Vec<u8> = c.iter().map(|&x| x - 1).collect();
        assert_eq!(TritInt::from_repr_b(&b_back).to_decimal(), val);

        // B→D→B
        let d = t.to_repr_d();
        assert_eq!(TritInt::from_repr_d(&d).to_decimal(), val);

        // A→C (composed)
        let a = t.to_repr_a();
        let from_a = TritInt::from_repr_a(&a);
        let c = from_a.to_repr_c();
        assert_eq!(TritInt::from_repr_c(&c).to_decimal(), val);

        // A→D (composed)
        let d = TritInt::from_repr_a(&a).to_repr_d();
        assert_eq!(TritInt::from_repr_d(&d).to_decimal(), val);

        // C→D (composed)
        let c = t.to_repr_c();
        let from_c = TritInt::from_repr_c(&c);
        let d = from_c.to_repr_d();
        assert_eq!(TritInt::from_repr_d(&d).to_decimal(), val);
    }

    // ── Forgery rejection tests ─────────────────────────────

    #[test]
    #[should_panic(expected = "forgery")]
    fn repr_c_rejects_zero_digit() {
        let _ = TritInt::from_repr_c(&[1, 0, 3]); // zero digit = forgery
    }

    #[test]
    #[should_panic(expected = "digit must be -1, 0, or +1")]
    fn repr_a_rejects_invalid_digit() {
        let _ = TritInt::from_repr_a(&[2]); // 2 is not a balanced digit
    }

    #[test]
    #[should_panic(expected = "digit must be 0, 1, or 2")]
    fn repr_b_rejects_invalid_digit() {
        let _ = TritInt::from_repr_b(&[3]);
    }

    // ── div_repunit tests ───────────────────────────────────

    #[test]
    fn div_repunit_correctness() {
        // Verify against general div_mod for R₁ through R₆
        let test_dividends: [u64; 5] = [364, 729, 1001, 15015, 118300];
        for n in 1..=6usize {
            let divisor = TritInt::repunit(n);
            for &dividend_val in &test_dividends {
                let dividend = TritInt::from_u64(dividend_val);
                let (q_opt, r_opt) = dividend.div_repunit(n);
                let (q_gen, r_gen) = dividend.div_mod(&divisor);
                assert_eq!(q_opt.to_decimal(), q_gen.to_decimal(),
                    "div_repunit quotient mismatch: {} / R_{}", dividend_val, n);
                assert_eq!(r_opt.to_decimal(), r_gen.to_decimal(),
                    "div_repunit remainder mismatch: {} / R_{}", dividend_val, n);
            }
        }
    }

    #[test]
    fn div_repunit_identity() {
        // R_n / R_n = 1 remainder 0
        for n in 1..=6usize {
            let r = TritInt::repunit(n);
            let (q, rem) = r.div_repunit(n);
            assert_eq!(q.to_decimal(), 1);
            assert_eq!(rem.to_decimal(), 0);
        }
    }

    // ── mod_pow tests ───────────────────────────────────────

    #[test]
    fn mod_pow_basic() {
        // 3^6 mod 100 = 729 mod 100 = 29
        let result = TritInt::from_u64(3).mod_pow(&TritInt::from_u64(6), &TritInt::from_u64(100));
        assert_eq!(result.to_decimal(), 29);
    }

    #[test]
    fn mod_pow_one_exponent() {
        // x^1 mod m = x mod m
        let result = TritInt::from_u64(14).mod_pow(&TritInt::one(), &TritInt::from_u64(10));
        assert_eq!(result.to_decimal(), 4);
    }

    #[test]
    fn mod_pow_zero_exponent() {
        // x^0 mod m = 1 (for m > 1)
        let result = TritInt::from_u64(14).mod_pow(&TritInt::zero(), &TritInt::from_u64(10));
        assert_eq!(result.to_decimal(), 1);
    }

    #[test]
    fn mod_pow_fermats_little() {
        // For prime p and a not divisible by p: a^(p-1) ≡ 1 (mod p)
        // 2^12 mod 13 = 1 (13 is prime)
        let result = TritInt::from_u64(2).mod_pow(&TritInt::from_u64(12), &TritInt::from_u64(13));
        assert_eq!(result.to_decimal(), 1);
    }

    // ── Phase 5: try_from_repr_c tests ──────────────────────

    #[test]
    fn try_from_repr_c_valid_roundtrip() {
        let test_vals: [u64; 5] = [0, 1, 14, 182, 364];
        for &val in &test_vals {
            let t = TritInt::from_u64(val);
            let repr = t.to_repr_c();
            let recovered = TritInt::try_from_repr_c(&repr).unwrap();
            assert_eq!(recovered.to_decimal(), val, "try_from_repr_c roundtrip failed for {}", val);
        }
    }

    #[test]
    fn try_from_repr_c_empty_returns_zero() {
        let t = TritInt::try_from_repr_c(&[]).unwrap();
        assert!(t.is_zero());
    }

    #[test]
    fn try_from_repr_c_rejects_zero_digit() {
        let result = TritInt::try_from_repr_c(&[1, 0, 3]);
        assert_eq!(result, Err(TritIntError::InvalidDigit(0)));
    }

    #[test]
    fn try_from_repr_c_rejects_digit_gt_3() {
        let result = TritInt::try_from_repr_c(&[1, 4, 2]);
        assert_eq!(result, Err(TritIntError::InvalidDigit(4)));
    }

    #[test]
    fn try_from_repr_c_rejects_too_long() {
        let input = vec![1u8; 9842]; // 9842 > R₉ = 9841
        let result = TritInt::try_from_repr_c(&input);
        assert_eq!(result, Err(TritIntError::TooLong));
    }

    #[test]
    fn try_from_repr_c_max_valid_length() {
        let input = vec![1u8; 40]; // exactly R₄ = 40 trits (inline)
        let result = TritInt::try_from_repr_c(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn try_from_repr_c_heap_sized() {
        let input = vec![2u8; 50]; // Rep C digit 2 → Rep B digit 1; 50 > 40 → heap
        let result = TritInt::try_from_repr_c(&input).unwrap();
        assert_eq!(result.trit_length(), 50);
        assert!(result.is_repunit());
    }

    // ── Phase 5: Zeroize test ───────────────────────────────

    #[test]
    fn zeroize_clears_value() {
        let mut t = TritInt::from_u64(118_300);
        assert!(!t.is_zero());
        t.zeroize();
        assert!(t.is_zero());
        assert_eq!(t.to_decimal(), 0);
    }

    // ── Phase 5: Serde tests ────────────────────────────────

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn serde_roundtrip_zero() {
            let t = TritInt::zero();
            let json = serde_json::to_string(&t).unwrap();
            assert_eq!(json, "[]");
            let back: TritInt = serde_json::from_str(&json).unwrap();
            assert_eq!(back, t);
        }

        #[test]
        fn serde_roundtrip_364() {
            let t = TritInt::from_u64(364);
            let json = serde_json::to_string(&t).unwrap();
            let back: TritInt = serde_json::from_str(&json).unwrap();
            assert_eq!(back, t);
            // Verify Rep C format: 364 = 111111₃ → Rep C = [2,2,2,2,2,2]
            assert_eq!(json, "[2,2,2,2,2,2]");
        }

        #[test]
        fn serde_roundtrip_14() {
            let t = TritInt::from_u64(14);
            let json = serde_json::to_string(&t).unwrap();
            // 14 = 112₃ → Rep C MSB-first = [2,2,3]
            assert_eq!(json, "[2,2,3]");
            let back: TritInt = serde_json::from_str(&json).unwrap();
            assert_eq!(back.to_decimal(), 14);
        }

        #[test]
        fn serde_deserialize_rejects_forgery() {
            let result: Result<TritInt, _> = serde_json::from_str("[1,0,3]");
            assert!(result.is_err(), "zero digit must be rejected");
        }
    }

    // ── Phase 6: Heap path tests ────────────────────────────

    #[test]
    fn large_mul_promotes_to_heap() {
        // Two 21-trit values: product is up to 42 trits > R₄ = 40
        let a = TritInt::from_u64(3u64.pow(20)); // 3^20 = 21 trits
        let b = TritInt::from_u64(3u64.pow(20));
        let result = TritInt::mul(&a, &b);
        // 3^20 × 3^20 = 3^40 — exactly 41 trits (1 followed by 40 zeros)
        assert_eq!(result.trit_length(), 41);
        assert!(result.is_power_of_3());
        assert_eq!(result.ternary_exponent(), Some(40));
    }

    #[test]
    fn pow_large_exponent() {
        // 3^25 has 26 trits. 3^25 × 3^25 = 3^50 has 51 trits.
        let base = TritInt::from_u64(3);
        let result = base.pow(50);
        assert_eq!(result.trit_length(), 51);
        assert!(result.is_power_of_3());
    }

    #[test]
    fn heap_shrinks_to_inline() {
        // Create a heap value, then divide to get an inline-sized result
        let big = TritInt::from_u64(3).pow(50); // 51 trits, heap
        assert!(big.trit_length() > 40);
        let (q, r) = big.div_mod(&TritInt::from_u64(3).pow(45));
        // q = 3^5 = 243, which is 6 trits (inline)
        assert_eq!(q.to_decimal(), 243);
        assert!(q.trit_length() <= 40);
        assert!(r.is_zero());
    }

    #[test]
    fn heap_inline_eq_and_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create 243 (= 3^5) via heap path (big value then divide)
        let big = TritInt::from_u64(3).pow(50);
        let (shrunk, _) = big.div_mod(&TritInt::from_u64(3).pow(45));

        // Create 243 directly (inline)
        let direct = TritInt::from_u64(243);

        // Must be equal
        assert_eq!(shrunk, direct);

        // Must have identical hashes
        let hash_of = |t: &TritInt| -> u64 {
            let mut h = DefaultHasher::new();
            t.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_of(&shrunk), hash_of(&direct));
    }

    #[test]
    fn heap_repr_c_roundtrip() {
        let big = TritInt::from_u64(3).pow(50); // 51 trits, heap
        let repr = big.to_repr_c();
        assert_eq!(repr.len(), 51);
        // First digit should be 2 (Rep C for trit 1: 1+1=2)
        assert_eq!(repr[0], 2); // MSB is 1 (power of 3), Rep C = 2
        // All others should be 1 (Rep C for trit 0: 0+1=1)
        for &d in &repr[1..] {
            assert_eq!(d, 1);
        }
        // Round-trip
        let back = TritInt::try_from_repr_c(&repr).unwrap();
        assert_eq!(back, big);
    }

    #[test]
    fn heap_add_sub_roundtrip() {
        let a = TritInt::from_u64(3).pow(45);
        let b = TritInt::from_u64(3).pow(44);
        let sum = TritInt::add(&a, &b);
        let diff = TritInt::sub(&sum, &b);
        assert_eq!(diff, a);
    }

    #[test]
    fn heap_comparison() {
        let a = TritInt::from_u64(3).pow(41); // 42 trits
        let b = TritInt::from_u64(3).pow(42); // 43 trits
        assert!(a < b);
        assert!(b > a);
        assert_ne!(a, b);
    }

    #[test]
    fn heap_display() {
        let val = TritInt::from_u64(3).pow(41);
        let s = format!("{}", val);
        // 3^41 in base 3 is "1" followed by 41 zeros, with ₃ suffix
        assert!(s.starts_with("1"));
        assert!(s.ends_with("₃"));
        assert_eq!(s.len(), 42 + "₃".len()); // 42 digits + subscript
    }

    #[test]
    fn zeroize_clears_heap() {
        let mut big = TritInt::from_u64(3).pow(50);
        assert!(!big.is_zero());
        big.zeroize();
        assert!(big.is_zero());
        assert_eq!(big.count(), 0);
    }

    #[test]
    fn heap_to_u128() {
        // 3^40 fits in u64 but 3^50 doesn't. Check to_u128 for 3^50.
        let val = TritInt::from_u64(3).pow(50);
        let result = val.to_u128();
        assert!(result.is_ok());
        // 3^50 = 717897987691852588770249
        let expected: u128 = 717_897_987_691_852_588_770_249;
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn heap_to_u64_overflow() {
        let val = TritInt::from_u64(3).pow(50); // > u64::MAX
        assert!(val.to_u64().is_err());
    }
}
