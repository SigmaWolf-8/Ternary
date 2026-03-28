// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const MIN_BLOCK_SIZE: usize = 32;
const BACK_PTR_SIZE: usize = core::mem::size_of::<usize>();

#[repr(C, align(8))]
struct BlockHeader {
    magic: usize,
    size: usize,
    next: usize,
}

const HEADER_SIZE: usize = core::mem::size_of::<BlockHeader>();
const HEADER_MAGIC_FREE: usize = 0xDEAD_BEEF_CAFE_F0EE;
const HEADER_MAGIC_USED: usize = 0xA110_CA7E_D000_B10C;

pub struct LinkedListAllocator {
    head: UnsafeCell<usize>,
    lock: AtomicBool,
    initialized: AtomicBool,
    heap_start: UnsafeCell<usize>,
    heap_end: UnsafeCell<usize>,
    allocated_bytes: AtomicUsize,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
}

unsafe impl Send for LinkedListAllocator {}
unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: UnsafeCell::new(0),
            lock: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            heap_start: UnsafeCell::new(0),
            heap_end: UnsafeCell::new(0),
            allocated_bytes: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.acquire_lock();

        let aligned_start = Self::align_up(heap_start, 8);
        let usable_size = heap_size - (aligned_start - heap_start);

        *self.heap_start.get() = aligned_start;
        *self.heap_end.get() = aligned_start + usable_size;

        let header = aligned_start as *mut BlockHeader;
        (*header).magic = HEADER_MAGIC_FREE;
        (*header).size = usable_size;
        (*header).next = 0;
        *self.head.get() = aligned_start;

        self.initialized.store(true, Ordering::Release);
        self.release_lock();
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    pub fn deallocation_count(&self) -> usize {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    pub fn free_bytes(&self) -> usize {
        unsafe {
            let total = (*self.heap_end.get()) - (*self.heap_start.get());
            total.saturating_sub(self.allocated_bytes.load(Ordering::Relaxed))
        }
    }

    fn acquire_lock(&self) {
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    fn release_lock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !self.initialized.load(Ordering::Acquire) {
            return ptr::null_mut();
        }

        let size = layout.size().max(MIN_BLOCK_SIZE);
        let align = layout.align().max(8);

        self.acquire_lock();

        let mut prev_addr: usize = 0;
        let mut current_addr = *self.head.get();

        while current_addr != 0 {
            let header = current_addr as *const BlockHeader;

            if (*header).magic != HEADER_MAGIC_FREE {
                self.release_lock();
                return ptr::null_mut();
            }

            let block_size = (*header).size;
            let next = (*header).next;

            let after_header = current_addr + HEADER_SIZE;
            let data_start = Self::align_up(after_header + BACK_PTR_SIZE, align);
            let total_needed = (data_start - current_addr) + size;

            if total_needed <= block_size {
                let remainder = block_size - total_needed;

                if remainder >= HEADER_SIZE + MIN_BLOCK_SIZE + BACK_PTR_SIZE {
                    let new_block_addr = Self::align_up(current_addr + total_needed, 8);
                    let actual_remainder = block_size - (new_block_addr - current_addr);

                    if actual_remainder >= HEADER_SIZE + MIN_BLOCK_SIZE + BACK_PTR_SIZE {
                        let new_header = new_block_addr as *mut BlockHeader;
                        (*new_header).magic = HEADER_MAGIC_FREE;
                        (*new_header).size = actual_remainder;
                        (*new_header).next = next;

                        let header_mut = current_addr as *mut BlockHeader;
                        (*header_mut).magic = HEADER_MAGIC_USED;
                        (*header_mut).size = new_block_addr - current_addr;
                        (*header_mut).next = 0;

                        if prev_addr == 0 {
                            *self.head.get() = new_block_addr;
                        } else {
                            (*(prev_addr as *mut BlockHeader)).next = new_block_addr;
                        }
                    } else {
                        let header_mut = current_addr as *mut BlockHeader;
                        (*header_mut).magic = HEADER_MAGIC_USED;
                        (*header_mut).next = 0;

                        if prev_addr == 0 {
                            *self.head.get() = next;
                        } else {
                            (*(prev_addr as *mut BlockHeader)).next = next;
                        }
                    }
                } else {
                    let header_mut = current_addr as *mut BlockHeader;
                    (*header_mut).magic = HEADER_MAGIC_USED;
                    (*header_mut).next = 0;

                    if prev_addr == 0 {
                        *self.head.get() = next;
                    } else {
                        (*(prev_addr as *mut BlockHeader)).next = next;
                    }
                }

                let back_ptr = (data_start - BACK_PTR_SIZE) as *mut usize;
                *back_ptr = current_addr;

                let used_header = current_addr as *const BlockHeader;
                self.allocated_bytes
                    .fetch_add((*used_header).size, Ordering::Relaxed);
                self.allocation_count.fetch_add(1, Ordering::Relaxed);

                self.release_lock();
                return data_start as *mut u8;
            }

            prev_addr = current_addr;
            current_addr = next;
        }

        self.release_lock();
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let data_addr = ptr as usize;
        let heap_start = *self.heap_start.get();
        let heap_end = *self.heap_end.get();

        if data_addr < heap_start + HEADER_SIZE + BACK_PTR_SIZE || data_addr >= heap_end {
            return;
        }

        let back_ptr = (data_addr - BACK_PTR_SIZE) as *const usize;
        let header_addr = *back_ptr;

        if header_addr < heap_start || header_addr + HEADER_SIZE > heap_end {
            return;
        }

        if header_addr % 8 != 0 {
            return;
        }

        let header = header_addr as *mut BlockHeader;
        if (*header).magic != HEADER_MAGIC_USED {
            return;
        }

        let block_size = (*header).size;
        if header_addr + block_size > heap_end {
            return;
        }

        self.acquire_lock();

        (*header).magic = HEADER_MAGIC_FREE;
        (*header).next = *self.head.get();
        *self.head.get() = header_addr;

        self.allocated_bytes.fetch_sub(
            block_size.min(self.allocated_bytes.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);

        self.coalesce_free_blocks();

        self.release_lock();
    }
}

impl LinkedListAllocator {
    unsafe fn coalesce_free_blocks(&self) {
        let heap_end = *self.heap_end.get();

        let mut merged = true;
        while merged {
            merged = false;
            let mut current_addr = *self.head.get();

            while current_addr != 0 {
                let header = current_addr as *mut BlockHeader;
                if (*header).magic != HEADER_MAGIC_FREE {
                    current_addr = (*header).next;
                    continue;
                }

                let block_end = current_addr + (*header).size;
                if block_end >= heap_end {
                    current_addr = (*header).next;
                    continue;
                }

                if block_end % 8 != 0 {
                    current_addr = (*header).next;
                    continue;
                }

                let next_candidate = block_end as *const BlockHeader;
                if (*next_candidate).magic == HEADER_MAGIC_FREE {
                    let next_size = (*next_candidate).size;
                    let next_next = (*next_candidate).next;

                    (*header).size += next_size;

                    let mut prev_addr: usize = 0;
                    let mut scan = *self.head.get();
                    while scan != 0 {
                        let scan_header = scan as *const BlockHeader;
                        if scan == block_end {
                            if prev_addr == 0 {
                                *self.head.get() = next_next;
                            } else {
                                (*(prev_addr as *mut BlockHeader)).next = next_next;
                            }
                            break;
                        }
                        prev_addr = scan;
                        scan = (*scan_header).next;
                    }

                    merged = true;
                    continue;
                }

                current_addr = (*header).next;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn make_allocator() -> (LinkedListAllocator, Vec<u8>) {
        let heap = alloc::vec![0u8; 1024 * 1024];
        let allocator = LinkedListAllocator::new();
        let start = heap.as_ptr() as usize;
        let start_aligned = (start + 7) & !7;
        let size = heap.len() - (start_aligned - start);
        unsafe { allocator.init(start_aligned, size) };
        (allocator, heap)
    }

    #[test]
    fn test_alloc_and_dealloc() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % 8, 0);
            core::ptr::write_bytes(ptr, 0xAA, 64);
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_multiple_alloc_dealloc() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(128, 8).unwrap();
        unsafe {
            let mut ptrs = Vec::new();
            for _ in 0..100 {
                let ptr = allocator.alloc(layout);
                assert!(!ptr.is_null());
                assert_eq!(ptr as usize % 8, 0);
                core::ptr::write_bytes(ptr, 0xBB, 128);
                ptrs.push(ptr);
            }
            for ptr in ptrs {
                allocator.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_alloc_different_sizes() {
        let (allocator, _heap) = make_allocator();
        unsafe {
            let l1 = Layout::from_size_align(32, 8).unwrap();
            let l2 = Layout::from_size_align(256, 16).unwrap();
            let l3 = Layout::from_size_align(4096, 64).unwrap();
            let p1 = allocator.alloc(l1);
            let p2 = allocator.alloc(l2);
            let p3 = allocator.alloc(l3);
            assert!(!p1.is_null());
            assert!(!p2.is_null());
            assert!(!p3.is_null());
            assert_ne!(p1, p2);
            assert_ne!(p2, p3);
            assert_eq!(p1 as usize % 8, 0);
            assert_eq!(p2 as usize % 16, 0);
            assert_eq!(p3 as usize % 64, 0);
            allocator.dealloc(p1, l1);
            allocator.dealloc(p2, l2);
            allocator.dealloc(p3, l3);
        }
    }

    #[test]
    fn test_alloc_reuse_after_free() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(512, 8).unwrap();
        unsafe {
            let p1 = allocator.alloc(layout);
            assert!(!p1.is_null());
            allocator.dealloc(p1, layout);

            let p2 = allocator.alloc(layout);
            assert!(!p2.is_null());
            allocator.dealloc(p2, layout);
        }
    }

    #[test]
    fn test_stats_tracking() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        assert_eq!(allocator.allocation_count(), 0);
        assert_eq!(allocator.deallocation_count(), 0);
        unsafe {
            let ptr = allocator.alloc(layout);
            assert_eq!(allocator.allocation_count(), 1);
            assert!(allocator.allocated_bytes() > 0);
            allocator.dealloc(ptr, layout);
            assert_eq!(allocator.deallocation_count(), 1);
        }
    }

    #[test]
    fn test_null_dealloc_safe() {
        let (_allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        unsafe {
            _allocator.dealloc(ptr::null_mut(), layout);
        }
    }

    #[test]
    fn test_interleaved_alloc_free() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        unsafe {
            let p1 = allocator.alloc(layout);
            let p2 = allocator.alloc(layout);
            let p3 = allocator.alloc(layout);
            assert!(!p1.is_null());
            assert!(!p2.is_null());
            assert!(!p3.is_null());

            allocator.dealloc(p2, layout);

            let p4 = allocator.alloc(layout);
            assert!(!p4.is_null());

            allocator.dealloc(p1, layout);
            allocator.dealloc(p3, layout);
            allocator.dealloc(p4, layout);
        }
    }

    #[test]
    fn test_large_alignment() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(256, 4096).unwrap();
        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % 4096, 0);
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_stress_alloc_free_cycles() {
        let (allocator, _heap) = make_allocator();
        unsafe {
            for size in [32, 64, 128, 256, 512, 1024] {
                let layout = Layout::from_size_align(size, 8).unwrap();
                let mut ptrs = Vec::new();
                for _ in 0..50 {
                    let ptr = allocator.alloc(layout);
                    assert!(!ptr.is_null());
                    core::ptr::write_bytes(ptr, 0xCC, size);
                    ptrs.push(ptr);
                }
                for ptr in ptrs.into_iter().rev() {
                    allocator.dealloc(ptr, layout);
                }
            }
        }
    }

    #[test]
    fn test_coalescing_adjacent_blocks() {
        let (allocator, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        unsafe {
            let p1 = allocator.alloc(layout);
            let p2 = allocator.alloc(layout);
            let p3 = allocator.alloc(layout);
            assert!(!p1.is_null());
            assert!(!p2.is_null());
            assert!(!p3.is_null());

            allocator.dealloc(p1, layout);
            allocator.dealloc(p2, layout);
            allocator.dealloc(p3, layout);

            let big_layout = Layout::from_size_align(192, 8).unwrap();
            let p_big = allocator.alloc(big_layout);
            assert!(!p_big.is_null());
            allocator.dealloc(p_big, big_layout);
        }
    }

    #[test]
    fn test_fragmentation_resistance() {
        let (allocator, _heap) = make_allocator();
        let small = Layout::from_size_align(64, 8).unwrap();
        let large = Layout::from_size_align(4096, 8).unwrap();
        unsafe {
            let mut small_ptrs = Vec::new();
            for _ in 0..100 {
                let ptr = allocator.alloc(small);
                assert!(!ptr.is_null());
                small_ptrs.push(ptr);
            }
            for ptr in small_ptrs {
                allocator.dealloc(ptr, small);
            }

            let big = allocator.alloc(large);
            assert!(!big.is_null());
            allocator.dealloc(big, large);
        }
    }
}
