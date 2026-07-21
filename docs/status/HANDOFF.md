# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, capture gate owner-blocked)

Canonical integration is clean `main` at `C:\Users\admin\Documents\raios2-main`;
`HEAD == main == origin/main` at `ea720e1`. Four pushed commits restored the
two-review gate, document authority/status, and this cursor.

The old root `C:\Users\admin\Documents\raios2` stays detached at `09751a7`
with preserved rollback, M8, NET8 and generated WIP. IOMMU, HID/hwtrace, W7.3
and website WIP also stay preserved. Do not stash, reset, clean, remove, merge,
or use that root for integration before lane disposition.

H25 `d617efd` plus saved extraction proves current-epoch completion of the
post-PMK `GET_HW_SPEC` canary. Association, `PORT_RELEASE`, DHCP, and traffic
remain unproven; the fault is Associate/BSS-specific. H26 (scan TSF + AP time in
Associate TLV `0x0113`) is selected but not implemented.

H26 is owner-blocked: `surface-pro-4.v1.json` has
`curated_context_ready:false` because structured CPU, memory, and device facts
from the Surface are missing (ADR 0027). No H26 worker or stick write is
authorized. `vendor/limine` is a dangling Gitlink without `.gitmodules`; the
empty directory keeps status clean but proves no provenance.

Two reviews agree on an in-raiOS CPUID/SMBIOS/Limine-map/PCI capture via
hash-chained RECLOG; Windows-only is insufficient.
They disagree whether Owner custody suffices or challenge, trusted build digest,
enrolled fingerprint and possibly TPM quote are mandatory. Owner decision and
an ADR recording this security tradeoff are required before implementation.

## Next step

Owner chooses the capture evidence bar. Then record both opinions and the
decision in ADR 0028 and dispatch one bounded capture-contract lane. Do not
dispatch H26 or write the stick before the accepted real-Surface capture makes
the manifest prompt-ready; H26 must use `scripts/invoke-codex-lane.ps1`.

## Recently (exactly 3, newest first)

### 2026-07-21 — Surface capture path reviewed
Both reviews require an in-raiOS capture; the attestation bar awaits the Owner.

### 2026-07-21 — H25 proved post-PMK mailbox liveness
The canary completed as expected; network stayed denied and cold reboot stayed
required. The remaining fault is Associate/BSS-specific.

### 2026-07-21 — Repository control plane normalized
All WIP was preserved; clean main moved out of `%TEMP%` and four commits landed.
