# 0016 — Atomic wait/notify become a first-class engine suspension; all scheduler truth stays in the kernel pump

Date: 2026-07-18 · Status: active

## Context

T1-a/b/c gave the vendored wasmi shared memories and the full atomic opcode
surface except `memory.atomic.wait32/64` and `memory.atomic.notify`. Those two
need a park/wake mechanism, but wasmi 0.31's resumable machinery only covers
host-function errors — an instruction-level trap is terminal today. The green-
thread scheduler (T2) will pump N resumable invocations of one job at fixed
fuel quanta; double builds must stay byte-identical. Two independent second
opinions were taken (Codex xhigh read-only review; Fable max), per CLAUDE.md.

## Decision

wait/notify are implemented as a **new, typed engine suspension**:

- Executor arms own the spec mechanics in binding order: alignment (natural,
  effective address), bounds, sharedness, then compare. Fast path: unequal
  value → push `1`, never leaves the hot loop. Unshared `wait` traps with a
  new additive `TrapCode` (`UnsharedMemoryAtomicWait`); unshared `notify`
  returns `0` inline (after its own OOB/alignment traps, also for count=0).
- Park path (wait-on-equal, shared notify): the executor follows the existing
  `call_func` exit discipline (bump ip, sync sp, push frame, cache reset) and
  returns a new `WasmOutcome::AtomicSuspend` carrying a typed payload
  `{ memory, addr, Wait{timeout_ns} | Notify{count} }`, threaded through
  `TaggedTrap` into `ResumableInvocation` as
  `Suspension::{Host{..}, Atomic(..)}`. Resume takes exactly one `i32`
  (wait: 0=woken / 2=timed out; notify: woken count). Non-resumable calls
  hitting a suspension fail deterministically (`AtomicSuspendNotResumable`),
  never hang.
- **The engine knows neither queues nor clocks.** The kernel pump owns waiter
  FIFOs keyed by `(memory identity, effective address)` in ordered structures
  (never hash maps), a virtual clock derived from pump rounds, and these
  binding wake rules: (1) notify wakes FIFO in park order at the event;
  (2) timeout expiry is processed only at round start, ordered by
  `(deadline, park sequence)`, result 2; (3) a same-round collision is won by
  notify; (4) all parked + no finite deadline + no open host op = a
  deterministic "futex deadlock" job error, never an endless pump.
- Timeouts: every negative `timeout_ns` means infinite (spec — not just -1);
  `timeout=0` parks and is resolved by the scheduler, the engine never
  invents result 2 itself. Wall-clock accuracy is explicitly not promised.
- Until T2 lands, the kernel's `classify_resumable` treats
  `Suspension::Atomic` as terminal (default-closed): a wait/notify guest ends
  cleanly instead of hanging.

## Alternatives & second opinions

- **Codex recommendation — hidden store-registered host-func bridge:** two
  private `Func` handlers on the Store; wait parks via the unchanged
  HostError path, notify calls its handler synchronously (no pump roundtrip).
  Smallest engine delta and notify stays cheap. Rejected because the atomic
  park would be indistinguishable from generic host suspends except by
  convention, fail-closed-ness depends on runtime handler registration, and
  the untyped `(i32 addr, i64 timeout)` payload drops the memory identity the
  queue key needs. Recorded as genuine dissent — it satisfies the hard
  requirements too.
- **Engine-internal futex table:** queue truth would live in vendor code
  while the clock stays outside — a split brain both reviews rejected.
- **Synchronous scheduler trait callback:** fastest notify; kept open as a
  later additive optimization if profiling shows the notify suspension
  roundtrip matters (contended path only). Not now: reentrancy contract +
  second continuation mechanism.
- **Module rewrite to host imports:** re-indexes a ~100 MB module and breaks
  the provenance hash chain. Rejected outright.

## Consequences

- One continuation mechanism for everything; suspensions are inspectable and
  typed; conformance tests can drive park/resume fully on the host.
- `ResumableInvocation` accessors become `Option`-shaped — a controlled break
  with verified blast radius (2 kernel lines, 3 engine sites).
- Each notify on the contended path costs one executor exit/reenter.
- The T2 thread cap must be ≥ ~40 (measured rustc need: 26-32 guest threads;
  the plan's "e.g. max 8" example is superseded).
- Package split (sequential, shared files): T1-d-1 opcode surface (M),
  T1-d-2 suspension core (L, high-risk, review mandatory), T1-d-3
  conformance + determinism trace (M). T2 additionally needs a non-terminal
  fuel-quantum yield — instruction-level OutOfFuel is terminal today.
