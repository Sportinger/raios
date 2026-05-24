        Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        Assert-LogContains -Name "quick:module_load_schema" -Needle '"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_manifest_missing" -Needle '"module_manifest": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_grant_missing" -Needle '"computed_capability_grant": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_no_inventory_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_not_attempted" -Needle '"load_attempted": false' -TimeoutSeconds 1

        Send-AgentCommand -Command "recovery.load_artifact" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact"
        Assert-LogContains -Name "quick:recovery_load_schema" -Needle '"schema": "raios.recovery_artifact_load_boundary.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_normal_path_not_used" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_identity_missing" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_not_attempted" -Needle '"load_attempted": false' -TimeoutSeconds 1

        Send-AgentCommand -Command "agent audit.events 16" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
        Assert-LogContains -Name "quick:audit_events_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_limit" -Needle '"limit": 16' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_module_load_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_recovery_load_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1
