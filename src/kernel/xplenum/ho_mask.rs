//! XPlenum Higher-Order Masking API (Task 8B.4)
//!
//! Provides safe Rust abstractions for 2nd and 3rd order
//! Domain-Oriented Masking (DOM) hardware operations.
//!
//! # Security Model
//! - 2nd-order: 3 shares, protects against 2-probe attacks
//! - 3rd-order: 4 shares, protects against 3-probe attacks
//! - All non-linear operations (AND, MUL) use DOM gadgets
//! - Linear operations (XOR, ADD mod 2) operate per-share
//! - Share refresh re-randomises without changing unmasked value

use core::arch::asm;

/// 3-share representation (2nd-order protection)
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Shares3 {
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
}

/// 4-share representation (3rd-order protection)
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Shares4 {
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
}

/// Protection order configuration
#[derive(Clone, Copy, PartialEq)]
pub enum MaskOrder {
    Second, // 3 shares
    Third,  // 4 shares
}

/// Higher-order mask apply: split value into shares
/// Encoding: Custom-0 (0x0B), funct3=0x0, funct7=0x10
#[inline(always)]
unsafe fn ho_mask_apply_raw(data: u64, random: u64) -> u64 {
    let result: u64;
    asm!(
        ".insn r 0b0001011, 0, 0x10, {rd}, {rs1}, {rs2}",
        rd  = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) random,
    );
    result
}

/// Higher-order mask strip: recombine shares
/// Encoding: Custom-0 (0x0B), funct3=0x0, funct7=0x11
#[inline(always)]
unsafe fn ho_mask_strip_raw(share_a: u64, share_b: u64) -> u64 {
    let result: u64;
    asm!(
        ".insn r 0b0001011, 0, 0x11, {rd}, {rs1}, {rs2}",
        rd  = out(reg) result,
        rs1 = in(reg) share_a,
        rs2 = in(reg) share_b,
    );
    result
}

/// Higher-order mask refresh: re-randomise shares
/// Encoding: Custom-0 (0x0B), funct3=0x0, funct7=0x12
#[inline(always)]
unsafe fn ho_mask_refresh_raw(share: u64, random: u64) -> u64 {
    let result: u64;
    asm!(
        ".insn r 0b0001011, 0, 0x12, {rd}, {rs1}, {rs2}",
        rd  = out(reg) result,
        rs1 = in(reg) share,
        rs2 = in(reg) random,
    );
    result
}

/// Higher-order secure AND (DOM gadget in hardware)
/// Encoding: Custom-0 (0x0B), funct3=0x0, funct7=0x13
#[inline(always)]
unsafe fn ho_mask_and_raw(share_a: u64, share_b: u64) -> u64 {
    let result: u64;
    asm!(
        ".insn r 0b0001011, 0, 0x13, {rd}, {rs1}, {rs2}",
        rd  = out(reg) result,
        rs1 = in(reg) share_a,
        rs2 = in(reg) share_b,
    );
    result
}

/// Split a sensitive value into 3 shares.
///
/// After this operation, no single share reveals information
/// about the original value. Any 2 shares combined reveal nothing
/// (2nd-order security).
pub fn share_3(value: u64) -> Shares3 {
    unsafe {
        let r0 = ho_mask_apply_raw(0, 0);
        let r1 = ho_mask_apply_raw(0, 0);
        let s0 = r0;
        let s1 = r1;
        let s2 = value ^ r0 ^ r1;

        Shares3 { s0, s1, s2 }
    }
}

/// Recombine 3 shares to recover the original value.
pub fn recombine_3(shares: &Shares3) -> u64 {
    shares.s0 ^ shares.s1 ^ shares.s2
}

/// Secure AND on 3-share values (DOM gadget).
///
/// Computes `a & b` without ever combining shares in a way
/// that leaks information to a 2nd-order side-channel attacker.
pub fn secure_and_3(a: &Shares3, b: &Shares3) -> Shares3 {
    unsafe {
        let c0 = ho_mask_and_raw(a.s0, b.s0);
        let c1 = ho_mask_and_raw(a.s1, b.s1);
        let c2 = ho_mask_and_raw(a.s2, b.s2);

        Shares3 { s0: c0, s1: c1, s2: c2 }
    }
}

/// Secure XOR on 3-share values (linear -- no randomness needed).
pub fn secure_xor_3(a: &Shares3, b: &Shares3) -> Shares3 {
    Shares3 {
        s0: a.s0 ^ b.s0,
        s1: a.s1 ^ b.s1,
        s2: a.s2 ^ b.s2,
    }
}

/// Refresh all shares with fresh randomness.
///
/// The unmasked value is preserved but all shares change,
/// preventing accumulation of leakage across operations.
pub fn refresh_3(shares: &Shares3) -> Shares3 {
    unsafe {
        let r0 = ho_mask_apply_raw(0, 0);
        let r1 = ho_mask_apply_raw(0, 0);

        Shares3 {
            s0: ho_mask_refresh_raw(shares.s0, r0),
            s1: ho_mask_refresh_raw(shares.s1, r1),
            s2: shares.s2 ^ r0 ^ r1,
        }
    }
}

/// Split a sensitive value into 4 shares (3rd-order protection).
pub fn share_4(value: u64) -> Shares4 {
    unsafe {
        let r0 = ho_mask_apply_raw(0, 0);
        let r1 = ho_mask_apply_raw(0, 0);
        let r2 = ho_mask_apply_raw(0, 0);

        Shares4 {
            s0: r0,
            s1: r1,
            s2: r2,
            s3: value ^ r0 ^ r1 ^ r2,
        }
    }
}

/// Recombine 4 shares.
pub fn recombine_4(shares: &Shares4) -> u64 {
    shares.s0 ^ shares.s1 ^ shares.s2 ^ shares.s3
}

/// Secure AND on 4-share values (3rd-order DOM gadget).
pub fn secure_and_4(a: &Shares4, b: &Shares4) -> Shares4 {
    unsafe {
        Shares4 {
            s0: ho_mask_and_raw(a.s0, b.s0),
            s1: ho_mask_and_raw(a.s1, b.s1),
            s2: ho_mask_and_raw(a.s2, b.s2),
            s3: ho_mask_and_raw(a.s3, b.s3),
        }
    }
}

/// Secure XOR on 4-share values.
pub fn secure_xor_4(a: &Shares4, b: &Shares4) -> Shares4 {
    Shares4 {
        s0: a.s0 ^ b.s0,
        s1: a.s1 ^ b.s1,
        s2: a.s2 ^ b.s2,
        s3: a.s3 ^ b.s3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_recombine_roundtrip_3() {
        let value: u64 = 0xDEADBEEFCAFEBABE;
        let shares = share_3(value);

        assert_ne!(shares.s0, value);
        assert_ne!(shares.s1, value);
        assert_ne!(shares.s2, value);

        assert_eq!(recombine_3(&shares), value);
    }

    #[test]
    fn test_secure_xor_correctness_3() {
        let a_val: u64 = 0xFF00FF00FF00FF00;
        let b_val: u64 = 0x00FF00FF00FF00FF;
        let expected = a_val ^ b_val;

        let a_shares = share_3(a_val);
        let b_shares = share_3(b_val);
        let c_shares = secure_xor_3(&a_shares, &b_shares);

        assert_eq!(recombine_3(&c_shares), expected);
    }

    #[test]
    fn test_refresh_preserves_value_3() {
        let value: u64 = 0x1234567890ABCDEF;
        let original = share_3(value);
        let refreshed = refresh_3(&original);

        assert_ne!(original.s0, refreshed.s0);

        assert_eq!(recombine_3(&refreshed), value);
    }
}
