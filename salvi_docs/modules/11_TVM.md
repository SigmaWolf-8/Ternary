# Module Guide: Ternary Virtual Machine (TVM)

**Module:** `salvi_tvm`  
**Status:** Complete (P3-001 to P3-005)  
**Tests:** ~95 tests

---

## Overview

The Ternary Virtual Machine (TVM) is a complete execution environment for ternary bytecode. It provides a 35-opcode instruction set, 27 registers, GF(3) arithmetic execution, and a ternary-aware garbage collector.

### Key Features

- **35-Opcode ISA** - Comprehensive instruction set for ternary operations
- **27 Registers** - 16 GP + 8 ternary coprocessor + 3 special
- **16-Byte Instructions** - Fixed-width encoding for simplicity
- **TAGC** - Ternary-Aware Garbage Collector with generational support
- **GF(3) ALU** - Native Galois Field arithmetic

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TVM Instance                              │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   Register File │  │  Program Memory │                   │
│  │  ┌───────────┐  │  │  ┌───────────┐  │                   │
│  │  │ R0-R15 GP │  │  │  │ Bytecode  │  │                   │
│  │  │ T0-T7 Ter │  │  │  │ (16B ins) │  │                   │
│  │  │ SP,FP,PC  │  │  │  └───────────┘  │                   │
│  │  └───────────┘  │  └─────────────────┘                   │
│  └─────────────────┘                                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │  Execution Unit │  │   Memory Unit   │                   │
│  │  ┌───────────┐  │  │  ┌───────────┐  │                   │
│  │  │ Decoder   │  │  │  │   Heap    │  │                   │
│  │  │ GF(3) ALU │  │  │  │   Stack   │  │                   │
│  │  │ Branch    │  │  │  │   TAGC    │  │                   │
│  │  └───────────┘  │  │  └───────────┘  │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

---

## Instruction Set Architecture

### Register Set

| Register | Name | Purpose |
|----------|------|---------|
| R0-R15 | General Purpose | Computation, addressing |
| T0-T7 | Ternary Coprocessor | GF(3) operations, trit manipulation |
| SP | Stack Pointer | Stack operations |
| FP | Frame Pointer | Function frames |
| PC | Program Counter | Instruction pointer |

### Instruction Format

All instructions are 16 bytes (128 bits / ~85 trits):

```
┌────────┬────────┬────────┬────────┬────────────────────────┐
│ Opcode │ Flags  │  Dst   │  Src1  │   Src2 / Immediate     │
│ 1 byte │ 1 byte │ 1 byte │ 1 byte │      12 bytes          │
└────────┴────────┴────────┴────────┴────────────────────────┘
```

### Opcode Categories

#### Data Movement (0x00-0x0F)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x00 | NOP | No operation |
| 0x01 | MOV | Move register to register |
| 0x02 | MOVI | Move immediate to register |
| 0x03 | LOAD | Load from memory |
| 0x04 | STORE | Store to memory |
| 0x05 | PUSH | Push to stack |
| 0x06 | POP | Pop from stack |
| 0x07 | LEA | Load effective address |

#### Arithmetic (0x10-0x1F)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x10 | ADD | Ternary addition |
| 0x11 | SUB | Ternary subtraction |
| 0x12 | MUL | Ternary multiplication |
| 0x13 | DIV | Ternary division |
| 0x14 | MOD | Ternary modulo |
| 0x15 | NEG | Ternary negation |
| 0x16 | INC | Increment |
| 0x17 | DEC | Decrement |

#### GF(3) Operations (0x20-0x2F)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x20 | GF3ADD | GF(3) addition |
| 0x21 | GF3MUL | GF(3) multiplication |
| 0x22 | GF3INV | GF(3) multiplicative inverse |
| 0x23 | GF3POW | GF(3) exponentiation |
| 0x24 | TRIT | Extract trit from tryte |
| 0x25 | SETTRIT | Set trit in tryte |
| 0x26 | ROTL | Rotate trits left |
| 0x27 | ROTR | Rotate trits right |

#### Control Flow (0x30-0x3F)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x30 | JMP | Unconditional jump |
| 0x31 | JZ | Jump if zero |
| 0x32 | JNZ | Jump if not zero |
| 0x33 | JP | Jump if positive |
| 0x34 | JN | Jump if negative |
| 0x35 | CALL | Call subroutine |
| 0x36 | RET | Return from subroutine |
| 0x37 | HALT | Stop execution |

#### Comparison (0x40-0x4F)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x40 | CMP | Compare (sets flags) |
| 0x41 | TEST | Test bits (AND, sets flags) |
| 0x42 | TCMP | Ternary compare (-1, 0, +1 result) |

---

## Basic Usage

### Creating a TVM Instance

```rust
use salvi_tvm::{TVM, TVMConfig};

// Default configuration
let mut vm = TVM::new();

// Custom configuration
let config = TVMConfig {
    memory_size: 1024 * 1024,  // 1MB
    stack_size: 64 * 1024,     // 64KB
    gc_threshold: 512 * 1024,  // GC at 512KB
    gc_strategy: GCStrategy::Generational,
};
let mut vm = TVM::with_config(config);
```

### Loading and Running Programs

```rust
use salvi_tvm::{TVM, Program};

let mut vm = TVM::new();

// Load from bytecode
let bytecode = std::fs::read("program.tbc")?;
vm.load_bytecode(&bytecode)?;

// Or load from assembly
let assembly = r#"
    MOVI R0, 10
    MOVI R1, 32
    ADD R2, R0, R1
    HALT
"#;
let program = Program::from_assembly(assembly)?;
vm.load_program(&program)?;

// Execute
match vm.run() {
    Ok(exit_code) => println!("Exited with code: {}", exit_code),
    Err(VMError::Halt) => println!("Normal halt"),
    Err(e) => eprintln!("Error: {:?}", e),
}

// Read result
let result = vm.get_register(Register::R2);
println!("R2 = {}", result.to_i64());
```

### Step-by-Step Execution

```rust
// Execute one instruction at a time
loop {
    match vm.step() {
        Ok(StepResult::Continue) => {
            println!("PC: {} | Instruction: {:?}", 
                vm.pc(), vm.current_instruction());
        },
        Ok(StepResult::Halt) => break,
        Err(e) => {
            eprintln!("Error at PC {}: {:?}", vm.pc(), e);
            break;
        }
    }
}
```

---

## GF(3) Arithmetic

The TVM has native support for Galois Field arithmetic:

```rust
// Assembly example: GF(3) operations
let program = r#"
    ; Load values into ternary registers
    MOVI T0, 1          ; T0 = 1 (element of GF(3))
    MOVI T1, 2          ; T1 = 2 (element of GF(3))
    
    ; GF(3) addition: (1 + 2) mod 3 = 0
    GF3ADD T2, T0, T1   ; T2 = 0
    
    ; GF(3) multiplication: (1 * 2) mod 3 = 2
    GF3MUL T3, T0, T1   ; T3 = 2
    
    ; GF(3) inverse: 2^(-1) mod 3 = 2 (since 2*2=4=1)
    GF3INV T4, T1       ; T4 = 2
    
    HALT
"#;

let program = Program::from_assembly(program)?;
vm.load_program(&program)?;
vm.run()?;

assert_eq!(vm.get_register(Register::T2).to_i64(), 0);
assert_eq!(vm.get_register(Register::T3).to_i64(), 2);
assert_eq!(vm.get_register(Register::T4).to_i64(), 2);
```

### Trit Manipulation

```rust
let program = r#"
    ; Create a tryte value
    MOVI R0, 42         ; R0 = 42 (some tryte value)
    
    ; Extract trit at position 2
    TRIT T0, R0, 2      ; T0 = trit[2] of R0
    
    ; Set trit at position 3 to +1
    MOVI T1, 1          ; T1 = +1 (positive trit)
    SETTRIT R0, R0, 3, T1  ; Set trit[3] = +1
    
    ; Rotate trits left by 2 positions
    ROTL R1, R0, 2      ; R1 = rotated value
    
    HALT
"#;
```

---

## Memory Model

### Memory Regions

```
┌─────────────────────────────────────────────┐  High Address
│                  Stack                       │  (grows down)
│                    |                         │
├─────────────────────────────────────────────┤
│                                             │
│               Free Space                    │
│                                             │
├─────────────────────────────────────────────┤
│                    ^                         │
│                  Heap                        │  (grows up)
├─────────────────────────────────────────────┤
│               Static Data                    │
├─────────────────────────────────────────────┤
│              Program Code                    │
└─────────────────────────────────────────────┘  Low Address
```

### Memory Operations

```rust
let program = r#"
    ; Allocate on stack
    PUSH R0             ; Push R0 to stack
    PUSH R1             ; Push R1 to stack
    
    ; Access stack via FP
    LOAD R2, [FP-1]     ; Load first local
    LOAD R3, [FP-2]     ; Load second local
    
    ; Heap allocation (via syscall)
    MOVI R0, 256        ; Size to allocate
    SYSCALL 1           ; Allocate, pointer in R0
    
    ; Store to heap
    STORE [R0], R1      ; Store R1 at heap location
    STORE [R0+1], R2    ; Store R2 at next location
    
    ; Load from heap
    LOAD R3, [R0]       ; Load back
    
    POP R1              ; Restore stack
    POP R0
    HALT
"#;
```

---

## Ternary-Aware Garbage Collector (TAGC)

### GC Overview

The TAGC uses a mark-sweep algorithm with generational support:

```
┌─────────────────────────────────────────────────────────────┐
│                    TAGC Heap Structure                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Generation 0 (Young)                                 │   │
│  │ - Small objects, frequently collected               │   │
│  │ - ~80% of objects die here                          │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Generation 1 (Tenured)                               │   │
│  │ - Long-lived objects, infrequently collected         │   │
│  │ - Promoted from Gen 0 after surviving N collections  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### GC Configuration

```rust
use salvi_tvm::gc::{GCConfig, GCStrategy};

let config = GCConfig {
    strategy: GCStrategy::Generational,
    gen0_threshold: 64 * 1024,   // Collect gen0 at 64KB
    gen1_threshold: 512 * 1024,  // Collect gen1 at 512KB
    promotion_age: 3,            // Promote after 3 gen0 collections
};

let mut vm = TVM::with_gc_config(config);
```

### GC Statistics

```rust
let stats = vm.gc_stats();
println!("Gen0 collections: {}", stats.gen0_collections);
println!("Gen1 collections: {}", stats.gen1_collections);
println!("Total reclaimed: {} bytes", stats.total_reclaimed);
println!("Current heap: {} bytes", stats.current_heap_size);
```

---

## Error Handling

```rust
use salvi_tvm::VMError;

match vm.run() {
    Ok(code) => { /* normal exit */ },
    Err(VMError::InvalidOpcode(op)) => {
        eprintln!("Unknown opcode: 0x{:02x}", op);
    },
    Err(VMError::StackOverflow) => {
        eprintln!("Stack overflow - increase stack size");
    },
    Err(VMError::StackUnderflow) => {
        eprintln!("Stack underflow - pop without matching push");
    },
    Err(VMError::DivisionByZero) => {
        eprintln!("Division by zero");
    },
    Err(VMError::OutOfMemory) => {
        eprintln!("Heap exhausted - increase memory or optimize");
    },
    Err(VMError::InvalidAddress(addr)) => {
        eprintln!("Invalid memory access at 0x{:x}", addr);
    },
    Err(VMError::Halt) => { /* normal halt instruction */ },
}
```

---

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Instruction decode | ~5 ns |
| Register access | ~1 ns |
| GF(3) addition | ~3 ns |
| GF(3) multiplication | ~5 ns |
| Memory load/store | ~10 ns |
| Function call overhead | ~20 ns |
| GC pause (gen0) | ~100 us |
| GC pause (gen1) | ~1 ms |

---

## Best Practices

### 1. Use Ternary Coprocessor Registers for GF(3)

```rust
// Good: T registers for ternary operations
"GF3ADD T2, T0, T1"

// Less efficient: GP registers need conversion
"GF3ADD R2, R0, R1"
```

### 2. Minimize Heap Allocations

```rust
// Prefer stack allocation
"PUSH R0"  // Stack - fast, no GC

// Use heap only when necessary
"SYSCALL 1"  // Heap - slower, GC overhead
```

### 3. Batch Operations

```rust
// Good: Process in bulk
"MOVI R0, 100"
".loop:"
"  GF3MUL T0, T0, T1"
"  DEC R0"
"  JNZ .loop"
```

---

## Related Modules

- [Kernel Memory](./01_KERNEL_MEMORY.md) - Memory management for TVM heap
- [Sync Primitives](./02_SYNC_PRIMITIVES.md) - Thread-safe TVM instances
- [Cryptography](./05_CRYPTOGRAPHY.md) - Post-quantum crypto in TVM programs
- [BTG](./13_BTG.md) - Binary-ternary conversion for I/O

---

*Part of the Salvi Framework Documentation. Cosi sia.*
