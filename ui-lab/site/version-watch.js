// Keep already-open production tabs on the latest deployed website build.
// Source checkouts use the "development" marker and deliberately stay inert.
(() => {
  const VERSION_META_NAME = "raios-build-version";
  const VERSION_ENDPOINT = "version.json";
  const RELOAD_QUERY = "_raios_build";
  const INITIAL_CHECK_DELAY_MS = 2500;
  const CHECK_INTERVAL_MS = 15000;
  const REQUEST_TIMEOUT_MS = 8000;

  const versionMeta = document.querySelector(`meta[name="${VERSION_META_NAME}"]`);
  const currentVersion = versionMeta ? versionMeta.content.trim() : "";
  const hosted = location.protocol === "http:" || location.protocol === "https:";
  if (!hosted || !currentVersion || currentVersion === "development") return;

  const cleanReloadMarker = () => {
    const cleanUrl = new URL(location.href);
    if (!cleanUrl.searchParams.has(RELOAD_QUERY)) return;
    cleanUrl.searchParams.delete(RELOAD_QUERY);
    history.replaceState(history.state, "", cleanUrl);
  };

  let checking = false;
  let reloading = false;

  const reloadWithVersion = (nextVersion) => {
    reloading = true;
    const reloadUrl = new URL(location.href);
    reloadUrl.searchParams.set(RELOAD_QUERY, nextVersion);
    location.replace(reloadUrl);
  };

  const checkForUpdate = async () => {
    if (checking || reloading || document.hidden) return;
    checking = true;
    const controller = new AbortController();
    const timeout = window.setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
    try {
      const endpoint = new URL(VERSION_ENDPOINT, document.baseURI);
      endpoint.searchParams.set("t", Date.now().toString(36));
      const response = await fetch(endpoint, {
        cache: "no-store",
        credentials: "same-origin",
        headers: { "Cache-Control": "no-cache" },
        signal: controller.signal,
      });
      if (!response.ok) return;
      const payload = await response.json();
      const nextVersion = typeof payload.version === "string" ? payload.version.trim() : "";
      if (nextVersion && nextVersion !== currentVersion) reloadWithVersion(nextVersion);
    } catch {
      // Being offline or switching networks must never interrupt the website.
    } finally {
      window.clearTimeout(timeout);
      checking = false;
    }
  };

  cleanReloadMarker();
  window.__RAIOS_BUILD_VERSION__ = currentVersion;
  window.setTimeout(checkForUpdate, INITIAL_CHECK_DELAY_MS);
  window.setInterval(checkForUpdate, CHECK_INTERVAL_MS);
  window.addEventListener("focus", checkForUpdate, { passive: true });
  window.addEventListener("pageshow", checkForUpdate, { passive: true });
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) checkForUpdate();
  });
})();
