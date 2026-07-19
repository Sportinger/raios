const SCALE = 2, ADV = 9, ROW_H = 12;
const PHYSICAL_WIDTH = 1920, PHYSICAL_HEIGHT = 1080;

// Exakte Portierung von raios-core/src/genesis_layout.rs. Das UI Lab zeichnet
// nativ in 1920x1080; die Website nutzt dieselbe responsive Geometrie bis zu
// dieser harten Obergrenze.
function computeGenesisLayout(physicalWidth, physicalHeight) {
  const W = Math.floor(physicalWidth / SCALE);
  const H = Math.floor(physicalHeight / SCALE);
  if (W < 480 || H < 360) throw new Error("viewport_too_small");
  const outer = 16, gap = 12, stripH = 38, composerH = 48;
  const innerW = Math.max(0, W - outer * 2);
  const composerY = H - outer - composerH;
  const contentY = stripH + outer;
  const contentBottom = composerY - gap;
  const contentH = contentBottom - contentY;
  const canUseColumns = innerW >= 300 + gap + 180;
  let conversation, context, panelFlow;
  if (canUseColumns) {
    const usableW = innerW - gap;
    const conversationW = Math.floor(usableW * 2 / 3);
    const contextW = usableW - conversationW;
    panelFlow = "side-by-side";
    conversation = { x: outer, y: contentY, w: conversationW, h: contentH };
    context = { x: outer + conversationW + gap, y: contentY, w: contextW, h: contentH };
  } else {
    const usableH = contentH - gap;
    const contextH = Math.max(Math.floor(usableH / 3), 96);
    const conversationH = usableH - contextH;
    if (conversationH < 128) throw new Error("viewport_too_small");
    panelFlow = "stacked";
    conversation = { x: outer, y: contentY, w: innerW, h: conversationH };
    context = { x: outer, y: contentY + conversationH + gap, w: innerW, h: contextH };
  }
  const personal = { x: 0, y: stripH, w: W, h: H - stripH };
  const overlayW = Math.min(innerW, 480);
  const overlayH = Math.min(Math.max(0, personal.h - outer * 2), 280);
  return {
    W, H, physicalWidth, physicalHeight, panelFlow,
    strip: { x: 0, y: 0, w: W, h: stripH },
    conversation,
    context,
    composer: { x: outer, y: composerY, w: innerW, h: composerH },
    overlay: {
      x: Math.floor((W - overlayW) / 2),
      y: personal.y + Math.floor((personal.h - overlayH) / 2),
      w: overlayW,
      h: overlayH,
    },
    personal,
  };
}

let L = computeGenesisLayout(PHYSICAL_WIDTH, PHYSICAL_HEIGHT);

const PAL = {
  DREAM_BG:    "#0B0C0E",
  APP_BG:      "#14161A",
  SURFACE_BG:  "#1D2026",
  SURFACE_ALT: "#272B33",
  SCRIM:       "#0C0E12",
  HAIRLINE:    "#3E434C",
  TEXT_MAIN:   "#EEF1F5",
  TEXT_MUTED:  "#A9B1BD",
  TEXT_FAINT:  "#757E8C",
  APP_BLUE:    "#4897F2",
  APP_GREEN:   "#54BB7D",
  APP_AMBER:   "#DEA447",
  APP_RED:     "#DA5852",
};

// Mauszeiger-Form aus genesis.rs draw_current_cursor (X = Rand, O = Fuellung)
const POINTER_SHAPE = [
  "X", "XX", "XOX", "XOOX", "XOOOX", "XOOOOX", "XOOOOOX", "XOOOOOOX",
  "XOOOOOOOX", "XOOOOX", "XOOXOX", "XOXXOX", "XX  XOX", "X    XOX", "     XOX", "      X",
];
const POINTER_OUTLINE = "#05080C";

const state = {
  scenario: "genesis.dream.closed",
  dream: true,                // Aktueller Dream-Renderer; ?classic=1 -> klassische Genesis-Referenz
  labMode: "actual",         // actual | proposal (Vorschlag immer sichtbar markiert)
  labDiagnostics: false,
  dreamTab: "chat",           // chat | console | build
  buildB: 34,                 // Sim: Doppel-Bau, Lauf B in Prozent
  approved: false,
  hoverCenter: false,
  hoverId: null,
  hoverT: 0,
  layoutT: -1,                // -1 = closed (Ruhezustand), 0 = ambient, 1 = open
  layoutGoal: -1,
  view: "genesis",            // Genesis plus die in lab/scenarios.js katalogisierten echten Host-Zustaende
  pointer: null,              // physische Canvas-Koordinaten
  pointerDown: false,
  composerFocus: false,
  composerText: "",
  cursorOn: true,
  pressed: null,
  recoveryOpen: false,
  recoveryFacts: {
    posture: "Probation",
    problem: "Service health needs attention",
    lastGood: "slot A / generation 41",
    crashed: "none",
    disabled: "none",
  },
  keyboardLayout: "US",
  chat: [],                   // {speaker: "user"|"assistant"|"system", text}
  chatScroll: 0,
  wifiNetworks: [             // Sim-Daten im echten scan_line-Format
    { ssid: "home-fiber-5G", ch: 36, sec: "WPA2", tag: "scan" },
    { ssid: "workshop-24",   ch: 6,  sec: "WPA2", tag: "scan" },
    { ssid: "neighbor-guest", ch: 11, sec: "OPEN", tag: "scan" },
  ],
  wifiSelected: null,
  wifiPassword: "",
  wifiRejected: false,
  wifiProgress: 0,
  wifiConnected: false,
  wifiStorage: "vault",       // vault | legacy | open
  wifiRemember: false,
  wifiFailureReason: "wifi_device_not_detected",
  vaultMode: "closed",        // Namen entsprechen vault_flow.rs::Mode
  vaultMaskedLen: 24,
  vaultRecoveryKey: "RR1-7K4M-2P9Q-6V8X-3D5F-1H7J-9L2N-4R6T-8W3Y-5B7C-0E2G-6I8K-1M3P-5S7U-9X2Z",
  vaultManage: "forget-provider",
  vaultForgetTarget: "provider",
  vaultConfirm: "cancel",
  vaultOutcome: "provisioned",
  personalMode: null,          // proof | calculator | editor
  personalInputReceived: false,
  calculatorValue: 42,
  editorText: "The current-boot editor is rendered through the bounded personal surface.",
  overlayAnim: 1,             // 0..1 beim Oeffnen (Sim-Erfindung)
};
let hits = [];

const cv = document.getElementById("screen");
const ctx = cv.getContext("2d");
ctx.imageSmoothingEnabled = false;
const labHud = document.getElementById("lab-hud");
const labSelftest = document.getElementById("lab-selftest");
const labCatalog = document.getElementById("lab-catalog");
const labMetrics = {
  sceneFrames: 0,
  presents: 0,
  lastSceneMs: 0,
  lastPresentMs: 0,
};

// Zwei-Ebenen-Modell wie framebuffer.rs: Szene -> Back-Buffer (teuer, nur bei
// Zustandsaenderung), Cursor/Pointer -> Front-Pixel obendrauf (billig, ohne
// Neuzeichnen der Szene; Kernel: set_front_pixel + restore_from_back_rect).
const back = document.createElement("canvas");
back.width = cv.width;
back.height = cv.height;
const bctx = back.getContext("2d");
bctx.imageSmoothingEnabled = false;
let target = bctx;

// Im Website-Mockup rendert die UI in der tatsaechlich sichtbaren
// Display-Aufloesung. 1920x1080 bleibt die harte Obergrenze. Unter 1280x720
// bleibt die interne Flaeche bei 720p, weil Genesis mindestens 640x360
// logische Pixel fuer seine sicheren Layoutgrenzen benoetigt.
const WEBSITE_RENDER_MIN_WIDTH = 1280;
const WEBSITE_RENDER_MIN_HEIGHT = 720;

function rendererResolutionForDisplay(displayWidth) {
  const finiteWidth = Number.isFinite(displayWidth) ? displayWidth : PHYSICAL_WIDTH;
  const width = Math.max(
    WEBSITE_RENDER_MIN_WIDTH,
    Math.min(PHYSICAL_WIDTH, Math.round(finiteWidth / 16) * 16),
  );
  const height = Math.max(
    WEBSITE_RENDER_MIN_HEIGHT,
    Math.min(PHYSICAL_HEIGHT, Math.round((width * 9 / 16) / 2) * 2),
  );
  return { width, height };
}

function setRendererResolution(width, height) {
  const nextWidth = Math.max(
    WEBSITE_RENDER_MIN_WIDTH,
    Math.min(PHYSICAL_WIDTH, Math.round(width / 2) * 2),
  );
  const nextHeight = Math.max(
    WEBSITE_RENDER_MIN_HEIGHT,
    Math.min(PHYSICAL_HEIGHT, Math.round(height / 2) * 2),
  );
  if (cv.width === nextWidth && cv.height === nextHeight) return false;

  cv.width = nextWidth;
  cv.height = nextHeight;
  back.width = nextWidth;
  back.height = nextHeight;
  ctx.imageSmoothingEnabled = false;
  bctx.imageSmoothingEnabled = false;
  L = computeGenesisLayout(nextWidth, nextHeight);
  bgCache = null;
  state.pointer = null;
  if (window.__RAIOS_UI_LAB__) {
    window.__RAIOS_UI_LAB__.physicalSize = { width: nextWidth, height: nextHeight };
    window.__RAIOS_UI_LAB__.logicalSize = { width: L.W, height: L.H };
  }
  return true;
}
