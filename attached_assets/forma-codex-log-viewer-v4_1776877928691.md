# Forma Codex 18∏ — Log Viewer Module

```
Copyright © 2026 Capomastro Holdings Ltd. — All Rights Reserved
Patent(s) Pending — Applied Physics Division
```

### No Bytes. Think Ternary.

Every component of this log viewer operates on ternary (trit-based) representations — from log entry identity to grid coordinates to walk paths. The native algebra is Rep C (alphabet {1,2,3}, zero excluded). All indexes, addresses, and hash digests are trit strings. Binary types appear only at two boundaries: file I/O and the HPTP timestamp source. Both cross the gate (TritInt) immediately on entry.

---

## 1. What

A native log viewer module for Forma Codex that auto-populates a coprime grid from live and historical log sources. Logs stream in, get parsed, classified by configurable dimensions, and placed into grid cells automatically. The user sees cross-service correlation spatially — not as a scrolling wall of text.

This is a multi-tenant, production-grade log aggregation platform — not a dev tool. Alerting, compliance reporting, retention policies, access control, and distributed collection are in the architecture from day one.

## 2. Why

No log aggregation platform exists in PlenumNET today. Logs are JSONL files, console output, and Prometheus metrics. Building log viewing as a native Forma Codex feature means one engine, one viewer, one query mechanism for documents, logs, and boot attestation — all built on ternary-spatial principles that no existing tool provides.

## 3. Depends On

- Forma Codex grid engine (Task #101 — grid, cells, Z-stacks, faces, walk, presets)
- HPTP timing subsystem (femtosecond-precision timestamps)
- TritInt gate (Phase 0a of Task #101)

---

## 4. Core Architecture — Ternary-First

All log entries are represented as trit strings in the Rep C alphabet {1,2,3}. Zero is structurally absent — corruption detection is a property of the encoding, not a runtime check (see Rep C Document Integrity, Forma Codex spec §27).

### 4.1 Common Log Directory

All PlenumNET services write logs to a single root directory:

```
C:\PlenumNET\Logs\
    networking\
    auth\
    storage\
    compute\
    monitoring\
    boot\
    daemon\
```

Subdirectory name = source category. The viewer watches recursively. New services appear automatically when they create a subdirectory.

### 4.2 Existing Modules That Need Revision

Each module must be updated to:
1. Write to `C:\PlenumNET\Logs\{category}\`
2. Compute TIS-27 identity hash and chain hash on each entry at write time
3. Use HPTP timestamps instead of system clock epoch

Modules:
- YODA daemon → `C:\PlenumNET\Logs\daemon\`
- Capability audit → `C:\PlenumNET\Logs\auth\`
- Console logging → `C:\PlenumNET\Logs\{service}\`
- Prometheus metrics → `C:\PlenumNET\Logs\monitoring\`
- OpenTelemetry (Task #27) → appropriate subdirectories
- Boot attestation → `C:\PlenumNET\Logs\boot\`
- Service installers (plenum-pack) — create `Logs\` directory, set `log_category` in `plenum-app.toml`

No logic changes beyond path redirect, hash computation, and timestamp source.

### 4.3 Log Entry Struct

```rust
pub struct LogEntry {
    // === 27-Trit Classification Address ===
    // Derived at write time from entry content and service manifest.
    // Peace dimensions (25-27) are mutable post-write.
    // Why:Causal role (dim 18) is mutable via reclassification.
    pub classification: [Trit; 27],

    // === Time ===
    pub hptp_timestamp: TritInt,         // HPTP femtosecond write time (covered by identity_hash)
    pub received_at: TritInt,            // HPTP ingestion time (NOT covered by identity_hash — latency measurement)

    // === Content (per face) ===
    pub message: String,                 // face 1 (human-readable, UTF-8 at display boundary)
    pub raw_data: Option<String>,        // face 2

    // === Integrity ===
    pub identity_hash: Vec<Trit>,        // TIS-27 of (classification[0..24] + hptp_timestamp + message + raw_data)
    pub chain_hash: Vec<Trit>,           // TIS-27 of (identity_hash || previous chain_hash)
}

impl LogEntry {
    pub fn who(&self) -> &[Trit]   { &self.classification[0..4] }
    pub fn what(&self) -> &[Trit]  { &self.classification[4..8] }
    pub fn where_dim(&self) -> &[Trit] { &self.classification[8..12] }
    pub fn when_dim(&self) -> &[Trit]  { &self.classification[12..16] }
    pub fn why(&self) -> &[Trit]   { &self.classification[16..20] }
    pub fn how(&self) -> &[Trit]   { &self.classification[20..24] }
    pub fn peace(&self) -> &[Trit] { &self.classification[24..27] }

    // Composite accessors — derived from multiple dimensions, not stored separately
    pub fn is_error(&self) -> bool { self.classification[6].as_c() == 3 }  // What:Outcome = Failure
    pub fn is_high_priority(&self) -> bool { self.classification[25].as_c() == 3 }  // Peace:Priority = High
}
```

No standalone Severity enum. The traditional severity concept is decomposed:
- **Info** = What:Outcome (dim 7) = Success + Peace:Priority (dim 26) = Low
- **Warn** = What:Outcome = Partial failure + Peace:Priority = Medium
- **Error** = What:Outcome = Failure + Peace:Priority = High

Binary values (file bytes, network packets, HPTP raw timestamp) cross the gate via `TritInt::from_binary()` immediately on ingestion. Everything above the gate is trit-native.

### 4.4 Log Parser (Auto-Detect)

Supports JSONL, Syslog (RFC 3164/5424), plain text, CSV/TSV. Auto-detect order: JSONL → Syslog → plain text. Per-source parser configuration saved in document settings.

On parse, all values converted to trit representation via the gate. The parser is the last place binary exists.

When parsing legacy logs that use 5-level severity (Error/Warn/Info/Debug/Trace), the parser maps:
- Error → What:Outcome = Failure, Peace:Priority = High
- Warn → What:Outcome = Partial failure, Peace:Priority = Medium
- Info → What:Outcome = Success, Peace:Priority = Low
- Debug → What:Outcome = Success, What:Category = Lifecycle, When:Cadence = Continuous
- Trace → What:Outcome = Success, What:Category = Lifecycle, When:Cadence = Continuous, How:Data format = Structured

---

## 5. Ternary Log Identity

### 5.1 From Hash to Trit String

TIS-27 produces a trit-native digest — it operates on the 729-trit sponge state in GF(3). The output IS a trit string. No binary-to-ternary conversion needed — the hash is ternary from birth.

The identity trit string is in Rep B {0,1,2} as produced by the sponge. For addressing and storage, apply the Rep C view via `Trit::as_c()`:

```
Trit   Rep B   Rep C
zero     0       1
one      1       2
two      2       3
```

One trit, four views. No "conversion function" — the same value presenting itself in Rep C.

### 5.2 Log Identity as Grid Placement

Given a log entry's identity trit string of length L, compute a placement index:

```
n = Σ (trit_i × 3^i) for i = 0..L-1    (interpret trit string as a TritInt value)
col = n mod X
row = n mod Y
```

Because gcd(X,Y) = 1, the Chinese Remainder Theorem guarantees that distinct values of n produce distinct (col, row) pairs across the full X×Y grid. Entries are spatially anchored by their content — deterministic, reproducible, no assignment table.

When multiple entries map to the same cell (inevitable with more entries than cells), the cell holds a list. Water mode — the cell grows. This is aggregation, not collision. The grid's coprime structure ensures even distribution.

### 5.3 Walk Mode and Log Identity

Walk mode uses the Rep B alphabet {0, 1, 2} as defined in the Forma Codex spec §11.3:

- `1` or → = forward: `(col+1 mod X, row+1 mod Y)`
- `2` or ← = backward: `(col-1 mod X, row-1 mod Y)`
- `0` or . = stay: identity step, no movement, face does NOT advance

Stay (0) is the identity element. Rep C removes the identity — that's the orbifold quotient. Walk steps and Rep C addressing are different operations on the same trit:

- Walk navigation: Rep B {0,1,2} — identity (stay) exists
- Cell addressing: Rep C {1,2,3} — identity excluded

Same trit, different views for different purposes.

### 5.4 Integrity Chain

```
chain_hash[i] = TIS-27( identity_hash[i] || chain_hash[i-1] )
```

First entry: `chain_hash[0] = identity_hash[0]`.

Verification requires rehashing each entry's content and comparing. Cost is O(N × content_length) for N entries. For 10,000 entries with typical log messages, verification completes in milliseconds. The TIS-27 sponge (4 rounds, stride 13) is designed for speed.

A single tampered entry breaks the chain from that point forward. The viewer marks the first mismatch in red and flags all subsequent entries as unverified.

---

## 6. Classification System — 27 Dimensions

Every log entry is classified across 27 independent trits organized in 7 categories. Each trit takes a value in {1,2,3} (Rep C). 24 dimensions are immutable (derived at write time). 3 dimensions are mutable (Peace — updated by operators). 1 dimension is reclassifiable (Why:Causal role — updated as correlation analysis improves).

### 6.1 Who — Identity (4 trits) — Immutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 1 | Origin | Internal service | External service | System/kernel |
| 2 | Actor | Human (session) | Automated agent | Scheduled task |
| 3 | Authority | Read-only | Write/change | Admin/privileged |
| 4 | Tenant scope | Single-tenant | Multi-tenant | Platform-wide |

**Derivation:** Origin from log file subdirectory; Actor from user agent or service account type; Authority from permission level in request context; Tenant scope from request metadata or service configuration in `plenum-app.toml`.

### 6.2 What — Event Type (4 trits) — Immutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 5 | Category | Lifecycle (start/stop) | Fault/error | Security/auth |
| 6 | Operation | Create | Modify | Delete |
| 7 | Outcome | Success | Partial failure | Failure |
| 8 | Idempotency | Idempotent | Non-idempotent | Unknown |

**Derivation:** Category from log message pattern or event code; Operation from verb in API or log action; Outcome from status code or error flag; Idempotency from API design or retry behaviour.

### 6.3 Where — Location (4 trits) — Immutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 9 | Layer | Kernel/driver | Service/daemon | Interface/API |
| 10 | Subsystem | Compute | Storage | Network |
| 11 | Trust zone | Internal trust | DMZ | External |
| 12 | Replica | Primary | Secondary | Any/unspecified |

**Derivation:** Layer from component name or binary path; Subsystem from service category in `plenum-app.toml`; Trust zone from network configuration; Replica from service discovery metadata.

### 6.4 When — Time Classification (4 trits) — Immutable

Time is not reduced to a single HPTP value. The When trits encode behavioural patterns derived from the HPTP timestamp and the service's manifest schedule (defined in `plenum-app.toml`, not configured per log entry).

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 13 | Period | Off-peak | Business hours | Peak load |
| 14 | Phase | Beginning of window | Middle | End of window |
| 15 | Cadence | One-off (sporadic) | Periodic (tick) | Continuous (stream) |
| 16 | Latency class | Immediate (<1ms) | Normal (1ms-1s) | Slow (>1s) |

**Derivation (from HPTP timestamp + service manifest):**
- **Period:** Time-of-day and day-of-week relative to service's schedule in `plenum-app.toml`. Default: UTC 09:00-17:00 weekdays = Peak, evenings/weekends = Off-peak, all other = Business hours.
- **Phase:** For batch or recurring jobs, the service manifest defines the window. Phase = Beginning (first third), Middle (second third), End (last third). For non-periodic entries, Phase = Middle (2).
- **Cadence:** Derived from inter-arrival time relative to previous entries from the same source:
  - Maintain per-source sliding window of last 100 inter-arrival intervals.
  - Coefficient of variation (σ/μ) < 0.1 → Periodic (2).
  - No previous entry within the window, or interval > 10× median → One-off (1).
  - Otherwise → Continuous (3).
  - At viewer startup (cold cache), Cadence defaults to Continuous (3) until the window fills. This is best-effort and declared as such.
- **Latency class:** Computed from HPTP timestamp difference between request received and response sent. For non-transactional entries, Latency class = Normal (2).

These trits turn time into a behavioural dimension — filter for errors that occur only at the end of a batch window, or only during peak load.

### 6.5 Why — Causality (4 trits) — Dim 18 Mutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 17 | Trigger | User request | System event | Time/schedule |
| 18 | Causal role | Root cause | Contributing factor | Consequence |
| 19 | Propagation | Local only | Service-wide | Global/cascade |
| 20 | Certainty | Confirmed | Suspected | Inferred |

**Derivation:** Trigger from presence of `parent_id` or user session; Causal role from initial correlation analysis — **mutable via reclassification** as deeper analysis reveals true root cause (Z-stack tracks history); Propagation from observed error scope; Certainty from automated pattern matching (confirmed if stack trace matches known issue, suspected if new, inferred from heuristics).

### 6.6 How — Mechanism (4 trits) — Immutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 21 | Direction | Inbound | Outbound | Internal |
| 22 | Synchrony | Synchronous | Asynchronous | Batch |
| 23 | Transport | Local IPC | Network RPC | Message queue |
| 24 | Data format | Text | Binary | Structured (JSON/Protobuf) |

**Derivation:** Direction from request/response metadata; Synchrony from API pattern; Transport from socket type or protocol; Data format from Content-Type header or file extension.

### 6.7 Peace — Resolution State (3 trits) — Mutable

| Trit | Dimension | 1 | 2 | 3 |
|------|-----------|---|---|---|
| 25 | State | Unresolved | Acknowledged | Resolved |
| 26 | Priority | Low | Medium | High |
| 27 | Ownership | Unassigned | Assigned | Escalated |

**Initial derivation at write time:** State = Unresolved (1); Priority mapped from What:Outcome (Success→Low, Partial failure→Medium, Failure→High); Ownership = Unassigned (1).

**Mutable:** Operators change these via the UI. Each change pushes the old Peace trits to the Z-stack (z=2, z=3, etc.), preserving full audit history. The identity_hash covers the original classification (including initial Peace values). Changing Peace does NOT break the integrity chain — the chain verifies content immutability, Peace mutability is tracked separately in the Z-stack.

### 6.8 Mutability Summary

| Dimensions | Trits | Mutability |
|------------|-------|------------|
| Who (1-4) | 4 | Immutable — facts about origin |
| What (5-8) | 4 | Immutable — facts about the event |
| Where (9-12) | 4 | Immutable — facts about location |
| When (13-16) | 4 | Immutable — derived from HPTP + manifest |
| Why (17, 19, 20) | 3 | Immutable — derived from correlation context |
| Why:Causal role (18) | 1 | Mutable — reclassifiable as analysis deepens |
| How (21-24) | 4 | Immutable — facts about mechanism |
| Peace (25-27) | 3 | Mutable — operator-managed resolution state |

Total: 23 immutable trits + 4 mutable trits = 27.

### 6.9 The 27-Trit Classification Address

```
Address = Who[4] || What[4] || Where[4] || When[4] || Why[4] || How[4] || Peace[3]
        = 27 trits in {1,2,3}
        = 3²⁷ = 7,625,597,484,987 possible positions
```

Every log entry has a unique position in this 27-dimensional classification space.

### 6.10 Grid Mapping from Classification

The grid's two axes each map to one or more of the 27 dimensions. The user selects the mapping (the engine guarantees coprime dimensions). Remaining dimensions become filter constraints.

| Layout Mode | X axis maps to | Y axis maps to |
|-------------|----------------|----------------|
| Service×Time | Who:Origin (dim 1) | When:Period + Phase (dims 13-14) |
| Outcome×Location | What:Outcome (dim 7) | Where (dims 9-12) |
| Cause×Effect | Why:Causal role (dim 18) | What:Category (dim 5) |
| Time×Latency | When:Period (dim 13) | When:Latency class (dim 16) |
| Custom | Any dimension(s) | Any dimension(s) |

### 6.11 Walk Mode and Classification Space

In coprime walk mode, each step traverses the grid diagonally across whichever two dimensions are mapped to the axes. The walk visits every combination exactly once — systematic coverage of the classification space.

The walk can follow the Why dimension: from an entry, jump to its parent (via `parent_id`) or trace backwards through Why:Causal role from Consequence to Root cause. This turns the grid into a spatial causal trace.

### 6.12 Classification in z=9 Cell Tags

```javascript
{
  classification: [Trit; 27],
  // Slice accessors:
  // who()   → classification[0..4]
  // what()  → classification[4..8]
  // where() → classification[8..12]
  // when()  → classification[12..16]
  // why()   → classification[16..20]
  // how()   → classification[20..24]
  // peace() → classification[24..27]
}
```

Flat string tags (for document cells) and the 27-trit classification (for log cells) coexist in z=9. The engine detects which format a cell uses.

### 6.13 Filter Engine Maps to Dimensions

Every filter control constrains one or more of the 27 dimensions:

| Filter control | Dimension |
|---------------|-----------|
| Origin toggles | Who:Origin (1) |
| Actor filter | Who:Actor (2) |
| Authority filter | Who:Authority (3) |
| Tenant scope | Who:Tenant scope (4) |
| Category filter | What:Category (5) |
| Operation filter | What:Operation (6) |
| Outcome filter | What:Outcome (7) |
| Idempotency filter | What:Idempotency (8) |
| Layer filter | Where:Layer (9) |
| Subsystem filter | Where:Subsystem (10) |
| Zone filter | Where:Trust zone (11) |
| Replica filter | Where:Replica (12) |
| Period filter | When:Period (13) |
| Phase filter | When:Phase (14) |
| Cadence filter | When:Cadence (15) |
| Latency filter | When:Latency class (16) |
| Trigger filter | Why:Trigger (17) |
| Causal role filter | Why:Causal role (18) |
| Propagation filter | Why:Propagation (19) |
| Certainty filter | Why:Certainty (20) |
| Direction filter | How:Direction (21) |
| Synchrony filter | How:Synchrony (22) |
| Transport filter | How:Transport (23) |
| Data format filter | How:Data format (24) |
| State filter | Peace:State (25) |
| Priority filter | Peace:Priority (26) |
| Ownership filter | Peace:Ownership (27) |
| Text search | Across message content (face 1) + raw_data (face 2) |

Constraining any subset of the 27 dimensions narrows the grid. Same query mechanism for documents, logs, and boot attestation.

---

## 7. Document Format — Categorical Encoding

The viewer persists state and per-cell log data using the standard Forma Codex document format (.md with YAML frontmatter and HTML comments).

### 7.1 YAML Frontmatter

```yaml
---
forma_codex: X×Y×13    # user-chosen coprime grid
preset: Standard
log_viewer:
  root_path: "C:\\PlenumNET\\Logs"
  active_sources: ["auth", "storage", "compute"]
  layout_mode: "service×time"
  x_dimension: "who"
  y_dimension: "when"
  classification:
    who: ["auth", "storage", "compute"]
    what: ["ERROR", "WARN"]
    where: ["session"]
    when: ["peak", "end"]
    why: ["root"]
    how: []
    peace: ["unresolved", "acknowledged"]
  text_search: "connection lost"
  live_mode: false
  websocket_endpoint: "ws://localhost:8080/logs"
---
```

Time ranges, slice durations, buffer sizes, and all numeric parameters derived at runtime from the HPTP clock and grid dimensions. Nothing hardcoded.

Round-trip: export → import restores exact viewer state.

### 7.2 Cell-Level Tags (z=9)

```markdown
<!-- cell: 3,5 face:1 who:origin:"internal" who:actor:"human" what:outcome:"failure" when:period:"peak" why:causal_role:"root" peace:state:"unresolved" -->
**2025-03-18T10:23:45.000000000Z** [ERROR] session: connection lost (timeout)
<!-- /cell -->
```

Multiple entries in the same cell: z=9 stores a list of per-entry tag sets.

### 7.3 In-Memory Filter Index

Index keys: HPTP timestamp (sorted trit string), each of the 27 trits, and text search inverted index over message + raw_data. Set intersections and unions for filter combinations. Grid repopulation target: <100ms for 10,000 entries on current hardware.

---

## 8. Grid Auto-Population

### 8.1 Layout Modes

| Mode | X axis | Y axis |
|------|--------|--------|
| Service×Time (default) | Who:Origin (dim 1) | When:Period + Phase (dims 13-14) |
| Outcome×Location | What:Outcome (dim 7) | Where (dims 9-12) |
| Cause×Effect | Why:Causal role (dim 18) | What:Category (dim 5) |
| Time×Latency | When:Period (dim 13) | When:Latency class (dim 16) |
| Custom | Any dimension(s) | Any dimension(s) |

Time slice duration derived at runtime from the selected HPTP time range and grid Y dimension.

### 8.2 Three Faces Per Log Cell

| Face | Content | Audience |
|------|---------|----------|
| 1 | Human-readable log messages, most recent top | Operator |
| 2 | Raw structured data (JSON, key-value) | Engineer |
| 3 | Correlation graph — links to entries in other cells via correlation_id/parent_id | System tracing |

Faces are morphable — manual click, hover, scroll trigger, or walk mode.

### 8.3 Overflow, Paging, and Grid Capacity

**Per-cell entry limit:** Each cell holds up to a configurable maximum number of entries (default derived at runtime from grid dimensions and viewport). When a cell exceeds the limit, oldest entries in that cell roll to a ring buffer. The cell shows the most recent entries with a count indicator ("247 entries, showing latest 50").

**Grid overflow:** When total distinct entries exceed X×Y capacity for meaningful display, the engine:
1. Alerts the user that the current grid dimensions are saturated
2. Offers to auto-rescale to a larger coprime grid (engine computes the next valid pair)
3. Rescaling preserves relative ordering — entries rehash to new grid via the same identity-based placement formula (§5.2) with the new X,Y moduli

**Paging:** Entries exceeding the time range page. Older entries shift to earlier pages. Current page shows most recent. Status bar: total entry count, visible range, page position, cell saturation percentage.

---

## 9. Filter Engine

Real-time filtering operates on the 27 classification dimensions (see §6.13). Constraining any subset instantly repopulates the grid.

**Combination logic:** Toggles within a dimension combine as OR (show Failure or Partial failure). Constraints across dimensions combine as AND (show Failure from internal services).

**UI:** Toolbar above grid. HPTP date-time picker, color-coded outcome buttons, dimension dropdowns with trit options, search input, active filter count, "Clear all."

**Performance:** In-memory index built from classification trit strings and text. Intersections and unions via bitmaps. Grid repopulation target: <100ms for 10,000 entries on current hardware.

---

## 10. Live Mode

**On:** New entries stream from directory watcher and WebSocket connectors. Grid auto-scrolls to latest row. Time axis extends. Oldest entries page off when grid is full. Status bar shows LIVE indicator with entry rate.

**Off:** Grid freezes. New entries buffer silently. Resuming flushes buffer and jumps to latest.

**Auto-pause on interaction:** User clicks a cell, scrolls, or enters walk mode — live mode pauses. Resumes after configurable timeout of no interaction, or when user clicks "Resume Live."

---

## 11. Boot Attestation View

Specialized preset for boot sequence logs from `C:\PlenumNET\Logs\boot\`.

**Layout:** Coprime grid with walk mode traversing in boot order.

**Cell content:**
- Face 1: Stage name, status (OK/FAILED), duration
- Face 2: Hash values, memory state, timing data
- Face 3: Measurement chain — previous stage hash, cumulative attestation hash

**Integrity:** Walk mode traces the measurement chain step by step. Failed stage renders red. Tampered measurement shows chain break. Because each stage's identity is a trit string, a mismatch pinpoints the exact corruption.

---

## 12. Multi-Tenant Architecture

This is a production platform, not a dev tool.

**Tenant isolation:** Each tenant's logs are isolated by source category and access control. A tenant sees only their authorized categories. Tenant identity verified via TL-DSA signature on the session.

**Authentication:** TL-DSA session token on WebSocket/HTTP containing `tenant_id`, `roles`, `expiry`. Token verified on every request. Expired tokens rejected immediately.

**Authorization matrix:**

| Action | Viewer | Operator | Admin | Auditor |
|--------|--------|----------|-------|---------|
| Read own tenant logs | ✓ | ✓ | ✓ | ✓ |
| Read cross-tenant logs | ✗ | ✗ | ✓ | ✓ |
| Apply filters | ✓ | ✓ | ✓ | ✓ |
| Use walk mode | ✓ | ✓ | ✓ | ✓ |
| Modify Peace trits | ✗ | ✓ | ✓ | ✗ |
| Modify Why:Causal role | ✗ | ✓ | ✓ | ✗ |
| Configure alert rules | ✗ | ✗ | ✓ | ✗ |
| Manage retention policies | ✗ | ✗ | ✓ | ✗ |
| Verify integrity chains | ✗ | ✓ | ✓ | ✓ |
| Export compliance reports | ✗ | ✗ | ✓ | ✓ |
| Manage tenant access | ✗ | ✗ | ✓ | ✗ |

**Data isolation:** In-memory index sharded by `(tenant_id, classification prefix)`. Every query filter injects `tenant_id = session.tenant` unless the role explicitly permits cross-tenant access (Admin, Auditor). No query can bypass tenant isolation without a valid role.

**Retention policies:** Per-category configurable. Retention duration, archival destination, auto-purge rules. Archived logs remain integrity-verifiable — chain hashes preserved in archive metadata.

**Compliance reporting:** Automated reports showing log completeness, chain integrity status, access audit trail, retention compliance. Exportable as Forma Codex documents.

**Alerting:** Rule-based alerts on log patterns. Configurable per any of the 27 classification dimensions, text match, entry rate threshold. Alert destinations: WebSocket push, email, webhook. Alert history is itself a log category (`C:\PlenumNET\Logs\monitoring\alerts\`).

**Distributed collection:** When PlenumLAN is available, logs from remote nodes aggregate to a central `C:\PlenumNET\Logs\` via PlenumLAN's secure transport. Each node's logs maintain their own integrity chain. The aggregator verifies chains on receipt. Cross-node correlation via shared correlation dimensions (Why trits 17-20).

---

## 13. Tamper-Evident Log Entries (TIS-27 Hash Chain)

Every log entry includes:
- `identity_hash` = TIS-27 of (classification trits 1-24 + hptp_timestamp + message + raw_data)
- `chain_hash` = TIS-27 of (identity_hash || previous chain_hash)

The identity_hash covers the 24 immutable classification trits and the HPTP write timestamp, but NOT the 3 Peace trits, Why:Causal role (dim 18), or received_at. Changing Peace or reclassifying causality does NOT break the integrity chain.

First entry in a file: `chain_hash = identity_hash`.

### 13.1 Incremental Verification

Full chain verification on every read is unnecessary. The viewer tracks a verification watermark per source file — the last entry index that was verified.

- **On initial load:** Verify the full chain for the visible time range. Store the watermark.
- **On new entries (live mode):** Verify only entries after the watermark. Each new entry is one hash computation — verify its identity_hash, then verify its chain_hash links to the previous chain_hash. Amortized O(1) per new entry.
- **On file rotation:** Verify the final chain_hash of the closed file matches the first entry's predecessor in the new file.
- **Periodic full re-verification:** Configurable interval (default: daily). Runs in background, does not block the UI.

### 13.2 Chain Break Handling

On mismatch:
1. Mark the entry as `verified: false`. Do NOT discard it — data loss is worse than unverified data.
2. Flag all subsequent entries from the same source file as dependent on broken chain (visual marker: amber border).
3. The first tampered entry shows a red integrity warning with the exact trit position of the mismatch.
4. Alert the monitoring system (`C:\PlenumNET\Logs\monitoring\alerts\`).
5. Offer the operator: "Re-verify from source" (re-read and re-hash the file) or "Accept and continue" (acknowledge the break, resume chain from next verified entry).

TIS-27 is already implemented: `tis27-hash.ts` (client), `tlsponge385.rs` (Rust), exposed via sponge-wasm-bridge. No new cryptographic code.

---

## 14. Configuration Persistence

Saved as part of the Forma Codex document:
- Log root path
- Active source filters
- Layout mode and dimension assignments
- All active filters
- Live mode settings
- WebSocket endpoints
- Tenant and access control settings
- Retention policy references
- Alert rule references

Opening a saved document reconnects to sources and resumes from last state.

---

## 15. Performance Contracts

| Operation | Target (p95) | Degradation trigger |
|-----------|-------------|---------------------|
| Grid repopulation after filter change | <100ms for 10K entries | Entries > 100K: progressive rendering (visible cells first) |
| Filter apply | <50ms for 10K entries | Dimensions > 12 active: warn user, suggest simplification |
| Chain verification (incremental) | O(1) per new entry | Full re-verify: background thread, no UI block |
| Chain verification (full, 10K entries) | <1s | >100K entries: background with progress indicator |
| Live ingestion throughput | 10K entries/sec per source | Backpressure: buffer to disk when viewer can't keep up |
| Cell rendering (visible viewport) | <16ms (60fps) | >500 entries per cell: show count + latest N, ring buffer older |
| Walk step | <5ms per step | Large grids: precompute walk path on grid init |
| Grid rescale (dimension change) | <500ms for 10K entries | >100K: progressive rehash with progress indicator |

When any target is exceeded, the viewer degrades gracefully — never crashes, never drops data silently. Degradation actions are visible to the user (progress indicators, amber badges, count approximations).

---

## 16. Failure and Recovery

| Failure | Detection | User experience | Recovery |
|---------|-----------|----------------|----------|
| Log directory unavailable | Startup check + periodic poll | Offline indicator per source, stale data shown with timestamp of last successful read | Auto-reconnect when directory reappears. Entries written during outage picked up on reconnect. |
| Log file locked/inaccessible | File open error | Source shown as degraded, other sources unaffected | Retry with exponential backoff (1s, 2s, 4s, max 60s) |
| Chain hash mismatch | On verification (incremental or full) | Red marker on first tampered entry, amber on all subsequent from same file | Offer re-verify or accept-and-continue (§13.2) |
| WebSocket disconnect | Heartbeat timeout (configurable, default 30s) | Live mode pauses, "Reconnecting..." indicator | Exponential backoff (1s, 2s, 4s, max 60s). Buffer entries locally during reconnect. |
| Disk full (log write) | Write error from service | Service degrades to memory-only logging with size cap, alerts monitoring | Auto-purge oldest archived files per retention policy. Alert admin. |
| Disk full (viewer index) | Index write error | Viewer switches to read-only mode, no new filters cached | Clear index, rebuild from source files on next space availability |
| Parse failure (malformed entry) | Parser exception | Entry shown as raw text (face 1), parse error in face 2, no classification | Fallback to plain text. Monitoring alert. Entry still ingested — no data loss. |
| HPTP clock unavailable | Timestamp derivation failure | Entries timestamped with monotonic fallback clock, flagged as "approximate" | Resume HPTP when available. Do not retroactively re-timestamp. |
| Out of memory (viewer) | Allocation failure | Shed oldest pages first, then reduce per-cell entry limit | Progressive shedding. Never crash. Alert admin with memory stats. |

Every failure mode preserves data. The viewer never discards entries due to its own errors. Service-side write failures degrade to memory buffering, not silence.

---

## 17. Log File Lifecycle

### 17.1 File Rotation

Each service writes to `{category}/current.log`. Rotation triggers:
- File size exceeds threshold (configurable in `plenum-app.toml`, no hardcoded default)
- Daily at 00:00 UTC (configurable)
- Manual rotation via admin command

On rotation:
1. Finalize `current.log` with a footer containing the final `chain_hash`
2. Rename to `{category}/{HPTP_timestamp}_{chain_hash_prefix}.log`
3. Create new `current.log` with header containing the previous file's final `chain_hash`
4. First entry in new file: `chain_hash = TIS-27(identity_hash || previous_file_final_chain_hash)`

The chain spans file boundaries. The viewer verifies continuity across files by matching footer → header chain_hash values.

### 17.2 Partial Write Detection

- HPTP timestamps must be monotonically increasing within a file. A non-monotonic timestamp indicates a partial/corrupt write.
- Each entry is length-prefixed. A truncated final entry (length prefix present but content incomplete) is detected and excluded from chain verification.
- The viewer retries reading a truncated entry after a short delay (the service may still be writing).

### 17.3 File Retention

Rotated files are retained per the category's retention policy (§12). The viewer's directory watcher detects file deletions and removes those entries from the in-memory index. Chain verification for remaining files is unaffected — each file's chain is self-contained with cross-file links via header/footer hashes.

---

## 18. Walk Mode — Log-Specific State Machine

Walk mode in the log viewer extends the base Forma Codex walk with within-cell navigation (cells contain multiple entries).

### 18.1 State

```
WalkState = {
    cell: (col, row),              // current grid position
    face: 1 | 2 | 3,              // current face
    entry_index: usize,            // position within cell's entry list
    walk_step: usize,              // position in coprime walk sequence
    filters_active: bool,          // whether walk skips filtered-out cells
}
```

### 18.2 Controls

| Key | Action |
|-----|--------|
| → or 1 | Next cell in coprime walk order. Reset entry_index to 0. |
| ← or 2 | Previous cell in coprime walk order. Reset entry_index to 0. |
| ↓ | Next entry within current cell. If at last entry, no-op. |
| ↑ | Previous entry within current cell. If at first entry, no-op. |
| Space | Cycle face: 1→2→3→1. Stay on same cell and entry. |
| 0 or . | Stay step — no cell movement, face does NOT advance. |
| Tab | Jump to correlated entry — follow Why:parent_id to the parent entry's cell. |
| Shift+Tab | Jump back from correlation (return to previous cell before Tab). |
| Esc | Exit walk mode. |

### 18.3 Filter Interaction

When filters are active, walk mode skips cells that contain no entries matching the current filters. The walk sequence is the same coprime path — filtered-out cells are simply stepped over. The step counter still counts skipped cells (the walk position in the coprime sequence is preserved, not recomputed).

### 18.4 Within-Cell Navigation

Cells in the log viewer may contain many entries (Water mode). ↓/↑ navigate the entry list within the current cell. Face 1 shows the entry's message, face 2 shows its raw data, face 3 shows its correlation context. Cycling faces with Space changes the view for the current entry.

---

## 19. Archive Format

Archived log files use a ternary-native container that preserves integrity chains.

### 19.1 Container Structure

```
[HEADER]
    magic:          [Trit; 9]       — fixed identifier for Forma Codex log archive
    version:        Trit            — archive format version
    tenant_id:      Vec<Trit>       — tenant that owns this archive
    source:         Vec<Trit>       — source category
    start_time:     TritInt         — HPTP timestamp of first entry
    end_time:       TritInt         — HPTP timestamp of last entry
    entry_count:    TritInt         — total entries in archive
    predecessor:    Vec<Trit>       — chain_hash from previous archive (cross-archive chain)

[INDEX]
    Per-entry: (offset: TritInt, identity_hash_prefix: [Trit; 9])
    Enables random access verification without reading the full body.

[BODY]
    Concatenated LogEntry structs with full classification, timestamps, content, and hashes.
    Entries in HPTP timestamp order.

[FOOTER]
    final_chain_hash: Vec<Trit>     — chain_hash of the last entry
    container_hash: Vec<Trit>       — TIS-27 of (HEADER + INDEX + BODY)
```

### 19.2 Verification

- **Entry-level:** Verify any single entry's identity_hash and chain_hash using the index for random access.
- **Container-level:** Verify container_hash covers the entire archive. Detects any modification including index tampering.
- **Cross-archive:** predecessor links archives into a chain. The full history from first boot to present is a single verifiable sequence.

### 19.3 Compliance

Archives are the compliance artifact. Retention policies specify which archives to keep and for how long. An auditor verifies an archive by checking: container_hash (whole-file integrity), chain continuity (entry-to-entry), and predecessor linkage (archive-to-archive). All three checks use TIS-27 — no external tools needed.

---

## 20. File Placement

```
ternary-math/src/forma_codex/log_viewer/
    mod.rs              — module root
    entry.rs            — LogEntry, composite accessors, trit-native types
    parser.rs           — auto-detect parsers (JSONL, syslog, text, CSV), legacy severity mapping
    watcher.rs          — directory watcher for C:\PlenumNET\Logs\
    connector.rs        — WebSocket + import connectors
    populator.rs        — grid auto-population, placement via identity hash
    filter.rs           — in-memory index, filter engine
    attestation.rs      — boot attestation view
    chain.rs            — TIS-27 hash chain verification, incremental watermark, cross-file continuity
    classification.rs   — 27-dimension trit-encoded classification, derivation logic, mutability rules
    tenant.rs           — multi-tenant isolation, access control, authorization matrix
    alert.rs            — rule-based alerting engine
    retention.rs        — retention policies, archival, auto-purge
    archive.rs          — ternary-native archive container (header/index/body/footer)
    walk_log.rs         — log-specific walk state machine (within-cell navigation, correlation jumps)
    lifecycle.rs        — file rotation, partial write detection, cross-file chain linking

client/src/components/forma-codex/
    log-toolbar.ts      — filter UI (27 dimension controls)
    log-live.ts         — live mode controls
    log-alerts.ts       — alert configuration and history UI
```

Existing module revisions:

```
tools/plenum-pack/       — add Logs\ directory creation, log_category field, service schedule for When derivation
server/                  — redirect log output paths, add TIS-27 hashing, use HPTP timestamps
```

---

## 21. Forma Codex vs ELK / Splunk / Loki

| Capability | ELK | Splunk | Loki | Forma Codex |
|------------|-----|--------|------|-------------|
| Cost | $$–$$$ | $$$$$ | $ | Built-in |
| Tamper-evidence | ❌ | ❌ | ❌ | ✅ TIS-27 hash chain |
| 27-dimension trit classification | ❌ | ❌ | ❌ | ✅ Full 3²⁷ space |
| Behavioural time classification (period, phase, cadence, latency) | ❌ | ❌ | ❌ | ✅ When dimensions |
| Spatial layout (coprime grid) | ❌ | ❌ | ❌ | ✅ Any two dimensions as axes |
| Walk mode | ❌ | ❌ | ❌ | ✅ Coprime traversal across classification space |
| Three faces per entry | ❌ | ❌ | ❌ | ✅ Human / raw / correlation |
| Resolution tracking with audit trail | ❌ Separate tool | ❌ Separate tool | ❌ | ✅ Peace trits + Z-stack history |
| Causal chain traversal | ❌ Separate tracing | ❌ Separate tracing | ❌ | ✅ Walk follows Why across grid |
| Reclassifiable causality | ❌ | ❌ | ❌ | ✅ Why:Causal role mutable with history |
| Boot attestation view | ❌ | ❌ | ❌ | ✅ Measurement chain as walkable document |
| Trit-native identity | ❌ | ❌ | ❌ | ✅ TIS-27 sponge output is ternary |
| Rep C zero-exclusion integrity | ❌ | ❌ | ❌ | ✅ Corruption structurally impossible in addressing |
| HPTP femtosecond timestamps | ❌ | ❌ | ❌ | ✅ Native precision |
| Native PlenumNET integration | ❌ | ❌ | ❌ | ✅ Same engine as documents |
| Auto-detect parser | ✅ Logstash grok | ✅ Source type detection | ❌ | ✅ JSONL → syslog → text |
| Live tail | ✅ | ✅ | ✅ | ✅ With attention-aware auto-pause |
| Multi-tenant | ✅ | ✅ | ✅ | ✅ Role-based, TL-DSA authenticated |
| Alerting | ✅ | ✅ | ✅ (via Alertmanager) | ✅ Rule-based across 27 dimensions |
| Compliance reporting | ✅ | ✅ | ❌ | ✅ Exportable as Forma Codex documents |
| Distributed collection | ✅ | ✅ | ✅ | ✅ Via PlenumLAN secure transport |
| Retention policies | ✅ | ✅ | ✅ | ✅ Per-category, archival-preserving integrity |
| Query language needed | Yes (Lucene/KQL) | Yes (SPL) | Yes (LogQL) | No — dimension constraints via UI |
| Petabyte scale | ✅ | ✅ | ✅ | Architecture supports — scales with storage |

The genuine differentiators are: tamper-evidence, 27-dimension trit-encoded classification with behavioural time dimensions, spatial correlation via coprime grids, walk-mode traversal of classification space, causal chain tracing with reclassifiable causality, resolution tracking with Z-stack audit trail, trit-native identity, zero-exclusion integrity, and HPTP femtosecond precision.

---

*Forma Codex 18∏ — Lo Sono Capomastro — Così sia.*
