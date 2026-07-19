function fillRect(x, y, w, h, col) {
  if (w <= 0 || h <= 0) return;
  target.fillStyle = PAL[col] || col;
  target.fillRect(Math.round(x) * SCALE, Math.round(y) * SCALE, Math.round(w) * SCALE, Math.round(h) * SCALE);
}

function glyphIndex(ch) {
  const c = ch.codePointAt(0);
  if (c >= 32 && c <= 126) return c - 32;
  if (ch in UMLAUT) return UMLAUT[ch];
  return 0;
}

function drawText(x, y, str, col) {
  target.fillStyle = PAL[col] || col;
  let px = x * SCALE;
  for (const ch of str) {
    const g = FONT8X8[glyphIndex(ch)];
    for (let row = 0; row < 8; row++) {
      const bits = g[row];
      for (let c = 0; c < 8; c++) {
        if ((bits >> c) & 1) target.fillRect(px + c * SCALE, (y + row) * SCALE, SCALE, SCALE);
      }
    }
    px += ADV * SCALE;
  }
}

function textWidth(str) { return [...str].length * ADV; }

// Physisch aufgeloester Text: dieselben 8x8-Bitmaps, aber 1 physisches Pixel
// pro Fontpixel (Kernel-Aequivalent: set_physical_pixel) — halbe Groesse,
// doppelte Schaerfe, weniger Pixel-Schreibarbeit als der 2x-Pfad.
function drawTextHi(x, y, str, col) {
  target.fillStyle = PAL[col] || col;
  let px = Math.round(x * SCALE);
  const py = Math.round(y * SCALE);
  for (const ch of str) {
    const g = FONT8X8[glyphIndex(ch)];
    for (let row = 0; row < 8; row++) {
      const bits = g[row];
      for (let c = 0; c < 8; c++) {
        if ((bits >> c) & 1) target.fillRect(px + c, py + row, 1, 1);
      }
    }
    px += 9;
  }
}

function drawTruncated(x, y, str, maxChars, col) {
  const chars = [...str];
  if (chars.length <= maxChars) { drawText(x, y, str, col); return; }
  if (maxChars < 4) return;
  drawText(x, y, chars.slice(0, maxChars - 3).join(""), col);
  drawText(x + (maxChars - 3) * ADV, y, "...", col);
}

function drawOutline(r, col) {
  fillRect(r.x, r.y, r.w, 1, col);
  fillRect(r.x, r.y + r.h - 1, r.w, 1, col);
  fillRect(r.x, r.y, 1, r.h, col);
  fillRect(r.x + r.w - 1, r.y, 1, r.h, col);
}

function drawPanel(r, title) {
  fillRect(r.x, r.y, r.w, r.h, "SURFACE_BG");
  drawOutline(r, "HAIRLINE");
  drawText(r.x + 14, r.y + 16, title, "TEXT_MAIN");
  fillRect(r.x + 14, r.y + 31, r.w - 28, 1, "HAIRLINE");
}

function drawButton(r, label, primary, id, action) {
  const isPressed = state.pressed === id;
  fillRect(r.x, r.y, r.w, r.h, primary || isPressed ? "APP_BLUE" : "SURFACE_ALT");
  drawOutline(r, primary || isPressed ? "APP_BLUE" : "HAIRLINE");
  const x = r.x + Math.max(0, r.w - textWidth(label)) / 2;
  drawText(Math.floor(x), r.y + 8, label, "TEXT_MAIN");
  hits.push({ ...r, action: () => { flash(id); if (action) action(); } });
}

function flash(id) {
  state.pressed = id;
  render();
  setTimeout(() => { state.pressed = null; render(); }, 160);
}
