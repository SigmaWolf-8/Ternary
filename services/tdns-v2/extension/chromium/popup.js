var searchInput = document.getElementById("search");
var resultDiv = document.getElementById("result");
var timeout = null;

searchInput.addEventListener("input", function() {
  clearTimeout(timeout);
  var val = searchInput.value.trim();
  if (!val) { resultDiv.className = "result"; return; }
  if (!val.endsWith(".plm")) val = val + ".plm";
  timeout = setTimeout(function() {
    fetch("https://plenumnet.replit.app/api/tdns/resolve?name=" + encodeURIComponent(val))
      .then(function(r) { return r.json(); })
      .then(function(data) {
        resultDiv.className = "result show";
        if (data.target) {
          resultDiv.innerHTML = '<div class="name">' + data.name + '</div><div class="target">' + data.target + '</div>';
        } else {
          resultDiv.innerHTML = '<div class="error">Not found: ' + val + '</div>';
        }
      })
      .catch(function() {
        resultDiv.className = "result show";
        resultDiv.innerHTML = '<div class="error">Connection failed</div>';
      });
  }, 300);
});

searchInput.addEventListener("keydown", function(e) {
  if (e.key === "Enter") {
    var val = searchInput.value.trim();
    if (!val) return;
    if (!val.endsWith(".plm")) val = val + ".plm";
    window.open("https://plenumnet.replit.app/api/tdns/resolve?name=" + encodeURIComponent(val) + "&redirect=1", "_blank");
  }
});
