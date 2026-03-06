// PlenumNET TDNS — Popup Script v1.0.2
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. — Applied Physics Division
// Patent(s) Pending — All Rights Reserved
//
// Fixes in v1.0.2 vs v1.0.1:
//  - GUI width 480px, font 14px base
//  - .plm name: TLD stripped automatically (google.com → google, plenumnet.replit.app → plenumnet-replit)
//  - Duplicate URL check before registration, with clear warning + overwrite confirmation
//  - Registered URLs tracked in chrome.storage.local
//  - Live name preview in register modal

"use strict";

const DEFAULT_API = "https://plenumnet.replit.app";
const MASTER_KEY  = "CAPOMASTRO";   // UX testing key — replace with billing integration

const AXIS_COLORS = {
  WHO:"#D4A017", WHAT:"#059669", WHERE:"#818CF8",
  WHEN:"#F87171", WHY:"#C084FC", HOW:"#38BDF8", PEACE:"#4ADE80",
};
const SCORE_DEFS = [
  { key:"trustIndex",          name:"Trust",      lk:"trustLabel"      },
  { key:"privacyFocusedIndex", name:"PFI",        lk:"pfiLabel"        },
  { key:"privacyScore",        name:"Privacy",    lk:"privacyLabel"    },
  { key:"maturityScore",       name:"Maturity",   lk:"maturityLabel"   },
  { key:"complexityScore",     name:"Complexity", lk:"complexityLabel" },
];
const PRO_IDS     = ["session_replay", "crm"];
const PIHOLE_CATS = {
  analytics:      "oisd.nl/big (analytics category)",
  social:         "dbl.oisd.nl (social trackers)",
  advertising:    "adaway.github.io/hosts (ads)",
  session_replay: "No standard list — use domain entries below",
  crm:            "No standard list — use domain entries below",
};

// ── State ─────────────────────────────────────────────────────────────────────
let currentResult  = null;
let currentUrl     = "";
let currentTabId   = null;
let prevScanHash   = null;
let tier           = "free";
let apiBase        = DEFAULT_API;
let activeBlockTab = "ublock";

// ── Helpers ───────────────────────────────────────────────────────────────────
const esc = s => String(s ?? "").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");
const el  = id => document.getElementById(id);

function scoreColor(v) {
  if (v >= 75) return "#4ADE80";
  if (v >= 60) return "#D4A017";
  if (v >= 40) return "#F59E0B";
  return "#EF4444";
}

function confPips(conf, color) {
  let h = '<div class="conf-bar">';
  for (let i = 1; i <= 9; i++)
    h += `<div class="conf-pip${i <= conf ? " on" : ""}"${i <= conf ? ` style="background:${color}"` : ""}></div>`;
  return h + "</div>";
}

function formatTime(iso) {
  try { return new Date(iso).toLocaleTimeString([], { hour:"2-digit", minute:"2-digit", second:"2-digit" }); }
  catch { return iso; }
}

// ── .plm name derivation ──────────────────────────────────────────────────────
// Strips TLD and www prefix, returns a clean identifier.
// google.com           → google
// www.google.com       → google
// plenumnet.replit.app → plenumnet-replit   (keep first N-1 labels, drop TLD)
// docs.github.io       → docs-github        (github.io treated as TLD)
//
// Common multi-part TLDs that should be fully stripped:
const COMPOUND_TLDS = new Set(["co.uk","co.nz","co.za","com.au","com.br","gov.uk","gov.au","gc.ca","ac.uk","net.au","org.uk"]);

function suggestPlmName(hostname) {
  if (!hostname) return "";
  // Strip www
  let h = hostname.toLowerCase().replace(/^www\./, "");
  // Check compound TLD
  const parts = h.split(".");
  if (parts.length >= 3) {
    const last2 = parts.slice(-2).join(".");
    if (COMPOUND_TLDS.has(last2)) {
      // Drop last 2 segments (compound TLD)
      return parts.slice(0, -2).join("-");
    }
  }
  // Standard: drop last segment (TLD)
  if (parts.length >= 2) {
    return parts.slice(0, -1).join("-");
  }
  return parts[0];
}

// Sanitise: only allow alphanumeric + hyphen
function sanitisePlmName(raw) {
  return raw.toLowerCase().replace(/[^a-z0-9-]/g, "-").replace(/-{2,}/g, "-").replace(/^-|-$/g, "");
}

// ── Storage helpers ───────────────────────────────────────────────────────────
function storageGet(keys) {
  return new Promise(resolve => {
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.get(keys, resolve);
    } else {
      const r = {};
      keys.forEach(k => { try { r[k] = JSON.parse(localStorage.getItem(k)); } catch {} });
      resolve(r);
    }
  });
}
function storageSet(obj) {
  return new Promise(resolve => {
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.set(obj, resolve);
    } else {
      Object.entries(obj).forEach(([k, v]) => localStorage.setItem(k, JSON.stringify(v)));
      resolve();
    }
  });
}

// ── Registered URL tracking ───────────────────────────────────────────────────
// Format: { [normalised_url]: ".plm name" }
let registeredUrls = {};

async function loadRegisteredUrls() {
  const data = await storageGet(["tdns_registered_urls"]);
  registeredUrls = data.tdns_registered_urls || {};
}
async function saveRegisteredUrl(url, name) {
  registeredUrls[normaliseUrl(url)] = name;
  await storageSet({ tdns_registered_urls: registeredUrls });
}
function normaliseUrl(url) {
  try { const u = new URL(url); return `${u.protocol}//${u.hostname}${u.pathname}`.replace(/\/$/, ""); }
  catch { return url; }
}
function existingRegistration(url) {
  return registeredUrls[normaliseUrl(url)] || null;
}

// ── Tier ──────────────────────────────────────────────────────────────────────
async function loadTier() {
  const data = await storageGet(["tdns_tier","tdns_api_base"]);
  tier    = data.tdns_tier    || "free";
  apiBase = data.tdns_api_base || DEFAULT_API;
  updateTierUI();
}
async function setTier(t) {
  tier = t;
  await storageSet({ tdns_tier: t });
  updateTierUI();
  if (currentResult) { renderTrackers(currentResult); renderBlockRecs(currentResult); }
}
function updateTierUI() {
  const isPro = tier === "pro";
  const pill = el("tier-pill");
  if (pill) { pill.textContent = isPro ? "PRO" : "FREE"; pill.className = `tier-pill ${isPro ? "tier-pro" : "tier-free"}`; }
  const badge = el("s-tier-badge");
  if (badge) { badge.textContent = isPro ? "PRO" : "FREE"; badge.className = `badge-lg ${isPro ? "badge-pro" : "badge-free"}`; }
  const sub = el("s-tier-sub");
  if (sub) sub.textContent = isPro
    ? "Pro — unlimited scans, all tracker categories, block recommendations"
    : "Free — unlimited scans, 3 rescans/day";
  const dr = el("s-downgrade-row");
  if (dr) dr.style.display = isPro ? "flex" : "none";
  const ub = el("s-upgrade-btn");
  if (ub) ub.style.display = isPro ? "none" : "";
}

// ── Panel visibility ──────────────────────────────────────────────────────────
function showLoading(msg) {
  el("loading").style.display    = "flex";
  el("error-panel").style.display= "none";
  el("report").style.display     = "none";
  el("loading-text").textContent = msg || "Scanning…";
}
function showError(msg) {
  el("loading").style.display    = "none";
  el("error-panel").style.display= "block";
  el("report").style.display     = "none";
  el("error-message").textContent= msg;
}
function showReport() {
  el("loading").style.display    = "none";
  el("error-panel").style.display= "none";
  el("report").style.display     = "block";
}

// ── Address ───────────────────────────────────────────────────────────────────
function renderAddress(r) {
  el("address").textContent      = r.address;
  el("hptp-badge").style.display = r.hptp_mandatory ? "inline-flex" : "none";
  el("crd-badge").textContent    = `CRD ${r.crd}`;
  const algo = r.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "SHA-256";
  el("hash-preview").textContent = `${algo}: ${(r.scan_hash || "").substring(0, 14)}…`;
  el("scan-time").textContent    = formatTime(r.scannedAt);
  el("address").onclick = () => {
    navigator.clipboard.writeText(r.address).catch(() => {});
    const orig = el("address").style.color;
    el("address").style.color = "#4ADE80";
    setTimeout(() => { el("address").style.color = orig; }, 800);
  };
}

// ── Scores ────────────────────────────────────────────────────────────────────
function renderScores(r) {
  el("scores-grid").innerHTML = SCORE_DEFS.map(d => {
    const v = r.scores?.[d.key] ?? 0;
    const c = scoreColor(v);
    return `<div class="score-card">
      <div class="score-value" style="color:${c}">${v}</div>
      <div class="score-label" style="color:${c}">${esc(r.scores?.[d.lk] || "")}</div>
      <div class="score-name">${esc(d.name)}</div>
    </div>`;
  }).join("");
}

// ── Security Alerts ─────────────────────────────────────────────────────────────
function renderSecurityAlerts(r) {
  const list = r.findings || [];
  const crit = list.filter(f => f.severity === "Critical").length;
  const warn = list.filter(f => f.severity === "Warning").length;
  const badge = el("findings-badge");
  if (crit)      { badge.textContent = `${crit} Critical`; badge.className = "section-badge badge-critical"; }
  else if (warn) { badge.textContent = `${warn} Warning`;  badge.className = "section-badge badge-warning"; }
  else           { badge.textContent = list.length ? `${list.length} Info` : "Clean"; badge.className = `section-badge ${list.length ? "badge-info" : "badge-clean"}`; }

  el("findings-list").innerHTML = list.length === 0
    ? `<p style="font-size:12px;color:var(--green);padding:6px 0">✓ No security alerts. Clean scan.</p>`
    : list.map(f => `
        <div class="finding finding-${f.severity.toLowerCase()}">
          <div class="finding-dot dot-${f.severity.toLowerCase()}"></div>
          <div>
            <div class="finding-title">${esc(f.title)}</div>
            <div class="finding-message">${esc(f.message)}</div>
            ${f.dimension ? `<div class="finding-dim">${f.dimension}</div>` : ""}
          </div>
        </div>`).join("");

  openSection("findings"); // always open
}

// ── Dimensions ────────────────────────────────────────────────────────────────
function renderDimensions(r) {
  const dims  = r.dimensions || [];
  const axes  = ["WHO","WHAT","WHERE","WHEN","WHY","HOW","PEACE"];
  const byAx  = {};
  dims.forEach(d => { (byAx[d.category] = byAx[d.category] || []).push(d); });
  let html = "";
  axes.forEach(axis => {
    const group = byAx[axis] || [];
    if (!group.length) return;
    const c = AXIS_COLORS[axis] || "#888";
    html += `<div class="axis-group"><div class="axis-label" style="color:${c}">${axis}<div class="axis-divider"></div></div>`;
    group.forEach(d => {
      const pol = d.polarity === "higher_is_better"
        ? (d.value===3?"↑":d.value===1?"↓":"→")
        : d.polarity === "higher_is_worse"
        ? (d.value===1?"↑":d.value===3?"↓":"→") : "→";
      html += `
        <div class="dim-row" data-dim="${d.number}">
          <div class="dim-num">D${String(d.number).padStart(2,"0")}</div>
          <div class="trit-badge" style="color:${c};border:1px solid ${c}33">${d.value}</div>
          <div>
            <div class="dim-question">${esc(d.question)} <span style="color:var(--fg-dim)">${pol}</span></div>
            <div class="dim-label" style="color:${c}">${esc(d.label)}</div>
            <div class="dim-meaning">${esc(d.meaning || "")}</div>
          </div>
          ${confPips(d.confidence, c)}
        </div>`;
    });
    html += `</div>`;
  });
  el("dimensions-content").innerHTML = html;
  el("dimensions-content").querySelectorAll(".dim-row").forEach(row =>
    row.addEventListener("click", () => row.classList.toggle("expanded"))
  );
}

// ── Security Headers ──────────────────────────────────────────────────────────
function renderHeaders(r) {
  const headers = r.security_headers || [];
  const crit = headers.filter(h => h.finding_severity === "Critical").length;
  const warn = headers.filter(h => h.finding_severity === "Warning").length;
  const badge = el("headers-badge");
  if (crit)      { badge.textContent = `${crit} Critical`; badge.className = "section-badge badge-critical"; }
  else if (warn) { badge.textContent = `${warn} Warning`;  badge.className = "section-badge badge-warning"; }
  else           { badge.textContent = "OK"; badge.className = "section-badge badge-clean"; }

  el("headers-content").innerHTML = headers.map(h => {
    const isRisk = h.header === "x-powered-by";
    const icon   = isRisk ? (h.present ? "⚠" : "✓") : (h.present ? "✓" : "✗");
    const ic     = isRisk
      ? (h.present ? "var(--amber)" : "var(--green)")
      : (h.present ? "var(--green)" : h.finding_severity === "Critical" ? "var(--red)" : "var(--amber)");
    return `<div class="header-row">
      <div class="header-status" style="color:${ic}">${icon}</div>
      <div><div class="header-name">${esc(h.header)}</div><div class="header-purpose">${esc(h.purpose)}</div></div>
      <div class="header-value">${h.present ? esc(h.value) : "—"}</div>
    </div>`;
  }).join("");
}

// ── Trackers ──────────────────────────────────────────────────────────────────
function renderTrackers(r) {
  const trackers = r.trackers || [];
  const isPro = tier === "pro";
  const detected = trackers.filter(t => t.detected).length;
  const hasSession = trackers.find(t => t.id === "session_replay")?.detected;

  const badge = el("trackers-badge");
  if (hasSession && isPro) { badge.textContent = "Session Replay!"; badge.className = "section-badge badge-critical"; }
  else if (detected)       { badge.textContent = `${detected} Detected`; badge.className = "section-badge badge-warning"; }
  else                     { badge.textContent = "Clean"; badge.className = "section-badge badge-clean"; }

  let html = "";
  trackers.forEach(t => {
    const isGated = PRO_IDS.includes(t.id) && !isPro;
    if (isGated) {
      html += `<div class="tracker-gate">
        <div>
          <div class="gate-name">${esc(t.name)} <span style="font-size:11px;color:var(--fg-dim)">🔒 Pro</span></div>
          <div class="gate-lock">${esc(t.sensitivity)} sensitivity · ${esc(t.privacy_law.split("—")[0].trim())}</div>
        </div>
        <button class="btn-unlock" onclick="openUpgradeModal()">Unlock</button>
      </div>`;
      return;
    }
    const sc  = t.sensitivity === "Critical" ? "sens-c" : t.sensitivity === "High" ? "sens-h" : "sens-m";
    const rc  = t.detected ? (t.sensitivity === "Critical" ? "detected-critical" : t.sensitivity === "High" ? "detected-high" : "detected-medium") : "";
    const nc  = t.detected ? (t.sensitivity === "Critical" ? "var(--red)" : t.sensitivity === "High" ? "var(--amber)" : "var(--blue)") : "var(--green)";
    html += `<div class="tracker-row ${rc}">
      <div class="tracker-header">
        <div class="tracker-name" style="color:${nc}">${t.detected ? "" : "✓ "}${esc(t.name)}</div>
        <span class="tracker-sens ${sc}">${esc(t.sensitivity)}</span>
      </div>
      ${t.detected
        ? (t.domains.length
          ? `<div class="tracker-domains">${t.domains.slice(0,4).map(esc).join(" · ")}${t.domains.length>4?` +${t.domains.length-4}`:""}</div>`
          : `<div class="tracker-domains" style="color:var(--amber)">Pattern match — exact domain not in response headers</div>`)
        : `<div class="tracker-none">Not detected in initial HTTP response</div>`}
      <div class="tracker-law">${esc(t.privacy_law)}</div>
    </div>`;
  });

  el("trackers-content").innerHTML = html;
  if (hasSession && isPro) openSection("trackers");
}

// ── Block Recommendations (Pro) ───────────────────────────────────────────────
function renderBlockRecs(r) {
  const container = el("block-recs-container");
  if (!container) return;
  if (tier !== "pro") { container.innerHTML = ""; return; }
  const detected = (r.trackers || []).filter(t => t.detected && t.domains.length > 0);
  if (!detected.length) { container.innerHTML = ""; return; }
  const allDomains = [...new Set(detected.flatMap(t => t.domains))];
  const rules = {
    ublock:  allDomains.map(d => `||${d}^`).join("\n"),
    hosts:   allDomains.map(d => `0.0.0.0 ${d}`).join("\n"),
    pihole:  detected.map(t => `# ${t.name}\n${PIHOLE_CATS[t.id] || "Custom list"}`).join("\n\n"),
    browser: detected.map(t => `${t.name}: block ${(t.domains[0] || "detected domain")}`).join("\n"),
  };
  const tabs = [{id:"ublock",label:"uBlock"},{id:"hosts",label:"Hosts File"},{id:"pihole",label:"Pi-hole"},{id:"browser",label:"Browser"}];
  container.innerHTML = `
    <div class="block-recs">
      <div class="block-recs-title">🛡 Block Recommendations — ${allDomains.length} domain${allDomains.length!==1?"s":""}</div>
      <div class="block-tab-bar" id="block-tab-bar">
        ${tabs.map(t => `<div class="block-tab${activeBlockTab===t.id?" active":""}" data-tab="${t.id}">${t.label}</div>`).join("")}
      </div>
      <div class="block-rules-box" id="block-rules-box">${esc(rules[activeBlockTab])}</div>
      <button class="btn-copy-rules" id="btn-copy-rules">Copy</button>
    </div>`;
  container.querySelectorAll(".block-tab").forEach(tab =>
    tab.addEventListener("click", () => {
      activeBlockTab = tab.dataset.tab;
      container.querySelectorAll(".block-tab").forEach(t => t.classList.toggle("active", t.dataset.tab === activeBlockTab));
      el("block-rules-box").textContent = rules[activeBlockTab];
    })
  );
  el("btn-copy-rules").onclick = () => {
    navigator.clipboard.writeText(rules[activeBlockTab]).catch(() => {});
    el("btn-copy-rules").textContent = "Copied!";
    setTimeout(() => { el("btn-copy-rules").textContent = "Copy"; }, 1500);
  };
}

// ── Section toggles ───────────────────────────────────────────────────────────
function openSection(name) {
  document.querySelector(`[data-section="${name}"]`)?.classList.add("open");
  el(`body-${name}`)?.classList.add("open");
}
document.querySelectorAll(".section-header[data-section]").forEach(hdr =>
  hdr.addEventListener("click", () => {
    hdr.classList.toggle("open");
    el(`body-${hdr.dataset.section}`)?.classList.toggle("open");
  })
);

// ── Modal helpers ─────────────────────────────────────────────────────────────
function openModal(name)  { el(`modal-${name}`)?.classList.add("open"); }
function closeModal(name) { el(`modal-${name}`)?.classList.remove("open"); }
document.querySelectorAll(".modal-close[data-modal]").forEach(btn =>
  btn.addEventListener("click", () => closeModal(btn.dataset.modal))
);
document.querySelectorAll(".modal-backdrop").forEach(bd =>
  bd.addEventListener("click", e => { if (e.target === bd) bd.classList.remove("open"); })
);
window.openUpgradeModal = () => { closeModal("settings"); openModal("upgrade"); };

// ── Settings ──────────────────────────────────────────────────────────────────
function setupSettings() {
  el("btn-settings").onclick = () => {
    el("s-api-display").textContent = apiBase;
    el("s-api-input").value = apiBase;
    openModal("settings");
  };
  el("tier-pill").onclick = () => { el("s-api-display").textContent = apiBase; openModal("settings"); };
  el("s-upgrade-btn").onclick = window.openUpgradeModal;
  el("s-downgrade-btn").onclick = async () => {
    await setTier("free");
    showFeedback("s-feedback", "Reverted to Free tier.", "ok");
  };
  el("s-edit-api-btn").onclick = () => {
    const row = el("s-api-edit");
    const vis = row.style.display !== "none";
    row.style.display = vis ? "none" : "block";
    el("s-edit-api-btn").textContent = vis ? "Edit" : "Cancel";
  };
  el("s-save-api-btn").onclick = async () => {
    const val = el("s-api-input").value.trim().replace(/\/$/, "");
    if (!val.startsWith("http")) { showFeedback("s-feedback", "URL must start with http/https", "err"); return; }
    apiBase = val;
    await storageSet({ tdns_api_base: apiBase });
    el("s-api-display").textContent = apiBase;
    el("s-api-edit").style.display = "none";
    el("s-edit-api-btn").textContent = "Edit";
    showFeedback("s-feedback", "Endpoint saved.", "ok");
  };
  el("s-clear-cache-btn").onclick = () => {
    chrome?.runtime?.sendMessage({ type:"CLEAR_CACHE", tabId:currentTabId });
    showFeedback("s-feedback", "Session cache cleared.", "ok");
  };
}
function showFeedback(id, msg, type = "ok") {
  const d = el(id); if (!d) return;
  d.innerHTML = `<div class="modal-result ${type}">${esc(msg)}</div>`;
  if (type === "ok") setTimeout(() => { d.innerHTML = ""; }, 2800);
}

// ── Upgrade ───────────────────────────────────────────────────────────────────
function setupUpgrade() {
  const submit = () => {
    const key = el("upgrade-key-input").value.trim().toUpperCase();
    if (!key) return;
    if (key === MASTER_KEY) {
      setTier("pro");
      el("upgrade-result").innerHTML = `<div class="modal-result ok">✓ Pro unlocked. All features enabled.</div>`;
      el("upgrade-key-input").value = "";
      setTimeout(() => closeModal("upgrade"), 1800);
    } else {
      el("upgrade-result").innerHTML = `<div class="modal-result err">✗ Invalid key. Contact RSalvi@Salvigroup.com for a subscription.</div>`;
    }
  };
  el("btn-upgrade-submit").onclick = submit;
  el("upgrade-key-input").addEventListener("keydown", e => { if (e.key === "Enter") submit(); });
}

// ── PDF Report ────────────────────────────────────────────────────────────────
function setupPDF() {
  el("btn-pdf").onclick = async () => {
    if (!currentResult) return;

    // Write data to storage FIRST, fully awaited, then open the tab.
    // chrome.tabs.create is more reliable than window.open from an extension popup.
    await storageSet({
      tdns_print_result: currentResult,
      tdns_print_tier:   tier,
      tdns_print_ts:     Date.now(),     // timestamp so report.html can detect stale data
    });

    const reportUrl = typeof chrome?.runtime?.getURL === "function"
      ? chrome.runtime.getURL("report.html")
      : "report.html";

    if (typeof chrome?.tabs?.create === "function") {
      chrome.tabs.create({ url: reportUrl });
    } else {
      // Fallback for dev/non-Chrome environments
      window.open(reportUrl, "_blank");
    }
  };
}

// ── Verify Hash ───────────────────────────────────────────────────────────────
function setupVerify() {
  el("btn-verify").onclick = () => {
    if (!currentResult) return;
    el("verify-hash-val").textContent = currentResult.scan_hash;
    el("verify-addr-val").textContent = currentResult.address;
    el("verify-input").value = "";
    el("verify-result").innerHTML = "";
    openModal("verify");
  };
  el("btn-verify-run").onclick = () => {
    const input    = el("verify-input").value.trim().toLowerCase();
    const expected = currentResult?.scan_hash?.toLowerCase();
    if (!input) return;
    el("verify-result").innerHTML = input === expected
      ? `<div class="modal-result ok">✓ Hash matches — scan integrity verified.</div>`
      : `<div class="modal-result err">✗ Hash mismatch — data may have changed or been tampered with.</div>`;
  };
}

// ── Decode ────────────────────────────────────────────────────────────────────
function setupDecode() {
  el("btn-decode").onclick = () => {
    if (!currentResult) return;
    el("decode-addr-val").textContent = currentResult.address;
    el("decode-breakdown").innerHTML = (currentResult.dimensions || []).map(d => {
      const c = AXIS_COLORS[d.category] || "#888";
      return `<div style="display:grid;grid-template-columns:32px 20px 1fr;gap:7px;align-items:start;padding:4px 0;border-bottom:1px solid var(--border)">
        <span style="font-family:var(--mono);font-size:10px;color:var(--fg-dim)">D${d.number}</span>
        <span style="font-size:13px;font-weight:800;color:${c}">${d.value}</span>
        <span style="font-size:11px;color:var(--fg-muted)">${esc(d.category)} · ${esc(d.question)}<br>
          <span style="color:${c};font-weight:700">${esc(d.label)}</span>
        </span>
      </div>`;
    }).join("");
    openModal("decode");
  };
}

// ── .plm Register ─────────────────────────────────────────────────────────────
function setupRegister() {
  const nameInput = el("register-name");
  const preview   = el("register-preview");
  const dupWarn   = el("dup-warning");

  // Live preview + sanitise
  nameInput?.addEventListener("input", () => {
    const clean = sanitisePlmName(nameInput.value);
    preview.textContent = (clean || "—") + ".plm";
  });

  el("btn-register").onclick = () => {
    el("register-url-val").textContent = currentUrl;

    // Suggest name: TLD-stripped hostname
    const suggested = suggestPlmName((() => { try { return new URL(currentUrl).hostname; } catch { return ""; } })());
    nameInput.value = sanitisePlmName(suggested);
    preview.textContent = (sanitisePlmName(suggested) || "—") + ".plm";
    el("register-result").innerHTML = "";

    // Duplicate check
    const existing = existingRegistration(currentUrl);
    if (existing) {
      dupWarn.style.display = "block";
      el("dup-existing-name").textContent = existing;
    } else {
      dupWarn.style.display = "none";
    }

    openModal("register");
  };

  el("btn-register-run").onclick = async () => {
    const raw  = nameInput.value.trim();
    const name = sanitisePlmName(raw);
    if (!name) { el("register-result").innerHTML = `<div class="modal-result err">Please enter a valid name.</div>`; return; }

    el("register-result").innerHTML = `<span style="color:var(--fg-muted);font-size:12px">Registering…</span>`;
    try {
      const resp = await fetch(`${apiBase}/api/tdns/register`, {
        method:"POST", headers:{"Content-Type":"application/json"},
        body: JSON.stringify({ name, url: currentUrl }),
      });
      const data = await resp.json();

      if (resp.status === 409) {
        // Server-side duplicate by URL — already registered under a different name
        el("register-result").innerHTML = `<div class="modal-result warn">
          ⚠ Already registered as <strong style="font-family:var(--mono)">${esc(data.existing_name || "?")}</strong>. 
          Use a different name to create a new entry, or confirm to overwrite.
        </div>`;
        return;
      }

      if (data.status === "ok") {
        await saveRegisteredUrl(currentUrl, data.name);
        dupWarn.style.display = "none";
        el("register-result").innerHTML = `<div class="modal-result ok">
          <div style="font-weight:700">${esc(data.name)} registered ✓</div>
          <div style="font-family:var(--mono);font-size:11px;margin-top:4px">${esc(data.address)}</div>
        </div>`;
      } else {
        el("register-result").innerHTML = `<div class="modal-result err">${esc(data.error || "Registration failed")}</div>`;
      }
    } catch {
      el("register-result").innerHTML = `<div class="modal-result err">Network error — check your connection.</div>`;
    }
  };
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
  showLoading("Scanning " + (() => { try { return new URL(url).hostname; } catch { return url; } })() + "…");
  try { el("nav-url").textContent = new URL(url).hostname; } catch {}

  try {
    const resp = await fetch(`${apiBase}/api/tdns/scan`, {
      method:"POST", headers:{"Content-Type":"application/json"},
      body: JSON.stringify({ url }),
    });
    if (!resp.ok) {
      const err = await resp.json().catch(() => ({ error:`HTTP ${resp.status}` }));
      throw new Error(err.error || `Server error ${resp.status}`);
    }
    const result = await resp.json();
    if (!result.address) throw new Error(result.error || "Invalid server response");

    if (prevScanHash && prevScanHash !== result.scan_hash) {
      result.findings = [
        { id:"F0", severity:"Warning", title:"Infrastructure Change Detected",
          message:"Scan hash changed since last scan. One or more dimensions have new trit values." },
        ...(result.findings || []),
      ];
    }

    currentResult = result;
    renderAddress(result);
    renderScores(result);
    renderSecurityAlerts(result);
    renderDimensions(result);
    renderHeaders(result);
    renderTrackers(result);
    renderBlockRecs(result);
    renderSEO(result);
    showReport();

    chrome?.runtime?.sendMessage({ type:"CACHE_SCAN", tabId:currentTabId, result });
  } catch (err) {
    showError(err.message || "Scan failed. Check your connection or try a different URL.");
  }
}


// ── SEO Analysis ──────────────────────────────────────────────────────────────
function renderSEO(r) {
  const signals = r.seo_signals || [];
  const fails   = signals.filter(s => s.status === "fail").length;
  const warns   = signals.filter(s => s.status === "warn").length;
  const passes  = signals.filter(s => s.status === "pass").length;

  const badge = el("seo-badge");
  if (badge) {
    if (fails)       { badge.textContent = `${fails} Issue${fails>1?"s":""}`; badge.className = "section-badge badge-critical"; }
    else if (warns)  { badge.textContent = `${warns} Warning${warns>1?"s":""}`; badge.className = "section-badge badge-warning"; }
    else if (passes) { badge.textContent = `${passes} Passed`; badge.className = "section-badge badge-clean"; }
    else             { badge.textContent = "—"; badge.className = "section-badge"; }
  }

  const catOrder = ["Discoverability","Metadata","Social","Technical"];
  const byCat = {};
  catOrder.forEach(c => { byCat[c] = []; });
  signals.forEach(s => { if (byCat[s.category]) byCat[s.category].push(s); });

  const icons = { pass:"✓", warn:"⚠", fail:"✗" };
  let html = "";

  catOrder.forEach(cat => {
    const group = byCat[cat];
    if (!group.length) return;
    html += `<div style="font-size:10px;font-weight:800;text-transform:uppercase;letter-spacing:.1em;
      color:var(--fg-dim);padding:8px 2px 4px;border-bottom:1px solid var(--border);margin-bottom:5px">${cat}</div>`;
    group.forEach(s => {
      html += `
        <div class="seo-row seo-${s.status}" data-seo="${esc(s.id)}">
          <div class="seo-icon">${icons[s.status]}</div>
          <div>
            <div class="seo-signal">${esc(s.signal)}</div>
            <div class="seo-detail">${esc(s.detail)}</div>
            <div class="seo-rec">💡 ${esc(s.recommendation)}</div>
          </div>
        </div>`;
    });
  });

  if (!html) html = '<p style="font-size:12px;color:var(--fg-muted);padding:6px 0">No SEO data available for this page.</p>';
  el("seo-content").innerHTML = html;

  // Click to expand recommendation (pass rows hidden by default)
  el("seo-content").querySelectorAll(".seo-row").forEach(row =>
    row.addEventListener("click", () => row.classList.toggle("expanded"))
  );

  // Auto-open if there are failures
  if (fails > 0) openSection("seo");
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────
document.addEventListener("DOMContentLoaded", async () => {
  await loadTier();
  await loadRegisteredUrls();
  setupSettings();
  setupUpgrade();
  setupPDF();
  setupVerify();
  setupDecode();
  setupRegister();
  setupRescan();

  try {
    chrome.runtime.sendMessage({ type:"GET_TAB_URL" }, response => {
      if (chrome.runtime.lastError) { showError("Extension error: " + chrome.runtime.lastError.message); return; }
      const url = response?.url;
      currentTabId = response?.tabId;
      if (!url || /^(chrome|chrome-extension|about|edge):/.test(url)) {
        showError("Navigate to a website first. Browser internal pages cannot be scanned.");
        return;
      }
      runScan(url);
    });
  } catch {
    // Dev fallback
    runScan("https://example.com");
  }
});
