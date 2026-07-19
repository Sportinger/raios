let layoutAnimTimer = null;
// Ease-in-out (kubisch), zeitbasiert und bildschirm-synchron:
// requestAnimationFrame liefert einen Frame pro Display-Refresh, die
// Position kommt aus der verstrichenen Zeit — kein Timer-Jitter mehr.
// (Kernel-Aequivalent: Tick-getriebene Frames, Position = f(uptime).)
function easeInOut(t) {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

const LAYOUT_ANIM_MS = 500;
function animateLayout(goal) {
  goal = Math.max(-1, Math.min(1, goal));
  state.layoutGoal = goal;
  if (layoutAnimTimer) cancelAnimationFrame(layoutAnimTimer);
  const start = state.layoutT;
  const t0 = performance.now();
  const step = (now) => {
    const t = Math.min(1, (now - t0) / LAYOUT_ANIM_MS);
    state.layoutT = lerp(start, goal, easeInOut(t));
    render();
    if (t < 1) layoutAnimTimer = requestAnimationFrame(step);
    else layoutAnimTimer = null;
  };
  layoutAnimTimer = requestAnimationFrame(step);
}

function drawBracket(xc, dir, amp, hook, top, bottom) {
  // Physische 1px-Praezision: waehrend der Transition gleiten die Boegen in
  // halben Logik-Pixeln statt in 2px-Spruengen — das nimmt das Stufige.
  hook = Math.round(hook);
  const span = Math.max(1, bottom - top);
  for (let y = top; y <= bottom; y++) {
    const t = (y - top) / span;
    const s = Math.sin(Math.PI * t);
    const x = xc + dir * amp * s;
    const w = s > 0.85 ? 3 : 2;
    const core = s > 0.92 ? "#5E7089" : s > 0.55 ? "#4A5568" : "#3E434C";
    const px = Math.round(x * 2), py = y * 2;
    target.fillStyle = "#151A21";
    target.fillRect(px - 2, py, w * 2 + 4, 2);
    target.fillStyle = core;
    target.fillRect(px, py, w * 2, 2);
  }
  xc = Math.round(xc);
  // Endhaken: kleine horizontale Abschluesse Richtung Mitte
  fillRect(xc - (dir > 0 ? hook - 1 : 0), top, hook, 1, "#3E434C");
  fillRect(xc - (dir > 0 ? hook - 1 : 0), bottom, hook, 1, "#3E434C");
}

function addBracketHits(xc, dir, amp, top, bottom) {
  // Kurvennahe Trefferstreifen statt eines grossen unsichtbaren Rechtecks.
  const span = Math.max(1, bottom - top);
  for (let y = top; y <= bottom; y += 8) {
    const t = (y - top) / span;
    const x = Math.round(xc + dir * amp * Math.sin(Math.PI * t));
    hits.push({ x: x - 18, y: y - 4, w: 37, h: 12, action: () => {
      state.hoverCenter = false;
      animateLayout(state.layoutGoal === -1 || state.layoutT < -0.5 ? 0 : -1);
    }});
  }
}

function railHeader(x, y, label) {
  drawText(x, y, label, "#5C6470");
  fillRect(x, y + 10, textWidth(label), 1, "#20252D");
}

function drawDiamond(x, y, col) {
  fillRect(x + 2, y, 1, 1, col);
  fillRect(x + 1, y + 1, 3, 1, col);
  fillRect(x, y + 2, 5, 1, col);
  fillRect(x + 1, y + 3, 3, 1, col);
  fillRect(x + 2, y + 4, 1, 1, col);
}

function segBar(x, y, w, pct, fill, light) {
  fillRect(x, y, w, 8, "#0D0F13");
  fillRect(x + 1, y + 1, w - 2, 1, "#07090B");
  drawOutline({ x, y, w, h: 8 }, "#1E232B");
  const filled = Math.round(12 * Math.min(100, pct) / 100);
  for (let i = 0; i < filled; i++) {
    const sx = x + 2 + i * 9;
    fillRect(sx, y + 2, 8, 4, fill);
    fillRect(sx, y + 2, 8, 1, light);
  }
}

// ---------- Hover-System: pro Button ein weicher 4-Frame-Aufwaerts-Fade ----------

function hoverAmount(id) {
  return state.hoverId === id ? state.hoverT : 0;
}

let hoverAnimTimer = null;
function animateHover() {
  if (hoverAnimTimer) clearTimeout(hoverAnimTimer);
  state.hoverT = 0;
  let frame = 0;
  const step = () => {
    frame++;
    state.hoverT = easeInOut(Math.min(1, frame / 4));
    render();
    if (frame < 4) hoverAnimTimer = setTimeout(step, 30);
    else hoverAnimTimer = null;
  };
  step();
}

// Zeitlose Andeutungs-Buttons: kein Kasten. Ruhend nur Label + feine
// Grundlinie; bei Hover waechst mittig eine blaue Linie und das Label hellt
// auf. Primaer traegt ruhend blaues Label + blaue Grundlinie.
// EINHEIT (Owner): alle Buttons 130x16, Text immer in der feinen Stufe.
const BTN_W = 130, BTN_H = 16;
function ghostButton(x, y, wIgnored, label, id, action, mode) {
  const w = BTN_W, hi = true;
  const r = { x, y, w, h: BTN_H };
  const tw = Math.ceil([...label].length * 4.5);
  const tx = x + Math.max(0, Math.floor((w - tw) / 2));
  const draw = drawTextHi;
  const ty = y + 4;
  if (mode === "disabled") {
    draw(tx, ty, label, "#454D5A");
    fillRect(x, y + 15, w, 1, "#1A1F26");
    return;
  }
  const hov = hoverAmount(id);
  const pressed = state.pressed === id;
  const primary = mode === "primary";
  if (pressed) fillRect(x, y, w, 16, "#2E5C94");
  else if (hov > 0) fillRect(x, y, w, 16, primary ? "#14273D" : "#12161C");
  draw(tx, ty, label, pressed || hov > 0 ? "TEXT_MAIN" : primary ? "#7CB3F5" : "TEXT_MUTED");
  fillRect(x, y + 15, w, 1, primary ? "#2E5C94" : "#20252D");
  const uw = Math.round(w * hov);
  if (uw > 0) fillRect(x + Math.floor((w - uw) / 2), y + 15, uw, primary ? 2 : 1, "APP_BLUE");
  hits.push({ ...r, id, action: () => { flash(id); if (action) action(); } });
}

function railButton(x, y, w, label, id, action, mode) { ghostButton(x, y, w, label, id, action, mode); }
function railButtonHi(x, y, w, label, id, action, mode) { ghostButton(x, y, w, label, id, action, mode); }

// Programm-Zeile mit Hover: Pfeil faerbt sich blau, Linie waechst unter dem Label
function ghostItem(x, y, w, label, id, hi) {
  const hov = hoverAmount(id);
  const draw = hi ? drawTextHi : drawText;
  draw(x, y, ">", hov > 0 ? "APP_BLUE" : "#454D5A");
  draw(x + 12, y, label, "TEXT_MAIN");
  const tw = hi ? Math.ceil([...label].length * 4.5) : textWidth(label);
  const uw = Math.round(tw * hov);
  if (uw > 0) fillRect(x + 12, y + (hi ? 9 : 10), uw, 1, "APP_BLUE");
  hits.push({ x, y: y - 3, w, h: 14, id, action: () => flash(id) });
}

function buildSteps() {
  const bDone = state.buildB >= 100;
  return [
    ["Sources verified", true],
    ["Build A complete", true],
    [bDone ? "Build B complete" : "Build B running", bDone],
    ["Byte compare", bDone],
    [state.approved ? "W5 approved" : "W5 approval", state.approved],
  ];
}

function drawBuildChecklist(x, y) {
  let activeSeen = false;
  for (const [label, done] of buildSteps()) {
    const active = !done && !activeSeen;
    if (active) activeSeen = true;
    const boxCol = done ? "APP_GREEN" : active ? "APP_AMBER" : "#333A45";
    drawOutline({ x, y, w: 7, h: 7 }, boxCol);
    if (done) {
      fillRect(x + 2, y + 3, 1, 1, "APP_GREEN");
      fillRect(x + 3, y + 4, 1, 1, "APP_GREEN");
      fillRect(x + 4, y + 2, 1, 1, "APP_GREEN");
    }
    drawTextHi(x + 13, y + 1, label, done ? "TEXT_MUTED" : active ? "TEXT_MAIN" : "#5C6470");
    y += 13;
  }
}

function segBarWide(x, y, w, pct, fill, light) {
  fillRect(x, y, w, 8, "#0D0F13");
  fillRect(x + 1, y + 1, w - 2, 1, "#07090B");
  drawOutline({ x, y, w, h: 8 }, "#1E232B");
  const filled = Math.round(12 * Math.min(100, pct) / 100);
  const slot = Math.max(2, Math.floor((w - 4) / 12));
  for (let i = 0; i < filled; i++) {
    const sx = x + 2 + i * slot;
    fillRect(sx, y + 2, Math.max(1, slot - 1), 4, fill);
    fillRect(sx, y + 2, Math.max(1, slot - 1), 1, light);
  }
}

function approveBuild() {
  if (state.buildB >= 100 && !state.approved) {
    state.approved = true;
    pushChat("system", "paint-tool approved (W5) - running");
  }
}

function drawAmbientRails(left, right) {
  const lx = left.x, lw = left.w;
  railHeader(lx, 42, "TRUST");
  drawDiamond(lx, 58, "APP_GREEN");
  drawText(lx + 10, 56, "Core safe", "TEXT_MAIN");
  drawText(lx + 10, 70, state.wifiConnected ? "Recovery ready" : "Recovery avail", "TEXT_MUTED");

  railHeader(lx, 96, "SOURCES");
  drawTextHi(lx, 111, "AI", "TEXT_MUTED");
  drawTextHi(lx + 50, 111, "Needs key", "APP_AMBER");
  drawTextHi(lx, 123, "Net", "TEXT_MUTED");
  drawTextHi(lx + 50, 123, state.wifiConnected ? "Ready" : "Connecting", state.wifiConnected ? "APP_GREEN" : "APP_AMBER");
  drawTextHi(lx, 135, "Vault", "TEXT_MUTED");
  drawTextHi(lx + 50, 135, "Locked", "TEXT_MUTED");

  railHeader(lx, 164, "PROBLEMS");
  drawDiamond(lx, 178, "APP_RED");
  drawText(lx + 10, 176, "1 critical", "APP_RED");
  railButton(lx, 204, lw, "AI setup", "d-ai", () => openOverlay("setup"), "normal");
  railButton(lx, 226, lw, "WiFi setup", "d-wifi", startWifi, "normal");
  railButton(lx, 248, lw, "Recovery", "d-rec", toggleRecovery, "normal");

  const rx = right.x, rw = right.w;
  railHeader(rx, 42, "PROGRAMS");
  ghostItem(rx, 56, rw, "Calculator", "d-calc", false);
  ghostItem(rx, 70, rw, "Editor", "d-edit", false);

  railHeader(rx, 96, "BUILD");
  drawText(rx, 110, "paint-tool", "TEXT_MAIN");
  drawTextHi(rx, 125, "A", "TEXT_MUTED");
  segBar(rx + 10, 123, Math.min(112, rw - 10), 100, "APP_GREEN", "#7FD4A4");
  drawTextHi(rx, 139, "B", "TEXT_MUTED");
  segBar(rx + 10, 137, Math.min(112, rw - 10), state.buildB, "APP_BLUE", "#7CB3F5");
  const bDone = state.buildB >= 100;
  if (state.approved) drawDiamond(rx, 152, "APP_GREEN");
  drawTextHi(state.approved ? rx + 10 : rx, 154, state.approved ? "running" : bDone ? "awaiting W5 click" : "double build running", state.approved ? "APP_GREEN" : bDone ? "APP_AMBER" : "TEXT_MUTED");
  railButton(rx, 170, rw, state.approved ? "Approved" : "Approve + run", "d-approve", approveBuild, bDone && !state.approved ? "primary" : "disabled");

  railHeader(rx, 206, "PLAYGROUND");
  railButton(rx, 222, rw, "New domain", "d-dom", null, "normal");
}

function drawClosedRails(left, right) {
  const lx = left.x, lw = left.w;
  const lg = 12, lcw = Math.floor((lw - lg) / 2), sx = lx + lcw + lg;
  railHeader(lx, 42, "TRUST");
  drawDiamond(lx, 58, "APP_GREEN");
  drawText(lx + 10, 56, "Core safe", "TEXT_MAIN");
  drawTextHi(lx + 10, 72, state.wifiConnected ? "Recovery ready" : "Recovery available", "TEXT_MUTED");

  railHeader(sx, 42, "SOURCES");
  drawTextHi(sx, 58, "AI", "TEXT_MUTED");
  drawTextHi(sx + 42, 58, "Needs key", "APP_AMBER");
  drawTextHi(sx, 71, "Net", "TEXT_MUTED");
  drawTextHi(sx + 42, 71, state.wifiConnected ? "Ready" : "Connecting", state.wifiConnected ? "APP_GREEN" : "APP_AMBER");
  drawTextHi(sx, 84, "Vault", "TEXT_MUTED");
  drawTextHi(sx + 42, 84, "Locked", "TEXT_MUTED");

  railHeader(lx, 108, "PROBLEMS");
  drawDiamond(lx, 124, "APP_RED");
  drawText(lx + 10, 122, "1 critical", "APP_RED");
  railButtonHi(lx, 146, 0, "AI setup", "d-ai", () => openOverlay("setup"), "normal");
  railButtonHi(lx + BTN_W + 12, 146, 0, "WiFi setup", "d-wifi", startWifi, "normal");
  railButtonHi(lx, 168, 0, "Recovery", "d-rec", toggleRecovery, "normal");

  const rx = right.x, rw = right.w;
  railHeader(rx, 42, "BUILD");
  drawText(rx, 58, "paint-tool", "TEXT_MAIN");
  const barX = rx + 12, barW = Math.min(200, rw - 44);
  drawTextHi(rx, 73, "A", "TEXT_MUTED");
  segBarWide(barX, 71, barW, 100, "APP_GREEN", "#7FD4A4");
  drawTextHi(rx, 87, "B", "TEXT_MUTED");
  segBarWide(barX, 85, barW, state.buildB, "APP_BLUE", "#7CB3F5");
  drawTextHi(barX + barW + 6, 87, state.buildB + "%", "TEXT_FAINT");
  const bDone = state.buildB >= 100;
  drawTextHi(rx, 101, state.approved ? "running in own domain" : bDone ? "awaiting W5 approval" : "double build running", state.approved ? "APP_GREEN" : bDone ? "APP_AMBER" : "TEXT_MUTED");
  railButtonHi(rx, 116, 0, state.approved ? "Approved" : "Approve + run", "d-approve", approveBuild, bDone && !state.approved ? "primary" : "disabled");
  drawBuildChecklist(rx, 144);

  const cg = 12, cw = Math.floor((rw - cg) / 2), px = rx + cw + cg;
  railHeader(rx, 226, "PROGRAMS");
  ghostItem(rx, 242, cw, "Calculator", "d-calc", true);
  ghostItem(rx, 255, cw, "Editor", "d-edit", true);
  railHeader(px, 226, "PLAYGROUND");
  railButtonHi(px, 242, 0, "New domain", "d-dom", null, "normal");
}

function dreamTabsStartX(layout) {
  const labelsWidth = textWidth("Chat") + textWidth("Console") + textWidth("Build");
  const totalWidth = labelsWidth + 36;
  return layout.center.x + Math.floor(Math.max(0, layout.center.w - totalWidth) / 2);
}

function drawDream() {
  if (!bgCache) bgCache = buildDreamBg();
  bctx.drawImage(bgCache, 0, 0);
  const layout = dreamLayout();
  const center = { x: Math.round(layout.center.x), w: Math.round(layout.center.w) };
  const left = { x: Math.round(layout.left.x), w: Math.round(layout.left.w) };
  const right = { x: Math.round(layout.right.x), w: Math.round(layout.right.w) };
  const brackets = {
    left: layout.brackets.left,
    right: layout.brackets.right,
    amp: layout.brackets.amp,
    hook: layout.brackets.hook,
  };
  const cx0 = center.x, cx1 = center.x + center.w;
  drawBracket(brackets.left, -1, brackets.amp, brackets.hook, layout.bracketTop, layout.bracketBottom);
  drawBracket(brackets.right, 1, brackets.amp, brackets.hook, layout.bracketTop, layout.bracketBottom);

  if (state.layoutT > -0.5) {
  // Reiter oben in der Mitte
  const tabs = [["Chat", "chat"], ["Console", "console"], ["Build", "build"]];
  let tx = dreamTabsStartX(layout);
  tabs.forEach(([label, key], ti) => {
    const w = textWidth(label);
    const active = state.dreamTab === key;
    const hov = hoverAmount("tab-" + key);
    drawText(tx, 46, label, active ? "TEXT_MAIN" : hov > 0 ? "TEXT_MUTED" : "#5C6470");
    if (active) {
      fillRect(tx, 58, w, 2, "APP_BLUE");
      fillRect(tx - 2, 60, w + 4, 1, "#1E3A5C");
    } else if (hov > 0) {
      fillRect(tx + Math.floor((w - Math.round(w * hov)) / 2), 58, Math.round(w * hov), 1, "#2E5C94");
    }
    hits.push({ x: tx - 4, y: 42, w: w + 8, h: 20, id: "tab-" + key, action: () => { state.dreamTab = key; render(); } });
    tx += w + 18;
    if (ti < tabs.length - 1) fillRect(tx - 11, 50, 2, 2, "#333A45");
  });

  // Mittelspalte
  const maxChars = Math.floor((cx1 - cx0) / ADV);
  const maxHi = Math.floor(((cx1 - cx0) - 10) / 4.5);
  if (state.dreamTab === "chat") {
    if (state.chat.length === 0) {
      drawTruncated(cx0, 82, "Welcome. What should your", maxChars, "TEXT_MAIN");
      drawTruncated(cx0, 96, "raiOS become?", maxChars, "TEXT_MAIN");
      drawTextHi(cx0, 118, [..."Ask for a tool, a workflow, or a change."].slice(0, maxHi).join(""), "TEXT_MUTED");
    } else {
      const conversationBottom = Math.max(DREAM_TOKENS.conversationTop, layout.composerY - DREAM_TOKENS.conversationComposerGap);
      const visible = Math.floor((conversationBottom - DREAM_TOKENS.conversationTop) / ROW_H);
      const rows = conversationRows(maxHi);
      const total = rows.length;
      state.chatScroll = Math.min(state.chatScroll, Math.max(0, total - visible));
      const endRow = total - state.chatScroll;
      const startRow = Math.max(0, endRow - visible);
      for (let i = startRow; i < endRow; i++) {
        const row = rows[i];
        if (!row) continue;
        const ry = DREAM_TOKENS.conversationTop + (i - startRow) * ROW_H;
        if (row[2] === "label") {
          fillRect(cx0, ry, 3, 8, row[1]);
          drawText(cx0 + 10, ry, row[0], row[1]);
        } else {
          drawTextHi(cx0 + 10, ry + 1, row[0], row[1]);
        }
      }
    }
    // Composer: Prompt-Zeile unten in der Klammer
    const shownMax = maxChars - 3;
    const chars = [...state.composerText];
    const shown = chars.length > shownMax ? chars.slice(chars.length - shownMax).join("") : state.composerText;
    drawText(cx0, layout.composerY, ">", "APP_BLUE");
    if (shown) drawText(cx0 + 2 * ADV, layout.composerY, shown, "TEXT_MAIN");
    else if (!state.composerFocus) drawText(cx0 + 2 * ADV, layout.composerY, "Ask anything...", "TEXT_FAINT");
    const uw = cx1 - cx0;
    fillRect(cx0, layout.composerY + 14, uw, 1, state.composerFocus ? "#2E5C94" : "HAIRLINE");
    fillRect(cx0, layout.composerY + 15, uw, 1, state.composerFocus ? "#1E3A5C" : "#171B21");
    state._composerCursor = { x: cx0 + (2 + [...shown].length) * ADV, y: layout.composerY - 3 };
    hits.push({ x: cx0, y: layout.composerY - 10, w: cx1 - cx0, h: 28, action: () => { state.composerFocus = true; state.cursorOn = true; present(); } });
  } else if (state.dreamTab === "console") {
    const lines = [
      ["> devices", "APP_BLUE"],
      ["FRAMEBUFFER: SEE UI", "TEXT_MUTED"],
      ["ENTROPY: READY - FILL 64/64 TOTAL 120 SRC RDRAND", "TEXT_MUTED"],
      ["USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 PWR 8 CONNECTED 2 KBD READY MOUSE READY", "TEXT_MUTED"],
      ["WIFI: MISSING - SURFACE PRO 4 88W8897 TARGET ABSENT SSID NONE KEY MISSING", "TEXT_MUTED"],
      ["NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2", "TEXT_MUTED"],
      ["INPUT: READY - USB HID KEYBOARD + POINTER", "TEXT_MUTED"],
    ];
    let y = 74;
    for (const [line, col] of lines) {
      drawTextHi(cx0, y, [...line].slice(0, maxHi).join(""), col);
      y += 9;
    }
  } else {
    const bDone = state.buildB >= 100;
    const steps = buildSteps();
    let y = 78;
    drawText(cx0, y, "paint-tool", "TEXT_MAIN");
    fillRect(cx0, y + 10, textWidth("paint-tool"), 1, "#20252D");
    y += 22;
    let activeSeen = false;
    for (const [label, done] of steps) {
      const active = !done && !activeSeen;
      if (active) activeSeen = true;
      const boxCol = done ? "APP_GREEN" : active ? "APP_AMBER" : "#333A45";
      drawOutline({ x: cx0, y: y, w: 7, h: 7 }, boxCol);
      if (done) {
        fillRect(cx0 + 2, y + 3, 1, 1, "APP_GREEN");
        fillRect(cx0 + 3, y + 4, 1, 1, "APP_GREEN");
        fillRect(cx0 + 4, y + 2, 1, 1, "APP_GREEN");
      }
      drawTextHi(cx0 + 13, y + 1, label, done ? "TEXT_MUTED" : active ? "TEXT_MAIN" : "#5C6470");
      y += 13;
      if (active && label.startsWith("Build B")) {
        const buildBarW = Math.min(150, center.w - 55);
        segBar(cx0 + 13, y, buildBarW, state.buildB, "APP_BLUE", "#7CB3F5");
        drawTextHi(cx0 + 19 + buildBarW, y + 1, state.buildB + "%", "TEXT_FAINT");
        y += 14;
      }
    }
    y += 8;
    drawText(cx0, y, state.approved ? "running in own domain" : "artifact inert until click", state.approved ? "APP_GREEN" : "APP_AMBER");
  }

  }

  // Inhalte wechseln an einer klaren Schwelle; die Geometrie selbst bleibt
  // waehrend der gesamten Transition kontinuierlich interpoliert.
  if (state.layoutT < -0.5) drawClosedRails(left, right);
  else if (state.layoutT > -0.5) drawAmbientRails(left, right);

  // Hover-Modell: die Klammern selbst sind keine Klickziele mehr — die Mitte
  // oeffnet sich beim Beruehren mit dem Zeiger und schliesst sich beim Verlassen.
}
