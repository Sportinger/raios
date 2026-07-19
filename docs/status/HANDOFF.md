# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~12:35, loop running)

**rustc --version COMPLETES on-device: `rustc 1.83.0-dev`, exit 0, 11
rounds** (4716732, shadow-20260719-122823, quick+needle 504/504). Today's
chain: fuel starvation diagnosed+fixed (e3962b0) → LLVM OOM captured via
new stderr/trap evidence → cause was the visible reserve-to-max grow →
fixed + grow evidence. Full details STATUS. Also landed today: dual quick
escape needles, floor doc, unsafe inventory, cargo-JSON-diag (§4 top-level
box), 114028 forensics, RUSTCLOCK tracer. Tree clean, pushed.

## Next step

Next rustc slice: a REAL compile on-device (hello.rs via /src mount, new
argv, output via /out) → closes scope/06 "rustc-as-Wasm compiles a real
program". Queued disjoint lanes (task list): §4 device-graph IRQ fields, §2
storage negative, §3 rollback-isolation; vendored ByteBuffer
max-reservation lane for multi-GB compiles. Owner items (not blocking):
(1) §5/§6 wording still pre-ADR-0005 — same reframe approval as §1–3;
(2) bare-metal escape-test run needs a Surface session; (3) unattended-loop
hardware = money/owner.

## Recently (exactly 3, newest first)

### 2026-07-19 — rustc --version completes inside raiOS
After the escrow top-up fix, stderr capture revealed 'LLVM ERROR: out of
memory': prepare_rustcrun pre-grew the guest to max, so its first allocator
grow was denied. Guest now keeps 399 initial pages; grows 399→401 approved;
`RAIOS_RUSTCSTDOUT text=rustc 1.83.0-dev.`, exit 0 (4716732).

### 2026-07-19 — Starvation fix verified: rustc executes for real
E3 top-up fix + conformance starvation test; decisive rerun: 4 rounds to
real stderr I/O + trap vs 200k dead rounds before. The on-device compiler
now runs and fails ordinarily; stderr capture (E4) is the next evidence.

### 2026-07-19 — Fuel starvation measured as THE rustc-init root cause
RUSTCLOCK (generic opt-in wasmi trace, disabled-path equality proven): 5000
rounds all park at __lock ip=0 with unmet debit 208; H1/H2 refuted, H3
confirmed as escrow-grant-only-on-empty. rustc is starved, not spinning;
"advancing rounds" heartbeats were parks. Fix: per-activation top-up (E3).
