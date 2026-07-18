# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 6 closed)

Main road = on-device factory. The engine side of T2 is DONE: all three
suspension sources exist (Host, Atomic, opt-in FuelQuantum behind
`resumable_fuel`, default off — kernel semantics untouched, 32 conformance
tests). The Bauplatz file world is complete except glue: read-only
chunk-CAS mounts, quota-atomic RAM arenas, /out freeze and the
double-build egress gate (32 shim + 612 core tests). What remains to the
first on-device hello.rs: T2-b pump, WASI slice 5+6, guest-memory
ceilings, sysroot artifacts.

## Next step

Running: T2-b1 (pure thread-scheduler state machine in raios-core — ADR
0016 wake rules host-proven before kernel glue) and WASI slice 5
(args/env, fuel clock, seeded PRNG, proc_exit). Then T2-b2 kernel pump
(N resumable invocations, thread-spawn import, fuel quanta), WASI slice 6
glue, Bauplatz memory ceilings. Owner questions pending: SCOPE §6
Cranelift wording; ADR 0017 veto window.

## Recently (exactly 3, newest first)

### 2026-07-18 — T2-a fuel-quantum yield + WASI-4 writable build root
Opt-in Suspension::FuelQuantum parks BEFORE the instruction, resumes with
zero inputs, default byte-identical (9137872). RAM arenas with four
atomic quotas, unshadowable reserved names, XDEV boundaries, /out freeze
to BuildFS v1 and typed egress only for byte-identical double runs
(f8f5804). 32 conformance + 32 shim + 612 core tests.

### 2026-07-18 — T1-d-2 suspension core + WASI-3 chunk-CAS readonly view
Typed WasmOutcome::AtomicSuspend with call_func exit discipline, one-i32
resume (2c59257, QEMU quick green shadow-20260718-190418-6008). BuildFS
v1 + range-verified reads: a 100-byte pread in a 71-MB fixture touches
exactly one chunk (2698a70). ADR 0017 fixed the build-guest contracts.

### 2026-07-18 — T1-d-1 opcode surface + WASI-2 shim core
wait/notify translate with typed stacks and fail-closed placeholder traps
(fab0dab). raios-wasi-preview1: errno golden table from official WITX,
escape-proof paths, lowest-free fd discipline, zero deps (91b3cd5).
