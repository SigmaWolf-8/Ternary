// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    pub result: String,
    pub confirmation: String,
    pub duration_ms: u64,
}

pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    pub fn new(data_dir: &PathBuf) -> Self {
        AuditLog {
            path: data_dir.join("ninja-exec-audit.jsonl"),
        }
    }

    pub fn append(&self, entry: &AuditEntry) {
        if let Ok(json) = serde_json::to_string(entry) {
            if let Some(parent) = self.path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub fn hash_payload(payload: &[u8]) -> String {
    let hash = ternary_math::sponge::derive_key(b"NinjaExec-AUDIT-HASH", payload, 16);
    let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("tis27:{}", hex)
}

pub type SharedAuditLog = std::sync::Arc<Mutex<AuditLog>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_append_and_format() {
        let dir = std::env::temp_dir().join(format!("ninja-exec-audit-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);

        let log = AuditLog::new(&dir);
        let entry = AuditEntry {
            timestamp: "2026-03-27T14:30:00Z".to_string(),
            operation: "sign".to_string(),
            context: Some("exec: Get-Service on node-1".to_string()),
            payload_hash: Some("tis27:abc123".to_string()),
            origin: Some("http://yoda.replit.app".to_string()),
            result: "signed".to_string(),
            confirmation: "auto".to_string(),
            duration_ms: 12,
        };
        log.append(&entry);

        let content = fs::read_to_string(log.path()).unwrap();
        assert!(content.contains("\"operation\":\"sign\""));
        assert!(content.contains("\"result\":\"signed\""));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hash_payload_deterministic() {
        let h1 = hash_payload(b"test payload");
        let h2 = hash_payload(b"test payload");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("tis27:"));
    }
}
