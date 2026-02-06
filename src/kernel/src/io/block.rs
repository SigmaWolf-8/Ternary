use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{IoError, IoResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockDeviceId(pub u32);

#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    pub id: BlockDeviceId,
    pub name: String,
    pub block_size: usize,
    pub total_blocks: u64,
    pub read_only: bool,
}

impl BlockDeviceInfo {
    pub fn total_size(&self) -> u64 {
        self.total_blocks * self.block_size as u64
    }
}

pub struct BlockDevice {
    pub info: BlockDeviceInfo,
    storage: BTreeMap<u64, Vec<i8>>,
    read_count: u64,
    write_count: u64,
}

impl BlockDevice {
    pub fn new(info: BlockDeviceInfo) -> Self {
        Self {
            info,
            storage: BTreeMap::new(),
            read_count: 0,
            write_count: 0,
        }
    }

    pub fn read_block(&mut self, block_num: u64) -> IoResult<Vec<i8>> {
        if block_num >= self.info.total_blocks {
            return Err(IoError::EndOfDevice);
        }
        self.read_count += 1;
        Ok(self.storage
            .get(&block_num)
            .cloned()
            .unwrap_or_else(|| alloc::vec![0i8; self.info.block_size]))
    }

    pub fn write_block(&mut self, block_num: u64, data: &[i8]) -> IoResult<()> {
        if self.info.read_only {
            return Err(IoError::ReadOnly);
        }
        if block_num >= self.info.total_blocks {
            return Err(IoError::EndOfDevice);
        }
        if data.len() != self.info.block_size {
            return Err(IoError::InvalidSize);
        }
        self.write_count += 1;
        self.storage.insert(block_num, data.to_vec());
        Ok(())
    }

    pub fn read_blocks(&mut self, start: u64, count: u64) -> IoResult<Vec<Vec<i8>>> {
        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count {
            result.push(self.read_block(start + i)?);
        }
        Ok(result)
    }

    pub fn write_blocks(&mut self, start: u64, blocks: &[Vec<i8>]) -> IoResult<()> {
        for (i, block) in blocks.iter().enumerate() {
            self.write_block(start + i as u64, block)?;
        }
        Ok(())
    }

    pub fn trim(&mut self, block_num: u64) -> IoResult<()> {
        if block_num >= self.info.total_blocks {
            return Err(IoError::EndOfDevice);
        }
        self.storage.remove(&block_num);
        Ok(())
    }

    pub fn read_count(&self) -> u64 {
        self.read_count
    }

    pub fn write_count(&self) -> u64 {
        self.write_count
    }

    pub fn used_blocks(&self) -> usize {
        self.storage.len()
    }
}

pub struct BlockDeviceManager {
    devices: BTreeMap<u32, BlockDevice>,
    next_id: u32,
}

impl BlockDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, name: String, block_size: usize, total_blocks: u64, read_only: bool) -> BlockDeviceId {
        let id = BlockDeviceId(self.next_id);
        self.next_id += 1;
        let info = BlockDeviceInfo { id, name, block_size, total_blocks, read_only };
        self.devices.insert(id.0, BlockDevice::new(info));
        id
    }

    pub fn remove(&mut self, id: BlockDeviceId) -> IoResult<()> {
        self.devices.remove(&id.0).ok_or(IoError::DeviceNotFound)?;
        Ok(())
    }

    pub fn get(&self, id: BlockDeviceId) -> IoResult<&BlockDevice> {
        self.devices.get(&id.0).ok_or(IoError::DeviceNotFound)
    }

    pub fn get_mut(&mut self, id: BlockDeviceId) -> IoResult<&mut BlockDevice> {
        self.devices.get_mut(&id.0).ok_or(IoError::DeviceNotFound)
    }

    pub fn count(&self) -> usize {
        self.devices.len()
    }

    pub fn list(&self) -> Vec<BlockDeviceId> {
        self.devices.keys().map(|&k| BlockDeviceId(k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dev(blocks: u64) -> BlockDevice {
        BlockDevice::new(BlockDeviceInfo {
            id: BlockDeviceId(1),
            name: String::from("blk0"),
            block_size: 512,
            total_blocks: blocks,
            read_only: false,
        })
    }

    #[test]
    fn test_read_write_block() {
        let mut dev = make_dev(100);
        let data = alloc::vec![1i8; 512];
        dev.write_block(0, &data).unwrap();
        let read = dev.read_block(0).unwrap();
        assert_eq!(read, data);
    }

    #[test]
    fn test_read_unwritten_block() {
        let mut dev = make_dev(100);
        let data = dev.read_block(5).unwrap();
        assert_eq!(data, alloc::vec![0i8; 512]);
    }

    #[test]
    fn test_write_past_end() {
        let mut dev = make_dev(10);
        assert_eq!(dev.write_block(10, &alloc::vec![0i8; 512]), Err(IoError::EndOfDevice));
    }

    #[test]
    fn test_write_wrong_size() {
        let mut dev = make_dev(10);
        assert_eq!(dev.write_block(0, &alloc::vec![0i8; 256]), Err(IoError::InvalidSize));
    }

    #[test]
    fn test_read_only() {
        let mut dev = BlockDevice::new(BlockDeviceInfo {
            id: BlockDeviceId(1),
            name: String::from("rom"),
            block_size: 512,
            total_blocks: 10,
            read_only: true,
        });
        assert_eq!(dev.write_block(0, &alloc::vec![0i8; 512]), Err(IoError::ReadOnly));
    }

    #[test]
    fn test_read_write_multiple() {
        let mut dev = make_dev(100);
        let blocks = alloc::vec![alloc::vec![1i8; 512], alloc::vec![2i8; 512]];
        dev.write_blocks(5, &blocks).unwrap();
        let read = dev.read_blocks(5, 2).unwrap();
        assert_eq!(read[0], blocks[0]);
        assert_eq!(read[1], blocks[1]);
    }

    #[test]
    fn test_trim() {
        let mut dev = make_dev(100);
        dev.write_block(3, &alloc::vec![1i8; 512]).unwrap();
        assert_eq!(dev.used_blocks(), 1);
        dev.trim(3).unwrap();
        assert_eq!(dev.used_blocks(), 0);
    }

    #[test]
    fn test_io_counters() {
        let mut dev = make_dev(100);
        dev.read_block(0).unwrap();
        dev.write_block(0, &alloc::vec![0i8; 512]).unwrap();
        assert_eq!(dev.read_count(), 1);
        assert_eq!(dev.write_count(), 1);
    }

    #[test]
    fn test_block_device_manager() {
        let mut mgr = BlockDeviceManager::new();
        let id1 = mgr.create(String::from("blk0"), 512, 100, false);
        let id2 = mgr.create(String::from("blk1"), 4096, 50, false);
        assert_eq!(mgr.count(), 2);
        assert_eq!(mgr.list().len(), 2);

        mgr.get_mut(id1).unwrap().write_block(0, &alloc::vec![1i8; 512]).unwrap();
        let data = mgr.get_mut(id1).unwrap().read_block(0).unwrap();
        assert_eq!(data[0], 1);

        mgr.remove(id2).unwrap();
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_total_size() {
        let info = BlockDeviceInfo {
            id: BlockDeviceId(1),
            name: String::from("test"),
            block_size: 4096,
            total_blocks: 1000,
            read_only: false,
        };
        assert_eq!(info.total_size(), 4096 * 1000);
    }
}
