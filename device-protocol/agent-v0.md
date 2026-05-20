# raiOS Agent Protocol V0

Stage-0 exposes the first native agent protocol over the existing serial
console. It is observation-first: mutating methods deny by default, while
selected valid diagnostics may record local-only current-boot evidence without
loading artifacts, exporting provider context, or changing service inventory.
Provider context injection remains disabled until the OpenAI direct TLS path has
fail-closed certificate verification or pinning and the selected snapshot
projection passes `system.snapshot.v0` redaction rules.

## Serial Commands

The console accepts short aliases and full method names:

```text
describe              -> system.describe
snapshot              -> system.snapshot
caps                  -> system.capabilities
bootlog               -> system.boot_log
services              -> service.inventory
problems              -> problem.list
device.graph          -> device.graph
agent memory.profile  -> memory.profile
agent memory.context diagnostic -> memory.context
agent memory.context provider_minimal -> memory.context with provider export disabled
agent provider.context_export provider_minimal -> denied export gate with audit/event ids
agent provider.context_gate provider_minimal -> read-only export gate diagnostics
agent provider.context_gate_selftest provider_minimal -> local-only negative gate selftest
agent provider.context_injection_gate provider_minimal -> read-only final injection gate diagnostics
agent provider.context_injection_gate_selftest provider_minimal -> local-only final injection negative selftest
agent module.grant_diagnostic -> read-only computed-grant hash-reference diagnostic
agent module.grant_diagnostic_selftest -> local-only module grant diagnostic selftest
agent module.audit_rollback_diagnostic -> audit/rollback hash-reference diagnostic with local-only retention for valid references
agent module.audit_rollback_diagnostic_selftest -> local-only audit/rollback hash-reference diagnostic selftest
agent module.load_gate_retained_selftest -> local-only retained-reference gate selftest
agent module.load_gate_audit_rollback_selftest -> local-only audit/rollback gate selftest
agent memory.recent_events -> memory.recent_events
agent audit.events 8 -> memory.recent_events with limit 8
agent <method>        -> dispatch raw method name
```

Each response is written to serial between markers:

```text
RAIOS_AGENT_BEGIN system.snapshot
{ ... JSON envelope ... }
RAIOS_AGENT_END system.snapshot
```

The envelope shape is:

```json
{
  "v": "raios.agent.v0",
  "t": "response",
  "id": "serial",
  "body": {
    "method": "system.snapshot",
    "result": {}
  }
}
```

## Read-Only Methods

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
problem.list
service.inventory
memory.profile
memory.context
memory.query
memory.trace
memory.recent_events
audit.events
provider.context_gate
provider.context_gate_selftest
provider.context_injection_gate
provider.context_injection_gate_selftest
module.grant_diagnostic
module.grant_diagnostic_selftest
module.audit_rollback_diagnostic
module.audit_rollback_diagnostic_selftest
module.load_gate_retained_selftest
module.load_gate_audit_rollback_selftest
```

`system.snapshot` reports `system.snapshot.v0` facts for framebuffer, entropy,
USB-xHCI, Wi-Fi target probe, e1000/IPv4 network state, input, provider state,
capabilities, and known problems. The current serial command emits a local
inspection profile. Provider adapters must not attach this raw local profile to
requests; they must use the field classification and redaction rules in
`system-snapshot-v0.md`.

`service.inventory` reports a static `service.inventory.v0` view over the
currently monolithic Stage-0 kernel. Entries already use stable ids such as
`core.boot`, `svc.ui.framebuffer`, `drv.usb.xhci`, `svc.net.ipv4`, and
`svc.provider.openai_direct`.

`memory.profile` reports the local context profiles known to Stage-0.
`memory.context` returns a bounded `raios.agent_context.v0` packet over
`current_boot` facts. `memory.query` and `memory.trace` expose the small V0
record index and source links. Provider-bound context injection remains disabled
even when `memory.context provider_minimal` is requested. That profile now emits
a local `raios.provider_context_projection.v0` preview with explicit
`public`/`local_only`/`secret` field treatment, included/omitted field lists,
packet/field-list hashes, and current-boot event ids for the local projection.
The preview still has `can_export: false` until provider trust is positive and
a real provider request binding plus provider export audit binding exists.

`memory.recent_events` returns a bounded RAM-only `event.log.v0` view over the
current-boot agent protocol event ring. `audit.events [limit]` is an alias for
the same data. Records use `audit.event.v0`, stable
`event.current_boot.<sequence>` ids, method names, classification, outcome,
requested capability, and compact evidence links. It does not persist memory and
is not exported to providers. Provider context export denial events also carry
structured non-authorizing `bindings` with packet and field-list hashes, but
those denial bindings are not positive provider export authority.
When a real direct OpenAI `ask` request is allowed to start, the event log may
also contain a local-only `provider_request.envelope_created` record with
`raios.provider_request_envelope.v0` hashes. That envelope is not created by
`provider.context_export` and is not a context export binding. On pinned/WebPKI
positive trust paths, the same real `ask` path may also record local-only
`raios.provider_request_binding.v0` and
`raios.provider_context_export_audit_binding.v0` records. They are current-boot
evidence for the request and audit binding gates only; automatic provider
context injection remains disabled.
`provider.context_gate provider_minimal` can validate retained positive binding
pairs read-only. `provider.context_export provider_minimal` can consume a valid
pair once for local gate evaluation, but still returns `capability_denied` and
does not attach context to a provider body.
`provider.context_gate_selftest provider_minimal` emits local-only
`raios.provider_context_gate_negative_selftest.v0` test infrastructure for
stale/dropped ids, previous-boot-or-unretained ids, substituted schemas,
substituted positive records, hash mismatches, and trust-bypass records. It does
not create request envelopes, positive binding records, provider writes, or
provider body attachment.
`provider.context_injection_gate provider_minimal` emits local-only
`raios.provider_context_injection_gate.v0` diagnostics for the separate final
body-attachment gate. It requires the future
`raios.provider_context_injection_authorization.v0` schema, currently reports
that authorization as missing, and keeps `can_attach_context: false`.
`provider.context_injection_gate_selftest provider_minimal` emits local-only
`raios.provider_context_injection_gate_negative_selftest.v0` test
infrastructure for missing, stale, substituted, body-hash mismatched,
trust-downgraded, and unauthorized body-attachment final authorization
candidates. It does not mutate the global event log, create real envelopes or
positive binding records, create final authorization records, write to a
provider, or attach context to a provider body.

`module.grant_diagnostic` emits local-only
`raios.module_computed_grant_diagnostic.v0`. With no arguments it reports the
computed grant as absent. With hash arguments it checks only the
`raios.computed_capability_grant.canonical.v0` hash reference:

```text
module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
```

The guest does not accept artifact bytes or JSON evidence through this method.
A valid hash reference sets `computed_candidate_present: true`, but still keeps
`grants_capability: false`, `grants_load_now: false`,
`authorizes_guest_load: false`, `can_load_now: false`,
`service_inventory_change: none`, and `load_attempted: false`.
It also records a local-only current-boot
`raios.module_computed_grant_reference.v0` event binding and returns it as
`retained_reference`; that record is diagnostic evidence only and is not a
loader token.
`module.grant_diagnostic_selftest` emits local-only
`raios.module_computed_grant_diagnostic_selftest.v0` test infrastructure for
absent, accepted-current-boot, stale, mismatched, and wrong-policy hash
references. It does not load artifacts or mutate `service.inventory.v0`.
`module.audit_rollback_diagnostic` emits local-only
`raios.module_audit_rollback_reference_diagnostic.v0`. With no arguments it
reports the audit/rollback reference as absent. With hash arguments it checks
only canonical hash references and current-boot ids:

```text
module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]
```

The guest does not accept artifact bytes, JSON evidence, local approval text,
or durable records through this method. A valid hash reference sets
`audit_record_hash_reference_present: true` and
`rollback_plan_hash_reference_present: true`, but still keeps
`durable_audit_written: false`, `rollback_plan_installed: false`,
`grants_capability: false`, `can_load_now: false`,
`service_inventory_change: none`, and `load_attempted: false`. It records a
local-only current-boot `raios.module_audit_rollback_reference.v0` event binding
and returns it as `retained_audit_rollback_reference`; that record is
hash-reference evidence only and is not durable audit, an installed rollback
plan, or a loader token.
`module.audit_rollback_diagnostic_selftest` emits local-only
`raios.module_audit_rollback_reference_diagnostic_selftest.v0` test
infrastructure for absent, accepted-current-boot, stale, previous-boot event
id, wrong-schema, substituted, mismatched, and invalid service-slot references.
It does not mutate the global event log, create durable audit records, create
rollback plans, allocate service slots, load artifacts, or mutate
`service.inventory.v0`.
`module.load_gate_retained_selftest` emits local-only
`raios.module_load_gate_retained_reference_selftest.v0` test infrastructure for
the denied load gate's retained-reference predicate. It covers missing,
accepted-current-boot-but-denied, stale/dropped event id,
previous-boot-or-unretained event id, wrong schema, substituted record, and hash
mismatch cases. It does not mutate the global event log, create retained
records, load artifacts, or mutate `service.inventory.v0`.
`module.load_gate_audit_rollback_selftest` emits local-only
`raios.module_load_gate_audit_rollback_selftest.v0` test infrastructure for the
denied load gate's durable audit and rollback predicates. It covers missing
audit record, missing rollback plan, matching audit/rollback evidence that is
still denied by missing loader and service slot, audit/rollback schema
mismatches, retained grant hash mismatch, manifest/artifact/VM-report/
local-attestation mismatches, local approval mismatch, rollback-plan hash
mismatch, rollback artifact mismatch, and rollback service-slot mismatch. It
does not mutate the global event log, create durable audit records, create
rollback plans, allocate service slots, load artifacts, or mutate
`service.inventory.v0`.

## Denied-By-Default Methods

Mutating or potentially mutating methods return `capability_denied`:

```text
memory.record_observation
memory.propose_policy
memory.supersede_fact
memory.redact
memory.compact
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
provider.configure
provider.context_export
wifi.configure
```

The denial includes `event_id` and `audit_event_id` fields that cite the
current-boot event record for the denied method. The denial names the missing
evidence: `raios.module_manifest.v0`,
`candidate_artifact_sha256`, `raios.vm_test_report.v0`,
`raios.local_attestation.v0`, computed capability grant, local approval,
durable audit record, rollback plan, and a ram-only service slot.

`module.load_ephemeral` and `service.load_ephemeral` use the explicit
`raios.module_load_gate.v0` denial schema. The gate reports
`load_mode: ram_only`, `requested_capability: cap.module.load_ephemeral`,
`target: live_service_graph`, missing required evidence plus any retained
computed-grant and audit/rollback hash references, the loader as `unavailable`,
`service_slot: unallocated`, `can_load: false`, and `load_attempted: false`. It
also exposes
`raios.module_load_gate_audit_rollback_requirements.v0`, which names the
required `raios.audit_record.v0` and `raios.rollback_plan.v0` bindings while
keeping record creation disabled. The matching `audit.event.v0` record carries
the same schemas as non-authorizing event bindings so the denial is visible
through `audit.events`.

`provider.context_export [provider_minimal]` uses the same denied-by-default
envelope but reports `raios.provider_context_export.v0` gate state instead of
module evidence. It records `cap.provider.context_export`, returns
`provider_write: not_attempted`, and requires positive provider trust, a
provider-minimal projection, packet/field-list evidence, provider request
binding, and a distinct export audit event before any future context attachment.
The current denied path emits request-binding-denial and export-denial-audit
records only; those records explicitly do not satisfy the export gates, even
though their event-log bindings carry exact provider-minimal packet and
field-list hashes.

The direct OpenAI `ask` path can create positive local-only binding records when
provider trust is pinned/verified, but the standalone `provider.context_export`
method still denies, must not fake a provider request envelope, and only
consumes retained positive bindings for local gate evaluation.
