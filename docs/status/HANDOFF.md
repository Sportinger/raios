# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~04:20, loop running)

Main road = on-device factory. **The real 91-MB rustc compiler LOADS and
INSTANTIATES in-kernel from the capability-gated store** (27fa7f6): parsed,
30-import authorized, shared memory allocated, linker instantiated —
`RAIOS_COMPILERLOAD stage=instantiated file_sha=ok imports=30 mem_pages=399`
(8-GiB profile, 502/502). Sysroot import + Bauplatz + Slice-6 + T2 all
closed earlier. Enabler this round: idempotent map_mmio (86fe9b9) ended the
MMIO VA leak that broke large reads. Tree clean, all pushed.

## Next step

**Execution milestone** (the W5 factory proof). The compiler start section
traps on isolated instantiation even with full fuel — it needs real threads
+ mounted files. Bricks: (1) seeder multi-tree append → one image with BOTH
sysroot (13daf6f9) + compiler (1b9214df); (2) run the compiler through the
resumable fuel pump (run_start) with the REAL T2 ThreadHost (not the deny
stub) and the mounted sysroot, args=["rustc","--version"], then hello.rs
double-build through the landed commit gate. Owner questions open: SCOPE §6
Cranelift wording; ADR 0017 veto window. Owner plans (b2eb324): socket →
GPU → installer.

## Recently (exactly 3, newest first)

### 2026-07-19 — Real rustc compiler loads + instantiates in-kernel (CL, MM)
91-MB module reassembled from 1457 CAS chunks (sha c6dccf3e), parsed,
authorized, instantiated with shared memory (27fa7f6). Unblocked by the
idempotent-MMIO-cache fix (86fe9b9) after the VA leak failed reassembly at
~1134 reads. Start-section execution is the next milestone.

### 2026-07-19 — Sysroot import live + Bauplatz closed
Real 71-MB sysroot read through the granted per-read-rehashed reader via a
single-pass ARTSTOR index (917174b, O(n²)→O(n)); full 1-GiB window proven
both RAM profiles via ADR 0021 bulk-fuel parking + doubling-aware limiter.

### 2026-07-18 — Slice-6 closed, T2 closed
Storage authority two-stage (ADR 0020): granted rehashed chunk reads,
pre-I/O commit gate. Futex deadlock → deterministic JobDeadlocked. Docs
hygiene 11/11 self-tested; ADRs 0018–0021.
