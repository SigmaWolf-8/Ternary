# Module Guide: Synchronization Primitives

**Module:** `salvi_kernel::sync`  
**Status:** Complete (P1-006 to P1-010)  
**Tests:** ~50 tests

---

## Overview

The Synchronization Primitives module provides thread-safe concurrency mechanisms designed for ternary computing. Each primitive integrates with the Modal Security System for security-gated access control.

### Key Features

- **Ternary Spinlock** - Ticket-based spinlock with balanced ternary ticket numbers
- **Security-Gated Mutex** - Mutex with ternary security mode enforcement
- **Counting Semaphore** - Ternary-valued permits with priority ordering
- **PhaseSafeMutex** - Phase-encryption-aware mutex with timing-window enforcement
- **Read-Write Lock** - Multiple-reader, single-writer with ternary priority

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Spinlock   │  │    Mutex    │  │  Semaphore  │         │
│  │  (Ticket)   │  │  (Security) │  │  (Counting) │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│  ┌─────────────┐  ┌─────────────┐                           │
│  │ PhaseSafe   │  │  RwLock     │                           │
│  │   Mutex     │  │ (Priority)  │                           │
│  └─────────────┘  └─────────────┘                           │
├─────────────────────────────────────────────────────────────┤
│                Modal Security System                         │
│         (Security mode verification)                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Primitives

### Ternary Spinlock

A ticket-based spinlock using balanced ternary ticket numbers:

```rust
use salvi_kernel::sync::TernarySpinlock;

let lock = TernarySpinlock::new();

// Acquire lock
let guard = lock.lock();
// Critical section...
// Lock automatically released when guard is dropped

// Try-lock variant
if let Some(guard) = lock.try_lock() {
    // Got the lock
}
```

### Security-Gated Mutex

A mutex that enforces security mode requirements:

```rust
use salvi_kernel::sync::{SecurityMutex, SecurityMode};

// Create mutex requiring kernel-mode access
let mutex = SecurityMutex::new(data, SecurityMode::Kernel);

// Only succeeds if current security mode >= Kernel
match mutex.lock() {
    Ok(guard) => {
        // Access protected data
        *guard = new_value;
    },
    Err(SecurityError::InsufficientMode) => {
        // Current mode too low
    }
}
```

### Counting Semaphore

```rust
use salvi_kernel::sync::TernarySemaphore;

// Create semaphore with 3 permits (ternary!)
let sem = TernarySemaphore::new(3);

// Acquire permit
sem.acquire();
// Do work...
sem.release();

// Try acquire (non-blocking)
if sem.try_acquire() {
    // Got a permit
    sem.release();
}
```

### PhaseSafeMutex

Phase-encryption-aware mutex with timing-window enforcement:

```rust
use salvi_kernel::sync::PhaseSafeMutex;
use salvi_kernel::timing::TimingWindow;

// Create mutex with timing window
let window = TimingWindow::new(Duration::from_millis(100));
let mutex = PhaseSafeMutex::new(data, window);

// Lock only succeeds within valid timing window
match mutex.lock() {
    Ok(guard) => {
        // Access data within timing window
    },
    Err(TimingError::WindowExpired) => {
        // Timing window has passed
    }
}
```

### Read-Write Lock

```rust
use salvi_kernel::sync::TernaryRwLock;

let rwlock = TernaryRwLock::new(data);

// Multiple readers
{
    let reader1 = rwlock.read();
    let reader2 = rwlock.read();
    println!("Value: {}", *reader1);
}

// Exclusive writer
{
    let mut writer = rwlock.write();
    *writer = new_value;
}
```

---

## Best Practices

### 1. Prefer RwLock for Read-Heavy Workloads

```rust
// Good: Multiple concurrent readers
let lock = TernaryRwLock::new(data);

// Bad: Mutex blocks all concurrent access
let lock = SecurityMutex::new(data, SecurityMode::User);
```

### 2. Use try_lock to Avoid Deadlocks

```rust
if let Some(guard) = lock.try_lock() {
    // Process
} else {
    // Fallback - don't block
}
```

### 3. Keep Critical Sections Short

```rust
// Good: Minimal work under lock
let value = {
    let guard = lock.lock();
    guard.clone()
};
process(value);  // Outside lock

// Bad: Long work under lock
let guard = lock.lock();
expensive_computation(&guard);  // Holds lock too long
```

---

## Related Modules

- [Kernel Memory](./01_KERNEL_MEMORY.md) - Memory protection integration
- [Cryptography](./05_CRYPTOGRAPHY.md) - Phase encryption for PhaseSafeMutex
- [TVM](./11_TVM.md) - Concurrent TVM instance management

---

*Part of the Salvi Framework Documentation. Cosi sia.*
