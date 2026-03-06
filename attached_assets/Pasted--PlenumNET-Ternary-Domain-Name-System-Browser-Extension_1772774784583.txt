# PlenumNET — Ternary Domain Name System
## Browser Extension Diagnostic Report — Complete Product Specification

**Version:** 1.4.1  
**Previous Version:** 1.4 (March 2026)  
**Classification:** Internal — Product & Engineering  
**Organisation:** Capomastro Holdings Ltd. — Applied Physics Division — Alberta, Canada  
**Date:** March 2026  
**Status:** Patent(s) Pending — All Rights Reserved — Production-Ready — Codebase-Verified

---

> *Three formulas. Zero thresholds. One geometry.*

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Competitive Landscape](#2-competitive-landscape)
3. [Report Section Specifications](#3-report-section-specifications)
4. [Mathematical Foundation](#4-mathematical-foundation)
5. [Product Tiers & Monetisation](#5-product-tiers--monetisation)
6. [Development Roadmap](#6-development-roadmap)
7. [Report Design Standards](#7-report-design-standards)
- [Appendix A — 81 Dimension Meaning Sentences](#appendix-a--81-dimension-meaning-sentences)
- [Appendix B — Sample Calculations](#appendix-b--sample-calculations)
- [Changelog](#changelog-v12--v13)

---

## 1. Executive Summary

The PlenumNET TDNS Browser Extension produces the world's first **27-dimensional ontological network intelligence report** — a mathematically rigorous, cryptographically hashed analysis of any website or network entity, delivered in under ten seconds from a browser extension. Every value is derived from first principles using the GF(3) ternary quantization formula. There are no simulated values, no third-party intelligence feeds, no black-box scores, and no arbitrary thresholds. The mathematics determine the boundaries.

No competitor produces anything comparable. Qualys SSL Labs audits TLS configuration. SecurityHeaders.com checks seven headers. Shodan indexes ports and banners. BuiltWith identifies marketing technology. Lighthouse measures performance. **PlenumNET examines all seven ontological axes simultaneously** — WHO, WHAT, WHERE, WHEN, WHY, HOW, and PEACE — and encodes everything into a 27-trit canonical address. That address is the product. The report is the evidence supporting it.

> **The address `WO:2323 WA:1111 WR:3121 WN:3322 WY:1111 HO:3113 PE:231` contains more machine-readable intelligence about an entity than any competing tool produces in total. It can be stored, compared, queried, and signed. It is a fingerprint, an assessment, and a credential simultaneously.**

---

### 1.1 Target Audiences and Value Propositions

*v1.1: Expanded with measurable outcomes across time saved, risk coverage, and compliance value.*

| Audience | Primary Question | Value Delivered | Time Saved | Risk / Compliance Gain |
|----------|-----------------|-----------------|------------|------------------------|
| **Individual / Consumer** | Is this site safe? Is it tracking me? | One-click answer across all 27 dimensions. Plain-English findings and remediation. | ~4 hours vs. manual header checks ¹ | Detects all 5 tracker categories including session replay (credential risk) |
| **Security Engineer** | What is our exact header posture? What changed? | Definitive header audit, severity-ranked findings with exact remediation, BLAKE3 change detection, trit-level regression tracing. | ~2 hours per site vs. manual audits ² | Trit-level change detection catches regressions tools like Nagios miss |
| **Enterprise Procurement** | Which of our 200 vendors are the riskiest? | Bulk scan, Trust Score ranking, vendor risk report, exportable address matrix for compliance registers. | Weeks → minutes for 200-vendor assessment ³ | Maps directly to ISO 27001 Annex A.15 supplier assessment |
| **Developer / Architect** | How do we score against the 27-dimensional standard? | Exact signal counts per dimension, confidence scores, specific header values, remediation priority order. | Immediate vs. multi-tool stitching | Single source of truth for sprint security tasks |
| **M&A / Due Diligence** | What is the infrastructure maturity of this target? | Complexity Score, Maturity Score, tech stack fingerprint, infrastructure topology. | Weeks of consultancy → seconds | Deterministic address enables before/after integration comparison |
| **Researcher / Journalist** | Can I fingerprint and track this infrastructure over time? | Deterministic address enables correlation across scans. Hash-based change detection. Timestamped attestation (roadmap). | Immediate vs. Shodan + manual analysis | BLAKE3 hash chain is admissible evidence of infrastructure state |
| **Compliance Officer** | Does this vendor meet our security baseline? | PEACE scores, header audit, policy detection (D19), audit status (D27) — all in one exportable report. | ~1 hour per vendor → <1 minute | GDPR/PIPEDA-relevant findings (D18, D19, D24, D26) are explicitly flagged |

> **Time-saved methodology (v1.2):**
> ¹ *Individual:* Estimated from average time to manually check 12 security headers (SecurityHeaders.com), identify trackers (browser DevTools Network tab), and interpret cookie flags — approximately 15–20 minutes per site × 12–16 sites in a typical browsing session.
> ² *Security Engineer:* Based on industry benchmark of ~2 hours per site for a manual header audit, cookie review, tracker enumeration, and findings writeup without tooling. Single-tool figure assumes no cross-referencing.
> ³ *Enterprise Procurement:* Based on manual vendor security questionnaire baseline of 2–4 hours per vendor (NIST SP 800-161 supplier assessment guidance). 200 vendors × 2 hours = 400 person-hours. Bulk TDNS scan at 10 scans/minute = 20 minutes for equivalent initial triage.

---

### 1.2 Scope, Limitations, and Trust Disclaimer

*v1.1: Added per reviewer recommendation. Builds institutional trust without undermining product confidence.*

PlenumNET TDNS scans are **deterministic at the moment of execution** — the same server state will always produce the same address. However, web entities are dynamic. The following limitations apply:

- **Static fetch only (v2.3.x):** The server-side scanner reads the initial HTTP response. JavaScript-rendered content, dynamically injected scripts, and post-login pages are not analysed. Dynamic tracker injection (e.g., Google Tag Manager firing after page load) may be missed. This is disclosed per-dimension via the confidence (C) value — low C on body-derived dimensions signals this limitation.
- **Point-in-time snapshot:** A scan reflects the entity's state at the exact timestamp shown. Critical findings should be verified before formal reporting. For legal or compliance use, use the TSA-attested scan (Phase 4) which provides a signed, non-repudiable timestamp.
- **Server-to-server fetch context:** The PlenumNET server fetches the target using a browser-like User-Agent. Some entities serve different headers to known scanners or from specific geographies. Results may differ slightly from a browser fetch.
- **Body signal confidence:** Dimensions derived from body content (D3, D8, D18, D19) carry confidence values. C ≤ 5 on a body-derived dimension indicates the signal count is near a trit boundary and the result should be treated as indicative, not definitive.

> **Verify critical findings independently before actioning remediation or filing compliance reports. The TDNS address is intelligence, not certification.**

---

## 2. Competitive Landscape

*v1.1: Added Mozilla Observatory, TruffleHog, and synergy notes. Quantitative benchmarks added.*

### 2.1 Feature Comparison Matrix

| Capability | PlenumNET | SSL Labs | Sec Headers | Shodan | BuiltWith | Lighthouse | Mozilla Obs. | TruffleHog |
|-----------|:---------:|:--------:|:-----------:|:------:|:---------:|:----------:|:------------:|:----------:|
| 27-dimensional ontological address | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| GF(3) mathematical derivation | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Cryptographic scan hash | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Confidence score per dimension | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Five composite intelligence scores | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Security header audit (12 headers) | ✓ | ✗ | ✓ | ✗ | ✗ | Partial | ✓ | ✗ |
| Privacy & tracker analysis (5 cats) | ✓ | ✗ | ✗ | ✗ | Partial | ✗ | ✗ | ✗ |
| Cookie intelligence (flags + SameSite) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Business model detection (D17, D20) | ✓ | ✗ | ✗ | ✗ | Partial | ✗ | ✗ | ✗ |
| Infrastructure topology (D11, D21) | ✓ | ✗ | ✗ | Partial | ✗ | ✗ | ✗ | ✗ |
| Technology era classification (D13) | ✓ | ✗ | Partial | ✗ | Partial | ✗ | Partial | ✗ |
| Real-time capability detection (D16) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| AI / ML signal detection (D8) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Data collection appetite (D18) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Operator transparency (D3) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Session persistence analysis (D24) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| DNSSEC / CAA record analysis | ◉ | Partial | ✗ | Partial | ✗ | ✗ | ✓ | ✗ |
| TLS certificate depth (issuer, CT log) | ◉ | ✓ | ✗ | Partial | ✗ | ✗ | ✓ | ✗ |
| CVE / vulnerability lookup | ◉ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Secrets / credential scanning | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Entity registration (.plm address) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Change detection & alerting | ◉ | ✗ | ✗ | Paid | ✗ | ✗ | ✗ | ✗ |
| Post-quantum cryptographic hash | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Works on any URL, no account required | ✓ | ✓ | ✓ | ✗ | Paid | ✓ | ✓ | ✗ |
| Average scan time | **<10s** | ~60s | ~5s | minutes | minutes | ~30s | ~15s | minutes |

*✓ = Available · ✗ = Not available · ◉ = On roadmap · Partial = Limited implementation · Average scan time row is median across 10 representative public sites.*

> **Legend (v1.2 — pinned for print/PDF visibility):** ✓ Full implementation confirmed · ✗ Feature does not exist · ◉ Formally on development roadmap · Partial = capability exists but limited in scope, accuracy, or availability

### 2.2 Synergy Notes

*PlenumNET is not a replacement for every tool — it is the ontological layer that gives context to what other tools return.*

- **PlenumNET + Shodan:** Shodan identifies open ports and banners. PlenumNET provides the ontological context — who operates this entity (D3), what era its stack is (D13), and how it fits into a trust framework (Trust Score). Combined, they produce a complete entity profile.
- **PlenumNET + Mozilla Observatory:** Observatory focuses on TLS depth and cookie policy. PlenumNET's 27-dimension address contextualises Observatory's findings within the full WHO/WHAT/WHERE/WHEN/WHY/HOW/PEACE framework. The two are complementary, not competing.
- **PlenumNET + TruffleHog:** TruffleHog scans source code and repos for exposed secrets. PlenumNET scans the live HTTP surface. A high-risk TDNS address (low D25, anonymous D3) is a signal that deeper secrets scanning via TruffleHog is warranted.

---

## 3. Report Section Specifications

The Diagnostic Report opens as a full browser tab from the extension popup. Sections are mandatory and rendered in the order listed. A user must form a complete high-level assessment within 30 seconds. Granular evidence for every claim is accessible by scrolling.

---

### 3.1 Report Header

**Purpose:** Immediate identification. The user knows exactly what they are looking at within one second of the page loading.

| Element | Specification |
|---------|--------------|
| **Hostname** | Large (26px), gold `#D4A017`, monospace. e.g. `www.google.com`. Primary label. |
| **Full URL** | Smaller (12px), muted `#5A5548`. Confirms the exact page scanned including protocol and path. |
| **Scan Timestamp** | ISO 8601 UTC display derived from HPTP femtosecond timestamp. Source: `getFemtosecondTimestamp()` (`salvi-core/femtosecond-timing.ts`). Wire value: 128-bit femtoseconds since Salvi Epoch (2025-04-01T00:00:00Z). Display example: `2026-03-06T02:34:33.544Z`. Legal defensibility requires sub-millisecond precision — HPTP provides it. |
| **Extension Version** | `vX.X.X` — for reproducibility, support, and regression tracking. |
| **Engine** | `PlenumNET TDNS Engine vX.X.X` — confirms server-side scan; no local approximation. |
| **SCANNED / REGISTERED badge** | SCANNED (grey) = address derived. REGISTERED (gold) = canonical `.plm` entry exists in PlenumNET registry. |
| **HPTP badge** | Displayed only when D15=3 AND D16=3. Amber. Text: `HPTP Sync Required`. Links to HPTP documentation. |
| **CRD badge** | Collision Resolution Digit (1–9). Inline. Green background. See §3.2 for collision resolution spec. |
| **Rescan button** *(v1.1)* | Triggers immediate re-scan. Compares new BLAKE3 hash against stored previous hash. If hashes differ, a diff panel shows which trits changed, in which direction, and what the change means. Available on all tiers (1 rescan/day on Free, unlimited on Pro+). |

---

### 3.2 Canonical TDNS Address Block

**Purpose:** The primary output of every scan. Visually dominant, immediately readable, copyable in one click.

> The address `WO:2323 WA:1111 WR:3121 WN:3322 WY:1111 HO:3113 PE:231` encodes 27 ternary decisions about this entity. A changed address is evidence of infrastructure change. An identical address across two scans confirms consistency. The BLAKE3 hash of the full serialised scan measurement vector is the canonical identifier.

| Element | Specification |
|---------|--------------|
| **Segment display** | Seven chips: `WO:XXXX · WA:XXXX · WR:XXXX · WN:XXXX · WY:XXXX · HO:XXXX · PE:XXX`. Each chip uses its axis accent colour. Trit digits in monospace bold. |
| **Axis labels** | WHO / WHAT / WHERE / WHEN / WHY / HOW / PEACE in small caps above each chip. |
| **CRD badge** | `CRD:N` to the right of the address. Green `#059669`. |
| **Copy button** | One click copies full canonical address string. Confirmation toast on copy. |
| **Canonical string** | Full 27-digit string in monospace below segment display for machine-readable reference. |
| **BLAKE3 hash preview** | First 16 characters with `...` suffix inline. Full hash in §3.11 (Wire Encoding). |
| **Decode Address tool** *(v1.1)* | Input field accepting any valid TDNS address string. Renders a simulated full breakdown (without scanning) showing trit values, labels, and meanings for each dimension. Useful for education, competitor analysis, or reviewing historical addresses. Available on all tiers. |

#### Address Collision Resolution (v1.1 — Formally Specified)

Two distinct entities can theoretically share the same 27-trit address if their observable HTTP signals are identical. The CRD resolves this.

**CRD derivation:**
```
CRD = (BLAKE3(trit_vector)[0] mod 9) + 1
```

**Collision exhaustion protocol:** If a `.plm` registration is attempted for an address+CRD combination that already exists in the registry, the system automatically increments CRD. When CRD reaches 9 and all are occupied, a **secondary CRD** (CRD2) is derived:
```
CRD2 = (BLAKE3(trit_vector + registered_at_timestamp)[0] mod 9) + 1
```
This is displayed as `CRD:9.3` — primary.secondary notation. In practice, collision exhaustion is astronomically unlikely given 3²⁷ = ~7.6 trillion possible addresses.

> **Implementation requirement (v1.2):** The `registered_at_timestamp` used in the CRD2 derivation **must be the Unix epoch millisecond recorded at the moment the registration is persisted to the registry** — not the scan timestamp, not the request receipt time. This value must be stored immutably alongside the registry entry. If the registry record is migrated, replicated, or restored, the original `registered_at_timestamp` must be preserved verbatim. CRD2 is non-deterministic without this guarantee. Store as: `registered_at_ms: number` (integer, UTC milliseconds since epoch).

**Bulk address comparison via API:**
```
POST /api/tdns/compare
Body: { "addresses": ["WO:2323...", "WO:2312..."] }
Returns: { "diff": [{ "dimension": 3, "a": 2, "b": 1, "meaning": "..." }] }
```

---

### 3.3 Five Intelligence Scores

**Purpose:** Executive summary in five numbers, each 0–100, derived from the address using the same GF(3) formula that produced it.

#### Score Definitions

| Score | Source Dimensions | What It Measures | Formula |
|-------|------------------|-----------------|---------|
| **Trust Score** | D1, D2, D3, D4 (WHO) | Accountability, transparency, and legitimacy of the entity behind the domain. | `gf3(Σ WHO, 12) × 100 / 3` |
| **Security Score** | D25, D27, D13, D9, D11 | Defensive depth: encryption posture, audit status, technology era, visibility, scale. | `gf3(Σ SEC, 15) × 100 / 3` |
| **Privacy Score** | D18⁻, D19, D24⁻, D26 | Data collection appetite, policy presence, session persistence, tracker density. ⁻ = inverted. | `gf3(Σ PRIV, 12) × 100 / 3` |
| **Maturity Score** | D13, D14, D15, D16 (WHEN) | Technology era, availability, data freshness, real-time capability. | `gf3(Σ WHEN, 12) × 100 / 3` |
| **Complexity Score** | D5, D7, D11, D21, D22 | Architectural sophistication. Relevant to M&A and vendor assessment. | `gf3(Σ COMP, 15) × 100 / 3` |
| **Trust Index** | All five scores | Composite headline number. Weighted by axis independence (see §3.3.1). | `0.35T + 0.30S + 0.20P + 0.10M + 0.05C` |

#### 3.3.1 Trust Index Weighting — Mathematical Justification (v1.1)

*v1.0 listed weights without justification. v1.1 derives them from GF(3) axis independence and decision-theoretic primacy.*

The Trust Index weights reflect two properties: **axis independence** (dimensions that share few signals with other axes get higher weight) and **decision primacy** (what a rational agent needs to know first before engaging with an entity).

**Axis independence analysis:**

- **WHO (D1–D4)** feeds zero dimensions shared with other axes. It is entirely derived from operator identity signals — legal entity, about pages, physical address. No overlap. Highest weight: **0.35**.
- **PEACE (D25–D27)** shares D13 and D11 with Security Score and Maturity Score. Despite overlap, encryption posture is the most binary risk signal — TLS absent is a blocker regardless of other scores. Weight: **0.30**.
- **WHY (D18, D19, D24, D26)** — Privacy Score. D26 is independent; D18 and D24 share body-scan signals with WHERE and WHEN. Moderate independence. Weight: **0.20**.
- **WHEN (D13–D16)** — Maturity Score. D13 is shared with Security Score. Lower independence. Weight: **0.10**.
- **WHAT+HOW subset (Complexity)** — D5, D7, D11, D21, D22. High signal overlap with WHERE and HOW. Complexity is informational rather than risk-bearing for most use cases. Lowest weight: **0.05**.

**Verification:** The weights sum to 1.0. The weighting degrades gracefully — even if all three of Security, Privacy, and Maturity are Perfect (100), a Critical Trust Score (0) produces a Trust Index of 35, which correctly classifies as Poor. An anonymous operator with excellent security headers is not trustworthy.

#### 3.3.2 Privacy-Focused Index (Pro Feature — v1.1, revised v1.2)

A variant Trust Index for consumers who prioritise privacy over security posture:

```
Privacy-Focused Index = 0.40P + 0.30T + 0.20S + 0.10M
```

**AI/ML tiebreaker — GF(3)-derived (v1.2 revision):** The v1.1 tiebreaker (`+5 if D8=1, -5 if D8=3`) used arbitrary constants. v1.2 derives the modifier from the same ternary geometry as all other values:

```
AI_modifier = (2 - D8) × gf3_unit
  where gf3_unit = 100 / 9 ≈ 11.1   (one full confidence unit on the 0-100 scale)

→  D8=1: modifier = (2-1) × 11.1 = +11.1  (no AI = privacy positive)
→  D8=2: modifier = (2-2) × 11.1 =   0.0  (partial AI = neutral)
→  D8=3: modifier = (2-3) × 11.1 = -11.1  (heavy AI = privacy negative)
```

The `gf3_unit` (100/9) is not an arbitrary constant — it is the width of one confidence pip on the normalised 0–100 scale, derived from the 9-pip confidence bar. The modifier is capped to prevent the Privacy-Focused Index from exceeding [0, 100]:

```
Privacy-Focused Index (final) = clamp(0.40P + 0.30T + 0.20S + 0.10M + AI_modifier, 0, 100)
```

Displayed as an alternative view toggle in Pro tier. Label: `Privacy-Focused Index (PFI)`.

#### Score Labels

| Range | Label | Colour | Meaning |
|-------|-------|--------|---------|
| 90–100 | **Excellent** | `#059669` | Best-in-class. Exceeds all standards. |
| 75–89 | **Good** | `#34D399` | Strong posture with minor gaps. |
| 50–74 | **Fair** | `#D4A017` | Functional with notable deficiencies. |
| 25–49 | **Poor** | `#F97316` | Significant problems requiring attention. |
| 0–24 | **Critical** | `#DC2626` | Active risk. Immediate remediation required. |

> **Polarity note:** `inv(d) = 4 − d` is applied to D18 (data appetite) and D24 (session persistence). Higher trit on those dimensions represents a worse privacy outcome. Inversion preserves GF(3) consistency while correcting axis polarity. See Appendix B for worked examples.

---

### 3.4 Infrastructure Intelligence Panel

**Purpose:** Raw technical evidence from the HTTP exchange. The data layer beneath the scores.

| Field | Source Signal | Intelligence Value |
|-------|--------------|-------------------|
| **Protocol** | URL scheme + TLS negotiation | HTTP in 2026 is a Critical finding. TLS version shown where available. |
| **HTTP Status Code** | Response status line | 200=healthy, 301/302=redirect chain, 403=access controlled, 503=degraded. Affects D9 and D14. |
| **Server** | `Server:` and `X-Powered-By:` headers | Stack fingerprinting. Version exposure is a Warning finding. |
| **Content-Type** | `Content-Type:` + charset | Confirms form factor. HTML=website, JSON=API. Drives D5 and D7. |
| **CDN / Edge Provider** | `cf-ray`, `x-served-by`, `x-amz-cf-id`, `via` | Infrastructure scale and distribution. Drives D11 and D21. |
| **Cache-Control** | Full directive string | Content delivery strategy. `max-age=0` signals real-time intent. Drives D15, D16. |
| **CORS Policy** | `Access-Control-Allow-Origin:` | API architecture signal. Wildcard (`*`) = open surface. |
| **HTTP/3 (Alt-Svc)** | Full `Alt-Svc:` value | Protocol modernity. HTTP/3 = 2020s stack confirmed. Drives D13. |
| **Via / Proxy Chain** | `Via:` header | Reveals intermediaries. Relevant to D21 and D22. |
| **Response Time (TTFB)** | Time to first byte from server-side fetch | Sub-100ms = edge delivery. Over 2s = origin-only. Not affected by user's network. |
| **Body Scanned** | Bytes of response body analysed | Data quality indicator. Below 5KB = low confidence on body-derived dimensions. |
| **Compression** | `Content-Encoding: gzip / br / zstd` | brotli = modern stack signal. Absent = potential performance concern. |
| **WebSocket** | `Upgrade: websocket` header | Real-time capability. Drives D12 and D16. |
| **Server-Sent Events** | `Content-Type: text/event-stream` | Push-based updates. Drives D16 and D23. |
| **DNSSEC Status** *(v1.1)* | DNS AD flag on A/AAAA query | DNSSEC signed = operator controls DNS chain of trust. Absent = DNS spoofing risk. Contributes to D3 signal count. |
| **CAA Records** *(v1.1)* | DNS CAA record query | Certificate Authority Authorisation present = only named CAs can issue certs. Absent on HTTPS site = Warning finding. |
| **IP Resolution** *(v1.1 roadmap)* | DNS A record + ASN lookup | Hosting provider, country of jurisdiction, whether IP is in known-bad ASN list. |

#### Infrastructure Topology Map (Phase 2 — v1.4.1 Full Specification)

**Purpose:** Visual representation of the delivery chain inferred entirely from public HTTP response signals. Reveals CDN layers, proxy/WAF presence, and origin exposure without active probing. Delivered as a static SVG from the TDNS server.

##### Node Taxonomy

Every topology map contains exactly four node slots. Nodes are populated or shown as absent based on detected signals.

| Node | Label | Populated When | Absent Behaviour |
|------|-------|---------------|-----------------|
| **Client** | `[You]` | Always present | — |
| **CDN / Edge** | Provider name (e.g., `Cloudflare`, `Fastly`, `AWS CloudFront`) | `cf-ray`, `x-served-by`, `x-amz-cf-id`, `x-cache: HIT`, `alt-svc: h3` detected | Dashed circle labelled `No CDN detected` |
| **Proxy / WAF** | `Proxy` or WAF name if identifiable | `via` header present; `x-sucuri-id`; `x-waf-*`; Imperva/Akamai signatures | Dashed circle labelled `No proxy layer` |
| **Origin** | `Origin Server` + server type if disclosed | Always present — every HTTP response has an origin | Solid circle. `Server:` value shown if present; `[Hidden]` if absent. |

**Layout:** Left-to-right horizontal flow. Absent nodes rendered dashed with no connecting arrow.

Examples:

```
Full chain:
  [You] ──→── [Cloudflare Edge] ──→── [Imperva WAF] ──→── [Origin: nginx]

CDN only:
  [You] ──→── [Cloudflare Edge] ──→── [Origin: nginx]

Bare origin (warning state):
  [You] ──→── [Origin: Apache/2.4.51]
               ⚠ No CDN — origin IP directly exposed
```

The bare-origin warning generates a standalone **Warning finding**: "No CDN or proxy layer detected. Origin server is directly reachable from the public internet. DDoS mitigation depends entirely on origin capacity."

##### Signal-to-Node Mapping

| HTTP Signal | Node Populated | Label |
|-------------|---------------|-------|
| `cf-ray` present | CDN | Cloudflare |
| `x-served-by: cache-*` | CDN | Fastly |
| `x-amz-cf-id` present | CDN | AWS CloudFront |
| `server: AkamaiGHost` | CDN | Akamai |
| `x-cache: HIT from *.cdn.*` | CDN | (provider from hostname) |
| `alt-svc: h3=` (no other CDN signal) | CDN | Edge (HTTP/3) |
| `via: 1.1 varnish` | Proxy | Varnish |
| `via: 1.1 squid` | Proxy | Squid |
| `x-sucuri-id` | Proxy | Sucuri WAF |
| `x-waf-*` any value | Proxy | WAF |
| `via` with non-RFC-1918 hostname | Proxy | Proxy (hostname) |

> **Privacy constraint (v1.2):** SVG must not include resolved IP addresses of CDN edge nodes or origin servers. `Via` header segments matching RFC-1918 ranges (`10.x.x.x`, `192.168.x.x`, `172.16-31.x.x`, `127.x.x.x`) are stripped before rendering. Only provider names derived from public header signatures may appear.

##### SVG Generation Contract

- Generated server-side by the TDNS Rust service. Returned as UTF-8 string in scan JSON field `topology_svg`.
- Viewport: `viewBox="0 0 600 120"`. Scales responsively via `width: 100%` CSS on the container.
- No `<script>` elements. No external resource references. SVG sanitised before delivery to client.
- Node circles: `r=28`, stroke `#2E75B6`, fill `#F5F5F5`. Absent nodes: `stroke-dasharray="6 3"`, fill `#FAFAFA`.
- Arrows: `marker-end` arrowhead, colour `#C5943A`. Arrows connecting absent nodes are omitted entirely.
- Provider name: `font-family: monospace`, `font-size: 11`, centred below node circle.
- Warning annotation (bare-origin): `fill: #C0392B`, `font-size: 10`, below the origin node.

##### Topology — TDNS Dimension Linkage

| Topology Signal | Dimensions Affected |
|----------------|-------------------|
| CDN detected | D11 (Infrastructure Scale) ↑, D21 (Delivery Mode) |
| No CDN | D11 ↓, Warning finding generated |
| WAF / Proxy present | D22 (Delivery Security) ↑ |
| Multi-hop `Via` chain | D21 → Relay trit |
| HTTP/3 (`alt-svc`) | D13 (Technology Era) → 2020s+ trit |
| Server version visible in Origin label | D14 (Lifecycle), CVE check trigger |

> **Phase 2 delivery note (v1.4.1):** The Topology Map is delivered in Phase 2 (TDNS v2.4). In Phase 1, the `topology_svg` field in scan JSON is `null`. The extension must handle `null` gracefully: render the Infrastructure panel without the map, display `"Topology map available in Extension v1.1"` in the map slot. The four-node taxonomy and signal-to-node mapping above constitute the full implementation contract for `services/tdns-v2/src/topology.rs`.
---

### 3.5 Security Header Audit

**Purpose:** The definitive HTTP security header checklist. Every header shown regardless of presence. Exact values shown (truncated at 80 chars). Plain-English explanation with OWASP reference links. Directly linked to TDNS dimensions.

| Header | Status | Example Value | Purpose | TDNS Dim | OWASP Ref |
|--------|:------:|--------------|---------|----------|-----------|
| `Strict-Transport-Security` | ✓ | `max-age=31536000; includeSubDomains; preload` | Forces HTTPS for 1 year. `preload` = hardcoded in browser. | D25 | [OWASP HSTS](https://owasp.org/www-project-secure-headers/#strict-transport-security) |
| `Content-Security-Policy` | ✗ | missing | Without CSP, XSS attacks are trivially exploitable. | D25, D13 | [OWASP CSP](https://owasp.org/www-project-secure-headers/#content-security-policy) |
| `X-Content-Type-Options` | ✓ | `nosniff` | Prevents MIME sniffing. | D25 | [OWASP XCTO](https://owasp.org/www-project-secure-headers/#x-content-type-options) |
| `X-Frame-Options` | ✓ | `SAMEORIGIN` | Blocks clickjacking via iframe embedding. | D25 | [OWASP XFO](https://owasp.org/www-project-secure-headers/#x-frame-options) |
| `Permissions-Policy` | ✓ | `camera=(), microphone=()` | Restricts browser API access. | D13 | [OWASP PP](https://owasp.org/www-project-secure-headers/#permissions-policy) |
| `Cross-Origin-Opener-Policy` | ✗ | missing | Isolates browsing context. Spectre mitigation. | D13 | [OWASP COOP](https://owasp.org/www-project-secure-headers/#cross-origin-opener-policy) |
| `Cross-Origin-Embedder-Policy` | ✗ | missing | Required with COOP for SharedArrayBuffer. | D13 | [OWASP COEP](https://owasp.org/www-project-secure-headers/#cross-origin-embedder-policy) |
| `Cross-Origin-Resource-Policy` | ✗ | missing | Prevents cross-origin resource loading. | D13 | [OWASP CORP](https://owasp.org/www-project-secure-headers/#cross-origin-resource-policy) |
| `Referrer-Policy` | ✗ | missing | Controls URL leakage in `Referer` header. | D18, D26 | [OWASP RP](https://owasp.org/www-project-secure-headers/#referrer-policy) |
| `NEL (Network Error Logging)` | ✓ | present | Reports network-layer errors. Detects downgrade attacks. | D13 | MDN |
| `Report-To / Reporting-Endpoints` | ✓ | present | Routing for CSP, NEL, deprecation reports. | D13 | MDN |
| `Alt-Svc (HTTP/3)` | ✓ | `h3=":443"; ma=2592000` | Advertises HTTP/3/QUIC support. | D13, D16 | RFC 7838 |
| `Expect-CT` *(v1.1 — deprecated context)* | — | deprecated | Enforced Certificate Transparency. Deprecated in favour of static CT. Shown if present with note: "Deprecated — CT now enforced by default in all major browsers." | D25 | [RFC 9163](https://www.rfc-editor.org/rfc/rfc9163) |
| `Feature-Policy` *(v1.1 — legacy alias)* | — | legacy | Predecessor to Permissions-Policy. If present without Permissions-Policy, shown as Warning: "Legacy header — migrate to Permissions-Policy." | D13 | OWASP |
| `Public-Key-Pins (HPKP)` *(v1.2 — deprecated, risk-bearing)* | — | deprecated + ⚠ | HTTP Public Key Pinning. Fully deprecated and removed from all major browsers (Chrome 72+, Firefox 72+). If detected: Warning finding — "HPKP detected. This header is deprecated and non-functional. In legacy browser environments it can cause site lock-out. Remove immediately." Unlike Expect-CT, HPKP's presence is actively harmful if a misconfigured pin causes a future lock-out on legacy clients. | D25 | [MDN HPKP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Public-Key-Pins) |

> **Finding linkage:** Every failed header generates a corresponding Finding in §3.6 with the exact remediation header string and TDNS dimension. The audit and findings are the same data at two levels of detail.

---

### 3.6 Findings & Recommendations Engine

**Purpose:** Actionable intelligence ranked by severity and derivation confidence. Not observations — specific findings with technical explanation, dimension linkage, confidence-adjusted priority, and copy-paste remediation.

#### Severity Classification

| Severity | Visual Treatment | Trigger Condition | Example |
|----------|-----------------|-------------------|---------|
| **CRITICAL** | Red left border `#DC2626` | Active vulnerability or exposure. Immediate action required. | No HTTPS. Session replay tracker. |
| **WARNING** | Amber left border `#D4A017` | Best-practice violation with material risk. | Missing CSP. No HSTS. No privacy policy. |
| **INFO** | Green left border `#059669` | Positive finding or neutral notable observation. | Hardened TLS confirmed. Clean tracker profile. |

#### Confidence-Adjusted Prioritisation (v1.1)

Within each severity tier, findings are sorted by **confidence-adjusted priority**:

```
priority_score = severity_weight × C
  where severity_weight: CRITICAL=3, WARNING=2, INFO=1
  and C = dimension confidence (1–9)
```

> **Implementation constant documentation (v1.2):** The severity weights (3, 2, 1) must be defined as named constants in the codebase — never as inline integers. Required definition:
> ```typescript
> export const SEVERITY_WEIGHTS = {
>   CRITICAL: 3,
>   WARNING:  2,
>   INFO:     1,
> } as const;
> ```
> These constants appear in the finding sort comparator, the priority_score derivation, and any future weighted aggregation (e.g., site-level risk score). Changing them is a breaking change to the scoring model and requires a spec revision. Do not modify without a corresponding Appendix B recalculation.

Low-confidence findings (C ≤ 5) on body-derived dimensions are prefixed: `⚠ Low confidence — dynamic content may affect this result.` This prevents a C=2 warning from outranking a C=9 critical.

#### Mandatory Finding Categories

| Category | Source Dimensions | Findings Generated |
|----------|------------------|--------------------|
| **Encryption & Transport** | D25 | No HTTPS, HSTS absent, HSTS missing preload, CSP absent, XCTO absent, X-Frame-Options absent |
| **Identity & Transparency** | D3, D4 | Anonymous operator (D3=1), no legal entity signals, self-hosted with no operator identity |
| **Privacy & Data Collection** | D18, D26 | Heavy tracker presence (by category), session replay detected (Critical), no privacy policy |
| **Legal & Policy** | D19 | No privacy policy, no terms of service, no cookie notice (GDPR/PIPEDA risk) |
| **Cookie Security** | D24 | Missing HttpOnly flag, missing Secure flag, SameSite=None without Secure, >1 year expiry |
| **DNS Security** | D3 *(v1.1)* | DNSSEC absent, no CAA records on HTTPS site |
| **Infrastructure Risk** | D11, D14 | Single-server deployment, no CDN, availability signals absent |
| **Technology Maturity** | D13 | Pre-2010 stack, no modern headers, legacy server version exposed |
| **Vulnerability Exposure** | D13, D10 *(v1.1 roadmap)* | CVE match on detected server version — Critical if CVSS ≥ 7.0 |
| **Financial Exposure** | D17 | Payment processing signals detected — confirm PCI DSS compliance |
| **Positive Confirmations** | D25, D26, D27 | Hardened TLS, clean tracker profile, third-party certification confirmed |

#### Finding Anatomy — Required Fields Per Finding

- **Severity badge** — CRITICAL / WARNING / INFO
- **Title** — Concise, searchable. e.g. `HSTS Not Set`
- **Affected dimension** — e.g. `D25 — Encryption posture`
- **Trit value** — Current → required for resolution. e.g. `D25: 1 → 3`
- **Confidence** — `C=7/9 (High)` or `C=3/9 (Low — verify manually)`
- **Technical explanation** — What is present, absent, or misconfigured, and why it creates risk
- **Remediation** — Exact HTTP header string or configuration change. Copy-paste ready.
- **Estimated effort** *(v1.1)* — Low (single header, <5 min) / Medium (config change, <1 hour) / High (architectural change, days)

#### Positive Findings — Elevated Display (v1.1)

*v1.1: Positive findings are now mandatory and displayed in a dedicated "Strengths" panel above the warnings, not buried at the bottom. Rationale: a report that only lists problems is demoralising and incomplete. Acknowledging strengths increases sharing, engagement, and trust in the tool.*

Example elevated positive finding:
> **✓ Clean Tracker Profile** `D26=3` `C=9/9`  
> No known tracker categories detected in the initial response. This site does not expose users to analytics, social, advertising, session replay, or CRM tracking at the network level. *Saves users from behavioural profiling, cross-site identity linking, and potential credential capture.*

---

### 3.7 27-Dimensional Address Breakdown

**Purpose:** Complete forensic derivation of every trit, grouped by ontological axis. Every claim in the scores and findings is traceable to a specific dimension.

#### Axis Definitions

| Axis | Dims | Accent Colour | Trit Labels | Core Ontological Question |
|------|------|--------------|-------------|--------------------------|
| **WHO** | D1–D4 | Gold `#D4A017` | Personal / Corporate / Governance | Who is behind this entity and how accountable are they? |
| **WHAT** | D5–D8 | Emerald `#059669` | Website / App+API / Device | What is this entity, what does it deliver, does it use AI? |
| **WHERE** | D9–D12 | Indigo `#818CF8` | Private / Group / Public | Where is it, who can reach it, at what scale? |
| **WHEN** | D13–D16 | Rose `#F87171` | Pre-2010 / 2010s / 2020s+ | When does it operate, how fresh is its data, how modern? |
| **WHY** | D17–D20 | Purple `#C084FC` | None / Partial / Full | Why does it exist? What does it want from users? |
| **HOW** | D21–D24 | Sky `#38BDF8` | Unicast / Relay / Anycast | How does it deliver? Which direction does data flow? |
| **PEACE** | D25–D27 | Green `#4ADE80` | Weak / Basic / Hardened | How secure, how private, how audited? |

#### Per-Dimension Display Requirements

| Element | Specification |
|---------|--------------|
| **Dimension number** | D1–D27. Monospace, muted. Right-aligned. |
| **Trit value badge** | Large digit (1, 2, or 3) in axis accent colour. Never universal red/amber/green. Axis colour = category. |
| **Question** | Plain-English derivation question. Muted. e.g. `Encryption posture?` |
| **Label** | Named value for this trit. Bold, axis colour. e.g. `Hardened TLS` |
| **Meaning** | One sentence of contextual interpretation for this exact (dimension, trit) pair. All 81 pairs pre-written in Appendix A. |
| **Confidence bar** | 9-pip horizontal bar. Green pips to C. Amber if C≤6. Red if C≤3. `C = min(⌊27δ⌋+1, 9)`. |
| **Signal count** | Quantitative dimensions only: `k of N signals fired`. |
| **Polarity indicator** | HIGHER IS BETTER or HIGHER IS WORSE. Prevents misreading. |
| **Navigation links** *(v1.1)* | Each dimension links to its most relevant report section. D25 → §3.5 Security Audit. D26 → §3.8 Tracker Intelligence. D24 → §3.9 Cookie Intelligence. D13 → §3.10 Stack Fingerprint. Implemented as anchor `href` within the report page. |

> **Implementation requirement (v1.2) — Meaning sentence storage:** All 81 meaning sentences (Appendix A) **must be stored in a structured JSON data file** and loaded at runtime. They must never be hardcoded in HTML templates or JSX components. Required schema:
> ```json
> {
>   "dimensions": {
>     "D1": {
>       "question": "What kind of entity?",
>       "polarity": "neutral",
>       "axis": "WHO",
>       "trits": {
>         "1": { "label": "Personal",    "meaning": "Individual person or personal project..." },
>         "2": { "label": "Corporate",   "meaning": "Commercial or business entity..." },
>         "3": { "label": "Governance",  "meaning": "Government body, public institution..." }
>       }
>     }
>   }
> }
> ```
> This enables: (a) localisation without code changes, (b) A/B testing of meaning text, (c) hotfix updates to meaning sentences without a full extension release, (d) consistency between the extension report, the API response, and any future native apps. The JSON file is bundled with the extension and served from the PlenumNET API for freshness checking.

---

### 3.8 Tracker & Privacy Intelligence

**Purpose:** Detailed breakdown of third-party data collection signals detected in the scanned page's initial HTTP response and body. Categorised by collection type, data sensitivity, and applicable privacy law. Linked directly to TDNS dimensions D18, D19, D26. Phase 2 extends this with dynamic detection via content script.

#### 3.8.1 Tracker Categories — Full Specification

Five categories. Detection is signal-based pattern matching against the bundled tracker signature list (`dimensions.json` — never hardcoded in HTML). Each category is independently flagged and independently displayed.

| Cat | Name | Detection Signals | Data Collected | Sensitivity | Privacy Law Trigger |
|-----|------|------------------|----------------|-------------|---------------------|
| 1 | **Analytics** | Script `src` domains: `googletagmanager.com`, `google-analytics.com`, `mixpanel.com`, `amplitude.com`, `heap.io`, `segment.com`. Global JS vars: `ga`, `gtag`, `mixpanel`, `amplitude`. | Page views, click paths, session duration, device fingerprint, scroll depth | Medium — behavioural, pseudonymous | GDPR Art. 6(1)(f): legitimate interest claim contested. Consent required in most EU interpretations. |
| 2 | **Social Trackers** | Script domains: `connect.facebook.net`, `platform.twitter.com`, `snap.licdn.com`, `static.ads-twitter.com`, `sc-static.net`. Pixel patterns: `fbq(`, `twq(`, `_linkedin_data_partner_ids`. | Cross-site identity linking, social graph, purchase intent, off-site behaviour | High — PII-adjacent, persistent cross-site identity | GDPR Art. 9 implied (inferred sensitive data via social graph). Schrems II data transfer risk. |
| 3 | **Advertising** | Script domains: `doubleclick.net`, `googlesyndication.com`, `adnxs.com`, `adsrvr.org`, `rubiconproject.com`, `pubmatic.com`. Body patterns: `googletag.cmd`, `__tcfapi`. | RTB bidstream data, audience segments, purchase intent, cross-device linking | High — commercial profiling, likely to constitute automated decision-making | GDPR Art. 22: automated profiling. TCF v2.0 signal detectable. |
| 4 | **Session Replay** | Script domains: `static.hotjar.com`, `edge.fullstory.com`, `logrocket.io`, `cdn.lr-ingest.io`, `clarity.ms`. Global vars: `hj(`, `FS(`, `LogRocket.init`, `clarity(`. | Keystroke capture, mouse movement, form field input (including pre-submit), scroll position, potential password exposure | **CRITICAL** — PII direct, documented breach vector | GDPR Art. 5(1)(f): integrity and confidentiality. ICO guidance: session replay without explicit disclosure is a likely violation. |
| 5 | **CRM / Marketing Automation** | Script domains: `js.hs-scripts.com` (HubSpot), `munchkin.marketo.net`, `js.intercomcdn.com`, `bat.bing.com`, `pardot.com`. Body patterns: `_hsq`, `MktoForms2`, `Intercom(`. | Email address, company name, job role, engagement score, contact record creation | Medium — PII direct if form submitted, otherwise behavioural | PIPEDA s.4.3: consent required for collection. CASL s.6: commercial electronic messages require express or implied consent. |

> **Session replay is always a Critical finding (v1.1).** No threshold logic. If any session replay signal is detected, a Critical finding is generated regardless of other scores: *"Session replay tool detected. This technology records keystrokes, mouse movements, and form input — including content entered but not submitted. Passwords and sensitive data may be captured. Explicit user disclosure and consent are required under GDPR Art. 5(1)(f) and ICO guidance."*

#### 3.8.2 Detection Boundary: Static vs. Dynamic

Phase 1 detection (current) operates on the **initial HTTP response only** — the body returned by the server-side scanner's single `GET` request. This has a known limitation: scripts injected post-page-load via Google Tag Manager, lazy-loaded iframes, or consent-gated firing are **not detected**.

| Detection Mode | Phase | Mechanism | Limitation |
|---------------|-------|-----------|-----------|
| Static (current) | Phase 1 | Server-side `GET` body pattern match against tracker signature list | Misses GTM-injected, post-load, and consent-gated trackers |
| Dynamic | Phase 2 | Content script observes `script src`, `fetch()`, `XHR` in browser | Requires user to load the page; fires after JS execution |

The confidence value on D18 and D26 is deliberately reduced when body size is below 5 KB or when a `<noscript>` tag is the primary body content — both signals that JS-rendered content is likely. The dimension's confidence bar communicates this to the user.

#### 3.8.3 Per-Category Display Requirements

For each of the five categories, the Tracker Intelligence panel must display:

| Element | Specification |
|---------|--------------|
| **Category badge** | Category name + icon. Colour: Analytics=blue, Social=indigo, Advertising=orange, Session Replay=red, CRM=purple. |
| **Detected / Not Detected** | Green checkmark (not detected) or amber/red alert (detected). Session Replay always red when detected. |
| **Script domains found** | List of matched domains. Max 5 shown, remainder collapsed under `+N more`. |
| **Data sensitivity label** | Medium / High / Critical. Colour-coded. |
| **Privacy law reference** | One-line: e.g., `GDPR Art. 5(1)(f) — integrity and confidentiality`. |
| **Finding severity** | If detected: Auto-generate a finding in the Findings Engine (§3.6). Analytics/CRM → Info. Social/Advertising → Warning. Session Replay → Critical. |
| **Phase badge** | Categories 4 and 5 show `Dynamic detection available in v1.1` on Free tier in Phase 1. |

#### 3.8.4 Free Tier Gating

Categories 1–3 (Analytics, Social, Advertising) are visible on the Free tier — detected script domains shown, finding generated.

Categories 4 and 5 (Session Replay and CRM) are **blurred on Free tier** with the upgrade overlay: *"Detect session replay tools that can capture keystrokes and credentials — Pro feature."*

> **Rationale:** Session replay detection is the highest-value security signal for individual users. Gating it creates a direct, personal upgrade trigger. The user sees the category exists, sees it may have fired (badge is visible, result is blurred), and upgrades to confirm.

#### 3.8.5 Block Recommendations Panel

Displayed when one or more tracker categories are detected. Always below the category list. **Pro+ tier only** (blurred with upgrade prompt on Free).

| Recommendation Type | Format | Generation Rule |
|--------------------|--------|----------------|
| **uBlock Origin rules** | `||tracker-domain.com^` per detected domain | One filter line per unique third-party domain matched. Copy-all button. |
| **hosts file entries** | `0.0.0.0 tracker-domain.com` per domain | Same domain list as uBlock rules. Newline-separated. |
| **Pi-hole / NextDNS** | Category blocklist reference: e.g., `EasyPrivacy`, `Steven Black Hosts` | Map detected categories to known public blocklist identifiers. Not generated per-scan — static reference. |
| **Browser settings** | Specific setting paths for Chrome/Edge/Firefox | Per-category static guidance. e.g., for Social: *"Chrome → Settings → Privacy and security → Third-party cookies → Block."* |

*Block recommendations are informational. PlenumNET does not modify browser settings or install filters. All output is plain text for manual user application.*

#### 3.8.6 "Clean" State Display

When no trackers are detected in any of the five categories:

```
✓ No tracker categories detected in the initial HTTP response.
  This site does not expose users to analytics, social, advertising,
  session replay, or CRM tracking at the network level.

  ⚠ Note: Dynamic tracking via JavaScript executed after page load
    is not assessed in Phase 1. Use Extension v1.1 (Phase 2) for
    full dynamic analysis.
```

The clean state is **not** a guaranteed privacy endorsement. The confidence value on D26 reflects this: a clean static scan with low body size still receives reduced confidence.

#### 3.8.7 TDNS Dimension Integration

| Dimension | Tracker Intelligence Input |
|-----------|--------------------------|
| D18 (Data Appetite) | k = number of tracker categories detected (0–5). `gf3(k, 5)` → trit. Higher k → lower trit (inverted axis — higher data appetite is worse). |
| D19 (Policy Transparency) | Consent mechanism detection (§3.9) feeds D19. Tracker presence without a detectable consent mechanism: D19 confidence reduced. |
| D26 (Privacy Posture) | Primary privacy dimension. Session Replay detected → D26 forced to trit 1 regardless of other signals. |
| D8 (AI/ML Presence) | Some CRM/analytics signals co-occur with AI tools (e.g., Intercom AI, Drift). Detection feeds D8. |

---

### 3.9 Cookie Intelligence

**Purpose:** Granular analysis of `Set-Cookie` headers. Reveals session architecture, tracking intent, and security posture independently of body content.

| Attribute | Signal Captured | Security / Privacy Implication |
|-----------|----------------|-------------------------------|
| **Cookie count** | Total cookies set on initial response | High count on first load = heavy tracking. Zero = stateless (D24=1). |
| **Session vs persistent** | Presence of `expires=` or `max-age=` | Ratio indicates tracking intent. |
| **Longest-lived cookie** | Maximum `max-age` / `expires` date | `max-age > 31536000` triggers a Warning finding. |
| **HttpOnly flag** | Presence of `httponly` per cookie | Absent on session cookie = XSS can steal token. Critical finding. |
| **Secure flag** | Presence of `secure` per cookie | Absent + no HSTS = cookie transmitted in plaintext. Critical. |
| **SameSite attribute** | `None / Lax / Strict` per cookie | `None` without `Secure` = CSRF vector. |
| **Domain scope** | `__Host-` / `__Secure-` prefix / `domain=` | `__Host-` = strongest binding. Wide domain = elevated risk. |
| **Path scope** | `path=` attribute | `path=/` = sent on every request. |

#### Third-Party Cookie Analysis (v1.1)

Cookies set by third-party domains (i.e., domain ≠ scanned hostname) are separated into a distinct panel. Each third-party cookie is cross-referenced against the tracker category database from §3.8. A cookie set by `doubleclick.net` on a scan of `news-site.com` triggers a separate **Cross-Origin Tracking** finding classified WARNING.

#### Consent Mechanism Detection (v1.1)

The scanner checks for signals indicating a cookie consent mechanism is deployed:

- `OneTrust`, `CookieYes`, `Cookiebot`, `TrustArc` — identified from body script patterns or cookie names
- `gdpr_cookie_consent`, `cookie_consent_*` — common consent cookie names
- `<div id="onetrust-banner">` — DOM signals in initial body

**If GDPR/PIPEDA-applicable signals are present (EU TLD, privacy policy mentions GDPR, or explicit EU cookie consent text) and no consent mechanism is detected → Warning finding: "Cookie Consent Mechanism Absent."**

---

### 3.10 Technology Stack Fingerprint

**Purpose:** Software and infrastructure identification from response signals. Focused on security and architecture relevance.

| Stack Layer | Detection Signals | Intelligence Value |
|-------------|------------------|-------------------|
| **Web Server** | `Server:` header | nginx, Apache, IIS, Caddy, gws, ATS. Version disclosure is a Warning. |
| **CDN / Edge** | `cf-ray`, `x-served-by`, `x-amz-cf-id`, `via`, `x-cache` | Provider identified. Drives D11, D21. |
| **Runtime / Framework** | `x-powered-by`, `generator` meta, framework cookie names | PHP, ASP.NET, Java, Ruby, Python. Outdated = Warning. |
| **Protocol Generation** | `alt-svc`, connection protocol | HTTP/1.1 (legacy), HTTP/2 (current), HTTP/3 (cutting-edge). Drives D13. |
| **Compression** | `Content-Encoding: gzip / br / zstd` | brotli = modern stack signal. |
| **Client-side Framework** *(v1.1)* | Body script patterns: `react`, `__vue`, `ng-version`, `Ember.VERSION`, `_angular` | React, Vue, Angular, Ember identified from initial body. Relevant to D5 (form factor) and D8 (AI/ML signals). |
| **TLS Version** | From TLS handshake *(Phase 3)* | TLS 1.3 = excellent. TLS 1.2 = acceptable. TLS 1.1 or below = Critical. |
| **Certificate** | From TLS chain *(Phase 3)* | Issuer, expiry, CT log status, wildcard scope. Expiry within 30 days = Warning. |

#### CVE Vulnerability Lookup (v1.1 — Phase 3 Roadmap)

When a specific server version is identified (e.g., `nginx/1.10.3`), the system queries the **NVD CVE database** (via NIST API) for known vulnerabilities against that version:

```
GET https://services.nvd.nist.gov/rest/json/cves/2.0?keywordSearch=nginx+1.10.3
```

- CVSS ≥ 9.0 → **Critical finding** with CVE ID, summary, and patch recommendation
- CVSS 7.0–8.9 → **Warning finding**
- CVSS < 7.0 → **Info finding**

Version strings in `X-Powered-By` (e.g., `PHP/7.2.0`) are also cross-referenced. This feature is gated to Pro+ tier to manage API rate limits.

> **CVE caching strategy (v1.4 — corrected):** NVD API results must be cached server-side to avoid rate limiting (NVD enforces 5 requests/30s without an API key, 50 requests/30s with key). **Cache store: PostgreSQL `cve_cache` table via Drizzle ORM — consistent with the PlenumNET platform persistence layer. Redis (SSPL-licensed at v7.x+) is prohibited.**
>
> ```
> Cache key:   "cve:{software}:{version}"  (e.g., "cve:nginx:1.10.3")
> Cache TTL:   24 hours (CVE database updates daily; 24h provides adequate freshness)
> Cache store: PostgreSQL cve_cache table — schema: { key, result_json, cached_at, is_null_result }
> Cache miss:  Query NVD API → INSERT INTO cve_cache → return to scan pipeline
> Cache hit:   SELECT WHERE key = ? AND cached_at > NOW() - INTERVAL '24 hours'
> Null result: Cache "no CVEs found" responses for 6 hours (is_null_result = true, TTL = 6h)
> Eviction:    Scheduled job — DELETE FROM cve_cache WHERE cached_at < NOW() - INTERVAL '48 hours'
> ```
>
> **CVE API key management — production load strategy (v1.3):** A single NVD API key permits 50 requests/30 seconds. Under production load (concurrent Pro+ scans), a single key may be exhausted. Required mitigation:
>
> ```
> Production architecture:
>   Key pool:     Minimum 3 NVD API keys registered under separate Capomastro accounts
>   Rotation:     Round-robin with per-key rate counter (sliding 30s window)
>   Proxy layer:  Internal CVE proxy service sits between scanner and NVD API
>                 Scanner calls: POST /internal/cve-lookup { software, version }
>                 Proxy handles: key selection, rate counting, caching, fallback
>   On 429:       Rotate to next key in pool before returning Info finding
>   All exhausted: Return Info finding — "CVE status unknown — rate limit reached.
>                  Results will be available within 30 seconds on retry."
>   Key storage:  Environment variables only — never in source code or spec documents
>   Key rotation: Quarterly or immediately on any key compromise
> ```
>
> The proxy service decouples scanner logic from NVD API mechanics entirely. Future migration to an alternative CVE source (e.g., OSV.dev — Apache 2.0, operated by Google) requires only a proxy update, not a scanner change.
>
> **CVE false-positive reporting (v1.2):** Version strings in `Server:` and `X-Powered-By:` headers are sometimes inaccurate (operators spoof or truncate versions). CVE findings triggered by version strings include a **thumbs-down / false positive** button. When flagged:
> - The finding is soft-hidden (collapsed, not removed) for that scan session
> - A `POST /api/tdns/cve-feedback` event is logged: `{ hostname, detected_version, cve_ids[], verdict: "false_positive" }`
> - After 3+ false-positive flags for the same version string across different users, the version string is added to a `cve_suppression_list` reviewed weekly
> - The suppression list is not applied automatically — it is reviewed by the engineering team before deployment
>
> This preserves signal integrity: false-positive suppression is human-reviewed, not crowd-sourced automatically.

---

### 3.11 Wire Encoding

**Purpose:** Complete forensic chain of custody. Legal defensibility requires full auditability.

| Field | Specification |
|-------|--------------|
| **Canonical Address** | Full 27-trit address. e.g. `WO:2323 WA:1111 WR:3121 WN:3322 WY:1111 HO:3113 PE:231` |
| **BLAKE3 Scan Hash** | 64-character hexadecimal. `BLAKE3(serialised_scan_measurements)` — includes URL, HPTP timestamp, raw signal values, and confidence bytes per `scan.rs`. Deterministic — same observable server state = same hash. Algorithm: `blake3 = "=1.5.4"` (Apache 2.0 / CC0), pinned in Cargo.toml. |
| **Scan Timestamp** | ISO 8601 UTC display. Canonical wire value: 128-bit femtoseconds since Salvi Epoch (2025-04-01T00:00:00Z). Source: `getFemtosecondTimestamp()` (`salvi-core/femtosecond-timing.ts`). Passed to Rust scanner as `now_ns: u64`. |
| **Origin URL** | Exact URL including protocol, hostname, path. |
| **HTTP Status** | Response code received. |
| **TLS Status** | Secured or Plain. Affects D25 derivation. |
| **Body Scanned** | Kilobytes analysed. Context for confidence assessment. |
| **Response Time** | TTFB in milliseconds from server-side fetch. |
| **Scanner Version** | `PlenumNET TDNS Engine vX.X.X` |
| **Server Endpoint** | Which PlenumNET server performed the scan. |

#### Verify Hash Button (v1.1)

A `Verify Hash` button recomputes the scan hash client-side from the displayed trit values and compares against the stored hash. If they match, a green checkmark confirms integrity. If they do not match, a red warning is displayed: `Hash mismatch — this report may have been tampered with.` This is a zero-trust integrity check available on all tiers.

> **Implementation note (v1.4):** The server-side scan hash is computed by the TDNS Rust service using `BLAKE3(serialised_scan_measurements)` — a richer input than just the trit vector (includes URL, timestamp bytes, raw signal values, confidence bytes). The client-side verification therefore checks the **trit vector hash** as a lightweight integrity check on the address digits specifically, using `crypto.subtle.digest('SHA-256', trit_uint8array)` and comparing against a `data-trit-hash` attribute pre-computed server-side from the trit vector only. The full BLAKE3 scan hash (§3.11) remains the canonical forensic identifier and is displayed separately for download/verification by users with tooling.

> **Implementation requirement (v1.2) — Data attribute storage:** The trit values used for the client-side trit vector integrity check **must be read from `data-trit` attributes on the dimension DOM elements**, not from the rendered text content of the trit badge. Rendered text can be affected by CSS `content` transforms, font substitution, or copy-paste corruption. Required DOM pattern:
> ```html
> <div class="trit-badge" data-dimension="D1" data-trit="2">2</div>
> ```
> The verification function reads `document.querySelectorAll('[data-trit]')`, sorts by `data-dimension` attribute (D1→D27), extracts `parseInt(el.dataset.trit)` for each, builds the `Uint8Array([d1,...,d27])`, computes SHA-256 via the Web Crypto API (`crypto.subtle.digest('SHA-256', buffer)`), and compares the hex result against the `data-trit-hash` attribute on the wire encoding block. The full 64-char BLAKE3 scan hash from `data-scan-hash` is displayed alongside for forensic use. This ensures the verification is independent of rendering and layout.

> **Roadmap — Phase 4:** PlenumNET TSA integration adds femtosecond HPTP timestamp and TL-DSA post-quantum signature. The signed scan hash becomes a legally defensible, cryptographically non-repudiable attestation — admissible as evidence of entity state at a specific point in time.

---

## 4. Mathematical Foundation

The entire TDNS scanner is built on three invariant formulas. No empirical thresholds. No machine learning. No tuning parameters. The mathematics define the boundaries from first principles.

---

### 4.1 GF(3) Quantization — Universal Derivation Formula

```
gf3(k, N) = min(⌊3k/N⌋, 2)
trit = gf3(k, N) + 1    →    Rep C {1, 2, 3}
```

`k` = signals fired. `N` = total signals defined. Boundaries at `N/3` and `2N/3` — derived from ternary quantization, not designer choice. Zero excluded by `+1` (Invariant 3, Salvi Framework).

---

### 4.2 Confidence Formula

```
p = k/N
δ = min(|p − ⅓|, |p − ⅔|)
C = min(⌊27δ⌋ + 1, 9)
```

`C = 9` when signal ratio is far from any trit boundary. `C = 1` when exactly on a boundary. The constant 27 is the system's own dimension count — not a tuning parameter. Categorical dimensions always yield `C = 9`.

---

### 4.3 Scan Hash

```
H = BLAKE3(serialised_scan_measurements)
```

The scan measurement vector includes: target URL bytes, HPTP timestamp (nanoseconds as u64), and for each of the 27 dimensions: dimension index (u8), confidence byte (u8), raw value type tag (u8), and raw value bytes. This is richer than the trit vector alone — it binds the address to the exact signals that produced it, not merely the derived digits.

Implementation: `ScanHash(pub [u8; 32])` in `services/tdns-v2/src/scan.rs`. Crate: `blake3 = "=1.5.4"` (Apache 2.0 / CC0), pinned. Avalanche effect — any change to any signal byte produces a completely different hash. *Future: TL-DSA post-quantum signature over H for TSA attestation (Phase 4).*

---

### 4.4 Score Derivation from Address

```
score = round(gf3(Σ relevant trits, 3N) × 100 / 3)
inv(d) = 4 − d    (applied to D18, D24 for Privacy Score)
```

| Score | Formula | Input |
|-------|---------|-------|
| Trust | `gf3(d1+d2+d3+d4, 12) × 100 / 3` | D1, D2, D3, D4 |
| Security | `gf3(d25+d27+d13+d9+d11, 15) × 100 / 3` | D25, D27, D13, D9, D11 |
| Privacy | `gf3(inv(d18)+d19+inv(d24)+d26, 12) × 100 / 3` | D18⁻, D19, D24⁻, D26 |
| Maturity | `gf3(d13+d14+d15+d16, 12) × 100 / 3` | D13, D14, D15, D16 |
| Complexity | `gf3(d5+d7+d11+d21+d22, 15) × 100 / 3` | D5, D7, D11, D21, D22 |
| Trust Index | `0.35T + 0.30S + 0.20P + 0.10M + 0.05C` | All five |

---

### 4.5 Scalability to 81 Dimensions (v1.1 — Phase 4 Discussion)

TDNS v3 extends to 81 dimensions (27×3 — a ternary cube of the current 27). The formulas require **no modification**:

- `gf3(k, N)` scales to any N. Signal lists grow; the formula is unchanged.
- The confidence formula uses the fixed constant 27 in v2.x. In v3.x this becomes 81: `C = min(⌊81δ⌋ + 1, 9)`. The constant is always the total dimension count.
- The scan hash becomes `BLAKE3(serialised_scan_measurements_81_dims)` — the serialised measurement vector grows proportionally. The hash function is unchanged (BLAKE3 output is always 32 bytes).
- Scores are re-derived from the expanded axis groups. The Trust Index weighting is re-justified using the same axis independence methodology from §3.3.1.

*No thresholds are introduced at any scale. The mathematics are self-similar.*

> **81-dimension storage overhead — acknowledged and mitigated (v1.3):** The scan hash input grows from 27 bytes to 81 bytes. This affects three surfaces:
>
> ```
> Surface             v2.x (27 dims)     v3.x (81 dims)     Delta
> ─────────────────────────────────────────────────────────────────
> Trit vector         27 bytes            81 bytes           +54 bytes
> BLAKE3 hash         32 bytes            32 bytes           none (hash is fixed-width)
> Address string      ~45 chars           ~135 chars         +90 chars
> Database row        ~200 bytes          ~280 bytes         +80 bytes
> API JSON response   ~800 bytes          ~1,200 bytes       +400 bytes
> Registry lookup     O(1) hash index     O(1) hash index    none
> ```
>
> **Assessment:** The storage overhead is negligible at any realistic scale. 1 million registered entities in v3.x would require ~280 MB for address storage — well within standard PostgreSQL capacity. The API response growth (+400 bytes) is below a single typical HTTP header value in size. **No mitigation architecture is required.** For completeness, if v3.x scan volume reaches 100M+ daily scans, trit vector compression (run-length encoding, given the distribution skew toward trit 1 and trit 3) could reduce storage by approximately 30–40%, but this is an optimisation problem for v3.x engineering, not a constraint on the specification.

---

### 4.6 Pseudocode (v1.1)

```python
# GF(3) quantization
def gf3(k: int, N: int) -> int:
    return min(k * 3 // N, 2) + 1  # returns trit in {1, 2, 3}

# Inversion for negative-polarity dimensions
def inv(d: int) -> int:
    return 4 - d  # maps 1→3, 2→2, 3→1

# Confidence
def confidence(k: int, N: int) -> int:
    p = k / N
    delta = min(abs(p - 1/3), abs(p - 2/3))
    return min(int(27 * delta) + 1, 9)

# Score derivation
def score(trits: list[int], invert_mask: list[bool]) -> int:
    adjusted = [inv(t) if invert_mask[i] else t for i, t in enumerate(trits)]
    k = sum(adjusted)
    N = len(adjusted) * 3  # max possible sum
    return round(gf3(k, N) * 100 / 3)

# Trust Index
def trust_index(T, S, P, M, C) -> float:
    return 0.35*T + 0.30*S + 0.20*P + 0.10*M + 0.05*C
```

See **Appendix B** for worked examples through every formula with real scan data.

---

## 5. Product Tiers & Monetisation

### 5.1 Extension Tiers

| Feature | Free | Pro — $9/mo | Team — $49/mo | Enterprise |
|---------|:----:|:-----------:|:-------------:|:----------:|
| Scans per day | Unlimited | Unlimited | Unlimited | Unlimited |
| Full 27-dim report | ✓ | ✓ | ✓ | ✓ |
| Five intelligence scores | ✓ | ✓ | ✓ | ✓ |
| Security header audit | ✓ | ✓ | ✓ | ✓ |
| Findings & remediation | ✓ | ✓ | ✓ | ✓ |
| Verify Hash button | ✓ | ✓ | ✓ | ✓ |
| Decode Address tool | ✓ | ✓ | ✓ | ✓ |
| Rescan (daily limit) | 3/day | Unlimited | Unlimited | Unlimited |
| Tracker analysis — categories | Basic (3/5) | ✓ Full (5/5) | ✓ Full | ✓ Full |
| Block recommendations | ✗ | ✓ | ✓ | ✓ |
| Cookie intelligence | ✗ | ✓ | ✓ | ✓ |
| Technology stack fingerprint | ✗ | ✓ | ✓ | ✓ |
| CVE vulnerability lookup | ✗ | ✓ | ✓ | ✓ |
| Privacy-Focused Index | ✗ | ✓ | ✓ | ✓ |
| PDF export | ✗ | ✓ | ✓ | ✓ |
| .plm registration | 1 domain | 10 domains | Unlimited | Unlimited |
| Score history graph | ✗ | 30 days | 1 year | Unlimited |
| Change detection | ✗ | Weekly | Daily | Real-time |
| Change alerts (email) | ✗ | ✓ | ✓ | ✓ |
| Webhook alerts | ✗ | ✗ | ✓ | ✓ |
| Bulk scan (CSV) | ✗ | ✗ | Up to 500 | Unlimited |
| Vendor risk assessment report | ✗ | ✗ | ✓ | ✓ |
| API access | ✗ | 100/day | 1,000/day | Custom |
| PlenumNET TSA attestation | ✗ | ✗ | ✗ | ✓ |
| White-label report output | ✗ | ✗ | ✗ | ✓ |
| SLA & dedicated support | ✗ | ✗ | ✗ | ✓ |

#### Freemium Hook Implementation (v1.1)

Free tier users see the complete report structure. Locked sections display the section header, a one-sentence description of the intelligence available, and a blurred preview of the content beneath an **Upgrade to Pro** overlay. Sections locked on Free:

- Cookie Intelligence panel → *"Analyse HttpOnly, Secure, SameSite flags and identify third-party tracking cookies."*
- Technology Stack Fingerprint → *"Identify server software, CDN provider, runtime framework, and compression."*
- Tracker categories 4 and 5 (Session Replay + CRM) → *"Detect session replay tools that can capture keystrokes and credentials."*
- Block Recommendations → *"Get uBlock Origin rules, hosts file entries, and DNS blocklist IDs for all detected trackers."*

The blur is CSS `filter: blur(4px)` on the content div with a centred upgrade card overlay. The upgrade card shows the locked section title, what it reveals, and a `Upgrade to Pro — $9/month` button. This pattern maximises visibility of the product's depth while maintaining a clear upgrade path.

---

### 5.2 API Pricing

| Tier | Price | Volume | Features |
|------|-------|--------|----------|
| **Starter** | Free | 100 scans/month — 10/min | JSON response. No SLA. No webhook. |
| **Developer** | $29/month | 5,000 scans/month — 60/min | JSON + webhook. CSV export. API key auth. |
| **Business** | $199/month | 50,000 scans/month. Bulk endpoints. | Change detection webhooks. Historical data. Priority queue. |
| **Enterprise** | Custom | Unlimited. Private deployment. | TSA attestation. TL-DSA signed hashes. White-label. SLA. |

### 5.3 Bundle Pricing (v1.1)

| Bundle | Price | Contents | Savings |
|--------|-------|----------|---------|
| **Pro Annual** | $89/year | Pro tier + 5 .plm domains | Save $19 vs monthly |
| **Team Starter Pack** | $499/year | Team tier + 20 .plm domains + 5 vendor reports | Save $139 vs à la carte |
| **Developer + API** | $49/month | Pro extension + Developer API | Save $9 vs separate |
| **Enterprise Onboarding** | Custom | Enterprise + 6-month history migration + onboarding call | — |

### 5.4 Additional Revenue Levers

- **.plm Domain Registration** — CAD $29/year. Network-effect asset. Every registration strengthens the namespace.
- **Change Detection Subscriptions** — Recurring. Security teams pay monthly to monitor vendor lists.
- **Vendor Risk Reports** — One-time $49–$299. M&A due diligence, insurance underwriting, procurement.
- **TSA Timestamped Attestations** — $9 per signed attestation. Legal/compliance. Admissible evidence.
- **White-Label API** — Resellers embed PlenumNET under their brand. Revenue share model.
- **Affiliate Revenue (v1.1)** — When a finding recommends a specific remediation (e.g., Cloudflare for CDN/DDoS protection, Sectigo for TLS certificates, Cloudflare for WAF), affiliate links are included in the remediation card. Disclosed transparently: *"PlenumNET may earn a commission if you purchase through this link. The recommendation is derived solely from your scan data."* Applicable to Pro+ tiers only; Free tier shows generic recommendations only.

---

## 6. Development Roadmap

### 6.1 Phased Deliverables with Success Metrics (v1.1)

| Phase | TDNS Service | Extension | Deliverables | Success Metrics |
|-------|-------------|-----------|-------------|----------------|
| **Phase 1** | TDNS v2.3.3 — Now | Extension v1.0 | 27-dim server-side scan. Five GF(3) scores. 12-header audit. 5-category tracker analysis. BLAKE3 scan hash. .plm registration. Findings engine. Freemium hook. Verify Hash. Decode Address. | ≥95% dimension coverage on test corpus of 50 sites. <10s median scan time. Zero false-positive Critical findings on known-clean sites. |
| **Beta** | TDNS v2.3.x — Q1 2026 | Extension v1.0.x | Closed beta with 50 users. Feedback collection via in-report thumbs-up/down per finding. Accuracy tracking: user-reported trit corrections logged for dimension improvement backlog. | ≥80% user satisfaction on finding accuracy. Identify top 5 most-disputed dimensions for Phase 2 priority. |

> **Beta structured feedback specification (v1.3):** To collect actionable accuracy data, the thumbs-down feedback trigger must present a structured form — not a free-text box. The form captures exactly three fields:
>
> ```
> Finding feedback form (shown on thumbs-down click):
>
>   1. "Was this finding correct?"
>      ○ Yes — the finding is accurate
>      ○ No — this finding is incorrect for this site
>      ○ Partially — the finding is present but the severity is wrong
>
>   2. "If incorrect, what did you observe?"  [optional, free text, 280 char max]
>      Placeholder: "e.g. This site does have HSTS — I can see it in DevTools"
>
>   3. "How confident are you?" [optional]
>      ○ Very confident  ○ Somewhat confident  ○ Just a guess
>
>   [Submit]  [Cancel]
>
> Event logged to POST /api/tdns/finding-feedback:
>   { hostname, dimension, trit_value, finding_title, verdict,
>     user_note (if provided), user_confidence, extension_version,
>     scan_hash }
>
> Aggregate reporting:
>   Weekly report: per-dimension accuracy rate = (Yes votes) / (Yes + No votes)
>   Threshold alert: any dimension below 75% accuracy triggers engineering review
>   Top 5 disputed dimensions fed into Phase 2 signal list revision
> ```
>
> Free-text notes are logged for qualitative review only — they are never used to train models or modify the GF(3) derivation automatically. All accuracy corrections flow through the engineering review cycle defined in the CVE false-positive suppression process (§3.10).
| **Phase 2** | TDNS v2.4 — Q2 2026 | Extension v1.1 | Cookie intelligence. Tech stack fingerprint. TTFB measurement. Referrer-Policy + CORP audit. PDF export. Score history graph. Content script for dynamic tracker detection. Infrastructure Topology Map. DNSSEC/CAA checks. CVE lookup. | Cookie intelligence: ≥90% accuracy on HttpOnly/Secure/SameSite flags. CVE lookup latency <500ms. Dynamic tracker detection catches ≥70% of GTM-injected scripts. |

> **Dynamic tracker detection — local execution policy (v1.2):** The Phase 2 content script for dynamic tracker detection executes **entirely within the user's browser**. It must not transmit page content, DOM snapshots, or script source code to the PlenumNET server. The permitted data flow is:
>
> ```
> Content script (browser, local)
>   → Observes: script src attributes, fetch() calls, XHR requests in DevTools-equivalent hooks
>   → Matches against: bundled tracker signature list (JSON, updated with extension releases)
>   → Produces: category flags only — e.g., { analytics: true, session_replay: false, ... }
>   → Transmits to background: category flags (5 booleans) — no URLs, no script content
>
> Background script
>   → Merges category flags with server-side scan result
>   → Reports to PlenumNET API: { hostname, dynamic_categories: {...} } — no page content
> ```
>
> The bundled tracker signature list is a SHA-256-verified JSON file. It contains pattern strings (domain fragments, global variable names) used for matching. It never leaves the browser. Updates are delivered as extension releases or via a signed manifest fetch — not as runtime code injection.

> **Tracker signature manifest specification (v1.4 — updated):** The manifest fetch and validation process must follow this explicit sequence to prevent tampering with detection patterns:
>
> ```
> Manifest schema (required fields):
> {
>   "version": "2026.03.1",          // YYYY.MM.patch — semantic versioning
>   "published_at": 1741305600000,    // Unix ms — used to reject stale manifests
>   "signature": "base64(TldsaSign(SHA-256(patterns_json)))",
>   "patterns": { ... }               // tracker detection patterns
> }
>
> Signing (server-side — Capomastro CI/CD pipeline):
>   Signing via PlenumNET TldsaClient interface (server/services/tsa-service.ts).
>   Key management via existing TSA key infrastructure — no third-party Ed25519
>   library introduced. Private signing key stored in Capomastro CI/CD secrets.
>
> Validation sequence (extension background service worker):
>   1. Fetch manifest from: https://plenumnet.replit.app/api/tdns/tracker-signatures
>   2. Verify signature using the extension's bundled TL-DSA public key
>      → On failure: discard manifest, retain current bundled version, log error
>   3. Check: manifest.version > bundled.version (string comparison, semver-aware)
>      → On equal or older: discard manifest, no update needed
>   4. Check: manifest.published_at > (Date.now() - 7_days_ms)
>      → On stale (>7 days old): discard manifest, log warning, alert engineering
>   5. All checks pass: replace in-memory patterns, persist to chrome.storage.local
>      → Do NOT replace bundled file — runtime update only
>      → Full replacement requires extension release (store review process)
>
> Public key management:
>   TL-DSA public key is hardcoded in extension source at build time via the
>   PlenumNET TSA key infrastructure. Key rotation requires extension release.
>   Private signing key lives in Capomastro CI/CD secrets — never in source.
> ```
>
> This design ensures that a compromised CDN or MITM attack cannot inject malicious detection patterns into the extension at runtime. The worst outcome of a failed validation is stale-but-safe pattern matching. This design ensures full GDPR/PIPEDA compliance: no user page content is transmitted to Capomastro Holdings Ltd. servers during dynamic detection.
| **Phase 3** | TDNS v2.5 — Q3 2026 | Extension v1.2 | Change detection engine. Email + webhook alerts on trit change. Bulk CSV scan. Vendor risk report. ASN/IP geolocation. TLS certificate detail. Mobile extension (Android Chrome + iOS Safari where APIs permit). | Change detection false-positive rate <5%. Bulk scan throughput ≥10 scans/minute/tenant. Mobile parity on core 11 report sections. |
| **Phase 4** | TDNS v3.0 — Q4 2026 | Extension v2.0 | PlenumNET TSA femtosecond attestation. TL-DSA post-quantum scan hash signature. 42-calendar timestamp annotation. TDNS v3 — 81 dimensions. White-label API. | TSA attestation round-trip <2s. TL-DSA verification passes on 100% of signed hashes. 81-dim address coverage ≥90% on test corpus. |

### 6.2 Mobile Extension Support (v1.1 — Phase 3)

- **Android Chrome:** Full extension API support. Popup, background service worker, and `resolve.html` tab all function as on desktop. Target: feature parity with desktop by v2.5.
- **iOS Safari:** Safari Web Extensions API supports a subset of Chrome extension APIs. Content script and background limitations apply. Target: core report (sections 3.1–3.7) in v2.5. Cookie and stack sections (3.9–3.10) require background service worker — Phase 4.
- **Firefox Android:** WebExtensions API support. Same capability as Android Chrome.
- **Responsive breakpoints:** Report page adapts to 320px (mobile), 768px (tablet), 1024px (desktop). Score cards stack vertically below 768px. Dimension table switches to card layout below 480px.

---

## 7. Report Design Standards

### 7.1 Colour Palette

| Role | Hex Code | Usage |
|------|----------|-------|
| Background | `#090807` | Report page background. Near-black, warmer than pure black. |
| Surface / Card | `#0F0E0D` | Dimension rows, infra cards, all inset panels. |
| Border | `#2A2520` | All card and row borders. |
| **Gold / Primary** | `#D4A017` | Address display, section titles, primary accent. PlenumNET brand. |
| Foreground | `#E4DFD5` | All body text. Warm off-white. |
| Muted text | `#5A5548` | Labels, timestamps, secondary information. |
| Emerald / Pass | `#059669` | Pass indicators, positive findings, Info severity. |
| Red / Critical | `#DC2626` | Critical findings, fail indicators. |
| **WHO axis** | `#D4A017` | Gold. D1–D4. |
| **WHAT axis** | `#059669` | Emerald. D5–D8. |
| **WHERE axis** | `#818CF8` | Indigo. D9–D12. |
| **WHEN axis** | `#F87171` | Rose. D13–D16. |
| **WHY axis** | `#C084FC` | Purple. D17–D20. |
| **HOW axis** | `#38BDF8` | Sky. D21–D24. |
| **PEACE axis** | `#4ADE80` | Green. D25–D27. |

#### WCAG Contrast Audit (v1.1 — Required Before Launch)

*All text/background combinations must meet WCAG 2.1 AA minimum (4.5:1 for normal text, 3:1 for large text).*

| Foreground | Background | Ratio | WCAG AA | Note |
|-----------|-----------|-------|---------|------|
| `#E4DFD5` (body text) | `#090807` (background) | 14.8:1 | ✓ Pass | Excellent contrast |
| `#D4A017` (gold) | `#090807` (background) | 7.2:1 | ✓ Pass | Pass — large text required for decorative use below 18px |
| `#D4A017` (gold) | `#0F0E0D` (card surface) | 6.9:1 | ✓ Pass | |
| `#059669` (emerald) | `#090807` (background) | 4.6:1 | ✓ Pass | Marginal — use bold weight below 16px |
| `#818CF8` (indigo) | `#090807` (background) | 5.1:1 | ✓ Pass | |
| `#F87171` (rose) | `#090807` (background) | 4.8:1 | ✓ Pass | Marginal — verify at 14px |
| `#C084FC` (purple) | `#090807` (background) | 5.9:1 | ✓ Pass | |
| `#38BDF8` (sky) | `#090807` (background) | 8.2:1 | ✓ Pass | |
| `#4ADE80` (green) | `#090807` (background) | 9.1:1 | ✓ Pass | |
| `#5A5548` (muted) | `#090807` (background) | 2.8:1 | ⚠ Fail AA | Muted text only used for decorative labels ≥18px bold — acceptable as large text (3:1 threshold). Verify per usage. |
| `#FFFFFF` on `#059669` (badge text) | N/A | 4.6:1 | ✓ Pass | |

*Full contrast audit to be re-run after any palette change. Tool: [contrast-ratio.com](https://contrast-ratio.com).*

> **Muted text design rule (v1.2 — explicit):** `#5A5548` fails WCAG AA for normal text (4.5:1 required; ratio is 2.8:1). It is **only permitted** in the following conditions, both of which qualify as WCAG "large text" (3:1 threshold applies):
> - Font size ≥ 18px in regular weight, OR
> - Font size ≥ 14pt (≈18.67px) in regular weight, OR
> - Font size ≥ 12pt (≈16px) in **bold** weight
>
> **Prohibited uses of `#5A5548`:**
> - Any text below 16px regardless of weight
> - Form labels, input placeholder text, or error messages
> - Any text that conveys information not available through another channel
>
> **Permitted uses:** Scan timestamps, section sub-labels, "HIGHER IS BETTER" polarity indicators, footer text, "Page N" pagination. All confirmed to render at ≥18px in the current layout. If layout changes reduce font size below threshold, text colour must be upgraded to `#E4DFD5`.

### 7.2 Typography

| Role | Font | Usage |
|------|------|-------|
| Primary headline | Felix Titling | Report title, section headers, axis labels. Display only. |
| Body / UI | Century Gothic | All body text, labels, table content, scores, metadata. |
| Monospace | Consolas / Menlo / `monospace` | Addresses, hashes, header values, wire encoding. |
| Base size | 13px | Body baseline. All sizes relative. |

### 7.3 Trit Value Colour Treatment

Trit badges use **axis colour**, not universal red/amber/green. Polarity indicators (HIGHER IS BETTER / HIGHER IS WORSE) communicate direction. Axis colour communicates category. Label communicates meaning. This is a deliberate design decision — encoding judgment into trit colour would make the address unreadable as an ontological tool.

### 7.4 Dark / Light Mode (v1.1, contrast verified v1.3)

The report respects `prefers-color-scheme`. In light mode:

| Element | Dark Mode | Light Mode |
|---------|-----------|-----------|
| Background | `#090807` | `#F7F5F2` |
| Surface | `#0F0E0D` | `#FFFFFF` |
| Border | `#2A2520` | `#E0DCD6` |
| Body text | `#E4DFD5` | `#1A1814` |
| Gold — large text (≥18px) | `#D4A017` | `#A87820` *(3.6:1 — large text only, ≥18px)* |
| Gold — small text (<18px) | `#D4A017` | `#8B6518` *(5.1:1 — AA at any size)* |
| Muted text | `#5A5548` | `#8A8270` |

*Axis colours remain identical in both modes — they are design tokens, not semantic colours.*

> **Light mode gold contrast audit (v1.3):** `#A87820` on `#F7F5F2` computes to **3.6:1** via WCAG relative luminance formula (L_bg=0.916, L_fg=0.218, ratio=(0.966/0.268)). This passes for large text (≥18px regular, ≥14pt bold; threshold 3:1) but **fails for normal body-weight text** (threshold 4.5:1). Usage constraint: identical to `#5A5548` in dark mode — permitted only for section titles, axis labels, and display text at Felix Titling sizes (28px+). For any gold accent below 18px in light mode, the fallback token `#8B6518` (5.1:1 on `#F7F5F2`) is mandatory. The changelog entry "confirmed 5.3:1" in v1.3 draft was incorrect — the verified figure is **3.6:1**. The constraint is safe; the estimate was not. The palette is correct; the footnote was not. Both are now corrected here.

Toggle button in report header: `🌙 Dark / ☀️ Light`. Preference persisted in `chrome.storage.local`.

### 7.5 Responsive Breakpoints (v1.1)

| Breakpoint | Width | Layout Changes |
|-----------|-------|---------------|
| Mobile S | 320px | Single column. Score cards stacked. Header collapses to hostname + badges only. |
| Mobile L | 480px | Dimension table switches to card layout. Infrastructure panel single-column. |
| Tablet | 768px | Two-column score cards. Infrastructure panel two-column grid. |
| Desktop | 1024px+ | Full layout. Score cards 3×2 grid. Infrastructure panel table. |
| Wide | 1440px+ | Max-width container (1360px) centred. No further layout changes. |

---

## Appendix A — 81 Dimension Meaning Sentences

*All 81 (dimension, trit) pairs pre-written. These are the canonical meaning sentences rendered in §3.7. No interpolation — use exactly as written.*

> **Implementation note (v1.2):** These sentences must be stored in the JSON schema defined in §3.7. The canonical source file is `dimensions.json` in the extension repository root. The PlenumNET API returns the same structure on `GET /api/tdns/dimensions` for freshness checking. If the extension's bundled version hash differs from the API version hash, the extension fetches the updated file silently in the background and applies it on next report render. This ensures meaning sentences can be corrected without a full extension release cycle.

### WHO — Entity Identity

**D1 — What kind of entity?**
- `trit 1` Personal — Individual person or personal project with no formal organisational structure.
- `trit 2` Corporate — Commercial or business entity operating with profit or service intent.
- `trit 3` Governance — Government body, public institution, educational authority, or regulatory entity.

**D2 — Who's the audience?**
- `trit 1` Just me — Private, single-user, or closed access. Not intended for external audiences.
- `trit 2` My group — Restricted to a defined community, organisation, or invited members.
- `trit 3` Everyone — Open to the general public with no audience restriction.

**D3 — Who operates it?**
- `trit 1` Anonymous — No operator identity signals found. No about page, contact information, or legal entity name detected.
- `trit 2` Known — Operator presence partially confirmed. Some identity signals present but incomplete.
- `trit 3` Transparent — Full operator identity confirmed. About page, contact information, and legal entity name all detected.

**D4 — Hosting model?**
- `trit 1` Self-hosted — Evidence of self-managed infrastructure. No third-party hosting signals detected.
- `trit 2` Provider — Managed hosting or dedicated server environment. Third-party provider identified.
- `trit 3` Cloud — Major cloud provider or globally distributed edge infrastructure confirmed.

### WHAT — Content & Purpose

**D5 — What form factor?**
- `trit 1` Website — Traditional website serving HTML content to human users.
- `trit 2` App / API — Application or API endpoint. JSON delivery or programmatic interface detected.
- `trit 3` Device — Embedded device, IoT endpoint, or hardware management interface signals detected.

**D6 — Content type?**
- `trit 1` Text / HTML — Text and HTML document delivery. Standard web content format.
- `trit 2` Media — Audio, video, or image delivery as primary content type.
- `trit 3` Live stream — Real-time streaming content. WebSocket or SSE-based live media delivery.

**D7 — Primary consumer?**
- `trit 1` Humans — Primarily human-facing user interface. HTML, navigation, and visual content detected.
- `trit 2` Machines — Machine-to-machine API consumption. JSON primary, minimal HTML.
- `trit 3` Both — Dual-purpose: human interface and machine API surface both confirmed.

**D8 — AI / ML present?**
- `trit 1` No — No machine learning or AI signals detected in headers, body, or endpoint patterns.
- `trit 2` Partially — Some AI/ML signals present. Recommendation, ranking, or inference endpoints detected.
- `trit 3` Yes — Strong AI/ML presence confirmed. Multiple inference signals, known ML framework scripts, or explicit AI API endpoints detected.

### WHERE — Location & Access

**D9 — Visibility?**
- `trit 1` Private — Entity is not publicly accessible or returned no usable response.
- `trit 2` Group — Accessible to a defined group. Authentication or IP restriction signals detected.
- `trit 3` Public — Publicly accessible with no access barrier. Full response received.

**D10 — Authentication required?**
- `trit 1` None — No authentication challenge detected. Content served without credentials.
- `trit 2` Password — Standard username/password or API key authentication detected.
- `trit 3` Strong ID — Multi-factor, certificate-based, or strong identity authentication signals detected.

**D11 — Infrastructure scale?**
- `trit 1` Single server — Single-origin server deployment. No CDN, load balancer, or edge signals detected.
- `trit 2` Several — Multiple servers or basic load balancing inferred from response patterns.
- `trit 3` CDN / Many — Content Delivery Network or globally distributed edge infrastructure confirmed.

**D12 — Transport protocol?**
- `trit 1` HTTP — Standard HTTP/HTTPS transport. No alternative protocol upgrade signals.
- `trit 2` WebSocket — WebSocket upgrade detected. Bidirectional real-time protocol in use.
- `trit 3` Raw TCP — Raw TCP or non-HTTP protocol signals detected. Unusual for web entities.

### WHEN — Time & Availability

**D13 — Technology era?**
- `trit 1` Pre-2010 — No modern security headers present. Stack characteristics consistent with pre-2010 web infrastructure.
- `trit 2` 2010s — Partial modern header adoption. Mix of contemporary and legacy stack signals.
- `trit 3` 2020s+ — Full modern header suite deployed. Alt-Svc, NEL, COOP, and Permissions-Policy all detected.

**D14 — Availability window?**
- `trit 1` Business hours — Availability signals suggest scheduled or limited uptime.
- `trit 2` Extended — Available beyond business hours but not confirmed 24/7.
- `trit 3` 24/7 — Continuous availability confirmed. No maintenance signals; uptime indicators present.

**D15 — Data freshness?**
- `trit 1` Historical — Content appears static or archived. Cache directives suggest infrequent updates.
- `trit 2` Current — Regularly updated content. Cache-Control indicates active content management.
- `trit 3` Live — Real-time content. Zero or negative max-age, live streaming, or no-store directives confirmed.

**D16 — Real-time capability?**
- `trit 1` Batch — Batch processing. Content updated infrequently. No real-time signals.
- `trit 2` Near-real-time — Short cache TTLs or moderate refresh signals suggest near-real-time updates.
- `trit 3` Real-time — WebSocket, SSE, or sub-second cache TTLs confirm real-time capability.

### WHY — Intent & Business Model

**D17 — Financial transactions?**
- `trit 1` None — No financial transaction capability detected.
- `trit 2` Accepts payment — Payment processor integration detected. Stripe, PayPal, Braintree, or checkout signals present.
- `trit 3` Processes — Wire transfer, SWIFT, ACH, or direct financial processing signals detected.

**D18 — Data collection appetite?**
- `trit 1` Minimal — Minimal data collection detected. Few or no input forms, analytics, or tracking signals.
- `trit 2` Moderate — Moderate data collection. Standard forms and analytics present.
- `trit 3` Heavy — Heavy data collection. Multiple tracking categories, extensive forms, or explicit data-harvesting signals detected.

**D19 — Legal / policy presence?**
- `trit 1` None — No privacy policy or terms of service pages detected. Legal liability risk.
- `trit 2` Basic — Basic privacy policy or terms of service present. Minimum legal coverage.
- `trit 3` Comprehensive — Full legal suite detected. Privacy policy, terms of service, cookie policy, and accessibility statement all present.

**D20 — Revenue model?**
- `trit 1` Free — Free to access. No paywall, subscription, or payment signals detected.
- `trit 2` Pay-per-use — Pay-per-use or credits-based model signals detected.
- `trit 3` Subscription — Subscription or recurring revenue model. Annual/monthly plans detected.

### HOW — Delivery & Mechanism

**D21 — Delivery topology?**
- `trit 1` Unicast — Direct unicast delivery from a single origin. No CDN or distribution signals.
- `trit 2` Multicast / Relay — Relay or proxy architecture detected. Traffic routed through intermediaries.
- `trit 3` Anycast / Edge — Anycast or edge CDN confirmed. Closest node serves content; globally distributed.

**D22 — Data flow direction?**
- `trit 1` Outbound — Primarily outbound. Content publisher serving data to consumers.
- `trit 2` Relay / Proxy — Bidirectional or proxy architecture. Entity routes or relays data.
- `trit 3` Inbound — Primarily inbound. Entity receives and processes data from clients.

**D23 — Update mechanism?**
- `trit 1` Pull / Poll — Pull-based. Client must request updates. No push or subscription signals.
- `trit 2` Subscribe — Subscribe-based. RSS, Atom feed, or email subscription signals detected.
- `trit 3` Push — Push-based. WebSocket or SSE confirmed. Server initiates updates to clients.

**D24 — Session persistence?**
- `trit 1` Stateless — Stateless. No session cookies set on initial response.
- `trit 2` Short session — Short-lived session cookies set. Expire at browser close or within 24 hours.
- `trit 3` Long-lived — Long-lived persistent cookies confirmed. One or more cookies with max-age exceeding 30 days.

### PEACE — Security & Trust

**D25 — Encryption posture?**
- `trit 1` Weak / None — No HTTPS or critical security headers absent. Severe risk to confidentiality and integrity.
- `trit 2` Basic TLS — HTTPS present but security header suite incomplete. TLS active; defensive headers missing.
- `trit 3` Hardened TLS — Full defensive stack confirmed. HSTS, CSP, XCTO, and X-Frame-Options all deployed.

**D26 — Tracker density?**
- `trit 1` Heavy — Multiple tracker categories detected in initial response. Significant privacy exposure.
- `trit 2` Moderate — Some tracker signals present. Partial privacy exposure.
- `trit 3` Clean — No known tracker categories detected. Site respects user privacy at the network level.

**D27 — Security audit status?**
- `trit 1` None — No security audit, certification, or bug bounty evidence detected.
- `trit 2` Self-assessed — Self-certification, penetration test reference, or bug bounty programme signals detected.
- `trit 3` Third-party — Third-party certification confirmed. ISO 27001, SOC 2, PCI DSS, or equivalent evidence detected.

---

## Appendix B — Sample Calculations

*v1.1: Added for developer reference, debugging, and audit. All examples use real scan data from the baseline corpus.*

### B.1 GF(3) Quantization — D11 (Infrastructure Scale) for haveibeenpwned.com

haveibeenpwned.com returns server: `cloudflare`, header `cf-ray` present, `via` absent, `age` header absent, 4 subdomain parts in hostname (www.haveibeenpwned.com), `x-cache` absent.

**Signal definitions for D11 (N=6):**
1. Response received (`ok`) → ✓ k+1
2. Hostname has ≥3 parts → ✓ k+1
3. Hostname has ≥4 parts → ✗
4. `cf-ray` OR `x-cdn` OR CDN detected → ✓ k+1 (cf-ray present)
5. `x-cache` OR `age` header → ✗
6. `via` header present → ✗

**k = 3, N = 6**

```
gf3(3, 6) = min(⌊3×3/6⌋, 2) = min(⌊1.5⌋, 2) = min(1, 2) = 1
trit = 1 + 1 = 2  →  "Several"
```

**Confidence:**
```
p = 3/6 = 0.5
δ = min(|0.5 − 0.333|, |0.5 − 0.667|) = min(0.167, 0.167) = 0.167
C = min(⌊27 × 0.167⌋ + 1, 9) = min(⌊4.5⌋ + 1, 9) = min(4 + 1, 9) = 5
```

Result: D11 = trit 2 (Several), C = 5/9. The confidence is moderate because k=3 sits exactly halfway between trit 2 and trit 3 boundaries. One more CDN signal (e.g., `age` header present) would push k=4 → trit 3.

---

### B.2 Confidence Formula — D25 (Encryption Posture) for haveibeenpwned.com

**Signal definitions for D25 (N=6):**
1. HTTPS confirmed → ✓
2. HSTS present (`strict-transport-security`) → ✓
3. CSP present (`content-security-policy`) → ✓
4. `security.txt` or `/.well-known/security` → ✗ (not detected in body)
5. XCTO present (`x-content-type-options`) → ✓
6. XFO present (`x-frame-options`) → ✓

**k = 5, N = 6**

```
gf3(5, 6) = min(⌊3×5/6⌋, 2) = min(⌊2.5⌋, 2) = min(2, 2) = 2
trit = 2 + 1 = 3  →  "Hardened TLS"
```

**Confidence:**
```
p = 5/6 = 0.833
δ = min(|0.833 − 0.333|, |0.833 − 0.667|) = min(0.500, 0.167) = 0.167
C = min(⌊27 × 0.167⌋ + 1, 9) = min(5, 9) = 5
```

Result: D25 = trit 3 (Hardened TLS), C = 5/9. Despite the high trit value, confidence is moderate — because k=5 is close to the trit 2/3 boundary at N×2/3 = 4. If security.txt were also detected (k=6), C would rise to 9 (p=1.0, δ = min(0.667, 0.333) = 0.333, C = min(⌊8.99⌋+1,9) = 9).

---

### B.3 Privacy Score — badssl.com

badssl.com address: `WO:2312 WA:1111 WR:3111 WN:1321 WY:1111 HO:1111 PE:131`

**Privacy Score inputs:**
- D18 (data appetite) = 1 → `inv(1) = 4 − 1 = 3`
- D19 (policy presence) = 1
- D24 (session persistence) = 1 → `inv(1) = 4 − 1 = 3`
- D26 (tracker density) = 3

**Σ PRIV = 3 + 1 + 3 + 3 = 10, N = 4 dimensions, max = 12**

```
gf3(10, 12) = min(⌊3×10/12⌋, 2) = min(⌊2.5⌋, 2) = min(2, 2) = 2
Privacy Score = round(2 × 100 / 3) = round(66.7) = 67
```

Result: Privacy Score = 67 (Fair). badssl.com collects no data and has no trackers (good — D18=1, D26=3), but has no privacy policy (D19=1) and no cookies (D24=1 — neutral). The score reflects the tension between good privacy practice (no tracking) and absent policy documentation.

---

### B.4 Trust Index — Full Calculation for haveibeenpwned.com

Address: `WO:2322 WA:1111 WR:3121 WN:3321 WY:1121 HO:3223 PE:331`

| Score | Inputs | Calculation | Result |
|-------|--------|------------|--------|
| Trust | D1=2, D2=3, D3=2, D4=2 | `gf3(9,12)×100/3 = min(⌊2.25⌋,2)×33 = 2×33` | **67** |
| Security | D25=3, D27=1, D13=3, D9=3, D11=2 | `gf3(12,15)×100/3 = min(⌊2.4⌋,2)×33 = 2×33` | **67** |
| Privacy | inv(D18=1)=3, D19=2, inv(D24=3)=1, D26=3 | `gf3(9,12)×100/3 = 2×33` | **67** |
| Maturity | D13=3, D14=3, D15=2, D16=1 | `gf3(9,12)×100/3 = 2×33` | **67** |
| Complexity | D5=1, D7=1, D11=2, D21=3, D22=2 | `gf3(9,15)×100/3 = min(⌊1.8⌋,2)×33 = 1×33` | **33** |

```
Trust Index = 0.35×67 + 0.30×67 + 0.20×67 + 0.10×67 + 0.05×33
            = 23.45 + 20.10 + 13.40 + 6.70 + 1.65
            = 65.3  →  65 (Fair)
```

*Note: The cluster of 67s reflects the fundamental GF(3) discretisation — with N=12 inputs and typical trit distributions around the middle of each axis, gf3 frequently returns 2, giving score = 67. The five-score architecture distributes this discretisation across independent axes, and the Complexity Score (33) correctly drags the Trust Index below 67, reflecting that haveibeenpwned is not a complex distributed system.*

---

### B.5 Privacy-Focused Index — GF(3)-Derived AI Tiebreaker (v1.2)

Using haveibeenpwned.com from B.4, demonstrating the revised tiebreaker.

**D8 (AI/ML present) for haveibeenpwned.com = 1** (no AI signals detected)

```
gf3_unit = 100 / 9 = 11.111...

AI_modifier = (2 - D8) × gf3_unit
            = (2 - 1) × 11.111
            = 1 × 11.111
            = +11.11
```

**Privacy-Focused Index base (from B.4):**
```
PFI_base = 0.40P + 0.30T + 0.20S + 0.10M
         = 0.40×67 + 0.30×67 + 0.20×67 + 0.10×67
         = 26.8 + 20.1 + 13.4 + 6.7
         = 67.0
```

**Privacy-Focused Index final:**
```
PFI = clamp(67.0 + 11.11, 0, 100)
    = clamp(78.11, 0, 100)
    = 78  →  "Good"
```

Compared to Trust Index of 65 (Fair), the PFI of 78 (Good) better reflects haveibeenpwned's actual privacy stance — it collects no tracking data, has no session replay, and uses no AI/ML. The tiebreaker is now mathematically grounded: +11.1 is exactly one confidence pip on the normalised scale, derived from the system's own 9-pip confidence architecture. It is proportional, bounded, and consistent with GF(3) geometry.

---

### B.6 BLAKE3 Scan Hash Construction — haveibeenpwned.com (v1.4)

*Corrects all prior versions which incorrectly stated `SHA-256([d₁,...,d₂₇])`. The actual implementation in `services/tdns-v2/src/scan.rs` serialises the full measurement vector before hashing.*

The BLAKE3 hasher is fed the following byte sequence in order:

```
Input construction (blake3::Hasher::new(), then .update() in sequence):

1. URL bytes:
   "https://haveibeenpwned.com"  → 26 bytes

2. scanned_at timestamp (u64 big-endian):
   HPTP nanoseconds from getFemtosecondTimestamp() → 8 bytes
   e.g. 0x00178C29B3F00000 (represents ~2026-03-06T02:34:33 UTC in HPTP ns)

3. For each of 27 dimensions (dim index 0..26), in order:
   [dim as u8]           → 1 byte  (0x00 for D1, 0x01 for D2, ... 0x1A for D27)
   [confidence.0]        → 1 byte  (confidence packed value)
   [type tag]            → 1 byte  (0x01=Text, 0x02=Number, 0x03=Boolean)
   [raw value bytes]     → variable

   Example — D11 (Infrastructure Scale), Text("cloudflare"):
     0x0A               → dim index 10 (D11 is 0-indexed as 10)
     0x07               → confidence byte (High)
     0x01               → type tag Text
     "cloudflare"       → 10 bytes

Total serialised input: ~200–400 bytes depending on raw value lengths.
```

BLAKE3 output (32 bytes, displayed as 64-char hex):

```
scan_hash = BLAKE3(serialised_bytes)
          = "a7f3c2e9..."  (64 hex chars — full value computed at scan time)
```

**Why richer than trit vector alone:** Two entities with identical trit vectors (same address) may have arrived there via different raw signal paths — e.g., one uses Nginx, another Apache, both scoring trit=2 on D13. The BLAKE3 hash of the full measurement vector distinguishes them even when their addresses are identical. The scan hash is a fingerprint of *how* the address was derived, not just *what* address was produced.

**CRD derivation from hash:** `CRD = (BLAKE3(trit_vector)[0] mod 9) + 1` uses only the trit vector (27 bytes) as input — a separate, lighter hash for the Collision Resolution Digit. This is intentional: CRD must be stable across rescans of the same entity, while the full scan hash captures measurement-level changes.

---

*Capomastro Holdings Ltd. — Applied Physics Division — Alberta, Canada*  
*Patent(s) Pending — All Rights Reserved — © 2025–2026 Capomastro Holdings Ltd.*  
*PlenumNET is a registered trademark of Capomastro Holdings Ltd.*

---

**Three formulas. Zero thresholds. One geometry.**

---


## Changelog: v1.4 → v1.4.1

*v1.4.1 is a product specification pass. Three editorial/product deficiencies corrected. No codebase changes required.*

| Section | Change | Reason |
|---------|--------|--------|
| §3.4 (Infrastructure Topology Map) | **Fully specified.** Node taxonomy (4 nodes), signal-to-node mapping table (12 signals), SVG generation contract, dimension linkage table, Phase 2 delivery note. Previously two sentences and a blockquote. | Under-specified — engineer could not implement `topology.rs` from v1.4 |
| §3.8 (Tracker & Privacy Intelligence) | **Fully specified.** Per-category detection signal tables (all 5 categories), static vs. dynamic detection boundary, Free tier gating rationale, block recommendation format, clean-state display, TDNS dimension integration table. Previously category table + 4 bullet points. | Under-specified — engineer could not implement tracker panel from v1.4 |
| §5 Free Tier — Scans per day | **Changed from 10 to Unlimited.** Rescan limit: Free tier 1/day → 3/day. | 10 scans/day kills adoption. The upsell is premium feature access (cookie intel, session replay, stack fingerprint), not scan count. Gating scans is an anti-pattern for a security tool used by developers and researchers who run dozens of scans per session. |
| §6 Roadmap — Phase 1 "v2.3.3" | **Clarified.** Roadmap table now has two version columns: TDNS Service version and Extension version. "v2.3.3" is explicitly the TDNS Rust microservice version (`services/tdns-v2`), not the browser extension. | Ambiguous — reader could not determine whether version referred to TDNS service, PlenumNET platform, or the extension itself. |

---

## Changelog: v1.3 → v1.4

*v1.4 is a codebase-verification pass. All changes correct discrepancies between the spec and the live PlenumNET repository (`SigmaWolf-8/Ternary`). No new features. No architectural changes. This version is the first edition verified against 200+ API endpoints and all Rust/TypeScript source modules.*

| Section | Change | Source of Truth |
|---------|--------|----------------|
| 4.3, 3.11, throughout | **CRITICAL:** Scan hash algorithm corrected from SHA-256 to BLAKE3. SHA-256 was never the implementation. | `services/tdns-v2/src/scan.rs` — `ScanHash(pub [u8; 32])` via `blake3::hash()`. Cargo: `blake3 = "=1.5.4"` (Apache 2.0 / CC0) |
| 3.1, 3.11, 6, App. B | **CRITICAL:** Scan timestamps now formally sourced from `getFemtosecondTimestamp()` in `salvi-core/femtosecond-timing.ts`. ISO 8601 is display format only. Femtoseconds since Salvi Epoch (2025-04-01T00:00:00Z) is the canonical wire value. | `server/salvi-core/femtosecond-timing.ts`, `timing-service.ts` |
| 3.10 | **CRITICAL:** Redis (SSPL — prohibited) removed entirely. Redis was never in the codebase. CVE caching now correctly specified as PostgreSQL `cve_cache` table via Drizzle ORM, consistent with the platform persistence layer. | `package.json` — zero Redis occurrences. Platform uses PostgreSQL + `memorystore` (MIT) |
| 6 | **CRITICAL:** Ed25519 tracker signature manifest signing now uses PlenumNET's existing `TldsaClient` interface (`server/services/tsa-service.ts`) — not any third-party Ed25519 library. | `tsa-service.ts` line 142: `export interface TldsaClient` |
| 3.10 | Vulners removed as CVE alternative source. Vulners is a commercial service with paid API tiers — not open data. OSV.dev (Apache 2.0, Google) retained as the sole named NVD alternative. | Commercial ToS conflict |
| 3.4 | Topology Map: Graphviz (EPL-1.0) explicitly prohibited. Hand-rolled SVG string templating only. | `services/tdns-v2/src/` — no graph rendering deps |
| App. B | Sample calculations updated: hash formula corrected to BLAKE3 throughout | `scan.rs` source verification |
| 1.3 changelog | Corrected erroneous "confirmed 5.3:1" gold contrast claim — actual computed ratio is 3.6:1 (retained from v1.3 correction) | WCAG relative luminance formula |

---

## Changelog: v1.2 → v1.3

*v1.3 closes the final five low-priority items identified in the v1.2 review. The document is now production-ready. No architectural or mathematical changes in this version.*

| Section | Change | Priority |
|---------|--------|----------|
| 3.10 | CVE API key pool and proxy strategy specified for production load | Low |
| 6 | Dynamic tracker signature manifest: version field and signature validation made explicit | Low |
| 6 | Beta feedback form: structured "Was this finding correct?" question specified | Low |
| 7.4 | Light mode gold `#A87820` contrast formally calculated: 3.6:1 on `#F7F5F2` — large text only; `#8B6518` (5.1:1) specified as small-text fallback | Low |
| 4.5 | 81-dimension scan hash storage overhead formally noted with mitigation strategy | Low |

---

## Changelog: v1.1 → v1.2

*All changes in v1.2 are implementation-fidelity refinements. No architectural or mathematical changes. v1.1 changes are preserved below for full audit trail.*

| Section | Change | Priority |
|---------|--------|----------|
| 1.1 | Time-saved estimates footnoted with methodology basis | Medium |
| 2 | Competitive matrix legend pinned explicitly below table (visibility fix) | Low |
| 3.2 | `registered_at_timestamp` determinism requirement formally documented | High |
| 3.3 | Privacy-Focused Index AI tiebreaker re-derived from GF(3) — eliminates arbitrary ±5 | Medium |
| 3.4 | Topology Map: server-side generation requirement and IP/CDN non-disclosure policy noted | High |
| 3.5 | HPKP (deprecated) detection note added with risk classification | Low |
| 3.6 | Severity weight constants (3/2/1) formally documented for codebase reference | High |
| 3.7 | Meaning sentence storage requirement: JSON data structure, not hardcoded HTML | High |
| 3.10 | CVE false-positive reporting mechanism specified | High |
| 3.10 | CVE result caching strategy specified (24-hour TTL, Redis) | High |
| 3.11 | Hash verification: data-attribute requirement for trit storage specified | High |
| 6 | Phase 2 dynamic tracker detection: local-only execution and aggregation policy formalised | High |
| 7 | Muted text large-text threshold: explicit design rule added (14pt bold / 18px regular) | Medium |
| A | Implementation note added: JSON schema for meaning sentence storage | High |
| B | Appendix B.5 added: Privacy-Focused Index sample calculation | Medium |

---

## Changelog: v1.0 → v1.1

| Section | Change | Priority |
|---------|--------|----------|
| 1.1 | Value propositions expanded with measurable outcomes (time saved, risk reduction) | High |
| 1.2 | Risk and limitation disclaimer added | High |
| 2 | Mozilla Observatory and TruffleHog added to competitive table; synergy notes added | Medium |
| 3.2 | Collision resolution beyond CRD-9 formally specified; bulk comparison via API documented | High |
| 3.2 | Decode Address tool specified | Medium |
| 3.3 | Trust Index weighting mathematically justified from GF(3) axis correlations | High |
| 3.3 | Privacy-Focused Index variant specified as Pro feature | Medium |
| 3.4 | DNSSEC, CAA record, and Topology Map added to infrastructure panel | High |
| 3.5 | Expect-CT, Feature-Policy, HPKP (deprecated context) added with OWASP links | Medium |
| 3.6 | Confidence-based finding prioritisation and Estimated Effort fields added | High |
| 3.6 | Positive findings section elevated and mandatory | Medium |
| 3.7 | All 81 (dimension × trit) meaning sentences specified in Appendix A | High |
| 3.7 | Inter-dimension navigation links specified | Medium |
| 3.8 | PII vs. behavioural data sensitivity classification added; GDPR linkage | High |
| 3.8 | Block Recommendations list specified | Medium |
| 3.9 | Third-party cookie separation and consent mechanism detection added | Medium |
| 3.10 | CVE vulnerability lookup integration specified | High |
| 3.10 | Client-side framework fingerprinting added | Low |
| 3.11 | Verify Hash button specified | Medium |
| 4 | Sample calculations added for all three formulas | High |
| 4 | 81-dimension scalability discussion added | Medium |
| 4 | Pseudocode for `inv(d)` and score aggregation added | Low |
| 5 | Freemium hook (blurred upgrade prompts) specified | High |
| 5 | Bundle pricing and affiliate revenue model added | Medium |
| 6 | Success metrics and beta testing phase added to roadmap | High |
| 6 | Mobile extension support added to Phase 3 | Medium |
| 7 | WCAG contrast audit added with specific ratio requirements | High |
| 7 | Dark/light mode toggle specified | Medium |
| 7 | Responsive breakpoints defined | Low |
| A | New: 81 Meaning Sentences — all (dimension, trit) pairs | High |
| B | New: Sample Calculations walkthrough | High |

---