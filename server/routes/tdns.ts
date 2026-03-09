// PlenumNET TDNS — Server-Side Scanner & Registry v2.5.0
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada) — Applied Physics Division
// Patent(s) Pending — All Rights Reserved
//
// TDNS Browser Extension API — full report payload
// All 27 classification trits derived server-side from live HTTP fetch.
// Derivation rules ported exactly from services/tdns-v2/src/derive.rs.
//
// TDNS v2.5.0: 54-trit address = 27 classification + 27 identity anchor.
// Identity derivation: IdentitySponge — ternary sponge, parameters derived from
//   TDNS architecture (state=54, rate=27, capacity=27, rounds=27). No binary
//   hash primitives in identity derivation. No domain crossing. Mirrors
//   services/tdns-v2/src/identity.rs exactly.
//
// Scan hash: TIS-27 (shared/tis-sponge.ts) — fast non-cryptographic integrity function.
//   4 rounds, 7-neighbor extended theta at ±1/±7/±13. NOT the identity sponge.
// Identity derivation: 27-round inline sponge (unchanged, mirrors identity.rs).
// Timestamps: ISO 8601 display (JS path).
// Production path: getFemtosecondTimestamp() via server/salvi-core/femtosecond-timing.ts.

import type { Express, Request, Response } from "express";
import { createLogger } from "../logger";

const log = createLogger("tdns");

// ── TDNS Identity Sponge ──────────────────────────────────────────────────────
//
// Ternary sponge for URL identity hashing.
// Parameters derived from TDNS architecture — not chosen arbitrarily:
//   State:    54 trits  ← full TDNS address width (27 classification + 27 identity)
//   Rate:     27 trits  ← identity anchor width = classification width
//   Capacity: 27 trits  ← classification layer width
//   Rounds:   27        ← one per output trit
//   Stride:   13        ← gcd(13,54)=1, complete permutation cycle
//
// Mirrors services/tdns-v2/src/identity.rs exactly.
// No SHA-256. No BLAKE3. All arithmetic in GF(3) = {0,1,2}.
// Output is in Rep C {1,2,3} — zero structurally impossible in legitimate output.

const TIS_STATE  = 54;
const TIS_RATE   = 27;
const TIS_ROUNDS = 9;  // 3² — 3× safety margin over 3-round full diffusion (7-neighbor theta)
const TIS_RC: readonly number[] = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];

function gf3Add(a: number, b: number): number { return (a + b) % 3; }

function tisByteToTrits(b: number): number[] {
  const out: number[] = [];
  let v = b;
  for (let i = 0; i < 6; i++) { out.push(v % 3); v = Math.floor(v / 3); }
  return out;
}

function tisTheta(s: Uint8Array): void {
  const t = new Uint8Array(TIS_STATE);
  const W = TIS_STATE;
  for (let i = 0; i < W; i++) {
    let left = s[(i + W - 13) % W] + s[(i + W - 7) % W] + s[(i + W - 1) % W];
    if (left >= 6) left -= 6; else if (left >= 3) left -= 3;
    let right = s[(i + 1) % W] + s[(i + 7) % W] + s[(i + 13) % W];
    if (right >= 6) right -= 6; else if (right >= 3) right -= 3;
    let sum = left + s[i] + right;
    if (sum >= 6) sum -= 6; else if (sum >= 3) sum -= 3;
    t[i] = sum;
  }
  s.set(t);
}

function tisPi(s: Uint8Array): void {
  const t = new Uint8Array(TIS_STATE);
  for (let i = 0; i < TIS_STATE; i++) { t[(i * 13) % TIS_STATE] = s[i]; }
  s.set(t);
}

function tisPermute(s: Uint8Array): void {
  for (let r = 0; r < TIS_ROUNDS; r++) {
    tisTheta(s);
    tisPi(s);
    s[0] = gf3Add(s[0], TIS_RC[r]);
  }
}

function deriveIdentityTrits(canonicalUrl: string): number[] {
  const state = new Uint8Array(TIS_STATE);

  const bytes = new TextEncoder().encode(canonicalUrl);
  const trits: number[] = [];
  for (const b of bytes) { trits.push(...tisByteToTrits(b)); }

  trits.push(1);
  while (trits.length % TIS_RATE !== TIS_RATE - 1) { trits.push(0); }
  trits.push(2);

  for (let off = 0; off < trits.length; off += TIS_RATE) {
    for (let i = 0; i < TIS_RATE; i++) {
      state[i] = gf3Add(state[i], trits[off + i] ?? 0);
    }
    tisPermute(state);
  }

  const out: number[] = [];
  for (let i = 0; i < TIS_RATE; i++) { out.push(state[i] + 1); }
  return out;
}

// ── TIS-27 Integrity Hash ─────────────────────────────────────────────────────
//
// Fast ternary integrity function. Mirrors ternary-math/src/tis_sponge.rs.
// NOT the identity sponge — different theta, different pi, different rounds.
//
//   Identity sponge (above):  27 rounds, 3-neighbor theta, scatter pi, XOR absorb
//   TIS-27 (below):           4 rounds,  7-neighbor theta, gather pi,  direct copy
//
// TIS-27 is for scan hashing and wire integrity. NOT for identity binding.

const TIS27_ROUNDS = 4;

const TIS27_PI: number[] = (() => {
  const t: number[] = new Array(TIS_STATE);
  for (let i = 0; i < TIS_STATE; i++) t[i] = (i * 13) % TIS_STATE;
  return t;
})();

const TIS27_RC_BASE = [0,0,1,1,2,1,1,1,0,2,0,2,1,0,0,1,1,2,1,1,1,0,2,0,2,1,0];
const TIS27_RC: number[][] = (() => {
  const rcs: number[][] = [];
  for (let r = 0; r < TIS27_ROUNDS; r++) {
    const row: number[] = new Array(27).fill(0);
    for (let i = 0; i < 27; i++) {
      row[i] = TIS27_RC_BASE[(i + r) % 27];
    }
    rcs.push(row);
  }
  return rcs;
})();

function gf3Add3(a: number, b: number, c: number): number {
  let s = a + b + c;
  if (s >= 6) return s - 6;
  if (s >= 3) return s - 3;
  return s;
}

function tis27Theta(s: Uint8Array): Uint8Array {
  const t = new Uint8Array(TIS_STATE);
  const W = TIS_STATE;
  for (let i = 0; i < W; i++) {
    const left = gf3Add3(
      s[(i + W - 13) % W],
      s[(i + W - 7) % W],
      s[(i + W - 1) % W],
    );
    const right = gf3Add3(
      s[(i + 1) % W],
      s[(i + 7) % W],
      s[(i + 13) % W],
    );
    t[i] = gf3Add3(left, s[i], right);
  }
  return t;
}

function tis27Pi(theta: Uint8Array): Uint8Array {
  const p = new Uint8Array(TIS_STATE);
  for (let i = 0; i < TIS_STATE; i++) {
    p[i] = theta[TIS27_PI[i]];
  }
  return p;
}

function tis27Permute(state: Uint8Array): void {
  for (let r = 0; r < TIS27_ROUNDS; r++) {
    const t = tis27Theta(state);
    const p = tis27Pi(t);
    const rc = TIS27_RC[r];
    for (let i = 0; i < 27; i++) {
      p[i] = gf3Add(p[i], rc[i]);
    }
    state.set(p);
  }
}

function canonicaliseUrl(raw: string): string {
  const noFrag  = raw.split("#")[0];
  const noQuery = noFrag.split("?")[0];
  try {
    const u    = new URL(noQuery);
    const host = u.hostname.toLowerCase();
    const isDefaultPort =
      (u.protocol === "https:" && (u.port === "443" || u.port === "")) ||
      (u.protocol === "http:"  && (u.port === "80"  || u.port === ""));
    const port = isDefaultPort ? "" : (u.port ? `:${u.port}` : "");
    const path = u.pathname === "/" ? "" : u.pathname;
    return `${u.protocol}//${host}${port}${path}`;
  } catch {
    return noQuery.toLowerCase();
  }
}

// ── Types ─────────────────────────────────────────────────────────────────────

interface Dimension {
  number:     number;
  category:   string;
  question:   string;
  value:      number;    // Rep C trit: {1, 2, 3} — zero is NEVER valid
  label:      string;
  meaning:    string;    // From dimensions.json — (dim, trit) meaning sentence
  confidence: number;    // 1–9 pips
  polarity:   "higher_is_better" | "higher_is_worse" | "neutral";
  signal_count?: string; // e.g. "3 of 5 signals fired" (quantitative dims only)
}

interface TrackerCategory {
  id:               string;   // "analytics" | "social" | "advertising" | "session_replay" | "crm"
  name:             string;
  detected:         boolean;
  domains:          string[]; // matched third-party domains
  sensitivity:      string;   // "Medium" | "High" | "Critical"
  privacy_law:      string;
  finding_severity: "Critical" | "Warning" | "Info" | null;
}

interface HeaderAudit {
  header:           string;
  present:          boolean;
  value:            string;   // Truncated at 80 chars
  purpose:          string;
  dimension:        string;   // e.g. "D25"
  finding_severity: "Critical" | "Warning" | null;
}

interface Finding {
  id:        string;
  severity:  "Critical" | "Warning" | "Info";
  title:     string;
  message:   string;
  dimension?: string;
}

// SEO signal categories and statuses
type SeoStatus = "pass" | "warn" | "fail";

interface SeoSignal {
  id:             string;
  category:       "Discoverability" | "Metadata" | "Social" | "Technical";
  status:         SeoStatus;
  signal:         string;
  detail:         string;
  recommendation: string;
}

interface CookieFlag {
  name:     string;   // Cookie name (truncated, no value)
  secure:   boolean;
  httponly: boolean;
  samesite: string;   // "Strict" | "Lax" | "None" | "missing"
  issues:   string[]; // List of specific problems
}

interface TechSignal {
  header:     string;
  value:      string;
  risk:       "info" | "warn" | "critical";
  finding:    string;   // What this reveals
  recommendation: string;
}

interface Scores {
  trustIndex:          number; trustLabel:          string;
  privacyScore:        number; privacyLabel:        string;
  complexityScore:     number; complexityLabel:     string;
  maturityScore:       number; maturityLabel:       string;
  privacyFocusedIndex: number; pfiLabel:            string;  // "Data Trust" in UI
}

interface ScanResult {
  status:          string;
  address:         string;
  identity_trits:  number[];
  cguid:           number;
  scan_hash:       string;
  scan_hash_algo:  string;
  hptp_mandatory:  boolean;
  crd:             number;
  dimensions:      Dimension[];
  scores:          Scores;
  trackers:        TrackerCategory[];
  security_headers: HeaderAudit[];
  findings:        Finding[];
  seo_signals:     SeoSignal[];
  cookie_audit:    CookieFlag[];
  tech_fingerprint: TechSignal[];
  topology_svg:    string | null;  // Phase 2 — null until services/tdns-v2/src/topology.rs
  meta:            Record<string, any>;
  scannedAt:       string;
}

interface RegistryEntry extends ScanResult {
  name:          string;
  zone:          string;
  url:           string;
  canonical_url: string;
  org_name?:     string;
  registered_at: string;
}

// ── Multi-URL Org Entity ───────────────────────────────────────────────────────

interface OrgMember {
  url:             string;
  canonical_url:   string;
  plm_name:        string;
  address:         string;
  identity_trits:  number[];
  cguid:           number;
  added_at:        string;
}

interface OrgEntity {
  org_name:        string;
  display_name?:   string;
  classification_address: string;
  members:         OrgMember[];
  created_at:      string;
  updated_at:      string;
}

// ── In-memory registry ────────────────────────────────────────────────────────
// Primary index: .plm name → entry
const registry    = new Map<string, RegistryEntry>();
// Secondary index: normalised URL → .plm name (dedup)
const urlIndex    = new Map<string, string>();
// Org registry: org handle → OrgEntity (multi-URL grouping)
const orgRegistry = new Map<string, OrgEntity>();
// Reverse index: .plm name → org handle
const orgIndex    = new Map<string, string>();

/** Normalise URL for dedup: strip trailing slash, lowercase host, ignore query/fragment. */
function normaliseRegistryUrl(raw: string): string {
  try {
    const u = new URL(raw);
    return `${u.protocol}//${u.hostname.toLowerCase()}${u.pathname.replace(/\/$/, "") || "/"}`;
  } catch {
    return raw.toLowerCase().replace(/\/$/, "");
  }
}

// ── GF(3) derivation — exact formula from services/tdns-v2/src/derive.rs ──────
// gf3 = min(floor(3k/N), 2)   trit = gf3 + 1  →  Rep C {1,2,3}
// Zero is NEVER a valid trit. Zero = sentinel = forgery.
function gf3(k: number, n: number): number {
  return Math.min(Math.floor(3 * k / n), 2) + 1;
}
function confDigit(k: number, n: number): number {
  const p = k / n;
  const d = Math.min(Math.abs(p - 1/3), Math.abs(p - 2/3));
  return Math.min(Math.floor(27 * d) + 1, 9);
}

// Normalize trit to 0-100 score: trit1=0, trit2=50, trit3=100
function norm(t: number): number { return (t - 1) * 50; }
// Inverted: higher trit = worse outcome → lower score
function inv(t: number): number  { return (3 - t) * 50; }

function scoreLabel(s: number): string {
  if (s >= 90) return "Excellent";
  if (s >= 75) return "Strong";
  if (s >= 60) return "Good";
  if (s >= 40) return "Fair";
  return "Poor";
}

// ── Meaning sentences — canonical (dim, trit) pairs ──────────────────────────
// Full set in extension/dimensions.json. Server keeps compact inline copy.
const MEANINGS: Record<string, [string, string, string]> = {
  D1:  ["Individual person or personal project — single operator, minimal accountability surface.",
        "Commercial or business entity operating for profit — registered company or startup.",
        "Government body, public institution, or educational establishment — highest accountability."],
  D2:  ["Restricted to a single person or device — private by design.",
        "Shared within a closed group, organisation, or team — not publicly listed.",
        "Open to the public internet — reachable by anyone with a URL."],
  D3:  ["Operator identity is anonymous or undisclosed — no verifiable accountability.",
        "Operator is identifiable but limited public disclosure — company name known.",
        "Fully transparent — legal entity, contact, and physical address publicly stated."],
  D4:  ["Self-hosted on own hardware or privately managed infrastructure.",
        "Hosted by a third-party provider — shared or dedicated infrastructure.",
        "Cloud-native — running on major public cloud or edge network."],
  D5:  ["Static website — HTML/CSS content delivery, no dynamic API surface.",
        "Dynamic application or API — server-side logic, endpoints, or data exchange.",
        "Device or embedded endpoint — IoT, hardware interface, or firmware service."],
  D6:  ["Text-based content — HTML documents, markdown, or structured data responses.",
        "Rich media — images, video, or audio served as primary content type.",
        "Live stream — real-time audio/video or continuous data emission."],
  D7:  ["Designed for human users — browser-rendered HTML, UI-first.",
        "Designed for machine consumers — API responses, structured data, no UI.",
        "Hybrid — serves both humans (UI) and machines (API) from the same surface."],
  D8:  ["No AI or machine learning signals detected — deterministic software only.",
        "Partial AI presence — recommendation, ranking, or inference endpoint detected.",
        "Full AI integration — model serving, LLM, or autonomous inference confirmed."],
  D9:  ["Access is restricted or blocked — returns error, requires VPN, or is private.",
        "Partially accessible — requires authentication or returns partial content.",
        "Fully public — accessible to any internet-connected client without credentials."],
  D10: ["No authentication required — content is open to anonymous access.",
        "Password-based authentication — standard login form or HTTP Basic/Digest.",
        "Strong identity verification — MFA, OAuth, certificate, or government ID."],
  D11: ["Single server or minimal infrastructure — no redundancy detected.",
        "Multiple servers or regional distribution — some redundancy present.",
        "CDN or global edge network — highly distributed, anycast delivery."],
  D12: ["Standard HTTP/HTTPS — request/response model only.",
        "WebSocket — persistent bidirectional connection established.",
        "Raw TCP or custom protocol — transport-layer service detected."],
  D13: ["Pre-2010 technology stack — legacy headers, no modern browser security features.",
        "2010s technology stack — some modern headers present, partial adoption.",
        "2020s technology stack — full modern header suite, HTTP/3, NEL, reporting APIs."],
  D14: ["Business hours availability — maintenance windows or limited uptime signals.",
        "Extended availability — beyond business hours but not continuous.",
        "24/7 continuous operation — high uptime signals, no maintenance indicators."],
  D15: ["Historical or cached data — content is static or infrequently updated.",
        "Current data — regularly refreshed, moderate cache TTL.",
        "Live data — real-time feed, zero cache, WebSocket or SSE delivery."],
  D16: ["Batch processing — scheduled updates, high cache-control TTL, no streaming.",
        "Near-real-time — low cache TTL or short polling interval detected.",
        "Real-time — WebSocket, SSE, or gRPC streaming active."],
  D17: ["No financial transaction signals — informational or content-only service.",
        "Accepts payment — checkout, e-commerce, or payment gateway detected.",
        "Processes financial transactions — wire transfers, ACH, or securities trading."],
  D18: ["Minimal data collection — no tracking scripts, no forms, no collection signals.",
        "Moderate data collection — standard analytics and form inputs present.",
        "Heavy data collection — multiple tracker categories and extensive form capture."],
  D19: ["No legal or policy pages detected — no privacy policy, no terms of service.",
        "Basic policies present — privacy policy or terms exist but limited.",
        "Comprehensive legal framework — privacy policy, ToS, cookie notice, GDPR compliance."],
  D20: ["Free service — no payment or subscription signals detected.",
        "Pay-per-use model — transactional pricing or credit-based system.",
        "Subscription model — monthly/annual plans, recurring billing detected."],
  D21: ["Unicast delivery — single-origin direct connection, no CDN or proxy.",
        "Relay or proxy — traffic routed through an intermediary layer.",
        "Anycast or edge delivery — CDN with global point-of-presence routing."],
  D22: ["Outbound delivery — server pushes content to client, one-way flow.",
        "Relay or bidirectional — traffic passes through an intermediary or flows both ways.",
        "Inbound data collection — client submits data to server as primary interaction."],
  D23: ["Pull model — client must request updates, no server-initiated delivery.",
        "Subscribe model — RSS, Atom, or webhook subscription mechanism detected.",
        "Push model — server initiates delivery via WebSocket, SSE, or push notification."],
  D24: ["Stateless — no session cookies or persistent state detected.",
        "Short session — session cookie present, expires at browser close or within 24 hours.",
        "Long-lived session — persistent cookie with expiry beyond 30 days detected."],
  D25: ["Weak or no encryption — HTTP only or minimal security header coverage.",
        "Basic TLS — HTTPS present but security headers are incomplete.",
        "Hardened TLS — HTTPS + full security header suite (HSTS, CSP, XCTO, XFO)."],
  D26: ["Heavy tracker presence — multiple tracking categories detected in initial response.",
        "Moderate tracking — one or two tracker categories detected.",
        "Clean — no known tracker categories detected in the initial HTTP response."],
  D27: ["No security audit evidence — no bug bounty, penetration test, or certification.",
        "Self-assessed — bug bounty program or self-certification mentioned.",
        "Third-party audited — ISO 27001, SOC 2, PCI DSS, or equivalent certification."],
};

// ── Security header definitions ───────────────────────────────────────────────
const SECURITY_HEADERS = [
  { name: "strict-transport-security", dim: "D25", purpose: "Forces HTTPS for duration of max-age. Preload = hardcoded in browsers.", critical_if_missing: true },
  { name: "content-security-policy",   dim: "D25,D13", purpose: "Prevents XSS by whitelisting script/style sources.", critical_if_missing: false },
  { name: "x-content-type-options",    dim: "D25", purpose: "Prevents MIME-type sniffing. Value must be 'nosniff'.", critical_if_missing: false },
  { name: "x-frame-options",           dim: "D25", purpose: "Blocks clickjacking via iframe embedding.", critical_if_missing: false },
  { name: "permissions-policy",        dim: "D13", purpose: "Controls browser feature access (camera, mic, geolocation).", critical_if_missing: false },
  { name: "cross-origin-opener-policy",dim: "D13", purpose: "Isolates browsing context — Spectre mitigation.", critical_if_missing: false },
  { name: "cross-origin-embedder-policy",dim:"D13",purpose: "Required with COOP for SharedArrayBuffer access.", critical_if_missing: false },
  { name: "cross-origin-resource-policy",dim:"D13",purpose: "Prevents cross-origin resource reads.", critical_if_missing: false },
  { name: "referrer-policy",           dim: "D18,D26", purpose: "Controls Referer header leakage on navigation.", critical_if_missing: false },
  { name: "nel",                       dim: "D13", purpose: "Network Error Logging — modern observability signal.", critical_if_missing: false },
  { name: "report-to",                 dim: "D13", purpose: "Reporting API endpoint for CSP, NEL, and COOP violations.", critical_if_missing: false },
  { name: "x-powered-by",             dim: "D14", purpose: "Exposes server framework version — information disclosure risk.", critical_if_missing: false },
];

// ── Tracker category detection signals ────────────────────────────────────────
const TRACKER_CATEGORIES = [
  {
    id: "analytics", name: "Analytics",
    sensitivity: "Medium", privacy_law: "GDPR Art. 6(1)(f) — consent required",
    domains: ["google-analytics.com","googletagmanager.com","mixpanel.com","amplitude.com","heap.io","segment.com","hotjar.com"],
    patterns: ["google-analytics","gtag(","mixpanel","amplitude","heap.load","segment.","_paq.push","matomo"],
    finding_severity: "Info" as const,
  },
  {
    id: "social", name: "Social Trackers",
    sensitivity: "High", privacy_law: "GDPR Art. 9 — inferred sensitive attributes via social graph",
    domains: ["connect.facebook.net","platform.twitter.com","snap.licdn.com","static.ads-twitter.com","sc-static.net"],
    patterns: ["fbq(","facebook.net","platform.twitter","snap.licdn","_linkedin_data","ttq.","snaptr("],
    finding_severity: "Warning" as const,
  },
  {
    id: "advertising", name: "Advertising",
    sensitivity: "High", privacy_law: "GDPR Art. 22 — automated profiling",
    domains: ["doubleclick.net","googlesyndication.com","adnxs.com","adsrvr.org","rubiconproject.com","pubmatic.com"],
    patterns: ["doubleclick","googlesyndication","googletag.cmd","adnxs","adsystem","__tcfapi","prebid"],
    finding_severity: "Warning" as const,
  },
  {
    id: "session_replay", name: "Session Replay",
    sensitivity: "Critical", privacy_law: "GDPR Art. 5(1)(f) — integrity/confidentiality breach risk",
    domains: ["static.hotjar.com","edge.fullstory.com","logrocket.io","cdn.lr-ingest.io","clarity.ms"],
    patterns: ["hotjar","fullstory","logrocket","clarity.ms","hj(","FS(","LogRocket.init","clarity("],
    finding_severity: "Critical" as const,
  },
  {
    id: "crm", name: "CRM / Marketing",
    sensitivity: "Medium", privacy_law: "PIPEDA s.4.3 / CASL s.6 — consent for collection",
    domains: ["js.hs-scripts.com","munchkin.marketo.net","js.intercomcdn.com","pardot.com","bat.bing.com"],
    patterns: ["_hsq","hubspot","marketo","intercom","pardot","MktoForms","drift(","clearbit"],
    finding_severity: "Info" as const,
  },
];

// ── Scanner ───────────────────────────────────────────────────────────────────
async function scanUrl(rawUrl: string): Promise<ScanResult> {
  if (!/^https?:\/\//i.test(rawUrl)) rawUrl = "https://" + rawUrl;
  const parsed   = new URL(rawUrl);
  const hostname = parsed.hostname.toLowerCase();
  const isHttps  = parsed.protocol === "https:";

  let hdrs: Record<string, string> = {};
  let statusCode = 0;
  let body       = "";
  let ok         = false;

  try {
    const resp = await fetch(rawUrl, {
      method:   "GET",
      redirect: "follow",
      signal:   AbortSignal.timeout(10000),
      headers:  {
        "User-Agent":      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        "Accept":          "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.5",
        "Accept-Encoding": "identity",
      },
    });
    ok         = resp.ok;
    statusCode = resp.status;
    resp.headers.forEach((v: string, k: string) => { hdrs[k.toLowerCase()] = v; });
    const buf  = await resp.arrayBuffer();
    body       = new TextDecoder().decode(buf.slice(0, 32768)).toLowerCase();
    log.info(`Scanned ${hostname}: HTTP ${statusCode}, ${body.length}b`);
  } catch (err: any) {
    log.warn(`Fetch failed for ${hostname}: ${err.message}`);
  }

  const h   = (k: string) => hdrs[k] || "";
  const has = (k: string) => !!hdrs[k];
  const b   = (p: string) => body.includes(p);

  // ── Build metadata ────────────────────────────────────────────────────────
  const meta: Record<string, any> = {
    url: rawUrl, hostname,
    isHttps:    statusCode > 0 && isHttps,
    statusCode, ok,
    server:     (h("server") || h("x-powered-by") || "").substring(0, 80),
    ct:         h("content-type"),
    cacheCtrl:  h("cache-control"),
    hsts:       h("strict-transport-security"),
    csp:        h("content-security-policy").substring(0, 80),
    cors:       h("access-control-allow-origin"),
    xfo:        h("x-frame-options"),
    xcto:       h("x-content-type-options"),
    via:        h("via"),
    cfRay:      has("cf-ray"),
    xCache:     has("x-cache") || has("x-amz-cf-id") || has("x-served-by"),
    altSvc:     h("alt-svc"),
    permsPolicy:h("permissions-policy"),
    nel:        has("nel"),
    reportTo:   has("report-to") || has("reporting-endpoints"),
    coep:       has("cross-origin-embedder-policy"),
    coop:       has("cross-origin-opener-policy"),
    corp:       has("cross-origin-resource-policy"),
    setCookie:  h("set-cookie"),
    bodySize:   body.length,
    hasHtml:    b("<!doctype") || h("content-type").includes("html"),
    hasJson:    h("content-type").includes("json"),
    isApi:      parsed.pathname.includes("/api/"),
    isWs:       h("upgrade").toLowerCase() === "websocket",
    isSse:      h("content-type").includes("event-stream"),
  };

  const isGov      = /\.(gov|mil|gc\.ca|gov\.uk|edu|ac\.uk)$/.test(hostname);
  const isCloud    = ["amazonaws","cloudflare","vercel","netlify","replit","fastly",
    "azurewebsites","googleusercontent","github.io","pages.dev","fly.dev","render.com",
    "google.com","youtube.com","facebook.com","meta.com","x.com","twitter.com",
    "microsoft.com","apple.com","amazon.com","netflix.com","linkedin.com","github.com",
  ].some(s => hostname === s || hostname.endsWith("." + s));

  const ct         = meta.ct as string;
  const cacheCtrl  = meta.cacheCtrl as string;
  const cookieVal  = meta.setCookie as string;
  const maxAgeM    = cacheCtrl.match(/max-age=(\d+)/);
  const maxAge     = maxAgeM ? parseInt(maxAgeM[1]) : 3600;

  // ── 27 Derivations — exact signal counts from services/tdns-v2/src/derive.rs ─

  const d1  = isGov ? 3 : 2;
  const d2  = 3;
  const k3  = +(b("/about")||b("about us")||b("our company"))
            + +(b("contact")||b("get in touch"))
            + +(b("inc.")||b(" ltd")||b("llc")||b("corp."))
            + +(b("street")||b("avenue")||b("suite")||b("postal code"))
            + +(isGov || isCloud);
  const d3  = gf3(k3, 5);  const c3  = confDigit(k3, 5);
  const d4  = isCloud ? 3 : 2;
  const d5  = (meta.isApi && !meta.hasHtml) ? 2 : 1;
  const d6  = meta.isWs || meta.isSse ? 3 : ct.includes("video")||ct.includes("image") ? 2 : 1;
  const d7  = meta.hasHtml && (meta.hasJson||meta.isApi) ? 3 : meta.hasJson||meta.isApi ? 2 : 1;
  const k8  = +(b("/predict")||b("/inference")||b("/model")||b("/embed")||b("/generate"))
            + +(b("tensorflow")||b("pytorch")||b("openai")||b("llm")||b("hugging face"))
            + +(b("recommendation")||b("personali")||b("suggested for you"))
            + +(b("ranking")||b("relevance score"))
            + +(has("x-ai")||has("x-model")||has("x-inference"));
  const d8  = gf3(k8, 5);  const c8  = confDigit(k8, 5);
  const k9  = +(ok||statusCode>0) + +(statusCode!==401&&statusCode!==403&&!has("www-authenticate")) + +(ok&&(meta.hasHtml||meta.hasJson));
  const d9  = gf3(k9, 3);  const c9  = confDigit(k9, 3);
  const d10 = has("www-authenticate") ? 2 : b("sign in")||b("log in")||b("login required") ? 2 : 1;
  const k11 = +(ok||statusCode>0) + +(hostname.split(".").length>=3) + +(hostname.split(".").length>=4)
            + +(meta.cfRay||has("x-cdn")||has("x-fastly")||isCloud) + +(meta.xCache||has("age")) + +(meta.via!=="");
  const d11 = gf3(k11, 6); const c11 = confDigit(k11, 6);
  const d12 = meta.isWs ? 2 : 1;
  const k13 = +(has("alt-svc")) + +(has("permissions-policy")) + +(has("nel"))
            + +(has("report-to")||has("reporting-endpoints")) + +(meta.coep||meta.coop) + +(has("content-security-policy"));
  const d13 = gf3(k13, 6); const c13 = confDigit(k13, 6);
  const k14 = +(!b("maintenance mode")&&!b("down for maintenance")) + +(!b("business hours")&&!b("office hours")) + +(b("99.")||b("uptime")||ok);
  const d14 = gf3(k14, 3); const c14 = confDigit(k14, 3);
  const d15 = meta.isWs||meta.isSse ? 3 : cacheCtrl.includes("max-age=0")||cacheCtrl.includes("no-store") ? 2 : 2;
  const k16 = +(!cacheCtrl.includes("immutable")&&maxAge<3600) + +(maxAge<60) + +(meta.isWs) + +(meta.isSse) + +(ct.includes("grpc")) + +(ct.includes("octet-stream")&&b("stream"));
  const d16 = gf3(k16, 6); const c16 = confDigit(k16, 6);
  const d17 = b("swift")||b("wire transfer") ? 3 : b("stripe")||b("paypal")||b("checkout")||b("buy now") ? 2 : 1;
  const k18 = +(b("<input")||b("input type=")) + +(b("sign up")||b("register")||b("create account"))
            + +(b("gtag(")||b("analytics.js")||b("fbq(")||b("mixpanel")) + +(b("cookie consent")||b("gdpr")||b("we use cookies"))
            + +(b("third party")||b("data partner"));
  const d18 = gf3(k18, 5); const c18 = confDigit(k18, 5);
  const k19 = +(b("privacy policy")||b("/privacy")) + +(b("terms of service")||b("terms of use")||b("/terms"))
            + +(b("cookie policy")||b("cookie notice")) + +(b("gdpr")||b("data protection")) + +(b("accessibility")||b("wcag"));
  const d19 = gf3(k19, 5); const c19 = confDigit(k19, 5);
  const d20 = b("subscribe")||b("per month")||b("/month") ? 3 : b("pay per")||b("credits") ? 2 : 1;
  const d21 = isCloud||meta.cfRay||meta.xCache ? 3 : meta.via!=="" ? 2 : 1;
  const d22 = meta.hasJson&&(b('"data"')||b('"result"')) ? 3 : meta.via||meta.cfRay ? 2 : 1;
  const d23 = meta.isWs||meta.isSse ? 3 : b("rss")||b("atom")||b("</feed") ? 2 : 1;
  const hasCookie = cookieVal !== "";
  const hasExpires= cookieVal.includes("expires=")||cookieVal.includes("max-age=");
  const longLived = (parseInt((cookieVal.match(/max-age=(\d+)/i)||[,"0"])[1]))>86400*30
                 || (cookieVal.includes("expires=")&&!cookieVal.includes("session"));
  const k24 = +hasCookie + +hasExpires + +longLived;
  const d24 = gf3(k24, 3); const c24 = confDigit(k24, 3);
  const k25 = +(meta.isHttps) + +(has("strict-transport-security")) + +(has("content-security-policy"))
            + +(b("security.txt")||b("/.well-known/security")) + +(has("x-content-type-options")) + +(has("x-frame-options"));
  const d25 = gf3(k25, 6); const c25 = confDigit(k25, 6);
  const k26 = +(!b("google-analytics")&&!b("gtag(")&&!b("mixpanel")) + +(!b("facebook.net")&&!b("platform.twitter")&&!b("snap.licdn"))
            + +(!b("doubleclick")&&!b("googlesyndication")&&!b("adsystem")) + +(!b("fullstory")&&!b("hotjar")&&!b("logrocket")&&!b("clarity.ms"))
            + +(!b("hubspot")&&!b("marketo")&&!b("intercom")&&!b("pardot"));
  const d26 = gf3(k26, 5); const c26 = confDigit(k26, 5);
  const d27 = b("iso 27001")||b("soc 2")||b("pci dss") ? 3 : b("penetration test")||b("bug bounty") ? 2 : 1;

  const trits = [d1,d2,d3,d4,d5,d6,d7,d8,d9,d10,d11,d12,d13,d14,d15,d16,
                 d17,d18,d19,d20,d21,d22,d23,d24,d25,d26,d27];
  const confs  = [9,9,c3,9,9,9,9,c8,c9,9,c11,9,c13,c14,9,c16,
                  9,c18,c19,9,9,9,9,c24,c25,c26,9];

  // ── Address ────────────────────────────────────────────────────────────────
  const classAddr = [
    `WO:${trits.slice(0,4).join("")}`, `WA:${trits.slice(4,8).join("")}`,
    `WR:${trits.slice(8,12).join("")}`, `WN:${trits.slice(12,16).join("")}`,
    `WY:${trits.slice(16,20).join("")}`, `HO:${trits.slice(20,24).join("")}`,
    `PE:${trits.slice(24,27).join("")}`,
  ].join(" ");

  const canonical      = canonicaliseUrl(rawUrl);
  const identity_trits = deriveIdentityTrits(canonical);
  const address        = `${classAddr} · ID:${identity_trits.join("")}`;

  // ── Scan hash via TIS-27 (4-round, 7-neighbor, gather pi) ──────────────
  // Direct copy absorption — matches ternary-math/src/tis_sponge.rs exactly.
  // Identity derivation (above) stays on the 27-round identity sponge.
  const scanTritsGf3 = trits.map((t: number) => t - 1);

  const scanState = new Uint8Array(TIS_STATE);
  const scanBlock = Math.min(TIS_RATE, scanTritsGf3.length);
  for (let i = 0; i < scanBlock; i++) {
    scanState[i] = scanTritsGf3[i];
  }

  tis27Permute(scanState);

  const scanTritsOut = Array.from({ length: TIS_RATE }, (_, i) => scanState[i] + 1);

  const scanBytes: number[] = [];
  const scanSqueeze = new Uint8Array(scanState);
  while (scanBytes.length < 32) {
    for (let i = 0; i + 4 < TIS_STATE; i += 5) {
      if (scanBytes.length >= 32) break;
      scanBytes.push(
        scanSqueeze[i] * 81 + scanSqueeze[i+1] * 27 +
        scanSqueeze[i+2] * 9 + scanSqueeze[i+3] * 3 + scanSqueeze[i+4]
      );
    }
    if (scanBytes.length < 32) tis27Permute(scanSqueeze);
  }
  const scan_hash = scanBytes.slice(0, 32)
    .map((b: number) => b.toString(16).padStart(2, "0")).join("");

  const crd = (scanTritsOut[0] - 1) * 3 + scanTritsOut[1];

  // ── 5 Scores ───────────────────────────────────────────────────────────────
  const whoAvg  = (norm(d1)+norm(d2)+norm(d3)+norm(d4)) / 4;
  const peaceAvg= (norm(d25)+norm(d26)+norm(d27)) / 3;
  const whyAvg  = (norm(d17)+inv(d18)+norm(d19)+norm(d20)) / 4;
  const whenAvg = (norm(d13)+norm(d14)+norm(d15)+norm(d16)) / 4;
  const compAvg = (norm(d9)+norm(d10)+norm(d11)+norm(d12)+norm(d21)+norm(d22)) / 6;

  const trustIndex  = Math.round(0.35*whoAvg + 0.30*peaceAvg + 0.20*whyAvg + 0.10*whenAvg + 0.05*compAvg);
  const privacyScore= Math.round((inv(d18)+norm(d19)+inv(d24)+norm(d26)+inv(d8)) / 5);
  const complexityScore = Math.round(compAvg);
  const maturityScore   = Math.round((norm(d13)+norm(d14)+norm(d25)+norm(d27)+norm(d5)) / 5);
  const privacyFocusedIndex = Math.round(Math.max(0, Math.min(100, trustIndex + (2-d8) * (100/9))));

  const scores: Scores = {
    trustIndex,          trustLabel:          scoreLabel(trustIndex),
    privacyScore,        privacyLabel:        scoreLabel(privacyScore),
    complexityScore,     complexityLabel:     scoreLabel(complexityScore),
    maturityScore,       maturityLabel:       scoreLabel(maturityScore),
    privacyFocusedIndex, pfiLabel:            scoreLabel(privacyFocusedIndex),
  };

  // ── SCHEMA ─────────────────────────────────────────────────────────────────
  const SCHEMA = [
    {cat:"WHO",  q:"What kind of entity?",      vals:["Personal","Corporate","Governance"],       pol:"neutral"},
    {cat:"WHO",  q:"Who's the audience?",        vals:["Just me","My group","Everyone"],            pol:"neutral"},
    {cat:"WHO",  q:"Who operates it?",           vals:["Anonymous","Known","Transparent"],          pol:"higher_is_better"},
    {cat:"WHO",  q:"Hosting model?",             vals:["Self-hosted","Provider","Cloud"],           pol:"neutral"},
    {cat:"WHAT", q:"What form factor?",          vals:["Website","App / API","Device"],             pol:"neutral"},
    {cat:"WHAT", q:"Content type?",              vals:["Text / HTML","Media","Live stream"],        pol:"neutral"},
    {cat:"WHAT", q:"Primary consumer?",          vals:["Humans","Machines","Both"],                 pol:"neutral"},
    {cat:"WHAT", q:"AI / ML present?",           vals:["No","Partially","Yes"],                     pol:"neutral"},
    {cat:"WHERE",q:"Visibility?",                vals:["Private","Group","Public"],                 pol:"neutral"},
    {cat:"WHERE",q:"Authentication required?",   vals:["None","Password","Strong ID"],              pol:"neutral"},
    {cat:"WHERE",q:"Infrastructure scale?",      vals:["Single server","Several","CDN / Many"],    pol:"higher_is_better"},
    {cat:"WHERE",q:"Transport protocol?",        vals:["HTTP","WebSocket","Raw TCP"],               pol:"neutral"},
    {cat:"WHEN", q:"Technology era?",            vals:["Pre-2010","2010s","2020s+"],                pol:"higher_is_better"},
    {cat:"WHEN", q:"Availability window?",       vals:["Business hours","Extended","24/7"],        pol:"higher_is_better"},
    {cat:"WHEN", q:"Data freshness?",            vals:["Historical","Current","Live"],              pol:"higher_is_better"},
    {cat:"WHEN", q:"Real-time capability?",      vals:["Batch","Near-real-time","Real-time"],      pol:"neutral"},
    {cat:"WHY",  q:"Financial transactions?",    vals:["None","Accepts payment","Processes"],       pol:"neutral"},
    {cat:"WHY",  q:"Data collection appetite?",  vals:["Minimal","Moderate","Heavy"],               pol:"higher_is_worse"},
    {cat:"WHY",  q:"Legal / policy presence?",   vals:["None","Basic","Comprehensive"],             pol:"higher_is_better"},
    {cat:"WHY",  q:"Revenue model?",             vals:["Free","Pay-per-use","Subscription"],        pol:"neutral"},
    {cat:"HOW",  q:"Delivery topology?",         vals:["Unicast","Multicast","Anycast / Edge"],    pol:"higher_is_better"},
    {cat:"HOW",  q:"Data flow direction?",       vals:["Outbound","Relay / Proxy","Inbound"],      pol:"neutral"},
    {cat:"HOW",  q:"Update mechanism?",          vals:["Pull / Poll","Subscribe","Push"],           pol:"neutral"},
    {cat:"HOW",  q:"Session persistence?",       vals:["Stateless","Short session","Long-lived"],   pol:"higher_is_worse"},
    {cat:"PEACE",q:"Encryption posture?",        vals:["Weak / None","Basic TLS","Hardened TLS"],  pol:"higher_is_better"},
    {cat:"PEACE",q:"Tracker density?",           vals:["Heavy","Moderate","Clean"],                 pol:"higher_is_better"},
    {cat:"PEACE",q:"Security audit status?",     vals:["None","Self-assessed","Third-party"],       pol:"higher_is_better"},
  ];

  const dimensions: Dimension[] = SCHEMA.map((s, i) => ({
    number:      i + 1,
    category:    s.cat,
    question:    s.q,
    value:       trits[i],
    label:       s.vals[trits[i] - 1],
    meaning:     MEANINGS[`D${i+1}`]?.[trits[i]-1] || `${s.cat} D${i+1} trit ${trits[i]}.`,
    confidence:  confs[i],
    polarity:    s.pol as Dimension["polarity"],
  }));

  // ── Security Header Audit ─────────────────────────────────────────────────
  const security_headers: HeaderAudit[] = SECURITY_HEADERS.map(def => {
    const rawVal   = hdrs[def.name] || "";
    const present  = !!rawVal;
    const value    = rawVal.substring(0, 80) + (rawVal.length > 80 ? "…" : "");
    // x-powered-by: presence is the risk, not absence
    const isRisk   = def.name === "x-powered-by";
    let finding_severity: "Critical" | "Warning" | null = null;
    if (isRisk && present)      finding_severity = "Warning";
    else if (!present && def.critical_if_missing) finding_severity = "Critical";
    else if (!present && !isRisk && !def.critical_if_missing && def.name !== "nel" && def.name !== "report-to")
                                finding_severity = "Warning";
    return { header: def.name, present, value: present ? value : "—", purpose: def.purpose, dimension: def.dim, finding_severity };
  });

  // ── Tracker Intelligence ───────────────────────────────────────────────────
  const trackers: TrackerCategory[] = TRACKER_CATEGORIES.map(cat => {
    const detectedDomains = cat.domains.filter(d => body.includes(d.replace(/^www\./, "")));
    const detectedPatterns= cat.patterns.some(p => body.includes(p));
    const detected        = detectedDomains.length > 0 || detectedPatterns;
    const finding_severity= detected ? cat.finding_severity : null;
    return {
      id: cat.id, name: cat.name, detected,
      domains: detectedDomains,
      sensitivity: cat.sensitivity,
      privacy_law: cat.privacy_law,
      finding_severity,
    };
  });

  // ── Findings Engine ────────────────────────────────────────────────────────
  const findings: Finding[] = [];
  let findingId = 0;
  const finding = (severity: Finding["severity"], title: string, message: string, dimension?: string) => {
    findings.push({ id: `F${++findingId}`, severity, title, message, dimension });
  };

  // HTTP (no HTTPS)
  if (!meta.isHttps && statusCode > 0) {
    finding("Critical", "No HTTPS", "Site is served over unencrypted HTTP. All traffic is visible to network observers. Passwords, sessions, and content can be intercepted.", "D25");
  }
  // HSTS missing on HTTPS
  if (meta.isHttps && !has("strict-transport-security")) {
    finding("Warning", "HSTS Missing", "Strict-Transport-Security header is absent. Browsers will not enforce HTTPS on future visits — vulnerable to SSL stripping.", "D25");
  }
  // CSP missing
  if (!has("content-security-policy")) {
    finding("Warning", "No Content-Security-Policy", "CSP is absent. Without CSP, successful XSS attacks have unrestricted script execution. Critical for any site handling user data.", "D25");
  }
  // Session replay — ALWAYS Critical per spec §3.8
  const sessionReplay = trackers.find(t => t.id === "session_replay");
  if (sessionReplay?.detected) {
    finding("Critical", "Session Replay Detected", "Session replay tool detected. This technology records keystrokes, mouse movements, and form input — including content entered but not submitted. Passwords and sensitive data may be captured. Explicit user disclosure and consent are required under GDPR Art. 5(1)(f) and ICO guidance.", "D26");
  }
  // Social trackers
  if (trackers.find(t => t.id === "social")?.detected) {
    finding("Warning", "Social Tracker Detected", "Social tracker pixels detected. These scripts link your identity across sites via the social network's identity graph. High risk of inferred sensitive attributes.", "D26");
  }
  // Advertising trackers
  if (trackers.find(t => t.id === "advertising")?.detected) {
    finding("Warning", "Advertising Network Detected", "Advertising network scripts detected. Your browsing is being included in RTB (real-time bidding) auction data. GDPR Art. 22 automated profiling applies.", "D26");
  }
  // No CDN (bare origin)
  if (d21 === 1) {
    finding("Warning", "No CDN Layer", "No CDN or proxy layer detected. Origin server is directly reachable from the public internet. DDoS mitigation depends entirely on origin capacity.", "D21");
  }
  // Long-lived session cookies
  if (d24 === 3) {
    finding("Info", "Long-Lived Session Cookie", "A persistent cookie with expiry beyond 30 days was detected. Long-lived cookies increase cross-session tracking exposure.", "D24");
  }
  // x-powered-by exposure
  if (has("x-powered-by")) {
    finding("Warning", "Server Version Disclosed", `X-Powered-By: ${h("x-powered-by").substring(0, 60)} — Framework version exposure enables targeted exploit selection.`, "D14");
  }
  // XCTO missing
  if (!has("x-content-type-options")) {
    finding("Info", "X-Content-Type-Options Missing", "nosniff directive absent — MIME-type sniffing enabled in older browsers.", "D25");
  }

  // ── Cookie Security Audit ────────────────────────────────────────────────────
  // Parse all Set-Cookie headers (may be multi-value or comma-separated)
  const rawCookies = h("set-cookie") || "";
  const cookieLines: string[] = rawCookies
    .split(/,(?=[^;]+=[^;]*)/)   // naive split — good enough for flag audit
    .map((s: string) => s.trim())
    .filter(Boolean);

  const cookieAudit: CookieFlag[] = cookieLines.slice(0, 8).map((line: string) => {
    const parts  = line.split(";").map((p: string) => p.trim());
    const nameVal= parts[0] || "";
    const name   = nameVal.split("=")[0] || "(unnamed)";
    const flags  = parts.slice(1).map((p: string) => p.toLowerCase());

    const secure   = flags.some((f: string) => f === "secure");
    const httponly = flags.some((f: string) => f === "httponly");
    const ssFlag   = flags.find((f: string) => f.startsWith("samesite="));
    const samesite = ssFlag ? ssFlag.split("=")[1] || "missing" : "missing";

    const issues: string[] = [];
    if (!secure)                    issues.push("Missing Secure flag — cookie sent over HTTP");
    if (!httponly)                  issues.push("Missing HttpOnly flag — accessible via JavaScript");
    if (samesite === "missing")     issues.push("Missing SameSite — CSRF risk");
    if (samesite === "none" && !secure) issues.push("SameSite=None without Secure is rejected by modern browsers");

    return { name: name.substring(0, 32), secure, httponly, samesite, issues };
  });

  // ── Technology Fingerprint ────────────────────────────────────────────────────
  // Surface version/stack disclosure from response headers
  const techSignals: TechSignal[] = [];

  const fingerHeaders: Array<{ key: string; label: string }> = [
    { key: "server",          label: "Server"          },
    { key: "x-powered-by",   label: "X-Powered-By"    },
    { key: "x-generator",    label: "X-Generator"      },
    { key: "x-aspnet-version", label: "X-AspNet-Version" },
    { key: "x-aspnetmvc-version", label: "X-AspNetMvc-Version" },
    { key: "x-drupal-cache", label: "X-Drupal-Cache"  },
    { key: "x-wordpress-cache", label: "X-WordPress-Cache" },
    { key: "x-shopify-stage", label: "X-Shopify-Stage" },
  ];

  for (const fh of fingerHeaders) {
    const val = h(fh.key);
    if (!val) continue;

    // Detect version disclosure
    const hasVersion = /[0-9]+\.[0-9]/.test(val);
    const isPhp      = /php/i.test(val);
    const isAsp      = /asp\.net|aspnet/i.test(val);
    const isExpress  = /express/i.test(val);
    const isNginx    = /nginx/i.test(val);
    const isApache   = /apache/i.test(val);

    let risk: "info" | "warn" | "critical" = "info";
    let finding = `${fh.label}: ${val.substring(0, 60)}`;
    let recommendation = "No immediate action required.";

    if (hasVersion) {
      risk = "warn";
      finding = `${fh.label} discloses version: ${val.substring(0, 60)}`;
      recommendation = `Remove or mask the ${fh.label} header to prevent version enumeration. Attackers use version strings to target known CVEs.`;
    }
    if (isPhp && hasVersion) {
      risk = "critical";
      finding = `PHP version disclosed: ${val.substring(0, 60)}`;
      recommendation = "Remove X-Powered-By entirely via php.ini (expose_php = Off). PHP version exposure enables targeted exploits.";
    }
    if (isAsp && hasVersion) {
      risk = "critical";
      finding = `ASP.NET version disclosed: ${val.substring(0, 60)}`;
      recommendation = 'Remove ASP.NET version headers in web.config: <httpRuntime enableVersionHeader="false"> and <customHeaders><remove name="X-Powered-By"/></customHeaders>.';
    }
    if ((isNginx || isApache) && hasVersion) {
      risk = "warn";
      finding = `Web server version disclosed (${isNginx ? "nginx" : "Apache"}): ${val.substring(0, 60)}`;
      recommendation = isNginx
        ? "Set server_tokens off; in nginx.conf to suppress version."
        : "Set ServerTokens Prod and ServerSignature Off in Apache config.";
    }

    techSignals.push({ header: fh.key, value: val.substring(0, 80), risk, finding, recommendation });
  }

  // CDN detection from common headers
  const cdnHeaders: Array<[string, string]> = [
    ["cf-ray",      "Cloudflare"],
    ["x-amz-cf-id", "AWS CloudFront"],
    ["x-azure-ref", "Azure CDN"],
    ["x-fastly-request-id", "Fastly"],
    ["x-cache",     "Generic CDN/cache"],
  ];
  for (const [hkey, cdn] of cdnHeaders) {
    if (h(hkey)) {
      techSignals.push({
        header: hkey, value: h(hkey).substring(0, 40), risk: "info",
        finding: `CDN detected: ${cdn}`,
        recommendation: "No action needed. CDN presence is noted for infrastructure context.",
      });
      break; // report first match only
    }
  }

  // ── HPTP mandatory ────────────────────────────────────────────────────────
  const hptp_mandatory = trits[14] === 3 && trits[15] === 3;

  // ── SEO Analysis ──────────────────────────────────────────────────────────
  // All signals derived from the already-fetched body (lowercased, ≤32 KB).
  // Regex helpers operate on raw (case-preserving) body slice for length checks.
  const bodyRaw    = body;   // already lowercase from fetch
  const seoSignals: SeoSignal[] = [];
  let   seoId      = 0;
  const seo = (
    category: SeoSignal["category"],
    status:   SeoStatus,
    signal:   string,
    detail:   string,
    recommendation: string,
  ) => seoSignals.push({ id: `SEO${++seoId}`, category, status, signal, detail, recommendation });

  // Helper: extract first regex match value from body
  const rex = (pattern: RegExp): string => { const m = bodyRaw.match(pattern); return m ? m[1] || "" : ""; };

  // ── Discoverability ───────────────────────────────────────────────────────

  // Robots meta noindex / nofollow
  const robotsMeta = rex(/meta[^>]+name=["']robots["'][^>]+content=["']([^"']+)["']/i)
                  || rex(/meta[^>]+content=["']([^"']+)["'][^>]+name=["']robots["']/i);
  const isNoindex  = /noindex/.test(robotsMeta);
  const isNofollow = /nofollow/.test(robotsMeta);
  if (isNoindex) {
    seo("Discoverability","fail","Robots Meta — noindex",
      `<meta name="robots" content="${robotsMeta}"> prevents search engines from indexing this page.`,
      "Remove the noindex directive unless this page is intentionally excluded from search results.");
  } else {
    seo("Discoverability","pass","Robots Meta — indexable",
      robotsMeta ? `Robots meta present: "${robotsMeta}".` : "No robots meta found — defaults to indexable.",
      "No action needed.");
  }
  if (isNofollow) {
    seo("Discoverability","warn","Robots Meta — nofollow",
      "nofollow prevents PageRank flowing to linked pages.",
      "Use nofollow sparingly; omit on pages whose outbound links should pass authority.");
  }

  // Canonical URL
  const canonicalUrl = rex(/rel=["']canonical["'][^>]+href=["']([^"']+)["']/i)
                    || rex(/href=["']([^"']+)["'][^>]*rel=["']canonical["']/i);
  if (!canonicalUrl) {
    seo("Discoverability","warn","Canonical URL",
      'No <link rel="canonical"> found.',
      "Add a canonical tag pointing to the preferred URL to prevent duplicate content issues.");
  } else {
    const canonOk = canonicalUrl.includes(hostname);
    seo("Discoverability", canonOk ? "pass" : "warn", "Canonical URL",
      canonOk
        ? `Canonical tag present and points to same domain.`
        : `Canonical points to a different domain: ${canonicalUrl.substring(0,80)}.`,
      canonOk ? "No action needed." : "Verify the cross-domain canonical is intentional.");
  }

  // Sitemap reference in body
  const hasSitemapRef = b("sitemap") || b("sitemap.xml");
  seo("Discoverability", hasSitemapRef ? "pass" : "warn", "Sitemap Reference",
    hasSitemapRef ? "Sitemap reference detected in page source." : "No sitemap reference found in page source.",
    hasSitemapRef ? "Ensure sitemap.xml is submitted to Google Search Console." : "Add a link to your sitemap.xml in robots.txt and submit it to Search Console.");

  // ── Metadata ──────────────────────────────────────────────────────────────

  // Page Title
  const titleText = rex(/<title[^>]*>([^<]{1,200})<\/title>/i).trim();
  const titleLen  = titleText.length;
  if (!titleText) {
    seo("Metadata","fail","Page Title",
      "No <title> tag found.",
      "Add a descriptive title of 50–60 characters. It appears in search results and browser tabs.");
  } else if (titleLen < 30) {
    seo("Metadata","warn","Page Title",
      `Title found but very short (${titleLen} chars): "${titleText.substring(0,60)}".`,
      "Expand the title to 50–60 characters to give search engines more context.");
  } else if (titleLen > 60) {
    seo("Metadata","warn","Page Title",
      `Title is ${titleLen} characters — search engines truncate around 60: "${titleText.substring(0,70)}…".`,
      "Shorten the title to 50–60 characters so it displays in full in SERPs.");
  } else {
    seo("Metadata","pass","Page Title",
      `Title is ${titleLen} chars (optimal 50–60): "${titleText.substring(0,60)}".`,
      "No action needed.");
  }

  // Meta Description
  const descText = rex(/meta[^>]+name=["']description["'][^>]+content=["']([^"']{0,500})["']/i)
                || rex(/meta[^>]+content=["']([^"']{0,500})["'][^>]+name=["']description["']/i);
  const descLen  = descText.length;
  if (!descText) {
    seo("Metadata","fail","Meta Description",
      "No meta description found.",
      "Add a meta description of 120–160 characters summarising the page. It often appears as the SERP snippet.");
  } else if (descLen < 70) {
    seo("Metadata","warn","Meta Description",
      `Description is very short (${descLen} chars): "${descText.substring(0,80)}".`,
      "Expand to 120–160 characters for better SERP click-through rates.");
  } else if (descLen > 160) {
    seo("Metadata","warn","Meta Description",
      `Description is ${descLen} characters — search engines truncate around 160.`,
      "Trim to 120–160 characters.");
  } else {
    seo("Metadata","pass","Meta Description",
      `Description is ${descLen} chars (optimal 120–160): "${descText.substring(0,80)}…".`,
      "No action needed.");
  }

  // H1 tags
  const h1Matches = Array.from(bodyRaw.matchAll(/<h1[^>]*>([^<]{1,200})<\/h1>/gi));
  const h1Count   = h1Matches.length;
  if (h1Count === 0) {
    seo("Metadata","fail","H1 Heading",
      "No <h1> tag found on this page.",
      "Add exactly one H1 that clearly describes the page topic. It is a primary on-page SEO signal.");
  } else if (h1Count === 1) {
    const h1Text = (h1Matches[0][1] || "").trim().substring(0, 80);
    seo("Metadata","pass","H1 Heading",
      `One H1 found: "${h1Text}".`,
      "No action needed.");
  } else {
    seo("Metadata","warn","H1 Heading",
      `${h1Count} H1 tags found — pages should have exactly one primary heading.`,
      "Reduce to a single H1. Use H2–H6 for subheadings.");
  }

  // ── Social (Open Graph + Twitter Card) ────────────────────────────────────

  const ogTitle    = b("og:title");
  const ogDesc     = b("og:description");
  const ogImage    = b("og:image");
  const twitterCard= b("twitter:card") || b('name="twitter:');
  const hasOg      = ogTitle && ogDesc && ogImage;

  if (!ogTitle && !ogDesc && !ogImage) {
    seo("Social","fail","Open Graph Tags",
      "No Open Graph meta tags found (og:title, og:description, og:image).",
      "Add OG tags so shared links render rich previews on Facebook, LinkedIn, and Slack.");
  } else if (!hasOg) {
    const missing = [!ogTitle && "og:title", !ogDesc && "og:description", !ogImage && "og:image"].filter(Boolean);
    seo("Social","warn","Open Graph Tags",
      `Partial OG implementation — missing: ${missing.join(", ")}.`,
      "Complete the OG tag set for consistent rich previews across platforms.");
  } else {
    seo("Social","pass","Open Graph Tags",
      "og:title, og:description, and og:image are all present.",
      "No action needed.");
  }

  if (!twitterCard) {
    seo("Social","warn","Twitter / X Card",
      "No twitter:card meta tag found.",
      'Add <meta name="twitter:card" content="summary_large_image"> for rich previews on X/Twitter.');
  } else {
    seo("Social","pass","Twitter / X Card",
      "twitter:card meta tag present.",
      "No action needed.");
  }

  // ── Technical SEO ─────────────────────────────────────────────────────────

  // Viewport (mobile-friendliness)
  const hasViewport = b('name="viewport"') || b("name='viewport'");
  seo("Technical", hasViewport ? "pass" : "fail", "Viewport Meta Tag",
    hasViewport ? "Viewport meta tag present — page signals mobile-responsiveness." : "No viewport meta tag found.",
    hasViewport ? "No action needed." : 'Add <meta name="viewport" content="width=device-width, initial-scale=1"> for mobile-friendliness. Google uses mobile-first indexing.');

  // Structured data
  const hasJsonLd   = b("application/ld+json");
  const hasMicrodata= b("itemscope") || b("itemtype=");
  const hasSchema   = hasJsonLd || hasMicrodata;
  seo("Technical", hasSchema ? "pass" : "warn", "Structured Data",
    hasSchema
      ? (hasJsonLd ? "JSON-LD structured data detected." : "Microdata structured data detected.")
      : "No structured data (JSON-LD or Microdata) found.",
    hasSchema ? "No action needed. Validate at schema.org/validator." : "Add JSON-LD structured data (e.g. Organization, WebPage, BreadcrumbList) to improve rich results eligibility.");

  // Content compression
  const encoding   = h("content-encoding");
  const compressed = encoding.includes("gzip") || encoding.includes("br") || encoding.includes("zstd");
  seo("Technical", compressed ? "pass" : "warn", "Content Compression",
    compressed ? `Response is compressed (${encoding}).` : "No content-encoding header — response may be uncompressed.",
    compressed ? "No action needed." : "Enable gzip or Brotli compression on your server to reduce page load time — a Core Web Vitals ranking factor.");

  // HTTPS (already in security findings — cross-reference here)
  seo("Technical", isHttps ? "pass" : "fail", "HTTPS",
    isHttps ? "Site is served over HTTPS — required for modern SEO ranking." : "Site is served over HTTP.",
    isHttps ? "No action needed." : "Migrate to HTTPS. Google has used HTTPS as a ranking signal since 2014.");

  // Image alt text (sampling — check whether any img lacks alt entirely)
  const imgCount    = (bodyRaw.match(/<img/gi) || []).length;
  const imgWithAlt  = (bodyRaw.match(/<img[^>]+alt=["'][^"']{1,}/gi) || []).length;
  const imgNoAlt    = imgCount - imgWithAlt;
  if (imgCount === 0) {
    seo("Technical","pass","Image Alt Text",
      "No images detected in page source.",
      "No action needed.");
  } else if (imgNoAlt > 0) {
    seo("Technical","warn","Image Alt Text",
      `${imgNoAlt} of ${imgCount} image(s) appear to be missing or have empty alt attributes.`,
      "Add descriptive alt text to all images. Alt text is used by search engines to understand image content and is required for accessibility (WCAG 2.1).");
  } else {
    seo("Technical","pass","Image Alt Text",
      `All ${imgCount} detected image(s) have alt attributes.`,
      "No action needed.");
  }

  // Sort: Critical first, then Warning, then Info
  const sev = { "Critical": 0, "Warning": 1, "Info": 2 };
  findings.sort((a, b) => sev[a.severity] - sev[b.severity]);

  return {
    status:           "ok",
    address,
    identity_trits,
    cguid:            (scanTritsOut[0] - 1) * 3 + scanTritsOut[1],
    scan_hash,
    scan_hash_algo:   "tis-27",
    hptp_mandatory,
    crd,
    dimensions,
    scores,
    trackers,
    security_headers,
    findings,
    seo_signals:      seoSignals,
    cookie_audit:     cookieAudit,
    tech_fingerprint: techSignals,
    topology_svg:     null,         // Phase 2: services/tdns-v2/src/topology.rs
    meta,
    scannedAt:        new Date().toISOString(),
  };
}

// ── Route registration ────────────────────────────────────────────────────────
export function registerTdnsRoutes(app: Express) {

  app.get("/api/tdns/health", (_req: Request, res: Response) => {
    res.json({ status: "ok", version: "2.5.0", entities: registry.size, engine: "server-js-v141" });
  });

  app.post("/api/tdns/scan", async (req: Request, res: Response) => {
    const { url } = req.body;
    if (!url) { res.status(400).json({ error: "url required" }); return; }
    try {
      const result = await scanUrl(url);
      res.json(result);
    } catch (err: any) {
      log.error(`Scan error: ${err.message}`);
      res.status(500).json({ error: err.message });
    }
  });

  app.post("/api/tdns/register", async (req: Request, res: Response) => {
    const { name, zone, url, overwrite, org_name } = req.body;
    if (!name || !url) { res.status(400).json({ error: "name and url required" }); return; }

    const cleanName = name
      .toLowerCase().replace(/[^a-z0-9-]/g, "-").replace(/-{2,}/g, "-").replace(/^-|-$/g, "");
    if (!cleanName) { res.status(400).json({ error: "name contains no valid characters" }); return; }

    const plmName    = cleanName.endsWith(".plm") ? cleanName : cleanName + ".plm";
    const normUrl    = normaliseRegistryUrl(url);
    const canonical  = canonicaliseUrl(url);

    if (!overwrite) {
      const existingName = urlIndex.get(normUrl);
      if (existingName) {
        log.warn(`Register blocked: ${normUrl} already registered as ${existingName}`);
        res.status(409).json({
          status: "duplicate", error: `URL already registered as ${existingName}`,
          existing_name: existingName, url: normUrl,
          hint: "Pass overwrite=true to replace, or use POST /api/tdns/org/add-url to add to an org.",
        });
        return;
      }
    } else {
      const oldName = urlIndex.get(normUrl);
      if (oldName && oldName !== plmName) {
        registry.delete(oldName); urlIndex.delete(normUrl);
        log.info(`Overwrite: removed old record ${oldName} for ${normUrl}`);
      }
    }

    try {
      const scan  = await scanUrl(url);
      const entry: RegistryEntry = {
        ...scan, name: plmName, zone: zone || "public", url, canonical_url: canonical,
        org_name: org_name ? sanitiseOrgHandle(org_name) : undefined,
        registered_at: new Date().toISOString(),
      };
      registry.set(plmName, entry);
      urlIndex.set(normUrl, plmName);

      let org: OrgEntity | undefined;
      if (org_name) {
        const handle = sanitiseOrgHandle(org_name);
        org = attachToOrg(handle, entry, scan);
      }

      log.info(`Registered ${plmName} → ${scan.address} (url: ${normUrl})${org ? ` [org: ${org.org_name}]` : ""}`);
      res.json({
        status: "ok", name: plmName, address: scan.address,
        identity_trits: scan.identity_trits, cguid: scan.cguid,
        scan_hash: scan.scan_hash, hptp_mandatory: scan.hptp_mandatory, crd: scan.crd,
        org: org ? { org_name: org.org_name, member_count: org.members.length } : undefined,
      });
    } catch (err: any) {
      res.status(500).json({ error: err.message });
    }
  });

  app.post("/api/tdns/org/create", (req: Request, res: Response) => {
    const { org_name, display_name } = req.body;
    if (!org_name) { res.status(400).json({ error: "org_name required" }); return; }
    const handle = sanitiseOrgHandle(org_name);
    if (orgRegistry.has(handle)) {
      res.status(409).json({ status: "duplicate", error: `Org '${handle}' already exists` });
      return;
    }
    const now = new Date().toISOString();
    const org: OrgEntity = {
      org_name: handle, display_name: display_name || undefined,
      classification_address: "", members: [], created_at: now, updated_at: now,
    };
    orgRegistry.set(handle, org);
    log.info(`Org created: ${handle}`);
    res.json({ status: "ok", org_name: handle });
  });

  app.post("/api/tdns/org/add-url", (req: Request, res: Response) => {
    const { org_name, plm_name } = req.body;
    if (!org_name || !plm_name) { res.status(400).json({ error: "org_name and plm_name required" }); return; }
    const handle = sanitiseOrgHandle(org_name);
    const name   = plm_name.endsWith(".plm") ? plm_name : plm_name + ".plm";
    const entry  = registry.get(name);
    if (!entry) { res.status(404).json({ error: `${name} not found in registry` }); return; }
    if (orgIndex.has(name)) {
      const existingOrg = orgIndex.get(name);
      res.status(409).json({ error: `${name} already belongs to org '${existingOrg}'` });
      return;
    }
    let org = orgRegistry.get(handle);
    if (!org) {
      const now = new Date().toISOString();
      org = { org_name: handle, classification_address: entry.address, members: [], created_at: now, updated_at: now };
      orgRegistry.set(handle, org);
    }
    const member: OrgMember = {
      url: entry.url, canonical_url: entry.canonical_url, plm_name: name,
      address: entry.address, identity_trits: entry.identity_trits,
      cguid: entry.cguid, added_at: new Date().toISOString(),
    };
    org.members.push(member);
    org.updated_at = new Date().toISOString();
    if (!org.classification_address) org.classification_address = entry.address;
    orgIndex.set(name, handle);
    entry.org_name = handle;
    log.info(`Org ${handle}: added ${name} (${entry.canonical_url})`);
    res.json({ status: "ok", org_name: handle, member_count: org.members.length });
  });

  app.get("/api/tdns/org/:name", (req: Request, res: Response) => {
    const handle = sanitiseOrgHandle(String(req.params.name));
    const org    = orgRegistry.get(handle);
    if (!org) { res.status(404).json({ status: "not_found", org_name: handle }); return; }
    res.json({
      status: "ok",
      org_name:               org.org_name,
      display_name:           org.display_name,
      classification_address: org.classification_address,
      member_count:           org.members.length,
      members:                org.members.map(m => ({
        plm_name:        m.plm_name,
        url:             m.url,
        canonical_url:   m.canonical_url,
        address:         m.address,
        cguid:           m.cguid,
        added_at:        m.added_at,
      })),
      created_at:  org.created_at,
      updated_at:  org.updated_at,
    });
  });

  app.get("/api/tdns/resolve/:name", (req: Request, res: Response) => {
    const raw   = String(req.params.name);
    const name  = raw.endsWith(".plm") ? raw : raw + ".plm";
    const entry = registry.get(name);
    if (!entry) { res.status(404).json({ status: "not_found", name }); return; }
    const orgHandle = entry.org_name;
    const org       = orgHandle ? orgRegistry.get(orgHandle) : undefined;
    res.json({
      status:          "ok",
      name:            entry.name,
      address:         entry.address,
      identity_trits:  entry.identity_trits,
      cguid:           entry.cguid,
      canonical_url:   entry.canonical_url,
      scan_hash:       entry.scan_hash,
      hptp_mandatory:  entry.hptp_mandatory,
      crd:             entry.crd,
      registered_at:   entry.registered_at,
      org: org ? {
        org_name:     org.org_name,
        display_name: org.display_name,
        member_count: org.members.length,
        members:      org.members.map(m => ({ plm_name: m.plm_name, url: m.url, address: m.address })),
      } : undefined,
    });
  });

  app.get("/api/tdns/list", (_req: Request, res: Response) => {
    res.json({
      status: "ok", count: registry.size, unique_urls: urlIndex.size,
      org_count: orgRegistry.size,
      entries: Array.from(registry.values()).map(e => ({
        name: e.name, address: e.address, url: e.url,
        org_name: e.org_name, registered_at: e.registered_at,
      })),
    });
  });

  app.get("/api/tdns/orgs", (_req: Request, res: Response) => {
    res.json({
      status: "ok", count: orgRegistry.size,
      orgs: Array.from(orgRegistry.values()).map(o => ({
        org_name: o.org_name, display_name: o.display_name,
        member_count: o.members.length,
        members: o.members.map(m => m.plm_name),
        created_at: o.created_at,
      })),
    });
  });

  log.info("TDNS routes registered v2.5.0 — multi-URL org entities active");
}

// ── Org helpers ───────────────────────────────────────────────────────────────

function sanitiseOrgHandle(raw: string): string {
  return raw.toLowerCase().replace(/[^a-z0-9-]/g, "-").replace(/-{2,}/g, "-").replace(/^-|-$/g, "");
}

function attachToOrg(handle: string, entry: RegistryEntry, scan: ScanResult): OrgEntity {
  const now = new Date().toISOString();
  let org   = orgRegistry.get(handle);
  if (!org) {
    org = {
      org_name:               handle,
      classification_address: scan.address,
      members:                [],
      created_at:             now,
      updated_at:             now,
    };
    orgRegistry.set(handle, org);
  }
  const member: OrgMember = {
    url:            entry.url,
    canonical_url:  entry.canonical_url,
    plm_name:       entry.name,
    address:        entry.address,
    identity_trits: entry.identity_trits,
    cguid:          entry.cguid,
    added_at:       now,
  };
  org.members.push(member);
  org.updated_at = now;
  orgIndex.set(entry.name, handle);
  return org;
}