# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18, iteration 5 closed)

Main road = on-device factory. **The cage now speaks the COMPLETE threads
instruction surface of the rustc artifact**: T1-a through T1-d-2 merged —
shared memories, 64 atomic operators, and typed wait/notify suspension with
exact park/resume discipline (27 conformance tests; kernel default-closed
until T2; QEMU quick passed shadow-20260718-190418-6008). WASI slices 0-3
are merged: measured import grant, no_std shim core, chunk-CAS read-only
build filesystem (20 shim + 605 core tests). ADRs 0016 + 0017 fix the
threads mechanism and the five build-guest contracts.

## Next step

Running: T2-a (opt-in resumable fuel-quantum yield — second engine risk
package) and WASI slice 4 (RAM-tmp, /out freeze, double-build egress).
Then T2-b (round-robin pump in the kernel, thread cap ≥40, wake rules per
ADR 0016), WASI slice 5 (args/env/clock/random/proc_exit), slice 6 kernel
glue, Bauplatz guest-memory ceilings, sysroot as store artifacts, first
hello.rs build in QEMU. Owner questions pending: SCOPE §6 Cranelift wording;
ADR 0017 veto window (five build-guest contracts, recommendations applied).

## Recently (exactly 3, newest first)

### 2026-07-18 — T1-d-2 suspension core + WASI-3 chunk-CAS readonly view
Typed WasmOutcome::AtomicSuspend with call_func exit discipline, Suspension
::{Host,Atomic}, one-i32 resume (2c59257, QEMU quick green). BuildFS v1 +
range-verified reads: 100-byte pread in a 71-MB fixture touches exactly one
chunk (2698a70). ADR 0017 fixed the five build-guest contracts (039ed01).

### 2026-07-18 — T1-d-1 opcode surface + WASI-2 shim core
wait/notify translate with typed stacks and fail-closed placeholder traps
(fab0dab, 23 tests). raios-wasi-preview1: errno golden table from official
WITX, escape-proof paths, lowest-free fd discipline, zero deps (91b3cd5).

### 2026-07-18 — T1-c RMW/cmpxchg + WASI grant family + ADR 0016
All 49 RMW operators with spec wrap-compare (323f974); measured 30-import
surface as typed fail-closed grant (2f29e96, 601 tests); wait/notify
mechanism decided with both second opinions in ADR 0016 (e2afccf).
