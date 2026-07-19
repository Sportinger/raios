function openOverlay(view) {
  state.view = view;
  state.composerFocus = false;
  state.overlayAnim = 1;
  render();
}

function closeOverlay() {
  state.view = "genesis";
  state.wifiSelected = null;
  state.wifiPassword = "";
  state.wifiRejected = false;
  render();
}

function overlayRect() {
  const t = state.overlayAnim;
  const o = L.overlay;
  const inset = Math.round((1 - t) * 14);
  return { x: o.x + inset, y: o.y + inset, w: o.w - 2 * inset, h: o.h - 2 * inset };
}

function drawScrim() {
  fillRect(L.personal.x, L.personal.y, L.personal.w, L.personal.h, "SCRIM");
  hits.push({ ...L.personal, action: closeOverlay });
}

function drawSetupOverlay() {
  drawScrim();
  const r = overlayRect();
  drawPanel(r, "Trusted setup");
  if (state.overlayAnim < 1) return;
  drawText(r.x + 20, r.y + 42, "Existing provider and WiFi setup", "TEXT_MUTED");
  drawText(r.x + 20, r.y + 70, "API key: unlock Secret Vault before saving", "TEXT_MAIN");
  drawButton({ x: r.x + 20, y: r.y + 104, w: r.w - 40, h: 24 }, "Secret Vault unavailable", false, "vault");
  const kbd = state.keyboardLayout === "US" ? "Keyboard: US / switch to DE" : "Keyboard: DE / switch to US";
  drawButton({ x: r.x + 20, y: r.y + 136, w: r.w - 40, h: 24 }, kbd, false, "kbd", () => {
    state.keyboardLayout = state.keyboardLayout === "US" ? "DE" : "US";
  });
  const bw = Math.floor((r.w - 52) / 2);
  const left = r.x + 20, right = left + bw + 12, first = r.y + r.h - 82;
  drawButton({ x: left,  y: first,      w: bw, h: 24 }, "Unlock Vault first", false, "api");
  drawButton({ x: right, y: first,      w: bw, h: 24 }, "Set WiFi", false, "set-wifi", startWifi);
  drawButton({ x: left,  y: first + 32, w: bw, h: 24 }, "Scan WiFi", false, "scan-wifi", startWifi);
  drawButton({ x: right, y: first + 32, w: bw, h: 24 }, "Close", false, "close-setup", closeOverlay);
}
