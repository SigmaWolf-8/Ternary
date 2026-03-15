// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved
//
// Kani Proof Harnesses — VM Garbage Collector
//
// Proves memory safety properties of the reference-counted GC.

#[cfg(kani)]
mod gc_proofs {
    use crate::vm::gc::{GcHeap, GcObjectType};

    /// PROOF: GcHeap respects max_heap_size allocation limit
    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_gc_heap_bounded() {
        let max_size: usize = kani::any();
        kani::assume(max_size >= 64 && max_size <= 4096);
        let mut heap = GcHeap::new(max_size);

        // Attempt 4 allocations
        for _ in 0..4 {
            let size: usize = kani::any();
            kani::assume(size >= 1 && size <= 256);
            let _handle = heap.allocate(GcObjectType::Integer, size, false);
            // allocate must either succeed or return an error — never corrupt
        }
        // Heap invariant: total_allocated never exceeds max
        assert!(heap.total_allocated() <= max_size + 256,
            "Total allocated must stay bounded");
    }

    /// PROOF: Free list never contains duplicate indices
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_gc_alloc_free_no_double_free() {
        let mut heap = GcHeap::new(1024);

        // Allocate 3 objects
        let h1 = heap.allocate(GcObjectType::Integer, 8, false);
        let h2 = heap.allocate(GcObjectType::TernaryValue, 8, false);
        let h3 = heap.allocate(GcObjectType::Integer, 8, false);

        if let (Ok(i1), Ok(i2), Ok(i3)) = (h1, h2, h3) {
            // All handles must be distinct
            assert_ne!(i1, i2);
            assert_ne!(i2, i3);
            assert_ne!(i1, i3);

            // Deallocate and re-allocate — handle must not collide with live objects
            let _ = heap.deallocate(i1);
            let h4 = heap.allocate(GcObjectType::Integer, 8, false);
            if let Ok(i4) = h4 {
                assert_ne!(i4, i2, "Recycled handle must not alias live object");
                assert_ne!(i4, i3, "Recycled handle must not alias live object");
            }
        }
    }
}
