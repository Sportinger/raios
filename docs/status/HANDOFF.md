# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-21, cleanup iteration secured)

Canonical integration is the clean `main` worktree at
`C:\Users\admin\Documents\raios2-main`; `HEAD == main == origin/main` at
`c71b863`. Three separate pushed commits restored the Owner's two-review gate,
made HANDOFF the cursor, and aligned the derived status views with H25/H26.

The old root `C:\Users\admin\Documents\raios2` remains detached at `09751a7`
and holds preserved stopped-lane WIP: rollback-image security, M8 crash-loop,
NET8/Schannel, plus generated diagnostics. IOMMU, old HID/hwtrace, rejected
W7.3, and website WIP also remain preserved. Do not stash, reset, clean,
remove, merge, or treat that root as integration before lane disposition.

H25 `d617efd` plus saved read-only extraction proves current-epoch completion
of the post-PMK `GET_HW_SPEC` canary. Association,
`PORT_RELEASE`, DHCP, and traffic remain unproven; the fault is
Associate/BSS-specific. H26 (scan firmware TSF + AP beacon timestamp in
Associate TLV `0x0113`) is selected but not implemented.

H26 is owner-blocked before dispatch: `surface-pro-4.v1.json` has
`curated_context_ready:false` because structured CPU, memory, and device facts
from the Surface are missing (ADR 0027). No H26 worker or stick write is
authorized. `vendor/limine` is a dangling Gitlink without `.gitmodules`; its
empty directory keeps status clean but establishes no provenance.

## Next step

Create one non-hardware Surface-capture-tool lane with fixture negatives for
the ADR-0027 CPU/memory/device facts. Do not dispatch H26 or write the stick.
After the Owner runs the capture on the Surface and
makes the manifest prompt-ready, pin its digest and dispatch H26 only through
`scripts/invoke-codex-lane.ps1`.

## Recently (exactly 3, newest first)

### 2026-07-21 — Repository control plane normalized
Two neutral reviews preserved all WIP; clean main was moved out of `%TEMP%`,
and control/README/status repairs landed as three pushed commits.

### 2026-07-21 — H25 proved post-PMK mailbox liveness
The canary completed as expected; network stayed denied and cold reboot stayed
required. The remaining fault is Associate/BSS-specific.

### 2026-07-21 — H26 restart point selected
Linux `mwifiex` comparison selected the missing scan-TSF Associate TLV; no H26
product work, build, or stick rewrite was started.
