    Send-AgentCommand -Command "agent module.manifest_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic"
    Assert-LogContains -Name "protocol:module_manifest_diag_schema" -Needle '"schema": "raios.module_manifest_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent_reason" -Needle '"validation_reason": "module_manifest_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_manifest_missing" -Needle '"module_manifest": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
    $moduleManifestReferenceCanonical = @(
        "canonicalization=raios.module_manifest_reference.canonical.v0",
        "schema=raios.module_manifest_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "manifest_schema=raios.module_manifest.v0",
        "manifest_sha256=$moduleGrantManifestHash",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleManifestReferenceHash = Get-TextSha256 -Text $moduleManifestReferenceCanonical
    $moduleManifestCommand = "agent module.manifest_diagnostic $moduleManifestReferenceHash $moduleGrantManifestHash"

    Send-AgentCommand -Command $moduleManifestCommand -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_manifest_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "module_manifest_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "manifest_present"; Needle = '"manifest_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" },
        @{ Suffix = "manifest_hash_echo"; Needle = "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
    $moduleManifestRetainedReferenceEventId = [string]$moduleManifestResponse.body.result.retained_manifest_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_manifest_retained_reference_event_id_captured" -Value $moduleManifestRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.manifest_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_manifest_selftest_schema" -Needle '"schema": "raios.module_manifest_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_records" -Needle '"creates_retained_manifest_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_valid_case" -Needle '"case": "accepted_current_boot_manifest_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_mismatch_case" -Needle '"case": "mismatched_manifest_hash_reference"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.grant_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic"
    Assert-LogContains -Name "protocol:module_grant_diag_schema" -Needle '"schema": "raios.module_computed_grant_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent_reason" -Needle '"validation_reason": "computed_capability_grant_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_computed_missing" -Needle '"computed_capability_grant": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_load_grant" -Needle '"grants_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_guest_load" -Needle '"authorizes_guest_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_loader_unavailable" -Needle '"loader", "state": "unavailable"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_slot_unallocated" -Needle '"service_slot", "state": "unallocated"' -TimeoutSeconds 1

    $moduleGrantArtifactHash = "2222222222222222222222222222222222222222222222222222222222222222"
    $moduleGrantReportHash = "3333333333333333333333333333333333333333333333333333333333333333"
    $moduleGrantAttestationHash = "4444444444444444444444444444444444444444444444444444444444444444"
    $moduleGrantCanonical = @(
        "canonicalization=raios.computed_capability_grant.canonical.v0",
        "schema=raios.computed_capability_grant.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleGrantHash = Get-TextSha256 -Text $moduleGrantCanonical
    $moduleGrantCommand = "agent module.grant_diagnostic $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_grant_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_retained"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "candidate_present"; Needle = '"computed_candidate_present": true' },
        @{ Suffix = "valid_still_no_capability"; Needle = '"grants_capability": false' },
        @{ Suffix = "valid_still_no_load"; Needle = '"can_load_now": false' },
        @{ Suffix = "valid_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" }
    )

    $moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $moduleAuditRetainedReferenceEventId = [string]$moduleGrantResponse.body.result.retained_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_grant_retained_reference_event_id_captured" -Value $moduleAuditRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.artifact_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic"
    Assert-LogContains -Name "protocol:module_artifact_diag_schema" -Needle '"schema": "raios.module_candidate_artifact_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent_reason" -Needle '"validation_reason": "candidate_artifact_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_artifact_missing" -Needle '"candidate_artifact": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleArtifactReferenceCanonical = @(
        "canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
        "schema=raios.module_candidate_artifact_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleArtifactReferenceHash = Get-TextSha256 -Text $moduleArtifactReferenceCanonical
    $moduleArtifactCommand = "agent module.artifact_diagnostic $moduleArtifactReferenceHash $moduleManifestRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleGrantManifestHash $moduleGrantHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleArtifactCommand -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_artifact_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "candidate_artifact_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"candidate_artifact_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
    $moduleArtifactRetainedReferenceEventId = [string]$moduleArtifactResponse.body.result.retained_candidate_artifact_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_artifact_retained_reference_event_id_captured" -Value $moduleArtifactRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.artifact_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_artifact_selftest_schema" -Needle '"schema": "raios.module_candidate_artifact_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_records" -Needle '"creates_retained_candidate_artifact_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_valid_case" -Needle '"case": "accepted_current_boot_artifact_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_mismatch_case" -Needle '"case": "mismatched_artifact_reference_hash"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.vm_report_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic"
    Assert-LogContains -Name "protocol:module_vm_report_diag_schema" -Needle '"schema": "raios.module_vm_test_report_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent_reason" -Needle '"validation_reason": "vm_test_report_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_report_missing" -Needle '"vm_test_report": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleVmReportReferenceCanonical = @(
        "canonicalization=raios.module_vm_test_report_reference.canonical.v0",
        "schema=raios.module_vm_test_report_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_vm_report_json=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleVmReportReferenceHash = Get-TextSha256 -Text $moduleVmReportReferenceCanonical
    $moduleVmReportCommand = "agent module.vm_report_diagnostic $moduleVmReportReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleVmReportCommand -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_vm_report_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "vm_test_report_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"vm_test_report_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" },
        @{ Suffix = "report_hash_echo"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "artifact_ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
    $moduleVmReportRetainedReferenceEventId = [string]$moduleVmReportResponse.body.result.retained_vm_test_report_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_vm_report_retained_reference_event_id_captured" -Value $moduleVmReportRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.vm_report_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_vm_report_selftest_schema" -Needle '"schema": "raios.module_vm_test_report_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_records" -Needle '"creates_retained_vm_test_report_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_valid_case" -Needle '"case": "accepted_current_boot_report_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_mismatch_case" -Needle '"case": "vm_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_grant_mismatch_case" -Needle '"case": "computed_grant_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.attestation_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic"
    Assert-LogContains -Name "protocol:module_attestation_diag_schema" -Needle '"schema": "raios.module_local_attestation_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent_reason" -Needle '"validation_reason": "local_attestation_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_attestation_missing" -Needle '"local_attestation": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAttestationReferenceCanonical = @(
        "canonicalization=raios.module_local_attestation_reference.canonical.v0",
        "schema=raios.module_local_attestation_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_vm_report_reference_event_id=$moduleVmReportRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "vm_test_report_reference_sha256=$moduleVmReportReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_local_attestation_json=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleAttestationReferenceHash = Get-TextSha256 -Text $moduleAttestationReferenceCanonical
    $moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_attestation_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "local_attestation_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"local_attestation_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" },
        @{ Suffix = "attestation_hash_echo"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "vm_ref_hash_echo"; Needle = "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $moduleAttestationRetainedReferenceEventId = [string]$moduleAttestationResponse.body.result.retained_local_attestation_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_attestation_retained_reference_event_id_captured" -Value $moduleAttestationRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.attestation_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_attestation_selftest_schema" -Needle '"schema": "raios.module_local_attestation_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_records" -Needle '"creates_retained_local_attestation_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_count" -Needle '"case_count": 9' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_valid_case" -Needle '"case": "accepted_current_boot_attestation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_mismatch_case" -Needle '"case": "local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.approval_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic"
    Assert-LogContains -Name "protocol:module_approval_diag_schema" -Needle '"schema": "raios.module_local_approval_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent_reason" -Needle '"validation_reason": "local_approval_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_approval_missing" -Needle '"local_approval": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAuditLocalApprovalHash = "6666666666666666666666666666666666666666666666666666666666666666"
    $moduleApprovalReferenceCanonical = @(
        "canonicalization=raios.module_local_approval_reference.canonical.v0",
        "schema=raios.module_local_approval_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_vm_report_reference_event_id=$moduleVmReportRetainedReferenceEventId",
        "retained_local_attestation_reference_event_id=$moduleAttestationRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "vm_test_report_reference_sha256=$moduleVmReportReferenceHash",
        "local_attestation_reference_sha256=$moduleAttestationReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "accepts_local_approval_text=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleApprovalReferenceHash = Get-TextSha256 -Text $moduleApprovalReferenceCanonical
    $moduleApprovalCommand = "agent module.approval_diagnostic $moduleApprovalReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAttestationRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleAttestationReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash"

    Send-AgentCommand -Command $moduleApprovalCommand -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_approval_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "local_approval_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"local_approval_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" },
        @{ Suffix = "approval_hash_echo"; Needle = "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" },
        @{ Suffix = "attestation_ref_hash_echo"; Needle = "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleApprovalResponse = Get-LastAgentResponseJson -Method "module.approval_diagnostic"
    $moduleApprovalRetainedReferenceEventId = [string]$moduleApprovalResponse.body.result.retained_local_approval_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_approval_retained_reference_event_id_captured" -Value $moduleApprovalRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.approval_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_approval_selftest_schema" -Needle '"schema": "raios.module_local_approval_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_records" -Needle '"creates_retained_local_approval_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_valid_case" -Needle '"case": "accepted_current_boot_approval_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_mismatch_case" -Needle '"case": "local_approval_reference_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.pre_audit"
    $modulePreAuditLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $moduleAuditDenialEventId = [string]$modulePreAuditLoadResponse.body.event_id
    Assert-CurrentBootEventId -Name "protocol:module_audit_denial_event_id_captured" -Value $moduleAuditDenialEventId
    Assert-LogContains -Name "policy:module_pre_audit_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_artifact_retained" -Needle '"candidate_artifact": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_vm_report_retained" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_attestation_retained" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_approval_retained" -Needle '"local_approval": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_audit_missing" -Needle '"durable_audit_record": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_rollback_missing" -Needle '"rollback_plan": "missing"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic"
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_schema" -Needle '"schema": "raios.module_audit_rollback_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent_reason" -Needle '"validation_reason": "audit_rollback_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAuditPreInventoryHash = "7777777777777777777777777777777777777777777777777777777777777777"
    $moduleAuditCleanupHash = "8888888888888888888888888888888888888888888888888888888888888888"
    $moduleAuditRamOnlyServiceSlotId = "ram_only:svc.test.0001"
    $moduleRollbackCanonical = @(
        "canonicalization=raios.rollback_plan.canonical.v0",
        "schema=raios.rollback_plan.v0",
        "load_mode=ram_only",
        "scope=current_boot",
        "artifact_sha256=$moduleGrantArtifactHash",
        "pre_load_service_inventory_sha256=$moduleAuditPreInventoryHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "cleanup_actions_sha256=$moduleAuditCleanupHash",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleRollbackHash = Get-TextSha256 -Text $moduleRollbackCanonical

    $moduleWrongAuditDenialEventId = $moduleAuditRetainedReferenceEventId
    $moduleWrongAuditCanonical = @(
        "canonicalization=raios.audit_record.canonical.v0",
        "schema=raios.audit_record.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "denial_event_id=$moduleWrongAuditDenialEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleWrongAuditHash = Get-TextSha256 -Text $moduleWrongAuditCanonical
    $moduleWrongAuditCommand = "agent module.audit_rollback_diagnostic $moduleWrongAuditHash $moduleRollbackHash $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash $moduleAuditPreInventoryHash $moduleAuditCleanupHash $moduleWrongAuditDenialEventId $moduleAuditRetainedReferenceEventId $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleWrongAuditCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic" -Name "command:module.audit_rollback_diagnostic.wrong_denial"
    Assert-LogContains -Name "protocol:module_wrong_audit_rollback_diag_valid_status" -Needle '"validation_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.rejected_audit_ref"
    Assert-LogContains -Name "policy:module_rejected_audit_reference_state" -Needle '"state": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_status" -Needle '"status": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_reason" -Needle '"reason": "retained_audit_rollback_reference_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_state" -Needle '"durable_audit_record": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_rollback_state" -Needle '"rollback_plan": "rejected_retained_reference"' -TimeoutSeconds 1

    $moduleAuditCanonical = @(
        "canonicalization=raios.audit_record.canonical.v0",
        "schema=raios.audit_record.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "denial_event_id=$moduleAuditDenialEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleAuditHash = Get-TextSha256 -Text $moduleAuditCanonical
    $moduleAuditCommand = "agent module.audit_rollback_diagnostic $moduleAuditHash $moduleRollbackHash $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash $moduleAuditPreInventoryHash $moduleAuditCleanupHash $moduleAuditDenialEventId $moduleAuditRetainedReferenceEventId $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleAuditCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "audit_rollback_reference_valid_but_loader_and_slot_missing"' },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_ref_present"; Needle = '"audit_record_hash_reference_present": true' },
        @{ Suffix = "rollback_ref_present"; Needle = '"rollback_plan_hash_reference_present": true' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "no_durable_write"; Needle = '"durable_audit_written": false' },
        @{ Suffix = "not_installed"; Needle = '"rollback_plan_installed": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "inventory_none"; Needle = '"service_inventory_change": "none"' }
    )

    $moduleAuditResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_diagnostic"
    $moduleServiceSlotRetainedAuditEventId = [string]$moduleAuditResponse.body.result.retained_audit_rollback_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_service_slot_retained_audit_reference_event_id_captured" -Value $moduleServiceSlotRetainedAuditEventId

    Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic"
    Assert-LogContains -Name "protocol:module_service_slot_diag_schema" -Needle '"schema": "raios.module_service_slot_reservation_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_inventory_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent_reason" -Needle '"validation_reason": "service_slot_reservation_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.service_slot_allocator" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_allocator"
    Assert-LogContains -Name "protocol:module_service_slot_allocator_schema" -Needle '"schema": "raios.module_service_slot_allocator_readiness.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_source_evidence_mutation" -Needle '"mutates_global_event_log": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_source_evidence_scope" -Needle '"global_event_log_mutation": "retained_current_boot_source_evidence_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_no_records" -Needle '"creates_service_slot_reservation_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_no_inventory_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContainsFields -NamePrefix "protocol:module_service_slot_allocator_source_evidence_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "runtime_schema"; Needle = '"schema": "raios.service_slot_allocator_runtime_source_evidence.v0"' },
        @{ Suffix = "runtime_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.service_slot_allocator_runtime"' },
        @{ Suffix = "registry_schema"; Needle = '"schema": "raios.service_slot_registry_binding_source_evidence.v0"' },
        @{ Suffix = "registry_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.service_slot_registry_binding"' },
        @{ Suffix = "health_schema"; Needle = '"schema": "raios.service_health_state_model_source_evidence.v0"' },
        @{ Suffix = "health_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.service_health_state_model"' },
        @{ Suffix = "cleanup_schema"; Needle = '"schema": "raios.service_unload_cleanup_plan_source_evidence.v0"' },
        @{ Suffix = "cleanup_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.service_unload_cleanup_plan"' },
        @{ Suffix = "durable_schema"; Needle = '"schema": "raios.service_slot_allocator_durable_audit_write_source_evidence.v0"' },
        @{ Suffix = "durable_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.durable_audit_write"' },
        @{ Suffix = "rollback_schema"; Needle = '"schema": "raios.service_slot_allocator_rollback_install_source_evidence.v0"' },
        @{ Suffix = "rollback_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.rollback_plan_install"' },
        @{ Suffix = "loader_schema"; Needle = '"schema": "raios.service_slot_allocator_module_loader_source_evidence.v0"' },
        @{ Suffix = "loader_locator"; Needle = '"source_fact_locator": "module.service_slot_allocator.module_loader"' },
        @{ Suffix = "retained"; Needle = '"status": "retained_current_boot_source_evidence"' },
        @{ Suffix = "event"; Needle = '"event_id": "event.current_boot.' }
    )
    Assert-LogContains -Name "protocol:module_service_slot_allocator_absent_state" -Needle '"retained_service_slot_reservation": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_absent_reason" -Needle '"reason": "retained_service_slot_reservation_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_absent_status" -Needle '"readiness_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_fact_schema" -Needle '"schema": "raios.ram_only_service_slot_allocator.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_fact_source_observed" -Needle '"source_evidence_state": "observed_current_boot_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_fact_source_event" -Needle '"source_evidence_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_prereq_gates" -Needle '"allocator_prerequisite_gates": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_durable_prereq" -Needle '"durable_audit_write": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_rollback_prereq" -Needle '"rollback_plan_install": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_loader_prereq" -Needle '"module_loader": {' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_runtime_missing" -Needle '"reason": "service_slot_allocator_runtime_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_can_allocate_false" -Needle '"can_allocate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleServiceSlotCanonical = @(
        "canonicalization=raios.module_service_slot_reservation.canonical.v0",
        "schema=raios.module_service_slot_reservation.v0",
        "load_mode=ram_only",
        "scope=current_boot",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "retained_audit_rollback_reference_event_id=$moduleServiceSlotRetainedAuditEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "audit_record_sha256=$moduleAuditHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "pre_load_service_inventory_sha256=$moduleAuditPreInventoryHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleServiceSlotReservationHash = Get-TextSha256 -Text $moduleServiceSlotCanonical
    $moduleServiceSlotCommand = "agent module.service_slot_diagnostic $moduleServiceSlotReservationHash $moduleAuditRetainedReferenceEventId $moduleServiceSlotRetainedAuditEventId $moduleGrantHash $moduleAuditHash $moduleRollbackHash $moduleAuditPreInventoryHash $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleServiceSlotCommand -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_service_slot_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"' },
        @{ Suffix = "reservation_hash_echo"; Needle = "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "inventory_hash_echo"; Needle = "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" },
        @{ Suffix = "slot_echo"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "policy_present"; Needle = '"reservation_reference_present": true' },
        @{ Suffix = "policy_no_reserved_slot"; Needle = '"service_slot_reserved": false' },
        @{ Suffix = "policy_can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "policy_inventory_none"; Needle = '"service_inventory_change": "none"' }
    )

    Send-AgentCommand -Command "agent module.service_slot_allocator" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_allocator"
    Assert-LogContainsFields -NamePrefix "protocol:module_service_slot_allocator_after_reservation_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "reservation_present"; Needle = '"retained_service_slot_reservation_present": true' },
        @{ Suffix = "reservation_state"; Needle = '"status": "retained_hash_reference_only_not_allocated"' },
        @{ Suffix = "reservation_not_allocator"; Needle = '"reason": "service_slot_reservation_is_evidence_not_allocator_state"' },
        @{ Suffix = "reservation_hash_echo"; Needle = "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "slot_echo"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "readiness_status"; Needle = '"readiness_status": "missing"' },
        @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_registry_binding_missing"' },
        @{ Suffix = "runtime_source_available"; Needle = '"source_evidence_state": "observed_current_boot_available"' },
        @{ Suffix = "runtime_source_status"; Needle = '"source_evidence_status": "available"' },
        @{ Suffix = "runtime_source_reason"; Needle = '"source_evidence_reason": "service_slot_allocator_runtime_available"' },
        @{ Suffix = "runtime_available_true"; Needle = '"allocator_runtime_available": true' },
        @{ Suffix = "registry_available_false"; Needle = '"registry_binding_available": false' },
        @{ Suffix = "durable_false"; Needle = '"durable_audit_written": false' },
        @{ Suffix = "rollback_false"; Needle = '"rollback_plan_installed": false' },
        @{ Suffix = "can_allocate_false"; Needle = '"can_allocate": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "inventory_none"; Needle = '"service_inventory_change": "none"' }
    )

    Send-AgentCommand -Command "agent module.loader_identity" -ExpectedMarker "RAIOS_AGENT_END module.loader_identity"
    Assert-LogContainsFields -NamePrefix "protocol:module_loader_identity_source_evidence_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_loader_identity.v0"' },
        @{ Suffix = "mutates_source_evidence_only"; Needle = '"mutates_global_event_log": true' },
        @{ Suffix = "mutation_scope"; Needle = '"global_event_log_mutation": "retained_current_boot_source_evidence_only"' },
        @{ Suffix = "source_schema"; Needle = '"schema": "raios.module_loader_identity_source_evidence.v0"' },
        @{ Suffix = "source_retained"; Needle = '"status": "retained_current_boot_source_evidence"' },
        @{ Suffix = "source_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "source_method"; Needle = '"source_method": "module.loader_identity"' },
        @{ Suffix = "source_locator"; Needle = '"source_fact_locator": "module.loader_identity.loader_identity"' },
        @{ Suffix = "retained_evidence_present"; Needle = '"retained_module_evidence_present": true' },
        @{ Suffix = "readiness_status"; Needle = '"readiness_status": "denied_missing_service_slot_allocator_runtime"' },
        @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_allocator_runtime_missing"' },
        @{ Suffix = "identity_missing"; Needle = '"identity_reason": "module_loader_identity_missing"' },
        @{ Suffix = "fact_source_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "source_state"; Needle = '"source_evidence_state": "retained_current_boot"' },
        @{ Suffix = "no_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_load"; Needle = '"loads_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.loader_artifact_hash_binding" -ExpectedMarker "RAIOS_AGENT_END module.loader_artifact_hash_binding"
    Assert-LogContainsFields -NamePrefix "protocol:module_loader_artifact_hash_binding_source_evidence_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_loader_artifact_hash_binding.v0"' },
        @{ Suffix = "mutates_source_evidence_only"; Needle = '"mutates_global_event_log": true' },
        @{ Suffix = "mutation_scope"; Needle = '"global_event_log_mutation": "retained_current_boot_source_evidence_only"' },
        @{ Suffix = "source_schema"; Needle = '"schema": "raios.module_loader_artifact_hash_binding_source_evidence.v0"' },
        @{ Suffix = "source_retained"; Needle = '"status": "retained_current_boot_source_evidence"' },
        @{ Suffix = "source_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "source_method"; Needle = '"source_method": "module.loader_artifact_hash_binding"' },
        @{ Suffix = "source_locator"; Needle = '"source_fact_locator": "module.loader_artifact_hash_binding.artifact_hash_binding"' },
        @{ Suffix = "retained_evidence_present"; Needle = '"retained_module_evidence_present": true' },
        @{ Suffix = "readiness_status"; Needle = '"readiness_status": "denied_missing_service_slot_allocator_runtime"' },
        @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_allocator_runtime_missing"' },
        @{ Suffix = "artifact_missing"; Needle = '"artifact_hash_binding_reason": "module_loader_artifact_hash_binding_missing"' },
        @{ Suffix = "identity_source_present"; Needle = '"loader_identity_source_evidence_present": true' },
        @{ Suffix = "identity_source_event"; Needle = '"loader_identity_source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "fact_source_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "source_state"; Needle = '"source_evidence_state": "retained_current_boot"' },
        @{ Suffix = "no_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_load"; Needle = '"loads_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.loader_entrypoint_abi" -ExpectedMarker "RAIOS_AGENT_END module.loader_entrypoint_abi"
    Assert-LogContainsFields -NamePrefix "protocol:module_loader_entrypoint_abi_source_evidence_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_loader_entrypoint_abi.v0"' },
        @{ Suffix = "mutates_source_evidence_only"; Needle = '"mutates_global_event_log": true' },
        @{ Suffix = "mutation_scope"; Needle = '"global_event_log_mutation": "retained_current_boot_source_evidence_only"' },
        @{ Suffix = "source_schema"; Needle = '"schema": "raios.module_loader_entrypoint_abi_source_evidence.v0"' },
        @{ Suffix = "source_retained"; Needle = '"status": "retained_current_boot_source_evidence"' },
        @{ Suffix = "source_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "source_method"; Needle = '"source_method": "module.loader_entrypoint_abi"' },
        @{ Suffix = "source_locator"; Needle = '"source_fact_locator": "module.loader_entrypoint_abi.entrypoint_abi"' },
        @{ Suffix = "retained_evidence_present"; Needle = '"retained_module_evidence_present": true' },
        @{ Suffix = "readiness_status"; Needle = '"readiness_status": "denied_missing_service_slot_allocator_runtime"' },
        @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_allocator_runtime_missing"' },
        @{ Suffix = "entrypoint_missing"; Needle = '"fact_reason": "module_loader_entrypoint_abi_missing"' },
        @{ Suffix = "dependency_source_event"; Needle = '"dependency_source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "fact_source_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "source_state"; Needle = '"source_evidence_state": "retained_current_boot"' },
        @{ Suffix = "no_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_load"; Needle = '"loads_artifact": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $loaderFactSourceEvidenceDiagnostics = @(
        @{ Prefix = "module_loader_address_space_boundary"; Method = "module.loader_address_space_boundary"; Schema = "raios.module_loader_address_space_boundary.v0"; SourceSchema = "raios.module_loader_address_space_boundary_source_evidence.v0"; Locator = "module.loader_address_space_boundary.address_space_boundary"; MissingReason = "module_loader_address_space_boundary_missing" },
        @{ Prefix = "module_loader_memory_map_constraints"; Method = "module.loader_memory_map_constraints"; Schema = "raios.module_loader_memory_map_constraints.v0"; SourceSchema = "raios.module_loader_memory_map_constraints_source_evidence.v0"; Locator = "module.loader_memory_map_constraints.memory_map_constraints"; MissingReason = "module_loader_memory_map_constraints_missing" },
        @{ Prefix = "module_loader_capability_import_table"; Method = "module.loader_capability_import_table"; Schema = "raios.module_loader_capability_import_table.v0"; SourceSchema = "raios.module_loader_capability_import_table_source_evidence.v0"; Locator = "module.loader_capability_import_table.capability_import_table"; MissingReason = "module_loader_capability_import_table_missing" },
        @{ Prefix = "module_loader_service_slot_binding"; Method = "module.loader_service_slot_binding"; Schema = "raios.module_loader_service_slot_binding.v0"; SourceSchema = "raios.module_loader_service_slot_binding_source_evidence.v0"; Locator = "module.loader_service_slot_binding.service_slot_binding"; MissingReason = "module_loader_service_slot_binding_missing" },
        @{ Prefix = "module_loader_health_state_hooks"; Method = "module.loader_health_state_hooks"; Schema = "raios.module_loader_health_state_hooks.v0"; SourceSchema = "raios.module_loader_health_state_hooks_source_evidence.v0"; Locator = "module.loader_health_state_hooks.health_state_hooks"; MissingReason = "module_loader_health_state_hooks_missing" },
        @{ Prefix = "module_loader_rollback_hooks"; Method = "module.loader_rollback_hooks"; Schema = "raios.module_loader_rollback_hooks.v0"; SourceSchema = "raios.module_loader_rollback_hooks_source_evidence.v0"; Locator = "module.loader_rollback_hooks.rollback_hooks"; MissingReason = "module_loader_rollback_hooks_missing" },
        @{ Prefix = "module_loader_audit_rollback_write_boundary_binding"; Method = "module.loader_audit_rollback_write_boundary_binding"; Schema = "raios.module_loader_audit_rollback_write_boundary_binding.v0"; SourceSchema = "raios.module_loader_audit_rollback_write_boundary_binding_source_evidence.v0"; Locator = "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding"; MissingReason = "module_loader_audit_rollback_write_boundary_binding_missing" }
    )
    foreach ($fact in $loaderFactSourceEvidenceDiagnostics) {
        Send-AgentCommand -Command ("agent " + $fact.Method) -ExpectedMarker ("RAIOS_AGENT_END " + $fact.Method)
        Assert-LogContainsFields -NamePrefix ("protocol:" + $fact.Prefix + "_source_evidence_") -TimeoutSeconds 1 -Fields @(
            @{ Suffix = "schema"; Needle = ('"schema": "' + $fact.Schema + '"') },
            @{ Suffix = "mutates_source_evidence_only"; Needle = '"mutates_global_event_log": true' },
            @{ Suffix = "mutation_scope"; Needle = '"global_event_log_mutation": "retained_current_boot_source_evidence_only"' },
            @{ Suffix = "source_schema"; Needle = ('"schema": "' + $fact.SourceSchema + '"') },
            @{ Suffix = "source_retained"; Needle = '"status": "retained_current_boot_source_evidence"' },
            @{ Suffix = "source_event_id"; Needle = '"event_id": "event.current_boot.' },
            @{ Suffix = "source_method"; Needle = ('"source_method": "' + $fact.Method + '"') },
            @{ Suffix = "source_locator"; Needle = ('"source_fact_locator": "' + $fact.Locator + '"') },
            @{ Suffix = "retained_evidence_present"; Needle = '"retained_module_evidence_present": true' },
            @{ Suffix = "readiness_status"; Needle = '"readiness_status": "denied_missing_service_slot_allocator_runtime"' },
            @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_allocator_runtime_missing"' },
            @{ Suffix = "fact_missing"; Needle = ('"fact_reason": "' + $fact.MissingReason + '"') },
            @{ Suffix = "dependency_source_event"; Needle = '"dependency_source_evidence_event_id": "event.current_boot.' },
            @{ Suffix = "fact_source_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
            @{ Suffix = "source_state"; Needle = '"source_evidence_state": "retained_current_boot"' },
            @{ Suffix = "no_descriptor"; Needle = '"accepts_loader_descriptor": false' },
            @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
            @{ Suffix = "no_load"; Needle = '"loads_artifact": false' },
            @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
        )
    }

    Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime"
    Assert-LogContainsFields -NamePrefix "protocol:module_loader_runtime_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_loader_runtime_readiness.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_load"; Needle = '"loads_artifact": false' },
        @{ Suffix = "no_slots"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_inventory_records"; Needle = '"creates_service_inventory_records": false' },
        @{ Suffix = "inventory_none"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "manifest_present"; Needle = '"retained_module_manifest_reference_available"' },
        @{ Suffix = "artifact_present"; Needle = '"retained_module_candidate_artifact_reference_available"' },
        @{ Suffix = "service_slot_present"; Needle = '"retained_module_service_slot_reservation_available"' },
        @{ Suffix = "allocator_schema"; Needle = '"schema": "raios.module_service_slot_allocator_readiness.v0"' },
        @{ Suffix = "allocator_source"; Needle = '"source_method": "module.service_slot_allocator"' },
        @{ Suffix = "allocator_not_ready"; Needle = '"service_slot_allocator_ready": false' },
        @{ Suffix = "readiness_status"; Needle = '"readiness_status": "denied_missing_service_slot_allocator_runtime"' },
        @{ Suffix = "readiness_reason"; Needle = '"readiness_reason": "service_slot_allocator_runtime_missing"' },
        @{ Suffix = "loader_fact_schema"; Needle = '"schema": "raios.module_loader_identity.v0"' },
        @{ Suffix = "loader_fact_missing"; Needle = '"reason": "module_loader_identity_missing"' },
        @{ Suffix = "loader_identity_source"; Needle = '"source_method": "module.loader_identity"' },
        @{ Suffix = "loader_identity_locator"; Needle = '"source_fact_locator": "module.loader_identity.loader_identity"' },
        @{ Suffix = "loader_identity_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_identity_source_evidence.v0"' },
        @{ Suffix = "loader_identity_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "loader_identity_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "loader_identity_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "loader_identity_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_identity_missing"' },
        @{ Suffix = "artifact_hash_source"; Needle = '"source_method": "module.loader_artifact_hash_binding"' },
        @{ Suffix = "artifact_hash_locator"; Needle = '"source_fact_locator": "module.loader_artifact_hash_binding.artifact_hash_binding"' },
        @{ Suffix = "artifact_hash_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_artifact_hash_binding_source_evidence.v0"' },
        @{ Suffix = "artifact_hash_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "artifact_hash_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "artifact_hash_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "artifact_hash_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_artifact_hash_binding_missing"' },
        @{ Suffix = "entrypoint_source"; Needle = '"source_method": "module.loader_entrypoint_abi"' },
        @{ Suffix = "entrypoint_locator"; Needle = '"source_fact_locator": "module.loader_entrypoint_abi.entrypoint_abi"' },
        @{ Suffix = "entrypoint_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_entrypoint_abi_source_evidence.v0"' },
        @{ Suffix = "entrypoint_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "entrypoint_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "entrypoint_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "entrypoint_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_entrypoint_abi_missing"' },
        @{ Suffix = "address_space_source"; Needle = '"source_method": "module.loader_address_space_boundary"' },
        @{ Suffix = "address_space_locator"; Needle = '"source_fact_locator": "module.loader_address_space_boundary.address_space_boundary"' },
        @{ Suffix = "address_space_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_address_space_boundary_source_evidence.v0"' },
        @{ Suffix = "address_space_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "address_space_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "address_space_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "address_space_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_address_space_boundary_missing"' },
        @{ Suffix = "memory_map_source"; Needle = '"source_method": "module.loader_memory_map_constraints"' },
        @{ Suffix = "memory_map_locator"; Needle = '"source_fact_locator": "module.loader_memory_map_constraints.memory_map_constraints"' },
        @{ Suffix = "memory_map_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_memory_map_constraints_source_evidence.v0"' },
        @{ Suffix = "memory_map_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "memory_map_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "memory_map_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "memory_map_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_memory_map_constraints_missing"' },
        @{ Suffix = "capability_table_source"; Needle = '"source_method": "module.loader_capability_import_table"' },
        @{ Suffix = "capability_table_locator"; Needle = '"source_fact_locator": "module.loader_capability_import_table.capability_import_table"' },
        @{ Suffix = "capability_table_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_capability_import_table_source_evidence.v0"' },
        @{ Suffix = "capability_table_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "capability_table_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "capability_table_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "capability_table_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_capability_import_table_missing"' },
        @{ Suffix = "service_slot_source"; Needle = '"source_method": "module.loader_service_slot_binding"' },
        @{ Suffix = "service_slot_locator"; Needle = '"source_fact_locator": "module.loader_service_slot_binding.service_slot_binding"' },
        @{ Suffix = "service_slot_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_service_slot_binding_source_evidence.v0"' },
        @{ Suffix = "service_slot_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "service_slot_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "service_slot_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "service_slot_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_service_slot_binding_missing"' },
        @{ Suffix = "health_source"; Needle = '"source_method": "module.loader_health_state_hooks"' },
        @{ Suffix = "health_locator"; Needle = '"source_fact_locator": "module.loader_health_state_hooks.health_state_hooks"' },
        @{ Suffix = "health_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_health_state_hooks_source_evidence.v0"' },
        @{ Suffix = "health_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "health_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "health_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "health_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_health_state_hooks_missing"' },
        @{ Suffix = "rollback_source"; Needle = '"source_method": "module.loader_rollback_hooks"' },
        @{ Suffix = "rollback_locator"; Needle = '"source_fact_locator": "module.loader_rollback_hooks.rollback_hooks"' },
        @{ Suffix = "rollback_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_rollback_hooks_source_evidence.v0"' },
        @{ Suffix = "rollback_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "rollback_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "rollback_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "rollback_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_rollback_hooks_missing"' },
        @{ Suffix = "write_boundary_source"; Needle = '"source_method": "module.loader_audit_rollback_write_boundary_binding"' },
        @{ Suffix = "write_boundary_locator"; Needle = '"source_fact_locator": "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding"' },
        @{ Suffix = "write_boundary_source_evidence_schema"; Needle = '"source_evidence_schema": "raios.module_loader_audit_rollback_write_boundary_binding_source_evidence.v0"' },
        @{ Suffix = "write_boundary_source_evidence_observed"; Needle = '"source_evidence_state": "observed_current_boot_missing"' },
        @{ Suffix = "write_boundary_source_evidence_event"; Needle = '"source_evidence_event_id": "event.current_boot.' },
        @{ Suffix = "write_boundary_source_evidence_status"; Needle = '"source_evidence_status": "missing"' },
        @{ Suffix = "write_boundary_source_evidence_reason"; Needle = '"source_evidence_reason": "module_loader_audit_rollback_write_boundary_binding_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
