# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~20:56, root orchestrator active)

Product branch `product/h20-surface` is at `fc26cd5`; pinned firmware `1f061b1`
and fail-closed WiFi `fc26cd5` are secured. GitHub branch `website` at `946f046`
is live on Cloudflare. `origin/main` remains site-only `941ac39`; restore the
product line immediately after the docs commit.

Cold Surface runs proved Genesis, composite HID, Marvell firmware, live scan,
WPA2 selection, and masked passphrase entry. H16-H19 narrowed connection
failure from zero response DMA through mailbox/interrupt loss to valid PCI
identity with MMIO already all-ones before PCI readback became `ffff`.
Association, traffic, domain isolation, and IOMMU containment remain unproven.

H20 aligned init to `PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL`.
At `Starting WiFi`, both HID devices froze and no WiFi trace appeared. ADR 0034
invokes the full brake: no further Marvell ring tests or checkbox closure.

RECLOG held one valid USB `boot_probe` (`errors=0`, `recoveries=0`) and no WiFi
failure trace; this does not prove USB remained healthy at freeze time.

## Next step

Commit the exact docs, restore the product line to `origin/main`, then review
Marvell DMA read-only and add negatives for bounds, ownership, non-overlap,
indices, lifetimes, and xHCI/kernel/heap/RECLOG separation. Only independent
acceptance plus owner authorization permits one cold-boot retest.

## Recently (exactly 3, newest first)

### 2026-07-20 — H20 product code secured
Pinned firmware is `1f061b1`; fail-closed physical WiFi is `fc26cd5`.

### 2026-07-20 — website split deployed
GitHub `website` at `946f046` is live on Cloudflare production.

### 2026-07-20 — H20 full brake
Simultaneous HID loss at `Starting WiFi`; cause unproven. ADR 0034 records stop.
