//! CNSA 2.0 Protocol Profiles
//!
//! Defines compliant cipher suite configurations for four core protocols:
//! - TLS 1.3 (RFC 8446 + CNSA 2.0)
//! - SSH (RFC 4253 + CNSA 2.0)
//! - IPsec/IKEv2 (RFC 7296 + CNSA 2.0)
//! - S/MIME (RFC 8551 + CNSA 2.0)
//!
//! Each profile specifies the required key exchange, authentication, symmetric
//! encryption, and hash algorithms per CNSA 2.0 requirements. The profiles
//! also define forbidden algorithms that MUST be rejected.
//!
//! # CNSA 2.0 Protocol Requirements
//! - Key Exchange: ML-KEM-1024 (FIPS 203)
//! - Authentication: ML-DSA-87 (FIPS 204) or XMSS/LMS (SP 800-208)
//! - Symmetric: AES-256-GCM
//! - Hash: SHA-384 (TLS KDF), SHA-512 (general integrity)
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Tls13,
    Ssh,
    IpsecIkev2,
    Smime,
}

impl ProtocolType {
    pub fn name(&self) -> &'static str {
        match self {
            ProtocolType::Tls13 => "TLS 1.3",
            ProtocolType::Ssh => "SSH",
            ProtocolType::IpsecIkev2 => "IPsec/IKEv2",
            ProtocolType::Smime => "S/MIME",
        }
    }
    pub fn rfc(&self) -> &'static str {
        match self {
            ProtocolType::Tls13 => "RFC 8446",
            ProtocolType::Ssh => "RFC 4253",
            ProtocolType::IpsecIkev2 => "RFC 7296",
            ProtocolType::Smime => "RFC 8551",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyExchangeAlgorithm {
    MlKem1024,
    MlKem768,
    EcdhP384,
    X25519,
    DhGroup14,
}

impl KeyExchangeAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            KeyExchangeAlgorithm::MlKem1024 => "ML-KEM-1024",
            KeyExchangeAlgorithm::MlKem768 => "ML-KEM-768",
            KeyExchangeAlgorithm::EcdhP384 => "ECDH-P384",
            KeyExchangeAlgorithm::X25519 => "X25519",
            KeyExchangeAlgorithm::DhGroup14 => "DH-Group14",
        }
    }
    pub fn is_cnsa2(&self) -> bool {
        matches!(self, KeyExchangeAlgorithm::MlKem1024 | KeyExchangeAlgorithm::MlKem768)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthAlgorithm {
    MlDsa87,
    MlDsa65,
    Xmss,
    Lms,
    EcdsaP384,
    RsaPss4096,
    Ed25519,
}

impl AuthAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            AuthAlgorithm::MlDsa87 => "ML-DSA-87",
            AuthAlgorithm::MlDsa65 => "ML-DSA-65",
            AuthAlgorithm::Xmss => "XMSS",
            AuthAlgorithm::Lms => "LMS",
            AuthAlgorithm::EcdsaP384 => "ECDSA-P384",
            AuthAlgorithm::RsaPss4096 => "RSA-PSS-4096",
            AuthAlgorithm::Ed25519 => "Ed25519",
        }
    }
    pub fn is_cnsa2(&self) -> bool {
        matches!(self, AuthAlgorithm::MlDsa87 | AuthAlgorithm::MlDsa65 | AuthAlgorithm::Xmss | AuthAlgorithm::Lms)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetricAlgorithm {
    Aes256Gcm,
    Aes128Gcm,
    ChaCha20Poly1305,
    Aes256Cbc,
    TripleDes,
}

impl SymmetricAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            SymmetricAlgorithm::Aes256Gcm => "AES-256-GCM",
            SymmetricAlgorithm::Aes128Gcm => "AES-128-GCM",
            SymmetricAlgorithm::ChaCha20Poly1305 => "ChaCha20-Poly1305",
            SymmetricAlgorithm::Aes256Cbc => "AES-256-CBC",
            SymmetricAlgorithm::TripleDes => "3DES",
        }
    }
    pub fn is_cnsa2(&self) -> bool {
        matches!(self, SymmetricAlgorithm::Aes256Gcm)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha384,
    Sha512,
    Sha256,
    Sha1,
}

impl HashAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha384 => "SHA-384",
            HashAlgorithm::Sha512 => "SHA-512",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha1 => "SHA-1",
        }
    }
    pub fn is_cnsa2(&self) -> bool {
        matches!(self, HashAlgorithm::Sha384 | HashAlgorithm::Sha512)
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolProfile {
    pub protocol: ProtocolType,
    pub required_kex: Vec<KeyExchangeAlgorithm>,
    pub required_auth: Vec<AuthAlgorithm>,
    pub required_symmetric: Vec<SymmetricAlgorithm>,
    pub required_hash: Vec<HashAlgorithm>,
    pub forbidden_kex: Vec<KeyExchangeAlgorithm>,
    pub forbidden_auth: Vec<AuthAlgorithm>,
    pub forbidden_symmetric: Vec<SymmetricAlgorithm>,
    pub forbidden_hash: Vec<HashAlgorithm>,
    pub notes: String,
}

impl ProtocolProfile {
    pub fn is_fully_cnsa2(&self) -> bool {
        self.required_kex.iter().all(|k| k.is_cnsa2())
            && self.required_auth.iter().all(|a| a.is_cnsa2())
            && self.required_symmetric.iter().all(|s| s.is_cnsa2())
            && self.required_hash.iter().all(|h| h.is_cnsa2())
    }

    pub fn validate_kex(&self, kex: KeyExchangeAlgorithm) -> bool {
        if self.forbidden_kex.contains(&kex) {
            return false;
        }
        self.required_kex.contains(&kex)
    }

    pub fn validate_auth(&self, auth: AuthAlgorithm) -> bool {
        if self.forbidden_auth.contains(&auth) {
            return false;
        }
        self.required_auth.contains(&auth)
    }

    pub fn validate_symmetric(&self, sym: SymmetricAlgorithm) -> bool {
        if self.forbidden_symmetric.contains(&sym) {
            return false;
        }
        self.required_symmetric.contains(&sym)
    }

    pub fn validate_hash(&self, hash: HashAlgorithm) -> bool {
        if self.forbidden_hash.contains(&hash) {
            return false;
        }
        self.required_hash.contains(&hash)
    }
}

pub fn tls13_cnsa() -> ProtocolProfile {
    ProtocolProfile {
        protocol: ProtocolType::Tls13,
        required_kex: vec![KeyExchangeAlgorithm::MlKem1024],
        required_auth: vec![AuthAlgorithm::MlDsa87, AuthAlgorithm::Xmss, AuthAlgorithm::Lms],
        required_symmetric: vec![SymmetricAlgorithm::Aes256Gcm],
        required_hash: vec![HashAlgorithm::Sha384],
        forbidden_kex: vec![KeyExchangeAlgorithm::X25519, KeyExchangeAlgorithm::DhGroup14],
        forbidden_auth: vec![AuthAlgorithm::Ed25519],
        forbidden_symmetric: vec![SymmetricAlgorithm::TripleDes, SymmetricAlgorithm::Aes128Gcm],
        forbidden_hash: vec![HashAlgorithm::Sha1],
        notes: String::from(
            "TLS 1.3 CNSA 2.0 profile: ML-KEM-1024 key exchange, \
             ML-DSA-87/XMSS/LMS authentication, AES-256-GCM encryption, \
             HKDF-SHA-384 key derivation. Cipher suite: TLS_AES_256_GCM_SHA384."
        ),
    }
}

pub fn ssh_cnsa() -> ProtocolProfile {
    ProtocolProfile {
        protocol: ProtocolType::Ssh,
        required_kex: vec![KeyExchangeAlgorithm::MlKem1024],
        required_auth: vec![AuthAlgorithm::MlDsa87, AuthAlgorithm::Xmss, AuthAlgorithm::Lms],
        required_symmetric: vec![SymmetricAlgorithm::Aes256Gcm],
        required_hash: vec![HashAlgorithm::Sha512],
        forbidden_kex: vec![KeyExchangeAlgorithm::X25519, KeyExchangeAlgorithm::DhGroup14],
        forbidden_auth: vec![AuthAlgorithm::Ed25519, AuthAlgorithm::RsaPss4096],
        forbidden_symmetric: vec![SymmetricAlgorithm::TripleDes, SymmetricAlgorithm::ChaCha20Poly1305],
        forbidden_hash: vec![HashAlgorithm::Sha1, HashAlgorithm::Sha256],
        notes: String::from(
            "SSH CNSA 2.0 profile: ML-KEM-1024 key exchange, \
             ML-DSA-87/XMSS host key authentication, AES-256-GCM transport, \
             SHA-512 integrity. Ed25519 and RSA host keys forbidden."
        ),
    }
}

pub fn ipsec_cnsa() -> ProtocolProfile {
    ProtocolProfile {
        protocol: ProtocolType::IpsecIkev2,
        required_kex: vec![KeyExchangeAlgorithm::MlKem1024],
        required_auth: vec![AuthAlgorithm::MlDsa87, AuthAlgorithm::Xmss],
        required_symmetric: vec![SymmetricAlgorithm::Aes256Gcm],
        required_hash: vec![HashAlgorithm::Sha384, HashAlgorithm::Sha512],
        forbidden_kex: vec![KeyExchangeAlgorithm::DhGroup14],
        forbidden_auth: vec![AuthAlgorithm::Ed25519],
        forbidden_symmetric: vec![SymmetricAlgorithm::TripleDes, SymmetricAlgorithm::Aes256Cbc],
        forbidden_hash: vec![HashAlgorithm::Sha1],
        notes: String::from(
            "IPsec/IKEv2 CNSA 2.0 profile: ML-KEM-1024 IKE key exchange, \
             ML-DSA-87/XMSS IKE authentication, AES-256-GCM ESP transform, \
             SHA-384/SHA-512 PRF and integrity. CBC mode forbidden."
        ),
    }
}

pub fn smime_cnsa() -> ProtocolProfile {
    ProtocolProfile {
        protocol: ProtocolType::Smime,
        required_kex: vec![KeyExchangeAlgorithm::MlKem1024],
        required_auth: vec![AuthAlgorithm::MlDsa87, AuthAlgorithm::Lms],
        required_symmetric: vec![SymmetricAlgorithm::Aes256Gcm],
        required_hash: vec![HashAlgorithm::Sha384, HashAlgorithm::Sha512],
        forbidden_kex: vec![KeyExchangeAlgorithm::X25519, KeyExchangeAlgorithm::EcdhP384],
        forbidden_auth: vec![AuthAlgorithm::Ed25519, AuthAlgorithm::EcdsaP384],
        forbidden_symmetric: vec![SymmetricAlgorithm::TripleDes],
        forbidden_hash: vec![HashAlgorithm::Sha1],
        notes: String::from(
            "S/MIME CNSA 2.0 profile: ML-KEM-1024 key transport, \
             ML-DSA-87/LMS message signing, AES-256-GCM content encryption, \
             SHA-384/SHA-512 digest. EC-based algorithms forbidden."
        ),
    }
}

pub fn all_profiles() -> Vec<ProtocolProfile> {
    vec![tls13_cnsa(), ssh_cnsa(), ipsec_cnsa(), smime_cnsa()]
}

#[derive(Debug, Clone)]
pub struct ProfileValidation {
    pub protocol: ProtocolType,
    pub kex_valid: bool,
    pub auth_valid: bool,
    pub symmetric_valid: bool,
    pub hash_valid: bool,
    pub overall_valid: bool,
    pub issues: Vec<String>,
}

pub fn validate_negotiation(
    profile: &ProtocolProfile,
    kex: KeyExchangeAlgorithm,
    auth: AuthAlgorithm,
    sym: SymmetricAlgorithm,
    hash: HashAlgorithm,
) -> ProfileValidation {
    let mut issues = Vec::new();
    let kex_valid = profile.validate_kex(kex);
    if !kex_valid {
        let mut issue = String::from("Key exchange ");
        issue.push_str(kex.name());
        issue.push_str(" not permitted");
        issues.push(issue);
    }
    let auth_valid = profile.validate_auth(auth);
    if !auth_valid {
        let mut issue = String::from("Authentication ");
        issue.push_str(auth.name());
        issue.push_str(" not permitted");
        issues.push(issue);
    }
    let symmetric_valid = profile.validate_symmetric(sym);
    if !symmetric_valid {
        let mut issue = String::from("Symmetric cipher ");
        issue.push_str(sym.name());
        issue.push_str(" not permitted");
        issues.push(issue);
    }
    let hash_valid = profile.validate_hash(hash);
    if !hash_valid {
        let mut issue = String::from("Hash ");
        issue.push_str(hash.name());
        issue.push_str(" not permitted");
        issues.push(issue);
    }
    let overall_valid = kex_valid && auth_valid && symmetric_valid && hash_valid;
    ProfileValidation {
        protocol: profile.protocol,
        kex_valid,
        auth_valid,
        symmetric_valid,
        hash_valid,
        overall_valid,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls13_cnsa_profile() {
        let profile = tls13_cnsa();
        assert_eq!(profile.protocol, ProtocolType::Tls13);
        assert!(profile.is_fully_cnsa2());
        assert!(profile.validate_kex(KeyExchangeAlgorithm::MlKem1024));
        assert!(!profile.validate_kex(KeyExchangeAlgorithm::X25519));
        assert!(profile.validate_auth(AuthAlgorithm::MlDsa87));
        assert!(!profile.validate_auth(AuthAlgorithm::Ed25519));
        assert!(profile.validate_symmetric(SymmetricAlgorithm::Aes256Gcm));
        assert!(!profile.validate_symmetric(SymmetricAlgorithm::TripleDes));
    }

    #[test]
    fn test_ssh_cnsa_profile() {
        let profile = ssh_cnsa();
        assert_eq!(profile.protocol, ProtocolType::Ssh);
        assert!(profile.validate_auth(AuthAlgorithm::Xmss));
        assert!(!profile.validate_auth(AuthAlgorithm::Ed25519));
        assert!(!profile.validate_hash(HashAlgorithm::Sha1));
    }

    #[test]
    fn test_ipsec_cnsa_profile() {
        let profile = ipsec_cnsa();
        assert_eq!(profile.protocol, ProtocolType::IpsecIkev2);
        assert!(profile.validate_symmetric(SymmetricAlgorithm::Aes256Gcm));
        assert!(!profile.validate_symmetric(SymmetricAlgorithm::Aes256Cbc));
    }

    #[test]
    fn test_smime_cnsa_profile() {
        let profile = smime_cnsa();
        assert_eq!(profile.protocol, ProtocolType::Smime);
        assert!(profile.validate_auth(AuthAlgorithm::Lms));
        assert!(!profile.validate_auth(AuthAlgorithm::EcdsaP384));
    }

    #[test]
    fn test_all_profiles_count() {
        let profiles = all_profiles();
        assert_eq!(profiles.len(), 4);
    }

    #[test]
    fn test_valid_negotiation() {
        let profile = tls13_cnsa();
        let result = validate_negotiation(
            &profile,
            KeyExchangeAlgorithm::MlKem1024,
            AuthAlgorithm::MlDsa87,
            SymmetricAlgorithm::Aes256Gcm,
            HashAlgorithm::Sha384,
        );
        assert!(result.overall_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_invalid_negotiation() {
        let profile = tls13_cnsa();
        let result = validate_negotiation(
            &profile,
            KeyExchangeAlgorithm::X25519,
            AuthAlgorithm::Ed25519,
            SymmetricAlgorithm::TripleDes,
            HashAlgorithm::Sha1,
        );
        assert!(!result.overall_valid);
        assert_eq!(result.issues.len(), 4);
    }

    #[test]
    fn test_protocol_names() {
        assert_eq!(ProtocolType::Tls13.name(), "TLS 1.3");
        assert_eq!(ProtocolType::Ssh.name(), "SSH");
        assert_eq!(ProtocolType::IpsecIkev2.name(), "IPsec/IKEv2");
        assert_eq!(ProtocolType::Smime.name(), "S/MIME");
    }

    #[test]
    fn test_algorithm_cnsa2_classification() {
        assert!(KeyExchangeAlgorithm::MlKem1024.is_cnsa2());
        assert!(!KeyExchangeAlgorithm::X25519.is_cnsa2());
        assert!(AuthAlgorithm::MlDsa87.is_cnsa2());
        assert!(!AuthAlgorithm::Ed25519.is_cnsa2());
        assert!(SymmetricAlgorithm::Aes256Gcm.is_cnsa2());
        assert!(!SymmetricAlgorithm::ChaCha20Poly1305.is_cnsa2());
        assert!(HashAlgorithm::Sha384.is_cnsa2());
        assert!(!HashAlgorithm::Sha1.is_cnsa2());
    }
}
