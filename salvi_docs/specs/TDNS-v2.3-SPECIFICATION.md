# PLENUMNET — TERNARY DOMAIN NAME SYSTEM

## Version 2.3.3 Specification — First-Principle Derivation Engine

**27-Dimensional Ontological Addressing + 28th Collision Resolution Digit**

*The Address IS the Description. The Description IS the Route. The Description IS a Measurement.*

---

Applied Physics Division  
Capomastro Holdings Ltd.  
March 2026

CONFIDENTIAL — PROPRIETARY

---

## 1. Executive Summary

TDNS v2.3.3 is the Ternary Domain Name System for PlenumNET — a 27-dimensional ontological addressing protocol that unifies name resolution, geometric routing, cryptographic identity, multicast distribution, and precision timing into a single mathematical structure.

Every addressable entity on the network receives a 27-trit coordinate in a ternary hypercube. Each trit is derived from a single universal formula applied to binary signal counts. Alongside each trit, a confidence digit (1–9) quantifies how far the measured proportion lies from the quantization boundaries. When two entities genuinely share the same 27-trit coordinate, a 28th Collision Resolution Digit (CRD, 1–9) distinguishes them, allowing up to nine entities per cube before the address is considered full.

- **Address space (27‑trit):** 3²⁷ = 7,625,597,484,987 (7.6 trillion)
- **Unique identifiers (with CRD):** 3²⁷ × 9 = 68,630,377,364,883 (68.6 trillion)
- **Neighbors per node:** 54 (2 per dimension)
- **Maximum diameter:** 27 hops
- **Routing tables:** Zero
- **Human input required for classification:** Zero
- **Derivation formulas:** Three (quantization + confidence + CRD assignment)
- **Arbitrary thresholds:** Zero
- **Fabric encryption:** All inter‑cube traffic encrypted via CON tunnels (PQ‑native, CryptoHash key derivation)

TDNS replaces five conventional protocol systems within a managed fabric: DNS (naming), BGP (routing), PKI (identity), IGMP/PIM (multicast), and PTP (timing). The geometry enforces all five.

### 1.1 Revision History

| Version | Date       | Key Changes |
|---------|------------|-------------|
| v2.0    | March 2026 | Initial 27‑dimension schema (What/Where/How, 3×3×3 structure) |
| v2.1    | March 2026 | 7‑category ontological structure; removed language/geography |
| v2.2    | March 2026 | Plain‑language dimensions; Entity Type restored; Regulatory added; `.plm` TLD |
| v2.2.1  | March 2026 | Category‑grouped display notation; HPTP Live Enforcement Rule (normative) |
| v2.2.2  | March 2026 | Dimension ordering rationale (normative); configurable HPTP thresholds; anycast tiebreaker |
| v2.2.3  | March 2026 | `project_to_GF3` formal definition; sparse routing; wildcard wire encoding; TRN format; security model |
| v2.2.4  | March 2026 | Address derivation principle restored; CRS trust anchor; sparse routing eventual consistency |
| v2.2.5  | March 2026 | Complete schema redesign: fully automatable ontology (WHO/WHAT/WHERE/WHEN/WHY/HOW/PEACE) |
| v2.2.6  | March 2026 | All open items resolved: era‑stable WHEN; data‑collection WHY; self‑cert re‑verification; HPTP 2‑trit trigger |
| v2.3    | March 2026 | Frozen release. Encryption model clarified (fabric + entity‑level). Scaling analysis formalized. Production‑ready. |
| v2.3.1  | March 2026 | Confidence derivation per dimension (§4.0‑4.2). Collision Resolution Digit for global uniqueness (§4.3‑4.7). Geometric resonance with 28 and 364 (§2.8). |
| v2.3.2  | March 2026 | First‑principle derivation engine: all 12 quantitative dimensions use binary signal counting with defined N values. CryptoHash abstraction (BLAKE3 → TL‑Sponge migration path). Scanner hardened: gov.uk governance, MFA auth‑context gating, expanded CDN detection. scan_hash includes full raw values with type tags. All arbitrary thresholds eliminated. |
| **v2.3.3** | **March 2026** | **SQLite persistent storage for TRN records (survives restarts). Browser extension with search‑engine interception for .plm names. Strategic overview: benefits, adoption roadmap, and monetisation framework (non‑normative §20).** |

---

## 2. Core Principles

### 2.1 Why Ternary

Ternary (base‑3) is the natural encoding for classification. Human reasoning defaults to three‑way splits: low/medium/high, yes/partial/no, none/some/full. Binary forces every property into a yes/no dichotomy. Decimal wastes resolution. Ternary matches how the world actually categorizes.

### 2.2 Why 27 Dimensions

Twenty‑seven dimensions arise from seven questions every person asks about anything on a network: Who is behind it? What is it? Where can I find it? When does it operate? Why does it exist? How does it work? Can I trust it? Each question decomposes into 3‑4 measurable properties, yielding 27 total.

### 2.3 The Address IS the Route

In the ternary hypercube, the route between any two nodes is computed by comparing their addresses trit‑by‑trit. Each differing trit corresponds to one hop. The forwarding algorithm is: find the first differing trit, flip it to match, forward to that neighbor. No routing tables. No convergence delays. No longest‑prefix matching. The geometry carries the routing.

### 2.4 The Address IS the Description

Every trit in the address is a deterministic derivation from the entity's measurable properties. The address does not label the entity — it IS the entity's position in ontological space.

If an entity's properties change, its address changes. The coordinate is the derivation, not an approximation of it.

When two entities genuinely share the same 27‑trit coordinate, they genuinely occupy the same point in ontological space. They are not displaced — they are distinguished by their CRD while sharing their natural position.

### 2.5 The Description IS a Measurement

Every dimension in the schema is machine‑deterministic. CRS derives the address by scanning the entity — no human input, no subjective judgment, no opinion. Each trit value corresponds to an observable, testable signal: a protocol handshake, a DNS lookup, a header inspection, a port scan.

**v2.3.2 Principle:** Every input signal is binary (present or absent). Every quantitative derivation counts binary signals and applies the universal formula. There are no continuous scores, no floating‑point boundaries, no tuning parameters. If a machine cannot determine the value from a scan, the dimension does not belong in the address.

### 2.6 Everything Is Encrypted

PlenumNET operates on a zero‑cleartext principle. All traffic between cubes travels through CON tunnels — PQ‑native encrypted channels with CryptoHash key derivation. There is no unencrypted path through the fabric.

Trit 25 ("Is it encrypted?") does not measure whether the fabric encrypts the entity's traffic — the fabric always does. Trit 25 measures what encryption the entity itself offers to its end users: does the entity serve plain HTTP (no), basic TLS (basic TLS), or TLS 1.3 with full hardening headers (full TLS)? This is a property of the entity's design, not of the network transport.

### 2.7 CryptoHash Abstraction (v2.3.2)

All cryptographic hashing in TDNS is accessed through the `CryptoHash` trait:

```rust
trait CryptoHash {
    fn hash(bytes: &[u8]) -> [u8; 32];
    fn keyed_hash(key: &[u8], bytes: &[u8]) -> [u8; 32];
}
```

**Current implementation:** BLAKE3 (256‑bit, quantum‑resistant via Grover bound at ~128‑bit post‑quantum security). Used for: scan hashes, CON tunnel key derivation, integrity fields in wire protocol.

**Target implementation:** TL‑Sponge from the Salvi Framework cryptographic module. TL‑Sponge is the PQ‑native primitive purpose‑built for this system. The interface is identical — only the backend changes. Migration is a single line swap at the crate level.

**Design principle:** Using a general‑purpose encryption API for hashing would be both slower and conceptually wrong. The CryptoHash primitive is specifically optimized for fast hashing, key derivation, and message authentication. The TL‑Sponge provides exceeding military‑grade post‑quantum security within the native ternary mathematical framework.

### 2.8 Design Philosophy

Each dimension answers a question a 12‑year‑old could understand. The three values are obvious and require no training. The questions are the seven that every person asks about anything: **WHO - WHAT - WHERE - WHEN - WHY - HOW - PEACE.**

### 2.9 Geometric Resonance: 27, 28, and the Circle

Beyond the first‑principles derivations, a remarkable numeric harmony emerges from the chosen dimensions — one that resonates with ancient geometric puzzles and suggests a deeper completeness.

Consider a 13‑dimensional hypercube as a conceptual subset of our 27‑dimensional space. Thirteen is not arbitrary; it is half of 26, and 26 appears in the relationship between the circle and its square when pi is taken as 14 — the only value that permits the exact squaring of the circle at 364 degrees.

- A full circle measured in 364 degrees: the quarter‑circle becomes 91 degrees.
- The circumference C of a circle of diameter d is C = π × d. Setting π = 14 gives C = 14d.
- For a circle whose circumference is 364, the diameter is 364 / 14 = 26.
- Now observe: 13 × 28 = 364. Here 13 is half the diameter (26/2 = 13), and 28 is the total number of digits in our full identifier (27 trits + 1 CRD).

Thus, the 13‑dimensional subcube, when extended by the 28th digit, completes a full circle of 364. The Collision Resolution Digit is not merely a tie‑breaker — it is the final degree that brings the geometry full circle.

| Concept               | Value |
|-----------------------|-------|
| Circle measure (degrees) | 364 |
| π (for squaring the circle) | 14 |
| Diameter              | 26 |
| Half‑diameter         | 13 |
| CRD‑augmented digits  | 28 |
| 13 × 28               | 364 |

This is not a coincidence to be dismissed, but a resonance to be appreciated. The universe of discourse, like the circle, returns to itself — and the CRD is the key that closes the loop.

---

## 3. The 27‑Dimensional Schema

### 3.0 Dimension Ordering (Normative)

Dimensions progress from identity (who) through function (what), location (where), temporality (when), purpose (why), mechanics (how), and finally trust (peace). Ordered **most stable first, most dynamic last.**

| Position | Category | Rationale |
|----------|----------|-----------|
| 1st      | WHO (1‑4)   | Entity identity and operator rarely change. |
| 2nd      | WHAT (5‑8)  | Form factor and content type are structural. |
| 3rd      | WHERE (9‑12) | Visibility and access are infrastructure decisions. |
| 4th      | WHEN (13‑16) | Temporal characteristics fixed at design time. |
| 5th      | WHY (17‑20) | Purpose and business model can shift. |
| 6th      | HOW (21‑24) | Delivery patterns may change with architecture. |
| 7th      | PEACE (25‑27) | Security posture is most dynamic. |

Greedy forwarding resolves the most fundamental difference first.

### 3.1 Category Overview

| Category | Trits | Width | Root Question                |
|----------|-------|-------|------------------------------|
| WHO      | 1‑4   | 4     | Who is behind it?            |
| WHAT     | 5‑8   | 4     | What is it?                  |
| WHERE    | 9‑12  | 4     | Where can I find it?         |
| WHEN     | 13‑16 | 4     | When does it operate?        |
| WHY      | 17‑20 | 4     | Why does it exist?           |
| HOW      | 21‑24 | 4     | How does it work?            |
| PEACE    | 25‑27 | 3     | Can I sleep at night?        |

**Total: 27 dimensions = 7.6 trillion addresses**

### 3.2 WHO — Who Is Behind It? (Trits 1‑4)

| Trit | Question          | Value 1    | Value 2    | Value 3       | Type | N |
|------|-------------------|------------|------------|---------------|------|---|
| 1    | **What kind?**    | Personal   | Corporate  | Governance    | CAT  | — |
| 2    | **Who's it for?** | Just me    | My group   | Everyone      | CAT  | — |
| 3    | **Who runs it?**  | Anonymous  | Known      | Transparent   | QNT  | 5 |
| 4    | **Who hosts it?** | Me         | A provider | The cloud     | CAT  | — |

**Trit 3 signals (N=5):** about_page, contact_info, legal_entity, physical_address, gov_domain.

### 3.3 WHAT — What Is It? (Trits 5‑8)

| Trit | Question           | Value 1  | Value 2 | Value 3   | Type | N |
|------|--------------------|----------|---------|-----------|------|---|
| 5    | **What is it?**    | Website  | App     | Device    | CAT  | — |
| 6    | **What's on it?**  | Text     | Media   | Live      | CAT  | — |
| 7    | **Who uses it?**   | People   | Software| Both      | CAT  | — |
| 8    | **Does it think?** | No       | Partly  | Yes       | QNT  | 5 |

**Trit 8 signals (N=5):** ml_endpoints, ml_frameworks, personalization, search_ranking, ml_headers.

### 3.4 WHERE — Where Can I Find It? (Trits 9‑12)

| Trit | Question                | Value 1    | Value 2    | Value 3       | Type | N |
|------|-------------------------|------------|------------|---------------|------|---|
| 9    | **Who can see it?**     | Just me    | My group   | Everyone      | QNT  | 3 |
| 10   | **Do I need to log in?**| No         | Password   | ID Check      | CAT  | — |
| 11   | **How many servers?**   | One        | Several    | Many          | QNT  | 6 |
| 12   | **What connection?**    | HTTP       | WebSocket  | Raw TCP       | CAT  | — |

**Trit 9 signals (N=3):** site_responds, no_auth_challenge, serves_public_content.

**Trit 11 signals (N=6):** dns_resolves, dns_multiple_records, dns_many_records, cdn_provider_header, cdn_cache_header, proxy_via_header.

### 3.5 WHEN — When Does It Operate? (Trits 13‑16)

| Trit | Question                 | Value 1       | Value 2   | Value 3   | Type | N |
|------|--------------------------|---------------|-----------|-----------|------|---|
| 13   | **What era?**            | Pre‑2010      | 2010s     | 2020s+    | QNT  | 6 |
| 14   | **When is it available?**| Business hours| Extended  | 24/7      | QNT  | 3 |
| 15   | **What kind of data?**   | Historical    | Current   | Live      | CAT  | — |
| 16   | **Is it real‑time?**     | Batch         | Near‑time | Real‑time | QNT  | 6 |

**Trit 13 signals (N=6):** alt_svc, permissions_policy, nel_or_report_to, cross_origin_policy, csp, modern_js_framework.

**Trit 14 signals (N=3):** responds, not_maintenance, no_business_hours_language.

**Trit 16 signals (N=6):** dynamic_cache, freshness_indicators, websocket, sse_eventsource, grpc, streaming_content_type.

### 3.6 WHY — Why Does It Exist? (Trits 17‑20)

| Trit | Question               | Value 1    | Value 2      | Value 3     | Type | N |
|------|------------------------|------------|--------------|-------------|------|---|
| 17   | **Does it handle money?**| No        | Accepts      | Processes   | CAT  | — |
| 18   | **Does it want my data?**| No        | Some         | Lots        | QNT  | 5 |
| 19   | **Does it have policies?**| No       | Basic        | Detailed    | QNT  | 5 |
| 20   | **Does it cost money?**| Free       | Pay‑per‑use  | Subscription| CAT  | — |

**Trit 18 signals (N=5):** input_fields, signup_form, analytics_scripts, cookie_consent, crm_scripts.

**Trit 19 signals (N=5):** privacy_page, terms_page, cookie_policy, gdpr_reference, accessibility_statement.

### 3.7 HOW — How Does It Work? (Trits 21‑24)

| Trit | Question                   | Value 1       | Value 2    | Value 3            | Type | N |
|------|----------------------------|---------------|------------|--------------------|------|---|
| 21   | **Who gets it?**           | One person    | A group    | Whoever's closest  | CAT  | — |
| 22   | **Which way does data go?**| Out           | Through    | In                 | CAT  | — |
| 23   | **How do I get updates?**  | I ask         | I subscribe| It tells me        | CAT  | — |
| 24   | **Does it remember me?**   | No            | For a bit  | Always             | QNT  | 3 |

**Trit 24 signals (N=3):** has_any_cookie, has_persistent_cookie, has_long_lived_cookie.

### 3.8 PEACE — Can I Sleep at Night? (Trits 25‑27)

| Trit | Question                | Value 1    | Value 2         | Value 3   | Type | N |
|------|-------------------------|------------|-----------------|-----------|------|---|
| 25   | **Is it encrypted?**    | No         | Basic TLS       | Full TLS  | QNT  | 6 |
| 26   | **How many trackers?**  | Many       | Few             | None      | QNT  | 5 |
| 27   | **Has it been audited?**| No         | Self‑certified  | Audited   | CAT  | — |

**Trit 25 signals (N=6):** tls_present, hsts_header, csp_header, security_txt, x_content_type_options, x_frame_options. NOTE: Measures entity‑level encryption, not CON fabric transport (§2.6).

**Trit 26 CLEAN signals (N=5, INVERTED):** no_analytics, no_social_trackers, no_ad_trackers, no_session_replay, no_crm_trackers. Each signal fires when a tracker category is ABSENT. More clean signals = fewer trackers = higher trit (better trust).

---

## 4. First‑Principle Derivation Mathematics

### 4.0 Justification of Ternary Quantization

The ternary thresholds at exactly 1/3 and 2/3 are not arbitrary but emerge naturally from multiple fundamental mathematical principles. Each of the following independent derivations leads to the same quantization rule.

**4.0.1 Base‑3 Representation.** Treat the proportion p = k/N as a number in [0,1]. Its first digit in base 3 (the most significant ternary digit) is floor(3p), which yields values 0, 1, 2. This is a canonical mathematical operation: extracting the first ternary digit is the most natural way to quantize a continuous value into three levels without any external parameters.

**4.0.2 Maximum‑Entropy Quantization.** If we assume a uniform prior distribution of p over [0,1] (the natural uninformative prior), then to maximize the entropy of the quantized output — i.e., to preserve as much information as possible — the three bins must be equally probable. This forces the cut points to be at 1/3 and 2/3.

**4.0.3 Minimum Mean‑Square Error Quantization.** For a uniform source on [0,1], the optimal scalar quantizer (minimising mean squared error) with three reconstruction levels has decision boundaries at 1/3 and 2/3 and reconstruction levels at the centroids 1/6, 1/2, 5/6. Although we only need the bin indices, the boundaries themselves arise from this optimisation.

**4.0.4 Symmetry and Simplicity.** Requiring the mapping to be symmetric under p → 1‑p (so that "low" and "high" are mirror images) forces the thresholds to be at a and 1‑a for some a. Adding the principle of equal spacing — the simplest symmetric partition — gives a = 1/3. This aligns with the intuitive idea of dividing the unit interval into three equal parts as the most natural unbiased choice.

All four approaches converge to the same clean result, confirming that the ternary quantization is deeply rooted in mathematics, not in arbitrary engineering decisions.

### 4.1 The Universal Derivation Formula

Every quantitative trit is derived from one formula:

```
T = min(floor(3k / N), 2) + 1    in {1, 2, 3}
```

Where:
- k = number of binary signals fired (integer, 0 ≤ k ≤ N)
- N = total possible signals for this dimension (compile‑time constant)
- T = trit value, lifted from GF(3) {0,1,2} to {1,2,3}

The boundaries between trit values fall at exactly N/3 and 2N/3. These are not parameters. They are the definition of dividing a count space into three equal parts. The math determines them.

*Note:* The trit values are stored in wire format as {1,2,3} (with 00 reserved for extension). This lifting avoids a null trit value and simplifies encoding.

### 4.2 The Derivation Type Taxonomy

**CATEGORICAL (15 dimensions).** Scanner produces a pattern string from discrete signal detection. Derivation rule maps it to a trit via pattern match. No quantization needed. Confidence is always 9 (maximum — the signal is unambiguous), unless ambiguity is detected (e.g., multiple matches), in which case the CRS may assign a lower confidence based on internal heuristics (non‑normative).

**QUANTITATIVE (12 dimensions).** Scanner counts binary signals (k out of N). Derivation rule applies project_to_gf3(k, N). Every input signal is binary (present/absent). Confidence is computed from the boundary‑distance formula (§4.3).

### 4.3 Confidence Derivation per Dimension

Each trit carries a confidence digit measuring how far p = k/N lies from the nearest quantization boundary.

Define the distance to the nearest boundary:

```
δ = min(|p − 1/3|, |p − 2/3|)
```

Because p is in [0,1], δ ranges from 0 (on a boundary) to 1/3 (exactly midway between boundaries). Map δ linearly to a confidence digit C in {1, …, 9}:

```
C = min(floor(27 × δ) + 1, 9)
```

- When δ = 0, C = 1 (lowest confidence — on boundary).
- When δ approaches 1/3, C = 9 (highest confidence — mid‑bin).
- 27 is the number of dimensions — the system's own structure determines the scaling.

**Examples:**
- p = 0.5: δ = 0.167, C = min(floor(4.5) + 1, 9) = 5.
- p = 0.34: δ = 0.007, C = min(floor(0.189) + 1, 9) = 1 (very low — near boundary).
- p = 0.9: δ = 0.233, C = min(floor(6.291) + 1, 9) = 7.
- p = 0.0: δ = 0.333, C = min(floor(9) + 1, 9) = 9 (mid‑bin for V1).
- p = 1/3 exactly: δ = 0, C = 1 (on boundary).

For categorical dimensions, confidence is always 9, unless signal ambiguity exists (e.g., a page contains both "personal" and "corporate" indicators). In such cases, the CRS may log a lower confidence, but this is implementation‑specific.

### 4.4 The Confidence Vector

For the full 27‑dimensional scan, we obtain a confidence vector:

```
C = (C₁, C₂, …, C₂₇)    with each Cᵢ in {1,…,9}
```

This vector is stored in the TRN record (optional) and may be used for analytics, debugging, re‑scan prioritization, or as input to higher‑layer policies. Entities with low confidence in one or more dimensions are closer to a boundary and thus more likely to drift on a future re‑scan.

### 4.5 The Collision Resolution Digit (CRD)

- **Range:** 1‑9 (decimal)
- **Assignment:** First‑come‑first‑serve. CRS assigns the smallest integer d in {1,…,9} not already occupied at that address. In case of ties (e.g., simultaneous registrations), the registration with the earlier timestamp (as measured by HPTP) receives the lower CRD; this ensures deterministic assignment.
- **Uniqueness:** For a given 27‑trit address A, every registered entity has a distinct CRD. The pair (A, CRD) is globally unique.

**Assignment algorithm:**

Let S be the set of CRD values already in use for address A. When a new entity with address A registers:

1. If |S| = 9, registration is rejected (no free slot).
2. Otherwise, the entity is assigned the smallest d in {1,…,9} not in S.
3. The TRN record stores A, d, and optionally the confidence vector C.

This assignment is permanent for the lifetime of the registration. When an entity deregisters or drifts to a new address, its CRD slot is freed for a later arrival.

### 4.6 Capacity with CRD

```
3²⁷ × 9 = 7,625,597,484,987 × 9 = 68,630,377,364,883 ~ 6.86 × 10¹³
```

### 4.7 Full Address Format

The full identifier of an entity is the concatenation of its 27‑trit address and its CRD.

- **Wire format:** Two fields packed sequentially:
  - `base_addr`: 27 trits, encoded as 7 bytes (56 bits) using two bits per trit (1→01, 2→10, 3→11; 00 reserved).
  - `crd`: 4‑bit unsigned integer (value 1‑9).
  Total: 7 bytes + 4 bits = 7.5 bytes. Implementations MAY pack into 8 bytes.

### 4.8 Resolution and Routing

- **Geometric routing** uses only the 27‑trit address. The CRD is NOT used for forwarding.
- When a packet arrives at the destination cube, the CRD selects the correct process/container/entity.
- This keeps the routing layer pure and unchanged while adding a thin multiplexing layer at the destination.

### 4.9 Ontological Stability and Slot Management

The confidence vector measures how "solid" each classification is. CRS MAY schedule more frequent re‑scans for entities with low confidence (any Cᵢ = 1). If a re‑scan yields a different address, the entity moves, freeing its CRD slot.

### 4.10 Summary of Mathematical Relationships

For each dimension i:

```
pᵢ = kᵢ / Nᵢ
Tᵢ = min(floor(3 × pᵢ), 2) + 1    in {1, 2, 3}
δᵢ = min(|pᵢ − 1/3|, |pᵢ − 2/3|)
Cᵢ = min(floor(27 × δᵢ) + 1, 9)    in {1, …, 9}
```

The full identifier is (T₁ T₂ … T₂₇) plus a CRD d in {1,…,9}. Total namespace:

```
3²⁷ × 9 = 68,630,377,364,883
```

### 4.11 The Circle Completed

Recall from §2.9: 13 × 28 = 364. The CRD, as the 28th digit, is the final degree that brings the geometry full circle. The universe of discourse, like the circle, returns to itself — and the CRD is the key that closes the loop.

---

## 5. Complete Schema Reference

| Trit | Cat  | Question                | V1         | V2          | V3            | Type | N  |
|------|------|-------------------------|------------|-------------|---------------|------|----|
| 1    | WHO  | What kind?              | Personal   | Corporate   | Governance    | CAT  | —  |
| 2    | WHO  | Who's it for?           | Just me    | My group    | Everyone      | CAT  | —  |
| 3    | WHO  | Who runs it?            | Anonymous  | Known       | Transparent   | QNT  | 5  |
| 4    | WHO  | Who hosts it?           | Me         | A provider  | The cloud     | CAT  | —  |
| 5    | WHAT | What is it?             | Website    | App         | Device        | CAT  | —  |
| 6    | WHAT | What's on it?           | Text       | Media       | Live          | CAT  | —  |
| 7    | WHAT | Who uses it?            | People     | Software    | Both          | CAT  | —  |
| 8    | WHAT | Does it think?          | No         | Partly      | Yes           | QNT  | 5  |
| 9    | WHERE| Who can see it?         | Just me    | My group    | Everyone      | QNT  | 3  |
| 10   | WHERE| Do I need to log in?    | No         | Password    | ID Check      | CAT  | —  |
| 11   | WHERE| How many servers?       | One        | Several     | Many          | QNT  | 6  |
| 12   | WHERE| What connection?        | HTTP       | WebSocket   | Raw TCP       | CAT  | —  |
| 13   | WHEN | What era?               | Pre‑2010   | 2010s       | 2020s+        | QNT  | 6  |
| 14   | WHEN | When is it available?   | Business hours | Extended | 24/7         | QNT  | 3  |
| 15   | WHEN | What kind of data?      | Historical | Current     | Live          | CAT  | —  |
| 16   | WHEN | Is it real‑time?        | Batch      | Near‑time   | Real‑time     | QNT  | 6  |
| 17   | WHY  | Does it handle money?   | No         | Accepts     | Processes     | CAT  | —  |
| 18   | WHY  | Does it want my data?   | No         | Some        | Lots          | QNT  | 5  |
| 19   | WHY  | Does it have policies?  | No         | Basic       | Detailed      | QNT  | 5  |
| 20   | WHY  | Does it cost money?     | Free       | Pay‑per‑use | Subscription  | CAT  | —  |
| 21   | HOW  | Who gets it?            | One person | A group     | Whoever's closest | CAT | — |
| 22   | HOW  | Which way does data go? | Out        | Through     | In            | CAT  | —  |
| 23   | HOW  | How do I get updates?   | I ask      | I subscribe | It tells me   | CAT  | —  |
| 24   | HOW  | Does it remember me?    | No         | For a bit   | Always        | QNT  | 3  |
| 25   | PEACE| Is it encrypted?        | No         | Basic TLS   | Full TLS      | QNT  | 6  |
| 26   | PEACE| How many trackers?      | Many       | Few         | None          | QNT  | 5  |
| 27   | PEACE| Has it been audited?    | No         | Self‑certified | Audited    | CAT  | —  |

**Total: 15 CATEGORICAL + 12 QUANTITATIVE = 27 dimensions**

---

## 6. Quantitative Dimension Signal Definitions (Normative)

Each quantitative dimension defines exactly N binary signals. Each signal is present (1) or absent (0). The sum k is fed to `project_to_gf3(k, N)`. Boundaries are at N/3 and 2N/3. No exceptions.

### 6.1 Trit 3 — Who runs it? (N=5)

| Signal | What it checks |
|--------|---------------|
| 1. about_page | /about or /about‑us returns 200 |
| 2. contact_info | Body contains contact@, mailto:, tel:, phone: |
| 3. legal_entity | Body contains inc., corp., ltd., llc, gmbh, registered |
| 4. physical_address | Body contains street/avenue/road/blvd/drive |
| 5. gov_domain | Domain ends with .gov, .edu, starts with gov., or contains .gov. |

### 6.2 Trit 8 — Does it think? (N=5)

| Signal | What it checks |
|--------|---------------|
| 1. ml_endpoints | Body contains /predict, /inference, /v1/models, /v1/completions, /classify, /embed, /generate, /ai/, /ml/ |
| 2. ml_frameworks | Body contains tensorflow, pytorch, openai, anthropic, hugging, copilot, gpt, llm |
| 3. personalization | Body contains "recommended for you", "you might like", "personalized" |
| 4. search_ranking | Body contains both "search" and "results" |
| 5. ml_headers | Response contains x‑model‑version or x‑inference‑time headers |

### 6.3 Trit 9 — Who can see it? (N=3)

| Signal | What it checks |
|--------|---------------|
| 1. site_responds | HTTP status > 0 |
| 2. no_auth_challenge | HTTP status is not 401 or 403 |
| 3. serves_public_content | HTTP status is 200‑299 |

### 6.4 Trit 11 — How many servers? (N=6)

| Signal | What it checks |
|--------|---------------|
| 1. dns_resolves | DNS A/AAAA record count > 0 |
| 2. dns_multiple | DNS record count > 1 |
| 3. dns_many | DNS record count > 4 |
| 4. cdn_provider | Response contains cf‑ray, x‑cdn, x‑amz‑cf‑id, x‑served‑by, x‑vercel‑id, x‑github‑request‑id, x‑fastly‑request‑id, x‑timer, x‑netlify‑request‑id |
| 5. cdn_cache | Response contains x‑cache or x‑cache‑hits |
| 6. proxy_via | Response contains Via header |

### 6.5 Trit 13 — What era? (N=6)

| Signal | What it checks |
|--------|---------------|
| 1. alt_svc | Response contains alt‑svc header (HTTP/3) |
| 2. permissions_policy | Response contains permissions‑policy header |
| 3. nel_report | Response contains nel or report‑to header |
| 4. cross_origin | Response contains cross‑origin‑embedder‑policy or cross‑origin‑opener‑policy |
| 5. csp | Response contains content‑security‑policy header |
| 6. modern_framework | Body contains __NEXT_DATA__, __NUXT__, svelte, remix, astro, vite |

### 6.6 Trit 14 — When is it available? (N=3)

| Signal | What it checks |
|--------|---------------|
| 1. responds | HTTP status > 0 |
| 2. not_maintenance | HTTP status is not 503 AND body does not contain "maintenance" |
| 3. no_business_hours | Body does not contain "business hours", "office hours", "mon‑fri", etc. |

### 6.7 Trit 16 — Is it real‑time? (N=6)

| Signal | What it checks |
|--------|---------------|
| 1. dynamic_cache | Cache‑Control contains no‑cache or no‑store |
| 2. freshness_indicators | Body contains datetime, timestamp, " ago", or "updated" |
| 3. websocket | Upgrade header present OR body contains "websocket" or "wss://" |
| 4. sse | Content‑Type contains event‑stream OR body contains "eventsource" |
| 5. grpc | Content‑Type contains grpc OR grpc‑status header present |
| 6. streaming | Body contains "real‑time", "realtime", "live feed" OR streaming content type |

### 6.8 Trit 18 — Does it want my data? (N=5)

| Signal | What it checks |
|--------|---------------|
| 1. input_fields | Body contains any `<input>` tags |
| 2. signup_form | Body contains "sign up", "register", or "create account" |
| 3. analytics_scripts | Body contains google‑analytics, gtag, fbevents, facebook.net, hotjar, fullstory, clarity.ms |
| 4. cookie_consent | Body contains "cookie consent" or "cookie preference" |
| 5. crm_scripts | Body contains segment.com, mixpanel, amplitude, intercom, hubspot, drift |

### 6.9 Trit 19 — Does it have policies? (N=5)

| Signal | What it checks |
|--------|---------------|
| 1. privacy_page | /privacy or /privacy‑policy returns 200 |
| 2. terms_page | /terms or /terms‑of‑service returns 200 |
| 3. cookie_policy | Body contains "cookie policy" |
| 4. gdpr_reference | Body contains "gdpr" or "data processing" |
| 5. accessibility | Body contains "accessibility" |

### 6.10 Trit 24 — Does it remember me? (N=3)

| Signal | What it checks |
|--------|---------------|
| 1. has_any_cookie | Set‑Cookie header present |
| 2. has_persistent | Cookie contains expires= or max‑age= |
| 3. has_long_lived | Cookie max‑age > 86400 (1 day) |

### 6.11 Trit 25 — Is it encrypted? (N=6)

| Signal | What it checks |
|--------|---------------|
| 1. tls_present | HTTPS connection succeeded |
| 2. hsts | strict‑transport‑security header present |
| 3. csp | content‑security‑policy header present |
| 4. security_txt | /.well‑known/security.txt or /security.txt returns 200 |
| 5. xcto | x‑content‑type‑options header present |
| 6. xfo | x‑frame‑options header present |

### 6.12 Trit 26 — How many trackers? (N=5, INVERTED)

| Signal | What it checks (ABSENCE = signal fires) |
|--------|----------------------------------------|
| 1. no_analytics | Body does NOT contain google‑analytics, googletagmanager, gtag/js, ga.js, analytics.js |
| 2. no_social | Body does NOT contain facebook.net, fbevents, ads‑twitter.com, linkedin.com/px, analytics.tiktok |
| 3. no_ads | Body does NOT contain doubleclick.net, googlesyndication, googleadservices |
| 4. no_replay | Body does NOT contain hotjar.com, fullstory.com, clarity.ms, mouseflow.com, crazyegg.com |
| 5. no_crm | Body does NOT contain hubspot.com, intercom.io, drift.com, segment.com, mixpanel.com |

---

## 7. Address Notation

### 7.1 Human Name

```
google.plm
pptpro.capomastro.plm
nonnas‑cucina.plm
```

### 7.2 Canonical Wire Format (27‑trit address only)

```
232.311.331.132.233.112.121.232.313
```

### 7.3 Category‑Grouped Debug Format

```
WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313
```

### 7.4 Full Identifier

```
WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313 CRD:1
```

### 7.5 Wildcard Mask Format

```
WO:**** WA:**** WR:**** WN:***3 WY:**** HO:**** PE:***
```

---

## 8. TRN Record Format (Normative)

### 8.1 Required Fields

| Field          | Type          | Description                              |
|----------------|---------------|------------------------------------------|
| `name`         | UTF‑8 string  | Human‑readable name                      |
| `address`      | 27 trits      | Canonical 27‑trit address                |
| `crd`          | u8            | Collision Resolution Digit (1‑9)         |
| `public_key`   | bytes         | Entity's public key                       |
| `ttl`          | u32           | Cache time‑to‑live in seconds            |
| `registered_at`| u64           | HPTP nanosecond timestamp                 |
| `zone`         | UTF‑8 string  | Authoritative zone                        |
| `scan_hash`    | 32 bytes      | CryptoHash of full scan results           |

### 8.2 Optional Fields

| Field               | Type          | Description                              |
|---------------------|---------------|------------------------------------------|
| `confidence`        | 27 bytes      | Confidence vector (each 1‑9)             |
| `valid_from`        | u64           | HPTP nanosecond timestamp                 |
| `valid_until`       | u64           | HPTP nanosecond timestamp                 |
| `hptp_sync_status`  | enum          | synced, degraded, unknown                 |
| `hptp_offset_ns`    | i64           | Last reported HPTP offset                  |
| `attributes`        | map           | The 27 measured values                     |
| `last_rescan`       | u64           | HPTP timestamp of most recent re‑scan      |

### 8.3 Scan Hash Computation (Normative, v2.3.2)

The scan hash MUST include the full raw measurement data, not just derived trit values:

```
hasher = CryptoHash::new()
hasher.update(target_url_bytes)
hasher.update(timestamp_be_bytes)
for each measurement:
    hasher.update(dim_index_byte)
    hasher.update(confidence_byte)
    hasher.update(type_tag_byte)    // 0x01=Text, 0x02=Numeric, 0x03=Boolean, 0x04=Pattern
    hasher.update(value_bytes)
for each trit:
    hasher.update(trit_value_byte)
scan_hash = hasher.finalize()
```

Type tags prevent collision between semantically different values (e.g., Numeric(0.5) vs Pattern("0.5")). Two different entities with different raw observations MUST produce different scan hashes, even if their trit vectors are identical.

**Example:** For dimension 3 (WHO runs it?), a measurement might be: dim_index=3, confidence=7, type_tag=0x04 (Pattern), value_bytes = UTF‑8 bytes of "about_page,contact_info". This ensures the hash uniquely represents the observed signals.

### 8.4 Persistent Storage (v2.3.3)

TRN records, drift events, and redirects are persisted in SQLite. The schema:

```sql
CREATE TABLE trn_records (
    name            TEXT PRIMARY KEY,
    address         TEXT NOT NULL,
    crd             INTEGER NOT NULL,
    public_key      BLOB NOT NULL,
    ttl             INTEGER NOT NULL,
    registered_at   INTEGER NOT NULL,
    zone            TEXT NOT NULL,
    scan_hash       TEXT NOT NULL,
    confidence      TEXT,
    hptp_sync       TEXT,
    hptp_offset_ns  INTEGER,
    last_rescan     INTEGER
);

CREATE TABLE drift_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    old_address     TEXT NOT NULL,
    new_address     TEXT NOT NULL,
    changed_dims    TEXT NOT NULL,
    detected_at     INTEGER NOT NULL
);

CREATE TABLE redirects (
    old_address     TEXT PRIMARY KEY,
    new_address     TEXT NOT NULL,
    expires_ns      INTEGER NOT NULL
);
```

On boot, CRS loads all TRN records from `trn_records` into memory via `import_trn()`, restoring in‑memory indexes and neighbor maps. Registrations and deregistrations are written through to SQLite immediately. The `drift_log` table is append‑only. Expired redirects are purged periodically.

---

## 9. Example Addresses

### 9.1 Google

**Address:** `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313 CRD:1`

### 9.2 PPTPro

**Address:** `WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332 CRD:1`

**HPTP‑mandatory:** Trits 15=3 AND 16=3.

### 9.3 Nonna's Food Blog

**Address:** `WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211 CRD:1`

### 9.4 Ontological Distance

| Pair                     | Differing Trits | Distance |
|--------------------------|-----------------|----------|
| Google ↔ PPTPro          | 19 of 27        | 19 hops  |
| Google ↔ Blog            | 16 of 27        | 16 hops  |
| PPTPro ↔ Blog            | 22 of 27        | 22 hops  |

---

## 10. Self‑Certifying Names

### 10.1 Design

All 27 trits are scan‑derived. No trits are reserved for identity. Public key lives in TRN. Ownership proof via challenge‑response.

### 10.2 Scan Hash Binding

CRS signs the scan hash. Entity proves ownership of name via public_key signature. CRS proves address derivation via scan_hash + CRS signature.

### 10.3 Re‑Verification Protocol (Normative)

Any party MAY request CRS to re‑scan a name. If re‑scan produces a different address, CRS flags as drifted and initiates re‑derivation per §14.

---

## 11. Time‑Aware Resolution (HPTP)

### 11.1 HPTP Live Enforcement Rule (Normative)

When trits 15=3 (Live) AND 16=3 (Real‑time), the entity is HPTP‑mandatory.

- **Registration:** CRS MUST verify sync within tolerance.
- **Runtime:** GLB MUST monitor via FTS. Degraded nodes dropped, routed around.
- **Tolerance:** Real‑time: ≤1µs. Near‑time: ≤100µs.

---

## 12. Geometric Operations

### 12.1 Routing

Compare trit‑by‑trit. Flip first difference. Forward. Path length = Hamming distance. Maximum: 27 hops.

### 12.2 Sparse Routing

54‑entry neighbor map per node. Eventual consistency. Zero convergence time.

### 12.3 Multicast

Wildcard address defines sub‑cube. Natural spanning tree. Zero additional state.

### 12.4 Anycast

Trit 21 = Whoever's closest. Route to nearest match by Hamming distance. Tiebreaker: lowest canonical address.

---

## 13. Integration with Inter‑Cube Services

### 13.1 CRS

Scans entities, derives addresses, computes confidence vectors, assigns CRDs, stores TRN records, maintains neighbor maps, performs re‑scans, prioritizes low‑confidence entities for more frequent scanning.

### 13.2 CON

PQ‑encrypted tunnels (CryptoHash key derivation) between all geometric neighbors. Zero‑cleartext fabric.

### 13.3 FTS

Heartbeats carry HPTP offset. Marks failed/degraded nodes. Sequence anomaly detection.

### 13.4 GLB

Greedy forwarding, sub‑cube multicast, HPTP enforcement, anycast tiebreaker, drift‑redirect.

### 13.5 Metatronic Bridge

`.plm` → TDNS. All else → legacy DNS.

---

## 14. CRS Re‑Scan and Re‑Derivation (Normative)

### 14.1 Re‑scan Policy

Default: weekly. Entities with any Cᵢ = 1: daily.

### 14.2 Property Drift

If re‑scan produces a different address: log drift event to `drift_log` table, assign new CRD at new address (if slot available), install redirect old→new for grace period (default 24h) in `redirects` table, free old CRD slot. The human‑readable name (`google.plm`) does not change — only the address it resolves to.

### 14.3 Forced Re‑derivation Triggers

Entity requests re‑registration. FTS detects prolonged offline. External re‑verification reports mismatch. Any confidence digit = 1.

---

## 15. Scaling Properties

### 15.1 Address Space

| System     | Address Space         |
|------------|-----------------------|
| IPv4       | 4.3 billion           |
| TDNS‑27    | 7.6 trillion          |
| TDNS‑27+CRD| 68.6 trillion         |
| IPv6       | 3.4 × 10³⁸            |

### 15.2 Routing Efficiency

Constant per‑node state (54 entries). Guaranteed worst‑case 27 hops. Zero convergence time. Predictable latency.

### 15.3 Extensibility

Dimension‑agnostic algorithms. 81 dimensions = 4.43 × 10³⁸ (comparable to IPv6). No flag day.

---

## 16. Security Considerations

### 16.1 Encryption Model

Dual‑layer: CON fabric (always on, PQ‑native via CryptoHash) + entity‑level (trit 25).

### 16.2 CryptoHash Migration Path

Current: BLAKE3. Target: TL‑Sponge from Salvi Framework. Interface identical. Single‑line swap at crate level. PQ‑native construction exceeding military‑grade requirements.

### 16.3 CRS as Critical Trust Anchor

Trusted execution, append‑only audit trail, BFT consensus for ordering, HSM key management.

---

## 17. What TDNS Replaces

| Protocol   | Conventional Role        | TDNS Equivalent                          |
|------------|--------------------------|------------------------------------------|
| DNS        | Name → IP                | Name → 27‑trit coordinate + CRD via TRN |
| BGP/OSPF   | Routing tables           | Greedy forwarding; neighbor maps only    |
| PKI/CA     | Certificate authorities  | Challenge‑response + scan hash binding   |
| IGMP/PIM   | Multicast groups         | Sub‑cube via dimensional wildcards       |
| PTP/NTP    | Time synchronization     | HPTP nanosecond timestamps               |

---

## 18. Implementation Status

### 18.1 Completed — TDNS v2.3.3 (Rust Crate)

17 modules, 2 binaries, 185+ tests, zero warnings.

| Module       | Lines | Role                                      |
|--------------|-------|-------------------------------------------|
| trit.rs      | 213   | Atomic ternary digit                       |
| addr.rs      | 465   | 27‑trit CubeAddr + wire encoding           |
| subcube.rs   | 291   | Wildcard multicast                         |
| schema.rs    | 389   | 27 ontological dimensions                  |
| scan.rs      | 259   | Scan types + CryptoHash binding            |
| trn.rs       | 292   | TRN records + CRD + confidence             |
| routing.rs   | 389   | Neighbor maps + greedy forwarding          |
| derive.rs    | 833   | project_to_gf3 + confidence_digit + 27 rules |
| storage.rs   | 378   | SQLite persistence (3 tables)              |
| crs.rs       | 1,109 | CRS registry + CRD assignment + import_trn |
| scanner.rs   | 1,565 | 27 live probes (12 signal‑counting + 15 categorical) |
| glb.rs       | 833   | Geometric Load Balancer                    |
| fts.rs       | 576   | Fault Tolerance Service                    |
| overlay.rs   | 539   | Cube Overlay Network                       |
| api.rs       | 665   | HTTP API (11 endpoints) + SQLite integration |
| bridge.rs    | 419   | Metatronic Bridge                          |
| wire.rs      | 641   | Binary packet framing                      |
| tdns_scan    | 210   | CLI binary                                 |
| tdns_server  | 391   | HTTP server binary (PORT env, SQLite default) |

**Deployment:** Docker + Replit. Tested live against github.com, google.com, wikipedia.org, stripe.com, gov.uk, nytimes.com. SQLite database (`tdns.db`) persists TRN records across restarts.

**Browser extension:** Chromium (Chrome, Edge, Brave, Opera, Vivaldi, Arc) + Firefox. Omnibox (`plm google`), search‑engine interception (`catch-plm.js` on Google/Bing/DuckDuckGo/Yahoo/Brave Search), toolbar popup, full 27‑dimensional resolution page.

---

## 19. Version Comparison

| Aspect                | v2.3                | v2.3.1                       | v2.3.2                                    | **v2.3.3** |
|-----------------------|---------------------|------------------------------|--------------------------------------------|------------|
| Categories            | WHO→PEACE (frozen)  | Same                         | Same                                       | Same |
| Human input           | Zero                | Zero                         | Zero                                       | Zero |
| Automation            | 27/27 stable        | 27/27 stable                 | 27/27 stable                               | 27/27 stable |
| Derivation            | Score‑based (arbitrary) | project_to_gf3 (first principles) | Signal‑count + project_to_gf3 (all dims)  | Same |
| Confidence            | enum {H,M,L}        | Digit 1‑9 (§4.1)           | Digit 1‑9 + categorical always 9            | Same |
| Collision             | Displacement         | CRD 1‑9                     | CRD 1‑9 (tie‑break by timestamp)           | Same |
| scan_hash             | dim + conf only      | Same                         | Full raw values with type tags              | Same |
| CryptoHash            | BLAKE3 hardcoded    | BLAKE3 hardcoded             | CryptoHash trait (BLAKE3 → TL‑Sponge)      | Same |
| Scanner               | Mix scores/counts   | Signal counts (4 dims)       | Signal counts (all 12 quantitative dims)    | Same |
| Storage               | Memory only          | Memory only                  | Memory only                                | **SQLite persistent** |
| Browser extension     | —                    | —                            | —                                          | **Chromium + Firefox** |
| Encryption            | Dual‑layer          | Same                         | Same                                       | Same |
| Status                | Frozen              | v2.3.1                       | v2.3.2                                     | **v2.3.3** |

---

## 20. Strategic Overview: Benefits

*This section is non‑normative and provides a business perspective.*

### 20.1 Key Benefits

| Benefit                         | How It Works                                                                 | Impact                                                                 |
|---------------------------------|------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| **No routing tables, no convergence** | Greedy trit‑flip forwarding in a 27‑D hypercube; each node only knows its 54 neighbors. | Zero convergence delays, fixed memory footprint, predictable latency. |
| **Self‑certifying, deterministic names** | Address derived solely from measurable properties; no human‑chosen names, no squatting. | Trust without central authority; addresses always truthful.           |
| **Built‑in security**           | All inter‑cube traffic encrypted via CON tunnels (PQ‑native). Trit 25 measures entity's own encryption. | Defense in depth; quantum‑ready fabric.                               |
| **Automatic classification**    | Scanner produces address without human input; 27 dimensions answer seven basic questions. | No manual tagging; continuous compliance monitoring.                  |
| **Unified protocol stack**      | One geometry handles naming, routing, identity, multicast, and time sync.    | Drastic simplification of network architecture.                       |
| **Massive scalability**         | 7.6T base addresses ×9 CRD = 68.6T unique IDs; max 27 hops.                  | Future‑proof; can expand to 81 dimensions if needed.                  |
| **Resilience and fault tolerance** | FTS monitors HPTP offsets and node health, automatically rerouting around failures. | High availability without complex failover protocols.                 |
| **Quantum‑ready cryptography**  | CryptoHash abstraction allows swapping BLAKE3 for TL‑Sponge (ternary PQ primitive). | Long‑term security; migration is a single line change.                |
| **Automated compliance & auditing** | Scanner continuously verifies properties (e.g., GDPR via trit 19, PCI via trit 17). | Entities get "self‑auditing" addresses; reduces regulatory burden.   |

---

© 2026 Capomastro Holdings Ltd. All rights reserved.

*The Salvi Framework — Applied Physics Division*

*Simple. Measurable. Automatable. Enforced. Deterministic. Derived. Confident. Unique.*

*Three formulas. Zero thresholds. One geometry.*