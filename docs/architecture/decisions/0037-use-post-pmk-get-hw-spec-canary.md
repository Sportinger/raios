# 0037 — Use one post-PMK GET_HW_SPEC canary before changing Associate

Date: 2026-07-21 · Status: active

## Kontext

H24 preserved the failing Associate flow and hardware-proved that firmware
cleared `CPU_INTR_DOOR_BELL`. At 118.495 seconds the correct `0x0012` request
still had no `CMD_DONE`, host interrupt status was zero, and the response stayed
untouched after verified cleanup. Doorbell acknowledgement therefore excludes
a missed host notification but does not prove payload consumption or a live
post-PMK command engine.

## Entscheidung

H25 replaces Associate once, after successful Profile and PMK, with exactly one
read-only `GET_HW_SPEC` liveness canary. It uses the existing mailbox
publication, fresh-low completion baseline, timeout, and quarantine rules.
Returned hardware data is not applied. Only a current-epoch `CMD_DONE` with the
expected command/sequence may complete it; a doorbell acknowledgement, response
contents, or event activity never count as completion.

A valid completion localizes the next investigation to Associate request/BSS
state. A timeout with a cleared doorbell favors a generic post-PMK
mailbox/firmware stall. Wrong or stale completion, malformed response, a
still-set doorbell, or unavailable MMIO fail closed as distinct bounded results.

## Alternativen & Zweitmeinungen

The single fresh independent read-only Codex review recommended this canary.
Associate-body validation lacks an independent firmware-ABI oracle and would
mostly repeat H23's internally consistent snapshot. Event-ring observation is
correlational and can include unrelated asynchronous activity. The canary
changes one command but reuses an already supported read-only command and most
directly separates generic post-PMK liveness from Associate-specific failure.

## Folgen

One cold boot settles one explicit hypothesis without weakening completion,
DMA-read, timeout, secrecy, or quarantine boundaries. H25 is diagnostic only:
it cannot claim association, network state, traffic, domain execution, or IOMMU
isolation.
