# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~22:50, root orchestrator active)

The H20 Surface observation remains unresolved: both HID devices froze at
`Starting WiFi`; the persisted RECLOG showed USB `errors=0` before the freeze
and no WiFi failure trace. H19/H20 also observed unreadable PCI config/MMIO,
without proving the cause or proving that bus mastering was quiesced.

Host-only recovery work is green. `36e8c12` restored the Shadow-VM Cargo-home
bootstrap and GitHub CI completed green. `bb46923` added the pure Marvell DMA
model for ring bounds, contiguity/non-overlap, epochs, and BME/publication
ordering; its 16 focused tests plus 710/5 core tests and GitHub CI are green.

`7428759` is accepted K1: one host DMA gate, device-pointer validation, one
`0xC05C` writer, removal of `poll_rx_ring`, gated triggers/jobs, and durable
mutation predicates. The release build is green and two independent final
Codex reviews returned ACCEPT. K1 has not run on the Surface and grants no
hardware-test release, association, traffic, driver-domain, or IOMMU claim.
The `docs/SCOPE.md:155` Marvell isolation checkbox remains open.

## Next step

K2: implement and independently verify BME-off with PCI command readback plus
complete command/response mailbox cleanup. A cleared BME readback is not proof
of DMA drain or IOMMU containment. After K2 acceptance, build the release
image, write the USB stick, and perform exactly one owner-authorized cold boot;
no warm retry or additional Surface experiment is authorized.

## Recently (exactly 3, newest first)

### 2026-07-20 — `7428759` K1 accepted twice
Host access is fail-closed after a latched pointer fault; hardware remains untested.

### 2026-07-20 — `bb46923` pure DMA model green
Ring-layout and publication-order predicates passed locally and in GitHub CI.

### 2026-07-20 — `36e8c12` Shadow-VM bootstrap restored
Cargo-home setup was repaired and the complete GitHub CI run passed.
