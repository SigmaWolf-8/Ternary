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
use std::ops::{Add, Sub, Mul, Div, Rem, AddAssign, SubAssign, MulAssign};

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

/// Powers of 3 within a single byte: 3⁰ through 3⁵.
/// Used for trit packing/unpacking. 3⁵ = 243 < 256 fits in u8.
const POW3: [u8; 6] = [1, 3, 9, 27, 81, 243];

/// Storage backend for TritInt.
#[derive(Clone)]
enum TritIntStorage {
    /// Inline: R₄/5 = 40/5 = 8 bytes, derived from trit capacity and packing ratio.
    /// trit_count is u8 — the smallest host integer type that holds R₄ = 40.
    Inline {
        packed: [u8; 8],
        trit_count: u8,
    },
    // Heap variant deferred to Phase 6.
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

    /// Extract packed array and count (const fn, takes self by value).
    const fn into_parts(self) -> ([u8; 8], u8) {
        match self.storage {
            TritIntStorage::Inline { packed, trit_count } => (packed, trit_count),
        }
    }

    /// Extract packed array and count from a reference (runtime only).
    fn parts(&self) -> ([u8; 8], u8) {
        match &self.storage {
            TritIntStorage::Inline { packed, trit_count } => (*packed, *trit_count),
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
        let val = self.to_u64_internal();
        if val > u32::MAX as u64 {
            Err(Overflow(32))
        } else {
            Ok(val as u32)
        }
    }

    /// Convert to u64. Returns Err(Overflow(64)) if value > u64::MAX.
    pub fn to_u64(&self) -> Result<u64, Overflow> {
        let (packed, count) = self.parts();
        // 40 trits = 3⁴⁰ ≈ 1.22 × 10¹⁹ < u64::MAX ≈ 1.84 × 10¹⁹
        // So all inline values fit in u64.
        Ok(to_u64_raw(packed, count))
    }

    /// Convert to u128. All inline values fit.
    pub fn to_u128(&self) -> Result<u128, Overflow> {
        Ok(self.to_u64_internal() as u128)
    }

    /// Const conversion to u32. Panics on overflow.
    pub const fn to_u32_const(self) -> u32 {
        let (packed, count) = self.into_parts();
        let val = to_u64_raw(packed, count);
        assert!(val <= u32::MAX as u64, "to_u32_const: value exceeds u32::MAX");
        val as u32
    }

    /// Const conversion to u64.
    pub const fn to_u64_const(self) -> u64 {
        let (packed, count) = self.into_parts();
        to_u64_raw(packed, count)
    }

    /// Convenience: convert to u64, panic on overflow.
    pub fn to_decimal(&self) -> u64 {
        self.to_u64_internal()
    }

    fn to_u64_internal(&self) -> u64 {
        let (packed, count) = self.parts();
        to_u64_raw(packed, count)
    }
}

// ══════════════════════════════════════════════════════════════
// RUNTIME ARITHMETIC
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Add two TritInts. Panics if result exceeds R₄ = 40 trits.
    pub fn add(&self, other: &TritInt) -> TritInt {
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        let (packed, count) = add_packed(a, ac, b, bc);
        make_inline(packed, count)
    }

    /// Subtract other from self. Panics if other > self (unsigned underflow).
    pub fn sub(&self, other: &TritInt) -> TritInt {
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        let (packed, count) = sub_packed(a, ac, b, bc);
        make_inline(packed, count)
    }

    /// Multiply two TritInts. Panics if result exceeds R₄ = 40 trits.
    pub fn mul(&self, other: &TritInt) -> TritInt {
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        let (packed, count) = mul_packed(a, ac, b, bc);
        make_inline(packed, count)
    }

    /// Division with remainder: returns (quotient, remainder).
    /// Panics if divisor is zero.
    pub fn div_mod(&self, divisor: &TritInt) -> (TritInt, TritInt) {
        assert!(!divisor.is_zero(), "div_mod: division by zero");

        if self.is_zero() {
            return (TritInt::zero(), TritInt::zero());
        }

        let (a, ac) = self.parts();
        let (b, bc) = divisor.parts();

        // If dividend < divisor, quotient = 0, remainder = dividend
        if lt_packed(a, ac, b, bc) {
            return (TritInt::zero(), self.clone());
        }

        // Base-3 long division
        let shift = (ac as i32) - (bc as i32);
        let mut remainder = self.clone();
        let mut quotient_trits = [0u8; 40];
        let mut q_count: u8 = 0;

        let mut i = shift;
        while i >= 0 {
            let pos = i as u8;
            // Compute divisor * 3^i by shifting trit positions
            let shifted = trit_shift_left(b, bc, pos);

            // Try digit = 2
            let doubled = {
                let (sp, sc) = shifted;
                add_packed(sp, sc, sp, sc)
            };
            let (rp, rc) = remainder.parts();
            let (sp, sc) = shifted;
            let (dp, dc) = doubled;

            if !lt_packed(rp, rc, dp, dc) {
                // remainder >= 2 * shifted_divisor
                quotient_trits[pos as usize] = 2;
                let (new_p, new_c) = sub_packed(rp, rc, dp, dc);
                remainder = make_inline(new_p, new_c);
            } else if !lt_packed(rp, rc, sp, sc) {
                // remainder >= shifted_divisor
                quotient_trits[pos as usize] = 1;
                let (new_p, new_c) = sub_packed(rp, rc, sp, sc);
                remainder = make_inline(new_p, new_c);
            }
            // else digit = 0, remainder unchanged

            if quotient_trits[pos as usize] != 0 && pos >= q_count {
                q_count = pos + 1;
            }

            i -= 1;
        }

        // Pack quotient
        let mut q_packed = [0u8; 8];
        let mut t: u8 = 0;
        while t < q_count {
            let byte_idx = (t / 5) as usize;
            let trit_idx = (t % 5) as usize;
            q_packed[byte_idx] += quotient_trits[t as usize] * POW3[trit_idx];
            t += 1;
        }

        (make_inline(q_packed, q_count), remainder)
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
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        let max = if ac > bc { ac } else { bc };

        let mut result = [0u8; 8];
        let mut carry: u8 = 0;
        let mut carry_count: u32 = 0;
        let mut max_carry_chain: u32 = 0;
        let mut current_chain: u32 = 0;
        let mut i: u8 = 0;
        let mut current_byte: u8 = 0;
        let mut trits_in_byte: u8 = 0;
        let mut byte_idx: usize = 0;

        while i < max || carry > 0 {
            let at = if i < ac { trit_at_packed(a, i) } else { 0 };
            let bt = if i < bc { trit_at_packed(b, i) } else { 0 };
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

            current_byte += (sum % 3) * POW3[trits_in_byte as usize];
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

        let (p, c) = normalize(result, i);
        TritIntAddResult {
            value: make_inline(p, c),
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
        let (ap, ac) = a.parts();
        let (bp, bc) = b.parts();
        if lt_packed(ap, ac, bp, bc) {
            (TritInt::sub(&b, &a), b_neg)
        } else if lt_packed(bp, bc, ap, ac) {
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
        let (_, count) = self.parts();
        count == 0
    }

    /// Number of significant trits (excluding leading zeros).
    pub fn trit_length(&self) -> usize {
        let (_, count) = self.parts();
        count as usize
    }

    /// Get the trit at position i (0 = LSB). Returns 0 for positions beyond trit_length.
    pub fn trit_at(&self, i: usize) -> u8 {
        let (packed, count) = self.parts();
        if i >= count as usize { return 0; }
        trit_at_packed(packed, i as u8)
    }

    /// How many distinct trit values {0, 1, 2} appear in this number.
    pub fn trit_diversity(&self) -> u8 {
        let (packed, count) = self.parts();
        let mut seen = [false; 3];
        let mut i: u8 = 0;
        while i < count {
            seen[trit_at_packed(packed, i) as usize] = true;
            i += 1;
        }
        seen[0] as u8 + seen[1] as u8 + seen[2] as u8
    }

    /// True if all trits are 1 (repunit). Zero is not a repunit.
    pub fn is_repunit(&self) -> bool {
        let (packed, count) = self.parts();
        if count == 0 { return false; }
        let mut i: u8 = 0;
        while i < count {
            if trit_at_packed(packed, i) != 1 { return false; }
            i += 1;
        }
        true
    }

    /// True if the value is a power of 3 (exactly one non-zero trit, which is 1).
    pub fn is_power_of_3(&self) -> bool {
        let (packed, count) = self.parts();
        if count == 0 { return false; }
        // The value is 3^(count-1) iff the MSB trit is 1 and all others are 0.
        if trit_at_packed(packed, count - 1) != 1 { return false; }
        let mut i: u8 = 0;
        while i < count - 1 {
            if trit_at_packed(packed, i) != 0 { return false; }
            i += 1;
        }
        true
    }

    /// If this value is a power of 3, return the exponent. Otherwise None.
    pub fn ternary_exponent(&self) -> Option<u32> {
        if self.is_power_of_3() {
            let (_, count) = self.parts();
            Some((count - 1) as u32)
        } else {
            None
        }
    }

    /// Extract all trits as a Vec, least significant first. Rep B {0, 1, 2}.
    pub fn to_trits(&self) -> Vec<u8> {
        let (packed, count) = self.parts();
        let mut result = Vec::with_capacity(count as usize);
        let mut i: u8 = 0;
        while i < count {
            result.push(trit_at_packed(packed, i));
            i += 1;
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
        let (packed, count) = self.parts();
        if count == 0 {
            return write!(f, "0₃");
        }
        let mut i = count;
        while i > 0 {
            i -= 1;
            write!(f, "{}", trit_at_packed(packed, i))?;
        }
        write!(f, "₃")
    }
}

impl fmt::Debug for TritInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TritInt({} = {})", self, self.to_decimal())
    }
}

// ── PartialEq, Eq ───────────────────────────────────────────
// Canonical form guaranteed by normalization, so byte comparison suffices.

impl PartialEq for TritInt {
    fn eq(&self, other: &Self) -> bool {
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        eq_packed(a, ac, b, bc)
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
        let (a, ac) = self.parts();
        let (b, bc) = other.parts();
        if eq_packed(a, ac, b, bc) {
            std::cmp::Ordering::Equal
        } else if lt_packed(a, ac, b, bc) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

// ── Hash ────────────────────────────────────────────────────

impl Hash for TritInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (packed, count) = self.parts();
        count.hash(state);
        // Hash only the bytes that contain valid trits
        let used_bytes = if count == 0 { 0 } else { ((count - 1) / 5) as usize + 1 };
        packed[..used_bytes].hash(state);
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
};

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
}
