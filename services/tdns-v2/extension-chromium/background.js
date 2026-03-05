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
      // Try scanning instead
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

// Catch direct navigation to .plm domains (e.g., http://google.plm)
chrome.webNavigation.onErrorOccurred.addListener((details) => {
  if (details.frameId !== 0) return; // Only main frame

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

// Also catch successful navigation attempts to .plm (in case OS resolves it somehow)
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
    return true; // async
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
});

console.log('PlenumNET TDNS Resolver loaded — .plm bridge active');
