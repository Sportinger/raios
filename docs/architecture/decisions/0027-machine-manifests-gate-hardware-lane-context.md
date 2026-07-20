# 0027 — Machine manifests gate hardware lane context before dispatch

Date: 2026-07-20 · Status: active

## Context

Commit `7af1a36` added versioned QEMU and Surface manifests, a schema/checker,
and report binding. The open Agent Fabric checkbox also requires those files to
be the curated-context source for lane system prompts. Two fresh independent
read-only Codex reviews rejected closure: the checker does not enforce the full
schema, manifest identity is not bound before use, no lane launcher consumes a
validated prompt block, and required CPU/memory facts are still `unknown`.

Both reviews distinguished honest unknown data from closure evidence. The QEMU
configuration is controlled by this repository and can provide deterministic
CPU/memory/device facts. The Surface Pro 4 has multiple hardware SKUs, so its
actual CPU features and memory topology cannot be inferred safely from public
model specifications; they require a structured capture from the owner device.

## Decision

1. Manifest `valid` and `curated_context_ready` are separate states. Structural
   validity permits explicitly unknown facts with provenance. Prompt readiness
   requires every fact path named by the lane order to be observed and bound to
   the selected machine.
2. A hardware-dependent lane order names an expected machine ID, expected
   manifest SHA-256, and required fact paths. The manifest is read once,
   fully validated, identity/digest checked, and rendered from that same
   in-memory value into a bounded prompt block containing selected facts and
   their provenance.
3. Hardware-dependent Codex workers are dispatched through one repository
   launcher. Any malformed schema, wrong machine, digest drift, missing required
   fact, or prompt-render failure stops before the child process starts. A fake
   child/sentinel negative test proves zero child invocations on every denial.
4. The manifest checker must enforce every schema rule on which the launcher
   relies, including required fields, types, enums, non-empty collections,
   additional-property rejection, unique identities, provenance binding, and
   `status`/`value` coupling. A separately stored schema that is only partially
   interpreted is not authority.
5. The QEMU manifest may become prompt-ready from repository-controlled launch
   facts and negative drift tests. The Surface manifest remains valid but not
   prompt-ready until the owner supplies a structured CPUID/boot-memory/device
   capture from the actual reference machine. That affected strand is
   owner-blocked; it does not stop independent QEMU/control-plane work.
6. VM launch may consume the same resolver later, but the current checkbox is
   closed only by the lane-prompt consumer. Report-time hashing after a launch
   is evidence, not a substitute for the pre-dispatch gate.

## Alternatives & second opinions

The two reviews agreed on the material gaps. One emphasized three defects in
the committed slice: partial schema enforcement, post-launch-only identity,
and no prompt consumer. The other additionally required the explicit
`valid`/`curated_context_ready` split and identified the Surface capture as an
owner dependency. There was no substantive disagreement.

- Treat valid JSON as curated context: rejected because no agent prompt is
  proven to consume it and missing facts would silently become authority.
- Make `unknown` structurally invalid: rejected because honest partial
  inventories must remain representable and useful for non-dependent work.
- Fill Surface values from public specifications: rejected because SKU and
  installed-memory differences would fabricate machine-specific evidence.
- Let every lane paste manifest excerpts manually: rejected because identity,
  digest, required-fact, and no-child-on-denial boundaries would be unverifiable.
- Require a new external JSON-schema dependency: not required. A complete
  repository checker is acceptable when its selftest covers every relied-on
  rule and its behavior remains fail-closed on the supported PowerShell host.

## Consequences

Hardware lane prompts gain reproducible machine identity and provenance, at the
cost of a mandatory launcher and stricter lane-order fields. QEMU context can
advance independently. Surface-dependent lanes remain explicitly blocked by
the owner capture rather than being falsely closed or halting unrelated work.
Changing the supported-machine set or bypassing the launcher is a governance
change and requires a later ADR.
