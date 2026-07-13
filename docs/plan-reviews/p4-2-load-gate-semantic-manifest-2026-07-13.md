# P4-2a — load-gate semantic manifest

Read-only inventory for P4-2b. No emitter, evaluator, harness, or existing
document is changed by this packet.

Notation:

- `R` = legacy `body.result`, except that the denied response currently places
  its fields directly in `body`; those fields are still written as `R.*` below.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

## 1. Response-path inventory

### `module.load_ephemeral` denied response

Emitter: `emit_module_load_ephemeral_denied()` in
`seed-kernel/src/agent_protocol_module_load_gate_render.rs:5332`.

Framing is:

```text
RAIOS_AGENT_BEGIN <method>
{
  v, t, id,
  body: { ... }
}
RAIOS_AGENT_END <method>
```

Unlike the normal response helper, this error response has no `body.result`.
The current body field order is:

```text
R:
 method, event_id, audit_event_id, code, schema, message,
 request, gate_state,
 retained_module_manifest_reference,
 retained_candidate_artifact_reference,
 receiver_identity_load_preflight,
 retained_vm_test_report_reference,
 retained_local_attestation_reference,
 retained_local_approval_reference,
 retained_computed_grant_reference,
 retained_audit_rollback_reference,
 retained_service_slot_reservation,
 service_slot_allocator_readiness,
 loader_runtime_readiness,
 audit_rollback_requirements,
 blocked_by, required, evidence
```

Nested order:

```text
request:
 load_mode, requested_capability, risk, target, subject

gate_state:
 module_manifest, candidate_artifact, vm_test_report,
 local_attestation, computed_capability_grant, local_approval,
 rollback_plan, durable_audit_record, service_slot,
 service_slot_allocator, loader_runtime, loader,
 artifact_loaded, service_started, persistence, can_load

retained_module_manifest_reference:
 state, retention, event_id, schema, status, [reason], classification,
 authorizes_guest_load, can_load_now, load_attempted,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, service_inventory_change,
 hashes {manifest_reference_hash, manifest_hash}

retained_candidate_artifact_reference:
 common retained prefix above,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, service_inventory_change,
 retained_manifest_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {artifact_reference_hash, manifest_reference_hash, manifest_hash,
         computed_capability_grant_hash, artifact_hash,
         vm_test_report_hash, local_attestation_hash}

retained_vm_test_report_reference:
 common retained prefix,
 accepts_manifest_json, accepts_artifact_bytes, accepts_vm_report_json,
 accepts_unsigned_service_code, service_inventory_change,
 retained_manifest_reference_event_id,
 retained_candidate_artifact_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {vm_test_report_reference_hash, manifest_reference_hash,
         artifact_reference_hash, manifest_hash, artifact_hash,
         computed_capability_grant_hash, vm_test_report_hash,
         local_attestation_hash}

retained_local_attestation_reference:
 common retained prefix,
 accepts_local_attestation_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, service_inventory_change,
 retained_manifest_reference_event_id,
 retained_candidate_artifact_reference_event_id,
 retained_vm_test_report_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {local_attestation_reference_hash, manifest_reference_hash,
         artifact_reference_hash, vm_test_report_reference_hash,
         manifest_hash, artifact_hash, computed_capability_grant_hash,
         vm_test_report_hash, local_attestation_hash}

retained_local_approval_reference:
 common retained prefix,
 accepts_local_approval_text, accepts_artifact_bytes,
 accepts_unsigned_service_code, service_inventory_change,
 retained_manifest_reference_event_id,
 retained_candidate_artifact_reference_event_id,
 retained_vm_test_report_reference_event_id,
 retained_local_attestation_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {local_approval_reference_hash, manifest_reference_hash,
         artifact_reference_hash, vm_test_report_reference_hash,
         local_attestation_reference_hash, manifest_hash, artifact_hash,
         computed_capability_grant_hash, vm_test_report_hash,
         local_attestation_hash, local_approval_hash}

retained_computed_grant_reference:
 state, retention, event_id, schema, status, [reason], classification,
 grants_capability, grants_load_now, authorizes_guest_load,
 can_load_now, load_attempted,
 hashes {computed_capability_grant_hash, manifest_hash, artifact_hash,
         vm_test_report_hash, local_attestation_hash}

retained_audit_rollback_reference:
 state, retention, event_id, schema, status, [reason], classification,
 durable_audit_written, rollback_plan_installed, can_load_now,
 load_attempted, grants_capability, grants_load_now,
 authorizes_guest_load, denial_event_id,
 retained_computed_grant_reference_event_id, ram_only_service_slot_id,
 hashes {audit_record_hash, rollback_plan_hash,
         computed_capability_grant_hash, manifest_hash, artifact_hash,
         vm_test_report_hash, local_attestation_hash, local_approval_hash,
         pre_load_service_inventory_hash, cleanup_actions_hash}

retained_service_slot_reservation:
 state, retention, event_id, schema, status, [reason], classification,
 allocates_service_slot, creates_service_inventory_records,
 can_load_now, load_attempted, grants_capability, grants_load_now,
 authorizes_guest_load,
 retained_computed_grant_reference_event_id,
 retained_audit_rollback_reference_event_id, ram_only_service_slot_id,
 hashes {reservation_hash, computed_capability_grant_hash,
         audit_record_hash, rollback_plan_hash,
         pre_load_service_inventory_hash}
```

The bracketed `reason` is conditional today. V1 must emit it explicitly; it
may not preserve omission.

The following larger projections preserve the stated order:

```text
receiver_identity_load_preflight:
 present, source_id, entry_id, status, reason, content_sha256,
 retained_part_count, receiver_identity_retained,
 receiver_identity_complete, guest_signature_verification_performed,
 retained_candidate_sha256, retained_candidate_present,
 retained_candidate_wasm_valid, catalog_finalize_candidate_sha256,
 retained_candidate_matches_catalog_finalize, preflight_evaluated,
 accepted, rejected, missing_gate_count,
 m6_reverification_gate_satisfied, m7_loader_policy_gate_satisfied,
 provider_trust_gate_satisfied, owner_seal_gate_satisfied,
 requires_m6_m7_reverify_for_load, requires_provider_trust_for_load,
 requires_owner_seal_for_load, can_load_now, load_authorized,
 install_authorized, load_attempted, execution_attempted,
 durable_write_attempted, authorizes_acquisition, authorizes_install,
 authorizes_load, authorizes_execute, authorizes_persist,
 writes_persistent_state, network_attempted, owner_sealed, trust_tier

service_slot_allocator_readiness:
 schema, scope, classification, source_method, state, readiness_status,
 readiness_reason, allocator_authority_boundary,
 allocation_intent_boundary, authority_input_boundaries,
 authority_decision, registry_write_commit_gate,
 source_evidence_event_id, status, reason,
 allocates_service_slot, creates_service_inventory_records,
 service_inventory_change, load_attempted,
 service_slot_allocator_ready, starts_service, marks_service_running,
 can_load_now

loader_runtime_readiness:
 schema, scope, classification, state, readiness_status, readiness_reason,
 retained_module_evidence_state, retained_module_evidence_reason,
 service_slot_allocator_ready,
 execution_commit_gate, descriptor_intake_boundary,
 artifact_byte_intake_boundary, execution_authorization_boundary,
 service_registry_mutation_boundary, live_load_boundary,
 accepts_loader_descriptor, accepts_descriptor_bytes,
 produces_parsed_descriptor, validates_descriptor_schema,
 produces_validated_descriptor, validates_descriptor_capabilities,
 produces_capability_validated_descriptor,
 authorizes_executable_load_plan, produces_executable_load_plan,
 produces_executable_image_layout, produces_executable_page_mapping_plan,
 binds_capability_validated_descriptor_to_executable_pages,
 parses_descriptor_bytes, accepts_artifact_bytes, loads_artifact,
 allocates_service_slot, creates_service_inventory_records,
 service_inventory_change, starts_service, marks_service_running,
 creates_service_health_records, writes_service_start_audit_record,
 unloads_service, cleans_up_service_slot, commits_live_load,
 writes_load_commit_audit_record, installs_commit_rollback_record,
 records_load_result, can_load_now, load_attempted,
 missing_facts, source_fact_count, source_fact_map_complete,
 source_fact_map[], loader_runtime_facts,
 m6_m7_reverify_input_check
```

`source_fact_map[]` order is the declared runtime-fact source order. Each item
is `fact, schema, id, source_method, source_fact_locator, missing_reason,
status, reason/present where supplied, present, authorizes_load`. The receiver
preflight item adds `receiver_identity_complete,
retained_candidate_matches_catalog_finalize, preflight_evaluated,
requires_m6_m7_reverify_for_load, can_load_now` before `authorizes_load`.

`loader_runtime_facts` uses the source fact name as its property key. Each fact
record preserves `schema, id, source_method, source_fact_locator, scope,
classification, status, reason, present`, followed by that fact's typed
descriptive fields. `receiver_identity_load_preflight` additionally carries
the receiver/hash/preflight fields listed in the source at
`agent_protocol_module_load_gate_render.rs:5245-5319`.

All loader boundary objects use this ordered grammar:

```text
schema, id, source_evidence_event_id, status, reason, present,
source_chain_complete and boundary-specific *_present/*_source_chain_complete,
requested_capability, load_mode, target,
descriptive accepts/produces/validates/source-presence fields,
authority/effect booleans
```

The boundary properties occur in this order:

```text
execution_commit_gate, descriptor_intake_boundary,
artifact_byte_intake_boundary, execution_authorization_boundary,
service_registry_mutation_boundary,
load_attempt_boundary, artifact_load_boundary,
executable_mapping_boundary, entrypoint_transfer_boundary,
service_start_boundary, service_health_binding_boundary,
service_running_state_boundary, service_start_audit_boundary,
service_unload_cleanup_boundary, live_load_commit_boundary,
commit_audit_boundary, commit_rollback_boundary, commit_result_boundary,
descriptor_acceptance_authority_boundary,
descriptor_parser_contract_boundary, descriptor_parser_result_boundary,
descriptor_schema_validation_boundary,
descriptor_capability_validation_boundary, descriptor_load_plan_boundary,
executable_load_plan_authority_boundary,
executable_load_plan_result_boundary, executable_image_layout_boundary,
executable_page_mapping_plan_boundary, executable_page_mapping_boundary,
descriptor_executable_page_binding_boundary,
executable_entrypoint_binding_boundary,
executable_entrypoint_transfer_authorization_boundary,
executable_entrypoint_transfer_boundary,
executable_entrypoint_handoff_boundary,
executable_entrypoint_invocation_boundary
```

The audit/rollback projection order is:

```text
audit_rollback_requirements:
 schema, classification, status, writes_enabled,
 creates_durable_audit_records, creates_rollback_plans,
 durable_audit_record {schema, state, durability, required_bindings[]},
 rollback_plan {schema, state, must_preexist_load, required_bindings[]},
 required_hashes {
   computed_capability_grant_hash, local_attestation_reference_hash,
   local_attestation_hash, local_approval_reference_hash,
   local_approval_hash, vm_test_report_reference_hash,
   vm_test_report_hash, artifact_reference_hash, artifact_hash,
   manifest_reference_hash, manifest_hash, audit_record_hash,
   rollback_plan_hash, pre_load_service_inventory_hash,
   cleanup_actions_hash, ram_only_service_slot_id,
   service_slot_reservation_hash
 },
 retained_reference_event_id, retained_manifest_reference_event_id,
 retained_local_attestation_reference_event_id,
 retained_local_approval_reference_event_id,
 retained_audit_rollback_reference_event_id,
 retained_service_slot_reservation_event_id,
 local_approval {state, reason, required, authorizes_guest_load},
 ram_only_service_slot {state, reason, required, allocates_service_slot},
 load_attempted, service_inventory_change, can_load

blocked_by[]: gate, state, reason

required[]: exact 50-string legacy prerequisite catalog in source order

evidence:
 denial_event_id, event_scope,
 computed_capability_grant_hash, local_attestation_reference_hash,
 local_attestation_hash, local_approval_reference_hash,
 local_approval_hash, vm_test_report_reference_hash,
 vm_test_report_hash, artifact_reference_hash, artifact_hash,
 manifest_reference_hash, manifest_hash, audit_record_hash,
 rollback_plan_hash, pre_load_service_inventory_hash,
 cleanup_actions_hash, ram_only_service_slot_id,
 service_slot_reservation_hash,
 service_inventory_change, load_attempted
```

### Event binding

`emit_module_load_gate_event_binding()` emits a direct object with no
BEGIN/END framing. Its order is:

```text
schema, status, load_mode, requested_capability, risk, target, subject,
gate_state,
retained_module_manifest_reference,
retained_candidate_artifact_reference,
retained_vm_test_report_reference,
retained_local_attestation_reference,
retained_local_approval_reference,
retained_computed_grant_reference,
retained_audit_rollback_reference,
retained_service_slot_reservation,
service_slot_allocator_readiness, loader_runtime_readiness,
audit_rollback_requirements, blocked_by, required, evidence
```

Its nested objects are compact renderings of the same values and order as the
denied response, except it has no separately emitted
`receiver_identity_load_preflight`; that fact is inside loader readiness.

### Selftest responses

All nine methods use `begin_response()`/`end_response()` and therefore:

```text
RAIOS_AGENT_BEGIN <method>
{v, t, id, body:{method, result:{...}}}
RAIOS_AGENT_END <method>
```

Methods and ordered `R` fields:

```text
module.load_gate_manifest_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, creates_retained_manifest_reference_records,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, loads_artifact,
 service_inventory_change, load_attempted, loader,
 case_count, passed, required_bindings, cases[], can_load

module.load_gate_artifact_selftest:
 same order, substituting creates_retained_candidate_artifact_reference_records

module.load_gate_vm_report_selftest:
 same order, substituting creates_retained_vm_test_report_reference_records
 and adding accepts_vm_report_json after accepts_artifact_bytes

module.load_gate_attestation_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log,
 creates_retained_local_attestation_reference_records,
 accepts_local_attestation_json, accepts_artifact_bytes, loads_artifact,
 service_inventory_change, load_attempted,
 case_count, passed, cases[], can_load

module.load_gate_approval_selftest:
 attestation order with creates_retained_local_approval_reference_records
 and accepts_local_approval_text

module.load_gate_retained_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, creates_retained_reference_records,
 loads_artifact, service_inventory_change, load_attempted,
 loader, service_slot, case_count, passed, cases[], can_load

module.load_gate_audit_rollback_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, creates_durable_audit_records,
 creates_rollback_plans, allocates_service_slot, loads_artifact,
 service_inventory_change, load_attempted, loader, service_slot,
 case_count, passed, required_bindings, cases[], can_load

module.load_gate_service_slot_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, creates_service_slot_reservation_records,
 allocates_service_slot, creates_service_inventory_records,
 loads_artifact, service_inventory_change, load_attempted,
 loader, service_slot, case_count, passed, required_bindings,
 cases[], can_load

module.load_gate_loader_runtime_selftest:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, accepts_loader_descriptor,
 accepts_artifact_bytes, loads_artifact, allocates_service_slot,
 creates_service_inventory_records, service_inventory_change,
 load_attempted, service_slot_allocator_ready, loader,
 case_count, passed, required_bindings, missing_runtime_facts,
 source_fact_count, source_fact_map_complete, source_fact_map[],
 cases[], can_load
```

Normal cases preserve:

```text
case, expected_status, expected_reason, actual_status, actual_reason,
family-specific actual/accepted fields, passed, can_load, load_attempted
```

Service-slot cases add `allocates_service_slot` before the two common false
fields. Loader-runtime cases preserve:

```text
case, expected_status, expected_reason,
expected_retained_module_evidence_state,
expected_service_slot_allocator_state, expected_loader_runtime_state,
actual_status, actual_reason,
actual_retained_module_evidence_state,
actual_retained_module_evidence_reason,
actual_service_slot_allocator_state,
actual_service_slot_allocator_status,
actual_service_slot_allocator_reason, actual_loader_runtime_state,
passed, loads_artifact, allocates_service_slot,
creates_service_inventory_records, can_load, load_attempted
```

## 2. Semantic manifest

### Envelope and fixed response fields

```text
v -> retired(redundant with v1 schema)
t -> retired(redundant with D.outcome)
id:"serial" -> v1.id (typed response.current_boot.NNNNNNNN)
R.method -> source_method
R.event_id -> event_id
R.audit_event_id -> retired(duplicate of event_id)
R.code:"capability_denied" -> D.outcome="denied"
R.schema -> constant("raios.evidence_response.v1")
R.message -> F.message

v1.family -> constant("module.load_gate")
v1.scope -> constant("current_boot")
v1.classification -> constant("local_only")
```

The event binding is not a second authority decision. When P4-4 embeds the
binding as event evidence, the same mappings apply under that event record;
its provenance comes from the enclosing event.

### Request and descriptive facts

```text
R.request.* -> F.request.*
R.required[] -> F.required[] (same strings and order)
R.message -> F.message
R.gate_state.persistence -> F.runtime.persistence
R.gate_state.loader -> E[loader].facts.status_detail
R.<projection>.scope -> scope
R.<projection>.classification -> E[projection id].classification
R.<projection>.source_method -> E[projection id].facts.source_method
R.<projection>.id -> E[projection id].facts.record_id
R.<projection>.schema -> E[projection id].facts.record_schema
R.<projection>.source_evidence_event_id
    -> E[projection id].source_event_id
R.<projection>.source_fact_locator -> E[projection id].facts.source_fact_locator
R.<projection>.required_bindings[] -> E[projection id].facts.required_bindings[]
R.<projection>.missing_facts[] -> E[loader_runtime].facts.missing_facts[]
```

Every descriptive projection leaf not named in the decision/effects section
maps mechanically to `E[owning top-level gate].facts.<same nested path>`. This
rule covers receiver identity hashes/counts/statuses, source-fact maps,
`*_present`, `*_source_chain_complete`, retained-match facts, trust tier,
required bindings, runtime contract facts, and all validation/production
observations. Their current null/present behavior and array order survive.

`status`, `state`, `readiness_status`, and other family statuses map as:

```text
R.<gate status/state> -> E[id].facts.status_detail
common E[id].status -> present | missing | rejected | verified |
                       unavailable | not_applicable
R.<gate reason/readiness_reason> -> E[id].reason
```

No existing family-specific status string is discarded.

### Retained prerequisite records

The eight retained reference objects map mechanically:

```text
R.retained_*.state -> E[id].facts.state
R.retained_*.retention -> E[id].facts.retention
R.retained_*.event_id -> E[id].source_event_id
R.retained_*.schema -> E[id].facts.record_schema
R.retained_*.status -> E[id].facts.status_detail
R.retained_*.reason -> E[id].reason
R.retained_*.classification -> E[id].classification
R.retained_*.*_event_id -> E[id].facts.*_event_id
R.retained_*.hashes.* -> E[id].facts.*
R.retained_*.ram_only_service_slot_id -> E[id].facts.ram_only_service_slot_id
R.retained_*.denial_event_id -> E[id].facts.denial_event_id
R.retained_*.accepts_* -> E[id].facts.accepts_*
```

Evidence IDs are:

```text
module_manifest
candidate_artifact
vm_test_report
local_attestation
local_approval
computed_capability_grant
durable_audit_record
rollback_plan
service_slot
service_slot_allocator
loader_runtime
loader
```

The combined audit/rollback legacy object supplies facts to both
`E[durable_audit_record]` and `E[rollback_plan]`. The combined
`audit_rollback_requirements` object maps its audit fields to the former, its
rollback fields to the latter, and common hashes/provenance to both as typed
facts; it does not create a thirteenth gate.

Hash duplication in `R.evidence`, `R.audit_rollback_requirements.required_hashes`
and the retained objects maps to the corresponding prerequisite's single
evidence fact:

```text
R.evidence.denial_event_id -> event_id
R.evidence.event_scope -> scope
R.evidence.<reference/hash/slot field>
    -> E[owning prerequisite].facts.<same field>
```

### Ordered gate evidence and denial

The exact current denial order is:

```text
1  module_manifest
2  candidate_artifact
3  vm_test_report
4  local_attestation
5  local_approval
6  computed_capability_grant
7  durable_audit_record
8  rollback_plan
9  service_slot
10 service_slot_allocator
11 loader_runtime
12 loader
```

The ordering source is
`seed-kernel/src/agent_protocol_module_load_gate_render.rs:5428-5486`; the
compact event binding repeats it at lines 5619-5634. P4-2b must produce one
evidence record for each item in exactly this order, and derive
`D.blocked_by` from the same evaluator-owned slice without sorting:

```text
R.gate_state.<gate> -> E[gate].facts.status_detail + E[gate].status
R.blocked_by[].gate -> D.blocked_by[].evidence_id
R.blocked_by[].state -> D.blocked_by[].status and E[id].facts.status_detail
R.blocked_by[].reason -> D.blocked_by[].reason and E[id].reason
```

The legacy top-level reason is effectively `missing_evidence`; v1 `D.reason`
must instead equal the first evaluator-produced blocker reason, as required by
the existing `DenialDecision` model. See the decision-needed item below.

### Decision, grants, effects, and retirements

The denied load-gate response has the invariant:

```text
D.outcome -> constant("denied")
D.requested_capability -> F.request.requested_capability
D.grants -> constant([])
D.effects -> constant([])
```

All scattered authority and effect claims map as follows:

```text
can_load=false / can_load_now=false / load_authorized=false /
authorizes_load=false / authorizes_guest_load=false
    -> retired(single D.outcome="denied" and D.grants=[])

grants_capability=false / grants_load_now=false /
authorizes_allocation=false / authorizes_execution=false /
authorizes_registry_write=false / authorizes_descriptor_intake=false /
authorizes_artifact_byte_intake=false / authorizes_acquisition=false /
authorizes_install=false / authorizes_execute=false /
authorizes_persist=false / authorizes_executable_load_plan=false
    -> retired(D.grants=[]; D.effects=[])

load_attempted=false / execution_attempted=false /
durable_write_attempted=false / network_attempted=false /
loads_artifact=false / artifact_loaded=false / service_started=false /
starts_service=false / marks_service_running=false /
allocates_service_slot=false / creates_service_inventory_records=false /
creates_service_health_records=false / mutates_service_registry=false /
writes_durable_audit_state=false / writes_persistent_state=false /
writes_service_start_audit_record=false /
writes_load_commit_audit_record=false /
installs_rollback_state=false / installs_commit_rollback_record=false /
unloads_service=false / cleans_up_service_slot=false /
commits_live_load=false / records_load_result=false
    -> retired(D.effects=[])

service_inventory_change:"none"
    -> retired(D.effects excludes inventory mutation)

mutates_global_event_log=false in selftests
    -> F.safety.event_log_write_count=0
creates_retained_*_records=false
    -> F.safety.retained_record_create_count=0
creates_durable_audit_records=false
    -> F.safety.durable_audit_record_create_count=0
creates_rollback_plans=false
    -> F.safety.rollback_plan_create_count=0
allocates_service_slot=false
    -> F.safety.service_slot_allocation_count=0
accepts_* candidate input=false
    -> F.safety.external_artifact_intake_count=0 plus F.intake.accepts_*=false
loads_artifact=false -> F.safety.artifact_load_count=0
load_attempted=false -> F.safety.load_attempt_count=0
can_load/authorizes_load=false -> F.safety.load_authorization_count=0
service_inventory_change:"none" -> F.safety.service_inventory_change="none"
```

Retention is provenance, not an effect. A retained record's `event_id` maps to
`E[id].source_event_id`; the response denial event maps to top-level
`event_id`. Neither appears in `D.effects`. Denied decisions remain absolutely
`grants:[]` and `effects:[]`.

The canonical hash input grammars in `module_evidence.rs` and the evaluator
hash helpers are outside response vocabulary. Embedded strings such as
`authorizes_guest_load=false`, `load_attempted=false`, and
`service_inventory_change=none` in authority-hash inputs are not retired or
regenerated.

### Selftest mapping

Each selftest family becomes `<load-gate family>.selftest` and maps:

```text
R.schema -> constant("raios.evidence_response.v1")
R.scope -> scope
R.classification -> classification
R.test_infrastructure -> F.test_infrastructure
R.case_count -> F.case_count
R.passed -> F.passed
R.required_bindings -> F.required_bindings
R.missing_runtime_facts -> F.missing_runtime_facts
R.source_fact_count -> F.source_fact_count
R.source_fact_map_complete -> F.source_fact_map_complete
R.source_fact_map[] descriptive fields -> F.source_fact_map[] same fields
R.cases[].case -> F.cases[].case
R.cases[].expected_status/reason -> F.cases[].expected.{status,reason}
R.cases[].actual_status/reason -> F.cases[].actual.{status,reason}
R.cases[].family-specific expected/actual fields
    -> F.cases[].expected/actual.<same field>
R.cases[].passed -> F.cases[].passed
R/case authority and effect false fields -> retired(F.safety counters)
evidence -> constant([])
D -> constant({outcome:"observed", reason:"selftest_completed"})
```

## 3. Predicate inventory

Only predicates that inspect the direct denied response or the nine direct
selftest responses are counted. `full-audit` assertions inspect the load-gate
binding through event output and belong to P4-4, just as recent-event
assertions were excluded from P4-1. Positive hello/echo/granted-candidate load
responses and `module.loader_runtime` responses belong to other families.

Runtime predicate counts (loops expanded):

| Profile | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| `full-module-load-gate` | 157 | 347 | 1 | 505 |
| `full-module-selftests` | 229 | 88 | 9 | 326 |
| `m6c-promotion` | 0 | 2 | 2 | 4 |
| `m6d-rollback` | 0 | 2 | 2 | 4 |
| `m12-distribution-provenance` | 0 | 2 | 2 | 4 |
| `quick` | 0 | 9 | 3 | 12 |
| **Total** | **386** | **450** | **19** | **855** |

Thus 855 predicates were reviewed and 450 are likely regenerated.

### (a) Leaf needles that survive unchanged

These are the scoped predicates not listed in (b) or (c), principally:

```text
"classification": "local_only"
request leaves: load_mode, requested_capability, risk, target, subject
hash leaves and ram_only_service_slot_id
reason/readiness_reason/missing_reason leaves
case names, expected/actual statuses and reasons, family actual-state leaves
case_count, passed, required binding/catalog string membership
source_method, source_fact_locator, fact counts and map-complete leaves
descriptive present/source-chain/match/trust/hash leaves
```

These pairs survive under `facts`, evidence `facts`, or selftest cases even
though their containing path changes.

### (b) Needles and path assertions that must regenerate

Every scoped predicate using any of these must be regenerated:

```text
legacy "schema": "raios.*.v0"
legacy "code": "capability_denied" (becomes D.outcome="denied")
legacy object-name needle "<retained/projection/boundary>": {
body.* or body.result.* PowerShell property paths
event_id or retained_*_event_id legacy property paths
gate_state pairs and blocked_by legacy shape
legacy "status" values moved to facts.status_detail
can_load, can_load_now, grants_*, authorizes_*, load_authorized
load_attempted, execution_attempted, durable_write_attempted, network_attempted
service_inventory_change
allocates_*, creates_*, loads_*, starts_*, marks_*, mutates_*, writes_*,
installs_*, records_*, commits_*, unloads_*, cleans_up_*
accepts_loader_descriptor/descriptor_bytes/artifact_bytes when used as an
authority assertion rather than a descriptive intake fact
produces_*, validates_*, parses_descriptor_bytes, maps_executable_pages,
jumps_to_entrypoint when asserted as boundary non-effects
```

Structured assertions requiring new v1 paths include:

```text
policy:module_retained_artifact_event_id
policy:module_retained_vm_report_event_id
policy:module_retained_approval_event_id
policy:module_loader_runtime_<seven invocation-boundary checks>
policy:module_loader_runtime_source_count
policy:module_loader_runtime_source_map_complete
policy:module_loader_runtime_<eleven source bindings>

protocol:module_load_gate_loader_runtime_selftest_source_count
protocol:module_load_gate_loader_runtime_selftest_source_map_complete
protocol:module_load_gate_loader_runtime_selftest_<ten source bindings>

m6c:ungranted_candidate_denied_no_instantiation
m6c:generic_durable_load_gate_stays_denied
m6d:ungranted_candidate_denied_no_instantiation
m6c:generic_durable_load_gate_stays_denied  [name typo in m6d profile]
m12-distribution:N3_provenance_does_not_enable_granted_candidate_load
m12-distribution:N4_generic_durable_load_gate_preserved
quick wrong-target and external-target denied shape checks
```

### Safety-assertion collapse hazards

The following predicate families would collapse if rewritten only as
`D.outcome:"denied"` or `D.effects:[]`. Each must use the proposed
distinguishing v1 needle (or be honestly renamed):

| Legacy predicate suffix | Distinguishing v1 needle |
|---|---|
| `*_can_load_false`, `*_no_authority`, `*_authorizes_load_false` | `D.outcome:"denied"` plus the gate-specific `D.reason` or `D.blocked_by[].evidence_id` |
| `*_load_attempted_false`, `*_no_attempt` | `F.safety.load_attempt_count:0` for selftests; live denial uses `E[loader_runtime].facts.load_attempt_boundary.reason` |
| `*_no_load`, `*_artifact_not_loaded` | selftest `F.safety.artifact_load_count:0`; live denial uses artifact-load boundary reason |
| `*_no_mutation`, `*_no_inventory`, `*_inventory_none` | selftest `F.safety.service_inventory_change:"none"`; live denial uses registry-mutation boundary reason |
| `*_no_records` | the specific safety count: retained, inventory, health, audit, result, or start-audit count; do not share one generic zero |
| `*_no_write`, `*_no_audit_records`, `*_no_rollback_plans` | `F.safety.durable_audit_record_create_count:0` or `rollback_plan_create_count:0`, or the corresponding gate reason |
| `*_no_allocation` | `F.safety.service_slot_allocation_count:0` or `E[service_slot].reason` |
| `*_no_descriptor`, `*_no_parse`, `*_no_schema_validation`, `*_no_load_plan` | boundary-specific `facts.status_detail` or boundary-specific reason |
| `*_no_exec_pages`, `*_no_entrypoint`, `*_no_execution` | executable-mapping, entrypoint-transfer, and execution-authorization reasons respectively |
| `*_no_start`, `*_no_running` | service-start and running-state reasons respectively |
| `*_no_unload`, `*_no_cleanup` | unload and cleanup boundary reasons respectively |
| `*_no_commit`, `*_no_record`, `*_no_install` | live-load-commit, audit, result, or rollback boundary reason respectively |

This applies to every predicate whose name contains those suffixes in both
`full-module-load-gate` and `full-module-selftests`; none may be bulk-replaced
with one identical decision needle.

### (c) Framing and command completion

All 19 `Send-AgentCommand` completion waits for
`RAIOS_AGENT_END module.load_ephemeral` or
`RAIOS_AGENT_END module.load_gate_*_selftest` survive unchanged. The renderer
literal count is lower because all nine selftests use the shared
`begin_response()` helper.

### Swept but excluded files

`rg -l "load_gate" vm-harness/` also found
`shadow-vm-smoke-profile-full-audit.ps1`. Its 313
`protocol:module_load_audit_*` needles inspect an event response containing the
compact binding. They are P4-4 event-family predicates, not direct P4-2
response predicates. `quick` also contains unrelated command-envelope audit
and positive hello/echo load assertions; those were reviewed and excluded.

## 4. Selftest strategy

| Selftest | Case source | Cases | Treatment |
|---|---|---:|---|
| manifest | `module_load_gate_manifest_selftest_cases()` | 7 | evaluator semantics survive verbatim |
| artifact | `module_load_gate_artifact_selftest_cases()` | 9 | evaluator semantics survive verbatim |
| VM report | `module_load_gate_vm_report_selftest_cases()` | 11 | evaluator semantics survive verbatim |
| attestation | `module_load_gate_attestation_selftest_cases()` | 11 | evaluator semantics survive verbatim |
| approval | `module_load_gate_approval_selftest_cases()` | 12 | evaluator semantics survive verbatim |
| retained grant | `MODULE_LOAD_GATE_RETAINED_CASES` adapters | 7 | evaluator semantics survive verbatim |
| audit/rollback | `MODULE_LOAD_GATE_AUDIT_ROLLBACK_CASES` adapters | 23 | evaluator semantics survive verbatim |
| service slot | `MODULE_LOAD_GATE_SERVICE_SLOT_CASES` adapters | 13 | evaluator semantics survive verbatim |
| loader runtime | local loader-runtime case table | 5 | expected gate states and order survive verbatim |

`agent_protocol_module_load_gate_selftest_reference_cases.rs` constructs real
candidates and calls the raios-core evaluators. Its candidate mutations, case
order, expected status/reason, accepted-hash checks, and fail-closed
`can_load/load_attempted` assertions are evaluator semantics and must not be
regenerated from output.

`agent_protocol_module_load_gate_selftest_eval.rs` is the type adapter into
`raios_core::module_load_gate`; its conversions and evaluator calls survive
verbatim unless P4-2b moves that already-real evaluation into a new typed
projection. It must not become an emitter-derived oracle.

`agent_protocol_module_load_gate_selftest_emit.rs`, the nine response-shape
goldens, all `body.result` harness paths, legacy schemas, scattered safety
booleans, and case-object nesting must be regenerated to the v1 envelope and
`SelftestFacts` shape. Case values and order are copied from evaluator output,
not recomputed by the renderer.

The loader-runtime `source_fact_map` order and the counts 11 in the live gate
and 10 in its selftest are distinct current semantics and must stay distinct.

## 5. Risks and P4-2b STOP-tripwires

1. The 12-item `blocked_by` array is renderer-owned today. P4-2b must first
   introduce one raios-core typed projection that owns evidence order,
   first-failure reason, and `DeniedBy`; deleting the old emitter before that
   projection exists is a STOP.
2. `gate_state` orders computed grant before local approval, while
   `blocked_by` orders local approval before computed grant. Evidence order
   follows `blocked_by`, not `gate_state`.
3. The legacy response places error fields directly in `body`, while v1 is a
   root evidence response. Any surviving `body.*` path means the switch is
   incomplete.
4. Retained objects conditionally omit `reason`. V1 must emit an explicit
   value; silently retaining omission is a STOP.
5. The render file combines P4-2 load-gate denial with loader-runtime facts
   that the P4 design table also names in P4-3. P4-2b may move their response
   vocabulary, but must not broaden into executable mapping/apply or change
   loader evaluator behavior.
6. Retained references, receiver preflight, runtime facts, and event IDs are
   acquired through separate accessors. P4-2b needs one coherent captured
   projection; mixed snapshots are a STOP.
7. Response migration must not alter module evidence, descriptor, artifact,
   audit, rollback, service-slot, or authority hashes.
8. A denied decision with any grant or effect is a STOP. Retention remains
   provenance only.
9. Boundary-specific safety predicates must retain distinguishing evidence;
   bulk replacement by `D.effects:[]` is a STOP.
10. The focused fragment has 505 runtime predicates after loop expansion;
    accepting static-line counts as the golden inventory is a STOP.
11. Event-binding harness churn belongs to P4-4. Changing full-audit event
    expectations in P4-2b would cross the packet boundary.
12. No `raw()`, `json_str()`, or `raw_bool()` JSON construction may remain in
    the converted response family; only framing may remain handwritten.

### OWNER/ORCHESTRATOR DECISION NEEDED

The P4 contract says `D.reason` is the evaluator's first failure and that the
evaluator owns `blocked_by`. Today there is no unified load-gate evaluator
producing that list: the renderer reconstructs all 12 blockers at
`agent_protocol_module_load_gate_render.rs:5428-5486`. The legacy response's
human-level denial is `missing_evidence`, while the current first blocker can
be a retained-but-non-authorizing manifest reason.

Decision required before P4-2b deletes the emitter: should v1 `D.reason` be
the exact first `blocked_by` reason (the P4 grammar and `DenialDecision` model),
or preserve legacy `missing_evidence` as the decision reason and expose first
failure only in `blocked_by[0]`? This manifest recommends the former because it
matches the adopted v1 contract, but does not authorize that semantic change.

### P4-2b1 note
The orchestrator ratified exact-first-blocker `D.reason`; legacy `missing_evidence` is retired vocabulary.

## 6. Static-check evidence

Required BEGIN-count command and verbatim output:

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/agent_protocol_module_load_gate_render.rs seed-kernel/src/agent_protocol_module_load_gate_selftest_emit.rs
seed-kernel/src/agent_protocol_module_load_gate_render.rs:1
```

The selftest file has zero literal matches because it calls
`begin_response()` nine times.

Required harness sweep and verbatim output:

```text
> rg -l "load_gate" vm-harness/
vm-harness/shadow-vm-smoke-profile-full-audit.ps1
vm-harness/shadow-vm-smoke-profile-full-module-load-gate.ps1
vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1
vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1
vm-harness/shadow-vm-smoke-profile-m12-distribution-provenance.ps1
vm-harness/shadow-vm-smoke-profile-quick.ps1
```

Needle-count commands and verbatim output:

```text
> rg -c "policy:" vm-harness/shadow-vm-smoke-profile-full-module-load-gate.ps1
488
> rg -c "protocol:module_load_gate_" vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
308
> rg -c "m6c:(ungranted_candidate_denied_no_instantiation|generic_durable_load_gate_stays_denied)" vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1
2
> rg -c "m6d:(ungranted_candidate_denied_no_instantiation|generic_durable_load_gate_stays_denied)" vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1
1
> rg -c "m12-distribution:N[34]_" vm-harness/shadow-vm-smoke-profile-m12-distribution-provenance.ps1
4
> rg -c "quick:module_load_" vm-harness/shadow-vm-smoke-profile-quick.ps1
7
```

The M6d count is one because the generic predicate is accidentally named
`m6c:generic_durable_load_gate_stays_denied`; it is nevertheless included in
the inventory. Prefix counts are static occurrence evidence, not the runtime
totals in section 3: the runtime totals expand 7 invocation-boundary checks,
11 live source bindings, and 10 selftest source bindings, and count command
completion separately.

No Cargo build or test was attempted, as required by this docs-only packet.

### P4-2b2a notes
The retired hand-written denied-response and compact direct-binding renderers were replaced by the typed load-gate projection and evidence-v1 envelope. `service.load_ephemeral` is covered by the shared denied action; framing and `source_method` preserve the invoked alias.
The 230 legacy predicates that collapsed onto an identical v1 evidence/decision carrier were honestly merged into the first equivalent predicate; the focused load-gate profile now has 250 unique literal needles and no duplicate needle groups.

+P4-2b2a STOP-10 repair completed: all 230 legacy predicate names removed by the initial rewrite are now individually accounted for. Fourteen distinct v1 carriers were regenerated in the focused harness, 210 legacy names are honest aliases of an already-surviving gate/evidence carrier, and six legacy effect/authority fields are retired exactly as section 2 specifies.

Static Assert-LogContains count: legacy 480; initial v1 rewrite 250; repaired v1 harness 264. With the unchanged parsed-path checks, expanded loops, and command completion, the repaired harness has 289 physical runtime predicates. The section-3 inventory reconciles exactly: 289 physical + 210 merged aliases + 6 retirements = 505 reviewed runtime semantics.

#### Regenerated legacy predicates (14)

| Legacy predicate | Distinguishing v1 carrier |
|---|---|
| policy:module_loader_runtime_reason | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_loader_runtime_state | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_loader_runtime_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_approval_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_artifact_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_attestation_event_id | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_attestation_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_audit_rollback_event_id | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_audit_rollback_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_grant_event_id | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_grant_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_service_slot_event_id | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_retained_vm_report_status | restored predicate of the same name; byte-exact evidence/facts needle |
| policy:module_service_slot_allocator_ready_false | restored predicate of the same name; byte-exact evidence/facts needle |

#### Honest merges (210)

Each carrier below is a surviving predicate with a unique needle. Repeated legacy booleans under one boundary reduce to that boundary-specific v1 reason; they do not reduce to the generic denied decision.

| Legacy predicate | Surviving v1 carrier |
|---|---|
| policy:candidate_artifact_retained_reason | policy:candidate_artifact_retained |
| policy:module_audit_reference_reason | policy:module_audit_reference_retained |
| policy:module_audit_rollback_hash_retained | policy:module_retained_rollback_hash |
| policy:module_computed_grant_retained_not_authorizing | policy:module_computed_grant_retained |
| policy:module_loader_runtime_artifact_byte_intake_boundary_no_artifact_bytes | policy:module_loader_runtime_artifact_byte_intake_boundary_reason |
| policy:module_loader_runtime_artifact_byte_intake_boundary_no_artifact_intake | policy:module_loader_runtime_artifact_byte_intake_boundary_reason |
| policy:module_loader_runtime_artifact_load_boundary_no_load | policy:module_loader_runtime_artifact_load_boundary_reason |
| policy:module_loader_runtime_commit_audit_boundary_no_record | policy:module_loader_runtime_commit_audit_boundary_reason |
| policy:module_loader_runtime_commit_result_boundary_no_result | policy:module_loader_runtime_commit_result_boundary_reason |
| policy:module_loader_runtime_commit_rollback_boundary_no_install | policy:module_loader_runtime_commit_rollback_boundary_reason |
| policy:module_loader_runtime_descriptor_acceptance_authority_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_acceptance_authority_boundary_reason |
| policy:module_loader_runtime_descriptor_acceptance_authority_boundary_no_descriptor | policy:module_loader_runtime_descriptor_acceptance_authority_boundary_reason |
| policy:module_loader_runtime_descriptor_acceptance_authority_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_acceptance_authority_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_capability_validation | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_descriptor | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_parse | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_result | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_schema_validation | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_capability_validation_boundary_no_validated_descriptor | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_authority | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_binding | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_capability_validation | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_descriptor | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_image_layout | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_load_plan | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_maps | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_page_mapping_plan | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_parse | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_result | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_schema_validation | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_executable_page_binding_boundary_no_validated_descriptor | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_descriptor_intake_boundary_no_descriptor | policy:module_loader_runtime_descriptor_intake_boundary_reason |
| policy:module_loader_runtime_descriptor_intake_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_intake_boundary_reason |
| policy:module_loader_runtime_descriptor_intake_boundary_no_intake | policy:module_loader_runtime_descriptor_intake_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_capability_validation | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_descriptor | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_executable_binding | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_load_plan | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_parse | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_result | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_schema_validation | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_load_plan_boundary_no_validated_descriptor | policy:module_loader_runtime_descriptor_load_plan_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_contract_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_parser_contract_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_contract_boundary_no_descriptor | policy:module_loader_runtime_descriptor_parser_contract_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_contract_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_parser_contract_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_contract_boundary_no_parse | policy:module_loader_runtime_descriptor_parser_contract_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_result_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_result_boundary_no_descriptor | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_result_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_result_boundary_no_parse | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_descriptor_parser_result_boundary_no_result | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_artifact_bytes | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_descriptor | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_descriptor_bytes | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_parse | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_result | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_schema_validation | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_descriptor_schema_validation_boundary_no_validated_descriptor | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_entrypoint_transfer_boundary_no_entrypoint | policy:module_loader_runtime_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_binding | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_entrypoint | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_image_layout | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_load_plan | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_maps | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_binding_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_binding | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_entrypoint | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_image_layout | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_load_plan | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_maps | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_handoff_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_binding | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_entrypoint | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_image_layout | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_load_plan | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_maps | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_binding | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_entrypoint | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_image_layout | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_load_plan | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_maps | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_entrypoint_transfer_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_authority | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_capability_validation | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_descriptor | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_descriptor_bytes | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_executable_binding | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_image_layout | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_load_plan | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_parse | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_result | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_schema_validation | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_image_layout_boundary_no_validated_descriptor | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_authority | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_capability_validation | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_descriptor | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_descriptor_bytes | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_executable_binding | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_load_plan | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_parse | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_result | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_schema_validation | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_authority_boundary_no_validated_descriptor | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_authority | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_capability_validation | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_descriptor | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_descriptor_bytes | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_executable_binding | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_load_plan | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_parse | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_result | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_schema_validation | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_load_plan_result_boundary_no_validated_descriptor | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_executable_mapping_boundary_no_exec_pages | policy:module_loader_runtime_executable_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_authority | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_capability_validation | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_descriptor | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_descriptor_bytes | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_executable_binding | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_image_layout | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_load_plan | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_maps | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_parse | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_result | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_schema_validation | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_boundary_no_validated_descriptor | policy:module_loader_runtime_executable_page_mapping_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_artifact_bytes | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_authority | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_capability_validated_descriptor | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_capability_validation | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_descriptor | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_descriptor_bytes | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_executable_binding | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_image_layout | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_load_plan | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_page_mapping_plan | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_parse | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_result | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_schema_validation | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_executable_page_mapping_plan_boundary_no_validated_descriptor | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_execution_authorization_boundary_no_entrypoint | policy:module_loader_runtime_execution_authorization_boundary_reason |
| policy:module_loader_runtime_execution_authorization_boundary_no_exec_pages | policy:module_loader_runtime_execution_authorization_boundary_reason |
| policy:module_loader_runtime_execution_authorization_boundary_no_execution | policy:module_loader_runtime_execution_authorization_boundary_reason |
| policy:module_loader_runtime_execution_commit_gate_no_artifact_bytes | policy:module_loader_runtime_execution_commit_gate_reason |
| policy:module_loader_runtime_execution_commit_gate_no_descriptor | policy:module_loader_runtime_execution_commit_gate_reason |
| policy:module_loader_runtime_execution_commit_gate_no_execution | policy:module_loader_runtime_execution_commit_gate_reason |
| policy:module_loader_runtime_live_load_commit_boundary_no_commit | policy:module_loader_runtime_live_load_commit_boundary_reason |
| policy:module_loader_runtime_load_attempt_boundary_no_attempt | policy:module_loader_runtime_load_attempt_boundary_reason |
| policy:module_loader_runtime_no_artifact_bytes | policy:module_loader_runtime_artifact_byte_intake_boundary_reason |
| policy:module_loader_runtime_no_capability_descriptor_executable_binding | policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason |
| policy:module_loader_runtime_no_capability_validated_descriptor | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_no_capability_validation | policy:module_loader_runtime_descriptor_capability_validation_boundary_reason |
| policy:module_loader_runtime_no_descriptor | policy:module_loader_runtime_descriptor_intake_boundary_reason |
| policy:module_loader_runtime_no_descriptor_parse | policy:module_loader_runtime_descriptor_parser_contract_boundary_reason |
| policy:module_loader_runtime_no_executable_load_plan_authority | policy:module_loader_runtime_executable_load_plan_authority_boundary_reason |
| policy:module_loader_runtime_no_image_layout | policy:module_loader_runtime_executable_image_layout_boundary_reason |
| policy:module_loader_runtime_no_load | policy:module_loader_runtime_artifact_load_boundary_reason |
| policy:module_loader_runtime_no_load_plan | policy:module_loader_runtime_executable_load_plan_result_boundary_reason |
| policy:module_loader_runtime_no_page_mapping_plan | policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason |
| policy:module_loader_runtime_no_parsed_descriptor | policy:module_loader_runtime_descriptor_parser_result_boundary_reason |
| policy:module_loader_runtime_no_schema_validation | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_no_validated_descriptor | policy:module_loader_runtime_descriptor_schema_validation_boundary_reason |
| policy:module_loader_runtime_service_health_binding_boundary_no_records | policy:module_loader_runtime_service_health_binding_boundary_reason |
| policy:module_loader_runtime_service_registry_mutation_boundary_no_mutation | policy:module_loader_runtime_service_registry_mutation_boundary_reason |
| policy:module_loader_runtime_service_registry_mutation_boundary_no_records | policy:module_loader_runtime_service_registry_mutation_boundary_reason |
| policy:module_loader_runtime_service_running_state_boundary_no_running | policy:module_loader_runtime_service_running_state_boundary_reason |
| policy:module_loader_runtime_service_start_audit_boundary_no_record | policy:module_loader_runtime_service_start_audit_boundary_reason |
| policy:module_loader_runtime_service_start_boundary_no_running | policy:module_loader_runtime_service_start_boundary_reason |
| policy:module_loader_runtime_service_start_boundary_no_start | policy:module_loader_runtime_service_start_boundary_reason |
| policy:module_loader_runtime_service_unload_cleanup_boundary_no_cleanup | policy:module_loader_runtime_service_unload_cleanup_boundary_reason |
| policy:module_loader_runtime_service_unload_cleanup_boundary_no_unload | policy:module_loader_runtime_service_unload_cleanup_boundary_reason |
| policy:module_loader_unimplemented_reason | policy:module_loader_unavailable |
| policy:module_local_approval_hash_retained | policy:module_retained_approval_hash |
| policy:module_local_approval_reference_hash_retained | policy:module_retained_approval_ref_hash |
| policy:module_local_approval_retained_reason | policy:module_local_approval_retained |
| policy:module_local_attestation_retained_reason | policy:module_local_attestation_retained |
| policy:module_manifest_retained_reason | policy:module_manifest_retained |
| policy:module_retained_approval_reason | policy:module_local_approval_retained |
| policy:module_retained_service_slot_no_allocation | policy:module_service_slot_retained |
| policy:module_retained_service_slot_no_inventory | policy:module_service_slot_retained |
| policy:module_retained_service_slot_reason | policy:module_service_slot_retained |
| policy:module_rollback_reference_reason | policy:module_rollback_reference_retained |
| policy:module_service_slot_authority_decision_no_authority | policy:module_service_slot_authority_decision_reason |
| policy:module_service_slot_registry_commit_gate_no_mutation | policy:module_service_slot_registry_commit_gate_reason |
| policy:module_service_slot_registry_commit_gate_no_write | policy:module_service_slot_registry_commit_gate_reason |
| policy:module_vm_report_hash_retained | policy:module_retained_vm_report_hash |
| policy:module_vm_report_retained_reason | policy:module_vm_report_retained |

#### Retired legacy predicates (6)

| Legacy predicate | Retirement |
|---|---|
| policy:module_audit_rollback_requirements_no_audit_records | retired into decision.effects: [] |
| policy:module_audit_rollback_requirements_no_rollback_plans | retired into decision.effects: [] |
| policy:module_audit_rollback_requirements_no_writes | retired into decision.effects: [] |
| policy:module_can_load_false | retired into denied outcome plus decision.grants: [] |
| policy:module_load_attempted_false | retired into decision.effects: [] |
| policy:module_service_not_started | retired into decision.effects: [] |

UNCERTAIN: none. All restored needles come from the current typed renderer/evaluator values; evidence items and nested facts are single-line InlineObjects, and no needle spans the response id.
The accounting above completes the focused-harness STOP-10 repair for orchestration.

Correction (P4-2b2a-fix2): the `memory.recent_events` load-gate event binding was not converted to evidence-v1; it remains on the pre-v1 compact vocabulary because event-record output belongs to the EVENT family and converts in P4-4, not P4-2. All P4-2b2a v1 response paths remain converted.
