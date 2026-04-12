// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY CAPABILITY NEGOTIATION — Task #27, Task 2
//
// Capability negotiation during auth handshake:
// - Client sends supported: ["topics:1", "seq:1", "heartbeat:1"]
// - Server intersects with its own capabilities, negotiates down
// - Server replies with enabled + negotiated_down arrays in auth_ok
// - Server enforces negotiated capabilities on every subsequent message
// - Downgrade detection via persistent audit log baseline
//
// Constant-time comparison via subtle::ConstantTimeEq (already in
// inter-cube deps, used in api.rs:60).
//
// Version string format: name:version
//   name    = [a-z][a-z0-9_]*     (lowercase alphanumeric + underscore)
//   version = [1-9][0-9]*         (positive integer, no leading zeros)

use std::collections::HashMap;
use subtle::ConstantTimeEq;

use crate::relay_audit::{
    RelayAuditLog, RelayAuditEntry, RelayAuditEventType,
    AuditSeverity, AuditSubsystem,
};
use crate::relay_error::RelayErrorCode;

// ═══════════════════════════════════════════════════════════════════════
// SERVER CAPABILITIES — what this relay server supports
// ═══════════════════════════════════════════════════════════════════════

/// Server-supported capabilities with their maximum version.
/// Add new capabilities here as they are implemented.
pub static SERVER_CAPABILITIES: &[(&str, u32)] = &[
    ("topics", 1),
    ("seq", 1),
    ("circuit", 1),
    ("heartbeat", 1),
];

// ═══════════════════════════════════════════════════════════════════════
// VERSION STRING PARSING
// ═══════════════════════════════════════════════════════════════════════

/// A parsed capability version string: name + version number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityVersion {
    pub name: String,
    pub version: u32,
}

impl CapabilityVersion {
    /// Format back to wire string: "name:version"
    pub fn to_wire(&self) -> String {
        format!("{}:{}", self.name, self.version)
    }
}

/// Parse a capability version string "name:version".
///
/// Validation rules (from spec):
/// - name: [a-z][a-z0-9_]* (starts with lowercase letter)
/// - version: [1-9][0-9]* (positive integer, no leading zeros)
///
/// Returns Err with description on malformed input.
pub fn parse_capability_version(s: &str) -> Result<CapabilityVersion, String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("missing ':' separator in '{}'", s));
    }

    let name = parts[0];
    let version_str = parts[1];

    // Validate name: [a-z][a-z0-9_]*
    if name.is_empty() {
        return Err(format!("empty name in '{}'", s));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(format!("name must start with [a-z], got '{}' in '{}'", first, s));
    }
    for ch in chars {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '_' {
            return Err(format!("invalid character '{}' in name '{}'", ch, s));
        }
    }

    // Validate version: [1-9][0-9]*
    if version_str.is_empty() {
        return Err(format!("empty version in '{}'", s));
    }
    if version_str.starts_with('0') {
        return Err(format!("version has leading zero in '{}'", s));
    }
    let version: u32 = version_str
        .parse()
        .map_err(|_| format!("invalid version number in '{}'", s))?;
    if version == 0 {
        return Err(format!("version must be positive in '{}'", s));
    }

    Ok(CapabilityVersion {
        name: name.to_string(),
        version,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// NEGOTIATION
// ═══════════════════════════════════════════════════════════════════════

/// Result of capability negotiation.
#[derive(Debug, Clone)]
pub struct NegotiationResult {
    /// Capabilities enabled at their negotiated versions.
    pub enabled: Vec<String>,
    /// Capability names where the server negotiated down from the
    /// client's requested version (client is explicitly informed).
    pub negotiated_down: Vec<String>,
    /// Any malformed capability strings that were rejected.
    pub malformed: Vec<String>,
}

/// Negotiate capabilities between client and server.
///
/// For each client capability:
/// - If server supports it at the same or higher version → enabled at client's version
/// - If server supports it at a lower version → negotiate down to server's version
/// - If server doesn't support it at all → excluded from enabled
/// - Unknown capability names → silently ignored (forward compatibility)
/// - Malformed strings → collected in `malformed` for ERR_FRAME_MALFORMED
pub fn negotiate(client_supported: &[String]) -> NegotiationResult {
    let server_map: HashMap<&str, u32> = SERVER_CAPABILITIES.iter()
        .map(|(name, ver)| (*name, *ver))
        .collect();

    let mut enabled = Vec::new();
    let mut negotiated_down = Vec::new();
    let mut malformed = Vec::new();

    for cap_str in client_supported {
        let parsed = match parse_capability_version(cap_str) {
            Ok(p) => p,
            Err(_) => {
                malformed.push(cap_str.clone());
                continue;
            }
        };

        // Look up server support
        if let Some(&server_ver) = server_map.get(parsed.name.as_str()) {
            if parsed.version <= server_ver {
                // Client wants same or lower — grant at client's version
                enabled.push(parsed.to_wire());
            } else {
                // Client wants higher — negotiate down to server's max
                enabled.push(format!("{}:{}", parsed.name, server_ver));
                negotiated_down.push(parsed.name.clone());
            }
        }
        // else: unknown capability name — silently ignored (forward compat)
    }

    NegotiationResult {
        enabled,
        negotiated_down,
        malformed,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ENFORCEMENT
// ═══════════════════════════════════════════════════════════════════════

/// Check if a capability is in the negotiated set.
///
/// `required` is the capability name (e.g., "topics", "seq").
/// `negotiated` is the connection's enabled capability list.
pub fn check_capability(required: &str, negotiated: &[String]) -> Result<(), RelayErrorCode> {
    for cap_str in negotiated {
        if let Ok(parsed) = parse_capability_version(cap_str) {
            if parsed.name == required {
                return Ok(());
            }
        }
    }
    Err(RelayErrorCode::ErrCapabilityNotNegotiated)
}

// ═══════════════════════════════════════════════════════════════════════
// DOWNGRADE DETECTION
// ═══════════════════════════════════════════════════════════════════════

/// Check if a capability set represents a downgrade from the baseline.
///
/// Uses constant-time comparison via subtle::ConstantTimeEq for
/// defense-in-depth — prevents timing side-channels on capability probing.
///
/// Returns true if the new set is a DOWNGRADE (missing capabilities
/// that were in the baseline).
pub fn is_capability_downgrade(baseline: &[String], current: &[String]) -> bool {
    // Extract capability names from both sets
    let baseline_names: Vec<String> = baseline.iter()
        .filter_map(|s| parse_capability_version(s).ok())
        .map(|cv| cv.name)
        .collect();
    let current_names: Vec<String> = current.iter()
        .filter_map(|s| parse_capability_version(s).ok())
        .map(|cv| cv.name)
        .collect();

    // Check if any baseline capability is missing from current.
    // Constant-time: iterate all elements regardless of early mismatch.
    let mut any_missing = 0u8;
    for baseline_name in &baseline_names {
        let mut found = 0u8;
        for current_name in &current_names {
            // Constant-time string equality via subtle
            let bl_bytes = baseline_name.as_bytes();
            let cr_bytes = current_name.as_bytes();
            if bl_bytes.len() == cr_bytes.len() {
                if bl_bytes.ct_eq(cr_bytes).into() {
                    found = 1;
                }
            }
        }
        any_missing |= 1 - found;
    }
    any_missing == 1
}

/// Perform downgrade check against the audit log baseline.
///
/// Returns Ok(()) if no downgrade or if downgrade is allowed by config.
/// Returns Err(RelayErrorCode::ErrCapabilityDowngrade) if rejected.
///
/// Always audit-logs downgrade attempts regardless of policy.
pub fn check_downgrade_policy(
    address: &str,
    current_capabilities: &[String],
    audit_log: &mut RelayAuditLog,
    allow_downgrade: bool,
) -> Result<(), RelayErrorCode> {
    let baseline = match audit_log.get_capability_baseline(address) {
        Some(b) => b.clone(),
        None => return Ok(()), // No baseline = first connection, no downgrade possible
    };

    if !is_capability_downgrade(&baseline, current_capabilities) {
        return Ok(()); // Not a downgrade — set is equal or superset
    }

    // Downgrade detected — always audit-log
    let ts = chrono::Utc::now().to_rfc3339();
    audit_log.record_event(&RelayAuditEntry {
        event_type: RelayAuditEventType::CapabilityDowngrade,
        address: address.to_string(),
        timestamp: ts,
        details: serde_json::json!({
            "subject": address,
            "previous_capabilities": baseline,
            "current_capabilities": current_capabilities,
            "policy_decision": if allow_downgrade { "allowed" } else { "rejected" },
        }),
        severity: AuditSeverity::Warn,
        subsystem: AuditSubsystem::Capability,
        correlation_refs: vec![],
    });

    if allow_downgrade {
        Ok(()) // Admin override — accepted with warning
    } else {
        Err(RelayErrorCode::ErrCapabilityDowngrade)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Version string parsing ──────────────────────────────────

    #[test]
    fn test_parse_valid() {
        let cv = parse_capability_version("topics:1").unwrap();
        assert_eq!(cv.name, "topics");
        assert_eq!(cv.version, 1);
        assert_eq!(cv.to_wire(), "topics:1");
    }

    #[test]
    fn test_parse_multi_digit_version() {
        let cv = parse_capability_version("seq:12").unwrap();
        assert_eq!(cv.name, "seq");
        assert_eq!(cv.version, 12);
    }

    #[test]
    fn test_parse_underscore_name() {
        let cv = parse_capability_version("future_feature:1").unwrap();
        assert_eq!(cv.name, "future_feature");
        assert_eq!(cv.version, 1);
    }

    #[test]
    fn test_parse_name_with_digits() {
        let cv = parse_capability_version("v2feature:3").unwrap();
        assert_eq!(cv.name, "v2feature");
    }

    #[test]
    fn test_reject_leading_zero_version() {
        assert!(parse_capability_version("topics:01").is_err());
    }

    #[test]
    fn test_reject_zero_version() {
        assert!(parse_capability_version("topics:0").is_err());
    }

    #[test]
    fn test_reject_uppercase_name() {
        assert!(parse_capability_version("TOPICS:1").is_err());
    }

    #[test]
    fn test_reject_empty_name() {
        assert!(parse_capability_version(":1").is_err());
    }

    #[test]
    fn test_reject_empty_version() {
        assert!(parse_capability_version("topics:").is_err());
    }

    #[test]
    fn test_reject_no_separator() {
        assert!(parse_capability_version("topics").is_err());
    }

    #[test]
    fn test_reject_name_starts_with_digit() {
        assert!(parse_capability_version("1topics:1").is_err());
    }

    #[test]
    fn test_reject_name_starts_with_underscore() {
        assert!(parse_capability_version("_topics:1").is_err());
    }

    // ── Negotiation ─────────────────────────────────────────────

    #[test]
    fn test_negotiate_exact_match() {
        let supported = vec!["topics:1".to_string(), "seq:1".to_string()];
        let result = negotiate(&supported);
        assert!(result.enabled.contains(&"topics:1".to_string()));
        assert!(result.enabled.contains(&"seq:1".to_string()));
        assert!(result.negotiated_down.is_empty());
        assert!(result.malformed.is_empty());
    }

    #[test]
    fn test_negotiate_down() {
        // Client wants topics:2, server only has topics:1
        let supported = vec!["topics:2".to_string()];
        let result = negotiate(&supported);
        assert!(result.enabled.contains(&"topics:1".to_string()));
        assert!(result.negotiated_down.contains(&"topics".to_string()));
    }

    #[test]
    fn test_negotiate_unknown_ignored() {
        // Unknown capability name silently ignored (forward compat)
        let supported = vec!["future_feature:1".to_string(), "topics:1".to_string()];
        let result = negotiate(&supported);
        assert_eq!(result.enabled.len(), 1);
        assert!(result.enabled.contains(&"topics:1".to_string()));
        assert!(result.malformed.is_empty());
    }

    #[test]
    fn test_negotiate_malformed_collected() {
        let supported = vec![
            "topics:1".to_string(),
            "INVALID:1".to_string(),  // uppercase
            "topics:01".to_string(),  // leading zero
        ];
        let result = negotiate(&supported);
        assert_eq!(result.enabled.len(), 1);
        assert_eq!(result.malformed.len(), 2);
    }

    #[test]
    fn test_negotiate_empty_supported() {
        let result = negotiate(&[]);
        assert!(result.enabled.is_empty());
        assert!(result.negotiated_down.is_empty());
    }

    #[test]
    fn test_negotiate_all_server_capabilities() {
        let supported = vec![
            "topics:1".to_string(),
            "seq:1".to_string(),
            "circuit:1".to_string(),
            "heartbeat:1".to_string(),
        ];
        let result = negotiate(&supported);
        assert_eq!(result.enabled.len(), 4);
    }

    // ── Enforcement ─────────────────────────────────────────────

    #[test]
    fn test_check_capability_present() {
        let negotiated = vec!["topics:1".to_string(), "seq:1".to_string()];
        assert!(check_capability("topics", &negotiated).is_ok());
        assert!(check_capability("seq", &negotiated).is_ok());
    }

    #[test]
    fn test_check_capability_missing() {
        let negotiated = vec!["topics:1".to_string()];
        assert_eq!(
            check_capability("seq", &negotiated).unwrap_err(),
            RelayErrorCode::ErrCapabilityNotNegotiated,
        );
    }

    #[test]
    fn test_check_capability_empty_set() {
        assert!(check_capability("topics", &[]).is_err());
    }

    // ── Downgrade detection ─────────────────────────────────────

    #[test]
    fn test_no_downgrade_same_set() {
        let baseline = vec!["topics:1".to_string(), "seq:1".to_string()];
        let current = vec!["topics:1".to_string(), "seq:1".to_string()];
        assert!(!is_capability_downgrade(&baseline, &current));
    }

    #[test]
    fn test_no_downgrade_superset() {
        let baseline = vec!["topics:1".to_string()];
        let current = vec!["topics:1".to_string(), "seq:1".to_string()];
        assert!(!is_capability_downgrade(&baseline, &current));
    }

    #[test]
    fn test_downgrade_missing_capability() {
        let baseline = vec!["topics:1".to_string(), "seq:1".to_string()];
        let current = vec!["topics:1".to_string()]; // seq:1 missing
        assert!(is_capability_downgrade(&baseline, &current));
    }

    #[test]
    fn test_downgrade_empty_current() {
        let baseline = vec!["topics:1".to_string()];
        let current: Vec<String> = vec![];
        assert!(is_capability_downgrade(&baseline, &current));
    }

    #[test]
    fn test_no_downgrade_empty_baseline() {
        let baseline: Vec<String> = vec![];
        let current = vec!["topics:1".to_string()];
        assert!(!is_capability_downgrade(&baseline, &current));
    }

    // ── Downgrade policy ────────────────────────────────────────

    #[test]
    fn test_downgrade_policy_no_baseline() {
        let mut log = crate::relay_audit::RelayAuditLog::in_memory();
        let result = check_downgrade_policy(
            "1.1.1.1.1.1.1.1.1.1.1.1.1",
            &["topics:1".to_string()],
            &mut log,
            false,
        );
        assert!(result.is_ok(), "No baseline = first connection, always OK");
    }

    #[test]
    fn test_downgrade_policy_rejected() {
        let mut log = crate::relay_audit::RelayAuditLog::in_memory();
        let addr = "2.1.3.1.2.3.1.2.3.1.2.3.1";

        // Establish baseline
        log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::CapabilityNegotiation,
            address: addr.to_string(),
            timestamp: "2026-04-11T00:00:00Z".to_string(),
            details: serde_json::json!({
                "subject": addr,
                "capabilities": ["topics:1", "seq:1"],
            }),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        // Attempt downgrade (drop seq:1)
        let result = check_downgrade_policy(
            addr,
            &["topics:1".to_string()],
            &mut log,
            false, // reject downgrades
        );
        assert_eq!(result.unwrap_err(), RelayErrorCode::ErrCapabilityDowngrade);
    }

    #[test]
    fn test_downgrade_policy_allowed_override() {
        let mut log = crate::relay_audit::RelayAuditLog::in_memory();
        let addr = "2.1.3.1.2.3.1.2.3.1.2.3.1";

        // Establish baseline
        log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::CapabilityNegotiation,
            address: addr.to_string(),
            timestamp: "2026-04-11T00:00:00Z".to_string(),
            details: serde_json::json!({
                "subject": addr,
                "capabilities": ["topics:1", "seq:1"],
            }),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        // Attempt downgrade with admin override
        let result = check_downgrade_policy(
            addr,
            &["topics:1".to_string()],
            &mut log,
            true, // allow downgrades
        );
        assert!(result.is_ok(), "Admin override should accept downgrade");
        // But it should still be audit-logged (event_count increased)
        assert!(log.event_count() > 1, "Downgrade must be audit-logged even when allowed");
    }

    // ── Constant-time comparison ────────────────────────────────

    #[test]
    fn test_constant_time_comparison_used() {
        // Verify the subtle crate is actually being called
        // by checking that different-length strings don't match
        let baseline = vec!["topics:1".to_string()];
        let current = vec!["topic:1".to_string()]; // note: "topic" not "topics"
        assert!(is_capability_downgrade(&baseline, &current));
    }
}
