# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~06:10, loop running)

Main road = on-device factory. **The real rustc EXECUTES inside raiOS**
(150a968): reassembled from the store (sha c6dccf3e), gated, real sysroot
mounted at /sysroot, instantiated on the merged WASI+threads pump (ADR
0022), start section runs (fueled) until it hits the wasip1-threads
`__wasm_init_memory` atomic.wait barrier — T1 models atomic.wait as a
suspension, which wasmi's non-resumable pre.start can't handle
(RAIOS_RUSTCRUN reason=pre_start_atomic_suspend). Everything up to that
barrier is proven. Tree clean, all pushed.

## Next step

**Vendored resumable-start seam** (next brick): run the module's start
section through the resumable pump (not pre.start), so the atomic.wait is
evaluated by the scheduler (main thread does __wasm_init_memory init, no
real block) instead of trapping — the direct analog of the fuel seam
(19fde8e). Then rustc proceeds to _start: expect worker spawns + real
sysroot reads → measure. Then hello.rs compile + /out freeze + double-build
egress (Brick C). Owner questions open: SCOPE §6 Cranelift wording; ADR
0017 veto window.

## Recently (exactly 3, newest first)

### 2026-07-19 — First in-kernel rustc execution (RB, Brick B)
Real compiler runs on the merged pump with the mounted sysroot; start
section executes to the threads shared-memory barrier (150a968). Needs a
vendored resumable start. Combined sysroot+compiler image kernel-verified.

### 2026-07-19 — WASI world married to the thread pump (ADR 0022, MP+FS)
One store, shared instance, queue-then-materialize spawn, per-thread fuel
escrow via a vendored raw-remaining swap (19fde8e), effect digest. Proven:
multi-thread WASI fixture double-run trace+effect equal (6e3886a, 502/502).

### 2026-07-19 — Real rustc compiler loads + instantiates (CL, MM)
91-MB module reassembled from CAS (sha c6dccf3e), parsed, authorized,
instantiated (27fa7f6). Unblocked by the idempotent-MMIO fix (86fe9b9).
