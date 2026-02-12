// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Constant-Time Utility Functions
//!
//! Provides primitive constant-time operations for cryptographic code.
//! All functions execute in time independent of input values, preventing
//! timing side-channel attacks.
//!
//! # Operations
//!
//! - **ct_eq_u8**: Constant-time byte equality
//! - **ct_neq_u8**: Constant-time byte inequality
//! - **ct_select_u8**: Constant-time conditional select (byte)
//! - **ct_select_i8**: Constant-time conditional select (trit)
//! - **ct_select_slice**: Constant-time conditional select (slice)
//! - **ct_cmov_slice**: Constant-time conditional move (in-place)
//! - **ct_eq_slices**: Constant-time slice equality
//! - **ct_is_zero**: Constant-time zero check
//!
//! # FIPS 140-3 Requirement
//!
//! Constant-time operations are required for FIPS 140-3 Level 3+
//! to resist non-invasive side-channel attacks (ISO 19790 §7.8).
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::vec::Vec;

#[inline(always)]
pub fn ct_eq_u8(a: u8, b: u8) -> u8 {
    let x = a ^ b;
    let x16 = x as u16;
    let neg = x16.wrapping_sub(1);
    (neg >> 8) as u8
}

#[inline(always)]
pub fn ct_neq_u8(a: u8, b: u8) -> u8 {
    !ct_eq_u8(a, b)
}

#[inline(always)]
pub fn ct_select_u8(condition: u8, if_true: u8, if_false: u8) -> u8 {
    let mask = ct_expand_mask(condition);
    (mask & if_true) | (!mask & if_false)
}

#[inline(always)]
pub fn ct_select_i8(condition: u8, if_true: i8, if_false: i8) -> i8 {
    let mask = ct_expand_mask(condition) as i8;
    (mask & if_true) | (!mask & if_false)
}

#[inline(always)]
fn ct_expand_mask(bit: u8) -> u8 {
    0u8.wrapping_sub(bit & 1)
}

#[inline(always)]
pub fn ct_select_u32(condition: u8, if_true: u32, if_false: u32) -> u32 {
    let mask = 0u32.wrapping_sub((condition & 1) as u32);
    (mask & if_true) | (!mask & if_false)
}

pub fn ct_select_slice(condition: u8, if_true: &[i8], if_false: &[i8], output: &mut [i8]) {
    let len = output.len().min(if_true.len()).min(if_false.len());
    let mask = ct_expand_mask(condition) as i8;
    for i in 0..len {
        output[i] = (mask & if_true[i]) | (!mask & if_false[i]);
    }
}

pub fn ct_select_vec(condition: u8, if_true: &[i8], if_false: &[i8]) -> Vec<i8> {
    let len = if_true.len().min(if_false.len());
    let mut result = Vec::with_capacity(len);
    let mask = ct_expand_mask(condition) as i8;
    for i in 0..len {
        result.push((mask & if_true[i]) | (!mask & if_false[i]));
    }
    result
}

pub fn ct_cmov_slice(condition: u8, target: &mut [i8], source: &[i8]) {
    let len = target.len().min(source.len());
    let mask = ct_expand_mask(condition) as i8;
    for i in 0..len {
        target[i] = (mask & source[i]) | (!mask & target[i]);
    }
}

pub fn ct_cmov_bytes(condition: u8, target: &mut [u8], source: &[u8]) {
    let len = target.len().min(source.len());
    let mask = ct_expand_mask(condition);
    for i in 0..len {
        target[i] = (mask & source[i]) | (!mask & target[i]);
    }
}

pub fn ct_eq_slices(a: &[i8], b: &[i8]) -> u8 {
    if a.len() != b.len() {
        return 0;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= (a[i] ^ b[i]) as u8;
    }
    ct_eq_u8(diff, 0)
}

pub fn ct_eq_byte_slices(a: &[u8], b: &[u8]) -> u8 {
    if a.len() != b.len() {
        return 0;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    ct_eq_u8(diff, 0)
}

#[inline(always)]
pub fn ct_is_zero(x: u8) -> u8 {
    ct_eq_u8(x, 0)
}

#[inline(always)]
pub fn ct_lt_u32(a: u32, b: u32) -> u8 {
    let diff = (a as u64).wrapping_sub(b as u64);
    ((diff >> 63) & 1) as u8
}

#[inline(always)]
pub fn ct_le_u32(a: u32, b: u32) -> u8 {
    ct_lt_u32(a, b.wrapping_add(1))
}

pub fn ct_lookup_u8(table: &[u8; 256], index: u8) -> u8 {
    let mut result: u8 = 0;
    for i in 0u16..256 {
        let mask = ct_eq_u8(i as u8, index);
        result |= table[i as usize] & mask;
    }
    result
}

pub fn ct_zeroize(data: &mut [u8]) {
    for b in data.iter_mut() {
        unsafe { core::ptr::write_volatile(b, 0) };
    }
}

pub fn ct_zeroize_i8(data: &mut [i8]) {
    for b in data.iter_mut() {
        unsafe { core::ptr::write_volatile(b, 0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_eq_u8_equal() {
        for i in 0..=255u8 {
            assert_eq!(ct_eq_u8(i, i), 0xFF, "ct_eq_u8({}, {}) should be 0xFF", i, i);
        }
    }

    #[test]
    fn test_ct_eq_u8_not_equal() {
        assert_eq!(ct_eq_u8(0, 1), 0x00);
        assert_eq!(ct_eq_u8(42, 43), 0x00);
        assert_eq!(ct_eq_u8(255, 0), 0x00);
    }

    #[test]
    fn test_ct_select_u8() {
        assert_eq!(ct_select_u8(1, 0xAA, 0xBB), 0xAA);
        assert_eq!(ct_select_u8(0, 0xAA, 0xBB), 0xBB);
    }

    #[test]
    fn test_ct_select_i8() {
        assert_eq!(ct_select_i8(1, 1, -1), 1);
        assert_eq!(ct_select_i8(0, 1, -1), -1);
    }

    #[test]
    fn test_ct_select_vec() {
        let a = [1i8, 0, -1, 1];
        let b = [-1i8, 1, 0, -1];
        let r1 = ct_select_vec(1, &a, &b);
        assert_eq!(r1, vec![1, 0, -1, 1]);
        let r0 = ct_select_vec(0, &a, &b);
        assert_eq!(r0, vec![-1, 1, 0, -1]);
    }

    #[test]
    fn test_ct_cmov_slice() {
        let mut target = [0i8, 0, 0];
        let source = [1i8, -1, 1];
        ct_cmov_slice(1, &mut target, &source);
        assert_eq!(target, [1, -1, 1]);

        let mut target2 = [0i8, 0, 0];
        ct_cmov_slice(0, &mut target2, &source);
        assert_eq!(target2, [0, 0, 0]);
    }

    #[test]
    fn test_ct_eq_slices() {
        let a = [1i8, 0, -1];
        let b = [1i8, 0, -1];
        let c = [1i8, 0, 1];
        assert_eq!(ct_eq_slices(&a, &b), 0xFF);
        assert_eq!(ct_eq_slices(&a, &c), 0x00);
    }

    #[test]
    fn test_ct_eq_slices_different_length() {
        let a = [1i8, 0];
        let b = [1i8, 0, -1];
        assert_eq!(ct_eq_slices(&a, &b), 0);
    }

    #[test]
    fn test_ct_is_zero() {
        assert_eq!(ct_is_zero(0), 0xFF);
        assert_eq!(ct_is_zero(1), 0x00);
        assert_eq!(ct_is_zero(255), 0x00);
    }

    #[test]
    fn test_ct_lt_u32() {
        assert_eq!(ct_lt_u32(0, 1), 1);
        assert_eq!(ct_lt_u32(1, 0), 0);
        assert_eq!(ct_lt_u32(5, 5), 0);
        assert_eq!(ct_lt_u32(100, 200), 1);
    }

    #[test]
    fn test_ct_lookup_u8() {
        let mut table = [0u8; 256];
        for i in 0..256 {
            table[i] = i as u8;
        }
        for i in 0..=255u8 {
            assert_eq!(ct_lookup_u8(&table, i), i);
        }
    }

    #[test]
    fn test_ct_zeroize() {
        let mut data = [0xAAu8; 16];
        ct_zeroize(&mut data);
        assert_eq!(data, [0u8; 16]);
    }

    #[test]
    fn test_ct_cmov_bytes() {
        let mut target = [0u8; 4];
        let source = [0xAA, 0xBB, 0xCC, 0xDD];
        ct_cmov_bytes(1, &mut target, &source);
        assert_eq!(target, source);

        let mut target2 = [0u8; 4];
        ct_cmov_bytes(0, &mut target2, &source);
        assert_eq!(target2, [0, 0, 0, 0]);
    }

    #[test]
    fn test_ct_eq_byte_slices() {
        let a = [0xAA, 0xBB, 0xCC];
        let b = [0xAA, 0xBB, 0xCC];
        let c = [0xAA, 0xBB, 0xDD];
        assert_eq!(ct_eq_byte_slices(&a, &b), 0xFF);
        assert_eq!(ct_eq_byte_slices(&a, &c), 0x00);
    }
}
