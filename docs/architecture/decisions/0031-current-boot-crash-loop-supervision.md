# 0031 — Crash-loop supervision is authoritative for the current boot

Date: 2026-07-20 · Status: active

## Context

Genesis requires rapid guest crashes to park a replaceable Wasm service rather
than respawn it forever. ADR 0003 places crash detection and rollback
supervision in the permanent core, while ADR 0004 makes the live typed snapshot
authoritative for current-boot facts and denies authoritative persistent memory
writes until persistence and rollback exist. V0 therefore needs a deterministic
bound that does not invent durable authority.

## Decision

The semantic owner is a generic permanent-core
`wasm_runtime::crash_loop_supervisor` submodule, compiled as a child module
through the existing clean `wasm_runtime.rs`; the echo service is only the
first integration proof. For each replaceable Wasm service, the core owns an
anchored, RAM-only current-boot crash window with threshold `N=3` and
`W=10_000 ms`.

The first classified crash anchors the window and sets its count to 1. A later
classified crash with boot-monotonic elapsed time `< W` increments the count;
at exactly `W`, or later, that crash anchors a new window at count 1. A
successful short-lived replacement does not reset or slide the history.
Counters and elapsed calculations saturate. A missing, regressed, overflowing,
or otherwise ambiguous monotonic clock is treated as inside the window,
fail-toward-park. Civil or CMOS wall time has no influence.

For crashes 1 and 2, the core first invalidates the failed instance's authority,
then allocates a fresh generation and instance epoch through normal artifact and
grant validation, and attempts exactly one replacement. A failure attributable
to that admitted replacement is another classified crash. On crash 3, the core
first invalidates the old authority, then parks immediately and creates or
starts no replacement.

Count only core-observed terminal guest failures attributable to an admitted
instance: traps, out-of-fuel, enforced deadlines, attributable instantiation
failures, and contract-defined abnormal entry failures. Guest-authored crash
claims are never authority. Authorized stop, kill, drop, hot-swap, operator
restart, admission or capability denial, a health warning without terminal
failure, and host/core failure do not increment the history.

Every dispatch, host call, deferred callback, and writable-handle boundary must
match the current service generation and instance epoch before effects. Crash,
restart, and park invalidate the old authority before replacement. A stale
generation or epoch is denied without guest, host-state, peer, or writable
effects.

Parking, the counter, and the window are authoritative RAM-only current-boot
state. Reboot clears all three in V0. Serial and the current snapshot must expose
machine-readable decisions, counts, generation/epoch, parked state, clock
status, and whether replacement was attempted. RECLOG is audit evidence when
available, never the authority. Evidence or storage failure must not prevent a
park or authorize another restart.

V0 adds no `service.unpark` capability. The existing capability-gated
`recovery.restart_last_good` is the sole explicit unpark route. It must name and
match the parked service's expected generation and park event, revalidate its
artifact and grants, clear crash history, and start a fresh generation and
instance epoch. Denial mutates nothing. Ordinary start, restart, drop, reload,
and aliases cannot bypass parked state.

Normative boundary results are therefore unambiguous: a crash at `W-1`
increments the anchored window; a crash at `W` starts a new window at 1; crash
3 parks without replacement; authorized recovery clears history and creates
one freshly validated generation/epoch; all other unpark attempts are denied.

## Alternatives & independent opinions

R43 recommended the selected anchored three-in-ten-seconds, current-boot policy.
Its authority matches ADR 0004: authoritative persistent writes are unavailable
until persistence and rollback exist, while current snapshots can authoritatively
describe this boot.

R44 instead recommended a durable budget of three consecutive crashes, reset
only after 60 seconds of healthy runtime and preserved across reboot. That
policy prevents reboot laundering and is a credible destination. It is rejected
for V0 because it requires a durable supervisor ledger/storage contract and
owner-approved recovery authority that the architecture does not yet provide,
and it expands beyond this bounded slice. Simulating that authority with serial
or snapshots would be dishonest.

R49 and R50 independently agreed that implementation must move to a new
generic permanent-core module outside `HELLO_ARTIFACT_SOURCE_SET`. They differed
on compilation placement. R49 placed the supervisor below `wasm_runtime.rs`;
R50 proposed compiling a top-level generic file through `echo_service.rs`.
This ADR selects R49's `wasm_runtime.rs` child-module path because it preserves
generic semantic ownership and follows the existing runtime submodule
structure. R50's echo compilation-owner path is rejected because it would
address generic policy under an echo fixture namespace, even if the file's
contents were otherwise generic.

`seed-kernel/src/current_boot_service.rs` is part of
`HELLO_ARTIFACT_SOURCE_SET`. Changing any byte changes hashes embedded in two
P-256-signed Hello identity descriptors. It therefore remains byte-identical
unless the owner authorizes re-signing. Editing the signed source set, patching
descriptor hashes without valid signatures, or claiming the new supervisor as
attested Hello source does not avoid that boundary. If architecture later
requires this policy to live in `current_boot_service.rs`, that alternative is
blocked on owner-authorized descriptor re-signing and associated signature
work.

Also rejected are resetting history after each successful start, starting a
third replacement before parking, wall-clock windows, guest-reported crashes,
and ordinary lifecycle operations that clear or bypass parking. Each permits
unbounded execution or assigns authority to the wrong source.

## Consequences

V0 tolerates two rapid classified failures and suppresses execution on the
third, while stale instances lose authority before any successor runs. The
policy remains deterministic when time or evidence is ambiguous and does not
depend on writable storage.

Reboot can launder a current-boot crash budget. This is an accepted, explicit
V0 residual, not a dismissed risk: durable cross-boot quarantine remains future
work and requires an owner-approved durable supervisor ledger, rollback and
failure semantics, and recovery authority. This ADR does not claim cross-reboot
quarantine and does not itself close the Genesis crash-loop checkbox.

A coherent next implementation and evidence allocation is limited to:

1. `seed-kernel/src/wasm_runtime.rs`
2. `seed-kernel/src/wasm_runtime/crash_loop_supervisor.rs`
3. `seed-kernel/src/echo_service.rs`
4. `seed-kernel/src/recovery_lifeline.rs`
5. `vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1`

The pure supervisor receives explicit boot-monotonic observations, so no
`time.rs` edit is needed. Missing, regressed, overflowing, or otherwise
ambiguous observations remain fail-toward-park exactly as decided above. The
allocation does not adopt inherited `main.rs` work and does not require a sixth
implementation file.

That slice must prove the threshold boundaries, crash classification, ordering,
authorized recovery, fresh authority creation, and escape negatives; this ADR
remains a separate architecture record.
