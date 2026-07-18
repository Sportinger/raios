# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 9 closed)

Main road = on-device factory. **The T2 box is closed**: the kernel pump
runs the full wasi-threads protocol — deferred thread-spawn, proc_exit
whole-job end, cap-48 denial — live in QEMU (`RAIOS_THREADS selftest=pass
… spawns=2 cap_denials=1 exit_code=0`, double-run trace equality, QEMU
quick green, 8321953). buildfs-pack turns a tree into verifiable BuildFS
v1 (c20f8ad). Before the WASI kernel-glue slice, two independent reviews
(owner directive) require an integration-boundary + new-code hardening
pass — now running. Remaining after that: WASI slice 6 glue, live 1-GiB
guest memories, sysroot import, first hello.rs.

## Next step

Running (owner-directed pre-glue hardening, no general kernel refactor):
HARD-core (bounded scheduler trace, park_wait current-thread check,
FrozenOutput bound to bytes, opaque AuthorizedBuildJob, guest-class
validate relations) and HARD-wasi (WasiBuildInstance one-FD/mount/process
world, ramfs node-churn quota, pure guest-range/iovec checker, split
base/inheriting rights) — both host-testable, kernel untouched so the
threads QEMU selftest stays green. Then the WASI slice-6 kernel glue.
Owner questions pending: SCOPE §6 Cranelift wording; ADR 0017 veto window.

## Recently (exactly 3, newest first)

### 2026-07-18 — T2 box closed + BuildFS packer (T2-b2b, buildfs-pack)
wasi thread-spawn (deferred materialization, no import reentry), proc_exit
whole-job end, cap-48 denial — live: spawns=2 cap_denials=1 exit_code=0,
QEMU quick green (8321953). buildfs-pack: deterministic manifest + dedup'd
content-addressed chunks, fail-closed, raios-core the sole format
authority (c20f8ad).

### 2026-07-18 — Kernel round-robin pump proven live (T2-b2a)
thread_job.rs pumps N threads of one job through the proven scheduler;
threads.selftest runs the job twice and passes only on byte-identical
traces. Live: waits=1 notifies=2 fuel_yields=32 sum=32 (dacbfee).
BuildGuestClassV1 binds all measured limits, 642 tests (e51dc2a).

### 2026-07-18 — T2-b1 scheduler policy + WASI-5 process determinism
JobThreadScheduler: cap 48, FIFO queues, timeout sweep, deadlock verdict,
replay trace (97d163c, 626 tests). Process world: preview1-exact args/env,
fuel clock, pinned xoshiro PRNG, typed Exit/Yield (83c8631, 45 tests).
