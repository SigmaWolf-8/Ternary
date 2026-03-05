// PlenumNET TDNS Resolver — Background Service Worker
// Capomastro Holdings Ltd. — Applied Physics Division

const TDNS_API = 'http://localhost:3927';

// ─── Omnibox: type "plm google" to resolve google.plm ─────────────────────

chrome.omnibox.onInputStarted.addListener(() => {
  chrome.omnibox.setDefaultSuggestion({
    description: 'Resolve a .plm name via TDNS — type the name (e.g., "google")'
  });
});

chrome.omnibox.onInputChanged.addListener(async (text, suggest) => {
  const name = text.trim();
  if (!name) return;

  const plmName = name.endsWith('.plm') ? name : name + '.plm';

  try {
    const resp = await fetch(`${TDNS_API}/api/v1/resolve/${plmName}`);
    if (resp.ok) {
      const data = await resp.json();
      suggest([{
        content: plmName,
        description: `✓ ${plmName} → ${data.address || 'resolved'} CRD:${data.crd || '?'}`
      }]);
    } else {
      suggest([{
        content: plmName,
        description: `⊘ ${plmName} — not registered. Press Enter to scan.`
      }]);
    }
  } catch (e) {
    suggest([{
      content: plmName,
      description: `⚠ TDNS API unavailable — is the server running on port 3927?`
    }]);
  }
});

chrome.omnibox.onInputEntered.addListener((text, disposition) => {
  const name = text.trim();
  const plmName = name.endsWith('.plm') ? name : name + '.plm';
  const resolveUrl = chrome.runtime.getURL(`resolve.html?name=${encodeURIComponent(plmName)}`);

  switch (disposition) {
    case 'currentTab':
      chrome.tabs.update({ url: resolveUrl });
      break;
    case 'newForegroundTab':
      chrome.tabs.create({ url: resolveUrl });
      break;
    case 'newBackgroundTab':
      chrome.tabs.create({ url: resolveUrl, active: false });
      break;
  }
});

// ─── Navigation Interception ───────────────────────────────────────────────

chrome.webNavigation.onErrorOccurred.addListener((details) => {
  if (details.frameId !== 0) return;

  try {
    const url = new URL(details.url);
    if (url.hostname.endsWith('.plm')) {
      const plmName = url.hostname;
      const resolveUrl = chrome.runtime.getURL(
        `resolve.html?name=${encodeURIComponent(plmName)}`
      );
      chrome.tabs.update(details.tabId, { url: resolveUrl });
    }
  } catch (e) {
    // Not a valid URL, ignore
  }
});

chrome.webNavigation.onBeforeNavigate.addListener((details) => {
  if (details.frameId !== 0) return;

  try {
    const url = new URL(details.url);
    if (url.hostname.endsWith('.plm')) {
      const plmName = url.hostname;
      const resolveUrl = chrome.runtime.getURL(
        `resolve.html?name=${encodeURIComponent(plmName)}`
      );
      chrome.tabs.update(details.tabId, { url: resolveUrl });
    }
  } catch (e) {
    // Ignore
  }
});

// ─── Message Handler ───────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === 'resolve') {
    fetch(`${TDNS_API}/api/v1/resolve/${msg.name}`)
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'scan') {
    fetch(`${TDNS_API}/api/v1/scan`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: msg.url })
    })
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'register') {
    fetch(`${TDNS_API}/api/v1/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(msg.payload)
    })
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'health') {
    fetch(`${TDNS_API}/api/v1/health`)
      .then(r => r.json())
      .then(data => sendResponse({ ok: true, data }))
      .catch(err => sendResponse({ ok: false, error: err.message }));
    return true;
  }

  if (msg.type === 'update_check') {
    const MANIFEST_URL =
      'https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/' +
      'services/tdns-v2/extension-chromium/manifest.json';
    const current = chrome.runtime.getManifest().version;
    fetch(MANIFEST_URL, { cache: 'no-store' })
      .then(r => r.json())
      .then(remote => {
        const latest = remote.version || current;
        const update_available = latest !== current;
        chrome.storage.local.set({ update_available, latest, current, checked: Date.now() });
        sendResponse({ update_available, latest, current });
      })
      .catch(err => {
        chrome.storage.local.get(['update_available', 'latest', 'current'], (cached) => {
          if (cached.latest) {
            sendResponse({ update_available: cached.update_available, latest: cached.latest, current: cached.current });
          } else {
            sendResponse({ ok: false, error: err.message });
          }
        });
      });
    return true;
  }
});

// ─── Periodic Update Check (every 6 hours) ────────────────────────────────
chrome.runtime.onInstalled.addListener(() => {
  chrome.alarms.create('plenumnet_update_check', { periodInMinutes: 360 });
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name !== 'plenumnet_update_check') return;
  const MANIFEST_URL =
    'https://raw.githubusercontent.com/SigmaWolf-8/Ternary/main/' +
    'services/tdns-v2/extension-chromium/manifest.json';
  const current = chrome.runtime.getManifest().version;
  fetch(MANIFEST_URL, { cache: 'no-store' })
    .then(r => r.json())
    .then(remote => {
      const latest = remote.version || current;
      chrome.storage.local.set({
        update_available: latest !== current,
        latest,
        current,
        checked: Date.now()
      });
    })
    .catch(() => {});
});

console.log('PlenumNET TDNS Resolver loaded — .plm bridge active');
