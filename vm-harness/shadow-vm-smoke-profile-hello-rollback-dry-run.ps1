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
    if ($helloDryRunLoad.body.result.schema -ne "raios.ram_only_hello_service.v0" -or -not $helloDryRunLoad.body.result.service.loaded -or -not $helloDryRunLoad.body.result.service.running) {
        throw "Expected Hello rollback dry-run profile to load the built-in RAM-only service"
    }

    $helloHealthEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.health requested_capability=cap.service.health.read classification=local_only"
    Send-AgentCommand -Command $helloHealthEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END service.health"
    $helloHealthEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $helloHealthEnvelope.body.result.accepted -or $helloHealthEnvelope.body.result.target_method -ne "service.health" -or -not ($helloHealthEnvelope.body.result.allowed_target_methods -contains "service.health") -or $helloHealthEnvelope.body.result.allowed_requested_capability -ne "cap.service.health.read" -or -not $helloHealthEnvelope.body.result.dispatches_existing_agent_method -or $helloHealthEnvelope.body.result.writes_durable_audit_log -or $helloHealthEnvelope.body.result.installs_rollback_plan -or $helloHealthEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch service.health without durable rollback authority"
    }
    $helloHealthEnvelopeResponse = Get-LastAgentResponseJson -Method "service.health"
    if ($helloHealthEnvelopeResponse.body.result.schema -ne "raios.ram_only_hello_service.health.v0" -or $helloHealthEnvelopeResponse.body.result.service.version -ne "v1" -or -not $helloHealthEnvelopeResponse.body.result.service.loaded -or -not $helloHealthEnvelopeResponse.body.result.service.running) {
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
    if ($helloDryRunHotSwap.body.result.service.version -ne "v2" -or -not $helloDryRunHotSwap.body.result.hot_swap_probation.probation_hash.StartsWith("sha256:")) {
        throw "Expected Hello rollback dry-run profile to create v2 hot-swap probation evidence"
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

    Send-AgentCommand -Command "service.rollback_apply svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply"
    $helloDryRunApply = Get-LastAgentResponseJson -Method "service.rollback_apply"
    if ($helloDryRunApply.t -ne "error" -or $helloDryRunApply.body.code -ne "capability_denied" -or $helloDryRunApply.body.reason -ne "rollback_apply_authority_missing") {
        throw "Expected Hello rollback apply to stay denied by missing rollback authority"
    }

    $helloDryRunPreflight = $helloDryRunApply.body.rollback_transaction_writer_storage_authority_gate.writer_storage_foundation.transaction_writer_readiness.scratch_only_writer_dry_run.append_record_dry_run.sector_plan_dry_run.scratch_sector_write_readback_dry_run.durable_append_authority_preflight
    $helloWriteAuthorityAvailability = $helloDryRunPreflight.durable_audit_policy_write_authority_availability
    $helloPolicyLedgerAvailability = $helloDryRunPreflight.durable_policy_ledger_availability
    $helloAuditPolicyAvailability = $helloDryRunPreflight.durable_audit_policy_availability
    $helloAppendAuthorityAvailability = $helloDryRunPreflight.durable_append_authority_availability
    $helloDryRunDecision = $helloDryRunPreflight.transaction_append_availability_decision
    $helloDryRunDenialGate = $helloDryRunPreflight.transaction_append_authority_denial_gate
    $helloPolicyLedgerAvailabilityDryRun = $helloDryRunPreflight.durable_policy_ledger_availability_dry_run
    $helloAuditPolicyAvailabilityDryRun = $helloDryRunPreflight.durable_audit_policy_availability_dry_run
    $helloAppendAuthorityAvailabilityDryRun = $helloDryRunPreflight.durable_append_authority_availability_dry_run
    $helloDryRunEvidence = $helloDryRunPreflight.transaction_append_dry_run
    $helloSectorInspection = $helloDryRunPreflight.target_region_sector_inspection
    $helloInspectSource = $helloDryRunPreflight.retained_recovery_rollback_inspect_source
    $helloPolicyWriteAuthorityDecision = $helloDryRunPreflight.durable_policy_write_authority_decision

    if (-not $helloDryRunEvidence -or $helloDryRunEvidence.schema -ne "raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0" -or $helloDryRunEvidence.id -ne "hello_rollback_transaction_append_dry_run.current_boot.svc.demo.hello.v0" -or $helloDryRunEvidence.status -ne "blocked_by_transaction_append_authority_denial_gate" -or $helloDryRunEvidence.reason -ne "missing_transaction_append_authority") {
        throw "Expected Hello rollback apply denial to expose transaction-append dry-run evidence"
    }
    if (-not $helloDryRunEvidence.dry_run_hash.StartsWith("sha256:") -or $helloDryRunEvidence.source_authority_denial_gate_hash -ne $helloDryRunDenialGate.gate_hash -or $helloDryRunEvidence.source_transaction_append_availability_decision_hash -ne $helloDryRunDecision.decision_hash) {
        throw "Expected transaction-append dry-run to bind the authority-denial gate and availability decision hashes"
    }
    if ($helloDryRunEvidence.audit_ledger_target_id -ne "append.audit_ledger.current_boot" -or $helloDryRunEvidence.audit_record_schema -ne "raios.audit_record.v0" -or $helloDryRunEvidence.rollback_store_target_id -ne "append.rollback_store.current_boot" -or $helloDryRunEvidence.rollback_transaction_schema -ne "raios.rollback_transaction.v0") {
        throw "Expected transaction-append dry-run to bind audit and rollback append targets"
    }
    if ($helloDryRunEvidence.target_start_lba -ne 1 -or $helloDryRunEvidence.target_lba_count -ne 1 -or $helloDryRunEvidence.target_byte_count -ne 512) {
        throw "Expected transaction-append dry-run to stay confined to the LBA1/512-byte target span"
    }
    if (-not $helloDryRunEvidence.authority_denial_gate_verified -or -not $helloDryRunEvidence.target_span_verified -or -not $helloDryRunEvidence.target_region_write_readback_verified -or -not $helloDryRunEvidence.append_image_ready -or -not $helloDryRunEvidence.blocked_by_authority_denial_gate -or -not $helloDryRunEvidence.test_infrastructure_media_write_authority_available) {
        throw "Expected transaction-append dry-run to prove test-media readiness while blocked by authority denial"
    }
    if ($helloDryRunEvidence.transaction_append_available -or $helloDryRunEvidence.authorizes_media_write -or $helloDryRunEvidence.authorizes_append -or $helloDryRunEvidence.authorizes_transaction_append -or $helloDryRunEvidence.writes_durable_audit_log -or $helloDryRunEvidence.writes_rollback_store -or $helloDryRunEvidence.appends_rollback_transaction -or $helloDryRunEvidence.transaction_append_attempted) {
        throw "Transaction-append dry-run must not authorize writes, append, transaction append, or durable side effects"
    }
    if (-not $helloPolicyLedgerAvailabilityDryRun -or $helloPolicyLedgerAvailabilityDryRun.schema -ne "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0" -or $helloPolicyLedgerAvailabilityDryRun.id -ne "hello_rollback_durable_policy_ledger_availability_dry_run.current_boot.svc.demo.hello.v0" -or $helloPolicyLedgerAvailabilityDryRun.status -ne "test_media_policy_ledger_availability_verified_not_durable_authority" -or $helloPolicyLedgerAvailabilityDryRun.reason -ne "policy_ledger_availability_bound_to_transaction_denial_current_boot") {
        throw "Expected Hello rollback apply denial to expose durable policy-ledger availability dry-run evidence"
    }
    if (-not $helloPolicyLedgerAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $helloPolicyLedgerAvailabilityDryRun.source_policy_ledger_availability_hash -ne $helloPolicyLedgerAvailability.availability_hash -or $helloPolicyLedgerAvailabilityDryRun.source_write_authority_availability_hash -ne $helloWriteAuthorityAvailability.availability_hash -or $helloPolicyLedgerAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_aware_acceptance_result.result_hash -or $helloPolicyLedgerAvailabilityDryRun.source_ledger_candidate_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_candidate.ledger_candidate_hash -or $helloPolicyLedgerAvailabilityDryRun.source_policy_preflight_hash -ne $helloDryRunPreflight.target_region_media_write_policy_preflight.preflight_hash -or $helloPolicyLedgerAvailabilityDryRun.source_target_region_write_readback_hash -ne $helloDryRunPreflight.target_region_write_readback_dry_run.dry_run_hash -or $helloPolicyLedgerAvailabilityDryRun.source_authority_denial_gate_hash -ne $helloDryRunDenialGate.gate_hash -or $helloPolicyLedgerAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $helloDryRunDecision.decision_hash) {
        throw "Expected durable policy-ledger availability dry-run to bind policy, ledger, target-region, and transaction-denial hashes"
    }
    if ($helloPolicyLedgerAvailabilityDryRun.audit_ledger_target_id -ne "append.audit_ledger.current_boot" -or $helloPolicyLedgerAvailabilityDryRun.audit_record_schema -ne "raios.audit_record.v0" -or $helloPolicyLedgerAvailabilityDryRun.rollback_store_target_id -ne "append.rollback_store.current_boot" -or $helloPolicyLedgerAvailabilityDryRun.rollback_transaction_schema -ne "raios.rollback_transaction.v0") {
        throw "Expected durable policy-ledger availability dry-run to bind audit and rollback append targets"
    }
    if ($helloPolicyLedgerAvailabilityDryRun.target_start_lba -ne 1 -or $helloPolicyLedgerAvailabilityDryRun.target_lba_count -ne 1 -or $helloPolicyLedgerAvailabilityDryRun.target_byte_count -ne 512 -or -not $helloPolicyLedgerAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $helloPolicyLedgerAvailabilityDryRun.write_authority_evidence_verified -or -not $helloPolicyLedgerAvailabilityDryRun.ledger_evidence_verified -or -not $helloPolicyLedgerAvailabilityDryRun.media_write_policy_verified -or -not $helloPolicyLedgerAvailabilityDryRun.target_region_write_readback_verified -or -not $helloPolicyLedgerAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $helloPolicyLedgerAvailabilityDryRun.target_span_verified -or -not $helloPolicyLedgerAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $helloPolicyLedgerAvailabilityDryRun.test_infrastructure_media_write_authority_available) {
        throw "Expected durable policy-ledger availability dry-run to verify the LBA1/512-byte target span and denial evidence"
    }
    if ($helloPolicyLedgerAvailabilityDryRun.write_authority_available -or $helloPolicyLedgerAvailabilityDryRun.durable_policy_ledger_available -or $helloPolicyLedgerAvailabilityDryRun.durable_audit_policy_available -or $helloPolicyLedgerAvailabilityDryRun.durable_append_authority_available -or $helloPolicyLedgerAvailabilityDryRun.transaction_append_available -or $helloPolicyLedgerAvailabilityDryRun.authorizes_media_write -or $helloPolicyLedgerAvailabilityDryRun.authorizes_append -or $helloPolicyLedgerAvailabilityDryRun.authorizes_transaction_append -or $helloPolicyLedgerAvailabilityDryRun.writes_durable_audit_log -or $helloPolicyLedgerAvailabilityDryRun.writes_rollback_store -or $helloPolicyLedgerAvailabilityDryRun.appends_rollback_transaction -or $helloPolicyLedgerAvailabilityDryRun.write_attempted -or $helloPolicyLedgerAvailabilityDryRun.applies_rollback -or $helloPolicyLedgerAvailabilityDryRun.installs_rollback_state) {
        throw "Durable policy-ledger availability dry-run must not authorize writes, appends, transaction append, rollback application, or installed rollback state"
    }
    if (-not $helloAuditPolicyAvailabilityDryRun -or $helloAuditPolicyAvailabilityDryRun.schema -ne "raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0" -or $helloAuditPolicyAvailabilityDryRun.id -ne "hello_rollback_durable_audit_policy_availability_dry_run.current_boot.svc.demo.hello.v0" -or $helloAuditPolicyAvailabilityDryRun.status -ne "test_media_audit_policy_availability_verified_not_durable_authority" -or $helloAuditPolicyAvailabilityDryRun.reason -ne "audit_policy_availability_bound_to_transaction_denial_current_boot") {
        throw "Expected Hello rollback apply denial to expose durable audit-policy availability dry-run evidence"
    }
    if (-not $helloAuditPolicyAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $helloAuditPolicyAvailabilityDryRun.source_audit_policy_availability_hash -ne $helloAuditPolicyAvailability.availability_hash -or $helloAuditPolicyAvailabilityDryRun.source_policy_ledger_availability_dry_run_hash -ne $helloPolicyLedgerAvailabilityDryRun.dry_run_hash -or $helloAuditPolicyAvailabilityDryRun.source_policy_ledger_availability_hash -ne $helloPolicyLedgerAvailability.availability_hash -or $helloAuditPolicyAvailabilityDryRun.source_write_authority_availability_hash -ne $helloWriteAuthorityAvailability.availability_hash -or $helloAuditPolicyAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_aware_acceptance_result.result_hash -or $helloAuditPolicyAvailabilityDryRun.source_ledger_candidate_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_candidate.ledger_candidate_hash -or $helloAuditPolicyAvailabilityDryRun.source_policy_preflight_hash -ne $helloDryRunPreflight.target_region_media_write_policy_preflight.preflight_hash -or $helloAuditPolicyAvailabilityDryRun.source_target_region_write_readback_hash -ne $helloDryRunPreflight.target_region_write_readback_dry_run.dry_run_hash -or $helloAuditPolicyAvailabilityDryRun.source_authority_denial_gate_hash -ne $helloDryRunDenialGate.gate_hash -or $helloAuditPolicyAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $helloDryRunDecision.decision_hash) {
        throw "Expected durable audit-policy availability dry-run to bind audit-policy, policy-ledger dry-run, target-region, and transaction-denial hashes"
    }
    if ($helloAuditPolicyAvailabilityDryRun.audit_ledger_target_id -ne "append.audit_ledger.current_boot" -or $helloAuditPolicyAvailabilityDryRun.audit_record_schema -ne "raios.audit_record.v0" -or $helloAuditPolicyAvailabilityDryRun.rollback_store_target_id -ne "append.rollback_store.current_boot" -or $helloAuditPolicyAvailabilityDryRun.rollback_transaction_schema -ne "raios.rollback_transaction.v0") {
        throw "Expected durable audit-policy availability dry-run to bind audit and rollback append targets"
    }
    if ($helloAuditPolicyAvailabilityDryRun.target_start_lba -ne 1 -or $helloAuditPolicyAvailabilityDryRun.target_lba_count -ne 1 -or $helloAuditPolicyAvailabilityDryRun.target_byte_count -ne 512 -or -not $helloAuditPolicyAvailabilityDryRun.audit_policy_availability_evidence_verified -or -not $helloAuditPolicyAvailabilityDryRun.policy_ledger_dry_run_evidence_verified -or -not $helloAuditPolicyAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $helloAuditPolicyAvailabilityDryRun.write_authority_evidence_verified -or -not $helloAuditPolicyAvailabilityDryRun.ledger_evidence_verified -or -not $helloAuditPolicyAvailabilityDryRun.media_write_policy_verified -or -not $helloAuditPolicyAvailabilityDryRun.target_region_write_readback_verified -or -not $helloAuditPolicyAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $helloAuditPolicyAvailabilityDryRun.target_span_verified -or -not $helloAuditPolicyAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $helloAuditPolicyAvailabilityDryRun.test_infrastructure_media_write_authority_available) {
        throw "Expected durable audit-policy availability dry-run to verify the LBA1/512-byte target span and denial evidence"
    }
    if ($helloAuditPolicyAvailabilityDryRun.write_authority_available -or $helloAuditPolicyAvailabilityDryRun.durable_policy_ledger_available -or $helloAuditPolicyAvailabilityDryRun.durable_audit_policy_available -or $helloAuditPolicyAvailabilityDryRun.durable_append_authority_available -or $helloAuditPolicyAvailabilityDryRun.transaction_append_available -or $helloAuditPolicyAvailabilityDryRun.authorizes_media_write -or $helloAuditPolicyAvailabilityDryRun.authorizes_append -or $helloAuditPolicyAvailabilityDryRun.authorizes_transaction_append -or $helloAuditPolicyAvailabilityDryRun.writes_durable_audit_log -or $helloAuditPolicyAvailabilityDryRun.writes_rollback_store -or $helloAuditPolicyAvailabilityDryRun.appends_rollback_transaction -or $helloAuditPolicyAvailabilityDryRun.write_attempted -or $helloAuditPolicyAvailabilityDryRun.applies_rollback -or $helloAuditPolicyAvailabilityDryRun.installs_rollback_state) {
        throw "Durable audit-policy availability dry-run must not authorize writes, appends, transaction append, rollback application, or installed rollback state"
    }
    if (-not $helloAppendAuthorityAvailabilityDryRun -or $helloAppendAuthorityAvailabilityDryRun.schema -ne "raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0" -or $helloAppendAuthorityAvailabilityDryRun.id -ne "hello_rollback_durable_append_authority_availability_dry_run.current_boot.svc.demo.hello.v0" -or $helloAppendAuthorityAvailabilityDryRun.status -ne "test_media_append_authority_availability_verified_not_durable_authority" -or $helloAppendAuthorityAvailabilityDryRun.reason -ne "append_authority_availability_bound_to_transaction_denial_current_boot") {
        throw "Expected Hello rollback apply denial to expose durable append-authority availability dry-run evidence"
    }
    if (-not $helloAppendAuthorityAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $helloAppendAuthorityAvailabilityDryRun.source_append_authority_availability_hash -ne $helloAppendAuthorityAvailability.availability_hash -or $helloAppendAuthorityAvailabilityDryRun.source_audit_policy_availability_dry_run_hash -ne $helloAuditPolicyAvailabilityDryRun.dry_run_hash -or $helloAppendAuthorityAvailabilityDryRun.source_audit_policy_availability_hash -ne $helloAuditPolicyAvailability.availability_hash -or $helloAppendAuthorityAvailabilityDryRun.source_policy_ledger_availability_dry_run_hash -ne $helloPolicyLedgerAvailabilityDryRun.dry_run_hash -or $helloAppendAuthorityAvailabilityDryRun.source_policy_ledger_availability_hash -ne $helloPolicyLedgerAvailability.availability_hash -or $helloAppendAuthorityAvailabilityDryRun.source_write_authority_availability_hash -ne $helloWriteAuthorityAvailability.availability_hash -or $helloAppendAuthorityAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_aware_acceptance_result.result_hash -or $helloAppendAuthorityAvailabilityDryRun.source_ledger_candidate_hash -ne $helloDryRunPreflight.durable_audit_policy_ledger_candidate.ledger_candidate_hash -or $helloAppendAuthorityAvailabilityDryRun.source_policy_preflight_hash -ne $helloDryRunPreflight.target_region_media_write_policy_preflight.preflight_hash -or $helloAppendAuthorityAvailabilityDryRun.source_target_region_write_readback_hash -ne $helloDryRunPreflight.target_region_write_readback_dry_run.dry_run_hash -or $helloAppendAuthorityAvailabilityDryRun.source_authority_denial_gate_hash -ne $helloDryRunDenialGate.gate_hash -or $helloAppendAuthorityAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $helloDryRunDecision.decision_hash) {
        throw "Expected durable append-authority availability dry-run to bind append-authority, audit-policy dry-run, target-region, and transaction-denial hashes"
    }
    if ($helloAppendAuthorityAvailabilityDryRun.audit_ledger_target_id -ne "append.audit_ledger.current_boot" -or $helloAppendAuthorityAvailabilityDryRun.audit_record_schema -ne "raios.audit_record.v0" -or $helloAppendAuthorityAvailabilityDryRun.rollback_store_target_id -ne "append.rollback_store.current_boot" -or $helloAppendAuthorityAvailabilityDryRun.rollback_transaction_schema -ne "raios.rollback_transaction.v0") {
        throw "Expected durable append-authority availability dry-run to bind audit and rollback append targets"
    }
    if ($helloAppendAuthorityAvailabilityDryRun.target_start_lba -ne 1 -or $helloAppendAuthorityAvailabilityDryRun.target_lba_count -ne 1 -or $helloAppendAuthorityAvailabilityDryRun.target_byte_count -ne 512 -or -not $helloAppendAuthorityAvailabilityDryRun.append_authority_availability_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.audit_policy_dry_run_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.audit_policy_availability_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.policy_ledger_dry_run_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.write_authority_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.ledger_evidence_verified -or -not $helloAppendAuthorityAvailabilityDryRun.media_write_policy_verified -or -not $helloAppendAuthorityAvailabilityDryRun.target_region_write_readback_verified -or -not $helloAppendAuthorityAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $helloAppendAuthorityAvailabilityDryRun.target_span_verified -or -not $helloAppendAuthorityAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $helloAppendAuthorityAvailabilityDryRun.test_infrastructure_media_write_authority_available) {
        throw "Expected durable append-authority availability dry-run to verify the LBA1/512-byte target span and denial evidence"
    }
    if ($helloAppendAuthorityAvailabilityDryRun.write_authority_available -or $helloAppendAuthorityAvailabilityDryRun.durable_policy_ledger_available -or $helloAppendAuthorityAvailabilityDryRun.durable_audit_policy_available -or $helloAppendAuthorityAvailabilityDryRun.durable_append_authority_available -or $helloAppendAuthorityAvailabilityDryRun.transaction_append_available -or $helloAppendAuthorityAvailabilityDryRun.authorizes_media_write -or $helloAppendAuthorityAvailabilityDryRun.authorizes_append -or $helloAppendAuthorityAvailabilityDryRun.authorizes_transaction_append -or $helloAppendAuthorityAvailabilityDryRun.writes_durable_audit_log -or $helloAppendAuthorityAvailabilityDryRun.writes_rollback_store -or $helloAppendAuthorityAvailabilityDryRun.appends_rollback_transaction -or $helloAppendAuthorityAvailabilityDryRun.write_attempted -or $helloAppendAuthorityAvailabilityDryRun.applies_rollback -or $helloAppendAuthorityAvailabilityDryRun.installs_rollback_state) {
        throw "Durable append-authority availability dry-run must not authorize writes, appends, transaction append, rollback application, or installed rollback state"
    }
    if (-not $helloSectorInspection -or $helloSectorInspection.schema -ne "raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0" -or $helloSectorInspection.id -ne "hello_rollback_target_region_sector_inspection.current_boot.svc.demo.hello.v0" -or $helloSectorInspection.status -ne "target_region_sector_read_parsed_not_durable_authority" -or $helloSectorInspection.reason -ne "target_region_sector_read_and_append_offsets_verified_current_boot") {
        throw "Expected rollback apply denial to expose target-region sector inspection evidence"
    }
    if (-not $helloSectorInspection.inspection_hash.StartsWith("sha256:") -or $helloSectorInspection.source_sector_plan_hash -ne $helloDryRunEvidence.source_sector_plan_hash -or $helloSectorInspection.source_target_region_write_readback_hash -ne $helloDryRunEvidence.source_target_region_write_readback_hash -or $helloSectorInspection.expected_sector_image_hash -ne $helloDryRunEvidence.planned_sector_image_hash -or $helloSectorInspection.sector_image_hash -ne $helloDryRunEvidence.readback_sector_image_hash) {
        throw "Expected target-region sector inspection to bind sector plan, write/readback, and sector image hashes"
    }
    if ($helloSectorInspection.target_start_lba -ne 1 -or $helloSectorInspection.target_lba_count -ne 1 -or $helloSectorInspection.target_byte_count -ne 512 -or $helloSectorInspection.audit_record_offset -ne 0 -or $helloSectorInspection.rollback_transaction_offset -ne $helloSectorInspection.audit_record_byte_length -or $helloSectorInspection.padding_offset -ne ($helloSectorInspection.audit_record_byte_length + $helloSectorInspection.rollback_transaction_byte_length)) {
        throw "Expected target-region sector inspection to parse the canonical append offsets in LBA1"
    }
    if (-not $helloSectorInspection.label_found -or -not $helloSectorInspection.read_attempted -or -not $helloSectorInspection.read_completed -or -not $helloSectorInspection.sector_hash_verified -or -not $helloSectorInspection.audit_record_hash_verified -or -not $helloSectorInspection.rollback_transaction_hash_verified -or -not $helloSectorInspection.offsets_verified -or -not $helloSectorInspection.padding_zeroed -or -not $helloSectorInspection.target_span_verified -or -not $helloSectorInspection.target_region_write_readback_verified -or -not $helloSectorInspection.inspection_verified) {
        throw "Expected target-region sector inspection to verify read, offsets, hashes, padding, and target span"
    }
    if ($helloSectorInspection.authorizes_media_write -or $helloSectorInspection.authorizes_append -or $helloSectorInspection.writes_durable_audit_log -or $helloSectorInspection.writes_rollback_store -or $helloSectorInspection.appends_rollback_transaction -or $helloSectorInspection.installs_rollback_state) {
        throw "Target-region sector inspection must not authorize writes, append, or installed rollback state"
    }
    if (-not $helloInspectSource -or $helloInspectSource.status -ne "retained_sector_inspection_source_reference" -or $helloInspectSource.reason -ne "retained_recovery_rollback_inspect_source_matches_sector_inspection" -or $helloInspectSource.source_event_id -ne $helloRecoveryInspectSourceAfterMaterialize.source_event_id -or $helloInspectSource.source_audit_event_id -ne $helloRecoveryInspectSourceAfterMaterialize.source_audit_event_id -or $helloInspectSource.reference_hash -ne $helloRecoveryInspectSourceAfterMaterialize.reference_hash -or $helloInspectSource.source_inspection_hash -ne $helloSectorInspection.inspection_hash -or $helloInspectSource.target_region_sector_inspection_hash -ne $helloSectorInspection.inspection_hash -or $helloInspectSource.source_sector_plan_hash -ne $helloSectorInspection.source_sector_plan_hash -or $helloInspectSource.source_target_region_write_readback_hash -ne $helloSectorInspection.source_target_region_write_readback_hash -or -not $helloInspectSource.source_available -or -not $helloInspectSource.source_matches_sector_inspection -or -not $helloInspectSource.source_event_retained -or -not $helloInspectSource.source_audit_event_retained -or $helloInspectSource.ram_audit_status -ne "retained_audit_event_verified" -or $helloInspectSource.ram_audit_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified" -or -not $helloInspectSource.ram_audit_validated -or $helloInspectSource.authorizes_rollback_apply) {
        throw "Expected rollback apply denial to consume retained recovery inspect source reference without granting rollback authority"
    }

    Send-AgentCommand -Command "recovery.rollback_inspect svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloRecoveryInspect = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    $helloRecoverySectorInspection = $helloRecoveryInspect.body.result.target_region_sector_inspection
    $helloRecoveryTargetWriteReadback = $helloRecoveryInspect.body.result.target_region_write_readback
    if ($helloRecoveryInspect.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRecoveryInspect.body.result.status -ne "target_region_sector_inspected_current_boot" -or -not $helloRecoveryInspect.body.result.read_only -or -not $helloRecoveryInspect.body.result.materialized_sector_evidence_available -or -not $helloRecoveryInspect.body.result.inspection_available) {
        throw "Expected recovery rollback inspect to expose read-only target-region sector evidence"
    }
    if (-not $helloRecoveryTargetWriteReadback -or $helloRecoveryTargetWriteReadback.dry_run_hash -ne $helloDryRunEvidence.source_target_region_write_readback_hash -or $helloRecoveryTargetWriteReadback.planned_sector_image_hash -ne $helloDryRunEvidence.planned_sector_image_hash -or $helloRecoveryTargetWriteReadback.readback_sector_image_hash -ne $helloDryRunEvidence.readback_sector_image_hash) {
        throw "Expected recovery rollback inspect to reuse retained target-region write/readback evidence"
    }
    if (-not $helloRecoverySectorInspection -or $helloRecoverySectorInspection.inspection_hash -ne $helloSectorInspection.inspection_hash -or $helloRecoverySectorInspection.source_sector_plan_hash -ne $helloSectorInspection.source_sector_plan_hash -or $helloRecoverySectorInspection.source_target_region_write_readback_hash -ne $helloSectorInspection.source_target_region_write_readback_hash -or $helloRecoverySectorInspection.audit_record_image_hash -ne $helloSectorInspection.audit_record_image_hash -or $helloRecoverySectorInspection.rollback_transaction_image_hash -ne $helloSectorInspection.rollback_transaction_image_hash) {
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
    if (-not $helloPolicyWriteAuthorityDecision -or $helloPolicyWriteAuthorityDecision.schema -ne "raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0" -or $helloPolicyWriteAuthorityDecision.id -ne "hello_rollback_durable_policy_write_authority_decision.current_boot.svc.demo.hello.v0" -or $helloPolicyWriteAuthorityDecision.status -ne "denied_missing_durable_policy_write_authority" -or $helloPolicyWriteAuthorityDecision.reason -ne "transaction_append_dry_run_verified_without_durable_policy_write_authority") {
        throw "Expected Hello rollback apply denial to expose durable policy/write-authority decision evidence"
    }
    if (-not $helloPolicyWriteAuthorityDecision.decision_hash.StartsWith("sha256:") -or $helloPolicyWriteAuthorityDecision.source_durable_append_authority_availability_dry_run_hash -ne $helloAppendAuthorityAvailabilityDryRun.dry_run_hash -or $helloPolicyWriteAuthorityDecision.source_transaction_append_dry_run_hash -ne $helloDryRunEvidence.dry_run_hash -or $helloPolicyWriteAuthorityDecision.source_target_region_sector_inspection_hash -ne $helloSectorInspection.inspection_hash -or $helloPolicyWriteAuthorityDecision.source_write_authority_availability_hash -ne $helloWriteAuthorityAvailability.availability_hash -or $helloPolicyWriteAuthorityDecision.source_audit_policy_availability_hash -ne $helloAuditPolicyAvailability.availability_hash -or $helloPolicyWriteAuthorityDecision.source_durable_append_authority_availability_hash -ne $helloAppendAuthorityAvailability.availability_hash -or $helloPolicyWriteAuthorityDecision.source_authority_denial_gate_hash -ne $helloDryRunDenialGate.gate_hash -or $helloPolicyWriteAuthorityDecision.source_transaction_append_availability_decision_hash -ne $helloDryRunDecision.decision_hash) {
        throw "Expected durable policy/write-authority decision to bind dry-run and authority availability evidence"
    }
    if ($helloPolicyWriteAuthorityDecision.target_start_lba -ne 1 -or $helloPolicyWriteAuthorityDecision.target_lba_count -ne 1 -or $helloPolicyWriteAuthorityDecision.target_byte_count -ne 512 -or -not $helloPolicyWriteAuthorityDecision.transaction_append_dry_run_verified -or -not $helloPolicyWriteAuthorityDecision.target_region_sector_inspection_verified -or -not $helloPolicyWriteAuthorityDecision.write_authority_evidence_verified -or -not $helloPolicyWriteAuthorityDecision.audit_policy_availability_evidence_verified -or -not $helloPolicyWriteAuthorityDecision.durable_append_authority_availability_evidence_verified -or -not $helloPolicyWriteAuthorityDecision.target_span_verified -or -not $helloPolicyWriteAuthorityDecision.test_infrastructure_media_write_authority_available) {
        throw "Expected durable policy/write-authority decision to verify dry-run evidence and the LBA1/512-byte target span"
    }
    if ($helloPolicyWriteAuthorityDecision.write_authority_available -or $helloPolicyWriteAuthorityDecision.durable_policy_ledger_available -or $helloPolicyWriteAuthorityDecision.durable_audit_policy_available -or $helloPolicyWriteAuthorityDecision.durable_append_authority_available -or $helloPolicyWriteAuthorityDecision.transaction_append_available -or $helloPolicyWriteAuthorityDecision.authorizes_media_write -or $helloPolicyWriteAuthorityDecision.authorizes_append -or $helloPolicyWriteAuthorityDecision.authorizes_transaction_append -or $helloPolicyWriteAuthorityDecision.writes_durable_audit_log -or $helloPolicyWriteAuthorityDecision.writes_rollback_store -or $helloPolicyWriteAuthorityDecision.appends_rollback_transaction -or $helloPolicyWriteAuthorityDecision.write_attempted -or $helloPolicyWriteAuthorityDecision.applies_rollback) {
        throw "Durable policy/write-authority decision must not authorize writes, appends, transaction append, or rollback application"
    }
    if ($helloDryRunApply.body.source_durable_policy_write_authority_decision_hash -ne $helloPolicyWriteAuthorityDecision.decision_hash -or $helloDryRunApply.body.source_recovery_rollback_inspect_source_reference_hash -ne $helloInspectSource.reference_hash -or -not $helloDryRunApply.body.retained_durable_policy_write_authority_decision_verified -or -not $helloDryRunApply.body.retained_recovery_rollback_inspect_source_reference_validated) {
        throw "Expected top-level rollback apply denial hash to bind durable policy decision and retained recovery inspect-source evidence"
    }

    $helloRecoveryInspectEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=recovery.rollback_inspect requested_capability=cap.recovery.rollback_inspect.read classification=local_only"
    Send-AgentCommand -Command $helloRecoveryInspectEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
    $helloRecoveryInspectEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $helloRecoveryInspectEnvelope.body.result.accepted -or $helloRecoveryInspectEnvelope.body.result.target_method -ne "recovery.rollback_inspect" -or -not ($helloRecoveryInspectEnvelope.body.result.allowed_target_methods -contains "recovery.rollback_inspect") -or $helloRecoveryInspectEnvelope.body.result.allowed_requested_capability -ne "cap.recovery.rollback_inspect.read" -or -not $helloRecoveryInspectEnvelope.body.result.dispatches_existing_agent_method -or $helloRecoveryInspectEnvelope.body.result.writes_durable_audit_log -or $helloRecoveryInspectEnvelope.body.result.installs_rollback_plan -or $helloRecoveryInspectEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch recovery.rollback_inspect without durable rollback authority"
    }
    $helloRecoveryInspectEnvelopeResponse = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
    if ($helloRecoveryInspectEnvelopeResponse.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRecoveryInspectEnvelopeResponse.body.result.status -ne "target_region_sector_inspected_current_boot" -or -not $helloRecoveryInspectEnvelopeResponse.body.result.read_only -or -not $helloRecoveryInspectEnvelopeResponse.body.result.inspection_available -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_media_write -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_append -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.authorizes_transaction_append -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.writes_durable_audit_log -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.writes_rollback_store -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.appends_rollback_transaction -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.applies_rollback -or $helloRecoveryInspectEnvelopeResponse.body.result.denied_surfaces.installs_rollback_state) {
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
    if ($helloDryRunHealth.body.result.service.version -ne "v2" -or -not $helloDryRunHealth.body.result.service.running) {
        throw "Expected denied rollback apply to preserve the active v2 Hello service"
    }

    Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
    $helloDryRunDrop = Get-LastAgentResponseJson -Method "service.drop"
    if ($helloDryRunDrop.body.result.service.loaded -or $helloDryRunDrop.body.result.service.running) {
        throw "Expected Hello rollback dry-run profile to drop the service after verification"
    }

    Send-AgentCommand -Command "agent audit.events 96" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
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
    Assert-LogContains -Name "hello_rollback_dry_run:audit_policy_write_decision_no_apply" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_applies_rollback": false' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_policy_decision" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_inspect_reference" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_policy_verified" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_verified": true' -TimeoutSeconds 1
    Assert-LogContains -Name "hello_rollback_dry_run:audit_apply_source_inspect_validated" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_validated": true' -TimeoutSeconds 1
