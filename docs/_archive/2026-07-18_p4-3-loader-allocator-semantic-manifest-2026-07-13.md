# P4-3a — loader/allocator semantic manifest

Static manifest for P4-3b. This packet changes no emitter, evaluator, harness,
or existing document.

Notation:

- `R` = legacy `body.result`.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy response vocabulary intentionally removed.

The family boundary is strict. Direct responses from the allocator, loader
identity/hash/fact methods, and loader runtime belong here. Loader/allocator
projections nested in `module.load_ephemeral` were inventoried by P4-2a.
Event-wrapped copies in `memory.recent_events` belong to P4-4. The positive
Wasm granted-candidate implementation remains owned by
`granted_candidate_service`; only its observational projection currently
embedded in `module.loader_runtime` is inventoried here.

## 1. Response-path inventory

All methods use `begin_response(method)` / `end_response(method)`, hence the
legacy transport is:

```text
RAIOS_AGENT_BEGIN <method>
{v, t, id, body:{method, result:{...}}}
RAIOS_AGENT_END <method>
```

The source contains no literal `RAIOS_AGENT_BEGIN`; framing is supplied by the
shared helper. P4-3b changes the result tree, not these framing markers.

### `module.service_slot_allocator`

Emitter: `emit_module_service_slot_allocator()` at
`seed-kernel/src/agent_protocol_module_service_slot_allocator.rs:13`. It
acquires event-log snapshots, records the source-evidence chain, evaluates it,
then emits at lines 367-593.

Current top-level field order:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, global_event_log_mutation,
creates_service_slot_reservation_records, allocates_service_slot,
creates_service_inventory_records, loads_artifact,
service_inventory_change, can_allocate, can_load_now, load_attempted,
source_evidence, retained_service_slot_reservation,
allocator_readiness_facts, allocator_prerequisite_gates,
allocator_authority_boundary, allocation_intent_boundary,
authority_input_boundaries, authority_decision,
registry_write_commit_gate, policy_result, blocked_by
```

Nested order is stable and table-shaped:

```text
source_evidence:
 service_slot_allocator_runtime, service_slot_registry_binding,
 service_health_state_model, service_unload_cleanup_plan,
 durable_audit_write, rollback_plan_install, module_loader,
 allocator_authority, allocation_intent,
 authority_inputs {policy_decision, registry_write_authority,
   loader_runtime_contract, health_monitor_binding,
   unload_cleanup_authority},
 authority_decision, registry_write_commit_gate

each source-evidence item:
 schema, state, status, reason, scope, classification, retention, event_id,
 fact_schema, fact_id, source_method, source_fact_locator,
 fact/prerequisite status and reason, present/available and binding booleans,
 dependency event IDs where applicable, then repeated non-effect booleans

retained_service_slot_reservation (present):
 state, schema, event_id, status, reason, classification,
 allocates_service_slot, creates_service_inventory_records,
 service_inventory_change, can_allocate, can_load_now, load_attempted,
 retained_computed_grant_reference_event_id,
 retained_audit_rollback_reference_event_id, ram_only_service_slot_id,
 hashes {reservation_hash, computed_capability_grant_hash,
   audit_record_hash, rollback_plan_hash, pre_load_service_inventory_hash}

retained_service_slot_reservation (missing):
 state, schema, event_id, status, reason, classification,
 allocates_service_slot, creates_service_inventory_records,
 can_allocate, can_load_now, load_attempted

allocator_readiness_facts (four, in order):
 service_slot_allocator_runtime, service_slot_registry_binding,
 service_health_state_model, service_unload_cleanup_plan

each readiness fact:
 schema, id, source_method, source_fact_locator,
 source_evidence_event_id, source_evidence_schema, source_evidence_state,
 source_evidence_status, source_evidence_reason, source_evidence_method,
 source_evidence_fact_locator, scope, classification, status, reason,
 present, schema_valid, provenance_valid,
 binds_retained_service_slot_reservation, binds_allocator_runtime,
 authority, persistence, durable, repeated non-effects,
 required_bindings, provenance

allocator_prerequisite_gates (three, in order):
 durable_audit_write, rollback_plan_install, module_loader

each prerequisite:
 schema, id, source_method, source_fact_locator,
 source_evidence_event_id/schema/state/status/reason/method/fact_locator,
 status, reason, available, scope, classification, authority, persistence,
 durable, repeated non-effects, provenance

allocator_authority_boundary:
 schema, id, scope, classification, source_method, source_fact_locator,
 source_evidence_event_id/schema/state/status/reason,
 status, reason, present, source_chain_complete, future_authority_inputs,
 repeated intake/authority/non-effect booleans

allocation_intent_boundary:
 same source prefix, status, reason, present, source_chain_complete,
 requested_capability, load_mode, target, repeated non-effects

authority_input_boundaries (five, in order):
 policy_decision, registry_write_authority, loader_runtime_contract,
 health_monitor_binding, unload_cleanup_authority

each authority input:
 schema, id, scope, classification, source_method, source_fact_locator,
 source_evidence_event_id/schema/state/status/reason,
 dependency_source_evidence_event_id, status, reason, present,
 source_chain_complete, requested_capability, load_mode, target,
 repeated non-effects

authority_decision:
 common source prefix, status, reason, present, input_chain_complete,
 source_chain_complete, requested_capability, load_mode, target,
 authorizes_allocation, authorizes_load, repeated non-effects

registry_write_commit_gate:
 common source prefix, status, reason, present, source_chain_complete,
 authority_decision_present, registry_write_authority_present,
 registry_binding_available, durable_audit_write_available,
 rollback_plan_install_available,
 retained_service_slot_reservation_present,
 requested_capability, load_mode, target,
 authorizes_registry_write, authorizes_allocation, authorizes_load,
 mutates_service_registry, writes_durable_audit_state,
 installs_rollback_state, repeated non-effects

policy_result:
 readiness_status, readiness_reason,
 retained_service_slot_reservation_present,
 retained_hash_reference_allocates_slot,
 allocator_runtime_available, registry_binding_available,
 health_state_available, unload_cleanup_available,
 durable_audit_written, rollback_plan_installed, module_loader_available,
 allocator_authority_status/reason, allocation_intent_status/reason,
 authority_input_statuses, authority_decision_status/reason,
 registry_write_commit_gate_status/reason,
 service_slot_reserved, registry_write_committed,
 mutates_service_registry, writes_durable_audit_state,
 installs_rollback_state, allocates_service_slot,
 creates_service_inventory_records, can_allocate, can_load_now,
 load_attempted

blocked_by (evaluator order):
 retained_service_slot_reservation, service_slot_allocator_runtime,
 service_slot_registry_binding, service_health_state_model,
 service_unload_cleanup_plan, durable_audit_write, rollback_plan_install,
 module_loader, service_slot_allocator_authority,
 service_slot_allocation_intent, the five authority inputs,
 service_slot_allocator_authority_decision,
 service_slot_registry_write_commit_gate
```

The readiness and prerequisite item orders are declared at
`agent_protocol_module_types.rs:58-96`, `:149-187`, and `:253-338`.
First failure is selected at
`agent_protocol_module_service_slot_allocator.rs:3586-3643`.

### Loader identity and artifact-hash binding

Emitters:

```text
module.loader_identity
  emit_module_loader_identity()
  agent_protocol_module_loader_identity.rs:16-139

module.loader_artifact_hash_binding
  emit_module_loader_artifact_hash_binding()
  agent_protocol_module_loader_artifact_hash_binding.rs:16-147
```

Both have the same outer order:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, global_event_log_mutation,
accepts_loader_descriptor, accepts_descriptor_bytes,
produces_parsed_descriptor, validates_descriptor_schema,
produces_validated_descriptor, validates_descriptor_capabilities,
produces_capability_validated_descriptor,
authorizes_executable_load_plan, produces_executable_load_plan,
produces_executable_image_layout, produces_executable_page_mapping_plan,
maps_executable_pages,
binds_capability_validated_descriptor_to_executable_pages,
parses_descriptor_bytes, accepts_artifact_bytes, loads_artifact,
allocates_service_slot, creates_service_inventory_records,
service_inventory_change, starts_service, marks_service_running,
creates_service_health_records, writes_service_start_audit_record,
unloads_service, cleans_up_service_slot, commits_live_load,
writes_load_commit_audit_record, installs_commit_rollback_record,
records_load_result, can_load_now, load_attempted, authorizes_guest_load,
[retained_module_evidence for identity only], source_evidence,
required_bindings, <owned fact>, policy_result, blocked_by
```

Identity nested order:

```text
retained_module_evidence:
 state, status, reason, classification, authority,
 manifest_reference_event_id, candidate_artifact_reference_event_id,
 vm_test_report_reference_event_id, local_attestation_reference_event_id,
 local_approval_reference_event_id, computed_grant_reference_event_id,
 audit_rollback_reference_event_id, service_slot_reservation_event_id

source_evidence:
 schema, state, status, reason, scope, classification, retention, event_id,
 fact_schema, fact_id, source_method, source_fact_locator,
 readiness_status, readiness_reason, identity_status, identity_reason,
 identity_present, retained_module_evidence_present,
 service_slot_allocator_readiness_present,
 service_slot_allocator_ready,
 audit_rollback_write_boundary_present, repeated non-effects

required_bindings:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 loader_identity_fact_present

loader_identity:
 schema, state, status, reason, scope, fact_scope, schema_valid,
 classification, provenance_valid, binds_retained_module_evidence,
 binds_service_slot_allocator, binds_audit_rollback_write_boundary,
 fact_id, source_method, source_fact_locator, source_evidence_event_id,
 source_evidence_schema, source_evidence_state, persistence, durable,
 repeated non-effects

policy_result:
 readiness_status, readiness_reason, retained_module_evidence_present,
 service_slot_allocator_readiness_present, service_slot_allocator_ready,
 audit_rollback_write_boundary_present, identity_available,
 repeated non-effects

blocked_by:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 loader_identity
```

Artifact-hash binding replaces the identity-owned leaf with:

```text
source_evidence:
 common source prefix, readiness_status/reason,
 artifact_hash_binding_status/reason/present,
 retained_module_evidence_present,
 service_slot_allocator_readiness_present/ready,
 audit_rollback_write_boundary_present,
 loader_identity_source_evidence_present,
 loader_identity_source_evidence_event_id,
 binds_loader_identity, repeated non-effects

required_bindings:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 loader_identity, artifact_hash_binding_present

artifact_hash_binding:
 common fact prefix, binds_retained_module_evidence,
 binds_service_slot_allocator, binds_audit_rollback_write_boundary,
 binds_loader_identity, fact_id, source_method, source_fact_locator,
 source_evidence_event_id/schema/state, persistence, durable,
 repeated non-effects

policy_result:
 readiness_status/reason, retained evidence and allocator presence/readiness,
 audit boundary presence, loader identity presence,
 artifact_hash_binding_available, repeated non-effects

blocked_by:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 loader_identity, artifact_hash_binding
```

These orders are emitted at identity lines 61-137 and 182-432, and artifact
binding lines 65-145 and 194-416.

### Eight loader-fact methods

`emit_module_loader_fact(method)` at
`agent_protocol_module_loader_fact.rs:329-413` serves, in declared order:

```text
module.loader_entrypoint_abi
module.loader_address_space_boundary
module.loader_memory_map_constraints
module.loader_capability_import_table
module.loader_service_slot_binding
module.loader_health_state_hooks
module.loader_rollback_hooks
module.loader_audit_rollback_write_boundary_binding
```

The table, schemas, IDs, dependencies, and reasons are at lines 90-327. Each
response order is:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, global_event_log_mutation,
accepts_loader_descriptor, accepts_artifact_bytes, loads_artifact,
allocates_service_slot, creates_service_inventory_records,
service_inventory_change, can_load_now, load_attempted,
[source_evidence], required_bindings, <owned fact>, policy_result, blocked_by
```

`source_evidence` is emitted for all eight current specs. Its order is:

```text
schema, state, status, reason, scope, classification, retention, event_id,
fact_schema, fact_id, source_method, source_fact_locator,
readiness_status, readiness_reason, fact_status, fact_reason, fact_present,
dependency_gate, dependency_schema, dependency_method, dependency_present,
dependency_source_evidence_event_id, repeated non-effects
```

Other nested order:

```text
required_bindings:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 <dependency_gate>, fact_present, dependency_schema, dependency_method

owned fact:
 schema, state, status, reason, scope, fact_scope, schema_valid,
 classification, provenance_valid, binds_retained_module_evidence,
 binds_service_slot_allocator, binds_audit_rollback_write_boundary,
 <binds_dependency>, fact_id, source_method, source_fact_locator,
 [source_evidence_event_id/schema/state], persistence, durable,
 repeated non-effects

policy_result:
 readiness_status, readiness_reason, retained_module_evidence_present,
 service_slot_allocator_readiness_present, service_slot_allocator_ready,
 audit_rollback_write_boundary_present, dependency_present,
 fact_available, repeated non-effects

blocked_by:
 retained_module_evidence, service_slot_allocator_readiness,
 service_slot_allocator_runtime, audit_rollback_write_boundary,
 <dependency_gate>, <owned fact>
```

The dependency chain is artifact hash -> entrypoint ABI -> address space ->
memory map -> capability imports -> service-slot binding -> health hooks ->
rollback hooks -> audit/rollback write-boundary binding. The evaluator and
blocked-list order are at lines 819-989 and 731-772.

### `module.loader_runtime`

Emitter: `emit_module_loader_runtime()` at
`agent_protocol_module_loader_runtime.rs:125-1167`. Acquisition and event
retention occur at lines 125-856; result emission begins at line 857.

Current top-level order:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, global_event_log_mutation,
accepts_loader_descriptor, accepts_artifact_bytes, loads_artifact,
allocates_service_slot, creates_service_inventory_records,
service_inventory_change, can_load_now, load_attempted,
retained_module_evidence, service_slot_allocator_readiness,
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
executable_entrypoint_invocation_boundary,
loader_runtime_facts, live_granted_load_projection,
policy_result, blocked_by
```

The field constructors are in `render.rs`. Their current templates are:

```text
retained_module_evidence item:
 schema, scope, classification, status, event_id, reason, authority,
 repeated non-effects

service_slot_allocator_readiness:
 schema, scope, classification, source_method, state, readiness_status,
 readiness_reason, allocator_authority_boundary,
 allocation_intent_boundary, authority_input_boundaries,
 authority_decision, registry_write_commit_gate,
 retained_service_slot_reservation_present, service_slot_allocator_ready,
 repeated non-effects

execution/intake/authorization/registry special boundaries:
 schema, id, source_evidence_event_id/schema/state/status/reason,
 source_method, source_fact_locator, status, reason, present,
 source_chain_complete, boundary-specific prerequisite facts,
 request {requested_capability, load_mode, target},
 boundary-specific action/authority booleans, repeated non-effects

the remaining 30 live-load boundaries:
 schema, id, source_evidence_event_id/schema/state/status/reason,
 source_method, source_fact_locator, status, reason, present,
 source_chain_complete, boundary-specific prerequisite observations,
 request, boundary-specific action/authority booleans,
 repeated non-effects

loader_runtime_facts (ten, declared order):
 loader_identity, artifact_hash_binding, entrypoint_abi,
 address_space_boundary, memory_map_constraints,
 capability_import_table, service_slot_binding,
 health_state_hooks, rollback_hooks,
 audit_rollback_write_boundary_binding

each runtime fact:
 schema, id, source_method, source_fact_locator,
 [source_evidence_event_id/schema/state/status/reason/method/fact_locator],
 scope, classification, status, reason, present, schema_valid,
 provenance_valid, binding booleans, authority, persistence, durable,
 repeated non-effects, required_bindings, provenance

policy_result:
 readiness_status, readiness_reason, retained_module_evidence_complete,
 service_slot_allocator_readiness_present, service_slot_allocator_ready,
 loader_runtime_facts_complete,
 every execution-boundary status/reason pair in emission order,
 repeated non-effects

blocked_by:
 every failed retained prerequisite, allocator gate, loader fact, and
 execution boundary in the exact evaluator order stated in section 3
```

Special boundary renderers are at `render.rs:1221-1550`, the shared live-load
boundary renderer at `:1553-1634`, and runtime facts at `:1637-1794`.

### Selftest responses

Methods:

```text
module.service_slot_allocator_selftest
module.loader_identity_selftest
module.loader_artifact_hash_binding_selftest
the eight <loader-fact>_selftest methods
module.loader_runtime_selftest
```

All use this outer order:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, family-specific repeated non-effects,
case_count, passed,
[source_fact_count, source_fact_map_complete, source_fact_map for runtime],
cases, can_load
```

Allocator cases currently emit a large family-specific expected/actual record;
identity, artifact binding, and loader-fact cases emit:

```text
case, expected_status, expected_reason, actual_status, actual_reason,
actual_<owned-fact>_status, actual_<owned-fact>_reason, passed,
loads_artifact, allocates_service_slot,
creates_service_inventory_records, can_load, load_attempted
```

Runtime case values are built at `evidence_core.rs:112-170`; the table is
assembled by `selftest.rs:110-1085`. These are reference diagnostics only and
must remain effect-free.

## 2. Semantic manifest

### Envelope and common fields

```text
v -> retired(redundant with v1 schema)
t -> retired(redundant with D.outcome)
id:"serial" -> v1.id (response.current_boot.NNNNNNNN)
body.method -> source_method
body.result -> flattened v1 response root
R.schema -> constant("raios.evidence_response.v1")
R.scope -> scope
R.classification -> classification
R.test_infrastructure -> F.test_infrastructure
v1.family -> constant(method-table family)
```

Family names are the direct method name with `.selftest` retained for
selftests. `module.loader_*` fact methods may share an implementation table,
but must retain distinct family strings and source methods.

### Mechanical fact/evidence mapping

Every descriptive leaf not listed under decision/effects maps to the evidence
item that owns its execution or evaluation boundary:

```text
R.<object>.schema -> E[object].facts.record_schema
R.<object>.id -> E[object].facts.record_id
R.<object>.scope -> scope
R.<object>.classification -> E[object].classification
R.<object>.source_method -> E[object].facts.source_method
R.<object>.source_fact_locator -> E[object].facts.source_fact_locator
R.<object>.source_evidence_event_id -> E[object].source_event_id
R.<object>.event_id -> E[object].source_event_id
R.<object>.source_evidence_schema -> E[object].facts.source_record_schema
R.<object>.source_evidence_state -> E[object].facts.source_state
R.<object>.source_evidence_status -> E[object].facts.source_status_detail
R.<object>.source_evidence_reason -> E[object].facts.source_reason
R.<object>.dependency_source_evidence_event_id
  -> E[object].facts.dependency_source_event_id
R.<object>.state/status/readiness_status
  -> E[object].facts.status_detail
R.<object>.reason/readiness_reason -> E[object].reason
R.<object>.present/available/source_chain_complete
  -> E[object].facts.<same leaf>
R.<object>.schema_valid/provenance_valid/binds_*
  -> E[object].facts.<same leaf>
R.<object>.required_bindings -> E[object].facts.required_bindings
R.<object>.provenance -> E[object].facts.provenance
R.<object>.hashes.* -> E[object].facts.*
```

Common `E.status` is `present | missing | rejected | verified | unavailable |
not_applicable`; every family-specific status survives as `facts.status_detail`.
Null event IDs remain explicit nulls.

Retained records map as provenance, never effects:

```text
R.retained_module_evidence.*_event_id
  -> E[retained_module_evidence].facts.*_event_id
R.retained_service_slot_reservation.event_id
  -> E[service_slot_reservation].source_event_id
R.source_evidence.event_id -> E[owned boundary].source_event_id
R.mutates_global_event_log / R.global_event_log_mutation
  -> retired(event_id and E.source_event_id carry the recording result)
```

The allocator source-evidence wrapper is not a second parallel evidence tree.
Its items merge into their owning `E[id]`: source provenance becomes
`source_event_id` plus `facts.source_*`, and evaluator status/reason remains the
authoritative item status/reason.

### Family-specific mappings

```text
R.retained_module_evidence -> E[retained_module_evidence]

R.allocator_readiness_facts.<name>
  -> E[service_slot_allocator_runtime | service_slot_registry_binding |
       service_health_state_model | service_unload_cleanup_plan]
R.allocator_prerequisite_gates.<name>
  -> E[durable_audit_write | rollback_plan_install | module_loader]
R.allocator_authority_boundary -> E[service_slot_allocator_authority]
R.allocation_intent_boundary -> E[service_slot_allocation_intent]
R.authority_input_boundaries.<name> -> E[name]
R.authority_decision -> E[service_slot_allocator_authority_decision]
R.registry_write_commit_gate -> E[service_slot_registry_write_commit_gate]

R.loader_identity -> E[loader_identity]
R.artifact_hash_binding -> E[artifact_hash_binding]
R.<loader fact field> -> E[<loader fact field>]
R.loader_runtime_facts.<name> -> E[name]

R.<runtime boundary> -> E[<runtime boundary>]
R.live_granted_load_projection.*
  -> E[live_granted_load_projection].facts.*
```

The duplicated runtime-fact copy and the corresponding direct fact method must
come from one core projection. They are two response views, not two evaluators.

### Decision, grants, effects, and first failure

All current allocator/identity/hash/fact/native-runtime decisions are denied:

```text
D.outcome -> constant("denied")
D.requested_capability -> constant("cap.module.load_ephemeral")
D.grants -> constant([])
D.effects -> constant([])
```

Scattered authority fields map as:

```text
can_allocate/can_load/can_load_now=false
authorizes_allocation/authorizes_load/authorizes_execution=false
authorizes_registry_write/authorizes_descriptor_intake=false
authorizes_artifact_byte_intake=false
authorizes_executable_load_plan=false
produces_executable_load_plan=false
  -> retired(D.outcome="denied" and D.grants=[])

allocates_service_slot/creates_service_slot_reservation_records=false
creates_service_inventory_records/creates_service_health_records=false
loads_artifact/load_attempted/maps_executable_pages=false
jumps_to_entrypoint/starts_service/marks_service_running=false
writes_service_start_audit_record/writes_load_commit_audit_record=false
unloads_service/cleans_up_service_slot/commits_live_load=false
installs_commit_rollback_record/records_load_result=false
mutates_service_registry/writes_durable_audit_state=false
installs_rollback_state/registry_write_committed=false
service_inventory_change:"none"
  -> retired(D.effects=[])
```

Descriptor parsing, validation, image-layout, plan, binding, and source-chain
booleans that describe observed readiness remain evidence facts. Booleans that
claim an action was performed or authorized retire into `D` as above. This
distinction must be explicit in the semantic-mapping host test.

`D.blocked_by` is projected from the same ordered evaluator entries as the
evidence array. `D.reason` is the first blocked entry's reason. Legacy aggregate
reasons such as `service_slot_allocator_authority_boundary_non_authorizing`,
`module_loader_*_not_load_authority`,
`module_loader_runtime_behavior_not_implemented`, and other generic
`defined_non_executable` readiness summaries are retained only as the owning
evidence item's reason/status detail when they are genuinely that item's
result; they are retired as top-level decision vocabulary whenever an earlier
blocked item exists.

### Selftest mapping

```text
R.case_count -> F.case_count
R.passed -> F.passed
R.source_fact_count -> F.source_fact_count
R.source_fact_map_complete -> F.source_fact_map_complete
R.source_fact_map -> F.source_fact_map (same entry and field order)
R.cases[].case -> F.cases[].case
R.cases[].expected_status/reason -> F.cases[].expected.{status,reason}
R.cases[].actual_status/reason -> F.cases[].actual.{status,reason}
R.cases[].family-specific expected/actual leaves
  -> F.cases[].expected/actual.<same semantic leaf>
R.cases[].passed -> F.cases[].passed
R/case repeated authority/effect false fields -> retired(F.safety counters)
evidence -> constant([])
D -> constant({outcome:"observed", reason:"selftest_completed"})
```

Required distinct selftest safety carriers are:

```text
F.safety.event_log_write_count = 0
F.safety.external_descriptor_intake_count = 0
F.safety.external_artifact_intake_count = 0
F.safety.service_slot_allocation_count = 0
F.safety.service_slot_reservation_create_count = 0
F.safety.service_inventory_record_create_count = 0
F.safety.service_health_record_create_count = 0
F.safety.artifact_load_count = 0
F.safety.load_attempt_count = 0
F.safety.load_authorization_count = 0
F.safety.registry_mutation_count = 0
F.safety.durable_audit_write_count = 0
F.safety.rollback_install_count = 0
F.safety.entrypoint_transfer_count = 0
F.safety.service_inventory_change = "none"
```

No canonical descriptor, artifact, authority, audit, rollback, reservation, or
event hash grammar changes merely because the response vocabulary changes.

## 3. Evidence-unit design

The v1 array is evaluator-ordered. No renderer may sort it or reconstruct it
from object-field order.

### Allocator units

One item per evaluated boundary, in this exact order:

```text
1  service_slot_reservation
2  service_slot_allocator_runtime
3  service_slot_registry_binding
4  service_health_state_model
5  service_unload_cleanup_plan
6  durable_audit_write
7  rollback_plan_install
8  module_loader
9  service_slot_allocator_authority
10 service_slot_allocation_intent
11 policy_decision
12 registry_write_authority
13 loader_runtime_contract
14 health_monitor_binding
15 unload_cleanup_authority
16 service_slot_allocator_authority_decision
17 service_slot_registry_write_commit_gate
```

The order is the legacy `blocked_by` order at allocator lines 517-590 and the
first-failure chain at lines 3586-3643. Present non-authorizing items remain
evidence; they are not grants.

### Identity, hash binding, and fact units

Identity:

```text
retained_module_evidence -> service_slot_allocator_readiness ->
service_slot_allocator_runtime -> audit_rollback_write_boundary ->
loader_identity
```

Artifact hash binding appends `loader_identity -> artifact_hash_binding` after
the four common prerequisites. Each loader-fact response appends its declared
dependency and then its owned fact after the four common prerequisites. The
direct ordering sources are identity lines 104-135, artifact lines 107-143,
and fact lines 731-772.

### Runtime units

One item is emitted for each execution boundary in evaluator order:

```text
retained prerequisites:
 manifest_reference, artifact_reference, vm_report_reference,
 local_attestation_reference, local_approval_reference,
 computed_grant_reference, audit_rollback_reference,
 service_slot_reservation

allocator:
 service_slot_allocator_readiness

loader facts:
 loader_identity, artifact_hash_binding, entrypoint_abi,
 address_space_boundary, memory_map_constraints,
 capability_import_table, service_slot_binding,
 health_state_hooks, rollback_hooks,
 audit_rollback_write_boundary_binding

execution boundaries:
 execution_commit_gate
 descriptor_intake_boundary
 artifact_byte_intake_boundary
 execution_authorization_boundary
 service_registry_mutation_boundary
 load_attempt_boundary
 artifact_load_boundary
 executable_mapping_boundary
 entrypoint_transfer_boundary
 service_start_boundary
 service_health_binding_boundary
 service_running_state_boundary
 service_start_audit_boundary
 service_unload_cleanup_boundary
 live_load_commit_boundary
 commit_audit_boundary
 commit_rollback_boundary
 commit_result_boundary
 descriptor_acceptance_authority_boundary
 descriptor_parser_contract_boundary
 descriptor_parser_result_boundary
 descriptor_schema_validation_boundary
 descriptor_capability_validation_boundary
 descriptor_load_plan_boundary
 executable_load_plan_authority_boundary
 executable_load_plan_result_boundary
 executable_image_layout_boundary
 executable_page_mapping_plan_boundary
 executable_page_mapping_boundary
 descriptor_executable_page_binding_boundary
 executable_entrypoint_binding_boundary
 executable_entrypoint_transfer_authorization_boundary
 executable_entrypoint_transfer_boundary
 executable_entrypoint_handoff_boundary
 executable_entrypoint_invocation_boundary
```

The complete first-failure chain is
`agent_protocol_module_loader_runtime/eval.rs:686-1096`. Boundary acquisition
and event recording follow the same chain in the root at lines 166-692. The
v1 projection must use one typed ordered slice for `evidence`, `D.blocked_by`,
and `D.reason`.

`E[live_granted_load_projection]` is observational state from a separate Wasm
grant/run path. It comes after native-runtime evidence and must not add grants
or effects to this denied native-loader decision. Its missing provenance is a
decision item in section 6.

## 4. Predicate inventory

Only direct family responses are counted. Runtime counts expand the literal
seven/eight-entry PowerShell loops and `Assert-LogContainsFields` tables.

| Profile | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| `full-module-evidence` | 251 | 427 | 13 | 691 |
| `full-module-selftests` | 433 | 220 | 22 | 675 |
| `m6c-promotion` | 0 | 1 | 1 | 2 |
| `m6d-rollback` | 0 | 1 | 1 | 2 |
| **Total** | **684** | **649** | **37** | **1,370** |

Thus 1,370 predicates were reviewed and 649 are likely regenerated.

### Surviving leaves

These remain exact leaf needles under `facts`, evidence facts, or selftest
case records:

```text
classification:"local_only"
case names, counts, passed
expected/actual statuses and reasons
family reason strings
source_method and source_fact_locator
fact IDs and source-map membership
hashes and ram_only_service_slot_id
present/available/source-chain/binding observations
```

### Regenerated assertions

Regenerate every scoped predicate using:

```text
legacy raios.*.v0 response/object schemas
legacy object anchors "<name>": {
body.result PowerShell paths
legacy event-id property paths
mutates_global_event_log / global_event_log_mutation
can_load/can_load_now/can_allocate
grants_*/authorizes_*/produces_* authority claims
accepts_* when used as an action/authority assertion
allocates_*/creates_*/loads_*/maps_*/jumps_*/starts_*/marks_*
mutates_*/writes_*/installs_*/records_*/commits_*/unloads_*/cleans_*
load_attempted and service_inventory_change
```

The two M6 structured assertions read legacy
`$loaderRuntime.body.result...`; they regenerate to the v1 root/evidence paths.

### Collapse hazards and distinguishing needles

No two distinct safety predicates may be rewritten only to
`D.outcome:"denied"`, `D.effects:[]`, or `D.reason` without a family anchor.

| Legacy assertion | Required distinguishing v1 needle |
|---|---|
| `*_can_load_false`, `*_can_allocate_false`, `*_no_authority` | family anchor plus first-failure `D.reason` / `D.blocked_by[].evidence_id` |
| `*_load_attempted_false`, `*_no_attempt` | selftest `F.safety.load_attempt_count:0`; live `E[load_attempt_boundary].reason` |
| `*_no_load` | selftest `artifact_load_count:0`; live `E[artifact_load_boundary].reason` |
| `*_no_slots`, `*_no_allocation` | `service_slot_allocation_count:0` or allocator boundary reason |
| `*_no_records` | the specific reservation/inventory/health/audit/result count; never one generic zero |
| `*_no_mutation`, `*_inventory_none` | `registry_mutation_count:0` plus `service_inventory_change:"none"`, or registry-boundary reason |
| `*_no_descriptor`, `*_no_artifact_bytes` | the matching intake count plus descriptor/artifact boundary reason |
| `*_no_exec_pages` | executable-page-mapping boundary reason |
| `*_no_entrypoint`, `*_no_handoff`, `*_no_invocation` | corresponding transfer/handoff/invocation evidence reason |
| `*_no_start`, `*_no_running` | service-start/running-state evidence reason |
| `*_no_write`, `*_no_audit_record` | durable/start/commit-audit boundary-specific reason or counter |
| `*_no_install` | rollback-install or commit-rollback reason |
| `*_no_unload`, `*_no_cleanup` | unload-cleanup evidence reason plus the specific action fact |
| `*_no_parse`, `*_no_validation`, `*_no_plan` | the exact parser/schema/capability/load-plan evidence reason and `status_detail` |

The P4-1c failure mode applies directly: a family anchor plus first-failure
decision needle is mandatory for live denial assertions; evidence reason plus
`facts.status_detail` is mandatory when two boundaries share the same common
status.

### Swept but excluded

- `full-module-load-gate` contains direct `module.load_ephemeral` predicates;
  P4-2a already inventoried its nested loader/allocator projections.
- `full-audit` asserts an event response containing compact loader/allocator
  bindings; P4-4 owns those event-family predicates.
- `m12-distribution-provenance` reads loader runtime nested inside a load-gate
  response and remains P4-2 scope.
- Unrelated granted-candidate service, service inventory, rollback-apply, and
  module-reference predicates in M6 profiles are excluded.

## 5. Selftest strategy

| Family | Reference cases | Treatment |
|---|---:|---|
| service-slot allocator | exhaustive table at allocator lines 1970-4299 | move candidate/evaluator/reference table to `raios-core`; regenerate only v1 response/value goldens |
| loader identity | 14 | evaluator semantics and order survive verbatim; regenerate response shape |
| artifact-hash binding | 14 | evaluator semantics and order survive verbatim; regenerate response shape |
| each of eight loader facts | 14 | one shared evaluator table parameterized by the existing spec; all reasons and dependency order survive verbatim |
| loader runtime | exhaustive table in `selftest.rs` | move immutable snapshots, evaluator, boundary evidence projections, and cases; regenerate response shape and safety carriers |

The following semantics survive verbatim:

- present/missing/rejected/available evaluation;
- scope, schema, provenance, binding, and source-chain checks;
- allocator and loader-fact dependency order;
- all reason strings;
- runtime first-failure order;
- dry/defined-non-executable and live-boundary classification;
- canonical hashes and source-map completeness.

The following regenerate:

- envelope and legacy schemas;
- case object nesting into `expected` / `actual`;
- repeated false authority/effect fields into distinct safety counters;
- evidence and decision value goldens;
- direct VM needles and PowerShell property paths.

W4 moves allocator/fact DTOs, missing/available constructors, pure evaluators,
source-evidence projections, and reference matrices. W5 moves runtime immutable
DTOs, pure ordered evaluation, hashes, evidence projections, and reference
matrices. Kernel adapters acquire one coherent snapshot, record returned
evidence, and frame the response.

## 6. Risks and P4-3b STOP-tripwires

1. **Envelope event ID is ambiguous.** Allocator and runtime calls record many
   source-evidence events before emitting one response. P4-3b must not pick the
   last write by accident. Each evidence item has its own `source_event_id`;
   the envelope `event_id` needs the explicit decision below.

2. **Granted projection is a different execution.** The embedded
   `live_granted_load_projection` can be positive while the native loader
   readiness decision is denied. It is observation of separately authorized
   Wasm state, not a grant/effect of this call.

3. **Snapshot tearing.** The current root reads many `latest_*` values and then
   records dependent evidence one event at a time. P4-3b must acquire one
   immutable kernel snapshot before calling core; core must not read the event
   log.

4. **Do not duplicate P4-2.** The load-gate renderer owns its response. P4-3b
   may expose reusable typed projections to it, but must not edit or re-inventory
   its v1 family in this packet.

5. **Retention is not authority.** Event recording is provenance. A denied
   response remains `grants:[]`, `effects:[]` even when source evidence was
   successfully retained.

6. **Status normalization may not erase detail.** Values such as
   `defined_non_authorizing`, `defined_non_executable`, and the many
   `denied_missing_*` states survive in `facts.status_detail`; common status is
   not a lossy replacement.

7. **One ordering source.** If evidence order, `blocked_by`, and first-failure
   reason are built by separate loops, stop. They must project the same typed
   evaluator slice.

8. **No generic policy framework.** Reuse `record_table` and
   `evidence_response`; add only family-local tables/projections.

9. **Kernel-only apply boundary.** The kernel retains event-log locks and
   reads/writes, candidate-byte access, page allocation and executable page
   mapping, entrypoint transfer/invocation, service start, service registry and
   inventory mutation, persistent/media writes, rollback/apply, and serial
   framing. Moving any of these into `raios-core` is a STOP condition.

10. **Core-only pure boundary.** W4-W5 may relocate immutable DTOs, snapshot
    projections after acquisition, evaluation, first-failure tables, evidence
    construction, canonical hashes, and reference cases. Core returns a plan or
    decision; it never applies one.

11. **Positive authority needs proof.** No currently constant false field may
    become a grant/effect merely because all diagnostic inputs are present.
    Any positive native-load decision requires an evaluator-created
    `GrantProof` and a separate kernel apply result.

12. **Do not mutate canonical grammars.** Response vocabulary regeneration is
    not authorization to change descriptor/artifact/audit/rollback hashes or
    embedded canonical false lines.

### OWNER/ORCHESTRATOR DECISION NEEDED

1. **Which event binds the envelope?** Choose one explicit rule for allocator
   and runtime responses: (a) record one new response-summary event after the
   evaluator result and use it as top-level `event_id`, or (b) set top-level
   `event_id:null` and rely exclusively on per-item `source_event_id`. Do not
   alias an arbitrary last boundary event to the whole response.

2. **How is `live_granted_load_projection` provenance-bound?** Choose either
   (a) keep it as `E[live_granted_load_projection]` with a real source event ID
   supplied by the granted-candidate owner, or (b) remove it from this denied
   family and expose it through its owning observational family. Until chosen,
   P4-3b must not claim source-equivalent v1 evidence for this projection.

No loader emission path materially contradicts the P4 family table once
retention is treated as provenance and the granted projection is treated as a
separate observation. The two provenance choices above are the only blocking
design gaps found.

### Static-check evidence

Required emitter scan (the helper owns the literal framing string):

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src/agent_protocol_module_loader_identity.rs seed-kernel/src/agent_protocol_module_loader_artifact_hash_binding.rs seed-kernel/src/agent_protocol_module_loader_fact.rs seed-kernel/src/agent_protocol_module_loader_runtime.rs seed-kernel/src/agent_protocol_module_loader_runtime/*.rs seed-kernel/src/agent_protocol_module_service_slot_allocator.rs seed-kernel/src/agent_protocol_module_service_slot_allocator_projection.rs
rg: seed-kernel/src/agent_protocol_module_loader_runtime/*.rs: Die Syntax für den Dateinamen, Verzeichnisnamen oder die Datenträgerbezeichnung ist falsch. (os error 123)
```

The Windows glob was then expanded explicitly; zero matches produced no output
and `rg` exit code 1:

```text
> $files=@('seed-kernel/src/agent_protocol_module_loader_identity.rs','seed-kernel/src/agent_protocol_module_loader_artifact_hash_binding.rs','seed-kernel/src/agent_protocol_module_loader_fact.rs','seed-kernel/src/agent_protocol_module_loader_runtime.rs','seed-kernel/src/agent_protocol_module_service_slot_allocator.rs','seed-kernel/src/agent_protocol_module_service_slot_allocator_projection.rs') + (Get-ChildItem seed-kernel/src/agent_protocol_module_loader_runtime -File | % FullName); rg -c "RAIOS_AGENT_BEGIN" $files
<no output; exit 1>
```

Required harness sweep:

```text
> rg -l "loader_runtime|service_slot_allocator|loader_fact|loader_identity" vm-harness/
vm-harness/shadow-vm-smoke-profile-full-audit.ps1
vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1
vm-harness/shadow-vm-smoke-profile-full-module-load-gate.ps1
vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
vm-harness/shadow-vm-smoke-profile-m12-distribution-provenance.ps1
vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1
vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1
```

Needle-count scan used after family scoping. It counts exact
`Assert-LogContains`, expanded `Assert-LogContainsFields` entries, and
structured `Add-Predicate`; loop multipliers are the literal seven fact-source
diagnostics and eight fact/selftest specs:

```text
> $regen='\"schema\"|\": \{|mutates_|allocates_|creates_|loads_|service_inventory_change|can_allocate|can_load|load_attempted|authorizes_|accepts_|produces_|validates_|parses_|maps_|jumps_|starts_|marks_|writes_|installs_|records_|commits_|unloads_|cleans_|event_id|source_evidence_event|global_event_log_mutation'; function C($p,$a,$b,$m=1){$x=Get-Content $p; $s=$x[($a-1)..($b-1)] | ? {($_ -match '^\s*Assert-LogContains\s+-') -or ($_ -match '^\s*Add-Predicate\s+-') -or ($_ -match '@\{ Suffix =')}; $r=($s|?{$_ -match '^\s*Add-Predicate' -or $_ -match $regen}).Count*$m; $l=($s.Count*$m)-$r; "${a}-${b} x$m leaf=$l regen=$r assertions=$($s.Count*$m)"}

> $p='vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1'; C $p 558 616; C $p 638 696; C $p 697 718; C $p 719 742; C $p 743 775; C $p 776 799 7; C $p 800 1208
558-616 x1 leaf=13 regen=24 assertions=37
638-696 x1 leaf=36 regen=19 assertions=55
697-718 x1 leaf=8 regen=10 assertions=18
719-742 x1 leaf=9 regen=11 assertions=20
743-775 x1 leaf=8 regen=11 assertions=19
776-799 x7 leaf=56 regen=77 assertions=133
800-1208 x1 leaf=121 regen=275 assertions=396

> $p='vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1'; C $p 72 144; C $p 145 322; C $p 323 336; C $p 337 364; C $p 365 384; C $p 385 435; C $p 436 486 8
72-144 x1 leaf=58 regen=10 assertions=68
145-322 x1 leaf=164 regen=10 assertions=174
323-336 x1 leaf=3 regen=9 assertions=12
337-364 x1 leaf=17 regen=9 assertions=26
365-384 x1 leaf=5 regen=13 assertions=18
385-435 x1 leaf=18 regen=9 assertions=27
436-486 x8 leaf=168 regen=160 assertions=328

> direct structured/framing scan for m6c-promotion and m6d-rollback
vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1: leaf=0 regenerate=1 framing=1 total=2
vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1: leaf=0 regenerate=1 framing=1 total=2
```

The classification regex treated legacy schemas/object anchors, legacy event
paths, and all scattered authority/effect action fields as regeneration. Exact
classification, reason, case, count, source locator, fact ID, and hash leaves
were classified as surviving. Framing counts are the runtime-expanded direct
family command completions, not source literals.

## P4-3b1 rulings

- Allocator and runtime observational responses use top-level `event_id:null`;
  provenance lives only on each evidence item's `source_event_id`. No response-
  summary event is recorded and no boundary event is aliased to the response.
- `live_granted_load_projection` moves out of the denied loader family to its
  owning granted-candidate observational family. P4-3 loader-family core
  evidence does not include it.

## P4-3b2 notes

Capability sentence: agents can now inspect direct allocator, loader-identity,
artifact-binding, loader-fact, and loader-runtime readiness through the typed
`raios.evidence_response.v1` envelope and evaluator-ordered evidence chain.

- Direct family envelopes bind `event_id:null`; only evidence items carry
  `source_event_id` provenance.
- `live_granted_load_projection` is not rendered by `module.loader_runtime`.
  The granted-candidate owner retains its pre-conversion response vocabulary.
- The kernel captures each response candidate/evaluation once, passes named
  typed inputs to `module_loader_allocator_projection`, and renders the core
  projection with the shared evidence-v1 response emitter.
- Direct legacy emitters were removed. `module_loader_runtime/render.rs` is
  intentionally reduced to a boundary note; P4-4-owned `memory.recent_events`
  renderers were not changed.

### Harness predicate disposition — completed STOP-10 accounting

The 942-assertion HEAD contract fell to 621 after P4-3b2. This repair adds
119 distinct carriers (115 unique case tuples plus four direct fact/evidence
needles), producing 735 runtime-expanded assertions. All 300 dropped literal
predicate names are assigned exactly once:

| Bucket | Count |
|---|---:|
| regenerate | 119 |
| honest merge | 130 |
| explicit retire | 51 |
| **accounted** | **300** |

#### Regenerated predicates (name -> byte-exact v1 needle)

- protocol:module_service_slot_allocator_selftest_missing_reservation_case -> "case": "missing_retained_service_slot_reservation", "expected": {"status": "missing", "reason": "retained_service_slot_reservation_missing"}, "actual": {"status": "missing", "reason": "retained_service_slot_reservation_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_allocator_missing_case -> "case": "service_slot_allocator_runtime_missing", "expected": {"status": "missing", "reason": "service_slot_allocator_runtime_missing"}, "actual": {"status": "missing", "reason": "service_slot_allocator_runtime_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_allocator_observed_case -> "case": "service_slot_allocator_runtime_observed_source_evidence_missing", "expected": {"status": "missing", "reason": "service_slot_allocator_runtime_missing"}, "actual": {"status": "missing", "reason": "service_slot_allocator_runtime_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_allocator_observed_available_case -> "case": "service_slot_allocator_runtime_observed_source_evidence_available_registry_missing", "expected": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "actual": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_registry_missing_case -> "case": "service_slot_registry_binding_missing", "expected": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "actual": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_registry_observed_case -> "case": "service_slot_registry_binding_observed_source_evidence_missing", "expected": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "actual": {"status": "missing", "reason": "service_slot_registry_binding_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_registry_observed_available_case -> "case": "service_slot_registry_binding_observed_source_evidence_available_health_missing", "expected": {"status": "missing", "reason": "service_health_state_model_missing"}, "actual": {"status": "missing", "reason": "service_health_state_model_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_health_case -> "case": "service_health_state_model_missing", "expected": {"status": "missing", "reason": "service_health_state_model_missing"}, "actual": {"status": "missing", "reason": "service_health_state_model_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_health_observed_case -> "case": "service_health_state_model_observed_source_evidence_missing", "expected": {"status": "missing", "reason": "service_health_state_model_missing"}, "actual": {"status": "missing", "reason": "service_health_state_model_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_health_observed_available_case -> "case": "service_health_state_model_observed_source_evidence_available_unload_missing", "expected": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "actual": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_cleanup_case -> "case": "service_unload_cleanup_plan_missing", "expected": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "actual": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_cleanup_observed_case -> "case": "service_unload_cleanup_plan_observed_source_evidence_missing", "expected": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "actual": {"status": "missing", "reason": "service_unload_cleanup_plan_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_cleanup_observed_available_case -> "case": "service_unload_cleanup_plan_observed_source_evidence_available_durable_missing", "expected": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "actual": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_durable_case -> "case": "durable_audit_write_missing", "expected": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "actual": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_durable_observed_case -> "case": "durable_audit_write_observed_source_evidence_missing", "expected": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "actual": {"status": "denied_missing_durable_audit_write", "reason": "durable_audit_write_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_durable_observed_available_case -> "case": "durable_audit_write_observed_source_evidence_available_rollback_missing", "expected": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "actual": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_rollback_case -> "case": "rollback_install_missing", "expected": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "actual": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_rollback_observed_case -> "case": "rollback_install_observed_source_evidence_missing", "expected": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "actual": {"status": "denied_missing_rollback_install", "reason": "rollback_install_missing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_rollback_observed_available_case -> "case": "rollback_install_observed_source_evidence_available_module_loader_unimplemented", "expected": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "actual": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "passed": true
- protocol:module_service_slot_allocator_selftest_loader_case -> "case": "module_loader_missing", "expected": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "actual": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "passed": true
- protocol:module_service_slot_allocator_selftest_loader_observed_case -> "case": "module_loader_observed_source_evidence_missing", "expected": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "actual": {"status": "denied_loader_unimplemented", "reason": "module_loader_unimplemented"}, "passed": true
- protocol:module_service_slot_allocator_selftest_loader_observed_available_case -> "case": "module_loader_observed_source_evidence_available_allocator_authority_boundary", "expected": {"status": "denied_allocator_authority_not_granted", "reason": "service_slot_allocator_authority_boundary_non_authorizing"}, "actual": {"status": "denied_allocator_authority_not_granted", "reason": "service_slot_allocator_authority_boundary_non_authorizing"}, "passed": true
- protocol:module_service_slot_allocator_selftest_registry_commit_gate_case -> "case": "registry_write_commit_gate_missing", "expected": {"status": "missing", "reason": "service_slot_registry_write_commit_gate_source_chain_incomplete"}, "actual": {"status": "missing", "reason": "service_slot_registry_write_commit_gate_source_chain_incomplete"}, "passed": true
- protocol:module_service_slot_allocator_selftest_ready_case -> "case": "all_inputs_ready_still_non_authorizing", "expected": {"status": "denied_allocator_authority_not_granted", "reason": "service_slot_allocator_authority_boundary_non_authorizing"}, "actual": {"status": "denied_allocator_authority_not_granted", "reason": "service_slot_allocator_authority_boundary_non_authorizing"}, "passed": true
- protocol:module_loader_runtime_selftest_missing_manifest_case -> "case": "missing_manifest_reference", "expected": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_manifest_reference_missing"}, "actual": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_manifest_reference_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_allocator_readiness_case -> "case": "missing_service_slot_allocator_readiness", "expected": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_allocator_runtime_case -> "case": "service_slot_allocator_runtime_missing", "expected": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_scope_case -> "case": "loader_identity_previous_boot", "expected": {"status": "rejected", "reason": "module_loader_identity_scope_must_be_current_boot"}, "actual": {"status": "rejected", "reason": "module_loader_identity_scope_must_be_current_boot"}, "passed": true
- protocol:module_loader_runtime_selftest_schema_case -> "case": "loader_identity_wrong_schema", "expected": {"status": "rejected", "reason": "module_loader_identity_schema_mismatch"}, "actual": {"status": "rejected", "reason": "module_loader_identity_schema_mismatch"}, "passed": true
- protocol:module_loader_runtime_selftest_provenance_case -> "case": "loader_identity_provenance_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_provenance_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_provenance_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_retained_binding_case -> "case": "loader_identity_retained_evidence_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_retained_evidence_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_retained_evidence_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_allocator_binding_case -> "case": "loader_identity_service_slot_allocator_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_service_slot_allocator_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_service_slot_allocator_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_audit_binding_case -> "case": "loader_identity_audit_write_boundary_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_audit_write_boundary_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_audit_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_identity_source_evidence_case -> "case": "loader_identity_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_identity_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_identity_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_artifact_hash_case -> "case": "artifact_hash_binding_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_artifact_hash_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_artifact_hash_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_artifact_hash_source_evidence_case -> "case": "artifact_hash_binding_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_artifact_hash_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_artifact_hash_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_entrypoint_case -> "case": "entrypoint_abi_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_entrypoint_abi_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_entrypoint_abi_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_entrypoint_source_evidence_case -> "case": "entrypoint_abi_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_entrypoint_abi_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_entrypoint_abi_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_address_space_case -> "case": "address_space_boundary_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_address_space_boundary_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_address_space_boundary_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_address_space_source_evidence_case -> "case": "address_space_boundary_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_address_space_boundary_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_address_space_boundary_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_memory_map_case -> "case": "memory_map_constraints_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_memory_map_constraints_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_memory_map_constraints_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_memory_map_source_evidence_case -> "case": "memory_map_constraints_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_memory_map_constraints_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_memory_map_constraints_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_capability_table_case -> "case": "capability_import_table_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_capability_import_table_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_capability_import_table_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_capability_table_source_evidence_case -> "case": "capability_import_table_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_capability_import_table_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_capability_import_table_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_service_slot_case -> "case": "service_slot_binding_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_service_slot_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_service_slot_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_service_slot_source_evidence_case -> "case": "service_slot_binding_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_service_slot_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_service_slot_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_health_case -> "case": "health_state_hooks_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_health_state_hooks_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_health_state_hooks_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_health_source_evidence_case -> "case": "health_state_hooks_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_health_state_hooks_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_health_state_hooks_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_rollback_case -> "case": "rollback_hooks_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_rollback_hooks_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_rollback_hooks_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_rollback_source_evidence_case -> "case": "rollback_hooks_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_rollback_hooks_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_rollback_hooks_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_write_boundary_case -> "case": "audit_rollback_write_boundary_binding_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_audit_rollback_write_boundary_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_audit_rollback_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_write_boundary_source_evidence_case -> "case": "audit_rollback_write_boundary_binding_observed_source_evidence_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_audit_rollback_write_boundary_binding_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_audit_rollback_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_runtime_selftest_execution_commit_gate_case -> "case": "execution_commit_gate_missing", "expected": {"status": "denied_missing_module_loader_runtime_execution_commit_gate", "reason": "module_loader_runtime_execution_commit_gate_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_runtime_execution_commit_gate", "reason": "module_loader_runtime_execution_commit_gate_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_intake_boundary_case -> "case": "descriptor_intake_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_intake_boundary", "reason": "module_loader_descriptor_intake_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_intake_boundary", "reason": "module_loader_descriptor_intake_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_artifact_byte_intake_boundary_case -> "case": "artifact_byte_intake_boundary_missing", "expected": {"status": "denied_missing_module_loader_artifact_byte_intake_boundary", "reason": "module_loader_artifact_byte_intake_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_artifact_byte_intake_boundary", "reason": "module_loader_artifact_byte_intake_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_execution_authorization_boundary_case -> "case": "execution_authorization_boundary_missing", "expected": {"status": "denied_missing_module_loader_execution_authorization_boundary", "reason": "module_loader_execution_authorization_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_execution_authorization_boundary", "reason": "module_loader_execution_authorization_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_registry_mutation_boundary_case -> "case": "service_registry_mutation_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_registry_mutation_boundary", "reason": "module_loader_service_registry_mutation_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_registry_mutation_boundary", "reason": "module_loader_service_registry_mutation_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_load_attempt_boundary_case -> "case": "load_attempt_boundary_missing", "expected": {"status": "denied_missing_module_loader_load_attempt_boundary", "reason": "module_loader_load_attempt_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_load_attempt_boundary", "reason": "module_loader_load_attempt_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_artifact_load_boundary_case -> "case": "artifact_load_boundary_missing", "expected": {"status": "denied_missing_module_loader_artifact_load_boundary", "reason": "module_loader_artifact_load_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_artifact_load_boundary", "reason": "module_loader_artifact_load_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_mapping_boundary_case -> "case": "executable_mapping_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_mapping_boundary", "reason": "module_loader_executable_mapping_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_mapping_boundary", "reason": "module_loader_executable_mapping_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_entrypoint_transfer_boundary_case -> "case": "entrypoint_transfer_boundary_missing", "expected": {"status": "denied_missing_module_loader_entrypoint_transfer_boundary", "reason": "module_loader_entrypoint_transfer_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_entrypoint_transfer_boundary", "reason": "module_loader_entrypoint_transfer_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_start_boundary_case -> "case": "service_start_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_start_boundary", "reason": "module_loader_service_start_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_start_boundary", "reason": "module_loader_service_start_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_health_binding_boundary_case -> "case": "service_health_binding_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_health_binding_boundary", "reason": "module_loader_service_health_binding_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_health_binding_boundary", "reason": "module_loader_service_health_binding_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_running_state_boundary_case -> "case": "service_running_state_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_running_state_boundary", "reason": "module_loader_service_running_state_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_running_state_boundary", "reason": "module_loader_service_running_state_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_start_audit_boundary_case -> "case": "service_start_audit_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_start_audit_boundary", "reason": "module_loader_service_start_audit_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_start_audit_boundary", "reason": "module_loader_service_start_audit_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_service_unload_cleanup_boundary_case -> "case": "service_unload_cleanup_boundary_missing", "expected": {"status": "denied_missing_module_loader_service_unload_cleanup_boundary", "reason": "module_loader_service_unload_cleanup_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_service_unload_cleanup_boundary", "reason": "module_loader_service_unload_cleanup_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_live_load_commit_boundary_case -> "case": "live_load_commit_boundary_missing", "expected": {"status": "denied_missing_module_loader_live_load_commit_boundary", "reason": "module_loader_live_load_commit_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_live_load_commit_boundary", "reason": "module_loader_live_load_commit_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_commit_audit_boundary_case -> "case": "commit_audit_boundary_missing", "expected": {"status": "denied_missing_module_loader_commit_audit_boundary", "reason": "module_loader_commit_audit_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_commit_audit_boundary", "reason": "module_loader_commit_audit_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_commit_rollback_boundary_case -> "case": "commit_rollback_boundary_missing", "expected": {"status": "denied_missing_module_loader_commit_rollback_boundary", "reason": "module_loader_commit_rollback_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_commit_rollback_boundary", "reason": "module_loader_commit_rollback_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_commit_result_boundary_case -> "case": "commit_result_boundary_missing", "expected": {"status": "denied_missing_module_loader_commit_result_boundary", "reason": "module_loader_commit_result_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_commit_result_boundary", "reason": "module_loader_commit_result_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_acceptance_authority_boundary_case -> "case": "descriptor_acceptance_authority_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_acceptance_authority_boundary", "reason": "module_loader_descriptor_acceptance_authority_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_acceptance_authority_boundary", "reason": "module_loader_descriptor_acceptance_authority_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_parser_contract_boundary_case -> "case": "descriptor_parser_contract_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_parser_contract_boundary", "reason": "module_loader_descriptor_parser_contract_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_parser_contract_boundary", "reason": "module_loader_descriptor_parser_contract_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_parser_result_boundary_case -> "case": "descriptor_parser_result_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_parser_result_boundary", "reason": "module_loader_descriptor_parser_result_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_parser_result_boundary", "reason": "module_loader_descriptor_parser_result_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_schema_validation_boundary_case -> "case": "descriptor_schema_validation_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_schema_validation_boundary", "reason": "module_loader_descriptor_schema_validation_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_schema_validation_boundary", "reason": "module_loader_descriptor_schema_validation_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_capability_validation_boundary_case -> "case": "descriptor_capability_validation_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_capability_validation_boundary", "reason": "module_loader_descriptor_capability_validation_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_capability_validation_boundary", "reason": "module_loader_descriptor_capability_validation_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_load_plan_boundary_case -> "case": "descriptor_load_plan_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_load_plan_boundary", "reason": "module_loader_descriptor_load_plan_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_load_plan_boundary", "reason": "module_loader_descriptor_load_plan_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_load_plan_authority_boundary_case -> "case": "executable_load_plan_authority_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_load_plan_authority_boundary", "reason": "module_loader_executable_load_plan_authority_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_load_plan_authority_boundary", "reason": "module_loader_executable_load_plan_authority_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_load_plan_result_boundary_case -> "case": "executable_load_plan_result_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_load_plan_result_boundary", "reason": "module_loader_executable_load_plan_result_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_load_plan_result_boundary", "reason": "module_loader_executable_load_plan_result_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_image_layout_boundary_case -> "case": "executable_image_layout_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_image_layout_boundary", "reason": "module_loader_executable_image_layout_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_image_layout_boundary", "reason": "module_loader_executable_image_layout_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_page_mapping_plan_boundary_case -> "case": "executable_page_mapping_plan_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_page_mapping_plan_boundary", "reason": "module_loader_executable_page_mapping_plan_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_page_mapping_plan_boundary", "reason": "module_loader_executable_page_mapping_plan_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_page_mapping_boundary_case -> "case": "executable_page_mapping_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_page_mapping_boundary", "reason": "module_loader_executable_page_mapping_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_page_mapping_boundary", "reason": "module_loader_executable_page_mapping_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_descriptor_executable_page_binding_boundary_case -> "case": "descriptor_executable_page_binding_boundary_missing", "expected": {"status": "denied_missing_module_loader_descriptor_executable_page_binding_boundary", "reason": "module_loader_descriptor_executable_page_binding_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_descriptor_executable_page_binding_boundary", "reason": "module_loader_descriptor_executable_page_binding_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_entrypoint_binding_boundary_case -> "case": "executable_entrypoint_binding_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_entrypoint_binding_boundary", "reason": "module_loader_executable_entrypoint_binding_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_entrypoint_binding_boundary", "reason": "module_loader_executable_entrypoint_binding_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_authorization_boundary_case -> "case": "executable_entrypoint_transfer_authorization_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_entrypoint_transfer_authorization_boundary", "reason": "module_loader_executable_entrypoint_transfer_authorization_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_entrypoint_transfer_authorization_boundary", "reason": "module_loader_executable_entrypoint_transfer_authorization_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_boundary_case -> "case": "executable_entrypoint_transfer_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_entrypoint_transfer_boundary", "reason": "module_loader_executable_entrypoint_transfer_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_entrypoint_transfer_boundary", "reason": "module_loader_executable_entrypoint_transfer_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_entrypoint_handoff_boundary_case -> "case": "executable_entrypoint_handoff_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_entrypoint_handoff_boundary", "reason": "module_loader_executable_entrypoint_handoff_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_entrypoint_handoff_boundary", "reason": "module_loader_executable_entrypoint_handoff_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_executable_entrypoint_invocation_boundary_case -> "case": "executable_entrypoint_invocation_boundary_missing", "expected": {"status": "denied_missing_module_loader_executable_entrypoint_invocation_boundary", "reason": "module_loader_executable_entrypoint_invocation_boundary_source_chain_incomplete"}, "actual": {"status": "denied_missing_module_loader_executable_entrypoint_invocation_boundary", "reason": "module_loader_executable_entrypoint_invocation_boundary_source_chain_incomplete"}, "passed": true
- protocol:module_loader_runtime_selftest_ready_case -> "case": "all_inputs_ready_defined_non_executable", "expected": {"status": "defined_non_executable", "reason": "module_loader_runtime_behavior_not_implemented"}, "actual": {"status": "defined_non_executable", "reason": "module_loader_runtime_behavior_not_implemented"}, "passed": true
- protocol:module_loader_identity_identity_missing -> "id": "loader_identity", "kind": "loader_fact", "status": "missing", "reason": "module_loader_identity_missing", "source_event_id": "event.current_boot.
- protocol:module_loader_identity_fact_id -> "record_schema": "raios.module_loader_identity.v0", "record_id": "module.loader_runtime.identity.current_boot", "source_method": "module.loader_identity", "source_fact_locator": "module.loader_identity.loader_identity"
- protocol:module_loader_identity_selftest_missing_evidence_case -> "case": "missing_retained_module_evidence", "expected": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_evidence_missing"}, "actual": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_evidence_missing"}, "passed": true
- protocol:module_loader_identity_selftest_allocator_readiness_case -> "case": "missing_service_slot_allocator_readiness", "expected": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "passed": true
- protocol:module_loader_identity_selftest_allocator_runtime_case -> "case": "service_slot_allocator_runtime_missing", "expected": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "passed": true
- protocol:module_loader_identity_selftest_audit_boundary_case -> "case": "audit_write_boundary_missing", "expected": {"status": "denied_missing_audit_rollback_write_boundary", "reason": "module_audit_rollback_write_boundary_binding_missing"}, "actual": {"status": "denied_missing_audit_rollback_write_boundary", "reason": "module_audit_rollback_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_identity_selftest_scope_case -> "case": "loader_identity_previous_boot", "expected": {"status": "rejected", "reason": "module_loader_identity_scope_must_be_current_boot"}, "actual": {"status": "rejected", "reason": "module_loader_identity_scope_must_be_current_boot"}, "passed": true
- protocol:module_loader_identity_selftest_schema_case -> "case": "loader_identity_schema_mismatch", "expected": {"status": "rejected", "reason": "module_loader_identity_schema_mismatch"}, "actual": {"status": "rejected", "reason": "module_loader_identity_schema_mismatch"}, "passed": true
- protocol:module_loader_identity_selftest_provenance_case -> "case": "loader_identity_provenance_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_provenance_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_provenance_missing"}, "passed": true
- protocol:module_loader_identity_selftest_retained_binding_case -> "case": "loader_identity_retained_evidence_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_retained_evidence_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_retained_evidence_binding_missing"}, "passed": true
- protocol:module_loader_identity_selftest_allocator_binding_case -> "case": "loader_identity_service_slot_allocator_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_service_slot_allocator_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_service_slot_allocator_binding_missing"}, "passed": true
- protocol:module_loader_identity_selftest_audit_binding_case -> "case": "loader_identity_audit_write_boundary_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_identity_audit_write_boundary_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_identity_audit_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_identity_selftest_missing_identity_case -> "case": "loader_identity_missing", "expected": {"status": "denied_missing_loader_identity", "reason": "module_loader_identity_missing"}, "actual": {"status": "denied_missing_loader_identity", "reason": "module_loader_identity_missing"}, "passed": true
- protocol:module_loader_identity_selftest_ready_case -> "case": "all_inputs_present_identity_non_authorizing", "expected": {"status": "available_non_authorizing", "reason": "module_loader_identity_not_load_authority"}, "actual": {"status": "available_non_authorizing", "reason": "module_loader_identity_not_load_authority"}, "passed": true
- protocol:module_loader_artifact_hash_binding_missing -> "id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing", "reason": "module_loader_artifact_hash_binding_missing", "source_event_id": "event.current_boot.
- protocol:module_loader_artifact_hash_binding_fact_id -> "record_schema": "raios.module_loader_artifact_hash_binding.v0", "record_id": "module.loader_runtime.artifact_hash_binding.current_boot", "source_method": "module.loader_artifact_hash_binding", "source_fact_locator": "module.loader_artifact_hash_binding.artifact_hash_binding"
- protocol:module_loader_artifact_hash_binding_fact_source_event -> "id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing", "reason": "module_loader_artifact_hash_binding_missing", "source_event_id": "event.current_boot.
- protocol:module_loader_artifact_hash_binding_fact_source_state -> "id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing", "reason": "module_loader_artifact_hash_binding_missing", "source_event_id": "event.current_boot.
- protocol:module_loader_artifact_hash_binding_selftest_missing_evidence_case -> "case": "missing_retained_module_evidence", "expected": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_evidence_missing"}, "actual": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_evidence_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_allocator_readiness_case -> "case": "missing_service_slot_allocator_readiness", "expected": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_readiness", "reason": "service_slot_allocator_readiness_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_allocator_runtime_case -> "case": "service_slot_allocator_runtime_missing", "expected": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "actual": {"status": "denied_missing_service_slot_allocator_runtime", "reason": "service_slot_allocator_runtime_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_loader_identity_case -> "case": "loader_identity_missing", "expected": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_identity_missing"}, "actual": {"status": "denied_missing_loader_runtime_fact", "reason": "module_loader_identity_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_scope_case -> "case": "artifact_hash_binding_previous_boot", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_scope_must_be_current_boot"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_scope_must_be_current_boot"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_schema_case -> "case": "artifact_hash_binding_schema_mismatch", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_schema_mismatch"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_schema_mismatch"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_provenance_case -> "case": "artifact_hash_binding_provenance_missing", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_provenance_missing"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_provenance_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_retained_binding_case -> "case": "artifact_hash_binding_retained_evidence_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_retained_evidence_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_retained_evidence_binding_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_allocator_binding_case -> "case": "artifact_hash_binding_service_slot_allocator_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_audit_binding_case -> "case": "artifact_hash_binding_audit_write_boundary_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_identity_binding_case -> "case": "artifact_hash_binding_loader_identity_binding_missing", "expected": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_loader_identity_binding_missing"}, "actual": {"status": "rejected", "reason": "module_loader_artifact_hash_binding_loader_identity_binding_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_missing_case -> "case": "artifact_hash_binding_missing", "expected": {"status": "denied_missing_loader_artifact_hash_binding", "reason": "module_loader_artifact_hash_binding_missing"}, "actual": {"status": "denied_missing_loader_artifact_hash_binding", "reason": "module_loader_artifact_hash_binding_missing"}, "passed": true
- protocol:module_loader_artifact_hash_binding_selftest_ready_case -> "case": "all_inputs_present_artifact_hash_binding_non_authorizing", "expected": {"status": "available_non_authorizing", "reason": "module_loader_artifact_hash_binding_not_load_authority"}, "actual": {"status": "available_non_authorizing", "reason": "module_loader_artifact_hash_binding_not_load_authority"}, "passed": true
#### Honestly merged predicates (full table)

| Legacy predicate | Surviving carrier |
|---|---|
| protocol:module_service_slot_allocator_selftest_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_service_slot_allocator_selftest_missing_reservation_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_allocator_missing_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_allocator_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_allocator_source_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_allocator_source_available | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_allocator_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_registry_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_registry_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_registry_binding_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_health_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_health_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_cleanup_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_cleanup_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_durable_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_durable_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_rollback_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_rollback_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_loader_available_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_loader_source_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_registry_commit_gate_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_service_slot_allocator_selftest_ready_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_loader_runtime_selftest_source_map_complete | surviving source_fact_map_complete:true |
| protocol:module_loader_runtime_selftest_missing_manifest_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_allocator_runtime_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_identity_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_identity_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_hash_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_hash_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_entrypoint_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_entrypoint_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_address_space_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_address_space_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_memory_map_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_memory_map_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_capability_table_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_capability_table_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_slot_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_slot_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_health_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_health_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_rollback_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_rollback_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_write_boundary_source_evidence_present | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_write_boundary_source_evidence_observed | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_execution_commit_gate_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_execution_commit_gate_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_intake_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_intake_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_byte_intake_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_byte_intake_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_execution_authorization_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_execution_authorization_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_registry_mutation_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_registry_mutation_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_load_attempt_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_load_attempt_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_load_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_artifact_load_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_mapping_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_mapping_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_entrypoint_transfer_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_entrypoint_transfer_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_start_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_start_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_health_binding_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_health_binding_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_running_state_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_running_state_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_start_audit_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_start_audit_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_unload_cleanup_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_service_unload_cleanup_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_live_load_commit_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_live_load_commit_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_audit_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_audit_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_rollback_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_rollback_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_result_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_commit_result_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_acceptance_authority_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_acceptance_authority_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_parser_contract_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_parser_contract_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_parser_result_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_parser_result_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_schema_validation_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_schema_validation_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_capability_validation_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_capability_validation_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_load_plan_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_load_plan_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_load_plan_authority_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_load_plan_authority_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_load_plan_result_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_load_plan_result_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_image_layout_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_image_layout_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_page_mapping_plan_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_page_mapping_plan_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_page_mapping_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_page_mapping_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_executable_page_binding_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_descriptor_executable_page_binding_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_binding_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_binding_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_authorization_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_authorization_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_transfer_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_handoff_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_handoff_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_invocation_boundary_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_executable_entrypoint_invocation_boundary_reason | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_runtime_selftest_ready_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_identity_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_loader_identity_selftest_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_loader_identity_selftest_passed | surviving passed:true |
| protocol:module_loader_identity_selftest_missing_identity_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_identity_selftest_ready_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_artifact_hash_binding_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_loader_artifact_hash_binding_source_evidence_schema | owned artifact_hash_binding evidence item |
| protocol:module_loader_artifact_hash_binding_source_evidence_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_artifact_hash_binding_source_evidence_event | owned artifact_hash_binding evidence item |
| protocol:module_loader_artifact_hash_binding_selftest_local_only | family v1 classification:local_only merged with surviving framing |
| protocol:module_loader_artifact_hash_binding_selftest_passed | surviving passed:true |
| protocol:module_loader_artifact_hash_binding_selftest_missing_status | corresponding regenerated tuple (expected, nested actual, passed) |
| protocol:module_loader_artifact_hash_binding_selftest_ready_status | corresponding regenerated tuple (expected, nested actual, passed) |

#### Explicitly retired predicates (full table)

| Legacy predicate | Replacement carrier |
|---|---|
| protocol:module_service_slot_allocator_selftest_no_mutation | safety.event_log_write_count:0 |
| protocol:module_service_slot_allocator_selftest_no_records | safety.retained_record_create_count:0 |
| protocol:module_service_slot_allocator_selftest_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_service_slot_allocator_selftest_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_service_slot_allocator_selftest_no_load | safety.artifact_load_count:0 |
| protocol:module_service_slot_allocator_selftest_inventory_none | safety.service_inventory_change:none |
| protocol:module_service_slot_allocator_selftest_can_allocate_false | safety.service_slot_allocation_count:0 |
| protocol:module_service_slot_allocator_selftest_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_service_slot_allocator_selftest_load_attempted_false | safety.load_attempt_count:0 |
| protocol:module_loader_runtime_selftest_no_mutation | safety.event_log_write_count:0 |
| protocol:module_loader_runtime_selftest_no_descriptor | safety.external_artifact_intake_count:0 |
| protocol:module_loader_runtime_selftest_no_artifact_bytes | safety.external_artifact_intake_count:0 |
| protocol:module_loader_runtime_selftest_no_load | safety.artifact_load_count:0 |
| protocol:module_loader_runtime_selftest_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_loader_runtime_selftest_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_loader_runtime_selftest_inventory_none | safety.service_inventory_change:none |
| protocol:module_loader_runtime_selftest_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_loader_runtime_selftest_load_attempted_false | safety.load_attempt_count:0 |
| protocol:module_loader_identity_no_mutation | safety.event_log_write_count:0 |
| protocol:module_loader_identity_no_descriptor | safety.external_artifact_intake_count:0 |
| protocol:module_loader_identity_no_artifact_bytes | safety.external_artifact_intake_count:0 |
| protocol:module_loader_identity_no_load | safety.artifact_load_count:0 |
| protocol:module_loader_identity_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_loader_identity_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_loader_identity_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_loader_identity_load_attempted_false | safety.load_attempt_count:0 |
| protocol:module_loader_identity_selftest_no_mutation | safety.event_log_write_count:0 |
| protocol:module_loader_identity_selftest_no_descriptor | safety.external_artifact_intake_count:0 |
| protocol:module_loader_identity_selftest_no_artifact_bytes | safety.external_artifact_intake_count:0 |
| protocol:module_loader_identity_selftest_no_load | safety.artifact_load_count:0 |
| protocol:module_loader_identity_selftest_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_loader_identity_selftest_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_loader_identity_selftest_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_loader_identity_selftest_load_attempted_false | safety.load_attempt_count:0 |
| protocol:module_loader_artifact_hash_binding_source_evidence_mutation | source_event_id provenance |
| protocol:module_loader_artifact_hash_binding_source_evidence_mutation_scope | source_event_id provenance |
| protocol:module_loader_artifact_hash_binding_no_descriptor | safety.external_artifact_intake_count:0 |
| protocol:module_loader_artifact_hash_binding_no_artifact_bytes | safety.external_artifact_intake_count:0 |
| protocol:module_loader_artifact_hash_binding_no_load | safety.artifact_load_count:0 |
| protocol:module_loader_artifact_hash_binding_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_loader_artifact_hash_binding_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_loader_artifact_hash_binding_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_loader_artifact_hash_binding_load_attempted_false | safety.load_attempt_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_mutation | safety.event_log_write_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_descriptor | safety.external_artifact_intake_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_artifact_bytes | safety.external_artifact_intake_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_load | safety.artifact_load_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_slots | safety.service_slot_allocation_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_no_inventory_records | safety.service_inventory_change:none |
| protocol:module_loader_artifact_hash_binding_selftest_can_load_false | decision.outcome:observed + safety.load_authorization_count:0 |
| protocol:module_loader_artifact_hash_binding_selftest_load_attempted_false | safety.load_attempt_count:0 |

#### Coverage floor and certainty

- Emitter tables contain 127 case occurrences (allocator 29, runtime 72,
  identity 12, artifact binding 14), representing 112 distinct names. The
  harness covers every name through 115 distinct full tuple needles; 12
  byte-identical cross-family occurrences are honest merges. Each tuple covers
  case, expected status/reason, nested actual status/reason, and passed=true.
- UNCERTAIN: all 119 regenerated carriers. The supplied run stopped before
  these four commands; the emitter-derived needles require the focused rerun.
- Live allocator bytes give first failure
  service_slot_allocator_authority_boundary_non_authorizing, not
  service_slot_allocator_runtime_missing.
- DECISION NEEDED: the bad needle is in forbidden
  vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1; its owning
  packet must replace it before rerun.

### P4-3b2 donor-removal note (orchestrator, verified against live transcripts)

Deleting the old loader/allocator selftest emitters exposed one vacuous
predicate in the ALREADY-CONVERTED P4-2 load-gate section:
`protocol:module_load_gate_service_slot_selftest_no_records` asserted
`"creates_service_slot_reservation_records": false`, a key its own v1 emitter
no longer renders — it had been matching the deleted P4-3 allocator selftest's
bytes (Assert-LogContains greps the whole serial log, so any response could
satisfy it). Re-pointed at the honest same-response carrier
`"retained_record_create_count": 0` (a service-slot reservation record IS a
retained record; the typed v1 safety counter is the successor of the legacy
boolean). This is the third confirmed instance of the donor-removal class and
the reason the batch needle-vs-live-log check runs after every family.

### P4-3 close: full-report predicate accounting (orchestrator, mechanical)

Comparing the two green full reports across the P4-3 switch
(shadow-20260713-135447-24872.json -> shadow-20260713-150007-25964.json):
3,786 -> 2,710 runtime predicates (1,325 gone, 249 added). The loss is
concentrated exactly where the design predicted: the eight loader-fact
methods, the allocator, and the loader runtime each used to emit ~59 repeated
non-effect booleans PER RESPONSE (no_records, no_slots,
no_inventory_records, source_evidence_mutation/scope, local_only, ...), which
the harness asserted one-by-one. v1 replaces that whole class with six typed
safety counters plus one decision, so the assertions collapse with them.

Coverage floor VERIFIED mechanically on the green report, not assumed: every
one of the eight loader-fact families retains 11 passing assertions (v1
envelope schema, its own evidence record with status+reason, its selftest
case_count, its three distinct evaluator reasons — binding / missing / ready —
and a safety counter); loader_identity 26, artifact_hash_binding 20,
loader_runtime 536, service_slot_allocator 63. 0 failed predicates in the run.
Assertion NAMES in the loader-fact block lost their `module_loader_` prefix
(they are keyed off the fact Id now) — a rename, not a deletion; noted here so
the next report diff does not read it as a coverage loss.
