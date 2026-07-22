# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, reviewed Surface capture image ready)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`a17c18b`. The detached old root `C:\Users\admin\Documents\raios2` contains
foreign WIP; never clean, reset, merge, or integrate it.

K3a PCI and the five-file K3 Fast Track are accepted and pushed. The complete
gate passed: PCI 26/26 plus three mutations, Surface predicate, unsafe baseline,
all diff checks, freestanding release build and one independent K3 ACCEPT.

Reviewed output is
`target\surface-capture-a17c18b\raios-surface-capture-gpt.img`, 537,936,384
bytes, sha256 `b43effcdb47bec0a534658f3d785a0a982f94c16221eb0f5aa27eca10603f843`.
Kernel sha256 is `33fbb29ad36bf5a25988b4fc136ec381e36e646fbeb6dc03bd5154b654905944`;
the embedded pinned Marvell firmware and slot-A generation-1 Core Policy were
verified. GPT primary/backup CRCs, A/B layout, BOOTCTL normal-A, SEED_DATA and
empty chained RECLOG inspect green. Worktree is clean; no physical write ran.

## Next step

Owner inserts the intended USB stick. Orchestrator lists disks read-only and
reports model, size, bus and disk number. Owner then explicitly confirms the
exact disk number and phrase before the image is written. After the Surface
cold boot, return the stick for read-only Surface Fact extraction and manifest
candidate verification. H26 remains blocked until that manifest is accepted.

## Recently (exactly 3, newest first)

### 2026-07-22 - Surface capture image ready
`a17c18b`: bounded five-file Fast Track accepted; signed firmware-bearing GPT
image built and inspected with an empty valid RECLOG.

### 2026-07-22 - K3a PCI capture boundary accepted
`2ed6a35`: 26/26 runtime tests, three mutation negatives and one independent
ACCEPT secure the fail-closed PCI Result enumeration.

### 2026-07-22 - Owner selected diagnostic Fast Track
`7508d25`: ADR 0044 retains damage-prevention boundaries and defers hardening.
