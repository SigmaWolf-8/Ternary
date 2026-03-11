#!/usr/bin/env node
// ═══════════════════════════════════════════════════════════════════════════════
// PlenumNET TDNS v2.5.0 — TIS-27 End-to-End Functional Test Suite
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Hits live API endpoints. No mocks. No simulated answers.
// Every assertion is derived from the actual server response.
//
// Usage:
//   node tis27-e2e-tests.js
//   BASE_URL=https://plenumnet.replit.app node tis27-e2e-tests.js
//
// Requirements: Node.js 18+ (built-in fetch)
// ═══════════════════════════════════════════════════════════════════════════════

const BASE = process.env.BASE_URL?.replace(/\/$/, "") ?? "https://plenumnet.replit.app";

// ─── Test harness ─────────────────────────────────────────────────────────────

let passed = 0, failed = 0, total = 0;
const failures = [];

function assert(label, condition, detail = "") {
  total++;
  if (condition) {
    passed++;
    console.log(`  ✓  ${label}`);
  } else {
    failed++;
    failures.push({ label, detail });
    console.log(`  ✗  ${label}${detail ? `\n       → ${detail}` : ""}`);
  }
}

function section(title) {
  console.log(`\n${"═".repeat(70)}`);
  console.log(`  ${title}`);
  console.log("═".repeat(70));
}

async function get(path) {
  const r = await fetch(`${BASE}${path}`, { headers: { "Accept": "application/json" } });
  const body = await r.json().catch(() => ({}));
  return { status: r.status, body };
}

async function post(path, payload) {
  const r = await fetch(`${BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "Accept": "application/json" },
    body: JSON.stringify(payload),
  });
  const body = await r.json().catch(() => ({}));
  return { status: r.status, body };
}

// ─── Validators ───────────────────────────────────────────────────────────────

function isRepC(arr) {
  // Every trit must be in {1, 2, 3} — zero is the forgery sentinel
  return Array.isArray(arr) && arr.length === 27 && arr.every(t => t === 1 || t === 2 || t === 3);
}

function noZeroTrits(arr) {
  return Array.isArray(arr) && arr.every(t => t !== 0);
}

function isValidAddress(addr) {
  // Format: "WO:XXXX WA:XXXX ... PE:XXX · ID:XXXXXXXXXXXXXXXXXXXXXXXXXXX"
  // Classification: 7 groups with labels, Identity: 27 trits after "· ID:"
  if (typeof addr !== "string") return false;
  const parts = addr.split(" · ID:");
  if (parts.length !== 2) return false;
  const id = parts[1];
  return id.length === 27 && /^[123]+$/.test(id);
}

function isValidScanHash(hash) {
  // TIS-27 scan hash stored as hex — 32 bytes = 64 hex chars
  return typeof hash === "string" && hash.length === 64 && /^[0-9a-f]+$/i.test(hash);
}

function allTritsDistinct(scans) {
  // Different URLs must produce different identity anchors
  const seen = new Set();
  for (const s of scans) {
    const key = (s.identity_trits || []).join(",");
    if (seen.has(key)) return false;
    seen.add(key);
  }
  return true;
}

function crdValid(crd, identity_trits) {
  // CRD = (T1−1)×3 + T2 from scan sponge output trits 0–1
  // We can't recompute the scan sponge here, but CRD must be 1–9
  return typeof crd === "number" && crd >= 1 && crd <= 9;
}

// ─── Test targets ─────────────────────────────────────────────────────────────

const TEST_URLS = [
  "https://google.com",
  "https://plenumnet.replit.app",
  "https://github.com",
  "https://cloudflare.com",
  "https://stripe.com",
];

// ─── Run tests ────────────────────────────────────────────────────────────────

console.log(`\nPlenumNET TDNS v2.5.0 — TIS-27 E2E Test Suite`);
console.log(`Target: ${BASE}`);
console.log(`Time:   ${new Date().toISOString()}`);

// ── 1. Health ─────────────────────────────────────────────────────────────────
section("1. Health Check");
{
  const { status, body } = await get("/api/tdns/health");
  assert("HTTP 200", status === 200, `got ${status}`);
  assert("status: ok", body.status === "ok", `got ${JSON.stringify(body.status)}`);
  assert("version: 2.5.0", body.version === "2.5.0", `got ${body.version} — server must be updated to v2.5.0`);
  assert("entities count present", typeof body.entities === "number", `got ${body.entities}`);
  console.log(`     Server version: ${body.version} | Registered entities: ${body.entities}`);
}

// ── 2. Scan — algorithm field ─────────────────────────────────────────────────
section("2. Scan — Algorithm Must Be TIS-27");
const scanResults = [];
for (const url of TEST_URLS) {
  console.log(`\n  Scanning: ${url}`);
  const { status, body } = await post("/api/tdns/scan", { url });
  assert(`HTTP 200 (${url})`, status === 200, `got ${status}`);
  if (status === 200) {
    assert(
      `scan_hash_algo = "tis-27"`,
      body.scan_hash_algo === "tis-27",
      `got "${body.scan_hash_algo}" — BLAKE3/SHA-256 means server not updated`
    );
    scanResults.push({ url, ...body });
  }
}

// ── 3. Scan — identity anchor shape ──────────────────────────────────────────
section("3. Scan — Identity Anchor (27-Trit Rep C)");
for (const r of scanResults) {
  console.log(`\n  ${r.url}`);
  assert("identity_trits is array[27]",  Array.isArray(r.identity_trits) && r.identity_trits.length === 27, `length=${r.identity_trits?.length}`);
  assert("all trits in Rep C {1,2,3}",   isRepC(r.identity_trits), `trits=${JSON.stringify(r.identity_trits)}`);
  assert("no zero trits (forgery check)", noZeroTrits(r.identity_trits), `trits=${JSON.stringify(r.identity_trits)}`);
  console.log(`     ID: ${(r.identity_trits || []).join("")}`);
}

// ── 4. Scan — address format ──────────────────────────────────────────────────
section("4. Scan — Full 54-Trit Address Format");
for (const r of scanResults) {
  console.log(`\n  ${r.url}`);
  assert("address contains · ID: separator", typeof r.address === "string" && r.address.includes("· ID:"), `got "${r.address?.substring(0, 60)}..."`);
  assert("identity anchor in address is 27 trits [123]+", isValidAddress(r.address), `addr="${r.address}"`);
  const idPart = r.address?.split("· ID:")[1] ?? "";
  assert("address ID matches identity_trits",
    idPart === (r.identity_trits || []).join(""),
    `address ID="${idPart}" vs trits="${(r.identity_trits||[]).join("")}"`
  );
  console.log(`     ${r.address}`);
}

// ── 5. Scan — scan hash ───────────────────────────────────────────────────────
section("5. Scan — Scan Hash (TIS-27, 32 bytes hex)");
for (const r of scanResults) {
  console.log(`\n  ${r.url}`);
  assert("scan_hash is 64-char hex string", isValidScanHash(r.scan_hash), `got "${r.scan_hash}"`);
  assert("scan_hash non-zero", r.scan_hash !== "0".repeat(64), "all-zero hash would indicate failed derivation");
  console.log(`     ${r.scan_hash}`);
}

// ── 6. Scan — CRD and CGUID ───────────────────────────────────────────────────
section("6. Scan — CRD (1–9) and CGUID (1–9)");
for (const r of scanResults) {
  console.log(`\n  ${r.url}`);
  assert("crd in range 1–9",   crdValid(r.crd, r.identity_trits), `got ${r.crd}`);
  assert("cguid in range 1–9", typeof r.cguid === "number" && r.cguid >= 1 && r.cguid <= 9, `got ${r.cguid}`);
  console.log(`     CRD=${r.crd}  CGUID=${r.cguid}`);
}

// ── 7. Determinism — same URL twice must produce identical output ──────────────
section("7. Determinism — Same URL → Identical Output (Both Calls)");
{
  const url = "https://google.com";
  console.log(`\n  Scanning ${url} twice...`);
  const { body: a } = await post("/api/tdns/scan", { url });
  const { body: b } = await post("/api/tdns/scan", { url });

  assert("identity_trits identical across calls",
    JSON.stringify(a.identity_trits) === JSON.stringify(b.identity_trits),
    `call1=${JSON.stringify(a.identity_trits)}\ncall2=${JSON.stringify(b.identity_trits)}`
  );
  assert("scan_hash identical (same content → same TIS-27 output)",
    a.scan_hash === b.scan_hash,
    `call1=${a.scan_hash}\ncall2=${b.scan_hash}`
  );
  assert("address identical",
    a.address === b.address,
    `call1=${a.address}\ncall2=${b.address}`
  );
  console.log(`     ID trits: ${(a.identity_trits||[]).join("")}`);
}

// ── 8. Uniqueness — different URLs must produce different identity anchors ─────
section("8. Uniqueness — Different URLs → Different Identity Anchors");
{
  assert(
    `all ${scanResults.length} identity anchors are distinct`,
    allTritsDistinct(scanResults),
    "collision: two URLs produced the same 27-trit identity anchor"
  );

  // Also check pairwise that http vs https differ
  console.log(`\n  Checking http:// vs https:// scheme separation...`);
  const { body: https_r } = await post("/api/tdns/scan", { url: "https://example.com" });
  const { body: http_r  } = await post("/api/tdns/scan", { url: "http://example.com" });
  assert(
    "http://example.com ≠ https://example.com identity",
    JSON.stringify(https_r.identity_trits) !== JSON.stringify(http_r.identity_trits),
    "scheme must be included in identity derivation"
  );
  console.log(`     https: ${(https_r.identity_trits||[]).join("")}`);
  console.log(`     http:  ${(http_r.identity_trits||[]).join("")}`);
}

// ── 9. Canonical URL collapse ─────────────────────────────────────────────────
section("9. Canonical URL Collapse — Variants Must Produce Same Identity");
{
  const variants = [
    "https://google.com",
    "https://google.com/",
    "HTTPS://GOOGLE.COM",
    "https://google.com:443",
    "https://google.com/#fragment",
    "https://google.com/?query=ignored",
  ];
  const base = scanResults.find(r => r.url === "https://google.com");
  if (base) {
    for (const v of variants) {
      if (v === "https://google.com") continue; // already have it
      console.log(`\n  Variant: ${v}`);
      const { body } = await post("/api/tdns/scan", { url: v });
      assert(
        `identity matches canonical baseline`,
        JSON.stringify(body.identity_trits) === JSON.stringify(base.identity_trits),
        `variant="${v}"\nvariant_id=${JSON.stringify(body.identity_trits)}\nbaseline=${JSON.stringify(base.identity_trits)}`
      );
    }
  }
}

// ── 10. Register + Resolve round-trip ─────────────────────────────────────────
section("10. Register → Resolve Round-Trip");
{
  const testName = `tis27-test-${Date.now()}`;
  const testUrl  = "https://stripe.com";
  console.log(`\n  Registering ${testUrl} as ${testName}.plm ...`);

  const { status: regStatus, body: regBody } = await post("/api/tdns/register", {
    name: testName,
    url:  testUrl,
    overwrite: true,
  });
  assert("register returns 200 or 409",
    regStatus === 200 || regStatus === 409,
    `got ${regStatus}: ${JSON.stringify(regBody)}`
  );

  if (regStatus === 200 || regStatus === 409) {
    console.log(`\n  Resolving ${testName}.plm ...`);
    const { status: resStatus, body: resBody } = await get(`/api/tdns/resolve/${testName}`);
    assert("resolve returns 200", resStatus === 200, `got ${resStatus}`);
    assert("resolved entry has identity_trits", Array.isArray(resBody.identity_trits) && resBody.identity_trits.length === 27);
    assert("resolved identity_trits in Rep C", isRepC(resBody.identity_trits), `trits=${JSON.stringify(resBody.identity_trits)}`);
    assert("resolved address has · ID: marker", typeof resBody.address === "string" && resBody.address.includes("· ID:"));

    // Verify the resolved identity matches what a fresh scan would produce
    console.log(`\n  Cross-checking resolved identity vs fresh scan of same URL...`);
    const { body: freshScan } = await post("/api/tdns/scan", { url: testUrl });
    assert(
      "resolved identity_trits matches fresh scan",
      JSON.stringify(resBody.identity_trits) === JSON.stringify(freshScan.identity_trits),
      `resolved: ${JSON.stringify(resBody.identity_trits)}\nfresh:    ${JSON.stringify(freshScan.identity_trits)}`
    );
    console.log(`     Identity: ${(resBody.identity_trits||[]).join("")}`);
    console.log(`     Address:  ${resBody.address}`);
  }
}

// ── 11. List — no legacy algo values in registry ──────────────────────────────
section("11. Registry Scan — No Legacy BLAKE3/SHA-256 Entries");
{
  const { status, body } = await get("/api/tdns/list");
  assert("list returns 200", status === 200, `got ${status}`);
  if (status === 200 && Array.isArray(body.entries)) {
    console.log(`\n  Registry has ${body.entries.length} entries`);
    // Re-scan a sample to check algo field (list doesn't return scan_hash_algo)
    const sample = body.entries.slice(0, 3);
    for (const entry of sample) {
      if (!entry.url) continue;
      const { body: scanBody } = await post("/api/tdns/scan", { url: entry.url });
      assert(
        `entry "${entry.name}" scan_hash_algo = tis-27`,
        scanBody.scan_hash_algo === "tis-27",
        `got "${scanBody.scan_hash_algo}" for ${entry.url}`
      );
    }
  }
}

// ── 12. Error handling ────────────────────────────────────────────────────────
section("12. Error Handling");
{
  const { status: s1 } = await post("/api/tdns/scan", {});
  assert("scan with no URL → 400", s1 === 400, `got ${s1}`);

  const { status: s2 } = await get("/api/tdns/resolve/definitely-does-not-exist-zzz9999");
  assert("resolve unknown name → 404", s2 === 404, `got ${s2}`);
}

// ─── Summary ──────────────────────────────────────────────────────────────────

console.log(`\n${"═".repeat(70)}`);
console.log(`  RESULTS`);
console.log("═".repeat(70));
console.log(`  Total:  ${total}`);
console.log(`  Passed: ${passed} ✓`);
console.log(`  Failed: ${failed} ${failed > 0 ? "✗" : ""}`);

if (failures.length > 0) {
  console.log(`\n  Failed assertions:`);
  failures.forEach((f, i) => {
    console.log(`\n  ${i + 1}. ${f.label}`);
    if (f.detail) console.log(`     ${f.detail}`);
  });
}

console.log(`\n  ${failed === 0 ? "🟢 ALL TESTS PASSED — TIS-27 is live end-to-end" : "🔴 FAILURES — see above"}`);

if (failed > 0) {
  const likelyCause = failures.some(f => f.detail?.includes("sha256") || f.detail?.includes("blake3"))
    ? "\n  ⚠  scan_hash_algo failures indicate the updated tdns.ts has not been deployed to Replit yet."
    : failures.some(f => f.detail?.includes("2.5.0"))
    ? "\n  ⚠  Version mismatch — server is running an older build."
    : "";
  if (likelyCause) console.log(likelyCause);
  process.exit(1);
}