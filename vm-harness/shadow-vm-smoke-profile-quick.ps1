        Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        Assert-LogContains -Name "quick:module_load_schema" -Needle '"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_manifest_missing" -Needle '"module_manifest": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_grant_missing" -Needle '"computed_capability_grant": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_no_inventory_change" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:module_load_not_attempted" -Needle '"load_attempted": false' -TimeoutSeconds 1

        Send-AgentCommand -Command "module.load_ephemeral svc.demo.nope" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        $wrongHelloTarget = Get-LastAgentResponseJson -Method "module.load_ephemeral"
        if ($wrongHelloTarget.t -ne "error" -or $wrongHelloTarget.body.schema -ne "raios.module_load_gate.v0" -or $wrongHelloTarget.body.code -ne "capability_denied") {
            throw "Expected wrong hello target to stay on denied module load gate"
        }

        Send-AgentCommand -Command "module.load_ephemeral external:svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        $externalHelloTarget = Get-LastAgentResponseJson -Method "module.load_ephemeral"
        if ($externalHelloTarget.t -ne "error" -or $externalHelloTarget.body.schema -ne "raios.module_load_gate.v0" -or $externalHelloTarget.body.code -ne "capability_denied") {
            throw "Expected external hello target to stay on denied module load gate"
        }

        Send-AgentCommand -Command "recovery.load_artifact" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact"
        Assert-LogContains -Name "quick:recovery_load_schema" -Needle '"schema": "raios.recovery_artifact_load_boundary.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_normal_path_not_used" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_identity_missing" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:recovery_load_not_attempted" -Needle '"load_attempted": false' -TimeoutSeconds 1

        Send-AgentCommand -Command "module.load_ephemeral svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        $helloLoad = Get-LastAgentResponseJson -Method "module.load_ephemeral"
        if ($helloLoad.body.result.schema -ne "raios.ram_only_hello_service.v0") {
            throw "Expected RAM-only hello service schema, got $($helloLoad.body.result.schema)"
        }
        Assert-CurrentBootEventId -Name "quick:hello_load_event_id" -Value $helloLoad.body.result.event_id
        if ($helloLoad.body.result.load_request.descriptor_id -ne "load_descriptor.current_boot.svc.demo.hello.v0") {
            throw "Expected hello load request to bind the current-boot descriptor id"
        }
        if ($helloLoad.body.result.load_descriptor.schema -ne "raios.current_boot_load_descriptor.v0") {
            throw "Expected typed current-boot hello load descriptor"
        }
        $helloDescriptorHash = $helloLoad.body.result.load_descriptor.source.sha256
        $helloDescriptorLocator = "current_image.descriptor_source.svc.demo.hello.v0"
        $helloDescriptorKind = "current_image_descriptor_source"
        if (-not $helloDescriptorHash -or -not $helloDescriptorHash.StartsWith("sha256:")) {
            throw "Expected hello load descriptor to expose a SHA-256 source hash"
        }
        if ($helloLoad.body.result.load_descriptor.source.locator -ne $helloDescriptorLocator) {
            throw "Expected hello load descriptor to cite its current-image source locator"
        }
        if ($helloLoad.body.result.load_descriptor.source.kind -ne $helloDescriptorKind) {
            throw "Expected hello load descriptor to cite its current-image source kind"
        }
        if (-not $helloLoad.body.result.load_descriptor.source.validated) {
            throw "Expected hello load descriptor source to be validated"
        }
        if ($helloLoad.body.result.load_descriptor.source.canonicalization -ne "raios.current_boot_load_descriptor.canonical.v0") {
            throw "Expected hello load descriptor to cite its canonicalization"
        }
        if ($helloLoad.body.result.load_descriptor.source.text -notlike "*schema=raios.current_boot_load_descriptor.v0*") {
            throw "Expected hello load descriptor to expose its canonical source text"
        }
        if ($helloLoad.body.result.load_descriptor.source.text -notlike "*source_kind=current_image_descriptor_source*") {
            throw "Expected hello load descriptor source text to identify the current-image source kind"
        }
        if ($helloLoad.body.result.load_descriptor.source.text -notlike "*source_locator=$helloDescriptorLocator*") {
            throw "Expected hello load descriptor source text to carry the current-image source locator"
        }
        if ($helloLoad.body.result.load_request.descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello load request to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.load_request.descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.load_request.descriptor_source_validated) {
            throw "Expected hello load request to cite the validated current-image descriptor source"
        }
        if ($helloLoad.body.result.service.load_descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello service response to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.service.load_descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.service.load_descriptor_source_validated) {
            throw "Expected hello service response to cite the validated current-image descriptor source"
        }
        if ($helloLoad.body.result.loader.descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello loader response to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.loader.descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.loader.descriptor_source_validated) {
            throw "Expected hello loader response to cite the validated current-image descriptor source"
        }
        if (-not $helloLoad.body.result.service.loaded -or -not $helloLoad.body.result.service.running) {
            throw "Expected loaded/running hello service after load_start"
        }
        if ($helloLoad.body.result.loader.accepts_external_artifact_bytes) {
            throw "Hello service must not accept external artifact bytes"
        }
        if ($helloLoad.body.result.loader.writes_persistent_state) {
            throw "Hello service must remain RAM-only"
        }

        Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory"
        $servicesAfterLoad = Get-LastAgentResponseJson -Method "service.inventory"
        $helloInventory = @($servicesAfterLoad.body.result.services | Where-Object { $_.id -eq "svc.demo.hello" })
        if ($helloInventory.Count -ne 1) {
            throw "Expected svc.demo.hello in service.inventory after load"
        }
        if ($helloInventory[0].health -ne "healthy" -or -not $helloInventory[0].running) {
            throw "Expected healthy/running svc.demo.hello in service.inventory"
        }
        if ($helloInventory[0].load_descriptor_id -ne "load_descriptor.current_boot.svc.demo.hello.v0") {
            throw "Expected svc.demo.hello inventory record to cite the load descriptor"
        }
        if ($helloInventory[0].load_descriptor_source_locator -ne $helloDescriptorLocator) {
            throw "Expected svc.demo.hello inventory record to cite the load descriptor source locator"
        }
        if ($helloInventory[0].load_descriptor_source_kind -ne $helloDescriptorKind -or -not $helloInventory[0].load_descriptor_source_validated) {
            throw "Expected svc.demo.hello inventory record to cite the validated current-image descriptor source"
        }
        if ($helloInventory[0].load_descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected svc.demo.hello inventory record to cite the load descriptor source hash"
        }

        Send-AgentCommand -Command "service.stop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.stop"
        $helloStop = Get-LastAgentResponseJson -Method "service.stop"
        Assert-CurrentBootEventId -Name "quick:hello_stop_event_id" -Value $helloStop.body.result.event_id
        if ($helloStop.body.result.service.running) {
            throw "Expected stopped hello service after service.stop"
        }

        Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
        $helloDrop = Get-LastAgentResponseJson -Method "service.drop"
        Assert-CurrentBootEventId -Name "quick:hello_drop_event_id" -Value $helloDrop.body.result.event_id
        if ($helloDrop.body.result.service.loaded -or $helloDrop.body.result.service.running) {
            throw "Expected dropped hello service after service.drop"
        }

        Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory"
        $servicesAfterDrop = Get-LastAgentResponseJson -Method "service.inventory"
        $helloAfterDrop = @($servicesAfterDrop.body.result.services | Where-Object { $_.id -eq "svc.demo.hello" })
        if ($helloAfterDrop.Count -ne 0) {
            throw "Expected svc.demo.hello to be removed from service.inventory after drop"
        }

        Send-AgentCommand -Command "module.load_ephemeral host_bound:svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
        $hostHelloLoad = Get-LastAgentResponseJson -Method "module.load_ephemeral"
        $hostDescriptorHash = $hostHelloLoad.body.result.load_descriptor.source.sha256
        $hostDescriptorLocator = "host_build.descriptor_source.svc.demo.hello.v0"
        $hostDescriptorKind = "host_bound_descriptor_source"
        if ($hostHelloLoad.body.result.schema -ne "raios.ram_only_hello_service.v0") {
            throw "Expected host-bound RAM-only hello service schema, got $($hostHelloLoad.body.result.schema)"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.locator -ne $hostDescriptorLocator) {
            throw "Expected host-bound hello load descriptor to cite its host-produced source locator"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.kind -ne $hostDescriptorKind) {
            throw "Expected host-bound hello load descriptor to cite its source kind"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.binds_source_locator -ne $helloDescriptorLocator) {
            throw "Expected host-bound descriptor to bind the current-image source locator"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.binds_source_kind -ne $helloDescriptorKind) {
            throw "Expected host-bound descriptor to bind the current-image source kind"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.binds_source_hash -ne $helloDescriptorHash) {
            throw "Expected host-bound descriptor to bind the current-image source hash"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.text -notlike "*binds_source_hash=$helloDescriptorHash*") {
            throw "Expected host-bound source text to carry the bound current-image source hash"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.text -notlike "*source_kind=$hostDescriptorKind*") {
            throw "Expected host-bound source text to identify its source kind"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.text -notlike "*source_locator=$hostDescriptorLocator*") {
            throw "Expected host-bound source text to carry its source locator"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.text -notlike "*binds_source_locator=$helloDescriptorLocator*") {
            throw "Expected host-bound source text to bind the current-image source locator"
        }
        if ($hostHelloLoad.body.result.load_descriptor.source.text -notlike "*binds_source_kind=$helloDescriptorKind*") {
            throw "Expected host-bound source text to bind the current-image source kind"
        }
        if ($hostHelloLoad.body.result.load_request.descriptor_source_hash -ne $hostDescriptorHash -or $hostHelloLoad.body.result.loader.descriptor_source_hash -ne $hostDescriptorHash) {
            throw "Expected host-bound load request and loader to cite the host-bound descriptor source hash"
        }

        Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory"
        $servicesAfterHostLoad = Get-LastAgentResponseJson -Method "service.inventory"
        $hostInventory = @($servicesAfterHostLoad.body.result.services | Where-Object { $_.id -eq "svc.demo.hello" })
        if ($hostInventory.Count -ne 1) {
            throw "Expected host-bound svc.demo.hello in service.inventory after load"
        }
        if ($hostInventory[0].load_descriptor_source_locator -ne $hostDescriptorLocator -or $hostInventory[0].load_descriptor_source_kind -ne $hostDescriptorKind) {
            throw "Expected host-bound inventory record to cite the host-produced descriptor source"
        }
        if ($hostInventory[0].load_descriptor_source_hash -ne $hostDescriptorHash) {
            throw "Expected host-bound inventory record to cite the host-bound descriptor source hash"
        }
        if ($hostInventory[0].binds_source_hash -ne $helloDescriptorHash) {
            throw "Expected host-bound inventory record to bind the current-image descriptor source hash"
        }

        Send-AgentCommand -Command "service.stop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.stop"
        $hostStop = Get-LastAgentResponseJson -Method "service.stop"
        if ($hostStop.body.result.loader.descriptor_source_locator -ne $hostDescriptorLocator) {
            throw "Expected host-bound stop event response to cite the active descriptor source"
        }

        Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
        $hostDrop = Get-LastAgentResponseJson -Method "service.drop"
        if ($hostDrop.body.result.loader.descriptor_source_locator -ne $hostDescriptorLocator) {
            throw "Expected host-bound drop event response to cite the active descriptor source"
        }

        Send-AgentCommand -Command "agent audit.events 32" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
        $recentEvents = Get-LastAgentResponseJson -Method "memory.recent_events"
        $helloEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.ram_only_hello_service.lifecycle" -and $_.resource -eq "svc.demo.hello" })
        if ($helloEvents.Count -lt 6) {
            throw "Expected hello load/stop/drop lifecycle events in RAM audit log"
        }
        $helloDescriptorEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "load_descriptor.current_boot.svc.demo.hello.v0" })
        if ($helloDescriptorEvents.Count -lt 6) {
            throw "Expected hello lifecycle events to cite the load descriptor"
        }
        $helloDescriptorHashEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "load_descriptor_source_hash" -and $_.bindings.load_descriptor_source_hash -eq $helloDescriptorHash })
        if ($helloDescriptorHashEvents.Count -lt 3) {
            throw "Expected hello lifecycle events to cite the descriptor source hash"
        }
        $helloDescriptorSourceEvents = @($helloEvents | Where-Object { $_.bindings.load_descriptor_source_locator -eq $helloDescriptorLocator -and $_.bindings.load_descriptor_source_kind -eq $helloDescriptorKind -and $_.bindings.load_descriptor_source_validated })
        if ($helloDescriptorSourceEvents.Count -lt 3) {
            throw "Expected hello lifecycle events to cite the validated current-image descriptor source"
        }
        $hostDescriptorHashEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "load_descriptor_source_hash" -and $_.bindings.load_descriptor_source_hash -eq $hostDescriptorHash })
        if ($hostDescriptorHashEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the host-bound descriptor source hash"
        }
        $hostDescriptorSourceEvents = @($helloEvents | Where-Object { $_.bindings.load_descriptor_source_locator -eq $hostDescriptorLocator -and $_.bindings.load_descriptor_source_kind -eq $hostDescriptorKind -and $_.bindings.load_descriptor_source_validated -and $_.bindings.binds_source_hash -eq $helloDescriptorHash })
        if ($hostDescriptorSourceEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the bound current-image descriptor source hash"
        }
        Assert-LogContains -Name "quick:audit_events_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_limit" -Needle '"limit": 32' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_module_load_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_recovery_load_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_kind" -Needle '"kind": "raios.ram_only_hello_service.lifecycle"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_resource" -Needle '"resource": "svc.demo.hello"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor" -Needle "load_descriptor.current_boot.svc.demo.hello.v0" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_hash" -Needle '"load_descriptor_source_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_kind" -Needle "current_image_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_source_kind" -Needle "host_bound_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_binds_hash" -Needle '"binds_source_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_validated" -Needle '"load_descriptor_source_validated": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1
