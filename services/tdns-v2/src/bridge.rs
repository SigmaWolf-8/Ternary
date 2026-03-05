// TDNS v2.3 — Metatronic Bridge
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The resolver gateway between TDNS and legacy DNS.
//
// `.plm` → TDNS resolution (27-trit address via CRS lookup)
// Everything else → legacy DNS (pass-through)
//
// Two worlds, one resolver.
//
// §12.5: .plm TLD signals the metatronic bridge to route through
// TDNS rather than legacy DNS.

use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::{Arc, RwLock};

use crate::addr::CubeAddr;
use crate::crs::CrsRegistry;

// ─── Constants ───────────────────────────────────────────────────────────────

/// The PlenumNET TLD.
pub const PLM_TLD: &str = ".plm";

// ─── Resolution Result ───────────────────────────────────────────────────────

/// The result of a metatronic bridge resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// TDNS resolution: name resolved to a 27-trit address.
    Tdns {
        name: String,
        address: CubeAddr,
        hptp_mandatory: bool,
        zone: String,
    },

    /// Legacy DNS resolution: name resolved to IP address(es).
    Legacy {
        name: String,
        addresses: Vec<String>,
    },

    /// TDNS redirect: address has drifted, follow the new address.
    TdnsRedirect {
        name: String,
        old_address: CubeAddr,
        new_address: CubeAddr,
    },

    /// Resolution failed.
    Failed {
        name: String,
        reason: String,
    },
}

// ─── Metatronic Bridge ───────────────────────────────────────────────────────

/// The metatronic bridge: routes `.plm` names to TDNS, everything else
/// to legacy DNS.
///
/// Thread-safe via `Arc<RwLock<CrsRegistry>>`. Multiple bridge instances
/// (or multiple threads) can share the same CRS registry. Multi-node
/// deployments each hold their own registry synced via CRS API.
pub struct Bridge {
    /// Shared reference to the CRS registry for TDNS lookups.
    crs: Arc<RwLock<CrsRegistry>>,
    /// Cache for legacy DNS results (name → IPs, with TTL).
    legacy_cache: HashMap<String, (Vec<String>, u64)>,
    /// Cache TTL for legacy results (nanoseconds).
    legacy_cache_ttl_ns: u64,
    /// Total TDNS resolutions.
    tdns_count: u64,
    /// Total legacy DNS resolutions.
    legacy_count: u64,
    /// Total failed resolutions.
    failed_count: u64,
}

impl Bridge {
    /// Create a bridge backed by a shared CRS registry.
    pub fn new(crs: Arc<RwLock<CrsRegistry>>) -> Self {
        Self {
            crs,
            legacy_cache: HashMap::new(),
            legacy_cache_ttl_ns: 60_000_000_000, // 60 seconds
            tdns_count: 0,
            legacy_count: 0,
            failed_count: 0,
        }
    }

    /// Resolve a name — routes to TDNS or legacy DNS based on TLD.
    pub fn resolve(&mut self, name: &str, now_ns: u64) -> Resolution {
        let normalized = name.trim().to_lowercase();

        if is_plm_name(&normalized) {
            self.resolve_tdns(&normalized, now_ns)
        } else {
            self.resolve_legacy(&normalized, now_ns)
        }
    }

    // ── TDNS Resolution ─────────────────────────────────────────────

    fn resolve_tdns(&mut self, name: &str, now_ns: u64) -> Resolution {
        let crs = match self.crs.read() {
            Ok(guard) => guard,
            Err(_) => {
                self.failed_count += 1;
                return Resolution::Failed {
                    name: name.to_string(),
                    reason: "CRS registry lock poisoned".into(),
                };
            }
        };

        // Look up TRN record
        if let Some(trn) = crs.resolve(name) {
            // Check time-lock validity
            if !trn.is_valid_at(now_ns) {
                self.failed_count += 1;
                return Resolution::Failed {
                    name: name.to_string(),
                    reason: "name exists but is outside its validity window".into(),
                };
            }

            // Check for drift redirect
            if let Some(new_addr) = crs.check_redirect(&trn.address, now_ns) {
                self.tdns_count += 1;
                return Resolution::TdnsRedirect {
                    name: name.to_string(),
                    old_address: trn.address,
                    new_address: new_addr,
                };
            }

            self.tdns_count += 1;
            Resolution::Tdns {
                name: name.to_string(),
                address: trn.address,
                hptp_mandatory: trn.is_hptp_mandatory(),
                zone: trn.zone.clone(),
            }
        } else {
            self.failed_count += 1;
            Resolution::Failed {
                name: name.to_string(),
                reason: format!("name '{}' not found in TDNS", name),
            }
        }
    }

    // ── Legacy DNS Resolution ───────────────────────────────────────

    fn resolve_legacy(&mut self, name: &str, now_ns: u64) -> Resolution {
        // Check cache first
        if let Some((addresses, expires)) = self.legacy_cache.get(name) {
            if now_ns < *expires {
                self.legacy_count += 1;
                return Resolution::Legacy {
                    name: name.to_string(),
                    addresses: addresses.clone(),
                };
            }
        }

        // Resolve via system DNS
        let addr_str = format!("{}:443", name);
        match addr_str.to_socket_addrs() {
            Ok(addrs) => {
                let addresses: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                if addresses.is_empty() {
                    self.failed_count += 1;
                    return Resolution::Failed {
                        name: name.to_string(),
                        reason: "DNS resolution returned no addresses".into(),
                    };
                }

                // Cache the result
                let expires = now_ns + self.legacy_cache_ttl_ns;
                self.legacy_cache
                    .insert(name.to_string(), (addresses.clone(), expires));

                self.legacy_count += 1;
                Resolution::Legacy {
                    name: name.to_string(),
                    addresses,
                }
            }
            Err(e) => {
                self.failed_count += 1;
                Resolution::Failed {
                    name: name.to_string(),
                    reason: format!("DNS resolution failed: {}", e),
                }
            }
        }
    }

    // ── Cache Management ────────────────────────────────────────────

    /// Purge expired legacy DNS cache entries.
    pub fn purge_cache(&mut self, now_ns: u64) -> usize {
        let before = self.legacy_cache.len();
        self.legacy_cache.retain(|_, (_, expires)| now_ns < *expires);
        before - self.legacy_cache.len()
    }

    /// Clear the entire legacy DNS cache.
    pub fn clear_cache(&mut self) {
        self.legacy_cache.clear();
    }

    /// Set legacy cache TTL.
    pub fn set_cache_ttl(&mut self, ttl_ns: u64) {
        self.legacy_cache_ttl_ns = ttl_ns;
    }

    // ── Metrics ─────────────────────────────────────────────────────

    pub fn tdns_count(&self) -> u64 { self.tdns_count }
    pub fn legacy_count(&self) -> u64 { self.legacy_count }
    pub fn failed_count(&self) -> u64 { self.failed_count }
    pub fn cache_size(&self) -> usize { self.legacy_cache.len() }
}

// ─── Utility Functions ───────────────────────────────────────────────────────

/// Check if a name belongs to the PlenumNET TLD.
pub fn is_plm_name(name: &str) -> bool {
    name.ends_with(PLM_TLD)
}

/// Extract the zone from a .plm name.
pub fn extract_zone(name: &str) -> &str {
    if !is_plm_name(name) {
        return name;
    }
    let without_tld = &name[..name.len() - PLM_TLD.len()];
    match without_tld.rfind('.') {
        Some(pos) => &name[pos + 1..],
        None => "plm",
    }
}

/// Extract the label (first component) from a .plm name.
pub fn extract_label(name: &str) -> &str {
    match name.find('.') {
        Some(pos) => &name[..pos],
        None => name,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::RawValue;

    fn make_crs_with_google() -> Arc<RwLock<CrsRegistry>> {
        let mut crs = CrsRegistry::new();
        crs.register(
            "google.plm".into(),
            "plm".into(),
            vec![0xDE, 0xAD],
            google_measurements(),
            1000,
            None,
        );
        Arc::new(RwLock::new(crs))
    }

    fn empty_crs() -> Arc<RwLock<CrsRegistry>> {
        Arc::new(RwLock::new(CrsRegistry::new()))
    }

    #[test]
    fn is_plm_detection() {
        assert!(is_plm_name("google.plm"));
        assert!(is_plm_name("pptpro.capomastro.plm"));
        assert!(!is_plm_name("google.com"));
        assert!(!is_plm_name("example.plm.com"));
    }

    #[test]
    fn zone_extraction() {
        assert_eq!(extract_zone("pptpro.capomastro.plm"), "capomastro.plm");
        assert_eq!(extract_zone("google.plm"), "plm");
        assert_eq!(extract_zone("deep.nested.zone.plm"), "zone.plm");
    }

    #[test]
    fn label_extraction() {
        assert_eq!(extract_label("pptpro.capomastro.plm"), "pptpro");
        assert_eq!(extract_label("google.plm"), "google");
        assert_eq!(extract_label("standalone"), "standalone");
    }

    #[test]
    fn tdns_resolution() {
        let crs = make_crs_with_google();
        let g_addr = crs.read().unwrap().resolve_addr("google.plm").unwrap();

        let mut bridge = Bridge::new(crs);
        let result = bridge.resolve("google.plm", 2000);

        match result {
            Resolution::Tdns { name, address, .. } => {
                assert_eq!(name, "google.plm");
                assert_eq!(address, g_addr);
            }
            other => panic!("expected Tdns, got {:?}", other),
        }
        assert_eq!(bridge.tdns_count(), 1);
    }

    #[test]
    fn tdns_not_found() {
        let mut bridge = Bridge::new(empty_crs());
        let result = bridge.resolve("nonexistent.plm", 1000);

        match result {
            Resolution::Failed { name, .. } => assert_eq!(name, "nonexistent.plm"),
            other => panic!("expected Failed, got {:?}", other),
        }
        assert_eq!(bridge.failed_count(), 1);
    }

    /// This test makes a real DNS call — gated behind `network-tests` feature
    /// so it doesn't flake in Docker builds or sandboxed CI.
    #[test]
    #[cfg(feature = "network-tests")]
    fn legacy_dns_resolution() {
        let mut bridge = Bridge::new(empty_crs());
        let result = bridge.resolve("github.com", 1000);

        match result {
            Resolution::Legacy { name, addresses } => {
                assert_eq!(name, "github.com");
                assert!(!addresses.is_empty());
            }
            other => panic!("expected Legacy, got {:?}", other),
        }
    }

    #[test]
    fn legacy_cache_hit() {
        let mut bridge = Bridge::new(empty_crs());
        bridge.legacy_cache.insert(
            "cached.com".into(),
            (vec!["1.2.3.4".into()], 999_999_999_999),
        );

        let result = bridge.resolve("cached.com", 1000);
        match result {
            Resolution::Legacy { addresses, .. } => {
                assert_eq!(addresses, vec!["1.2.3.4".to_string()]);
            }
            other => panic!("expected cached Legacy, got {:?}", other),
        }
    }

    #[test]
    fn cache_purge() {
        let mut bridge = Bridge::new(empty_crs());
        bridge.legacy_cache.insert("fresh.com".into(), (vec!["1.1.1.1".into()], 5000));
        bridge.legacy_cache.insert("stale.com".into(), (vec!["2.2.2.2".into()], 1000));

        assert_eq!(bridge.cache_size(), 2);
        let purged = bridge.purge_cache(3000);
        assert_eq!(purged, 1);
        assert_eq!(bridge.cache_size(), 1);
    }

    #[test]
    fn routing_decision_plm_vs_legacy() {
        assert!(is_plm_name("pptpro.capomastro.plm"));
        assert!(!is_plm_name("pptpro.capomastro.com"));
        assert!(!is_plm_name("plm.example.com"));
    }

    fn google_measurements() -> Vec<RawValue> {
        vec![
            RawValue::Pattern("corporate".into()),
            RawValue::Pattern("public".into()),
            RawValue::Numeric(2.0),  // trit 3: 2 signals
            RawValue::Pattern("cloud".into()),
            RawValue::Pattern("website".into()),
            RawValue::Pattern("text".into()),
            RawValue::Pattern("both".into()),
            RawValue::Numeric(4.0),  // trit 8: 4 ML signals
            RawValue::Numeric(200.0),
            RawValue::Pattern("none".into()),
            RawValue::Numeric(100.0),
            RawValue::Pattern("http".into()),
            RawValue::Numeric(1998.0),
            RawValue::Numeric(99.99),
            RawValue::Pattern("current".into()),
            RawValue::Numeric(0.0),  // trit 16: 0 rt, 0 batch
            RawValue::Pattern("accepts".into()),
            RawValue::Numeric(20.0),
            RawValue::Numeric(4.0),
            RawValue::Pattern("free".into()),
            RawValue::Pattern("unicast".into()),
            RawValue::Pattern("through".into()),
            RawValue::Pattern("poll".into()),
            RawValue::Numeric(3600.0),
            RawValue::Numeric(3.0),  // trit 25: 3 signals
            RawValue::Numeric(30.0),
            RawValue::Pattern("soc2".into()),
        ]
    }
}