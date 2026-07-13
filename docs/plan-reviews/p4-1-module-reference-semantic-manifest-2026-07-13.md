# P4-1a — module reference/evidence semantic manifest

Read-only inventory. No files changed, committed, pushed, or merged.

Notation:

- `R` = legacy `body.result`.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

## 1. Response-path inventory

All 14 methods currently use:

```text
RAIOS_AGENT_BEGIN <method>
{
  v,
  t,
  id,
  body: {
    method,
    result: { ... }
  }
}
RAIOS_AGENT_END <method>
```

The JSON transport-envelope order is `v`, `t`, `id`, `body`; body order is `method`, `result`.

### Diagnostics

The following lists preserve current field order.

#### `module.manifest_diagnostic`

```text
R:
 schema[E], scope[E], classification[E],
 test_infrastructure[F],
 mutates_global_event_log[D], global_event_log_mutation[D],
 accepts_manifest_json[F], accepts_artifact_bytes[F],
 accepts_unsigned_service_code[F], loads_artifact[D],
 service_inventory_change[D], load_attempted[D],
 reference_format[F],
 request[F],
 module_manifest_reference[E],
 retained_manifest_reference[E],
 gate_state[E/D],
 policy_result[E/D],
 blocked_by[D]
```

Nested:

```text
request:
 requested_capability, load_mode, subject, resource,
 manifest_schema, manifest_reference_schema,
 manifest_reference_canonicalization

module_manifest_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 manifest_schema, manifest_reference_hash,
 expected_manifest_reference_hash, manifest_hash

retained_manifest_reference:
 state, retention, event_id, recorded_event_id,
 matches_current_reference, schema, status, classification,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, authorizes_guest_load,
 can_load_now, service_inventory_change, load_attempted,
 hashes {manifest_reference_hash, manifest_hash}
```

Missing retained form replaces fields after `status` with `reason`, `can_load_now`, `load_attempted`.

#### `module.artifact_diagnostic`

Same skeleton, with top-level additions `allocates_service_slot`, `artifact_loaded`, `service_started`.

```text
request:
 requested_capability, load_mode, subject, resource,
 artifact_reference_schema, artifact_reference_canonicalization

candidate_artifact_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 retained_manifest_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {
   artifact_reference_hash, expected_artifact_reference_hash,
   manifest_reference_hash, manifest_hash,
   computed_capability_grant_hash,
   expected_computed_capability_grant_hash,
   artifact_hash, vm_test_report_hash, local_attestation_hash
 }

retained_candidate_artifact_reference:
 common retained fields,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, authorizes_guest_load,
 can_load_now, service_inventory_change, load_attempted,
 retained_manifest_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {
   artifact_reference_hash, manifest_reference_hash, manifest_hash,
   computed_capability_grant_hash, artifact_hash,
   vm_test_report_hash, local_attestation_hash
 }
```

#### `module.vm_report_diagnostic`

```text
R:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, global_event_log_mutation,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_vm_report_json, accepts_unsigned_service_code,
 allocates_service_slot, loads_artifact, artifact_loaded,
 service_started, service_inventory_change, load_attempted,
 reference_format, request, vm_test_report_reference,
 retained_vm_test_report_reference, gate_state,
 policy_result, blocked_by
```

```text
request:
 requested_capability, load_mode, subject, resource,
 vm_test_report_schema, vm_test_report_reference_schema,
 vm_test_report_reference_canonicalization

vm_test_report_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 retained_manifest_reference_event_id,
 retained_candidate_artifact_reference_event_id,
 retained_computed_grant_reference_event_id,
 vm_test_report_schema,
 vm_test_report_reference_hash,
 expected_vm_test_report_reference_hash,
 expected_computed_capability_grant_hash,
 manifest_reference_hash, artifact_reference_hash,
 manifest_hash, artifact_hash, computed_capability_grant_hash,
 vm_test_report_hash, local_attestation_hash

retained_vm_test_report_reference:
 common retained fields,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_vm_report_json, accepts_unsigned_service_code,
 authorizes_guest_load, can_load_now,
 service_inventory_change, load_attempted,
 retained_manifest_reference_event_id,
 retained_candidate_artifact_reference_event_id,
 retained_computed_grant_reference_event_id,
 hashes {
   vm_test_report_reference_hash, manifest_reference_hash,
   artifact_reference_hash, manifest_hash, artifact_hash,
   computed_capability_grant_hash, vm_test_report_hash,
   local_attestation_hash
 }
```

#### `module.attestation_diagnostic`

```text
R:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, promotion_signature_retained,
 global_event_log_mutation,
 accepts_manifest_json, accepts_artifact_bytes,
 accepts_vm_report_json, accepts_local_attestation_json,
 accepts_unsigned_service_code, allocates_service_slot,
 loads_artifact, artifact_loaded, service_started,
 service_inventory_change, load_attempted,
 reference_format, request, local_attestation_reference,
 retained_local_attestation_reference, gate_state,
 policy_result, blocked_by
```

```text
local_attestation_reference:
 state, validation_status, validation_reason,
 signature_verified [conditionally present only when true],
 arity_valid, scope,
 four retained_*_event_id fields,
 hashes {
   local_attestation_reference_hash,
   expected_local_attestation_reference_hash,
   manifest_reference_hash, artifact_reference_hash,
   vm_test_report_reference_hash, manifest_hash, artifact_hash,
   computed_capability_grant_hash,
   expected_computed_capability_grant_hash,
   vm_test_report_hash, local_attestation_hash
 }

retained_local_attestation_reference:
 common retained fields,
 accepts_local_attestation_json, accepts_artifact_bytes,
 accepts_unsigned_service_code, authorizes_guest_load,
 can_load_now, service_inventory_change, load_attempted,
 four retained_*_event_id fields,
 hashes {
   local_attestation_reference_hash, manifest_reference_hash,
   artifact_reference_hash, vm_test_report_reference_hash,
   manifest_hash, artifact_hash, computed_capability_grant_hash,
   vm_test_report_hash, local_attestation_hash
 }
```

#### `module.approval_diagnostic`

Attestation shape plus `accepts_local_approval_text`.

```text
local_approval_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 five retained_*_event_id fields,
 hashes {
   local_approval_reference_hash,
   expected_local_approval_reference_hash,
   manifest_reference_hash, artifact_reference_hash,
   vm_test_report_reference_hash,
   local_attestation_reference_hash,
   manifest_hash, artifact_hash,
   computed_capability_grant_hash,
   expected_computed_capability_grant_hash,
   vm_test_report_hash, local_attestation_hash,
   local_approval_hash
 }

retained_local_approval_reference:
 common retained fields,
 accepts_local_approval_text, accepts_artifact_bytes,
 accepts_unsigned_service_code, authorizes_guest_load,
 can_load_now, service_inventory_change, load_attempted,
 five retained_*_event_id fields,
 hashes {
   local_approval_reference_hash, manifest_reference_hash,
   artifact_reference_hash, vm_test_report_reference_hash,
   local_attestation_reference_hash, manifest_hash, artifact_hash,
   computed_capability_grant_hash, vm_test_report_hash,
   local_attestation_hash, local_approval_hash
 }
```

#### `module.grant_diagnostic`

```text
R:
 schema, scope, classification, test_infrastructure,
 accepts_artifact_bytes, artifact_loaded, service_started,
 service_inventory_change, load_attempted, reference_format,
 request, computed_grant_reference, retained_reference,
 gate_state, policy_result, blocked_by
```

```text
computed_grant_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 computed_capability_grant_hash,
 expected_computed_capability_grant_hash,
 manifest_hash, artifact_hash, vm_test_report_hash,
 local_attestation_hash

retained_reference:
 state, retention, event_id, recorded_event_id,
 matches_current_reference, schema, status,
 grants_capability, grants_load_now, authorizes_guest_load,
 can_load_now, load_attempted,
 hashes {
   computed_capability_grant_hash, manifest_hash, artifact_hash,
   vm_test_report_hash, local_attestation_hash
 }

policy_result:
 computed_candidate_present, grants_capability, trust_tier,
 grants_load_now, authorizes_guest_load, can_load_now,
 dev_tier_can_load_now, service_inventory_change,
 load_attempted, guest_evidence_authority,
 required_before_load
```

#### `module.audit_rollback_diagnostic`

```text
R:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, global_event_log_mutation,
 accepts_artifact_bytes, creates_durable_audit_records,
 creates_rollback_plans, allocates_service_slot,
 loads_artifact, artifact_loaded, service_started,
 service_inventory_change, load_attempted,
 reference_format, request, audit_rollback_reference,
 retained_audit_rollback_reference, gate_state,
 policy_result, blocked_by
```

```text
audit_rollback_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 denial_event_id, retained_reference_event_id,
 ram_only_service_slot_id,
 hashes {
   audit_record_hash, expected_audit_record_hash,
   rollback_plan_hash, expected_rollback_plan_hash,
   computed_capability_grant_hash,
   expected_computed_capability_grant_hash,
   manifest_hash, artifact_hash, vm_test_report_hash,
   local_attestation_hash, local_approval_hash,
   pre_load_service_inventory_hash, cleanup_actions_hash
 }

retained_audit_rollback_reference:
 common retained fields,
 durable_audit_written, rollback_plan_installed,
 grants_capability, grants_load_now, authorizes_guest_load,
 can_load_now, load_attempted, denial_event_id,
 retained_computed_grant_reference_event_id,
 ram_only_service_slot_id,
 hashes {
   audit_record_hash, rollback_plan_hash,
   computed_capability_grant_hash, manifest_hash, artifact_hash,
   vm_test_report_hash, local_attestation_hash,
   local_approval_hash, pre_load_service_inventory_hash,
   cleanup_actions_hash
 }
```

#### `module.service_slot_diagnostic`

```text
R:
 schema, scope, classification, test_infrastructure,
 mutates_global_event_log, global_event_log_mutation,
 accepts_artifact_bytes, allocates_service_slot,
 creates_service_inventory_records, loads_artifact,
 reference_format,
 service_slot_reservation_reference,
 retained_service_slot_reservation,
 policy_result,
 live_granted_service_slot [optional],
 blocked_by
```

```text
service_slot_reservation_reference:
 state, validation_status, validation_reason, arity_valid, scope,
 reservation_hash, expected_reservation_hash,
 retained_computed_grant_reference_event_id,
 retained_audit_rollback_reference_event_id,
 computed_capability_grant_hash, audit_record_hash,
 rollback_plan_hash, pre_load_service_inventory_hash,
 ram_only_service_slot_id

retained_service_slot_reservation:
 common retained fields,
 allocates_service_slot, service_inventory_change,
 can_load_now, load_attempted,
 retained_computed_grant_reference_event_id,
 retained_audit_rollback_reference_event_id,
 ram_only_service_slot_id,
 hashes {
   reservation_hash, computed_capability_grant_hash,
   audit_record_hash, rollback_plan_hash,
   pre_load_service_inventory_hash
 }

live_granted_service_slot:
 state, service_id, ram_only_service_slot_id,
 service_slot_allocated, running, health,
 service_slot_activation_id, service_slot_activation_hash,
 service_slot_activation_status, service_slot_activation_active,
 trust_tier, load_mechanism, maps_executable_pages,
 durable, owner_sealed, authorizes_native_guest_load
```

### Selftests

Methods:

```text
module.manifest_diagnostic_selftest
module.artifact_diagnostic_selftest
module.vm_report_diagnostic_selftest
module.attestation_diagnostic_selftest
module.approval_diagnostic_selftest
module.grant_diagnostic_selftest
module.audit_rollback_diagnostic_selftest
module.service_slot_diagnostic_selftest
```

Common ordered tail:

```text
case_count, passed, cases[], can_load
```

Every case currently carries, in order:

```text
case, expected_status, expected_reason,
actual_status, actual_reason, passed,
can_load/can_load_now, load_attempted
```

Pre-tail fields are the family’s non-mutating guardrails: schema/scope/classification, `test_infrastructure:true`, mutation/record-creation false fields, intake false fields, loader/slot unavailability where applicable.

## 2. Semantic manifest

### Legacy transport envelope

```text
v -> retired_redundancy(v1 schema is the protocol version)
t -> retired_redundancy(all surviving records are evidence responses)
id:"serial" -> v1.id (replace with typed response.current_boot.NNNNNNNN)
body.method -> v1.source_method
body.result -> flattened v1 response root
```

Every v1 response adds:

```text
schema -> constant("raios.evidence_response.v1")
family -> constant(method-table family)
event_id -> kernel-acquired event ID or null
```

Family names:

```text
module.manifest_reference
module.candidate_artifact_reference
module.vm_test_report_reference
module.local_attestation_reference
module.local_approval_reference
module.computed_grant
module.audit_rollback_reference
module.service_slot_reservation
```

Selftests append `.selftest`.

### Common diagnostic mappings

```text
R.schema -> constant("raios.evidence_response.v1")
R.scope -> scope
R.classification -> classification
R.test_infrastructure -> F.test_infrastructure
R.reference_format -> F.reference_format
R.request.* -> F.request.*
```

Reference objects map mechanically:

```text
R.<reference>.state -> E[family reference].facts.state
R.<reference>.validation_status -> E[family reference].facts.status_detail
R.<reference>.validation_reason -> E[family reference].reason
R.<reference>.arity_valid -> E[family reference].facts.arity_valid
R.<reference>.scope -> E[family reference].facts.scope
R.<reference>.*_event_id -> E[family reference].facts.*_event_id
R.<reference>.*_hash -> E[family reference].facts.*_hash
R.<reference>.hashes.* -> E[family reference].facts.* 
```

Retained objects map mechanically:

```text
R.retained_*.state -> E[family retained].facts.state
R.retained_*.retention -> E[family retained].facts.retention
R.retained_*.event_id -> E[family retained].source_event_id
R.retained_*.recorded_event_id -> event_id
R.retained_*.matches_current_reference
    -> E[family retained].facts.matches_current_reference
R.retained_*.schema -> E[family retained].facts.record_schema
R.retained_*.status -> E[family retained].facts.status_detail
R.retained_*.reason -> E[family retained].reason
R.retained_*.classification -> E[family retained].classification
R.retained_*.*_event_id -> E[family retained].facts.*_event_id
R.retained_*.hashes.* -> E[family retained].facts.*
```

Intake/trust-boundary fields remain descriptive facts:

```text
R.accepts_* -> F.intake.accepts_*
R.<retained>.accepts_* -> E[family retained].facts.accepts_*
R.reference/object schema and canonicalization fields
    -> F.request.* or E[family reference].facts.*
R.guest_evidence_authority -> F.guest_evidence_authority
R.required_before_load -> F.required_before_load
R.loader / R.service_slot descriptive states -> F.runtime.*
R.trust_tier -> F.trust_tier
```

Gate state becomes ordered evidence:

```text
R.gate_state.<gate> -> E[id=<gate>].status + facts.status_detail
```

`hash_reference_valid`, `signature_verified`, `missing`, `unavailable`, and similar family values remain in `facts.status_detail`; common `status` becomes `verified`, `missing`, `rejected`, `unavailable`, or `not_applicable`.

Blocked gates:

```text
R.blocked_by[].gate -> D.blocked_by[].evidence_id
R.blocked_by[].state -> D.blocked_by[].status
R.blocked_by[].reason -> D.blocked_by[].reason
```

Evaluator order must remain unchanged.

### Decision/effects mapping

```text
can_load=false / can_load_now=false
    -> D.outcome="denied"
can_load_now=true
    -> D.outcome="granted" only with evaluator-created GrantProof

grants_capability=true
    -> requested capability appears in D.grants
grants_capability=false
    -> D.grants=[]

grants_load_now / authorizes_guest_load / authorizes_native_guest_load
    -> corresponding load effect membership in D.effects

load_attempted=false
    -> D.effects excludes "artifact_load_attempted"
load_attempted=true
    -> D.effects includes "artifact_load_attempted"

loads_artifact / artifact_loaded
    -> D.effects membership "artifact_loaded"
service_started
    -> D.effects membership "service_started"
allocates_service_slot / service_slot_allocated
    -> D.effects membership "service_slot_allocated"
creates_service_inventory_records
    -> D.effects membership "service_inventory_record_created"
creates_durable_audit_records / durable_audit_written
    -> D.effects membership "durable_audit_record_written"
creates_rollback_plans / rollback_plan_installed
    -> D.effects membership "rollback_plan_installed"
service_inventory_change:"none"
    -> D.effects excludes every inventory mutation
mutates_global_event_log=true
    -> event_id (non-null) + E[family retained].source_event_id
mutates_global_event_log=false
    -> event_id = null
```

ORCHESTRATOR RESOLUTION (2026-07-13, risk 1 of section 5, applied): the first
draft mapped reference RETENTION into `decision.effects`, which contradicts
the fail-closed contract (P4 design section 4: a denied decision always
renders `effects: []`) — and a valid reference IS recorded while the load
stays denied. Resolution: retention is PROVENANCE, never an authorized
effect. It is carried by the response `event_id` and the retained evidence
record's `source_event_id`; `decision.effects` lists only effects the
capability decision authorized. This keeps `denied => grants:[] && effects:[]`
absolute, needs no change to the P4-0 substrate, and makes
`global_event_log_mutation` a clean retired redundancy (`event_id != null`
is the same fact). The recording RESULT (not the `check.valid` prediction)
remains the source, per risk 1.

Explicit retirements:

```text
R.global_event_log_mutation
    -> retired_redundancy(D.effects + event_id carry the same result)

R.gate_state.can_load
R.policy_result.can_load_now duplicates
R.retained_*.can_load_now duplicates
    -> retired_redundancy(single D.outcome)

R.policy_result.service_inventory_change
R.retained_*.service_inventory_change
top-level service_inventory_change
    -> retired_redundancy(single D.effects)

all repeated load_attempted/authorizes/grants false fields
    -> retired_redundancy(single D.grants/D.effects)

R.policy_result.*_reference_present
    -> retired_redundancy(reference evidence status)

R.policy_result.local_attestation_present
R.policy_result.local_approval_present
    -> retired_redundancy(corresponding evidence status)

R.policy_result.computed_candidate_present
    -> retired_redundancy(computed-grant evidence status)

R.policy_result.dev_tier_can_load_now
    -> retired_redundancy(D.outcome + F.trust_tier)
```

Important invariant: the canonical inputs in `module_evidence.rs` remain byte-identical. Its embedded lines such as `grants_load_now=false`, `authorizes_guest_load=false`, `service_inventory_change=none`, and `load_attempted=false` are authority-hash grammar, not response-vocabulary fields, and must not be regenerated.

## 3. Predicate inventory

Only predicates asserting these 14 direct responses are counted. Assertions in `full-audit` inspect `memory.recent_events`; those are P4-4 event-output predicates, not P4-1 response predicates. `full-module-audit-rollback` contains no direct scoped response assertion.

| Profile | leaf survives | must regenerate | framing survives | Total |
|---|---:|---:|---:|---:|
| `full-module-evidence` | 101 | 139 | 22 | 262 |
| `full-module-selftests` | 46 | 19 | 3 | 68 |
| `m6c-promotion` | 0 | 2 | 7 | 9 |
| `m6d-rollback` | 0 | 2 | 7 | 9 |
| `persistence-reboot` | 0 | 6 | 6 | 12 |
| **Total** | **147** | **168** | **45** | **360** |

### (a) Leaf needles that survive

By name, these are the scoped profile predicates not listed under (b) or (c), principally:

```text
*_local_only
*_no_*_json / *_no_artifact_bytes / *_no_unsigned_code
*_retained_matches
*_ref_hash_echo / *_hash_echo
*_count / *_passed
*_case
*_actual_status / *_actual_reason
```

Their exact leaf pairs remain present under `facts` or evidence `facts`; case name, expected/actual status/reason, hashes, classification, and `passed` values do not change.

### (b) Must regenerate

The exact affected needles are every scoped predicate using one of:

```text
"schema": "<legacy raios.*.v0>"
"validation_status": ...
"validation_reason": ...
"mutates_global_event_log": ...
"global_event_log_mutation": ...
"status": "retained_hash_reference_load_still_denied"
"event_id": "event.current_boot....
"recorded_event_id": "event.current_boot....
"<gate-name>": "missing|hash_reference_*|unavailable|unallocated"
"*_reference_present": true
"computed_candidate_present": true
"grants_capability": ...
"grants_load_now": ...
"authorizes_guest_load": ...
"can_load": ...
"can_load_now": ...
"load_attempted": ...
"service_inventory_change": ...
"allocates_service_slot": ...
"creates_service_inventory_records": ...
"creates_retained_*_records": ...
"creates_durable_audit_records": ...
"creates_rollback_plans": ...
"loads_artifact": ...
```

Named path assertions requiring regeneration:

```text
protocol:module_manifest_retained_reference_event_id_captured
  $moduleManifestResponse.body.result.retained_manifest_reference.event_id

protocol:module_grant_retained_reference_event_id_captured
  $moduleGrantResponse.body.result.retained_reference.event_id

protocol:module_artifact_retained_reference_event_id_captured
  $moduleArtifactResponse.body.result.retained_candidate_artifact_reference.event_id

protocol:module_vm_report_retained_reference_event_id_captured
  $moduleVmReportResponse.body.result.retained_vm_test_report_reference.event_id

protocol:module_attestation_retained_reference_event_id_captured
  $moduleAttestationResponse.body.result.retained_local_attestation_reference.event_id

protocol:module_approval_retained_reference_event_id_captured
  $moduleApprovalResponse.body.result.retained_local_approval_reference.event_id

protocol:module_service_slot_retained_audit_reference_event_id_captured
  $moduleAuditRollbackResponse.body.result.retained_audit_rollback_reference.event_id
```

Other named shape predicates:

```text
m6c:grant_reports_dev_tier_can_load_now
m6c:live_service_slot_diagnostic_reports_allocated_granted_slot
m6d:grant_reports_dev_tier_can_load_now
m6d:live_service_slot_diagnostic_reports_allocated_granted_slot

boot1:manifest-reference-event-id
boot1:grant-reference-event-id
boot1:artifact-reference-event-id
boot1:vm-report-reference-event-id
boot1:signature-verified-true
boot1:grant-reports-dev-tier-can-load-now
```

Their exact PowerShell paths currently start at `body.result...`; they must target the v1 root, `facts`, `evidence`, or `decision`.

### (c) Framing survives

All 45 `Send-AgentCommand` completion predicates waiting for:

```text
RAIOS_AGENT_END module.<family>_diagnostic
RAIOS_AGENT_END module.<family>_diagnostic_selftest
```

survive unchanged. This includes dynamic command-name predicates whose runtime name contains the fully expanded hash-bearing command.

## 4. Fixture note

Embedded tables/functions:

| Response | Table/function | Cases |
|---|---|---:|
| manifest selftest | `module_manifest_selftest_cases()` | 5 |
| artifact selftest | `module_artifact_selftest_cases()` | 7 |
| VM-report selftest | `module_vm_report_selftest_cases()` | 8 |
| attestation selftest | `LOCAL_ATTESTATION_CASES` | 10 |
| approval selftest | `LOCAL_APPROVAL_CASES` | 10 |
| grant selftest | `GRANT_CASES` | 10 |
| audit/rollback selftest | `AUDIT_ROLLBACK_CASES` | 10 |
| service-slot selftest | `SERVICE_SLOT_CASES` | 5 |

The v1 selftest response should use:

```text
family: "<family>.selftest"
facts: {
  test_infrastructure,
  case_count,
  passed,
  cases: [
    {
      case,
      expected: {status, reason},
      actual: {status, reason},
      passed
    }
  ]
}
evidence: []
decision: {outcome:"observed", reason:"selftest_completed"}
```

`can_load` and `load_attempted` disappear from each case and the response root because they are constant non-effects; the same case name, expected/actual status and reason, ordering, count, and `passed` value remain.

## 5. Risks requiring resolution before phase (b)

1. `mutates_global_event_log` is derived from `check.valid`, while actual provenance comes from `recorded_event_id`; use the recording result, not the prediction.

2. Retained snapshots are fetched separately after recording. `recorded_event_id`, retained `event_id`, and `matches_current_reference` need one captured projection to prevent mixed snapshots.

3. Attestation `signature_verified` is conditionally omitted when false. V1 must resolve this to explicit `false` or `null`; omission cannot continue.

4. `promotion_signature_retained` comes from a second event-log write, separate from the attestation evaluator and retained-reference snapshot.

5. Grant authority combines four sources: parsed grant check, latest retained attestation, live candidate-byte readiness, and Wasm/runtime readiness. It is not currently one evaluator projection.

6. `module_grant_selftest_case_can_load_now()` derives authority from the case-name string rather than the evaluator result. This must be replaced before claiming same-source equivalence.

7. Service-slot live projection combines `loaded_snapshot()`, `live_load_projection()`, descriptor constants, activation accessors, and health accessors. Capture one coherent snapshot/projection first.

8. `gate_state`, `policy_result`, and most `blocked_by` lists are emitter-reconstructed rather than returned by one evaluator accessor. Their order and first-failure semantics need typed evaluator ownership.

9. Audit/rollback expected hashes combine `module_evidence.rs` canonical hash functions with parsed fields and event IDs. Response migration must not alter those canonical grammars.

10. Existing M6 harness code reads attestation `validation_status` at `body.result.validation_status`, although the emitter places it under `local_attestation_reference.validation_status`. That live-positive check is not reliable semantic evidence and must be corrected during golden regeneration.