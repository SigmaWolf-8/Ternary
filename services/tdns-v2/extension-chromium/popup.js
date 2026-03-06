// PlenumNET TDNS — Popup Script
// Capomastro Holdings Ltd. — Applied Physics Division
// All JS extracted from popup.html to comply with MV3 CSP (no inline scripts/handlers).

// ── Tab switching ────────────────────────────────────────────────────────────
function switchTab(tab) {
  document.getElementById('panelResolve').classList.toggle('active', tab === 'resolve');
  document.getElementById('panelScanner').classList.toggle('active', tab === 'scanner');
  document.getElementById('tabResolveBtn').classList.toggle('active', tab === 'resolve');
  document.getElementById('tabScannerBtn').classList.toggle('active', tab === 'scanner');
  if (tab === 'scanner') detectCurrentUrl();
}

document.getElementById('tabResolveBtn').addEventListener('click', () => switchTab('resolve'));
document.getElementById('tabScannerBtn').addEventListener('click', () => switchTab('scanner'));

// ── Resolve tab ──────────────────────────────────────────────────────────────
const nameInput  = document.getElementById('nameInput');
const resolveBtn = document.getElementById('resolveBtn');
const result     = document.getElementById('result');
const addressBox = document.getElementById('addressBox');
const scanHash   = document.getElementById('scanHash');
const hptpStatus = document.getElementById('hptpStatus');
const errorMsg   = document.getElementById('errorMsg');
const statusDot  = document.getElementById('statusDot');
const statusText = document.getElementById('statusText');
const openFull   = document.getElementById('openFull');

// Server health check — shows local vs live endpoint in status bar
chrome.runtime.sendMessage({ type: 'health' }, (resp) => {
  if (resp && resp.ok) {
    statusDot.classList.remove('offline');
    const v   = resp.data.version || '?';
    const n   = resp.data.entities || 0;
    const src = resp.endpoint && resp.endpoint.includes('localhost') ? 'local' : 'live';
    statusText.textContent = `Connected (${src}) — v${v} — ${n} entities`;
  } else {
    statusDot.classList.add('offline');
    statusText.textContent = 'TDNS offline — Docker or plenumnet.replit.app unreachable';
  }
});

function doResolve() {
  let name = nameInput.value.trim();
  if (!name) return;
  if (!name.endsWith('.plm')) name += '.plm';

  resolveBtn.disabled = true;
  errorMsg.classList.remove('show');
  result.classList.remove('show');

  chrome.runtime.sendMessage({ type: 'resolve', name }, (resp) => {
    resolveBtn.disabled = false;
    if (resp && resp.ok && resp.data.status === 'ok') {
      const d = resp.data;
      const crd = d.crd || 1;
      addressBox.innerHTML = `${d.address} <span class="crd-badge">CRD:${crd}</span>`;
      scanHash.textContent = (d.scan_hash || '').substring(0, 16) + '...';
      hptpStatus.textContent = d.hptp_mandatory ? 'Yes' : 'No';
      openFull.onclick = () => {
        chrome.tabs.create({
          url: chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(name)}`)
        });
      };
      result.classList.add('show');
    } else {
      errorMsg.textContent = `${name} not registered. Use the API to register it first.`;
      errorMsg.classList.add('show');
    }
  });
}

resolveBtn.addEventListener('click', doResolve);
nameInput.addEventListener('keydown', (e) => { if (e.key === 'Enter') doResolve(); });
nameInput.focus();

// ── Auto-update check ────────────────────────────────────────────────────────
chrome.runtime.sendMessage({ type: 'update_check' }, (resp) => {
  if (!resp) return;
  const banner   = document.getElementById('updateBanner');
  const bannerTxt = document.getElementById('updateBannerText');
  if (resp.update_available) {
    bannerTxt.textContent =
      `Update available: v${resp.current} → v${resp.latest} — reload extension to apply`;
    banner.classList.add('show');
  } else if (resp.error) {
    // Silently ignore — offline or GitHub unreachable
  }
});

document.getElementById('updateDismiss').addEventListener('click', () => {
  document.getElementById('updateBanner').classList.remove('show');
});

// ── Scanner tab ──────────────────────────────────────────────────────────────
const CAT_COLORS = {
  WHO:   { border: '#D4A017', text: '#D4A017', bg: '#D4A01715' },
  WHAT:  { border: '#059669', text: '#059669', bg: '#05966915' },
  WHERE: { border: '#818CF8', text: '#818CF8', bg: '#818CF815' },
  WHEN:  { border: '#F87171', text: '#F87171', bg: '#F8717115' },
  WHY:   { border: '#C084FC', text: '#C084FC', bg: '#C084FC15' },
  HOW:   { border: '#38BDF8', text: '#38BDF8', bg: '#38BDF815' },
  PEACE: { border: '#4ADE80', text: '#4ADE80', bg: '#4ADE8015' },
};

let currentTabUrl       = null;
let scanProgressInterval = null;

function detectCurrentUrl() {
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs && tabs[0] && tabs[0].url) {
      currentTabUrl = tabs[0].url;
      document.getElementById('currentUrl').textContent = currentTabUrl;
    } else {
      document.getElementById('currentUrl').textContent = 'Could not detect page URL';
    }
  });
}

function startScanProgress() {
  const fill = document.getElementById('progressFill');
  const pct  = document.getElementById('progressPct');
  const dim  = document.getElementById('progressDim');
  const probes = [
    'WHO: entity type','WHO: audience','WHO: operator','WHO: hosting',
    'WHAT: form','WHAT: content','WHAT: consumers','WHAT: intelligence',
    'WHERE: visibility','WHERE: auth','WHERE: scale','WHERE: transport',
    'WHEN: era','WHEN: availability','WHEN: freshness','WHEN: real-time',
    'WHY: commerce','WHY: data','WHY: policies','WHY: cost',
    'HOW: delivery','HOW: direction','HOW: updates','HOW: memory',
    'PEACE: encryption','PEACE: trackers','PEACE: audit'
  ];
  let step = 0;
  scanProgressInterval = setInterval(() => {
    if (step >= 27) { clearInterval(scanProgressInterval); return; }
    const p = Math.round((step + 1) / 27 * 90);
    fill.style.width  = p + '%';
    pct.textContent   = p + '%';
    dim.textContent   = 'PROBING ' + probes[step];
    step++;
  }, 111);
}

function stopScanProgress(success) {
  clearInterval(scanProgressInterval);
  document.getElementById('progressFill').style.width = '100%';
  document.getElementById('progressPct').textContent  = success ? '100%' : 'ERR';
}

function buildDimsHtml(dimensions) {
  const byCategory = {};
  dimensions.forEach(d => {
    if (!byCategory[d.category]) byCategory[d.category] = [];
    byCategory[d.category].push(d);
  });

  let html = '';
  Object.entries(byCategory).forEach(([cat, dims]) => {
    const cc    = CAT_COLORS[cat] || { border: '#5A5548', text: '#8A8578', bg: '#1A181615' };
    const trits = dims.map(d => d.value).join('');
    html += `<div class="cat-block">
      <div class="cat-label" style="background:${cc.bg};color:${cc.text};border-left:2px solid ${cc.border}">
        <span>${cat}</span>
        <span style="font-family:monospace;letter-spacing:0.08em;font-size:10px;">${trits}</span>
      </div>`;
    dims.forEach(d => {
      const isHptp  = (d.number === 15 || d.number === 16) && d.value === 3;
      const confNum = parseFloat(d.confidence);
      const confPct = isNaN(confNum) ? d.confidence : Math.round(confNum * 100) + '%';
      const hptpTag = isHptp ? `<span class="hptp-tag">HPTP</span>` : '';
      html += `<div class="dim-row" style="border-left-color:${cc.border}40;">
        <span class="dim-trit" style="color:${cc.text}">${d.value}</span>
        <div class="dim-info">
          <div class="dim-q">${d.question}${hptpTag}</div>
          <div class="dim-label" style="color:${cc.text}">→ ${d.label}</div>
        </div>
        <span class="dim-conf">${confPct}</span>
      </div>`;
    });
    html += '</div>';
  });
  return html;
}

document.getElementById('scanBtn').addEventListener('click', () => {
  if (!currentTabUrl) {
    document.getElementById('scanError').textContent = 'No page URL detected.';
    document.getElementById('scanError').classList.add('show');
    return;
  }

  document.getElementById('scanError').classList.remove('show');
  document.getElementById('scanResult').classList.remove('show');
  document.getElementById('scanProgress').classList.add('show');
  document.getElementById('scanBtn').disabled = true;
  startScanProgress();

  chrome.runtime.sendMessage({ type: 'scan', url: currentTabUrl }, (resp) => {
    stopScanProgress(resp && resp.ok);
    document.getElementById('scanBtn').disabled = false;

    setTimeout(() => {
      document.getElementById('scanProgress').classList.remove('show');

      if (!resp || !resp.ok || !resp.data) {
        document.getElementById('scanError').textContent =
          'Scan failed — both localhost:3927 and plenumnet.replit.app are unreachable.';
        document.getElementById('scanError').classList.add('show');
        return;
      }

      const d = resp.data;
      document.getElementById('scanAddressBox').textContent =
        d.address_canonical || d.address;
      document.getElementById('scanResultHash').textContent =
        (d.scan_hash || '').substring(0, 16) + '...';
      document.getElementById('scanHptpStatus').textContent =
        d.hptp_mandatory ? '⚠ Yes (femtosecond sync required)' : '✓ No';
      document.getElementById('scanHptpStatus').style.color =
        d.hptp_mandatory ? '#F87171' : '#059669';

      if (d.dimensions && d.dimensions.length) {
        document.getElementById('dimsBody').innerHTML = buildDimsHtml(d.dimensions);
      }

      // Open Full Diagnostic Report → resolve.html for this hostname
      try {
        const plmName = new URL(currentTabUrl).hostname;
        document.getElementById('openReport').addEventListener('click', () => {
          chrome.tabs.create({
            url: chrome.runtime.getURL(
              `resolve.html?name=${encodeURIComponent(plmName)}`
            )
          });
        }, { once: true });
      } catch (_) {}

      document.getElementById('scanResult').classList.add('show');
    }, 300);
  });
});

// Dims accordion toggle
document.getElementById('dimsToggle').addEventListener('click', () => {
  const body    = document.getElementById('dimsBody');
  const chevron = document.getElementById('dimsChevron');
  const open    = body.classList.toggle('open');
  chevron.textContent = open ? '▴' : '▾';
});