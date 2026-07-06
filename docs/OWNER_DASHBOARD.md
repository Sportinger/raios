# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-06 (M7 underway — making things survive a restart. raiOS now
has TWO of its three safe disk-write abilities live: (1) it appends durable
records to a log — build, bounds-check, write, read back, confirm byte-identical
before saying "done"; and NEW (2) it safely marks a boot as successful and does
crash-safe A/B boot-slot switching, entering a safe "recovery" mode if the
control record is missing/damaged (M7C complete). Every other place on the disk
stays refused. Independent security checks could not break either write. Still
within a single boot; surviving an actual reboot is next (M7D).).

## What raiOS can actually do today

- Boots on a VM and on the bonded machine into its own graphical UI.
- You can chat with OpenAI from inside the OS over a pinned, fail-closed
  TLS connection (pin-only; not yet full certificate-chain validation).
- The system can describe itself through typed read-only commands
  (snapshot, devices, services, problems, event log).
- One built-in demo service can be loaded, hot-swapped v1<->v2, and
  rollback-previewed — all RAM-only.
- NEW (M6A-1 + M6A-2a): raiOS now has a working receiving door for outside
  code. A real Wasm program that did NOT come baked into the system can be
  sent in over the console (in small encoded pieces that get reassembled),
  checked for realness (fingerprint + parse), and held in memory as an
  inert "candidate" — while running, loading, and saving it stay firmly
  refused. Verified end-to-end with a real 4 KB program. Giving that code
  any rights is the next, gated step (M6B). Independently security-checked;
  one known limit noted: the realness-check itself isn't yet time-capped.

## Gate status

- Full verification profile: **GREEN** as of 2026-07-06 — 8,168/8,168
  checks passed in one run (report shadow-20260706-231420-33040.json,
  hash-verified) after M7C complete (boot control read + write). The focused
  persistence profile is now 40/40 (adds the boot-control read, SAFE-posture,
  boot-success-mark, ping-pong, and last-good needles) and the audit-rollback
  profile is unchanged-green (1,709/1,709). The old "mystery" failures are
  explained: the test tooling asked for too much data at once and then misread
  its own connection loss — no bug in the OS itself.
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

Now active: **M6 Promotion Loop v0** — the project's first true product
moment: one AI-authored artifact travels
the whole safe loop end to end — authored, tested in the Shadow VM,
capability-granted, promoted live, and rolled back — with evidence at
every step. Split into M6A (candidate intake) → M6B (grant) → M6C
(promote) -> M6D (rollback). **M6 COMPLETE (dev-tier RAM loop closed).** A real
outside program is received over the console, checked, its identity recorded,
granted its rights, loaded, run inside the sandbox, and rolled back in RAM
through a verified undo path. It does not yet save to disk and does not claim
durable/native/owner-sealed authority. Today's signing key is a deliberate DEV
key so the loop can be built and tested; **your own key K seals it for real
later** (the sealing ceremony is the very last step).

**M7 Persistence Foundation now active — making things survive a restart.**
Done so far: the kernel reads the disk's layout + its durable log (M7A/M7B-1,
read-only); performs its **first real safe WRITE** — appending one durable
record and reading it back to confirm it, every other disk area still refused
(M7B-2); reads the boot-control area to decide which system copy (A/B) to boot and
whether to enter a safe "recovery" mode (M7C-1); and now — NEW — **safely marks
a boot as successful** and lets safe-mode switch off saving (M7C-2). Concretely:
when a boot passes its health checks, the system writes a "this copy booted OK"
record into the *spare* boot slot and reads it back to confirm — so a power cut
mid-write can only damage the spare, never the copy that's currently trusted
(crash-safe A/B switching). And if the boot-control record is missing or
damaged, the system refuses to save anything (safe recovery mode). There's also
a small offline tool for you to pre-select which copy boots next. raiOS now has
**two** of its three safe disk-write abilities live (the log, and boot control);
everything else stays refused. **Next: M7D** — survive an actual reboot, preceded
by saving the promotion record durably (M6D-2).

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M6 first external AI-built service through the full safe-promotion loop
-> M7 things survive a restart (persistence + automatic fall-back to the
last good state) -> M8 emergency lifeline -> M9 real long-term memory ->
M10 stronger provider trust + a second AI provider -> M11 shrinking the
core (network parsing moves out into replaceable services) -> M12+ Wi-Fi,
downloading modules over the network, moving to new hardware.

NEW (2026-07-06): M7-M11 are fully pre-planned as step-by-step maps with
ready-made worker instructions, plus a procedure handbook
(`docs/ORCHESTRATOR_PLAYBOOK.md`). Purpose: cheaper AI agents can keep
building correctly even without an expensive orchestrator model. Every
map starts with a mandatory "check the plan against reality" step.
