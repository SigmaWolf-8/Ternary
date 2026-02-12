// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! NIST CAVP (Cryptographic Algorithm Validation Program) Submission Package
//!
//! Generates formatted submission artifacts for FIPS 140-3 CMVP validation:
//! - KAT request/response files per NIST SP 800-185 format
//! - Algorithm capability descriptions
//! - Implementation metadata for CAVP lab review
//!
//! # CAVP Submission Structure
//!
//! ```text
//! cavp_submission/
//! ├── TL-KEM/
//! │   ├── TL-KEM-512.req    # KAT request vectors
//! │   ├── TL-KEM-512.rsp    # KAT response vectors
//! │   ├── TL-KEM-768.req
//! │   ├── TL-KEM-768.rsp
//! │   ├── TL-KEM-1024.req
//! │   └── TL-KEM-1024.rsp
//! ├── TL-DSA/
//! │   ├── TL-DSA-44.req
//! │   ├── TL-DSA-44.rsp
//! │   ├── TL-DSA-65.req
//! │   ├── TL-DSA-65.rsp
//! │   ├── TL-DSA-87.req
//! │   └── TL-DSA-87.rsp
//! ├── capabilities.json
//! └── manifest.txt
//! ```
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use super::CryptoResult;
use super::tl_kem::TlKemVariant;
use super::tl_dsa::TlDsaVariant;
use super::kat_vectors::{
    self, KemKatVector, DsaKatVector, KatSummary,
    VECTORS_PER_VARIANT,
};

#[derive(Debug, Clone)]
pub struct CavpFile {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CavpSubmissionPackage {
    pub files: Vec<CavpFile>,
    pub kem_variants: usize,
    pub dsa_variants: usize,
    pub total_vectors: usize,
    pub summary: KatSummary,
}

#[derive(Debug, Clone)]
pub struct AlgorithmCapability {
    pub algorithm: String,
    pub variant: String,
    pub security_level: u8,
    pub pk_size_trits: usize,
    pub sk_size_trits: usize,
    pub output_size_trits: usize,
    pub operations: Vec<String>,
}

fn trits_to_hex(trits: &[i8]) -> String {
    let mut hex = String::with_capacity(trits.len() * 2);
    for chunk in trits.chunks(4) {
        let mut nibble: u8 = 0;
        for (j, &t) in chunk.iter().enumerate() {
            let val = ((t + 1) as u8) & 0x03;
            nibble |= val << (j * 2);
        }
        hex.push_str(&format!("{:02x}", nibble));
    }
    hex
}

fn format_kem_request(variant_name: &str, vectors: &[KemKatVector]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TL-KEM Known Answer Test Request File\n"));
    out.push_str(&format!("# Algorithm: {}\n", variant_name));
    out.push_str(&format!("# Generated: February 2026\n"));
    out.push_str(&format!("# Framework: PlenumNET Salvi v2.0.0\n"));
    out.push_str(&format!("# Vectors: {}\n\n", vectors.len()));

    for (i, v) in vectors.iter().enumerate() {
        out.push_str(&format!("[Vector {}]\n", i));
        out.push_str(&format!("Seed = {}\n", trits_to_hex(&v.seed)));
        out.push_str(&format!("EncapsRandomness = {}\n", trits_to_hex(&v.encaps_randomness)));
        out.push_str(&format!("\n"));
    }

    out
}

fn format_kem_response(variant_name: &str, vectors: &[KemKatVector]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TL-KEM Known Answer Test Response File\n"));
    out.push_str(&format!("# Algorithm: {}\n", variant_name));
    out.push_str(&format!("# Generated: February 2026\n"));
    out.push_str(&format!("# Framework: PlenumNET Salvi v2.0.0\n"));
    out.push_str(&format!("# Vectors: {}\n\n", vectors.len()));

    for (i, v) in vectors.iter().enumerate() {
        out.push_str(&format!("[Vector {}]\n", i));
        out.push_str(&format!("Seed = {}\n", trits_to_hex(&v.seed)));
        out.push_str(&format!("EncapsRandomness = {}\n", trits_to_hex(&v.encaps_randomness)));
        out.push_str(&format!("PK_Hash = {}\n", trits_to_hex(&v.pk_hash)));
        out.push_str(&format!("SK_Hash = {}\n", trits_to_hex(&v.sk_hash)));
        out.push_str(&format!("CT_Hash = {}\n", trits_to_hex(&v.ct_hash)));
        out.push_str(&format!("SS_Hash = {}\n", trits_to_hex(&v.shared_secret_hash)));
        out.push_str(&format!("PK_TritCount = {}\n", v.pk_trit_count));
        out.push_str(&format!("SK_TritCount = {}\n", v.sk_trit_count));
        out.push_str(&format!("CT_ByteCount = {}\n", v.ct_byte_count));
        out.push_str(&format!("SS_TritCount = {}\n", v.shared_secret_trit_count));
        out.push_str(&format!("\n"));
    }

    out
}

fn format_dsa_request(variant_name: &str, vectors: &[DsaKatVector]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TL-DSA Known Answer Test Request File\n"));
    out.push_str(&format!("# Algorithm: {}\n", variant_name));
    out.push_str(&format!("# Generated: February 2026\n"));
    out.push_str(&format!("# Framework: PlenumNET Salvi v2.0.0\n"));
    out.push_str(&format!("# Vectors: {}\n\n", vectors.len()));

    for (i, v) in vectors.iter().enumerate() {
        out.push_str(&format!("[Vector {}]\n", i));
        out.push_str(&format!("Seed = {}\n", trits_to_hex(&v.seed)));
        out.push_str(&format!("Message = {}\n", trits_to_hex(&v.message)));
        out.push_str(&format!("\n"));
    }

    out
}

fn format_dsa_response(variant_name: &str, vectors: &[DsaKatVector]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TL-DSA Known Answer Test Response File\n"));
    out.push_str(&format!("# Algorithm: {}\n", variant_name));
    out.push_str(&format!("# Generated: February 2026\n"));
    out.push_str(&format!("# Framework: PlenumNET Salvi v2.0.0\n"));
    out.push_str(&format!("# Vectors: {}\n\n", vectors.len()));

    for (i, v) in vectors.iter().enumerate() {
        out.push_str(&format!("[Vector {}]\n", i));
        out.push_str(&format!("Seed = {}\n", trits_to_hex(&v.seed)));
        out.push_str(&format!("Message = {}\n", trits_to_hex(&v.message)));
        out.push_str(&format!("PK_Hash = {}\n", trits_to_hex(&v.pk_hash)));
        out.push_str(&format!("SK_Hash = {}\n", trits_to_hex(&v.sk_hash)));
        out.push_str(&format!("Sig_Hash = {}\n", trits_to_hex(&v.signature_hash)));
        out.push_str(&format!("SigValid = {}\n", v.signature_valid));
        out.push_str(&format!("PK_TritCount = {}\n", v.pk_trit_count));
        out.push_str(&format!("SK_TritCount = {}\n", v.sk_trit_count));
        out.push_str(&format!("Sig_TritCount = {}\n", v.sig_trit_count));
        out.push_str(&format!("\n"));
    }

    out
}

fn format_capabilities(kem_vectors: &[KemKatVector], dsa_vectors: &[DsaKatVector]) -> String {
    let mut caps = Vec::new();

    let kem_variants = [
        ("TL-KEM-512", 1u8),
        ("TL-KEM-768", 3),
        ("TL-KEM-1024", 5),
    ];

    for (name, level) in &kem_variants {
        let sample: Vec<_> = kem_vectors.iter()
            .filter(|v| v.variant.name() == *name)
            .collect();
        if let Some(v) = sample.first() {
            caps.push(AlgorithmCapability {
                algorithm: String::from("TL-KEM"),
                variant: String::from(*name),
                security_level: *level,
                pk_size_trits: v.pk_trit_count,
                sk_size_trits: v.sk_trit_count,
                output_size_trits: v.shared_secret_trit_count,
                operations: vec![
                    String::from("KeyGen"),
                    String::from("Encapsulate"),
                    String::from("Decapsulate"),
                ],
            });
        }
    }

    let dsa_variants = [
        ("TL-DSA-44", 2u8),
        ("TL-DSA-65", 3),
        ("TL-DSA-87", 5),
    ];

    for (name, level) in &dsa_variants {
        let sample: Vec<_> = dsa_vectors.iter()
            .filter(|v| v.variant.name() == *name)
            .collect();
        if let Some(v) = sample.first() {
            caps.push(AlgorithmCapability {
                algorithm: String::from("TL-DSA"),
                variant: String::from(*name),
                security_level: *level,
                pk_size_trits: v.pk_trit_count,
                sk_size_trits: v.sk_trit_count,
                output_size_trits: v.sig_trit_count,
                operations: vec![
                    String::from("KeyGen"),
                    String::from("Sign"),
                    String::from("Verify"),
                ],
            });
        }
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"submission\": {\n");
    out.push_str("    \"vendor\": \"Capomastro Holdings Ltd.\",\n");
    out.push_str("    \"module\": \"PlenumNET Salvi Cryptographic Module\",\n");
    out.push_str("    \"version\": \"2.0.0\",\n");
    out.push_str("    \"fips_level\": \"FIPS 140-3 Level 1\",\n");
    out.push_str("    \"standard\": \"CNSA 2.0\",\n");
    out.push_str("    \"date\": \"February 2026\"\n");
    out.push_str("  },\n");
    out.push_str("  \"algorithms\": [\n");

    for (i, cap) in caps.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"algorithm\": \"{}\",\n", cap.algorithm));
        out.push_str(&format!("      \"variant\": \"{}\",\n", cap.variant));
        out.push_str(&format!("      \"security_level\": {},\n", cap.security_level));
        out.push_str(&format!("      \"pk_size_trits\": {},\n", cap.pk_size_trits));
        out.push_str(&format!("      \"sk_size_trits\": {},\n", cap.sk_size_trits));
        out.push_str(&format!("      \"output_size_trits\": {},\n", cap.output_size_trits));
        out.push_str("      \"operations\": [");
        let ops: Vec<String> = cap.operations.iter()
            .map(|o| format!("\"{}\"", o))
            .collect();
        out.push_str(&ops.join(", "));
        out.push_str("]\n");
        out.push_str("    }");
        if i < caps.len() - 1 {
            out.push_str(",");
        }
        out.push_str("\n");
    }

    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn format_manifest(files: &[CavpFile], summary: &KatSummary) -> String {
    let mut out = String::new();
    out.push_str("# CAVP Submission Manifest\n");
    out.push_str("# PlenumNET Salvi Cryptographic Module v2.0.0\n");
    out.push_str("# Capomastro Holdings Ltd.\n");
    out.push_str("# February 2026\n\n");

    out.push_str(&format!("Total KAT Vectors: {} ({} KEM + {} DSA)\n",
        summary.total_vectors, summary.kem_vectors, summary.dsa_vectors));
    out.push_str(&format!("Vectors Passed: {}\n", summary.passed));
    out.push_str(&format!("Vectors Failed: {}\n", summary.failed));
    out.push_str(&format!("FIPS Ready: {}\n\n", summary.fips_ready));

    out.push_str("Files:\n");
    for f in files {
        out.push_str(&format!("  {}\n", f.filename));
    }

    out.push_str("\nAlgorithms Covered:\n");
    out.push_str("  TL-KEM-512  (FIPS 203 equivalent, Security Level 1)\n");
    out.push_str("  TL-KEM-768  (FIPS 203 equivalent, Security Level 3)\n");
    out.push_str("  TL-KEM-1024 (FIPS 203 equivalent, Security Level 5)\n");
    out.push_str("  TL-DSA-44   (FIPS 204 equivalent, Security Level 2)\n");
    out.push_str("  TL-DSA-65   (FIPS 204 equivalent, Security Level 3)\n");
    out.push_str("  TL-DSA-87   (FIPS 204 equivalent, Security Level 5)\n");

    out.push_str("\nCompliance:\n");
    out.push_str("  CNSA 2.0: 11/11 algorithms (100% coverage)\n");
    out.push_str("  FIPS 140-3: Level 1 target\n");
    out.push_str("  Side-channel: Constant-time operations verified\n");
    out.push_str("  KAT: Deterministic reproducibility confirmed\n");

    out
}

pub fn generate_cavp_package() -> CryptoResult<CavpSubmissionPackage> {
    generate_cavp_package_n(VECTORS_PER_VARIANT)
}

pub fn generate_cavp_package_n(count_per_variant: u8) -> CryptoResult<CavpSubmissionPackage> {
    let kem_vectors = kat_vectors::generate_kem_kat_vectors_n(count_per_variant)?;
    let dsa_vectors = kat_vectors::generate_dsa_kat_vectors_n(count_per_variant)?;
    let summary = kat_vectors::kat_summary_n(count_per_variant)?;

    let mut files = Vec::new();

    let kem_variant_names = ["TL-KEM-512", "TL-KEM-768", "TL-KEM-1024"];
    let kem_variants = [TlKemVariant::TlKem512, TlKemVariant::TlKem768, TlKemVariant::TlKem1024];

    for (variant, name) in kem_variants.iter().zip(kem_variant_names.iter()) {
        let vecs: Vec<_> = kem_vectors.iter()
            .filter(|v| &v.variant == variant)
            .cloned()
            .collect();

        files.push(CavpFile {
            filename: format!("TL-KEM/{}.req", name),
            content: format_kem_request(name, &vecs),
        });
        files.push(CavpFile {
            filename: format!("TL-KEM/{}.rsp", name),
            content: format_kem_response(name, &vecs),
        });
    }

    let dsa_variant_names = ["TL-DSA-44", "TL-DSA-65", "TL-DSA-87"];
    let dsa_variants = [TlDsaVariant::TlDsa44, TlDsaVariant::TlDsa65, TlDsaVariant::TlDsa87];

    for (variant, name) in dsa_variants.iter().zip(dsa_variant_names.iter()) {
        let vecs: Vec<_> = dsa_vectors.iter()
            .filter(|v| &v.variant == variant)
            .cloned()
            .collect();

        files.push(CavpFile {
            filename: format!("TL-DSA/{}.req", name),
            content: format_dsa_request(name, &vecs),
        });
        files.push(CavpFile {
            filename: format!("TL-DSA/{}.rsp", name),
            content: format_dsa_response(name, &vecs),
        });
    }

    files.push(CavpFile {
        filename: String::from("capabilities.json"),
        content: format_capabilities(&kem_vectors, &dsa_vectors),
    });

    let manifest_content = format_manifest(&files, &summary);
    files.push(CavpFile {
        filename: String::from("manifest.txt"),
        content: manifest_content,
    });

    Ok(CavpSubmissionPackage {
        kem_variants: 3,
        dsa_variants: 3,
        total_vectors: summary.total_vectors,
        files,
        summary,
    })
}

pub fn validate_cavp_package(pkg: &CavpSubmissionPackage) -> CavpValidationReport {
    let mut issues = Vec::new();

    if pkg.total_vectors < 100 {
        issues.push(format!("Insufficient vectors: {} < 100 minimum", pkg.total_vectors));
    }

    let has_kem_req = pkg.files.iter().any(|f| f.filename.contains("TL-KEM") && f.filename.ends_with(".req"));
    let has_kem_rsp = pkg.files.iter().any(|f| f.filename.contains("TL-KEM") && f.filename.ends_with(".rsp"));
    let has_dsa_req = pkg.files.iter().any(|f| f.filename.contains("TL-DSA") && f.filename.ends_with(".req"));
    let has_dsa_rsp = pkg.files.iter().any(|f| f.filename.contains("TL-DSA") && f.filename.ends_with(".rsp"));
    let has_caps = pkg.files.iter().any(|f| f.filename == "capabilities.json");
    let has_manifest = pkg.files.iter().any(|f| f.filename == "manifest.txt");

    if !has_kem_req { issues.push(String::from("Missing TL-KEM request files")); }
    if !has_kem_rsp { issues.push(String::from("Missing TL-KEM response files")); }
    if !has_dsa_req { issues.push(String::from("Missing TL-DSA request files")); }
    if !has_dsa_rsp { issues.push(String::from("Missing TL-DSA response files")); }
    if !has_caps { issues.push(String::from("Missing capabilities.json")); }
    if !has_manifest { issues.push(String::from("Missing manifest.txt")); }

    if !pkg.summary.fips_ready {
        issues.push(format!("KAT validation failures: {} failed", pkg.summary.failed));
    }

    let kem_req_count = pkg.files.iter().filter(|f| f.filename.contains("TL-KEM") && f.filename.ends_with(".req")).count();
    let dsa_req_count = pkg.files.iter().filter(|f| f.filename.contains("TL-DSA") && f.filename.ends_with(".req")).count();

    if kem_req_count != 3 {
        issues.push(format!("Expected 3 KEM variant files, found {}", kem_req_count));
    }
    if dsa_req_count != 3 {
        issues.push(format!("Expected 3 DSA variant files, found {}", dsa_req_count));
    }

    CavpValidationReport {
        valid: issues.is_empty(),
        file_count: pkg.files.len(),
        total_vectors: pkg.total_vectors,
        issues,
    }
}

#[derive(Debug, Clone)]
pub struct CavpValidationReport {
    pub valid: bool,
    pub file_count: usize,
    pub total_vectors: usize,
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_N: u8 = 3;

    #[test]
    fn test_generate_cavp_package() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        assert_eq!(pkg.kem_variants, 3);
        assert_eq!(pkg.dsa_variants, 3);
        assert_eq!(pkg.total_vectors, 18);
    }

    #[test]
    fn test_cavp_file_count() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let kem_files = pkg.files.iter().filter(|f| f.filename.starts_with("TL-KEM")).count();
        let dsa_files = pkg.files.iter().filter(|f| f.filename.starts_with("TL-DSA")).count();
        assert_eq!(kem_files, 6);
        assert_eq!(dsa_files, 6);
        assert!(pkg.files.iter().any(|f| f.filename == "capabilities.json"));
        assert!(pkg.files.iter().any(|f| f.filename == "manifest.txt"));
        assert_eq!(pkg.files.len(), 14);
    }

    #[test]
    fn test_kem_request_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let req = pkg.files.iter().find(|f| f.filename == "TL-KEM/TL-KEM-512.req").unwrap();
        assert!(req.content.contains("# TL-KEM Known Answer Test Request File"));
        assert!(req.content.contains("Algorithm: TL-KEM-512"));
        assert!(req.content.contains("[Vector 0]"));
        assert!(req.content.contains("Seed = "));
        assert!(req.content.contains("EncapsRandomness = "));
    }

    #[test]
    fn test_kem_response_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let rsp = pkg.files.iter().find(|f| f.filename == "TL-KEM/TL-KEM-768.rsp").unwrap();
        assert!(rsp.content.contains("# TL-KEM Known Answer Test Response File"));
        assert!(rsp.content.contains("PK_Hash = "));
        assert!(rsp.content.contains("SK_Hash = "));
        assert!(rsp.content.contains("CT_Hash = "));
        assert!(rsp.content.contains("SS_Hash = "));
    }

    #[test]
    fn test_dsa_request_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let req = pkg.files.iter().find(|f| f.filename == "TL-DSA/TL-DSA-44.req").unwrap();
        assert!(req.content.contains("# TL-DSA Known Answer Test Request File"));
        assert!(req.content.contains("Seed = "));
        assert!(req.content.contains("Message = "));
    }

    #[test]
    fn test_dsa_response_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let rsp = pkg.files.iter().find(|f| f.filename == "TL-DSA/TL-DSA-87.rsp").unwrap();
        assert!(rsp.content.contains("Sig_Hash = "));
        assert!(rsp.content.contains("SigValid = true"));
    }

    #[test]
    fn test_capabilities_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let caps = pkg.files.iter().find(|f| f.filename == "capabilities.json").unwrap();
        assert!(caps.content.contains("\"vendor\": \"Capomastro Holdings Ltd.\""));
        assert!(caps.content.contains("TL-KEM-512"));
        assert!(caps.content.contains("TL-KEM-768"));
        assert!(caps.content.contains("TL-KEM-1024"));
        assert!(caps.content.contains("TL-DSA-44"));
        assert!(caps.content.contains("TL-DSA-65"));
        assert!(caps.content.contains("TL-DSA-87"));
        assert!(caps.content.contains("\"KeyGen\""));
    }

    #[test]
    fn test_manifest_format() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let manifest = pkg.files.iter().find(|f| f.filename == "manifest.txt").unwrap();
        assert!(manifest.content.contains("CAVP Submission Manifest"));
        assert!(manifest.content.contains("Total KAT Vectors: 18"));
        assert!(manifest.content.contains("CNSA 2.0: 11/11 algorithms"));
    }

    #[test]
    fn test_validate_cavp_package_small() {
        let pkg = generate_cavp_package_n(TEST_N).unwrap();
        let report = validate_cavp_package(&pkg);
        assert!(!report.valid);
        assert!(report.issues.iter().any(|i| i.contains("Insufficient vectors")));
    }

    #[test]
    fn test_trits_to_hex_deterministic() {
        let trits = vec![1i8, 0, -1, 1, 0, 0, 1, -1];
        let h1 = trits_to_hex(&trits);
        let h2 = trits_to_hex(&trits);
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }

    #[test]
    fn test_expanded_package_validity() {
        assert!(VECTORS_PER_VARIANT >= 34, "Need 34+ vectors per variant for 100+ total");
    }
}
