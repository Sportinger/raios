// Website-only film clock for "The Factory Moves In".
//
// The SVG is deliberately declarative: this module owns time and state, while
// the document owns every shape. All hooks except the film root are optional so
// that the animatic and the finished artwork can share the same clock.

(function () {
  "use strict";

  const DURATION = 120;
  const POSTER_TIME = 118;
  const PROMPT = "> build me a music player";
  const SCENE_FADE = 0.55;
  const CAMERA_REFERENCE_WIDTH = 1200;
  const CAMERA_REFERENCE_HEIGHT = 680;
  const CAMERA_WIDE_FACTOR = 0.92;
  const CAMERA_VERTICAL_BIAS = 110;
  const SCROLL_TRIGGER_FRAME_Y = 0.18;
  const SCROLL_TRIGGER_VIEWPORT_Y = 0.38;
  const SCROLL_TRIGGER_TOLERANCE = 0.06;
  const SCROLL_PAGE_TRAVEL_VH = 0.20;
  const SCROLL_PAGE_TRAVEL_END_TIME = 6;
  const WHEEL_SCRUB_EASING_MS = 140;
  const WHEEL_MAX_LEAD_SECONDS = 8;
  const TIMELINE_REVEAL_TIME = 2.975;

  const sceneDefinitions = [
    [1, "sentence", "Prompt", 0, 6, 1, 1.15, 0.50, 0.49],
    [2, "world", "The world under the glass", 6, 14, 1, 1.35, 0.30, 0.47],
    [3, "net", "The agent needs the net", 14, 25, 1, 1.35, 0.30, 0.47],
    [4, "build-site", "Unlock the builder deck", 25, 33, 2, 0.82, 0.50, 0.50],
    [5, "inert-source", "Material becomes one workpiece", 33, 41, 2, 1.02, 0.72, 0.48],
    [6, "compiler", "Compiler round one", 41, 52, 2, 1.12, 0.74, 0.49],
    [7, "feedback", "Red report, precise fix", 52, 62, 2, 0.98, 0.58, 0.49],
    [8, "twins", "Twin build, byte equal", 62, 72, 2, 1.08, 0.74, 0.49],
    [9, "proof-cellar", "Attack drills attach proof", 72, 80, 2, 1.08, 0.76, 0.51],
    [10, "ring", "Owner approval guards out", 80, 88, 3, 1.28, 0.50, 0.49],
    [11, "approval", "Out opens the player domain", 88, 96, 3, 0.98, 0.58, 0.49],
    [12, "running", "Player lands beside Genesis", 96, 106, 3, 1.38, 0.57, 0.49],
    [13, "factory", "The builder folds away", 106, 114, 3, 0.92, 0.51, 0.50],
    [14, "loop", "The loop", 114, 120, 3, 0.84, 0.50, 0.50],
  ];

  const SCENES = Object.freeze(sceneDefinitions.map((definition) => Object.freeze({
    number: definition[0],
    id: definition[1],
    title: definition[2],
    start: definition[3],
    end: definition[4],
    act: definition[5],
    camera: Object.freeze({
      scale: definition[6],
      focusX: definition[7],
      focusY: definition[8],
    }),
  })));

  // Extra points inside screen-heavy scenes keep camera moves continuous while
  // all scene boundaries remain visible in SCENES and easy to retime.
  const CAMERA_KEYFRAMES = Object.freeze([
    { at: 0, scale: 1.15, focusX: 0.50, focusY: 0.49 },
    { at: 5.2, scale: 1.15, focusX: 0.50, focusY: 0.49 },
    { at: 13.4, scale: 1.35, focusX: 0.30, focusY: 0.47 },
    { at: 26.15, scale: 1.35, focusX: 0.30, focusY: 0.47 },
    { at: 28.5, scale: 0.78, focusX: 0.50, focusY: 0.50 },
    { at: 33, scale: 0.82, focusX: 0.50, focusY: 0.50 },
    { at: 41, scale: 1.02, focusX: 0.72, focusY: 0.48 },
    { at: 45.8, scale: 1.12, focusX: 0.74, focusY: 0.49 },
    { at: 50.6, scale: 1.12, focusX: 0.74, focusY: 0.49 },
    { at: 52, scale: 0.98, focusX: 0.58, focusY: 0.49 },
    { at: 58.4, scale: 0.98, focusX: 0.58, focusY: 0.49 },
    { at: 62, scale: 1.05, focusX: 0.72, focusY: 0.49 },
    { at: 72, scale: 1.08, focusX: 0.75, focusY: 0.50 },
    { at: 78.6, scale: 1.08, focusX: 0.75, focusY: 0.50 },
    { at: 80, scale: 1.22, focusX: 0.58, focusY: 0.47 },
    { at: 81.2, scale: 1.28, focusX: 0.50, focusY: 0.49 },
    { at: 86.6, scale: 1.28, focusX: 0.50, focusY: 0.49 },
    { at: 88, scale: 0.98, focusX: 0.58, focusY: 0.49 },
    { at: 94.7, scale: 0.98, focusX: 0.58, focusY: 0.49 },
    { at: 96, scale: 1.10, focusX: 0.57, focusY: 0.46 },
    { at: 99.2, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 104.6, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 106, scale: 0.92, focusX: 0.51, focusY: 0.50 },
    { at: 113.4, scale: 0.92, focusX: 0.51, focusY: 0.50 },
    { at: 114.1, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 116.45, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 117.15, scale: 0.84, focusX: 0.50, focusY: 0.50 },
    { at: 118.3, scale: 0.84, focusX: 0.50, focusY: 0.50 },
    { at: 120, scale: 1.15, focusX: 0.50, focusY: 0.49 },
  ].map(Object.freeze));

  const clamp = (value, min, max) => Math.min(max, Math.max(min, value));
  const lerp = (a, b, amount) => a + (b - a) * amount;
  const smoothstep = (amount) => {
    const value = clamp(amount, 0, 1);
    return value * value * (3 - 2 * value);
  };
  const smootherstep = (amount) => {
    const value = clamp(amount, 0, 1);
    return value * value * value * (value * (value * 6 - 15) + 10);
  };
  const intervalProgress = (time, start, end) => {
    if (end <= start) return time >= end ? 1 : 0;
    return clamp((time - start) / (end - start), 0, 1);
  };

  function sceneAt(time) {
    if (time >= DURATION) return SCENES[SCENES.length - 1];
    return SCENES.find((scene) => time >= scene.start && time < scene.end) || SCENES[0];
  }

  function parseTime(value, fallback) {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? clamp(parsed, 0, DURATION) : fallback;
  }

  function parseWindow(element) {
    const raw = element.getAttribute("data-film-window")
      || element.getAttribute("data-window")
      || "";
    const numbers = raw.match(/(?:\d+(?:\.\d*)?|\.\d+)/g) || [];
    if (numbers.length === 0) return null;
    const start = parseTime(numbers[0], 0);
    const end = numbers.length > 1 ? parseTime(numbers[1], DURATION) : DURATION;
    const fadeValue = numbers.length > 2
      ? numbers[2]
      : element.dataset.filmFade || element.dataset.fade || "0.35";
    const fade = Math.max(0, Number.parseFloat(fadeValue) || 0);
    return { start, end, fade };
  }

  function parseAction(element) {
    const raw = element.getAttribute("data-film-action")
      || element.getAttribute("data-action")
      || "";
    const parts = raw.split(",").map((part) => part.trim()).filter(Boolean);
    const start = parseTime(parts[0], 0);
    const end = parseTime(parts[1], start);
    const kind = (parts[2] || parts[0] || "reveal").toLowerCase();
    return { start, end, kind };
  }

  function windowState(time, windowDefinition) {
    if (!windowDefinition) return { active: true, alpha: 1, progress: 1 };
    const { start, end, fade } = windowDefinition;
    const progress = intervalProgress(time, start, end);
    let active;
    if (start <= end) active = time >= start && time <= end;
    else active = time >= start || time <= end;
    if (!active) return { active: false, alpha: 0, progress };
    if (start > end) return { active: true, alpha: 1, progress };
    const fadeIn = fade > 0 ? smoothstep((time - start) / fade) : 1;
    const fadeOut = fade > 0 ? smoothstep((end - time) / fade) : 1;
    return { active: true, alpha: Math.min(fadeIn, fadeOut), progress };
  }

  function cameraAt(time) {
    let before = CAMERA_KEYFRAMES[0];
    let after = CAMERA_KEYFRAMES[CAMERA_KEYFRAMES.length - 1];
    for (let index = 1; index < CAMERA_KEYFRAMES.length; index += 1) {
      if (time <= CAMERA_KEYFRAMES[index].at) {
        after = CAMERA_KEYFRAMES[index];
        before = CAMERA_KEYFRAMES[index - 1];
        break;
      }
    }
    const progress = smoothstep(intervalProgress(time, before.at, after.at));
    return {
      scale: lerp(before.scale, after.scale, progress),
      focusX: lerp(before.focusX, after.focusX, progress),
      focusY: lerp(before.focusY, after.focusY, progress),
    };
  }

  function unavailableApi() {
    let time = 0;
    const api = {
      ready: false,
      duration: DURATION,
      posterTime: POSTER_TIME,
      scenes: SCENES,
      cameraKeyframes: CAMERA_KEYFRAMES,
      getTime: () => time,
      setTime: (next) => { time = parseTime(next, time); return time; },
      seek: (next) => api.setTime(next),
      renderAt: (next) => api.setTime(next),
      render: () => time,
      pause: () => time,
      play: () => false,
      refresh: () => false,
    };
    Object.defineProperties(api, {
      time: { get: () => time },
      scene: { get: () => sceneAt(time) },
      playing: { get: () => false },
    });
    return api;
  }

  function initialiseFilm() {
    const svg = document.getElementById("film-svg") || document.getElementById("film-animatic");
    const root = document.getElementById("film-animatic") || svg;
    if (!svg || !root) {
      window.__RAIOS_FILM__ = unavailableApi();
      return;
    }

    const params = new URLSearchParams(location.search);
    if (params.get("film") === "isolate") document.body.classList.add("film-isolate");
    const reduceMotionQuery = typeof window.matchMedia === "function"
      ? window.matchMedia("(prefers-reduced-motion: reduce)")
      : null;
    const deterministic = params.get("anim") === "0";
    const reducedMotion = Boolean(reduceMotionQuery && reduceMotionQuery.matches);
    const requestedTime = params.has("animt")
      ? parseTime(params.get("animt"), 0)
      : deterministic
        ? 0
        : reducedMotion
          ? POSTER_TIME
          : 0;

    let currentTime = requestedTime;
    let playing = false;
    let wantsPlayback = false;
    let suspendedByMode = false;
    let animationFrame = 0;
    let anchorTime = currentTime;
    let anchorNow = performance.now();
    let sceneNodes = [];
    let windowNodes = [];
    let actionNodes = [];
    let pulseNodes = [];
    let doorLeaves = [];
    let terminalLines = [];
    let cycleNodes = [];
    let teardownNodes = [];
    let hooks = {};
    const baseTransforms = new WeakMap();
    let scrubber = null;
    let scrubRange = null;
    let scrubOutput = null;
    let scrubPlay = null;
    let scrubMarks = [];

    function selectOne() {
      for (const selector of arguments) {
        const node = root.querySelector(selector) || document.querySelector(selector);
        if (node) return node;
      }
      return null;
    }

    function selectAll(selector) {
      const local = Array.from(root.querySelectorAll(selector));
      if (root.matches && root.matches(selector)) local.unshift(root);
      return local;
    }

    function refresh() {
      sceneNodes = SCENES.map((scene) => selectOne(
        `[data-film-scene="${scene.number}"]`,
        `#film-scene-${String(scene.number).padStart(2, "0")}`,
        `#film-scene-${scene.number}`,
        `#film-s${scene.number}`,
      ));
      windowNodes = selectAll("[data-film-window], [data-window]").map((node) => ({
        node,
        definition: parseWindow(node),
      }));
      actionNodes = selectAll("[data-film-action], [data-action]");
      pulseNodes = selectAll("[data-film-pulse], .film-vein-pulse, .film-pulse");
      doorLeaves = selectAll("[data-film-door-leaf]");
      terminalLines = selectAll("[data-film-line], [data-film-terminal-line]").map((node) => ({
        node,
        at: parseTime(node.dataset.filmLine || node.dataset.filmTerminalLine, 0),
      }));
      cycleNodes = selectAll("[data-film-cycle]");
      if (cycleNodes.length === 0) cycleNodes = selectAll(".film-loop-node");
      teardownNodes = selectAll("[data-film-teardown]").map((node) => {
        const values = (node.dataset.filmTeardown || "0,0,0,0").split(",").map(Number);
        return {
          node,
          start: values[0] || 0,
          end: values[1] || values[0] || 0,
          dx: values[2] || 0,
          dy: values[3] || 0,
          managedBefore: !node.matches("[data-film-window], [data-window], [data-film-action], [data-action]"),
        };
      });
      const twinCells = selectAll(".film-twin-cell");
      const twinHashes = selectAll(".film-twin-cell .film-hash");
      const ringSegments = selectAll("#film-ring .film-ring-segment");
      hooks = {
        camera: selectOne("#film-cam", "#cam"),
        promptShell: selectOne(".film-prompt-only"),
        prompt: selectOne("#film-prompt-text", "#film-prompt", "[data-film-prompt]"),
        promptCursor: selectOne("#film-prompt-cursor", "[data-film-prompt-cursor]"),
        importCount: selectOne("#film-import-count"),
        pageCount: selectOne("#film-pages-count", "#film-page-count"),
        terminal: selectOne("#film-terminal-text"),
        buildA: selectOne("#film-build-a") || twinCells[0] || null,
        buildB: selectOne("#film-build-b") || twinCells[1] || null,
        hashA: selectOne("#film-hash-a") || twinHashes[0] || null,
        hashB: selectOne("#film-hash-b") || twinHashes[1] || null,
        equalGroup: selectOne("#film-twin-equal", "#film-equal"),
        equalText: selectOne("#film-twin-equal text", "#film-equal-text", "#film-equal"),
        ringManifest: selectOne("#film-ring-manifest") || ringSegments[0] || null,
        ringHash: selectOne("#film-ring-hash") || ringSegments[1] || null,
        ringReport: selectOne("#film-ring-report") || ringSegments[2] || null,
        ringApproval: selectOne("#film-ring-approval") || ringSegments[3] || null,
        approvalPointer: selectOne("#film-approval-pointer"),
        approvalButton: selectOne("#film-approval-button", "#film-approve-button"),
        remoteDenied: selectOne("#film-remote-denied", ".film-remote-denied"),
        netVein: selectOne("#film-net-vein", "#film-net .film-pulse"),
        reclog: selectOne("#film-reclog"),
        titleCard: selectOne("#film-title-card", ".film-title"),
        titleSubtitle: selectOne(".film-subtitle"),
        sceneNumber: selectOne("#film-scene-number"),
        sceneName: selectOne("#film-scene-name"),
        timecode: selectOne("#film-timecode"),
        timelineProgress: selectOne("#film-progress"),
        fuelCount: selectOne("#film-fuel-count"),
        fuelRing: selectOne(".film-fuel"),
        trackProgress: selectOne("#film-track-progress"),
        crashNeighbor: selectOne("#film-crash-neighbor"),
        trapFlash: selectOne("#film-trap-flash"),
        cutCable: selectOne("#film-cut-cable"),
        badVersion: selectOne("#film-bad-version"),
        builderDeck: selectOne("#film-builder-deck"),
        builderFloor: selectOne("#film-builder-deck .film-builder-floor"),
        builderHatch: selectOne("#film-builder-deck .film-builder-hatch"),
        kernelSide: selectOne("#film-kernel-side"),
        kernelFacet: selectOne("#film-kernel-facet"),
        kernelGlow: selectOne("#film-kernel-glow"),
        kernelCalloutCard: selectOne("#film-kernel-callout .film-callout-card"),
        genesisCalloutCard: selectOne("#film-genesis-callout .film-callout-card"),
        kernelLayerTitle: selectOne("#film-kernel-layer-title"),
        genesisLayerTitle: selectOne("#film-genesis-layer-title"),
        compilerProgress: selectOne("#film-compiler-progress"),
        compilerRound: selectOne("#film-compiler-round"),
        verifierProgress: selectOne("#film-verifier-progress"),
        verifierRound: selectOne("#film-verifier-round"),
        workpiece: selectOne("#film-workpiece"),
        workpieceLabel: selectOne("#film-workpiece-label"),
        sealTwin: selectOne("#film-seal-twin"),
        sealDrills: selectOne("#film-seal-drills"),
        sealReport: selectOne("#film-seal-report"),
        sealReportText: selectOne("#film-seal-report text"),
        outGateStatus: selectOne("#film-out-gate-status"),
        outGateCopy: selectOne("#film-out-gate-copy"),
        outDoor: selectOne("#film-out-door"),
        drillWall: selectOne("#film-drill-wall"),
        drillImport: selectOne("#film-drill-import"),
        drillFuel: selectOne("#film-drill-fuel"),
        workshopLinks: selectAll("[data-film-workshop-link]"),
        materialPath: selectOne("#film-material-path"),
        materialMain: selectOne("#film-material-main"),
        materialCargo: selectOne("#film-material-cargo"),
        compilerFeedbackPath: selectOne("#film-compiler-feedback-path"),
        verifierFeedbackPath: selectOne("#film-verifier-feedback-path"),
        errorReportOne: selectOne("#film-error-report-one"),
        errorReportTwo: selectOne("#film-error-report-two"),
        editOne: selectOne("#film-edit-one"),
        editTwo: selectOne("#film-edit-two"),
        shutdownPath: selectOne("#film-shutdown-path"),
        shutdownOrder: selectOne("#film-shutdown-order"),
        playerDomain: selectOne("#film-player-domain"),
        finalPlayer: selectOne("#film-final-player"),
      };
      actionNodes.concat(teardownNodes.map((entry) => entry.node), [hooks.approvalPointer, hooks.crashNeighbor].filter(Boolean)).forEach((node) => {
        if (!baseTransforms.has(node)) baseTransforms.set(node, node.getAttribute("transform") || "");
      });
      return true;
    }

    function setOpacity(node, opacity) {
      if (!node) return;
      const value = clamp(opacity, 0, 1);
      node.style.opacity = value.toFixed(3);
      node.style.visibility = value <= 0.001 ? "hidden" : "visible";
    }

    function setText(node, text) {
      if (node && node.textContent !== text) node.textContent = text;
    }

    function setProgress(node, progress) {
      if (!node) return;
      const value = clamp(progress, 0, 1);
      node.style.setProperty("--film-progress", value.toFixed(4));
      node.dataset.filmProgress = value.toFixed(3);
    }

    function setComposedTransform(node, actionTransform) {
      if (!node) return;
      if (!baseTransforms.has(node)) baseTransforms.set(node, node.getAttribute("transform") || "");
      const base = baseTransforms.get(node);
      node.setAttribute("transform", `${base}${base && actionTransform ? " " : ""}${actionTransform}`.trim());
    }

    function renderScenes(time, activeScene) {
      sceneNodes.forEach((node, index) => {
        if (!node) return;
        const scene = SCENES[index];
        let opacity = 0;
        if (time >= scene.start && time <= scene.end) {
          opacity = scene.number === 1
            ? 1
            : smoothstep((time - scene.start) / SCENE_FADE);
        } else if (time > scene.end && time < scene.end + SCENE_FADE) {
          opacity = 1 - smoothstep((time - scene.end) / SCENE_FADE);
        }
        setOpacity(node, opacity);
        node.style.pointerEvents = scene === activeScene ? "auto" : "none";
        node.dataset.filmActive = String(scene === activeScene);
        node.setAttribute("aria-hidden", String(opacity <= 0.001));
      });
    }

    function renderCamera(time) {
      if (!hooks.camera) return;
      const camera = cameraAt(time);
      const cameraScale = camera.scale * CAMERA_WIDE_FACTOR;
      const viewBox = svg.viewBox && svg.viewBox.baseVal;
      const boxX = viewBox && viewBox.width ? viewBox.x : 0;
      const boxY = viewBox && viewBox.height ? viewBox.y : 0;
      const boxWidth = viewBox && viewBox.width ? viewBox.width : 1000;
      const boxHeight = viewBox && viewBox.height ? viewBox.height : 640;
      const centerX = boxX + boxWidth / 2;
      const centerY = boxY + boxHeight / 2;
      const focusX = CAMERA_REFERENCE_WIDTH * camera.focusX;
      const focusY = CAMERA_REFERENCE_HEIGHT * camera.focusY + CAMERA_VERTICAL_BIAS;
      const translateX = centerX - focusX * cameraScale;
      const translateY = centerY - focusY * cameraScale;
      hooks.camera.setAttribute(
        "transform",
        `translate(${translateX.toFixed(3)} ${translateY.toFixed(3)}) scale(${cameraScale.toFixed(5)})`,
      );
      hooks.camera.style.setProperty("--film-camera-scale", cameraScale.toFixed(5));
    }

    function renderWindows(time) {
      for (const entry of windowNodes) {
        const state = windowState(time, entry.definition);
        entry.node.style.setProperty("--film-window-progress", state.progress.toFixed(4));
        entry.node.style.setProperty("--film-window-alpha", state.alpha.toFixed(4));
        entry.node.dataset.filmActive = String(state.active);
        if (entry.node.dataset.filmWindowOpacity !== "manual") {
          setOpacity(entry.node, state.alpha);
        }
      }
    }

    function renderDeclarativeActions(time) {
      for (const node of actionNodes) {
        const actionDefinition = parseAction(node);
        const progress = smoothstep(intervalProgress(time, actionDefinition.start, actionDefinition.end));
        const started = time >= actionDefinition.start;
        const action = actionDefinition.kind;
        node.style.setProperty("--film-action-progress", progress.toFixed(4));
        node.style.setProperty("--film-action-opacity", String(started ? progress : 0));
        node.dataset.filmActionState = !started ? "waiting" : progress < 1 ? "moving" : "complete";
        if (action === "rise" || action === "drop" || action === "stamp" || action === "grant") {
          let offsetY = 0;
          let scale = 1;
          let rotation = 0;
          if (action === "rise") {
            const distance = Number.parseFloat(node.dataset.filmDistance || "38") || 38;
            const isometricGrow = node.dataset.filmGrow === "iso";
            if (isometricGrow) {
              node.style.clipPath = "none";
              node.style.setProperty("--film-iso-reveal", progress.toFixed(4));
              offsetY = distance * (1 - progress);
              scale = 1;
            } else {
              node.style.clipPath = "none";
              offsetY = distance * (1 - progress);
              scale = 0.94 + progress * 0.06;
            }
          } else if (action === "drop") {
            offsetY = -48 * (1 - progress);
            scale = 0.96 + progress * 0.04;
          } else if (action === "stamp") {
            const impact = smoothstep(intervalProgress(progress, 0, 0.72));
            const rebound = progress > 0.72 ? Math.sin((progress - 0.72) / 0.28 * Math.PI) * -5 : 0;
            offsetY = -64 * (1 - impact) + rebound;
            scale = 0.95 + impact * 0.05;
          } else {
            const insert = smoothstep(intervalProgress(progress, 0, 0.68));
            const turn = smoothstep(intervalProgress(progress, 0.68, 1));
            offsetY = 64 * (1 - insert);
            rotation = -58 * turn;
            scale = 0.9 + insert * 0.1;
          }
          node.style.setProperty("--film-action-y", `${offsetY.toFixed(2)}px`);
          node.style.setProperty("--film-action-scale", scale.toFixed(4));
          setComposedTransform(node, `translate(0 ${offsetY.toFixed(3)}) rotate(${rotation.toFixed(3)} 40 16) scale(${scale.toFixed(5)})`);
          const isometricGrow = action === "rise" && node.dataset.filmGrow === "iso";
          setOpacity(node, started ? Math.min(1, progress * (isometricGrow ? 3.2 : 2.5)) : 0);
        } else if (action === "type") {
            const text = node.dataset.filmText || "";
            const count = Math.floor(text.length * progress + 0.0001);
            setText(node, text.slice(0, count));
        } else if (action === "counter") {
            const from = Number.parseFloat(node.dataset.filmFrom || "0");
            const to = Number.parseFloat(node.dataset.filmTo || "1");
            const precision = Math.max(0, Number.parseInt(node.dataset.filmPrecision || "0", 10) || 0);
            const value = lerp(from, to, progress).toFixed(precision);
            setText(node, (node.dataset.filmFormat || "{value}").replace("{value}", value));
        } else if (action === "pulse" || action === "dash") {
            const speed = Number.parseFloat(node.dataset.filmSpeed || "22") || 22;
            const direction = node.dataset.filmDirection === "reverse" ? -1 : 1;
            const offset = -time * speed * direction;
            node.style.strokeDashoffset = offset.toFixed(2);
            node.style.setProperty("--film-dash-offset", offset.toFixed(2));
        } else if (action === "reveal" || action === "opacity") {
          setOpacity(node, started ? progress : 0);
        }
      }
    }

    function renderDoorCeremonies(time) {
      doorLeaves.forEach((leaf) => {
        const openAt = Number.parseFloat(leaf.dataset.filmDoorAt || "0") || 0;
        const progress = smoothstep(intervalProgress(time, openAt, openAt + 0.9));
        const mix = (from, to, amount) => from + (to - from) * amount;
        const hingeTop = { x: 20, y: 12 };
        const hingeBottom = { x: 20, y: 68 };
        const freeTop = { x: mix(66, -18, progress), y: mix(12, 20, progress) };
        const freeBottom = { x: mix(66, -18, progress), y: mix(68, 76, progress) };
        const pointOnLeaf = (across, down) => {
          const hingeX = mix(hingeTop.x, hingeBottom.x, down);
          const hingeY = mix(hingeTop.y, hingeBottom.y, down);
          const freeX = mix(freeTop.x, freeBottom.x, down);
          const freeY = mix(freeTop.y, freeBottom.y, down);
          return { x: mix(hingeX, freeX, across), y: mix(hingeY, freeY, across) };
        };
        const pathPoint = (point) => `${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
        const panel = leaf.querySelector("[data-film-door-panel]");
        if (panel) {
          panel.setAttribute("d", `M${pathPoint(hingeBottom)}L${pathPoint(hingeTop)}L${pathPoint(freeTop)}L${pathPoint(freeBottom)}Z`);
        }
        const detail = leaf.querySelector("[data-film-door-detail]");
        if (detail) {
          const detailBottomLeft = pointOnLeaf(0.12, 0.89);
          const detailTopLeft = pointOnLeaf(0.12, 0.11);
          const detailTopRight = pointOnLeaf(0.88, 0.11);
          const detailBottomRight = pointOnLeaf(0.88, 0.89);
          detail.setAttribute("d", `M${pathPoint(detailBottomLeft)}L${pathPoint(detailTopLeft)}L${pathPoint(detailTopRight)}L${pathPoint(detailBottomRight)}Z`);
        }
        const handle = leaf.querySelector("[data-film-door-handle]");
        if (handle) {
          const handlePoint = pointOnLeaf(0.84, 0.55);
          handle.setAttribute("cx", handlePoint.x.toFixed(2));
          handle.setAttribute("cy", handlePoint.y.toFixed(2));
        }
        leaf.style.setProperty("--film-door-progress", progress.toFixed(4));
        leaf.dataset.filmDoorOpen = String(progress >= 1);
        if (leaf.parentElement) leaf.parentElement.dataset.filmDoorOpen = String(progress >= 1);
      });
    }

    function renderPrompt(time) {
      if (hooks.promptShell) {
        const callProgress = intervalProgress(time, 0.08, 1.02);
        const shifted = callProgress - 1;
        const backEase = 1 + 2.70158 * shifted * shifted * shifted
          + 1.70158 * shifted * shifted;
        const callScale = 0.78 + 0.22 * backEase;
        const callLift = 26 * (1 - smoothstep(callProgress));
        const callOpacity = smoothstep(intervalProgress(time, 0.08, 0.52));
        setComposedTransform(
          hooks.promptShell,
          `translate(600 334) translate(0 ${callLift.toFixed(3)}) scale(${callScale.toFixed(5)}) translate(-600 -334)`,
        );
        setOpacity(hooks.promptShell, callOpacity);
        hooks.promptShell.dataset.filmCalled = String(callProgress >= 1);
        hooks.promptShell.style.setProperty("--film-prompt-call", callProgress.toFixed(4));
      }
      if (hooks.prompt) {
        const progress = intervalProgress(time, 1.15, 4.8);
        const count = Math.floor(PROMPT.length * progress + 0.0001);
        setText(hooks.prompt, PROMPT.slice(0, count));
        hooks.prompt.dataset.filmSubmitted = String(time >= 5.1);
      }
      if (hooks.promptCursor) {
        const blink = time >= 0.82 && time < 5.15 && Math.floor(time * 2.4) % 2 === 0;
        setOpacity(hooks.promptCursor, blink ? 1 : 0);
        const promptX = hooks.prompt ? Number.parseFloat(hooks.prompt.getAttribute("x") || "270") : 270;
        const promptWidth = hooks.prompt && typeof hooks.prompt.getComputedTextLength === "function"
          ? hooks.prompt.getComputedTextLength()
          : 0;
        hooks.promptCursor.setAttribute("x", (promptX + promptWidth + 7).toFixed(2));
      }
    }

    function renderLayerCallouts(time) {
      const descent = smoothstep(intervalProgress(time, 12.05, 12.85));
      const docking = smoothstep(intervalProgress(time, 12.85, 13.85));
      const cardOpacity = 1 - smoothstep(intervalProgress(time, 13.1, 13.72));
      const titleOpacity = smoothstep(intervalProgress(time, 13.18, 13.85));
      const cardTransform = (x, y, angle, pivotX, pivotY) => {
        const offsetX = x * docking;
        const offsetY = 120 * descent + (y - 120) * docking;
        return `translate(${offsetX.toFixed(3)} ${offsetY.toFixed(3)}) rotate(${(angle * docking).toFixed(3)} ${pivotX} ${pivotY})`;
      };

      setComposedTransform(
        hooks.kernelCalloutCard,
        cardTransform(226, 332, 26.565, -73, 128),
      );
      setComposedTransform(
        hooks.genesisCalloutCard,
        cardTransform(-42, 262, -26.565, 542, 128),
      );
      setOpacity(hooks.kernelCalloutCard, cardOpacity);
      setOpacity(hooks.genesisCalloutCard, cardOpacity);
      setOpacity(hooks.kernelLayerTitle, titleOpacity);
      setOpacity(hooks.genesisLayerTitle, titleOpacity);
    }

    function renderBuilderArchitecture(time) {
      const networkExpansion = smoothstep(intervalProgress(time, 15.95, 17.35));
      const builderExpansion = smoothstep(intervalProgress(time, 24.85, 27.15));
      const stageValue = (initial, network, builder) => (
        lerp(lerp(initial, network, networkExpansion), builder, builderExpansion)
      );
      const point = (initialX, initialY, networkX, networkY, builderX, builderY) => (
        `${stageValue(initialX, networkX, builderX).toFixed(2)} ${stageValue(initialY, networkY, builderY).toFixed(2)}`
      );
      const left = point(0, 345, -40, 360, 24, 360);
      const top = point(315, 187.5, 420, 130, 600, 72);
      const right = point(630, 345, 880, 360, 1176, 360);
      const bottom = point(315, 502.5, 420, 590, 600, 648);
      const leftLower = point(0, 415, -40, 420, 24, 420);
      const rightLower = point(630, 415, 880, 420, 1176, 420);
      const bottomLower = point(315, 572.5, 420, 650, 600, 708);
      if (hooks.kernelFacet) hooks.kernelFacet.setAttribute("d", `M${left}L${top}L${right}L${bottom}Z`);
      if (hooks.kernelSide) hooks.kernelSide.setAttribute("d", `M${left}L${bottom}L${right}L${rightLower}L${bottomLower}L${leftLower}Z`);
      if (hooks.kernelGlow) {
        hooks.kernelGlow.setAttribute("cx", stageValue(315, 420, 600).toFixed(2));
        hooks.kernelGlow.setAttribute("cy", stageValue(560, 620, 670).toFixed(2));
        hooks.kernelGlow.setAttribute("rx", stageValue(310, 430, 535).toFixed(2));
        hooks.kernelGlow.setAttribute("ry", stageValue(38, 40, 44).toFixed(2));
      }

      if (hooks.builderDeck && time >= 29.7) {
        const rest = smoothstep(intervalProgress(time, 112.4, 114.0));
        hooks.builderDeck.dataset.filmResting = String(rest >= 1);
        hooks.builderDeck.style.setProperty("--film-builder-rest", rest.toFixed(4));
      }
      const floorOnline = smoothstep(intervalProgress(time, 30.15, 31.15));
      setOpacity(hooks.builderHatch, 1 - floorOnline);
      if (hooks.builderFloor) hooks.builderFloor.dataset.filmOnline = String(floorOnline >= 1);
    }

    function renderCompiler(time) {
      const importProgress = smoothstep(intervalProgress(time, 43.2, 48.4));
      const importCount = Math.max(1, Math.round(30 * importProgress));
      setText(hooks.importCount, String(importCount));
      const pageProgress = smoothstep(intervalProgress(time, 43.2, 49.2));
      const pages = Math.round(399 + (512 - 399) * pageProgress);
      setText(hooks.pageCount, String(pages));

      const terminalContent = [
        [43.8, "$ rustc --version"],
        [44.7, "rustc 1.83.0-dev"],
        [45.7, "$ rustc hello.rs --target wasm32-wasip1"],
        [47.0, "parse · resolve std · typecheck · emit"],
        [50.1, "/out/hello.wasm"],
      ];
      if (hooks.terminal) {
        setText(
          hooks.terminal,
          terminalContent.filter((line) => time >= line[0]).map((line) => line[1]).join("\n"),
        );
      }
      terminalLines.forEach((entry) => {
        setOpacity(entry.node, smoothstep(intervalProgress(time, entry.at, entry.at + 0.28)));
      });
    }

    function renderWorkshop(time) {
      (hooks.workshopLinks || []).forEach((node) => {
        const [startText, endText] = (node.dataset.filmWorkshopLink || "0,0").split(",");
        const progress = smoothstep(intervalProgress(time, Number(startText), Number(endText)));
        setOpacity(node, progress);
      });
      const placeOnPath = (node, path, start, end, options = {}) => {
        if (!node || !path || typeof path.getTotalLength !== "function") return;
        const progress = smoothstep(intervalProgress(time, start, end));
        const length = path.getTotalLength();
        const point = path.getPointAtLength(length * progress);
        const enter = smoothstep(intervalProgress(time, start - 0.2, start + 0.18));
        const absorb = 1 - smoothstep(intervalProgress(time, end - 0.34, end));
        const startScale = options.startScale ?? 0.55;
        const endScale = options.endScale ?? 1;
        const scale = startScale + (endScale - startScale) * absorb;
        setComposedTransform(
          node,
          `translate(${point.x.toFixed(2)} ${point.y.toFixed(2)}) scale(${scale.toFixed(4)})`,
        );
        setOpacity(node, enter * absorb);
      };
      placeOnPath(hooks.materialMain, hooks.materialPath, 33.0, 36.3);
      placeOnPath(hooks.materialCargo, hooks.materialPath, 33.8, 37.2);
      placeOnPath(hooks.errorReportOne, hooks.compilerFeedbackPath, 46.0, 49.4, {
        startScale: 0.72,
        endScale: 0.92,
      });
      placeOnPath(hooks.editOne, hooks.materialPath, 49.95, 52.55, {
        startScale: 0.7,
        endScale: 0.92,
      });
      placeOnPath(hooks.errorReportTwo, hooks.verifierFeedbackPath, 60.8, 63.85, {
        startScale: 0.72,
        endScale: 0.92,
      });
      placeOnPath(hooks.editTwo, hooks.materialPath, 64.4, 66.95, {
        startScale: 0.7,
        endScale: 0.92,
      });
      const route = [
        { at: 33.0, x: 900, y: 345, scale: 0.54 },
        { at: 36.2, x: 900, y: 345, scale: 1 },
        { at: 41.0, x: 900, y: 345, scale: 1 },
        { at: 42.0, x: 880, y: 320, scale: 1 },
        { at: 46.2, x: 880, y: 320, scale: 1 },
        { at: 49.4, x: 900, y: 345, scale: 1 },
        { at: 52.55, x: 900, y: 345, scale: 1 },
        { at: 55.8, x: 880, y: 320, scale: 1 },
        { at: 58.0, x: 1025, y: 390, scale: 1 },
        { at: 60.8, x: 1025, y: 390, scale: 1 },
        { at: 63.85, x: 900, y: 345, scale: 1 },
        { at: 66.95, x: 900, y: 345, scale: 1 },
        { at: 69.7, x: 880, y: 320, scale: 1 },
        { at: 71.2, x: 1025, y: 390, scale: 1 },
        { at: 78.6, x: 1025, y: 390, scale: 1 },
        { at: 88.0, x: 1025, y: 390, scale: 1 },
        { at: 89.6, x: 751, y: 294, scale: 1 },
        { at: 92.0, x: 607.5, y: 200, scale: 1 },
        { at: 120.0, x: 607.5, y: 200, scale: 1 },
      ];
      const before = route.reduce((best, frame) => (frame.at <= time ? frame : best), route[0]);
      const after = route.find((frame) => frame.at > time) || route[route.length - 1];
      const routeProgress = before === after
        ? 1
        : smoothstep(intervalProgress(time, before.at, after.at));
      const workpieceX = lerp(before.x, after.x, routeProgress);
      const workpieceY = lerp(before.y, after.y, routeProgress);
      const workpieceScale = lerp(before.scale, after.scale, routeProgress);
      const workpieceAlpha = windowState(time, { start: 33, end: 120, fade: 0.5 }).alpha;
      if (hooks.workpiece) {
        setComposedTransform(
          hooks.workpiece,
          `translate(${workpieceX.toFixed(2)} ${workpieceY.toFixed(2)}) scale(${workpieceScale.toFixed(4)})`,
        );
        setOpacity(hooks.workpiece, workpieceAlpha);
        const state = time >= 92.0
          ? "resident"
          : time >= 88.0
            ? "egress"
            : time >= 77.4
              ? "proved"
              : (time >= 49.4 && time < 52.55) || (time >= 63.85 && time < 66.95)
                ? "editing"
                : (time >= 46.0 && time < 49.4) || (time >= 60.8 && time < 63.85)
                  ? "failed"
                  : "building";
        hooks.workpiece.dataset.filmState = state;
      }
      if (hooks.workpieceLabel) {
        const label = time >= 92.0
          ? "PLAYER.RS"
          : time >= 84.55
            ? "PLAYER · APPROVED"
            : time >= 77.4
              ? "PLAYER · PROVED"
              : time >= 71.2
                ? "PLAYER.WASM"
                : time >= 66.95
                  ? "FIX 02"
                  : time >= 63.85
                    ? "APPLYING EDIT 02"
                    : time >= 60.8
                      ? "FAILED · PRÜFER"
                      : time >= 52.55
                        ? "FIX 01"
                        : time >= 49.4
                          ? "APPLYING EDIT 01"
                          : time >= 46.0
                            ? "FAILED · COMPILER"
                            : "PLAYER.RS";
        setText(hooks.workpieceLabel, label);
      }

      const firstCompile = smoothstep(intervalProgress(time, 42.0, 46.0));
      const secondCompile = smoothstep(intervalProgress(time, 52.55, 55.8));
      const thirdCompile = smoothstep(intervalProgress(time, 66.95, 69.7));
      const compilerRound = time < 52.55 ? 1 : time < 66.95 ? 2 : 3;
      const compilerProgress = compilerRound === 1
        ? firstCompile
        : compilerRound === 2
          ? secondCompile
          : thirdCompile;
      const compilerResult = time < 42.0
        ? "waiting"
        : time < 46.0
          ? "building"
          : time < 52.55
            ? "failed"
            : time < 55.8
              ? "building"
              : time < 66.95
                ? "passed"
                : time < 69.7
                  ? "building"
                  : "passed";
      if (hooks.compilerProgress) {
        hooks.compilerProgress.setAttribute("width", (112 * compilerProgress).toFixed(2));
        hooks.compilerProgress.dataset.filmRound = String(compilerRound);
        hooks.compilerProgress.dataset.filmResult = compilerResult;
      }
      if (hooks.compilerRound) {
        setText(
          hooks.compilerRound,
          time < 42.0
            ? "READY · ROUND 0/3"
            : time < 46.0
              ? `COMPILING · ROUND 1/3 · ${Math.round(firstCompile * 100)}%`
              : time < 52.55
                ? "FAILED · ROUND 1/3 · DIAG 01"
                : time < 55.8
                  ? `COMPILING · ROUND 2/3 · ${Math.round(secondCompile * 100)}%`
                  : time < 66.95
                    ? "PASSED · ROUND 2/3"
                    : time < 69.7
                      ? `COMPILING · ROUND 3/3 · ${Math.round(thirdCompile * 100)}%`
                      : "PASSED · ROUND 3/3",
        );
      }

      const verifierSecond = smoothstep(intervalProgress(time, 57.8, 60.8));
      const verifierThird = smoothstep(intervalProgress(time, 71.2, 75.6));
      const verifierRound = time < 66.95 ? 2 : 3;
      const verifierProgress = verifierRound === 2 ? verifierSecond : verifierThird;
      const verifierResult = time < 57.8
        ? "waiting"
        : time < 60.8
          ? "building"
          : time < 66.95
            ? "failed"
            : time < 71.2
              ? "waiting"
              : time < 75.6
                ? "building"
                : "passed";
      if (hooks.verifierProgress) {
        hooks.verifierProgress.setAttribute("width", (103 * verifierProgress).toFixed(2));
        hooks.verifierProgress.dataset.filmRound = String(verifierRound);
        hooks.verifierProgress.dataset.filmResult = verifierResult;
      }
      if (hooks.verifierRound) {
        setText(
          hooks.verifierRound,
          time < 57.8
            ? "READY · NEXT ROUND 2/3"
            : time < 60.8
              ? `CHECKING · ROUND 2/3 · ${Math.round(verifierSecond * 100)}%`
              : time < 66.95
                ? "FAILED · ROUND 2/3 · DIAG 02"
                : time < 71.2
                  ? "READY · NEXT ROUND 3/3"
                  : time < 75.6
                    ? `CHECKING · ROUND 3/3 · ${Math.round(verifierThird * 100)}%`
                    : "PASSED · ROUND 3/3",
        );
      }

      const twinAttached = time >= 74.0;
      const drillsAttached = time >= 75.6;
      const approvalReady = time >= 78.0;
      const reportAttached = time >= 84.55;
      setOpacity(hooks.sealTwin, twinAttached ? smoothstep(intervalProgress(time, 74.0, 74.5)) : 0);
      setOpacity(hooks.sealDrills, drillsAttached ? smoothstep(intervalProgress(time, 75.6, 76.1)) : 0);
      setOpacity(hooks.sealReport, approvalReady ? smoothstep(intervalProgress(time, 78.0, 78.5)) : 0);
      if (hooks.sealReport) {
        hooks.sealReport.dataset.filmPending = String(approvalReady && !reportAttached);
        hooks.sealReport.dataset.filmAttached = String(reportAttached);
      }
      setText(hooks.sealReportText, reportAttached ? "✓" : "?");

      const drillTimes = [
        [hooks.drillWall, 72.8],
        [hooks.drillImport, 74.2],
        [hooks.drillFuel, 75.6],
      ];
      drillTimes.forEach(([node, at]) => {
        if (node) node.dataset.filmPassed = String(time >= at);
      });

      const seals = Number(twinAttached) + Number(drillsAttached) + Number(reportAttached);
      const approved = time >= 84.55;
      const unlocked = time >= 84.8 && seals === 3 && approved;
      if (hooks.outGateStatus) hooks.outGateStatus.dataset.filmUnlocked = String(unlocked);
      if (hooks.outDoor) hooks.outDoor.dataset.filmUnlocked = String(unlocked);
      if (hooks.outGateCopy) {
        const gateCopy = unlocked
          ? "/out UNLOCKED · APPROVED"
          : approvalReady
            ? "/out LOCKED · OWNER APPROVAL"
            : `/out LOCKED · ${seals}/3 SEALS`;
        setText(hooks.outGateCopy, gateCopy);
      }
    }

    function renderFeedback(time) {
      const cards = selectAll("[data-film-feedback-card]");
      cards.forEach((card, index) => {
        const start = 55.0 + index * 0.42;
        const alpha = smoothstep(intervalProgress(time, start, start + 0.32));
        setOpacity(card, alpha);
        card.style.setProperty("--film-card-progress", alpha.toFixed(4));
      });
    }

    function renderTwins(time) {
      const failedBuildProgress = smoothstep(intervalProgress(time, 57.8, 60.8));
      const passedBuildProgress = smoothstep(intervalProgress(time, 71.2, 74.0));
      const buildProgress = time < 66.95 ? failedBuildProgress : passedBuildProgress;
      setProgress(hooks.buildA, buildProgress);
      setProgress(hooks.buildB, buildProgress);
      const hashesVisible = time >= 60.3;
      setOpacity(hooks.hashA, hashesVisible ? 1 : 0);
      setOpacity(hooks.hashB, hashesVisible ? 1 : 0);
      const verifierFailure = time >= 60.8 && time < 66.95;
      if (hooks.hashB) hooks.hashB.dataset.filmMismatch = String(verifierFailure);
      if (hooks.equalGroup) {
        const passed = time >= 74.0;
        setText(hooks.equalText, verifierFailure ? "RED · BYTE DRIFT" : passed ? "EQUAL" : "BUILDING A/B");
        const resultAlpha = verifierFailure
          ? smoothstep(intervalProgress(time, 60.8, 61.25))
          : smoothstep(intervalProgress(time, 71.2, 71.8));
        setOpacity(hooks.equalGroup, resultAlpha);
        hooks.equalGroup.dataset.filmResult = verifierFailure ? "denied" : passed ? "equal" : "building";
      }
      [hooks.buildA, hooks.buildB].forEach((node) => {
        if (node) node.dataset.filmResult = time >= 74.0 ? "complete" : verifierFailure ? "failed" : "building";
      });
    }

    function renderPersistentVeins(time) {
      const pulseOffset = -time * 22;
      const pulseOpacity = 0.58 + Math.sin(time * Math.PI * 1.7) * 0.14;
      pulseNodes.forEach((node) => {
        const speed = Number.parseFloat(node.dataset.filmSpeed || "22") || 22;
        const direction = node.dataset.filmDirection === "reverse" ? -1 : 1;
        const offset = -time * speed * direction;
        node.style.strokeDashoffset = offset.toFixed(2);
        node.style.setProperty("--film-dash-offset", offset.toFixed(2));
      });
      if (hooks.netVein) {
        hooks.netVein.style.strokeDashoffset = pulseOffset.toFixed(2);
        hooks.netVein.style.setProperty("--film-dash-offset", pulseOffset.toFixed(2));
        setOpacity(hooks.netVein, time >= 18.2 ? pulseOpacity : 0);
        hooks.netVein.dataset.filmConnected = String(time >= 18.2);
      }
      if (hooks.reclog) {
        const logProgress = smoothstep(intervalProgress(time, 74.0, 79.2));
        hooks.reclog.style.strokeDasharray = "100";
        hooks.reclog.style.strokeDashoffset = (100 * (1 - logProgress)).toFixed(2);
        hooks.reclog.style.setProperty("--film-log-progress", logProgress.toFixed(4));
        setOpacity(hooks.reclog, time >= 73.6 ? 1 : 0);
      }
    }

    function renderBuilderShutdown(time) {
      const orderStart = 105.6;
      const orderEnd = 107.2;
      if (hooks.shutdownOrder && hooks.shutdownPath && typeof hooks.shutdownPath.getTotalLength === "function") {
        const progress = smoothstep(intervalProgress(time, orderStart, orderEnd));
        const length = hooks.shutdownPath.getTotalLength();
        const point = hooks.shutdownPath.getPointAtLength(length * progress);
        const enter = smoothstep(intervalProgress(time, orderStart - 0.2, orderStart + 0.18));
        const exit = 1 - smoothstep(intervalProgress(time, orderEnd - 0.22, orderEnd + 0.18));
        setComposedTransform(hooks.shutdownOrder, `translate(${point.x.toFixed(2)} ${point.y.toFixed(2)})`);
        setOpacity(hooks.shutdownOrder, enter * exit);
        hooks.shutdownOrder.dataset.filmState = time < orderStart
          ? "waiting"
          : time < orderEnd
            ? "travelling"
            : "delivered";
      }

      teardownNodes.forEach((entry) => {
        const { node, start, end, dx, dy, managedBefore } = entry;
        if (time < start) {
          setComposedTransform(node, "");
          if (managedBefore) setOpacity(node, 1);
          node.dataset.filmTeardownState = "standing";
          return;
        }
        const progress = smoothstep(intervalProgress(time, start, end));
        setComposedTransform(node, `translate(${(dx * progress).toFixed(2)} ${(dy * progress).toFixed(2)})`);
        setOpacity(node, 1 - progress);
        node.dataset.filmTeardownState = progress >= 1 ? "released" : "releasing";
      });

      if (hooks.builderDeck) hooks.builderDeck.dataset.filmReleased = String(time >= 113.4);
      if (hooks.playerDomain) hooks.playerDomain.dataset.filmAttached = String(time >= 92.0);
    }

    function renderRingAndApproval(time) {
      const ringStates = [
        [hooks.ringManifest, 81.2],
        [hooks.ringHash, 82.2],
        [hooks.ringReport, 83.2],
        [hooks.ringApproval, 84.55],
      ];
      ringStates.forEach(([node, at], index) => {
        if (!node) return;
        const attached = time >= at;
        const alpha = index < 3
          ? smoothstep(intervalProgress(time, at, at + 0.45))
          : attached
            ? smoothstep(intervalProgress(time, at, at + 0.25))
            : 0.25 + 0.18 * (0.5 + 0.5 * Math.sin(time * 5.2));
        setOpacity(node, alpha);
        node.dataset.filmAttached = String(attached);
      });

      const pointerProgress = smoothstep(intervalProgress(time, 81.4, 84.25));
      if (hooks.approvalPointer) {
        hooks.approvalPointer.style.setProperty("--film-pointer-progress", pointerProgress.toFixed(4));
        hooks.approvalPointer.dataset.filmClicked = String(time >= 84.25);
        setComposedTransform(
          hooks.approvalPointer,
          `translate(${(58 * (1 - pointerProgress)).toFixed(2)} ${(-54 * (1 - pointerProgress)).toFixed(2)})`,
        );
        setOpacity(hooks.approvalPointer, windowState(time, { start: 80.9, end: 85.4, fade: 0.3 }).alpha);
      }
      if (hooks.approvalButton) {
        const pressed = time >= 84.15 && time < 84.62;
        hooks.approvalButton.dataset.filmPressed = String(pressed);
        hooks.approvalButton.dataset.filmApproved = String(time >= 84.55);
      }
      if (hooks.remoteDenied) {
        setOpacity(hooks.remoteDenied, windowState(time, { start: 85.0, end: 87.2, fade: 0.24 }).alpha);
      }
    }

    function renderFinale(time) {
      if (hooks.titleCard) {
        setOpacity(hooks.titleCard, windowState(time, { start: 115.0, end: 119.45, fade: 0.65 }).alpha);
      }
      if (hooks.titleSubtitle) {
        setOpacity(hooks.titleSubtitle, windowState(time, { start: 115.35, end: 119.45, fade: 0.65 }).alpha);
      }
      cycleNodes.forEach((node, index) => {
        const cycleStart = 114.3 + index * 0.48;
        const cycleProgress = smoothstep(intervalProgress(time, cycleStart, cycleStart + 0.35));
        node.style.setProperty("--film-cycle-progress", cycleProgress.toFixed(4));
        node.dataset.filmLit = String(time >= cycleStart);
      });
    }

    function renderSceneDetails(time) {
      if (hooks.fuelCount) {
        const fuel = Math.round(100 * (1 - smoothstep(intervalProgress(time, 73.2, 78.0))));
        setText(hooks.fuelCount, String(fuel));
        if (hooks.fuelRing) {
          hooks.fuelRing.style.setProperty("--film-fuel", (fuel / 100).toFixed(4));
          hooks.fuelRing.dataset.filmEmpty = String(fuel === 0);
        }
      }
      if (hooks.trackProgress) {
        const track = smoothstep(intervalProgress(time, 97.0, 105.4));
        hooks.trackProgress.setAttribute("width", (30 + 270 * track).toFixed(2));
        hooks.trackProgress.style.setProperty("--film-track-progress", track.toFixed(4));
      }
      if (hooks.crashNeighbor) {
        const crash = smoothstep(intervalProgress(time, 101.4, 104.3));
        setOpacity(hooks.crashNeighbor, time < 103.1 ? 1 : 0);
        setComposedTransform(hooks.crashNeighbor, `translate(0 ${(36 * crash).toFixed(2)})`);
        hooks.crashNeighbor.dataset.filmTrapped = String(time >= 101.4);
      }
      if (hooks.trapFlash) {
        setOpacity(hooks.trapFlash, time >= 101.4 && time < 102.25 ? 1 : 0);
      }
      if (hooks.cutCable) {
        const cut = time >= 102.25;
        hooks.cutCable.style.setProperty("--film-cut-progress", cut ? "1" : "0");
        hooks.cutCable.dataset.filmCut = String(cut);
        setOpacity(hooks.cutCable, cut ? 0 : 1);
      }
      if (hooks.badVersion) {
        const rollback = smoothstep(intervalProgress(time, 109.3, 111.2));
        hooks.badVersion.style.setProperty("--film-rollback-progress", rollback.toFixed(4));
        hooks.badVersion.dataset.filmRolledBack = String(rollback >= 1);
        setOpacity(hooks.badVersion, 1 - rollback);
      }
    }

    function formatTimecode(time) {
      const seconds = Math.floor(clamp(time, 0, DURATION));
      const minutes = Math.floor(seconds / 60);
      const remainder = seconds % 60;
      return `${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")} / 02:00`;
    }

    function renderHud(time, activeScene) {
      setText(hooks.sceneNumber, `${String(activeScene.number).padStart(2, "0")} / 14`);
      setText(hooks.sceneName, activeScene.title.toUpperCase());
      setText(hooks.timecode, formatTimecode(time));
      if (hooks.timelineProgress) {
        const length = 1132 * clamp(time / DURATION, 0, 1);
        hooks.timelineProgress.setAttribute("d", `M34 871h${length.toFixed(2)}`);
        hooks.timelineProgress.style.setProperty("--film-total-progress", (time / DURATION).toFixed(6));
      }
    }

    function updateScrubber(time, activeScene) {
      if (scrubber) scrubber.classList.toggle("is-introduced", time >= TIMELINE_REVEAL_TIME);
      if (scrubRange && document.activeElement !== scrubRange) scrubRange.value = time.toFixed(2);
      if (scrubOutput) scrubOutput.textContent = `${formatTimecode(time).slice(0, 5)} · ${activeScene.title}`;
      if (scrubPlay) {
        scrubPlay.textContent = playing ? "Pause" : "Play";
        scrubPlay.setAttribute("aria-pressed", String(playing));
      }
      scrubMarks.forEach((button, index) => {
        const scene = SCENES[index];
        const active = scene.number === activeScene.number;
        button.classList.toggle("is-active", active);
        button.classList.toggle("is-past", time >= scene.start);
        if (active) button.setAttribute("aria-current", "step");
        else button.removeAttribute("aria-current");
      });
    }

    function render(time = currentTime) {
      currentTime = parseTime(time, currentTime);
      const activeScene = sceneAt(currentTime);
      const sceneProgress = intervalProgress(currentTime, activeScene.start, activeScene.end);
      root.dataset.filmScene = String(activeScene.number);
      root.dataset.filmSceneId = activeScene.id;
      root.dataset.filmPlaying = String(playing);
      root.style.setProperty("--film-time", currentTime.toFixed(4));
      root.style.setProperty("--film-total-progress", (currentTime / DURATION).toFixed(6));
      root.style.setProperty("--film-scene-progress", sceneProgress.toFixed(6));
      root.style.setProperty("--film-breathe", (0.5 + 0.5 * Math.sin(currentTime * Math.PI)).toFixed(4));
      renderScenes(currentTime, activeScene);
      renderCamera(currentTime);
      renderWindows(currentTime);
      renderDeclarativeActions(currentTime);
      renderDoorCeremonies(currentTime);
      renderBuilderArchitecture(currentTime);
      renderPrompt(currentTime);
      renderLayerCallouts(currentTime);
      renderCompiler(currentTime);
      renderFeedback(currentTime);
      renderTwins(currentTime);
      renderWorkshop(currentTime);
      renderBuilderShutdown(currentTime);
      renderPersistentVeins(currentTime);
      renderRingAndApproval(currentTime);
      renderFinale(currentTime);
      renderSceneDetails(currentTime);
      renderHud(currentTime, activeScene);
      updateScrubber(currentTime, activeScene);
      return currentTime;
    }

    function frame(now) {
      if (!playing) return;
      const elapsed = (now - anchorNow) / 1000;
      currentTime = (anchorTime + elapsed) % DURATION;
      render(currentTime);
      animationFrame = requestAnimationFrame(frame);
    }

    function setTime(nextTime) {
      currentTime = parseTime(nextTime, currentTime);
      anchorTime = currentTime >= DURATION && playing ? 0 : currentTime;
      anchorNow = performance.now();
      render(currentTime);
      return currentTime;
    }

    function stopClock(preserveIntent) {
      if (animationFrame) cancelAnimationFrame(animationFrame);
      animationFrame = 0;
      playing = false;
      if (!preserveIntent) wantsPlayback = false;
      root.dataset.filmPlaying = "false";
      updateScrubber(currentTime, sceneAt(currentTime));
      return currentTime;
    }

    function pause() {
      return stopClock(false);
    }

    function play() {
      if (playing) return true;
      wantsPlayback = true;
      suspendedByMode = false;
      if (currentTime >= DURATION) currentTime = 0;
      anchorTime = currentTime;
      anchorNow = performance.now();
      playing = true;
      root.dataset.filmPlaying = "true";
      animationFrame = requestAnimationFrame(frame);
      updateScrubber(currentTime, sceneAt(currentTime));
      return true;
    }

    function createScrubber() {
      const controls = document.createElement("div");
      controls.className = "film-scrubber";
      controls.classList.toggle("is-introduced", currentTime >= TIMELINE_REVEAL_TIME);
      const timelineRequested = params.get("timeline") === "1" || params.get("film") === "scrub";
      controls.hidden = deterministic && !timelineRequested;
      root.dataset.filmTimeline = String(!controls.hidden);
      controls.setAttribute("role", "group");
      controls.setAttribute("aria-label", "Film timeline and scene states; use the mouse wheel over the film to scrub");

      const playButton = document.createElement("button");
      playButton.className = "film-scrubber__play";
      playButton.type = "button";
      playButton.textContent = "Play";
      controls.appendChild(playButton);

      const track = document.createElement("div");
      track.className = "film-scrubber__track";

      const range = document.createElement("input");
      range.className = "film-scrubber__range";
      range.type = "range";
      range.min = "0";
      range.max = String(DURATION);
      range.step = "0.01";
      range.value = currentTime.toFixed(2);
      range.setAttribute("aria-label", "Scrub film time; release to continue playback");
      track.appendChild(range);

      const marks = document.createElement("div");
      marks.className = "film-scrubber__marks";
      let wheelTargetTime = currentTime;
      let wheelScrubFrame = 0;
      let wheelLastFrame = 0;

      function cancelWheelScrub() {
        if (wheelScrubFrame) cancelAnimationFrame(wheelScrubFrame);
        wheelScrubFrame = 0;
        wheelLastFrame = 0;
        wheelTargetTime = currentTime;
        controls.classList.remove("is-wheel-scrubbing");
      }

      function animateWheelScrub(now) {
        const elapsed = wheelLastFrame
          ? Math.min(48, Math.max(1, now - wheelLastFrame))
          : 16.7;
        wheelLastFrame = now;
        const distance = wheelTargetTime - currentTime;
        if (Math.abs(distance) <= 0.004) {
          setTime(wheelTargetTime);
          syncPinnedScrollToFilm();
          wheelScrubFrame = 0;
          wheelLastFrame = 0;
          controls.classList.remove("is-wheel-scrubbing");
          return;
        }
        const easing = 1 - Math.exp(-elapsed / WHEEL_SCRUB_EASING_MS);
        setTime(currentTime + distance * easing);
        syncPinnedScrollToFilm();
        wheelScrubFrame = requestAnimationFrame(animateWheelScrub);
      }

      SCENES.forEach((scene) => {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = String(scene.number).padStart(2, "0");
        button.title = `${scene.start}s · ${scene.title}`;
        button.setAttribute("aria-label", `State ${scene.number}: ${scene.title}, ${scene.start} seconds`);
        button.dataset.filmJump = String(scene.start);
        button.style.setProperty("--film-mark-position", `${(scene.start / DURATION) * 100}%`);
        button.addEventListener("click", () => {
          cancelWheelScrub();
          stopClock(false);
          setTime(scene.start);
        });
        marks.appendChild(button);
      });
      track.appendChild(marks);
      controls.appendChild(track);

      const output = document.createElement("output");
      output.className = "film-scrubber__time";
      controls.appendChild(output);

      playButton.addEventListener("click", () => {
        cancelWheelScrub();
        if (playing) pause();
        else play();
      });
      const snapTime = (value, threshold = 1.8) => {
        const nearest = SCENES.reduce((best, scene) => (
          Math.abs(scene.start - value) < Math.abs(best - value) ? scene.start : best
        ), SCENES[0].start);
        return Math.abs(nearest - value) <= threshold ? nearest : value;
      };
      let scrubbing = false;
      const beginScrub = () => {
        cancelWheelScrub();
        scrubbing = true;
        stopClock(false);
        controls.classList.add("is-scrubbing");
      };
      const updateFromRange = () => {
        if (!scrubbing) beginScrub();
        const snapped = snapTime(Number.parseFloat(range.value));
        range.value = snapped.toFixed(2);
        setTime(snapped);
      };
      const finishScrub = () => {
        if (!scrubbing) return;
        scrubbing = false;
        controls.classList.remove("is-scrubbing");
        const snapped = snapTime(Number.parseFloat(range.value), 2.8);
        range.value = snapped.toFixed(2);
        setTime(snapped);
      };
      range.addEventListener("pointerdown", beginScrub);
      range.addEventListener("input", updateFromRange);
      range.addEventListener("pointerup", finishScrub);
      range.addEventListener("pointercancel", finishScrub);
      range.addEventListener("change", finishScrub);
      range.addEventListener("keydown", (event) => {
        if (event.key === "Home" || event.key === "End" || event.key.startsWith("Arrow")) beginScrub();
      });
      range.addEventListener("keyup", (event) => {
        if (event.key === "Home" || event.key === "End" || event.key.startsWith("Arrow")) finishScrub();
      });

      const wheelSurface = root.closest(".film-frame") || root;
      let scrollPinned = false;
      let pinnedScrollY = 0;
      let pinnedScrollStartY = 0;
      let pinnedScrollEndY = 0;
      let pinCorrectionFrame = 0;
      const scrollTriggerGeometry = () => {
        const bounds = wheelSurface.getBoundingClientRect();
        const frameAnchor = bounds.top + bounds.height * SCROLL_TRIGGER_FRAME_Y;
        const viewportAnchor = window.innerHeight * SCROLL_TRIGGER_VIEWPORT_Y;
        return {
          distance: frameAnchor - viewportAnchor,
          tolerance: window.innerHeight * SCROLL_TRIGGER_TOLERANCE,
        };
      };
      const scrollTriggerIsActive = () => {
        if (controls.hidden) return false;
        const geometry = scrollTriggerGeometry();
        const active = scrollPinned || Math.abs(geometry.distance) <= geometry.tolerance;
        controls.classList.toggle("is-scroll-armed", active);
        root.dataset.filmScrollArmed = String(active);
        root.dataset.filmScrollPinned = String(scrollPinned);
        return active;
      };
      const lockScrollPosition = () => {
        const maximumScrollY = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
        pinnedScrollStartY = window.scrollY;
        pinnedScrollEndY = Math.min(
          maximumScrollY,
          pinnedScrollStartY + window.innerHeight * SCROLL_PAGE_TRAVEL_VH,
        );
        pinnedScrollY = pinnedScrollStartY;
        scrollPinned = true;
        scrollTriggerIsActive();
      };
      const syncPinnedScrollToFilm = () => {
        if (!scrollPinned) return;
        const travelProgress = reducedMotion
          ? intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME)
          : smootherstep(intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME));
        pinnedScrollY = lerp(pinnedScrollStartY, pinnedScrollEndY, travelProgress);
        if (Math.abs(window.scrollY - pinnedScrollY) > 0.35) {
          window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
        }
      };
      const releaseScrollPin = () => {
        scrollPinned = false;
        if (pinCorrectionFrame) cancelAnimationFrame(pinCorrectionFrame);
        pinCorrectionFrame = 0;
        scrollTriggerIsActive();
      };
      let previousTriggerDistance = scrollTriggerGeometry().distance;
      const updateScrollTrigger = () => {
        const geometry = scrollTriggerGeometry();
        const crossedEntry = previousTriggerDistance !== 0
          && Math.sign(previousTriggerDistance) !== Math.sign(geometry.distance);
        previousTriggerDistance = geometry.distance;
        if (!scrollPinned && !controls.hidden
          && (crossedEntry || Math.abs(geometry.distance) <= geometry.tolerance)) {
          lockScrollPosition();
          return;
        }
        if (scrollPinned && Math.abs(window.scrollY - pinnedScrollY) > 1) {
          if (pinCorrectionFrame) cancelAnimationFrame(pinCorrectionFrame);
          pinCorrectionFrame = requestAnimationFrame(() => {
            pinCorrectionFrame = 0;
            if (scrollPinned) {
              window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
            }
          });
        }
        scrollTriggerIsActive();
      };
      window.addEventListener("scroll", updateScrollTrigger, { passive: true });
      window.addEventListener("resize", updateScrollTrigger, { passive: true });
      scrollTriggerIsActive();

      window.addEventListener("wheel", (event) => {
        if (controls.hidden || event.ctrlKey) return;
        if (!scrollTriggerIsActive()) return;
        const primaryDelta = Math.abs(event.deltaY) >= Math.abs(event.deltaX)
          ? event.deltaY
          : event.deltaX;
        if (!primaryDelta) return;
        const deltaPixels = primaryDelta * (event.deltaMode === 1
          ? 16
          : event.deltaMode === 2
            ? window.innerHeight
            : 1);
        const deltaSeconds = clamp(deltaPixels * 0.012, -2.4, 2.4);
        if (!wheelScrubFrame) wheelTargetTime = currentTime;
        const atStart = wheelTargetTime <= 0.001 && deltaSeconds < 0;
        const atEnd = wheelTargetTime >= DURATION - 0.001 && deltaSeconds > 0;
        if (atStart || atEnd) {
          releaseScrollPin();
          return;
        }

        event.preventDefault();
        if (!scrollPinned) lockScrollPosition();
        else if (Math.abs(window.scrollY - pinnedScrollY) > 1) {
          window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
        }
        stopClock(false);
        wheelTargetTime = clamp(
          wheelTargetTime + deltaSeconds,
          Math.max(0, currentTime - WHEEL_MAX_LEAD_SECONDS),
          Math.min(DURATION, currentTime + WHEEL_MAX_LEAD_SECONDS),
        );
        controls.classList.add("is-wheel-scrubbing");
        if (reducedMotion) {
          setTime(wheelTargetTime);
          syncPinnedScrollToFilm();
          controls.classList.remove("is-wheel-scrubbing");
        } else if (!wheelScrubFrame) {
          wheelScrubFrame = requestAnimationFrame(animateWheelScrub);
        }
      }, { passive: false });

      root.insertAdjacentElement("afterend", controls);
      scrubber = controls;
      scrubRange = range;
      scrubOutput = output;
      scrubPlay = playButton;
      scrubMarks = Array.from(marks.querySelectorAll("button"));
    }

    refresh();
    createScrubber();

    const api = {
      ready: true,
      duration: DURATION,
      posterTime: POSTER_TIME,
      scenes: SCENES,
      cameraKeyframes: CAMERA_KEYFRAMES,
      getTime: () => currentTime,
      setTime,
      seek: setTime,
      renderAt: setTime,
      render: (time) => render(typeof time === "undefined" ? currentTime : time),
      pause,
      play,
      refresh: () => { refresh(); render(currentTime); return true; },
    };
    Object.defineProperties(api, {
      time: { enumerable: true, get: () => currentTime },
      scene: { enumerable: true, get: () => sceneAt(currentTime) },
      playing: { enumerable: true, get: () => playing },
      scrubber: { enumerable: true, get: () => scrubber },
    });
    window.__RAIOS_FILM__ = api;

    document.addEventListener("raios:website-mode", (event) => {
      const enabled = Boolean(event.detail && event.detail.enabled);
      if (!enabled && playing) {
        suspendedByMode = true;
        stopClock(true);
      } else if (enabled && suspendedByMode && wantsPlayback) {
        play();
      }
    });
    document.addEventListener("visibilitychange", () => {
      if (document.hidden && playing) {
        suspendedByMode = true;
        stopClock(true);
      } else if (!document.hidden && suspendedByMode && wantsPlayback
          && document.body.dataset.shellMode === "website") {
        play();
      }
    });

    if (reduceMotionQuery && typeof reduceMotionQuery.addEventListener === "function") {
      reduceMotionQuery.addEventListener("change", (event) => {
        if (deterministic) return;
        if (event.matches) {
          stopClock(false);
          setTime(POSTER_TIME);
        }
      });
    }

    render(currentTime);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialiseFilm, { once: true });
  } else {
    initialiseFilm();
  }
})();
