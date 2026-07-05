# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-06 (byte-identical collapse complete; decision needed).

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

Now active: **M2 — the byte-identical collapse program is COMPLETE**
(Batches 1-5: one dispatch table, shared command structures, one
selftest runner, descriptor-table event bindings, table-built hash
inputs). Agent layer: 138k -> ~126.5k lines with PROVEN identical
behavior — eight green FULL profiles (7,814/7,814 each) along the way.
Structure quality is transformed: one record model, one dispatch, one
selftest runner, all files under 5k lines, zero-warning build, all
attested.

DECISION NEEDED FROM YOU (no rush): reaching the original ~20k-line M2
goal requires changing what the OS actually outputs (compacting the
evidence vocabulary, moving negative selftests to PC-side tests). That
is safe but heavier: test needles must be updated and an architecture
decision (ADR) recorded. Option A: accept ~126k as the M2 result and
re-scope the milestone sentence honestly. Option B: authorize the
vocabulary compaction (est. -30k+ more, gets near the goal).

Also this session: first real kernel bug found and fixed by the new
tooling (3.8 MB stack copy, ~50% random crashes) — proven by 5/5 runs.

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M1 host tests + CI -> M2 shrink code ~10x -> M3 first real disk write ->
M4 real service isolation (Wasm) -> M5 second service -> M6 first external
AI-built service through the full safe-promotion loop.
