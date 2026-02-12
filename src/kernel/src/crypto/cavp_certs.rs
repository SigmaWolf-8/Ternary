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

//! CAVP Certificate Tracking Registry
//!
//! Tracks NIST CAVP algorithm validation certificate numbers for
//! every approved algorithm in the module. Certificate numbers are
//! issued by NIST after successful ACVTS testing and are required
//! for the CMVP submission package.
//!
//! # Status
//! Currently PENDING — certificates will be populated after ACVTS
//! server registration and test vector exchange (Task E1).
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertStatus {
    Pending,
    Submitted,
    Issued { cert_number: String },
    Revoked { reason: String },
}

impl core::fmt::Display for CertStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CertStatus::Pending => write!(f, "Pending"),
            CertStatus::Submitted => write!(f, "Submitted to ACVTS"),
            CertStatus::Issued { cert_number } => write!(f, "Issued: #{}", cert_number),
            CertStatus::Revoked { reason } => write!(f, "Revoked: {}", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CavpCertRecord {
    pub algorithm: String,
    pub fips_standard: String,
    pub status: CertStatus,
    pub acvts_test_session: Option<String>,
    pub validation_date: Option<String>,
}

pub fn get_certificate_registry() -> [CavpCertRecord; 11] {
    [
        CavpCertRecord {
            algorithm: String::from("AES-256-GCM"),
            fips_standard: String::from("FIPS 197 / SP 800-38D"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("SHA-384"),
            fips_standard: String::from("FIPS 180-4"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("SHA-512"),
            fips_standard: String::from("FIPS 180-4"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("SHA3-384"),
            fips_standard: String::from("FIPS 202"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("SHA3-512"),
            fips_standard: String::from("FIPS 202"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("HMAC-SHA-384"),
            fips_standard: String::from("FIPS 198-1"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("ML-KEM-1024 (TL-KEM)"),
            fips_standard: String::from("FIPS 203"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("ML-DSA-87 (TL-DSA)"),
            fips_standard: String::from("FIPS 204"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("LMS"),
            fips_standard: String::from("SP 800-208"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("XMSS"),
            fips_standard: String::from("SP 800-208"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
        CavpCertRecord {
            algorithm: String::from("HMAC-DRBG-SHA384"),
            fips_standard: String::from("SP 800-90A"),
            status: CertStatus::Pending,
            acvts_test_session: None,
            validation_date: None,
        },
    ]
}

pub fn pending_count() -> usize {
    get_certificate_registry().iter()
        .filter(|r| matches!(r.status, CertStatus::Pending))
        .count()
}

pub fn issued_count() -> usize {
    get_certificate_registry().iter()
        .filter(|r| matches!(r.status, CertStatus::Issued { .. }))
        .count()
}

pub fn total_algorithms() -> usize {
    11
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_completeness() {
        let registry = get_certificate_registry();
        assert_eq!(registry.len(), 11);
    }

    #[test]
    fn test_all_pending() {
        assert_eq!(pending_count(), 11);
        assert_eq!(issued_count(), 0);
    }

    #[test]
    fn test_cert_status_display() {
        assert_eq!(alloc::format!("{}", CertStatus::Pending), "Pending");
        let issued = CertStatus::Issued { cert_number: String::from("A12345") };
        assert!(alloc::format!("{}", issued).contains("A12345"));
    }

    #[test]
    fn test_total_algorithms() {
        assert_eq!(total_algorithms(), 11);
    }

    #[test]
    fn test_registry_fips_standards() {
        let registry = get_certificate_registry();
        let standards: Vec<&str> = registry.iter().map(|r| r.fips_standard.as_str()).collect();
        assert!(standards.contains(&"FIPS 197 / SP 800-38D"));
        assert!(standards.contains(&"FIPS 180-4"));
        assert!(standards.contains(&"FIPS 202"));
        assert!(standards.contains(&"FIPS 203"));
        assert!(standards.contains(&"SP 800-208"));
        assert!(standards.contains(&"SP 800-90A"));
    }
}
