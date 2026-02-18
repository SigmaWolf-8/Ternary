// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPlenum RISC-V Extension — Inline Assembly Wrappers
// All 21 custom instructions + 12 CSR accessors
//
// Instruction encoding: custom-0 opcode (0x0B)
// .insn r opcode, funct3, funct7, rd, rs1, rs2

#![allow(unused_unsafe)]

use core::arch::asm;

// ============================================================================
// CSR Addresses (0x7C0–0x7CB)
// ============================================================================
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

// XPSTATUS bit masks
pub const XPSTATUS_MASK_EN: u32 = 1 << 0;
pub const XPSTATUS_DOM_EN: u32 = 1 << 1;
pub const XPSTATUS_CAP_EN: u32 = 1 << 2;
pub const XPSTATUS_SIG_EN: u32 = 1 << 3;

// Exception codes
pub const XP_EXC_NONE: u32 = 0x0;
pub const XP_EXC_DOM_VIOLATION: u32 = 0x1;
pub const XP_EXC_CAP_INVALID: u32 = 0x2;
pub const XP_EXC_CAP_REVOKED: u32 = 0x3;
pub const XP_EXC_CAP_BOUNDS: u32 = 0x4;
pub const XP_EXC_MASK_FAULT: u32 = 0x5;
pub const XP_EXC_TRIT_OVERFLOW: u32 = 0x6;
pub const XP_EXC_PRIV_FAULT: u32 = 0x7;

// ============================================================================
// CSR Read/Write Helpers
// ============================================================================

#[inline(always)]
pub unsafe fn csrr_xpstatus() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C0", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xpstatus(val: u32) {
    asm!("csrw 0x7C0, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpdomid() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C1", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xpdomid(val: u32) {
    asm!("csrw 0x7C1, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpcapbase() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C2", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xpcapbase(val: u32) {
    asm!("csrw 0x7C2, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpcapbound() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C3", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xpcapbound(val: u32) {
    asm!("csrw 0x7C3, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrw_xpmask_seed(val: u32) {
    asm!("csrw 0x7C4, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpmask_state() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C5", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrr_xptrit_mode() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C6", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xptrit_mode(val: u32) {
    asm!("csrw 0x7C6, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpsig_cfg() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C7", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrw_xpsig_cfg(val: u32) {
    asm!("csrw 0x7C7, {}", in(reg) val, options(nomem, nostack));
}

#[inline(always)]
pub unsafe fn csrr_xpexc_cause() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C8", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrr_xpexc_addr() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7C9", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrr_xpperf_cnt() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7CA", out(reg) val, options(nomem, nostack));
    val
}

#[inline(always)]
pub unsafe fn csrr_xpversion() -> u32 {
    let val: u32;
    asm!("csrr {}, 0x7CB", out(reg) val, options(nomem, nostack));
    val
}

// ============================================================================
// Masking Instructions (funct3=0b000)
// ============================================================================

/// TMASK rd, rs1, rs2 — Apply ternary mask
/// rd = trit_add(rs1, rs2) for each of 16 trits
#[inline(always)]
pub unsafe fn tmask(data: u32, mask: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 0, 0, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) mask,
        options(nomem, nostack),
    );
    result
}

/// TUNMASK rd, rs1, rs2 — Remove ternary mask
/// rd = trit_sub(rs1, rs2) for each of 16 trits
#[inline(always)]
pub unsafe fn tunmask(data: u32, mask: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 0, 1, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) mask,
        options(nomem, nostack),
    );
    result
}

/// TMASKR rd, rs1 — Generate random mask from CTR_DRBG + apply
/// Mask state updated; rd = trit_add(rs1, random_mask)
#[inline(always)]
pub unsafe fn tmaskr(data: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 0, 2, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) data,
        options(nomem, nostack),
    );
    result
}

/// TMASKRF rd, rs1 — Refresh mask (unmask old, remask new)
/// rd = trit_add(trit_sub(rs1, old_mask), new_random_mask)
#[inline(always)]
pub unsafe fn tmaskrf(data: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 0, 3, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) data,
        options(nomem, nostack),
    );
    result
}

// ============================================================================
// Domain Isolation Instructions (funct3=0b001)
// ============================================================================

/// TDOMSET rd, rs1, rs2 — Set domain tag
/// rs1 = target address/index, rs2 = domain descriptor
#[inline(always)]
pub unsafe fn tdomset(target: u32, descriptor: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 1, 0, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) target,
        rs2 = in(reg) descriptor,
        options(nomem, nostack),
    );
    result
}

/// TDOMCHK rd, rs1, rs2 — Check domain permission
/// rs1 = target, rs2 = required permission mask
/// rd = 1 if permitted, 0 if denied (exception on failure)
#[inline(always)]
pub unsafe fn tdomchk(target: u32, perm_mask: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 1, 1, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) target,
        rs2 = in(reg) perm_mask,
        options(nomem, nostack),
    );
    result
}

/// TDOMCLR rd, rs1 — Clear domain tag
/// rs1 = target index
#[inline(always)]
pub unsafe fn tdomclr(target: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 1, 2, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) target,
        options(nomem, nostack),
    );
    result
}

/// TDOMXFR rd, rs1, rs2 — Transfer domain ownership
/// rs1 = source domain, rs2 = destination domain
#[inline(always)]
pub unsafe fn tdomxfr(src_dom: u32, dst_dom: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 1, 3, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) src_dom,
        rs2 = in(reg) dst_dom,
        options(nomem, nostack),
    );
    result
}

// ============================================================================
// Capability Instructions (funct3=0b010)
// ============================================================================

/// TCAPLD rd, rs1 — Load capability descriptor
/// rs1 = capability index, rd = loaded capability descriptor (lower 32 bits)
#[inline(always)]
pub unsafe fn tcapld(index: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 2, 0, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) index,
        options(nomem, nostack),
    );
    result
}

/// TCAPCHK rd, rs1, rs2 — Check capability permission
/// rs1 = capability index, rs2 = address to check
/// rd = 1 if valid, exception if invalid
#[inline(always)]
pub unsafe fn tcapchk(index: u32, addr: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 2, 1, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) index,
        rs2 = in(reg) addr,
        options(nomem, nostack),
    );
    result
}

/// TCAPST rd, rs1, rs2 — Store capability descriptor
/// rs1 = capability index, rs2 = descriptor value
#[inline(always)]
pub unsafe fn tcapst(index: u32, descriptor: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 2, 2, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) index,
        rs2 = in(reg) descriptor,
        options(nomem, nostack),
    );
    result
}

/// TCAPREV rd, rs1 — Revoke capability
/// rs1 = capability index
#[inline(always)]
pub unsafe fn tcaprev(index: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 2, 3, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) index,
        options(nomem, nostack),
    );
    result
}

// ============================================================================
// Ternary Cryptographic Primitives (funct3=0b011)
// ============================================================================

/// TROTL rd, rs1, rs2 — Ternary rotate left
/// Rotates 16-trit word in rs1 left by rs2[3:0] trit positions
#[inline(always)]
pub unsafe fn trotl(data: u32, amount: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 3, 0, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) amount,
        options(nomem, nostack),
    );
    result
}

/// TROTR rd, rs1, rs2 — Ternary rotate right
/// Rotates 16-trit word in rs1 right by rs2[3:0] trit positions
#[inline(always)]
pub unsafe fn trotr(data: u32, amount: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 3, 1, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) amount,
        options(nomem, nostack),
    );
    result
}

/// TTBOX rd, rs1, rs2 — Ternary substitution box lookup
/// rd = TBOX[rs1 % 27] XOR rs2 (nonlinear mixing)
#[inline(always)]
pub unsafe fn ttbox(index: u32, mix: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 3, 2, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) index,
        rs2 = in(reg) mix,
        options(nomem, nostack),
    );
    result
}

/// TPERM rd, rs1, rs2 — Ternary permutation
/// Permutes trits in rs1 according to permutation pattern in rs2
#[inline(always)]
pub unsafe fn tperm(data: u32, pattern: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 3, 3, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) data,
        rs2 = in(reg) pattern,
        options(nomem, nostack),
    );
    result
}

// ============================================================================
// Trit Encoding/Decoding (funct3=0b100)
// ============================================================================

/// TTRIT rd, rs1 — Binary to ternary encoding
/// Converts binary value in rs1 to balanced ternary representation
#[inline(always)]
pub unsafe fn ttrit(binary_val: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 4, 0, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) binary_val,
        options(nomem, nostack),
    );
    result
}

/// TDETRIT rd, rs1 — Ternary to binary decoding
/// Converts balanced ternary representation in rs1 to binary value
#[inline(always)]
pub unsafe fn tdetrit(trit_val: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 4, 1, {rd}, {rs1}, x0",
        rd = out(reg) result,
        rs1 = in(reg) trit_val,
        options(nomem, nostack),
    );
    result
}

// ============================================================================
// Signal Processing (funct3=0b101)
// ============================================================================

/// TSIGFLT rd, rs1, rs2 — Ternary signal filter
/// Applies FIR filter with coefficients from rs2 to signal in rs1
#[inline(always)]
pub unsafe fn tsigflt(signal: u32, coefficients: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 5, 0, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) signal,
        rs2 = in(reg) coefficients,
        options(nomem, nostack),
    );
    result
}

/// TSIGCMP rd, rs1, rs2 — Ternary signal compare
/// Computes trit-wise comparison metric between rs1 and rs2
#[inline(always)]
pub unsafe fn tsigcmp(signal_a: u32, signal_b: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 5, 1, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) signal_a,
        rs2 = in(reg) signal_b,
        options(nomem, nostack),
    );
    result
}

/// TSIGACC rd, rs1, rs2 — Ternary signal accumulate
/// Accumulates ternary signal values: acc += rs1 * rs2 (trit-wise)
#[inline(always)]
pub unsafe fn tsigacc(signal: u32, weight: u32) -> u32 {
    let result: u32;
    asm!(
        ".insn r 0x0B, 5, 2, {rd}, {rs1}, {rs2}",
        rd = out(reg) result,
        rs1 = in(reg) signal,
        rs2 = in(reg) weight,
        options(nomem, nostack),
    );
    result
}
