function proposalRegions() {
  if (!state.dream) {
    return [
      { name: "conversation", ...L.conversation },
      { name: "context", ...L.context },
      { name: "composer", ...L.composer },
      { name: "trusted-overlay", ...L.overlay },
    ];
  }
  const layout = dreamLayout();
  const top = layout.bracketTop;
  const height = Math.max(1, layout.bracketBottom - top + 1);
  return [
    { name: "left-scene", x: layout.left.x, y: top, w: layout.left.w, h: height },
    { name: "center-scene", x: layout.center.x, y: top, w: layout.center.w, h: height },
    { name: "right-scene", x: layout.right.x, y: top, w: layout.right.w, h: height },
    { name: "composer", x: layout.center.x, y: layout.composerY - 10, w: layout.center.w, h: 28 },
  ];
}

function drawProposalOverlay() {
  ctx.save();
  ctx.scale(SCALE, SCALE);
  ctx.lineWidth = 0.5;
  ctx.setLineDash([4, 3]);
  ctx.strokeStyle = PAL.APP_AMBER;
  ctx.fillStyle = "rgba(222, 164, 71, 0.035)";
  for (const region of proposalRegions()) {
    if (region.w <= 0 || region.h <= 0) continue;
    ctx.fillRect(region.x, region.y, region.w, region.h);
    ctx.strokeRect(region.x + 0.25, region.y + 0.25, Math.max(0, region.w - 0.5), Math.max(0, region.h - 0.5));
  }
  ctx.restore();
}

function formatMiB(bytes) { return (bytes / (1024 * 1024)).toFixed(2) + " MiB"; }

function updateLabHud() {
  const visible = state.labDiagnostics || state.labMode === "proposal";
  labHud.classList.toggle("visible", visible);
  labHud.dataset.mode = state.labMode;
  document.body.dataset.labMode = state.labMode;
  if (!visible) return;
  const frameBytes = cv.width * cv.height * 4;
  const header = state.labMode === "proposal"
    ? "PROPOSAL SIMULATION · NICHT IM KERNEL"
    : "ACTUAL LAB · RUST-GEOMETRIE";
  const modeLines = state.labMode === "proposal"
    ? [
        "Plan: DamageSet<32> + present_rect",
        "Plan: background / core / front layers",
        "Markierungen: geplante Damage-Domänen",
        "Wahrheit: Browserpfad bleibt aktuell FULL",
      ]
    : [
        "Present: FULL background + FULL front copy",
        "Pointer/Caret: separate Front-Ebene",
        "Daten: klar markierte lokale Fixtures",
      ];
  labHud.textContent = [
    header,
    `${cv.width}×${cv.height} physisch · ${L.W}×${L.H} logisch`,
    `Geometry selftest: ${labSelftest.dataset.status.toUpperCase()}`,
    `Scene ${labMetrics.lastSceneMs.toFixed(2)} ms · Present ${labMetrics.lastPresentMs.toFixed(2)} ms`,
    `Full path ≈ ${formatMiB(frameBytes * 2)} pro Szenenframe`,
    ...modeLines,
    "Ctrl+Shift+M: ACTUAL / PROPOSAL",
    "Ctrl+Shift+D: Diagnose ein/aus",
  ].join("\n");
}
