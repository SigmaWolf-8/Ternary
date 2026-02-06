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

pub struct PCB {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub name: String,
    pub state: ProcessState,
    pub exit_code: Option<i32>,
    pub priority: Priority,
    pub time_slice: u64,
    pub cpu_time: u64,
    pub address_space: AddressSpace,
    pub heap_start: TernaryAddress,
    pub heap_end: TernaryAddress,
    pub stack_pointer: TernaryAddress,
    pub context: ProcessContext,
    pub security_mode: Mode,
    pub capabilities: CapabilitySet,
    pub open_files: Vec<FileDescriptor>,
    pub children: Vec<ProcessId>,
}
```

### Creating Processes

```rust
use salvi_kernel::process::{ProcessManager, ProcessBuilder};

let pm = ProcessManager::instance();

let process = ProcessBuilder::new("my_process")
    .priority(Priority::Normal)
    .security_mode(Mode::User)
    .entry_point(entry_function)
    .stack_size(64 * 1024)
    .build()?;

let pid = pm.spawn(process)?;
println!("Spawned process with PID: {}", pid);

let child_pid = pm.fork()?;
if child_pid == 0 {
    println!("I am the child");
} else {
    println!("Spawned child with PID: {}", child_pid);
}
```

### Process Lifecycle

```rust
let exit_code = pm.wait(child_pid)?;
let (pid, exit_code) = pm.wait_any()?;
pm.kill(pid, Signal::TERM)?;
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
        for priority in 0..=5 {
            if let Some(pid) = self.queues[priority].pop_front() {
                let process = self.get_process(pid);
                let wait_trits = process.wait_time.to_trits();
                let quantum = BASE_QUANTUM + wait_trits.sum() as u64;
                process.time_slice = quantum;
                return Some(pid);
            }
        }
        None
    }
}
```

### Scheduler API

```rust
use salvi_kernel::process::scheduler;

scheduler::yield_now();
scheduler::sleep(Duration::from_millis(100));
scheduler::set_priority(pid, Priority::High)?;

let stats = scheduler::stats();
println!("Context switches: {}", stats.context_switches);
println!("Avg wait time: {:?}", stats.avg_wait_time);
```

---

## Context Switching

### Saved Context

```rust
pub struct ProcessContext {
    pub r0: Tryte, pub r1: Tryte, pub r2: Tryte, pub r3: Tryte,
    pub r4: Tryte, pub r5: Tryte, pub r6: Tryte, pub r7: Tryte,
    pub r8: Tryte, pub r9: Tryte, pub r10: Tryte, pub r11: Tryte,
    pub r12: Tryte, pub r13: Tryte, pub r14: Tryte, pub r15: Tryte,
    pub t0: Tryte, pub t1: Tryte, pub t2: Tryte, pub t3: Tryte,
    pub t4: Tryte, pub t5: Tryte, pub t6: Tryte, pub t7: Tryte,
    pub sp: TernaryAddress,
    pub fp: TernaryAddress,
    pub pc: TernaryAddress,
    pub flags: StatusFlags,
    pub security_mode: Mode,
}
```

### Context Switch Flow

```
Current Process                    Next Process
┌─────────────┐                  ┌─────────────┐
│  Save regs  │                  │  Load regs  │
│  Save SP/FP │                  │  Load SP/FP │
│  Save PC    │──── Switch ────▶│  Load PC    │
│  Save flags │                  │  Load flags │
│  Save mode  │                  │  Load mode  │
└─────────────┘                  └─────────────┘
      │                                │
      ▼                                ▼
   PCB (old)                       PCB (new)
```

---

## Inter-Process Communication

### Message Passing

```rust
use salvi_kernel::process::ipc::{MessageQueue, Message};

let queue = MessageQueue::create(100)?;

queue.send(Message::new(
    sender_pid,
    receiver_pid,
    MessageType::Data,
    payload.as_bytes(),
))?;

let msg = queue.receive()?;
println!("From {}: {:?}", msg.sender, msg.payload);
```

### Shared Memory

```rust
use salvi_kernel::process::ipc::SharedMemory;

let shm = SharedMemory::create("my_shm", 4096)?;
let ptr = shm.attach()?;

unsafe {
    std::ptr::write(ptr as *mut u32, 42);
}

shm.detach(ptr)?;
```

### Signals

```rust
use salvi_kernel::process::signal::{Signal, SignalHandler};

pm.register_handler(Signal::USR1, SignalHandler::new(|sig| {
    println!("Received signal: {:?}", sig);
}))?;

pm.send_signal(target_pid, Signal::USR1)?;
```

---

## Best Practices

### 1. Use Appropriate Priority Levels

```rust
// System service
ProcessBuilder::new("logger").priority(Priority::High).build()?;

// User application
ProcessBuilder::new("editor").priority(Priority::Normal).build()?;

// Background task
ProcessBuilder::new("indexer").priority(Priority::BelowNormal).build()?;
```

### 2. Always Clean Up Child Processes

```rust
let pid = pm.spawn(child)?;
// ... later ...
let _exit = pm.wait(pid)?;  // Prevent zombies
```

### 3. Prefer Message Passing Over Shared Memory

Message passing provides type safety and avoids synchronization issues. Use shared memory only for high-throughput scenarios.

---

## Error Handling

```rust
use salvi_kernel::process::ProcessError;

match pm.spawn(process) {
    Ok(pid) => println!("Process {} started", pid),
    Err(ProcessError::ResourceLimit) => println!("Too many processes"),
    Err(ProcessError::OutOfMemory) => println!("Not enough memory"),
    Err(ProcessError::PermissionDenied) => println!("Insufficient privileges"),
    Err(e) => println!("Error: {:?}", e),
}
```

---

*Così sia.* 
