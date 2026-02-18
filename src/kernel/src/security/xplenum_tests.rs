// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPlenum HAL Unit Tests
// Software-emulated tests for all abstraction layer functions
// (Hardware tests require RISC-V target with XPlenum extension)

#[cfg(test)]
mod tests {
    use crate::security::xplenum_hal::*;

    // ========================================================================
    // DomainDescriptor Tests
    // ========================================================================

    #[test]
    fn test_domain_descriptor_roundtrip() {
        let desc = DomainDescriptor {
            owner: 0x42,
            permissions: DOM_PERM_READ | DOM_PERM_WRITE,
            transfer_auth: 0x10,
            state: DOM_STATE_ACTIVE,
        };
        let encoded = desc.to_u32();
        let decoded = DomainDescriptor::from_u32(encoded);

        assert_eq!(decoded.owner, 0x42);
        assert_eq!(decoded.permissions, DOM_PERM_READ | DOM_PERM_WRITE);
        assert_eq!(decoded.transfer_auth, 0x10);
        assert_eq!(decoded.state, DOM_STATE_ACTIVE);
    }

    #[test]
    fn test_domain_descriptor_all_perms() {
        let desc = DomainDescriptor {
            owner: 0xFF,
            permissions: DOM_PERM_READ | DOM_PERM_WRITE | DOM_PERM_EXEC | DOM_PERM_CROSS,
            transfer_auth: 0xFF,
            state: DOM_STATE_LOCKED,
        };
        let val = desc.to_u32();
        assert_eq!(val, 0xFF0FFF02);
    }

    #[test]
    fn test_domain_descriptor_zero() {
        let desc = DomainDescriptor::from_u32(0);
        assert_eq!(desc.owner, 0);
        assert_eq!(desc.permissions, 0);
        assert_eq!(desc.transfer_auth, 0);
        assert_eq!(desc.state, DOM_STATE_INVALID);
    }

    #[test]
    fn test_domain_state_values() {
        assert_eq!(DOM_STATE_INVALID, 0x00);
        assert_eq!(DOM_STATE_ACTIVE, 0x01);
        assert_eq!(DOM_STATE_LOCKED, 0x02);
        assert_eq!(DOM_STATE_TRANSFER, 0x03);
    }

    #[test]
    fn test_domain_perm_bits() {
        assert_eq!(DOM_PERM_READ, 0x01);
        assert_eq!(DOM_PERM_WRITE, 0x02);
        assert_eq!(DOM_PERM_EXEC, 0x04);
        assert_eq!(DOM_PERM_CROSS, 0x08);
    }

    // ========================================================================
    // CapabilityDescriptor Tests
    // ========================================================================

    #[test]
    fn test_capability_valid() {
        let cap = CapabilityDescriptor {
            tag: 0xFF,
            permissions: 0x07,
            base: 0x1000,
            bound: 0x2000,
            otype: 0x01,
            seal: SEAL_OPEN,
        };
        assert!(cap.is_valid());
        assert!(!cap.is_sealed());
    }

    #[test]
    fn test_capability_sealed() {
        let cap = CapabilityDescriptor {
            tag: 0xFF,
            permissions: 0x07,
            base: 0x1000,
            bound: 0x2000,
            otype: 0x01,
            seal: SEAL_SEALED,
        };
        assert!(cap.is_valid());
        assert!(cap.is_sealed());
    }

    #[test]
    fn test_capability_frozen() {
        let cap = CapabilityDescriptor {
            tag: 0xFF,
            permissions: 0x07,
            base: 0x1000,
            bound: 0x2000,
            otype: 0x01,
            seal: SEAL_FROZEN,
        };
        assert!(cap.is_valid());
        assert!(cap.is_sealed());
    }

    #[test]
    fn test_capability_invalid() {
        let cap = CapabilityDescriptor {
            tag: 0x00,
            permissions: 0x00,
            base: 0x0000,
            bound: 0x0000,
            otype: 0x00,
            seal: SEAL_OPEN,
        };
        assert!(!cap.is_valid());
    }

    #[test]
    fn test_seal_values() {
        assert_eq!(SEAL_OPEN, 0x00);
        assert_eq!(SEAL_SEALED, 0x01);
        assert_eq!(SEAL_FROZEN, 0x02);
    }

    // ========================================================================
    // XPlenumError Tests
    // ========================================================================

    #[test]
    fn test_exc_code_none() {
        assert!(XPlenumError::from_exc_code(0x0).is_none());
    }

    #[test]
    fn test_exc_code_domain_violation() {
        assert_eq!(
            XPlenumError::from_exc_code(0x1),
            Some(XPlenumError::DomainViolation)
        );
    }

    #[test]
    fn test_exc_code_cap_invalid() {
        assert_eq!(
            XPlenumError::from_exc_code(0x2),
            Some(XPlenumError::CapabilityInvalid)
        );
    }

    #[test]
    fn test_exc_code_cap_revoked() {
        assert_eq!(
            XPlenumError::from_exc_code(0x3),
            Some(XPlenumError::CapabilityRevoked)
        );
    }

    #[test]
    fn test_exc_code_cap_bounds() {
        assert_eq!(
            XPlenumError::from_exc_code(0x4),
            Some(XPlenumError::CapabilityBounds)
        );
    }

    #[test]
    fn test_exc_code_mask_fault() {
        assert_eq!(
            XPlenumError::from_exc_code(0x5),
            Some(XPlenumError::MaskFault)
        );
    }

    #[test]
    fn test_exc_code_trit_overflow() {
        assert_eq!(
            XPlenumError::from_exc_code(0x6),
            Some(XPlenumError::TritOverflow)
        );
    }

    #[test]
    fn test_exc_code_priv_fault() {
        assert_eq!(
            XPlenumError::from_exc_code(0x7),
            Some(XPlenumError::PrivilegeFault)
        );
    }

    #[test]
    fn test_exc_code_unknown() {
        assert_eq!(
            XPlenumError::from_exc_code(0xFF),
            Some(XPlenumError::HardwareUnavailable)
        );
    }

    // ========================================================================
    // CSR Address Tests
    // ========================================================================

    #[test]
    fn test_csr_addresses() {
        use crate::arch::xplenum;
        assert_eq!(xplenum::CSR_XPSTATUS, 0x7C0);
        assert_eq!(xplenum::CSR_XPDOMID, 0x7C1);
        assert_eq!(xplenum::CSR_XPCAPBASE, 0x7C2);
        assert_eq!(xplenum::CSR_XPCAPBOUND, 0x7C3);
        assert_eq!(xplenum::CSR_XPMASK_SEED, 0x7C4);
        assert_eq!(xplenum::CSR_XPMASK_STATE, 0x7C5);
        assert_eq!(xplenum::CSR_XPTRIT_MODE, 0x7C6);
        assert_eq!(xplenum::CSR_XPSIG_CFG, 0x7C7);
        assert_eq!(xplenum::CSR_XPEXC_CAUSE, 0x7C8);
        assert_eq!(xplenum::CSR_XPEXC_ADDR, 0x7C9);
        assert_eq!(xplenum::CSR_XPPERF_CNT, 0x7CA);
        assert_eq!(xplenum::CSR_XPVERSION, 0x7CB);
    }

    // ========================================================================
    // XPSTATUS Bit Mask Tests
    // ========================================================================

    #[test]
    fn test_status_bits() {
        use crate::arch::xplenum;
        assert_eq!(xplenum::XPSTATUS_MASK_EN, 0x01);
        assert_eq!(xplenum::XPSTATUS_DOM_EN, 0x02);
        assert_eq!(xplenum::XPSTATUS_CAP_EN, 0x04);
        assert_eq!(xplenum::XPSTATUS_SIG_EN, 0x08);
    }

    #[test]
    fn test_status_all_enabled() {
        use crate::arch::xplenum;
        let all = xplenum::XPSTATUS_MASK_EN
            | xplenum::XPSTATUS_DOM_EN
            | xplenum::XPSTATUS_CAP_EN
            | xplenum::XPSTATUS_SIG_EN;
        assert_eq!(all, 0x0F);
    }
}
