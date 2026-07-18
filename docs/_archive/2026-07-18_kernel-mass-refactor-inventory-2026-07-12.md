# Kernel Mass Refactor P0 Inventory (2026-07-12)

> **INTERRUPTED — packet stop condition triggered.** During final row validation,
> a concurrent lane added `seed-kernel/src/agent_protocol_project_install.rs`
> (783 lines / 28,240 bytes) and modified `agent_protocol.rs` (+18 lines) and
> `agent_protocol_system.rs` (+49 net lines). All three are inside the explicit
> `agent_protocol*` inventory family. Per the packet contract, work stopped
> immediately; the table below remains the completed pre-change census but is
> **not a final verified P0 handoff**. It was not rebased onto the moving W6 tree.

> **Orchestrator verification (2026-07-12):** five-row spot check
> (`loader_runtime` 10,156 ln / 487,849 B; `memory` 3,263 ln / 611,483 B;
> `load_gate_render` 6,768 ln; `recovery` 6,167 ln; `module.loader_runtime`
> dispatch entry in `agent_protocol.rs`) and all eight family subtotals match
> the orchestrator's independent pre-P0 measurements. The census is ACCEPTED
> as the P0 baseline for tree `a7eecd6`. The concurrent W6 additions
> (`agent_protocol_project_install.rs` and related files) extend the
> KEEP-IN-KERNEL `project*` family by definition and change no RETIRE or
> RELOCATE route. P1 implementation gates on the committed, green post-W6
> tree plus the fresh full+recovery baseline.

Packet: `REFACTOR-P0-INVENTORY`. This is a read-only routing map against
`a7eecd6` plus docs-only `bf8bf5f`. `git diff --name-only a7eecd6..HEAD` and the
working-tree diff were empty for every inventoried family at census time.
Unrelated concurrent changes existed in `raios-core` and ignored/untracked
release artifacts; they were not read as baseline evidence and were not touched.

## Census and method-mapping rules

- Scope is all 95 `seed-kernel/src/agent_protocol*.rs` files, the root
  `hello_service.rs` plus its 19 child modules, and all 6 `event_log*.rs` files:
  **121 rows, 140,938 lines, 6,784,018 bytes**.
- Line counts use PowerShell's `Get-Content <file>.Count`, which counts blank
  lines and agrees with the repository plan's known 10,156-line loader-runtime
  value. Bytes use `Get-Item <file>.Length`.
- The `AGENT_METHODS` body parsed as **307** one-line `method!`,
  `envelope_method!`, `pred_method!`, or `pred_envelope_method!` entries. A
  method is listed as direct only when its `MethodAction`/predicate handler
  resolves to that file (including explicit import aliases). "Indirect/support"
  means an exact canonical method literal or call-site dependency exists but the
  `MethodEntry` handler lives in a facade. Where neither is mechanically
  reliable, the cell says `UNCERTAIN`.
- Harness consumers are exact canonical-method/predicate searches in
  `shadow-vm-smoke-profile-*.ps1`. Names are profile-fragment names without the
  fixed prefix/suffix. `full` and `recovery` compose several of these fragments.
- `superseded?` asks whether the evidence role is covered by the real M6/M7
  promotion+persistence loop (and, for recovery behavior, completed M8). It does
  not mean the file may disappear without updating dispatch, harness, docs, or
  source attestation.

## Family routing and subtotals

| family | files | lines | bytes | family route | exceptions |
| --- | ---: | ---: | ---: | --- | --- |
| module_* | 29 | 50,611 | 2,201,488 | RELOCATE | 10 generic `module_write_boundary*` files RETIRE; M7 uses real scoped append evaluators instead |
| recovery* | 47 | 39,193 | 1,672,620 | RETIRE | none; this is the legacy diagnostic/evidence lattice, not the real M8 `recovery_lifeline.rs` runtime |
| memory | 1 | 3,263 | 611,483 | RELOCATE | none |
| provider | 1 | 3,002 | 106,796 | RELOCATE | none |
| project* | 6 | 1,571 | 50,152 | KEEP-IN-KERNEL | none; this is the active W5 real workspace/build/run loop |
| event_log | 6 | 15,064 | 669,410 | RELOCATE | retain a thin kernel RAM-ring/event-source adapter |
| hello_service | 20 | 19,156 | 1,091,142 | RETIRE | root adapter, `runtime.rs`, and `state_machine.rs` KEEP; 6 surviving pure/emit modules RELOCATE; explicit rows govern |
| other | 11 | 9,078 | 380,927 | KEEP-IN-KERNEL | P4 may later compact vocabulary; P0 does not speculate beyond the named P2 families |
| **total** | **121** | **140,938** | **6,784,018** |  |  |

The family route is the dominant program fate. Per-file exceptions below are
authoritative. In particular, P2 must not relocate any row marked RETIRE.

## Per-file inventory

| file | lines | bytes | live dispatch methods | harness consumers | classification | superseded? | route |
| --- | ---: | ---: | --- | --- | --- | --- | --- |
| `seed-kernel/src/agent_protocol_module_approval.rs` | 1,315 | 52,414 | module.approval_diagnostic, module.approval_diagnostic_selftest | full-audit, full-module-evidence | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_attestation.rs` | 1,326 | 51,778 | module.attestation_diagnostic, module.attestation_diagnostic_selftest | full-audit, full-module-evidence, m6c-promotion, m6d-rollback | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_audit.rs` | 1,089 | 40,954 | module.audit_rollback_diagnostic, module.audit_rollback_diagnostic_selftest | full-audit, full-module-evidence, full-module-selftests | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_grant.rs` | 985 | 34,609 | module.grant_diagnostic, module.grant_diagnostic_selftest | full-audit, full-module-audit-rollback, full-module-evidence, full-module-selftests, m6c-promotion, m6d-rollback | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate.rs` | 10 | 626 | UNCERTAIN (support-only) | UNCERTAIN | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_render.rs` | 6,768 | 322,324 | module.distribution_receiver_identity_load_preflight (indirect/support) | full-audit, full-module-load-gate, m12-distribution-provenance | emit-vocabulary | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_selftest.rs` | 1,204 | 48,332 | UNCERTAIN (support-only) | UNCERTAIN | selftest | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_selftest_emit.rs` | 708 | 26,122 | module.load_gate_approval_selftest, module.load_gate_artifact_selftest, module.load_gate_attestation_selftest, module.load_gate_audit_rollback_selftest, module.load_gate_loader_runtime_selftest, module.load_gate_manifest_selftest, module.load_gate_retained_selftest, module.load_gate_service_slot_selftest, module.load_gate_vm_report_selftest | full-module-selftests | selftest | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_selftest_eval.rs` | 1,480 | 57,993 | UNCERTAIN (support-only) | UNCERTAIN | selftest | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_load_gate_selftest_reference_cases.rs` | 1,252 | 50,090 | UNCERTAIN (support-only) | UNCERTAIN | selftest | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_loader_artifact_hash_binding.rs` | 864 | 35,294 | module.loader_artifact_hash_binding, module.loader_artifact_hash_binding_selftest | full-audit, full-module-evidence, full-module-load-gate, full-module-selftests | pure-logic | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_loader_fact.rs` | 1,227 | 51,530 | module.loader_address_space_boundary[+selftest], module.loader_audit_rollback_write_boundary_binding[+selftest], module.loader_capability_import_table[+selftest], module.loader_entrypoint_abi[+selftest], module.loader_health_state_hooks[+selftest], module.loader_memory_map_constraints[+selftest], module.loader_rollback_hooks[+selftest], module.loader_service_slot_binding[+selftest] | full-audit, full-module-evidence, full-module-load-gate, full-module-selftests | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_loader_identity.rs` | 850 | 33,871 | module.loader_identity, module.loader_identity_selftest | full-audit, full-module-evidence, full-module-load-gate, full-module-selftests | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_loader_runtime.rs` | 10,156 | 487,849 | module.loader_runtime, module.loader_runtime_selftest; consumes eight `module.loader_*` facts plus module.service_slot_allocator | full-audit, full-module-evidence, full-module-load-gate, full-module-selftests, m6c-promotion, m6d-rollback | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_reference.rs` | 2,341 | 86,118 | module.artifact_diagnostic[+selftest], module.manifest_diagnostic[+selftest], module.vm_report_diagnostic[+selftest] | full-audit, full-module-evidence, m6c-promotion, m6d-rollback | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_service_slot.rs` | 791 | 30,606 | module.service_slot_diagnostic, module.service_slot_diagnostic_selftest | full-audit, full-module-evidence, full-module-selftests, m6c-promotion, m6d-rollback | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_service_slot_allocator.rs` | 4,299 | 183,027 | module.service_slot_allocator, module.service_slot_allocator_selftest | full-module-evidence, full-module-load-gate, full-module-selftests | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_service_slot_allocator_projection.rs` | 1,227 | 58,482 | UNCERTAIN (support-only) | UNCERTAIN | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_types.rs` | 3,144 | 170,652 | 12 module loader/slot methods (indirect typed vocabulary; exact handler subset UNCERTAIN) | full-audit, full-module-evidence, full-module-load-gate, full-module-selftests, m6c-promotion, m6d-rollback | emit-vocabulary | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_module_write_boundary.rs` | 25 | 1,430 | UNCERTAIN (facade) | UNCERTAIN | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_append_contract.rs` | 1,722 | 69,150 | module.audit_rollback_append_contract, module.audit_rollback_append_contract_selftest | full-module-audit-rollback, full-module-selftests, quick | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_append_engine.rs` | 808 | 30,683 | module.audit_rollback_append_engine, module.audit_rollback_append_engine_selftest | full-module-audit-rollback, full-module-selftests | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_append_intent.rs` | 1,088 | 42,244 | module.audit_rollback_append_intent, module.audit_rollback_append_intent_selftest | full-module-audit-rollback, full-module-selftests | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_append_payload_hash.rs` | 1,095 | 46,063 | module.audit_rollback_append_payload_hash, module.audit_rollback_append_payload_hash_selftest | full-module-audit-rollback, full-module-selftests | pure-logic | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_availability.rs` | 442 | 16,179 | module.audit_rollback_availability, module.audit_rollback_availability_selftest | full-module-audit-rollback, full-module-selftests | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_boundary.rs` | 1,854 | 75,478 | module.audit_rollback_write_boundary, module.audit_rollback_write_boundary_selftest | full-module-audit-rollback, full-module-selftests, persistence | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_emit.rs` | 76 | 2,501 | module.audit_rollback_availability (indirect/support) | full-module-audit-rollback, full-module-selftests | emit-vocabulary | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_storage_layout.rs` | 1,865 | 72,629 | module.audit_rollback_storage_layout[+selftest], module.audit_rollback_append_contract (indirect) | full-module-audit-rollback, full-module-selftests, quick | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_module_write_boundary_write_policy.rs` | 600 | 22,460 | module.audit_rollback_write_policy, module.audit_rollback_write_policy_selftest | full-module-audit-rollback, full-module-selftests, persistence | mixed | YES (M7 scoped append) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery.rs` | 6,167 | 296,022 | 62 recovery diagnostic MethodEntry methods (direct facade; exact set parsed from table) | full-audit, recovery-artifact-evidence, recovery-audit, recovery-command-authority, recovery-command-effects, recovery-command-frontdoor, recovery-execution-binding, recovery-lifeline-foundation | mixed | YES (M6/M7+M8 real lifeline) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_artifact_reference.rs` | 3,579 | 143,298 | UNCERTAIN (support-only) | UNCERTAIN | mixed | YES (M6/M7+M8) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_artifact_reference_emit.rs` | 1,098 | 44,596 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6/M7+M8) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_artifact_selftest_emit.rs` | 144 | 5,007 | UNCERTAIN (support-only) | UNCERTAIN | selftest | YES (M6/M7+M8) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_artifact_types.rs` | 331 | 14,426 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M6/M7+M8) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_admission_emit.rs` | 337 | 12,860 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real admission) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_admission_eval.rs` | 1,125 | 41,263 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real admission) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_authorization_types.rs` | 26 | 1,927 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real authority) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_body_emit.rs` | 177 | 7,655 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_dispatch_emit.rs` | 506 | 19,873 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_dispatch_types.rs` | 268 | 12,480 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_effect_emit.rs` | 612 | 25,002 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real effects) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_effect_reference_eval.rs` | 2,550 | 109,886 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real effects) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_effect_types.rs` | 24 | 1,710 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real effects) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_envelope_emit.rs` | 209 | 8,231 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_eval.rs` | 3,215 | 138,839 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_handler_emit.rs` | 130 | 5,398 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real handlers) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_reference_eval.rs` | 3,001 | 117,862 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_command_reference_selftests.rs` | 1,735 | 64,721 | UNCERTAIN (support-only) | UNCERTAIN | selftest | YES (M8 integration profiles) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_constants.rs` | 63 | 4,697 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (legacy vocabulary) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_durable_write_emit.rs` | 211 | 8,404 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M7/M8 real durable writes) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_execution.rs` | 2,899 | 147,055 | 17 recovery.lifeline_command_execution_* diagnostic MethodEntry methods | full-audit, recovery-audit, recovery-command-authority, recovery-execution-binding | mixed | YES (M8 real execution) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline.rs` | 80 | 3,386 | six legacy recovery.lifeline.* command specs (indirect/support) | recovery-* fragments, exact subset UNCERTAIN | mixed | YES (real M8 table is outside this family) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_emit.rs` | 209 | 7,818 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 frozen real vocabulary) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_command_vocabulary_eval.rs` | 317 | 13,837 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 frozen real vocabulary) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_eval.rs` | 7 | 515 | UNCERTAIN (facade) | UNCERTAIN | pure-logic | YES (M8 real lifeline) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_emit.rs` | 182 | 7,953 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real lifeline) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_eval.rs` | 323 | 12,822 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real lifeline) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_lifeline_protocol_types.rs` | 84 | 3,426 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real lifeline) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_load_binding.rs` | 961 | 38,344 | recovery.load_binding[+selftest] (indirect via recovery facade) | recovery-artifact-evidence, recovery-lifeline-foundation | mixed | YES (M6 reverify + M8 load-by-hash) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_load_binding_emit.rs` | 298 | 19,805 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6 reverify + M8 load-by-hash) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_loader_runtime_emit.rs` | 220 | 9,369 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6 real loader) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_loader_runtime_eval.rs` | 660 | 26,446 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M6 real loader) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_memory_provenance_emit.rs` | 252 | 11,377 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M9 typed durable memory) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_memory_provenance_eval.rs` | 1,690 | 60,501 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M9 typed durable memory) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_memory_write_emit.rs` | 198 | 7,762 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 scoped durable audit) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_methods.rs` | 182 | 6,055 | 26 legacy recovery diagnostic parsers/specs (indirect/support) | recovery fragments; exact handler ownership is facade-bound | pure-logic | YES (M8 dedicated dispatch) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_persistence_emit.rs` | 239 | 10,650 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M7 persistence) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_persistence_eval.rs` | 1,297 | 49,395 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M7 persistence) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_rollback_apply_emit.rs` | 182 | 7,093 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6 rollback + M8 recovery) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_rollback_preview_emit.rs` | 153 | 6,270 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6 rollback + M8 recovery) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_rollback_transaction_emit.rs` | 219 | 9,220 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M7 transaction) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_rollback_transaction_eval.rs` | 930 | 37,198 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M7 transaction) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_runtime_types.rs` | 1,121 | 53,879 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | YES (M8 real runtime) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_service_inventory_effect_emit.rs` | 215 | 8,673 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M6/M8 real inventory effects) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_status_handler_emit.rs` | 143 | 5,846 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | YES (M8 real status) | RETIRE |
| `seed-kernel/src/agent_protocol_recovery_target_binding_emit.rs` | 624 | 23,768 | recovery.load_artifact (indirect denial source) | common, full-audit, m8-lifeline, quick, recovery-artifact-evidence, recovery-audit, recovery-command-authority, recovery-command-frontdoor, recovery-execution-binding, recovery-lifeline-foundation | emit-vocabulary | YES (M8 real target binding) | RETIRE |
| `seed-kernel/src/agent_protocol_memory.rs` | 3,263 | 611,483 | memory.context, memory.profile, memory.query, memory.recent_events, memory.trace | common, full-module-evidence, full-module-load-gate, full-provider-memory, hello-rollback-dry-run, m8-lifeline, memory-durable, quick, recovery-artifact-evidence, recovery-command-authority, recovery-command-effects, recovery-command-frontdoor, recovery-execution-binding, recovery-lifeline-foundation | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_provider.rs` | 3,002 | 106,796 | provider.context_export_authorized_selftest, provider.context_export_authorized_selftest_smuggle, provider.context_export_packet_selftest, provider.context_gate, provider.context_gate_selftest, provider.context_injection_gate, provider.context_injection_gate_selftest, provider.trust_honesty | common, full-provider-memory, memory-durable, provider-memory | mixed | NO | RELOCATE |
| `seed-kernel/src/agent_protocol_project.rs` | 258 | 8,080 | project.import_begin, project.import_chunk, project.import_commit, project.import_file_begin, project.import_file_finalize, project.inspect (via import aliases) | project-build, project-workspace | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_project_build.rs` | 408 | 13,354 | project.build_begin, project.build_commit, project.build_dependency_read, project.build_discard, project.build_receipts, project.build_run, project.build_source_read (via import aliases) | project-build | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_project_dependency.rs` | 199 | 6,877 | project.dependencies, project.dependency_begin, project.dependency_chunk, project.dependency_commit, project.dependency_discard, project.dependency_file_begin, project.dependency_file_finalize (via import aliases) | project-build, project-workspace | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_project_editor.rs` | 282 | 7,884 | project.edit_begin, project.edit_chunk, project.edit_commit, project.edit_delete, project.edit_diff, project.edit_discard, project.edit_file_begin, project.edit_file_finalize (via import aliases) | project-workspace | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_project_query.rs` | 141 | 5,129 | project.read, project.search (via import aliases) | project-workspace | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_project_run.rs` | 283 | 8,828 | project.run_approve, project.run_cancel, project.run_prepare, project.run_status; service.drop/health/start/stop and system.snapshot projections | project-app; common/full-provider-memory/genesis-ui/quick for projections | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/event_log.rs` | 7,141 | 282,628 | memory.recent_events read plus event recording for 44 dispatch methods (indirect; exact producer call graph UNCERTAIN) | common, full-audit, full-module-audit-rollback, full-module-evidence, full-module-load-gate, full-module-selftests, full-provider-memory, hello-rollback-dry-run, m6c-promotion, m6d-rollback, memory-durable, quick, recovery-* fragments | mixed | NO | RELOCATE |
| `seed-kernel/src/event_log_evidence.rs` | 2,544 | 111,585 | UNCERTAIN (support-only) | UNCERTAIN | emit-vocabulary | NO | RELOCATE |
| `seed-kernel/src/event_log_hello_selftest.rs` | 248 | 9,596 | recovery.rollback_inspect, recovery.rollback_inspect_source_reference_selftest (indirect) | hello-rollback-dry-run, quick, recovery-execution-binding | selftest | NO | RELOCATE |
| `seed-kernel/src/event_log_module_checks.rs` | 317 | 14,681 | UNCERTAIN (support-only) | UNCERTAIN | mixed | NO | RELOCATE |
| `seed-kernel/src/event_log_provider_selftest.rs` | 896 | 34,807 | provider.context_gate_selftest, provider.context_injection_gate_selftest (indirect) | full-provider-memory, provider-memory | selftest | NO | RELOCATE |
| `seed-kernel/src/event_log_types.rs` | 3,918 | 216,113 | UNCERTAIN (shared event vocabulary) | UNCERTAIN | emit-vocabulary | NO | RELOCATE |
| `seed-kernel/src/hello_service/command_targets.rs` | 149 | 5,160 | module.load_ephemeral, service.hot_swap, service.load_ephemeral (indirect) | full-audit, full-module-evidence, full-module-load-gate, hello-rollback-dry-run, m11-4-buffer-channel, m11-wasm-import-grant, m12-distribution-provenance, m6c-promotion, m6d-rollback, m8-lifeline, memory-durable, quick, recovery-artifact-evidence, recovery-execution-binding | pure-logic | YES (M6 real target/load path) | RETIRE |
| `seed-kernel/src/hello_service/constants.rs` | 400 | 28,882 | UNCERTAIN (shared hello vocabulary) | UNCERTAIN | mixed | PARTIAL | RELOCATE |
| `seed-kernel/src/hello_service/descriptor_identity.rs` | 78 | 2,862 | UNCERTAIN (descriptor support) | UNCERTAIN | pure-logic | YES (M6 signed candidate identity) | RETIRE |
| `seed-kernel/src/hello_service/emitters.rs` | 5,086 | 265,421 | recovery.rollback_inspect, service.rollback_apply (indirect), plus lifecycle responses via runtime | hello-rollback-dry-run, m6d-rollback, quick; lifecycle consumers UNCERTAIN | mixed | PARTIAL | RELOCATE |
| `seed-kernel/src/hello_service/hash_support.rs` | 105 | 2,735 | UNCERTAIN (support-only) | UNCERTAIN | pure-logic | NO | RELOCATE |
| `seed-kernel/src/hello_service/lifecycle_binding.rs` | 1,233 | 120,263 | hello lifecycle methods (indirect; exact subset UNCERTAIN) | full-module-evidence, hello-rollback-dry-run, quick | pure-logic | NO | RELOCATE |
| `seed-kernel/src/hello_service/preflight.rs` | 410 | 15,779 | service.artifact_load_plan_preflight_selftest (indirect) | full-module-load-gate, recovery-artifact-evidence | selftest | YES (M6 real preflight/reverify) | RETIRE |
| `seed-kernel/src/hello_service/records.rs` | 889 | 40,924 | UNCERTAIN (shared records) | UNCERTAIN | mixed | PARTIAL | RELOCATE |
| `seed-kernel/src/hello_service/rollback_authority_gates.rs` | 4,610 | 191,339 | recovery.rollback_inspect (indirect) | hello-rollback-dry-run, quick | mixed | YES (M7/M8 scoped durable authority) | RETIRE |
| `seed-kernel/src/hello_service/rollback_bindings.rs` | 2 | 76 | UNCERTAIN (include facade) | UNCERTAIN | emit-vocabulary | YES (M6/M7/M8) | RETIRE |
| `seed-kernel/src/hello_service/rollback_hashes_a.rs` | 2 | 72 | UNCERTAIN (include facade) | UNCERTAIN | pure-logic | YES (M6/M7/M8) | RETIRE |
| `seed-kernel/src/hello_service/rollback_hashes_b.rs` | 2 | 76 | UNCERTAIN (include facade) | UNCERTAIN | pure-logic | YES (M6/M7/M8) | RETIRE |
| `seed-kernel/src/hello_service/rollback_writer_bindings.rs` | 1,304 | 178,339 | recovery.rollback_inspect (indirect) | hello-rollback-dry-run, quick | mixed | YES (M7/M8 real writes/recovery) | RETIRE |
| `seed-kernel/src/hello_service/rollback_writer_gate.rs` | 2,458 | 88,330 | service.rollback_apply (indirect) | hello-rollback-dry-run, m6d-rollback, quick | mixed | YES (M6 rollback + M7 persistence) | RETIRE |
| `seed-kernel/src/hello_service/runtime.rs` | 633 | 21,930 | module.load_ephemeral; recovery.rollback_inspect/materialize/selftest; service descriptor/artifact/preflight selftests; service.drop/health/hot_swap/restart/rollback_apply/rollback_preview/start/stop | full-audit, full-module-evidence, full-module-load-gate, hello-rollback-dry-run, m11-4-buffer-channel, m11-wasm-import-grant, m12-distribution-provenance, m6c-promotion, m6d-rollback, m8-lifeline, memory-durable, project-app, quick, recovery-artifact-evidence, recovery-execution-binding, recovery-lifeline-foundation | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/hello_service/state_machine.rs` | 607 | 19,244 | service lifecycle/hot-swap/rollback methods (indirect; exact subset through runtime) | hello-rollback-dry-run, quick | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/hello_service/state_records.rs` | 248 | 8,926 | UNCERTAIN (state support) | UNCERTAIN | pure-logic | NO | RELOCATE |
| `seed-kernel/src/hello_service/storage_authority_gate.rs` | 878 | 98,827 | UNCERTAIN (rollback support) | UNCERTAIN | mixed | YES (M7 real scoped storage authority) | RETIRE |
| `seed-kernel/src/hello_service/storage_gate_hash.rs` | 2 | 74 | UNCERTAIN (include facade) | UNCERTAIN | pure-logic | YES (M7) | RETIRE |
| `seed-kernel/src/hello_service.rs` | 60 | 1,883 | hello methods are re-exported; exact handler set is in runtime.rs | all hello-consuming fragments | hardware-touching | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol.rs` | 961 | 81,535 | all 307 MethodEntry rows | all profile fragments through central dispatch | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_distribution.rs` | 337 | 12,297 | module.distribution_provenance_diagnostic, module.distribution_provenance_diagnostic_selftest | m12-distribution-provenance | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_honesty.rs` | 656 | 24,379 | system.honesty_report | common, m12-distribution-provenance | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_policy.rs` | 273 | 13,298 | generic denied MethodEntry actions (exact per-method ownership is central/UNCERTAIN) | most profiles through negative checks | pure-logic | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_program.rs` | 168 | 5,529 | program.submit_chunk, program.submit_finalize, program.workspace; system.snapshot projection | common, full-provider-memory, genesis-ui, quick | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_registry.rs` | 2,950 | 108,627 | module.distribution_receiver_identity_load_preflight, module.registry_selection_diagnostic[+selftest], module.submit_distribution_begin/begin_from_catalog/catalog_entry/chunk/finalize/receiver_identity/receiver_identity_evidence/receiver_identity_finalize | full-audit, full-module-load-gate, m12-distribution-provenance | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_support.rs` | 766 | 21,385 | UNCERTAIN (shared serializer/dispatch support) | UNCERTAIN | pure-logic | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_system.rs` | 1,407 | 50,258 | device.graph, persist.layout, problem.list, service.inventory, system.boot_log, system.capabilities, system.describe, system.snapshot | candidate-delivery, common, full-module-audit-rollback, full-provider-memory, genesis-ui, hello-rollback-dry-run, m12-distribution-provenance, m6c-promotion, m6d-rollback, persistence, project-app, quick, recovery-execution-binding | hardware-touching | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_time.rs` | 413 | 17,239 | system.cert_time_check_selftest, system.time_authority | common | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_ui.rs` | 189 | 7,404 | ui.personal_shell_proof | genesis-ui | mixed | NO | KEEP-IN-KERNEL |
| `seed-kernel/src/agent_protocol_wasm.rs` | 958 | 38,976 | module.submit_candidate_chunk, module.submit_candidate_finalize, wasm.bufecho_probe, wasm.certspki_probe, wasm.certwindow_probe, wasm.echo_probe, wasm.httphead_probe | candidate-delivery, m11-4-buffer-channel, m11-6-certwindow, m11-7-httphead, m11-8-certspki, m6c-promotion, m6d-rollback, quick | mixed | NO | KEEP-IN-KERNEL |

## Firm P2/P3 numeric targets

The committed post-W5 tree contains **207,047** Rust lines under
`seed-kernel/src` (197 `.rs` files), rather than the plan's pre-W5 206,481-line
snapshot. Targets therefore use the packet-pinned post-W5 count:

1. **RETIRE deletes 58,663 seed-kernel lines** (68 files): 39,193 legacy
   `recovery*`, 9,575 generic module write-boundary, and 9,895 superseded hello
   descriptor/preflight/rollback/storage evidence lines.
2. **RELOCATE moves 70,326 source lines to host-testable crates** (33 files):
   41,036 surviving module, 15,064 event-log, 3,263 memory, 3,002 provider, and
   7,961 surviving hello pure/emit lines. This is the source-line migration
   target; no copy may remain merely to preserve an old VM selftest.
3. **Expected `seed-kernel/src` after P2+P3: at most 84,058 Rust lines.** Formula:
   `207,047 - 58,663 - 70,326 + 6,000 thin-adapter allowance = 84,058`. The
   6,000-line allowance is a hard combined ceiling for kernel-facing adapters,
   hardware snapshots, RAM-ring plumbing, and one integration sanity path per
   relocated family. It is not permission to duplicate moved evaluators.

The resulting target is deliberately below the program's ~120k exit ceiling;
P4 compaction is additional reduction, not required to make these P2/P3 numbers
work.

## Hidden consumers and retirement risks

1. **Shadow VM is the primary executable consumer.** Every profile fragment
   named in the table sends a method or asserts an emitted field/predicate.
   `full` and `recovery` compose those fragments, so P3 must delete dispatch
   entries and predicates in the same slice as each retirement. A lower
   predicate count is expected and must be named, not silently tolerated.
2. **Hello source attestation is a hard build consumer.** `seed-kernel/build.rs`
   lines 10-31 enumerate all 20 hello source files in
   `HELLO_ARTIFACT_SOURCE_SET`, hash their ordered raw bytes, and generate signed
   descriptor/artifact constants. Removing or moving any hello row without
   updating this source set breaks the kernel build or the signed evidence.
3. **CI consumes the surfaces indirectly.** `.github/workflows/ci.yml` builds
   the whole kernel and runs `shadow-vm-smoke.ps1 -Profile quick`; there were no
   direct method/file-name hits under `.github/`, `scripts/`, `tools/`, or
   `ci/`, but quick-profile composition still makes CI an emit consumer.
4. **Operator docs are command consumers.** `docs/DEBUGGING.md` contains 48
   `module.audit_rollback_` references, recovery diagnostic command blocks, and
   34 `raios.ram_only_hello_service` references. Retiring those surfaces without
   deleting or marking those probes historical leaves false runbooks. ROADMAP,
   PROJECT_STATUS, archived roadmaps, and plan reviews also contain historical
   schema/method names; historical evidence should remain, while active command
   lists must change.
5. **Memory has a hidden legacy recovery dependency.** `agent_protocol_memory.rs`
   imports `recovery_lifeline_status_result_read_state()` from the legacy
   `agent_protocol_recovery_execution.rs`. Recovery retirement must rebind
   `memory.context` to the real M8 lifeline status before deleting that file.
6. **Event-log relocation is not retirement.** The RAM ring is still the live
   current-boot event source and many methods append to it. M9 durable records do
   not by themselves replace every event type or ordering rule. Keep the thin
   kernel lock/ring adapter and relocate only pure types, projection, hashing,
   reference checks, and selftests.
7. **M12 and W5 are active.** `agent_protocol_registry.rs`, load-gate receiver
   preflight, project workspace/build/run, and Wasm candidate methods are live
   product paths. They are explicitly KEEP/RELOCATE, never retirement collateral.

## Baseline report filenames

- Full: `shadow-20260712-025759-11164.json`
- Recovery: `shadow-20260711-024914-26232.json`
- Focused post-W5: `shadow-20260712-153736-17972.json`

The full and recovery files are the newest available pre-W5 checkpoint reports;
the focused file is the packet-pinned W5 close report.

## Out-of-scope observations

- The plan's measured 206,481 kernel lines is 566 lines below the pinned post-W5
  tree's 207,047; this inventory uses the latter and does not edit the plan.
- `agent_protocol_memory.rs` is 611,483 bytes and
  `agent_protocol_module_loader_runtime.rs` is 10,156 lines, confirming both P1
  emergency readability triggers.
- Concurrent unrelated changes were present in `raios-core/src/lib.rs`, an
  untracked `raios-core/src/project_install.rs`, and untracked release images,
  screenshots, and USB output. They were not inspected as refactor evidence and
  were not modified.

## Required command evidence

Final `git status --short` at the stop point (verbatim):

```text
 M raios-core/src/lib.rs
 M seed-kernel/src/agent_protocol.rs
 M seed-kernel/src/agent_protocol_system.rs
 M seed-kernel/src/artifact_store.rs
 M seed-kernel/src/main.rs
 M seed-kernel/src/shell_host/genesis.rs
 M seed-kernel/src/workspace_candidate_service.rs
 M vm-harness/shadow-vm-smoke-profile-project-app.ps1
 M vm-harness/shadow-vm-smoke-profile-project-build.ps1
 M vm-harness/shadow-vm-smoke.ps1
?? docs/plan-reviews/kernel-mass-refactor-inventory-2026-07-12.md
?? raios-core/src/project_install.rs
?? release/enum-console-shot.png
?? release/iommu-wifi-shot.png
?? release/raios-genesis-live-preview.img
?? release/raios-stage0-preview.img
?? release/set-wifi-scan-shot.png
?? release/ui-pill-detail-shot.png
?? release/usb-write-result.txt
?? seed-kernel/src/agent_protocol_project_install.rs
?? seed-kernel/src/project_app_autoload.rs
?? seed-kernel/src/project_install_store.rs
```

The required five-row final line/byte spot check was not run after this point:
the family-change stop condition takes precedence over continuing validation.
Before interruption, the mechanical validator confirmed 121 unique table rows
and exact line/byte agreement for all 121 pre-change rows. Its re-read against
the moving tree then detected the new 122nd file and size changes in the two
modified rows, which is what triggered the stop.
