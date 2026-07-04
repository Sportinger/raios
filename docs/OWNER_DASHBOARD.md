# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-04 (plan restructuring session, review-4).

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

- Full verification profile: **RED since 2026-07-02** (last green:
  6789/6789 predicates). Two of the failures are real, unexplained
  predicate failures — not test-infrastructure noise.
- Working tree: ~20,500 lines uncommitted (must be committed in M0).

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Current milestone

**M0 Stabilize** — commit the working tree, fix the full test harness,
get the gate green. Nothing else until then.

## Next milestones (docs/ROADMAP.md)

M1 host tests + CI -> M2 shrink code ~10x -> M3 first real disk write ->
M4 real service isolation (Wasm) -> M5 second service -> M6 first external
AI-built service through the full safe-promotion loop.
