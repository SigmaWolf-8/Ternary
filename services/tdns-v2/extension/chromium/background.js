var PROD_API = "https://plenumnet.replit.app";
var DEV_API = "";
var PLM_API = PROD_API;

function tryResolve(name, callback) {
  var urls = [PROD_API];
  if (DEV_API) urls.unshift(DEV_API);

  function attempt(i) {
    if (i >= urls.length) { callback(null); return; }
    fetch(urls[i] + "/api/tdns/resolve?name=" + encodeURIComponent(name))
      .then(function(r) { return r.json(); })
      .then(function(data) {
        if (data.resolved) {
          PLM_API = urls[i];
          callback(data);
        } else {
          callback(data);
        }
      })
      .catch(function() { attempt(i + 1); });
  }
  attempt(0);
}

var TDNS_SERVER = "http://localhost:3927";

chrome.runtime.onInstalled.addListener(function() {
  chrome.storage.local.set({ version: "2.3.3" });
  chrome.action.setBadgeText({ text: "T" });
  chrome.action.setBadgeBackgroundColor({ color: "#d4a017" });
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
  if (!name.endsWith(".plm")) name = name + ".plm";
  chrome.tabs.update({
    url: PLM_API + "/api/tdns/resolve?name=" + encodeURIComponent(name) + "&redirect=1"
  });
});

chrome.omnibox.onInputChanged.addListener(function(text, suggest) {
  var name = text.trim();
  if (!name.endsWith(".plm")) name = name + ".plm";
  tryResolve(name, function(data) {
    if (data && data.resolved) {
      suggest([{ content: name, description: name + " -> " + data.target }]);
    }
  });
});

chrome.runtime.onMessage.addListener(function(msg, sender, sendResponse) {
  if (msg.type === "resolve_plm") {
    tryResolve(msg.name, function(data) {
      sendResponse(data || { error: "Resolution failed" });
    });
    return true;
  }
  if (msg.type === "set_dev_api") {
    DEV_API = msg.url;
    PLM_API = msg.url;
    sendResponse({ ok: true });
    return true;
  }
  if (msg.type === "health") {
    fetch(TDNS_SERVER + "/api/v1/health")
      .then(function(r) { return r.json(); })
      .then(function(data) { sendResponse({ ok: true, data: data }); })
      .catch(function() { sendResponse({ ok: false }); });
    return true;
  }
  if (msg.type === "resolve") {
    fetch(TDNS_SERVER + "/api/v1/resolve/" + encodeURIComponent(msg.name))
      .then(function(r) { return r.json(); })
      .then(function(data) { sendResponse({ ok: true, data: data }); })
      .catch(function() { sendResponse({ ok: false }); });
    return true;
  }
  if (msg.type === "scan") {
    fetch(TDNS_SERVER + "/api/v1/scan", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url: msg.url })
    })
      .then(function(r) { return r.json(); })
      .then(function(data) { sendResponse({ ok: true, data: data }); })
      .catch(function() { sendResponse({ ok: false }); });
    return true;
  }
});
