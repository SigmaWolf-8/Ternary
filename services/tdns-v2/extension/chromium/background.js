// PlenumNET TDNS — Background Service Worker
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada) — Applied Physics Division
//
// Architecture: Extension → plenumnet.replit.app/api/tdns/*
// The server runs the 27-dimension scan engine. No Docker. No localhost.

const API = 'https://plenumnet.replit.app/api/tdns';

// ── Omnibox ───────────────────────────────────────────────────────────────────
chrome.omnibox.onInputStarted.addListener(() =>
  chrome.omnibox.setDefaultSuggestion({ description: 'Resolve a .plm name — type the name' }));

chrome.omnibox.onInputChanged.addListener(async (text, suggest) => {
  const plm = text.trim().endsWith('.plm') ? text.trim() : text.trim() + '.plm';
  try {
    const r = await fetch(`${API}/resolve/${plm}`);
    if (r.ok) {
      const d = await r.json();
      suggest([{ content: plm, description: `\u2713 ${plm} \u2192 ${d.address || 'resolved'}` }]);
    } else {
      suggest([{ content: plm, description: `\u2298 ${plm} \u2014 not registered` }]);
    }
  } catch(_) {
    suggest([{ content: plm, description: `\u26A0 ${plm} \u2014 server unavailable` }]);
  }
});

chrome.omnibox.onInputEntered.addListener((text, disposition) => {
  const plm = text.trim().endsWith('.plm') ? text.trim() : text.trim() + '.plm';
  const u   = chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(plm)}`);
  if (disposition === 'currentTab') chrome.tabs.update({ url: u });
  else chrome.tabs.create({ url: u, active: disposition === 'newForegroundTab' });
});

// ── .plm navigation interception ─────────────────────────────────────────────
function interceptPlm(details) {
  if (details.frameId !== 0) return;
  try {
    const u = new URL(details.url);
    if (u.hostname.endsWith('.plm'))
      chrome.tabs.update(details.tabId, {
        url: chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(u.hostname)}`)
      });
  } catch(_) {}
}
chrome.webNavigation.onErrorOccurred.addListener(interceptPlm);
chrome.webNavigation.onBeforeNavigate.addListener(interceptPlm);

// ── Message handler ───────────────────────────────────────────────────────────
chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {

  if (msg.type === 'health') {
    fetch(`${API}/health`)
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(() => sendResponse({ ok: false }));
    return true;
  }

  if (msg.type === 'resolve') {
    fetch(`${API}/resolve/${msg.name}`)
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'scan') {
    fetch(`${API}/scan`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ url: msg.url })
    })
      .then(r => r.json())
      .then(data => {
        const key = 'scan_' + new URL(msg.url).hostname.replace(/\./g, '_');
        chrome.storage.session.set({ [key]: data });
        sendResponse({ ok: true, data });
      })
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'register') {
    fetch(`${API}/register`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({
        name: msg.name || new URL(msg.url).hostname.replace(/\./g, '-') + '.plm',
        zone: 'public',
        url:  msg.url,
      })
    })
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'get_scan') {
    const key = 'scan_' + msg.hostname.replace(/\./g, '_');
    chrome.storage.session.get(key, items =>
      sendResponse({ ok: true, data: items[key] || null })
    );
    return true;
  }

  if (msg.type === 'update_check') {
    const RAW     = 'https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension-chromium/manifest.json';
    const current = chrome.runtime.getManifest().version;
    fetch(RAW, { cache: 'no-store' })
      .then(r => r.json())
      .then(d => sendResponse({ update_available: d.version !== current, latest: d.version, current }))
      .catch(() => sendResponse({ update_available: false }));
    return true;
  }
});

// ── Periodic update check ─────────────────────────────────────────────────────
chrome.runtime.onInstalled.addListener(() =>
  chrome.alarms.create('plenumnet_update', { periodInMinutes: 360 }));

chrome.alarms.onAlarm.addListener(alarm => {
  if (alarm.name !== 'plenumnet_update') return;
  const RAW     = 'https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/services/tdns-v2/extension-chromium/manifest.json';
  const current = chrome.runtime.getManifest().version;
  fetch(RAW, { cache: 'no-store' })
    .then(r => r.json())
    .then(d => chrome.storage.local.set({ update_available: d.version !== current, latest: d.version }))
    .catch(() => {});
});

console.log('PlenumNET TDNS \u2014 background loaded');
