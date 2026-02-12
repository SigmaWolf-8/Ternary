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

use alloc::vec::Vec;
use crate::ternary::Trit;

/// Constant-time ternary operations (T-38)
/// These operations take the same number of CPU cycles regardless of input values,
/// preventing timing side-channel attacks in cryptographic contexts.
pub struct ConstantTimeTernary;

impl ConstantTimeTernary {
    /// Constant-time GF(3) addition: avoids branches by using arithmetic
    pub fn ct_add(a: i8, b: i8) -> i8 {
        let sum = a as i16 + b as i16;
        let rem = ((sum % 3) + 3) % 3;
        let r = rem as i8;
        r - 3 * (r >> 1)
    }

    /// Constant-time GF(3) multiplication
    pub fn ct_mul(a: i8, b: i8) -> i8 {
        let prod = a as i16 * b as i16;
        let rem = ((prod % 3) + 3) % 3;
        let r = rem as i8;
        r - 3 * (r >> 1)
    }

    /// Constant-time GF(3) negation
    pub fn ct_neg(a: i8) -> i8 {
        -a
    }

    /// Constant-time ternary min (Kleene XOR)
    pub fn ct_min(a: i8, b: i8) -> i8 {
        let diff = a as i16 - b as i16;
        let mask = (diff >> 15) as i8;
        b + (diff as i8 & mask)
    }

    /// Constant-time ternary max (Kleene OR)
    pub fn ct_max(a: i8, b: i8) -> i8 {
        let diff = a as i16 - b as i16;
        let mask = (diff >> 15) as i8;
        a - (diff as i8 & mask)
    }

    /// Constant-time conditional select: if sel == 0, return a; else return b
    pub fn ct_select(sel: i8, a: i8, b: i8) -> i8 {
        let mask = (sel != 0) as i8;
        let neg_mask = 1 - mask;
        a * neg_mask + b * mask
    }

    /// Constant-time trit equality check (returns 1 if equal, 0 otherwise)
    pub fn ct_eq(a: i8, b: i8) -> i8 {
        let diff = a ^ b;
        let is_zero = ((!diff as u8 & 0x01) & (!((diff as u8) >> 1) & 0x01)) as i8;
        is_zero
    }

    /// Constant-time packed word add (operates on all 27 trits)
    pub fn ct_packed_add(a: i64, b: i64) -> i64 {
        let ta = crate::ternary::unpack_trits(a);
        let tb = crate::ternary::unpack_trits(b);
        let mut result = [Trit::from_a(0).unwrap(); 27];
        for i in 0..27 {
            let r = Self::ct_add(ta[i].to_a(), tb[i].to_a());
            result[i] = Trit::from_a(r).unwrap_or(Trit::from_a(0).unwrap());
        }
        crate::ternary::pack_trits(&result)
    }

    /// Constant-time packed word multiply
    pub fn ct_packed_mul(a: i64, b: i64) -> i64 {
        let ta = crate::ternary::unpack_trits(a);
        let tb = crate::ternary::unpack_trits(b);
        let mut result = [Trit::from_a(0).unwrap(); 27];
        for i in 0..27 {
            let r = Self::ct_mul(ta[i].to_a(), tb[i].to_a());
            result[i] = Trit::from_a(r).unwrap_or(Trit::from_a(0).unwrap());
        }
        crate::ternary::pack_trits(&result)
    }
}

/// Instruction cache for decoded instructions (T-39)
/// Caches decoded instructions to avoid repeated decode overhead.
pub struct InstructionCache {
    entries: Vec<Option<CacheEntry>>,
    capacity: usize,
    hits: u64,
    misses: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    address: u64,
    opcode_byte: u8,
    dst: u8,
    src1: u8,
    src2: u8,
    immediate: i64,
}

impl InstructionCache {
    pub fn new(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(None);
        }
        Self {
            entries,
            capacity,
            hits: 0,
            misses: 0,
        }
    }

    fn index(&self, address: u64) -> usize {
        (address as usize) % self.capacity
    }

    pub fn lookup(&mut self, address: u64) -> Option<(u8, u8, u8, u8, i64)> {
        let idx = self.index(address);
        if let Some(ref entry) = self.entries[idx] {
            if entry.address == address {
                self.hits += 1;
                return Some((entry.opcode_byte, entry.dst, entry.src1, entry.src2, entry.immediate));
            }
        }
        self.misses += 1;
        None
    }

    pub fn insert(&mut self, address: u64, opcode_byte: u8, dst: u8, src1: u8, src2: u8, immediate: i64) {
        let idx = self.index(address);
        self.entries[idx] = Some(CacheEntry {
            address,
            opcode_byte,
            dst,
            src1,
            src2,
            immediate,
        });
    }

    pub fn invalidate(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
        self.hits = 0;
        self.misses = 0;
    }

    pub fn invalidate_address(&mut self, address: u64) {
        let idx = self.index(address);
        self.entries[idx] = None;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    pub fn hits(&self) -> u64 { self.hits }
    pub fn misses(&self) -> u64 { self.misses }
    pub fn capacity(&self) -> usize { self.capacity }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_add() {
        assert_eq!(ConstantTimeTernary::ct_add(1, 1), -1);
        assert_eq!(ConstantTimeTernary::ct_add(1, 0), 1);
        assert_eq!(ConstantTimeTernary::ct_add(0, 0), 0);
        assert_eq!(ConstantTimeTernary::ct_add(-1, 1), 0);
        assert_eq!(ConstantTimeTernary::ct_add(-1, -1), 1);
    }

    #[test]
    fn test_ct_mul() {
        assert_eq!(ConstantTimeTernary::ct_mul(1, 1), 1);
        assert_eq!(ConstantTimeTernary::ct_mul(1, -1), -1);
        assert_eq!(ConstantTimeTernary::ct_mul(-1, -1), 1);
        assert_eq!(ConstantTimeTernary::ct_mul(0, 1), 0);
        assert_eq!(ConstantTimeTernary::ct_mul(0, -1), 0);
    }

    #[test]
    fn test_ct_min_max() {
        assert_eq!(ConstantTimeTernary::ct_min(1, -1), -1);
        assert_eq!(ConstantTimeTernary::ct_min(-1, 1), -1);
        assert_eq!(ConstantTimeTernary::ct_min(0, 1), 0);
        assert_eq!(ConstantTimeTernary::ct_max(1, -1), 1);
        assert_eq!(ConstantTimeTernary::ct_max(-1, 0), 0);
    }

    #[test]
    fn test_ct_select() {
        assert_eq!(ConstantTimeTernary::ct_select(0, 1, -1), 1);
        assert_eq!(ConstantTimeTernary::ct_select(1, 1, -1), -1);
        assert_eq!(ConstantTimeTernary::ct_select(-1, 1, -1), -1);
    }

    #[test]
    fn test_ct_packed_add() {
        let a = crate::ternary::pack_trits(&[Trit::from_a(1).unwrap(); 27]);
        let b = crate::ternary::pack_trits(&[Trit::from_a(1).unwrap(); 27]);
        let result = ConstantTimeTernary::ct_packed_add(a, b);
        let trits = crate::ternary::unpack_trits(result);
        assert_eq!(trits[0].to_a(), -1);
    }

    #[test]
    fn test_cache_insert_lookup() {
        let mut cache = InstructionCache::new(64);
        cache.insert(0, 0x10, 1, 2, 3, 42);
        let result = cache.lookup(0);
        assert!(result.is_some());
        let (op, dst, _s1, _s2, imm) = result.unwrap();
        assert_eq!(op, 0x10);
        assert_eq!(dst, 1);
        assert_eq!(imm, 42);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = InstructionCache::new(64);
        assert!(cache.lookup(99).is_none());
        assert_eq!(cache.misses(), 1);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = InstructionCache::new(64);
        cache.insert(0, 0x10, 0, 0, 0, 0);
        cache.invalidate();
        assert!(cache.lookup(0).is_none());
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut cache = InstructionCache::new(64);
        cache.insert(0, 0x10, 0, 0, 0, 0);
        cache.lookup(0);
        cache.lookup(1);
        assert!(cache.hit_rate() > 0.49);
    }
}
