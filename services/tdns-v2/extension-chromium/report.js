/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

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
    el("no-data").style.display = "none";
    el("report-root").style.display = "block";

    // Header meta
    el("r-url").textContent       = result.meta?.url || "—";
    el("r-timestamp").textContent = "Scanned: " + new Date(result.scannedAt).toLocaleString();
    el("r-tier").textContent      = `Tier: ${tier.toUpperCase()} — ${result.scan_hash_algo || "sha256-js"}`;

    // Address
    el("r-address").childNodes[0].textContent = result.address + " ";
    el("r-hptp").style.display = result.hptp_mandatory ? "inline-block" : "none";
    el("r-crd").textContent     = result.crd;
    el("r-hash-algo").textContent = result.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "SHA-256";
    el("r-hash").textContent    = (result.scan_hash || "").substring(0, 32) + "…";
    el("r-full-hash").textContent= result.scan_hash || "—";
    el("r-algo-name").textContent= result.scan_hash_algo === "blake3-rs" ? "BLAKE3" : "SHA-256 (JS path)";

    // Scores
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

    // Findings
    const findings = result.findings || [];
    const crit = findings.filter(f=>f.severity==="Critical").length;
    const warn = findings.filter(f=>f.severity==="Warning").length;
    el("r-findings-count").textContent = crit ? `— ${crit} Critical` : warn ? `— ${warn} Warning` : "— Clean"; // Security Alerts
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

    // Dimensions
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

    // Headers
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

    // Trackers
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
    // SEO Analysis
    const seoSignals = result.seo_signals || [];
    const catOrder   = ["Discoverability","Metadata","Social","Technical"];
    const byCat      = {};
    catOrder.forEach(c => { byCat[c] = []; });
    seoSignals.forEach(s => { if (byCat[s.category]) byCat[s.category].push(s); });

    const seoIcons = { pass:"✓", warn:"⚠", fail:"✗" };
    let seoHtml = "";

    catOrder.forEach(cat => {
      const group = byCat[cat];
      if (!group.length) return;
      seoHtml += `<tr class="seo-cat-header"><td colspan="5">${esc(cat)}</td></tr>`;
      group.forEach(s => {
        seoHtml += `<tr class="seo-${s.status}">
          <td class="seo-status">${seoIcons[s.status]}</td>
          <td class="seo-signal-cell">${esc(s.signal)}</td>
          <td style="font-size:10px;color:var(--dim)">${esc(s.category)}</td>
          <td class="seo-detail-cell">${esc(s.detail)}</td>
          <td class="seo-rec-cell">${s.status !== "pass" ? esc(s.recommendation) : "—"}</td>
        </tr>`;
      });
    });

    if (!seoHtml) seoHtml = '<tr><td colspan="5" style="color:var(--muted)">No SEO data available.</td></tr>';
    if (el("r-seo")) el("r-seo").innerHTML = seoHtml;

  }

  // Load from chrome.storage.local
  function loadAndRender() {
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.get(["tdns_print_result","tdns_print_tier"], data => {
        if (!data.tdns_print_result) {
          el("no-data").style.display = "block";
          return;
        }
        render(data.tdns_print_result, data.tdns_print_tier || "free");
      });
    } else {
      // Dev fallback — try URL hash
      const hash = location.hash.slice(1);
      if (hash) {
        try {
          const { result, tier } = JSON.parse(decodeURIComponent(hash));
          render(result, tier || "free");
        } catch { el("no-data").style.display = "block"; }
      } else {
        el("no-data").style.display = "block";
      }
    }
  }

  loadAndRender();