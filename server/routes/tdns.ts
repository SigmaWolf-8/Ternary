// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
// TDNS Scanner & Registry — server-side implementation
// The authoritative scan engine runs here. No Docker, no Rust dependency.
// Extension → plenumnet.replit.app/api/tdns/scan. That's it.

import type { Express, Request, Response } from "express";
import { createLogger } from "../logger";
import crypto from "crypto";

const log = createLogger("tdns");
const TDNS_VERSION = "2.3.3";

// ── In-memory registry (will move to Postgres when scale requires it) ────────
const registry: Map<string, {
  name: string;
  address: string;
  zone: string;
  url: string;
  scan_hash: string;
  registered_at: string;
  crd: number;
}> = new Map();

// ── GF(3) derivation — exact formula from services/tdns-v2/src/derive.rs ─────
// trit = min(floor(3k/N), 2) + 1 → Rep C {1,2,3}, zero excluded
function gf3(k: number, n: number): number {
  return Math.min(Math.floor((3 * k) / n), 2) + 1;
}

function conf(k: number, n: number): number {
  const p = k / n;
  const d = Math.min(Math.abs(p - 1 / 3), Math.abs(p - 2 / 3));
  return Math.min(Math.floor(27 * d) + 1, 9);
}

// ── Schema: 27 ontological dimensions ────────────────────────────────────────
const SCHEMA = [
  { cat: "WHO",   q: "What kind of entity?",      vals: ["Personal", "Corporate", "Governance"] },
  { cat: "WHO",   q: "Who's the audience?",        vals: ["Just me", "My group", "Everyone"] },
  { cat: "WHO",   q: "Who operates it?",           vals: ["Anonymous", "Known", "Transparent"] },
  { cat: "WHO",   q: "Hosting model?",             vals: ["Self-hosted", "Provider", "Cloud"] },
  { cat: "WHAT",  q: "What form factor?",          vals: ["Website", "App / API", "Device"] },
  { cat: "WHAT",  q: "Content type?",              vals: ["Text / HTML", "Media", "Live stream"] },
  { cat: "WHAT",  q: "Primary consumer?",          vals: ["Humans", "Machines", "Both"] },
  { cat: "WHAT",  q: "AI / ML present?",           vals: ["No", "Partially", "Yes"] },
  { cat: "WHERE", q: "Visibility?",                vals: ["Private", "Group", "Public"] },
  { cat: "WHERE", q: "Authentication required?",   vals: ["None", "Password", "Strong ID"] },
  { cat: "WHERE", q: "Infrastructure scale?",      vals: ["Single server", "Several", "CDN / Many"] },
  { cat: "WHERE", q: "Transport protocol?",        vals: ["HTTP", "WebSocket", "Raw TCP"] },
  { cat: "WHEN",  q: "Technology era?",            vals: ["Pre-2010", "2010s", "2020s+"] },
  { cat: "WHEN",  q: "Availability window?",       vals: ["Business hours", "Extended", "24/7"] },
  { cat: "WHEN",  q: "Data freshness?",            vals: ["Historical", "Current", "Live"] },
  { cat: "WHEN",  q: "Real-time capability?",      vals: ["Batch", "Near-real-time", "Real-time"] },
  { cat: "WHY",   q: "Financial transactions?",    vals: ["None", "Accepts payment", "Processes"] },
  { cat: "WHY",   q: "Data collection appetite?",  vals: ["Minimal", "Moderate", "Heavy"] },
  { cat: "WHY",   q: "Legal / policy presence?",   vals: ["None", "Basic", "Comprehensive"] },
  { cat: "WHY",   q: "Revenue model?",             vals: ["Free", "Pay-per-use", "Subscription"] },
  { cat: "HOW",   q: "Delivery topology?",         vals: ["Unicast", "Multicast", "Anycast"] },
  { cat: "HOW",   q: "Data flow direction?",       vals: ["Outbound", "Relay / Proxy", "Inbound"] },
  { cat: "HOW",   q: "Update mechanism?",          vals: ["Pull / Poll", "Subscribe", "Push"] },
  { cat: "HOW",   q: "Session persistence?",       vals: ["Stateless", "Short session", "Long-lived"] },
  { cat: "PEACE", q: "Encryption posture?",        vals: ["Weak / None", "Basic TLS", "Hardened TLS"] },
  { cat: "PEACE", q: "Tracker density?",           vals: ["Heavy", "Moderate", "Clean"] },
  { cat: "PEACE", q: "Security audit status?",     vals: ["None", "Self-assessed", "Third-party"] },
];

// ── 27-dimension scanner ─────────────────────────────────────────────────────
async function scanUrl(targetUrl: string) {
  const parsed = new URL(targetUrl);
  const hostname = parsed.hostname.toLowerCase();
  const isHttps = parsed.protocol === "https:";

  let hdrs: Record<string, string> = {};
  let status = 0;
  let body = "";
  let ok = false;

  try {
    const resp = await fetch(targetUrl, {
      method: "GET",
      redirect: "follow",
      signal: AbortSignal.timeout(9000),
      headers: { "User-Agent": "PlenumNET-TDNS-Scanner/2.3.3" },
    });
    ok = resp.ok;
    status = resp.status;
    resp.headers.forEach((v, k) => { hdrs[k.toLowerCase()] = v.toLowerCase(); });
    const buf = await resp.arrayBuffer();
    body = new TextDecoder().decode(new Uint8Array(buf).slice(0, 12288)).toLowerCase();
  } catch (e: any) {
    log.error("Scan fetch error for %s: %s", targetUrl, e.message);
  }

  const h = (k: string) => hdrs[k] || "";
  const has = (k: string) => !!hdrs[k];
  const b = (p: string) => body.includes(p);

  const meta = {
    url: targetUrl,
    hostname,
    isHttps,
    statusCode: status,
    server: h("server") || h("x-powered-by") || "—",
    ct: h("content-type"),
    cacheCtrl: h("cache-control"),
    hsts: h("strict-transport-security"),
    csp: h("content-security-policy"),
    cors: h("access-control-allow-origin"),
    xfo: h("x-frame-options"),
    xcto: h("x-content-type-options"),
    via: h("via"),
    cfRay: has("cf-ray"),
    xCache: has("x-cache") || has("x-amz-cf-id") || has("x-served-by"),
    altSvc: h("alt-svc"),
    permsPolicy: h("permissions-policy"),
    nel: has("nel"),
    reportTo: has("report-to") || has("reporting-endpoints"),
    coep: has("cross-origin-embedder-policy"),
    coop: has("cross-origin-opener-policy"),
    hpkp: has("public-key-pins"),
    setCookie: h("set-cookie"),
    bodySize: body.length,
    hasHtml: b("<!doctype") || h("content-type").includes("html"),
    hasJson: h("content-type").includes("json"),
    isApi: parsed.pathname.includes("/api/"),
    isWs: h("upgrade") === "websocket",
    isSse: h("content-type").includes("event-stream"),
  };

  const isGov = /\.(gov|mil|gc\.ca|gov\.uk)$/.test(hostname);
  const isEdu = /\.(edu|ac\.uk|edu\.au)$/.test(hostname);
  const isCloud = ["amazonaws", "cloudflare", "vercel", "netlify", "replit",
    "fastly", "azurewebsites", "googleusercontent", "github.io",
    "pages.dev", "fly.dev", "render.com"].some(s => hostname.includes(s));
  const isLargeCorp = ["google.com", "youtube.com", "facebook.com", "x.com",
    "twitter.com", "linkedin.com", "microsoft.com", "apple.com",
    "amazon.com", "netflix.com", "github.com"].some(s => hostname === s || hostname.endsWith("." + s));
  const maxAgeMatch = meta.cacheCtrl.match(/max-age=(\d+)/);
  const maxAge = maxAgeMatch ? parseInt(maxAgeMatch[1]) : 3600;
  const cookieVal = meta.setCookie;

  // ── 27 derivations ──────────────────────────────────────────────────────────
  const d1 = isGov || isEdu ? 3 : isLargeCorp ? 2 : 2;
  const d2 = 3;
  const k3 = +(b("/about") || b("about us") || b("about-us"))
    + +(b("contact") || b("contact us") || b("get in touch"))
    + +(b("inc.") || b(" ltd") || b("llc") || b("corp.") || b("co.,"))
    + +(b("address:") || b("street") || b("avenue") || b("suite") || b("postal"))
    + +(isGov || isLargeCorp ? 1 : 0);
  const d3 = gf3(k3, 5), c3 = conf(k3, 5);
  const d4 = isCloud ? 3 : 2;
  const d5 = meta.isApi && !meta.hasHtml ? 2 : 1;
  const d6 = meta.isWs || meta.isSse ? 3 : meta.ct.includes("video") || meta.ct.includes("audio") || meta.ct.includes("image") ? 2 : 1;
  const d7 = meta.hasHtml && (meta.hasJson || meta.isApi) ? 3 : meta.hasJson || meta.isApi ? 2 : 1;
  const k8 = +(b("/predict") || b("/inference") || b("/model") || b("/embed") || b("/generate"))
    + +(b("tensorflow") || b("pytorch") || b("hugging face") || b("openai") || b("llm"))
    + +(b("recommendation") || b("personali") || b("suggested for you"))
    + +(b("ranking") || b("relevance score") || b("similarity"))
    + +(has("x-ai") || has("x-model") || has("x-inference") ? 1 : 0);
  const d8 = gf3(k8, 5), c8 = conf(k8, 5);
  const k9 = +(ok || status > 0 ? 1 : 0) + +(status !== 401 && status !== 403 && !has("www-authenticate") ? 1 : 0) + +(ok && (meta.hasHtml || meta.hasJson) ? 1 : 0);
  const d9 = gf3(k9, 3), c9 = conf(k9, 3);
  const d10 = has("www-authenticate") ? 2 : b("sign in") || b("log in") || b("login") ? 2 : 1;
  const parts = hostname.split(".");
  const k11 = +(ok || status > 0 ? 1 : 0) + +(parts.length >= 3 ? 1 : 0) + +(parts.length >= 4 ? 1 : 0) + +(meta.cfRay || has("x-cdn") ? 1 : 0) + +(meta.xCache || has("age") ? 1 : 0) + +(meta.via !== "" ? 1 : 0);
  const d11 = gf3(k11, 6), c11 = conf(k11, 6);
  const d12 = meta.isWs ? 2 : 1;
  const k13 = +(has("alt-svc") ? 1 : 0) + +(has("permissions-policy") ? 1 : 0) + +(has("nel") ? 1 : 0)
    + +(has("report-to") || has("reporting-endpoints") ? 1 : 0)
    + +(has("cross-origin-opener-policy") || has("cross-origin-embedder-policy") ? 1 : 0)
    + +(has("content-security-policy") ? 1 : 0);
  const d13 = gf3(k13, 6), c13 = conf(k13, 6);
  const k14 = +(!b("maintenance") && !b("down for") ? 1 : 0) + +(!b("business hours") && !b("office hours") ? 1 : 0) + +(b("99.") || b("uptime") || ok ? 1 : 0);
  const d14 = gf3(k14, 3), c14 = conf(k14, 3);
  const d15 = meta.isWs || meta.isSse ? 3 : meta.cacheCtrl.includes("no-store") || meta.cacheCtrl.includes("no-cache") ? 2 : 2;
  const k16 = +(!meta.cacheCtrl.includes("immutable") && maxAge < 3600 ? 1 : 0) + +(maxAge < 60 ? 1 : 0) + +(meta.isWs ? 1 : 0) + +(meta.isSse ? 1 : 0) + +(meta.ct.includes("grpc") ? 1 : 0);
  const d16 = gf3(k16, 6), c16 = conf(k16, 6);
  const d17 = b("swift") || b("wire transfer") || b("ach transfer") ? 3 : b("stripe") || b("paypal") || b("checkout") || b("braintree") || b("buy now") ? 2 : 1;
  const k18 = +(b("<input") || b("input type=") ? 1 : 0) + +(b("sign up") || b("register") || b("create account") ? 1 : 0) + +(b("gtag(") || b("analytics.js") || b("fbq(") || b("_paq.push") ? 1 : 0) + +(b("cookie consent") || b("gdpr") || b("we use cookies") ? 1 : 0) + +(b("third party") || b("data partner") ? 1 : 0);
  const d18 = gf3(k18, 5), c18 = conf(k18, 5);
  const k19 = +(b("privacy policy") || b("/privacy") ? 1 : 0) + +(b("terms of service") || b("terms of use") || b("/terms") ? 1 : 0) + +(b("cookie policy") || b("cookie notice") ? 1 : 0) + +(b("gdpr") || b("data protection") ? 1 : 0) + +(b("accessibility") || b("wcag") ? 1 : 0);
  const d19 = gf3(k19, 5), c19 = conf(k19, 5);
  const d20 = b("subscribe") || b("per month") || b("/month") || b("annual plan") ? 3 : b("pay per") || b("per use") || b("credits") ? 2 : 1;
  const d21 = meta.cfRay || meta.xCache ? 3 : meta.via !== "" ? 2 : 1;
  const d22 = meta.hasJson && (b('"data"') || b('"result"') || b('"id"')) ? 3 : meta.via !== "" || meta.cfRay ? 2 : 1;
  const d23 = meta.isWs || meta.isSse ? 3 : b("rss") || b("atom") || b("</feed") ? 2 : 1;
  const hasCookie = cookieVal !== "";
  const hasExp = cookieVal.includes("expires=") || cookieVal.includes("max-age=");
  const longLivedMatch = cookieVal.match(/max-age=(\d+)/i);
  const longLived = (longLivedMatch ? parseInt(longLivedMatch[1]) > 86400 * 30 : false) || (cookieVal.includes("expires=") && !cookieVal.includes("session"));
  const k24 = +(hasCookie ? 1 : 0) + +(hasExp ? 1 : 0) + +(longLived ? 1 : 0);
  const d24 = gf3(k24, 3), c24 = conf(k24, 3);
  const k25 = +(isHttps ? 1 : 0) + +(has("strict-transport-security") ? 1 : 0) + +(has("content-security-policy") ? 1 : 0) + +(b("security.txt") || b("/.well-known/security") ? 1 : 0) + +(has("x-content-type-options") ? 1 : 0) + +(has("x-frame-options") ? 1 : 0);
  const d25 = gf3(k25, 6), c25 = conf(k25, 6);
  const k26 = +(!b("google-analytics") && !b("gtag(") && !b("mixpanel") && !b("amplitude") ? 1 : 0)
    + +(!b("facebook.net") && !b("platform.twitter") && !b("snap.licdn") ? 1 : 0)
    + +(!b("doubleclick") && !b("googlesyndication") && !b("adsystem") ? 1 : 0)
    + +(!b("fullstory") && !b("hotjar") && !b("logrocket") && !b("clarity.ms") ? 1 : 0)
    + +(!b("hubspot") && !b("marketo") && !b("intercom") && !b("pardot") ? 1 : 0);
  const d26 = gf3(k26, 5), c26 = conf(k26, 5);
  const d27 = b("iso 27001") || b("soc 2") || b("pci dss") || b("soc2") ? 3 : b("penetration test") || b("bug bounty") || b("self-certified") ? 2 : 1;

  const trits = [d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15, d16, d17, d18, d19, d20, d21, d22, d23, d24, d25, d26, d27];
  const confs = [9, 9, c3, 9, 9, 9, 9, c8, c9, 9, c11, 9, c13, c14, 9, c16, 9, c18, c19, 9, 9, 9, 9, c24, c25, c26, 9];

  const address = [
    `WO:${trits.slice(0, 4).join("")}`,
    `WA:${trits.slice(4, 8).join("")}`,
    `WR:${trits.slice(8, 12).join("")}`,
    `WN:${trits.slice(12, 16).join("")}`,
    `WY:${trits.slice(16, 20).join("")}`,
    `HO:${trits.slice(20, 24).join("")}`,
    `PE:${trits.slice(24, 27).join("")}`,
  ].join(" ");

  const scan_hash = crypto
    .createHash("sha256")
    .update(Buffer.from(trits))
    .digest("hex");

  const dimensions = SCHEMA.map((s, i) => ({
    number: i + 1,
    category: s.cat,
    question: s.q,
    value: trits[i],
    label: s.vals[trits[i] - 1],
    confidence: confs[i],
  }));

  const hptp_mandatory = trits[14] === 3 && trits[15] === 3;
  const securityScore = Math.round(((trits[24] + trits[25] + trits[26]) / 9) * 100);
  const privacyScore = Math.round(((trits[25] + trits[18]) / 6) * 100);

  return {
    address,
    hptp_mandatory,
    scan_hash,
    dimensions,
    meta,
    securityScore,
    privacyScore,
    scannedAt: new Date().toISOString(),
  };
}

export function registerTdnsRoutes(app: Express) {
  // ── Health ──────────────────────────────────────────────────────────────────
  app.get("/api/tdns/health", (_req: Request, res: Response) => {
    res.json({
      status: "ok",
      version: TDNS_VERSION,
      entities: registry.size,
      engine: "express",
    });
  });

  // ── Scan ────────────────────────────────────────────────────────────────────
  app.post("/api/tdns/scan", async (req: Request, res: Response) => {
    const { url } = req.body;
    if (!url || typeof url !== "string") {
      res.status(400).json({ error: "Missing or invalid 'url' field" });
      return;
    }
    try {
      new URL(url);
    } catch {
      res.status(400).json({ error: "Invalid URL format" });
      return;
    }
    try {
      const result = await scanUrl(url);
      log.info("Scanned %s → %s", url, result.address);
      res.json(result);
    } catch (err: any) {
      log.error("Scan failed for %s: %s", url, err.message);
      res.status(500).json({ error: "Scan failed", details: err.message });
    }
  });

  // ── Register ────────────────────────────────────────────────────────────────
  app.post("/api/tdns/register", async (req: Request, res: Response) => {
    const { name, zone, url } = req.body;
    if (!name || !url) {
      res.status(400).json({ error: "Missing 'name' or 'url'" });
      return;
    }
    const plmName = name.endsWith(".plm") ? name : name + ".plm";
    try {
      const scanResult = await scanUrl(url);
      const entry = {
        name: plmName,
        address: scanResult.address,
        zone: zone || "public",
        url,
        scan_hash: scanResult.scan_hash,
        registered_at: new Date().toISOString(),
        crd: Math.floor(Math.random() * 900) + 100,
      };
      registry.set(plmName, entry);
      log.info("Registered %s → %s", plmName, scanResult.address);
      res.json({ ...entry, scan: scanResult });
    } catch (err: any) {
      log.error("Register failed for %s: %s", plmName, err.message);
      res.status(500).json({ error: "Registration failed", details: err.message });
    }
  });

  // ── Resolve ─────────────────────────────────────────────────────────────────
  app.get("/api/tdns/resolve/:name", (req: Request, res: Response) => {
    const name = req.params.name.endsWith(".plm") ? req.params.name : req.params.name + ".plm";
    const entry = registry.get(name);
    if (!entry) {
      res.status(404).json({ error: "Name not found", name });
      return;
    }
    res.json(entry);
  });

  // ── List ────────────────────────────────────────────────────────────────────
  app.get("/api/tdns/entities", (_req: Request, res: Response) => {
    const entries = Array.from(registry.values());
    res.json({ count: entries.length, entities: entries });
  });

  // ── Status ──────────────────────────────────────────────────────────────────
  app.get("/api/tdns-status", (_req: Request, res: Response) => {
    res.json({
      status: "available",
      version: TDNS_VERSION,
      entities: registry.size,
      engine: "express",
    });
  });
}
