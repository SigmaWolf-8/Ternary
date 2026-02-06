//! Process Table
//!
//! Manages the registry of all active processes in the kernel.
//! Provides creation, lookup, state transitions, and cleanup operations.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::timing::FemtosecondTimestamp;
use super::{
    ProcessDescriptor, ProcessError, ProcessId, ProcessKind,
    ProcessResult, ProcessState, Priority,
};

const DEFAULT_MAX_PROCESSES: usize = 4096;

pub struct ProcessTable {
    processes: BTreeMap<ProcessId, ProcessDescriptor>,
    next_pid: ProcessId,
    max_processes: usize,
}

impl ProcessTable {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: 0,
            max_processes: DEFAULT_MAX_PROCESSES,
        }
    }

    pub fn with_capacity(max: usize) -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: 0,
            max_processes: max,
        }
    }

    pub fn create_process(
        &mut self,
        name: String,
        parent_pid: Option<ProcessId>,
        kind: ProcessKind,
        security_mode: SecurityMode,
        priority: Priority,
        timestamp: FemtosecondTimestamp,
    ) -> ProcessResult<ProcessId> {
        if self.processes.len() >= self.max_processes {
            return Err(ProcessError::TableFull { max: self.max_processes });
        }

        if let Some(ppid) = parent_pid {
            if !self.processes.contains_key(&ppid) {
                return Err(ProcessError::NotFound(ppid));
            }
        }

        let pid = self.next_pid;
        self.next_pid += 1;

        let proc = ProcessDescriptor::new(
            pid, parent_pid, name, kind, security_mode, priority, timestamp,
        );

        self.processes.insert(pid, proc);

        if let Some(ppid) = parent_pid {
            if let Some(parent) = self.processes.get_mut(&ppid) {
                parent.children.push(pid);
            }
        }

        Ok(pid)
    }

    pub fn get(&self, pid: ProcessId) -> ProcessResult<&ProcessDescriptor> {
        self.processes.get(&pid).ok_or(ProcessError::NotFound(pid))
    }

    pub fn get_mut(&mut self, pid: ProcessId) -> ProcessResult<&mut ProcessDescriptor> {
        self.processes.get_mut(&pid).ok_or(ProcessError::NotFound(pid))
    }

    pub fn transition(
        &mut self,
        pid: ProcessId,
        new_state: ProcessState,
    ) -> ProcessResult<()> {
        let proc = self.processes.get_mut(&pid)
            .ok_or(ProcessError::NotFound(pid))?;
        proc.transition_to(new_state)
    }

    pub fn terminate(&mut self, pid: ProcessId, exit_code: i32) -> ProcessResult<()> {
        let proc = self.processes.get_mut(&pid)
            .ok_or(ProcessError::NotFound(pid))?;

        if proc.state == ProcessState::Running {
            proc.state = ProcessState::Terminated;
        } else if proc.state.can_transition_to(&ProcessState::Terminated) {
            proc.state = ProcessState::Terminated;
        } else {
            proc.state = ProcessState::Terminated;
        }

        proc.exit_code = Some(exit_code);
        Ok(())
    }

    pub fn remove(&mut self, pid: ProcessId) -> ProcessResult<ProcessDescriptor> {
        let proc = self.processes.remove(&pid)
            .ok_or(ProcessError::NotFound(pid))?;

        if let Some(ppid) = proc.parent_pid {
            if let Some(parent) = self.processes.get_mut(&ppid) {
                parent.children.retain(|&c| c != pid);
            }
        }

        Ok(proc)
    }

    pub fn reap_zombies(&mut self) -> Vec<ProcessDescriptor> {
        let zombie_pids: Vec<ProcessId> = self.processes.iter()
            .filter(|(_, p)| p.state == ProcessState::Zombie)
            .map(|(&pid, _)| pid)
            .collect();

        let mut reaped = Vec::new();
        for pid in zombie_pids {
            if let Ok(proc) = self.remove(pid) {
                reaped.push(proc);
            }
        }
        reaped
    }

    pub fn ready_processes(&self) -> Vec<ProcessId> {
        self.processes.iter()
            .filter(|(_, p)| p.state == ProcessState::Ready)
            .map(|(&pid, _)| pid)
            .collect()
    }

    pub fn processes_by_state(&self, state: ProcessState) -> Vec<ProcessId> {
        self.processes.iter()
            .filter(|(_, p)| p.state == state)
            .map(|(&pid, _)| pid)
            .collect()
    }

    pub fn processes_by_security_mode(&self, mode: SecurityMode) -> Vec<ProcessId> {
        self.processes.iter()
            .filter(|(_, p)| p.security_mode == mode)
            .map(|(&pid, _)| pid)
            .collect()
    }

    pub fn children_of(&self, pid: ProcessId) -> ProcessResult<Vec<ProcessId>> {
        let proc = self.processes.get(&pid)
            .ok_or(ProcessError::NotFound(pid))?;
        Ok(proc.children.clone())
    }

    pub fn count(&self) -> usize {
        self.processes.len()
    }

    pub fn alive_count(&self) -> usize {
        self.processes.values()
            .filter(|p| p.state.is_alive())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    pub fn contains(&self, pid: ProcessId) -> bool {
        self.processes.contains_key(&pid)
    }

    pub fn all_pids(&self) -> Vec<ProcessId> {
        self.processes.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts() -> FemtosecondTimestamp {
        FemtosecondTimestamp::new(1_000_000)
    }

    #[test]
    fn test_table_creation() {
        let table = ProcessTable::new();
        assert_eq!(table.count(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_create_process() {
        let mut table = ProcessTable::new();
        let pid = table.create_process(
            String::from("init"), None, ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, make_ts(),
        ).unwrap();
        assert_eq!(pid, 0);
        assert_eq!(table.count(), 1);
    }

    #[test]
    fn test_create_child_process() {
        let mut table = ProcessTable::new();
        let parent = table.create_process(
            String::from("init"), None, ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, make_ts(),
        ).unwrap();

        let child = table.create_process(
            String::from("worker"), Some(parent), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        assert_eq!(child, 1);
        let parent_desc = table.get(parent).unwrap();
        assert!(parent_desc.children.contains(&child));
    }

    #[test]
    fn test_create_orphan_fails() {
        let mut table = ProcessTable::new();
        let result = table.create_process(
            String::from("orphan"), Some(999), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_table_full() {
        let mut table = ProcessTable::with_capacity(2);
        table.create_process(
            String::from("p1"), None, ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, make_ts(),
        ).unwrap();
        table.create_process(
            String::from("p2"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        let result = table.create_process(
            String::from("p3"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_process() {
        let mut table = ProcessTable::new();
        let pid = table.create_process(
            String::from("test"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        let proc = table.get(pid).unwrap();
        assert_eq!(proc.name, "test");
    }

    #[test]
    fn test_get_nonexistent() {
        let table = ProcessTable::new();
        assert!(table.get(999).is_err());
    }

    #[test]
    fn test_state_transition() {
        let mut table = ProcessTable::new();
        let pid = table.create_process(
            String::from("test"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        table.transition(pid, ProcessState::Ready).unwrap();
        assert_eq!(table.get(pid).unwrap().state, ProcessState::Ready);

        table.transition(pid, ProcessState::Running).unwrap();
        assert_eq!(table.get(pid).unwrap().state, ProcessState::Running);
    }

    #[test]
    fn test_terminate_process() {
        let mut table = ProcessTable::new();
        let pid = table.create_process(
            String::from("test"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        table.terminate(pid, 0).unwrap();
        let proc = table.get(pid).unwrap();
        assert_eq!(proc.state, ProcessState::Terminated);
        assert_eq!(proc.exit_code, Some(0));
    }

    #[test]
    fn test_remove_process() {
        let mut table = ProcessTable::new();
        let parent = table.create_process(
            String::from("parent"), None, ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, make_ts(),
        ).unwrap();
        let child = table.create_process(
            String::from("child"), Some(parent), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        table.remove(child).unwrap();
        assert!(!table.contains(child));

        let parent_desc = table.get(parent).unwrap();
        assert!(!parent_desc.children.contains(&child));
    }

    #[test]
    fn test_ready_processes() {
        let mut table = ProcessTable::new();
        let p1 = table.create_process(
            String::from("p1"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        let p2 = table.create_process(
            String::from("p2"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        table.transition(p1, ProcessState::Ready).unwrap();
        table.transition(p2, ProcessState::Ready).unwrap();

        let ready = table.ready_processes();
        assert_eq!(ready.len(), 2);
        assert!(ready.contains(&p1));
        assert!(ready.contains(&p2));
    }

    #[test]
    fn test_reap_zombies() {
        let mut table = ProcessTable::new();
        let pid = table.create_process(
            String::from("zombie"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        table.transition(pid, ProcessState::Ready).unwrap();
        table.transition(pid, ProcessState::Running).unwrap();
        table.transition(pid, ProcessState::Terminated).unwrap();
        table.transition(pid, ProcessState::Zombie).unwrap();

        let reaped = table.reap_zombies();
        assert_eq!(reaped.len(), 1);
        assert_eq!(reaped[0].pid, pid);
        assert!(table.is_empty());
    }

    #[test]
    fn test_processes_by_security_mode() {
        let mut table = ProcessTable::new();
        table.create_process(
            String::from("kernel"), None, ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, make_ts(),
        ).unwrap();
        table.create_process(
            String::from("user1"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        table.create_process(
            String::from("user2"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        let phi = table.processes_by_security_mode(SecurityMode::ModePhi);
        assert_eq!(phi.len(), 1);

        let one = table.processes_by_security_mode(SecurityMode::ModeOne);
        assert_eq!(one.len(), 2);
    }

    #[test]
    fn test_alive_count() {
        let mut table = ProcessTable::new();
        let p1 = table.create_process(
            String::from("p1"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        table.create_process(
            String::from("p2"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();

        assert_eq!(table.alive_count(), 2);
        table.terminate(p1, 0).unwrap();
        assert_eq!(table.alive_count(), 1);
    }

    #[test]
    fn test_pid_monotonic() {
        let mut table = ProcessTable::new();
        let p1 = table.create_process(
            String::from("p1"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        let p2 = table.create_process(
            String::from("p2"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        let p3 = table.create_process(
            String::from("p3"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, make_ts(),
        ).unwrap();
        assert!(p2 > p1);
        assert!(p3 > p2);
    }
}
