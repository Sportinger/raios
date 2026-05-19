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
even when `memory.context provider_minimal` is requested.

`memory.recent_events` returns a bounded RAM-only `event.log.v0` view over the
current-boot agent protocol event ring. `audit.events [limit]` is an alias for
the same data. Records use `audit.event.v0`, stable
`event.current_boot.<sequence>` ids, method names, classification, outcome,
requested capability, and compact evidence links. It does not persist memory and
is not exported to providers.

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
wifi.configure
```

The denial includes `event_id` and `audit_event_id` fields that cite the
current-boot event record for the denied method. The denial names the missing
evidence: `raios.module_manifest.v0`,
`raios.vm_test_report.v0`, `local_attestation.v0`, computed capability grant,
local approval, and rollback plan.
