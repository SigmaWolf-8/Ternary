const PLM_API = "https://plenumnet.replit.app";

chrome.webRequest?.onBeforeRequest?.addListener(
  (details) => {
    const url = new URL(details.url);
    if (url.hostname.endsWith(".plm")) {
      const plmName = url.hostname.replace(/\.plm$/, "");
      return {
        redirectUrl: `${PLM_API}/api/tdns/resolve?name=${encodeURIComponent(plmName)}&redirect=1`
      };
    }
  },
  { urls: ["*://*.plm/*"] },
  ["blocking"]
);

chrome.omnibox?.onInputEntered?.addListener((text) => {
  if (text.startsWith("plm ")) {
    const name = text.slice(4).trim();
    chrome.tabs.update({
      url: `${PLM_API}/api/tdns/resolve?name=${encodeURIComponent(name)}&redirect=1`
    });
  }
});

chrome.runtime.onInstalled.addListener(() => {
  console.log("PlenumNET TDNS Resolver v2.3.2 installed");
  chrome.storage.local.set({ plm_api: PLM_API, version: "2.3.2" });
});
