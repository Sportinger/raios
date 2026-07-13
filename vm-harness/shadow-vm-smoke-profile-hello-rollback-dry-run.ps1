    function Get-LastMarkerJsonAfterOffset {
        param(
            [string]$Prefix,
            [int]$Offset,
            [string]$Name
        )

        $content = Get-SerialLogContent -Path $SerialLog
        if ($null -eq $content) {
            throw "No serial log content found in $SerialLog"
        }
        $slice = if ($Offset -lt $content.Length) { $content.Substring($Offset) } else { "" }
        $markerIndex = $slice.LastIndexOf($Prefix, [System.StringComparison]::Ordinal)
        $passed = $markerIndex -ge 0
        Add-Predicate -Name $Name -Expected "serial_contains_after_offset:$Prefix" -Passed $passed -Actual $(if ($passed) { "found_after_offset:$Offset" } else { Get-SerialLogTail -Path $SerialLog })
        if (-not $passed) {
            throw "Expected marker '$Prefix' after serial offset $Offset"
        }

        $jsonStart = $markerIndex + $Prefix.Length
        $lineEnd = $slice.IndexOf("`n", $jsonStart, [System.StringComparison]::Ordinal)
        if ($lineEnd -lt 0) {
            $lineEnd = $slice.Length
        }
        $json = $slice.Substring($jsonStart, $lineEnd - $jsonStart).Trim()
        return $json | ConvertFrom-Json
    }

    Send-AgentCommand -Command "recovery.rollback_inspect_source_reference_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect_source_reference_selftest"
    $helloInspectSourceSelftest = Get-LastAgentResponseJson -Method "recovery.rollback_inspect_source_reference_selftest"
    if ($helloInspectSourceSelftest.body.result.schema -ne "raios.recovery_rollback_inspect_source_reference_selftest.v0" -or $helloInspectSourceSelftest.body.result.case_count -ne 7 -or $helloInspectSourceSelftest.body.result.passed_count -ne 7 -or -not $helloInspectSourceSelftest.body.result.all_passed) {
        throw "Expected recovery rollback inspect source-reference selftest to pass all retained RAM-audit verifier cases"
    }
    if (-not $helloInspectSourceSelftest.body.result.read_only -or $helloInspectSourceSelftest.body.result.mutates_global_event_log -or $helloInspectSourceSelftest.body.result.creates_source_reference_events -or $helloInspectSourceSelftest.body.result.denied_surfaces.rollback_apply -ne "denied" -or $helloInspectSourceSelftest.body.result.denied_surfaces.persistence -ne "denied") {
        throw "Recovery rollback inspect source-reference selftest must stay read-only, non-mutating, and non-authorizing"
    }

    Send-AgentCommand -Command "module.load_ephemeral svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
    $helloDryRunLoad = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $helloDryRunLoadState = @($helloDryRunLoad.evidence | Where-Object { $_.id -eq "state_transition" })[0]
    if ($helloDryRunLoad.schema -ne "raios.evidence_response.v1" -or $helloDryRunLoad.family -ne "hello.lifecycle" -or $helloDryRunLoad.decision.reason -ne "load_performed" -or -not $helloDryRunLoadState.facts.after.loaded -or -not $helloDryRunLoadState.facts.after.running) {
        throw "Expected Hello rollback dry-run profile to load the built-in RAM-only service"
    }

    $helloHealthEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.health requested_capability=cap.service.health.read classification=local_only"
    Send-AgentCommand -Command $helloHealthEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END service.health"
    $helloHealthEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $helloHealthEnvelope.body.result.accepted -or $helloHealthEnvelope.body.result.target_method -ne "service.health" -or -not ($helloHealthEnvelope.body.result.allowed_target_methods -contains "service.health") -or $helloHealthEnvelope.body.result.allowed_requested_capability -ne "cap.service.health.read" -or -not $helloHealthEnvelope.body.result.dispatches_existing_agent_method -or $helloHealthEnvelope.body.result.writes_durable_audit_log -or $helloHealthEnvelope.body.result.installs_rollback_plan -or $helloHealthEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch service.health without durable rollback authority"
    }
    $helloHealthEnvelopeResponse = Get-LastAgentResponseJson -Method "service.health"
    $helloHealthEnvelopeState = @($helloHealthEnvelopeResponse.evidence | Where-Object { $_.id -eq "state_transition" })[0]
    if ($helloHealthEnvelopeResponse.schema -ne "raios.evidence_response.v1" -or $helloHealthEnvelopeResponse.family -ne "hello.health" -or $helloHealthEnvelopeResponse.decision.reason -ne "health_performed" -or $helloHealthEnvelopeState.facts.after.version -ne "v1" -or -not $helloHealthEnvelopeState.facts.after.loaded -or -not $helloHealthEnvelopeState.facts.after.running) {
        throw "Expected enveloped service.health to return the loaded v1 Hello service health"
    }

    $helloHealthMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.health requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $helloHealthMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($helloHealthMismatch.body.result.accepted -or $helloHealthMismatch.body.result.reason -ne "requested_capability_denied" -or $helloHealthMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected service.health envelope with wrong capability to be denied before dispatch"
    }
    $helloHealthMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($helloHealthMismatchOffset)
    $helloHealthMismatchNoDispatch = -not $helloHealthMismatchAfter.Contains("RAIOS_AGENT_END service.health")
    Add-Predicate -Name "protocol:hello_service_health_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END service.health" -Passed $helloHealthMismatchNoDispatch -Actual $(if ($helloHealthMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $helloHealthMismatchNoDispatch) {
        throw "Expected service.health capability mismatch to avoid health dispatch"
    }

    Send-AgentCommand -Command "service.hot_swap svc.demo.hello.v2" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
    $helloDryRunHotSwap = Get-LastAgentResponseJson -Method "service.hot_swap"
    $helloDryRunHotSwapState = @($helloDryRunHotSwap.evidence | Where-Object { $_.id -eq "state_transition" })[0]
    $helloDryRunHotSwapMigration = @($helloDryRunHotSwap.evidence | Where-Object { $_.id -eq "state_migration" })[0]
    if ($helloDryRunHotSwap.family -ne "hello.lifecycle" -or $helloDryRunHotSwapState.facts.after.version -ne "v2" -or $helloDryRunHotSwapMigration.facts.to_version -ne "v2" -or -not $helloDryRunHotSwapMigration.facts.migration_hash.StartsWith("sha256:")) {
        throw "Expected Hello rollback dry-run profile to expose v2 migration evidence"
    }

    $helloRollbackPreviewEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.rollback_preview requested_capability=cap.service.rollback_preview.read classification=local_only"
    Send-AgentCommand -Command $helloRollbackPreviewEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview"
    $helloRollbackPreviewEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $helloRollbackPreviewEnvelope.body.result.accepted -or $helloRollbackPreviewEnvelope.body.result.target_method -ne "service.rollback_preview" -or $helloRollbackPreviewEnvelope.body.result.allowed_requested_capability -ne "cap.service.rollback_preview.read" -or -not $helloRollbackPreviewEnvelope.body.result.dispatches_existing_agent_method -or $helloRollbackPreviewEnvelope.body.result.writes_durable_audit_log -or $helloRollbackPreviewEnvelope.body.result.installs_rollback_plan -or $helloRollbackPreviewEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch service.rollback_preview without durable rollback authority"
    }
    $helloRollbackPreviewEnvelopeResponse = Get-LastAgentResponseJson -Method "service.rollback_preview"
    if ($helloRollbackPreviewEnvelopeResponse.body.result.schema -ne "raios.ram_only_hello_service_rollback_preview.v0" -or -not $helloRollbackPreviewEnvelopeResponse.body.result.preview_available -or -not $helloRollbackPreviewEnvelopeResponse.body.result.read_only -or $helloRollbackPreviewEnvelopeResponse.body.result.denied_surfaces.applies_rollback -or $helloRollbackPreviewEnvelopeResponse.body.result.denied_surfaces.installs_rollback_plan) {
        throw "Expected enveloped service.rollback_preview to stay read-only and non-applying"
    }

    $helloRollbackPreviewMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.rollback_preview requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $helloRollbackPreviewMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($helloRollbackPreviewMismatch.body.result.accepted -or $helloRollbackPreviewMismatch.body.result.reason -ne "requested_capability_denied" -or $helloRollbackPreviewMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected service.rollback_preview envelope with wrong capability to be denied before dispatch"
    }
    $helloRollbackPreviewMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($helloRollbackPreviewMismatchOffset)
    $helloRollbackPreviewMismatchNoDispatch = -not $helloRollbackPreviewMismatchAfter.Contains("RAIOS_AGENT_END service.rollback_preview")
    Add-Predicate -Name "protocol:hello_rollback_preview_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END service.rollback_preview" -Passed $helloRollbackPreviewMismatchNoDispatch -Actual $(if ($helloRollbackPreviewMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $helloRollbackPreviewMismatchNoDispatch) {
        throw "Expected service.rollback_preview capability mismatch to avoid preview dispatch"
    }

    Send-AgentCommand -Command "service.rollback_preview svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview"
    $helloDryRunPreview = Get-LastAgentResponseJson -Method "service.rollback_preview"
    if ($helloDryRunPreview.body.result.schema -ne "raios.ram_only_hello_service_rollback_preview.v0" -or -not $helloDryRunPreview.body.result.preview_available -or -not $helloDryRunPreview.body.result.preview_hash.StartsWith("sha256:")) {
        throw "Expected Hello rollback dry-run profile to expose a rollback preview over hot-swap probation"
    }

    Send-AgentCommand -Command "recovery.rollback_inspect svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloPreApplyRecoveryInspect = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    if ($helloPreApplyRecoveryInspect.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloPreApplyRecoveryInspect.body.result.status -ne "materialized_target_region_sector_missing" -or -not $helloPreApplyRecoveryInspect.body.result.read_only -or $helloPreApplyRecoveryInspect.body.result.materialized_sector_evidence_available -or $helloPreApplyRecoveryInspect.body.result.inspection_available) {
        throw "Expected recovery rollback inspect before apply to stay read-only and report missing materialized sector"
    }
    if ($helloPreApplyRecoveryInspect.body.result.target_region_write_readback.write_attempted -or $helloPreApplyRecoveryInspect.body.result.denied_surfaces.writes_durable_audit_log -or $helloPreApplyRecoveryInspect.body.result.denied_surfaces.writes_rollback_store -or $helloPreApplyRecoveryInspect.body.result.denied_surfaces.appends_rollback_transaction -or $helloPreApplyRecoveryInspect.body.result.denied_surfaces.applies_rollback) {
        throw "Pre-apply recovery rollback inspect must not write, append, or apply rollback"
    }

    Send-AgentCommand -Command "service.rollback_apply svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply"
    $helloPreMaterializeApply = Get-LastAgentResponseJson -Method "service.rollback_apply"
    if ($helloPreMaterializeApply.t -ne "error" -or $helloPreMaterializeApply.body.code -ne "capability_denied" -or $helloPreMaterializeApply.body.reason -ne "rollback_apply_authority_missing") {
        throw "Expected pre-materialize Hello rollback apply to stay denied by missing rollback authority"
    }
    $helloPreMaterializePreflight = $helloPreMaterializeApply.body.rollback_transaction_writer_storage_authority_gate.writer_storage_foundation.transaction_writer_readiness.scratch_only_writer_dry_run.append_record_dry_run.sector_plan_dry_run.scratch_sector_write_readback_dry_run.durable_append_authority_preflight
    $helloPreMaterializeTargetWriteReadback = $helloPreMaterializePreflight.target_region_write_readback_dry_run
    $helloPreMaterializeEvidence = $helloPreMaterializePreflight.transaction_append_dry_run
    $helloPreMaterializeSectorInspection = $helloPreMaterializePreflight.target_region_sector_inspection
    $helloPreMaterializeInspectSource = $helloPreMaterializePreflight.retained_recovery_rollback_inspect_source
    if (-not $helloPreMaterializeTargetWriteReadback -or $helloPreMaterializeTargetWriteReadback.status -ne "missing" -or $helloPreMaterializeTargetWriteReadback.reason -ne "recovery_rollback_materialize_dry_run_missing" -or $helloPreMaterializeTargetWriteReadback.write_attempted -or $helloPreMaterializeTargetWriteReadback.write_completed -or $helloPreMaterializeTargetWriteReadback.readback_completed -or $helloPreMaterializeTargetWriteReadback.readback_matches_planned_image -or $helloPreMaterializeTargetWriteReadback.test_infrastructure_media_write_authority_available) {
        throw "Expected pre-materialize rollback apply to consume missing retained materializer evidence without target-region write/readback"
    }
    if (-not $helloPreMaterializeEvidence -or $helloPreMaterializeEvidence.target_region_write_readback_verified -or $helloPreMaterializeEvidence.test_infrastructure_media_write_authority_available -or $helloPreMaterializeEvidence.append_image_ready -or $helloPreMaterializeEvidence.transaction_append_available -or $helloPreMaterializeEvidence.transaction_append_attempted -or $helloPreMaterializeEvidence.appends_rollback_transaction) {
        throw "Expected pre-materialize transaction-append dry-run evidence to remain blocked by missing materializer evidence"
    }
    if (-not $helloPreMaterializeSectorInspection -or $helloPreMaterializeSectorInspection.status -ne "missing" -or $helloPreMaterializeSectorInspection.reason -ne "recovery_rollback_inspect_missing" -or $helloPreMaterializeSectorInspection.read_attempted -or $helloPreMaterializeSectorInspection.read_completed -or $helloPreMaterializeSectorInspection.target_region_write_readback_verified -or $helloPreMaterializeSectorInspection.inspection_verified -or $helloPreMaterializeSectorInspection.appends_rollback_transaction -or $helloPreMaterializeSectorInspection.installs_rollback_state) {
        throw "Expected pre-materialize target-region inspection to remain non-authorizing and unverified"
    }
    if (-not $helloPreMaterializeInspectSource -or $helloPreMaterializeInspectSource.schema -ne "raios.recovery_rollback_inspect_source_reference.v0" -or $helloPreMaterializeInspectSource.id -ne "recovery_rollback_inspect_source.current_boot.svc.demo.hello.v0" -or $helloPreMaterializeInspectSource.status -ne "missing" -or $helloPreMaterializeInspectSource.reason -ne "recovery_rollback_inspect_source_missing" -or $helloPreMaterializeInspectSource.source_available -or $helloPreMaterializeInspectSource.source_matches_sector_inspection -or $helloPreMaterializeInspectSource.source_event_id -ne $null -or $helloPreMaterializeInspectSource.source_audit_event_id -ne $null -or $helloPreMaterializeInspectSource.source_event_retained -or $helloPreMaterializeInspectSource.source_audit_event_retained -or $helloPreMaterializeInspectSource.ram_audit_status -ne "missing" -or $helloPreMaterializeInspectSource.ram_audit_reason -ne "recovery_rollback_inspect_source_missing" -or $helloPreMaterializeInspectSource.ram_audit_validated -or $helloPreMaterializeInspectSource.reference_hash -ne $null -or $helloPreMaterializeInspectSource.source_inspection_hash -ne $null -or $helloPreMaterializeInspectSource.target_region_sector_inspection_hash -ne $helloPreMaterializeSectorInspection.inspection_hash -or $helloPreMaterializeInspectSource.authorizes_rollback_apply) {
        throw "Expected pre-materialize rollback apply to expose missing retained recovery inspect source reference"
    }

    Send-AgentCommand -Command "recovery.rollback_materialize_dry_run svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_materialize_dry_run"
    $helloMaterialize = Get-LastAgentResponseJson -Method "recovery.rollback_materialize_dry_run"
    $helloMaterializeAppendRecord = $helloMaterialize.body.result.append_record_dry_run
    $helloMaterializeSectorPlan = $helloMaterialize.body.result.sector_plan_dry_run
    $helloMaterializeTargetWriteReadback = $helloMaterialize.body.result.target_region_write_readback
    if ($helloMaterialize.body.result.schema -ne "raios.recovery_rollback_materialize_dry_run.v0" -or $helloMaterialize.body.result.status -ne "target_region_sector_materialized_current_boot" -or $helloMaterialize.body.result.read_only -or -not $helloMaterialize.body.result.test_infrastructure -or -not $helloMaterialize.body.result.materialized_sector_evidence_available) {
        throw "Expected recovery rollback materialize dry-run to materialize the target-region test sector"
    }
    if (-not $helloMaterializeAppendRecord -or -not $helloMaterializeAppendRecord.dry_run_hash.StartsWith("sha256:") -or $helloMaterializeAppendRecord.target_start_lba -ne 1 -or $helloMaterializeAppendRecord.target_lba_count -ne 1 -or $helloMaterializeAppendRecord.target_byte_count -ne 512) {
        throw "Expected recovery rollback materialize dry-run to expose append-record evidence for the LBA1 target span"
    }
    if (-not $helloMaterializeSectorPlan -or -not $helloMaterializeSectorPlan.plan_hash.StartsWith("sha256:") -or $helloMaterializeSectorPlan.sector_image_hash -ne $helloMaterializeTargetWriteReadback.planned_sector_image_hash -or $helloMaterializeSectorPlan.audit_record_offset -ne 0 -or $helloMaterializeSectorPlan.rollback_transaction_offset -ne $helloMaterializeSectorPlan.audit_record_byte_length -or $helloMaterializeSectorPlan.padding_offset -ne ($helloMaterializeSectorPlan.audit_record_byte_length + $helloMaterializeSectorPlan.rollback_transaction_byte_length)) {
        throw "Expected recovery rollback materialize dry-run to expose the canonical append sector plan"
    }
    if (-not $helloMaterializeTargetWriteReadback -or -not $helloMaterializeTargetWriteReadback.dry_run_hash.StartsWith("sha256:") -or -not $helloMaterializeTargetWriteReadback.test_infrastructure_media_write_authority_available -or -not $helloMaterializeTargetWriteReadback.write_attempted -or -not $helloMaterializeTargetWriteReadback.write_completed -or -not $helloMaterializeTargetWriteReadback.readback_completed -or -not $helloMaterializeTargetWriteReadback.readback_matches_planned_image -or $helloMaterializeTargetWriteReadback.planned_sector_image_hash -ne $helloMaterializeTargetWriteReadback.readback_sector_image_hash) {
        throw "Expected recovery rollback materialize dry-run to write/read back the target-region test sector"
    }
    if ($helloMaterialize.body.result.denied_surfaces.mutates_service_state -or $helloMaterialize.body.result.denied_surfaces.authorizes_media_write -or $helloMaterialize.body.result.denied_surfaces.authorizes_append -or $helloMaterialize.body.result.denied_surfaces.authorizes_transaction_append -or $helloMaterialize.body.result.denied_surfaces.writes_durable_audit_log -or $helloMaterialize.body.result.denied_surfaces.writes_rollback_store -or $helloMaterialize.body.result.denied_surfaces.appends_rollback_transaction -or $helloMaterialize.body.result.denied_surfaces.applies_rollback -or $helloMaterialize.body.result.denied_surfaces.installs_rollback_state) {
        throw "Recovery rollback materialize dry-run must not authorize durable writes, append, transaction append, or rollback apply"
    }

    Send-AgentCommand -Command "service.rollback_apply svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply"
    $helloPreInspectApply = Get-LastAgentResponseJson -Method "service.rollback_apply"
    if ($helloPreInspectApply.t -ne "error" -or $helloPreInspectApply.body.code -ne "capability_denied" -or $helloPreInspectApply.body.reason -ne "rollback_apply_authority_missing") {
        throw "Expected pre-inspect Hello rollback apply to stay denied by missing rollback authority"
    }
    $helloPreInspectPreflight = $helloPreInspectApply.body.rollback_transaction_writer_storage_authority_gate.writer_storage_foundation.transaction_writer_readiness.scratch_only_writer_dry_run.append_record_dry_run.sector_plan_dry_run.scratch_sector_write_readback_dry_run.durable_append_authority_preflight
    $helloPreInspectTargetWriteReadback = $helloPreInspectPreflight.target_region_write_readback_dry_run
    $helloPreInspectSectorInspection = $helloPreInspectPreflight.target_region_sector_inspection
    $helloPreInspectSource = $helloPreInspectPreflight.retained_recovery_rollback_inspect_source
    if (-not $helloPreInspectTargetWriteReadback -or $helloPreInspectTargetWriteReadback.dry_run_hash -ne $helloMaterializeTargetWriteReadback.dry_run_hash -or -not $helloPreInspectTargetWriteReadback.test_infrastructure_media_write_authority_available) {
        throw "Expected pre-inspect rollback apply to consume retained materializer write/readback evidence"
    }
    if (-not $helloPreInspectSectorInspection -or $helloPreInspectSectorInspection.status -ne "missing" -or $helloPreInspectSectorInspection.reason -ne "recovery_rollback_inspect_missing" -or $helloPreInspectSectorInspection.read_attempted -or $helloPreInspectSectorInspection.read_completed -or $helloPreInspectSectorInspection.inspection_verified -or $helloPreInspectSectorInspection.appends_rollback_transaction -or $helloPreInspectSectorInspection.installs_rollback_state) {
        throw "Expected pre-inspect rollback apply to report missing retained recovery inspection without target-sector read"
    }
    if (-not $helloPreInspectSource -or $helloPreInspectSource.status -ne "missing" -or $helloPreInspectSource.reason -ne "recovery_rollback_inspect_source_missing" -or $helloPreInspectSource.source_available -or $helloPreInspectSource.source_matches_sector_inspection -or $helloPreInspectSource.source_event_id -ne $null -or $helloPreInspectSource.source_audit_event_id -ne $null -or $helloPreInspectSource.source_event_retained -or $helloPreInspectSource.source_audit_event_retained -or $helloPreInspectSource.ram_audit_status -ne "missing" -or $helloPreInspectSource.ram_audit_reason -ne "recovery_rollback_inspect_source_missing" -or $helloPreInspectSource.ram_audit_validated -or $helloPreInspectSource.reference_hash -ne $null -or $helloPreInspectSource.source_inspection_hash -ne $null -or $helloPreInspectSource.target_region_sector_inspection_hash -ne $helloPreInspectSectorInspection.inspection_hash -or $helloPreInspectSource.authorizes_rollback_apply) {
        throw "Expected pre-inspect rollback apply to expose missing retained recovery inspect source reference"
    }

    Send-AgentCommand -Command "recovery.rollback_inspect svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloRecoveryInspectAfterMaterialize = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    $helloRecoverySectorInspectionAfterMaterialize = $helloRecoveryInspectAfterMaterialize.body.result.target_region_sector_inspection
    $helloRecoveryTargetWriteReadbackAfterMaterialize = $helloRecoveryInspectAfterMaterialize.body.result.target_region_write_readback
    $helloRecoveryInspectSourceAfterMaterialize = $helloRecoveryInspectAfterMaterialize.body.result.retained_recovery_rollback_inspect_source
    if ($helloRecoveryInspectAfterMaterialize.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRecoveryInspectAfterMaterialize.body.result.status -ne "target_region_sector_inspected_current_boot" -or -not $helloRecoveryInspectAfterMaterialize.body.result.read_only -or -not $helloRecoveryInspectAfterMaterialize.body.result.materialized_sector_evidence_available -or -not $helloRecoveryInspectAfterMaterialize.body.result.inspection_available) {
        throw "Expected recovery rollback inspect after materialize dry-run to expose read-only target-region sector evidence"
    }
    if (-not $helloRecoveryTargetWriteReadbackAfterMaterialize -or $helloRecoveryTargetWriteReadbackAfterMaterialize.dry_run_hash -ne $helloMaterializeTargetWriteReadback.dry_run_hash -or $helloRecoveryTargetWriteReadbackAfterMaterialize.planned_sector_image_hash -ne $helloMaterializeTargetWriteReadback.planned_sector_image_hash -or $helloRecoveryTargetWriteReadbackAfterMaterialize.readback_sector_image_hash -ne $helloMaterializeTargetWriteReadback.readback_sector_image_hash) {
        throw "Expected recovery rollback inspect after materialize dry-run to reuse retained target-region write/readback evidence"
    }
    if (-not $helloRecoverySectorInspectionAfterMaterialize -or -not $helloRecoverySectorInspectionAfterMaterialize.inspection_hash.StartsWith("sha256:") -or $helloRecoverySectorInspectionAfterMaterialize.source_sector_plan_hash -ne $helloMaterializeSectorPlan.plan_hash -or $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -ne $helloMaterializeTargetWriteReadback.dry_run_hash -or $helloRecoverySectorInspectionAfterMaterialize.expected_sector_image_hash -ne $helloMaterializeSectorPlan.sector_image_hash -or $helloRecoverySectorInspectionAfterMaterialize.sector_image_hash -ne $helloMaterializeTargetWriteReadback.readback_sector_image_hash -or $helloRecoverySectorInspectionAfterMaterialize.audit_record_image_hash -ne $helloMaterializeAppendRecord.audit_record_image_hash -or $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_image_hash -ne $helloMaterializeAppendRecord.rollback_transaction_image_hash) {
        throw "Expected recovery rollback inspect after materialize dry-run to bind sector, audit-record, and rollback-transaction hashes"
    }
    if ($helloRecoverySectorInspectionAfterMaterialize.target_start_lba -ne 1 -or $helloRecoverySectorInspectionAfterMaterialize.target_lba_count -ne 1 -or $helloRecoverySectorInspectionAfterMaterialize.target_byte_count -ne 512 -or $helloRecoverySectorInspectionAfterMaterialize.audit_record_offset -ne 0 -or $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_offset -ne $helloRecoverySectorInspectionAfterMaterialize.audit_record_byte_length -or $helloRecoverySectorInspectionAfterMaterialize.padding_offset -ne ($helloRecoverySectorInspectionAfterMaterialize.audit_record_byte_length + $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_byte_length)) {
        throw "Expected recovery rollback inspect after materialize dry-run to expose canonical LBA1 append offsets"
    }
    if (-not $helloRecoverySectorInspectionAfterMaterialize.read_attempted -or -not $helloRecoverySectorInspectionAfterMaterialize.read_completed -or -not $helloRecoverySectorInspectionAfterMaterialize.sector_hash_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.audit_record_hash_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_hash_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.offsets_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.padding_zeroed -or -not $helloRecoverySectorInspectionAfterMaterialize.target_span_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.target_region_write_readback_verified -or -not $helloRecoverySectorInspectionAfterMaterialize.inspection_verified) {
        throw "Expected recovery rollback inspect after materialize dry-run to verify target-sector read, hashes, offsets, and padding"
    }
    if (-not $helloRecoveryInspectSourceAfterMaterialize -or $helloRecoveryInspectSourceAfterMaterialize.schema -ne "raios.recovery_rollback_inspect_source_reference.v0" -or $helloRecoveryInspectSourceAfterMaterialize.id -ne "recovery_rollback_inspect_source.current_boot.svc.demo.hello.v0" -or $helloRecoveryInspectSourceAfterMaterialize.status -ne "retained_sector_inspection_source_reference" -or $helloRecoveryInspectSourceAfterMaterialize.reason -ne "retained_recovery_rollback_inspect_source_matches_sector_inspection" -or $helloRecoveryInspectSourceAfterMaterialize.source_method -ne "recovery.rollback_inspect" -or $helloRecoveryInspectSourceAfterMaterialize.source_event_id -eq $null -or $helloRecoveryInspectSourceAfterMaterialize.source_audit_event_id -eq $null -or -not $helloRecoveryInspectSourceAfterMaterialize.source_available -or -not $helloRecoveryInspectSourceAfterMaterialize.source_matches_sector_inspection -or -not $helloRecoveryInspectSourceAfterMaterialize.source_event_retained -or -not $helloRecoveryInspectSourceAfterMaterialize.source_audit_event_retained -or $helloRecoveryInspectSourceAfterMaterialize.ram_audit_status -ne "retained_audit_event_verified" -or $helloRecoveryInspectSourceAfterMaterialize.ram_audit_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified" -or -not $helloRecoveryInspectSourceAfterMaterialize.ram_audit_validated -or -not $helloRecoveryInspectSourceAfterMaterialize.reference_hash.StartsWith("sha256:") -or $helloRecoveryInspectSourceAfterMaterialize.source_inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash -or $helloRecoveryInspectSourceAfterMaterialize.target_region_sector_inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash -or $helloRecoveryInspectSourceAfterMaterialize.source_sector_plan_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_sector_plan_hash -or $helloRecoveryInspectSourceAfterMaterialize.source_target_region_write_readback_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloRecoveryInspectSourceAfterMaterialize.authorizes_rollback_apply) {
        throw "Expected recovery rollback inspect to retain a non-authorizing source reference over the verified sector inspection"
    }

    $helloDryRunApplyOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "service.rollback_apply svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply"
    $helloDryRunApply = Get-LastAgentResponseJson -Method "service.rollback_apply"
    $helloApplied = $helloDryRunApply.body.result
    if ($helloDryRunApply.t -ne "response" -or $helloApplied.schema -ne "raios.ram_only_hello_service_rollback_apply.v0" -or $helloApplied.status -ne "current_boot_rollback_applied" -or $helloApplied.reason -ne "verified_authorized_append_readback_and_inspection") {
        throw "Expected Hello rollback apply to apply after the verified append/readback/inspection chain"
    }
    if ($helloApplied.authority_record.rollback_transaction_hash -ne $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_image_hash -or $helloApplied.authority_record.write_readback_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloApplied.authority_record.inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash) {
        throw "Expected applied rollback evidence to cite transaction, write/readback, and inspection hashes"
    }
    if (-not $helloApplied.authority_record.authorized_append_hash.StartsWith("sha256:") -or -not $helloApplied.authority_record.scope_decision_hash.StartsWith("sha256:") -or $helloApplied.authority_record.inspected_rollback_transaction_hash -ne $helloApplied.authority_record.rollback_transaction_hash) {
        throw "Expected applied rollback authority record to bind authorized append and inspected transaction hashes"
    }
    if ($helloApplied.state_transition.from_version -ne "v2" -or $helloApplied.state_transition.to_version -ne "v1" -or $helloApplied.state_transition.from_generation -ne 2 -or $helloApplied.state_transition.to_generation -ne 1 -or -not $helloApplied.state_transition.state_counter_preserved -or -not $helloApplied.state_transition.current_boot_service_state_mutated) {
        throw "Expected rollback apply to transition the current boot service from v2 back to previous-good v1 state"
    }
    if (-not $helloApplied.side_effects.authorizes_media_write -or -not $helloApplied.side_effects.authorizes_append -or -not $helloApplied.side_effects.authorizes_transaction_append -or -not $helloApplied.side_effects.writes_durable_audit_log -or -not $helloApplied.side_effects.writes_rollback_store -or -not $helloApplied.side_effects.appends_rollback_transaction -or -not $helloApplied.side_effects.applies_rollback -or -not $helloApplied.side_effects.mutates_service_state -or $helloApplied.side_effects.installs_rollback_state -or $helloApplied.side_effects.persistent_install -ne "denied" -or $helloApplied.side_effects.external_artifact_load -ne "denied" -or $helloApplied.side_effects.broad_mutation -ne "denied") {
        throw "Expected rollback apply side effects to apply the current-boot service only while broad/persistent mutation stays denied"
    }

    Send-AgentCommand -Command "recovery.rollback_inspect svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloRecoveryInspect = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    $helloRecoverySectorInspection = $helloRecoveryInspect.body.result.target_region_sector_inspection
    $helloRecoveryTargetWriteReadback = $helloRecoveryInspect.body.result.target_region_write_readback
    $helloRecoveryAppliedAuthorityRecord = $helloRecoveryInspect.body.result.applied_authority_record
    if ($helloRecoveryInspect.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRecoveryInspect.body.result.status -ne "rollback_applied_transaction_inspected" -or -not $helloRecoveryInspect.body.result.read_only -or -not $helloRecoveryInspect.body.result.materialized_sector_evidence_available -or -not $helloRecoveryInspect.body.result.inspection_available) {
        throw "Expected recovery rollback inspect to expose read-only applied transaction sector evidence"
    }
    if (-not $helloRecoveryAppliedAuthorityRecord -or $helloRecoveryAppliedAuthorityRecord.schema -ne "raios.rollback_transaction.v0" -or $helloRecoveryAppliedAuthorityRecord.source_method -ne "service.rollback_apply" -or $helloRecoveryAppliedAuthorityRecord.source_event_id -ne $helloApplied.event_id -or $helloRecoveryAppliedAuthorityRecord.source_audit_event_id -ne $helloApplied.audit_event_id -or $helloRecoveryAppliedAuthorityRecord.status -ne "current_boot_rollback_applied" -or $helloRecoveryAppliedAuthorityRecord.rollback_transaction_hash -ne $helloApplied.authority_record.rollback_transaction_hash -or $helloRecoveryAppliedAuthorityRecord.write_readback_hash -ne $helloApplied.authority_record.write_readback_hash -or $helloRecoveryAppliedAuthorityRecord.inspection_hash -ne $helloApplied.authority_record.inspection_hash -or $helloRecoveryAppliedAuthorityRecord.audit_record_hash -ne $helloApplied.authority_record.audit_record_hash -or $helloRecoveryAppliedAuthorityRecord.inspected_rollback_transaction_hash -ne $helloApplied.authority_record.inspected_rollback_transaction_hash -or $helloRecoveryAppliedAuthorityRecord.target_region_write_readback_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloRecoveryAppliedAuthorityRecord.target_region_sector_inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash) {
        throw "Expected recovery rollback inspect to reference the applied authority record and same transaction hashes"
    }
    if (-not $helloRecoveryTargetWriteReadback -or $helloRecoveryTargetWriteReadback.dry_run_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloRecoveryTargetWriteReadback.planned_sector_image_hash -ne $helloMaterializeTargetWriteReadback.planned_sector_image_hash -or $helloRecoveryTargetWriteReadback.readback_sector_image_hash -ne $helloMaterializeTargetWriteReadback.readback_sector_image_hash) {
        throw "Expected recovery rollback inspect to reuse retained target-region write/readback evidence"
    }
    if (-not $helloRecoverySectorInspection -or $helloRecoverySectorInspection.inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash -or $helloRecoverySectorInspection.source_sector_plan_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_sector_plan_hash -or $helloRecoverySectorInspection.source_target_region_write_readback_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloRecoverySectorInspection.audit_record_image_hash -ne $helloRecoverySectorInspectionAfterMaterialize.audit_record_image_hash -or $helloRecoverySectorInspection.rollback_transaction_image_hash -ne $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_image_hash) {
        throw "Expected recovery rollback inspect to return the same sector, audit-record, and rollback-transaction hashes"
    }
    if ($helloRecoverySectorInspection.target_start_lba -ne 1 -or $helloRecoverySectorInspection.target_lba_count -ne 1 -or $helloRecoverySectorInspection.target_byte_count -ne 512 -or $helloRecoverySectorInspection.audit_record_offset -ne 0 -or $helloRecoverySectorInspection.rollback_transaction_offset -ne $helloRecoverySectorInspection.audit_record_byte_length -or $helloRecoverySectorInspection.padding_offset -ne ($helloRecoverySectorInspection.audit_record_byte_length + $helloRecoverySectorInspection.rollback_transaction_byte_length)) {
        throw "Expected recovery rollback inspect to expose the canonical LBA1 append offsets"
    }
    if (-not $helloRecoverySectorInspection.read_attempted -or -not $helloRecoverySectorInspection.read_completed -or -not $helloRecoverySectorInspection.sector_hash_verified -or -not $helloRecoverySectorInspection.audit_record_hash_verified -or -not $helloRecoverySectorInspection.rollback_transaction_hash_verified -or -not $helloRecoverySectorInspection.offsets_verified -or -not $helloRecoverySectorInspection.padding_zeroed -or -not $helloRecoverySectorInspection.target_span_verified -or -not $helloRecoverySectorInspection.target_region_write_readback_verified -or -not $helloRecoverySectorInspection.inspection_verified) {
        throw "Expected recovery rollback inspect to verify the target-sector read, hashes, offsets, and padding"
    }
    if ($helloRecoveryInspect.body.result.denied_surfaces.authorizes_media_write -or $helloRecoveryInspect.body.result.denied_surfaces.authorizes_append -or $helloRecoveryInspect.body.result.denied_surfaces.authorizes_transaction_append -or $helloRecoveryInspect.body.result.denied_surfaces.writes_durable_audit_log -or $helloRecoveryInspect.body.result.denied_surfaces.writes_rollback_store -or $helloRecoveryInspect.body.result.denied_surfaces.appends_rollback_transaction -or $helloRecoveryInspect.body.result.denied_surfaces.applies_rollback -or $helloRecoveryInspect.body.result.denied_surfaces.installs_rollback_state) {
        throw "Recovery rollback inspect must remain read-only and non-applying"
    }
    $helloAuthorizedAppend = Get-LastMarkerJsonAfterOffset -Prefix "RAIOS_ROLLBACK_AUTHORIZED_APPEND" -Offset $helloDryRunApplyOffset -Name "hello_rollback_dry_run:authorized_append_marker"
    if ($helloAuthorizedAppend.schema -ne "raios.scoped_rollback_authorized_append.v0" -or $helloAuthorizedAppend.id -ne "scoped_rollback_authorized_append.current_boot.svc.demo.hello.v0" -or $helloAuthorizedAppend.status -ne "performed" -or $helloAuthorizedAppend.reason -ne "authorized_lba1_transaction_append_readback_and_inspection_verified" -or -not $helloAuthorizedAppend.transaction_append_performed) {
        throw "Expected scoped rollback authorized append evidence to report a performed append"
    }
    if (-not $helloAuthorizedAppend.append_hash.StartsWith("sha256:") -or -not $helloAuthorizedAppend.scope_decision.authorized -or -not $helloAuthorizedAppend.scope_decision.decision_hash.StartsWith("sha256:")) {
        throw "Expected authorized append evidence to bind the positive scope decision hash"
    }
    if ($helloAuthorizedAppend.target_scope.target_region_id -ne "target_region.audit_rollback.current_boot" -or $helloAuthorizedAppend.target_scope.target_region_marker -ne "RAIOS_AUDITRB_V0" -or $helloAuthorizedAppend.target_scope.target_start_lba -ne 1 -or $helloAuthorizedAppend.target_scope.target_lba_count -ne 1 -or $helloAuthorizedAppend.target_scope.target_byte_count -ne 512) {
        throw "Expected authorized append to stay confined to the RAIOS_AUDITRB_V0 LBA1/512-byte target span"
    }
    if ($helloAuthorizedAppend.source_hashes.sector_plan_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_sector_plan_hash -or $helloAuthorizedAppend.source_hashes.write_readback_hash -ne $helloRecoverySectorInspectionAfterMaterialize.source_target_region_write_readback_hash -or $helloAuthorizedAppend.source_hashes.inspection_hash -ne $helloRecoverySectorInspectionAfterMaterialize.inspection_hash) {
        throw "Expected authorized append to bind the same scoped sector plan, write/readback, and inspection evidence"
    }
    if ($helloAuthorizedAppend.sector_hashes.planned_sector_image_hash -ne $helloMaterializeTargetWriteReadback.planned_sector_image_hash -or $helloAuthorizedAppend.sector_hashes.readback_sector_image_hash -ne $helloMaterializeTargetWriteReadback.readback_sector_image_hash -or $helloAuthorizedAppend.sector_hashes.inspected_sector_image_hash -ne $helloRecoverySectorInspectionAfterMaterialize.sector_image_hash) {
        throw "Expected authorized append to bind planned, readback, and inspected sector hashes"
    }
    if ($helloAuthorizedAppend.transaction_hashes.audit_record_image_hash -ne $helloRecoverySectorInspectionAfterMaterialize.audit_record_image_hash -or $helloAuthorizedAppend.transaction_hashes.rollback_transaction_image_hash -ne $helloRecoverySectorInspectionAfterMaterialize.rollback_transaction_image_hash -or $helloAuthorizedAppend.transaction_hashes.rollback_transaction_image_hash -ne $helloAuthorizedAppend.transaction_hashes.inspected_rollback_transaction_image_hash) {
        throw "Expected authorized append to bind audit-record and rollback-transaction hashes"
    }
    if ($helloAuthorizedAppend.sector_layout.audit_record_offset -ne 0 -or $helloAuthorizedAppend.sector_layout.rollback_transaction_offset -ne $helloAuthorizedAppend.sector_layout.audit_record_byte_length -or $helloAuthorizedAppend.sector_layout.padding_offset -ne ($helloAuthorizedAppend.sector_layout.audit_record_byte_length + $helloAuthorizedAppend.sector_layout.rollback_transaction_byte_length) -or $helloAuthorizedAppend.sector_layout.padding_byte_length -ne 32) {
        throw "Expected authorized append to verify canonical offsets and zero-padding length"
    }
    if (-not $helloAuthorizedAppend.write_readback.write_attempted -or -not $helloAuthorizedAppend.write_readback.write_completed -or -not $helloAuthorizedAppend.write_readback.readback_completed -or -not $helloAuthorizedAppend.write_readback.readback_matches_planned_image -or -not $helloAuthorizedAppend.inspection.read_attempted -or -not $helloAuthorizedAppend.inspection.read_completed -or -not $helloAuthorizedAppend.inspection.sector_hash_verified -or -not $helloAuthorizedAppend.inspection.audit_record_hash_verified -or -not $helloAuthorizedAppend.inspection.rollback_transaction_hash_verified -or -not $helloAuthorizedAppend.inspection.offsets_verified -or -not $helloAuthorizedAppend.inspection.padding_zeroed -or -not $helloAuthorizedAppend.inspection.target_span_verified -or -not $helloAuthorizedAppend.inspection.inspection_verified) {
        throw "Expected authorized append to prove write/readback plus post-write inspection"
    }
    if (-not $helloAuthorizedAppend.side_effects.authorizes_media_write -or -not $helloAuthorizedAppend.side_effects.authorizes_append -or -not $helloAuthorizedAppend.side_effects.authorizes_transaction_append -or -not $helloAuthorizedAppend.side_effects.writes_durable_audit_log -or -not $helloAuthorizedAppend.side_effects.writes_rollback_store -or -not $helloAuthorizedAppend.side_effects.appends_rollback_transaction -or $helloAuthorizedAppend.side_effects.applies_rollback -or $helloAuthorizedAppend.side_effects.mutates_service_state -or $helloAuthorizedAppend.side_effects.installs_rollback_state) {
        throw "Expected authorized append side effects to stop at append/readback evidence without applying rollback"
    }

    $helloRecoveryInspectEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.rollback_inspect requested_capability=cap.recovery.rollback_inspect.read classification=local_only"
    Send-AgentCommand -Command $helloRecoveryInspectEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloRecoveryInspectEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $helloRecoveryInspectEnvelope.body.result.accepted -or $helloRecoveryInspectEnvelope.body.result.target_method -ne "recovery.rollback_inspect" -or -not ($helloRecoveryInspectEnvelope.body.result.allowed_target_methods -contains "recovery.rollback_inspect") -or $helloRecoveryInspectEnvelope.body.result.allowed_requested_capability -ne "cap.recovery.rollback_inspect.read" -or -not $helloRecoveryInspectEnvelope.body.result.dispatches_existing_agent_method -or $helloRecoveryInspectEnvelope.body.result.writes_durable_audit_log -or $helloRecoveryInspectEnvelope.body.result.installs_rollback_plan -or $helloRecoveryInspectEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch recovery.rollback_inspect without durable rollback authority"
    }
    $helloRecoveryInspectEnvelopeResponse = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    if ($helloRecoveryInspectEnvelopeResponse.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRecoveryInspectEnvelopeResponse.body.result.status -ne "rollback_applied_transaction_inspected" -or -not $helloRecoveryInspectEnvelopeResponse.body.result.read_only -or -not $helloRecoveryInspectEnvelopeResponse.body.result.materialized_sector_evidence_available -or -not $helloRecoveryInspectEnvelopeResponse.body.result.inspection_available -or -not $helloRecoveryInspectEnvelopeResponse.body.result.applied_authority_record -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_media_write -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_append -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_transaction_append -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.writes_durable_audit_log -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.writes_rollback_store -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.appends_rollback_transaction -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.applies_rollback -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.installs_rollback_state) {
        throw "Expected enveloped recovery.rollback_inspect to stay read-only and non-applying"
    }

    $helloRecoveryInspectMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.rollback_inspect requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $helloRecoveryInspectMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($helloRecoveryInspectMismatch.body.result.accepted -or $helloRecoveryInspectMismatch.body.result.reason -ne "requested_capability_denied" -or $helloRecoveryInspectMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected recovery.rollback_inspect envelope with wrong capability to be denied before dispatch"
    }
    $helloRecoveryInspectMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($helloRecoveryInspectMismatchOffset)
    $helloRecoveryInspectMismatchNoDispatch = -not $helloRecoveryInspectMismatchAfter.Contains("RAIOS_AGENT_END recovery.rollback_inspect")
    Add-Predicate -Name "protocol:hello_recovery_rollback_inspect_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END recovery.rollback_inspect" -Passed $helloRecoveryInspectMismatchNoDispatch -Actual $(if ($helloRecoveryInspectMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $helloRecoveryInspectMismatchNoDispatch) {
        throw "Expected recovery.rollback_inspect capability mismatch to avoid inspect dispatch"
    }

    Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
    $helloDryRunHealth = Get-LastAgentResponseJson -Method "service.health"
    $helloDryRunHealthState = @($helloDryRunHealth.evidence | Where-Object { $_.id -eq "state_transition" })[0]
    if ($helloDryRunHealth.family -ne "hello.health" -or $helloDryRunHealthState.facts.after.version -ne "v1" -or -not $helloDryRunHealthState.facts.after.running) {
        throw "Expected applied rollback to restore the active v1 Hello service"
    }

    Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
    $helloDryRunDrop = Get-LastAgentResponseJson -Method "service.drop"
    $helloDryRunDropState = @($helloDryRunDrop.evidence | Where-Object { $_.id -eq "state_transition" })[0]
    if ($helloDryRunDrop.family -ne "hello.lifecycle" -or $helloDryRunDrop.decision.reason -ne "drop_performed" -or $helloDryRunDropState.facts.after.loaded -or $helloDryRunDropState.facts.after.running) {
        throw "Expected Hello rollback dry-run profile to drop the service after verification"
    }

    Send-AgentCommand -Command "agent audit.events 96" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "hello_rollback_dry_run:authorized_append_schema" -Needle '"schema": "raios.scoped_rollback_authorized_append.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:authorized_append_performed" -Needle '"transaction_append_performed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:authorized_append_no_apply" -Needle '"applies_rollback": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:rollback_apply_applied_status" -Needle '"status": "current_boot_rollback_applied"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:rollback_apply_authority_record_hash" -Needle '"rollback_transaction_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:post_apply_inspect_applied_status" -Needle '"status": "rollback_applied_transaction_inspected"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:post_apply_inspect_applied_authority" -Needle '"applied_authority_record": {"schema": "raios.rollback_transaction.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:rollback_apply_binding_schema" -Needle '"schema": "raios.ram_only_hello_service.rollback_apply_applied_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:rollback_apply_authorized" -Needle '"rollback_apply_authorized": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:rollback_apply_mutates_state" -Needle '"rollback_apply_mutates_service_state": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_binding_schema" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_schema": "raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_binding_hash" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_binding_blocked" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_blocked_by_authority_denial_gate": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_binding_no_append" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_appends_rollback_transaction": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_ledger_availability_dry_run_schema" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_ledger_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_ledger_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_ledger_availability_dry_run_no_append" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_appends_rollback_transaction": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_ledger_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_availability_dry_run_schema" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_availability_dry_run_no_append" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_appends_rollback_transaction": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:append_authority_availability_dry_run_schema" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:append_authority_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:append_authority_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:append_authority_availability_dry_run_no_append" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_appends_rollback_transaction": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:append_authority_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_sector_inspection_schema" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_schema": "raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_sector_inspection_hash" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_sector_inspection_verified" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_sector_inspection_no_append" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_appends_rollback_transaction": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_schema" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_schema": "raios.recovery_rollback_inspect_source_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_hash" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_audit_event" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_id": ' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_available" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_available": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_matches" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_matches_sector_inspection": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_ram_audit_validated" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_validated": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_binding" -Needle '"schema": "raios.recovery_rollback_inspect_source_reference_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_inspect_source_no_apply" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_authorizes_rollback_apply": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_write_decision_schema" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_schema": "raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_write_decision_hash" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_write_decision_consumes_dry_run" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_dry_run_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_write_decision_consumes_sector_inspection" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_region_sector_inspection_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_transaction_applied" -Needle '"rollback_transaction_applies_rollback": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_policy_decision" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_inspect_reference" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_policy_verified" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_inspect_validated" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_validated": true' -TimeoutSeconds 1
