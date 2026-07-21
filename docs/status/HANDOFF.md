# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~13:10, root orchestrator active)

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
extractor selftest, release build, and final read-only ACCEPT are green.

H23 is written to serial-pinned SanDisk `0101d57ec458c24f1b93`. Kernel SHA-256
is `6834c6a320549d378b8e19b3c64d10bed00388ac12536d6af4d4b99d2d31a537`;
firmware remains `cf4f51f41bd7ef4d7fe65fb76b8a2a0897bc70a0742bc4aea13d93b03fffd03a`.
A/B ESP, Boot Control, SEED_DATA, empty RECLOG, and writer readback are green.
Transcript: `%TEMP%\raios-h23-usb-write-f77ca05.transcript.log`. No scope box
closes until the physical H23 result exists.

## Next step

Perform exactly one cold Surface boot and start Wi-Fi once. Do not use same-boot
Retry after quarantine. Record the first visible result. On failure, power down,
return the stick, and extract RECLOG before any rewrite.

## Recently (exactly 3, newest first)

### 2026-07-21 — H23 persistent Surface stick prepared
Kernel-bound policy, pinned firmware, A/B layout, SEED_DATA, and readback green.

### 2026-07-21 — `f77ca05` adds the H23 timeout fingerprint
The bounded trace records header class only after verified terminal cleanup.

### 2026-07-21 — H22 reaches physical association
Profile and PMK completed; `ASSOCIATE_CMD 0x0012` timed out without `CMD_DONE`.
