// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY AUDIT LOG — Task #27
//
// Merkle-chained audit log using TIS-27 (sponge::hash_hex()) directly.
// No SHA-3. No N-API. No migration — TIS-27 from day one.
//
// Reads existing v1 (SHA-3) entries in capability-audit.jsonl for chain
// continuity. Writes only v2 (TIS-27) entries. The first v2 entry's
// parent_event_hash references the last v1 entry's SHA-3 hash — the
// chain is continuous across the boundary.
//
// Sponge Context String Registry:
// | Context String                 | Usage                    | Module           |
// |-------------------------------|--------------------------|------------------|
// | "relay-audit-genesis"          | Genesis hash for empty chain | relay_audit.rs |

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ternary_math::sponge::hash_hex;

// ═══════════════════════════════════════════════════════════════════════
// AUDIT EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Relay audit event types. Superset of the retired TypeScript
/// RelayAuditEventType union from node-watchdog.ts:162-170.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelayAuditEventType {
    // ── Existing (from retired TS) ──────────────────────────
    #[serde(rename = "relay.auth_success")]
    AuthSuccess,
    #[serde(rename = "relay.auth_failure")]
    AuthFailure,
    #[serde(rename = "relay.disconnect")]
    Disconnect,
    #[serde(rename = "relay.reconnect")]
    Reconnect,
    #[serde(rename = "relay.error")]
    Error,
    #[serde(rename = "relay.circuit_breaker")]
    CircuitBreaker,
    #[serde(rename = "relay.go_away")]
    GoAway,
    #[serde(rename = "relay.peer_offline")]
    PeerOffline,
    // ── New (Task #27) ──────────────────────────────────────
    #[serde(rename = "relay.capability_negotiation")]
    CapabilityNegotiation,
    #[serde(rename = "relay.capability_enforcement")]
    CapabilityEnforcement,
    #[serde(rename = "relay.capability_downgrade")]
    CapabilityDowngrade,
    #[serde(rename = "relay.topic_subscribe")]
    TopicSubscribe,
    #[serde(rename = "relay.topic_publish")]
    TopicPublish,
    #[serde(rename = "relay.topic_reauth_failure")]
    TopicReauthFailure,
    #[serde(rename = "relay.topic_revoked")]
    TopicRevoked,
    #[serde(rename = "relay.topic_reset")]
    TopicReset,
    #[serde(rename = "relay.topic_lifecycle")]
    TopicLifecycle,
    #[serde(rename = "relay.tombstone")]
    Tombstone,
    #[serde(rename = "relay.resync_rate_limit")]
    ResyncRateLimit,
    #[serde(rename = "relay.heartbeat_failure")]
    HeartbeatFailure,
    #[serde(rename = "relay.heartbeat_interval_change")]
    HeartbeatIntervalChange,
    #[serde(rename = "relay.circuit_breaker_manual_reset")]
    CircuitBreakerManualReset,
}

/// Severity levels for Forma Codex forward-compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Warn,
    Error,
    Critical,
}

/// Subsystem classification for Forma Codex forward-compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSubsystem {
    Capability,
    Topic,
    Heartbeat,
    CircuitBreaker,
    Shutdown,
    Sequencing,
}

// ═══════════════════════════════════════════════════════════════════════
// AUDIT ENTRY
// ═══════════════════════════════════════════════════════════════════════

/// A single relay audit event for recording.
#[derive(Debug, Clone)]
pub struct RelayAuditEntry {
    pub event_type: RelayAuditEventType,
    pub address: String,
    pub timestamp: String,
    pub details: serde_json::Value,
    pub severity: AuditSeverity,
    pub subsystem: AuditSubsystem,
    /// Optional eventId references linking causally related events.
    pub correlation_refs: Vec<String>,
}

/// A persisted audit log entry (JSONL line format).
#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedAuditEntry {
    pub hash: String,
    pub parent_event_hash: String,
    #[serde(rename = "hashAlgorithm")]
    pub hash_algorithm: String,
    pub event: String,
    pub jti: String,
    pub ts: String,
    pub details: serde_json::Value,
    // ── Forma Codex forward-compatibility fields ────────────
    #[serde(rename = "eventId")]
    pub event_id: String,
    pub source_service: String,
    pub severity: AuditSeverity,
    pub subsystem: AuditSubsystem,
    #[serde(rename = "correlationRefs", skip_serializing_if = "Vec::is_empty")]
    pub correlation_refs: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// RELAY AUDIT LOG
// ═══════════════════════════════════════════════════════════════════════

/// TIS-27 Merkle-chained relay audit log.
///
/// - Hashes with `sponge::hash_hex()` directly — no SHA-3, no N-API
/// - Reads v1 (SHA-3) entries for chain continuity on startup
/// - Writes v2 (TIS-27) entries exclusively
/// - In-memory capability index for O(1) downgrade baseline lookup
pub struct RelayAuditLog {
    /// Merkle chain leaves (hashes of all entries).
    leaves: Vec<String>,
    /// Path to the JSONL audit log file.
    persist_path: PathBuf,
    /// Hash of the most recent entry (chain tip).
    last_event_hash: String,
    /// In-memory index: Rep C address → last negotiated capability set.
    /// Populated from audit log on startup (reverse scan).
    capability_index: HashMap<String, Vec<String>>,
    /// Total events recorded.
    event_count: u64,
}

impl RelayAuditLog {
    /// Create a new audit log backed by the given file.
    ///
    /// Genesis hash uses TIS-27: `hash_hex(b"relay-audit-genesis")`.
    /// On startup, loads existing entries (v1 or v2) to continue the chain.
    pub fn new(persist_path: PathBuf) -> Self {
        let genesis = hash_hex(b"relay-audit-genesis");
        let mut log = RelayAuditLog {
            leaves: Vec::new(),
            persist_path,
            last_event_hash: genesis,
            capability_index: HashMap::new(),
            event_count: 0,
        };
        log.load_from_disk();
        log
    }

    /// Create an in-memory-only log (for testing).
    pub fn in_memory() -> Self {
        let genesis = hash_hex(b"relay-audit-genesis");
        RelayAuditLog {
            leaves: Vec::new(),
            persist_path: PathBuf::new(),
            last_event_hash: genesis,
            capability_index: HashMap::new(),
            event_count: 0,
        }
    }

    /// Load existing entries from disk. Reads both v1 (SHA-3) and v2 (TIS-27)
    /// formats. The last entry's hash becomes the chain tip regardless of format.
    fn load_from_disk(&mut self) {
        if self.persist_path.as_os_str().is_empty() {
            return; // in-memory mode
        }
        if !self.persist_path.exists() {
            return;
        }

        let file = match fs::File::open(&self.persist_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[relay-audit] Failed to open audit log: {}", e);
                return;
            }
        };

        let reader = std::io::BufReader::new(file);
        let mut loaded = 0u64;
        let mut last_cap_entries: HashMap<String, Vec<String>> = HashMap::new();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(entry) => {
                    if let Some(hash) = entry.get("hash").and_then(|h| h.as_str()) {
                        self.leaves.push(hash.to_string());
                        self.last_event_hash = hash.to_string();
                        loaded += 1;
                    }

                    // Build capability index from negotiation events
                    if let Some(event) = entry.get("event").and_then(|e| e.as_str()) {
                        if event == "relay.capability_negotiation" {
                            if let Some(details) = entry.get("details") {
                                if let Some(addr) = details.get("subject").and_then(|s| s.as_str()) {
                                    if let Some(caps) = details.get("capabilities").and_then(|c| c.as_array()) {
                                        let cap_set: Vec<String> = caps.iter()
                                            .filter_map(|c| c.as_str().map(|s| s.to_string()))
                                            .collect();
                                        last_cap_entries.insert(addr.to_string(), cap_set);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => continue, // Skip malformed lines
            }
        }

        self.capability_index = last_cap_entries;
        self.event_count = loaded;

        if loaded > 0 {
            println!("[relay-audit] Loaded {} audit entries, {} capability baselines",
                loaded, self.capability_index.len());
        }
    }

    /// Record a new audit event. Returns the TIS-27 hash of the entry.
    ///
    /// The hash input is: `event_type|address|timestamp|details_json`
    /// The Merkle chain links via `parent_event_hash → last_event_hash`.
    pub fn record_event(&mut self, entry: &RelayAuditEntry) -> String {
        let event_type_str = serde_json::to_string(&entry.event_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let details_str = serde_json::to_string(&entry.details).unwrap_or_default();
        let hash_input = format!(
            "{}|{}|{}|{}",
            event_type_str, entry.address, entry.timestamp, details_str
        );
        let event_hash = hash_hex(hash_input.as_bytes());

        let event_id = uuid::Uuid::new_v4().to_string();

        let persisted = PersistedAuditEntry {
            hash: event_hash.clone(),
            parent_event_hash: self.last_event_hash.clone(),
            hash_algorithm: "tis-27".to_string(),
            event: event_type_str.clone(),
            jti: format!("relay-{}-{}", event_type_str, self.event_count),
            ts: entry.timestamp.clone(),
            details: entry.details.clone(),
            event_id,
            source_service: "relay".to_string(),
            severity: entry.severity.clone(),
            subsystem: entry.subsystem.clone(),
            correlation_refs: entry.correlation_refs.clone(),
        };

        self.leaves.push(event_hash.clone());
        self.last_event_hash = event_hash.clone();
        self.event_count += 1;

        // Update capability index for negotiation events
        if entry.event_type == RelayAuditEventType::CapabilityNegotiation {
            if let Some(caps) = entry.details.get("capabilities").and_then(|c| c.as_array()) {
                let cap_set: Vec<String> = caps.iter()
                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                    .collect();
                self.capability_index.insert(entry.address.clone(), cap_set);
            }
        }

        // Persist to disk
        self.persist(&persisted);

        event_hash
    }

    /// Persist a single entry to the JSONL file.
    fn persist(&self, entry: &PersistedAuditEntry) {
        if self.persist_path.as_os_str().is_empty() {
            return; // in-memory mode
        }
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.persist_path)
        {
            Ok(mut file) => {
                if let Ok(json) = serde_json::to_string(entry) {
                    let _ = writeln!(file, "{}", json);
                }
            }
            Err(e) => {
                eprintln!("[relay-audit] Failed to persist audit event: {}", e);
            }
        }
    }

    /// Get the last negotiated capability set for a node (O(1) lookup).
    pub fn get_capability_baseline(&self, address: &str) -> Option<&Vec<String>> {
        self.capability_index.get(address)
    }

    /// Get the current chain tip hash.
    pub fn last_event_hash(&self) -> &str {
        &self.last_event_hash
    }

    /// Get the total number of events in the log.
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Compute the Merkle root of all leaves using TIS-27.
    pub fn merkle_root(&self) -> String {
        if self.leaves.is_empty() {
            return hash_hex(b"relay-audit-genesis");
        }
        let mut level = self.leaves.clone();
        while level.len() > 1 {
            let mut next = Vec::with_capacity((level.len() + 1) / 2);
            for i in (0..level.len()).step_by(2) {
                let left = &level[i];
                let right = if i + 1 < level.len() { &level[i + 1] } else { left };
                let combined = format!("{}{}", left, right);
                next.push(hash_hex(combined.as_bytes()));
            }
            level = next;
        }
        level.into_iter().next().unwrap_or_default()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_entry(event_type: RelayAuditEventType) -> RelayAuditEntry {
        RelayAuditEntry {
            event_type,
            address: "1.1.1.1.1.1.1.1.1.1.1.1.1".to_string(),
            timestamp: "2026-04-11T00:00:00Z".to_string(),
            details: json!({"test": true}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        }
    }

    #[test]
    fn test_genesis_hash_is_tis27() {
        let log = RelayAuditLog::in_memory();
        // Genesis hash must NOT be SHA-3 — it's hash_hex(b"relay-audit-genesis")
        let expected = hash_hex(b"relay-audit-genesis");
        assert_eq!(log.last_event_hash(), expected);
        assert!(!expected.is_empty());
    }

    #[test]
    fn test_record_event_advances_chain() {
        let mut log = RelayAuditLog::in_memory();
        let genesis = log.last_event_hash().to_string();

        let hash1 = log.record_event(&test_entry(RelayAuditEventType::AuthSuccess));
        assert_ne!(hash1, genesis);
        assert_eq!(log.last_event_hash(), hash1);
        assert_eq!(log.event_count(), 1);

        let hash2 = log.record_event(&test_entry(RelayAuditEventType::Disconnect));
        assert_ne!(hash2, hash1);
        assert_eq!(log.last_event_hash(), hash2);
        assert_eq!(log.event_count(), 2);
    }

    #[test]
    fn test_capability_index_populated() {
        let mut log = RelayAuditLog::in_memory();
        let entry = RelayAuditEntry {
            event_type: RelayAuditEventType::CapabilityNegotiation,
            address: "2.1.3.1.2.3.1.2.3.1.2.3.1".to_string(),
            timestamp: "2026-04-11T00:00:00Z".to_string(),
            details: json!({
                "subject": "2.1.3.1.2.3.1.2.3.1.2.3.1",
                "capabilities": ["topics:1", "seq:1", "heartbeat:1"]
            }),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        };
        log.record_event(&entry);

        let baseline = log.get_capability_baseline("2.1.3.1.2.3.1.2.3.1.2.3.1");
        assert!(baseline.is_some());
        let caps = baseline.unwrap();
        assert_eq!(caps.len(), 3);
        assert!(caps.contains(&"topics:1".to_string()));
    }

    #[test]
    fn test_capability_index_updates_on_reconnect() {
        let mut log = RelayAuditLog::in_memory();
        let addr = "1.1.1.1.1.1.1.1.1.1.1.1.1".to_string();

        // First negotiation: topics:1
        log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::CapabilityNegotiation,
            address: addr.clone(),
            timestamp: "2026-04-11T00:00:00Z".to_string(),
            details: json!({"subject": &addr, "capabilities": ["topics:1"]}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        // Second negotiation: topics:1 + seq:1
        log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::CapabilityNegotiation,
            address: addr.clone(),
            timestamp: "2026-04-11T00:01:00Z".to_string(),
            details: json!({"subject": &addr, "capabilities": ["topics:1", "seq:1"]}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        // Index should have the latest set
        let baseline = log.get_capability_baseline(&addr).unwrap();
        assert_eq!(baseline.len(), 2);
        assert!(baseline.contains(&"seq:1".to_string()));
    }

    #[test]
    fn test_merkle_root_single() {
        let mut log = RelayAuditLog::in_memory();
        log.record_event(&test_entry(RelayAuditEventType::AuthSuccess));
        let root = log.merkle_root();
        assert!(!root.is_empty());
    }

    #[test]
    fn test_merkle_root_multiple() {
        let mut log = RelayAuditLog::in_memory();
        log.record_event(&test_entry(RelayAuditEventType::AuthSuccess));
        log.record_event(&test_entry(RelayAuditEventType::Disconnect));
        log.record_event(&test_entry(RelayAuditEventType::GoAway));
        let root = log.merkle_root();
        assert!(!root.is_empty());
        assert_eq!(log.event_count(), 3);
    }

    #[test]
    fn test_file_backed_persistence() {
        let dir = std::env::temp_dir().join("plenum_relay_audit_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test-audit.jsonl");
        let _ = fs::remove_file(&path);

        // Write
        {
            let mut log = RelayAuditLog::new(path.clone());
            log.record_event(&test_entry(RelayAuditEventType::AuthSuccess));
            log.record_event(&test_entry(RelayAuditEventType::Disconnect));
        }

        // Read back
        {
            let log = RelayAuditLog::new(path.clone());
            assert_eq!(log.event_count(), 2);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_all_event_types_serialize() {
        let types = [
            RelayAuditEventType::AuthSuccess, RelayAuditEventType::AuthFailure,
            RelayAuditEventType::Disconnect, RelayAuditEventType::Reconnect,
            RelayAuditEventType::Error, RelayAuditEventType::CircuitBreaker,
            RelayAuditEventType::GoAway, RelayAuditEventType::PeerOffline,
            RelayAuditEventType::CapabilityNegotiation, RelayAuditEventType::CapabilityEnforcement,
            RelayAuditEventType::CapabilityDowngrade, RelayAuditEventType::TopicSubscribe,
            RelayAuditEventType::TopicPublish, RelayAuditEventType::TopicReauthFailure,
            RelayAuditEventType::TopicRevoked, RelayAuditEventType::TopicReset,
            RelayAuditEventType::TopicLifecycle, RelayAuditEventType::Tombstone,
            RelayAuditEventType::ResyncRateLimit, RelayAuditEventType::HeartbeatFailure,
            RelayAuditEventType::HeartbeatIntervalChange, RelayAuditEventType::CircuitBreakerManualReset,
        ];
        for t in &types {
            let s = serde_json::to_string(t).unwrap();
            assert!(s.starts_with('"'));
            assert!(s.contains("relay."));
        }
    }
}
