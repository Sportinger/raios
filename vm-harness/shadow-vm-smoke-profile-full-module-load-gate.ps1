    $moduleLoaderRuntimeGateSources = @(
        @{ Suffix = "identity"; Method = "module.loader_identity"; Locator = "module.loader_identity.loader_identity" },
        @{ Suffix = "artifact_hash"; Method = "module.loader_artifact_hash_binding"; Locator = "module.loader_artifact_hash_binding.artifact_hash_binding" },
        @{ Suffix = "entrypoint"; Method = "module.loader_entrypoint_abi"; Locator = "module.loader_entrypoint_abi.entrypoint_abi" },
        @{ Suffix = "address_space"; Method = "module.loader_address_space_boundary"; Locator = "module.loader_address_space_boundary.address_space_boundary" },
        @{ Suffix = "memory_map"; Method = "module.loader_memory_map_constraints"; Locator = "module.loader_memory_map_constraints.memory_map_constraints" },
        @{ Suffix = "capability_table"; Method = "module.loader_capability_import_table"; Locator = "module.loader_capability_import_table.capability_import_table" },
        @{ Suffix = "service_slot"; Method = "module.loader_service_slot_binding"; Locator = "module.loader_service_slot_binding.service_slot_binding" },
        @{ Suffix = "health"; Method = "module.loader_health_state_hooks"; Locator = "module.loader_health_state_hooks.health_state_hooks" },
        @{ Suffix = "rollback"; Method = "module.loader_rollback_hooks"; Locator = "module.loader_rollback_hooks.rollback_hooks" },
        @{ Suffix = "write_boundary"; Method = "module.loader_audit_rollback_write_boundary_binding"; Locator = "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding" },
        @{ Suffix = "receiver_preflight"; Method = "module.distribution_receiver_identity_load_preflight"; Locator = "module.load_ephemeral.receiver_identity_load_preflight" }
    )

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
    $moduleFinalLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $moduleLoadGateLoaderRuntime = ($moduleFinalLoadResponse.evidence | Where-Object { $_.id -eq "loader_runtime" }).facts
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"outcome": "denied", "reason": "retained_module_manifest_reference_not_authorizing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_gate_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:mutating_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_mode_ram_only" -Needle '"load_mode": "ram_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_target" -Needle '"target": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_retained" -Needle '"reason": "retained_module_manifest_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_reference" -Needle '"id": "module_manifest", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_present" -Needle '"record_schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_status" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_event_id" -Needle '"source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_retained" -Needle '"reason": "retained_candidate_artifact_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_reference" -Needle '"id": "candidate_artifact", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_present" -Needle '"record_schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_status" -Needle '"id": "candidate_artifact", "kind": "retained_reference", "status": "verified", "reason": "retained_candidate_artifact_reference_not_authorizing"' -TimeoutSeconds 1
    $moduleLoadArtifactEventId = [string](($moduleFinalLoadResponse.evidence | Where-Object { $_.id -eq "candidate_artifact" }).source_event_id)
    $moduleLoadArtifactEventIdMatches = $moduleLoadArtifactEventId -eq $moduleArtifactRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_artifact_event_id" -Expected $moduleArtifactRetainedReferenceEventId -Passed $moduleLoadArtifactEventIdMatches -Actual $moduleLoadArtifactEventId
    if (-not $moduleLoadArtifactEventIdMatches) {
        throw "Expected retained artifact event id $moduleArtifactRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadArtifactEventId"
    }
    Assert-LogContains -Name "policy:module_retained_artifact_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_retained" -Needle '"reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_reference" -Needle '"id": "vm_test_report", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_present" -Needle '"record_schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_status" -Needle '"id": "vm_test_report", "kind": "retained_reference", "status": "verified", "reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    $moduleLoadVmReportEventId = [string](($moduleFinalLoadResponse.evidence | Where-Object { $_.id -eq "vm_test_report" }).source_event_id)
    $moduleLoadVmReportEventIdMatches = $moduleLoadVmReportEventId -eq $moduleVmReportRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_vm_report_event_id" -Expected $moduleVmReportRetainedReferenceEventId -Passed $moduleLoadVmReportEventIdMatches -Actual $moduleLoadVmReportEventId
    if (-not $moduleLoadVmReportEventIdMatches) {
        throw "Expected retained VM report event id $moduleVmReportRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadVmReportEventId"
    }
    Assert-LogContains -Name "policy:module_retained_vm_report_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_retained" -Needle '"reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_reference" -Needle '"id": "local_attestation", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_present" -Needle '"record_schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_status" -Needle '"id": "local_attestation", "kind": "retained_reference", "status": "verified", "reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_event_id" -Needle '"id": "local_attestation", "kind": "retained_reference", "status": "verified", "reason": "retained_local_attestation_reference_not_authorizing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_reference" -Needle '"id": "computed_capability_grant", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_present" -Needle '"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_status" -Needle '"id": "computed_capability_grant", "kind": "retained_reference", "status": "verified", "reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_event_id" -Needle '"id": "computed_capability_grant", "kind": "retained_reference", "status": "verified", "reason": "retained_computed_grant_reference_not_authorizing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_retained" -Needle '"reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_reference" -Needle '"id": "local_approval", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_present" -Needle '"record_schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_status" -Needle '"id": "local_approval", "kind": "retained_reference", "status": "verified", "reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    $moduleLoadApprovalEventId = [string](($moduleFinalLoadResponse.evidence | Where-Object { $_.id -eq "local_approval" }).source_event_id)
    $moduleLoadApprovalEventIdMatches = $moduleLoadApprovalEventId -eq $moduleApprovalRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_approval_event_id" -Expected $moduleApprovalRetainedReferenceEventId -Passed $moduleLoadApprovalEventIdMatches -Actual $moduleLoadApprovalEventId
    if (-not $moduleLoadApprovalEventIdMatches) {
        throw "Expected retained approval event id $moduleApprovalRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadApprovalEventId"
    }
    Assert-LogContains -Name "policy:module_retained_approval_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_retained" -Needle '"reason": "rollback_install_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_retained" -Needle '"reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_reference" -Needle '"id": "durable_audit_record", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_present" -Needle '"reference_schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_status" -Needle '"id": "durable_audit_record", "kind": "retained_reference", "status": "verified", "reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_event_id" -Needle '"id": "durable_audit_record", "kind": "retained_reference", "status": "verified", "reason": "durable_audit_write_missing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_rollback_hash" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_unavailable" -Needle '"reason": "module_loader_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_retained" -Needle '"reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_reference" -Needle '"id": "service_slot", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_present" -Needle '"record_schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_status" -Needle '"status_detail": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_event_id" -Needle '"id": "service_slot", "kind": "retained_reference", "status": "verified", "reason": "retained_service_slot_reservation_not_allocated", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_state" -Needle '"reason": "service_slot_allocator_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_schema" -Needle '"record_schema": "raios.module_service_slot_allocator_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_source" -Needle '"source_method": "module.service_slot_allocator"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_status" -Needle '"readiness_status": "denied_allocator_authority_not_granted"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_reason" -Needle '"readiness_reason": "service_slot_allocator_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_authority_boundary" -Needle '"allocator_authority_boundary": {"record_schema": "raios.module_service_slot_allocator_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_authority_schema" -Needle '"record_schema": "raios.module_service_slot_allocator_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocation_intent_boundary" -Needle '"allocation_intent_boundary": {"record_schema": "raios.service_slot_allocation_intent.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocation_intent_schema" -Needle '"record_schema": "raios.service_slot_allocation_intent.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocation_intent_reason" -Needle '"reason": "service_slot_allocation_intent_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_authority_inputs" -Needle '"authority_input_boundaries": {"policy_decision": {"record_schema": "raios.service_slot_allocator_policy_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_policy_decision_schema" -Needle '"record_schema": "raios.service_slot_allocator_policy_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_policy_decision_reason" -Needle '"reason": "service_slot_allocator_policy_decision_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_write_schema" -Needle '"record_schema": "raios.service_slot_registry_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_write_reason" -Needle '"reason": "service_slot_registry_write_authority_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_contract_schema" -Needle '"record_schema": "raios.module_loader_runtime_contract.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_contract_reason" -Needle '"reason": "module_loader_runtime_contract_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_health_monitor_schema" -Needle '"record_schema": "raios.service_health_monitor_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_health_monitor_reason" -Needle '"reason": "service_health_monitor_binding_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_unload_cleanup_authority_schema" -Needle '"record_schema": "raios.service_unload_cleanup_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_unload_cleanup_authority_reason" -Needle '"reason": "service_unload_cleanup_authority_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_authority_decision" -Needle '"authority_decision": {"record_schema": "raios.module_service_slot_allocator_authority_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_authority_decision_schema" -Needle '"record_schema": "raios.module_service_slot_allocator_authority_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_authority_decision_reason" -Needle '"reason": "service_slot_allocator_authority_decision_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_commit_gate" -Needle '"registry_write_commit_gate": {"record_schema": "raios.service_slot_registry_write_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_commit_gate_schema" -Needle '"record_schema": "raios.service_slot_registry_write_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_commit_gate_reason" -Needle '"reason": "service_slot_registry_write_commit_gate_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_schema" -Needle '"record_schema": "raios.module_loader_runtime_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_state" -Needle '"id": "loader_runtime", "kind": "readiness", "status": "present", "reason": "service_slot_allocator_authority_boundary_non_authorizing", "source_event_id": null, "classification": "local_only", "facts": {"status_detail": "blocked_by_service_slot_allocator_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_status" -Needle '"record_schema": "raios.module_loader_runtime_readiness.v0", "state": "blocked_by_service_slot_allocator_authority", "readiness_status": "denied_allocator_authority_not_granted"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_reason" -Needle '"state": "blocked_by_service_slot_allocator_authority", "readiness_status": "denied_allocator_authority_not_granted", "readiness_reason": "service_slot_allocator_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_retained_evidence" -Needle '"retained_module_evidence_state": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_ready_false" -Needle '"retained_module_evidence_reason": "retained_module_evidence_available", "service_slot_allocator_ready": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_commit_gate" -Needle '"execution_commit_gate": {"record_schema": "raios.module_loader_runtime_execution_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_commit_gate_schema" -Needle '"record_schema": "raios.module_loader_runtime_execution_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_commit_gate_reason" -Needle '"reason": "module_loader_runtime_execution_commit_gate_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_intake_boundary" -Needle '"descriptor_intake_boundary": {"record_schema": "raios.module_loader_descriptor_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_intake_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_intake_boundary_reason" -Needle '"reason": "module_loader_descriptor_intake_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_byte_intake_boundary" -Needle '"artifact_byte_intake_boundary": {"record_schema": "raios.module_loader_artifact_byte_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_byte_intake_boundary_schema" -Needle '"record_schema": "raios.module_loader_artifact_byte_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_byte_intake_boundary_reason" -Needle '"reason": "module_loader_artifact_byte_intake_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_authorization_boundary" -Needle '"execution_authorization_boundary": {"record_schema": "raios.module_loader_execution_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_authorization_boundary_schema" -Needle '"record_schema": "raios.module_loader_execution_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_authorization_boundary_reason" -Needle '"reason": "module_loader_execution_authorization_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_registry_mutation_boundary" -Needle '"service_registry_mutation_boundary": {"record_schema": "raios.module_loader_service_registry_mutation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_registry_mutation_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_registry_mutation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_registry_mutation_boundary_reason" -Needle '"reason": "module_loader_service_registry_mutation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_load_attempt_boundary" -Needle '"load_attempt_boundary": {"record_schema": "raios.module_loader_load_attempt_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_load_attempt_boundary_schema" -Needle '"record_schema": "raios.module_loader_load_attempt_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_load_attempt_boundary_reason" -Needle '"reason": "module_loader_load_attempt_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_load_boundary" -Needle '"artifact_load_boundary": {"record_schema": "raios.module_loader_artifact_load_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_load_boundary_schema" -Needle '"record_schema": "raios.module_loader_artifact_load_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_artifact_load_boundary_reason" -Needle '"reason": "module_loader_artifact_load_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_mapping_boundary" -Needle '"executable_mapping_boundary": {"record_schema": "raios.module_loader_executable_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_mapping_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_mapping_boundary_reason" -Needle '"reason": "module_loader_executable_mapping_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_entrypoint_transfer_boundary" -Needle '"entrypoint_transfer_boundary": {"record_schema": "raios.module_loader_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_entrypoint_transfer_boundary_schema" -Needle '"record_schema": "raios.module_loader_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_entrypoint_transfer_boundary_reason" -Needle '"reason": "module_loader_entrypoint_transfer_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_boundary" -Needle '"service_start_boundary": {"record_schema": "raios.module_loader_service_start_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_start_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_boundary_reason" -Needle '"reason": "module_loader_service_start_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_health_binding_boundary" -Needle '"service_health_binding_boundary": {"record_schema": "raios.module_loader_service_health_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_health_binding_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_health_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_health_binding_boundary_reason" -Needle '"reason": "module_loader_service_health_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_running_state_boundary" -Needle '"service_running_state_boundary": {"record_schema": "raios.module_loader_service_running_state_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_running_state_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_running_state_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_running_state_boundary_reason" -Needle '"reason": "module_loader_service_running_state_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_audit_boundary" -Needle '"service_start_audit_boundary": {"record_schema": "raios.module_loader_service_start_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_audit_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_start_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_start_audit_boundary_reason" -Needle '"reason": "module_loader_service_start_audit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_unload_cleanup_boundary" -Needle '"service_unload_cleanup_boundary": {"record_schema": "raios.module_loader_service_unload_cleanup_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_unload_cleanup_boundary_schema" -Needle '"record_schema": "raios.module_loader_service_unload_cleanup_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_service_unload_cleanup_boundary_reason" -Needle '"reason": "module_loader_service_unload_cleanup_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_live_load_commit_boundary" -Needle '"live_load_commit_boundary": {"record_schema": "raios.module_loader_live_load_commit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_live_load_commit_boundary_schema" -Needle '"record_schema": "raios.module_loader_live_load_commit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_live_load_commit_boundary_reason" -Needle '"reason": "module_loader_live_load_commit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_audit_boundary" -Needle '"commit_audit_boundary": {"record_schema": "raios.module_loader_commit_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_audit_boundary_schema" -Needle '"record_schema": "raios.module_loader_commit_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_audit_boundary_reason" -Needle '"reason": "module_loader_commit_audit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_rollback_boundary" -Needle '"commit_rollback_boundary": {"record_schema": "raios.module_loader_commit_rollback_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_rollback_boundary_schema" -Needle '"record_schema": "raios.module_loader_commit_rollback_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_rollback_boundary_reason" -Needle '"reason": "module_loader_commit_rollback_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_result_boundary" -Needle '"commit_result_boundary": {"record_schema": "raios.module_loader_commit_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_result_boundary_schema" -Needle '"record_schema": "raios.module_loader_commit_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_commit_result_boundary_reason" -Needle '"reason": "module_loader_commit_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_acceptance_authority_boundary" -Needle '"descriptor_acceptance_authority_boundary": {"record_schema": "raios.module_loader_descriptor_acceptance_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_acceptance_authority_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_acceptance_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_acceptance_authority_boundary_reason" -Needle '"reason": "module_loader_descriptor_acceptance_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_contract_boundary" -Needle '"descriptor_parser_contract_boundary": {"record_schema": "raios.module_loader_descriptor_parser_contract_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_contract_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_parser_contract_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_contract_boundary_reason" -Needle '"reason": "module_loader_descriptor_parser_contract_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_result_boundary" -Needle '"descriptor_parser_result_boundary": {"record_schema": "raios.module_loader_descriptor_parser_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_result_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_parser_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_parser_result_boundary_reason" -Needle '"reason": "module_loader_descriptor_parser_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_schema_validation_boundary" -Needle '"descriptor_schema_validation_boundary": {"record_schema": "raios.module_loader_descriptor_schema_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_schema_validation_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_schema_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_schema_validation_boundary_reason" -Needle '"reason": "module_loader_descriptor_schema_validation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_capability_validation_boundary" -Needle '"descriptor_capability_validation_boundary": {"record_schema": "raios.module_loader_descriptor_capability_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_capability_validation_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_capability_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_capability_validation_boundary_reason" -Needle '"reason": "module_loader_descriptor_capability_validation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_load_plan_boundary" -Needle '"descriptor_load_plan_boundary": {"record_schema": "raios.module_loader_descriptor_load_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_load_plan_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_load_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_load_plan_boundary_reason" -Needle '"reason": "module_loader_descriptor_load_plan_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_authority_boundary" -Needle '"executable_load_plan_authority_boundary": {"record_schema": "raios.module_loader_executable_load_plan_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_authority_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_load_plan_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_authority_boundary_reason" -Needle '"reason": "module_loader_executable_load_plan_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_result_boundary" -Needle '"executable_load_plan_result_boundary": {"record_schema": "raios.module_loader_executable_load_plan_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_result_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_load_plan_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_load_plan_result_boundary_reason" -Needle '"reason": "module_loader_executable_load_plan_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_image_layout_boundary" -Needle '"executable_image_layout_boundary": {"record_schema": "raios.module_loader_executable_image_layout_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_image_layout_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_image_layout_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_image_layout_boundary_reason" -Needle '"reason": "module_loader_executable_image_layout_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_plan_boundary" -Needle '"executable_page_mapping_plan_boundary": {"record_schema": "raios.module_loader_executable_page_mapping_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_plan_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_page_mapping_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_plan_boundary_reason" -Needle '"reason": "module_loader_executable_page_mapping_plan_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_boundary" -Needle '"executable_page_mapping_boundary": {"record_schema": "raios.module_loader_executable_page_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_page_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_page_mapping_boundary_reason" -Needle '"reason": "module_loader_executable_page_mapping_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_executable_page_binding_boundary" -Needle '"descriptor_executable_page_binding_boundary": {"record_schema": "raios.module_loader_descriptor_executable_page_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_executable_page_binding_boundary_schema" -Needle '"record_schema": "raios.module_loader_descriptor_executable_page_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_descriptor_executable_page_binding_boundary_reason" -Needle '"reason": "module_loader_descriptor_executable_page_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_binding_boundary" -Needle '"executable_entrypoint_binding_boundary": {"record_schema": "raios.module_loader_executable_entrypoint_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_binding_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_entrypoint_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_binding_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary" -Needle '"executable_entrypoint_transfer_authorization_boundary": {"record_schema": "raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_transfer_authorization_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_boundary" -Needle '"executable_entrypoint_transfer_boundary": {"record_schema": "raios.module_loader_executable_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_transfer_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_transfer_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_handoff_boundary" -Needle '"executable_entrypoint_handoff_boundary": {"record_schema": "raios.module_loader_executable_entrypoint_handoff_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_handoff_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_entrypoint_handoff_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_handoff_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_handoff_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_invocation_boundary" -Needle '"executable_entrypoint_invocation_boundary": {"record_schema": "raios.module_loader_executable_entrypoint_invocation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_invocation_boundary_schema" -Needle '"record_schema": "raios.module_loader_executable_entrypoint_invocation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_executable_entrypoint_invocation_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_invocation_boundary_non_authorizing"' -TimeoutSeconds 1
    $moduleLoadGateInvocationBoundary = $moduleLoadGateLoaderRuntime.executable_entrypoint_invocation_boundary
    $moduleLoadGateInvocationBoundaryChecks = @(
        @{ Suffix = "executable_entrypoint_invocation_boundary_status_scoped"; Expected = "defined_non_authorizing"; Actual = [string]$moduleLoadGateInvocationBoundary.status_detail },
        @{ Suffix = "executable_entrypoint_invocation_boundary_reason_scoped"; Expected = "module_loader_executable_entrypoint_invocation_boundary_non_authorizing"; Actual = [string]$moduleLoadGateInvocationBoundary.reason },
        @{ Suffix = "executable_entrypoint_invocation_boundary_present_scoped"; Expected = $true; Actual = [bool]$moduleLoadGateInvocationBoundary.present },
        @{ Suffix = "executable_entrypoint_invocation_boundary_chain_complete_scoped"; Expected = $true; Actual = [bool]$moduleLoadGateInvocationBoundary.source_chain_complete },
        @{ Suffix = "executable_entrypoint_invocation_boundary_binding_present_scoped"; Expected = $true; Actual = [bool]$moduleLoadGateInvocationBoundary.executable_entrypoint_binding_boundary_present },
        @{ Suffix = "executable_entrypoint_invocation_boundary_mapping_present_scoped"; Expected = $true; Actual = [bool]$moduleLoadGateInvocationBoundary.executable_page_mapping_boundary_present },
        @{ Suffix = "executable_entrypoint_invocation_boundary_artifact_intake_present_scoped"; Expected = $true; Actual = [bool]$moduleLoadGateInvocationBoundary.artifact_byte_intake_boundary_present }
    )
    foreach ($check in $moduleLoadGateInvocationBoundaryChecks) {
        $passed = ($null -ne $moduleLoadGateInvocationBoundary) -and $check.Actual -eq $check.Expected
        Add-Predicate -Name ("policy:module_loader_runtime_" + $check.Suffix) -Expected ([string]$check.Expected) -Passed $passed -Actual ([string]$check.Actual)
        if (-not $passed) {
            throw ("Expected module.load_ephemeral " + $check.Suffix + " to be " + [string]$check.Expected + ", got " + [string]$check.Actual)
        }
    }
    Assert-LogContains -Name "policy:module_loader_runtime_fact_schema" -Needle '"schema": "raios.module_loader_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_fact_missing" -Needle '"reason": "module_loader_identity_missing"' -TimeoutSeconds 1
    $moduleLoadGateSourceCount = [int]$moduleLoadGateLoaderRuntime.source_fact_count
    $moduleLoadGateSourceCountMatches = $moduleLoadGateSourceCount -eq 11
    Add-Predicate -Name "policy:module_loader_runtime_source_count" -Expected 11 -Passed $moduleLoadGateSourceCountMatches -Actual $moduleLoadGateSourceCount
    if (-not $moduleLoadGateSourceCountMatches) {
        throw "Expected 11 module.load_ephemeral loader-runtime source facts, got $moduleLoadGateSourceCount"
    }
    $moduleLoadGateSourceMapComplete = [bool]$moduleLoadGateLoaderRuntime.source_fact_map_complete
    Add-Predicate -Name "policy:module_loader_runtime_source_map_complete" -Expected $true -Passed $moduleLoadGateSourceMapComplete -Actual $moduleLoadGateSourceMapComplete
    if (-not $moduleLoadGateSourceMapComplete) {
        throw "Expected module.load_ephemeral loader-runtime source fact map to be complete"
    }
    foreach ($source in $moduleLoaderRuntimeGateSources) {
        $matchingSource = @($moduleLoadGateLoaderRuntime.source_fact_map | Where-Object { $_.source_method -eq $source.Method -and $_.source_fact_locator -eq $source.Locator })
        $sourcePresent = $matchingSource.Count -eq 1
        Add-Predicate -Name ("policy:module_loader_runtime_" + $source.Suffix + "_source_binding") -Expected ($source.Method + " -> " + $source.Locator) -Passed $sourcePresent -Actual ($matchingSource | ConvertTo-Json -Compress)
        if (-not $sourcePresent) {
            throw ("Expected module.load_ephemeral loader-runtime source binding " + $source.Method + " -> " + $source.Locator)
        }
    }
    Assert-LogContains -Name "policy:module_manifest_required" -Needle "raios.module_manifest.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_sha256_required" -Needle "candidate_artifact_sha256" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:vm_report_required" -Needle "raios.vm_test_report.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:local_attestation_required" -Needle "raios.local_attestation.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_required" -Needle "computed_capability_grant" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_required" -Needle "local_approval" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_required" -Needle "raios.audit_record.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_required" -Needle "rollback_plan" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_required" -Needle "ram_only_service_slot" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_required" -Needle "raios.module_service_slot_allocator_readiness.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_authority_required" -Needle "raios.module_service_slot_allocator_authority.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocation_intent_required" -Needle "raios.service_slot_allocation_intent.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_policy_decision_required" -Needle "raios.service_slot_allocator_policy_decision.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_write_required" -Needle "raios.service_slot_registry_write_authority.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_contract_required" -Needle "raios.module_loader_runtime_contract.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_health_monitor_required" -Needle "raios.service_health_monitor_binding.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:service_unload_cleanup_authority_required" -Needle "raios.service_unload_cleanup_authority.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_authority_decision_required" -Needle "raios.module_service_slot_allocator_authority_decision.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_registry_commit_gate_required" -Needle "raios.service_slot_registry_write_commit_gate.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_execution_commit_gate_required" -Needle "raios.module_loader_runtime_execution_commit_gate.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_intake_boundary_required" -Needle "raios.module_loader_descriptor_intake_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_artifact_byte_intake_boundary_required" -Needle "raios.module_loader_artifact_byte_intake_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_execution_authorization_boundary_required" -Needle "raios.module_loader_execution_authorization_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_registry_mutation_boundary_required" -Needle "raios.module_loader_service_registry_mutation_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_load_attempt_boundary_required" -Needle "raios.module_loader_load_attempt_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_artifact_load_boundary_required" -Needle "raios.module_loader_artifact_load_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_mapping_boundary_required" -Needle "raios.module_loader_executable_mapping_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_entrypoint_transfer_boundary_required" -Needle "raios.module_loader_entrypoint_transfer_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_start_boundary_required" -Needle "raios.module_loader_service_start_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_health_binding_boundary_required" -Needle "raios.module_loader_service_health_binding_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_running_state_boundary_required" -Needle "raios.module_loader_service_running_state_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_start_audit_boundary_required" -Needle "raios.module_loader_service_start_audit_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_service_unload_cleanup_boundary_required" -Needle "raios.module_loader_service_unload_cleanup_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_live_load_commit_boundary_required" -Needle "raios.module_loader_live_load_commit_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_commit_audit_boundary_required" -Needle "raios.module_loader_commit_audit_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_commit_rollback_boundary_required" -Needle "raios.module_loader_commit_rollback_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_commit_result_boundary_required" -Needle "raios.module_loader_commit_result_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_acceptance_authority_boundary_required" -Needle "raios.module_loader_descriptor_acceptance_authority_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_parser_contract_boundary_required" -Needle "raios.module_loader_descriptor_parser_contract_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_parser_result_boundary_required" -Needle "raios.module_loader_descriptor_parser_result_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_schema_validation_boundary_required" -Needle "raios.module_loader_descriptor_schema_validation_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_capability_validation_boundary_required" -Needle "raios.module_loader_descriptor_capability_validation_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_load_plan_boundary_required" -Needle "raios.module_loader_descriptor_load_plan_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_load_plan_authority_boundary_required" -Needle "raios.module_loader_executable_load_plan_authority_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_load_plan_result_boundary_required" -Needle "raios.module_loader_executable_load_plan_result_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_image_layout_boundary_required" -Needle "raios.module_loader_executable_image_layout_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_page_mapping_plan_boundary_required" -Needle "raios.module_loader_executable_page_mapping_plan_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_page_mapping_boundary_required" -Needle "raios.module_loader_executable_page_mapping_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_descriptor_executable_page_binding_boundary_required" -Needle "raios.module_loader_descriptor_executable_page_binding_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_entrypoint_binding_boundary_required" -Needle "raios.module_loader_executable_entrypoint_binding_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_entrypoint_transfer_authorization_boundary_required" -Needle "raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_entrypoint_transfer_boundary_required" -Needle "raios.module_loader_executable_entrypoint_transfer_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_entrypoint_handoff_boundary_required" -Needle "raios.module_loader_executable_entrypoint_handoff_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_executable_entrypoint_invocation_boundary_required" -Needle "raios.module_loader_executable_entrypoint_invocation_boundary.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_required" -Needle "raios.module_loader_runtime_readiness.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_schema" -Needle '"record_schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_status" -Needle '"status": "required_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_schema" -Needle '"record_schema": "raios.audit_record.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_plan_schema" -Needle '"record_schema": "raios.rollback_plan.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_retained_reference_required" -Needle '"retained_computed_grant_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_retained_approval_reference_required" -Needle '"retained_local_approval_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_slot_id_retained" -Needle "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_hash_retained" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_requirement_retained" -Needle '"id": "service_slot", "kind": "retained_reference", "status": "verified", "reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_hash_retained" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_artifact_hash_retained" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_hash_retained" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_inventory_unchanged" -Needle '"service_registry_mutation_boundary": {"record_schema": "raios.module_loader_service_registry_mutation_boundary.v0", "record_id": "module_loader_service_registry_mutation_boundary", "source_event_id": null, "status_detail": "defined_non_authorizing", "reason": "module_loader_service_registry_mutation_boundary_non_authorizing"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_load_gate"
