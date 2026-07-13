    Send-AgentCommand -Command "agent provider.context_injection_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate"
    $providerContextInjectionGate = Get-LastAgentResponseJson -Method "provider.context_injection_gate"
    $providerContextInjectionAuthorization = $providerContextInjectionGate.evidence | Where-Object { $_.id -eq "provider_context_injection_authorization" } | Select-Object -First 1
    $providerContextBodyCheck = $providerContextInjectionGate.evidence | Where-Object { $_.id -eq "final_prewrite_body_check" } | Select-Object -First 1
    $providerContextProjectionBinding = $providerContextInjectionGate.evidence | Where-Object { $_.id -eq "provider_projection_binding" } | Select-Object -First 1
    $providerContextInjectionGateOk = (
        $providerContextInjectionGate.schema -eq "raios.evidence_response.v1" -and
        $providerContextInjectionGate.family -eq "provider.context_injection_gate" -and
        $providerContextInjectionGate.classification -eq "local_only" -and
        $providerContextInjectionAuthorization.status -eq "blocked" -and
        $providerContextInjectionAuthorization.reason -eq "final_injection_authorization_missing" -and
        $providerContextBodyCheck.status -eq "not_attempted" -and
        $providerContextProjectionBinding.facts.redaction_policy_hash -cmatch '^sha256:[0-9a-f]{64}$' -and
        $providerContextProjectionBinding.facts.field_classification_hash -cmatch '^sha256:[0-9a-f]{64}$' -and
        $providerContextProjectionBinding.facts.token_budget_hash -cmatch '^sha256:[0-9a-f]{64}$' -and
        $providerContextInjectionGate.decision.outcome -eq "denied" -and
        @($providerContextInjectionGate.decision.grants).Count -eq 0 -and
        @($providerContextInjectionGate.decision.effects).Count -eq 0
    )
    Add-Predicate -Name "protocol:provider_context_injection_gate_v1_denial" -Expected "v1 denial carries ordered authorization/body/hash evidence with no grants or effects" -Passed $providerContextInjectionGateOk -Actual ($providerContextInjectionGate | ConvertTo-Json -Compress -Depth 12)

    Send-AgentCommand -Command "agent provider.context_injection_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate_selftest"
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
