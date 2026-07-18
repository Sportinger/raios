# P3-DESIGN — ordered deletion slices

Baseline: P0 routes exactly **58,663 Rust lines / 68 files** to `RETIRE`. Because P1-A and P2 wave 1 will land before execution, each slice must re-check line counts and predicate names against then-current `main`; the files and ownership boundaries below remain authoritative.

## P3-1 — generic module audit/rollback write-boundary lattice

Capability justification: real M7 scoped append and persistence paths replace the diagnostic-only module write-boundary model.

- Write set:
  - Ten files below
  - `seed-kernel/src/agent_protocol.rs`
  - `vm-harness/shadow-vm-smoke-profile-full-module-audit-rollback.ps1`
  - `vm-harness/shadow-vm-smoke-profile-full-module-selftests.ps1`
  - Relevant composition references in `shadow-vm-smoke.ps1`
  - Active command references in `docs/DEBUGGING.md`
  - Status/dashboard entries
- Delete:
  - `agent_protocol_module_write_boundary.rs` — 25
  - `..._append_contract.rs` — 1,722
  - `..._append_engine.rs` — 808
  - `..._append_intent.rs` — 1,088
  - `..._append_payload_hash.rs` — 1,095
  - `..._availability.rs` — 442
  - `..._boundary.rs` — 1,854
  - `..._emit.rs` — 76
  - `..._storage_layout.rs` — 1,865
  - `..._write_policy.rs` — 600
- Dispatch entries to remove:
  - `module.audit_rollback_availability[ _selftest]`
  - `module.audit_rollback_write_policy[ _selftest]`
  - `module.audit_rollback_storage_layout[ _selftest]`
  - `module.audit_rollback_append_engine[ _selftest]`
  - `module.audit_rollback_append_contract[ _selftest]`
  - `module.audit_rollback_append_payload_hash[ _selftest]`
  - `module.audit_rollback_append_intent[ _selftest]`
  - `module.audit_rollback_write_boundary[ _selftest]`
  - Remove their aliases: `store_availability`, `policy`, `persistence_layout`, `append_engine_readiness`, `storage_contract`, `append_payload`, `append_request`, and `write_gate`, including `_selftest`.
- Harness predicates to retire:
  - Every predicate whose name begins with the exact method stems above in:
    - `shadow-vm-smoke-profile-full-module-audit-rollback.ps1`
    - `shadow-vm-smoke-profile-full-module-selftests.ps1`
  - Any matching quick-profile predicate for `append_contract` or `storage_layout`.
  - Do not retire predicates covering real M7 append/write/readback behavior.
- Estimated reduction: **9,575 lines**
- Verification: focused `module-audit-rollback` replacement profile proving the real M7 path, then `full`
- Risk: **high — storage/authority vocabulary**, although deletion grants nothing

## P3-2 — legacy recovery artifact/load evidence

Capability justification: signed M6 candidate identity, re-verification, real loader, and M8 load-by-hash now provide the executable evidence path.

- Delete:
  - `agent_protocol_recovery_artifact_reference.rs` — 3,579
  - `..._artifact_reference_emit.rs` — 1,098
  - `..._artifact_selftest_emit.rs` — 144
  - `..._artifact_types.rs` — 331
  - `..._load_binding.rs` — 961
  - `..._load_binding_emit.rs` — 298
  - `..._loader_runtime_emit.rs` — 220
  - `..._loader_runtime_eval.rs` — 660
  - `..._target_binding_emit.rs` — 624
- Dispatch entries to remove:
  - `recovery.identity_diagnostic[ _selftest]`
  - `recovery.trust_diagnostic[ _selftest]`
  - `recovery.vm_test_diagnostic[ _selftest]`
  - `recovery.local_approval_diagnostic[ _selftest]`
  - `recovery.loader_diagnostic[ _selftest]`
  - `recovery.rollback_evidence_diagnostic[ _selftest]`
  - `recovery.load_binding[ _selftest]`
  - Legacy `recovery.load_artifact` diagnostic/denial facade only
- Harness predicates to retire by profile:
  - `recovery-artifact-evidence`: all legacy `policy:recovery_load_*`, artifact identity/trust/VM-test/local-approval/loader/rollback-evidence, and load-binding predicates.
  - `recovery-execution-binding`:  
    `protocol:recovery_binding_identity_event_id_matches_retained`,  
    `protocol:recovery_binding_trust_event_id_matches_retained`,  
    `protocol:recovery_binding_vm_test_event_id_matches_retained`,  
    `protocol:recovery_binding_local_approval_event_id_matches_retained`,  
    `protocol:recovery_binding_loader_event_id_matches_retained`,  
    `protocol:recovery_binding_rollback_evidence_event_id_matches_retained`,  
    `protocol:recovery_load_after_binding_completion_event_id_matches_retained`.
  - Retire equivalent legacy predicates composed into `full-audit`.
- Keep:
  - `quick:m8d1_load_artifact_by_hash_denied`
  - `quick:m8d1_load_malformed_hash_denied`
  - Real M6 candidate signature/reverification predicates
- Estimated reduction: **7,915 lines**
- Verification: `recovery-artifact-evidence` rewritten around real M6/M8 behavior, then `m8-lifeline`
- Risk: **high — provider/artifact trust**

## P3-3 — legacy recovery command protocol and admission lattice

Capability justification: the frozen M8 lifeline table and dedicated dispatch implement the real command front door.

- Delete:
  - `agent_protocol_recovery_command_admission_emit.rs` — 337
  - `..._admission_eval.rs` — 1,125
  - `..._authorization_types.rs` — 26
  - `..._body_emit.rs` — 177
  - `..._dispatch_emit.rs` — 506
  - `..._dispatch_types.rs` — 268
  - `..._envelope_emit.rs` — 209
  - `..._command_eval.rs` — 3,215
  - `..._handler_emit.rs` — 130
  - `..._reference_eval.rs` — 3,001
  - `..._reference_selftests.rs` — 1,735
  - `..._constants.rs` — 63
  - `..._lifeline.rs` — 80
  - `..._lifeline_command_vocabulary_emit.rs` — 209
  - `..._lifeline_command_vocabulary_eval.rs` — 317
  - `..._lifeline_eval.rs` — 7
  - `..._lifeline_protocol_emit.rs` — 182
  - `..._lifeline_protocol_eval.rs` — 323
  - `..._lifeline_protocol_types.rs` — 84
  - `..._methods.rs` — 182
  - `..._status_handler_emit.rs` — 143
- Dispatch entries to remove:
  - `recovery.lifeline_request_diagnostic[ _selftest]`
  - `recovery.lifeline_protocol_diagnostic[ _selftest]`
  - `recovery.lifeline_command_vocabulary[ _selftest]`
  - `recovery.lifeline_command_admission[ _selftest]`
  - All legacy `recovery.lifeline_command_{envelope,dispatch,body_canonicalization,handler_binding}*_diagnostic[ _selftest]`
  - `recovery.lifeline_status_read_handler_diagnostic[ _selftest]`
- Harness predicates to retire:
  - Entire legacy predicate sets in:
    - `shadow-vm-smoke-profile-recovery-command-frontdoor.ps1`
    - `...-recovery-command-authority.ps1`
  - From `recovery-lifeline-foundation`:
    - `protocol:recovery_lifeline_protocol_request_event_id_matches_retained`
    - `protocol:recovery_lifeline_command_vocab_request_event_id_matches_retained`
    - `protocol:recovery_lifeline_command_admission_request_event_id_matches_retained`
  - From `recovery-execution-binding`:
    - `protocol:lifeline_status_envelope_mismatch_no_status_dispatch`
- Keep:
  - `quick:m8a1_lifeline_table`
  - `quick:m8a1_case_insensitive_denied`
  - Every `quick:m8a1_<mutating-command>_denied`
  - Real M8 parser/dispatcher malformed-input and fail-closed checks
- Estimated reduction: **12,319 lines**
- Verification: `m8-lifeline`
- Risk: **critical — recovery command admission/dispatch**

## P3-4 — legacy recovery effects and execution-denial chain

Capability justification: M8 executors and their real SAFE preflights replace the speculative execution-enablement chain.

- Delete:
  - `agent_protocol_recovery_command_effect_emit.rs` — 612
  - `..._effect_reference_eval.rs` — 2,550
  - `..._effect_types.rs` — 24
  - `..._execution.rs` — 2,899
  - `..._runtime_types.rs` — 1,121
  - `..._service_inventory_effect_emit.rs` — 215
- Dispatch entries to remove:
  - `recovery.service_inventory_side_effect_boundary_diagnostic[ _selftest]`
  - `recovery.lifeline_command_dispatch_behavior_diagnostic[ _selftest]`
  - `recovery.lifeline_command_executor_capability_table_diagnostic[ _selftest]`
  - `recovery.lifeline_command_side_effect_gate_diagnostic[ _selftest]`
  - All `recovery.lifeline_command_execution_{enablement,preflight,intent,commit_gate,result_denial,audit_denial,observation_denial,completion_denial}_diagnostic[ _selftest]`
  - Legacy `recovery.lifeline_status_execution_result_diagnostic`
- Harness predicates to retire:
  - Entire legacy predicate sets in:
    - `shadow-vm-smoke-profile-recovery-command-effects.ps1`
    - Legacy portions of `...-recovery-execution-binding.ps1`
    - Matching diagnostic predicates in `...-recovery-audit.ps1`
  - Explicit retained-event predicates retired:
    - `protocol:recovery_binding_completion_denial_event_id_matches_retained`
    - Any `protocol:recovery_*execution*_*matches_retained` sourced solely from the deleted diagnostic chain
- Keep:
  - `quick:m8b2_restart_last_good_safe_denied`
  - Real executor outcome, no-mutation, no-append, and authority-denial predicates
- Estimated reduction: **7,421 lines**
- Verification: `m8-lifeline`
- Risk: **critical — recovery execution and mutation containment**

## P3-5 — legacy recovery persistence/rollback/memory evidence

Capability justification: real M7 transaction persistence and M8 recovery consume actual durable records instead of speculative recovery evidence.

- Delete:
  - `agent_protocol_recovery_durable_write_emit.rs` — 211
  - `..._memory_provenance_emit.rs` — 252
  - `..._memory_provenance_eval.rs` — 1,690
  - `..._memory_write_emit.rs` — 198
  - `..._persistence_emit.rs` — 239
  - `..._persistence_eval.rs` — 1,297
  - `..._rollback_apply_emit.rs` — 182
  - `..._rollback_preview_emit.rs` — 153
  - `..._rollback_transaction_emit.rs` — 219
  - `..._rollback_transaction_eval.rs` — 930
- Dispatch entries to remove:
  - `recovery.rollback_preview_authorization_diagnostic[ _selftest]`
  - `recovery.rollback_apply_authorization_diagnostic[ _selftest]`
  - `recovery.memory_write_authority_diagnostic[ _selftest]`
  - `recovery.durable_audit_rollback_write_authority_diagnostic[ _selftest]`
  - Legacy diagnostic `recovery.memory_provenance[ _selftest]`
  - Legacy diagnostic `recovery.rollback_transaction_engine[ _selftest]`
  - Legacy diagnostic `recovery.durable_audit_rollback_persistence[ _selftest]`
- Harness predicates to retire:
  - From `recovery-lifeline-foundation`:
    - `protocol:recovery_rollback_transaction_engine_request_event_id_matches_retained`
    - `protocol:recovery_durable_audit_rollback_persistence_request_event_id_matches_retained`
    - `protocol:recovery_memory_provenance_request_event_id_matches_retained`
  - Corresponding diagnostic predicates in `recovery-audit`, `recovery-command-authority`, and `full-audit`.
- Keep:
  - `quick:m8c1_durable_last_good_missing_evidence`
  - Real M7 durable append/readback/reparse/rescan checks
  - Real M8 last-good lookup, rollback preview, and SAFE denial checks
- Estimated reduction: **5,371 lines**
- Verification: `memory-durable` plus `m8-lifeline`
- Risk: **critical — persistence, rollback, durable authority**

## P3-6 — recovery facade removal

Capability justification: after P3-2 through P3-5, no legacy recovery implementation remains behind the facade.

- Delete:
  - `seed-kernel/src/agent_protocol_recovery.rs` — 6,167
- Remove from `agent_protocol.rs`:
  - All remaining dispatch entries implemented by the deleted legacy facade.
  - The inventory’s **62 legacy recovery diagnostic methods** and **17 legacy execution diagnostic methods**.
- Do not remove real dispatch entries owned elsewhere:
  - `recovery.lifeline_table`
  - `recovery.snapshot`
  - `recovery.restart_last_good`
  - `recovery.rollback`
  - `recovery.disable_module`
  - `recovery.load_artifact_by_hash`
  - Their genuine selftests in `recovery_lifeline.rs`/durable-store code
- Harness retirements:
  - Delete now-empty legacy profile files and their includes:
    - `recovery-artifact-evidence`
    - `recovery-command-frontdoor`
    - `recovery-command-authority`
    - `recovery-command-effects`
    - `recovery-execution-binding`
    - `recovery-audit`
  - A profile may remain only if it has been rewritten to exercise the real M6–M8 runtime.
- Estimated reduction: **6,167 lines**
- Verification: `recovery`, then `full`
- Risk: **critical — central dispatch/recovery composition**

## P3-7 — superseded Hello descriptor/preflight and rollback evidence

Capability justification: the built-in Hello service remains a lifecycle integration artifact, while real M6/M7/M8 paths replace its duplicate descriptor, preflight, rollback-authority, and storage evidence machinery.

- Delete:
  - `hello_service/command_targets.rs` — 149
  - `hello_service/descriptor_identity.rs` — 78
  - `hello_service/preflight.rs` — 410
  - `hello_service/rollback_authority_gates.rs` — 4,610
  - `hello_service/rollback_bindings.rs` — 2
  - `hello_service/rollback_hashes_a.rs` — 2
  - `hello_service/rollback_hashes_b.rs` — 2
  - `hello_service/rollback_writer_bindings.rs` — 1,304
  - `hello_service/rollback_writer_gate.rs` — 2,458
  - `hello_service/storage_authority_gate.rs` — 878
  - `hello_service/storage_gate_hash.rs` — 2
- Dispatch removal:
  - `service.artifact_load_plan_preflight_selftest`
  - Hello-only legacy `recovery.rollback_inspect_source_reference_selftest`
  - Hello-only legacy `recovery.rollback_materialize_dry_run`
  - Remove `recovery.rollback_inspect` only if P2 has replaced it with the real recovery/store inspector.
  - Keep shared `module.load_ephemeral` and service lifecycle dispatch.
  - Keep `service.rollback_apply` only through the real M6/M7 implementation; remove the Hello denial-lattice handler.
- Harness predicates to retire:
  - Delete `shadow-vm-smoke-profile-hello-rollback-dry-run.ps1` and all its predicates.
  - Retire Hello-specific rollback-gate, append-intent, payload-envelope, storage-authority, dry-run sector-plan, media-write-authority, and durable-policy predicate names from `quick` and `m6d-rollback`.
  - Retire Hello preflight-selftest predicates from:
    - `full-module-load-gate`
    - `recovery-artifact-evidence`
- Estimated reduction: **9,895 lines**
- Verification: `m6c-promotion`, `m6d-rollback`, then `quick`
- Risk: **high — lifecycle integration plus rollback authority**

# Hello evidence that must remain

The following Hello roles are not superseded and must survive P3:

- Built-in, explicitly labeled test artifact used to exercise the real service registry.
- Current-boot load/start/stop/restart/drop lifecycle.
- `service.inventory` and `service.health` integration.
- Generation and running/stopped state transitions.
- RAM-only state counter and explicit state migration across accepted v1↔v2 hot swaps.
- Accepted hot-swap state preservation and rejected reset-state no-mutation behavior.
- Lifecycle and health RAM-audit events.
- Proof that external unsigned bytes, persistence, durable audit writes, broad mutation, and unearned capabilities remain denied.
- The thin root adapter:
  - `hello_service.rs`
  - `hello_service/runtime.rs`
  - `hello_service/state_machine.rs`
- P2-relocated surviving support:
  - `constants.rs`
  - `emitters.rs`
  - `hash_support.rs`
  - `lifecycle_binding.rs`
  - `records.rs`
  - `state_records.rs`

Everything else in `hello_service` is routed to P3-7 deletion.

# Cross-slice invariants

- No deletion may turn a denial into acceptance.
- Unknown methods, malformed arguments, missing evidence, missing signatures, absent durable media, absent policy, and absent authority must continue to fail closed.
- Real M6 signature verification, descriptor re-verification, candidate isolation, and import grants remain mandatory.
- Real M7 scoped write authority, append authorization, target-span validation, readback/reparse/rescan, transaction ordering, and rollback integrity remain mandatory.
- Real M8 SAFE-mode admission, immutable command table, executor capability table, no-mutation preflights, and recovery authority checks remain mandatory.
- Do not remove a shared dispatch entry merely because one retired predicate handler used the same method string.
- Historical status/report references remain historical; active runbooks and profile composition must be corrected.

Golden handling:

- P3-1 through P3-5 and P3-7 require named predicate-list changes but no emitted-shape regeneration for surviving methods.
- P3-6 requires full-profile and recovery-profile golden regeneration because central dispatch and complete legacy profile fragments disappear.
- Any slice that changes a surviving response shape has crossed into P4 and must stop.
- Storage, rollback, recovery, authority, provider-trust, descriptor-signing, and boot-touching slices retain their mandatory focused VM evidence regardless of deletion dominance.

# Predicate-retirement commit template

> Retire `<predicate-name>` because `<deleted legacy method/schema>` was superseded by `<real M6/M7/M8 capability and replacement predicate>`; fail-closed `<authority/denial invariant>` remains covered by `<focused-profile>:<replacement-predicate>`.

# Total estimated reduction

| Family | Lines |
|---|---:|
| Generic module write-boundary | 9,575 |
| Legacy recovery | 39,193 |
| Superseded Hello evidence | 9,895 |
| **Total** | **58,663** |

Read-only packet completed; no files changed, committed, pushed, or merged.