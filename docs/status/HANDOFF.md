# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, PCI gate accepted; K3 capture awaiting re-review)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`e926f06`. The detached old root at `09751a7` contains foreign WIP; never clean,
reset, merge, or integrate it.

H25 hardware-proves post-PMK mailbox liveness. ADR 0042 and the replacement
PCI BAR probe are accepted and pushed. The production predicate passes 21/21
tests, binds CF8 dword to CFC/CFE word writes in emitted code, rejects both
dword-data and swapped-association mutations, and passed two fresh independent
read-only reviews. Real partial-word behavior remains owner-boot evidence.

K3 is now the only dirty product slice: five files for same-boot Surface fact
capture through USB RECLOG. Its previous predicate/build were green, but its
old review was invalidated by the then-missing PCI proof. K3 remains unaccepted,
uncommitted, and unsafe for owner boot until fresh review and verification.

## Next step

Re-run the K3 predicate and freestanding release build against accepted PCI
commit `e926f06`. Dispatch two fresh independent read-only reviews over exactly
`docs/architecture/unsafe-inventory-baseline-v2.json`, `seed-kernel/src/main.rs`,
`seed-kernel/src/usb.rs`, `scripts/test-surface-fact-capture-kernel.ps1`, and
`seed-kernel/src/surface_fact_capture.rs`. Accept and secure only if the diff
matches ADR 0038/0039/0040/0042, captures before `usb::init()`, emits bounded
Wire V1 frames, and introduces no PCI/USB ownership escape. Then build/package
the capture image. Do not write the physical stick without an explicit final
disk-number confirmation from the owner.

## Recently (exactly 3, newest first)

### 2026-07-22 - Header-bounded PCI proof accepted
`e926f06`: 21/21 runtime tests, two transport mutation negatives, and fresh
R-G/R-H accepts; exact two-file slice committed and pushed.

### 2026-07-22 - PCI architecture rescope accepted
`6f27bcc`: ADR 0042 separates slot consumption from BAR usability and requires
header windows, fail-closed validation, exact restore, and emitted-code proof.

### 2026-07-22 - Initial PCI seam parked
`3f0213f`: two earlier implementation/review strategies were rejected; no
unsafe product diff was accepted.
