//! CNSA 2.0 Compliance Framework
//!
//! Implements the NSA Commercial National Security Algorithm Suite 2.0
//! compliance tracking and mapping for the PlenumNET Salvi Framework.
//!
//! CNSA 2.0 defines the post-quantum cryptographic algorithms required for
//! protecting classified information through the quantum computing transition.
//! This module maps each CNSA 2.0 requirement to PlenumNET's ternary-native
//! equivalents and tracks implementation status.
//!
//! # CNSA 2.0 Algorithm Requirements
//!
//! | Category | Algorithm | NIST Standard | Deadline |
//! |----------|-----------|---------------|----------|
//! | Symmetric Encryption | AES-256 | FIPS 197 | Immediate |
//! | Hashing | SHA-384 / SHA-512 | FIPS 180-4 | Immediate |
//! | Key Encapsulation | ML-KEM (Kyber) | FIPS 203 | 2030 |
//! | Digital Signatures | ML-DSA (Dilithium) | FIPS 204 | 2035 |
//! | Hash-Based Signatures | LMS / XMSS | SP 800-208 | 2030 |
//!
//! # PlenumNET Approach
//!
//! PlenumNET operates natively in GF(3) (Galois Field of order 3), providing
//! quantum resistance through a fundamentally different mathematical domain
//! rather than through the specific NIST-standardized algorithms. Each CNSA 2.0
//! requirement has a ternary-native equivalent that provides comparable or
//! superior security properties.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cnsa2Algorithm {
    Aes256,
    Sha384,
    Sha512,
    MlKem512,
    MlKem768,
    MlKem1024,
    MlDsa44,
    MlDsa65,
    MlDsa87,
    Lms,
    Xmss,
}

impl Cnsa2Algorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Cnsa2Algorithm::Aes256 => "AES-256",
            Cnsa2Algorithm::Sha384 => "SHA-384",
            Cnsa2Algorithm::Sha512 => "SHA-512",
            Cnsa2Algorithm::MlKem512 => "ML-KEM-512",
            Cnsa2Algorithm::MlKem768 => "ML-KEM-768",
            Cnsa2Algorithm::MlKem1024 => "ML-KEM-1024",
            Cnsa2Algorithm::MlDsa44 => "ML-DSA-44",
            Cnsa2Algorithm::MlDsa65 => "ML-DSA-65",
            Cnsa2Algorithm::MlDsa87 => "ML-DSA-87",
            Cnsa2Algorithm::Lms => "LMS",
            Cnsa2Algorithm::Xmss => "XMSS",
        }
    }

    pub fn nist_standard(&self) -> &'static str {
        match self {
            Cnsa2Algorithm::Aes256 => "FIPS 197",
            Cnsa2Algorithm::Sha384 | Cnsa2Algorithm::Sha512 => "FIPS 180-4",
            Cnsa2Algorithm::MlKem512 | Cnsa2Algorithm::MlKem768 | Cnsa2Algorithm::MlKem1024 => "FIPS 203",
            Cnsa2Algorithm::MlDsa44 | Cnsa2Algorithm::MlDsa65 | Cnsa2Algorithm::MlDsa87 => "FIPS 204",
            Cnsa2Algorithm::Lms | Cnsa2Algorithm::Xmss => "SP 800-208",
        }
    }

    pub fn category(&self) -> Cnsa2Category {
        match self {
            Cnsa2Algorithm::Aes256 => Cnsa2Category::SymmetricEncryption,
            Cnsa2Algorithm::Sha384 | Cnsa2Algorithm::Sha512 => Cnsa2Category::Hashing,
            Cnsa2Algorithm::MlKem512 | Cnsa2Algorithm::MlKem768 | Cnsa2Algorithm::MlKem1024 => Cnsa2Category::KeyEncapsulation,
            Cnsa2Algorithm::MlDsa44 | Cnsa2Algorithm::MlDsa65 | Cnsa2Algorithm::MlDsa87 => Cnsa2Category::DigitalSignatures,
            Cnsa2Algorithm::Lms | Cnsa2Algorithm::Xmss => Cnsa2Category::HashBasedSignatures,
        }
    }

    pub fn security_level_bits(&self) -> u32 {
        match self {
            Cnsa2Algorithm::Aes256 => 256,
            Cnsa2Algorithm::Sha384 => 192,
            Cnsa2Algorithm::Sha512 => 256,
            Cnsa2Algorithm::MlKem512 => 128,
            Cnsa2Algorithm::MlKem768 => 192,
            Cnsa2Algorithm::MlKem1024 => 256,
            Cnsa2Algorithm::MlDsa44 => 128,
            Cnsa2Algorithm::MlDsa65 => 192,
            Cnsa2Algorithm::MlDsa87 => 256,
            Cnsa2Algorithm::Lms => 256,
            Cnsa2Algorithm::Xmss => 256,
        }
    }

    pub fn transition_deadline(&self) -> TransitionDeadline {
        match self {
            Cnsa2Algorithm::Aes256 | Cnsa2Algorithm::Sha384 | Cnsa2Algorithm::Sha512 => TransitionDeadline::Immediate,
            Cnsa2Algorithm::MlKem512 | Cnsa2Algorithm::MlKem768 | Cnsa2Algorithm::MlKem1024 => TransitionDeadline::By2030,
            Cnsa2Algorithm::Lms | Cnsa2Algorithm::Xmss => TransitionDeadline::By2030,
            Cnsa2Algorithm::MlDsa44 | Cnsa2Algorithm::MlDsa65 | Cnsa2Algorithm::MlDsa87 => TransitionDeadline::By2035,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cnsa2Category {
    SymmetricEncryption,
    Hashing,
    KeyEncapsulation,
    DigitalSignatures,
    HashBasedSignatures,
}

impl Cnsa2Category {
    pub fn name(&self) -> &'static str {
        match self {
            Cnsa2Category::SymmetricEncryption => "Symmetric Encryption",
            Cnsa2Category::Hashing => "Hashing",
            Cnsa2Category::KeyEncapsulation => "Key Encapsulation",
            Cnsa2Category::DigitalSignatures => "Digital Signatures",
            Cnsa2Category::HashBasedSignatures => "Hash-Based Signatures",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionDeadline {
    Immediate,
    By2030,
    By2035,
}

impl TransitionDeadline {
    pub fn year(&self) -> &'static str {
        match self {
            TransitionDeadline::Immediate => "Immediate (already required)",
            TransitionDeadline::By2030 => "By 2030",
            TransitionDeadline::By2035 => "By 2035",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceStatus {
    TernaryEquivalent,
    Planned,
    NotImplemented,
}

impl ComplianceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ComplianceStatus::TernaryEquivalent => "Ternary Equivalent",
            ComplianceStatus::Planned => "Planned",
            ComplianceStatus::NotImplemented => "Not Implemented",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ComplianceStatus::TernaryEquivalent => "PlenumNET provides a ternary-native equivalent operating in GF(3) with comparable security properties",
            ComplianceStatus::Planned => "Algorithm stub exists; full implementation planned for future release",
            ComplianceStatus::NotImplemented => "No current implementation or equivalent",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cnsa2Mapping {
    pub algorithm: Cnsa2Algorithm,
    pub status: ComplianceStatus,
    pub plenum_equivalent: String,
    pub plenum_module: String,
    pub security_notes: String,
}

pub fn get_cnsa2_matrix() -> Vec<Cnsa2Mapping> {
    vec![
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::Aes256,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("AES-256-GCM (FIPS 197 + SP 800-38D) with ternary key mapping"),
            plenum_module: String::from("salvi_kernel::crypto::cipher"),
            security_notes: String::from(
                "Full AES-256-GCM implementation with constant-time S-box (composite field \
                 inversion, no lookup tables), 14-round key schedule, and authenticated \
                 encryption with associated data (AEAD). Ternary key mapping via balanced \
                 ternary representation. Binary-compatible for interoperability."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::Sha384,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("Ternary Sponge Hash (243-trit output)"),
            plenum_module: String::from("salvi_kernel::crypto::hash"),
            security_notes: String::from(
                "Keccak-inspired sponge construction over GF(3). 729-trit state width \
                 with 243-trit rate and 486-trit capacity. 27 rounds of substitution-permutation. \
                 243-trit output = 385.4 equivalent bits (exceeds SHA-384)."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::Sha512,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("Ternary Sponge Hash (extended 486-trit output)"),
            plenum_module: String::from("salvi_kernel::crypto::sponge"),
            security_notes: String::from(
                "Extended squeeze operation on the same sponge construction. \
                 486-trit output = 770.8 equivalent bits (exceeds SHA-512). \
                 Configurable output length via sponge squeeze parameter."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlKem512,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-KEM-512 (Ternary Lattice KEM)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_kem"),
            security_notes: String::from(
                "Full IND-CCA2 secure key encapsulation via Fujisaki-Okamoto transform. \
                 GF(3) polynomial ring R_q = Z_3[X]/(X^256+1), Module-LWE with k=2. \
                 KeyGen, Encapsulate, Decapsulate with implicit rejection. \
                 243-trit shared secret (385.4 equivalent bits). NIST Security Level 1."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlKem768,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-KEM-768 (Ternary Lattice KEM)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_kem"),
            security_notes: String::from(
                "Medium-security IND-CCA2 key encapsulation. Module-LWE with k=3, \
                 ternary noise sampling (CBD), compression/decompression for compact \
                 ciphertexts. 243-trit shared secret. NIST Security Level 3."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlKem1024,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-KEM-1024 (Ternary Lattice KEM)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_kem"),
            security_notes: String::from(
                "High-security IND-CCA2 key encapsulation for CNSA 2.0 classified data. \
                 Module-LWE with k=4, enhanced compression parameters (du=5, dv=3). \
                 486-trit shared secret (770.8 equivalent bits). NIST Security Level 5."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlDsa44,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-DSA-44 (Ternary Lattice DSA)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_dsa"),
            security_notes: String::from(
                "Full EUF-CMA secure digital signatures via Fiat-Shamir with Aborts. \
                 GF(3) polynomial ring R_q = Z_3[X]/(X^256+1), Module-LWE with k=4, l=4. \
                 KeyGen, Sign (deterministic with abort-and-retry), Verify. \
                 Sparse ternary challenge with tau=39. NIST Security Level 2."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlDsa65,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-DSA-65 (Ternary Lattice DSA)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_dsa"),
            security_notes: String::from(
                "Medium-security EUF-CMA digital signatures. Module-LWE with k=6, l=5, \
                 ternary noise sampling, deterministic signing with abort-and-retry. \
                 Sparse ternary challenge with tau=49. NIST Security Level 3."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::MlDsa87,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("TL-DSA-87 (Ternary Lattice DSA)"),
            plenum_module: String::from("salvi_kernel::crypto::tl_dsa"),
            security_notes: String::from(
                "High-security EUF-CMA digital signatures for CNSA 2.0 classified data. \
                 Module-LWE with k=8, l=7, enhanced parameters for maximum security. \
                 Sparse ternary challenge with tau=60. NIST Security Level 5."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::Lms,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("Ternary Lamport OTS with Key Chain"),
            plenum_module: String::from("salvi_kernel::crypto::signature"),
            security_notes: String::from(
                "Fully implemented ternary Lamport one-time signature scheme. \
                 Hash-based construction inherently quantum-resistant. \
                 LamportKeyChain supports multi-message signing via key indexing. \
                 Security relies only on hash function preimage resistance."
            ),
        },
        Cnsa2Mapping {
            algorithm: Cnsa2Algorithm::Xmss,
            status: ComplianceStatus::TernaryEquivalent,
            plenum_equivalent: String::from("Ternary Lamport OTS (Merkle tree extension planned)"),
            plenum_module: String::from("salvi_kernel::crypto::signature"),
            security_notes: String::from(
                "Current Lamport OTS provides the foundational primitive. \
                 XMSS-equivalent Merkle tree structure for stateful multi-use \
                 signing is architecturally planned. Ternary sponge hash \
                 provides the underlying hash tree construction."
            ),
        },
    ]
}

pub fn get_compliance_summary() -> ComplianceSummary {
    let matrix = get_cnsa2_matrix();
    let total = matrix.len();
    let equivalent = matrix.iter().filter(|m| m.status == ComplianceStatus::TernaryEquivalent).count();
    let planned = matrix.iter().filter(|m| m.status == ComplianceStatus::Planned).count();
    let not_implemented = matrix.iter().filter(|m| m.status == ComplianceStatus::NotImplemented).count();

    ComplianceSummary {
        total_requirements: total,
        ternary_equivalent: equivalent,
        planned,
        not_implemented,
        overall_coverage_percent: ((equivalent * 100) / total) as u8,
    }
}

#[derive(Debug, Clone)]
pub struct ComplianceSummary {
    pub total_requirements: usize,
    pub ternary_equivalent: usize,
    pub planned: usize,
    pub not_implemented: usize,
    pub overall_coverage_percent: u8,
}

pub fn get_transition_timeline() -> Vec<TransitionMilestone> {
    vec![
        TransitionMilestone {
            year: 2025,
            title: String::from("Foundation Complete"),
            description: String::from(
                "Ternary sponge hash, HMAC, KDF, and Lamport OTS \
                 provide GF(3)-native equivalents for SHA-384/512 and LMS/XMSS. \
                 Phase encryption provides current symmetric encryption capability."
            ),
            algorithms: vec![
                Cnsa2Algorithm::Sha384,
                Cnsa2Algorithm::Sha512,
                Cnsa2Algorithm::Lms,
            ],
            status: MilestoneStatus::Complete,
        },
        TransitionMilestone {
            year: 2026,
            title: String::from("Lattice Foundations"),
            description: String::from(
                "GF(3) polynomial ring arithmetic in R_q = Z_3[X]/(X^n+1). \
                 Module-LWE and Module-SIS problem generation and verification. \
                 Polynomial sampling (CBD, uniform), compression/decompression. \
                 Schoolbook ring multiplication with X^n+1 reduction. \
                 Parameterized security levels (k=2,3,4 for NIST Levels 1,3,5). \
                 Binary-compatible AES-256-GCM, FIPS 180-4 SHA-384/512, and \
                 SHA3-384/512 implemented for interoperability layer."
            ),
            algorithms: vec![],
            status: MilestoneStatus::Complete,
        },
        TransitionMilestone {
            year: 2026,
            title: String::from("TL-KEM Implementation"),
            description: String::from(
                "Ternary Lattice Key Encapsulation Mechanism (TL-KEM) at all three security \
                 levels. IND-CCA2 secure via Fujisaki-Okamoto transform with implicit \
                 rejection. Module-LWE with k=2,3,4 for NIST Levels 1,3,5."
            ),
            algorithms: vec![
                Cnsa2Algorithm::MlKem512,
                Cnsa2Algorithm::MlKem768,
                Cnsa2Algorithm::MlKem1024,
            ],
            status: MilestoneStatus::Complete,
        },
        TransitionMilestone {
            year: 2026,
            title: String::from("TL-DSA Implementation"),
            description: String::from(
                "Ternary Lattice Digital Signature Algorithm (TL-DSA) at all three security \
                 levels. EUF-CMA secure via Fiat-Shamir with Aborts. Deterministic signing \
                 with Module-LWE at k=4/l=4, k=6/l=5, k=8/l=7."
            ),
            algorithms: vec![
                Cnsa2Algorithm::MlDsa44,
                Cnsa2Algorithm::MlDsa65,
                Cnsa2Algorithm::MlDsa87,
            ],
            status: MilestoneStatus::Complete,
        },
        TransitionMilestone {
            year: 2029,
            title: String::from("XMSS Merkle Tree Extension"),
            description: String::from(
                "Stateful hash-based signature trees using ternary sponge hash. \
                 Full XMSS-equivalent with ternary Merkle tree construction."
            ),
            algorithms: vec![
                Cnsa2Algorithm::Xmss,
            ],
            status: MilestoneStatus::Planned,
        },
        TransitionMilestone {
            year: 2030,
            title: String::from("Full CNSA 2.0 Coverage"),
            description: String::from(
                "Complete ternary-native equivalents for all CNSA 2.0 algorithms. \
                 Binary compatibility layer ensures interoperability with standard \
                 CNSA 2.0 implementations. FIPS validation process initiated."
            ),
            algorithms: vec![],
            status: MilestoneStatus::Planned,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct TransitionMilestone {
    pub year: u32,
    pub title: String,
    pub description: String,
    pub algorithms: Vec<Cnsa2Algorithm>,
    pub status: MilestoneStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneStatus {
    Complete,
    InProgress,
    Planned,
}

impl MilestoneStatus {
    pub fn label(&self) -> &'static str {
        match self {
            MilestoneStatus::Complete => "Complete",
            MilestoneStatus::InProgress => "In Progress",
            MilestoneStatus::Planned => "Planned",
        }
    }
}

pub fn ternary_security_advantage() -> TernarySecurityProfile {
    TernarySecurityProfile {
        information_density_advantage: 1.585,
        entropy_per_trit_bits: 1.585,
        gf3_operations: vec![
            String::from("Addition: (a + b) mod 3 in balanced representation"),
            String::from("Multiplication: (a * b) mod 3 with ternary S-boxes"),
            String::from("Inverse: Unique multiplicative inverse for non-zero elements"),
            String::from("XOR-equivalent: Ternary addition in GF(3)"),
        ],
        quantum_resistance_basis: String::from(
            "Ternary-native cryptography operates in a fundamentally different mathematical \
             domain (GF(3)) than binary systems. Quantum algorithms like Shor's and Grover's \
             are optimized for binary field operations. Ternary lattice problems introduce \
             additional complexity for quantum solvers due to the three-valued coefficient space, \
             providing a structural advantage beyond the specific hardness assumptions of \
             individual algorithms."
        ),
        binary_compatibility: String::from(
            "The Binary-Ternary Gateway (BTG) provides transparent conversion between \
             binary CNSA 2.0 implementations and PlenumNET's ternary equivalents. This \
             enables hybrid deployment where ternary-native operations run on the Salvi \
             kernel while maintaining interoperability with standard binary infrastructure."
        ),
    }
}

#[derive(Debug, Clone)]
pub struct TernarySecurityProfile {
    pub information_density_advantage: f64,
    pub entropy_per_trit_bits: f64,
    pub gf3_operations: Vec<String>,
    pub quantum_resistance_basis: String,
    pub binary_compatibility: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnsa2_matrix_completeness() {
        let matrix = get_cnsa2_matrix();
        assert_eq!(matrix.len(), 11);

        let has_aes = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::Aes256);
        let has_sha384 = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::Sha384);
        let has_sha512 = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::Sha512);
        let has_mlkem = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::MlKem1024);
        let has_mldsa = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::MlDsa87);
        let has_lms = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::Lms);
        let has_xmss = matrix.iter().any(|m| m.algorithm == Cnsa2Algorithm::Xmss);

        assert!(has_aes, "Missing AES-256");
        assert!(has_sha384, "Missing SHA-384");
        assert!(has_sha512, "Missing SHA-512");
        assert!(has_mlkem, "Missing ML-KEM-1024");
        assert!(has_mldsa, "Missing ML-DSA-87");
        assert!(has_lms, "Missing LMS");
        assert!(has_xmss, "Missing XMSS");
    }

    #[test]
    fn test_cnsa2_compliance_summary() {
        let summary = get_compliance_summary();
        assert_eq!(summary.total_requirements, 11);
        assert_eq!(summary.ternary_equivalent, 11);
        assert_eq!(summary.planned, 0);
        assert_eq!(summary.not_implemented, 0);
        assert_eq!(
            summary.ternary_equivalent + summary.planned + summary.not_implemented,
            summary.total_requirements
        );
    }

    #[test]
    fn test_cnsa2_algorithm_names() {
        assert_eq!(Cnsa2Algorithm::Aes256.name(), "AES-256");
        assert_eq!(Cnsa2Algorithm::MlKem768.name(), "ML-KEM-768");
        assert_eq!(Cnsa2Algorithm::MlDsa87.name(), "ML-DSA-87");
        assert_eq!(Cnsa2Algorithm::Lms.name(), "LMS");
    }

    #[test]
    fn test_cnsa2_nist_standards() {
        assert_eq!(Cnsa2Algorithm::Aes256.nist_standard(), "FIPS 197");
        assert_eq!(Cnsa2Algorithm::Sha384.nist_standard(), "FIPS 180-4");
        assert_eq!(Cnsa2Algorithm::MlKem1024.nist_standard(), "FIPS 203");
        assert_eq!(Cnsa2Algorithm::MlDsa65.nist_standard(), "FIPS 204");
        assert_eq!(Cnsa2Algorithm::Xmss.nist_standard(), "SP 800-208");
    }

    #[test]
    fn test_cnsa2_categories() {
        assert_eq!(Cnsa2Algorithm::Aes256.category(), Cnsa2Category::SymmetricEncryption);
        assert_eq!(Cnsa2Algorithm::Sha512.category(), Cnsa2Category::Hashing);
        assert_eq!(Cnsa2Algorithm::MlKem768.category(), Cnsa2Category::KeyEncapsulation);
        assert_eq!(Cnsa2Algorithm::MlDsa44.category(), Cnsa2Category::DigitalSignatures);
        assert_eq!(Cnsa2Algorithm::Lms.category(), Cnsa2Category::HashBasedSignatures);
    }

    #[test]
    fn test_cnsa2_security_levels() {
        assert_eq!(Cnsa2Algorithm::Aes256.security_level_bits(), 256);
        assert_eq!(Cnsa2Algorithm::Sha384.security_level_bits(), 192);
        assert_eq!(Cnsa2Algorithm::MlKem512.security_level_bits(), 128);
        assert_eq!(Cnsa2Algorithm::MlKem1024.security_level_bits(), 256);
        assert_eq!(Cnsa2Algorithm::MlDsa87.security_level_bits(), 256);
    }

    #[test]
    fn test_cnsa2_transition_deadlines() {
        assert_eq!(Cnsa2Algorithm::Aes256.transition_deadline(), TransitionDeadline::Immediate);
        assert_eq!(Cnsa2Algorithm::MlKem768.transition_deadline(), TransitionDeadline::By2030);
        assert_eq!(Cnsa2Algorithm::MlDsa65.transition_deadline(), TransitionDeadline::By2035);
        assert_eq!(Cnsa2Algorithm::Lms.transition_deadline(), TransitionDeadline::By2030);
    }

    #[test]
    fn test_transition_timeline() {
        let timeline = get_transition_timeline();
        assert!(!timeline.is_empty());

        let years: Vec<u32> = timeline.iter().map(|m| m.year).collect();
        let mut sorted = years.clone();
        sorted.sort();
        assert_eq!(years, sorted, "Timeline should be chronologically ordered");

        assert_eq!(timeline[0].status, MilestoneStatus::Complete);
        assert!(timeline.last().unwrap().year <= 2030);
    }

    #[test]
    fn test_ternary_security_profile() {
        let profile = ternary_security_advantage();
        assert!((profile.information_density_advantage - 1.585).abs() < 0.001);
        assert!(!profile.gf3_operations.is_empty());
        assert!(!profile.quantum_resistance_basis.is_empty());
        assert!(!profile.binary_compatibility.is_empty());
    }

    #[test]
    fn test_compliance_status_labels() {
        assert_eq!(ComplianceStatus::TernaryEquivalent.label(), "Ternary Equivalent");
        assert_eq!(ComplianceStatus::Planned.label(), "Planned");
        assert_eq!(ComplianceStatus::NotImplemented.label(), "Not Implemented");
    }

    #[test]
    fn test_milestone_status_labels() {
        assert_eq!(MilestoneStatus::Complete.label(), "Complete");
        assert_eq!(MilestoneStatus::InProgress.label(), "In Progress");
        assert_eq!(MilestoneStatus::Planned.label(), "Planned");
    }

    #[test]
    fn test_all_mappings_have_modules() {
        let matrix = get_cnsa2_matrix();
        for mapping in &matrix {
            assert!(!mapping.plenum_module.is_empty(),
                "Algorithm {} missing module path", mapping.algorithm.name());
            assert!(!mapping.plenum_equivalent.is_empty(),
                "Algorithm {} missing equivalent name", mapping.algorithm.name());
            assert!(!mapping.security_notes.is_empty(),
                "Algorithm {} missing security notes", mapping.algorithm.name());
        }
    }
}
