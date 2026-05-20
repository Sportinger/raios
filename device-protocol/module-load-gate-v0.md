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
    "computed_capability_grant": "missing | retained_hash_reference_only",
    "local_approval": "missing",
    "rollback_plan": "missing | retained_hash_reference_only_not_installed | rejected_retained_reference",
    "durable_audit_record": "missing | retained_hash_reference_only_not_durable | rejected_retained_reference",
    "loader": "unavailable",
    "service_slot": "unallocated | retained_hash_reference_only_not_allocated | rejected_retained_reference",
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
retained_computed_grant_reference_not_authorizing
durable_audit_record_missing
rollback_plan_missing
retained_audit_record_reference_not_durable
retained_rollback_plan_reference_not_installed
retained_audit_rollback_reference_wrong_schema_or_variant
retained_audit_rollback_reference_substituted_record
retained_audit_rollback_computed_grant_hash_mismatch
retained_audit_record_hash_mismatch
retained_rollback_plan_hash_mismatch
retained_audit_rollback_service_slot_mismatch
retained_service_slot_reservation_not_allocated
retained_service_slot_reservation_stale_or_dropped_event_id
retained_service_slot_reservation_wrong_schema_or_variant
retained_service_slot_reservation_substituted_record
retained_service_slot_reservation_computed_grant_hash_mismatch
retained_service_slot_reservation_audit_record_hash_mismatch
retained_service_slot_reservation_rollback_plan_hash_mismatch
retained_service_slot_reservation_pre_load_inventory_hash_mismatch
retained_service_slot_reservation_service_slot_mismatch
retained_service_slot_reservation_hash_mismatch
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
raios.computed_capability_grant.v0
local_approval
raios.audit_record.v0
rollback_plan
ram_only_service_slot
```

The current guest does not accept those records as inputs yet. Host-side
manifest validation, VM reports, registry evidence, and local attestations are
evidence artifacts only until an in-guest policy path computes a grant.

`raios.computed_capability_grant.v0` now defines the first host-side diagnostic
for that future grant. It can validate and hash an exact
manifest/artifact/VM-report/local-attestation tuple, but it is deliberately
non-authorizing in Stage-0: `grants_capability: false`,
`grants_load_now: false`, `can_load_now: false`, and `load_attempted: false`.
The read-only `module.grant_diagnostic` method can recompute and inspect a
hash-reference to that diagnostic without accepting artifact bytes. A valid
reference is retained as a local-only current-boot
`raios.module_computed_grant_reference.v0` event binding, but it still keeps
`can_load_now: false` and `load_attempted: false`. The mutating
`module.load_ephemeral` gate snapshots that retained reference into
`retained_computed_grant_reference` and reports
`computed_capability_grant: retained_hash_reference_only` with reason
`retained_computed_grant_reference_not_authorizing`. It continues to deny
loading until retained grant evidence is joined with durable audit, rollback,
loader, and ram-only service-slot evidence.

The denial now exposes the first explicit audit/rollback requirement object:

```json
{
  "schema": "raios.module_load_gate_audit_rollback_requirements.v0",
  "status": "required_missing",
  "writes_enabled": false,
  "creates_durable_audit_records": false,
  "creates_rollback_plans": false,
  "durable_audit_record": {
    "schema": "raios.audit_record.v0",
    "state": "missing | retained_hash_reference_only_not_durable | rejected_retained_reference",
    "durability": "required_before_load",
    "required_bindings": [
      "denial_event_id",
      "retained_computed_grant_reference_event_id",
      "computed_capability_grant_hash",
      "manifest_hash",
      "artifact_hash",
      "vm_test_report_hash",
      "local_attestation_hash",
      "local_approval",
      "rollback_plan_hash",
      "ram_only_service_slot_id"
    ]
  },
  "rollback_plan": {
    "schema": "raios.rollback_plan.v0",
    "state": "missing | retained_hash_reference_only_not_installed | rejected_retained_reference",
    "must_preexist_load": true,
    "required_bindings": [
      "artifact_hash",
      "pre_load_service_inventory_hash",
      "ram_only_service_slot_id",
      "cleanup_actions_hash"
    ]
  },
  "retained_audit_rollback_reference_event_id": "null | event.current_boot.00000032"
}
```

This object is a requirement schema, not persistence. Stage-0 does not create
durable audit records, rollback plans, service slots, or loader entries.
The host-side `raios.module_audit_rollback_diagnostic.v0` can now compute
canonical `raios.audit_record.v0` and `raios.rollback_plan.v0` candidates for
that evidence shape. The guest-side `module.audit_rollback_diagnostic` method
can inspect those candidates as hashes only. A valid guest hash reference is
retained as local-only RAM current-boot evidence in
`raios.module_audit_rollback_reference.v0`, but it is still not durable audit,
not an installed rollback plan, and not load authority.

Before a denied live load gate reports a retained audit/rollback reference as
hash-reference evidence, it revalidates that retained event against the current
RAM event log. The live predicate requires the latest retained computed-grant
reference, a prior retained `raios.module_load_gate.v0` denial event, matching
canonical computed-grant, rollback-plan, and audit-record hashes, and a valid
`ram_only:` service-slot id. If the retained record points at a wrong-schema
event, stale/dropped event, substituted record, mismatched hash, or mismatched
slot, the denied response reports `rejected_retained_reference`; audit and
rollback hashes stay `null` in accepted evidence fields.

`raios.module_service_slot_reservation.v0` now defines the first
non-authorizing service-slot reservation evidence. The guest-side
`module.service_slot_diagnostic` method validates only a canonical hash
reference over:

```text
retained_reference_event_id
retained_audit_rollback_reference_event_id
computed_capability_grant_sha256
audit_record_sha256
rollback_plan_sha256
pre_load_service_inventory_sha256
ram_only_service_slot_id
```

A valid reference is retained as local-only current-boot event evidence, but it
does not allocate the slot, create service inventory records, load an artifact,
or grant execution. Before the denied live load gate reports a retained
reservation as hash-reference evidence, it revalidates the retained grant event,
the retained audit/rollback event, canonical reservation hash,
computed-grant/audit/rollback hashes, pre-load service-inventory hash, and
`ram_only:` slot id. A valid reservation changes the service-slot gate state to
`retained_hash_reference_only_not_allocated`, exposes
`retained_service_slot_reservation.state: present` and
`service_slot_reservation_hash`, and still keeps
`allocates_service_slot: false`, `service_inventory_change: none`, and
`load_attempted: false`.

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
  computed_capability_grant_reference_checked
  durable_audit_record_required
  rollback_plan_required
  rollback_bindings_required
  service_inventory_unchanged
  load_not_attempted
bindings.schema: raios.module_load_gate.v0
```

The binding repeats the gate state and evidence hashes:

```text
retained_computed_grant_reference.state: missing | present
retained_audit_rollback_reference.state: missing | present | rejected
retained_audit_rollback_reference.schema: raios.module_audit_rollback_reference.v0
retained_service_slot_reservation.state: missing | present | rejected
retained_service_slot_reservation.schema: raios.module_service_slot_reservation.v0
computed_capability_grant_hash: null | sha256:<retained grant hash>
manifest_hash: null | sha256:<retained manifest hash>
artifact_hash: null | sha256:<retained artifact hash>
vm_test_report_hash: null | sha256:<retained report hash>
local_attestation_hash: null | sha256:<retained attestation hash>
audit_record_hash: null | sha256:<retained audit record hash>
rollback_plan_hash: null | sha256:<retained rollback plan hash>
local_approval_hash: null | sha256:<retained approval hash>
pre_load_service_inventory_hash: null | sha256:<retained inventory hash>
cleanup_actions_hash: null | sha256:<retained cleanup hash>
ram_only_service_slot_id: null | ram_only:<service slot id>
service_slot_reservation_hash: null | sha256:<retained reservation hash>
audit_rollback_requirements.schema: raios.module_load_gate_audit_rollback_requirements.v0
audit_rollback_requirements.status: required_missing
audit_rollback_requirements.durable_audit_record.schema: raios.audit_record.v0
audit_rollback_requirements.durable_audit_record.state: missing | retained_hash_reference_only_not_durable | rejected_retained_reference
audit_rollback_requirements.rollback_plan.schema: raios.rollback_plan.v0
audit_rollback_requirements.rollback_plan.state: missing | retained_hash_reference_only_not_installed | rejected_retained_reference
audit_rollback_requirements.writes_enabled: false
service_inventory_change: none
load_attempted: false
```

## Retained Reference Selftest

The read-only method:

```text
agent module.load_gate_retained_selftest
```

emits `raios.module_load_gate_retained_reference_selftest.v0`. It is local test
infrastructure over the retained-reference predicate and must report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_retained_reference_records: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing retained reference, accepted current-boot
reference still denied, stale/dropped retained-reference event id,
previous-boot-or-unretained reference, wrong schema or variant substitution,
substituted retained-reference record, and mismatched computed-grant hash.

## Audit/Rollback Requirement Selftest

The read-only method:

```text
agent module.load_gate_audit_rollback_selftest
```

emits `raios.module_load_gate_audit_rollback_selftest.v0`. It is local test
infrastructure over the retained audit/rollback reference, durable audit, and
rollback predicates and must report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_retained_audit_rollback_reference_records: false
creates_durable_audit_records: false
creates_rollback_plans: false
allocates_service_slot: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing, stale/dropped, previous-boot-or-unretained,
wrong-schema, substituted, computed-grant-hash-mismatched,
audit-hash-mismatched, rollback-hash-mismatched, and service-slot-mismatched
retained audit/rollback references; missing durable audit record; missing
rollback plan; matching audit/rollback evidence still denied by missing loader
and service slot; audit/rollback schema mismatches; retained grant hash
mismatch; audit-bound manifest/artifact/VM-report/local-attestation mismatches;
local approval mismatch; audit-bound rollback hash mismatch; rollback artifact
mismatch; and rollback service-slot mismatch.

## Service-Slot Gate Selftest

The read-only method:

```text
agent module.load_gate_service_slot_selftest
```

emits `raios.module_load_gate_service_slot_selftest.v0`. It is local test
infrastructure over the retained service-slot reservation predicate and must
report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_service_slot_reservation_records: false
allocates_service_slot: false
creates_service_inventory_records: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing, accepted-current-boot-but-denied,
stale/dropped, wrong-schema, substituted, computed-grant-hash-mismatched,
audit-hash-mismatched, rollback-hash-mismatched, inventory-hash-mismatched,
service-slot-mismatched, and reservation-hash-mismatched retained service-slot
reservations. Rejected cases report `actual_service_slot_state:
rejected_retained_reference` and `accepted_service_slot_reservation_hash:
false`.

## Audit/Rollback Hash-Reference Diagnostic

The read-only method:

```text
agent module.audit_rollback_diagnostic
agent module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]
agent module.audit_rollback_diagnostic_selftest
```

emits `raios.module_audit_rollback_reference_diagnostic.v0` and
`raios.module_audit_rollback_reference_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids. A valid command records a local-only
`raios.module_audit_rollback_reference.v0` event binding with
`global_event_log_mutation: valid_hash_reference_retention_only`, keeps
`creates_durable_audit_records: false`, `creates_rollback_plans: false`,
`allocates_service_slot: false`, `loads_artifact: false`, `can_load_now: false`,
and `load_attempted: false`. The selftest covers stale, previous-boot,
wrong-schema, substituted, and mismatched audit/rollback references without
mutating the global event log.

## Service-Slot Reservation Diagnostic

The read-only method:

```text
agent module.service_slot_diagnostic
agent module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]
agent module.service_slot_diagnostic_selftest
```

emits `raios.module_service_slot_reservation_diagnostic.v0` and
`raios.module_service_slot_reservation_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids. A valid command records a
local-only `raios.module_service_slot_reservation.v0` event binding with
`global_event_log_mutation: valid_hash_reference_retention_only`, keeps
`allocates_service_slot: false`, `creates_service_inventory_records: false`,
`loads_artifact: false`, `service_inventory_change: none`,
`can_load_now: false`, and `load_attempted: false`. The selftest covers absent,
accepted-current-boot, stale, mismatched reservation hash, and invalid
`ram_only:` slot references without mutating the global event log.

## Invariants

- No artifact bytes are loaded by this gate.
- No service inventory row is added, removed, or changed.
- No provider response, manifest claim, test report, registry entry, or local
  attestation grants execution by itself.
- A host-side `raios.computed_capability_grant.v0` diagnostic is evidence, not a
  loader token.
- A valid `module.grant_diagnostic` hash reference is read-only evidence, not a
  loader token.
- A valid `module.service_slot_diagnostic` reservation reference is read-only
  evidence, not a service inventory row or allocated slot.
- A retained `raios.module_computed_grant_reference.v0` event binding is
  current-boot diagnostic evidence only; it is not durable audit authority.
- `raios.module_load_gate_audit_rollback_requirements.v0` is a requirement
  shape only; it does not prove that durable audit or rollback state exists.
- A valid `module.audit_rollback_diagnostic` hash reference may be retained as
  `raios.module_audit_rollback_reference.v0` current-boot diagnostic evidence;
  it is not durable audit or rollback authority.
- A retained audit/rollback reference must pass the live load-gate predicate
  before its hashes appear as accepted audit/rollback evidence in
  `module.load_ephemeral` or `service.load_ephemeral`.
- A retained service-slot reservation must pass the live load-gate predicate
  before its reservation hash appears as accepted service-slot evidence in
  `module.load_ephemeral` or `service.load_ephemeral`.
- `module.load_gate_audit_rollback_selftest` is test infrastructure and must
  not create retained reference records, audit records, rollback plans, service
  slots, loader state, or service inventory changes.
- A valid `raios.module_manifest.v0` is only one input to a future computed
  grant.
- The normal module gate does not authorize recovery artifacts; recovery loads
  use `raios.recovery.v0` and a separate recovery trust boundary.
- A future positive path must keep denial reasons explicit when any evidence is
  missing, stale, mismatched, or outside the current boot scope.
