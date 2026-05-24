    Send-AgentCommand -Command "agent recovery.lifeline_command_handler_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_handler_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_handler_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_handler_binding_diagnostic' },
        @{ Suffix = "handler_schema"; Needle = '"handler_binding_schema": "raios.recovery_lifeline_command_handler_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"handler_binding_canonicalization": "raios.recovery_lifeline_command_handler_binding.canonical.v0"' },
        @{ Suffix = "handler_boundary"; Needle = '"handler_binding_boundary_id": "boundary.recovery_lifeline_command_handler_binding.current_boot"' },
        @{ Suffix = "status_handler_fact"; Needle = '"fact": "status_read_handler"' },
        @{ Suffix = "preview_auth_fact"; Needle = '"fact": "rollback_preview_authorization"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_handler_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_handler_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "handler_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "handler_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_handler_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "handler_case"; Needle = '"case": "handler_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "handler_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_body_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_handler_binding_still_non_executable"' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandHandlerBindingBoundaryId = "boundary.recovery_lifeline_command_handler_binding.current_boot"
    $recoveryCommandHandlerInputCanonical = @(
        "schema=raios.recovery_lifeline_command_handler_input_binding.v0",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "handler_id=$recoveryCommandHandlerBindingBoundaryId",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryCommandHandlerInputBindingHash = Get-TextSha256 -Text $recoveryCommandHandlerInputCanonical
    $recoveryCommandHandlerBindingCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_handler_binding.canonical.v0",
        "schema=raios.recovery_lifeline_command_handler_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_handler",
        "scope=current_boot",
        "retained_recovery_lifeline_command_body_canonicalization_event_id=$recoveryLifelineCommandBodyEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "handler_id=$recoveryCommandHandlerBindingBoundaryId",
        "handler_input_binding_sha256=$recoveryCommandHandlerInputBindingHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryCommandHandlerBindingHash = Get-TextSha256 -Text $recoveryCommandHandlerBindingCanonical
    $recoveryCommandHandlerBindingCommand = "agent recovery.lifeline_command_handler_binding_diagnostic $recoveryCommandHandlerBindingHash $recoveryLifelineCommandBodyEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandDispatchBoundaryId $recoveryCommandHandlerBindingBoundaryId $recoveryCommandHandlerInputBindingHash"

    Send-AgentCommand -Command $recoveryCommandHandlerBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_handler_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "handler_id"; Needle = "`"handler_id`": `"$recoveryCommandHandlerBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "input_hash"; Needle = "`"handler_input_binding_hash`": `"sha256:$recoveryCommandHandlerInputBindingHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandHandlerBindingResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_handler_binding_diagnostic"
    $recoveryCommandHandlerBindingEventId = [string]$recoveryCommandHandlerBindingResponse.body.result.retained_command_handler_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_handler_retained_reference_event_id_captured" -Value $recoveryCommandHandlerBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_handler_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_missing"; Needle = '"status_read_handler_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_status_read_handler_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_read_handler_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_status_execute"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_status_read_handler_diagnostic' },
        @{ Suffix = "status_handler_schema"; Needle = '"status_read_handler_schema": "raios.recovery_lifeline_status_read_handler.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"status_read_handler_canonicalization": "raios.recovery_lifeline_status_read_handler.canonical.v0"' },
        @{ Suffix = "status_handler_boundary"; Needle = '"status_read_handler_boundary_id": "boundary.recovery_lifeline_status_read_handler.current_boot"' },
        @{ Suffix = "preview_auth_fact"; Needle = '"fact": "rollback_preview_authorization"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_status_read_handler_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_read_handler_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "status_read_handler_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "status_read_handler_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_status_read_handler"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "handler_case"; Needle = '"case": "status_handler_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "status_read_handler_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_handler_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_status_read_handler_still_non_executable"' },
        @{ Suffix = "status_execute_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryStatusReadHandlerBoundaryId = "boundary.recovery_lifeline_status_read_handler.current_boot"
    $recoveryStatusReadProjectionCanonical = @(
        "schema=raios.recovery_lifeline_status_read_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryStatusReadProjectionHash = Get-TextSha256 -Text $recoveryStatusReadProjectionCanonical
    $recoveryStatusReadHandlerCanonical = @(
        "canonicalization=raios.recovery_lifeline_status_read_handler.canonical.v0",
        "schema=raios.recovery_lifeline_status_read_handler.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_status_read_handler",
        "scope=current_boot",
        "retained_recovery_lifeline_command_handler_binding_event_id=$recoveryCommandHandlerBindingEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "status_handler_id=$recoveryStatusReadHandlerBoundaryId",
        "status_read_projection_sha256=$recoveryStatusReadProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryStatusReadHandlerHash = Get-TextSha256 -Text $recoveryStatusReadHandlerCanonical
    $recoveryStatusReadHandlerCommand = "agent recovery.lifeline_status_read_handler_diagnostic $recoveryStatusReadHandlerHash $recoveryCommandHandlerBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryCommandDispatchBoundaryId $recoveryStatusReadHandlerBoundaryId $recoveryStatusReadProjectionHash"

    Send-AgentCommand -Command $recoveryStatusReadHandlerCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "status_handler_id"; Needle = "`"status_handler_id`": `"$recoveryStatusReadHandlerBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"status_read_projection_hash`": `"sha256:$recoveryStatusReadProjectionHash`"" },
        @{ Suffix = "status_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "status_execute_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryStatusReadHandlerResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_status_read_handler_diagnostic"
    $recoveryStatusReadHandlerEventId = [string]$recoveryStatusReadHandlerResponse.body.result.retained_status_read_handler_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_status_read_handler_retained_reference_event_id_captured" -Value $recoveryStatusReadHandlerEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_status_handler_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_missing"; Needle = '"rollback_preview_authorization_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_preview_authorization_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_preview_authorization_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_preview"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.rollback_preview_authorization_diagnostic' },
        @{ Suffix = "preview_schema"; Needle = '"rollback_preview_authorization_schema": "raios.recovery_rollback_preview_authorization.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"rollback_preview_authorization_canonicalization": "raios.recovery_rollback_preview_authorization.canonical.v0"' },
        @{ Suffix = "preview_boundary"; Needle = '"rollback_preview_authorization_boundary_id": "boundary.recovery_rollback_preview_authorization.current_boot"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_preview_authorization_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_preview_authorization_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "rollback_preview_authorization_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "rollback_preview_authorization_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_rollback_preview_authorization"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authorization_case"; Needle = '"case": "rollback_preview_authorization_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "rollback_preview_authorization_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_status_read_handler_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_rollback_preview_authorization_still_non_executable"' },
        @{ Suffix = "preview_false"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackPreviewAuthorizationBoundaryId = "boundary.recovery_rollback_preview_authorization.current_boot"
    $recoveryRollbackPreviewProjectionCanonical = @(
        "schema=raios.recovery_rollback_preview_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRollbackPreviewProjectionHash = Get-TextSha256 -Text $recoveryRollbackPreviewProjectionCanonical
    $recoveryRollbackPreviewAuthorizationCanonical = @(
        "canonicalization=raios.recovery_rollback_preview_authorization.canonical.v0",
        "schema=raios.recovery_rollback_preview_authorization.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_rollback_preview_authorization",
        "scope=current_boot",
        "retained_recovery_lifeline_status_read_handler_event_id=$recoveryStatusReadHandlerEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "rollback_preview_authorization_id=$recoveryRollbackPreviewAuthorizationBoundaryId",
        "rollback_preview_projection_sha256=$recoveryRollbackPreviewProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackPreviewAuthorizationHash = Get-TextSha256 -Text $recoveryRollbackPreviewAuthorizationCanonical
    $recoveryRollbackPreviewAuthorizationCommand = "agent recovery.rollback_preview_authorization_diagnostic $recoveryRollbackPreviewAuthorizationHash $recoveryStatusReadHandlerEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryCommandDispatchBoundaryId $recoveryRollbackPreviewAuthorizationBoundaryId $recoveryRollbackPreviewProjectionHash"

    Send-AgentCommand -Command $recoveryRollbackPreviewAuthorizationCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "preview_authorization_id"; Needle = "`"rollback_preview_authorization_id`": `"$recoveryRollbackPreviewAuthorizationBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"rollback_preview_projection_hash`": `"sha256:$recoveryRollbackPreviewProjectionHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "preview_false"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackPreviewAuthorizationResponse = Get-LastAgentResponseJson -Method "recovery.rollback_preview_authorization_diagnostic"
    $recoveryRollbackPreviewAuthorizationEventId = [string]$recoveryRollbackPreviewAuthorizationResponse.body.result.retained_rollback_preview_authorization_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_preview_authorization_retained_reference_event_id_captured" -Value $recoveryRollbackPreviewAuthorizationEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_preview_auth_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_missing"; Needle = '"rollback_apply_authorization_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_apply_authorization_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_apply_authorization_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_apply"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.rollback_apply_authorization_diagnostic' },
        @{ Suffix = "apply_schema"; Needle = '"rollback_apply_authorization_schema": "raios.recovery_rollback_apply_authorization.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"rollback_apply_authorization_canonicalization": "raios.recovery_rollback_apply_authorization.canonical.v0"' },
        @{ Suffix = "apply_boundary"; Needle = '"rollback_apply_authorization_boundary_id": "boundary.recovery_rollback_apply_authorization.current_boot"' },
        @{ Suffix = "disable_target_fact"; Needle = '"fact": "disable_module_target_binding"' },
        @{ Suffix = "restart_target_fact"; Needle = '"fact": "restart_last_good_target_binding"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_apply_authorization_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_apply_authorization_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "rollback_apply_authorization_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "rollback_apply_authorization_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_rollback_apply_authorization"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authorization_case"; Needle = '"case": "rollback_apply_authorization_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "rollback_apply_authorization_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_rollback_preview_authorization_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_rollback_apply_authorization_still_non_executable"' },
        @{ Suffix = "apply_false"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackApplyAuthorizationBoundaryId = "boundary.recovery_rollback_apply_authorization.current_boot"
    $recoveryRollbackApplyProjectionCanonical = @(
        "schema=raios.recovery_rollback_apply_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRollbackApplyProjectionHash = Get-TextSha256 -Text $recoveryRollbackApplyProjectionCanonical
    $recoveryRollbackApplyAuthorizationCanonical = @(
        "canonicalization=raios.recovery_rollback_apply_authorization.canonical.v0",
        "schema=raios.recovery_rollback_apply_authorization.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_rollback_apply_authorization",
        "scope=current_boot",
        "retained_recovery_rollback_preview_authorization_event_id=$recoveryRollbackPreviewAuthorizationEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "rollback_apply_authorization_id=$recoveryRollbackApplyAuthorizationBoundaryId",
        "rollback_apply_projection_sha256=$recoveryRollbackApplyProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackApplyAuthorizationHash = Get-TextSha256 -Text $recoveryRollbackApplyAuthorizationCanonical
    $recoveryRollbackApplyAuthorizationCommand = "agent recovery.rollback_apply_authorization_diagnostic $recoveryRollbackApplyAuthorizationHash $recoveryRollbackPreviewAuthorizationEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryCommandDispatchBoundaryId $recoveryRollbackApplyAuthorizationBoundaryId $recoveryRollbackApplyProjectionHash"

    Send-AgentCommand -Command $recoveryRollbackApplyAuthorizationCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "apply_authorization_id"; Needle = "`"rollback_apply_authorization_id`": `"$recoveryRollbackApplyAuthorizationBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"rollback_apply_projection_hash`": `"sha256:$recoveryRollbackApplyProjectionHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "apply_false"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackApplyAuthorizationResponse = Get-LastAgentResponseJson -Method "recovery.rollback_apply_authorization_diagnostic"
    $recoveryRollbackApplyAuthorizationEventId = [string]$recoveryRollbackApplyAuthorizationResponse.body.result.retained_rollback_apply_authorization_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_apply_authorization_retained_reference_event_id_captured" -Value $recoveryRollbackApplyAuthorizationEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_apply_auth_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_missing"; Needle = '"disable_module_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.disable_module_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_disable_module_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_disable"; Needle = '"disables_module": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.disable_module_target_binding_diagnostic' },
        @{ Suffix = "disable_schema"; Needle = '"disable_module_target_binding_schema": "raios.recovery_disable_module_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"disable_module_target_binding_canonicalization": "raios.recovery_disable_module_target_binding.canonical.v0"' },
        @{ Suffix = "disable_boundary"; Needle = '"disable_module_target_binding_boundary_id": "boundary.recovery_disable_module_target_binding.current_boot"' },
        @{ Suffix = "restart_target_fact"; Needle = '"fact": "restart_last_good_target_binding"' },
        @{ Suffix = "load_target_fact"; Needle = '"fact": "load_artifact_by_hash_target_binding"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.disable_module_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_disable_module_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "disable_module_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "disable_module_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_disable_module_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "disable_module_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "disable_module_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_rollback_apply_authorization_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_disable_module_target_binding_still_non_executable"' },
        @{ Suffix = "disable_false"; Needle = '"disables_module": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryDisableModuleTargetBindingBoundaryId = "boundary.recovery_disable_module_target_binding.current_boot"
    $recoveryDisableModuleTargetProjectionCanonical = @(
        "schema=raios.recovery_disable_module_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryDisableModuleTargetProjectionHash = Get-TextSha256 -Text $recoveryDisableModuleTargetProjectionCanonical
    $recoveryDisableModuleTargetBindingCanonical = @(
        "canonicalization=raios.recovery_disable_module_target_binding.canonical.v0",
        "schema=raios.recovery_disable_module_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_disable_module_target_binding",
        "scope=current_boot",
        "retained_recovery_rollback_apply_authorization_event_id=$recoveryRollbackApplyAuthorizationEventId",
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
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "disable_module_target_id=$recoveryDisableModuleTargetBindingBoundaryId",
        "disable_module_target_projection_sha256=$recoveryDisableModuleTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "disables_module=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryDisableModuleTargetBindingHash = Get-TextSha256 -Text $recoveryDisableModuleTargetBindingCanonical
    $recoveryDisableModuleTargetBindingCommand = "agent recovery.disable_module_target_binding_diagnostic $recoveryDisableModuleTargetBindingHash $recoveryRollbackApplyAuthorizationEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryCommandDispatchBoundaryId $recoveryDisableModuleTargetBindingBoundaryId $recoveryDisableModuleTargetProjectionHash"

    Send-AgentCommand -Command $recoveryDisableModuleTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "disable_target_id"; Needle = "`"disable_module_target_id`": `"$recoveryDisableModuleTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"disable_module_target_projection_hash`": `"sha256:$recoveryDisableModuleTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "disable_false"; Needle = '"disables_module": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryDisableModuleTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.disable_module_target_binding_diagnostic"
    $recoveryDisableModuleTargetBindingEventId = [string]$recoveryDisableModuleTargetBindingResponse.body.result.retained_disable_module_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_disable_module_target_binding_retained_reference_event_id_captured" -Value $recoveryDisableModuleTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_disable_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_missing"; Needle = '"restart_last_good_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.restart_last_good_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_restart_last_good_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_restart"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.restart_last_good_target_binding_diagnostic' },
        @{ Suffix = "restart_schema"; Needle = '"restart_last_good_target_binding_schema": "raios.recovery_restart_last_good_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"restart_last_good_target_binding_canonicalization": "raios.recovery_restart_last_good_target_binding.canonical.v0"' },
        @{ Suffix = "restart_boundary"; Needle = '"restart_last_good_target_binding_boundary_id": "boundary.recovery_restart_last_good_target_binding.current_boot"' },
        @{ Suffix = "load_target_fact"; Needle = '"fact": "load_artifact_by_hash_target_binding"' },
        @{ Suffix = "memory_authority_fact"; Needle = '"fact": "recovery_memory_write_authority"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.restart_last_good_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_restart_last_good_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "restart_last_good_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "restart_last_good_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_restart_last_good_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "restart_last_good_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "restart_last_good_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_disable_module_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_restart_last_good_target_binding_still_non_executable"' },
        @{ Suffix = "restart_false"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRestartLastGoodTargetBindingBoundaryId = "boundary.recovery_restart_last_good_target_binding.current_boot"
    $recoveryRestartLastGoodTargetProjectionCanonical = @(
        "schema=raios.recovery_restart_last_good_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRestartLastGoodTargetProjectionHash = Get-TextSha256 -Text $recoveryRestartLastGoodTargetProjectionCanonical
    $recoveryRestartLastGoodTargetBindingCanonical = @(
        "canonicalization=raios.recovery_restart_last_good_target_binding.canonical.v0",
        "schema=raios.recovery_restart_last_good_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_restart_last_good_target_binding",
        "scope=current_boot",
        "retained_recovery_disable_module_target_binding_event_id=$recoveryDisableModuleTargetBindingEventId",
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
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "restart_last_good_target_id=$recoveryRestartLastGoodTargetBindingBoundaryId",
        "restart_last_good_target_projection_sha256=$recoveryRestartLastGoodTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "restarts_last_good=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRestartLastGoodTargetBindingHash = Get-TextSha256 -Text $recoveryRestartLastGoodTargetBindingCanonical
    $recoveryRestartLastGoodTargetBindingCommand = "agent recovery.restart_last_good_target_binding_diagnostic $recoveryRestartLastGoodTargetBindingHash $recoveryDisableModuleTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryRestartLastGoodTargetBindingBoundaryId $recoveryRestartLastGoodTargetProjectionHash"

    Send-AgentCommand -Command $recoveryRestartLastGoodTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "restart_target_id"; Needle = "`"restart_last_good_target_id`": `"$recoveryRestartLastGoodTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"restart_last_good_target_projection_hash`": `"sha256:$recoveryRestartLastGoodTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "restart_false"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRestartLastGoodTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.restart_last_good_target_binding_diagnostic"
    $recoveryRestartLastGoodTargetBindingEventId = [string]$recoveryRestartLastGoodTargetBindingResponse.body.result.retained_restart_last_good_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_restart_last_good_target_binding_retained_reference_event_id_captured" -Value $recoveryRestartLastGoodTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_restart_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_missing"; Needle = '"load_artifact_by_hash_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_artifact_by_hash_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_load_artifact_by_hash_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authorize"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.load_artifact_by_hash_target_binding_diagnostic' },
        @{ Suffix = "load_schema"; Needle = '"load_artifact_by_hash_target_binding_schema": "raios.recovery_load_artifact_by_hash_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"load_artifact_by_hash_target_binding_canonicalization": "raios.recovery_load_artifact_by_hash_target_binding.canonical.v0"' },
        @{ Suffix = "load_boundary"; Needle = '"load_artifact_by_hash_target_binding_boundary_id": "boundary.recovery_load_artifact_by_hash_target_binding.current_boot"' },
        @{ Suffix = "memory_authority_fact"; Needle = '"fact": "recovery_memory_write_authority"' },
        @{ Suffix = "durable_authority_fact"; Needle = '"fact": "durable_audit_rollback_write_authority"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_artifact_by_hash_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_load_artifact_by_hash_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "load_artifact_by_hash_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "load_artifact_by_hash_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_load_artifact_by_hash_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "load_artifact_by_hash_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "load_artifact_by_hash_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_restart_last_good_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_load_artifact_by_hash_target_binding_still_non_executable"' },
        @{ Suffix = "load_false"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoadArtifactByHashTargetBindingBoundaryId = "boundary.recovery_load_artifact_by_hash_target_binding.current_boot"
    $recoveryLoadArtifactByHashTargetProjectionCanonical = @(
        "schema=raios.recovery_load_artifact_by_hash_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "artifact_hash=$recoveryArtifactHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryLoadArtifactByHashTargetProjectionHash = Get-TextSha256 -Text $recoveryLoadArtifactByHashTargetProjectionCanonical
    $recoveryLoadArtifactByHashTargetBindingCanonical = @(
        "canonicalization=raios.recovery_load_artifact_by_hash_target_binding.canonical.v0",
        "schema=raios.recovery_load_artifact_by_hash_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_load_artifact_by_hash_target_binding",
        "scope=current_boot",
        "retained_recovery_restart_last_good_target_binding_event_id=$recoveryRestartLastGoodTargetBindingEventId",
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
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "load_artifact_by_hash_target_id=$recoveryLoadArtifactByHashTargetBindingBoundaryId",
        "load_artifact_by_hash_target_artifact_sha256=$recoveryArtifactHash",
        "load_artifact_by_hash_target_projection_sha256=$recoveryLoadArtifactByHashTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLoadArtifactByHashTargetBindingHash = Get-TextSha256 -Text $recoveryLoadArtifactByHashTargetBindingCanonical
    $recoveryLoadArtifactByHashTargetBindingCommand = "agent recovery.load_artifact_by_hash_target_binding_diagnostic $recoveryLoadArtifactByHashTargetBindingHash $recoveryRestartLastGoodTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryLoadArtifactByHashTargetBindingBoundaryId $recoveryArtifactHash $recoveryLoadArtifactByHashTargetProjectionHash"

    Send-AgentCommand -Command $recoveryLoadArtifactByHashTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "load_target_id"; Needle = "`"load_artifact_by_hash_target_id`": `"$recoveryLoadArtifactByHashTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "restart_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "artifact_hash"; Needle = "`"load_artifact_by_hash_target_artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"load_artifact_by_hash_target_projection_hash`": `"sha256:$recoveryLoadArtifactByHashTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "load_false"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authorize"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoadArtifactByHashTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.load_artifact_by_hash_target_binding_diagnostic"
    $recoveryLoadArtifactByHashTargetBindingEventId = [string]$recoveryLoadArtifactByHashTargetBindingResponse.body.result.retained_load_artifact_by_hash_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_load_artifact_by_hash_target_binding_retained_reference_event_id_captured" -Value $recoveryLoadArtifactByHashTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_load_hash_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_present"; Needle = '"load_artifact_by_hash_target_binding_present": true' },
        @{ Suffix = "memory_authority_missing"; Needle = '"recovery_memory_write_authority_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_write_authority_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_write_authority_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_write_authority_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_write"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.memory_write_authority_diagnostic' },
        @{ Suffix = "memory_schema"; Needle = '"recovery_memory_write_authority_schema": "raios.recovery_memory_write_authority.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"recovery_memory_write_authority_canonicalization": "raios.recovery_memory_write_authority.canonical.v0"' },
        @{ Suffix = "memory_boundary"; Needle = '"recovery_memory_write_authority_boundary_id": "boundary.recovery_memory_write_authority.current_boot"' },
        @{ Suffix = "durable_authority_fact"; Needle = '"fact": "durable_audit_rollback_write_authority"' },
        @{ Suffix = "service_boundary_fact"; Needle = '"fact": "service_inventory_side_effect_boundary"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_write_authority_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_write_authority_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_write_authority_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_memory_write_authority_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_memory_write_authority_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_memory_write_authority"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authority_case"; Needle = '"case": "recovery_memory_write_authority_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "recovery_memory_write_authority_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_load_artifact_by_hash_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_recovery_memory_write_authority_still_non_executable"' },
        @{ Suffix = "write_false"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryMemoryWriteAuthorityBoundaryId = "boundary.recovery_memory_write_authority.current_boot"
    $recoveryMemoryProjectionCanonical = @(
        "schema=raios.recovery_memory_write_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryMemoryProjectionHash = Get-TextSha256 -Text $recoveryMemoryProjectionCanonical
    $recoveryMemoryWriteAuthorityCanonical = @(
        "canonicalization=raios.recovery_memory_write_authority.canonical.v0",
        "schema=raios.recovery_memory_write_authority.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_memory_write_authority",
        "scope=current_boot",
        "retained_recovery_load_artifact_by_hash_target_binding_event_id=$recoveryLoadArtifactByHashTargetBindingEventId",
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
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "recovery_memory_write_authority_id=$recoveryMemoryWriteAuthorityBoundaryId",
        "recovery_memory_projection_sha256=$recoveryMemoryProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "writes_recovery_memory=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryMemoryWriteAuthorityHash = Get-TextSha256 -Text $recoveryMemoryWriteAuthorityCanonical
    $recoveryMemoryWriteAuthorityCommand = "agent recovery.memory_write_authority_diagnostic $recoveryMemoryWriteAuthorityHash $recoveryLoadArtifactByHashTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryMemoryWriteAuthorityBoundaryId $recoveryMemoryProjectionHash"

    Send-AgentCommand -Command $recoveryMemoryWriteAuthorityCommand -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_valid_but_memory_writes_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_memory_write_authority_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "memory_authority_id"; Needle = "`"recovery_memory_write_authority_id`": `"$recoveryMemoryWriteAuthorityBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "restart_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "load_hash"; Needle = "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"recovery_memory_projection_hash`": `"sha256:$recoveryMemoryProjectionHash`"" },
        @{ Suffix = "authority_hash"; Needle = "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "write_false"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryMemoryWriteAuthorityResponse = Get-LastAgentResponseJson -Method "recovery.memory_write_authority_diagnostic"
    $recoveryMemoryWriteAuthorityEventId = [string]$recoveryMemoryWriteAuthorityResponse.body.result.retained_recovery_memory_write_authority_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_memory_write_authority_retained_reference_event_id_captured" -Value $recoveryMemoryWriteAuthorityEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_memory_authority_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "durable_audit_rollback_write_authority_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_present"; Needle = '"load_artifact_by_hash_target_binding_present": true' },
        @{ Suffix = "memory_authority_present"; Needle = '"recovery_memory_write_authority_present": true' },
        @{ Suffix = "durable_authority_missing"; Needle = '"durable_audit_rollback_write_authority_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.durable_audit_rollback_write_authority_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:durable_audit_rollback_write_authority_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.durable_audit_rollback_write_authority_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "durable_audit_rollback_write_authority_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_durable_audit_rollback_write_authority_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_durable_write"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "no_rollback_write"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.durable_audit_rollback_write_authority_diagnostic' },
        @{ Suffix = "durable_schema"; Needle = '"durable_audit_rollback_write_authority_schema": "raios.durable_audit_rollback_write_authority.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"durable_audit_rollback_write_authority_canonicalization": "raios.durable_audit_rollback_write_authority.canonical.v0"' },
        @{ Suffix = "durable_boundary"; Needle = '"durable_audit_rollback_write_authority_boundary_id": "boundary.durable_audit_rollback_write_authority.current_boot"' },
        @{ Suffix = "service_boundary_fact"; Needle = '"fact": "service_inventory_side_effect_boundary"' },
        @{ Suffix = "behavior_fact"; Needle = '"fact": "recovery_lifeline_command_dispatch_behavior"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.durable_audit_rollback_write_authority_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_write_authority_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:durable_audit_rollback_write_authority_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.durable_audit_rollback_write_authority_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_durable_audit_rollback_write_authority_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "durable_audit_rollback_write_authority_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "durable_audit_rollback_write_authority_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_durable_audit_rollback_write_authority"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authority_case"; Needle = '"case": "durable_audit_rollback_write_authority_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "durable_audit_rollback_write_authority_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_recovery_memory_write_authority_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_durable_audit_rollback_write_authority_still_non_executable"' },
        @{ Suffix = "durable_write_false"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "rollback_write_false"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $durableAuditRollbackWriteAuthorityBoundaryId = "boundary.durable_audit_rollback_write_authority.current_boot"
    $durableAuditRollbackProjectionCanonical = @(
        "schema=raios.durable_audit_rollback_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "recovery_memory_write_authority_hash=$recoveryMemoryWriteAuthorityHash",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $durableAuditRollbackProjectionHash = Get-TextSha256 -Text $durableAuditRollbackProjectionCanonical
    $durableAuditRollbackWriteAuthorityCanonical = @(
        "canonicalization=raios.durable_audit_rollback_write_authority.canonical.v0",
        "schema=raios.durable_audit_rollback_write_authority.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=durable_audit_rollback_write_authority",
        "scope=current_boot",
        "retained_recovery_memory_write_authority_event_id=$recoveryMemoryWriteAuthorityEventId",
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
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "durable_audit_rollback_write_authority_id=$durableAuditRollbackWriteAuthorityBoundaryId",
        "durable_audit_rollback_projection_sha256=$durableAuditRollbackProjectionHash",
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
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $durableAuditRollbackWriteAuthorityHash = Get-TextSha256 -Text $durableAuditRollbackWriteAuthorityCanonical
    $durableAuditRollbackWriteAuthorityCommand = "agent recovery.durable_audit_rollback_write_authority_diagnostic $durableAuditRollbackWriteAuthorityHash $recoveryMemoryWriteAuthorityEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryMemoryWriteAuthorityHash $recoveryCommandDispatchBoundaryId $durableAuditRollbackWriteAuthorityBoundaryId $durableAuditRollbackProjectionHash"

    Send-AgentCommand -Command $durableAuditRollbackWriteAuthorityCommand -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:durable_audit_rollback_write_authority_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "durable_audit_rollback_write_authority_valid_but_durable_writes_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_durable_audit_rollback_write_authority_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "durable_authority_id"; Needle = "`"durable_audit_rollback_write_authority_id`": `"$durableAuditRollbackWriteAuthorityBoundaryId`"" },
        @{ Suffix = "memory_event_id"; Needle = "`"retained_recovery_memory_write_authority_event_id`": `"$recoveryMemoryWriteAuthorityEventId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "restart_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "load_hash"; Needle = "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" },
        @{ Suffix = "memory_hash"; Needle = "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"durable_audit_rollback_projection_hash`": `"sha256:$durableAuditRollbackProjectionHash`"" },
        @{ Suffix = "authority_hash"; Needle = "`"durable_audit_rollback_write_authority_hash`": `"sha256:$durableAuditRollbackWriteAuthorityHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "durable_write_false"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "rollback_write_false"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $durableAuditRollbackWriteAuthorityResponse = Get-LastAgentResponseJson -Method "recovery.durable_audit_rollback_write_authority_diagnostic"
    $durableAuditRollbackWriteAuthorityEventId = [string]$durableAuditRollbackWriteAuthorityResponse.body.result.retained_durable_audit_rollback_write_authority_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:durable_audit_rollback_write_authority_retained_reference_event_id_captured" -Value $durableAuditRollbackWriteAuthorityEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_durable_authority_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_service_inventory_side_effect_boundary_missing"' },
        @{ Suffix = "memory_authority_present"; Needle = '"recovery_memory_write_authority_present": true' },
        @{ Suffix = "durable_authority_present"; Needle = '"durable_audit_rollback_write_authority_present": true' },
        @{ Suffix = "service_boundary_missing"; Needle = '"service_inventory_side_effect_boundary_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
