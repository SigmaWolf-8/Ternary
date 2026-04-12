# TritInt Audit Report

**Date:** 2026-04-11  
**Scope:** 20,847 lines across 35 files  
**Finding:** TritInt was built with hardcoded capacity caps that contradict the design (unbounded, auto-sizing). The caps survived six build phases and propagated into constructors, repr conversions, wire validation, WASM exports, and error types.

---

## Design vs Implementation

**Design (established 2026-04-08, restated repeatedly):**
> "No ceiling. No boundary. The ocean is the formula, and trit strings have no maximum width."  
> TritInt auto-sizes at runtime from the master formula. Unbounded. No u64 ceiling, no u128 ceiling. Grows as needed.

**Implementation (built across Phases 1–6):**
- Phase 1: `MAX_INLINE_TRITS = 40` — labeled "scaffolding, heap deferred"
- Phase 6: Added heap path but capped at `MAX_HEAP_TRITS = 9841`
- Neither cap was ever removed. Both are still live.

---

## File Inventory

| File | Lines | Tests | TritInt Role |
|------|-------|-------|-------------|
| trit_int.rs | 2,698 | 94 | Core type — WHERE THE CAPS LIVE |
| trit.rs | 1,438 | 74 | Three TritInts → algebraic triple |
| gf3_algebra.rs | 478 | 26 | Rep A↔B↔C↔D conversion |
| coprime.rs | 436 | 11 | Coprime walk, CRT — uses TritInt |
| ags.rs | 749 | 9 | Active Generator Set — uses TritInt |
| tri182.rs | 260 | 13 | ℤ[φ] arithmetic — uses TritInt |
| wasm_exports.rs | 645 | — | FFI boundary — ENFORCES CAPS |
| trit_benchmarks.rs | 90 | — | Benchmarks |
| constants.rs | 1,639 | — | Framework constants (TritInt consumer) |
| Other ternary-math | 2,730 | — | Various consumers |
| **ternary-math total** | **12,163** | **227** | |
| sponge.rs | 1,205 | — | RAW i8 TRITS — NO TritInt |
| tl_dsa.rs | 1,361 | — | RAW i8 TRITS — NO TritInt |
| tl_kem.rs | 532 | — | RAW i8 TRITS — NO TritInt |
| firmware_sign.rs | 266 | — | No TritInt |
| self_test.rs | 564 | — | No TritInt |
| inter-cube services | 4,756 | — | Only relay_circuit.rs uses TritInt |
| **Grand total** | **20,847** | | |

---

## Contamination Map

### CATEGORY 1: Constructors that panic on valid input

These functions refuse values that TritInt should accept.

| Function | Line | Cap | What happens |
|----------|------|-----|-------------|
| `from_trits()` | 609 | 40 trits | `pub const fn` — asserts `count <= MAX_INLINE_TRITS`. ALL repr conversions call this. |
| `from_u128()` | 639 | 40 trits | `pub const fn` — panics if `val > u64::MAX`. Phase 6 heap exists but from_u128 was never updated. |
| `from_u64()` | 628 | 40 trits | `pub const fn` — asserts in `from_u64_raw`. u64::MAX = 3⁴⁰ so this cap is technically not reachable, but the assert is still wrong in principle. |
| `repunit(n)` | 585 | 40 trits | `pub const fn` — asserts `n <= MAX_INLINE_TRITS`. Cannot create R₄₁ or larger. |
| `from_repr_a()` | 1436 | 40 trits | Calls `from_trits()` → inherits cap |
| `from_repr_b()` | 1478 | 40 trits | Calls `from_trits()` → inherits cap |
| `from_repr_c()` | 1495 | 40 trits | Calls `from_trits()` → inherits cap |
| `from_repr_d()` | 1508 | 40 trits | Calls `from_trits()` → inherits cap |
| `try_from_repr_c()` | 1659 | 9,841 trits | Rejects input > `MAX_HEAP_TRITS`. Bigger cage, still a cage. |

### CATEGORY 2: Const-path helpers that panic on Heap values

These are Rust `const fn` limitations — can't allocate in const context. The inline 40-trit cap for const is a language constraint, not a design choice. But it needs to be DOCUMENTED as such, not presented as the type's actual limit.

| Function | Line | Issue |
|----------|------|-------|
| `into_parts()` | 656 | Panics on Heap variant |
| `parts()` | 664 | Panics on Heap variant |
| `const_add()` | 719 | Uses `add_packed` → 40-trit assert |
| `const_sub()` | 727 | Uses `sub_packed` → 40-trit assert |
| `const_mul()` | 736 | Asserts `max_trits <= MAX_INLINE_TRITS` |
| `trit_shift_left()` | 1024 | Asserts `new_count <= MAX_INLINE_TRITS` |

### CATEGORY 3: Error type that shouldn't exist

| Item | Line | Issue |
|------|------|-------|
| `TritIntError::TooLong` | 1635 | Error variant for exceeding a cap that shouldn't exist |
| `Display for TooLong` | 1642 | "input exceeds TritInt maximum capacity (R₉ = 9841 trits)" |
| `MAX_HEAP_TRITS` const | 93 | `const MAX_HEAP_TRITS: u32 = 9841` — the cap itself |
| Const assert | 1339 | `assert!(MAX_HEAP_TRITS == 9841)` — test validating the wrong behavior |

### CATEGORY 4: WASM exports enforcing caps

| Function | Line | Issue |
|----------|------|-------|
| `trit_int_to_decimal()` | 409 | `if repr_c.len() > 40` hardcoded |
| `trit_int_display()` | 419 | `if repr_c.len() > 40` hardcoded |
| `trit_int_from_repr_c()` | 434 | `if repr_c.len() > 40` hardcoded |

### CATEGORY 5: Tests validating caps as correct

| Test | Line | Issue |
|------|------|-------|
| `try_from_repr_c_rejects_too_long` | 2496 | Asserts 9842 trits → `Err(TooLong)` |
| `try_from_repr_c_max_valid_length` | 2503 | Asserts 9841 trits → Ok |
| Inline size assertion | 1908 | `assert!(t.trit_length() <= 40)` |

---

## What Is Actually Clean

The runtime arithmetic path is UNCAPPED and works correctly:

| Function | Mechanism | Status |
|----------|-----------|--------|
| `add()` | → `add_gen()` → `make_from_trits()` → `make_from_packed()` | **CLEAN — auto-promotes to heap** |
| `sub()` | → `sub_gen()` → `make_from_trits()` → `make_from_packed()` | **CLEAN** |
| `mul()` | → `mul_gen()` → `make_from_trits()` → `make_from_packed()` | **CLEAN** |
| `div_mod()` | Runtime, uses `_gen` functions | **CLEAN** |
| `pow()` | Runtime, uses `mul()` | **CLEAN** |
| `gcd()` | Runtime | **CLEAN** |
| `extended_gcd()` | Runtime | **CLEAN** |
| `add_with_carry()` | Runtime | **CLEAN** |
| `div_repunit()` | Runtime | **CLEAN** |
| `mod_pow()` | Runtime | **CLEAN** |
| `make_from_packed()` | Auto-promotes to heap, NO cap | **CLEAN** |
| `to_repr_a/b/c/d()` | Output, works on any size | **CLEAN** |
| All accessors | `trit_length()`, `trit_at()`, etc. | **CLEAN** |
| `Display`, `Debug` | Work on any size | **CLEAN** |
| `PartialEq/Ord/Hash` | Use `_gen` comparators | **CLEAN** |
| Operator traits | Dispatch to clean runtime functions | **CLEAN** |
| `Zeroize` | Handles both Inline and Heap | **CLEAN** |
| `Serde` | Uses `to_repr_c()` / `try_from_repr_c()` | **CONTAMINATED by try_from_repr_c cap** |

---

## Separate Issue: Crypto Bypasses TritInt Entirely (INVARIANT 8)

The crypto modules (sponge.rs, tl_dsa.rs, tl_kem.rs) do NOT use TritInt. They operate on raw `i8` / `Vec<i8>` with manual decomposition functions:

| File | Raw trit type count | Manual decomposition |
|------|-------------------|---------------------|
| sponge.rs | 132 occurrences of `i8` | Internal trit manipulation |
| tl_dsa.rs | 91 occurrences of `i8` | `bytes_to_trits()`, `u16_to_trits()` |
| tl_kem.rs | 42 occurrences of `i8` | Via sponge + ternary_lattice |

TritInt is supposed to be THE gate (INVARIANT 8: "no bare integer holds a trit value above the gate"). The entire crypto subsystem operates below the gate. This is a separate remediation but should be on the radar.

---

## Fix List

### trit_int.rs — Required Changes

1. **`from_u128()`** — Add runtime version that auto-promotes to heap. Keep const version for compile-time use but rename to `from_u128_const()` or gate with a clear "const fn limitation" comment.

2. **`from_trits()`** — Add runtime version (`pub fn from_trits_rt()` or make `make_from_trits()` public). The four `from_repr_*` functions must call the runtime version, not the const version.

3. **`repunit(n)`** — Add runtime version for n > 40. Keep const for framework constants.

4. **`from_repr_a/b/c/d()`** — Route through runtime `make_from_trits()` instead of const `from_trits()`.

5. **`try_from_repr_c()`** — Remove `MAX_HEAP_TRITS` cap entirely. Accept any length.

6. **`TritIntError::TooLong`** — Remove the variant. The type has no maximum.

7. **`MAX_HEAP_TRITS`** — Delete the constant.

8. **Const assert at line 1339** — Delete `assert!(MAX_HEAP_TRITS == 9841)`.

9. **Tests** — Delete `try_from_repr_c_rejects_too_long`, `try_from_repr_c_max_valid_length`. Rewrite any test that validates caps as correct behavior.

### wasm_exports.rs — Required Changes

10. **Three functions** — Remove `if repr_c.len() > 40` guards. Route through `try_from_repr_c()` (which itself gets uncapped).

### Serde — Required Change

11. **Deserialize** — After `try_from_repr_c()` is uncapped, Serde automatically becomes uncapped. No separate fix needed.

### Const path — Document, Don't Fix

12. **`const_add/sub/mul`, `into_parts()`, `parts()`** — These are Rust `const fn` constraints (can't allocate). Keep them but add comments: "Const path: limited to inline (≤ 40 trits) by Rust const fn constraints. Use runtime `add()`/`sub()`/`mul()` for unbounded arithmetic."

---

## Files That Need No Changes

| File | Why |
|------|-----|
| trit.rs | Wraps TritInt — inherits fix automatically |
| gf3_algebra.rs | Returns Rep B values, doesn't construct TritInt |
| coprime.rs | Uses `from_u64()` on small values — fine |
| ags.rs | Uses const constructors on small values — fine |
| tri182.rs | Uses `from_trits()` on ≤ 5-element arrays — fine but will route through runtime path after fix |
| constants.rs | All framework constants fit in 40 trits — const path is correct here |
| All crypto files | Don't use TritInt (separate INVARIANT 8 issue) |
| inter-cube services | Only relay_circuit.rs uses small-value `from_u64()` — fine |

---

## Summary

- **20,847 total lines audited**
- **trit_int.rs: 12 specific contamination points** (constructors, error types, caps, const assert)
- **wasm_exports.rs: 3 hardcoded cap checks**
- **Runtime arithmetic: CLEAN — already unbounded**
- **Downstream consumers: functionally unaffected** (use small values) but `from_repr_*` calls would panic on large input
- **Crypto: separate INVARIANT 8 bypass** (no TritInt usage at all, raw i8 throughout)
