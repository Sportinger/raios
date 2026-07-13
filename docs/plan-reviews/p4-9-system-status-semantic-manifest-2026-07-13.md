# P4-9a — system/status remainder semantic manifest

Static subtraction manifest for P4-9b. This packet changes no Rust, harness,
existing document, report, artifact, authority state, commit, or remote state.

Notation:

- `R` = legacy `body.result`; hand-written errors place their fields directly
  under `body`.
- `F` = v1 `facts`.
- `E[id]` = ordered v1 evidence record selected by `id`.
- `D` = v1 `decision`.
- `constant(...)` = invariant, not evaluator data.
- `retired(...)` = redundant legacy vocabulary intentionally removed.

The subtraction found no double-owned emitter. It did find a material scope
expansion: P4-9 is not only system/device/service/problem/status. Forty-seven
response-emission sites in `agent_protocol*.rs` are owned by no earlier
manifest. They represent 44 framing functions, 79 named direct methods, and a
generic-denial emitter serving 24 dispatch-table entries. Section 6 requires an
orchestrator decision before implementation groups active project/install
authority with the observational system/status close.

## 1. Response-path inventory

### Mechanical sweep and subtraction

Required sweep command and output, verbatim:

```text
> $out=rg -n --glob 'agent_protocol*.rs' 'RAIOS_AGENT_BEGIN|begin_response\(|emit_evidence_v1_response\(' seed-kernel/src; $out | ForEach-Object { ($_ -split ':')[0] } | Group-Object | Sort-Object Name | ForEach-Object {"$($_.Name)`t$($_.Count)"}; "TOTAL=$(@($out).Count)"
seed-kernel/src\agent_protocol_distribution.rs	2
seed-kernel/src\agent_protocol_honesty.rs	1
seed-kernel/src\agent_protocol_memory.rs	6
seed-kernel/src\agent_protocol_module_approval.rs	2
seed-kernel/src\agent_protocol_module_attestation.rs	2
seed-kernel/src\agent_protocol_module_audit.rs	2
seed-kernel/src\agent_protocol_module_grant.rs	2
seed-kernel/src\agent_protocol_module_load_gate_render.rs	1
seed-kernel/src\agent_protocol_module_load_gate_selftest_emit.rs	1
seed-kernel/src\agent_protocol_module_loader_artifact_hash_binding.rs	2
seed-kernel/src\agent_protocol_module_loader_fact.rs	2
seed-kernel/src\agent_protocol_module_loader_identity.rs	2
seed-kernel/src\agent_protocol_module_loader_runtime.rs	2
seed-kernel/src\agent_protocol_module_reference.rs	8
seed-kernel/src\agent_protocol_module_service_slot.rs	2
seed-kernel/src\agent_protocol_module_service_slot_allocator.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_append_contract.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_append_engine.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_append_intent.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_append_payload_hash.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_availability.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_boundary.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_storage_layout.rs	2
seed-kernel/src\agent_protocol_module_write_boundary_write_policy.rs	2
seed-kernel/src\agent_protocol_policy.rs	1
seed-kernel/src\agent_protocol_program.rs	3
seed-kernel/src\agent_protocol_project.rs	2
seed-kernel/src\agent_protocol_project_build.rs	2
seed-kernel/src\agent_protocol_project_dependency.rs	1
seed-kernel/src\agent_protocol_project_editor.rs	1
seed-kernel/src\agent_protocol_project_install.rs	1
seed-kernel/src\agent_protocol_project_query.rs	2
seed-kernel/src\agent_protocol_project_run.rs	1
seed-kernel/src\agent_protocol_provider.rs	10
seed-kernel/src\agent_protocol_registry.rs	12
seed-kernel/src\agent_protocol_support.rs	3
seed-kernel/src\agent_protocol_system.rs	8
seed-kernel/src\agent_protocol_time.rs	2
seed-kernel/src\agent_protocol_ui.rs	1
seed-kernel/src\agent_protocol_wasm.rs	7
TOTAL=112
```

The three `agent_protocol_support.rs` matches are the shared
`begin_response()` definition plus its BEGIN sites in `begin_response()` and
`begin_error()`. They are transport, not response emitters. Subtraction totals:

```text
raw sweep matches                       112
shared transport-helper matches          3
response-emission sites found           109
owned by P4-1 through P4-8               62
remaining for P4-9                       47
double-owned response emitters            0
```

### Excluded emitter -> owning manifest table

The table accounts for every one of the 62 excluded response-emission sites.
Method aliases and table-driven methods share their canonical emitter; a mere
mention of another family inside an embedded projection is not ownership.

| Excluded emitter/sites | Sites | Owning manifest | Boundary note |
|---|---:|---|---|
| `emit_evidence_v1_response` plus manifest/artifact/VM-report/attestation/approval/grant/audit/service-slot diagnostic and selftest callers | 18 | P4-1 module reference | Direct reference responses only |
| `emit_load_gate_v1` and table-driven load-gate selftest response | 2 | P4-2 load gate | Includes `service.load_ephemeral` alias; event copy is P4-4 |
| allocator, loader identity, artifact binding, loader-fact, and loader-runtime diagnostic/selftest callers | 10 | P4-3 loader/allocator | Direct responses; load-gate nesting remains P4-2 |
| `emit_recent_events` | 1 | P4-4 event evidence | Owns the entire `memory.recent_events`/`audit.events` response |
| `memory.profile`, `memory.context`, `memory.query`, `memory.trace`, generic memory-mutation denial | 5 | P4-5 memory | Provider projection internals remain P4-8 |
| Hello lifecycle emitters | 0 in this sweep | P4-6 hello lifecycle | They live under `hello_service/`, not `agent_protocol*.rs`; `service.inventory` is explicitly P4-9 |
| Eight write-boundary diagnostics plus eight selftests | 16 | P4-7 rollback/write boundary | Exact 16 BEGIN sites |
| provider trust/gates/selftests/export denial | 10 | P4-8 provider | Nine functions; authorized selftest has two branch-local BEGIN calls |
| **Total excluded** | **62** |  |  |

There is no STOP for double ownership. Apparent text overlaps resolve as
follows: P4-8 owns provider projection internals, not `system.snapshot`; P4-6
explicitly assigns `service.inventory` to P4-9; P4-5's locator strings
`service.inventory.current` and `problem.list.current` do not claim those
responses; P4-4 exclusively owns `memory.recent_events` despite references in
other manifests.

### Remaining emitters and exact current field order

All ordinary paths retain:

```text
RAIOS_AGENT_BEGIN <method>
{v,t,id,body:{method,result:{...}}}
RAIOS_AGENT_END <method>
```

The generic denial uses `t:error` and direct `body` fields. The lists below are
in source order. Conditional fields are called out; v1 must render nullable
values explicitly.

#### System/status family — 8 sites

Source: `seed-kernel/src/agent_protocol_system.rs`.

| Emitter | Line | Current ordered `R` fields |
|---|---:|---|
| `emit_describe` | 513 | `schema, os, protocol, methods, denied_methods` |
| `emit_snapshot` | 536 | `schema, os, status, details, provider, capabilities, problems` |
| `emit_capabilities` | 571 | `schema, capabilities[]` |
| `emit_boot_log` | 601 | `schema, source, lines[]` |
| `emit_persist_layout` | 632 | `schema, scope, classification, status, reason, source, source_port_index, controller_present, read_attempted, read_completed, read_only, write_attempted, write_dma_ext_called, writes_enabled, persistence_claimed, gpt_layout, data_layout` |
| `emit_device_graph` | 675 | `schema, devices[]` |
| `emit_problem_list` | 700 | `schema, problems[]` |
| `emit_service_inventory` | 711 | `schema, services[]` |

Nested order:

```text
os: name, product, stage
protocol: version, transport, provider_context_injection, mutation_policy
status: framebuffer, entropy, usb_xhci, wifi, network, input
details.<device>: state, detail
provider: selected, route, api_key_state, direct_phase, direct_endpoint,
 direct_model, trust_state, pin_kind, pin_id, pin_slot,
 pin_rotation_policy, pin_rotation_id, development_bypass,
 verifier_decision {schema, verifier_id, stage, outcome, reason}
capabilities[]: id, risk, granted, scope, summary
boot lines[]: index, text
devices[]: id, kind, state, detail
problems[]: id, severity, summary
static services[]: id, kind, health, replaceable, core_owned, last_error,
 capabilities
```

Dynamic `service.inventory` variants preserve their source order:

```text
workspace candidate:
 id, kind, health, replaceable, core_owned, last_error, scope, persistence,
 classification, trust_tier, durable_install_trust_tier,
 activation_authority, install_checked, install_phase, install_reason,
 install_boot_posture, durable_installed, auto_start, install_generation,
 install_head_commit_sha256, probation_install_commit_sha256,
 active_install_commit_sha256, last_good_install_commit_sha256,
 install_candidate_sha256, install_receipt_sha256,
 activation_attempt_persisted, activation_success_persisted,
 rollback_persisted, fallback_running, install_tombstone_written,
 entrypoint, granted_host_imports, host_import_count, phase, running,
 generation, run_count, last_run_outcome, last_return_value_i32,
 last_fuel_used, candidate_sha256, receipt_sha256,
 physical_approval_present, fuel_budget, memory_limit_bytes,
 instance_limit, memory_count_limit, table_limit, capabilities

personal shell:
 id, kind, health, replaceable, core_owned, last_error, scope, persistence,
 trust_tier, owner_sealed, artifact_id, entrypoint, capability_envelope,
 granted_host_imports, host_import_count, running, last_lifecycle_reason,
 capabilities

hello:
 id, kind, health, replaceable, core_owned, last_error, scope, persistence,
 artifact_id, version, artifact identity/content/reference IDs and hashes,
 signature envelopes, preflight ID/hash/status, activation ID/hash/status/active,
 slot ID, descriptor schema/id/source kind/locator/validated/hash/signature,
 source bindings, running, generation, state, hot_swap_probation, last_action,
 capabilities

echo:
 id, kind, health, replaceable, core_owned, last_error, scope, persistence,
 artifact_id, version, identity/artifact/preflight/activation/slot facts,
 descriptor facts, capability_envelope, granted_host_imports,
 host_import_count, entrypoint, running, generation, run_count,
 last_run_outcome, last_return_value_i32, last_fuel_used,
 last_log_line_emitted, last_action, capabilities

granted candidate:
 id, kind, health, replaceable, core_owned, last_error, scope, persistence,
 classification, artifact_id, artifact_kind, version, trust_tier,
 activation ID/hash/status/active, slot ID, capability_envelope,
 granted_host_imports, host_import_count, entrypoint, running, generation,
 run_count, last_run_outcome, last_return_value_i32, last_fuel_used,
 last_log_line_emitted, last_action, capabilities
```

The `persist.layout` child order remains owned by existing core record builders:
`gpt_layout_record()` then `data_layout_record()`. Their stored/layout hashes
are not response-vocabulary hashes.

#### Honesty and time — 3 sites

| Emitter | File:line | Current ordered `R` fields |
|---|---|---|
| `emit_system_honesty_report` | `agent_protocol_honesty.rs:61` | `schema, scope, classification, method, owner_sealed, trust_tier, read_only, durable_write, provider_write, transmission, state_change, capability_granted, provider_no_overclaim, time_no_overclaim, cert_time_no_overclaim, provider_export_no_overclaim, wasm_no_overclaim, external_no_overclaim, owner_key_no_overclaim, no_dishonest_overclaim, provider_trust, time_authority, cert_time_validation, provider_export, wasm_import_surface, external_acquisition, owner_key_provisioning` |
| `emit_system_time_authority` | `agent_protocol_time.rs:97` | `schema, decision_schema, decision_id, decision_marker, source, classification, scope, read_status, year, month, day, hour, minute, second, data_mode, hour_mode, century_source, trusted, source_verified, host_settable, timezone_validated, validates_cert_time, authorizes_provider_request, authorizes_provider_export, durable_write, capability_granted, provider_write, transmission, performed, status, reason, honest` |
| `emit_system_cert_time_check_selftest` | `agent_protocol_time.rs:158` | `schema, test_infrastructure, fixture_kind, real_cert_fixture, basis_source, read_status, now_source, now, trusted, source_verified, validates_cert_time, authorizes_provider_request, authorizes_provider_export, durable_write, capability_granted, provider_write, transmission, real_cert_probe, cases` |

Honesty child records preserve their current builder order: provider trust
decision/descriptor/trust/no-write facts; time source/read/trust/no-write
facts; cert-time posture; provider-export posture; Wasm import posture;
external-acquisition decision; owner-key entropy/hardware-binding/RAM-candidate
posture. Cert-time `now`, `not_before`, and `not_after` use
`year,month,day,hour,minute,second`; each case is `case, fixture_source,
basis_source, not_before, not_after, status, expected_status, passed` followed
by the distinct no-authority fields.

#### Candidate intake and Wasm probes — 7 sites

Source: `agent_protocol_wasm.rs`.

```text
module.submit_candidate_chunk (line 38):
 method, scope, classification, chunk_index, decoded_byte_len,
 pending_byte_len, pending_chunk_count, accepted, rejected,
 discarded_pending_delivery, reason, load_attempted, execution_attempted,
 authorizes_load, authorizes_execution, writes_persistent_state,
 external_delivery_channel, evidence_complete

module.submit_candidate_finalize (line 80):
 method, scope, classification, delivered_byte_len, delivered_chunk_count,
 byte_len, artifact_sha256, wasm_valid, retained_in_ram, rejected, reason,
 load_attempted, execution_attempted, authorizes_load,
 authorizes_execution, writes_persistent_state, external_delivery_channel,
 candidate, evidence_complete

wasm.echo_probe (line 127):
 schema, scope, classification, test_infrastructure, method, service_id,
 artifact_id, artifact_sha256, artifact_identity_descriptor_sha256,
 artifact_signature_envelope_sha256, validation_ok, capability_envelope,
 granted_host_imports, host_import_count, instantiation_ok, entrypoint,
 run_outcome, return_value_i32, fuel_budget, fuel_used, log_prefix,
 log_line_emitted, log_line, negative_probe, negative_module_imports,
 negative_validation_ok, negative_instantiation_ok, negative_link_error_kind,
 negative_missing_import_module, negative_missing_import_name,
 capability_boundary_held, hardening_case_count, hardening_passed_count,
 hardening_all_passed, hardening_cases, accepts_external_artifact_bytes,
 maps_executable_pages, writes_persistent_state, mutates_service_inventory,
 mutates_global_event_log, candidate_intake, evidence_complete

wasm.bufecho_probe (line 222):
 schema, scope, classification, method, service_id, input_len, input_sha256,
 captured_output_len, captured_output_sha256, run_outcome,
 authorized_import_count, linked_host_import_count,
 module_imports_within_authorized_list, audit_dedupe, audit_record_id,
 audit_reason, negative, evidence_complete

wasm.certwindow_probe (line 299):
 common probe identity/input/run/import/fuel/output fields,
 guest parse/status/window fields, core parse/status/window fields,
 guest_matches_core, output_bytes_match, guest_output_is_evidence_only,
 core_is_authority, policy_allows_beyond_env, owner_sealed, trust_tier,
 validates_cert_time, authorizes_provider_request,
 authorizes_provider_export, durable_write, capability_granted,
 malformed_case, negative, evidence_complete

wasm.httphead_probe (line 441):
 common probe fields, guest/core status/content-length/completion/chunked,
 guest_matches_core, output_bytes_match, evidence/authority posture,
 malformed_case, content_length_case, negative, evidence_complete

wasm.certspki_probe (line 617):
 common probe fields, guest/core parse/error/SPKI length/public-key hashes,
 guest_matches_core, output_bytes_match, evidence/authority posture,
 validates_provider_spki, malformed_case, negative, evidence_complete
```

Every `negative` object remains ordered
`module_imports_within_authorized_list, run_outcome, missing_import_module,
instantiation_ok, captured_output_len`. Probe results are observations; a guest
parser matching core never grants provider or storage authority.

#### Personal-shell proof — 1 site

`emit_personal_shell_proof()` at `agent_protocol_ui.rs:17` emits:

```text
method, schema, scope, classification, test_infrastructure, non_default,
activation_mode, activation_requested, activation_request_reason, service_id,
trust_tier, owner_sealed, artifact_sha256,
artifact_identity_descriptor_sha256, artifact_signature_evidence_sha256,
load_descriptor_sha256, descriptor_signature_evidence_sha256,
authorized_import_list_sha256, authorized_imports, artifact_validation_ok,
authorized_import_count, linked_host_import_count, fuel_budget,
normal, sanitized_input, frame_changed_after_sanitized_input,
malformed_frame, malformed_frame_rejected_atomically,
clipped_overdraw, clipped_overdraw_proved, guest_trap, guest_trap_rejected,
fuel_exhaustion, fuel_exhaustion_rejected,
missing_frame_submit_linker_denial, broader_import_denial,
generic_loader_used, accepts_external_artifact_bytes,
authorizes_external_artifact_intake, authorizes_arbitrary_shell_artifacts,
authorizes_persistent_install, writes_persistent_state,
authorizes_provider_access, provider_auto_load, authorizes_provider_export,
authorizes_secret_access, authorizes_secret_plaintext,
authorizes_network_access, authorizes_recovery_access,
authorizes_capability_decision, authorizes_raw_framebuffer_access,
authorizes_broader_mutation, persistent_service_install,
service_inventory_change, evidence_complete
```

Each proof case is ordered `accepted, instantiation_error_kind, run_outcome,
fuel_used, frame_sha256, clipped_overdraw`; authorized imports are ordered
`module,name`. The proof is test infrastructure, but a successful request also
queues a bounded current-boot activation. That queue result is an effect and
must not be mislabeled as a pure observation.

#### Program workspace — 3 sites

Source: `agent_protocol_program.rs`.

```text
program.submit_chunk (line 13):
 method, scope, classification, accepted, rejected, reason,
 decoded_byte_len, pending_byte_len, pending_chunk_count,
 discarded_pending_delivery, signing_attempted, load_attempted,
 execution_attempted, writes_persistent_state

program.submit_finalize / program.workspace (lines 44/51), shared snapshot:
 method, scope, classification, retention, status,
 [accepted, rejected, reason, attempted_byte_len only for finalize],
 present, revision, byte_len, program_sha256, source, provider_request_id,
 serial_chunk_count, pending_byte_len, pending_chunk_count,
 pending_provider_request_id, original_request_present,
 original_request_byte_len, provider_source_spec_present,
 provider_source_spec_byte_len, provider_source_spec_sha256,
 parent_program_sha256, root_program_sha256, lineage_depth,
 last_rejection_reason, last_rejection_attempted_byte_len,
 signing_attempted, load_attempted, execution_attempted,
 authorizes_load, authorizes_execution, writes_persistent_state
```

#### Project family — 10 framing sites, 42 methods

The ten shared framing functions and their exact method sets are:

| Framing function | File:line | Methods |
|---|---|---|
| `emit_operation` | `agent_protocol_project.rs:103` | `project.import_begin`, `import_file_begin`, `import_chunk`, `import_file_finalize`, `import_commit` |
| `emit_inspect` | `agent_protocol_project.rs:50` | `project.inspect` |
| query read/search | `agent_protocol_project_query.rs:13,56` | `project.read`, `project.search` |
| editor `emit` | `agent_protocol_project_editor.rs:73` | eight `project.edit_*` methods |
| dependency `emit` | `agent_protocol_project_dependency.rs:45` | six `project.dependency_*` methods plus `project.dependencies` |
| build `emit_build` | `agent_protocol_project_build.rs:55` | `build_begin`, `build_run`, `build_commit`, `build_discard`, `build_receipts` |
| build `emit_read` | `agent_protocol_project_build.rs:193` | `build_source_read`, `build_dependency_read` |
| run `emit_snapshot` | `agent_protocol_project_run.rs:92` | `run_prepare`, `run_status`, `run_cancel`, `run_approve` |
| install `emit` | `agent_protocol_project_install.rs:606` | install prepare/signature/status/approve, uninstall prepare/signature/approve, rollback status |

Current field order by shared shape:

```text
project import operation:
 method, scope, classification, retention, persistence_posture, qemu_only,
 physical_media_supported, physical_media_attempted, status, reason,
 accepted, rejected, project_id, file_count, total_byte_len, active_path,
 active_byte_len, expected_byte_len, parent_revision_sha256,
 revision_action, tree_sha256, revision_sha256, files,
 storage_write_attempted, writes_persistent_state, builder_attempted,
 build_authorized, provider_export_attempted, provider_export_authorized,
 install_attempted, install_authorized, load_attempted, load_authorized,
 execution_attempted, execution_authorized

project.inspect:
 common project posture, status, reason, present, project_id,
 parent_revision_sha256, revision_action, tree_sha256, revision_sha256,
 file_count, total_byte_len, files, then the same authority tail

project.read:
 method, scope, classification, persistence_posture, qemu_only,
 physical_media_supported, physical_media_attempted, status, reason,
 project_id, revision_sha256, path, file_classification, media_type,
 blob_sha256, file_byte_len, offset, requested_len, returned_len, eof,
 bytes_hex, then the read-only authority tail

project.search:
 same posture, status, reason, project_id, revision_sha256, query_sha256,
 query_byte_len, searched_file_count, limit, match_count, truncated,
 matches, then the read-only authority tail

editor:
 method, scope, classification, retention, persistence_posture, qemu_only,
 physical_media_supported, physical_media_attempted, status, reason,
 accepted, rejected, action, project_id, base_revision_sha256, file_count,
 total_byte_len, active_path, active_byte_len, expected_byte_len, diff_count,
 diff, proposed_tree_sha256, proposed_revision_sha256,
 parent_revision_sha256, tree_sha256, revision_sha256, files,
 then the authority tail

dependency:
 method, scope, classification, persistence_posture, qemu_only, source_kind,
 status, reason, accepted, rejected, project_id, project_revision_sha256,
 cargo_lock_sha256, name, version, origin, origin_verified, origin_evidence,
 license_expression, license_path, license_sha256, license_verified,
 license_evidence, file_count, chunk_count, active_path, active_byte_len,
 expected_byte_len, chunk_sha256, chunk_byte_len, chunk_persisted,
 bundle_visible, orphan_chunk_count, tree_sha256, bundle_sha256,
 build_script_present, files, bundle_count, bundles, then authority facts

build result:
 method, scope, classification, persistence_posture, qemu_only, source_kind,
 target, flags_contract_sha256, environment_contract_sha256,
 build_code_policy, toolchain_trust, pinned_toolchain_manifest_sha256,
 generic_filesystem_exposed, status, reason, accepted, rejected, project_id,
 project_revision_sha256, source_tree_sha256, cargo_lock_sha256,
 toolchain_manifest_sha256, input_manifest_sha256, source_file_count,
 dependency_bundle_count, dependency_chunk_count, source_files, dependencies,
 run_one, run_two, candidate_sha256, candidate_byte_len,
 candidate_wasm_valid, receipt_sha256, receipt_present, receipt_count,
 receipts, evidence_quality, host_build_attested, reproducible,
 independently_verified, then the no-authority tail

build read:
 same build posture, status, reason, accepted, rejected, project_id,
 project_revision_sha256, path, file_byte_len, blob_sha256, bundle_sha256,
 tree_sha256, whole_sha256, chunk_count, chunks, offset, requested_len,
 returned_len, eof, bytes_b64, then the no-authority tail

run:
 method, scope, classification, persistence_posture, status, reason,
 accepted, rejected, service_id, phase, health, project_id,
 project_revision_sha256, source_tree_sha256, cargo_lock_sha256,
 input_manifest_sha256, receipt_sha256, candidate_sha256,
 candidate_byte_len, import_list_observed, import_count, import_list_sha256,
 entrypoint, fuel_budget, memory_limit_bytes, instance_limit,
 memory_count_limit, table_limit, approval_challenge_sha256,
 physical_approval_present, approval_source, approval_sha256, generation,
 run_count, run_outcome, return_value_i32, return_value_i32_bits,
 fuel_used, trust_tier, serial_approval_authorized,
 writes_persistent_state, storage_write, authorizes_promotion,
 authorizes_install, authorizes_native_load, authorizes_persistence

install/uninstall/rollback status:
 method, scope, classification, persistence_posture, status, reason,
 accepted, rejected, service_id, phase, action_kind, signature_verified,
 action_signature_message_sha256, physical_approval_sha256,
 authority_key_sha256, trust_tier, generation, log_sequence,
 previous_commit_sha256, candidate_sha256, receipt_sha256,
 last_commit_sha256, candidate_blob_offset, candidate_blob_frame_len,
 last_reason, serial_approval_authorized, physical_pointer_required,
 writes_persistent_state
```

Nested project orders are stable: file records are `path, classification,
media_type, byte_len, blob_sha256`; search matches are `path, byte_offset,
match_len`; dependency chunks are `byte_len, sha256`; build run records are
`exit_code, exit_code_negative, stdout_sha256, stderr_sha256,
output_byte_len, output_sha256`; build receipts preserve their exact input and
candidate hashes before authority booleans.

#### Distribution/registry family — 14 sites, 15 methods

Sources: `agent_protocol_registry.rs` and `agent_protocol_distribution.rs`.
The sweep has 12 registry sites because two public wrappers and the shared
begin-response function are all matched; there are nine registry framing
functions. The two provenance functions are separate.

```text
submit catalog entry (registry.rs:535):
 method, scope, classification, source_id, entry_id, content_sha256,
 total_length, chunk_count, max_chunk_count, signature_byte_len,
 receiver_identity_retained, retained_in_catalog, accepted, rejected, reason,
 then load/execution/durable-write/authorization/network/owner/trust posture

submit receiver identity (583):
 method, scope, classification, source_id, entry_id, content_sha256,
 receiver_identity, retained_in_catalog, accepted, rejected, reason,
 metadata_is_non_authorizing, guest_signature_verification_performed,
 requires_m6_m7_reverify_for_load, then the same no-authority tail

receiver identity evidence/finalize shared response (763):
 method, scope, classification, source_id, entry_id, content_sha256,
 evidence_kind, evidence_sha256, decoded_byte_len, retained_part_count,
 receiver_identity, receiver_identity_complete, accepted, rejected, reason,
 metadata_is_non_authorizing, guest_signature_verification_performed,
 requires_m6_m7_reverify_for_load, then the same no-authority tail

receiver identity preflight (646):
 method, scope, classification, source_id, entry_id, status, reason,
 content_sha256, retained_part_count, receiver_identity,
 receiver_identity_retained, receiver_identity_complete,
 guest_signature_verification_performed, retained_candidate_sha256,
 retained_candidate_present, retained_candidate_wasm_valid,
 catalog_finalize_candidate_sha256,
 retained_candidate_matches_catalog_finalize, preflight_evaluated,
 accepted, rejected, missing_gate_count, four gate-satisfied booleans,
 three requires-* booleans, can_load_now, load_authorized,
 install_authorized, then the no-authority tail

distribution begin / begin_from_catalog shared response (843):
 method, scope, classification, delivery_channel, source_id, entry_id,
 content_sha256, total_length, chunk_count, max_chunk_count,
 signature_byte_len, receiver_identity, receiver_identity_retained,
 accepted, rejected, reason, then the no-authority tail

distribution chunk (897):
 method, scope, classification, delivery_channel, content_sha256,
 chunk_index, chunk_sha256, decoded_byte_len, pending_chunk_count,
 accepted, rejected, discarded_pending_delivery, reason,
 then the no-authority tail

distribution finalize (940):
 method, scope, classification, delivery_channel, source_id, entry_id,
 status, reason, content_sha256, total_length, declared_chunk_count,
 accepted_chunk_count, delivered_byte_len, selection, staged_candidate,
 retained_provenance, receiver_identity, receiver_identity_retained,
 staged_only_after_valid_selection, provenance_is_origin_evidence_only,
 requires_m6_reverify_for_load, then the no-authority tail

registry selection diagnostic (2393; fields built at 2745):
 method, scope, classification, parse_ok, requested_artifact_sha256,
 registry_entry_count, registry_capacity, status, reason, selection, entry_id,
 staged_candidate, retained_provenance, staged_only_after_valid_selection,
 recomputed_sha256_matches_selection, provenance_is_origin_evidence_only,
 requires_m6_reverify_for_load, then the no-authority tail

registry selection selftest (2401):
 method, scope, classification, passed, case_count, cases, read_only,
 durable_write, owner_sealed, trust_tier, install_authorized,
 load_authorized, execute_authorized, persist_authorized

provenance diagnostic (distribution.rs:42):
 method, schema, scope, classification, source_kind, artifact_sha256,
 retained_present, retained_wasm_valid, provenance_signature_present,
 provenance_verified, publisher_key_sha256, decision_schema, decision_id,
 decision_marker, status, reason, honest, load_authorized,
 install_authorized, owner_sealed, requires_m6_reverify_for_load,
 trust_tier, load_attempted, execution_attempted, authorizes_load,
 authorizes_execution, writes_persistent_state,
 live_load_projection_present, live_load_projection_can_load_now,
 durable_write, evidence_complete

provenance selftest (distribution.rs:117):
 method, scope, classification, passed, case_count, cases,
 decision_schema, decision_id, decision_marker, trust_tier, owner_sealed,
 read_only, durable_write
```

Selection, candidate, retained-provenance, receiver-identity, and selftest case
child records retain the field order of their existing typed `Value` builders.
Their canonical artifact/signature hashes are not response hashes.

#### Generic denial — 1 site, 24 dispatch entries

`emit_capability_denied()` at `agent_protocol_policy.rs:19` serves the current
`DeniedGeneric` fallback entries for module proposal/test/recovery/persist/
rollback, service fallback, config/provider/Wi-Fi configuration, drawing,
probing, download, and module-test methods. Current direct `body` order:

```text
method, event_id, audit_event_id, code, message, required
```

`required` is ordered `raios.module_manifest.v0`,
`raios.vm_test_report.v0`, `local_attestation.v0`,
`computed_capability_grant`, `local_approval`, `rollback_plan`. Service methods
may be intercepted by a concrete service before this fallback; that does not
change the fallback emitter's ownership.

## 2. Field-by-field mapping to the v1 envelope

Every response uses the fixed v1 order `schema,id,family,scope,classification,
source_method,event_id,facts,evidence,decision`. Observational calls bind
top-level `event_id:null`; provenance belongs on each evidence record's
`source_event_id`. No P4-9 renderer may reuse the dispatch `record_read()` ID as
an observational response event.

| Legacy field/group | V1 carrier | Classification | Same-source rule |
|---|---|---|---|
| `v,t,id,body.method` | legacy envelope retired; `source_method` and typed response `id` | local_only transport | Dispatch canonical method |
| legacy response `schema` | `constant(raios.evidence_response.v1)`; old schema moves to `F.legacy_record_schema` only when semantically useful | local_only | Static method table |
| OS/stage/protocol/method catalogs | `F.os`, `F.protocol`, `F.methods`, `F.denied_methods` | public for OS labels; local_only for method/denial catalog unless export policy explicitly allows | Existing constants/arrays |
| system status/details | `E[system_status].facts.status/details` | local_only | One `SystemSnapshot::collect()` result |
| provider object inside snapshot | `E[provider_posture].facts.*` | mixed source, response local_only | Exact `provider::snapshot()` accessors; do not rebuild from P4-8 projection |
| capability rows | `E[capability.<id>].facts.{risk,scope,summary,status_detail}` | local_only | Existing `CAPABILITIES` row; `granted` becomes evidence status, not emitter authority |
| boot-log source/lines | `F.source`, ordered `E[boot_line.<index>].facts.{index,text}` | local_only | Same `serial::log_snapshot()`; never provider-exportable by default |
| persist layout status/reason | `E[persist_layout].status/reason` and layout facts | local_only | Same `PersistLayoutEvidence`; validity must not be replaced by controller presence |
| device rows | ordered `E[device.<id>].facts.{kind,state,detail}` | local_only | Same `SystemSnapshot` status line |
| problem rows | ordered `E[problem.<id>].facts.{severity,summary}` | local_only | Same `system_problem_facts::collect()` order |
| service rows | ordered `E[service.<id>].facts.*` | local_only | Same static row or exact dynamic snapshot accessor; no mixed re-read |
| honesty aggregate booleans | `F.posture.*`; component results become ordered evidence | local_only | Same component decision/accessor; aggregate recomputed once from those projections |
| time clock components | `E[time_source].facts.*` | local_only | Same single `live_time_authority_honesty()` clock read |
| cert-time cases | `F.cases[].expected/actual` plus `F.safety` | local_only test infrastructure | Same parser/evaluator result |
| candidate/program/project/distribution request/result facts | family-local `F.request` plus ordered intake/storage/build/runtime evidence | local_only | Existing outcome/snapshot object captured once |
| hashes and byte counts | owning evidence `facts` | local_only unless existing field is explicitly public | Same hash/length accessor; no response rehash substitutes authority hash |
| retained provenance/event IDs | owning `E[id].source_event_id` | local_only | Actual retained record ID, never top-level observational `event_id` |
| `accepted/rejected/status/reason` observations | evidence status/reason; `D` only when an evaluator owns authority | local_only | Existing evaluator outcome, not method-name inference |
| repeated false `authorizes_*`, `*_attempted`, `writes_*`, mutation fields | distinct evidence/safety carrier where it is an observed safety assertion; otherwise retired into typed `D` | local_only | Preserve each distinct source; never collapse category-wide |
| generic denial `required[]` | ordered missing/unavailable evidence records and `D.blocked_by` | local_only | Existing order; evaluator must own the list |
| generic denial `code/message` | `D.outcome=denied`; message retired as prose redundancy; reason is first blocker | local_only | First `blocked_by` reason |

Family/decision mapping:

| Family | `F` | Ordered `E` | `D` |
|---|---|---|---|
| `system.describe` | OS/protocol/catalog facts | none unless catalog provenance is added | `observed/system_description_returned` |
| `system.snapshot` | snapshot identity | status, provider posture, capabilities, problems | `observed/system_snapshot_returned` |
| `system.capabilities` | catalog metadata | one item per capability in declared order | `observed/capability_catalog_returned`; a row's legacy `granted` is not this call granting authority |
| `system.boot_log` | source/window metadata | one item per line | `observed/boot_log_returned` |
| `persist.layout` | read-only query facts | controller/read, GPT, data-layout evidence | `observed/persist_layout_returned` |
| `device.graph` | graph metadata | devices in current order | `observed/device_graph_returned` |
| `problem.list` | collection metadata | problems in collector order | `observed/problem_list_returned` |
| `service.inventory` | inventory metadata | static and dynamic services in emission order | `observed/service_inventory_returned` |
| honesty/time/probes/selftests | request/fixture/safety facts | one component/case/probe boundary per evaluator order | `observed/*_returned` |
| generic denial | request facts | six existing required items in order | denied; `reason = D.blocked_by[0].reason`, `grants:[]`, `effects:[]` |
| active candidate/program/UI/project/distribution operations | request plus captured result | intake/storage/build/approval/activation/runtime evidence | **DECISION NEEDED**: observed is false for real effects; granted requires evaluator-created proof |

Denied ALWAYS renders `"grants": [], "effects": []`. No retained record,
successful parser, valid hash, signed provenance, project receipt, physical
preview, or guest/core agreement changes that rule. Positive effects may be
rendered only from an evaluator-created grant proof and a kernel apply result.

Distinct safety mapping is mandatory, not category-level:

```text
load_attempted -> E[load_attempt].facts.count/status
execution_attempted -> E[execution_attempt].facts.count/status
storage_write_attempted -> E[storage_write].facts.attempted
writes_persistent_state -> E[persistent_write].facts.performed
provider_export_attempted -> E[provider_export].facts.attempted
install_attempted -> E[install].facts.attempted
service_inventory mutation -> E[service_registry_mutation].facts.change
guest/core parser agreement -> E[guest_parser_crosscheck].facts.*
```

Two legacy assertions may share one v1 carrier only through an explicitly
named honest merge proving they read the same accessor. `accepted` versus
`present`, validity versus presence, attempted versus authorized, and RAM
retention versus durable persistence are never mergeable.

## 3. Constants and invariants

The following survive as `constant(...)`, not evaluator data:

- envelope schema `raios.evidence_response.v1`, typed response ID grammar, and
  field order;
- observational top-level `event_id:null`;
- family/source-method table entries and aliases;
- `scope`/`classification` values where the current method contract fixes
  them; dynamic stored-record classifications remain evidence data;
- OS name/product/stage and protocol transport labels;
- bounded method/capability catalog order;
- device IDs/kinds and static service descriptor labels;
- `persist.layout` is a read-only query; this does not make its observed
  controller/read results constant;
- test-infrastructure labels and fixture identities;
- bounded Wasm import names, entrypoint names, fuel/memory limits, and parser
  fixture bytes/hashes where compiled constants already own them;
- project build target, flags/environment contracts, build-code policy, and
  pinned toolchain identity; measured receipt/run/hash results remain evidence;
- distribution channel/source constants, registry capacity, and
  `requires_m6_reverify_for_load` policy only where the core evaluator already
  treats them as fixed inputs;
- generic-denial requested capability only after the method table resolves it;
- denied `grants:[]` and `effects:[]`;
- selftest no-effect counters only when the fixture does not perform that
  effect; real project/store writes must not be rewritten as constant zero.

Canonical record, descriptor, artifact, signature, GPT/data-layout, project
revision/tree/blob, dependency bundle/chunk, build receipt, candidate,
provenance, owner-key, and durable-store hash grammars survive byte-identically.
Response migration never regenerates them.

## 4. Harness predicate inventory

### Counting rule

Counts below inventory predicates that directly consume a remaining response
or wait for its END marker. Reused `common` assertions are listed once. The
project reports expand helper loops from the latest green profile reports;
therefore their counts are execution counts, not literal PowerShell call-site
counts. To avoid the P4-1 donor failure, this manifest conservatively marks all
non-framing direct assertions for regeneration. A leaf may later be retained
unchanged only after it is family-anchored, unique to its own response, and
proved to use the same accessor. Thus `survive=0` is intentional, not missing
accounting.

| Profile/fragment | survive | regenerate | framing | Total |
|---|---:|---:|---:|---:|
| common system/status/time/honesty | 0 | 41 | 8 | 49 |
| `candidate-delivery` | 0 | 14 | 3 | 17 |
| `quick` P4-9 additions | 0 | 60 | 12 | 72 |
| M11 buffer/certwindow/httphead/certspki fragments | 0 | 16 | 6 | 22 |
| `m12-distribution-provenance` P4-9 portion | 0 | 32 | 37 | 69 |
| `m6c-promotion` candidate/inventory portion | 0 | 1 | 4 | 5 |
| `m6d-rollback` candidate/inventory portion | 0 | 2 | 5 | 7 |
| `genesis-ui` | 0 | 43 | 24 | 67 |
| `project-app` | 0 | 4 | 16 | 20 |
| `project-build` | 0 | 137 | 106 | 243 |
| `project-install` | 0 | 64 | 13 | 77 |
| `project-workspace` | 0 | 383 | 214 | 597 |
| `persistence` direct `persist.layout` portion | 0 | 5 | 1 | 6 |
| **Total reviewed** | **0** | **802** | **449** | **1,251** |

The 1,251 reviewed predicates are **951 above** the P4 design estimate's upper
bound of 300 for the system/status remainder. This is not estimate noise: the
post-design project/distribution/program/Wasm families dominate the result.

Latest reports used only as static predicate inventories:

```text
quick: shadow-20260713-131346-5544.json
m12-distribution-provenance: shadow-20260713-133939-18836.json
m6c-promotion: shadow-20260713-133732-964.json
m6d-rollback: shadow-20260713-133835-18768.json
genesis-ui: shadow-20260712-025218-6208.json
project-workspace: shadow-20260712-135131-25884.json
project-build: shadow-20260712-145618-13408.json
project-app: shadow-20260712-153736-17972.json
project-install: shadow-20260712-171300-16808.json
```

No VM was run by this packet.

### Needle-count scan

Required literal needle-count command and output, verbatim:

```text
> foreach ($x in @('common','candidate-delivery','full-audit','full-module-audit-rollback','full-module-load-gate','full-provider-memory','genesis-ui','hello-rollback-dry-run','m11-4-buffer-channel','m11-6-certwindow','m11-7-httphead','m11-8-certspki','m12-distribution-provenance','m6c-promotion','m6d-rollback','persistence','project-app','project-build','project-install','project-workspace','quick')) { $f="vm-harness/shadow-vm-smoke-profile-$x.ps1"; $a=(Select-String -Path $f -Pattern '\bAssert-LogContains(?:Fields)?\b').Count; $p=(Select-String -Path $f -Pattern '\bAdd-Predicate\b').Count; $s=(Select-String -Path $f -Pattern '\bSend-AgentCommand\b').Count; Write-Output ("$x assert=$a add=$p send=$s total=$($a+$p+$s)") }
common assert=125 add=17 send=14 total=156
candidate-delivery assert=11 add=3 send=3 total=17
full-audit assert=373 add=0 send=0 total=373
full-module-audit-rollback assert=8 add=8 send=24 total=40
full-module-load-gate assert=264 add=7 send=2 total=273
full-provider-memory assert=130 add=0 send=12 total=142
genesis-ui assert=11 add=22 send=19 total=52
hello-rollback-dry-run assert=50 add=4 send=20 total=74
m11-4-buffer-channel assert=0 add=3 send=3 total=6
m11-6-certwindow assert=0 add=4 send=1 total=5
m11-7-httphead assert=0 add=5 send=1 total=6
m11-8-certspki assert=0 add=4 send=1 total=5
m12-distribution-provenance assert=10 add=1 send=31 total=42
m6c-promotion assert=0 add=17 send=15 total=32
m6d-rollback assert=0 add=25 send=18 total=43
persistence assert=4 add=33 send=4 total=41
project-app assert=4 add=0 send=4 total=8
project-build assert=1 add=2 send=1 total=4
project-install assert=8 add=1 send=2 total=11
project-workspace assert=3 add=2 send=1 total=6
quick assert=196 add=30 send=65 total=291
```

Literal call-site counts are not execution counts: project helpers and loops
expand into hundreds of report predicates, M12 expands chunk/evidence loops,
and quick generates 43 Wasm leaf needles from a table. P4-9-relevant direct
literal/runtime-generated needles are: common 27; candidate-delivery 11;
quick 45 (two direct generic-denial needles plus 43 generated Wasm leaves);
M12 4 response needles (six owner-key console needles excluded); M11 0;
project profiles' literal Assert counts are transport/host markers and their
response semantics are instead counted from expanded report predicates.

### Donor-removal and distinct-assertion audit

Every `Assert-LogContains` searches the whole serial log. The following are
non-unique and must regenerate with a response-family anchor or parsed response
path; none may simply survive:

- bare schemas, `status`, `reason`, `scope`, `classification`, `accepted`,
  `rejected`, `passed`, and `trust_tier`;
- `authorizes_*:false`, `*_attempted:false`, `writes_persistent_state:false`,
  `owner_sealed:false`, and `capability_granted:false`;
- candidate `artifact_sha256`, `wasm_valid`, `retained_in_ram`, and delivery
  channel leaves, which appear in registry, distribution, Wasm probe, and
  project-build responses;
- service IDs and inventory health/running leaves, which appear in lifecycle,
  event, and inventory responses;
- provider verifier and problem leaves, shared by snapshot, provider, memory,
  and honesty responses;
- project status/reason/posture leaves repeated across all project emitters.

Each distinct safety assertion keeps a distinguishing carrier. In particular:
load not attempted, execution not attempted, persistence not written, provider
export not attempted, install not attempted, service inventory unchanged,
storage write not attempted, and capability not granted are eight distinct
assertions. No `D.outcome`, empty `effects`, or category-level counter replaces
all eight.

`full-audit`, provider-memory, memory-durable, hello, and rollback fragments are
excluded from ownership but must be donor-scanned after each emitter deletion.
Their bare needles can currently pass through bytes donated by a P4-9 response.
Any exposed predicate is regenerated against its own family or retired by
name; it is never silently dropped from totals.

## 5. Deletion set and expected line delta

P4-9b may delete only response-vocabulary construction after same-source core
projections and host mappings exist. Acquisition, evaluation, locks, I/O,
stores, parsers, service state, and dispatch remain.

| Deletion candidate | Expected net delta |
|---|---:|
| raw system/status JSON and duplicated service/status helpers | -550 to -900 |
| honesty/time response field assembly | -120 to -260 |
| generic denial hand-written envelope/required list duplication | -20 to -45 |
| program/candidate/Wasm response wrappers and repeated no-effect fields | -180 to -400 |
| registry/distribution response wrappers and repeated authority tails | -220 to -450 |
| project response wrappers and repeated posture/no-authority tails | -250 to -550 |
| obsolete agent-protocol raw JSON support made unused by the final conversion | -80 to -220 |
| v1 family-local projections/tables and adapters added | +100 to +250 |
| **Expected net** | **-1,320 to -2,575** |

Planning center: approximately **-1,950 kernel lines**. This is below the
design row's optimistic -3.5k edge because program/project/registry/Wasm already
construct typed `Value` fields through `emit_record_fields`; replacing their
envelope does not justify deleting evaluator or acquisition code. No line
credit is claimed for moving real storage, install, runtime, parser, or device
behavior.

Exact deletion rules:

1. Delete legacy `v/t/id/body/result` construction and old family schemas.
2. Replace raw system/service/status assembly with family-local typed tables or
   projections using the existing record serializer; do not add another
   serializer or generic policy framework.
3. Delete repeated false authority/effect leaves only after each named harness
   assertion has a distinct v1 carrier or named honest retirement.
4. Preserve all canonical hash builders and stored bytes.
5. Keep BEGIN/END framing and serial backpressure unchanged.
6. Remove shared raw helpers only after `rg` proves no surviving caller,
   including Hello and non-agent markers.
7. Do not count a relocation as deletion unless the kernel adapter plus core
   projection is net smaller and ownership is clearer.

## 6. STOPs, risks, and DECISION NEEDED items

### STOPs

1. **Double ownership:** none found. If implementation discovers an earlier
   manifest claiming a direct emitter in the 47-site remainder, stop and name
   both manifests; do not resolve it locally.
2. **Authority reconstructed by an emitter:** project import/edit/dependency,
   build, run, install/uninstall, distribution intake, and candidate retention
   can perform real effects. If v1 `D` is inferred from `accepted`, method name,
   or emitted booleans instead of an evaluator-created proof plus apply result,
   stop.
3. **Storage/installation folded into an observational decision:** stop.
   `project.install_*`, project structured-store commits, dependency chunk
   persistence, and durable audit evidence are not read-only merely because the
   response is inspectable.
4. **Snapshot tearing:** stop if service inventory, system snapshot, honesty,
   project, or registry projections re-read mutable sources while building one
   response. Capture once, then project.
5. **Same-source substitution:** stop on validity/presence, attempted/performed,
   RAM/durable, accepted/present, or trust-state/descriptor-presence swaps.
6. **Decision collapse:** stop if distinct no-load/no-execute/no-write/no-export/
   no-install/no-mutation assertions share only a bare denied outcome or empty
   effects.
7. **Canonical hash churn:** stop if any artifact, descriptor, signature,
   project, receipt, GPT/layout, candidate, provenance, or stored-record hash
   changes solely because response vocabulary changed.

### DECISION NEEDED

1. **Scope split.** Choose one:
   - **A:** P4-9 remains the literal last remainder and is executed as several
     ownership-safe sub-slices: system/status; honesty/time; Wasm/program;
     project; distribution/registry; generic denial.
   - **B:** P4-9 closes only the design-row system/status family, and the 36
     post-design non-system framing functions receive explicitly named later
     manifests. They cannot be declared vocabulary-v1 complete at P4-9 close.

   The manifest inventory supports either; silently ignoring the 47-site
   subtraction does not.

2. **Positive project/install decision ownership.** Name the core evaluator and
   `GrantProof`/apply-result type for each real project/store/install effect.
   Until that exists, those responses cannot migrate honestly to granted v1
   decisions. Treating them as `observed` would hide effects; treating them as
   `granted` from emitter booleans would violate the fail-closed substrate.

3. **Classification.** Confirm field-level policy for method catalogs, boot-log
   text, device details, project paths/bytes, build stdout/stderr hashes,
   registry metadata, and owner-key posture. The enclosing responses should
   remain `local_only`; any public child must be explicitly classified and
   export-gated.

4. **`persist.layout` verification tier.** Its response is observational, but
   its facts sit on the persistence/storage trust boundary. Choose the focused
   `persistence` profile in addition to `quick`; do not rely on quick alone.

5. **Phase-close claim.** Because the remainder has 1,251 reviewed predicates,
   decide whether P4-9 closes with one full+recovery run after all sub-slices or
   whether active authority sub-slices also require their existing focused
   project/M12/persistence profiles before that close. The standing verification
   rules require the latter for real storage/authority boundaries.

### Risks

- `service.inventory` composes static rows plus personal-shell, Hello, echo,
  granted-candidate, and workspace snapshots. It is the largest cross-family
  donor and the easiest place to mix snapshots or duplicate P4-6/P4-8 policy.
- `system.snapshot` embeds provider posture from the live provider accessor;
  mapping it from the provider-minimal export projection would change source
  and classification semantics.
- `system.capabilities` currently reports descriptive `granted` booleans. They
  must not become grants made by the read call.
- boot-log lines may contain local operational detail. Keeping them
  `local_only` is mandatory; semantic/RAG consumers remain locators only.
- the generic denial's prose message names a stale six-item ceremony. V1 must
  preserve actual evaluator blockers, not freeze that prose as policy.
- project and distribution profile helpers expand loops; literal PowerShell
  counts understate runtime predicate churn by hundreds.
- deleting a broad emitter will expose unrelated whole-log needles. Run the
  donor scan after every sub-slice, not only at phase close.
- project/Wasm/distribution sources landed after the original P4 estimate.
  Forcing them into one ordinary `quick` slice would cross runtime, storage,
  install, provider-trust, and boot-risk boundaries contrary to the repository
  verification rules.

Headline finding: **zero double-owned emitters; 47 response-emission sites are
owned by no earlier manifest.** P4-9 exists to catch them, but implementation
must resolve the scope and positive-authority decisions above before claiming
the last vocabulary-v1 manifest is executable.

## Orchestrator rulings (2026-07-13, binding)

**D1 — OPTION B, with the carve-out named, not hidden.** P4-9 closes the
system/status remainder the design row actually meant (system.describe/snapshot/
capabilities/boot_log, persist.layout, device.graph, problem.list,
service.inventory, honesty/time/probes/selftests, and the generic denial).
The 36 post-design framing functions — project (workspace/build/install/app),
distribution/registry, Wasm/program, genesis-ui — are EXPLICITLY CARVED OUT and
get their own named manifests. They are NOT claimed as vocabulary-v1 complete at
the P4 phase close, and the phase-close report must say so in those words.

Why B and not A: those carved-out families PERFORM REAL EFFECTS (project
installs, structured-store commits, dependency-chunk persistence, distribution
intake). Under the fail-closed substrate a positive decision can only be
rendered from an evaluator-created GrantProof plus an apply result — and that
proof type does not exist for them yet (DECISION NEEDED #2 in this manifest is
the honest statement of that gap). Forcing them into P4-9 would leave exactly
two options, both unacceptable: render their real effects as `observed` (which
HIDES effects — the precise failure the vocabulary exists to prevent), or
synthesize `granted` from emitter booleans (which is the fail-closed violation
the substrate's pub(crate) constructors were built to make impossible). Option B
is the only path that does not require lying about what the machine did.

This also means the P4 line target (~139,281) was always computed WITHOUT these
families in scope — it was written before they existed. The phase close reports
the achieved number and the carve-out together; it does not quietly move the
goalposts in either direction.

**D2 — positive-authority substrate is a SEPARATE, LATER phase.** Do not build
GrantProof types for project/install/distribution inside P4. Their evaluators do
not exist; inventing them under vocabulary pressure is how a fail-closed system
acquires a back door. Name the gap, carve it out, close the phase honestly.

**D3 — classification.** Enclosing responses stay `local_only`. Boot-log text,
project paths, build stdout/stderr hashes, device details and owner-key posture
stay `local_only` (no public child without an explicit export gate). Method and
capability catalogs are `local_only` for now — a public catalog is a separate,
deliberate export decision, not a side effect of a vocabulary migration.

**D4 — persist.layout verifies on the focused `persistence` profile in addition
to `quick`.** It sits on the storage trust boundary; quick alone is not evidence.
(Note: shadow-vm-persistence-reboot.ps1 has a known pre-existing support-drift
crash — repair it before relying on it.)

**D5 — capability rows stay observations.** `system.capabilities` reporting
`granted: true` for a row must render as evidence status, never as this read
call granting anything. Any implementation that lets the catalog read produce a
grant is a STOP.

## P4-9b implementation notes (2026-07-13)

Converted in scope: `system.describe`, `system.snapshot`,
`system.capabilities`, `system.boot_log`, `persist.layout`, `device.graph`,
`problem.list`, `service.inventory`, `system.honesty_report`,
`system.time_authority`, `system.cert_time_check_selftest`, and the generic
`emit_capability_denied` path. All observational envelopes are `local_only`,
use `event_id:null`, and omit `grants`/`effects`; the generic denial alone keeps
empty arrays and the six evaluator blockers in their original order.

Predicate accounting (no predicate was silently dropped):

| Bucket | Predicate names | Replacement/carrier |
|---|---|---|
| REGENERATED byte-exact | `protocol:describe_schema`, `protocol:snapshot_schema`, `protocol:time_authority_schema`, `protocol:cert_time_check_selftest_schema`, `protocol:system_honesty_report_schema`, `protocol:capabilities_schema`, `protocol:service_inventory_schema`, `protocol:problem_list_schema` | family-scoped `raios.evidence_response.v1` envelope needles |
| REGENERATED byte-exact | `protocol:snapshot_provider_verifier_decision_outcome` | `protocol:snapshot_provider_verifier_decision_status` on the same captured provider verifier value |
| REGENERATED parsed | `protocol:time_authority_structural_ranges`, `protocol:time_authority_grants_nothing`, `protocol:cert_time_check_selftest_schema_fixture`, `protocol:cert_time_check_selftest_wide_status`, `protocol:cert_time_check_selftest_expired_status`, `cert_time_real_cert:parsed`, `cert_time_real_cert:within_on_unverified_basis`, `cert_time_real_cert:grants_nothing`, `protocol:cert_time_check_selftest_grants_nothing` | envelope `.facts`, plus observed/no-grant-key checks |
| REGENERATED parsed | `protocol:system_honesty_report_owner_seal_dev_tier`, `protocol:system_honesty_report_grants_nothing`, `protocol:system_honesty_report_provider_time_match`, `protocol:system_honesty_report_standing_posture`, `protocol:system_honesty_report_no_dishonest_overclaim`, `m12-distribution:N5_owner_key_provisioning_posture` | envelope `.facts`; provider/time SAME-SOURCE comparison retained |
| REGENERATED parsed | `protocol:describe_v1_observed`, `protocol:snapshot_v1_observed`, `protocol:capability_catalog_status_observed`, `protocol:service_inventory_v1_observed`, `protocol:problem_list_v1_observed`, `persistence:persist_layout_v1_observed`, `protocol:generic_capability_denial_v1_ordered` | family-scoped parsed v1 assertions |
| REGENERATED parsed | unnamed quick command-envelope routing checks for `system.describe`, `system.snapshot`, `system.boot_log`, `system.capabilities`, `device.graph`, `service.inventory`, `problem.list` | routed response `schema`/`family`; boot-log/device also assert observed/null-event/no grant/effect keys |
| HONEST MERGE | `quick:echo_lifecycle_inventory_lists_echo`, `quick:echo_lifecycle_inventory_removes_echo`, `m6c:live_service_inventory_lists_granted_candidate`, `m6d:live_service_inventory_lists_granted_candidate`, `m6d:post_rollback_service_inventory_removes_granted_candidate`, `genesis-ui:calculator-runs-only-as-ui-current-boot-service`, `genesis-ui:calculator-f12-restores-core-genesis`, `genesis-ui:personal-shell-dynamic-inventory`, `genesis-ui:personal-shell-f12-removes-dynamic-inventory`, `genesis-ui:personal-shell-fallback-removes-dynamic-inventory`, `app_inventory:running`, `app_f12:inventory_removed`, `boot4:no_workspace_autoload` | one captured `service.inventory` projection; detailed lifecycle authority remains owned by the already-landed lifecycle responses |

The quick Hello running/removed/host-bound inventory checks are unnamed
hard-fail assertions rather than report predicates; each was regenerated in
place against the same captured `facts.services` carrier.

Donor-removal scan hits that intentionally remain are
`protocol:memory_context_snapshot_source` (`system.snapshot.v0`),
`protocol:memory_context_service_source` (`service.inventory.v0`), and
`protocol:memory_context_problem_source` (`problem.list.v0`): each needle is
emitted by its own memory-context source-locator facts, not donated by a P4-9b
response. The Genesis profile's `genesisSystem.schema == system.snapshot.v0`
and `genesisProblemResult.schema == problem.list.v0` are fields of the carved-out
Genesis response and were not converted.

Confirmed: no project/distribution/registry/Wasm/program/Genesis emitter source
file was touched. Only harness consumers of the in-scope `service.inventory`
and `system.honesty_report` responses changed in those profiles. No canonical
hash builder or hash formatting path changed.
