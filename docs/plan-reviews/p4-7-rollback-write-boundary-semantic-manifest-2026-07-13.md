# P4-7a — Rollback/write-boundary semantic manifest

Read-only inventory for P4-7b. This packet changes no Rust, harness, existing
documentation, descriptor, signature, build, status, roadmap, release artifact,
or authority state.

Notation:

- `R` = legacy response `body.result` (or legacy error `body`).
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy field intentionally removed.

The P4-7 boundary is the complete rollback/write family: module write-boundary
availability, policy, layout, append engine/contract/payload/intent and final
boundary; Hello rollback preview/apply, recovery materialization/inspection,
transaction append/readback, retained inspection, and the mixed rollback tails
handed off by P4-6a. P4-7 remains isolated. Only W6 pure evaluation/projection
may move; storage I/O and rollback state application remain kernel-owned.

## 1. Response-path inventory

### Files and ownership boundary

All ten `agent_protocol_module_write_boundary*.rs` files are in scope:

```text
agent_protocol_module_write_boundary.rs
agent_protocol_module_write_boundary_availability.rs
agent_protocol_module_write_boundary_write_policy.rs
agent_protocol_module_write_boundary_storage_layout.rs
agent_protocol_module_write_boundary_append_engine.rs
agent_protocol_module_write_boundary_append_contract.rs
agent_protocol_module_write_boundary_append_payload_hash.rs
agent_protocol_module_write_boundary_append_intent.rs
agent_protocol_module_write_boundary_boundary.rs
agent_protocol_module_write_boundary_emit.rs
```

The Hello-owned surfaces are:

| File | P4-7 ownership |
|---|---|
| `hello_service/emitters.rs` | rollback nested records `:233-3280`; recovery materialize/inspect `:3295-3470`; rollback apply denied/applied and scoped markers/proof assembly `:3472-4382`; preview `:4384-4427` |
| `hello_service/rollback_authority_gates.rs` | all policy/storage/append/readback/inspection/retained-reference projections and canonical hashes |
| `hello_service/rollback_writer_gate.rs` | preview/apply/preflight/payload/record/sector-plan/write-readback/append-foundation projections and hashes |
| `hello_service/rollback_writer_bindings.rs` | all rollback binding population, including positive applied bindings |
| `hello_service/rollback_bindings.rs`, `rollback_hashes_a.rs`, `rollback_hashes_b.rs` | compatibility re-exports only; remove only after every caller is repointed |
| `hello_service/storage_authority_gate.rs`, `storage_gate_hash.rs` | storage-authority binding/hash and compatibility re-export |
| `hello_service/lifecycle_binding.rs` | rollback-default tail and every rollback-populated field; P4-6 owns only its lifecycle prefix |
| `hello_service/state_machine.rs` | rollback preview/apply transitions `:402-606`, including real applied-state mutation `:480-590` |
| `hello_service/records.rs` | rollback/write DTOs, `ScopedRollbackApplyProof`, and `AppliedRollbackRecord` |
| `hello_service/runtime.rs` | rollback dispatch/acquisition `:238-310` |
| `hello_service/constants.rs`, `hash_support.rs` | rollback constants and canonical line/sector-image grammar only |

`raios-core/src/scoped_rollback_apply.rs` is the authority truth for scoped apply,
authorized-append verification, and verified current-boot apply. Existing v1
envelope/decision construction is in `record_table.rs:137-270` and
`evidence_response.rs:14-92`.

### Transport, dispatch, and aliases

All ordinary paths retain `RAIOS_AGENT_BEGIN <method>` and
`RAIOS_AGENT_END <method>`. Legacy success order is `v,t,id,body`, then
`body.method,result`; denied apply uses the legacy error body. The literal marker
is in shared support, not any inventoried emitter, so the required marker count
is zero (check output in section 4).

Module dispatch is the ordered table at `agent_protocol.rs:491-506`:

| Canonical method | Legacy alias | Emitter |
|---|---|---|
| `module.audit_rollback_availability` | `module.audit_rollback_store_availability` | `emit_module_audit_rollback_availability` |
| `module.audit_rollback_write_policy` | `module.audit_rollback_policy` | `emit_module_audit_rollback_write_policy` |
| `module.audit_rollback_storage_layout` | `module.audit_rollback_persistence_layout` | `emit_module_audit_rollback_storage_layout` |
| `module.audit_rollback_append_engine` | `module.audit_rollback_append_engine_readiness` | `emit_module_audit_rollback_append_engine` |
| `module.audit_rollback_append_contract` | `module.audit_rollback_storage_contract` | `emit_module_audit_rollback_append_contract` |
| `module.audit_rollback_append_payload_hash` | `module.audit_rollback_append_payload` | `emit_module_audit_rollback_append_payload_hash` |
| `module.audit_rollback_append_intent` | `module.audit_rollback_append_request` | `emit_module_audit_rollback_append_intent` |
| `module.audit_rollback_write_boundary` | `module.audit_rollback_write_gate` | `emit_module_audit_rollback_write_boundary` |

Each has the corresponding `_selftest` method and alias. The direct diagnostics
and their command-envelope dispatches are observational/denial-only: none writes,
loads, allocates, appends, or applies.

Hello dispatch is:

| Method | Acquisition and response path | Authority character |
|---|---|---|
| `service.rollback_preview` | `runtime.rs:238-241` -> `state_machine::rollback_preview` -> `emit_rollback_preview_response` | read-only observation; no grant |
| `recovery.rollback_materialize_dry_run` | `runtime.rs:278-290` -> `recovery_rollback_materialization_evidence` -> target write/readback | **real test-media LBA1 write/readback**, not denial-only |
| `recovery.rollback_inspect` | `runtime.rs:267-275` -> live/applied evidence selection -> read/parse/retain source reference | read-only observation; event retention is provenance |
| `service.rollback_apply` denied | `runtime.rs:244-248` -> `state_machine.rs:435-478` -> error emitter plus scoped markers | denial; empty grants/effects |
| `service.rollback_apply` applied | `state_machine.rs:435-590` -> scoped proof -> current-boot state mutation -> applied emitter/markers | positive scoped apply; only core evaluator decisions may grant |
| `recovery.rollback_inspect_source_reference_selftest` | `runtime.rs:293-EOF` | test observation only |

### Exact legacy response order

Every module diagnostic begins with:

```text
schema, scope, classification, test_infrastructure,
mutates_global_event_log, global_event_log_mutation, writes_enabled,
creates_durable_audit_records, creates_rollback_plans,
installs_rollback_plan, service_inventory_change, load_attempted
```

Then, in exact emitter order:

| Method | Remaining top-level order |
|---|---|
| availability | `availability_facts`, `policy_result`, `blocked_by` (`availability.rs:94-164`) |
| write policy | `write_policy_facts`, `policy_result`, `blocked_by` (`write_policy.rs:121-187`) |
| storage layout | `storage_layout_facts`, `storage_authority_foundation`, `policy_result`, `blocked_by` (`storage_layout.rs:290-392`) |
| append engine | `storage_layout_inputs`, `append_engine_facts`, `policy_result`, `blocked_by` (`append_engine.rs:150-239`) |
| append contract | `storage_layout_inputs`, `append_engine_inputs`, `append_contract_facts`, `append_target_owner`, `transaction_writer_readiness`, `policy_result`, `blocked_by` (`append_contract.rs:232-362`) |
| append payload hash | `retained_inputs`, `append_contract_inputs`, `append_payload_hash_facts`, `policy_result`, `blocked_by` (`append_payload_hash.rs:174-266`) |
| append intent | `append_contract_inputs`, `append_payload_hash_inputs`, `append_intent_facts`, `policy_result`, `blocked_by` (`append_intent.rs:181-310`) |
| write boundary | `pre_load_write_request`, retained `inputs`, availability/policy/contract/payload/intent inputs, `denial_evidence`, `policy_result`, `blocked_by` (`boundary.rs:195-385`) |

The final boundary's `pre_load_write_request` field order is `schema,
canonicalization, requested_capability, load_mode, subject, resource,
requested_writes, required_retained_references, recovery_artifact_loading`
(`boundary.rs:249-293`). Its blockers are emitted in evaluator order: preconditions
when rejected, durable audit, rollback install, audit/rollback envelopes,
audit/rollback payload hashes, audit/rollback intents, module loader
(`boundary.rs:319-382`).

All eight module selftests use `schema,scope,classification,test_infrastructure,
mutates_global_event_log,creates_durable_audit_records,creates_rollback_plans,
installs_rollback_plan,service_inventory_change,load_attempted,case_count,passed,
cases,can_load`; boundary additionally carries the two load booleans. A case is
`case,expected_status,expected_reason,actual_status,actual_reason,passed`, followed
by its false safety fields.

Hello top-level order is:

```text
rollback preview:
 schema,id,scope,classification,persistence,read_only,status,preview_available,
 event_id,audit_event_id,service_id,current_generation,current_state,
 source_probation,preview_hash,rollback_target,current_candidate,state_migration,
 denied_surfaces                                      (emitters.rs:4384-4427)

materialize dry-run:
 schema,id,scope,classification,persistence,read_only,test_infrastructure,
 event_id,audit_event_id,status,reason,service_id,requested_capability,
 active_generation,source_probation,append_record_dry_run,sector_plan_dry_run,
 target_region_write_readback,materialized_sector_evidence_available,
 denied_surfaces                                      (emitters.rs:3295-3366)

recovery inspect:
 schema,id,scope,classification,persistence,read_only,event_id,status,reason,
 service_id,requested_capability,active_generation,source_probation,
 materialized_sector_evidence_available,inspection_available,
 target_region_write_readback,target_region_sector_inspection,
 retained_recovery_rollback_inspect_source,denied_surfaces,
 [applied_authority_record]                            (emitters.rs:3368-3470)

denied apply error body:
 method,event_id,audit_event_id,code,schema,id,scope,classification,persistence,
 status,reason,message,service_id,active_generation,active_descriptor_id,
 current_state,source_probation,required_preview,rollback_apply_hash,
 source_durable_policy_write_authority_decision_hash,
 source_recovery_rollback_inspect_source_reference_hash,
 retained_durable_policy_write_authority_decision_verified,
 retained_recovery_rollback_inspect_source_reference_validated,
 rollback_transaction_preflight,rollback_write_authority_gate,
 rollback_append_intent_gate,rollback_payload_envelope_gate,
 rollback_transaction_writer_storage_authority_gate,rollback_target,
 current_candidate,state_migration,required,denied_surfaces
                                                        (emitters.rs:3530-3628)

applied apply:
 schema,id,scope,classification,persistence,status,reason,event_id,audit_event_id,
 service_id,rollback_apply_hash,authority_record,state_transition,previous_state,
 applied_state,source_probation,rollback_target,current_candidate,state_migration,
 side_effects,service                                (emitters.rs:3630-3710)
```

Nested record order is source-authoritative at `emitters.rs:233-523`
(preflight/write/intent/payload gates), `:524-2883` (discovery plus storage
foundation), `:2972-3014` (write/readback), `:3017-3078` (inspection),
`:3087-3127` (retained inspection source), `:3129-3178` (append record/sector
plan), and `:3181-3277` (applied authority and rollback target/candidate).
The mapping rules below cover every child under those prefixes; no child is
implicitly dropped.

Two unframed marker records follow apply responses:

```text
RAIOS_SCOPED_ROLLBACK_APPLY_DECISION <single-line JSON>
RAIOS_SCOPED_ROLLBACK_AUTHORIZED_APPEND <single-line JSON>
```

Their exact field order is `emitters.rs:3804-3903` and `:3947-4055`.

## 2. Semantic mapping

### Envelope and common invariants

```text
v -> retired(v1 schema is protocol version)
t -> retired(D.outcome)
id:"serial" -> v1.id (typed response.current_boot.NNNNNNNN)
body.method -> source_method
body.result / error body -> flattened v1 envelope
legacy schema -> constant("raios.evidence_response.v1")
legacy scope -> envelope.scope (same source)
legacy classification -> envelope.classification (same source)
event_id -> envelope.event_id
audit_event_id -> retired(duplicate envelope event_id)
source event/audit IDs -> E[inspection].source_event_id / facts provenance
persistence -> F.persistence (descriptive durability fact, never authority)
test_infrastructure -> F.test_infrastructure
```

Proposed families are `rollback.write_boundary`, `rollback.preview`,
`rollback.materialize`, `rollback.inspect`, `rollback.apply`, and
`rollback.write_boundary.selftest`.

### Module diagnostic mappings

Prefix rules are exhaustive: `R.X.* -> ...*` maps every present/null child in
legacy order to the named v1 carrier in the same child order.

```text
R.availability_facts.* -> E[policy].facts.availability.*
R.write_policy_facts.* -> E[policy].facts.write_policy.*
R.storage_layout_facts.* -> E[storage].facts.layout.*
R.storage_authority_foundation.* -> E[storage].facts.foundation.*
R.storage_layout_inputs.* -> E[storage].facts.layout_inputs.*
R.append_engine_facts.* -> E[append].facts.engine.*
R.append_engine_inputs.* -> E[append].facts.engine_inputs.*
R.append_contract_facts.* -> E[append].facts.contract.*
R.append_target_owner.* -> E[append].facts.target_owner.*
R.transaction_writer_readiness.* -> E[append].facts.writer_readiness.*
R.retained_inputs.* -> E[inspection].facts.retained_inputs.*
R.append_contract_inputs.* -> E[append].facts.contract_inputs.*
R.append_payload_hash_facts.* -> E[append].facts.payload_hashes.*
R.append_payload_hash_inputs.* -> E[append].facts.payload_hash_inputs.*
R.append_intent_facts.* -> E[append].facts.intents.*
R.append_intent_inputs.* -> E[append].facts.intent_inputs.*
R.pre_load_write_request descriptive fields -> F.request.*
R.inputs.* -> E[inspection].facts.retained_references.*
R.denial_evidence descriptive requirements -> E[policy]/E[append].facts.*
R.blocked_by[] -> D.blocked_by[] (same evaluator order, renamed evidence_id)
```

Status/reason pairs move to their owning evidence record:

```text
availability/policy status+reason -> E[policy].status/reason + facts.status_detail
layout/device status+reason -> E[storage].status/reason + facts.status_detail
engine/contract/payload/intent status+reason -> E[append].status/reason + facts.status_detail
inspection/reference status+reason -> E[inspection].status/reason + facts.status_detail
```

The following legacy authority-shaped fields may not remain free facts:

```text
writes_enabled, creates_durable_audit_records, creates_rollback_plans,
installs_rollback_plan, allocates_service_slot, loads_artifact,
loads_recovery_artifact, can_load[_now], load_attempted,
*_are_*_authority, authorizes_*, writes_*, appends_*, applies_rollback,
mutates_service_state, service_inventory_change
    -> D.outcome/grants/effects only when backed by evaluator GrantProof;
       otherwise retired(empty grants/effects expresses the invariant)

global_event_log_mutation="none" -> retired(event_id=null proves no event)
retained_hash_refs_are_*_authority=false -> retired(no grant/effect)
*_missing complement booleans -> retired(status/reason is authoritative)
```

Every module write-boundary diagnostic remains `D.outcome="denied"`, with
empty grants/effects and requested capability `cap.module.load_ephemeral`.
These emitters are denial-only; no positive module writer exists here.

### Hello rollback mappings

```text
R.current_state / previous_state / applied_state
    -> E[scoped_apply].facts.state.{current,previous,applied}.*
R.source_probation.* -> E[scoped_apply].facts.probation.*
R.state_migration.* -> E[scoped_apply].facts.state_migration.*
R.rollback_target.* -> E[scoped_apply].facts.rollback_target.*
R.current_candidate.* -> E[scoped_apply].facts.current_candidate.*

R.rollback_transaction_preflight.* -> E[policy].facts.transaction_preflight.*
R.rollback_write_authority_gate.* -> E[policy].facts.write_authority_gate.*
R.rollback_append_intent_gate.* -> E[append].facts.intent_gate.*
R.rollback_payload_envelope_gate.* -> E[append].facts.payload_envelope_gate.*
R.rollback_transaction_writer_storage_authority_gate.*
    -> ordered E[policy], E[storage], E[append], E[inspection], E[scoped_apply]
       child facts, preserving its legacy child order

R.append_record_dry_run.* -> E[append].facts.record.*
R.sector_plan_dry_run.* -> E[append].facts.sector_plan.*
R.target_region_write_readback.* -> E[storage].facts.write_readback.*
R.target_region_sector_inspection.* -> E[inspection].facts.sector.*
R.retained_recovery_rollback_inspect_source.*
    -> E[inspection].facts.retained_source.*
R.applied_authority_record.* -> E[scoped_apply].facts.applied_authority.*
```

Preview is observed: `preview_available`, target/candidate and preview hash are
facts/evidence; `read_only` is `constant(true)`; all false denied surfaces are
retired. Materialize is not denial-only: its real write/readback facts stay in
`E[storage]`; a positive v1 decision requires a dedicated evaluator-created
grant proof for `cap.recovery.rollback_materialize_dry_run.current_boot`.
Inspection is observed and cannot grant apply.

Denied apply maps as:

```text
body.code="capability_denied" -> D.outcome="denied"
body.reason -> D.reason (first failure)
body.required[] -> ordered evidence requirements / D.blocked_by
body.message -> F.message (diagnostic only)
all denied_surfaces -> retired(D.grants=[], D.effects=[])
```

Applied apply maps as:

```text
R.status/reason -> D.outcome="granted", D.reason
R.side_effects.authorizes_media_write/append/transaction_append
    -> D.grants (only from scoped GrantProof)
R.side_effects.writes_durable_audit_log/writes_rollback_store/
  appends_rollback_transaction/applies_rollback/mutates_service_state
    -> D.effects (only from scoped GrantProof)
R.side_effects.installs_rollback_state=false and other denied strings
    -> retired(absent grants/effects)
R.state_transition.* -> E[scoped_apply].facts.state_transition.*
R.service.* -> F.service.* (P4-6-compatible projection)
```

### Authority source and SECURITY FINDING

The core positive branches are, in order:

1. scoped scope decision `scoped_rollback_apply.rs:261-560`;
2. authorized-append verification `:563-750`;
3. verified current-boot apply `:753-796`.

No module write-boundary diagnostic grants. Preview and inspect do not grant.
Materialize performs real I/O but currently has no v1 `GrantProof` producer.
Only a core positive branch may create v1 grant authority.

**SECURITY FINDING / P4-7b STOP:** `emitters.rs:3726-3757` currently constructs
`ScopedRollbackApplyProof`, and `:3780-3793` plus `:3906-3931` construct and hash
the decision/authorized-append records in the emitter module. The booleans come
from core evaluators, but the proof wrapper and the `scope_decision_hash` later
accepted by the append evaluator are emitter-created. This violates the P4 v1
rule that an emitter cannot construct authority. P4-7b must add a scoped
rollback evaluator-owned positive constructor returning an unforgeable
`GrantProof`/typed granted decision; the emitter may only render it.

The actual write precedes that apply proof: materialization calls
`hello_rollback_target_region_write_readback_dry_run` at
`rollback_authority_gates.rs:3651-3673`, which calls the kernel AHCI write at
`:3660`; authorized-append verification later proves the retained write and
inspection. Do not rewrite history as if the apply evaluator authorized the
earlier I/O.

### Hash classes that must not be regenerated

Response-vocabulary movement must not change any of these canonical inputs:

| Hash class | Canonical input grammar |
|---|---|
| probation/state-migration inputs | `state_records.rs:56-92`, `:143-247` (P4-6 handoff) |
| rollback preview | `rollback_writer_gate.rs:3-93` ordered canonical `key=value\n` lines |
| apply denial and retained denial sources | `rollback_writer_gate.rs:94-282` |
| transaction preflight, write-authority gate, append-intent gate | `rollback_writer_gate.rs:420-755` |
| rollback transaction payload and provenance | `rollback_writer_gate.rs:756-958` |
| persisted audit-record image | `rollback_writer_gate.rs:959-1000` |
| persisted rollback-transaction image | `rollback_writer_gate.rs:1001-1042` |
| persisted 512-byte sector image/layout | `rollback_writer_gate.rs:1043-1318`; byte writer grammar `hash_support.rs:43-108` |
| scratch write/readback and media/writer preflight | `rollback_writer_gate.rs:1319-1768` |
| durable append authorization/readiness/preflight | `rollback_writer_gate.rs:1769-2444` |
| policy, ledger, acceptance, availability, append and inspection authority chain | `rollback_authority_gates.rs:3-4353` in function order |
| rollback writer storage foundation | `rollback_authority_gates.rs:4354-4609` |
| storage-authority aggregate chain | `storage_authority_gate.rs:21-872` |
| recovery inspect retained reference | `rollback_authority_gates.rs:4210-4353` |
| scoped apply decision hash | exact legacy `Value` grammar `emitters.rs:3804-3903`, hashed at `:3788-3792` |
| scoped authorized-append hash | exact legacy `Value` grammar `emitters.rs:3947-4055`, hashed at `:3739-3746` |
| applied authority chain | transaction/write-readback/inspection/audit/authorized-append/scope hashes copied at `emitters.rs:3650-3659`, retained at `state_machine.rs:546-590` |

Authority hashes, rollback-log hashes, persisted-record hashes, sector-image
hashes, and transaction-chain hashes are not response hashes. None may be
regenerated because paths moved. The two scoped hashes are currently coupled to
legacy response-shaped `Value`s: P4-7b must preserve an explicit hash-only
canonical projection byte-for-byte while changing the serial vocabulary.
Changing those digests is a STOP, not golden regeneration.

## 3. Evidence-unit design and ownership

The ordered evidence sequence is fixed:

| Order | ID | Kind/status | Content |
|---:|---|---|---|
| 1 | `policy` | `policy`; present/missing/rejected/verified | availability, write policy, preflight, requested capability, exact target and first failed policy input |
| 2 | `storage` | `storage`; unavailable/present/verified | controller/device/layout/marker/bounds, LBA span, write/readback facts; never a grant by itself |
| 3 | `append` | `append`; missing/rejected/verified | engine, contract, payload/provenance, record images, sector plan, offsets/padding, transaction chain |
| 4 | `inspection` | `inspection`; unavailable/rejected/verified | readback/parse hashes, offsets, zero padding, retained source event/reference |
| 5 | `scoped_apply` | `scoped_apply`; rejected/verified/not_applicable | probation/state/target scope, exact evaluator result and positive proof locator |

`D.blocked_by` preserves evaluator order. Scoped apply first checks method,
service, target IDs/marker/schemas and exact LBA span (`scoped_rollback_apply.rs:
282-363`), probation/preview/state (`:365-401`), evidence readiness (`:403-426`),
source-chain hashes (`:428-487`), sector/record hashes (`:489-530`), then retained
inspection (`:532-554`). Authorized append checks scope/hash presence/span,
source bindings, sector/record hashes, offsets/padding, actual write/readback, and
inspection in that order (`:584-744`). Verified apply re-evaluates append before
accepting the transaction hash (`:766-795`). No renderer may reconstruct or sort
this order.

### W6 relocation candidates

Only these pure pieces may move to `raios-core` during P4-7:

- typed DTOs for policy/storage/append/inspection/scoped-apply projections;
- pure first-failure evaluators and exact hash equality/span/offset checks;
- field tables and ordered evidence projection;
- canonical hash functions, only with byte-identical fixtures;
- the evaluator-owned scoped `GrantProof` constructor and granted-decision
  projection;
- the existing selftest truth tables and semantic-mapping test.

Kernel-owned functions that **must stay**:

- `ahci::write_readback_scratch_sector_image` (`ahci.rs:1679-1698`);
- `ahci::write_readback_audit_rollback_target_sector_image` (`:1700-1725`);
- `ahci::write_readback_audit_rollback_target_sector_image_for_authorized_append`
  (`:1727-1743`);
- `ahci::cached_audit_rollback_target_sector_write_readback` (`:1745-1755`);
- `ahci::inspect_audit_rollback_target_sector_image` (`:1757-1796`) and the
  authorized-append variant (`:1798-1825`);
- the uncached MMIO/device traversal and write/readback implementation
  (`ahci.rs:2089-2182`, actual target write/readback `:3348-3503`, inspection
  `:3506-EOF`);
- `hello_rollback_target_region_write_readback_dry_run` and
  `hello_rollback_target_region_authorized_append_write_readback`
  (`rollback_authority_gates.rs:3651-3697`) as acquisition adapters;
- `STATE` locking, `state_machine::rollback_apply`, `rollback_apply_verified`,
  and `applied_rollback_record` (`state_machine.rs:435-590`);
- command parsing/dispatch, PCI acquisition, cache mutation, event acquisition/
  retention, serial framing and backpressure.

`lifecycle_binding.rs` remains mixed ownership. Its rollback defaults and
populated rollback fields may be replaced only together with the P4-7 projection;
its P4-6 lifecycle prefix must remain semantically unchanged. Probation hashes
are P4-6-produced P4-7 inputs, never regenerated here.

## 4. Predicate inventory

### Required sweeps and exact output

PowerShell does not expand native-command globs, so the marker scan uses `--glob`:

```text
> rg -c "RAIOS_AGENT_BEGIN" seed-kernel/src --glob "agent_protocol_module_write_boundary*.rs" --glob "hello_service/rollback*.rs" --glob "hello_service/storage_authority_gate.rs" --glob "hello_service/storage_gate_hash.rs" --glob "hello_service/emitters.rs"
[no stdout; exit 1: zero literal markers in every inventoried file]
```

```text
> rg -l "rollback|write_boundary|write_policy|append_engine" vm-harness/
vm-harness/create-local-attestation.ps1
vm-harness/shadow-vm-persistence-reboot.ps1
vm-harness/shadow-vm-smoke-profile-full-module-load-gate.ps1
vm-harness/shadow-vm-smoke-profile-full-audit.ps1
vm-harness/shadow-vm-smoke-profile-full-module-audit-rollback.ps1
vm-harness/shadow-vm-smoke-profile-hello-rollback-dry-run.ps1
vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1
vm-harness/shadow-vm-smoke-profile-full-module-evidence.ps1
vm-harness/shadow-vm-smoke-profile-genesis-ui.ps1
vm-harness/shadow-vm-smoke-profile-m12-distribution-provenance.ps1
vm-harness/shadow-vm-smoke-profile-m6c-promotion.ps1
vm-harness/shadow-vm-smoke-profile-m8-lifeline.ps1
vm-harness/shadow-vm-smoke-profile-memory-durable.ps1
vm-harness/shadow-vm-smoke-profile-m6d-rollback.ps1
vm-harness/shadow-vm-smoke-profile-persistence.ps1
vm-harness/shadow-vm-smoke-profile-project-install.ps1
vm-harness/shadow-vm-smoke-profile-quick.ps1
vm-harness/shadow-vm-smoke-support.ps1
vm-harness/validate-module-manifest.ps1
vm-harness/shadow-vm-smoke.ps1
```

Only five files own family assertions. The others are profile composition,
unrelated module-reference/load/provider/recovery-lifeline semantics, fixture
text, validation, or shared plumbing; their donor exposure is reviewed below.

### Needle-count scan

Counting rules: every `Assert-LogContainsFields` field is expanded; a compound
`if (...) { throw }` is one assertion; an `Add-Predicate` followed by the throw
is counted once; M6D's unpaired `Add-Predicate` is retained; selected ranges
exclude P4-6 health/lifecycle and P4-9 inventory. The selftest range expands all
eight responses (156 evaluator cases); 135 exact case-name/`passed` leaves can
survive, while old schemas, safety booleans, and flat expected/actual paths must
move.

Command used:

```powershell
# full-module-audit-rollback: whole file; fields + if checks + sends
# full-module-selftests: lines 487-755
# quick: 1125-1231, 1642-1687, 2342-2356, 2367-2441
# hello rollback dry-run: 29-37 and 75-298
# m6d rollback: 129-149, 537-570, 580-600
```

Verbatim output:

```text
> needle-count scan (owned ranges; Assert-LogContainsFields arrays and runtime checks expanded)
full-module-audit-rollback leaf_survives=0 must_regenerate=503 framing_survives=24 total=527
full-module-selftests leaf_survives=135 must_regenerate=119 framing_survives=8 total=262
quick leaf_survives=0 must_regenerate=57 framing_survives=8 total=65
hello-rollback-dry-run leaf_survives=0 must_regenerate=54 framing_survives=13 total=67
m6d-rollback leaf_survives=0 must_regenerate=5 framing_survives=3 total=8
TOTAL leaf_survives=135 must_regenerate=738 framing_survives=56 total=929
```

This exceeds the design estimate of 300-500 reviewed / 200-360 regenerated.
The estimate omitted 471 expanded diagnostic field needles, eight explicit
selftest families, positive LBA1 write/readback/inspection chains, pre- and
post-apply inspections, unframed scoped markers, and M6D's separate generic
rollback path.

### Serialization and collapse rules

Record-model object lines arrive as CR CR LF over the observed serial path.
Every multiline PowerShell needle must use explicit `` `r`r`n ``. `facts`, each
evidence item, and `decision` are single-line `InlineObject`s. No needle may span
the response `id`; anchor before it or from `family` onward.

Required distinguishing replacements:

1. Bare `"status":"missing"`/`"reason":...` collapses policy, storage,
   append, and inspection. Anchor evidence `id`, status, reason, and one unique
   target/hash fact.
2. `writes_enabled:false`, `can_load:false`, and `load_attempted:false` are
   donor-prone retired redundancy. Replace with the full denial decision,
   requested capability, empty grants/effects, and first blocker.
3. Storage layout and append contract share layout/engine reasons. Anchor the
   family plus `E[storage]` versus `E[append]` ID.
4. Audit and rollback payload/intent halves share status. Anchor target schema/
   ID and payload/intent hash.
5. Preview, materialize, pre-apply inspect, post-materialize inspect, and
   post-apply inspect share service/probation hashes. Anchor family, operation
   status, and evidence-unit IDs.
6. A write/readback `verified` status must include LBA1 span, planned/readback
   hash equality, and `write_attempted/write_completed/readback_completed`; a
   bare status cannot prove real I/O.
7. Inspection must distinguish live retained probation evidence from applied
   authority evidence by `applied_authority_record` and source event/hash chain.
8. Applied rollback must anchor the scoped decision reason, authorized-append
   hash, exact rollback transaction hash, state-transition versions/generations,
   and effects. No single `applies_rollback:true` needle is adequate.
9. The unframed scoped markers are not envelopes and remain single-line. Their
   marker prefix plus full decision identity must be asserted separately.
10. Selftest case names may survive, but each case must be tied to its enclosing
    family/case count or parsed from that response; global case-name search is
    donor-prone.

### Donor-removal exposure

Other fragments currently contain needles that can pass through this family's
large responses. P4-7b must pre-retire or re-point them before deleting donors:

- `shadow-vm-smoke-profile-full-audit.ps1`: generic `load_attempted:false`,
  retained audit/rollback hashes, `source_method`, and write-boundary loader
  source-map entries can match P4-7 output while their owning P4-1/P4-3 response
  is absent.
- `shadow-vm-smoke-profile-full-module-load-gate.ps1` and
  `...full-module-evidence.ps1`: generic audit/rollback schema, hash,
  `writes_enabled:false`, and `installs_rollback_plan:false` needles are donors.
- `shadow-vm-smoke-profile-full-module-selftests.ps1` outside lines 487-755:
  global `"passed":true`, `"can_load":false`, `"load_attempted":false`, and
  repeated case/status/reason strings can pass through a P4-7 selftest.
- `shadow-vm-smoke-profile-persistence.ps1` and
  `shadow-vm-persistence-reboot.ps1`: broad persistence/rollback strings are not
  proof of this family's LBA1 transaction or reboot durability.
- `shadow-vm-smoke-profile-m8-lifeline.ps1`, `genesis-ui`, `memory-durable`,
  `project-install`, `m12-distribution-provenance`, and `m6c-promotion`: generic
  rollback, target, policy, or event strings must remain anchored to their own
  response family.
- `shadow-vm-smoke.ps1` and support: profile-name/dispatch strings are framing
  only, never semantic proof.

The implementation regeneration must parse the last response by method/family
for all positive-authority checks; a global serial substring is unacceptable.

## 5. Selftest strategy

Existing module selftests and case counts are availability 8, write policy 12,
storage layout 18, append engine 16, append contract 24, append payload hash 20,
append intent 20, and final write boundary 22 (156 total). Existing core scoped
tests cover positive scope, ordered denials, authorized append, verified apply,
and retained inspection (`scoped_rollback_apply.rs:998-1208`). The recovery
inspect-source selftest has seven cases (`runtime.rs:293-EOF`). Guest sequences
cover pre-materialize denial, real materialize write/readback, pre-inspect denial,
inspection, positive apply, applied inspection, and one-shot behavior.

P4-7b must leave the smallest complete host set:

1. all 156 legacy module cases in identical order, rendered through five ordered
   evidence units and one mapping-completeness test;
2. exhaustive scoped evaluator first-failure order for every input, not only the
   current mutation sample;
3. positive `GrantProof` construction only after exact scope, policy, write/
   readback, append and inspection verification; every one-bit/one-hash mutation
   must deny with empty grants/effects;
4. materialize acquisition fixture distinguishing no controller, wrong marker/
   overlap, failed write, failed readback, hash mismatch, and verified LBA1;
5. byte-identical fixtures for every hash class in section 2, especially audit
   record, rollback transaction, sector image, scoped decision and authorized
   append;
6. same-source test proving policy/storage/append/inspection/scoped-apply facts
   come from one captured chain, not independently reacquired caches;
7. applied-state transition test proving mutation occurs only after verified
   apply and remains one-shot;
8. retained inspection/source-event test across live and applied evidence;
9. v1 renderer tests for denial, observed preview/inspect, materialize authority,
   and granted apply; no reserved authority key may enter a facts table;
10. guest verification in order: `module-audit-rollback`, `m6d-rollback`, then
    `hello-rollback-dry-run`, followed by the P4-7 authority block-close profiles
    required by the project rules.

No Cargo or VM command is run by P4-7a.

## 6. Risks and P4-7b STOP-tripwires

1. **Emitter-built proof — SECURITY FINDING.** STOP until the scoped positive
   authority is returned as an unforgeable core evaluator proof. Rendering code
   cannot assemble `ScopedRollbackApplyProof`, synthesize its decision hash, or
   turn evaluator booleans into grants/effects.
2. **Materialize is a real write — OWNER/ORCHESTRATOR DECISION NEEDED.** Decide
   the exact capability evaluator/GrantProof that authorizes the test-media LBA1
   write before I/O. Do not label it observed or denial-only merely because the
   method says `dry_run`.
3. **Hash/response coupling — ORCHESTRATOR DECISION NEEDED.** Preserve the scoped
   decision and authorized-append legacy `Value` grammars as explicit hash-only
   canonical projections, or stop for an approved authority-record migration.
   Normal golden regeneration may not change them.
4. **Grant-proof effect set — OWNER/ORCHESTRATOR DECISION NEEDED.** Fix the exact
   capability/grants/effects for materialization, authorized append, and
   current-boot state apply. Do not reuse `module_grant_decision` or invent broad
   capability strings.
5. **I/O/apply ownership.** STOP if W6 relocation moves PCI/MMIO, AHCI cache or
   write/readback/inspection calls, `STATE` mutation, event retention, or apply
   into `raios-core`. Only pure evaluation/projection moves.
6. **Durable-store semantics.** A successful readback proves the current LBA1
   sector image, not reboot durability, append history, rollback installation,
   or a general durable store. Preserve `current_boot_target_region_lba1` and do
   not claim persistence beyond observed evidence.
7. **Positive ordering.** STOP if any write occurs newly or moves earlier than
   its chosen grant proof, or if state mutation occurs before verified append/
   inspection. Existing historical ordering must be documented honestly while
   repairing it.
8. **Mixed Hello ownership.** Preserve P4-6 lifecycle/state response semantics
   and P4-4 event ownership. `emitters.rs`, `lifecycle_binding.rs`, `records.rs`,
   `runtime.rs`, and `state_machine.rs` are function-level boundaries, not files
   available for wholesale deletion.
9. **Probation/state hash dependency.** STOP on any P4-6 probation, migration,
   descriptor, artifact, or state hash drift. These are authority inputs.
10. **Same-source/cache race.** STOP if a v1 response mixes a fresh policy fact
    with cached write/readback or inspection from a different sector-plan hash.
    Capture and bind one chain.
11. **Applied inspection retention.** Preserve the transaction/write-readback/
    inspection equality check at `state_machine.rs:552-576` and core retention
    predicate `scoped_rollback_apply.rs:799-810`.
12. **No authority from field tables.** Reserved decision keys stay outside
    facts/evidence tables; denied/observed variants cannot carry grants/effects.
13. **Predicate regeneration.** STOP on a multiline needle without `` `r`r`n ``,
    a needle spanning response `id`, an InlineObject treated as multiline, or a
    positive check using global serial substring instead of its response.
14. **Donor exposure.** Pre-retire/re-point the other-fragment needles in section
    4 before deleting any old emitter; otherwise green predicates are vacuous.
15. **Isolation.** P4-7 remains isolated. Do not fold provider, system/status,
    lifecycle, event, loader-runtime, persistence-reboot, or recovery-lifeline
    families into this critical slice.

Scope-creep observation: the design row names `hello_service rollback
emitters/foundation`, but the real boundary crosses AHCI I/O, core evaluators,
state mutation, event retention, mixed lifecycle bindings, and two unframed
authority markers. P4-7b must cut by the exact functions above. The module
diagnostic half is denial-only; the Hello half contains real LBA1 write/readback
and positive current-boot apply. Treating them as one undifferentiated renderer
would erase the trust boundary this manifest exists to protect.

## Orchestrator rulings (2026-07-13, binding for P4-7b)

**R1 — THE SECURITY FINDING IS CONFIRMED, and the fix is already proven.**
`ScopedRollbackApplyProof` (seed-kernel/src/hello_service/emitters.rs) is not a proof.
It is a struct the RENDERER assembles out of evaluator booleans and hashes. Any kernel
code could construct one with `authorized: true` and nothing would stop it. The name
promises unforgeability the type does not deliver.

The fix is the pattern P4-5b1 already landed and proved on the append evaluator:
- The proof type lives in raios-core, next to the evaluator that decides it.
- Its fields are PRIVATE. It is constructible ONLY inside that module, ONLY after the
  whole chain verifies.
- The requested capability and effects are passed IN by the caller and merely CERTIFIED
  by core — core never mints a capability.
- A passing chain with NO caller-declared capability yields NO proof. (P4-5b1's
  `passed_gate_without_caller_authority_labels_has_no_proof` is the regression guard.)
Apply it to `evaluate_scoped_rollback_apply` (the scope decision — this is the one that
gates the WRITE, so it is load-bearing) and to `evaluate_scoped_rollback_verified_apply`.
The kernel's ScopedRollbackApplyProof then stops being a bag of booleans and becomes a
carrier of core proofs. Not one guard, hash, or reason string may change.

**R2 — the write ordering is DEFENSIBLE, but its enforcement is not. Fix the enforcement.**
I checked the code before ruling. The manifest says materialization "performs the real
LBA1 write BEFORE the scoped apply evaluator verifies the chain". That is literally true
and is NOT the vulnerability: you cannot verify a readback before you have written. The
order is scope-evaluator authorizes -> write -> append-evaluator verifies the readback
and inspection. That is correct.

The REAL weakness is how the write is gated. `hello_rollback_target_region_authorized_append_write_readback`
(rollback_authority_gates.rs:3675) has exactly ONE caller (emitters.rs:4083), and that
caller does check `if !scope_decision.authorized { return input; }` first (emitters.rs:4071).
So today it is gated — by a CONVENTION in one call site. A second caller added tomorrow
could write to the media without ever consulting the evaluator, and nothing in the type
system would object.

RULING: make the write function REQUIRE the scope proof as an argument
(`&ScopedRollbackApplyProof` from R1, the core one). Then verify-then-write is enforced by
the COMPILER, not by an if-statement someone must remember to copy. That is the whole
point of a fail-closed substrate: the unsafe program should not compile.

**R3 — hash grammars are frozen; only the RESPONSE moves.** The legacy `Value` grammars
for the scoped decision and the authorized append are HASH INPUTS. Preserve them as
explicit hash-only canonical projections, byte-identical. The response vocabulary moves;
the hash input grammar does not. Golden regeneration is NOT permitted — if a golden
changes, the durable format broke and that is a STOP.

**R4 — capability and effects come from the code, not from imagination.** The capability
is HELLO_ROLLBACK_APPLY_CAPABILITY (it already exists and is already declared as
`requested_capability` at emitters.rs:248/328/387/453). The append effect is
`durable_transaction_append` (established in P4-5b1). For materialization and current-boot
state apply, DERIVE the effect names from what the code already does and name them
exactly. Do NOT reuse `module_grant_decision`. Do NOT invent broad capability strings. If
an effect name cannot be derived from existing behavior, STOP and report — do not guess.

**R5 — I/O ownership confirmed.** Only pure evaluation and projection move to raios-core.
PCI/MMIO, AHCI cache, the write/readback/inspection calls, `STATE` mutation, event
retention, and apply stay in the kernel. A relocation that moves any of them is a STOP.

**R6 — say exactly what a readback proves.** A successful readback proves the CURRENT LBA1
sector image. It does NOT prove reboot durability, append history, rollback installation,
or that the transaction survives a power cut. The evidence records must state the narrow
truth, not the flattering one. (This is the same discipline as "retention is provenance,
not an effect".)

## P4-7b2 worker note (2026-07-13, append-only)

R2-CORRECTED is implemented at the two non-circular side-effect boundaries:

- `hello_rollback_target_region_authorized_append_write_readback` now requires
  `&raios_core::scoped_rollback_apply::ScopedRollbackApplyProof` before its existing
  arguments. Its only caller is `scoped_rollback_authorized_append_input` in
  `hello_service/emitters.rs`. The former `if !scope_decision.authorized` guard is
  unreachable as an authority boundary and was replaced by extraction of the evaluator-owned
  proof; without the proof the call cannot be expressed.
- `rollback_apply_verified` now requires
  `&raios_core::scoped_rollback_apply::ScopedRollbackVerifiedApplyProof` before its existing
  arguments. Its only caller is `rollback_apply` in `hello_service/state_machine.rs`; the
  `STATE` mutation remains unchanged and is reached only after extracting the verified-apply
  proof.
- The constructible kernel carrier was renamed from `ScopedRollbackApplyProof` to
  `ScopedRollbackApplyEvidence`. It carries the scope, authorized-append, and verified-apply
  core proofs alongside the unchanged rendering/hash inputs; it is not itself authority.

The two prerequisite writes remain findings, not changes:

1. `hello_rollback_append_sector_write_readback_dry_run` calls
   `ahci::write_readback_scratch_sector_image` from five acquisition/projection call sites.
   No typed evaluator proof or caller-side authority guard precedes the call. The AHCI path
   fail-closes on controller/MMIO/device discovery and the labeled scratch-region bounds and
   overlap checks, but those device-safety checks are not capability authority.
2. `recovery_rollback_materialization_evidence` is the sole caller of
   `hello_rollback_target_region_write_readback_dry_run`, which calls
   `ahci::write_readback_audit_rollback_target_sector_image`. It passes a
   `TargetRegionMediaWritePolicyPreflight`, but the acquisition adapter invokes AHCI before
   that preflight is consumed by the returned evidence projection. The AHCI path fail-closes
   on controller/MMIO/device discovery, the labeled target region, bounds, boot/partition
   metadata exclusion, and scratch overlap; again, those checks are not an unforgeable
   capability proof.

STOP: the v1 materialization response cannot truthfully render a positive grant from an
evaluator-owned proof because writes 1 and 2 have no such proof producer. Rendering it as
`observed` would hide real media effects, while rendering it as `granted` would synthesize
authority in the kernel. Per R2-CORRECTED no evaluator was invented in this packet, and the
remaining rollback-family v1 response/predicate conversion is not attempted past this STOP.
The frozen scoped-decision and authorized-append hash-only `Value` projections were not
changed.

### R2 CORRECTED — the proof ladder, and why a downstream proof cannot gate an upstream write

The P4-7b2 worker stopped and found something I had missed. My R2 said "make the write
functions require the scope proof". That is right for ONE of them and IMPOSSIBLE for two,
because the writes and the evaluators form a LADDER in which each write PRODUCES the evidence
the next evaluator checks:

  1. scratch-sector write            -> produces write_readback evidence
  2. initial LBA1 materialization    -> produces target_region_write_readback evidence
  3. SCOPE evaluator verifies 1 + 2  -> ScopedRollbackApplyProof
  4. authorized-append LBA1 write
  5. APPEND evaluator verifies 4     -> ScopedRollbackAuthorizedAppendProof
  6. VERIFIED-APPLY evaluator        -> ScopedRollbackVerifiedApplyProof
  7. the real STATE mutation

`ScopedRollbackApplyProof` requires `scratch_readiness_verified` and
`target_region_write_readback_verified` — evidence that writes 1 and 2 CREATE. Demanding that
proof at writes 1 and 2 would be circular and permanently denied. The general rule the ladder
teaches:

**Gate each write with the proof of the evaluator that authorizes IT — never with a
downstream one.** A proof cannot guard the write that produces the evidence it is made of.

So R2 becomes:
- write 4 REQUIRES &ScopedRollbackApplyProof   (non-circular; the proof precedes it)
- step 7 REQUIRES &ScopedRollbackVerifiedApplyProof (non-circular)
- writes 1 and 2 are NOT gated by a downstream proof, and P4-7b2 does not force it.

What authorizes writes 1 and 2 is a separate question and is being answered explicitly rather
than assumed. If their gate turns out to be a bare boolean any caller can satisfy, that is a
NEW finding of the same family as the one this slice is closing — it gets recorded with its
function names and call sites, and it does NOT get an evaluator invented for it under a
rendering deadline. That rule has held for the whole phase and it holds here.

### NEW FINDING (P4-7b2) — the materialization write checks its policy AFTER it has written

While gating the writes, the worker traced every real media write in the family. Two of the
four are now gated by the compiler. The trace turned up something that was not on anyone's
list:

**`recovery_rollback_materialization_evidence` performs the initial LBA1 media write via
`ahci::write_readback_audit_rollback_target_sector_image`, and its policy preflight is
CONSUMED AFTER the AHCI call.** It is not a pre-write gate at all. The write happens; then the
policy is consulted about whether it should have. Write-then-check.

That is a different bug from the one this slice set out to fix (an emitter-assembled "proof"),
and it is a worse one, because no amount of type discipline downstream can un-write a sector.
It is recorded here rather than fixed under a rendering deadline, per the rule that has held
all phase: inventing an evaluator to satisfy a slice is how a fail-closed system grows a back
door.

**Also unauthorized by any typed gate: the scratch-sector write**
(`hello_rollback_append_sector_write_readback_dry_run`, five call sites). AHCI's label, bounds
and overlap checks give it DEVICE safety — they do not give it CAPABILITY authority. A caller
with no business writing anything can still call it and the sector will be written.

**Consequence for P4-7's response conversion (PART 2 is deliberately NOT done):** the
materialization response cannot be rendered honestly today. `observed` would hide a real media
write; `granted` would forge an authority no evaluator issued. Both are the failures this whole
vocabulary exists to prevent. So the materialization/scratch response paths are CARVED OUT of
P4-7 and named, exactly as P4-9's D1 carved out project/install/distribution for the same
reason. What P4-7 DOES land is the part that can be true: the compiler now refuses to emit the
authorized-append write or the STATE mutation without a proof that cannot be forged.

Follow-up work, in priority order:
1. Move the materialization policy preflight BEFORE the AHCI call and give it a typed proof;
   make the write require it. (This is a correctness bug, not a vocabulary one.)
2. Give the scratch write a typed pre-write gate, or prove that it can never reach media a
   caller is not entitled to.
3. Then, and only then, convert the materialization/scratch responses to v1.
