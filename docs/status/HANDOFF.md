# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~06:40, loop running)

Main road = on-device factory. **The real rustc compiler EXECUTES inside
raiOS.** Full pipeline live: store → exact-30 gate → sysroot mounted →
instantiate → start section (via the vendored resumable-start seam,
afbfba8) → _start running real rustc bytecode on the merged pump. Proven
by a deterministic round counter climbing 6944→351968 (sustained; fuel
metering forces a suspension per quantum, so advancing = forward
execution). Tree clean, all pushed.

## Next step

**Diagnose the runtime-init spin.** At ~350k rounds (~3.5e11 instructions,
~100x a native `rustc --version` budget) stdout=0 / spawns=0 — rustc is
caught in an init spin against some preview1 edge semantic (the plan's
named post-T1/T2 recalibration). Next brick: a per-import call histogram in
the pump to find the hammered WASI call (likely a clock/futex/thread-detect
loop), then fix that semantic. Then hello.rs compile + /out freeze +
double-build egress (Brick C). Separately: the AOT execution stage
(roadmap Stufe 4) for practical speed — interpreter-under-TCG is inherently
slow. Owner questions open: SCOPE §6 Cranelift wording; ADR 0017 veto.

## Recently (exactly 3, newest first)

### 2026-07-19 — Real rustc executes on-device (RB + RS)
Compiler runs on the merged pump with mounted sysroot; resumable-start seam
(start_split, afbfba8) clears the threads atomic barrier; _start executes
(rounds advance). Frontier: a preview1-semantic init spin + AOT speed.

### 2026-07-19 — WASI world married to the thread pump (ADR 0022, MP+FS)
One store, shared instance, queue-then-materialize spawn, per-thread fuel
escrow via a vendored raw-remaining swap (19fde8e), WASI-effect digest.
Multi-thread fixture double-run trace+effect equal (6e3886a, 502/502).

### 2026-07-19 — Real rustc compiler loads + instantiates (CL, MM)
91-MB module reassembled from CAS (sha c6dccf3e), parsed, authorized,
instantiated (27fa7f6). Unblocked by the idempotent-MMIO fix (86fe9b9).
