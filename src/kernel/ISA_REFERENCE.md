# Salvi Framework — Ternary VM ISA Reference

Version 1.0 · Salvi Framework / PlenumNET Platform
Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada) — Patent(s) Pending

---

## 1. Overview

The Ternary Virtual Machine (TVM) is a 27-register balanced-ternary processor that executes both conventional binary integer arithmetic and native GF(3) trit-parallel operations on packed 27-trit words stored in 64-bit registers.

Key characteristics:

- **27 general-purpose registers** (r0–r26), each holding a 64-bit value with a per-register ternary-mode flag.
- **Packed-trit architecture**: 27 trits encoded in 54 bits of an `i64` using 2-bit-per-trit encoding.
- **Dual-mode execution**: instructions operate on scalar integers or packed trit words depending on register mode.
- **Three instruction encodings**: Legacy (16 B), Compact (4/6 B), Balanced Ternary (5/7 B).
- **GF(3) native operations**: addition, multiplication, rotation, Kleene logic, Łukasiewicz conjunction, NTT, polynomial arithmetic, and SIMD vector variants.
- **Two-ring privilege model** (Ring0 / Ring1) with hardware-style trap and syscall support.
- **Cycle-counted execution** with configurable max-cycle limits and time-slice preemption.

---

## 2. Register File

### 2.1 General-Purpose Registers

| Register | Width | Description |
|----------|-------|-------------|
| r0–r26 | 64-bit | General-purpose; each has an independent `ternary_mode` flag |

When `ternary_mode` is **true**, the register value is interpreted as a packed 27-trit word. Binary operations that produce a ternary result automatically set this flag on the destination register.

### 2.2 Special Registers

| Register | Width | Description |
|----------|-------|-------------|
| PC | 64-bit | Program counter (instruction index, not byte offset) |
| SP | 64-bit | Stack pointer (managed via Push/Pop through an internal value stack) |

### 2.3 Flags Register

| Flag | Bit | Set when… |
|------|-----|-----------|
| zero | — | Result == 0 |
| negative | — | Result < 0 (signed) |
| positive | — | Result > 0 |
| overflow | — | Arithmetic overflow detected |
| ternary | — | (per-register) Register contains packed trit data |
| halted | — | HALT instruction has executed; VM stops stepping |

### 2.4 Privilege Levels

| Level | Value | Description |
|-------|-------|-------------|
| Ring0 | 0 | Kernel / supervisor — full access |
| Ring1 | 1 | User — restricted; `TRAP` requires Ring0 |

Default privilege after reset is **Ring0**.

---

## 3. Trit Representations

### 3.1 Three Bijective Encodings

| Name | Symbol | Domain | Use |
|------|--------|--------|-----|
| Computational | A | {−1, 0, +1} | Arithmetic, GF(3) field ops |
| Network | B | {0, 1, 2} | Wire encoding, serialisation |
| Human | C | {1, 2, 3} | Display, debugging |

**Bijections:**

| From → To | Formula |
|-----------|---------|
| A → B | b = a + 1 |
| A → C | c = a + 2 |
| B → C | c = b + 1 |

### 3.2 Packed Trit Word (27 trits in i64)

Each trit occupies 2 bits in the low 54 bits of a `u64`:

| 2-bit code | Trit value (Rep A) |
|------------|-------------------|
| `00` | 0 |
| `01` | +1 |
| `10` | −1 |
| `11` | (invalid) |

Trit index `i` is stored at bits `[2i .. 2i+1]`. Bits `[54..63]` are reserved and must be zero.

### 3.3 Tryte

A tryte is 6 trits (729 states ≈ 9.5 bits). A ternary word is 27 trits = 3 trytes × 9 trits or 4.5 trytes.

---

## 4. Instruction Encoding Formats

### 4.1 Legacy (16 bytes)

```
Byte  0       : opcode (u8)
Byte  1       : dst    (u8, register index 0-26)
Byte  2       : src1   (u8)
Byte  3       : src2   (u8)
Bytes 4-11    : immediate (i64, little-endian)
Bytes 12-15   : reserved (zero)
```

Fixed 16-byte width. Full 64-bit immediate range.

### 4.2 Compact (4 or 6 bytes)

```
Byte  0       : opcode (u8)
Bytes 1-2     : packed register word (u16, little-endian)
                  [15:11] dst   (5 bits)
                  [10:6]  src1  (5 bits)
                  [5:1]   src2  (5 bits)
                  [0]     has_imm flag
Byte  3       : reserved (format A, 4 bytes total)
—— or ——
Bytes 3-4     : immediate (i16, little-endian, clamped to ±32767)
Byte  5       : reserved (format B, 6 bytes total)
```

### 4.3 Balanced Ternary (5 or 7 bytes)

```
Bytes 0-2     : ternary-encoded opcode (3 bytes, 9 trits in base-3 → u24)
Bytes 3-4     : packed register word (u16, same layout as Compact)
—— if has_imm ——
Bytes 5-6     : immediate (i16, little-endian)
```

The opcode byte value is converted to base-3 digits stored as 2-bit pairs across 3 bytes (9 trit positions × 2 bits = 18 bits used of 24).

---

## 5. Opcode Table

### 5.1 Basic Arithmetic (0x00–0x07)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x00 | NOP | — | — | No operation |
| 0x01 | HALT | HLT | — | Stop execution; set halted flag |
| 0x02 | ADD | — | dst, src1, src2 | dst ← src1 + src2 (i64, overflow-detecting) |
| 0x03 | SUB | — | dst, src1, src2 | dst ← src1 − src2 |
| 0x04 | MUL | — | dst, src1, src2 | dst ← src1 × src2 |
| 0x05 | DIV | — | dst, src1, src2 | dst ← src1 / src2 (trap on zero) |
| 0x06 | MOD | — | dst, src1, src2 | dst ← src1 % src2 (trap on zero) |
| 0x07 | NEG | — | dst, src1 | dst ← −src1 |

### 5.2 Ternary Core (0x10–0x1F)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x10 | TADD | — | dst, src1, src2 | GF(3) trit-wise addition |
| 0x11 | TMUL | — | dst, src1, src2 | GF(3) trit-wise multiplication |
| 0x12 | TNEG | — | dst, src1 | Trit-wise negation (NOT): −t per trit |
| 0x13 | TROT | — | dst, src1, src2 | Trit-wise rotation (−1→0→+1→−1) by src2 positions |
| 0x14 | TXOR | — | dst, src1, src2 | Ternary XOR: Kleene min(a, b) per trit |
| 0x15 | TCONVERT | TCVT | dst, src1, src2, #imm | Convert trits: src2 = from-repr (0=A,1=B,2=C), imm = to-repr |
| 0x16 | TAND | — | dst, src1, src2 | Łukasiewicz conjunction: max(a+b−1, −1) per trit |
| 0x17 | TOR | — | dst, src1, src2 | Kleene disjunction: max(a, b) per trit |
| 0x18 | TSUB | — | dst, src1, src2 | GF(3) trit-wise subtraction: add(a, not(b)) |
| 0x19 | TINV | — | dst, src1 | GF(3) multiplicative inverse per trit (nonzero trits are self-inverse) |
| 0x1A | TSHIFT | TSHL | dst, src1, src2/imm | Trit shift: positive = left, negative = right; zero-fill |
| 0x1B | TCMP | — | dst, src1, src2 | Trit-wise compare: −1 if a<b, 0 if a==b, +1 if a>b |
| 0x1C | TLOAD | TLD | dst, src1, #imm | Load i64 from memory using ternary or binary address |
| 0x1D | TSTORE | TST | dst, src1, #imm | Store dst to memory using ternary or binary address |
| 0x1E | TREDUCE | TRED | dst, src1, #imm | Reduce all 27 trits via gate (0=add, 1=mul, 2=min, 3=max) |
| 0x1F | TROTINV | TROTI | dst, src1 | Trit-wise inverse rotation (+1→0→−1→+1) |

### 5.3 Memory & Register (0x20–0x25)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x20 | LOAD | LD | dst, src1, #imm | dst ← mem[src1 + imm] (8-byte read) |
| 0x21 | STORE | ST | dst, src1, #imm | mem[src1 + imm] ← dst (8-byte write) |
| 0x22 | MOVE | MOV | dst, src1 | dst ← src1 |
| 0x23 | LOADIMM | LDI | dst, #imm | dst ← immediate |
| 0x24 | PUSH | — | src1 | Push src1 onto value stack |
| 0x25 | POP | — | dst | Pop top of value stack into dst |

### 5.4 Control Flow (0x30–0x36)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x30 | JUMP | JMP | #imm | PC ← imm (unconditional) |
| 0x31 | JUMPZERO | JZ | #imm | PC ← imm if zero flag set |
| 0x32 | JUMPNEG | JN | #imm | PC ← imm if negative flag set |
| 0x33 | JUMPPOS | JP | #imm | PC ← imm if !zero && !negative |
| 0x34 | CALL | — | #imm | Push PC; PC ← imm |
| 0x35 | RETURN | RET | — | PC ← pop() |
| 0x36 | JUMPNOTZERO | JNZ | #imm | PC ← imm if !zero |

### 5.5 Comparison (0x40–0x41)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x40 | CMP | — | src1, src2 | Update flags from src1 − src2 (result discarded) |
| 0x41 | CMPIMM | CMPI | src1, #imm | Update flags from src1 − imm |

### 5.6 Binary Logic (0x50–0x55)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x50 | AND | — | dst, src1, src2 | dst ← src1 & src2 (bitwise) |
| 0x51 | OR | — | dst, src1, src2 | dst ← src1 \| src2 |
| 0x52 | XOR | — | dst, src1, src2 | dst ← src1 ^ src2 |
| 0x53 | SHL | — | dst, src1, src2 | dst ← src1 << (src2 & 63) |
| 0x54 | SHR | — | dst, src1, src2 | dst ← src1 >> (src2 & 63) (arithmetic) |
| 0x55 | NOT | — | dst, src1 | dst ← !src1 (bitwise complement) |

### 5.7 Crypto Acceleration (0x60–0x67)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x60 | TPOLYMUL | — | dst, src1, src2, #imm | GF(3) polynomial multiplication mod x^d (d = imm or 13) |
| 0x61 | TNTT | — | dst, src1, #imm | Number-Theoretic Transform (imm=0 forward, imm≠0 inverse) |
| 0x62 | THASH | — | dst, src1 | Ternary sponge-based hash of packed trit word |
| 0x63 | TENTROPY | — | dst, src1, src2 | Generate pseudorandom packed trit word from two seeds + cycle counter |
| 0x64 | TPOLYADD | — | dst, src1, src2 | Coefficient-wise GF(3) polynomial addition |
| 0x65 | TPOLYSAMPLE | TPSAMP | dst, src1 | Sample random GF(3) polynomial from seed |
| 0x66 | TCOMPRESS | TCOMP | dst, src1, src2 | Compress: strip zero trits; count written to src2 |
| 0x67 | TDECOMPRESS | TDCOMP | dst, src1, src2 | Decompress: expand src2 nonzero trits back to 27 positions |

### 5.8 SIMD Vector (0x70–0x73)

These operate identically to their scalar counterparts but always set `ternary_mode = true` on the destination and treat operands as packed 27-trit words unconditionally.

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x70 | TADDV | — | dst, src1, src2 | Vector GF(3) addition (27 trits parallel) |
| 0x71 | TMULV | — | dst, src1, src2 | Vector GF(3) multiplication |
| 0x72 | TNEGV | — | dst, src1 | Vector trit negation |
| 0x73 | TROTV | — | dst, src1, src2/imm | Vector trit-position rotation (circular left by n positions) |

### 5.9 System (0x80–0x84)

| Hex | Mnemonic | Alias | Operands | Description |
|-----|----------|-------|----------|-------------|
| 0x80 | SYSCALL | SYS | dst, src1, src2 | System call; src1 = syscall number, src2 = arg; result in dst |
| 0x81 | TRAP | INT | #imm | Ring0-only trap; raises error with trap code |
| 0x82 | ALLOC | — | dst, src1, #imm | GC-managed allocation; size = src1, type = imm; handle in dst |
| 0x83 | FREE | — | dst, src1 | Remove GC root for handle in src1; dst ← 0 |
| 0x84 | READTIME | RDTIME | dst, src1, #imm | dst ← HPTP timestamp component selected by imm (see below) |

**READTIME Component Selector (via READTIME dst, src1, #imm):**

Timestamp is obtained from the injected `HptpProvider` trait object. The VM
constructor `TernaryVm::new(memory_size, Box<dyn HptpProvider>)` enforces
explicit provider injection at compile time — there are no hidden defaults.

Provider implementations:
- `SimulatedHptp::new().with_epoch(fs).with_cycle_period(fs)` — deterministic
  builder for tests/simulation (default: epoch=0, period=1000 fs).
- `LiveHptp::new(callback, source)` — production hardware clocks via callback.
- Hot-swap at runtime via `vm.set_hptp_provider(new_provider)`.

| imm | Returns |
|-----|---------|
| 0 | Atomic fs pair — low 64 bits in dst, high 64 bits auto-stored in src1 register |
| 1 | Seconds since Salvi Epoch |
| 2 | Milliseconds component |
| 3 | Nanoseconds component |
| 4 | Picoseconds component |
| 5 | Remaining femtoseconds (sub-picosecond residual) |
| 6 | Raw cycle count |
| 7 | TimingSource discriminant (for clock quality branching) |

**Syscall Numbers (via SYSCALL src1):**

| Number | Returns |
|--------|---------|
| 0 | 0 (no-op) |
| 1 | Current cycle count |
| 2 | 27 (trit word width) |
| 3 | Memory size in bytes |
| 4 | Current security domain |

---

## 6. Semantic Details

### 6.1 GF(3) Field Operations

All ternary operations work in GF(3) using balanced representation A = {−1, 0, +1}.

| Operation | Definition | Identity |
|-----------|-----------|----------|
| TAdd | (a + b) mod 3, normalized to {−1,0,+1} | 0 |
| TMul | (a × b) mod 3, normalized | 1 |
| TSub | add(a, not(b)) | — |
| TNeg/NOT | −a | — |

### 6.2 Ternary Logic Operations

| Operation | Definition | Logic Family |
|-----------|-----------|--------------|
| TXor | min(a, b) | Kleene strong (min) |
| TAnd | max(a + b − 1, −1) | Łukasiewicz conjunction |
| TOr | max(a, b) | Kleene strong disjunction |
| TCmp | sgn(a − b) → {−1, 0, +1} | Three-valued comparison |

### 6.3 Rotation

`TRot` cycles each trit through: −1 → 0 → +1 → −1 (bijective, period 3).
`TRotInv` is the inverse: +1 → 0 → −1 → +1.
`rotate · rotate_inverse = identity` for all trit values.

### 6.4 GF(3) Multiplicative Inverse

Every nonzero element of GF(3) is its own inverse: inv(+1) = +1, inv(−1) = −1, inv(0) = 0.

### 6.5 Packed Reduction (TReduce)

Folds all 27 trits of a packed word using a specified gate:

| Gate (imm & 0x03) | Operation |
|-------------------|-----------|
| 0 | GF(3) addition |
| 1 | GF(3) multiplication |
| 2 | min |
| 3 | max |

### 6.6 Dual-Mode Dispatch

Most ternary instructions (`TAdd`, `TMul`, `TXor`, `TAnd`, `TOr`, `TSub`, `TCmp`) check `ternary_mode` on the source registers:

- **If either source is in ternary mode**: operate element-wise across all 27 packed trits via `packed_zip`.
- **Otherwise**: normalize the scalar `i64` to a single trit via `scalar_to_trit` (val mod 3 → {−1,0,+1}), perform the operation, and store the scalar result.

Destination `ternary_mode` is propagated (set to true if either source was ternary).

---

## 7. Addressing Modes

### 7.1 Standard Binary Addressing

Used by `LOAD` and `STORE`:

```
effective_address = register[src1] + immediate
```

Accesses 8 bytes (i64) at the computed byte address.

### 7.2 Ternary Balanced Addressing

Used by `TLOAD` and `TSTORE` when `src1` is in ternary mode:

```
trits = unpack_trits(register[src1])
address = Σ (trit[i].to_a() × 3^i)  for i = 0..26
effective_address = address + immediate
```

When `src1` is **not** in ternary mode, falls back to standard binary addressing.

`TLOAD` always sets `ternary_mode = true` on the destination register.

---

## 8. Privilege Model

| Instruction | Required Level |
|-------------|---------------|
| TRAP | Ring0 |
| All others | Ring0 or Ring1 |

Executing a privileged instruction from Ring1 raises a `VmError` ("Privilege violation: Ring0 required").

The privilege level can be changed programmatically via `set_privilege()` on the VM instance. The security domain is a separate `u8` field queryable via syscall 4.

---

## 9. VM Architecture

### 9.1 Memory

- Flat byte-addressable memory (`VmMemory`) of configurable size.
- Supports `read_u8`, `write_u8`, `read_i64`, `write_i64`, `read_bytes`.
- Out-of-bounds access raises `SegmentationFault` or `InvalidMemoryAccess`.

### 9.2 Stack

- Separate value stack (`VmStack`) with configurable max depth (default 4096 entries).
- Stores `i64` values.
- Overflow and underflow are trapped.

### 9.3 Garbage Collector

- `GcHeap` with typed allocations (Integer, TernaryValue, Array, String, Closure, Custom).
- Handles are integer indices returned by `ALLOC` and released by `FREE`.

### 9.4 Execution Model

- Instruction-indexed PC (not byte-indexed).
- Single-step (`step()`) or run-to-halt (`run()`).
- Configurable `max_cycles` (default 1,000,000) — exceeding raises an error.
- Time-slice preemption: each step decrements `time_remaining`; query `is_time_slice_exhausted()`.

---

## 10. Toolchain

### 10.1 Assembler

Module: `vm::assembler::Assembler`

**Syntax:**

```asm
; comment
label:
  MNEMONIC dst, src1, src2, #immediate
  MNEMONIC dst, src1, src2
  MNEMONIC @label              ; label reference resolved to instruction index
```

- Registers: `r0`–`r26` (case-insensitive)
- Immediates: `#value` or bare integer
- Labels: identifier followed by `:`; referenced with `@label`
- Comments: `;` to end of line

### 10.2 Disassembler

Module: `vm::assembler::Disassembler`

- `disassemble_instruction(inst)` → single-line text
- `disassemble_program(prog)` → numbered listing with program name header

Preferred short mnemonics are used in output (e.g., `TCVT`, `TLD`, `TST`, `JMP`, `JZ`, `MOV`, `LDI`, `RET`, `SYS`, `RDTIME`).

### 10.3 Debugger

Module: `vm::debugger::VmDebugger`

Capabilities:

- **Breakpoints**: add/remove/toggle by instruction address; hit counting.
- **Register snapshots**: capture full register state (values + ternary mode flags), PC, flags, and cycle count.
- **Execution history**: ring buffer of up to 1000 snapshots.
- **Step mode**: single-instruction stepping with breakpoint checking.
- **Watch expressions**: monitor register or memory changes.
- **Formatted state dump**: human-readable register file, flags, and memory hex dump.

### 10.4 THDL Compiler

Crate: `src/thdl`

The Ternary Hardware Description Language compiler translates high-level ternary circuit descriptions into TVM programs.

### 10.5 Mnemonic / Alias Quick Reference

| Full Mnemonic | Short Alias(es) |
|---------------|-----------------|
| HALT | HLT |
| TCONVERT | TCVT |
| TSHIFT | TSHL |
| TLOAD | TLD |
| TSTORE | TST |
| TREDUCE | TRED |
| TROTINV | TROTI |
| LOAD | LD |
| STORE | ST |
| MOVE | MOV |
| LOADIMM | LDI |
| JUMP | JMP |
| JUMPZERO | JZ |
| JUMPNEG | JN |
| JUMPPOS | JP |
| JUMPNOTZERO | JNZ |
| RETURN | RET |
| CMPIMM | CMPI |
| TPOLYSAMPLE | TPSAMP |
| TCOMPRESS | TCOMP |
| TDECOMPRESS | TDCOMP |
| SYSCALL | SYS |
| TRAP | INT |
| READTIME | RDTIME |

---

## Appendix A — Opcode Map (Sorted by Hex)

```
0x00  NOP          0x10  TADD         0x20  LOAD         0x30  JUMP
0x01  HALT         0x11  TMUL         0x21  STORE        0x31  JUMPZERO
0x02  ADD          0x12  TNEG         0x22  MOVE         0x32  JUMPNEG
0x03  SUB          0x13  TROT         0x23  LOADIMM      0x33  JUMPPOS
0x04  MUL          0x14  TXOR         0x24  PUSH         0x34  CALL
0x05  DIV          0x15  TCONVERT     0x25  POP          0x35  RETURN
0x06  MOD          0x16  TAND                             0x36  JUMPNOTZERO
0x07  NEG          0x17  TOR
                   0x18  TSUB         0x40  CMP
                   0x19  TINV         0x41  CMPIMM
                   0x1A  TSHIFT
                   0x1B  TCMP         0x50  AND          0x60  TPOLYMUL
                   0x1C  TLOAD        0x51  OR           0x61  TNTT
                   0x1D  TSTORE       0x52  XOR          0x62  THASH
                   0x1E  TREDUCE      0x53  SHL          0x63  TENTROPY
                   0x1F  TROTINV      0x54  SHR          0x64  TPOLYADD
                                      0x55  NOT          0x65  TPOLYSAMPLE
                                                         0x66  TCOMPRESS
0x70  TADDV                                              0x67  TDECOMPRESS
0x71  TMULV
0x72  TNEGV        0x80  SYSCALL
0x73  TROTV        0x81  TRAP
                   0x82  ALLOC
                   0x83  FREE
                   0x84  READTIME
```

---

*End of ISA Reference.*
