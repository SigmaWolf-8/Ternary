// PlenumNET TDNS — Popup UI
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada) — Applied Physics Division

const $ = id => document.getElementById(id);

// ── Tab switching ─────────────────────────────────────────────────────────────
function switchTab(tab) {
  ['Resolve','Scanner'].forEach(t => {
    const on = (tab === t.toLowerCase());
    $('panel'+t).classList.toggle('active', on);
    $('tab'+t+'Btn').classList.toggle('active', on);
  });
  if (tab === 'scanner') detectCurrentUrl();
}
$('tabResolveBtn').addEventListener('click', () => switchTab('resolve'));
$('tabScannerBtn').addEventListener('click', () => switchTab('scanner'));

// ── Health check ──────────────────────────────────────────────────────────────
chrome.runtime.sendMessage({ type:'health' }, resp => {
  if (resp?.ok) {
    $('statusDot').classList.remove('offline');
    $('statusText').textContent = `Connected \u2014 v${resp.data?.version||'?'} \u2014 ${resp.data?.entities||0} entities`;
  } else {
    $('statusDot').classList.add('offline');
    $('statusText').textContent = 'Registry offline \u2014 scanner still works';
  }
});

// ── Register tab ──────────────────────────────────────────────────────────────
// "Register" scans the current page AND posts to PlenumNET to create a .plm entry.
// The .plm name is auto-derived from the hostname.

let registerUrl = null;

function doRegister() {
  if (!registerUrl) {
    // If no current page stored, use the manual input
    let name = $('nameInput').value.trim();
    if (!name) { showError('Enter a hostname or switch to the Scanner tab first.'); return; }
    if (!name.endsWith('.plm')) name += '.plm';
    const baseHost = name.replace(/\.plm$/, '');
    registerUrl = `https://${baseHost}`;
  }

  $('resolveBtn').disabled = true;
  $('errorMsg').classList.remove('show');
  $('result').classList.remove('show');
  $('resolveBtn').textContent = 'Scanning\u2026';

  chrome.runtime.sendMessage({ type:'register', url:registerUrl }, resp => {
    $('resolveBtn').disabled = false;
    $('resolveBtn').textContent = 'Register';

    if (!resp?.ok) {
      showError(resp?.error || 'Registration failed.');
      return;
    }

    const d = resp.data;
    const scan = d.scan || d;
    const hostname = new URL(registerUrl).hostname;

    $('addressBox').innerHTML = (scan.address || d.address) + (d.crd ? ` <span class="crd-badge">CRD:${d.crd}</span>` : '');
    $('scanHash').textContent   = (scan.scan_hash || d.scan_hash || '').substring(0,16) + '\u2026';
    $('hptpStatus').textContent = (scan.hptp_mandatory || d.hptp_mandatory) ? 'Yes' : 'No';
    $('openFull').onclick = () => chrome.tabs.create({
      url: chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(hostname)}`)
    });
    $('result').classList.add('show');

    if (d.name)
      $('addressBox').innerHTML += `<div style="font-size:10px;color:#059669;margin-top:4px">Registered as ${d.name}</div>`;
  });
}

function showError(msg) {
  $('errorMsg').textContent = msg;
  $('errorMsg').classList.add('show');
}

$('resolveBtn').addEventListener('click', doRegister);
$('nameInput').addEventListener('keydown', e => { if (e.key==='Enter') doRegister(); });
$('nameInput').focus();

// ── Update banner ─────────────────────────────────────────────────────────────
chrome.runtime.sendMessage({ type:'update_check' }, resp => {
  if (!resp?.update_available) return;
  $('updateBannerText').textContent = `Update available: v${resp.current} \u2192 v${resp.latest}`;
  $('updateBanner').classList.add('show');
});
$('updateDismiss').addEventListener('click', () => $('updateBanner').classList.remove('show'));

// ── Scanner tab ───────────────────────────────────────────────────────────────
const CAT_COLORS = {
  WHO:'#D4A017', WHAT:'#059669', WHERE:'#818CF8',
  WHEN:'#F87171', WHY:'#C084FC', HOW:'#38BDF8', PEACE:'#4ADE80',
};

let currentTabUrl = null;

function detectCurrentUrl() {
  chrome.tabs.query({ active:true, currentWindow:true }, tabs => {
    currentTabUrl = tabs?.[0]?.url || null;
    $('currentUrl').textContent = currentTabUrl || 'Could not detect URL';
    // Pre-fill register input too
    if (currentTabUrl) {
      try {
        registerUrl = currentTabUrl;
        $('nameInput').placeholder = new URL(currentTabUrl).hostname + '.plm';
      } catch(_) {}
    }
  });
}

// Detect current URL immediately on load too
chrome.tabs.query({ active:true, currentWindow:true }, tabs => {
  if (tabs?.[0]?.url) {
    currentTabUrl = tabs[0].url;
    registerUrl   = currentTabUrl;
    try { $('nameInput').placeholder = new URL(currentTabUrl).hostname + '.plm'; } catch(_) {}
  }
});

function buildDimsHtml(dimensions) {
  const cats = {};
  dimensions.forEach(d => { (cats[d.category]=cats[d.category]||[]).push(d); });
  return Object.entries(cats).map(([cat, dims]) => {
    const col   = CAT_COLORS[cat]||'#8A8578';
    const trits = dims.map(d=>d.value).join('');
    const rows  = dims.map(d => {
      const isHptp = (d.number===15||d.number===16) && d.value===3;
      const pct    = Math.round(d.confidence/9*100)+'%';
      return `<div class="dim-row" style="border-left-color:${col}40">
        <span class="dim-trit" style="color:${col}">${d.value}</span>
        <div class="dim-info">
          <div class="dim-q">${d.question}${isHptp?'<span class="hptp-tag">HPTP</span>':''}</div>
          <div class="dim-label" style="color:${col}">\u2192 ${d.label}</div>
        </div>
        <span class="dim-conf">${pct}</span>
      </div>`;
    }).join('');
    return `<div class="cat-block">
      <div class="cat-label" style="background:${col}15;color:${col};border-left:2px solid ${col}">
        <span>${cat}</span>
        <span style="font-family:monospace;font-size:10px;letter-spacing:.08em">${trits}</span>
      </div>${rows}</div>`;
  }).join('');
}

$('scanBtn').addEventListener('click', () => {
  if (!currentTabUrl) {
    $('scanError').textContent = 'No URL \u2014 switch to a tab first.';
    $('scanError').classList.add('show');
    return;
  }
  $('scanError').classList.remove('show');
  $('scanResult').classList.remove('show');
  $('scanProgress').classList.add('show');
  $('scanBtn').disabled = true;

  const probes = [
    'WHO: entity type','WHO: audience','WHO: operator','WHO: hosting',
    'WHAT: form factor','WHAT: content type','WHAT: consumers','WHAT: AI/ML',
    'WHERE: visibility','WHERE: auth model','WHERE: scale','WHERE: transport',
    'WHEN: tech era','WHEN: availability','WHEN: freshness','WHEN: real-time',
    'WHY: payments','WHY: data appetite','WHY: policies','WHY: cost model',
    'HOW: delivery','HOW: data flow','HOW: updates','HOW: sessions',
    'PEACE: encryption','PEACE: trackers','PEACE: audit',
  ];
  let step = 0;
  const ticker = setInterval(() => {
    if (step >= 27) { clearInterval(ticker); return; }
    const p = Math.round((step+1)/27*85);
    $('progressFill').style.width = p+'%';
    $('progressPct').textContent  = p+'%';
    $('progressDim').textContent  = 'DERIVING '+probes[step++];
  }, 100);

  chrome.runtime.sendMessage({ type:'scan', url:currentTabUrl }, resp => {
    clearInterval(ticker);
    $('progressFill').style.width = '100%';
    $('progressPct').textContent  = '100%';

    setTimeout(() => {
      $('scanProgress').classList.remove('show');
      $('scanBtn').disabled = false;

      if (!resp?.ok) {
        $('scanError').textContent = `Error: ${resp?.error||'unknown'}`;
        $('scanError').classList.add('show');
        return;
      }

      const d = resp.data;
      $('scanAddressBox').textContent = d.address;
      $('scanResultHash').textContent = d.scan_hash.substring(0,16)+'\u2026';
      const hEl = $('scanHptpStatus');
      hEl.textContent = d.hptp_mandatory ? '\u26A0 Yes (femtosecond required)' : '\u2713 No';
      hEl.style.color = d.hptp_mandatory ? '#F87171' : '#059669';

      // Scores inline
      const sec  = d.securityScore  ?? Math.round(((d.dimensions[24].value+d.dimensions[25].value+d.dimensions[26].value)/9)*100);
      const priv = d.privacyScore   ?? Math.round(((d.dimensions[25].value+d.dimensions[18].value)/6)*100);
      const scoreHtml = `
        <div style="display:flex;gap:8px;padding:8px 0;border-top:1px solid #1A1816;margin-top:6px">
          <div style="flex:1;text-align:center">
            <div style="font-size:9px;color:#5A5548;text-transform:uppercase;letter-spacing:.06em;margin-bottom:2px">Security</div>
            <div style="font-size:20px;font-weight:700;color:${sec>=75?'#34D399':sec>=50?'#D4A017':'#F87171'}">${sec}</div>
          </div>
          <div style="flex:1;text-align:center">
            <div style="font-size:9px;color:#5A5548;text-transform:uppercase;letter-spacing:.06em;margin-bottom:2px">Privacy</div>
            <div style="font-size:20px;font-weight:700;color:${priv>=75?'#34D399':priv>=50?'#D4A017':'#F87171'}">${priv}</div>
          </div>
        </div>`;
      $('scanAddressBox').insertAdjacentHTML('afterend', scoreHtml);

      $('dimsBody').innerHTML = buildDimsHtml(d.dimensions);

      try {
        const hostname = new URL(currentTabUrl).hostname;
        $('openReport').addEventListener('click', () => {
          chrome.tabs.create({ url: chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(hostname)}`) });
        }, { once:true });
      } catch(_) {}

      $('scanResult').classList.add('show');
    }, 150);
  });
});

$('dimsToggle').addEventListener('click', () => {
  const open = $('dimsBody').classList.toggle('open');
  $('dimsChevron').textContent = open ? '\u25B4' : '\u25BE';
});
