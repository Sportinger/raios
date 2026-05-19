# raiOS Agent Protocol V0

Stage-0 exposes the first native agent protocol over the existing serial
console. This is intentionally read-only except for explicit denial responses.
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
`raios.vm_test_report.v0`, `local_attestation.v0`, computed capability grant,
local approval, and rollback plan.

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
