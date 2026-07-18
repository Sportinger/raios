# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 7 closed)

Main road = on-device factory. The T2 policy is host-PROVEN: the pure
JobThreadScheduler encodes every ADR 0016 wake rule with a replay-equal
event trace (626 core tests). The WASI shim is complete on the host side
(slices 0-5): file world, process world (manifest-bound args/env, fuel
clock, pinned xoshiro PRNG, exit-first-wins) — 45 shim tests. All engine
suspension sources shipped earlier. Remaining to the first on-device
hello.rs: T2-b2 kernel pump glue, WASI slice 6 glue, Bauplatz memory
ceilings, sysroot artifacts.

## Next step

Running: T2-b2a (kernel round-robin pump over fixed pre-instantiated
threads, wat build fixture, RAIOS_THREADS selftest with internal
double-run trace equality — worker writes, orchestrator compiles/QEMUs)
and Bauplatz-2a (typed BuildGuestClassV1 contract with measured limits).
Then T2-b2b (thread-spawn import, proc_exit job end), WASI slice 6,
Bauplatz kernel wiring, sysroot import. Owner questions pending: SCOPE §6
Cranelift wording; ADR 0017 veto window.

## Recently (exactly 3, newest first)

### 2026-07-18 — T2-b1 scheduler policy + WASI-5 process determinism
JobThreadScheduler: cap 48, resumed round-robin, FIFO queues, round-start
timeout sweep, notify-beats-timeout, deadlock verdict, typed replay trace
(97d163c, 626 tests). Process world: preview1-exact args/env, fuel clock,
seeded xoshiro256** with pinned vectors, typed Exit/Yield, clock-only
poll_oneoff with next_wake_fuel (83c8631, 45 tests).

### 2026-07-18 — T2-a fuel-quantum yield + WASI-4 writable build root
Opt-in Suspension::FuelQuantum parks BEFORE the instruction, default
byte-identical (9137872). RAM arenas with atomic quotas, /out freeze to
BuildFS v1, typed egress only for byte-identical double runs (f8f5804).

### 2026-07-18 — T1-d-2 suspension core + WASI-3 chunk-CAS readonly view
Typed WasmOutcome::AtomicSuspend with one-i32 resume (2c59257, QEMU quick
green). BuildFS v1 range-verified reads: a 100-byte pread in a 71-MB
fixture touches exactly one chunk (2698a70).
