use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{DeviceId, DeviceError, DeviceResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DmaChannelId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    MemoryToDevice,
    DeviceToMemory,
    MemoryToMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaTransferMode {
    Single,
    Block,
    Burst,
    Scatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaChannelState {
    Free,
    Allocated,
    Configured,
    Active,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
pub struct DmaRequest {
    pub source_addr: u64,
    pub dest_addr: u64,
    pub length: usize,
    pub direction: DmaDirection,
    pub mode: DmaTransferMode,
    pub device_id: DeviceId,
}

#[derive(Debug, Clone)]
pub struct DmaChannel {
    pub id: DmaChannelId,
    pub state: DmaChannelState,
    pub owner: Option<DeviceId>,
    pub current_request: Option<DmaRequest>,
    pub bytes_transferred: u64,
    pub transfer_count: u64,
    pub priority: u8,
}

impl DmaChannel {
    pub fn new(id: DmaChannelId) -> Self {
        Self {
            id,
            state: DmaChannelState::Free,
            owner: None,
            current_request: None,
            bytes_transferred: 0,
            transfer_count: 0,
            priority: 0,
        }
    }
}

pub struct DmaController {
    channels: BTreeMap<u8, DmaChannel>,
    max_channels: u8,
    max_transfer_size: usize,
}

impl DmaController {
    pub fn new(num_channels: u8, max_transfer_size: usize) -> Self {
        let mut channels = BTreeMap::new();
        for i in 0..num_channels {
            channels.insert(i, DmaChannel::new(DmaChannelId(i)));
        }
        Self {
            channels,
            max_channels: num_channels,
            max_transfer_size,
        }
    }

    pub fn allocate_channel(&mut self, device_id: DeviceId) -> DeviceResult<DmaChannelId> {
        for (_, ch) in self.channels.iter_mut() {
            if ch.state == DmaChannelState::Free {
                ch.state = DmaChannelState::Allocated;
                ch.owner = Some(device_id);
                return Ok(ch.id);
            }
        }
        Err(DeviceError::CapacityExceeded)
    }

    pub fn release_channel(&mut self, channel_id: DmaChannelId) -> DeviceResult<()> {
        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        if ch.state == DmaChannelState::Active {
            return Err(DeviceError::InvalidState);
        }
        ch.state = DmaChannelState::Free;
        ch.owner = None;
        ch.current_request = None;
        Ok(())
    }

    pub fn configure_transfer(&mut self, channel_id: DmaChannelId, request: DmaRequest) -> DeviceResult<()> {
        if request.length > self.max_transfer_size {
            return Err(DeviceError::InvalidParameter);
        }
        if request.length == 0 {
            return Err(DeviceError::InvalidParameter);
        }

        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        if ch.state != DmaChannelState::Allocated && ch.state != DmaChannelState::Complete {
            return Err(DeviceError::InvalidState);
        }
        if ch.owner != Some(request.device_id) {
            return Err(DeviceError::InvalidParameter);
        }

        ch.current_request = Some(request);
        ch.state = DmaChannelState::Configured;
        Ok(())
    }

    pub fn start_transfer(&mut self, channel_id: DmaChannelId) -> DeviceResult<()> {
        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        if ch.state != DmaChannelState::Configured {
            return Err(DeviceError::InvalidState);
        }
        ch.state = DmaChannelState::Active;
        Ok(())
    }

    pub fn complete_transfer(&mut self, channel_id: DmaChannelId) -> DeviceResult<u64> {
        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        if ch.state != DmaChannelState::Active {
            return Err(DeviceError::InvalidState);
        }

        let transferred = ch.current_request.as_ref().map_or(0, |r| r.length as u64);
        ch.bytes_transferred += transferred;
        ch.transfer_count += 1;
        ch.state = DmaChannelState::Complete;
        Ok(transferred)
    }

    pub fn abort_transfer(&mut self, channel_id: DmaChannelId) -> DeviceResult<()> {
        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        if ch.state != DmaChannelState::Active {
            return Err(DeviceError::InvalidState);
        }
        ch.state = DmaChannelState::Error;
        Ok(())
    }

    pub fn get_channel(&self, channel_id: DmaChannelId) -> DeviceResult<&DmaChannel> {
        self.channels.get(&channel_id.0).ok_or(DeviceError::NotFound)
    }

    pub fn free_channels(&self) -> usize {
        self.channels.values().filter(|c| c.state == DmaChannelState::Free).count()
    }

    pub fn active_channels(&self) -> usize {
        self.channels.values().filter(|c| c.state == DmaChannelState::Active).count()
    }

    pub fn total_channels(&self) -> u8 {
        self.max_channels
    }

    pub fn channels_for_device(&self, device_id: DeviceId) -> Vec<DmaChannelId> {
        self.channels
            .values()
            .filter(|c| c.owner == Some(device_id))
            .map(|c| c.id)
            .collect()
    }

    pub fn set_priority(&mut self, channel_id: DmaChannelId, priority: u8) -> DeviceResult<()> {
        let ch = self.channels.get_mut(&channel_id.0).ok_or(DeviceError::NotFound)?;
        ch.priority = priority;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(device_id: DeviceId, len: usize) -> DmaRequest {
        DmaRequest {
            source_addr: 0x1000,
            dest_addr: 0x2000,
            length: len,
            direction: DmaDirection::MemoryToDevice,
            mode: DmaTransferMode::Block,
            device_id,
        }
    }

    #[test]
    fn test_allocate_channel() {
        let mut dma = DmaController::new(4, 65536);
        assert_eq!(dma.free_channels(), 4);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        assert_eq!(dma.free_channels(), 3);
        assert_eq!(dma.get_channel(ch).unwrap().owner, Some(DeviceId(1)));
    }

    #[test]
    fn test_allocate_all_channels() {
        let mut dma = DmaController::new(2, 65536);
        dma.allocate_channel(DeviceId(1)).unwrap();
        dma.allocate_channel(DeviceId(2)).unwrap();
        assert_eq!(dma.allocate_channel(DeviceId(3)), Err(DeviceError::CapacityExceeded));
    }

    #[test]
    fn test_release_channel() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        dma.release_channel(ch).unwrap();
        assert_eq!(dma.free_channels(), 4);
    }

    #[test]
    fn test_full_transfer_cycle() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        let req = make_request(DeviceId(1), 4096);

        dma.configure_transfer(ch, req).unwrap();
        dma.start_transfer(ch).unwrap();
        assert_eq!(dma.active_channels(), 1);

        let bytes = dma.complete_transfer(ch).unwrap();
        assert_eq!(bytes, 4096);
        assert_eq!(dma.active_channels(), 0);
        assert_eq!(dma.get_channel(ch).unwrap().transfer_count, 1);
    }

    #[test]
    fn test_transfer_too_large() {
        let mut dma = DmaController::new(4, 4096);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        let req = make_request(DeviceId(1), 8192);
        assert_eq!(dma.configure_transfer(ch, req), Err(DeviceError::InvalidParameter));
    }

    #[test]
    fn test_transfer_zero_length() {
        let mut dma = DmaController::new(4, 4096);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        let req = make_request(DeviceId(1), 0);
        assert_eq!(dma.configure_transfer(ch, req), Err(DeviceError::InvalidParameter));
    }

    #[test]
    fn test_wrong_owner() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        let req = make_request(DeviceId(2), 1024);
        assert_eq!(dma.configure_transfer(ch, req), Err(DeviceError::InvalidParameter));
    }

    #[test]
    fn test_abort_transfer() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        dma.configure_transfer(ch, make_request(DeviceId(1), 1024)).unwrap();
        dma.start_transfer(ch).unwrap();
        dma.abort_transfer(ch).unwrap();
        assert_eq!(dma.get_channel(ch).unwrap().state, DmaChannelState::Error);
    }

    #[test]
    fn test_release_active_channel() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        dma.configure_transfer(ch, make_request(DeviceId(1), 1024)).unwrap();
        dma.start_transfer(ch).unwrap();
        assert_eq!(dma.release_channel(ch), Err(DeviceError::InvalidState));
    }

    #[test]
    fn test_channels_for_device() {
        let mut dma = DmaController::new(4, 65536);
        dma.allocate_channel(DeviceId(1)).unwrap();
        dma.allocate_channel(DeviceId(1)).unwrap();
        dma.allocate_channel(DeviceId(2)).unwrap();
        assert_eq!(dma.channels_for_device(DeviceId(1)).len(), 2);
    }

    #[test]
    fn test_set_priority() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();
        dma.set_priority(ch, 5).unwrap();
        assert_eq!(dma.get_channel(ch).unwrap().priority, 5);
    }

    #[test]
    fn test_multiple_transfers() {
        let mut dma = DmaController::new(4, 65536);
        let ch = dma.allocate_channel(DeviceId(1)).unwrap();

        dma.configure_transfer(ch, make_request(DeviceId(1), 1024)).unwrap();
        dma.start_transfer(ch).unwrap();
        dma.complete_transfer(ch).unwrap();

        dma.configure_transfer(ch, make_request(DeviceId(1), 2048)).unwrap();
        dma.start_transfer(ch).unwrap();
        dma.complete_transfer(ch).unwrap();

        let channel = dma.get_channel(ch).unwrap();
        assert_eq!(channel.transfer_count, 2);
        assert_eq!(channel.bytes_transferred, 3072);
    }
}
