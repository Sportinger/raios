# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18 ~22:40, loop running)

Main road = on-device factory. **Slice-6 kernel glue box CLOSED** (gate
4e17c10 + ADR 0020 storage: core authority 9028e61, kernel adapters
30fb378) and **T2 box CLOSED** (deadlock negative 3f2a64a) — both under
permanent quick-profile needles, 499/499. Sysroot BuildFS pinned
(13daf6f9, artifacts verified on C:, E: optional). One lane live: **B1G**
(Bauplatz memory proof: growth-to-ceiling fixture + over-class denial).

## Next step

**Fuel suspension for dynamic bulk charges** (ADR pending, twin opinions
in flight): wasmi's per-byte memory.grow/copy/fill charge is TERMINAL
when it exceeds the residual quantum — host-proven root cause of the red
memselftest (repro test 3248408; selftest landed honest-red 581279d,
green needles follow the fix). Rustc itself will hit this. Then: Bauplatz
box (512M + Surface-RAM via -GuestMemoryMB, c5533eb), sysroot import,
hello.rs W5 proof. Owner questions open: SCOPE §6 Cranelift wording; ADR
0017 veto window. Owner forward plans recorded (b2eb324): socket → GPU →
installer, activation after the factory proof.

## Recently (exactly 3, newest first)

### 2026-07-18 — Slice-6 closed, T2 closed, sysroot pinned (iteration 10)
Storage authority two-stage per ADR 0020 (twin second opinions, dissent
recorded): granted per-read-rehashed chunk reads, pre-I/O commit gate,
single-use write handle. Futex deadlock → deterministic JobDeadlocked
live. Docs hygiene now 10/11 self-tested (ADR form + archive dating,
metadata backfill). 24 commits, all pushed.

### 2026-07-18 — Slice-6 stage A: kernel WASI gate live (G1a)
Module bytes → ordered import extraction → authorize → exact-30 linker (no
fallback) → checked guest memory → runner. Positive fixture ran to exit 0,
extra-import fixture denied pre-instantiation. Host 54/54, QEMU quick
499/499 (4e17c10).

### 2026-07-18 — Docs hygiene mechanized (D1)
single-source phrase, root-instruction path check, STATUS red path,
plan-category mapping — self-tested predicates (2259b95, c265679).
ADR 0019 records branchless main + single git writer (aaa9a1c).
