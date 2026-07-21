# 0035 — Defer the firmware-owned RX pointer read until runtime

Date: 2026-07-21 · Status: active

## Kontext

The H21 Surface boot left three valid RECLOG frames. USB reported zero errors,
the pre-BME checkpoint recorded PCI Command `0x0402`, and the terminal Marvell
step was `0xD1030001` at 42.757 seconds. The stable diagnostic map identifies
that value as an all-ones decode of the shared firmware-owned RX-WR/TX-RD
register at `0xC08C` while `arm_rx_ring_while_gated` was constructing the ring.

At that point raiOS had not yet sent `PCIE_DESC_DETAILS`. Upstream Linux
`mwifiex` initializes the 88W8897 RX write-pointer shadow to zero during ring
construction and first reads the firmware-owned register in actual RX/TX
runtime processing. The register offset itself agrees with upstream.

## Entscheidung

Ring construction does not observe `PCIE_RX_WR_PTR`. It initializes only the
local diagnostic shadow to zero, retains the host rollover pointer, validates
the same DMA translations, and builds the same descriptors. Real RX and TX
paths continue to read `0xC08C`, decode the raw value before any data use or
doorbell, and quarantine every decoder error including `0xFFFFFFFF`.

This slice changes only RX construction. Event construction remains unchanged
because its physical read succeeded and changing it is not necessary to cross
the observed H21 boundary. BME, durable checkpoint, DMA gate, reboot latch,
pointer publication, and network-grant rules remain unchanged.

## Alternativen & Zweitmeinungen

Two fresh independent read-only Codex reviews agreed that the initial RX read
is premature. One recommended removing both RX and Event construction-time
reads for full upstream symmetry. The other recommended the narrower RX-only
change because only `0xC08C` failed and an Event change would add an unneeded
hardware variable. We chose the narrower slice; Event can become a separate
evidence-driven change if a later boot identifies it.

Treating all ones as zero was rejected. The zero value is a software shadow
seed before observation, never a normalized MMIO result. Treating the marker
as proof of a dead BAR was also rejected because earlier firmware and HostCmd
MMIO transactions completed.

## Folgen

H22 can proceed past the exact H21 pre-registration read without weakening the
steady-state fail-closed boundary. A later `0xC08C` all-ones read may still
quarantine the transport; that would be a new, legitimate runtime-liveness
result. This decision proves neither connection nor traffic, and it does not
close the in-kernel driver, IOMMU, or isolation scope boxes.
