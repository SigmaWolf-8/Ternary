const PLM_API = "https://plenumnet.replit.app";

chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.set({ plm_api: PLM_API, version: "2.3.2" });
});

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === "resolve_plm") {
    fetch(PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(msg.name))
      .then(r => r.json())
      .then(data => sendResponse(data))
      .catch(err => sendResponse({ error: err.message }));
    return true;
  }
});
