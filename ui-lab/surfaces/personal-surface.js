// Mirrors the validated display-list compositor in shell_host/genesis.rs and
// the checked-in guests in wasm-guests/svc-personal-shell-proof.

const PERSONAL_COLORS = {
  proofIdle: "#111820",
  proofInput: "#101F2A",
  proofPanel: "#1C2733",
  proofBlue: "#63A7FF",
  proofGreen: "#36C7A1",
  appBackground: "#14161A",
  appField: "#272B33",
  appEdit: "#1B212B",
};

function personalOrigin() {
  return { x: L.personal.x, y: L.personal.y, w: L.personal.w, h: L.personal.h };
}

function clearPersonalSurface(color) {
  fillRect(0, 0, L.W, L.H, "#08090B");
  const area = personalOrigin();
  fillRect(area.x, area.y, area.w, area.h, color);
  hits.push({ ...area, action: () => {} });
}

function drawPersonalProof() {
  const area = personalOrigin();
  const input = state.personalInputReceived;
  const accent = input ? PERSONAL_COLORS.proofGreen : PERSONAL_COLORS.proofBlue;
  clearPersonalSurface(input ? PERSONAL_COLORS.proofInput : PERSONAL_COLORS.proofIdle);
  const panel = {
    x: area.x + 24,
    y: area.y + 24,
    w: Math.max(1, area.w - 48),
    h: Math.max(1, Math.min(area.h - 48, 220)),
  };
  fillRect(panel.x, panel.y, panel.w, panel.h, PERSONAL_COLORS.proofPanel);
  drawOutline(panel, accent);
  drawText(area.x + 48, area.y + 52, "PERSONAL SHELL PROOF", "#F4F7FA");
  drawText(
    area.x + 48,
    area.y + 92,
    input ? "SANITIZED INPUT RECEIVED" : "READY FOR SANITIZED INPUT",
    accent,
  );
  drawOutline({ x: area.x + 44, y: area.y + 78, w: Math.max(1, panel.w - 40), h: 40 }, "APP_AMBER");
}

function drawPersonalCalculator() {
  const area = personalOrigin();
  clearPersonalSurface(PERSONAL_COLORS.appBackground);
  drawText(area.x + 8, area.y + 8, "Calculator", "#F4F7FA");

  const display = { x: area.x, y: area.y + 40, w: 320, h: 56 };
  fillRect(display.x, display.y, display.w, display.h, PERSONAL_COLORS.appField);
  drawOutline(display, "HAIRLINE");
  drawText(display.x + 12, display.y + 18, String(state.calculatorValue), "#F4F7FA");

  const labels = ["7", "8", "9", "/", "4", "5", "6", "*", "1", "2", "3", "-", "C", "0", "=", "+"];
  labels.forEach((label, index) => {
    const col = index % 4;
    const row = Math.floor(index / 4);
    const rect = { x: area.x + col * 80, y: area.y + 104 + row * 64, w: 72, h: 56 };
    fillRect(rect.x, rect.y, rect.w, rect.h, PERSONAL_COLORS.appField);
    drawOutline(rect, "APP_BLUE");
    drawText(rect.x + 12, rect.y + 18, label, "#F4F7FA");
    hits.push({ ...rect, action: () => personalCalculatorKey(label) });
  });
}

function personalCalculatorKey(label) {
  if (/^[0-9]$/.test(label)) {
    const next = Math.abs(state.calculatorValue) > 999999999
      ? Number(label)
      : state.calculatorValue * 10 + Number(label);
    state.calculatorValue = next;
  } else if (label === "C") {
    state.calculatorValue = 0;
  }
  render();
}

function editorVisualLines(value, columns) {
  const lines = [];
  for (const paragraph of value.split("\n")) {
    if (!paragraph.length) {
      lines.push("");
      continue;
    }
    for (let start = 0; start < paragraph.length; start += columns) {
      lines.push(paragraph.slice(start, start + columns));
    }
  }
  return lines;
}

function drawPersonalEditor() {
  const area = personalOrigin();
  clearPersonalSurface(PERSONAL_COLORS.appBackground);
  drawText(area.x + 16, area.y + 16, "raiOS EDIT  F12=EXIT", "#F4F7FA");
  const edit = { x: area.x + 8, y: area.y + 40, w: 608, h: 384 };
  fillRect(edit.x, edit.y, edit.w, edit.h, PERSONAL_COLORS.appEdit);
  drawOutline(edit, "HAIRLINE");

  const columns = Math.max(1, Math.min(52, Math.floor((edit.w - 24) / ADV)));
  const rows = Math.max(1, Math.floor((edit.h - 24) / 20));
  const lines = editorVisualLines(state.editorText, columns);
  const visible = lines.slice(Math.max(0, lines.length - rows));
  visible.forEach((line, row) => drawText(edit.x + 12, edit.y + 18 + row * 20, line, "#F4F7FA"));
  const finalLine = visible[visible.length - 1] || "";
  const cursorX = Math.min(edit.x + edit.w - 8, edit.x + 12 + finalLine.length * ADV);
  const cursorY = Math.min(edit.y + edit.h - 16, edit.y + 18 + (visible.length - 1) * 20);
  fillRect(cursorX, cursorY, 8, 16, PERSONAL_COLORS.proofGreen);

  const clear = { x: area.x + 8, y: area.y + 436, w: 96, h: 36 };
  fillRect(clear.x, clear.y, clear.w, clear.h, PERSONAL_COLORS.appField);
  drawOutline(clear, "APP_BLUE");
  drawText(clear.x + 12, clear.y + 18, "CLEAR", "#F4F7FA");
  hits.push({ ...clear, action: () => { state.editorText = ""; render(); } });
}

function drawPersonalSurface() {
  switch (state.personalMode) {
    case "calculator": drawPersonalCalculator(); break;
    case "editor": drawPersonalEditor(); break;
    default: drawPersonalProof(); break;
  }
}

function handlePersonalKey(event) {
  if (event.key === "Escape") {
    state.view = "genesis";
    state.personalMode = null;
    render();
    return true;
  }
  if (state.personalMode === "proof" && event.key.length === 1) {
    state.personalInputReceived = true;
    render();
    return true;
  }
  if (state.personalMode === "calculator") {
    if (/^[0-9]$/.test(event.key)) personalCalculatorKey(event.key);
    else if (event.key.toLowerCase() === "c") personalCalculatorKey("C");
    else return false;
    return true;
  }
  if (state.personalMode === "editor") {
    if (event.key === "Backspace") state.editorText = state.editorText.slice(0, -1);
    else if (event.key === "Enter") state.editorText += "\n";
    else if (event.key.length === 1 && state.editorText.length < 2048) state.editorText += event.key;
    else return false;
    render();
    return true;
  }
  return false;
}
