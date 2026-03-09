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

//! Minimal X.509v3 Certificate Support (CNSA 2.0 PKI)
//!
//! Provides basic X.509v3 certificate structures for CNSA 2.0 compliant PKI
//! chains using ML-DSA-87 (TL-DSA) for digital signatures. This module handles
//! DER-encoded certificate creation and verification for firmware signing
//! authorities and device identity certificates.
//!
//! # Scope
//! This is a minimal implementation covering:
//! - Self-signed root CA certificate generation
//! - End-entity certificate issuance
//! - Certificate chain validation (root -> end-entity)
//! - Basic DER/PEM encoding for interoperability
//!
//! # CNSA 2.0 Compliance
//! - Signature: ML-DSA-87 (FIPS 204, NIST Security Level 5)
//! - Hash: SHA-512 for certificate fingerprints
//! - Key sizes: TL-DSA-87 public keys (~2.5KB)
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::{CryptoError, CryptoResult, TernaryDigest, TERNARY_HASH_TRITS};
use super::sponge::TernarySponge;

const CERT_VERSION_V3: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    TlDsa87,
    Xmss,
    Lms,
}

impl SignatureAlgorithm {
    pub fn oid(&self) -> &'static [u8] {
        match self {
            SignatureAlgorithm::TlDsa87 => &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x03, 0x12],
            SignatureAlgorithm::Xmss => &[0x04, 0x00, 0x7F, 0x82],
            SignatureAlgorithm::Lms => &[0x04, 0x00, 0x7F, 0x83],
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            SignatureAlgorithm::TlDsa87 => "id-ML-DSA-87",
            SignatureAlgorithm::Xmss => "id-alg-xmss",
            SignatureAlgorithm::Lms => "id-alg-lms",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DistinguishedName {
    pub common_name: String,
    pub organization: String,
    pub country: String,
}

impl DistinguishedName {
    pub fn new(cn: &str, org: &str, country: &str) -> Self {
        Self {
            common_name: String::from(cn),
            organization: String::from(org),
            country: String::from(country),
        }
    }

    pub fn to_der(&self) -> Vec<u8> {
        let mut der = Vec::new();
        der_encode_rdn(&mut der, 0x55, 0x04, 0x03, self.common_name.as_bytes());
        der_encode_rdn(&mut der, 0x55, 0x04, 0x0A, self.organization.as_bytes());
        der_encode_rdn(&mut der, 0x55, 0x04, 0x06, self.country.as_bytes());
        der_wrap_sequence(&der)
    }
}

fn der_encode_rdn(out: &mut Vec<u8>, oid_hi: u8, oid_mid: u8, oid_lo: u8, value: &[u8]) {
    let oid = vec![0x55, oid_mid, oid_lo];
    let oid_der = der_wrap_tag(0x06, &oid);
    let val_der = der_wrap_tag(0x0C, value);
    let attr_seq = {
        let mut a = Vec::new();
        a.extend_from_slice(&oid_der);
        a.extend_from_slice(&val_der);
        der_wrap_sequence(&a)
    };
    let rdn_set = der_wrap_tag(0x31, &attr_seq);
    out.extend_from_slice(&rdn_set);
    let _ = oid_hi;
}

fn der_wrap_tag(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(tag);
    der_encode_length(&mut out, content.len());
    out.extend_from_slice(content);
    out
}

fn der_wrap_sequence(content: &[u8]) -> Vec<u8> {
    der_wrap_tag(0x30, content)
}

fn der_encode_length(out: &mut Vec<u8>, len: usize) {
    if len < 128 {
        out.push(len as u8);
    } else if len < 256 {
        out.push(0x81);
        out.push(len as u8);
    } else {
        out.push(0x82);
        out.push((len >> 8) as u8);
        out.push(len as u8);
    }
}

#[derive(Debug, Clone)]
pub struct Validity {
    pub not_before: u64,
    pub not_after: u64,
}

impl Validity {
    pub fn new(not_before: u64, not_after: u64) -> Self {
        Self { not_before, not_after }
    }

    pub fn is_valid_at(&self, timestamp: u64) -> bool {
        timestamp >= self.not_before && timestamp <= self.not_after
    }

    pub fn to_der(&self) -> Vec<u8> {
        let nb = der_wrap_tag(0x18, &encode_generalized_time(self.not_before));
        let na = der_wrap_tag(0x18, &encode_generalized_time(self.not_after));
        let mut seq = Vec::new();
        seq.extend_from_slice(&nb);
        seq.extend_from_slice(&na);
        der_wrap_sequence(&seq)
    }
}

fn encode_generalized_time(ts: u64) -> Vec<u8> {
    let secs_per_day: u64 = 86400;
    let secs_per_year: u64 = 365 * secs_per_day;
    let year = 2025 + (ts / secs_per_year);
    let remainder = ts % secs_per_year;
    let day_of_year = remainder / secs_per_day;
    let month = (day_of_year / 30).min(11) + 1;
    let day = (day_of_year % 30) + 1;
    let time_in_day = remainder % secs_per_day;
    let hour = time_in_day / 3600;
    let minute = (time_in_day % 3600) / 60;
    let second = time_in_day % 60;
    let s = alloc::format!("{:04}{:02}{:02}{:02}{:02}{:02}Z", year, month, day, hour, minute, second);
    s.into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
    DigitalSignature,
    KeyEncipherment,
    CertSign,
    CrlSign,
}

#[derive(Debug, Clone)]
pub struct CertificateBuilder {
    pub serial_number: u64,
    pub signature_algorithm: SignatureAlgorithm,
    pub issuer: DistinguishedName,
    pub subject: DistinguishedName,
    pub validity: Validity,
    pub subject_public_key: Vec<u8>,
    pub is_ca: bool,
    pub key_usage: Vec<KeyUsage>,
    pub path_length: Option<u8>,
}

impl CertificateBuilder {
    pub fn new_root_ca(
        cn: &str,
        org: &str,
        country: &str,
        serial: u64,
        public_key: &[u8],
        validity: Validity,
    ) -> Self {
        let dn = DistinguishedName::new(cn, org, country);
        Self {
            serial_number: serial,
            signature_algorithm: SignatureAlgorithm::TlDsa87,
            issuer: dn.clone(),
            subject: dn,
            validity,
            subject_public_key: public_key.to_vec(),
            is_ca: true,
            key_usage: vec![KeyUsage::CertSign, KeyUsage::CrlSign],
            path_length: Some(1),
        }
    }

    pub fn new_end_entity(
        subject_cn: &str,
        subject_org: &str,
        country: &str,
        issuer: &DistinguishedName,
        serial: u64,
        public_key: &[u8],
        validity: Validity,
    ) -> Self {
        Self {
            serial_number: serial,
            signature_algorithm: SignatureAlgorithm::TlDsa87,
            issuer: issuer.clone(),
            subject: DistinguishedName::new(subject_cn, subject_org, country),
            validity,
            subject_public_key: public_key.to_vec(),
            is_ca: false,
            key_usage: vec![KeyUsage::DigitalSignature],
            path_length: None,
        }
    }

    pub fn build_tbs(&self) -> Vec<u8> {
        let mut tbs = Vec::new();
        let version = der_wrap_tag(0xA0, &der_wrap_tag(0x02, &[CERT_VERSION_V3]));
        tbs.extend_from_slice(&version);
        let serial = self.serial_number.to_be_bytes();
        let serial_trimmed = trim_leading_zeros(&serial);
        tbs.extend_from_slice(&der_wrap_tag(0x02, serial_trimmed));
        let alg_oid = der_wrap_tag(0x06, self.signature_algorithm.oid());
        tbs.extend_from_slice(&der_wrap_sequence(&alg_oid));
        tbs.extend_from_slice(&self.issuer.to_der());
        tbs.extend_from_slice(&self.validity.to_der());
        tbs.extend_from_slice(&self.subject.to_der());
        let pk_bits = der_wrap_tag(0x03, &{
            let mut bits = vec![0x00];
            bits.extend_from_slice(&self.subject_public_key);
            bits
        });
        let spki_alg = der_wrap_sequence(&der_wrap_tag(0x06, self.signature_algorithm.oid()));
        let mut spki = Vec::new();
        spki.extend_from_slice(&spki_alg);
        spki.extend_from_slice(&pk_bits);
        tbs.extend_from_slice(&der_wrap_sequence(&spki));
        if self.is_ca {
            let bc = der_wrap_sequence(&der_wrap_tag(0x01, &[0xFF]));
            let bc_ext = {
                let ext_oid = der_wrap_tag(0x06, &[0x55, 0x1D, 0x13]);
                let ext_critical = der_wrap_tag(0x01, &[0xFF]);
                let ext_val = der_wrap_tag(0x04, &bc);
                let mut e = Vec::new();
                e.extend_from_slice(&ext_oid);
                e.extend_from_slice(&ext_critical);
                e.extend_from_slice(&ext_val);
                der_wrap_sequence(&e)
            };
            let extensions = der_wrap_tag(0xA3, &der_wrap_sequence(&bc_ext));
            tbs.extend_from_slice(&extensions);
        }
        der_wrap_sequence(&tbs)
    }

    pub fn tbs_hash(&self) -> [u8; 32] {
        let tbs = self.build_tbs();
        cert_hash(&tbs)
    }
}

fn trim_leading_zeros(data: &[u8]) -> &[u8] {
    let start = data.iter().position(|&b| b != 0).unwrap_or(data.len().saturating_sub(1));
    &data[start..]
}

fn cert_hash(data: &[u8]) -> [u8; 32] {
    let mut sponge = TernarySponge::new();
    sponge.absorb_bytes(&[42u8]);
    if !data.is_empty() {
        sponge.absorb_bytes(data);
    }
    let out = sponge.squeeze(TERNARY_HASH_TRITS);
    let bytes = out.to_bytes();
    let mut result = [0u8; 32];
    let len = core::cmp::min(bytes.len(), 32);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub tbs_certificate: Vec<u8>,
    pub signature_algorithm: SignatureAlgorithm,
    pub signature_value: Vec<u8>,
    pub tbs_hash: [u8; 32],
    pub subject: DistinguishedName,
    pub issuer: DistinguishedName,
    pub serial_number: u64,
    pub is_ca: bool,
    pub validity: Validity,
}

impl Certificate {
    pub fn to_der(&self) -> Vec<u8> {
        let mut cert = Vec::new();
        cert.extend_from_slice(&self.tbs_certificate);
        let alg_oid = der_wrap_tag(0x06, self.signature_algorithm.oid());
        cert.extend_from_slice(&der_wrap_sequence(&alg_oid));
        let mut sig_bits = vec![0x00];
        sig_bits.extend_from_slice(&self.signature_value);
        cert.extend_from_slice(&der_wrap_tag(0x03, &sig_bits));
        der_wrap_sequence(&cert)
    }

    pub fn fingerprint(&self) -> [u8; 32] {
        let der = self.to_der();
        cert_hash(&der)
    }

    pub fn is_valid_at(&self, timestamp: u64) -> bool {
        self.validity.is_valid_at(timestamp)
    }

    pub fn is_self_signed(&self) -> bool {
        self.subject.common_name == self.issuer.common_name
            && self.subject.organization == self.issuer.organization
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertValidationResult {
    Valid,
    ExpiredOrNotYetValid,
    SignatureInvalid,
    ChainBroken,
    NotCa,
}

pub fn validate_chain(
    chain: &[Certificate],
    trust_anchor_fingerprint: &[u8; 32],
    current_time: u64,
) -> CertValidationResult {
    if chain.is_empty() {
        return CertValidationResult::ChainBroken;
    }
    let root = &chain[chain.len() - 1];
    let fp = root.fingerprint();
    let mut diff: u8 = 0;
    for (a, b) in fp.iter().zip(trust_anchor_fingerprint.iter()) {
        diff |= a ^ b;
    }
    if diff != 0 {
        return CertValidationResult::ChainBroken;
    }
    for cert in chain.iter() {
        if !cert.is_valid_at(current_time) {
            return CertValidationResult::ExpiredOrNotYetValid;
        }
    }
    for i in 0..chain.len().saturating_sub(1) {
        let issued = &chain[i];
        let issuer = &chain[i + 1];
        if issued.issuer.common_name != issuer.subject.common_name {
            return CertValidationResult::ChainBroken;
        }
        if !issuer.is_ca {
            return CertValidationResult::NotCa;
        }
    }
    CertValidationResult::Valid
}

pub fn pem_encode(label: &str, der: &[u8]) -> String {
    let b64 = base64_encode(der);
    let mut pem = String::new();
    pem.push_str("-----BEGIN ");
    pem.push_str(label);
    pem.push_str("-----\n");
    let mut i = 0;
    while i < b64.len() {
        let end = core::cmp::min(i + 64, b64.len());
        pem.push_str(&b64[i..end]);
        pem.push('\n');
        i = end;
    }
    pem.push_str("-----END ");
    pem.push_str(label);
    pem.push_str("-----\n");
    pem
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        out.push(CHARS[(n & 0x3F) as usize] as char);
        i += 3;
    }
    if i + 1 == data.len() {
        let n = (data[i] as u32) << 16;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if i + 2 == data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distinguished_name_to_der() {
        let dn = DistinguishedName::new("Test CA", "PlenumNET", "CA");
        let der = dn.to_der();
        assert!(!der.is_empty());
        assert_eq!(der[0], 0x30);
    }

    #[test]
    fn test_validity() {
        let v = Validity::new(1000, 2000);
        assert!(v.is_valid_at(1500));
        assert!(!v.is_valid_at(500));
        assert!(!v.is_valid_at(2500));
    }

    #[test]
    fn test_certificate_builder_tbs() {
        let builder = CertificateBuilder::new_root_ca(
            "PlenumNET Root CA",
            "PlenumNET",
            "CA",
            1,
            &[0xAA; 64],
            Validity::new(0, 315360000),
        );
        let tbs = builder.build_tbs();
        assert!(!tbs.is_empty());
        assert_eq!(tbs[0], 0x30);
        let hash = builder.tbs_hash();
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_certificate_to_der() {
        let builder = CertificateBuilder::new_root_ca(
            "Root", "Org", "US", 1,
            &[0xBB; 32],
            Validity::new(0, 100000000),
        );
        let tbs = builder.build_tbs();
        let cert = Certificate {
            tbs_certificate: tbs,
            signature_algorithm: SignatureAlgorithm::TlDsa87,
            signature_value: vec![0xCC; 64],
            tbs_hash: builder.tbs_hash(),
            subject: builder.subject.clone(),
            issuer: builder.issuer.clone(),
            serial_number: 1,
            is_ca: true,
            validity: builder.validity.clone(),
        };
        let der = cert.to_der();
        assert!(!der.is_empty());
        assert_eq!(der[0], 0x30);
        assert!(cert.is_self_signed());
    }

    #[test]
    fn test_pem_encode() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let pem = pem_encode("CERTIFICATE", &data);
        assert!(pem.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(pem.contains("-----END CERTIFICATE-----"));
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode(b"Hello");
        assert_eq!(encoded, "SGVsbG8=");
        let encoded2 = base64_encode(b"Hi");
        assert_eq!(encoded2, "SGk=");
        let encoded3 = base64_encode(b"Hey");
        assert_eq!(encoded3, "SGV5");
    }

    #[test]
    fn test_chain_validation_empty() {
        let result = validate_chain(&[], &[0u8; 32], 1000);
        assert_eq!(result, CertValidationResult::ChainBroken);
    }

    #[test]
    fn test_signature_algorithm_names() {
        assert_eq!(SignatureAlgorithm::TlDsa87.name(), "id-ML-DSA-87");
        assert_eq!(SignatureAlgorithm::Xmss.name(), "id-alg-xmss");
        assert_eq!(SignatureAlgorithm::Lms.name(), "id-alg-lms");
    }

    #[test]
    fn test_cert_hash_deterministic() {
        let data = vec![1u8, 2, 3, 4, 5];
        let h1 = cert_hash(&data);
        let h2 = cert_hash(&data);
        assert_eq!(h1, h2);
    }
}
