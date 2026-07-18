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
    "module_manifest": "missing | retained_hash_reference_only | rejected_retained_reference",
    "candidate_artifact": "missing | retained_hash_reference_only | rejected_retained_reference",
    "vm_test_report": "missing | retained_hash_reference_only | rejected_retained_reference",
    "local_attestation": "missing | retained_hash_reference_only | rejected_retained_reference",
    "computed_capability_grant": "missing | retained_hash_reference_only",
    "local_approval": "missing | retained_hash_reference_only | rejected_retained_reference",
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
retained_module_manifest_reference_not_authorizing
retained_module_manifest_reference_stale_or_dropped_event_id
retained_module_manifest_reference_wrong_schema_or_variant
retained_module_manifest_reference_substituted_record
retained_module_manifest_reference_hash_mismatch
retained_module_manifest_reference_computed_grant_mismatch
candidate_artifact_missing
retained_candidate_artifact_reference_not_authorizing
retained_candidate_artifact_reference_stale_or_dropped_event_id
retained_candidate_artifact_reference_wrong_schema_or_variant
retained_candidate_artifact_reference_substituted_record
retained_candidate_artifact_reference_hash_mismatch
retained_candidate_artifact_reference_manifest_reference_mismatch
retained_candidate_artifact_reference_computed_grant_reference_mismatch
retained_candidate_artifact_hash_mismatch
vm_test_report_missing
retained_vm_test_report_reference_not_authorizing
retained_vm_test_report_reference_stale_or_dropped_event_id
retained_vm_test_report_reference_wrong_schema_or_variant
retained_vm_test_report_reference_substituted_record
retained_vm_test_report_reference_hash_mismatch
retained_vm_test_report_reference_manifest_reference_mismatch
retained_vm_test_report_reference_artifact_reference_mismatch
retained_vm_test_report_reference_computed_grant_reference_mismatch
retained_vm_test_report_hash_mismatch
local_attestation_missing
retained_local_attestation_reference_not_authorizing
retained_local_attestation_reference_stale_or_dropped_event_id
retained_local_attestation_reference_previous_boot_or_unretained
retained_local_attestation_reference_wrong_schema_or_variant
retained_local_attestation_reference_substituted_record
retained_local_attestation_reference_hash_mismatch
retained_local_attestation_reference_manifest_reference_mismatch
retained_local_attestation_reference_artifact_reference_mismatch
retained_local_attestation_reference_vm_report_reference_mismatch
retained_local_attestation_reference_computed_grant_reference_mismatch
local_approval_missing
retained_local_approval_reference_not_authorizing
retained_local_approval_reference_stale_or_dropped_event_id
retained_local_approval_reference_previous_boot_or_unretained
retained_local_approval_reference_wrong_schema_or_variant
retained_local_approval_reference_substituted_record
retained_local_approval_reference_hash_mismatch
retained_local_approval_reference_manifest_reference_mismatch
retained_local_approval_reference_artifact_reference_mismatch
retained_local_approval_reference_vm_report_reference_mismatch
retained_local_approval_reference_local_attestation_reference_mismatch
retained_local_approval_reference_computed_grant_reference_mismatch
computed_capability_grant_missing
retained_computed_grant_reference_not_authorizing
durable_audit_write_missing
rollback_install_missing
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

The current guest accepts some of these only as canonical hash references and
current-boot event ids. Host-side manifest validation, VM reports, registry
evidence, and local attestations remain evidence artifacts only until an
in-guest policy path computes a grant; raw JSON or artifact bytes are not
trusted authority.

The read-only `module.manifest_diagnostic` method can inspect only a canonical
`raios.module_manifest_reference.v0` hash reference. A valid reference is
retained as local-only current-boot event evidence, but it still keeps
`authorizes_guest_load: false`, `can_load_now: false`, and
`load_attempted: false`. The mutating `module.load_ephemeral` gate snapshots
that retained reference only after live current-boot event validation and
reports `module_manifest: retained_hash_reference_only` with reason
`retained_module_manifest_reference_not_authorizing`.

The read-only `module.artifact_diagnostic` method can inspect only a canonical
`raios.module_candidate_artifact_reference.v0` hash reference. A valid reference
is retained as local-only current-boot event evidence, but it still accepts no
artifact bytes, loads no artifact, and keeps `load_attempted: false`. The
mutating `module.load_ephemeral` gate snapshots that retained reference only
after live current-boot event validation against the retained manifest and
computed-grant references, then reports
`candidate_artifact: retained_hash_reference_only` with reason
`retained_candidate_artifact_reference_not_authorizing`.

The read-only `module.vm_report_diagnostic` method can inspect only a canonical
`raios.module_vm_test_report_reference.v0` hash reference. A valid reference is
retained as local-only current-boot event evidence, but it accepts no
VM-report JSON, loads no artifact, and keeps `load_attempted: false`. The
mutating `module.load_ephemeral` gate snapshots that retained reference only
after live current-boot event validation against the retained manifest,
candidate-artifact, and computed-grant references, then reports
`vm_test_report: retained_hash_reference_only` with reason
`retained_vm_test_report_reference_not_authorizing`.

The read-only `module.attestation_diagnostic` method can inspect only a
canonical `raios.module_local_attestation_reference.v0` hash reference. A valid
reference is retained as local-only current-boot event evidence, but it accepts
no local-attestation JSON, loads no artifact, and keeps `load_attempted: false`.
The mutating `module.load_ephemeral` gate snapshots that retained reference
only after live current-boot event validation against the retained manifest,
candidate-artifact, VM-report, and computed-grant references, then reports
`local_attestation: retained_hash_reference_only` with reason
`retained_local_attestation_reference_not_authorizing`.

The read-only `module.approval_diagnostic` method can inspect only a canonical
`raios.module_local_approval_reference.v0` hash reference. A valid reference is
retained as local-only current-boot event evidence, but it accepts no approval
text, loads no artifact, and keeps `load_attempted: false`. The mutating
`module.load_ephemeral` gate snapshots that retained reference only after live
current-boot event validation against the retained manifest, candidate-artifact,
VM-report, local-attestation, and computed-grant references, then reports
`local_approval: retained_hash_reference_only` with reason
`retained_local_approval_reference_not_authorizing`.

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
      "local_approval_hash",
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
  "retained_local_approval_reference_event_id": "null | event.current_boot.00000032",
  "retained_audit_rollback_reference_event_id": "null | event.current_boot.00000033"
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
  module_manifest_reference_checked
  candidate_artifact_reference_checked
  vm_test_report_reference_checked
  local_attestation_reference_checked
  local_approval_reference_checked
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
retained_module_manifest_reference.state: missing | present | rejected
retained_module_manifest_reference.schema: raios.module_manifest_reference.v0
retained_candidate_artifact_reference.state: missing | present | rejected
retained_candidate_artifact_reference.schema: raios.module_candidate_artifact_reference.v0
retained_vm_test_report_reference.state: missing | present | rejected
retained_vm_test_report_reference.schema: raios.module_vm_test_report_reference.v0
retained_local_attestation_reference.state: missing | present | rejected
retained_local_attestation_reference.schema: raios.module_local_attestation_reference.v0
retained_local_approval_reference.state: missing | present | rejected
retained_local_approval_reference.schema: raios.module_local_approval_reference.v0
retained_computed_grant_reference.state: missing | present
retained_audit_rollback_reference.state: missing | present | rejected
retained_audit_rollback_reference.schema: raios.module_audit_rollback_reference.v0
retained_service_slot_reservation.state: missing | present | rejected
retained_service_slot_reservation.schema: raios.module_service_slot_reservation.v0
computed_capability_grant_hash: null | sha256:<retained grant hash>
manifest_reference_hash: null | sha256:<retained manifest reference hash>
manifest_hash: null | sha256:<retained manifest hash>
artifact_reference_hash: null | sha256:<retained artifact reference hash>
artifact_hash: null | sha256:<retained artifact hash>
vm_test_report_reference_hash: null | sha256:<retained VM-report reference hash>
vm_test_report_hash: null | sha256:<retained report hash>
local_attestation_reference_hash: null | sha256:<retained local-attestation reference hash>
local_attestation_hash: null | sha256:<retained attestation hash>
local_approval_reference_hash: null | sha256:<retained local-approval reference hash>
local_approval_hash: null | sha256:<retained approval hash>
audit_record_hash: null | sha256:<retained audit record hash>
rollback_plan_hash: null | sha256:<retained rollback plan hash>
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

## Manifest Reference Gate Selftest

The read-only method:

```text
agent module.load_gate_manifest_selftest
```

emits `raios.module_load_gate_manifest_selftest.v0`. It is local test
infrastructure over the retained manifest-reference predicate and must report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_retained_manifest_reference_records: false
accepts_manifest_json: false
accepts_artifact_bytes: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing retained manifest reference,
accepted-current-boot-but-denied, stale/dropped event id,
previous-boot-or-unretained event id, wrong schema or variant, substituted
record, and manifest-reference hash mismatch.

## Artifact Reference Gate Selftest

The read-only method:

```text
agent module.load_gate_artifact_selftest
```

emits `raios.module_load_gate_artifact_selftest.v0`. It is local test
infrastructure over the retained candidate-artifact predicate and must report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_retained_candidate_artifact_reference_records: false
accepts_artifact_bytes: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing retained artifact reference,
accepted-current-boot-but-denied, stale/dropped event id,
previous-boot-or-unretained event id, wrong schema or variant, substituted
record, artifact-reference hash mismatch, manifest-reference mismatch, and
computed-grant-reference mismatch.

## VM Test Report Reference Gate Selftest

The read-only method:

```text
agent module.load_gate_vm_report_selftest
```

emits `raios.module_load_gate_vm_report_selftest.v0`. It is local test
infrastructure over the retained VM-test-report predicate and must report:

```text
test_infrastructure: true
mutates_global_event_log: false
creates_retained_vm_test_report_reference_records: false
accepts_vm_report_json: false
accepts_artifact_bytes: false
loads_artifact: false
service_inventory_change: none
load_attempted: false
can_load: false
```

The current cases cover missing retained VM-test-report reference,
accepted-current-boot-but-denied, stale/dropped event id,
previous-boot-or-unretained event id, wrong schema or variant, substituted
record, VM-test-report-reference hash mismatch, manifest-reference mismatch,
artifact-reference mismatch, computed-grant-reference mismatch, and
VM-test-report hash mismatch.

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

## VM Test Report Hash-Reference Diagnostic

The read-only method:

```text
agent module.vm_report_diagnostic
agent module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.vm_report_diagnostic_selftest
```

emits `raios.module_vm_test_report_reference_diagnostic.v0` and
`raios.module_vm_test_report_reference_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids. A valid command records a
local-only `raios.module_vm_test_report_reference.v0` event binding with
`global_event_log_mutation: valid_hash_reference_retention_only`, keeps
`accepts_vm_report_json: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`,
`can_load_now: false`, and `load_attempted: false`. The selftest covers absent,
accepted-current-boot, stale, mismatched, computed-grant-mismatched, and
non-current-boot event-id references without mutating the global event log.

## Local Attestation Hash-Reference Diagnostic

The read-only method:

```text
agent module.attestation_diagnostic
agent module.attestation_diagnostic <local_attestation_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]
agent module.attestation_diagnostic_selftest
```

emits `raios.module_local_attestation_reference_diagnostic.v0` and
`raios.module_local_attestation_reference_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids. A valid command records a
local-only `raios.module_local_attestation_reference.v0` event binding with
`global_event_log_mutation: valid_hash_reference_retention_only`, keeps
`accepts_local_attestation_json: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`,
`can_load_now: false`, and `load_attempted: false`. The selftest covers absent,
accepted-current-boot, stale, mismatched, computed-grant-mismatched, and
non-current-boot event-id references without mutating the global event log.

## Local Attestation Reference Gate Selftest

The read-only method:

```text
agent module.load_gate_attestation_selftest
```

emits `raios.module_load_gate_local_attestation_selftest.v0`. It is local test
infrastructure over the retained local-attestation-reference predicate and must
keep `mutates_global_event_log: false`,
`creates_retained_local_attestation_reference_records: false`,
`accepts_local_attestation_json: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`, `can_load: false`,
and `load_attempted: false`. The current cases cover missing,
accepted-current-boot-but-denied, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted, hash-mismatch,
manifest-reference mismatch, artifact-reference mismatch, VM-report-reference
mismatch, and computed-grant-reference mismatch candidates.

## Local Approval Hash-Reference Diagnostic

The read-only method:

```text
agent module.approval_diagnostic
agent module.approval_diagnostic <local_approval_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_local_attestation_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <local_attestation_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> [current_boot]
agent module.approval_diagnostic_selftest
```

emits `raios.module_local_approval_reference_diagnostic.v0` and
`raios.module_local_approval_reference_diagnostic_selftest.v0`. It validates
only canonical hashes and current-boot ids. A valid command records a
local-only `raios.module_local_approval_reference.v0` event binding with
`global_event_log_mutation: valid_hash_reference_retention_only`, keeps
`accepts_local_approval_text: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`,
`can_load_now: false`, and `load_attempted: false`. The selftest covers absent,
accepted-current-boot, stale, mismatched, computed-grant-mismatched, and
non-current-boot event-id references without mutating the global event log.

## Local Approval Reference Gate Selftest

The read-only method:

```text
agent module.load_gate_approval_selftest
```

emits `raios.module_load_gate_local_approval_selftest.v0`. It is local test
infrastructure over the retained local-approval-reference predicate and must
keep `mutates_global_event_log: false`,
`creates_retained_local_approval_reference_records: false`,
`accepts_local_approval_text: false`, `accepts_artifact_bytes: false`,
`loads_artifact: false`, `service_inventory_change: none`, `can_load: false`,
and `load_attempted: false`. The current cases cover missing,
accepted-current-boot-but-denied, stale/dropped,
previous-boot-or-unretained, wrong-schema, substituted, hash-mismatch,
manifest-reference mismatch, artifact-reference mismatch, VM-report-reference
mismatch, local-attestation-reference mismatch, and computed-grant-reference
mismatch candidates.

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

## Audit/Rollback Write Boundary Diagnostic

The read-only method:

```text
agent module.audit_rollback_availability
agent module.audit_rollback_availability_selftest
agent module.audit_rollback_write_policy
agent module.audit_rollback_write_policy_selftest
agent module.audit_rollback_storage_layout
agent module.audit_rollback_storage_layout_selftest
agent module.audit_rollback_append_engine
agent module.audit_rollback_append_engine_selftest
agent module.audit_rollback_append_contract
agent module.audit_rollback_append_contract_selftest
agent module.audit_rollback_write_boundary
agent module.audit_rollback_write_boundary_selftest
```

`module.audit_rollback_availability` emits
`raios.module_audit_rollback_availability.v0` and
`module.audit_rollback_availability_selftest` emits
`raios.module_audit_rollback_availability_selftest.v0`. The live diagnostic
reports typed `raios.durable_audit_ledger.v0` and `raios.rollback_store.v0`
current-boot availability facts as missing, local-only, non-durable, and
non-authorizing. The selftest covers missing ledger/store, previous-boot, schema
mismatch, missing provenance, and available-facts-but-policy-missing cases.

`module.audit_rollback_write_policy` emits
`raios.module_audit_rollback_write_policy.v0` and
`module.audit_rollback_write_policy_selftest` emits
`raios.module_audit_rollback_write_policy_selftest.v0`. The live diagnostic
reports typed `raios.durable_audit_write_policy.v0` and
`raios.rollback_install_policy.v0` current-boot policy facts as missing,
local-only, non-durable, and non-authorizing. The selftest covers missing
policy pairs, previous-boot, schema mismatch, missing provenance, retained
evidence binding gaps, availability binding gaps, and
available-policy-but-writer-missing cases.

`module.audit_rollback_storage_layout` emits
`raios.module_audit_rollback_storage_layout.v0` and
`module.audit_rollback_storage_layout_selftest` emits
`raios.module_audit_rollback_storage_layout_selftest.v0`. The live diagnostic
reports typed `raios.persistence_device_inventory.v0` and
`raios.audit_rollback_storage_layout.v0` current-boot facts as missing,
local-only, non-durable, and non-authorizing. It separates persistence device
identity, partition inventory, write-path availability, audit-ledger and
rollback-store layout regions, append slots, and recovery-region separation
from write or append authority.

`module.audit_rollback_append_engine` emits
`raios.module_audit_rollback_append_engine.v0` and
`module.audit_rollback_append_engine_selftest` emits
`raios.module_audit_rollback_append_engine_selftest.v0`. The live diagnostic
reports typed `raios.audit_ledger_append_engine.v0` and
`raios.rollback_store_transaction_engine.v0` current-boot facts as missing,
local-only, non-durable, and non-authorizing. It consumes the storage-layout
diagnostic as input while keeping append-only behavior, flush support, replay
support, storage-layout binding, write-policy binding, and recovery separation
separate from write authority.

`module.audit_rollback_append_contract` emits
`raios.module_audit_rollback_append_contract.v0` and
`module.audit_rollback_append_contract_selftest` emits
`raios.module_audit_rollback_append_contract_selftest.v0`. The live diagnostic
reports typed `raios.audit_ledger_append_envelope.v0` and
`raios.rollback_store_transaction_envelope.v0` current-boot facts as missing,
local-only, non-durable, and non-authorizing. It consumes the storage-layout
and append-engine diagnostics separately from availability and policy, and the
selftest covers missing, stale/scope, schema, provenance, explicit stable-id
binding, storage-layout, and append-engine-missing cases. Future append
envelopes must bind storage-layout, append-engine, write-policy, availability,
and provenance ids before any write path can evaluate them as structurally
ready.

`module.audit_rollback_write_boundary` emits
`raios.module_audit_rollback_write_boundary.v0` and
`module.audit_rollback_write_boundary_selftest` emits
`raios.module_audit_rollback_write_boundary_selftest.v0`. It consumes the
retained current-boot manifest, candidate-artifact, VM-report, computed-grant,
local-attestation, local-approval, audit/rollback, service-slot reservation, and
audit/rollback availability plus write-policy, storage-layout, append-engine
readiness through the append contract, and append-contract facts through
the same live gate snapshot used by denied `module.load_ephemeral`. It emits a typed
`raios.module_pre_load_audit_rollback_write_request.v0` plus
`raios.module_audit_rollback_write_denial_evidence.v0`.

The diagnostic must keep:

```text
mutates_global_event_log: false
writes_enabled: false
creates_durable_audit_records: false
creates_rollback_plans: false
installs_rollback_plan: false
allocates_service_slot: false
loads_artifact: false
loads_recovery_artifact: false
service_inventory_change: none
load_attempted: false
```

A fully valid retained evidence chain still returns
`validation_status: denied_missing_durable_write_boundary` with
`durable_audit_write_missing`, `rollback_install_missing`,
`storage_layout_missing`, and `append_engine_missing`. Retained hash references
remain current-boot evidence only; they are not durable audit authority,
rollback-store authority, append authority, loader authority, or recovery
artifact authority. The selftest covers missing, stale, substituted,
previous-boot, wrong-schema, mismatched hash, service-slot mismatch,
recovery-artifact separation, one-sided missing availability,
available-facts-but-policy-missing, rollback-policy-missing,
append-contract-missing, writer-unimplemented, and
accepted-current-boot-but-denied candidates without creating durable records or
loading artifacts.

## Invariants

- No artifact bytes are loaded by this gate.
- No service inventory row is added, removed, or changed.
- No provider response, manifest claim, test report, registry entry, or local
  attestation/local-approval record grants execution by itself.
- A host-side `raios.computed_capability_grant.v0` diagnostic is evidence, not a
  loader token.
- A valid `module.grant_diagnostic` hash reference is read-only evidence, not a
  loader token.
- A valid `module.vm_report_diagnostic` hash reference is read-only evidence,
  not VM-report JSON and not a loader token.
- A retained `raios.module_vm_test_report_reference.v0` event binding is only a
  current-boot hash reference; it is not report content, not a signature, and
  not load authority.
- A valid `module.attestation_diagnostic` hash reference is read-only evidence,
  not local-attestation JSON and not a loader token.
- A retained `raios.module_local_attestation_reference.v0` event binding is
  only a current-boot hash reference; it is not attestation content, not a
  signature, and not load authority.
- A valid `module.approval_diagnostic` hash reference is read-only evidence,
  not free-form approval text and not a loader token.
- A retained `raios.module_local_approval_reference.v0` event binding is only a
  current-boot hash reference; it is not durable consent and not load authority.
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
- `module.audit_rollback_write_boundary` is read-only pre-load denial evidence;
  it does not create durable audit records, install rollback plans, allocate
  service slots, load normal artifacts, or load recovery artifacts.
- `module.audit_rollback_availability` reports availability facts only; missing
  durable audit-ledger and rollback-store facts must not be silently replaced by
  fake durable records or fallback stores.
- `module.audit_rollback_write_policy` reports policy facts only; missing
  durable-write and rollback-install policies must not be silently treated as
  writer authority.
- `module.audit_rollback_storage_layout` reports persistence-device and
  storage-layout facts only; missing device identity, partition inventory,
  layout regions, append slots, or recovery separation must not be silently
  treated as storage or append authority.
- `module.audit_rollback_append_engine` reports append-engine readiness facts
  only; missing append-only, flush, replay, storage-layout binding,
  write-policy binding, or recovery separation must not be silently treated as
  writer authority.
- `module.audit_rollback_append_contract` reports append-envelope and
  storage-layout plus append-engine readiness facts only; missing append
  contracts, storage layouts, append engines, stable-id bindings, or provenance
  bindings must not be silently treated as writer authority.
- `module.load_gate_audit_rollback_selftest` is test infrastructure and must
  not create retained reference records, audit records, rollback plans, service
  slots, loader state, or service inventory changes.
- A valid `raios.module_manifest.v0` is only one input to a future computed
  grant.
- A valid `raios.module_manifest_reference.v0` is only a current-boot hash
  reference. It is not manifest content, not a signature, and not load
  authority.
- The normal module gate does not authorize recovery artifacts; recovery loads
  use `raios.recovery.v0` and a separate recovery trust boundary.
- A future positive path must keep denial reasons explicit when any evidence is
  missing, stale, mismatched, or outside the current boot scope.
