# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21 ~11:12, root orchestrator active)

H21 produced useful persistent evidence. The returned SanDisk contained three
valid chained RECLOG frames with a clean zero tail: USB `errors=0`, the durable
pre-BME checkpoint at PCI Command `0x0402`, and terminal
`MarvellPublicationStep=0xD1030001` at 42.757 seconds. This is the exact
`arm_rx_ring_while_gated` branch where the firmware-owned shared RX-WR/TX-RD
register `0xC08C` decoded as all ones before `PCIE_DESC_DETAILS`. The stick was
read only; GPT identity and SEED_DATA remained valid.

Upstream Linux initializes the 88W8897 RX write-pointer shadow to zero during
ring construction and first reads the device-owned register during real RX/TX
processing. Commit `c787320` implements only that narrow correction. Event
initialization, later raw `0xC08C` decoders, all-ones quarantine, BME/checkpoint,
DMA gate, reboot latch, pointer publication, and grants are unchanged.

Acceptance is green: both focused PowerShell suites and their mutations, 16/16
pointer tests, full seed-kernel host typecheck, real release build, and a fresh
read-only final review. Two earlier opinions agreed on the RX defect but
disagreed about also changing Event; ADR 0035 records the conservative RX-only
decision. No Wi-Fi, traffic, domain, IOMMU, or isolation checkbox closes.

The code and tests are committed and pushed on `main` at `c787320`. The SanDisk
still contains H21 and is not yet the H22 test image.

## Next step

Package `c787320` into the persistent A/B image, validate signed policy and
extractor, then serial-pin and write the SanDisk. Perform exactly one cold
Surface boot and start Wi-Fi once; no same-boot retry after quarantine. On any
failure, power down, return the stick, and extract RECLOG before rewriting it.

## Recently (exactly 3, newest first)

### 2026-07-21 — `c787320` defers the premature RX pointer read
RX construction matches upstream ownership; runtime all-ones rejection remains.

### 2026-07-21 — H21 persisted exact `0xD1030001`
USB stayed clean and the Marvell failure localized to pre-registration `0xC08C`.

### 2026-07-21 — `d8d8f34` exact data-ring cause accepted twice
Specific ring failures persist before fail-closed quarantine.
