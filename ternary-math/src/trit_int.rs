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
//! Larger values use heap allocation.
//!
//! **Storage:** one trit per slot. Each slot holds a Rep B atom from
//! the set {0, 1, 2}. There is no host-binary smuggling — the underlying
//! `u8` slot is just the host's narrowest addressable cell, used to carry
//! a single trit value. No multi-trit packing, no `3⁵ = 243 < 256` trick,
//! no powers-of-three table.
//!
//! **Trit ordering:** least significant trit first. Trit 0 sits in slot 0.
//!
//! **Internal representation:** Rep B {0, 1, 2}. Zero is the additive
//! identity. Rep C {1, 2, 3} is the wire format — conversion happens at
//! the boundary via `to_repr_c()` / `from_repr_c()` / `try_from_repr_c()`.
//!
//! **Position in the type chain:**
//! - `TritInt` — one ternary integer (this module)
//! - `Trit` — three TritInts: v\[0\] = ℤ, v\[1\] = φ, v\[2\] = ω (Phase 2)
//! - `[Trit; 3]` — one vertex coordinate in ℤ\[φ,ω\]
//! - Triangles, meshes, manifolds — built on Trit
//!
//! # Heap Path and Memory Management
//!
//! Dual-storage architecture: inline (≤ 40 trits, stack-only `[u8; 40]`)
//! and heap (> 40 trits, `Vec<u8>` allocation, one trit per byte). The
//! heap variant wraps its buffer in `ManuallyDrop` to preserve
//! const-compatibility — `TritInt` has no `Drop` impl, which allows
//! framework constants (all inline) to live in `const` items and `const fn`
//! accessors to remain `const`.
//!
//! **Heap-allocated TritInts will leak if dropped without cleanup.**
//! All heap values must be explicitly cleaned up via one of:
//! - `.zeroize()` — zeros the buffer and deallocates (preferred for crypto)
//! - `.drop_heap()` — zeros and deallocates, resets to inline zero
//!
//! Inline values (≤ 40 trits) have no cleanup requirement. All framework
//! constants and repunits are inline. Heap allocation only occurs through
//! runtime arithmetic on large operands or `try_from_repr_c` with > 40 digits.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use std::ops::{Add, Sub, Mul, Div, Rem, AddAssign, SubAssign, MulAssign};
use crate::repx::AlgebraicTrit;
use zeroize::Zeroize;

// ══════════════════════════════════════════════════════════════
// TYPE DEFINITIONS
// ══════════════════════════════════════════════════════════════

/// Result of addition with carry metadata.
pub struct TritIntAddResult {
    pub value: TritInt,
    pub carry_count: u32,
    pub max_carry_chain: u32,
}

/// Maximum trit count for the inline path.
/// R₄ = (3⁴ − 1)/2 = 40 — the four-digit repunit.
const MAX_INLINE_TRITS: u8 = 40;

/// Storage backend for TritInt.
///
/// One trit per slot in both variants. No binary packing — each slot
/// carries exactly one Rep B atom from {0, 1, 2}.
enum TritIntStorage {
    /// Inline: 40 trit slots on the stack.
    /// trit_count is u8 — the smallest host integer type that holds R₄ = 40.
    Inline {
        trits: [u8; 40],
        trit_count: u8,
    },
    /// Heap: arbitrary-length trit buffer. One trit per byte.
    /// ManuallyDrop keeps the enum const-compatible (no implicit Drop on TritInt).
    /// Cleanup: call .zeroize() or .drop_heap() before going out of scope.
    /// Framework constants are always Inline — no heap leak risk for const items.
    /// trit_count is u32 — host bookkeeping field (not a mathematical value).
    Heap {
        trits: ManuallyDrop<Vec<u8>>,
        trit_count: u32,
    },
}

impl Clone for TritIntStorage {
    fn clone(&self) -> Self {
        match self {
            TritIntStorage::Inline { trits, trit_count } =>
                TritIntStorage::Inline { trits: *trits, trit_count: *trit_count },
            TritIntStorage::Heap { trits, trit_count } =>
                TritIntStorage::Heap {
                    trits: ManuallyDrop::new((**trits).clone()),
                    trit_count: *trit_count,
                },
        }
    }
}

/// A ternary integer — one whole number stored in base 3.
///
/// One trit per slot. Internal representation is Rep B {0, 1, 2}.
/// Least significant trit first.
#[derive(Clone)]
pub struct TritInt {
    storage: TritIntStorage,
}

// ══════════════════════════════════════════════════════════════
// INTERNAL CONST HELPERS
//
// These operate on raw [u8; 40] slot arrays and counts. They exist
// because const fn cannot take &mut parameters — all mutation must
// be on local variables. The public const methods on TritInt extract
// arrays, call these helpers, and wrap the result.
// ══════════════════════════════════════════════════════════════

/// Normalize: strip leading zero trits and clear slots beyond trit_count.
/// Returns (trits, count) in canonical form. Two TritInts with the same
/// mathematical value always produce identical (trits, count).
const fn normalize_inline(mut trits: [u8; 40], count: u8) -> ([u8; 40], u8) {
    let mut c = count;
    while c > 0 && trits[(c - 1) as usize] == 0 {
        c -= 1;
    }
    let mut i = c as usize;
    while i < 40 {
        trits[i] = 0;
        i += 1;
    }
    (trits, c)
}

/// Pack a TritInt from a normalized slot array and count.
const fn make_inline(trits: [u8; 40], trit_count: u8) -> TritInt {
    let (t, c) = normalize_inline(trits, trit_count);
    TritInt { storage: TritIntStorage::Inline { trits: t, trit_count: c } }
}

/// Add two slot arrays, returning (result_trits, result_count).
const fn add_inline(
    a: [u8; 40], ac: u8,
    b: [u8; 40], bc: u8,
) -> ([u8; 40], u8) {
    let max = if ac > bc { ac } else { bc };
    let mut result = [0u8; 40];
    let mut carry: u8 = 0;
    let mut i: u8 = 0;

    while i < max || carry > 0 {
        let at = if i < ac { a[i as usize] } else { 0 };
        let bt = if i < bc { b[i as usize] } else { 0 };
        let sum = at + bt + carry;
        assert!((i as usize) < 40, "const_add: result exceeds R4 = 40 trits");
        result[i as usize] = sum % 3;
        carry = sum / 3;
        i += 1;
    }

    normalize_inline(result, i)
}

/// Subtract b from a (a ≥ b required), returning (result_trits, result_count).
const fn sub_inline(
    a: [u8; 40], ac: u8,
    b: [u8; 40], bc: u8,
) -> ([u8; 40], u8) {
    let max = if ac > bc { ac } else { bc };
    let mut result = [0u8; 40];
    let mut borrow: u8 = 0;
    let mut i: u8 = 0;

    while i < max {
        let at = if i < ac { a[i as usize] } else { 0 };
        let bt = if i < bc { b[i as usize] } else { 0 };
        let bt_plus = bt + borrow;
        let (digit, new_borrow) = if at >= bt_plus {
            (at - bt_plus, 0u8)
        } else {
            (at + 3 - bt_plus, 1u8)
        };
        borrow = new_borrow;
        result[i as usize] = digit;
        i += 1;
    }

    assert!(borrow == 0, "const_sub underflow: subtrahend > minuend");
    normalize_inline(result, max)
}

/// Schoolbook ternary long multiplication on slot arrays.
const fn mul_inline(
    a: [u8; 40], ac: u8,
    b: [u8; 40], bc: u8,
) -> ([u8; 40], u8) {
    let max_trits = ac + bc;
    assert!(max_trits <= MAX_INLINE_TRITS, "const_mul: result exceeds R4 = 40 trits");

    let mut result = [0u8; 40];
    let mut i: u8 = 0;
    while i < ac {
        let a_trit = a[i as usize];
        if a_trit != 0 {
            let mut carry: u8 = 0;
            let mut j: u8 = 0;
            while j < bc {
                let b_trit = b[j as usize];
                let pos = (i + j) as usize;
                let sum = a_trit * b_trit + result[pos] + carry;
                result[pos] = sum % 3;
                carry = sum / 3;
                j += 1;
            }
            let mut k = (i + bc) as usize;
            while carry > 0 {
                assert!(k < 40, "const_mul: carry overflow");
                let sum = result[k] + carry;
                result[k] = sum % 3;
                carry = sum / 3;
                k += 1;
            }
        }
        i += 1;
    }

    normalize_inline(result, max_trits)
}

/// Compare two slot arrays: true if a < b.
const fn lt_inline(a: [u8; 40], ac: u8, b: [u8; 40], bc: u8) -> bool {
    if ac != bc {
        return ac < bc;
    }
    if ac == 0 {
        return false;
    }
    let mut i = ac;
    while i > 0 {
        i -= 1;
        let at = a[i as usize];
        let bt = b[i as usize];
        if at < bt { return true; }
        if at > bt { return false; }
    }
    false
}

/// Compare two slot arrays for equality.
const fn eq_inline(a: [u8; 40], ac: u8, b: [u8; 40], bc: u8) -> bool {
    if ac != bc { return false; }
    let mut i: u8 = 0;
    while i < ac {
        if a[i as usize] != b[i as usize] { return false; }
        i += 1;
    }
    true
}

// ══════════════════════════════════════════════════════════════
// RUNTIME DYNAMIC HELPERS
//
// Slice-based versions of the const helpers for heap-sized values.
// One trit per byte in both directions, with u32 positions and
// arbitrary-length slices.
// ══════════════════════════════════════════════════════════════

/// Read a trit from a slot slice. Returns 0 for out-of-bounds.
fn trit_at_slice(trits: &[u8], pos: u32) -> u8 {
    let i = pos as usize;
    if i >= trits.len() { 0 } else { trits[i] }
}

/// Build a TritInt from a trit slice (Rep B {0,1,2}, LSB first).
/// Strips leading zeros and auto-selects inline / heap.
fn make_from_trits_lsb(trits: &[u8]) -> TritInt {
    let mut count = trits.len();
    while count > 0 && trits[count - 1] == 0 { count -= 1; }

    if count <= MAX_INLINE_TRITS as usize {
        let mut inline = [0u8; 40];
        let mut i = 0;
        while i < count {
            inline[i] = trits[i];
            i += 1;
        }
        TritInt { storage: TritIntStorage::Inline { trits: inline, trit_count: count as u8 } }
    } else {
        let mut v = trits[..count].to_vec();
        v.shrink_to_fit();
        TritInt { storage: TritIntStorage::Heap { trits: ManuallyDrop::new(v), trit_count: count as u32 } }
    }
}

/// General addition on trit slices. No size limit.
fn add_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    let max = std::cmp::max(ac, bc);
    let mut result = Vec::with_capacity((max + 1) as usize);
    let mut carry: u8 = 0;
    for i in 0..max {
        let sum = trit_at_slice(a, i) + trit_at_slice(b, i) + carry;
        result.push(sum % 3);
        carry = sum / 3;
    }
    if carry > 0 { result.push(carry); }
    make_from_trits_lsb(&result)
}

/// General subtraction on trit slices. Panics on underflow.
fn sub_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    let max = std::cmp::max(ac, bc);
    let mut result = Vec::with_capacity(max as usize);
    let mut borrow: u8 = 0;
    for i in 0..max {
        let at = trit_at_slice(a, i);
        let bt = trit_at_slice(b, i) + borrow;
        let (digit, new_borrow) = if at >= bt {
            (at - bt, 0u8)
        } else {
            (at + 3 - bt, 1u8)
        };
        borrow = new_borrow;
        result.push(digit);
    }
    assert!(borrow == 0, "sub underflow: subtrahend > minuend");
    make_from_trits_lsb(&result)
}

/// General multiplication on trit slices. No size limit.
fn mul_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> TritInt {
    if ac == 0 || bc == 0 { return TritInt::zero(); }
    let max_trits = (ac + bc) as usize;
    let mut result = vec![0u8; max_trits];
    for i in 0..ac {
        let a_trit = trit_at_slice(a, i);
        if a_trit == 0 { continue; }
        let mut carry: u8 = 0;
        for j in 0..bc {
            let pos = (i + j) as usize;
            let sum = a_trit * trit_at_slice(b, j) + result[pos] + carry;
            result[pos] = sum % 3;
            carry = sum / 3;
        }
        let mut k = (i + bc) as usize;
        while carry > 0 {
            if k >= result.len() { result.push(0); }
            let sum = result[k] + carry;
            result[k] = sum % 3;
            carry = sum / 3;
            k += 1;
        }
    }
    make_from_trits_lsb(&result)
}

/// General less-than on trit slices.
fn lt_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> bool {
    if ac != bc { return ac < bc; }
    if ac == 0 { return false; }
    let mut i = ac;
    while i > 0 {
        i -= 1;
        let at = trit_at_slice(a, i);
        let bt = trit_at_slice(b, i);
        if at < bt { return true; }
        if at > bt { return false; }
    }
    false
}

/// General equality on trit slices.
fn eq_gen(a: &[u8], ac: u32, b: &[u8], bc: u32) -> bool {
    if ac != bc { return false; }
    for i in 0..ac {
        if trit_at_slice(a, i) != trit_at_slice(b, i) { return false; }
    }
    true
}

/// Shift trits left by `shift` positions (multiply by 3^shift) on slot slices.
fn trit_shift_left_gen(trits: &[u8], count: u32, shift: u32) -> (Vec<u8>, u32) {
    if count == 0 { return (Vec::new(), 0); }
    let new_count = count + shift;
    let mut result = vec![0u8; new_count as usize];
    for i in 0..count {
        result[(i + shift) as usize] = trit_at_slice(trits, i);
    }
    (result, new_count)
}

// ══════════════════════════════════════════════════════════════
// CONSTRUCTORS
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// The additive identity: zero.
    pub const fn zero() -> Self {
        TritInt { storage: TritIntStorage::Inline { trits: [0u8; 40], trit_count: 0 } }
    }

    /// The multiplicative identity: one = 1₃.
    pub const fn one() -> Self {
        let mut trits = [0u8; 40];
        trits[0] = 1;
        TritInt { storage: TritIntStorage::Inline { trits, trit_count: 1 } }
    }

    /// Repunit R_n = 111...1₃ (n ones). R_n = (3ⁿ − 1) / 2.
    ///
    /// Every trit is 1.
    /// CONST PATH — limited to ≤ 40 trits by the inline storage.
    /// For n > 40, use `repunit_rt()`.
    pub const fn repunit(n: usize) -> Self {
        assert!(n as u8 <= MAX_INLINE_TRITS, "repunit: const path limited to R4 = 40 trits — use repunit_rt() for larger");
        let mut trits = [0u8; 40];
        let mut i = 0;
        while i < n {
            trits[i] = 1;
            i += 1;
        }
        TritInt { storage: TritIntStorage::Inline { trits, trit_count: n as u8 } }
    }

    /// Repunit R_n = 111...1₃ (n ones). R_n = (3ⁿ − 1) / 2.
    /// RUNTIME PATH — unbounded, auto-promotes to heap for n > 40.
    pub fn repunit_rt(n: usize) -> Self {
        if n <= MAX_INLINE_TRITS as usize {
            return Self::repunit(n);
        }
        let trits = vec![1u8; n];
        make_from_trits_lsb(&trits)
    }

    /// Construct from a trit slice (CONST PATH). Rep B {0, 1, 2}, LSB first.
    /// Limited to ≤ 40 trits by the inline storage.
    /// For unbounded runtime construction, use `from_trit_slice()`.
    /// Panics if any trit value ≥ 3 or if count > 40.
    pub const fn from_trits(trits: &[u8]) -> Self {
        let count = trits.len();
        assert!(count <= MAX_INLINE_TRITS as usize, "from_trits: const path limited to R4 = 40 trits — use from_trit_slice() for larger");

        let mut slots = [0u8; 40];
        let mut i = 0;
        while i < count {
            assert!(trits[i] < 3, "from_trits: trit value must be 0, 1, or 2 (Rep B)");
            slots[i] = trits[i];
            i += 1;
        }
        let (s, c) = normalize_inline(slots, count as u8);
        TritInt { storage: TritIntStorage::Inline { trits: s, trit_count: c } }
    }

    /// Construct from a trit slice (RUNTIME PATH). Rep B {0, 1, 2}, LSB first.
    /// Unbounded — auto-promotes to heap for values exceeding 40 trits.
    /// Panics if any trit value ≥ 3.
    pub fn from_trit_slice(trits: &[u8]) -> Self {
        for &t in trits {
            assert!(t < 3, "from_trit_slice: trit value must be 0, 1, or 2 (Rep B)");
        }
        make_from_trits_lsb(trits)
    }

    // ── Internal extractors ─────────────────────────────────

    /// Extract slot array and count (const fn, inline-only, takes self by value).
    /// Panics on Heap — const fn cannot handle heap allocation.
    const fn into_parts(self) -> ([u8; 40], u8) {
        match self.storage {
            TritIntStorage::Inline { trits, trit_count } => (trits, trit_count),
            TritIntStorage::Heap { .. } => panic!("into_parts() called on heap TritInt"),
        }
    }

    /// Slot slice view. Works for both inline and heap.
    fn trits_view(&self) -> &[u8] {
        match &self.storage {
            TritIntStorage::Inline { trits, .. } => &trits[..],
            TritIntStorage::Heap { trits, .. } => &trits[..],
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
    /// Const equality comparison (ternary-native).
    pub const fn const_eq(self, other: TritInt) -> bool {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        eq_inline(a, ac, b, bc)
    }

    /// Const less-than comparison. Compares trit-by-trit from MSB.
    pub const fn const_lt(self, other: TritInt) -> bool {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        lt_inline(a, ac, b, bc)
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
        let (trits, count) = add_inline(a, ac, b, bc);
        make_inline(trits, count)
    }

    /// Const subtraction. Panics on underflow (other > self).
    pub const fn const_sub(self, other: TritInt) -> TritInt {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        let (trits, count) = sub_inline(a, ac, b, bc);
        make_inline(trits, count)
    }

    /// Const multiplication. Ternary long multiplication. Panics if result > R₄ trits.
    /// All framework constant derivations produce results under 25 trits.
    pub const fn const_mul(self, other: TritInt) -> TritInt {
        let (a, ac) = self.into_parts();
        let (b, bc) = other.into_parts();
        let (trits, count) = mul_inline(a, ac, b, bc);
        make_inline(trits, count)
    }
}

// ══════════════════════════════════════════════════════════════
// RUNTIME ARITHMETIC
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Add two TritInts. Auto-promotes to heap if result > R₄ = 40 trits.
    pub fn add(&self, other: &TritInt) -> TritInt {
        add_gen(self.trits_view(), self.count(), other.trits_view(), other.count())
    }

    /// Subtract other from self. Panics if other > self (unsigned underflow).
    pub fn sub(&self, other: &TritInt) -> TritInt {
        sub_gen(self.trits_view(), self.count(), other.trits_view(), other.count())
    }

    /// Multiply two TritInts. Auto-promotes to heap if result > R₄ = 40 trits.
    pub fn mul(&self, other: &TritInt) -> TritInt {
        mul_gen(self.trits_view(), self.count(), other.trits_view(), other.count())
    }

    /// Division with remainder: returns (quotient, remainder).
    /// Panics if divisor is zero.
    pub fn div_mod(&self, divisor: &TritInt) -> (TritInt, TritInt) {
        assert!(!divisor.is_zero(), "div_mod: division by zero");

        if self.is_zero() {
            return (TritInt::zero(), TritInt::zero());
        }

        let ac = self.count();
        let bc = divisor.count();

        if lt_gen(self.trits_view(), ac, divisor.trits_view(), bc) {
            return (TritInt::zero(), self.clone());
        }

        let shift = (ac as i64) - (bc as i64);
        let mut remainder = self.clone();
        let mut quotient_trits = vec![0u8; (shift + 1) as usize];
        let mut q_count: u32 = 0;

        let mut i = shift;
        while i >= 0 {
            let pos = i as u32;
            let (sp, sc) = trit_shift_left_gen(divisor.trits_view(), bc, pos);
            let doubled = add_gen(&sp, sc, &sp, sc);

            let rp = remainder.trits_view();
            let rc = remainder.count();

            if !lt_gen(rp, rc, doubled.trits_view(), doubled.count()) {
                quotient_trits[pos as usize] = 2;
                remainder = sub_gen(rp, rc, doubled.trits_view(), doubled.count());
            } else if !lt_gen(rp, rc, &sp, sc) {
                quotient_trits[pos as usize] = 1;
                remainder = sub_gen(rp, rc, &sp, sc);
            }

            if quotient_trits[pos as usize] != 0 && pos >= q_count {
                q_count = pos + 1;
            }

            i -= 1;
        }

        let q = make_from_trits_lsb(&quotient_trits[..q_count as usize]);
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

            let qs = TritInt::mul(&quotient, &s);
            let (new_s, new_s_neg) = signed_sub(old_s, old_s_neg, qs, s_neg);
            old_s = s;
            old_s_neg = s_neg;
            s = new_s;
            s_neg = new_s_neg;
        }

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
            let at = trit_at_slice(self.trits_view(), i);
            let bt = trit_at_slice(other.trits_view(), i);
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
            value: make_from_trits_lsb(&result_trits),
            carry_count,
            max_carry_chain,
        }
    }
}

// ── Signed subtraction helper for extended_gcd ──────────────

/// Compute (a_val, a_neg) - (b_val, b_neg) with sign tracking.
fn signed_sub(a: TritInt, a_neg: bool, b: TritInt, b_neg: bool) -> (TritInt, bool) {
    let b_neg_flipped = !b_neg;
    signed_add(a, a_neg, b, b_neg_flipped)
}

/// Compute (a_val, a_neg) + (b_val, b_neg) with sign tracking.
fn signed_add(a: TritInt, a_neg: bool, b: TritInt, b_neg: bool) -> (TritInt, bool) {
    if a_neg == b_neg {
        (TritInt::add(&a, &b), a_neg)
    } else {
        if lt_gen(a.trits_view(), a.count(), b.trits_view(), b.count()) {
            (TritInt::sub(&b, &a), b_neg)
        } else if lt_gen(b.trits_view(), b.count(), a.trits_view(), a.count()) {
            (TritInt::sub(&a, &b), a_neg)
        } else {
            (TritInt::zero(), false)
        }
    }
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
        trit_at_slice(self.trits_view(), i as u32)
    }

    /// How many distinct trit values {0, 1, 2} appear in this number.
    pub fn trit_diversity(&self) -> u8 {
        let mut seen = [false; 3];
        for i in 0..self.count() {
            seen[trit_at_slice(self.trits_view(), i) as usize] = true;
        }
        seen[0] as u8 + seen[1] as u8 + seen[2] as u8
    }

    /// True if all trits are 1 (repunit). Zero is not a repunit.
    pub fn is_repunit(&self) -> bool {
        if self.count() == 0 { return false; }
        for i in 0..self.count() {
            if trit_at_slice(self.trits_view(), i) != 1 { return false; }
        }
        true
    }

    /// True if the value is a power of 3 (exactly one non-zero trit, which is 1).
    pub fn is_power_of_3(&self) -> bool {
        let c = self.count();
        if c == 0 { return false; }
        if trit_at_slice(self.trits_view(), c - 1) != 1 { return false; }
        for i in 0..c - 1 {
            if trit_at_slice(self.trits_view(), i) != 0 { return false; }
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
            result.push(trit_at_slice(self.trits_view(), i));
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
// HOST BOUNDARY CROSSINGS
//
// Every method here is an explicit binary-host crossing. The
// `host_` prefix marks every call site as a boundary point —
// ternary leaves the framework, binary enters the host.
//
// Use only at FFI / JSON / log / array-sizing edges. New
// arithmetic should stay ternary-native via the methods above.
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Cross into host u32 at compile time. Inline storage only.
    /// Panics on heap storage or on overflow past u32::MAX.
    pub const fn host_u32(&self) -> u32 {
        match &self.storage {
            TritIntStorage::Inline { trits, trit_count } => {
                let mut acc: u64 = 0;
                let mut i = *trit_count as usize;
                while i > 0 {
                    i -= 1;
                    acc = acc * 3 + trits[i] as u64;
                }
                assert!(acc <= u32::MAX as u64, "host_u32: value exceeds u32::MAX");
                acc as u32
            }
            TritIntStorage::Heap { .. } => {
                panic!("host_u32: heap storage not supported in const path");
            }
        }
    }

    /// Cross into host u64 at runtime. Panics on overflow past u64::MAX.
    pub fn host_u64(&self) -> u64 {
        let mut acc: u128 = 0;
        let mut i = self.count() as usize;
        let view = self.trits_view();
        while i > 0 {
            i -= 1;
            acc = acc * 3 + view[i] as u128;
            assert!(acc <= u64::MAX as u128, "host_u64: value exceeds u64::MAX");
        }
        acc as u64
    }

    /// Build a TritInt from a host u64 at runtime.
    pub fn from_host_u64(mut v: u64) -> TritInt {
        if v == 0 { return TritInt::zero(); }
        let mut trits = Vec::with_capacity(41);
        while v > 0 {
            trits.push((v % 3) as u8);
            v /= 3;
        }
        make_from_trits_lsb(&trits)
    }

    /// Cross into host u128 at runtime. Panics on overflow past u128::MAX.
    /// Used at femtosecond-timestamp / wide-counter boundary points.
    pub fn host_u128(&self) -> u128 {
        let mut acc: u128 = 0;
        let mut i = self.count() as usize;
        let view = self.trits_view();
        while i > 0 {
            i -= 1;
            // u128::MAX / 3 ≈ 1.13 × 10³⁸ — overflow guard.
            assert!(acc <= u128::MAX / 3, "host_u128: value exceeds u128::MAX");
            acc = acc * 3 + view[i] as u128;
        }
        acc
    }

    /// Build a TritInt from a host u128 at runtime.
    pub fn from_host_u128(mut v: u128) -> TritInt {
        if v == 0 { return TritInt::zero(); }
        let mut trits = Vec::with_capacity(82);
        while v > 0 {
            trits.push((v % 3) as u8);
            v /= 3;
        }
        make_from_trits_lsb(&trits)
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
            write!(f, "{}", trit_at_slice(self.trits_view(), i))?;
        }
        write!(f, "₃")
    }
}

impl fmt::Debug for TritInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let storage = if matches!(self.storage, TritIntStorage::Heap { .. }) { "heap" } else { "inline" };
        write!(f, "TritInt({} [{} trits, {}])", self, self.count(), storage)
    }
}

// ── PartialEq, Eq ───────────────────────────────────────────

impl PartialEq for TritInt {
    fn eq(&self, other: &Self) -> bool {
        eq_gen(self.trits_view(), self.count(), other.trits_view(), other.count())
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
        if eq_gen(self.trits_view(), self.count(), other.trits_view(), other.count()) {
            std::cmp::Ordering::Equal
        } else if lt_gen(self.trits_view(), self.count(), other.trits_view(), other.count()) {
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
        let view = self.trits_view();
        view[..c as usize].hash(state);
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
// Pure ternary identities. No host-binary boundary, no `to_u32_const`.
// Each assertion compares two `TritInt` expressions via `const_eq`,
// where the right-hand side is the explicit Rep B trit pattern (LSB first).
// ══════════════════════════════════════════════════════════════

const _: () = {
    // Repunit identity: R_n is exactly n consecutive ones (LSB-first).
    assert!(TritInt::repunit(1).const_eq(TritInt::from_trits(&[1])));
    assert!(TritInt::repunit(2).const_eq(TritInt::from_trits(&[1, 1])));
    assert!(TritInt::repunit(3).const_eq(TritInt::from_trits(&[1, 1, 1])));
    assert!(TritInt::repunit(4).const_eq(TritInt::from_trits(&[1, 1, 1, 1])));
    assert!(TritInt::repunit(5).const_eq(TritInt::from_trits(&[1, 1, 1, 1, 1])));
    assert!(TritInt::repunit(6).const_eq(TritInt::from_trits(&[1, 1, 1, 1, 1, 1])));

    // Repunit recurrence R(n) = three · R(n−1) + 1, where three = [0,1] LSB-first.
    assert!(TritInt::from_trits(&[0, 1]).const_mul(TritInt::repunit(1)).const_add(TritInt::one())
        .const_eq(TritInt::repunit(2)));
    assert!(TritInt::from_trits(&[0, 1]).const_mul(TritInt::repunit(2)).const_add(TritInt::one())
        .const_eq(TritInt::repunit(3)));
    assert!(TritInt::from_trits(&[0, 1]).const_mul(TritInt::repunit(3)).const_add(TritInt::one())
        .const_eq(TritInt::repunit(4)));
    assert!(TritInt::from_trits(&[0, 1]).const_mul(TritInt::repunit(4)).const_add(TritInt::one())
        .const_eq(TritInt::repunit(5)));
    assert!(TritInt::from_trits(&[0, 1]).const_mul(TritInt::repunit(5)).const_add(TritInt::one())
        .const_eq(TritInt::repunit(6)));

    // π = R₃ + 1, Rep B LSB-first pattern [2, 1, 1].
    assert!(TritInt::repunit(3).const_add(TritInt::one())
        .const_eq(TritInt::from_trits(&[2, 1, 1])));

    // ARC_ROOT_SEMI = π × R₃, Rep B LSB-first pattern [2, 0, 2, 0, 2].
    assert!(TritInt::from_trits(&[2, 1, 1]).const_mul(TritInt::repunit(3))
        .const_eq(TritInt::from_trits(&[2, 0, 2, 0, 2])));

    // Discriminant Δ₂ = 1 + R₂ × ARC_ROOT_SEMI = 3⁶, Rep B LSB-first six zeros then a one.
    assert!(
        TritInt::one()
            .const_add(
                TritInt::repunit(2)
                    .const_mul(TritInt::from_trits(&[2, 0, 2, 0, 2]))
            )
            .const_eq(TritInt::from_trits(&[0, 0, 0, 0, 0, 0, 1]))
    );

    // ARC_ROOT_COMP = ARC_ROOT_DOUBLE − ARC_ROOT_SEMI (Vieta); Rep B LSB-first patterns.
    // ARC_ROOT_DOUBLE LSB pattern [1, 1, 2, 0, 1, 0, 1]; ARC_ROOT_COMP LSB pattern [2, 0, 0, 0, 2, 2].
    assert!(
        TritInt::from_trits(&[1, 1, 2, 0, 1, 0, 1])
            .const_sub(TritInt::from_trits(&[2, 0, 2, 0, 2]))
            .const_eq(TritInt::from_trits(&[2, 0, 0, 0, 2, 2]))
    );

    // Vieta sum: ARC_ROOT_SEMI + ARC_ROOT_COMP = ARC_ROOT_DOUBLE.
    assert!(
        TritInt::from_trits(&[2, 0, 2, 0, 2])
            .const_add(TritInt::from_trits(&[2, 0, 0, 0, 2, 2]))
            .const_eq(TritInt::from_trits(&[1, 1, 2, 0, 1, 0, 1]))
    );

    // LCM_PRIMARY = seven · eleven · thirteen, with thirteen = R₃.
    // seven LSB pattern [1, 2]; eleven LSB pattern [2, 0, 1]; product LSB pattern [2, 0, 0, 1, 0, 1, 1].
    assert!(
        TritInt::from_trits(&[1, 2])
            .const_mul(TritInt::from_trits(&[2, 0, 1]))
            .const_mul(TritInt::repunit(3))
            .const_eq(TritInt::from_trits(&[2, 0, 0, 1, 0, 1, 1]))
    );

    // CENTER × 2 = ARC_ROOT_SEMI + R₄. CENTER × 2 LSB pattern [0, 2, 0, 2, 2].
    assert!(
        TritInt::from_trits(&[2, 0, 2, 0, 2])
            .const_add(TritInt::repunit(4))
            .const_eq(TritInt::from_trits(&[0, 2, 0, 2, 2]))
    );

    // MAGIC_CONSTANT = three × CENTER. CENTER LSB pattern [0, 1, 0, 1, 1]; product LSB pattern [0, 0, 1, 0, 1, 1].
    assert!(
        TritInt::from_trits(&[0, 1])
            .const_mul(TritInt::from_trits(&[0, 1, 0, 1, 1]))
            .const_eq(TritInt::from_trits(&[0, 0, 1, 0, 1, 1]))
    );

    // Coprime-pair product = seven × R₃; LSB pattern [1, 0, 1, 0, 1].
    assert!(
        TritInt::from_trits(&[1, 2])
            .const_mul(TritInt::repunit(3))
            .const_eq(TritInt::from_trits(&[1, 0, 1, 0, 1]))
    );

    // Negative path: R₅ ≠ R₆.
    assert!(!TritInt::repunit(5).const_eq(TritInt::repunit(6)));

    // Const comparison
    assert!(TritInt::repunit(3).const_lt(TritInt::repunit(4)));
    assert!(!TritInt::repunit(4).const_lt(TritInt::repunit(3)));
    assert!(TritInt::zero().const_eq(TritInt::zero()));
    assert!(!TritInt::zero().const_eq(TritInt::one()));
};

// ══════════════════════════════════════════════════════════════
// REPRESENTATION CONVERSIONS
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
            let digit = trit_at_slice(self.trits_view(), i) + carry;
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

        while balanced.last() == Some(&0) {
            balanced.pop();
        }
        balanced.reverse();
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
            result.push(trit_at_slice(self.trits_view(), i) + 1);
        }
        result
    }

    /// Convert to Rep D (algebraic, {Zero, One, Omega}), MSB-first.
    pub fn to_repr_d(&self) -> Vec<AlgebraicTrit> {
        let c = self.count();
        if c == 0 { return Vec::new(); }

        let mut result = Vec::with_capacity(c as usize);
        let mut i = c;
        while i > 0 {
            i -= 1;
            result.push(match trit_at_slice(self.trits_view(), i) {
                0 => AlgebraicTrit::Zero,
                1 => AlgebraicTrit::One,
                2 => AlgebraicTrit::Omega,
                _ => unreachable!(),
            });
        }
        result
    }

    /// Construct from Rep A (balanced, {−1, 0, +1}), MSB-first input.
    pub fn from_repr_a(balanced: &[i8]) -> Self {
        if balanced.is_empty() { return TritInt::zero(); }

        for &d in balanced {
            assert!(d >= -1 && d <= 1, "from_repr_a: digit must be -1, 0, or +1, got {}", d);
        }

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
            panic!("from_repr_a: conversion produced negative carry — invalid input");
        }

        TritInt::from_trit_slice(&rep_b)
    }

    /// Construct from Rep B (standard, {0, 1, 2}), MSB-first input.
    pub fn from_repr_b(standard: &[u8]) -> Self {
        if standard.is_empty() { return TritInt::zero(); }

        for &d in standard {
            assert!(d < 3, "from_repr_b: digit must be 0, 1, or 2, got {}", d);
        }

        let mut lsb_first: Vec<u8> = standard.to_vec();
        lsb_first.reverse();
        TritInt::from_trit_slice(&lsb_first)
    }

    /// Construct from Rep C (bijective, {1, 2, 3}), MSB-first input.
    ///
    /// Panics on any digit = 0 (forgery detection) or digit > 3.
    pub fn from_repr_c(bijective: &[u8]) -> Self {
        if bijective.is_empty() { return TritInt::zero(); }

        for &d in bijective {
            assert!(d >= 1 && d <= 3, "from_repr_c: digit must be 1, 2, or 3, got {} — zero = forgery", d);
        }

        let mut lsb_first: Vec<u8> = bijective.iter().map(|&c| c - 1).collect();
        lsb_first.reverse();
        TritInt::from_trit_slice(&lsb_first)
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
        TritInt::from_trit_slice(&lsb_first)
    }
}

// ══════════════════════════════════════════════════════════════
// DIV_REPUNIT AND MOD_POW
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

        let divisor = TritInt::repunit_rt(n);

        if *self < divisor {
            return (TritInt::zero(), self.clone());
        }

        let trits = self.to_trits();
        let num_chunks = (trits.len() + n - 1) / n;

        let mut chunk_sum = TritInt::zero();
        for c in 0..num_chunks {
            let start = c * n;
            let end = std::cmp::min(start + n, trits.len());
            let chunk = TritInt::from_trit_slice(&trits[start..end]);
            chunk_sum = TritInt::add(&chunk_sum, &chunk);
        }

        let (_, remainder) = if chunk_sum >= divisor {
            chunk_sum.div_mod(&divisor)
        } else {
            (TritInt::zero(), chunk_sum)
        };

        let numerator = TritInt::sub(self, &remainder);
        let (quotient, check_rem) = numerator.div_mod(&divisor);
        assert!(check_rem.is_zero(), "div_repunit: internal error — inexact quotient");

        (quotient, remainder)
    }

    /// Modular exponentiation: self^exp mod modulus.
    ///
    /// Ternary-native left-to-right square-and-multiply: walk the exponent
    /// trit-by-trit from MSB; for each step compute result³ · base^trit.
    /// NOT constant-time — must NOT be used for cryptographic operations
    /// (all crypto uses TLSponge-385 or TL-DSA).
    pub fn mod_pow(&self, exp: &TritInt, modulus: &TritInt) -> TritInt {
        assert!(!modulus.is_zero(), "mod_pow: modulus must be non-zero");

        if exp.is_zero() {
            return if *modulus > TritInt::one() {
                TritInt::one()
            } else {
                TritInt::zero()
            };
        }

        let exp_trits = exp.trits_msb_first();
        let base = self.div_mod(modulus).1;
        let base_squared = TritInt::mul(&base, &base).div_mod(modulus).1;

        let mut result = TritInt::one();
        for &t in &exp_trits {
            // result = result³ mod modulus
            let r2 = TritInt::mul(&result, &result).div_mod(modulus).1;
            result = TritInt::mul(&r2, &result).div_mod(modulus).1;

            // multiply by base^t (t ∈ {0,1,2})
            if t == 1 {
                result = TritInt::mul(&result, &base).div_mod(modulus).1;
            } else if t == 2 {
                result = TritInt::mul(&result, &base_squared).div_mod(modulus).1;
            }
        }

        result
    }
}

// ══════════════════════════════════════════════════════════════
// TritIntError + try_from_repr_c
//
// Result-based Rep C parsing for untrusted wire input. The single
// enforcement point for input validation — both Serde and FFI callers
// route through here.
// ══════════════════════════════════════════════════════════════

/// Error type for TritInt parsing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TritIntError {
    /// A digit in the Rep C input is invalid (0 = forgery, or > 3).
    InvalidDigit(u8),
}

impl fmt::Display for TritIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TritIntError::InvalidDigit(d) => write!(f, "invalid Rep C digit: {} (must be 1, 2, or 3; zero = forgery)", d),
        }
    }
}

impl std::error::Error for TritIntError {}

impl TritInt {
    /// Parse Rep C input (MSB-first) into a TritInt, returning Result.
    ///
    /// This is the single enforcement point for untrusted input validation.
    /// Unbounded — auto-promotes to heap for values exceeding 40 trits.
    ///
    /// - Empty input → Ok(TritInt::zero())
    /// - Zero digit → Err(InvalidDigit(0)) — forgery detection
    /// - Digit > 3 → Err(InvalidDigit(d))
    pub fn try_from_repr_c(bijective: &[u8]) -> Result<Self, TritIntError> {
        if bijective.is_empty() {
            return Ok(TritInt::zero());
        }

        for &d in bijective {
            if d == 0 || d > 3 {
                return Err(TritIntError::InvalidDigit(d));
            }
        }

        let mut lsb_first: Vec<u8> = bijective.iter().map(|&c| c - 1).collect();
        lsb_first.reverse();

        Ok(make_from_trits_lsb(&lsb_first))
    }
}

// ══════════════════════════════════════════════════════════════
// ZEROIZE
//
// Cryptographic erasure for TritInt values that may contain key material.
// Heap allocations leak by default (ManuallyDrop preserves const-compatibility);
// call .zeroize() or .drop_heap() to release them.
// ══════════════════════════════════════════════════════════════

impl TritInt {
    /// Explicitly drop a heap-allocated TritInt, zeroing and deallocating the buffer.
    /// No-op for inline values. Must be called before heap TritInts go out of scope
    /// to prevent memory leaks (ManuallyDrop suppresses automatic Drop for const compatibility).
    pub fn drop_heap(&mut self) {
        if matches!(self.storage, TritIntStorage::Heap { .. }) {
            if let TritIntStorage::Heap { trits, trit_count } = &mut self.storage {
                trits.zeroize();
                *trit_count = 0;
                unsafe { ManuallyDrop::drop(trits); }
            }
            self.storage = TritIntStorage::Inline { trits: [0u8; 40], trit_count: 0 };
        }
    }
}

impl Zeroize for TritInt {
    fn zeroize(&mut self) {
        let was_heap = matches!(self.storage, TritIntStorage::Heap { .. });
        match &mut self.storage {
            TritIntStorage::Inline { trits, trit_count } => {
                trits.zeroize();
                *trit_count = 0;
            }
            TritIntStorage::Heap { trits, trit_count } => {
                trits.zeroize();
                *trit_count = 0;
                unsafe { ManuallyDrop::drop(trits); }
            }
        }
        if was_heap {
            self.storage = TritIntStorage::Inline { trits: [0u8; 40], trit_count: 0 };
        }
    }
}

// ══════════════════════════════════════════════════════════════
// SERDE (behind #[cfg(feature = "serde")])
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

    // ── Round-trips ─────────────────────────────────────────

    #[test]
    fn round_trip_single_trit() {
        for trit in 0..3u8 {
            let t = TritInt::from_trits(&[trit]);
            assert_eq!(t.trit_at(0), trit);
            assert_eq!(t.trit_length(), if trit == 0 { 0 } else { 1 });
        }
    }

    #[test]
    fn round_trip_inline_boundary() {
        // R₄ = 40 trits: the inline boundary
        let trits = vec![1u8; 40];
        let t = TritInt::from_trit_slice(&trits);
        assert_eq!(t.trit_length(), 40);
        for i in 0..40 {
            assert_eq!(t.trit_at(i), 1);
        }
        assert!(t.is_repunit());
    }

    #[test]
    fn round_trip_heap() {
        // R₅ = 121 trits: forces heap
        let trits = vec![1u8; 121];
        let t = TritInt::from_trit_slice(&trits);
        assert_eq!(t.trit_length(), 121);
        for i in 0..121 {
            assert_eq!(t.trit_at(i), 1);
        }
        assert!(t.is_repunit());
    }

    // ── Repunits ────────────────────────────────────────────

    #[test]
    fn repunit_const_path() {
        assert_eq!(TritInt::repunit(1).to_trits(), vec![1]);
        assert_eq!(TritInt::repunit(2).to_trits(), vec![1, 1]);
        assert_eq!(TritInt::repunit(6).to_trits(), vec![1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn repunit_rt_promotes_to_heap() {
        let r60 = TritInt::repunit_rt(60);
        assert_eq!(r60.trit_length(), 60);
        for i in 0..60 {
            assert_eq!(r60.trit_at(i), 1);
        }
    }

    // ── Arithmetic ──────────────────────────────────────────

    #[test]
    fn add_repunit_chain() {
        // R₃ + 1 = π = [2,1,1] (LSB)
        let pi = TritInt::add(&TritInt::repunit(3), &TritInt::one());
        assert_eq!(pi.to_trits(), vec![2, 1, 1]);
    }

    #[test]
    fn mul_pi_r3_yields_arc_root_semi() {
        // π × R₃ = ARC_ROOT_SEMI = [2,0,2,0,2] (LSB)
        let pi = TritInt::from_trits(&[2, 1, 1]);
        let semi = TritInt::mul(&pi, &TritInt::repunit(3));
        assert_eq!(semi.to_trits(), vec![2, 0, 2, 0, 2]);
    }

    #[test]
    fn delta2_is_3_to_the_6() {
        // Δ₂ = 1 + R₂ × ARC_ROOT_SEMI = 3⁶ = [0,0,0,0,0,0,1] (LSB)
        let semi = TritInt::from_trits(&[2, 0, 2, 0, 2]);
        let prod = TritInt::mul(&TritInt::repunit(2), &semi);
        let delta2 = TritInt::add(&TritInt::one(), &prod);
        assert_eq!(delta2.to_trits(), vec![0, 0, 0, 0, 0, 0, 1]);
        assert!(delta2.is_power_of_3());
        assert_eq!(delta2.ternary_exponent(), Some(6));
    }

    #[test]
    fn div_mod_repunit() {
        // R₆ ÷ R₃ = 3⁴ = [0,0,0,0,1] (LSB) = 81... wait: R₆/R₃ = (3⁶−1)/(3³−1) = 728/26 = 28.
        // 28 LSB: 28/3=9 r1, 9/3=3 r0, 3/3=1 r0, 1/3=0 r1 → [1,0,0,1]
        let (q, r) = TritInt::repunit(6).div_mod(&TritInt::repunit(3));
        assert_eq!(r.to_trits(), Vec::<u8>::new());
        assert_eq!(q.to_trits(), vec![1, 0, 0, 1]);
    }

    // ── Representations ─────────────────────────────────────

    #[test]
    fn rep_c_round_trip() {
        // Rep B [2,1,1] LSB → Rep C MSB [2,2,3]
        let pi = TritInt::from_trits(&[2, 1, 1]);
        let rep_c = pi.to_repr_c();
        assert_eq!(rep_c, vec![2, 2, 3]);
        let back = TritInt::from_repr_c(&rep_c);
        assert_eq!(back.to_trits(), vec![2, 1, 1]);
    }

    #[test]
    fn try_from_repr_c_rejects_zero_digit() {
        assert_eq!(
            TritInt::try_from_repr_c(&[1, 0, 2]),
            Err(TritIntError::InvalidDigit(0))
        );
    }

    #[test]
    fn try_from_repr_c_rejects_digit_above_three() {
        assert_eq!(
            TritInt::try_from_repr_c(&[1, 4, 2]),
            Err(TritIntError::InvalidDigit(4))
        );
    }

    // ── Const evaluation ────────────────────────────────────

    #[test]
    fn const_eq_repunit_chain() {
        const _: () = {
            assert!(TritInt::repunit(6).const_eq(TritInt::from_trits(&[1, 1, 1, 1, 1, 1])));
        };
    }

    #[test]
    fn const_arithmetic_matches_runtime() {
        const SUM: TritInt = TritInt::repunit(3).const_add(TritInt::one());
        let runtime = TritInt::add(&TritInt::repunit(3), &TritInt::one());
        assert_eq!(SUM, runtime);
    }
}
