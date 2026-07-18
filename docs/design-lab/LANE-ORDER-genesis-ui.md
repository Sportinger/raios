# Lane order: genesis-ui — immediate-mode UI harness for the Genesis layer

Owner-approved track (2026-07-18). Three sequential lanes; each has its own
gate. UI track is disjoint from wasm_runtime/WASI lanes by file set.
Spec source: `C:\Users\admin\Documents\raios2-ui-lab\raios-ui-lab.html`
(browser prototype; README there documents renderer laws and fidelity
classes; Ctrl+Shift+E in the lab exports the design delta as JSON).

## Why (one paragraph for the ADR)

The shell needs a real widget system (buttons, tabs, lists, progress,
hover/press states, layout keyframes) instead of ad-hoc draw calls. The
established pattern for a no_std single-threaded framebuffer kernel is an
immediate-mode GUI (egui/Dear-ImGui family): UI = pure function of state per
frame, widget identity via ids, hit list rebuilt per frame, bounded
tick-driven animations. Foreign frameworks (Slint, egui, LVGL) are the wrong
trust fit for the Genesis anchor — we take the API shape, not the code.

## Lane 1 — carve the crate, prove byte-identity (M/L, conservative)

Goal: new crate `crates/raios-genesis-ui` (no_std, ZERO deps) that renders
the CURRENT genesis shell 1:1 through a widget layer.

- Core: `Frame` context over a plain pixel-buffer surface trait (the kernel
  FramebufferSurface implements it; a host Vec<u32> implements it too):
  hits list, hover_id/pressed_id, tick-driven bounded animations, theme
  tokens (present palette from genesis.rs).
- Widgets: panel, header+rule, label (chunky 2x + hi-res 1:1 physical text
  tiers), button (normal/primary/disabled + hover/press), list item,
  checkbox, seg-bar, outline, diamond, pointer glyph, bracket arc.
- Layout: rect keyframes + lerp helper (the lab's DREAM_LAYOUTS pattern).
- seed-kernel `shell_host/genesis.rs` switches to the crate via re-export;
  rendering must stay EXACTLY as today.

GATE (hard): golden-frame — host test renders the shell for a fixed
RuntimeStatus into a pixel buffer through the OLD path and the NEW path;
byte-equal or red. Plus: QEMU quick profile needles unchanged, kernel
release build green. Taboos: no behavior change, no new colors/metrics, no
allocator use beyond what shell_host already does, no foreign crates.

## Lane 2 — wasm target + lab swap (S/M)

Goal: `wasm32-unknown-unknown` build of raios-genesis-ui + a thin JS shim;
the lab page replaces its hand-written JS replica with the real crate
(canvas gets the pixel buffer). Gate: lab renders pixel-identical to Lane 1
host render (compare PNG hashes); the JS replica is retired to a reference
folder. From then on design iteration IS Rust-code iteration.

## Lane 3 — dream skin as opt-in theme (M, only after 1+2 green)

Goal: the lab's dream design (bracket layout, closed/ambient/open keyframes,
hover-breathing, ghost buttons 130x16 uniform, hi-res text tier, baked
dither background, starfield) lands as a SECOND theme/layout in the crate,
selectable at the shell (default stays the current design until owner
flips). Implementation constraints from the performance check: background
baked once into a RAM buffer at boot; transitions max 6 frames tick-driven;
hover zone hysteresis +-20; pointer stays front-layer. Gate: golden-frame
against the lab render for the three keyframe states + QEMU quick green +
owner W5-style look approval.

## Design-system rule (owner, "aus einem Guss")

Consistency is enforced at compile time, not by style guide:

- ALL metrics live in one `tokens` module: button 130x16, row heights, the
  spacing scale, underline weights, both text tiers, the full palette.
  No widget takes a free-form color or size — only token references.
- The widget layer is the ONLY drawing API the shell sees; the raw
  primitives (fill_rect, text) are `pub(crate)`-private behind it. A shell
  file that tries to draw ad-hoc does not compile.
- One `scale` value multiplies the whole token set (future HiDPI/other
  panels) so everything scales together or not at all.
- Decorative background art (dither, starfield, procedural candle) is baked
  once at boot into the background buffer — never drawn per frame.

## Standing rules

- Design changes arrive ONLY as design-delta exports from the lab; the lab
  README documents the protocol. No invented semantics: new panels/buttons
  land unwired (no-op actions) — wiring is a separate lane.
- The orchestrator owns file sets and commits, as always.
