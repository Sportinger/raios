    Send-AgentCommand -Command "describe" -ExpectedMarker "RAIOS_AGENT_END system.describe"
    Assert-LogContains -Name "protocol:describe_schema" -Needle '"schema": "system.describe.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "snapshot" -ExpectedMarker "RAIOS_AGENT_END system.snapshot"
    Assert-LogContains -Name "protocol:snapshot_schema" -Needle '"schema": "system.snapshot.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:snapshot_provider_verifier_decision_schema" -Needle '"schema": "raios.provider_trust_verifier_decision.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:snapshot_provider_verifier_decision_stage" -Needle '"stage": "pin_config"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:snapshot_provider_verifier_decision_outcome" -Needle '"outcome": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:snapshot_provider_verifier_decision_reason" -Needle '"reason": "pin_config_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:snapshot_provider_pin_rotation_policy_missing" -Needle '"pin_rotation_policy": "missing_active_pin"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_trust_problem" -Needle "provider.tls_pin_config_missing" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.trust_honesty" -ExpectedMarker "RAIOS_AGENT_END provider.trust_honesty"
    Assert-LogContains -Name "protocol:provider_trust_honesty_schema" -Needle '"schema": "raios.provider_trust_honesty.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_trust_honesty_decision_id" -Needle '"decision_id": "scoped_provider_trust_honesty.current_boot.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_trust_honesty_reason" -Needle '"reason": "trust_state_not_pin_verified"' -TimeoutSeconds 1
    $providerTrustHonesty = Get-LastAgentResponseJson -Method "provider.trust_honesty"
    $trustHonesty = $providerTrustHonesty.body.result
    $providerTrustHonestyUnpinnedDenial = (
        $trustHonesty.schema -eq "raios.provider_trust_honesty.v0" -and
        $trustHonesty.decision_schema -eq "raios.scoped_provider_trust_honesty_decision.v0" -and
        $trustHonesty.decision_id -eq "scoped_provider_trust_honesty.current_boot.v0" -and
        $trustHonesty.provider_id -eq "openai" -and
        $trustHonesty.trust_state -eq "pin_config_missing" -and
        $trustHonesty.chain_policy -eq "pin_only_no_webpki_chain_validation" -and
        $trustHonesty.time_policy -eq "not_validated_stage0" -and
        $trustHonesty.development_bypass -eq $false -and
        $trustHonesty.performed -eq $false -and
        $trustHonesty.status -eq "denied" -and
        $trustHonesty.reason -eq "trust_state_not_pin_verified" -and
        $trustHonesty.honest -eq $false -and
        $trustHonesty.chain_validated -eq $false -and
        $trustHonesty.time_validated -eq $false
    )
    Add-Predicate -Name "protocol:provider_trust_honesty_unpinned_denial" -Expected "live unpinned boot is honestly denied by M10A-1 trust_state_not_pin_verified" -Passed $providerTrustHonestyUnpinnedDenial -Actual $(if ($providerTrustHonestyUnpinnedDenial) { "pin_config_missing denied" } else { ($trustHonesty | ConvertTo-Json -Compress -Depth 5) })
    if (-not $providerTrustHonestyUnpinnedDenial) {
        throw "Expected provider.trust_honesty to report the live unpinned boot as an honest M10A-1 denial"
    }
    $providerTrustHonestyGrantsNothing = (
        $trustHonesty.authorizes_provider_request -eq $false -and
        $trustHonesty.authorizes_provider_export -eq $false -and
        $trustHonesty.owner_sealed -eq $false -and
        $trustHonesty.trust_tier -eq "dev_key_not_owner_sealed" -and
        $trustHonesty.durable_write -eq $false -and
        $trustHonesty.capability_granted -eq $false -and
        $trustHonesty.provider_write -eq "not_attempted" -and
        $trustHonesty.transmission -eq $false
    )
    Add-Predicate -Name "protocol:provider_trust_honesty_grants_nothing" -Expected "provider.trust_honesty authorizes no request/export/write/transmission" -Passed $providerTrustHonestyGrantsNothing -Actual $(if ($providerTrustHonestyGrantsNothing) { "grants_nothing" } else { ($trustHonesty | ConvertTo-Json -Compress -Depth 5) })
    if (-not $providerTrustHonestyGrantsNothing) {
        throw "Expected provider.trust_honesty to grant nothing"
    }

    Send-AgentCommand -Command "caps" -ExpectedMarker "RAIOS_AGENT_END system.capabilities"
    Assert-LogContains -Name "protocol:capabilities_schema" -Needle '"schema": "system.capabilities.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_capability" -Needle '"id": "cap.memory.recent_events.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_capability" -Needle '"id": "cap.audit.events.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_read_capability" -Needle '"id": "cap.provider.context_export.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_read_capability" -Needle '"id": "cap.provider.context_injection.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_read_capability" -Needle '"id": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_capability_listed" -Needle '"id": "cap.provider.context_export"' -TimeoutSeconds 1

    Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory"
    Assert-LogContains -Name "protocol:service_inventory_schema" -Needle '"schema": "service.inventory.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:openai_service_listed" -Needle "svc.provider.openai_direct" -TimeoutSeconds 1

    Send-AgentCommand -Command "problems" -ExpectedMarker "RAIOS_AGENT_END problem.list"
    Assert-LogContains -Name "protocol:problem_list_schema" -Needle '"schema": "problem.list.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.profile" -ExpectedMarker "RAIOS_AGENT_END memory.profile"
    Assert-LogContains -Name "protocol:memory_profile_schema" -Needle '"schema": "memory.profile.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_provider_minimal" -Needle '"provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_provider_local_projection" -Needle '"local_projection": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_diagnostic" -Needle '"diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_planning" -Needle '"planning"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.context diagnostic" -ExpectedMarker "RAIOS_AGENT_END memory.context"
    Assert-LogContains -Name "protocol:memory_context_schema" -Needle '"schema": "raios.agent_context.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_profile" -Needle '"profile": "diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_event_id" -Needle '"context_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_audit_event_id" -Needle '"audit_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_snapshot_source" -Needle "system.snapshot.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_service_source" -Needle "service.inventory.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_problem_source" -Needle "problem.list.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_source" -Needle '"raios.recovery_lifeline_status_projection.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_record" -Needle '"id": "recovery.lifeline.status.current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_missing" -Needle '"status": "unavailable_missing_retained_result"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_reason" -Needle '"reason": "recovery_lifeline_status_execution_result_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_unverified" -Needle '"source_retained_result_verified": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_no_execution" -Needle '"command_execution_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_recovery_status_no_provider_export" -Needle '"provider_export": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_trust_problem" -Needle "provider.tls_pin_config_missing" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.context provider_minimal" -ExpectedMarker "RAIOS_AGENT_END memory.context"
    Assert-LogContains -Name "protocol:memory_context_provider_profile" -Needle '"profile": "provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_schema" -Needle '"schema": "raios.provider_context_projection.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_verifier_decision" -Needle '"verifier_decision": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_mode" -Needle '"mode": "local_read_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_present" -Needle '"redaction_projection": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_classification_default" -Needle '"classification_default": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_event" -Needle '"local_projection_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_can_export" -Needle '"can_export": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_trust_gate" -Needle '"reason": "provider_trust_not_positive"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_audit_gate" -Needle '"reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_evidence" -Needle '"packet_evidence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_canonicalization" -Needle '"canonicalization": "raios.provider_minimal.packet.canonical.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_hash" -Needle '"projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_redaction_policy_hash" -Needle '"redaction_policy_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_field_classification_hash" -Needle '"field_classification_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_token_budget_hash" -Needle '"token_budget_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_status" -Needle '"field": "current.status.*"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_key_state" -Needle '"field": "current.provider.api_key_state"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_verifier_decision" -Needle '"field": "current.provider.verifier_decision.*"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_pin_rotation" -Needle '"field": "current.provider.pin_rotation_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_raw_snapshot" -Needle '"field": "system.snapshot.raw"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_secret_prompt" -Needle '"field": "provider.direct_last_prompt"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_recovery_status_fact" -Needle '"field": "current.recovery_lifeline_status"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_recovery_status_locator" -Needle '"field": "recovery.lifeline.status.current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_purpose" -Needle '"purpose": "current_boot_provider_context"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_snapshot_projection_record" -Needle "snapshot.current.provider_minimal" -TimeoutSeconds 1
    $providerMinimalContext = Get-LastAgentResponseJson -Method "memory.context"
    $providerMinimalProjection = $providerMinimalContext.body.result.provider_projection
    $providerMinimalOmittedFields = @($providerMinimalProjection.omitted_fields | ForEach-Object { $_.field })
    $providerMinimalPacketIncludedCurrent = @($providerMinimalProjection.packet.included.current)
    $providerMinimalPacketOmittedFields = @($providerMinimalProjection.packet.omitted | ForEach-Object { $_.field })
    $providerMinimalOmittedRecoveryStatus = (
        $providerMinimalOmittedFields -contains "current.recovery_lifeline_status" -and
        $providerMinimalOmittedFields -contains "recovery.lifeline.status.current_boot" -and
        $providerMinimalPacketOmittedFields -contains "current.recovery_lifeline_status" -and
        $providerMinimalPacketOmittedFields -contains "recovery.lifeline.status.current_boot" -and
        -not ($providerMinimalPacketIncludedCurrent -contains "recovery.lifeline.status.current_boot")
    )
    Add-Predicate -Name "protocol:memory_context_provider_recovery_status_omitted_from_packet" -Expected "provider_minimal_omits_recovery_status_local_only_fact" -Passed $providerMinimalOmittedRecoveryStatus -Actual $(if ($providerMinimalOmittedRecoveryStatus) { "omitted" } else { "included_or_missing_omission" })
    if (-not $providerMinimalOmittedRecoveryStatus) {
        throw "Expected provider_minimal packet to omit the local-only recovery lifeline status fact"
    }

    Send-AgentCommand -Command "agent provider.context_export provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_export"
    Assert-LogContains -Name "protocol:provider_context_export_schema" -Needle '"schema": "raios.provider_context_export.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_profile" -Needle '"profile": "provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_capability" -Needle '"requested_capability": "cap.provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_trust_block" -Needle '"reason": "provider_trust_not_positive"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_binding_present" -Needle '"packet_evidence_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_exported_fields_binding_present" -Needle '"exported_field_list_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_omitted_fields_binding_present" -Needle '"omitted_field_list_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_canonicalization" -Needle '"packet_canonicalization": "raios.provider_minimal.packet.canonical.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_hash" -Needle '"projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_redaction_policy_hash" -Needle '"redaction_policy_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_field_classification_hash" -Needle '"field_classification_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_token_budget_hash" -Needle '"token_budget_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_missing" -Needle '"provider_request_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_state" -Needle '"provider_request_binding_denial": "present_denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_missing" -Needle '"provider_export_audit_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_state" -Needle '"provider_export_denial_audit": "present_denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_schema" -Needle '"schema": "raios.provider_request_binding_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_id" -Needle '"id": "provider_request_binding_denial.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_attempt_id" -Needle '"attempted_request_id": "provider_request_attempt.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_status" -Needle '"status": "denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_denial_not_gate" -Needle '"satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_schema" -Needle '"schema": "raios.provider_context_export_denial_audit.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_id" -Needle '"id": "provider_context_export_denial_audit.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_status" -Needle '"status": "denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_not_gate" -Needle '"satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_write_path_disabled" -Needle '"reason": "automatic_context_injection_disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_required" -Needle '"reason": "provider_request_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_required" -Needle '"reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_evidence" -Needle '"provider_request_binding_denial_id": "provider_request_binding_denial.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_event_evidence" -Needle '"provider_request_binding_denial_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_attempt_evidence" -Needle '"provider_request_attempt_id": "provider_request_attempt.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_status_evidence" -Needle '"export_audit_binding_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_evidence" -Needle '"export_denial_audit_id": "provider_context_export_denial_audit.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_event_evidence" -Needle '"export_denial_audit_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_not_gate" -Needle '"export_denial_audit_satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_not_binding" -Needle '"denial_event_is_export_binding": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_projection_locator" -Needle '"local_projection_locator": "snapshot.current.provider_minimal"' -TimeoutSeconds 1
    $providerContextExport = Get-LastAgentResponseJson -Method "provider.context_export"
    $providerContextExportOmission = $providerContextExport.body.evidence.recovery_status_omission
    $providerContextExportOmissionOk = (
        $providerContextExportOmission.schema -eq "raios.provider_minimal.local_only_omission.v0" -and
        $providerContextExportOmission.status -eq "omitted_from_provider_context" -and
        $providerContextExportOmission.fact_field -eq "current.recovery_lifeline_status" -and
        $providerContextExportOmission.locator -eq "recovery.lifeline.status.current_boot" -and
        $providerContextExportOmission.classification -eq "local_only" -and
        $providerContextExportOmission.provider_export -eq $false -and
        $providerContextExportOmission.context_attached_to_provider_body -eq $false -and
        $providerContextExportOmission.provider_write -eq "not_attempted"
    )
    Add-Predicate -Name "protocol:provider_context_export_recovery_status_omission_evidence" -Expected "provider_context_export_recovery_status_omitted" -Passed $providerContextExportOmissionOk -Actual $(if ($providerContextExportOmissionOk) { "omitted" } else { "missing_or_exportable" })
    if (-not $providerContextExportOmissionOk) {
        throw "Expected provider.context_export to expose recovery status omission evidence without export"
    }
    Assert-LogDoesNotContain -Name "protocol:provider_context_export_did_not_fake_request_envelope" -Needle "raios.provider_request_envelope.v0"

    Send-AgentCommand -Command "agent provider.context_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_gate"
    Assert-LogContains -Name "protocol:provider_context_gate_schema" -Needle '"schema": "raios.provider_context_export_gate_state.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_binding_missing" -Needle '"binding_validation_reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_request_binding_missing" -Needle '"provider_request_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_audit_binding_missing" -Needle '"provider_export_audit_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_current_boot_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1
    $providerContextGate = Get-LastAgentResponseJson -Method "provider.context_gate"
    $providerContextGateOmission = $providerContextGate.body.result.evidence.recovery_status_omission
    $providerContextGateOmissionOk = (
        $providerContextGateOmission.schema -eq "raios.provider_minimal.local_only_omission.v0" -and
        $providerContextGateOmission.status -eq "omitted_from_provider_context" -and
        $providerContextGateOmission.fact_field -eq "current.recovery_lifeline_status" -and
        $providerContextGateOmission.locator -eq "recovery.lifeline.status.current_boot" -and
        $providerContextGateOmission.classification -eq "local_only" -and
        $providerContextGateOmission.provider_export -eq $false -and
        $providerContextGateOmission.context_attached_to_provider_body -eq $false -and
        $providerContextGateOmission.provider_write -eq "not_attempted"
    )
    Add-Predicate -Name "protocol:provider_context_gate_recovery_status_omission_evidence" -Expected "provider_context_gate_recovery_status_omitted" -Passed $providerContextGateOmissionOk -Actual $(if ($providerContextGateOmissionOk) { "omitted" } else { "missing_or_exportable" })
    if (-not $providerContextGateOmissionOk) {
        throw "Expected provider.context_gate to expose recovery status omission evidence without export"
    }
