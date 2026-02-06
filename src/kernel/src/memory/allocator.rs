//! Physical Frame Allocator
//!
//! Implements a bitmap-based physical frame allocator for the PlenumNET kernel.
//! Supports both standard binary pages (4KiB) and ternary-aligned pages (2187 bytes).
//!
//! The allocator tracks physical memory frames using a compact bitmap where each
//! bit represents one frame. It supports allocation, deallocation, and contiguous
//! multi-frame allocation for DMA and ternary compute buffers.

use super::{MemoryError, MemoryResult, MemoryStats, DEFAULT_PAGE_SIZE};
use alloc::vec::Vec;

const BITMAP_ENTRY_BITS: usize = 64;

pub struct FrameAllocator {
    bitmap: Vec<u64>,
    total_frames: usize,
    free_frames: usize,
    page_size: usize,
    base_address: usize,
}

impl FrameAllocator {
    pub fn new(memory_size: usize, page_size: usize, base_address: usize) -> Self {
        let total_frames = memory_size / page_size;
        let bitmap_entries = (total_frames + BITMAP_ENTRY_BITS - 1) / BITMAP_ENTRY_BITS;

        Self {
            bitmap: alloc::vec![0u64; bitmap_entries],
            total_frames,
            free_frames: total_frames,
            page_size,
            base_address,
        }
    }

    pub fn with_default_page_size(memory_size: usize) -> Self {
        Self::new(memory_size, DEFAULT_PAGE_SIZE, 0)
    }

    pub fn allocate_frame(&mut self) -> MemoryResult<usize> {
        for (entry_idx, entry) in self.bitmap.iter_mut().enumerate() {
            if *entry != u64::MAX {
                let bit = (!*entry).trailing_zeros() as usize;
                let frame_idx = entry_idx * BITMAP_ENTRY_BITS + bit;

                if frame_idx >= self.total_frames {
                    break;
                }

                *entry |= 1u64 << bit;
                self.free_frames -= 1;

                return Ok(self.base_address + frame_idx * self.page_size);
            }
        }
        Err(MemoryError::FrameExhausted)
    }

    pub fn allocate_contiguous(&mut self, count: usize) -> MemoryResult<usize> {
        if count == 0 {
            return Err(MemoryError::InvalidAlignment(0));
        }
        if count == 1 {
            return self.allocate_frame();
        }

        let mut run_start = 0;
        let mut run_length = 0;

        for frame_idx in 0..self.total_frames {
            if self.is_frame_free(frame_idx) {
                if run_length == 0 {
                    run_start = frame_idx;
                }
                run_length += 1;

                if run_length == count {
                    for i in run_start..(run_start + count) {
                        self.mark_frame_used(i);
                    }
                    self.free_frames -= count;
                    return Ok(self.base_address + run_start * self.page_size);
                }
            } else {
                run_length = 0;
            }
        }

        Err(MemoryError::OutOfMemory)
    }

    pub fn deallocate_frame(&mut self, address: usize) -> MemoryResult<()> {
        let frame_idx = self.address_to_frame(address)?;

        if self.is_frame_free(frame_idx) {
            return Err(MemoryError::DoubleFree(address));
        }

        let entry_idx = frame_idx / BITMAP_ENTRY_BITS;
        let bit = frame_idx % BITMAP_ENTRY_BITS;
        self.bitmap[entry_idx] &= !(1u64 << bit);
        self.free_frames += 1;

        Ok(())
    }

    pub fn deallocate_contiguous(&mut self, address: usize, count: usize) -> MemoryResult<()> {
        for i in 0..count {
            let addr = address + i * self.page_size;
            self.deallocate_frame(addr)?;
        }
        Ok(())
    }

    fn address_to_frame(&self, address: usize) -> MemoryResult<usize> {
        if address < self.base_address {
            return Err(MemoryError::InvalidAddress(address));
        }

        let offset = address - self.base_address;
        if offset % self.page_size != 0 {
            return Err(MemoryError::InvalidAlignment(address));
        }

        let frame_idx = offset / self.page_size;
        if frame_idx >= self.total_frames {
            return Err(MemoryError::InvalidAddress(address));
        }

        Ok(frame_idx)
    }

    fn is_frame_free(&self, frame_idx: usize) -> bool {
        let entry_idx = frame_idx / BITMAP_ENTRY_BITS;
        let bit = frame_idx % BITMAP_ENTRY_BITS;
        (self.bitmap[entry_idx] & (1u64 << bit)) == 0
    }

    fn mark_frame_used(&mut self, frame_idx: usize) {
        let entry_idx = frame_idx / BITMAP_ENTRY_BITS;
        let bit = frame_idx % BITMAP_ENTRY_BITS;
        self.bitmap[entry_idx] |= 1u64 << bit;
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_frames: self.total_frames,
            free_frames: self.free_frames,
            used_frames: self.total_frames - self.free_frames,
            page_size: self.page_size,
            total_bytes: self.total_frames * self.page_size,
            free_bytes: self.free_frames * self.page_size,
            used_bytes: (self.total_frames - self.free_frames) * self.page_size,
            heap_allocated: 0,
            heap_free: 0,
        }
    }

    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub fn free_frames(&self) -> usize {
        self.free_frames
    }

    pub fn used_frames(&self) -> usize {
        self.total_frames - self.free_frames
    }

    pub fn reserve_range(&mut self, start_address: usize, size: usize) -> MemoryResult<()> {
        let start_frame = self.address_to_frame(start_address)?;
        let frame_count = (size + self.page_size - 1) / self.page_size;

        for i in 0..frame_count {
            let frame_idx = start_frame + i;
            if frame_idx >= self.total_frames {
                return Err(MemoryError::InvalidAddress(start_address + i * self.page_size));
            }
            if !self.is_frame_free(frame_idx) {
                return Err(MemoryError::RegionOverlap {
                    base: start_address,
                    size,
                });
            }
            self.mark_frame_used(frame_idx);
            self.free_frames -= 1;
        }

        Ok(())
    }
}

pub struct BumpAllocator {
    base: usize,
    current: usize,
    limit: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            current: base,
            limit: base + size,
            allocations: 0,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> MemoryResult<usize> {
        let aligned = (self.current + align - 1) & !(align - 1);
        let end = aligned + size;

        if end > self.limit {
            return Err(MemoryError::OutOfMemory);
        }

        self.current = end;
        self.allocations += 1;
        Ok(aligned)
    }

    pub fn reset(&mut self) {
        self.current = self.base;
        self.allocations = 0;
    }

    pub fn used(&self) -> usize {
        self.current - self.base
    }

    pub fn remaining(&self) -> usize {
        self.limit - self.current
    }

    pub fn allocations(&self) -> usize {
        self.allocations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_allocator_creation() {
        let alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        assert_eq!(alloc.total_frames(), 256);
        assert_eq!(alloc.free_frames(), 256);
        assert_eq!(alloc.used_frames(), 0);
    }

    #[test]
    fn test_frame_allocate_single() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_frame().unwrap();
        assert_eq!(addr, 0);
        assert_eq!(alloc.free_frames(), 255);
        assert_eq!(alloc.used_frames(), 1);
    }

    #[test]
    fn test_frame_allocate_sequential() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let a1 = alloc.allocate_frame().unwrap();
        let a2 = alloc.allocate_frame().unwrap();
        let a3 = alloc.allocate_frame().unwrap();
        assert_eq!(a1, 0);
        assert_eq!(a2, 4096);
        assert_eq!(a3, 8192);
    }

    #[test]
    fn test_frame_deallocate() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_frame().unwrap();
        assert_eq!(alloc.free_frames(), 255);
        alloc.deallocate_frame(addr).unwrap();
        assert_eq!(alloc.free_frames(), 256);
    }

    #[test]
    fn test_frame_reuse_after_dealloc() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(addr).unwrap();
        let addr2 = alloc.allocate_frame().unwrap();
        assert_eq!(addr, addr2);
    }

    #[test]
    fn test_frame_double_free() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_frame().unwrap();
        alloc.deallocate_frame(addr).unwrap();
        assert!(alloc.deallocate_frame(addr).is_err());
    }

    #[test]
    fn test_frame_exhaustion() {
        let mut alloc = FrameAllocator::with_default_page_size(4096 * 2);
        alloc.allocate_frame().unwrap();
        alloc.allocate_frame().unwrap();
        assert!(alloc.allocate_frame().is_err());
    }

    #[test]
    fn test_contiguous_allocation() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_contiguous(4).unwrap();
        assert_eq!(addr, 0);
        assert_eq!(alloc.free_frames(), 252);
    }

    #[test]
    fn test_contiguous_deallocation() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        let addr = alloc.allocate_contiguous(4).unwrap();
        alloc.deallocate_contiguous(addr, 4).unwrap();
        assert_eq!(alloc.free_frames(), 256);
    }

    #[test]
    fn test_reserve_range() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        alloc.reserve_range(0, 4096 * 4).unwrap();
        assert_eq!(alloc.used_frames(), 4);
        let addr = alloc.allocate_frame().unwrap();
        assert_eq!(addr, 4096 * 4);
    }

    #[test]
    fn test_stats() {
        let mut alloc = FrameAllocator::with_default_page_size(1024 * 1024);
        alloc.allocate_frame().unwrap();
        alloc.allocate_frame().unwrap();
        let stats = alloc.stats();
        assert_eq!(stats.total_frames, 256);
        assert_eq!(stats.used_frames, 2);
        assert_eq!(stats.free_frames, 254);
        assert_eq!(stats.used_bytes, 8192);
    }

    #[test]
    fn test_bump_allocator_basic() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        let a1 = bump.allocate(64, 8).unwrap();
        assert_eq!(a1, 0x1000);
        assert_eq!(bump.allocations(), 1);
    }

    #[test]
    fn test_bump_allocator_alignment() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        bump.allocate(1, 1).unwrap();
        let a2 = bump.allocate(16, 16).unwrap();
        assert_eq!(a2 % 16, 0);
    }

    #[test]
    fn test_bump_allocator_exhaustion() {
        let mut bump = BumpAllocator::new(0x1000, 64);
        bump.allocate(32, 1).unwrap();
        bump.allocate(32, 1).unwrap();
        assert!(bump.allocate(1, 1).is_err());
    }

    #[test]
    fn test_bump_allocator_reset() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        bump.allocate(1024, 1).unwrap();
        assert_eq!(bump.used(), 1024);
        bump.reset();
        assert_eq!(bump.used(), 0);
        assert_eq!(bump.remaining(), 4096);
    }

    #[test]
    fn test_bump_allocator_remaining() {
        let mut bump = BumpAllocator::new(0x1000, 4096);
        assert_eq!(bump.remaining(), 4096);
        bump.allocate(100, 1).unwrap();
        assert_eq!(bump.remaining(), 3996);
    }
}
