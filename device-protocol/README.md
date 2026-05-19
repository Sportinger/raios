# Device Protocol

Specifications and tooling for the raiOS device/agent protocol.

Current V0 docs:

- `agent-v0.md` - serial-accessible `raios.agent.v0` read-only methods and
  denied-by-default mutating methods.
- `system-snapshot-v0.md` - typed current-boot snapshot plus field
  classification and provider redaction rules.
- `service-inventory-v0.md` - static service graph for the current monolithic
  Stage-0 kernel.
- `capabilities-v0.md` - current observe-only capability policy and mutation
  denial rules.
- `provider-trust-v0.md` - provider peer-trust states, fail-closed default, and
  the implemented OpenAI SPKI/leaf-certificate pinning slices.
- `memory-context-v0.md` - read-only `current_boot` memory profile, context,
  query, and trace methods for bounded `raios.agent_context.v0` packets.
- `provider-context-export-v0.md` - denied-by-default provider context export
  gate, local-only positive binding records, and audit-binding requirements.
- `provider-request-envelope-v0.md` - local pre-write envelope contract for
  binding one provider context packet to one real provider request.
- `event-log-v0.md` - RAM-only `current_boot` event/audit log for agent
  protocol reads and denials.
- `module-manifest-v0.md` - manifest contract for agent-proposed artifacts.
- `vm-test-report-v0.md` - Shadow-VM report emitted by
  `vm-harness\shadow-vm-smoke.ps1`.
- `local-attestation-v0.md` - local evidence record binding manifest,
  artifact, VM report, approval, and rollback mode.
- `recovery-v0.md` - planned recovery control protocol, separate from normal
  provider chat.
