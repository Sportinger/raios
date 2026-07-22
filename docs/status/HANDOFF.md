# HANDOFF - Where do we stand?

> Window, not log: one Now, one Next step, three Recently entries. Replace on
> update; never append. Keep under 60 lines and roughly 2 KB.

## Now (2026-07-22, H26 review blocked)

Canonical `main` is `C:\Users\admin\Documents\raios2-main` at pushed
`57f0886`. The detached old root remains foreign WIP; never clean, reset,
merge, or integrate it. Four uncommitted files belong to the stopped H26
hardware lane: `seed-kernel/src/{wifi.rs,marvell_wifi_pcie.rs}` and the two
focused WLAN predicate scripts. Do not discard or edit them outside a new
authorized repair lane.

ADR 0045 authorized and consumed exactly one Surface H26 implementation
dispatch. Its claim is persisted at `target/state/adr0045-h26.claim`. The lane
retains firmware scan TSF plus AP beacon timestamp, requires both for selected
BSS readiness, restores PMK -> Associate, uses only the TSF builder, and adds
fail-closed mutations. Both focused suites, rustfmt, diff/file scope and the
root freestanding release build are green.

Acceptance is blocked. Unsafe inventory remains 404 total but two normalized
hashes changed because H26 code is inside existing unsafe blocks. Two fresh
independent reviews disagreed: one `ACCEPT_BASELINE`; one `REJECT` found that
scan completion is not sequence-bound and selected-target revalidation is not
atomic through Associate publication/network release. No baseline update,
product commit, image package, or USB write occurred.

## Next step

Owner must explicitly authorize one second H26 repair dispatch and ADR-0045
amendment. That bounded lane must close only the two review findings, rerun all
predicates/build/inventory, obtain fresh independent acceptance, then update
the two exact unsafe hashes. Without that authorization, park H26 blocked.

## Recently (exactly 3, newest first)

### 2026-07-22 - H26 one-shot launcher secured
`57f0886`: atomic single-use claim, ADR handle lease and replay/race tests;
43/43 and independent `ACCEPT`, production claim absent before real dispatch.

### 2026-07-22 - H26 implementation reached review
Four-file diff and mutations are green; root release artifact built, but the
security review disagreement prevents acceptance.

### 2026-07-22 - H25 isolated the fault
Returned-stick evidence proved post-PMK mailbox liveness with clean USB and
network denial as designed, leaving Associate/BSS-specific semantics.
