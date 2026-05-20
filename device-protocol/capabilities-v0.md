# raiOS Capability Policy V0

`system.capabilities.v0` is the first local capability catalog for
`raios.agent.v0`. It documents what Stage-0 may expose to an agent, what is
only requested by manifests, and why all mutating methods currently fail closed.

V0 is intentionally small:

```text
read-only self-description is granted for the current boot
all mutation is denied unless a later policy can prove local evidence
```

The current implementation source of truth is `seed-kernel/src/agent_protocol.rs`.
This document specifies the policy that file currently exposes over
`system.capabilities` and `capability_denied` responses.

## Policy Invariants

- Capabilities are explicit names, not a generic shell or command runner.
- A capability grant is computed by raiOS local policy. It is never trusted from
  a manifest, provider response, agent claim, or test report by itself.
- Manifest `requested_caps` are requests only. Manifest `granted_caps` must stay
  empty for V0 artifacts.
- Missing evidence denies the action. Denial is structured and visible as
  `capability_denied`, not a silent fallback.
- Provider context injection is separate from local capability grants and remains
  gated by TLS trust hardening.
- Persistent writes, hardware mutation, live loading, and service control remain
  denied until manifest, VM report, local attestation, computed grant, local
  approval, rollback, and audit records exist.

## Risk Classes

| Risk | Meaning | V0 result |
| --- | --- | --- |
| `observe` | Read local facts, logs, inventory, capabilities, and problems. | Granted for selected current-boot methods. |
| `diagnose` | Run bounded probes or checks that may touch devices but should not change state. | Not granted in Stage-0 V0. |
| `simulate` | Build or test outside the live system, usually in a VM harness. | Evidence may be produced outside the guest; not a guest mutation grant. |
| `export` | Send local system context or evidence across the provider boundary. | Denied. |
| `modify_ram` | Change current-boot runtime state, load ephemeral code, or alter live services. | Denied. |
| `persist` | Change boot image, saved config, module set, policy, or rollback state. | Denied. |
| `hardware` | Touch physical device state with effects outside a VM or beyond observation. | Denied. |

## Grants Versus Requests

Requests:

- appear in `raios.module_manifest.v0` as `requested_caps`
- describe what an artifact or agent wants
- do not authorize execution
- may be used by policy, VM harnesses, and review tools to decide what evidence
  is required

Grants:

- appear in `system.capabilities.v0` with `granted: true`
- are computed by local policy for a subject, resource, scope, and duration
- are scoped to `current_boot` for V0 read-only methods
- must be auditable before any future mutating grant exists

A future mutating grant should bind at least:

```json
{
  "id": "cap.module.load_ephemeral",
  "subject": "module:<manifest-name>",
  "resource": "live_service_graph",
  "scope": "current_boot",
  "duration": "ephemeral",
  "grant_source": "local_policy",
  "evidence": [
    "raios.module_manifest.v0",
    "candidate_artifact_sha256",
    "raios.vm_test_report.v0",
    "raios.local_attestation.v0",
    "raios.computed_capability_grant.v0",
    "local_approval",
    "raios.audit_record.v0",
    "rollback_plan",
    "ram_only_service_slot"
  ],
  "audit_level": "full"
}
```

This shape is descriptive only. Stage-0 V0 does not grant it.

## Initial Read-Only Grants

These capabilities are granted for the current boot and map directly to
read-only protocol methods:

| Capability | Method | Risk | Scope | Meaning |
| --- | --- | --- | --- | --- |
| `cap.system.describe.read` | `system.describe` | `observe` | `current_boot` | Read OS and protocol identity. |
| `cap.system.snapshot.read` | `system.snapshot` | `observe` | `current_boot` | Read typed Stage-0 status facts. |
| `cap.system.boot_log.read` | `system.boot_log` | `observe` | `current_boot` | Read the local serial boot log ring. |
| `cap.system.capabilities.read` | `system.capabilities` | `observe` | `current_boot` | Read the current capability catalog. |
| `cap.service.inventory.read` | `service.inventory` | `observe` | `current_boot` | Read static service inventory. |
| `cap.device.graph.read` | `device.graph` | `observe` | `current_boot` | Read known device graph facts. |
| `cap.problem.list.read` | `problem.list` | `observe` | `current_boot` | Read known local problems and gaps. |
| `cap.memory.profile.read` | `memory.profile` | `observe` | `current_boot` | Read available memory context profiles. |
| `cap.memory.context.read` | `memory.context` | `observe` | `current_boot` | Read bounded current-boot agent context. |
| `cap.memory.query.read` | `memory.query` | `observe` | `current_boot` | Query current-boot memory record ids. |
| `cap.memory.trace.read` | `memory.trace` | `observe` | `current_boot` | Trace current-boot memory records to source evidence. |
| `cap.memory.recent_events.read` | `memory.recent_events` | `observe` | `current_boot` | Read bounded current-boot memory event records. |
| `cap.audit.events.read` | `audit.events` | `observe` | `current_boot` | Read bounded current-boot audit event records. |
| `cap.provider.context_export.read` | `provider.context_gate`, `provider.context_gate_selftest` | `observe` | `current_boot` | Read provider context gate diagnostics and local predicate selftests. |
| `cap.provider.context_injection.read` | `provider.context_injection_gate`, `provider.context_injection_gate_selftest` | `observe` | `current_boot` | Read final provider context injection gate diagnostics and local predicate selftests. |
| `cap.module.grant_diagnostic.read` | `module.grant_diagnostic`, `module.grant_diagnostic_selftest`, `module.audit_rollback_diagnostic`, `module.audit_rollback_diagnostic_selftest`, `module.load_gate_retained_selftest`, `module.load_gate_audit_rollback_selftest` | `observe` | `current_boot` | Read module computed-grant, audit/rollback hash-reference diagnostics, and denied-load gate predicate selftests. |

`system.snapshot` also reports `capability_denied.for_all_mutating_methods` so an
agent can discover that mutation is intentionally unavailable.

## Denied-By-Default Methods

The following methods are present as protocol vocabulary but must return
`capability_denied` in Stage-0 V0:

```text
memory.record_observation
memory.propose_policy
memory.supersede_fact
memory.redact
memory.compact
provider.context_export
module.propose
module.build_result
module.test_request
module.test_result
module.load_ephemeral
module.persist
module.rollback
service.load_ephemeral
service.restart
service.start
service.stop
config.apply
apply_config
provider.configure
wifi.configure
draw_text
probe_device
download_signed_module
run_module_test
```

Memory mutation methods map to the denied catalog capability
`cap.memory.mutate`. They are visible so future agents can reason about the
missing grant without treating RAM-only records as durable memory.

`module.load_ephemeral` and `service.load_ephemeral` map to
`cap.module.load_ephemeral`, risk `modify_ram`, and resource
`live_service_graph`. They return the explicit `raios.module_load_gate.v0`
denial schema and record the same gate as an `event.log.v0` binding. The gate
keeps `can_load: false`, `load_attempted: false`,
`service_inventory_change: none`, `artifact_loaded: false`, and
`service_started: false` until the manifest, exact artifact hash, VM report,
local attestation, computed capability grant, local approval, durable audit
record, rollback plan, and ram-only service slot are all present and locally
verified. The first host-side computed grant diagnostic is
`raios.computed_capability_grant.v0`; it can prove the evidence tuple is
consistent, but it still reports `grants_capability: false`,
`grants_load_now: false`, and `authorizes_guest_load: false` until an in-guest
policy path exists. The read-only `module.grant_diagnostic` method can inspect
the canonical hash reference for that diagnostic without accepting artifact
bytes. A valid reference is retained as a local-only current-boot
`raios.module_computed_grant_reference.v0` event binding, but it also keeps
`can_load_now: false` and `load_attempted: false`.
The read-only `module.audit_rollback_diagnostic` method can inspect canonical
`raios.audit_record.v0` and `raios.rollback_plan.v0` hash references without
accepting durable records or artifact bytes. A valid audit/rollback reference
is retained as a local-only current-boot
`raios.module_audit_rollback_reference.v0` event binding, but still keeps
`durable_audit_written: false`, `rollback_plan_installed: false`,
`can_load_now: false`, and `load_attempted: false`. The denied live load gate
must revalidate that retained reference before reporting it as accepted
evidence; rejected retained references remain non-authorizing and expose no
accepted audit/rollback hashes.

Provider context export maps to `cap.provider.context_export` and risk
`export`. It is denied until positive provider trust, the
`provider_minimal` redaction projection, packet/field-list evidence, provider
request binding, and a distinct provider export audit binding exist. The
standalone denial path may emit request-binding-denial and export-denial-audit
records, but those records do not satisfy the positive binding gates. The real
pinned OpenAI `ask` path may emit positive local-only request/export audit
binding records. `provider.context_gate` may validate retained positive binding
pairs read-only, and `provider.context_export` may consume a valid pair once for
local gate evaluation, but those records do not grant provider export or body
attachment while automatic context injection is disabled. The standalone denial
must report `provider_write: not_attempted`.
`provider.context_gate_selftest` is granted only as local read/test
infrastructure under `cap.provider.context_export.read`; it must not create
request envelopes, positive binding records, provider writes, or body
attachment.
`provider.context_injection_gate` and
`provider.context_injection_gate_selftest` are granted only as local read/test
diagnostics under `cap.provider.context_injection.read`; they name and test the
future final authorization schema but do not grant `cap.provider.context_export`
or body attachment.

Capability denials must name the relevant evidence gates:

```text
raios.module_manifest.v0
candidate_artifact_sha256
raios.vm_test_report.v0
raios.local_attestation.v0
raios.computed_capability_grant.v0
local_approval
ram_only_service_slot
rollback_plan
raios.audit_record.v0
raios.memory_persistence.v0
raios.provider_context_projection.v0
raios.provider_context_export.v0
projected_packet_hash
exported_field_list_hash
omitted_field_list_hash
provider_request_binding
provider_context_export_audit_binding
checked_current_boot_binding_consumption
raios.provider_context_injection_authorization.v0
final_prewrite_body_hash_check
```

V0 does not distinguish between "method known but currently unsafe" and "method
will be allowed soon" by returning partial success. The durable behavior is:
known mutating method -> structured denial; unknown method -> unknown command.

## Audit Expectation

Read-only methods and known `capability_denied` outcomes are recorded in the
RAM-only `event.log.v0` ring for the current boot. The serial log and protocol
transcript remain useful evidence, but they are no longer the only structured
observation record. Denial responses also cite the current-boot `event_id` and
`audit_event_id`.

Before any mutating grant can exist, raiOS must emit or persist an audit record
that can explain the decision later. The current RAM ring is not durable enough
for that grant. The minimum durable record should include:

```text
agent session id
request id
method
requested capability
subject
resource
risk
manifest hash
artifact hash
VM report hash
local attestation hash
approval decision
load mode
rollback pointer or rollback denial reason
policy decision: granted | denied
decision reason
timestamp or boot-relative sequence
```

For persistence, the audit record must also bind the previous-good boot pointer,
pending artifact set, boot-success marker plan, rollback trigger, and safe-mode
rule that disables non-core modules and persistent writes.

## Provider Boundary

Capability grants are local execution authority. They are not permission to send
the same data to a provider.

Fields that leave the machine still need redaction classification:
`public`, `local_only`, or `secret`. Provider requests must not automatically
attach `system.snapshot.v0` or capability/audit context unless the current
provider trust state is one of the positive verified states and the outbound
projection has applied the matching redaction profile. Local serial access may
show read-only facts that the provider path is not yet allowed to receive.
`provider.context_export provider_minimal` is the V0 gate for this boundary and
must remain denied until it can create positive request and export audit
bindings.

## Open Questions

- What exact subject identifiers should V0 use for local users, provider
  sessions, workstation-side tools, guest diagnostics, and live services?
- Where should audit records live before persistent storage and rollback slots
  exist?
- Which low-risk diagnostic capability, if any, is the first candidate for a
  non-observe grant?
- Should `module.propose` remain denied in the guest while workstation-side
  proposal tooling accepts manifests outside the raiOS runtime?
