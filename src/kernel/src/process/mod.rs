//! PlenumNET Process Management Subsystem
//!
//! Provides process/task management for the Salvi Framework kernel.
//! Implements a priority-based scheduler with ternary security mode
//! integration, process lifecycle management, context switching,
//! and inter-process communication.
//!
//! # Architecture
//! - **ProcessDescriptor** - Core process metadata and state
//! - **ProcessTable** - Registry of all active processes
//! - **Scheduler** - Priority round-robin with security-mode awareness
//! - **Context** - CPU register state for context switching
//! - **IPC** - Message-passing between processes
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod table;
pub mod scheduler;
pub mod context;
pub mod ipc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::memory::SecurityMode;
use crate::timing::FemtosecondTimestamp;

pub type ProcessId = u64;

const PID_KERNEL: ProcessId = 0;
const PID_INIT: ProcessId = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Created,
    Ready,
    Running,
    Blocked,
    Sleeping,
    Suspended,
    Terminated,
    Zombie,
}

impl ProcessState {
    pub fn can_transition_to(&self, target: &ProcessState) -> bool {
        use ProcessState::*;
        matches!(
            (self, target),
            (Created, Ready)
                | (Ready, Running)
                | (Running, Ready)
                | (Running, Blocked)
                | (Running, Sleeping)
                | (Running, Suspended)
                | (Running, Terminated)
                | (Blocked, Ready)
                | (Sleeping, Ready)
                | (Suspended, Ready)
                | (Terminated, Zombie)
                | (Zombie, Terminated)
        )
    }

    pub fn is_runnable(&self) -> bool {
        matches!(self, ProcessState::Ready | ProcessState::Running)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self, ProcessState::Terminated | ProcessState::Zombie)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    RealTime = 4,
    Critical = 5,
}

impl Priority {
    pub fn time_slice_fs(&self) -> u128 {
        match self {
            Priority::Idle => 50_000_000_000_000,      // 50ms
            Priority::Low => 20_000_000_000_000,        // 20ms
            Priority::Normal => 10_000_000_000_000,     // 10ms
            Priority::High => 5_000_000_000_000,        // 5ms
            Priority::RealTime => 2_000_000_000_000,    // 2ms
            Priority::Critical => 1_000_000_000_000,    // 1ms
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessKind {
    Kernel,
    User,
    TernaryCompute,
    PhaseEncryption,
    TimingCritical,
}

#[derive(Debug, Clone)]
pub struct ProcessDescriptor {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub name: String,
    pub state: ProcessState,
    pub priority: Priority,
    pub kind: ProcessKind,
    pub security_mode: SecurityMode,
    pub created_at: FemtosecondTimestamp,
    pub cpu_time_fs: u128,
    pub context_switches: u64,
    pub exit_code: Option<i32>,
    pub children: Vec<ProcessId>,
    pub stack_base: usize,
    pub stack_size: usize,
    pub heap_base: usize,
    pub heap_size: usize,
}

impl ProcessDescriptor {
    pub fn new(
        pid: ProcessId,
        parent_pid: Option<ProcessId>,
        name: String,
        kind: ProcessKind,
        security_mode: SecurityMode,
        priority: Priority,
        created_at: FemtosecondTimestamp,
    ) -> Self {
        Self {
            pid,
            parent_pid,
            name,
            state: ProcessState::Created,
            priority,
            kind,
            security_mode,
            created_at,
            cpu_time_fs: 0,
            context_switches: 0,
            exit_code: None,
            children: Vec::new(),
            stack_base: 0,
            stack_size: 0,
            heap_base: 0,
            heap_size: 0,
        }
    }

    pub fn transition_to(&mut self, new_state: ProcessState) -> Result<(), ProcessError> {
        if !self.state.can_transition_to(&new_state) {
            return Err(ProcessError::InvalidTransition {
                pid: self.pid,
                from: self.state,
                to: new_state,
            });
        }
        self.state = new_state;
        Ok(())
    }

    pub fn add_cpu_time(&mut self, fs: u128) {
        self.cpu_time_fs += fs;
    }

    pub fn increment_context_switches(&mut self) {
        self.context_switches += 1;
    }

    pub fn set_memory_layout(&mut self, stack_base: usize, stack_size: usize, heap_base: usize, heap_size: usize) {
        self.stack_base = stack_base;
        self.stack_size = stack_size;
        self.heap_base = heap_base;
        self.heap_size = heap_size;
    }

    pub fn is_kernel(&self) -> bool {
        self.kind == ProcessKind::Kernel
    }
}

#[derive(Debug, Clone)]
pub enum ProcessError {
    NotFound(ProcessId),
    AlreadyExists(ProcessId),
    InvalidTransition { pid: ProcessId, from: ProcessState, to: ProcessState },
    SecurityViolation { pid: ProcessId, required: SecurityMode, actual: SecurityMode },
    TableFull { max: usize },
    InvalidPid(ProcessId),
    NoRunnable,
    IpcError(String),
}

impl core::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProcessError::NotFound(pid) => write!(f, "Process {} not found", pid),
            ProcessError::AlreadyExists(pid) => write!(f, "Process {} already exists", pid),
            ProcessError::InvalidTransition { pid, from, to } => {
                write!(f, "Process {}: invalid state transition {:?} -> {:?}", pid, from, to)
            }
            ProcessError::SecurityViolation { pid, .. } => {
                write!(f, "Security violation for process {}", pid)
            }
            ProcessError::TableFull { max } => write!(f, "Process table full (max {})", max),
            ProcessError::InvalidPid(pid) => write!(f, "Invalid PID: {}", pid),
            ProcessError::NoRunnable => write!(f, "No runnable processes"),
            ProcessError::IpcError(msg) => write!(f, "IPC error: {}", msg),
        }
    }
}

pub type ProcessResult<T> = core::result::Result<T, ProcessError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_state_valid_transitions() {
        assert!(ProcessState::Created.can_transition_to(&ProcessState::Ready));
        assert!(ProcessState::Ready.can_transition_to(&ProcessState::Running));
        assert!(ProcessState::Running.can_transition_to(&ProcessState::Ready));
        assert!(ProcessState::Running.can_transition_to(&ProcessState::Blocked));
        assert!(ProcessState::Running.can_transition_to(&ProcessState::Terminated));
        assert!(ProcessState::Blocked.can_transition_to(&ProcessState::Ready));
        assert!(ProcessState::Sleeping.can_transition_to(&ProcessState::Ready));
    }

    #[test]
    fn test_process_state_invalid_transitions() {
        assert!(!ProcessState::Created.can_transition_to(&ProcessState::Running));
        assert!(!ProcessState::Ready.can_transition_to(&ProcessState::Blocked));
        assert!(!ProcessState::Terminated.can_transition_to(&ProcessState::Ready));
        assert!(!ProcessState::Blocked.can_transition_to(&ProcessState::Running));
    }

    #[test]
    fn test_process_state_runnable() {
        assert!(ProcessState::Ready.is_runnable());
        assert!(ProcessState::Running.is_runnable());
        assert!(!ProcessState::Blocked.is_runnable());
        assert!(!ProcessState::Terminated.is_runnable());
    }

    #[test]
    fn test_process_state_alive() {
        assert!(ProcessState::Created.is_alive());
        assert!(ProcessState::Ready.is_alive());
        assert!(ProcessState::Running.is_alive());
        assert!(ProcessState::Blocked.is_alive());
        assert!(!ProcessState::Terminated.is_alive());
        assert!(!ProcessState::Zombie.is_alive());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::RealTime);
        assert!(Priority::RealTime > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
        assert!(Priority::Low > Priority::Idle);
    }

    #[test]
    fn test_priority_time_slices() {
        assert!(Priority::Critical.time_slice_fs() < Priority::RealTime.time_slice_fs());
        assert!(Priority::RealTime.time_slice_fs() < Priority::High.time_slice_fs());
        assert!(Priority::High.time_slice_fs() < Priority::Normal.time_slice_fs());
        assert!(Priority::Normal.time_slice_fs() < Priority::Low.time_slice_fs());
        assert!(Priority::Low.time_slice_fs() < Priority::Idle.time_slice_fs());
    }

    #[test]
    fn test_process_descriptor_creation() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let proc = ProcessDescriptor::new(
            1, Some(0), String::from("test"), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        );
        assert_eq!(proc.pid, 1);
        assert_eq!(proc.state, ProcessState::Created);
        assert_eq!(proc.priority, Priority::Normal);
        assert_eq!(proc.cpu_time_fs, 0);
    }

    #[test]
    fn test_process_descriptor_transition() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let mut proc = ProcessDescriptor::new(
            1, Some(0), String::from("test"), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        );
        assert!(proc.transition_to(ProcessState::Ready).is_ok());
        assert_eq!(proc.state, ProcessState::Ready);

        assert!(proc.transition_to(ProcessState::Terminated).is_err());
    }

    #[test]
    fn test_process_descriptor_cpu_time() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let mut proc = ProcessDescriptor::new(
            1, None, String::from("test"), ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, ts,
        );
        proc.add_cpu_time(5000);
        proc.add_cpu_time(3000);
        assert_eq!(proc.cpu_time_fs, 8000);
    }

    #[test]
    fn test_process_descriptor_context_switches() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let mut proc = ProcessDescriptor::new(
            1, None, String::from("test"), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        );
        proc.increment_context_switches();
        proc.increment_context_switches();
        assert_eq!(proc.context_switches, 2);
    }

    #[test]
    fn test_process_memory_layout() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let mut proc = ProcessDescriptor::new(
            1, None, String::from("test"), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        );
        proc.set_memory_layout(0x1000, 4096, 0x10000, 65536);
        assert_eq!(proc.stack_base, 0x1000);
        assert_eq!(proc.stack_size, 4096);
        assert_eq!(proc.heap_base, 0x10000);
        assert_eq!(proc.heap_size, 65536);
    }

    #[test]
    fn test_process_kind() {
        let ts = FemtosecondTimestamp::new(1_000_000);
        let kproc = ProcessDescriptor::new(
            0, None, String::from("kernel"), ProcessKind::Kernel,
            SecurityMode::ModePhi, Priority::Critical, ts,
        );
        assert!(kproc.is_kernel());

        let uproc = ProcessDescriptor::new(
            1, Some(0), String::from("user"), ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        );
        assert!(!uproc.is_kernel());
    }
}
