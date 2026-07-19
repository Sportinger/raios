# 0024 — Version-bound grant snapshots make rollback a revocation transaction

Date: 2026-07-19 · Status: active

## Context

ADR 0023 requires rollback to revoke `live-now - live-at-target` before any
instance rebuild. The existing promoted-version record carries only an import
list digest, not the approved per-surface set, while the boot projection drops
the exact parent grant identity needed by a chained revoke. The current rollback
also tears down first and unpromotes to absence; neither fact can authorize a
rollback to a real version that retains surface B while removing surface A.

## Decision

1. Every promoted Wasm version carries an immutable canonical
   `GrantTargetSnapshot`: schema, service id, version/artifact binding hash,
   ordered duplicate-free `(HostImportId, typed scope)` entries, count and
   snapshot hash. These fields are covered by the version's existing signed and
   canonical promotion identity. The old import-list digest remains consistency
   evidence only; linker declarations are never rollback authority.
2. The durable live projection retains each live grant's exact `grant_id`,
   canonical grant record hash, grant epoch, generation, binding, surface and
   scope. Rollback never reconstructs a parent reference from ambient state.
3. Rollback is a durable transaction: quarantine the domain; verify the target
   version and snapshot; compute the ordered exact-parent delta; preflight space;
   append an intent binding source head/projection, target snapshot and delta;
   append one exact chained revoke per delta member; re-fold after each append;
   append a commit binding the resulting projection to the target snapshot.
   Only then install the fold in RAM, tear down, and rebuild the target instance.
   Teardown remains cleanup, never revocation.
4. Retry is idempotent. `ensure_revoked(parent, transaction)` returns appended,
   already-revoked-by-this-transaction, or denied; it never creates a second
   revoke fork. An uncommitted intent found at boot quarantines only that domain
   and resumes/converges the transaction before any guest call.
5. Any missing/tampered snapshot, parent mismatch, capacity shortfall, partial
   append failure or ambiguous recovery fails closed before teardown. Revokes
   already durable remain authority-reducing; peers and retained target grants
   remain untouched.

## Alternatives & second opinion

A fresh read-only Codex opinion was requested neutrally. Claude agents were not
available by explicit owner instruction, so no Claude opinion was solicited.
The advisor recommended the chosen version-bound snapshot and rejected:

- Deriving the target set from historical grant events keyed only by artifact:
  event existence does not prove the selected version's approved set or cutoff;
  making it sound requires the same signed snapshot plus more replay ambiguity.
- Treating target absence as sufficient: it is fail-closed but is unpromotion,
  not rollback to a version, destroys retained B, and cannot prove delta semantics.

## Consequences

Rollback gains a deterministic, auditable authority transition and safe crash
recovery, at the cost of extending promotion/version records and adding an
intent/commit state machine. The decisive predicate must show A revoked with
zero effect, B retained, a peer domain unchanged, reboot persistence, and
idempotent recovery after every transaction boundary. Slice 5 still owns the
exclusive migration of remaining host surfaces to the common gate.
