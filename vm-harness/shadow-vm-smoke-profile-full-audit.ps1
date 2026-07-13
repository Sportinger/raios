    Assert-LogContains -Name "protocol:module_manifest_audit_source" -Needle '"source_method": "module.manifest_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_kind" -Needle '"kind": "module.manifest_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_hash" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_source" -Needle '"source_method": "module.artifact_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_kind" -Needle '"kind": "module.artifact_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_hash" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_source" -Needle '"source_method": "module.vm_report_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_kind" -Needle '"kind": "module.vm_test_report_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_source" -Needle '"source_method": "module.attestation_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_kind" -Needle '"kind": "module.local_attestation_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_hash" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_source" -Needle '"source_method": "module.approval_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_kind" -Needle '"kind": "module.local_approval_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_source" -Needle '"source_method": "module.grant_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_kind" -Needle '"kind": "module.computed_grant_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_capability" -Needle '"requested_capability": "cap.module.grant_diagnostic.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_computed_grant_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_source" -Needle '"source_method": "module.audit_rollback_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_kind" -Needle '"kind": "module.audit_rollback_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_source" -Needle '"source_method": "module.service_slot_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_kind" -Needle '"kind": "module.service_slot_reservation.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_outcome" -Needle '"status_detail": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_binding_schema" -Needle '"binding": {"schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_reservation_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_risk" -Needle '"risk": "modify_ram"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_resource" -Needle '"resource": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_reason" -Needle '"reason": "missing_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_gate" -Needle '"module_load_gate_evaluated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_attestation_checked" -Needle '"local_attestation_reference_checked"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_approval_checked" -Needle '"local_approval_reference_checked"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_audit_required" -Needle '"durable_audit_record_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_rollback_required" -Needle '"rollback_plan_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_rollback_bindings" -Needle '"rollback_bindings_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_inventory" -Needle '"service_inventory_unchanged"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_no_load" -Needle '"load_not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_schema" -Needle '"binding": {"record_schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_status" -Needle '"status": "denied_missing_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_schema" -Needle '"audit_rollback_requirements": {"record_schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_no_load" -Needle '"audit_rollback_requirements": {"record_schema": "raios.module_load_gate_audit_rollback_requirements.v0", "classification": "public", "status": "required_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_state" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_binding" -Needle '"retained_computed_grant_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_reason" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_state" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_binding" -Needle '"retained_vm_test_report_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_reason" -Needle '"reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_state" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_binding" -Needle '"retained_local_attestation_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_reason" -Needle '"reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_rollback_binding" -Needle '"retained_audit_rollback_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_state" -Needle '"durable_audit_record": "retained_hash_reference_only_not_durable"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_rollback_state" -Needle '"rollback_plan": "retained_hash_reference_only_not_installed"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_state" -Needle '"service_slot": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_binding" -Needle '"retained_service_slot_reservation": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_status" -Needle '"status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_reason" -Needle '"reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_rollback_hash" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_evidence_hash" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_event_id" -Needle '"retained_service_slot_reservation_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_state" -Needle '"service_slot_allocator": "defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_schema" -Needle '"service_slot_allocator_readiness": {"schema": "raios.module_service_slot_allocator_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_reason" -Needle '"readiness_reason": "service_slot_allocator_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_authority_boundary" -Needle '"allocator_authority_boundary": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_authority_schema" -Needle '"schema": "raios.module_service_slot_allocator_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocation_intent_boundary" -Needle '"allocation_intent_boundary": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocation_intent_schema" -Needle '"schema": "raios.service_slot_allocation_intent.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocation_intent_reason" -Needle '"reason": "service_slot_allocation_intent_defined_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_authority_inputs" -Needle '"authority_input_boundaries": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_policy_decision_schema" -Needle '"schema": "raios.service_slot_allocator_policy_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_registry_write_schema" -Needle '"schema": "raios.service_slot_registry_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_contract_schema" -Needle '"schema": "raios.module_loader_runtime_contract.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_health_monitor_schema" -Needle '"schema": "raios.service_health_monitor_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_cleanup_authority_schema" -Needle '"schema": "raios.service_unload_cleanup_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_authority_decision" -Needle '"authority_decision": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_authority_decision_schema" -Needle '"schema": "raios.module_service_slot_allocator_authority_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_authority_decision_reason" -Needle '"reason": "service_slot_allocator_authority_decision_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_registry_commit_gate" -Needle '"registry_write_commit_gate": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_registry_commit_gate_schema" -Needle '"schema": "raios.service_slot_registry_write_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_registry_commit_gate_reason" -Needle '"reason": "service_slot_registry_write_commit_gate_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_registry_commit_gate_no_write" -Needle '"authorizes_registry_write": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_state" -Needle '"loader_runtime": "blocked_by_service_slot_allocator_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_schema" -Needle '"loader_runtime_readiness": {"schema": "raios.module_loader_runtime_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_status" -Needle '"readiness_status": "denied_allocator_authority_not_granted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_commit_gate" -Needle '"execution_commit_gate": {"schema": "raios.module_loader_runtime_execution_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_commit_gate_reason" -Needle '"reason": "module_loader_runtime_execution_commit_gate_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_commit_gate_no_execution" -Needle '"authorizes_execution": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_intake_boundary" -Needle '"descriptor_intake_boundary": {"schema": "raios.module_loader_descriptor_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_intake_boundary_reason" -Needle '"reason": "module_loader_descriptor_intake_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_intake_boundary_no_intake" -Needle '"authorizes_descriptor_intake": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_byte_intake_boundary" -Needle '"artifact_byte_intake_boundary": {"schema": "raios.module_loader_artifact_byte_intake_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_byte_intake_boundary_reason" -Needle '"reason": "module_loader_artifact_byte_intake_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_byte_intake_boundary_no_intake" -Needle '"authorizes_artifact_byte_intake": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_authorization_boundary" -Needle '"execution_authorization_boundary": {"schema": "raios.module_loader_execution_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_authorization_boundary_reason" -Needle '"reason": "module_loader_execution_authorization_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_authorization_boundary_no_exec_pages" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_authorization_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_execution_authorization_boundary_no_execution" -Needle '"authorizes_execution": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_registry_mutation_boundary" -Needle '"service_registry_mutation_boundary": {"schema": "raios.module_loader_service_registry_mutation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_registry_mutation_boundary_reason" -Needle '"reason": "module_loader_service_registry_mutation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_registry_mutation_boundary_no_mutation" -Needle '"mutates_service_registry": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_registry_mutation_boundary_no_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_load_attempt_boundary" -Needle '"load_attempt_boundary": {"schema": "raios.module_loader_load_attempt_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_load_attempt_boundary_reason" -Needle '"reason": "module_loader_load_attempt_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_load_attempt_boundary_no_attempt" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_load_boundary" -Needle '"artifact_load_boundary": {"schema": "raios.module_loader_artifact_load_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_load_boundary_reason" -Needle '"reason": "module_loader_artifact_load_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_artifact_load_boundary_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_mapping_boundary" -Needle '"executable_mapping_boundary": {"schema": "raios.module_loader_executable_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_mapping_boundary_reason" -Needle '"reason": "module_loader_executable_mapping_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_mapping_boundary_no_exec_pages" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_entrypoint_transfer_boundary" -Needle '"entrypoint_transfer_boundary": {"schema": "raios.module_loader_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_entrypoint_transfer_boundary_reason" -Needle '"reason": "module_loader_entrypoint_transfer_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_entrypoint_transfer_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_boundary" -Needle '"service_start_boundary": {"schema": "raios.module_loader_service_start_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_boundary_reason" -Needle '"reason": "module_loader_service_start_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_boundary_no_start" -Needle '"starts_service": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_boundary_no_running" -Needle '"marks_service_running": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_health_binding_boundary" -Needle '"service_health_binding_boundary": {"schema": "raios.module_loader_service_health_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_health_binding_boundary_reason" -Needle '"reason": "module_loader_service_health_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_health_binding_boundary_no_records" -Needle '"creates_service_health_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_running_state_boundary" -Needle '"service_running_state_boundary": {"schema": "raios.module_loader_service_running_state_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_running_state_boundary_reason" -Needle '"reason": "module_loader_service_running_state_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_running_state_boundary_no_running" -Needle '"marks_service_running": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_audit_boundary" -Needle '"service_start_audit_boundary": {"schema": "raios.module_loader_service_start_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_audit_boundary_reason" -Needle '"reason": "module_loader_service_start_audit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_start_audit_boundary_no_record" -Needle '"writes_service_start_audit_record": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_unload_cleanup_boundary" -Needle '"service_unload_cleanup_boundary": {"schema": "raios.module_loader_service_unload_cleanup_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_unload_cleanup_boundary_reason" -Needle '"reason": "module_loader_service_unload_cleanup_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_unload_cleanup_boundary_no_unload" -Needle '"unloads_service": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_service_unload_cleanup_boundary_no_cleanup" -Needle '"cleans_up_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_live_load_commit_boundary" -Needle '"live_load_commit_boundary": {"schema": "raios.module_loader_live_load_commit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_live_load_commit_boundary_reason" -Needle '"reason": "module_loader_live_load_commit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_live_load_commit_boundary_no_commit" -Needle '"commits_live_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_audit_boundary" -Needle '"commit_audit_boundary": {"schema": "raios.module_loader_commit_audit_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_audit_boundary_reason" -Needle '"reason": "module_loader_commit_audit_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_audit_boundary_no_record" -Needle '"writes_load_commit_audit_record": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_rollback_boundary" -Needle '"commit_rollback_boundary": {"schema": "raios.module_loader_commit_rollback_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_rollback_boundary_reason" -Needle '"reason": "module_loader_commit_rollback_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_rollback_boundary_no_install" -Needle '"installs_commit_rollback_record": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_result_boundary" -Needle '"commit_result_boundary": {"schema": "raios.module_loader_commit_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_result_boundary_reason" -Needle '"reason": "module_loader_commit_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_commit_result_boundary_no_result" -Needle '"records_load_result": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary" -Needle '"descriptor_acceptance_authority_boundary": {"schema": "raios.module_loader_descriptor_acceptance_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary_reason" -Needle '"reason": "module_loader_descriptor_acceptance_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_acceptance_authority_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary" -Needle '"descriptor_parser_contract_boundary": {"schema": "raios.module_loader_descriptor_parser_contract_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary_reason" -Needle '"reason": "module_loader_descriptor_parser_contract_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_contract_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary" -Needle '"descriptor_parser_result_boundary": {"schema": "raios.module_loader_descriptor_parser_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_reason" -Needle '"reason": "module_loader_descriptor_parser_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_parser_result_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary" -Needle '"descriptor_schema_validation_boundary": {"schema": "raios.module_loader_descriptor_schema_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_reason" -Needle '"reason": "module_loader_descriptor_schema_validation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_schema_validation_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary" -Needle '"descriptor_capability_validation_boundary": {"schema": "raios.module_loader_descriptor_capability_validation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_reason" -Needle '"reason": "module_loader_descriptor_capability_validation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_capability_validation_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary" -Needle '"descriptor_load_plan_boundary": {"schema": "raios.module_loader_descriptor_load_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_reason" -Needle '"reason": "module_loader_descriptor_load_plan_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_load_plan_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary" -Needle '"executable_load_plan_authority_boundary": {"schema": "raios.module_loader_executable_load_plan_authority_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_reason" -Needle '"reason": "module_loader_executable_load_plan_authority_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_authority_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary" -Needle '"executable_load_plan_result_boundary": {"schema": "raios.module_loader_executable_load_plan_result_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_reason" -Needle '"reason": "module_loader_executable_load_plan_result_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_load_plan_result_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary" -Needle '"executable_image_layout_boundary": {"schema": "raios.module_loader_executable_image_layout_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_reason" -Needle '"reason": "module_loader_executable_image_layout_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_image_layout_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary" -Needle '"executable_page_mapping_plan_boundary": {"schema": "raios.module_loader_executable_page_mapping_plan_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_reason" -Needle '"reason": "module_loader_executable_page_mapping_plan_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_plan_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary" -Needle '"executable_page_mapping_boundary": {"schema": "raios.module_loader_executable_page_mapping_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_reason" -Needle '"reason": "module_loader_executable_page_mapping_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_executable_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_page_mapping_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary" -Needle '"descriptor_executable_page_binding_boundary": {"schema": "raios.module_loader_descriptor_executable_page_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_reason" -Needle '"reason": "module_loader_descriptor_executable_page_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_authority" -Needle '"authorizes_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_capability_validation" -Needle '"validates_descriptor_capabilities": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_capability_validated_descriptor" -Needle '"produces_capability_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_schema_validation" -Needle '"validates_descriptor_schema": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_validated_descriptor" -Needle '"produces_validated_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_result" -Needle '"produces_parsed_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_parse" -Needle '"parses_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_descriptor_bytes" -Needle '"accepts_descriptor_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_descriptor_executable_page_binding_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary" -Needle '"executable_entrypoint_binding_boundary": {"schema": "raios.module_loader_executable_entrypoint_binding_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_binding_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_binding_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary" -Needle '"executable_entrypoint_transfer_authorization_boundary": {"schema": "raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_transfer_authorization_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_authorization_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary" -Needle '"executable_entrypoint_transfer_boundary": {"schema": "raios.module_loader_executable_entrypoint_transfer_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_transfer_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_transfer_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary" -Needle '"executable_entrypoint_handoff_boundary": {"schema": "raios.module_loader_executable_entrypoint_handoff_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_handoff_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_entrypoint" -Needle '"jumps_to_entrypoint": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_binding" -Needle '"binds_capability_validated_descriptor_to_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_maps" -Needle '"maps_executable_pages": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_page_mapping_plan" -Needle '"produces_executable_page_mapping_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_image_layout" -Needle '"produces_executable_image_layout": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_load_plan" -Needle '"produces_executable_load_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_handoff_boundary_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_invocation_boundary" -Needle '"executable_entrypoint_invocation_boundary": {"schema": "raios.module_loader_executable_entrypoint_invocation_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_executable_entrypoint_invocation_boundary_reason" -Needle '"reason": "module_loader_executable_entrypoint_invocation_boundary_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_no_load" -Needle '"loader_runtime_readiness": {"schema": "raios.module_loader_runtime_readiness.v0", "scope": "current_boot", "classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_source_count" -Needle '"source_fact_count": 11, "source_fact_map_complete": true, "source_fact_map": ' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_source_map_identity" -Needle '"fact": "loader_identity"' -TimeoutSeconds 1
    $moduleLoaderRuntimeAuditSources = @(
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
    foreach ($source in $moduleLoaderRuntimeAuditSources) {
        Assert-LogContains -Name ("protocol:module_load_audit_loader_runtime_" + $source.Suffix + "_source_method") -Needle ('"source_method": "' + $source.Method + '"') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:module_load_audit_loader_runtime_" + $source.Suffix + "_source_locator") -Needle ('"source_fact_locator": "' + $source.Locator + '"') -TimeoutSeconds 1
    }
    Assert-LogContains -Name "protocol:module_load_audit_binding_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
