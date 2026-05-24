# Project Status

Development memory for future agents: build normal changes in the repo with
real code, tests, VM reports, and docs; do not fake the finished raiOS memory
architecture during development. Keep slices on the final architecture path by
splitting stable boundaries early, separating runtime/diagnostic/harness/handoff
surfaces, and making observed execution evidence more authoritative than copied
command lists or prose summaries.

Last verified locally: 2026-05-24 on Windows with QEMU 11 after suppressing
framebuffer redraws for serial command-mode echo, caching Shadow VM serial-log
reads, moving recovery load-binding evaluation, retained-chain mismatch checks,
and load-binding selftest fixtures into
`seed-kernel/src/agent_protocol_recovery_load_binding.rs`, moving recovery
lifeline protocol/vocabulary/runtime/rollback/persistence/memory/admission
evaluators and selftest fixtures into
`seed-kernel/src/agent_protocol_recovery_lifeline_eval.rs`, moving recovery
lifeline command reference parsers, evaluators, and event-log binding builders
into `seed-kernel/src/agent_protocol_recovery_command_reference_eval.rs`,
moving command envelope/dispatch/body and downstream command evaluator
selftest helpers into
`seed-kernel/src/agent_protocol_recovery_command_eval.rs`, and extracting the
recovery lifeline command specs into
`seed-kernel/src/agent_protocol_recovery_lifeline.rs`, updating Shadow VM
reports to derive `commands`/`executed_commands` from actual serial command
execution, moving recovery lifeline execution-stage selftest fixtures and
retained-chain matchers plus execution-stage JSON emission and retained-event
recording plus the thin execution-stage public wrapper methods and
method-predicate wiring plus retained execution-stage chain-presence evaluation
into
`seed-kernel/src/agent_protocol_recovery_execution.rs`, extracting recovery
method predicates and diagnostic argument parsers into
`seed-kernel/src/agent_protocol_recovery_methods.rs`, extracting recovery
capability, selftest-count, and boundary-id constants into
`seed-kernel/src/agent_protocol_recovery_constants.rs`, moving recovery
load-binding types into
`seed-kernel/src/agent_protocol_recovery_load_binding.rs`, moving recovery
artifact-reference types into
`seed-kernel/src/agent_protocol_recovery_artifact_types.rs`, moving recovery
artifact-reference parsers, evaluators, selftest fixtures, and event-log
binding builders into
`seed-kernel/src/agent_protocol_recovery_artifact_reference.rs`, moving lifeline
protocol and command-vocabulary types into
`seed-kernel/src/agent_protocol_recovery_lifeline_protocol_types.rs`, moving
lifeline runtime/isolation/rollback/persistence/provenance/admission types into
`seed-kernel/src/agent_protocol_recovery_runtime_types.rs`, moving command
envelope, dispatch-denial, and body-canonicalization types into
`seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs`, moving
handler/status/rollback-authorization/target-binding types into
`seed-kernel/src/agent_protocol_recovery_command_authorization_types.rs`,
moving memory/durable-write/service-inventory/command-effect gate types into
`seed-kernel/src/agent_protocol_recovery_command_effect_types.rs`, moving
recovery artifact-reference emit helpers into
`seed-kernel/src/agent_protocol_recovery_artifact_reference_emit.rs`, moving
recovery artifact/lifeline request selftest emit helpers into
`seed-kernel/src/agent_protocol_recovery_artifact_selftest_emit.rs`, moving
lifeline protocol emit helpers into
`seed-kernel/src/agent_protocol_recovery_lifeline_protocol_emit.rs`, moving
lifeline command-vocabulary emit helpers into
`seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_emit.rs`,
moving loader-runtime-isolation emit helpers into
`seed-kernel/src/agent_protocol_recovery_loader_runtime_emit.rs`, moving
rollback-transaction, durable-persistence, memory-provenance, and
command-admission emit helpers into
`seed-kernel/src/agent_protocol_recovery_rollback_transaction_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_persistence_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_memory_provenance_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_command_admission_emit.rs`, moving
command-envelope, command-dispatch, command-body-canonicalization, and
command-handler emit helpers into
`seed-kernel/src/agent_protocol_recovery_command_envelope_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_command_dispatch_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_command_body_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_command_handler_emit.rs`, moving
status-read, rollback-preview, rollback-apply, and disable/restart/load-target
emit helpers into
`seed-kernel/src/agent_protocol_recovery_status_handler_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_rollback_preview_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_rollback_apply_emit.rs`, and
`seed-kernel/src/agent_protocol_recovery_target_binding_emit.rs`, moving
memory-write, durable-write, service-inventory side-effect, and command-effect
emit helpers into
`seed-kernel/src/agent_protocol_recovery_memory_write_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_durable_write_emit.rs`,
`seed-kernel/src/agent_protocol_recovery_service_inventory_effect_emit.rs`,
and `seed-kernel/src/agent_protocol_recovery_command_effect_emit.rs`, moving
recovery load-binding emit helpers into
`seed-kernel/src/agent_protocol_recovery_load_binding_emit.rs`, and
preserving the
previously verified guest
`module.audit_rollback_availability`,
`module.audit_rollback_availability_selftest`,
`module.audit_rollback_write_policy`,
`module.audit_rollback_write_policy_selftest`,
`module.audit_rollback_storage_layout`,
`module.audit_rollback_storage_layout_selftest`,
`module.audit_rollback_append_engine`,
`module.audit_rollback_append_engine_selftest`,
`module.audit_rollback_append_contract`,
`module.audit_rollback_append_contract_selftest`,
`module.audit_rollback_append_payload_hash`,
`module.audit_rollback_append_payload_hash_selftest`,
`module.audit_rollback_append_intent`,
`module.audit_rollback_append_intent_selftest`,
`module.audit_rollback_write_boundary`, and
`module.audit_rollback_write_boundary_selftest`, plus denied
`recovery.load_artifact`/`module.load_recovery_artifact`, read-only
`recovery.identity_diagnostic`/`recovery.identity_diagnostic_selftest`,
`recovery.trust_diagnostic`/`recovery.trust_diagnostic_selftest`, and
`recovery.vm_test_diagnostic`/`recovery.vm_test_diagnostic_selftest`,
`recovery.local_approval_diagnostic`/
`recovery.local_approval_diagnostic_selftest`,
`recovery.load_binding`/`recovery.load_binding_selftest`, and
`recovery.lifeline_request_diagnostic`/
`recovery.lifeline_request_diagnostic_selftest`, plus
`recovery.lifeline_protocol_diagnostic`/
`recovery.lifeline_protocol_diagnostic_selftest`, plus
`recovery.lifeline_command_vocabulary`/
`recovery.lifeline_command_vocabulary_selftest`, plus
`recovery.loader_runtime_isolation`/
`recovery.loader_runtime_isolation_selftest`, plus
`recovery.rollback_transaction_engine`/
`recovery.rollback_transaction_engine_selftest`, plus
`recovery.durable_audit_rollback_persistence`/
`recovery.durable_audit_rollback_persistence_selftest`, plus
`recovery.memory_provenance`/`recovery.memory_provenance_selftest`, plus
`recovery.lifeline_command_admission`/
`recovery.lifeline_command_admission_selftest`, plus
`recovery.lifeline_command_envelope_diagnostic`/
`recovery.lifeline_command_envelope_diagnostic_selftest`, plus
`recovery.lifeline_command_dispatch_diagnostic`/
`recovery.lifeline_command_dispatch_diagnostic_selftest`, plus
`recovery.lifeline_command_body_canonicalization_diagnostic`/
`recovery.lifeline_command_body_canonicalization_diagnostic_selftest`, plus
`recovery.lifeline_command_handler_binding_diagnostic`/
`recovery.lifeline_command_handler_binding_diagnostic_selftest`, plus
`recovery.lifeline_status_read_handler_diagnostic`/
`recovery.lifeline_status_read_handler_diagnostic_selftest`, plus
`recovery.rollback_preview_authorization_diagnostic`/
`recovery.rollback_preview_authorization_diagnostic_selftest`, plus
`recovery.rollback_apply_authorization_diagnostic`/
`recovery.rollback_apply_authorization_diagnostic_selftest`, plus
`recovery.disable_module_target_binding_diagnostic`/
`recovery.disable_module_target_binding_diagnostic_selftest`, plus
`recovery.restart_last_good_target_binding_diagnostic`/
`recovery.restart_last_good_target_binding_diagnostic_selftest`, plus
`recovery.load_artifact_by_hash_target_binding_diagnostic`/
`recovery.load_artifact_by_hash_target_binding_diagnostic_selftest`, plus
`recovery.memory_write_authority_diagnostic`/
`recovery.memory_write_authority_diagnostic_selftest`, plus
`recovery.durable_audit_rollback_write_authority_diagnostic`/
`recovery.durable_audit_rollback_write_authority_diagnostic_selftest`, plus
`recovery.service_inventory_side_effect_boundary_diagnostic`/
`recovery.service_inventory_side_effect_boundary_diagnostic_selftest`, plus
`recovery.lifeline_command_dispatch_behavior_diagnostic`/
`recovery.lifeline_command_dispatch_behavior_diagnostic_selftest`, plus
`recovery.lifeline_command_executor_capability_table_diagnostic`/
`recovery.lifeline_command_executor_capability_table_diagnostic_selftest`, plus
`recovery.lifeline_command_side_effect_gate_diagnostic`/
`recovery.lifeline_command_side_effect_gate_diagnostic_selftest`, plus
`recovery.lifeline_command_execution_enablement_diagnostic`/
`recovery.lifeline_command_execution_enablement_diagnostic_selftest`,
`recovery.lifeline_command_execution_preflight_diagnostic`/
`recovery.lifeline_command_execution_preflight_diagnostic_selftest`,
`recovery.lifeline_command_execution_intent_diagnostic`/
`recovery.lifeline_command_execution_intent_diagnostic_selftest`, and
`recovery.lifeline_command_execution_commit_gate_diagnostic`/
`recovery.lifeline_command_execution_commit_gate_diagnostic_selftest`, and
`recovery.lifeline_command_execution_result_denial_diagnostic`/
`recovery.lifeline_command_execution_result_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_audit_denial_diagnostic`/
`recovery.lifeline_command_execution_audit_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_observation_denial_diagnostic`/
`recovery.lifeline_command_execution_observation_denial_diagnostic_selftest`, and
`recovery.lifeline_command_execution_completion_denial_diagnostic`/
`recovery.lifeline_command_execution_completion_denial_diagnostic_selftest`, plus typed missing
`raios.durable_audit_ledger.v0`/`raios.rollback_store.v0` availability facts,
typed missing `raios.durable_audit_write_policy.v0`/
`raios.rollback_install_policy.v0` policy facts, typed missing
`raios.persistence_device_inventory.v0`/
`raios.audit_rollback_storage_layout.v0` storage-layout facts, typed missing
`raios.audit_ledger_append_engine.v0`/
`raios.rollback_store_transaction_engine.v0` append-engine facts, typed missing
`raios.audit_ledger_append_envelope.v0`/
`raios.rollback_store_transaction_envelope.v0` append-contract facts, typed
`raios.audit_record_append_payload_hash_envelope.v0`/
`raios.rollback_transaction_append_payload_hash_envelope.v0` append payload-hash
envelope facts derived from retained current-boot audit/rollback and
service-slot evidence, typed
missing `raios.audit_record_append_intent.v0`/
`raios.rollback_transaction_append_intent.v0` append-intent facts, and explicit
missing storage-layout, append-engine, append-contract, append-envelope,
append-payload, append-intent stable-id, payload-hash, and provenance binding
inputs over the retained module evidence chain, and typed current-boot
`raios.recovery_artifact_load_denial_evidence.v0` facts for missing recovery
artifact identity, trust, VM-test, local approval, loader, and rollback
evidence on the separate `cap.recovery.load_artifact` path, plus local-only
retained `raios.recovery_artifact_identity.v0` and
`raios.recovery_artifact_trust.v0`,
`raios.recovery_artifact_vm_test.v0`, and
`raios.recovery_artifact_local_approval.v0` hash-reference diagnostics whose
event ids are consumed by `recovery.load_binding`, plus retained recovery-only
evidence-id binding diagnostics that
reject normal module append-intent, append-payload, writer, service-slot, and
`module.load_ephemeral` authority, and a retained local-only
`raios.recovery_lifeline_request.v0` hash-reference diagnostic over the fully
retained recovery evidence chain and a read-only
`raios.recovery_lifeline_protocol_state.v0` diagnostic that consumes that
lifeline request plus the six recovery evidence event ids, plus a read-only
`raios.recovery_lifeline_command_vocabulary.v0` envelope that enumerates
recovery lifeline command ids, argument schemas, required capabilities, and
denial reasons while still denying recovery loading, durable writes, rollback
installs, service-slot allocation, loader execution, direct-OpenAI recovery
shortcuts, command dispatch, and lifeline behavior, plus a read-only
`raios.recovery_loader_runtime_isolation.v0` boundary that enumerates missing
address-space, entrypoint ABI, memory-map, capability-import,
artifact-hash-binding, provider-separation, and normal-module-separation facts
while rejecting invalid request/protocol-state/command-vocabulary inputs and
still loading nothing, plus a read-only
`raios.recovery_rollback_transaction_engine.v0` boundary that reuses the
retained lifeline chain, command-vocabulary envelope, and loader isolation
boundary, enumerates missing rollback target, transaction provenance,
last-good, disabled-module set, artifact-hash, replay, recovery-capability
import, atomic apply/abort, durable persistence, and recovery-memory facts, and
keeps rollback preview/apply non-executable, plus a read-only
`raios.durable_audit_rollback_persistence.v0` boundary that consumes the
rollback engine boundary, enumerates missing persistence device, storage
layout, audit append-log, rollback store, replay cursor, last-good checkpoint,
write ordering, crash consistency, integrity root/hash chain, and
recovery-memory-provenance facts, and keeps durable writes, rollback replay,
recovery-memory writes, and rollback installs disabled, plus a read-only
`raios.recovery_memory_provenance.v0` boundary that consumes that durable
persistence boundary, enumerates missing source record ids, source schema
hashes, classification, authority level, rollback-transaction binding,
last-good checkpoint binding, recovery-only export profile, redaction state,
replay window, and audit linkage facts, and keeps memory writes and provider
export disabled, plus a read-only
`raios.recovery_lifeline_command_admission.v0` boundary that consumes recovery
memory provenance and defines non-executing admission requirements for lifeline
status, rollback preview, rollback apply, disable module, restart last-good,
and load recovery artifact by hash commands while rejecting invalid request,
protocol-state, command-vocabulary, loader-isolation, rollback-engine, durable
persistence, and memory-provenance chains, plus a read-only
`raios.recovery_lifeline_command_envelope_reference.v0` hash-reference
diagnostic that consumes command admission and validates allowed lifeline
command id, argument schema, argument hash, required capability, target
locator, command-admission boundary id, and retained request hash while
accepting no command body and dispatching no command behavior, plus a read-only
`raios.recovery_lifeline_command_dispatch_denial.v0` boundary that consumes the
retained command-envelope reference, exposes missing command body
canonicalization, command handler binding, status handler, rollback
authorization, per-command target binding, memory/durable write authority, and
service-inventory side-effect facts, and still accepts no command body and
dispatches no behavior, plus a read-only
`raios.recovery_lifeline_command_body_canonicalization.v0` hash-reference
diagnostic that consumes the retained command-envelope reference and the
dispatch-denial boundary, validates command id, argument schema, argument hash,
target locator, command-envelope reference hash, dispatch boundary id, and
current-boot scope, retains only local-only current-boot body-canonicalization
hash evidence, exposes missing body schema canonicalization, body redaction/
classification, handler input binding, rollback authorization linkage,
recovery-memory write linkage, durable audit/rollback write linkage, and
service-inventory side-effect linkage facts, and still accepts no raw command
body or command envelope and dispatches no behavior, plus a read-only
`raios.recovery_lifeline_command_handler_binding.v0` hash-reference diagnostic
that consumes the retained body-canonicalization reference, validates command
id, argument schema, argument hash, target locator, command-envelope reference
hash, body-canonicalization hash, dispatch boundary id, handler id, and
handler-input binding hash, retains only local-only current-boot handler
binding evidence, and advances dispatch only to missing status-read handler
while still accepting no raw command body and dispatching no behavior, plus a
read-only `raios.recovery_lifeline_status_read_handler.v0` hash-reference
diagnostic that consumes the retained handler-binding reference, validates
command id, argument schema, argument hash, target locator, command-envelope
reference hash, body-canonicalization hash, handler-binding hash, dispatch
boundary id, status handler id, and status-read projection hash, retains only
local-only current-boot status-read handler evidence, and advances dispatch
only to missing rollback-preview authorization while still executing no status
read and dispatching no behavior, plus a read-only
`raios.recovery_rollback_preview_authorization.v0` hash-reference
diagnostic that consumes the retained status-read handler reference, validates
command id, argument schema, argument hash, target locator, command-envelope
reference hash, body-canonicalization hash, handler-binding hash, status-read
handler hash, dispatch boundary id, rollback-preview authorization id, and
preview projection hash, retains only local-only current-boot preview
authorization evidence, and advances dispatch only to missing rollback-apply
authorization while still executing no rollback preview or recovery command,
plus a read-only `raios.recovery_rollback_apply_authorization.v0`
hash-reference diagnostic that consumes the retained rollback-preview
authorization reference, validates command id, argument schema, argument hash,
target locator, command-envelope reference hash, body-canonicalization hash,
handler-binding hash, status-read handler hash, rollback-preview authorization
hash, dispatch boundary id, rollback-apply authorization id, and apply
projection hash, retains only local-only current-boot apply authorization
evidence, and advances dispatch only to missing disable-module target binding
while still executing no rollback apply or recovery command, plus read-only
`raios.recovery_disable_module_target_binding.v0`,
`raios.recovery_restart_last_good_target_binding.v0`,
`raios.recovery_load_artifact_by_hash_target_binding.v0`, and
`raios.recovery_memory_write_authority.v0` hash-reference diagnostics that
chain rollback-apply authorization through disable/restart/load/memory
authority without disabling modules, restarting services, loading artifacts,
or writing recovery memory, plus a read-only
`raios.durable_audit_rollback_write_authority.v0` hash-reference diagnostic
that consumes the retained recovery-memory write authority, validates the
command/argument/target/envelope/body/handler/status/authorization/target/
memory/dispatch/projection hashes, retains only local-only current-boot
durable-write authority evidence, and advances dispatch only to the missing
service-inventory side-effect boundary while still writing no durable audit or
rollback records and dispatching no behavior, plus a read-only
`raios.recovery_service_inventory_side_effect_boundary.v0` hash-reference
diagnostic that consumes the retained durable-audit/rollback write-authority
reference, validates the same command/argument/target/envelope/body/handler/
status/authorization/target/memory/durable/dispatch/projection hashes, retains
only local-only current-boot service-inventory side-effect boundary evidence,
and advances dispatch only to explicit `defined_non_executable` behavior while
still allocating no service slot, creating no service inventory records,
changing no service inventory, and dispatching no behavior, via
headless
Shadow VM smoke
covering
deterministic `provider_minimal`
packet/field-list evidence, explicit provider request-binding denial and
export-denial audit records, the denied `provider.context_export` gate, the
local redaction projection, read-only memory context, the RAM-only current-boot
event log with structured denial bindings, the runtime
`raios.provider_request_envelope.v0` marker on the real OpenAI request path,
positive local-only request/export audit binding records on the SPKI pinned
OpenAI path, checked current-boot binding consumption with single-use rejection,
a local-only negative gate selftest for stale/dropped,
previous-boot-or-unretained, substituted-schema, substituted-positive-record,
and mismatched-hash cases, the separate fail-closed
`raios.provider_context_injection_gate.v0` diagnostic, local-only negative
final-injection authorization selftests, the fail-closed
`raios.module_load_gate.v0` denial with event-log binding for denied
`module.load_ephemeral`, read-only `module.manifest_diagnostic`,
`module.manifest_diagnostic_selftest`, `module.artifact_diagnostic`,
`module.artifact_diagnostic_selftest`, `module.vm_report_diagnostic`,
`module.vm_report_diagnostic_selftest`, `module.grant_diagnostic`, and
`module.grant_diagnostic_selftest`, `module.attestation_diagnostic`,
`module.attestation_diagnostic_selftest`, `module.load_gate_attestation_selftest`,
`module.approval_diagnostic`, `module.approval_diagnostic_selftest`, and
`module.load_gate_approval_selftest` manifest, candidate-artifact, VM-report,
computed-grant, local-attestation, and local-approval hash-reference diagnostics,
local-only current-boot retention of valid manifest, artifact, VM-report,
computed-grant, local-attestation, and local-approval hash references, the
denied module load gate reporting retained manifest, artifact, VM-report,
computed-grant, local-attestation, and local-approval references without
authorizing loading, guest
audit/rollback hash-reference diagnostics that retain valid references only as
local-only current-boot evidence, the denied module load gate reporting retained
audit/rollback references as non-authorizing hash evidence only after live
current-boot predicate validation, rejection of a wrong-schema retained
audit/rollback reference in the live denied load gate, and local-only negative
manifest, artifact, retained-reference, plus audit/rollback evidence gate
selftests plus
`module.audit_rollback_diagnostic_selftest` guest hash-reference diagnostics,
and guest `module.service_slot_diagnostic` RAM-only service-slot reservation
hash-reference diagnostics that retain valid reservations as local-only
current-boot evidence without allocating a slot or loading artifacts, plus the
denied module load gate live-validating that retained reservation as
non-authorizing service-slot evidence and local-only service-slot gate
selftests for rejected retained reservations, and local-only
audit/rollback write-boundary selftests for missing, stale, substituted,
previous-boot, wrong-schema, mismatched, recovery-separated, and
accepted-current-boot-but-denied candidates, and local-only
`module.load_gate_vm_report_selftest` coverage for missing, stale,
wrong-schema, substituted, hash-mismatched, and binding-mismatched VM-report
references, local-only retained local-attestation reference gate selftests, and
local-only retained local-approval reference gate selftests.
Direct OpenAI pin-mismatch plus SPKI pinned-trust smokes using a fake local API
key remain previously verified from the prior handoff.

Latest host-tool verification: 2026-05-23 on Windows with
`cargo test --locked -p ota-tools -p registry-core -p registry-tools -p fake-cloud-server`
covering OTA/registry tooling plus the non-authorizing
`raios.computed_capability_grant.v0` diagnostic, host-side
`raios.module_audit_rollback_diagnostic.v0` audit/rollback candidates, and
negative manifest/artifact/report/attestation/audit/rollback evidence cases.

Latest quick guest-protocol verification: 2026-05-23 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile quick -TimeoutSeconds 180`, report
`release\vm-reports\shadow-20260523-174556-23200.json` with 136/136
predicates, 13 `executed_commands` entries, and no static command inventory,
covering the real QEMU/serial path through boot readiness, core read-only
methods, provider-minimal export gates, denied `module.load_ephemeral`, denied
`recovery.load_artifact`, and RAM-only audit visibility.

Latest focused recovery guest-protocol verification: 2026-05-24 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile recovery -TimeoutSeconds 180`, report
`release\vm-reports\shadow-20260524-101315-27892.json` with 2725/2725
predicates, 142 `executed_commands` entries, and no static command inventory,
covering the real QEMU/serial path through the recovery artifact boundary,
recovery evidence retention, lifeline-command diagnostics, load-binding denial,
and RAM-only recovery audit visibility while skipping the normal module-loading
diagnostic matrix.

Latest guest-protocol verification: 2026-05-23 on Windows with
`vm-harness\shadow-vm-smoke.ps1 -Profile full -TimeoutSeconds 180`, report
`release\vm-reports\shadow-20260523-223645-13488.json` with 4500/4500
predicates, 206 `executed_commands` entries, and no static command inventory,
covering absent/accepted/stale/mismatched/invalid module-manifest
hash-reference diagnostics, RAM-only retention of valid manifest and
candidate-artifact references, absent/accepted/stale/mismatched/binding-checked
VM-report hash-reference diagnostics, RAM-only retention of a valid VM-report
reference, live denied load-gate visibility of retained manifest, artifact, and
VM-report hash evidence, negative manifest/artifact/VM-report-reference gate
selftests, absent/accepted/stale/mismatched/wrong-policy module computed-grant
hash-reference diagnostics plus RAM-only retention of a valid computed-grant
hash reference and its visibility in the denied module load gate while live
loading remains denied, absent/accepted/stale/mismatched local-attestation
hash-reference diagnostics plus RAM-only retention of a valid local-attestation
reference and its visibility in the denied module load gate only after live
current-boot validation, negative retained-reference and retained
local-attestation-reference gate selftests, negative retained audit/rollback
reference gate selftests, absent/accepted/stale/mismatched local-approval
hash-reference diagnostics plus RAM-only retention of a valid local-approval
reference and its visibility in the denied module load gate only after live
current-boot validation, negative retained local-approval-reference gate
selftests,
read-only `module.audit_rollback_availability` exposing typed current-boot
missing durable audit-ledger and rollback-store availability facts, its
local-only negative selftest coverage, read-only
`module.audit_rollback_write_policy` exposing typed missing durable-write and
rollback-install policy facts plus selftests for stale/schema/provenance/
binding gaps, read-only `module.audit_rollback_storage_layout` exposing typed
missing persistence-device inventory and audit/rollback storage-layout facts
plus current-boot layout selftests, read-only
`module.audit_rollback_append_engine` exposing typed missing audit-ledger
append-engine and rollback-store transaction-engine facts plus current-boot
append-engine readiness selftests, read-only
`module.audit_rollback_append_contract` exposing typed
missing audit-ledger append-envelope and rollback-store transaction-envelope
facts plus explicit storage-layout, append-engine, write-policy, availability,
and provenance binding fields while consuming those diagnostics, read-only
`module.audit_rollback_append_payload_hash` exposing typed local-only
current-boot audit-record and rollback-transaction payload-hash envelope facts
derived from retained audit/rollback candidates, service-slot reservation
evidence, pre-load write-request shape, and bound append-contract ids while
keeping those envelopes non-durable and non-authorizing, read-only
`module.audit_rollback_append_intent` exposing typed missing audit-record and
rollback-transaction append-intent facts plus explicit append-contract,
append-engine, storage-layout, write-policy, availability, payload-hash, and
provenance binding fields while consuming the bound append contract plus
payload-hash envelope readiness, and
read-only `module.audit_rollback_write_boundary`
consuming those availability, policy, storage-layout, append-engine through
append-contract, append-contract facts, append payload-hash envelopes, and
append-intent facts plus the retained module evidence chain and returning
`denied_missing_durable_write_boundary` with
`durable_audit_write_missing`, `rollback_install_missing`,
`storage_layout_missing`, and `append_engine_missing`, plus
`module.audit_rollback_append_payload_hash_selftest` coverage for missing,
previous-boot, wrong-schema, provenance, retained-evidence binding,
service-slot binding, pre-load write-request binding, append-contract id
binding, target-schema binding, payload-hash, retained-evidence missing,
service-slot missing, append-contract missing, and available-but-non-authorizing
payload-hash candidates, plus `module.audit_rollback_append_intent_selftest`
coverage for missing, previous-boot, wrong-schema, provenance, append-contract,
append-engine, storage-layout, write-policy, availability, payload-hash,
payload-envelope missing, and
available-but-non-authorizing append-intent candidates, plus
`module.audit_rollback_write_boundary_selftest` negative coverage for missing,
stale, substituted, previous-boot, wrong-schema, mismatched,
availability-missing, policy-missing, append-contract-missing,
append-intent-missing, payload-envelope-missing, writer-unimplemented, and
recovery-separated candidates,
missing/mismatched durable audit plus rollback evidence selftests, and guest
audit/rollback hash-reference diagnostics over `raios.audit_record.v0` and
`raios.rollback_plan.v0` candidates, including RAM-only retention of a valid
audit/rollback reference, live rejection of a wrong-schema retained
audit/rollback reference, and valid retained audit/rollback visibility in the
denied module load gate, plus RAM-only service-slot reservation diagnostics and
selftests over retained computed-grant/audit/rollback event ids, canonical
reservation hashes, pre-load service-inventory hashes, and `ram_only:` slot ids,
including live denied load-gate visibility of valid retained service-slot
reservation evidence without allocation, local-only negative service-slot gate
selftests, and the separate denied recovery artifact load boundary proving
`cap.recovery.load_artifact`, typed missing recovery identity/trust/VM-test/
approval/loader/rollback facts, event-log binding, no normal module capability
reuse, no recovery artifact load, and no service inventory change, plus
read-only recovery artifact identity/trust hash-reference diagnostics retaining
valid local-only current-boot `raios.recovery_artifact_identity.v0` and
`raios.recovery_artifact_trust.v0` event bindings without artifact bytes or
load authority, read-only recovery artifact VM-test/local-approval
hash-reference diagnostics retaining valid local-only current-boot
`raios.recovery_artifact_vm_test.v0` and
`raios.recovery_artifact_local_approval.v0` event bindings without accepting
VM-test JSON, approval text, artifact bytes, or load authority, read-only
recovery artifact loader/rollback-evidence hash-reference diagnostics retaining
valid local-only current-boot `raios.recovery_artifact_loader.v0` and
`raios.recovery_artifact_rollback_evidence.v0` event bindings without accepting
loader descriptors, rollback evidence JSON, artifact bytes, or load authority,
plus read-only `recovery.load_binding` and `recovery.load_binding_selftest`
proving all six required recovery-only evidence ids, normal module
append-intent, append-payload, writer, service-slot, and
`module.load_ephemeral` facts are non-authority, append payload-hash envelopes
remain non-authority inputs, plus read-only
`recovery.lifeline_request_diagnostic` and
`recovery.lifeline_request_diagnostic_selftest` proving
`raios.recovery_lifeline_request.v0` consumes the retained recovery identity,
trust, VM-test, local-approval, loader, and rollback-evidence event ids only as
local-only current-boot hash references, rejects missing, stale, previous-boot,
wrong-schema, substituted, and mismatched chains, and keeps recovery artifacts
non-loaded, non-durable, local-only, and non-authorizing until a recovery
lifeline protocol behavior exists, plus read-only
`recovery.lifeline_protocol_diagnostic` and
`recovery.lifeline_protocol_diagnostic_selftest` proving
`raios.recovery_lifeline_protocol_state.v0` consumes the retained
`raios.recovery_lifeline_request.v0` event id plus those six recovery evidence
event ids, rejects missing, stale, previous-boot, wrong-schema, substituted,
and mismatched lifeline request/evidence chains before reporting protocol gaps,
and exposes typed local-only missing facts for lifeline protocol state,
command vocabulary, loader runtime isolation, rollback transaction engine,
durable audit/rollback persistence, and recovery memory provenance.
It also covers read-only `recovery.lifeline_command_vocabulary` and
`recovery.lifeline_command_vocabulary_selftest`, proving
`raios.recovery_lifeline_command_vocabulary.v0` enumerates recovery command ids,
argument-envelope schemas, required capabilities, and denial reasons only after
the retained lifeline request/evidence chain validates, rejects missing, stale,
previous-boot, wrong-schema, substituted, and mismatched request/protocol-state
inputs, and keeps command envelopes, command dispatch, recovery loading,
durable writes, rollback installs, loader execution, service-slot allocation,
and service inventory changes disabled. It also covers read-only
`recovery.loader_runtime_isolation` and
`recovery.loader_runtime_isolation_selftest`, proving
`raios.recovery_loader_runtime_isolation.v0` reuses the retained lifeline
request/evidence chain and command-vocabulary envelope, rejects invalid
request/protocol-state/command-vocabulary inputs, exposes missing loader
address-space, entrypoint ABI, memory-map, capability-import, artifact-hash,
provider-separation, normal-module-separation, rollback transaction, durable
persistence, and recovery-memory-provenance facts, and keeps loader execution,
artifact loading, command dispatch, durable writes, rollback installs,
service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled. It also covers read-only
`recovery.rollback_transaction_engine` and
`recovery.rollback_transaction_engine_selftest`, proving
`raios.recovery_rollback_transaction_engine.v0` consumes the retained lifeline
request/evidence chain, command-vocabulary envelope, and loader runtime
isolation boundary, rejects invalid request/protocol-state/command-vocabulary/
loader-isolation inputs, exposes missing rollback target, transaction
id/provenance, last-good, disabled-module set, artifact-hash, replay,
recovery-capability import, atomic apply/abort, durable audit/rollback
persistence, and recovery-memory-provenance facts, and keeps rollback preview,
rollback apply, lifeline command dispatch, loader execution, artifact loading,
durable writes, rollback installs, service-slot allocation, direct-OpenAI
recovery shortcuts, and service inventory changes disabled. It also covers
read-only `recovery.durable_audit_rollback_persistence` and
`recovery.durable_audit_rollback_persistence_selftest`, proving
`raios.durable_audit_rollback_persistence.v0` consumes the rollback transaction
engine boundary after the retained lifeline chain, command-vocabulary envelope,
and loader isolation boundary validate, rejects invalid rollback-engine/
loader-isolation/command-vocabulary/protocol-state/request inputs, exposes
missing persistence-device inventory, storage-layout identity, audit append-log
identity, rollback-store identity, replay cursor, last-good checkpoint, write
ordering, crash consistency, integrity root/hash chain, and recovery-memory
provenance facts, and keeps durable writes, rollback replay, recovery-memory
writes, rollback preview/apply, loader execution, artifact loading, rollback
installs, service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled.
It also covers read-only `recovery.memory_provenance` and
`recovery.memory_provenance_selftest`, proving
`raios.recovery_memory_provenance.v0` consumes the durable persistence boundary
after the retained lifeline chain, command-vocabulary envelope, loader
isolation boundary, and rollback transaction-engine boundary validate, rejects
invalid durable-persistence/rollback-engine/loader-isolation/
command-vocabulary/protocol-state/request inputs, exposes missing source record
ids, source schema hashes, classification, authority level,
rollback-transaction binding, last-good checkpoint binding, recovery-only
export profile, redaction state, replay window, and audit linkage facts, and
keeps memory writes, provider export, durable writes, rollback replay,
rollback preview/apply, command dispatch, loader execution, artifact loading,
rollback installs, service-slot allocation, direct-OpenAI recovery shortcuts,
and service inventory changes disabled.
It also covers read-only `recovery.lifeline_command_admission` and
`recovery.lifeline_command_admission_selftest`, proving
`raios.recovery_lifeline_command_admission.v0` consumes the recovery memory
provenance boundary after the retained lifeline chain, command-vocabulary
envelope, loader isolation boundary, rollback transaction-engine boundary, and
durable persistence boundary validate, rejects invalid memory-provenance/
durable-persistence/rollback-engine/loader-isolation/command-vocabulary/
protocol-state/request inputs, enumerates non-executing admission requirements
for lifeline status, rollback preview, rollback apply, disable module, restart
last-good, and load recovery artifact by hash command envelopes, and keeps
command envelopes, command dispatch, rollback preview/apply, memory writes,
provider export, durable writes, rollback replay, loader execution, artifact
loading, rollback installs, service-slot allocation, direct-OpenAI recovery
shortcuts, and service inventory changes disabled.
It also covers read-only `recovery.lifeline_command_envelope_diagnostic` and
`recovery.lifeline_command_envelope_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_envelope_reference.v0` consumes command
admission after the retained lifeline chain, validates only hash/reference
shape for lifeline status, rollback preview/apply, disable module, restart
last-good, and load-artifact-by-hash command ids, rejects invalid
command-admission/memory-provenance/durable-persistence/rollback-engine/
loader-isolation/command-vocabulary/protocol-state/request chains, and keeps
command bodies, command envelope acceptance, command dispatch, rollback
preview/apply, memory writes, provider export, durable writes, rollback replay,
loader execution, artifact loading, rollback installs, service-slot
allocation, direct-OpenAI recovery shortcuts, and service inventory changes
disabled while retaining a valid status-command hash reference only as
local-only current-boot evidence.
It also covers read-only `recovery.lifeline_command_dispatch_diagnostic` and
`recovery.lifeline_command_dispatch_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_dispatch_denial.v0` consumes the retained
command-envelope reference, rejects invalid command-envelope/admission/
memory-provenance/durable-persistence/rollback-engine/loader-isolation/
command-vocabulary/protocol-state/request chains, exposes missing command body
canonicalization, command handler binding, status-read handler,
rollback-preview/apply authorization, disable-module/restart-last-good/
load-artifact-by-hash target bindings, recovery-memory write authority,
durable audit/rollback write authority, and service-inventory side-effect
facts, and keeps command bodies, command envelope acceptance, command dispatch,
rollback preview/apply, memory writes, provider export, durable writes,
rollback replay, loader execution, artifact loading, rollback installs,
service-slot allocation, direct-OpenAI recovery shortcuts, and service
inventory changes disabled.
It also covers read-only
`recovery.lifeline_command_body_canonicalization_diagnostic` and
`recovery.lifeline_command_body_canonicalization_diagnostic_selftest`, proving
`raios.recovery_lifeline_command_body_canonicalization.v0` consumes the
retained command-envelope reference plus the dispatch-denial boundary,
validates only the canonical command-body hash/reference shape, rejects invalid
dispatch/envelope/admission/memory-provenance/durable-persistence/
rollback-engine/loader-isolation/command-vocabulary/protocol-state/request
chains, retains a valid status-command body-canonicalization reference only as
local-only current-boot evidence, and causes the dispatch diagnostic to advance
only to the next missing handler-binding fact while still accepting no raw
command body and dispatching no recovery command.

## Verified Boot State

- Repository path: `C:\Users\admin\Documents\raios2`
- Boot image: `release/raios-stage0.img`
- Firmware vars seed: `release/ovmf_vars.fd`
- Bootloader: Limine 10 UEFI binary at `release/esp/EFI/BOOT/BOOTX64.EFI`
- Config file: `limine.conf` at ESP root and `EFI/BOOT/limine.conf`
- Kernel path inside image: `/kernel/kernel.elf`

The image boots in QEMU using the Windows PowerShell runner:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting
```

For interactive serial commands, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555
```

For a QEMU xHCI inventory run, add `-UsbXhciInput`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-qemu.ps1 -StopExisting -SerialMode tcp -SerialTcpPort 4555 -Headless -UsbXhciInput
```

For the bare-metal-style VM profile with USB keyboard, USB mouse, RDRAND, and
e1000 networking, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run-stage0-baremetal-vm.ps1 -StopExisting
```

Expected xHCI inventory lines in that mode:

```text
usb-xhci: controller @ 00:03.0 detected
usb-xhci: hci 0x0100, ports 8, connected 2
usb-hid: boot keyboard ready on slot 1 endpoint 0x81
usb-hid: boot mouse ready on slot 2 endpoint 0x81
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
```

Expected visible framebuffer UI:

```text
AI  CONSOLE                                      SET
RAIOS
DIRECT AI HOST
NET CONFIGURED   INPUT READY   USB READY   RNG READY
CHAT
TYPE MESSAGE AND PRESS ENTER
```

Expected useful serial lines:

```text
Seed kernel: early init start
Limine loaded base revision: 3
HHDM offset=0xffff800000000000
Framebuffer response revision: 1
Framebuffer negotiated via Limine
status FRAMEBUFFER: READY - 1280x800 PITCH 5120
status ENTROPY: READY - FILL 64/64 TOTAL 64 SRC RDRAND
status USB-XHCI: READY - 00:03.0 HCI 0100 PORTS 8 CONNECTED 2 KBD READY MOUSE READY
e1000: device 00:02.0 id=0x100e mmio=0x81040000 size=131072 mac 52:54:00:12:34:56
e1000 network initialised; DHCP polling enabled
DHCP lease acquired: ip 10.0.2.15/24 gw 10.0.2.2 dns ["10.0.2.3"]
status NETWORK: CONFIGURED - IP 10.0.2.15/24 GW 10.0.2.2
status INPUT: READY - USB HID KEYBOARD + POINTER
```

Console commands verified over TCP serial and USB-HID keyboard input:

```text
help
status
devices
log
provider
openai
setup
ask <text>
```

The framebuffer UI defaults to an AI chat mode. The `CONSOLE` tab keeps the
debug console visible, and the `SET` tab opens provider settings. `setup` also
opens the in-VM OpenAI/API-key menu. API-key entry is masked, held only in guest
RAM, and not printed into the console or serial output. For local-only testing,
the build scripts can also embed `OPENAI_API_KEY` into a separate non-default
image with `-EmbedOpenAiApiKeyFromEnv`.

Direct OpenAI trust-gate smoke over TCP serial:

```text
> provider
PROVIDER: OPENAI    API KEY: SET
ROUTE: OPENAI DIRECT
TLS TRUST: pin_config_missing
> ask direct provider smoke
OPENAI TLS TRUST DENIED: pin_config_missing
```

Direct OpenAI SPKI pinned-trust smoke is verified with a temporary image built
from a process-local fake API key and a current `OPENAI_SPKI_SHA256` pin.
Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_spki sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

The legacy leaf-certificate pinned-trust smoke remains supported with
`OPENAI_CERT_SHA256`. Expected positive trust lines:

```text
openai: TLS 1.3 established
openai: TLS provider trust verified: pinned_cert sha256:<pin-id>
openai: HTTPS request sent
OPENAI HTTP
```

## Current Architecture Decision

Do not run or port the Codex CLI inside Stage-0.

Stage-0 should grow a small native agent host:

- framebuffer UI
- serial/keyboard/mouse input
- USB/input and PCI device inventory
- network status
- explicit capability-gated agent tools

Codex/OpenAI integrations should use a small native provider boundary. The OS
boundary should stay small and auditable; a full host CLI is not part of
Stage-0.

See `docs/architecture-decisions/0001-raios-agent-protocol.md`.

## Exact Next Task

Continue splitting the recovery lifeline command machinery without changing
behavior:

- the latest behavior-neutral slice moved the six recovery lifeline command
  specs and dispatch boundary constant into
  `seed-kernel/src/agent_protocol_recovery_lifeline.rs`, keeping public command
  vocabulary and all schema/boundary ids unchanged
- the follow-up behavior-neutral slice moved execution-stage selftest case
  construction, retained-chain reference matchers, JSON response emission, and
  retained execution-stage event recording into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`
- the latest behavior-neutral slice moved the thin execution-stage public
  wrapper methods and method-predicate wiring into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`, and the central
  agent dispatcher now imports those wrappers directly from the execution module
- the current behavior-neutral slice moved retained execution-stage
  chain-presence evaluation into
  `seed-kernel/src/agent_protocol_recovery_execution.rs` while leaving the
  recovery dispatch candidate type in `agent_protocol_recovery.rs`
- the previous behavior-neutral slice moved the shared execution-stage
  descriptor/input ownership, method/argument matching helpers, stage
  descriptor constants, execution-stage boundary IDs, reference-check type,
  parser/evaluator, hash-validation, and live-chain validation helpers into
  `seed-kernel/src/agent_protocol_recovery_execution.rs`
- the latest behavior-neutral slices moved recovery method predicates and
  diagnostic argument parsers into
  `seed-kernel/src/agent_protocol_recovery_methods.rs`, recovery capability,
  selftest-count, and boundary-id constants into
  `seed-kernel/src/agent_protocol_recovery_constants.rs`, recovery
  load-binding types into
  `seed-kernel/src/agent_protocol_recovery_load_binding.rs`, recovery
  artifact-reference types into
  `seed-kernel/src/agent_protocol_recovery_artifact_types.rs`, plus recovery
  artifact-reference parsers, evaluators, selftest fixtures, and event-log
  binding builders into
  `seed-kernel/src/agent_protocol_recovery_artifact_reference.rs`, and lifeline
  protocol/command-vocabulary types into
  `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_types.rs`, plus
  lifeline runtime/isolation/rollback/persistence/provenance/admission types
  into `seed-kernel/src/agent_protocol_recovery_runtime_types.rs`, and command
  envelope/dispatch-denial/body-canonicalization types into
  `seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs`, plus
  handler/status/rollback-authorization/target-binding types into
  `seed-kernel/src/agent_protocol_recovery_command_authorization_types.rs`,
  and memory/durable-write/service-inventory/command-effect gate types into
  `seed-kernel/src/agent_protocol_recovery_command_effect_types.rs`, plus
  recovery lifeline command reference parsers, evaluators, and event-log
  binding builders into
  `seed-kernel/src/agent_protocol_recovery_command_reference_eval.rs`, plus
  command envelope/dispatch/body and downstream command evaluator selftest
  helpers into `seed-kernel/src/agent_protocol_recovery_command_eval.rs`, plus
  recovery artifact-reference emit helpers into
  `seed-kernel/src/agent_protocol_recovery_artifact_reference_emit.rs`,
  recovery artifact/lifeline request selftest emit helpers into
  `seed-kernel/src/agent_protocol_recovery_artifact_selftest_emit.rs`, and
  lifeline protocol emit helpers into
  `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_emit.rs`, plus
  lifeline command-vocabulary emit helpers into
  `seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_emit.rs`,
  and loader-runtime-isolation emit helpers into
  `seed-kernel/src/agent_protocol_recovery_loader_runtime_emit.rs`, plus
  rollback-transaction, durable-persistence, memory-provenance, and
  command-admission emit helpers into
  `seed-kernel/src/agent_protocol_recovery_rollback_transaction_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_persistence_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_memory_provenance_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_command_admission_emit.rs`, plus
  command-envelope, command-dispatch, command-body-canonicalization, and
  command-handler emit helpers into
  `seed-kernel/src/agent_protocol_recovery_command_envelope_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_command_dispatch_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_command_body_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_command_handler_emit.rs`, plus
  status-read, rollback-preview, rollback-apply, and disable/restart/
  load-target emit helpers into
  `seed-kernel/src/agent_protocol_recovery_status_handler_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_rollback_preview_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_rollback_apply_emit.rs`, and
  `seed-kernel/src/agent_protocol_recovery_target_binding_emit.rs`, plus
  memory-write, durable-write, service-inventory side-effect, and
  command-effect emit helpers into
  `seed-kernel/src/agent_protocol_recovery_memory_write_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_durable_write_emit.rs`,
  `seed-kernel/src/agent_protocol_recovery_service_inventory_effect_emit.rs`,
  and `seed-kernel/src/agent_protocol_recovery_command_effect_emit.rs`, plus
  recovery load-binding emit helpers into
  `seed-kernel/src/agent_protocol_recovery_load_binding_emit.rs`
- `seed-kernel/src/agent_protocol_recovery.rs` is now below the 10k-line
  threshold; continue future extraction only over stable ownership boundaries
  such as remaining protocol retained-chain helpers or further splitting the
  focused command evaluator modules
- preserve every public method name, schema id, boundary id, denial reason,
  canonical hash line, event-log binding, and shadow-smoke expectation exactly
  except for file/module ownership
- keep the refactor commit behavior-neutral and prove it with
  `build-seed-kernel.ps1 -Profile release`, `cargo fmt --all -- --check`,
  `git diff --check`, workspace Cargo tests, secret scan, and
  `vm-harness\shadow-vm-smoke.ps1`
- when running `vm-harness\shadow-vm-smoke.ps1` through an agent tool on this
  Windows/QEMU setup, allow at least a 30-minute outer timeout, pass
  `-TimeoutSeconds 180` when the default per-command serial timeout is too
  tight, and check `release\vm-reports\shadow-*.json` plus the temp
  `serial.log` before treating a timeout as a protocol failure
- for fast iteration, `vm-harness\shadow-vm-smoke.ps1 -Profile quick` runs the
  real QEMU/serial path through boot, core read-only methods, provider-minimal
  export gates, denied module loading, denied recovery artifact loading, and
  RAM-only audit visibility while skipping the exhaustive negative matrix; the
  default `-Profile full` remains the complete evidence path
- for focused recovery iteration, `vm-harness\shadow-vm-smoke.ps1 -Profile recovery`
  skips the provider selftest, memory mutation, and normal module-loading matrix
  while preserving the real recovery lifeline and audit path; the harness writes
  serial commands in chunks by default to avoid spending minutes on host-side
  byte pacing for long diagnostic commands
- keep this refactor style as the standing development rule: when a stable
  boundary is visible, split it before files or docs become large, and make
  reports derive from observed execution rather than duplicated static lists

The verified foundation for that task is:

- `recovery.lifeline_command_execution_completion_denial_diagnostic` and
  `recovery.lifeline_command_execution_completion_denial_diagnostic_selftest`
  now retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_completion_denial.v0` hash
  reference over the retained execution-observation-denial reference while
  still accepting no raw command body, no lifeline command body, no lifeline
  command envelope, dispatching no command, executing no recovery behavior,
  observing no command result, exporting no provider context, and writing no
  memory, audit, rollback, completion, or service-inventory records. Dispatch
  now advances through completion-denial before ending at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_execution_observation_denial_diagnostic` and
  `recovery.lifeline_command_execution_observation_denial_diagnostic_selftest`
  now retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_observation_denial.v0` hash
  reference over the retained execution-audit-denial reference while still
  accepting no raw command body, no lifeline command body, no lifeline command
  envelope, dispatching no command, executing no recovery behavior, observing
  no command result, exporting no provider context, and writing no memory,
  audit, rollback, or service-inventory records. Dispatch now advances through
  audit-denial and observation-denial before ending at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_execution_audit_denial_diagnostic` and
  `recovery.lifeline_command_execution_audit_denial_diagnostic_selftest` now
  retain a local-only current-boot
  `raios.recovery_lifeline_command_execution_audit_denial.v0` hash reference
  over the retained execution-result-denial reference while still accepting no
  raw command body, no lifeline command body, no lifeline command envelope,
  dispatching no command, executing no recovery behavior, and writing no audit
  or rollback records. Dispatch now advances through result-denial and
  audit-denial before ending at explicit `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`.

- `recovery.lifeline_command_side_effect_gate_diagnostic` and
  `recovery.lifeline_command_side_effect_gate_diagnostic_selftest` now retain
  only local-only current-boot side-effect-gate hash references over the
  retained executor-capability-table reference and advance dispatch only to the
  missing execution-enablement boundary until that boundary is retained. They
  do not accept raw command bodies or lifeline envelopes, dispatch commands,
  execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_execution_enablement_diagnostic`,
  `recovery.lifeline_command_execution_preflight_diagnostic`,
  `recovery.lifeline_command_execution_intent_diagnostic`, and
  `recovery.lifeline_command_execution_commit_gate_diagnostic`, plus
  `recovery.lifeline_command_execution_result_denial_diagnostic`, and
  `recovery.lifeline_command_execution_audit_denial_diagnostic`, and
  `recovery.lifeline_command_execution_observation_denial_diagnostic`, and
  `recovery.lifeline_command_execution_completion_denial_diagnostic`, with their
  selftests, now retain local-only current-boot hash references over the
  previous execution stage. They validate the same command, target, authority,
  side-effect-gate, executor, and dispatch hashes, advance dispatch through the
  enablement, preflight, intent, commit-gate, result-denial, audit-denial, and
  observation-denial, and completion-denial facts, and still end at explicit
  `defined_non_executable` /
  `recovery_lifeline_command_dispatch_execution_disabled`. They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_executor_capability_table_diagnostic` and
  `recovery.lifeline_command_executor_capability_table_diagnostic_selftest`
  now retain only local-only current-boot executor-capability-table hash
  references over the retained command-dispatch behavior reference and advance
  dispatch only to the missing side-effect gate until that gate is retained.
  They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.lifeline_command_dispatch_behavior_diagnostic` and
  `recovery.lifeline_command_dispatch_behavior_diagnostic_selftest` now retain
  only local-only current-boot command-dispatch behavior hash references over
  the retained service-inventory side-effect boundary reference and advance
  dispatch only to the missing executor-capability table until that table is
  retained. They do not accept
  raw command bodies or lifeline envelopes, dispatch commands, execute
  lifeline status/rollback/module/load behavior, allocate service slots, mutate
  service inventory, write recovery memory, write durable audit/rollback state,
  or export provider context.
- `recovery.service_inventory_side_effect_boundary_diagnostic` and
  `recovery.service_inventory_side_effect_boundary_diagnostic_selftest` now
  retain only local-only current-boot service-inventory side-effect boundary
  hash references over the retained durable-audit/rollback write-authority
  reference and leave dispatch at explicit `defined_non_executable` behavior.
  They do not dispatch commands, allocate service slots, create service
  inventory records, change service inventory, write recovery memory, or write
  durable audit/rollback state.
- `recovery.durable_audit_rollback_write_authority_diagnostic` and
  `recovery.durable_audit_rollback_write_authority_diagnostic_selftest` now
  retain only local-only current-boot durable-audit/rollback write-authority
  hash references over the retained recovery-memory write-authority reference
  and leave dispatch stopped at missing service-inventory side-effect
  boundary. They do not dispatch commands, write durable audit/rollback state,
  write recovery memory, load artifacts, allocate service slots, or change
  service inventory.
- `recovery.memory_write_authority_diagnostic` and
  `recovery.memory_write_authority_diagnostic_selftest` now retain only
  local-only current-boot recovery-memory write-authority hash references over
  the retained load-artifact-by-hash target binding and leave dispatch stopped
  at missing durable-audit/rollback write authority. They do not dispatch
  commands, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- `recovery.load_artifact_by_hash_target_binding_diagnostic` and
  `recovery.load_artifact_by_hash_target_binding_diagnostic_selftest` now
  retain only local-only current-boot load-target hash references over the
  retained restart-last-good target binding and leave dispatch stopped at
  missing recovery-memory write authority. They do not dispatch commands, load
  artifacts, authorize recovery load, write recovery memory, create durable
  records, or change service inventory.
- `recovery.restart_last_good_target_binding_diagnostic` and
  `recovery.restart_last_good_target_binding_diagnostic_selftest` now retain
  only local-only current-boot restart-target hash references over the retained
  disable-module target binding and leave dispatch stopped at missing
  load-artifact-by-hash target binding. They do not dispatch commands, restart
  services, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- `recovery.disable_module_target_binding_diagnostic` and
  `recovery.disable_module_target_binding_diagnostic_selftest` now retain only
  local-only current-boot disable-target hash references over the retained
  rollback-apply authorization and leave dispatch stopped at missing
  restart-last-good target binding. They do not dispatch commands, disable
  modules, write recovery memory, create durable records, load artifacts, or
  change service inventory.
- Virtio has been removed from the Stage-0 kernel runtime and VM runner path.
- RDRAND seeds entropy in the bare-metal-style VM profile.
- Intel e1000 configures RX/TX rings, negotiates DHCP through smoltcp, and shows
  IP/gateway state in the framebuffer UI and serial console.
- a PS/2/i8042 polling fallback is present for first bare-metal keyboard tests
  on machines that expose legacy keyboard compatibility. It is only reported as
  ready after an acknowledge from the keyboard or real scancode input.
- a polled xHCI path now inventories USB controllers, resets directly attached
  root-port devices, enumerates HID boot keyboards, relative boot mice, and QEMU
  HID tablets, and feeds reports into the same input queue as PS/2.
- if no USB keyboard or pointer is active, the event loop periodically re-probes
  xHCI so a keyboard plugged in after boot can be picked up without rebooting.
- the USB status line includes `EV`, `ERR`, and `TCC` counters for HID input
  reports and interrupt transfer diagnostics on bare metal.
- the USB-XHCI row now includes keyboard and mouse readiness.
- the framebuffer renderer is double-buffered to avoid visible full-screen
  redraw flicker, and pointer movement now updates only a small cursor overlay
  instead of forcing a full UI redraw.
- the visible QEMU GTK profile uses `usb-tablet` absolute pointer input by
  default and hides the host cursor over the guest area without automatic mouse
  grab, so only the raiOS pointer is visible and remains aligned after focus
  changes; `-RelativeMouse` or `-MouseGrab` switches back to relative
  `usb-mouse` for stricter boot-mouse testing.
- the visible UI now defaults to a chat-first surface with `AI`, `CONSOLE`, and
  `SET` modes. Serial commands continue to use the command interpreter so VM
  harnesses remain deterministic.
- USB/PS2 keyboard input now carries special keys into the UI: Tab and arrow
  keys move a visible focus ring through the top navigation, chat/console input,
  and settings actions; Enter activates the focused item and Esc backs out of
  settings/API-key entry.
- the Surface Pro 4 internal WLAN target has been selected as Marvell AVASTAR
  88W8897 (`11ab:2b38`, Linux reference driver family `mwifiex_pcie`). Stage-0
  now probes PCI for that device and exposes it as a Wi-Fi status chip/log line,
  and the settings menu can record a RAM-only SSID and WPA passphrase. Firmware
  upload, WPA, and packet transport are not implemented yet.
- a VM-local `setup` menu now records a RAM-only OpenAI API key without echoing
  the key back into the serial log.
- `ask <text>` now stays inside the guest. In the normal build it requires the
  VM API key state and then fails closed at provider trust before API-key copy or
  HTTPS write unless a syntactically valid provider pin is configured. With
  `-EmbedOpenAiSpkiPinFromEnv`, the preferred verifier slice checks the OpenAI
  leaf SubjectPublicKeyInfo SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  `-EmbedOpenAiCertPinFromEnv`, the first positive verifier slice checks the
  OpenAI leaf certificate SHA-256 pin and the TLS 1.3 P-256 ECDSA
  `CertificateVerify` proof before copying the API key or writing HTTPS. With
  the explicit development override
  `-AllowUnverifiedOpenAiTls`, it resolves `api.openai.com`, opens TCP 443
  through e1000, performs TLS 1.3 with `NoVerify`, sends an HTTPS Responses API
  request, parses `output_text`, and prints the provider response.
- the provider trust state is visible in console/provider status,
  `system.snapshot.v0`, `problem.list`, and `service.inventory`; the default
  trust problem is `provider.tls_pin_config_missing`, while a successful pinned
  handshake reports `pinned_spki_verified` or `pinned_cert_verified`.
- `raios.agent.v0` exposes read-only serial methods for `system.describe`,
  `system.snapshot`, `system.capabilities`, `system.boot_log`, `device.graph`,
  `problem.list`, `service.inventory`, `memory.profile`, `memory.context`,
  `memory.query`, `memory.trace`, `memory.recent_events`, `audit.events`,
  `module.manifest_diagnostic`, `module.artifact_diagnostic`,
  `module.vm_report_diagnostic`, `module.grant_diagnostic`,
  `module.attestation_diagnostic`, `module.approval_diagnostic`,
  `module.audit_rollback_append_intent`, and their current selftest methods.
- mutating or potentially mutating methods such as `module.load_ephemeral`,
  `service.restart`, `config.apply`, `provider.configure`, and `wifi.configure`
  return structured `capability_denied` until manifest, VM test report, local
  attestation, computed capability grant, approval, audit, and rollback evidence
  exist.
- `module.load_ephemeral` and `service.load_ephemeral` now return
  `raios.module_load_gate.v0`, which reports the manifest, exact artifact, VM
  report, local attestation, computed grant, local approval, durable audit,
  rollback plan, loader, and ram-only service slot gates; the current state is
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.manifest_diagnostic` now exposes a read-only guest diagnostic for a
  module-manifest hash reference. It accepts no manifest JSON, artifact bytes,
  or unsigned service code and validates only the canonical
  `raios.module_manifest_reference.v0` hash over the manifest hash, requested
  capability, load mode, subject, resource, and current-boot scope.
- A valid `module.manifest_diagnostic` reference is retained as a local-only
  current-boot `raios.module_manifest_reference.v0` event binding. The retained
  record stores hashes only, appears through `retained_manifest_reference` and
  `audit.events`, and remains non-authorizing with
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained manifest reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `module_manifest: retained_hash_reference_only`,
  `retained_module_manifest_reference.state: present`, retained hashes, and
  `retained_module_manifest_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  manifest hashes as accepted evidence.
- `module.load_gate_manifest_selftest` now exposes local-only
  `raios.module_load_gate_manifest_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch retained manifest-reference candidates without mutating the
  global event log, accepting manifest JSON or artifact bytes, or loading
  artifacts.
- `module.artifact_diagnostic` now exposes a read-only guest diagnostic for a
  candidate-artifact hash reference. It accepts no manifest JSON, artifact
  bytes, or unsigned service code and validates the canonical
  `raios.module_candidate_artifact_reference.v0` hash over retained manifest and
  computed-grant event ids plus manifest, artifact, report, attestation, and
  grant hashes.
- A valid `module.artifact_diagnostic` reference is retained as a local-only
  current-boot `raios.module_candidate_artifact_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_candidate_artifact_reference` and `audit.events`, and remains
  non-authorizing with `artifact_loaded: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained candidate-artifact reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `candidate_artifact: retained_hash_reference_only`,
  retained artifact hashes, and
  `retained_candidate_artifact_reference_not_authorizing`; stale, substituted,
  wrong-schema, or hash-mismatched references are rejected without exposing their
  artifact hashes as accepted evidence.
- `module.load_gate_artifact_selftest` now exposes local-only
  `raios.module_load_gate_artifact_selftest.v0` test infrastructure for missing,
  accepted-current-boot-but-denied, stale/dropped, previous-boot-or-unretained,
  wrong-schema, substituted-record, hash-mismatch, manifest-reference mismatch,
  and computed-grant-reference mismatch candidates without mutating the global
  event log or loading artifacts.
- `module.vm_report_diagnostic` now exposes a read-only guest diagnostic for a
  VM-test-report hash reference. It accepts no manifest JSON, report JSON,
  artifact bytes, or unsigned service code and validates the canonical
  `raios.module_vm_test_report_reference.v0` hash over retained manifest,
  candidate-artifact, and computed-grant event ids plus manifest, artifact,
  report, attestation, grant, manifest-reference, and artifact-reference hashes.
- A valid `module.vm_report_diagnostic` reference is retained as a local-only
  current-boot `raios.module_vm_test_report_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_vm_test_report_reference` and `audit.events`, and remains
  non-authorizing with `accepts_vm_report_json: false`, `can_load_now: false`,
  and `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained VM-test-report reference before snapshotting it into the denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `vm_test_report: retained_hash_reference_only`,
  retained report hashes, and
  `retained_vm_test_report_reference_not_authorizing`; stale, substituted,
  wrong-schema, hash-mismatched, or manifest/artifact/grant-mismatched
  references are rejected without exposing their report hashes as accepted
  evidence.
- `module.load_gate_vm_report_selftest` now exposes local-only
  `raios.module_load_gate_vm_report_selftest.v0` test infrastructure for
  missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record,
  hash-mismatch, manifest-reference mismatch, artifact-reference mismatch,
  computed-grant-reference mismatch, and VM-report-hash mismatch candidates
  without mutating the global event log, accepting report JSON, or loading
  artifacts.
- host-side `registry-tools grant-diagnostic` now emits
  `raios.computed_capability_grant.v0` over an exact module manifest,
  candidate artifact, Shadow-VM report, local attestation, approval phrase,
  requested capability, subject, resource, and current-boot scope. The
  diagnostic is evidence only: valid tuples set
  `computed_candidate_present: true`, while `grants_capability`,
  `grants_load_now`,
  `authorizes_guest_load`, `can_load_now`, and `load_attempted` remain false.
- `registry-core` unit tests reject mismatched manifest/artifact/report/
  attestation hashes, non-empty manifest `granted_caps`, wrong approval
  phrases, and `limits.grants_load_now: true` attestations.
- `module.grant_diagnostic` now exposes a read-only guest diagnostic for a
  computed-grant hash reference. It accepts no artifact bytes and validates only
  the `raios.computed_capability_grant.canonical.v0` hash over manifest,
  artifact, VM-report, and local-attestation hashes. A valid reference sets
  `computed_candidate_present: true` but still keeps
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`,
  `can_load_now: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- A valid `module.grant_diagnostic` reference is now retained as a local-only
  current-boot `raios.module_computed_grant_reference.v0` event binding. The
  retained record stores hashes only, appears through `retained_reference` and
  `audit.events`, and remains non-authorizing with
  `grants_capability: false`, `grants_load_now: false`,
  `authorizes_guest_load: false`, `can_load_now: false`, and
  `load_attempted: false`.
- `module.attestation_diagnostic` now exposes
  `raios.module_local_attestation_reference_diagnostic.v0` as a read-only guest
  hash-reference diagnostic. It accepts only canonical hashes and current-boot
  event ids for retained manifest, artifact, VM-report, and computed-grant
  evidence; it accepts no local-attestation JSON, no artifact bytes, and keeps
  `authorizes_guest_load`, `can_load_now`, and `load_attempted` false.
- A valid `module.attestation_diagnostic` reference is retained as a local-only
  current-boot `raios.module_local_attestation_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_local_attestation_reference` and `audit.events`, and remains
  non-authorizing.
- `module.load_ephemeral` and `service.load_ephemeral` now snapshot the latest
  retained computed-grant reference into their denied
  `raios.module_load_gate.v0` response and event binding. With a retained
  reference, the gate reports
  `computed_capability_grant: retained_hash_reference_only`,
  `retained_computed_grant_reference.state: present`, retained hashes, and
  `retained_computed_grant_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained local-attestation reference before snapshotting it into their denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `local_attestation: retained_hash_reference_only`,
  `retained_local_attestation_reference.state: present`, retained hashes, and
  `retained_local_attestation_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_attestation_selftest` now exposes local-only
  `raios.module_load_gate_local_attestation_selftest.v0` test infrastructure
  for missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record,
  hash-mismatch, manifest-reference mismatch, artifact-reference mismatch,
  VM-report-reference mismatch, and computed-grant-reference mismatch
  candidates without mutating the global event log, accepting local-attestation
  JSON, accepting artifact bytes, loading artifacts, or mutating service
  inventory.
- `module.approval_diagnostic` now exposes
  `raios.module_local_approval_reference_diagnostic.v0` as a read-only guest
  hash-reference diagnostic. It accepts only canonical hashes and current-boot
  event ids for retained manifest, artifact, VM-report, computed-grant, and
  local-attestation evidence; it accepts no free-form local approval text,
  artifact bytes, or unsigned service code, and keeps `authorizes_guest_load`,
  `can_load_now`, and `load_attempted` false.
- A valid `module.approval_diagnostic` reference is retained as a local-only
  current-boot `raios.module_local_approval_reference.v0` event binding. The
  retained record stores hashes only, appears through
  `retained_local_approval_reference` and `audit.events`, and remains
  non-authorizing.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained local-approval reference before snapshotting it into their denied
  `raios.module_load_gate.v0` response and event binding. With a valid retained
  reference, the gate reports `local_approval: retained_hash_reference_only`,
  `retained_local_approval_reference.state: present`, retained approval hashes,
  and `retained_local_approval_reference_not_authorizing`, while still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_approval_selftest` now exposes local-only
  `raios.module_load_gate_local_approval_selftest.v0` test infrastructure for
  missing, accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, hash-mismatch,
  manifest-reference mismatch, artifact-reference mismatch, VM-report-reference
  mismatch, local-attestation-reference mismatch, and computed-grant-reference
  mismatch candidates without mutating the global event log, accepting approval
  text, accepting artifact bytes, loading artifacts, or mutating service
  inventory.
- `module.load_gate_retained_selftest` now exposes local-only
  `raios.module_load_gate_retained_reference_selftest.v0` test infrastructure
  for the denied load gate's retained-reference predicate. It covers missing,
  accepted-current-boot-but-denied, stale/dropped,
  previous-boot-or-unretained, wrong-schema, substituted-record, and
  hash-mismatch candidates without mutating the global event log, creating
  retained records, loading artifacts, or mutating service inventory.
- `module.load_ephemeral` and `service.load_ephemeral` also expose
  `raios.module_load_gate_audit_rollback_requirements.v0` in the denied
  response and event binding. The requirement schema names
  `raios.audit_record.v0`, `raios.rollback_plan.v0`, retained grant/reference
  ids, local approval, rollback-plan hash, and ram-only service-slot id as
  required but missing, with writes disabled and `can_load: false`.
- `module.load_gate_audit_rollback_selftest` now exposes local-only
  `raios.module_load_gate_audit_rollback_selftest.v0` test infrastructure for
  missing/stale/previous-boot/wrong-schema/substituted retained
  audit/rollback references, retained computed-grant/audit/rollback hash
  mismatches, retained service-slot mismatch, missing durable audit, missing
  rollback plan, matching-but-still-denied audit/rollback evidence,
  audit/rollback schema mismatches, retained grant hash mismatch,
  manifest/artifact/VM-report/local-attestation mismatches, local approval
  mismatch, rollback hash mismatch, rollback artifact mismatch, and rollback
  service-slot mismatch. It creates no retained references, durable audit
  records, rollback plans, service slots, event-log records, or loads.
- `registry-tools audit-rollback-diagnostic` now emits
  `raios.module_audit_rollback_diagnostic.v0` with nested
  `raios.audit_record.v0` and `raios.rollback_plan.v0` candidates. It binds
  the retained computed-grant hash, retained-reference event id, denied load
  event id, local approval, ram-only service-slot id, rollback plan hash,
  manifest, artifact, VM report, and local attestation while keeping
  `durable_audit_written: false`, `rollback_plan_installed: false`,
  `can_load_now: false`, and `load_attempted: false`.
- `registry-core` unit tests now reject audit/rollback candidate mismatches for
  retained grant hash, manifest, artifact, report, attestation, approval,
  rollback hash, and service-slot ids.
- `module.audit_rollback_diagnostic` now exposes
  `raios.module_audit_rollback_reference_diagnostic.v0` as a guest
  hash-reference diagnostic. It accepts only hashes and current-boot ids for the
  audit record, rollback plan, computed grant, retained reference, denied load
  event, manifest, artifact, VM report, local attestation, local approval,
  pre-load service inventory, cleanup actions, and ram-only service slot. A
  valid reference reports `valid_hash_reference_load_still_denied`, records one
  local-only current-boot `raios.module_audit_rollback_reference.v0` event
  binding, and still keeps `durable_audit_written`,
  `rollback_plan_installed`, `can_load_now`, and `load_attempted` false.
- `module.audit_rollback_diagnostic_selftest` covers absent, accepted
  current-boot, stale, previous-boot event id, wrong-schema, substituted audit
  hash, rollback hash mismatch, computed-grant hash mismatch, and invalid
  ram-only service-slot cases without creating audit records, rollback plans,
  service slots, retained references, or service inventory changes.
- `module.service_slot_diagnostic` now exposes
  `raios.module_service_slot_reservation_diagnostic.v0` as a guest
  hash-reference diagnostic. It binds a reservation hash to the retained
  computed-grant reference id, retained audit/rollback reference id, computed
  grant hash, audit-record hash, rollback-plan hash, pre-load service-inventory
  hash, and `ram_only:` slot id. A valid reference records only a local-only
  current-boot `raios.module_service_slot_reservation.v0` event binding and
  keeps `allocates_service_slot`, `creates_service_inventory_records`,
  `can_load_now`, and `load_attempted` false.
- `module.service_slot_diagnostic_selftest` covers absent, accepted
  current-boot, stale, mismatched reservation hash, and invalid `ram_only:`
  service-slot cases without mutating the global event log, creating retained
  reservation records, allocating slots, loading artifacts, or changing service
  inventory.
- `module.audit_rollback_availability` now exposes
  `raios.module_audit_rollback_availability.v0` as a read-only current-boot
  diagnostic over typed `raios.durable_audit_ledger.v0` and
  `raios.rollback_store.v0` availability facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it keeps
  `writes_enabled`, `creates_durable_audit_records`, `creates_rollback_plans`,
  `installs_rollback_plan`, `can_load_now`, and `load_attempted` false.
- `module.audit_rollback_availability_selftest` covers missing ledger/store,
  previous-boot, schema mismatch, missing provenance, and
  available-facts-but-policy-missing cases without mutating the global event
  log or creating durable records.
- `module.audit_rollback_write_policy` now exposes
  `raios.module_audit_rollback_write_policy.v0` as a read-only current-boot
  diagnostic over typed `raios.durable_audit_write_policy.v0` and
  `raios.rollback_install_policy.v0` policy facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it names the
  retained-evidence and availability bindings required before a future writer
  could append audit records or install rollback plans.
- `module.audit_rollback_write_policy_selftest` covers missing policy pairs,
  previous-boot, schema mismatch, missing provenance, retained-evidence binding
  gaps, availability binding gaps, and available-policy-but-writer-missing
  cases without mutating the global event log, enabling writes, or installing
  rollback plans.
- `module.audit_rollback_storage_layout` now exposes
  `raios.module_audit_rollback_storage_layout.v0` as a read-only current-boot
  diagnostic over typed `raios.persistence_device_inventory.v0` and
  `raios.audit_rollback_storage_layout.v0` facts. The live slice reports both
  facts as missing, local-only, non-durable, and non-authorizing; it separates
  persistence device identity, partition inventory, write-path availability,
  layout regions, append slots, and recovery-region separation from write or
  append authority.
- `module.audit_rollback_storage_layout_selftest` covers missing storage inputs,
  previous-boot, schema mismatch, missing provenance, missing stable device
  identity, missing partition inventory, layout-device binding gaps, missing
  audit-ledger and rollback-store layout regions, missing append slots, recovery
  boundary gaps, and available-but-still-non-authorizing storage layout cases
  without mutating the global event log, enabling writes, or installing rollback
  plans.
- `module.audit_rollback_append_engine` now exposes
  `raios.module_audit_rollback_append_engine.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_ledger_append_engine.v0` and
  `raios.rollback_store_transaction_engine.v0` facts. The live slice reports
  both facts as missing, local-only, non-durable, and non-authorizing; it
  consumes the storage-layout diagnostic as input while keeping append-only,
  flush, replay, write-policy binding, and recovery separation separate from
  write authority.
- `module.audit_rollback_append_engine_selftest` covers missing append-engine
  pairs, previous-boot, schema mismatch, missing provenance, storage-layout
  binding gaps, write-policy binding gaps, missing append-only/flush/replay
  support, recovery-boundary gaps, and available-but-still-non-authorizing
  append-engine cases without mutating the global event log, enabling writes,
  or installing rollback plans.
- `module.audit_rollback_append_contract` now exposes
  `raios.module_audit_rollback_append_contract.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_ledger_append_envelope.v0` and
  `raios.rollback_store_transaction_envelope.v0` append-contract facts. The
  live slice reports both facts as missing, local-only, non-durable, and
  non-authorizing; it consumes the storage-layout and append-engine diagnostics
  while exposing explicit stable-id/provenance bindings for storage-layout,
  append-engine, write-policy, and availability facts.
- `module.audit_rollback_append_contract_selftest` covers missing append
  envelope pairs, previous-boot, schema mismatch, missing provenance,
  provenance-binding gaps, write-policy binding and id gaps, availability
  binding and id gaps, storage-layout id gaps, append-engine id gaps,
  storage-layout gaps, and append-engine-missing cases without mutating the
  global event log, enabling writes, or installing rollback plans.
- `module.audit_rollback_append_payload_hash` now exposes
  `raios.module_audit_rollback_append_payload_hash.v0` as a read-only
  current-boot diagnostic over typed
  `raios.audit_record_append_payload_hash_envelope.v0` and
  `raios.rollback_transaction_append_payload_hash_envelope.v0` facts. The live
  slice derives envelope hashes only from retained audit/rollback candidates,
  retained service-slot reservation evidence, the pre-load write-request shape,
  and bound append-contract ids; because append-contract facts are still
  missing, the payload envelopes remain `missing`, local-only, non-durable, and
  non-authorizing.
- `module.audit_rollback_append_payload_hash_selftest` covers missing payload
  hash pairs, previous-boot, schema mismatch, missing provenance,
  retained-audit/rollback binding gaps, service-slot binding gaps, pre-load
  write-request binding gaps, append-contract id gaps, target-schema gaps,
  payload-hash gaps, retained-evidence missing, service-slot missing,
  append-contract missing, and available-but-still-non-authorizing payload-hash
  cases without mutating the global event log, enabling writes, or installing
  rollback plans.
- `module.audit_rollback_append_intent` now exposes
  `raios.module_audit_rollback_append_intent.v0` as a read-only current-boot
  diagnostic over typed `raios.audit_record_append_intent.v0` and
  `raios.rollback_transaction_append_intent.v0` facts. The live slice reports
  both facts as missing, local-only, non-durable, and non-authorizing; it
  consumes the bound append-contract facts and the append payload-hash envelope
  readiness while naming explicit
  append-contract, append-engine, storage-layout, write-policy, availability,
  payload-hash, and provenance bindings for future append requests.
- `module.audit_rollback_append_intent_selftest` covers missing append-intent
  pairs, previous-boot, schema mismatch, missing provenance,
  provenance-binding gaps, append-contract binding gaps, append-engine id gaps,
  storage-layout id gaps, write-policy id gaps, availability id gaps,
  payload-hash gaps, append-contract-missing cases, payload-envelope-missing
  cases, and
  available-but-still-non-authorizing append-intent cases without mutating the
  global event log, enabling writes, or installing rollback plans.
- `module.audit_rollback_write_boundary` now exposes
  `raios.module_audit_rollback_write_boundary.v0` as a read-only guest
  diagnostic over the retained manifest, candidate-artifact, VM-report,
  computed-grant, local-attestation, local-approval, audit/rollback, and
  service-slot reservation chain plus the audit/rollback availability,
  durable-write policy, storage-layout, append-engine through append-contract,
  append-contract facts, append payload-hash envelopes, and append-intent facts.
  It emits a typed
  `raios.module_pre_load_audit_rollback_write_request.v0` plus
  `raios.module_audit_rollback_write_denial_evidence.v0`, keeps
  `writes_enabled`, `creates_durable_audit_records`, `creates_rollback_plans`,
  `installs_rollback_plan`, `loads_artifact`, and `loads_recovery_artifact`
  false, and reports explicit `durable_audit_write_missing`,
  `rollback_install_missing`, `storage_layout_missing`, and
  `append_engine_missing` gates.
- `module.audit_rollback_write_boundary_selftest` covers missing, stale,
  substituted, previous-boot, wrong-schema, mismatched hash, service-slot
  mismatch, recovery-artifact separation, one-sided missing availability,
  available-facts-but-policy-missing, rollback-policy-missing,
  append-contract-missing, rollback-transaction-missing, append-intent-missing,
  payload-envelope-missing, writer-unimplemented, and accepted-current-boot-but-denied cases without
  mutating the global event log, creating durable records, installing rollback
  plans, or loading artifacts.
- `module.load_ephemeral` and `service.load_ephemeral` now also validate the
  latest retained audit/rollback reference before snapshotting it into the
  denied `raios.module_load_gate.v0` response and event binding. The live
  predicate checks that the retained reference binds the latest retained
  computed-grant reference, a prior denied load event, canonical computed-grant,
  rollback-plan, and audit-record hashes, and a valid `ram_only:` service-slot
  id. With a valid retained reference, the gate reports
  `durable_audit_record: retained_hash_reference_only_not_durable`,
  `rollback_plan: retained_hash_reference_only_not_installed`,
  `retained_audit_rollback_reference.state: present`, retained audit/rollback
  hashes, `durable_audit_write_missing`, and `rollback_install_missing`, while
  still keeping
  `can_load: false`, `service_inventory_change: none`, and
  `load_attempted: false`. A retained reference that points at a wrong-schema
  event or mismatched hashes is reported as `rejected_retained_reference`, and
  its audit/rollback hashes are not exposed as accepted gate evidence.
- `module.load_ephemeral` and `service.load_ephemeral` now validate the latest
  retained `raios.module_service_slot_reservation.v0` before snapshotting it
  into the same denied gate. The live predicate checks the retained grant and
  audit/rollback event ids, referenced event variants, canonical reservation
  hash, computed-grant/audit/rollback hashes, pre-load service-inventory hash,
  and `ram_only:` slot id. A valid reservation reports
  `service_slot: retained_hash_reference_only_not_allocated`,
  `retained_service_slot_reservation.state: present`, and
  `service_slot_reservation_hash` while keeping
  `allocates_service_slot: false`, `can_load: false`,
  `service_inventory_change: none`, and
  `load_attempted: false`.
- `module.load_gate_service_slot_selftest` now exposes local-only
  `raios.module_load_gate_service_slot_selftest.v0` test infrastructure for
  missing, accepted-current-boot, stale/dropped, wrong-schema, substituted,
  computed-grant-hash, audit-hash, rollback-hash, inventory-hash,
  service-slot, and reservation-hash retained service-slot reservation cases.
  Rejected cases report `rejected_retained_reference` and keep accepted
  `service_slot_reservation_hash` evidence absent.
- `module.grant_diagnostic_selftest` covers absent, accepted-current-boot,
  stale previous-boot, mismatched manifest-hash, and wrong-policy computed
  grant references without loading artifacts or mutating service inventory.
- `vm-harness\shadow-vm-smoke.ps1` verifies the read-only agent protocol,
  provider trust problem visibility, static service inventory, and denied module
  load behavior, then writes a `raios.vm_test_report.v0` report.
- `memory.profile`, `memory.context`, `memory.query`, and `memory.trace` now
  expose a local read-only `current_boot` memory context slice. The
  `memory.context` result schema is `raios.agent_context.v0`, includes
  current-boot `context_event_id`/`audit_event_id` handles for the local read,
  and provider export is explicitly disabled.
- `memory.context provider_minimal` now emits a local-only
  `raios.provider_context_projection.v0` preview with explicit
  `public`/`local_only`/`secret` field classification, included and omitted
  field lists, deterministic `packet_evidence` hashes for the canonical packet
  plus exported and omitted field lists, a nested redacted
  `raios.agent_context.v0` packet, and `can_export: false` until positive
  provider trust and a distinct provider export audit binding exist.
- `provider.context_export provider_minimal` now exposes the first
  `raios.provider_context_export.v0` gate. It returns structured
  `capability_denied`, records `cap.provider.context_export` with risk
  `export`, reports `provider_write: not_attempted`, reports packet and
  field-list evidence bindings as present, keeps the positive provider request
  binding and export audit binding gates missing, and emits separate
  current-boot denial evidence as
  `raios.provider_request_binding_denial.v0` and
  `raios.provider_context_export_denial_audit.v0`.
- `event.log.v0` now carries structured `bindings` for those denial events:
  both include the canonical provider-minimal packet hash plus exported and
  omitted field-list hashes, and both explicitly report
  `satisfies_current_boot_export_gate: false`.
- The real OpenAI `ask` path now emits a local-only
  `OPENAI_PROVIDER_REQUEST_ENVELOPE` serial marker with schema
  `raios.provider_request_envelope.v0` after request id allocation and before
  DNS/TCP/TLS/API-key copy/HTTPS write. It records redacted request-body shape,
  body hash, envelope hash, trust snapshot, `provider_write: not_attempted`,
  and `context_attached_to_provider_body: false` without raw prompt text,
  `Content-Length`, API keys, or Authorization values.
- On the `pinned_spki_verified` direct OpenAI path, after TLS proof and matching
  request-body hash validation but before API-key copy or HTTPS write, Stage-0
  now records local-only positive
  `raios.provider_request_binding.v0` and
  `raios.provider_context_export_audit_binding.v0` events. They bind the exact
  request-body hash, request-envelope hash, provider-minimal packet hash,
  exported-field-list hash, and omitted-field-list hash. The request binding
  satisfies only `satisfies_request_binding_gate: true`; the export audit
  binding sets `positive_export_authorization: true`, but both retain
  `satisfies_current_boot_export_gate: false`,
  `automatic_context_injection: disabled`, and
  `context_attached_to_provider_body: false`.
- `provider.context_gate provider_minimal` now exposes a read-only
  `raios.provider_context_export_gate_state.v0` diagnostic over retained
  current-boot binding records. It can validate one matching positive request
  binding plus export-audit binding pair while keeping `can_export: false`.
- `provider.context_gate_selftest provider_minimal` exposes local-only test
  infrastructure over the same gate predicate. It does not mutate the global
  event log, create request envelopes, create positive binding records, or
  attempt provider writes. The Shadow VM smoke now covers stale/dropped event
  ids, previous-boot-or-unretained ids, denial-schema substitution,
  positive-record substitution, request/body/binding hash mismatches, context
  hash mismatches, and trust-bypass records.
- `provider.context_export provider_minimal` now consumes one valid retained
  positive binding pair for local gate evaluation only, records
  `raios.provider_context_binding_consumption.v0`, and still returns
  `capability_denied` because `automatic_context_injection` remains disabled.
  A second attempt against the same pair is rejected as
  `binding_already_consumed`.
- `provider.context_injection_gate provider_minimal` now exposes the separate
  `raios.provider_context_injection_gate.v0` diagnostic. It reports final
  authorization as missing, requires
  `raios.provider_context_injection_authorization.v0`, keeps
  `automatic_context_injection: disabled`, and reports
  `can_attach_context: false`.
- `provider.context_injection_gate_selftest provider_minimal` exposes local-only
  test infrastructure over the final injection predicate. It does not mutate the
  global event log, create real request envelopes, create positive binding
  records, create final authorization records, attempt provider writes, or
  attach context. The Shadow VM smoke now covers missing, stale/dropped,
  wrong-schema, substituted-positive-record, final body-hash mismatch, trust
  downgrade, and body-attachment-without-final-authorization cases.
- On positive pinned/WebPKI OpenAI request paths, Stage-0 now emits a local-only
  `OPENAI_PROVIDER_CONTEXT_INJECTION_GATE` marker after request/export binding
  evidence and before API-key copy or HTTPS write. The marker binds the request
  body hash, request-envelope hash, and provider-minimal context hashes while
  keeping provider write not attempted and body attachment false.
- `provider.context_export` still does not create a request envelope; the
  Shadow VM smoke checks that denied export cannot fake one.
- `memory.query` and `memory.trace` include
  `snapshot.current.provider_minimal` as the stable locator for the redacted
  current-status projection.
- `memory.recent_events` and `audit.events [limit]` expose a bounded RAM-only
  `event.log.v0` ring containing compact `audit.event.v0` records for agent
  protocol reads, known `capability_denied` outcomes, provider request-binding
  denials, provider export-denial audits with hash-valued denial bindings, and
  the `raios.module_load_gate.v0` denial binding.
- denied memory/module/service/config methods include current-boot `event_id`
  and `audit_event_id` handles, while all durable audit, persistence, policy
  mutation, redaction mutation, and rollback behavior remains denied.
- memory mutation methods (`memory.record_observation`,
  `memory.propose_policy`, `memory.supersede_fact`, `memory.redact`, and
  `memory.compact`) return structured `capability_denied` with missing audit and
  persistence evidence.
- `vm-harness\shadow-vm-smoke.ps1` now verifies memory-context schemas,
  context event ids, the local `provider_minimal` redaction projection, the
  provider-minimal packet/field-list hashes, the denied
  `provider.context_export` gate with hash bindings present, positive request
  and export audit bindings still missing, denial-audit records present but not
  satisfying export gates, provider writes still not attempted, memory
  query/trace, event log schemas, audit alias, memory mutation denials with
  event ids, the read-only `provider.context_gate` missing-binding state, the
  `provider.context_gate_selftest` negative predicate cases, the separate
  `provider.context_injection_gate` missing-final-authorization state, the
  `provider.context_injection_gate_selftest` negative final-authorization cases,
  the read-only module manifest, candidate-artifact, VM-report, computed-grant,
  local-attestation, and local-approval diagnostics and selftests, the module audit/rollback
  hash-reference diagnostics and selftests, retained
  `raios.module_manifest_reference.v0`,
  `raios.module_candidate_artifact_reference.v0`,
  `raios.module_vm_test_report_reference.v0`,
  `raios.module_computed_grant_reference.v0`,
  `raios.module_local_attestation_reference.v0`, and
  `raios.module_local_approval_reference.v0`, plus
  `raios.module_audit_rollback_reference.v0` event bindings, and the denied
  module load gate including retained manifest, retained artifact, retained
  VM-report, retained computed-grant, retained local-attestation, retained
  local-approval, plus retained audit/rollback reference state in the response and event-log binding, live
  wrong-schema retained audit/rollback rejection, plus negative
  manifest-reference, artifact-reference, VM-report-reference,
  retained-reference, retained local-attestation-reference, retained
  local-approval-reference, retained audit/rollback reference, and
  audit/rollback requirement selftests,
  service-slot reservation diagnostics and selftests, live denied load-gate
  visibility of valid retained service-slot reservation evidence, negative
  retained service-slot reservation gate selftests, and read-only
  audit/rollback availability, write-policy, storage-layout, append-engine,
  append-contract, append payload-hash, append-intent, plus write-boundary
  diagnostics/selftests, and the separate denied recovery artifact load
  boundary with typed missing recovery identity, trust, VM-test, approval,
  loader, and rollback evidence, plus read-only recovery loader and
  rollback-evidence hash-reference diagnostics, all six retained recovery
  evidence ids bound into `recovery.load_binding`, recovery lifeline request
  hash-reference diagnostics over the fully retained recovery chain, recovery
  lifeline protocol-state gap diagnostics over that request and its six
  evidence ids, recovery lifeline command-vocabulary envelope diagnostics, and
  recovery loader runtime-isolation plus rollback transaction-engine boundary
  diagnostics plus durable audit/rollback persistence boundary diagnostics and
  selftests, plus recovery memory-provenance boundary diagnostics and selftests
  over source record ids, source schema hashes, classification, authority,
  rollback-transaction binding, last-good checkpoint binding, recovery-only
  export profile, redaction state, replay window, and audit linkage facts, plus
  recovery lifeline command-admission diagnostics and selftests over status,
  rollback preview/apply, disable module, restart last-good, and load recovery
  artifact by hash admission requirements, plus recovery lifeline
  command-envelope reference diagnostics and selftests over allowed command ids,
  argument schemas, argument hashes, required capabilities, target locators,
  command-admission boundary ids, retained request hashes, and a valid retained
  status-command envelope reference that still dispatches no command, plus
  recovery lifeline command-dispatch denial diagnostics and selftests over
  missing body canonicalization, handler binding, status-read handling,
  rollback authorization, per-command target binding, recovery-memory write
  authority, durable audit/rollback write authority, and service-inventory
  side-effect facts, plus recovery lifeline command-body canonicalization
  diagnostics and selftests over the retained command-envelope reference,
  dispatch-denial boundary id, canonical command-body metadata hash/reference,
  local-only missing redaction/classification and handler-input linkage facts,
  and the still-non-executing dispatch boundary after body evidence is retained.
  Latest focused reports:
  `release\vm-reports\shadow-20260524-094611-25144.json` with 136/136 quick
  predicates, 13 executed commands, and `duration_ms: 16874`, and
  `release\vm-reports\shadow-20260524-101315-27892.json` with 2725/2725
  recovery predicates, 142 executed commands, and `duration_ms: 158371`.
  Both reports derive `commands` from observed serial execution. The recovery
  profile still exercises the same predicate/command count, but serial command
  echo no longer forces framebuffer redraws while long hash-reference commands
  are being received.
- `vm-harness\openai-direct-smoke.ps1 -ExpectPinMismatch` was run against a
  local image built with a fake API key and intentionally wrong SPKI pin. It
  verified the real request envelope marker appears on the `ask` path, omits raw
  prompt/Content-Length/Authorization values, then fails at pin mismatch before
  HTTPS request data is sent and without positive request/export audit binding
  markers.
- `vm-harness\openai-direct-smoke.ps1 -ExpectSpkiPinnedTrust` was run against a
  local image built with a fake API key and the current OpenAI SPKI pin. It
  verified the real request envelope marker, positive request binding marker,
  positive export audit binding marker, and blocked injection-gate marker appear
  before the HTTPS write path, that marker body/envelope/binding/context hashes
  match, that
  `provider.context_gate` validates the retained pair, that
  `provider.context_export` consumes it once for local gate evaluation, and that
  the second consumption attempt returns `binding_already_consumed`, while
  provider-minimal context remains unattached.
- the development serial relay and old host-framing path have been removed from
  the runtime path.
- the next trust milestone is WebPKI or broader certificate algorithm support
  once trust anchors, time, hostname checks, and chain handling are specified.

## Known Gaps

- Windows now has a minimal image repackaging path:
  `scripts\package-stage0.ps1` creates `release\raios-stage0.img` from
  `release\esp`.
- `scripts/package-stage0.sh` is Linux/WSL-oriented and expects `mkfs.fat`,
  `mmd`, and `mcopy`.
- Network failure/timeout states and packet counters are still minimal.
- Keyboard input uses a minimal US/Linux keycode mapping; no layout selection,
  modifier completeness, or text editing beyond Backspace exists yet.
- Bare-metal support is experimental. Minimal direct xHCI USB-HID boot keyboard,
  mouse, hub traversal, and a limited no-input USB hotplug rescan exist, but full
  detach/reconfigure handling and broad NIC coverage do not exist yet, so real
  hardware may still boot to the UI but lack input/network unless it matches the
  implemented paths.
- Wi-Fi support currently detects the Surface Pro 4 Marvell AVASTAR 88W8897
  target and stores RAM-only SSID/WPA configuration for the current boot. The
  next implementation step is a Marvell PCIe firmware-upload path before 802.11
  association or WPA2 can work.
- Bare-metal USB preparation scripts exist, but writing a USB disk is destructive
  and must be done with an explicit disk number and confirmation string.
- API key entry exists in the VM, but the key is RAM-only and not persisted in
  the default image. A local test image can embed the key explicitly, but must
  not be committed or shared.
- Stage-0 has verified DNS/TCP/TLS/HTTPS for `api.openai.com:443` behind the
  explicit unverified development override, the preferred SPKI pin verifier, and
  the legacy leaf-certificate pin verifier. SPKI pinning still depends on the
  leaf using the currently supported P-256 ECDSA `CertificateVerify` path;
  broader algorithm support or WebPKI remains a hardening step.
- The OpenAI JSON response parser is intentionally minimal and only extracts the
  first `output_text` string.
- QEMU TCP serial is single-client in practice; do not run two serial clients
  against the same port at the same time.
- No signed module runtime exists yet.

## Do Not Regress

- Do not rename `limine.conf` back to `limine.cfg`.
- Do not remove Limine request start/end markers.
- Do not link the kernel lower-half.
- Do not assume Linux packaging tools are available on this Windows host.
- Do not delete or overwrite `release/raios-stage0.img` unless the replacement
  has booted in QEMU.
