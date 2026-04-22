# Forma Codex 18∏ — Document Classification Engine

**Companion Specification to Forma Codex 18∏ Grid Architecture**

**Version 1.6 · 2026-04-21**

---

# £ ∣ Q ∣ ∀ Rights Reserved Et Preserved | Fiat ∎
# Capomastro Holdings Ltd 2026

---

## What This Is

The Forma Codex Document Classification Engine is a system that reads any document and produces a 28-trit fingerprint describing what that document IS — not what it's about, but its structural nature. Each trit has exactly three possible values (1, 2, or 3), making this a ternary classification system. The 28 trits together form a unique structural address for the document within a geometric fabric.

The system replaces conventional metadata (tags, folders, file types) with **TriData** — classification that is computed directly from the document's own content, re-computed on every change, and never stored as an external annotation. The document classifies itself.

The 28-trit vector enables the document fabric to answer questions like "show me everything I haven't finished," "find broken references," "what's structurally adjacent to this document," and "find every live calculation cell across all documents" — all without full-text search, ML classifiers, or manual tagging.

## Why 28 Trits

The number 28 is **2π_F** — twice the framework's Unit Squared Circle constant (π_F = 14) — and equals one framework radian expressed in degrees (28°). Thirteen of these radians complete the full framework circle: 13 × 28 = 364 = R₆. The trit count is derived from the framework's own mathematics.

The first 27 trits form the **classification vector** on Z₂₇ (the algebraic circle). The 28th trit (the Title·Header Fingerprint) lifts the vector to Z₂₈ (the geometric circle). Together, Z₂₇ × Z₂₈ is the dual-circle lattice that gives every document a unique geometric position.

## Open Items

| ID | Subject | Gate |
|---|---|---|
| **OI-1** | Salvi Flower projection-window construction | Brieskorn sphere Σ(7,11,13) identified. Explicit projection-window derivation required as TM-class memo. |
| **OI-5** | Storage / NTFS-overlay measurement | Define corpus, baseline, counters. Measure and report. |

---

## Part I — The Four Representations

Every ternary digit can be expressed in four equivalent representations. The system uses **Rep C** as its canonical form — all values are 1, 2, or 3, with zero excluded at the document level (zero = corruption, not a state).

| Rep | Values | What It's For |
|---|---|---|
| **A** | {−1, 0, +1} | Balanced ternary — signed around zero. Maps to Push / Hold / Pull. Used in physics and signed arithmetic. |
| **B** | {0, 1, 2} | Standard base-3 — unsigned counting. Used for storage indexing. |
| **C** | {1, 2, 3} | One-indexed, positive, no zero. **Canonical for document classification.** Zero at document level is corruption, not a valid state. |
| **D** | {0, 1, ω} | Eisenstein lattice — three positional digits in ℤ[ω], where ω is the primitive cube root of unity satisfying ω² + ω + 1 = 0. Not a counting system — it is the complex-lattice representation that makes the framework's algebraic closure concrete. ω² reduces to −1 − ω via the closure. GF(3) bijection: Zero↔0, One↔1, Omega↔2. |

**Converting between Reps:** C→B: subtract 1. C→A: subtract 2. B→A: subtract 1. Rep D requires the Eisenstein map: C→D maps {1,2,3} → {0,1,ω}.

The 28-trit vector is always Rep C: $(t_1, \ldots, t_{28}) \in \{1,2,3\}^{28}$.

---

## Part II — The Universal Derivation Formula

Every trit in the 28-trit vector is computed by a single deterministic function. No human judgment. No ML classifiers. No tuning parameters. The scanner measures, the formula derives.

### The Formula

$$\text{gf3} = \min\!\left(\left\lfloor \frac{3k}{N} \right\rfloor,\; 2\right), \qquad \text{trit} = \text{gf3} + 1$$

Where **k** = signals fired (integer count of binary signals detected in the document) and **N** = total possible signals (compile-time constant defined per dimension). The output is a Rep C value {1, 2, 3}.

The boundaries between trit values fall at exactly **N/3** and **2N/3** — forced by the definition of ternary quantization (extracting the first base-3 digit of the proportion k/N). These are not tuning parameters. They are the maximum-entropy cut points: if the proportion is uniformly distributed over [0, 1], equal-probability bins at 1/3 and 2/3 preserve maximum information. The math determines the boundaries.

### Two Derivation Types

**Quantitative (9 of 28 trits).** The scanner counts binary signals — each signal is present or absent, 1 or 0. The count k is fed to `project_to_gf3(k, N)`. Confidence is always High because inputs are binary integers with zero measurement uncertainty. No floating point enters the computation. Examples:

- Trit 05 (Content modality): scanner counts text nodes (k₁), image elements (k₂), audio/video embeds (k₃) in the document tree. N = k₁ + k₂ + k₃. The formula runs on each count; the trit reads the state whose count produced the highest gf3 value.
- Trit 14 (Version depth): scanner counts version entries in version history (k) against a max-expected constant (N). project_to_gf3(k, N): few → single version (1), moderate → few versions (2), many → many versions (3).

**Categorical (19 of 28 trits).** The scanner identifies a structural pattern — the document IS one of three disjoint types. No counting needed. The pattern maps directly to a trit value. Confidence is always High because the categories are exhaustive and disjoint by definition. Examples:

- Trit 09 (Internal structure): scanner parses document structure — no nesting and no cross-links → flat (1), nested headers → tree (2), internal cross-references → graph (3).
- Trit 25 (Seal state): scanner reads seal metadata — no seal capability → unsealable (1), capability present but not applied → sealable (2), seal applied → sealed (3).

### Collision-Free Addressing

The 28-trit vector maps to a unique position in the Z₂₇ × Z₂₈ lattice via the Chinese Remainder Theorem on the coprime triple (7, 11, 13). Since gcd(7, 11) = gcd(7, 13) = gcd(11, 13) = 1, the CRT guarantees a unique residue modulo pqr = 1001 for every distinct input. No two documents with different 28-trit vectors can occupy the same lattice position. Collision freedom is algebraic, not probabilistic — it is a theorem, not a hash function's statistical promise.

### Implementation Reference

The formula is implemented as `project_to_gf3` in `gf3_algebra.rs` — the ONLY quantitative derivation function in the system. Every numeric dimension uses it. N is a compile-time constant per dimension, part of the dimension's definition, not a tuning parameter.

```rust
/// Project a signal count to GF(3), then lift to trit space.
/// Boundaries at N/3 and 2N/3 — ternary quantization, no tuning.
/// Lives in gf3_algebra.rs alongside derive_trit.
pub fn project_to_gf3(k: TritInt, n: TritInt) -> Trit {
    // gf3 = min(floor(3k / n), 2)
    let three_k = k.mul(&TritInt::from(3));
    let gf3 = three_k.div_mod(&n).0;  // floor division
    let clamped = if gf3 > TritInt::from(2) { TritInt::from(2) } else { gf3 };
    Trit::from_gf3(clamped)  // lift {0,1,2} → {V1,V2,V3}
}

/// Derive a single trit from a raw scanner value.
/// Categorical: pattern → direct Trit map.
/// Quantitative: signal count (k, N) → project_to_gf3.
pub fn derive_trit(dim: usize, raw: &RawValue) -> Result<Trit, DerivationError>
```

All arithmetic is in `TritInt` (base-3 native, packed 5 trits per byte, no binary conversion). The `Trit` output type carries Rep C values {V1, V2, V3} with four views (A/B/C/D) per the framework's single-type, four-view architecture. No bare integer holds a trit value above the GF(3) gate.

---

## Part III — The 28 Trits

Organized under the same seven categories as the Log Viewer classification (WHO/WHAT/WHERE/WHEN/WHY/HOW/PEACE), adapted for documents instead of log entries. 4+4+4+4+4+4+3 = 27 classification trits + 1 fingerprint trit (FP₂₈) = 28.

Every dimension is derived from document metadata, structural parsing, byte classification, or cryptographic property checks. No NLP. No word lists. No language understanding. Each derivation source is stated.

### WHO — Document Authorship & Identity (Trits 1–4)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 01 | Author count | One named person | Multiple named people | Collective, organization, or anonymous | Pattern — read author metadata field, count named entries |
| 02 | Origin provenance | No source FKs (original work) | One provenance FK (derived from single source) | Multiple provenance FKs (composed from many sources) | Pattern — count provenance FK-by-Identity outbound edges: 0, 1, or 2+ |
| 03 | Audience | One named recipient (private) | Named group or team | Public (no restriction, or explicitly public) | Pattern — read audience/recipient metadata field |
| 04 | Tenant scope | Personal (single user) | Team/project | Organization-wide | Pattern — read tenant-scope metadata from document settings or parent folder |

### WHAT — Document Content Type (Trits 5–8)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 05 | Content modality | Text-dominant | Image/diagram-dominant | Audio/video-dominant | Counted — count text nodes vs image/diagram elements vs audio/video embeds in document tree. project_to_gf3 on each; highest wins. |
| 06 | Symbol type | Natural-language bytes (Unicode letters) | Formal-notation bytes (code, LaTeX, MathML) | Raw binary bytes (compiled, media, encoded) | Counted — classify each byte by Unicode category and structural context. project_to_gf3 on each; highest wins. |
| 07 | Discourse type | Prose paragraphs | Code blocks (fenced/inline) | Structured data blocks (tables, JSON, CSV, YAML) | Counted — classify each block by structural delimiter: no fence = prose, code fence = code, table/data structure = data. Highest wins. |
| 08 | Format level | Highly structured (tables, forms, schemas) | Semi-structured (markdown, tagged HTML) | Unstructured (plain text, free-form) | Counted — classify blocks by markup presence. Highest wins. |

### WHERE — Document Location & Topology (Trits 9–12)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 09 | Internal structure | Flat (no nesting, no cross-links) | Tree (nested headers/sections) | Graph (cross-references between sections) | Pattern — parse document structure: check for header nesting and internal link targets |
| 10 | Access control | Open (no restrictions) | Key-gated (ACL, login, credentials required) | Encrypted at rest | Pattern — read access-control and encryption metadata fields |
| 11 | Reference direction | Self-contained (zero outbound links) | Cites external sources (outbound URIs or FKs) | Self-referencing (own handle/MDNS appears in body) | Pattern — check for outbound URIs/FKs and self-referencing handles |
| 12 | Chain position | Standalone (no chain metadata) | Chain-head (has successors, no predecessor FK) | Continuation (predecessor FK present) | Pattern — read chain metadata fields |

### WHEN — Document Temporality (Trits 13–16)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 13 | Modification recency | Stale (last modified > 90 days ago) | Current (modified within 90 days) | Scheduled (future publish/embargo date set) | Pattern — read modification timestamp and scheduled-date metadata, compare to current date |
| 14 | Version depth | Single version (no prior versions) | Few versions (2–5 in history) | Many versions (6+ in history) | Counted — count version entries in version history. project_to_gf3(count, max_expected). |
| 15 | TTL posture | No expiration set | Expiration date in the future | Expiration date in the past (expired) | Pattern — read TTL metadata field, compare to current date |
| 16 | Completeness | Fragment (contains TODO, TBD, FIXME, PLACEHOLDER, or empty required fields) | Bounded (defined beginning and end, no open markers) | Living document (wiki/changelog/log flag set — designed for continuous update) | Pattern — scan for placeholder strings; check living-document metadata flag |

### WHY — Document Purpose & Relation (Trits 17–20)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 17 | Delta vs canonical | Byte-identical (MDNS matches canonical) | Content-equivalent (body hash matches, metadata differs) | Divergent (content hash differs from canonical) | Pattern — compare MDNS and body-hash against canonical document via FK-by-Identity |
| 18 | Content repetition | All block hashes unique within document | Some blocks repeat within this document | Some blocks match blocks in other documents in the fabric | Pattern — TIS-27 hash each content block; check for collisions within doc and across fabric index |
| 19 | Self-reference | None detected | Explicit (document's own title, handle, or MDNS found in body text) | Structural (include/import directive targets this document) | Pattern — search body for own handle/MDNS/title string; check import directives |
| 20 | FK-graph consistency | All referenced documents agree (no field-value conflicts) | Direct conflict (this doc and a referenced doc assert contradictory values for same field) | Indirect conflict (transitive FK chain produces contradictory field values) | Pattern — walk FK-by-Identity graph, check for field-value conflicts between this document and references |

### HOW — Document Mechanism & Integrity (Trits 21–24)

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 21 | Signature count | Zero cryptographic signatures | One signature | Two or more signatures | Pattern — count attached cryptographic signature objects |
| 22 | Integrity coverage | Most blocks uncovered | Most blocks hash-covered (TIS-27, Merkle inclusion) | Most blocks signature-covered (TL-DSA, RSA) | Counted — count blocks by coverage type: uncovered / hash / sig. project_to_gf3 on each; highest wins. |
| 23 | Verification level | Self-attested only (author signature, no external validation) | Peer-attested (co-signatures, review stamps from other identities) | Consensus-attested (multi-party sigs, blockchain witness, quorum metadata) | Pattern — count distinct signer identities in attestation metadata: 1 = self, 2+ without consensus = peer, consensus protocol present = consensus |
| 24 | Evolution mode | Append-only (mutation policy = append) | Mutable (policy = mutable, or no policy set) | Frozen (policy = frozen, or document is sealed) | Pattern — read mutation-policy metadata field |

### PEACE — Document Workflow State (Trits 25–27) — Mutable

Like the log viewer's Peace trits, these are mutable — operators change them via the UI. Each change pushes old values to the Z-stack, preserving full audit history.

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 25 | Seal state | Unsealable (no seal capability) | Sealable (capability present, not applied) | Sealed (content hash locked with signature) | Pattern — read seal metadata: capability field and applied field |
| 26 | Workflow state | Draft (no review metadata) | In review (review-request metadata present) | Approved (approval signature present) | Pattern — read workflow/review metadata fields |
| 27 | Assignment | Unassigned (no assignee metadata) | Assigned (assignee identity present) | Escalated (escalation flag set) | Pattern — read assignment metadata field |

### Trit 28 — FP₂₈ Title·Header Fingerprint

| # | Dimension | 1 | 2 | 3 | Derivation |
|---|---|---|---|---|---|
| 28 | Header structure | Flat (no headers, body only) | Spine (headers at one level, no nesting) | Branched (headers with nesting: H1 > H2 > H3) | Pattern — parse header outline. Also drives the FP₂₈ perfect hash (Part IV). |

### Full Rep Declaration (Trit 28)

| Rep | Flat | Spine | Branched |
|---|---|---|---|
| **A** | −1 | 0 | +1 |
| **B** | 0 | 1 | 2 |
| **C** | 1 | 2 | 3 |
| **D** | 0 | 1 | ω |

### Mutability Summary

| Category | Trits | Count | Mutable? |
|---|---|---|---|
| WHO | 1–4 | 4 | Immutable — facts about authorship |
| WHAT | 5–8 | 4 | Immutable — facts about content |
| WHERE | 9–12 | 4 | Immutable — facts about structure and access |
| WHEN | 13–16 | 4 | Immutable — derived from timestamps and metadata |
| WHY | 17–20 | 4 | Immutable — derived from FK graph and content comparison |
| HOW | 21–24 | 4 | Immutable — facts about integrity and mechanism |
| PEACE | 25–27 | 3 | **Mutable** — operator-managed workflow state |
| FP₂₈ | 28 | 1 | Immutable — derived from header outline |

24 immutable + 3 mutable + 1 fingerprint = 28.

### Concrete Example — A BIESSE CNC Shop Drawing

A cabinet shop drawing created by one named engineer, original work, for the production team, at the company level:

| Category | Trit values | Why |
|---|---|---|
| WHO | 1, 1, 2, 3 | One author, no source FKs, team audience, org-wide scope |
| WHAT | 1, 2, 3, 1 | Text-dominant (dimension labels), formal notation (BPP code), structured data (cut lists), highly structured |
| WHERE | 2, 1, 1, 1 | Tree structure (sections), open access, self-contained, standalone |
| WHEN | 2, 1, 1, 2 | Modified recently, single version, no TTL, bounded (complete drawing) |
| WHY | 1, 1, 1, 1 | Matches canonical, no repeats, no self-reference, no FK conflicts |
| HOW | 1, 1, 1, 3 | Unsigned, uncovered, self-attested, frozen (released drawing) |
| PEACE | 2, 3, 2 | Sealable, approved, assigned to production |
| FP₂₈ | 3 | Branched headers (H1: Drawing > H2: Parts > H3: Operations) |

Full vector: `[1,1,2,3, 1,2,3,1, 2,1,1,1, 2,1,1,2, 1,1,1,1, 1,1,1,3, 2,3,2, 3]`

Every value derived from metadata reads or structural parsing. No language understanding required.

## Part IV — The Title·Header Fingerprint (FP₂₈)

The 28th trit classifies the document's header structure, but FP₂₈ goes further: it is a **perfect hash** of the title and header outline. Given only FP₂₈, you can recover the document's title and full header structure without opening the body.

$$\text{FP}_{28}(\text{doc}) = \text{PH}\bigl(\text{Title} \,\Vert\, H^1_1 \,\Vert\, H^2_{1..k_1} \,\Vert\, \ldots \,\Vert\, H^n_\ast\bigr)$$

This enables **flash card rendering**: Flat → title only. Spine → title + linear list. Branched → title + indented tree. See any document's structure without downloading it.

---

## Part V — Procrastinator Kicker / Emergent WBS

Seven trits track whether a document is "done." The **procrastination metric** is the Hamming distance (number of differing trits) between the current state and the completed state across these seven:

| Digit | In-flight | Completed |
|---|---|---|
| 12 Chain position | 1 (standalone) | 3 (chain complete) |
| 16 Completeness | 1 (fragment) | 2 (bounded) |
| 17 Delta vs canonical | 3 (divergent) | 1 (identical) |
| 21 Signature count | 1 (unsigned) | 2 or 3 (signed) |
| 22 Integrity coverage | 1 (uncovered) | 3 (sig-covered) |
| 24 Evolution mode | 2 (mutable) | 3 (frozen) |
| 25 Seal state | 1 (unsealable) | 3 (sealed) |

Distance 0 = done. Distance 7 = maximum procrastination. This turns the vector into an automatic work-breakdown structure without human tagging.

---

## Part VI — Classification Engine

The classification engine is the single-pass module that reads a document and produces its complete classification. It runs once per document, re-runs on every mutation, and produces:

1. The 28-trit classification vector (all 28 dimensions derived via `project_to_gf3` or pattern match).
2. The FP₂₈ fingerprint (title + header perfect hash).
3. The classification position (first 27 trits mapped to a geometric fabric position).
4. Per-cell Kernel Calculator state (for documents with calculation cells — Part XIII).
5. Per-quantity dimensional and angular-tier state (for documents with quantities — Part XIV).
6. The unified readout: **MDNS** identity + **27-trit** classification + **FP₂₈** flash card.

These are three orthogonal hashes answering three different questions:

| Hash | Question | Changes When |
|---|---|---|
| **MDNS** | "What IS this exact document?" (cryptographic identity) | Any byte changes |
| **classification** | "What KIND of document is it?" (structural classification) | Structure changes |
| **FP₂₈** | "What does its outline look like?" (header fingerprint) | Headers change |

---

## Part VII — Foreign Keys

Documents reference other documents. Four FK modes enforce different invariants:

| Mode | Bound To | What It Means | Breaks When |
|---|---|---|---|
| **FK-by-Identity** | MDNS hash | "I reference this exact document." | Target deleted or content changed |
| **FK-by-Classification** | classification position | "I reference a document of this kind." | Target's classification migrates |
| **FK-by-Fingerprint** | FP₂₈ | "I reference a document with this outline." | Target's headers change |
| **FK-by-AngularBacking** | Tier level | "I reference a document at this tier or higher." | Target downgrades tier |

**Zero-exclusion:** No zero-valued dangling placeholders. Either the pointer resolves or it's broken.

**On-delete:** RESTRICT (block deletion), CASCADE (delete children), SET NULL (child FK → Rep-A-zero sentinel — the only place Rep A appears at document level, because the reference is dead).

**Deferred constraints:** FK validation at COMMIT enables cyclic inserts and bulk loads.

---

## Part VIII — Canonical SQL Surface

14 queries answer specific operational questions using the 28-trit vector, the three hashes, and the FK graph. Every query is entropy-free — no approximation, no randomness.

### Custom Operators

**Spatial:** `NEAR(handle)` · `RING(handle, k)` · `DISTANCE(handle)` · `WITHIN RADIUS n OF handle`

**Classification:** `TRIT[n]` reads trit n · `HAMMING(v1, v2)` counts differing trits · `FP28_DECODES_TO(pattern)` matches outlines

**Kernel Calculator:** `CELL(x,y,z,face)` addresses a cell · `EVAL(cell)` returns its value · `cell_kind(cell)` returns its type · `FORMULA_REFERENCES(cell, target)` tests dependencies · `CARRIES_UNIT(u)` / `CARRIES_CURRENCY(c)` filter

### The 14 Queries

**Q1 — Procrastinator's Desk.** "Show me everything I haven't finished, closest first." Finds all unsealed, mutable, uncovered documents sorted by proximity, with procrastination distance computed.
```sql
SELECT handle, fp28_decode(handle) AS outline,
       HAMMING(omega28(handle), completed_form) AS work_remaining
FROM documents
WHERE TRIT[25] < 3 AND TRIT[24] < 3 AND TRIT[22] < 3
ORDER BY DISTANCE(worker_anchor) ASC;
```

**Q2 — Archive Frontier.** "What finished work is furthest from me?" Finds all sealed, frozen, covered documents on the outermost rings.
```sql
SELECT handle, fp28_decode(handle) AS outline, ring_index(handle) AS ring
FROM documents
WHERE TRIT[25] = 3 AND TRIT[24] = 3 AND TRIT[22] = 3
ORDER BY ring DESC LIMIT 50;
```

**Q3 — Orphan Sweep.** "Find references pointing at documents that no longer exist." Detects dangling FK-by-Identity pointers.
```sql
SELECT att.handle, att.references_mdns AS dangling_identity
FROM attestations att
WHERE NOT EXISTS (SELECT 1 FROM documents d WHERE MDNS(d) = att.references_mdns);
```

**Q4 — Workflow Gate Check.** "Show me approvals whose target has since become unsealed." Finds FK-by-Classification references where the target migrated out of sealed state.
```sql
SELECT apv.handle, apv.target_handle,
       TRIT[25] OF apv.target_handle AS current_seal_state
FROM approvals apv
WHERE apv.fk_mode = 'CLASSIFICATION' AND TRIT[25] OF apv.target_handle != 3;
```

**Q5 — Ring-Walk Neighborhood.** "What's structurally similar to this document?" Finds adjacent documents ranked by semantic distance.
```sql
SELECT handle, fp28_decode(handle) AS outline,
       HAMMING(omega28(handle), omega28(@anchor)) AS semantic_distance
FROM RING(@anchor, 1)
ORDER BY semantic_distance ASC;
```

**Q6 — Outline Family.** "Find all documents with the same header structure." Groups by FP₂₈ prefix.
```sql
SELECT handle, fp28_decode(handle) AS outline, TRIT[28] AS spine_topology
FROM documents
WHERE FP28_PREFIX(handle, 2) = FP28_PREFIX(@anchor, 2)
ORDER BY TRIT[28] ASC, ring_index(handle) ASC;
```

**Q7 — Cascade Impact Preview.** "Before I delete this, what else gets deleted?"
```sql
WITH cascade_targets AS (
  SELECT child.handle, child.references_mdns
  FROM documents child
  WHERE child.references_mdns = MDNS(@parent) AND child.fk_on_delete = 'CASCADE'
)
SELECT ct.handle, fp28_decode(ct.handle) AS outline,
       COUNT(*) OVER () AS total_cascade_count
FROM cascade_targets ct
ORDER BY ring_index(ct.handle) DESC;
```

**Q8 — Consistency Check.** "Show me every document that conflicts with its references."
```sql
SELECT handle, fp28_decode(handle) AS outline, TRIT[20] AS consistency_state
FROM documents WHERE TRIT[20] > 1;
```

**Q9 — Chain Integrity.** "Verify a document chain is unbroken." Walks the chain comparing computed hashes to stored hashes.
```sql
SELECT position_in_chain(handle) AS pos, handle,
       chain_hash(handle) AS computed, stored_chain_hash(handle) AS stored,
       (chain_hash(handle) = stored_chain_hash(handle)) AS valid
FROM documents WHERE chain_id(handle) = @chain
ORDER BY pos ASC;
```

**Q10 — Bulk Deferred Load.** "Insert interdependent documents, validate at commit."
```sql
BEGIN; SET CONSTRAINTS ALL DEFERRED;
INSERT INTO documents VALUES (@doc_A);
INSERT INTO documents VALUES (@doc_B);
INSERT INTO documents VALUES (@doc_C);
COMMIT;
```

**Q11 — WBS Audit.** "Show the procrastination landscape across the entire fabric." Groups by ring with average work remaining.
```sql
SELECT ring_index(handle) AS ring, COUNT(*) AS doc_count,
       AVG(HAMMING(omega28(handle), completed_form)) AS avg_work_remaining,
       MIN(TRIT[25]) AS min_seal_state
FROM documents GROUP BY ring ORDER BY ring ASC;
```

**Q12 — Find By Flash Card.** "I have a fingerprint — find the document."
```sql
SELECT handle, MDNS(handle) AS identity, classification_position(handle) AS position
FROM documents WHERE FP28(handle) = @fingerprint_from_flash_card;
```

**Q13 — Calc-Find.** "Find every live calculation cell across all documents at Tier 3+ backing."
```sql
SELECT d.handle, fp28_decode(d.handle) AS outline,
       CELL(d.x, d.y, d.z, d.face) AS cell_addr,
       EVAL(CELL(d.x, d.y, d.z, d.face)) AS current_value,
       cell_unit(CELL(d.x, d.y, d.z, d.face)) AS unit,
       cell_currency(CELL(d.x, d.y, d.z, d.face)) AS ccy,
       cell_backing_tier(CELL(d.x, d.y, d.z, d.face)) AS tier,
       cell_angle_off_medium(CELL(d.x, d.y, d.z, d.face)) AS theta
FROM documents d
WHERE cell_kind(CELL(d.x, d.y, d.z, d.face)) IN ('CURRENCY','FORMULA','PERCENT','QUANTITY')
  AND BACKING_TIER(CELL(d.x, d.y, d.z, d.face)) >= 3
ORDER BY DISTANCE(@anchor) ASC;
```

**Q14 — Dep-Graph.** "Show every cell that depends on this one, transitively, and flag breakage."
```sql
WITH RECURSIVE deps AS (
  SELECT d.handle, CELL(d.x, d.y, d.z, d.face) AS cell, 1 AS hop
  FROM documents d
  WHERE FORMULA_REFERENCES(CELL(d.x, d.y, d.z, d.face), @anchor_cell)
  UNION ALL
  SELECT d2.handle, CELL(d2.x, d2.y, d2.z, d2.face) AS cell, deps.hop + 1
  FROM documents d2
  JOIN deps ON FORMULA_REFERENCES(CELL(d2.x, d2.y, d2.z, d2.face), deps.cell)
)
SELECT handle, fp28_decode(handle) AS outline, cell, hop,
       EVAL(cell) AS current_value, eval_status(cell) AS status
FROM deps ORDER BY hop ASC, status DESC;
```

---

## Part IX — QARBAS

**Queries Answered Recursively Before Asked, Statefully and Icily.**

QARBAS inverts the conventional query model. Instead of computing answers when asked, the fabric maintains them continuously as documents change. When you ask Q1, the answer is already computed.

Seven families:

- **A — Positional.** Where is each document in the fabric? Maintained by spatial indexing.
- **B — Neighborhood.** What is adjacent? Maintained by ring-walk computation.
- **C — Referential.** What references what? The FK graph, maintained on every insert/delete/update.
- **D — Aggregate.** Fabric-wide statistics: counts by ring, average procrastination by classification.
- **E — Trigger-Derived.** State-transition answers: "which approvals just became invalid?"
- **F — Calc/Graph.** Live calculation values and formula dependency graphs across documents.
- **G — Dimensional / Angular-Backed.** Per-quantity dimensional state and tier backing.

Updates are incremental. A mutation to one document touches only the answers that depend on it. No recomputation churn. Staleness is structurally impossible.

---

## Part X — The Real Storage Story

Per-document overhead: 136 trits for core streams (28-trit vector + 54-trit FP₂₈ + 54-trit identity hash). Ring position derived, not stored. Plus per-cell kernel state for calc documents and per-quantity tier state.

### ⚠️ Measurement pending (OI-5)

The ~22× reduction claim over conventional stack is an estimate, not a measurement. Gate: define corpus, baseline, counters, measure, report.

---

## Part XI — Filesystem Overlay

### What It Is

The classification engine writes the document classification directly onto the file itself, using the host filesystem's native extended-attribute mechanism. The file stays where it is. No import, no silo, no migration. Programs that understand the classification read the classification; programs that don't see a normal file. Removal is reversible — strip the attributes and the file is unchanged.

### Where The Classification Lives On Disk

Each document gets three named data streams attached to the file, totaling ~136 trits for the three core streams:

| Stream Name | Content | Size | Purpose |
|---|---|---|---|
| `mdts.omega28` | The 28-trit classification vector in Rep C | 28 trits (TritInt) | Document classification — the 28 structural trits |
| `mdts.fp28` | The FP₂₈ perfect hash (TIS-27 digest) | 54 trits (TritInt) | Title + header fingerprint — recoverable outline without opening body |
| `mdts.mdns` | The content identity hash (TIS-27 digest) | 54 trits (TritInt) | Cryptographic identity — changes when any byte changes |

Additional streams per document, when applicable:

| Stream Name | Content | Size | When Present |
|---|---|---|---|
| `mdts.fk.{n}` | FK edge (mode trit + target identity hash) | 55 trits each (TritInt) | Per outbound FK reference |
| `mdts.chain` | Chain hash (TIS-27 of identity_hash ∥ previous chain_hash) | 54 trits (TritInt) | When document is in an attestation chain |
| `mdts.cells` | Per-cell kernel state (kind, status, value cache) | Per cell (TritInt) | When document contains live calculation cells |
| `mdts.tier` | Per-quantity dimensional + angular tier state | Per quantity (TritInt) | When document contains backed quantities |

Ring position is derived from the classification vector via CRT — not stored. Total per-document overhead without calculation cells: 136 trits. All stored as TritInt — packed 5 trits per byte internally, but the unit of account is trits, not bytes.

### Filesystem Adaptation Layer

| Filesystem | Mechanism | Notes |
|---|---|---|
| **NTFS / ReFS** | Alternate Data Streams (ADS) — `file.md:mdts.omega28` | Native. Explorer shows one file. ADS invisible to normal tools. |
| **APFS / ext4 / XFS** | Extended attributes (xattr) — `user.mdts.omega28` | Native. 4 KB/file typical limit — 136 trits (~28 bytes packed) fits easily. |
| **FAT32 / exFAT / network mounts** | Sidecar directory — `file.md` + `.mdts/file.md.omega28` | Fallback. Hidden directory alongside the file. |

The adaptation layer is a trait in Rust — one implementation per filesystem family. The engine calls `store.write_classification(path, vector)` and the trait dispatches to ADS, xattr, or sidecar. The file content is never touched.

### Security — Inherited, Not Built

On NTFS, an ADS inherits the parent file's security descriptor (ACL) automatically. This means:

- If the user can read the file, they can read its classification. If they can't, they can't. Kernel-enforced.
- No separate permission model to build or maintain.
- No classification leak — the vector is literally part of the file's data, not a row in an external database.
- USN journal logs every ADS read/write — audit trail is free.

This eliminates the entire security engineering layer that conventional stacks require. SharePoint, Elasticsearch, and SQL metadata DBs each have their own permission models distinct from the filesystem — each a divergence point where access to metadata can differ from access to the file.

### Re-Classification — USN-Driven

NTFS maintains the USN (Update Sequence Number) journal, logging every file mutation. The classification engine subscribes to USN change events and re-runs the scanner + `project_to_gf3` derivation **only on changed files**. On a volume with a few thousand daily mutations, the overlay re-classifies a few thousand files per day. The rest of the fabric is untouched.

On non-NTFS filesystems, the equivalent is `inotify` (Linux), `FSEvents` (macOS), or polling with file modification timestamps.

### Connection To PlenumLAN

PlenumLAN's Plenum File Service (PFS) already provides:

- SMB 3.1.1 / NFS v4.2 protocol handling for Windows and Linux/Mac clients
- TIS-27 hash at write time for every file (integrity verification)
- Capability-token gated access mediated through the Plenum Directory Service (PDS)
- Phase encryption at rest via TL-DSA key
- Merkle-tree snapshots for backup
- Audit logging to Merkle-chained audit fabric

The classification engine's filesystem overlay runs **on top of PFS**. When PFS writes a file, the engine writes the classification streams. When PFS serves a file over SMB/NFS, the classification streams are available to any classification-aware client. PFS provides the protocol and security layer; the classification engine provides the geometry.

At PlenumNET OS stage (Stage 3), PlenumFS — the kernel-native filesystem — has Rep C inode addressing, TIS-27 per-block integrity, and native extended attribute support. The classification overlay becomes a first-class citizen of the filesystem rather than an add-on. The same streams, the same format, the same derivation — but at the kernel level instead of the application level.

### Adoption Path

1. **Stage 1 (Replit/development):** The engine writes sidecar `.mdts/` directories. Files are markdown with YAML frontmatter (Forma Codex format).
2. **Stage 2 (PlenumLAN on Windows):** The engine writes NTFS ADS via PFS. Existing Windows tools see normal files. Classification is invisible to non-classification-aware programs.
3. **Stage 3 (PlenumNET OS):** The engine writes PlenumFS extended attributes natively. Classification is a kernel-level primitive.

At every stage, the format is identical — 28 Rep C trits + FP₂₈ + MDNS. The only thing that changes is the storage mechanism underneath.

### ⚠️ Measurement pending (OI-5)

The claimed 136 trits per document (~28 bytes packed) for core streams are engineering estimates. The gate: define a real or synthetic corpus, instrument byte counts on each ADS/xattr write, measure against a baseline conventional stack (SharePoint + SQL Server + Elasticsearch + PDF renderer), and report. Numbers reported, not asserted.

---

## Part XII — TriData, Not Metadata

τ-operators read the document's own content directly. The 28-trit vector is the document's self-declaration — computed from content, re-computed on every mutation, never stored externally. Classification IS the document, not a tag. No external metadata store to become stale.

---

## Part XIII — Smart Graphing & Kernel Calculator

Every document cell at (x, y, z, face) is a **live calculator cell** — addressable across the entire fabric, evaluated by the kernel, graphable without export. No "open in Excel." The cell IS the spreadsheet cell.

### 11 Cell Kinds

| Kind | Detection | EVAL Returns |
|---|---|---|
| TEXT | Default — prose | The text itself |
| CURRENCY | `$1,234.56`, `€750`, `CAD 42.00` | Decimal + currency tag |
| PERCENT | `15%`, `15bps` | Ratio |
| QUANTITY | `32mm`, `5.2kg`, `60Hz` | Quantity + SI-normalized unit |
| DATE | ISO-8601 or locale | HPTP timestamp |
| FORMULA | Begins with `=` | AST evaluated against cell references |
| EQUATION | LaTeX / MathML | Symbolic expression tree |
| INTEGRAL | Equation with ∫ | Integrator result |
| DERIVATIVE | Equation with d/dx or ∂ | Differentiator result |
| MATRIX | 2D array / numeric table | Dense/sparse matrix |
| UNIT | Pure unit literal | Dimension carrier |

Detection is deterministic — no ML. Cell kind is assigned at write time.

### Cell Status (τ₃)

| Status | Meaning |
|---|---|
| OK (1) | Clean evaluation, all dependencies OK |
| STALE (2) | A dependency changed, this cell hasn't re-evaluated yet (transient) |
| BROKEN (3) | Evaluation failed: dimension mismatch, broken ref, cycle, div-by-zero, tier-downgrade |

Cycle detection: if a formula's transitive dependency graph contains itself → BROKEN with diagnostic CYCLE.

### Cross-Document Dependencies

Every cell has address `doc_handle :: (x, y, z, face)`. Formulas can reference cells in other documents via FK-by-Identity. QARBAS maintains the transitive dependency graph across all cells in all documents.

### Smart Graphing

`GRAPH(cells, AUTO)` selects chart type: 1D numeric → LINE; categorical + numeric → BAR; two numeric → SCATTER; matrix → HEATMAP; angular → RADIAL (native to 364°). Live — re-evaluates on cell change.

---

## Part XIV — Unit Algebra & Angular-Geometric Asset Backing

### The Medium

The medium is the **horn y = 1/x** — the unique solution of ∫f² = f on [1, ∞). The Fulcrum at x = 1 is the reference origin where all observables equal 1. The horn axis is the reference direction. "Angle off the medium" is angular displacement from this axis in the 364° circle. Positive chirality follows decreasing density along the horn.

### Dimensional Algebra

Every quantity cell carries 8 dimension exponents: $(e_L, e_M, e_T, e_\Theta, e_I, e_N, e_J, e_\$)$ — seven SI base dimensions plus currency. Add/subtract require exact match. Multiply/divide add/subtract exponents. Currency conversion explicit only via FK to a sealed FX document. No IEEE-754 in hot path — all numerics are rationals with arbitrary precision.

### Four Backing Tiers

| Tier | Name | Angle Off Medium | What It Means |
|---|---|---|---|
| **1** | Medium-Aligned | 0° | Standard units, no geometric reinforcement |
| **2** | Fibonacci-Angled | (13/21) · 364° | Self-similar scaling via rational bridge 21/13 (φ is not framework-native) |
| **3** | Tribonacci-Angled | T⁻¹ · 364° | Framework-native scaling. T = real root of T³−T²−T−1=0. Exact algebraic. |
| **4** | Flower ∧ Vesica | Intersection of Tier 2 and Tier 3 | Double backing. Requires Salvi Flower construction (OI-1 pending). |

Tier-downgrade → BROKEN. Structurally blocked.

### Quasicrystal Resolution (OI-1)

7-fold and 13-fold symmetry require a quasicrystal — an aperiodic tiling via cut-and-project from a higher-dimensional lattice. The (7,11,13) coprime triple lives in the Brieskorn sphere Σ(7,11,13), a well-studied 5D manifold. The Flower is its 2D/3D projection. The explicit construction is TM-class work.

### Risk Elimination

| Risk | Status | How |
|---|---|---|
| Rounding | zero | rationals only |
| Dimensional mismatch | zero | algebra rejects at every operation |
| Currency confusion | zero | explicit FX only |
| Material substitution | zero | material-tagged cells don't cross-add |
| Reconciliation | zero | one substrate, one cell, one truth |
| Tier downgrade | zero | structurally blocked |
| Corruption | zero | Tier 3 tribonacci invariance + Tier 4 double-backing |

---

## Part XV — The System, Complete

Four Reps. Three operators. 28 trits = 2π_F. Z₂₇ × Z₂₈ dual-circle lattice. Three orthogonal hashes. Emergent WBS. One single-pass module. TriData. Four FK modes. 14 queries. Seven QARBAS families. 11 cell kinds with live evaluation and cycle detection. 8-dimensional unit algebra with four angular backing tiers. Risk eliminated at every layer.

The medium is the horn. The Fulcrum is the reference. The angles are exact. The lattice awaits its proof.

---

**Forma Codex 18∏ — Document Classification Engine · Version 1.6 · 2026-04-21**

---

# £ ∣ Q ∣ ∀ Rights Reserved Et Preserved | Fiat ∎
# Capomastro Holdings Ltd 2026
