# 0036 — Observe the associate doorbell acknowledgement before changing HostCmd

Date: 2026-07-21 · Status: active

## Kontext

H23 proved that the Surface publishes an internally consistent association
header for command `0x0012`, length 132, after Profile and PMK completed. At the
60.857-second timeout the host interrupt status was zero; after verified
terminal quiesce and cleanup the response header was still all zero. This proves
neither firmware consumption of the doorbell nor a malformed association
payload. Extending the timeout or treating a response header as completion
would weaken existing fail-closed boundaries without explaining the failure.

## Entscheidung

H24 preserves the H23 connection flow and completion contract. On the real
Associate timeout, before quarantine clears the mailbox, it reads
`PCIE_CPU_INT_STATUS` exactly once and persists only whether
`CPU_INTR_DOOR_BELL` is cleared, remains set, or the register is unavailable.
It does not inspect payload bytes, extend the timeout, add a completion path, or
read response DMA before verified cleanup.

A still-set bit accepts the narrow hypothesis that firmware did not acknowledge
the current Associate doorbell. A cleared bit rejects that hypothesis and moves
the next slice to Associate-specific payload/BSS state. An unavailable register
is an invalid probe result, not evidence for either side.

## Alternativen & Zweitmeinungen

Two fresh independent read-only Codex reviews both returned `PROBE_REQUIRED`.
The PCIe/DMA review recommended this observation-only doorbell probe. The
HostCmd review instead proposed replacing Associate once with a post-PMK
`GET_HW_SPEC` liveness canary. That canary distinguishes a mailbox already dead
after PMK from an Associate-specific wedge, but it changes the command sequence
and no longer reproduces the H23 failure.

We choose the doorbell probe first because it adds one bounded observation to
the unchanged failing path. The canary remains the next independent experiment
if H24 proves that the doorbell was acknowledged but source evidence still does
not justify a concrete HostCmd repair.

## Folgen

One more cold boot can settle the firmware-notified boundary without changing
association behavior. H24 does not claim connection progress and closes no
driver, traffic, IOMMU, or isolation checkbox. Its trace value is secret-free
and remains subject to the existing one-shot RECLOG and reboot-quarantine rules.
