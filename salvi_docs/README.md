# Salvi Framework Documentation

**Version:** 1.0 (Complete)  
**Repository:** [github.com/SigmaWolf-8/Ternary](https://github.com/SigmaWolf-8/Ternary)  
**Tests:** 1,011 passing

---

## Quick Links

| I want to... | Go to... |
|--------------|----------|
| Get started quickly | [Getting Started Tutorial](./tutorials/GETTING_STARTED.md) |
| Understand the architecture | [Architecture Overview](#architecture-overview) |
| Learn a specific module | [Module Guides](#module-guides) |
| Find API documentation | [API Reference](#api-reference) |

---

## Architecture Overview

The Salvi Framework implements the complete **Unified Post-Quantum Ternary-Torsion Internet (UPQTTI)** stack:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│   │   TVM   │  │  T3P    │  │  TDNS   │  │  Apps   │       │
│   └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
├─────────────────────────────────────────────────────────────┤
│                    Protocol Layer                            │
│   ┌─────────────────┐  ┌─────────────────┐                  │
│   │ TTP (Transport) │  │ HPTP (Timing)   │                  │
│   └─────────────────┘  └─────────────────┘                  │
├─────────────────────────────────────────────────────────────┤
│                    Network Layer                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │          13D Torsion Topology (PlenumNET)           │   │
│   └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Kernel Layer                              │
│   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
│   │ Memory │ │  Sync  │ │Process │ │Security│ │ Crypto │   │
│   └────────┘ └────────┘ └────────┘ └────────┘ └────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                      │
│   ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│   │  Drivers   │  │    I/O     │  │ Filesystem │           │
│   └────────────┘  └────────────┘  └────────────┘           │
├─────────────────────────────────────────────────────────────┤
│                    Hardware Abstraction                      │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│   │  x86_64 │  │ aarch64 │  │ RISC-V  │  │   TPU   │       │
│   └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
├─────────────────────────────────────────────────────────────┤
│                  Compatibility Layer                         │
│   ┌─────────────────────────────────────────────────────┐   │
│   │        Binary-Ternary Gateway (BTG)                 │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Tutorials

Step-by-step guides to get you productive quickly:

| Tutorial | Description | Time |
|----------|-------------|------|
| [Getting Started](./tutorials/GETTING_STARTED.md) | First ternary program in 15 minutes | 15 min |

---

## Module Guides

Comprehensive documentation for each subsystem:

### Kernel (P1)

| Module | Description | Guide |
|--------|-------------|-------|
| Memory Subsystem | Frame allocator, page tables, heap | [01_KERNEL_MEMORY.md](./modules/01_KERNEL_MEMORY.md) |
| Sync Primitives | Spinlock, mutex, semaphore, RwLock | [02_SYNC_PRIMITIVES.md](./modules/02_SYNC_PRIMITIVES.md) |
| Process Management | PCB, scheduler, context switch, IPC | 03_PROCESS_MANAGEMENT.md |
| Modal Security | Domains, capabilities, audit, policy | 04_MODAL_SECURITY.md |
| Cryptography | Hash, HMAC, KDF, Lamport OTS, cipher | [05_CRYPTOGRAPHY.md](./modules/05_CRYPTOGRAPHY.md) |

### Infrastructure (P1.5)

| Module | Description | Guide |
|--------|-------------|-------|
| Device Drivers | Lifecycle, bus hierarchy, registry | 06_DEVICE_FRAMEWORK.md |
| I/O Subsystem | Priority scheduler, buffer cache | 07_IO_SUBSYSTEM.md |
| Filesystem | Inodes, directories, file ops, mount | 08_FILESYSTEM.md |

### Core Features (P2)

| Module | Description | Guide |
|--------|-------------|-------|
| Torsion Network | N-dim torus, geodesic routing | [09_TORSION_NETWORK.md](./modules/09_TORSION_NETWORK.md) |
| Network Protocols | TTP, T3P, TDNS | 10_NETWORK_PROTOCOLS.md |
| Architecture | x86_64, aarch64, RISC-V abstraction | ARCHITECTURE.md |
| Hardware Drivers | TPU FPGA/ASIC, femtosecond clock | HARDWARE_DRIVERS.md |

### Extended Features (P3)

| Module | Description | Guide |
|--------|-------------|-------|
| TVM | 35-opcode ISA, GF(3) ALU, TAGC | [11_TVM.md](./modules/11_TVM.md) |
| Timing Protocol | HPTP, optical clocks, certification | 12_TIMING_PROTOCOL.md |
| BTG | Binary-ternary conversion, adapters | [13_BTG.md](./modules/13_BTG.md) |

---

## API Reference

Auto-generated documentation from source code:

```bash
# Generate rustdoc
cargo doc --open

# Or view online (when hosted)
# https://docs.salvi-framework.dev/api/
```

---

## Key Concepts

### Ternary Computing

| Concept | Binary | Ternary |
|---------|--------|---------|
| Digit values | 0, 1 | -1, 0, +1 (balanced) |
| Basic unit | Bit | Trit |
| Byte equivalent | 8 bits | 6 trits (Tryte) |
| Information density | 1 bit/digit | 1.58 bits/digit |

### GF(3) Arithmetic

Galois Field with 3 elements: {0, 1, 2}

| Operation | Definition |
|-----------|------------|
| Addition | (a + b) mod 3 |
| Multiplication | (a * b) mod 3 |
| Inverse | a^-1 such that a * a^-1 = 1 (mod 3) |

### Torsion Topology

- **Dimensions:** 7D (minimal) to 13D (full)
- **Nodes:** 3^N per N-dimensional torus
- **Routing:** Geodesic paths with torsion weighting

---

## Development

### Building from Source

```bash
git clone https://github.com/SigmaWolf-8/Ternary.git
cd Ternary
cargo build --release
```

### Running Tests

```bash
cargo test                    # All tests
cargo test --package salvi_kernel  # Specific package
cargo test -- --nocapture     # Show output
```

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy (`cargo clippy`)
- Document public APIs with rustdoc comments

---

## Project Structure

```
Ternary/
├── src/
│   ├── kernel/          # P1: Kernel Core
│   │   ├── memory/      # Frame allocator, page tables
│   │   ├── sync/        # Synchronization primitives
│   │   ├── process/     # Process management
│   │   ├── security/    # Modal security
│   │   └── crypto/      # Cryptographic primitives
│   ├── drivers/         # P1.5 + P2: Device drivers
│   ├── io/              # P1.5: I/O subsystem
│   ├── fs/              # P1.5: Filesystem
│   ├── arch/            # P2: Architecture modules
│   ├── network/         # P2: Torsion + protocols
│   ├── tvm/             # P3: Ternary Virtual Machine
│   ├── timing/          # P3: High-precision timing
│   └── btg/             # P3: Binary-Ternary Gateway
├── docs/                # Documentation (you are here)
├── tests/               # Integration tests
├── benches/             # Benchmarks
└── examples/            # Example programs
```

---

## Versioning

This documentation corresponds to:

- **Framework Version:** 1.0.0
- **Roadmap Version:** v1.6 FINAL
- **UPQTTI Whitepaper:** v4.21

---

## License

Capomastro Holdings Ltd. - All Rights Reserved and Preserved

---

## Support

- **Issues:** [GitHub Issues](https://github.com/SigmaWolf-8/Ternary/issues)
- **Discussions:** GitHub Discussions (when enabled)

---

*Cosi sia.*
