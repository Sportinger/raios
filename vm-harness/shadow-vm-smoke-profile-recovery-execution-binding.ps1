    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_side_effect_gate_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "defined_non_executable"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_execution_enablement_not_implemented"' },
        @{ Suffix = "service_boundary_present"; Needle = '"service_inventory_side_effect_boundary_present": true' },
        @{ Suffix = "behavior_present"; Needle = '"command_dispatch_behavior_present": true' },
        @{ Suffix = "executor_present"; Needle = '"executor_capability_table_present": true' },
        @{ Suffix = "side_effect_present"; Needle = '"side_effect_gate_present": true' },
        @{ Suffix = "enablement_missing"; Needle = '"execution_enablement_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Assert-LogContainsFields -NamePrefix "protocol:status_execution_readiness_after_side_effect_gate_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_execution_readiness.v0"' },
        @{ Suffix = "blocked"; Needle = '"status": "blocked_missing_evidence"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_execution_enablement_not_implemented"' },
        @{ Suffix = "command"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "completion_missing"; Needle = '"execution_completion_denial_present": false' },
        @{ Suffix = "would_execute_false"; Needle = '"would_execute_lifeline_status_read": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command "agent recovery.lifeline_status_execution_result_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_execution_result_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:status_execution_result_after_side_effect_gate_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_execution_result_diagnostic.v0"' },
        @{ Suffix = "blocked"; Needle = '"status": "blocked_missing_evidence"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_execution_enablement_not_implemented"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_status_execution_result_records": false' },
        @{ Suffix = "readiness_blocked"; Needle = '"status_execution_readiness": {' },
        @{ Suffix = "readiness_reason"; Needle = '"reason": "recovery_lifeline_command_execution_enablement_not_implemented"' },
        @{ Suffix = "recorded_null"; Needle = '"recorded_event_id": null' },
        @{ Suffix = "would_execute_false"; Needle = '"would_execute_lifeline_status_read": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command "agent recovery.lifeline_status_result_read" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_result_read"
    Assert-LogContainsFields -NamePrefix "protocol:status_result_read_before_result_record_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_result_read.v0"' },
        @{ Suffix = "denied"; Needle = '"status": "denied_missing_retained_result"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_execution_result_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "source_missing"; Needle = '"status": "missing"' },
        @{ Suffix = "result_hash_null"; Needle = '"status_execution_result_hash": null' },
        @{ Suffix = "projection_unavailable"; Needle = '"status": "unavailable_missing_retained_result"' },
        @{ Suffix = "source_unverified"; Needle = '"source_retained_result_verified": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command "agent recovery.lifeline.status" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline.status"
    Assert-LogContainsFields -NamePrefix "protocol:lifeline_status_command_before_result_record_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_result_read.v0"' },
        @{ Suffix = "denied"; Needle = '"status": "denied_missing_retained_result"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_execution_result_missing"' },
        @{ Suffix = "command_facing"; Needle = '"command_envelope_facing": true' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "source_missing"; Needle = '"status": "missing"' },
        @{ Suffix = "projection_unavailable"; Needle = '"status": "unavailable_missing_retained_result"' },
        @{ Suffix = "source_unverified"; Needle = '"source_retained_result_verified": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryStatusEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.lifeline.status requested_capability=cap.recovery.load_artifact.read classification=local_only"
    Send-AgentCommand -Command $recoveryStatusEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline.status"
    $recoveryStatusEnvelopeBefore = Get-LastAgentResponseJson -Method "agent.command_envelope"
    Assert-CurrentBootEventId -Name "protocol:lifeline_status_envelope_before_event_id" -Value $recoveryStatusEnvelopeBefore.body.result.event_id
    if (-not $recoveryStatusEnvelopeBefore.body.result.accepted -or $recoveryStatusEnvelopeBefore.body.result.reason -ne "accepted" -or -not $recoveryStatusEnvelopeBefore.body.result.dispatches_existing_agent_method) {
        throw "Expected recovery.lifeline.status envelope to dispatch the existing read method before retained result"
    }
    if ($recoveryStatusEnvelopeBefore.body.result.target_method -ne "recovery.lifeline.status" -or $recoveryStatusEnvelopeBefore.body.result.requested_capability -ne "cap.recovery.load_artifact.read" -or $recoveryStatusEnvelopeBefore.body.result.allowed_requested_capability -ne "cap.recovery.load_artifact.read") {
        throw "Expected recovery.lifeline.status envelope to bind the recovery read capability"
    }
    if ($recoveryStatusEnvelopeBefore.body.result.creates_parallel_dispatcher -or $recoveryStatusEnvelopeBefore.body.result.loads_candidate_bytes -or $recoveryStatusEnvelopeBefore.body.result.writes_persistent_state -or $recoveryStatusEnvelopeBefore.body.result.writes_durable_audit_log -or $recoveryStatusEnvelopeBefore.body.result.installs_rollback_plan -or $recoveryStatusEnvelopeBefore.body.result.grants_broad_mutation) {
        throw "Expected recovery.lifeline.status envelope to avoid unsafe side effects"
    }
    $recoveryStatusEnvelopeReadBefore = Get-LastAgentResponseJson -Method "recovery.lifeline.status"
    if ($recoveryStatusEnvelopeReadBefore.body.result.status -ne "denied_missing_retained_result" -or $recoveryStatusEnvelopeReadBefore.body.result.command_envelope_facing -ne $true -or $recoveryStatusEnvelopeReadBefore.body.result.dispatches_lifeline_command -or $recoveryStatusEnvelopeReadBefore.body.result.command_execution_enabled -or $recoveryStatusEnvelopeReadBefore.body.result.executes_lifeline_status) {
        throw "Expected enveloped recovery.lifeline.status before retained result to stay read-only and denied"
    }
    $recoveryStatusEnvelopeMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.lifeline.status requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $recoveryStatusEnvelopeMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($recoveryStatusEnvelopeMismatch.body.result.accepted -or $recoveryStatusEnvelopeMismatch.body.result.reason -ne "requested_capability_denied" -or $recoveryStatusEnvelopeMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected recovery.lifeline.status envelope with wrong capability to be denied before dispatch"
    }
    $recoveryStatusMismatchLog = Get-SerialLogContent -Path $SerialLog
    $recoveryStatusMismatchAfter = if ($recoveryStatusMismatchLog.Length -gt $recoveryStatusEnvelopeMismatchOffset) { $recoveryStatusMismatchLog.Substring([int]$recoveryStatusEnvelopeMismatchOffset) } else { "" }
    $recoveryStatusMismatchNoDispatch = -not $recoveryStatusMismatchAfter.Contains("RAIOS_AGENT_END recovery.lifeline.status")
    Add-Predicate -Name "protocol:lifeline_status_envelope_mismatch_no_status_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END recovery.lifeline.status" -Passed $recoveryStatusMismatchNoDispatch -Actual $(if ($recoveryStatusMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $recoveryStatusMismatchNoDispatch) {
        throw "Expected recovery.lifeline.status envelope capability mismatch to avoid status dispatch"
    }

    function Get-RecoveryExecutionStageHash {
        param(
            [string]$Canonicalization,
            [string]$Schema,
            [string]$Resource,
            [string]$RetainedEventField,
            [string]$RetainedEventId,
            [string[]]$PriorStageHashLines,
            [string]$StageIdField,
            [string]$StageId,
            [string]$ProjectionField,
            [string]$ProjectionHash
        )
        $lines = @(
            "canonicalization=$Canonicalization",
            "schema=$Schema",
            "load_mode=recovery_only",
            "subject=agent.session.serial",
            "resource=$Resource",
            "scope=current_boot",
            "$RetainedEventField=$RetainedEventId",
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
            "side_effect_gate_sha256=$recoverySideEffectGateHash",
            "source_rollback_apply_denial_sha256=$recoveryRollbackApplySourceDenialHash",
            "source_durable_policy_write_authority_decision_sha256=$recoveryRollbackApplySourceDurablePolicyDecisionHash",
            "source_recovery_rollback_inspect_source_reference_sha256=$recoveryRollbackApplySourceInspectReferenceHash"
        )
        $lines = @($lines + $PriorStageHashLines + @(
            "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
            "$StageIdField=$StageId",
            "$ProjectionField=$ProjectionHash",
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
        ))
        Get-TextSha256 -Text ($lines -join "`n")
    }

    function Assert-RecoveryExecutionStageLoadDenialSource {
        param(
            [string]$NamePrefix,
            [string]$SchemaSuffix,
            [string]$Schema,
            [object[]]$StageFields = @()
        )

        $fields = @(
            @{ Suffix = $SchemaSuffix; Needle = "`"schema`": `"$Schema`"" },
            @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
            @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
            @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
            @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
            @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
            @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
            @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
            @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
            @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
            @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
            @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" }
        )
        $fields = @($fields + $StageFields + @(
            @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_execution_stage_records": false' },
            @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
            @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
            @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
            @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
            @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
            @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
            @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
        ))

        Assert-LogContainsFields -NamePrefix $NamePrefix -TimeoutSeconds 1 -Fields $fields
    }

    function Invoke-RecoveryExecutionStage {
        param(
            [string]$StageName,
            [string]$Method,
            [string]$SelftestMethod,
            [string]$DiagnosticSchema,
            [string]$SelftestSchema,
            [string]$ReferenceSchema,
            [string]$Canonicalization,
            [string]$Resource,
            [string]$StageHashName,
            [string]$StageIdField,
            [string]$StageId,
            [string]$ProjectionField,
            [string]$ProjectionHash,
            [string]$RetainedEventField,
            [string]$RetainedEventId,
            [string[]]$PriorStageHashArgs,
            [string[]]$PriorStageHashLines,
            [string]$AbsentReason,
            [string]$ValidReason,
            [string]$NextDispatchReason,
            [string]$NextPresentNeedle,
            [string]$PreviousEventNeedleName
        )

        Send-AgentCommand -Command "agent $Method" -ExpectedMarker "RAIOS_AGENT_END $Method"
        Assert-LogContainsFields -NamePrefix "protocol:$($StageName)_absent_" -TimeoutSeconds 1 -Fields @(
            @{ Suffix = "schema"; Needle = "`"schema`": `"$DiagnosticSchema`"" },
            @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
            @{ Suffix = "status"; Needle = '"status": "missing"' },
            @{ Suffix = "reason"; Needle = "`"reason`": `"$AbsentReason`"" },
            @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_execution_stage_records": false' },
            @{ Suffix = "stage_schema"; Needle = "`"execution_stage_schema`": `"$ReferenceSchema`"" },
            @{ Suffix = "stage_id"; Needle = "`"execution_stage_id`": `"$StageId`"" },
            @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
            @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' }
        )

        Send-AgentCommand -Command "agent $SelftestMethod" -ExpectedMarker "RAIOS_AGENT_END $SelftestMethod"
        Assert-LogContainsFields -NamePrefix "protocol:$($StageName)_selftest_" -TimeoutSeconds 1 -Fields @(
            @{ Suffix = "schema"; Needle = "`"schema`": `"$SelftestSchema`"" },
            @{ Suffix = "case_count"; Needle = '"case_count": 10' },
            @{ Suffix = "passed"; Needle = '"passed": true' },
            @{ Suffix = "absent_case"; Needle = "`"case`": `"$AbsentReason`"" },
            @{ Suffix = "valid_case"; Needle = "`"case`": `"$ValidReason`"" },
            @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
            @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' }
        )

        $stageHash = Get-RecoveryExecutionStageHash -Canonicalization $Canonicalization -Schema $ReferenceSchema -Resource $Resource -RetainedEventField $RetainedEventField -RetainedEventId $RetainedEventId -PriorStageHashLines $PriorStageHashLines -StageIdField $StageIdField -StageId $StageId -ProjectionField $ProjectionField -ProjectionHash $ProjectionHash
        $commandParts = @(
            "agent",
            $Method,
            $stageHash,
            $RetainedEventId,
            "recovery.lifeline.status",
            "raios.recovery_lifeline_command.status_args.v0",
            $recoveryLifelineStatusArgumentHash,
            $recoveryCommandTargetLocator,
            $recoveryLifelineCommandEnvelopeReferenceHash,
            $recoveryLifelineCommandBodyCanonicalizationHash,
            $recoveryCommandHandlerBindingHash,
            $recoveryStatusReadHandlerHash,
            $recoveryRollbackPreviewAuthorizationHash,
            $recoveryRollbackApplyAuthorizationHash,
            $recoveryDisableModuleTargetBindingHash,
            $recoveryRestartLastGoodTargetBindingHash,
            $recoveryLoadArtifactByHashTargetBindingHash,
            $recoveryMemoryWriteAuthorityHash,
            $durableAuditRollbackWriteAuthorityHash,
            $serviceInventorySideEffectBoundaryHash,
            $recoveryCommandDispatchBehaviorHash,
            $recoveryExecutorCapabilityTableHash,
            $recoverySideEffectGateHash,
            $recoveryRollbackApplySourceDenialHash,
            $recoveryRollbackApplySourceDurablePolicyDecisionHash,
            $recoveryRollbackApplySourceInspectReferenceHash
        ) + $PriorStageHashArgs + @(
            $recoveryCommandDispatchBoundaryId,
            $StageId,
            $ProjectionHash
        )
        Send-AgentCommand -Command ($commandParts -join " ") -ExpectedMarker "RAIOS_AGENT_END $Method"
        Assert-LogContainsFields -NamePrefix "protocol:$($StageName)_valid_" -TimeoutSeconds 1 -Fields @(
            @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
            @{ Suffix = "reason"; Needle = "`"reason`": `"$ValidReason`"" },
            @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_execution_stage_records": true' },
            @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
            @{ Suffix = "stage_id"; Needle = "`"execution_stage_id`": `"$StageId`"" },
            @{ Suffix = "stage_hash"; Needle = "`"$StageHashName`": `"sha256:$stageHash`"" },
            @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
            @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
            @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
            @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
            @{ Suffix = "previous_event"; Needle = "`"$PreviousEventNeedleName`": `"$RetainedEventId`"" },
            @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
            @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' }
        )

        $response = Get-LastAgentResponseJson -Method $Method
        $eventId = [string]$response.body.result.retained_recovery_lifeline_command_execution_stage_reference.recorded_event_id
        Assert-CurrentBootEventId -Name "protocol:$($StageName)_retained_reference_event_id_captured" -Value $eventId

        Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
        Assert-LogContainsFields -NamePrefix "protocol:dispatch_after_$($StageName)_" -TimeoutSeconds 1 -Fields @(
            @{ Suffix = "status"; Needle = '"status": "defined_non_executable"' },
            @{ Suffix = "reason"; Needle = "`"reason`": `"$NextDispatchReason`"" },
            @{ Suffix = "stage_present"; Needle = $NextPresentNeedle },
            @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
            @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' }
        )
        if ($StageName -eq "recovery_lifeline_command_execution_completion_denial") {
            Assert-LogContainsFields -NamePrefix "protocol:status_execution_readiness_after_completion_denial_" -TimeoutSeconds 1 -Fields @(
                @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_execution_readiness.v0"' },
                @{ Suffix = "available"; Needle = '"status": "available_read_only_non_authorizing"' },
                @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_ready_command_execution_disabled"' },
                @{ Suffix = "command"; Needle = '"command_id": "recovery.lifeline.status"' },
                @{ Suffix = "argument_schema"; Needle = '"argument_schema": "raios.recovery_lifeline_command.status_args.v0"' },
                @{ Suffix = "handler_event"; Needle = "`"retained_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" },
                @{ Suffix = "handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
                @{ Suffix = "projection_hash"; Needle = "`"status_read_projection_hash`": `"sha256:$recoveryStatusReadProjectionHash`"" },
                @{ Suffix = "handler_id"; Needle = "`"status_handler_id`": `"$recoveryStatusReadHandlerBoundaryId`"" },
                @{ Suffix = "completion_present"; Needle = '"execution_completion_denial_present": true' },
                @{ Suffix = "would_execute_true"; Needle = '"would_execute_lifeline_status_read": true' },
                @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
                @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
                @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
                @{ Suffix = "no_memory_write"; Needle = '"writes_recovery_memory": false' },
                @{ Suffix = "no_durable_write"; Needle = '"writes_durable_audit_log": false' },
                @{ Suffix = "no_rollback_write"; Needle = '"writes_rollback_store": false' },
                @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
                @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
                @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
                @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
            )
        }
        return [pscustomobject]@{
            Hash = $stageHash
            EventId = $eventId
        }
    }

    $recoveryExecutionEnablementId = "boundary.recovery_lifeline_command_execution_enablement.current_boot"
    $recoveryExecutionEnablementProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_enablement_projection.v0",
        "command_id=recovery.lifeline.status",
        "side_effect_gate_hash=$recoverySideEffectGateHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionEnablement = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_enablement" -Method "recovery.lifeline_command_execution_enablement_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_enablement_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_enablement_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_enablement_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_enablement.v0" -Canonicalization "raios.recovery_lifeline_command_execution_enablement.canonical.v0" -Resource "recovery_lifeline_command_execution_enablement" -StageHashName "execution_enablement_hash" -StageIdField "execution_enablement_id" -StageId $recoveryExecutionEnablementId -ProjectionField "execution_projection_sha256" -ProjectionHash $recoveryExecutionEnablementProjectionHash -RetainedEventField "retained_side_effect_gate_event_id" -RetainedEventId $recoverySideEffectGateEventId -PriorStageHashArgs @() -PriorStageHashLines @() -AbsentReason "recovery_lifeline_command_execution_enablement_absent" -ValidReason "recovery_lifeline_command_execution_enablement_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_preflight_not_implemented" -NextPresentNeedle '"execution_enablement_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionEnablementHash = [string]$executionEnablement.Hash
    $recoveryExecutionEnablementEventId = [string]$executionEnablement.EventId

    $recoveryExecutionPreflightId = "boundary.recovery_lifeline_command_execution_preflight.current_boot"
    $recoveryExecutionPreflightProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_preflight_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_enablement_hash=$recoveryExecutionEnablementHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionPreflight = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_preflight" -Method "recovery.lifeline_command_execution_preflight_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_preflight_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_preflight_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_preflight_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_preflight.v0" -Canonicalization "raios.recovery_lifeline_command_execution_preflight.canonical.v0" -Resource "recovery_lifeline_command_execution_preflight" -StageHashName "execution_preflight_hash" -StageIdField "execution_preflight_id" -StageId $recoveryExecutionPreflightId -ProjectionField "execution_preflight_projection_sha256" -ProjectionHash $recoveryExecutionPreflightProjectionHash -RetainedEventField "retained_execution_enablement_event_id" -RetainedEventId $recoveryExecutionEnablementEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash") -AbsentReason "recovery_lifeline_command_execution_preflight_absent" -ValidReason "recovery_lifeline_command_execution_preflight_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_intent_not_implemented" -NextPresentNeedle '"execution_preflight_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionPreflightHash = [string]$executionPreflight.Hash
    $recoveryExecutionPreflightEventId = [string]$executionPreflight.EventId

    $recoveryExecutionIntentId = "boundary.recovery_lifeline_command_execution_intent.current_boot"
    $recoveryExecutionIntentProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_intent_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_preflight_hash=$recoveryExecutionPreflightHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionIntent = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_intent" -Method "recovery.lifeline_command_execution_intent_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_intent_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_intent_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_intent_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_intent.v0" -Canonicalization "raios.recovery_lifeline_command_execution_intent.canonical.v0" -Resource "recovery_lifeline_command_execution_intent" -StageHashName "execution_intent_hash" -StageIdField "execution_intent_id" -StageId $recoveryExecutionIntentId -ProjectionField "execution_intent_projection_sha256" -ProjectionHash $recoveryExecutionIntentProjectionHash -RetainedEventField "retained_execution_preflight_event_id" -RetainedEventId $recoveryExecutionPreflightEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash") -AbsentReason "recovery_lifeline_command_execution_intent_absent" -ValidReason "recovery_lifeline_command_execution_intent_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_commit_gate_not_implemented" -NextPresentNeedle '"execution_intent_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionIntentHash = [string]$executionIntent.Hash
    $recoveryExecutionIntentEventId = [string]$executionIntent.EventId

    $recoveryExecutionCommitGateId = "boundary.recovery_lifeline_command_execution_commit_gate.current_boot"
    $recoveryExecutionCommitGateProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_commit_gate_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_intent_hash=$recoveryExecutionIntentHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionCommitGate = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_commit_gate" -Method "recovery.lifeline_command_execution_commit_gate_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_commit_gate_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_commit_gate_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_commit_gate_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_commit_gate.v0" -Canonicalization "raios.recovery_lifeline_command_execution_commit_gate.canonical.v0" -Resource "recovery_lifeline_command_execution_commit_gate" -StageHashName "execution_commit_gate_hash" -StageIdField "execution_commit_gate_id" -StageId $recoveryExecutionCommitGateId -ProjectionField "execution_commit_gate_projection_sha256" -ProjectionHash $recoveryExecutionCommitGateProjectionHash -RetainedEventField "retained_execution_intent_event_id" -RetainedEventId $recoveryExecutionIntentEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash, $recoveryExecutionIntentHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash", "execution_intent_sha256=$recoveryExecutionIntentHash") -AbsentReason "recovery_lifeline_command_execution_commit_gate_absent" -ValidReason "recovery_lifeline_command_execution_commit_gate_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_result_denial_not_implemented" -NextPresentNeedle '"execution_commit_gate_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionCommitGateHash = [string]$executionCommitGate.Hash
    $recoveryExecutionCommitGateEventId = [string]$executionCommitGate.EventId

    $recoveryExecutionResultDenialId = "boundary.recovery_lifeline_command_execution_result_denial.current_boot"
    $recoveryExecutionResultDenialProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_result_denial_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_commit_gate_hash=$recoveryExecutionCommitGateHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionResultDenial = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_result_denial" -Method "recovery.lifeline_command_execution_result_denial_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_result_denial_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_result_denial_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_result_denial_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_result_denial.v0" -Canonicalization "raios.recovery_lifeline_command_execution_result_denial.canonical.v0" -Resource "recovery_lifeline_command_execution_result_denial" -StageHashName "execution_result_denial_hash" -StageIdField "execution_result_denial_id" -StageId $recoveryExecutionResultDenialId -ProjectionField "execution_result_projection_sha256" -ProjectionHash $recoveryExecutionResultDenialProjectionHash -RetainedEventField "retained_execution_commit_gate_event_id" -RetainedEventId $recoveryExecutionCommitGateEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash, $recoveryExecutionIntentHash, $recoveryExecutionCommitGateHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash", "execution_intent_sha256=$recoveryExecutionIntentHash", "execution_commit_gate_sha256=$recoveryExecutionCommitGateHash") -AbsentReason "recovery_lifeline_command_execution_result_denial_absent" -ValidReason "recovery_lifeline_command_execution_result_denial_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_audit_denial_not_implemented" -NextPresentNeedle '"execution_result_denial_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionResultDenialHash = [string]$executionResultDenial.Hash
    $recoveryExecutionResultDenialEventId = [string]$executionResultDenial.EventId

    $recoveryExecutionAuditDenialId = "boundary.recovery_lifeline_command_execution_audit_denial.current_boot"
    $recoveryExecutionAuditDenialProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_audit_denial_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_result_denial_hash=$recoveryExecutionResultDenialHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionAuditDenial = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_audit_denial" -Method "recovery.lifeline_command_execution_audit_denial_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_audit_denial_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_audit_denial_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_audit_denial_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_audit_denial.v0" -Canonicalization "raios.recovery_lifeline_command_execution_audit_denial.canonical.v0" -Resource "recovery_lifeline_command_execution_audit_denial" -StageHashName "execution_audit_denial_hash" -StageIdField "execution_audit_denial_id" -StageId $recoveryExecutionAuditDenialId -ProjectionField "execution_audit_projection_sha256" -ProjectionHash $recoveryExecutionAuditDenialProjectionHash -RetainedEventField "retained_execution_result_denial_event_id" -RetainedEventId $recoveryExecutionResultDenialEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash, $recoveryExecutionIntentHash, $recoveryExecutionCommitGateHash, $recoveryExecutionResultDenialHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash", "execution_intent_sha256=$recoveryExecutionIntentHash", "execution_commit_gate_sha256=$recoveryExecutionCommitGateHash", "execution_result_denial_sha256=$recoveryExecutionResultDenialHash") -AbsentReason "recovery_lifeline_command_execution_audit_denial_absent" -ValidReason "recovery_lifeline_command_execution_audit_denial_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_observation_denial_not_implemented" -NextPresentNeedle '"execution_audit_denial_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionAuditDenialHash = [string]$executionAuditDenial.Hash
    $recoveryExecutionAuditDenialEventId = [string]$executionAuditDenial.EventId

    $recoveryExecutionObservationDenialId = "boundary.recovery_lifeline_command_execution_observation_denial.current_boot"
    $recoveryExecutionObservationDenialProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_observation_denial_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_audit_denial_hash=$recoveryExecutionAuditDenialHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "observes_lifeline_command_result=false",
        "exports_provider_context=false",
        "writes_recovery_memory=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionObservationDenial = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_observation_denial" -Method "recovery.lifeline_command_execution_observation_denial_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_observation_denial_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_observation_denial_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_observation_denial_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_observation_denial.v0" -Canonicalization "raios.recovery_lifeline_command_execution_observation_denial.canonical.v0" -Resource "recovery_lifeline_command_execution_observation_denial" -StageHashName "execution_observation_denial_hash" -StageIdField "execution_observation_denial_id" -StageId $recoveryExecutionObservationDenialId -ProjectionField "execution_observation_projection_sha256" -ProjectionHash $recoveryExecutionObservationDenialProjectionHash -RetainedEventField "retained_execution_audit_denial_event_id" -RetainedEventId $recoveryExecutionAuditDenialEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash, $recoveryExecutionIntentHash, $recoveryExecutionCommitGateHash, $recoveryExecutionResultDenialHash, $recoveryExecutionAuditDenialHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash", "execution_intent_sha256=$recoveryExecutionIntentHash", "execution_commit_gate_sha256=$recoveryExecutionCommitGateHash", "execution_result_denial_sha256=$recoveryExecutionResultDenialHash", "execution_audit_denial_sha256=$recoveryExecutionAuditDenialHash") -AbsentReason "recovery_lifeline_command_execution_observation_denial_absent" -ValidReason "recovery_lifeline_command_execution_observation_denial_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_execution_completion_denial_not_implemented" -NextPresentNeedle '"execution_observation_denial_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionObservationDenialHash = [string]$executionObservationDenial.Hash
    $recoveryExecutionObservationDenialEventId = [string]$executionObservationDenial.EventId

    $recoveryExecutionCompletionDenialId = "boundary.recovery_lifeline_command_execution_completion_denial.current_boot"
    $recoveryExecutionCompletionDenialProjectionHash = Get-TextSha256 -Text (@(
        "schema=raios.recovery_lifeline_command_execution_completion_denial_projection.v0",
        "command_id=recovery.lifeline.status",
        "execution_observation_denial_hash=$recoveryExecutionObservationDenialHash",
        "command_execution_enabled=false",
        "dispatches_lifeline_command=false",
        "observes_lifeline_command_result=false",
        "exports_provider_context=false",
        "writes_recovery_memory=false",
        "writes_completion_record=false",
        "service_inventory_change=none"
    ) -join "`n")
    $executionCompletionDenial = Invoke-RecoveryExecutionStage -StageName "recovery_lifeline_command_execution_completion_denial" -Method "recovery.lifeline_command_execution_completion_denial_diagnostic" -SelftestMethod "recovery.lifeline_command_execution_completion_denial_diagnostic_selftest" -DiagnosticSchema "raios.recovery_lifeline_command_execution_completion_denial_diagnostic.v0" -SelftestSchema "raios.recovery_lifeline_command_execution_completion_denial_selftest.v0" -ReferenceSchema "raios.recovery_lifeline_command_execution_completion_denial.v0" -Canonicalization "raios.recovery_lifeline_command_execution_completion_denial.canonical.v0" -Resource "recovery_lifeline_command_execution_completion_denial" -StageHashName "execution_completion_denial_hash" -StageIdField "execution_completion_denial_id" -StageId $recoveryExecutionCompletionDenialId -ProjectionField "execution_completion_projection_sha256" -ProjectionHash $recoveryExecutionCompletionDenialProjectionHash -RetainedEventField "retained_execution_observation_denial_event_id" -RetainedEventId $recoveryExecutionObservationDenialEventId -PriorStageHashArgs @($recoveryExecutionEnablementHash, $recoveryExecutionPreflightHash, $recoveryExecutionIntentHash, $recoveryExecutionCommitGateHash, $recoveryExecutionResultDenialHash, $recoveryExecutionAuditDenialHash, $recoveryExecutionObservationDenialHash) -PriorStageHashLines @("execution_enablement_sha256=$recoveryExecutionEnablementHash", "execution_preflight_sha256=$recoveryExecutionPreflightHash", "execution_intent_sha256=$recoveryExecutionIntentHash", "execution_commit_gate_sha256=$recoveryExecutionCommitGateHash", "execution_result_denial_sha256=$recoveryExecutionResultDenialHash", "execution_audit_denial_sha256=$recoveryExecutionAuditDenialHash", "execution_observation_denial_sha256=$recoveryExecutionObservationDenialHash") -AbsentReason "recovery_lifeline_command_execution_completion_denial_absent" -ValidReason "recovery_lifeline_command_execution_completion_denial_valid_but_execution_disabled" -NextDispatchReason "recovery_lifeline_command_dispatch_execution_disabled" -NextPresentNeedle '"execution_completion_denial_present": true' -PreviousEventNeedleName "retained_previous_stage_event_id"
    $recoveryExecutionCompletionDenialHash = [string]$executionCompletionDenial.Hash
    $recoveryExecutionCompletionDenialEventId = [string]$executionCompletionDenial.EventId
    Assert-LogContainsFields -NamePrefix "protocol:dispatch_completion_denial_reference_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "event_id"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_execution_completion_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "retained_hash_reference_command_still_denied"' },
        @{ Suffix = "stage_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" },
        @{ Suffix = "result_hash"; Needle = "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" },
        @{ Suffix = "observation_hash"; Needle = "`"execution_observation_denial_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" }
    )

    $recoveryStatusExecutionResultId = "result.recovery_lifeline_status.current_boot"
    $recoveryStatusExecutionResultHash = Get-TextSha256 -Text (@(
        "canonicalization=raios.recovery_lifeline_status_execution_result.canonical.v0",
        "schema=raios.recovery_lifeline_status_execution_result.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_status_execution_result",
        "scope=current_boot",
        "retained_status_read_handler_event_id=$recoveryStatusReadHandlerEventId",
        "retained_execution_completion_denial_event_id=$recoveryExecutionCompletionDenialEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "status_read_projection_sha256=$recoveryStatusReadProjectionHash",
        "command_dispatch_behavior_sha256=$recoveryCommandDispatchBehaviorHash",
        "executor_capability_table_sha256=$recoveryExecutorCapabilityTableHash",
        "side_effect_gate_sha256=$recoverySideEffectGateHash",
        "source_rollback_apply_denial_sha256=$recoveryRollbackApplySourceDenialHash",
        "source_durable_policy_write_authority_decision_sha256=$recoveryRollbackApplySourceDurablePolicyDecisionHash",
        "source_recovery_rollback_inspect_source_reference_sha256=$recoveryRollbackApplySourceInspectReferenceHash",
        "execution_enablement_sha256=$recoveryExecutionEnablementHash",
        "execution_preflight_sha256=$recoveryExecutionPreflightHash",
        "execution_intent_sha256=$recoveryExecutionIntentHash",
        "execution_commit_gate_sha256=$recoveryExecutionCommitGateHash",
        "execution_result_denial_sha256=$recoveryExecutionResultDenialHash",
        "execution_audit_denial_sha256=$recoveryExecutionAuditDenialHash",
        "execution_observation_denial_sha256=$recoveryExecutionObservationDenialHash",
        "execution_completion_denial_sha256=$recoveryExecutionCompletionDenialHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "status_execution_result_id=$recoveryStatusExecutionResultId",
        "status_execution_readiness=available_read_only_non_authorizing",
        "readiness_reason=recovery_lifeline_status_read_ready_command_execution_disabled",
        "would_execute_lifeline_status_read=true",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "command_execution_enabled=false",
        "executes_lifeline_status=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "loads_recovery_artifact=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "allocates_service_slot=false",
        "creates_service_inventory_records=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n")
    Send-AgentCommand -Command "agent recovery.lifeline_status_execution_result_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_execution_result_diagnostic"
    $statusExecutionResultResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_status_execution_result_diagnostic"
    $recoveryStatusExecutionResultEventId = [string]$statusExecutionResultResponse.body.result.retained_status_execution_result_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:status_execution_result_retained_event_id_captured" -Value $recoveryStatusExecutionResultEventId
    Assert-LogContainsFields -NamePrefix "protocol:status_execution_result_after_completion_denial_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_execution_result_diagnostic.v0"' },
        @{ Suffix = "retained"; Needle = '"status": "retained_read_only_result_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_execution_result_retained_command_execution_disabled"' },
        @{ Suffix = "mutation"; Needle = '"mutates_global_event_log": true' },
        @{ Suffix = "records"; Needle = '"creates_retained_recovery_lifeline_status_execution_result_records": true' },
        @{ Suffix = "readiness_available"; Needle = '"status": "available_read_only_non_authorizing"' },
        @{ Suffix = "readiness_reason"; Needle = '"reason": "recovery_lifeline_status_read_ready_command_execution_disabled"' },
        @{ Suffix = "handler_event"; Needle = "`"retained_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "recorded_event"; Needle = "`"recorded_event_id`": `"$recoveryStatusExecutionResultEventId`"" },
        @{ Suffix = "result_hash"; Needle = "`"status_execution_result_hash`": `"sha256:$recoveryStatusExecutionResultHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"status_read_projection_hash`": `"sha256:$recoveryStatusReadProjectionHash`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "behavior_hash"; Needle = "`"command_dispatch_behavior_hash`": `"sha256:$recoveryCommandDispatchBehaviorHash`"" },
        @{ Suffix = "executor_hash"; Needle = "`"executor_capability_table_hash`": `"sha256:$recoveryExecutorCapabilityTableHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "would_execute_true"; Needle = '"would_execute_lifeline_status_read": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_memory_write"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_durable_write"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "no_rollback_write"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command "agent recovery.lifeline_status_result_read" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_result_read"
    Assert-LogContainsFields -NamePrefix "protocol:status_result_read_after_result_record_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_result_read.v0"' },
        @{ Suffix = "available"; Needle = '"status": "available_read_only_current_boot"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_result_read_available"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "source_accepted"; Needle = '"status": "accepted_read_only_source"' },
        @{ Suffix = "result_event"; Needle = "`"event_id`": `"$recoveryStatusExecutionResultEventId`"" },
        @{ Suffix = "result_hash"; Needle = "`"status_execution_result_hash`": `"sha256:$recoveryStatusExecutionResultHash`"" },
        @{ Suffix = "handler_event"; Needle = "`"retained_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "projection_schema"; Needle = '"schema": "raios.recovery_lifeline_status_projection.v0"' },
        @{ Suffix = "bounded"; Needle = '"bounded_current_boot_projection": true' },
        @{ Suffix = "source_verified"; Needle = '"source_retained_result_verified": true' },
        @{ Suffix = "core_alive"; Needle = '"recovery_core_alive": true' },
        @{ Suffix = "provider_route_false"; Needle = '"provider_recovery_route_enabled": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "module_disable_false"; Needle = '"module_disable_enabled": false' },
        @{ Suffix = "restart_false"; Needle = '"restart_last_good_enabled": false' },
        @{ Suffix = "load_false"; Needle = '"recovery_artifact_load_enabled": false' },
        @{ Suffix = "memory_write_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "durable_write_false"; Needle = '"durable_audit_writes_enabled": false' },
        @{ Suffix = "rollback_write_false"; Needle = '"rollback_store_writes_enabled": false' },
        @{ Suffix = "inventory_mutation_false"; Needle = '"service_inventory_mutation_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command "agent recovery.lifeline.status" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline.status"
    Assert-LogContainsFields -NamePrefix "protocol:lifeline_status_command_after_result_record_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_result_read.v0"' },
        @{ Suffix = "available"; Needle = '"status": "available_read_only_current_boot"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_result_read_available"' },
        @{ Suffix = "command_facing"; Needle = '"command_envelope_facing": true' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "source_accepted"; Needle = '"status": "accepted_read_only_source"' },
        @{ Suffix = "result_event"; Needle = "`"event_id`": `"$recoveryStatusExecutionResultEventId`"" },
        @{ Suffix = "result_hash"; Needle = "`"status_execution_result_hash`": `"sha256:$recoveryStatusExecutionResultHash`"" },
        @{ Suffix = "handler_event"; Needle = "`"retained_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "projection_schema"; Needle = '"schema": "raios.recovery_lifeline_status_projection.v0"' },
        @{ Suffix = "bounded"; Needle = '"bounded_current_boot_projection": true' },
        @{ Suffix = "source_verified"; Needle = '"source_retained_result_verified": true' },
        @{ Suffix = "core_alive"; Needle = '"recovery_core_alive": true' },
        @{ Suffix = "provider_route_false"; Needle = '"provider_recovery_route_enabled": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "module_disable_false"; Needle = '"module_disable_enabled": false' },
        @{ Suffix = "restart_false"; Needle = '"restart_last_good_enabled": false' },
        @{ Suffix = "load_false"; Needle = '"recovery_artifact_load_enabled": false' },
        @{ Suffix = "memory_write_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "durable_write_false"; Needle = '"durable_audit_writes_enabled": false' },
        @{ Suffix = "rollback_write_false"; Needle = '"rollback_store_writes_enabled": false' },
        @{ Suffix = "inventory_mutation_false"; Needle = '"service_inventory_mutation_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Send-AgentCommand -Command $recoveryStatusEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline.status"
    $recoveryStatusEnvelopeAfter = Get-LastAgentResponseJson -Method "agent.command_envelope"
    Assert-CurrentBootEventId -Name "protocol:lifeline_status_envelope_after_event_id" -Value $recoveryStatusEnvelopeAfter.body.result.event_id
    if (-not $recoveryStatusEnvelopeAfter.body.result.accepted -or $recoveryStatusEnvelopeAfter.body.result.reason -ne "accepted" -or -not $recoveryStatusEnvelopeAfter.body.result.dispatches_existing_agent_method) {
        throw "Expected recovery.lifeline.status envelope to dispatch the existing read method after retained result"
    }
    $recoveryStatusEnvelopeReadAfter = Get-LastAgentResponseJson -Method "recovery.lifeline.status"
    if ($recoveryStatusEnvelopeReadAfter.body.result.status -ne "available_read_only_current_boot" -or $recoveryStatusEnvelopeReadAfter.body.result.command_envelope_facing -ne $true -or $recoveryStatusEnvelopeReadAfter.body.result.recovery_status.source_retained_result_verified -ne $true -or $recoveryStatusEnvelopeReadAfter.body.result.dispatches_lifeline_command -or $recoveryStatusEnvelopeReadAfter.body.result.command_execution_enabled -or $recoveryStatusEnvelopeReadAfter.body.result.executes_lifeline_status) {
        throw "Expected enveloped recovery.lifeline.status after retained result to stay read-only and available"
    }
    Send-AgentCommand -Command "agent memory.context diagnostic" -ExpectedMarker "RAIOS_AGENT_END memory.context"
    $recoveryStatusContext = Get-LastAgentResponseJson -Method "memory.context"
    $recoveryStatusFact = $recoveryStatusContext.body.result.current.recovery_lifeline_status
    if ($recoveryStatusFact.schema -ne "raios.agent_context.recovery_lifeline_status_fact.v0" -or $recoveryStatusFact.id -ne "recovery.lifeline.status.current_boot" -or $recoveryStatusFact.status -ne "available_read_only_current_boot" -or $recoveryStatusFact.reason -ne "recovery_lifeline_status_result_read_available" -or $recoveryStatusFact.source_retained_result_status -ne "accepted_read_only_source" -or $recoveryStatusFact.source_retained_result_verified -ne $true -or $recoveryStatusFact.retained_status_execution_result_event_id -ne $recoveryStatusExecutionResultEventId -or $recoveryStatusFact.status_execution_result_hash -ne "sha256:$recoveryStatusExecutionResultHash") {
        throw "Expected memory.context to expose the verified recovery lifeline status fact"
    }
    if ($recoveryStatusFact.projection.status -ne "available_read_only_current_boot" -or $recoveryStatusFact.projection.bounded_current_boot_projection -ne $true -or $recoveryStatusFact.projection.source_retained_result_verified -ne $true -or $recoveryStatusFact.projection.dispatches_lifeline_command -or $recoveryStatusFact.projection.command_execution_enabled -or $recoveryStatusFact.projection.executes_lifeline_status -or $recoveryStatusFact.projection.recovery_memory_writes_enabled -or $recoveryStatusFact.projection.durable_audit_writes_enabled -or $recoveryStatusFact.projection.rollback_store_writes_enabled -or $recoveryStatusFact.projection.service_inventory_mutation_enabled -or $recoveryStatusFact.projection.load_attempted) {
        throw "Expected recovery lifeline status context projection to stay bounded and non-executing"
    }
    if ($recoveryStatusFact.side_effects.writes_memory -or $recoveryStatusFact.side_effects.provider_export -or $recoveryStatusFact.side_effects.fallback_executor -or $recoveryStatusFact.side_effects.recovery_command_dispatch -or $recoveryStatusFact.side_effects.mutates_service_inventory) {
        throw "Expected recovery lifeline status context fact to deny writes, export, fallback execution, dispatch, and inventory mutation"
    }
    Send-AgentCommand -Command "agent memory.query" -ExpectedMarker "RAIOS_AGENT_END memory.query"
    $recoveryStatusQuery = Get-LastAgentResponseJson -Method "memory.query"
    $recoveryStatusQueryRecords = @($recoveryStatusQuery.body.result.records | Where-Object { $_.id -eq "recovery.lifeline.status.current_boot" })
    if ($recoveryStatusQueryRecords.Count -ne 1 -or $recoveryStatusQueryRecords[0].kind -ne "recovery_lifeline_status" -or $recoveryStatusQueryRecords[0].classification -ne "local_only") {
        throw "Expected memory.query to expose the recovery lifeline status locator"
    }
    Send-AgentCommand -Command "agent memory.trace recovery.lifeline.status.current_boot" -ExpectedMarker "RAIOS_AGENT_END memory.trace"
    $recoveryStatusTrace = Get-LastAgentResponseJson -Method "memory.trace"
    $recoveryStatusTraceRecords = @($recoveryStatusTrace.body.result.records | Where-Object { $_.id -eq "recovery.lifeline.status.current_boot" })
    if ($recoveryStatusTraceRecords.Count -ne 1 -or $recoveryStatusTraceRecords[0].source_method -ne "recovery.lifeline.status" -or $recoveryStatusTraceRecords[0].source -ne "seed-kernel/src/agent_protocol_recovery_execution.rs") {
        throw "Expected memory.trace to point recovery lifeline status back to the read-only recovery status source"
    }

    Send-AgentCommand -Command "agent recovery.load_binding" -ExpectedMarker "RAIOS_AGENT_END recovery.load_binding"
    $recoveryBindingResponse = Get-LastAgentResponseJson -Method "recovery.load_binding"
    Assert-LogContains -Name "protocol:recovery_binding_schema" -Needle '"schema": "raios.recovery_artifact_load_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_status" -Needle '"status": "denied_missing_recovery_binding"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_records" -Needle '"creates_retained_recovery_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_read_capability" -Needle '"read_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_recovery_capability" -Needle '"recovery_only_capability_used": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_normal_capability_false" -Needle '"normal_module_capability_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_separate_from_module" -Needle '"separate_from": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_id_required" -Needle '"recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_id_required" -Needle '"recovery_artifact_trust_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_id_required" -Needle '"recovery_vm_test_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_approval_id_required" -Needle '"recovery_local_approval_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_id_required" -Needle '"recovery_loader_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_id_required" -Needle '"recovery_rollback_evidence_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_denial_id_required" -Needle '"recovery_lifeline_command_execution_completion_denial_event_id"' -TimeoutSeconds 1
    $recoveryBindingIdentityEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_artifact_identity_event_id.event_id
    $recoveryBindingIdentityEventIdMatches = $recoveryBindingIdentityEventId -eq $recoveryIdentityEventId
    Add-Predicate -Name "protocol:recovery_binding_identity_event_id_matches_retained" -Expected $recoveryIdentityEventId -Passed $recoveryBindingIdentityEventIdMatches -Actual $recoveryBindingIdentityEventId
    if (-not $recoveryBindingIdentityEventIdMatches) {
        throw "Expected recovery binding identity event id $recoveryIdentityEventId, got $recoveryBindingIdentityEventId"
    }
    $recoveryBindingTrustEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_artifact_trust_event_id.event_id
    $recoveryBindingTrustEventIdMatches = $recoveryBindingTrustEventId -eq $recoveryTrustEventId
    Add-Predicate -Name "protocol:recovery_binding_trust_event_id_matches_retained" -Expected $recoveryTrustEventId -Passed $recoveryBindingTrustEventIdMatches -Actual $recoveryBindingTrustEventId
    if (-not $recoveryBindingTrustEventIdMatches) {
        throw "Expected recovery binding trust event id $recoveryTrustEventId, got $recoveryBindingTrustEventId"
    }
    $recoveryBindingVmTestEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_vm_test_event_id.event_id
    $recoveryBindingVmTestEventIdMatches = $recoveryBindingVmTestEventId -eq $recoveryVmTestEventId
    Add-Predicate -Name "protocol:recovery_binding_vm_test_event_id_matches_retained" -Expected $recoveryVmTestEventId -Passed $recoveryBindingVmTestEventIdMatches -Actual $recoveryBindingVmTestEventId
    if (-not $recoveryBindingVmTestEventIdMatches) {
        throw "Expected recovery binding VM-test event id $recoveryVmTestEventId, got $recoveryBindingVmTestEventId"
    }
    $recoveryBindingLocalApprovalEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_local_approval_event_id.event_id
    $recoveryBindingLocalApprovalEventIdMatches = $recoveryBindingLocalApprovalEventId -eq $recoveryLocalApprovalEventId
    Add-Predicate -Name "protocol:recovery_binding_local_approval_event_id_matches_retained" -Expected $recoveryLocalApprovalEventId -Passed $recoveryBindingLocalApprovalEventIdMatches -Actual $recoveryBindingLocalApprovalEventId
    if (-not $recoveryBindingLocalApprovalEventIdMatches) {
        throw "Expected recovery binding local-approval event id $recoveryLocalApprovalEventId, got $recoveryBindingLocalApprovalEventId"
    }
    $recoveryBindingLoaderEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_loader_event_id.event_id
    $recoveryBindingLoaderEventIdMatches = $recoveryBindingLoaderEventId -eq $recoveryLoaderEventId
    Add-Predicate -Name "protocol:recovery_binding_loader_event_id_matches_retained" -Expected $recoveryLoaderEventId -Passed $recoveryBindingLoaderEventIdMatches -Actual $recoveryBindingLoaderEventId
    if (-not $recoveryBindingLoaderEventIdMatches) {
        throw "Expected recovery binding loader event id $recoveryLoaderEventId, got $recoveryBindingLoaderEventId"
    }
    $recoveryBindingRollbackEvidenceEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_rollback_evidence_event_id.event_id
    $recoveryBindingRollbackEvidenceEventIdMatches = $recoveryBindingRollbackEvidenceEventId -eq $recoveryRollbackEvidenceEventId
    Add-Predicate -Name "protocol:recovery_binding_rollback_evidence_event_id_matches_retained" -Expected $recoveryRollbackEvidenceEventId -Passed $recoveryBindingRollbackEvidenceEventIdMatches -Actual $recoveryBindingRollbackEvidenceEventId
    if (-not $recoveryBindingRollbackEvidenceEventIdMatches) {
        throw "Expected recovery binding rollback-evidence event id $recoveryRollbackEvidenceEventId, got $recoveryBindingRollbackEvidenceEventId"
    }
    $recoveryBindingCompletionDenialEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_lifeline_command_execution_completion_denial_event_id.event_id
    $recoveryBindingCompletionDenialEventIdMatches = $recoveryBindingCompletionDenialEventId -eq $recoveryExecutionCompletionDenialEventId
    Add-Predicate -Name "protocol:recovery_binding_completion_denial_event_id_matches_retained" -Expected $recoveryExecutionCompletionDenialEventId -Passed $recoveryBindingCompletionDenialEventIdMatches -Actual $recoveryBindingCompletionDenialEventId
    if (-not $recoveryBindingCompletionDenialEventIdMatches) {
        throw "Expected recovery binding completion-denial event id $recoveryExecutionCompletionDenialEventId, got $recoveryBindingCompletionDenialEventId"
    }
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_denial_retained_status" -Needle '"status": "retained_hash_reference_command_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_reason" -Needle '"reason": "retained_recovery_artifact_identity_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_reason" -Needle '"reason": "retained_recovery_artifact_trust_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_reason" -Needle '"reason": "retained_recovery_artifact_vm_test_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_reason" -Needle '"reason": "retained_recovery_artifact_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_reason" -Needle '"reason": "retained_recovery_artifact_loader_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_reason" -Needle '"reason": "retained_recovery_artifact_rollback_evidence_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_denial_retained_reason" -Needle '"reason": "retained_recovery_lifeline_command_execution_completion_denial_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_hash" -Needle "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_hash" -Needle "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_hash" -Needle "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_hash" -Needle "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_hash" -Needle "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_denial_hash" -Needle "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_side_effect_hash" -Needle "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_source_denial_hash" -Needle "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_source_policy_hash" -Needle "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_source_inspect_hash" -Needle "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_enablement_hash" -Needle "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_preflight_hash" -Needle "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_intent_hash" -Needle "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_commit_hash" -Needle "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_result_hash" -Needle "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_audit_hash" -Needle "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_completion_observation_hash" -Needle "`"execution_observation_denial_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_artifact_hash" -Needle "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_material_hash" -Needle "`"trust_hash`": `"sha256:$recoveryTrustHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_material_hash" -Needle "`"vm_test_hash`": `"sha256:$recoveryVmTestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_material_hash" -Needle "`"local_approval_hash`": `"sha256:$recoveryLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_material_hash" -Needle "`"loader_hash`": `"sha256:$recoveryLoaderHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_material_hash" -Needle "`"rollback_evidence_hash`": `"sha256:$recoveryRollbackEvidenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_boundary_lifeline_missing" -Needle '"reason": "recovery_lifeline_protocol_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_intent_rejected" -Needle '"module_append_intent_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_payload_not_authority" -Needle '"module_append_payload_hash_used_as_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_writer_rejected" -Needle '"module_writer_facts_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_slot_rejected" -Needle '"module_service_slot_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_payload_non_authority" -Needle '"non_authority_input_only": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_payload_authority_false" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_beyond_denial" -Needle '"can_move_beyond_denial": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_recovery_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_durable_records" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_rollback_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_service_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent recovery.load_binding_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.load_binding_selftest"
    Assert-LogContains -Name "protocol:recovery_binding_selftest_schema" -Needle '"schema": "raios.recovery_artifact_load_binding_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_records" -Needle '"creates_retained_recovery_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_durable" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_recovery_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_count" -Needle '"case_count": 15' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_identity" -Needle '"case": "missing_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_previous_identity" -Needle '"case": "previous_boot_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_wrong_identity_schema" -Needle '"case": "wrong_schema_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_trust" -Needle '"case": "missing_recovery_artifact_trust_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_vm_test" -Needle '"case": "missing_recovery_vm_test_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_approval" -Needle '"case": "missing_recovery_local_approval_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_loader" -Needle '"case": "missing_recovery_loader_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_rollback" -Needle '"case": "missing_recovery_rollback_evidence_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_completion_denial" -Needle '"case": "missing_recovery_lifeline_command_execution_completion_denial_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_capability" -Needle '"case": "module_load_ephemeral_capability_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_intent" -Needle '"case": "normal_module_append_intent_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_authority" -Needle '"case": "append_payload_hash_claimed_as_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_writer" -Needle '"case": "normal_module_writer_facts_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_service_slot" -Needle '"case": "normal_module_service_slot_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_available_denied" -Needle '"case": "available_recovery_binding_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_capability_reason" -Needle '"actual_reason": "recovery_load_requires_cap_recovery_load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_intent_reason" -Needle '"actual_reason": "normal_module_append_intent_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_reason" -Needle '"actual_reason": "append_payload_hash_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_writer_reason" -Needle '"actual_reason": "normal_module_writer_facts_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_service_slot_reason" -Needle '"actual_reason": "normal_module_service_slot_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_completion_denial_reason" -Needle '"actual_reason": "recovery_lifeline_command_execution_completion_denial_event_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_available_reason" -Needle '"actual_reason": "recovery_lifeline_protocol_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_can_move_false" -Needle '"can_move_beyond_denial": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_cap_not_accepted" -Needle '"normal_module_capability_accepted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_authority_false" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "recovery.load_artifact" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact"
    $recoveryLoadAfterBindingResponse = Get-LastAgentResponseJson -Method "recovery.load_artifact"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_after_binding_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_load_boundary.v0"' },
        @{ Suffix = "denied"; Needle = '"code": "capability_denied"' },
        @{ Suffix = "status"; Needle = '"status": "denied_recovery_load_binding_not_authorizing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_binding_not_authorizing"' },
        @{ Suffix = "load_binding_schema"; Needle = '"recovery_load_binding": {' },
        @{ Suffix = "load_binding_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "load_binding_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event_id"; Needle = "`"event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_inventory"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryLoadCompletionEventId = [string]$recoveryLoadAfterBindingResponse.body.recovery_load_binding.required_retained_evidence.recovery_lifeline_command_execution_completion_denial_event_id.event_id
    $recoveryLoadCompletionEventIdMatches = $recoveryLoadCompletionEventId -eq $recoveryExecutionCompletionDenialEventId
    Add-Predicate -Name "protocol:recovery_load_after_binding_completion_event_id_matches_retained" -Expected $recoveryExecutionCompletionDenialEventId -Passed $recoveryLoadCompletionEventIdMatches -Actual $recoveryLoadCompletionEventId
    if (-not $recoveryLoadCompletionEventIdMatches) {
        throw "Expected recovery.load_artifact nested load-binding completion-denial event id $recoveryExecutionCompletionDenialEventId, got $recoveryLoadCompletionEventId"
    }

    Send-AgentCommand -Command "agent recovery.lifeline_command_admission" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_admission"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_admission_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "source_event"; Needle = '"retained_recovery_artifact_load_denied_event_id": "event.current_boot.' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_artifact_by_hash_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_hash_target_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "target_schema"; Needle = '"schema": "raios.recovery_load_artifact_by_hash_target_binding_diagnostic.v0"' },
        @{ Suffix = "target_missing"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_absent"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_dispatch_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "dispatch_schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_write_authority_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "memory_schema"; Needle = '"schema": "raios.recovery_memory_write_authority_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_write_authority_records": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_write"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.durable_audit_rollback_write_authority_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_durable_audit_rollback_write_authority_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "durable_schema"; Needle = '"schema": "raios.durable_audit_rollback_write_authority_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_durable_audit_rollback_write_authority_records": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_durable_write"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "no_rollback_write"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.service_inventory_side_effect_boundary_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.service_inventory_side_effect_boundary_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_service_inventory_side_effect_boundary_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "service_schema"; Needle = '"schema": "raios.recovery_service_inventory_side_effect_boundary_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_service_inventory_side_effect_boundary_records": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_behavior_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_behavior_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_command_dispatch_behavior_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "behavior_schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_behavior_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_behavior_records": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_executor_capability_table_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_executor_capability_table_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_executor_capability_table_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "executor_schema"; Needle = '"schema": "raios.recovery_lifeline_command_executor_capability_table_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_executor_capability_table_records": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_side_effect_gate_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_side_effect_gate_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_side_effect_gate_load_denial_source_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "side_effect_schema"; Needle = '"schema": "raios.recovery_lifeline_command_side_effect_gate_diagnostic.v0"' },
        @{ Suffix = "source_schema"; Needle = '"recovery_artifact_load_denial_source": {' },
        @{ Suffix = "source_schema_name"; Needle = '"schema": "raios.recovery_artifact_load_denial_source.v0"' },
        @{ Suffix = "source_present"; Needle = '"source_evidence_present": true' },
        @{ Suffix = "source_status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "source_reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_side_effect_gate_records": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_inventory_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_execution"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_enablement_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_enablement_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_enablement_load_denial_source_" -SchemaSuffix "enablement_schema" -Schema "raios.recovery_lifeline_command_execution_enablement_diagnostic.v0"

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_preflight_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_preflight_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_preflight_load_denial_source_" -SchemaSuffix "preflight_schema" -Schema "raios.recovery_lifeline_command_execution_preflight_diagnostic.v0"

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_intent_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_intent_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_intent_load_denial_source_" -SchemaSuffix "intent_schema" -Schema "raios.recovery_lifeline_command_execution_intent_diagnostic.v0"

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_commit_gate_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_commit_gate_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_commit_gate_load_denial_source_" -SchemaSuffix "commit_gate_schema" -Schema "raios.recovery_lifeline_command_execution_commit_gate_diagnostic.v0" -StageFields @(
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_result_denial_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_result_denial_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_result_denial_load_denial_source_" -SchemaSuffix "result_denial_schema" -Schema "raios.recovery_lifeline_command_execution_result_denial_diagnostic.v0" -StageFields @(
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_audit_denial_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_audit_denial_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_audit_denial_load_denial_source_" -SchemaSuffix "audit_denial_schema" -Schema "raios.recovery_lifeline_command_execution_audit_denial_diagnostic.v0" -StageFields @(
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" },
        @{ Suffix = "result_hash"; Needle = "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_observation_denial_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_observation_denial_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_observation_denial_load_denial_source_" -SchemaSuffix "observation_denial_schema" -Schema "raios.recovery_lifeline_command_execution_observation_denial_diagnostic.v0" -StageFields @(
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" },
        @{ Suffix = "result_hash"; Needle = "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_execution_completion_denial_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_execution_completion_denial_diagnostic"
    Assert-RecoveryExecutionStageLoadDenialSource -NamePrefix "protocol:recovery_execution_completion_denial_load_denial_source_" -SchemaSuffix "completion_denial_schema" -Schema "raios.recovery_lifeline_command_execution_completion_denial_diagnostic.v0" -StageFields @(
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" },
        @{ Suffix = "result_hash"; Needle = "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" },
        @{ Suffix = "observation_hash"; Needle = "`"execution_observation_denial_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" }
    )

    Send-AgentCommand -Command "agent audit.events 96" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.recovery_execution_binding"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_audit_load_binding_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "binding_schema"; Needle = '"bindings": {"schema": "raios.recovery_artifact_load_denial_evidence.v0"' },
        @{ Suffix = "binding_status"; Needle = '"status": "denied_recovery_load_binding_not_authorizing"' },
        @{ Suffix = "binding_reason"; Needle = '"reason": "recovery_load_binding_not_authorizing"' },
        @{ Suffix = "load_binding_schema"; Needle = '"recovery_load_binding": {"schema": "raios.recovery_artifact_load_binding.v0"' },
        @{ Suffix = "status"; Needle = '"status": "available_non_authorizing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_missing"' },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "side_effect_hash"; Needle = "`"side_effect_gate_hash`": `"sha256:$recoverySideEffectGateHash`"" },
        @{ Suffix = "source_denial_hash"; Needle = "`"source_rollback_apply_denial_hash`": `"sha256:$recoveryRollbackApplySourceDenialHash`"" },
        @{ Suffix = "source_policy_hash"; Needle = "`"source_durable_policy_write_authority_decision_hash`": `"sha256:$recoveryRollbackApplySourceDurablePolicyDecisionHash`"" },
        @{ Suffix = "source_inspect_hash"; Needle = "`"source_recovery_rollback_inspect_source_reference_hash`": `"sha256:$recoveryRollbackApplySourceInspectReferenceHash`"" },
        @{ Suffix = "enablement_hash"; Needle = "`"execution_enablement_hash`": `"sha256:$recoveryExecutionEnablementHash`"" },
        @{ Suffix = "preflight_hash"; Needle = "`"execution_preflight_hash`": `"sha256:$recoveryExecutionPreflightHash`"" },
        @{ Suffix = "intent_hash"; Needle = "`"execution_intent_hash`": `"sha256:$recoveryExecutionIntentHash`"" },
        @{ Suffix = "commit_hash"; Needle = "`"execution_commit_gate_hash`": `"sha256:$recoveryExecutionCommitGateHash`"" },
        @{ Suffix = "result_hash"; Needle = "`"execution_result_denial_hash`": `"sha256:$recoveryExecutionResultDenialHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"execution_audit_denial_hash`": `"sha256:$recoveryExecutionAuditDenialHash`"" },
        @{ Suffix = "observation_hash"; Needle = "`"execution_observation_denial_hash`": `"sha256:$recoveryExecutionObservationDenialHash`"" },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_inventory"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    Assert-LogContainsFields -NamePrefix "protocol:audit_status_execution_result_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "kind"; Needle = '"kind": "recovery.lifeline_status_execution_result.retained"' },
        @{ Suffix = "source"; Needle = '"source_method": "recovery.lifeline_status_execution_result_diagnostic"' },
        @{ Suffix = "outcome"; Needle = '"outcome": "retained_read_only_result_command_still_denied"' },
        @{ Suffix = "binding_schema"; Needle = '"bindings": {"schema": "raios.recovery_lifeline_status_execution_result.v0"' },
        @{ Suffix = "binding_status"; Needle = '"status": "retained_read_only_result_command_still_denied"' },
        @{ Suffix = "result_hash"; Needle = "`"status_execution_result_hash`": `"sha256:$recoveryStatusExecutionResultHash`"" },
        @{ Suffix = "handler_event"; Needle = "`"retained_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" },
        @{ Suffix = "completion_event"; Needle = "`"retained_execution_completion_denial_event_id`": `"$recoveryExecutionCompletionDenialEventId`"" },
        @{ Suffix = "completion_hash"; Needle = "`"execution_completion_denial_hash`": `"sha256:$recoveryExecutionCompletionDenialHash`"" },
        @{ Suffix = "would_execute"; Needle = '"would_execute_lifeline_status_read": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "executes_status_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
