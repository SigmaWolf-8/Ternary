"use strict";

const esc = s => String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");
const el  = id => document.getElementById(id);

const AXIS_COLORS = {
  WHO:"#D4A017", WHAT:"#059669", WHERE:"#818CF8",
  WHEN:"#F87171", WHY:"#C084FC", HOW:"#38BDF8", PEACE:"#4ADE80"
};

function scoreColor(v) {
  if (v >= 75) return "#4ADE80";
  if (v >= 60) return "#D4A017";
  if (v >= 40) return "#F59E0B";
  return "#EF4444";
}

function pips(conf) {
  let h = '<div class="conf-row">';
  for (let i = 1; i <= 9; i++) h += `<div class="pip${i <= conf ? " on" : ""}"></div>`;
  return h + '</div>';
}

function render(result, tier) {
  try {
    el("no-data").style.display = "none";
    el("report-root").style.display = "block";

    el("r-url").textContent       = result.meta?.url || "—";
    el("r-timestamp").textContent = "Scanned: " + new Date(result.scannedAt).toLocaleString();
    el("r-tier").textContent      = `Tier: ${tier.toUpperCase()} — ${result.scan_hash_algo || "sha256-js"}`;

    const addrEl = el("r-address");
    if (addrEl.childNodes.length > 0) {
      addrEl.childNodes[0].textContent = result.address + " ";
    } else {
      addrEl.insertBefore(document.createTextNode(result.address + " "), addrEl.firstChild);
    }
    el("r-hptp").style.display = result.hptp_mandatory ? "inline-block" : "none";
    el("r-crd").textContent     = result.crd;
    el("r-hash-algo").textContent = result.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "SHA-256";
    el("r-hash").textContent    = (result.scan_hash || "").substring(0, 32) + "…";
    el("r-full-hash").textContent= result.scan_hash || "—";
    el("r-algo-name").textContent= result.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "SHA-256 (JS path)";

    const scoreDefs = [
      { key:"trustIndex",          name:"Trust Index",    lk:"trustLabel"          },
      { key:"privacyFocusedIndex", name:"PFI",            lk:"pfiLabel"            },
      { key:"privacyScore",        name:"Privacy",        lk:"privacyLabel"        },
      { key:"maturityScore",       name:"Maturity",       lk:"maturityLabel"       },
      { key:"complexityScore",     name:"Complexity",     lk:"complexityLabel"     },
    ];
    el("r-scores").innerHTML = scoreDefs.map(d => {
      const v = result.scores?.[d.key] ?? 0;
      const c = scoreColor(v);
      return `<div class="score-card">
        <div class="score-val" style="color:${c}">${v}</div>
        <div class="score-lbl" style="color:${c}">${esc(result.scores?.[d.lk] || "")}</div>
        <div class="score-name">${esc(d.name)}</div>
      </div>`;
    }).join("");

    const findings = result.findings || [];
    const crit = findings.filter(f=>f.severity==="Critical").length;
    const warn = findings.filter(f=>f.severity==="Warning").length;
    el("r-findings-count").textContent = crit ? `— ${crit} Critical` : warn ? `— ${warn} Warning` : "— Clean";
    el("r-findings").innerHTML = findings.length
      ? findings.map(f => `
          <div class="finding finding-${f.severity.toLowerCase()}">
            <div class="f-dot f-${f.severity.toLowerCase()}"></div>
            <div>
              <div class="f-title">${esc(f.title)}</div>
              <div class="f-msg">${esc(f.message)}</div>
              ${f.dimension ? `<div class="f-dim">${f.dimension}</div>` : ""}
            </div>
          </div>`).join("")
      : `<div style="color:#059669;font-size:12px">✓ No findings. Clean scan.</div>`;

    el("r-dims").innerHTML = (result.dimensions || []).map(d => {
      const c = AXIS_COLORS[d.category] || "#888";
      return `<tr>
        <td class="dim-num-cell">D${d.number}</td>
        <td class="dim-axis-cell" style="color:${c}">${d.category}</td>
        <td><div class="trit-num" style="color:${c};border-color:${c}33">${d.value}</div></td>
        <td class="dim-q-cell">${esc(d.question)}</td>
        <td class="dim-lbl-cell" style="color:${c}">${esc(d.label)}</td>
        <td class="dim-mean-cell">${esc(d.meaning)}</td>
        <td>${pips(d.confidence)}</td>
      </tr>`;
    }).join("");

    el("r-headers").innerHTML = (result.security_headers || []).map(h => {
      const isRisk = h.header === "x-powered-by";
      const icon   = isRisk
        ? (h.present ? "⚠" : "✓")
        : (h.present ? "✓" : (h.finding_severity === "Critical" ? "✗" : "✗"));
      const iconColor = isRisk
        ? (h.present ? "#F59E0B" : "#059669")
        : (h.present ? "#059669" : (h.finding_severity === "Critical" ? "#EF4444" : "#F59E0B"));
      return `<tr>
        <td style="color:${iconColor};font-size:13px;width:18px">${icon}</td>
        <td><div class="hdr-name">${esc(h.header)}</div></td>
        <td><div class="hdr-val">${h.present ? esc(h.value) : "—"}</div></td>
        <td class="hdr-purpose">${esc(h.purpose)}</td>
        <td style="font-family:monospace;font-size:9px;color:var(--dim)">${esc(h.dimension)}</td>
      </tr>`;
    }).join("");

    const isPro = tier !== "free";
    const premiumIds = ["session_replay","crm"];
    el("r-trackers").innerHTML = (result.trackers || []).map(t => {
      const isGated = premiumIds.includes(t.id) && !isPro;
      if (isGated) {
        return `<div class="tracker-row-full" style="filter:blur(4px);position:relative;">
          <div class="tr-header">
            <div class="tr-name">${esc(t.name)}</div>
            <span class="tr-sens sens-m">Pro Feature</span>
          </div>
          <div class="tr-law">Upgrade to Pro to view ${esc(t.name)} detection results.</div>
        </div>`;
      }
      const rowClass = t.detected
        ? (t.sensitivity === "Critical" ? "det-critical" : t.sensitivity === "High" ? "det-high" : "det-medium") : "";
      const sensClass = t.sensitivity === "Critical" ? "sens-c" : t.sensitivity === "High" ? "sens-h" : "sens-m";
      return `<div class="tracker-row-full ${rowClass}">
        <div class="tr-header">
          <div class="tr-name" style="color:${t.detected?(t.sensitivity==="Critical"?"#EF4444":t.sensitivity==="High"?"#F59E0B":"#3B82F6"):"#059669"}">
            ${t.detected ? "" : "✓ "}${esc(t.name)}
          </div>
          <span class="tr-sens ${sensClass}">${esc(t.sensitivity)}</span>
        </div>
        ${t.detected
          ? (t.domains.length > 0
            ? `<div class="tr-domains">${t.domains.map(esc).join(" · ")}</div>`
            : `<div class="tr-domains" style="color:#F59E0B">Pattern match — domain not listed</div>`)
          : `<div class="tr-clean">Not detected in initial HTTP response</div>`}
        <div class="tr-law">${esc(t.privacy_law)}</div>
      </div>`;
    }).join("");
  } catch (err) {
    console.error("TDNS report render error:", err);
    el("no-data").style.display = "block";
    el("no-data").innerHTML = `<h2>Render Error</h2><p>${esc(err.message)}</p><p style="font-size:11px;margin-top:8px">Check the browser console (F12) for details.</p>`;
  }
}

function loadAndRender() {
  if (typeof chrome !== "undefined" && chrome.storage && chrome.storage.local) {
    let attempts = 0;
    const maxAttempts = 10;
    const pollInterval = 200;

    function tryLoad() {
      attempts++;
      chrome.storage.local.get(["tdns_print_result", "tdns_print_tier"], data => {
        if (data && data.tdns_print_result) {
          render(data.tdns_print_result, data.tdns_print_tier || "free");
        } else if (attempts < maxAttempts) {
          setTimeout(tryLoad, pollInterval);
        } else {
          el("no-data").style.display = "block";
        }
      });
    }

    tryLoad();
  } else {
    const hash = location.hash.slice(1);
    if (hash) {
      try {
        const { result, tier } = JSON.parse(decodeURIComponent(hash));
        render(result, tier || "free");
      } catch {
        el("no-data").style.display = "block";
      }
    } else {
      el("no-data").style.display = "block";
    }
  }
}

function init() {
  const btnPrint = document.getElementById("btn-do-print");
  const btnClose = document.getElementById("btn-do-close");
  if (btnPrint) btnPrint.addEventListener("click", () => window.print());
  if (btnClose) btnClose.addEventListener("click", () => window.close());
  loadAndRender();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
