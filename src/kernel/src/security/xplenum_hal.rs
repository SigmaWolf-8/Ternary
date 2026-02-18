// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPlenum Hardware Abstraction Layer (HAL)
// Safe Rust wrappers for XPlenum RISC-V ternary security extension
// Provides: XPlenumMask, XPlenumDomain, XPlenumCap, XPlenumCrypto, XPlenumTrit

use crate::arch::xplenum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XPlenumError {
    SubsystemDisabled,
    DomainViolation,
    CapabilityInvalid,
    CapabilityRevoked,
    CapabilityBounds,
    MaskFault,
    TritOverflow,
    PrivilegeFault,
    HardwareUnavailable,
    HealthCheckFailed,
}

impl XPlenumError {
    pub fn from_exc_code(code: u32) -> Option<Self> {
        match code {
            xplenum::XP_EXC_NONE => None,
            xplenum::XP_EXC_DOM_VIOLATION => Some(Self::DomainViolation),
            xplenum::XP_EXC_CAP_INVALID => Some(Self::CapabilityInvalid),
            xplenum::XP_EXC_CAP_REVOKED => Some(Self::CapabilityRevoked),
            xplenum::XP_EXC_CAP_BOUNDS => Some(Self::CapabilityBounds),
            xplenum::XP_EXC_MASK_FAULT => Some(Self::MaskFault),
            xplenum::XP_EXC_TRIT_OVERFLOW => Some(Self::TritOverflow),
            xplenum::XP_EXC_PRIV_FAULT => Some(Self::PrivilegeFault),
            _ => Some(Self::HardwareUnavailable),
        }
    }
}

pub type XPlenumResult<T> = Result<T, XPlenumError>;

fn check_exception() -> XPlenumResult<()> {
    let exc = unsafe { xplenum::csrr_xpexc_cause() };
    match XPlenumError::from_exc_code(exc) {
        None => Ok(()),
        Some(e) => Err(e),
    }
}

// ============================================================================
// XPlenumStatus — Global Extension Status
// ============================================================================

pub struct XPlenumStatus;

impl XPlenumStatus {
    pub fn read() -> u32 {
        unsafe { xplenum::csrr_xpstatus() }
    }

    pub fn mask_enabled() -> bool {
        Self::read() & xplenum::XPSTATUS_MASK_EN != 0
    }

    pub fn domain_enabled() -> bool {
        Self::read() & xplenum::XPSTATUS_DOM_EN != 0
    }

    pub fn capability_enabled() -> bool {
        Self::read() & xplenum::XPSTATUS_CAP_EN != 0
    }

    pub fn signal_enabled() -> bool {
        Self::read() & xplenum::XPSTATUS_SIG_EN != 0
    }

    pub fn enable_mask() {
        let val = Self::read() | xplenum::XPSTATUS_MASK_EN;
        unsafe { xplenum::csrw_xpstatus(val) };
    }

    pub fn enable_domain() {
        let val = Self::read() | xplenum::XPSTATUS_DOM_EN;
        unsafe { xplenum::csrw_xpstatus(val) };
    }

    pub fn enable_capability() {
        let val = Self::read() | xplenum::XPSTATUS_CAP_EN;
        unsafe { xplenum::csrw_xpstatus(val) };
    }

    pub fn enable_all() {
        unsafe {
            xplenum::csrw_xpstatus(
                xplenum::XPSTATUS_MASK_EN
                    | xplenum::XPSTATUS_DOM_EN
                    | xplenum::XPSTATUS_CAP_EN
                    | xplenum::XPSTATUS_SIG_EN,
            );
        }
    }

    pub fn version() -> u32 {
        unsafe { xplenum::csrr_xpversion() }
    }

    pub fn perf_counter() -> u32 {
        unsafe { xplenum::csrr_xpperf_cnt() }
    }
}

// ============================================================================
// XPlenumMask — Ternary Masking Operations
// ============================================================================

pub struct XPlenumMask;

impl XPlenumMask {
    pub fn apply(data: u32, mask: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::mask_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tmask(data, mask) };
        check_exception()?;
        Ok(result)
    }

    pub fn remove(data: u32, mask: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::mask_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tunmask(data, mask) };
        check_exception()?;
        Ok(result)
    }

    pub fn apply_random(data: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::mask_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tmaskr(data) };
        check_exception()?;
        Ok(result)
    }

    pub fn refresh(data: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::mask_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tmaskrf(data) };
        check_exception()?;
        Ok(result)
    }

    pub fn current_state() -> u32 {
        unsafe { xplenum::csrr_xpmask_state() }
    }

    pub fn seed_drbg(seed: u32) {
        unsafe { xplenum::csrw_xpmask_seed(seed) };
    }
}

// ============================================================================
// XPlenumDomain — Domain Isolation Operations
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct DomainDescriptor {
    pub owner: u8,
    pub permissions: u8,
    pub transfer_auth: u8,
    pub state: u8,
}

impl DomainDescriptor {
    pub fn to_u32(self) -> u32 {
        ((self.owner as u32) << 24)
            | ((self.permissions as u32) << 16)
            | ((self.transfer_auth as u32) << 8)
            | (self.state as u32)
    }

    pub fn from_u32(val: u32) -> Self {
        Self {
            owner: ((val >> 24) & 0xFF) as u8,
            permissions: ((val >> 16) & 0xFF) as u8,
            transfer_auth: ((val >> 8) & 0xFF) as u8,
            state: (val & 0xFF) as u8,
        }
    }
}

pub const DOM_PERM_READ: u8 = 1 << 0;
pub const DOM_PERM_WRITE: u8 = 1 << 1;
pub const DOM_PERM_EXEC: u8 = 1 << 2;
pub const DOM_PERM_CROSS: u8 = 1 << 3;

pub const DOM_STATE_INVALID: u8 = 0x00;
pub const DOM_STATE_ACTIVE: u8 = 0x01;
pub const DOM_STATE_LOCKED: u8 = 0x02;
pub const DOM_STATE_TRANSFER: u8 = 0x03;

pub struct XPlenumDomain;

impl XPlenumDomain {
    pub fn current_id() -> u32 {
        unsafe { xplenum::csrr_xpdomid() }
    }

    pub fn set_id(domain_id: u32) {
        unsafe { xplenum::csrw_xpdomid(domain_id) };
    }

    pub fn set_tag(target: u32, descriptor: DomainDescriptor) -> XPlenumResult<u32> {
        if !XPlenumStatus::domain_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tdomset(target, descriptor.to_u32()) };
        check_exception()?;
        Ok(result)
    }

    pub fn check_permission(target: u32, required_perms: u8) -> XPlenumResult<bool> {
        if !XPlenumStatus::domain_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tdomchk(target, required_perms as u32) };
        check_exception()?;
        Ok(result != 0)
    }

    pub fn clear_tag(target: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::domain_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tdomclr(target) };
        check_exception()?;
        Ok(result)
    }

    pub fn transfer(src_domain: u32, dst_domain: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::domain_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tdomxfr(src_domain, dst_domain) };
        check_exception()?;
        Ok(result)
    }
}

// ============================================================================
// XPlenumCap — Capability-Based Access Control
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct CapabilityDescriptor {
    pub tag: u8,
    pub permissions: u8,
    pub base: u16,
    pub bound: u16,
    pub otype: u8,
    pub seal: u8,
}

pub const SEAL_OPEN: u8 = 0x00;
pub const SEAL_SEALED: u8 = 0x01;
pub const SEAL_FROZEN: u8 = 0x02;

impl CapabilityDescriptor {
    pub fn is_valid(&self) -> bool {
        self.tag != 0
    }

    pub fn is_sealed(&self) -> bool {
        self.seal == SEAL_SEALED || self.seal == SEAL_FROZEN
    }
}

pub struct XPlenumCap;

impl XPlenumCap {
    pub fn configure_table(base: u32, bound: u32) {
        unsafe {
            xplenum::csrw_xpcapbase(base);
            xplenum::csrw_xpcapbound(bound);
        }
    }

    pub fn table_base() -> u32 {
        unsafe { xplenum::csrr_xpcapbase() }
    }

    pub fn table_bound() -> u32 {
        unsafe { xplenum::csrr_xpcapbound() }
    }

    pub fn load(index: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::capability_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tcapld(index) };
        check_exception()?;
        Ok(result)
    }

    pub fn check(index: u32, addr: u32) -> XPlenumResult<bool> {
        if !XPlenumStatus::capability_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tcapchk(index, addr) };
        check_exception()?;
        Ok(result != 0)
    }

    pub fn store(index: u32, descriptor: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::capability_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tcapst(index, descriptor) };
        check_exception()?;
        Ok(result)
    }

    pub fn revoke(index: u32) -> XPlenumResult<u32> {
        if !XPlenumStatus::capability_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        let result = unsafe { xplenum::tcaprev(index) };
        check_exception()?;
        Ok(result)
    }
}

// ============================================================================
// XPlenumCrypto — Ternary Cryptographic Primitives
// ============================================================================

pub struct XPlenumCrypto;

impl XPlenumCrypto {
    fn check_enabled() -> XPlenumResult<()> {
        let status = XPlenumStatus::read();
        if status & xplenum::XPSTATUS_MASK_EN == 0 {
            return Err(XPlenumError::SubsystemDisabled);
        }
        Ok(())
    }

    pub fn rotate_left(data: u32, amount: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::trotl(data, amount) };
        check_exception()?;
        Ok(result)
    }

    pub fn rotate_right(data: u32, amount: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::trotr(data, amount) };
        check_exception()?;
        Ok(result)
    }

    pub fn sbox_lookup(index: u32, mix: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::ttbox(index, mix) };
        check_exception()?;
        Ok(result)
    }

    pub fn permute(data: u32, pattern: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::tperm(data, pattern) };
        check_exception()?;
        Ok(result)
    }

    pub fn ternary_round(state: u32, round_key: u32, round_num: u32) -> XPlenumResult<u32> {
        let rotated = Self::rotate_left(state, round_num)?;
        let substituted = Self::sbox_lookup(rotated, round_key)?;
        Self::permute(substituted, round_num)
    }
}

// ============================================================================
// XPlenumTrit — Trit Encoding/Decoding
// ============================================================================

pub struct XPlenumTrit;

impl XPlenumTrit {
    pub fn encode(binary_val: u32) -> XPlenumResult<u32> {
        let result = unsafe { xplenum::ttrit(binary_val) };
        check_exception()?;
        Ok(result)
    }

    pub fn decode(trit_val: u32) -> XPlenumResult<u32> {
        let result = unsafe { xplenum::tdetrit(trit_val) };
        check_exception()?;
        Ok(result)
    }

    pub fn mode() -> u32 {
        unsafe { xplenum::csrr_xptrit_mode() }
    }

    pub fn set_mode(mode: u32) {
        unsafe { xplenum::csrw_xptrit_mode(mode) };
    }
}

// ============================================================================
// XPlenumSignal — Signal Processing
// ============================================================================

pub struct XPlenumSignal;

impl XPlenumSignal {
    fn check_enabled() -> XPlenumResult<()> {
        if !XPlenumStatus::signal_enabled() {
            return Err(XPlenumError::SubsystemDisabled);
        }
        Ok(())
    }

    pub fn filter(signal: u32, coefficients: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::tsigflt(signal, coefficients) };
        check_exception()?;
        Ok(result)
    }

    pub fn compare(signal_a: u32, signal_b: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::tsigcmp(signal_a, signal_b) };
        check_exception()?;
        Ok(result)
    }

    pub fn accumulate(signal: u32, weight: u32) -> XPlenumResult<u32> {
        Self::check_enabled()?;
        let result = unsafe { xplenum::tsigacc(signal, weight) };
        check_exception()?;
        Ok(result)
    }

    pub fn config() -> u32 {
        unsafe { xplenum::csrr_xpsig_cfg() }
    }

    pub fn set_config(cfg: u32) {
        unsafe { xplenum::csrw_xpsig_cfg(cfg) };
    }
}
