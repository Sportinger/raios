# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, active milestone: real Surface capture)

Clean canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`a0dec5e`. The old root remains detached at `09751a7` with foreign WIP; never
clean, reset, merge, or integrate from it.

**Start:** H25 proves post-PMK mailbox liveness; Association, `PORT_RELEASE`,
DHCP, and traffic are unproven. Owner-custodied policy (ADR 0038), bounded model
(`93a30e3`), Wire V1 (ADR 0039), and codec (`a0dec5e`) are accepted and pushed.

**Goal:** one complete same-boot CPUID/SMBIOS/Limine-map/PCI capture through
hash-chained USB RECLOG; then make `surface-pro-4.v1.json` prompt-ready and pin
its digest.

**Finished when:** kernel and extractor are accepted, the reviewed stick boots
the Surface, readback and manifest checks pass, and the digest is pinned. Only
then may H26 add scan TSF/AP beacon time to Associate TLV `0x0113`.

**Not now:** moving Wi-Fi out of kernel, full network stack, TPM/remote
attestation, or machine-identity claims. `vendor/limine` remains a dangling
Gitlink without `.gitmodules`; its empty directory proves no provenance.

## Next step

Run two disjoint lanes in parallel. K3 measures facts before PCI BAR sizing can
mutate configuration and appends Wire V1 after USB init. E3 extracts one
contiguous series, validates records/completion/digest, and emits only a
manifest candidate. Review and secure each separately. Then package through the
hardware launcher and capture on Surface; H26 stays blocked through manifest
pinning.

## Recently (exactly 3, newest first)

### 2026-07-21 — Canonical Wire V1 codec accepted
`a0dec5e`: 584-line bounded encoder/decoder slice passed R11 and is pushed.

### 2026-07-21 — Surface wire boundary frozen
ADR 0039 records both independent opinions and resolves their small layout
disagreement before kernel and extractor work diverge.

### 2026-07-21 — Bounded capture model accepted
`93a30e3`: PCI/IRQ/BAR and part/digest/completion boundaries passed review.
