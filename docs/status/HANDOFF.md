# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, Surface capture contract secured)

Canonical integration is clean `main` at `C:\Users\admin\Documents\raios2-main`;
`HEAD == main == origin/main` at `93a30e3`. The old root
`C:\Users\admin\Documents\raios2` stays detached at `09751a7` with preserved
foreign WIP. Do not clean, reset, merge, or integrate from that root.

H25 plus the saved Surface extraction proves current-epoch completion of the
post-PMK `GET_HW_SPEC` canary. Association, `PORT_RELEASE`, DHCP, and traffic
remain unproven; the fault is Associate/BSS-specific. H26 will retain scan TSF
and AP beacon time and append Associate TLV `0x0113`.

The Owner selected development evidence under physical stick/image custody in
ADR 0038. It requires one raiOS boot to capture CPUID, SMBIOS, Limine memory map,
and PCI into hash-chained RECLOG; it claims no TPM, remote, or production
attestation. `93a30e3` adds the accepted bounded no_std core contract: 798 added
lines, 7/7 focused tests, and independent read-only ACCEPT.

H26 remains blocked until a real Surface capture makes
`surface-pro-4.v1.json` prompt-ready and pins its digest. No H26 worker or stick
write is authorized. Marvell bring-up stays in-kernel for now; capture does not
decide the later driver-domain split. `vendor/limine` remains a dangling Gitlink
without `.gitmodules`, so its empty directory proves no provenance.

Disk pressure is cleared: 43 marked `%TEMP%` Cargo caches freed ~27.7 GB; source,
worktrees, images, and reports were preserved.

## Next step

Dispatch one bounded core wire-codec lane with malformed-wire negatives, review,
and secure it. The format is shared, so kernel persistence and extraction must
not race its design. Once frozen, run two disjoint lanes in parallel: kernel
measurement plus RECLOG/USB append, and the host extractor. Then package through
the hardware launcher, capture on Surface, pin the manifest digest, and resume
H26 through `scripts/invoke-codex-lane.ps1`.

## Recently (exactly 3, newest first)

### 2026-07-21 — Bounded Surface capture contract accepted
PCI/IRQ/BAR and part/digest/completion boundaries passed 7 tests and R9 review;
`93a30e3` is pushed.

### 2026-07-21 — Owner chose custody-based development evidence
ADR 0038 records the narrower non-attestation evidence bar and both reviews.

### 2026-07-21 — H25 proved post-PMK mailbox liveness
The canary completed; network stayed denied and cold reboot stayed required.
