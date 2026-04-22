# FORMA CODEX 18∏

## Grid Architecture Specification

**Document Viewer · Editor · Rendering Engine**

```
Copyright © 2026 Capomastro Holdings Ltd. — All Rights Reserved
Patent(s) Pending — Applied Physics Division
Lo Sono Capomastro
```

---

## 1. The Four Dimensions

| Dimension | Declared | Range | Purpose |
|-----------|----------|-------|---------|
| X | Yes | 3+ | Columns. Any integer ≥ 3. Interchangeable with Y for landscape. |
| Y | Yes | 3+ | Rows. Any integer ≥ 3. gcd(X,Y) must equal 1. |
| Z | Fixed | Always 13 | Cell state stack. z=0 guard, z=1–z=3 content undo, z=4–z=11 metadata, z=12 LAST³, z=13 GRI. |
| W | Always 3 | Fixed | Three content faces per cell (face 1, 2, 3). Face 0 is null sentinel. |

**Pages are emergent.** Content that overflows Y generates new pages automatically. The user never declares page count. Pages only emerge after the grid's uniform scaling has been exhausted — only when further scaling would push any content cell below the 81 pixel minimum.

**Indexing starts at 1.** z=0 and face 0 are null sentinels. A cell or face reading index 0 is provably corrupt. This is a security and integrity feature — documents with zeroed references are caught at the data model level before rendering.

---

## 2. Grid Sizing — Uniform Shrink to Fit

The grid occupies the full viewport. Cell size is computed, never declared:

```
cellWidth  = (viewportWidth  - totalColOffsetWidth)  / X
cellHeight = (viewportHeight - totalRowOffsetHeight) / Y
```

Both dimensions scale uniformly with the viewport. All content reflows within its cell at any size.

### Minimum Cell Dimension: 3⁴ = 81 pixels

For content cells (text, images, shapes). Structural cells (color swatches, rules, decorative spacing) allow 3³ = 27 pixels.

Why 81: it is the fourth power of the radix (3⁴). Below 81px, the Tribonacci scale system cannot produce a legible font size. 81px fits 8–12 characters at minimum readable size (10px). It is the base unit at nesting depth 2 in the cascade (729→243→81→27→9→3). And 81 = R₄ + R₃ + R₂ + R₁ = 40 + 13 + 4 + 1, encoding the full shallow repunit stack.

### When the Grid Doesn't Fit

On every viewport resize, the engine checks whether any cell dimension drops below minimum. If breached, a non-blocking amber badge appears suggesting the next viable coprime grid:

```
"7×13 needs 567×1053px minimum. Current viewport: 480×600.
 Suggested: 5×7 (fits at 96×85px per cell) [Apply] [Dismiss]"
```

- **Apply** — content from cells that no longer exist flows into remaining cells in reading order. No content is lost.
- **Dismiss** — grid stays at declared size, horizontal scroll enabled.
- If viewport is too small for any preset (minimum 3×7 at 81px = 243×567px), the engine enters **linear mode** — single-column flow, full viewport width.

The engine never auto-applies a degenerate grid. Every suggestion maintains coprime integrity (gcd(X,Y) = 1).

---

## 3. Cell Modes — Water, Ice, Plasma

Three cell modes determine how the container behaves relative to its content. The metaphor follows states of matter: content is water, the grid is the container.

| Mode | State | Container | Content | Grid Impact |
|------|-------|-----------|---------|-------------|
| Variable | Water (liquid) | Grows/shrinks with content | Flows freely | Row/column heights adjust. May create pages after 81px limit. |
| Fixed | Ice (solid) | Locked size | Overflow setting: `scroll` (scrollbars) or `clip` (silent clip, no scrollbars) | None — grid undisturbed. |
| Plasma | Plasma | Collapses to tab OR expands to overlay | Hidden when collapsed, visible when expanded | Collapsed: minimal. Expanded: overlay, no reflow. |

### 3.1 Variable Mode — Water

The default. When a variable cell's content grows taller, the entire row grows taller. Every cell in that row gets the same new height. Horizontal alignment is preserved. Every row below shifts downward. The grid first attempts uniform rescale to keep everything in the viewport. **Only after rescaling would break the 81px minimum** does the grid extend past the viewport, creating a new page.

When content shrinks, the row contracts to the tallest remaining cell. Rows below shift upward. Pages may collapse.

The grid is always tight. There is never dead space. Row height is set by the tallest variable cell.

### 3.2 Fixed Mode — Ice

The cell has a locked size. Content that exceeds the boundary is handled by the overflow setting:

- `scroll` — content scrolls internally with scrollbars (default)
- `clip` — content clips silently at the cell boundary, no scrollbars, no overflow indicator

The grid is undisturbed. Fixed cells do not contribute to row height calculation in rows with variable cells.

### 3.3 Plasma Mode

A plasma cell can collapse to a clickable tab (minimum 27×27 pixels) and expand to occupy a configurable maximum area of the viewport, revealing hidden content.

**Collapsed state:** Renders as a small tab showing only a label, icon, or color indicator. Occupies minimal grid space. Other cells flow around it as if it were a 27px structural cell.

**Expanded state:** User clicks the tab. Cell animates open using its configured morph effect. The expanded cell overlays the grid — floats above neighbors rather than pushing them. Can grow up to a configurable maximum (viewport fraction).

```javascript
plasma: {
  collapsed: true,
  tabLabel: '§7 Orbifold',
  tabIcon: null,
  tabSize: { w: 27, h: 27 },      // 27×27 minimal square; widens to fit label
  expandMax: { w: '80vw', h: '70vh' },
  expandOrigin: 'center',          // center | top-left | cursor
  expandEffect: 'scale',
  expandDuration: 350,
  dimBackground: true,
  closeOnClickOutside: true,
  persistent: false,               // if true, stays open across page transitions
}
```

**Plasma auto-collapse:** On any page transition (effect boundary triggered, keyboard page navigation, walk step that moves cursor off the cell), all expanded plasma cells automatically collapse — unless `persistent: true`. Persistent plasma cells stay open across transitions. This is for cases like a sidebar detail panel that the user intentionally wants visible while browsing.

**Plasma and walk mode:** When walk mode is active, any plasma cell the cursor lands on automatically expands (overlay), regardless of its trigger setting. When the cursor moves to the next cell, the previous plasma cell collapses. This is a global walk behavior, not per-cell. The walk step still counts as "visited" whether or not the cell expanded. During walk mode, auto-expand/collapse applies to all plasma cells regardless of `persistent`. The `persistent` flag only affects behavior outside walk mode (e.g., manual expand survives a page transition).

**Plasma use cases:**

- **Detail panels** — dashboard cells collapse to tabs, expand on click to show full breakdowns.
- **Annotations** — appendix material appears as labeled tabs, reader expands only what they need.
- **Interactive presentations** — walk cursor auto-expands plasma cells on arrival, auto-collapses on departure (spotlight effect).
- **Mobile/responsive** — dense grids collapse to tabs, user taps to expand.
- **Global toggle** — all plasma cells in a group expand/collapse simultaneously.

---

## 4. Cell Spanning

A cell can span multiple rows and columns:

```javascript
{ x: 3, y: 5, span: { cols: 3, rows: 2 } }
```

User creates spans via "Merge Cells." Splits via right-click → "Split Cell." When merged, content flows into the merged cell in reading order. When split, content stays in the top-left cell.

Spanning does not change the grid's coprime identity. In walk mode, a merged cell is ONE walk step. Consumed grid positions are skipped.

---

## 5. Padding and Spacing

Every cell has inner padding — the gap between cell border and content.

```
┌─────────────────────────────────────────┐
│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│░░┌───────────────────────────────────┐░░│
│░░│ The radian unit is exactly 13°    │░░│
│░░│ when π = 14. The full circle is   │░░│
│░░│ 364° and the circumference is     │░░│
│░░│ 28 × 13 = 364.                   │░░│
│░░└───────────────────────────────────┘░░│
│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
└─────────────────────────────────────────┘

░ = padding     Content fills all remaining space.
```

Content width = cell width − left padding − right padding.

### Global vs Per-Cell

The document has global defaults. Every cell inherits unless its z=5 layer contains an override:

```javascript
globalPadding: { top: 13, right: 13, bottom: 13, left: 13 },
globalLineSpacing: 3,
globalTextWrap: 'wrap',
globalOverflow: 'grow',
globalContentAlign: { v: 'top', h: 'left' },
```

Padding values are in pixels, range 0–32. The user sets any value via slider or numeric input.

### Content Alignment — Nine Positions

| | Left | Center | Right |
|---|---|---|---|
| **Top** | Top-Left | Top-Center | Top-Right |
| **Middle** | Middle-Left | Middle-Center | Middle-Right |
| **Bottom** | Bottom-Left | Bottom-Center | Bottom-Right |

Default: Top-Left.

---

## 6. Z-Axis — Structured State Stack

Every cell has a Z-stack of 14 layers (z=0 through z=13). The Z-axis is a complete cell state machine — content history AND metadata are versioned together. Each layer has a dedicated role. Every cell in every document has the full stack.

```
z=0    GUARD
       Tripwire. Initialized to a non-zero marker on cell creation.
       Never written to. Never read during normal operation.
       Integrity scanner checks this value — if it changed, the cell
       is corrupt. The engine surfaces a warning, restores from GRI,
       and reports the location. This is the Rep C zero-exclusion:
       z=0 is the forbidden address.

z=1    LIVE CONTENT
       What the user currently sees and edits.
       Contains: { type: 'blocks', blocks: [...] }

z=2    LAST CONTENT
       The content that was in z=1 before the most recent edit. Undo target.
       Every edit pushes z=1 → z=2, z=2 → z=3, z=3 → z=12.

z=3    LAST² CONTENT
       Two edits ago.
       When a new edit pushes z=3 out, it moves to z=12 (LAST³).

z=4    CELL STYLE
       Visual styling of the cell as a container.
       • Background color or gradient
       • Border style (visible edges, thickness, color)
       • Cell style: normal | emboss | raise
       • Emboss/raise intensity (0.1–2.0)
       • Shadow direction and intensity
       • Opacity (0.0–1.0)

z=5    CELL LAYOUT
       Padding, alignment, and overflow rules.
       • Inner padding { top, right, bottom, left } in pixels (0–32)
       • Content alignment: { v: top|middle|bottom, h: left|center|right }
       • Line spacing override (null = inherit global)
       • Text wrap: wrap | nowrap | truncate
       • Overflow: grow | scroll | clip

z=6    CELL ANIMATION
       W-axis morph behavior — when and how faces rotate.
       • Trigger: manual | scroll | hover | timer | global | walk
       • Effect: cut | dissolve | slide | flip | cube | scale | emboss | raise
       • Duration (100–2000 ms)
       • Easing: linear | ease-in | ease-out | ease-in-out | cubic-bezier(custom)
       • Direction: up | down | left | right
       • Intensity: 0.1–2.0 multiplier
       • Timer interval (ms, if trigger = timer)
       • Scroll threshold (0.0–1.0, if trigger = scroll)

z=7    CELL CONNECTIONS
       Relationships to other cells.
       • Mirror source: { x, y } — this cell copies content from the source cell.
         Source edits propagate automatically. Manual editing breaks the mirror
         (cell becomes independent with "unlinked" indicator).
       • Face link (Phase 2): { sources: [...], rule: 'add_mod3' }
         Active face computed from other cells via mod-3 arithmetic.
       • Groups: user-named collections for batch operations.
         ['revenue-section', 'summary-cards']
       • Cone point flag: true/false — marks this cell as intentionally
         breaking grid symmetry.

z=8    CELL STRUCTURE
       Nesting and spanning metadata.
       • Has nested grid: true/false
       • Nested grid dimensions (X×Y×Z of sub-grid)
       • Nested grid cells (recursive)
       • Current nesting depth (max 5: 729→243→81→27→9→3)
       • Row effect: applied to entire row
         (highlight | collapse | separator | morph-sync | freeze | hide)
       • Column effect: applied to entire column

z=9    CELL NOTES
       User annotations visible only in edit mode. Never printed.
       • Comments: [{ author, text, timestamp }, ...]
       • Tags: string labels for categorization and search
         ['needs-review', 'final', 'draft']
       • Color flag: none | red | yellow | green | blue
         Visible dot in cell corner during editing.

z=10   CELL EXPORT
       Print and PDF behavior.
       • Print behavior: include | exclude | page-break-before | page-break-after
       • PDF layer: foreground | background | hidden
       • Accessibility alt text for screen readers
       • Print style overrides: { color: '#000000' } (force black text in print)

z=11   CELL HISTORY
       Change tracking metadata only. No content.
       • Last modified timestamp
       • Edit count (times z=1 changed since GRI was set)
       • Change source: 'user' | 'mirror-sync' | 'import' | 'undo'

z=12   LAST³ CONTENT
       Third undo level. Three edits ago.
       When a new edit pushes z=12 out, it is discarded. GRI never moves.

z=13   GRI — GLOBAL REFERENTIAL INDEX
       The original content of this cell when first created or imported.
       Locked. Cannot be edited. Only reset by explicit "Set New Baseline" action.
       Reset to GRI: copy z=13 into z=1, clear z=2, z=3, z=12.
       This is the cell's birth certificate. You can always get back to it.
```

Z is always 13. Every cell in every document has the full stack. No presets, no upgrades, no variable depth. The 14 layers (z=0 through z=13) are the complete cell state machine. Every metadata change is versioned independently — undo a style change without touching content, undo an animation without touching layout. One unified system per cell.

---

## 7. W-Axis — Three Content Faces Per Cell

Every cell has three content faces. The user sees one at a time. Face 1 is default. Face 2 and 3 are null until the user adds content (right-click → "Add Face 2").

Each face has its own Z-stack (all 14 layers). Editing face 2 pushes undo states on face 2's Z-stack. Face 1's history is untouched. GRI at z=13 is per-face.

Face rotation is cyclic: 1 → 2 → 3 → 1. Each trigger event advances one step. The user can also jump directly to any face.

---

## 8. Morph Triggers — When Faces Rotate

| Trigger | Behavior |
|---------|----------|
| `manual` | User clicks a face toggle control on the cell. **Default.** |
| `scroll` | Face advances when cell crosses a configurable viewport threshold (0.0 = top, 1.0 = bottom). |
| `hover` | Face advances on cursor enter. Optionally reverses on leave. |
| `timer` | Face advances every N seconds. Per-cell configurable. Auto-cycling content. |
| `global` | Follows document-wide face toggle. All global cells rotate together. **Static elements:** logo, footer, copyright set to global with `facePinned: true` — stays on current face while others rotate. |
| `walk` | Face advances when walk cursor arrives. Only active in walk mode. |

---

## 9. Morph Effects — How Faces Rotate

Each effect has four configurable parameters: duration (100–2000ms), easing (linear, ease-in, ease-out, ease-in-out, cubic-bezier), direction (up, down, left, right), intensity (0.1–2.0).

| Effect | Description | Configurable |
|--------|-------------|-------------|
| `cut` | Instant switch. No animation. | — |
| `dissolve` | Crossfade between faces. | Duration, easing |
| `slide` | Old slides out, new slides in from opposite edge. | Duration, easing, direction |
| `flip` | 3D card flip around an axis. | Duration, easing, direction (axis), intensity (angle) |
| `cube` | Cell rotates like a cube revealing next face. | Duration, easing, direction, intensity (perspective) |
| `scale` | Old shrinks to center, new grows from center. | Duration, easing, intensity (min scale) |
| `emboss` | Transition with deep sunken shadow effect. | Duration, easing, intensity (depth), shadow color |
| `raise` | Transition with raised/beveled chamfered edges. | Duration, easing, intensity (height), edge radius |

### Persistent Cell Styles

Independent of morph effects, a cell can have a persistent visual style:

| Style | Appearance |
|-------|-----------|
| `normal` | Flat cell, no depth effect. Default. |
| `emboss` | Surface appears sunken. Content pressed into panel. Shadow falls inward. |
| `raise` | Surface appears elevated. Chamfered edges, shadow falls outward. |

A cell can be permanently embossed AND use a cube morph. Style and morph are independent.

---

## 10. Offsets — Full-Width Rows and Full-Height Columns

Structural elements inserted outside the base grid. They consume viewport space without changing the coprime identity.

### Row Offset Types

| Type | Purpose | Default Style |
|------|---------|---------------|
| `title-header` | Document title (H1). Before row 1. Title, subtitle, author, date. | Raise |
| `title-footer` | Document closing. After last row. Copyright, doc ID, classification. | Emboss |
| `section-header` | Section divider (H2). Number and title. Blue rule + double bars. | Raise |
| `section-footer` | Section closer. Mirrored rule, right-aligned double bars. | Emboss |
| `sticky-header` | Locked to viewport top. Document title, section breadcrumb. | Raise |
| `sticky-footer` | Locked to viewport bottom. Page number, status, copyright. | Emboss |
| `banner` | Full-width image, gradient, or decorative element. | Normal |
| `page-break` | Forces content onto next page. Invisible in print. | Normal |
| `effect-boundary` | Triggers page transition when scrolled past. Invisible. | Normal |

### Column Offset Types

| Type | Purpose |
|------|---------|
| `sidebar` | Persistent navigation, TOC, info panel. Sticky during scroll. |
| `margin-notes` | Annotation column. Smaller text, links to adjacent cells. |
| `rule` | Vertical divider between sections. Thin line with optional label. |
| `gutter` | Empty spacing column. Maintains grid geometry. |
| `image` | Full-height image or video column. |

Column offset data model, default styles, and behavior (sticky, resize, interaction with cell sizing) deferred to implementation phase. Will match row offset detail level when built.

### Layout Example — 7×13 Grid with Offsets

```
[=============== Sticky Header: "PlenumNET — UV Spectral Protocol" ===============]
[=============== Title Header: TM-2026-026 v1.2 — March 2026 ====================]

                 Col Offset
                 (sidebar)
    c1  c2  c3  c4 [S] c5  c6  c7
r1  [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
r2  [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
r3  [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
--- [======= Section Header: §2 The Four System Wavelengths =======]
r4  [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
r5  [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
...
r13 [ ] [ ] [ ] [ ] [S] [ ] [ ] [ ]
=== [=================== Section Footer: End §2 ===================]

[=============== Sticky Footer: "Page 1 of 3 — © Capomastro" ====================]
```

If adding an offset pushes cell size below 81px minimum, the engine warns with options to narrow the offset, reorganize the grid, or add anyway.

### Offset Data Model

```javascript
// Row Offset
{
  type: 'row-offset',
  position: 'after-row-6',
  offsetType: 'section-header',
  content: { number: '§2', title: 'The Four System Wavelengths' },
  height: 'auto',
  sticky: false,
  effectBoundary: true,
  transitionEffect: 'dissolve',
  transitionDuration: 400,
  style: 'raise',
}

// Sticky Footer
{
  type: 'row-offset',
  position: 'viewport-bottom',
  offsetType: 'sticky-footer',
  content: {
    left: 'TM-2026-026 v1.2',
    center: 'Page {n} of {total}',
    right: '© Capomastro Holdings Ltd.',
  },
  height: 40,
  sticky: true,
  style: 'emboss',
}
```

---

## 11. Page Navigation

Five navigation methods.

### 11.1 Scroll (Default)

Smooth pixel scrolling. Behavior depends on the document-level `scrollMode` setting:

**Paged mode** (`scrollMode: 'paged'`, default): When a Row Offset with `effectBoundary: true` enters the viewport, scroll pauses, the page transition plays, then scrolling resumes. Transitions are triggered by content structure, not fixed page sizes.

**Continuous mode** (`scrollMode: 'continuous'`): Single tall canvas, uninterrupted vertical flow. No viewport clipping, no page transitions. Effect boundaries are ignored during scroll. Row Offsets render inline as static elements. This is the v5.33 behavior — useful for tall grids where the user wants to see the full document as one continuous surface. Cell morphs (scroll trigger, hover trigger) still function in continuous mode.

### 11.2 Keyboard

| Key | Action |
|-----|--------|
| Page Down / Space | Advance one viewport height. Boundaries trigger transitions. |
| Page Up | Reverse one viewport height. |
| Arrow Down | Advance one grid row. |
| Arrow Up | Reverse one grid row. |
| Home | Jump to row 1. |
| End | Jump to last row with content. |

In **continuous mode**, Page Down/Up scrolls one viewport height without triggering any transition effects. Effect boundaries are ignored during all keyboard navigation in continuous mode.

### 11.3 Walk Mode

Toolbar toggle (shortcut: W). Walk cursor appears — highlighted cell with blue glow.

| Mode | Order | Use Case |
|------|-------|----------|
| **Reading** | Left to right, top to bottom, row by row | Review, proofreading, accessibility |
| **Coprime** | Diagonal Hamiltonian cycle from coprime structure | Presentation reveal, progressive disclosure |

**Reading mode** visits cells sequentially. Intuitive, predictable.

**Coprime mode** follows the Hamiltonian cycle. In 7×13, the walk moves diagonally: (1,1), (2,2), (3,3)..., wrapping at edges. Because gcd(7,13)=1, all 91 cells are visited exactly once. Each step shows a cell in a different grid region.

Walk controls:

- `→` or `1`: Step forward — `(+1 col, +1 row) mod (X, Y)`
- `←` or `2`: Step backward (two steps forward = one step back, since 2 = -1 mod 3)
- `0` or `.`: Stay on current cell (identity step — face does NOT advance)

These correspond to the Rep B alphabet {0, 1, 2}. Walk step numbers display as small indicators in each cell corner.

**Walk in continuous mode:** The walk is confined to the currently active nested grid. The user selects which root cell (and its nested grid) is active. When the walk completes a full cycle within that nested grid, it advances to the next root cell in reading order and enters its nested grid. The walk traverses all root cells sequentially, completing each nested grid before moving on.

### 11.4 Click Navigation

A sidebar Column Offset lists sections (TOC). Clicking jumps to the section's Row Offset. Only the final boundary's transition plays.

### 11.5 Timer / Presentation Mode

Auto-advances one page every N seconds. Boundaries trigger transitions. Self-running presentation.

---

## 12. Page Transition Effects

When a transition triggers: engine captures outgoing viewport as bitmap, captures incoming viewport as bitmap, runs transition compositor, resumes normal rendering.

### Phase 1 Effects (2D Canvas)

| Effect | Description |
|--------|-------------|
| `none` | Continuous scroll. No transition. Default. |
| `slide` | Outgoing slides away, incoming slides in from opposite edge. |
| `dissolve` | Crossfade between pages. |
| `cube` | Pages textured onto cube faces. Cube rotates 90°. Uses existing proj3D/rotVert. |
| `flip` | Page hinges along edge, flips like a book page. |
| `zoom` | Outgoing shrinks to vanishing point, incoming grows from same point. |

### Phase 2 Effects (WebGL)

| Effect | Description |
|--------|-------------|
| `water` | Outgoing ripples with sine displacement, clears to reveal incoming. |
| `torus` | Outgoing wraps onto torus, rotates to reveal incoming. Reuses Three.js. |

---

## 13. Three Independent Systems

**Scroll** moves the viewport vertically. **Page transitions** play at effect boundaries and affect the entire viewport. **Cell morphs** play when individual cells meet their trigger and affect one cell. All three operate simultaneously:

```
1. Smooth scroll through rows 1–6
   → Cell (3,2) has trigger:'scroll', effect:'flip'
   → Flips face 1 → face 2 as it crosses threshold
   → All other cells static

2. Section Header after row 6: effectBoundary, transition:'cube'
   → Scroll pauses → viewport cube-rotates → scroll resumes

3. Smooth scroll through rows 7–13
   → Cell (5,9) has trigger:'hover', effect:'dissolve'
   → User hovers → dissolves face 1 → face 2

4. Content overflows past row 13
   → New page (same grid geometry)
   → Effect boundary at overflow → transition plays
```

---

## 14. Row and Column Effects

Stored in z=8 of any cell in the target row/column:

| Type | Scope | Purpose |
|------|-------|---------|
| `highlight` | Row or Col | Tinted background across all cells |
| `collapse` | Row or Col | Collapses to thin line, expandable on click |
| `separator` | Row or Col | Visual divider rule |
| `morph-sync` | Row or Col | All cells morph faces together as a unit |
| `freeze` | Row or Col | Stays visible during scroll (freeze panes) |
| `hide` | Row or Col | Present in data, not rendered |

---

## 15. Cell Mirrors and Groups

### Mirrors

A cell mirrors another cell's content. Right-click → "Mirror Cell" → select source. Edits to source propagate. Manual editing breaks the mirror ("unlinked" indicator appears). This is how repeating elements work — one source, mirrors at each position.

### Groups

User-named collections. Batch operations: collapse all in group, morph-sync group, highlight group, select group.

### Face Links (Phase 2)

Active face computed from source cells: `activeFace = ((sum of source faces) mod 3) + 1`. Cycle prevention: a cell cannot be both source and target in the same chain.

---

## 16. Cone Points and Focus Mode

A cone point is any cell that intentionally breaks grid symmetry:

- A `facePinned` cell — cone point on W-axis
- A broken mirror — cone point on spatial symmetry
- A cell with per-cell overrides differing from globals — cone point on style uniformity

**Focus Mode** (toolbar toggle) dims all non-cone-point cells and highlights cone points. Shows: "here is the truly unique content — everything else is pattern." Cone point density |C| / total cells displayed in status bar.

---

## 17. Cell Nesting

A cell can contain a sub-grid instead of blocks. Sub-grid has its own X×Y×Z, its own cells, its own content. Depth limited to 5 levels (base unit cascade 729→243→81→27→9→3). Minimum cell size applies at each level.

---

## 18. Complete Cell Data Model

```javascript
{
  // Position
  x: 3, y: 5,
  span: { cols: 1, rows: 1 },
  mode: 'variable',              // variable | fixed | plasma
  role: 'content',               // content | structural

  // Plasma (only when mode = 'plasma')
  plasma: {
    collapsed: true,
    tabLabel: '§7 Orbifold',
    tabIcon: null,
    tabSize: { w: 27, h: 27 },
    expandMax: { w: '80vw', h: '70vh' },
    expandOrigin: 'center',
    expandEffect: 'scale',
    expandDuration: 350,
    dimBackground: true,
    closeOnClickOutside: true,
    persistent: false,
  },

  // Content Faces — W-axis
  // Each face contains its own Z-stack
  faces: [
    null,                          // face 0: sentinel
    {                              // face 1: default
      zStack: [
        null,                      // z=0: GUARD (tripwire — corruption detection)
        { type: 'blocks', blocks: [  // z=1: LIVE CONTENT
            { type: 'h2', text: 'The Four System Wavelengths' },
            { type: 'body', text: 'Every system wavelength is a multiple of 13.' },
        ]},
        { ... },                   // z=2: LAST CONTENT
        { ... },                   // z=3: LAST² CONTENT
        {                          // z=4: CELL STYLE
          background: null,
          borderStyle: null,
          cellStyle: 'normal',
          cellStyleIntensity: 0.8,
          shadowDirection: null,
          opacity: 1.0,
        },
        {                          // z=5: CELL LAYOUT
          padding: null,           // null = inherit global
          contentAlign: { v: 'top', h: 'left' },
          lineSpacing: null,
          textWrap: null,
          overflow: null,
        },
        {                          // z=6: CELL ANIMATION
          trigger: 'manual',
          effect: 'dissolve',
          duration: 400,
          easing: 'ease-out',
          direction: 'right',
          intensity: 1.0,
          timerInterval: null,
          scrollThreshold: null,
        },
        {                          // z=7: CELL CONNECTIONS
          mirrorSource: null,
          faceLink: null,
          groups: [],
          conePoint: false,
        },
        {                          // z=8: CELL STRUCTURE
          hasNestedGrid: false,
          nestedGrid: null,
          depth: 1,
          rowEffect: null,
          colEffect: null,
        },
        {                          // z=9: CELL NOTES
          comments: [],
          tags: [],                // flat strings for documents, key-value for logs
          colorFlag: 'none',
        },
        {                          // z=10: CELL EXPORT
          printBehavior: 'include',
          pdfLayer: 'foreground',
          accessibilityAlt: null,
          printStyleOverrides: null,
        },
        {                          // z=11: CELL HISTORY
          lastModified: null,
          editCount: 0,
          changeSource: 'user',
        },
        { ... },                   // z=12: LAST³ CONTENT
        {                          // z=13: GRI
          type: 'blocks',
          blocks: [...],
          locked: true,
        },
      ],
    },
    null,                          // face 2: empty until user adds
    null,                          // face 3: empty until user adds
  ],
  activeFace: 1,
  facePinned: false,
}
```

---

## 19. Document Globals

```javascript
{
  grid: {
    x: 7,
    y: 13,
    z: 13,                           // fixed — always 13
    orientation: 'portrait',         // portrait | landscape
  },

  scrollMode: 'paged',              // paged | continuous
  // paged: viewport shows one page at a time, effect boundaries trigger transitions
  // continuous: single tall canvas, smooth scroll, no page transitions —
  //   the v5.33 behavior. For tall grids where the user wants uninterrupted
  //   vertical flow without viewport clipping or transition effects.

  topology: {                      // computed, not declared
    gcd: 1,
    cycle: 91,
    hamiltonian: true,
    identity: 'λ_EUV',
  },

  defaults: {
    padding: { top: 13, right: 13, bottom: 13, left: 13 },
    lineSpacing: 3,
    textWrap: 'wrap',
    overflow: 'grow',
    contentAlign: { v: 'top', h: 'left' },
    cellStyle: 'normal',
    morphTrigger: 'manual',
    morphEffect: 'dissolve',
    morphDuration: 400,
  },

  globalFace: 1,                   // current document-wide active face

  walk: {
    active: false,
    mode: 'reading',               // reading | coprime
    currentStep: 0,
    currentCell: { x: 1, y: 1 },
  },

  defaultTransition: {
    effect: 'dissolve',
    duration: 400,
    easing: 'ease-out',
  },

  presentation: {
    active: false,
    timerInterval: 5000,
  },

  notesPanel: {
    visible: false,
    filter: 'all',
    sortBy: 'position',
  },

  rowOffsets: [...],
  colOffsets: [...],
  cells: [...],
}
```

---

## 20. Grid Presets

All presets define X×Y. Z=13 always. W=3 always. X and Y interchangeable for portrait/landscape. All grids coprime.

In **paged mode** (`scrollMode: 'paged'`), the preset X×Y defines the grid per page.

In **continuous mode** (`scrollMode: 'continuous'`), the root grid is X×1 — a single row of X cells, each auto-growing vertically with content. Coprime grids live as nested grids inside these cells. The walk operates on the nested grids. The root X×1 is just the scroll container.

Custom X and Y values beyond the preset list are allowed via the Custom dialog. The user selects one axis first (e.g., X=3). The dialog then computes and displays all valid coprime values for the other axis as a selectable list — the user picks from the list, never enters a raw number. No knowledge of gcd required. The engine warns if X×Y exceeds 1000 cells.

### 20.1 Grid Presets

| Preset | X×Y×Z | Cells | Cycle | Paper Match | Use |
|--------|-------|-------|-------|-------------|-----|
| Micro | 3×4×13 | 12 | 12 | Business card 2×3.5 | Tiny cards, labels, badges |
| ★ Index Card | 5×7×13 | 35 | 35 | Index 3×5 | Flash cards, small cards |
| Postcard | 3×7×13 | 21 | 21 | Postcard 4×6 | Invitations, postcards |
| Brochure | 3×11×13 | 33 | 33 | A6 | Small brochure |
| Digest | 5×7×13 | 35 | 35 | Digest 5.5×8.5 | Booklets |
| Pamphlet | 3×13×13 | 39 | 39 | DL 99×210mm | Tri-fold pamphlet |
| ★ Document | 3×17×13 | 51 | 51 | — | Standard 3-column document |
| Report | 5×11×13 | 55 | 55 | B5 176×250mm | Reports, proposals |
| Ledger | 7×9×13 | 63 | 63 | Statement 5.5×8.5 | Financial statements |
| Brief | 5×13×13 | 65 | 65 | Executive 7.25×10.5 | Executive briefs |
| ★ Tri-Column | 3×22×13 | 66 | 66 | Letter 8.5×11 | Tri-column layout, margin+body+margin |
| ★ Compact | 7×11×13 | 77 | 77 | A5 148×210mm | Quick notes, memos |
| ★ Standard | 7×13×13 | 91 | 91 | Letter 8.5×11 | Default working document |
| Folio | 9×11×13 | 99 | 99 | A4 210×297mm | European standard |
| Technical | 7×15×13 | 105 | 105 | Half-Tabloid | Technical documentation |
| Manuscript | 9×13×13 | 117 | 117 | Quarto 8×10 | Academic manuscripts |
| Broadsheet | 9×14×13 | 126 | 126 | Foolscap 8×13 | Long-form editorial |
| ★ Reference | 11×13×13 | 143 | 143 | Legal 8.5×14 | Dense reference material |
| ★ Widescreen | 13×14×13 | 182 | 182 | — | Screen-first documents |
| Architectural | 7×26×13 | 182 | 182 | ANSI B 11×17 | Engineering, architecture |
| Editorial | 9×28×13 | 252 | 252 | Broadsheet 15×22 | Newspaper layout |
| Scroll | 7×37×13 | 259 | 259 | — | Long-form continuous scroll |
| Long Report | 9×29×13 | 261 | 261 | — | Multi-section reports |
| Manuscript Long | 9×31×13 | 279 | 279 | — | Thesis, dissertation |
| Deep Scroll | 7×41×13 | 287 | 287 | — | Extended continuous content |
| Long Reference | 11×29×13 | 319 | 319 | — | Deep reference documents |
| ★ Poster | 13×28×13 | 364 | 364 | A1 594×841mm | Large format, posters |
| Mural | 11×43×13 | 473 | 473 | — | Wall-sized layouts |
| Canvas | 13×37×13 | 481 | 481 | — | Large working canvas |
| Full Canvas | 13×41×13 | 533 | 533 | — | Maximum working area |
| Maximum | 27×28×13 | 756 | 756 | — | Largest available grid |

### 20.2 UV Spectral Templates

| Preset | X×Y | Cells | Cycle | UV Identity | System Ref |
|--------|-----|-------|-------|-------------|------------|
| Deep UV | 7×11 | 77 | 77 | Sub-ionization EUV | [UV-Sub] |
| Ionization | 7×13 | 91 | 91 | 91nm threshold | [UV-EUV] |
| Control Point | 11×13 | 143 | 143 | αβ angle | [UV-Control] |
| Absorption | 7×26 | 182 | 182 | 182nm O₂ wall | [UV-UVC] |
| Absorption Alt | 13×14 | 182 | 182 | R₃×π | [UV-UVC-alt] |
| Ozone Bridge | 11×26 | 286 | 286 | 286nm bridge | [UV-UVB] |
| Therapeutic | 11×28 | 308 | 308 | 308nm excimer | [UV-Therapeutic] |
| Full Spectrum | 13×28 | 364 | 364 | R₆ transmission | [UV-UVA] |

### 20.3 Z-Stack Reference

Z is always 13. Every cell in every preset has the full 14-layer stack.

| Z | Layer | Contents |
|---|-------|----------|
| z=0 | GUARD | Tripwire — corruption detection. Never written, never read in normal operation. |
| z=1 | LIVE CONTENT | What the user currently sees and edits. |
| z=2 | LAST CONTENT | Previous edit. Undo target. |
| z=3 | LAST² CONTENT | Two edits ago. Pushes to z=12 on overflow. |
| z=4 | CELL STYLE | Background, borders, emboss/raise, shadow, opacity. |
| z=5 | CELL LAYOUT | Padding, alignment, line spacing, wrap, overflow. |
| z=6 | CELL ANIMATION | Trigger, effect, duration, easing, direction, intensity. |
| z=7 | CELL CONNECTIONS | Mirrors, groups, face links, cone point flag. |
| z=8 | CELL STRUCTURE | Nested grid config, row/column effects. |
| z=9 | CELL NOTES | Comments, tags, color flags. Edit-only, never printed. |
| z=10 | CELL EXPORT | Print behavior, PDF layer, accessibility alt text. |
| z=11 | CELL HISTORY | Last modified, edit count, change source. |
| z=12 | LAST³ CONTENT | Third undo level. Three edits ago. Discarded on overflow. |
| z=13 | GRI | Global Referential Index. Locked baseline. Immutable until explicit reset. |

---

## 21. Print and Export

### Print

Page bitmap at pixel resolution is the print master. Scale = targetDPI / screenDPI × (printPageDimensions / viewportDimensions). Integer pixels produce exact rational coordinates. All faces print — face 1 primary, faces 2 and 3 as supplementary pages or appendix sections (user-configurable per document). Effect boundaries become page breaks.

### Export to PDF

Same rendering as print. z=10 export metadata controls per-cell behavior.

### Export to Markdown

Content from all faces across all pages in reading order. Grid structure preserved in YAML frontmatter. Cell metadata (style, notes, tags) preserved as HTML comments inline. Reimport reconstructs the grid.

```markdown
---
forma_codex: 7×13×13
preset: Standard
faces: 3
cells: 91
---
# Document Title
<!-- cell: 1,1 face:1 style:raise tags:final -->
...
<!-- cell: 1,1 face:2 -->
...
```

Round-trip: export to Markdown → reimport → grid structure, faces, notes, and tags restored. Style overrides and animation settings preserved in frontmatter or inline comments. Image references preserved as standard Markdown image links.

### Import from DOCX

mammoth.js → HTML → turndown → Markdown → parseMD() → blocks → grid cells. Options: preserve formatting or text only.

### Import from PDF

pdf.js text extraction → heuristic heading/paragraph detection → Markdown → blocks → grid cells.

---

## 22. Performance

- **Viewport culling:** Only visible cells + 1 row margin rendered.
- **Bitmap capture:** Capped at 2× DPR for transitions.
- **Z-stack structural sharing:** Undo states store diffs. Images never duplicated — only metadata versioned.
- **Plasma overlay:** Expanded plasma cells render in separate canvas layer.

---

## 23. Linear Mode

When viewport is too small for any grid: cells render as single column, full width, stacked in reading order. Coprime walk disabled (reading-order only). Cell morphs and page transitions still function. Plasma cells remain functional. Grid topology preserved in data model.

---

## 24. What Carries Forward from PlenumText v5.33

| v5.33 Component | Status |
|-----------------|--------|
| measureBlk / renderBlk | Survives. Receives cell content width. |
| doLay / rLines / prep | Line-breaking engine upgraded: greedy wrap replaced by Knuth-Plass optimal paragraph justification. Minimizes total badness across all lines simultaneously — critical for narrow cells in 3-column layouts (e.g., 3×17). Scoped to cell. |
| renderShape3D / image system | Survives. Constrained to cell bounds. |
| Style groups / templates | Survives. Per-cell or per-block. |
| Grid Mode (3×3 panel) | Promoted to core architecture. |
| Block drag-drop | Becomes drag between cells. |
| drawGrid | Replaced by cell boundary renderer. |
| proj3D / rotVert | Reused for cube transition and flip morph. |
| Brand palette B | Identical. Direct reuse. |
| parseMD | Survives as import parser. |
| Flat blks[] array | Replaced by recursive cell tree. |
| Single-canvas tall scroll | Retained as `scrollMode: 'continuous'`. Default is `'paged'` (viewport + transitions). |
| blockY[] | Replaced by cell layout engine. |
| cWidth | Replaced by computed cell dimensions. |

### What Carries Forward from the Resonance Proof Engine

| Engine Component | Status |
|------------------|--------|
| Three.js initScene pattern | Reused for Phase 2 torus page transition. Camera controls (mkCtrl), render loop, cleanup. |
| Clifford torus projection (cV, stereographic) | Reused for torus page transition effect and torus walk visualization overlay. |
| updGeo vertex coloring | Reused for highlighting walk positions on torus visualization — emission band technique maps to cell highlighting. |
| Settings persistence (window.storage) | Reused for document settings save/load with debounce. |
| Tooltip system (Tip component) | Reused for coprime advisor badges, cell info popups, walk step tooltips. |
| Gauge components (SVG arc) | Reused for status bar indicators — cone point density, walk position, cell count. |
| SVG emboss filter definition | Reused for emboss/raise persistent cell styles. |
| Brand palette B | Identical across all engines. Direct reuse. |
| PhasorVis (canvas animation pattern) | Pattern reused for cell morph effect rendering — offscreen canvas capture and compositor. |

---

## 25. Native Log Viewer

Forma Codex includes a native log viewer as a built-in feature. Logs are not imported manually — they stream in, auto-populate the grid, and organize spatially.

### 25.1 Log Sources

All PlenumNET services write logs to `C:\PlenumNET\Logs\` with subdirectories per service category (networking, auth, storage, compute, monitoring, boot, daemon). The viewer watches this root path recursively — subdirectory name IS the source category. No per-source configuration needed. New services appear automatically.

Additional source types:

- **WebSocket:** Live stream from running services for real-time push.
- **Import:** Load external or historical log files not in the common directory.

### 25.2 Grid Layout

Logs populate a coprime grid automatically. The user chooses the layout:

**Service columns (default):** Each column is a service or function category (e.g., networking, authentication, storage, compute, monitoring). Each row is a time slice. Cross-service correlation visible at a glance — related events from different categories sit in the same row.

**Severity rows:** Rows grouped by severity (ERROR, WARN, INFO, DEBUG, TRACE). Columns are time windows.

**Custom:** User picks X axis (service category, subsystem, host, custom dimension) and Y axis (time, severity, event type, custom dimension). Engine validates coprime and offers valid grid dimensions.

Auto-population: as log entries arrive, the engine places them in the correct cell based on their classification dimensions. Cells grow (Water mode) as entries accumulate. When a time row fills, the grid scrolls or pages.

### 25.3 Three Faces Per Log Cell

| Face | Content | Audience |
|------|---------|----------|
| 1 | Human-readable log message | Operator glancing at status |
| 2 | Raw structured data (JSON, key-value pairs) | Engineer debugging |
| 3 | Correlation context — what triggered this entry, what it triggered, foreign key references to related entries across services and categories | System tracing causality |

Face 1 is always populated. Faces 2 and 3 populate automatically if the log source provides structured data and correlation IDs.

### 25.4 Filtering Controls

The log viewer toolbar provides:

- **Date/time range:** Start and end datetime pickers. Presets: Last 5 min, 15 min, 1 hour, 24 hours, 7 days, custom.
- **Severity filter:** Toggle ERROR, WARN, INFO, DEBUG, TRACE independently.
- **Service filter:** Toggle individual services on/off.
- **Text search:** Full-text search across all visible log content. Highlights matching cells.
- **Tag filter:** Filter by structured classification dimensions (subsystem, event type, host, custom tags).

Filters apply instantly. The grid repopulates showing only matching entries. Cell count and time range displayed in status bar.

### 25.5 Live Mode

Toggle live mode on: new log entries stream in and append to the grid in real time. The grid auto-scrolls to show the latest entries. Coprime walk mode paused during live streaming (walk resumes from current position when live mode is toggled off).

Toggle live mode off: grid freezes at current state for inspection. Scroll and walk freely through captured entries.

### 25.6 Log Cell Classification (z=9 Tags)

Log cells use structured tags instead of flat strings:

```javascript
// Document cell tags (flat strings)
tags: ['needs-review', 'final', 'draft']

// Log cell tags (key-value dimensions)
tags: {
  source: 'auth-service',
  severity: 'ERROR',
  subsystem: 'session',
  event: 'connection_lost',
  host: 'node-1',
  trace_id: 'abc123'
}
```

Both formats supported. The engine detects which format a cell uses. Document cells use flat strings. Log cells use key-value dimensions. Query by constraining any subset of dimensions.

### 25.7 Boot Attestation View

Boot sequence logs produce a Forma Codex document automatically. Each measurement — BIOS hash, kernel hash, module load, driver init — is a cell with:

- Face 1: Stage name and status (OK / FAILED)
- Face 2: Hash value, timing, memory state
- Face 3: Measurement chain — previous stage hash, cumulative attestation

Walk mode traverses the boot sequence in pipeline order. Any cell's integrity is verifiable by re-deriving from its content.

### 25.8 What This Replaces

No existing log aggregation is implemented (no ELK, no Splunk, no Loki). The log viewer is the first and only implementation — built as a native feature of the document engine rather than a bolted-on third-party stack.

---

## 26. Build Order

### Phase 1A — Grid Engine
1. Core grid layout (cell dimensions, borders, uniform shrink-to-fit)
2. 81px minimum enforcement with coprime grid suggestion
3. Port measureBlk/renderBlk into cell content area (Knuth-Plass line-breaking replaces greedy wrap)
4. Cell selection and markdown editing (multi-select, pivot-based spanning, smart duplication of cell groups via mirrors, clipboard for cell content)
5. Variable/Fixed modes with overflow setting (scroll vs clip) and row-height propagation
6. Z-stack (all 14 layers, z=0 through z=13)
7. Viewport culling

### Phase 1B — Faces, Effects, and Connections
8. W-axis (3 faces) with manual trigger and dissolve
9. Cell mirrors and groups (batch operations, morph-sync)
10. Scroll trigger with viewport threshold
11. Global face toggle with facePinned
12. Emboss and Raise persistent styles

### Phase 1C — Structure
13. Row Offsets (title header, section header, sticky footer)
14. Column Offsets (sidebar, margin notes)
15. Page transitions (none, dissolve, slide, cube)
16. Cell spanning (merge/split)

### Phase 1D — Plasma
17. Collapsed tab rendering
18. Expand/collapse animation with overlay canvas
19. Walk mode integration (auto-expand on cursor)
20. Global toggle integration (batch expand/collapse)

### Phase 1E — Navigation
21. Walk mode — reading order
22. Walk mode — coprime order with step numbers
23. Keyboard navigation
24. Presentation mode (timer)

### Phase 1F — Polish
25. Cell notes and global notes panel
26. Cone point detection and Focus Mode
27. Structural vs content cell role enforcement
28. Print/PDF export
29. Markdown and DOCX import

---

## 27. Rep C Document Integrity

The Forma Codex 18∏ document format uses Rep C addressing — the orbifold quotient applied to document structure. In the Buried Question, Rep C = {1, 2, 3} is the step alphabet with the identity (zero) removed. The same principle governs every address in the format.

### The Zero Exclusion Principle

```
Valid Z addresses:    {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13}
Valid Face addresses: {1, 2, 3}
Valid Cell addresses: {(1,1) through (X,Y)}
Zero does not exist in any address space.
```

This is not a convention — it is the mathematical consequence of operating in the quotient space where the identity element has been removed. Rep C has no zero. The document format inherits this property.

### Three Detection Layers

**Layer 1 — Address Validation (Rep C)**

Every reference is checked against the valid range. Z addresses must be 1..Z. Face addresses must be 1..3. Cell coordinates must be 1..X and 1..Y. A zero anywhere triggers immediate rejection.

```
Conventional format:
  Read cell reference → get 0 → is this "cell zero" or corruption?
  → Ambiguous. Parser must guess. Attack surface.

Rep C format:
  Read cell reference → get 0 → IMPOSSIBLE in valid document.
  → Provably corrupt. Reject immediately.
```

This catches null injection, uninitialized memory reads, and buffer overflow artifacts — the most common vectors for document-borne exploits.

**Layer 2 — Sentinel Verification**

z=0 and face 0 are not merely "invalid" — they are actively monitored. The sentinel slots are initialized to a known non-zero marker pattern. If the sentinel's value changes from the marker, something wrote to a forbidden address. This catches memory corruption that might overwrite adjacent data without producing a zero — a subtler attack than null injection.

**Layer 3 — Rep C Structural Integrity (Native Ternary)**

In a natively ternary document (running on the PlenumNET kernel), the format uses trit-strings. Rep C trit-strings contain only {1, 2, 3} (mapped from the algebraic {1, ω}). A zero trit in a Rep C stream means the quotient was violated:

| Zero Location | Diagnosis |
|---------------|-----------|
| Cell address | External tampering or parser exploit |
| Z-stack index | Memory corruption or buffer overflow |
| Content trit stream | Transmission corruption or format downgrade |
| Sentinel slot | Adjacent memory overwrite (hardware fault or sophisticated attack) |

All four are detectable. All four are distinguishable by where the zero appears.

### Native Ternary Path

**Phase 1 (JavaScript/browser):** Software enforcement. The engine validates all indices on read and write. Zero triggers an error state.

**Phase 2 (WASM bridge):** The Rust WASM bridge validates at the boundary. Trit-strings entering the document engine from external sources are checked for Rep C compliance before acceptance.

**Phase 3 (Native ternary):** Hardware enforcement. The PlenumNET kernel's memory model uses Rep C addressing. Zero trits in address space trigger a hardware fault. Document corruption is physically impossible within the native execution environment.

### Why This Is Not Arbitrary

The zero-exclusion is the orbifold quotient applied to document addressing. Rep C removes the identity from the step space. The document format removes the identity from the address space. Same operation, different domain. The security benefit is a direct consequence of the mathematical structure: the quotient space has no identity element, therefore there is no null pointer by construction.

In conventional formats, null is a valid state that must be checked at runtime. In Rep C, null is a structural impossibility that is enforced by the encoding. The difference is between "we check for null" and "null cannot be expressed."

---

### The ∏ Symbol

18∏ — the ∏ is Unicode U+220F, the N-ARY PRODUCT operator. It reads as "18 product" which decodes to 18 × (the product that defines the framework). In the Salvi Framework: 182 = 14 × 13 = π × radian = the half-turn. The symbol encodes the axiom in the name itself. The product operator is exactly correct — 182 IS a product. General users see a mathematical accent mark. Framework-aware readers decode it instantly.

---

*Forma Codex 18∏ — Lo Sono Capomastro — Così sia.*
