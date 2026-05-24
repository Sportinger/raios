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
            "side_effect_gate_sha256=$recoverySideEffectGateHash"
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
            $recoverySideEffectGateHash
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
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_reason" -Needle '"reason": "retained_recovery_artifact_identity_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_reason" -Needle '"reason": "retained_recovery_artifact_trust_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_reason" -Needle '"reason": "retained_recovery_artifact_vm_test_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_reason" -Needle '"reason": "retained_recovery_artifact_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_reason" -Needle '"reason": "retained_recovery_artifact_loader_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_reason" -Needle '"reason": "retained_recovery_artifact_rollback_evidence_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_hash" -Needle "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_hash" -Needle "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_hash" -Needle "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_hash" -Needle "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_hash" -Needle "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:recovery_binding_selftest_count" -Needle '"case_count": 14' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_identity" -Needle '"case": "missing_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_previous_identity" -Needle '"case": "previous_boot_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_wrong_identity_schema" -Needle '"case": "wrong_schema_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_trust" -Needle '"case": "missing_recovery_artifact_trust_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_vm_test" -Needle '"case": "missing_recovery_vm_test_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_approval" -Needle '"case": "missing_recovery_local_approval_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_loader" -Needle '"case": "missing_recovery_loader_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_rollback" -Needle '"case": "missing_recovery_rollback_evidence_event_id"' -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:recovery_binding_selftest_available_reason" -Needle '"actual_reason": "recovery_lifeline_protocol_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_can_move_false" -Needle '"can_move_beyond_denial": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_cap_not_accepted" -Needle '"normal_module_capability_accepted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_authority_false" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 256" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
