// ---------- Render ----------

// Teurer Pfad: komplette Szene in den Back-Buffer (Kernel: render_inner).
function renderScene() {
  const started = performance.now();
  hits = [];
  target = bctx;
  if (state.view === "personal") {
    drawPersonalSurface();
  } else {
    if (state.dream) {
      drawDream();
      if (state.recoveryOpen) {
        drawPanel(L.context, "Recovery context");
        drawRecoveryContext(L.context);
      }
    } else {
      fillRect(0, 0, L.W, L.H, "APP_BG");
      drawSecureStrip();
      drawConversation();
      drawContext();
      drawComposer();
    }
    if (state.view === "setup") drawSetupOverlay();
    else if (state.view === "wifi-starting") drawWifiStarting();
    else if (state.view === "wifi-list") drawWifiList();
    else if (state.view === "wifi-password") drawWifiPassword();
    else if (state.view === "wifi-connecting") drawWifiConnecting();
    else if (state.view === "wifi-configured") drawWifiConfigured();
    else if (state.view === "wifi-failed") drawWifiFailed();
    else if (state.view === "vault") drawVaultFlow();
  }
  labMetrics.sceneFrames += 1;
  labMetrics.lastSceneMs = performance.now() - started;
}

// Billiger Pfad: Back-Buffer blitten + Front-Pixel (Kernel: set_front_pixel /
// restore_from_back_rect) — Cursorblinken zeichnet die Szene NICHT neu.
function present() {
  const started = performance.now();
  ctx.drawImage(back, 0, 0);
  target = ctx;
  if (state.view === "genesis" && state.composerFocus && state.cursorOn && state._composerCursor) {
    fillRect(state._composerCursor.x, state._composerCursor.y, 2, 15, "APP_BLUE");
  }
  if (state.view === "wifi-password" && state.cursorOn && state._pwCursor) {
    fillRect(state._pwCursor.x, state._pwCursor.y, 2, 15, "APP_BLUE");
  }
  if (state.labMode === "proposal") drawProposalOverlay();
  // raiOS-Mauszeiger: Front-Ebene, Fuellung blau bei gedruecktem Knopf
  if (state.pointer) {
    const fill = state.pointerDown ? PAL.APP_BLUE : PAL.TEXT_MAIN;
    for (let row = 0; row < POINTER_SHAPE.length; row++) {
      const pattern = POINTER_SHAPE[row];
      for (let col = 0; col < pattern.length; col++) {
        const px = pattern[col];
        if (px !== "X" && px !== "O") continue;
        ctx.fillStyle = px === "X" ? POINTER_OUTLINE : fill;
        ctx.fillRect(state.pointer.x + col * SCALE, state.pointer.y + row * SCALE, SCALE, SCALE);
      }
    }
  }
  target = bctx;
  labMetrics.presents += 1;
  labMetrics.lastPresentMs = performance.now() - started;
  updateLabHud();
}

function render() {
  renderScene();
  present();
}

function toLogical(ev) {
  const r = cv.getBoundingClientRect();
  return {
    x: Math.floor((ev.clientX - r.left) / r.width * L.W),
    y: Math.floor((ev.clientY - r.top) / r.height * L.H),
  };
}

function hitAt(p) {
  for (let i = hits.length - 1; i >= 0; i--) {
    const h = hits[i];
    if (p.x >= h.x && p.x < h.x + h.w && p.y >= h.y && p.y < h.y + h.h) return h;
  }
  return null;
}

cv.addEventListener("click", (ev) => {
  cv.focus();
  const h = hitAt(toLogical(ev));
  const wasFocus = state.composerFocus;
  if (h) {
    h.action();
  } else if (wasFocus) {
    state.composerFocus = false;
    render();
  }
});

function toPhysical(ev) {
  const r = cv.getBoundingClientRect();
  return {
    x: Math.floor((ev.clientX - r.left) / r.width * cv.width),
    y: Math.floor((ev.clientY - r.top) / r.height * cv.height),
  };
}

function pointerAtViewportPoint(clientX, clientY, rect, physicalWidth, physicalHeight) {
  if (
    clientX < rect.left || clientX >= rect.right
      || clientY < rect.top || clientY >= rect.bottom
      || rect.width <= 0 || rect.height <= 0
  ) return null;
  return {
    x: Math.floor((clientX - rect.left) / rect.width * physicalWidth),
    y: Math.floor((clientY - rect.top) / rect.height * physicalHeight),
  };
}

let viewportPointer = null;
let pointerScrollFrame = 0;

function syncPointerFromViewport() {
  if (!viewportPointer || document.body.dataset.shellMode !== "website") return false;
  const next = pointerAtViewportPoint(
    viewportPointer.x,
    viewportPointer.y,
    cv.getBoundingClientRect(),
    cv.width,
    cv.height,
  );
  const changed = !next || !state.pointer
    ? next !== state.pointer
    : next.x !== state.pointer.x || next.y !== state.pointer.y;
  state.pointer = next;
  if (changed) present();
  return changed;
}

function schedulePointerSync() {
  cancelAnimationFrame(pointerScrollFrame);
  pointerScrollFrame = requestAnimationFrame(syncPointerFromViewport);
}

// Zeigerbewegung = nur Front-Ebene neu (billiger Pfad, wie im Kernel)
cv.addEventListener("mousemove", (ev) => {
  viewportPointer = { x: ev.clientX, y: ev.clientY };
  state.pointer = toPhysical(ev);
  if (state.dream) {
    // Reines Hover-Modell: Mitte beruehren = aufziehen, sonst geschlossen.
    // Die Zone folgt dem interpolierten Layout (Hysterese durch +-20).
    const p = toLogical(ev);
    const lay = dreamLayout();
    const inC = p.x >= lay.center.x - 20 && p.x <= lay.center.x + lay.center.w + 20 && p.y >= lay.bracketTop && p.y <= lay.bracketBottom;
    const goal = inC ? 1 : -1;
    state.hoverCenter = inC;
    if (goal !== state.layoutGoal) animateLayout(goal);
    // Button-Hover: weicher Fade beim Betreten, sofortiges Zuruecksetzen beim Verlassen
    const hh = hitAt(p);
    const hid = hh && hh.id ? hh.id : null;
    if (hid !== state.hoverId) {
      state.hoverId = hid;
      if (hid) animateHover();
      else { state.hoverT = 0; render(); }
    }
  }
  present();
});
cv.addEventListener("mouseleave", (ev) => {
  viewportPointer = { x: ev.clientX, y: ev.clientY };
  state.pointer = null;
  if (state.dream && state.layoutGoal !== -1) {
    state.hoverCenter = false;
    animateLayout(-1);
  }
  present();
});
cv.addEventListener("mousedown", () => { state.pointerDown = true; present(); });
cv.addEventListener("mouseup", () => { state.pointerDown = false; present(); });

window.addEventListener("scroll", schedulePointerSync, { passive: true });

function chatWheelResult(currentScroll, deltaY, maxScroll) {
  const boundedCurrent = Math.max(0, Math.min(maxScroll, currentScroll));
  const step = deltaY < 0 ? 2 : deltaY > 0 ? -2 : 0;
  const next = Math.max(0, Math.min(maxScroll, boundedCurrent + step));
  return { next, consumed: next !== boundedCurrent };
}

function genesisChatScrollMax() {
  const r = L.conversation;
  const maxChars = Math.max(1, Math.floor((r.w - 48) / ADV));
  const visible = Math.max(1, Math.floor((r.h - 54) / ROW_H));
  return Math.max(0, conversationRows(maxChars).length - visible);
}

function dreamChatScrollMax(layout) {
  const maxHi = Math.max(1, Math.floor((layout.center.w - 10) / 4.5));
  const conversationBottom = Math.max(
    DREAM_TOKENS.conversationTop,
    layout.composerY - DREAM_TOKENS.conversationComposerGap,
  );
  const visible = Math.max(
    1,
    Math.floor((conversationBottom - DREAM_TOKENS.conversationTop) / ROW_H),
  );
  return Math.max(0, conversationRows(maxHi).length - visible);
}

cv.addEventListener("wheel", (ev) => {
  const p = toLogical(ev);
  if (state.dream) {
    const layout = dreamLayout();
    const conversationBottom = Math.max(DREAM_TOKENS.conversationTop, layout.composerY - DREAM_TOKENS.conversationComposerGap);
    if (state.layoutT > -0.5 && state.dreamTab === "chat" && p.x >= layout.center.x && p.x < layout.center.x + layout.center.w && p.y >= DREAM_TOKENS.conversationTop && p.y < conversationBottom) {
      const result = chatWheelResult(state.chatScroll, ev.deltaY, dreamChatScrollMax(layout));
      if (result.consumed) {
        ev.preventDefault();
        state.chatScroll = result.next;
        render();
      }
    }
    return;
  }
  const r = L.conversation;
  if (state.view === "genesis" && p.x >= r.x && p.x < r.x + r.w && p.y >= r.y && p.y < r.y + r.h) {
    const result = chatWheelResult(state.chatScroll, ev.deltaY, genesisChatScrollMax());
    if (result.consumed) {
      ev.preventDefault();
      state.chatScroll = result.next;
      render();
    }
  }
}, { passive: false });

cv.addEventListener("keydown", (ev) => {
  if (ev.ctrlKey && ev.shiftKey && ev.key.toLowerCase() === "m") {
    state.labMode = state.labMode === "actual" ? "proposal" : "actual";
    state.labDiagnostics = true;
    render();
    ev.preventDefault();
    return;
  }
  if (ev.ctrlKey && ev.shiftKey && ev.key.toLowerCase() === "d") {
    state.labDiagnostics = !state.labDiagnostics;
    updateLabHud();
    ev.preventDefault();
    return;
  }
  if (ev.ctrlKey && ev.shiftKey && ev.key.toLowerCase() === "e") {
    exportDesignDelta();
    ev.preventDefault();
    return;
  }
  if (state.view === "personal" && handlePersonalKey(ev)) {
    ev.preventDefault();
    return;
  }
  if (state.dream && ev.key === "Escape" && state.layoutGoal !== -1) {
    state.hoverCenter = false;
    animateLayout(-1);
    ev.preventDefault();
    return;
  }
  if (state.view === "wifi-password") {
    if (ev.key === "Backspace") state.wifiPassword = state.wifiPassword.slice(0, -1);
    else if (ev.key === "Escape") { state.view = "wifi-list"; }
    else if (ev.key === "Enter") { submitWifiPassword(); ev.preventDefault(); return; }
    else if (ev.key.length === 1 && state.wifiPassword.length < 63) state.wifiPassword += ev.key;
    else return;
    ev.preventDefault();
    render();
    return;
  }
  if (state.view !== "genesis" && ev.key === "Escape") { closeOverlay(); return; }
  if (!state.composerFocus) return;
  if (ev.key === "Backspace") {
    state.composerText = state.composerText.slice(0, -1);
  } else if (ev.key === "Escape") {
    state.composerFocus = false;
  } else if (ev.key === "Enter") {
    flash("send");
    submitChat();
    return;
  } else if (ev.key.length === 1) {
    state.composerText += ev.key;
  } else {
    return;
  }
  ev.preventDefault();
  render();
});

setInterval(() => {
  state.cursorOn = !state.cursorOn;
  // Blinken = nur Front-Ebene, Szene bleibt unberuehrt (billig)
  if (state.composerFocus || state.view === "wifi-password") present();
  // Sim: Doppel-Bau Lauf B tickt (Szene-Redraw = ehrlich teuer)
  if (state.dream && !state.approved && state.buildB < 100) {
    state.buildB = Math.min(100, state.buildB + 6);
    render();
  }
}, 530);

function runLabSelftests() {
  const failures = [];
  const expect = (name, actual, expected) => {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      failures.push(`${name}: ${JSON.stringify(actual)} != ${JSON.stringify(expected)}`);
    }
  };
  const legacy = computeGenesisLayout(1280, 800);
  expect("genesis-1280-conversation", legacy.conversation, { x: 16, y: 54, w: 397, h: 270 });
  expect("genesis-1280-context", legacy.context, { x: 425, y: 54, w: 199, h: 270 });
  expect("genesis-1280-composer", legacy.composer, { x: 16, y: 336, w: 608, h: 48 });
  const fullHd = computeGenesisLayout(1920, 1080);
  expect("genesis-1080-logical", [fullHd.W, fullHd.H], [960, 540]);
  expect("genesis-1080-conversation", fullHd.conversation, { x: 16, y: 54, w: 610, h: 410 });
  expect("genesis-1080-context", fullHd.context, { x: 638, y: 54, w: 306, h: 410 });
  expect("genesis-1080-overlay", fullHd.overlay, { x: 240, y: 149, w: 480, h: 280 });
  const oldDream = dreamKeyframes(1280, 800);
  expect("dream-1280-closed", oldDream.closed.center, { x: 316, w: 8 });
  expect("dream-1280-ambient", oldDream.ambient.center, { x: 185, w: 270 });
  expect("dream-1280-open", oldDream.open.center, { x: 177, w: 286 });
  const hdDream = dreamKeyframes(1920, 1080);
  expect("dream-1080-closed", hdDream.closed.center, { x: 476, w: 8 });
  expect("dream-1080-ambient", hdDream.ambient.center, { x: 278, w: 405 });
  expect("dream-1080-open", hdDream.open.center, { x: 266, w: 429 });
  expect("dream-1080-bounds", [hdDream.open.bracketTop, hdDream.open.bracketBottom, hdDream.open.composerY], [40, 522, 496]);
  expect(
    "canvas-native-size-cap",
    cv.width <= PHYSICAL_WIDTH && cv.height <= PHYSICAL_HEIGHT,
    true,
  );
  expect("renderer-resolution-small-boundary", rendererResolutionForDisplay(500), { width: 1280, height: 720 });
  expect("renderer-resolution-native", rendererResolutionForDisplay(1600), { width: 1600, height: 900 });
  expect("renderer-resolution-1080p-cap", rendererResolutionForDisplay(3000), { width: 1920, height: 1080 });
  expect("chat-wheel-no-overflow-passes-page", chatWheelResult(0, -100, 0), { next: 0, consumed: false });
  expect("chat-wheel-bottom-passes-page", chatWheelResult(0, 100, 8), { next: 0, consumed: false });
  expect("chat-wheel-scrolls-when-available", chatWheelResult(0, -100, 8), { next: 2, consumed: true });
  expect("chat-wheel-top-passes-page", chatWheelResult(8, -100, 8), { next: 8, consumed: false });
  const wheelState = {
    dream: state.dream,
    view: state.view,
    layoutT: state.layoutT,
    layoutGoal: state.layoutGoal,
    dreamTab: state.dreamTab,
    chat: state.chat,
    chatScroll: state.chatScroll,
  };
  state.dream = true;
  state.view = "genesis";
  state.layoutT = 1;
  state.layoutGoal = 1;
  state.dreamTab = "chat";
  state.chat = [];
  state.chatScroll = 0;
  const wheelLayout = dreamLayout();
  const wheelRect = cv.getBoundingClientRect();
  const wheelPoint = {
    x: wheelRect.left + (wheelLayout.center.x + wheelLayout.center.w / 2) / L.W * wheelRect.width,
    y: wheelRect.top + (DREAM_TOKENS.conversationTop + 2) / L.H * wheelRect.height,
  };
  const emptyChatWheel = new WheelEvent("wheel", {
    clientX: wheelPoint.x,
    clientY: wheelPoint.y,
    deltaY: -100,
    bubbles: true,
    cancelable: true,
  });
  cv.dispatchEvent(emptyChatWheel);
  expect("chat-wheel-empty-event-passes-page", emptyChatWheel.defaultPrevented, false);
  state.chat = Array.from({ length: 30 }, (_, index) => ({
    speaker: index % 2 === 0 ? "user" : "assistant",
    text: `Overflow row ${index}: this message proves that the embedded conversation can really scroll.`,
  }));
  const overflowingChatWheel = new WheelEvent("wheel", {
    clientX: wheelPoint.x,
    clientY: wheelPoint.y,
    deltaY: -100,
    bubbles: true,
    cancelable: true,
  });
  cv.dispatchEvent(overflowingChatWheel);
  expect("chat-wheel-overflow-event-is-consumed", overflowingChatWheel.defaultPrevented, true);
  expect("chat-wheel-overflow-event-moves-chat", state.chatScroll > 0, true);
  Object.assign(state, wheelState);
  const pointerTestRect = { left: 10, top: 20, right: 210, bottom: 120, width: 200, height: 100 };
  expect("pointer-scroll-reprojects-inside", pointerAtViewportPoint(110, 70, pointerTestRect, 1920, 1080), { x: 960, y: 540 });
  expect("pointer-scroll-clears-outside", pointerAtViewportPoint(9, 70, pointerTestRect, 1920, 1080), null);
  expect("surface-canvas-parent", cv.parentElement.id, "surface-display-slot");
  expect("surface-assets-ready", surfaceComposite.dataset.assetsReady, "true");
  expect("website-mode-file-default", shouldUseWebsiteMode(new URLSearchParams(), "file:"), false);
  expect("website-mode-hosted-default", shouldUseWebsiteMode(new URLSearchParams(), "https:"), true);
  expect("website-mode-hosted-explicit-ui", shouldUseWebsiteMode(new URLSearchParams("site=0"), "https:"), false);
  expect(
    "website-title-wrap-groups",
    [...websiteTitle.children].map((span) => span.textContent),
    ["Eine ruhige Oberfläche für ein", "KI-natives Betriebssystem."],
  );
  expect(
    "surface-display-measured-left",
    surfaceComposite.style.getPropertyValue("--display-left"),
    "27.604166666667%",
  );
  const shellModeBeforeNativeMeasure = document.body.dataset.shellMode;
  setWebsiteMode(true);
  const nativeDisplayRect = cv.getBoundingClientRect();
  const measuredCompositeWidth = surfaceComposite.getBoundingClientRect().width;
  const expectedDisplayWidth = Math.min(
    measuredCompositeWidth * 1100 / 2304,
    PHYSICAL_WIDTH,
  );
  expect(
    "surface-display-portrait-photo-css-size",
    [Math.round(nativeDisplayRect.width), Math.round(nativeDisplayRect.height)],
    [Math.round(expectedDisplayWidth), Math.round(expectedDisplayWidth * 9 / 16)],
  );
  expect(
    "surface-canvas-native-size-cap",
    nativeDisplayRect.width <= PHYSICAL_WIDTH + 0.01
      && nativeDisplayRect.height <= PHYSICAL_HEIGHT + 0.01,
    true,
  );
  const displaySlotRect = cv.parentElement.getBoundingClientRect();
  const websiteActionsRect = websiteActionsRow.getBoundingClientRect();
  expect(
    "surface-display-responsive-x-center",
    Math.abs(
      displaySlotRect.left + displaySlotRect.width / 2
        - document.documentElement.clientWidth / 2,
    ) <= 1,
    true,
  );
  expect(
    "surface-display-responsive-action-gap",
    Math.round(displaySlotRect.top - websiteActionsRect.bottom),
    Math.round(surfaceDisplayGapForViewport(SURFACE_ASSET_CONFIG, window.innerWidth)),
  );
  expect(
    "surface-display-gap-regular-viewport",
    surfaceDisplayGapForViewport(SURFACE_ASSET_CONFIG, 1280),
    64,
  );
  expect(
    "surface-display-gap-wide-viewport",
    surfaceDisplayGapForViewport(SURFACE_ASSET_CONFIG, 2048),
    92.16,
  );
  expect(
    "surface-display-gap-ultrawide-cap",
    surfaceDisplayGapForViewport(SURFACE_ASSET_CONFIG, 3440),
    112,
  );
  const websiteModeRect = websiteMode.getBoundingClientRect();
  const surfaceCompositeRect = surfaceComposite.getBoundingClientRect();
  expect(
    "surface-photo-never-misses-top",
    surfaceCompositeRect.top <= websiteModeRect.top + 0.01,
    true,
  );
  expect(
    "surface-reflection-crop-left",
    surfaceComposite.style.getPropertyValue("--reflection-left"),
    "27.560763888889%",
  );
  setWebsiteMode(shellModeBeforeNativeMeasure === "website");
  expect("wifi-list-row-height", LIST_ROW_H, 28);
  expect("vault-outcome-count", Object.keys(VAULT_OUTCOMES).length, 15);
  const scenarioIds = UI_LAB_SCENARIOS.map((scenario) => scenario.id);
  expect("scenario-ids-unique", new Set(scenarioIds).size, scenarioIds.length);
  expect("scenario-count", UI_LAB_SCENARIOS.length, 44);
  if (params.get("suite") === "all") {
    const saved = JSON.parse(JSON.stringify(state));
    for (const scenario of UI_LAB_SCENARIOS) {
      try {
        applyScenario(scenario.id, false);
        renderScene();
      } catch (error) {
        failures.push(`scenario-render:${scenario.id}: ${error.message}`);
      }
    }
    Object.assign(state, saved);
  }
  const modeBeforeToggle = document.body.dataset.shellMode;
  viewModeToggle.click();
  expect(
    "website-toggle-changes-mode",
    document.body.dataset.shellMode,
    modeBeforeToggle === "website" ? "ui" : "website",
  );
  viewModeToggle.click();
  expect("website-toggle-restores-mode", document.body.dataset.shellMode, modeBeforeToggle);
  const passed = failures.length === 0;
  labSelftest.dataset.status = passed ? "pass" : "fail";
  labSelftest.textContent = passed
    ? params.get("suite") === "all"
      ? `geometry-parity-pass;scenario-render-pass:${UI_LAB_SCENARIOS.length}`
      : "geometry-parity-pass"
    : failures.join("\n");
  window.__RAIOS_UI_LAB__ = {
    version: "1080p-structured-mirror-v2",
    selftest: { passed, failures },
    physicalSize: { width: cv.width, height: cv.height },
    logicalSize: { width: L.W, height: L.H },
    rendererCommit: "ad07424",
    scenarioCount: UI_LAB_SCENARIOS.length,
  };
  if (!passed) {
    state.labDiagnostics = true;
    console.error("UI Lab geometry parity failed", failures);
  }
  return passed;
}

// Deterministische Start-Zustaende fuer Screenshot-Automation:
// ?view=setup|wifi-starting|wifi-list|wifi-password|wifi-connecting
// &recovery=1 &wifi=connected &chat=demo &pw=<text> &progress=<0-100> &focus=1
// &layout=closed|open (Traum-Layout sofort auf den Screenshot-Keyframe setzen)
const params = new URLSearchParams(location.search);
if (params.get("scenario")) applyScenario(params.get("scenario"), false);
if (params.get("mode") === "proposal") {
  state.labMode = "proposal";
  state.labDiagnostics = true;
}
if (params.get("diagnostics") === "1") state.labDiagnostics = true;
if (params.get("classic") === "1" || params.get("dream") === "0") state.dream = false;
if (params.get("tab")) state.dreamTab = params.get("tab");
if (params.get("built") === "1") state.buildB = 100;
if (params.get("approved") === "1") { state.buildB = 100; state.approved = true; }
if (params.get("expand") === "1") { state.layoutT = 1; state.layoutGoal = 1; state.hoverCenter = true; }
if (params.get("layout") === "closed") { state.layoutT = -1; state.layoutGoal = -1; state.hoverCenter = false; }
else if (params.get("layout") === "open") { state.layoutT = 1; state.layoutGoal = 1; state.hoverCenter = true; }
if (params.get("view")) { state.view = params.get("view"); state.dream = false; }
// Testhilfe: synthetischer Klick auf den linken Bogen durch die echte Event-Kette
if (params.get("clicktest") === "toggle") {
  setTimeout(() => {
    const r = cv.getBoundingClientRect();
    cv.dispatchEvent(new MouseEvent("click", {
      clientX: r.left + 320 / L.W * r.width,
      clientY: r.top + 18 / L.H * r.height,
      bubbles: true,
    }));
  }, 400);
}
if (params.get("clicktest") === "bracket") {
  setTimeout(() => {
    const r = cv.getBoundingClientRect();
    const lay = dreamLayout();
    const bx = lay.brackets.left - lay.brackets.amp * Math.sin(Math.PI * 0.5);
    cv.dispatchEvent(new MouseEvent("click", {
      clientX: r.left + (bx + 1) / L.W * r.width,
      clientY: r.top + 220 / L.H * r.height,
      bubbles: true,
    }));
  }, 400);
}
if (params.get("recovery") === "1") state.recoveryOpen = true;
if (params.get("wifi") === "connected") state.wifiConnected = true;
if (params.get("chat") === "demo") {
  state.chat = [
    { speaker: "user", text: "Build me a small paint tool" },
    { speaker: "assistant", text: "Ok, machen wir!" },
    { speaker: "system", text: "Signed shell proof requested" },
  ];
}
if (state.view.startsWith("wifi-") && state.wifiSelected === null) state.wifiSelected = 0;
if (params.get("pw")) state.wifiPassword = params.get("pw");
if (params.get("progress")) state.wifiProgress = parseInt(params.get("progress"), 10) || 0;
if (params.get("focus") === "1") state.composerFocus = true;

buildScenarioCatalog();
if (params.get("catalog") === "1") toggleScenarioCatalog(true);
document.addEventListener("keydown", (event) => {
  if (event.key === "F2") {
    toggleScenarioCatalog();
    event.preventDefault();
  }
}, true);

runLabSelftests();
render();
if (document.body.dataset.shellMode !== "website") cv.focus();
