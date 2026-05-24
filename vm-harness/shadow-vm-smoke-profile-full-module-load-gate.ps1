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
        @{ Suffix = "write_boundary"; Method = "module.loader_audit_rollback_write_boundary_binding"; Locator = "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding" }
    )

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
    $moduleFinalLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_gate_schema" -Needle '"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:mutating_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_mode_ram_only" -Needle '"load_mode": "ram_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_target" -Needle '"target": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_retained" -Needle '"module_manifest": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_reference" -Needle '"retained_module_manifest_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_present" -Needle '"schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_event_id" -Needle '"retained_manifest_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_retained" -Needle '"candidate_artifact": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_reference" -Needle '"retained_candidate_artifact_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_present" -Needle '"schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadArtifactEventId = [string]$moduleFinalLoadResponse.body.retained_candidate_artifact_reference.event_id
    $moduleLoadArtifactEventIdMatches = $moduleLoadArtifactEventId -eq $moduleArtifactRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_artifact_event_id" -Expected $moduleArtifactRetainedReferenceEventId -Passed $moduleLoadArtifactEventIdMatches -Actual $moduleLoadArtifactEventId
    if (-not $moduleLoadArtifactEventIdMatches) {
        throw "Expected retained artifact event id $moduleArtifactRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadArtifactEventId"
    }
    Assert-LogContains -Name "policy:module_retained_artifact_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_retained" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_reference" -Needle '"retained_vm_test_report_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_present" -Needle '"schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadVmReportEventId = [string]$moduleFinalLoadResponse.body.retained_vm_test_report_reference.event_id
    $moduleLoadVmReportEventIdMatches = $moduleLoadVmReportEventId -eq $moduleVmReportRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_vm_report_event_id" -Expected $moduleVmReportRetainedReferenceEventId -Passed $moduleLoadVmReportEventIdMatches -Actual $moduleLoadVmReportEventId
    if (-not $moduleLoadVmReportEventIdMatches) {
        throw "Expected retained VM report event id $moduleVmReportRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadVmReportEventId"
    }
    Assert-LogContains -Name "policy:module_retained_vm_report_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_retained" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_reference" -Needle '"retained_local_attestation_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_present" -Needle '"schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_event_id" -Needle '"retained_local_attestation_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_reference" -Needle '"retained_computed_grant_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_present" -Needle '"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_retained" -Needle '"local_approval": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_reference" -Needle '"retained_local_approval_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_present" -Needle '"schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadApprovalEventId = [string]$moduleFinalLoadResponse.body.retained_local_approval_reference.event_id
    $moduleLoadApprovalEventIdMatches = $moduleLoadApprovalEventId -eq $moduleApprovalRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_approval_event_id" -Expected $moduleApprovalRetainedReferenceEventId -Passed $moduleLoadApprovalEventIdMatches -Actual $moduleLoadApprovalEventId
    if (-not $moduleLoadApprovalEventIdMatches) {
        throw "Expected retained approval event id $moduleApprovalRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadApprovalEventId"
    }
    Assert-LogContains -Name "policy:module_retained_approval_reason" -Needle '"reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_retained" -Needle '"rollback_plan": "retained_hash_reference_only_not_installed"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_retained" -Needle '"durable_audit_record": "retained_hash_reference_only_not_durable"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_reference" -Needle '"retained_audit_rollback_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_present" -Needle '"schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_event_id" -Needle '"retained_audit_rollback_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_rollback_hash" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_unavailable" -Needle '"loader": "unavailable"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_retained" -Needle '"service_slot": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_reference" -Needle '"retained_service_slot_reservation": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_present" -Needle '"schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_status" -Needle '"status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_event_id" -Needle '"retained_service_slot_reservation_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_reason" -Needle '"reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_state" -Needle '"service_slot_allocator": "missing_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_schema" -Needle '"schema": "raios.module_service_slot_allocator_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_source" -Needle '"source_method": "module.service_slot_allocator"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_status" -Needle '"readiness_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_readiness_reason" -Needle '"readiness_reason": "service_slot_allocator_runtime_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_ready_false" -Needle '"service_slot_allocator_ready": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_state" -Needle '"loader_runtime": "blocked_by_service_slot_allocator_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_schema" -Needle '"schema": "raios.module_loader_runtime_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_status" -Needle '"readiness_status": "denied_missing_service_slot_allocator_runtime"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_reason" -Needle '"readiness_reason": "service_slot_allocator_runtime_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_retained_evidence" -Needle '"retained_module_evidence_state": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_no_descriptor" -Needle '"accepts_loader_descriptor": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_fact_schema" -Needle '"schema": "raios.module_loader_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_fact_missing" -Needle '"reason": "module_loader_identity_missing"' -TimeoutSeconds 1
    $moduleLoadGateSourceCount = [int]$moduleFinalLoadResponse.body.loader_runtime_readiness.source_fact_count
    $moduleLoadGateSourceCountMatches = $moduleLoadGateSourceCount -eq 10
    Add-Predicate -Name "policy:module_loader_runtime_source_count" -Expected 10 -Passed $moduleLoadGateSourceCountMatches -Actual $moduleLoadGateSourceCount
    if (-not $moduleLoadGateSourceCountMatches) {
        throw "Expected 10 module.load_ephemeral loader-runtime source facts, got $moduleLoadGateSourceCount"
    }
    $moduleLoadGateSourceMapComplete = [bool]$moduleFinalLoadResponse.body.loader_runtime_readiness.source_fact_map_complete
    Add-Predicate -Name "policy:module_loader_runtime_source_map_complete" -Expected $true -Passed $moduleLoadGateSourceMapComplete -Actual $moduleLoadGateSourceMapComplete
    if (-not $moduleLoadGateSourceMapComplete) {
        throw "Expected module.load_ephemeral loader-runtime source fact map to be complete"
    }
    foreach ($source in $moduleLoaderRuntimeGateSources) {
        $matchingSource = @($moduleFinalLoadResponse.body.loader_runtime_readiness.source_fact_map | Where-Object { $_.source_method -eq $source.Method -and $_.source_fact_locator -eq $source.Locator })
        $sourcePresent = $matchingSource.Count -eq 1
        Add-Predicate -Name ("policy:module_loader_runtime_" + $source.Suffix + "_source_binding") -Expected ($source.Method + " -> " + $source.Locator) -Passed $sourcePresent -Actual ($matchingSource | ConvertTo-Json -Compress)
        if (-not $sourcePresent) {
            throw ("Expected module.load_ephemeral loader-runtime source binding " + $source.Method + " -> " + $source.Locator)
        }
    }
    Assert-LogContains -Name "policy:module_artifact_not_loaded" -Needle '"artifact_loaded": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_not_started" -Needle '"service_started": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_retained_reason" -Needle '"reason": "retained_module_manifest_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_retained_reason" -Needle '"reason": "retained_candidate_artifact_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_retained_reason" -Needle '"reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_retained_reason" -Needle '"reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_retained_reason" -Needle '"reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained_not_authorizing" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_reason" -Needle '"reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_reason" -Needle '"reason": "rollback_install_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_unimplemented_reason" -Needle '"reason": "module_loader_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_required" -Needle "raios.module_manifest.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_sha256_required" -Needle "candidate_artifact_sha256" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:vm_report_required" -Needle "raios.vm_test_report.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:local_attestation_required" -Needle "raios.local_attestation.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_required" -Needle "computed_capability_grant" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_required" -Needle "local_approval" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_required" -Needle "raios.audit_record.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_required" -Needle "rollback_plan" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_required" -Needle "ram_only_service_slot" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_allocator_required" -Needle "raios.module_service_slot_allocator_readiness.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_runtime_required" -Needle "raios.module_loader_runtime_readiness.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_schema" -Needle '"schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_status" -Needle '"status": "required_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_writes" -Needle '"writes_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_schema" -Needle '"schema": "raios.audit_record.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_plan_schema" -Needle '"schema": "raios.rollback_plan.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_retained_reference_required" -Needle '"retained_computed_grant_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_retained_approval_reference_required" -Needle '"retained_local_approval_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_hash_retained" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_slot_id_retained" -Needle "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_hash_retained" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_requirement_retained" -Needle '"ram_only_service_slot": {"state": "retained_hash_reference_only_not_allocated", "reason": "retained_service_slot_reservation_not_allocated", "required": true, "allocates_service_slot": false}' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_hash_retained" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_artifact_hash_retained" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_hash_retained" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_hash_retained" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_reference_hash_retained" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_hash_retained" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_inventory_unchanged" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
