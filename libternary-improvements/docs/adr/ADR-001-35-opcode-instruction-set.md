# ADR-001: 55-Opcode Instruction Set for the PlenumNET VM

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

**The VM exposes exactly 55 opcodes encoded in a 3-trit primary field plus a 1-trit extension flag (total: 4 trits per opcode word, but only 55 of the 81 slots are defined).**

The 55 opcodes partition into five functional classes:

| Class | Count | Purpose |
|-------|-------|---------|
| **Core** | 14 | Trit-native add, mul, inv, GF(3) field operations, stack primitives, and algebraic self-tests |
| **Extended** | 11 | Advanced memory, load/store, dup, swap, rot, and control flow — jump, branch-on-trit, call, return, halt |
| **Crypto Acceleration** | 14 | Lamport one-time signature ops (hash-chain step, leaf-verify, Merkle-path-check), lattice basis sample, NTT butterfly, modular reduce, AES-256-GCM, SHA-2/SHA-3 — the CNSA 2.0 critical path |
| **SIMD** | 10 | Ternary SIMD operations for parallel trit-vector arithmetic, batch GF(3) ops, and vectorised crypto primitives |
| **Timing & Density** | 6 | τ-register read/write, density-field sample, HPTP sync pulse, epoch-boundary fence, `verifyTau` intrinsic, timing self-test trigger |

### 2.1 · Why not 27?

Removing the extension flag and collapsing to 27 opcodes would force one of two bad trade-offs:

1. **Merge the Crypto and Timing classes** into overloaded opcodes selected by stack context. This creates implicit state that is hostile to formal verification — exactly the opposite of what a post-quantum system needs.
2. **Drop timing intrinsics** into userspace library calls. This destroys the nanosecond-determinism guarantee that HPTP alignment requires. The τ-register must be read atomically at the VM level; a library call crosses the FFI boundary and introduces jitter.

### 2.2 · Why not 81?

Filling 81 slots would require inventing opcodes that exist only for encoding symmetry. Empty opcode slots in a post-quantum VM are a security surface: a malicious program that lands on an undefined opcode must trap deterministically, and the more undefined slots exist, the more trap-path testing the implementation demands. 55 defined + 26 reserved-as-`ILLEGAL` is a better ratio than 27 + 0 or 81 + 0.

### 2.3 · Why exactly 55?

55 = 27 + 28. The base 27 (3^3) opcodes cover the general-purpose VM (core + extended). The 28 extension opcodes — accessible only when the extension trit is non-zero — are reserved for the cryptographic, SIMD, and timing classes, which form the hottest path in post-quantum handshake execution. The number 28 itself derives from the Tribonacci mod-28 symmetry that governs the indexing layer. Separating them behind an extension flag means:

* General-purpose programs never pay the decode cost of crypto/SIMD opcodes.
* The crypto and SIMD opcodes can be hardware-accelerated (or FPGA-gated) independently.
* FIPS validation can scope its audit to the 14-opcode crypto surface without reviewing the entire instruction set.

## 3 · Consequences

* The instruction decoder in `libternary` is a two-stage pipeline: 3-trit primary decode → optional 1-trit extension decode. This is slightly more complex than a flat table but allows the timing class to sit alongside crypto without polluting the general namespace.
* Future opcode additions must justify consuming a reserved slot via a new ADR. The 26 reserved slots provide ample runway.
* All 55 opcodes must have corresponding entries in the `verifyTau()` self-test matrix; any opcode without a timing-correctness proof is considered unshipped.

## 4 · Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| 27-opcode flat encoding | Insufficient separation of crypto/timing from general ops; hostile to FIPS scoping |
| 81-opcode 4-trit flat encoding | Excessive undefined-opcode surface; wasted fetch bandwidth |
| Variable-length encoding (CISC-style) | Incompatible with deterministic timing guarantees; decode jitter breaks HPTP sync |
| 32 opcodes (power-of-two compromise) | Misaligned with ternary word boundaries; wastes 5 encoding states in a 3^3 space |

---

*Così sia.*
