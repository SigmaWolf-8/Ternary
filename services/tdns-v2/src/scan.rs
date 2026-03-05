// TDNS v2.3 — Scan Types
// Capomastro Holdings Ltd. — Applied Physics Division
//
// CRS scan results and the derivation pipeline.
// Each dimension produces a ScanMeasurement → Trit derivation.
// The scan_hash binds the address to its measurements.

use serde::{Deserialize, Serialize};

use crate::addr::{CubeAddr, DIMENSIONS};
use crate::trit::Trit;

// ─── Scan Hash ───────────────────────────────────────────────────────────────

/// BLAKE3 hash of the complete scan results.
/// Stored in TRN records. CRS-signed. Binds address to measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScanHash(pub [u8; 32]);

impl ScanHash {
    /// Compute BLAKE3 hash of serialized scan results.
    pub fn compute(scan_data: &[u8]) -> Self {
        Self(*blake3::hash(scan_data).as_bytes())
    }

    /// The raw 32-byte hash.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Display as hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl std::fmt::Display for ScanHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// ─── Raw Scan Measurement ────────────────────────────────────────────────────

/// A single measurement from a CRS scan, before trit derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMeasurement {
    /// Which dimension this measures (0-based).
    pub dim: usize,
    /// The raw measurement value (interpretation depends on dimension).
    pub raw: RawValue,
    /// Confidence level: how certain CRS is about this measurement.
    pub confidence: Confidence,
}

/// Raw values from different scan methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawValue {
    /// String-based (e.g., WHOIS result, TLS version).
    Text(String),
    /// Numeric (e.g., record count, latency ms, tracker count).
    Numeric(f64),
    /// Boolean (e.g., presence of /privacy page).
    Boolean(bool),
    /// Categorical match from a list of known patterns.
    Pattern(String),
}

/// CRS scan confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// Definitive measurement — no ambiguity.
    High,
    /// Reasonable inference from partial signals.
    Medium,
    /// Best guess from weak signals.
    Low,
}

// ─── Complete Scan Result ────────────────────────────────────────────────────

/// The complete scan result for an entity: 27 measurements + derived address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// The target that was scanned (URL, IP, domain).
    pub target: String,
    /// Timestamp of the scan (HPTP nanoseconds).
    pub scanned_at: u64,
    /// Raw measurements for each dimension.
    pub measurements: Vec<ScanMeasurement>,
    /// The derived 27-trit address.
    pub derived_address: CubeAddr,
    /// BLAKE3 hash of the serialized measurements.
    pub scan_hash: ScanHash,
}

impl ScanResult {
    /// Build a ScanResult from measurements and derived trits.
    ///
    /// In production, each measurement is fed through a dimension-specific
    /// derivation function to produce the trit. Here we accept pre-derived
    /// trits for testing and manual construction.
    pub fn new(
        target: String,
        scanned_at: u64,
        measurements: Vec<ScanMeasurement>,
        trit_values: [Trit; DIMENSIONS],
    ) -> Self {
        let derived_address = CubeAddr::new(trit_values);

        // Compute scan hash from FULL measurement data.
        // Includes: target URL, timestamp, and every raw value + confidence.
        // This ensures two different entities with different raw observations
        // always produce different hashes, even if their trit vectors match.
        let mut hasher = blake3::Hasher::new();
        hasher.update(target.as_bytes());
        hasher.update(&scanned_at.to_be_bytes());
        for m in &measurements {
            hasher.update(&[m.dim as u8]);
            hasher.update(&[match m.confidence {
                Confidence::High => 3,
                Confidence::Medium => 2,
                Confidence::Low => 1,
            }]);
            // Serialize raw value into hash
            match &m.raw {
                RawValue::Text(s) => {
                    hasher.update(&[0x01]); // type tag
                    hasher.update(s.as_bytes());
                }
                RawValue::Numeric(n) => {
                    hasher.update(&[0x02]);
                    hasher.update(&n.to_be_bytes());
                }
                RawValue::Boolean(b) => {
                    hasher.update(&[0x03]);
                    hasher.update(&[*b as u8]);
                }
                RawValue::Pattern(s) => {
                    hasher.update(&[0x04]);
                    hasher.update(s.as_bytes());
                }
            }
        }
        // Also include the derived trit vector for completeness
        for t in &trit_values {
            hasher.update(&[t.value()]);
        }
        let hash_output = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(hash_output.as_bytes());
        let scan_hash = ScanHash(hash_bytes);

        Self {
            target,
            scanned_at,
            measurements,
            derived_address,
            scan_hash,
        }
    }

    /// Number of high-confidence measurements.
    pub fn high_confidence_count(&self) -> usize {
        self.measurements
            .iter()
            .filter(|m| m.confidence == Confidence::High)
            .count()
    }

    /// Any dimensions where confidence was low?
    pub fn low_confidence_dims(&self) -> Vec<usize> {
        self.measurements
            .iter()
            .filter(|m| m.confidence == Confidence::Low)
            .map(|m| m.dim)
            .collect()
    }
}

// ─── Derivation Rule (Trait) ─────────────────────────────────────────────────

/// A derivation rule converts a raw scan measurement into a trit value.
///
/// Each of the 27 dimensions implements this trait with dimension-specific
/// logic. The CRS scanner framework calls these in sequence to produce
/// the 27-trit address.
pub trait DerivationRule: Send + Sync {
    /// Which dimension this rule covers (0-based).
    fn dimension(&self) -> usize;

    /// Derive a trit value from a raw measurement.
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError>;
}

/// Errors during trit derivation.
#[derive(Debug, thiserror::Error)]
pub enum DerivationError {
    #[error("cannot derive trit for dim {dim}: {reason}")]
    Underivable { dim: usize, reason: String },

    #[error("unexpected raw value type for dim {dim}")]
    TypeMismatch { dim: usize },
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_hash_deterministic() {
        let data = b"test scan data";
        let h1 = ScanHash::compute(data);
        let h2 = ScanHash::compute(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn scan_hash_different_for_different_data() {
        let h1 = ScanHash::compute(b"scan A");
        let h2 = ScanHash::compute(b"scan B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn scan_hash_is_32_bytes() {
        let h = ScanHash::compute(b"data");
        assert_eq!(h.as_bytes().len(), 32);
    }

    #[test]
    fn scan_hash_hex_is_64_chars() {
        let h = ScanHash::compute(b"data");
        assert_eq!(h.to_hex().len(), 64);
    }

    #[test]
    fn scan_result_construction() {
        let trits: [Trit; 27] = [Trit::V2; 27];
        let result = ScanResult::new(
            "https://example.com".into(),
            1_000_000_000,
            vec![],
            trits,
        );

        assert_eq!(result.target, "https://example.com");
        assert_eq!(result.derived_address.trit(0), Trit::V2);
    }
}