//! Heap Allocator
//!
//! Implements a free-list based heap allocator for dynamic memory allocation
//! in the PlenumNET kernel. Supports standard allocation/deallocation with
//! coalescing of adjacent free blocks to reduce fragmentation.

use super::{MemoryError, MemoryResult};
use alloc::vec::Vec;
use core::cmp;

const MIN_BLOCK_SIZE: usize = 16;
const ALIGNMENT: usize = 8;

#[derive(Debug, Clone, Copy)]
struct BlockHeader {
    size: usize,
    is_free: bool,
}

impl BlockHeader {
    const SIZE: usize = core::mem::size_of::<Self>();
}

pub struct HeapAllocator {
    base: usize,
    size: usize,
    blocks: Vec<(usize, BlockHeader)>,
    total_allocated: usize,
    allocation_count: usize,
}

impl HeapAllocator {
    pub fn new(base: usize, size: usize) -> Self {
        let usable_size = size - BlockHeader::SIZE;
        let initial_block = BlockHeader {
            size: usable_size,
            is_free: true,
        };

        Self {
            base,
            size,
            blocks: alloc::vec![(base, initial_block)],
            total_allocated: 0,
            allocation_count: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> MemoryResult<usize> {
        if size == 0 {
            return Err(MemoryError::InvalidAlignment(0));
        }

        let aligned_size = align_up(cmp::max(size, MIN_BLOCK_SIZE), ALIGNMENT);

        let block_idx = self.find_best_fit(aligned_size);

        match block_idx {
            Some(idx) => {
                let (addr, block) = self.blocks[idx];
                let remaining = block.size - aligned_size;

                self.blocks[idx].1.size = aligned_size;
                self.blocks[idx].1.is_free = false;

                if remaining >= MIN_BLOCK_SIZE + BlockHeader::SIZE {
                    let new_addr = addr + BlockHeader::SIZE + aligned_size;
                    let new_block = BlockHeader {
                        size: remaining - BlockHeader::SIZE,
                        is_free: true,
                    };
                    self.blocks.insert(idx + 1, (new_addr, new_block));
                }

                self.total_allocated += aligned_size;
                self.allocation_count += 1;

                Ok(addr + BlockHeader::SIZE)
            }
            None => Err(MemoryError::OutOfMemory),
        }
    }

    pub fn deallocate(&mut self, ptr: usize) -> MemoryResult<()> {
        let block_addr = ptr - BlockHeader::SIZE;

        let idx = self.blocks.iter()
            .position(|(addr, _)| *addr == block_addr)
            .ok_or(MemoryError::InvalidAddress(ptr))?;

        if self.blocks[idx].1.is_free {
            return Err(MemoryError::DoubleFree(ptr));
        }

        self.total_allocated -= self.blocks[idx].1.size;
        self.allocation_count -= 1;
        self.blocks[idx].1.is_free = true;

        self.coalesce(idx);

        Ok(())
    }

    fn find_best_fit(&self, size: usize) -> Option<usize> {
        let mut best: Option<(usize, usize)> = None;

        for (idx, (_addr, block)) in self.blocks.iter().enumerate() {
            if block.is_free && block.size >= size {
                match best {
                    None => best = Some((idx, block.size)),
                    Some((_, best_size)) if block.size < best_size => {
                        best = Some((idx, block.size));
                    }
                    _ => {}
                }
            }
        }

        best.map(|(idx, _)| idx)
    }

    fn coalesce(&mut self, idx: usize) {
        if idx + 1 < self.blocks.len() && self.blocks[idx + 1].1.is_free {
            let next_size = self.blocks[idx + 1].1.size + BlockHeader::SIZE;
            self.blocks[idx].1.size += next_size;
            self.blocks.remove(idx + 1);
        }

        if idx > 0 && self.blocks[idx - 1].1.is_free {
            let current_size = self.blocks[idx].1.size + BlockHeader::SIZE;
            self.blocks[idx - 1].1.size += current_size;
            self.blocks.remove(idx);
        }
    }

    pub fn total_size(&self) -> usize {
        self.size
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn total_free(&self) -> usize {
        self.blocks.iter()
            .filter(|(_, b)| b.is_free)
            .map(|(_, b)| b.size)
            .sum()
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn fragmentation_ratio(&self) -> f64 {
        let free_blocks: Vec<&(usize, BlockHeader)> = self.blocks.iter()
            .filter(|(_, b)| b.is_free)
            .collect();

        if free_blocks.is_empty() {
            return 0.0;
        }

        let largest_free = free_blocks.iter()
            .map(|(_, b)| b.size)
            .max()
            .unwrap_or(0);

        let total_free = self.total_free();

        if total_free == 0 {
            return 0.0;
        }

        1.0 - (largest_free as f64 / total_free as f64)
    }
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_creation() {
        let heap = HeapAllocator::new(0x10000, 65536);
        assert_eq!(heap.total_size(), 65536);
        assert_eq!(heap.allocation_count(), 0);
        assert_eq!(heap.block_count(), 1);
    }

    #[test]
    fn test_heap_allocate() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let ptr = heap.allocate(64).unwrap();
        assert!(ptr > 0x10000);
        assert_eq!(heap.allocation_count(), 1);
    }

    #[test]
    fn test_heap_allocate_multiple() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(64).unwrap();
        let p2 = heap.allocate(128).unwrap();
        let p3 = heap.allocate(256).unwrap();
        assert!(p2 > p1);
        assert!(p3 > p2);
        assert_eq!(heap.allocation_count(), 3);
    }

    #[test]
    fn test_heap_deallocate() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let ptr = heap.allocate(64).unwrap();
        heap.deallocate(ptr).unwrap();
        assert_eq!(heap.allocation_count(), 0);
    }

    #[test]
    fn test_heap_double_free() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let ptr = heap.allocate(64).unwrap();
        heap.deallocate(ptr).unwrap();
        assert!(heap.deallocate(ptr).is_err());
    }

    #[test]
    fn test_heap_coalesce() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(64).unwrap();
        let p2 = heap.allocate(64).unwrap();
        let p3 = heap.allocate(64).unwrap();
        let blocks_after_alloc = heap.block_count();

        heap.deallocate(p2).unwrap();
        heap.deallocate(p1).unwrap();

        assert!(heap.block_count() <= blocks_after_alloc);
    }

    #[test]
    fn test_heap_coalesce_all() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(64).unwrap();
        let p2 = heap.allocate(64).unwrap();

        heap.deallocate(p1).unwrap();
        heap.deallocate(p2).unwrap();

        assert_eq!(heap.block_count(), 1);
    }

    #[test]
    fn test_heap_reuse_freed_memory() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(64).unwrap();
        heap.deallocate(p1).unwrap();

        let p2 = heap.allocate(32).unwrap();
        assert!(p2 > 0);
        assert_eq!(heap.allocation_count(), 1);
    }

    #[test]
    fn test_heap_zero_size() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        assert!(heap.allocate(0).is_err());
    }

    #[test]
    fn test_heap_alignment() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(7).unwrap();
        let p2 = heap.allocate(13).unwrap();
        assert_eq!(p1 % ALIGNMENT, 0);
        assert_eq!(p2 % ALIGNMENT, 0);
    }

    #[test]
    fn test_heap_fragmentation_zero() {
        let heap = HeapAllocator::new(0x10000, 65536);
        assert!((heap.fragmentation_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heap_fragmentation_with_holes() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        let p1 = heap.allocate(64).unwrap();
        let _p2 = heap.allocate(64).unwrap();
        let p3 = heap.allocate(64).unwrap();

        heap.deallocate(p1).unwrap();
        heap.deallocate(p3).unwrap();

        assert!(heap.fragmentation_ratio() > 0.0);
    }

    #[test]
    fn test_heap_stats() {
        let mut heap = HeapAllocator::new(0x10000, 65536);
        heap.allocate(256).unwrap();
        heap.allocate(512).unwrap();

        assert!(heap.total_allocated() > 0);
        assert!(heap.total_free() > 0);
        assert!(heap.total_allocated() + heap.total_free() <= heap.total_size());
    }

    #[test]
    fn test_heap_best_fit() {
        let mut heap = HeapAllocator::new(0x10000, 65536);

        let p1 = heap.allocate(100).unwrap();
        let _p2 = heap.allocate(200).unwrap();
        let p3 = heap.allocate(50).unwrap();
        let _p4 = heap.allocate(300).unwrap();

        heap.deallocate(p1).unwrap();
        heap.deallocate(p3).unwrap();

        let p5 = heap.allocate(40).unwrap();
        assert!(p5 > 0);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(7, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(17, 16), 32);
    }
}
