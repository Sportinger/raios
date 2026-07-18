# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 3 closed)

Main road = on-device factory. The cage speaks atomics: T1-a/b/c are merged
(shared memories, 15+49 atomic operators, 15 conformance integration tests).
The rustc import surface is MEASURED (30 imports) and frozen behind the
fail-closed grant family `raios.wasi_build_imports.v1` (601 core tests).
Kernel heap is memmap-based (291 MiB in the 512M VM). wait/notify design is
decided: ADR 0016 (first-class engine suspension; dissent recorded).

## Next step

Running: T1-d-1 (wait/notify opcode surface) and WASI slice 2
(`raios-wasi-preview1` core crate). Then T1-d-2 (suspension core — the risk
package, review mandatory), T1-d-3, WASI slices 3-5, T2 pump (thread cap
≥40 per ADR 0016, needs non-terminal fuel yield). Owner questions pending:
SCOPE §6 Cranelift wording; WASI ADR list in plan §6 (BuildFS format, guest
epoch, root-tmp policy).

## Recently (exactly 3, newest first)

### 2026-07-18 — T1-c RMW/cmpxchg + WASI grant family + ADR 0016
All 49 RMW operators execute with spec wrap-compare (323f974); the measured
30-import surface is a typed fail-closed grant in raios-core (2f29e96,
601 tests); wait/notify mechanism decided with both second opinions in
ADR 0016 (e2afccf) — thread cap corrected to ≥40 (measured 26-32).

### 2026-07-18 — Atomics stage 1 + measured import inventory + docs predicate
T1-b: 15 atomic loads/stores + fence with UnalignedAtomic trap (863ea4d).
wasm-import-inventory tool measured the pinned artifact (re-downloaded,
SHA byte-identical): 30 imports incl. env.memory shared max 1 GiB (b3c2df4).
Docs hygiene predicate green + first three scope-07 boxes checked (2a1c63a).

### 2026-07-18 — Heap from memmap + wasmi shared-memory foundation (T1-a)
Kernel heap memmap-based: QEMU quick passed (shadow-20260718-175235-9680);
512M probe → `RAIOS_HEAP source=memmap size_mib=291`; 256M negative →
`static_fallback` with boot continuing. wasmi takes shared memories opt-in
(8a58f5f, b93a743).
