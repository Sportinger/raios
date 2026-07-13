
    Send-AgentCommand -Command "agent module.manifest_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic"
    Assert-LogContains -Name "protocol:module_manifest_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent_reason" -Needle '"reason": "module_manifest_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_manifest_missing" -Needle '"id": "module_manifest", "kind": "reference", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "module_manifest_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "module_manifest_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "manifest_present"; Needle = '"id": "module_manifest", "kind": "reference", "status": "verified"' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" },
        @{ Suffix = "manifest_hash_echo"; Needle = "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" },
        @{ Suffix = "valid_decision_denied_no_authority"; Needle = '"outcome": "denied", "reason": "candidate_artifact_missing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' }
    )

    $moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
    $moduleManifestRetainedReferenceEventId = [string]($moduleManifestResponse.evidence | Where-Object id -eq "module_manifest_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_manifest_retained_reference_event_id_captured" -Value $moduleManifestRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_manifest_reference"

    Send-AgentCommand -Command "agent module.manifest_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_manifest_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_valid_case" -Needle '"case": "accepted_current_boot_manifest_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_mismatch_case" -Needle '"case": "mismatched_manifest_hash_reference"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.grant_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic"
    Assert-LogContains -Name "protocol:module_grant_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent_reason" -Needle '"reason": "computed_capability_grant_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_computed_missing" -Needle '"id": "computed_capability_grant", "kind": "reference", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "computed_capability_grant_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_loader_unavailable" -Needle '"evidence_id": "loader", "status": "unavailable"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_slot_unallocated" -Needle '"evidence_id": "service_slot", "status": "unallocated"' -TimeoutSeconds 1


    # M6A-2b: bind the REAL delivered-candidate artifact identity (the echo
    # wasm actually delivered over the serial channel in M6A-2a) instead of a
    # synthetic 2222... placeholder, so the module-evidence cross-check
    # evaluates the real candidate SHA-256 while load stays denied. Computed
    # from the on-disk artifact and anchored to the known ECHO hash (== the
    # intake artifact_sha256 proven in the candidate-delivery profile).
    $echoCandidateArtifactPath = Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
    $moduleGrantArtifactHash = Get-FileSha256OrNull -Path $echoCandidateArtifactPath
    $expectedEchoCandidateSha = "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"
    $realCandidateShaOk = ($null -ne $moduleGrantArtifactHash) -and ($moduleGrantArtifactHash -eq $expectedEchoCandidateSha)
    Add-Predicate -Name "protocol:module_evidence_real_candidate_sha_matches_echo" -Expected "candidate artifact sha256 == delivered echo wasm == $expectedEchoCandidateSha" -Passed $realCandidateShaOk -Actual $(if ($realCandidateShaOk) { "matched" } else { "got=$moduleGrantArtifactHash path=$echoCandidateArtifactPath" })
    if (-not $realCandidateShaOk) {
        throw "M6A-2b: real echo candidate artifact SHA missing or != known ECHO hash"
    }
    # vm_test_report / local_attestation identities remain synthetic here
    # (report-file hash is generated post-run; binding a real one is a later
    # land-if-cheap step) — documented gap, not a fake capability.
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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_retained"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "computed_grant_reference_verified"; Needle = '"id": "computed_capability_grant", "kind": "reference", "status": "verified"' },
        @{ Suffix = "valid_reference_load_still_denied"; Needle = '"reason": "hash_reference_valid_but_loader_audit_rollback_and_slot_missing", "source_event_id": null, "classification": "local_only", "facts": {"state": "present", "status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" }
    )

    $moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $moduleAuditRetainedReferenceEventId = [string]($moduleGrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_grant_retained_reference_event_id_captured" -Value $moduleAuditRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_grant_reference"

    Send-AgentCommand -Command "agent module.artifact_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic"
    Assert-LogContains -Name "protocol:module_artifact_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent_reason" -Needle '"reason": "candidate_artifact_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_artifact_missing" -Needle '"id": "candidate_artifact", "kind": "reference", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "candidate_artifact_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "candidate_artifact_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"id": "candidate_artifact", "kind": "reference", "status": "verified"' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" },
        @{ Suffix = "valid_decision_denied_no_authority"; Needle = '"outcome": "denied", "reason": "vm_test_report_missing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' }
    )

    $moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
    $moduleArtifactRetainedReferenceEventId = [string]($moduleArtifactResponse.evidence | Where-Object id -eq "candidate_artifact_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_artifact_retained_reference_event_id_captured" -Value $moduleArtifactRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_artifact_reference"

    Send-AgentCommand -Command "agent module.artifact_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_artifact_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_valid_case" -Needle '"case": "accepted_current_boot_artifact_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_mismatch_case" -Needle '"case": "mismatched_artifact_reference_hash"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.vm_report_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic"
    Assert-LogContains -Name "protocol:module_vm_report_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent_reason" -Needle '"reason": "vm_test_report_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_report_missing" -Needle '"id": "vm_test_report", "kind": "reference", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "vm_test_report_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "vm_test_report_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"id": "vm_test_report", "kind": "reference", "status": "verified"' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" },
        @{ Suffix = "report_hash_echo"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "artifact_ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "valid_decision_denied_no_authority"; Needle = '"outcome": "denied", "reason": "local_attestation_missing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' }
    )

    $moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
    $moduleVmReportRetainedReferenceEventId = [string]($moduleVmReportResponse.evidence | Where-Object id -eq "vm_test_report_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_vm_report_retained_reference_event_id_captured" -Value $moduleVmReportRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_vm_report_reference"

    Send-AgentCommand -Command "agent module.vm_report_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_vm_report_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_valid_case" -Needle '"case": "accepted_current_boot_report_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_mismatch_case" -Needle '"case": "vm_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_grant_mismatch_case" -Needle '"case": "computed_grant_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.attestation_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic"
    Assert-LogContains -Name "protocol:module_attestation_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent_reason" -Needle '"reason": "local_attestation_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_attestation_missing" -Needle '"id": "local_attestation", "kind": "reference", "status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "local_attestation_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "local_attestation_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"id": "local_attestation", "kind": "reference", "status": "verified"' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" },
        @{ Suffix = "attestation_hash_echo"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "vm_report_hash_echo"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "valid_reference_load_still_denied"; Needle = '"reason": "local_attestation_reference_valid_but_loader_and_evidence_missing", "source_event_id": null, "classification": "local_only", "facts": {"state": "present", "status_detail": "valid_hash_reference_load_still_denied"' }
    )

    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $moduleAttestationRetainedReferenceEventId = [string]($moduleAttestationResponse.evidence | Where-Object id -eq "local_attestation_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_attestation_retained_reference_event_id_captured" -Value $moduleAttestationRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_attestation_reference"

    Send-AgentCommand -Command "agent module.attestation_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_attestation_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_valid_case" -Needle '"case": "accepted_current_boot_attestation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_mismatch_case" -Needle '"case": "local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_signature_invalid_case" -Needle '"case": "promotion_signature_invalid"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_signature_invalid_status" -Needle '"actual": {"status": "mismatched_local_attestation_signature"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.approval_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic"
    Assert-LogContains -Name "protocol:module_approval_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent_reason" -Needle '"reason": "local_approval_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_approval_missing" -Needle '"id": "local_approval", "kind": "reference", "status": "missing"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_approval_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "local_approval_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "local_approval_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"id": "local_approval", "kind": "reference", "status": "verified"' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" },
        @{ Suffix = "approval_hash_echo"; Needle = "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" },
        @{ Suffix = "attestation_hash_echo"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "valid_reference_load_still_denied"; Needle = '"reason": "local_approval_reference_valid_but_loader_and_evidence_missing", "source_event_id": null, "classification": "local_only", "facts": {"state": "present", "status_detail": "valid_hash_reference_load_still_denied"' }
    )

    $moduleApprovalResponse = Get-LastAgentResponseJson -Method "module.approval_diagnostic"
    $moduleApprovalRetainedReferenceEventId = [string]($moduleApprovalResponse.evidence | Where-Object id -eq "local_approval_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_approval_retained_reference_event_id_captured" -Value $moduleApprovalRetainedReferenceEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_approval_reference"

    Send-AgentCommand -Command "agent module.approval_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_approval_selftest_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_mutation" -Needle '"event_log_write_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_records" -Needle '"retained_record_create_count": 0' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_valid_case" -Needle '"case": "accepted_current_boot_approval_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_mismatch_case" -Needle '"case": "local_approval_reference_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.pre_audit"
    $modulePreAuditLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"

    $moduleAuditDenialEventId = [string]$modulePreAuditLoadResponse.event_id
    Assert-CurrentBootEventId -Name "protocol:module_audit_denial_event_id_captured" -Value $moduleAuditDenialEventId
    Assert-LogContains -Name "policy:module_pre_audit_load_denied" -Needle '"outcome": "denied", "reason": "durable_audit_write_missing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_artifact_retained" -Needle '"id": "candidate_artifact", "kind": "retained_reference", "status": "verified", "reason": "retained_candidate_artifact_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_vm_report_retained" -Needle '"id": "vm_test_report", "kind": "retained_reference", "status": "verified", "reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_attestation_retained" -Needle '"id": "local_attestation", "kind": "retained_reference", "status": "verified", "reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_grant_retained" -Needle '"id": "computed_capability_grant", "kind": "retained_reference", "status": "verified", "reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_approval_retained" -Needle '"id": "local_approval", "kind": "retained_reference", "status": "verified", "reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_audit_missing" -Needle '"id": "durable_audit_record", "kind": "retained_reference", "status": "missing", "reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_rollback_missing" -Needle '"id": "rollback_plan", "kind": "retained_reference", "status": "missing", "reason": "rollback_install_missing"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic"
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_mutation" -Needle "`"family`": `"module.audit_rollback_reference`",`r`r`n  `"scope`": `"current_boot`",`r`r`n  `"classification`": `"local_only`",`r`r`n  `"source_method`": `"module.audit_rollback_diagnostic`",`r`r`n  `"event_id`": null" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "audit_rollback_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent_reason" -Needle '"reason": "audit_rollback_reference_absent"' -TimeoutSeconds 1

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
    Assert-LogContains -Name "protocol:module_wrong_audit_rollback_diag_valid_status" -Needle '"status_detail": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.rejected_audit_ref"
    Assert-LogContains -Name "policy:module_rejected_audit_reference_state" -Needle '"state": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_status" -Needle '"status": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_reason" -Needle '"reason": "retained_audit_rollback_reference_wrong_schema_or_variant"' -TimeoutSeconds 1

    Assert-LogContains -Name "policy:module_rejected_audit_state" -Needle '"id": "durable_audit_record", "kind": "retained_reference", "status": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_rollback_state" -Needle '"id": "rollback_plan", "kind": "retained_reference", "status": "rejected"' -TimeoutSeconds 1

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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "audit_rollback_reference_valid_but_loader_and_slot_missing"' },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_rollback_references_present"; Needle = '"id": "audit_rollback_reference", "kind": "reference", "status": "verified"' },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "valid_decision_denied_no_authority"; Needle = '"outcome": "denied", "reason": "module_loader_unimplemented", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' }
    )

    $moduleAuditResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_diagnostic"
    $moduleServiceSlotRetainedAuditEventId = [string]($moduleAuditResponse.evidence | Where-Object id -eq "audit_rollback_reference_retained").source_event_id
    Assert-CurrentBootEventId -Name "protocol:module_service_slot_retained_audit_reference_event_id_captured" -Value $moduleServiceSlotRetainedAuditEventId
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_audit_rollback_reference"

    Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic"
    Assert-LogContains -Name "protocol:module_service_slot_diag_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_mutation" -Needle "`"family`": `"module.service_slot_reservation`",`r`r`n  `"scope`": `"current_boot`",`r`r`n  `"classification`": `"local_only`",`r`r`n  `"source_method`": `"module.service_slot_diagnostic`",`r`r`n  `"event_id`": null" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_decision_denied_no_authority" -Needle '"outcome": "denied", "reason": "service_slot_reservation_reference_absent", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent" -Needle '"status_detail": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent_reason" -Needle '"reason": "service_slot_reservation_reference_absent"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.service_slot_allocator" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_allocator"
Assert-LogContains -Name "protocol:module_service_slot_allocator_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_envelope" -Needle "`"family`": `"module.service_slot_allocator`",`r`r`n  `"scope`": `"current_boot`",`r`r`n  `"classification`": `"local_only`",`r`r`n  `"source_method`": `"module.service_slot_allocator`",`r`r`n  `"event_id`": null" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_reservation_missing" -Needle '"id": "service_slot_reservation", "kind": "retained_reference", "status": "missing", "reason": "retained_service_slot_reservation_missing", "source_event_id": null' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_no_authority" -Needle '"outcome": "denied", "reason": "retained_service_slot_reservation_missing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
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
        @{ Suffix = "valid_status"; Needle = '"status_detail": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"' },
        @{ Suffix = "reservation_hash_echo"; Needle = "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "inventory_hash_echo"; Needle = "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" },
        @{ Suffix = "slot_echo"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "recorded_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_status"; Needle = '"reason": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"source_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "policy_present"; Needle = '"id": "service_slot_reservation", "kind": "reference", "status": "verified"' },
        @{ Suffix = "valid_decision_denied_no_authority"; Needle = '"outcome": "denied", "reason": "ram_only_service_slot_allocator_unimplemented", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' }
    )
    Send-AgentCommand -Command "agent audit.events 24" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_service_slot_reference"

    Send-AgentCommand -Command "agent module.service_slot_allocator" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_allocator"
    Assert-LogContains -Name "protocol:module_service_slot_allocator_after_reservation_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_after_reservation_runtime" -Needle '"id": "service_slot_allocator_runtime", "kind": "readiness", "status": "verified", "reason": "service_slot_allocator_runtime_available", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_allocator_after_reservation_denied" -Needle '"outcome": "denied", "reason": "service_slot_allocator_authority_boundary_non_authorizing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Send-AgentCommand -Command "agent module.loader_identity" -ExpectedMarker "RAIOS_AGENT_END module.loader_identity"
    Assert-LogContains -Name "protocol:module_loader_identity_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_identity_fact" -Needle '"id": "loader_identity", "kind": "loader_fact", "status": "missing", "reason": "module_loader_identity_missing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    Send-AgentCommand -Command "agent module.loader_artifact_hash_binding" -ExpectedMarker "RAIOS_AGENT_END module.loader_artifact_hash_binding"
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_artifact_hash_binding_fact" -Needle '"id": "artifact_hash_binding", "kind": "loader_fact", "status": "missing", "reason": "module_loader_artifact_hash_binding_missing", "source_event_id": "event.current_boot.' -TimeoutSeconds 1
    $loaderFacts = @(
        @{ Method = "module.loader_entrypoint_abi"; Id = "entrypoint_abi"; Dependency = "artifact_hash_binding"; Reason = "module_loader_entrypoint_abi_missing" },
        @{ Method = "module.loader_address_space_boundary"; Id = "address_space_boundary"; Dependency = "entrypoint_abi"; Reason = "module_loader_address_space_boundary_missing" },
        @{ Method = "module.loader_memory_map_constraints"; Id = "memory_map_constraints"; Dependency = "address_space_boundary"; Reason = "module_loader_memory_map_constraints_missing" },
        @{ Method = "module.loader_capability_import_table"; Id = "capability_import_table"; Dependency = "memory_map_constraints"; Reason = "module_loader_capability_import_table_missing" },
        @{ Method = "module.loader_service_slot_binding"; Id = "service_slot_binding"; Dependency = "capability_import_table"; Reason = "module_loader_service_slot_binding_missing" },
        @{ Method = "module.loader_health_state_hooks"; Id = "health_state_hooks"; Dependency = "service_slot_binding"; Reason = "module_loader_health_state_hooks_missing" },
        @{ Method = "module.loader_rollback_hooks"; Id = "rollback_hooks"; Dependency = "health_state_hooks"; Reason = "module_loader_rollback_hooks_missing" },
        @{ Method = "module.loader_audit_rollback_write_boundary_binding"; Id = "audit_rollback_write_boundary_binding"; Dependency = "rollback_hooks"; Reason = "module_loader_audit_rollback_write_boundary_binding_missing" }
    )
    foreach ($fact in $loaderFacts) {
        Send-AgentCommand -Command ("agent " + $fact.Method) -ExpectedMarker ("RAIOS_AGENT_END " + $fact.Method)
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_schema") -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_dependency") -Needle ('"id": "' + $fact.Dependency + '", "kind": "loader_fact"') -TimeoutSeconds 1
        Assert-LogContains -Name ("protocol:" + $fact.Id + "_fact") -Needle ('"id": "' + $fact.Id + '", "kind": "loader_fact", "status": "missing", "reason": "' + $fact.Reason + '", "source_event_id": "event.current_boot.') -TimeoutSeconds 1
    }
    Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime"
    Assert-LogContains -Name "protocol:module_loader_runtime_schema" -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds 1

    Assert-LogContains -Name "protocol:module_loader_runtime_envelope" -Needle "`"family`": `"module.loader_runtime`",`r`r`n  `"scope`": `"current_boot`",`r`r`n  `"classification`": `"local_only`",`r`r`n  `"source_method`": `"module.loader_runtime`",`r`r`n  `"event_id`": null" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_first" -Needle '"id": "manifest_reference", "kind": "retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_attempt" -Needle '"id": "load_attempt_boundary", "kind": "execution_boundary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_last" -Needle '"id": "executable_entrypoint_invocation_boundary", "kind": "execution_boundary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_loader_runtime_denied" -Needle '"outcome": "denied", "reason": "service_slot_allocator_authority_boundary_non_authorizing", "requested_capability": "cap.module.load_ephemeral", "grants": [], "effects": []' -TimeoutSeconds 1
    Send-AgentCommand -Command "agent audit.events 64" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "command:agent.audit.events.module_loader_runtime_source_evidence"
    $moduleLoaderRuntimeAuditEventsResponse = Get-LastAgentResponseJson -Method "memory.recent_events"
    $moduleLoaderRuntimeInvocationEvents = @($moduleLoaderRuntimeAuditEventsResponse.body.result.events | Where-Object { $_.bindings.schema -eq "raios.module_loader_executable_entrypoint_invocation_boundary_source_evidence.v0" })
    $moduleLoaderRuntimeInvocationBoundary = if ($moduleLoaderRuntimeInvocationEvents.Count -gt 0) { $moduleLoaderRuntimeInvocationEvents[0].bindings } else { $null }
    $moduleLoaderRuntimeAuditInvocationChecks = @(
        @{ Suffix = "no_entrypoint_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.jumps_to_entrypoint } else { $null }) },
        @{ Suffix = "no_binding_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.binds_capability_validated_descriptor_to_executable_pages } else { $null }) },
        @{ Suffix = "no_maps_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.maps_executable_pages } else { $null }) },
        @{ Suffix = "no_page_mapping_plan_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.produces_executable_page_mapping_plan } else { $null }) },
        @{ Suffix = "no_image_layout_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.produces_executable_image_layout } else { $null }) },
        @{ Suffix = "no_load_plan_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.produces_executable_load_plan } else { $null }) },
        @{ Suffix = "no_artifact_bytes_scoped"; Expected = $false; Actual = $(if ($null -ne $moduleLoaderRuntimeInvocationBoundary) { [bool]$moduleLoaderRuntimeInvocationBoundary.accepts_artifact_bytes } else { $null }) }
    )
    foreach ($check in $moduleLoaderRuntimeAuditInvocationChecks) {
        $passed = ($null -ne $moduleLoaderRuntimeInvocationBoundary) -and $check.Actual -eq $check.Expected
        Add-Predicate -Name ("protocol:module_loader_runtime_audit_event_invocation_boundary_" + $check.Suffix) -Expected ([string]$check.Expected) -Passed $passed -Actual ([string]$check.Actual)
        if (-not $passed) {
            throw ("Expected module.loader_runtime audit event invocation boundary " + $check.Suffix + " to be " + [string]$check.Expected + ", got " + [string]$check.Actual)
        }
    }
