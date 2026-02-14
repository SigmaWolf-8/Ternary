# ADR-001: 62-Opcode Instruction Set for the PlenumNET VM

| Field       | Value                          |
|-------------|--------------------------------|
| **Status**  | Accepted                       |
| **Date**    | 2026-02-09                     |
| **Author**  | Salvi · Capomastro Holdings Ltd — Applied Physics Division |
| **Context** | PlenumNET Ternary VM · `libternary` Rust Kernel |

---

## 1 · Context

The PlenumNET virtual machine executes bijective ternary programs compiled through the Salvi Framework. Every opcode consumes exactly one *tryte* (a 3-trit word) from the instruction stream. The question is how many opcodes the VM should expose and why.

Balanced ternary (digits {−1, 0, +1}) yields 3^n unique states per n-trit word. A single tryte of 3 trits gives 27 states; a 4-trit word gives 81. Both extremes have problems:

* **27 opcodes** saturates the entire 3-trit encoding space, leaving zero room for inline operand hints, future extensions, or a `NOP`/`RESERVED` sentinel class. It also forces every cryptographic primitive (Lamport hash-chain ops, lattice reduction steps) to be macro-expanded from a dangerously small basis set — increasing code size and verification surface.
* **81 opcodes** (4 trits) wastes encoding bandwidth for a VM that is intentionally minimal. Most of those slots would sit empty, and the wider instruction word penalises the timing-critical fetch-decode path that must stay within HPTP synchronisation tolerances.

## 2 · Decision

**The VM exposes exactly 62 opcodes encoded in a 3-trit primary field plus a 1-trit extension flag (total: 4 trits per opcode word, but only 62 of the 81 slots are defined).**

The 62 opcodes partition into nine functional classes:

| Class | Count | Purpose |
|-------|-------|---------|
| **Core Arithmetic** | 8 | NOP, HALT, integer add, sub, mul, div, mod, neg with overflow detection |
| **Extended Ternary** | 16 | GF(3) field operations — TAdd, TMul, TNeg, TRot, TXor, TConvert, TAnd, TOr, TSub, TInv, TShift, TCmp, TLoad, TStore, TReduce, TRotInv |
| **Memory** | 6 | Load, store, move, load-immediate, push, pop — register and stack data transfer |
| **Control Flow** | 7 | Jump, conditional branches (zero, neg, pos, not-zero), call, return |
| **Comparison** | 2 | Register-register and register-immediate comparison with flag updates |
| **Bitwise** | 6 | AND, OR, XOR, SHL, SHR, NOT — standard logic operations |
| **Crypto Acceleration** | 8 | Polynomial multiply, NTT butterfly, ternary hash, entropy sample, polynomial add, polynomial sample, compress, decompress — the CNSA 2.0 critical path |
| **SIMD** | 4 | Vectorised ternary add, mul, neg, rot for parallel trit-vector arithmetic |
| **System** | 5 | Syscall, trap, memory alloc/free, HPTP time read |

### 2.1 · Why not 27?

Removing the extension flag and collapsing to 27 opcodes would force one of two bad trade-offs:

1. **Merge the Crypto and Timing classes** into overloaded opcodes selected by stack context. This creates implicit state that is hostile to formal verification — exactly the opposite of what a post-quantum system needs.
2. **Drop timing intrinsics** into userspace library calls. This destroys the nanosecond-determinism guarantee that HPTP alignment requires. The τ-register must be read atomically at the VM level; a library call crosses the FFI boundary and introduces jitter.

### 2.2 · Why not 81?

Filling 81 slots would require inventing opcodes that exist only for encoding symmetry. Empty opcode slots in a post-quantum VM are a security surface: a malicious program that lands on an undefined opcode must trap deterministically, and the more undefined slots exist, the more trap-path testing the implementation demands. 62 defined + 19 reserved-as-`ILLEGAL` is a better ratio than 27 + 0 or 81 + 0.

### 2.3 · Why exactly 62?

62 = 45 base opcodes + 17 extension opcodes. The base 45 opcodes cover the general-purpose VM (core arithmetic, extended ternary, memory, control flow, comparison, and bitwise). The 17 extension opcodes — accessible only when the extension trit is non-zero — are reserved for the crypto acceleration, SIMD, and system classes, which form the hottest path in post-quantum handshake execution. Separating them behind an extension flag means:

* General-purpose programs never pay the decode cost of crypto/SIMD/system opcodes.
* The crypto and SIMD opcodes can be hardware-accelerated (or FPGA-gated) independently.
* FIPS validation can scope its audit to the 8-opcode crypto surface without reviewing the entire instruction set.

## 3 · Consequences

* The instruction decoder in `libternary` is a two-stage pipeline: 3-trit primary decode → optional 1-trit extension decode. This is slightly more complex than a flat table but allows the timing class to sit alongside crypto without polluting the general namespace.
* Future opcode additions must justify consuming a reserved slot via a new ADR. The 19 reserved slots provide runway for future extensions.
* All 62 opcodes must have corresponding entries in the `verifyTau()` self-test matrix; any opcode without a timing-correctness proof is considered unshipped.

## 4 · Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| 27-opcode flat encoding | Insufficient separation of crypto/timing from general ops; hostile to FIPS scoping |
| 81-opcode 4-trit flat encoding | Excessive undefined-opcode surface; wasted fetch bandwidth |
| Variable-length encoding (CISC-style) | Incompatible with deterministic timing guarantees; decode jitter breaks HPTP sync |
| 32 opcodes (power-of-two compromise) | Misaligned with ternary word boundaries; wastes 5 encoding states in a 3^3 space |

---

*Così sia.*
