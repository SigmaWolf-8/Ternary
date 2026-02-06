//! Security Domains
//!
//! Provides isolation boundaries for processes with controlled transitions.
//! Each domain has a security mode, a set of allowed transitions, and
//! maintains a list of member processes.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::process::ProcessId;
use crate::timing::FemtosecondTimestamp;
use super::{DomainId, SecurityError, SecurityResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    Upgrade,
    Downgrade,
    Lateral,
}

#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub from: DomainId,
    pub to: DomainId,
    pub kind: TransitionKind,
    pub required_mode: SecurityMode,
    pub requires_audit: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityDomain {
    pub id: DomainId,
    pub name: String,
    pub mode: SecurityMode,
    pub members: BTreeSet<ProcessId>,
    pub max_members: usize,
    pub created_at: FemtosecondTimestamp,
    pub isolated: bool,
}

impl SecurityDomain {
    pub fn new(
        id: DomainId,
        name: String,
        mode: SecurityMode,
        max_members: usize,
        created_at: FemtosecondTimestamp,
    ) -> Self {
        Self {
            id,
            name,
            mode,
            members: BTreeSet::new(),
            max_members,
            created_at,
            isolated: false,
        }
    }

    pub fn add_member(&mut self, pid: ProcessId) -> SecurityResult<()> {
        if self.members.len() >= self.max_members {
            return Err(SecurityError::PolicyViolation(
                String::from("Domain member limit reached"),
            ));
        }
        self.members.insert(pid);
        Ok(())
    }

    pub fn remove_member(&mut self, pid: ProcessId) -> bool {
        self.members.remove(&pid)
    }

    pub fn contains(&self, pid: ProcessId) -> bool {
        self.members.contains(&pid)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn set_isolated(&mut self, isolated: bool) {
        self.isolated = isolated;
    }
}

pub struct DomainManager {
    domains: BTreeMap<DomainId, SecurityDomain>,
    transitions: Vec<TransitionRule>,
    next_id: u64,
    process_domain: BTreeMap<ProcessId, DomainId>,
}

impl DomainManager {
    pub fn new() -> Self {
        Self {
            domains: BTreeMap::new(),
            transitions: Vec::new(),
            next_id: 0,
            process_domain: BTreeMap::new(),
        }
    }

    pub fn create_domain(
        &mut self,
        name: String,
        mode: SecurityMode,
        max_members: usize,
        timestamp: FemtosecondTimestamp,
    ) -> DomainId {
        let id = DomainId(self.next_id);
        self.next_id += 1;
        let domain = SecurityDomain::new(id, name, mode, max_members, timestamp);
        self.domains.insert(id, domain);
        id
    }

    pub fn get_domain(&self, id: DomainId) -> SecurityResult<&SecurityDomain> {
        self.domains.get(&id).ok_or(SecurityError::DomainNotFound(id))
    }

    pub fn get_domain_mut(&mut self, id: DomainId) -> SecurityResult<&mut SecurityDomain> {
        self.domains.get_mut(&id).ok_or(SecurityError::DomainNotFound(id))
    }

    pub fn assign_process(
        &mut self,
        pid: ProcessId,
        domain_id: DomainId,
    ) -> SecurityResult<()> {
        if let Some(&old_domain) = self.process_domain.get(&pid) {
            if let Some(old) = self.domains.get_mut(&old_domain) {
                old.remove_member(pid);
            }
        }

        let domain = self.domains.get_mut(&domain_id)
            .ok_or(SecurityError::DomainNotFound(domain_id))?;
        domain.add_member(pid)?;
        self.process_domain.insert(pid, domain_id);
        Ok(())
    }

    pub fn remove_process(&mut self, pid: ProcessId) -> Option<DomainId> {
        if let Some(domain_id) = self.process_domain.remove(&pid) {
            if let Some(domain) = self.domains.get_mut(&domain_id) {
                domain.remove_member(pid);
            }
            Some(domain_id)
        } else {
            None
        }
    }

    pub fn process_domain(&self, pid: ProcessId) -> Option<DomainId> {
        self.process_domain.get(&pid).copied()
    }

    pub fn process_mode(&self, pid: ProcessId) -> Option<SecurityMode> {
        self.process_domain.get(&pid)
            .and_then(|did| self.domains.get(did))
            .map(|d| d.mode)
    }

    pub fn add_transition_rule(&mut self, rule: TransitionRule) {
        self.transitions.push(rule);
    }

    pub fn can_transition(
        &self,
        pid: ProcessId,
        from: DomainId,
        to: DomainId,
        caller_mode: SecurityMode,
    ) -> SecurityResult<&TransitionRule> {
        let from_domain = self.domains.get(&from)
            .ok_or(SecurityError::DomainNotFound(from))?;
        let _to_domain = self.domains.get(&to)
            .ok_or(SecurityError::DomainNotFound(to))?;

        if from_domain.isolated {
            return Err(SecurityError::PolicyViolation(
                String::from("Source domain is isolated"),
            ));
        }

        if !from_domain.contains(pid) {
            return Err(SecurityError::PolicyViolation(
                String::from("Process not in source domain"),
            ));
        }

        for rule in &self.transitions {
            if rule.from == from && rule.to == to {
                if caller_mode.can_access(&rule.required_mode) {
                    return Ok(rule);
                }
            }
        }

        Err(SecurityError::InvalidTransition {
            from: from_domain.mode,
            to: self.domains.get(&to).map(|d| d.mode).unwrap_or(SecurityMode::ModeZero),
        })
    }

    pub fn execute_transition(
        &mut self,
        pid: ProcessId,
        from: DomainId,
        to: DomainId,
        caller_mode: SecurityMode,
    ) -> SecurityResult<TransitionKind> {
        let kind = {
            let rule = self.can_transition(pid, from, to, caller_mode)?;
            rule.kind
        };

        if let Some(domain) = self.domains.get_mut(&from) {
            domain.remove_member(pid);
        }

        let to_domain = self.domains.get_mut(&to)
            .ok_or(SecurityError::DomainNotFound(to))?;
        to_domain.add_member(pid)?;
        self.process_domain.insert(pid, to);

        Ok(kind)
    }

    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    pub fn destroy_domain(&mut self, id: DomainId) -> SecurityResult<Vec<ProcessId>> {
        let domain = self.domains.remove(&id)
            .ok_or(SecurityError::DomainNotFound(id))?;

        let displaced: Vec<ProcessId> = domain.members.iter().copied().collect();
        for &pid in &displaced {
            self.process_domain.remove(&pid);
        }

        self.transitions.retain(|r| r.from != id && r.to != id);

        Ok(displaced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts() -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(1_000_000)
    }

    #[test]
    fn test_domain_creation() {
        let domain = SecurityDomain::new(
            DomainId(0), String::from("kernel"), SecurityMode::ModePhi, 64, make_ts(),
        );
        assert_eq!(domain.id, DomainId(0));
        assert_eq!(domain.mode, SecurityMode::ModePhi);
        assert!(domain.is_empty());
    }

    #[test]
    fn test_domain_add_remove_member() {
        let mut domain = SecurityDomain::new(
            DomainId(0), String::from("test"), SecurityMode::ModeOne, 10, make_ts(),
        );
        domain.add_member(1).unwrap();
        assert!(domain.contains(1));
        assert_eq!(domain.member_count(), 1);

        assert!(domain.remove_member(1));
        assert!(!domain.contains(1));
    }

    #[test]
    fn test_domain_member_limit() {
        let mut domain = SecurityDomain::new(
            DomainId(0), String::from("small"), SecurityMode::ModeOne, 2, make_ts(),
        );
        domain.add_member(1).unwrap();
        domain.add_member(2).unwrap();
        assert!(domain.add_member(3).is_err());
    }

    #[test]
    fn test_domain_isolation() {
        let mut domain = SecurityDomain::new(
            DomainId(0), String::from("test"), SecurityMode::ModeOne, 10, make_ts(),
        );
        assert!(!domain.isolated);
        domain.set_isolated(true);
        assert!(domain.isolated);
    }

    #[test]
    fn test_domain_manager_create() {
        let mut mgr = DomainManager::new();
        let id = mgr.create_domain(
            String::from("kernel"), SecurityMode::ModePhi, 64, make_ts(),
        );
        assert_eq!(id, DomainId(0));
        assert_eq!(mgr.domain_count(), 1);
    }

    #[test]
    fn test_domain_manager_assign_process() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );
        mgr.assign_process(1, d1).unwrap();
        assert_eq!(mgr.process_domain(1), Some(d1));
        assert_eq!(mgr.process_mode(1), Some(SecurityMode::ModeOne));
    }

    #[test]
    fn test_domain_manager_reassign_process() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );
        let d2 = mgr.create_domain(
            String::from("elevated"), SecurityMode::ModePhi, 64, make_ts(),
        );

        mgr.assign_process(1, d1).unwrap();
        mgr.assign_process(1, d2).unwrap();

        assert_eq!(mgr.process_domain(1), Some(d2));
        assert_eq!(mgr.get_domain(d1).unwrap().member_count(), 0);
        assert_eq!(mgr.get_domain(d2).unwrap().member_count(), 1);
    }

    #[test]
    fn test_domain_manager_remove_process() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );
        mgr.assign_process(1, d1).unwrap();
        let removed = mgr.remove_process(1);
        assert_eq!(removed, Some(d1));
        assert_eq!(mgr.process_domain(1), None);
    }

    #[test]
    fn test_domain_transition() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );
        let d2 = mgr.create_domain(
            String::from("elevated"), SecurityMode::ModePhi, 64, make_ts(),
        );

        mgr.add_transition_rule(TransitionRule {
            from: d1, to: d2,
            kind: TransitionKind::Upgrade,
            required_mode: SecurityMode::ModePhi,
            requires_audit: true,
        });

        mgr.assign_process(1, d1).unwrap();

        let kind = mgr.execute_transition(1, d1, d2, SecurityMode::ModePhi).unwrap();
        assert_eq!(kind, TransitionKind::Upgrade);
        assert_eq!(mgr.process_domain(1), Some(d2));
    }

    #[test]
    fn test_domain_transition_insufficient_mode() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );
        let d2 = mgr.create_domain(
            String::from("kernel"), SecurityMode::ModePhi, 64, make_ts(),
        );

        mgr.add_transition_rule(TransitionRule {
            from: d1, to: d2,
            kind: TransitionKind::Upgrade,
            required_mode: SecurityMode::ModePhi,
            requires_audit: true,
        });

        mgr.assign_process(1, d1).unwrap();

        let result = mgr.execute_transition(1, d1, d2, SecurityMode::ModeOne);
        assert!(result.is_err());
    }

    #[test]
    fn test_domain_transition_isolated() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("quarantine"), SecurityMode::ModeZero, 64, make_ts(),
        );
        let d2 = mgr.create_domain(
            String::from("user"), SecurityMode::ModeOne, 64, make_ts(),
        );

        mgr.add_transition_rule(TransitionRule {
            from: d1, to: d2,
            kind: TransitionKind::Upgrade,
            required_mode: SecurityMode::ModeOne,
            requires_audit: true,
        });

        mgr.assign_process(1, d1).unwrap();
        mgr.get_domain_mut(d1).unwrap().set_isolated(true);

        let result = mgr.execute_transition(1, d1, d2, SecurityMode::ModePhi);
        assert!(result.is_err());
    }

    #[test]
    fn test_destroy_domain() {
        let mut mgr = DomainManager::new();
        let d1 = mgr.create_domain(
            String::from("temp"), SecurityMode::ModeOne, 64, make_ts(),
        );
        mgr.assign_process(1, d1).unwrap();
        mgr.assign_process(2, d1).unwrap();

        let displaced = mgr.destroy_domain(d1).unwrap();
        assert_eq!(displaced.len(), 2);
        assert_eq!(mgr.domain_count(), 0);
        assert_eq!(mgr.process_domain(1), None);
    }
}
