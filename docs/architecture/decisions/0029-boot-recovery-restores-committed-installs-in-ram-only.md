# 0029 — Boot recovery restores committed installs in RAM only

Date: 2026-07-20 · Status: active

## Context

ADR 0026 attempted to treat a recovery re-persist of one signed install action
as the same installation while rejecting an adversarial authentic duplicate.
Independent review found that the implemented distinction used only a reused
authorization link, a fresh promotion frame hash, and a fresh ARTSTOR offset.
Those values are not authenticated recovery provenance: an offline writer can
copy every authentic signed payload, choose a new blob position, and recompute
the unkeyed RECLOG framing into exactly the accepted recovery shape.

Two fresh independent read-only Codex opinions agreed that current durable
records cannot distinguish legitimate recovery from that replay. They disagreed
on the replacement. One recommended a kernel-signed recovery authorization
bound to a protected device key, prior authenticated head, and monotonic
anti-replay state. The other recommended eliminating recovery re-persistence
and restoring the already committed install in RAM only. The repository has no
owner-pinned recovery key or external monotonic anchor, and introducing one
would require owner authority and hardware/policy work beyond this checkbox.

## Decision

1. A physical Wasm install identity is exactly one fully verified owner-signed
   authorization, linked promotion, artifact persist record, and verified blob.
   Any second complete triple for the same `install_action_sha256` is a hard
   duplicate. Different signed context for that identity is ambiguous history.
2. Normal boot recovery re-verifies the original committed identity, blob,
   signatures, grant snapshot, and boot posture, then restores its signed
   generation, promotion metadata, grant projection, and service state in RAM.
   It appends no install authorization, promotion, artifact persist, blob copy,
   or grant-authority record.
3. Recovery authority is boot-internal and restore-only. General agent or serial
   methods may request verification/diagnostics but cannot reach a durable
   install-effect path or cause recovery persistence.
4. A second recovery attempt in one boot is denied before load/start or durable
   effect. A later reboot may recover the same committed install again after
   complete re-verification because volatile state has been lost.
5. Crash at any recovery boundary leaves the original committed install as the
   sole durable authority. The next boot retries verification and RAM restore;
   there is no partial recovery transaction to resume or deduplicate.
6. An incomplete physical install is not auto-completed from copied authentic
   records. It remains quarantined and requires a new physical-owner-authorized
   transaction. A client retry after a fully committed original install returns
   already committed without appending a second triple.
7. New physical installs still choose one greater than the maximum generation
   among fully verified unique physical installs. Recovery restores the signed
   generation unchanged. Core predecessor selection remains strict in both
   generation and signed log sequence.
8. Legacy media containing multiple same-action re-persists remains fail closed;
   frame order, offsets, timestamps, or best-guess migration cannot bless it.
   Owner reauthorization or an explicit future format migration is required.
9. Valid zero-grant targets remain supported end to end. Restoring or rolling
   back such a target must expose an empty import set and cannot fail after
   revocation merely because its surface mask is zero.

## Alternatives & second opinions

- Kernel-signed recovery authorization: cryptographically viable only with a
  device-protected key, owner-authenticated public-key policy, and monotonic
  anti-replay anchor. One opinion recommended this if re-persistence is
  mandatory. Rejected for the current design because none of those authorities
  exists and a disk-stored key or unsigned recovery flag would not solve replay.
- Continue deduplicating by same action plus fresh links/offsets: rejected because
  it exactly matches an attacker-constructible authentic replay.
- Bind recovery only to boot posture, RECLOG tail, sequence, random nonce, or
  storage position: rejected because all are public or disk-replayable without
  a protected signing/MAC key and external freshness.
- Stop recovery entirely: rejected because the original committed install and
  blob already provide sufficient authority for a fully verified RAM restore.
- Keep writing evidence-only recovery audits: allowed only if they are excluded
  from install history and rollback selection and can never authorize effects;
  not required for this repair.

## Consequences

Boot recovery becomes simpler and removes the indistinguishable durable replay
shape. It no longer creates a storage-healing copy, and a partial physical
install cannot be auto-completed without fresh owner authority. Existing valid
single-triple installs remain compatible. The harness must prove zero durable
install-history growth across recovery, same-boot idempotency, later-reboot
restore, original-auth/new-promotion replay denial, crash safety, and unchanged
strict rollback selection. A future requirement for authenticated re-persist or
whole-disk anti-rollback is owner-blocked pending protected key and monotonic
hardware policy and requires another ADR.
