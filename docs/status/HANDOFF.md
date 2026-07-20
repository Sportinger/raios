# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~21:15, root orchestrator active)

GitHub split complete: full product on `main` (restore `4557600`), H20 on
`product/h20-surface` (`29f14f9`), and deployed site on `website` (`946f046`).
Owner's dirty worktree remains untouched.

Host tests and release-kernel CI are green. Shadow VM repeats the pre-existing
`09751a7` failure: `.cargo-home` is absent before `Resolve-Path`.

Cold Surface runs proved Genesis, composite HID, firmware, live scan, WPA2,
and masked passphrase entry. H16-H19 narrowed failure from zero response DMA
through mailbox/interrupt loss to valid PCI identity with MMIO already
all-ones before PCI readback became `ffff`.
Association, traffic, domain isolation, and IOMMU containment remain unproven.

H20 aligned init to `PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL`.
At `Starting WiFi`, both HID devices froze and no WiFi trace appeared. ADR 0034
invokes the full brake: no further Marvell ring tests or checkbox closure.

RECLOG held one valid USB `boot_probe` (`errors=0`, `recoveries=0`) and no WiFi
failure trace; this does not prove USB remained healthy at freeze time.

## Next step

Restore the shadow-VM bootstrap predicate, then review Marvell DMA read-only
and add negatives for bounds, ownership, non-overlap, indices, lifetimes, and
xHCI/kernel/heap/RECLOG separation. Only independent acceptance plus owner
authorization permits one cold-boot retest.

## Recently (exactly 3, newest first)

### 2026-07-20 — branch split pushed to GitHub
Full product is on `main`; the deployed site remains isolated on `website`.

### 2026-07-20 — H20 product code and evidence secured
Firmware `1f061b1`, WiFi `fc26cd5`, and docs `29f14f9` are preserved.

### 2026-07-20 — H20 full brake
Simultaneous HID loss at `Starting WiFi`; cause unproven. ADR 0034 records stop.
