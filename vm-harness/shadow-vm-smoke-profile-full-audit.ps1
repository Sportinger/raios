    Assert-LogContains -Name "protocol:module_manifest_audit_source" -Needle '"source_method": "module.manifest_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_kind" -Needle '"kind": "module.manifest_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_hash" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_source" -Needle '"source_method": "module.artifact_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_kind" -Needle '"kind": "module.artifact_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_hash" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_source" -Needle '"source_method": "module.vm_report_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_kind" -Needle '"kind": "module.vm_test_report_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_source" -Needle '"source_method": "module.attestation_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_kind" -Needle '"kind": "module.local_attestation_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_hash" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_source" -Needle '"source_method": "module.approval_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_kind" -Needle '"kind": "module.local_approval_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_source" -Needle '"source_method": "module.grant_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_kind" -Needle '"kind": "module.computed_grant_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_capability" -Needle '"requested_capability": "cap.module.grant_diagnostic.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_computed_grant_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_source" -Needle '"source_method": "module.audit_rollback_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_kind" -Needle '"kind": "module.audit_rollback_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_source" -Needle '"source_method": "module.service_slot_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_kind" -Needle '"kind": "module.service_slot_reservation.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:module_load_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_status" -Needle '"status": "denied_missing_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_schema" -Needle '"audit_rollback_requirements": {"schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_no_load" -Needle '"audit_rollback_requirements": {"schema": "raios.module_load_gate_audit_rollback_requirements.v0", "classification": "public", "status": "required_missing", "writes_enabled": false' -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_state" -Needle '"service_slot_allocator": "missing_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_schema" -Needle '"service_slot_allocator_readiness": {"schema": "raios.module_service_slot_allocator_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_service_slot_allocator_reason" -Needle '"readiness_reason": "service_slot_allocator_runtime_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_state" -Needle '"loader_runtime": "blocked_by_service_slot_allocator_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_schema" -Needle '"loader_runtime_readiness": {"schema": "raios.module_loader_runtime_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_status" -Needle '"readiness_status": "denied_missing_service_slot_allocator_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_no_load" -Needle '"loader_runtime_readiness": {"schema": "raios.module_loader_runtime_readiness.v0", "scope": "current_boot", "classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_loader_runtime_source_count" -Needle '"source_fact_count": 10, "source_fact_map_complete": true, "source_fact_map": ' -TimeoutSeconds 1
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
        @{ Suffix = "write_boundary"; Method = "module.loader_audit_rollback_write_boundary_binding"; Locator = "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding" }
    )
    foreach ($source in $moduleLoaderRuntimeAuditSources) {
        Assert-LogContains -Name ("protocol:module_load_audit_loader_runtime_" + $source.Suffix + "_source_method") -Needle ('"source_method": "' + $source.Method + '"') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:module_load_audit_loader_runtime_" + $source.Suffix + "_source_locator") -Needle ('"source_fact_locator": "' + $source.Locator + '"') -TimeoutSeconds 1
    }
    Assert-LogContains -Name "protocol:module_load_audit_binding_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_source" -Needle '"source_method": "recovery.identity_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_kind" -Needle '"kind": "recovery.artifact_identity_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_hash" -Needle "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_source" -Needle '"source_method": "recovery.trust_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_kind" -Needle '"kind": "recovery.artifact_trust_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_trust.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_identity_event" -Needle "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_hash" -Needle "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_source" -Needle '"source_method": "recovery.vm_test_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_kind" -Needle '"kind": "recovery.artifact_vm_test_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_vm_test.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_trust_event" -Needle "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_hash" -Needle "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_source" -Needle '"source_method": "recovery.local_approval_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_kind" -Needle '"kind": "recovery.artifact_local_approval_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_local_approval.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_vm_event" -Needle "`"retained_recovery_artifact_vm_test_event_id`": `"$recoveryVmTestEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_source" -Needle '"source_method": "recovery.loader_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_kind" -Needle '"kind": "recovery.artifact_loader_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_loader.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_approval_event" -Needle "`"retained_recovery_artifact_local_approval_event_id`": `"$recoveryLocalApprovalEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_hash" -Needle "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_source" -Needle '"source_method": "recovery.rollback_evidence_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_kind" -Needle '"kind": "recovery.artifact_rollback_evidence_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_rollback_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_loader_event" -Needle "`"retained_recovery_artifact_loader_event_id`": `"$recoveryLoaderEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_hash" -Needle "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_no_durable" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_source" -Needle '"source_method": "recovery.lifeline_request_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_kind" -Needle '"kind": "recovery.lifeline_request_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_request.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_rollback_event" -Needle "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_hash" -Needle "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_no_loader" -Needle '"loads_recovery_loader": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_no_slot" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_protocol_audit_source" -Needle '"source_method": "recovery.lifeline_protocol_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_protocol_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_protocol_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_vocab_audit_source" -Needle '"source_method": "recovery.lifeline_command_vocabulary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_vocab_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_vocabulary_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_runtime_isolation_audit_source" -Needle '"source_method": "recovery.loader_runtime_isolation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_runtime_isolation_selftest_audit_source" -Needle '"source_method": "recovery.loader_runtime_isolation_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_transaction_engine_audit_source" -Needle '"source_method": "recovery.rollback_transaction_engine"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_transaction_engine_selftest_audit_source" -Needle '"source_method": "recovery.rollback_transaction_engine_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_durable_audit_rollback_persistence_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_persistence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_durable_audit_rollback_persistence_selftest_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_persistence_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_provenance_audit_source" -Needle '"source_method": "recovery.memory_provenance"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_provenance_selftest_audit_source" -Needle '"source_method": "recovery.memory_provenance_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_admission_audit_source" -Needle '"source_method": "recovery.lifeline_command_admission"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_admission_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_admission_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_envelope_audit_source" -Needle '"source_method": "recovery.lifeline_command_envelope_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_envelope_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_envelope_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_source" -Needle '"source_method": "recovery.lifeline_command_body_canonicalization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_body_canonicalization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_kind" -Needle '"kind": "recovery.lifeline_command_body_canonicalization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_body_canonicalization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_envelope_event" -Needle "`"retained_recovery_lifeline_command_envelope_event_id`": `"$recoveryLifelineCommandEnvelopeEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_hash" -Needle "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_source" -Needle '"source_method": "recovery.lifeline_command_handler_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_handler_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_kind" -Needle '"kind": "recovery.lifeline_command_handler_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_handler_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_body_event" -Needle "`"retained_recovery_lifeline_command_body_canonicalization_event_id`": `"$recoveryLifelineCommandBodyEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_hash" -Needle "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_source" -Needle '"source_method": "recovery.lifeline_status_read_handler_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_status_read_handler_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_kind" -Needle '"kind": "recovery.lifeline_status_read_handler.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_status_read_handler.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_handler_event" -Needle "`"retained_recovery_lifeline_command_handler_binding_event_id`": `"$recoveryCommandHandlerBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_hash" -Needle "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_no_status_execute" -Needle '"executes_lifeline_status": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_source" -Needle '"source_method": "recovery.rollback_preview_authorization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_selftest_audit_source" -Needle '"source_method": "recovery.rollback_preview_authorization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_kind" -Needle '"kind": "recovery.rollback_preview_authorization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_rollback_preview_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_status_event" -Needle "`"retained_recovery_lifeline_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_hash" -Needle "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_no_preview" -Needle '"executes_rollback_preview": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_source" -Needle '"source_method": "recovery.rollback_apply_authorization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_selftest_audit_source" -Needle '"source_method": "recovery.rollback_apply_authorization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_kind" -Needle '"kind": "recovery.rollback_apply_authorization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_rollback_apply_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_preview_event" -Needle "`"retained_recovery_rollback_preview_authorization_event_id`": `"$recoveryRollbackPreviewAuthorizationEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_hash" -Needle "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_no_apply" -Needle '"executes_rollback_apply": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_source" -Needle '"source_method": "recovery.disable_module_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.disable_module_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_kind" -Needle '"kind": "recovery.disable_module_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_disable_module_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_apply_event" -Needle "`"retained_recovery_rollback_apply_authorization_event_id`": `"$recoveryRollbackApplyAuthorizationEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_hash" -Needle "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_no_disable" -Needle '"disables_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_source" -Needle '"source_method": "recovery.restart_last_good_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.restart_last_good_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_kind" -Needle '"kind": "recovery.restart_last_good_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_restart_last_good_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_disable_event" -Needle "`"retained_recovery_disable_module_target_binding_event_id`": `"$recoveryDisableModuleTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_hash" -Needle "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_no_restart" -Needle '"restarts_last_good": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_source" -Needle '"source_method": "recovery.load_artifact_by_hash_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.load_artifact_by_hash_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_kind" -Needle '"kind": "recovery.load_artifact_by_hash_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_load_artifact_by_hash_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_restart_event" -Needle "`"retained_recovery_restart_last_good_target_binding_event_id`": `"$recoveryRestartLastGoodTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_hash" -Needle "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_source" -Needle '"source_method": "recovery.memory_write_authority_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_selftest_audit_source" -Needle '"source_method": "recovery.memory_write_authority_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_kind" -Needle '"kind": "recovery.memory_write_authority.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_memory_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_load_event" -Needle "`"retained_recovery_load_artifact_by_hash_target_binding_event_id`": `"$recoveryLoadArtifactByHashTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_hash" -Needle "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_no_write" -Needle '"writes_recovery_memory": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_write_authority_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_selftest_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_write_authority_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_kind" -Needle '"kind": "recovery.durable_audit_rollback_write_authority.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_binding_schema" -Needle '"bindings": {"schema": "raios.durable_audit_rollback_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_memory_event" -Needle "`"retained_recovery_memory_write_authority_event_id`": `"$recoveryMemoryWriteAuthorityEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_hash" -Needle "`"durable_audit_rollback_write_authority_hash`": `"sha256:$durableAuditRollbackWriteAuthorityHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_no_durable_write" -Needle '"writes_durable_audit_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:durable_audit_rollback_write_authority_audit_no_rollback_write" -Needle '"writes_rollback_store": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_source" -Needle '"source_method": "recovery.service_inventory_side_effect_boundary_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_selftest_audit_source" -Needle '"source_method": "recovery.service_inventory_side_effect_boundary_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_kind" -Needle '"kind": "recovery.service_inventory_side_effect_boundary.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_service_inventory_side_effect_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_durable_event" -Needle "`"retained_durable_audit_rollback_write_authority_event_id`": `"$durableAuditRollbackWriteAuthorityEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_hash" -Needle "`"service_inventory_side_effect_boundary_hash`": `"sha256:$serviceInventorySideEffectBoundaryHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_no_service_slot" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_no_service_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:service_inventory_side_effect_boundary_audit_no_service_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_behavior_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_behavior_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_kind" -Needle '"kind": "recovery.lifeline_command_dispatch_behavior.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_dispatch_behavior.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_service_event" -Needle "`"retained_service_inventory_side_effect_boundary_event_id`": `"$serviceInventorySideEffectBoundaryEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_hash" -Needle "`"command_dispatch_behavior_hash`": `"sha256:$recoveryCommandDispatchBehaviorHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_service_hash" -Needle "`"service_inventory_side_effect_boundary_hash`": `"sha256:$serviceInventorySideEffectBoundaryHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_behavior_audit_no_service_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_source" -Needle '"source_method": "recovery.lifeline_command_executor_capability_table_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_executor_capability_table_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_kind" -Needle '"kind": "recovery.lifeline_command_executor_capability_table.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_executor_capability_table.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_behavior_event" -Needle "`"retained_command_dispatch_behavior_event_id`": `"$recoveryCommandDispatchBehaviorEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_hash" -Needle "`"executor_capability_table_hash`": `"sha256:$recoveryExecutorCapabilityTableHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_behavior_hash" -Needle "`"command_dispatch_behavior_hash`": `"sha256:$recoveryCommandDispatchBehaviorHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_projection_hash" -Needle "`"executor_capability_projection_hash`": `"sha256:$recoveryExecutorCapabilityProjectionHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_executor_capability_table_audit_no_service_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_source" -Needle '"source_method": "recovery.lifeline_command_side_effect_gate_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_side_effect_gate_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_kind" -Needle '"kind": "recovery.lifeline_command_side_effect_gate.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_side_effect_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_executor_event" -Needle "`"retained_executor_capability_table_event_id`": `"$recoveryExecutorCapabilityTableEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_hash" -Needle "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_executor_hash" -Needle "`"executor_capability_table_hash`": `"sha256:$recoveryExecutorCapabilityTableHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_projection_hash" -Needle "`"side_effect_projection_hash`": `"sha256:$recoverySideEffectProjectionHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_side_effect_gate_audit_no_service_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_enablement_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_enablement_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_enablement.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_enablement.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionEnablementHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_previous_event" -Needle "`"retained_previous_stage_event_id`": `"$recoverySideEffectGateEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_enablement_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_preflight_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_preflight_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_preflight.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_preflight.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionPreflightHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_preflight_audit_enablement_hash" -Needle "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_intent_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_intent_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_intent.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_intent.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionIntentHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_intent_audit_preflight_hash" -Needle "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_commit_gate_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_commit_gate_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_commit_gate.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_commit_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_intent_hash" -Needle "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_commit_gate_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_result_denial_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_result_denial_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_result_denial.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_result_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_commit_hash" -Needle "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_result_denial_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_audit_denial_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_audit_denial_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_audit_denial.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_audit_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_result_hash" -Needle "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_audit_denial_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_observation_denial_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_observation_denial_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_observation_denial.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_observation_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_hash" -Needle "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_observation_denial_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_completion_denial_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_execution_completion_denial_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_kind" -Needle '"kind": "recovery.lifeline_command_execution_completion_denial.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_execution_completion_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_stage_hash" -Needle "`"execution_stage_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_hash" -Needle "`"execution_observation_denial_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_execution_completion_denial_audit_execution_false" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_risk" -Needle '"risk": "recovery_modify_ram"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_resource" -Needle '"resource": "recovery_lifeline"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_reason" -Needle '"reason": "missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_boundary" -Needle '"recovery_artifact_load_boundary_evaluated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_identity" -Needle '"recovery_artifact_identity_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_normal_path" -Needle '"normal_module_load_path_not_used"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_load_denial_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_status" -Needle '"status": "denied_missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_normal_path" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_missing_identity" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_missing_rollback" -Needle '"recovery_rollback_evidence": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_audit_source" -Needle '"source_method": "recovery.load_binding"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_audit_source" -Needle '"source_method": "recovery.load_binding_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
