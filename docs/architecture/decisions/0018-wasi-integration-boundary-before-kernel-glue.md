# 0018 — Harden the WASI integration boundary before the kernel-glue slice; no general seed-kernel refactor

Date: 2026-07-18 · Status: active

## Context

With the T2 threads box closed and the WASI shim complete on the host side
(slices 0-5, plus buildfs-pack), the next slice wires the shim into the
kernel behind the grant gate. Two independent read-only reviews were taken
before starting it — one on overall seed-kernel structure, one on the new
WASI/threads code. Both reports are archived
(`docs/_archive/2026-07-18_review-kernel-structure.md`,
`…_review-new-code.md`).

Both reviews reached the same conclusion from opposite ends: the real
integration risk is not the old large kernel files (event_log.rs,
durable_store.rs, usb.rs are god-modules but have small fan-in to the WASI
zone), it is the still-open seam in the NEW code — a third parallel
instantiation/pointer/lifecycle path would otherwise be created, and several
resource and trust-chain gaps exist.

## Decision

Do **not** refactor the legacy kernel modules first — that would enlarge
diff surface and regression risk with only QEMU verification available. But
**before** the kernel-glue slice, consolidate the integration boundary and
fix the new-code hardening findings, all in the host-testable crates so the
kernel (and its threads QEMU selftest) stays green and unchanged:

Integration boundary:
- an opaque, privately-constructed `AuthorizedBuildJob` — the only ticket
  the kernel accepts — binding the WASI grant, the observed module imports,
  the validated `BuildGuestClassV1` hash, and the mount-manifest hashes;
- a `WasiBuildInstance` with one FD table, a mount table, and the process
  world, so the kernel never translates FDs or coordinates cross-mount rules;
- a pure, host-testable guest-range/iovec checker (the kernel adds only a
  thin `wasmi::Memory` adapter);
- a single typed link/import plan;
- a job lifecycle that is NOT driven by the UI input tick.

The WASI adapter has zero direct dependencies on `event_log`,
`durable_store`, `usb`, or the legacy `cap.module.load_ephemeral` renderer.

New-code hardening (fail-closed):
- bounded scheduler trace (event counter + rolling digest + capped ring),
  keeping `trace()`'s signature so the kernel selftest is unaffected;
- `park_wait` gains the current-thread causality check `on_quantum_end`
  already has;
- RamFs gains a node-churn quota (generation-tagged slot reuse) so
  create/unlink under the live-file quota cannot grow kernel memory;
- a non-forgeable `FrozenOutput` whose digest is recomputed from the actual
  output bytes — the egress gate compares bound content, not claimed hashes;
- `BuildGuestClassV1::validate()` enforces field relations, not just
  non-zero;
- split `fs_rights_base`/`fs_rights_inheriting` + mount id in `FdEntry`.

Negative integration tests are mandatory: pointer wrap/OOB, extra import,
thread 49, cross-mount FDs, node churn, forged output hash.

## Acceptance

The existing threads QEMU selftest stays green and unchanged.

## Alternatives & second opinions

- **General kernel refactor first** (split event_log/durable_store/usb):
  rejected by both reviews — disproportionate regression risk under
  QEMU-only verification, and it does not address the actual blockers.
- **Continue straight into glue with no boundary work**: rejected — it
  would harden the current `wasm_runtime` instantiation/pointer duplication
  into permanent architecture (a third parallel path beside envelope.rs and
  personal_shell.rs).

The two reviews are the two independent second opinions; they concur, so no
dissent to record beyond the two rejected alternatives above.

## Consequences

One host-testable hardening slice precedes the glue; the glue slice then
only decodes validated guest pointers and drives one `WasiBuildInstance`
behind an `AuthorizedBuildJob`. The legacy god-modules are left for a
separate, later, test-backed effort — flagged, not forgotten.
