# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` or the fitting plan.
> Max ~4 lines per entry; file max 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~23:50, loop running)

ADR-0023 Slices 1–3 are on origin/main. Durable typed grant/revoke events now
fold at boot into the real env.counter_get call gate; focused grant-reboot is
29/29 (replay + semantic link tamper), quick is 510/510. §2 explicit typed
grant/revoke, typed records, next-call durable revoke, and the whole narrow
floor-contract group are green. §4 owned-device register maps are green.
Foreign work remains untouched: AGENTS.md, release/diagnostics/, fixture lock.

## Next step

ADR-0023 Slice 4: rollback computes live-grants-now minus live-grants-at-target,
durably appends one revoke per delta before rebuilding instances, and proves a
rollback to fewer grants denies the removed surface with zero effect while a
retained surface and peer domain still work. Then check §3 rollback-delta.
Slice 5 remains an exclusive migration of all remaining host surfaces.

## Recently (exactly 3, newest first)

### 2026-07-19 — durable revoke reached the real reboot gate
Initial 211/211 evidence was rejected as selftest-tautological; repair moved
projection consumption into Envelope construction. After 3 NET-8 setup stalls,
strategy changed to a network-free focused harness: 29/29, review ACCEPT.

### 2026-07-19 — Genesis floor contract became mechanically closed
Machine-readable v1 descriptor matches raios-core's exact 5+30 imports/digest;
15/15 tests distinguish undeclared imports from kernel-internal/non-wire types.
Docs mapping gained a 14/14 red path; top-level floor box is green.

### 2026-07-19 — hardware metadata moved out of Rust prose
Four validated register maps cover 71 owned-device registers/constants with
10/10 malformed-map negatives. Versioned QEMU/Surface manifests and 12/12
drift negatives landed; manifest box stays open pending Surface CPU/RAM capture.
