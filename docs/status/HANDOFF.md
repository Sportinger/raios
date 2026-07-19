# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~11:00, loop running)

4 lanes landed+pushed this iteration: 114028 identified as musl __lock — a
single-threaded __lock CANNOT legally spin, so the rustc spin is a proven
host defect (H1 CAS/backing, H2 phantom need_locks, H3 fuel replay; 0d69f83);
quick now carries BOTH day-1 escape needles (503/503, 7ec3ae3); unsafe
inventory 389 sites/4 tagged (8d87c31); genesis-layer floor doc cited from
code (4c5baf1). Running: Lane E RUSTCLOCK instrumentation (H1/H2/H3
discriminator, 64-round cmpxchg trace in fn 114028), Lane F cargo-JSON-diag.

## Next step

Collect E → orchestrator builds + runs wasi.rustclock on the combined image
(C:\Users\admin\raios-artifacts\rustc-wasm\seeded\persist-combined.img, quick
+ appended command, per shadow-20260719-095302) → verdict picks the fix lane.
Collect F → verify fixture proofs → §4 JSON-diag box. Owner items (not
blocking): (1) §5 drivers-as-domains + §6 "Cranelift" wording still
pre-ADR-0005 — same reframe approval as §1–3; (2) bare-metal escape-test run
needs a Surface session; (3) unattended-loop hardware = money/owner.

## Recently (exactly 3, newest first)

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

### 2026-07-19 — Foundation boxes checked; architecture mismatch surfaced
6 evidenced boxes closed (9a522c1). An assessment showed most §1–3 open boxes
describe the pre-ADR-0005 microkernel, not the built Wasm-isolation system —
escalated for owner rewording. ISO escape-test lane dispatched.
