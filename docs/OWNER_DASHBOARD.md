# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-05 (M0 closed; M1 first slice landed).

## What raiOS can actually do today

- Boots on a VM and on the bonded machine into its own graphical UI.
- You can chat with OpenAI from inside the OS over a pinned, fail-closed
  TLS connection (pin-only; not yet full certificate-chain validation).
- The system can describe itself through typed read-only commands
  (snapshot, devices, services, problems, event log).
- One built-in demo service can be loaded, hot-swapped v1<->v2, and
  rollback-previewed — all RAM-only. Nothing can be written to disk yet,
  and no externally-built module can run yet.

## Gate status

- Full verification profile: **GREEN** as of 2026-07-04 — 7,814/7,814
  checks passed in one run (report shadow-20260704-184615-9224.json,
  hash-verified). First green since 2026-07-02. The old "mystery"
  failures are explained: the test tooling asked for too much data at
  once and then misread its own connection loss — no bug in the OS
  itself.
- Working tree: the ~36,900-line backlog was committed 2026-07-04 in
  three honest commits; release binaries are no longer tracked in git.

## Current milestone

**M0 Stabilize is DONE** (2026-07-05). New since last update: every test
run now records exactly WHY it died (VM crashed — with exit code — vs
connection glitch), and a dead VM fails the run in seconds instead of
wasting 7 minutes. Failures can no longer be misfiled as "flaky tests".

Now active: **M1 Testable Core**, nearly done. The new `raios-core`
library holds shared kernel logic and the protocol parsers, tested on
the normal PC in under a second (previously: full VM boot needed).
NEW: every commit is now built and tested automatically by GitHub (first
run green). The repo-copy mismatch is resolved — the 2 online-only
commits were your own README edits, merged safely. Last M1 step: the
automatic check also boots the OS in a VM (M1-3b), then M1 closes.

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M1 host tests + CI -> M2 shrink code ~10x -> M3 first real disk write ->
M4 real service isolation (Wasm) -> M5 second service -> M6 first external
AI-built service through the full safe-promotion loop.
