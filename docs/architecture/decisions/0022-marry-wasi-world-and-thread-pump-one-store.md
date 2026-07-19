# 0022 — Run the real compiler on ONE store: the WASI world and the thread pump married behind the exact-30 gate

Date: 2026-07-19 · Status: active

## Context

The 91-MB rustc module loads and instantiates in-kernel (ADR 0018/0020
chain, commit 27fa7f6) but its start section traps under single-threaded
isolated instantiation — it needs real threads and mounted files. Two
subsystems must be married to run it: the WASI build runner
(`Store<WasiHostState>`, 30-import linker, single-threaded `run_start`) and
the deterministic green-thread pump (`Store<ThreadJobStoreState>`, 3-import
linker, `JobThreadScheduler`, replay-equal). Two independent second
opinions were taken (Codex xhigh read-only, Fable max); they converged.

## Decision

**One store carries the whole run.** wasmi entities are store-owned and
fuel is store-global, so connecting two stores with locks/Arc is wrong —
the WASI world and the thread world merge into a single store.

1. **Merged state.** Extend `WasiHostState` with a thread world:
   `ThreadHostMode { Deny, Scheduled(ThreadWorld) }`, where `ThreadWorld`
   carries today's `ThreadJobStoreState` verbatim (JobThreadScheduler,
   pending_spawns FIFO, spawn/cap counters). The `WasiBuildInstance` (one
   fd table, mounts, RAM arenas, PRNG) and the one `GrantedChunkReader`
   stay in store data, shared by every thread — no lock, no per-thread
   view, no cloned reader. This is safe because the pump is single-core and
   exclusive: exactly one continuation or host shim runs between pump
   points, so a WASI call is an atomic linearization point in scheduler
   order. **Merge law:** anything a host import touches lives in store data;
   anything that resumes wasmi (module, linker, memory handle, thread
   slots, continuations, fuel banks) lives in the runner — which makes the
   no-reentry invariant structural (an import cannot reach a continuation).

2. **Import routing.** The linker stays the measured 30 entries in
   inventory order; the AuthorizedBuildJob gate and the
   registered==30==authorized check are untouched. Only two host fns branch
   on the mode: `wasi.thread-spawn` (Scheduled: reserve a TID synchronously
   via `scheduler.spawn()`, push a SpawnRequest, return TID; cap → −1; NO
   wasmi reentry) and `proc_exit` (one job transition: commit process exit
   + `scheduler.proc_exit`, set terminal latch, raise the typed trap). The
   other 28 route to the instance from whichever thread runs. Sharing fd
   table + mounts across threads costs zero code — it falls out of one
   store. Worker materialization re-instantiates the SAME authorized module
   through the SAME 30-import linker sharing the one `env.memory`, then
   `wasi_thread_start(tid,start_arg)`; it runs only when scheduling selects
   the TID. Any materialization failure ends the job deterministically.

3. **Per-thread fuel via a narrow vendored seam.** Store fuel is one
   counter, so per-thread fuel is a pump discipline — but it must NOT be
   done with public `add_fuel`/`consume_fuel` (that inflates wasmi's total
   counter and corrupts the fuel-derived clock — Codex's objection,
   adopted). Add a narrow vendored-wasmi seam that SWAPS the raw remaining
   fuel without changing total. Per selected thread: assert remaining is 0,
   install the slot's escrow (+ one class quantum if it needs refill,
   incrementing a job-wide granted-total checked against
   `max_total_fuel` — the ADR 0021 rule that the ceiling caps GRANTS, not
   consumption), run/resume, then sweep remaining back to the slot at 0.
   Fuel cannot cross TIDs (escrow swept out before the next thread installs
   — the escrow-theft invariant is structural). ADR 0021 bulk parking is
   preserved: a large charge restores its own residual and banks +Q per
   scheduled round until it fits.

4. **A separate logical-fuel ledger drives clocks.** Once per-thread escrow
   is installed, `fuel_consumed()` no longer means job time. Maintain a
   monotonic logical-fuel ledger (active_start_remaining − current) synced
   at host calls and pump boundaries; `clock_time_get`, atomic deadlines,
   and clock-only `poll_oneoff` read it. A not-yet-due poll becomes a
   deterministic parked-host op completed by (deadline, request_sequence,
   tid), never by wall-clock or I/O order.

5. **Determinism levers (both opinions insist).** (a) Reserve the shared
   memory's backing at class max (16384 pages) at job admission
   (try_reserve, fail fast before any guest byte) so grow never reallocates
   — this purifies `ResourceLimiter::memory_growing` of the non-replayable
   `ALLOCATOR.free()` term AND removes the Vec-doubling transient; the
   admission check becomes a pure `desired ≤ class.max_pages`. (b) Record a
   WASI-EFFECT digest {round, tid, sequence, opcode, arg digest, result
   digest} over every fd/path/random/clock/stdout effect — scheduler-trace
   equality alone does NOT prove interleaved-effect equality. (c) The class
   runner records the rolling trace_digest + bounded counters, not the
   fixture-scale full trace Vec; the round limit is class-derived
   (max_total_fuel/quantum), not the fixture's 4096.

6. **Both existing selftests stay frozen.** The single-thread WASI
   selftests keep `WasiHostState`+`ThreadHostMode::Deny`+`run_start`
   bit-identical; the threads QEMU selftest keeps `thread_job.rs`, its
   fixture, 3-import linker, serial format, and double-run predicate
   UNCHANGED in the marrying commit. The merged pump is a new module reusing
   the same `JobThreadScheduler`; a later mechanical lane may fold the
   fixture onto a generic pump (parameterized by fuel policy) only behind
   the frozen selftest as the gate. One commit of ~200-line pump
   duplication is cheaper than risking the only proven determinism baseline.

## Alternatives & second opinions

The two opinions concur on the architecture (one store, shared instance,
runner-owned module/linker/slots, queue-then-materialize spawn, escrow
fuel, effect digest, frozen selftests, capstone double-run). Recorded
dissent:

- **State shape:** Fable — extend `WasiHostState` in place with a
  `ThreadHostMode` enum (the 28 fns keep their state type; minimal churn).
  Codex — a new `BuildThreadState { WasiWorld, ThreadControl, fuel ledger }`
  with the 28 shims refactored over a state-access trait (tidier long-term,
  more churn now). **Chosen:** Fable's extend-in-place for the marrying
  commit (smallest diff, 28 fns untouched); Codex's trait refactor is a
  fast-follow once the merged pump has its own green digest baseline.
- **Fuel mechanism:** Fable — sweep/install via store fuel ops. Codex —
  a narrow vendored raw-remaining swap seam, because public fuel ops
  inflate the total counter and corrupt the clock. **Chosen:** Codex's
  seam (safer; Fable's banking semantics ride on top of it unchanged), plus
  the separate logical-fuel ledger both effectively require.

## Consequences

Easier: the real compiler runs with the full WASI surface AND deterministic
threads on one store; the gate, reader, and per-read rehash apply to every
thread for free; ADR 0021 and the deadlock/​cap proofs carry over. Harder: a
new vendored fuel seam (kept minimal, behind resumable_fuel), a WASI-effect
digest to maintain, and a 1-GiB up-front memory reservation per build job
(deliberate: determinism over frugality). The capstone predicate is a full
compiler DOUBLE run with equal trace digest AND equal effect digest AND
equal frozen-output sha — schedule and effect determinism pinned together.
Negative battery: escrow theft, split-WASI-world (two threads one fd),
spawn-after-exit, cap-49, secondary-start canary, ceiling-mid-multithread,
grow-at-max-from-worker, futex deadlock, authority bypass from a worker.
