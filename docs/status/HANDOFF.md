# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~12:10, loop running)

STARVATION FIX LANDED + PROVEN (e3962b0): escrow tops up to the quantum per
activation; rustc went from 200k zero-WASI parked rounds to REAL EXECUTION
in 4 rounds — full libc init, then fd_write ×3 to stderr, fd_filestat_get,
fd_seek, guest_trap (shadow-20260719-113709, quick+needle 504/504,
conformance 55/55 incl. new starvation test). rustc now fails like a normal
program; its stderr text is currently DISCARDED (wasi_preview1 write_fd
keeps only stdout content). Lane E4 in flight: retain+emit stderr + trap
detail. Tree clean, pushed.

## Next step

Collect E4 → build + combined-image rerun → READ rustc's error message →
next fix lane per message (likely argv/env/mount shape). Measurement recipe
in [KeepImage memory + STATUS]. Then queued: §4 device-graph IRQ fields, §2
storage negative, §3 rollback-isolation. Owner items (not blocking): (1)
§5/§6 wording still pre-ADR-0005 — same reframe approval as §1–3; (2)
bare-metal escape-test run needs a Surface session; (3) unattended-loop
hardware = money/owner.

## Recently (exactly 3, newest first)

### 2026-07-19 — Starvation fix verified: rustc executes for real
E3 top-up fix + conformance starvation test; decisive rerun: 4 rounds to
real stderr I/O + trap vs 200k dead rounds before. The on-device compiler
now runs and fails ordinarily; stderr capture (E4) is the next evidence.

### 2026-07-19 — Fuel starvation measured as THE rustc-init root cause
RUSTCLOCK (generic opt-in wasmi trace, disabled-path equality proven): 5000
rounds all park at __lock ip=0 with unmet debit 208; H1/H2 refuted, H3
confirmed as escrow-grant-only-on-empty. rustc is starved, not spinning;
"advancing rounds" heartbeats were parks. Fix: per-activation top-up (E3).

### 2026-07-19 — Spin = host defect; both escape needles permanent
Lane A proved fn 114028 is musl __lock via libc.a object match; BSS-zero
need_locks gate ⇒ legal single-thread path returns immediately ⇒ raiOS-side
defect, 3 falsifiable hypotheses + specced RUSTCLOCK discriminator. Quick
503/503 with the new ungranted-import needle (red-run negative proven).
