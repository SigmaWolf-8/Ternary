# FORMA CODEX 18∏ — Implementation Architecture

```
Copyright © 2026 Capomastro Holdings Ltd. — All Rights Reserved
Patent(s) Pending — Applied Physics Division
```

---

## 1. Core Principle — Shared Math, Specific Application

The Buried Question defines five compression mechanisms. Forma Codex and the compression engine both consume the same mathematical primitives. Those primitives live in SHARED modules at the repo root — not inside either application.

```
SHARED (used by compression engine + Forma Codex + future applications)
├── constants.rs          ← EXISTING — 80+ master constants
├── ternary_math.rs       ← EXISTING — gcd, pow3, mod3, is_coprime
├── repunit.rs            ← EXISTING — repunit(n), repunit_factors
├── tis27.rs              ← EXISTING — hash primitive
├── coprime.rs            ← NEW — coprime walk generator, topology computation
├── repc.rs               ← NEW — Rep C encoding, zero-exclusion validation
└── sparse.rs             ← NEW — sparse serialization (null-skip encoding)

FORMA CODEX (document engine — uses shared modules)
├── forma_codex/grid.rs       ← viewport layout, cell dimensions, shrink-to-fit
├── forma_codex/cell.rs       ← Cell struct, ZStack (14 layers), Face system
├── forma_codex/document.rs   ← Document struct, globals, offsets
├── forma_codex/preset.rs     ← 31 grid presets, custom dialog options
└── forma_codex/mod.rs        ← module declarations

Compression Engine (uses SAME shared modules)
├── (existing compression)    ← uses coprime.rs for stride patterns
│                               uses repc.rs for quotient encoding
│                               uses sparse.rs for output format
```

---

## 2. What the Compression Engine Gains From This Work

The three NEW shared modules benefit the compression engine directly:

| Shared Module | Compression Engine Use | Forma Codex Use |
|---------------|---------|-----------------|
| coprime.rs | Stride-delta encoding patterns. CoprimePair validates Z27×Z28. HamiltonianWalk generates stride sequences for container decomposition. | Grid validation. Walk navigation. Custom dialog coprime options. |
| repc.rs | Quotient encoding — orbifold quotient removes identity from step space. validate_address() checks trit-stream integrity. GUARD_MARKER validates container headers. | Document integrity — zero-exclusion on all addresses. Guard check on z=0. Integrity scanner. |
| sparse.rs | Compressed output — only non-zero residuals stored. sparse_ratio() reports compression efficiency. | Document save — only populated Z-layers and faces stored. sparse_ratio() reports save size. |

When coprime.rs exists, compression modules reference it it instead of maintaining their own coprime validation. When repc.rs exists, the quotient step references shared repc instead of inline logic. No duplication.

---

## 3. System Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  RUST (SigmaWolf-8/Ternary)                                        │
│                                                                     │
│  SHARED MODULES (repo root — used by all applications)     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ constants.rs     REPUNIT_3=13, ROOT_X1=14, QUAD_PRODUCT=364 │   │
│  │ ternary_math.rs  gcd(), is_coprime(), pow3(), mod3()         │   │
│  │ repunit.rs       repunit(n), repunit_factors()               │   │
│  │ tis27.rs         TIS-27 hash primitive                       │   │
│  │ coprime.rs       CoprimePair, HamiltonianWalk, topology()    │   │
│  │ repc.rs          RepCAddress, validate(), zero_exclusion()   │   │
│  │ sparse.rs        sparse_encode(), sparse_decode()            │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                             │                                       │
│          ┌──────────────────┼──────────────────┐                    │
│          │                  │                  │                    │
│  ┌───────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐            │
│  │ forma_codex/  │  │ (compression)         │  │ (future)     │            │
│  │ grid.rs       │  │ Uses same:   │  │ Uses same:   │            │
│  │ cell.rs       │  │ coprime.rs   │  │ coprime.rs   │            │
│  │ document.rs   │  │ repc.rs      │  │ repc.rs      │            │
│  │ preset.rs     │  │ sparse.rs    │  │ sparse.rs    │            │
│  └───────┬───────┘  └──────────────┘  └──────────────┘            │
│          │                                                         │
│  ┌───────┴──────────────────────────────┐                          │
│  │ wasm_bridge.rs                       │                          │
│  │   Existing 80+ constant getters      │                          │
│  │ + Shared coprime/repc/sparse exports │                          │
│  │ + Forma Codex fc_ exports            │                          │
│  └───────┬──────────────────────────────┘                          │
└──────────┼─────────────────────────────────────────────────────────┘
           │  WASM boundary
┌──────────┼─────────────────────────────────────────────────────────┐
│  JSX (thin — draws what Rust tells it)                             │
│  ┌───────┴──────────┐                                              │
│  │ useFormaCodex.js │  WASM when built, DEV fallback when not      │
│  └───────┬──────────┘                                              │
│  ┌───────┴──────────────────────────────────────────────────────┐  │
│  │ FormaCodex.jsx  — canvas setup, state, render loop           │  │
│  │ canvas.js       — all draw operations (cells, UI, overlays)  │  │
│  │ input.js        — pointer + keyboard + hidden textarea       │  │
│  │ transitions.js  — page transition compositor (bitmap ops)    │  │
│  │ blocks.js       — port of measureBlk/renderBlk from v5.33   │  │
│  │ dev/fallback.js — mirrors Rust API in JS for prototyping     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  Rule: if it computes, it is in Rust. If it draws pixels, it is   │
│  in JSX. No exceptions.                                            │
└────────────────────────────────────────────────────────────────────┘
```

---

## 4. Shared Modules — Exact Definitions

### 4.1 coprime.rs (NEW shared)

Used by compression engine for stride patterns. Used by Forma Codex for grid navigation.

- `CoprimePair` — validated struct, gcd(a,b)=1 guaranteed by construction
- `CoprimePair::topology()` → TorusTopology { gcd, cycle, hamiltonian }
- `HamiltonianWalk::new(pair)` — generates full walk order, step (+1,+1) mod (a,b)
- `HamiltonianWalk::forward()` / `backward()` / `stay()` — Rep B {1,2,0} step keys
- `HamiltonianWalk::reading_order(pair)` — row-major order for same bounds
- `HamiltonianWalk::step_at(col, row)` → Option<u32> step number at position
- `coprime_options(axis, min, max)` → Vec<u32> valid partners for custom dialog / stride selection

### 4.2 repc.rs (NEW shared)

Used by compression engine for quotient encoding. Used by Forma Codex for document integrity.

- `validate_address(coords: &[u32])` — any zero → RepCViolation
- `validate_cell_address(x, y, z, face)` — shorthand for 4-coordinate check
- `validate_bounds(x, y, max_x, max_y)` — range check after zero check
- `GUARD_MARKER: u64` — 0x0D0D0D0D0D0D0D0D (REPUNIT_3 repeated)
- `guard_intact(value)` → bool

### 4.3 sparse.rs (NEW shared)

Used by compression engine for compressed output. Used by Forma Codex for document save.

- `SparseEntry { index: u32, data: Vec<u8> }` — one non-null slot
- `sparse_encode(items, serialize_fn)` → Vec<SparseEntry> — skips None entries
- `sparse_decode(entries, total_size, deserialize_fn)` → Vec<Option<T>> — reconstructs full array
- `sparse_ratio(total, populated)` → f64 — compression/save efficiency metric

---

## 5. Forma Codex Modules — Application-Specific

### 5.1 forma_codex/grid.rs

Uses shared `coprime::CoprimePair` for the underlying pair. Adds viewport-specific logic.

- `Grid` struct — contains CoprimePair + z(=13) + w(=3) + orientation + scroll_mode
- `Grid::new(x, y)` — validates x,y ≥ 3, delegates coprime check to shared module
- `Grid::cell_dimensions(vp_w, vp_h, col_off, row_off)` → CellDimensions
- `Grid::walk()` → HamiltonianWalk — delegates to shared module
- `MIN_CONTENT_CELL = 81` (3^4), `MIN_STRUCTURAL_CELL = 27` (3^3), `PERF_WARNING_THRESHOLD = 1000`

### 5.2 forma_codex/cell.rs

Uses shared `repc::GUARD_MARKER` and `repc::guard_intact()`.

- `ZLayer` enum — 11 variants covering all 14 slots (Content used for z=1,2,3,12)
- `ZStack` — fixed [Option<ZLayer>; 14], initialized with Guard at z=0
- `ZStack::push_undo()` — z=1→z=2→z=3→z=12, z=12 overflow discarded
- `ZStack::undo()` — reverse: z=2→z=1, z=3→z=2, z=12→z=3
- `ZStack::reset_to_gri()` — z=13→z=1, clear undo slots
- `ZStack::guard_check()` — delegates to shared repc::guard_intact()
- `Cell` struct — position, span, mode, role, plasma, 4 faces (0=null,1-3=content), active_face
- `Cell::advance_face()` — cyclic 1→2→3→1, respects face_pinned
- `Cell::integrity_check()` — checks guard on all non-null faces
- All Z-layer data types: CellStyleData, CellLayoutData, CellAnimData, CellConnData, CellStructData, CellNotesData, CellExportData, CellHistData, GriData, ContentData, Block enum

### 5.3 forma_codex/document.rs

Uses shared `sparse.rs` for save/load.

- `Document` struct — Grid, defaults, global_face, walk state, offsets, cells
- `Document::save()` — sparse_encode each cell's Z-stacks, omit null faces
- `Document::load(data)` — sparse_decode, reconstruct full 14-slot stacks
- `Document::save_ratio()` — delegates to sparse_ratio() for UI display
- `DocumentDefaults`, `RowOffset`, `ColOffset` — all offset types from spec §10

### 5.4 forma_codex/preset.rs

Uses shared `coprime::coprime_options()`.

- `PRESETS: &[GridPreset]` — all 31 presets from spec §20.1
- `custom_options(axis, min, max)` — delegates to shared coprime_options()

---

## 6. WASM Bridge

Additions to existing wasm_bridge.rs. Two categories:

**Shared exports** (usable by any frontend, including future compression UI):
- `coprime_validate(a, b)` → bool
- `coprime_options(axis, min, max)` → Vec<u32>
- `coprime_walk(a, b)` → JsValue (walk order array)
- `repc_validate(x, y, z, face)` → bool

**Forma Codex exports:**
- `fc_grid_new(x, y)` → JsValue
- `fc_cell_dimensions(x, y, vp_w, vp_h, col_off, row_off)` → JsValue
- `fc_presets()` → JsValue
- `fc_min_content_cell()` → u32
- `fc_min_structural_cell()` → u32
- `fc_perf_warning_threshold()` → u32

---

## 7. JSX Renderer — 7 Files

| File | Responsibility | Source |
|------|---------------|--------|
| FormaCodex.jsx | Canvas setup, ResizeObserver, state, render loop | New |
| useFormaCodex.js | WASM/DEV hook — same API both modes (Resonance Engine useC pattern) | New, pattern from existing |
| canvas.js | ALL draw operations: cells, borders, content, emboss, offsets, walk cursor, plasma tabs/overlays, toolbar, context menus, tooltips. 18-layer draw order, single canvas. | New |
| input.js | ALL input: hidden textarea (keyboard capture), pointer events (click, drag, right-click), keyboard nav (arrows, Page Up/Down, walk keys 0/1/2), clipboard | New |
| transitions.js | Page transition compositor: capture viewport bitmap, blend with dissolve/slide/cube/flip/zoom. Ports proj3D/rotVert from v5.33. | Port from existing |
| blocks.js | Block rendering inside cells: ports measureBlk, renderBlk, doLay, rLines, prep from PlenumText v5.33. Scoped to cell content bounds. | Port from existing |
| dev/fallback.js | DEV fallback: mirrors entire Rust API in JS. gcd(), is_coprime(), walk generation, presets, cell dimensions, repc validation. Constants match master constants.rs. | New |

**Rule: if it computes, it is in Rust. If it draws pixels, it is in JSX.**

---

## 8. What Carries Forward

| Source | Component | Destination |
|--------|-----------|-------------|
| PlenumText v5.33 | measureBlk, renderBlk, doLay, rLines, prep | blocks.js |
| PlenumText v5.33 | proj3D, rotVert | transitions.js |
| PlenumText v5.33 | parseMD | blocks.js (import parser) |
| PlenumText v5.33 | renderShape3D, image system | canvas.js (constrained to cell bounds) |
| PlenumText v5.33 | Brand palette B | dev/fallback.js (constants) |
| Resonance Engine | useC hook pattern | useFormaCodex.js (WASM/DEV dual mode) |
| Resonance Engine | Three.js initScene, Clifford torus | transitions.js (Phase 2 torus effect) |
| Resonance Engine | Tooltip (Tip), Gauge components | canvas.js (canvas-rendered versions) |
| Resonance Engine | SVG emboss filter | canvas.js (canvas shadow operations) |
| Resonance Engine | Settings persistence (window.storage) | FormaCodex.jsx |

---

## 9. File Placement in Repo

```
SigmaWolf-8/Ternary/
├── src/
│   ├── constants.rs              ← EXISTING (untouched)
│   ├── ternary_math.rs           ← EXISTING (untouched)
│   ├── repunit.rs                ← EXISTING (untouched)
│   ├── tis27.rs                  ← EXISTING (untouched)
│   ├── coprime.rs                ← NEW SHARED
│   ├── repc.rs                   ← NEW SHARED
│   ├── sparse.rs                 ← NEW SHARED
│   ├── wasm_bridge.rs            ← EXISTING (add shared + fc_ exports)
│   ├── forma_codex/
│   │   ├── mod.rs                ← NEW
│   │   ├── grid.rs               ← NEW (uses coprime.rs)
│   │   ├── cell.rs               ← NEW (uses repc.rs)
│   │   ├── document.rs           ← NEW (uses sparse.rs)
│   │   └── preset.rs             ← NEW (uses coprime.rs)
│   └── (existing compression modules)
│       └── ...                   ← EXISTING (will reference coprime, repc, sparse)
├── tests/
│   ├── coprime_tests.rs          ← NEW SHARED
│   ├── repc_tests.rs             ← NEW SHARED
│   ├── sparse_tests.rs           ← NEW SHARED
│   └── forma_codex/
│       ├── grid_tests.rs         ← NEW
│       ├── cell_tests.rs         ← NEW
│       └── preset_tests.rs       ← NEW
└── frontend/
    └── forma-codex/
        ├── FormaCodex.jsx        ← NEW
        ├── useFormaCodex.js      ← NEW
        ├── canvas.js             ← NEW
        ├── input.js              ← NEW
        ├── transitions.js        ← NEW (ports from v5.33)
        ├── blocks.js             ← NEW (ports from v5.33)
        └── dev/
            └── fallback.js       ← NEW
```

---

## 10. Deliverable Summary

| Deliverable | Count | Format | Where |
|-------------|-------|--------|-------|
| Shared Rust modules | 3 | .rs + tests | src/ (repo root) |
| Forma Codex Rust modules | 4 + mod.rs | .rs + tests | src/forma_codex/ |
| WASM bridge additions | 1 (additions to existing) | .rs | src/wasm_bridge.rs |
| JSX frontend | 7 files | .jsx + .js | frontend/forma-codex/ |
| Working canvas artifact | 1 | .jsx | Viewable here in Claude |

**What you get:** A working Rust engine (10 modules, compiles, tests pass) with a WASM bridge, and a thin JSX frontend that renders on canvas using that engine. The Rust modules drop into SigmaWolf-8/Ternary. The JSX deploys on Replit. The DEV fallback lets you test the artifact here before WASM is built. The 3 shared modules immediately benefit the compression engine with zero additional work.

---

*Forma Codex 18∏ — Lo Sono Capomastro — Così sia.*
