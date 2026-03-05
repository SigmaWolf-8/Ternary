var PLM_API = "https://plenumnet.replit.app";

chrome.runtime.onInstalled.addListener(function() {
  chrome.storage.local.set({ plm_api: PLM_API, version: "2.3.4" });
});

chrome.webNavigation.onBeforeNavigate.addListener(function(details) {
  if (details.frameId !== 0) return;

  var url;
  try { url = new URL(details.url); } catch(e) { return; }

  if (url.hostname.endsWith(".plm")) {
    chrome.tabs.update(details.tabId, {
      url: PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(url.hostname) + "&redirect=1"
    });
    return;
  }

  var isSearch = url.hostname.includes("bing.com") ||
                 url.hostname.includes("google.com") ||
                 url.hostname.includes("duckduckgo.com") ||
                 url.hostname.includes("yahoo.com") ||
                 url.hostname.includes("search.brave.com");

  if (isSearch) {
    var q = url.searchParams.get("q") || url.searchParams.get("p") || "";
    q = q.trim();
    if (/^[a-z0-9._-]+\.plm$/i.test(q)) {
      chrome.tabs.update(details.tabId, {
        url: PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(q) + "&redirect=1"
      });
    }
  }
});

chrome.omnibox.onInputEntered.addListener(function(text) {
  var name = text.trim();
  if (!name.endsWith(".plm")) {
    name = name + ".plm";
  }
  chrome.tabs.update({
    url: PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(name) + "&redirect=1"
  });
});

chrome.omnibox.onInputChanged.addListener(function(text, suggest) {
  var name = text.trim();
  if (!name.endsWith(".plm")) {
    name = name + ".plm";
  }
  fetch(PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(name))
    .then(function(r) { return r.json(); })
    .then(function(data) {
      if (data.resolved) {
        suggest([{
          content: name,
          description: name + " -> " + data.target
        }]);
      }
    })
    .catch(function() {});
});

chrome.runtime.onMessage.addListener(function(msg, sender, sendResponse) {
  if (msg.type === "resolve_plm") {
    fetch(PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(msg.name))
      .then(function(r) { return r.json(); })
      .then(function(data) { sendResponse(data); })
      .catch(function(err) { sendResponse({ error: err.message }); });
    return true;
  }
});
