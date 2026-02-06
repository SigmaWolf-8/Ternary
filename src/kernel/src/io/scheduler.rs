use alloc::collections::VecDeque;
use alloc::vec::Vec;
use super::{IoError, IoResult};
use crate::device::DeviceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IoRequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    Read,
    Write,
    Flush,
    Trim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Realtime = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoRequestState {
    Pending,
    Dispatched,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub struct IoRequest {
    pub id: IoRequestId,
    pub device_id: DeviceId,
    pub operation: IoOperation,
    pub offset: u64,
    pub length: usize,
    pub priority: IoPriority,
    pub state: IoRequestState,
}

pub struct IoScheduler {
    queues: [VecDeque<IoRequest>; 4],
    next_id: u64,
    max_queue_depth: usize,
    dispatched: Vec<IoRequest>,
    completed_count: u64,
    failed_count: u64,
}

impl IoScheduler {
    pub fn new(max_queue_depth: usize) -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            next_id: 1,
            max_queue_depth,
            dispatched: Vec::new(),
            completed_count: 0,
            failed_count: 0,
        }
    }

    pub fn submit(&mut self, device_id: DeviceId, operation: IoOperation, offset: u64, length: usize, priority: IoPriority) -> IoResult<IoRequestId> {
        let queue_idx = priority as usize;
        if self.queues[queue_idx].len() >= self.max_queue_depth {
            return Err(IoError::QueueFull);
        }
        if length == 0 {
            return Err(IoError::InvalidSize);
        }

        let id = IoRequestId(self.next_id);
        self.next_id += 1;

        let req = IoRequest {
            id,
            device_id,
            operation,
            offset,
            length,
            priority,
            state: IoRequestState::Pending,
        };

        self.queues[queue_idx].push_back(req);
        Ok(id)
    }

    pub fn dispatch_next(&mut self) -> Option<IoRequest> {
        for queue_idx in (0..4).rev() {
            if let Some(mut req) = self.queues[queue_idx].pop_front() {
                req.state = IoRequestState::Dispatched;
                let ret = req.clone();
                self.dispatched.push(req);
                return Some(ret);
            }
        }
        None
    }

    pub fn complete(&mut self, id: IoRequestId) -> IoResult<()> {
        if let Some(pos) = self.dispatched.iter().position(|r| r.id == id) {
            self.dispatched.remove(pos);
            self.completed_count += 1;
            Ok(())
        } else {
            Err(IoError::InvalidRequest)
        }
    }

    pub fn fail(&mut self, id: IoRequestId) -> IoResult<()> {
        if let Some(pos) = self.dispatched.iter().position(|r| r.id == id) {
            self.dispatched.remove(pos);
            self.failed_count += 1;
            Ok(())
        } else {
            Err(IoError::InvalidRequest)
        }
    }

    pub fn pending_count(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }

    pub fn dispatched_count(&self) -> usize {
        self.dispatched.len()
    }

    pub fn completed_count(&self) -> u64 {
        self.completed_count
    }

    pub fn failed_count(&self) -> u64 {
        self.failed_count
    }

    pub fn pending_for_device(&self, device_id: DeviceId) -> usize {
        self.queues
            .iter()
            .flat_map(|q| q.iter())
            .filter(|r| r.device_id == device_id)
            .count()
    }

    pub fn cancel(&mut self, id: IoRequestId) -> IoResult<()> {
        for queue in &mut self.queues {
            if let Some(pos) = queue.iter().position(|r| r.id == id) {
                queue.remove(pos);
                return Ok(());
            }
        }
        Err(IoError::InvalidRequest)
    }

    pub fn cancel_for_device(&mut self, device_id: DeviceId) -> usize {
        let mut cancelled = 0;
        for queue in &mut self.queues {
            let before = queue.len();
            queue.retain(|r| r.device_id != device_id);
            cancelled += before - queue.len();
        }
        cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_request() {
        let mut sched = IoScheduler::new(64);
        let id = sched.submit(DeviceId(1), IoOperation::Read, 0, 4096, IoPriority::Normal).unwrap();
        assert_eq!(sched.pending_count(), 1);
        assert_eq!(id.0, 1);
    }

    #[test]
    fn test_submit_zero_length() {
        let mut sched = IoScheduler::new(64);
        assert_eq!(sched.submit(DeviceId(1), IoOperation::Read, 0, 0, IoPriority::Normal), Err(IoError::InvalidSize));
    }

    #[test]
    fn test_queue_full() {
        let mut sched = IoScheduler::new(2);
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();
        sched.submit(DeviceId(1), IoOperation::Read, 512, 512, IoPriority::Normal).unwrap();
        assert_eq!(sched.submit(DeviceId(1), IoOperation::Read, 1024, 512, IoPriority::Normal), Err(IoError::QueueFull));
    }

    #[test]
    fn test_priority_dispatch() {
        let mut sched = IoScheduler::new(64);
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Low).unwrap();
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Realtime).unwrap();
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();

        let req = sched.dispatch_next().unwrap();
        assert_eq!(req.priority, IoPriority::Realtime);
    }

    #[test]
    fn test_complete_request() {
        let mut sched = IoScheduler::new(64);
        let id = sched.submit(DeviceId(1), IoOperation::Read, 0, 4096, IoPriority::Normal).unwrap();
        sched.dispatch_next().unwrap();
        sched.complete(id).unwrap();
        assert_eq!(sched.completed_count(), 1);
        assert_eq!(sched.dispatched_count(), 0);
    }

    #[test]
    fn test_fail_request() {
        let mut sched = IoScheduler::new(64);
        let id = sched.submit(DeviceId(1), IoOperation::Write, 0, 4096, IoPriority::Normal).unwrap();
        sched.dispatch_next().unwrap();
        sched.fail(id).unwrap();
        assert_eq!(sched.failed_count(), 1);
    }

    #[test]
    fn test_cancel_request() {
        let mut sched = IoScheduler::new(64);
        let id = sched.submit(DeviceId(1), IoOperation::Read, 0, 4096, IoPriority::Normal).unwrap();
        sched.cancel(id).unwrap();
        assert_eq!(sched.pending_count(), 0);
    }

    #[test]
    fn test_cancel_for_device() {
        let mut sched = IoScheduler::new(64);
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();
        sched.submit(DeviceId(1), IoOperation::Read, 512, 512, IoPriority::High).unwrap();
        sched.submit(DeviceId(2), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();
        let cancelled = sched.cancel_for_device(DeviceId(1));
        assert_eq!(cancelled, 2);
        assert_eq!(sched.pending_count(), 1);
    }

    #[test]
    fn test_pending_for_device() {
        let mut sched = IoScheduler::new(64);
        sched.submit(DeviceId(1), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();
        sched.submit(DeviceId(2), IoOperation::Read, 0, 512, IoPriority::Normal).unwrap();
        sched.submit(DeviceId(1), IoOperation::Write, 0, 512, IoPriority::High).unwrap();
        assert_eq!(sched.pending_for_device(DeviceId(1)), 2);
    }

    #[test]
    fn test_dispatch_empty() {
        let mut sched = IoScheduler::new(64);
        assert!(sched.dispatch_next().is_none());
    }
}
