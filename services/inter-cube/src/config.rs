// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # PlenumNET Feature Flags Configuration
//!
//! Per-feature runtime toggles for safe rollout and immediate rollback
//! of hardened wire-format changes and cryptographic enhancements.
//!
//! ## Design
//!
//! Each major feature introduced by SPEC-2026-NEXT has its own boolean flag.
//! Features can be enabled per-node for testing, then globally. If a feature
//! causes issues, flip the flag and the node reverts to legacy behavior
//! without a code deploy.
//!
//! ## Environment Variables
//!
//! All flags load from environment variables with sensible defaults.
//! The naming convention is `PLENUM_<FEATURE>`:
//!
//! | Variable | Default | Controls |
//! |----------|---------|----------|
//! | `PLENUM_REQUIRE_SIGNATURE` | `false` | T-06: Reject unsigned CRS registrations |
//! | `PLENUM_ENABLE_DUAL_CHECKSUM` | `false` | T-10: mod-364 + mod-333 wire checksum |
//! | `PLENUM_ENABLE_WIRE_ECC` | `false` | T-17: 8-trit ECC syndrome on addresses |
//! | `PLENUM_ENABLE_SPONGE_SHUFFLES` | `false` | T-18: σ_A–σ_D block permutations |
//! | `PLENUM_PROTOCOL_VERSION` | `2` | T-01: Wire protocol version to emit |
//! | `PLENUM_PROTOCOL_VERSION_MIN` | `1` | Minimum version to accept (dual-accept) |
//! | `PLENUM_ENABLE_RATE_LIMIT` | `false` | T-11: CRS registration rate limiting |
//! | `PLENUM_POW_K` | `5` | T-11: Proof-of-work leading zero trits |
//! | `PLENUM_V25_GRACE_DAYS` | `14` | T-09: Days to warn before hard-rejecting v2.5 |
//!
//! ## Usage
//!
//! ```rust
//! use inter_cube::config::PlenumConfig;
//!
//! let config = PlenumConfig::from_env();
//! if config.require_signature {
//!     // Verify TL-DSA signature on CRS registration
//! }
//! ```
//!
//! ## Created by T-05 (SPEC-2026-NEXT)

use std::collections::HashMap;
use std::env;

use plenumlan::cube::projection::SlotAddress;
use plenumlan::cube::port::port_to_slot;

// ═══════════════════════════════════════════════════════════════════════
// CONFIGURATION STRUCT
// ═══════════════════════════════════════════════════════════════════════

/// Runtime feature flags for PlenumNET hardening features.
///
/// Each flag gates one or more tasks from SPEC-2026-NEXT.
/// All flags default to `false` (legacy behavior) or conservative values,
/// ensuring a safe rollout path: deploy code first, enable features second.
#[derive(Debug, Clone)]
pub struct PlenumConfig {
    // ── Cryptographic Hardening (Phase 1) ───────────────────────

    /// T-06: Require TL-DSA signature on CRS registrations.
    ///
    /// When `true`, CRS rejects unsigned registrations with `CrsError::InvalidSignature`.
    /// When `false`, unsigned registrations are accepted (legacy behavior).
    ///
    /// Rollback: Set to `false` to accept unsigned registrations again.
    /// Tunnels built during the signed period remain valid.
    pub require_signature: bool,

    /// T-11: Enable rate limiting on CRS registration endpoint.
    ///
    /// When `true`, per-IP rate caps and proof-of-work are enforced.
    pub enable_rate_limit: bool,

    /// T-11: Proof-of-work difficulty for CRS registration.
    ///
    /// Number of leading zero trits required in `TIS-27(address ‖ nonce)`.
    /// Defaults: K=5 bootstrap (<1K nodes), K=8 steady-state, K=10 under load.
    pub pow_k: u8,

    /// T-09: v2.5 grace period in days.
    ///
    /// During this window, v2.5 fallback attempts are logged as warnings.
    /// After the window expires, v2.5 is hard-rejected.
    pub v25_grace_days: u32,

    // ── Wire Format Extensions (Phase 2) ────────────────────────

    /// T-10: Enable dual checksum (mod-364 + mod-333) on wire addresses.
    ///
    /// When `true`, 12 checksum trits are appended to 54-trit addresses.
    /// When `false`, legacy single-checksum (or no checksum) behavior.
    pub enable_dual_checksum: bool,

    /// T-17: Enable 8-trit ECC syndrome on wire addresses.
    ///
    /// When `true`, an 8-trit syndrome is appended for error correction.
    /// When `false`, errors are detected-or-rejected (no correction).
    pub enable_wire_ecc: bool,

    /// T-18: Enable σ_A–σ_D block shuffles in sponge rounds.
    ///
    /// When `true`, round-dependent block permutations are applied before stride-13.
    /// When `false`, sponge uses identity permutation (backward compatible).
    pub enable_sponge_shuffles: bool,

    // ── Protocol Version ────────────────────────────────────────

    /// Protocol version to emit in wire message headers.
    ///
    /// Set to 3 for Array3 Node Cluster format. Set to 2 for V2 hardened.
    /// This controls what VERSION byte goes into outgoing messages.
    pub protocol_version: u8,

    /// Minimum protocol version to accept from peers.
    ///
    /// During dual-acceptance period: 2 (accept V2 and V3).
    /// After full V3 rollout: 3 (reject V2 peers).
    pub protocol_version_min: u8,

    // ── Array3 Node Cluster (V3) ──────────────────────────────

    /// T-35: Enable Array3 slot addressing in wire messages.
    ///
    /// When `true`, V3 message types (0x60-0x6F) are emitted and accepted.
    /// When `false`, slot operations are disabled (V2 compatibility mode).
    pub enable_slot_addressing: bool,

    /// T-35: Enable key freshness zone reporting in heartbeats.
    ///
    /// When `true`, heartbeat payloads include the key's freshness zone
    /// (fresh/active/aging) and birth epoch.
    pub enable_key_freshness: bool,

    /// T-35: Node ID within Array3 cluster (Rep C {1,2,3}, 0=unset).
    ///
    /// Read from CUBE_NODE_ID env var. Node 1 = cluster gateway.
    /// Zero-sentinel: 0 means unconfigured (single-node mode).
    pub cube_node_id: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// DEFAULTS
// ═══════════════════════════════════════════════════════════════════════

impl Default for PlenumConfig {
    /// Conservative defaults: all features OFF, legacy behavior preserved.
    ///
    /// This means a fresh deploy with no env vars behaves identically
    /// to the pre-hardening code. Features are opted into explicitly.
    fn default() -> Self {
        PlenumConfig {
            require_signature: false,
            enable_rate_limit: false,
            pow_k: 5,
            v25_grace_days: 14,
            enable_dual_checksum: false,
            enable_wire_ecc: false,
            enable_sponge_shuffles: false,
            protocol_version: 3,
            protocol_version_min: 2,
            enable_slot_addressing: false,
            enable_key_freshness: false,
            cube_node_id: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ENVIRONMENT LOADING
// ═══════════════════════════════════════════════════════════════════════

impl PlenumConfig {
    /// Load configuration from environment variables.
    ///
    /// Each flag is read from its corresponding `PLENUM_*` env var.
    /// Missing or unparseable values fall back to defaults.
    pub fn from_env() -> Self {
        let mut config = PlenumConfig::default();

        config.require_signature = env_bool("PLENUM_REQUIRE_SIGNATURE", config.require_signature);
        config.enable_rate_limit = env_bool("PLENUM_ENABLE_RATE_LIMIT", config.enable_rate_limit);
        config.pow_k = env_u8("PLENUM_POW_K", config.pow_k);
        config.v25_grace_days = env_u32("PLENUM_V25_GRACE_DAYS", config.v25_grace_days);
        config.enable_dual_checksum = env_bool("PLENUM_ENABLE_DUAL_CHECKSUM", config.enable_dual_checksum);
        config.enable_wire_ecc = env_bool("PLENUM_ENABLE_WIRE_ECC", config.enable_wire_ecc);
        config.enable_sponge_shuffles = env_bool("PLENUM_ENABLE_SPONGE_SHUFFLES", config.enable_sponge_shuffles);
        config.protocol_version = env_u8("PLENUM_PROTOCOL_VERSION", config.protocol_version);
        config.protocol_version_min = env_u8("PLENUM_PROTOCOL_VERSION_MIN", config.protocol_version_min);
        config.enable_slot_addressing = env_bool("PLENUM_ENABLE_SLOT_ADDRESSING", config.enable_slot_addressing);
        config.enable_key_freshness = env_bool("PLENUM_ENABLE_KEY_FRESHNESS", config.enable_key_freshness);
        config.cube_node_id = env_u8("CUBE_NODE_ID", config.cube_node_id);

        config
    }

    /// Create a config with all features ENABLED.
    ///
    /// Useful for testing the fully-hardened path.
    pub fn all_enabled() -> Self {
        PlenumConfig {
            require_signature: true,
            enable_rate_limit: true,
            pow_k: 8,
            v25_grace_days: 14,
            enable_dual_checksum: true,
            enable_wire_ecc: true,
            enable_sponge_shuffles: true,
            protocol_version: 3,
            protocol_version_min: 3,
            enable_slot_addressing: true,
            enable_key_freshness: true,
            cube_node_id: 1,
        }
    }

    /// Check if the protocol version configuration is consistent.
    ///
    /// `protocol_version_min` must not exceed `protocol_version`.
    pub fn is_valid(&self) -> bool {
        self.protocol_version_min <= self.protocol_version
            && self.protocol_version >= 1
            && self.pow_k <= 27
    }

    /// Log the current configuration for startup diagnostics.
    pub fn log_startup(&self) {
        println!("=== PlenumConfig ===");
        println!("  require_signature:     {}", self.require_signature);
        println!("  enable_rate_limit:     {}", self.enable_rate_limit);
        println!("  pow_k:                 {}", self.pow_k);
        println!("  v25_grace_days:        {}", self.v25_grace_days);
        println!("  enable_dual_checksum:  {}", self.enable_dual_checksum);
        println!("  enable_wire_ecc:       {}", self.enable_wire_ecc);
        println!("  enable_sponge_shuffles:{}", self.enable_sponge_shuffles);
        println!("  protocol_version:      v{}", self.protocol_version);
        println!("  protocol_version_min:  v{}", self.protocol_version_min);
        println!("  enable_slot_addressing:{}", self.enable_slot_addressing);
        println!("  enable_key_freshness:  {}", self.enable_key_freshness);
        println!("  cube_node_id:          {}", self.cube_node_id);
        println!("====================");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ENV PARSING HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Read a boolean from an environment variable.
///
/// Accepts "true", "1", "yes" (case-insensitive) as true.
/// Everything else (including missing) returns the default.
pub fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(val) => matches!(val.to_lowercase().as_str(), "true" | "1" | "yes"),
        Err(_) => default,
    }
}

/// Read a u64 from an environment variable.
pub fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Read a u8 from an environment variable.
fn env_u8(key: &str, default: u8) -> u8 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Read a u32 from an environment variable.
fn env_u32(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════
// DAEMON CONFIG — Slot inventory, auth, and probing settings
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub cube_node_id: u8,
    pub is_gateway: bool,
    pub api_key: Option<String>,
    pub slots_auth_required: bool,
    pub enable_rate_limit: bool,
    pub bind_addr: String,
    pub slot_probe_timeout_ms: u64,
    pub slot_registry: HashMap<SlotAddress, String>,
}

pub const GATEWAY_NODE_ID: u8 = 1;

impl Default for DaemonConfig {
    fn default() -> Self {
        DaemonConfig {
            cube_node_id: GATEWAY_NODE_ID,
            is_gateway: true,
            api_key: None,
            slots_auth_required: false,
            enable_rate_limit: false,
            bind_addr: "127.0.0.1".to_string(),
            slot_probe_timeout_ms: 500,
            slot_registry: HashMap::new(),
        }
    }
}

fn is_valid_service_type(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 {
        return false;
    }
    !value.chars().any(|c| c.is_control() || c == '<' || c == '>' || c == '&' || c == '"' || c == '\'')
}

fn parse_trit_address(key: &str) -> Option<SlotAddress> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let vals: Vec<u8> = parts.iter().filter_map(|p| p.parse::<u8>().ok()).collect();
    if vals.len() != 3 {
        return None;
    }
    if vals.iter().all(|&v| v >= 1 && v <= 3) {
        Some(SlotAddress::new(vals[0], vals[1], vals[2]))
    } else {
        None
    }
}

pub fn load_slot_registry(cube_node_id: u8) -> HashMap<SlotAddress, String> {
    let json_str = env::var("PLENUM_SLOT_REGISTRY")
        .ok()
        .or_else(|| {
            env::var("PLENUM_SLOT_REGISTRY_FILE")
                .ok()
                .and_then(|path| std::fs::read_to_string(&path).ok())
        });

    let json_str = match json_str {
        Some(s) if !s.trim().is_empty() => s,
        _ => return HashMap::new(),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[SLOTS] WARNING: Failed to parse PLENUM_SLOT_REGISTRY JSON: {}. All slots will have service_type: null", e);
            return HashMap::new();
        }
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => {
            eprintln!("[SLOTS] WARNING: PLENUM_SLOT_REGISTRY is not a JSON object. All slots will have service_type: null");
            return HashMap::new();
        }
    };

    let mut registry = HashMap::new();
    let mut seen_keys: HashMap<SlotAddress, String> = HashMap::new();

    for (key, value) in obj {
        let value_str = match value.as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!("[SLOTS] WARNING: Registry key '{}' has non-string value — skipping", key);
                continue;
            }
        };

        if !is_valid_service_type(&value_str) {
            eprintln!(
                "[SLOTS] WARNING: Registry key '{}' has invalid service_type '{}' (must be 1-64 chars, no control/HTML chars) — skipping",
                key, value_str
            );
            continue;
        }

        let slot_addr = if let Some(port) = key.parse::<u16>().ok() {
            match port_to_slot(port) {
                Some((node_id, slot)) => {
                    if node_id != cube_node_id {
                        eprintln!(
                            "[SLOTS] WARNING: Port key '{}' maps to node {} but this daemon is node {} — skipping",
                            key, node_id, cube_node_id
                        );
                        continue;
                    }
                    slot
                }
                None => {
                    eprintln!("[SLOTS] WARNING: Port key '{}' is outside the Array3 port range — skipping", key);
                    continue;
                }
            }
        } else if let Some(slot) = parse_trit_address(key) {
            slot
        } else {
            eprintln!("[SLOTS] WARNING: Registry key '{}' is neither a valid port nor a P.R.I trit address ([1-3].[1-3].[1-3]) — skipping", key);
            continue;
        };

        if let Some(prev_key) = seen_keys.get(&slot_addr) {
            eprintln!(
                "[SLOTS] WARNING: Slot {}.{}.{} registered via both key '{}' and key '{}' — last value wins: '{}'",
                slot_addr.plane, slot_addr.role, slot_addr.instance,
                prev_key, key, value_str
            );
        }
        seen_keys.insert(slot_addr, key.clone());
        registry.insert(slot_addr, value_str);
    }

    // Future enhancement: hot-reload registry without daemon restart.
    // Alternative registration models (future work):
    //   - Push-model: services self-register at startup via a local API
    //   - Pull-model: daemon probes ports and discovers services automatically

    registry
}

impl DaemonConfig {
    pub fn from_env() -> Self {
        let cube_node_id = env_u8("CUBE_NODE_ID", 1);
        let is_gateway = cube_node_id == GATEWAY_NODE_ID;
        let api_key = env::var("PLENUM_API_KEY").ok().filter(|s| !s.is_empty());
        let bind_addr = env::var("PLENUM_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
        let enable_rate_limit = env_bool("PLENUM_ENABLE_RATE_LIMIT", false);

        if api_key.is_none() && bind_addr != "127.0.0.1" {
            eprintln!(
                "[SLOTS] WARNING: Daemon bound to {} without API key — slots endpoint is unauthenticated on a non-localhost interface. Set PLENUM_API_KEY to secure.",
                bind_addr
            );
        }

        let slots_auth_required = env_bool("PLENUM_SLOTS_AUTH_REQUIRED", false);

        if !slots_auth_required {
            eprintln!("[SLOTS] Slots endpoint serving unauthenticated traffic — set PLENUM_SLOTS_AUTH_REQUIRED=true and PLENUM_API_KEY to enforce auth");
        }

        DaemonConfig {
            cube_node_id,
            is_gateway,
            api_key,
            slots_auth_required,
            enable_rate_limit,
            bind_addr,
            slot_probe_timeout_ms: env_u64("PLENUM_SLOT_PROBE_TIMEOUT_MS", 500),
            slot_registry: load_slot_registry(cube_node_id),
        }
    }

    pub fn log_startup(&self) {
        println!("=== DaemonConfig ===");
        println!("  cube_node_id:          {}", self.cube_node_id);
        println!("  is_gateway:            {}", self.is_gateway);
        println!("  api_key:               {}", if self.api_key.is_some() { "set" } else { "unset" });
        println!("  slots_auth_required:   {}", self.slots_auth_required);
        println!("  enable_rate_limit:     {}", self.enable_rate_limit);
        println!("  bind_addr:             {}", self.bind_addr);
        println!("  slot_probe_timeout_ms: {}", self.slot_probe_timeout_ms);
        println!("  slot_registry entries: {}", self.slot_registry.len());
        println!("====================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_all_features_off() {
        let config = PlenumConfig::default();
        assert!(!config.require_signature);
        assert!(!config.enable_rate_limit);
        assert!(!config.enable_dual_checksum);
        assert!(!config.enable_wire_ecc);
        assert!(!config.enable_sponge_shuffles);
        assert!(!config.enable_slot_addressing);
        assert!(!config.enable_key_freshness);
        assert_eq!(config.cube_node_id, 0);
    }

    #[test]
    fn test_default_config_protocol_versions() {
        let config = PlenumConfig::default();
        assert_eq!(config.protocol_version, 3);
        assert_eq!(config.protocol_version_min, 2);
    }

    #[test]
    fn test_default_config_is_valid() {
        assert!(PlenumConfig::default().is_valid());
    }

    #[test]
    fn test_all_enabled_config() {
        let config = PlenumConfig::all_enabled();
        assert!(config.require_signature);
        assert!(config.enable_rate_limit);
        assert!(config.enable_dual_checksum);
        assert!(config.enable_wire_ecc);
        assert!(config.enable_sponge_shuffles);
        assert!(config.enable_slot_addressing);
        assert!(config.enable_key_freshness);
        assert_eq!(config.protocol_version, 3);
        assert_eq!(config.protocol_version_min, 3);
        assert_eq!(config.cube_node_id, 1);
    }

    #[test]
    fn test_all_enabled_is_valid() {
        assert!(PlenumConfig::all_enabled().is_valid());
    }

    #[test]
    fn test_invalid_config_min_exceeds_current() {
        let config = PlenumConfig {
            protocol_version: 1,
            protocol_version_min: 2,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn test_invalid_config_pow_k_too_high() {
        let config = PlenumConfig {
            pow_k: 28,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn test_from_env_defaults() {
        let config = PlenumConfig::from_env();
        assert!(config.is_valid());
    }

    #[test]
    fn test_env_bool_parsing() {
        assert_eq!(env_bool("NONEXISTENT_VAR_12345", false), false);
        assert_eq!(env_bool("NONEXISTENT_VAR_12345", true), true);
    }

    #[test]
    fn test_env_u8_parsing() {
        assert_eq!(env_u8("NONEXISTENT_VAR_12345", 42), 42);
    }

    #[test]
    fn test_env_u32_parsing() {
        assert_eq!(env_u32("NONEXISTENT_VAR_12345", 365), 365);
    }

    #[test]
    fn test_default_pow_k() {
        let config = PlenumConfig::default();
        assert_eq!(config.pow_k, 5, "Default PoW K should be 5 (bootstrap mode)");
    }

    #[test]
    fn test_default_v25_grace() {
        let config = PlenumConfig::default();
        assert_eq!(config.v25_grace_days, 14, "Default v2.5 grace should be 14 days");
    }

    #[test]
    fn test_daemon_config_defaults() {
        let config = DaemonConfig::default();
        assert!(config.api_key.is_none());
        assert!(!config.slots_auth_required);
        assert_eq!(config.bind_addr, "127.0.0.1");
        assert_eq!(config.slot_probe_timeout_ms, 500);
        assert!(config.slot_registry.is_empty());
    }

    #[test]
    fn test_parse_trit_address_valid() {
        let slot = parse_trit_address("1.2.3").unwrap();
        assert_eq!(slot.plane, 1);
        assert_eq!(slot.role, 2);
        assert_eq!(slot.instance, 3);

        let center = parse_trit_address("2.2.2").unwrap();
        assert_eq!(center.plane, 2);
        assert_eq!(center.role, 2);
        assert_eq!(center.instance, 2);
    }

    #[test]
    fn test_parse_trit_address_invalid_range() {
        assert!(parse_trit_address("4.1.1").is_none());
        assert!(parse_trit_address("0.2.3").is_none());
        assert!(parse_trit_address("1.0.1").is_none());
        assert!(parse_trit_address("1.2.4").is_none());
    }

    #[test]
    fn test_parse_trit_address_invalid_format() {
        assert!(parse_trit_address("abc").is_none());
        assert!(parse_trit_address("1.2").is_none());
        assert!(parse_trit_address("1.2.3.4").is_none());
        assert!(parse_trit_address("").is_none());
        assert!(parse_trit_address("1..3").is_none());
    }

    #[test]
    fn test_service_type_validation_valid() {
        assert!(is_valid_service_type("gateway"));
        assert!(is_valid_service_type("yoda"));
        assert!(is_valid_service_type("llm-inference"));
        assert!(is_valid_service_type("a"));
    }

    #[test]
    fn test_service_type_validation_too_long() {
        let long = "a".repeat(65);
        assert!(!is_valid_service_type(&long));
        let exact = "a".repeat(64);
        assert!(is_valid_service_type(&exact));
    }

    #[test]
    fn test_service_type_validation_empty() {
        assert!(!is_valid_service_type(""));
    }

    #[test]
    fn test_service_type_validation_control_chars() {
        assert!(!is_valid_service_type("gate\x00way"));
        assert!(!is_valid_service_type("gate\nway"));
        assert!(!is_valid_service_type("gate\tway"));
    }

    #[test]
    fn test_service_type_validation_html_injection() {
        assert!(!is_valid_service_type("<script>"));
        assert!(!is_valid_service_type("foo&bar"));
        assert!(!is_valid_service_type("a\"b"));
        assert!(!is_valid_service_type("a'b"));
        assert!(!is_valid_service_type("a>b"));
    }

    #[test]
    fn test_load_registry_empty_env() {
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
        std::env::remove_var("PLENUM_SLOT_REGISTRY_FILE");
        let registry = load_slot_registry(1);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_load_registry_trit_keys() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", r#"{"2.2.2": "gateway", "1.1.1": "yoda"}"#);
        let registry = load_slot_registry(1);
        assert_eq!(registry.get(&SlotAddress::new(2, 2, 2)), Some(&"gateway".to_string()));
        assert_eq!(registry.get(&SlotAddress::new(1, 1, 1)), Some(&"yoda".to_string()));
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_port_keys() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", r#"{"11111": "yoda"}"#);
        let registry = load_slot_registry(1);
        assert_eq!(registry.get(&SlotAddress::new(1, 1, 1)), Some(&"yoda".to_string()));
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_invalid_trit_address_skipped() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", r#"{"4.1.1": "bad", "1.1.1": "good"}"#);
        let registry = load_slot_registry(1);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get(&SlotAddress::new(1, 1, 1)), Some(&"good".to_string()));
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_cross_node_port_rejected() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", r#"{"11138": "bad"}"#);
        let registry = load_slot_registry(1);
        assert!(registry.is_empty());
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_invalid_value_skipped() {
        let long_val = "a".repeat(65);
        let json = format!(r#"{{"1.1.1": "{}"}}"#, long_val);
        std::env::set_var("PLENUM_SLOT_REGISTRY", &json);
        let registry = load_slot_registry(1);
        assert!(registry.is_empty());
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_malformed_json() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", "not json at all");
        let registry = load_slot_registry(1);
        assert!(registry.is_empty());
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }

    #[test]
    fn test_load_registry_collision_both_resolve_to_same_slot() {
        std::env::set_var("PLENUM_SLOT_REGISTRY", r#"{"1.1.1": "first", "11111": "second"}"#);
        let registry = load_slot_registry(1);
        assert_eq!(registry.len(), 1);
        let val = registry.get(&SlotAddress::new(1, 1, 1)).unwrap();
        assert!(
            val == "first" || val == "second",
            "Collision should keep exactly one entry; JSON object iteration order determines winner"
        );
        std::env::remove_var("PLENUM_SLOT_REGISTRY");
    }
}
