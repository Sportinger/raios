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

  const sceneDefinitions = [
    [1, "sentence", "A sentence", 0, 6, 1, 1.15, 0.50, 0.49],
    [2, "world", "The world under the glass", 6, 14, 1, 0.88, 0.50, 0.51],
    [3, "net", "The agent needs the net", 14, 25, 1, 1.02, 0.53, 0.48],
    [4, "build-site", "Genesis stamps a build site", 25, 33, 2, 1.02, 0.52, 0.51],
    [5, "inert-source", "The answer becomes inert source", 33, 41, 2, 1.13, 0.47, 0.43],
    [6, "compiler", "The compiler moves in", 41, 52, 2, 1.35, 0.50, 0.49],
    [7, "feedback", "The loop that fixes itself", 52, 62, 2, 1.12, 0.48, 0.48],
    [8, "twins", "Build twice, believe once", 62, 72, 2, 1.08, 0.53, 0.48],
    [9, "proof-cellar", "The proof cellar", 72, 80, 2, 1.15, 0.52, 0.62],
    [10, "ring", "The ring closes", 80, 88, 3, 1.22, 0.58, 0.47],
    [11, "approval", "The physical click", 88, 96, 3, 1.28, 0.50, 0.49],
    [12, "running", "Running in its cage", 96, 106, 3, 1.38, 0.57, 0.49],
    [13, "factory", "The factory stays", 106, 114, 3, 0.92, 0.51, 0.50],
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
    { at: 13.4, scale: 0.88, focusX: 0.50, focusY: 0.51 },
    { at: 25, scale: 1.02, focusX: 0.53, focusY: 0.48 },
    { at: 41, scale: 1.08, focusX: 0.54, focusY: 0.47 },
    { at: 45.8, scale: 1.35, focusX: 0.50, focusY: 0.49 },
    { at: 50.6, scale: 1.35, focusX: 0.50, focusY: 0.49 },
    { at: 52, scale: 1.12, focusX: 0.48, focusY: 0.48 },
    { at: 72, scale: 1.08, focusX: 0.53, focusY: 0.48 },
    { at: 80, scale: 1.15, focusX: 0.52, focusY: 0.62 },
    { at: 88, scale: 1.22, focusX: 0.58, focusY: 0.47 },
    { at: 90.2, scale: 1.28, focusX: 0.50, focusY: 0.49 },
    { at: 94.7, scale: 1.28, focusX: 0.50, focusY: 0.49 },
    { at: 96, scale: 1.10, focusX: 0.57, focusY: 0.46 },
    { at: 99.2, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 104.6, scale: 1.38, focusX: 0.57, focusY: 0.49 },
    { at: 106, scale: 0.92, focusX: 0.51, focusY: 0.50 },
    { at: 114, scale: 0.84, focusX: 0.50, focusY: 0.50 },
    { at: 118.3, scale: 0.84, focusX: 0.50, focusY: 0.50 },
    { at: 120, scale: 1.15, focusX: 0.50, focusY: 0.49 },
  ].map(Object.freeze));

  const clamp = (value, min, max) => Math.min(max, Math.max(min, value));
  const lerp = (a, b, amount) => a + (b - a) * amount;
  const smoothstep = (amount) => {
    const value = clamp(amount, 0, 1);
    return value * value * (3 - 2 * value);
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
    let wantsPlayback = !deterministic && !reducedMotion;
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
      const twinCells = selectAll(".film-twin-cell");
      const twinHashes = selectAll(".film-twin-cell .film-hash");
      const ringSegments = selectAll("#film-ring .film-ring-segment");
      hooks = {
        camera: selectOne("#film-cam", "#cam"),
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
      };
      actionNodes.concat([hooks.approvalPointer, hooks.crashNeighbor].filter(Boolean)).forEach((node) => {
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
      const viewBox = svg.viewBox && svg.viewBox.baseVal;
      const boxX = viewBox && viewBox.width ? viewBox.x : 0;
      const boxY = viewBox && viewBox.height ? viewBox.y : 0;
      const boxWidth = viewBox && viewBox.width ? viewBox.width : 1000;
      const boxHeight = viewBox && viewBox.height ? viewBox.height : 640;
      const centerX = boxX + boxWidth / 2;
      const centerY = boxY + boxHeight / 2;
      const focusX = boxX + boxWidth * camera.focusX;
      const focusY = boxY + boxHeight * camera.focusY;
      const translateX = centerX - focusX * camera.scale;
      const translateY = centerY - focusY * camera.scale;
      hooks.camera.setAttribute(
        "transform",
        `translate(${translateX.toFixed(3)} ${translateY.toFixed(3)}) scale(${camera.scale.toFixed(5)})`,
      );
      hooks.camera.style.setProperty("--film-camera-scale", camera.scale.toFixed(5));
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
              const hiddenTop = (1 - progress) * 100;
              node.style.clipPath = `inset(${hiddenTop.toFixed(3)}% 0 0 0)`;
              node.style.setProperty("--film-iso-reveal", progress.toFixed(4));
              offsetY = 0;
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
          setOpacity(node, started ? (isometricGrow ? 1 : Math.min(1, progress * 2.5)) : 0);
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
      if (hooks.prompt) {
        const progress = intervalProgress(time, 2.0, 5.05);
        const count = Math.floor(PROMPT.length * progress + 0.0001);
        setText(hooks.prompt, PROMPT.slice(0, count));
        hooks.prompt.dataset.filmSubmitted = String(time >= 5.18);
      }
      if (hooks.promptCursor) {
        const blink = time >= 1.8 && time < 5.25 && Math.floor(time * 2.4) % 2 === 0;
        setOpacity(hooks.promptCursor, blink ? 1 : 0);
        const promptX = hooks.prompt ? Number.parseFloat(hooks.prompt.getAttribute("x") || "270") : 270;
        const promptWidth = hooks.prompt && typeof hooks.prompt.getComputedTextLength === "function"
          ? hooks.prompt.getComputedTextLength()
          : 0;
        hooks.promptCursor.setAttribute("x", (promptX + promptWidth + 7).toFixed(2));
      }
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
      const buildProgress = smoothstep(intervalProgress(time, 63.0, 68.3));
      setProgress(hooks.buildA, buildProgress);
      setProgress(hooks.buildB, buildProgress);
      const mismatch = time >= 70.55 && time < 71.45;
      const hashesVisible = time >= 67.7;
      setOpacity(hooks.hashA, hashesVisible ? 1 : 0);
      setOpacity(hooks.hashB, hashesVisible ? 1 : 0);
      if (hooks.hashB) hooks.hashB.dataset.filmMismatch = String(mismatch);
      if (hooks.equalGroup) {
        setText(hooks.equalText, mismatch ? "SEALED · BYTE DIFFERS" : "EQUAL");
        setOpacity(hooks.equalGroup, smoothstep(intervalProgress(time, 68.5, 69.1)));
        hooks.equalGroup.dataset.filmResult = mismatch ? "denied" : "equal";
      }
      [hooks.buildA, hooks.buildB].forEach((node) => {
        if (node) node.dataset.filmResult = mismatch ? "sealed" : buildProgress >= 1 ? "complete" : "building";
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

    function renderRingAndApproval(time) {
      const ringStates = [
        [hooks.ringManifest, 81.2],
        [hooks.ringHash, 82.2],
        [hooks.ringReport, 83.2],
        [hooks.ringApproval, 92.55],
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

      const pointerProgress = smoothstep(intervalProgress(time, 89.4, 92.25));
      if (hooks.approvalPointer) {
        hooks.approvalPointer.style.setProperty("--film-pointer-progress", pointerProgress.toFixed(4));
        hooks.approvalPointer.dataset.filmClicked = String(time >= 92.25);
        setComposedTransform(
          hooks.approvalPointer,
          `translate(${(58 * (1 - pointerProgress)).toFixed(2)} ${(-54 * (1 - pointerProgress)).toFixed(2)})`,
        );
        setOpacity(hooks.approvalPointer, windowState(time, { start: 89.0, end: 93.4, fade: 0.3 }).alpha);
      }
      if (hooks.approvalButton) {
        const pressed = time >= 92.15 && time < 92.62;
        hooks.approvalButton.dataset.filmPressed = String(pressed);
        hooks.approvalButton.dataset.filmApproved = String(time >= 92.55);
      }
      if (hooks.remoteDenied) {
        setOpacity(hooks.remoteDenied, windowState(time, { start: 93.0, end: 95.1, fade: 0.24 }).alpha);
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
        hooks.timelineProgress.setAttribute("d", `M34 651h${length.toFixed(2)}`);
        hooks.timelineProgress.style.setProperty("--film-total-progress", (time / DURATION).toFixed(6));
      }
    }

    function updateScrubber(time, activeScene) {
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
      renderPrompt(currentTime);
      renderCompiler(currentTime);
      renderFeedback(currentTime);
      renderTwins(currentTime);
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
      const timelineRequested = params.get("timeline") === "1" || params.get("film") === "scrub";
      controls.hidden = deterministic && !timelineRequested;
      root.dataset.filmTimeline = String(!controls.hidden);
      controls.setAttribute("role", "group");
      controls.setAttribute("aria-label", "Film timeline and scene states");

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
      SCENES.forEach((scene) => {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = String(scene.number).padStart(2, "0");
        button.title = `${scene.start}s · ${scene.title}`;
        button.setAttribute("aria-label", `State ${scene.number}: ${scene.title}, ${scene.start} seconds`);
        button.dataset.filmJump = String(scene.start);
        button.style.setProperty("--film-mark-position", `${(scene.start / DURATION) * 100}%`);
        button.addEventListener("click", () => {
          stopClock(true);
          setTime(scene.start);
          if (!deterministic && !reducedMotion) play();
        });
        marks.appendChild(button);
      });
      track.appendChild(marks);
      controls.appendChild(track);

      const output = document.createElement("output");
      output.className = "film-scrubber__time";
      controls.appendChild(output);

      playButton.addEventListener("click", () => {
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
        scrubbing = true;
        stopClock(true);
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
        if (!deterministic && !reducedMotion) play();
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
    if (wantsPlayback && document.body.dataset.shellMode === "website") play();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialiseFilm, { once: true });
  } else {
    initialiseFilm();
  }
})();
