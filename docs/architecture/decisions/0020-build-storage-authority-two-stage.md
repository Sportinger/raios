# 0020 — Build storage authority: logical in core, materialized in the store, committed by record

Date: 2026-07-18 · Status: active

## Context

Slice-6 stage A landed the WASI build gate (ADR 0018): the kernel
instantiates a build guest only behind an opaque `AuthorizedBuildJob`. The
stage-B lane (store adapters) stopped at a real gap: the core vocabulary
binds mount-manifest hashes but carries no storage authority — nothing says
which chunk content may be read from ARTSTOR or where double-build-verified
output may be written. Two independent second opinions were taken (Codex
xhigh read-only; Fable max) on a neutral design question.

## Decision

Authority keeps one shape per layer: **content-addressed in raios-core,
offset-shaped only inside ARTSTOR, handle-shaped in the WASI zone.**

1. **Second core stage `BuildStorageAuthority` (opaque, privately
   constructed):** built only from (&AuthorizedBuildJob, both actual BuildFS
   manifests, an opaque kernel-minted output lease descriptor). Core
   re-validates manifest structure, recomputes their hashes and requires
   equality with the ticket's pins (closing the seam where kernel code could
   mount manifests unrelated to the ticket), validates the lease SHAPE
   (target class, quota relations vs. a new sibling mount/output budget —
   BuildGuestClassV1 and its golden hash stay frozen), and binds everything
   under a domain-separated job-binding digest. No offsets, no LBAs in core.
2. **Materialization before instantiation (kernel store):** ARTSTOR resolves
   every chunk digest of the validated manifests into a kernel-private,
   per-job chunk table (generation-bound, GC-pinned, single-use instance
   nonce per run). The WASI-zone reader maps a `ChunkReadRequest` ONLY
   through this table — no fallback lookup; absent entry = NotCapable. The
   chunk hash is re-verified on every read (load-bearing TOCTOU defense —
   never optimized away after "verified once").
3. **Egress commits by record, just in time:** both runs execute with zero
   store write authority (RAM arenas only). The existing double-build gate
   yields a hardened, job- and destination-bound egress result
   (authorizes_load=false, private ctor). A NEW pre-I/O core evaluator
   (own type — `ScopedArtifactStoreBlobInput` is NOT reused; only its
   overflow-safe span predicate is extracted) checks the store-chosen
   reservation against the authority; ARTSTOR re-checks live geometry under
   its allocator lock, writes, readback-verifies, and appends the typed
   commit record LAST. Liveness = commit record, not blob presence: crash
   or failure leaves only inert garbage for existing GC. "Nothing persisted
   on failure" is defined as "no committed authority", which multi-sector
   append hardware can actually guarantee.

## Alternatives & second opinions

- **Bind storage into `AuthorizedBuildJob` itself (Codex preference):**
  one ticket carrying all authority. Rejected for now: the ticket's API
  just landed in the kernel gate; the two-stage form keeps it stable and
  makes authority conjunctive (adapters require BOTH objects with equal
  binding digests). Recorded as dissent; revisit if the pairing check ever
  proves error-prone in practice.
- **Pre-granted concrete egress span in core (orchestrator's initial lean):**
  rejected by both opinions — offsets in core break layer purity and invite
  staleness; the store picks offsets at commit time under its lock.
- **Reuse the scoped_artifact_store_blob evaluator as the grant (Fable
  leaned to its truth-table style):** style adopted, type NOT reused —
  Codex showed it embeds promotion policy and is post-write evidence today.
  Both positions recorded; the reconciliation (new dedicated pre-I/O
  evaluator + in-store recheck + readback evidence) satisfies both.
- Shared risks named by both and bound into the design: grant→use TOCTOU
  (per-read hashing), GC/offset reuse (generation + pinning), ticket replay
  (single-use instance nonce; the Copy ticket alone is not consumable),
  digest-only commits (bundle bytes bound, not just the manifest hash),
  unbounded manifests (explicit mount budget), reservation staleness
  (capacity accounting at authorization, rebuild on loss).

## Consequences

Easier: G1b becomes mechanical again (adapters consume validated objects);
every failure mode maps to a typed denial; the commit point is auditable in
RECLOG. Harder: two-object pairing must be checked (binding digest); ARTSTOR
gains a small transactional surface (lease → write handle → record). The
deliberate trade: more core vocabulary now, so the kernel WASI zone can stay
structurally incapable of ambient storage access forever.
