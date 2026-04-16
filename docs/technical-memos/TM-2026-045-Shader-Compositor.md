# TM-2026-045 — Shader Compositor

## Programmable Visual Processing for the PlenumNET Platform

**Capomastro Holdings Ltd. — Applied Physics Division**
**Patent(s) Pending — All Rights Reserved**
**Version 1.5 — April 2026**

**π = 14 · radian = 13 · full circle = 364°**

-----

## Revision History

|Version|Date|Changes|
|-------|----|-------|
|1.0|April 2026|Initial draft. Over-referenced external documents. Mapped cascade to algebraic invariance without justification.|
|1.1|April 2026|Overcorrected. Stripped framework connections, removed PlenumNET uniforms, eliminated 3D use cases.|
|1.2|April 2026|Restored full vision. Introduced "PlenumDoc" as a separate concept (it isn't — Forma Codex is the editor, viewer, and format). Hardcoded example grid dimensions as if they were architecture.|
|1.3|April 2026|Fixed the three corrections but unnecessarily cut content from v1.2 and trivialized the `@plenum` encoding system.|
|1.4|April 2026|Full v1.2 content restored. Three targeted corrections applied: (1) Forma Codex is one product — PlenumText evolved — editor, viewer, and format. (2) Grid dimensions are runtime-derived, never hardcoded. (3) `@plenum` encoding properly described as the embedded indexing, primary/foreign key, and searchability system, not merely a save format.|
|**1.5**|**April 2026**|**Applied review findings (cube-rotation geometry fixed to body-diagonal only, `u_tier` renumbered to 1–4 per Rep C, output terminology moved to TSL-level "shader color output node" with GLSL/WGSL footnote, `cellStyle` vs `shaderEffect` interaction clarified as independent compositing layer with compatibility matrix, kernel-path 3D line marked speculative, §11.1 performance table re-anchored to reference device classes, Phase 1 configuration logic moved to a new `plenum-compositor` crate, new §9A accessibility section, new §9B custom-shader security/validation section, minor corrections on attribution, disdyakis normalization, `drawElement()` polyfill dependency, and `updGeo` reference) and framework-native upgrades (new §5A Sin₃₆₄/Cos₃₆₄ period-28 trigonometry primitives with `u_phase28`/`u_phaseRadian`/`u_phaseFullCircle`; new §5B horn density and gradient uniforms `u_hornDensity`/`u_hornGradient`/`u_hornVolume`; new §5C wave propagation constant c² = π = 14; new §5D calendar time quanta `u_dayOfYear`/`u_moon`/`u_dayOfMoon`/`u_isDOT`/`u_milankovitch`; new §5E 3D horn coordinate system `u_hornX`/`u_hornTheta`/`u_hornW`; PDTF §4.2 cross-referenced to TM-2026-017 §20.15 bridge coefficient κ; §8 preset rows documented with Period / Wave speed / Native-trig columns; three new manifold-aware presets — Horn Density, Coprime Phase, Brieskorn Surface; Tier Heat color derivation via torus-simplex + κ; §13 Framework-Native Status paragraph. Open Problem #4 (Framework Trigonometry monograph) is called out as the prerequisite for full Sin₃₆₄/Cos₃₆₄ shader integration, with interim standard-trig scaling specified.**|

-----

## Glossary

|Term|Definition|
|----|----------|
|**Compositor**|The stage in a rendering pipeline that combines painted content into the final image sent to the display. This document specifies a programmable version of that stage.|
|**Fragment shader**|A small program that runs once per pixel on the GPU (Graphics Processing Unit). It reads pixel data in and writes modified pixel data out.|
|**Forma Codex**|The PlenumNET document engine. Evolved from PlenumText v5.33. It is simultaneously the editor, the viewer, and the file format. Documents are enhanced markdown with `@plenum` encoding (§3.4) providing embedded indexing, primary/foreign keys, and full searchability — encrypted at rest.|
|**TSL**|Three Shading Language. A node-graph format from the Three.js library that describes shader logic as connected blocks rather than raw code. Compiles to GLSL (for WebGL) or WGSL (for WebGPU).|
|**GLSL**|OpenGL Shading Language. The standard programming language for fragment shaders on WebGL-capable GPUs.|
|**WGSL**|WebGPU Shading Language. The successor to GLSL for the WebGPU API, supported in modern browsers.|
|**WebGPU**|The modern browser API for GPU access, replacing WebGL.|
|**`drawElement()`**|A browser API that captures a live HTML element as a GPU texture — a pixel image the GPU can read. The captured HTML remains interactive. Availability today requires the Bansal polyfill (§2.3) on every non-experimental browser build.|
|**HTMLTexture**|A Three.js class that uses `drawElement()` to render HTML as a texture on any 3D surface. Buttons still click. Inputs still type. The HTML is live on the 3D geometry. Attribution and merge date (three.js PR #31233, April 10 2026) are stated as of v1.5 publication — downstream integrators should re-verify against the current three.js changelog before pinning a version.|
|**PDTF**|Plenum Display Transfer Function. The tone-mapping curve that converts linear color values to display-ready values. Uses a Bézier curve with control points at (0, 182, 650) — the roots of the framework's arc equation.|
|**WASM**|WebAssembly. A binary format that lets Rust code run in web browsers at near-native speed.|
|**Z-stack**|The 14-layer state structure (z=0 through z=13) attached to every cell in Forma Codex. Each layer has a dedicated role — z=1 is live content, z=4 is visual style, z=13 is the immutable baseline (GRI).|
|**W-axis**|The face dimension of a Forma Codex cell. Every cell has three content faces (face 1, 2, 3) viewed one at a time. Face rotation is cyclic: 1→2→3→1. Face 0 is a null sentinel.|
|**MRT**|Multiple Render Targets. A GPU technique where one shader pass writes to two or more output textures simultaneously — for example, a color image and a hit-testing map.|
|**Pick buffer**|A hidden image where each pixel stores the ID of the element at that position, used for fast mouse/touch hit-testing.|
|**AGS**|Active Generator Set. The dynamic set of coprime numbers that governs address space capacity in PlenumNET. It grows when data grows and shrinks when data shrinks. All capacity is runtime-derived — no fixed sizes.|
|**Coprime walk**|A path through a grid or torus where the step sizes share no common factor (gcd = 1), guaranteeing every position is visited exactly once (a Hamiltonian cycle). The grid dimensions are user-selected with coprime validation; the engine derives the walk.|
|**Circumsphere**|The sphere on which all vertices of an inscribed solid lie. In the 364° Crystal, all 13 antiprisms share one circumsphere.|
|**Antiprism**|A 3D solid with two parallel polygon faces (top and bottom) connected by a belt of triangles. Each of the 13 polygons in the 364° construction produces one antiprism.|
|**Torus-simplex**|The color model from TM-2026-030 where colors are addressed by walk index on a torus, with simplex weights resolving to display primaries via W = k × 1001.|
|**`@plenum` encoding**|The embedded metadata system in Forma Codex files. Each content block carries its 54-trit TDNS address as an HTML comment. This encoding provides: primary keys (identity trits = content fingerprint), foreign keys (explicit reference trits linking blocks), classification indexes (27 ontological dimensions for instant structural queries), integrity verification (re-derive address from content, compare), version tracking (identity trits change when content changes, classification prefix persists), network routing (the address IS the route on the 13D hypercube), and full searchability (constrain any subset of classification trits to query). The `@plenum` tags are invisible to standard markdown renderers — full backward compatibility.|

-----

## §1. What This Document Specifies

This document specifies how PlenumNET renders interactive HTML content through programmable GPU shaders — including on 3D surfaces.

**Plain English:** After the browser draws all the text, images, and UI elements, a GPU program can modify every pixel before it reaches the screen. This program can add glow effects, scanline overlays, color shifts, or interactive visual responses to touch and cursor movement. The HTML underneath remains fully interactive — buttons still work, text is still selectable, inputs still accept typing.

More importantly: that same HTML can be rendered as a texture on 3D geometry. A cube with live, interactive web content on each face. A document's blocks mapped onto a sphere. A cell's three content faces displayed as literal faces of a rotating 3D solid. The content is not a screenshot — it is live HTML, interactive through the 3D surface.

This capability has three parts:

1. **Shader compositing** — a fragment shader processes the rendered content pixels before display
2. **3D HTML surfaces** — live HTML rendered as interactive textures on Three.js geometry
3. **Framework integration** — shader parameters driven by PlenumNET's manifold properties (walk position, spectral tier, manifold depth)

-----

## §2. The Four Enabling Technologies

Four independent developments, all surfacing in the same week of April 2026, make this specification practical:

### 2.1 Zsolt Kacso — HTML in Canvas + Fragment Shader

Kacso (@kaolti) demonstrated a technique where actual interactive HTML is rendered into a canvas element, and a fragment shader is applied to the rendered pixels. The shader modifies how the content looks — adding chromatic aberration, glow, distortion — while the HTML underneath remains live. Inputs work. Focus works. Hit-testing works. The shader operates on the visual output without breaking the interactive layer.

**What this proved:** Per-pixel shader effects on live, interactive web content are achievable today. Not as a concept — as a shipped demo with a public link.

### 2.2 Ricardo Cabello (mrdoob) — HTMLTexture in Three.js (PR #31233)

The creator of Three.js merged `HTMLTexture` into the library — a class that renders HTML elements as textures on 3D surfaces using the `drawElement()` Canvas API. His subsequent tweet confirmed interaction support: the textured HTML stays clickable through the 3D surface.

**What this enables:** HTML content on any Three.js geometry. A plane, a cube, a sphere, a torus, an antiprism. The content is not a flat image — it is live DOM rendered as a GPU texture, interactive through the 3D surface. This is the bridge between PlenumNET's 2D document editing and 3D geometric visualization.

### 2.3 Palash Bansal (@repalash) — Cross-Browser Polyfill

Bansal published a polyfill that makes html-in-canvas work in all browsers including Safari, without experimental flags. His assessment: "works better than in canary (for now)." He invited developers to send examples for testing.

**What this eliminates:** The "only works in Chrome Canary with a flag" barrier. The polyfill means PlenumNET can ship shader compositing to every user on every browser today.

### 2.4 Dan Greenheck — WebGPU Three.js TSL Skill

Greenheck published `webgpu-claude-skill` — a skill file for AI-assisted shader authoring using TSL (Three Shading Language) targeting WebGPU. The repository includes a `skills/webgpu-threejs-tsl` folder with model-ready patterns for TSL node material construction.

**What this provides:** A proven AI-assisted workflow for creating TSL shaders. This maps directly to PlenumNET's skill file architecture and means custom shader effects can be authored by describing the desired visual result in natural language.

-----

## §3. The 3D HTML Cube — Primary Use Case

### 3.1 What It Is

Every Forma Codex cell has three content faces (the W-axis). Today, face transitions are simulated with 2D canvas operations — the cube morph effect captures a bitmap of the outgoing face, captures the incoming face, and draws a perspective-projected cube rotation between them using PlenumText's `proj3D` and `rotVert` functions.

With HTMLTexture, the cube becomes real. Three live HTML surfaces — one per face — textured onto an actual Three.js `BoxGeometry`. The cube rotates in GPU-accelerated 3D. Each face is interactive: you can click a button on face 2 while looking at the cube from an angle. The fragment shader processes the composited result — the cube's rendered appearance passes through the shader stage before reaching the screen.

**Plain English:** Instead of faking a 3D cube rotation with 2D tricks, each face of the cube shows actual live content — text you can edit, buttons you can click, inputs you can type into — all rendered on a real 3D object that rotates smoothly on the GPU. A visual effect (glow, scanlines, holographic sweep) can be applied on top.

### 3.2 How It Connects to Forma Codex

Forma Codex's cell data model already has everything this needs:

- **Three faces (W-axis):** Face 1, 2, 3 map to three faces of the cube. Face 0 is null (the hidden back faces of the cube, or transparent).
- **Face rotation is cyclic:** 1→2→3→1. This is a 120° rotation of the cube around its body-diagonal axis — the single canonical choice. A 120° body-diagonal rotation has order 3 and cycles exactly three faces, matching the three-face W-axis. A 90° face-normal rotation would have order 4 and would expose a fourth face that does not exist in the data model, so it is excluded by construction.
- **Each face has its own Z-stack:** Editing face 2 doesn't affect face 1's undo history. The shader at z=4 can differ per face — face 1 with a clean style, face 2 with a scanline effect, face 3 with a holographic sweep.
- **Morph trigger and effect** at z=6 determine WHEN the cube rotates (manual click, scroll threshold, hover, walk cursor arrival) and HOW it animates (rotation speed, easing, direction).

The cube morph effect in Forma Codex §9 specifies: "Cell rotates like a cube revealing next face. Configurable: duration, easing, direction, intensity (perspective)." HTMLTexture makes this specification literal rather than simulated.

### 3.3 Beyond Cubes — HTML on Any Geometry

HTMLTexture is not limited to cubes. Any Three.js geometry can receive an HTML texture:

|Geometry|PlenumNET Use Case|
|--------|-----------------|
|**BoxGeometry** (cube)|Forma Codex cell face rotation — three interactive content faces|
|**PlaneGeometry** (flat)|Standard 2D compositor — shader applied to flat content|
|**SphereGeometry**|364° Crystal circumsphere — document blocks mapped to sphere positions by their ℤ[φ] coordinates|
|**Custom antiprism meshes**|364° Crystal — each antiprism face can display the content block at that geometric position|
|**TorusGeometry**|Coprime walk visualization — content blocks wrapped around a torus, walk path visible as a curve|
|**DisdyakisTriacontahedronGeometry** (custom)|120 scalene triangle faces — each face renders a content block. Orbit coloring (red 5-fold, green 3-fold, blue 2-fold) applied as per-face shader tint. The circumsphere relation R² = 14 + 5φ assumes the short-edge normalization in which the shortest edge of the scalene triangle has unit length; alternative normalizations (e.g., midsphere radius = 1) rescale R by a constant factor. See TM-2026-034 for the canonical edge-length normalization used across the framework; this document adopts the same convention.|

**Plain English:** The same technology that puts live HTML on a cube face can put it on a sphere, a torus, or any 3D shape. A document could be displayed as a globe where each region shows a section. A data dashboard could be wrapped around a crystal. The content stays interactive on every surface.

### 3.4 The `@plenum` Encoding and the 3D Cube

The three faces of a cube serialize into Forma Codex's enhanced markdown with `@plenum` tags. All three faces share classification trits (they're about the same topic) but have distinct identity trits (different content on each face):

````markdown
<!--@plenum:cell:face:1:c:CLASSIFICATION_TRITS:i:IDENTITY_TRITS_FACE1-->
# Section Title

Prose explanation for face 1...

<!--@plenum:cell:face:2:c:CLASSIFICATION_TRITS:i:IDENTITY_TRITS_FACE2-->
```rust
fn implementation_for_face_2() {
    // Code view of the same concept
}
```

<!--@plenum:cell:face:3:c:CLASSIFICATION_TRITS:i:IDENTITY_TRITS_FACE3:r:REF_TO_RELATED_BLOCK-->
| Parameter | Value  | Notes               |
|-----------|--------|---------------------|
| Data      | Table  | View for face 3     |
````

The `@plenum` encoding is not merely a save format. It is the embedded indexing and searchability system:

**Primary key:** The identity trits (`i:`) are the content fingerprint. Each face has its own identity trits because each face has different content. Re-derive the identity trits from the content via TIS-27 sponge (191 ns) — if they don't match, the content has been tampered with.

**Foreign key:** The reference field (`r:`) links to related blocks anywhere on the manifold — other documents, other cells, code files, conversation chunks. These are explicit cross-references, stored as identity trit addresses. Forward lookup (what does this block reference?) and reverse lookup (what references this block?) are both TPT prefix searches.

**Classification index:** The classification trits (`c:`) are 27 ontological dimensions — WHO, WHAT, WHERE, WHEN, WHY, HOW, PEACE. They are a 27-dimensional structural index embedded directly in the document. Querying "find all technical code blocks from 2025" is constraining a handful of classification trits — no search engine, no full-text index, no external database. The document IS the index.

**Searchability:** Any subset of the 27 classification trits can be constrained in a query. Each additional constraint narrows results. Going from "everything by this entity" to "the exact function" costs 27 integer comparisons — about 15 nanoseconds. The `@plenum` tags make every block in every Forma Codex document instantly queryable by structural properties.

**Version tracking:** Edit a block → identity trits change (content fingerprint updated) → classification trits may or may not change (the topic might stay the same). The chain of identity trit mutations at the same classification prefix IS the version history. No Git required. The math IS the version control.

**Network routing:** The 54-trit address IS the network route. A block is routable from any node on the 13D hypercube by Hamming distance trit-flip walk. The document carries its own routing information.

**Backward compatibility:** The `@plenum` tags are HTML comments. Standard markdown renderers ignore them completely. Open a Forma Codex document in any text editor or markdown viewer — it renders normally. The encoding is purely additive.

The cube's physical structure (three faces sharing edges) maps to the manifold's topology (three blocks sharing a classification prefix, co-located in the TPT). A query for the topic described by those shared classification trits finds all three faces at Hamming distance 0 from each other. The geometry IS the index relationship.

**Runtime-derived, not hardcoded:** Cell coordinates, grid dimensions, walk positions, and AGS capacity are NOT stored in the `@plenum` tags. The file stores content, classification trits, identity trits, and references. The engine derives everything else at runtime from the current grid configuration and AGS state. If the document moves to a region with a different AGS, the addresses stay the same — only the walk positions are recomputed (one modulo per block per new generator, via CRT nesting).

### 3.5 Interaction Through 3D Surfaces

mrdoob's HTMLTexture handles interaction by raycasting: when you click on a 3D surface, Three.js computes which point on the texture you hit, converts that to DOM coordinates, and forwards the event to the HTML element at that position. This means:

- Click a button on the side face of a rotated cube → the button receives the click
- Type into an input on a sphere-mapped document → keystrokes reach the input
- Hover over a link on a torus-wrapped walk → the link highlights

The Rust browser kernel will implement this natively: since it controls both the 3D compositor and the DOM event pipeline, raycasting and event forwarding happen in the same process with no bridge overhead.

-----

## §4. Pipeline Position

### 4.1 Where the Shader Sits

```
Content (Forma Codex cells, blocks, UI elements)
  → Layout (compute positions and sizes)
    → Paint (draw text, images, borders to a buffer)
      → drawElement() captures painted content as GPU texture
        → [Optional] Map texture onto 3D geometry (HTMLTexture)
          → ★ Fragment Shader (modify pixels on the GPU)  ← THIS SPEC
            → PDTF tone curve (Bézier control points 0, 182, 650)
              → Screen
```

### 4.2 Why Before PDTF

The PDTF (Plenum Display Transfer Function) uses a Bézier curve whose control points are the roots of the arc equation: arc² − 832·arc + 118,300 = 0, giving roots 182 and 650. This curve maps linear light values to the non-linear response of a display.

The shader must operate BEFORE this curve is applied. If the shader ran after PDTF, all its color math — blending, lighting, mixing — would operate on already-curved values. Colors would blend incorrectly. Brightness would be wrong. Linear-space operations require linear-space input.

**Plain English:** The PDTF is the last color adjustment before the image hits the screen. The shader needs to work on the "raw" image before that adjustment. Adjusting brightness after correction looks washed out. Adjusting it before looks right.

**Derivation status:** The pipeline position follows from the mathematical properties of the PDTF curve. The Bézier control points (0, 182, 650) are exact — 182 and 650 are roots of the arc equation, verified by Vieta's formulas (182 + 650 = 832, 182 × 650 = 118,300). Operating the shader before this curve is mathematically necessary for correct color math, not an aesthetic preference.

**Composition with the bridge coefficient κ.** The PDTF curve owns tone mapping. The shader owns pixel-level effects. They compose; they do not overlap. Any color-space correction the shader applies (white-point compensation, contrast adjustment, spectral tint) must be consistent with the bridge coefficient κ = 1 + 26,147,000/137,036² (TM-2026-017 §20.15) and the +0.139% spectral offset characterized in TM-2026-026. The shader is NOT a second independent gamma function. If a preset needs a color-space correction, it consumes κ as a precomputed constant and applies it linearly — it does not re-derive tone mapping. This guarantees the PDTF → shader composition remains a single correction chain rather than two stacked nonlinear curves.

-----

## §5. What the Shader Reads and Writes

### 5.1 Standard Inputs

Every shader receives these values automatically:

|Uniform|Type|What It Is|
|-------|----|----------|
|`u_content`|Texture|The painted HTML content as a GPU-readable image. If mapped onto 3D geometry, this is the texture on the geometry's surface.|
|`u_resolution`|vec2|Width and height of the content area in pixels.|
|`u_pointer`|vec2|Current touch or mouse position, relative to the content area. On 3D surfaces, this is the UV coordinate from the raycast hit point.|
|`u_active`|float|1.0 when pointer is engaged, 0.0 when not. Fades smoothly over 200ms on release.|
|`u_time`|float|Seconds elapsed since compositor start. Drives animations: pulsing, sweeping, cycling.|

### 5.2 PlenumNET-Specific Inputs

These uniforms connect the shader to PlenumNET's manifold properties. They allow effects that respond not just to pointer position and time, but to the content's mathematical identity on the manifold:

|Uniform|Type|What It Is|Source|
|-------|----|----------|------|
|`u_walkPos`|float (0.0–1.0)|The content's coprime walk position, normalized to the current AGS capacity. Blocks at walk position 0 are at the "start" of the Hamiltonian cycle; blocks at 1.0 are at the "end." This lets the shader vary its effect along the walk path — a luminance gradient that makes the walk order visible.|Computed from the block's 54-trit address projected onto the current AGS. The AGS is runtime-derived; the normalized position adapts automatically as the AGS grows or shrinks.|
|`u_tier`|int (1–4)|The content's UV spectral tier: 1=EUV (permanent, 91 nm), 2=VUV (archival, 182 nm), 3=BC (operational, 286 nm), 4=A (ephemeral, 364 nm). This lets the shader tint content by its persistence tier — cool tones for permanent records, warm tones for ephemeral data.|Assigned by the eigenvalue-driven tier migration system. Tier boundaries at 91/182/286/364 nm derive from the axiom; the eigenvalue dynamics of the transfer matrix (eigenvalues 182 and 650) govern migration between tiers.|
|`u_depth`|float (0.0–1.0)|The element's visual depth from the Forma Codex Z-stack z=4 style layer. 0.0 = flat (normal), 0.5 = embossed, 1.0 = fully raised. This lets the shader compute depth-dependent effects: deeper shadow on embossed cells, brighter highlights on raised cells.|Read directly from z=4 `cellStyleIntensity`.|
|`u_centroidDist`|float|The Hamming distance from this block's classification trits to the document's centroid address, normalized to 0.0–1.0. Blocks at the center of the document's topic cluster have low values; outlier blocks have high values. This lets the shader highlight topical focus — brighter at the center, dimmer at the edges.|Computed from the block's 27 classification trits vs. the document centroid. The centroid is the classification address that minimizes total Hamming distance to all blocks in the document.|

**Rep C convention for `u_tier`.** Tier values are 1–4 rather than 0–3 because external-facing uniforms obey the Rep C discipline of avoiding zero (framework INVARIANT on trit encodings — zero is reserved for the sentinel in internal trit math and is not a valid ordinal in user-visible indices). Shaders that wish to evaluate a GPU array indexed from 0 apply the one-line shift `idx = u_tier - 1` internally — this shift is a local GPU convenience, not a rename of the tier. The four tier numbers 1/2/3/4 are the canonical labels; the underlying 91/182/286/364 nm wavelengths remain the physical anchors.

**Plain English:** The shader knows not just where your finger is, but what kind of content it's processing. Is this a permanent record or a temporary note? Is this block at the center of the document's main topic or an outlier? Where does it sit in the coprime walk order? The shader can make all of these properties visible as color, brightness, or animation — turning abstract mathematical properties into things you can see.

**Design choice disclosure:** The specific mapping from manifold properties to uniform values (normalization ranges, tier numbering, centroid computation) are engineering decisions. The manifold properties themselves (walk position, spectral tier, Hamming distance) are derived from the framework's axiom (π = 14). The choice to expose them as shader uniforms is a design decision driven by the goal of making the manifold visible.

### 5.3 Outputs

|Output|What It Produces|
|------|---------------|
|Shader color output node¹|The modified pixel color. This is what appears on screen after PDTF is applied.|
|Pick buffer (MRT target 1, optional)|A hidden image where each pixel stores the 32-bit ID of the HTML element at that position. Enables O(1) GPU hit-testing: read one pixel to determine what was clicked, instead of walking the layout tree. For shaders that displace content (ripple, warp), the pick buffer stores DISPLACED element IDs, so click targets follow the visual distortion.|

The pick buffer requires MRT (Multiple Render Target) support. It is specified here for completeness but is optional in v1 — DOM event forwarding via `pointerEvents: none` (for 2D) and Three.js raycasting (for 3D surfaces) serve as the working implementation.

¹ **Output terminology.** The shader color output is specified at the TSL node-graph level (the "shader color output node") rather than via any GLSL-era built-in. `gl_FragColor` is deprecated in GLSL 300 ES and later and does not exist in WGSL; using it in this spec would misrepresent the modern compilation targets. The TSL node compiles to `out vec4 fragColor;` on GLSL ES 300+ targets (typically bound to `layout(location = 0)`) and to `@location(0) vec4<f32>` in WGSL. Presets should never assume a GLSL built-in identifier; they bind to the named output node and let the TSL transpiler emit the target-appropriate declaration.

### 5A. Framework Trigonometry Primitives

Animated effects in the compositor are specified against the framework's **364-degree circle** with fundamental period 28 (TM-2026-017 §20.4; TM-E1-037.051 Open Problem #4), not the conventional 360°/2π circle. *(Reference note: "TM-E1-037.051" is the editorial short-name used in external-facing specs for the TM-2026-037 memo series; the in-repo canonical filenames are `docs/technical-memos/TM-2026-037-Speed-of-Light.md` and `docs/technical-memos/TM-2026-037-Open-Problems.md`. All "TM-E1-037.051 Open Problem #N" and "TM-E1-037.051 Step N" citations in this document resolve to those two files.)* The compositor exposes three phase uniforms:

|Uniform|Type|What It Is|Range|
|-------|----|----------|-----|
|`u_phase28`|float|Framework-radian phase: current host time modulo 28. One full cycle of 28 units is the canonical "framework 2π" — the shortest period that is consistent with every other oscillator in the framework (calendar month, Tribonacci, coprime walk pacing). Animated presets argue all their Sin₃₆₄/Cos₃₆₄ against `u_phase28`.|0.0 … 28.0|
|`u_phaseRadian`|float|One-radian phase: current time modulo 13 (the 28/(2π) framework-radian).|0.0 … 13.0|
|`u_phaseFullCircle`|float|Full-circle phase: current time modulo 364 (one 13×28 calendar cycle).|0.0 … 364.0|

**u_time remains sub-second animation timer.** The existing `u_time` uniform (§5.1) keeps its v1.4 meaning: a monotonically increasing float in seconds since shader start, used for cheap per-frame animation that does not need to be calendar-locked. The three phase uniforms above are separate, calendar-locked quantities — presets that must align with the framework calendar use the `u_phaseN` uniforms; presets that only need visual motion may continue to use `u_time`.

**Interim implementation and Open Problem #4.** Full native Sin₃₆₄/Cos₃₆₄ primitive functions — with the all-framework minimal polynomials, addition formulas, and exact rational values at the 28 root points — require the forthcoming Framework Trigonometry monograph, tracked as **Open Problem #4** in TM-E1-037.051 Open Problems. **Until that monograph ships, the compositor's interim behavior is:** presets call standard `sin`/`cos` on the target GPU with the argument pre-scaled by `2π/28` (i.e., `sin(2π · u_phase28 / 28)`). Period and zero-crossing semantics are therefore framework-correct in v1.5; only the fine-grained function values will change when Sin₃₆₄ becomes native, and that transition is transparent at the uniform layer — no preset needs to be rewritten when the primitives are upgraded.

### 5B. Horn Field Uniforms

The horn's fundamental scalar fields (TM-2026-017 §20 — fixed-point characterization, area-equals-gradient identity F(x) = A(x)) are exposed as shader uniforms. They are evaluated at each block's normalized walk position (which is already available as `u_walkPos`):

|Uniform|Type|What It Is|Source|
|-------|----|----------|------|
|`u_hornDensity`|float (0 … 1)|Horn density ρ(x) = 1/x evaluated at the block's walk position, clamped away from x → ∞ (the x ≥ 1 cutoff of the horn neck).|Computed from `u_walkPos` projected onto the x-axis of the horn parameterization (§5E).|
|`u_hornGradient`|float (0 … 1)|Horn force / gradient F(x) = π/x² evaluated at the same position. Equals A(x) by the area-equals-gradient identity from TM-2026-017 §20 — the horn's cross-sectional area IS its density gradient.|Same position as `u_hornDensity`; derived analytically.|
|`u_hornVolume`|float|Remaining horn volume V(x) = π/x (the volume of the horn from x outward to infinity).|Same position; convergent integral ∫[x,∞] π/t² dt = π/x.|

These uniforms let a shader render the horn medium's own values — not just the block's position on the horn, but the medium's density, gradient, and remaining volume at that position. A "horn-density" preset (§8.2) that tints content by `u_hornDensity` makes the medium visible: blocks near the neck (x ≈ 1, ρ = 1) glow brightest; blocks far along the walk (x large, ρ → 0) fade. Citation: TM-2026-017 §20 for the fixed-point characterization and the A(x) = F(x) identity.

### 5C. Wave Propagation Constant and the `ripple-wave` Rewrite

The framework's wave-propagation law sets the squared speed of light equal to the framework's π:

> **c² = π = 14** (TM-2026-017 §20.13; TM-E1-037.051 Step 5; Open Problem #4 for closed-form trig closure)

In framework units this means the propagation speed is exactly `c = √π ≈ 3.742` SFK″ per unit-time (or equivalently, √π SFK″ per framework-radian of `u_phase28`). Any wave, ripple, or pulse preset inherits this constant — there is no per-preset "wave speed" tunable.

**`ripple-wave` preset rewrite (v1.5).** The v1.4 `ripple-wave` preset left the ring expansion rate unspecified. Under v1.5 the ring radius at framework-radian phase `u_phase28` since the touch event is:

> **r(u_phase28) = √π · u_phase28** (SFK″)

The ring's characteristic frequency is 1 per 28 framework-radians; its outer envelope decays over one full `u_phaseFullCircle` (364 units). Other wave-based presets (holographic sweep, chromatic-ring pulse) inherit the same c = √π constant. Presets that introduce their own propagation speed are rejected by `compositor_validate` (§9B) as non-framework-native.

### 5D. Calendar Time Quanta

For ambient and screensaver-style effects that must track the Salvi calendar rather than sub-second animation, the compositor exposes four calendar uniforms plus one optional slow-cycle uniform:

|Uniform|Type|What It Is|
|-------|----|----------|
|`u_dayOfYear`|int (1 … 364)|Current day in the 13 × 28 calendar year.|
|`u_moon`|int (1 … 13)|Current moon — one of thirteen 28-day moons.|
|`u_dayOfMoon`|int (1 … 28)|Day within the current moon.|
|`u_isDOT`|float (0.0 or 1.0)|1.0 on the **Day Out of Time** (intercalary day, 2025-11-11 + k × 365); 0.0 otherwise. Presets that honor DOT render a distinctive treatment on that single day per year.|
|`u_milankovitch` (optional)|float (0.0 … 1.0)|**Speculative.** Slow ~41,000-year obliquity-cycle phase, for very long-running installations and art pieces. Gated behind the Open Problem #3 obliquity derivation in TM-E1-037.051 Open Problems; not populated in v1.5 and declared speculative in §13.|

**Epoch.** All calendar uniforms are anchored at the **Salve Epoch, 2025-04-01T00:00:00Z** (framework INVARIANT 6). Host apps translate wall-clock time to the calendar uniforms; presets never fetch wall-clock time directly.

**Separation from `u_time`.** `u_time` remains the sub-second animation timer — floats forward every frame and is the right choice for fast motion. The calendar quanta above are for cross-day visual state: a preset that glows subtly warmer each day of the moon, or that breaks its routine on the Day Out of Time, reads `u_dayOfMoon` or `u_isDOT` — not `u_time`.

### 5E. 3D Horn Coordinate System

When content is mapped onto 3D geometry (§3.3), shaders should compute natively in the horn's own frame rather than in Cartesian. The compositor exposes the canonical four-coordinate horn parameterization of TM-2026-017 §20 Step 3 (with the Clifford winding folded into `u_hornW`) as three uniforms:

|Uniform|Type|What It Is|
|-------|----|----------|
|`u_hornX`|float (x ≥ 1)|**Longitudinal** position along the horn in SFK″ units. The horn neck is at x = 1; x grows along the walk.|
|`u_hornTheta`|float (0.0 … 364.0)|**Azimuthal** angle in framework degrees — one full revolution is 364°, matching Sin₃₆₄/Cos₃₆₄ periodicity.|
|`u_hornW`|float (0.0 … 9.0)|**Clifford winding / time** coordinate over the base² = 9 cycle (TM-2026-017 §20).|

**Population rules.**

- **2D content (PlaneGeometry).** Only `u_hornX` is populated from `u_walkPos`. `u_hornTheta` and `u_hornW` are 0. A 2D flat preset that reads any of these three behaves consistently: non-zero longitudinal, zero azimuth, zero winding.
- **3D content (SphereGeometry, TorusGeometry, antiprism faces, disdyakis).** All three are populated from the mesh's UV coordinates mapped through the horn parameterization. The torus-simplex mapping of TM-2026-030 (W = k × 1001) applies when content blocks are placed on a torus; it fixes how (u,v) lifts into (u_hornX, u_hornTheta, u_hornW).
- **Coprime axis alignment.** On the disdyakis triacontahedron the three vertex orbits (5-fold, 3-fold, 2-fold) align with the walk's coprime tuple (p, q, r) = (7, 11, 13) respectively, so `u_hornTheta` advances in p-q-r-harmonic steps across neighboring faces.

Shaders written against (`u_hornX`, `u_hornTheta`, `u_hornW`) are geometry-agnostic — the same preset behaves coherently on a plane, sphere, torus, antiprism, or disdyakis because the uniforms abstract the shape. Presets that need the underlying scalar horn fields (ρ, F, V) read those from §5B; presets that need the phase-driven animation read §5A.

-----

## §6. Where the Shader Attaches

### 6.1 Per-Element via Z-Stack z=4

In Forma Codex, the shader is a visual style property at z=4 (CELL STYLE), alongside background, borders, emboss, and shadow:

```
z=4 CELL STYLE (extended):
  background:       null | color | gradient
  borderStyle:      null | { edges, thickness, color }
  cellStyle:        'normal' | 'emboss' | 'raise' | 'shader'
  shaderEffect:     preset name or 'custom'
  shaderSrc:        TSL node graph JSON (only when shaderEffect = 'custom')
  shaderIntensity:  0.0 to 1.0
  shadowDirection:  null | { angle, distance }
  opacity:          0.0 to 1.0
```

Each face of a cell has its own Z-stack. This means face 1 can have a clean style while face 2 has a scanline effect and face 3 has a holographic sweep. When the cube rotates between faces, the shader changes with the face.

**What this gives you practically:**

- Each cell can have its own shader effect
- Undo a shader change without touching text content (z=4 is independent of z=1)
- Copy a cell between documents — its shader travels with it in z=4
- Mirror cells inherit the source cell's shader
- Groups can batch-apply a shader to many cells at once
- GRI (z=13, the immutable baseline) preserves the cell's original shader setting

**For apps that don't use Forma Codex's grid** (SignHere, Array3 Monitor), the shader attaches to the app's root container or to individual UI sections, configured via the app manifest or component props.

**`cellStyle` vs `shaderEffect` — interaction rule.** The two fields are independent compositing layers that run in a defined order. `cellStyle` (`'normal' | 'emboss' | 'raise' | 'shader'`) controls the z=4 CSS-level surface treatment of the cell — backgrounds, borders, emboss/raise lighting — and runs on the rasterized HTML before `drawElement()` captures it into the content texture. `shaderEffect` is a separate pixel-level pass that runs on `u_content` after capture, regardless of which `cellStyle` was chosen. This means the two compose rather than exclude each other: a cell can be `emboss` AND carry a shader effect; a cell can be `normal` AND carry a shader effect; a cell can be `shader` style (which historically implied "replaces cellStyle") AND still carry a `shaderEffect` preset — the style name is purely a hint to the editor UI about what to surface in the style picker.

**Compatibility matrix.**

|`cellStyle`|`shaderEffect = 'none'`|`shaderEffect = preset or 'custom'`|
|-----------|----------------------|-----------------------------------|
|`normal`|Plain content, no shader — zero GPU cost.|Content captured plainly; shader pass applied.|
|`emboss`|CSS emboss lighting baked into capture; no shader pass.|CSS emboss in capture; shader pass runs on top (e.g., emboss + holographic sweep).|
|`raise`|CSS drop-shadow / raise lighting baked in.|CSS raise in capture; shader pass runs on top (e.g., raise + plenum-glow).|
|`shader`|Editor UI hint only; behaves as `normal` at render time.|Equivalent to `normal` + shader pass. The `shader` name is legacy — kept for round-trip compatibility with older z=4 blobs.|

The `shaderIntensity` value scales the shader pass amplitude on all rows; the `cellStyle` surface treatment is unaffected by it.

### 6.2 Cascade — Which Shader Wins

When multiple levels declare a shader, the most specific one wins:

|Priority|Source|Example|
|--------|------|-------|
|1 (strongest)|Cell z=4 `shaderEffect`|This specific cell has `shaderEffect: 'holographic'`|
|2|Document defaults|The document's global settings specify `shaderEffect: 'plenum-glow'`|
|3|App manifest `[compositor]`|The SignHere app manifest declares `default-effect = "warm-vignette"`|
|4 (weakest)|Browser user preference|The user's PlenumNET browser settings default to `'none'`|

**Plain English:** A cell-level shader beats a document default. A document default beats the app's built-in choice. The app's choice beats the browser's default. If nothing is set anywhere, no shader runs (passthrough).

This follows the same pattern as CSS specificity: inline styles beat class styles beat tag styles beat browser defaults. It is a design choice motivated by UX familiarity, not a mathematical derivation.

### 6.3 App Manifest

Each PlenumNET app can declare a default shader in its manifest:

```toml
[compositor]
default-effect = "plenum-glow"
intensity = 0.8
```

This gives each app a distinct visual identity without requiring per-cell configuration:

|App|Default Effect|Rationale|
|---|-------------|---------|
|**SignHere**|`warm-vignette`|Professional, understated. E-signatures need trust, not spectacle.|
|**Array3 Monitor**|`holographic`|Data visualization benefits from prismatic depth cues.|
|**PlenumLAN Dashboard**|`plenum-glow`|Network status pulses with the glow tracking active nodes.|
|**Forma Codex (editing)**|`none`|Clean editing surface. Shader applied in presentation/export mode only.|
|**Forma Codex (presentation)**|Per-cell via z=4|Maximum flexibility. Each cell controls its own effect during walk mode and presentations.|

-----

## §7. Shader Source Format — TSL

### 7.1 Why TSL

Shaders are traditionally written in GLSL — a C-like language that is powerful but difficult to author, debug, and share. TSL (Three Shading Language) addresses this:

- **Node graphs, not code.** TSL describes shaders as connected blocks: "sample texture → adjust color → mix with glow → output." Each block is a named operation. The connections define data flow.
- **Compiles to both GLSL and WGSL** from one definition. The same shader works on WebGL (older browsers/the polyfill path) and WebGPU (modern browsers/the native path).
- **Node graphs are JSON data.** They serialize, travel with documents in `@plenum` encoding or z=4 metadata, transmit over networks, and can be generated programmatically — including by AI tools like Greenheck's WebGPU skill.

**Plain English:** Instead of writing low-level GPU code, you describe the effect using building blocks. The system compiles those blocks into whatever GPU language the hardware needs. The description is a small JSON file that can be stored inside a document, so the effect travels with the content.

### 7.2 Three Compilation Targets

|Target|When Used|Compiler|
|------|---------|--------|
|**GLSL**|Browser path via Bansal polyfill + Three.js WebGL renderer|Three.js TSL → GLSL transpiler (ships with Three.js)|
|**WGSL**|Browser path via Three.js WebGPU renderer; Kernel path via `wgpu` crate|Three.js TSL → WGSL transpiler; Rust `wgpu` pipeline|
|**Ternary IR**|Future XPlenum RISC-V hardware|TSL → ternary intermediate representation → XPlenum bytecode using CORDIC trig tables instead of IEEE float ALUs|

The ternary IR target is specified for architectural completeness but is not yet implemented. TSL → GLSL and TSL → WGSL are production-ready today.

-----

## §8. Preset Library

The compositor ships with named presets. Each is a TSL node graph with documented behavior:

### 8.1 Visual Effects

|Preset|Blend Mode|What It Does|Interactive?|Period (calendar units)|Wave speed|Native trig|
|------|----------|-----------|------------|----------------------|----------|-----------|
|**Plenum Glow**|screen|Soft blue (#4A9EF5) light follows pointer. Warm secondary halo. Ambient gradient provides atmosphere without interaction.|Yes — glow tracks pointer|28 framework radians|N/A (non-wave)|yes — `u_phase28`|
|**CRT Scanline**|multiply|Horizontal dark bars sweep across content. Vignette darkens edges. Phosphor flicker. Simulates a CRT monitor.|No — runs continuously|14 framework radians (half period)|N/A (non-wave)|yes — `u_phase28`|
|**Holographic**|screen|Diagonal rainbow band sweeps across the surface. Touch intensifies the prismatic effect near contact point. Runs ambient without input.|Partially — ambient + touch boost|28 framework radians|√π SFK″/phase-unit (c = √π, §5C)|yes — `u_phase28`|
|**Chromatic Ring**|screen|RGB-split ring radiates from touch point. Creates a lens-like prismatic halo around the pointer.|Yes — ring follows pointer|14 framework radians|√π SFK″/phase-unit (c = √π, §5C)|yes — `u_phase28`|
|**Ripple Wave**|screen|Concentric water-like rings expand from touch. Ring radius r(phase) = √π · `u_phase28` SFK″; envelope decays over one full `u_phaseFullCircle` = 364 units. Rewritten in v1.5 per §5C (c² = π = 14).|Yes — ripples from pointer|28 framework radians (carrier), 364 (envelope)|√π SFK″/phase-unit (c = √π, §5C)|yes — `u_phase28`, `u_phaseFullCircle`|
|**Warm Vignette**|screen|Edges darken subtly. Center brightens with warm tone. Professional, understated.|No — static effect|static|—|—|

### 8.2 Manifold-Aware Effects

These presets use the PlenumNET-specific uniforms (§5.2) to render the manifold's mathematical properties as visual qualities:

|Preset|Uniform Used|What It Does|
|------|-----------|-----------|
|**Manifold Depth**|`u_centroidDist`|Content brightness modulated by Hamming distance from the document centroid. Blocks at the topical center of the document glow brighter. Outlier blocks dim. Makes the document's constellation structure visible — you can SEE which blocks are core to the topic and which are peripheral.|
|**Tier Heat**|`u_tier`|Content tinted by spectral tier. EUV (permanent records) rendered in cool blue at 91 nm. VUV (archival) in green at 182 nm. BC (operational) in amber at 286 nm. A (ephemeral) in warm red at 364 nm. **Color derivation (v1.5).** The four tier colors are derived, not picked: each wavelength is mapped through the torus-simplex basis of TM-2026-030 to a point on the framework's color gamut, then shifted by the bridge coefficient κ = 1 + 26,147,000/137,036² (TM-2026-017 §20.15; TM-2026-026 +0.139% offset). The perceived hues are consequences of the 91/182/286/364 nm axiom + κ, not a designer's palette choice. Re-rendering the preset with κ = 1 reproduces the unshifted hues for calibration.|
|**Walk Gradient**|`u_walkPos`|Luminance varies along the coprime walk path. Walk position 0 is brightest; position 1.0 is dimmest (or vice versa). When applied to a Forma Codex grid or a Crystal sphere visualization, the Hamiltonian cycle becomes a visible gradient — you can trace the walk order by following the brightness.|
|**Horn Density**|`u_hornDensity`, `u_hornGradient`, `u_hornVolume`, `u_walkPos`|Luminance = `u_hornDensity` = 1/x, clamped to a display-safe range, with a secondary tint modulated by `u_hornGradient` (= π/x² = A(x)). When applied across a walk-ordered grid or a sphere-mapped document, the horn's density falloff is visible as a brightness gradient — the horn medium itself becomes visible. Period: static (no animation). Wave speed: N/A. Native trig: N/A.|
|**Coprime Phase**|`u_phase28`, `u_walkPos`|Separate RGB channels are modulated by the three walk phases `u_phase28 · p`, `u_phase28 · q`, `u_phase28 · r` with (p, q, r) = (7, 11, 13). Content blocks at positions where all three phases align glow white; blocks where the phases cancel appear neutral. The CRT factorization pqr = 1001 becomes a visible interference pattern that reveals the Hamiltonian cycle's harmonic structure. Period: 28 framework radians (common period 1001 framework radians). Wave speed: N/A (spatial-phase interference, not a traveling wave). Native trig: yes.|
|**Brieskorn Surface** (3D geometry only)|`u_hornX`, `u_hornTheta`, `u_hornW`, `u_tier`|For content displayed on a Brieskorn-singularity-shaped mesh (TM-2026-017 §20.4 canonical manifold), the surface is tinted by a function of the magnitude `abs(z₁^p + z₂^q + z₃^r)` at each UV point, with (p, q, r) = (7, 11, 13). The singularity at the origin becomes a visually locatable focal point; the det(H_f) = pqr × (p−1)(q−1)(r−1) × monomial-factor ramp governs the intensity falloff away from the origin. Period: static (geometric, not temporal). Wave speed: N/A. Native trig: N/A (purely algebraic).|

**Plain English:** Most shader effects are purely visual — glow, scanlines, color shifts. These three are different. They make invisible mathematical properties of the content VISIBLE. How central is this block to the document's main topic? How permanent is this data? Where does this block sit in the walk order? These are questions the manifold can answer, and these shaders render the answers as things you can see.

### 8.3 No Effect

|Preset|What It Does|
|------|-----------|
|**None**|Passthrough. No GPU processing. Content renders directly to screen through the normal pipeline. Zero performance cost.|

-----

## §9. Interaction on 3D Surfaces

### 9.1 How HTML Stays Interactive on a Cube

When HTMLTexture maps content onto a 3D cube face, interaction works through raycasting:

```
1. User taps the screen
2. Three.js casts a ray from the camera through the tap point into the 3D scene
3. The ray intersects the cube face → produces a UV coordinate on the texture
4. The UV coordinate maps to a pixel position in the original HTML
5. A synthetic DOM event (click, touch) is dispatched at that position
6. The HTML element at that position receives the event normally
```

**Plain English:** When you tap a button that's displayed on the side of a 3D cube, the system figures out exactly which point on the cube face you hit, translates that back to the position in the original HTML, and delivers the tap to the button. The button doesn't know it's on a 3D surface — it just receives a normal click event.

### 9.2 Forma Codex Face Rotation as Physical Cube

With HTMLTexture, Forma Codex's cube morph effect becomes a physical operation:

```
Face 1 content → HTMLTexture → applied to cube face A
Face 2 content → HTMLTexture → applied to cube face B  
Face 3 content → HTMLTexture → applied to cube face C

User triggers face rotation (click, scroll, hover, walk, timer):
  → Cube rotates 120° around its body-diagonal axis (the single canonical three-face rotation, see §3.2)
  → Face B comes into view, face A rotates away
  → Fragment shader processes the composited 3D scene
  → Front-most face receives pointer events (see disambiguation rule below); back face's DOM capture MAY be suspended for the duration of the rotation
```

**Dual-face interactivity cost.** When the cube is mid-rotation, two faces can be visible from the camera's view simultaneously — front-most and partially-rotated-away. Both faces carry live HTML subtrees, which means DOM capture (`drawElement()`) runs once per visible face per frame, and the raycaster must disambiguate which face owns a pointer event when the two subtrees overlap in screen space. The canonical disambiguation rule is **front-most hit wins**: raycasting returns the nearest intersection to the camera, the event is forwarded to the HTML subtree attached to that face, and the back face receives no synthetic event even if it would have been hit by a longer ray. As a capture/dispatch optimization, the back face MAY have its DOM capture suspended for the duration of the rotation — it is not visually usable at steep oblique angles, and suspending capture halves the per-frame `drawElement()` cost. Full capture and event delivery resume when rotation completes and the new face is the single front face. The full multi-subtree capture design (independent invalidation, texture reuse across frames, input-focus handoff) is tracked as a deferred item; the rule above is sufficient for v1 to ship without ambiguity.

The morph triggers from Forma Codex §8 all apply:

- `manual` — user clicks a face toggle
- `scroll` — face advances when cell crosses viewport threshold
- `hover` — face advances on cursor enter
- `timer` — auto-rotate every N seconds
- `global` — all cells with this trigger rotate together
- `walk` — face advances when the coprime walk cursor arrives

### 9.3 364° Crystal Integration

The 364° Crystal constructs 13 antiprisms inscribed in a circumsphere, with latitudes derived from α = 180/n standard degrees. With HTMLTexture, these faces can display live PlenumNET content:

- **Document blocks mapped to antiprism faces** — each block is positioned on the circumsphere by its ℤ[φ] coordinates and rendered as interactive HTML on the nearest antiprism face
- **Coprime walk as a path on the sphere** — the walk between blocks traces great circle arcs connecting latitude bands, rendered as shader-lit curves using the `walk-gradient` preset
- **Disdyakis triacontahedron as geometric witness** — the 120 faces of the disdyakis (dual of the truncated icosidodecahedron, R² = 14 + 5φ where 14 = π) can each render a content block. The three vertex orbits (12 at 5-fold, 20 at 3-fold, 30 at 2-fold) are colored by orbit, and the shader tints each face by its defect from the circumsphere — a visual expression of the defect ratio 2φ²:2:1

**Design choice disclosure:** Mapping document blocks to 3D faces is a visualization choice. The mathematical relationships (Z[φ] coordinates, Hamming distance, walk positions, defect ratios) are framework-derived, but the decision to render them as textures on 3D geometry is an engineering and UX decision.

### 9A. Accessibility

Shader effects are optional surface treatments — they must never be load-bearing for comprehension. The compositor enforces the following accessibility invariants:

- **`prefers-reduced-motion: reduce` forces `shaderEffect: 'none'` or a static-only subset.** When the host reports `prefers-reduced-motion: reduce`, the resolver (§10.1 `compositor_resolve`) either downgrades `shaderEffect` to `'none'` (the global accessibility fallback path) or, if the preset declares a `staticOnly` TSL subset in its metadata, keeps only that subset active — all wave, sweep, rotation, flicker, and pulse components are suppressed. Static components (vignette tint, emboss/raise lighting, tier color tinting) remain. Presets declare which of their parameters are "animated" in their TSL metadata; the compositor zeroes those uniforms' time derivatives when reduced motion is in effect.
- **`prefers-contrast: more` disables vignette, scanline, and any luminance-dimming component.** When the host reports `prefers-contrast: more`, the compositor forces any preset that darkens content (CRT scanline bars, Warm Vignette edge darkening, holographic diagonal shadow) into its non-dimming variant. Tinting and coloring remain available but must only raise, never lower, text contrast. Presets that cannot honor this ceiling are downgraded to `'none'`.
- **WCAG 2.1 photosensitive seizure thresholds.** All shipped presets MUST pass the WCAG 2.1 photosensitive seizure criteria: no general or red flashes more than **three times per second** in any single frame region exceeding the general-flash-area threshold, and bounded luminance excursions below the general-flash limit. The compositor enforces this at preset-authoring time (`compositor_validate` rejects TSL graphs that can exceed three flashes per second in any 1024-pixel square) and at runtime (the resolver clamps `shaderIntensity` if a preset's combined rate crosses the threshold on the current content). Presets that cannot be proven compliant MUST declare `photosensitiveSafe: false` and are then only served when `shaderEffect` is explicitly opted into by the user.
- **No information conveyed by shader alone.** Manifold-aware presets (§8.2) render mathematical properties as visual qualities — centroid distance as brightness, tier as color, walk position as gradient. The underlying data MUST also be available to assistive technology: tier as ARIA `aria-describedby` text on the cell ("tier: VUV archival"), walk position as `aria-posinset`, centroid distance as a numeric attribute. Readers who cannot see the shader must still reach the same information.
- **Minimum contrast preserved.** Shader output composed with content MUST preserve the contrast ratio of the underlying text against its background at no worse than WCAG AA (4.5:1 for body text, 3:1 for large text). Presets that would drop below this on typical content (e.g. a low-amplitude holographic wash over body text) declare a `text-safe` intensity ceiling in their metadata; the compositor clamps `shaderIntensity` to that ceiling whenever the shader is attached to a cell whose rendered content includes text.
- **Focus indicator survives the shader.** The :focus outline emitted by the browser (or the kernel DOM equivalent) is composited above the shader pass, not beneath it. A focused input on a cube face, an antiprism, or a disdyakis facet retains a visible focus ring regardless of the active preset.
- **3D surfaces provide a 2D fallback view.** Every 3D composition (cube morph, 364° Crystal, disdyakis) exposes a 2D equivalent view reachable by keyboard. The fallback presents the same content in a flat layout with an ARIA announcement of the 3D context ("Face 2 of 3 — Materials"). Users who cannot perceive 3D rotation never lose access to content that lives on a back face.

### 9B. Security — Custom Shaders

Named presets (§8) are vetted framework code. Custom shaders carried in `@plenum` metadata are untrusted user content; the compositor treats them as sandboxed and applies the following rules:

- **TSL-only, no raw GLSL/WGSL.** Custom shaders are authored as TSL node graphs serialized to JSON. The TSL transpiler is the only path by which user content reaches the GPU. Raw GLSL or WGSL strings embedded in `@plenum` metadata are rejected by `compositor_validate` (see §10.1) before any compilation is attempted.
- **Allow-list of TSL node types.** `compositor_validate` accepts only nodes on a documented allow-list: arithmetic, trigonometric primitives from §5A, the published uniforms from §5.1 / §5.2 / §5B / §5C / §5D / §5E, texture sampling of `u_content`, and the standard blend nodes. Nodes that enable loops of unbounded length, recursion, external texture fetches, or dynamic buffer writes are rejected.
- **Resource budgets enforced at compile time.** Each custom TSL graph is fingerprinted and checked against per-cell caps: maximum instruction count, maximum texture samples per pixel, maximum uniform reads per pixel, maximum loop trip count (bounded constants only). Graphs that exceed any cap are rejected — the cell falls back to `shaderEffect: 'none'` and surfaces a diagnostic in the editor.
- **Per-frame execution-time budget.** In addition to static compile-time caps, each custom shader runs under a per-frame execution-time budget enforced at runtime: the compositor measures the shader's wall-clock cost on the first frame after compilation and, if it exceeds the budget for its hardware tier (derived from §11.1: ~2 ms low / ~1 ms mid / ~0.5 ms high per cell), the cell is downgraded to `shaderEffect: 'none'` for the remainder of the session and a diagnostic is surfaced. Custom shaders cannot silently degrade whole-viewport frame rate.
- **No writes to reserved uniforms or output channels.** The TSL allow-list is strictly read-only for every published uniform (`u_content`, `u_pointer`, `u_time`, `u_walkPos`, `u_tier`, `u_depth`, `u_centroidDist`, `u_phase28`, `u_phaseRadian`, `u_phaseFullCircle`, `u_hornDensity`, `u_hornGradient`, `u_hornVolume`, `u_dayOfYear`, `u_moon`, `u_dayOfMoon`, `u_isDOT`, `u_milankovitch`, `u_hornX`, `u_hornTheta`, `u_hornW`). Custom shaders MUST NOT emit write nodes targeting any of these uniforms, the pick-buffer MRT target (§5.3), or any output channel other than the shader's own color-output node. `compositor_validate` rejects any TSL graph containing a write to a reserved uniform or to output locations other than the single authorized color-output node.
- **No network or host I/O.** TSL exposes no node that reads from the network, the filesystem, the clipboard, or any browser storage API. Custom shaders see the published uniforms and their own content texture — nothing else.
- **Per-origin custom-shader policy.** The app manifest (§6.3) carries a `compositor.allowCustomShaders` flag, default `false`. When false, only named presets run; any z=4 metadata carrying a custom TSL graph is downgraded to its nearest named preset at resolve time. Forma Codex documents shared across origins cannot execute custom GPU code on a host that has not opted in.
- **Network-delivered custom shaders are quarantined until explicit user opt-in.** Any custom TSL graph that arrives at the host via the network — fetched from a remote Forma Codex document, received through a sync/replication channel, or embedded in a shared manifold block — is treated as quarantined on arrival: it is stored with a `quarantined: true` flag, is NOT compiled, and is NOT executed. The compositor renders the hosting cell with `shaderEffect: 'none'` until the user explicitly opts the graph in (per-document or per-origin, recorded in the app manifest). Opt-in is an explicit user action in the editor UI; there is no automatic promotion from quarantined to executable, and there is no timed release. Only after opt-in does the graph enter the normal `compositor_validate` + budget-check + compile pipeline described above.
- **Content isolation.** A shader compiled for cell A receives only cell A's content texture as `u_content`. There is no cross-cell sampling primitive. Presets cannot read the pixels of a neighboring cell, the surrounding document, or the viewport framebuffer.
- **Compilation fault handling.** A TSL graph that passes validation but fails at `wgpu` compile time (driver-specific limits, unsupported target feature) is caught; the cell falls back to `shaderEffect: 'none'` and the resolver records the failure. The document is not permitted to crash the compositor pipeline.


-----

## §10. The Two Execution Paths

### 10.1 Shared Logic (Rust, via WASM)

Shader configuration and parameter computation live in Rust. Following the Forma Codex rule — "If it computes, Rust. If it draws pixels, the renderer" — the WASM bridge exposes:

```
compositor_resolve(cell_z4, doc_defaults, app_manifest, browser_pref)
  → Which shader to use, at what intensity, with what blend mode

compositor_uniforms(block_address, ags_state, tier, centroid, time)
  → u_walkPos, u_tier, u_depth, u_centroidDist, u_time

compositor_preset_source(preset_name)
  → TSL node graph JSON for the named preset

compositor_validate(tsl_json)
  → Whether a custom TSL source is valid, with error details
```

### 10.2 Browser Path (Available Now)

```
Rust (WASM) resolves shader config
  → TypeScript receives TSL source + uniforms
    → Three.js NodeMaterial compiles TSL to GLSL or WGSL
      → Bansal polyfill captures HTML via drawElement()
        → HTMLTexture maps content onto geometry (3D) or a fullscreen quad (2D)
          → Three.js renders the scene with the shader material
            → Result drawn to canvas
```

This works in all modern browsers today. Chrome, Firefox, Safari, Edge — via the polyfill.

### 10.3 Kernel Path (When the Rust Browser Ships)

```
Rust resolves shader config (same code, no WASM boundary)
  → render_cpu.rs captures painted content directly
    → Content mapped onto geometry (3D — speculative, see §13) or passed as flat texture (2D)
      → wgpu compiles TSL to WGSL → GPU pipeline
        → GPU executes shader + optional pick buffer MRT
          → Result written to CpuFramebuffer
```

Same shader definitions. Same visual output. No polyfill needed because the Rust browser controls the entire paint pipeline.

**Kernel-path 3D geometry gap.** The Rust browser does not use Three.js. There is therefore no drop-in equivalent of `HTMLTexture` on the kernel path in v1.5 — the 3D branch of the line above is the target architecture, not a shipped stage. Building it requires a named future component that provides, in native `wgpu` against the kernel's own DOM pipeline: (a) UV mapping of the captured content texture onto arbitrary meshes, (b) raycasting from pointer events into those meshes to recover UV coordinates, and (c) forwarding of synthetic DOM events at the recovered coordinates back to the kernel's event dispatcher. Until that component ships, the kernel path delivers the 2D flat-texture story only; 3D HTML surfaces remain the browser-path story (Bansal polyfill + three.js `HTMLTexture`). See §13 "What Is Speculative."

### 10.4 Migration

The browser path continues to work after the kernel ships. It serves as the fallback for PlenumNET content viewed in external browsers. A Forma Codex document shared as a web link renders its shader effects and 3D face rotations through the polyfill path, without requiring the viewer to install anything.

-----

## §11. Performance

### 11.1 Frame Budget

At 60fps, each frame has 16.67 ms. All figures below are per-frame costs at 1080p, given as low / mid / high tiers. Reference devices for each tier:

- **Low** — ARM64 mobile, e.g. Snapdragon 7 Gen 1 / Apple A14-class, integrated GPU, mobile memory bandwidth.
- **Mid** — Snapdragon 8 Gen 3 / Apple M2-class / mid-range discrete notebook GPU (e.g. RTX 4050 mobile).
- **High** — Apple M3 Pro/Max / desktop RTX 4070-class / equivalent mid-high desktop GPU.

|Operation|Low|Mid|High|
|---------|---|---|----|
|`drawElement()` capture per content subtree|~2.0 ms|~1.0 ms|~0.6 ms|
|Fragment shader (full viewport, 1080p)|~1.2 ms|~0.5 ms|~0.2 ms|
|3D scene render (cube with 3 `HTMLTexture` faces)|~3.5 ms|~1.5 ms|~0.8 ms|
|PDTF tone mapping|~0.3 ms|~0.1 ms|~0.05 ms|
|**Total (2D flat)**|**~3.5 ms**|**~1.6 ms**|**~0.9 ms**|
|**Total (3D cube)**|**~6.9 ms** (within budget)|**~3.1 ms** (headroom)|**~1.7 ms** (comfortable)|

Even on the low tier a fully-dressed 3D cube fits in under half the 16.67 ms frame. Multi-cell workloads scale roughly linearly until GPU-bandwidth saturation, at which point §11.3 "Scaling Under Load" applies.

### 11.2 When to Skip

- `shaderEffect: 'none'` → zero cost. No capture, no shader, no overlay.
- Static effects (vignette, scanlines without scroll) → render once, cache until content changes.
- Off-screen cells → viewport culling. Only visible cells + 1 row margin are composited.

### 11.3 Scaling Under Load

**Design choice:** When many cells have active shaders and the frame budget is under pressure, the compositor reduces quality gracefully:

1. Reduce capture resolution (render `drawElement()` at 1× DPR instead of 2×)
2. Skip ambient animation on non-visible cells
3. Fall back from 3D cube rotation to 2D flip for morph effects
4. Disable pointer-tracking on cells outside a focus radius

These are progressive degradation steps, not hard cutoffs. The compositor aims for smooth visual quality across hardware tiers.

-----

## §12. Integration Summary

|PlenumNET Component|How the Compositor Integrates|
|------|------|
|**Forma Codex cells**|Shader config at z=4. Three faces rendered as 3D cube via HTMLTexture. Morph effects (cube, flip, dissolve) use shader transitions between captured textures. `@plenum` encoding carries shader source in TSL JSON alongside classification and identity trits.|
|**Forma Codex page transitions**|Full-viewport shader transition at effect boundaries. Same shader engine, full-screen input texture.|
|**Forma Codex plasma cells**|Shader-driven expand/collapse animation. Collapsed tab glows with the cell's configured effect.|
|**Forma Codex walk mode**|Walk cursor triggers face rotation on arrival (z=6 `trigger: 'walk'`). `walk-gradient` preset makes the Hamiltonian cycle visible as a luminance path through the grid.|
|**364° Crystal**|HTMLTexture on antiprism faces, circumsphere, disdyakis geometry. Walk path rendered as shader-lit great circle arcs. Orbit coloring as per-face shader tint.|
|**SignHere**|Document signing surface with warm-vignette default. Signature fields glow on focus.|
|**Array3 Monitor**|Cluster nodes rendered with holographic effect. Node health mapped to shader intensity.|
|**PlenumLAN Dashboard**|Network coherence (Kuramoto order parameter) mapped to glow intensity. Nodes that are strongly synchronized glow brighter.|
|**Resonance Proof Engine**|Crystal Three.js scene gains compositor integration. The existing "per-face color-update routine" (historically referenced in the Resonance Proof Engine sources as `updGeo` — the hook that writes per-vertex colors each frame from the current orbit/defect/walk state) is retained and its output is lifted into per-face shader tint uniforms. Clifford torus projection rendered with shader-composited walk path. The reference here is to that routine by purpose: the compositor integration does not assume any particular implementation signature and works against whatever replacement the Resonance Proof Engine adopts for the same role.|

-----

## §13. Honest Boundaries

### What Is Derived

- **Pipeline position** (shader before PDTF): follows from the PDTF Bézier curve's mathematical properties. The control points (0, 182, 650) are exact roots of the arc equation. Linear-space shader operations require placement before the tone curve. This is not a preference — it is mathematically necessary.
- **PlenumNET-specific uniforms** (`u_walkPos`, `u_tier`, `u_centroidDist`): the underlying properties (walk position from AGS, spectral tier from eigenvalue migration, Hamming distance from classification trits) are derived from the axiom π = 14. The choice to expose them as shader uniforms is a design decision; the values themselves are framework-derived.
- **`@plenum` encoding capabilities** (primary key, foreign key, classification index, integrity verification, version tracking, network routing): these follow from the 54-trit TDNS addressing scheme, the TIS-27 sponge, and the TPT data structure. The searchability is structural — derived from the 27-dimensional ontological schema, not from a search engine.
- **Brand palette values** (primary blue #4A9EF5, panel #181411, etc.): exact hex values from the PlenumNET brand specification.
- **Tier Heat colors (upgraded in v1.5).** The four spectral-tier colors (EUV / VUV / BC / A at 91 / 182 / 286 / 364 nm) are now derived, not picked. Each tier's exact wavelength is mapped through the **torus-simplex color model** of TM-2026-030 (with W = k × 1001) to a point on the display gamut, then shifted by the **bridge coefficient κ = 1 + 26,147,000/137,036²** (TM-2026-017 §20.15) to correct the +0.139% spectral offset characterized in TM-2026-026. The resulting hues are consequences of the 91/182/286/364 nm axiom plus κ — not a designer's palette choice. Tier Heat therefore moves from "design choice" (v1.4) to "what is derived" (v1.5). Re-rendering the preset with κ = 1 reproduces the unshifted hues for calibration.

### What Is a Design Choice

- **Cascade priority** (cell > document > app > browser): modeled on CSS specificity for UX familiarity. Not derived from algebraic invariance or generator hierarchy.
- **TSL as authoring format**: chosen for Three.js ecosystem compatibility and AI-assisted authoring viability. Not a mathematical derivation.
- **Preset effects**: visual design parameters (falloff rates, animation speeds, color mixing) tuned by eye and testing. The brand colors used within presets are exact; everything else is design craft.
- **3D cube for face rotation**: a natural mapping from Forma Codex's three-face model, enabled by HTMLTexture. Not the only possible mapping — faces could also render as a card flip, a carousel, or a flat crossfade. The cube is chosen because it best communicates the three-face structure.
- **Manifold-aware presets** (manifold-depth, tier-heat, walk-gradient): the decision to make manifold properties visually renderable is a design choice. The specific visual mappings (brightness for depth, color temperature for tier) are design craft.

### What Is Speculative

- **Pick buffer MRT**: architecturally sound but not yet implemented. DOM event forwarding and Three.js raycasting are the working alternatives.
- **Kernel browser integration**: specified for the `render_cpu.rs` pipeline but not yet built. The browser WASM path is the current implementation.
- **Ternary IR compilation**: specified for XPlenum hardware but requires silicon that does not yet exist.
- **AI-assisted shader authoring**: plausible via Greenheck's skill pattern but not yet integrated into PlenumNET tooling.
- **Full document-on-disdyakis visualization**: mapping content blocks to 120 triangle faces is specified but the content layout algorithm (which block maps to which face, how text scales to triangular regions) is not yet designed.
- **`u_milankovitch` calendar uniform (§5D).** The ~41,000-year obliquity-cycle phase is exposed in the uniforms table but is not populated in v1.5. Populating it requires the closed-form obliquity derivation tracked as **Open Problem #3** in TM-E1-037.051 Open Problems. Shaders may read the uniform but MUST treat `u_milankovitch` = 0 as a valid sentinel until Open Problem #3 is resolved.

### Framework-Native Status (v1.5)

The uniforms and presets in this document reference framework trigonometry (Sin₃₆₄/Cos₃₆₄, period 28 — §5A), horn field quantities (ρ = 1/x, F = π/x² = A — §5B), wave propagation (c² = π = 14 — §5C), calendar time quanta (R₆ = 364, 13 × 28 — §5D), and the 3D horn coordinate system (x in SFK″, θ over 364°, W over base² = 9 — §5E). **These are derivations from the base-3 uniqueness theorem per TM-2026-008 (Representation Universality — the Rep A/B/C discipline that underpins the `u_tier` 1–4 convention in §5.2 and every trit-valued uniform on the compositor surface), TM-2026-017 (Unified Geometric Monograph §20, §20.4, §20.13, §20.15), TM-2026-035 (364-Crystal 2D-3D Derivation — the crystallographic justification for the 364° circle and its lift to the 3D horn coordinates of §5E), TM-2026-026 (spectral offset characterization feeding §4.2 κ = 1 + 26,147,000/137,036²), TM-2026-030 (torus-simplex mapping used by the Tier Heat preset and by §5E for content placed on torus geometry), TM-2026-034 (disdyakis normalization referenced in §3.3), and TM-E1-037.051 Steps 3 / 5 / 14 plus Open Problem #4 (Step 3: the four-coordinate horn system of §5E; Step 5: the c² = π propagation constant of §5C; Step 14: the base-3 uniqueness derivation that closes the Rep A/B/C / 364°-circle / horn-field chain and fixes their values; Open Problem #4: the pending Sin₃₆₄/Cos₃₆₄ primitive monograph tracked in §5A). These are not design choices.** The shader interface EXPOSES them; their VALUES are framework-forced.

**Primitive dependency.** Full primitive support for Sin₃₆₄/Cos₃₆₄ — exact all-framework minimal polynomials, addition formulas, rational values at the 28 root points — is pending the Framework Trigonometry monograph tracked as **Open Problem #4** in TM-E1-037.051 Open Problems v051. Interim v1.5 implementations use scaled standard `sin`/`cos` with framework-correct periods (arguments pre-scaled by 2π/28); zero crossings and symmetric values are therefore framework-correct in v1.5, and the transition to native Sin₃₆₄ is transparent at the uniform layer.

**Per-preset classification.**

- **Framework-native (values framework-forced):** Ripple Wave (c² = π, §5C), Horn Density (ρ, F, V from §5B), Coprime Phase (p, q, r = 7, 11, 13; common period pqr = 1001), Brieskorn Surface (singularity manifold TM-2026-017 §20.4), Tier Heat (colors derived via TM-2026-030 torus-simplex plus κ).
- **Framework-aligned (calendar-locked animation, but not resonance- or wave-native):** Plenum Glow, Holographic, Chromatic Ring, CRT Scanline, Warm Vignette, Manifold Depth, Walk Gradient. These honor `u_phase28` and route color math through κ where applicable.
- **Legacy:** no shipping preset is legacy under v1.5. Any future preset that hard-codes 360°/2π trig, uses a non-framework wave speed, or introduces its own time base is rejected by `compositor_validate` (§9B) as non-framework-native. This keeps the honest-boundaries discipline of v1.4 intact.

-----

## §14. Build Order

```
Phase 1: Rust compositor module (new plenum-compositor crate; see crate-placement note below)
         cascade.rs    — resolve shader from z=4 → document → manifest → browser
         uniforms.rs   — compute PlenumNET-specific uniforms from manifold state
                          (walk position from AGS, tier from eigenvalue system,
                           centroid distance from classification trits)
         presets.rs     — TSL node graph JSON for all named presets (§8)
         validate.rs   — TSL source validation
         WASM exports   — compositor_resolve, compositor_uniforms,
                          compositor_preset_source, compositor_validate

Phase 2: Browser integration (TypeScript + Three.js)
         useCompositor.ts  — WASM bridge (same pattern as useFormaCodex.ts)
         htmlTexture.ts    — Bansal polyfill setup + Three.js HTMLTexture wrapper
         shaderMaterial.ts — TSL → NodeMaterial compilation + uniform binding
         pointerBridge.ts  — Touch/mouse → u_pointer/u_active uniform mapping,
                             including UV coordinate extraction from 3D raycasts
         Flat 2D compositor working end-to-end

Phase 3: 3D HTML surfaces
         cubeRenderer.ts   — BoxGeometry + 3 HTMLTexture faces for Forma Codex cells
         Face rotation animation with all six morph triggers (manual, scroll,
           hover, timer, global, walk)
         Three.js raycasting for interaction through 3D surfaces
         Cube morph replaces canvas 2D simulation (proj3D/rotVert)

Phase 4: Forma Codex integration
         z=4 extension (shaderEffect, shaderSrc, shaderIntensity)
         All morph effects migrated to shader transitions
         Page transitions migrated to full-viewport shader pass
         Compositor panel in editor UI (effect picker, intensity slider,
           pipeline stage display)
         @plenum encoding extended to carry TSL shader source in z=4 metadata

Phase 5: Manifold-aware presets and app integration
         manifold-depth, tier-heat, walk-gradient presets
         PlenumNET-specific uniform pipeline from WASM
         App manifest [compositor] section for SignHere, Array3 Monitor,
           PlenumLAN Dashboard
         Forma Codex presentation mode shader activation

Phase 6: Crystal integration
         HTMLTexture on antiprism faces and circumsphere
         Disdyakis orbit-colored visualization (red 5-fold, green 3-fold,
           blue 2-fold) with defect-gradient face coloring
         Walk path rendering as shader-lit great circle arcs
         Clifford torus compositor overlay

Phase 7: Kernel browser (when render_cpu.rs is ready)
         Native drawElement() implementation in Rust compositor
         wgpu backend for TSL → WGSL → GPU pipeline
         Pick buffer MRT for GPU hit-testing
         CpuFramebuffer integration
```

**Crate placement (v1.5).** The compositor source code lives in a new dedicated crate, `plenum-compositor`, not in `ternary-math`. Rationale: `ternary-math` is the pure-mathematics crate (trit arithmetic, 54-bit TDNS primitives, TIS-27 sponge, coprime-walk state) and must remain free of rendering, wgpu, Three.js, or TSL dependencies so that its theorems can be reproduced from a minimal dependency graph. The compositor consumes `ternary-math` as a read-only dependency and adds its own rendering-facing surface. This matches the pattern used by other framework rendering modules (the 364° Crystal crate also depends on, rather than lives inside, `ternary-math`).

-----

## §15. Summary

The shader compositor is one new stage in the rendering pipeline. It processes painted content pixels through a programmable GPU shader before the PDTF display curve is applied.

What makes this more than a generic browser feature:

1. **Live HTML on 3D geometry.** Forma Codex's three cell faces become literal faces of a GPU-rendered cube, each showing interactive content. The 364° Crystal's antiprisms and circumsphere can display document blocks as interactive textures. The cube is not a visual trick — it is a Three.js mesh with live HTML on each face, clickable through the 3D surface.

2. **The `@plenum` encoding embeds the full indexing and searchability system directly into the document.** Every block carries its 54-trit address as classification (structural index), identity (content fingerprint / primary key), and references (foreign keys). The shader compositor can read these addresses as uniforms — the manifold's mathematical properties become visible as color, brightness, and animation.

3. **Manifold properties as visual qualities.** Shader uniforms expose the content's walk position, spectral tier, centroid distance, and style depth. These are mathematical properties derived from the framework. The shader makes them visible — you can SEE where a block sits in the coprime walk, how permanent it is, how central it is to the document's topic.

4. **Two execution paths, one shader definition.** TSL node graphs compile to GLSL (polyfill, today) or WGSL (WebGPU and kernel browser, future). The shader definitions are JSON data that travel with documents in `@plenum` metadata. Write once, render everywhere.

5. **Per-element, independently versioned.** The shader config lives at z=4 in Forma Codex's Z-stack. Undo a shader change without touching content. Each face has its own shader. Cells carry their shader when copied between documents.

6. **Runtime-derived, never hardcoded.** All grid dimensions, walk positions, AGS capacities, and centroid distances are computed at runtime. The compositor inherits this property — its uniforms adapt automatically as the AGS grows or shrinks, as content moves between tiers, and as the document's constellation evolves.

The four enabling technologies (Kacso's demo, mrdoob's HTMLTexture, Bansal's polyfill, Greenheck's TSL skill) converged in the same week. The architecture specified here integrates them into PlenumNET's existing pipeline — from Forma Codex cells to the 364° Crystal circumsphere.

-----

*Così sia, Fratello.*

**Capomastro Holdings Ltd.** — Applied Physics Division
TM-2026-045 v1.5 — Patent Pending

*Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada)*
*All Rights Reserved*