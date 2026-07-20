# 0034 — Halt Marvell ring tests after a cross-subsystem input freeze

Date: 2026-07-20 · Status: active

## Context

Surface Pro 4 cold boots on 2026-07-20 proved the direct kernel path through
composite USB hub keyboard and mouse, Marvell 88W8897 firmware readiness, a live
2.4-GHz scan, WPA2-PSK/CCMP selection, and masked physical passphrase entry.
They did not prove association, `PORT_RELEASE`, DHCP, traffic, a driver domain,
or IOMMU containment.

H16-H19 narrowed the observed connection failure from a zero response DMA to
mailbox/interrupt loss and then to valid PCI identity with MMIO already reading
all ones before PCI command readback became `0xffff`. H20 used checked PCI
command access and ordered
`PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL` before scan. At
`Starting WiFi`, keyboard and mouse stopped responding together and no WiFi
failure trace became visible.

Read-only RECLOG extraction after shutdown found one valid
`raios.usb_diag.v0` `boot_probe` record with `errors=0` and `recoveries=0`, and
no `raios.hw_failure_trace.v0` record. This proves only the state persisted
before the freeze. It does not prove USB health at freeze time, a Marvell DMA
escape, or any other H20 root cause.

Subsequent work was host-only. Commit `bb46923` established a pure model for
ring bounds, physical contiguity and pairwise non-overlap, epoch ownership, and
BME/pointer publication ordering; its focused and core tests and GitHub CI were
green. Commit `7428759` implemented K1 and passed a release build plus two
independent final read-only Codex reviews. Neither commit was tested on the
Surface.

## Decision

The Marvell hardware full brake remains active. No further ring, association,
or traffic experiment is authorized yet, and no WiFi, isolation, common
RECLOG-diagnostic, driver-domain, or IOMMU checkbox closes from H14-H20 or K1.

K1 is accepted only as a host-side fail-closed boundary:

- all host DMA-buffer work, Marvell MMIO pointer writes, doorbells, and
  bus-master-enable paths covered by K1 serialize through one DMA gate;
- complete device pointer fields are validated before descriptor use or host
  pointer publication;
- `0xC05C` has one combined RX/TX writer and there is no competing
  `poll_rx_ring` consumer; and
- an invalid ring pointer latches reboot-required and prevents subsequent
  gated host access, trigger, or job publication in that boot.

K1 does not read back a cleared PCI Bus Master Enable bit when an invalid
pointer is detected. It therefore does not establish that the device stopped
autonomous DMA after the latch. K2 must provide verified BME-off/readback and
complete command/response mailbox cleanup before this ADR permits another
hardware run.

After K2 predicates, build, and independent review are accepted, the next
hardware action is a fresh release image and USB stick followed by exactly one
owner-authorized cold boot. Warm retry, automatic rerun, and additional ring
experimentation remain forbidden.

## Alternatives, uncertainty, and opinions

Continuing because firmware upload and scan succeeded is rejected: those facts
do not bound later device DMA. Treating the freeze as proof of a Marvell DMA
escape is also rejected. DMA corruption, xHCI failure, polling starvation,
interrupt state, PCI function loss, and unrelated input-path faults remain
hypotheses.

A successful BME-clear readback would prove only the observed PCI command-bit
state. It would not prove that outstanding DMA drained, that the device honored
the transition promptly, or that an IOMMU contained prior or future DMA. VT-d
translation and a foreign/out-of-range DMA negative remain separate scope work.

The two final K1 reviews agreed on the host boundary above. Their acceptance is
not a Surface safety result and does not resolve the H20 root cause.

## Consequences

H14-H20 remain bounded evidence for physical boot, composite HID, firmware
upload, scan, WPA2 input, and connection diagnostics. K1 permits continued
host-side hardening without permitting hardware execution. K2 is the sole next
Marvell safety slice; the isolation checkbox remains open until domain
execution, an owned IOMMU-fenced DMA region, connection, traffic, and the other
documented acceptance conditions are actually proven.
