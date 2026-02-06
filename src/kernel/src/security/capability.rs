//! Capability-Based Access Control
//!
//! Implements capability tokens for fine-grained access control.
//! Each capability token grants specific permissions on a specific resource
//! and is bound to a process and security mode.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::process::ProcessId;
use crate::timing::FemtosecondTimestamp;
use super::{
    Action, CapabilityKind, ResourceId, SecurityError, SecurityResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenId(pub u64);

#[derive(Debug, Clone)]
pub struct CapabilityToken {
    pub id: TokenId,
    pub owner: ProcessId,
    pub kind: CapabilityKind,
    pub resource: ResourceId,
    pub actions: Vec<Action>,
    pub mode: SecurityMode,
    pub created_at: FemtosecondTimestamp,
    pub expires_at: Option<FemtosecondTimestamp>,
    pub revoked: bool,
    pub delegatable: bool,
    pub parent_token: Option<TokenId>,
}

impl CapabilityToken {
    pub fn is_valid(&self, now: FemtosecondTimestamp) -> bool {
        if self.revoked {
            return false;
        }
        if let Some(expires) = self.expires_at {
            if now.femtoseconds >= expires.femtoseconds {
                return false;
            }
        }
        true
    }

    pub fn permits(&self, action: &Action) -> bool {
        self.actions.contains(action)
    }

    pub fn check_access(
        &self,
        resource: ResourceId,
        action: &Action,
        caller_mode: SecurityMode,
        now: FemtosecondTimestamp,
    ) -> SecurityResult<()> {
        if !self.is_valid(now) {
            if self.revoked {
                return Err(SecurityError::CapabilityRevoked { token_id: self.id.0 });
            }
            return Err(SecurityError::CapabilityExpired { token_id: self.id.0 });
        }

        if self.resource != resource {
            return Err(SecurityError::AccessDenied {
                subject: self.owner,
                resource,
                action: *action,
            });
        }

        if !self.permits(action) {
            return Err(SecurityError::AccessDenied {
                subject: self.owner,
                resource,
                action: *action,
            });
        }

        if !caller_mode.can_access(&self.mode) {
            return Err(SecurityError::InsufficientCapability {
                required: self.kind,
                held: None,
            });
        }

        Ok(())
    }
}

pub struct CapabilityManager {
    tokens: BTreeMap<TokenId, CapabilityToken>,
    process_tokens: BTreeMap<ProcessId, Vec<TokenId>>,
    next_token_id: u64,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self {
            tokens: BTreeMap::new(),
            process_tokens: BTreeMap::new(),
            next_token_id: 0,
        }
    }

    pub fn grant(
        &mut self,
        owner: ProcessId,
        kind: CapabilityKind,
        resource: ResourceId,
        actions: Vec<Action>,
        mode: SecurityMode,
        granter_mode: SecurityMode,
        now: FemtosecondTimestamp,
        expires_at: Option<FemtosecondTimestamp>,
        delegatable: bool,
    ) -> SecurityResult<TokenId> {
        if !granter_mode.can_access(&kind.minimum_mode()) {
            return Err(SecurityError::InsufficientCapability {
                required: kind,
                held: None,
            });
        }

        let id = TokenId(self.next_token_id);
        self.next_token_id += 1;

        let token = CapabilityToken {
            id,
            owner,
            kind,
            resource,
            actions,
            mode,
            created_at: now,
            expires_at,
            revoked: false,
            delegatable,
            parent_token: None,
        };

        self.tokens.insert(id, token);
        self.process_tokens.entry(owner).or_insert_with(Vec::new).push(id);

        Ok(id)
    }

    pub fn delegate(
        &mut self,
        parent_id: TokenId,
        new_owner: ProcessId,
        now: FemtosecondTimestamp,
    ) -> SecurityResult<TokenId> {
        let parent = self.tokens.get(&parent_id)
            .ok_or(SecurityError::InvalidToken)?;

        if !parent.is_valid(now) {
            return Err(SecurityError::CapabilityExpired { token_id: parent_id.0 });
        }

        if !parent.delegatable {
            return Err(SecurityError::PolicyViolation(
                alloc::string::String::from("Token is not delegatable"),
            ));
        }

        let child_id = TokenId(self.next_token_id);
        self.next_token_id += 1;

        let child = CapabilityToken {
            id: child_id,
            owner: new_owner,
            kind: parent.kind,
            resource: parent.resource,
            actions: parent.actions.clone(),
            mode: parent.mode,
            created_at: now,
            expires_at: parent.expires_at,
            revoked: false,
            delegatable: false,
            parent_token: Some(parent_id),
        };

        self.tokens.insert(child_id, child);
        self.process_tokens.entry(new_owner).or_insert_with(Vec::new).push(child_id);

        Ok(child_id)
    }

    pub fn revoke(&mut self, token_id: TokenId) -> SecurityResult<()> {
        let token = self.tokens.get_mut(&token_id)
            .ok_or(SecurityError::InvalidToken)?;
        token.revoked = true;

        let children: Vec<TokenId> = self.tokens.iter()
            .filter(|(_, t)| t.parent_token == Some(token_id))
            .map(|(&id, _)| id)
            .collect();

        for child_id in children {
            if let Some(child) = self.tokens.get_mut(&child_id) {
                child.revoked = true;
            }
        }

        Ok(())
    }

    pub fn check_access(
        &self,
        pid: ProcessId,
        resource: ResourceId,
        action: &Action,
        caller_mode: SecurityMode,
        now: FemtosecondTimestamp,
    ) -> SecurityResult<()> {
        let token_ids = self.process_tokens.get(&pid)
            .ok_or(SecurityError::AccessDenied { subject: pid, resource, action: *action })?;

        for tid in token_ids {
            if let Some(token) = self.tokens.get(tid) {
                if token.check_access(resource, action, caller_mode, now).is_ok() {
                    return Ok(());
                }
            }
        }

        Err(SecurityError::AccessDenied { subject: pid, resource, action: *action })
    }

    pub fn get_token(&self, id: TokenId) -> Option<&CapabilityToken> {
        self.tokens.get(&id)
    }

    pub fn process_tokens(&self, pid: ProcessId) -> Vec<TokenId> {
        self.process_tokens.get(&pid).cloned().unwrap_or_default()
    }

    pub fn cleanup_process(&mut self, pid: ProcessId) {
        if let Some(token_ids) = self.process_tokens.remove(&pid) {
            for tid in token_ids {
                self.tokens.remove(&tid);
            }
        }
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn active_tokens(&self, now: FemtosecondTimestamp) -> usize {
        self.tokens.values().filter(|t| t.is_valid(now)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn make_ts() -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(1_000_000)
    }

    fn make_ts_at(fs: u128) -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(fs)
    }

    #[test]
    fn test_grant_capability() {
        let mut mgr = CapabilityManager::new();
        let tid = mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read, Action::Write],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        assert_eq!(tid, TokenId(0));
        assert_eq!(mgr.token_count(), 1);
    }

    #[test]
    fn test_grant_insufficient_mode() {
        let mut mgr = CapabilityManager::new();
        let result = mgr.grant(
            1, CapabilityKind::Security, ResourceId(10),
            vec![Action::ModifyPolicy],
            SecurityMode::ModePhi, SecurityMode::ModeOne,
            make_ts(), None, false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_granted() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read, Action::Write],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        let result = mgr.check_access(
            1, ResourceId(10), &Action::Read,
            SecurityMode::ModeOne, make_ts(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_access_wrong_action() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        let result = mgr.check_access(
            1, ResourceId(10), &Action::Write,
            SecurityMode::ModeOne, make_ts(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_wrong_resource() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        let result = mgr.check_access(
            1, ResourceId(99), &Action::Read,
            SecurityMode::ModeOne, make_ts(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiry() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts_at(1000), Some(make_ts_at(5000)), false,
        ).unwrap();

        let ok = mgr.check_access(
            1, ResourceId(10), &Action::Read,
            SecurityMode::ModeOne, make_ts_at(3000),
        );
        assert!(ok.is_ok());

        let expired = mgr.check_access(
            1, ResourceId(10), &Action::Read,
            SecurityMode::ModeOne, make_ts_at(6000),
        );
        assert!(expired.is_err());
    }

    #[test]
    fn test_revoke_token() {
        let mut mgr = CapabilityManager::new();
        let tid = mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        mgr.revoke(tid).unwrap();

        let result = mgr.check_access(
            1, ResourceId(10), &Action::Read,
            SecurityMode::ModeOne, make_ts(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delegate_token() {
        let mut mgr = CapabilityManager::new();
        let parent_tid = mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read, Action::Write],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, true,
        ).unwrap();

        let child_tid = mgr.delegate(parent_tid, 2, make_ts()).unwrap();

        let result = mgr.check_access(
            2, ResourceId(10), &Action::Read,
            SecurityMode::ModeOne, make_ts(),
        );
        assert!(result.is_ok());

        let child = mgr.get_token(child_tid).unwrap();
        assert!(!child.delegatable);
        assert_eq!(child.parent_token, Some(parent_tid));
    }

    #[test]
    fn test_delegate_non_delegatable() {
        let mut mgr = CapabilityManager::new();
        let tid = mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        let result = mgr.delegate(tid, 2, make_ts());
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_cascades_to_children() {
        let mut mgr = CapabilityManager::new();
        let parent_tid = mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, true,
        ).unwrap();

        let child_tid = mgr.delegate(parent_tid, 2, make_ts()).unwrap();

        mgr.revoke(parent_tid).unwrap();

        let child = mgr.get_token(child_tid).unwrap();
        assert!(child.revoked);
    }

    #[test]
    fn test_cleanup_process() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();
        mgr.grant(
            1, CapabilityKind::Ipc, ResourceId(20),
            vec![Action::IpcSend],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts(), None, false,
        ).unwrap();

        assert_eq!(mgr.token_count(), 2);
        mgr.cleanup_process(1);
        assert_eq!(mgr.token_count(), 0);
    }

    #[test]
    fn test_active_tokens_count() {
        let mut mgr = CapabilityManager::new();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(10),
            vec![Action::Read],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts_at(1000), Some(make_ts_at(5000)), false,
        ).unwrap();
        mgr.grant(
            1, CapabilityKind::Memory, ResourceId(20),
            vec![Action::Write],
            SecurityMode::ModeOne, SecurityMode::ModePhi,
            make_ts_at(1000), None, false,
        ).unwrap();

        assert_eq!(mgr.active_tokens(make_ts_at(3000)), 2);
        assert_eq!(mgr.active_tokens(make_ts_at(6000)), 1);
    }
}
