// PlenumNET TDNS — Background Service Worker
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. — Applied Physics Division
// Phase 1 — v1.0.0

const API_BASE = "https://plenumnet.replit.app";

// Cache last scan result per tab for rescan comparison
const scanCache = new Map();

// On install: set defaults
chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.set({
    tier:        "free",
    scanCount:   0,
    rescanCount: {},   // { [date]: { [hostname]: count } }
    apiBase:     API_BASE,
  });
  console.log("[PlenumNET] Extension installed — TDNS Scanner v1.0.0");
});

// Message handler — popup and content script communicate via here
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === "GET_TAB_URL") {
    chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
      const tab = tabs[0];
      sendResponse({ url: tab?.url || "", tabId: tab?.id });
    });
    return true; // keep channel open for async
  }

  if (msg.type === "CACHE_SCAN") {
    scanCache.set(msg.tabId, msg.result);
    sendResponse({ ok: true });
    return true;
  }

  if (msg.type === "GET_CACHED_SCAN") {
    sendResponse({ result: scanCache.get(msg.tabId) || null });
    return true;
  }

  if (msg.type === "CLEAR_CACHE") {
    scanCache.delete(msg.tabId);
    sendResponse({ ok: true });
    return true;
  }

  // Phase 2: dynamic tracker flags from content script
  if (msg.type === "DYNAMIC_TRACKER_FLAGS") {
    // { tabId, hostname, flags: { analytics, social, advertising, session_replay, crm } }
    // Merge into cached scan result if present
    const cached = scanCache.get(msg.tabId);
    if (cached && cached.meta?.hostname === msg.hostname) {
      // Update tracker categories with dynamic detection
      if (cached.trackers) {
        Object.entries(msg.flags).forEach(([id, detected]) => {
          const cat = cached.trackers.find(t => t.id === id);
          if (cat && detected) cat.detected_dynamic = true;
        });
      }
      scanCache.set(msg.tabId, cached);
    }
    sendResponse({ ok: true });
    return true;
  }
});

// Track tab updates to invalidate stale cache
chrome.tabs.onUpdated.addListener((tabId, changeInfo) => {
  if (changeInfo.status === "loading") {
    scanCache.delete(tabId);
  }
});

chrome.tabs.onRemoved.addListener(tabId => {
  scanCache.delete(tabId);
});
