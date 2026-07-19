# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~08:20, loop running)

The real rustc EXECUTES inside raiOS (factory-compile strand parked for an
owner priority call — see Next). Re-engaged the wider SCOPE: checked 6
genuinely-evidenced foundation boxes (§3 report pipeline + rollback, §4
harness + gated reports + doc-size predicates, 9a522c1). ISO lane in flight:
the day-1 Wasm-isolation escape negative test (guest OOB → trap + log, zero
host effect). Tree clean, all pushed.

## Next step — TWO owner decisions gate "every box checked"

1. **SCOPE §1–3 architecture mismatch (biggest blocker).** ~40 open boxes in
   §1 (per-domain MMU, userspace driver domains), §2 (hardware-capability
   granularity), §3 (MMU escape tests) describe a microkernel that ADR 0005 +
   ADR 0015 (owner-approved) deliberately REPLACED with Wasm-sandbox isolation
   on a monolithic kernel. They are "unbuilt by design". Owner must decide:
   reword the §1–3 top-level boxes to the Wasm-isolation framing (achievable),
   or keep the microkernel target (those boxes stay long-term open). Top-level
   SCOPE changes need owner approval.
2. **Factory-compile.** rustc runs but busy-spins in earliest std/libc init
   (zero WASI, zero atomic.wait, pre-args, pre-threads); needs interpreter
   PC-sampling/wasmtime-diff + the AOT speed stage. Deep std-init work vs
   steer per "vision = loop not features".

Buildable meanwhile (no owner input): ISO escape test (running), §4 JSON
compiler diagnostics in the lane cycle, §1 unsafe inventory, §4 device-graph
predicate.

## Recently (exactly 3, newest first)

### 2026-07-19 — Foundation boxes checked; architecture mismatch surfaced
6 evidenced boxes closed (9a522c1). An assessment showed most §1–3 open boxes
describe the pre-ADR-0005 microkernel, not the built Wasm-isolation system —
escalated for owner rewording. ISO escape-test lane dispatched.

### 2026-07-19 — Real rustc executes on-device (RB + RS)
Compiler runs on the merged pump with mounted sysroot; resumable-start seam
(afbfba8) clears the threads atomic barrier; _start executes (rounds
advance). Frontier: a std-init busy-spin + AOT speed.

### 2026-07-19 — WASI world married to the thread pump (ADR 0022)
One store, shared instance, per-thread fuel escrow (vendored swap, 19fde8e),
effect digest. Multi-thread fixture double-run trace+effect equal (6e3886a).
