document.addEventListener("click", function(e) {
  var link = e.target.closest("a");
  if (!link) return;
  var href = link.getAttribute("href") || "";
  var match = href.match(/^https?:\/\/([^/]+\.plm)(\/.*)?$/);
  if (!match) return;
  e.preventDefault();
  var name = match[1];
  var path = match[2] || "";
  chrome.runtime.sendMessage({ type: "resolve_plm", name: name }, function(resp) {
    if (resp && resp.target) {
      window.location.href = resp.target + path;
    } else {
      window.location.href = "https://plenumnet.replit.app/api/tdns/resolve?name=" + encodeURIComponent(name) + "&redirect=1";
    }
  });
}, true);
