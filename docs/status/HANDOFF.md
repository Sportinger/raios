# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~11:35, loop running)

THE rustc "spin" IS SOLVED IN DIAGNOSIS: RUSTCLOCK measured cas_total=0,
parks=5000/5000 at __lock entry (debit=208, G=0) — pump FUEL STARVATION
(sweep returns remainder 1..207, grant only fires on escrow==0; 22ded66,
STATUS has the full chain). Fix lane E3 in flight (top-up-to-quantum per
activation). Also landed: RUSTCLOCK instrumentation, cargo-JSON-diag (§4
top-level box checked, bd9c409/723af04), floor doc, unsafe inventory, dual
quick needles. Tree clean, pushed.

## Next step

Collect E3 → I build + conformance + quick-regression + combined-image rerun:
positive = rustc makes WASI calls / stdout after the fix (measurement recipe:
quick + wasi.rustclock needle + -PersistDiskPath persist-combined.img
-GuestMemoryMB 8192 -KeepImage; harness deletes run dirs on pass otherwise).
Then queued lanes (task list): §4 device-graph IRQ fields, §2 storage
negative, §3 rollback-isolation. Owner items (not blocking): (1) §5/§6
wording still pre-ADR-0005 — same reframe approval as §1–3; (2) bare-metal
escape-test run needs a Surface session; (3) unattended-loop hardware.

## Recently (exactly 3, newest first)

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

### 2026-07-19 — Reframe landed; ISO boxes earned; spin pinpointed
§1–3 rewritten to the built Wasm-isolation architecture (owner-approved). OOB
escape negative test green as permanent quick needle (isolation.selftest,
502/502) + import-deny evidence verified → §1/§2/§3 boxes checked. RUSTCPC
profile: 98% of samples in fn 114028, directly before the thread-spawn caller.
