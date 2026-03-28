// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationConfig {
    #[serde(default = "default_require_for")]
    pub require_for: Vec<String>,
    #[serde(default = "default_auto_approve")]
    pub auto_approve: Vec<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_require_for() -> Vec<String> {
    vec!["exec".into(), "model-swap".into(), "file-push".into()]
}

fn default_auto_approve() -> Vec<String> {
    vec!["verify".into(), "pubkey".into(), "status".into(), "tail".into(), "file-pull".into()]
}

fn default_timeout_secs() -> u64 {
    60
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        ConfirmationConfig {
            require_for: default_require_for(),
            auto_approve: default_auto_approve(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationResult {
    AutoApproved,
    Approved,
    Rejected,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub id: String,
    pub context: String,
    pub payload_hash: String,
    pub origin: Option<String>,
    pub created: Instant,
    pub decision: Option<ConfirmationResult>,
}

#[allow(dead_code)]
pub struct ConfirmationQueue {
    pending: HashMap<String, PendingRequest>,
    counter: u64,
}

#[allow(dead_code)]
impl ConfirmationQueue {
    pub fn new() -> Self {
        ConfirmationQueue {
            pending: HashMap::new(),
            counter: 0,
        }
    }

    pub fn submit(&mut self, context: String, payload_hash: String, origin: Option<String>) -> String {
        self.counter += 1;
        let id = format!("req-{:06}", self.counter);
        self.pending.insert(id.clone(), PendingRequest {
            id: id.clone(),
            context,
            payload_hash,
            origin,
            created: Instant::now(),
            decision: None,
        });
        id
    }

    pub fn approve(&mut self, id: &str) -> bool {
        if let Some(req) = self.pending.get_mut(id) {
            req.decision = Some(ConfirmationResult::Approved);
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, id: &str) -> bool {
        if let Some(req) = self.pending.get_mut(id) {
            req.decision = Some(ConfirmationResult::Rejected);
            true
        } else {
            false
        }
    }

    pub fn check(&mut self, id: &str, timeout: Duration) -> Option<ConfirmationResult> {
        if let Some(req) = self.pending.get(id) {
            if let Some(ref decision) = req.decision {
                let result = decision.clone();
                self.pending.remove(id);
                return Some(result);
            }
            if req.created.elapsed() > timeout {
                self.pending.remove(id);
                return Some(ConfirmationResult::Timeout);
            }
            None
        } else {
            Some(ConfirmationResult::Rejected)
        }
    }

    #[allow(dead_code)]
    pub fn pending_list(&self) -> Vec<&PendingRequest> {
        self.pending.values().collect()
    }

    pub fn expire_stale(&mut self, timeout: Duration) {
        self.pending.retain(|_, req| req.created.elapsed() <= timeout);
    }
}

pub type SharedConfirmationQueue = Arc<Mutex<ConfirmationQueue>>;

impl ConfirmationConfig {
    pub fn requires_confirmation(&self, context: &str) -> bool {
        let ctx_lower = context.to_lowercase();
        for pattern in &self.require_for {
            if ctx_lower.starts_with(&pattern.to_lowercase()) {
                return true;
            }
        }
        false
    }

    pub fn is_auto_approved(&self, context: &str) -> bool {
        let ctx_lower = context.to_lowercase();
        for pattern in &self.auto_approve {
            if ctx_lower.starts_with(&pattern.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

pub fn evaluate_confirmation(config: &ConfirmationConfig, context: &str, headless: bool) -> ConfirmationResult {
    if config.is_auto_approved(context) {
        return ConfirmationResult::AutoApproved;
    }

    if !config.requires_confirmation(context) {
        return ConfirmationResult::AutoApproved;
    }

    if headless {
        return ConfirmationResult::AutoApproved;
    }

    ConfirmationResult::Rejected
}

pub fn confirmation_label(result: &ConfirmationResult) -> &'static str {
    match result {
        ConfirmationResult::AutoApproved => "auto",
        ConfirmationResult::Approved => "approved",
        ConfirmationResult::Rejected => "rejected",
        ConfirmationResult::Timeout => "timeout",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_confirmation() {
        let config = ConfirmationConfig::default();
        assert!(config.requires_confirmation("exec: Get-Service"));
        assert!(config.requires_confirmation("model-swap: gpt4"));
        assert!(config.requires_confirmation("file-push: config.json"));
        assert!(!config.requires_confirmation("unknown-op: something"));
    }

    #[test]
    fn test_auto_approve() {
        let config = ConfirmationConfig::default();
        assert!(config.is_auto_approved("verify: check signature"));
        assert!(config.is_auto_approved("pubkey: export"));
        assert!(config.is_auto_approved("status: check"));
    }

    #[test]
    fn test_headless_auto_approves() {
        let config = ConfirmationConfig::default();
        let result = evaluate_confirmation(&config, "exec: Get-Service", true);
        assert_eq!(result, ConfirmationResult::AutoApproved);
    }

    #[test]
    fn test_interactive_rejects_without_gui() {
        let config = ConfirmationConfig::default();
        let result = evaluate_confirmation(&config, "exec: Get-Service", false);
        assert_eq!(result, ConfirmationResult::Rejected);
    }

    #[test]
    fn test_auto_approve_operations_always_pass() {
        let config = ConfirmationConfig::default();
        let result = evaluate_confirmation(&config, "verify: check", false);
        assert_eq!(result, ConfirmationResult::AutoApproved);
    }

    #[test]
    fn test_confirmation_queue_submit_approve() {
        let mut queue = ConfirmationQueue::new();
        let id = queue.submit("exec: test".into(), "abc123".into(), None);
        assert!(queue.approve(&id));
        let result = queue.check(&id, Duration::from_secs(60));
        assert_eq!(result, Some(ConfirmationResult::Approved));
    }

    #[test]
    fn test_confirmation_queue_submit_reject() {
        let mut queue = ConfirmationQueue::new();
        let id = queue.submit("exec: test".into(), "abc123".into(), None);
        assert!(queue.reject(&id));
        let result = queue.check(&id, Duration::from_secs(60));
        assert_eq!(result, Some(ConfirmationResult::Rejected));
    }

    #[test]
    fn test_confirmation_queue_timeout() {
        let mut queue = ConfirmationQueue::new();
        let id = queue.submit("exec: test".into(), "abc123".into(), None);
        let result = queue.check(&id, Duration::from_secs(0));
        assert_eq!(result, Some(ConfirmationResult::Timeout));
    }

    #[test]
    fn test_confirmation_queue_pending() {
        let mut queue = ConfirmationQueue::new();
        let id = queue.submit("exec: test".into(), "abc123".into(), None);
        let result = queue.check(&id, Duration::from_secs(60));
        assert_eq!(result, None);
    }
}
