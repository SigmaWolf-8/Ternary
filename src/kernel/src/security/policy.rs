//! Security Policy Engine
//!
//! Configurable policy rules that govern access control decisions
//! across the modal security system. Supports both mandatory access
//! control (MAC) and discretionary access control (DAC) patterns.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use super::{Action, ResourceId, SecurityError, SecurityResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyScope {
    Global,
    PerProcess,
    PerResource,
    PerMode,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub id: u64,
    pub name: String,
    pub scope: PolicyScope,
    pub mode_filter: Option<SecurityMode>,
    pub action_filter: Option<Action>,
    pub resource_filter: Option<ResourceId>,
    pub decision: PolicyDecision,
    pub priority: u32,
    pub enabled: bool,
}

impl PolicyRule {
    pub fn matches(
        &self,
        caller_mode: SecurityMode,
        action: &Action,
        resource: ResourceId,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(mode) = self.mode_filter {
            if mode != caller_mode {
                return false;
            }
        }

        if let Some(ref act) = self.action_filter {
            if act != action {
                return false;
            }
        }

        if let Some(res) = self.resource_filter {
            if res != resource {
                return false;
            }
        }

        true
    }
}

pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    next_rule_id: u64,
    default_decision: PolicyDecision,
    mode_restrictions: BTreeMap<u8, Vec<Action>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            next_rule_id: 0,
            default_decision: PolicyDecision::Deny,
            mode_restrictions: BTreeMap::new(),
        };
        engine.initialize_default_restrictions();
        engine
    }

    pub fn with_default(default: PolicyDecision) -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            next_rule_id: 0,
            default_decision: default,
            mode_restrictions: BTreeMap::new(),
        };
        engine.initialize_default_restrictions();
        engine
    }

    fn initialize_default_restrictions(&mut self) {
        self.mode_restrictions.insert(
            SecurityMode::ModePhi.access_level(),
            Vec::new(),
        );

        self.mode_restrictions.insert(
            SecurityMode::ModeOne.access_level(),
            vec![Action::ModifyPolicy],
        );

        self.mode_restrictions.insert(
            SecurityMode::ModeZero.access_level(),
            vec![
                Action::ModifyPolicy,
                Action::CreateProcess,
                Action::TerminateProcess,
                Action::PhaseEncrypt,
                Action::PhaseDecrypt,
            ],
        );
    }

    pub fn add_rule(&mut self, rule: PolicyRule) -> u64 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;

        let mut rule = rule;
        rule.id = id;
        self.rules.push(rule);

        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        id
    }

    pub fn remove_rule(&mut self, id: u64) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() < len_before
    }

    pub fn enable_rule(&mut self, id: u64) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_rule(&mut self, id: u64) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn evaluate(
        &self,
        caller_mode: SecurityMode,
        action: &Action,
        resource: ResourceId,
    ) -> PolicyDecision {
        if let Some(restricted) = self.mode_restrictions.get(&caller_mode.access_level()) {
            if restricted.contains(action) {
                return PolicyDecision::Deny;
            }
        }

        for rule in &self.rules {
            if rule.matches(caller_mode, action, resource) {
                return rule.decision;
            }
        }

        self.default_decision
    }

    pub fn check_access(
        &self,
        caller_mode: SecurityMode,
        action: &Action,
        resource: ResourceId,
    ) -> SecurityResult<PolicyDecision> {
        let decision = self.evaluate(caller_mode, action, resource);
        match decision {
            PolicyDecision::Deny => Err(SecurityError::PolicyViolation(
                String::from("Access denied by policy"),
            )),
            other => Ok(other),
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn active_rule_count(&self) -> usize {
        self.rules.iter().filter(|r| r.enabled).count()
    }

    pub fn get_rule(&self, id: u64) -> Option<&PolicyRule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn rules_for_mode(&self, mode: SecurityMode) -> Vec<&PolicyRule> {
        self.rules.iter()
            .filter(|r| r.mode_filter == Some(mode) || r.mode_filter.is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_creation() {
        let engine = PolicyEngine::new();
        assert_eq!(engine.rule_count(), 0);
        assert_eq!(engine.default_decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_default_deny() {
        let engine = PolicyEngine::new();
        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_mode_restrictions_zero() {
        let engine = PolicyEngine::new();

        let decision = engine.evaluate(
            SecurityMode::ModeZero, &Action::ModifyPolicy, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);

        let decision = engine.evaluate(
            SecurityMode::ModeZero, &Action::CreateProcess, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_mode_restrictions_phi_unrestricted() {
        let engine = PolicyEngine::with_default(PolicyDecision::Allow);

        let decision = engine.evaluate(
            SecurityMode::ModePhi, &Action::ModifyPolicy, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn test_add_allow_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("allow reads"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: Some(Action::Read),
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 10,
            enabled: true,
        });

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Allow);

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Write, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_rule_priority() {
        let mut engine = PolicyEngine::new();

        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("allow all"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: None,
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 1,
            enabled: true,
        });

        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("deny writes"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: Some(Action::Write),
            resource_filter: None,
            decision: PolicyDecision::Deny,
            priority: 10,
            enabled: true,
        });

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Write, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn test_mode_specific_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("phi can do anything"),
            scope: PolicyScope::PerMode,
            mode_filter: Some(SecurityMode::ModePhi),
            action_filter: None,
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 100,
            enabled: true,
        });

        let decision = engine.evaluate(
            SecurityMode::ModePhi, &Action::Write, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Allow);

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Write, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_resource_specific_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("public resource"),
            scope: PolicyScope::PerResource,
            mode_filter: None,
            action_filter: Some(Action::Read),
            resource_filter: Some(ResourceId(42)),
            decision: PolicyDecision::Allow,
            priority: 10,
            enabled: true,
        });

        let decision = engine.evaluate(
            SecurityMode::ModeZero, &Action::Read, ResourceId(42),
        );
        assert_eq!(decision, PolicyDecision::Allow);

        let decision = engine.evaluate(
            SecurityMode::ModeZero, &Action::Read, ResourceId(43),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_disable_enable_rule() {
        let mut engine = PolicyEngine::new();
        let id = engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("allow reads"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: Some(Action::Read),
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 10,
            enabled: true,
        });

        assert_eq!(engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        ), PolicyDecision::Allow);

        engine.disable_rule(id);

        assert_eq!(engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        ), PolicyDecision::Deny);

        engine.enable_rule(id);

        assert_eq!(engine.evaluate(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        ), PolicyDecision::Allow);
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = PolicyEngine::new();
        let id = engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("test"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: None,
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 10,
            enabled: true,
        });

        assert_eq!(engine.rule_count(), 1);
        assert!(engine.remove_rule(id));
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn test_check_access() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("allow reads"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: Some(Action::Read),
            resource_filter: None,
            decision: PolicyDecision::Allow,
            priority: 10,
            enabled: true,
        });

        assert!(engine.check_access(
            SecurityMode::ModeOne, &Action::Read, ResourceId(1),
        ).is_ok());

        assert!(engine.check_access(
            SecurityMode::ModeOne, &Action::Write, ResourceId(1),
        ).is_err());
    }

    #[test]
    fn test_audit_decision() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule {
            id: 0,
            name: String::from("audit writes"),
            scope: PolicyScope::Global,
            mode_filter: None,
            action_filter: Some(Action::Write),
            resource_filter: None,
            decision: PolicyDecision::Audit,
            priority: 10,
            enabled: true,
        });

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::Write, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Audit);
    }

    #[test]
    fn test_mode_one_restricted() {
        let engine = PolicyEngine::new();

        let decision = engine.evaluate(
            SecurityMode::ModeOne, &Action::ModifyPolicy, ResourceId(1),
        );
        assert_eq!(decision, PolicyDecision::Deny);
    }
}
