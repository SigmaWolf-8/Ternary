use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{IoError, IoResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PollFd(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollEvents {
    pub readable: bool,
    pub writable: bool,
    pub error: bool,
    pub hangup: bool,
}

impl PollEvents {
    pub fn none() -> Self {
        Self { readable: false, writable: false, error: false, hangup: false }
    }

    pub fn read() -> Self {
        Self { readable: true, writable: false, error: false, hangup: false }
    }

    pub fn write() -> Self {
        Self { readable: false, writable: true, error: false, hangup: false }
    }

    pub fn read_write() -> Self {
        Self { readable: true, writable: true, error: false, hangup: false }
    }

    pub fn has_events(&self) -> bool {
        self.readable || self.writable || self.error || self.hangup
    }
}

#[derive(Debug, Clone)]
pub struct PollEntry {
    pub fd: PollFd,
    pub requested: PollEvents,
    pub returned: PollEvents,
}

pub struct PollSet {
    entries: BTreeMap<u32, PollEntry>,
    max_fds: usize,
}

impl PollSet {
    pub fn new(max_fds: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_fds,
        }
    }

    pub fn add(&mut self, fd: PollFd, events: PollEvents) -> IoResult<()> {
        if self.entries.len() >= self.max_fds {
            return Err(IoError::QueueFull);
        }
        self.entries.insert(fd.0, PollEntry {
            fd,
            requested: events,
            returned: PollEvents::none(),
        });
        Ok(())
    }

    pub fn remove(&mut self, fd: PollFd) -> IoResult<()> {
        self.entries.remove(&fd.0).ok_or(IoError::DeviceNotFound)?;
        Ok(())
    }

    pub fn set_ready(&mut self, fd: PollFd, events: PollEvents) -> IoResult<()> {
        let entry = self.entries.get_mut(&fd.0).ok_or(IoError::DeviceNotFound)?;
        entry.returned.readable = events.readable && entry.requested.readable;
        entry.returned.writable = events.writable && entry.requested.writable;
        entry.returned.error = events.error;
        entry.returned.hangup = events.hangup;
        Ok(())
    }

    pub fn poll(&self) -> Vec<&PollEntry> {
        self.entries
            .values()
            .filter(|e| e.returned.has_events())
            .collect()
    }

    pub fn ready_count(&self) -> usize {
        self.entries.values().filter(|e| e.returned.has_events()).count()
    }

    pub fn clear_ready(&mut self) {
        for entry in self.entries.values_mut() {
            entry.returned = PollEvents::none();
        }
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn contains(&self, fd: PollFd) -> bool {
        self.entries.contains_key(&fd.0)
    }

    pub fn modify(&mut self, fd: PollFd, events: PollEvents) -> IoResult<()> {
        let entry = self.entries.get_mut(&fd.0).ok_or(IoError::DeviceNotFound)?;
        entry.requested = events;
        entry.returned = PollEvents::none();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_set_add() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        assert_eq!(ps.count(), 1);
        assert!(ps.contains(PollFd(1)));
    }

    #[test]
    fn test_poll_set_capacity() {
        let mut ps = PollSet::new(2);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.add(PollFd(2), PollEvents::read()).unwrap();
        assert_eq!(ps.add(PollFd(3), PollEvents::read()), Err(IoError::QueueFull));
    }

    #[test]
    fn test_poll_ready() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.add(PollFd(2), PollEvents::write()).unwrap();

        ps.set_ready(PollFd(1), PollEvents { readable: true, writable: false, error: false, hangup: false }).unwrap();
        assert_eq!(ps.ready_count(), 1);

        let ready = ps.poll();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].fd, PollFd(1));
    }

    #[test]
    fn test_poll_filtered_events() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.set_ready(PollFd(1), PollEvents::write()).unwrap();
        assert_eq!(ps.ready_count(), 0);
    }

    #[test]
    fn test_poll_error_always_reported() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.set_ready(PollFd(1), PollEvents { readable: false, writable: false, error: true, hangup: false }).unwrap();
        assert_eq!(ps.ready_count(), 1);
    }

    #[test]
    fn test_clear_ready() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.set_ready(PollFd(1), PollEvents { readable: true, writable: false, error: false, hangup: false }).unwrap();
        ps.clear_ready();
        assert_eq!(ps.ready_count(), 0);
    }

    #[test]
    fn test_modify() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.modify(PollFd(1), PollEvents::write()).unwrap();
        ps.set_ready(PollFd(1), PollEvents::write()).unwrap();
        assert_eq!(ps.ready_count(), 1);
    }

    #[test]
    fn test_remove() {
        let mut ps = PollSet::new(64);
        ps.add(PollFd(1), PollEvents::read()).unwrap();
        ps.remove(PollFd(1)).unwrap();
        assert_eq!(ps.count(), 0);
    }
}
