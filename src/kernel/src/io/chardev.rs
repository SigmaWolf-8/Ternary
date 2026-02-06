use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use super::{IoError, IoResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharDeviceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharDeviceMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct CharDevice {
    pub id: CharDeviceId,
    pub name: String,
    pub mode: CharDeviceMode,
    read_buf: VecDeque<i8>,
    write_buf: VecDeque<i8>,
    buf_capacity: usize,
    bytes_read: u64,
    bytes_written: u64,
}

impl CharDevice {
    pub fn new(id: CharDeviceId, name: String, mode: CharDeviceMode, buf_capacity: usize) -> Self {
        Self {
            id,
            name,
            mode,
            read_buf: VecDeque::with_capacity(buf_capacity),
            write_buf: VecDeque::with_capacity(buf_capacity),
            buf_capacity,
            bytes_read: 0,
            bytes_written: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [i8]) -> IoResult<usize> {
        if self.mode == CharDeviceMode::WriteOnly {
            return Err(IoError::WriteOnly);
        }
        let count = core::cmp::min(buf.len(), self.read_buf.len());
        for i in 0..count {
            buf[i] = self.read_buf.pop_front().unwrap();
        }
        self.bytes_read += count as u64;
        Ok(count)
    }

    pub fn write(&mut self, data: &[i8]) -> IoResult<usize> {
        if self.mode == CharDeviceMode::ReadOnly {
            return Err(IoError::ReadOnly);
        }
        let available = self.buf_capacity.saturating_sub(self.write_buf.len());
        let count = core::cmp::min(data.len(), available);
        if count == 0 && !data.is_empty() {
            return Err(IoError::BufferFull);
        }
        for &byte in &data[..count] {
            self.write_buf.push_back(byte);
        }
        self.bytes_written += count as u64;
        Ok(count)
    }

    pub fn feed_input(&mut self, data: &[i8]) -> IoResult<usize> {
        let available = self.buf_capacity.saturating_sub(self.read_buf.len());
        let count = core::cmp::min(data.len(), available);
        if count == 0 && !data.is_empty() {
            return Err(IoError::BufferFull);
        }
        for &byte in &data[..count] {
            self.read_buf.push_back(byte);
        }
        Ok(count)
    }

    pub fn drain_output(&mut self) -> Vec<i8> {
        self.write_buf.drain(..).collect()
    }

    pub fn readable(&self) -> usize {
        self.read_buf.len()
    }

    pub fn writable(&self) -> usize {
        self.buf_capacity.saturating_sub(self.write_buf.len())
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

pub struct CharDeviceManager {
    devices: BTreeMap<u32, CharDevice>,
    next_id: u32,
}

impl CharDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: String, mode: CharDeviceMode, buf_capacity: usize) -> CharDeviceId {
        let id = CharDeviceId(self.next_id);
        self.next_id += 1;
        self.devices.insert(id.0, CharDevice::new(id, name, mode, buf_capacity));
        id
    }

    pub fn remove(&mut self, id: CharDeviceId) -> IoResult<()> {
        self.devices.remove(&id.0).ok_or(IoError::DeviceNotFound)?;
        Ok(())
    }

    pub fn get(&self, id: CharDeviceId) -> IoResult<&CharDevice> {
        self.devices.get(&id.0).ok_or(IoError::DeviceNotFound)
    }

    pub fn get_mut(&mut self, id: CharDeviceId) -> IoResult<&mut CharDevice> {
        self.devices.get_mut(&id.0).ok_or(IoError::DeviceNotFound)
    }

    pub fn count(&self) -> usize {
        self.devices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_device_read_write() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 256);
        dev.feed_input(&[1i8, 0, -1]).unwrap();
        let mut buf = [0i8; 3];
        let n = dev.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(buf, [1, 0, -1]);
    }

    #[test]
    fn test_char_device_write() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 256);
        dev.write(&[1i8, 0, -1, 1]).unwrap();
        let out = dev.drain_output();
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn test_read_only() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("kbd"), CharDeviceMode::ReadOnly, 256);
        assert_eq!(dev.write(&[0i8]), Err(IoError::ReadOnly));
    }

    #[test]
    fn test_write_only() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("printer"), CharDeviceMode::WriteOnly, 256);
        let mut buf = [0i8; 1];
        assert_eq!(dev.read(&mut buf), Err(IoError::WriteOnly));
    }

    #[test]
    fn test_buffer_full() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 4);
        dev.write(&[0i8; 4]).unwrap();
        assert_eq!(dev.write(&[0i8; 1]), Err(IoError::BufferFull));
    }

    #[test]
    fn test_partial_read() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 256);
        dev.feed_input(&[1i8, 0]).unwrap();
        let mut buf = [0i8; 5];
        let n = dev.read(&mut buf).unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn test_counters() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 256);
        dev.feed_input(&[1i8, 0, -1]).unwrap();
        let mut buf = [0i8; 3];
        dev.read(&mut buf).unwrap();
        dev.write(&[0i8, 1]).unwrap();
        assert_eq!(dev.bytes_read(), 3);
        assert_eq!(dev.bytes_written(), 2);
    }

    #[test]
    fn test_readable_writable() {
        let mut dev = CharDevice::new(CharDeviceId(1), String::from("tty0"), CharDeviceMode::ReadWrite, 10);
        dev.feed_input(&[1i8; 3]).unwrap();
        dev.write(&[0i8; 4]).unwrap();
        assert_eq!(dev.readable(), 3);
        assert_eq!(dev.writable(), 6);
    }

    #[test]
    fn test_char_device_manager() {
        let mut mgr = CharDeviceManager::new();
        let id = mgr.create(String::from("tty0"), CharDeviceMode::ReadWrite, 256);
        assert_eq!(mgr.count(), 1);
        mgr.get_mut(id).unwrap().write(&[1i8]).unwrap();
        mgr.remove(id).unwrap();
        assert_eq!(mgr.count(), 0);
    }
}
