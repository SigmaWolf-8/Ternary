//! Cryptographic Agility Policy Engine (CNSA 2.0)
//!
//! Provides policy enforcement for algorithm selection during the quantum
//! transition period. Three policies control which algorithms are permitted:
//!
//! - **CnsaOnly**: Only CNSA 2.0 approved algorithms allowed (post-2030 target)
//! - **Hybrid**: Both CNSA 2.0 and legacy algorithms permitted (transition period)
//! - **Legacy**: Legacy algorithms only (pre-transition, not recommended)
//!
//! # Usage
//! The policy engine validates algorithm choices at the API boundary, ensuring
//! that system components cannot accidentally use deprecated or non-compliant
//! algorithms when the policy requires CNSA 2.0 compliance.
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::cnsa2::Cnsa2Algorithm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgilityPolicy {
    CnsaOnly,
    Hybrid,
    Legacy,
}

impl AgilityPolicy {
    pub fn name(&self) -> &'static str {
        match self {
            AgilityPolicy::CnsaOnly => "CNSA 2.0 Only",
            AgilityPolicy::Hybrid => "Hybrid (CNSA 2.0 + Legacy)",
            AgilityPolicy::Legacy => "Legacy Only",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AgilityPolicy::CnsaOnly => "Only CNSA 2.0 approved algorithms are permitted. Required for classified systems post-2030.",
            AgilityPolicy::Hybrid => "Both CNSA 2.0 and legacy algorithms are permitted. Suitable for transition period interoperability.",
            AgilityPolicy::Legacy => "Only legacy algorithms permitted. Not recommended — no quantum resistance.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmClass {
    Cnsa2Approved,
    LegacyAcceptable,
    Prohibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Cnsa2(Cnsa2Algorithm),
    RsaOaep2048,
    RsaOaep4096,
    RsaPss2048,
    RsaPss4096,
    EcdsaP256,
    EcdsaP384,
    X25519,
    Ed25519,
    Sha256,
    TripleDes,
    Rc4,
}

impl Algorithm {
    pub fn classify(&self) -> AlgorithmClass {
        match self {
            Algorithm::Cnsa2(_) => AlgorithmClass::Cnsa2Approved,
            Algorithm::RsaOaep4096
            | Algorithm::RsaPss4096
            | Algorithm::EcdsaP384
            | Algorithm::Sha256 => AlgorithmClass::LegacyAcceptable,
            Algorithm::RsaOaep2048
            | Algorithm::RsaPss2048
            | Algorithm::EcdsaP256
            | Algorithm::X25519
            | Algorithm::Ed25519 => AlgorithmClass::LegacyAcceptable,
            Algorithm::TripleDes | Algorithm::Rc4 => AlgorithmClass::Prohibited,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Cnsa2(alg) => alg.name(),
            Algorithm::RsaOaep2048 => "RSA-OAEP-2048",
            Algorithm::RsaOaep4096 => "RSA-OAEP-4096",
            Algorithm::RsaPss2048 => "RSA-PSS-2048",
            Algorithm::RsaPss4096 => "RSA-PSS-4096",
            Algorithm::EcdsaP256 => "ECDSA-P256",
            Algorithm::EcdsaP384 => "ECDSA-P384",
            Algorithm::X25519 => "X25519",
            Algorithm::Ed25519 => "Ed25519",
            Algorithm::Sha256 => "SHA-256",
            Algorithm::TripleDes => "3DES",
            Algorithm::Rc4 => "RC4",
        }
    }

    pub fn is_quantum_resistant(&self) -> bool {
        matches!(self, Algorithm::Cnsa2(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub algorithm: Algorithm,
    pub permitted: bool,
    pub reason: String,
}

pub fn evaluate(policy: AgilityPolicy, algorithm: Algorithm) -> PolicyDecision {
    let class = algorithm.classify();
    let (permitted, reason) = match policy {
        AgilityPolicy::CnsaOnly => match class {
            AlgorithmClass::Cnsa2Approved => (true, String::from("CNSA 2.0 approved algorithm")),
            AlgorithmClass::LegacyAcceptable => (false, String::from("Legacy algorithm not permitted under CNSA-Only policy")),
            AlgorithmClass::Prohibited => (false, String::from("Prohibited algorithm — known vulnerabilities")),
        },
        AgilityPolicy::Hybrid => match class {
            AlgorithmClass::Cnsa2Approved => (true, String::from("CNSA 2.0 approved (preferred)")),
            AlgorithmClass::LegacyAcceptable => (true, String::from("Legacy acceptable during hybrid transition")),
            AlgorithmClass::Prohibited => (false, String::from("Prohibited algorithm — known vulnerabilities")),
        },
        AgilityPolicy::Legacy => match class {
            AlgorithmClass::Cnsa2Approved => (false, String::from("CNSA 2.0 algorithms not available in Legacy mode")),
            AlgorithmClass::LegacyAcceptable => (true, String::from("Legacy algorithm permitted")),
            AlgorithmClass::Prohibited => (false, String::from("Prohibited algorithm — known vulnerabilities")),
        },
    };
    PolicyDecision { algorithm, permitted, reason }
}

pub fn evaluate_suite(policy: AgilityPolicy, algorithms: &[Algorithm]) -> Vec<PolicyDecision> {
    algorithms.iter().map(|alg| evaluate(policy, *alg)).collect()
}

pub fn all_permitted(decisions: &[PolicyDecision]) -> bool {
    decisions.iter().all(|d| d.permitted)
}

pub fn rejected_algorithms(decisions: &[PolicyDecision]) -> Vec<&PolicyDecision> {
    decisions.iter().filter(|d| !d.permitted).collect()
}

pub fn recommended_suite(policy: AgilityPolicy) -> Vec<Algorithm> {
    match policy {
        AgilityPolicy::CnsaOnly => vec![
            Algorithm::Cnsa2(Cnsa2Algorithm::Aes256),
            Algorithm::Cnsa2(Cnsa2Algorithm::Sha384),
            Algorithm::Cnsa2(Cnsa2Algorithm::MlKem1024),
            Algorithm::Cnsa2(Cnsa2Algorithm::MlDsa87),
            Algorithm::Cnsa2(Cnsa2Algorithm::Xmss),
        ],
        AgilityPolicy::Hybrid => vec![
            Algorithm::Cnsa2(Cnsa2Algorithm::Aes256),
            Algorithm::Cnsa2(Cnsa2Algorithm::Sha384),
            Algorithm::Cnsa2(Cnsa2Algorithm::MlKem1024),
            Algorithm::Cnsa2(Cnsa2Algorithm::MlDsa87),
            Algorithm::EcdsaP384,
            Algorithm::RsaPss4096,
        ],
        AgilityPolicy::Legacy => vec![
            Algorithm::RsaPss4096,
            Algorithm::EcdsaP384,
            Algorithm::Sha256,
        ],
    }
}

#[derive(Debug, Clone)]
pub struct PolicyReport {
    pub policy: AgilityPolicy,
    pub total_evaluated: usize,
    pub permitted_count: usize,
    pub rejected_count: usize,
    pub quantum_resistant_count: usize,
    pub decisions: Vec<PolicyDecision>,
}

pub fn generate_report(policy: AgilityPolicy, algorithms: &[Algorithm]) -> PolicyReport {
    let decisions = evaluate_suite(policy, algorithms);
    let permitted_count = decisions.iter().filter(|d| d.permitted).count();
    let rejected_count = decisions.len() - permitted_count;
    let quantum_resistant_count = algorithms.iter().filter(|a| a.is_quantum_resistant()).count();
    PolicyReport {
        policy,
        total_evaluated: algorithms.len(),
        permitted_count,
        rejected_count,
        quantum_resistant_count,
        decisions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnsa_only_blocks_legacy() {
        let decision = evaluate(AgilityPolicy::CnsaOnly, Algorithm::EcdsaP256);
        assert!(!decision.permitted);
    }

    #[test]
    fn test_cnsa_only_allows_cnsa2() {
        let decision = evaluate(AgilityPolicy::CnsaOnly, Algorithm::Cnsa2(Cnsa2Algorithm::Aes256));
        assert!(decision.permitted);
    }

    #[test]
    fn test_hybrid_allows_both() {
        let d1 = evaluate(AgilityPolicy::Hybrid, Algorithm::Cnsa2(Cnsa2Algorithm::MlKem1024));
        assert!(d1.permitted);
        let d2 = evaluate(AgilityPolicy::Hybrid, Algorithm::EcdsaP384);
        assert!(d2.permitted);
    }

    #[test]
    fn test_prohibited_blocked_everywhere() {
        for policy in &[AgilityPolicy::CnsaOnly, AgilityPolicy::Hybrid, AgilityPolicy::Legacy] {
            let d = evaluate(*policy, Algorithm::Rc4);
            assert!(!d.permitted);
            let d2 = evaluate(*policy, Algorithm::TripleDes);
            assert!(!d2.permitted);
        }
    }

    #[test]
    fn test_legacy_blocks_cnsa2() {
        let decision = evaluate(AgilityPolicy::Legacy, Algorithm::Cnsa2(Cnsa2Algorithm::MlDsa87));
        assert!(!decision.permitted);
    }

    #[test]
    fn test_legacy_allows_acceptable() {
        let decision = evaluate(AgilityPolicy::Legacy, Algorithm::RsaPss4096);
        assert!(decision.permitted);
    }

    #[test]
    fn test_suite_evaluation() {
        let suite = vec![
            Algorithm::Cnsa2(Cnsa2Algorithm::Aes256),
            Algorithm::Cnsa2(Cnsa2Algorithm::Sha384),
            Algorithm::RsaPss4096,
        ];
        let decisions = evaluate_suite(AgilityPolicy::CnsaOnly, &suite);
        assert_eq!(decisions.len(), 3);
        assert!(decisions[0].permitted);
        assert!(decisions[1].permitted);
        assert!(!decisions[2].permitted);
        assert!(!all_permitted(&decisions));
    }

    #[test]
    fn test_recommended_suites() {
        let cnsa = recommended_suite(AgilityPolicy::CnsaOnly);
        assert!(!cnsa.is_empty());
        for alg in &cnsa {
            assert!(alg.is_quantum_resistant());
        }
        let legacy = recommended_suite(AgilityPolicy::Legacy);
        for alg in &legacy {
            assert!(!alg.is_quantum_resistant());
        }
    }

    #[test]
    fn test_policy_report() {
        let algs = vec![
            Algorithm::Cnsa2(Cnsa2Algorithm::Aes256),
            Algorithm::Rc4,
            Algorithm::EcdsaP256,
        ];
        let report = generate_report(AgilityPolicy::CnsaOnly, &algs);
        assert_eq!(report.total_evaluated, 3);
        assert_eq!(report.permitted_count, 1);
        assert_eq!(report.rejected_count, 2);
        assert_eq!(report.quantum_resistant_count, 1);
    }

    #[test]
    fn test_algorithm_names() {
        assert_eq!(Algorithm::Cnsa2(Cnsa2Algorithm::Xmss).name(), "XMSS");
        assert_eq!(Algorithm::EcdsaP256.name(), "ECDSA-P256");
        assert_eq!(Algorithm::TripleDes.name(), "3DES");
    }

    #[test]
    fn test_policy_names() {
        assert_eq!(AgilityPolicy::CnsaOnly.name(), "CNSA 2.0 Only");
        assert_eq!(AgilityPolicy::Hybrid.name(), "Hybrid (CNSA 2.0 + Legacy)");
        assert_eq!(AgilityPolicy::Legacy.name(), "Legacy Only");
    }
}
