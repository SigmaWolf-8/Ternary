# FORMA CODEX 18∏ — Implementation Architecture

```
Copyright © 2026 Capomastro Holdings Ltd. — All Rights Reserved
Patent(s) Pending — Applied Physics Division

Rewritten against actual source code (17 Rust files, 6,239 lines verified).
All decisions from 2026-04-07/08 session applied.
```

---

## 1. The Gate

Binary below. Ternary above. Translation happens once at the boundary.

```
┌─────────────────────────────────────┐
│  Forma Codex, coprime walks,        │
│  grid engine, constants, presets    │  ternary-native (above the gate)
│  gf3_algebra — all new code         │
├─────────────────────────────────────┤
│  THE GATE — trit_int.rs             │  binary↔ternary boundary
│  TritInt, Trit (one type, 4 views)  │
├─────────────────────────────────────┤
│  Binary hardware, WASM/JS boundary, │
│  DO-NOT-MODIFY modules              │  binary (current reality)
└─────────────────────────────────────┘
```

---

## 2. One Trit, Four Views

```
     Rep A    Rep B    Rep C    Rep D
     (bal)    (std)    (bij)    (alg)

      -1       0        1        0
       0       1        2        1
      +1       2        3        ω
```

One stored value. Four accessor methods. The 12 conversion functions in gf3_algebra.rs collapse into `Trit::as_a()`, `as_b()`, `as_c()`, `as_d()` and corresponding `from_a()`, `from_b()`, `from_c()`, `from_d()`.

---

## 3. Build Order

```
PHASE 0a:  trit_int.rs              ← THE GATE. Blocks everything.
PHASE 0b:  constants.rs MIGRATE     ← Ternary values → TritInt
           gf3_algebra.rs REWRITE   ← u8 → Trit, conversions → views
PHASE 1:   coprime.rs               ← Canonical coprime operations (TritInt)
           sparse.rs                ← SparseGrid, GuardSentinel, ZStack
PHASE FC:  forma_codex/             ← Grid, cell, document, preset
           wasm_exports.rs adds     ← §C Shared/Coprime, §D Forma Codex
           JSX frontend             ← 7 files, thin canvas renderer
```

---

## 4. Modules — Status

### MIGRATE (above the gate after this task)

| Module | Lines | Action |
|--------|-------|--------|
| constants.rs | 1,639 | Ternary values → TritInt. Non-ternary values stay native. Tests produce identical results. |

### REWRITE (above the gate after this task)

| Module | Lines | Action |
|--------|-------|--------|
| gf3_algebra.rs | 97 (→ ~200) | 22 pub functions: u8 → Trit. 12 conversion functions removed (→ Trit views). 9 tests rewritten. Eisenstein arithmetic added. |

### DO-NOT-MODIFY (stay below the gate)

| Module | Lines | What Forma Codex imports from it |
|--------|-------|--------------------------------|
| gf3.rs | 550 | Gf3 type (face link mod-3 via below-gate interop) |
| tribonacci.rs | 690 | TernaryRepr, TritVec (below-gate interop) |
| ternary_circle.rs | 510 | Z₂₈ cyclic group, 364° geometry |
| torus.rs | 596 | TorusAddress concepts |
| repunit_circles.rs | 132 | repunit(n) → u64, R₃–R₉ |
| tlsponge385.rs | 861 | TIS-27 fast path |
| cube_addr.rs | 59 | 13-trit Rep C address pattern |

### NEW

| Module | Location | What |
|--------|----------|------|
| trit_int.rs | ternary-math/src/ | TritInt + Trit. The gate. |
| coprime.rs | ternary-math/src/ | gcd, is_coprime, euler_totient, coprime_options, coprime_walk_2d, coprime_combinations, multidim_walk |
| sparse.rs | ternary-math/src/ | SparseGrid<T>, GuardSentinel, ZStack<T> (guard + 13 layers) |
| forma_codex/mod.rs | ternary-math/src/forma_codex/ | Module root |
| forma_codex/grid.rs | ternary-math/src/forma_codex/ | FormaGrid, validate_cell_address (against grid dims), walk, cell_dimensions |
| forma_codex/cell.rs | ternary-math/src/forma_codex/ | Cell, CellLayer (13 variants), 3 faces + sentinel, push_undo, guard_intact |
| forma_codex/document.rs | ternary-math/src/forma_codex/ | FormaDocument, serialize/deserialize |
| forma_codex/preset.rs | ternary-math/src/forma_codex/ | 30 presets, GridPreset, SpectralClass |

---

## 5. Source Findings (Verified)

**repunit()** — exists twice. constants.rs:72 returns u32. repunit_circles.rs:12 returns u64. New modules import from constants.rs via TritInt.

**gcd()** — exists 10 times (3 in ternary-math, 7 in kernel). coprime.rs provides the canonical version above the gate.

**Rep B↔C** — gf3_algebra.rs lines 26–29: `rep_b_to_c(b) = b+1`, `rep_c_to_b(c) = c-1`. Collapse into Trit views.

**Rep D** — does not exist yet. Added as Trit's fourth view (as_d) with Eisenstein arithmetic.

**Cargo.toml** — no serde dependency. Must add: serde, serde_json, serde-wasm-bindgen 0.6.

**wasm_exports.rs** — 485 lines (not 486). New §C + §D append after line 485.

**lib.rs** — 55 lines, 27 pub mod. Append: `pub mod trit_int; pub mod coprime; pub mod sparse; pub mod forma_codex;`

**Kernel coprime_walk.rs** — 183 lines, 1D ring walker. coprime.rs provides 2D/k-D CRT position generators. No overlap.

---

## 6. Rust/JS Split

**If it computes, Rust. If it draws pixels, JS.**

| Operation | Where | Why |
|-----------|-------|-----|
| Grid topology, cell dimensions, scale factors | Rust (WASM) | Computation |
| Style group lookups, preset validation | Rust (WASM) | Computation |
| Z-stack management, guard verification | Rust (WASM) | Computation |
| Walk position, coprime walk sequence | Rust (WASM) | Computation |
| Text measurement (ctx.measureText) | JS | Requires canvas context |
| Text rendering (ctx.fillText) | JS with Rust-provided params | Requires canvas context |
| Grid/cell/cursor drawing | JS with Rust-provided dims | Requires canvas context |
| Morph effects, transitions | JS | Requires requestAnimationFrame |
| User input (pointer, keyboard) | JS | Requires DOM events |

JS never hardcodes a ternary value. Every constant, scale factor, grid dimension, and style parameter comes from the WASM bridge.

---

## 7. What Carries Forward

### From PlenumText v5.33 (2,112 lines)

COMPUTE → Rust: mkS (scale system), defaultGroups (10 style groups), snapToGrid, parseMD
DRAW → JS: rLines, renderBlk, drawGrid, renderShape3D, drawContourLine
MEASURE → JS with Rust params: prep, doLay (Knuth-Plass), measureBlk

Also carries: FONTS (13 fonts), TEMPLATES (5), Brand palette B, proj3D/rotVert, genPrismVerts/Faces, genSphereVerts/Faces, getShapeSilhouette, renderTextOnPath, traceContour.

### From ResonanceProofEngine (1,770 lines)

Three.js scene init/cleanup, useC hook (WASM constants with DEV fallback), Tip tooltip, Gauge component, Brand palette B.

### From sponge-wasm-bridge.ts (80 lines)

WASM bridge pattern: try import → catch → TS fallback.

### From Threepipe (reference)

Multi-select, pivot-based spanning, smart duplication, clipboard interaction model.

---

## 8. WASM Bridge

Pattern from sponge-wasm-bridge.ts. useFormaCodex.ts follows:

```typescript
let wasmModule: any = null;
let useWasm = false;
async function initWasm(): Promise<boolean> {
  try {
    wasmModule = await import('../../../../ternary-math/pkg/ternary_math');
    useWasm = true;
    return true;
  } catch { return false; }
}
```

Returns `{ mode: 'wasm'|'dev', ready: boolean, ...api }`.

WASM boundary IS the gate's outer edge. JS sends u32/number. Exports convert to TritInt/Trit at entry, convert back at exit.

---

## 9. JSX Frontend — 7 Files

| File | Location | What |
|------|----------|------|
| FormaCodex.tsx | client/src/components/forma-codex/ | Main component, canvas, RAF loop, presets, mode badge |
| useFormaCodex.ts | client/src/components/forma-codex/ | WASM bridge hook |
| canvas.ts | client/src/components/forma-codex/ | All draw ops — grid, cells, Z-stack layers, walk, cursor |
| input.ts | client/src/components/forma-codex/ | Pointer, keyboard, hidden textarea, walk controls |
| transitions.ts | client/src/components/forma-codex/ | 3D morph effects (from PlenumText proj3D/rotVert) |
| blocks.ts | client/src/components/forma-codex/ | Text engine (from PlenumText measureBlk/renderBlk/doLay with Knuth-Plass) |
| dev/fallback.ts | client/src/components/forma-codex/dev/ | TS fallback mirroring WASM exports |

---

## 10. Killed Decisions

| Killed | Correct Location |
|--------|-----------------|
| repc.rs as separate module | Trit::as_c() — one type, four views |
| repd.rs as separate module | Trit::as_d() — one type, four views |
| Frost mode (Presentation) | Overflow setting on Ice: scroll vs clip |
| Fire as name for Plasma | Plasma — states of matter metaphor |
| "trit-pixel" unit | Pixels. Just pixels. |
| Padding restricted to {3,9,13,27,39,81} | 0–32px, any value |
| u32/u64 for ternary arithmetic | TritInt above the gate |
| u8 for trit values | Trit above the gate |
| gf3_algebra.rs EXTEND | REWRITE — 22 signatures change |
| constants.rs DO-NOT-MODIFY | MIGRATE — ternary values → TritInt |
| 4 faces per cell | 3 content faces + face 0 sentinel |
| Print only active face | All faces print |
| Markdown export loses metadata | Round-trip with YAML frontmatter |
| Hardcoded grid dimensions in specs | User chooses. Engine validates coprime. |

---

## 11. Relationship to Kernel Browser

The kernel browser (12 modules, 7,267 lines) is the native counterpart:

| Kernel Browser | Forma Codex | Shared |
|----------------|-------------|--------|
| parse.rs (HTML → DOM) | parseMD / HTML import | ternary-math constants |
| layout.rs (taffy Grid/Flex) | grid.rs (coprime grid) | coprime.rs, TritInt |
| render_cpu.rs (CpuFramebuffer) | canvas.ts (HTML5 canvas) | Same math, different output |
| color.rs (PlenumColor mesh) | Brand palette B | Same values |
| tabs.rs (multi-tab) | Multi-face cells | Face cycling |

When the kernel ships, Forma Codex renders natively through CpuFramebuffer instead of HTML5 canvas. The Rust grid engine doesn't change — only the output target.

---

## 12. Tests

| File | What |
|------|------|
| coprime_tests.rs | coprime_walk_2d(7,11).len() == 77, gcd, coprime_options |
| sparse_tests.rs | GuardSentinel verify, ZStack layer access, SparseGrid CRUD |
| forma_codex/grid_tests.rs | Grid creation, validate_cell_address, walk_step |
| forma_codex/cell_tests.rs | push_undo (z=1→z=2→z=3), face cycling, guard_intact |
| forma_codex/preset_tests.rs | 30 presets, all coprime, custom_preset validation |
| gf3_algebra.rs (inline) | Trit roundtrips, Eisenstein tables, all existing tests pass |

**HARD GATE:** cargo test — zero failures before any frontend or WASM work.

---

## 13. Deliverables

| Category | Count |
|----------|-------|
| New Rust modules | 7 (trit_int, coprime, sparse, forma_codex/{mod,grid,cell,document,preset}) |
| Migrated Rust modules | 1 (constants.rs) |
| Rewritten Rust modules | 1 (gf3_algebra.rs) |
| WASM exports additions | §C + §D appended to wasm_exports.rs |
| lib.rs additions | 4 pub mod lines |
| Cargo.toml additions | 3 serde deps |
| Test files | 5 + inline gf3_algebra tests |
| JSX frontend | 7 files |

---

*Forma Codex 18∏ — Lo Sono Capomastro — Così sia.*
