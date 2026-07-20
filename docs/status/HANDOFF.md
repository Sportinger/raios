# HANDOFF — Where do we stand?

> Window, not log: exactly one Now, one Next step, and three Recently entries.
> Replace on update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-20 ~11:00, old loop intentionally stopped)

The owner retired all Claude-specific control surfaces. ADR 0025 makes
`AGENTS.md` the single Codex control plane with explicit orchestrator/worker
roles, Codex-only reviews, immediate accepted-slice commit+push, and hard WIP
checkpoints. The stale resumed root and its active repair worker were stopped;
their files were preserved. Committed HEAD remains dc039ba on origin/main.

ADR-0024 rollback work is inherited as a large dirty patch across Core,
durable store, repromotion, runtime, image builder, and QEMU harness. Older
evidence passed delta 494/494, boundaries 152/152, and quick 510/510, but later
reviews found real authorization/evidence gaps. The final strengthened run
`shadow-rollback-grant-delta-20260720-104725-22632` is red: 107 predicates,
2 failures at `rollback:delta-commit-before-rebuild`. The rollback-delta box
therefore remains open. Foreign AGENTS-era diagnostics/fixture lock remain
untouched; inventory exact ownership before staging.

## Next step

Start a fresh Codex root session, never resume the retired transcript. First
dispatch a narrow read-only diagnosis of the 10:47 report and inventory the
inherited rollback diff into independently verifiable slices. Repair only the
failing slice, rerun focused QEMU plus quick, obtain an independent Codex
review, then immediately commit and push each accepted slice. Keep disjoint
non-core lanes running unless an explicit domain-isolation full brake applies.

## Recently (exactly 3, newest first)

### 2026-07-20 — agent control consolidated on AGENTS.md
Removed live `CLAUDE.md`/`.claude` control surfaces and added mechanical docs
hygiene rejection so they cannot silently return. Historical ADR mentions stay.

### 2026-07-20 — hardened rollback run exposed a positive-path regression
New durable no-mutation/source-identity assertions were added after review;
fresh QEMU failed before delta commit/rebuild, so no unsafe closure or commit.

### 2026-07-19 — durable revoke reached the real reboot gate
Projection consumption moved into Envelope construction. Focused grant-reboot
was 29/29 and quick 510/510; explicit durable grant/revoke core stayed green.
