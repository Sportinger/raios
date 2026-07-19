// ---------- WiFi-Flow: exakte Geometrie aus wifi_flow.rs (zentrierte Fenster, kein Scrim) ----------

const LIST_LIMIT = 8;
const LIST_ROW_H = 28;

function centeredRect(w, h) {
  w = Math.min(w, L.W - 24);
  h = Math.min(h, L.H - 44);
  return { x: Math.floor((L.W - w) / 2), y: Math.floor((L.H - h) / 2), w, h };
}

function blockBackground() {
  hits.push({ ...L.personal, action: () => {} });
}

function startWifi() {
  state.view = "wifi-starting";
  state.wifiProgress = 0;
  const tick = () => {
    if (state.view !== "wifi-starting") return;
    state.wifiProgress += 18;
    if (state.wifiProgress >= 100) { state.view = "wifi-list"; render(); return; }
    render();
    setTimeout(tick, 130);
  };
  render();
  setTimeout(tick, 130);
}

function drawProgressWindow(title, label) {
  blockBackground();
  const r = centeredRect(340, 126);
  drawPanel(r, title);
  drawTruncated(r.x + 20, r.y + 52, label, Math.floor((r.w - 40) / ADV), "TEXT_MAIN");
  fillRect(r.x + 20, r.y + 78, r.w - 40, 12, "SURFACE_ALT");
  fillRect(r.x + 20, r.y + 78, Math.floor((r.w - 40) * Math.min(100, state.wifiProgress) / 100), 12, "APP_BLUE");
  drawOutline({ x: r.x + 20, y: r.y + 78, w: r.w - 40, h: 12 }, "HAIRLINE");
}

function drawWifiStarting() { drawProgressWindow("Starting WiFi", "Loading firmware and scanning"); }
function drawWifiConnecting() { drawProgressWindow("Connecting WiFi", "Associating and requesting address"); }

function drawWifiList() {
  blockBackground();
  const count = Math.min(state.wifiNetworks.length, LIST_LIMIT);
  const r = centeredRect(360, 112 + count * LIST_ROW_H);
  drawPanel(r, "WiFi networks");
  if (count === 0) drawText(r.x + 20, r.y + 56, "No networks found", "TEXT_MUTED");
  state.wifiNetworks.slice(0, LIST_LIMIT).forEach((n, i) => {
    const row = { x: r.x + 18, y: r.y + 48 + i * LIST_ROW_H, w: r.w - 36, h: 22 };
    fillRect(row.x, row.y, row.w, row.h, "SURFACE_ALT");
    drawOutline(row, "HAIRLINE");
    const line = n.ssid + "  CH" + n.ch + "  " + n.sec + "  " + n.tag;
    drawTruncated(row.x + 8, row.y + 7, line, Math.floor((row.w - 16) / ADV), "TEXT_MAIN");
    hits.push({ ...row, action: () => {
      state.wifiSelected = i;
      state.wifiPassword = "";
      state.wifiRejected = false;
      state.view = "wifi-password";
      render();
    }});
  });
  const bw = Math.floor((r.w - 50) / 2);
  const by = r.y + r.h - 32;
  drawButton({ x: r.x + 16, y: by, w: bw, h: 20 }, "Scan again", false, "scan-again");
  drawButton({ x: r.x + 34 + bw, y: by, w: bw, h: 20 }, "Close", false, "close-wifi", closeOverlay);
}

function drawWifiPassword() {
  blockBackground();
  const r = centeredRect(360, 190);
  drawPanel(r, "WiFi password");
  const n = state.wifiNetworks[state.wifiSelected] || { ssid: "" };
  drawTruncated(r.x + 20, r.y + 48, n.ssid, Math.floor((r.w - 40) / ADV), "TEXT_MAIN");
  const field = { x: r.x + 20, y: r.y + 66, w: r.w - 40, h: 28 };
  fillRect(field.x, field.y, field.w, field.h, "SURFACE_ALT");
  drawOutline(field, "APP_BLUE");
  const visible = Math.min(state.wifiPassword.length, Math.floor((r.w - 56) / ADV));
  for (let i = 0; i < visible; i++) drawText(r.x + 28 + i * ADV, r.y + 76, "*", "TEXT_MAIN");
  state._pwCursor = { x: r.x + 28 + visible * ADV, y: r.y + 73 };
  drawTruncated(
    r.x + 20,
    r.y + 106,
    state.wifiRejected ? "Password must contain 8-63 printable characters" : "8-63 printable characters",
    Math.floor((r.w - 40) / ADV),
    state.wifiRejected ? "APP_RED" : "TEXT_FAINT",
  );
  const storageLine = state.wifiStorage === "vault"
    ? "Will be encrypted for this exact access point"
    : state.wifiRemember
      ? "[x] Remember for this boot (LEGACY RAM-ONLY)"
      : "[ ] Remember for this boot (LEGACY RAM-ONLY)";
  drawTruncated(r.x + 20, r.y + 120, storageLine, Math.floor((r.w - 40) / ADV), "TEXT_MUTED");
  const bw = Math.floor((r.w - 50) / 2);
  const by = r.y + r.h - 32;
  drawButton({ x: r.x + 16, y: by, w: bw, h: 20 }, "Back", false, "wifi-back", () => {
    state.view = "wifi-list";
    render();
  });
  drawButton(
    { x: r.x + 34 + bw, y: by, w: bw, h: 20 },
    state.wifiStorage === "vault" ? "Save and connect" : "Set credentials",
    true,
    "wifi-save",
    submitWifiPassword,
  );
}

function drawWifiConfigured() {
  blockBackground();
  const rect = centeredRect(340, 152);
  const network = state.wifiNetworks[state.wifiSelected] || { ssid: "home-fiber-5G" };
  drawPanel(rect, "WiFi setup");
  drawTruncated(rect.x + 20, rect.y + 50, network.ssid, Math.floor((rect.w - 40) / ADV), "APP_GREEN");
  const storage = state.wifiStorage === "vault"
    ? "Credential saved in Secret Vault"
    : state.wifiStorage === "open"
      ? "Open network selected for this boot"
      : "Credential ready (LEGACY RAM-ONLY)";
  drawText(rect.x + 20, rect.y + 70, storage, "TEXT_MAIN");
  drawText(rect.x + 20, rect.y + 86, state.wifiConnected ? "Connected" : "Link ready - requesting network address", "APP_AMBER");
  const bw = Math.floor((rect.w - 50) / 2);
  const by = rect.y + rect.h - 32;
  drawButton({ x: rect.x + 34 + bw, y: by, w: bw, h: 20 }, "Done", true, "wifi-done", closeOverlay);
}

function drawWifiFailed() {
  blockBackground();
  const rect = centeredRect(340, 152);
  drawPanel(rect, "WiFi unavailable");
  drawTruncated(rect.x + 20, rect.y + 54, state.wifiFailureReason, Math.floor((rect.w - 40) / ADV), "APP_RED");
  drawText(rect.x + 20, rect.y + 76, "No network state was granted", "TEXT_MUTED");
  const bw = Math.floor((rect.w - 50) / 2);
  const by = rect.y + rect.h - 32;
  drawButton({ x: rect.x + 16, y: by, w: bw, h: 20 }, "Retry", false, "wifi-retry", startWifi);
  drawButton({ x: rect.x + 34 + bw, y: by, w: bw, h: 20 }, "Close", false, "wifi-close", closeOverlay);
}

function submitWifiPassword() {
  if (state.wifiPassword.length < 8) {
    state.wifiRejected = true;
    render();
    return;
  }
  state.view = "wifi-connecting";
  state.wifiProgress = 0;
  const tick = () => {
    if (state.view !== "wifi-connecting") return;
    state.wifiProgress += 15;
    if (state.wifiProgress >= 100) {
      state.wifiConnected = true;
      const ssid = (state.wifiNetworks[state.wifiSelected] || {}).ssid || "";
      closeOverlay();
      pushChat("system", "WiFi connected: " + ssid);
      return;
    }
    render();
    setTimeout(tick, 140);
  };
  render();
  setTimeout(tick, 140);
}

// Ctrl+Shift+E: Design-Delta als JSON in die Zwischenablage (Ruekkanal-Protokoll)
