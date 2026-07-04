    Send-AgentCommand -Command "agent provider.context_injection_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate"
    Assert-LogContains -Name "protocol:provider_context_injection_gate_schema" -Needle '"schema": "raios.provider_context_injection_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_schema" -Needle '"final_authorization_schema": "raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_missing" -Needle '"final_authorization": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_body_check_not_attempted" -Needle '"final_prewrite_body_check": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_can_attach_false" -Needle '"can_attach_context": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_current_boot_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_final" -Needle '"reason": "final_injection_authorization_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_disabled" -Needle '"reason": "automatic_context_injection_disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_required_authorization" -Needle '"raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_required_trust_verifier" -Needle '"provider_trust_verifier_metadata"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_redaction_hash" -Needle '"redaction_policy_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_classification_hash" -Needle '"field_classification_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_budget_hash" -Needle '"token_budget_hash": "sha256:' -TimeoutSeconds 1
    $providerContextInjectionGate = Get-LastAgentResponseJson -Method "provider.context_injection_gate"
    $providerContextInjectionOmission = $providerContextInjectionGate.body.result.evidence.recovery_status_omission
    $providerContextInjectionOmissionOk = (
        $providerContextInjectionOmission.schema -eq "raios.provider_minimal.local_only_omission.v0" -and
        $providerContextInjectionOmission.status -eq "omitted_from_provider_context" -and
        $providerContextInjectionOmission.fact_field -eq "current.recovery_lifeline_status" -and
        $providerContextInjectionOmission.locator -eq "recovery.lifeline.status.current_boot" -and
        $providerContextInjectionOmission.classification -eq "local_only" -and
        $providerContextInjectionOmission.provider_export -eq $false -and
        $providerContextInjectionOmission.context_attached_to_provider_body -eq $false -and
        $providerContextInjectionOmission.provider_write -eq "not_attempted"
    )
    Add-Predicate -Name "protocol:provider_context_injection_gate_recovery_status_omission_evidence" -Expected "provider_context_injection_gate_recovery_status_omitted" -Passed $providerContextInjectionOmissionOk -Actual $(if ($providerContextInjectionOmissionOk) { "omitted" } else { "missing_or_exportable" })
    if (-not $providerContextInjectionOmissionOk) {
        throw "Expected provider.context_injection_gate to expose recovery status omission evidence without body attachment"
    }

    Send-AgentCommand -Command "agent provider.context_injection_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate_selftest"
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema" -Needle '"schema": "raios.provider_context_injection_gate_negative_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_export" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_omitted_hash_case" -Needle '"case": "final_authorization_omitted_field_list_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_omitted_hash_reason" -Needle '"actual_reason": "final_injection_authorization_substituted_record"' -TimeoutSeconds 1
    $providerContextInjectionSelftest = Get-LastAgentResponseJson -Method "provider.context_injection_gate_selftest"
    $providerContextInjectionOmittedHashCase = $providerContextInjectionSelftest.body.result.cases | Where-Object { $_.case -eq "final_authorization_omitted_field_list_hash_mismatch" } | Select-Object -First 1
    $providerContextInjectionOmittedHashCaseOk = (
        $providerContextInjectionSelftest.body.result.case_count -eq 8 -and
        $providerContextInjectionSelftest.body.result.passed -eq $true -and
        $providerContextInjectionOmittedHashCase.expected_status -eq "rejected" -and
        $providerContextInjectionOmittedHashCase.expected_reason -eq "final_injection_authorization_substituted_record" -and
        $providerContextInjectionOmittedHashCase.actual_status -eq "rejected" -and
        $providerContextInjectionOmittedHashCase.actual_reason -eq "final_injection_authorization_substituted_record" -and
        $providerContextInjectionOmittedHashCase.passed -eq $true
    )
    Add-Predicate -Name "protocol:provider_context_injection_selftest_omitted_hash_fail_closed" -Expected "tampered_omitted_hash_rejected" -Passed $providerContextInjectionOmittedHashCaseOk -Actual $(if ($providerContextInjectionOmittedHashCaseOk) { "rejected" } else { "missing_or_not_rejected" })
    if (-not $providerContextInjectionOmittedHashCaseOk) {
        throw "Expected provider.context_injection_gate_selftest to reject a tampered omitted_field_list_hash"
    }
