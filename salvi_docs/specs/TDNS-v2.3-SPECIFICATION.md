# PLENUMNET — TERNARY DOMAIN NAME SYSTEM

## Version 2.3 Specification — The Automatable Ontology

**27-Dimensional Ontological Addressing**

*The Address IS the Description. The Description IS the Route. The Description IS a Measurement.*

---

Applied Physics Division
Capomastro Holdings Ltd.
March 2026

CONFIDENTIAL — PROPRIETARY

---

## 1. Executive Summary

TDNS v2.3 is the Ternary Domain Name System for PlenumNET — a 27-dimensional ontological addressing protocol that unifies name resolution, geometric routing, cryptographic identity, multicast distribution, and precision timing into a single mathematical structure.

Every addressable entity on the network receives a 27-trit coordinate in a ternary hypercube. Each trit is derived from a measurable, machine-deterministic property of the entity. No human judgment. No subjective classification. The Cube Registration Service points a scanner at the entity and the address derives itself.

- **Address space:** 3²⁷ = 7,625,597,484,987 (7.6 trillion)
- **Neighbors per node:** 54 (2 per dimension)
- **Maximum diameter:** 27 hops
- **Routing tables:** Zero
- **Human input required for classification:** Zero
- **Fabric encryption:** All inter-cube traffic encrypted via CON tunnels (PQ-native, BLAKE3 key derivation)

TDNS replaces five conventional protocol systems within a managed fabric: DNS (naming), BGP (routing), PKI (identity), IGMP/PIM (multicast), and PTP (timing). The geometry enforces all five.

### 1.1 Revision History

| Version | Date | Key Changes |
|---------|------|-------------|
| v2.0 | March 2026 | Initial 27-dimension schema (What/Where/How, 3×3×3 structure) |
| v2.1 | March 2026 | 7-category ontological structure; removed language/geography |
| v2.2 | March 2026 | Plain-language dimensions; Entity Type restored; Regulatory added; `.plm` TLD |
| v2.2.1 | March 2026 | Category-grouped display notation; HPTP Live Enforcement Rule (normative) |
| v2.2.2 | March 2026 | Dimension ordering rationale (normative); configurable HPTP thresholds; anycast tiebreaker |
| v2.2.3 | March 2026 | `project_to_GF3` formal definition; sparse routing; wildcard wire encoding; TRN format; security model |
| v2.2.4 | March 2026 | Address derivation principle restored; CRS trust anchor; sparse routing eventual consistency |
| v2.2.5 | March 2026 | Complete schema redesign: fully automatable ontology (WHO/WHAT/WHERE/WHEN/WHY/HOW/PEACE) |
| v2.2.6 | March 2026 | All open items resolved: era-stable WHEN; data-collection WHY; self-cert re-verification; HPTP 2-trit trigger |
| **v2.3** | **March 2026** | **Frozen release. Encryption model clarified (fabric + entity-level). Scaling analysis formalized. Production-ready.** |

---

## 2. Core Principles

### 2.1 Why Ternary

Ternary (base-3) is the natural encoding for classification. Human reasoning defaults to three-way splits: low/medium/high, yes/partial/no, none/some/full. Binary forces every property into a yes/no dichotomy. Decimal wastes resolution. Ternary matches how the world actually categorizes.

### 2.2 Why 27 Dimensions

Twenty-seven dimensions arise from seven questions every person asks about anything on a network: Who is behind it? What is it? Where can I find it? When does it operate? Why does it exist? How does it work? Can I trust it? Each question decomposes into 3–4 measurable properties, yielding 27 total.

### 2.3 The Address IS the Route

In the ternary hypercube, the route between any two nodes is computed by comparing their addresses trit-by-trit. Each differing trit corresponds to one hop. The forwarding algorithm is: find the first differing trit, flip it to match, forward to that neighbor. No routing tables. No convergence delays. No longest-prefix matching. The geometry carries the routing.

### 2.4 The Address IS the Description

Every trit in the address is a deterministic derivation from the entity's measurable properties. The address does not label the entity — it IS the entity's position in ontological space.

If an entity's properties change, its address changes. The coordinate is the derivation, not an approximation of it.

### 2.5 The Description IS a Measurement

Every dimension in the schema is machine-deterministic. CRS derives the address by scanning the entity — no human input, no subjective judgment, no opinion. Each trit value corresponds to an observable, testable signal: a protocol handshake, a DNS lookup, a header inspection, a port scan.

If a machine cannot determine the value from a scan, the dimension does not belong in the address.

### 2.6 Everything Is Encrypted

PlenumNET operates on a zero-cleartext principle. All traffic between cubes travels through CON tunnels — PQ-native encrypted channels with BLAKE3 key derivation. There is no unencrypted path through the fabric.

Trit 25 ("Is it encrypted?") does not measure whether the fabric encrypts the entity's traffic — the fabric always does. Trit 25 measures what encryption the entity itself offers to its end users: does the entity serve plain HTTP (no), basic TLS (basic TLS), or TLS 1.3 with full hardening headers (full TLS)? This is a property of the entity's design, not of the network transport.

The distinction: CON encrypts the pipe. Trit 25 measures what the entity puts into the pipe.

### 2.7 Design Philosophy

Each dimension answers a question a 12-year-old could understand. The three values are obvious and require no training. The questions are the seven that every person asks about anything: **WHO · WHAT · WHERE · WHEN · WHY · HOW · PEACE.**

---

## 3. The 27-Dimensional Schema

### 3.0 Dimension Ordering (Normative)

Dimensions progress from identity (who) through function (what), location (where), temporality (when), purpose (why), mechanics (how), and finally trust (peace). Ordered **most stable first, most dynamic last.**

| Position | Category | Rationale |
|----------|----------|-----------|
| 1st | WHO (1–4) | Entity identity and operator rarely change. A personal blog doesn't become a government portal. |
| 2nd | WHAT (5–8) | Form factor and content type are structural. A website doesn't become a device. |
| 3rd | WHERE (9–12) | Visibility and access are infrastructure decisions, relatively stable. |
| 4th | WHEN (13–16) | Temporal characteristics fixed at design time: era of origin, operating schedule, data freshness, latency profile. |
| 5th | WHY (17–20) | Purpose and business model can shift over an entity's lifetime. |
| 6th | HOW (21–24) | Delivery patterns and protocols may change with architecture updates. |
| 7th | PEACE (25–27) | Security posture is most dynamic. Entity-level encryption, trackers, and audits change with every update. |

Greedy forwarding resolves the most fundamental difference first. Two services from the same operator differing only in tracker count are routed through nearly identical paths.

**Note:** The ordering rationale holds in the fault-free case. Under fault-tolerant rerouting (§11.3), GLB may flip a lower-priority trit to detour around a failed neighbor. Fault tolerance takes precedence over ordering aesthetics.

### 3.1 Category Overview

| Category | Trits | Width | Root Question |
|----------|-------|-------|---------------|
| WHO | 1–4 | 4 | Who is behind it? |
| WHAT | 5–8 | 4 | What is it? |
| WHERE | 9–12 | 4 | Where can I find it? |
| WHEN | 13–16 | 4 | When does it operate? |
| WHY | 17–20 | 4 | Why does it exist? |
| HOW | 21–24 | 4 | How does it work? |
| PEACE | 25–27 | 3 | Can I sleep at night? |

**Total: 27 dimensions = 7.6 trillion addresses**

---

### 3.2 WHO — Who Is Behind It? (Trits 1–4)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 1 | **What kind?** | Personal | Corporate | Governance | WHOIS + legal entity DB |
| 2 | **Who's it for?** | Just me | My group | Everyone | Access patterns, robots.txt |
| 3 | **Who runs it?** | Anonymous | Known | Transparent | About page, WHOIS privacy, business registry |
| 4 | **Who hosts it?** | Me | A provider | The cloud | ASN lookup, IP range, cloud provider fingerprint |

**What kind?** Is this a person's thing, a company's thing, or a government thing? WHOIS and registry data answer immediately.

**Who's it for?** Intended audience. A private journal (just me), a Slack workspace (my group), or Wikipedia (everyone). Observable from access controls and scope declarations.

**Who runs it?** Operator transparency. A random forum with no about page (anonymous). Most company sites list contact info (known). A government portal with full legal disclosure, named officials, physical address, ownership chain (transparent).

**Who hosts it?** Infrastructure model. Self-hosted on a home server (me). A hosting provider like DigitalOcean (a provider). AWS/Azure/GCP (the cloud). ASN and IP range fingerprint this instantly.

---

### 3.3 WHAT — What Is It? (Trits 5–8)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 5 | **What is it?** | Website | App | Device | HTTP headers, content-type, TCP fingerprint |
| 6 | **What's on it?** | Text | Media | Live | MIME types served |
| 7 | **Who uses it?** | People | Software | Both | UI presence vs API-only patterns |
| 8 | **Does it think?** | No | Partly | Yes | ML endpoint detection, inference headers |

**What is it?** A website serves HTML. An app has API endpoints and state. A device responds on non-HTTP ports with IoT/MQTT/custom protocols.

**What's on it?** A blog serves text. YouTube serves media. A stock ticker serves live data. MIME type analysis determines this in milliseconds.

**Who uses it?** Does it have a user interface (people), only API endpoints (software), or both? HTML/CSS presence versus JSON/gRPC-only responses.

**Does it think?** Static file server (no). Recommendation engine, search ranking (partly). Full ML inference, autonomous decisions (yes). Detectable via model-serving headers, inference endpoints, response pattern analysis.

---

### 3.4 WHERE — Where Can I Find It? (Trits 9–12)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 9 | **Who can see it?** | Just me | My group | Everyone | Unauthenticated GET: 200 / 401 / timeout |
| 10 | **Do I need to log in?** | No | Password | ID Check | Challenge detection: none / form / MFA+cert |
| 11 | **How many servers?** | One | Several | Many | DNS A/AAAA record count, CDN detection |
| 12 | **What connection?** | HTTP | WebSocket | Raw TCP | Port scan, protocol handshake |

**Who can see it?** Actual visibility, not intended audience (that's trit 2). Hit the front door with no credentials. 200 = everyone. 401 = my group. Connection refused = just me.

**Do I need to log in?** No challenge (no). Username/password form (password). Multi-factor, client certificate, biometric (ID check).

**How many servers?** One A record (one). A handful of A records or a small CDN (several). Hundreds of edge nodes, global CDN (many).

**What connection?** Standard HTTP/HTTPS (HTTP). Persistent bidirectional (WebSocket). Raw TCP/UDP, MQTT, custom protocols (Raw TCP).

---

### 3.5 WHEN — When Does It Operate? (Trits 13–16)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 13 | **What era?** | Pre-2010 | 2010s | 2020s+ | Domain registration date, first cert issuance, protocol fingerprint |
| 14 | **When is it available?** | Business hours | Extended | 24/7 | Uptime monitoring over sample window |
| 15 | **What kind of data?** | Historical | Current | Live | Content timestamps, streaming protocol detection |
| 16 | **Is it real-time?** | Batch | Near-time | Real-time | Latency measurement, WebSocket/gRPC/SSE |

**What era?** The technological epoch in which the entity was born. Pre-2010 legacy foundations (HTTP/1.0, no TLS, early web). 2010s cloud/mobile era (REST APIs, TLS 1.2, responsive design). 2020s+ AI-native, edge-first, post-quantum era (TLS 1.3, gRPC, ML endpoints). Derived from domain registration date, first certificate issuance, and protocol stack fingerprint. Fixed at registration — an entity's era never changes.

**When is it available?** Operational schedule. Down nights/weekends (business hours). Extended but not full coverage (extended). Always responding (24/7). Determined by uptime monitoring over a sample window.

**What kind of data?** A historical archive (historical). A news site that updates periodically (current). A sensor or ticker that updates continuously (live). Detectable from content timestamps and streaming protocol presence.

**Is it real-time?** Batch-processed reports (batch). API responses in seconds (near-time). Sub-second streaming (real-time). Measurable via latency probes and protocol detection.

This is the HPTP layer encoded in the address. See Section 10.4 for the Live Enforcement Rule.

---

### 3.6 WHY — Why Does It Exist? (Trits 17–20)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 17 | **Does it handle money?** | No | Accepts | Processes | Payment endpoint detection, merchant headers |
| 18 | **Does it want my data?** | No | Some | Lots | Input field count, registration forms, data-sharing scripts |
| 19 | **Does it have policies?** | No | Basic | Detailed | Scan for /privacy, /terms, cookie consent |
| 20 | **Does it cost money?** | Free | Pay-per-use | Subscription | Paywall detection, pricing page |

**Does it handle money?** A blog handles no money (no). A shop accepts payments via Stripe (accepts). A bank or exchange processes transactions as its core function (processes). Detectable via payment endpoint scanning, merchant headers, PCI-DSS indicators.

**Does it want my data?** How much data does the entity collect from users? No input fields, no registration, no forms (no). Basic registration, a few input fields, some cookie consent (some). Extensive data collection — multi-step registration, data-sharing scripts, behavioral tracking endpoints, third-party data-broker integrations (lots). Scannable from input field count, registration flow analysis, and data-sharing script detection.

**Does it have policies?** No privacy policy, no terms (no). A basic privacy page and cookie banner (basic). Comprehensive privacy policy, terms of service, GDPR notices, accessibility statement (detailed). Fully automatable via page scan.

**Does it cost money?** Completely free (free). Charges per use or metered billing (pay-per-use). Recurring payment model (subscription). Detectable via paywall behavior, pricing page presence, payment form analysis.

---

### 3.7 HOW — How Does It Work? (Trits 21–24)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 21 | **Who gets it?** | One person | A group | Whoever's closest | Multicast headers, anycast DNS, CDN fanout |
| 22 | **Which way does data go?** | Out | Through | In | GET vs POST ratio, data flow analysis |
| 23 | **How do I get updates?** | I ask | I subscribe | It tells me | RSS/Atom, WebSocket/SSE, polling detection |
| 24 | **Does it remember me?** | No | For a bit | Always | Cookie/session/localStorage analysis |

**Who gets it?** A direct API call returns to one person. A multicast stream delivers to a group. Anycast DNS routes to whoever's closest. Observable from DNS configuration and protocol behavior.

**Which way does data go?** A sensor or blog sends data out. A CDN or proxy passes data through. A logging service takes data in. Determinable from HTTP method ratios and endpoint analysis.

**How do I get updates?** Polling/request-response only (I ask). RSS/Atom feeds, email subscriptions (I subscribe). WebSocket push, SSE, push notifications (it tells me).

**Does it remember me?** No cookies, no sessions (no). Session cookies that expire (for a bit). Persistent login, user profiles, long-lived state (always).

---

### 3.8 PEACE — Can I Sleep at Night? (Trits 25–27)

| Trit | Question | Value 1 | Value 2 | Value 3 | CRS Scans... |
|------|----------|---------|---------|---------|-------------|
| 25 | **Is it encrypted?** | No | Basic TLS | Full TLS | TLS version, HSTS, CSP, security.txt |
| 26 | **How many trackers?** | Many | Few | None | Third-party request count on page load |
| 27 | **Has it been audited?** | No | Self-certified | Audited | SOC2/ISO badge scan, audit certificates |

**Is it encrypted?** This measures the entity's own encryption posture to its end users — not the fabric transport, which is always encrypted via CON (§2.6). Plain HTTP, no encryption (no). Standard TLS but missing hardening headers (basic TLS). TLS 1.3 + HSTS + CSP + security.txt (full TLS).

**How many trackers?** Load the page, count third-party requests to known tracking domains. Many (10+), few (1–9), none (0). No judgment — just counting.

**Has it been audited?** No audit certifications found (no). Self-declared compliance badges without third-party verification (self-certified). SOC 2, ISO 27001, or equivalent third-party audit certificates present (audited).

---

## 4. Complete Schema Reference

| Trit | Category | Question | Value 1 | Value 2 | Value 3 |
|------|----------|----------|---------|---------|---------|
| 1 | WHO | What kind? | Personal | Corporate | Governance |
| 2 | WHO | Who's it for? | Just me | My group | Everyone |
| 3 | WHO | Who runs it? | Anonymous | Known | Transparent |
| 4 | WHO | Who hosts it? | Me | A provider | The cloud |
| 5 | WHAT | What is it? | Website | App | Device |
| 6 | WHAT | What's on it? | Text | Media | Live |
| 7 | WHAT | Who uses it? | People | Software | Both |
| 8 | WHAT | Does it think? | No | Partly | Yes |
| 9 | WHERE | Who can see it? | Just me | My group | Everyone |
| 10 | WHERE | Do I need to log in? | No | Password | ID Check |
| 11 | WHERE | How many servers? | One | Several | Many |
| 12 | WHERE | What connection? | HTTP | WebSocket | Raw TCP |
| 13 | WHEN | What era? | Pre-2010 | 2010s | 2020s+ |
| 14 | WHEN | When is it available? | Business hours | Extended | 24/7 |
| 15 | WHEN | What kind of data? | Historical | Current | Live |
| 16 | WHEN | Is it real-time? | Batch | Near-time | Real-time |
| 17 | WHY | Does it handle money? | No | Accepts | Processes |
| 18 | WHY | Does it want my data? | No | Some | Lots |
| 19 | WHY | Does it have policies? | No | Basic | Detailed |
| 20 | WHY | Does it cost money? | Free | Pay-per-use | Subscription |
| 21 | HOW | Who gets it? | One person | A group | Whoever's closest |
| 22 | HOW | Which way does data go? | Out | Through | In |
| 23 | HOW | How do I get updates? | I ask | I subscribe | It tells me |
| 24 | HOW | Does it remember me? | No | For a bit | Always |
| 25 | PEACE | Is it encrypted? | No | Basic TLS | Full TLS |
| 26 | PEACE | How many trackers? | Many | Few | None |
| 27 | PEACE | Has it been audited? | No | Self-certified | Audited |

---

## 5. Address Notation

### 5.1 Human Name

```
google.plm
pptpro.capomastro.plm
nonnas-cucina.plm
```

The `.plm` TLD signals the metatronic bridge to route through TDNS rather than legacy DNS.

### 5.2 Canonical Wire Format

```
232.311.331.332.121.312.121.331.313
```

Nine groups of three trits. Each trit in {1, 2, 3} — never 0.

### 5.3 Category-Grouped Debug Format

Trits grouped by category with two-letter prefixes:

```
WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313
```

| Prefix | Category | Trits | Width |
|--------|----------|-------|-------|
| `WO:` | WHO | 1–4 | 4 trits |
| `WA:` | WHAT | 5–8 | 4 trits |
| `WR:` | WHERE | 9–12 | 4 trits |
| `WN:` | WHEN | 13–16 | 4 trits |
| `WY:` | WHY | 17–20 | 4 trits |
| `HO:` | HOW | 21–24 | 4 trits |
| `PE:` | PEACE | 25–27 | 3 trits |

### 5.4 Wildcard Mask Format

Human-readable `*` for wildcarded trits:

```
Category:   WO:**** WA:**** WR:**** WN:***3 WY:**** HO:**** PE:***
```

This constrains only Is it real-time?=Real-time (trit 16=3). Sub-cube of 3²⁶ = 2.54 trillion addresses.

### 5.5 Wildcard Wire Encoding (Normative)

On the wire, a sub-cube destination is two 27-trit fields:

| Field | Size | Description |
|-------|------|-------------|
| `base` | 27 trits | Constrained trit values. Wildcarded positions set to 1 (neutral convention). |
| `mask` | 27 trits | 0 = wildcarded (ignore base). 1 = constrained (match base). |

GLB membership test: `(local_addr[i] == base[i]) OR (mask[i] == 0)` for all i. Implementations MUST ignore `base[i]` when `mask[i] == 0`.

Mask trits use values {0, 1} only — a 27-bit binary vector. Implementations MAY pack as a 27-bit integer (4 bytes).

A point address (single entity) has mask = all 1s.

### 5.6 Conversion

All three formats are lossless and interconvertible:

```
google.plm
  → TRN lookup → canonical wire format
  → category split → WO:xxxx WA:xxxx WR:xxxx WN:xxxx WY:xxxx HO:xxxx PE:xxx
```

---

## 6. Human Naming Layer

### 6.1 Name Structure

```
google.plm
pptpro.capomastro.plm
nonnas-cucina.plm
crs.infra.capomastro.plm
```

### 6.2 Registration

When an entity registers with TDNS, CRS scans the entity and derives the 27-trit address from measured properties. No human input required. The name and address are stored as a TRN record.

### 6.3 Resolution

A lookup for `pptpro.capomastro.plm` returns the full 27-trit address. The routing path is immediately computable.

### 6.4 Dimensional Queries (Machine Interface)

```
tdns query --kind=corporate --encrypted=full-tls --real-time=yes
tdns query --audience=everyone --data=live --audited=yes
tdns query --era=2020s --connection=websocket --trackers=none
```

Each parameter constrains trits. Unconstrained dimensions are wildcarded. The result is a sub-cube.

---

## 7. TRN Record Format (Normative)

### 7.1 Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | UTF-8 string | Human-readable name (e.g., `pptpro.capomastro.plm`) |
| `address` | 27 trits | Canonical 27-trit address, derived from scan measurements |
| `public_key` | bytes | Entity's public key (for ownership proof) |
| `ttl` | u32 | Cache time-to-live in seconds |
| `registered_at` | u64 | HPTP nanosecond timestamp of registration |
| `zone` | UTF-8 string | Authoritative zone (e.g., `capomastro.plm`) |
| `scan_hash` | 32 bytes | BLAKE3 hash of the scan results that produced this address |

### 7.2 Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `valid_from` | u64 | HPTP nanosecond timestamp — resolves only after |
| `valid_until` | u64 | HPTP nanosecond timestamp — resolves only before |
| `hptp_sync_status` | enum | `synced`, `degraded`, or `unknown`. Required for HPTP-mandatory entities. |
| `hptp_offset_ns` | i64 | Last reported HPTP offset in nanoseconds |
| `attributes` | map | The 27 measured values as scanned |
| `last_rescan` | u64 | HPTP timestamp of most recent re-scan |

### 7.3 Wire Encoding

Serialized as length-prefixed binary, network byte order. The 27-trit address packed as 27 two-bit values in 7 bytes (54 bits, padded to 56). Trit encoding: 1 = `01`, 2 = `10`, 3 = `11`. Value `00` reserved, MUST NOT appear. Attributes as (dimension_id: u8, value: u8) pairs. Typically under 512 bytes.

---

## 8. Example Addresses

### 8.1 Google

**Human name:** `google.plm`

| Trit | Question | Answer | Value |
|------|----------|--------|-------|
| 1 | What kind? | Corporate | 2 |
| 2 | Who's it for? | Everyone | 3 |
| 3 | Who runs it? | Known | 2 |
| 4 | Who hosts it? | The cloud | 3 |
| 5 | What is it? | Website | 1 |
| 6 | What's on it? | Text | 1 |
| 7 | Who uses it? | Both | 3 |
| 8 | Does it think? | Yes | 3 |
| 9 | Who can see it? | Everyone | 3 |
| 10 | Do I need to log in? | No | 1 |
| 11 | How many servers? | Many | 3 |
| 12 | What connection? | HTTP | 1 |
| 13 | What era? | Pre-2010 | 1 |
| 14 | When is it available? | 24/7 | 3 |
| 15 | What kind of data? | Current | 2 |
| 16 | Is it real-time? | Near-time | 2 |
| 17 | Does it handle money? | Accepts | 2 |
| 18 | Does it want my data? | Lots | 3 |
| 19 | Does it have policies? | Detailed | 3 |
| 20 | Does it cost money? | Free | 1 |
| 21 | Who gets it? | One person | 1 |
| 22 | Which way does data go? | Through | 2 |
| 23 | How do I get updates? | I ask | 1 |
| 24 | Does it remember me? | For a bit | 2 |
| 25 | Is it encrypted? | Full TLS | 3 |
| 26 | How many trackers? | Many | 1 |
| 27 | Has it been audited? | Audited | 3 |

**Canonical:** `232.311.331.132.233.112.121.232.313`
**Category:** `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313`

---

### 8.2 PPTPro (Plenum Pulse Tonal Professor)

**Human name:** `pptpro.capomastro.plm`

| Trit | Question | Answer | Value |
|------|----------|--------|-------|
| 1 | What kind? | Corporate | 2 |
| 2 | Who's it for? | Everyone | 3 |
| 3 | Who runs it? | Transparent | 3 |
| 4 | Who hosts it? | The cloud | 3 |
| 5 | What is it? | App | 2 |
| 6 | What's on it? | Live | 3 |
| 7 | Who uses it? | Both | 3 |
| 8 | Does it think? | Yes | 3 |
| 9 | Who can see it? | My group | 2 |
| 10 | Do I need to log in? | Password | 2 |
| 11 | How many servers? | Several | 2 |
| 12 | What connection? | WebSocket | 2 |
| 13 | What era? | 2020s+ | 3 |
| 14 | When is it available? | 24/7 | 3 |
| 15 | What kind of data? | Live | 3 |
| 16 | Is it real-time? | Real-time | 3 |
| 17 | Does it handle money? | No | 1 |
| 18 | Does it want my data? | Some | 2 |
| 19 | Does it have policies? | Basic | 2 |
| 20 | Does it cost money? | Free | 1 |
| 21 | Who gets it? | A group | 2 |
| 22 | Which way does data go? | Out | 1 |
| 23 | How do I get updates? | It tells me | 3 |
| 24 | Does it remember me? | Always | 3 |
| 25 | Is it encrypted? | Full TLS | 3 |
| 26 | How many trackers? | None | 3 |
| 27 | Has it been audited? | Self-certified | 2 |

**Canonical:** `233.323.322.222.333.312.121.331.332`
**Category:** `WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332`

**HPTP-mandatory:** Trits 15=3 AND 16=3 (Live data + Real-time). See Section 10.4.

---

### 8.3 Nonna's Food Blog

**Human name:** `nonnas-cucina.plm`

| Trit | Question | Answer | Value |
|------|----------|--------|-------|
| 1 | What kind? | Personal | 1 |
| 2 | Who's it for? | Everyone | 3 |
| 3 | Who runs it? | Anonymous | 1 |
| 4 | Who hosts it? | A provider | 2 |
| 5 | What is it? | Website | 1 |
| 6 | What's on it? | Text | 1 |
| 7 | Who uses it? | People | 1 |
| 8 | Does it think? | No | 1 |
| 9 | Who can see it? | Everyone | 3 |
| 10 | Do I need to log in? | No | 1 |
| 11 | How many servers? | One | 1 |
| 12 | What connection? | HTTP | 1 |
| 13 | What era? | 2010s | 2 |
| 14 | When is it available? | 24/7 | 3 |
| 15 | What kind of data? | Historical | 1 |
| 16 | Is it real-time? | Batch | 1 |
| 17 | Does it handle money? | No | 1 |
| 18 | Does it want my data? | No | 1 |
| 19 | Does it have policies? | No | 1 |
| 20 | Does it cost money? | Free | 1 |
| 21 | Who gets it? | One person | 1 |
| 22 | Which way does data go? | Out | 1 |
| 23 | How do I get updates? | I ask | 1 |
| 24 | Does it remember me? | No | 1 |
| 25 | Is it encrypted? | Basic TLS | 2 |
| 26 | How many trackers? | Many | 1 |
| 27 | Has it been audited? | No | 1 |

**Canonical:** `131.211.131.111.231.111.111.111.211`
**Category:** `WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211`

---

### 8.4 Side-by-Side

```
Google:  WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313
PPTPro:  WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332
Blog:    WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211
```

### 8.5 Ontological Distance

| Pair | Differing Trits | Distance |
|------|----------------|----------|
| Google ↔ PPTPro | 15 of 27 | 15 hops |
| Google ↔ Blog | 16 of 27 | 16 hops |
| PPTPro ↔ Blog | 21 of 27 | 21 hops |

PPTPro and the blog differ on 21 of 27 dimensions — nearly as different as two things on the internet can be. The topology reflects this.

---

## 9. Self-Certifying Names

### 9.1 Design for Automatable Schema

All 27 trits are scan-derived measurements. No trits are reserved for identity. The public key lives in the TRN record and provides ownership proof via challenge-response, separate from the address derivation.

### 9.2 Ownership Proof

To prove ownership of a name, the entity signs a challenge (the current HPTP timestamp) with its private key. The verifier checks the signature against the `public_key` in the TRN record. No certificate authority required.

### 9.3 Scan Hash Binding

The TRN record includes a `scan_hash` field: `BLAKE3(scan_results)`. CRS signs the scan hash with its own key:

```
entity proves ownership of name → via public_key signature
CRS proves address derivation → via scan_hash + CRS signature
```

Every trit is a measurement. Ownership is proved externally, not embedded in the coordinate.

### 9.4 Re-Verification Protocol (Normative)

```
1. Entity claims: "I am pptpro.capomastro.plm"
2. Verifier fetches TRN record from CRS
3. Entity signs HPTP timestamp with private key
4. Verifier checks signature against TRN public_key    → proves ownership
5. Verifier checks scan_hash against CRS signature     → proves address integrity
6. Verifier re-scans entity, compares to TRN attributes → proves address still accurate
```

Step 6 is an open protocol: any party MAY request CRS to re-scan a name and return updated attributes with a new scan_hash. If the re-scan produces a different address than the current TRN, CRS flags the record as **drifted** and initiates re-derivation per Section 13.

---

## 10. Time-Aware Resolution (HPTP)

### 10.1 Deterministic Failover

Records carry `valid_from` and `valid_until` with nanosecond precision. All caches see the same HPTP time. No split-brain.

### 10.2 Time-Locked Names

`auction-2026Q1.exchange.capomastro.plm` resolves only during the auction window. Time IS access control.

### 10.3 Temporal Multicast

Sub-cube addressing + time-windowed resolution. MiFID II compliance where timing IS the regulatory requirement.

### 10.4 HPTP Live Enforcement Rule (Normative)

When an entity's address declares both of the following trit values, the entity is classified as **HPTP-mandatory**:

| Trit | Question | Required Value |
|------|----------|---------------|
| 15 | What kind of data? | Live (3) |
| 16 | Is it real-time? | Real-time (3) |

An entity serving live data with real-time delivery is making a timing guarantee the network must enforce.

**At registration:** CRS MUST verify HPTP synchronization within tolerance. Registration MUST be rejected if sync cannot be confirmed.

**At runtime:** GLB MUST monitor HPTP sync status via FTS heartbeats. If an HPTP-mandatory node's offset exceeds tolerance, GLB MUST:

1. Mark the node as **HPTP-degraded** in the FTS dead set.
2. Stop forwarding packets to the degraded node.
3. Route around via alternate trit flip.
4. Resume when sync is restored (hold-down: default 5 seconds).

**At resolution:** TRN records for HPTP-mandatory entities MUST include `hptp_sync_status`.

**Sync tolerance thresholds (CRS policy):**

| Timing Value | Default Tolerance |
|-------------|-------------------|
| Real-time (trit 16=3) | ≤1μs offset |
| Near-time (trit 16=2) | ≤100μs offset |

**Rationale:** An address is a derivation from measurements. If the measurements include "live data" and "real-time delivery," the network must verify those measurements continuously or the derivation loses integrity.

---

## 11. Geometric Operations

### 11.1 Routing

1. Compare S and D trit-by-trit.
2. Find the first dimension i where S[i] ≠ D[i].
3. Flip S[i] to D[i], forward to that neighbor.
4. Repeat.

Path length = Hamming distance. Worst: 27 hops. Average: ~18. Loop-free in a fault-free hypercube. First flip resolves the most fundamental difference (WHO before WHAT before WHERE...).

### 11.2 Sparse Hypercube Routing (Normative)

Each node maintains a **neighbor map** — 54 entries max (27 dims × 2 directions × 1 address):

```
neighbor_map[i][target_v] = closest_populated_addr with trit i == target_v
```

CRS selects geometrically closest candidate per direction. Per-node state is constant regardless of network size.

**Consistency model:** Eventual consistency bounded by HPTP staleness. A stale map causes suboptimal paths but cannot cause routing loops — the forwarding algorithm is strictly dimension-correcting. Stale entries pointing to deregistered nodes are caught by FTS heartbeat failure; GLB falls back to the next differing dimension.

If no populated node exists in a required direction, forwarding falls back to the next differing dimension. Deployments MUST monitor population density per dimension via CRS metrics.

### 11.3 Geometric Multicast

A wildcard address (base + mask per §5.5) defines a sub-cube. Packets reach all matching entities. No IGMP. No PIM. The geometry IS the distribution tree.

### 11.4 Sub-cube Forwarding

1. Test membership per §5.5. Deliver if inside.
2. Forward to qualifying neighbors, excluding arrival node.
3. Forward only along unconstrained dimensions in ascending order.

Natural spanning tree. Zero additional state.

### 11.5 Anycast

When trit 21 (Who gets it?) = Whoever's closest, GLB routes to the nearest match by Hamming distance.

**Deterministic tiebreaker:** Lowest canonical wire-format address, lexicographic, trit 1 most significant.

---

## 12. Integration with Inter-Cube Services

### 12.1 CRS (Cube Registration Service)

Scans entities and derives 27-trit addresses from measurements. Stores TRN records with scan hashes. Maintains neighbor maps with eventual consistency. For HPTP-mandatory entities, verifies sync at registration. Performs periodic re-scans to detect property drift. Supports open re-verification protocol (§9.4). Critical trust anchor (§15.6).

### 12.2 CON (Cube Overlay Network)

PQ-encrypted tunnels (BLAKE3 key derivation) between all geometric neighbors. Every inter-cube link is encrypted. There is no unencrypted path through the fabric. Each of the 54 neighbor links has its own derived tunnel key: `BLAKE3(local_addr || neighbor_addr || shared_secret)`.

### 12.3 FTS (Fault Tolerance Service)

Heartbeats carry HPTP offset data. Marks failed and HPTP-degraded nodes in dead set.

### 12.4 GLB (Geometric Load Balancer)

Dimension-agnostic next-hop. Point, sub-cube, and sparse routing. HPTP enforcement. Anycast tiebreaker. Drift-redirect during re-derivation grace periods.

### 12.5 Metatronic Bridge

`.plm` → TDNS. All else → legacy DNS. Two worlds, one resolver.

---

## 13. CRS Re-Scan and Re-Derivation (Normative)

Because the address is derived from measurements, and measurements can change, CRS MUST periodically re-scan registered entities.

### 13.1 Re-scan Policy

CRS re-scans on a deployment-configurable schedule (default: weekly). If re-scan produces a different 27-trit address, the entity has undergone **property drift**.

### 13.2 Property Drift Handling

1. CRS computes the new address from the new scan.
2. CRS logs the drift event (old address, new address, changed trits) to the audit trail.
3. CRS updates the TRN record to the new address.
4. GLB redirects packets from old address to new address for a configurable grace period (default: 24 hours).
5. After grace period, the old address is released.

This is not a "mutability model." There are no immutable or mutable classes. Every trit is derived from measurement. If the measurement changes, the address changes. The math does not approximate.

### 13.3 Forced Re-derivation Triggers

Beyond scheduled re-scans, CRS MUST re-derive when:

- The entity requests re-registration.
- FTS detects the entity has been offline beyond a configurable threshold.
- An external re-verification (§9.4 step 6) reports measurement mismatch.

---

## 14. Scaling Properties

### 14.1 Address Space

The ternary hypercube at 27 dimensions provides 3²⁷ = 7,625,597,484,987 addresses — 7.6 trillion. For context:

| System | Address Space | Ratio to TDNS |
|--------|-------------|---------------|
| IPv4 | 4.3 billion | TDNS is 1,770× larger |
| MAC-48 | 281 trillion | TDNS is 3.7% of MAC space |
| IPv6 | 3.4 × 10³⁸ | IPv6 is astronomically larger |
| TDNS-27 | 7.6 trillion | Current specification |

7.6 trillion addresses is sufficient for every device, service, website, and entity currently on the internet — by several orders of magnitude. The approximately 30 billion connected devices worldwide today would occupy 0.0004% of the address space.

### 14.2 Routing Efficiency

Greedy geometric forwarding has properties that no conventional routing protocol can match:

**Constant per-node state.** Each node stores 54 neighbor map entries regardless of network size. A 1,000-node fabric and a 1,000,000-node fabric have identical per-node memory requirements. Compare to BGP, where the global routing table exceeds 1 million entries and grows with every new prefix.

**Guaranteed worst-case path length.** The maximum distance between any two nodes is 27 hops. This is a mathematical invariant of the geometry, not a function of network size. Adding a billion nodes does not increase the diameter by one hop. Compare to internet BGP, where typical AS path lengths are 4–5 hops but worst-case paths can exceed 10, and the diameter grows with the network.

**Zero convergence time.** There is no convergence event after a topology change. When a node registers or deregisters, CRS pushes neighbor map updates to affected nodes. There is no distributed routing protocol negotiating paths. Compare to BGP convergence, which can take minutes during prefix storms.

**Predictable latency.** Every hop in the hypercube corresponds to exactly one trit correction. The path from S to D is deterministic — given the same source and destination, the path is the same every time. No load-based rerouting, no ECMP, no jitter from route oscillation.

### 14.3 Sparse Cube Behavior

At realistic deployment sizes, the cube is overwhelmingly sparse:

| Nodes | Occupancy | Avg Neighbor Distance |
|-------|-----------|----------------------|
| 10,000 | 1 in 762 billion | ~13 trits apart |
| 1 million | 1 in 7.6 million | ~10 trits apart |
| 1 billion | 1 in 7,626 | ~6 trits apart |
| 1 trillion | 1 in 7.6 | ~2 trits apart |

As density increases, the neighbor map degenerates toward direct geometric neighbors and the sparse routing algorithm reduces to standard greedy forwarding. The system gets faster as it gets bigger — the opposite of conventional routing.

The critical threshold is per-dimension coverage: for efficient routing, every populated node should have at least one neighbor map entry per dimension per direction. With 27 dimensions × 2 directions = 54 entries needed, this requires roughly 54 nodes minimum. Below that, some dimensions may have no populated neighbor, forcing multi-hop fallbacks.

### 14.4 Extensibility

The forwarding algorithm, multicast, and self-certifying mechanism are all dimension-agnostic. Scaling beyond 27 dimensions requires only widening the address:

| Dimensions | Address Space | Neighbors | Max Hops |
|------------|--------------|-----------|----------|
| 13 | 1.59 million | 26 | 13 |
| 27 | 7.63 trillion | 54 | 27 |
| 39 | 4.05 × 10¹⁸ | 78 | 39 |
| 54 | 5.81 × 10²⁵ | 108 | 54 |
| 81 | 4.43 × 10³⁸ | 162 | 81 |

New dimensions append at trits 28+. Existing 27-trit resolvers ignore unknown high trits; forwarding remains compatible on known differing dimensions. No flag day. No hard fork. No architectural rewrite.

At 81 dimensions, the address space (4.43 × 10³⁸) is comparable to IPv6. But unlike IPv6, every bit of the address carries semantic meaning. There are no wasted prefix hierarchies, no private ranges consuming public space, no NAT.

### 14.5 What Does NOT Scale

**CRS re-scan load.** At 1 billion entities with weekly re-scans, CRS must process ~1.65 million scans per second continuously. This is the dominant scaling constraint. Mitigation: distribute scanning across CRS replicas by zone, use differential scanning (only re-check trits that have historically drifted for each entity), and increase re-scan intervals for stable entities.

**Neighbor map push storms.** When a high-traffic node registers or deregisters, CRS must push updates to all nodes that had it in their neighbor maps — potentially thousands. Mitigation: eventual consistency (§11.2) absorbs the latency. Stale entries cause suboptimal paths, not failures.

**Multicast fan-out.** A sub-cube with few constraints (e.g., one trit constrained = 3²⁶ addresses) could theoretically fan out to trillions of nodes. In practice, only populated addresses receive packets. In a sparse cube, the fan-out is bounded by actual population. Deployments MUST set maximum fan-out limits per sub-cube query.

---

## 15. Security Considerations

### 15.1 Encryption Model

PlenumNET operates on a dual-layer encryption model:

**Fabric layer (always on).** All inter-cube traffic is encrypted via CON tunnels. BLAKE3 key derivation. PQ-native. There is no unencrypted path between any two cubes. This is not optional — the fabric does not support cleartext forwarding.

**Entity layer (trit 25).** Trit 25 measures the encryption the entity offers to its end users — the content inside the tunnel. An entity serving plain HTTP over a CON tunnel is still encrypted at the fabric level, but its content is unencrypted to the end user. An entity serving TLS 1.3 provides end-to-end encryption from user to entity, with CON providing an additional encryption layer underneath.

The practical consequence: every packet on PlenumNET is encrypted at least once (CON). HPTP-mandatory entities operating under strict regulatory regimes (MiFID II, FINRA 613) get double encryption — CON for the pipe, TLS for the payload.

### 15.2 Address Spoofing

Address is derived from measurements, not declared. CRS scans the entity directly. Spoofing requires making the entity appear different to CRS's scanner — a harder problem than forging a registration form.

### 15.3 Address Squatting

Crafted entities near high-value targets. Mitigated by: CRS scans actual properties (not self-reported), rate-limiting registrations per zone, neighbor map monitoring for unexpected registrations.

### 15.4 Routing Attacks

No routing tables to poison. Neighbor maps maintained exclusively by CRS. FTS heartbeats authenticated via CON tunnel keys.

### 15.5 Timing Manipulation

CON encryption prevents HPTP forgery. Live Enforcement Rule prevents false real-time claims. Ownership proof replay prevented by HPTP timestamp challenges.

### 15.6 Scan Manipulation

An adversary could present different properties to CRS's scanner than to actual users. Mitigated by: random re-scan scheduling, scan hash binding in TRN records, and open re-verification (§9.4 step 6 — anyone can re-scan and compare).

### 15.7 CRS as Critical Trust Anchor

CRS is the single source of truth for address derivation, TRN storage, and neighbor map maintenance.

- **Trusted execution:** CRS SHOULD run on hardware-attested environments.
- **Audit logging:** All CRS operations MUST be logged to an append-only audit trail. Deployments SHOULD witness to Hedera HCS.
- **Replication:** Production deployments MUST use BFT consensus (minimum 3f+1 replicas). Consensus required only for ordering registrations, not for the derivation itself.
- **Key management:** CRS signing keys in HSMs, rotated per deployment schedule.

A compromised CRS cannot silently alter the derivation algorithm (any party can independently re-scan and verify via §9.4), but it can inject false TRN records. The audit trail and BFT replication are the primary defenses.

---

## 16. What TDNS Replaces

| Protocol | Conventional Role | TDNS Equivalent |
|----------|-------------------|-----------------|
| DNS | Name → IP | Name → 27-trit coordinate via TRN |
| BGP/OSPF | Routing tables | Greedy forwarding; neighbor maps only |
| PKI/CA | Certificate authorities | Ownership via challenge-response + scan hash binding |
| IGMP/PIM | Multicast groups | Sub-cube via dimensional wildcards |
| PTP/NTP | Time synchronization | HPTP nanosecond timestamps |

Five protocol systems collapsed into the geometry of a 27-dimensional ternary hypercube.

---

## 17. Implementation Status

### 17.1 Completed — TDNS v1 (Kernel Module)

`tdns.rs`: 530 lines Rust, 8 record types, BTreeMap cache, 19 tests.

### 17.2 Completed — Inter-Cube Services (Docker)

CRS/CON/FTS/GLB: 3,553 lines Rust, 57 tests, 4-node Docker, 11 HTTP endpoints.

### 17.3 Next — TDNS v2.3

1. `CubeAddr` 13-trit → 27-trit.
2. CRS scanner framework: protocol fingerprinting, header analysis, WHOIS lookup, MIME detection, latency probing, tracker counting, TLS inspection, audit badge scanning, data-collection analysis.
3. Scan-to-trit derivation engine for all 27 dimensions.
4. `scan_hash` field: BLAKE3 of scan results, CRS-signed.
5. TRN record wire format per §7.
6. `SubCube { base, mask }` with wire encoding per §5.5.
7. Ownership proof via challenge-response against TRN `public_key`.
8. Open re-verification protocol (§9.4 step 6).
9. HPTP `valid_from` / `valid_until` + `hptp_sync_status` on TRN.
10. Configurable HPTP tolerance thresholds in CRS policy.
11. Sparse neighbor map with eventual-consistency push.
12. FTS heartbeats carry HPTP offset.
13. GLB: sub-cube multicast + HPTP enforcement + anycast tiebreaker + drift-redirect.
14. CRS periodic re-scan and re-derivation engine with differential scanning.
15. `SCHEMA` record type for publishing dimensional mappings.
16. Category-grouped display formatter (`WO:xxxx WA:xxxx WR:xxxx WN:xxxx WY:xxxx HO:xxxx PE:xxx`).

---

## 18. Version Comparison

| Aspect | v2.0 | v2.1 | v2.2 | v2.2.1–2 | v2.2.3–4 | v2.2.5 | v2.2.6 | v2.3 |
|--------|------|------|------|----------|----------|--------|--------|------|
| Categories | 3×3×3 | 7 ontol. | 7 plain | Same | Same | WHO→PEACE | Same | Same (frozen) |
| Human input | Required | Required | Required | Required | Required | Zero | Zero | Zero |
| Automation | — | — | — | — | — | 27/27 | 27/27 stable | 27/27 stable |
| Encryption | — | — | — | — | — | — | — | Dual-layer (§2.6, §15.1) |
| HPTP | — | — | — | Live Rule | + Thresh | 2-trit | Confirmed | Confirmed |
| Self-cert | — | — | — | — | GF3→addr | TRN+scan | + Re-verify | Same |
| Scaling | Table only | Same | Same | Same | Same | Same | Same | Full analysis (§14) |
| Security | — | — | — | — | Threats | + Scan manip. | Same | + Encryption model |
| Status | — | — | — | — | — | — | — | **Frozen** |

---

© 2026 Capomastro Holdings Ltd. All rights reserved.

*The Salvi Framework — Applied Physics Division*

*Simple. Measurable. Automatable. Enforced. Deterministic. Derived.*