# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-06 (M6A-1 — raiOS can now take in an outside program as a
checked, inert candidate; loading it still refused).

## What raiOS can actually do today

- Boots on a VM and on the bonded machine into its own graphical UI.
- You can chat with OpenAI from inside the OS over a pinned, fail-closed
  TLS connection (pin-only; not yet full certificate-chain validation).
- The system can describe itself through typed read-only commands
  (snapshot, devices, services, problems, event log).
- One built-in demo service can be loaded, hot-swapped v1<->v2, and
  rollback-previewed — all RAM-only.
- NEW (M6A-1): raiOS can now take a Wasm program that did NOT come baked
  into the system, check that it is real (hash + parse), and hold it in
  memory as an inert "candidate" — while running, loading, and saving it
  stay firmly refused. This is the receiving door for outside code; giving
  that code any rights is a later, gated step. Delivery from truly outside
  the image arrives next slice (today the bytes are a labeled test sample).

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

**M2 is CLOSED** (ADR 0006, provisional-overridable): the structural
disease is cured — one record model with non-divergent hashing, one
dispatch table, one command representation, one selftest runner, every
file agent-readable, zero-warning build, nine green FULL profiles. Line
count landed at ~126.5k (not the original ~20k); the optional extra
shrink (changing output vocabulary) is deferred and remains YOUR call —
say the word and it gets scheduled.

**M3 and M4 are CLOSED.** M3: raiOS performed its first real,
policy-authorized durable disk write and the hello rollback now actually
applies using that transaction as its authority record. M4 (the deepest
safety milestone so far): foreign code now runs INSIDE a real in-kernel
WebAssembly sandbox and physically cannot call anything outside its
granted functions — a module that even *imports* a forbidden function
fails to load. Four hostile-guest cases (broken bytes, memory hog,
infinite loop, crash) all end as clean evidence, never a kernel crash.
Proven: 465/465 checks incl. 49 wasm-specific ones.

**M5 is CLOSED — the rebuild is vindicated.** Adding a whole second
service (echo, which loads, runs its sandboxed wasm, reports health,
appears in the inventory, stops) cost **~1,060 lines** — a descriptor
plus a small state machine reusing everything built in M2–M4. A copy of
the old approach would have been ~19,000 lines. That number IS the proof
that the giant refactor worked: the system can now grow by services, not
by monoliths. Verified: 486/486 checks (67 echo-specific) + full profile
7,825/7,825.

Now active: **M6 Promotion Loop v0** — the FINAL milestone and the
project's first true product moment: one AI-authored artifact travels
the whole safe loop end to end — authored, tested in the Shadow VM,
capability-granted, promoted live, and rolled back — with evidence at
every step. Split into M6A (candidate intake) → M6B (grant) → M6C
(promote) → M6D (rollback). **M6A-1 done**: the receiving door exists
(above). Next (M6A-2): deliver a real outside Wasm file to the running
system and bind a test report to it — loading still refused until M6B.

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M1 host tests + CI -> M2 shrink code ~10x -> M3 first real disk write ->
M4 real service isolation (Wasm) -> M5 second service -> M6 first external
AI-built service through the full safe-promotion loop.
