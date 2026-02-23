// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// XPlenum RISC-V Extension — Software Emulation Stub for non-RISC-V targets
// Provides identical API surface with software-only implementations.
// Constants are identical; instruction functions use arithmetic emulation.

#![allow(unused_variables)]

pub const CSR_XPSTATUS: u16 = 0x7C0;
pub const CSR_XPDOMID: u16 = 0x7C1;
pub const CSR_XPCAPBASE: u16 = 0x7C2;
pub const CSR_XPCAPBOUND: u16 = 0x7C3;
pub const CSR_XPMASK_SEED: u16 = 0x7C4;
pub const CSR_XPMASK_STATE: u16 = 0x7C5;
pub const CSR_XPTRIT_MODE: u16 = 0x7C6;
pub const CSR_XPSIG_CFG: u16 = 0x7C7;
pub const CSR_XPEXC_CAUSE: u16 = 0x7C8;
pub const CSR_XPEXC_ADDR: u16 = 0x7C9;
pub const CSR_XPPERF_CNT: u16 = 0x7CA;
pub const CSR_XPVERSION: u16 = 0x7CB;

pub const XPSTATUS_MASK_EN: u32 = 1 << 0;
pub const XPSTATUS_DOM_EN: u32 = 1 << 1;
pub const XPSTATUS_CAP_EN: u32 = 1 << 2;
pub const XPSTATUS_SIG_EN: u32 = 1 << 3;

pub const XP_EXC_NONE: u32 = 0x0;
pub const XP_EXC_DOM_VIOLATION: u32 = 0x1;
pub const XP_EXC_CAP_INVALID: u32 = 0x2;
pub const XP_EXC_CAP_REVOKED: u32 = 0x3;
pub const XP_EXC_CAP_BOUNDS: u32 = 0x4;
pub const XP_EXC_MASK_FAULT: u32 = 0x5;
pub const XP_EXC_TRIT_OVERFLOW: u32 = 0x6;
pub const XP_EXC_PRIV_FAULT: u32 = 0x7;

fn trit_add_word(a: u32, b: u32) -> u32 {
    let mut result = 0u32;
    for i in 0..16 {
        let ta = ((a >> (i * 2)) & 0x3) as i8 - 1;
        let tb = ((b >> (i * 2)) & 0x3) as i8 - 1;
        let sum = ((ta + tb).rem_euclid(3) - 1 + 1) as u32;
        result |= (sum & 0x3) << (i * 2);
    }
    result
}

fn trit_sub_word(a: u32, b: u32) -> u32 {
    let mut result = 0u32;
    for i in 0..16 {
        let ta = ((a >> (i * 2)) & 0x3) as i8 - 1;
        let tb = ((b >> (i * 2)) & 0x3) as i8 - 1;
        let diff = ((ta - tb).rem_euclid(3) - 1 + 1) as u32;
        result |= (diff & 0x3) << (i * 2);
    }
    result
}

pub unsafe fn csrr_xpstatus() -> u32 { 0 }
pub unsafe fn csrw_xpstatus(val: u32) {}
pub unsafe fn csrr_xpdomid() -> u32 { 0 }
pub unsafe fn csrw_xpdomid(val: u32) {}
pub unsafe fn csrr_xpcapbase() -> u32 { 0 }
pub unsafe fn csrw_xpcapbase(val: u32) {}
pub unsafe fn csrr_xpcapbound() -> u32 { 0 }
pub unsafe fn csrw_xpcapbound(val: u32) {}
pub unsafe fn csrw_xpmask_seed(val: u32) {}
pub unsafe fn csrr_xpmask_state() -> u32 { 0 }
pub unsafe fn csrr_xptrit_mode() -> u32 { 0 }
pub unsafe fn csrw_xptrit_mode(val: u32) {}
pub unsafe fn csrr_xpsig_cfg() -> u32 { 0 }
pub unsafe fn csrw_xpsig_cfg(val: u32) {}
pub unsafe fn csrr_xpexc_cause() -> u32 { 0 }
pub unsafe fn csrr_xpexc_addr() -> u32 { 0 }
pub unsafe fn csrr_xpperf_cnt() -> u32 { 0 }
pub unsafe fn csrr_xpversion() -> u32 { 0x0100 }

pub unsafe fn tmask(data: u32, mask: u32) -> u32 { trit_add_word(data, mask) }
pub unsafe fn tunmask(data: u32, mask: u32) -> u32 { trit_sub_word(data, mask) }
pub unsafe fn tmaskr(data: u32) -> u32 { data }
pub unsafe fn tmaskrf(data: u32) -> u32 { data }

pub unsafe fn tdomset(target: u32, descriptor: u32) -> u32 { 1 }
pub unsafe fn tdomchk(target: u32, perm_mask: u32) -> u32 { 1 }
pub unsafe fn tdomclr(target: u32) -> u32 { 1 }
pub unsafe fn tdomxfr(src_dom: u32, dst_dom: u32) -> u32 { 1 }

pub unsafe fn tcapld(index: u32) -> u32 { 0 }
pub unsafe fn tcapchk(index: u32, addr: u32) -> u32 { 1 }
pub unsafe fn tcapst(index: u32, descriptor: u32) -> u32 { 1 }
pub unsafe fn tcaprev(index: u32) -> u32 { 1 }

pub unsafe fn trotl(data: u32, amount: u32) -> u32 {
    let shift = (amount & 0xF) * 2;
    (data << shift) | (data >> (32 - shift))
}
pub unsafe fn trotr(data: u32, amount: u32) -> u32 {
    let shift = (amount & 0xF) * 2;
    (data >> shift) | (data << (32 - shift))
}
pub unsafe fn ttbox(index: u32, mix: u32) -> u32 {
    let sbox_val = index.wrapping_mul(0x9E3779B9).wrapping_add(0x6A09E667);
    sbox_val ^ mix
}
pub unsafe fn tperm(data: u32, pattern: u32) -> u32 {
    let mut result = 0u32;
    for i in 0..16 {
        let src_pos = ((pattern >> (i * 2)) & 0x3) as u32;
        let src_idx = (src_pos + i as u32) % 16;
        let trit = (data >> (src_idx * 2)) & 0x3;
        result |= trit << (i * 2);
    }
    result
}

pub unsafe fn ttrit(binary_val: u32) -> u32 {
    let mut val = binary_val;
    let mut result = 0u32;
    for i in 0..16 {
        let rem = val % 3;
        let trit = rem as u32;
        result |= trit << (i * 2);
        val /= 3;
    }
    result
}
pub unsafe fn tdetrit(trit_val: u32) -> u32 {
    let mut result = 0u32;
    let mut power = 1u32;
    for i in 0..16 {
        let trit = ((trit_val >> (i * 2)) & 0x3) as i8 - 1;
        result = result.wrapping_add((trit as u32).wrapping_mul(power));
        power *= 3;
    }
    result
}

pub unsafe fn tsigflt(signal: u32, coefficients: u32) -> u32 { trit_add_word(signal, coefficients) }
pub unsafe fn tsigcmp(signal_a: u32, signal_b: u32) -> u32 { trit_sub_word(signal_a, signal_b) }
pub unsafe fn tsigacc(signal: u32, weight: u32) -> u32 { trit_add_word(signal, weight) }
