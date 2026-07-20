    Send-AgentCommand -Command "agent module.audit_rollback_availability" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_availability"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_availability_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_availability.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "ledger_schema"; Needle = '"schema": "raios.durable_audit_ledger.v0"' },
        @{ Suffix = "ledger_id"; Needle = '"id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "ledger_reason"; Needle = '"reason": "durable_audit_ledger_missing"' },
        @{ Suffix = "ledger_present_false"; Needle = '"present": false' },
        @{ Suffix = "ledger_provenance_false"; Needle = '"provenance_valid": false' },
        @{ Suffix = "rollback_schema"; Needle = '"schema": "raios.rollback_store.v0"' },
        @{ Suffix = "rollback_id"; Needle = '"id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_reason"; Needle = '"reason": "rollback_store_missing"' },
        @{ Suffix = "availability_status"; Needle = '"availability_status": "missing"' },
        @{ Suffix = "availability_reason"; Needle = '"availability_reason": "durable_audit_ledger_missing_and_rollback_store_missing"' },
        @{ Suffix = "audit_write_missing"; Needle = '"durable_audit_write_missing": true' },
        @{ Suffix = "rollback_install_missing"; Needle = '"rollback_install_missing": true' },
        @{ Suffix = "policy_missing"; Needle = '"durable_write_policy_available": false' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_available": false' },
        @{ Suffix = "not_durable_authority"; Needle = '"retained_hash_refs_are_durable_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackWriteBoundaryEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_boundary requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackWriteBoundaryEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary"
    $moduleAuditRollbackWriteBoundaryEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackWriteBoundaryEnvelope.body.result.accepted -or $moduleAuditRollbackWriteBoundaryEnvelope.body.result.target_method -ne "module.audit_rollback_write_boundary" -or -not ($moduleAuditRollbackWriteBoundaryEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_write_boundary") -or $moduleAuditRollbackWriteBoundaryEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackWriteBoundaryEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackWriteBoundaryEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackWriteBoundaryEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackWriteBoundaryEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_write_boundary without durable rollback authority"
    }
    $moduleAuditRollbackWriteBoundaryEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_write_boundary"
    if ($moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_write_boundary.v0" -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.loads_recovery_artifact -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.preconditions_status -ne "denied_missing_durable_write_boundary" -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.preconditions_reason -ne "durable_audit_write_missing_and_rollback_install_missing" -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.durable_audit_write_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.rollback_install_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.durable_write_policy_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.rollback_install_policy_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.storage_layout_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.append_engine_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.payload_hash_missing -or -not $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.append_intent_missing -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackWriteBoundaryEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_write_boundary to stay read-only, denied, and non-loading"
    }

    $moduleAuditRollbackWriteBoundaryMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_boundary requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackWriteBoundaryMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackWriteBoundaryMismatch.body.result.accepted -or $moduleAuditRollbackWriteBoundaryMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackWriteBoundaryMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_write_boundary envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackWriteBoundaryMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackWriteBoundaryMismatchOffset)
    $moduleAuditRollbackWriteBoundaryMismatchNoDispatch = -not $moduleAuditRollbackWriteBoundaryMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_write_boundary")
    Add-Predicate -Name "protocol:module_audit_rollback_write_boundary_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_write_boundary" -Passed $moduleAuditRollbackWriteBoundaryMismatchNoDispatch -Actual $(if ($moduleAuditRollbackWriteBoundaryMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackWriteBoundaryMismatchNoDispatch) {
        throw "Expected module.audit_rollback_write_boundary capability mismatch to avoid write-boundary dispatch"
    }

    $moduleAuditRollbackAvailabilityEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_availability requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackAvailabilityEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_availability"
    $moduleAuditRollbackAvailabilityEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackAvailabilityEnvelope.body.result.accepted -or $moduleAuditRollbackAvailabilityEnvelope.body.result.target_method -ne "module.audit_rollback_availability" -or -not ($moduleAuditRollbackAvailabilityEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_availability") -or $moduleAuditRollbackAvailabilityEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackAvailabilityEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackAvailabilityEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackAvailabilityEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackAvailabilityEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_availability without durable rollback authority"
    }
    $moduleAuditRollbackAvailabilityEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_availability"
    if ($moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_availability.v0" -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.can_load_now -or $moduleAuditRollbackAvailabilityEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_availability to stay read-only, unavailable, and non-loading"
    }

    $moduleAuditRollbackAvailabilityMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_availability requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackAvailabilityMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackAvailabilityMismatch.body.result.accepted -or $moduleAuditRollbackAvailabilityMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackAvailabilityMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_availability envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackAvailabilityMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackAvailabilityMismatchOffset)
    $moduleAuditRollbackAvailabilityMismatchNoDispatch = -not $moduleAuditRollbackAvailabilityMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_availability")
    Add-Predicate -Name "protocol:module_audit_rollback_availability_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_availability" -Passed $moduleAuditRollbackAvailabilityMismatchNoDispatch -Actual $(if ($moduleAuditRollbackAvailabilityMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackAvailabilityMismatchNoDispatch) {
        throw "Expected module.audit_rollback_availability capability mismatch to avoid availability dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_write_policy" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_write_policy_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_write_policy.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "durable_policy_schema"; Needle = '"schema": "raios.durable_audit_write_policy.v0"' },
        @{ Suffix = "durable_policy_id"; Needle = '"id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "durable_policy_reason"; Needle = '"reason": "durable_write_policy_missing"' },
        @{ Suffix = "durable_policy_present_false"; Needle = '"present": false' },
        @{ Suffix = "durable_policy_binding_false"; Needle = '"binds_retained_evidence": false' },
        @{ Suffix = "durable_policy_availability_false"; Needle = '"binds_availability": false' },
        @{ Suffix = "rollback_policy_schema"; Needle = '"schema": "raios.rollback_install_policy.v0"' },
        @{ Suffix = "rollback_policy_id"; Needle = '"id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_policy_reason"; Needle = '"reason": "rollback_install_policy_missing"' },
        @{ Suffix = "policy_status"; Needle = '"policy_status": "missing"' },
        @{ Suffix = "policy_reason"; Needle = '"policy_reason": "durable_write_policy_missing_and_rollback_install_policy_missing"' },
        @{ Suffix = "policy_missing"; Needle = '"durable_write_policy_missing": true' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_missing": true' },
        @{ Suffix = "policy_authority_false"; Needle = '"retained_hash_refs_are_policy_authority": false' },
        @{ Suffix = "availability_authority_false"; Needle = '"availability_facts_are_policy_authority": false' },
        @{ Suffix = "required_manifest"; Needle = '"module_manifest": "raios.module_manifest_reference.v0"' },
        @{ Suffix = "required_availability"; Needle = '"durable_audit_ledger": "raios.durable_audit_ledger.v0"' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackWritePolicyEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_policy requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackWritePolicyEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy"
    $moduleAuditRollbackWritePolicyEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackWritePolicyEnvelope.body.result.accepted -or $moduleAuditRollbackWritePolicyEnvelope.body.result.target_method -ne "module.audit_rollback_write_policy" -or -not ($moduleAuditRollbackWritePolicyEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_write_policy") -or $moduleAuditRollbackWritePolicyEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackWritePolicyEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackWritePolicyEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackWritePolicyEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackWritePolicyEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_write_policy without durable rollback authority"
    }
    $moduleAuditRollbackWritePolicyEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_write_policy"
    if ($moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_write_policy.v0" -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.can_load_now -or $moduleAuditRollbackWritePolicyEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_write_policy to stay read-only, unavailable, and non-loading"
    }

    $moduleAuditRollbackWritePolicyMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_write_policy requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackWritePolicyMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackWritePolicyMismatch.body.result.accepted -or $moduleAuditRollbackWritePolicyMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackWritePolicyMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_write_policy envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackWritePolicyMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackWritePolicyMismatchOffset)
    $moduleAuditRollbackWritePolicyMismatchNoDispatch = -not $moduleAuditRollbackWritePolicyMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_write_policy")
    Add-Predicate -Name "protocol:module_audit_rollback_write_policy_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_write_policy" -Passed $moduleAuditRollbackWritePolicyMismatchNoDispatch -Actual $(if ($moduleAuditRollbackWritePolicyMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackWritePolicyMismatchNoDispatch) {
        throw "Expected module.audit_rollback_write_policy capability mismatch to avoid write-policy dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_storage_layout" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_storage_layout_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_storage_layout.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "device_schema"; Needle = '"schema": "raios.persistence_device_inventory.v0"' },
        @{ Suffix = "device_id"; Needle = '"id": "storage.persistence_device_inventory.current_boot"' },
        @{ Suffix = "device_reason"; Needle = '"reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "device_present_true"; Needle = '"present": true' },
        @{ Suffix = "storage_controller_observed"; Needle = '"storage_controller_observed": true' },
        @{ Suffix = "block_driver_true"; Needle = '"block_driver_available": true' },
        @{ Suffix = "ahci_schema"; Needle = '"schema": "raios.ahci_controller_probe.v0"' },
        @{ Suffix = "ahci_source"; Needle = '"source": "ahci_abar_mmio_identify_sector0_partition_read_only_scratch_write_readback_target_label_scan"' },
        @{ Suffix = "ahci_observed"; Needle = '"observed": true' },
        @{ Suffix = "ahci_is_ahci"; Needle = '"controller_is_ahci": true' },
        @{ Suffix = "ahci_abar_available"; Needle = '"abar_available": true' },
        @{ Suffix = "ahci_registers_mapped"; Needle = '"registers_mapped": true' },
        @{ Suffix = "ahci_first_port"; Needle = '"implemented": true' },
        @{ Suffix = "block_identity_true"; Needle = '"block_device_identity_available": true' },
        @{ Suffix = "sector_read_true"; Needle = '"sector_read_available": true' },
        @{ Suffix = "ahci_identify"; Needle = '"identify_command_issued": true' },
        @{ Suffix = "ahci_register_write"; Needle = '"controller_register_write_attempted": true' },
        @{ Suffix = "ahci_dma"; Needle = '"dma_started": true' },
        @{ Suffix = "ahci_identify_complete"; Needle = '"identify_completed": true' },
        @{ Suffix = "ahci_identify_status"; Needle = '"identify_status": "identify_complete"' },
        @{ Suffix = "ahci_sector_read_attempted"; Needle = '"sector_read_attempted": true' },
        @{ Suffix = "ahci_sector_read_completed"; Needle = '"sector_read_completed": true' },
        @{ Suffix = "ahci_sector_read_status"; Needle = '"sector_read_status": "sector0_read_complete"' },
        @{ Suffix = "ahci_identity_available"; Needle = '"block_device_identity": {' },
        @{ Suffix = "ahci_identity_sector_size"; Needle = '"logical_sector_size_bytes": 512' },
        @{ Suffix = "ahci_sector0"; Needle = '"sector0_read": {' },
        @{ Suffix = "ahci_sector0_available"; Needle = '"available": true' },
        @{ Suffix = "ahci_sector0_lba"; Needle = '"lba": 0' },
        @{ Suffix = "ahci_sector0_bytes"; Needle = '"byte_count": 512' },
        @{ Suffix = "ahci_sector0_mbr"; Needle = '"mbr_signature": 43605' },
        @{ Suffix = "ahci_sector0_mbr_valid"; Needle = '"mbr_signature_valid": true' },
        @{ Suffix = "ahci_partition_inventory"; Needle = '"partition_inventory": {' },
        @{ Suffix = "ahci_partition_schema"; Needle = '"schema": "raios.boot_partition_inventory.v0"' },
        @{ Suffix = "ahci_partition_source"; Needle = '"source": "sector0_mbr_partition_table"' },
        @{ Suffix = "ahci_partition_available"; Needle = '"available": true' },
        @{ Suffix = "ahci_partition_scheme"; Needle = '"scheme": "mbr_empty"' },
        @{ Suffix = "ahci_partition_count"; Needle = '"entry_count": 0' },
        @{ Suffix = "ahci_partition_bootable_count"; Needle = '"bootable_entry_count": 0' },
        @{ Suffix = "ahci_block_driver"; Needle = '"block_driver": {' },
        @{ Suffix = "ahci_block_driver_schema"; Needle = '"schema": "raios.read_only_block_driver.v0"' },
        @{ Suffix = "ahci_block_driver_id"; Needle = '"id": "block_driver.ahci.read_only.current_boot"' },
        @{ Suffix = "ahci_block_driver_source"; Needle = '"source": "ahci_dma_read_verified_sector0"' },
        @{ Suffix = "ahci_block_driver_available"; Needle = '"available": true' },
        @{ Suffix = "ahci_block_driver_binds_partition"; Needle = '"binds_partition_inventory": true' },
        @{ Suffix = "ahci_block_driver_supports_read"; Needle = '"supports_read": true' },
        @{ Suffix = "ahci_block_driver_supports_write_false"; Needle = '"supports_write": false' },
        @{ Suffix = "scratch_schema"; Needle = '"schema": "raios.scratch_block_region_write_readback.v0"' },
        @{ Suffix = "scratch_id"; Needle = '"id": "scratch.block_region.current_boot.v0"' },
        @{ Suffix = "scratch_test_infra"; Needle = '"test_infrastructure": true' },
        @{ Suffix = "scratch_available"; Needle = '"status": "available"' },
        @{ Suffix = "scratch_reason"; Needle = '"reason": "scratch_write_readback_verified"' },
        @{ Suffix = "scratch_marker"; Needle = '"marker": "RAIOS_SCRATCH_V0"' },
        @{ Suffix = "scratch_region_start"; Needle = '"region_start_lba": 1' },
        @{ Suffix = "scratch_region_count"; Needle = '"region_lba_count": 1' },
        @{ Suffix = "scratch_identity"; Needle = '"device_identity_available": true' },
        @{ Suffix = "scratch_bounds"; Needle = '"region_within_device_bounds": true' },
        @{ Suffix = "scratch_no_boot_overlap"; Needle = '"boot_port_overlap": false' },
        @{ Suffix = "scratch_no_metadata_overlap"; Needle = '"metadata_lba_overlap": false' },
        @{ Suffix = "scratch_no_overlap"; Needle = '"no_boot_or_partition_metadata_overlap": true' },
        @{ Suffix = "scratch_block_write_authority_true"; Needle = '"block_write_authority_available": true' },
        @{ Suffix = "scratch_authority_schema"; Needle = '"schema": "raios.scratch_block_write_authority.v0"' },
        @{ Suffix = "scratch_authority_id"; Needle = '"id": "scratch.block_write_authority.current_boot.v0"' },
        @{ Suffix = "scratch_authority_owner"; Needle = '"owner": "vm_harness.scratch_region.current_boot"' },
        @{ Suffix = "scratch_authorizes_readback"; Needle = '"authorizes_scratch_write_readback": true' },
        @{ Suffix = "scratch_label_found"; Needle = '"label_found": true' },
        @{ Suffix = "scratch_write_attempted"; Needle = '"write_attempted": true' },
        @{ Suffix = "scratch_write_completed"; Needle = '"write_completed": true' },
        @{ Suffix = "scratch_readback_completed"; Needle = '"readback_completed": true' },
        @{ Suffix = "scratch_readback_matches"; Needle = '"readback_matches": true' },
        @{ Suffix = "scratch_no_rollback"; Needle = '"authorizes_audit_rollback": false' },
        @{ Suffix = "scratch_no_append"; Needle = '"authorizes_append": false' },
        @{ Suffix = "scratch_test_write_exercised"; Needle = '"test_write_exercised": true' },
        @{ Suffix = "target_region_raw"; Needle = '"audit_rollback_target_region": {' },
        @{ Suffix = "target_region_raw_marker"; Needle = '"marker": "RAIOS_AUDITRB_V0"' },
        @{ Suffix = "target_region_raw_available"; Needle = '"status": "available"' },
        @{ Suffix = "target_region_raw_reason"; Needle = '"reason": "dedicated_audit_rollback_region_discovered_read_only"' },
        @{ Suffix = "target_region_raw_read"; Needle = '"read_completed": true' },
        @{ Suffix = "target_region_raw_no_scratch_overlap"; Needle = '"scratch_region_overlap": false' },
        @{ Suffix = "target_region_raw_read_only"; Needle = '"read_only": true' },
        @{ Suffix = "target_region_raw_no_write"; Needle = '"write_attempted": false' },
        @{ Suffix = "ahci_reason"; Needle = '"reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "device_stable_true"; Needle = '"stable_identity": true' },
        @{ Suffix = "partition_inventory_true"; Needle = '"partition_inventory_available": true' },
        @{ Suffix = "write_path_false"; Needle = '"write_path_available": false' },
        @{ Suffix = "layout_schema"; Needle = '"schema": "raios.audit_rollback_storage_layout.v0"' },
        @{ Suffix = "layout_id"; Needle = '"id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "layout_reason"; Needle = '"reason": "audit_rollback_storage_layout_missing"' },
        @{ Suffix = "layout_binding_false"; Needle = '"binds_persistence_device": false' },
        @{ Suffix = "audit_region_false"; Needle = '"has_audit_ledger_region": false' },
        @{ Suffix = "rollback_region_false"; Needle = '"has_rollback_store_region": false' },
        @{ Suffix = "append_slots_false"; Needle = '"append_slots_available": false' },
        @{ Suffix = "storage_authority_schema"; Needle = '"schema": "raios.audit_rollback_storage_authority.v0"' },
        @{ Suffix = "storage_authority_id"; Needle = '"id": "storage.authority.audit_rollback.current_boot"' },
        @{ Suffix = "storage_authority_owner"; Needle = '"owner_method": "module.audit_rollback_storage_layout"' },
        @{ Suffix = "storage_authority_append_denied"; Needle = '"authorizes_append": false' },
        @{ Suffix = "audit_append_target"; Needle = '"audit_ledger": {"id": "append.audit_ledger.current_boot", "schema": "raios.audit_record.v0", "available": false}' },
        @{ Suffix = "rollback_append_target"; Needle = '"rollback_store": {"id": "append.rollback_store.current_boot", "schema": "raios.rollback_transaction.v0", "available": false}' },
        @{ Suffix = "target_region_discovery_schema"; Needle = '"schema": "raios.audit_rollback_target_region_discovery.v0"' },
        @{ Suffix = "target_region_discovery_id"; Needle = '"id": "target_region.audit_rollback.current_boot"' },
        @{ Suffix = "target_region_discovery_source"; Needle = '"source": "dedicated_audit_rollback_label_scan"' },
        @{ Suffix = "target_region_discovery_reason"; Needle = '"reason": "dedicated_audit_rollback_region_discovered_read_only"' },
        @{ Suffix = "target_region_candidate"; Needle = '"candidate_region_present": true' },
        @{ Suffix = "target_region_not_scratch"; Needle = '"candidate_region_is_scratch": false' },
        @{ Suffix = "target_region_no_boot_overlap"; Needle = '"candidate_overlaps_boot_metadata": false' },
        @{ Suffix = "target_region_rejects_scratch"; Needle = '"scratch_rejected_as_durable_authority": true' },
        @{ Suffix = "target_region_available"; Needle = '"durable_region_available": true' },
        @{ Suffix = "transaction_writer_owner"; Needle = '"transaction_writer_owner": "module.audit_rollback_append_contract"' },
        @{ Suffix = "observed_block_write_path_false"; Needle = '"block_write_path_available": false' },
        @{ Suffix = "observed_scratch_write_path_true"; Needle = '"scratch_write_path_available": true' },
        @{ Suffix = "observed_scratch_authority_true"; Needle = '"scratch_block_write_authority_available": true' },
        @{ Suffix = "observed_scratch_bounds"; Needle = '"scratch_region_within_device_bounds": true' },
        @{ Suffix = "observed_scratch_no_overlap"; Needle = '"scratch_region_no_boot_or_partition_metadata_overlap": true' },
        @{ Suffix = "observed_scratch_no_rollback"; Needle = '"scratch_region_authorizes_audit_rollback": false' },
        @{ Suffix = "observed_ahci_register_probe"; Needle = '"ahci_register_probe_available": true' },
        @{ Suffix = "observed_block_identity_true"; Needle = '"block_device_identity_available": true' },
        @{ Suffix = "observed_sector_read_true"; Needle = '"sector_read_available": true' },
        @{ Suffix = "observed_block_write_path_reason"; Needle = '"block_write_path_reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "block_write_path_gate_schema"; Needle = '"schema": "raios.block_write_path_authority_gate.v0"' },
        @{ Suffix = "block_write_path_gate_id"; Needle = '"id": "block_write_path.authority.audit_rollback.current_boot"' },
        @{ Suffix = "block_write_path_gate_status"; Needle = '"status": "missing"' },
        @{ Suffix = "block_write_path_gate_driver"; Needle = '"read_only_block_driver_id": "block_driver.ahci.read_only.current_boot"' },
        @{ Suffix = "block_write_path_gate_driver_available"; Needle = '"read_only_block_driver_available": true' },
        @{ Suffix = "block_write_path_gate_driver_read"; Needle = '"read_only_block_driver_supports_read": true' },
        @{ Suffix = "block_write_path_gate_driver_write_false"; Needle = '"read_only_block_driver_supports_write": false' },
        @{ Suffix = "block_write_path_gate_partition"; Needle = '"partition_inventory_scheme": "mbr_empty"' },
        @{ Suffix = "block_write_path_gate_scratch_true"; Needle = '"scratch_write_path_available": true' },
        @{ Suffix = "block_write_path_gate_scratch_authority"; Needle = '"scratch_block_write_authority_available": true' },
        @{ Suffix = "block_write_path_gate_scratch_authority_id"; Needle = '"scratch_block_write_authority_id": "scratch.block_write_authority.current_boot.v0"' },
        @{ Suffix = "block_write_path_gate_scratch_bounds"; Needle = '"scratch_region_within_device_bounds": true' },
        @{ Suffix = "block_write_path_gate_scratch_no_overlap"; Needle = '"scratch_region_no_boot_or_partition_metadata_overlap": true' },
        @{ Suffix = "block_write_path_gate_scratch_id"; Needle = '"scratch_region_id": "scratch.block_region.current_boot.v0"' },
        @{ Suffix = "block_write_path_gate_scratch_status"; Needle = '"scratch_region_status": "available"' },
        @{ Suffix = "block_write_path_gate_scratch_reason"; Needle = '"scratch_region_reason": "scratch_write_readback_verified"' },
        @{ Suffix = "block_write_path_gate_scratch_no_rollback"; Needle = '"scratch_region_authorizes_audit_rollback": false' },
        @{ Suffix = "block_write_path_gate_no_media_write"; Needle = '"authorizes_media_write": false' },
        @{ Suffix = "block_write_path_gate_write_attempted_false"; Needle = '"write_attempted": false' },
        @{ Suffix = "storage_status"; Needle = '"storage_layout_status": "missing"' },
        @{ Suffix = "storage_reason"; Needle = '"storage_layout_reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "storage_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_missing_true"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_storage_authority"; Needle = '"retained_hash_refs_are_storage_authority": false' },
        @{ Suffix = "storage_not_append_authority"; Needle = '"storage_layout_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackStorageLayoutEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_storage_layout requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackStorageLayoutEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout"
    $moduleAuditRollbackStorageLayoutEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackStorageLayoutEnvelope.body.result.accepted -or $moduleAuditRollbackStorageLayoutEnvelope.body.result.target_method -ne "module.audit_rollback_storage_layout" -or -not ($moduleAuditRollbackStorageLayoutEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_storage_layout") -or $moduleAuditRollbackStorageLayoutEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackStorageLayoutEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackStorageLayoutEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackStorageLayoutEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackStorageLayoutEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_storage_layout without durable rollback authority"
    }
    $moduleAuditRollbackStorageLayoutEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_storage_layout"
    if ($moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_storage_layout.v0" -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.policy_result.storage_layout_available -or -not $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.policy_result.storage_layout_missing -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackStorageLayoutEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_storage_layout to stay read-only, missing, and non-loading"
    }

    $moduleAuditRollbackStorageLayoutMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_storage_layout requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackStorageLayoutMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackStorageLayoutMismatch.body.result.accepted -or $moduleAuditRollbackStorageLayoutMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackStorageLayoutMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_storage_layout envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackStorageLayoutMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackStorageLayoutMismatchOffset)
    $moduleAuditRollbackStorageLayoutMismatchNoDispatch = -not $moduleAuditRollbackStorageLayoutMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_storage_layout")
    Add-Predicate -Name "protocol:module_audit_rollback_storage_layout_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_storage_layout" -Passed $moduleAuditRollbackStorageLayoutMismatchNoDispatch -Actual $(if ($moduleAuditRollbackStorageLayoutMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackStorageLayoutMismatchNoDispatch) {
        throw "Expected module.audit_rollback_storage_layout capability mismatch to avoid storage-layout dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_append_engine" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_engine"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_engine_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_engine.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "storage_inputs"; Needle = '"storage_layout_inputs": {' },
        @{ Suffix = "storage_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_missing_true"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "storage_not_engine_authority"; Needle = '"storage_layout_facts_are_append_engine_authority": false' },
        @{ Suffix = "audit_engine_schema"; Needle = '"schema": "raios.audit_ledger_append_engine.v0"' },
        @{ Suffix = "audit_engine_id"; Needle = '"id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_engine_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_engine_reason"; Needle = '"reason": "audit_ledger_append_engine_missing"' },
        @{ Suffix = "audit_engine_present_false"; Needle = '"present": false' },
        @{ Suffix = "audit_engine_storage_binding_false"; Needle = '"binds_storage_layout": false' },
        @{ Suffix = "audit_engine_policy_binding_false"; Needle = '"binds_write_policy": false' },
        @{ Suffix = "audit_engine_append_only_false"; Needle = '"supports_append_only": false' },
        @{ Suffix = "rollback_engine_schema"; Needle = '"schema": "raios.rollback_store_transaction_engine.v0"' },
        @{ Suffix = "rollback_engine_id"; Needle = '"id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_engine_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_engine_reason"; Needle = '"reason": "rollback_store_transaction_engine_missing"' },
        @{ Suffix = "append_engine_status"; Needle = '"append_engine_status": "missing"' },
        @{ Suffix = "append_engine_reason"; Needle = '"append_engine_reason": "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing"' },
        @{ Suffix = "append_engine_available_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "append_engine_missing_true"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "retained_not_engine_authority"; Needle = '"retained_hash_refs_are_append_engine_authority": false' },
        @{ Suffix = "engine_not_append_authority"; Needle = '"append_engine_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackAppendEngineEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_engine requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackAppendEngineEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_engine"
    $moduleAuditRollbackAppendEngineEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackAppendEngineEnvelope.body.result.accepted -or $moduleAuditRollbackAppendEngineEnvelope.body.result.target_method -ne "module.audit_rollback_append_engine" -or -not ($moduleAuditRollbackAppendEngineEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_append_engine") -or $moduleAuditRollbackAppendEngineEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackAppendEngineEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackAppendEngineEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackAppendEngineEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendEngineEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_append_engine without durable rollback authority"
    }
    $moduleAuditRollbackAppendEngineEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_append_engine"
    if ($moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_append_engine.v0" -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.policy_result.append_engine_available -or -not $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.policy_result.append_engine_missing -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackAppendEngineEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_append_engine to stay read-only, missing, and non-loading"
    }

    $moduleAuditRollbackAppendEngineMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_engine requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackAppendEngineMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackAppendEngineMismatch.body.result.accepted -or $moduleAuditRollbackAppendEngineMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackAppendEngineMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_append_engine envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackAppendEngineMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackAppendEngineMismatchOffset)
    $moduleAuditRollbackAppendEngineMismatchNoDispatch = -not $moduleAuditRollbackAppendEngineMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_append_engine")
    Add-Predicate -Name "protocol:module_audit_rollback_append_engine_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_engine" -Passed $moduleAuditRollbackAppendEngineMismatchNoDispatch -Actual $(if ($moduleAuditRollbackAppendEngineMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackAppendEngineMismatchNoDispatch) {
        throw "Expected module.audit_rollback_append_engine capability mismatch to avoid append-engine dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_append_contract" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_contract_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_contract.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "storage_inputs"; Needle = '"storage_layout_inputs": {' },
        @{ Suffix = "storage_input_device"; Needle = '"persistence_device_inventory": {' },
        @{ Suffix = "storage_input_layout"; Needle = '"audit_rollback_storage_layout": {' },
        @{ Suffix = "storage_input_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_input_authority_false"; Needle = '"storage_layout_facts_are_append_authority": false' },
        @{ Suffix = "append_engine_inputs"; Needle = '"append_engine_inputs": {' },
        @{ Suffix = "append_engine_input_audit"; Needle = '"audit_ledger_append_engine": {' },
        @{ Suffix = "append_engine_input_rollback"; Needle = '"rollback_store_transaction_engine": {' },
        @{ Suffix = "append_engine_input_available_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "append_engine_input_authority_false"; Needle = '"append_engine_facts_are_append_authority": false' },
        @{ Suffix = "audit_append_schema"; Needle = '"schema": "raios.audit_ledger_append_envelope.v0"' },
        @{ Suffix = "audit_append_id"; Needle = '"id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_append_storage_schema"; Needle = '"storage_layout_schema": "raios.audit_rollback_storage_layout.v0"' },
        @{ Suffix = "audit_append_storage_id"; Needle = '"storage_layout_id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "audit_append_engine_id"; Needle = '"append_engine_id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_policy_id"; Needle = '"write_policy_id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "audit_append_availability_id"; Needle = '"availability_id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_provenance_schema"; Needle = '"append_contract_provenance_schema": "raios.append_contract_envelope_provenance.v0"' },
        @{ Suffix = "audit_append_reason"; Needle = '"reason": "audit_append_envelope_missing"' },
        @{ Suffix = "audit_append_storage_binding_false"; Needle = '"binds_storage_layout_id": false' },
        @{ Suffix = "audit_append_engine_binding_false"; Needle = '"binds_append_engine_id": false' },
        @{ Suffix = "audit_append_policy_id_binding_false"; Needle = '"binds_write_policy_id": false' },
        @{ Suffix = "audit_append_availability_id_binding_false"; Needle = '"binds_availability_id": false' },
        @{ Suffix = "audit_append_provenance_binding_false"; Needle = '"binds_envelope_provenance": false' },
        @{ Suffix = "storage_layout_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "append_engine_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "rollback_transaction_schema"; Needle = '"schema": "raios.rollback_store_transaction_envelope.v0"' },
        @{ Suffix = "rollback_transaction_id"; Needle = '"id": "append.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_target"; Needle = '"target_schema": "raios.rollback_transaction.v0"' },
        @{ Suffix = "rollback_transaction_engine_id"; Needle = '"append_engine_id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_policy_id"; Needle = '"write_policy_id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_transaction_availability_id"; Needle = '"availability_id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_reason"; Needle = '"reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "append_target_owner_schema"; Needle = '"schema": "raios.audit_rollback_append_target_owner.v0"' },
        @{ Suffix = "append_target_owner_id"; Needle = '"id": "append_target_owner.audit_rollback.current_boot"' },
        @{ Suffix = "append_target_owner_reason"; Needle = '"reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "block_write_path_reason"; Needle = '"block_write_path_reason": "persistence_device_write_path_missing"' },
        @{ Suffix = "block_write_path_gate_schema"; Needle = '"schema": "raios.block_write_path_authority_gate.v0"' },
        @{ Suffix = "block_write_path_gate_id"; Needle = '"id": "block_write_path.authority.audit_rollback.current_boot"' },
        @{ Suffix = "block_write_path_gate_driver"; Needle = '"read_only_block_driver_id": "block_driver.ahci.read_only.current_boot"' },
        @{ Suffix = "block_write_path_gate_partition"; Needle = '"partition_inventory_scheme": "mbr_empty"' },
        @{ Suffix = "block_write_path_gate_scratch_authority"; Needle = '"scratch_block_write_authority_available": true' },
        @{ Suffix = "block_write_path_gate_scratch_no_overlap"; Needle = '"scratch_region_no_boot_or_partition_metadata_overlap": true' },
        @{ Suffix = "block_write_path_gate_no_append"; Needle = '"authorizes_append": false' },
        @{ Suffix = "transaction_writer_readiness_schema"; Needle = '"schema": "raios.audit_rollback_transaction_writer_readiness.v0"' },
        @{ Suffix = "transaction_writer_readiness_id"; Needle = '"id": "transaction_writer.audit_rollback.current_boot"' },
        @{ Suffix = "transaction_writer_readiness_ready_false"; Needle = '"ready": false' },
        @{ Suffix = "scratch_writer_dry_run_schema"; Needle = '"schema": "raios.audit_rollback_transaction_writer_scratch_dry_run.v0"' },
        @{ Suffix = "scratch_writer_dry_run_id"; Needle = '"id": "transaction_writer.scratch_dry_run.audit_rollback.current_boot"' },
        @{ Suffix = "scratch_writer_dry_run_status"; Needle = '"status": "scratch_range_ready_not_durable_authority"' },
        @{ Suffix = "scratch_writer_dry_run_reason"; Needle = '"reason": "scratch_write_authority_verified_current_boot"' },
        @{ Suffix = "scratch_writer_dry_run_authority"; Needle = '"source_authority_id": "scratch.block_write_authority.current_boot.v0"' },
        @{ Suffix = "scratch_writer_dry_run_region"; Needle = '"source_region_id": "scratch.block_region.current_boot.v0"' },
        @{ Suffix = "scratch_writer_dry_run_start"; Needle = '"target_region_start_lba": 1' },
        @{ Suffix = "scratch_writer_dry_run_count"; Needle = '"target_region_lba_count": 1' },
        @{ Suffix = "scratch_writer_dry_run_bytes"; Needle = '"target_byte_count": 512' },
        @{ Suffix = "scratch_writer_dry_run_owned"; Needle = '"target_range_scratch_owned": true' },
        @{ Suffix = "scratch_writer_dry_run_bounds"; Needle = '"target_range_within_device_bounds": true' },
        @{ Suffix = "scratch_writer_dry_run_no_overlap"; Needle = '"target_range_no_boot_or_partition_metadata_overlap": true' },
        @{ Suffix = "scratch_writer_dry_run_ready"; Needle = '"target_range_ready": true' },
        @{ Suffix = "scratch_writer_dry_run_audit_target"; Needle = '"audit_ledger_target_id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "scratch_writer_dry_run_rollback_target"; Needle = '"rollback_store_target_id": "append.rollback_store.current_boot"' },
        @{ Suffix = "scratch_writer_dry_run_no_append"; Needle = '"authorizes_append": false' },
        @{ Suffix = "scratch_writer_dry_run_no_write"; Needle = '"write_attempted": false' },
        @{ Suffix = "block_write_path_false"; Needle = '"block_write_path_available": false' },
        @{ Suffix = "contract_status"; Needle = '"append_contract_status": "missing"' },
        @{ Suffix = "contract_reason"; Needle = '"append_contract_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' },
        @{ Suffix = "storage_layout_missing"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_append_authority"; Needle = '"retained_hash_refs_are_append_authority": false' },
        @{ Suffix = "policy_not_append_authority"; Needle = '"policy_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackAppendContractEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_contract requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackAppendContractEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract"
    $moduleAuditRollbackAppendContractEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackAppendContractEnvelope.body.result.accepted -or $moduleAuditRollbackAppendContractEnvelope.body.result.target_method -ne "module.audit_rollback_append_contract" -or -not ($moduleAuditRollbackAppendContractEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_append_contract") -or $moduleAuditRollbackAppendContractEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackAppendContractEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackAppendContractEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackAppendContractEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendContractEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_append_contract without durable rollback authority"
    }
    $moduleAuditRollbackAppendContractEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_append_contract"
    if ($moduleAuditRollbackAppendContractEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_append_contract.v0" -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.append_contract_status -ne "missing" -or -not $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.audit_append_missing -or -not $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.rollback_transaction_missing -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.storage_layout_available -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.append_engine_available -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackAppendContractEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_append_contract to stay read-only, missing, and non-loading"
    }

    $moduleAuditRollbackAppendContractMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_contract requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackAppendContractMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackAppendContractMismatch.body.result.accepted -or $moduleAuditRollbackAppendContractMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackAppendContractMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_append_contract envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackAppendContractMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackAppendContractMismatchOffset)
    $moduleAuditRollbackAppendContractMismatchNoDispatch = -not $moduleAuditRollbackAppendContractMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_append_contract")
    Add-Predicate -Name "protocol:module_audit_rollback_append_contract_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_contract" -Passed $moduleAuditRollbackAppendContractMismatchNoDispatch -Actual $(if ($moduleAuditRollbackAppendContractMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackAppendContractMismatchNoDispatch) {
        throw "Expected module.audit_rollback_append_contract capability mismatch to avoid append-contract dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_append_payload_hash" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_payload_hash"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_payload_hash_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_payload_hash.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "retained_inputs"; Needle = '"retained_payload_inputs": {' },
        @{ Suffix = "audit_event"; Needle = '"audit_rollback": {"event_id": "event.current_boot.' },
        @{ Suffix = "service_slot_event"; Needle = '"service_slot_reservation": {"event_id": "event.current_boot.' },
        @{ Suffix = "audit_hash_input"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_input"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "inventory_hash_input"; Needle = "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" },
        @{ Suffix = "reservation_hash_input"; Needle = "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "slot_id_input"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "payload_facts"; Needle = '"append_payload_hash_facts": {' },
        @{ Suffix = "audit_payload_schema"; Needle = '"schema": "raios.audit_record_append_payload_hash_envelope.v0"' },
        @{ Suffix = "audit_payload_id"; Needle = '"id": "append_payload.audit_record.current_boot"' },
        @{ Suffix = "audit_payload_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_payload_contract"; Needle = '"append_contract_id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "payload_hash_present"; Needle = '"payload_hash": "sha256:' },
        @{ Suffix = "audit_source_payload_hash"; Needle = "`"source_payload_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "audit_payload_reason"; Needle = '"reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "binds_retained"; Needle = '"binds_retained_audit_rollback": true' },
        @{ Suffix = "binds_slot"; Needle = '"binds_service_slot_reservation": true' },
        @{ Suffix = "binds_request"; Needle = '"binds_pre_load_write_request": true' },
        @{ Suffix = "binds_contract_id"; Needle = '"binds_append_contract_id": true' },
        @{ Suffix = "binds_payload"; Needle = '"binds_payload_hash": true' },
        @{ Suffix = "retained_available"; Needle = '"retained_audit_rollback_available": true' },
        @{ Suffix = "service_slot_available"; Needle = '"service_slot_reservation_available": true' },
        @{ Suffix = "append_contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "rollback_payload_schema"; Needle = '"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0"' },
        @{ Suffix = "rollback_payload_id"; Needle = '"id": "append_payload.rollback_transaction.current_boot"' },
        @{ Suffix = "rollback_payload_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_source_payload_hash"; Needle = "`"source_payload_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "rollback_payload_reason"; Needle = '"reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "payload_status"; Needle = '"payload_hash_status": "missing"' },
        @{ Suffix = "payload_reason"; Needle = '"payload_hash_reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "retained_available_result"; Needle = '"retained_evidence_available": true' },
        @{ Suffix = "payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "payload_missing_true"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "not_payload_authority"; Needle = '"payload_hash_envelopes_are_append_intent_authority": false' },
        @{ Suffix = "retained_not_payload_authority"; Needle = '"retained_hash_refs_are_payload_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackAppendPayloadHashEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_payload_hash requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackAppendPayloadHashEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_payload_hash"
    $moduleAuditRollbackAppendPayloadHashEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.accepted -or $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.target_method -ne "module.audit_rollback_append_payload_hash" -or -not ($moduleAuditRollbackAppendPayloadHashEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_append_payload_hash") -or $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendPayloadHashEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_append_payload_hash without durable rollback authority"
    }
    $moduleAuditRollbackAppendPayloadHashEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_append_payload_hash"
    if ($moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_append_payload_hash.v0" -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.payload_hash_status -ne "missing" -or -not $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.retained_evidence_available -or -not $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.service_slot_reservation_available -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.append_contract_available -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.payload_hash_available -or -not $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.payload_hash_missing -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackAppendPayloadHashEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_append_payload_hash to stay read-only, missing, and non-loading"
    }

    $moduleAuditRollbackAppendPayloadHashMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_payload_hash requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackAppendPayloadHashMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackAppendPayloadHashMismatch.body.result.accepted -or $moduleAuditRollbackAppendPayloadHashMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackAppendPayloadHashMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_append_payload_hash envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackAppendPayloadHashMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackAppendPayloadHashMismatchOffset)
    $moduleAuditRollbackAppendPayloadHashMismatchNoDispatch = -not $moduleAuditRollbackAppendPayloadHashMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_append_payload_hash")
    Add-Predicate -Name "protocol:module_audit_rollback_append_payload_hash_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_payload_hash" -Passed $moduleAuditRollbackAppendPayloadHashMismatchNoDispatch -Actual $(if ($moduleAuditRollbackAppendPayloadHashMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackAppendPayloadHashMismatchNoDispatch) {
        throw "Expected module.audit_rollback_append_payload_hash capability mismatch to avoid append-payload-hash dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_append_intent" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_intent"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_intent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_intent.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "audit_contract_input"; Needle = '"audit_append_envelope": {"schema": "raios.audit_ledger_append_envelope.v0", "status": "missing", "reason": "audit_append_envelope_missing"' },
        @{ Suffix = "rollback_contract_input"; Needle = '"rollback_transaction_envelope": {"schema": "raios.rollback_store_transaction_envelope.v0", "status": "missing", "reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "contract_not_intent_authority"; Needle = '"append_contract_facts_are_append_intent_authority": false' },
        @{ Suffix = "payload_inputs"; Needle = '"append_payload_hash_inputs": {' },
        @{ Suffix = "audit_payload_input"; Needle = '"audit_record_append_payload_hash": {"schema": "raios.audit_record_append_payload_hash_envelope.v0", "status": "missing", "reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "rollback_payload_input"; Needle = '"rollback_transaction_append_payload_hash": {"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0", "status": "missing", "reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "payload_hash_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "payload_not_intent_authority"; Needle = '"payload_hash_envelopes_are_append_intent_authority": false' },
        @{ Suffix = "append_intent_facts"; Needle = '"append_intent_facts": {' },
        @{ Suffix = "audit_intent_schema"; Needle = '"schema": "raios.audit_record_append_intent.v0"' },
        @{ Suffix = "audit_intent_id"; Needle = '"id": "append_intent.audit_record.current_boot"' },
        @{ Suffix = "audit_intent_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_intent_contract_id"; Needle = '"append_contract_id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "audit_intent_engine_id"; Needle = '"append_engine_id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_intent_storage_id"; Needle = '"storage_layout_id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "audit_intent_policy_id"; Needle = '"write_policy_id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "audit_intent_availability_id"; Needle = '"availability_id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "intent_provenance_schema"; Needle = '"intent_provenance_schema": "raios.append_intent_provenance.v0"' },
        @{ Suffix = "payload_hash_schema"; Needle = '"payload_hash_schema": "raios.append_intent_payload_hash.v0"' },
        @{ Suffix = "payload_hash_envelope_schema"; Needle = '"payload_hash_envelope_schema": "raios.append_payload_hash_envelope.canonical.v0"' },
        @{ Suffix = "audit_intent_reason"; Needle = '"reason": "audit_record_append_intent_missing"' },
        @{ Suffix = "audit_contract_binding_false"; Needle = '"binds_append_contract_id": false' },
        @{ Suffix = "audit_engine_binding_false"; Needle = '"binds_append_engine_id": false' },
        @{ Suffix = "audit_storage_binding_false"; Needle = '"binds_storage_layout_id": false' },
        @{ Suffix = "audit_policy_binding_false"; Needle = '"binds_write_policy_id": false' },
        @{ Suffix = "audit_availability_binding_false"; Needle = '"binds_availability_id": false' },
        @{ Suffix = "audit_payload_binding_false"; Needle = '"binds_payload_hash": false' },
        @{ Suffix = "audit_provenance_binding_false"; Needle = '"binds_intent_provenance": false' },
        @{ Suffix = "audit_payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "rollback_intent_schema"; Needle = '"schema": "raios.rollback_transaction_append_intent.v0"' },
        @{ Suffix = "rollback_intent_id"; Needle = '"id": "append_intent.rollback_transaction.current_boot"' },
        @{ Suffix = "rollback_intent_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_intent_contract_id"; Needle = '"append_contract_id": "append.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_engine_id"; Needle = '"append_engine_id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_policy_id"; Needle = '"write_policy_id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_intent_availability_id"; Needle = '"availability_id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_reason"; Needle = '"reason": "rollback_transaction_append_intent_missing"' },
        @{ Suffix = "intent_status"; Needle = '"append_intent_status": "missing"' },
        @{ Suffix = "intent_reason"; Needle = '"append_intent_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' },
        @{ Suffix = "intent_available_false"; Needle = '"append_intent_available": false' },
        @{ Suffix = "intent_missing_true"; Needle = '"append_intent_missing": true' },
        @{ Suffix = "payload_missing_true"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "intent_not_writer_authority"; Needle = '"append_intent_facts_are_writer_authority": false' },
        @{ Suffix = "retained_not_intent_authority"; Needle = '"retained_hash_refs_are_append_intent_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $moduleAuditRollbackAppendIntentEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_intent requested_capability=cap.module.grant_diagnostic.read classification=local_only"
    Send-AgentCommand -Command $moduleAuditRollbackAppendIntentEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_intent"
    $moduleAuditRollbackAppendIntentEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if (-not $moduleAuditRollbackAppendIntentEnvelope.body.result.accepted -or $moduleAuditRollbackAppendIntentEnvelope.body.result.target_method -ne "module.audit_rollback_append_intent" -or -not ($moduleAuditRollbackAppendIntentEnvelope.body.result.allowed_target_methods -contains "module.audit_rollback_append_intent") -or $moduleAuditRollbackAppendIntentEnvelope.body.result.allowed_requested_capability -ne "cap.module.grant_diagnostic.read" -or -not $moduleAuditRollbackAppendIntentEnvelope.body.result.dispatches_existing_agent_method -or $moduleAuditRollbackAppendIntentEnvelope.body.result.writes_durable_audit_log -or $moduleAuditRollbackAppendIntentEnvelope.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendIntentEnvelope.body.result.grants_broad_mutation) {
        throw "Expected agent command envelope to accept and dispatch module.audit_rollback_append_intent without durable rollback authority"
    }
    $moduleAuditRollbackAppendIntentEnvelopeResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_append_intent"
    if ($moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.schema -ne "raios.module_audit_rollback_append_intent.v0" -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.classification -ne "local_only" -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.writes_enabled -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.creates_durable_audit_records -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.creates_rollback_plans -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.installs_rollback_plan -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.append_intent_status -ne "missing" -or -not $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.audit_intent_missing -or -not $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.rollback_intent_missing -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.append_contract_available -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.payload_hash_available -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.append_intent_available -or -not $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.append_intent_missing -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.policy_result.can_load_now -or $moduleAuditRollbackAppendIntentEnvelopeResponse.body.result.load_attempted) {
        throw "Expected enveloped module.audit_rollback_append_intent to stay read-only, missing, and non-loading"
    }

    $moduleAuditRollbackAppendIntentMismatchOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.audit_rollback_append_intent requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
    $moduleAuditRollbackAppendIntentMismatch = Get-LastAgentResponseJson -Method "agent.command_envelope"
    if ($moduleAuditRollbackAppendIntentMismatch.body.result.accepted -or $moduleAuditRollbackAppendIntentMismatch.body.result.reason -ne "requested_capability_denied" -or $moduleAuditRollbackAppendIntentMismatch.body.result.dispatches_existing_agent_method) {
        throw "Expected module.audit_rollback_append_intent envelope with wrong capability to be denied before dispatch"
    }
    $moduleAuditRollbackAppendIntentMismatchAfter = (Get-SerialLogContent -Path $SerialLog).Substring($moduleAuditRollbackAppendIntentMismatchOffset)
    $moduleAuditRollbackAppendIntentMismatchNoDispatch = -not $moduleAuditRollbackAppendIntentMismatchAfter.Contains("RAIOS_AGENT_END module.audit_rollback_append_intent")
    Add-Predicate -Name "protocol:module_audit_rollback_append_intent_envelope_mismatch_no_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END module.audit_rollback_append_intent" -Passed $moduleAuditRollbackAppendIntentMismatchNoDispatch -Actual $(if ($moduleAuditRollbackAppendIntentMismatchNoDispatch) { "absent" } else { "found" })
    if (-not $moduleAuditRollbackAppendIntentMismatchNoDispatch) {
        throw "Expected module.audit_rollback_append_intent capability mismatch to avoid append-intent dispatch"
    }

    Send-AgentCommand -Command "agent module.audit_rollback_write_boundary" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary"
    Assert-LogContainsFields -NamePrefix "protocol:module_write_boundary_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_write_boundary.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_recovery_artifact"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "request_schema"; Needle = '"schema": "raios.module_pre_load_audit_rollback_write_request.v0"' },
        @{ Suffix = "availability_inputs"; Needle = '"availability_inputs": {' },
        @{ Suffix = "ledger_availability_input"; Needle = '"durable_audit_ledger": {"schema": "raios.durable_audit_ledger.v0", "status": "missing", "reason": "durable_audit_ledger_missing"' },
        @{ Suffix = "rollback_availability_input"; Needle = '"rollback_store": {"schema": "raios.rollback_store.v0", "status": "missing", "reason": "rollback_store_missing"' },
        @{ Suffix = "policy_inputs"; Needle = '"policy_inputs": {' },
        @{ Suffix = "durable_policy_input"; Needle = '"durable_write_policy": {"schema": "raios.durable_audit_write_policy.v0", "status": "missing", "reason": "durable_write_policy_missing"' },
        @{ Suffix = "rollback_policy_input"; Needle = '"rollback_install_policy": {"schema": "raios.rollback_install_policy.v0", "status": "missing", "reason": "rollback_install_policy_missing"' },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "audit_append_input"; Needle = '"audit_append_envelope": {"schema": "raios.audit_ledger_append_envelope.v0", "status": "missing", "reason": "audit_append_envelope_missing"' },
        @{ Suffix = "rollback_transaction_input"; Needle = '"rollback_transaction_envelope": {"schema": "raios.rollback_store_transaction_envelope.v0", "status": "missing", "reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "append_payload_hash_inputs"; Needle = '"append_payload_hash_inputs": {' },
        @{ Suffix = "audit_payload_hash_input"; Needle = '"audit_record_append_payload_hash": {"schema": "raios.audit_record_append_payload_hash_envelope.v0", "status": "missing", "reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "rollback_payload_hash_input"; Needle = '"rollback_transaction_append_payload_hash": {"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0", "status": "missing", "reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "append_payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "append_payload_not_writer_authority"; Needle = '"payload_hash_envelopes_are_writer_authority": false' },
        @{ Suffix = "append_intent_inputs"; Needle = '"append_intent_inputs": {' },
        @{ Suffix = "audit_append_intent_input"; Needle = '"audit_record_append_intent": {"schema": "raios.audit_record_append_intent.v0", "status": "missing", "reason": "audit_record_append_intent_missing"' },
        @{ Suffix = "rollback_append_intent_input"; Needle = '"rollback_transaction_append_intent": {"schema": "raios.rollback_transaction_append_intent.v0", "status": "missing", "reason": "rollback_transaction_append_intent_missing"' },
        @{ Suffix = "append_intent_available_false"; Needle = '"append_intent_available": false' },
        @{ Suffix = "append_intent_not_writer_authority"; Needle = '"append_intent_facts_are_writer_authority": false' },
        @{ Suffix = "denial_schema"; Needle = '"schema": "raios.module_audit_rollback_write_denial_evidence.v0"' },
        @{ Suffix = "validation_status"; Needle = '"validation_status": "denied_missing_durable_write_boundary"' },
        @{ Suffix = "validation_reason"; Needle = '"validation_reason": "durable_audit_write_missing_and_rollback_install_missing"' },
        @{ Suffix = "audit_write_missing"; Needle = '"reason": "durable_audit_write_missing"' },
        @{ Suffix = "rollback_install_missing"; Needle = '"reason": "rollback_install_missing"' },
        @{ Suffix = "durable_policy_missing"; Needle = '"durable_write_policy_missing": true' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_missing": true' },
        @{ Suffix = "audit_append_missing"; Needle = '"audit_append_status": "missing"' },
        @{ Suffix = "rollback_transaction_missing"; Needle = '"rollback_transaction_status": "missing"' },
        @{ Suffix = "audit_payload_hash_missing"; Needle = '"audit_append_payload_hash_status": "missing"' },
        @{ Suffix = "rollback_payload_hash_missing"; Needle = '"rollback_transaction_append_payload_hash_status": "missing"' },
        @{ Suffix = "audit_append_intent_missing"; Needle = '"audit_append_intent_status": "missing"' },
        @{ Suffix = "rollback_transaction_intent_missing"; Needle = '"rollback_transaction_append_intent_status": "missing"' },
        @{ Suffix = "append_intent_missing"; Needle = '"append_intent_missing": true' },
        @{ Suffix = "payload_missing"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "storage_layout_missing"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_append_authority"; Needle = '"retained_hash_refs_are_append_authority": false' },
        @{ Suffix = "not_payload_authority"; Needle = '"retained_hash_refs_are_payload_authority": false' },
        @{ Suffix = "not_append_intent_authority"; Needle = '"retained_hash_refs_are_append_intent_authority": false' },
        @{ Suffix = "not_durable_authority"; Needle = '"retained_hash_refs_are_durable_authority": false' },
        @{ Suffix = "manifest_event"; Needle = '"module_manifest": {"event_id": "event.current_boot.' },
        @{ Suffix = "artifact_event"; Needle = '"candidate_artifact": {"event_id": "event.current_boot.' },
        @{ Suffix = "vm_report_event"; Needle = '"vm_test_report": {"event_id": "event.current_boot.' },
        @{ Suffix = "grant_event"; Needle = '"computed_capability_grant": {"event_id": "event.current_boot.' },
        @{ Suffix = "attestation_event"; Needle = '"local_attestation": {"event_id": "event.current_boot.' },
        @{ Suffix = "approval_event"; Needle = '"local_approval": {"event_id": "event.current_boot.' },
        @{ Suffix = "audit_event"; Needle = '"audit_rollback": {"event_id": "event.current_boot.' },
        @{ Suffix = "service_slot_event"; Needle = '"service_slot_reservation": {"event_id": "event.current_boot.' },
        @{ Suffix = "manifest_hash"; Needle = "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" },
        @{ Suffix = "artifact_hash"; Needle = "`"candidate_artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" },
        @{ Suffix = "vm_report_hash"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "attestation_hash"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "approval_hash"; Needle = "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "service_slot_hash"; Needle = "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "slot_id"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
