// TDNS v2.3 — TRN (Ternary Resource Name)
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The TRN is the fundamental record of TDNS: a binding between
// a human name, a 27-trit address, a public key, and a scan hash.
//
// Wire format: length-prefixed binary, network byte order.

use serde::{Deserialize, Serialize};

use crate::addr::CubeAddr;
use crate::scan::ScanHash;

// ─── HPTP Sync Status ───────────────────────────────────────────────────────

/// Synchronization status for HPTP-mandatory entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HptpSyncStatus {
    /// Within tolerance.
    Synced,
    /// Outside tolerance but still operational.
    Degraded,
    /// Sync status not measured or not applicable.
    Unknown,
}

impl HptpSyncStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HptpSyncStatus::Synced => "synced",
            HptpSyncStatus::Degraded => "degraded",
            HptpSyncStatus::Unknown => "unknown",
        }
    }
}

// ─── TRN Record ──────────────────────────────────────────────────────────────

/// A Ternary Resource Name record.
///
/// This is the TDNS equivalent of a DNS record — it binds a human-readable
/// name to a 27-trit address, a public key for ownership proof, and a
/// scan hash that proves the address was derived from actual measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trn {
    // ── Required fields (§7.1) ───────────────────────────────────────

    /// Human-readable name (e.g., "pptpro.capomastro.plm").
    pub name: String,

    /// The 27-trit address, derived from CRS scan measurements.
    pub address: CubeAddr,

    /// Entity's public key (for ownership proof via challenge-response).
    pub public_key: Vec<u8>,

    /// Cache time-to-live in seconds.
    pub ttl: u32,

    /// HPTP nanosecond timestamp of registration.
    pub registered_at: u64,

    /// Authoritative zone (e.g., "capomastro.plm").
    pub zone: String,

    /// BLAKE3 hash of the scan results that produced this address.
    /// CRS-signed. Binds address to measurements.
    pub scan_hash: ScanHash,

    // ── Optional fields (§7.2) ───────────────────────────────────────

    /// HPTP timestamp: resolves only after this time.
    pub valid_from: Option<u64>,

    /// HPTP timestamp: resolves only before this time.
    pub valid_until: Option<u64>,

    /// HPTP synchronization status. Required for HPTP-mandatory entities.
    pub hptp_sync_status: Option<HptpSyncStatus>,

    /// Last reported HPTP offset in nanoseconds.
    pub hptp_offset_ns: Option<i64>,

    /// The 27 measured attribute values as scanned by CRS.
    /// Stored for re-verification (§9.4 step 6).
    pub attributes: Option<Vec<(u8, u8)>>,

    /// HPTP timestamp of most recent CRS re-scan.
    pub last_rescan: Option<u64>,

    // ── Collision Resolution (§4.3–4.7) ──────────────────────────────

    /// Collision Resolution Digit (1–9). Distinguishes up to 9 entities
    /// sharing the same 27-trit address. First-come-first-serve.
    pub crd: u8,

    /// Confidence vector: one digit (1–9) per dimension, measuring
    /// distance from nearest quantization boundary.
    /// C_i = min(floor(27 * δ_i) + 1, 9) where δ_i = min(|p_i - 1/3|, |p_i - 2/3|).
    /// Stored as metadata, not used for routing.
    pub confidence: Option<Vec<u8>>,
}

impl Trn {
    /// Create a new TRN with required fields only.
    pub fn new(
        name: String,
        address: CubeAddr,
        public_key: Vec<u8>,
        ttl: u32,
        registered_at: u64,
        zone: String,
        scan_hash: ScanHash,
    ) -> Self {
        Self {
            name,
            address,
            public_key,
            ttl,
            registered_at,
            zone,
            scan_hash,
            valid_from: None,
            valid_until: None,
            hptp_sync_status: None,
            hptp_offset_ns: None,
            attributes: None,
            last_rescan: None,
            crd: 1,
            confidence: None,
        }
    }

    /// Is this TRN valid at the given HPTP timestamp?
    pub fn is_valid_at(&self, now_ns: u64) -> bool {
        if let Some(from) = self.valid_from {
            if now_ns < from {
                return false;
            }
        }
        if let Some(until) = self.valid_until {
            if now_ns > until {
                return false;
            }
        }
        true
    }

    /// Is this entity HPTP-mandatory (trits 15+16 both = 3)?
    pub fn is_hptp_mandatory(&self) -> bool {
        self.address.is_hptp_mandatory()
    }

    /// Is HPTP sync confirmed?
    pub fn is_hptp_synced(&self) -> bool {
        matches!(self.hptp_sync_status, Some(HptpSyncStatus::Synced))
    }

    /// Is the cache entry expired?
    pub fn is_cache_expired(&self, registered_at: u64, now_ns: u64) -> bool {
        let ttl_ns = (self.ttl as u64) * 1_000_000_000;
        now_ns > registered_at + ttl_ns
    }

    /// Extract the zone from the name if not explicitly set.
    pub fn effective_zone(&self) -> &str {
        if self.zone.is_empty() {
            // Fall back: everything after first dot
            self.name
                .find('.')
                .map(|i| &self.name[i + 1..])
                .unwrap_or(&self.name)
        } else {
            &self.zone
        }
    }

    /// Set time-lock window (for auction names, etc.).
    pub fn with_time_lock(mut self, valid_from: u64, valid_until: u64) -> Self {
        self.valid_from = Some(valid_from);
        self.valid_until = Some(valid_until);
        self
    }

    /// Set HPTP sync status.
    pub fn with_hptp_status(mut self, status: HptpSyncStatus, offset_ns: i64) -> Self {
        self.hptp_sync_status = Some(status);
        self.hptp_offset_ns = Some(offset_ns);
        self
    }

    /// Set the Collision Resolution Digit (1–9).
    pub fn with_crd(mut self, crd: u8) -> Self {
        assert!(crd >= 1 && crd <= 9, "CRD must be 1–9");
        self.crd = crd;
        self
    }

    /// Set the confidence vector.
    pub fn with_confidence(mut self, confidence: Vec<u8>) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Full identifier: (address, CRD) pair. Globally unique.
    pub fn full_id(&self) -> (CubeAddr, u8) {
        (self.address, self.crd)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trn(name: &str, addr_str: &str) -> Trn {
        let addr = CubeAddr::from_category_string(addr_str).unwrap();
        let scan_hash = ScanHash::compute(b"test");
        Trn::new(
            name.into(),
            addr,
            vec![0xDE, 0xAD],
            3600,
            1_000_000_000,
            "capomastro.plm".into(),
            scan_hash,
        )
    }

    #[test]
    fn pptpro_is_hptp_mandatory() {
        let trn = make_trn(
            "pptpro.capomastro.plm",
            "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332",
        );
        assert!(trn.is_hptp_mandatory());
    }

    #[test]
    fn blog_is_not_hptp_mandatory() {
        let trn = make_trn(
            "nonnas-cucina.plm",
            "WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211",
        );
        assert!(!trn.is_hptp_mandatory());
    }

    #[test]
    fn time_lock_validation() {
        let trn = make_trn(
            "auction.plm",
            "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313",
        )
        .with_time_lock(100, 200);

        assert!(!trn.is_valid_at(50));  // before window
        assert!(trn.is_valid_at(150)); // in window
        assert!(!trn.is_valid_at(250)); // after window
    }

    #[test]
    fn no_time_lock_always_valid() {
        let trn = make_trn(
            "google.plm",
            "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313",
        );
        assert!(trn.is_valid_at(0));
        assert!(trn.is_valid_at(u64::MAX));
    }

    #[test]
    fn effective_zone() {
        let trn = make_trn(
            "pptpro.capomastro.plm",
            "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332",
        );
        assert_eq!(trn.effective_zone(), "capomastro.plm");
    }

    #[test]
    fn hptp_status_tracking() {
        let trn = make_trn(
            "pptpro.capomastro.plm",
            "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332",
        )
        .with_hptp_status(HptpSyncStatus::Synced, 42);

        assert!(trn.is_hptp_synced());
        assert_eq!(trn.hptp_offset_ns, Some(42));
    }
}
