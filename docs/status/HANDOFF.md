# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18 ~21:40, loop running)

Main road = on-device factory. **Slice-6 stage A LANDED and QEMU-proven**
(4e17c10): AuthorizedBuildJob gate → exact-30 linker → WasiBuildInstance →
fuel-bounded runner; `wasi.selftest` + `threads.selftest` are permanent
quick-profile needles now (499/499, shadow-20260718-213033-10928). Two
lanes live: **G1b** (store adapters: granted chunk reads + double-build
egress — completes the slice-6 box) and **T2c** (futex-deadlock negative —
closes the T2 box). Their file sets are disjoint by order.

## Next step

Collect G1b + T2c: compile loop + QEMU quick with the updated RAIOS_WASI
needle (orchestrator runs both, one QEMU at a time). Then: Bauplatz 1-GiB
memory box (399/16384 window live + over-class denial), then sysroot
import — **blocked on E: drive** (attach or re-download artifacts; owner).
Owner questions open: SCOPE §6 Cranelift wording; ADR 0017 veto window.

## Recently (exactly 3, newest first)

### 2026-07-18 — Slice-6 stage A: kernel WASI gate live (G1a)
Module bytes → ordered import extraction → authorize → exact-30 linker (no
fallback) → checked guest memory → runner. Positive fixture ran to exit 0,
extra-import fixture denied pre-instantiation, threads selftest unchanged.
Host 54/54, x86_64-seed release green, QEMU quick 499/499 (4e17c10).

### 2026-07-18 — Docs hygiene mechanized (D1)
single-source phrase, root-instruction path check, STATUS red path,
plan-category mapping — self-tested predicates, 7/7 planted faults caught
(2259b95, c265679). ADR 0019 records branchless main + single git writer
(aaa9a1c).

### 2026-07-18 — Hardening pass collected, orchestrator stopped (owner)
HARD-core: bounded trace digest, FrozenOutput byte-bound, opaque
AuthorizedBuildJob (aa3bbec). HARD-wasi: WasiBuildInstance world,
generation-tagged ramfs, guest_range checker, split rights (c8483b3,
53/53). Hung duplicate h1 killed; nothing lost.
