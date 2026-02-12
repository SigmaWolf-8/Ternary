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

//! FIPS 140-3 Service Interface
//!
//! Enumerates all cryptographic services provided by the module per
//! ISO/IEC 19790 Section 7.4. Maps each service to:
//! - Approved or non-approved status
//! - Required module state for availability
//! - Required role for access
//!
//! # Approved Mode Indicator (SP 800-140B)
//! Provides a runtime queryable indicator of whether the module is
//! operating in FIPS-approved mode (CNSA 2.0 only) or non-approved
//! mode (hybrid/legacy algorithms permitted).
//!
//! # Roles (ISO/IEC 19790 Section 7.4.3)
//! - CryptoOfficer: All services, module configuration
//! - User: Approved and non-approved crypto services
//! - None: Status query only
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;

use super::module_state::{ModuleState, ModuleStateMachine, ModeIndicator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoService {
    Aes256GcmEncrypt,
    Aes256GcmDecrypt,
    Sha384Hash,
    Sha512Hash,
    Sha3_384Hash,
    Sha3_512Hash,
    HmacSha384,
    HmacSha512,
    TlKem1024Keygen,
    TlKem1024Encapsulate,
    TlKem1024Decapsulate,
    TlKem768Keygen,
    TlKem512Keygen,
    TlDsa87Keygen,
    TlDsa87Sign,
    TlDsa87Verify,
    TlDsa65Keygen,
    TlDsa44Keygen,
    LmsKeygen,
    LmsSign,
    LmsVerify,
    XmssKeygen,
    XmssSign,
    XmssVerify,
    DrbgInstantiate,
    DrbgGenerate,
    DrbgReseed,
    KeyZeroize,
    SelfTestRun,
    StatusShow,
    PhaseEncrypt,
    PhaseDecrypt,
}

impl CryptoService {
    pub fn is_approved(&self) -> bool {
        matches!(self,
            CryptoService::Aes256GcmEncrypt |
            CryptoService::Aes256GcmDecrypt |
            CryptoService::Sha384Hash |
            CryptoService::Sha512Hash |
            CryptoService::Sha3_384Hash |
            CryptoService::Sha3_512Hash |
            CryptoService::HmacSha384 |
            CryptoService::HmacSha512 |
            CryptoService::TlKem1024Keygen |
            CryptoService::TlKem1024Encapsulate |
            CryptoService::TlKem1024Decapsulate |
            CryptoService::TlDsa87Keygen |
            CryptoService::TlDsa87Sign |
            CryptoService::TlDsa87Verify |
            CryptoService::LmsKeygen |
            CryptoService::LmsSign |
            CryptoService::LmsVerify |
            CryptoService::XmssKeygen |
            CryptoService::XmssSign |
            CryptoService::XmssVerify |
            CryptoService::DrbgInstantiate |
            CryptoService::DrbgGenerate |
            CryptoService::DrbgReseed |
            CryptoService::KeyZeroize |
            CryptoService::SelfTestRun |
            CryptoService::StatusShow
        )
    }

    pub fn fips_reference(&self) -> &str {
        match self {
            CryptoService::Aes256GcmEncrypt | CryptoService::Aes256GcmDecrypt => "FIPS 197, SP 800-38D",
            CryptoService::Sha384Hash | CryptoService::Sha512Hash => "FIPS 180-4",
            CryptoService::Sha3_384Hash | CryptoService::Sha3_512Hash => "FIPS 202",
            CryptoService::HmacSha384 | CryptoService::HmacSha512 => "FIPS 198-1",
            CryptoService::TlKem1024Keygen |
            CryptoService::TlKem1024Encapsulate |
            CryptoService::TlKem1024Decapsulate |
            CryptoService::TlKem768Keygen |
            CryptoService::TlKem512Keygen => "FIPS 203",
            CryptoService::TlDsa87Keygen |
            CryptoService::TlDsa87Sign |
            CryptoService::TlDsa87Verify |
            CryptoService::TlDsa65Keygen |
            CryptoService::TlDsa44Keygen => "FIPS 204",
            CryptoService::LmsKeygen | CryptoService::LmsSign | CryptoService::LmsVerify |
            CryptoService::XmssKeygen | CryptoService::XmssSign | CryptoService::XmssVerify => "SP 800-208",
            CryptoService::DrbgInstantiate |
            CryptoService::DrbgGenerate |
            CryptoService::DrbgReseed => "SP 800-90A",
            CryptoService::KeyZeroize => "ISO 19790 §7.9.7",
            CryptoService::SelfTestRun => "ISO 19790 §7.10",
            CryptoService::StatusShow => "SP 800-140B",
            CryptoService::PhaseEncrypt | CryptoService::PhaseDecrypt => "Non-Approved (ternary-native)",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CryptoService::Aes256GcmEncrypt => "AES-256-GCM authenticated encryption",
            CryptoService::Aes256GcmDecrypt => "AES-256-GCM authenticated decryption",
            CryptoService::Sha384Hash => "SHA-384 hash computation",
            CryptoService::Sha512Hash => "SHA-512 hash computation",
            CryptoService::Sha3_384Hash => "SHA3-384 hash computation",
            CryptoService::Sha3_512Hash => "SHA3-512 hash computation",
            CryptoService::HmacSha384 => "HMAC-SHA-384 message authentication",
            CryptoService::HmacSha512 => "HMAC-SHA-512 message authentication",
            CryptoService::TlKem1024Keygen => "TL-KEM-1024 keypair generation (CNSA 2.0)",
            CryptoService::TlKem1024Encapsulate => "TL-KEM-1024 key encapsulation",
            CryptoService::TlKem1024Decapsulate => "TL-KEM-1024 key decapsulation",
            CryptoService::TlKem768Keygen => "TL-KEM-768 keypair generation (non-CNSA)",
            CryptoService::TlKem512Keygen => "TL-KEM-512 keypair generation (non-CNSA)",
            CryptoService::TlDsa87Keygen => "TL-DSA-87 keypair generation (CNSA 2.0)",
            CryptoService::TlDsa87Sign => "TL-DSA-87 signature generation",
            CryptoService::TlDsa87Verify => "TL-DSA-87 signature verification",
            CryptoService::TlDsa65Keygen => "TL-DSA-65 keypair generation (non-CNSA)",
            CryptoService::TlDsa44Keygen => "TL-DSA-44 keypair generation (non-CNSA)",
            CryptoService::LmsKeygen => "LMS keypair generation (SP 800-208)",
            CryptoService::LmsSign => "LMS signature generation",
            CryptoService::LmsVerify => "LMS signature verification",
            CryptoService::XmssKeygen => "XMSS keypair generation (SP 800-208)",
            CryptoService::XmssSign => "XMSS signature generation",
            CryptoService::XmssVerify => "XMSS signature verification",
            CryptoService::DrbgInstantiate => "HMAC-DRBG-SHA384 instantiation",
            CryptoService::DrbgGenerate => "HMAC-DRBG-SHA384 random bit generation",
            CryptoService::DrbgReseed => "HMAC-DRBG-SHA384 reseeding",
            CryptoService::KeyZeroize => "Cryptographic key zeroization",
            CryptoService::SelfTestRun => "Execute power-on or on-demand self-tests",
            CryptoService::StatusShow => "Query module status and mode indicator",
            CryptoService::PhaseEncrypt => "Phase encryption (ternary-native, non-approved)",
            CryptoService::PhaseDecrypt => "Phase decryption (ternary-native, non-approved)",
        }
    }
}

impl core::fmt::Display for CryptoService {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleRole {
    CryptoOfficer,
    User,
    None,
}

impl core::fmt::Display for ModuleRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModuleRole::CryptoOfficer => write!(f, "Crypto Officer"),
            ModuleRole::User => write!(f, "User"),
            ModuleRole::None => write!(f, "Unauthenticated"),
        }
    }
}

pub fn is_service_available(
    service: CryptoService,
    state: ModuleState,
) -> bool {
    match state {
        ModuleState::Operational | ModuleState::NonApprovedMode => true,
        ModuleState::ApprovedMode => service.is_approved(),
        _ => {
            matches!(service, CryptoService::StatusShow | CryptoService::SelfTestRun)
        }
    }
}

pub fn required_role(service: CryptoService) -> ModuleRole {
    match service {
        CryptoService::SelfTestRun | CryptoService::KeyZeroize => ModuleRole::CryptoOfficer,
        CryptoService::StatusShow => ModuleRole::None,
        _ => ModuleRole::User,
    }
}

pub fn list_available_services(state: ModuleState) -> Vec<CryptoService> {
    let all = [
        CryptoService::Aes256GcmEncrypt,
        CryptoService::Aes256GcmDecrypt,
        CryptoService::Sha384Hash,
        CryptoService::Sha512Hash,
        CryptoService::Sha3_384Hash,
        CryptoService::Sha3_512Hash,
        CryptoService::HmacSha384,
        CryptoService::HmacSha512,
        CryptoService::TlKem1024Keygen,
        CryptoService::TlKem1024Encapsulate,
        CryptoService::TlKem1024Decapsulate,
        CryptoService::TlKem768Keygen,
        CryptoService::TlKem512Keygen,
        CryptoService::TlDsa87Keygen,
        CryptoService::TlDsa87Sign,
        CryptoService::TlDsa87Verify,
        CryptoService::TlDsa65Keygen,
        CryptoService::TlDsa44Keygen,
        CryptoService::LmsKeygen,
        CryptoService::LmsSign,
        CryptoService::LmsVerify,
        CryptoService::XmssKeygen,
        CryptoService::XmssSign,
        CryptoService::XmssVerify,
        CryptoService::DrbgInstantiate,
        CryptoService::DrbgGenerate,
        CryptoService::DrbgReseed,
        CryptoService::KeyZeroize,
        CryptoService::SelfTestRun,
        CryptoService::StatusShow,
        CryptoService::PhaseEncrypt,
        CryptoService::PhaseDecrypt,
    ];

    all.iter()
        .filter(|&&s| is_service_available(s, state))
        .copied()
        .collect()
}

pub fn list_approved_services() -> Vec<CryptoService> {
    list_available_services(ModuleState::ApprovedMode)
}

pub fn list_non_approved_services() -> Vec<CryptoService> {
    let all = list_available_services(ModuleState::NonApprovedMode);
    all.into_iter().filter(|s| !s.is_approved()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approved_services_in_approved_mode() {
        let services = list_available_services(ModuleState::ApprovedMode);
        for s in &services {
            assert!(s.is_approved(), "{:?} should be approved", s);
        }
        assert!(services.contains(&CryptoService::Aes256GcmEncrypt));
        assert!(services.contains(&CryptoService::Sha384Hash));
        assert!(services.contains(&CryptoService::TlKem1024Keygen));
        assert!(services.contains(&CryptoService::DrbgGenerate));
    }

    #[test]
    fn test_non_approved_blocked_in_approved_mode() {
        assert!(!is_service_available(CryptoService::PhaseEncrypt, ModuleState::ApprovedMode));
        assert!(!is_service_available(CryptoService::TlKem768Keygen, ModuleState::ApprovedMode));
        assert!(!is_service_available(CryptoService::TlDsa44Keygen, ModuleState::ApprovedMode));
    }

    #[test]
    fn test_all_available_in_non_approved() {
        let services = list_available_services(ModuleState::NonApprovedMode);
        assert!(services.contains(&CryptoService::PhaseEncrypt));
        assert!(services.contains(&CryptoService::TlKem768Keygen));
        assert!(services.contains(&CryptoService::Aes256GcmEncrypt));
    }

    #[test]
    fn test_error_state_limited_services() {
        let services = list_available_services(ModuleState::Error);
        assert!(services.contains(&CryptoService::StatusShow));
        assert!(services.contains(&CryptoService::SelfTestRun));
        assert!(!services.contains(&CryptoService::Aes256GcmEncrypt));
    }

    #[test]
    fn test_required_roles() {
        assert_eq!(required_role(CryptoService::SelfTestRun), ModuleRole::CryptoOfficer);
        assert_eq!(required_role(CryptoService::KeyZeroize), ModuleRole::CryptoOfficer);
        assert_eq!(required_role(CryptoService::StatusShow), ModuleRole::None);
        assert_eq!(required_role(CryptoService::Aes256GcmEncrypt), ModuleRole::User);
    }

    #[test]
    fn test_fips_references() {
        assert_eq!(CryptoService::Aes256GcmEncrypt.fips_reference(), "FIPS 197, SP 800-38D");
        assert_eq!(CryptoService::DrbgGenerate.fips_reference(), "SP 800-90A");
        assert_eq!(CryptoService::LmsSign.fips_reference(), "SP 800-208");
    }

    #[test]
    fn test_service_display() {
        let desc = alloc::format!("{}", CryptoService::TlKem1024Keygen);
        assert!(desc.contains("TL-KEM-1024"));
    }

    #[test]
    fn test_non_approved_only_list() {
        let non_approved = list_non_approved_services();
        for s in &non_approved {
            assert!(!s.is_approved(), "{:?} should not be approved", s);
        }
        assert!(non_approved.contains(&CryptoService::PhaseEncrypt));
    }
}
