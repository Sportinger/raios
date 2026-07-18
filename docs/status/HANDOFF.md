# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18)

Main road = on-device factory (`docs/plans/plan-personal-rust-playground.md`):
threads-in-the-cage T1 has its foundation merged — vendored wasmi accepts
shared memories behind an opt-in flag (host-proven, 7 tests). The first memory
wall is down: the kernel heap comes from the Limine memmap (291 MiB in the
512M VM, 1 GiB cap on hardware; static 64-MiB fallback proven at the
boundary). WASI preview1 has a measured 7-slice plan (plan §6). Everything is
on `main`; origin/main is current again (was ~400 commits behind, 2026-07-18).

## Next step

T1-b: atomic loads/stores/fence in vendored wasmi (host-testable), then T1-c
RMW/cmpxchg, T1-d wait/notify via resumable suspension, T2 round-robin pump.
Parallel: WASI slice 0 (import inventory tool; orchestrator runs it against
`E:\raios-probe-rustc-wasm`) and the docs-hygiene predicate (scope 07).
Owner questions pending: SCOPE §6 Cranelift wording (breakdown already says
Route B); WASI ADR list in plan §6 (BuildFS format, guest epoch, root-tmp).

## Recently (exactly 3, newest first)

### 2026-07-18 — Heap from memmap + wasmi shared-memory foundation (T1-a)
Kernel heap memmap-based: QEMU quick passed (shadow-20260718-175235-9680);
512M probe → `RAIOS_HEAP source=memmap size_mib=291`; 256M negative →
`static_fallback` at largest=35 MiB with boot continuing. wasmi takes shared
memories opt-in; atomics still cleanly rejected (slice boundary). b93a743, 8a58f5f.

### 2026-07-18 — WASI preview1 slice plan landed (plan §6)
7 slices, measure-first (import inventory tool), new typed grant family
`raios.wasi_build_imports.v1`, dependency-free no_std shim crate, fuel-derived
deterministic clock/random so double-build stays byte-equal. Full report:
`docs/_archive/2026-07-18_wasi-preview1-slice-plan-full.md`.

### 2026-07-18 — main consolidated, workspace-grouping folded in
origin/main was ~400 commits stale; fast-forwarded to the workspace-grouping
tip, branch deleted, main-only invariant restored. Host conformance crate
`raios-wasmi-conformance` scaffolded (wat cached offline) as the T-family seam.
