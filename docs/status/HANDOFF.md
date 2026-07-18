# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-18 ~21:00, orchestrator stopped by owner)

Main road = on-device factory. **The pre-glue hardening pass is COMPLETE
and collected**: HARD-core landed via the loop (aa3bbec), HARD-wasi
finished after the owner hard-stopped the orchestrator and was verified
(53/53 independently re-run) and collected by the owner session (c8483b3).
A third dispatched worker (pkg-h1) hung 25 min with zero file output and
was killed — its content was superseded by HARD-core (trace_digest chain
confirmed on disk), nothing lost. Tree clean, all pushed.

## Next step

Resume the loop at: WASI slice-6 kernel glue along the reviewed
integration boundary (opaque AuthorizedBuildJob -> WasiBuildInstance ->
checked guest_memory -> linker table -> runner; zero deps on event_log/
durable_store/usb/legacy renderer; threads QEMU selftest must stay green
unchanged). Then live 1-GiB guest memories, sysroot import, first
hello.rs. Owner questions pending: SCOPE §6 Cranelift wording; ADR 0017
veto window. Side note: owner runs a UI design lab outside this repo
(raios2-ui-lab); UI changes will arrive as design-delta lane orders.

## Recently (exactly 3, newest first)

### 2026-07-18 — Hardening pass collected, orchestrator stopped (owner)
HARD-core: bounded trace digest, FrozenOutput byte-bound, opaque
AuthorizedBuildJob (aa3bbec). HARD-wasi: WasiBuildInstance one-FD/mount
world, generation-tagged ramfs slots, guest_range checker, split rights
(c8483b3, collected by owner session, 53/53). Hung duplicate h1 killed.

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
