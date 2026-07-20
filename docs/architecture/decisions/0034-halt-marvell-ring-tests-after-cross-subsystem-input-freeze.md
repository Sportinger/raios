# 0034 — Halt Marvell ring tests after a cross-subsystem input freeze

Date: 2026-07-20 · Status: active

## Context

Surface Pro 4 cold boots on 2026-07-20 proved the direct kernel path through
composite USB hub keyboard and mouse, Marvell 88W8897 firmware readiness, a live
2.4-GHz scan, selection of a WPA2-PSK/CCMP BSS, and masked physical passphrase
entry. They did not prove association, `PORT_RELEASE`, DHCP, traffic, a driver
domain, or IOMMU containment.

The connection experiments produced progressively narrower observations:

- H16 reached `PCIE_DESC_DETAILS` command completion, but the response DMA was
  zero and rejected as `command_response_failed` / `bad_length`.
- H17 reported a mailbox-register readback mismatch: interrupt status remained
  zero, command scratch readback was zero, and response scratch readback was
  all ones. A reboot was required.
- H18 stopped earlier with `host_int_status_unavailable`; no valid interrupt
  sample was obtained.
- H19 read the expected PCI identity `0x2b3811ab`, command `0x0406`, and an
  already-unavailable pre-operation MMIO status of `0xffffffff`; later PCI
  command readback was `0xffff`. This did not prove that enabling bus mastering
  caused the loss: memory-space and bus-master bits were already set in
  `0x0406`, and MMIO was unavailable before the helper ran.

Earlier independent read-only Codex reviews covered the H17-H19 transport and
initialization evidence. They supported hardening PCI command access and testing
the documented Marvell initialization order, but did not establish a root
cause.

The H20 image used checked command-word access and moved ring preparation plus
`PCIE_DESC_DETAILS → FUNC_INIT → GET_HW_SPEC → MAC_CONTROL` ahead of scan. The
Surface again reached the scanned-network and WPA2-entry path. After the UI
entered `Starting WiFi`, keyboard and mouse stopped responding together and no
WiFi failure trace became visible.

Read-only RECLOG extraction after shutdown found one valid
`raios.usb_diag.v0` `boot_probe` record with `errors=0` and `recoveries=0`. It
found no `raios.hw_failure_trace.v0` record. That proves neither USB health at
freeze time nor a Marvell cause; it proves only what had persisted before the
machine stopped responding.

## Decision

The simultaneous cross-subsystem input loss activates the AGENTS.md
isolation-suspicion protocol. All further Marvell ring hardware tests stop.
No WiFi, common RECLOG-diagnostic, isolation, or driver-domain checkbox closes
from H14-H20.

The full brake remains until read-only review and host-side negative predicates
settle the complete DMA ring map and ownership boundary: allocation sizes and
alignment, physical ranges, descriptor counts and strides, index/wrap bounds,
pairwise non-overlap, mailbox/response buffers, device-visible lifetimes, and
non-overlap with xHCI, kernel, heap, and RECLOG staging memory. The review must
also show that failure and retry cannot publish stale or foreign addresses.

The absence of enabled VT-d translation remains explicit. A host-side proof can
reject known-bad layouts, but it cannot claim hardware DMA containment. If a
future hardware run is authorized after the negative predicates and review are
accepted, it must begin from a cold boot. Warm retry, automatic rerun, and
additional ring experimentation remain forbidden by this decision.

## Alternatives, uncertainty, and opinions

Continuing because firmware upload and scan succeeded is rejected: those facts
do not bound later device DMA. Treating the freeze as proof of a Marvell DMA
escape is also rejected. DMA corruption, xHCI failure, polling starvation,
interrupt state, and unrelated input-path faults remain hypotheses, not facts.

The orchestrator invoked the full brake from the observed cross-subsystem
effect. The earlier independent Codex opinions addressed transport and init
ordering only; no unfinished review is represented here as an accepted
opinion. Claude was unavailable and also forbidden by the owner's Codex-only
instruction, so no Claude opinion or tooling was used.

## Consequences

H14-H20 remain useful bounded evidence: physical boot, composite HID, firmware
upload, scan, WPA2 input, and increasingly precise connection diagnostics are
real. They do not establish safe connection or traffic. Work may continue on
read-only analysis and negative predicates, while hardware Marvell ring work
is parked until the full-brake release conditions are met.
