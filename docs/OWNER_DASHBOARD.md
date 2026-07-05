# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-05 night (M2: both emit boundaries fully ported).

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

**M0 and M1 are DONE** (2026-07-05). What that means concretely:
- Test runs record exactly WHY they died (VM crash w/ exit code vs
  connection glitch); a dead VM fails in seconds, not 7 minutes (M0).
- Kernel logic lives in a `raios-core` library tested on a normal PC in
  under a second — previously every logic check needed a VM boot (M1).
- GitHub now automatically builds the kernel, runs the tests, AND boots
  the OS in a VM with 417 checks on EVERY commit (all green, ~7 min).
  A bonus: the signed-source protection proved itself by correctly
  rejecting a mis-configured build machine on the first CI attempt.

Now active: **M2 Ceremony Collapse**. BOTH large emit boundaries
(recovery: 22 modules; module: full porting map incl. the hash-coupled
files) now render through ONE typed record model — roughly -2,000 kernel
lines at byte-identical behavior, every batch proven by VM runs, twice
capped by a green FULL profile (7,814/7,814). Remaining M2 surface: the
22.5k-line hello service file (scoping underway).

HIGHLIGHT of the day: the tooling caught and fixed the first real kernel
bug — a command copied 3.8 MB onto a small stack, randomly crashing the
OS (~50% of runs, misfiled for weeks as "flaky tests"). Chain: instant
death classification (M0) -> failure-log pattern -> checkpoint bisection
-> fix -> proven by 5/5 clean runs.

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M1 host tests + CI -> M2 shrink code ~10x -> M3 first real disk write ->
M4 real service isolation (Wasm) -> M5 second service -> M6 first external
AI-built service through the full safe-promotion loop.
