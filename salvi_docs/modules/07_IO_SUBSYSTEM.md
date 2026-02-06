# Module Guide: I/O Subsystem

**Module:** `salvi_io`  
**Status:** Complete (P1.5-006 to P1.5-010)  
**Tests:** ~52 tests

---

## Overview

The I/O Subsystem provides the core infrastructure for data transfer between the kernel and devices. It implements a 4-level priority scheduler, LRU buffer cache, and unified interfaces for both block and character devices.

### Key Features

- **4-Level Priority Scheduler** — Prioritized I/O request handling
- **Buffer Cache** — LRU caching with write-back support
- **Block Device Layer** — Sector-based storage access
- **Character Device Layer** — Stream-based device access
- **Polling Infrastructure** — Efficient multiplexed I/O

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    I/O Manager                               │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │              I/O Priority Scheduler                  │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │   │
│  │  │Realtime │ │  High   │ │ Normal  │ │  Idle   │   │   │
│  │  │ Queue   │ │ Queue   │ │ Queue   │ │ Queue   │   │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │                   Buffer Cache                        │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │ Hash Table │ LRU List │ Dirty List │ Free List │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
├────────────────────────┬────────────────────────────────────┤
│    Block Devices       │       Character Devices            │
│  ┌──────────────────┐  │  ┌──────────────────┐             │
│  │ Request Queue    │  │  │ Read/Write Ops   │             │
│  │ Sector I/O       │  │  │ Line Discipline  │             │
│  │ Partitions       │  │  │ TTY Support      │             │
│  └──────────────────┘  │  └──────────────────┘             │
└────────────────────────┴────────────────────────────────────┘
```

---

## I/O Priority Scheduler

### Priority Levels

| Priority | Level | Use Case | Deadline |
|----------|-------|----------|----------|
| Realtime | 0 | Time-critical I/O | Immediate |
| High | 1 | Interactive I/O | 10ms |
| Normal | 2 | Regular I/O | 100ms |
| Idle | 3 | Background I/O | Best effort |

### Submitting I/O Requests

```rust
use salvi_io::{IoRequest, IoOp, IoPriority};

let request = IoRequest::builder()
    .device(block_device)
    .operation(IoOp::Read)
    .offset(sector * SECTOR_SIZE)
    .buffer(&mut buffer)
    .length(SECTOR_SIZE)
    .priority(IoPriority::Normal)
    .build()?;

io_scheduler::submit(request).await?;

let handle = io_scheduler::submit_async(request);
handle.await?;
```

### I/O Statistics

```rust
let stats = io_scheduler::stats();
println!("Requests completed: {}", stats.completed);
println!("Requests pending: {}", stats.pending);
println!("Average latency: {:?}", stats.avg_latency);
println!("Throughput: {} MB/s", stats.throughput_mbps);
```

---

## Buffer Cache

### LRU Cache Design

```
┌─────────────────────────────────────────────────────────────┐
│                     Buffer Cache                             │
├─────────────────────────────────────────────────────────────┤
│  Hash Table: [device, block] -> Buffer*                     │
├─────────────────────────────────────────────────────────────┤
│  LRU List:   MRU <-> ... <-> ... <-> LRU                    │
├─────────────────────────────────────────────────────────────┤
│  Dirty List: Oldest dirty <-> ... <-> Newest dirty          │
└─────────────────────────────────────────────────────────────┘
```

### Buffer Operations

```rust
use salvi_io::cache::{BufferCache, Buffer, BufferFlags};

let cache = BufferCache::instance();

let buffer = cache.get_block(device, block_num)?;
let data = buffer.data();

let mut buffer = cache.get_block_mut(device, block_num)?;
buffer.data_mut().copy_from_slice(&new_data);
buffer.mark_dirty();
buffer.sync()?;
drop(buffer);
```

### Cache Configuration

```rust
use salvi_io::cache::CacheConfig;

let config = CacheConfig {
    max_buffers: 10_000,
    buffer_size: 4096,
    write_back_interval: Duration::from_secs(30),
    write_back_threshold: 0.7,
    read_ahead_blocks: 8,
};

BufferCache::configure(config)?;
```

### Write Policies

```rust
use salvi_io::cache::WritePolicy;

cache.set_write_policy(device, WritePolicy::WriteBack);
cache.set_write_policy(device, WritePolicy::WriteThrough);
cache.sync_device(device)?;
cache.sync_all()?;
```

---

## Block Devices

### Block Device Interface

```rust
use salvi_io::block::{BlockDevice, BlockDeviceOps};

pub trait BlockDeviceOps {
    fn read_blocks(&self, start: u64, count: usize, buf: &mut [u8]) -> Result<()>;
    fn write_blocks(&self, start: u64, count: usize, buf: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
    fn block_size(&self) -> usize;
    fn total_blocks(&self) -> u64;
    fn is_read_only(&self) -> bool;
}
```

### Registering Block Devices

```rust
use salvi_io::block::{BlockDeviceRegistry, BlockDeviceInfo};

let registry = BlockDeviceRegistry::instance();

let info = BlockDeviceInfo {
    name: "sda",
    block_size: 512,
    total_blocks: 1_000_000_000,
    read_only: false,
    removable: false,
};

let device_id = registry.register(info, Box::new(my_block_driver))?;
```

---

## Character Devices

### Character Device Interface

```rust
use salvi_io::char::{CharDevice, CharDeviceOps};

pub trait CharDeviceOps {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, buf: &[u8]) -> Result<usize>;
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize>;
}
```

### TTY Support

```rust
use salvi_io::tty::{Tty, LineDiscipline};

let tty = Tty::new("tty0")?;
tty.set_discipline(LineDiscipline::Canonical)?;
tty.set_baud_rate(115200)?;
tty.write(b"Hello from kernel!\n")?;
```

---

## I/O Multiplexing

### Poll Interface

```rust
use salvi_io::poll::{Poll, PollEvent, PollFlags};

let mut poll = Poll::new()?;

poll.register(fd1, PollFlags::READABLE)?;
poll.register(fd2, PollFlags::READABLE | PollFlags::WRITABLE)?;

let events = poll.wait(Duration::from_secs(5))?;
for event in events {
    if event.is_readable() {
        handle_read(event.fd)?;
    }
    if event.is_writable() {
        handle_write(event.fd)?;
    }
}
```

---

## Best Practices

### 1. Use Appropriate I/O Priority
Reserve Realtime for truly time-critical operations.

### 2. Leverage the Buffer Cache
Always access block devices through the buffer cache for performance.

### 3. Sync Before Shutdown
Call `cache.sync_all()` before system shutdown to prevent data loss.

### 4. Use Async I/O for Non-Critical Paths
Submit requests asynchronously when possible to improve throughput.

---

*Così sia.*
