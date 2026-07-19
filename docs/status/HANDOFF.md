# HANDOFF — Where do we stand?

> **Format rule (hard):** This file is a window, not a log. Exactly ONE "Now"
> block, ONE "Next step", and exactly THREE "Recently" entries. On update:
> insert the new entry on top, delete the oldest WITHOUT replacement. What
> matters durably goes FIRST to `docs/status/STATUS.md` (evidence) or the
> fitting plan under `docs/plans/`. Max ~4 lines of text per entry, file max
> 60 lines. Replace, never append.

## Now (as of 2026-07-19 ~01:00, loop running)

Main road = on-device factory. **Bauplatz box CLOSED**: full 1-GiB window
live at the Surface-RAM profile (`pages_max=16384`, pinned needle,
shadow-20260719-005556) after ADR 0021 bulk-fuel parking (98b2955),
doubling-aware grow limiter + 4-GiB heap cap (15331a3). Slice-6 and T2
closed earlier today. Quick profile carries wasi/threads/memselftest
needles permanently (501/501). No lane live right now.

## Next step

**Sysroot import — one fix from green.** Route decided + built: BuildFS
seeded offline into ARTSTOR (real 71-MB image on C:, 1161 chunks), kernel
reads via the granted per-read-rehashed session. Manifest loads live;
end-to-end blocked on ONE diagnosed defect (S2 in flight): materialize
rescans ARTSTOR per chunk (O(n^2), ~44 GB reads) → AHCI read returns no
bytes at chunk 2 (instrumented: detail=io at=2). Fix = single-pass index.
Then compiler artifact (91 MB) same route, then hello.rs double-build =
W5 factory proof. Owner questions open: SCOPE §6 Cranelift wording; ADR
0017 veto window. Owner forward plans (b2eb324): socket → GPU → installer.

## Recently (exactly 3, newest first)

### 2026-07-19 — Bauplatz closed via fuel parking + honest memory model
Three QEMU failures → host repro (3248408) proved wasmi's dynamic bulk
fuel charge terminal; ADR 0021 (fact-checked twin reviews) → park-before-
charge in the vendored engine, 41/41 conformance incl. pacing invariance;
Vec-doubling limiter + 4-GiB heap after a live OOM panic. Full window
proven both profiles.

### 2026-07-18 — Slice-6 closed, T2 closed, sysroot pinned (iteration 10)
Storage authority two-stage per ADR 0020 (dissent recorded): granted
per-read-rehashed chunk reads, pre-I/O commit gate, single-use write
handle. Futex deadlock → deterministic JobDeadlocked live. Docs hygiene
10/11 self-tested. Sysroot BuildFS pin 13daf6f9.

### 2026-07-18 — Slice-6 stage A: kernel WASI gate live (G1a)
Module bytes → ordered import extraction → authorize → exact-30 linker →
checked guest memory → runner; extra-import fixture denied
pre-instantiation. QEMU quick with permanent needles (4e17c10).
