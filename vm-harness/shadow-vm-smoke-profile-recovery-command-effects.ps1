    Send-AgentCommand -Command "agent recovery.service_inventory_side_effect_boundary_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.service_inventory_side_effect_boundary_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:service_inventory_side_effect_boundary_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_service_inventory_side_effect_boundary_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_service_inventory_side_effect_boundary_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_service_inventory_side_effect_boundary_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_service_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.service_inventory_side_effect_boundary_diagnostic' },
        @{ Suffix = "service_schema"; Needle = '"service_inventory_side_effect_boundary_schema": "raios.recovery_service_inventory_side_effect_boundary.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"service_inventory_side_effect_boundary_canonicalization": "raios.recovery_service_inventory_side_effect_boundary.canonical.v0"' },
        @{ Suffix = "service_boundary"; Needle = '"service_inventory_side_effect_boundary_id": "boundary.recovery_service_inventory_side_effect_boundary.current_boot"' },
        @{ Suffix = "behavior_fact"; Needle = '"fact": "recovery_lifeline_command_dispatch_behavior"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.service_inventory_side_effect_boundary_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.service_inventory_side_effect_boundary_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:service_inventory_side_effect_boundary_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_service_inventory_side_effect_boundary_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_service_inventory_side_effect_boundary_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_service_inventory_side_effect_boundary_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_service_inventory_side_effect_boundary_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_service_inventory_side_effect_boundary"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "side_effect_case"; Needle = '"case": "service_inventory_side_effect_boundary_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "service_inventory_side_effect_boundary_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_durable_audit_rollback_write_authority_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_service_inventory_side_effect_boundary_still_non_executable"' },
        @{ Suffix = "no_service_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $serviceInventorySideEffectBoundaryId = "boundary.recovery_service_inventory_side_effect_boundary.current_boot"
    $serviceInventoryProjectionCanonical = @(
        "schema=raios.recovery_service_inventory_side_effect_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "durable_audit_rollback_write_authority_hash=$durableAuditRollbackWriteAuthorityHash",
        "recovery_memory_write_authority_hash=$recoveryMemoryWriteAuthorityHash",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash",
        "service_inventory_change=none"
    ) -join "`n"
    $serviceInventoryProjectionHash = Get-TextSha256 -Text $serviceInventoryProjectionCanonical
    $serviceInventorySideEffectBoundaryCanonical = @(
        "canonicalization=raios.recovery_service_inventory_side_effect_boundary.canonical.v0",
        "schema=raios.recovery_service_inventory_side_effect_boundary.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=service_inventory_side_effect_boundary",
        "scope=current_boot",
        "retained_durable_audit_rollback_write_authority_event_id=$durableAuditRollbackWriteAuthorityEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "load_artifact_by_hash_target_binding_sha256=$recoveryLoadArtifactByHashTargetBindingHash",
        "recovery_memory_write_authority_sha256=$recoveryMemoryWriteAuthorityHash",
        "durable_audit_rollback_write_authority_sha256=$durableAuditRollbackWriteAuthorityHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "service_inventory_side_effect_boundary_id=$serviceInventorySideEffectBoundaryId",
        "service_inventory_projection_sha256=$serviceInventoryProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "allocates_service_slot=false",
        "creates_service_inventory_records=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $serviceInventorySideEffectBoundaryHash = Get-TextSha256 -Text $serviceInventorySideEffectBoundaryCanonical
    $serviceInventorySideEffectBoundaryCommand = "agent recovery.service_inventory_side_effect_boundary_diagnostic $serviceInventorySideEffectBoundaryHash $durableAuditRollbackWriteAuthorityEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryMemoryWriteAuthorityHash $durableAuditRollbackWriteAuthorityHash $recoveryCommandDispatchBoundaryId $serviceInventorySideEffectBoundaryId $serviceInventoryProjectionHash"

    Send-AgentCommand -Command $serviceInventorySideEffectBoundaryCommand -ExpectedMarker "RAIOS_AGENT_END recovery.service_inventory_side_effect_boundary_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:service_inventory_side_effect_boundary_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_service_inventory_side_effect_boundary_valid_but_service_inventory_unchanged"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_service_inventory_side_effect_boundary_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "service_boundary_id"; Needle = "`"service_inventory_side_effect_boundary_id`": `"$serviceInventorySideEffectBoundaryId`"" },
        @{ Suffix = "durable_event_id"; Needle = "`"retained_durable_audit_rollback_write_authority_event_id`": `"$durableAuditRollbackWriteAuthorityEventId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "durable_hash"; Needle = "`"durable_audit_rollback_write_authority_hash`": `"sha256:$durableAuditRollbackWriteAuthorityHash`"" },
        @{ Suffix = "memory_hash"; Needle = "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"service_inventory_projection_hash`": `"sha256:$serviceInventoryProjectionHash`"" },
        @{ Suffix = "boundary_hash"; Needle = "`"service_inventory_side_effect_boundary_hash`": `"sha256:$serviceInventorySideEffectBoundaryHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_service_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $serviceInventorySideEffectBoundaryResponse = Get-LastAgentResponseJson -Method "recovery.service_inventory_side_effect_boundary_diagnostic"
    $serviceInventorySideEffectBoundaryEventId = [string]$serviceInventorySideEffectBoundaryResponse.body.result.retained_service_inventory_side_effect_boundary_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:service_inventory_side_effect_boundary_retained_reference_event_id_captured" -Value $serviceInventorySideEffectBoundaryEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_service_inventory_boundary_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "defined_non_executable"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_dispatch_behavior_not_implemented"' },
        @{ Suffix = "memory_authority_present"; Needle = '"recovery_memory_write_authority_present": true' },
        @{ Suffix = "durable_authority_present"; Needle = '"durable_audit_rollback_write_authority_present": true' },
        @{ Suffix = "service_boundary_present"; Needle = '"service_inventory_side_effect_boundary_present": true' },
        @{ Suffix = "behavior_missing"; Needle = '"command_dispatch_behavior_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_behavior_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_behavior_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_behavior_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_behavior_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_dispatch_behavior_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_behavior_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_dispatch_behavior_diagnostic' },
        @{ Suffix = "behavior_schema"; Needle = '"command_dispatch_behavior_schema": "raios.recovery_lifeline_command_dispatch_behavior.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"command_dispatch_behavior_canonicalization": "raios.recovery_lifeline_command_dispatch_behavior.canonical.v0"' },
        @{ Suffix = "behavior_boundary"; Needle = '"command_dispatch_behavior_id": "boundary.recovery_lifeline_command_dispatch_behavior.current_boot"' },
        @{ Suffix = "executor_fact"; Needle = '"fact": "executor_capability_table"' },
        @{ Suffix = "side_effect_fact"; Needle = '"fact": "side_effect_gate"' },
        @{ Suffix = "execution_fact"; Needle = '"fact": "command_execution_enablement"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_behavior_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_behavior_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_behavior_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_behavior_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_behavior_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_lifeline_command_dispatch_behavior_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_lifeline_command_dispatch_behavior_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_lifeline_command_dispatch_behavior"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "behavior_id_case"; Needle = '"case": "command_dispatch_behavior_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "command_dispatch_behavior_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_recovery_service_inventory_side_effect_boundary_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_recovery_lifeline_command_dispatch_behavior_still_non_executable"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandDispatchBehaviorId = "boundary.recovery_lifeline_command_dispatch_behavior.current_boot"
    $recoveryCommandDispatchBehaviorProjectionCanonical = @(
        "schema=raios.recovery_lifeline_command_dispatch_behavior_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "service_inventory_side_effect_boundary_hash=$serviceInventorySideEffectBoundaryHash",
        "durable_audit_rollback_write_authority_hash=$durableAuditRollbackWriteAuthorityHash",
        "recovery_memory_write_authority_hash=$recoveryMemoryWriteAuthorityHash",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash",
        "command_execution_enabled=false"
    ) -join "`n"
    $recoveryCommandDispatchBehaviorProjectionHash = Get-TextSha256 -Text $recoveryCommandDispatchBehaviorProjectionCanonical
    $recoveryCommandDispatchBehaviorCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_dispatch_behavior.canonical.v0",
        "schema=raios.recovery_lifeline_command_dispatch_behavior.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_dispatch_behavior",
        "scope=current_boot",
        "retained_service_inventory_side_effect_boundary_event_id=$serviceInventorySideEffectBoundaryEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "load_artifact_by_hash_target_binding_sha256=$recoveryLoadArtifactByHashTargetBindingHash",
        "recovery_memory_write_authority_sha256=$recoveryMemoryWriteAuthorityHash",
        "durable_audit_rollback_write_authority_sha256=$durableAuditRollbackWriteAuthorityHash",
        "service_inventory_side_effect_boundary_sha256=$serviceInventorySideEffectBoundaryHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "command_dispatch_behavior_id=$recoveryCommandDispatchBehaviorId",
        "command_dispatch_behavior_projection_sha256=$recoveryCommandDispatchBehaviorProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "command_execution_enabled=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "allocates_service_slot=false",
        "creates_service_inventory_records=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryCommandDispatchBehaviorHash = Get-TextSha256 -Text $recoveryCommandDispatchBehaviorCanonical
    $recoveryCommandDispatchBehaviorCommand = "agent recovery.lifeline_command_dispatch_behavior_diagnostic $recoveryCommandDispatchBehaviorHash $serviceInventorySideEffectBoundaryEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryMemoryWriteAuthorityHash $durableAuditRollbackWriteAuthorityHash $serviceInventorySideEffectBoundaryHash $recoveryCommandDispatchBoundaryId $recoveryCommandDispatchBehaviorId $recoveryCommandDispatchBehaviorProjectionHash"

    Send-AgentCommand -Command $recoveryCommandDispatchBehaviorCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_behavior_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_behavior_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_dispatch_behavior_valid_but_execution_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_behavior_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "behavior_id"; Needle = "`"command_dispatch_behavior_id`": `"$recoveryCommandDispatchBehaviorId`"" },
        @{ Suffix = "service_event_id"; Needle = "`"retained_service_inventory_side_effect_boundary_event_id`": `"$serviceInventorySideEffectBoundaryEventId`"" },
        @{ Suffix = "service_hash"; Needle = "`"service_inventory_side_effect_boundary_hash`": `"sha256:$serviceInventorySideEffectBoundaryHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"command_dispatch_behavior_projection_hash`": `"sha256:$recoveryCommandDispatchBehaviorProjectionHash`"" },
        @{ Suffix = "behavior_hash"; Needle = "`"command_dispatch_behavior_hash`": `"sha256:$recoveryCommandDispatchBehaviorHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandDispatchBehaviorResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_dispatch_behavior_diagnostic"
    $recoveryCommandDispatchBehaviorEventId = [string]$recoveryCommandDispatchBehaviorResponse.body.result.retained_recovery_lifeline_command_dispatch_behavior_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_dispatch_behavior_retained_reference_event_id_captured" -Value $recoveryCommandDispatchBehaviorEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_behavior_boundary_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "defined_non_executable"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_executor_capability_table_not_implemented"' },
        @{ Suffix = "service_boundary_present"; Needle = '"service_inventory_side_effect_boundary_present": true' },
        @{ Suffix = "behavior_present"; Needle = '"command_dispatch_behavior_present": true' },
        @{ Suffix = "executor_missing"; Needle = '"executor_capability_table_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_executor_capability_table_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_executor_capability_table_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_executor_capability_table_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_executor_capability_table_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_executor_capability_table_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_executor_capability_table_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_executor_capability_table_diagnostic' },
        @{ Suffix = "executor_schema"; Needle = '"executor_capability_table_schema": "raios.recovery_lifeline_command_executor_capability_table.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"executor_capability_table_canonicalization": "raios.recovery_lifeline_command_executor_capability_table.canonical.v0"' },
        @{ Suffix = "executor_boundary"; Needle = '"executor_capability_table_id": "boundary.recovery_lifeline_command_executor_capability_table.current_boot"' },
        @{ Suffix = "side_effect_fact"; Needle = '"fact": "side_effect_gate"' },
        @{ Suffix = "execution_fact"; Needle = '"fact": "command_execution_enablement"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_executor_capability_table_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_executor_capability_table_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_executor_capability_table_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_executor_capability_table_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_executor_capability_table_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_lifeline_command_executor_capability_table_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_lifeline_command_executor_capability_table_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_lifeline_command_executor_capability_table"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "executor_id_case"; Needle = '"case": "executor_capability_table_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "executor_capability_table_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_recovery_lifeline_command_dispatch_behavior_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_recovery_lifeline_command_executor_capability_table_still_non_executable"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryExecutorCapabilityTableId = "boundary.recovery_lifeline_command_executor_capability_table.current_boot"
    $recoveryExecutorCapabilityProjectionCanonical = @(
        "schema=raios.recovery_lifeline_command_executor_capability_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "command_dispatch_behavior_hash=$recoveryCommandDispatchBehaviorHash",
        "service_inventory_side_effect_boundary_hash=$serviceInventorySideEffectBoundaryHash",
        "durable_audit_rollback_write_authority_hash=$durableAuditRollbackWriteAuthorityHash",
        "recovery_memory_write_authority_hash=$recoveryMemoryWriteAuthorityHash",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash",
        "command_execution_enabled=false"
    ) -join "`n"
    $recoveryExecutorCapabilityProjectionHash = Get-TextSha256 -Text $recoveryExecutorCapabilityProjectionCanonical
    $recoveryExecutorCapabilityTableCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_executor_capability_table.canonical.v0",
        "schema=raios.recovery_lifeline_command_executor_capability_table.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_executor_capability_table",
        "scope=current_boot",
        "retained_command_dispatch_behavior_event_id=$recoveryCommandDispatchBehaviorEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "load_artifact_by_hash_target_binding_sha256=$recoveryLoadArtifactByHashTargetBindingHash",
        "recovery_memory_write_authority_sha256=$recoveryMemoryWriteAuthorityHash",
        "durable_audit_rollback_write_authority_sha256=$durableAuditRollbackWriteAuthorityHash",
        "service_inventory_side_effect_boundary_sha256=$serviceInventorySideEffectBoundaryHash",
        "command_dispatch_behavior_sha256=$recoveryCommandDispatchBehaviorHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "executor_capability_table_id=$recoveryExecutorCapabilityTableId",
        "executor_capability_projection_sha256=$recoveryExecutorCapabilityProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "command_execution_enabled=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "allocates_service_slot=false",
        "creates_service_inventory_records=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryExecutorCapabilityTableHash = Get-TextSha256 -Text $recoveryExecutorCapabilityTableCanonical
    $recoveryExecutorCapabilityTableCommand = "agent recovery.lifeline_command_executor_capability_table_diagnostic $recoveryExecutorCapabilityTableHash $recoveryCommandDispatchBehaviorEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryMemoryWriteAuthorityHash $durableAuditRollbackWriteAuthorityHash $serviceInventorySideEffectBoundaryHash $recoveryCommandDispatchBehaviorHash $recoveryCommandDispatchBoundaryId $recoveryExecutorCapabilityTableId $recoveryExecutorCapabilityProjectionHash"

    Send-AgentCommand -Command $recoveryExecutorCapabilityTableCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_executor_capability_table_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_executor_capability_table_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_executor_capability_table_valid_but_execution_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_executor_capability_table_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "executor_id"; Needle = "`"executor_capability_table_id`": `"$recoveryExecutorCapabilityTableId`"" },
        @{ Suffix = "behavior_event_id"; Needle = "`"retained_command_dispatch_behavior_event_id`": `"$recoveryCommandDispatchBehaviorEventId`"" },
        @{ Suffix = "behavior_hash"; Needle = "`"command_dispatch_behavior_hash`": `"sha256:$recoveryCommandDispatchBehaviorHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"executor_capability_projection_hash`": `"sha256:$recoveryExecutorCapabilityProjectionHash`"" },
        @{ Suffix = "executor_hash"; Needle = "`"executor_capability_table_hash`": `"sha256:$recoveryExecutorCapabilityTableHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryExecutorCapabilityTableResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_executor_capability_table_diagnostic"
    $recoveryExecutorCapabilityTableEventId = [string]$recoveryExecutorCapabilityTableResponse.body.result.retained_recovery_lifeline_command_executor_capability_table_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_executor_capability_table_retained_reference_event_id_captured" -Value $recoveryExecutorCapabilityTableEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_executor_capability_table_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "defined_non_executable"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_side_effect_gate_not_implemented"' },
        @{ Suffix = "service_boundary_present"; Needle = '"service_inventory_side_effect_boundary_present": true' },
        @{ Suffix = "behavior_present"; Needle = '"command_dispatch_behavior_present": true' },
        @{ Suffix = "executor_present"; Needle = '"executor_capability_table_present": true' },
        @{ Suffix = "side_effect_missing"; Needle = '"side_effect_gate_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_side_effect_gate_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_side_effect_gate_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_side_effect_gate_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_side_effect_gate_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_side_effect_gate_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"global_event_log_mutation": "none"' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_side_effect_gate_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_side_effect_gate_diagnostic' },
        @{ Suffix = "side_effect_schema"; Needle = '"side_effect_gate_schema": "raios.recovery_lifeline_command_side_effect_gate.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"side_effect_gate_canonicalization": "raios.recovery_lifeline_command_side_effect_gate.canonical.v0"' },
        @{ Suffix = "side_effect_gate"; Needle = '"side_effect_gate_id": "boundary.recovery_lifeline_command_side_effect_gate.current_boot"' },
        @{ Suffix = "execution_enablement_fact"; Needle = '"fact": "command_execution_enablement"' },
        @{ Suffix = "side_effect_present_false"; Needle = '"side_effect_gate_reference_present": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_side_effect_gate_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_side_effect_gate_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_side_effect_gate_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_side_effect_gate_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_side_effect_gate_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_lifeline_command_side_effect_gate_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_lifeline_command_side_effect_gate_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_lifeline_command_side_effect_gate"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "side_effect_id_case"; Needle = '"case": "side_effect_gate_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "side_effect_gate_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_recovery_lifeline_command_executor_capability_table_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_recovery_lifeline_command_side_effect_gate_still_non_executable"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoverySideEffectGateId = "boundary.recovery_lifeline_command_side_effect_gate.current_boot"
    $recoverySideEffectProjectionCanonical = @(
        "schema=raios.recovery_lifeline_command_side_effect_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "executor_capability_table_hash=$recoveryExecutorCapabilityTableHash",
        "command_dispatch_behavior_hash=$recoveryCommandDispatchBehaviorHash",
        "service_inventory_side_effect_boundary_hash=$serviceInventorySideEffectBoundaryHash",
        "durable_audit_rollback_write_authority_hash=$durableAuditRollbackWriteAuthorityHash",
        "recovery_memory_write_authority_hash=$recoveryMemoryWriteAuthorityHash",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash",
        "command_execution_enabled=false",
        "service_inventory_change=none"
    ) -join "`n"
    $recoverySideEffectProjectionHash = Get-TextSha256 -Text $recoverySideEffectProjectionCanonical
    $recoverySideEffectGateCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_side_effect_gate.canonical.v0",
        "schema=raios.recovery_lifeline_command_side_effect_gate.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_side_effect_gate",
        "scope=current_boot",
        "retained_executor_capability_table_event_id=$recoveryExecutorCapabilityTableEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "load_artifact_by_hash_target_binding_sha256=$recoveryLoadArtifactByHashTargetBindingHash",
        "recovery_memory_write_authority_sha256=$recoveryMemoryWriteAuthorityHash",
        "durable_audit_rollback_write_authority_sha256=$durableAuditRollbackWriteAuthorityHash",
        "service_inventory_side_effect_boundary_sha256=$serviceInventorySideEffectBoundaryHash",
        "command_dispatch_behavior_sha256=$recoveryCommandDispatchBehaviorHash",
        "executor_capability_table_sha256=$recoveryExecutorCapabilityTableHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "side_effect_gate_id=$recoverySideEffectGateId",
        "side_effect_projection_sha256=$recoverySideEffectProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "command_execution_enabled=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "allocates_service_slot=false",
        "creates_service_inventory_records=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoverySideEffectGateHash = Get-TextSha256 -Text $recoverySideEffectGateCanonical
    $recoverySideEffectGateCommand = "agent recovery.lifeline_command_side_effect_gate_diagnostic $recoverySideEffectGateHash $recoveryExecutorCapabilityTableEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryMemoryWriteAuthorityHash $durableAuditRollbackWriteAuthorityHash $serviceInventorySideEffectBoundaryHash $recoveryCommandDispatchBehaviorHash $recoveryExecutorCapabilityTableHash $recoveryCommandDispatchBoundaryId $recoverySideEffectGateId $recoverySideEffectProjectionHash"

    Send-AgentCommand -Command $recoverySideEffectGateCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_side_effect_gate_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_side_effect_gate_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_side_effect_gate_valid_but_execution_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_side_effect_gate_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "side_effect_id"; Needle = "`"side_effect_gate_id`": `"$recoverySideEffectGateId`"" },
        @{ Suffix = "executor_event_id"; Needle = "`"retained_executor_capability_table_event_id`": `"$recoveryExecutorCapabilityTableEventId`"" },
        @{ Suffix = "executor_hash"; Needle = "`"executor_capability_table_hash`": `"sha256:$recoveryExecutorCapabilityTableHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"side_effect_projection_hash`": `"sha256:$recoverySideEffectProjectionHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoverySideEffectGateResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_side_effect_gate_diagnostic"
    $recoverySideEffectGateEventId = [string]$recoverySideEffectGateResponse.body.result.retained_recovery_lifeline_command_side_effect_gate_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_side_effect_gate_retained_reference_event_id_captured" -Value $recoverySideEffectGateEventId
