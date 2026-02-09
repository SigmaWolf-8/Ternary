# ADR-002: GF(3) Field Arithmetic as the Kernel Primitive

| Field       | Value |
|-------------|-------|
| **Status**  | Accepted |
| **Date**    | 2025-12-15 |
| **Author**  | Capomastro Holdings Ltd |
| **Context** | Choosing the correct algebraic foundation for all ternary operations in the kernel |

## 1 · Context

PlenumNET's kernel must perform arithmetic on ternary values. There are two natural choices for the underlying algebra:

1. **Balanced ternary arithmetic** — treating trits as integers in {-1, 0, +1} and using standard integer addition/multiplication with carries.
2. **GF(3) field arithmetic** — treating trits as elements of the Galois field with 3 elements, where addition and multiplication are performed modulo 3.

The distinction matters because balanced ternary arithmetic is *not* a field (there are no multiplicative inverses for all non-zero elements in the integer sense), while GF(3) *is* a field (every non-zero element has a multiplicative inverse: 1^-1 = 1, 2^-1 = 2).

An earlier implementation used balanced ternary with the mapping `f(a) = a + 1` to convert between {-1, 0, +1} and {0, 1, 2}. This mapping is **not** a ring homomorphism — it maps the balanced-ternary value -1 to 0, which is the additive identity in GF(3). This caused -1 to behave as zero in all arithmetic, producing incorrect results for 6 of the 9 entries in the addition table and 4 of the 9 entries in the multiplication table.

## 2 · Decision

All ternary arithmetic in the PlenumNET kernel operates in **GF(3)** using the standard modular equivalence for conversion between balanced ternary and GF(3):

```
toGF3(a)   = ((a % 3) + 3) % 3
fromGF3(g) = g > 1 ? g - 3 : g
```

This yields the correct mapping:

| Balanced Ternary | GF(3) |
|-----------------|-------|
| -1 | 2 |
| 0 | 0 |
| +1 | 1 |

Verification: -1 mod 3 = 2 (in the mathematical sense, not C's truncated remainder). This is a ring homomorphism preserving both addition and multiplication structure.

The kernel provides two canonical conversion functions — `toGF3()` and `fromGF3()` — and all code paths must use these functions exclusively. Direct arithmetic conversion (e.g., `a + 1`, `a - 1`, manual lookup tables) is prohibited.

Three trit representations are supported:

- **Representation A** (computational/balanced): {-1, 0, +1}
- **Representation B** (network/unbalanced): {0, 1, 2} — identical to GF(3)
- **Representation C** (human/bijective): {1, 2, 3}

Conversions between representations are performed via Representation A as the canonical intermediate form.

## 3 · Consequences

**Positive:**
- All GF(3) arithmetic is provably correct. The 9-entry addition and multiplication tables match the field axioms exactly, and can be exhaustively verified on every CI run.
- Field properties (associativity, commutativity, distributivity, identity, inverse) hold by construction, enabling algebraic optimizations in the compiler/VM.
- The mapping preserves the group structure: `toGF3(a + b) = toGF3(a) + toGF3(b)` in GF(3), and likewise for multiplication. This is critical for the correctness of phase encryption and hash functions that compose multiple GF(3) operations.
- Constant-time implementation is straightforward — the modular operations involve no data-dependent branches.

**Negative:**
- The `((a % 3) + 3) % 3` formula requires two modular reductions, which is slightly more expensive than `a + 1`. On modern hardware this is negligible (1-2 extra cycles), but it must be noted for the FPGA/ASIC hardware driver targets where cycle budgets are tight.
- Contributors must understand the distinction between balanced ternary (a notation) and GF(3) (an algebraic structure). The CONTRIBUTING.md covers this, but the conceptual overhead is real.
- Legacy code using the old `a + 1` mapping must be identified and migrated. A `grep` for `+ 1` and `- 1` in ternary conversion contexts is part of the migration checklist.

## 4 · Alternatives Considered

**Balanced ternary with carry propagation (integer arithmetic):**
Rejected. While balanced ternary is a valid positional numeral system, single-trit operations without carry propagation do not form a field. Specifically, there is no multiplicative inverse for -1 in the integer sense (-1 * x = 1 has no solution in {-1, 0, +1} under integer multiplication). GF(3) arithmetic is required for the cryptographic and algebraic operations in the Salvi Framework.

**The `f(a) = a + 1` mapping:**
Rejected. This maps -1 to 0 (the additive identity), which is not a homomorphism. Proof by counterexample: `f(-1) * f(-1) = 0 * 0 = 0`, but `f((-1) * (-1)) = f(+1) = 2`. Since `0 != 2`, the mapping does not preserve multiplication. This caused silent arithmetic errors in the original implementation.

**Lookup table instead of formula:**
Considered but not adopted as the primary implementation. A 3-element lookup table for toGF3 (`[-1 -> 2, 0 -> 0, 1 -> 1]`) is constant-time and avoids the modular arithmetic, but introduces a branch or memory access that may not be constant-time on all architectures. The formula `((a % 3) + 3) % 3` is preferred for its portability and verifiability. Lookup tables may be used as an optimization in hardware-specific paths (FPGA drivers) where the memory access pattern is guaranteed constant-time.
