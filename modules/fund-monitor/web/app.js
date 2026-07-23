document.documentElement.dataset.ready = "true";
console.log("fund-monitor ready");

setupFundDetailAutoRefresh();

function setupFundDetailAutoRefresh() {
  var targetId = "fund-detail-snapshot";
  var target = document.getElementById(targetId);

  if (!target) {
    return;
  }

  var refreshUrl = target.dataset.autoRefreshUrl;
  var intervalMs = Number(target.dataset.autoRefreshIntervalMs || "0");
  if (!refreshUrl || !Number.isFinite(intervalMs) || intervalMs <= 0) {
    return;
  }

  var isRefreshing = false;

  async function refreshSnapshot() {
    if (isRefreshing || document.hidden) {
      return;
    }

    var current = document.getElementById(targetId);
    if (!current) {
      return;
    }

    isRefreshing = true;

    try {
      var response = await fetch(refreshUrl, {
        headers: {
          "X-Requested-With": "fund-monitor-auto-refresh",
        },
      });
      if (!response.ok) {
        return;
      }

      var html = await response.text();
      var container = document.createElement("div");
      container.innerHTML = html.trim();

      var replacement = container.firstElementChild;
      if (!replacement) {
        return;
      }

      current.replaceWith(replacement);
    } catch (error) {
      console.error("fund-monitor snapshot auto refresh failed", error);
    } finally {
      isRefreshing = false;
    }
  }

  window.setInterval(refreshSnapshot, intervalMs);
}
