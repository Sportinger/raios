// ---------- Genesis-Grundansicht ----------

function drawSecureStrip() {
  const r = L.strip;
  fillRect(r.x, r.y, r.w, r.h, "SURFACE_BG");
  fillRect(r.x, r.y + r.h - 1, r.w, 1, "HAIRLINE");
  drawText(12, 14, "raiOS / Genesis", "TEXT_MAIN");
  const right = state.recoveryOpen
    ? "Recovery context / Click to close"
    : state.wifiConnected
      ? "Core safe / Recovery ready"
      : "Core safe / Recovery available";
  drawText(r.w - textWidth(right) - 12, 14, right, "TEXT_MUTED");
  hits.push({ x: r.w - 238, y: 0, w: 238, h: 38, action: () => {
    toggleRecovery();
  }});
}

function wrapLines(value, maxChars) {
  const out = [];
  for (const seg of value.split("\n")) {
    const chars = [...seg];
    if (chars.length === 0) { out.push(""); continue; }
    for (let i = 0; i < chars.length; i += maxChars) {
      out.push(chars.slice(i, i + maxChars).join(""));
    }
  }
  return out;
}

function conversationRows(maxChars) {
  // draw_conversation: pro Nachricht Label-Zeile + Textzeilen + Leerzeile
  const rows = [];
  for (const line of state.chat) {
    if (!line.text) continue;
    const label = line.speaker === "user" ? ["You", "APP_BLUE", "label"]
      : line.speaker === "assistant" ? ["raiOS", "TEXT_MAIN", "label"]
      : ["System", "TEXT_MUTED", "label"];
    rows.push(label);
    const bodyCol = line.speaker === "assistant" ? "TEXT_MAIN" : "TEXT_MUTED";
    for (const w of wrapLines(line.text, maxChars)) rows.push([w, bodyCol, "body"]);
    rows.push(null);
  }
  if (rows.length && rows[rows.length - 1] === null) rows.pop();
  return rows;
}

function drawConversation() {
  const r = L.conversation;
  drawPanel(r, "Conversation");
  if (state.chat.length === 0) {
    drawText(r.x + 18, r.y + 54, "Welcome. What should your raiOS become?", "TEXT_MAIN");
    drawText(r.x + 18, r.y + 74, "Ask for a tool, workflow, or a change.", "TEXT_MUTED");
    return;
  }
  const maxChars = Math.floor((r.w - 48) / ADV);
  const visible = Math.max(1, Math.floor((r.h - 54) / ROW_H));
  const rows = conversationRows(maxChars);
  const total = rows.length;
  state.chatScroll = Math.min(state.chatScroll, Math.max(0, total - visible));
  const endRow = total - state.chatScroll;
  const startRow = Math.max(0, endRow - visible);
  const contentY = r.y + 42;
  for (let i = startRow; i < endRow; i++) {
    const row = rows[i];
    if (!row) continue;
    drawText(r.x + 18, contentY + (i - startRow) * ROW_H, row[0], row[1]);
  }
  if (total > visible) {
    const trackY = r.y + 42, trackH = r.h - 54;
    const thumbH = Math.min(trackH, Math.max(12, Math.floor(trackH * visible / total)));
    const maxStart = Math.max(1, total - visible);
    const thumbY = trackY + Math.floor((trackH - thumbH) * Math.min(startRow, maxStart) / maxStart);
    fillRect(r.x + r.w - 7, trackY, 2, trackH, "HAIRLINE");
    fillRect(r.x + r.w - 8, thumbY, 4, thumbH, "APP_BLUE");
  }
}

function drawContext() {
  const r = L.context;
  drawPanel(r, state.recoveryOpen ? "Recovery context" : "Context");

  if (state.recoveryOpen) {
    drawRecoveryContext(r);
  } else {
    drawButton({ x: r.x + 12, y: r.y + 42, w: r.w - 24, h: 24 }, "Run signed shell proof", false, "shell-proof", () => {
      pushChat("system", "Signed shell proof requested");
    });
    const rows = [
      ["AI connection", "Needs key", "APP_AMBER"],
      ["Network", state.wifiConnected ? "Ready" : "Connecting", state.wifiConnected ? "APP_GREEN" : "APP_AMBER"],
      ["Secret Vault", "Unavailable", "TEXT_MUTED"],
      ["Problems", "Critical problem present", "APP_RED"],
    ];
    let y = r.y + 78;
    const maxChars = Math.floor((r.w - 28) / ADV);
    for (const [label, value, col] of rows) {
      drawText(r.x + 14, y, label, "TEXT_MUTED");
      drawTruncated(r.x + 14, y + 11, value, maxChars, col);
      y += 31;
    }
    drawButton({ x: r.x + 12, y: r.y + r.h - 62, w: r.w - 24, h: 22 }, "AI setup", false, "ai-setup", () => openOverlay("setup"));
    drawButton({ x: r.x + 12, y: r.y + r.h - 34, w: r.w - 24, h: 22 }, "WiFi setup", true, "wifi-setup", () => startWifi());
  }
}

function drawComposer() {
  const r = L.composer;
  fillRect(r.x, r.y, r.w, r.h, "SURFACE_ALT");
  drawOutline(r, "HAIRLINE");
  const maxChars = Math.floor((r.w - 58) / ADV);
  const value = state.composerText;
  let cursorChars = 0;
  if (value.length === 0) {
    drawText(r.x + 14, r.y + 18, "Ask anything, or /build <program>...", "TEXT_FAINT");
  } else {
    const chars = [...value];
    const limit = maxChars - 1;
    if (chars.length > limit) {
      drawText(r.x + 14, r.y + 18, "<", "TEXT_FAINT");
      drawText(r.x + 14 + ADV, r.y + 18, chars.slice(chars.length - limit).join(""), "TEXT_MAIN");
      cursorChars = limit + 1;
    } else {
      drawText(r.x + 14, r.y + 18, value, "TEXT_MAIN");
      cursorChars = chars.length;
    }
  }
  state._composerCursor = { x: r.x + 14 + cursorChars * ADV, y: r.y + 15 };
  hits.push({ ...r, action: () => { state.composerFocus = true; state.cursorOn = true; present(); } });

  drawButton({ x: r.x + r.w - 38, y: r.y + 8, w: 30, h: 30 }, ">", true, "send", submitChat);
}

function pushChat(speaker, text) {
  state.chat.push({ speaker, text });
  state.chatScroll = 0;
  render();
}

function submitChat() {
  const value = state.composerText.trim();
  if (!value) return;
  state.composerText = "";
  pushChat("user", value);
  setTimeout(() => pushChat("assistant", "Ok, machen wir!"), 650);
}

// ---------- Overlays ----------
