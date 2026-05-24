    Send-AgentCommand -Command "recovery.load_artifact" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact"
    $recoveryLoadResponse = Get-LastAgentResponseJson -Method "recovery.load_artifact"
    Assert-LogContains -Name "policy:recovery_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_schema" -Needle '"schema": "raios.recovery_artifact_load_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_separate_capability" -Needle '"separate_from": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_normal_path_not_used" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_normal_cap_not_used" -Needle '"normal_module_capability_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_denial_evidence_schema" -Needle '"schema": "raios.recovery_artifact_load_denial_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_status" -Needle '"status": "denied_missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_identity_missing" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_trust_missing" -Needle '"recovery_artifact_trust": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_vm_test_missing" -Needle '"recovery_vm_test": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_approval_missing" -Needle '"recovery_local_approval": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_loader_missing" -Needle '"recovery_loader": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_rollback_missing" -Needle '"recovery_rollback_evidence": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_identity_schema" -Needle '"schema": "raios.recovery_artifact_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_trust_schema" -Needle '"schema": "raios.recovery_artifact_trust.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_vm_test_schema" -Needle '"schema": "raios.recovery_artifact_vm_test.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_approval_schema" -Needle '"schema": "raios.recovery_artifact_local_approval.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_loader_schema" -Needle '"schema": "raios.recovery_artifact_loader.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_rollback_schema" -Needle '"schema": "raios.recovery_artifact_rollback_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_append_payload_not_authority" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    $recoveryLoadEventId = [string]$recoveryLoadResponse.body.event_id
    $recoveryLoadEventIdPresent = $recoveryLoadEventId.StartsWith("event.current_boot.")
    Add-Predicate -Name "policy:recovery_load_current_boot_event_id" -Expected "event.current_boot.*" -Passed $recoveryLoadEventIdPresent -Actual $recoveryLoadEventId
    if (-not $recoveryLoadEventIdPresent) {
        throw "Expected current-boot event id for recovery.load_artifact, got $recoveryLoadEventId"
    }

    Send-AgentCommand -Command "agent recovery.identity_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_identity_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_normal_load"; Needle = '"loads_normal_module": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_identity_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_identity_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryArtifactHash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    $recoveryIdentityCanonical = @(
        "canonicalization=raios.recovery_artifact_identity.canonical.v0",
        "schema=raios.recovery_artifact_identity.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "artifact_sha256=$recoveryArtifactHash",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryIdentityReferenceHash = Get-TextSha256 -Text $recoveryIdentityCanonical
    $recoveryIdentityCommand = "agent recovery.identity_diagnostic $recoveryIdentityReferenceHash $recoveryArtifactHash"

    Send-AgentCommand -Command $recoveryIdentityCommand -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "still_denied"; Needle = '"can_move_beyond_denial": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryIdentityResponse = Get-LastAgentResponseJson -Method "recovery.identity_diagnostic"
    $recoveryIdentityEventId = [string]$recoveryIdentityResponse.body.result.retained_recovery_artifact_identity_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_identity_retained_reference_event_id_captured" -Value $recoveryIdentityEventId

    Send-AgentCommand -Command "agent recovery.identity_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_identity_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_identity_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 6' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_identity_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "wrong_schema_case"; Needle = '"case": "wrong_schema_identity_reference"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_identity_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "identity_reference_hash_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.trust_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_trust_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_trust_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_trust_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryTrustHash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    $recoveryTrustCanonical = @(
        "canonicalization=raios.recovery_artifact_trust.canonical.v0",
        "schema=raios.recovery_artifact_trust.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryTrustReferenceHash = Get-TextSha256 -Text $recoveryTrustCanonical
    $recoveryTrustCommand = "agent recovery.trust_diagnostic $recoveryTrustReferenceHash $recoveryIdentityEventId $recoveryIdentityReferenceHash $recoveryArtifactHash $recoveryTrustHash"

    Send-AgentCommand -Command $recoveryTrustCommand -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_hash_echo"; Needle = "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "trust_material_hash_echo"; Needle = "`"trust_hash`": `"sha256:$recoveryTrustHash`"" },
        @{ Suffix = "still_denied"; Needle = '"can_move_beyond_denial": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryTrustResponse = Get-LastAgentResponseJson -Method "recovery.trust_diagnostic"
    $recoveryTrustEventId = [string]$recoveryTrustResponse.body.result.retained_recovery_artifact_trust_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_trust_retained_reference_event_id_captured" -Value $recoveryTrustEventId

    Send-AgentCommand -Command "agent recovery.trust_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_trust_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_trust_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 8' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_trust_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "identity_not_current_case"; Needle = '"case": "retained_identity_event_not_current_boot"' },
        @{ Suffix = "identity_missing_case"; Needle = '"case": "retained_identity_missing"' },
        @{ Suffix = "identity_schema_case"; Needle = '"case": "retained_identity_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_identity_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "trust_reference_hash_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.vm_test_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_vm_test_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_vm_test_json"; Needle = '"accepts_vm_test_json": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_vm_test_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_vm_test_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryVmTestHash = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    $recoveryVmTestCanonical = @(
        "canonicalization=raios.recovery_artifact_vm_test.canonical.v0",
        "schema=raios.recovery_artifact_vm_test.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "accepts_vm_test_json=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryVmTestReferenceHash = Get-TextSha256 -Text $recoveryVmTestCanonical
    $recoveryVmTestCommand = "agent recovery.vm_test_diagnostic $recoveryVmTestReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash"

    Send-AgentCommand -Command $recoveryVmTestCommand -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_event"; Needle = "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" },
        @{ Suffix = "vm_test_hash_echo"; Needle = "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" },
        @{ Suffix = "trust_hash_echo"; Needle = "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "vm_test_material_hash_echo"; Needle = "`"vm_test_hash`": `"sha256:$recoveryVmTestHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryVmTestResponse = Get-LastAgentResponseJson -Method "recovery.vm_test_diagnostic"
    $recoveryVmTestEventId = [string]$recoveryVmTestResponse.body.result.retained_recovery_artifact_vm_test_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_vm_test_retained_reference_event_id_captured" -Value $recoveryVmTestEventId

    Send-AgentCommand -Command "agent recovery.vm_test_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_vm_test_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_vm_test_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_vm_test_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "trust_not_current_case"; Needle = '"case": "retained_trust_event_not_current_boot"' },
        @{ Suffix = "identity_missing_case"; Needle = '"case": "retained_identity_missing"' },
        @{ Suffix = "trust_schema_case"; Needle = '"case": "retained_trust_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_trust_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "vm_test_reference_hash_mismatch"' },
        @{ Suffix = "binding_mismatch_case"; Needle = '"case": "trust_binding_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.local_approval_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_local_approval_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_approval_text"; Needle = '"accepts_local_approval_text": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_local_approval_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_local_approval_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLocalApprovalHash = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
    $recoveryLocalApprovalCanonical = @(
        "canonicalization=raios.recovery_artifact_local_approval.canonical.v0",
        "schema=raios.recovery_artifact_local_approval.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "accepts_local_approval_text=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLocalApprovalReferenceHash = Get-TextSha256 -Text $recoveryLocalApprovalCanonical
    $recoveryLocalApprovalCommand = "agent recovery.local_approval_diagnostic $recoveryLocalApprovalReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash"

    Send-AgentCommand -Command $recoveryLocalApprovalCommand -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_local_approval_reference_valid_but_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_event"; Needle = "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" },
        @{ Suffix = "vm_test_event"; Needle = "`"retained_recovery_artifact_vm_test_event_id`": `"$recoveryVmTestEventId`"" },
        @{ Suffix = "approval_hash_echo"; Needle = "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" },
        @{ Suffix = "vm_test_hash_echo"; Needle = "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" },
        @{ Suffix = "local_approval_material_hash_echo"; Needle = "`"local_approval_hash`": `"sha256:$recoveryLocalApprovalHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLocalApprovalResponse = Get-LastAgentResponseJson -Method "recovery.local_approval_diagnostic"
    $recoveryLocalApprovalEventId = [string]$recoveryLocalApprovalResponse.body.result.retained_recovery_artifact_local_approval_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_local_approval_retained_reference_event_id_captured" -Value $recoveryLocalApprovalEventId

    Send-AgentCommand -Command "agent recovery.local_approval_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_local_approval_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_local_approval_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 11' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_local_approval_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "vm_not_current_case"; Needle = '"case": "retained_vm_test_event_not_current_boot"' },
        @{ Suffix = "vm_missing_case"; Needle = '"case": "retained_vm_test_missing"' },
        @{ Suffix = "vm_schema_case"; Needle = '"case": "retained_vm_test_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_vm_test_reference_record"' },
        @{ Suffix = "approval_mismatch_case"; Needle = '"case": "local_approval_reference_hash_mismatch"' },
        @{ Suffix = "vm_hash_mismatch_case"; Needle = '"case": "vm_test_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_local_approval_reference_valid_but_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.loader_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_loader_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_loader_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_loader_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoaderHash = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    $recoveryLoaderCanonical = @(
        "canonicalization=raios.recovery_artifact_loader.canonical.v0",
        "schema=raios.recovery_artifact_loader.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "retained_recovery_artifact_local_approval_event_id=$recoveryLocalApprovalEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "local_approval_reference_sha256=$recoveryLocalApprovalReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "loader_sha256=$recoveryLoaderHash",
        "accepts_loader_descriptor=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_loader=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLoaderReferenceHash = Get-TextSha256 -Text $recoveryLoaderCanonical
    $recoveryLoaderCommand = "agent recovery.loader_diagnostic $recoveryLoaderReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryLocalApprovalEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryLocalApprovalReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash $recoveryLoaderHash"

    Send-AgentCommand -Command $recoveryLoaderCommand -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "approval_event"; Needle = "`"retained_recovery_artifact_local_approval_event_id`": `"$recoveryLocalApprovalEventId`"" },
        @{ Suffix = "loader_ref_hash"; Needle = "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" },
        @{ Suffix = "approval_ref_hash"; Needle = "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" },
        @{ Suffix = "loader_hash"; Needle = "`"loader_hash`": `"sha256:$recoveryLoaderHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoaderResponse = Get-LastAgentResponseJson -Method "recovery.loader_diagnostic"
    $recoveryLoaderEventId = [string]$recoveryLoaderResponse.body.result.retained_recovery_artifact_loader_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_loader_retained_reference_event_id_captured" -Value $recoveryLoaderEventId

    Send-AgentCommand -Command "agent recovery.loader_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_loader_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_loader_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_loader_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "approval_not_current_case"; Needle = '"case": "retained_local_approval_event_not_current_boot"' },
        @{ Suffix = "approval_missing_case"; Needle = '"case": "retained_local_approval_missing"' },
        @{ Suffix = "approval_schema_case"; Needle = '"case": "retained_local_approval_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_local_approval_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "loader_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_evidence_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_rollback_evidence_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_rollback_json"; Needle = '"accepts_rollback_evidence_json": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_rollback_evidence_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_rollback_evidence_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackEvidenceHash = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    $recoveryRollbackEvidenceCanonical = @(
        "canonicalization=raios.recovery_artifact_rollback_evidence.canonical.v0",
        "schema=raios.recovery_artifact_rollback_evidence.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "retained_recovery_artifact_local_approval_event_id=$recoveryLocalApprovalEventId",
        "retained_recovery_artifact_loader_event_id=$recoveryLoaderEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "local_approval_reference_sha256=$recoveryLocalApprovalReferenceHash",
        "loader_reference_sha256=$recoveryLoaderReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "loader_sha256=$recoveryLoaderHash",
        "rollback_evidence_sha256=$recoveryRollbackEvidenceHash",
        "accepts_rollback_evidence_json=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackEvidenceReferenceHash = Get-TextSha256 -Text $recoveryRollbackEvidenceCanonical
    $recoveryRollbackEvidenceCommand = "agent recovery.rollback_evidence_diagnostic $recoveryRollbackEvidenceReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryLocalApprovalEventId $recoveryLoaderEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryLocalApprovalReferenceHash $recoveryLoaderReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash $recoveryLoaderHash $recoveryRollbackEvidenceHash"

    Send-AgentCommand -Command $recoveryRollbackEvidenceCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "loader_event"; Needle = "`"retained_recovery_artifact_loader_event_id`": `"$recoveryLoaderEventId`"" },
        @{ Suffix = "rollback_ref_hash"; Needle = "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" },
        @{ Suffix = "loader_ref_hash"; Needle = "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" },
        @{ Suffix = "rollback_hash"; Needle = "`"rollback_evidence_hash`": `"sha256:$recoveryRollbackEvidenceHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackEvidenceResponse = Get-LastAgentResponseJson -Method "recovery.rollback_evidence_diagnostic"
    $recoveryRollbackEvidenceEventId = [string]$recoveryRollbackEvidenceResponse.body.result.retained_recovery_artifact_rollback_evidence_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_evidence_retained_reference_event_id_captured" -Value $recoveryRollbackEvidenceEventId

    Send-AgentCommand -Command "agent recovery.rollback_evidence_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_rollback_evidence_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_evidence_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_rollback_evidence_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "loader_not_current_case"; Needle = '"case": "retained_loader_event_not_current_boot"' },
        @{ Suffix = "loader_missing_case"; Needle = '"case": "retained_loader_missing"' },
        @{ Suffix = "loader_schema_case"; Needle = '"case": "retained_loader_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_loader_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "rollback_evidence_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
