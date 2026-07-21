# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~12:15, root orchestrator active)

H22 hardware-proved commit `c787320`: the old construction-time
`0xD1030001` RX-pointer failure is absent, HID remains usable, and Wi-Fi reaches
the third connection command. The returned stick retained three valid chained
RECLOG frames with a clean zero tail and USB `errors=0`. Its terminal frame at
75.789 seconds is `phase=Associate(5)`, `status=CommandTimeout(100)`, register
`MarvellHostInterruptStatus=0`; the screen also showed PCI Command
`0x0402>0x0406` and firmware `0xfedcba00`. Supplicant profile and PMK commands
therefore completed; `ASSOCIATE_CMD 0x0012` published but no `CMD_DONE` arrived.

Two independent read-only reviews selected diagnostic-first. Commit `f77ca05`
adds one secret-free association-timeout fingerprint after verified terminal
quiesce, while preserving `CMD_DONE` as the sole success predicate and the
one-shot terminal host-status step. Focused tests, mutation negatives,
extractor selftest, release build, and final read-only ACCEPT are green. Code is
pushed on `main`. No Wi-Fi, traffic, domain, IOMMU, or isolation checkbox closes.

## Next step

Package and write H23 from `f77ca05`, verify both slots and readback, then perform
exactly one cold Surface boot and start Wi-Fi once. Do not use same-boot Retry
after quarantine. On failure, power down and extract RECLOG before any rewrite.

## Recently (exactly 3, newest first)

### 2026-07-21 — `f77ca05` adds the H23 timeout fingerprint
The bounded trace distinguishes untouched, expected, mismatched, or unavailable
response headers only after verified cleanup; no secret-bearing bytes persist.

### 2026-07-21 — H22 reaches physical association
Profile and PMK completed; `ASSOCIATE_CMD 0x0012` timed out without `CMD_DONE`.

### 2026-07-21 — `c787320` is hardware-proven
The premature RX-pointer failure disappeared and HID remained usable.
