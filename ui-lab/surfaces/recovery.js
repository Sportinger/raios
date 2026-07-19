// Mirrors seed-kernel/src/shell_host/recovery.rs.

function drawRecoveryContext(rect) {
  const maxChars = Math.floor((rect.w - 28) / ADV);
  if (rect.h < 260) {
    drawTruncated(rect.x + 14, rect.y + 48, state.recoveryFacts.posture, maxChars, "APP_AMBER");
    drawTruncated(rect.x + 14, rect.y + 68, state.recoveryFacts.problem, maxChars, "APP_AMBER");
    return;
  }

  const rows = [
    ["Posture", state.recoveryFacts.posture, "APP_AMBER"],
    ["Current problem", state.recoveryFacts.problem, "APP_AMBER"],
    ["Last known good", state.recoveryFacts.lastGood, "TEXT_MUTED"],
    ["Crashed services", state.recoveryFacts.crashed, "TEXT_MUTED"],
    ["Disabled modules", state.recoveryFacts.disabled, "TEXT_MUTED"],
  ];
  let y = rect.y + 43;
  for (const [label, value, color] of rows) {
    drawText(rect.x + 14, y, label, "TEXT_MUTED");
    drawTruncated(rect.x + 14, y + 10, value, maxChars, color);
    y += 26;
  }

  const actions = [
    ["Restart last good", true],
    ["Disable module", true],
    ["Load local hash", false],
    ["Rollback unavailable", false],
  ];
  const firstY = rect.y + rect.h - 94;
  actions.forEach(([label, enabled], index) => {
    drawButton(
      { x: rect.x + 12, y: firstY + index * 20, w: rect.w - 24, h: 18 },
      label,
      enabled,
      "recovery-" + index,
    );
  });
}

function toggleRecovery() {
  state.recoveryOpen = !state.recoveryOpen;
  render();
}
