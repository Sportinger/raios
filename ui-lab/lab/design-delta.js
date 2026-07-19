function exportDesignDelta() {
  const delta = {
    baseline: {
      sources: [
        "seed-kernel/src/shell_host/genesis.rs",
        "seed-kernel/src/shell_host/dream.rs",
        "seed-kernel/src/shell_host/recovery.rs",
        "seed-kernel/src/shell_host/wifi_flow.rs",
        "seed-kernel/src/shell_host/vault_flow.rs",
        "seed-kernel/src/shell_host/personal_surface.rs",
        "crates/raios-core/src/genesis_layout.rs",
        "crates/raios-core/src/ui_program.rs",
        "seed-kernel/src/framebuffer.rs",
      ],
      rendererCommit: "ad07424",
      physicalSize: { width: cv.width, height: cv.height },
      logicalSize: { width: L.W, height: L.H },
    },
    tokens: PAL,
    layout: { genesis: L, dream: dreamKeyframes(cv.width, cv.height) },
    scenarios: UI_LAB_SCENARIOS.map(({ id, group, title, source }) => ({ id, group, title, source })),
    additions: [
      { kind: "behavior", name: "sim-ai-reply", wired: false, note: "Chat antwortet immer 'Ok, machen wir!'" },
      { kind: "behavior", name: "shell-proof-system-line", wired: false, note: "System-Chatzeile nach Klick" },
      { kind: "data", name: "wifi-scan-results", wired: false, note: "erfundene Netzliste im scan_line-Format" },
      { kind: "data", name: "recovery-snapshot", wired: false, note: "erfundene Recovery-Werte (Posture/Problem/...)" },
      { kind: "ui", name: "dream-layout", wired: true, note: "Im Kernel gelandet; Lab spiegelt responsive Geometrie fuer 1280x800 und 1920x1080." },
      { kind: "primitive", name: "text-hires", wired: true, note: "Im Kernel als physischer 1px-Bitmaptext gelandet." },
      { kind: "interaction", name: "bracket-breathing", wired: true, note: "Im Kernel zeitbegrenzt und tick-getrieben gelandet." },
      { kind: "interaction", name: "layout-state-machine", wired: true, note: "Closed/Ambient/Open und Hover-Hysterese sind im Kernel gelandet." },
      { kind: "visual-pass", name: "dream-polish-v2", wired: true, note: "Bayer-Hintergrund, Sternenfeld, Kerze, Klammern und Dream-Widgets sind im Kernel gelandet." },
      { kind: "ui", name: "host-surface-catalog", wired: true, note: "Genesis, Recovery, WiFi, Vault und Personal-Surface sind nach ihren Rust-Ownern getrennt gespiegelt." },
      { kind: "performance", name: "damage-set-32", wired: false, note: "PROPOSAL: host-berechnete Dirty Rects, Merge und Full-Frame-Fallback." },
      { kind: "performance", name: "three-layer-compositor", wired: false, note: "PROPOSAL: baked background, core scene, trusted front layer." },
      { kind: "performance", name: "active-60hz-deadline", wired: false, note: "PROPOSAL: 16.67ms nur waehrend aktiver Animation; Frames werden nicht aufgestaut." },
    ],
  };
  navigator.clipboard.writeText(JSON.stringify(delta, null, 2)).catch(() => {});
}
