# Kernel Layout-Bug Hunt — Slice Plan (2026-07-12)

The dedicated plan for finding and fixing the latent layout-sensitive
kernel memory-safety defect that blocks the mass-refactor program. The
evidence dossier lives at the top of `docs/PROJECT_STATUS.md`; this file
is the executable slice plan a session (or `/goal`) can reference.

## Goal (exit criteria — all three)

1. Root cause identified and stated as a one-paragraph mechanism with the
   offending write site named (file:line).
2. Fix landed on main with the fix's own regression evidence: the
   `memory-durable` profile is GREEN on main AND on at least two of the
   previously red layouts (apply the P1-A stash `p1-kernel-attribution-test`
   and/or branch `refactor/p2-wave1-parked` on top of the fix; both are
   sub-10-minute oracles).
3. Classification updated in `docs/PROJECT_STATUS.md`; the parked work is
   explicitly UNBLOCKED there (landing it is the NEXT slice, not this one).

Suggested `/goal` one-liner:
`Layout-Bug per docs/plan-reviews/kernel-layout-bug-hunt-plan-2026-07-12.md
gefunden & gefixt: memory-durable gruen auf main + p2-wave1-parked-Layout,
Ursache+Fix committet, PROJECT_STATUS aktualisiert`

## Dataset (already reproduced, 2026-07-12)

| Layout | memory-durable result |
| --- | --- |
| main (green baseline, `shadow-20260712-220828-26452.json`) | green |
| main + P1-A loader split (stash `p1-kernel-attribution-test`) | freezes probe 6 (wasm-import-grant) |
| main + P2 wave (branch `refactor/p2-wave1-parked`, 29c56eb) | freezes probe 4 (broker, `shadow-20260712-214657-20004`) |
| main + core-modules-only (reverted commits 976e776/9b514aa) | freezes probe 7 (export-packet) while 4 and 6 pass |

Freeze signature: the child receives a truncated `agent memory.record_log_appen`
command line, consumes no further serial input, prints nothing more. The
victim probe is identified by the SMALLEST `serial-memrecord-*.log` in the
failed run's `%TEMP%\raios-shadow-*` scratch dir — never by the throw
message alone. Ruled out by controlled probes: content differences (token
audit), codegen-units partitioning (red at cgu=1), Limine main-stack size
(red at 4 MiB), environment (bisection-controlled).

## Suspect region

Child probes differ only in their persisted fixture disk contents; the
frozen probes (broker, wasm-import-grant, export-packet) all parse
persisted regions during child boot. Prime suspect: an out-of-bounds write
in or around the boot-time persist-region/record parsing that lands in
layout-adjacent `.bss`/`.data`, silently corrupting serial-input state
when the layout happens to place it next door.

## Probe plan (in order; 1 and 2 are static, no QEMU)

1. **Symbol-map diff.** Build the four layouts; dump `.bss`/`.data` symbol
   maps (nm/objdump on `target\x86_64-seed\release\seed-kernel`); for each
   red layout, list what sits immediately around the serial/input statics
   and around the persist-parsing buffers; intersect across layouts. The
   moved-victim pattern should point at the clobber source.
2. **Bound the write site statically.** Audit the suspect parsing paths
   for fixed-size buffers, index arithmetic, and `unsafe` blocks
   (`ptr::copy`, slice casts) reachable during child boot with the
   broker/import-grant/export fixtures.
3. **Canary statics.** Bracket the top suspects with guard statics filled
   with sentinel patterns; boot the red layout; report which canary is
   dirty and with what bytes (the payload usually names its writer).
4. **Instrumented serial consumer.** If still ambiguous: serial-print the
   input-state struct address and contents at first corruption detection.

## Rules for the slice

- Red Gate discipline: main stays green; all experiments run on probe
  branches/stashes or are reverted before session end.
- Every red/green run gets its layout named in the report classification.
- The fix must be a real memory-safety fix, not a layout tweak that
  re-hides the bug (re-hiding = explicitly forbidden outcome).
- After the fix: land the parked harvest in order (P1-A stash, P2 wave
  branch, then P3 pending owner confirmation) — each with its own focused
  + full evidence, as the refactor plan already specifies.
