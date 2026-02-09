# ADR-001: 35-Opcode Instruction Set for the PlenumNET VM

| Field       | Value |
|-------------|-------|
| **Status**  | Accepted |
| **Date**    | 2025-12-15 |
| **Author**  | Capomastro Holdings Ltd |
| **Context** | Defining the instruction set architecture for the Ternary Virtual Machine (TVM) |

## 1 · Context

PlenumNET requires a virtual machine capable of executing ternary programs natively on binary hardware. The VM must support GF(3) field arithmetic as first-class operations alongside conventional integer arithmetic, memory access, and control flow. The instruction set must be compact enough for efficient WASM compilation while expressive enough to implement the full Salvi Framework — including phase encryption, timing verification, and ternary data processing.

The number of general-purpose registers is fixed at 27 (3^3), reflecting the ternary architecture. The instruction encoding must be fixed-width for deterministic decoding and cycle counting.

## 2 · Decision

The TVM uses a **35-opcode instruction set** organized into six categories:

| Category | Opcodes | Count |
|----------|---------|-------|
| **Arithmetic** | NOP, HALT, ADD, SUB, MUL, DIV, MOD, NEG | 8 |
| **Ternary** | TADD, TMUL, TNEG, TROT, TXOR, TCONVERT | 6 |
| **Memory** | LOAD, STORE, MOVE, LOADIMM, PUSH, POP | 6 |
| **Control** | JUMP, JUMPZERO, JUMPNEG, JUMPPOS, CALL, RETURN, JUMPNOTZERO | 7 |
| **Compare** | CMP, CMPIMM | 2 |
| **Bitwise** | AND, OR, XOR, SHL, SHR, NOT | 6 |

Key design properties:

- **Fixed-width 16-byte instructions.** Each instruction encodes: opcode (1 byte), dst register (1 byte), src1 register (1 byte), src2 register (1 byte), immediate value (8 bytes), padding (4 bytes). This enables O(1) instruction fetch and deterministic cycle counting.
- **27 general-purpose registers** (R0-R26), each 64 bits wide. 27 = 3^3 aligns with the ternary architecture.
- **Dedicated ternary opcodes.** TADD, TMUL, TNEG, TROT, TXOR, and TCONVERT operate in GF(3) directly, avoiding the overhead of encoding GF(3) operations as sequences of binary arithmetic.
- **TCONVERT** bridges between balanced ternary {-1,0,+1}, unbalanced {0,1,2}, and bijective {1,2,3} representations within the VM, enabling zero-copy interop between representation conventions.
- **Stack model** is LIFO with a maximum depth of 4096, supporting PUSH/POP for register saves and CALL/RETURN for subroutine linkage.

Theory-derived parameters within the VM:

- Finalization rounds: 13 (from T(7) = 13, the dimensional constant)
- Hash seed base: tau^2 (from SO(8) graph stability)
- Hash mixing multiplier: tau^7 (instanton action volume)
- GC cycle interval: tau^13 (fundamental period constant)

## 3 · Consequences

**Positive:**
- The 35-opcode set is small enough that exhaustive conformance testing is feasible (all opcodes can be tested with bounded input spaces).
- Fixed-width encoding eliminates variable-length decode complexity and enables WASM compilation with predictable performance.
- Dedicated ternary opcodes make GF(3) arithmetic a first-class citizen, avoiding lossy binary emulation of ternary operations.
- The ISA is formally specified in machine-readable YAML/JSON (`src/kernel/spec/tvm-isa-v1.yaml`), enabling automated conformance test generation.

**Negative:**
- 35 opcodes leaves limited room for future expansion without breaking the encoding scheme. Reserved opcode slots should be consumed sparingly (new opcodes require an ADR).
- Fixed-width 16-byte encoding wastes 4 bytes of padding per instruction. This is an acceptable trade-off for decode simplicity, but increases program memory footprint by ~33% compared to a variable-width encoding.
- The register file is intentionally small (27 registers). Programs requiring more working state must spill to memory via PUSH/POP, adding latency.

## 4 · Alternatives Considered

**Variable-width instruction encoding (2-16 bytes):**
Rejected. Variable-width decoding adds branch prediction complexity and makes cycle counting non-deterministic, which conflicts with HPTP timing guarantees.

**64 opcodes (6-bit opcode field):**
Rejected. A larger opcode space invites feature creep. 35 opcodes cover arithmetic, ternary ops, memory, control flow, comparison, and bitwise — the minimal set needed for the Salvi Framework. Additional functionality (e.g., SIMD-style trit-vector operations) can be implemented as library routines composed from the base opcodes.

**Stack-based architecture (no registers):**
Rejected. Stack-based VMs have lower code density for the expression patterns common in GF(3) arithmetic (many binary operations on the same operands). A register machine with 27 registers reduces redundant loads and stores.

**Binary-only opcodes with GF(3) as a library:**
Rejected. Encoding GF(3) addition as `((a + b) % 3)` using binary ADD and MOD requires 3 instructions per trit operation and introduces timing variability from the MOD division. A dedicated TADD opcode is constant-time and single-cycle.
