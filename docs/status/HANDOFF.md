# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, PCI restore slice parked after two failed strategies)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`dc22477`. The detached old root at `09751a7` contains foreign WIP; never clean,
reset, merge, or integrate it.

H25 proves post-PMK mailbox liveness. K3 remains an unaccepted five-file dirty
Surface-capture slice. The PCI prerequisite remains an unaccepted two-file
dirty slice in `seed-kernel/src/pci.rs` and
`scripts/test-pci-bar-sizing.ps1`; no product file from this iteration is
accepted, staged, committed, pushed, or safe for owner boot.

Codex CLI 0.145.0 fixed the prior Windows workspace-write dispatch blocker.
The orchestrator externally ran the current PCI predicate green (10/10), plus
rustfmt and diff checks. Two fresh independent reviews still rejected it:
production `outw` can false-green behind the fake seam; a final Memory64 BAR
may probe beyond header `bar_count`; alignment/range overflow and all-ones
device-disappearance cases are not rejected; the untracked predicate cannot
check itself reliably. This is the second failed implementation/review
strategy, so the stuck rule parks it. Owner: orchestrator architecture.

## Next step

Do not iterate a third patch on the same seam and do not write/boot the Surface
stick. Rescope the PCI proof as a new architecture decision before product
work: define one bounded probe outcome carrying slot width independently from
BAR acceptance, bind the concrete x86 word-write adapter into the testable
boundary, and specify fail-closed validity for header bounds, all-ones reads,
alignment, and range overflow. Obtain two fresh neutral read-only Codex
opinions on that replacement boundary. Only an accepted ADR may authorize a
new implementation lane. K3 and H26 remain dependency-blocked; no safe
independent WLAN lane exists.

## Recently (exactly 3, newest first)

### 2026-07-22 - PCI P4 predicate green, reviews rejected
Ten production-logic host tests passed externally; R-C/R-D found remaining
boundary and false-green defects, so no acceptance followed.

### 2026-07-22 - Worker sandbox restored
Codex 0.145.0 workspace-write smoke test succeeded in canonical main; linker
execution remains denied only inside managed workers and is verified externally.

### 2026-07-21 - PCI u16 decision recorded
`dc22477`: ADR 0041 requires real word cycles so Command writes preserve PCI
Status W1C bits.
