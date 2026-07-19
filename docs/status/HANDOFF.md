# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~10:30, loop running)

Both 07-19 owner decisions are in execution: §1–3 Wasm reframe is committed;
rustc staged plan sits at "spin fix" — the PC profiler pinned the init spin to
function 114028 (200112/203891 samples, zero WASI calls, spawns=0). 4 lanes in
flight: (A) disassemble 114028 → fix hypothesis; (B) ungranted-import denial as
second permanent quick needle; (C) unsafe-inventory generator; (D) genesis-layer
floor doc. Tree clean, pushed.

## Next step

Collect lanes → verify → commit each. For B: orchestrator builds the kernel +
runs quick (worker cannot run rustc). Then check the §2/§3 boxes B feeds, and
dispatch §4 JSON-compiler-diagnostics next. Owner items (not blocking lanes):
(1) §5 drivers-as-domains + §6 "Cranelift" wording still carry pre-ADR-0005
microkernel framing — needs the same reframe approval as §1–3; (2) bare-metal
run of both escape tests needs a Surface session; (3) unattended-loop hardware
(smart plug, watchdog) = money/owner.

## Recently (exactly 3, newest first)

### 2026-07-19 — Reframe landed; ISO boxes earned; spin pinpointed
§1–3 rewritten to the built Wasm-isolation architecture (owner-approved). OOB
escape negative test green as permanent quick needle (isolation.selftest,
502/502) + import-deny evidence verified → §1/§2/§3 boxes checked. RUSTCPC
profile: 98% of samples in fn 114028, directly before the thread-spawn caller.

### 2026-07-19 — Foundation boxes checked; architecture mismatch surfaced
6 evidenced boxes closed (9a522c1). An assessment showed most §1–3 open boxes
describe the pre-ADR-0005 microkernel, not the built Wasm-isolation system —
escalated for owner rewording. ISO escape-test lane dispatched.

### 2026-07-19 — Real rustc executes on-device (RB + RS)
Compiler runs on the merged pump with mounted sysroot; resumable-start seam
(afbfba8) clears the threads atomic barrier; _start executes (rounds
advance). Frontier: a std-init busy-spin + AOT speed.
