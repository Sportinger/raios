# Final Plan Decision After Multi-Agent Review

## Basis
This file summarizes the decision after three independent reviews and two
consensus passes:

- `review-1-core-boundary.md`
- `review-2-agent-protocol-provider.md`
- `review-3-runtime-vm-persistence.md`
- `consensus-a-plan-changes.md`
- `consensus-b-plan-changes.md`

## Decision
The plan remains good, but the sequencing needed tightening. The codebase is a
strong Stage-0 MVP, not yet a live-rebuildable OS. The next plan steps are now:

```text
1. fail-closed provider trust
2. read-only agent protocol
3. typed system.snapshot.v0
4. static service.inventory.v0
5. capability policy and redaction
6. module_manifest.v0
7. vm_test_report.v0
8. local_attestation.v0
9. denied-by-default mutating methods
10. only then ephemeral live services
```

## Accepted Changes
- Treat TLS verification or provider/SPKI pinning as a gate, not polish.
- Treat the direct OpenAI path as a normal provider-service candidate, not the
  recovery lifeline.
- Make Phase 5 a static service inventory and typed snapshot phase before any
  dynamic service runtime.
- Keep the first agent protocol read-only.
- Require data classification before any provider context injection.
- Require manifest, VM report, local attestation, health, and audit before live
  loading.
- Require an image/state layout document before persistence or boot media write
  logic grows.

## Deferred
- Guest Wasm runtime.
- OTA consumption inside SeedOS.
- Persistent secret storage.
- Recovery over normal OpenAI chat.
- Core-generation handoff.
- Full QMP fault-injection harness.

## Documents Updated
- `docs/ROADMAP.md`
- `docs/architecture-decisions/0002-agent-self-description-and-live-built-modules.md`
- `docs/architecture-decisions/0003-always-on-core-and-live-rebuildable-world.md`

