// Website-only film clock for "The Factory Moves In".
//
// The SVG is deliberately declarative: this module owns time and state, while
// the document owns every shape. All hooks except the film root are optional so
// that the animatic and the finished artwork can share the same clock.

(function () {
  "use strict";

  // ================= SECTION: Timeline model & constants =================
  const DURATION = 120;
  const POSTER_TIME = 118;
  const PROMPT = "> build me a music player";
  const PROMPT_TYPE_START = 1.15;
  const PROMPT_TYPE_END = 3.72;
  // ================= SECTION: Audio cues & captions =================
  const PROMPT_AUDIO_URL = "ui-lab/assets/audio/build-me-a-music-player.mp3";
  const PROMPT_AUDIO_REVERSED_URL = "ui-lab/assets/audio/build-me-a-music-player-reversed.mp3";
  const PROMPT_AUDIO_START = 1.15;
  const PROMPT_AUDIO_DURATION = 2.742813;
  const PROMPT_AUDIO_END = PROMPT_AUDIO_START + PROMPT_AUDIO_DURATION;
  const LAYER_VOICE_REVEAL_DELAY = 0.55;
  const KERNEL_AUDIO_URL = "ui-lab/assets/audio/kernel-layer.mp3";
  const KERNEL_AUDIO_REVERSED_URL = "ui-lab/assets/audio/kernel-layer-reversed.mp3";
  const KERNEL_AUDIO_DURATION = 3.84;
  const KERNEL_AUDIO_START = PROMPT_AUDIO_END + LAYER_VOICE_REVEAL_DELAY;
  const KERNEL_AUDIO_END = KERNEL_AUDIO_START + KERNEL_AUDIO_DURATION;
  const GENESIS_AUDIO_URL = "ui-lab/assets/audio/genesis-layer.mp3";
  const GENESIS_AUDIO_REVERSED_URL = "ui-lab/assets/audio/genesis-layer-reversed.mp3";
  const GENESIS_AUDIO_DURATION = 3.761625;
  const GENESIS_AUDIO_START = KERNEL_AUDIO_END + LAYER_VOICE_REVEAL_DELAY;
  const GENESIS_AUDIO_END = GENESIS_AUDIO_START + GENESIS_AUDIO_DURATION;
  const AGENT_AUDIO_URL = "ui-lab/assets/audio/agent-internet-key.mp3";
  const AGENT_AUDIO_REVERSED_URL = "ui-lab/assets/audio/agent-internet-key-reversed.mp3";
  const AGENT_AUDIO_DURATION = 12.120813;
  const AGENT_AUDIO_START = GENESIS_AUDIO_END;
  const AGENT_AUDIO_END = AGENT_AUDIO_START + AGENT_AUDIO_DURATION;
  const BUILDER_AUDIO_URL = "ui-lab/assets/audio/builder-playground.mp3";
  const BUILDER_AUDIO_REVERSED_URL = "ui-lab/assets/audio/builder-playground-reversed.mp3";
  const BUILDER_AUDIO_DURATION = 13.87102;
  const BUILDER_AUDIO_START = AGENT_AUDIO_END + 0.2;
  const BUILDER_AUDIO_END = BUILDER_AUDIO_START + BUILDER_AUDIO_DURATION;
  const COMPILER_FAIL_AUDIO_URL = "ui-lab/assets/audio/compiler-first-fail.mp3";
  const COMPILER_FAIL_AUDIO_REVERSED_URL = "ui-lab/assets/audio/compiler-first-fail-reversed.mp3";
  const COMPILER_FAIL_AUDIO_DURATION = 9.273438;
  const COMPILER_FAIL_AUDIO_START = 42.25;
  const COMPILER_FAIL_AUDIO_END = COMPILER_FAIL_AUDIO_START + COMPILER_FAIL_AUDIO_DURATION;
  const PROMPT_AUDIO_VOLUME = 0.72;
  const PROMPT_AUDIO_MIN_RATE = 0.25;
  const PROMPT_AUDIO_MAX_RATE = 4;
  const PROMPT_AUDIO_SCROLL_BRIDGE_MS = 820;
  const PROMPT_AUDIO_VELOCITY_EASE_MS = 92;
  const PROMPT_AUDIO_RATE_EASE_MS = 185;
  const FILM_AUDIO_CUE_DEFINITIONS = Object.freeze([
    Object.freeze({
      id: "prompt",
      start: PROMPT_AUDIO_START,
      end: PROMPT_AUDIO_END,
      duration: PROMPT_AUDIO_DURATION,
      forwardUrl: PROMPT_AUDIO_URL,
      reverseUrl: PROMPT_AUDIO_REVERSED_URL,
      volume: PROMPT_AUDIO_VOLUME,
    }),
    Object.freeze({
      id: "kernel",
      start: KERNEL_AUDIO_START,
      end: KERNEL_AUDIO_END,
      duration: KERNEL_AUDIO_DURATION,
      forwardUrl: KERNEL_AUDIO_URL,
      reverseUrl: KERNEL_AUDIO_REVERSED_URL,
      volume: 0.72,
    }),
    Object.freeze({
      id: "genesis",
      start: GENESIS_AUDIO_START,
      end: GENESIS_AUDIO_END,
      duration: GENESIS_AUDIO_DURATION,
      forwardUrl: GENESIS_AUDIO_URL,
      reverseUrl: GENESIS_AUDIO_REVERSED_URL,
      volume: 0.72,
    }),
    Object.freeze({
      id: "agent",
      start: AGENT_AUDIO_START,
      end: AGENT_AUDIO_END,
      duration: AGENT_AUDIO_DURATION,
      forwardUrl: AGENT_AUDIO_URL,
      reverseUrl: AGENT_AUDIO_REVERSED_URL,
      volume: 0.72,
    }),
    Object.freeze({
      id: "builder",
      start: BUILDER_AUDIO_START,
      end: BUILDER_AUDIO_END,
      duration: BUILDER_AUDIO_DURATION,
      forwardUrl: BUILDER_AUDIO_URL,
      reverseUrl: BUILDER_AUDIO_REVERSED_URL,
      volume: 0.72,
    }),
    Object.freeze({
      id: "compiler-fail",
      start: COMPILER_FAIL_AUDIO_START,
      end: COMPILER_FAIL_AUDIO_END,
      duration: COMPILER_FAIL_AUDIO_DURATION,
      forwardUrl: COMPILER_FAIL_AUDIO_URL,
      reverseUrl: COMPILER_FAIL_AUDIO_REVERSED_URL,
      volume: 0.74,
    }),
  ]);
  const FILM_AUDIO_CAPTIONS = Object.freeze({
    kernel: Object.freeze([
      Object.freeze([0, 2.0, "The base is a custom"]),
      Object.freeze([2.0, KERNEL_AUDIO_DURATION, "Rust kernel."]),
    ]),
    genesis: Object.freeze([
      Object.freeze([0, 1.95, "On top of that, we have"]),
      Object.freeze([1.95, GENESIS_AUDIO_DURATION, "the Genesis layer."]),
    ]),
    agent: Object.freeze([
      Object.freeze([0, 2.45, "Now we can spawn an agent."]),
      Object.freeze([2.45, 4.65, "It gets granted access"]),
      Object.freeze([4.65, 6.65, "to the internet."]),
      Object.freeze([6.65, 9.0, "The key that opens the door"]),
      Object.freeze([9.0, 10.5, "comes directly from"]),
      Object.freeze([10.5, AGENT_AUDIO_DURATION, "the Genesis layer."]),
    ]),
    builder: Object.freeze([
      Object.freeze([0, 2.2, "The agent now requests"]),
      Object.freeze([2.2, 4.38, "a building playground."]),
      Object.freeze([4.38, 5.75, "Once it is set up,"]),
      Object.freeze([5.75, 7.45, "the agent can create Rust code"]),
      Object.freeze([7.45, 9.4, "inside a safe environment."]),
      Object.freeze([9.4, 10.8, "It also contains"]),
      Object.freeze([10.8, BUILDER_AUDIO_DURATION, "a Rust compiler and a guard."]),
    ]),
    "compiler-fail": Object.freeze([
      Object.freeze([0, 1.9, "The compiler compiles Rust"]),
      Object.freeze([1.9, 3.55, "into Wasm code."]),
      Object.freeze([3.55, 5.85, "Or it does NOT. God dammit."]),
      Object.freeze([5.85, COMPILER_FAIL_AUDIO_DURATION, "Let's try that again."]),
    ]),
  });
  // ================= SECTION: Scene, camera & scroll tuning =================
  const SCENE_FADE = 0.55;
  const CAMERA_REFERENCE_WIDTH = 1200;
  const CAMERA_REFERENCE_HEIGHT = 680;
  const CAMERA_WIDE_FACTOR = 0.92;
  const CAMERA_WIDE_DISTANCE_PER_ASPECT = 0.70;
  const CAMERA_WIDE_MAX_DISTANCE_OFFSET = 1.25;
  const CAMERA_VERTICAL_BIAS = 110;
  const CAMERA_WIDE_VERTICAL_BIAS_PER_ASPECT = 175;
  const CAMERA_WIDE_MAX_VERTICAL_BIAS = 220;
  const SCROLL_TRIGGER_FRAME_Y = 0.18;
  const SCROLL_TRIGGER_FRAME_Y_PER_CAMERA_DISTANCE = 0.12;
  const SCROLL_TRIGGER_FRAME_Y_MAX = 0.34;
  const SCROLL_TRIGGER_VIEWPORT_Y = 0.38;
  const SCROLL_TRIGGER_TOLERANCE = 0.06;
  const SCROLL_PAGE_TRAVEL_VH = 0.20;
  // Mobile deliberately shares the desktop scroll-trigger tuning above.
  const MOBILE_SCROLL_TRIGGER_FRAME_Y = SCROLL_TRIGGER_FRAME_Y;
  const MOBILE_SCROLL_TRIGGER_FRAME_Y_PER_CAMERA_DISTANCE = SCROLL_TRIGGER_FRAME_Y_PER_CAMERA_DISTANCE;
  const MOBILE_SCROLL_TRIGGER_FRAME_Y_MAX = SCROLL_TRIGGER_FRAME_Y_MAX;
  const MOBILE_SCROLL_TRIGGER_VIEWPORT_Y = SCROLL_TRIGGER_VIEWPORT_Y;
  const MOBILE_SCROLL_TRIGGER_TOLERANCE = SCROLL_TRIGGER_TOLERANCE;
  const MOBILE_SCROLL_PAGE_TRAVEL_VH = SCROLL_PAGE_TRAVEL_VH;
  const SCROLL_PAGE_TRAVEL_END_TIME = 6;
  const VIEWPORT_RESIZE_SETTLE_DELAY_MS = 90;
  const VIEWPORT_RESIZE_EASING_MS = 320;
  const WHEEL_SCRUB_EASING_MS = 175;
  const WHEEL_MAX_LEAD_SECONDS = 4;
  const WHEEL_SECONDS_PER_PIXEL = 0.006;
  const WHEEL_SLOW_SECONDS_PER_PIXEL = 0.0026;
  const TIMELINE_REVEAL_TIME = PROMPT_TYPE_START
    + (PROMPT_TYPE_END - PROMPT_TYPE_START) * 0.5;
  const COMPILER_FAIL_IMPRINT_START = 46.0;
  const COMPILER_FAIL_IMPRINT_END = 52.55;
  const COMPILER_FAIL_IMPRINT_PUNCH_SECONDS = 0.15;
  const COMPILER_FAIL_DECK_PRESS_PX = 1.8;
  const GENESIS_IMPRINT_CONSUME_START = 48.15;
  const GENESIS_IMPRINT_CONSUME_END = 49.55;
  const COMPILER_STOMP_CROUCH_START = 45.24;
  const COMPILER_STOMP_LAUNCH_START = 45.43;
  const COMPILER_STOMP_APEX = 45.68;
  const COMPILER_STOMP_IMPACT = COMPILER_FAIL_IMPRINT_START;
  const COMPILER_STOMP_SQUASH_END = 46.18;
  const COMPILER_STOMP_SETTLE_END = 46.72;
  const COMPILER_STOMP_TARGET_X = 4;
  const COMPILER_STOMP_TARGET_Y = -110;
  const COMPILER_STOMP_APEX_Y = -164;
  const COMPILER_WORKPIECE_X = 900;
  const COMPILER_WORKPIECE_Y = 330;
  const VERIFIER_WORKPIECE_X = 930;
  const VERIFIER_WORKPIECE_Y = 350;
  const GUARD_QUEUE_X = 880;
  const GUARD_QUEUE_Y = 360;
  const GUARD_STEP_START = 84.55;
  const GUARD_STEP_END = 85.25;
  const GUARD_SHIFT_X = -96;
  const GUARD_SHIFT_Y = 44;
  const LIVE_GRANT_REVOKE_START = 92.15;
  const LIVE_GRANT_REVOKE_END = 93.0;
  const VERIFIER_STATUS_X = 1067;
  const VERIFIER_STATUS_Y = 289;
  const LIVE_DOOR_X = 796;
  const LIVE_DOOR_Y = 271.5;
  const GUARD_GRANT_X = 796;
  const GUARD_GRANT_Y = 241.5;
  const COMPILER_ACTIVE_WINDOWS = Object.freeze([
    Object.freeze([42.25, 46.0]),
    Object.freeze([53.25, 55.4]),
    Object.freeze([67.5, 69.35]),
  ]);
  const COMPILER_SUCCESS_WINDOWS = Object.freeze([
    Object.freeze([55.4, 56.05]),
    Object.freeze([69.35, 70.0]),
  ]);
  const COMPILER_SUCCESS_DELIVERIES = Object.freeze([
    Object.freeze({ start: 55.4, attach: 55.72, carry: 56.18, handoff: 57.8, end: 58.55 }),
    Object.freeze({ start: 69.35, attach: 69.67, carry: 70.1, handoff: 71.2, end: 71.95 }),
  ]);

  // Per-scene visibility windows live in data-film-window attributes in
  // raios-ui-lab.html and may intentionally extend past the scene end for
  // crossfades (scene 2's window ends at 12.9 while the scene ends at 12.59).
  const sceneDefinitions = [
    [1, "sentence", "Prompt", 0, 3.89, 1, 1.15, 0.50, 0.49],
    [2, "world", "The world under the glass", 3.89, 12.59, 1, 1.35, 0.30, 0.47],
    [3, "net", "The agent needs the net", 12.59, 25, 1, 1.35, 0.30, 0.47],
    [4, "build-site", "Unlock the builder deck", 25, 33, 2, 0.82, 0.50, 0.50],
    [5, "inert-source", "Material becomes one workpiece", 33, 41, 2, 1.02, 0.72, 0.48],
    [6, "compiler", "Compiler round one", 41, 52, 2, 1.12, 0.74, 0.49],
    [7, "feedback", "Red report, precise fix", 52, 62, 2, 0.98, 0.58, 0.49],
    [8, "twins", "Twin build, byte equal", 62, 72, 2, 1.08, 0.74, 0.49],
    [9, "proof-cellar", "Harness tests in a disposable Shadow World", 72, 80, 2, 1.08, 0.76, 0.51],
    [10, "ring", "Guard binds report, hash, rights and owner", 80, 88, 3, 1.28, 0.50, 0.49],
    [11, "approval", "Guard opens the live door", 88, 96, 3, 0.98, 0.58, 0.49],
    [12, "running", "Player lands beside Genesis", 96, 106, 3, 1.38, 0.57, 0.49],
    [13, "factory", "The released world stays clean", 106, 114, 3, 0.92, 0.51, 0.50],
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
  const TIMELINE_WAYPOINTS = Object.freeze(SCENES.map((scene, index) => {
    const openingWaypoints = [
      { at: 3.89, title: "Rust kernel layer enters" },
      { at: 8.28, title: "Rust kernel text complete" },
      { at: 12.59, title: "Genesis layer and text complete" },
      { at: 24.72, title: "Agent connected through the net door" },
    ];
    const opening = openingWaypoints[index];
    return Object.freeze({
      number: scene.number,
      at: opening ? opening.at : scene.start,
      title: opening ? opening.title : scene.title,
    });
  }));

  // ================= SECTION: Camera model =================
  // Camera curve authored in the on-canvas editor and embedded as the reset
  // baseline. Runtime edits can still override it through localStorage.
  const CAMERA_KEYFRAMES = Object.freeze([
    { at: 4.34, scale: 1.349527665317139, focusX: 0.48, focusY: 0.47 },
    { at: 6.93, scale: 1.5151515151515151, focusX: 0.24, focusY: 0.47 },
    { at: 13.06, scale: 1.700004177047695, focusX: 0.2477268412672603, focusY: 0.5086342063363015 },
    { at: 14.74, scale: 1.5151515151515151, focusX: 0.38, focusY: 0.59 },
    { at: 24.2, scale: 1.5625, focusX: 0.41, focusY: 0.61 },
    { at: 28.5, scale: 1.5625, focusX: 0.45236168080038514, focusY: 0.5064492247101696 },
    { at: 33.31, scale: 1.4285714285714286, focusX: 0.49, focusY: 0.59 },
    { at: 45.91, scale: 1.5873015873015872, focusX: 0.57, focusY: 0.48 },
    { at: 79.95, scale: 1.5384615384615383, focusX: 0.61, focusY: 0.49 },
    { at: 81.26, scale: 1.0989010989010988, focusX: 0.48, focusY: 0.53 },
    { at: 86.41, scale: 1.0309278350515465, focusX: 0.48, focusY: 0.5 },
    { at: 88.44, scale: 1.639344262295082, focusX: 0.7, focusY: 0.39 },
    { at: 92.03, scale: 1.4492753623188408, focusX: 0.49, focusY: 0.34 },
    { at: 94.07, scale: 1.36986301369863, focusX: 0.52, focusY: 0.42 },
    { at: 107.12, scale: 1.4084507042253522, focusX: 0.32, focusY: 0.42 },
    { at: 115.49, scale: 0.8264462809917356, focusX: 0.46, focusY: 0.63 },
  ].map(Object.freeze));
  const CAMERA_STORAGE_KEY = "raios-film-camera-keyframes-v2";
  const cameraKeyframes = CAMERA_KEYFRAMES.map((frame) => ({ ...frame }));

  // ================= SECTION: Math & model helpers =================
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
  const cameraAspectExcess = () => {
    const viewportWidth = Math.max(1, window.innerWidth || CAMERA_REFERENCE_WIDTH);
    const viewportHeight = Math.max(1, window.innerHeight || CAMERA_REFERENCE_HEIGHT);
    const referenceAspect = CAMERA_REFERENCE_WIDTH / CAMERA_REFERENCE_HEIGHT;
    const viewportAspect = viewportWidth / viewportHeight;
    return Math.max(0, viewportAspect - referenceAspect);
  };
  const cameraAspectDistanceOffset = () => {
    return clamp(
      cameraAspectExcess() * CAMERA_WIDE_DISTANCE_PER_ASPECT,
      0,
      CAMERA_WIDE_MAX_DISTANCE_OFFSET,
    );
  };
  const cameraAspectVerticalBias = () => clamp(
    cameraAspectExcess() * CAMERA_WIDE_VERTICAL_BIAS_PER_ASPECT,
    0,
    CAMERA_WIDE_MAX_VERTICAL_BIAS,
  );
  const scrollFocusWindow = (time, start, end, feather = 0.7) => {
    const enter = smoothstep(intervalProgress(time, start - feather, start));
    const leave = 1 - smoothstep(intervalProgress(time, end, end + feather));
    return Math.min(enter, leave);
  };
  const wheelSecondsPerPixel = (time) => {
    const deliberateMoments = [
      [0, 6, 0.55], // prompt voice and type-on need fine, audible scroll steps
      [3.89, 12.92, 0.75], // Kernel, then Genesis: label lines burn and dock separately
      [32.6, 39.7, 0.9], // source files travel into the workpiece
      [45.5, 52.7, 0.7], // compiler diagnostic returns; edit 01 travels back
      [60.2, 67.2, 0.7], // verifier diagnostic returns; edit 02 travels back
      [87.7, 92.2, 0.55], // approved player crosses /out and reaches its deck
      [92.15, 95.8, 0.6], // capability links close while the builder releases
    ];
    const focus = deliberateMoments.reduce(
      (strongest, [start, end, feather]) => Math.max(
        strongest,
        scrollFocusWindow(time, start, end, feather),
      ),
      0,
    );
    const baseSpeed = lerp(WHEEL_SECONDS_PER_PIXEL, WHEEL_SLOW_SECONDS_PER_PIXEL, focus);
    const waypointThreeToFour = scrollFocusWindow(time, 12.59, 24.72, 0.35);
    return baseSpeed / lerp(1, 3, waypointThreeToFour);
  };

  function sceneAt(time) {
    if (time >= DURATION) return SCENES[SCENES.length - 1];
    return SCENES.find((scene) => time >= scene.start && time < scene.end) || SCENES[0];
  }

  function waypointAt(time) {
    let active = null;
    for (const waypoint of TIMELINE_WAYPOINTS) {
      if (time < waypoint.at) break;
      active = waypoint;
    }
    return active;
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
    const frames = cameraKeyframes.length > 0 ? cameraKeyframes : CAMERA_KEYFRAMES;
    let before = frames[0];
    let after = frames[frames.length - 1];
    for (let index = 1; index < frames.length; index += 1) {
      if (time <= frames[index].at) {
        after = frames[index];
        before = frames[index - 1];
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
      waypoints: TIMELINE_WAYPOINTS,
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
    // ================= SECTION: Setup & state =================
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
    let scrubSceneTitle = null;
    let scrubPlay = null;
    let scrubMarks = [];
    let cameraEditor = null;
    let cameraEditorTime = null;
    let cameraEditorMarkers = null;
    let cameraEditorStatus = null;
    let cameraEditorInputs = {};
    let cameraPreview = null;
    let responsiveCameraDistance = cameraAspectDistanceOffset();
    let responsiveCameraVerticalBias = cameraAspectVerticalBias();
    let promptAudioUnlocked = false;
    let promptAudioUnlockPending = false;
    let promptAudioFrame = 0;
    let promptAudioLastNow = performance.now();
    let promptAudioLastFilmTime = currentTime;
    let promptAudioLastMotionAt = 0;
    let promptAudioDirection = 0;
    let promptAudioPlayingDirection = 0;
    let promptAudioPlayingCue = "";
    let promptAudioVelocity = 0;
    let promptAudioScrollRate = 1;
    let promptAudioScrollIntentAt = 0;
    let promptAudioMotionHeld = false;
    let syncFilmScrollPosition = () => {};

    // ================= SECTION: Audio playback engine =================
    const createPromptAudioTrack = (source, direction, cueId) => {
      const track = new Audio(source);
      track.preload = "auto";
      track.volume = 0;
      track.loop = false;
      track.preservesPitch = true;
      track.mozPreservesPitch = true;
      track.webkitPreservesPitch = true;
      track.dataset.filmAudioDirection = direction;
      track.dataset.filmAudioCue = cueId;
      return track;
    };
    const filmAudioCues = FILM_AUDIO_CUE_DEFINITIONS.map((definition) => ({
      ...definition,
      rate: definition.duration / (definition.end - definition.start),
      forward: createPromptAudioTrack(definition.forwardUrl, "forward", definition.id),
      reverse: createPromptAudioTrack(definition.reverseUrl, "reverse", definition.id),
    }));
    const promptAudioTracks = filmAudioCues.flatMap((cue) => [cue.forward, cue.reverse]);

    const promptAudioPosition = (cue, time, reverse = false) => {
      const progress = intervalProgress(time, cue.start, cue.end);
      return (reverse ? 1 - progress : progress) * cue.duration;
    };
    const pausePromptAudioTrack = (track) => {
      if (!track.paused) track.pause();
      track.volume = 0;
    };
    const stopPromptAudio = (state = "idle") => {
      promptAudioTracks.forEach(pausePromptAudioTrack);
      promptAudioPlayingDirection = 0;
      promptAudioPlayingCue = "";
      root.dataset.filmPromptAudio = state;
      root.dataset.filmAudioCue = "none";
    };
    const playPromptAudioTrack = (track) => {
      if (!track.paused || track.ended || track.currentTime >= track.duration - 0.015) return;
      const attempt = track.play();
      if (attempt && typeof attempt.catch === "function") {
        attempt.catch(() => {
          promptAudioUnlocked = false;
          root.dataset.filmPromptAudio = "locked";
        });
      }
    };

    function renderPromptAudio(now) {
      const elapsedMs = clamp(now - promptAudioLastNow, 1, 80);
      const elapsedSeconds = elapsedMs / 1000;
      const filmDelta = currentTime - promptAudioLastFilmTime;
      const rawVelocity = filmDelta / elapsedSeconds;
      if (Math.abs(rawVelocity) > 0.012) {
        promptAudioDirection = rawVelocity > 0 ? 1 : -1;
        promptAudioLastMotionAt = now;
      }
      const scrollGap = now - promptAudioLastMotionAt;
      // Trackpad and wheel input arrives in packets. Between those packets we
      // retain the last useful velocity instead of collapsing the voice to its
      // minimum rate every frame.
      const velocityTarget = Math.abs(rawVelocity) > 0.012
        ? rawVelocity
        : scrollGap <= PROMPT_AUDIO_SCROLL_BRIDGE_MS
          ? promptAudioVelocity
          : 0;
      const velocityEase = 1 - Math.exp(-elapsedMs / PROMPT_AUDIO_VELOCITY_EASE_MS);
      promptAudioVelocity = lerp(promptAudioVelocity, velocityTarget, velocityEase);

      const activeCue = filmAudioCues.find((cue) => (
        currentTime >= cue.start && currentTime < cue.end - 0.01
      ));
      const inPrompt = Boolean(activeCue);
      const normalPlayback = playing;
      const scrollIntentFresh = now - promptAudioScrollIntentAt
        <= PROMPT_AUDIO_SCROLL_BRIDGE_MS;
      const moving = normalPlayback || promptAudioMotionHeld || scrollIntentFresh || (
        scrollGap <= PROMPT_AUDIO_SCROLL_BRIDGE_MS
        && Math.abs(promptAudioVelocity) > 0.018
      );
      // The Play button is the reference performance: narration always runs
      // at its recorded 1x tempo. Only manual scroll/scrub motion may bend it.
      const velocityRate = normalPlayback
        ? 1
        : scrollIntentFresh
          ? promptAudioScrollRate
          : clamp(
            Math.abs(promptAudioVelocity),
            PROMPT_AUDIO_MIN_RATE,
            PROMPT_AUDIO_MAX_RATE,
          );

      const activeTrack = activeCue
        ? promptAudioDirection < 0 ? activeCue.reverse : activeCue.forward
        : null;
      const inactiveTrack = activeCue
        ? promptAudioDirection < 0 ? activeCue.forward : activeCue.reverse
        : null;
      const expectedPosition = activeCue
        ? promptAudioPosition(activeCue, currentTime, promptAudioDirection < 0)
        : 0;
      const shouldPlay = promptAudioUnlocked && inPrompt && moving;

      if (shouldPlay) {
        const directionChanged = promptAudioPlayingDirection !== promptAudioDirection
          || promptAudioPlayingCue !== activeCue.id;
        if (directionChanged) {
          // A hard hand-off avoids the doubled voice/echo of a crossfade. The
          // new direction starts at the exact equivalent point in its buffer.
          pausePromptAudioTrack(inactiveTrack);
          pausePromptAudioTrack(activeTrack);
          promptAudioTracks.forEach((track) => {
            if (track !== activeTrack) pausePromptAudioTrack(track);
          });
          activeTrack.currentTime = clamp(expectedPosition, 0, activeCue.duration - 0.01);
          promptAudioPlayingDirection = promptAudioDirection;
          promptAudioPlayingCue = activeCue.id;
        }

        // Never seek a running voice to correct drift: even tiny backwards
        // corrections repeat phonemes. Let the narration gate follow the
        // uninterrupted media clock and adjust only its playback rate.
        const correctedRate = normalPlayback ? 1 : velocityRate;
        if (normalPlayback) {
          activeCue.rate = 1;
        } else {
          const rateEase = 1 - Math.exp(-elapsedMs / PROMPT_AUDIO_RATE_EASE_MS);
          activeCue.rate = lerp(activeCue.rate, correctedRate, rateEase);
        }
        activeTrack.playbackRate = activeCue.rate;
        inactiveTrack.playbackRate = activeCue.rate;

        const volumeEase = 1 - Math.exp(-elapsedMs / 28);
        activeTrack.volume = lerp(activeTrack.volume, activeCue.volume, volumeEase);
        pausePromptAudioTrack(inactiveTrack);
        playPromptAudioTrack(activeTrack);
        root.dataset.filmPromptAudio = promptAudioDirection < 0 ? "reverse" : "forward";
        root.dataset.filmAudioCue = activeCue.id;
      } else {
        const volumeEase = 1 - Math.exp(-elapsedMs / 95);
        promptAudioTracks.forEach((track) => {
          track.volume = lerp(track.volume, 0, volumeEase);
          if (track.volume < 0.01) {
            pausePromptAudioTrack(track);
            promptAudioPlayingDirection = 0;
            promptAudioPlayingCue = "";
          }
        });
        if (promptAudioUnlocked) root.dataset.filmPromptAudio = inPrompt ? "idle" : "ready";
        root.dataset.filmAudioCue = activeCue ? activeCue.id : "none";
      }

      promptAudioLastNow = now;
      promptAudioLastFilmTime = currentTime;
      promptAudioFrame = requestAnimationFrame(renderPromptAudio);
    }

    function unlockPromptAudio() {
      if (deterministic || reducedMotion || promptAudioUnlocked || promptAudioUnlockPending) return;
      promptAudioUnlockPending = true;
      const warmups = promptAudioTracks.map((track) => {
        track.volume = 0;
        track.currentTime = 0;
        const attempt = track.play();
        return attempt && typeof attempt.then === "function" ? attempt : Promise.resolve();
      });
      Promise.all(warmups).then(() => {
        promptAudioTracks.forEach((track) => track.pause());
        promptAudioUnlocked = true;
        promptAudioUnlockPending = false;
        promptAudioLastNow = performance.now();
        promptAudioLastFilmTime = currentTime;
        root.dataset.filmPromptAudio = "ready";
        if (!promptAudioFrame) promptAudioFrame = requestAnimationFrame(renderPromptAudio);
      }).catch(() => {
        promptAudioTracks.forEach(pausePromptAudioTrack);
        promptAudioUnlockPending = false;
        root.dataset.filmPromptAudio = "locked";
      });
    }

    root.dataset.filmPromptAudio = deterministic || reducedMotion ? "disabled" : "locked";
    root.addEventListener("pointerdown", unlockPromptAudio, { passive: true });
    window.addEventListener("keydown", unlockPromptAudio, { passive: true });

    // ================= SECTION: DOM hooks & wiring =================
    const normaliseCameraFrames = (frames) => {
      if (!Array.isArray(frames)) return [];
      const finiteValue = (value, fallback) => {
        const parsed = Number.parseFloat(value);
        return Number.isFinite(parsed) ? parsed : fallback;
      };
      return frames.map((frame) => ({
        at: parseTime(frame && frame.at, 0),
        scale: clamp(finiteValue(frame && frame.scale, 1), 0.45, 2.4),
        focusX: clamp(finiteValue(frame && frame.focusX, 0.5), 0, 1),
        focusY: clamp(finiteValue(frame && frame.focusY, 0.5), 0, 1),
      })).sort((a, b) => a.at - b.at);
    };
    const replaceCameraFrames = (frames) => {
      if (!Array.isArray(frames)) return false;
      const normalised = normaliseCameraFrames(frames);
      cameraKeyframes.splice(0, cameraKeyframes.length, ...normalised);
      return true;
    };
    const saveCameraFrames = () => {
      try {
        localStorage.setItem(CAMERA_STORAGE_KEY, JSON.stringify(cameraKeyframes));
      } catch (_) {
        // The editor still works for this session when storage is unavailable.
      }
    };
    try {
      const storedCameraFrames = localStorage.getItem(CAMERA_STORAGE_KEY);
      if (storedCameraFrames) replaceCameraFrames(JSON.parse(storedCameraFrames));
    } catch (_) {
      // Invalid or unavailable local storage falls back to source keyframes.
    }

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
        backgroundDim: selectOne("#film-background-dim"),
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
        approvalButtonCopy: selectOne("#film-approve-button-copy"),
        approvalButtonSubcopy: selectOne("#film-approve-button-subcopy"),
        approveOwnerBinding: selectOne("#film-approve-owner-binding"),
        remoteDenied: selectOne("#film-remote-denied", ".film-remote-denied"),
        netVein: selectOne("#film-net"),
        netPulseRings: selectAll("#film-net-node .film-net-pulse-ring"),
        netSignal: selectOne("#film-net-node .film-net-signal"),
        netBeacon: selectOne("#film-net-node .film-net-beacon"),
        reclog: selectOne("#film-reclog"),
        titleCard: selectOne("#film-title-card", ".film-title"),
        titleSubtitle: selectOne(".film-subtitle"),
        voiceCaption: selectOne("#film-voice-caption"),
        voiceCaptionText: selectOne("#film-voice-caption-text"),
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
        builderSurface: selectOne("#film-builder-surface"),
        builderFloor: selectOne("#film-builder-deck .film-builder-floor"),
        builderHatch: selectOne("#film-builder-deck .film-builder-hatch"),
        genesisSurface: selectOne("#film-genesis-surface"),
        genesisFloor: selectOne("#film-genesis-layer .film-floor"),
        keyForge: selectOne("#film-key-forge"),
        keyForgeCore: selectOne("#film-key-forge-core"),
        keyForgeBurst: selectOne("#film-key-forge-burst"),
        genesisKeyNodes: selectAll("[data-film-genesis-key]"),
        genesisKeyRoutes: selectAll(".film-genesis-key-routes path"),
        kernelSide: selectOne("#film-kernel-side"),
        kernelFacet: selectOne("#film-kernel-facet"),
        kernelGlow: selectOne("#film-kernel-glow"),
        kernelCalloutCard: selectOne("#film-kernel-callout .film-callout-card"),
        genesisCalloutCard: selectOne("#film-genesis-callout .film-callout-card"),
        builderCalloutCard: selectOne("#film-builder-callout .film-callout-card"),
        kernelCallout: selectOne("#film-kernel-callout"),
        genesisCallout: selectOne("#film-genesis-callout"),
        builderCallout: selectOne("#film-builder-callout"),
        genesisCalloutLine: selectOne("#film-genesis-callout .film-callout-line"),
        genesisCalloutTarget: selectOne("#film-genesis-callout .film-callout-target"),
        kernelLayerTitle: selectOne("#film-kernel-layer-title"),
        genesisLayerTitle: selectOne("#film-genesis-layer-title"),
        builderLayerTitle: selectOne("#film-builder-layer-title"),
        compilerProgress: selectOne("#film-compiler-progress"),
        compilerRound: selectOne("#film-compiler-round"),
        compiler: selectOne("#film-workshop-compiler"),
        compilerBody: selectOne("#film-workshop-compiler-body"),
        compilerSuccess: selectOne("#film-compiler-success"),
        imprintBuilder: selectOne("#film-imprint-builder"),
        imprintBuilderPunch: selectOne("#film-imprint-builder-punch"),
        imprintWorkpiece: selectOne("#film-imprint-workpiece"),
        imprintWorkpiecePunch: selectOne("#film-imprint-workpiece-punch"),
        imprintGenesisConsume: selectOne("#film-imprint-genesis-consume"),
        imprintGenesis: selectOne("#film-imprint-genesis"),
        imprintGenesisPunch: selectOne("#film-imprint-genesis-punch"),
        verifierProgress: selectOne("#film-verifier-progress"),
        verifierRound: selectOne("#film-verifier-round"),
        verifier: selectOne("#film-workshop-verifier"),
        shadowWorld: selectOne("#film-shadow-world"),
        shadowEntryRoute: selectOne("#film-shadow-entry-route"),
        shadowEntryDoor: selectOne("#film-shadow-entry-door"),
        shadowPlayer: selectOne("#film-shadow-player"),
        shadowAttacks: selectOne("#film-shadow-attacks"),
        shadowAttackPaths: selectAll("#film-shadow-attacks .film-shadow-attack"),
        shadowImpacts: selectAll("#film-shadow-attacks .film-shadow-impact"),
        verifierThumb: selectOne("#film-verifier-thumb"),
        testReport: selectOne("#film-test-report"),
        guard: selectOne("#film-workshop-guard"),
        guardChecks: selectOne("#film-guard-checks"),
        guardReport: selectOne("#film-guard-report"),
        guardHash: selectOne("#film-guard-hash"),
        guardCaps: selectOne("#film-guard-caps"),
        guardOwner: selectOne("#film-guard-owner"),
        workpiece: selectOne("#film-workpiece"),
        workpieceBody: selectOne("#film-workpiece-body"),
        workpieceLabel: selectOne("#film-workpiece-label"),
        sealTwin: selectOne("#film-seal-twin"),
        sealDrills: selectOne("#film-seal-drills"),
        sealReport: selectOne("#film-seal-report"),
        sealReportText: selectOne("#film-seal-report text"),
        outGateStatus: selectOne("#film-out-gate-status"),
        outGateCopy: selectOne("#film-out-gate-copy"),
        outDoor: selectOne("#film-out-door"),
        liveGrantLine: selectOne("#film-live-grant-line"),
        liveGrantPulse: selectOne("#film-live-grant-pulse"),
        liveDataPath: selectOne("#film-live-data-path"),
        liveDataDots: selectAll("[data-film-live-data-dot]"),
        drillWall: selectOne("#film-drill-wall"),
        drillImport: selectOne("#film-drill-import"),
        drillFuel: selectOne("#film-drill-fuel"),
        workshopLinks: selectAll("[data-film-workshop-link]"),
        materialPath: selectOne("#film-material-path"),
        materialMain: selectOne("#film-material-main"),
        materialCargo: selectOne("#film-material-cargo"),
        editOneFiles: selectAll("#film-edit-one [data-film-edit-file]"),
        editTwoFiles: selectAll("#film-edit-two [data-film-edit-file]"),
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

    // ================= SECTION: Scene animators =================
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
      const camera = cameraPreview && Math.abs(cameraPreview.at - time) < 0.05
        ? cameraPreview
        : cameraAt(time);
      const authoredDistance = 1 / Math.max(0.001, camera.scale);
      const responsiveDistance = authoredDistance + responsiveCameraDistance;
      const cameraScale = CAMERA_WIDE_FACTOR / responsiveDistance;
      const viewBox = svg.viewBox && svg.viewBox.baseVal;
      const boxX = viewBox && viewBox.width ? viewBox.x : 0;
      const boxY = viewBox && viewBox.height ? viewBox.y : 0;
      const boxWidth = viewBox && viewBox.width ? viewBox.width : 1000;
      const boxHeight = viewBox && viewBox.height ? viewBox.height : 640;
      const centerX = boxX + boxWidth / 2;
      const centerY = boxY + boxHeight / 2;
      const focusX = CAMERA_REFERENCE_WIDTH * camera.focusX;
      const focusY = CAMERA_REFERENCE_HEIGHT * camera.focusY
        + CAMERA_VERTICAL_BIAS
        + responsiveCameraVerticalBias;
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
        } else if (action === "draw") {
          node.setAttribute("pathLength", "1");
          node.style.strokeDasharray = "1";
          node.style.strokeDashoffset = (1 - progress).toFixed(4);
          setOpacity(node, started ? 1 : 0);
        } else if (action === "trace") {
          const revealed = progress * 100;
          const dashPattern = [];
          let covered = 0;
          let drawing = true;
          while (covered < revealed - 0.001) {
            const segment = drawing ? 4 : 6;
            const amount = Math.min(segment, revealed - covered);
            dashPattern.push(amount.toFixed(3));
            covered += amount;
            drawing = !drawing;
          }
          if (drawing) dashPattern.push("0.000");
          dashPattern.push(Math.max(0, 100 - revealed).toFixed(3));
          node.setAttribute("pathLength", "100");
          node.style.strokeDasharray = dashPattern.join(" ");
          node.style.strokeDashoffset = "0";
          setOpacity(node, started ? 1 : 0);
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

    function renderGenesisKeys(time) {
      let forgePulse = 0;
      (hooks.genesisKeyNodes || []).forEach((node) => {
        const start = Number.parseFloat(node.dataset.filmKeyStart || "0") || 0;
        const detach = Number.parseFloat(node.dataset.filmKeyDetach || String(start)) || start;
        const end = Number.parseFloat(node.dataset.filmKeyEnd || String(detach)) || detach;
        const targetScale = Number.parseFloat(node.dataset.filmKeyScale || ".48") || 0.48;
        const routeId = node.dataset.filmKeyRoute || "";
        const route = routeId ? selectOne(`#${routeId}`) : null;
        const grow = smootherstep(intervalProgress(time, start, detach));
        const flight = smootherstep(intervalProgress(time, detach, end));
        const outro = 1 - smoothstep(intervalProgress(time, end + 0.2, end + 0.52));
        const active = time >= start && time < end + 0.52;

        if (route && typeof route.getTotalLength === "function") {
          const length = route.getTotalLength();
          const point = route.getPointAtLength(length * flight);
          const emergenceY = lerp(18, 0, grow);
          const snap = time >= detach && time < detach + 0.24
            ? Math.sin(intervalProgress(time, detach, detach + 0.24) * Math.PI)
            : 0;
          const scale = targetScale * lerp(0.04, 1, grow) * (1 + snap * 0.16);
          const rotation = lerp(-22, 0, grow) + flight * 64 - Math.sin(flight * Math.PI) * 18;
          setComposedTransform(
            node,
            `translate(${point.x.toFixed(3)} ${(point.y + emergenceY).toFixed(3)}) rotate(${rotation.toFixed(3)} 40 16) scale(${scale.toFixed(5)})`,
          );
          setOpacity(node, active ? grow * outro : 0);
          node.dataset.filmKeyState = !active
            ? time < start ? "seed" : "granted"
            : flight > 0 ? "flying" : grow < 1 ? "growing" : "detaching";

          const routeIntro = smoothstep(intervalProgress(time, start + 0.12, detach));
          const routeOutro = 1 - smoothstep(intervalProgress(time, end - 0.08, end + 0.18));
          route.style.strokeDashoffset = (-(time - start) * 28).toFixed(3);
          route.dataset.filmKeyRouteState = flight > 0 ? "granting" : "forming";
          setOpacity(route, routeIntro * routeOutro * 0.72);
        } else {
          setOpacity(node, 0);
        }

        const growPulse = time >= start && time < detach
          ? Math.sin(intervalProgress(time, start, detach) * Math.PI) * 0.48
          : 0;
        const detachPulse = time >= detach - 0.06 && time < detach + 0.32
          ? Math.sin(intervalProgress(time, detach - 0.06, detach + 0.32) * Math.PI)
          : 0;
        forgePulse = Math.max(forgePulse, growPulse, detachPulse);
      });

      if (hooks.keyForgeCore) {
        hooks.keyForgeCore.setAttribute("r", (7 + forgePulse * 4.5).toFixed(3));
        hooks.keyForgeCore.dataset.filmForging = String(forgePulse > 0.01);
      }
      if (hooks.keyForgeBurst) {
        setComposedTransform(hooks.keyForgeBurst, `scale(${(0.72 + forgePulse * 0.78).toFixed(4)})`);
        setOpacity(hooks.keyForgeBurst, forgePulse);
      }
    }

    function renderDoorCeremonies(time) {
      doorLeaves.forEach((leaf) => {
        const openAt = Number.parseFloat(leaf.dataset.filmDoorAt || "0") || 0;
        const progress = smoothstep(intervalProgress(time, openAt, openAt + 0.9));
        const mix = (from, to, amount) => from + (to - from) * amount;
        const hingeTop = { x: 25, y: 12 };
        const hingeBottom = { x: 25, y: 68 };
        const freeTop = { x: mix(61, -10, progress), y: mix(12, 20, progress) };
        const freeBottom = { x: mix(61, -10, progress), y: mix(68, 76, progress) };
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
        const door = leaf.closest(".film-door-instance");
        if (door) door.dataset.filmDoorOpen = String(progress >= 1);
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
        const progress = intervalProgress(time, PROMPT_TYPE_START, PROMPT_TYPE_END);
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

    function renderBackgroundDim(time) {
      const progress = smootherstep(intervalProgress(time, PROMPT_TYPE_END, 8.01));
      setOpacity(hooks.backgroundDim, progress);
    }

    function renderNetNode(time) {
      const start = 13.2;
      const activeTime = Math.max(0, time - start);
      (hooks.netPulseRings || []).forEach((ring, index) => {
        const cycle = 3.6;
        const phase = ((activeTime - index * 1.2) % cycle + cycle) % cycle / cycle;
        const opacity = time < start
          ? 0
          : phase < 0.18
            ? lerp(0, 0.85, phase / 0.18)
            : lerp(0.85, 0, (phase - 0.18) / 0.82);
        ring.setAttribute("r", lerp(3, 30, phase).toFixed(3));
        setOpacity(ring, opacity);
      });
      if (hooks.netSignal) {
        const angle = time < start ? 0 : (activeTime / 24 * 360) % 360;
        setComposedTransform(hooks.netSignal, `rotate(${angle.toFixed(3)} 59 28)`);
      }
      if (hooks.netBeacon) {
        const phase = ((activeTime % 1.6) + 1.6) % 1.6 / 1.6;
        const opacity = time < start ? 0 : 0.6 + 0.4 * Math.cos(phase * Math.PI * 2);
        setOpacity(hooks.netBeacon, opacity);
      }
    }

    function renderLayerCallouts(time) {
      // Keep the callout target on the same moving face as the Genesis layer.
      // These times deliberately mirror the layer's declarative rise action.
      const genesisRise = smoothstep(intervalProgress(time, 8.28, 10.58));
      const genesisLift = 118 * (1 - genesisRise);
      const genesisTargetX = 188;
      const genesisTargetY = 431 + genesisLift;
      if (hooks.genesisCalloutLine) {
        hooks.genesisCalloutLine.setAttribute(
          "d",
          `M315 372C292 ${(392 + genesisLift * 0.18).toFixed(2)} 245 ${(420 + genesisLift * 0.62).toFixed(2)} ${genesisTargetX} ${genesisTargetY.toFixed(2)}`,
        );
      }
      if (hooks.genesisCalloutTarget) {
        hooks.genesisCalloutTarget.setAttribute("cx", String(genesisTargetX));
        hooks.genesisCalloutTarget.setAttribute("cy", genesisTargetY.toFixed(2));
      }
      const burnCallout = (node, start, tempo = 1) => {
        if (!node) return 0;
        const at = (offset) => start + offset * tempo;
        const outline = smootherstep(intervalProgress(time, start, at(2.15)));
        const line = smootherstep(intervalProgress(time, at(0.25), at(2.65)));
        const target = smootherstep(intervalProgress(time, at(1.05), at(2.35)));
        const titleProgress = smootherstep(intervalProgress(time, at(0.72), at(2.2)));
        const copyProgress = smootherstep(intervalProgress(time, at(1.32), at(2.75)));
        const flicker = clamp(
          0.86 + Math.sin(time * 29.0) * 0.09 + Math.sin(time * 47.0) * 0.05,
          0.68,
          1,
        );
        const title = titleProgress >= 0.985 ? 1 : titleProgress * flicker;
        const copy = copyProgress >= 0.985 ? 1 : copyProgress * (0.9 + flicker * 0.1);
        const fill = smootherstep(intervalProgress(time, at(0.48), at(2.4)));
        const heat = outline * (0.12 + (1 - fill) * (0.42 + flicker * 0.28));
        node.style.setProperty("--film-callout-outline", outline.toFixed(4));
        node.style.setProperty("--film-callout-line", line.toFixed(4));
        node.style.setProperty("--film-callout-target", target.toFixed(4));
        node.style.setProperty("--film-callout-title", title.toFixed(4));
        node.style.setProperty("--film-callout-copy", copy.toFixed(4));
        node.style.setProperty("--film-callout-fill", fill.toFixed(4));
        node.style.setProperty("--film-callout-heat", heat.toFixed(4));
        node.dataset.filmBurnState = copyProgress >= 1
          ? "burned"
          : outline > 0
            ? "burning"
            : "waiting";
        return Math.max(outline, fill, title, copy);
      };
      const kernelBurn = burnCallout(hooks.kernelCallout, 3.89, 0.38);
      const genesisBurn = burnCallout(hooks.genesisCallout, 8.28, 0.38);
      const builderBurn = burnCallout(hooks.builderCallout, 27.45, 0.6);
      const calloutPhase = (introStart, introEnd, dockStart, dockEnd, titleStart) => ({
        intro: smootherstep(intervalProgress(time, introStart, introEnd)),
        docking: smootherstep(intervalProgress(time, dockStart, dockEnd)),
        cardExit: 1 - smootherstep(intervalProgress(time, titleStart, dockEnd)),
        titleOpacity: smootherstep(intervalProgress(time, titleStart, dockEnd)),
      });
      const kernelPhase = calloutPhase(3.99, 4.79, 6.63, 8.28, 7.38);
      const genesisPhase = calloutPhase(8.38, 9.18, 10.58, 12.59, 11.59);
      const builderPhase = calloutPhase(28.25, 29.05, 31.45, 32.9, 32.0);
      const cardTransform = (x, y, angle, pivotX, pivotY, phase) => {
        const { intro, docking } = phase;
        const offsetX = x * docking;
        const offsetY = 18 * (1 - intro) + y * docking;
        const introScale = 0.94 + intro * 0.06;
        const outroScale = 1 - docking * 0.62;
        return `translate(${offsetX.toFixed(3)} ${offsetY.toFixed(3)}) rotate(${(angle * docking).toFixed(3)} ${pivotX} ${pivotY}) translate(${pivotX} ${pivotY}) scale(${(introScale * outroScale).toFixed(4)}) translate(${-pivotX} ${-pivotY})`;
      };

      setComposedTransform(
        hooks.kernelCalloutCard,
        cardTransform(-162, 131, 26.565, 315, 328, kernelPhase),
      );
      setComposedTransform(
        hooks.genesisCalloutCard,
        cardTransform(-127, 125, 26.565, 315, 328, genesisPhase),
      );
      setComposedTransform(
        hooks.builderCalloutCard,
        cardTransform(172, 125, 26.565, 600, 328, builderPhase),
      );
      setOpacity(hooks.kernelCalloutCard, kernelBurn * kernelPhase.cardExit);
      setOpacity(hooks.genesisCalloutCard, genesisBurn * genesisPhase.cardExit);
      setOpacity(hooks.builderCalloutCard, builderBurn * builderPhase.cardExit);
      setOpacity(hooks.kernelLayerTitle, kernelPhase.titleOpacity);
      setOpacity(hooks.genesisLayerTitle, genesisPhase.titleOpacity);
      setOpacity(hooks.builderLayerTitle, builderPhase.titleOpacity);
      if (hooks.kernelLayerTitle) {
        hooks.kernelLayerTitle.style.setProperty("--film-layer-title-burn", kernelPhase.titleOpacity.toFixed(4));
      }
      if (hooks.genesisLayerTitle) {
        hooks.genesisLayerTitle.style.setProperty("--film-layer-title-burn", genesisPhase.titleOpacity.toFixed(4));
      }
      if (hooks.builderLayerTitle) {
        hooks.builderLayerTitle.style.setProperty("--film-layer-title-burn", builderPhase.titleOpacity.toFixed(4));
      }
    }

    function renderBuilderArchitecture(time) {
      const foundationExpansion = smoothstep(intervalProgress(time, 13.02, 14.82));
      const stageValue = (initial, final) => lerp(initial, final, foundationExpansion);
      const stagePoint = (initialX, initialY, finalX, finalY) => ({
        x: stageValue(initialX, finalX),
        y: stageValue(initialY, finalY),
      });
      const point = ({ x, y }) => `${x.toFixed(2)} ${y.toFixed(2)}`;
      const left = stagePoint(0, 345, 24, 360);
      const top = stagePoint(315, 187.5, 600, 72);
      const right = stagePoint(630, 345, 1176, 360);
      const bottom = stagePoint(315, 502.5, 600, 648);
      const leftLower = stagePoint(0, 415, 24, 420);
      const rightLower = stagePoint(630, 415, 1176, 420);
      const bottomLower = stagePoint(315, 572.5, 600, 708);
      if (hooks.kernelFacet) hooks.kernelFacet.setAttribute("d", `M${point(left)}L${point(top)}L${point(right)}L${point(bottom)}Z`);
      if (hooks.kernelSide) hooks.kernelSide.setAttribute("d", `M${point(left)}L${point(bottom)}L${point(right)}L${point(rightLower)}L${point(bottomLower)}L${point(leftLower)}Z`);
      if (hooks.kernelGlow) {
        const glowCenterX = stageValue(315, 600);
        const glowCenterY = stageValue(560, 670);
        const glowRadiusX = stageValue(310, 535);
        const glowRadiusY = stageValue(38, 44);
        hooks.kernelGlow.setAttribute(
          "d",
          `M${(glowCenterX - glowRadiusX).toFixed(2)} ${glowCenterY.toFixed(2)}`
            + `L${glowCenterX.toFixed(2)} ${(glowCenterY - glowRadiusY).toFixed(2)}`
            + `L${(glowCenterX + glowRadiusX).toFixed(2)} ${glowCenterY.toFixed(2)}`
            + `L${glowCenterX.toFixed(2)} ${(glowCenterY + glowRadiusY).toFixed(2)}Z`,
        );
      }

      const stickTitleToEdge = (node, from, to, amount, offsetY = 0) => {
        if (!node || !from || !to) return;
        const x = lerp(from.x, to.x, amount);
        const y = lerp(from.y, to.y, amount) + offsetY;
        const angle = Math.atan2(to.y - from.y, to.x - from.x) * 180 / Math.PI;
        node.setAttribute("x", "0");
        node.setAttribute("y", "0");
        node.setAttribute(
          "transform",
          `translate(${x.toFixed(2)} ${y.toFixed(2)}) rotate(${angle.toFixed(3)})`,
        );
      };
      stickTitleToEdge(hooks.kernelLayerTitle, left, bottom, 0.485, 38);
      if (hooks.genesisFloor && hooks.genesisLayerTitle
          && typeof hooks.genesisFloor.getTotalLength === "function") {
        const length = hooks.genesisFloor.getTotalLength();
        const genesisLeft = hooks.genesisFloor.getPointAtLength(0);
        const genesisBottom = hooks.genesisFloor.getPointAtLength(length * 0.75);
        stickTitleToEdge(hooks.genesisLayerTitle, genesisLeft, genesisBottom, 0.5, 22);
      }
      if (hooks.builderFloor && hooks.builderLayerTitle
          && typeof hooks.builderFloor.getTotalLength === "function") {
        const length = hooks.builderFloor.getTotalLength();
        const builderLeft = hooks.builderFloor.getPointAtLength(0);
        const builderBottom = hooks.builderFloor.getPointAtLength(length * 0.75);
        stickTitleToEdge(hooks.builderLayerTitle, builderLeft, builderBottom, 0.5, 22);
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

      const compileWindow = COMPILER_ACTIVE_WINDOWS.find(([start, end]) => (
        time >= start && time < end
      ));
      let shakeX = 0;
      let shakeY = 0;
      let shakeRotation = 0;
      if (compileWindow) {
        const [start, end] = compileWindow;
        const rampIn = smoothstep(intervalProgress(time, start, start + 0.16));
        const rampOut = 1 - smoothstep(intervalProgress(time, end - 0.12, end));
        const strength = Math.min(rampIn, rampOut);
        shakeX = (
          Math.sin(time * Math.PI * 25.4)
          + Math.sin(time * Math.PI * 41.8) * 0.42
        ) * 1.7 * strength;
        shakeY = (
          Math.sin(time * Math.PI * 31.2 + 0.8)
          + Math.sin(time * Math.PI * 18.6) * 0.36
        ) * 1.25 * strength;
        shakeRotation = Math.sin(time * Math.PI * 22.8 + 1.3) * 0.72 * strength;
      }
      let stompX = 0;
      let stompY = 0;
      let stompScaleX = 1;
      let stompScaleY = 1;
      let stompRotation = 0;
      let stompState = "idle";
      if (time >= COMPILER_STOMP_CROUCH_START && time < COMPILER_STOMP_LAUNCH_START) {
        const crouch = smoothstep(intervalProgress(
          time,
          COMPILER_STOMP_CROUCH_START,
          COMPILER_STOMP_LAUNCH_START,
        ));
        stompY = 4 * crouch;
        stompScaleX = lerp(1, 1.06, crouch);
        stompScaleY = lerp(1, 0.86, crouch);
        stompRotation = -0.55 * crouch;
        stompState = "crouching";
      } else if (time >= COMPILER_STOMP_LAUNCH_START && time < COMPILER_STOMP_APEX) {
        const launch = smootherstep(intervalProgress(
          time,
          COMPILER_STOMP_LAUNCH_START,
          COMPILER_STOMP_APEX,
        ));
        stompX = lerp(0, COMPILER_STOMP_TARGET_X * 0.3, launch);
        stompY = lerp(4, COMPILER_STOMP_APEX_Y, launch);
        stompScaleX = lerp(1.06, 0.98, launch);
        stompScaleY = lerp(0.86, 1.04, launch);
        stompRotation = lerp(-0.55, 0.7, launch);
        stompState = "launching";
      } else if (time >= COMPILER_STOMP_APEX && time < COMPILER_STOMP_IMPACT) {
        const fall = intervalProgress(time, COMPILER_STOMP_APEX, COMPILER_STOMP_IMPACT);
        const dive = fall * fall;
        stompX = lerp(COMPILER_STOMP_TARGET_X * 0.3, COMPILER_STOMP_TARGET_X, fall);
        stompY = lerp(COMPILER_STOMP_APEX_Y, COMPILER_STOMP_TARGET_Y, dive);
        stompScaleX = lerp(0.98, 1, fall);
        stompScaleY = lerp(1.04, 1, fall);
        stompRotation = Math.sin(fall * Math.PI) * 0.85;
        stompState = "falling";
      } else if (time >= COMPILER_STOMP_IMPACT && time < COMPILER_STOMP_SQUASH_END) {
        const impact = intervalProgress(time, COMPILER_STOMP_IMPACT, COMPILER_STOMP_SQUASH_END);
        const pulse = Math.sin(impact * Math.PI);
        stompX = COMPILER_STOMP_TARGET_X;
        stompY = COMPILER_STOMP_TARGET_Y + 5 * pulse;
        stompScaleX = 1 + 0.12 * pulse;
        stompScaleY = 1 - 0.17 * pulse;
        stompRotation = -0.7 * pulse;
        stompState = "impact";
      } else if (time >= COMPILER_STOMP_SQUASH_END && time < COMPILER_STOMP_SETTLE_END) {
        const settle = intervalProgress(time, COMPILER_STOMP_SQUASH_END, COMPILER_STOMP_SETTLE_END);
        const returnEase = smootherstep(settle);
        const rebound = Math.sin(settle * Math.PI) * (1 - settle);
        stompX = lerp(COMPILER_STOMP_TARGET_X, 0, returnEase);
        stompY = lerp(COMPILER_STOMP_TARGET_Y, 0, returnEase) - 18 * rebound;
        stompScaleX = 1 - 0.035 * rebound;
        stompScaleY = 1 + 0.05 * rebound;
        stompRotation = 0.35 * rebound;
        stompState = "settling";
      }
      const successWindow = COMPILER_SUCCESS_WINDOWS.find(([start, end]) => (
        time >= start && time < end
      ));
      let joyY = 0;
      let joyScaleX = 1;
      let joyScaleY = 1;
      let joyRotation = 0;
      if (successWindow) {
        const [start, end] = successWindow;
        const joy = intervalProgress(time, start, end);
        const hop = Math.sin(joy * Math.PI);
        const wiggle = Math.sin(joy * Math.PI * 4) * (1 - joy);
        joyY = -12 * hop;
        joyScaleX = 1 - 0.045 * hop;
        joyScaleY = 1 + 0.065 * hop;
        joyRotation = 2.2 * wiggle;
      }
      setComposedTransform(
        hooks.compilerBody,
        `translate(${(shakeX + stompX).toFixed(3)} ${(shakeY + stompY + joyY).toFixed(3)})`
          + ` rotate(${(shakeRotation + stompRotation + joyRotation).toFixed(3)} 80 92)`
          + ` translate(80 116) scale(${(stompScaleX * joyScaleX).toFixed(5)} ${(stompScaleY * joyScaleY).toFixed(5)}) translate(-80 -116)`,
      );
      if (hooks.compilerBody) {
        hooks.compilerBody.dataset.filmCompiling = String(Boolean(compileWindow));
        hooks.compilerBody.dataset.filmStomp = stompState;
        hooks.compilerBody.dataset.filmCelebrating = String(Boolean(successWindow));
      }
      if (hooks.compilerSuccess) {
        const delivery = COMPILER_SUCCESS_DELIVERIES.find(({ start, end }) => (
          time >= start && time < end
        ));
        const pop = delivery
          ? smootherstep(intervalProgress(time, delivery.start, delivery.attach))
          : 0;
        const attach = delivery
          ? smootherstep(intervalProgress(time, delivery.attach, delivery.carry))
          : 0;
        const handoff = delivery
          ? smootherstep(intervalProgress(time, delivery.handoff, delivery.end))
          : 0;
        const carriedX = lerp(33, 30, attach);
        const carriedY = lerp(-79, -26, attach);
        const verifierTargetX = VERIFIER_STATUS_X - VERIFIER_WORKPIECE_X;
        const verifierTargetY = VERIFIER_STATUS_Y - VERIFIER_WORKPIECE_Y;
        const tokenX = lerp(carriedX, verifierTargetX, handoff);
        const tokenY = lerp(carriedY, verifierTargetY, handoff)
          - Math.sin(handoff * Math.PI) * 10;
        const carriedScale = lerp(0.62, 0.5, attach);
        const tokenScale = pop * lerp(carriedScale, 0.12, handoff);
        const handoffFade = 1 - smoothstep(intervalProgress(handoff, 0.72, 1));
        setComposedTransform(
          hooks.compilerSuccess,
          `translate(${tokenX.toFixed(3)} ${tokenY.toFixed(3)}) scale(${tokenScale.toFixed(4)}) rotate(${(8 * (1 - pop) - 10 * handoff).toFixed(3)})`,
        );
        setOpacity(hooks.compilerSuccess, delivery ? handoffFade : 0);
        hooks.compilerSuccess.style.setProperty(
          "--film-success-draw",
          (delivery
            ? smoothstep(intervalProgress(time, delivery.start + 0.08, delivery.attach + 0.08))
            : 0).toFixed(4),
        );
        hooks.compilerSuccess.dataset.filmSuccessState = !delivery
          ? "waiting"
          : handoff > 0
            ? "handoff"
            : attach >= 1
              ? "carried"
              : "attaching";
        if (hooks.verifier) {
          hooks.verifier.dataset.filmReceiving = String(Boolean(delivery && handoff > 0));
        }
      }
      const compilerForeground = time >= COMPILER_STOMP_LAUNCH_START
        && time < COMPILER_STOMP_SETTLE_END;
      const compilerParent = hooks.compiler?.parentNode;
      if (compilerParent && hooks.imprintBuilder?.parentNode === compilerParent) {
        if (compilerForeground && compilerParent.lastElementChild !== hooks.compiler) {
          compilerParent.appendChild(hooks.compiler);
        } else if (!compilerForeground
            && hooks.compiler.nextElementSibling !== hooks.imprintBuilder) {
          compilerParent.insertBefore(hooks.compiler, hooks.imprintBuilder);
        }
      }
    }

    function renderCompilerFailureImprints(time) {
      const active = time >= COMPILER_FAIL_IMPRINT_START
        && time < COMPILER_FAIL_IMPRINT_END;
      const genesisActive = time >= COMPILER_FAIL_IMPRINT_START
        && time < GENESIS_IMPRINT_CONSUME_END;
      const punchProgress = intervalProgress(
        time,
        COMPILER_FAIL_IMPRINT_START,
        COMPILER_FAIL_IMPRINT_START + COMPILER_FAIL_IMPRINT_PUNCH_SECONDS,
      );
      // Ease into the surface with residual velocity, then stop dead at 1.0.
      const punch = Math.pow(punchProgress, 1.7);
      const scale = lerp(1.15, 1, punch);
      const punchTransform = `translate(75 23) scale(${scale.toFixed(5)}) translate(-75 -23)`;
      setComposedTransform(hooks.imprintBuilderPunch, punchTransform);
      setComposedTransform(hooks.imprintWorkpiecePunch, punchTransform);
      setComposedTransform(hooks.imprintGenesisPunch, punchTransform);
      setOpacity(hooks.imprintBuilder, active ? 1 : 0);
      setOpacity(hooks.imprintWorkpiece, active ? 1 : 0);
      const consumeProgress = smootherstep(intervalProgress(
        time,
        GENESIS_IMPRINT_CONSUME_START,
        GENESIS_IMPRINT_CONSUME_END,
      ));
      const consumeFade = 1 - smoothstep(intervalProgress(
        time,
        GENESIS_IMPRINT_CONSUME_END - 0.34,
        GENESIS_IMPRINT_CONSUME_END,
      ));
      setOpacity(hooks.imprintGenesis, genesisActive ? consumeFade : 0);
      const consumeX = -127 * consumeProgress;
      const consumeY = -40 * consumeProgress - Math.sin(consumeProgress * Math.PI) * 18;
      const consumeScale = lerp(1, 0.06, consumeProgress);
      setComposedTransform(
        hooks.imprintGenesisConsume,
        `translate(${consumeX.toFixed(3)} ${consumeY.toFixed(3)}) translate(447 330) scale(${consumeScale.toFixed(4)}) translate(-447 -330)`,
      );
      if (hooks.imprintBuilder) hooks.imprintBuilder.dataset.filmImprinted = String(active);
      if (hooks.imprintWorkpiece) hooks.imprintWorkpiece.dataset.filmImprinted = String(active);
      if (hooks.imprintGenesis) {
        hooks.imprintGenesis.dataset.filmImprinted = String(genesisActive);
        hooks.imprintGenesis.dataset.filmConsumed = String(consumeProgress >= 1);
      }

      const pressing = active && punchProgress > 0 && punchProgress < 1;
      const deckPress = pressing
        ? Math.sin(punchProgress * Math.PI) * COMPILER_FAIL_DECK_PRESS_PX
        : 0;
      const deckTransform = `translate(0 ${deckPress.toFixed(3)})`;
      setComposedTransform(hooks.builderSurface, deckTransform);
      setComposedTransform(hooks.genesisSurface, deckTransform);
      const workpiecePress = pressing ? Math.sin(punchProgress * Math.PI) : 0;
      setComposedTransform(
        hooks.workpieceBody,
        `translate(0 ${(workpiecePress * 4).toFixed(3)}) scale(${(1 + workpiecePress * 0.08).toFixed(4)} ${(1 - workpiecePress * 0.16).toFixed(4)})`,
      );
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
      placeOnPath(hooks.materialMain, hooks.materialPath, 34.0, 37.3);
      placeOnPath(hooks.materialCargo, hooks.materialPath, 34.8, 38.2);
      (hooks.editOneFiles || []).forEach((node, index) => {
        const start = 49.95 + index * 0.4;
        placeOnPath(node, hooks.materialPath, start, start + 1.8);
      });
      (hooks.editTwoFiles || []).forEach((node, index) => {
        const start = 64.4 + index * 0.4;
        placeOnPath(node, hooks.materialPath, start, start + 1.75);
      });
      const route = [
        { at: 34.0, x: 900, y: 345, scale: 0.54 },
        { at: 37.2, x: 900, y: 345, scale: 1 },
        { at: 41.0, x: 900, y: 345, scale: 1 },
        { at: 41.85, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 46.2, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 49.4, x: 900, y: 345, scale: 1 },
        { at: 52.55, x: 900, y: 345, scale: 1 },
        { at: 53.05, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 56.05, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 57.8, x: VERIFIER_WORKPIECE_X, y: VERIFIER_WORKPIECE_Y, scale: 1 },
        { at: 60.8, x: VERIFIER_WORKPIECE_X, y: VERIFIER_WORKPIECE_Y, scale: 1 },
        { at: 63.85, x: 900, y: 345, scale: 1 },
        { at: 66.95, x: 900, y: 345, scale: 1 },
        { at: 67.3, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 70.0, x: COMPILER_WORKPIECE_X, y: COMPILER_WORKPIECE_Y, scale: 1 },
        { at: 71.2, x: VERIFIER_WORKPIECE_X, y: VERIFIER_WORKPIECE_Y, scale: 1 },
        { at: 77.7, x: VERIFIER_WORKPIECE_X, y: VERIFIER_WORKPIECE_Y, scale: 1 },
        { at: 79.8, x: GUARD_QUEUE_X, y: GUARD_QUEUE_Y, scale: 1 },
        { at: 88.0, x: GUARD_QUEUE_X, y: GUARD_QUEUE_Y, scale: 1 },
        { at: 89.6, x: LIVE_DOOR_X, y: LIVE_DOOR_Y, scale: 1 },
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
      const workpieceAlpha = windowState(time, { start: 34, end: 120, fade: 0.5 }).alpha;
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
          ? "PLAYER.WASM"
          : time >= 84.55
            ? "PLAYER.WASM · AUTHORIZED"
            : time >= 77.7
              ? "PLAYER.WASM · 3 PROOFS"
              : time >= 71.2
                ? "PLAYER.WASM"
                : time >= 66.95
                  ? "FIX 02"
                  : time >= 63.85
                    ? "APPLYING EDIT 02"
                    : time >= 60.8
                      ? "FAILED · HARNESS"
                      : time >= 55.4
                        ? "PLAYER.WASM"
                      : time >= 52.55
                        ? "FIX 01"
                        : time >= 49.4
                          ? "APPLYING EDIT 01"
                          : "PLAYER.RS";
        setText(hooks.workpieceLabel, label);
      }

      const firstCompile = smoothstep(intervalProgress(time, 42.25, 46.0));
      const secondCompile = smoothstep(intervalProgress(time, 53.25, 55.4));
      const thirdCompile = smoothstep(intervalProgress(time, 67.5, 69.35));
      const compilerRound = time < 52.55 ? 1 : time < 66.95 ? 2 : 3;
      const compilerProgress = compilerRound === 1
        ? firstCompile
        : compilerRound === 2
          ? secondCompile
          : thirdCompile;
      const compilerResult = time < 42.25
        ? "waiting"
        : time < 46.0
          ? "building"
          : time < 52.55
            ? "failed"
            : time < 53.25
              ? "waiting"
              : time < 55.4
                ? "building"
              : time < 66.95
                ? "passed"
                : time < 67.5
                  ? "waiting"
                  : time < 69.35
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
          time < 42.25
            ? "READY · ROUND 0/3"
            : time < 46.0
              ? `COMPILING · ROUND 1/3 · ${Math.round(firstCompile * 100)}%`
              : time < 52.55
                ? "FAILED · ROUND 1/3 · DIAG 01"
                : time < 53.25
                  ? "READY · ROUND 2/3"
                  : time < 55.4
                    ? `COMPILING · ROUND 2/3 · ${Math.round(secondCompile * 100)}%`
                  : time < 66.95
                    ? "PASSED · ROUND 2/3"
                    : time < 67.5
                      ? "READY · ROUND 3/3"
                      : time < 69.35
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

      const verifierScanWindow = time >= 57.8 && time < 60.8
        ? [57.8, 60.8]
        : time >= 71.2 && time < 75.6
          ? [71.2, 75.6]
          : null;
      const verifierScanProgress = verifierScanWindow
        ? intervalProgress(time, verifierScanWindow[0], verifierScanWindow[1])
        : 0;
      const verifierScanIntro = verifierScanWindow
        ? smootherstep(intervalProgress(time, verifierScanWindow[0], verifierScanWindow[0] + 0.55))
        : 0;
      const verifierScanOutro = verifierScanWindow
        ? 1 - smootherstep(intervalProgress(time, verifierScanWindow[1] - 0.42, verifierScanWindow[1]))
        : 0;
      const verifierScanAlpha = verifierScanIntro * verifierScanOutro;
      if (hooks.verifier) {
        hooks.verifier.dataset.filmScanning = String(Boolean(verifierScanWindow));
      }

      const shadowStart = verifierScanWindow ? verifierScanWindow[0] : 0;
      const shadowEnd = verifierScanWindow ? verifierScanWindow[1] : 0;
      const shadowRise = lerp(16, 0, verifierScanIntro);
      const shadowScale = lerp(0.93, 1, verifierScanIntro);
      setOpacity(hooks.shadowWorld, verifierScanAlpha * 0.6);
      setComposedTransform(
        hooks.shadowWorld,
        `translate(0 ${shadowRise.toFixed(3)}) translate(607.5 200) scale(${shadowScale.toFixed(4)}) translate(-607.5 -200)`,
      );
      if (hooks.shadowWorld) {
        hooks.shadowWorld.dataset.filmShadowState = !verifierScanWindow
          ? "dormant"
          : verifierScanProgress < 0.18
            ? "spawning"
            : verifierScanProgress > 0.86
              ? "discarding"
              : "testing";
      }

      const shadowDoorIntro = verifierScanWindow
        ? smootherstep(intervalProgress(time, shadowStart + 0.22, shadowStart + 0.72))
        : 0;
      const shadowDoorAlpha = shadowDoorIntro * verifierScanOutro;
      const shadowDoorPunch = Math.sin(shadowDoorIntro * Math.PI) * (1 - shadowDoorIntro);
      setOpacity(hooks.shadowEntryDoor, shadowDoorAlpha);
      setComposedTransform(
        hooks.shadowEntryDoor,
        `translate(0 ${(lerp(18, 0, shadowDoorIntro)).toFixed(3)}) scale(${(lerp(0.68, 1, shadowDoorIntro) + shadowDoorPunch * 0.1).toFixed(4)})`,
      );

      const shadowEntryProgress = verifierScanWindow
        ? smootherstep(intervalProgress(time, shadowStart + 0.48, Math.min(shadowStart + 1.58, shadowEnd - 0.72)))
        : 0;
      const shadowPlayerIntro = verifierScanWindow
        ? smootherstep(intervalProgress(time, shadowStart + 0.4, shadowStart + 0.68))
        : 0;
      if (hooks.shadowPlayer && hooks.shadowEntryRoute && typeof hooks.shadowEntryRoute.getTotalLength === "function") {
        const shadowRouteLength = hooks.shadowEntryRoute.getTotalLength();
        const shadowPoint = hooks.shadowEntryRoute.getPointAtLength(shadowRouteLength * shadowEntryProgress);
        const doorCrossing = Math.exp(-Math.pow((shadowEntryProgress - 0.64) / 0.12, 2));
        const shadowPlayerScale = lerp(0.82, 0.72, shadowEntryProgress) * (1 - doorCrossing * 0.34);
        const shadowHover = shadowEntryProgress >= 1
          ? Math.sin((time - shadowStart) * Math.PI * 2.1) * 1.7
          : 0;
        setComposedTransform(
          hooks.shadowPlayer,
          `translate(${shadowPoint.x.toFixed(3)} ${(shadowPoint.y + shadowHover).toFixed(3)}) scale(${shadowPlayerScale.toFixed(4)})`,
        );
        setOpacity(hooks.shadowPlayer, shadowPlayerIntro * verifierScanOutro);
      } else {
        setOpacity(hooks.shadowPlayer, 0);
      }
      if (hooks.shadowEntryRoute) {
        hooks.shadowEntryRoute.style.strokeDashoffset = (-verifierScanProgress * 44).toFixed(3);
        setOpacity(hooks.shadowEntryRoute, shadowDoorAlpha * 0.82);
      }

      const attackIntro = verifierScanWindow
        ? smootherstep(intervalProgress(time, shadowStart + 1.28, Math.min(shadowStart + 1.72, shadowEnd - 0.48)))
        : 0;
      const attackAlpha = attackIntro * verifierScanOutro;
      setOpacity(hooks.shadowAttacks, attackAlpha);
      (hooks.shadowAttackPaths || []).forEach((node, index) => {
        node.style.strokeDashoffset = (-(time - shadowStart) * (34 + index * 7)).toFixed(3);
      });
      (hooks.shadowImpacts || []).forEach((node, index) => {
        const impactPulse = 0.45 + 0.55 * Math.max(0, Math.sin((time - shadowStart) * 8.6 - index * 1.75));
        const impactScale = 0.72 + impactPulse * 0.38;
        setComposedTransform(node, `scale(${impactScale.toFixed(4)})`);
        setOpacity(node, attackAlpha * impactPulse);
      });

      let verifierVerdict = "none";
      let verifierVerdictStart = 0;
      let verifierVerdictEnd = 0;
      let verifierVerdictRotation = 0;
      if (time >= 60.72 && time < 66.8) {
        verifierVerdict = "denied";
        verifierVerdictStart = 60.72;
        verifierVerdictEnd = 66.8;
        verifierVerdictRotation = 180;
      } else if (time >= 75.5 && time < 79.2) {
        verifierVerdict = "granted";
        verifierVerdictStart = 75.5;
        verifierVerdictEnd = 79.2;
        verifierVerdictRotation = 0;
      }
      if (hooks.verifierThumb) {
        const verdictIn = verifierVerdict === "none"
          ? 0
          : smootherstep(intervalProgress(time, verifierVerdictStart, verifierVerdictStart + 0.68));
        const verdictOut = verifierVerdict === "none"
          ? 0
          : 1 - smoothstep(intervalProgress(time, verifierVerdictEnd - 0.35, verifierVerdictEnd));
        const verdictAlpha = verdictIn * verdictOut;
        const verdictPunch = Math.sin(verdictIn * Math.PI) * (1 - verdictIn);
        const verdictRotation = lerp(-92, verifierVerdictRotation, verdictIn)
          + (verifierVerdict === "denied" ? -18 : 14) * verdictPunch;
        const verdictScale = lerp(0.42, 1, verdictIn) * (1 + 0.16 * verdictPunch);
        setComposedTransform(
          hooks.verifierThumb,
          `translate(1134 307) rotate(${verdictRotation.toFixed(3)}) scale(${verdictScale.toFixed(4)})`,
        );
        setOpacity(hooks.verifierThumb, verdictAlpha);
        hooks.verifierThumb.dataset.filmVerdict = verifierVerdict;
      }

      const twinAttached = time >= 74.0;
      const drillsAttached = time >= 75.6;
      const reportAttached = time >= 77.7;
      const approvalReady = time >= 81.7;
      setOpacity(hooks.sealTwin, twinAttached ? smoothstep(intervalProgress(time, 74.0, 74.5)) : 0);
      setOpacity(hooks.sealDrills, drillsAttached ? smoothstep(intervalProgress(time, 75.6, 76.1)) : 0);
      setOpacity(hooks.sealReport, reportAttached ? smoothstep(intervalProgress(time, 78.0, 78.5)) : 0);
      setOpacity(
        hooks.testReport,
        reportAttached
          ? windowState(time, { start: 77.55, end: 85.15, fade: 0.3 }).alpha
          : 0,
      );
      if (hooks.sealReport) {
        hooks.sealReport.dataset.filmPending = "false";
        hooks.sealReport.dataset.filmAttached = String(reportAttached);
      }
      setText(hooks.sealReportText, reportAttached ? "R" : "?");

      const drillTimes = [
        [hooks.drillWall, 72.8],
        [hooks.drillImport, 74.2],
        [hooks.drillFuel, 75.6],
      ];
      drillTimes.forEach(([node, at]) => {
        if (node) node.dataset.filmPassed = String(time >= at);
      });

      const seals = Number(twinAttached) + Number(drillsAttached) + Number(reportAttached);
      const guardReportAccepted = reportAttached && time >= 79.8;
      const guardHashMatched = time >= 80.7;
      const guardCapsAllowed = time >= 81.55;
      const ownerAuthorized = time >= 84.55;
      const guardAuthorized = guardReportAccepted
        && guardHashMatched
        && guardCapsAllowed
        && ownerAuthorized;
      const unlocked = time >= 85.25 && seals === 3 && guardAuthorized;
      [
        [hooks.guardReport, guardReportAccepted],
        [hooks.guardHash, guardHashMatched],
        [hooks.guardCaps, guardCapsAllowed],
        [hooks.guardOwner, ownerAuthorized],
      ].forEach(([node, passed]) => {
        if (node) node.dataset.filmPassed = String(passed);
      });
      if (hooks.guard) {
        hooks.guard.dataset.filmAuthorized = String(guardAuthorized);
        hooks.guard.dataset.filmGuardState = unlocked
          ? "open"
          : approvalReady
            ? "owner"
            : guardReportAccepted
              ? "verifying"
              : "waiting";
      }
      const guardStepProgress = smootherstep(intervalProgress(
        time,
        GUARD_STEP_START,
        GUARD_STEP_END,
      ));
      if (hooks.guardChecks) {
        const checksAlpha = time >= 42.9
          ? 1 - smoothstep(intervalProgress(time, GUARD_STEP_START, GUARD_STEP_END))
          : 0;
        setOpacity(hooks.guardChecks, checksAlpha);
      }
      if (hooks.liveGrantLine) {
        const grantProgress = smootherstep(intervalProgress(
          time,
          GUARD_STEP_START,
          GUARD_STEP_END,
        ));
        const revokeProgress = smootherstep(intervalProgress(
          time,
          LIVE_GRANT_REVOKE_START,
          LIVE_GRANT_REVOKE_END,
        ));
        const visibleGrant = grantProgress * (1 - revokeProgress);
        const guardAnchorX = GUARD_GRANT_X + GUARD_SHIFT_X * guardStepProgress;
        const guardAnchorY = GUARD_GRANT_Y + GUARD_SHIFT_Y * guardStepProgress;
        const controlX = lerp(guardAnchorX, LIVE_DOOR_X, 0.56);
        const controlY = Math.min(guardAnchorY, LIVE_DOOR_Y) - 18;
        hooks.liveGrantLine.setAttribute(
          "d",
          `M${guardAnchorX.toFixed(2)} ${guardAnchorY.toFixed(2)}`
            + `Q${controlX.toFixed(2)} ${controlY.toFixed(2)} ${LIVE_DOOR_X} ${LIVE_DOOR_Y}`,
        );
        hooks.liveGrantLine.style.strokeDasharray = `${(visibleGrant * 100).toFixed(3)} 100`;
        hooks.liveGrantLine.style.strokeDashoffset = "0";
        hooks.liveGrantLine.dataset.filmGrantState = time < GUARD_STEP_START
          ? "idle"
          : time < GUARD_STEP_END
            ? "granting"
            : time < LIVE_GRANT_REVOKE_START
              ? "active"
              : time < LIVE_GRANT_REVOKE_END
                ? "revoking"
                : "revoked";
        setOpacity(
          hooks.liveGrantLine,
          time >= GUARD_STEP_START && time < LIVE_GRANT_REVOKE_END ? 1 : 0,
        );
        if (hooks.liveGrantPulse && typeof hooks.liveGrantLine.getTotalLength === "function") {
          const pulseProgress = smootherstep(intervalProgress(
            time,
            GUARD_STEP_START,
            GUARD_STEP_END,
          ));
          const pulseLength = hooks.liveGrantLine.getTotalLength();
          const pulsePoint = hooks.liveGrantLine.getPointAtLength(pulseLength * pulseProgress);
          hooks.liveGrantPulse.setAttribute("cx", pulsePoint.x.toFixed(2));
          hooks.liveGrantPulse.setAttribute("cy", pulsePoint.y.toFixed(2));
          const pulseActive = time >= GUARD_STEP_START && time < GUARD_STEP_END;
          setOpacity(
            hooks.liveGrantPulse,
            pulseActive ? Math.sin(pulseProgress * Math.PI) : 0,
          );
        }
      }
      if (hooks.liveDataPath && typeof hooks.liveDataPath.getTotalLength === "function") {
        const dataLength = hooks.liveDataPath.getTotalLength();
        (hooks.liveDataDots || []).forEach((dot, index) => {
          const start = 88.0 + index * 0.16;
          const end = 89.28 + index * 0.16;
          const dataProgress = smootherstep(intervalProgress(time, start, end));
          const dataPoint = hooks.liveDataPath.getPointAtLength(dataLength * dataProgress);
          dot.setAttribute("cx", dataPoint.x.toFixed(2));
          dot.setAttribute("cy", dataPoint.y.toFixed(2));
          setOpacity(dot, windowState(time, { start, end, fade: 0.18 }).alpha);
        });
      }
      if (hooks.outGateStatus) hooks.outGateStatus.dataset.filmUnlocked = String(unlocked);
      if (hooks.outDoor) hooks.outDoor.dataset.filmUnlocked = String(unlocked);
      if (hooks.outGateCopy) {
        const gateCopy = unlocked
          ? "GUARD · LIVE DOOR OPEN"
          : !guardReportAccepted
            ? "GUARD · WAITING FOR TEST REPORT"
            : !guardHashMatched
              ? "REPORT OK · MATCHING EXACT HASH"
              : !guardCapsAllowed
                ? "HASH OK · CHECKING EXACT RIGHTS"
                : !ownerAuthorized
                  ? "RIGHTS OK · OWNER CONSENT"
                  : "ALL BINDINGS MATCH · OPENING";
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
      pulseNodes.forEach((node) => {
        const action = parseAction(node);
        if (action.kind === "trace" && time < action.end) return;
        const speed = Number.parseFloat(node.dataset.filmSpeed || "22") || 22;
        const direction = node.dataset.filmDirection === "reverse" ? -1 : 1;
        const offset = -time * speed * direction;
        node.style.strokeDashoffset = offset.toFixed(2);
        node.style.setProperty("--film-dash-offset", offset.toFixed(2));
      });
      if (hooks.netVein) {
        hooks.netVein.dataset.filmConnected = String(time >= 22.37);
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
      const orderStart = 92.15;
      const orderEnd = 92.95;
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
        const isGuard = node === hooks.guard;
        const guardStep = isGuard
          ? smootherstep(intervalProgress(time, GUARD_STEP_START, GUARD_STEP_END))
          : 0;
        // The Guard's base group is scaled to 0.5, so use local-space values
        // twice as large to produce a clear one-body-width sidestep on deck.
        const guardDx = GUARD_SHIFT_X * 2 * guardStep;
        const guardDy = GUARD_SHIFT_Y * 2 * guardStep;
        if (time < start) {
          setComposedTransform(
            node,
            isGuard
              ? `translate(${guardDx.toFixed(2)} ${guardDy.toFixed(2)})`
              : "",
          );
          if (managedBefore) setOpacity(node, 1);
          node.dataset.filmTeardownState = isGuard && guardStep > 0
            ? "stepping-aside"
            : "standing";
          return;
        }
        const progress = smoothstep(intervalProgress(time, start, end));
        setComposedTransform(
          node,
          `translate(${(guardDx + dx * progress).toFixed(2)} ${(guardDy + dy * progress).toFixed(2)})`,
        );
        setOpacity(node, 1 - progress);
        node.dataset.filmTeardownState = progress >= 1 ? "released" : "releasing";
      });

      if (hooks.builderDeck) hooks.builderDeck.dataset.filmReleased = String(time >= 95.8);
      if (hooks.playerDomain) hooks.playerDomain.dataset.filmAttached = String(time >= 92.0);
    }

    function renderRingAndApproval(time) {
      const ringStates = [
        [hooks.ringManifest, 79.8],
        [hooks.ringHash, 80.7],
        [hooks.ringReport, 81.55],
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

      const pointerProgress = smoothstep(intervalProgress(time, 82.15, 84.25));
      if (hooks.approvalPointer) {
        const clickDown = smoothstep(intervalProgress(time, 84.12, 84.25))
          * (1 - smoothstep(intervalProgress(time, 84.25, 84.5)));
        hooks.approvalPointer.style.setProperty("--film-pointer-progress", pointerProgress.toFixed(4));
        hooks.approvalPointer.dataset.filmClicked = String(time >= 84.25);
        setComposedTransform(
          hooks.approvalPointer,
          `translate(${(58 * (1 - pointerProgress)).toFixed(2)} ${(-54 * (1 - pointerProgress) + 5 * clickDown).toFixed(2)})`
            + ` scale(${(1 - 0.045 * clickDown).toFixed(4)})`,
        );
        setOpacity(hooks.approvalPointer, windowState(time, { start: 82.0, end: 85.4, fade: 0.3 }).alpha);
      }
      if (hooks.approvalButton) {
        const pressed = time >= 84.15 && time < 84.62;
        hooks.approvalButton.dataset.filmPressed = String(pressed);
        hooks.approvalButton.dataset.filmApproved = String(time >= 84.55);
      }
      if (hooks.approveOwnerBinding) {
        hooks.approveOwnerBinding.dataset.filmPassed = String(time >= 84.55);
      }
      setText(
        hooks.approvalButtonCopy,
        time >= 84.55 ? "LIVE EXECUTION GRANTED" : "GRANT LIVE EXECUTION",
      );
      setText(
        hooks.approvalButtonSubcopy,
        time >= 84.55
          ? "BOUND TO PLAYER.WASM + THIS EXACT RIGHT SET"
          : "PLAYER.WASM · EXACT HASH · 3 CAPABILITIES",
      );
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

    // ================= SECTION: HUD & voice captions =================
    function formatTimecode(time) {
      const seconds = Math.floor(clamp(time, 0, DURATION));
      const minutes = Math.floor(seconds / 60);
      const remainder = seconds % 60;
      return `${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")} / 02:00`;
    }

    function renderVoiceCaption(time) {
      const cue = FILM_AUDIO_CUE_DEFINITIONS.find((candidate) => (
        time >= candidate.start && time < candidate.end
      ));
      const localTime = cue ? time - cue.start : 0;
      const caption = cue
        ? (FILM_AUDIO_CAPTIONS[cue.id] || []).find(([start, end]) => (
          localTime >= start && localTime < end
        ))
        : null;
      const copy = caption ? caption[2] : "";
      setText(hooks.voiceCaptionText, copy);
      if (hooks.voiceCaption) {
        const cueAlpha = cue && copy
          ? smoothstep(intervalProgress(time, cue.start, cue.start + 0.18))
            * (1 - smoothstep(intervalProgress(time, cue.end - 0.18, cue.end)))
          : 0;
        setOpacity(hooks.voiceCaption, cueAlpha);
      }
    }

    function renderHud(time) {
      setText(hooks.timecode, formatTimecode(time));
      if (hooks.timelineProgress) {
        const length = 1132 * clamp(time / DURATION, 0, 1);
        hooks.timelineProgress.setAttribute("d", `M34 871h${length.toFixed(2)}`);
        hooks.timelineProgress.style.setProperty("--film-total-progress", (time / DURATION).toFixed(6));
      }
    }

    // ================= SECTION: Render & transport =================
    function updateScrubber(time, activeScene) {
      const activeWaypoint = waypointAt(time);
      if (scrubber) scrubber.classList.toggle("is-introduced", time >= TIMELINE_REVEAL_TIME);
      if (scrubRange && document.activeElement !== scrubRange) scrubRange.value = time.toFixed(2);
      if (scrubOutput) {
        scrubOutput.textContent = `${formatTimecode(time).slice(0, 5)} · ${activeWaypoint ? activeWaypoint.title : activeScene.title}`;
      }
      if (scrubSceneTitle) {
        scrubSceneTitle.textContent = !activeScene
          ? ""
          : activeWaypoint && activeWaypoint.title !== activeScene.title
            ? activeWaypoint.title
            : activeScene.title;
      }
      if (scrubPlay) {
        scrubPlay.textContent = playing ? "Pause" : "Play";
        scrubPlay.setAttribute("aria-pressed", String(playing));
      }
      scrubMarks.forEach((button, index) => {
        const waypoint = TIMELINE_WAYPOINTS[index];
        const active = Boolean(activeWaypoint && waypoint.number === activeWaypoint.number);
        button.classList.toggle("is-active", active);
        button.classList.toggle("is-past", time >= waypoint.at);
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
      renderGenesisKeys(currentTime);
      renderDoorCeremonies(currentTime);
      renderBuilderArchitecture(currentTime);
      renderPrompt(currentTime);
      renderBackgroundDim(currentTime);
      renderNetNode(currentTime);
      renderVoiceCaption(currentTime);
      renderLayerCallouts(currentTime);
      renderCompiler(currentTime);
      renderCompilerFailureImprints(currentTime);
      renderFeedback(currentTime);
      renderTwins(currentTime);
      renderWorkshop(currentTime);
      renderBuilderShutdown(currentTime);
      renderPersistentVeins(currentTime);
      renderRingAndApproval(currentTime);
      renderFinale(currentTime);
      renderSceneDetails(currentTime);
      renderHud(currentTime);
      updateScrubber(currentTime, activeScene);
      updateCameraEditor(currentTime);
      return currentTime;
    }

    function frame(now) {
      if (!playing) return;
      const elapsed = (now - anchorNow) / 1000;
      currentTime = (anchorTime + elapsed) % DURATION;
      render(currentTime);
      syncFilmScrollPosition();
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

    // ================= SECTION: Camera keyframe editor =================
    function renderCameraEditorMarkers() {
      if (!cameraEditorMarkers) return;
      cameraEditorMarkers.replaceChildren();
      cameraKeyframes.forEach((frame, index) => {
        const marker = document.createElement("button");
        marker.type = "button";
        marker.className = "film-camera-editor__marker";
        marker.style.setProperty("--film-camera-key-position", `${(frame.at / DURATION) * 100}%`);
        marker.title = `${frame.at.toFixed(2)}s · x ${frame.focusX.toFixed(2)} · y ${frame.focusY.toFixed(2)} · dist ${(1 / frame.scale).toFixed(2)}`;
        marker.setAttribute("aria-label", `Camera keyframe ${index + 1} at ${frame.at.toFixed(2)} seconds`);
        marker.addEventListener("click", () => {
          cameraPreview = null;
          stopClock(false);
          setTime(frame.at);
        });
        cameraEditorMarkers.appendChild(marker);
      });
    }

    function updateCameraEditor(time) {
      if (!cameraEditor) return;
      if (cameraPreview && Math.abs(cameraPreview.at - time) >= 0.05) cameraPreview = null;
      const camera = cameraPreview || cameraAt(time);
      if (cameraEditorTime && document.activeElement !== cameraEditorTime) {
        cameraEditorTime.value = time.toFixed(2);
      }
      const values = {
        x: camera.focusX,
        y: camera.focusY,
        distance: 1 / camera.scale,
      };
      Object.entries(values).forEach(([name, value]) => {
        const input = cameraEditorInputs[name];
        if (input && document.activeElement !== input) input.value = value.toFixed(3);
      });
      if (cameraEditorStatus) {
        cameraEditorStatus.textContent = `${time.toFixed(2)}s · ${cameraKeyframes.length} keys${cameraPreview ? " · preview" : ""}`;
      }
      if (cameraEditorMarkers) {
        Array.from(cameraEditorMarkers.children).forEach((marker, index) => {
          marker.classList.toggle("is-active", Math.abs(cameraKeyframes[index].at - time) < 0.08);
        });
      }
    }

    function createCameraEditor() {
      const explicitlyRequested = params.get("camedit") === "1";
      const localDefault = location.protocol === "file:"
        && params.get("camedit") !== "0";
      if (!explicitlyRequested && !localDefault) return;
      const host = root.closest(".film-frame") || root.parentElement;
      if (!host) return;

      const editor = document.createElement("section");
      editor.className = "film-camera-editor";
      editor.setAttribute("aria-label", "Camera keyframe editor");
      editor.innerHTML = `
        <div class="film-camera-editor__head">
          <strong>CAMERA KEYFRAMES</strong>
          <output data-camera-status>0.00s</output>
        </div>
        <div class="film-camera-editor__timeline">
          <input data-camera-time type="range" min="0" max="${DURATION}" step="0.01" value="${currentTime}" aria-label="Film time">
          <div class="film-camera-editor__markers" aria-hidden="true"></div>
        </div>
        <div class="film-camera-editor__values">
          <label>X <input data-camera-value="x" type="number" min="0" max="1" step="0.01"></label>
          <label>Y <input data-camera-value="y" type="number" min="0" max="1" step="0.01"></label>
          <label>DIST <input data-camera-value="distance" type="number" min="0.42" max="2.2" step="0.01"></label>
          <button type="button" data-camera-action="set">SET</button>
          <button type="button" data-camera-action="delete">DELETE</button>
          <button type="button" data-camera-action="delete-all">DELETE ALL</button>
          <button type="button" data-camera-action="reset">RESET</button>
          <button type="button" data-camera-action="copy">COPY</button>
          <button type="button" data-camera-action="download">DOWNLOAD</button>
        </div>`;
      document.body.appendChild(editor);
      cameraEditor = editor;
      editor.addEventListener("wheel", (event) => event.stopPropagation(), { passive: true });
      const toggleEditor = document.createElement("button");
      toggleEditor.type = "button";
      toggleEditor.className = "film-camera-editor__toggle";
      toggleEditor.textContent = "KEYFRAMES";
      toggleEditor.setAttribute("aria-expanded", "false");
      toggleEditor.setAttribute("aria-label", "Toggle camera keyframe editor");
      toggleEditor.addEventListener("wheel", (event) => event.stopPropagation(), { passive: true });
      toggleEditor.addEventListener("click", () => {
        const open = editor.classList.toggle("is-open");
        toggleEditor.setAttribute("aria-expanded", String(open));
      });
      document.body.appendChild(toggleEditor);
      cameraEditorTime = editor.querySelector("[data-camera-time]");
      cameraEditorMarkers = editor.querySelector(".film-camera-editor__markers");
      cameraEditorStatus = editor.querySelector("[data-camera-status]");
      editor.querySelectorAll("[data-camera-value]").forEach((input) => {
        cameraEditorInputs[input.dataset.cameraValue] = input;
      });

      cameraEditorTime.addEventListener("input", () => {
        cameraPreview = null;
        stopClock(false);
        setTime(cameraEditorTime.value);
      });
      const previewCamera = () => {
        stopClock(false);
        const base = cameraAt(currentTime);
        const focusX = clamp(Number.parseFloat(cameraEditorInputs.x.value), 0, 1);
        const focusY = clamp(Number.parseFloat(cameraEditorInputs.y.value), 0, 1);
        const distance = clamp(Number.parseFloat(cameraEditorInputs.distance.value), 0.42, 2.2);
        cameraPreview = {
          at: currentTime,
          focusX: Number.isFinite(focusX) ? focusX : base.focusX,
          focusY: Number.isFinite(focusY) ? focusY : base.focusY,
          scale: Number.isFinite(distance) && distance > 0 ? 1 / distance : base.scale,
        };
        renderCamera(currentTime);
        updateCameraEditor(currentTime);
      };
      Object.values(cameraEditorInputs).forEach((input) => input.addEventListener("input", previewCamera));

      editor.querySelector('[data-camera-action="set"]').addEventListener("click", () => {
        const frame = cameraPreview || { ...cameraAt(currentTime), at: currentTime };
        frame.at = currentTime;
        const existing = cameraKeyframes.findIndex((entry) => Math.abs(entry.at - currentTime) < 0.06);
        if (existing >= 0) cameraKeyframes.splice(existing, 1, { ...frame });
        else cameraKeyframes.push({ ...frame });
        cameraKeyframes.sort((a, b) => a.at - b.at);
        cameraPreview = null;
        saveCameraFrames();
        renderCameraEditorMarkers();
        render(currentTime);
      });
      editor.querySelector('[data-camera-action="delete"]').addEventListener("click", () => {
        if (cameraKeyframes.length === 0) return;
        const nearest = cameraKeyframes.reduce((best, frame, index) => (
          Math.abs(frame.at - currentTime) < Math.abs(cameraKeyframes[best].at - currentTime) ? index : best
        ), 0);
        cameraKeyframes.splice(nearest, 1);
        cameraPreview = null;
        saveCameraFrames();
        renderCameraEditorMarkers();
        render(currentTime);
      });
      editor.querySelector('[data-camera-action="delete-all"]').addEventListener("click", () => {
        cameraKeyframes.splice(0, cameraKeyframes.length);
        cameraPreview = null;
        saveCameraFrames();
        renderCameraEditorMarkers();
        render(currentTime);
      });
      editor.querySelector('[data-camera-action="reset"]').addEventListener("click", () => {
        replaceCameraFrames(CAMERA_KEYFRAMES);
        cameraPreview = null;
        try { localStorage.removeItem(CAMERA_STORAGE_KEY); } catch (_) { /* no-op */ }
        renderCameraEditorMarkers();
        render(currentTime);
      });
      editor.querySelector('[data-camera-action="copy"]').addEventListener("click", async (event) => {
        const json = JSON.stringify(cameraKeyframes, null, 2);
        try {
          await navigator.clipboard.writeText(json);
          event.currentTarget.textContent = "COPIED";
          window.setTimeout(() => { event.currentTarget.textContent = "COPY"; }, 900);
        } catch (_) {
          if (cameraEditorStatus) cameraEditorStatus.textContent = "clipboard unavailable";
        }
      });
      editor.querySelector('[data-camera-action="download"]').addEventListener("click", () => {
        const blob = new Blob([JSON.stringify(cameraKeyframes, null, 2)], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = "raios-camera-keyframes.json";
        document.body.appendChild(link);
        link.click();
        link.remove();
        window.setTimeout(() => URL.revokeObjectURL(url), 0);
      });

      renderCameraEditorMarkers();
      updateCameraEditor(currentTime);
    }

    // ================= SECTION: Scrubber/transport UI =================
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
      let scrollResumeTimer = 0;
      let autoPlaybackArmed = false;
      root.dataset.filmAutoPlayback = "false";

      function clearScrollResume() {
        if (scrollResumeTimer) window.clearTimeout(scrollResumeTimer);
        scrollResumeTimer = 0;
      }

      function cancelWheelScrub() {
        if (wheelScrubFrame) cancelAnimationFrame(wheelScrubFrame);
        wheelScrubFrame = 0;
        wheelLastFrame = 0;
        wheelTargetTime = currentTime;
        promptAudioMotionHeld = false;
        controls.classList.remove("is-wheel-scrubbing");
      }

      function scheduleScrollResume() {
        clearScrollResume();
        if (!autoPlaybackArmed) return;
        scrollResumeTimer = window.setTimeout(() => {
          scrollResumeTimer = 0;
          if (!autoPlaybackArmed || document.hidden || suspendedByMode) return;
          cancelWheelScrub();
          play();
        }, 500);
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
          promptAudioMotionHeld = false;
          controls.classList.remove("is-wheel-scrubbing");
          return;
        }
        const easing = 1 - Math.exp(-elapsed / WHEEL_SCRUB_EASING_MS);
        setTime(currentTime + distance * easing);
        syncPinnedScrollToFilm();
        wheelScrubFrame = requestAnimationFrame(animateWheelScrub);
      }

      TIMELINE_WAYPOINTS.forEach((waypoint, index) => {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = SCENES[index].id.slice(0, 3).toUpperCase();
        button.title = `${waypoint.at}s · ${waypoint.title}`;
        button.setAttribute("aria-label", `Waypoint ${waypoint.number}: ${waypoint.title}, ${waypoint.at} seconds`);
        button.dataset.filmJump = String(waypoint.at);
        button.style.setProperty("--film-mark-position", `${(waypoint.at / DURATION) * 100}%`);
        button.addEventListener("click", () => {
          cancelWheelScrub();
          stopClock(false);
          setTime(waypoint.at);
        });
        marks.appendChild(button);
      });
      track.appendChild(marks);
      controls.appendChild(track);

      const output = document.createElement("output");
      output.className = "film-scrubber__time";
      controls.appendChild(output);

      const sceneTitle = document.createElement("output");
      sceneTitle.id = "film-scene-title";
      sceneTitle.className = "film-scrubber__scene-title";
      controls.appendChild(sceneTitle);

      playButton.addEventListener("click", () => {
        cancelWheelScrub();
        clearScrollResume();
        if (playing) {
          autoPlaybackArmed = false;
          root.dataset.filmAutoPlayback = "false";
          pause();
        }
        else {
          autoPlaybackArmed = true;
          root.dataset.filmAutoPlayback = "true";
          unlockPromptAudio();
          if (currentTime < SCROLL_PAGE_TRAVEL_END_TIME && !scrollPinned) lockScrollPosition();
          play();
        }
      });
      const snapTime = (value, threshold = 1.8) => {
        const nearest = TIMELINE_WAYPOINTS.reduce((best, waypoint) => (
          Math.abs(waypoint.at - value) < Math.abs(best - value) ? waypoint.at : best
        ), TIMELINE_WAYPOINTS[0].at);
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

      // ================= SECTION: Scroll & wheel pinning =================
      const wheelSurface = root.closest(".film-frame") || root;
      let scrollPinned = false;
      let userPinCooldown = false;
      let pinnedScrollY = 0;
      let pinnedScrollStartY = 0;
      let pinnedScrollEndY = 0;
      let pinCorrectionFrame = 0;
      let viewportResizeTimer = 0;
      let viewportResizeFrame = 0;
      let viewportResizeSettling = false;
      const isMobileFilmView = () => window.innerWidth <= 880;
      const scrollSnapProfile = () => isMobileFilmView()
        ? {
          frameY: MOBILE_SCROLL_TRIGGER_FRAME_Y,
          frameYPerDistance: MOBILE_SCROLL_TRIGGER_FRAME_Y_PER_CAMERA_DISTANCE,
          frameYMax: MOBILE_SCROLL_TRIGGER_FRAME_Y_MAX,
          viewportY: MOBILE_SCROLL_TRIGGER_VIEWPORT_Y,
          tolerance: MOBILE_SCROLL_TRIGGER_TOLERANCE,
          pageTravelVh: MOBILE_SCROLL_PAGE_TRAVEL_VH,
        }
        : {
          frameY: SCROLL_TRIGGER_FRAME_Y,
          frameYPerDistance: SCROLL_TRIGGER_FRAME_Y_PER_CAMERA_DISTANCE,
          frameYMax: SCROLL_TRIGGER_FRAME_Y_MAX,
          viewportY: SCROLL_TRIGGER_VIEWPORT_Y,
          tolerance: SCROLL_TRIGGER_TOLERANCE,
          pageTravelVh: SCROLL_PAGE_TRAVEL_VH,
        };
      const scrollTriggerGeometry = (aspectDistance = responsiveCameraDistance) => {
        const bounds = wheelSurface.getBoundingClientRect();
        const profile = scrollSnapProfile();
        const responsiveFrameY = clamp(
          profile.frameY + aspectDistance * profile.frameYPerDistance,
          profile.frameY,
          profile.frameYMax,
        );
        const frameAnchor = bounds.top + bounds.height * responsiveFrameY;
        const viewportAnchor = window.innerHeight * profile.viewportY;
        return {
          distance: frameAnchor - viewportAnchor,
          tolerance: window.innerHeight * profile.tolerance,
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
        const profile = scrollSnapProfile();
        pinnedScrollStartY = window.scrollY;
        pinnedScrollEndY = Math.min(
          maximumScrollY,
          pinnedScrollStartY + window.innerHeight * profile.pageTravelVh,
        );
        pinnedScrollY = pinnedScrollStartY;
        scrollPinned = true;
        scrollTriggerIsActive();
      };
      const syncPinnedScrollToFilm = () => {
        if (!scrollPinned || viewportResizeSettling) return;
        const travelProgress = reducedMotion
          ? intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME)
          : smootherstep(intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME));
        pinnedScrollY = lerp(pinnedScrollStartY, pinnedScrollEndY, travelProgress);
        if (Math.abs(window.scrollY - pinnedScrollY) > 0.35) {
          window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
        }
      };
      syncFilmScrollPosition = syncPinnedScrollToFilm;
      const releaseScrollPin = () => {
        scrollPinned = false;
        userPinCooldown = isMobileFilmView();
        if (pinCorrectionFrame) cancelAnimationFrame(pinCorrectionFrame);
        pinCorrectionFrame = 0;
        scrollTriggerIsActive();
      };
      let previousTriggerDistance = scrollTriggerGeometry().distance;
      const updateScrollTrigger = () => {
        if (viewportResizeSettling) {
          scrollTriggerIsActive();
          return;
        }
        const geometry = scrollTriggerGeometry();
        const crossedEntry = previousTriggerDistance !== 0
          && Math.sign(previousTriggerDistance) !== Math.sign(geometry.distance);
        previousTriggerDistance = geometry.distance;
        if (!scrollPinned && isMobileFilmView()
          && Math.abs(geometry.distance) > geometry.tolerance) {
          userPinCooldown = false;
        }
        if (!scrollPinned && !controls.hidden
          && (!isMobileFilmView() || !userPinCooldown)
          && (crossedEntry || Math.abs(geometry.distance) <= geometry.tolerance)) {
          lockScrollPosition();
          return;
        }
        if (scrollPinned && Math.abs(window.scrollY - pinnedScrollY) > 1) {
          if (pinCorrectionFrame) {
            cancelAnimationFrame(pinCorrectionFrame);
            pinCorrectionFrame = 0;
          }
          if (isMobileFilmView()) {
            const travel = pinnedScrollEndY - pinnedScrollStartY;
            const withinTravel = travel > 1
              && window.scrollY >= pinnedScrollStartY - 1
              && window.scrollY <= pinnedScrollEndY + 1
              && currentTime <= SCROLL_PAGE_TRAVEL_END_TIME + 0.05;
            if (withinTravel) {
              // Mobile native swipe becomes film input during the opening move.
              pinnedScrollY = clamp(window.scrollY, pinnedScrollStartY, pinnedScrollEndY);
              const travelProgress = clamp(
                (pinnedScrollY - pinnedScrollStartY) / travel,
                0,
                1,
              );
              cancelWheelScrub();
              stopClock(true);
              setTime(travelProgress * SCROLL_PAGE_TRAVEL_END_TIME);
              scheduleScrollResume();
            } else {
              releaseScrollPin();
            }
          } else {
            // Desktop remains magnetically fixed at its authored snap point.
            pinCorrectionFrame = requestAnimationFrame(() => {
              pinCorrectionFrame = 0;
              if (scrollPinned) {
                window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
              }
            });
          }
        }
        scrollTriggerIsActive();
      };
      const scheduleViewportSettle = () => {
        if (viewportResizeTimer) window.clearTimeout(viewportResizeTimer);
        if (viewportResizeFrame) cancelAnimationFrame(viewportResizeFrame);
        viewportResizeFrame = 0;
        viewportResizeSettling = true;
        viewportResizeTimer = window.setTimeout(() => {
          viewportResizeTimer = 0;
          const distanceFrom = responsiveCameraDistance;
          const distanceTo = cameraAspectDistanceOffset();
          const verticalBiasFrom = responsiveCameraVerticalBias;
          const verticalBiasTo = cameraAspectVerticalBias();
          const scrollFrom = window.scrollY;
          let scrollTo = scrollFrom;
          let nextPinnedStartY = pinnedScrollStartY;
          let nextPinnedEndY = pinnedScrollEndY;

          if (scrollPinned) {
            const maximumScrollY = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
            const geometry = scrollTriggerGeometry(distanceTo);
            const profile = scrollSnapProfile();
            const travelProgress = reducedMotion
              ? intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME)
              : smootherstep(intervalProgress(currentTime, 0, SCROLL_PAGE_TRAVEL_END_TIME));
            const fullTravel = window.innerHeight * profile.pageTravelVh;
            const currentTravel = fullTravel * travelProgress;
            const targetAtCurrentTime = clamp(
              scrollFrom + geometry.distance + currentTravel,
              0,
              maximumScrollY,
            );
            nextPinnedStartY = clamp(targetAtCurrentTime - currentTravel, 0, maximumScrollY);
            nextPinnedEndY = Math.min(maximumScrollY, nextPinnedStartY + fullTravel);
            scrollTo = lerp(nextPinnedStartY, nextPinnedEndY, travelProgress);
          }

          const startedAt = performance.now();
          viewportResizeSettling = true;
          const settleFrame = (now) => {
            const elapsed = now - startedAt;
            const rawProgress = reducedMotion
              ? 1
              : clamp(elapsed / VIEWPORT_RESIZE_EASING_MS, 0, 1);
            const progress = smootherstep(rawProgress);
            responsiveCameraDistance = lerp(distanceFrom, distanceTo, progress);
            responsiveCameraVerticalBias = lerp(verticalBiasFrom, verticalBiasTo, progress);
            if (scrollPinned) {
              pinnedScrollY = lerp(scrollFrom, scrollTo, progress);
              if (Math.abs(window.scrollY - pinnedScrollY) > 0.35) {
                window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
              }
            }
            renderCamera(currentTime);
            if (rawProgress < 1) {
              viewportResizeFrame = requestAnimationFrame(settleFrame);
              return;
            }
            responsiveCameraDistance = distanceTo;
            responsiveCameraVerticalBias = verticalBiasTo;
            if (scrollPinned) {
              pinnedScrollStartY = nextPinnedStartY;
              pinnedScrollEndY = nextPinnedEndY;
              pinnedScrollY = scrollTo;
              window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
            }
            viewportResizeFrame = 0;
            viewportResizeSettling = false;
            previousTriggerDistance = scrollTriggerGeometry().distance;
            scrollTriggerIsActive();
          };
          viewportResizeFrame = requestAnimationFrame(settleFrame);
        }, VIEWPORT_RESIZE_SETTLE_DELAY_MS);
      };
      window.addEventListener("scroll", updateScrollTrigger, { passive: true });
      window.addEventListener("resize", scheduleViewportSettle, { passive: true });
      scrollTriggerIsActive();

      // Shared scrub step for wheel and touch input. Returns true when the
      // gesture drives the film (native scroll must be suppressed), false at
      // the film edges where the gesture falls through to normal page scroll.
      const scrubByDelta = (deltaPixels) => {
        if (!wheelScrubFrame) wheelTargetTime = currentTime;
        const deltaSeconds = clamp(
          deltaPixels * wheelSecondsPerPixel(wheelTargetTime),
          -0.9,
          0.9,
        );
        const wheelIntentNow = performance.now();
        const wheelIntentGap = promptAudioScrollIntentAt
          ? clamp(wheelIntentNow - promptAudioScrollIntentAt, 16, 180)
          : 180;
        const requestedScrollRate = clamp(
          0.55 + (Math.abs(deltaPixels) / wheelIntentGap) * 0.52,
          PROMPT_AUDIO_MIN_RATE,
          PROMPT_AUDIO_MAX_RATE,
        );
        const scrollRateBlend = requestedScrollRate > promptAudioScrollRate ? 0.58 : 0.3;
        promptAudioScrollRate = lerp(promptAudioScrollRate, requestedScrollRate, scrollRateBlend);
        promptAudioScrollIntentAt = wheelIntentNow;
        promptAudioDirection = deltaSeconds < 0 ? -1 : 1;
        promptAudioLastMotionAt = promptAudioScrollIntentAt;
        const atStart = wheelTargetTime <= 0.001 && deltaSeconds < 0;
        const atEnd = wheelTargetTime >= DURATION - 0.001 && deltaSeconds > 0;
        if (atStart || atEnd) {
          releaseScrollPin();
          return false;
        }

        unlockPromptAudio();
        autoPlaybackArmed = true;
        wantsPlayback = true;
        root.dataset.filmAutoPlayback = "true";
        if (!scrollPinned) lockScrollPosition();
        else if (Math.abs(window.scrollY - pinnedScrollY) > 1) {
          window.scrollTo({ top: pinnedScrollY, left: window.scrollX, behavior: "auto" });
        }
        stopClock(true);
        wheelTargetTime = clamp(
          wheelTargetTime + deltaSeconds,
          Math.max(0, currentTime - WHEEL_MAX_LEAD_SECONDS),
          Math.min(DURATION, currentTime + WHEEL_MAX_LEAD_SECONDS),
        );
        promptAudioMotionHeld = true;
        controls.classList.add("is-wheel-scrubbing");
        if (reducedMotion) {
          setTime(wheelTargetTime);
          syncPinnedScrollToFilm();
          controls.classList.remove("is-wheel-scrubbing");
        } else if (!wheelScrubFrame) {
          wheelScrubFrame = requestAnimationFrame(animateWheelScrub);
        }
        scheduleScrollResume();
        return true;
      };

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
        if (scrubByDelta(deltaPixels)) event.preventDefault();
      }, { passive: false });

      // Touch: a swipe over the pinned film scrubs the animation while native
      // page scroll stays suppressed (same contract as the wheel handler).
      const TOUCH_SCRUB_PIXEL_FACTOR = 0.6;
      let touchScrubY = 0;
      let touchScrubActive = false;
      wheelSurface.addEventListener("touchstart", (event) => {
        if (!isMobileFilmView() || controls.hidden
          || event.touches.length !== 1 || !scrollTriggerIsActive()) {
          touchScrubActive = false;
          return;
        }
        if (event.target.closest("button, input, a, .film-scrubber, .film-camera-editor")) {
          touchScrubActive = false;
          return;
        }
        touchScrubY = event.touches[0].clientY;
        touchScrubActive = true;
      }, { passive: true });
      wheelSurface.addEventListener("touchmove", (event) => {
        if (!touchScrubActive || event.touches.length !== 1) return;
        const touchY = event.touches[0].clientY;
        const deltaPixels = (touchScrubY - touchY) * TOUCH_SCRUB_PIXEL_FACTOR;
        touchScrubY = touchY;
        if (!deltaPixels) return;
        if (scrubByDelta(deltaPixels)) {
          event.preventDefault();
        } else {
          // Film edge reached: let the rest of the swipe scroll the page.
          touchScrubActive = false;
        }
      }, { passive: false });
      const endTouchScrub = () => { touchScrubActive = false; };
      wheelSurface.addEventListener("touchend", endTouchScrub, { passive: true });
      wheelSurface.addEventListener("touchcancel", endTouchScrub, { passive: true });

      root.insertAdjacentElement("afterend", controls);
      scrubber = controls;
      scrubRange = range;
      scrubOutput = output;
      scrubSceneTitle = sceneTitle;
      scrubPlay = playButton;
      scrubMarks = Array.from(marks.querySelectorAll("button"));
    }

    // ================= SECTION: Mode, visibility & boot =================
    refresh();
    const topGradient = document.getElementById("film-top-gradient");
    if (topGradient && topGradient.parentElement !== document.body) {
      document.body.appendChild(topGradient);
    }
    if (hooks.voiceCaption && hooks.voiceCaption.parentElement !== document.body) {
      document.body.appendChild(hooks.voiceCaption);
    }
    createScrubber();
    createCameraEditor();

    const api = {
      ready: true,
      duration: DURATION,
      posterTime: POSTER_TIME,
      scenes: SCENES,
      waypoints: TIMELINE_WAYPOINTS,
      cameraKeyframes,
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
      if (document.hidden) stopPromptAudio(promptAudioUnlocked ? "ready" : "locked");
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
