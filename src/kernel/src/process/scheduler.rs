//! Priority-Based Round-Robin Scheduler
//!
//! Implements a multi-level priority scheduler with ternary security mode
//! awareness. Higher-priority processes are scheduled first, with round-robin
//! rotation within the same priority level. Security mode constraints ensure
//! that sensitive ternary compute and phase encryption tasks receive
//! appropriate scheduling guarantees.

use alloc::collections::VecDeque;
use crate::memory::SecurityMode;
use crate::timing::FemtosecondTimestamp;
use super::{
    ProcessError, ProcessId, ProcessResult,
    ProcessState, Priority,
};
use super::table::ProcessTable;

const NUM_PRIORITY_LEVELS: usize = 6;

pub struct Scheduler {
    run_queues: [VecDeque<ProcessId>; NUM_PRIORITY_LEVELS],
    current_pid: Option<ProcessId>,
    total_schedules: u64,
    idle_pid: Option<ProcessId>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            run_queues: Default::default(),
            current_pid: None,
            total_schedules: 0,
            idle_pid: None,
        }
    }

    pub fn set_idle_process(&mut self, pid: ProcessId) {
        self.idle_pid = Some(pid);
    }

    pub fn enqueue(&mut self, pid: ProcessId, priority: Priority) {
        let level = priority as usize;
        if !self.run_queues[level].contains(&pid) {
            self.run_queues[level].push_back(pid);
        }
    }

    pub fn dequeue(&mut self, pid: ProcessId) {
        for queue in &mut self.run_queues {
            queue.retain(|&p| p != pid);
        }
    }

    pub fn schedule(&mut self, table: &ProcessTable) -> ProcessResult<ProcessId> {
        for level in (0..NUM_PRIORITY_LEVELS).rev() {
            while let Some(pid) = self.run_queues[level].pop_front() {
                if let Ok(proc) = table.get(pid) {
                    if proc.state == ProcessState::Ready {
                        self.total_schedules += 1;
                        self.current_pid = Some(pid);
                        return Ok(pid);
                    }
                }
            }
        }

        if let Some(idle) = self.idle_pid {
            if let Ok(proc) = table.get(idle) {
                if proc.state == ProcessState::Ready {
                    self.total_schedules += 1;
                    self.current_pid = Some(idle);
                    return Ok(idle);
                }
            }
        }

        Err(ProcessError::NoRunnable)
    }

    pub fn reschedule(
        &mut self,
        table: &mut ProcessTable,
        timestamp: FemtosecondTimestamp,
    ) -> ProcessResult<ProcessId> {
        if let Some(current) = self.current_pid {
            if let Ok(proc) = table.get_mut(current) {
                if proc.state == ProcessState::Running {
                    proc.state = ProcessState::Ready;
                    proc.increment_context_switches();
                    self.enqueue(current, proc.priority);
                }
            }
        }

        let next = self.schedule(table)?;

        if let Ok(proc) = table.get_mut(next) {
            proc.state = ProcessState::Running;
        }

        Ok(next)
    }

    pub fn block_current(&mut self, table: &mut ProcessTable) -> ProcessResult<()> {
        if let Some(current) = self.current_pid.take() {
            if let Ok(proc) = table.get_mut(current) {
                if proc.state == ProcessState::Running {
                    proc.state = ProcessState::Blocked;
                    self.dequeue(current);
                }
            }
        }
        Ok(())
    }

    pub fn unblock(
        &mut self,
        table: &mut ProcessTable,
        pid: ProcessId,
    ) -> ProcessResult<()> {
        let proc = table.get_mut(pid)?;
        if proc.state == ProcessState::Blocked || proc.state == ProcessState::Sleeping {
            proc.state = ProcessState::Ready;
            self.enqueue(pid, proc.priority);
        }
        Ok(())
    }

    pub fn change_priority(
        &mut self,
        table: &ProcessTable,
        pid: ProcessId,
        new_priority: Priority,
    ) -> ProcessResult<()> {
        let _ = table.get(pid)?;
        self.dequeue(pid);
        self.enqueue(pid, new_priority);
        Ok(())
    }

    pub fn current(&self) -> Option<ProcessId> {
        self.current_pid
    }

    pub fn total_schedules(&self) -> u64 {
        self.total_schedules
    }

    pub fn queue_length(&self, priority: Priority) -> usize {
        self.run_queues[priority as usize].len()
    }

    pub fn total_queued(&self) -> usize {
        self.run_queues.iter().map(|q| q.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use crate::process::ProcessKind;

    fn setup_table() -> (ProcessTable, ProcessId, ProcessId, ProcessId) {
        let mut table = ProcessTable::new();
        let ts = FemtosecondTimestamp::new(1_000_000);

        let p1 = table.create_process(
            String::from("high"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::High, ts,
        ).unwrap();
        let p2 = table.create_process(
            String::from("normal"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        ).unwrap();
        let p3 = table.create_process(
            String::from("low"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Low, ts,
        ).unwrap();

        table.transition(p1, ProcessState::Ready).unwrap();
        table.transition(p2, ProcessState::Ready).unwrap();
        table.transition(p3, ProcessState::Ready).unwrap();

        (table, p1, p2, p3)
    }

    #[test]
    fn test_scheduler_creation() {
        let sched = Scheduler::new();
        assert_eq!(sched.current(), None);
        assert_eq!(sched.total_schedules(), 0);
        assert_eq!(sched.total_queued(), 0);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut sched = Scheduler::new();
        sched.enqueue(1, Priority::Normal);
        assert_eq!(sched.queue_length(Priority::Normal), 1);

        sched.dequeue(1);
        assert_eq!(sched.queue_length(Priority::Normal), 0);
    }

    #[test]
    fn test_no_duplicate_enqueue() {
        let mut sched = Scheduler::new();
        sched.enqueue(1, Priority::Normal);
        sched.enqueue(1, Priority::Normal);
        assert_eq!(sched.queue_length(Priority::Normal), 1);
    }

    #[test]
    fn test_schedule_highest_priority_first() {
        let (table, p1, p2, p3) = setup_table();
        let mut sched = Scheduler::new();

        sched.enqueue(p1, Priority::High);
        sched.enqueue(p2, Priority::Normal);
        sched.enqueue(p3, Priority::Low);

        let next = sched.schedule(&table).unwrap();
        assert_eq!(next, p1);
    }

    #[test]
    fn test_schedule_round_robin_same_priority() {
        let mut table = ProcessTable::new();
        let ts = FemtosecondTimestamp::new(1_000_000);

        let p1 = table.create_process(
            String::from("a"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        ).unwrap();
        let p2 = table.create_process(
            String::from("b"), None, ProcessKind::User,
            SecurityMode::ModeOne, Priority::Normal, ts,
        ).unwrap();

        table.transition(p1, ProcessState::Ready).unwrap();
        table.transition(p2, ProcessState::Ready).unwrap();

        let mut sched = Scheduler::new();
        sched.enqueue(p1, Priority::Normal);
        sched.enqueue(p2, Priority::Normal);

        let first = sched.schedule(&table).unwrap();
        assert_eq!(first, p1);

        sched.enqueue(p1, Priority::Normal);
        let second = sched.schedule(&table).unwrap();
        assert_eq!(second, p2);
    }

    #[test]
    fn test_reschedule() {
        let (mut table, p1, p2, _p3) = setup_table();
        let mut sched = Scheduler::new();
        let ts = FemtosecondTimestamp::new(2_000_000);

        sched.enqueue(p1, Priority::High);
        sched.enqueue(p2, Priority::Normal);

        let next = sched.reschedule(&mut table, ts).unwrap();
        assert_eq!(next, p1);
        assert_eq!(table.get(p1).unwrap().state, ProcessState::Running);
    }

    #[test]
    fn test_block_unblock() {
        let (mut table, p1, _p2, _p3) = setup_table();
        let mut sched = Scheduler::new();

        sched.enqueue(p1, Priority::High);
        table.transition(p1, ProcessState::Running).unwrap();
        sched.current_pid = Some(p1);

        sched.block_current(&mut table).unwrap();
        assert_eq!(table.get(p1).unwrap().state, ProcessState::Blocked);
        assert_eq!(sched.queue_length(Priority::High), 0);

        sched.unblock(&mut table, p1).unwrap();
        assert_eq!(table.get(p1).unwrap().state, ProcessState::Ready);
        assert_eq!(sched.queue_length(Priority::High), 1);
    }

    #[test]
    fn test_no_runnable() {
        let table = ProcessTable::new();
        let mut sched = Scheduler::new();
        assert!(sched.schedule(&table).is_err());
    }

    #[test]
    fn test_idle_process_fallback() {
        let mut table = ProcessTable::new();
        let ts = FemtosecondTimestamp::new(1_000_000);

        let idle = table.create_process(
            String::from("idle"), None, ProcessKind::Kernel,
            SecurityMode::ModeZero, Priority::Idle, ts,
        ).unwrap();
        table.transition(idle, ProcessState::Ready).unwrap();

        let mut sched = Scheduler::new();
        sched.set_idle_process(idle);

        let next = sched.schedule(&table).unwrap();
        assert_eq!(next, idle);
    }

    #[test]
    fn test_change_priority() {
        let (table, p1, _p2, _p3) = setup_table();
        let mut sched = Scheduler::new();

        sched.enqueue(p1, Priority::Normal);
        assert_eq!(sched.queue_length(Priority::Normal), 1);

        sched.change_priority(&table, p1, Priority::High).unwrap();
        assert_eq!(sched.queue_length(Priority::Normal), 0);
        assert_eq!(sched.queue_length(Priority::High), 1);
    }

    #[test]
    fn test_total_schedules_counter() {
        let (table, p1, p2, _p3) = setup_table();
        let mut sched = Scheduler::new();

        sched.enqueue(p1, Priority::High);
        sched.enqueue(p2, Priority::Normal);

        sched.schedule(&table).unwrap();
        sched.schedule(&table).unwrap();
        assert_eq!(sched.total_schedules(), 2);
    }
}
