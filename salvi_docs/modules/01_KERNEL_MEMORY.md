# Module Guide: Kernel Memory Subsystem

**Module:** `salvi_kernel::memory`  
**Status:** Complete (P1-001 to P1-005)  
**Tests:** ~60 tests

---

## Overview

The Memory Subsystem provides ternary-native memory management for the Salvi kernel. Unlike binary systems that operate on 8-bit bytes, this subsystem works with **trytes** (6-trit units) as the fundamental addressable unit.

### Key Features

- **Ternary Frame Allocator** - Physical memory management using balanced ternary addresses
- **Page Table Management** - Multi-level page tables with ternary indexing
- **Heap Allocator** - Dynamic allocation with ternary-aligned blocks
- **Memory Protection** - Integration with Modal Security System

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Virtual Address Space                     │
├─────────────────────────────────────────────────────────────┤
│  User Space (Mode phi)  │  Kernel Space (Mode 0/1)          │
├───────────────────────┴─────────────────────────────────────┤
│                      Page Tables                             │
│    ┌─────────┐    ┌─────────┐    ┌─────────┐               │
│    │ Level 3 │───>│ Level 2 │───>│ Level 1 │───> Frame     │
│    └─────────┘    └─────────┘    └─────────┘               │
├─────────────────────────────────────────────────────────────┤
│                    Frame Allocator                           │
│    ┌──────────────────────────────────────────────────┐    │
│    │  Free List  │  Bitmap  │  Statistics            │    │
│    └──────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Physical Memory                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Types

### TernaryAddress

A 27-trit (4.5 tryte) virtual or physical address:

```rust
use salvi_kernel::memory::TernaryAddress;

// Create from integer
let addr = TernaryAddress::from_u64(0x1000);

// Create from tryte array
let addr = TernaryAddress::from_trytes([t0, t1, t2, t3]);

// Arithmetic operations
let next_page = addr.add_offset(PAGE_SIZE);
let aligned = addr.align_down(PAGE_SIZE);
```

### Frame

Represents a physical memory frame (ternary page):

```rust
use salvi_kernel::memory::{Frame, FrameAllocator};

// Allocate a single frame
let frame = allocator.allocate_frame()?;

// Get physical address
let phys_addr = frame.start_address();

// Deallocate when done
allocator.deallocate_frame(frame);
```

### PageTable

Multi-level page table structure:

```rust
use salvi_kernel::memory::{PageTable, PageTableEntry, PageFlags};

// Create new page table
let mut page_table = PageTable::new();

// Map virtual to physical
page_table.map(
    virtual_addr,
    physical_frame,
    PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER
)?;

// Translate address
let phys = page_table.translate(virtual_addr)?;

// Unmap
page_table.unmap(virtual_addr)?;
```

---

## Frame Allocator

The frame allocator manages physical memory using a hybrid bitmap/free-list approach optimized for ternary arithmetic.

### Initialization

```rust
use salvi_kernel::memory::{FrameAllocator, MemoryRegion};

// Define available physical memory regions
let regions = [
    MemoryRegion::new(0x100000, 0x1000000),  // 1MB - 16MB
    MemoryRegion::new(0x1000000, 0x10000000), // 16MB - 256MB
];

// Initialize allocator
let mut allocator = FrameAllocator::new(&regions);
```

### Allocation Strategies

```rust
// Single frame allocation
let frame = allocator.allocate_frame()?;

// Contiguous allocation (for DMA buffers)
let frames = allocator.allocate_contiguous(16)?;  // 16 contiguous frames

// Zone-based allocation
let frame = allocator.allocate_from_zone(Zone::DMA)?;     // Below 16MB
let frame = allocator.allocate_from_zone(Zone::Normal)?;  // Normal memory
let frame = allocator.allocate_from_zone(Zone::High)?;    // Above 4GB
```

### Statistics

```rust
let stats = allocator.statistics();
println!("Total frames: {}", stats.total_frames);
println!("Free frames: {}", stats.free_frames);
println!("Used frames: {}", stats.used_frames);
println!("Fragmentation: {:.2}%", stats.fragmentation_percent);
```

---

## Heap Allocator

The heap allocator provides `malloc`/`free` style allocation for kernel objects.

### Basic Usage

```rust
use salvi_kernel::memory::heap::{TernaryHeap, Layout};

// Initialize heap with memory region
let mut heap = TernaryHeap::new(heap_start, heap_size);

// Allocate memory
let layout = Layout::from_size_align(1024, 8)?;
let ptr = heap.allocate(layout)?;

// Use memory...

// Deallocate
heap.deallocate(ptr, layout);
```

### Integration with Rust's Global Allocator

```rust
use salvi_kernel::memory::heap::TernaryAllocator;

#[global_allocator]
static ALLOCATOR: TernaryAllocator = TernaryAllocator::new();

// Now you can use standard collections
let vec: Vec<Tryte> = Vec::new();
let boxed = Box::new(SomeStruct::new());
```

### Allocation Policies

The heap uses size-class segregation for efficiency:

| Size Class | Block Size | Use Case |
|------------|------------|----------|
| Tiny | 1-8 trytes | Small structures |
| Small | 9-64 trytes | Buffers, strings |
| Medium | 65-512 trytes | Large objects |
| Large | 513+ trytes | Direct mapping |

---

## Page Table Management

### Three-Level Paging

The Salvi Framework uses three-level page tables with ternary indexing:

```
Virtual Address (27 trits):
┌─────────┬─────────┬─────────┬──────────┐
│ L3 Index│ L2 Index│ L1 Index│  Offset  │
│ (9 trits)│(9 trits)│(9 trits)│(variable)│
└─────────┴─────────┴─────────┴──────────┘
```

Each index is 9 trits, supporting 3^9 = 19,683 entries per table.

### Page Table Entry Format

```rust
pub struct PageTableEntry {
    // Flags (lower 6 trits)
    present: Trit,      // Page is present in memory
    writable: Trit,     // Page is writable
    user: Trit,         // User-mode accessible
    accessed: Trit,     // Page has been accessed
    dirty: Trit,        // Page has been modified
    global: Trit,       // Global page (not flushed on context switch)
    
    // Physical frame number (upper 21 trits)
    frame: TernaryAddress,
}
```

### Creating Address Spaces

```rust
use salvi_kernel::memory::{AddressSpace, MappingFlags};

// Create new address space for a process
let mut space = AddressSpace::new()?;

// Map code section (read + execute)
space.map_region(
    code_start,
    code_size,
    MappingFlags::READ | MappingFlags::EXECUTE
)?;

// Map data section (read + write)
space.map_region(
    data_start,
    data_size,
    MappingFlags::READ | MappingFlags::WRITE
)?;

// Map stack (read + write, grows down)
space.map_region(
    stack_top - stack_size,
    stack_size,
    MappingFlags::READ | MappingFlags::WRITE | MappingFlags::STACK
)?;

// Switch to this address space
space.activate();
```

---

## Memory Protection

Integration with the Modal Security System provides hardware-enforced protection:

### Security Modes

| Mode | Name | Access Level |
|------|------|--------------|
| 0 | Hypervisor | Full access |
| 1 | Kernel | System memory |
| phi | Supervisor | Elevated user |
| phi+ | User | Restricted |

### Setting Protection

```rust
use salvi_kernel::memory::PageFlags;
use salvi_kernel::security::Mode;

// Kernel-only page
let flags = PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::KERNEL_ONLY;

// User-accessible page
let flags = PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER;

// Read-only shared page
let flags = PageFlags::PRESENT | PageFlags::USER;
```

---

## Best Practices

### 1. Always Check Allocation Results

```rust
// Good
if let Some(frame) = allocator.allocate_frame() {
    // Use frame
} else {
    // Handle out-of-memory
}

// Or use Result
let frame = allocator.allocate_frame().ok_or(MemoryError::OutOfMemory)?;
```

### 2. Align Allocations Properly

```rust
// Ternary alignment should be power-of-3 for efficiency
let layout = Layout::from_size_align(size, 9)?;  // 9-trit aligned
```

### 3. Use RAII for Frame Management

```rust
// FrameGuard automatically deallocates on drop
let guard = FrameGuard::new(allocator.allocate_frame()?);
// Frame is automatically freed when guard goes out of scope
```

### 4. Batch Operations When Possible

```rust
// Inefficient: many small allocations
for _ in 0..100 {
    let frame = allocator.allocate_frame()?;
    frames.push(frame);
}

// Efficient: single contiguous allocation
let frames = allocator.allocate_contiguous(100)?;
```

---

## Error Handling

```rust
use salvi_kernel::memory::MemoryError;

match allocator.allocate_frame() {
    Ok(frame) => { /* success */ },
    Err(MemoryError::OutOfMemory) => { /* no free frames */ },
    Err(MemoryError::InvalidAlignment) => { /* bad alignment */ },
    Err(MemoryError::AddressNotMapped) => { /* translation failed */ },
    Err(MemoryError::PermissionDenied) => { /* security violation */ },
}
```

---

## Performance Considerations

- **Frame allocation:** O(1) average using free list
- **Page table walk:** O(3) - three levels
- **TLB caching:** Use global pages for kernel mappings
- **Large pages:** Supported for 2187-tryte (3^7) regions

---

## Related Modules

- [Sync Primitives](./02_SYNC_PRIMITIVES.md) - Thread-safe memory access
- [Cryptography](./05_CRYPTOGRAPHY.md) - Secure memory operations
- [TVM](./11_TVM.md) - Virtual machine memory model

---

*Part of the Salvi Framework Documentation. Cosi sia.*
