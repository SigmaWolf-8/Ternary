use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{IoError, IoResult};

const DEFAULT_BLOCK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockAddress {
    pub device_id: u32,
    pub block_num: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub address: BlockAddress,
    pub data: Vec<i8>,
    pub dirty: bool,
    pub access_count: u64,
    pub pinned: bool,
}

impl CacheEntry {
    pub fn new(address: BlockAddress, data: Vec<i8>) -> Self {
        Self {
            address,
            data,
            dirty: false,
            access_count: 0,
            pinned: false,
        }
    }
}

pub struct BufferCache {
    entries: BTreeMap<BlockAddress, CacheEntry>,
    capacity: usize,
    block_size: usize,
    hit_count: u64,
    miss_count: u64,
}

impl BufferCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            capacity,
            block_size: DEFAULT_BLOCK_SIZE,
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn with_block_size(capacity: usize, block_size: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            capacity,
            block_size,
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&mut self, addr: &BlockAddress) -> Option<&[i8]> {
        if let Some(entry) = self.entries.get_mut(addr) {
            entry.access_count += 1;
            self.hit_count += 1;
            Some(&entry.data)
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn insert(&mut self, addr: BlockAddress, data: Vec<i8>) -> IoResult<()> {
        if data.len() != self.block_size {
            return Err(IoError::InvalidSize);
        }

        if self.entries.len() >= self.capacity && !self.entries.contains_key(&addr) {
            self.evict_one()?;
        }

        let entry = CacheEntry::new(addr, data);
        self.entries.insert(addr, entry);
        Ok(())
    }

    pub fn mark_dirty(&mut self, addr: &BlockAddress) -> IoResult<()> {
        let entry = self.entries.get_mut(addr).ok_or(IoError::CacheMiss)?;
        entry.dirty = true;
        Ok(())
    }

    pub fn pin(&mut self, addr: &BlockAddress) -> IoResult<()> {
        let entry = self.entries.get_mut(addr).ok_or(IoError::CacheMiss)?;
        entry.pinned = true;
        Ok(())
    }

    pub fn unpin(&mut self, addr: &BlockAddress) -> IoResult<()> {
        let entry = self.entries.get_mut(addr).ok_or(IoError::CacheMiss)?;
        entry.pinned = false;
        Ok(())
    }

    pub fn flush(&mut self, addr: &BlockAddress) -> IoResult<Option<Vec<i8>>> {
        if let Some(entry) = self.entries.get_mut(addr) {
            if entry.dirty {
                entry.dirty = false;
                return Ok(Some(entry.data.clone()));
            }
            Ok(None)
        } else {
            Err(IoError::CacheMiss)
        }
    }

    pub fn flush_all(&mut self) -> Vec<(BlockAddress, Vec<i8>)> {
        let mut flushed = Vec::new();
        for (addr, entry) in self.entries.iter_mut() {
            if entry.dirty {
                flushed.push((*addr, entry.data.clone()));
                entry.dirty = false;
            }
        }
        flushed
    }

    pub fn invalidate(&mut self, addr: &BlockAddress) -> IoResult<()> {
        if let Some(entry) = self.entries.get(addr) {
            if entry.pinned {
                return Err(IoError::InvalidRequest);
            }
            if entry.dirty {
                return Err(IoError::InvalidRequest);
            }
        }
        self.entries.remove(addr);
        Ok(())
    }

    pub fn invalidate_device(&mut self, device_id: u32) -> usize {
        let to_remove: Vec<BlockAddress> = self.entries
            .iter()
            .filter(|(addr, entry)| addr.device_id == device_id && !entry.pinned && !entry.dirty)
            .map(|(addr, _)| *addr)
            .collect();
        let count = to_remove.len();
        for addr in to_remove {
            self.entries.remove(&addr);
        }
        count
    }

    fn evict_one(&mut self) -> IoResult<()> {
        let victim = self.entries
            .iter()
            .filter(|(_, e)| !e.pinned && !e.dirty)
            .min_by_key(|(_, e)| e.access_count)
            .map(|(addr, _)| *addr);

        if let Some(addr) = victim {
            self.entries.remove(&addr);
            Ok(())
        } else {
            Err(IoError::BufferFull)
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 { 0.0 } else { self.hit_count as f64 / total as f64 }
    }

    pub fn dirty_count(&self) -> usize {
        self.entries.values().filter(|e| e.dirty).count()
    }

    pub fn contains(&self, addr: &BlockAddress) -> bool {
        self.entries.contains_key(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(dev: u32, blk: u64) -> BlockAddress {
        BlockAddress { device_id: dev, block_num: blk }
    }

    fn block(val: i8) -> Vec<i8> {
        alloc::vec![val; DEFAULT_BLOCK_SIZE]
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = BufferCache::new(16);
        let a = addr(1, 0);
        cache.insert(a, block(0)).unwrap();
        assert!(cache.get(&a).is_some());
        assert_eq!(cache.size(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = BufferCache::new(16);
        assert!(cache.get(&addr(1, 0)).is_none());
    }

    #[test]
    fn test_invalid_block_size() {
        let mut cache = BufferCache::new(16);
        let result = cache.insert(addr(1, 0), alloc::vec![0i8; 100]);
        assert_eq!(result, Err(IoError::InvalidSize));
    }

    #[test]
    fn test_dirty_flush() {
        let mut cache = BufferCache::new(16);
        let a = addr(1, 0);
        cache.insert(a, block(1)).unwrap();
        cache.mark_dirty(&a).unwrap();
        assert_eq!(cache.dirty_count(), 1);

        let flushed = cache.flush(&a).unwrap();
        assert!(flushed.is_some());
        assert_eq!(cache.dirty_count(), 0);
    }

    #[test]
    fn test_flush_all() {
        let mut cache = BufferCache::new(16);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.insert(addr(1, 1), block(1)).unwrap();
        cache.mark_dirty(&addr(1, 0)).unwrap();
        cache.mark_dirty(&addr(1, 1)).unwrap();

        let flushed = cache.flush_all();
        assert_eq!(flushed.len(), 2);
        assert_eq!(cache.dirty_count(), 0);
    }

    #[test]
    fn test_eviction() {
        let mut cache = BufferCache::new(2);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.insert(addr(1, 1), block(1)).unwrap();
        cache.insert(addr(1, 2), block(2)).unwrap();
        assert_eq!(cache.size(), 2);
        assert!(cache.contains(&addr(1, 2)));
    }

    #[test]
    fn test_pinned_not_evicted() {
        let mut cache = BufferCache::new(2);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.pin(&addr(1, 0)).unwrap();
        cache.insert(addr(1, 1), block(1)).unwrap();
        cache.insert(addr(1, 2), block(2)).unwrap();
        assert!(cache.contains(&addr(1, 0)));
    }

    #[test]
    fn test_invalidate() {
        let mut cache = BufferCache::new(16);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.invalidate(&addr(1, 0)).unwrap();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_invalidate_pinned_fails() {
        let mut cache = BufferCache::new(16);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.pin(&addr(1, 0)).unwrap();
        assert_eq!(cache.invalidate(&addr(1, 0)), Err(IoError::InvalidRequest));
    }

    #[test]
    fn test_invalidate_dirty_fails() {
        let mut cache = BufferCache::new(16);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.mark_dirty(&addr(1, 0)).unwrap();
        assert_eq!(cache.invalidate(&addr(1, 0)), Err(IoError::InvalidRequest));
    }

    #[test]
    fn test_invalidate_device() {
        let mut cache = BufferCache::new(16);
        cache.insert(addr(1, 0), block(0)).unwrap();
        cache.insert(addr(1, 1), block(1)).unwrap();
        cache.insert(addr(2, 0), block(0)).unwrap();
        let removed = cache.invalidate_device(1);
        assert_eq!(removed, 2);
        assert_eq!(cache.size(), 1);
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = BufferCache::new(16);
        let a = addr(1, 0);
        cache.insert(a, block(0)).unwrap();
        cache.get(&a);
        cache.get(&a);
        cache.get(&addr(1, 99));
        assert!(cache.hit_rate() > 0.6);
    }
}
