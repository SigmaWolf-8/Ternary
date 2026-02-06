# Module Guide: Process Management

**Module:** `salvi_kernel::process`  
**Status:** Complete (P1-011 to P1-015)  
**Tests:** ~55 tests

---

## Overview

The Process Management module provides comprehensive process and thread lifecycle management with ternary-aware scheduling. It implements an 8-state process model, priority-based scheduling with 6 levels, efficient context switching, and typed inter-process communication.

### Key Features

- **8-State Process Model** — Extended state machine for ternary computing
- **Priority Scheduler** — 6 priority levels with ternary fairness
- **Context Switching** — 16 GP + 8 ternary registers preserved
- **Typed IPC** — Type-safe message passing between processes
- **Thread Support** — Lightweight threads within processes

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Process Manager                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    Scheduler                         │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │   │
│  │  │Priority │ │Priority │ │Priority │ │   ...   │   │   │
│  │  │ Queue 0 │ │ Queue 1 │ │ Queue 2 │ │  (0-5)  │   │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ Process Table    │  │ Thread Table     │                │
│  │ ┌──────────────┐ │  │ ┌──────────────┐ │                │
│  │ │ PCB 0       │ │  │ │ TCB 0       │ │                │
│  │ │ PCB 1       │ │  │ │ TCB 1       │ │                │
│  │ │ ...        │ │  │ │ ...        │ │                │
│  │ └──────────────┘ │  │ └──────────────┘ │                │
│  └──────────────────┘  └──────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Inter-Process Communication             │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐             │   │
│  │  │ Message │  │ Shared  │  │ Signals │             │   │
│  │  │ Queues  │  │ Memory  │  │         │             │   │
│  │  └─────────┘  └─────────┘  └─────────┘             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Process States

The Salvi kernel uses an 8-state process model:

```
                    ┌─────────┐
                    │  NEW    │
                    └────┬────┘
                         │ admit
                         ▼
┌─────────┐        ┌─────────┐        ┌─────────┐
│ BLOCKED │◄──────▶│  READY  │◄──────▶│ RUNNING │
└────┬────┘  wait/ └────┬────┘dispatch└────┬────┘
     │       signal     │     /preempt     │
     │                  │                  │
     │    ┌─────────┐   │                  │
     └───▶│SUSPENDED│◄──┘                  │
          │ BLOCKED │                      │
          └─────────┘                      │
                                          │
          ┌─────────┐                      │
          │SUSPENDED│◄─────────────────────┤ suspend
          │  READY  │                      │
          └─────────┘                      │
                                          │
          ┌─────────┐        ┌─────────┐  │
          │  ZOMBIE │◄───────│TERMINTD │◄─┘ exit
          └─────────┘ parent └─────────┘
                      wait
```

| State | Description |
|-------|-------------|
| NEW | Process being created |
| READY | Waiting for CPU |
| RUNNING | Currently executing |
| BLOCKED | Waiting for I/O or event |
| SUSPENDED_READY | Swapped out, was ready |
| SUSPENDED_BLOCKED | Swapped out, was blocked |
| TERMINATED | Execution complete |
| ZOMBIE | Terminated, waiting for parent |

---

## Process Control Block (PCB)

```rust
use salvi_kernel::process::{Process, ProcessId, ProcessState};

/// Process Control Block
pub struct PCB {
    // Identity
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub name: String,
    
    // State
    pub state: ProcessState,
    pub exit_code: Option<i32>,
    
    // Scheduling
    pub priority: Priority,
    pub time_slice: u64,
    pub cpu_time: u64,
    
    // Memory
    pub address_space: AddressSpace,
    pub heap_start: TernaryAddress,
    pub heap_end: TernaryAddress,
    pub stack_pointer: TernaryAddress,
    
    // Context (saved on switch)
    pub context: ProcessContext,
    
    // Security
    pub security_mode: Mode,
    pub capabilities: CapabilitySet,
    
    // Resources
    pub open_files: Vec<FileDescriptor>,
    pub children: Vec<ProcessId>,
}
```

### Creating Processes

```rust
use salvi_kernel::process::{ProcessManager, ProcessBuilder};

let pm = ProcessManager::instance();

// Create a new process
let process = ProcessBuilder::new("my_process")
    .priority(Priority::Normal)
    .security_mode(Mode::User)
    .entry_point(entry_function)
    .stack_size(64 * 1024)
    .build()?;

let pid = pm.spawn(process)?;
println!("Spawned process with PID: {}", pid);

// Fork current process
let child_pid = pm.fork()?;
if child_pid == 0 {
    // Child process
    println!("I am the child");
} else {
    // Parent process
    println!("Spawned child with PID: {}", child_pid);
}
```

### Process Lifecycle

```rust
// Wait for process to exit
let exit_code = pm.wait(child_pid)?;

// Wait for any child
let (pid, exit_code) = pm.wait_any()?;

// Terminate process
pm.kill(pid, Signal::TERM)?;

// Force kill
pm.kill(pid, Signal::KILL)?;
```

---

## Scheduler

### Priority Levels

| Level | Name | Use Case |
|-------|------|----------|
| 0 | REALTIME | Interrupt handlers, critical kernel |
| 1 | HIGH | System services |
| 2 | ABOVE_NORMAL | Important user processes |
| 3 | NORMAL | Default for user processes |
| 4 | BELOW_NORMAL | Background tasks |
| 5 | IDLE | Only when nothing else runs |

### Scheduling Algorithm

The scheduler uses a **multilevel feedback queue** with ternary fairness:

```rust
impl Scheduler {
    pub fn schedule(&mut self) -> Option<ProcessId> {
        // Check each priority queue from highest to lowest
        for priority in 0..=5 {
            if let Some(pid) = self.queues[priority].pop_front() {
                // Ternary fairness: adjust quantum based on wait time
                let process = self.get_process(pid);
                let wait_trits = process.wait_time.to_trits();
                
                // Longer wait = larger quantum (balanced ternary bonus)
                let quantum = BASE_QUANTUM + wait_trits.sum() as u64;
                process.time_slice = quantum;
                
                return Some(pid);
            }
        }
        None  // No runnable process
    }
}
```

### Scheduler API

```rust
use salvi_kernel::process::scheduler;

// Yield CPU to another process
scheduler::yield_now();

// Sleep for duration
scheduler::sleep(Duration::from_millis(100));

// Set process priority
scheduler::set_priority(pid, Priority::High)?;

// Get scheduling statistics
let stats = scheduler::stats();
println!("Context switches: {}", stats.context_switches);
println!("Avg wait time: {:?}", stats.avg_wait_time);
```

---

## Context Switching

### Saved Context

Context switch preserves all CPU state:

```rust
/// Saved processor context
pub struct ProcessContext {
    // General purpose registers (16)
    pub r0: Tryte, pub r1: Tryte, pub r2: Tryte, pub r3: Tryte,
    pub r4: Tryte, pub r5: Tryte, pub r6: Tryte, pub r7: Tryte,
    pub r8: Tryte, pub r9: Tryte, pub r10: Tryte, pub r11: Tryte,
    pub r12: Tryte, pub r13: Tryte, pub r14: Tryte, pub r15: Tryte,
    
    // Ternary coprocessor registers (8)
    pub t0: Tryte, pub t1: Tryte, pub t2: Tryte, pub t3: Tryte,
    pub t4: Tryte, pub t5: Tryte, pub t6: Tryte, pub t7: Tryte,
    
    // Special registers
    pub sp: TernaryAddress,  // Stack pointer
    pub fp: TernaryAddress,  // Frame pointer
    pub pc: TernaryAddress,  // Program counter
    pub flags: Tryte,        // Status flags
    
    // Security context
    pub mode: Mode,
    pub page_table_base: TernaryAddress,
}
```

### Context Switch Implementation

```rust
impl Scheduler {
    /// Switch from current process to next
    #[naked]
    pub unsafe extern "C" fn context_switch(
        old_ctx: *mut ProcessContext,
        new_ctx: *const ProcessContext,
    ) {
        // Save current context
        asm!(
            // Save GP registers
            "store [r0], {old_ctx}+0",
            "store [r1], {old_ctx}+1",
            // ... (all 16 GP registers)
            
            // Save ternary registers
            "store [t0], {old_ctx}+16",
            // ... (all 8 ternary registers)
            
            // Save special registers
            "store [sp], {old_ctx}+24",
            "store [fp], {old_ctx}+25",
            "store [pc], {old_ctx}+26",
            
            // Load new context
            "load {new_ctx}+0, [r0]",
            "load {new_ctx}+1, [r1]",
            // ... (restore all registers)
            
            // Switch page tables
            "load {new_ctx}+28, [cr3]",
            
            // Return to new process
            "ret",
            
            old_ctx = in(reg) old_ctx,
            new_ctx = in(reg) new_ctx,
        );
    }
}
```

---

## Threads

Lightweight execution units within a process:

```rust
use salvi_kernel::process::{Thread, ThreadBuilder};

// Create thread in current process
let thread = ThreadBuilder::new()
    .name("worker")
    .entry(worker_function)
    .arg(worker_data)
    .stack_size(16 * 1024)
    .build()?;

let tid = thread.spawn()?;

// Join thread
let result = thread.join()?;

// Detach thread (won't be joined)
thread.detach();
```

### Thread Local Storage

```rust
use salvi_kernel::process::tls;

// Define thread-local variable
thread_local! {
    static COUNTER: Cell<u32> = Cell::new(0);
}

// Access thread-local
COUNTER.with(|c| {
    c.set(c.get() + 1);
    println!("Thread-local counter: {}", c.get());
});
```

---

## Inter-Process Communication (IPC)

### Message Passing

```rust
use salvi_kernel::ipc::{Channel, Message};

// Create channel
let (sender, receiver) = Channel::new();

// Send message (typed)
let msg = Message::new(42i64);
sender.send(msg)?;

// Receive message
let received: Message<i64> = receiver.recv()?;
assert_eq!(*received, 42);

// Non-blocking receive
match receiver.try_recv() {
    Ok(msg) => println!("Got: {:?}", msg),
    Err(TryRecvError::Empty) => println!("No message"),
    Err(TryRecvError::Disconnected) => println!("Channel closed"),
}
```

### Shared Memory

```rust
use salvi_kernel::ipc::{SharedMemory, SharedMemoryFlags};

// Create shared memory region
let shm = SharedMemory::create(
    "my_shared_data",
    4096,  // Size in trytes
    SharedMemoryFlags::READ | SharedMemoryFlags::WRITE,
)?;

// Map into address space
let ptr = shm.map()?;

// Write data
unsafe {
    ptr.write(data);
}

// Other process can open by name
let shm2 = SharedMemory::open("my_shared_data")?;
let ptr2 = shm2.map()?;
```

### Signals

```rust
use salvi_kernel::ipc::{Signal, SignalHandler};

// Register signal handler
Signal::register(Signal::USR1, |sig| {
    println!("Received signal: {:?}", sig);
})?;

// Send signal to process
Signal::send(target_pid, Signal::USR1)?;

// Block signals temporarily
let guard = Signal::block(&[Signal::INT, Signal::TERM]);
// ... critical section ...
drop(guard);  // Signals unblocked
```

### Pipes

```rust
use salvi_kernel::ipc::Pipe;

// Create pipe
let (read_end, write_end) = Pipe::new()?;

// Write to pipe
write_end.write(b"Hello through pipe")?;

// Read from pipe
let mut buffer = [0u8; 1024];
let n = read_end.read(&mut buffer)?;
```

---

## Process Groups and Sessions

```rust
use salvi_kernel::process::{ProcessGroup, Session};

// Create process group
let pgid = ProcessGroup::create()?;

// Add process to group
ProcessGroup::join(pgid, pid)?;

// Send signal to group
ProcessGroup::signal(pgid, Signal::TERM)?;

// Create session (for daemons)
let sid = Session::create()?;
```

---

## Best Practices

### 1. Use Appropriate Priority

```rust
// Realtime only for critical, short tasks
ProcessBuilder::new("critical_handler")
    .priority(Priority::Realtime)  // Use sparingly!
    .build();

// Background work at low priority
ProcessBuilder::new("backup_task")
    .priority(Priority::BelowNormal)
    .build();
```

### 2. Clean Up Resources

```rust
// Good: Process cleans up on exit
fn process_main() {
    let file = File::open("data.txt")?;
    let shm = SharedMemory::create("temp", 1024)?;
    
    // Work...
    
    // Explicit cleanup (also happens on drop)
    drop(file);
    shm.unlink()?;
}
```

### 3. Handle Signals Gracefully

```rust
static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    Signal::register(Signal::TERM, |_| {
        RUNNING.store(false, Ordering::SeqCst);
    })?;
    
    while RUNNING.load(Ordering::SeqCst) {
        do_work();
    }
    
    cleanup();
}
```

### 4. Use Typed IPC

```rust
// Good: Type-safe messages
#[derive(Serialize, Deserialize)]
struct Request {
    id: u64,
    operation: Operation,
    data: Vec<u8>,
}

let msg = Message::new(Request { ... });
sender.send(msg)?;

// Bad: Raw bytes (error-prone)
sender.send_raw(&bytes)?;
```

---

## Performance

| Operation | Time |
|-----------|------|
| Fork | ~50 µs |
| Context switch | ~2 µs |
| Thread create | ~10 µs |
| Message send | ~500 ns |
| Signal delivery | ~1 µs |

---

## Related Modules

- [Memory Management](./01_KERNEL_MEMORY.md) — Address space per process
- [Sync Primitives](./02_SYNC_PRIMITIVES.md) — Thread synchronization
- [Modal Security](./04_MODAL_SECURITY.md) — Process security context

---

*Part of the Salvi Framework Documentation. Così sia.* 🔱
