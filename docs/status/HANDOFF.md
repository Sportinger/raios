# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~06:40, loop running)

Main road = on-device factory. **The real rustc compiler EXECUTES inside
raiOS.** Full pipeline live: store → exact-30 gate → sysroot mounted →
instantiate → start section (via the vendored resumable-start seam,
afbfba8) → _start running real rustc bytecode on the merged pump. Proven
by a deterministic round counter climbing 6944→351968 (sustained; fuel
metering forces a suspension per quantum, so advancing = forward
execution). Tree clean, all pushed.

## Next step — OWNER PRIORITY CALL (factory-compile strand parked)

Fully diagnosed: rustc busy-spins in the EARLIEST std/libc init — before
args (args_sizes_get=0), before threads (spawns=0), zero WASI calls
(fcb2820), zero atomic.wait, one __wasm_init_memory notify (1be0495). So a
pure busy-spin-loop on a value no worker will change. Pinpointing it needs
interpreter PC-sampling or a wasmtime execution differential (heavier than
serial tokens) — likely a wasi-libc main-thread/TLS-registration expectation
our green-thread main doesn't satisfy. BEHIND it: even fixed, a real compile
is hours under the TCG interpreter → the AOT execution stage (roadmap Stufe
4, "deliberate later ADR") is required for a practical on-device compile.
Owner decision needed: (a) push the deep std-init debug + AOT now, or (b)
bank the "rustc executes on-device" milestone and steer elsewhere per
"vision = loop not features". Everything up to _start execution is proven &
committed. Other open: SCOPE §6 Cranelift wording; ADR 0017 veto.

## Recently (exactly 3, newest first)

### 2026-07-19 — Real rustc executes on-device (RB + RS)
Compiler runs on the merged pump with mounted sysroot; resumable-start seam
(start_split, afbfba8) clears the threads atomic barrier; _start executes
(rounds advance). Frontier: a preview1-semantic init spin + AOT speed.

### 2026-07-19 — WASI world married to the thread pump (ADR 0022, MP+FS)
One store, shared instance, queue-then-materialize spawn, per-thread fuel
escrow via a vendored raw-remaining swap (19fde8e), WASI-effect digest.
Multi-thread fixture double-run trace+effect equal (6e3886a, 502/502).

### 2026-07-19 — Real rustc compiler loads + instantiates (CL, MM)
91-MB module reassembled from CAS (sha c6dccf3e), parsed, authorized,
instantiated (27fa7f6). Unblocked by the idempotent-MMIO fix (86fe9b9).
