# 0021 — Bulk-operation fuel parks before charging; refills cap against granted totals

Date: 2026-07-18 · Status: active

## Context

Host reproduction (conformance test, 3248408) proved why the Bauplatz
memory selftest is red: the vendored wasmi charges dynamic per-byte fuel
for bulk operations (memory.grow/fill/copy/init and the table quartet),
and when that charge exceeds the residual quantum the engine traps
terminally — T1's resumable fuel only parks at block boundaries. The real
rustc workload performs large bulk operations routinely, so this is a
factory blocker, not a test nicety. Twin design opinions were taken; both
were fact-checked against the code before deciding.

## Decision

1. **Park before charge, retry on resume** (opt-in path only). The kernel
   runs wasmi in the default Lazy fuel mode, whose pre-effect check
   (`sufficient_fuel`) is pure — a failed check has mutated nothing and no
   limiter/host call has happened. Under `resumable_fuel(true)` only: when
   that check reports OutOfFuel at a dynamic-charge site, the executor
   restores the popped operands, parks via the existing
   `park_fuel_quantum` machinery (ip still at the bulk instruction), and
   yields the existing FuelQuantum suspension. Resume re-executes the
   whole instruction: recompute, recheck, and only when the banked
   residual covers the full charge does the operation execute — exactly
   once, effects and consume together. No new suspension state, no debt
   bookkeeping, no change to the default config path (byte-identity
   proofs stand). An engine-construction assert pins
   `resumable_fuel ⇒ Lazy` (in Eager mode the retry would double-charge).
2. **Banking with a granted-total ceiling.** Residual fuel banks
   monotonically (+Q per round while parked; a charge C with residual r
   completes after exactly ceil((C−r)/Q) suspensions). Because banking
   makes consumption lag grants, the refill loop MUST cap against the
   cumulative GRANTED total, not consumed fuel — otherwise retained
   residuals let total grants exceed `max_total_fuel`. The build pump
   gains a checked granted-total counter with the same terminal
   "fuel ceiling reached" behavior.
3. **Feasibility clamp.** Escrow only for operations that can actually
   execute: in the park path (never the default path) a cheap
   engine-internal precheck — ranges vs. current memory/table size,
   grow vs. declared max, never the host limiter — lets doomed operations
   run immediately into their real trap or −1 with zero fuel suspensions,
   instead of banking dozens of quanta for an operation that must fail.

Pinning tests (mandatory in the implementing lane): the memselftest repro
flips to ProcExit(0) with exact suspension counts AND total fuel consumed
equal to a large-quantum control run (pacing invariance — catches any
charge-on-park or double charge); charge > whole quantum completes in the
predicted round count; park-then-kill leaves target bytes unchanged and
no limiter entry; doomed-op fast-fail keeps limiter-log cardinality
unchanged; ceiling-shortfall terminates deterministically with no effect;
default-config freeze reproduces upstream OutOfFuel at consumed=942337;
double-run trace equality over a park-heavy workload.

## Alternatives & second opinions

- **Prepaid fuel debt carried in the continuation** (Codex): a typed
  FuelBulk suspension holding total/unpaid charge, tranche payment, and a
  recommendation to pin Eager mode. Rejected on a verified fact: the
  kernel's Lazy mode makes the pre-effect check pure, so retry semantics
  need no debt state at all and the double-charge hazard the debt design
  defends against does not exist; pinning Eager would create it. The
  smaller mechanism wins. Recorded as dissent.
- **Adopted from the same opinion**: the granted-total ceiling flaw
  (real under banking) and several negative tests (escrow-theft pump
  invariant for future multi-thread pumping, resume-integrity checks).
- **Raising the class quantum** (32 MiB proved the fixture host-side):
  rejected as a workaround — the residual before any bulk op can be
  arbitrarily small, so no fixed quantum removes the trap class.

## Consequences

Easier: the memselftest and every future bulk-heavy guest (rustc) pace
through quanta deterministically; kill-while-parked is trivially clean.
Harder: fuel-cost changes now shift park counts and therefore traces —
re-baselining is a procedure, not a bug; the T2 multi-thread pump must
one day enforce the bank-monotonicity invariant (pinned here, implemented
when T2 pumps multiple bulk-heavy threads). A parked-then-funded bulk
operation still executes physically indivisibly — the quantum stays a
determinism budget, not a latency bound.
