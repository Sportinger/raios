# raiOS Module Load Gate V0

`raios.module_load_gate.v0` is the current Stage-0 gate for
`module.load_ephemeral` and `service.load_ephemeral`.

It is a denial schema, not a loader. The gate makes the missing evidence and
runtime blockers explicit before raiOS has a module ABI, ram-only service slots,
durable audit records, rollback state, or an in-guest policy grant writer.

## Current Trigger

```text
module.load_ephemeral
service.load_ephemeral
```

Both methods return `capability_denied`. The response includes matching
`event_id` and `audit_event_id` handles that point at the current-boot
`audit.event.v0` record. That event also carries a compact
`raios.module_load_gate.v0` binding.

## Denial Shape

The serial error response reports:

```json
{
  "code": "capability_denied",
  "schema": "raios.module_load_gate.v0",
  "request": {
    "load_mode": "ram_only",
    "requested_capability": "cap.module.load_ephemeral",
    "risk": "modify_ram",
    "target": "live_service_graph",
    "subject": "agent.session.serial"
  },
  "gate_state": {
    "module_manifest": "missing",
    "candidate_artifact": "missing",
    "vm_test_report": "missing",
    "local_attestation": "missing",
    "computed_capability_grant": "missing",
    "local_approval": "missing",
    "rollback_plan": "missing",
    "durable_audit_record": "missing",
    "loader": "unavailable",
    "service_slot": "unallocated",
    "artifact_loaded": false,
    "service_started": false,
    "persistence": "none",
    "can_load": false
  }
}
```

The denial must also expose `blocked_by` reasons for each missing gate:

```text
module_manifest_missing
candidate_artifact_missing
vm_test_report_missing
local_attestation_missing
computed_capability_grant_missing
durable_audit_record_missing
rollback_plan_missing
module_loader_unimplemented
```

## Required Evidence

The current gate names the evidence that a future positive load decision must
bind:

```text
raios.module_manifest.v0
candidate_artifact_sha256
raios.vm_test_report.v0
raios.local_attestation.v0
computed_capability_grant
local_approval
raios.audit_record.v0
rollback_plan
ram_only_service_slot
```

The current guest does not accept those records as inputs yet. Host-side
manifest validation, VM reports, registry evidence, and local attestations are
evidence artifacts only until an in-guest policy path computes a grant.

## Event Binding

The current-boot event record uses:

```text
kind: agent_protocol.capability_denied
source_method: module.load_ephemeral
requested_capability: cap.module.load_ephemeral
risk: modify_ram
resource: live_service_graph
reason: missing_evidence
evidence:
  missing_required_evidence
  capability_denied
  module_load_gate_evaluated
  service_inventory_unchanged
  load_not_attempted
bindings.schema: raios.module_load_gate.v0
```

The binding repeats the gate state and evidence hashes:

```text
manifest_hash: null
artifact_hash: null
vm_test_report_hash: null
local_attestation_hash: null
service_inventory_change: none
load_attempted: false
```

## Invariants

- No artifact bytes are loaded by this gate.
- No service inventory row is added, removed, or changed.
- No provider response, manifest claim, test report, registry entry, or local
  attestation grants execution by itself.
- A valid `raios.module_manifest.v0` is only one input to a future computed
  grant.
- The normal module gate does not authorize recovery artifacts; recovery loads
  use `raios.recovery.v0` and a separate recovery trust boundary.
- A future positive path must keep denial reasons explicit when any evidence is
  missing, stale, mismatched, or outside the current boot scope.
