# 0030 — Durable grants require same-generation signed install authority

Date: 2026-07-20 · Status: active

## Context

Durable capability-grant history is interpreted from attacker-writable media.
Disk records, outer RECLOG framing, order fields, unsigned grant events, and
rollback markers are attacker-copyable. They can provide structure and trusted
append order after validation, but not cryptographic authority. Authority comes
from the exact semantics of a fully verified signed install identity: service,
generation, artifact, and signed import snapshot.

The normal generation-2 producer appends the signed authorization, its linked
promotion, the artifact persist record and blob, and only then grants imports in
the fixed order `env.log`, `env.counter_get`. Its generation-insensitive reuse of
an already-live surface can retain a predecessor-generation log grant while a
generation-2 install adds the counter grant.

## Decision

Every accepted durable grant must follow exactly one fully verified signed
install identity for the same service, generation, artifact, and signed import
snapshot, and that snapshot must contain the grant's import and scope. Missing,
later, duplicate, crossed, or otherwise ambiguous identity is rejected
fail-closed. Grant generation fields and record order constrain the association;
they do not themselves authorize it. The linked promotion, artifact persist,
blob, and all existing grant-fold and rollback links must independently validate.

For a mixed `(1,2)` live projection, the required grant ordering is
`auth1 < log1 < auth2 < counter2`. The generation-2 identity cannot
retroactively authorize `log1`. For `(2,2)`, the requirement is
`auth2 < log2 < counter2`. In both cases, the promotion/persist/blob records
belonging to each physical install remain between its authorization and grant
phase and must validate.

The deterministic compatibility identities are generation-1 target artifact
`33ea...5756c` with the log-only snapshot and generation-2 source artifact
`f81f...abd2` with the log-plus-counter snapshot. They illustrate the rule;
the architectural requirement applies to every service, generation, artifact,
and signed snapshot.

ADR 0029 remains unchanged: RAM-only boot recovery may reverify and fold only
already durable, correctly authorized history. It never appends a missing grant
or install authority and cannot complete or synthesize an authorization chain.

## Alternatives & independent opinions

R36 and R37 independently agreed on `AUTH_PER_GENERATION_ORDER`: authority is
assigned per grant generation, not from whichever signed install is current.
They also identified one compatibility nuance. The seeded four-record
predecessor bundle contains authorization, promotion, persist, and unpromotion,
but does not synthesize a generation-1 grant. A mixed history therefore requires
a real earlier grant phase or partial/legacy producer history with `auth1`
before `log1`; the seed bundle alone cannot justify that grant.

Rejected alternatives:

- Requiring current authorization before all grants rejects a legitimate
  retained predecessor-generation grant.
- Accepting current authorization anywhere before rollback intent permits a
  later signature to authorize an earlier attacker-copied grant retroactively.
- Trusting a grant's generation field or record order alone treats unsigned,
  attacker-copyable data as authority.
- Dropping generation-specific validation makes mixed and replayed histories
  cryptographically ambiguous.

## Consequences

Offline validators must group grants by generation, fully verify each matching
signed install identity and snapshot, enforce the per-generation record-index
order, and reject missing or ambiguous duplicates and crossings. They must also
preserve the existing derived grant projections, exact parent links, promotion
and artifact linkage, and rollback transactions. This admits authentic retained
predecessor grants without allowing a later install signature to bless copied
history, at the cost of validating every generation represented in the live
projection.
