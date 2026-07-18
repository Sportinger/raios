# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 8 closed)

Main road = on-device factory. **The kernel runs deterministic green
threads**: the fixed-thread round-robin pump drives real wait/notify/
atomics/fuel-quantum jobs in QEMU — `RAIOS_THREADS selftest=pass` with
internal double-run trace equality, numbers byte-identical to the host
model (dacbfee). BuildGuestClassV1 binds all measured limits (e51dc2a).
The 06 breakdown is refined to separately provable units per owner
directive — 6 boxes genuinely green (c23901a). Remaining to hello.rs:
thread-spawn/proc_exit/cap in the pump, WASI slice 6 glue, live 1-GiB
guest memories, sysroot artifacts.

## Next step

Running: T2-b2b (wasi thread-spawn import, proc_exit job end, cap 48 in
the kernel pump — closes the T2 box) and buildfs-pack (host tool packing
a directory tree into BuildFS v1 manifests + chunks — sysroot/compiler
delivery prep). Then WASI slice 6 kernel glue, live guest memories,
sysroot import, first hello.rs build. Owner questions pending: SCOPE §6
Cranelift wording; ADR 0017 veto window. Note: kernel wat build-dep
pinned the lockfile to indexmap 2.7.1 (edition2024 vs pinned nightly).

## Recently (exactly 3, newest first)

### 2026-07-18 — Kernel round-robin pump proven live (T2-b2a)
thread_job.rs pumps N threads of one job through the proven scheduler;
serial-triggered threads.selftest runs the job twice and passes only on
byte-identical traces. Live QEMU: waits=1 notifies=2 fuel_yields=32
switches=35 sum=32 rounds=35 — exactly the host-model numbers. QEMU
quick passed (shadow-20260718-195441-26032). dacbfee, e51dc2a, c23901a.

### 2026-07-18 — T2-b1 scheduler policy + WASI-5 process determinism
JobThreadScheduler: cap 48, FIFO queues, round-start timeout sweep,
notify-beats-timeout, deadlock verdict, typed replay trace (97d163c,
626 tests). Process world: preview1-exact args/env, fuel clock, pinned
xoshiro PRNG, typed Exit/Yield (83c8631, 45 tests).

### 2026-07-18 — T2-a fuel-quantum yield + WASI-4 writable build root
Opt-in Suspension::FuelQuantum parks BEFORE the instruction, default
byte-identical (9137872). RAM arenas with atomic quotas, /out freeze to
BuildFS v1, typed egress only for byte-identical double runs (f8f5804).
