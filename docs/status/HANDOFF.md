# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, Surface capture blocked at PCI restore proof)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`57e5dae`. The detached old root at `09751a7` contains foreign WIP; never clean,
reset, merge, or integrate it.

**Start:** H25 proves post-PMK mailbox liveness, but Association, DHCP, and
traffic are unproven. The bounded model, Wire V1, codec, host extractor E3, and
ADR 0040 are accepted and pushed. K3 is an unaccepted five-file dirty slice in
canonical main; its predicate and freestanding build are green.

**Goal:** capture same-boot CPUID/SMBIOS/Limine-map/PCI facts through USB
RECLOG, validate real Surface readback, make `surface-pro-4.v1.json`
prompt-ready, and pin its digest.

**Finished when:** the PCI production-logic restore test and K3 pass independent
review, exact slices are committed/pushed, the owner-custodied stick cold-boots
the Surface, extraction validates, and the manifest digest is pinned.

**Not now:** moving Wi-Fi out of the kernel, a full network stack, H26 scan
TSF/beacon timestamp work, TPM/remote attestation, or machine-identity claims.

## Next step

`docs/scope/05-drivers-hardware.md` Wi-Fi prerequisite is **blocked**; owner:
orchestrator tooling. Two bounded Codex implementation strategies (clean linked
worktree P1 and canonical trusted worktree P2) made no changes because Codex
0.144.6 `workspace-write` cannot enforce its split Windows writable roots.
Resolve and smoke-test that sandbox outside product files before dispatching a
fresh PCI lane. Then prove I/O/Mem32/Mem64 BAR and full Command restoration,
review it read-only, secure the exact two-file slice, and re-review K3. No safe
second WLAN lane exists: K3 depends on this gate; H26 depends on real capture
and manifest pinning. Do not boot the Surface with K3 yet.

## Recently (exactly 3, newest first)

### 2026-07-21 - PCI restore decision recorded
`57e5dae`: ADR 0040 resolves R14/R15 and requires a shared production-logic
restore test before owner boot.

### 2026-07-21 - Host extractor accepted
`160af76`: E3 passed 28/28 tests and independent R13 review.

### 2026-07-21 - Surface capture integration built, not accepted
K3's five-file slice passed its predicate/build, but review exposed the missing
PCI restore proof; it remains uncommitted and must not expand.
