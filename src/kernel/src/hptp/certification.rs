use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegulatoryStandard {
    Finra613,
    MifidII,
    Ptp1588,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceLevel {
    NonCompliant,
    Degraded,
    Compliant,
    Exceeds,
}

#[derive(Debug, Clone)]
pub struct CertificationResult {
    pub standard: RegulatoryStandard,
    pub level: ComplianceLevel,
    pub measured_offset_ns: i64,
    pub max_allowed_offset_ns: i64,
    pub timestamp_fs: u128,
    pub details: String,
    pub certified: bool,
}

impl CertificationResult {
    pub fn new(
        standard: RegulatoryStandard,
        level: ComplianceLevel,
        measured_offset_ns: i64,
        max_allowed_offset_ns: i64,
        timestamp_fs: u128,
        details: String,
        certified: bool,
    ) -> Self {
        Self {
            standard,
            level,
            measured_offset_ns,
            max_allowed_offset_ns,
            timestamp_fs,
            details,
            certified,
        }
    }

    pub fn is_compliant(&self) -> bool {
        self.certified
    }
}

pub struct TimestampCertifier {
    standards: Vec<RegulatoryStandard>,
    certifications: Vec<CertificationResult>,
    total_checks: u64,
    passed_checks: u64,
}

impl TimestampCertifier {
    pub fn new() -> Self {
        Self {
            standards: Vec::new(),
            certifications: Vec::new(),
            total_checks: 0,
            passed_checks: 0,
        }
    }

    pub fn add_standard(&mut self, standard: RegulatoryStandard) {
        self.standards.push(standard);
    }

    pub fn certify_timestamp(&mut self, timestamp_fs: u128, clock_offset_ns: i64) -> Vec<CertificationResult> {
        let mut results = Vec::new();
        for standard in &self.standards {
            let result = match standard {
                RegulatoryStandard::Finra613 => self.check_finra_613(clock_offset_ns, timestamp_fs),
                RegulatoryStandard::MifidII => self.check_mifid_ii(clock_offset_ns, timestamp_fs),
                RegulatoryStandard::Ptp1588 => {
                    let abs_offset = if clock_offset_ns < 0 { -clock_offset_ns } else { clock_offset_ns };
                    let (level, certified) = if abs_offset < 1_000 {
                        (ComplianceLevel::Exceeds, true)
                    } else if abs_offset < 1_000_000 {
                        (ComplianceLevel::Compliant, true)
                    } else {
                        (ComplianceLevel::NonCompliant, false)
                    };
                    CertificationResult::new(
                        RegulatoryStandard::Ptp1588,
                        level,
                        clock_offset_ns,
                        1_000_000,
                        timestamp_fs,
                        String::from("PTP IEEE 1588 check"),
                        certified,
                    )
                }
                RegulatoryStandard::Custom(name) => {
                    CertificationResult::new(
                        RegulatoryStandard::Custom(name.clone()),
                        ComplianceLevel::Compliant,
                        clock_offset_ns,
                        0,
                        timestamp_fs,
                        alloc::format!("Custom standard: {}", name),
                        true,
                    )
                }
            };
            self.total_checks += 1;
            if result.certified {
                self.passed_checks += 1;
            }
            self.certifications.push(result.clone());
            results.push(result);
        }
        results
    }

    pub fn check_finra_613(&self, clock_offset_ns: i64, timestamp_fs: u128) -> CertificationResult {
        let abs_offset = if clock_offset_ns < 0 { -clock_offset_ns } else { clock_offset_ns };
        let (level, certified) = if abs_offset < 50_000_000 {
            (ComplianceLevel::Compliant, true)
        } else if abs_offset < 100_000_000 {
            (ComplianceLevel::Degraded, false)
        } else {
            (ComplianceLevel::NonCompliant, false)
        };
        CertificationResult::new(
            RegulatoryStandard::Finra613,
            level,
            clock_offset_ns,
            50_000_000,
            timestamp_fs,
            alloc::format!("FINRA 613: offset {}ns, max 50ms", clock_offset_ns),
            certified,
        )
    }

    pub fn check_mifid_ii(&self, clock_offset_ns: i64, timestamp_fs: u128) -> CertificationResult {
        let abs_offset = if clock_offset_ns < 0 { -clock_offset_ns } else { clock_offset_ns };
        let (level, certified) = if abs_offset < 100_000 {
            (ComplianceLevel::Exceeds, true)
        } else if abs_offset < 1_000_000 {
            (ComplianceLevel::Compliant, true)
        } else if abs_offset < 10_000_000 {
            (ComplianceLevel::Degraded, false)
        } else {
            (ComplianceLevel::NonCompliant, false)
        };
        CertificationResult::new(
            RegulatoryStandard::MifidII,
            level,
            clock_offset_ns,
            1_000_000,
            timestamp_fs,
            alloc::format!("MiFID II: offset {}ns", clock_offset_ns),
            certified,
        )
    }

    pub fn compliance_rate(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        (self.passed_checks as f64 / self.total_checks as f64) * 100.0
    }

    pub fn certification_count(&self) -> u64 {
        self.total_checks
    }

    pub fn last_certification(&self) -> Option<&CertificationResult> {
        self.certifications.last()
    }

    pub fn is_fully_compliant(&self) -> bool {
        if self.certifications.is_empty() {
            return false;
        }
        let standards_count = self.standards.len();
        if standards_count == 0 {
            return false;
        }
        let last_n: Vec<&CertificationResult> = self.certifications.iter().rev().take(standards_count).collect();
        last_n.iter().all(|r| r.certified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finra_613_compliant() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_finra_613(10_000_000, 1000);
        assert_eq!(result.level, ComplianceLevel::Compliant);
        assert!(result.certified);
    }

    #[test]
    fn test_finra_613_degraded() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_finra_613(75_000_000, 1000);
        assert_eq!(result.level, ComplianceLevel::Degraded);
        assert!(!result.certified);
    }

    #[test]
    fn test_finra_613_non_compliant() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_finra_613(200_000_000, 1000);
        assert_eq!(result.level, ComplianceLevel::NonCompliant);
        assert!(!result.certified);
    }

    #[test]
    fn test_finra_613_negative_offset() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_finra_613(-10_000_000, 1000);
        assert_eq!(result.level, ComplianceLevel::Compliant);
        assert!(result.certified);
    }

    #[test]
    fn test_mifid_ii_exceeds() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_mifid_ii(50_000, 2000);
        assert_eq!(result.level, ComplianceLevel::Exceeds);
        assert!(result.certified);
    }

    #[test]
    fn test_mifid_ii_compliant() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_mifid_ii(500_000, 2000);
        assert_eq!(result.level, ComplianceLevel::Compliant);
        assert!(result.certified);
    }

    #[test]
    fn test_mifid_ii_degraded() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_mifid_ii(5_000_000, 2000);
        assert_eq!(result.level, ComplianceLevel::Degraded);
        assert!(!result.certified);
    }

    #[test]
    fn test_mifid_ii_non_compliant() {
        let certifier = TimestampCertifier::new();
        let result = certifier.check_mifid_ii(50_000_000, 2000);
        assert_eq!(result.level, ComplianceLevel::NonCompliant);
        assert!(!result.certified);
    }

    #[test]
    fn test_certifier_with_multiple_standards() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.add_standard(RegulatoryStandard::MifidII);
        let results = certifier.certify_timestamp(5000, 1_000);
        assert_eq!(results.len(), 2);
        assert!(results[0].certified);
        assert!(results[1].certified);
    }

    #[test]
    fn test_compliance_rate() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.certify_timestamp(1000, 10_000_000);
        certifier.certify_timestamp(2000, 10_000_000);
        assert_eq!(certifier.compliance_rate(), 100.0);
    }

    #[test]
    fn test_compliance_rate_partial() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.certify_timestamp(1000, 10_000_000);
        certifier.certify_timestamp(2000, 200_000_000);
        assert_eq!(certifier.compliance_rate(), 50.0);
    }

    #[test]
    fn test_compliance_rate_zero() {
        let certifier = TimestampCertifier::new();
        assert_eq!(certifier.compliance_rate(), 0.0);
    }

    #[test]
    fn test_certification_count() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.add_standard(RegulatoryStandard::MifidII);
        certifier.certify_timestamp(1000, 100);
        assert_eq!(certifier.certification_count(), 2);
    }

    #[test]
    fn test_last_certification() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        assert!(certifier.last_certification().is_none());
        certifier.certify_timestamp(5000, 1000);
        let last = certifier.last_certification().unwrap();
        assert_eq!(last.timestamp_fs, 5000);
    }

    #[test]
    fn test_is_fully_compliant() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.add_standard(RegulatoryStandard::MifidII);
        certifier.certify_timestamp(1000, 1_000);
        assert!(certifier.is_fully_compliant());
    }

    #[test]
    fn test_is_not_fully_compliant() {
        let mut certifier = TimestampCertifier::new();
        certifier.add_standard(RegulatoryStandard::Finra613);
        certifier.add_standard(RegulatoryStandard::MifidII);
        certifier.certify_timestamp(1000, 60_000_000);
        assert!(!certifier.is_fully_compliant());
    }
}
