
    Send-AgentCommand -Command "agent module.grant_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_grant_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_artifacts" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_case_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_co_emission_invariant" -Needle '"co_emission_invariant": "can_load_now_true_implies_trust_tier_dev_key_not_owner_sealed_and_grants_capability"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_signed_grants_case" -Needle '"case": "signed_fully_bound_attestation_grants_capability"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_attestation_case" -Needle '"case": "no_retained_attestation_no_grant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_unsigned_case" -Needle '"case": "unsigned_hash_valid_attestation_no_grant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_different_grant_case" -Needle '"case": "signed_different_grant_attestation_no_grant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_shadowed_case" -Needle '"case": "signed_record_shadowed_by_unsigned_retain_no_grant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_valid_status" -Needle '"actual": {"status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_stale_status" -Needle '"actual": {"status": "stale_or_non_current_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_mismatch_case" -Needle '"case": "mismatched_manifest_hash_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_mismatch_status" -Needle '"actual": {"status": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_wrong_policy_case" -Needle '"case": "grants_load_now_or_wrong_policy_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_artifacts" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_audit_records" -Needle '"durable_audit_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_rollback_plans" -Needle '"rollback_plan_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_slots" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_valid_status" -Needle '"actual": {"status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_event_case" -Needle '"case": "previous_boot_denial_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_audit_schema_case" -Needle '"case": "audit_record_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_rollback_schema_case" -Needle '"case": "rollback_plan_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_substituted_case" -Needle '"case": "substituted_audit_record_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_rollback_hash_case" -Needle '"case": "mismatched_rollback_plan_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_grant_hash_case" -Needle '"case": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_slot_case" -Needle '"case": "invalid_ram_only_service_slot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.service_slot_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_service_slot_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_slots" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_case" -Needle '"case": "accepted_current_boot_reservation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_status" -Needle '"actual": {"status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_reason" -Needle '"actual": {"status": "valid_hash_reference_load_still_denied", "reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_stale_case" -Needle '"case": "stale_previous_boot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_stale_status" -Needle '"actual": {"status": "stale_or_non_current_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_mismatch_case" -Needle '"case": "mismatched_reservation_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_mismatch_reason" -Needle '"actual": {"status": "mismatched_reservation_hash", "reason": "service_slot_reservation_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_slot_case" -Needle '"case": "invalid_ram_only_service_slot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_slot_reason" -Needle '"actual": {"status": "rejected", "reason": "ram_only_service_slot_id_invalid"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1


    Send-AgentCommand -Command "agent module.service_slot_allocator_selftest" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_allocator_selftest"
Assert-LogContains -Name "protocol:module_service_slot_allocator_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_selftest_count" -Needle '"case_count": 29' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_selftest_safety_slot" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_selftest_safety_inventory" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    $allocatorSelftestCases = @(
        @("missing_retained_service_slot_reservation", "missing", "retained_service_slot_reservation_missing"),
        @("service_slot_allocator_previous_boot", "rejected", "service_slot_allocator_scope_must_be_current_boot"),
        @("service_slot_allocator_wrong_schema", "rejected", "service_slot_allocator_schema_mismatch"),
        @("service_slot_allocator_provenance_missing", "rejected", "service_slot_allocator_provenance_missing"),
        @("service_slot_allocator_retained_reservation_binding_missing", "rejected", "service_slot_allocator_retained_reservation_binding_missing"),
        @("service_slot_allocator_runtime_missing", "missing", "service_slot_allocator_runtime_missing"),
        @("service_slot_allocator_runtime_observed_source_evidence_missing", "missing", "service_slot_allocator_runtime_missing"),
        @("service_slot_allocator_runtime_observed_source_evidence_available_registry_missing", "missing", "service_slot_registry_binding_missing"),
        @("service_slot_registry_binding_missing", "missing", "service_slot_registry_binding_missing"),
        @("service_slot_registry_binding_observed_source_evidence_missing", "missing", "service_slot_registry_binding_missing"),
        @("service_slot_registry_binding_observed_source_evidence_available_health_missing", "missing", "service_health_state_model_missing"),
        @("service_slot_registry_allocator_runtime_binding_missing", "rejected", "service_slot_registry_allocator_runtime_binding_missing"),
        @("service_health_state_model_missing", "missing", "service_health_state_model_missing"),
        @("service_health_state_model_observed_source_evidence_missing", "missing", "service_health_state_model_missing"),
        @("service_health_state_model_observed_source_evidence_available_unload_missing", "missing", "service_unload_cleanup_plan_missing"),
        @("service_unload_cleanup_plan_missing", "missing", "service_unload_cleanup_plan_missing"),
        @("service_unload_cleanup_plan_observed_source_evidence_missing", "missing", "service_unload_cleanup_plan_missing"),
        @("service_unload_cleanup_plan_observed_source_evidence_available_durable_missing", "denied_missing_durable_audit_write", "durable_audit_write_missing"),
        @("durable_audit_write_missing", "denied_missing_durable_audit_write", "durable_audit_write_missing"),
        @("durable_audit_write_observed_source_evidence_missing", "denied_missing_durable_audit_write", "durable_audit_write_missing"),
        @("durable_audit_write_observed_source_evidence_available_rollback_missing", "denied_missing_rollback_install", "rollback_install_missing"),
        @("rollback_install_missing", "denied_missing_rollback_install", "rollback_install_missing"),
        @("rollback_install_observed_source_evidence_missing", "denied_missing_rollback_install", "rollback_install_missing"),
        @("rollback_install_observed_source_evidence_available_module_loader_unimplemented", "denied_loader_unimplemented", "module_loader_unimplemented"),
        @("module_loader_missing", "denied_loader_unimplemented", "module_loader_unimplemented"),
        @("module_loader_observed_source_evidence_missing", "denied_loader_unimplemented", "module_loader_unimplemented"),
        @("module_loader_observed_source_evidence_available_allocator_authority_boundary", "denied_allocator_authority_not_granted", "service_slot_allocator_authority_boundary_non_authorizing"),
        @("registry_write_commit_gate_missing", "missing", "service_slot_registry_write_commit_gate_source_chain_incomplete"),
        @("all_inputs_ready_still_non_authorizing", "denied_allocator_authority_not_granted", "service_slot_allocator_authority_boundary_non_authorizing")
    )
    foreach ($case in $allocatorSelftestCases) {
        Assert-LogContains -Name ("protocol:module_service_slot_allocator_selftest_case_" + $case[0]) -Needle ('"case": "' + $case[0] + '", "expected": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "actual": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "passed": true') -TimeoutSeconds 1
    }

    $loaderRuntimeFactSources = @(
        @{ Suffix = "identity"; Method = "module.loader_identity"; Locator = "module.loader_identity.loader_identity" },
        @{ Suffix = "artifact_hash"; Method = "module.loader_artifact_hash_binding"; Locator = "module.loader_artifact_hash_binding.artifact_hash_binding" },
        @{ Suffix = "entrypoint"; Method = "module.loader_entrypoint_abi"; Locator = "module.loader_entrypoint_abi.entrypoint_abi" },
        @{ Suffix = "address_space"; Method = "module.loader_address_space_boundary"; Locator = "module.loader_address_space_boundary.address_space_boundary" },
        @{ Suffix = "memory_map"; Method = "module.loader_memory_map_constraints"; Locator = "module.loader_memory_map_constraints.memory_map_constraints" },
        @{ Suffix = "capability_table"; Method = "module.loader_capability_import_table"; Locator = "module.loader_capability_import_table.capability_import_table" },
        @{ Suffix = "service_slot"; Method = "module.loader_service_slot_binding"; Locator = "module.loader_service_slot_binding.service_slot_binding" },
        @{ Suffix = "health"; Method = "module.loader_health_state_hooks"; Locator = "module.loader_health_state_hooks.health_state_hooks" },
        @{ Suffix = "rollback"; Method = "module.loader_rollback_hooks"; Locator = "module.loader_rollback_hooks.rollback_hooks" },
        @{ Suffix = "write_boundary"; Method = "module.loader_audit_rollback_write_boundary_binding"; Locator = "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding" }
    )

    Send-AgentCommand -Command "agent module.loader_runtime_selftest" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime_selftest"
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_count" -Needle '"case_count": 72' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_source_count" -Needle '"source_fact_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_source_complete" -Needle '"source_fact_map_complete": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_safety_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_selftest_safety_attempt" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1
    $loaderRuntimeSelftestCases = @(
        @("missing_manifest_reference", "denied_missing_retained_module_evidence", "retained_module_manifest_reference_missing"),
        @("missing_artifact_reference", "denied_missing_retained_module_evidence", "retained_module_candidate_artifact_reference_missing"),
        @("missing_vm_report_reference", "denied_missing_retained_module_evidence", "retained_module_vm_test_report_reference_missing"),
        @("missing_local_attestation_reference", "denied_missing_retained_module_evidence", "retained_module_local_attestation_reference_missing"),
        @("missing_local_approval_reference", "denied_missing_retained_module_evidence", "retained_module_local_approval_reference_missing"),
        @("missing_computed_grant_reference", "denied_missing_retained_module_evidence", "retained_module_computed_grant_reference_missing"),
        @("missing_audit_rollback_reference", "denied_missing_retained_module_evidence", "retained_module_audit_rollback_reference_missing"),
        @("missing_service_slot_reservation", "denied_missing_retained_module_evidence", "retained_module_service_slot_reservation_missing"),
        @("missing_service_slot_allocator_readiness", "denied_missing_service_slot_allocator_readiness", "service_slot_allocator_readiness_missing"),
        @("service_slot_allocator_runtime_missing", "denied_missing_service_slot_allocator_runtime", "service_slot_allocator_runtime_missing"),
        @("loader_identity_previous_boot", "rejected", "module_loader_identity_scope_must_be_current_boot"),
        @("loader_identity_wrong_schema", "rejected", "module_loader_identity_schema_mismatch"),
        @("loader_identity_provenance_missing", "rejected", "module_loader_identity_provenance_missing"),
        @("loader_identity_retained_evidence_binding_missing", "rejected", "module_loader_identity_retained_evidence_binding_missing"),
        @("loader_identity_service_slot_allocator_binding_missing", "rejected", "module_loader_identity_service_slot_allocator_binding_missing"),
        @("loader_identity_audit_write_boundary_binding_missing", "rejected", "module_loader_identity_audit_write_boundary_binding_missing"),
        @("loader_identity_missing", "denied_missing_loader_runtime_fact", "module_loader_identity_missing"),
        @("loader_identity_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_identity_missing"),
        @("artifact_hash_binding_missing", "denied_missing_loader_runtime_fact", "module_loader_artifact_hash_binding_missing"),
        @("artifact_hash_binding_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_artifact_hash_binding_missing"),
        @("entrypoint_abi_missing", "denied_missing_loader_runtime_fact", "module_loader_entrypoint_abi_missing"),
        @("entrypoint_abi_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_entrypoint_abi_missing"),
        @("address_space_boundary_missing", "denied_missing_loader_runtime_fact", "module_loader_address_space_boundary_missing"),
        @("address_space_boundary_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_address_space_boundary_missing"),
        @("memory_map_constraints_missing", "denied_missing_loader_runtime_fact", "module_loader_memory_map_constraints_missing"),
        @("memory_map_constraints_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_memory_map_constraints_missing"),
        @("capability_import_table_missing", "denied_missing_loader_runtime_fact", "module_loader_capability_import_table_missing"),
        @("capability_import_table_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_capability_import_table_missing"),
        @("service_slot_binding_missing", "denied_missing_loader_runtime_fact", "module_loader_service_slot_binding_missing"),
        @("service_slot_binding_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_service_slot_binding_missing"),
        @("health_state_hooks_missing", "denied_missing_loader_runtime_fact", "module_loader_health_state_hooks_missing"),
        @("health_state_hooks_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_health_state_hooks_missing"),
        @("rollback_hooks_missing", "denied_missing_loader_runtime_fact", "module_loader_rollback_hooks_missing"),
        @("rollback_hooks_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_rollback_hooks_missing"),
        @("audit_rollback_write_boundary_binding_missing", "denied_missing_loader_runtime_fact", "module_loader_audit_rollback_write_boundary_binding_missing"),
        @("audit_rollback_write_boundary_binding_observed_source_evidence_missing", "denied_missing_loader_runtime_fact", "module_loader_audit_rollback_write_boundary_binding_missing"),
        @("execution_commit_gate_missing", "denied_missing_module_loader_runtime_execution_commit_gate", "module_loader_runtime_execution_commit_gate_source_chain_incomplete"),
        @("descriptor_intake_boundary_missing", "denied_missing_module_loader_descriptor_intake_boundary", "module_loader_descriptor_intake_boundary_source_chain_incomplete"),
        @("artifact_byte_intake_boundary_missing", "denied_missing_module_loader_artifact_byte_intake_boundary", "module_loader_artifact_byte_intake_boundary_source_chain_incomplete"),
        @("execution_authorization_boundary_missing", "denied_missing_module_loader_execution_authorization_boundary", "module_loader_execution_authorization_boundary_source_chain_incomplete"),
        @("service_registry_mutation_boundary_missing", "denied_missing_module_loader_service_registry_mutation_boundary", "module_loader_service_registry_mutation_boundary_source_chain_incomplete"),
        @("load_attempt_boundary_missing", "denied_missing_module_loader_load_attempt_boundary", "module_loader_load_attempt_boundary_source_chain_incomplete"),
        @("artifact_load_boundary_missing", "denied_missing_module_loader_artifact_load_boundary", "module_loader_artifact_load_boundary_source_chain_incomplete"),
        @("executable_mapping_boundary_missing", "denied_missing_module_loader_executable_mapping_boundary", "module_loader_executable_mapping_boundary_source_chain_incomplete"),
        @("entrypoint_transfer_boundary_missing", "denied_missing_module_loader_entrypoint_transfer_boundary", "module_loader_entrypoint_transfer_boundary_source_chain_incomplete"),
        @("service_start_boundary_missing", "denied_missing_module_loader_service_start_boundary", "module_loader_service_start_boundary_source_chain_incomplete"),
        @("service_health_binding_boundary_missing", "denied_missing_module_loader_service_health_binding_boundary", "module_loader_service_health_binding_boundary_source_chain_incomplete"),
        @("service_running_state_boundary_missing", "denied_missing_module_loader_service_running_state_boundary", "module_loader_service_running_state_boundary_source_chain_incomplete"),
        @("service_start_audit_boundary_missing", "denied_missing_module_loader_service_start_audit_boundary", "module_loader_service_start_audit_boundary_source_chain_incomplete"),
        @("service_unload_cleanup_boundary_missing", "denied_missing_module_loader_service_unload_cleanup_boundary", "module_loader_service_unload_cleanup_boundary_source_chain_incomplete"),
        @("live_load_commit_boundary_missing", "denied_missing_module_loader_live_load_commit_boundary", "module_loader_live_load_commit_boundary_source_chain_incomplete"),
        @("commit_audit_boundary_missing", "denied_missing_module_loader_commit_audit_boundary", "module_loader_commit_audit_boundary_source_chain_incomplete"),
        @("commit_rollback_boundary_missing", "denied_missing_module_loader_commit_rollback_boundary", "module_loader_commit_rollback_boundary_source_chain_incomplete"),
        @("commit_result_boundary_missing", "denied_missing_module_loader_commit_result_boundary", "module_loader_commit_result_boundary_source_chain_incomplete"),
        @("descriptor_acceptance_authority_boundary_missing", "denied_missing_module_loader_descriptor_acceptance_authority_boundary", "module_loader_descriptor_acceptance_authority_boundary_source_chain_incomplete"),
        @("descriptor_parser_contract_boundary_missing", "denied_missing_module_loader_descriptor_parser_contract_boundary", "module_loader_descriptor_parser_contract_boundary_source_chain_incomplete"),
        @("descriptor_parser_result_boundary_missing", "denied_missing_module_loader_descriptor_parser_result_boundary", "module_loader_descriptor_parser_result_boundary_source_chain_incomplete"),
        @("descriptor_schema_validation_boundary_missing", "denied_missing_module_loader_descriptor_schema_validation_boundary", "module_loader_descriptor_schema_validation_boundary_source_chain_incomplete"),
        @("descriptor_capability_validation_boundary_missing", "denied_missing_module_loader_descriptor_capability_validation_boundary", "module_loader_descriptor_capability_validation_boundary_source_chain_incomplete"),
        @("descriptor_load_plan_boundary_missing", "denied_missing_module_loader_descriptor_load_plan_boundary", "module_loader_descriptor_load_plan_boundary_source_chain_incomplete"),
        @("executable_load_plan_authority_boundary_missing", "denied_missing_module_loader_executable_load_plan_authority_boundary", "module_loader_executable_load_plan_authority_boundary_source_chain_incomplete"),
        @("executable_load_plan_result_boundary_missing", "denied_missing_module_loader_executable_load_plan_result_boundary", "module_loader_executable_load_plan_result_boundary_source_chain_incomplete"),
        @("executable_image_layout_boundary_missing", "denied_missing_module_loader_executable_image_layout_boundary", "module_loader_executable_image_layout_boundary_source_chain_incomplete"),
        @("executable_page_mapping_plan_boundary_missing", "denied_missing_module_loader_executable_page_mapping_plan_boundary", "module_loader_executable_page_mapping_plan_boundary_source_chain_incomplete"),
        @("executable_page_mapping_boundary_missing", "denied_missing_module_loader_executable_page_mapping_boundary", "module_loader_executable_page_mapping_boundary_source_chain_incomplete"),
        @("descriptor_executable_page_binding_boundary_missing", "denied_missing_module_loader_descriptor_executable_page_binding_boundary", "module_loader_descriptor_executable_page_binding_boundary_source_chain_incomplete"),
        @("executable_entrypoint_binding_boundary_missing", "denied_missing_module_loader_executable_entrypoint_binding_boundary", "module_loader_executable_entrypoint_binding_boundary_source_chain_incomplete"),
        @("executable_entrypoint_transfer_authorization_boundary_missing", "denied_missing_module_loader_executable_entrypoint_transfer_authorization_boundary", "module_loader_executable_entrypoint_transfer_authorization_boundary_source_chain_incomplete"),
        @("executable_entrypoint_transfer_boundary_missing", "denied_missing_module_loader_executable_entrypoint_transfer_boundary", "module_loader_executable_entrypoint_transfer_boundary_source_chain_incomplete"),
        @("executable_entrypoint_handoff_boundary_missing", "denied_missing_module_loader_executable_entrypoint_handoff_boundary", "module_loader_executable_entrypoint_handoff_boundary_source_chain_incomplete"),
        @("executable_entrypoint_invocation_boundary_missing", "denied_missing_module_loader_executable_entrypoint_invocation_boundary", "module_loader_executable_entrypoint_invocation_boundary_source_chain_incomplete"),
        @("all_inputs_ready_defined_non_executable", "defined_non_executable", "module_loader_runtime_behavior_not_implemented")
    )
    foreach ($case in $loaderRuntimeSelftestCases) {
        Assert-LogContains -Name ("protocol:module_loader_runtime_selftest_case_" + $case[0]) -Needle ('"case": "' + $case[0] + '", "expected": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "actual": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "passed": true') -TimeoutSeconds 1
    }
    foreach ($source in $loaderRuntimeFactSources) {
        Assert-LogContains -Name ("protocol:runtime_selftest_" + $source.Suffix + "_method") -Needle ('"source_method": "' + $source.Method + '"') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:runtime_selftest_" + $source.Suffix + "_locator") -Needle ('"source_fact_locator": "' + $source.Locator + '"') -TimeoutSeconds 1
    }
    Send-AgentCommand -Command "agent module.loader_identity" -ExpectedMarker "RAIOS_AGENT_END module.loader_identity"
    Assert-LogContains -Name "protocol:module_loader_identity_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_identity_fact" -Needle '"id": "loader_identity", "kind": "loader_fact", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_identity_fact_reason" -Needle '"id": "loader_identity", "kind": "loader_fact", "status": "missing", "reason": "module_loader_identity_missing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_identity_fact_record" -Needle '"record_schema": "raios.module_loader_identity.v0", "record_id": "module.loader_runtime.identity.current_boot", "source_method": "module.loader_identity", "source_fact_locator": "module.loader_identity.loader_identity"' -TimeoutSeconds 1
    Send-AgentCommand -Command "agent module.loader_identity_selftest" -ExpectedMarker "RAIOS_AGENT_END module.loader_identity_selftest"
    Assert-LogContains -Name "protocol:module_loader_identity_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_identity_selftest_count" -Needle '"case_count": 12' -TimeoutSeconds 1
    $loaderIdentitySelftestCases = @(
        @("missing_retained_module_evidence", "denied_missing_retained_module_evidence", "retained_module_evidence_missing"),
        @("audit_write_boundary_missing", "denied_missing_audit_rollback_write_boundary", "module_audit_rollback_write_boundary_binding_missing"),
        @("loader_identity_schema_mismatch", "rejected", "module_loader_identity_schema_mismatch"),
        @("loader_identity_missing", "denied_missing_loader_identity", "module_loader_identity_missing"),
        @("all_inputs_present_identity_non_authorizing", "available_non_authorizing", "module_loader_identity_not_load_authority")
    )
    foreach ($case in $loaderIdentitySelftestCases) {
        Assert-LogContains -Name ("protocol:module_loader_identity_selftest_case_" + $case[0]) -Needle ('"case": "' + $case[0] + '", "expected": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "actual": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "passed": true') -TimeoutSeconds 1
    }
    Send-AgentCommand -Command "agent module.loader_artifact_hash_binding" -ExpectedMarker "RAIOS_AGENT_END module.loader_artifact_hash_binding"
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_fact" -Needle '"id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_fact_reason" -Needle '"id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing", "reason": "module_loader_artifact_hash_binding_missing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_fact_record" -Needle '"record_schema": "raios.module_loader_artifact_hash_binding.v0", "record_id": "module.loader_runtime.artifact_hash_binding.current_boot", "source_method": "module.loader_artifact_hash_binding", "source_fact_locator": "module.loader_artifact_hash_binding.artifact_hash_binding"' -TimeoutSeconds 1
    Send-AgentCommand -Command "agent module.loader_artifact_hash_binding_selftest" -ExpectedMarker "RAIOS_AGENT_END module.loader_artifact_hash_binding_selftest"
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_selftest_count" -Needle '"case_count": 14' -TimeoutSeconds 1
    $artifactHashBindingSelftestCases = @(
        @("artifact_hash_binding_previous_boot", "rejected", "module_loader_artifact_hash_binding_scope_must_be_current_boot"),
        @("artifact_hash_binding_schema_mismatch", "rejected", "module_loader_artifact_hash_binding_schema_mismatch"),
        @("artifact_hash_binding_provenance_missing", "rejected", "module_loader_artifact_hash_binding_provenance_missing"),
        @("artifact_hash_binding_retained_evidence_binding_missing", "rejected", "module_loader_artifact_hash_binding_retained_evidence_binding_missing"),
        @("artifact_hash_binding_service_slot_allocator_binding_missing", "rejected", "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing"),
        @("artifact_hash_binding_audit_write_boundary_binding_missing", "rejected", "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing"),
        @("artifact_hash_binding_loader_identity_binding_missing", "rejected", "module_loader_artifact_hash_binding_loader_identity_binding_missing"),
        @("artifact_hash_binding_missing", "denied_missing_loader_artifact_hash_binding", "module_loader_artifact_hash_binding_missing"),
        @("all_inputs_present_artifact_hash_binding_non_authorizing", "available_non_authorizing", "module_loader_artifact_hash_binding_not_load_authority")
    )
    foreach ($case in $artifactHashBindingSelftestCases) {
        Assert-LogContains -Name ("protocol:module_loader_artifact_hash_binding_selftest_case_" + $case[0]) -Needle ('"case": "' + $case[0] + '", "expected": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "actual": {"status": "' + $case[1] + '", "reason": "' + $case[2] + '"}, "passed": true') -TimeoutSeconds 1
    }
    $loaderFactDiagnostics = @(
        @{ Method = "module.loader_entrypoint_abi"; Selftest = "module.loader_entrypoint_abi_selftest"; Id = "entrypoint_abi"; MissingReason = "module_loader_entrypoint_abi_missing"; BindingReason = "module_loader_entrypoint_abi_artifact_hash_binding_binding_missing"; ReadyReason = "module_loader_entrypoint_abi_not_load_authority" },
        @{ Method = "module.loader_address_space_boundary"; Selftest = "module.loader_address_space_boundary_selftest"; Id = "address_space_boundary"; MissingReason = "module_loader_address_space_boundary_missing"; BindingReason = "module_loader_address_space_boundary_entrypoint_abi_binding_missing"; ReadyReason = "module_loader_address_space_boundary_not_load_authority" },
        @{ Method = "module.loader_memory_map_constraints"; Selftest = "module.loader_memory_map_constraints_selftest"; Id = "memory_map_constraints"; MissingReason = "module_loader_memory_map_constraints_missing"; BindingReason = "module_loader_memory_map_constraints_address_space_boundary_binding_missing"; ReadyReason = "module_loader_memory_map_constraints_not_load_authority" },
        @{ Method = "module.loader_capability_import_table"; Selftest = "module.loader_capability_import_table_selftest"; Id = "capability_import_table"; MissingReason = "module_loader_capability_import_table_missing"; BindingReason = "module_loader_capability_import_table_memory_map_constraints_binding_missing"; ReadyReason = "module_loader_capability_import_table_not_load_authority" },
        @{ Method = "module.loader_service_slot_binding"; Selftest = "module.loader_service_slot_binding_selftest"; Id = "service_slot_binding"; MissingReason = "module_loader_service_slot_binding_missing"; BindingReason = "module_loader_service_slot_binding_capability_import_table_binding_missing"; ReadyReason = "module_loader_service_slot_binding_not_load_authority" },
        @{ Method = "module.loader_health_state_hooks"; Selftest = "module.loader_health_state_hooks_selftest"; Id = "health_state_hooks"; MissingReason = "module_loader_health_state_hooks_missing"; BindingReason = "module_loader_health_state_hooks_service_slot_binding_binding_missing"; ReadyReason = "module_loader_health_state_hooks_not_load_authority" },
        @{ Method = "module.loader_rollback_hooks"; Selftest = "module.loader_rollback_hooks_selftest"; Id = "rollback_hooks"; MissingReason = "module_loader_rollback_hooks_missing"; BindingReason = "module_loader_rollback_hooks_health_state_hooks_binding_missing"; ReadyReason = "module_loader_rollback_hooks_not_load_authority" },
        @{ Method = "module.loader_audit_rollback_write_boundary_binding"; Selftest = "module.loader_audit_rollback_write_boundary_binding_selftest"; Id = "audit_rollback_write_boundary_binding"; MissingReason = "module_loader_audit_rollback_write_boundary_binding_missing"; BindingReason = "module_loader_audit_rollback_write_boundary_binding_rollback_hooks_binding_missing"; ReadyReason = "module_loader_audit_rollback_write_boundary_binding_not_load_authority" }
    )
    foreach ($fact in $loaderFactDiagnostics) {
        Send-AgentCommand -Command ("agent " + $fact.Method) -ExpectedMarker ("RAIOS_AGENT_END " + $fact.Method)
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_schema") -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_fact") -Needle ('"id": "' + $fact.Id + '", "kind": "loader_fact", "status": "missing", "reason": "' + $fact.MissingReason + '"') -TimeoutSeconds 1
        Send-AgentCommand -Command ("agent " + $fact.Selftest) -ExpectedMarker ("RAIOS_AGENT_END " + $fact.Selftest)
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_schema") -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_count") -Needle '"case_count": 14' -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_binding") -Needle ('"actual": {"status": "rejected", "reason": "' + $fact.BindingReason + '"}') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_missing") -Needle ('"actual": {"status": "denied_missing_loader_runtime_fact", "reason": "' + $fact.MissingReason + '"}') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_ready") -Needle ('"actual": {"status": "available_non_authorizing", "reason": "' + $fact.ReadyReason + '"}') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_selftest_safety") -Needle '"load_attempt_count": 0' -TimeoutSeconds 1
    }
    Send-AgentCommand -Command "agent module.audit_rollback_availability_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_availability_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_availability_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_missing_case" -Needle '"case": "missing_ledger_and_store_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_scope_case" -Needle '"case": "durable_audit_ledger_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_schema_case" -Needle '"case": "durable_audit_ledger_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_provenance_case" -Needle '"case": "durable_audit_ledger_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_scope_case" -Needle '"case": "rollback_store_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_schema_case" -Needle '"case": "rollback_store_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_provenance_case" -Needle '"case": "rollback_store_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_case" -Needle '"case": "available_facts_policy_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_status" -Needle '"actual_status": "denied_missing_durable_write_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_reason" -Needle '"actual_reason": "durable_write_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_write_policy_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_write_policy_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_count" -Needle '"case_count": 12' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_missing_case" -Needle '"case": "missing_policy_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_scope_case" -Needle '"case": "durable_write_policy_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_schema_case" -Needle '"case": "durable_write_policy_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_provenance_case" -Needle '"case": "durable_write_policy_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_binding_case" -Needle '"case": "durable_write_policy_retained_evidence_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_availability_case" -Needle '"case": "durable_write_policy_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_scope_case" -Needle '"case": "rollback_install_policy_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_schema_case" -Needle '"case": "rollback_install_policy_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_provenance_case" -Needle '"case": "rollback_install_policy_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_binding_case" -Needle '"case": "rollback_install_policy_retained_evidence_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_availability_case" -Needle '"case": "rollback_install_policy_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_case" -Needle '"case": "available_policy_facts_writer_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_storage_layout_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_storage_layout_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_count" -Needle '"case_count": 18' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_missing_case" -Needle '"case": "missing_storage_inputs_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_missing_reason" -Needle '"actual_reason": "persistence_device_inventory_missing_and_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_scope_case" -Needle '"case": "persistence_device_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_schema_case" -Needle '"case": "persistence_device_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_provenance_case" -Needle '"case": "persistence_device_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_identity_case" -Needle '"case": "persistence_device_stable_identity_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_block_identity_case" -Needle '"case": "block_device_identity_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_sector_read_case" -Needle '"case": "persistence_sector_read_path_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_block_driver_case" -Needle '"case": "persistence_block_driver_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_partition_case" -Needle '"case": "persistence_partition_inventory_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_scope_case" -Needle '"case": "audit_rollback_storage_layout_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_schema_case" -Needle '"case": "audit_rollback_storage_layout_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_provenance_case" -Needle '"case": "audit_rollback_storage_layout_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_binding_case" -Needle '"case": "storage_layout_device_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_audit_region_case" -Needle '"case": "audit_ledger_layout_region_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_rollback_region_case" -Needle '"case": "rollback_store_layout_region_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_append_slots_case" -Needle '"case": "storage_layout_append_slots_missing"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_recovery_case" -Needle '"case": "storage_layout_recovery_boundary_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_case" -Needle '"case": "available_storage_layout_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_storage_layout_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_engine_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_engine_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_engine_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_count" -Needle '"case_count": 16' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_missing_case" -Needle '"case": "missing_append_engine_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_missing_reason" -Needle '"actual_reason": "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_scope_case" -Needle '"case": "audit_ledger_append_engine_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_schema_case" -Needle '"case": "audit_ledger_append_engine_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_provenance_case" -Needle '"case": "audit_ledger_append_engine_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_storage_case" -Needle '"case": "audit_ledger_append_engine_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_policy_case" -Needle '"case": "audit_ledger_append_engine_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_append_only_case" -Needle '"case": "audit_ledger_append_engine_append_only_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_flush_case" -Needle '"case": "audit_ledger_append_engine_flush_support_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_recovery_case" -Needle '"case": "audit_ledger_append_engine_recovery_boundary_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_scope_case" -Needle '"case": "rollback_store_transaction_engine_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_schema_case" -Needle '"case": "rollback_store_transaction_engine_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_provenance_case" -Needle '"case": "rollback_store_transaction_engine_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_storage_case" -Needle '"case": "rollback_store_transaction_engine_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_policy_case" -Needle '"case": "rollback_store_transaction_engine_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_replay_case" -Needle '"case": "rollback_store_transaction_engine_replay_support_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_case" -Needle '"case": "available_append_engines_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_engine_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_contract_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_contract_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_count" -Needle '"case_count": 24' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_missing_case" -Needle '"case": "missing_append_envelope_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_missing_reason" -Needle '"actual_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_scope_case" -Needle '"case": "audit_append_envelope_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_schema_case" -Needle '"case": "audit_append_envelope_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_provenance_case" -Needle '"case": "audit_append_envelope_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_provenance_binding_case" -Needle '"case": "audit_append_envelope_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_policy_binding_case" -Needle '"case": "audit_append_envelope_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_policy_id_case" -Needle '"case": "audit_append_envelope_write_policy_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_availability_case" -Needle '"case": "audit_append_envelope_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_availability_id_case" -Needle '"case": "audit_append_envelope_availability_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_storage_id_case" -Needle '"case": "audit_append_envelope_storage_layout_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_engine_id_case" -Needle '"case": "audit_append_envelope_append_engine_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_storage_case" -Needle '"case": "audit_ledger_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_envelope_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_envelope_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_envelope_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_provenance_binding_case" -Needle '"case": "rollback_transaction_envelope_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_policy_binding_case" -Needle '"case": "rollback_transaction_envelope_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_policy_id_case" -Needle '"case": "rollback_transaction_envelope_write_policy_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_availability_case" -Needle '"case": "rollback_transaction_envelope_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_availability_id_case" -Needle '"case": "rollback_transaction_envelope_availability_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_storage_id_case" -Needle '"case": "rollback_transaction_envelope_storage_layout_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_engine_id_case" -Needle '"case": "rollback_transaction_envelope_append_engine_id_missing"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_storage_case" -Needle '"case": "rollback_store_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_case" -Needle '"case": "available_envelopes_append_engine_still_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_status" -Needle '"actual_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_reason" -Needle '"actual_reason": "audit_ledger_append_engine_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_payload_hash_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_payload_hash_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_payload_hash_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_count" -Needle '"case_count": 20' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_missing_case" -Needle '"case": "missing_payload_hash_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_missing_reason" -Needle '"actual_reason": "audit_record_append_payload_hash_missing_and_rollback_transaction_append_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_scope_case" -Needle '"case": "audit_record_payload_hash_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_schema_case" -Needle '"case": "audit_record_payload_hash_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_provenance_case" -Needle '"case": "audit_record_payload_hash_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_retained_binding_case" -Needle '"case": "audit_record_payload_hash_retained_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_slot_binding_case" -Needle '"case": "audit_record_payload_hash_service_slot_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_request_binding_case" -Needle '"case": "audit_record_payload_hash_write_request_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_contract_binding_case" -Needle '"case": "audit_record_payload_hash_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_target_binding_case" -Needle '"case": "audit_record_payload_hash_target_schema_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_payload_case" -Needle '"case": "audit_record_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_retained_missing_case" -Needle '"case": "audit_record_retained_audit_rollback_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_slot_missing_case" -Needle '"case": "audit_record_service_slot_reservation_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_contract_missing_case" -Needle '"case": "audit_record_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_payload_hash_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_payload_hash_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_payload_hash_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_contract_binding_case" -Needle '"case": "rollback_transaction_payload_hash_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_payload_case" -Needle '"case": "rollback_transaction_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_contract_missing_case" -Needle '"case": "rollback_transaction_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_case" -Needle '"case": "available_payload_hashes_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_payload_hash_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_intent_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_intent_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_intent_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_count" -Needle '"case_count": 20' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_missing_case" -Needle '"case": "missing_append_intent_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_missing_reason" -Needle '"actual_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_scope_case" -Needle '"case": "audit_record_append_intent_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_schema_case" -Needle '"case": "audit_record_append_intent_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_provenance_case" -Needle '"case": "audit_record_append_intent_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_provenance_binding_case" -Needle '"case": "audit_record_append_intent_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_contract_binding_case" -Needle '"case": "audit_record_append_intent_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_engine_binding_case" -Needle '"case": "audit_record_append_intent_append_engine_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_storage_binding_case" -Needle '"case": "audit_record_append_intent_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_policy_binding_case" -Needle '"case": "audit_record_append_intent_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_availability_binding_case" -Needle '"case": "audit_record_append_intent_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_payload_case" -Needle '"case": "audit_record_append_intent_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_contract_missing_case" -Needle '"case": "audit_record_append_intent_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_payload_envelope_case" -Needle '"case": "audit_record_append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_append_intent_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_append_intent_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_append_intent_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_contract_binding_case" -Needle '"case": "rollback_transaction_append_intent_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_payload_case" -Needle '"case": "rollback_transaction_append_intent_payload_hash_missing"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_payload_envelope_case" -Needle '"case": "rollback_transaction_append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_case" -Needle '"case": "available_append_intents_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_intent_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_write_boundary_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary_selftest"
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_write_boundary_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_recovery_artifact" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_count" -Needle '"case_count": 22' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_missing_manifest" -Needle '"case": "missing_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_stale_artifact" -Needle '"case": "stale_artifact_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_substituted_vm_report" -Needle '"case": "substituted_vm_report_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_previous_boot" -Needle '"case": "previous_boot_write_request"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_schema_mismatch" -Needle '"case": "write_request_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_missing_grant" -Needle '"case": "missing_computed_grant_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_attestation_mismatch" -Needle '"case": "local_attestation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_approval_mismatch" -Needle '"case": "local_approval_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_audit_hash_mismatch" -Needle '"case": "audit_record_service_slot_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_hash_mismatch" -Needle '"case": "rollback_plan_service_slot_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_service_slot_substituted" -Needle '"case": "substituted_service_slot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_recovery_separate" -Needle '"case": "recovery_artifact_loader_requested"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_ledger_available_case" -Needle '"case": "durable_audit_ledger_available_rollback_store_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_available_case" -Needle '"case": "rollback_store_available_durable_audit_ledger_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_denied_case" -Needle '"case": "availability_facts_present_policy_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_status" -Needle '"actual_status": "denied_missing_durable_write_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_reason" -Needle '"actual_reason": "durable_write_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_case" -Needle '"case": "durable_write_policy_available_rollback_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_status" -Needle '"actual_status": "denied_missing_rollback_install_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_reason" -Needle '"actual_reason": "rollback_install_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_case" -Needle '"case": "policy_facts_available_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_status" -Needle '"actual_status": "denied_missing_append_contract"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_reason" -Needle '"actual_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_transaction_missing_case" -Needle '"case": "audit_append_available_rollback_transaction_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_transaction_missing_reason" -Needle '"actual_reason": "rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_case" -Needle '"case": "append_contract_available_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_status" -Needle '"actual_status": "denied_missing_append_intent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_reason" -Needle '"actual_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_payload_envelope_missing_case" -Needle '"case": "append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_payload_envelope_missing_reason" -Needle '"actual_reason": "audit_record_append_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_case" -Needle '"case": "append_intents_available_writer_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_denied" -Needle '"case": "accepted_current_boot_preconditions_write_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_status" -Needle '"actual_status": "denied_missing_durable_write_boundary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_reason" -Needle '"actual_reason": "durable_audit_write_missing_and_rollback_install_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_manifest_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_manifest_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_manifest_json" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_missing_case" -Needle '"case": "missing_retained_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_module_manifest_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_case" -Needle '"case": "accepted_current_boot_manifest_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_stale_case" -Needle '"case": "stale_dropped_manifest_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_stale_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_module_manifest_reference_stale_or_dropped_event_id"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_previous_case" -Needle '"case": "previous_boot_or_unretained_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_previous_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_module_manifest_reference_previous_boot_or_unretained"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_wrong_schema_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_module_manifest_reference_wrong_schema_or_variant"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_substituted_case" -Needle '"case": "substituted_manifest_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_substituted_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_module_manifest_reference_substituted_record"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_hash_case" -Needle '"case": "manifest_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_module_manifest_reference_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_artifact_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_artifact_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_manifest_json" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_count" -Needle '"case_count": 9' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_missing_case" -Needle '"case": "missing_retained_candidate_artifact_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_candidate_artifact_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_case" -Needle '"case": "accepted_current_boot_artifact_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_stale_case" -Needle '"case": "stale_dropped_retained_artifact_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_substituted_case" -Needle '"case": "substituted_artifact_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_hash_case" -Needle '"case": "artifact_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_vm_report_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_vm_report_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_manifest_json" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_count" -Needle '"case_count": 11' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_missing_case" -Needle '"case": "missing_retained_vm_test_report_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_vm_test_report_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_case" -Needle '"case": "accepted_current_boot_report_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_reason" -Needle '"actual": {"status": "retained_hash_reference_only", "reason": "retained_vm_test_report_reference_not_authorizing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_stale_case" -Needle '"case": "stale_dropped_retained_vm_test_report_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_substituted_case" -Needle '"case": "substituted_vm_test_report_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_hash_case" -Needle '"case": "vm_test_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_vm_test_report_reference_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_artifact_case" -Needle '"case": "artifact_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_report_hash_case" -Needle '"case": "vm_test_report_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_report_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_vm_test_report_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_attestation_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_attestation_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_attestation_json" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_count" -Needle '"case_count": 11' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_missing_case" -Needle '"case": "missing_retained_local_attestation_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_local_attestation_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_case" -Needle '"case": "accepted_current_boot_attestation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_reason" -Needle '"actual": {"status": "retained_hash_reference_only", "reason": "retained_local_attestation_reference_not_authorizing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_stale_case" -Needle '"case": "stale_dropped_retained_local_attestation_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_substituted_case" -Needle '"case": "substituted_local_attestation_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_hash_case" -Needle '"case": "local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_local_attestation_reference_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_artifact_case" -Needle '"case": "artifact_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_vm_report_case" -Needle '"case": "vm_report_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_approval_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_approval_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_approval_text" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_count" -Needle '"case_count": 12' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_missing_case" -Needle '"case": "missing_retained_local_approval_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_local_approval_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_case" -Needle '"case": "accepted_current_boot_approval_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_reason" -Needle '"actual": {"status": "retained_hash_reference_only", "reason": "retained_local_approval_reference_not_authorizing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_stale_case" -Needle '"case": "stale_dropped_retained_local_approval_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_substituted_case" -Needle '"case": "substituted_local_approval_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_substituted_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_local_approval_reference_substituted_record"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_hash_case" -Needle '"case": "local_approval_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_local_approval_reference_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_attestation_case" -Needle '"case": "local_attestation_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_retained_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_retained_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_missing_case" -Needle '"case": "missing_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_reason" -Needle '"actual": {"status": "retained_hash_reference_only", "reason": "retained_computed_grant_reference_not_authorizing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_stale_case" -Needle '"case": "stale_dropped_retained_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_stale_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_reference_stale_or_dropped_event_id"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_previous_boot_case" -Needle '"case": "previous_boot_or_unretained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_previous_boot_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_reference_previous_boot_or_unretained"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_wrong_schema_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_reference_wrong_schema_or_variant"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_substituted_case" -Needle '"case": "substituted_retained_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_substituted_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_reference_substituted_record"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_hash_case" -Needle '"case": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_reference_hash_mismatch"}' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_audit_rollback_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_audit_rollback_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_audit_records" -Needle '"durable_audit_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_rollback_plans" -Needle '"rollback_plan_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_slots" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_count" -Needle '"case_count": 23' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_retained_audit_ref" -Needle '"case": "missing_retained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_retained_audit_ref_reason" -Needle '"actual": {"status": "missing", "reason": "retained_audit_rollback_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_stale_retained_audit_ref" -Needle '"case": "stale_dropped_retained_audit_rollback_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_stale_retained_audit_ref_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_reference_stale_or_dropped_event_id"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_previous_retained_audit_ref" -Needle '"case": "previous_boot_or_unretained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_previous_retained_audit_ref_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_reference_previous_boot_or_unretained"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_wrong_schema_retained_audit_ref" -Needle '"case": "retained_audit_rollback_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_wrong_schema_retained_audit_ref_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_reference_wrong_schema_or_variant"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_substituted_retained_audit_ref" -Needle '"case": "substituted_retained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_substituted_retained_audit_ref_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_reference_substituted_record"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_grant_hash_mismatch" -Needle '"case": "retained_audit_rollback_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_grant_hash_mismatch_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_computed_grant_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_audit_hash_mismatch" -Needle '"case": "retained_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_audit_hash_mismatch_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_record_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_rollback_hash_mismatch" -Needle '"case": "retained_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_rollback_hash_mismatch_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_rollback_plan_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_slot_mismatch" -Needle '"case": "retained_audit_rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_slot_mismatch_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_audit_rollback_service_slot_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_audit" -Needle '"case": "missing_durable_audit_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_audit_reason" -Needle '"actual": {"status": "missing", "reason": "durable_audit_write_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback" -Needle '"case": "missing_rollback_plan"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback_reason" -Needle '"actual": {"status": "missing", "reason": "rollback_install_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_audit_schema_mismatch" -Needle '"case": "durable_audit_record_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_audit_schema_reason" -Needle '"actual": {"status": "rejected", "reason": "durable_audit_record_schema_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_schema_mismatch" -Needle '"case": "rollback_plan_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_schema_reason" -Needle '"actual": {"status": "rejected", "reason": "rollback_plan_schema_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_case" -Needle '"case": "valid_audit_and_rollback_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_status" -Needle '"actual": {"status": "validated_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_reason" -Needle '"actual": {"status": "validated_non_authorizing", "reason": "loader_and_service_slot_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_grant_mismatch" -Needle '"case": "audit_retained_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_manifest_mismatch" -Needle '"case": "audit_manifest_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_artifact_mismatch" -Needle '"case": "audit_artifact_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_report_mismatch" -Needle '"case": "audit_vm_report_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_attestation_mismatch" -Needle '"case": "audit_local_attestation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_approval_mismatch" -Needle '"case": "local_approval_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_hash_mismatch" -Needle '"case": "audit_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_artifact_mismatch" -Needle '"case": "rollback_artifact_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_slot_mismatch" -Needle '"case": "rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_slot_reason" -Needle '"actual": {"status": "rejected", "reason": "rollback_service_slot_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_bindings" -Needle '"required_bindings":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_service_slot_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_service_slot_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_slots" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_count" -Needle '"case_count": 13' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_missing_case" -Needle '"case": "missing_retained_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_missing_reason" -Needle '"actual": {"status": "missing", "reason": "retained_service_slot_reservation_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_case" -Needle '"case": "accepted_current_boot_reservation_still_denied"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_status" -Needle '"actual": {"status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_stale_case" -Needle '"case": "stale_dropped_retained_service_slot_reservation_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_stale_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_stale_or_dropped_event_id"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_wrong_schema_case" -Needle '"case": "retained_service_slot_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_wrong_schema_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_wrong_schema_or_variant"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_substituted_case" -Needle '"case": "substituted_retained_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_substituted_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_substituted_record"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_schema_case" -Needle '"case": "retained_service_slot_grant_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_schema_case" -Needle '"case": "retained_service_slot_audit_rollback_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_hash_mismatch" -Needle '"case": "retained_service_slot_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_computed_grant_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_hash_mismatch" -Needle '"case": "retained_service_slot_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_audit_record_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rollback_hash_mismatch" -Needle '"case": "retained_service_slot_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rollback_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_rollback_plan_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_mismatch" -Needle '"case": "retained_service_slot_inventory_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_pre_load_inventory_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_slot_mismatch" -Needle '"case": "retained_service_slot_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_slot_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_service_slot_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_hash_mismatch" -Needle '"case": "retained_service_slot_reservation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_hash_reason" -Needle '"actual": {"status": "rejected", "reason": "retained_service_slot_reservation_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_loader_runtime_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_loader_runtime_selftest"
    $moduleLoadGateRuntimeSelftestResponse = Get-LastAgentResponseJson -Method "module.load_gate_loader_runtime_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_no_descriptor" -Needle '"external_artifact_intake_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_no_load" -Needle '"artifact_load_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_no_slots" -Needle '"service_slot_allocation_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    $moduleLoadGateRuntimeSelftestSourceCount = [int]$moduleLoadGateRuntimeSelftestResponse.facts.source_fact_count
    $moduleLoadGateRuntimeSelftestSourceCountMatches = $moduleLoadGateRuntimeSelftestSourceCount -eq 10
    Add-Predicate -Name "protocol:module_load_gate_loader_runtime_selftest_source_count" -Expected 10 -Passed $moduleLoadGateRuntimeSelftestSourceCountMatches -Actual $moduleLoadGateRuntimeSelftestSourceCount
    if (-not $moduleLoadGateRuntimeSelftestSourceCountMatches) {
        throw "Expected 10 module.load_gate_loader_runtime_selftest source facts, got $moduleLoadGateRuntimeSelftestSourceCount"
    }
    $moduleLoadGateRuntimeSelftestSourceMapComplete = [bool]$moduleLoadGateRuntimeSelftestResponse.facts.source_fact_map_complete
    Add-Predicate -Name "protocol:module_load_gate_loader_runtime_selftest_source_map_complete" -Expected $true -Passed $moduleLoadGateRuntimeSelftestSourceMapComplete -Actual $moduleLoadGateRuntimeSelftestSourceMapComplete
    if (-not $moduleLoadGateRuntimeSelftestSourceMapComplete) {
        throw "Expected module.load_gate_loader_runtime_selftest source fact map to be complete"
    }
    foreach ($source in $loaderRuntimeAggregateSources) {
        $matchingSource = @($moduleLoadGateRuntimeSelftestResponse.facts.source_fact_map | Where-Object { $_.source_method -eq $source.Method -and $_.source_fact_locator -eq $source.Locator })
        $sourcePresent = $matchingSource.Count -eq 1
        Add-Predicate -Name ("protocol:module_load_gate_loader_runtime_selftest_" + $source.Suffix + "_source_binding") -Expected ($source.Method + " -> " + $source.Locator) -Passed $sourcePresent -Actual ($matchingSource | ConvertTo-Json -Compress -Depth 6)
        if (-not $sourcePresent) {
            throw ("Expected module.load_gate_loader_runtime_selftest source binding " + $source.Method + " -> " + $source.Locator)
        }
    }
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_missing_manifest_case" -Needle '"case": "missing_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_missing_manifest_reason" -Needle '"actual": {"status": "denied_missing_retained_module_evidence", "reason": "retained_module_manifest_reference_missing"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_rejected_artifact_case" -Needle '"case": "rejected_artifact_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_rejected_artifact_reason" -Needle '"actual": {"status": "denied_missing_retained_module_evidence", "reason": "retained_candidate_artifact_reference_hash_mismatch"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_missing_slot_case" -Needle '"case": "missing_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_missing_slot_reason" -Needle '"actual": {"status": "denied_missing_retained_module_evidence", "reason": "ram_only_service_slot_unallocated"}' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_rejected_slot_case" -Needle '"case": "rejected_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_allocator_authority_case" -Needle '"case": "all_retained_evidence_ready_allocator_authority_boundary_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_allocator_authority_status" -Needle '"actual": {"status": "denied_allocator_authority_not_granted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_allocation_intent" -Needle '"raios.service_slot_allocation_intent.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_policy_decision" -Needle '"raios.service_slot_allocator_policy_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_registry_write" -Needle '"raios.service_slot_registry_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_loader_contract" -Needle '"raios.module_loader_runtime_contract.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_health_monitor" -Needle '"raios.service_health_monitor_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_cleanup_authority" -Needle '"raios.service_unload_cleanup_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_authority_decision" -Needle '"raios.module_service_slot_allocator_authority_decision.v0"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_registry_commit_gate" -Needle '"raios.service_slot_registry_write_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_execution_commit_gate" -Needle '"raios.module_loader_runtime_execution_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_descriptor_intake_boundary" -Needle '"raios.module_loader_descriptor_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_artifact_byte_intake_boundary" -Needle '"raios.module_loader_artifact_byte_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_required_loader_identity" -Needle '"raios.module_loader_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_can_load_false" -Needle '"load_authorization_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_loader_runtime_selftest_load_attempted_false" -Needle '"load_attempt_count": 0' -TimeoutSeconds 1
