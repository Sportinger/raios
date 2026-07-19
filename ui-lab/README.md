# UI-Lab module map

The browser mirror is split along the same ownership boundaries as the current
kernel UI. `raios-ui-lab.html` is only the dependency-ordered entry point; it
must not accumulate renderer logic again.

| Browser module | Current Rust owner | Responsibility |
| --- | --- | --- |
| `core/font.js` | `seed-kernel/src/text.rs` | 8x8 bitmap font and glyph mapping |
| `core/model.js` | typed host snapshots/state | fixtures and lab state only |
| `core/primitives.js` | `framebuffer.rs`, `genesis.rs` | rect, outline, text, panel, button |
| `surfaces/genesis.js` | `shell_host/genesis.rs` | classic shell, conversation, context, composer |
| `surfaces/dream-geometry.js` | `shell_host/dream.rs::DreamGeometry` | responsive keyframes and interpolation |
| `surfaces/dream-background.js` | `shell_host/dream.rs` | cached background, stars, candle |
| `surfaces/dream.js` | `shell_host/dream.rs` | rails, tabs, chat, console, build center |
| `surfaces/recovery.js` | `shell_host/recovery.rs` | recovery projection and action rows |
| `surfaces/wifi-flow.js` | `shell_host/wifi_flow.rs` | every visually distinct WiFi state |
| `surfaces/vault-flow.js` | `shell_host/vault_flow.rs` | Vault modes and all outcome messages |
| `surfaces/personal-surface.js` | `personal_surface.rs`, `ui_program.rs`, checked-in guest | proof, calculator, editor display lists |
| `lab/scenarios.js` | lab-only | F2 catalog and deterministic state fixtures |
| `site/surface-config.js` | website-only | photo paths and measured display rectangle |
| `site/story.css` | website-only | story sections below the display: layout, scrim, SVG animation styles |
| `site/story.js` | website-only | scroll reveal plus `anim=0`/`animt`/`scroll`/`storydebug` params for deterministic shots |
| `site/film.js` | website-only | 120-second film clock, camera keyframes, deterministic frames, reduced-motion poster and development scrubber |
| `lab/website-mode.js` | website-only | UI/Website toggle, three-layer composite, `--story-overlap` measurement |
| `lab/diagnostics.js` | lab-only | ACTUAL/PROPOSAL diagnostics |
| `lab/app.js` | `shell_host/genesis.rs::ShellHost` | render order, input routing, selftests |

## Transfer rule

Model/state changes belong in `core/model.js`; geometry belongs in the matching
surface geometry module; painting belongs in the matching surface renderer;
browser-only navigation and fixtures belong under `lab/`. A later Rust port can
therefore move one category at a time without extracting behavior from one large
HTML script.

The scenario catalog covers every current host window/state class and every
Secret Vault outcome. Its data is simulated unless the root README explicitly
marks it as code-derived. The browser port is not yet a golden-pixel proof; a
shared no-dependency Rust/Wasm renderer remains the final way to eliminate
manual drift.

The Website mode intentionally keeps its photo and glass reflection outside the
renderer. See `assets/surface/README.md`: the live canvas remains the middle
layer, so switching F2 scenarios also updates the image shown inside the final
photographed Surface.

In Website mode the canvas backing store follows the measured display width at
16:9 instead of always shrinking a Full-HD frame. It is bounded to
1280x720..1920x1080; the 720p floor preserves Genesis' minimum safe logical
layout. UI-Lab mode remains canonical 1920x1080. Wheel input is consumed only
while the conversation can move in that direction, otherwise the page keeps
scrolling. During page scroll the drawn raiOS pointer is reprojected from the
stationary browser-pointer position.

## Architecture film controls

The first website story point is the 120-second “The Factory Moves In” SVG
film. It uses a JavaScript master clock instead of SMIL. `anim=0&animt=N`
renders one deterministic frame at second `N`. The normal website view shows a
fourteen-state timeline: drag to scrub, release to resume playback, or click a
numbered state to snap to it. `timeline=1` also exposes it in deterministic QA
shots. Reduced-motion visitors see the circle-wide poster at second 118. The
runtime is available to QA as `window.__RAIOS_FILM__`.
