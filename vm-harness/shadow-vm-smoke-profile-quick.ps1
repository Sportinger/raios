        $HelloLoadPlanPreflightSchema = "raios.current_boot_artifact_load_plan_preflight.v0"
        $HelloLoadPlanPreflightId = "artifact_load_plan_preflight.current_boot.svc.demo.hello.v0"
        $HelloLoadPlanPreflightStatus = "accepted_builtin_current_boot_only"
        $HelloServiceSlotIntentId = "service_slot_intent.current_boot.svc.demo.hello.v0"
        $HelloRamOnlyServiceSlotId = "ram_only:svc.demo.hello"
        $HelloServiceSlotActivationSchema = "raios.ram_only_service_slot_activation.v0"
        $HelloServiceSlotActivationId = "service_slot_activation.current_boot.svc.demo.hello.v0"
        $HelloServiceSlotActivationActiveStatus = "active_current_boot"
        $HelloServiceSlotActivationStoppedStatus = "stopped_current_boot"
        $HelloServiceSlotActivationClearedStatus = "cleared_current_boot"
        $HelloServiceSlotActivationMissingStatus = "missing_current_boot"
        $HelloLoadPlanPreflightSelftestSchema = "raios.current_boot_artifact_load_plan_preflight_selftest.v0"
        $HelloLoadPlanPreflightSelftestId = "artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0"

        $AssertHelloLoadPlanPreflight = {
            param(
                [string]$Name,
                [object]$Preflight,
                [string]$DescriptorSourceLocator,
                [string]$DescriptorSourceHash,
                [string]$ArtifactIdentityHash,
                [string]$ArtifactContentHash,
                [string]$ArtifactReferenceHash,
                [string]$ArtifactBytesHash
            )

            if (-not $Preflight) {
                throw "Expected $Name to expose artifact load-plan preflight"
            }
            if ($Preflight.schema -ne $HelloLoadPlanPreflightSchema) {
                throw "Expected $Name artifact load-plan preflight schema"
            }
            if ($Preflight.id -ne $HelloLoadPlanPreflightId) {
                throw "Expected $Name artifact load-plan preflight id"
            }
            if ($Preflight.scope -ne "current_boot" -or $Preflight.classification -ne "local_only" -or $Preflight.status -ne $HelloLoadPlanPreflightStatus) {
                throw "Expected $Name artifact load-plan preflight current_boot/local_only accepted status"
            }
            if (-not $Preflight.preflight_hash -or -not $Preflight.preflight_hash.StartsWith("sha256:")) {
                throw "Expected $Name artifact load-plan preflight hash"
            }
            if ($Preflight.service_id -ne "svc.demo.hello" -or $Preflight.artifact_id -ne "builtin:svc.demo.hello" -or $Preflight.load_descriptor_id -ne "load_descriptor.current_boot.svc.demo.hello.v0") {
                throw "Expected $Name artifact load-plan preflight to bind the Hello service, artifact, and descriptor"
            }
            if ($Preflight.descriptor_source_locator -ne $DescriptorSourceLocator -or $Preflight.descriptor_source_hash -ne $DescriptorSourceHash) {
                throw "Expected $Name artifact load-plan preflight to bind the selected descriptor source"
            }
            if ($Preflight.artifact_identity_id -ne "builtin_artifact_identity.svc.demo.hello.v0" -or $Preflight.artifact_identity_hash -ne $ArtifactIdentityHash) {
                throw "Expected $Name artifact load-plan preflight to bind the artifact identity"
            }
            if ($Preflight.artifact_content_binding_hash -ne $ArtifactContentHash) {
                throw "Expected $Name artifact load-plan preflight to bind the artifact content"
            }
            if ($Preflight.artifact_reference_id -ne "builtin_artifact_reference.svc.demo.hello.v0" -or $Preflight.artifact_reference_hash -ne $ArtifactReferenceHash -or $Preflight.artifact_bytes_sha256 -ne $ArtifactBytesHash) {
                throw "Expected $Name artifact load-plan preflight to bind the artifact reference and bytes"
            }
            if ($Preflight.service_slot_intent_schema -ne "raios.ram_only_service_slot_intent.v0" -or $Preflight.service_slot_intent_id -ne $HelloServiceSlotIntentId -or $Preflight.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name artifact load-plan preflight to bind the RAM-only service slot intent"
            }
            if (-not $Preflight.accepted -or -not $Preflight.authorizes_builtin_current_boot_start) {
                throw "Expected $Name artifact load-plan preflight to authorize only the built-in current-boot start"
            }
            if ($Preflight.authorizes_candidate_artifact_execution -or $Preflight.accepts_external_artifact_bytes -or $Preflight.loads_candidate_bytes -or $Preflight.maps_executable_pages -or $Preflight.writes_persistent_state -or $Preflight.writes_durable_audit_log -or $Preflight.installs_rollback_plan -or $Preflight.grants_broad_mutation) {
                throw "Expected $Name artifact load-plan preflight to deny candidate execution, external bytes, executable mapping, persistence, durable audit, rollback, and broad mutation"
            }

            return $Preflight.preflight_hash
        }

        $AssertHelloLoadPlanPreflightReference = {
            param(
                [string]$Name,
                [object]$Record,
                [string]$ExpectedHash,
                [bool]$ExpectServiceSlotIntent = $false
            )

            if ($Record.artifact_load_plan_preflight_id -ne $HelloLoadPlanPreflightId) {
                throw "Expected $Name to cite the artifact load-plan preflight id"
            }
            if ($Record.artifact_load_plan_preflight_hash -ne $ExpectedHash) {
                throw "Expected $Name to cite the artifact load-plan preflight hash"
            }
            if ($Record.artifact_load_plan_preflight_status -ne $HelloLoadPlanPreflightStatus) {
                throw "Expected $Name to cite the artifact load-plan preflight status"
            }
            if ($ExpectServiceSlotIntent -and $Record.service_slot_intent_id -ne $HelloServiceSlotIntentId) {
                throw "Expected $Name to cite the RAM-only service slot intent"
            }
            if ($Record.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name to cite the RAM-only service slot id"
            }
        }

        $AssertHelloServiceSlotActivation = {
            param(
                [string]$Name,
                [object]$Activation,
                [string]$DescriptorSourceHash,
                [string]$PreflightHash,
                [string]$ExpectedStatus,
                [bool]$ExpectedActive
            )

            if (-not $Activation) {
                throw "Expected $Name to expose service-slot activation"
            }
            if ($Activation.schema -ne $HelloServiceSlotActivationSchema -or $Activation.id -ne $HelloServiceSlotActivationId) {
                throw "Expected $Name service-slot activation schema/id"
            }
            if ($Activation.scope -ne "current_boot" -or $Activation.classification -ne "local_only" -or $Activation.persistence -ne "none") {
                throw "Expected $Name service-slot activation current_boot/local_only/none"
            }
            if ($Activation.status -ne $ExpectedStatus -or $Activation.active -ne $ExpectedActive) {
                throw "Expected $Name service-slot activation status $ExpectedStatus active=$ExpectedActive"
            }
            if (-not $Activation.activation_hash -or -not $Activation.activation_hash.StartsWith("sha256:")) {
                throw "Expected $Name service-slot activation hash"
            }
            if ($Activation.service_id -ne "svc.demo.hello" -or $Activation.artifact_id -ne "builtin:svc.demo.hello" -or $Activation.load_descriptor_id -ne "load_descriptor.current_boot.svc.demo.hello.v0") {
                throw "Expected $Name service-slot activation to bind the Hello service, artifact, and descriptor"
            }
            if ($Activation.descriptor_source_hash -ne $DescriptorSourceHash) {
                throw "Expected $Name service-slot activation to bind the selected descriptor source hash"
            }
            if ($Activation.artifact_load_plan_preflight_id -ne $HelloLoadPlanPreflightId -or $Activation.artifact_load_plan_preflight_hash -ne $PreflightHash -or $Activation.artifact_load_plan_preflight_status -ne $HelloLoadPlanPreflightStatus) {
                throw "Expected $Name service-slot activation to derive from the accepted load-plan preflight"
            }
            if ($Activation.service_slot_intent_id -ne $HelloServiceSlotIntentId -or $Activation.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name service-slot activation to bind the RAM-only service slot"
            }
            if (-not $Activation.accepted_preflight -or -not $Activation.authorizes_builtin_current_boot_start) {
                throw "Expected $Name service-slot activation to require accepted built-in current-boot preflight"
            }
            if ($Activation.authorizes_candidate_artifact_execution -or $Activation.writes_persistent_state) {
                throw "Expected $Name service-slot activation to deny candidate execution and persistence"
            }

            return $Activation.activation_hash
        }

        $AssertHelloServiceSlotActivationReference = {
            param(
                [string]$Name,
                [object]$Record,
                [string]$ExpectedHash,
                [string]$ExpectedStatus,
                [bool]$ExpectedActive
            )

            if ($Record.service_slot_activation_id -ne $HelloServiceSlotActivationId) {
                throw "Expected $Name to cite the service-slot activation id"
            }
            if ($Record.service_slot_activation_hash -ne $ExpectedHash) {
                throw "Expected $Name to cite the service-slot activation hash"
            }
            if ($Record.service_slot_activation_status -ne $ExpectedStatus) {
                throw "Expected $Name to cite service-slot activation status $ExpectedStatus"
            }
            if ($Record.service_slot_activation_active -ne $ExpectedActive) {
                throw "Expected $Name to cite service-slot activation active=$ExpectedActive"
            }
        }

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

        Send-AgentCommand -Command "service.descriptor_source_trust_selftest" -ExpectedMarker "RAIOS_AGENT_END service.descriptor_source_trust_selftest"
        $descriptorTrustSelftest = Get-LastAgentResponseJson -Method "service.descriptor_source_trust_selftest"
        if ($descriptorTrustSelftest.body.result.schema -ne "raios.descriptor_source_trust_selftest.v0") {
            throw "Expected descriptor-source trust selftest schema"
        }
        if ($descriptorTrustSelftest.body.result.id -ne "descriptor_source_trust_selftest.current_image.svc.demo.hello.v0") {
            throw "Expected stable descriptor-source trust selftest id"
        }
        if (-not $descriptorTrustSelftest.body.result.read_only -or $descriptorTrustSelftest.body.result.persistence -ne "none") {
            throw "Descriptor-source trust selftest must be read-only and non-persistent"
        }
        if (-not $descriptorTrustSelftest.body.result.diagnostic_hash -or -not $descriptorTrustSelftest.body.result.diagnostic_hash.StartsWith("sha256:")) {
            throw "Expected descriptor-source trust selftest diagnostic hash"
        }
        if ($descriptorTrustSelftest.body.result.case_count -ne 5 -or $descriptorTrustSelftest.body.result.passed_count -ne 5 -or -not $descriptorTrustSelftest.body.result.all_passed) {
            throw "Expected all descriptor-source trust selftest cases to pass"
        }
        if (-not $descriptorTrustSelftest.body.result.signature_envelope.signature_verified) {
            throw "Expected descriptor-source trust selftest to cite the verified signature envelope"
        }
        $descriptorTrustCaseNames = @($descriptorTrustSelftest.body.result.cases | ForEach-Object { $_.name })
        foreach ($caseName in @("valid_current_image_envelope", "tampered_payload_denied", "tampered_locator_kind_denied", "tampered_public_key_hash_denied", "tampered_signature_denied")) {
            if ($descriptorTrustCaseNames -notcontains $caseName) {
                throw "Missing descriptor-source trust selftest case $caseName"
            }
        }
        $descriptorTrustFailedCases = @($descriptorTrustSelftest.body.result.cases | Where-Object { -not $_.passed })
        if ($descriptorTrustFailedCases.Count -ne 0) {
            throw "Expected no descriptor-source trust selftest failures"
        }
        if ($descriptorTrustSelftest.body.result.denied_surfaces.descriptor_bytes_intake -ne "denied" -or $descriptorTrustSelftest.body.result.denied_surfaces.external_artifact_load -ne "denied" -or $descriptorTrustSelftest.body.result.denied_surfaces.persistent_install -ne "denied") {
            throw "Descriptor-source trust selftest must keep descriptor bytes, artifact load, and persistence denied"
        }

        Send-AgentCommand -Command "service.artifact_reference_trust_selftest" -ExpectedMarker "RAIOS_AGENT_END service.artifact_reference_trust_selftest"
        $artifactReferenceTrustSelftest = Get-LastAgentResponseJson -Method "service.artifact_reference_trust_selftest"
        if ($artifactReferenceTrustSelftest.body.result.schema -ne "raios.builtin_artifact_reference_trust_selftest.v0") {
            throw "Expected artifact-reference trust selftest schema"
        }
        if ($artifactReferenceTrustSelftest.body.result.id -ne "artifact_reference_trust_selftest.builtin.svc.demo.hello.v0") {
            throw "Expected stable artifact-reference trust selftest id"
        }
        if (-not $artifactReferenceTrustSelftest.body.result.read_only -or $artifactReferenceTrustSelftest.body.result.mutates_global_event_log -or $artifactReferenceTrustSelftest.body.result.persistence -ne "none") {
            throw "Artifact-reference trust selftest must be read-only, RAM-only, and non-mutating"
        }
        if (-not $artifactReferenceTrustSelftest.body.result.diagnostic_hash -or -not $artifactReferenceTrustSelftest.body.result.diagnostic_hash.StartsWith("sha256:")) {
            throw "Expected artifact-reference trust selftest diagnostic hash"
        }
        if ($artifactReferenceTrustSelftest.body.result.case_count -ne 5 -or $artifactReferenceTrustSelftest.body.result.passed_count -ne 5 -or -not $artifactReferenceTrustSelftest.body.result.all_passed) {
            throw "Expected all artifact-reference trust selftest cases to pass"
        }
        if ($artifactReferenceTrustSelftest.body.result.artifact_reference.schema -ne "raios.builtin_artifact_reference.v0" -or -not $artifactReferenceTrustSelftest.body.result.artifact_reference.validated) {
            throw "Expected artifact-reference trust selftest to cite validated artifact reference evidence"
        }
        if ($artifactReferenceTrustSelftest.body.result.identity_signature_envelope.schema -ne "raios.builtin_artifact_identity_signature_envelope.v0" -or -not $artifactReferenceTrustSelftest.body.result.identity_signature_envelope.signature_verified) {
            throw "Expected artifact-reference trust selftest to cite verified identity trust envelope"
        }
        $artifactReferenceTrustCaseNames = @($artifactReferenceTrustSelftest.body.result.cases | ForEach-Object { $_.name })
        foreach ($caseName in @("valid_builtin_artifact_reference", "tampered_artifact_bytes_hash_denied", "tampered_content_binding_hash_denied", "tampered_reference_hash_denied", "tampered_trust_payload_hash_denied")) {
            if ($artifactReferenceTrustCaseNames -notcontains $caseName) {
                throw "Missing artifact-reference trust selftest case $caseName"
            }
        }
        $artifactReferenceTrustFailedCases = @($artifactReferenceTrustSelftest.body.result.cases | Where-Object { -not $_.passed })
        if ($artifactReferenceTrustFailedCases.Count -ne 0) {
            throw "Expected no artifact-reference trust selftest failures"
        }
        if ($artifactReferenceTrustSelftest.body.result.denied_surfaces.artifact_bytes_intake -ne "denied" -or $artifactReferenceTrustSelftest.body.result.denied_surfaces.artifact_load -ne "denied" -or $artifactReferenceTrustSelftest.body.result.denied_surfaces.executable_mapping -ne "denied" -or $artifactReferenceTrustSelftest.body.result.denied_surfaces.persistent_install -ne "denied") {
            throw "Artifact-reference trust selftest must keep artifact bytes, artifact load, executable mapping, and persistence denied"
        }

        Send-AgentCommand -Command "service.artifact_load_plan_preflight_selftest" -ExpectedMarker "RAIOS_AGENT_END service.artifact_load_plan_preflight_selftest"
        $loadPlanPreflightSelftest = Get-LastAgentResponseJson -Method "service.artifact_load_plan_preflight_selftest"
        if ($loadPlanPreflightSelftest.body.result.schema -ne $HelloLoadPlanPreflightSelftestSchema) {
            throw "Expected artifact load-plan preflight selftest schema"
        }
        if ($loadPlanPreflightSelftest.body.result.id -ne $HelloLoadPlanPreflightSelftestId) {
            throw "Expected stable artifact load-plan preflight selftest id"
        }
        if (-not $loadPlanPreflightSelftest.body.result.read_only -or $loadPlanPreflightSelftest.body.result.mutates_global_event_log -or $loadPlanPreflightSelftest.body.result.persistence -ne "none") {
            throw "Artifact load-plan preflight selftest must be read-only, RAM-only, and non-mutating"
        }
        if (-not $loadPlanPreflightSelftest.body.result.diagnostic_hash -or -not $loadPlanPreflightSelftest.body.result.diagnostic_hash.StartsWith("sha256:")) {
            throw "Expected artifact load-plan preflight selftest diagnostic hash"
        }
        if ($loadPlanPreflightSelftest.body.result.service_slot_intent_id -ne $HelloServiceSlotIntentId -or $loadPlanPreflightSelftest.body.result.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
            throw "Expected artifact load-plan preflight selftest to cite the RAM-only service slot intent"
        }
        if ($loadPlanPreflightSelftest.body.result.artifact_load_plan_preflight.schema -ne $HelloLoadPlanPreflightSchema -or $loadPlanPreflightSelftest.body.result.artifact_load_plan_preflight.id -ne $HelloLoadPlanPreflightId -or -not $loadPlanPreflightSelftest.body.result.artifact_load_plan_preflight.accepted) {
            throw "Expected artifact load-plan preflight selftest to cite the accepted preflight"
        }
        if ($loadPlanPreflightSelftest.body.result.case_count -ne 8 -or $loadPlanPreflightSelftest.body.result.passed_count -ne 8 -or -not $loadPlanPreflightSelftest.body.result.all_passed) {
            throw "Expected all artifact load-plan preflight selftest cases to pass"
        }
        $loadPlanPreflightCaseNames = @($loadPlanPreflightSelftest.body.result.cases | ForEach-Object { $_.name })
        foreach ($caseName in @("valid_current_boot_load_plan_preflight", "tampered_descriptor_source_hash_denied", "tampered_artifact_identity_hash_denied", "tampered_content_binding_hash_denied", "tampered_artifact_reference_hash_denied", "tampered_artifact_bytes_hash_denied", "tampered_service_slot_intent_denied", "tampered_denial_flags_denied")) {
            if ($loadPlanPreflightCaseNames -notcontains $caseName) {
                throw "Missing artifact load-plan preflight selftest case $caseName"
            }
        }
        $loadPlanPreflightFailedCases = @($loadPlanPreflightSelftest.body.result.cases | Where-Object { -not $_.passed })
        if ($loadPlanPreflightFailedCases.Count -ne 0) {
            throw "Expected no artifact load-plan preflight selftest failures"
        }
        if ($loadPlanPreflightSelftest.body.result.denied_surfaces.external_artifact_bytes -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.candidate_artifact_execution -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.executable_mapping -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.persistent_install -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.durable_audit -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.rollback_install -ne "denied" -or $loadPlanPreflightSelftest.body.result.denied_surfaces.broad_mutation -ne "denied") {
            throw "Artifact load-plan preflight selftest must keep candidate execution, executable mapping, persistence, durable audit, rollback, and mutation denied"
        }

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
        $helloDescriptorEnvelope = $helloLoad.body.result.load_descriptor.source.signature_envelope
        if (-not $helloDescriptorEnvelope) {
            throw "Expected current-image hello descriptor source to expose a signature envelope"
        }
        if ($helloDescriptorEnvelope.schema -ne "raios.descriptor_source_signature_envelope.v0") {
            throw "Expected current-image hello descriptor source signature envelope schema"
        }
        if ($helloDescriptorEnvelope.id -ne "descriptor_source_signature.current_image.svc.demo.hello.v0") {
            throw "Expected current-image hello descriptor source signature envelope id"
        }
        if ($helloDescriptorEnvelope.algorithm -ne "ecdsa_p256_sha256_asn1_der") {
            throw "Expected current-image hello descriptor source to use the P-256 signature envelope"
        }
        if ($helloDescriptorEnvelope.verification_phase -ne "runtime_before_descriptor_selection") {
            throw "Expected descriptor source envelope to be verified before descriptor source selection"
        }
        if ($helloDescriptorEnvelope.payload_sha256 -ne $helloDescriptorHash) {
            throw "Expected descriptor source envelope payload hash to bind the current-image source hash"
        }
        if (-not $helloDescriptorEnvelope.envelope_hash -or -not $helloDescriptorEnvelope.envelope_hash.StartsWith("sha256:")) {
            throw "Expected descriptor source envelope hash"
        }
        if (-not $helloDescriptorEnvelope.public_key_sha256 -or -not $helloDescriptorEnvelope.public_key_sha256.StartsWith("sha256:")) {
            throw "Expected descriptor source envelope public key hash"
        }
        if (-not $helloDescriptorEnvelope.signature_sha256 -or -not $helloDescriptorEnvelope.signature_sha256.StartsWith("sha256:")) {
            throw "Expected descriptor source envelope signature hash"
        }
        if (-not $helloDescriptorEnvelope.signature_verified) {
            throw "Expected descriptor source envelope signature to verify"
        }
        if ($helloDescriptorEnvelope.authorizes_external_artifact_load -or $helloDescriptorEnvelope.authorizes_persistent_install) {
            throw "Descriptor source signature envelope must not authorize artifact loading or persistence"
        }
        $helloArtifactIdentity = $helloLoad.body.result.load_descriptor.artifact_identity
        if (-not $helloArtifactIdentity) {
            throw "Expected hello load descriptor to expose built-in artifact identity"
        }
        $helloArtifactIdentityHash = $helloArtifactIdentity.sha256
        if ($helloArtifactIdentity.schema -ne "raios.builtin_artifact_identity.v0") {
            throw "Expected built-in artifact identity schema"
        }
        if ($helloArtifactIdentity.id -ne "builtin_artifact_identity.svc.demo.hello.v0") {
            throw "Expected stable built-in artifact identity id"
        }
        if ($helloArtifactIdentity.artifact_id -ne "builtin:svc.demo.hello" -or $helloArtifactIdentity.service_id -ne "svc.demo.hello") {
            throw "Expected built-in artifact identity to bind hello service/artifact ids"
        }
        if (-not $helloArtifactIdentityHash -or -not $helloArtifactIdentityHash.StartsWith("sha256:")) {
            throw "Expected built-in artifact identity hash"
        }
        if (-not $helloArtifactIdentity.validated) {
            throw "Expected built-in artifact identity to be validated"
        }
        if ($helloArtifactIdentity.accepts_external_artifact_bytes -or $helloArtifactIdentity.loads_external_artifact -or $helloArtifactIdentity.maps_executable_pages -or $helloArtifactIdentity.writes_persistent_state) {
            throw "Built-in artifact identity must not accept/load/map external artifact bytes or write state"
        }
        $helloArtifactIdentityEnvelope = $helloArtifactIdentity.signature_envelope
        if ($helloArtifactIdentityEnvelope.schema -ne "raios.builtin_artifact_identity_signature_envelope.v0") {
            throw "Expected built-in artifact identity signature envelope schema"
        }
        if ($helloArtifactIdentityEnvelope.id -ne "artifact_identity_signature.builtin.svc.demo.hello.v0") {
            throw "Expected stable built-in artifact identity signature envelope id"
        }
        if ($helloArtifactIdentityEnvelope.payload_sha256 -ne $helloArtifactIdentityHash) {
            throw "Expected artifact identity envelope payload hash to bind identity hash"
        }
        if (-not $helloArtifactIdentityEnvelope.signature_verified) {
            throw "Expected artifact identity envelope signature to verify"
        }
        if ($helloArtifactIdentityEnvelope.authorizes_external_artifact_load -or $helloArtifactIdentityEnvelope.authorizes_persistent_install -or $helloArtifactIdentityEnvelope.authorizes_rollback_install) {
            throw "Artifact identity signature envelope must not authorize load, persistence, or rollback"
        }
        $helloArtifactContentBinding = $helloArtifactIdentity.content_binding
        if (-not $helloArtifactContentBinding) {
            throw "Expected built-in artifact identity to expose content binding"
        }
        $helloArtifactContentHash = $helloArtifactContentBinding.binding_hash
        if ($helloArtifactContentBinding.schema -ne "raios.builtin_artifact_content_binding.v0") {
            throw "Expected built-in artifact content binding schema"
        }
        if ($helloArtifactContentBinding.id -ne "builtin_artifact_content.svc.demo.hello.v0") {
            throw "Expected stable built-in artifact content binding id"
        }
        if ($helloArtifactContentBinding.content_kind -ne "repo_source_snapshot") {
            throw "Expected built-in artifact content to bind a repo source snapshot"
        }
        if ($helloArtifactContentBinding.source_locator -ne "seed-kernel/src/hello_service.rs") {
            throw "Expected built-in artifact content to cite hello_service.rs"
        }
        if (-not $helloArtifactContentBinding.source_sha256 -or -not $helloArtifactContentBinding.source_sha256.StartsWith("sha256:")) {
            throw "Expected built-in artifact content source hash"
        }
        if (-not $helloArtifactContentHash -or -not $helloArtifactContentHash.StartsWith("sha256:")) {
            throw "Expected built-in artifact content binding hash"
        }
        if ($helloArtifactContentBinding.trusted_by_envelope_id -ne $helloArtifactIdentityEnvelope.id -or $helloArtifactContentBinding.trusted_by_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected artifact content binding to cite the artifact identity trust envelope"
        }
        if (-not $helloArtifactContentBinding.trust_signature_verified -or -not $helloArtifactContentBinding.validated) {
            throw "Expected artifact content binding trust and validation to pass"
        }
        if ($helloArtifactContentBinding.accepts_external_artifact_bytes -or $helloArtifactContentBinding.loads_external_artifact -or $helloArtifactContentBinding.maps_executable_pages -or $helloArtifactContentBinding.writes_persistent_state) {
            throw "Artifact content binding must keep external artifact load, executable mapping, and persistence denied"
        }
        $helloArtifactReference = $helloArtifactIdentity.artifact_reference
        if (-not $helloArtifactReference) {
            throw "Expected built-in artifact identity to expose artifact reference"
        }
        $helloArtifactReferenceHash = $helloArtifactReference.reference_hash
        $helloArtifactBytesHash = $helloArtifactReference.artifact_bytes_sha256
        if ($helloArtifactReference.schema -ne "raios.builtin_artifact_reference.v0") {
            throw "Expected built-in artifact reference schema"
        }
        if ($helloArtifactReference.id -ne "builtin_artifact_reference.svc.demo.hello.v0") {
            throw "Expected stable built-in artifact reference id"
        }
        if ($helloArtifactReference.reference_kind -ne "repo_artifact_bytes_snapshot") {
            throw "Expected built-in artifact reference to bind repo artifact bytes"
        }
        if ($helloArtifactReference.artifact_locator -ne "seed-kernel/artifacts/svc.demo.hello.builtin.artifact") {
            throw "Expected built-in artifact reference to cite the repo artifact bytes"
        }
        if (-not $helloArtifactReferenceHash -or -not $helloArtifactReferenceHash.StartsWith("sha256:")) {
            throw "Expected built-in artifact reference hash"
        }
        if (-not $helloArtifactBytesHash -or -not $helloArtifactBytesHash.StartsWith("sha256:")) {
            throw "Expected built-in artifact bytes hash"
        }
        if ($helloArtifactReference.content_binding_hash -ne $helloArtifactContentHash) {
            throw "Expected artifact reference to bind the artifact content binding hash"
        }
        if ($helloArtifactReference.trusted_by_envelope_id -ne $helloArtifactIdentityEnvelope.id -or $helloArtifactReference.trusted_by_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected artifact reference to cite the artifact identity trust envelope"
        }
        if (-not $helloArtifactReference.trust_signature_verified -or -not $helloArtifactReference.validated) {
            throw "Expected artifact reference trust and validation to pass"
        }
        if ($helloArtifactReference.accepts_external_artifact_bytes -or $helloArtifactReference.loads_artifact_as_code -or $helloArtifactReference.maps_executable_pages -or $helloArtifactReference.writes_persistent_state) {
            throw "Artifact reference must keep artifact byte intake, code loading, executable mapping, and persistence denied"
        }
        $helloLoadPlanPreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "hello load response" `
            -Preflight $helloLoad.body.result.artifact_load_plan_preflight `
            -DescriptorSourceLocator $helloDescriptorLocator `
            -DescriptorSourceHash $helloDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        $helloLoadDescriptorPreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "hello load descriptor" `
            -Preflight $helloLoad.body.result.load_descriptor.artifact_load_plan_preflight `
            -DescriptorSourceLocator $helloDescriptorLocator `
            -DescriptorSourceHash $helloDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        if ($helloLoadDescriptorPreflightHash -ne $helloLoadPlanPreflightHash) {
            throw "Expected hello load response and descriptor to agree on artifact load-plan preflight hash"
        }
        $helloServiceSlotActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello load response" `
            -Activation $helloLoad.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($helloLoad.body.result.load_request.descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello load request to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.load_request.descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.load_request.descriptor_source_validated) {
            throw "Expected hello load request to cite the validated current-image descriptor source"
        }
        if ($helloLoad.body.result.load_request.descriptor_source_signature_envelope.envelope_hash -ne $helloDescriptorEnvelope.envelope_hash -or -not $helloLoad.body.result.load_request.descriptor_source_signature_envelope.signature_verified) {
            throw "Expected hello load request to cite the verified descriptor source envelope"
        }
        if ($helloLoad.body.result.load_request.artifact_identity_hash -ne $helloArtifactIdentityHash -or -not $helloLoad.body.result.load_request.artifact_identity_signature_envelope.signature_verified) {
            throw "Expected hello load request to cite the verified artifact identity"
        }
        if ($helloLoad.body.result.load_request.artifact_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.load_request.artifact_content_source_hash -ne $helloArtifactContentBinding.source_sha256 -or $helloLoad.body.result.load_request.artifact_content_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello load request to cite the verified artifact content binding"
        }
        if ($helloLoad.body.result.load_request.artifact_reference_hash -ne $helloArtifactReferenceHash -or $helloLoad.body.result.load_request.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or $helloLoad.body.result.load_request.artifact_reference_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.load_request.artifact_reference_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello load request to cite the verified artifact byte reference"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "hello load request" -Record $helloLoad.body.result.load_request -ExpectedHash $helloLoadPlanPreflightHash -ExpectServiceSlotIntent $true
        if ($helloLoad.body.result.service.load_descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello service response to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.service.load_descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.service.load_descriptor_source_validated) {
            throw "Expected hello service response to cite the validated current-image descriptor source"
        }
        if ($helloLoad.body.result.service.load_descriptor_source_signature_envelope.envelope_hash -ne $helloDescriptorEnvelope.envelope_hash -or -not $helloLoad.body.result.service.load_descriptor_source_signature_envelope.signature_verified) {
            throw "Expected hello service response to cite the verified descriptor source envelope"
        }
        if ($helloLoad.body.result.service.artifact_identity_hash -ne $helloArtifactIdentityHash -or -not $helloLoad.body.result.service.artifact_identity_signature_envelope.signature_verified) {
            throw "Expected hello service response to cite the verified artifact identity"
        }
        if ($helloLoad.body.result.service.artifact_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.service.artifact_content_source_hash -ne $helloArtifactContentBinding.source_sha256 -or $helloLoad.body.result.service.artifact_content_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello service response to cite the verified artifact content binding"
        }
        if ($helloLoad.body.result.service.artifact_reference_hash -ne $helloArtifactReferenceHash -or $helloLoad.body.result.service.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or $helloLoad.body.result.service.artifact_reference_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.service.artifact_reference_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello service response to cite the verified artifact byte reference"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "hello service response" -Record $helloLoad.body.result.service -ExpectedHash $helloLoadPlanPreflightHash
        & $AssertHelloServiceSlotActivationReference -Name "hello service response" -Record $helloLoad.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        if ($helloLoad.body.result.loader.descriptor_source_hash -ne $helloDescriptorHash) {
            throw "Expected hello loader response to cite the same descriptor source hash"
        }
        if ($helloLoad.body.result.loader.descriptor_source_kind -ne $helloDescriptorKind -or -not $helloLoad.body.result.loader.descriptor_source_validated) {
            throw "Expected hello loader response to cite the validated current-image descriptor source"
        }
        if ($helloLoad.body.result.loader.descriptor_source_signature_envelope.envelope_hash -ne $helloDescriptorEnvelope.envelope_hash -or -not $helloLoad.body.result.loader.descriptor_source_signature_envelope.signature_verified) {
            throw "Expected hello loader response to cite the verified descriptor source envelope"
        }
        if ($helloLoad.body.result.loader.artifact_identity_hash -ne $helloArtifactIdentityHash -or -not $helloLoad.body.result.loader.artifact_identity_signature_envelope.signature_verified) {
            throw "Expected hello loader response to cite the verified artifact identity"
        }
        if ($helloLoad.body.result.loader.artifact_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.loader.artifact_content_source_hash -ne $helloArtifactContentBinding.source_sha256 -or $helloLoad.body.result.loader.artifact_content_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello loader response to cite the verified artifact content binding"
        }
        if ($helloLoad.body.result.loader.artifact_reference_hash -ne $helloArtifactReferenceHash -or $helloLoad.body.result.loader.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or $helloLoad.body.result.loader.artifact_reference_content_binding_hash -ne $helloArtifactContentHash -or $helloLoad.body.result.loader.artifact_reference_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected hello loader response to cite the verified artifact byte reference"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "hello loader response" -Record $helloLoad.body.result.loader -ExpectedHash $helloLoadPlanPreflightHash -ExpectServiceSlotIntent $true
        & $AssertHelloServiceSlotActivationReference -Name "hello loader response" -Record $helloLoad.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        if (-not $helloLoad.body.result.service.loaded -or -not $helloLoad.body.result.service.running) {
            throw "Expected loaded/running hello service after load_start"
        }
        if ($helloLoad.body.result.loader.accepts_external_artifact_bytes) {
            throw "Hello service must not accept external artifact bytes"
        }
        if ($helloLoad.body.result.loader.writes_persistent_state) {
            throw "Hello service must remain RAM-only"
        }
        if ($helloLoad.body.result.loader.maps_executable_pages) {
            throw "Hello service must not map executable artifact pages"
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
        if ($helloInventory[0].load_descriptor_source_signature_envelope.envelope_hash -ne $helloDescriptorEnvelope.envelope_hash -or -not $helloInventory[0].load_descriptor_source_signature_envelope.signature_verified) {
            throw "Expected svc.demo.hello inventory record to cite the verified descriptor source envelope"
        }
        if ($helloInventory[0].artifact_identity_hash -ne $helloArtifactIdentityHash -or -not $helloInventory[0].artifact_identity_signature_envelope.signature_verified) {
            throw "Expected svc.demo.hello inventory record to cite the verified artifact identity"
        }
        if ($helloInventory[0].artifact_content_binding_hash -ne $helloArtifactContentHash -or $helloInventory[0].artifact_content_source_hash -ne $helloArtifactContentBinding.source_sha256 -or $helloInventory[0].artifact_content_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected svc.demo.hello inventory record to cite the verified artifact content binding"
        }
        if ($helloInventory[0].artifact_reference_hash -ne $helloArtifactReferenceHash -or $helloInventory[0].artifact_bytes_sha256 -ne $helloArtifactBytesHash -or $helloInventory[0].artifact_reference_content_binding_hash -ne $helloArtifactContentHash -or $helloInventory[0].artifact_reference_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected svc.demo.hello inventory record to cite the verified artifact byte reference"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "svc.demo.hello inventory record" -Record $helloInventory[0] -ExpectedHash $helloLoadPlanPreflightHash
        & $AssertHelloServiceSlotActivationReference -Name "svc.demo.hello inventory record" -Record $helloInventory[0] -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthRunning = Get-LastAgentResponseJson -Method "service.health"
        Assert-CurrentBootEventId -Name "quick:hello_health_running_event_id" -Value $helloHealthRunning.body.result.event_id
        if ($helloHealthRunning.body.result.schema -ne "raios.ram_only_hello_service.health.v0") {
            throw "Expected typed hello health response"
        }
        if ($helloHealthRunning.body.result.service.health -ne "healthy" -or -not $helloHealthRunning.body.result.service.loaded -or -not $helloHealthRunning.body.result.service.running) {
            throw "Expected hello health probe to report healthy/running after load"
        }
        if ($helloHealthRunning.body.result.load_descriptor.source.sha256 -ne $helloDescriptorHash -or $helloHealthRunning.body.result.load_descriptor.source.kind -ne $helloDescriptorKind) {
            throw "Expected hello health probe to cite the current-image descriptor source"
        }
        if ($helloHealthRunning.body.result.load_descriptor.source.signature_envelope.envelope_hash -ne $helloDescriptorEnvelope.envelope_hash -or -not $helloHealthRunning.body.result.load_descriptor.source.signature_envelope.signature_verified) {
            throw "Expected hello health probe to cite the verified descriptor source envelope"
        }
        if ($helloHealthRunning.body.result.load_descriptor.artifact_identity.sha256 -ne $helloArtifactIdentityHash -or -not $helloHealthRunning.body.result.load_descriptor.artifact_identity.signature_envelope.signature_verified) {
            throw "Expected hello health probe to cite the verified artifact identity"
        }
        if ($helloHealthRunning.body.result.load_descriptor.artifact_identity.content_binding.binding_hash -ne $helloArtifactContentHash -or -not $helloHealthRunning.body.result.load_descriptor.artifact_identity.content_binding.trust_signature_verified) {
            throw "Expected hello health probe to cite the verified artifact content binding"
        }
        if ($helloHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.reference_hash -ne $helloArtifactReferenceHash -or $helloHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or -not $helloHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.trust_signature_verified) {
            throw "Expected hello health probe to cite the verified artifact byte reference"
        }
        $helloHealthRunningPreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "hello running health descriptor" `
            -Preflight $helloHealthRunning.body.result.load_descriptor.artifact_load_plan_preflight `
            -DescriptorSourceLocator $helloDescriptorLocator `
            -DescriptorSourceHash $helloDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        if ($helloHealthRunningPreflightHash -ne $helloLoadPlanPreflightHash) {
            throw "Expected hello running health descriptor to retain the artifact load-plan preflight hash"
        }
        $helloHealthRunningActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello running health response" `
            -Activation $helloHealthRunning.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($helloHealthRunningActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello running health to retain the service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivation `
            -Name "hello running health descriptor" `
            -Activation $helloHealthRunning.body.result.load_descriptor.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true | Out-Null
        & $AssertHelloServiceSlotActivationReference -Name "hello running health service" -Record $helloHealthRunning.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.stop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.stop"
        $helloStop = Get-LastAgentResponseJson -Method "service.stop"
        Assert-CurrentBootEventId -Name "quick:hello_stop_event_id" -Value $helloStop.body.result.event_id
        if ($helloStop.body.result.service.running) {
            throw "Expected stopped hello service after service.stop"
        }
        $helloStopActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello stop response" `
            -Activation $helloStop.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationStoppedStatus `
            -ExpectedActive $true
        if ($helloStopActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello stop response to cite the same service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello stop service response" -Record $helloStop.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationStoppedStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello stop loader response" -Record $helloStop.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationStoppedStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthStopped = Get-LastAgentResponseJson -Method "service.health"
        Assert-CurrentBootEventId -Name "quick:hello_health_stopped_event_id" -Value $helloHealthStopped.body.result.event_id
        if ($helloHealthStopped.body.result.service.health -ne "stopped" -or -not $helloHealthStopped.body.result.service.loaded -or $helloHealthStopped.body.result.service.running) {
            throw "Expected hello health probe to report stopped while loaded"
        }
        if ($helloHealthStopped.body.result.load_descriptor.source.sha256 -ne $helloDescriptorHash) {
            throw "Expected stopped hello health probe to retain descriptor source evidence"
        }
        if ($helloHealthStopped.body.result.load_descriptor.artifact_load_plan_preflight.preflight_hash -ne $helloLoadPlanPreflightHash) {
            throw "Expected stopped hello health probe to retain artifact load-plan preflight evidence"
        }
        $helloHealthStoppedActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello stopped health response" `
            -Activation $helloHealthStopped.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationStoppedStatus `
            -ExpectedActive $true
        if ($helloHealthStoppedActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello stopped health to retain the service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello stopped health service" -Record $helloHealthStopped.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationStoppedStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
        $helloDrop = Get-LastAgentResponseJson -Method "service.drop"
        Assert-CurrentBootEventId -Name "quick:hello_drop_event_id" -Value $helloDrop.body.result.event_id
        if ($helloDrop.body.result.service.loaded -or $helloDrop.body.result.service.running) {
            throw "Expected dropped hello service after service.drop"
        }
        $helloDropActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello drop response" `
            -Activation $helloDrop.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationClearedStatus `
            -ExpectedActive $false
        if ($helloDropActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello drop response to cite the same service-slot activation hash before cleanup"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello drop service response" -Record $helloDrop.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationClearedStatus -ExpectedActive $false
        & $AssertHelloServiceSlotActivationReference -Name "hello drop loader response" -Record $helloDrop.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationClearedStatus -ExpectedActive $false

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthMissing = Get-LastAgentResponseJson -Method "service.health"
        Assert-CurrentBootEventId -Name "quick:hello_health_missing_event_id" -Value $helloHealthMissing.body.result.event_id
        if ($helloHealthMissing.body.result.service.health -ne "missing" -or $helloHealthMissing.body.result.service.loaded -or $helloHealthMissing.body.result.service.running) {
            throw "Expected hello health probe to report missing after drop"
        }
        & $AssertHelloServiceSlotActivation `
            -Name "hello missing health response" `
            -Activation $helloHealthMissing.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationClearedStatus `
            -ExpectedActive $false | Out-Null
        & $AssertHelloServiceSlotActivationReference -Name "hello missing health service" -Record $helloHealthMissing.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationClearedStatus -ExpectedActive $false

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
        if ($null -ne $hostHelloLoad.body.result.load_descriptor.source.signature_envelope) {
            throw "Host-bound descriptor source must remain hash-bound, not a signed artifact-loader path"
        }
        if ($hostHelloLoad.body.result.load_request.descriptor_source_hash -ne $hostDescriptorHash -or $hostHelloLoad.body.result.loader.descriptor_source_hash -ne $hostDescriptorHash) {
            throw "Expected host-bound load request and loader to cite the host-bound descriptor source hash"
        }
        if ($hostHelloLoad.body.result.load_descriptor.artifact_identity.sha256 -ne $helloArtifactIdentityHash -or -not $hostHelloLoad.body.result.load_descriptor.artifact_identity.signature_envelope.signature_verified) {
            throw "Expected host-bound load to keep the same verified built-in artifact identity"
        }
        if ($hostHelloLoad.body.result.load_descriptor.artifact_identity.content_binding.binding_hash -ne $helloArtifactContentHash -or -not $hostHelloLoad.body.result.load_descriptor.artifact_identity.content_binding.trust_signature_verified) {
            throw "Expected host-bound load to keep the same verified built-in artifact content binding"
        }
        if ($hostHelloLoad.body.result.load_descriptor.artifact_identity.artifact_reference.reference_hash -ne $helloArtifactReferenceHash -or $hostHelloLoad.body.result.load_descriptor.artifact_identity.artifact_reference.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or -not $hostHelloLoad.body.result.load_descriptor.artifact_identity.artifact_reference.trust_signature_verified) {
            throw "Expected host-bound load to keep the same verified built-in artifact byte reference"
        }
        $hostLoadPlanPreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "host-bound hello load descriptor" `
            -Preflight $hostHelloLoad.body.result.load_descriptor.artifact_load_plan_preflight `
            -DescriptorSourceLocator $hostDescriptorLocator `
            -DescriptorSourceHash $hostDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        $hostLoadResponsePreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "host-bound hello load response" `
            -Preflight $hostHelloLoad.body.result.artifact_load_plan_preflight `
            -DescriptorSourceLocator $hostDescriptorLocator `
            -DescriptorSourceHash $hostDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        if ($hostLoadResponsePreflightHash -ne $hostLoadPlanPreflightHash) {
            throw "Expected host-bound hello load response and descriptor to agree on artifact load-plan preflight hash"
        }
        $hostServiceSlotActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "host-bound hello load response" `
            -Activation $hostHelloLoad.body.result.service_slot_activation `
            -DescriptorSourceHash $hostDescriptorHash `
            -PreflightHash $hostLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($hostServiceSlotActivationHash -eq $helloServiceSlotActivationHash) {
            throw "Expected host-bound service-slot activation hash to be derived from the host-bound preflight"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "host-bound hello load request" -Record $hostHelloLoad.body.result.load_request -ExpectedHash $hostLoadPlanPreflightHash -ExpectServiceSlotIntent $true
        & $AssertHelloLoadPlanPreflightReference -Name "host-bound hello service response" -Record $hostHelloLoad.body.result.service -ExpectedHash $hostLoadPlanPreflightHash
        & $AssertHelloLoadPlanPreflightReference -Name "host-bound hello loader response" -Record $hostHelloLoad.body.result.loader -ExpectedHash $hostLoadPlanPreflightHash -ExpectServiceSlotIntent $true
        & $AssertHelloServiceSlotActivationReference -Name "host-bound hello service response" -Record $hostHelloLoad.body.result.service -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "host-bound hello loader response" -Record $hostHelloLoad.body.result.loader -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

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
        if ($hostInventory[0].artifact_identity_hash -ne $helloArtifactIdentityHash -or -not $hostInventory[0].artifact_identity_signature_envelope.signature_verified) {
            throw "Expected host-bound inventory record to cite the verified artifact identity"
        }
        if ($hostInventory[0].artifact_content_binding_hash -ne $helloArtifactContentHash -or $hostInventory[0].artifact_content_source_hash -ne $helloArtifactContentBinding.source_sha256 -or $hostInventory[0].artifact_content_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected host-bound inventory record to cite the verified artifact content binding"
        }
        if ($hostInventory[0].artifact_reference_hash -ne $helloArtifactReferenceHash -or $hostInventory[0].artifact_bytes_sha256 -ne $helloArtifactBytesHash -or $hostInventory[0].artifact_reference_content_binding_hash -ne $helloArtifactContentHash -or $hostInventory[0].artifact_reference_trust_envelope_hash -ne $helloArtifactIdentityEnvelope.envelope_hash) {
            throw "Expected host-bound inventory record to cite the verified artifact byte reference"
        }
        & $AssertHelloLoadPlanPreflightReference -Name "host-bound inventory record" -Record $hostInventory[0] -ExpectedHash $hostLoadPlanPreflightHash
        & $AssertHelloServiceSlotActivationReference -Name "host-bound inventory record" -Record $hostInventory[0] -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $hostHealthRunning = Get-LastAgentResponseJson -Method "service.health"
        Assert-CurrentBootEventId -Name "quick:hello_health_host_running_event_id" -Value $hostHealthRunning.body.result.event_id
        if ($hostHealthRunning.body.result.service.health -ne "healthy" -or -not $hostHealthRunning.body.result.service.running) {
            throw "Expected host-bound hello health probe to report healthy/running"
        }
        if ($hostHealthRunning.body.result.load_descriptor.source.sha256 -ne $hostDescriptorHash -or $hostHealthRunning.body.result.load_descriptor.source.binds_source_hash -ne $helloDescriptorHash) {
            throw "Expected host-bound hello health probe to cite the host-bound source and bound current-image hash"
        }
        if ($hostHealthRunning.body.result.load_descriptor.artifact_identity.sha256 -ne $helloArtifactIdentityHash -or -not $hostHealthRunning.body.result.load_descriptor.artifact_identity.signature_envelope.signature_verified) {
            throw "Expected host-bound hello health probe to cite the verified artifact identity"
        }
        if ($hostHealthRunning.body.result.load_descriptor.artifact_identity.content_binding.binding_hash -ne $helloArtifactContentHash -or -not $hostHealthRunning.body.result.load_descriptor.artifact_identity.content_binding.trust_signature_verified) {
            throw "Expected host-bound hello health probe to cite the verified artifact content binding"
        }
        if ($hostHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.reference_hash -ne $helloArtifactReferenceHash -or $hostHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.artifact_bytes_sha256 -ne $helloArtifactBytesHash -or -not $hostHealthRunning.body.result.load_descriptor.artifact_identity.artifact_reference.trust_signature_verified) {
            throw "Expected host-bound hello health probe to cite the verified artifact byte reference"
        }
        $hostHealthPreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "host-bound hello health descriptor" `
            -Preflight $hostHealthRunning.body.result.load_descriptor.artifact_load_plan_preflight `
            -DescriptorSourceLocator $hostDescriptorLocator `
            -DescriptorSourceHash $hostDescriptorHash `
            -ArtifactIdentityHash $helloArtifactIdentityHash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash
        if ($hostHealthPreflightHash -ne $hostLoadPlanPreflightHash) {
            throw "Expected host-bound hello health descriptor to retain artifact load-plan preflight evidence"
        }
        $hostHealthActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "host-bound hello health response" `
            -Activation $hostHealthRunning.body.result.service_slot_activation `
            -DescriptorSourceHash $hostDescriptorHash `
            -PreflightHash $hostLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($hostHealthActivationHash -ne $hostServiceSlotActivationHash) {
            throw "Expected host-bound health to retain the service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "host-bound health service" -Record $hostHealthRunning.body.result.service -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.stop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.stop"
        $hostStop = Get-LastAgentResponseJson -Method "service.stop"
        if ($hostStop.body.result.loader.descriptor_source_locator -ne $hostDescriptorLocator) {
            throw "Expected host-bound stop event response to cite the active descriptor source"
        }
        & $AssertHelloServiceSlotActivation `
            -Name "host-bound stop response" `
            -Activation $hostStop.body.result.service_slot_activation `
            -DescriptorSourceHash $hostDescriptorHash `
            -PreflightHash $hostLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationStoppedStatus `
            -ExpectedActive $true | Out-Null
        & $AssertHelloServiceSlotActivationReference -Name "host-bound stop loader response" -Record $hostStop.body.result.loader -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationStoppedStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
        $hostDrop = Get-LastAgentResponseJson -Method "service.drop"
        if ($hostDrop.body.result.loader.descriptor_source_locator -ne $hostDescriptorLocator) {
            throw "Expected host-bound drop event response to cite the active descriptor source"
        }
        & $AssertHelloServiceSlotActivation `
            -Name "host-bound drop response" `
            -Activation $hostDrop.body.result.service_slot_activation `
            -DescriptorSourceHash $hostDescriptorHash `
            -PreflightHash $hostLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationClearedStatus `
            -ExpectedActive $false | Out-Null
        & $AssertHelloServiceSlotActivationReference -Name "host-bound drop loader response" -Record $hostDrop.body.result.loader -ExpectedHash $hostServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationClearedStatus -ExpectedActive $false

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
        $helloDescriptorEnvelopeEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "descriptor_source_signature_verified" -and $_.bindings.load_descriptor_source_envelope_hash -eq $helloDescriptorEnvelope.envelope_hash -and $_.bindings.load_descriptor_source_signature_verified })
        if ($helloDescriptorEnvelopeEvents.Count -lt 3) {
            throw "Expected hello lifecycle events to cite the verified descriptor source envelope"
        }
        $helloArtifactIdentityEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "artifact_identity_signature_verified" -and $_.bindings.artifact_identity_hash -eq $helloArtifactIdentityHash -and $_.bindings.artifact_identity_signature_verified })
        if ($helloArtifactIdentityEvents.Count -lt 6) {
            throw "Expected hello lifecycle events to cite the verified artifact identity"
        }
        $helloArtifactContentEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "artifact_content_binding_hash" -and $_.bindings.artifact_content_binding_hash -eq $helloArtifactContentHash -and $_.bindings.artifact_content_trust_signature_verified })
        if ($helloArtifactContentEvents.Count -lt 6) {
            throw "Expected hello lifecycle events to cite the verified artifact content binding"
        }
        $helloArtifactReferenceEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "artifact_reference_hash" -and $_.bindings.artifact_reference_hash -eq $helloArtifactReferenceHash -and $_.bindings.artifact_bytes_sha256 -eq $helloArtifactBytesHash -and $_.bindings.artifact_reference_trust_signature_verified })
        if ($helloArtifactReferenceEvents.Count -lt 6) {
            throw "Expected hello lifecycle events to cite the verified artifact byte reference"
        }
        $helloLoadPlanPreflightEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "artifact_load_plan_preflight_hash" -and $_.bindings.artifact_load_plan_preflight_id -eq $HelloLoadPlanPreflightId -and $_.bindings.artifact_load_plan_preflight_hash -eq $helloLoadPlanPreflightHash -and $_.bindings.artifact_load_plan_preflight_status -eq $HelloLoadPlanPreflightStatus -and $_.bindings.artifact_load_plan_preflight_accepted -and $_.bindings.service_slot_intent_id -eq $HelloServiceSlotIntentId -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($helloLoadPlanPreflightEvents.Count -lt 3) {
            throw "Expected hello lifecycle events to cite the accepted current-image artifact load-plan preflight"
        }
        $helloServiceSlotActivationEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "service_slot_activation_hash" -and $_.bindings.service_slot_activation_id -eq $HelloServiceSlotActivationId -and $_.bindings.service_slot_activation_hash -eq $helloServiceSlotActivationHash -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($helloServiceSlotActivationEvents.Count -lt 3) {
            throw "Expected hello lifecycle events to cite the current-image service-slot activation"
        }
        $helloServiceSlotActivationStatuses = @($helloServiceSlotActivationEvents | ForEach-Object { $_.bindings.service_slot_activation_status } | Select-Object -Unique)
        foreach ($status in @($HelloServiceSlotActivationActiveStatus, $HelloServiceSlotActivationStoppedStatus, $HelloServiceSlotActivationClearedStatus)) {
            if ($helloServiceSlotActivationStatuses -notcontains $status) {
                throw "Expected hello lifecycle service-slot activation status $status"
            }
        }
        $hostDescriptorHashEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "load_descriptor_source_hash" -and $_.bindings.load_descriptor_source_hash -eq $hostDescriptorHash })
        if ($hostDescriptorHashEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the host-bound descriptor source hash"
        }
        $hostDescriptorSourceEvents = @($helloEvents | Where-Object { $_.bindings.load_descriptor_source_locator -eq $hostDescriptorLocator -and $_.bindings.load_descriptor_source_kind -eq $hostDescriptorKind -and $_.bindings.load_descriptor_source_validated -and $_.bindings.binds_source_hash -eq $helloDescriptorHash })
        if ($hostDescriptorSourceEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the bound current-image descriptor source hash"
        }
        $hostLoadPlanPreflightEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "artifact_load_plan_preflight_hash" -and $_.bindings.artifact_load_plan_preflight_hash -eq $hostLoadPlanPreflightHash -and $_.bindings.service_slot_intent_id -eq $HelloServiceSlotIntentId -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($hostLoadPlanPreflightEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the host-bound artifact load-plan preflight"
        }
        $hostServiceSlotActivationEvents = @($helloEvents | Where-Object { @($_.evidence) -contains "service_slot_activation_hash" -and $_.bindings.service_slot_activation_hash -eq $hostServiceSlotActivationHash -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($hostServiceSlotActivationEvents.Count -lt 3) {
            throw "Expected host-bound hello lifecycle events to cite the host-bound service-slot activation"
        }
        $hostServiceSlotActivationStatuses = @($hostServiceSlotActivationEvents | ForEach-Object { $_.bindings.service_slot_activation_status } | Select-Object -Unique)
        foreach ($status in @($HelloServiceSlotActivationActiveStatus, $HelloServiceSlotActivationStoppedStatus, $HelloServiceSlotActivationClearedStatus)) {
            if ($hostServiceSlotActivationStatuses -notcontains $status) {
                throw "Expected host-bound lifecycle service-slot activation status $status"
            }
        }
        $helloHealthEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.ram_only_hello_service.health" -and $_.resource -eq "svc.demo.hello" })
        if ($helloHealthEvents.Count -lt 4) {
            throw "Expected hello health probe events in RAM audit log"
        }
        $helloHealthStateEvents = @($helloHealthEvents | Where-Object { @("healthy", "stopped", "missing") -contains $_.outcome })
        if ($helloHealthStateEvents.Count -lt 3) {
            throw "Expected hello health events to cover healthy, stopped, and missing states"
        }
        $helloHealthEnvelopeEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "descriptor_source_signature_verified" -and $_.bindings.load_descriptor_source_envelope_hash -eq $helloDescriptorEnvelope.envelope_hash -and $_.bindings.load_descriptor_source_signature_verified })
        if ($helloHealthEnvelopeEvents.Count -lt 3) {
            throw "Expected hello health events to cite the verified descriptor source envelope"
        }
        $helloHealthArtifactIdentityEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "artifact_identity_signature_verified" -and $_.bindings.artifact_identity_hash -eq $helloArtifactIdentityHash -and $_.bindings.artifact_identity_signature_verified })
        if ($helloHealthArtifactIdentityEvents.Count -lt 4) {
            throw "Expected hello health events to cite the verified artifact identity"
        }
        $helloHealthArtifactContentEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "artifact_content_binding_hash" -and $_.bindings.artifact_content_binding_hash -eq $helloArtifactContentHash -and $_.bindings.artifact_content_trust_signature_verified })
        if ($helloHealthArtifactContentEvents.Count -lt 4) {
            throw "Expected hello health events to cite the verified artifact content binding"
        }
        $helloHealthArtifactReferenceEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "artifact_reference_hash" -and $_.bindings.artifact_reference_hash -eq $helloArtifactReferenceHash -and $_.bindings.artifact_bytes_sha256 -eq $helloArtifactBytesHash -and $_.bindings.artifact_reference_trust_signature_verified })
        if ($helloHealthArtifactReferenceEvents.Count -lt 4) {
            throw "Expected hello health events to cite the verified artifact byte reference"
        }
        $helloHealthLoadPlanPreflightEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "artifact_load_plan_preflight_hash" -and $_.bindings.artifact_load_plan_preflight_hash -eq $helloLoadPlanPreflightHash -and $_.bindings.artifact_load_plan_preflight_accepted -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($helloHealthLoadPlanPreflightEvents.Count -lt 3) {
            throw "Expected hello health events to cite the current-image artifact load-plan preflight"
        }
        $helloHealthServiceSlotActivationEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "service_slot_activation_hash" -and $_.bindings.service_slot_activation_hash -eq $helloServiceSlotActivationHash -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($helloHealthServiceSlotActivationEvents.Count -lt 3) {
            throw "Expected hello health events to cite the current-image service-slot activation"
        }
        $helloHealthServiceSlotActivationStatuses = @($helloHealthServiceSlotActivationEvents | ForEach-Object { $_.bindings.service_slot_activation_status } | Select-Object -Unique)
        foreach ($status in @($HelloServiceSlotActivationActiveStatus, $HelloServiceSlotActivationStoppedStatus, $HelloServiceSlotActivationClearedStatus)) {
            if ($helloHealthServiceSlotActivationStatuses -notcontains $status) {
                throw "Expected hello health service-slot activation status $status"
            }
        }
        $hostHealthEvents = @($helloHealthEvents | Where-Object { $_.bindings.load_descriptor_source_hash -eq $hostDescriptorHash -and $_.bindings.binds_source_hash -eq $helloDescriptorHash })
        if ($hostHealthEvents.Count -lt 1) {
            throw "Expected host-bound health event to cite the host-bound source and bound current-image hash"
        }
        $hostHealthLoadPlanPreflightEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "artifact_load_plan_preflight_hash" -and $_.bindings.artifact_load_plan_preflight_hash -eq $hostLoadPlanPreflightHash -and $_.bindings.artifact_load_plan_preflight_accepted -and $_.bindings.ram_only_service_slot_id -eq $HelloRamOnlyServiceSlotId })
        if ($hostHealthLoadPlanPreflightEvents.Count -lt 1) {
            throw "Expected host-bound health event to cite the host-bound artifact load-plan preflight"
        }
        $hostHealthServiceSlotActivationEvents = @($helloHealthEvents | Where-Object { @($_.evidence) -contains "service_slot_activation_hash" -and $_.bindings.service_slot_activation_hash -eq $hostServiceSlotActivationHash -and $_.bindings.service_slot_activation_status -eq $HelloServiceSlotActivationActiveStatus })
        if ($hostHealthServiceSlotActivationEvents.Count -lt 1) {
            throw "Expected host-bound health event to cite the host-bound service-slot activation"
        }
        Assert-LogContains -Name "quick:audit_events_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_limit" -Needle '"limit": 32' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_module_load_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_recovery_load_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_kind" -Needle '"kind": "raios.ram_only_hello_service.lifecycle"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_health_kind" -Needle '"kind": "raios.ram_only_hello_service.health"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_resource" -Needle '"resource": "svc.demo.hello"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor" -Needle "load_descriptor.current_boot.svc.demo.hello.v0" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_hash" -Needle '"load_descriptor_source_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_envelope_hash" -Needle '"load_descriptor_source_envelope_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_signature_verified" -Needle '"load_descriptor_source_signature_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_identity_hash" -Needle '"artifact_identity_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_identity_signature_verified" -Needle '"artifact_identity_signature_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_content_binding_hash" -Needle '"artifact_content_binding_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_content_signature_verified" -Needle '"artifact_content_trust_signature_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_reference_hash" -Needle '"artifact_reference_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_bytes_hash" -Needle '"artifact_bytes_sha256": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_artifact_reference_signature_verified" -Needle '"artifact_reference_trust_signature_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_load_plan_preflight_hash" -Needle '"artifact_load_plan_preflight_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_load_plan_preflight_accepted" -Needle '"artifact_load_plan_preflight_accepted": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_ram_only_slot" -Needle '"ram_only_service_slot_id": "ram_only:svc.demo.hello"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_slot_activation_id" -Needle '"service_slot_activation_id": "service_slot_activation.current_boot.svc.demo.hello.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_slot_activation_hash" -Needle '"service_slot_activation_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_slot_activation_status" -Needle '"service_slot_activation_status": "' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_kind" -Needle "current_image_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_source_kind" -Needle "host_bound_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_binds_hash" -Needle '"binds_source_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_validated" -Needle '"load_descriptor_source_validated": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1
