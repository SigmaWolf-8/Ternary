// PlenumNET TDNS — Popup Script v1.0.0
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. — Applied Physics Division
// Patent(s) Pending — All Rights Reserved
//
// Calls POST /api/tdns/scan, renders full 27-dim report.
// Design: #090807 bg / #D4A017 gold / #E4DFD5 fg — PlenumNET SignHere system.

"use strict";

// ── Configuration ─────────────────────────────────────────────────────────────
const API_BASE  = "https://plenumnet.replit.app";
const TIER      = "free";  // "free" | "pro" | "team" | "enterprise"

// Axis accent colours — from spec §3.7
const AXIS_COLORS = {
  WHO:   "#D4A017",
  WHAT:  "#059669",
  WHERE: "#818CF8",
  WHEN:  "#F87171",
  WHY:   "#C084FC",
  HOW:   "#38BDF8",
  PEACE: "#4ADE80",
};

// Score card config
const SCORE_DEFS = [
  { key: "trustIndex",          name: "Trust",      labelKey: "trustLabel"          },
  { key: "privacyFocusedIndex", name: "PFI",        labelKey: "pfiLabel"            },
  { key: "privacyScore",        name: "Privacy",    labelKey: "privacyLabel"        },
  { key: "maturityScore",       name: "Maturity",   labelKey: "maturityLabel"       },
  { key: "complexityScore",     name: "Complexity", labelKey: "complexityLabel"     },
];

// ── State ─────────────────────────────────────────────────────────────────────
let currentResult   = null;
let currentUrl      = "";
let currentTabId    = null;
let prevScanHash    = null;   // for change detection on rescan

// ── Helpers ───────────────────────────────────────────────────────────────────
const esc = s => String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");
const el  = id => document.getElementById(id);

function scoreColor(v) {
  if (v >= 75) return "#4ADE80";
  if (v >= 60) return "#D4A017";
  if (v >= 40) return "#F59E0B";
  return "#EF4444";
}

function confPips(conf, axisColor) {
  const pips = [];
  for (let i = 1; i <= 9; i++) {
    const filled = i <= conf;
    const color  = filled ? (conf <= 3 ? "red" : conf <= 6 ? "amber" : "") : "";
    pips.push(`<div class="conf-pip${filled ? " filled" : ""}${color ? " "+color : ""}" style="${filled && !color ? `background:${axisColor}` : ""}"></div>`);
  }
  return `<div class="conf-bar">${pips.join("")}</div>`;
}

function severityDot(sev) {
  const map = { Critical: "critical", Warning: "warning", Info: "info" };
  return `<div class="finding-dot dot-${map[sev] || "info"}"></div>`;
}

function trackerSensClass(s) {
  if (s === "Critical") return "sens-critical";
  if (s === "High")     return "sens-high";
  return "sens-medium";
}

function formatTime(iso) {
  try { return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" }); }
  catch { return iso; }
}

// ── Show / hide panels ────────────────────────────────────────────────────────
function showLoading(msg) {
  el("loading").style.display = "flex";
  el("error-panel").style.display = "none";
  el("report").style.display = "none";
  el("loading-text").textContent = msg || "Scanning…";
}

function showError(msg) {
  el("loading").style.display = "none";
  el("error-panel").style.display = "block";
  el("report").style.display = "none";
  el("error-message").textContent = msg;
}

function showReport() {
  el("loading").style.display = "none";
  el("error-panel").style.display = "none";
  el("report").style.display = "block";
}

// ── Render address block ──────────────────────────────────────────────────────
function renderAddress(r) {
  el("address").textContent       = r.address;
  el("hptp-badge").style.display  = r.hptp_mandatory ? "inline-flex" : "none";
  el("crd-badge").textContent     = `CRD ${r.crd}`;
  el("hash-preview").innerHTML    = `<span style="color:var(--fg-dim);font-size:9px">${r.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "HASH"}</span>: ${r.scan_hash.substring(0, 16)}…`;
  el("scan-time").textContent     = formatTime(r.scannedAt);

  el("address").onclick = () => {
    navigator.clipboard.writeText(r.address).catch(() => {});
    el("address").style.color = "#4ADE80";
    setTimeout(() => el("address").style.color = "", 800);
  };
}

// ── Render 5 score cards ──────────────────────────────────────────────────────
function renderScores(r) {
  const grid = el("scores-grid");
  grid.innerHTML = SCORE_DEFS.map(def => {
    const val   = r.scores[def.key];
    const label = r.scores[def.labelKey];
    const color = scoreColor(val);
    return `
      <div class="score-card">
        <div class="score-value" style="color:${color}">${val}</div>
        <div class="score-label" style="color:${color}">${label}</div>
        <div class="score-name">${def.name}</div>
      </div>`;
  }).join("");
}

// ── Render findings ───────────────────────────────────────────────────────────
function renderFindings(r) {
  const list = r.findings || [];
  const critCount = list.filter(f => f.severity === "Critical").length;
  const warnCount = list.filter(f => f.severity === "Warning").length;

  // Badge on section header
  const badge = el("findings-badge");
  if (critCount > 0) {
    badge.textContent = `${critCount} Critical`;
    badge.className = "section-badge badge-critical";
  } else if (warnCount > 0) {
    badge.textContent = `${warnCount} Warning`;
    badge.className = "section-badge badge-warning";
  } else {
    badge.textContent = list.length > 0 ? `${list.length} Info` : "Clean";
    badge.className = `section-badge ${list.length ? "badge-info" : "badge-clean"}`;
  }

  el("findings-list").innerHTML = list.length === 0
    ? `<p style="padding:8px 0;font-size:11px;color:var(--green)">✓ No findings. Clean scan.</p>`
    : list.map(f => `
      <div class="finding finding-${f.severity.toLowerCase()}">
        ${severityDot(f.severity)}
        <div class="finding-body">
          <div class="finding-title">${esc(f.title)}</div>
          <div class="finding-message">${esc(f.message)}</div>
          ${f.dimension ? `<div class="finding-dim">${f.dimension}</div>` : ""}
        </div>
      </div>`).join("");

  // Auto-open if there are critical findings
  if (critCount > 0) openSection("findings");
}

// ── Render 27-dim breakdown ───────────────────────────────────────────────────
function renderDimensions(r) {
  const dims = r.dimensions || [];
  const byAxis = {};
  dims.forEach(d => {
    if (!byAxis[d.category]) byAxis[d.category] = [];
    byAxis[d.category].push(d);
  });

  const axisOrder = ["WHO", "WHAT", "WHERE", "WHEN", "WHY", "HOW", "PEACE"];
  let html = "";

  axisOrder.forEach(axis => {
    const axisDims = byAxis[axis] || [];
    if (!axisDims.length) return;
    const color = AXIS_COLORS[axis] || "#888";
    html += `<div class="axis-group">
      <div class="axis-label" style="color:${color}">
        ${axis}
        <div class="axis-divider"></div>
      </div>`;

    axisDims.forEach(d => {
      const trit     = d.value;
      const polIcon  = d.polarity === "higher_is_better"
        ? (trit === 3 ? "↑" : trit === 1 ? "↓" : "→")
        : d.polarity === "higher_is_worse"
        ? (trit === 1 ? "↑" : trit === 3 ? "↓" : "→")
        : "→";

      html += `
        <div class="dim-row" data-dim="${d.number}">
          <div class="dim-num">D${String(d.number).padStart(2,"0")}</div>
          <div class="trit-badge" style="color:${color};border:1px solid ${color}33">${trit}</div>
          <div class="dim-content">
            <div class="dim-question">${esc(d.question)} <span style="color:var(--fg-dim)">${polIcon}</span></div>
            <div class="dim-label" style="color:${color}">${esc(d.label)}</div>
            <div class="dim-meaning">${esc(d.meaning)}</div>
          </div>
          ${confPips(d.confidence, color)}
        </div>`;
    });

    html += `</div>`;
  });

  el("dimensions-content").innerHTML = html;

  // Click to expand meaning
  el("dimensions-content").querySelectorAll(".dim-row").forEach(row => {
    row.addEventListener("click", () => row.classList.toggle("expanded"));
  });
}

// ── Render security headers ───────────────────────────────────────────────────
function renderHeaders(r) {
  const headers = r.security_headers || [];
  const critCount = headers.filter(h => h.finding_severity === "Critical").length;
  const warnCount = headers.filter(h => h.finding_severity === "Warning").length;

  const badge = el("headers-badge");
  if (critCount > 0) { badge.textContent = `${critCount} Critical`; badge.className = "section-badge badge-critical"; }
  else if (warnCount > 0) { badge.textContent = `${warnCount} Warning`; badge.className = "section-badge badge-warning"; }
  else { badge.textContent = "OK"; badge.className = "section-badge badge-clean"; }

  el("headers-content").innerHTML = headers.map(h => {
    const isRisk = h.header === "x-powered-by";
    const statusIcon = isRisk
      ? (h.present ? `<span style="color:var(--amber)">⚠</span>` : `<span style="color:var(--green)">✓</span>`)
      : (h.present ? `<span style="color:var(--green)">✓</span>` : `<span style="color:${h.finding_severity === "Critical" ? "var(--red)" : "var(--amber)"}">✗</span>`);

    return `
      <div class="header-row">
        <div class="header-status">${statusIcon}</div>
        <div>
          <div class="header-name">${esc(h.header)}</div>
          <div class="header-purpose">${esc(h.purpose)}</div>
        </div>
        <div class="header-value">${h.present ? esc(h.value) : "—"}</div>
      </div>`;
  }).join("");
}

// ── Render tracker intelligence ───────────────────────────────────────────────
function renderTrackers(r) {
  const trackers = r.trackers || [];
  const detectedCount = trackers.filter(t => t.detected).length;
  const hasSession = trackers.find(t => t.id === "session_replay")?.detected;

  const badge = el("trackers-badge");
  if (hasSession) { badge.textContent = "Session Replay!"; badge.className = "section-badge badge-critical"; }
  else if (detectedCount > 0) { badge.textContent = `${detectedCount} Detected`; badge.className = "section-badge badge-warning"; }
  else { badge.textContent = "Clean"; badge.className = "section-badge badge-clean"; }

  // Free tier: categories 4 (session_replay) and 5 (crm) are gated
  const premiumIds = ["session_replay", "crm"];
  const isPro = TIER !== "free";

  let html = "";
  trackers.forEach(t => {
    const isPremium = premiumIds.includes(t.id) && !isPro;
    const sensClass = trackerSensClass(t.sensitivity);
    const rowClass  = t.detected
      ? (t.sensitivity === "Critical" ? "detected-critical" : t.sensitivity === "High" ? "detected-high" : "detected-medium")
      : "";

    const inner = `
      <div class="tracker-header">
        <div class="tracker-name">
          ${t.detected
            ? `<span style="color:var(--${t.sensitivity === "Critical" ? "red" : t.sensitivity === "High" ? "amber" : "blue"})">${esc(t.name)}</span>`
            : `<span style="color:var(--green)">✓ ${esc(t.name)}</span>`}
        </div>
        <span class="tracker-sensitivity ${sensClass}">${esc(t.sensitivity)}</span>
      </div>
      ${t.detected
        ? (t.domains.length > 0
          ? `<div class="tracker-domains">${t.domains.slice(0,4).map(esc).join(" · ")}${t.domains.length > 4 ? ` +${t.domains.length-4}` : ""}</div>`
          : `<div class="tracker-domains" style="color:var(--amber)">Pattern match — domain not listed</div>`)
        : `<div class="tracker-none">Not detected in initial response</div>`}
      <div class="tracker-law">${esc(t.privacy_law)}</div>`;

    if (isPremium) {
      html += `
        <div class="tracker-row premium-gate ${rowClass}">
          <div class="blurred-content">${inner}</div>
          <div class="premium-overlay">
            <p>Detect ${esc(t.name)} — session capture risk</p>
            <button class="btn-upgrade" onclick="window.open('https://plenumnet.replit.app/#upgrade','_blank')">Upgrade to Pro — $9/mo</button>
          </div>
        </div>`;
    } else {
      html += `<div class="tracker-row ${rowClass}">${inner}</div>`;
    }
  });

  if (!html) html = `<p style="padding:8px 0;font-size:11px;color:var(--green)">✓ No trackers detected.</p>`;

  // Block recommendations (Pro+ only) — scaffold
  if (isPro && detectedCount > 0) {
    const allDomains = trackers.filter(t => t.detected).flatMap(t => t.domains);
    if (allDomains.length > 0) {
      const uboRules   = allDomains.map(d => `||${d}^`).join("\n");
      const hostsRules = allDomains.map(d => `0.0.0.0 ${d}`).join("\n");
      html += `
        <div style="margin-top:8px;padding:8px 10px;background:var(--bg-muted);border:1px solid var(--border);border-radius:var(--radius)">
          <div style="font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:.07em;color:var(--fg-muted);margin-bottom:6px">Block Recommendations</div>
          <div style="font-family:var(--mono);font-size:9px;color:var(--fg-dim);white-space:pre;overflow-x:auto">${esc(uboRules)}</div>
          <button class="btn-action" style="margin-top:6px;font-size:10px" onclick="navigator.clipboard.writeText(${JSON.stringify(uboRules)})">Copy uBlock Rules</button>
        </div>`;
    }
  }

  el("trackers-content").innerHTML = html;

  // Auto-open if session replay detected
  if (hasSession) openSection("trackers");
}

// ── Section toggle ─────────────────────────────────────────────────────────────
function openSection(name) {
  const header = document.querySelector(`[data-section="${name}"]`);
  const body   = el(`body-${name}`);
  if (header && body) {
    header.classList.add("open");
    body.classList.add("open");
  }
}

function setupSectionToggles() {
  document.querySelectorAll(".section-header[data-section]").forEach(header => {
    header.addEventListener("click", () => {
      const name = header.dataset.section;
      const body = el(`body-${name}`);
      if (!body) return;
      header.classList.toggle("open");
      body.classList.toggle("open");
    });
  });
}

// ── Verify Hash Modal ─────────────────────────────────────────────────────────
function setupVerifyHash() {
  el("btn-verify").onclick = () => {
    if (!currentResult) return;
    el("verify-hash-val").textContent = currentResult.scan_hash;
    el("verify-addr-val").textContent = currentResult.address;
    el("verify-input").value = "";
    el("verify-result").innerHTML = "";
    el("modal-verify").classList.add("open");
  };

  el("btn-verify-run").onclick = () => {
    const input = el("verify-input").value.trim().toLowerCase();
    const expected = currentResult?.scan_hash?.toLowerCase();
    if (!input) return;
    const match = input === expected;
    el("verify-result").innerHTML = match
      ? `<div class="modal-result match">✓ Hash matches — scan integrity verified.</div>`
      : `<div class="modal-result no-match">✗ Hash mismatch — scan data may have changed or been tampered with.</div>`;
  };
}

// ── Decode Address Modal ──────────────────────────────────────────────────────
function setupDecodeAddress() {
  el("btn-decode").onclick = () => {
    if (!currentResult) return;
    el("decode-addr-val").textContent = currentResult.address;

    const dims = currentResult.dimensions || [];
    const html  = dims.map(d => {
      const color = AXIS_COLORS[d.category] || "#888";
      return `
        <div style="display:grid;grid-template-columns:32px 18px 1fr;gap:6px;align-items:start;padding:3px 0;border-bottom:1px solid var(--border)">
          <span style="font-family:var(--mono);font-size:9px;color:var(--fg-dim)">D${d.number}</span>
          <span style="font-size:12px;font-weight:700;color:${color}">${d.value}</span>
          <span style="font-size:10px;color:var(--fg-muted)">${esc(d.category)} · ${esc(d.question)}<br>
            <span style="color:${color};font-weight:600">${esc(d.label)}</span></span>
        </div>`;
    }).join("");

    el("decode-breakdown").innerHTML = html;
    el("modal-decode").classList.add("open");
  };
}

// ── .plm Register Modal ───────────────────────────────────────────────────────
function setupRegister() {
  el("btn-register").onclick = () => {
    el("register-url-val").textContent = currentUrl;
    el("register-name").value = currentUrl ? new URL(currentUrl).hostname.replace(/^www\./, "") : "";
    el("register-result").innerHTML = "";
    el("modal-register").classList.add("open");
  };

  el("btn-register-run").onclick = async () => {
    const name = el("register-name").value.trim();
    if (!name) return;
    el("register-result").innerHTML = `<span style="color:var(--fg-muted)">Registering…</span>`;
    try {
      const resp = await fetch(`${API_BASE}/api/tdns/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name, url: currentUrl }),
      });
      const data = await resp.json();
      if (data.status === "ok") {
        el("register-result").innerHTML = `
          <div style="color:var(--green);font-weight:600">${esc(data.name)} registered</div>
          <div style="font-family:var(--mono);font-size:10px;color:var(--fg-muted);margin-top:4px">${esc(data.address)}</div>`;
      } else {
        el("register-result").innerHTML = `<span style="color:var(--red)">${esc(data.error || "Registration failed")}</span>`;
      }
    } catch (e) {
      el("register-result").innerHTML = `<span style="color:var(--red)">Network error</span>`;
    }
  };
}

// ── Close modals ──────────────────────────────────────────────────────────────
function setupModalClose() {
  document.querySelectorAll(".modal-close[data-modal]").forEach(btn => {
    btn.addEventListener("click", () => {
      el(`modal-${btn.dataset.modal}`).classList.remove("open");
    });
  });
  document.querySelectorAll(".modal-backdrop").forEach(bd => {
    bd.addEventListener("click", e => { if (e.target === bd) bd.classList.remove("open"); });
  });
}

// ── Rescan ────────────────────────────────────────────────────────────────────
function setupRescan() {
  el("btn-rescan").onclick = () => {
    if (!currentUrl) return;
    prevScanHash = currentResult?.scan_hash;
    runScan(currentUrl);
  };
}

// ── Main scan ─────────────────────────────────────────────────────────────────
async function runScan(url) {
  currentUrl = url;
  showLoading("Scanning " + new URL(url).hostname + "…");

  // Update nav URL
  try { el("nav-url").textContent = new URL(url).hostname; } catch {}

  try {
    const resp = await fetch(`${API_BASE}/api/tdns/scan`, {
      method:  "POST",
      headers: { "Content-Type": "application/json" },
      body:    JSON.stringify({ url }),
    });

    if (!resp.ok) {
      const err = await resp.json().catch(() => ({ error: `HTTP ${resp.status}` }));
      throw new Error(err.error || `Server error ${resp.status}`);
    }

    const result = await resp.json();
    if (result.status !== "ok" && !result.address) {
      throw new Error(result.error || "Invalid response from server");
    }

    currentResult = result;

    // Change detection on rescan
    if (prevScanHash && prevScanHash !== result.scan_hash) {
      // Prepend a change detection finding
      result.findings.unshift({
        id: "F0", severity: "Warning",
        title: "Infrastructure Change Detected",
        message: `Scan hash changed since last scan. BLAKE3 address has shifted — one or more dimensions have new trit values.`,
      });
    }

    renderAddress(result);
    renderScores(result);
    renderFindings(result);
    renderDimensions(result);
    renderHeaders(result);
    renderTrackers(result);
    showReport();

    // Open findings if critical
    const hasCritical = (result.findings || []).some(f => f.severity === "Critical");
    if (hasCritical) openSection("findings");
    else openSection("findings"); // always open findings by default

    // Cache result in background
    if (currentTabId) {
      chrome.runtime.sendMessage({ type: "CACHE_SCAN", tabId: currentTabId, result });
    }

  } catch (err) {
    showError(err.message || "Scan failed. Check your connection or try a different URL.");
  }
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────
document.addEventListener("DOMContentLoaded", async () => {
  setupSectionToggles();
  setupVerifyHash();
  setupDecodeAddress();
  setupRegister();
  setupModalClose();
  setupRescan();

  // Get current tab URL from background
  try {
    chrome.runtime.sendMessage({ type: "GET_TAB_URL" }, response => {
      if (chrome.runtime.lastError) {
        showError("Extension communication error: " + chrome.runtime.lastError.message);
        return;
      }
      const url = response?.url;
      currentTabId = response?.tabId;

      if (!url || url.startsWith("chrome://") || url.startsWith("chrome-extension://") || url.startsWith("about:")) {
        showError("Navigate to a website to scan it. Chrome internal pages cannot be scanned.");
        return;
      }

      runScan(url);
    });
  } catch (e) {
    // Fallback for testing outside Chrome
    runScan("https://example.com");
  }
});
