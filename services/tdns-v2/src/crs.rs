// TDNS v2.3 — CRS Registry Service
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The Cube Registration Service: scans entities, derives 27-trit addresses,
// stores TRN records, maintains neighbor maps, and detects property drift.
//
// CRS is the critical trust anchor (§15.6). All address derivation flows
// through this service. No human judgment. No self-reported values.
//
// §12.1: Scans entities and derives addresses from measurements.
// §13:   Periodic re-scan and re-derivation on property drift.
// §9.4:  Open re-verification protocol support.

use std::collections::{BTreeMap, HashMap};

use crate::addr::{CubeAddr, DIMENSIONS};
use crate::derive::all_rules;
use crate::routing::NeighborMap;
use crate::scan::{DerivationRule, RawValue, ScanMeasurement, ScanResult};
use crate::trit::Trit;
use crate::trn::{HptpSyncStatus, Trn};

// ─── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CrsConfig {
    pub default_ttl: u32,
    pub rescan_interval_ns: u64,
    pub drift_grace_period_ns: u64,
    pub hptp_realtime_tolerance_ns: u64,
    pub hptp_neartime_tolerance_ns: u64,
    pub max_registrations_per_zone_per_hour: u32,
}

impl Default for CrsConfig {
    fn default() -> Self {
        Self {
            default_ttl: 3600,
            rescan_interval_ns: 7 * 24 * 3600 * 1_000_000_000,
            drift_grace_period_ns: 24 * 3600 * 1_000_000_000,
            hptp_realtime_tolerance_ns: 1_000,
            hptp_neartime_tolerance_ns: 100_000,
            max_registrations_per_zone_per_hour: 100,
        }
    }
}

// ─── Drift Event ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DriftEvent {
    pub name: String,
    pub old_address: CubeAddr,
    pub new_address: CubeAddr,
    pub changed_dims: Vec<usize>,
    pub detected_at: u64,
}

// ─── Registration Result ─────────────────────────────────────────────────────

#[derive(Debug)]
pub enum RegistrationResult {
    Ok {
        trn: Trn,
        address: CubeAddr,
    },
    HptpSyncRequired {
        address: CubeAddr,
        measured_offset_ns: i64,
        tolerance_ns: u64,
    },
    ScanFailed {
        reason: String,
    },
}

// ─── Re-verification Result (§9.4) ──────────────────────────────────────────

#[derive(Debug)]
pub enum VerificationResult {
    Verified {
        name: String,
        address: CubeAddr,
    },
    Drifted {
        name: String,
        old_address: CubeAddr,
        new_address: CubeAddr,
        changed_dims: Vec<usize>,
    },
    NotFound {
        name: String,
    },
}

// ─── CRS Registry ────────────────────────────────────────────────────────────

pub struct CrsRegistry {
    config: CrsConfig,
    records_by_name: HashMap<String, Trn>,
    names_by_addr: BTreeMap<CubeAddr, String>,
    neighbor_maps: HashMap<CubeAddr, NeighborMap>,
    drift_log: Vec<DriftEvent>,
    redirects: HashMap<CubeAddr, (CubeAddr, u64)>,
    rules: Vec<Box<dyn DerivationRule>>,
}

impl CrsRegistry {
    pub fn new() -> Self {
        Self::with_config(CrsConfig::default())
    }

    pub fn with_config(config: CrsConfig) -> Self {
        Self {
            config,
            records_by_name: HashMap::new(),
            names_by_addr: BTreeMap::new(),
            neighbor_maps: HashMap::new(),
            drift_log: Vec::new(),
            redirects: HashMap::new(),
            rules: all_rules(),
        }
    }

    // ── Core Derivation ─────────────────────────────────────────────────

    pub fn derive_address(
        &self,
        measurements: &[RawValue],
    ) -> Result<([Trit; DIMENSIONS], Vec<ScanMeasurement>), String> {
        if measurements.len() != DIMENSIONS {
            return Err(format!(
                "expected {} measurements, got {}",
                DIMENSIONS,
                measurements.len()
            ));
        }

        let mut trits = [Trit::V1; DIMENSIONS];
        let mut scan_measurements = Vec::with_capacity(DIMENSIONS);

        for (i, (rule, raw)) in self.rules.iter().zip(measurements.iter()).enumerate() {
            let (trit, confidence) = rule.derive(raw).map_err(|e| format!("dim {}: {}", i + 1, e))?;
            trits[i] = trit;
            scan_measurements.push(ScanMeasurement {
                dim: i,
                raw: raw.clone(),
                confidence,
            });
        }

        Ok((trits, scan_measurements))
    }

    // ── Registration ────────────────────────────────────────────────────

    pub fn register(
        &mut self,
        name: String,
        zone: String,
        public_key: Vec<u8>,
        measurements: Vec<RawValue>,
        now_ns: u64,
        hptp_offset_ns: Option<i64>,
    ) -> RegistrationResult {
        let (trits, scan_measurements) = match self.derive_address(&measurements) {
            Ok(result) => result,
            Err(reason) => return RegistrationResult::ScanFailed { reason },
        };

        let address = CubeAddr::new(trits);

        if address.is_hptp_mandatory() {
            let offset = hptp_offset_ns.unwrap_or(i64::MAX);
            let tolerance = self.config.hptp_realtime_tolerance_ns;
            if offset.unsigned_abs() > tolerance {
                return RegistrationResult::HptpSyncRequired {
                    address,
                    measured_offset_ns: offset,
                    tolerance_ns: tolerance,
                };
            }
        }

        let scan_result = ScanResult::new(
            name.clone(),
            now_ns,
            scan_measurements,
            trits,
        );
        let scan_hash = scan_result.scan_hash;

        let mut trn = Trn::new(
            name.clone(),
            address,
            public_key,
            self.config.default_ttl,
            now_ns,
            zone,
            scan_hash,
        );

        if address.is_hptp_mandatory() {
            let offset = hptp_offset_ns.unwrap_or(0);
            trn = trn.with_hptp_status(HptpSyncStatus::Synced, offset);
        }
        trn.last_rescan = Some(now_ns);

        self.records_by_name.insert(name.clone(), trn.clone());
        self.names_by_addr.insert(address, name);

        self.init_neighbor_map(address);
        self.update_neighbor_maps_for_registration(address);

        RegistrationResult::Ok { trn, address }
    }

    // ── Lookup ──────────────────────────────────────────────────────────

    pub fn resolve(&self, name: &str) -> Option<&Trn> {
        self.records_by_name.get(name)
    }

    pub fn resolve_addr(&self, name: &str) -> Option<CubeAddr> {
        self.records_by_name.get(name).map(|trn| trn.address)
    }

    pub fn reverse_lookup(&self, addr: &CubeAddr) -> Option<&str> {
        self.names_by_addr.get(addr).map(|s| s.as_str())
    }

    pub fn check_redirect(&self, addr: &CubeAddr, now_ns: u64) -> Option<CubeAddr> {
        if let Some(&(new_addr, expires)) = self.redirects.get(addr) {
            if now_ns <= expires {
                return Some(new_addr);
            }
        }
        None
    }

    pub fn entity_count(&self) -> usize {
        self.records_by_name.len()
    }

    // ── Re-scan & Drift (§13) ───────────────────────────────────────────

    pub fn rescan(
        &mut self,
        name: &str,
        new_measurements: Vec<RawValue>,
        now_ns: u64,
    ) -> Option<VerificationResult> {
        let current_trn = self.records_by_name.get(name)?.clone();
        let old_address = current_trn.address;

        let (new_trits, scan_measurements) = match self.derive_address(&new_measurements) {
            Ok(result) => result,
            Err(_) => return None,
        };
        let new_address = CubeAddr::new(new_trits);

        if let Some(trn) = self.records_by_name.get_mut(name) {
            trn.last_rescan = Some(now_ns);
        }

        if new_address == old_address {
            return Some(VerificationResult::Verified {
                name: name.to_string(),
                address: old_address,
            });
        }

        let changed_dims = old_address.differing_dims(&new_address);

        self.drift_log.push(DriftEvent {
            name: name.to_string(),
            old_address,
            new_address,
            changed_dims: changed_dims.clone(),
            detected_at: now_ns,
        });

        let scan_result = ScanResult::new(
            name.to_string(),
            now_ns,
            scan_measurements,
            new_trits,
        );

        if let Some(trn) = self.records_by_name.get_mut(name) {
            trn.address = new_address;
            trn.scan_hash = scan_result.scan_hash;
            trn.last_rescan = Some(now_ns);
        }

        self.names_by_addr.remove(&old_address);
        self.names_by_addr.insert(new_address, name.to_string());

        let expires = now_ns + self.config.drift_grace_period_ns;
        self.redirects.insert(old_address, (new_address, expires));

        self.remove_from_neighbor_maps(old_address);
        self.init_neighbor_map(new_address);
        self.update_neighbor_maps_for_registration(new_address);

        Some(VerificationResult::Drifted {
            name: name.to_string(),
            old_address,
            new_address,
            changed_dims,
        })
    }

    pub fn entities_due_for_rescan(&self, now_ns: u64) -> Vec<String> {
        self.records_by_name
            .iter()
            .filter(|(_, trn)| {
                let last = trn.last_rescan.unwrap_or(trn.registered_at);
                now_ns - last >= self.config.rescan_interval_ns
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    // ── Re-verification (§9.4) ──────────────────────────────────────────

    pub fn verify(
        &mut self,
        name: &str,
        fresh_measurements: Vec<RawValue>,
        now_ns: u64,
    ) -> VerificationResult {
        match self.rescan(name, fresh_measurements, now_ns) {
            Some(result) => result,
            None => VerificationResult::NotFound {
                name: name.to_string(),
            },
        }
    }

    // ── Deregistration ──────────────────────────────────────────────────

    pub fn deregister(&mut self, name: &str) -> Option<Trn> {
        if let Some(trn) = self.records_by_name.remove(name) {
            self.names_by_addr.remove(&trn.address);
            self.remove_from_neighbor_maps(trn.address);
            self.neighbor_maps.remove(&trn.address);
            Some(trn)
        } else {
            None
        }
    }

    // ── Neighbor Map Management ─────────────────────────────────────────

    pub fn neighbor_map(&self, addr: &CubeAddr) -> Option<&NeighborMap> {
        self.neighbor_maps.get(addr)
    }

    fn init_neighbor_map(&mut self, addr: CubeAddr) {
        if !self.neighbor_maps.contains_key(&addr) {
            self.neighbor_maps.insert(addr, NeighborMap::new(addr));
        }
    }

    fn update_neighbor_maps_for_registration(&mut self, new_addr: CubeAddr) {
        let all_addrs: Vec<CubeAddr> = self.names_by_addr.keys().copied().collect();

        for existing_addr in &all_addrs {
            if *existing_addr == new_addr {
                continue;
            }

            for dim in 0..DIMENSIONS {
                let existing_val = existing_addr.trit(dim);
                let new_val = new_addr.trit(dim);

                if existing_val != new_val {
                    let new_dist = existing_addr.distance(&new_addr);

                    if let Some(map) = self.neighbor_maps.get_mut(existing_addr) {
                        let should_update = match map.get(dim, new_val) {
                            Some(current) => new_dist < current.distance,
                            None => true,
                        };
                        if should_update {
                            map.set(dim, new_val, new_addr);
                        }
                    }

                    if let Some(new_map) = self.neighbor_maps.get_mut(&new_addr) {
                        let rev_dist = new_addr.distance(existing_addr);
                        let should_update = match new_map.get(dim, existing_val) {
                            Some(current) => rev_dist < current.distance,
                            None => true,
                        };
                        if should_update {
                            new_map.set(dim, existing_val, *existing_addr);
                        }
                    }
                }
            }
        }
    }

    fn remove_from_neighbor_maps(&mut self, removed_addr: CubeAddr) {
        for (_, map) in self.neighbor_maps.iter_mut() {
            for dim in 0..DIMENSIONS {
                let removed_val = removed_addr.trit(dim);
                if let Some(entry) = map.get(dim, removed_val) {
                    if entry.addr == removed_addr {
                        map.remove(dim, removed_val);
                    }
                }
            }
        }
    }

    // ── Audit ───────────────────────────────────────────────────────────

    pub fn drift_log(&self) -> &[DriftEvent] {
        &self.drift_log
    }

    pub fn expired_redirects(&self, now_ns: u64) -> Vec<CubeAddr> {
        self.redirects
            .iter()
            .filter(|(_, (_, expires))| now_ns > *expires)
            .map(|(old_addr, _)| *old_addr)
            .collect()
    }

    pub fn purge_expired_redirects(&mut self, now_ns: u64) -> usize {
        let expired = self.expired_redirects(now_ns);
        let count = expired.len();
        for addr in expired {
            self.redirects.remove(&addr);
        }
        count
    }

    // ── Metrics ─────────────────────────────────────────────────────────

    pub fn dimension_density(&self) -> Vec<[usize; 3]> {
        let mut density = vec![[0usize; 3]; DIMENSIONS];
        for trn in self.records_by_name.values() {
            for dim in 0..DIMENSIONS {
                let idx = trn.address.trit(dim).index();
                density[dim][idx] += 1;
            }
        }
        density
    }

    pub fn hptp_mandatory_count(&self) -> usize {
        self.records_by_name
            .values()
            .filter(|trn| trn.is_hptp_mandatory())
            .count()
    }
}

impl Default for CrsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn google_measurements() -> Vec<RawValue> {
        vec![
            RawValue::Pattern("corporate".into()),
            RawValue::Pattern("public".into()),
            RawValue::Numeric(0.5),
            RawValue::Pattern("cloud".into()),
            RawValue::Pattern("website".into()),
            RawValue::Pattern("text".into()),
            RawValue::Pattern("both".into()),
            RawValue::Numeric(0.9),
            RawValue::Numeric(200.0),
            RawValue::Pattern("none".into()),
            RawValue::Numeric(100.0),
            RawValue::Pattern("http".into()),
            RawValue::Numeric(1998.0),
            RawValue::Numeric(99.99),
            RawValue::Pattern("current".into()),
            RawValue::Numeric(200.0),
            RawValue::Pattern("accepts".into()),
            RawValue::Numeric(20.0),
            RawValue::Numeric(4.0),
            RawValue::Pattern("free".into()),
            RawValue::Pattern("unicast".into()),
            RawValue::Pattern("through".into()),
            RawValue::Pattern("poll".into()),
            RawValue::Numeric(3600.0),
            RawValue::Numeric(0.95),
            RawValue::Numeric(30.0),
            RawValue::Pattern("soc2".into()),
        ]
    }

    fn pptpro_measurements() -> Vec<RawValue> {
        vec![
            RawValue::Pattern("corporate".into()),
            RawValue::Pattern("public".into()),
            RawValue::Numeric(0.9),
            RawValue::Pattern("cloud".into()),
            RawValue::Pattern("app".into()),
            RawValue::Pattern("live".into()),
            RawValue::Pattern("both".into()),
            RawValue::Numeric(0.95),
            RawValue::Numeric(401.0),
            RawValue::Pattern("password".into()),
            RawValue::Numeric(3.0),
            RawValue::Pattern("websocket".into()),
            RawValue::Numeric(2024.0),
            RawValue::Numeric(99.99),
            RawValue::Pattern("live".into()),
            RawValue::Numeric(5.0),
            RawValue::Pattern("no".into()),
            RawValue::Numeric(3.0),
            RawValue::Numeric(2.0),
            RawValue::Pattern("free".into()),
            RawValue::Pattern("multicast".into()),
            RawValue::Pattern("out".into()),
            RawValue::Pattern("push".into()),
            RawValue::Numeric(999999.0),
            RawValue::Numeric(0.95),
            RawValue::Numeric(0.0),
            RawValue::Pattern("self-certified".into()),
        ]
    }

    #[test]
    fn register_and_resolve() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        let result = crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        match result {
            RegistrationResult::Ok { trn, address } => {
                assert_eq!(trn.name, "google.plm");
                assert!(!address.is_hptp_mandatory());

                let resolved = crs.resolve("google.plm").unwrap();
                assert_eq!(resolved.address, address);

                let name = crs.reverse_lookup(&address).unwrap();
                assert_eq!(name, "google.plm");
            }
            other => panic!("expected Ok, got {:?}", other),
        }

        assert_eq!(crs.entity_count(), 1);
    }

    #[test]
    fn hptp_mandatory_requires_sync() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        let result = crs.register(
            "pptpro.capomastro.plm".into(),
            "capomastro.plm".into(),
            vec![0xBE, 0xEF],
            pptpro_measurements(),
            now,
            None,
        );

        match result {
            RegistrationResult::HptpSyncRequired { address, .. } => {
                assert!(address.is_hptp_mandatory());
            }
            other => panic!("expected HptpSyncRequired, got {:?}", other),
        }
    }

    #[test]
    fn hptp_mandatory_with_sync() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        let result = crs.register(
            "pptpro.capomastro.plm".into(),
            "capomastro.plm".into(),
            vec![0xBE, 0xEF],
            pptpro_measurements(),
            now,
            Some(500),
        );

        match result {
            RegistrationResult::Ok { trn, .. } => {
                assert!(trn.is_hptp_mandatory());
                assert!(trn.is_hptp_synced());
            }
            other => panic!("expected Ok, got {:?}", other),
        }
    }

    #[test]
    fn rescan_no_drift() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        let result = crs.rescan("google.plm", google_measurements(), now + 1000).unwrap();
        match result {
            VerificationResult::Verified { name, .. } => {
                assert_eq!(name, "google.plm");
            }
            other => panic!("expected Verified, got {:?}", other),
        }

        assert!(crs.drift_log().is_empty());
    }

    #[test]
    fn rescan_with_drift() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        let old_addr = crs.resolve_addr("google.plm").unwrap();

        let mut changed = google_measurements();
        changed[19] = RawValue::Pattern("subscription".into());

        let later = now + 1_000_000_000;
        let result = crs.rescan("google.plm", changed, later).unwrap();

        match result {
            VerificationResult::Drifted {
                old_address,
                new_address,
                changed_dims,
                ..
            } => {
                assert_eq!(old_address, old_addr);
                assert_ne!(old_address, new_address);
                assert!(changed_dims.contains(&19));

                let redirect = crs.check_redirect(&old_address, later);
                assert_eq!(redirect, Some(new_address));
            }
            other => panic!("expected Drifted, got {:?}", other),
        }

        assert_eq!(crs.drift_log().len(), 1);
    }

    #[test]
    fn deregister() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        assert_eq!(crs.entity_count(), 1);
        let trn = crs.deregister("google.plm").unwrap();
        assert_eq!(trn.name, "google.plm");
        assert_eq!(crs.entity_count(), 0);
        assert!(crs.resolve("google.plm").is_none());
    }

    #[test]
    fn neighbor_maps_populated_on_registration() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );
        crs.register(
            "pptpro.capomastro.plm".into(),
            "capomastro.plm".into(),
            vec![0xBE, 0xEF],
            pptpro_measurements(),
            now,
            Some(100),
        );

        let g_addr = crs.resolve_addr("google.plm").unwrap();
        let p_addr = crs.resolve_addr("pptpro.capomastro.plm").unwrap();

        let g_map = crs.neighbor_map(&g_addr).unwrap();
        assert!(!g_map.is_empty());

        let p_map = crs.neighbor_map(&p_addr).unwrap();
        assert!(!p_map.is_empty());
    }

    #[test]
    fn dimension_density() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        let density = crs.dimension_density();
        assert_eq!(density.len(), DIMENSIONS);

        for dim_density in &density {
            let total: usize = dim_density.iter().sum();
            assert_eq!(total, 1);
        }
    }

    #[test]
    fn expired_redirect_cleanup() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        let mut changed = google_measurements();
        changed[19] = RawValue::Pattern("subscription".into());
        crs.rescan("google.plm", changed, now + 1000);

        assert_eq!(crs.expired_redirects(now + 1000).len(), 0);

        let after_grace = now + 1000 + crs.config.drift_grace_period_ns + 1;
        assert_eq!(crs.expired_redirects(after_grace).len(), 1);

        let purged = crs.purge_expired_redirects(after_grace);
        assert_eq!(purged, 1);
    }

    #[test]
    fn entities_due_for_rescan() {
        let mut crs = CrsRegistry::new();
        let now = 1_000_000_000u64;

        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            now,
            None,
        );

        let due = crs.entities_due_for_rescan(now + 1000);
        assert!(due.is_empty());

        let after_interval = now + crs.config.rescan_interval_ns + 1;
        let due = crs.entities_due_for_rescan(after_interval);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0], "google.plm");
    }
}
