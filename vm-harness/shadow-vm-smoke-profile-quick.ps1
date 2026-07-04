        $HelloLoadPlanPreflightSchema = "raios.current_boot_artifact_load_plan_preflight.v0"
        $HelloLoadPlanPreflightId = "artifact_load_plan_preflight.current_boot.svc.demo.hello.v0"
        $HelloLoadPlanPreflightStatus = "accepted_builtin_current_boot_only"
        $HelloArtifactIdentityId = "builtin_artifact_identity.svc.demo.hello.v0"
        $HelloArtifactIdentityV2Id = "builtin_artifact_identity.svc.demo.hello.v2"
        $HelloServiceSlotIntentId = "service_slot_intent.current_boot.svc.demo.hello.v0"
        $HelloRamOnlyServiceSlotId = "ram_only:svc.demo.hello"
        $HelloServiceSlotActivationSchema = "raios.ram_only_service_slot_activation.v0"
        $HelloServiceSlotActivationId = "service_slot_activation.current_boot.svc.demo.hello.v0"
        $HelloServiceSlotActivationActiveStatus = "active_current_boot"
        $HelloServiceSlotActivationStoppedStatus = "stopped_current_boot"
        $HelloServiceSlotActivationClearedStatus = "cleared_current_boot"
        $HelloServiceSlotActivationMissingStatus = "missing_current_boot"
        $HelloStateSchema = "raios.ram_only_hello_service_state.v0"
        $HelloStateId = "hello_state.current_boot.svc.demo.hello.v0"
        $HelloStateMigrationSchema = "raios.ram_only_hello_service_state_migration.v0"
        $HelloStateMigrationId = "hello_state_migration.current_boot.svc.demo.hello.v0"
        $HelloHotSwapProbationSchema = "raios.ram_only_hello_service_hot_swap_probation.v0"
        $HelloHotSwapProbationId = "hello_hot_swap_probation.current_boot.svc.demo.hello.v0"
        $HelloHotSwapProbationStatus = "active_current_boot_probation"
        $HelloRollbackPreviewSchema = "raios.ram_only_hello_service_rollback_preview.v0"
        $HelloRollbackPreviewId = "hello_rollback_preview.current_boot.svc.demo.hello.v0"
        $HelloRollbackPreviewStatus = "preview_only_current_boot"
        $HelloRollbackApplySchema = "raios.ram_only_hello_service_rollback_apply.v0"
        $HelloRollbackApplyId = "hello_rollback_apply.current_boot.svc.demo.hello.v0"
        $HelloRollbackApplyStatus = "denied_missing_rollback_apply_authority"
        $HelloRollbackTransactionPreflightSchema = "raios.ram_only_hello_service_rollback_transaction_preflight.v0"
        $HelloRollbackTransactionPreflightId = "hello_rollback_transaction_preflight.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionPreflightStatus = "denied_missing_write_authorities"
        $HelloRollbackWriteAuthorityGateSchema = "raios.ram_only_hello_service_rollback_write_authority_gate.v0"
        $HelloRollbackWriteAuthorityGateId = "hello_rollback_write_authority_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackWriteAuthorityGateStatus = "denied_missing_durable_write_authority"
        $HelloRollbackAppendIntentGateSchema = "raios.ram_only_hello_service_rollback_append_intent_gate.v0"
        $HelloRollbackAppendIntentGateId = "hello_rollback_append_intent_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackAppendIntentGateStatus = "denied_missing_rollback_transaction_append_authority"
        $HelloRollbackPayloadEnvelopeGateSchema = "raios.ram_only_hello_service_rollback_payload_envelope_gate.v0"
        $HelloRollbackPayloadEnvelopeGateId = "hello_rollback_payload_envelope_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackPayloadEnvelopeGateStatus = "denied_missing_rollback_transaction_writer"
        $HelloRollbackTransactionPayloadSchema = "raios.rollback_transaction.v0"
        $HelloRollbackTransactionPayloadId = "rollback_transaction.proposed.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionPayloadStatus = "proposed_not_appended_current_boot"
        $HelloRollbackTransactionWriterStorageAuthorityGateSchema = "raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0"
        $HelloRollbackTransactionWriterStorageAuthorityGateId = "hello_rollback_transaction_writer_storage_authority_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionWriterStorageAuthorityGateStatus = "denied_missing_transaction_writer_storage_authority"
        $HelloRollbackTransactionWriterStorageFoundationSchema = "raios.module_audit_rollback_append_contract.v0"
        $HelloRollbackTransactionWriterStorageFoundationOwner = "module.audit_rollback_append_contract"
        $HelloRollbackTransactionWriterStorageAuthorityId = "storage.authority.audit_rollback.current_boot"
        $HelloRollbackTransactionWriterStorageAuthorityOwner = "module.audit_rollback_storage_layout"
        $HelloRollbackTransactionWriterStorageAuditTargetId = "append.audit_ledger.current_boot"
        $HelloRollbackTransactionWriterStorageAuditTargetSchema = "raios.audit_record.v0"
        $HelloRollbackTransactionWriterStorageTargetId = "append.rollback_store.current_boot"
        $HelloRollbackTransactionWriterStorageTargetSchema = "raios.rollback_transaction.v0"
        $HelloRollbackTransactionWriterStorageWriterOwner = "module.audit_rollback_append_contract"
        $HelloRollbackBlockWritePathGateSchema = "raios.block_write_path_authority_gate.v0"
        $HelloRollbackBlockWritePathGateId = "block_write_path.authority.audit_rollback.current_boot"
        $HelloRollbackBlockWritePathGateStatus = "missing"
        $HelloRollbackAhciProbeSource = "ahci_abar_mmio_identify_sector0_partition_read_only_scratch_write_readback_target_label_scan"
        $HelloRollbackScratchBlockRegionSchema = "raios.scratch_block_region_write_readback.v0"
        $HelloRollbackScratchBlockRegionId = "scratch.block_region.current_boot.v0"
        $HelloRollbackScratchBlockWriteAuthoritySchema = "raios.scratch_block_write_authority.v0"
        $HelloRollbackScratchBlockWriteAuthorityId = "scratch.block_write_authority.current_boot.v0"
        $HelloRollbackReadOnlyBlockDriverId = "block_driver.ahci.read_only.current_boot"
        $HelloRollbackPartitionInventoryScheme = "mbr_empty"
        $HelloRollbackAppendTargetOwnerSchema = "raios.audit_rollback_append_target_owner.v0"
        $HelloRollbackAppendTargetOwnerId = "append_target_owner.audit_rollback.current_boot"
        $HelloRollbackAppendTargetOwnerStatus = "missing"
        $HelloRollbackAppendTargetOwnerReason = "persistence_device_write_path_missing"
        $HelloRollbackTransactionWriterReadinessSchema = "raios.audit_rollback_transaction_writer_readiness.v0"
        $HelloRollbackTransactionWriterReadinessId = "transaction_writer.audit_rollback.current_boot"
        $HelloRollbackTransactionWriterReadinessStatus = "missing"
        $HelloRollbackTransactionWriterReadinessReason = "persistence_device_write_path_missing"
        $HelloRollbackScratchWriterDryRunSchema = "raios.audit_rollback_transaction_writer_scratch_dry_run.v0"
        $HelloRollbackScratchWriterDryRunId = "transaction_writer.scratch_dry_run.audit_rollback.current_boot"
        $HelloRollbackScratchWriterDryRunStatus = "scratch_range_ready_not_durable_authority"
        $HelloRollbackScratchWriterDryRunReason = "scratch_write_authority_verified_current_boot"
        $HelloRollbackTargetRegionWriterContractSchema = "raios.audit_rollback_target_region_writer_contract.v0"
        $HelloRollbackTargetRegionWriterContractId = "target_region_writer_contract.audit_rollback.current_boot"
        $HelloRollbackTargetRegionWriterContractStatus = "target_region_ready_not_write_authority"
        $HelloRollbackTargetRegionWriterContractReason = "target_region_read_only_missing_media_write_authority"
        $HelloRollbackTargetRegionMediaWritePolicyPreflightSchema = "raios.audit_rollback_target_region_media_write_policy_preflight.v0"
        $HelloRollbackTargetRegionMediaWritePolicyPreflightId = "target_region_media_write_policy_preflight.audit_rollback.current_boot"
        $HelloRollbackTargetRegionMediaWritePolicyPreflightStatus = "denied_missing_media_write_authority_and_durable_audit_policy"
        $HelloRollbackTargetRegionMediaWritePolicyPreflightReason = "target_region_contract_ready_policy_or_write_authority_missing"
        $HelloRollbackMediaWriteAuthorityGateSchema = "raios.ram_only_hello_service_rollback_media_write_authority_gate.v0"
        $HelloRollbackMediaWriteAuthorityGateId = "hello_rollback_media_write_authority_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackMediaWriteAuthorityGateStatus = "denied_missing_durable_audit_policy"
        $HelloRollbackMediaWriteAuthorityGateReason = "target_region_test_media_write_verified_durable_audit_policy_missing"
        $HelloRollbackTestMediaWriteAuthorityReason = "test_infrastructure_media_write_authority_verified_current_boot"
        $HelloRollbackTargetRegionWriteReadbackDryRunSchema = "raios.ram_only_hello_service_rollback_target_region_write_readback_dry_run.v0"
        $HelloRollbackTargetRegionWriteReadbackDryRunId = "hello_rollback_target_region_write_readback_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackTargetRegionWriteReadbackDryRunStatus = "target_region_sector_image_readback_verified_not_durable_authority"
        $HelloRollbackTargetRegionWriteReadbackDryRunReason = "planned_sector_image_written_and_read_back_on_target_region_current_boot"
        $HelloRollbackScratchRegionStartLba = 1
        $HelloRollbackScratchRegionLbaCount = 1
        $HelloRollbackScratchRegionByteCount = 512
        $HelloRollbackAppendRecordDryRunSchema = "raios.ram_only_hello_service_rollback_append_record_dry_run.v0"
        $HelloRollbackAppendRecordDryRunId = "hello_rollback_append_record_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackAppendRecordDryRunStatus = "scratch_record_image_ready_not_durable_authority"
        $HelloRollbackAppendRecordDryRunReason = "scratch_writer_dry_run_verified_no_append_authority"
        $HelloRollbackAppendRecordCanonicalization = "raios.rollback_append_record_image.canonical.v0"
        $HelloRollbackAppendRecordAuditByteLength = 255
        $HelloRollbackAppendRecordRollbackByteLength = 225
        $HelloRollbackAppendRecordTotalByteLength = 480
        $HelloRollbackAppendSectorPlanDryRunSchema = "raios.ram_only_hello_service_rollback_append_sector_plan_dry_run.v0"
        $HelloRollbackAppendSectorPlanDryRunId = "hello_rollback_append_sector_plan_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackAppendSectorPlanDryRunStatus = "scratch_sector_image_ready_not_durable_authority"
        $HelloRollbackAppendSectorPlanDryRunReason = "append_record_images_fit_scratch_sector_without_write_authority"
        $HelloRollbackAppendSectorPlanCanonicalization = "raios.rollback_append_sector_plan.canonical.v0"
        $HelloRollbackAppendSectorPaddingPolicy = "zero_pad_to_logical_sector"
        $HelloRollbackAppendSectorWriteReadbackDryRunSchema = "raios.ram_only_hello_service_rollback_append_sector_write_readback_dry_run.v0"
        $HelloRollbackAppendSectorWriteReadbackDryRunId = "hello_rollback_append_sector_write_readback_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackAppendSectorWriteReadbackDryRunStatus = "scratch_sector_image_readback_verified_not_durable_authority"
        $HelloRollbackAppendSectorWriteReadbackDryRunReason = "planned_sector_image_written_and_read_back_on_scratch_only"
        $HelloRollbackDurableAppendAuthorityPreflightSchema = "raios.ram_only_hello_service_rollback_durable_append_authority_preflight.v0"
        $HelloRollbackDurableAppendAuthorityPreflightId = "hello_rollback_durable_append_authority_preflight.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendAuthorityPreflightStatus = "denied_missing_durable_append_authority"
        $HelloRollbackDurableAppendAuthorityPreflightReason = "durable_append_authority_not_granted"
        $HelloRollbackDurableAppendAuthorityRemainingReason = "durable_append_authority_missing"
        $HelloRollbackDurableWriterPolicyPreflightSchema = "raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0"
        $HelloRollbackDurableWriterPolicyPreflightId = "hello_rollback_durable_writer_policy_preflight.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableWriterPolicyPreflightStatus = "candidate_ready_not_append_authority"
        $HelloRollbackDurableWriterPolicyPreflightReason = "writer_candidates_verified_without_append_authority"
        $HelloRollbackDurableAppendTransactionAuthorizationGateSchema = "raios.ram_only_hello_service_rollback_durable_append_transaction_authorization_gate.v0"
        $HelloRollbackDurableAppendTransactionAuthorizationGateId = "hello_rollback_durable_append_transaction_authorization_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendTransactionAuthorizationGateStatus = "denied_missing_durable_append_transaction_authority"
        $HelloRollbackDurableAppendTransactionAuthorizationGateReason = "writer_policy_ready_durable_append_authority_missing"
        $HelloRollbackTargetRegionDiscoverySchema = "raios.audit_rollback_target_region_discovery.v0"
        $HelloRollbackTargetRegionDiscoveryId = "target_region.audit_rollback.current_boot"
        $HelloRollbackTargetRegionDiscoveryStatus = "available"
        $HelloRollbackTargetRegionDiscoveryReason = "dedicated_audit_rollback_region_discovered_read_only"
        $HelloRollbackTargetRegionDiscoverySource = "dedicated_audit_rollback_label_scan"
        $HelloRollbackAppendSectorAuditOffset = 0
        $HelloRollbackAppendSectorRollbackOffset = 255
        $HelloRollbackAppendSectorPaddingOffset = 480
        $HelloRollbackAppendSectorPaddingByteLength = 32
        $HelloRollbackTransactionWriterStorageLayoutStatus = "missing"
        $HelloRollbackTransactionWriterStorageLayoutReason = "persistence_device_write_path_missing"
        $HelloRollbackTransactionWriterStorageAppendEngineStatus = "missing"
        $HelloRollbackTransactionWriterStorageAppendEngineReason = "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing"
        $HelloRollbackAppendEngineReadinessDecisionStatus = "available"
        $HelloRollbackAppendEngineReadinessDecisionReason = "transaction_append_engine_ready"
        $HelloRollbackAppendEngineReadinessDecisionSchema = "raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0"
        $HelloRollbackAppendEngineReadinessDecisionId = "hello_rollback_append_engine_readiness_decision.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendAuthorityDecisionSchema = "raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0"
        $HelloRollbackDurableAppendAuthorityDecisionId = "hello_rollback_durable_append_authority_decision.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendAuthorityDecisionStatus = "denied_missing_durable_append_authority"
        $HelloRollbackDurableAppendAuthorityDecisionReason = "append_engine_ready_durable_audit_policy_missing"
        $HelloRollbackDurableAuditPolicyDecisionSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0"
        $HelloRollbackDurableAuditPolicyDecisionId = "hello_rollback_durable_audit_policy_decision.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyDecisionStatus = "denied_missing_durable_audit_policy"
        $HelloRollbackDurableAuditPolicyDecisionReason = "durable_append_authority_blocked_by_missing_durable_audit_policy"
        $HelloRollbackDurableAuditPolicyCandidateSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0"
        $HelloRollbackDurableAuditPolicyCandidateId = "hello_rollback_durable_audit_policy_candidate.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyCandidateStatus = "candidate_ready_not_durable_authority"
        $HelloRollbackDurableAuditPolicyCandidateReason = "audit_record_image_and_media_policy_verified_without_write_authority"
        $HelloRollbackDurableAuditPolicyAcceptanceGateSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0"
        $HelloRollbackDurableAuditPolicyAcceptanceGateId = "hello_rollback_durable_audit_policy_acceptance_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyAcceptanceGateStatus = "denied_missing_durable_policy_ledger_or_write_authority"
        $HelloRollbackDurableAuditPolicyAcceptanceGateReason = "candidate_verified_without_durable_policy_ledger_or_write_authority"
        $HelloRollbackDurableAuditPolicyLedgerCandidateSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0"
        $HelloRollbackDurableAuditPolicyLedgerCandidateId = "hello_rollback_durable_audit_policy_ledger_candidate.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyLedgerCandidateStatus = "candidate_ready_read_only_not_write_authority"
        $HelloRollbackDurableAuditPolicyLedgerCandidateReason = "acceptance_gate_bound_without_durable_write_authority"
        $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0"
        $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultId = "hello_rollback_durable_audit_policy_ledger_aware_acceptance_result.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultStatus = "denied_missing_write_authority"
        $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultReason = "read_only_ledger_candidate_without_durable_write_authority"
        $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilitySchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_write_authority_availability.v0"
        $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityId = "hello_rollback_durable_audit_policy_write_authority_availability.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityStatus = "denied_missing_durable_write_authority"
        $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityReason = "ledger_aware_acceptance_result_without_durable_write_authority"
        $HelloRollbackDurablePolicyLedgerAvailabilitySchema = "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0"
        $HelloRollbackDurablePolicyLedgerAvailabilityId = "hello_rollback_durable_policy_ledger_availability.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurablePolicyLedgerAvailabilityStatus = "denied_missing_durable_policy_ledger"
        $HelloRollbackDurablePolicyLedgerAvailabilityReason = "write_authority_availability_without_durable_policy_ledger"
        $HelloRollbackDurablePolicyLedgerAvailabilityDryRunSchema = "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0"
        $HelloRollbackDurablePolicyLedgerAvailabilityDryRunId = "hello_rollback_durable_policy_ledger_availability_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurablePolicyLedgerAvailabilityDryRunStatus = "test_media_policy_ledger_availability_verified_not_durable_authority"
        $HelloRollbackDurablePolicyLedgerAvailabilityDryRunReason = "policy_ledger_availability_bound_to_transaction_denial_current_boot"
        $HelloRollbackDurableAuditPolicyAvailabilitySchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0"
        $HelloRollbackDurableAuditPolicyAvailabilityId = "hello_rollback_durable_audit_policy_availability.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyAvailabilityStatus = "denied_missing_durable_audit_policy"
        $HelloRollbackDurableAuditPolicyAvailabilityReason = "durable_policy_ledger_availability_without_durable_audit_policy"
        $HelloRollbackDurableAuditPolicyAvailabilityDryRunSchema = "raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0"
        $HelloRollbackDurableAuditPolicyAvailabilityDryRunId = "hello_rollback_durable_audit_policy_availability_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAuditPolicyAvailabilityDryRunStatus = "test_media_audit_policy_availability_verified_not_durable_authority"
        $HelloRollbackDurableAuditPolicyAvailabilityDryRunReason = "audit_policy_availability_bound_to_transaction_denial_current_boot"
        $HelloRollbackDurableAppendAuthorityAvailabilitySchema = "raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0"
        $HelloRollbackDurableAppendAuthorityAvailabilityId = "hello_rollback_durable_append_authority_availability.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendAuthorityAvailabilityStatus = "denied_missing_durable_append_authority"
        $HelloRollbackDurableAppendAuthorityAvailabilityReason = "durable_audit_policy_availability_without_durable_append_authority"
        $HelloRollbackDurableAppendAuthorityAvailabilityDryRunSchema = "raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0"
        $HelloRollbackDurableAppendAuthorityAvailabilityDryRunId = "hello_rollback_durable_append_authority_availability_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurableAppendAuthorityAvailabilityDryRunStatus = "test_media_append_authority_availability_verified_not_durable_authority"
        $HelloRollbackDurableAppendAuthorityAvailabilityDryRunReason = "append_authority_availability_bound_to_transaction_denial_current_boot"
        $HelloRollbackTransactionAppendAvailabilityDecisionSchema = "raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0"
        $HelloRollbackTransactionAppendAvailabilityDecisionId = "hello_rollback_transaction_append_availability_decision.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionAppendAvailabilityDecisionStatus = "denied_missing_rollback_transaction_append_authority"
        $HelloRollbackTransactionAppendAvailabilityDecisionReason = "durable_append_authority_availability_without_transaction_append_authority"
        $HelloRollbackTransactionAppendAuthorityDenialGateSchema = "raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0"
        $HelloRollbackTransactionAppendAuthorityDenialGateId = "hello_rollback_transaction_append_authority_denial_gate.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionAppendAuthorityDenialGateStatus = "denied_missing_rollback_transaction_append_authority"
        $HelloRollbackTransactionAppendAuthorityDenialGateReason = "transaction_append_availability_decision_without_append_authority"
        $HelloRollbackTransactionAppendDryRunSchema = "raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0"
        $HelloRollbackTransactionAppendDryRunId = "hello_rollback_transaction_append_dry_run.current_boot.svc.demo.hello.v0"
        $HelloRollbackTransactionAppendDryRunStatus = "blocked_by_transaction_append_authority_denial_gate"
        $HelloRollbackTransactionAppendDryRunReason = "missing_transaction_append_authority"
        $HelloRollbackTargetRegionSectorInspectionSchema = "raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0"
        $HelloRollbackTargetRegionSectorInspectionId = "hello_rollback_target_region_sector_inspection.current_boot.svc.demo.hello.v0"
        $HelloRollbackTargetRegionSectorInspectionStatus = "target_region_sector_read_parsed_not_durable_authority"
        $HelloRollbackTargetRegionSectorInspectionReason = "target_region_sector_read_and_append_offsets_verified_current_boot"
        $HelloRecoveryRollbackInspectSourceReferenceSchema = "raios.recovery_rollback_inspect_source_reference.v0"
        $HelloRecoveryRollbackInspectSourceReferenceId = "recovery_rollback_inspect_source.current_boot.svc.demo.hello.v0"
        $HelloRecoveryRollbackInspectSourceReferenceStatus = "retained_sector_inspection_source_reference"
        $HelloRecoveryRollbackInspectSourceReferenceReason = "retained_recovery_rollback_inspect_source_matches_sector_inspection"
        $HelloRollbackDurablePolicyWriteAuthorityDecisionSchema = "raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0"
        $HelloRollbackDurablePolicyWriteAuthorityDecisionId = "hello_rollback_durable_policy_write_authority_decision.current_boot.svc.demo.hello.v0"
        $HelloRollbackDurablePolicyWriteAuthorityDecisionStatus = "denied_missing_durable_policy_write_authority"
        $HelloRollbackDurablePolicyWriteAuthorityDecisionReason = "transaction_append_dry_run_verified_without_durable_policy_write_authority"
        $HelloRollbackTransactionWriterStorageAppendContractStatus = "missing"
        $HelloRollbackTransactionWriterStorageAppendContractReason = "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"
        $HelloRollbackTransactionWriterStorageEnvelopeStatus = "missing"
        $HelloRollbackTransactionWriterStorageEnvelopeReason = "rollback_transaction_envelope_missing"
        $HelloLoadPlanPreflightSelftestSchema = "raios.current_boot_artifact_load_plan_preflight_selftest.v0"
        $HelloLoadPlanPreflightSelftestId = "artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0"
        $HelloRecoveryRollbackInspectSourceReferenceSelftestSchema = "raios.recovery_rollback_inspect_source_reference_selftest.v0"
        $HelloRecoveryRollbackInspectSourceReferenceSelftestId = "recovery_rollback_inspect_source_reference_selftest.current_boot.svc.demo.hello.v0"

        $AssertHelloRollbackTargetRegionDiscovery = {
            param(
                [string]$Name,
                [object]$TargetRegion
            )

            if (-not $TargetRegion -or $TargetRegion.schema -ne $HelloRollbackTargetRegionDiscoverySchema -or $TargetRegion.id -ne $HelloRollbackTargetRegionDiscoveryId -or $TargetRegion.status -ne $HelloRollbackTargetRegionDiscoveryStatus -or $TargetRegion.reason -ne $HelloRollbackTargetRegionDiscoveryReason -or $TargetRegion.source -ne $HelloRollbackTargetRegionDiscoverySource -or $TargetRegion.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or -not $TargetRegion.partition_inventory_available -or $TargetRegion.partition_inventory_scheme -ne $HelloRollbackPartitionInventoryScheme -or $TargetRegion.partition_inventory_source_lba -ne 0 -or $TargetRegion.partition_entry_count -ne 0 -or -not $TargetRegion.mbr_signature_valid -or $TargetRegion.boot_metadata_lba -ne 0 -or -not $TargetRegion.candidate_region_present -or $TargetRegion.candidate_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegion.candidate_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TargetRegion.candidate_region_is_scratch -or $TargetRegion.candidate_overlaps_boot_metadata -or $TargetRegion.candidate_overlaps_scratch -or $TargetRegion.scratch_region_id -ne $HelloRollbackScratchBlockRegionId -or -not $TargetRegion.scratch_region_available -or $TargetRegion.scratch_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegion.scratch_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or -not $TargetRegion.scratch_rejected_as_durable_authority -or -not $TargetRegion.durable_region_available -or $TargetRegion.authorizes_append -or $TargetRegion.writes_durable_audit_log -or $TargetRegion.writes_rollback_store -or $TargetRegion.write_attempted) {
                throw "Expected $Name to expose a read-only dedicated audit/rollback target region while rejecting scratch/write authority"
            }
        }

        $AssertHelloRollbackScratchWriterDryRun = {
            param(
                [string]$Name,
                [object]$DryRun,
                [bool]$RequireAppendRecord = $false
            )

            if (-not $DryRun -or $DryRun.schema -ne $HelloRollbackScratchWriterDryRunSchema -or $DryRun.id -ne $HelloRollbackScratchWriterDryRunId -or $DryRun.status -ne $HelloRollbackScratchWriterDryRunStatus -or $DryRun.reason -ne $HelloRollbackScratchWriterDryRunReason -or $DryRun.source_authority_id -ne $HelloRollbackScratchBlockWriteAuthorityId -or $DryRun.source_region_id -ne $HelloRollbackScratchBlockRegionId -or $DryRun.target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DryRun.target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DryRun.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DryRun.target_range_scratch_owned -or -not $DryRun.target_range_within_device_bounds -or -not $DryRun.target_range_no_boot_or_partition_metadata_overlap -or -not $DryRun.target_range_ready -or $DryRun.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $DryRun.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DryRun.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $DryRun.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or -not $DryRun.dry_run_evaluated -or $DryRun.authorizes_append -or $DryRun.writes_durable_audit_log -or $DryRun.writes_rollback_store -or $DryRun.appends_rollback_transaction -or $DryRun.write_attempted) {
                throw "Expected $Name to expose scratch-only rollback writer dry-run readiness without authorizing append"
            }
            if ($RequireAppendRecord) {
                $AppendRecord = $DryRun.append_record_dry_run
                if (-not $AppendRecord -or $AppendRecord.schema -ne $HelloRollbackAppendRecordDryRunSchema -or $AppendRecord.id -ne $HelloRollbackAppendRecordDryRunId -or $AppendRecord.canonicalization -ne $HelloRollbackAppendRecordCanonicalization -or $AppendRecord.status -ne $HelloRollbackAppendRecordDryRunStatus -or $AppendRecord.reason -ne $HelloRollbackAppendRecordDryRunReason -or -not $AppendRecord.dry_run_hash -or -not $AppendRecord.dry_run_hash.StartsWith("sha256:") -or $AppendRecord.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AppendRecord.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or -not $AppendRecord.audit_record_image_hash -or -not $AppendRecord.audit_record_image_hash.StartsWith("sha256:") -or $AppendRecord.audit_record_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or $AppendRecord.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AppendRecord.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or -not $AppendRecord.rollback_transaction_image_hash -or -not $AppendRecord.rollback_transaction_image_hash.StartsWith("sha256:") -or $AppendRecord.rollback_transaction_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $AppendRecord.total_record_byte_length -ne $HelloRollbackAppendRecordTotalByteLength -or $AppendRecord.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AppendRecord.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AppendRecord.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AppendRecord.target_range_ready -or -not $AppendRecord.source_payload_hash -or -not $AppendRecord.source_payload_hash.StartsWith("sha256:") -or -not $AppendRecord.source_provenance_hash -or -not $AppendRecord.source_provenance_hash.StartsWith("sha256:") -or $AppendRecord.authorizes_append -or $AppendRecord.writes_durable_audit_log -or $AppendRecord.writes_rollback_store -or $AppendRecord.appends_rollback_transaction -or $AppendRecord.write_attempted) {
                    throw "Expected $Name to expose canonical append-record dry-run image/hash evidence without authorizing append"
                }
                $SectorPlan = $AppendRecord.sector_plan_dry_run
                if (-not $SectorPlan -or $SectorPlan.schema -ne $HelloRollbackAppendSectorPlanDryRunSchema -or $SectorPlan.id -ne $HelloRollbackAppendSectorPlanDryRunId -or $SectorPlan.canonicalization -ne $HelloRollbackAppendSectorPlanCanonicalization -or $SectorPlan.status -ne $HelloRollbackAppendSectorPlanDryRunStatus -or $SectorPlan.reason -ne $HelloRollbackAppendSectorPlanDryRunReason -or -not $SectorPlan.plan_hash -or -not $SectorPlan.plan_hash.StartsWith("sha256:") -or -not $SectorPlan.sector_image_hash -or -not $SectorPlan.sector_image_hash.StartsWith("sha256:") -or $SectorPlan.sector_size_bytes -ne $HelloRollbackScratchRegionByteCount -or $SectorPlan.audit_record_offset -ne $HelloRollbackAppendSectorAuditOffset -or $SectorPlan.audit_record_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or $SectorPlan.rollback_transaction_offset -ne $HelloRollbackAppendSectorRollbackOffset -or $SectorPlan.rollback_transaction_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $SectorPlan.padding_policy -ne $HelloRollbackAppendSectorPaddingPolicy -or $SectorPlan.padding_offset -ne $HelloRollbackAppendSectorPaddingOffset -or $SectorPlan.padding_byte_length -ne $HelloRollbackAppendSectorPaddingByteLength -or $SectorPlan.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $SectorPlan.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $SectorPlan.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $SectorPlan.target_range_ready -or $SectorPlan.source_append_record_hash -ne $AppendRecord.dry_run_hash -or $SectorPlan.authorizes_append -or $SectorPlan.writes_durable_audit_log -or $SectorPlan.writes_rollback_store -or $SectorPlan.appends_rollback_transaction -or $SectorPlan.write_attempted) {
                    throw "Expected $Name to expose scratch append-sector plan/hash evidence without authorizing append or media writes"
                }
                $SectorWriteReadback = $SectorPlan.scratch_sector_write_readback_dry_run
                if (-not $SectorWriteReadback -or $SectorWriteReadback.schema -ne $HelloRollbackAppendSectorWriteReadbackDryRunSchema -or $SectorWriteReadback.id -ne $HelloRollbackAppendSectorWriteReadbackDryRunId -or $SectorWriteReadback.status -ne $HelloRollbackAppendSectorWriteReadbackDryRunStatus -or $SectorWriteReadback.reason -ne $HelloRollbackAppendSectorWriteReadbackDryRunReason -or -not $SectorWriteReadback.dry_run_hash -or -not $SectorWriteReadback.dry_run_hash.StartsWith("sha256:") -or $SectorWriteReadback.source_plan_hash -ne $SectorPlan.plan_hash -or $SectorWriteReadback.planned_sector_image_hash -ne $SectorPlan.sector_image_hash -or $SectorWriteReadback.readback_sector_image_hash -ne $SectorPlan.sector_image_hash -or $SectorWriteReadback.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $SectorWriteReadback.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $SectorWriteReadback.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $SectorWriteReadback.label_found -or -not $SectorWriteReadback.target_range_ready -or -not $SectorWriteReadback.write_attempted -or -not $SectorWriteReadback.write_completed -or -not $SectorWriteReadback.readback_completed -or -not $SectorWriteReadback.readback_matches_planned_image -or $SectorWriteReadback.authorizes_append -or $SectorWriteReadback.writes_durable_audit_log -or $SectorWriteReadback.writes_rollback_store -or $SectorWriteReadback.appends_rollback_transaction) {
                    throw "Expected $Name to write/read back the planned sector image on scratch only"
                }
                $DurablePreflight = $SectorWriteReadback.durable_append_authority_preflight
                $TargetRegionWriteReadback = $DurablePreflight.target_region_write_readback_dry_run
                if (-not $DurablePreflight -or $DurablePreflight.schema -ne $HelloRollbackDurableAppendAuthorityPreflightSchema -or $DurablePreflight.id -ne $HelloRollbackDurableAppendAuthorityPreflightId -or $DurablePreflight.status -ne $HelloRollbackDurableAppendAuthorityPreflightStatus -or $DurablePreflight.reason -ne $HelloRollbackDurableAppendAuthorityPreflightReason -or -not $DurablePreflight.preflight_hash -or -not $DurablePreflight.preflight_hash.StartsWith("sha256:") -or $DurablePreflight.source_write_readback_hash -ne $SectorWriteReadback.dry_run_hash -or $DurablePreflight.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or -not $DurablePreflight.test_infrastructure_media_write_authority_available -or $DurablePreflight.remaining_denial_reason -ne $HelloRollbackDurableAppendAuthorityRemainingReason -or $DurablePreflight.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $DurablePreflight.append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $DurablePreflight.transaction_writer_readiness_id -ne $HelloRollbackTransactionWriterReadinessId -or $DurablePreflight.audit_ledger_writer_fact_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $DurablePreflight.rollback_store_writer_fact_id -ne $HelloRollbackTransactionWriterStorageTargetId -or -not $DurablePreflight.scratch_write_readback_verified -or $DurablePreflight.scratch_used_as_durable_authority -or -not $DurablePreflight.durable_audit_writer_available -or -not $DurablePreflight.rollback_store_writer_available -or -not $DurablePreflight.transaction_append_writer_available -or $DurablePreflight.authorizes_append -or $DurablePreflight.writes_durable_audit_log -or $DurablePreflight.writes_rollback_store -or $DurablePreflight.appends_rollback_transaction) {
                    throw "Expected $Name to expose durable append-authority preflight as denied over missing durable append authority"
                }
                $DurableWriterPolicyPreflight = $DurablePreflight.durable_writer_policy_preflight
                if (-not $DurableWriterPolicyPreflight -or $DurableWriterPolicyPreflight.schema -ne $HelloRollbackDurableWriterPolicyPreflightSchema -or $DurableWriterPolicyPreflight.id -ne $HelloRollbackDurableWriterPolicyPreflightId -or $DurableWriterPolicyPreflight.status -ne $HelloRollbackDurableWriterPolicyPreflightStatus -or $DurableWriterPolicyPreflight.reason -ne $HelloRollbackDurableWriterPolicyPreflightReason -or -not $DurableWriterPolicyPreflight.preflight_hash -or -not $DurableWriterPolicyPreflight.preflight_hash.StartsWith("sha256:") -or $DurableWriterPolicyPreflight.source_append_record_hash -ne $AppendRecord.dry_run_hash -or $DurableWriterPolicyPreflight.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $DurableWriterPolicyPreflight.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableWriterPolicyPreflight.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $DurableWriterPolicyPreflight.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurableWriterPolicyPreflight.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $DurableWriterPolicyPreflight.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $DurableWriterPolicyPreflight.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableWriterPolicyPreflight.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableWriterPolicyPreflight.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableWriterPolicyPreflight.target_range_ready -or -not $DurableWriterPolicyPreflight.test_infrastructure_media_write_authority_available -or -not $DurableWriterPolicyPreflight.durable_audit_writer_available -or -not $DurableWriterPolicyPreflight.rollback_store_writer_available -or -not $DurableWriterPolicyPreflight.transaction_append_writer_available -or $DurableWriterPolicyPreflight.authorizes_append -or $DurableWriterPolicyPreflight.writes_durable_audit_log -or $DurableWriterPolicyPreflight.writes_rollback_store -or $DurableWriterPolicyPreflight.appends_rollback_transaction -or $DurableWriterPolicyPreflight.write_attempted) {
                    throw "Expected $Name to expose denied durable writer-policy preflight over verified target-region test media"
                }
                $DurableAppendTransactionAuthorizationGate = $DurablePreflight.durable_append_transaction_authorization_gate
                if (-not $DurableAppendTransactionAuthorizationGate -or $DurableAppendTransactionAuthorizationGate.schema -ne $HelloRollbackDurableAppendTransactionAuthorizationGateSchema -or $DurableAppendTransactionAuthorizationGate.id -ne $HelloRollbackDurableAppendTransactionAuthorizationGateId -or $DurableAppendTransactionAuthorizationGate.status -ne $HelloRollbackDurableAppendTransactionAuthorizationGateStatus -or $DurableAppendTransactionAuthorizationGate.reason -ne $HelloRollbackDurableAppendTransactionAuthorizationGateReason -or -not $DurableAppendTransactionAuthorizationGate.gate_hash -or -not $DurableAppendTransactionAuthorizationGate.gate_hash.StartsWith("sha256:") -or $DurableAppendTransactionAuthorizationGate.source_writer_policy_preflight_hash -ne $DurableWriterPolicyPreflight.preflight_hash -or $DurableAppendTransactionAuthorizationGate.source_append_record_hash -ne $AppendRecord.dry_run_hash -or $DurableAppendTransactionAuthorizationGate.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $DurableAppendTransactionAuthorizationGate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAppendTransactionAuthorizationGate.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $DurableAppendTransactionAuthorizationGate.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurableAppendTransactionAuthorizationGate.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $DurableAppendTransactionAuthorizationGate.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $DurableAppendTransactionAuthorizationGate.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAppendTransactionAuthorizationGate.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAppendTransactionAuthorizationGate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAppendTransactionAuthorizationGate.target_range_ready -or -not $DurableAppendTransactionAuthorizationGate.test_infrastructure_media_write_authority_available -or -not $DurableAppendTransactionAuthorizationGate.append_engine_available -or -not $DurableAppendTransactionAuthorizationGate.durable_audit_writer_available -or -not $DurableAppendTransactionAuthorizationGate.rollback_store_writer_available -or -not $DurableAppendTransactionAuthorizationGate.transaction_append_writer_available -or $DurableAppendTransactionAuthorizationGate.authorizes_append -or $DurableAppendTransactionAuthorizationGate.authorizes_transaction_append -or $DurableAppendTransactionAuthorizationGate.writes_durable_audit_log -or $DurableAppendTransactionAuthorizationGate.writes_rollback_store -or $DurableAppendTransactionAuthorizationGate.appends_rollback_transaction -or $DurableAppendTransactionAuthorizationGate.write_attempted) {
                    throw "Expected $Name to expose denied durable append/transaction authorization gate over writer-policy evidence"
                }
                $AppendEngineReadinessDecision = $DurablePreflight.append_engine_readiness_decision
                if (-not $AppendEngineReadinessDecision -or $AppendEngineReadinessDecision.schema -ne $HelloRollbackAppendEngineReadinessDecisionSchema -or $AppendEngineReadinessDecision.id -ne $HelloRollbackAppendEngineReadinessDecisionId -or $AppendEngineReadinessDecision.status -ne $HelloRollbackAppendEngineReadinessDecisionStatus -or $AppendEngineReadinessDecision.reason -ne $HelloRollbackAppendEngineReadinessDecisionReason -or -not $AppendEngineReadinessDecision.decision_hash -or -not $AppendEngineReadinessDecision.decision_hash.StartsWith("sha256:") -or $AppendEngineReadinessDecision.source_authorization_gate_hash -ne $DurableAppendTransactionAuthorizationGate.gate_hash -or $AppendEngineReadinessDecision.source_writer_policy_preflight_hash -ne $DurableWriterPolicyPreflight.preflight_hash -or $AppendEngineReadinessDecision.source_append_record_hash -ne $AppendRecord.dry_run_hash -or $AppendEngineReadinessDecision.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $AppendEngineReadinessDecision.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $AppendEngineReadinessDecision.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AppendEngineReadinessDecision.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AppendEngineReadinessDecision.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AppendEngineReadinessDecision.target_range_ready -or -not $AppendEngineReadinessDecision.test_infrastructure_media_write_authority_available -or -not $AppendEngineReadinessDecision.append_engine_available -or -not $AppendEngineReadinessDecision.durable_audit_writer_available -or -not $AppendEngineReadinessDecision.rollback_store_writer_available -or -not $AppendEngineReadinessDecision.transaction_append_writer_available -or -not $AppendEngineReadinessDecision.ready -or $AppendEngineReadinessDecision.authorizes_append -or $AppendEngineReadinessDecision.authorizes_transaction_append -or $AppendEngineReadinessDecision.writes_durable_audit_log -or $AppendEngineReadinessDecision.writes_rollback_store -or $AppendEngineReadinessDecision.appends_rollback_transaction -or $AppendEngineReadinessDecision.write_attempted) {
                    throw "Expected $Name to consume authorization gate into ready, non-authorizing append-engine evidence"
                }
                & $AssertHelloRollbackTargetRegionDiscovery -Name "$Name durable append preflight" -TargetRegion $DurablePreflight.target_region_discovery
                $MediaWritePolicyPreflight = $DurablePreflight.target_region_media_write_policy_preflight
                if (-not $MediaWritePolicyPreflight -or $MediaWritePolicyPreflight.schema -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightSchema -or $MediaWritePolicyPreflight.id -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightId -or $MediaWritePolicyPreflight.status -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightStatus -or $MediaWritePolicyPreflight.reason -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightReason -or -not $MediaWritePolicyPreflight.preflight_hash -or -not $MediaWritePolicyPreflight.preflight_hash.StartsWith("sha256:") -or $MediaWritePolicyPreflight.source_contract_schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $MediaWritePolicyPreflight.source_contract_id -ne $HelloRollbackTargetRegionWriterContractId -or $MediaWritePolicyPreflight.source_contract_status -ne $HelloRollbackTargetRegionWriterContractStatus -or $MediaWritePolicyPreflight.source_contract_reason -ne $HelloRollbackTargetRegionWriterContractReason -or $MediaWritePolicyPreflight.owner_method -ne $HelloRollbackTransactionWriterStorageFoundationOwner -or $MediaWritePolicyPreflight.append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $MediaWritePolicyPreflight.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $MediaWritePolicyPreflight.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $MediaWritePolicyPreflight.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $MediaWritePolicyPreflight.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $MediaWritePolicyPreflight.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $MediaWritePolicyPreflight.target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $MediaWritePolicyPreflight.target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $MediaWritePolicyPreflight.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $MediaWritePolicyPreflight.source_contract_target_range_ready -or -not $MediaWritePolicyPreflight.owner_ids_verified -or -not $MediaWritePolicyPreflight.target_ids_verified -or -not $MediaWritePolicyPreflight.target_span_verified -or -not $MediaWritePolicyPreflight.schema_ids_verified -or -not $MediaWritePolicyPreflight.media_write_authority_required -or $MediaWritePolicyPreflight.media_write_authority_available -or $MediaWritePolicyPreflight.media_write_authority_reason -ne "media_write_authority_missing" -or -not $MediaWritePolicyPreflight.durable_audit_policy_required -or $MediaWritePolicyPreflight.durable_audit_policy_available -or $MediaWritePolicyPreflight.durable_audit_policy_reason -ne "durable_audit_policy_missing" -or $MediaWritePolicyPreflight.authorizes_media_write -or $MediaWritePolicyPreflight.authorizes_append -or $MediaWritePolicyPreflight.writes_durable_audit_log -or $MediaWritePolicyPreflight.writes_rollback_store -or $MediaWritePolicyPreflight.appends_rollback_transaction -or $MediaWritePolicyPreflight.write_attempted) {
                    throw "Expected $Name to bind target-region media-write policy preflight into denied durable append authority"
                }
                $TargetRegionWriteReadback = $DurablePreflight.target_region_write_readback_dry_run
                $MediaWriteAuthorityGate = $DurablePreflight.media_write_authority_gate
                if (-not $MediaWriteAuthorityGate -or $MediaWriteAuthorityGate.schema -ne $HelloRollbackMediaWriteAuthorityGateSchema -or $MediaWriteAuthorityGate.id -ne $HelloRollbackMediaWriteAuthorityGateId -or $MediaWriteAuthorityGate.status -ne $HelloRollbackMediaWriteAuthorityGateStatus -or $MediaWriteAuthorityGate.reason -ne $HelloRollbackMediaWriteAuthorityGateReason -or -not $MediaWriteAuthorityGate.gate_hash -or -not $MediaWriteAuthorityGate.gate_hash.StartsWith("sha256:") -or $MediaWriteAuthorityGate.source_durable_append_authority_preflight_hash -ne $DurablePreflight.preflight_hash -or $MediaWriteAuthorityGate.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $MediaWriteAuthorityGate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $MediaWriteAuthorityGate.source_contract_schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $MediaWriteAuthorityGate.source_contract_id -ne $HelloRollbackTargetRegionWriterContractId -or $MediaWriteAuthorityGate.source_contract_status -ne $HelloRollbackTargetRegionWriterContractStatus -or $MediaWriteAuthorityGate.source_contract_reason -ne $HelloRollbackTargetRegionWriterContractReason -or $MediaWriteAuthorityGate.target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $MediaWriteAuthorityGate.target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $MediaWriteAuthorityGate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $MediaWriteAuthorityGate.source_contract_target_range_ready -or -not $MediaWriteAuthorityGate.owner_ids_verified -or -not $MediaWriteAuthorityGate.target_ids_verified -or -not $MediaWriteAuthorityGate.target_span_verified -or -not $MediaWriteAuthorityGate.schema_ids_verified -or -not $MediaWriteAuthorityGate.media_write_authority_required -or -not $MediaWriteAuthorityGate.media_write_authority_available -or $MediaWriteAuthorityGate.media_write_authority_reason -ne $HelloRollbackTestMediaWriteAuthorityReason -or -not $MediaWriteAuthorityGate.test_infrastructure_media_write_authority_available -or -not $MediaWriteAuthorityGate.durable_audit_policy_required -or $MediaWriteAuthorityGate.durable_audit_policy_available -or $MediaWriteAuthorityGate.durable_audit_policy_reason -ne "durable_audit_policy_missing" -or $MediaWriteAuthorityGate.authorizes_media_write -or $MediaWriteAuthorityGate.authorizes_append -or $MediaWriteAuthorityGate.writes_durable_audit_log -or $MediaWriteAuthorityGate.writes_rollback_store -or $MediaWriteAuthorityGate.appends_rollback_transaction -or -not $MediaWriteAuthorityGate.target_region_write_attempted -or $MediaWriteAuthorityGate.write_attempted) {
                    throw "Expected $Name to expose a fail-closed media-write-authority gate over the target-region policy preflight"
                }
                $DurableAppendAuthorityDecision = $DurablePreflight.durable_append_authority_decision
                if (-not $DurableAppendAuthorityDecision -or $DurableAppendAuthorityDecision.schema -ne $HelloRollbackDurableAppendAuthorityDecisionSchema -or $DurableAppendAuthorityDecision.id -ne $HelloRollbackDurableAppendAuthorityDecisionId -or $DurableAppendAuthorityDecision.status -ne $HelloRollbackDurableAppendAuthorityDecisionStatus -or $DurableAppendAuthorityDecision.reason -ne $HelloRollbackDurableAppendAuthorityDecisionReason -or -not $DurableAppendAuthorityDecision.decision_hash -or -not $DurableAppendAuthorityDecision.decision_hash.StartsWith("sha256:") -or $DurableAppendAuthorityDecision.source_durable_append_authority_preflight_hash -ne $DurablePreflight.preflight_hash -or $DurableAppendAuthorityDecision.source_writer_policy_preflight_hash -ne $DurableWriterPolicyPreflight.preflight_hash -or $DurableAppendAuthorityDecision.source_append_engine_readiness_decision_hash -ne $AppendEngineReadinessDecision.decision_hash -or $DurableAppendAuthorityDecision.source_media_write_authority_gate_hash -ne $MediaWriteAuthorityGate.gate_hash -or $DurableAppendAuthorityDecision.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $DurableAppendAuthorityDecision.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAppendAuthorityDecision.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAppendAuthorityDecision.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAppendAuthorityDecision.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAppendAuthorityDecision.writer_policy_ready -or -not $DurableAppendAuthorityDecision.append_engine_ready -or -not $DurableAppendAuthorityDecision.media_write_gate_ready -or -not $DurableAppendAuthorityDecision.test_infrastructure_media_write_authority_available -or $DurableAppendAuthorityDecision.durable_audit_policy_available -or $DurableAppendAuthorityDecision.durable_append_authority_available -or $DurableAppendAuthorityDecision.authorizes_append -or $DurableAppendAuthorityDecision.authorizes_transaction_append -or $DurableAppendAuthorityDecision.writes_durable_audit_log -or $DurableAppendAuthorityDecision.writes_rollback_store -or $DurableAppendAuthorityDecision.appends_rollback_transaction -or $DurableAppendAuthorityDecision.write_attempted) {
                    throw "Expected $Name to expose a denied durable append-authority decision over ready writer/append evidence"
                }
                $DurableAuditPolicyDecision = $DurablePreflight.durable_audit_policy_decision
                if (-not $DurableAuditPolicyDecision -or $DurableAuditPolicyDecision.schema -ne $HelloRollbackDurableAuditPolicyDecisionSchema -or $DurableAuditPolicyDecision.id -ne $HelloRollbackDurableAuditPolicyDecisionId -or $DurableAuditPolicyDecision.status -ne $HelloRollbackDurableAuditPolicyDecisionStatus -or $DurableAuditPolicyDecision.reason -ne $HelloRollbackDurableAuditPolicyDecisionReason -or -not $DurableAuditPolicyDecision.decision_hash -or -not $DurableAuditPolicyDecision.decision_hash.StartsWith("sha256:") -or $DurableAuditPolicyDecision.source_durable_append_authority_decision_hash -ne $DurableAppendAuthorityDecision.decision_hash -or $DurableAuditPolicyDecision.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $DurableAuditPolicyDecision.source_media_write_authority_gate_hash -ne $MediaWriteAuthorityGate.gate_hash -or $DurableAuditPolicyDecision.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAuditPolicyDecision.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAuditPolicyDecision.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAuditPolicyDecision.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAuditPolicyDecision.append_engine_ready -or -not $DurableAuditPolicyDecision.media_write_policy_verified -or -not $DurableAuditPolicyDecision.test_infrastructure_media_write_authority_available -or $DurableAuditPolicyDecision.durable_append_authority_available -or $DurableAuditPolicyDecision.durable_audit_policy_available -or $DurableAuditPolicyDecision.authorizes_append -or $DurableAuditPolicyDecision.writes_durable_audit_log -or $DurableAuditPolicyDecision.writes_rollback_store -or $DurableAuditPolicyDecision.appends_rollback_transaction -or $DurableAuditPolicyDecision.write_attempted) {
                    throw "Expected $Name to expose a denied durable audit-policy decision over durable append/media-policy evidence"
                }
                $DurableAuditPolicyCandidate = $DurablePreflight.durable_audit_policy_candidate
                if (-not $DurableAuditPolicyCandidate -or $DurableAuditPolicyCandidate.schema -ne $HelloRollbackDurableAuditPolicyCandidateSchema -or $DurableAuditPolicyCandidate.id -ne $HelloRollbackDurableAuditPolicyCandidateId -or $DurableAuditPolicyCandidate.status -ne $HelloRollbackDurableAuditPolicyCandidateStatus -or $DurableAuditPolicyCandidate.reason -ne $HelloRollbackDurableAuditPolicyCandidateReason -or -not $DurableAuditPolicyCandidate.candidate_hash -or -not $DurableAuditPolicyCandidate.candidate_hash.StartsWith("sha256:") -or $DurableAuditPolicyCandidate.source_durable_audit_policy_decision_hash -ne $DurableAuditPolicyDecision.decision_hash -or $DurableAuditPolicyCandidate.source_audit_record_image_hash -ne $AppendRecord.audit_record_image_hash -or $DurableAuditPolicyCandidate.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $DurableAuditPolicyCandidate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAuditPolicyCandidate.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurableAuditPolicyCandidate.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAuditPolicyCandidate.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAuditPolicyCandidate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAuditPolicyCandidate.media_write_policy_verified -or -not $DurableAuditPolicyCandidate.durable_audit_policy_candidate_available -or $DurableAuditPolicyCandidate.durable_audit_policy_available -or $DurableAuditPolicyCandidate.durable_append_authority_available -or $DurableAuditPolicyCandidate.authorizes_append -or $DurableAuditPolicyCandidate.writes_durable_audit_log -or $DurableAuditPolicyCandidate.writes_rollback_store -or $DurableAuditPolicyCandidate.appends_rollback_transaction -or $DurableAuditPolicyCandidate.write_attempted) {
                    throw "Expected $Name to expose a no-write durable audit-policy candidate over audit-record/media-policy evidence"
                }
                $DurableAuditPolicyAcceptanceGate = $DurablePreflight.durable_audit_policy_acceptance_gate
                if (-not $DurableAuditPolicyAcceptanceGate -or $DurableAuditPolicyAcceptanceGate.schema -ne $HelloRollbackDurableAuditPolicyAcceptanceGateSchema -or $DurableAuditPolicyAcceptanceGate.id -ne $HelloRollbackDurableAuditPolicyAcceptanceGateId -or $DurableAuditPolicyAcceptanceGate.status -ne $HelloRollbackDurableAuditPolicyAcceptanceGateStatus -or $DurableAuditPolicyAcceptanceGate.reason -ne $HelloRollbackDurableAuditPolicyAcceptanceGateReason -or -not $DurableAuditPolicyAcceptanceGate.gate_hash -or -not $DurableAuditPolicyAcceptanceGate.gate_hash.StartsWith("sha256:") -or $DurableAuditPolicyAcceptanceGate.source_durable_audit_policy_candidate_hash -ne $DurableAuditPolicyCandidate.candidate_hash -or $DurableAuditPolicyAcceptanceGate.source_durable_audit_policy_decision_hash -ne $DurableAuditPolicyDecision.decision_hash -or $DurableAuditPolicyAcceptanceGate.source_audit_record_image_hash -ne $AppendRecord.audit_record_image_hash -or $DurableAuditPolicyAcceptanceGate.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $DurableAuditPolicyAcceptanceGate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAuditPolicyAcceptanceGate.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurableAuditPolicyAcceptanceGate.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAuditPolicyAcceptanceGate.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAuditPolicyAcceptanceGate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAuditPolicyAcceptanceGate.candidate_available -or -not $DurableAuditPolicyAcceptanceGate.media_write_policy_verified -or $DurableAuditPolicyAcceptanceGate.durable_policy_ledger_available -or $DurableAuditPolicyAcceptanceGate.write_authority_available -or $DurableAuditPolicyAcceptanceGate.durable_audit_policy_available -or $DurableAuditPolicyAcceptanceGate.durable_append_authority_available -or $DurableAuditPolicyAcceptanceGate.authorizes_append -or $DurableAuditPolicyAcceptanceGate.writes_durable_audit_log -or $DurableAuditPolicyAcceptanceGate.writes_rollback_store -or $DurableAuditPolicyAcceptanceGate.appends_rollback_transaction -or $DurableAuditPolicyAcceptanceGate.write_attempted) {
                    throw "Expected $Name to expose a fail-closed durable audit-policy acceptance gate over the candidate"
                }
                $DurableAuditPolicyLedgerCandidate = $DurablePreflight.durable_audit_policy_ledger_candidate
                if (-not $DurableAuditPolicyLedgerCandidate -or $DurableAuditPolicyLedgerCandidate.schema -ne $HelloRollbackDurableAuditPolicyLedgerCandidateSchema -or $DurableAuditPolicyLedgerCandidate.id -ne $HelloRollbackDurableAuditPolicyLedgerCandidateId -or $DurableAuditPolicyLedgerCandidate.status -ne $HelloRollbackDurableAuditPolicyLedgerCandidateStatus -or $DurableAuditPolicyLedgerCandidate.reason -ne $HelloRollbackDurableAuditPolicyLedgerCandidateReason -or -not $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or -not $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash.StartsWith("sha256:") -or $DurableAuditPolicyLedgerCandidate.source_acceptance_gate_hash -ne $DurableAuditPolicyAcceptanceGate.gate_hash -or $DurableAuditPolicyLedgerCandidate.source_durable_audit_policy_candidate_hash -ne $DurableAuditPolicyCandidate.candidate_hash -or $DurableAuditPolicyLedgerCandidate.source_durable_audit_policy_decision_hash -ne $DurableAuditPolicyDecision.decision_hash -or $DurableAuditPolicyLedgerCandidate.source_audit_record_image_hash -ne $AppendRecord.audit_record_image_hash -or $DurableAuditPolicyLedgerCandidate.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $DurableAuditPolicyLedgerCandidate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $DurableAuditPolicyLedgerCandidate.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurableAuditPolicyLedgerCandidate.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurableAuditPolicyLedgerCandidate.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurableAuditPolicyLedgerCandidate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurableAuditPolicyLedgerCandidate.read_only_ledger_candidate_available -or -not $DurableAuditPolicyLedgerCandidate.candidate_available -or -not $DurableAuditPolicyLedgerCandidate.media_write_policy_verified -or $DurableAuditPolicyLedgerCandidate.durable_policy_ledger_available -or $DurableAuditPolicyLedgerCandidate.write_authority_available -or $DurableAuditPolicyLedgerCandidate.durable_audit_policy_available -or $DurableAuditPolicyLedgerCandidate.durable_append_authority_available -or $DurableAuditPolicyLedgerCandidate.authorizes_append -or $DurableAuditPolicyLedgerCandidate.writes_durable_audit_log -or $DurableAuditPolicyLedgerCandidate.writes_rollback_store -or $DurableAuditPolicyLedgerCandidate.appends_rollback_transaction -or $DurableAuditPolicyLedgerCandidate.write_attempted) {
                    throw "Expected $Name to expose a read-only durable audit-policy ledger candidate without write authority"
                }
                $LedgerAwareAcceptanceResult = $DurablePreflight.durable_audit_policy_ledger_aware_acceptance_result
                if (-not $LedgerAwareAcceptanceResult -or $LedgerAwareAcceptanceResult.schema -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultSchema -or $LedgerAwareAcceptanceResult.id -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultId -or $LedgerAwareAcceptanceResult.status -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultStatus -or $LedgerAwareAcceptanceResult.reason -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultReason -or -not $LedgerAwareAcceptanceResult.result_hash -or -not $LedgerAwareAcceptanceResult.result_hash.StartsWith("sha256:") -or $LedgerAwareAcceptanceResult.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $LedgerAwareAcceptanceResult.source_acceptance_gate_hash -ne $DurableAuditPolicyAcceptanceGate.gate_hash -or $LedgerAwareAcceptanceResult.source_durable_audit_policy_candidate_hash -ne $DurableAuditPolicyCandidate.candidate_hash -or $LedgerAwareAcceptanceResult.source_durable_audit_policy_decision_hash -ne $DurableAuditPolicyDecision.decision_hash -or $LedgerAwareAcceptanceResult.source_audit_record_image_hash -ne $AppendRecord.audit_record_image_hash -or $LedgerAwareAcceptanceResult.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $LedgerAwareAcceptanceResult.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $LedgerAwareAcceptanceResult.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $LedgerAwareAcceptanceResult.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $LedgerAwareAcceptanceResult.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $LedgerAwareAcceptanceResult.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $LedgerAwareAcceptanceResult.read_only_ledger_candidate_available -or -not $LedgerAwareAcceptanceResult.ledger_evidence_verified -or $LedgerAwareAcceptanceResult.write_authority_available -or $LedgerAwareAcceptanceResult.durable_policy_ledger_available -or $LedgerAwareAcceptanceResult.durable_audit_policy_available -or $LedgerAwareAcceptanceResult.durable_append_authority_available -or $LedgerAwareAcceptanceResult.authorizes_append -or $LedgerAwareAcceptanceResult.writes_durable_audit_log -or $LedgerAwareAcceptanceResult.writes_rollback_store -or $LedgerAwareAcceptanceResult.appends_rollback_transaction -or $LedgerAwareAcceptanceResult.write_attempted) {
                    throw "Expected $Name to expose ledger-aware acceptance result without write authority"
                }
                $WriteAuthorityAvailability = $DurablePreflight.durable_audit_policy_write_authority_availability
                if (-not $WriteAuthorityAvailability -or $WriteAuthorityAvailability.schema -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilitySchema -or $WriteAuthorityAvailability.id -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityId -or $WriteAuthorityAvailability.status -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityStatus -or $WriteAuthorityAvailability.reason -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityReason -or -not $WriteAuthorityAvailability.availability_hash -or -not $WriteAuthorityAvailability.availability_hash.StartsWith("sha256:") -or $WriteAuthorityAvailability.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $WriteAuthorityAvailability.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $WriteAuthorityAvailability.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $WriteAuthorityAvailability.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $WriteAuthorityAvailability.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $WriteAuthorityAvailability.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $WriteAuthorityAvailability.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $WriteAuthorityAvailability.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $WriteAuthorityAvailability.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $WriteAuthorityAvailability.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $WriteAuthorityAvailability.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $WriteAuthorityAvailability.ledger_evidence_verified -or -not $WriteAuthorityAvailability.media_write_policy_verified -or -not $WriteAuthorityAvailability.target_region_write_readback_verified -or -not $WriteAuthorityAvailability.target_span_verified -or -not $WriteAuthorityAvailability.audit_rollback_target_ids_verified -or -not $WriteAuthorityAvailability.test_infrastructure_media_write_authority_available -or $WriteAuthorityAvailability.write_authority_available -or $WriteAuthorityAvailability.durable_policy_ledger_available -or $WriteAuthorityAvailability.durable_audit_policy_available -or $WriteAuthorityAvailability.durable_append_authority_available -or $WriteAuthorityAvailability.authorizes_media_write -or $WriteAuthorityAvailability.authorizes_append -or $WriteAuthorityAvailability.writes_durable_audit_log -or $WriteAuthorityAvailability.writes_rollback_store -or $WriteAuthorityAvailability.appends_rollback_transaction -or $WriteAuthorityAvailability.write_attempted) {
                    throw "Expected $Name to expose write-authority availability as verified-but-denied evidence"
                }
                $PolicyLedgerAvailability = $DurablePreflight.durable_policy_ledger_availability
                if (-not $PolicyLedgerAvailability -or $PolicyLedgerAvailability.schema -ne $HelloRollbackDurablePolicyLedgerAvailabilitySchema -or $PolicyLedgerAvailability.id -ne $HelloRollbackDurablePolicyLedgerAvailabilityId -or $PolicyLedgerAvailability.status -ne $HelloRollbackDurablePolicyLedgerAvailabilityStatus -or $PolicyLedgerAvailability.reason -ne $HelloRollbackDurablePolicyLedgerAvailabilityReason -or -not $PolicyLedgerAvailability.availability_hash -or -not $PolicyLedgerAvailability.availability_hash.StartsWith("sha256:") -or $PolicyLedgerAvailability.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $PolicyLedgerAvailability.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $PolicyLedgerAvailability.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $PolicyLedgerAvailability.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $PolicyLedgerAvailability.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $PolicyLedgerAvailability.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $PolicyLedgerAvailability.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $PolicyLedgerAvailability.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $PolicyLedgerAvailability.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $PolicyLedgerAvailability.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $PolicyLedgerAvailability.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $PolicyLedgerAvailability.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $PolicyLedgerAvailability.write_authority_evidence_verified -or -not $PolicyLedgerAvailability.ledger_evidence_verified -or -not $PolicyLedgerAvailability.media_write_policy_verified -or -not $PolicyLedgerAvailability.target_region_write_readback_verified -or -not $PolicyLedgerAvailability.target_span_verified -or -not $PolicyLedgerAvailability.audit_rollback_target_ids_verified -or -not $PolicyLedgerAvailability.test_infrastructure_media_write_authority_available -or $PolicyLedgerAvailability.write_authority_available -or $PolicyLedgerAvailability.durable_policy_ledger_available -or $PolicyLedgerAvailability.durable_audit_policy_available -or $PolicyLedgerAvailability.durable_append_authority_available -or $PolicyLedgerAvailability.authorizes_media_write -or $PolicyLedgerAvailability.authorizes_append -or $PolicyLedgerAvailability.writes_durable_audit_log -or $PolicyLedgerAvailability.writes_rollback_store -or $PolicyLedgerAvailability.appends_rollback_transaction -or $PolicyLedgerAvailability.write_attempted) {
                    throw "Expected $Name to expose durable policy-ledger availability as verified-but-denied evidence"
                }
                $AuditPolicyAvailability = $DurablePreflight.durable_audit_policy_availability
                if (-not $AuditPolicyAvailability -or $AuditPolicyAvailability.schema -ne $HelloRollbackDurableAuditPolicyAvailabilitySchema -or $AuditPolicyAvailability.id -ne $HelloRollbackDurableAuditPolicyAvailabilityId -or $AuditPolicyAvailability.status -ne $HelloRollbackDurableAuditPolicyAvailabilityStatus -or $AuditPolicyAvailability.reason -ne $HelloRollbackDurableAuditPolicyAvailabilityReason -or -not $AuditPolicyAvailability.availability_hash -or -not $AuditPolicyAvailability.availability_hash.StartsWith("sha256:") -or $AuditPolicyAvailability.source_policy_ledger_availability_hash -ne $PolicyLedgerAvailability.availability_hash -or $AuditPolicyAvailability.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $AuditPolicyAvailability.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $AuditPolicyAvailability.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $AuditPolicyAvailability.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $AuditPolicyAvailability.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $AuditPolicyAvailability.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AuditPolicyAvailability.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $AuditPolicyAvailability.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AuditPolicyAvailability.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $AuditPolicyAvailability.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AuditPolicyAvailability.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AuditPolicyAvailability.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AuditPolicyAvailability.policy_ledger_availability_evidence_verified -or -not $AuditPolicyAvailability.write_authority_evidence_verified -or -not $AuditPolicyAvailability.ledger_evidence_verified -or -not $AuditPolicyAvailability.media_write_policy_verified -or -not $AuditPolicyAvailability.target_region_write_readback_verified -or -not $AuditPolicyAvailability.target_span_verified -or -not $AuditPolicyAvailability.audit_rollback_target_ids_verified -or -not $AuditPolicyAvailability.test_infrastructure_media_write_authority_available -or $AuditPolicyAvailability.write_authority_available -or $AuditPolicyAvailability.durable_policy_ledger_available -or $AuditPolicyAvailability.durable_audit_policy_available -or $AuditPolicyAvailability.durable_append_authority_available -or $AuditPolicyAvailability.authorizes_media_write -or $AuditPolicyAvailability.authorizes_append -or $AuditPolicyAvailability.writes_durable_audit_log -or $AuditPolicyAvailability.writes_rollback_store -or $AuditPolicyAvailability.appends_rollback_transaction -or $AuditPolicyAvailability.write_attempted) {
                    throw "Expected $Name to expose durable audit-policy availability as verified-but-denied evidence"
                }
                $AppendAuthorityAvailability = $DurablePreflight.durable_append_authority_availability
                if (-not $AppendAuthorityAvailability -or $AppendAuthorityAvailability.schema -ne $HelloRollbackDurableAppendAuthorityAvailabilitySchema -or $AppendAuthorityAvailability.id -ne $HelloRollbackDurableAppendAuthorityAvailabilityId -or $AppendAuthorityAvailability.status -ne $HelloRollbackDurableAppendAuthorityAvailabilityStatus -or $AppendAuthorityAvailability.reason -ne $HelloRollbackDurableAppendAuthorityAvailabilityReason -or -not $AppendAuthorityAvailability.availability_hash -or -not $AppendAuthorityAvailability.availability_hash.StartsWith("sha256:") -or $AppendAuthorityAvailability.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $AppendAuthorityAvailability.source_policy_ledger_availability_hash -ne $PolicyLedgerAvailability.availability_hash -or $AppendAuthorityAvailability.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $AppendAuthorityAvailability.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $AppendAuthorityAvailability.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $AppendAuthorityAvailability.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $AppendAuthorityAvailability.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $AppendAuthorityAvailability.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AppendAuthorityAvailability.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $AppendAuthorityAvailability.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AppendAuthorityAvailability.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $AppendAuthorityAvailability.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AppendAuthorityAvailability.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AppendAuthorityAvailability.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AppendAuthorityAvailability.audit_policy_availability_evidence_verified -or -not $AppendAuthorityAvailability.policy_ledger_availability_evidence_verified -or -not $AppendAuthorityAvailability.write_authority_evidence_verified -or -not $AppendAuthorityAvailability.ledger_evidence_verified -or -not $AppendAuthorityAvailability.media_write_policy_verified -or -not $AppendAuthorityAvailability.target_region_write_readback_verified -or -not $AppendAuthorityAvailability.target_span_verified -or -not $AppendAuthorityAvailability.audit_rollback_target_ids_verified -or -not $AppendAuthorityAvailability.test_infrastructure_media_write_authority_available -or $AppendAuthorityAvailability.write_authority_available -or $AppendAuthorityAvailability.durable_policy_ledger_available -or $AppendAuthorityAvailability.durable_audit_policy_available -or $AppendAuthorityAvailability.durable_append_authority_available -or $AppendAuthorityAvailability.authorizes_media_write -or $AppendAuthorityAvailability.authorizes_append -or $AppendAuthorityAvailability.writes_durable_audit_log -or $AppendAuthorityAvailability.writes_rollback_store -or $AppendAuthorityAvailability.appends_rollback_transaction -or $AppendAuthorityAvailability.write_attempted) {
                    throw "Expected $Name to expose durable append-authority availability as verified-but-denied evidence"
                }
                $TransactionAppendAvailabilityDecision = $DurablePreflight.transaction_append_availability_decision
                if (-not $TransactionAppendAvailabilityDecision -or $TransactionAppendAvailabilityDecision.schema -ne $HelloRollbackTransactionAppendAvailabilityDecisionSchema -or $TransactionAppendAvailabilityDecision.id -ne $HelloRollbackTransactionAppendAvailabilityDecisionId -or $TransactionAppendAvailabilityDecision.status -ne $HelloRollbackTransactionAppendAvailabilityDecisionStatus -or $TransactionAppendAvailabilityDecision.reason -ne $HelloRollbackTransactionAppendAvailabilityDecisionReason -or -not $TransactionAppendAvailabilityDecision.decision_hash -or -not $TransactionAppendAvailabilityDecision.decision_hash.StartsWith("sha256:") -or $TransactionAppendAvailabilityDecision.source_durable_append_authority_availability_hash -ne $AppendAuthorityAvailability.availability_hash -or $TransactionAppendAvailabilityDecision.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $TransactionAppendAvailabilityDecision.source_append_engine_readiness_decision_hash -ne $AppendEngineReadinessDecision.decision_hash -or $TransactionAppendAvailabilityDecision.source_writer_policy_preflight_hash -ne $DurableWriterPolicyPreflight.preflight_hash -or $TransactionAppendAvailabilityDecision.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $TransactionAppendAvailabilityDecision.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $TransactionAppendAvailabilityDecision.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $TransactionAppendAvailabilityDecision.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $TransactionAppendAvailabilityDecision.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $TransactionAppendAvailabilityDecision.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $TransactionAppendAvailabilityDecision.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TransactionAppendAvailabilityDecision.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TransactionAppendAvailabilityDecision.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $TransactionAppendAvailabilityDecision.durable_append_authority_availability_evidence_verified -or -not $TransactionAppendAvailabilityDecision.audit_policy_availability_evidence_verified -or -not $TransactionAppendAvailabilityDecision.append_engine_ready -or -not $TransactionAppendAvailabilityDecision.writer_policy_ready -or -not $TransactionAppendAvailabilityDecision.media_write_policy_verified -or -not $TransactionAppendAvailabilityDecision.target_region_write_readback_verified -or -not $TransactionAppendAvailabilityDecision.target_span_verified -or -not $TransactionAppendAvailabilityDecision.audit_rollback_target_ids_verified -or -not $TransactionAppendAvailabilityDecision.test_infrastructure_media_write_authority_available -or $TransactionAppendAvailabilityDecision.durable_append_authority_available -or $TransactionAppendAvailabilityDecision.durable_audit_policy_available -or $TransactionAppendAvailabilityDecision.transaction_append_available -or $TransactionAppendAvailabilityDecision.authorizes_media_write -or $TransactionAppendAvailabilityDecision.authorizes_append -or $TransactionAppendAvailabilityDecision.authorizes_transaction_append -or $TransactionAppendAvailabilityDecision.writes_durable_audit_log -or $TransactionAppendAvailabilityDecision.writes_rollback_store -or $TransactionAppendAvailabilityDecision.appends_rollback_transaction -or $TransactionAppendAvailabilityDecision.write_attempted) {
                    throw "Expected $Name to expose transaction-append availability as ready-but-denied evidence"
                }
                $TransactionAppendAuthorityDenialGate = $DurablePreflight.transaction_append_authority_denial_gate
                if (-not $TransactionAppendAuthorityDenialGate -or $TransactionAppendAuthorityDenialGate.schema -ne $HelloRollbackTransactionAppendAuthorityDenialGateSchema -or $TransactionAppendAuthorityDenialGate.id -ne $HelloRollbackTransactionAppendAuthorityDenialGateId -or $TransactionAppendAuthorityDenialGate.status -ne $HelloRollbackTransactionAppendAuthorityDenialGateStatus -or $TransactionAppendAuthorityDenialGate.reason -ne $HelloRollbackTransactionAppendAuthorityDenialGateReason -or -not $TransactionAppendAuthorityDenialGate.gate_hash -or -not $TransactionAppendAuthorityDenialGate.gate_hash.StartsWith("sha256:") -or $TransactionAppendAuthorityDenialGate.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $TransactionAppendAuthorityDenialGate.source_durable_append_authority_availability_hash -ne $AppendAuthorityAvailability.availability_hash -or $TransactionAppendAuthorityDenialGate.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $TransactionAppendAuthorityDenialGate.source_append_engine_readiness_decision_hash -ne $AppendEngineReadinessDecision.decision_hash -or $TransactionAppendAuthorityDenialGate.source_writer_policy_preflight_hash -ne $DurableWriterPolicyPreflight.preflight_hash -or $TransactionAppendAuthorityDenialGate.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $TransactionAppendAuthorityDenialGate.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $TransactionAppendAuthorityDenialGate.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $TransactionAppendAuthorityDenialGate.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $TransactionAppendAuthorityDenialGate.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $TransactionAppendAuthorityDenialGate.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $TransactionAppendAuthorityDenialGate.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TransactionAppendAuthorityDenialGate.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TransactionAppendAuthorityDenialGate.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $TransactionAppendAuthorityDenialGate.availability_decision_evidence_verified -or -not $TransactionAppendAuthorityDenialGate.append_engine_ready -or -not $TransactionAppendAuthorityDenialGate.writer_policy_ready -or -not $TransactionAppendAuthorityDenialGate.media_write_policy_verified -or -not $TransactionAppendAuthorityDenialGate.target_region_write_readback_verified -or -not $TransactionAppendAuthorityDenialGate.target_span_verified -or -not $TransactionAppendAuthorityDenialGate.audit_rollback_target_ids_verified -or -not $TransactionAppendAuthorityDenialGate.test_infrastructure_media_write_authority_available -or $TransactionAppendAuthorityDenialGate.durable_append_authority_available -or $TransactionAppendAuthorityDenialGate.durable_audit_policy_available -or $TransactionAppendAuthorityDenialGate.transaction_append_available -or -not $TransactionAppendAuthorityDenialGate.missing_transaction_append_authority -or $TransactionAppendAuthorityDenialGate.authorizes_media_write -or $TransactionAppendAuthorityDenialGate.authorizes_append -or $TransactionAppendAuthorityDenialGate.authorizes_transaction_append -or $TransactionAppendAuthorityDenialGate.writes_durable_audit_log -or $TransactionAppendAuthorityDenialGate.writes_rollback_store -or $TransactionAppendAuthorityDenialGate.appends_rollback_transaction -or $TransactionAppendAuthorityDenialGate.write_attempted) {
                    throw "Expected $Name to expose transaction-append authority denial as no-write gate evidence"
                }
                $PolicyLedgerAvailabilityDryRun = $DurablePreflight.durable_policy_ledger_availability_dry_run
                if (-not $PolicyLedgerAvailabilityDryRun -or $PolicyLedgerAvailabilityDryRun.schema -ne $HelloRollbackDurablePolicyLedgerAvailabilityDryRunSchema -or $PolicyLedgerAvailabilityDryRun.id -ne $HelloRollbackDurablePolicyLedgerAvailabilityDryRunId -or $PolicyLedgerAvailabilityDryRun.status -ne $HelloRollbackDurablePolicyLedgerAvailabilityDryRunStatus -or $PolicyLedgerAvailabilityDryRun.reason -ne $HelloRollbackDurablePolicyLedgerAvailabilityDryRunReason -or -not $PolicyLedgerAvailabilityDryRun.dry_run_hash -or -not $PolicyLedgerAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $PolicyLedgerAvailabilityDryRun.source_policy_ledger_availability_hash -ne $PolicyLedgerAvailability.availability_hash -or $PolicyLedgerAvailabilityDryRun.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $PolicyLedgerAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $PolicyLedgerAvailabilityDryRun.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $PolicyLedgerAvailabilityDryRun.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $PolicyLedgerAvailabilityDryRun.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $PolicyLedgerAvailabilityDryRun.source_authority_denial_gate_hash -ne $TransactionAppendAuthorityDenialGate.gate_hash -or $PolicyLedgerAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $PolicyLedgerAvailabilityDryRun.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $PolicyLedgerAvailabilityDryRun.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $PolicyLedgerAvailabilityDryRun.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $PolicyLedgerAvailabilityDryRun.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $PolicyLedgerAvailabilityDryRun.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $PolicyLedgerAvailabilityDryRun.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $PolicyLedgerAvailabilityDryRun.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $PolicyLedgerAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $PolicyLedgerAvailabilityDryRun.write_authority_evidence_verified -or -not $PolicyLedgerAvailabilityDryRun.ledger_evidence_verified -or -not $PolicyLedgerAvailabilityDryRun.media_write_policy_verified -or -not $PolicyLedgerAvailabilityDryRun.target_region_write_readback_verified -or -not $PolicyLedgerAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $PolicyLedgerAvailabilityDryRun.target_span_verified -or -not $PolicyLedgerAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $PolicyLedgerAvailabilityDryRun.test_infrastructure_media_write_authority_available -or $PolicyLedgerAvailabilityDryRun.write_authority_available -or $PolicyLedgerAvailabilityDryRun.durable_policy_ledger_available -or $PolicyLedgerAvailabilityDryRun.durable_audit_policy_available -or $PolicyLedgerAvailabilityDryRun.durable_append_authority_available -or $PolicyLedgerAvailabilityDryRun.transaction_append_available -or $PolicyLedgerAvailabilityDryRun.authorizes_media_write -or $PolicyLedgerAvailabilityDryRun.authorizes_append -or $PolicyLedgerAvailabilityDryRun.authorizes_transaction_append -or $PolicyLedgerAvailabilityDryRun.writes_durable_audit_log -or $PolicyLedgerAvailabilityDryRun.writes_rollback_store -or $PolicyLedgerAvailabilityDryRun.appends_rollback_transaction -or $PolicyLedgerAvailabilityDryRun.write_attempted -or $PolicyLedgerAvailabilityDryRun.applies_rollback -or $PolicyLedgerAvailabilityDryRun.installs_rollback_state) {
                    throw "Expected $Name to bind durable policy-ledger availability to the transaction-denial gate as test-media-only dry-run evidence"
                }
                $AuditPolicyAvailabilityDryRun = $DurablePreflight.durable_audit_policy_availability_dry_run
                if (-not $AuditPolicyAvailabilityDryRun -or $AuditPolicyAvailabilityDryRun.schema -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunSchema -or $AuditPolicyAvailabilityDryRun.id -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunId -or $AuditPolicyAvailabilityDryRun.status -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunStatus -or $AuditPolicyAvailabilityDryRun.reason -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunReason -or -not $AuditPolicyAvailabilityDryRun.dry_run_hash -or -not $AuditPolicyAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $AuditPolicyAvailabilityDryRun.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $AuditPolicyAvailabilityDryRun.source_policy_ledger_availability_dry_run_hash -ne $PolicyLedgerAvailabilityDryRun.dry_run_hash -or $AuditPolicyAvailabilityDryRun.source_policy_ledger_availability_hash -ne $PolicyLedgerAvailability.availability_hash -or $AuditPolicyAvailabilityDryRun.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $AuditPolicyAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $AuditPolicyAvailabilityDryRun.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $AuditPolicyAvailabilityDryRun.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $AuditPolicyAvailabilityDryRun.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $AuditPolicyAvailabilityDryRun.source_authority_denial_gate_hash -ne $TransactionAppendAuthorityDenialGate.gate_hash -or $AuditPolicyAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $AuditPolicyAvailabilityDryRun.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AuditPolicyAvailabilityDryRun.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $AuditPolicyAvailabilityDryRun.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AuditPolicyAvailabilityDryRun.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $AuditPolicyAvailabilityDryRun.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AuditPolicyAvailabilityDryRun.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AuditPolicyAvailabilityDryRun.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AuditPolicyAvailabilityDryRun.audit_policy_availability_evidence_verified -or -not $AuditPolicyAvailabilityDryRun.policy_ledger_dry_run_evidence_verified -or -not $AuditPolicyAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $AuditPolicyAvailabilityDryRun.write_authority_evidence_verified -or -not $AuditPolicyAvailabilityDryRun.ledger_evidence_verified -or -not $AuditPolicyAvailabilityDryRun.media_write_policy_verified -or -not $AuditPolicyAvailabilityDryRun.target_region_write_readback_verified -or -not $AuditPolicyAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $AuditPolicyAvailabilityDryRun.target_span_verified -or -not $AuditPolicyAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $AuditPolicyAvailabilityDryRun.test_infrastructure_media_write_authority_available -or $AuditPolicyAvailabilityDryRun.write_authority_available -or $AuditPolicyAvailabilityDryRun.durable_policy_ledger_available -or $AuditPolicyAvailabilityDryRun.durable_audit_policy_available -or $AuditPolicyAvailabilityDryRun.durable_append_authority_available -or $AuditPolicyAvailabilityDryRun.transaction_append_available -or $AuditPolicyAvailabilityDryRun.authorizes_media_write -or $AuditPolicyAvailabilityDryRun.authorizes_append -or $AuditPolicyAvailabilityDryRun.authorizes_transaction_append -or $AuditPolicyAvailabilityDryRun.writes_durable_audit_log -or $AuditPolicyAvailabilityDryRun.writes_rollback_store -or $AuditPolicyAvailabilityDryRun.appends_rollback_transaction -or $AuditPolicyAvailabilityDryRun.write_attempted -or $AuditPolicyAvailabilityDryRun.applies_rollback -or $AuditPolicyAvailabilityDryRun.installs_rollback_state) {
                    throw "Expected $Name to bind durable audit-policy availability to policy-ledger dry-run and transaction-denial evidence"
                }
                $AppendAuthorityAvailabilityDryRun = $DurablePreflight.durable_append_authority_availability_dry_run
                if (-not $AppendAuthorityAvailabilityDryRun -or $AppendAuthorityAvailabilityDryRun.schema -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunSchema -or $AppendAuthorityAvailabilityDryRun.id -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunId -or $AppendAuthorityAvailabilityDryRun.status -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunStatus -or $AppendAuthorityAvailabilityDryRun.reason -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunReason -or -not $AppendAuthorityAvailabilityDryRun.dry_run_hash -or -not $AppendAuthorityAvailabilityDryRun.dry_run_hash.StartsWith("sha256:") -or $AppendAuthorityAvailabilityDryRun.source_append_authority_availability_hash -ne $AppendAuthorityAvailability.availability_hash -or $AppendAuthorityAvailabilityDryRun.source_audit_policy_availability_dry_run_hash -ne $AuditPolicyAvailabilityDryRun.dry_run_hash -or $AppendAuthorityAvailabilityDryRun.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $AppendAuthorityAvailabilityDryRun.source_policy_ledger_availability_dry_run_hash -ne $PolicyLedgerAvailabilityDryRun.dry_run_hash -or $AppendAuthorityAvailabilityDryRun.source_policy_ledger_availability_hash -ne $PolicyLedgerAvailability.availability_hash -or $AppendAuthorityAvailabilityDryRun.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $AppendAuthorityAvailabilityDryRun.source_ledger_aware_acceptance_result_hash -ne $LedgerAwareAcceptanceResult.result_hash -or $AppendAuthorityAvailabilityDryRun.source_ledger_candidate_hash -ne $DurableAuditPolicyLedgerCandidate.ledger_candidate_hash -or $AppendAuthorityAvailabilityDryRun.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $AppendAuthorityAvailabilityDryRun.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $AppendAuthorityAvailabilityDryRun.source_authority_denial_gate_hash -ne $TransactionAppendAuthorityDenialGate.gate_hash -or $AppendAuthorityAvailabilityDryRun.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $AppendAuthorityAvailabilityDryRun.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AppendAuthorityAvailabilityDryRun.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $AppendAuthorityAvailabilityDryRun.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AppendAuthorityAvailabilityDryRun.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $AppendAuthorityAvailabilityDryRun.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $AppendAuthorityAvailabilityDryRun.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $AppendAuthorityAvailabilityDryRun.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $AppendAuthorityAvailabilityDryRun.append_authority_availability_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.audit_policy_dry_run_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.audit_policy_availability_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.policy_ledger_dry_run_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.policy_ledger_availability_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.write_authority_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.ledger_evidence_verified -or -not $AppendAuthorityAvailabilityDryRun.media_write_policy_verified -or -not $AppendAuthorityAvailabilityDryRun.target_region_write_readback_verified -or -not $AppendAuthorityAvailabilityDryRun.transaction_append_denial_gate_verified -or -not $AppendAuthorityAvailabilityDryRun.target_span_verified -or -not $AppendAuthorityAvailabilityDryRun.audit_rollback_target_ids_verified -or -not $AppendAuthorityAvailabilityDryRun.test_infrastructure_media_write_authority_available -or $AppendAuthorityAvailabilityDryRun.write_authority_available -or $AppendAuthorityAvailabilityDryRun.durable_policy_ledger_available -or $AppendAuthorityAvailabilityDryRun.durable_audit_policy_available -or $AppendAuthorityAvailabilityDryRun.durable_append_authority_available -or $AppendAuthorityAvailabilityDryRun.transaction_append_available -or $AppendAuthorityAvailabilityDryRun.authorizes_media_write -or $AppendAuthorityAvailabilityDryRun.authorizes_append -or $AppendAuthorityAvailabilityDryRun.authorizes_transaction_append -or $AppendAuthorityAvailabilityDryRun.writes_durable_audit_log -or $AppendAuthorityAvailabilityDryRun.writes_rollback_store -or $AppendAuthorityAvailabilityDryRun.appends_rollback_transaction -or $AppendAuthorityAvailabilityDryRun.write_attempted -or $AppendAuthorityAvailabilityDryRun.applies_rollback -or $AppendAuthorityAvailabilityDryRun.installs_rollback_state) {
                    throw "Expected $Name to bind durable append-authority availability to audit-policy dry-run and transaction-denial evidence"
                }
                $TransactionAppendDryRun = $DurablePreflight.transaction_append_dry_run
                if (-not $TransactionAppendDryRun -or $TransactionAppendDryRun.schema -ne $HelloRollbackTransactionAppendDryRunSchema -or $TransactionAppendDryRun.id -ne $HelloRollbackTransactionAppendDryRunId -or $TransactionAppendDryRun.status -ne $HelloRollbackTransactionAppendDryRunStatus -or $TransactionAppendDryRun.reason -ne $HelloRollbackTransactionAppendDryRunReason -or -not $TransactionAppendDryRun.dry_run_hash -or -not $TransactionAppendDryRun.dry_run_hash.StartsWith("sha256:") -or $TransactionAppendDryRun.source_authority_denial_gate_hash -ne $TransactionAppendAuthorityDenialGate.gate_hash -or $TransactionAppendDryRun.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $TransactionAppendDryRun.source_append_record_hash -ne $AppendRecord.dry_run_hash -or $TransactionAppendDryRun.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $TransactionAppendDryRun.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $TransactionAppendDryRun.planned_sector_image_hash -ne $SectorPlan.sector_image_hash -or $TransactionAppendDryRun.readback_sector_image_hash -ne $SectorPlan.sector_image_hash -or $TransactionAppendDryRun.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $TransactionAppendDryRun.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $TransactionAppendDryRun.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $TransactionAppendDryRun.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $TransactionAppendDryRun.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TransactionAppendDryRun.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TransactionAppendDryRun.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $TransactionAppendDryRun.authority_denial_gate_verified -or -not $TransactionAppendDryRun.target_span_verified -or -not $TransactionAppendDryRun.target_region_write_readback_verified -or -not $TransactionAppendDryRun.append_image_ready -or -not $TransactionAppendDryRun.blocked_by_authority_denial_gate -or -not $TransactionAppendDryRun.test_infrastructure_media_write_authority_available -or $TransactionAppendDryRun.transaction_append_available -or $TransactionAppendDryRun.authorizes_media_write -or $TransactionAppendDryRun.authorizes_append -or $TransactionAppendDryRun.authorizes_transaction_append -or $TransactionAppendDryRun.writes_durable_audit_log -or $TransactionAppendDryRun.writes_rollback_store -or $TransactionAppendDryRun.appends_rollback_transaction -or $TransactionAppendDryRun.transaction_append_attempted) {
                    throw "Expected $Name to expose transaction-append dry-run readiness while the authority-denial gate blocks append"
                }
                $TargetRegionSectorInspection = $DurablePreflight.target_region_sector_inspection
                if (-not $TargetRegionSectorInspection -or $TargetRegionSectorInspection.schema -ne $HelloRollbackTargetRegionSectorInspectionSchema -or $TargetRegionSectorInspection.id -ne $HelloRollbackTargetRegionSectorInspectionId -or $TargetRegionSectorInspection.status -ne $HelloRollbackTargetRegionSectorInspectionStatus -or $TargetRegionSectorInspection.reason -ne $HelloRollbackTargetRegionSectorInspectionReason -or -not $TargetRegionSectorInspection.inspection_hash -or -not $TargetRegionSectorInspection.inspection_hash.StartsWith("sha256:") -or $TargetRegionSectorInspection.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $TargetRegionSectorInspection.source_target_region_write_readback_hash -ne $TargetRegionWriteReadback.dry_run_hash -or $TargetRegionSectorInspection.expected_sector_image_hash -ne $SectorPlan.sector_image_hash -or $TargetRegionSectorInspection.sector_image_hash -ne $SectorPlan.sector_image_hash -or $TargetRegionSectorInspection.audit_record_image_hash -ne $AppendRecord.audit_record_image_hash -or $TargetRegionSectorInspection.rollback_transaction_image_hash -ne $AppendRecord.rollback_transaction_image_hash -or $TargetRegionSectorInspection.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegionSectorInspection.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TargetRegionSectorInspection.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or $TargetRegionSectorInspection.audit_record_offset -ne $HelloRollbackAppendSectorAuditOffset -or $TargetRegionSectorInspection.audit_record_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or $TargetRegionSectorInspection.rollback_transaction_offset -ne $HelloRollbackAppendSectorRollbackOffset -or $TargetRegionSectorInspection.rollback_transaction_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $TargetRegionSectorInspection.padding_offset -ne $HelloRollbackAppendSectorPaddingOffset -or $TargetRegionSectorInspection.padding_byte_length -ne $HelloRollbackAppendSectorPaddingByteLength -or -not $TargetRegionSectorInspection.label_found -or -not $TargetRegionSectorInspection.read_attempted -or -not $TargetRegionSectorInspection.read_completed -or -not $TargetRegionSectorInspection.sector_hash_verified -or -not $TargetRegionSectorInspection.audit_record_hash_verified -or -not $TargetRegionSectorInspection.rollback_transaction_hash_verified -or -not $TargetRegionSectorInspection.offsets_verified -or -not $TargetRegionSectorInspection.padding_zeroed -or -not $TargetRegionSectorInspection.target_span_verified -or -not $TargetRegionSectorInspection.target_region_write_readback_verified -or -not $TargetRegionSectorInspection.inspection_verified -or $TargetRegionSectorInspection.authorizes_media_write -or $TargetRegionSectorInspection.authorizes_append -or $TargetRegionSectorInspection.writes_durable_audit_log -or $TargetRegionSectorInspection.writes_rollback_store -or $TargetRegionSectorInspection.appends_rollback_transaction -or $TargetRegionSectorInspection.installs_rollback_state) {
                    throw "Expected $Name to read and parse the target-region sector without write or append authority"
                }
                $DurablePolicyWriteAuthorityDecision = $DurablePreflight.durable_policy_write_authority_decision
                if (-not $DurablePolicyWriteAuthorityDecision -or $DurablePolicyWriteAuthorityDecision.schema -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionSchema -or $DurablePolicyWriteAuthorityDecision.id -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionId -or $DurablePolicyWriteAuthorityDecision.status -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionStatus -or $DurablePolicyWriteAuthorityDecision.reason -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionReason -or -not $DurablePolicyWriteAuthorityDecision.decision_hash -or -not $DurablePolicyWriteAuthorityDecision.decision_hash.StartsWith("sha256:") -or $DurablePolicyWriteAuthorityDecision.source_durable_append_authority_availability_dry_run_hash -ne $AppendAuthorityAvailabilityDryRun.dry_run_hash -or $DurablePolicyWriteAuthorityDecision.source_transaction_append_dry_run_hash -ne $TransactionAppendDryRun.dry_run_hash -or $DurablePolicyWriteAuthorityDecision.source_write_authority_availability_hash -ne $WriteAuthorityAvailability.availability_hash -or $DurablePolicyWriteAuthorityDecision.source_audit_policy_availability_hash -ne $AuditPolicyAvailability.availability_hash -or $DurablePolicyWriteAuthorityDecision.source_durable_append_authority_availability_hash -ne $AppendAuthorityAvailability.availability_hash -or $DurablePolicyWriteAuthorityDecision.source_authority_denial_gate_hash -ne $TransactionAppendAuthorityDenialGate.gate_hash -or $DurablePolicyWriteAuthorityDecision.source_transaction_append_availability_decision_hash -ne $TransactionAppendAvailabilityDecision.decision_hash -or $DurablePolicyWriteAuthorityDecision.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $DurablePolicyWriteAuthorityDecision.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $DurablePolicyWriteAuthorityDecision.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $DurablePolicyWriteAuthorityDecision.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $DurablePolicyWriteAuthorityDecision.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $DurablePolicyWriteAuthorityDecision.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $DurablePolicyWriteAuthorityDecision.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $DurablePolicyWriteAuthorityDecision.transaction_append_dry_run_verified -or -not $DurablePolicyWriteAuthorityDecision.write_authority_evidence_verified -or -not $DurablePolicyWriteAuthorityDecision.audit_policy_availability_evidence_verified -or -not $DurablePolicyWriteAuthorityDecision.durable_append_authority_availability_evidence_verified -or -not $DurablePolicyWriteAuthorityDecision.target_span_verified -or -not $DurablePolicyWriteAuthorityDecision.test_infrastructure_media_write_authority_available -or $DurablePolicyWriteAuthorityDecision.write_authority_available -or $DurablePolicyWriteAuthorityDecision.durable_policy_ledger_available -or $DurablePolicyWriteAuthorityDecision.durable_audit_policy_available -or $DurablePolicyWriteAuthorityDecision.durable_append_authority_available -or $DurablePolicyWriteAuthorityDecision.transaction_append_available -or $DurablePolicyWriteAuthorityDecision.authorizes_media_write -or $DurablePolicyWriteAuthorityDecision.authorizes_append -or $DurablePolicyWriteAuthorityDecision.authorizes_transaction_append -or $DurablePolicyWriteAuthorityDecision.writes_durable_audit_log -or $DurablePolicyWriteAuthorityDecision.writes_rollback_store -or $DurablePolicyWriteAuthorityDecision.appends_rollback_transaction -or $DurablePolicyWriteAuthorityDecision.write_attempted -or $DurablePolicyWriteAuthorityDecision.applies_rollback) {
                    throw "Expected $Name to consume transaction-append dry-run evidence into a no-write durable policy/write-authority decision"
                }
                if ($DurablePolicyWriteAuthorityDecision.source_target_region_sector_inspection_hash -ne $TargetRegionSectorInspection.inspection_hash -or -not $DurablePolicyWriteAuthorityDecision.target_region_sector_inspection_verified) {
                    throw "Expected $Name durable policy/write-authority decision to consume target-region sector inspection evidence"
                }
                $TargetRegionWriteReadback = $DurablePreflight.target_region_write_readback_dry_run
                if (-not $TargetRegionWriteReadback -or $TargetRegionWriteReadback.schema -ne $HelloRollbackTargetRegionWriteReadbackDryRunSchema -or $TargetRegionWriteReadback.id -ne $HelloRollbackTargetRegionWriteReadbackDryRunId -or $TargetRegionWriteReadback.status -ne $HelloRollbackTargetRegionWriteReadbackDryRunStatus -or $TargetRegionWriteReadback.reason -ne $HelloRollbackTargetRegionWriteReadbackDryRunReason -or -not $TargetRegionWriteReadback.dry_run_hash -or -not $TargetRegionWriteReadback.dry_run_hash.StartsWith("sha256:") -or $TargetRegionWriteReadback.source_sector_plan_hash -ne $SectorPlan.plan_hash -or $TargetRegionWriteReadback.source_policy_preflight_hash -ne $MediaWritePolicyPreflight.preflight_hash -or $TargetRegionWriteReadback.planned_sector_image_hash -ne $SectorPlan.sector_image_hash -or $TargetRegionWriteReadback.readback_sector_image_hash -ne $SectorPlan.sector_image_hash -or $TargetRegionWriteReadback.target_region_id -ne $HelloRollbackTargetRegionDiscoveryId -or $TargetRegionWriteReadback.target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegionWriteReadback.target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TargetRegionWriteReadback.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $TargetRegionWriteReadback.label_found -or -not $TargetRegionWriteReadback.target_range_ready -or -not $TargetRegionWriteReadback.test_infrastructure_media_write_authority_available -or -not $TargetRegionWriteReadback.write_attempted -or -not $TargetRegionWriteReadback.write_completed -or -not $TargetRegionWriteReadback.readback_completed -or -not $TargetRegionWriteReadback.readback_matches_planned_image -or $TargetRegionWriteReadback.authorizes_media_write -or $TargetRegionWriteReadback.authorizes_append -or $TargetRegionWriteReadback.writes_durable_audit_log -or $TargetRegionWriteReadback.writes_rollback_store -or $TargetRegionWriteReadback.appends_rollback_transaction -or $TargetRegionWriteReadback.installs_rollback_state) {
                    throw "Expected $Name to write/read back the planned sector image on the dedicated target region without durable authority"
                }
            }
        }

        $AssertHelloRollbackScratchWriterDryRunBinding = {
            param(
                [string]$Name,
                [object]$Bindings
            )

            if (-not $Bindings -or $Bindings.rollback_transaction_writer_storage_scratch_dry_run_schema -ne $HelloRollbackScratchWriterDryRunSchema -or $Bindings.rollback_transaction_writer_storage_scratch_dry_run_id -ne $HelloRollbackScratchWriterDryRunId -or $Bindings.rollback_transaction_writer_storage_scratch_dry_run_status -ne $HelloRollbackScratchWriterDryRunStatus -or $Bindings.rollback_transaction_writer_storage_scratch_dry_run_reason -ne $HelloRollbackScratchWriterDryRunReason -or $Bindings.rollback_transaction_writer_storage_scratch_write_authority_id -ne $HelloRollbackScratchBlockWriteAuthorityId -or $Bindings.rollback_transaction_writer_storage_scratch_region_id -ne $HelloRollbackScratchBlockRegionId -or $Bindings.rollback_transaction_writer_storage_scratch_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_scratch_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_scratch_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_scratch_target_owned -or -not $Bindings.rollback_transaction_writer_storage_scratch_target_within_bounds -or -not $Bindings.rollback_transaction_writer_storage_scratch_target_no_metadata_overlap -or -not $Bindings.rollback_transaction_writer_storage_scratch_target_ready) {
                throw "Expected $Name to bind scratch-only rollback writer dry-run readiness evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_append_record_dry_run_schema -ne $HelloRollbackAppendRecordDryRunSchema -or $Bindings.rollback_transaction_writer_storage_append_record_dry_run_id -ne $HelloRollbackAppendRecordDryRunId -or $Bindings.rollback_transaction_writer_storage_append_record_dry_run_status -ne $HelloRollbackAppendRecordDryRunStatus -or $Bindings.rollback_transaction_writer_storage_append_record_dry_run_reason -ne $HelloRollbackAppendRecordDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_record_canonicalization -ne $HelloRollbackAppendRecordCanonicalization -or -not $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or -not $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_record_audit_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or -not $Bindings.rollback_transaction_writer_storage_append_record_rollback_image_hash -or -not $Bindings.rollback_transaction_writer_storage_append_record_rollback_image_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_record_rollback_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $Bindings.rollback_transaction_writer_storage_append_record_total_byte_length -ne $HelloRollbackAppendRecordTotalByteLength -or $Bindings.rollback_transaction_writer_storage_append_record_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_append_record_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_append_record_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_append_record_source_payload_hash -or -not $Bindings.rollback_transaction_writer_storage_append_record_source_payload_hash.StartsWith("sha256:") -or -not $Bindings.rollback_transaction_writer_storage_append_record_source_provenance_hash -or -not $Bindings.rollback_transaction_writer_storage_append_record_source_provenance_hash.StartsWith("sha256:") -or -not $Bindings.rollback_transaction_writer_storage_append_record_target_range_ready -or $Bindings.rollback_transaction_writer_storage_append_record_authorizes_append -or $Bindings.rollback_transaction_writer_storage_append_record_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_append_record_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_append_record_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_append_record_write_attempted) {
                throw "Expected $Name to bind canonical append-record dry-run image/hash evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_append_sector_plan_dry_run_schema -ne $HelloRollbackAppendSectorPlanDryRunSchema -or $Bindings.rollback_transaction_writer_storage_append_sector_plan_dry_run_id -ne $HelloRollbackAppendSectorPlanDryRunId -or $Bindings.rollback_transaction_writer_storage_append_sector_plan_dry_run_status -ne $HelloRollbackAppendSectorPlanDryRunStatus -or $Bindings.rollback_transaction_writer_storage_append_sector_plan_dry_run_reason -ne $HelloRollbackAppendSectorPlanDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or -not $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_sector_plan_canonicalization -ne $HelloRollbackAppendSectorPlanCanonicalization -or -not $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or -not $Bindings.rollback_transaction_writer_storage_append_sector_image_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_sector_size_bytes -ne $HelloRollbackScratchRegionByteCount -or $Bindings.rollback_transaction_writer_storage_append_sector_audit_record_offset -ne $HelloRollbackAppendSectorAuditOffset -or $Bindings.rollback_transaction_writer_storage_append_sector_audit_record_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or $Bindings.rollback_transaction_writer_storage_append_sector_rollback_transaction_offset -ne $HelloRollbackAppendSectorRollbackOffset -or $Bindings.rollback_transaction_writer_storage_append_sector_rollback_transaction_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $Bindings.rollback_transaction_writer_storage_append_sector_padding_policy -ne $HelloRollbackAppendSectorPaddingPolicy -or $Bindings.rollback_transaction_writer_storage_append_sector_padding_offset -ne $HelloRollbackAppendSectorPaddingOffset -or $Bindings.rollback_transaction_writer_storage_append_sector_padding_byte_length -ne $HelloRollbackAppendSectorPaddingByteLength -or $Bindings.rollback_transaction_writer_storage_append_sector_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_append_sector_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_append_sector_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or $Bindings.rollback_transaction_writer_storage_append_sector_source_record_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_append_sector_target_range_ready -or $Bindings.rollback_transaction_writer_storage_append_sector_authorizes_append -or $Bindings.rollback_transaction_writer_storage_append_sector_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_append_sector_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_append_sector_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_append_sector_write_attempted) {
                throw "Expected $Name to bind scratch append-sector plan/hash evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_schema -ne $HelloRollbackAppendSectorWriteReadbackDryRunSchema -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_id -ne $HelloRollbackAppendSectorWriteReadbackDryRunId -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_status -ne $HelloRollbackAppendSectorWriteReadbackDryRunStatus -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_reason -ne $HelloRollbackAppendSectorWriteReadbackDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_source_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_planned_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_readback_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_label_found -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_write_attempted -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_write_completed -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_readback_completed -or -not $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_matches_planned_image -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_authorizes_append -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_appends_rollback_transaction) {
                throw "Expected $Name to bind scratch-only append-sector write/readback evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_schema -ne $HelloRollbackDurableAppendAuthorityPreflightSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_id -ne $HelloRollbackDurableAppendAuthorityPreflightId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_status -ne $HelloRollbackDurableAppendAuthorityPreflightStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_reason -ne $HelloRollbackDurableAppendAuthorityPreflightReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_source_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_remaining_denial_reason -ne $HelloRollbackDurableAppendAuthorityRemainingReason -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_audit_writer_fact_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_writer_fact_id -ne $HelloRollbackTransactionWriterStorageTargetId -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_write_readback_verified -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_used_as_durable_authority -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_durable_audit_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_store_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_transaction_append_writer_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_appends_rollback_transaction) {
                throw "Expected $Name to bind denied durable append-authority preflight evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_schema -ne $HelloRollbackDurableWriterPolicyPreflightSchema -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_id -ne $HelloRollbackDurableWriterPolicyPreflightId -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_status -ne $HelloRollbackDurableWriterPolicyPreflightStatus -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_reason -ne $HelloRollbackDurableWriterPolicyPreflightReason -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_append_record_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_test_media_write_authority_available -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_durable_audit_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_transaction_append_writer_available -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_write_attempted) {
                throw "Expected $Name to bind denied durable writer-policy preflight evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_schema -ne $HelloRollbackDurableAppendTransactionAuthorizationGateSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_id -ne $HelloRollbackDurableAppendTransactionAuthorizationGateId -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_status -ne $HelloRollbackDurableAppendTransactionAuthorizationGateStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_reason -ne $HelloRollbackDurableAppendTransactionAuthorizationGateReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_writer_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_append_record_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_test_media_write_authority_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_append_engine_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_durable_audit_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_writer_available -or -not $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_transaction_append_writer_available -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_write_attempted) {
                throw "Expected $Name to bind denied durable append/transaction authorization gate evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_schema -ne $HelloRollbackAppendEngineReadinessDecisionSchema -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_id -ne $HelloRollbackAppendEngineReadinessDecisionId -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_status -ne $HelloRollbackAppendEngineReadinessDecisionStatus -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_reason -ne $HelloRollbackAppendEngineReadinessDecisionReason -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_hash -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_source_authorization_gate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_hash -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_source_writer_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_source_append_record_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_test_media_write_authority_available -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_append_engine_available -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_durable_audit_writer_available -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_rollback_store_writer_available -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_transaction_append_writer_available -or -not $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_ready -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_append -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_write_attempted) {
                throw "Expected $Name to bind ready, non-authorizing append-engine decision evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_schema -ne $HelloRollbackTargetRegionDiscoverySchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_id -ne $HelloRollbackTargetRegionDiscoveryId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_status -ne $HelloRollbackTargetRegionDiscoveryStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_reason -ne $HelloRollbackTargetRegionDiscoveryReason -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_source -ne $HelloRollbackTargetRegionDiscoverySource -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_inventory_scheme -ne $HelloRollbackPartitionInventoryScheme -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_entry_count -ne 0 -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_mbr_signature_valid -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_present -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_is_scratch -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_boot_metadata -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_scratch -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_scratch_rejected_as_durable_authority -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_durable_region_available) {
                throw "Expected $Name to bind read-only durable target-region discovery evidence without authorizing append"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_id -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_status -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_reason -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_id -ne $HelloRollbackTargetRegionWriterContractId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_status -ne $HelloRollbackTargetRegionWriterContractStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_reason -ne $HelloRollbackTargetRegionWriterContractReason -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_method -ne $HelloRollbackTransactionWriterStorageFoundationOwner -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_required -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_reason -ne "media_write_authority_missing" -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_required -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_reason -ne "durable_audit_policy_missing" -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_write_attempted) {
                throw "Expected $Name to bind target-region media-write policy preflight into RAM audit evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_media_write_authority_gate_schema -ne $HelloRollbackMediaWriteAuthorityGateSchema -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_id -ne $HelloRollbackMediaWriteAuthorityGateId -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_status -ne $HelloRollbackMediaWriteAuthorityGateStatus -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_reason -ne $HelloRollbackMediaWriteAuthorityGateReason -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_hash -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_durable_append_authority_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_hash -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_id -ne $HelloRollbackTargetRegionWriterContractId -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_status -ne $HelloRollbackTargetRegionWriterContractStatus -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_reason -ne $HelloRollbackTargetRegionWriterContractReason -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_owner_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_schema_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_required -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_reason -ne $HelloRollbackTestMediaWriteAuthorityReason -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_test_media_write_authority_available -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_required -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_reason -ne "durable_audit_policy_missing" -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_appends_rollback_transaction -or -not $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_target_region_write_attempted -or $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_write_attempted) {
                throw "Expected $Name to bind the media-write-authority gate into RAM audit evidence without write attempts"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_schema -ne $HelloRollbackDurableAppendAuthorityDecisionSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_id -ne $HelloRollbackDurableAppendAuthorityDecisionId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_status -ne $HelloRollbackDurableAppendAuthorityDecisionStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_reason -ne $HelloRollbackDurableAppendAuthorityDecisionReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_durable_append_authority_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_writer_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_append_engine_readiness_decision_hash -ne $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_media_write_authority_gate_hash -ne $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_writer_policy_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_append_engine_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_media_write_gate_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_write_attempted) {
                throw "Expected $Name to bind denied durable append-authority decision evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_schema -ne $HelloRollbackDurableAuditPolicyDecisionSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_id -ne $HelloRollbackDurableAuditPolicyDecisionId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_status -ne $HelloRollbackDurableAuditPolicyDecisionStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_reason -ne $HelloRollbackDurableAuditPolicyDecisionReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_source_durable_append_authority_decision_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_source_media_write_authority_gate_hash -ne $Bindings.rollback_transaction_writer_storage_media_write_authority_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_append_engine_ready -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_write_attempted) {
                throw "Expected $Name to bind denied durable audit-policy decision evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_schema -ne $HelloRollbackDurableAuditPolicyCandidateSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_id -ne $HelloRollbackDurableAuditPolicyCandidateId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_status -ne $HelloRollbackDurableAuditPolicyCandidateStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_reason -ne $HelloRollbackDurableAuditPolicyCandidateReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_durable_audit_policy_decision_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_audit_record_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_write_attempted) {
                throw "Expected $Name to bind no-write durable audit-policy candidate evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_schema -ne $HelloRollbackDurableAuditPolicyAcceptanceGateSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_id -ne $HelloRollbackDurableAuditPolicyAcceptanceGateId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_status -ne $HelloRollbackDurableAuditPolicyAcceptanceGateStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_reason -ne $HelloRollbackDurableAuditPolicyAcceptanceGateReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_decision_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_audit_record_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_candidate_available -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_media_write_policy_verified -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_attempted) {
                throw "Expected $Name to bind fail-closed durable audit-policy acceptance-gate evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_schema -ne $HelloRollbackDurableAuditPolicyLedgerCandidateSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_id -ne $HelloRollbackDurableAuditPolicyLedgerCandidateId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_status -ne $HelloRollbackDurableAuditPolicyLedgerCandidateStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_reason -ne $HelloRollbackDurableAuditPolicyLedgerCandidateReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_acceptance_gate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_decision_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_audit_record_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_read_only_available -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_candidate_available -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_media_write_policy_verified -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_attempted) {
                throw "Expected $Name to bind read-only durable audit-policy ledger-candidate evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_schema -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_id -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_status -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_reason -ne $HelloRollbackDurableAuditPolicyLedgerAwareAcceptanceResultReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_acceptance_gate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_decision_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_audit_record_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_read_only_available -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_ledger_evidence_verified -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_attempted) {
                throw "Expected $Name to bind ledger-aware acceptance result evidence without write authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_schema -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilitySchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_id -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_status -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_reason -ne $HelloRollbackDurableAuditPolicyWriteAuthorityAvailabilityReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_attempted) {
                throw "Expected $Name to bind write-authority availability evidence without durable write authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_schema -ne $HelloRollbackDurablePolicyLedgerAvailabilitySchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_id -ne $HelloRollbackDurablePolicyLedgerAvailabilityId -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_status -ne $HelloRollbackDurablePolicyLedgerAvailabilityStatus -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_reason -ne $HelloRollbackDurablePolicyLedgerAvailabilityReason -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_attempted) {
                throw "Expected $Name to bind durable policy-ledger availability evidence without durable policy ledger"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_schema -ne $HelloRollbackDurableAuditPolicyAvailabilitySchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_id -ne $HelloRollbackDurableAuditPolicyAvailabilityId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_status -ne $HelloRollbackDurableAuditPolicyAvailabilityStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_reason -ne $HelloRollbackDurableAuditPolicyAvailabilityReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_ledger_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_policy_ledger_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_write_attempted) {
                throw "Expected $Name to bind durable audit-policy availability evidence without durable audit policy"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_schema -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_id -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_status -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunStatus -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_reason -ne $HelloRollbackDurableAuditPolicyAvailabilityDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_dry_run_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_authority_denial_gate_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_transaction_append_availability_decision_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_policy_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_dry_run_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_available -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_attempted -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_applies_rollback -or $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_installs_rollback_state) {
                throw "Expected $Name to bind durable audit-policy availability dry-run evidence without durable authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_schema -ne $HelloRollbackDurableAppendAuthorityAvailabilitySchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_id -ne $HelloRollbackDurableAppendAuthorityAvailabilityId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_status -ne $HelloRollbackDurableAppendAuthorityAvailabilityStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_reason -ne $HelloRollbackDurableAppendAuthorityAvailabilityReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_ledger_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_audit_policy_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_policy_ledger_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_write_attempted) {
                throw "Expected $Name to bind durable append-authority availability evidence without durable append authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_schema -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_id -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_status -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunStatus -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_reason -ne $HelloRollbackDurableAppendAuthorityAvailabilityDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_append_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_dry_run_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_dry_run_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_result_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_ledger_candidate_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_authority_denial_gate_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_transaction_append_availability_decision_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_append_authority_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_dry_run_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_dry_run_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_ledger_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_denial_gate_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_available -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_attempted -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_applies_rollback -or $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_installs_rollback_state) {
                throw "Expected $Name to bind durable append-authority availability dry-run evidence without durable authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_schema -ne $HelloRollbackTransactionAppendAvailabilityDecisionSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_id -ne $HelloRollbackTransactionAppendAvailabilityDecisionId -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_status -ne $HelloRollbackTransactionAppendAvailabilityDecisionStatus -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_reason -ne $HelloRollbackTransactionAppendAvailabilityDecisionReason -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_durable_append_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_append_engine_readiness_decision_hash -ne $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_writer_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_policy_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_append_engine_ready -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_writer_policy_ready -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_transaction_append_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_write_attempted) {
                throw "Expected $Name to bind transaction-append availability decision without append authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_schema -ne $HelloRollbackTransactionAppendAuthorityDenialGateSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_id -ne $HelloRollbackTransactionAppendAuthorityDenialGateId -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_status -ne $HelloRollbackTransactionAppendAuthorityDenialGateStatus -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_reason -ne $HelloRollbackTransactionAppendAuthorityDenialGateReason -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_transaction_append_availability_decision_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_durable_append_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_append_engine_readiness_decision_hash -ne $Bindings.rollback_transaction_writer_storage_append_engine_readiness_decision_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_writer_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_availability_decision_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_append_engine_ready -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writer_policy_ready -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_media_write_policy_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_rollback_target_ids_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_transaction_append_available -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_missing_transaction_append_authority -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_write_attempted) {
                throw "Expected $Name to bind transaction-append authority denial gate without append authority"
            }
            if ($Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_schema -ne $HelloRollbackTransactionAppendDryRunSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_id -ne $HelloRollbackTransactionAppendDryRunId -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_status -ne $HelloRollbackTransactionAppendDryRunStatus -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_reason -ne $HelloRollbackTransactionAppendDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_source_authority_denial_gate_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_source_transaction_append_availability_decision_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_source_append_record_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_planned_sector_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_readback_sector_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_authority_denial_gate_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_append_image_ready -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_blocked_by_authority_denial_gate -or -not $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_available -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_attempted) {
                throw "Expected $Name to bind transaction-append dry-run evidence without appending a rollback transaction"
            }
            if ($Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_schema -ne $HelloRollbackTargetRegionSectorInspectionSchema -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_id -ne $HelloRollbackTargetRegionSectorInspectionId -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_status -ne $HelloRollbackTargetRegionSectorInspectionStatus -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_reason -ne $HelloRollbackTargetRegionSectorInspectionReason -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_hash -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_expected_sector_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_sector_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_audit_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_record_rollback_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_offset -ne $HelloRollbackAppendSectorAuditOffset -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_byte_length -ne $HelloRollbackAppendRecordAuditByteLength -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_offset -ne $HelloRollbackAppendSectorRollbackOffset -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_byte_length -ne $HelloRollbackAppendRecordRollbackByteLength -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_padding_offset -ne $HelloRollbackAppendSectorPaddingOffset -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_padding_byte_length -ne $HelloRollbackAppendSectorPaddingByteLength -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_label_found -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_read_attempted -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_read_completed -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_sector_hash_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_hash_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_hash_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_offsets_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_padding_zeroed -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_target_region_write_readback_verified -or -not $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_verified -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_append -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_installs_rollback_state) {
                throw "Expected $Name to bind target-region sector inspection evidence without authorizing append"
            }
            if ($Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_schema -ne $HelloRecoveryRollbackInspectSourceReferenceSchema -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_id -ne $HelloRecoveryRollbackInspectSourceReferenceId -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_status -ne $HelloRecoveryRollbackInspectSourceReferenceStatus -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_reason -ne $HelloRecoveryRollbackInspectSourceReferenceReason -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_method -ne "recovery.rollback_inspect" -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_event_id -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_id -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_inspection_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_hash -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_target_region_sector_inspection_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_hash -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_target_region_write_readback_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_available -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_matches_sector_inspection -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_status -ne "retained_audit_event_verified" -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified" -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_event_retained -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_retained -or -not $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_validated -or $Bindings.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_authorizes_rollback_apply) {
                throw "Expected $Name to bind retained recovery inspect source reference without authorizing rollback apply"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_schema -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionSchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_id -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionId -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_status -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionStatus -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_reason -ne $HelloRollbackDurablePolicyWriteAuthorityDecisionReason -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_dry_run_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_dry_run_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_dry_run_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_write_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_audit_policy_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_audit_policy_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_availability_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_authority_denial_gate_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_availability_decision_hash -ne $Bindings.rollback_transaction_writer_storage_transaction_append_availability_decision_hash -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_dry_run_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_policy_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_availability_evidence_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_span_verified -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_test_media_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_policy_ledger_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_audit_policy_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_available -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_append -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_transaction_append -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_attempted -or $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_applies_rollback) {
                throw "Expected $Name to bind durable policy/write-authority decision evidence without authorizing rollback writes"
            }
            if ($Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_target_region_sector_inspection_hash -ne $Bindings.rollback_transaction_writer_storage_target_region_sector_inspection_hash -or -not $Bindings.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_region_sector_inspection_verified) {
                throw "Expected $Name durable policy/write-authority decision binding to consume target-region sector inspection evidence"
            }
            if ($Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_schema -ne $HelloRollbackTargetRegionWriteReadbackDryRunSchema -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_id -ne $HelloRollbackTargetRegionWriteReadbackDryRunId -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_status -ne $HelloRollbackTargetRegionWriteReadbackDryRunStatus -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_reason -ne $HelloRollbackTargetRegionWriteReadbackDryRunReason -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash.StartsWith("sha256:") -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_source_sector_plan_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_plan_hash -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_source_policy_preflight_hash -ne $Bindings.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_planned_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_readback_image_hash -ne $Bindings.rollback_transaction_writer_storage_append_sector_image_hash -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_target_start_lba -ne $HelloRollbackScratchRegionStartLba -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_target_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_label_found -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_target_range_ready -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_test_media_write_authority_available -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_write_attempted -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_write_completed -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_readback_completed -or -not $Bindings.rollback_transaction_writer_storage_target_region_write_readback_matches_planned_image -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_authorizes_media_write -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_authorizes_append -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_writes_durable_audit_log -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_writes_rollback_store -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_appends_rollback_transaction -or $Bindings.rollback_transaction_writer_storage_target_region_write_readback_installs_rollback_state) {
                throw "Expected $Name to bind target-region write/readback evidence without durable append authority"
            }
        }

        $AssertHelloLoadPlanPreflight = {
            param(
                [string]$Name,
                [object]$Preflight,
                [string]$DescriptorSourceLocator,
                [string]$DescriptorSourceHash,
                [string]$ArtifactIdentityHash,
                [string]$ArtifactContentHash,
                [string]$ArtifactReferenceHash,
                [string]$ArtifactBytesHash,
                [string]$ExpectedArtifactIdentityId = $HelloArtifactIdentityId
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
            if ($Preflight.artifact_identity_id -ne $ExpectedArtifactIdentityId -or $Preflight.artifact_identity_hash -ne $ArtifactIdentityHash) {
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

        $AssertHelloState = {
            param(
                [string]$Name,
                [object]$State,
                [int]$ExpectedCounter,
                [string]$ExpectedVersion
            )

            if (-not $State) {
                throw "Expected $Name to expose Hello RAM-only state"
            }
            if ($State.schema -ne $HelloStateSchema -or $State.id -ne $HelloStateId) {
                throw "Expected $Name Hello state schema/id"
            }
            if ($State.scope -ne "current_boot" -or $State.classification -ne "local_only" -or $State.persistence -ne "none") {
                throw "Expected $Name Hello state current_boot/local_only/none"
            }
            if ($State.service_id -ne "svc.demo.hello" -or $State.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name Hello state to bind the RAM-only service slot"
            }
            if ($State.version -ne $ExpectedVersion -or $State.state_counter -ne $ExpectedCounter) {
                throw "Expected $Name Hello state version $ExpectedVersion counter $ExpectedCounter"
            }
            if (-not $State.state_hash -or -not $State.state_hash.StartsWith("sha256:")) {
                throw "Expected $Name Hello state hash"
            }
            if ($State.writes_persistent_state) {
                throw "Expected $Name Hello state to remain RAM-only"
            }

            return $State.state_hash
        }

        $AssertHelloStateMigration = {
            param(
                [string]$Name,
                [object]$Migration,
                [string]$ExpectedFromVersion,
                [string]$ExpectedToVersion,
                [int]$ExpectedCounter,
                [string]$ExpectedStateHash
            )

            if (-not $Migration) {
                throw "Expected $Name to expose Hello state migration evidence"
            }
            if ($Migration.schema -ne $HelloStateMigrationSchema -or $Migration.id -ne $HelloStateMigrationId) {
                throw "Expected $Name Hello state migration schema/id"
            }
            if ($Migration.scope -ne "current_boot" -or $Migration.classification -ne "local_only" -or $Migration.persistence -ne "none") {
                throw "Expected $Name Hello state migration current_boot/local_only/none"
            }
            if (-not $Migration.migration_hash -or -not $Migration.migration_hash.StartsWith("sha256:")) {
                throw "Expected $Name Hello state migration hash"
            }
            if ($Migration.service_id -ne "svc.demo.hello" -or $Migration.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name Hello state migration to bind the RAM-only service slot"
            }
            if ($Migration.from_version -ne $ExpectedFromVersion -or $Migration.to_version -ne $ExpectedToVersion) {
                throw "Expected $Name Hello state migration from $ExpectedFromVersion to $ExpectedToVersion"
            }
            if ($Migration.pre_state_counter -ne $ExpectedCounter -or $Migration.post_state_counter -ne $ExpectedCounter) {
                throw "Expected $Name Hello state migration to preserve counter $ExpectedCounter"
            }
            if ($Migration.pre_state_hash -ne $ExpectedStateHash -or $Migration.post_state_hash -ne $ExpectedStateHash) {
                throw "Expected $Name Hello state migration to preserve state hash"
            }
            if (-not $Migration.state_preserved -or -not $Migration.accepted) {
                throw "Expected $Name Hello state migration to be accepted and preserved"
            }
            if ($Migration.writes_persistent_state -or $Migration.writes_durable_audit_log -or $Migration.installs_rollback_plan) {
                throw "Expected $Name Hello state migration to deny persistence, durable audit, and rollback install"
            }

            return $Migration.migration_hash
        }

        $AssertHelloHotSwapProbation = {
            param(
                [string]$Name,
                [object]$Probation,
                [string]$ExpectedPreviousVersion,
                [string]$ExpectedNewVersion,
                [int]$ExpectedPreviousGeneration,
                [int]$ExpectedNewGeneration,
                [string]$ExpectedPreviousStateHash,
                [string]$ExpectedNewStateHash,
                [string]$ExpectedPreviousArtifactIdentityHash,
                [string]$ExpectedNewArtifactIdentityHash,
                [string]$ExpectedStateMigrationHash
            )

            if (-not $Probation) {
                throw "Expected $Name to expose Hello hot-swap probation evidence"
            }
            if ($Probation.schema -ne $HelloHotSwapProbationSchema -or $Probation.id -ne $HelloHotSwapProbationId) {
                throw "Expected $Name Hello hot-swap probation schema/id"
            }
            if ($Probation.scope -ne "current_boot" -or $Probation.classification -ne "local_only" -or $Probation.persistence -ne "none") {
                throw "Expected $Name Hello hot-swap probation current_boot/local_only/none"
            }
            if ($Probation.status -ne $HelloHotSwapProbationStatus) {
                throw "Expected $Name Hello hot-swap probation status"
            }
            if (-not $Probation.probation_hash -or -not $Probation.probation_hash.StartsWith("sha256:")) {
                throw "Expected $Name Hello hot-swap probation hash"
            }
            if ($Probation.service_id -ne "svc.demo.hello" -or $Probation.ram_only_service_slot_id -ne $HelloRamOnlyServiceSlotId) {
                throw "Expected $Name Hello hot-swap probation to bind the RAM-only service slot"
            }
            if ($Probation.previous_version -ne $ExpectedPreviousVersion -or $Probation.new_version -ne $ExpectedNewVersion) {
                throw "Expected $Name Hello hot-swap probation previous/new versions"
            }
            if ($Probation.previous_generation -ne $ExpectedPreviousGeneration -or $Probation.new_generation -ne $ExpectedNewGeneration) {
                throw "Expected $Name Hello hot-swap probation previous/new generations"
            }
            if ($Probation.previous_state_hash -ne $ExpectedPreviousStateHash -or $Probation.new_state_hash -ne $ExpectedNewStateHash) {
                throw "Expected $Name Hello hot-swap probation previous/new state hashes"
            }
            if ($Probation.previous_state_counter -ne 3 -or $Probation.new_state_counter -ne 3) {
                throw "Expected $Name Hello hot-swap probation to preserve state counter 3"
            }
            if ($Probation.previous_artifact_identity_hash -ne $ExpectedPreviousArtifactIdentityHash -or $Probation.new_artifact_identity_hash -ne $ExpectedNewArtifactIdentityHash) {
                throw "Expected $Name Hello hot-swap probation previous/new artifact identity hashes"
            }
            if ($Probation.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected $Name Hello hot-swap probation to bind the state migration hash"
            }
            if (-not $Probation.accepted -or $Probation.loads_candidate_bytes -or $Probation.maps_executable_pages -or $Probation.writes_persistent_state -or $Probation.writes_durable_audit_log -or $Probation.installs_rollback_plan -or $Probation.applies_rollback) {
                throw "Expected $Name Hello hot-swap probation accepted but no candidate execution, persistence, durable audit, rollback install, or rollback apply"
            }

            return $Probation.probation_hash
        }

        $AssertHelloRollbackPreview = {
            param(
                [object]$Preview,
                [string]$ExpectedProbationHash,
                [string]$ExpectedTargetVersion,
                [string]$ExpectedCurrentVersion,
                [int]$ExpectedTargetGeneration,
                [int]$ExpectedCurrentGeneration,
                [string]$ExpectedTargetArtifactIdentityHash,
                [string]$ExpectedCurrentArtifactIdentityHash,
                [string]$ExpectedStateHash,
                [string]$ExpectedStateMigrationHash
            )

            if ($Preview.schema -ne $HelloRollbackPreviewSchema -or $Preview.id -ne $HelloRollbackPreviewId) {
                throw "Expected Hello rollback preview schema/id"
            }
            if ($Preview.scope -ne "current_boot" -or $Preview.classification -ne "local_only" -or $Preview.persistence -ne "none" -or -not $Preview.read_only) {
                throw "Expected Hello rollback preview to be current_boot/local_only/read-only"
            }
            if ($Preview.status -ne $HelloRollbackPreviewStatus -or -not $Preview.preview_available) {
                throw "Expected Hello rollback preview to be available"
            }
            if (-not $Preview.preview_hash -or -not $Preview.preview_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback preview hash"
            }
            if ($Preview.service_id -ne "svc.demo.hello" -or $Preview.source_probation.probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback preview to bind the retained probation evidence"
            }
            if ($Preview.rollback_target.version -ne $ExpectedTargetVersion -or $Preview.rollback_target.generation -ne $ExpectedTargetGeneration) {
                throw "Expected Hello rollback preview target version/generation"
            }
            if ($Preview.current_candidate.version -ne $ExpectedCurrentVersion -or $Preview.current_candidate.generation -ne $ExpectedCurrentGeneration -or $Preview.current_generation -ne $ExpectedCurrentGeneration) {
                throw "Expected Hello rollback preview current candidate version/generation"
            }
            if ($Preview.rollback_target.artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $Preview.current_candidate.artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash) {
                throw "Expected Hello rollback preview artifact identities"
            }
            if ($Preview.rollback_target.state_hash -ne $ExpectedStateHash -or $Preview.current_candidate.state_hash -ne $ExpectedStateHash -or $Preview.current_state.state_hash -ne $ExpectedStateHash) {
                throw "Expected Hello rollback preview to retain state hashes"
            }
            if ($Preview.rollback_target.state_counter -ne 3 -or $Preview.current_candidate.state_counter -ne 3 -or $Preview.current_state.state_counter -ne 3) {
                throw "Expected Hello rollback preview to retain state counter 3"
            }
            if ($Preview.state_migration.migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback preview to bind the state migration hash"
            }
            if ($Preview.denied_surfaces.mutates_service_state -or $Preview.denied_surfaces.applies_rollback -or $Preview.denied_surfaces.installs_rollback_plan) {
                throw "Expected Hello rollback preview to avoid service mutation and rollback apply"
            }
            if ($Preview.denied_surfaces.persistent_install -ne "denied" -or $Preview.denied_surfaces.durable_audit_write -ne "denied" -or $Preview.denied_surfaces.external_artifact_load -ne "denied" -or $Preview.denied_surfaces.candidate_artifact_execution -ne "denied" -or $Preview.denied_surfaces.executable_mapping -ne "denied" -or $Preview.denied_surfaces.provider_auto_load -ne "denied" -or $Preview.denied_surfaces.broad_mutation -ne "denied") {
                throw "Expected Hello rollback preview to keep unsafe surfaces denied"
            }
        }

        $AssertHelloRollbackApplyDenied = {
            param(
                [object]$Apply,
                [string]$ExpectedProbationHash,
                [string]$ExpectedPreviewHash,
                [string]$ExpectedTargetVersion,
                [string]$ExpectedCurrentVersion,
                [int]$ExpectedTargetGeneration,
                [int]$ExpectedCurrentGeneration,
                [string]$ExpectedTargetArtifactIdentityHash,
                [string]$ExpectedCurrentArtifactIdentityHash,
                [string]$ExpectedStateHash,
                [string]$ExpectedStateMigrationHash
            )

            if ($Apply.code -ne "capability_denied" -or $Apply.schema -ne $HelloRollbackApplySchema -or $Apply.id -ne $HelloRollbackApplyId) {
                throw "Expected Hello rollback apply to return structured capability_denied"
            }
            if ($Apply.scope -ne "current_boot" -or $Apply.classification -ne "local_only" -or $Apply.persistence -ne "none" -or $Apply.status -ne $HelloRollbackApplyStatus) {
                throw "Expected Hello rollback apply denial to be current_boot/local_only/non-persistent"
            }
            if ($Apply.reason -ne "rollback_apply_authority_missing") {
                throw "Expected Hello rollback apply to be denied by missing apply authority"
            }
            if (-not $Apply.rollback_apply_hash -or -not $Apply.rollback_apply_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback apply denial hash"
            }
            if (-not $Apply.source_durable_policy_write_authority_decision_hash -or -not $Apply.source_durable_policy_write_authority_decision_hash.StartsWith("sha256:") -or -not $Apply.source_recovery_rollback_inspect_source_reference_hash -or -not $Apply.source_recovery_rollback_inspect_source_reference_hash.StartsWith("sha256:") -or -not $Apply.retained_durable_policy_write_authority_decision_verified -or -not $Apply.retained_recovery_rollback_inspect_source_reference_validated) {
                throw "Expected Hello rollback apply denial hash to consume retained durable-policy and recovery-inspect source evidence"
            }
            $Preflight = $Apply.rollback_transaction_preflight
            if (-not $Preflight) {
                throw "Expected Hello rollback apply denial to expose rollback transaction preflight"
            }
            if ($Preflight.schema -ne $HelloRollbackTransactionPreflightSchema -or $Preflight.id -ne $HelloRollbackTransactionPreflightId) {
                throw "Expected Hello rollback transaction preflight schema/id"
            }
            if ($Preflight.scope -ne "current_boot" -or $Preflight.classification -ne "local_only" -or $Preflight.persistence -ne "none" -or $Preflight.status -ne $HelloRollbackTransactionPreflightStatus) {
                throw "Expected Hello rollback transaction preflight to be current_boot/local_only/non-persistent"
            }
            if ($Preflight.requested_capability -ne "cap.service.rollback_apply.current_boot") {
                throw "Expected Hello rollback transaction preflight to bind requested rollback apply capability"
            }
            if (-not $Preflight.preflight_hash -or -not $Preflight.preflight_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback transaction preflight hash"
            }
            $BaseRollbackApplyHash = $Preflight.rollback_apply_hash
            if (-not $BaseRollbackApplyHash -or -not $BaseRollbackApplyHash.StartsWith("sha256:") -or $Preflight.rollback_preview_hash -ne $ExpectedPreviewHash -or $Preflight.source_probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback transaction preflight to bind apply, preview, and probation hashes"
            }
            if ($Apply.service_id -ne "svc.demo.hello" -or $Apply.source_probation.probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback apply denial to bind retained probation evidence"
            }
            if ($Apply.required_preview.schema -ne $HelloRollbackPreviewSchema -or $Apply.required_preview.id -ne $HelloRollbackPreviewId -or $Apply.required_preview.status -ne $HelloRollbackPreviewStatus -or $Apply.required_preview.preview_hash -ne $ExpectedPreviewHash) {
                throw "Expected Hello rollback apply denial to bind the current rollback preview hash"
            }
            if ($Apply.rollback_target.version -ne $ExpectedTargetVersion -or $Apply.rollback_target.generation -ne $ExpectedTargetGeneration) {
                throw "Expected Hello rollback apply rollback target version/generation"
            }
            if ($Apply.current_candidate.version -ne $ExpectedCurrentVersion -or $Apply.current_candidate.generation -ne $ExpectedCurrentGeneration -or $Apply.active_generation -ne $ExpectedCurrentGeneration) {
                throw "Expected Hello rollback apply current candidate version/generation"
            }
            if ($Apply.rollback_target.artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $Apply.current_candidate.artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash) {
                throw "Expected Hello rollback apply artifact identities"
            }
            if ($Apply.rollback_target.state_hash -ne $ExpectedStateHash -or $Apply.current_candidate.state_hash -ne $ExpectedStateHash -or $Apply.current_state.state_hash -ne $ExpectedStateHash) {
                throw "Expected Hello rollback apply denial to retain state hashes"
            }
            if ($Apply.rollback_target.state_counter -ne 3 -or $Apply.current_candidate.state_counter -ne 3 -or $Apply.current_state.state_counter -ne 3) {
                throw "Expected Hello rollback apply denial to retain state counter 3"
            }
            if ($Apply.state_migration.migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback apply denial to bind state migration hash"
            }
            if ($Preflight.rollback_target_artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $Preflight.current_candidate_artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash) {
                throw "Expected Hello rollback transaction preflight to bind target/current artifact identities"
            }
            if ($Preflight.rollback_target_generation -ne $ExpectedTargetGeneration -or $Preflight.current_candidate_generation -ne $ExpectedCurrentGeneration) {
                throw "Expected Hello rollback transaction preflight to bind target/current generations"
            }
            if ($Preflight.current_state_hash -ne $ExpectedStateHash -or $Preflight.rollback_target_state_hash -ne $ExpectedStateHash -or $Preflight.current_candidate_state_hash -ne $ExpectedStateHash) {
                throw "Expected Hello rollback transaction preflight to bind state hashes"
            }
            if ($Preflight.current_state_counter -ne 3 -or $Preflight.rollback_target_state_counter -ne 3 -or $Preflight.current_candidate_state_counter -ne 3) {
                throw "Expected Hello rollback transaction preflight to bind state counter 3"
            }
            if ($Preflight.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback transaction preflight to bind state migration hash"
            }
            if (-not $Preflight.missing_authorities.rollback_apply_authority -or -not $Preflight.missing_authorities.rollback_transaction_authority -or -not $Preflight.missing_authorities.durable_audit_write_authority -or -not $Preflight.missing_authorities.persistent_install_authority) {
                throw "Expected Hello rollback transaction preflight to expose missing write authorities"
            }
            if ($Preflight.side_effects.mutates_service_state -or $Preflight.side_effects.applies_rollback -or $Preflight.side_effects.writes_persistent_state -or $Preflight.side_effects.writes_durable_audit_log -or $Preflight.side_effects.writes_rollback_store -or $Preflight.side_effects.installs_rollback_plan) {
                throw "Expected Hello rollback transaction preflight to keep rollback/storage side effects disabled"
            }
            if ($Preflight.side_effects.accepts_external_artifact_bytes -or $Preflight.side_effects.loads_candidate_bytes -or $Preflight.side_effects.maps_executable_pages -or $Preflight.side_effects.provider_auto_load -or $Preflight.side_effects.grants_broad_mutation) {
                throw "Expected Hello rollback transaction preflight to keep external/load/mutation side effects disabled"
            }
            $WriteGate = $Apply.rollback_write_authority_gate
            if (-not $WriteGate) {
                throw "Expected Hello rollback apply denial to expose rollback write-authority gate"
            }
            if ($WriteGate.schema -ne $HelloRollbackWriteAuthorityGateSchema -or $WriteGate.id -ne $HelloRollbackWriteAuthorityGateId) {
                throw "Expected Hello rollback write-authority gate schema/id"
            }
            if ($WriteGate.scope -ne "current_boot" -or $WriteGate.classification -ne "local_only" -or $WriteGate.persistence -ne "none" -or $WriteGate.status -ne $HelloRollbackWriteAuthorityGateStatus) {
                throw "Expected Hello rollback write-authority gate to be current_boot/local_only/non-persistent"
            }
            if ($WriteGate.requested_capability -ne "cap.service.rollback_apply.current_boot") {
                throw "Expected Hello rollback write-authority gate to bind requested rollback apply capability"
            }
            if (-not $WriteGate.gate_hash -or -not $WriteGate.gate_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback write-authority gate hash"
            }
            if ($WriteGate.rollback_transaction_preflight_hash -ne $Preflight.preflight_hash -or $WriteGate.rollback_apply_hash -ne $BaseRollbackApplyHash -or $WriteGate.rollback_preview_hash -ne $ExpectedPreviewHash -or $WriteGate.source_probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback write-authority gate to bind preflight/apply/preview/probation hashes"
            }
            if ($WriteGate.required_schemas.audit_record -ne "raios.audit_record.v0" -or $WriteGate.required_schemas.rollback_transaction -ne "raios.rollback_transaction.v0") {
                throw "Expected Hello rollback write-authority gate to cite required durable schemas"
            }
            if (-not $WriteGate.unavailable_authorities.durable_audit_write_authority -or -not $WriteGate.unavailable_authorities.rollback_store_write_authority -or -not $WriteGate.unavailable_authorities.rollback_transaction_append) {
                throw "Expected Hello rollback write-authority gate to expose unavailable durable write authorities"
            }
            if ($WriteGate.side_effects.writes_durable_audit_log -or $WriteGate.side_effects.writes_rollback_store -or $WriteGate.side_effects.appends_rollback_transaction -or $WriteGate.side_effects.installs_rollback_plan -or $WriteGate.side_effects.applies_rollback) {
                throw "Expected Hello rollback write-authority gate to keep write/apply side effects disabled"
            }
            if ($WriteGate.current_state_hash -ne $ExpectedStateHash -or $WriteGate.current_state_counter -ne 3 -or $WriteGate.rollback_target_artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $WriteGate.current_candidate_artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash -or $WriteGate.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback write-authority gate to bind state, target/current artifact identities, and migration hash"
            }
            $AppendGate = $Apply.rollback_append_intent_gate
            if (-not $AppendGate) {
                throw "Expected Hello rollback apply denial to expose rollback append-intent gate"
            }
            if ($AppendGate.schema -ne $HelloRollbackAppendIntentGateSchema -or $AppendGate.id -ne $HelloRollbackAppendIntentGateId) {
                throw "Expected Hello rollback append-intent gate schema/id"
            }
            if ($AppendGate.scope -ne "current_boot" -or $AppendGate.classification -ne "local_only" -or $AppendGate.persistence -ne "none" -or $AppendGate.status -ne $HelloRollbackAppendIntentGateStatus) {
                throw "Expected Hello rollback append-intent gate to be current_boot/local_only/non-persistent"
            }
            if ($AppendGate.requested_capability -ne "cap.service.rollback_apply.current_boot") {
                throw "Expected Hello rollback append-intent gate to bind requested rollback apply capability"
            }
            if (-not $AppendGate.gate_hash -or -not $AppendGate.gate_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback append-intent gate hash"
            }
            if ($AppendGate.rollback_write_authority_gate_hash -ne $WriteGate.gate_hash -or $AppendGate.rollback_transaction_preflight_hash -ne $Preflight.preflight_hash -or $AppendGate.rollback_apply_hash -ne $BaseRollbackApplyHash -or $AppendGate.rollback_preview_hash -ne $ExpectedPreviewHash -or $AppendGate.source_probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback append-intent gate to bind write gate, preflight, apply, preview, and probation hashes"
            }
            if ($AppendGate.required_schemas.audit_record -ne "raios.audit_record.v0" -or $AppendGate.required_schemas.rollback_transaction -ne "raios.rollback_transaction.v0") {
                throw "Expected Hello rollback append-intent gate to cite required durable schemas"
            }
            if (-not $AppendGate.unavailable_authorities.append_intent -or -not $AppendGate.unavailable_authorities.rollback_transaction_append -or -not $AppendGate.unavailable_authorities.durable_audit_store -or -not $AppendGate.unavailable_authorities.rollback_store) {
                throw "Expected Hello rollback append-intent gate to expose unavailable append/store authorities"
            }
            if ($AppendGate.side_effects.writes_durable_audit_log -or $AppendGate.side_effects.writes_rollback_store -or $AppendGate.side_effects.appends_rollback_transaction -or $AppendGate.side_effects.installs_rollback_plan -or $AppendGate.side_effects.applies_rollback) {
                throw "Expected Hello rollback append-intent gate to keep append/write/apply side effects disabled"
            }
            if ($AppendGate.current_state_hash -ne $ExpectedStateHash -or $AppendGate.current_state_counter -ne 3 -or $AppendGate.rollback_target_descriptor_source_hash -ne $Preflight.rollback_target_descriptor_source_hash -or $AppendGate.current_candidate_descriptor_source_hash -ne $Preflight.current_candidate_descriptor_source_hash -or $AppendGate.rollback_target_artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $AppendGate.current_candidate_artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash -or $AppendGate.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback append-intent gate to bind state, descriptor sources, target/current artifact identities, and migration hash"
            }
            $PayloadGate = $Apply.rollback_payload_envelope_gate
            if (-not $PayloadGate) {
                throw "Expected Hello rollback apply denial to expose rollback payload-envelope gate"
            }
            if ($PayloadGate.schema -ne $HelloRollbackPayloadEnvelopeGateSchema -or $PayloadGate.id -ne $HelloRollbackPayloadEnvelopeGateId) {
                throw "Expected Hello rollback payload-envelope gate schema/id"
            }
            if ($PayloadGate.scope -ne "current_boot" -or $PayloadGate.classification -ne "local_only" -or $PayloadGate.persistence -ne "none" -or $PayloadGate.status -ne $HelloRollbackPayloadEnvelopeGateStatus) {
                throw "Expected Hello rollback payload-envelope gate to be current_boot/local_only/non-persistent"
            }
            if ($PayloadGate.requested_capability -ne "cap.service.rollback_apply.current_boot") {
                throw "Expected Hello rollback payload-envelope gate to bind requested rollback apply capability"
            }
            if (-not $PayloadGate.gate_hash -or -not $PayloadGate.gate_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback payload-envelope gate hash"
            }
            if ($PayloadGate.rollback_append_intent_gate_hash -ne $AppendGate.gate_hash -or $PayloadGate.rollback_write_authority_gate_hash -ne $WriteGate.gate_hash -or $PayloadGate.rollback_transaction_preflight_hash -ne $Preflight.preflight_hash -or $PayloadGate.rollback_apply_hash -ne $BaseRollbackApplyHash -or $PayloadGate.rollback_preview_hash -ne $ExpectedPreviewHash -or $PayloadGate.source_probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback payload-envelope gate to bind append gate, write gate, preflight, apply, preview, and probation hashes"
            }
            if ($PayloadGate.proposed_transaction.schema -ne $HelloRollbackTransactionPayloadSchema -or $PayloadGate.proposed_transaction.id -ne $HelloRollbackTransactionPayloadId -or $PayloadGate.proposed_transaction.status -ne $HelloRollbackTransactionPayloadStatus) {
                throw "Expected Hello rollback payload-envelope gate to expose proposed rollback transaction payload metadata"
            }
            if (-not $PayloadGate.proposed_transaction.payload_hash -or -not $PayloadGate.proposed_transaction.payload_hash.StartsWith("sha256:") -or -not $PayloadGate.proposed_transaction.provenance_hash -or -not $PayloadGate.proposed_transaction.provenance_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback payload-envelope gate to expose payload and provenance hashes"
            }
            if ($PayloadGate.proposed_transaction.appended_to_rollback_log) {
                throw "Expected Hello rollback payload-envelope gate to keep proposed payload unapplied"
            }
            if ($PayloadGate.required_schemas.audit_record -ne "raios.audit_record.v0" -or $PayloadGate.required_schemas.rollback_transaction -ne "raios.rollback_transaction.v0") {
                throw "Expected Hello rollback payload-envelope gate to cite required durable schemas"
            }
            if (-not $PayloadGate.unavailable_authorities.transaction_writer -or -not $PayloadGate.unavailable_authorities.rollback_transaction_append -or -not $PayloadGate.unavailable_authorities.durable_audit_store -or -not $PayloadGate.unavailable_authorities.rollback_store) {
                throw "Expected Hello rollback payload-envelope gate to expose unavailable writer/store authorities"
            }
            if ($PayloadGate.side_effects.writes_durable_audit_log -or $PayloadGate.side_effects.writes_rollback_store -or $PayloadGate.side_effects.appends_rollback_transaction -or $PayloadGate.side_effects.installs_rollback_plan -or $PayloadGate.side_effects.applies_rollback) {
                throw "Expected Hello rollback payload-envelope gate to keep append/write/apply side effects disabled"
            }
            if ($PayloadGate.current_state_hash -ne $ExpectedStateHash -or $PayloadGate.current_state_counter -ne 3 -or $PayloadGate.rollback_target_descriptor_source_hash -ne $Preflight.rollback_target_descriptor_source_hash -or $PayloadGate.current_candidate_descriptor_source_hash -ne $Preflight.current_candidate_descriptor_source_hash -or $PayloadGate.rollback_target_artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $PayloadGate.current_candidate_artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash -or $PayloadGate.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback payload-envelope gate to bind state, descriptor sources, target/current artifact identities, and migration hash"
            }
            $WriterStorageGate = $Apply.rollback_transaction_writer_storage_authority_gate
            if (-not $WriterStorageGate) {
                throw "Expected Hello rollback apply denial to expose rollback transaction writer/storage authority gate"
            }
            if ($WriterStorageGate.schema -ne $HelloRollbackTransactionWriterStorageAuthorityGateSchema -or $WriterStorageGate.id -ne $HelloRollbackTransactionWriterStorageAuthorityGateId) {
                throw "Expected Hello rollback writer/storage authority gate schema/id"
            }
            if ($WriterStorageGate.scope -ne "current_boot" -or $WriterStorageGate.classification -ne "local_only" -or $WriterStorageGate.persistence -ne "none" -or $WriterStorageGate.status -ne $HelloRollbackTransactionWriterStorageAuthorityGateStatus) {
                throw "Expected Hello rollback writer/storage authority gate to be current_boot/local_only/non-persistent"
            }
            if ($WriterStorageGate.requested_capability -ne "cap.service.rollback_apply.current_boot") {
                throw "Expected Hello rollback writer/storage authority gate to bind requested rollback apply capability"
            }
            if (-not $WriterStorageGate.gate_hash -or -not $WriterStorageGate.gate_hash.StartsWith("sha256:")) {
                throw "Expected Hello rollback writer/storage authority gate hash"
            }
            if ($WriterStorageGate.rollback_payload_envelope_gate_hash -ne $PayloadGate.gate_hash -or $WriterStorageGate.payload_hash -ne $PayloadGate.proposed_transaction.payload_hash -or $WriterStorageGate.provenance_hash -ne $PayloadGate.proposed_transaction.provenance_hash -or $WriterStorageGate.rollback_append_intent_gate_hash -ne $AppendGate.gate_hash -or $WriterStorageGate.rollback_write_authority_gate_hash -ne $WriteGate.gate_hash -or $WriterStorageGate.rollback_transaction_preflight_hash -ne $Preflight.preflight_hash -or $WriterStorageGate.rollback_apply_hash -ne $BaseRollbackApplyHash -or $WriterStorageGate.rollback_preview_hash -ne $ExpectedPreviewHash -or $WriterStorageGate.source_probation_hash -ne $ExpectedProbationHash) {
                throw "Expected Hello rollback writer/storage authority gate to bind payload envelope, payload, provenance, append gate, write gate, preflight, apply, preview, and probation hashes"
            }
            if ($WriterStorageGate.required_schemas.audit_record -ne "raios.audit_record.v0" -or $WriterStorageGate.required_schemas.rollback_transaction -ne "raios.rollback_transaction.v0") {
                throw "Expected Hello rollback writer/storage authority gate to cite required durable schemas"
            }
            $WriterStorageFoundation = $WriterStorageGate.writer_storage_foundation
            if (-not $WriterStorageFoundation -or $WriterStorageFoundation.schema -ne $HelloRollbackTransactionWriterStorageFoundationSchema -or $WriterStorageFoundation.owner_method -ne $HelloRollbackTransactionWriterStorageFoundationOwner -or $WriterStorageFoundation.recovery_visible_method -ne $HelloRollbackTransactionWriterStorageFoundationOwner) {
                throw "Expected Hello rollback writer/storage gate to cite the shared append-contract foundation"
            }
            if (-not $WriterStorageFoundation.storage_authority -or $WriterStorageFoundation.storage_authority.id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $WriterStorageFoundation.storage_authority.owner_method -ne $HelloRollbackTransactionWriterStorageAuthorityOwner -or $WriterStorageFoundation.storage_authority.status -ne $HelloRollbackTransactionWriterStorageLayoutStatus -or $WriterStorageFoundation.storage_authority.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason -or $WriterStorageFoundation.storage_authority.available -or $WriterStorageFoundation.storage_authority.authorizes_append) {
                throw "Expected Hello rollback writer/storage foundation to name the unavailable storage authority"
            }
            if (-not $WriterStorageFoundation.append_targets -or $WriterStorageFoundation.append_targets.audit_ledger.id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $WriterStorageFoundation.append_targets.audit_ledger.schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $WriterStorageFoundation.append_targets.audit_ledger.available -or $WriterStorageFoundation.append_targets.rollback_store.id -ne $HelloRollbackTransactionWriterStorageTargetId -or $WriterStorageFoundation.append_targets.rollback_store.schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $WriterStorageFoundation.append_targets.rollback_store.available -or $WriterStorageFoundation.append_targets.rollback_store.append_available) {
                throw "Expected Hello rollback writer/storage foundation to name unavailable audit and rollback append targets"
            }
            & $AssertHelloRollbackTargetRegionDiscovery -Name "Hello rollback writer/storage foundation" -TargetRegion $WriterStorageFoundation.target_region_discovery
            if (-not $WriterStorageFoundation.append_target_owner -or $WriterStorageFoundation.append_target_owner.schema -ne $HelloRollbackAppendTargetOwnerSchema -or $WriterStorageFoundation.append_target_owner.id -ne $HelloRollbackAppendTargetOwnerId -or $WriterStorageFoundation.append_target_owner.owner_method -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $WriterStorageFoundation.append_target_owner.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $WriterStorageFoundation.append_target_owner.status -ne $HelloRollbackAppendTargetOwnerStatus -or $WriterStorageFoundation.append_target_owner.reason -ne $HelloRollbackAppendTargetOwnerReason -or $WriterStorageFoundation.append_target_owner.available -or $WriterStorageFoundation.append_target_owner.block_write_path_available -or $WriterStorageFoundation.append_target_owner.authorizes_append) {
                throw "Expected Hello rollback writer/storage foundation to retain denied append target owner readiness"
            }
            if (-not $WriterStorageFoundation.transaction_writer_readiness -or $WriterStorageFoundation.transaction_writer_readiness.schema -ne $HelloRollbackTransactionWriterReadinessSchema -or $WriterStorageFoundation.transaction_writer_readiness.id -ne $HelloRollbackTransactionWriterReadinessId -or $WriterStorageFoundation.transaction_writer_readiness.owner_method -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $WriterStorageFoundation.transaction_writer_readiness.append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $WriterStorageFoundation.transaction_writer_readiness.status -ne $HelloRollbackTransactionWriterReadinessStatus -or $WriterStorageFoundation.transaction_writer_readiness.reason -ne $HelloRollbackTransactionWriterReadinessReason -or $WriterStorageFoundation.transaction_writer_readiness.ready -or $WriterStorageFoundation.transaction_writer_readiness.block_write_path_available -or $WriterStorageFoundation.transaction_writer_readiness.writes_durable_audit_log -or $WriterStorageFoundation.transaction_writer_readiness.writes_rollback_store -or $WriterStorageFoundation.transaction_writer_readiness.appends_rollback_transaction) {
                throw "Expected Hello rollback writer/storage foundation to retain denied transaction writer readiness"
            }
            & $AssertHelloRollbackScratchWriterDryRun -Name "Hello rollback writer/storage foundation" -DryRun $WriterStorageFoundation.transaction_writer_readiness.scratch_only_writer_dry_run -RequireAppendRecord $true
            if (-not $WriterStorageFoundation.block_write_path_authority_gate -or $WriterStorageFoundation.block_write_path_authority_gate.schema -ne $HelloRollbackBlockWritePathGateSchema -or $WriterStorageFoundation.block_write_path_authority_gate.id -ne $HelloRollbackBlockWritePathGateId -or $WriterStorageFoundation.block_write_path_authority_gate.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $WriterStorageFoundation.block_write_path_authority_gate.status -ne $HelloRollbackBlockWritePathGateStatus -or $WriterStorageFoundation.block_write_path_authority_gate.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason -or $WriterStorageFoundation.block_write_path_authority_gate.available -or $WriterStorageFoundation.block_write_path_authority_gate.read_only_block_driver_id -ne $HelloRollbackReadOnlyBlockDriverId -or -not $WriterStorageFoundation.block_write_path_authority_gate.read_only_block_driver_available -or -not $WriterStorageFoundation.block_write_path_authority_gate.partition_inventory_available -or $WriterStorageFoundation.block_write_path_authority_gate.partition_inventory_scheme -ne $HelloRollbackPartitionInventoryScheme -or $WriterStorageFoundation.block_write_path_authority_gate.authorizes_media_write -or $WriterStorageFoundation.block_write_path_authority_gate.authorizes_append -or $WriterStorageFoundation.block_write_path_authority_gate.writes_enabled -or $WriterStorageFoundation.block_write_path_authority_gate.write_attempted) {
                throw "Expected Hello rollback writer/storage foundation to retain the fail-closed block write-path gate"
            }
            if (-not $WriterStorageFoundation.block_write_path_authority_gate.scratch_block_write_authority_available -or $WriterStorageFoundation.block_write_path_authority_gate.scratch_block_write_authority_id -ne $HelloRollbackScratchBlockWriteAuthorityId -or -not $WriterStorageFoundation.block_write_path_authority_gate.scratch_region_within_device_bounds -or -not $WriterStorageFoundation.block_write_path_authority_gate.scratch_region_no_boot_or_partition_metadata_overlap) {
                throw "Expected Hello rollback writer/storage foundation to bind verified scratch-only block write authority facts without opening the rollback write path"
            }
            if ($WriterStorageFoundation.append_target.id -ne $HelloRollbackTransactionWriterStorageTargetId -or $WriterStorageFoundation.append_target.schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $WriterStorageFoundation.append_target.available) {
                throw "Expected Hello rollback writer/storage foundation to name the unavailable rollback transaction append target"
            }
            if ($WriterStorageFoundation.storage_layout.status -ne $HelloRollbackTransactionWriterStorageLayoutStatus -or $WriterStorageFoundation.storage_layout.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason -or $WriterStorageFoundation.storage_layout.available) {
                throw "Expected Hello rollback writer/storage foundation to bind the missing storage layout"
            }
            if ($WriterStorageFoundation.append_engine.status -ne $HelloRollbackTransactionWriterStorageAppendEngineStatus -or $WriterStorageFoundation.append_engine.reason -ne $HelloRollbackTransactionWriterStorageAppendEngineReason -or $WriterStorageFoundation.append_engine.available) {
                throw "Expected Hello rollback writer/storage foundation to bind the missing append engine"
            }
            if ($WriterStorageFoundation.append_contract.status -ne $HelloRollbackTransactionWriterStorageEnvelopeStatus -or $WriterStorageFoundation.append_contract.reason -ne $HelloRollbackTransactionWriterStorageEnvelopeReason -or $WriterStorageFoundation.append_contract.available) {
                throw "Expected Hello rollback writer/storage foundation to bind the missing rollback transaction envelope"
            }
            if ($WriterStorageFoundation.transaction_writer.owner -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $WriterStorageFoundation.transaction_writer.status -ne $HelloRollbackTransactionWriterStorageAppendContractStatus -or $WriterStorageFoundation.transaction_writer.reason -ne $HelloRollbackTransactionWriterStorageAppendContractReason -or $WriterStorageFoundation.transaction_writer.available) {
                throw "Expected Hello rollback writer/storage foundation to keep the transaction writer unavailable"
            }
            if (-not $WriterStorageGate.unavailable_authorities.transaction_writer -or -not $WriterStorageGate.unavailable_authorities.durable_audit_store -or -not $WriterStorageGate.unavailable_authorities.rollback_store -or -not $WriterStorageGate.unavailable_authorities.rollback_transaction_append) {
                throw "Expected Hello rollback writer/storage authority gate to expose unavailable writer/store/append authorities"
            }
            if ($WriterStorageGate.side_effects.writes_durable_audit_log -or $WriterStorageGate.side_effects.writes_rollback_store -or $WriterStorageGate.side_effects.appends_rollback_transaction -or $WriterStorageGate.side_effects.installs_rollback_plan -or $WriterStorageGate.side_effects.applies_rollback) {
                throw "Expected Hello rollback writer/storage authority gate to keep append/write/apply side effects disabled"
            }
            if ($WriterStorageGate.current_state_hash -ne $ExpectedStateHash -or $WriterStorageGate.current_state_counter -ne 3 -or $WriterStorageGate.rollback_target_descriptor_source_hash -ne $Preflight.rollback_target_descriptor_source_hash -or $WriterStorageGate.current_candidate_descriptor_source_hash -ne $Preflight.current_candidate_descriptor_source_hash -or $WriterStorageGate.rollback_target_artifact_identity_hash -ne $ExpectedTargetArtifactIdentityHash -or $WriterStorageGate.current_candidate_artifact_identity_hash -ne $ExpectedCurrentArtifactIdentityHash -or $WriterStorageGate.state_migration_hash -ne $ExpectedStateMigrationHash) {
                throw "Expected Hello rollback writer/storage authority gate to bind state, descriptor sources, target/current artifact identities, and migration hash"
            }
            if ($Apply.denied_surfaces.mutates_service_state -or $Apply.denied_surfaces.applies_rollback) {
                throw "Expected Hello rollback apply denial to avoid service mutation and rollback apply"
            }
            if ($Apply.denied_surfaces.descriptor_mutation -ne "not_attempted" -or $Apply.denied_surfaces.generation_mutation -ne "not_attempted" -or $Apply.denied_surfaces.running_state_mutation -ne "not_attempted" -or $Apply.denied_surfaces.ram_only_state_mutation -ne "not_attempted") {
                throw "Expected Hello rollback apply denial to avoid descriptor, generation, running-state, and RAM-only state mutation"
            }
            if ($Apply.denied_surfaces.persistent_install -ne "denied" -or $Apply.denied_surfaces.durable_audit_write -ne "denied" -or $Apply.denied_surfaces.external_artifact_load -ne "denied" -or $Apply.denied_surfaces.candidate_artifact_execution -ne "denied" -or $Apply.denied_surfaces.executable_mapping -ne "denied" -or $Apply.denied_surfaces.provider_auto_load -ne "denied" -or $Apply.denied_surfaces.broad_mutation -ne "denied") {
                throw "Expected Hello rollback apply denial to keep unsafe surfaces denied"
            }

            return $Apply.rollback_apply_hash
        }

        Send-AgentCommand -Command "agent module.audit_rollback_storage_layout" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout"
        $StorageLayout = Get-LastAgentResponseJson -Method "module.audit_rollback_storage_layout"
        $StorageLayoutResult = $StorageLayout.body.result
        if ($StorageLayoutResult.schema -ne "raios.module_audit_rollback_storage_layout.v0" -or $StorageLayoutResult.classification -ne "local_only" -or $StorageLayoutResult.writes_enabled) {
            throw "Expected storage-layout diagnostic to stay local-only and non-writing"
        }
        $StorageFoundation = $StorageLayoutResult.storage_authority_foundation
        if (-not $StorageFoundation -or $StorageFoundation.schema -ne "raios.audit_rollback_storage_authority.v0" -or $StorageFoundation.id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $StorageFoundation.owner_method -ne $HelloRollbackTransactionWriterStorageAuthorityOwner -or $StorageFoundation.available -or $StorageFoundation.authorizes_append -or $StorageFoundation.writes_enabled -or $StorageFoundation.write_attempted) {
            throw "Expected storage-layout diagnostic to expose the unavailable current-boot storage authority without authorizing append"
        }
        if ($StorageFoundation.append_targets.audit_ledger.id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $StorageFoundation.append_targets.audit_ledger.schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $StorageFoundation.append_targets.audit_ledger.available) {
            throw "Expected storage-layout diagnostic to expose unavailable audit append target"
        }
        if ($StorageFoundation.append_targets.rollback_store.id -ne $HelloRollbackTransactionWriterStorageTargetId -or $StorageFoundation.append_targets.rollback_store.schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $StorageFoundation.append_targets.rollback_store.available) {
            throw "Expected storage-layout diagnostic to expose unavailable rollback append target"
        }
        & $AssertHelloRollbackTargetRegionDiscovery -Name "storage-layout authority foundation" -TargetRegion $StorageFoundation.audit_rollback_target_region_discovery
        if ($StorageFoundation.required_authorities.transaction_writer_owner -ne $HelloRollbackTransactionWriterStorageWriterOwner -or -not $StorageFoundation.required_authorities.persistence_device_inventory -or -not $StorageFoundation.required_authorities.audit_rollback_storage_layout -or -not $StorageFoundation.required_authorities.append_engine) {
            throw "Expected storage-layout diagnostic to name required writer/storage authorities"
        }
        if ($StorageFoundation.observed_boot_storage.persistence_device_available -or $StorageFoundation.observed_boot_storage.storage_layout_available -or $StorageFoundation.observed_boot_storage.append_engine_available -or $StorageFoundation.observed_boot_storage.block_write_path_available) {
            throw "Expected storage-layout diagnostic to keep observed storage/write path unavailable"
        }
        if (-not $StorageFoundation.observed_boot_storage.storage_controller_observed -or -not $StorageFoundation.observed_boot_storage.ahci_register_probe_available -or -not $StorageFoundation.observed_boot_storage.block_device_identity_available -or -not $StorageFoundation.observed_boot_storage.sector_read_available -or -not $StorageFoundation.observed_boot_storage.partition_inventory_available -or -not $StorageFoundation.observed_boot_storage.block_driver_available -or -not $StorageFoundation.observed_boot_storage.scratch_write_path_available -or -not $StorageFoundation.observed_boot_storage.scratch_block_write_authority_available -or -not $StorageFoundation.observed_boot_storage.scratch_region_within_device_bounds -or -not $StorageFoundation.observed_boot_storage.scratch_region_no_boot_or_partition_metadata_overlap -or -not $StorageFoundation.observed_boot_storage.audit_rollback_target_region_available -or -not $StorageFoundation.observed_boot_storage.audit_rollback_target_region_within_device_bounds -or -not $StorageFoundation.observed_boot_storage.audit_rollback_target_region_no_boot_or_partition_metadata_overlap -or $StorageFoundation.observed_boot_storage.scratch_region_authorizes_audit_rollback -or $StorageFoundation.observed_boot_storage.block_write_path_reason -ne $HelloRollbackTransactionWriterStorageLayoutReason) {
            throw "Expected storage-layout diagnostic to expose read-only boot storage plus scratch write/readback evidence while keeping rollback writes unavailable"
        }
        if (-not $StorageFoundation.block_write_path_authority_gate -or $StorageFoundation.block_write_path_authority_gate.schema -ne $HelloRollbackBlockWritePathGateSchema -or $StorageFoundation.block_write_path_authority_gate.id -ne $HelloRollbackBlockWritePathGateId -or $StorageFoundation.block_write_path_authority_gate.status -ne $HelloRollbackBlockWritePathGateStatus -or $StorageFoundation.block_write_path_authority_gate.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason -or $StorageFoundation.block_write_path_authority_gate.available -or $StorageFoundation.block_write_path_authority_gate.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $StorageFoundation.block_write_path_authority_gate.read_only_block_driver_id -ne $HelloRollbackReadOnlyBlockDriverId -or -not $StorageFoundation.block_write_path_authority_gate.read_only_block_driver_available -or -not $StorageFoundation.block_write_path_authority_gate.read_only_block_driver_supports_read -or $StorageFoundation.block_write_path_authority_gate.read_only_block_driver_supports_write -or -not $StorageFoundation.block_write_path_authority_gate.partition_inventory_available -or $StorageFoundation.block_write_path_authority_gate.partition_inventory_scheme -ne $HelloRollbackPartitionInventoryScheme -or -not $StorageFoundation.block_write_path_authority_gate.scratch_write_path_available -or $StorageFoundation.block_write_path_authority_gate.scratch_region_id -ne $HelloRollbackScratchBlockRegionId -or $StorageFoundation.block_write_path_authority_gate.scratch_region_status -ne "available" -or $StorageFoundation.block_write_path_authority_gate.scratch_region_reason -ne "scratch_write_readback_verified" -or $StorageFoundation.block_write_path_authority_gate.scratch_region_authorizes_audit_rollback -or -not $StorageFoundation.block_write_path_authority_gate.requires_media_write_path -or $StorageFoundation.block_write_path_authority_gate.authorizes_media_write -or $StorageFoundation.block_write_path_authority_gate.authorizes_append -or $StorageFoundation.block_write_path_authority_gate.writes_enabled -or $StorageFoundation.block_write_path_authority_gate.write_attempted) {
            throw "Expected storage-layout diagnostic to bind the missing write path to read-only block-driver and partition evidence without authorizing writes"
        }
        if (-not $StorageFoundation.block_write_path_authority_gate.scratch_block_write_authority_available -or $StorageFoundation.block_write_path_authority_gate.scratch_block_write_authority_id -ne $HelloRollbackScratchBlockWriteAuthorityId -or -not $StorageFoundation.block_write_path_authority_gate.scratch_region_within_device_bounds -or -not $StorageFoundation.block_write_path_authority_gate.scratch_region_no_boot_or_partition_metadata_overlap) {
            throw "Expected storage-layout diagnostic to bind verified scratch-only block write authority facts"
        }
        $AhciProbe = $StorageLayoutResult.storage_layout_facts.persistence_device_inventory.ahci_controller_probe
        if (-not $AhciProbe -or $AhciProbe.schema -ne "raios.ahci_controller_probe.v0" -or $AhciProbe.source -ne $HelloRollbackAhciProbeSource -or -not $AhciProbe.observed -or -not $AhciProbe.controller_is_ahci -or -not $AhciProbe.abar_available -or -not $AhciProbe.registers_mapped -or -not $AhciProbe.first_port.implemented -or -not $AhciProbe.block_device_identity_available -or -not $AhciProbe.sector_read_available -or -not $AhciProbe.partition_inventory_available -or -not $AhciProbe.block_driver_available -or $AhciProbe.write_path_available -or -not $AhciProbe.identify_command_issued -or -not $AhciProbe.controller_register_write_attempted -or -not $AhciProbe.dma_started -or -not $AhciProbe.identify_completed -or $AhciProbe.identify_status -ne "identify_complete" -or -not $AhciProbe.sector_read_attempted -or -not $AhciProbe.sector_read_completed -or $AhciProbe.sector_read_status -ne "sector0_read_complete" -or -not $AhciProbe.write_attempted -or $AhciProbe.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason) {
            throw "Expected storage-layout diagnostic to expose completed AHCI boot reads plus scratch-only media write evidence without opening rollback writes"
        }
        if (-not $AhciProbe.block_device_identity -or -not $AhciProbe.block_device_identity.available -or $AhciProbe.block_device_identity.logical_sector_size_bytes -ne 512 -or $AhciProbe.block_device_identity.lba28_sector_count -le 0 -or [string]::IsNullOrWhiteSpace($AhciProbe.block_device_identity.model)) {
            throw "Expected storage-layout diagnostic to expose AHCI IDENTIFY block-device identity"
        }
        if (-not $AhciProbe.sector0_read -or -not $AhciProbe.sector0_read.available -or $AhciProbe.sector0_read.lba -ne 0 -or $AhciProbe.sector0_read.byte_count -ne 512 -or $AhciProbe.sector0_read.mbr_signature -ne 43605 -or -not $AhciProbe.sector0_read.mbr_signature_valid -or [string]::IsNullOrWhiteSpace($AhciProbe.sector0_read.first16_hex)) {
            throw "Expected storage-layout diagnostic to expose read-only AHCI sector-0 evidence"
        }
        if (-not $AhciProbe.partition_inventory -or -not $AhciProbe.partition_inventory.available -or $AhciProbe.partition_inventory.schema -ne "raios.boot_partition_inventory.v0" -or $AhciProbe.partition_inventory.source -ne "sector0_mbr_partition_table" -or $AhciProbe.partition_inventory.scheme -ne "mbr_empty" -or -not $AhciProbe.partition_inventory.mbr_signature_valid -or $AhciProbe.partition_inventory.entry_count -ne 0 -or $AhciProbe.partition_inventory.bootable_entry_count -ne 0 -or $AhciProbe.partition_inventory.entries.Count -ne 4) {
            throw "Expected storage-layout diagnostic to expose read-only empty-MBR partition inventory evidence"
        }
        $ScratchRegion = $AhciProbe.scratch_block_region
        if (-not $ScratchRegion -or $ScratchRegion.schema -ne $HelloRollbackScratchBlockRegionSchema -or $ScratchRegion.id -ne $HelloRollbackScratchBlockRegionId -or -not $ScratchRegion.test_infrastructure -or $ScratchRegion.persistence -ne "none" -or -not $ScratchRegion.available -or $ScratchRegion.status -ne "available" -or $ScratchRegion.reason -ne "scratch_write_readback_verified" -or $ScratchRegion.marker -ne "RAIOS_SCRATCH_V0" -or $ScratchRegion.marker_lba -ne 0 -or $ScratchRegion.test_lba -ne 1 -or $ScratchRegion.byte_count -ne 512 -or -not $ScratchRegion.label_found -or -not $ScratchRegion.write_attempted -or -not $ScratchRegion.write_completed -or -not $ScratchRegion.readback_completed -or -not $ScratchRegion.readback_matches -or [string]::IsNullOrWhiteSpace($ScratchRegion.readback_first16_hex) -or $ScratchRegion.authorizes_audit_rollback -or $ScratchRegion.authorizes_append -or -not $ScratchRegion.test_write_exercised -or $ScratchRegion.writes_enabled) {
            throw "Expected storage-layout diagnostic to prove the labeled VM scratch block region with a confined write/readback while denying append and rollback authority"
        }
        if (-not $ScratchRegion.device_identity_available -or $ScratchRegion.logical_sector_size_bytes -ne 512 -or ($ScratchRegion.lba28_sector_count -le 0 -and $ScratchRegion.lba48_sector_count -le 0) -or [string]::IsNullOrWhiteSpace($ScratchRegion.model) -or $ScratchRegion.region_start_lba -ne 1 -or $ScratchRegion.region_lba_count -ne 1 -or -not $ScratchRegion.region_within_device_bounds -or $ScratchRegion.boot_port_overlap -or $ScratchRegion.metadata_lba_overlap -or -not $ScratchRegion.no_boot_or_partition_metadata_overlap -or -not $ScratchRegion.block_write_authority_available) {
            throw "Expected scratch block region to bind device identity, explicit one-sector bounds, and no boot/metadata overlap facts"
        }
        $ScratchAuthority = $AhciProbe.scratch_block_write_authority
        if (-not $ScratchAuthority -or $ScratchAuthority.schema -ne $HelloRollbackScratchBlockWriteAuthoritySchema -or $ScratchAuthority.id -ne $HelloRollbackScratchBlockWriteAuthorityId -or -not $ScratchAuthority.test_infrastructure -or $ScratchAuthority.persistence -ne "none" -or $ScratchAuthority.source_region_id -ne $HelloRollbackScratchBlockRegionId -or $ScratchAuthority.owner -ne "vm_harness.scratch_region.current_boot" -or -not $ScratchAuthority.available -or $ScratchAuthority.status -ne "available" -or $ScratchAuthority.reason -ne "scratch_write_readback_verified" -or $ScratchAuthority.region_start_lba -ne 1 -or $ScratchAuthority.region_lba_count -ne 1 -or $ScratchAuthority.byte_count -ne 512 -or -not $ScratchAuthority.device_identity_available -or -not $ScratchAuthority.region_within_device_bounds -or -not $ScratchAuthority.no_boot_or_partition_metadata_overlap -or -not $ScratchAuthority.authorizes_scratch_write_readback -or $ScratchAuthority.authorizes_audit_rollback -or $ScratchAuthority.authorizes_append -or $ScratchAuthority.writes_enabled) {
            throw "Expected scratch block write authority to authorize only the current-boot scratch write/readback proof"
        }
        $AuditRollbackTargetRegion = $AhciProbe.audit_rollback_target_region
        if (-not $AuditRollbackTargetRegion -or $AuditRollbackTargetRegion.schema -ne $HelloRollbackTargetRegionDiscoverySchema -or $AuditRollbackTargetRegion.id -ne $HelloRollbackTargetRegionDiscoveryId -or -not $AuditRollbackTargetRegion.test_infrastructure -or $AuditRollbackTargetRegion.persistence -ne "none" -or -not $AuditRollbackTargetRegion.available -or $AuditRollbackTargetRegion.status -ne $HelloRollbackTargetRegionDiscoveryStatus -or $AuditRollbackTargetRegion.reason -ne $HelloRollbackTargetRegionDiscoveryReason -or $AuditRollbackTargetRegion.marker -ne "RAIOS_AUDITRB_V0" -or $AuditRollbackTargetRegion.marker_lba -ne 0 -or $AuditRollbackTargetRegion.region_start_lba -ne 1 -or $AuditRollbackTargetRegion.region_lba_count -ne 1 -or $AuditRollbackTargetRegion.byte_count -ne 512 -or -not $AuditRollbackTargetRegion.device_identity_available -or $AuditRollbackTargetRegion.logical_sector_size_bytes -ne 512 -or -not $AuditRollbackTargetRegion.label_found -or -not $AuditRollbackTargetRegion.read_attempted -or -not $AuditRollbackTargetRegion.read_completed -or [string]::IsNullOrWhiteSpace($AuditRollbackTargetRegion.read_first16_hex) -or -not $AuditRollbackTargetRegion.region_within_device_bounds -or $AuditRollbackTargetRegion.boot_port_overlap -or $AuditRollbackTargetRegion.metadata_lba_overlap -or $AuditRollbackTargetRegion.scratch_region_overlap -or -not $AuditRollbackTargetRegion.no_boot_or_partition_metadata_overlap -or -not $AuditRollbackTargetRegion.read_only -or $AuditRollbackTargetRegion.authorizes_audit_rollback -or $AuditRollbackTargetRegion.authorizes_append -or $AuditRollbackTargetRegion.writes_enabled -or $AuditRollbackTargetRegion.write_attempted) {
            throw "Expected AHCI probe to expose a read-only non-scratch audit/rollback target region without append or write authority"
        }
        if (-not $AhciProbe.block_driver -or $AhciProbe.block_driver.schema -ne "raios.read_only_block_driver.v0" -or $AhciProbe.block_driver.id -ne "block_driver.ahci.read_only.current_boot" -or $AhciProbe.block_driver.source -ne "ahci_dma_read_verified_sector0" -or -not $AhciProbe.block_driver.available -or $AhciProbe.block_driver.logical_sector_size_bytes -ne 512 -or $AhciProbe.block_driver.lba28_sector_count -le 0 -or $AhciProbe.block_driver.verified_read_lba -ne 0 -or $AhciProbe.block_driver.verified_read_byte_count -ne 512 -or -not $AhciProbe.block_driver.binds_partition_inventory -or -not $AhciProbe.block_driver.supports_read -or $AhciProbe.block_driver.supports_write -or $AhciProbe.block_driver.writes_enabled -or $AhciProbe.block_driver.write_attempted) {
            throw "Expected storage-layout diagnostic to expose read-only boot block-driver readiness while scratch writes stay outside rollback authority"
        }
        if ($StorageLayoutResult.policy_result.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $StorageLayoutResult.policy_result.audit_append_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $StorageLayoutResult.policy_result.rollback_append_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $StorageLayoutResult.policy_result.transaction_writer_owner -ne $HelloRollbackTransactionWriterStorageWriterOwner) {
            throw "Expected storage-layout policy result to retain stable storage authority and append target ids"
        }

        Send-AgentCommand -Command "agent module.audit_rollback_append_contract" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract"
        $AppendContract = Get-LastAgentResponseJson -Method "module.audit_rollback_append_contract"
        $AppendContractResult = $AppendContract.body.result
        if ($AppendContractResult.schema -ne "raios.module_audit_rollback_append_contract.v0" -or $AppendContractResult.classification -ne "local_only" -or $AppendContractResult.writes_enabled) {
            throw "Expected append-contract diagnostic to stay local-only and non-writing"
        }
        if (-not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.schema -ne $HelloRollbackBlockWritePathGateSchema -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.id -ne $HelloRollbackBlockWritePathGateId -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.reason -ne $HelloRollbackTransactionWriterStorageLayoutReason -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.available -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.read_only_block_driver_id -ne $HelloRollbackReadOnlyBlockDriverId -or -not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.read_only_block_driver_available -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.read_only_block_driver_supports_write -or -not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.partition_inventory_available -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.partition_inventory_scheme -ne $HelloRollbackPartitionInventoryScheme -or -not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_write_path_available -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_region_id -ne $HelloRollbackScratchBlockRegionId -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_region_status -ne "available" -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_region_authorizes_audit_rollback -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.authorizes_media_write -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.writes_enabled -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.write_attempted) {
            throw "Expected append-contract diagnostic to consume the fail-closed block write-path gate"
        }
        if (-not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_block_write_authority_available -or $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_block_write_authority_id -ne $HelloRollbackScratchBlockWriteAuthorityId -or -not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_region_within_device_bounds -or -not $AppendContractResult.storage_layout_inputs.block_write_path_authority_gate.scratch_region_no_boot_or_partition_metadata_overlap) {
            throw "Expected append-contract diagnostic to consume scratch-only write authority facts while keeping append unavailable"
        }
        $AppendTargetOwner = $AppendContractResult.append_target_owner
        if (-not $AppendTargetOwner -or $AppendTargetOwner.schema -ne $HelloRollbackAppendTargetOwnerSchema -or $AppendTargetOwner.id -ne $HelloRollbackAppendTargetOwnerId -or $AppendTargetOwner.owner_method -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $AppendTargetOwner.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $AppendTargetOwner.status -ne $HelloRollbackAppendTargetOwnerStatus -or $AppendTargetOwner.reason -ne $HelloRollbackAppendTargetOwnerReason -or $AppendTargetOwner.available -or $AppendTargetOwner.block_write_path_available -or $AppendTargetOwner.authorizes_append -or $AppendTargetOwner.writes_enabled) {
            throw "Expected append-contract diagnostic to expose unavailable append target owner over missing block write path"
        }
        if ($AppendTargetOwner.targets.audit_ledger.id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $AppendTargetOwner.targets.audit_ledger.schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $AppendTargetOwner.targets.audit_ledger.owner_bound -or $AppendTargetOwner.targets.audit_ledger.available) {
            throw "Expected append target owner to keep audit target unbound and unavailable"
        }
        if ($AppendTargetOwner.targets.rollback_store.id -ne $HelloRollbackTransactionWriterStorageTargetId -or $AppendTargetOwner.targets.rollback_store.schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $AppendTargetOwner.targets.rollback_store.owner_bound -or $AppendTargetOwner.targets.rollback_store.available) {
            throw "Expected append target owner to keep rollback target unbound and unavailable"
        }
        $TransactionWriterReadiness = $AppendContractResult.transaction_writer_readiness
        if (-not $TransactionWriterReadiness -or $TransactionWriterReadiness.schema -ne $HelloRollbackTransactionWriterReadinessSchema -or $TransactionWriterReadiness.id -ne $HelloRollbackTransactionWriterReadinessId -or $TransactionWriterReadiness.owner_method -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $TransactionWriterReadiness.append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $TransactionWriterReadiness.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $TransactionWriterReadiness.status -ne $HelloRollbackTransactionWriterReadinessStatus -or $TransactionWriterReadiness.reason -ne $HelloRollbackTransactionWriterReadinessReason -or $TransactionWriterReadiness.ready -or $TransactionWriterReadiness.block_write_path_available -or $TransactionWriterReadiness.writes_durable_audit_log -or $TransactionWriterReadiness.writes_rollback_store -or $TransactionWriterReadiness.appends_rollback_transaction) {
            throw "Expected transaction writer readiness to stay denied on the missing block write path"
        }
        & $AssertHelloRollbackScratchWriterDryRun -Name "append-contract transaction writer readiness" -DryRun $TransactionWriterReadiness.scratch_only_writer_dry_run
        $TargetRegionWriterContract = $TransactionWriterReadiness.target_region_writer_contract
        if (-not $TargetRegionWriterContract -or $TargetRegionWriterContract.schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $TargetRegionWriterContract.id -ne $HelloRollbackTargetRegionWriterContractId -or $TargetRegionWriterContract.status -ne $HelloRollbackTargetRegionWriterContractStatus -or $TargetRegionWriterContract.reason -ne $HelloRollbackTargetRegionWriterContractReason -or $TargetRegionWriterContract.source_discovery_schema -ne $HelloRollbackTargetRegionDiscoverySchema -or $TargetRegionWriterContract.source_discovery_id -ne $HelloRollbackTargetRegionDiscoveryId -or $TargetRegionWriterContract.source_discovery_status -ne $HelloRollbackTargetRegionDiscoveryStatus -or $TargetRegionWriterContract.source_discovery_reason -ne $HelloRollbackTargetRegionDiscoveryReason -or -not $TargetRegionWriterContract.target_region_present -or $TargetRegionWriterContract.target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegionWriterContract.target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TargetRegionWriterContract.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or $TargetRegionWriterContract.target_region_is_scratch -or $TargetRegionWriterContract.target_region_overlaps_boot_metadata -or $TargetRegionWriterContract.target_region_overlaps_scratch -or -not $TargetRegionWriterContract.target_range_ready -or $TargetRegionWriterContract.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $TargetRegionWriterContract.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $TargetRegionWriterContract.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $TargetRegionWriterContract.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $TargetRegionWriterContract.write_authority_available -or $TargetRegionWriterContract.durable_audit_policy_available -or $TargetRegionWriterContract.authorizes_append -or $TargetRegionWriterContract.writes_durable_audit_log -or $TargetRegionWriterContract.writes_rollback_store -or $TargetRegionWriterContract.appends_rollback_transaction -or $TargetRegionWriterContract.write_attempted) {
            throw "Expected append-contract diagnostic to bind the read-only non-scratch target region without granting writer authority"
        }
        $TargetRegionMediaWritePolicyPreflight = $TargetRegionWriterContract.media_write_policy_preflight
        if (-not $TargetRegionMediaWritePolicyPreflight -or $TargetRegionMediaWritePolicyPreflight.schema -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightSchema -or $TargetRegionMediaWritePolicyPreflight.id -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightId -or $TargetRegionMediaWritePolicyPreflight.status -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightStatus -or $TargetRegionMediaWritePolicyPreflight.reason -ne $HelloRollbackTargetRegionMediaWritePolicyPreflightReason -or $TargetRegionMediaWritePolicyPreflight.source_contract_schema -ne $HelloRollbackTargetRegionWriterContractSchema -or $TargetRegionMediaWritePolicyPreflight.source_contract_id -ne $HelloRollbackTargetRegionWriterContractId -or $TargetRegionMediaWritePolicyPreflight.source_contract_status -ne $HelloRollbackTargetRegionWriterContractStatus -or $TargetRegionMediaWritePolicyPreflight.source_contract_reason -ne $HelloRollbackTargetRegionWriterContractReason -or $TargetRegionMediaWritePolicyPreflight.owner_method -ne $HelloRollbackTransactionWriterStorageWriterOwner -or $TargetRegionMediaWritePolicyPreflight.append_target_owner_id -ne $HelloRollbackAppendTargetOwnerId -or $TargetRegionMediaWritePolicyPreflight.storage_authority_id -ne $HelloRollbackTransactionWriterStorageAuthorityId -or $TargetRegionMediaWritePolicyPreflight.audit_ledger_target_id -ne $HelloRollbackTransactionWriterStorageAuditTargetId -or $TargetRegionMediaWritePolicyPreflight.audit_record_schema -ne $HelloRollbackTransactionWriterStorageAuditTargetSchema -or $TargetRegionMediaWritePolicyPreflight.rollback_store_target_id -ne $HelloRollbackTransactionWriterStorageTargetId -or $TargetRegionMediaWritePolicyPreflight.rollback_transaction_schema -ne $HelloRollbackTransactionWriterStorageTargetSchema -or $TargetRegionMediaWritePolicyPreflight.target_region_start_lba -ne $HelloRollbackScratchRegionStartLba -or $TargetRegionMediaWritePolicyPreflight.target_region_lba_count -ne $HelloRollbackScratchRegionLbaCount -or $TargetRegionMediaWritePolicyPreflight.target_byte_count -ne $HelloRollbackScratchRegionByteCount -or -not $TargetRegionMediaWritePolicyPreflight.source_contract_target_range_ready -or -not $TargetRegionMediaWritePolicyPreflight.owner_ids_verified -or -not $TargetRegionMediaWritePolicyPreflight.target_ids_verified -or -not $TargetRegionMediaWritePolicyPreflight.target_span_verified -or -not $TargetRegionMediaWritePolicyPreflight.schema_ids_verified -or -not $TargetRegionMediaWritePolicyPreflight.media_write_authority_required -or $TargetRegionMediaWritePolicyPreflight.media_write_authority_available -or $TargetRegionMediaWritePolicyPreflight.media_write_authority_reason -ne "media_write_authority_missing" -or -not $TargetRegionMediaWritePolicyPreflight.durable_audit_policy_required -or $TargetRegionMediaWritePolicyPreflight.durable_audit_policy_available -or $TargetRegionMediaWritePolicyPreflight.durable_audit_policy_reason -ne "durable_audit_policy_missing" -or $TargetRegionMediaWritePolicyPreflight.authorizes_media_write -or $TargetRegionMediaWritePolicyPreflight.authorizes_append -or $TargetRegionMediaWritePolicyPreflight.writes_durable_audit_log -or $TargetRegionMediaWritePolicyPreflight.writes_rollback_store -or $TargetRegionMediaWritePolicyPreflight.appends_rollback_transaction -or $TargetRegionMediaWritePolicyPreflight.write_attempted) {
            throw "Expected target-region media-write/policy preflight to consume the writer contract and still deny all writes"
        }
        if ($AppendContractResult.policy_result.append_target_owner_status -ne $HelloRollbackAppendTargetOwnerStatus -or $AppendContractResult.policy_result.append_target_owner_reason -ne $HelloRollbackAppendTargetOwnerReason -or $AppendContractResult.policy_result.append_target_owner_available -or $AppendContractResult.policy_result.transaction_writer_status -ne $HelloRollbackTransactionWriterReadinessStatus -or $AppendContractResult.policy_result.transaction_writer_reason -ne $HelloRollbackTransactionWriterReadinessReason -or $AppendContractResult.policy_result.transaction_writer_ready -or $AppendContractResult.policy_result.block_write_path_available) {
            throw "Expected append-contract policy result to retain denied owner/readiness state"
        }

        $agentEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.describe requested_capability=cap.system.describe.read classification=local_only"
        Send-AgentCommand -Command $agentEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END system.describe"
        $agentEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        if ($agentEnvelope.body.result.schema -ne "raios.agent_command_envelope.v0") {
            throw "Expected agent command envelope schema"
        }
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_event_id" -Value $agentEnvelope.body.result.event_id
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_audit_event_id" -Value $agentEnvelope.body.result.audit_event_id
        if (-not $agentEnvelope.body.result.accepted -or $agentEnvelope.body.result.reason -ne "accepted" -or -not $agentEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected valid agent command envelope to dispatch the existing system.describe method"
        }
        if ($agentEnvelope.body.result.target_method -ne "system.describe" -or $agentEnvelope.body.result.requested_capability -ne "cap.system.describe.read") {
            throw "Expected agent command envelope to bind system.describe and its read capability"
        }
        if (
            $agentEnvelope.body.result.creates_parallel_dispatcher -or
            ($agentEnvelope.body.result.provider_write -ne "not_attempted") -or
            $agentEnvelope.body.result.loads_candidate_bytes -or
            $agentEnvelope.body.result.writes_persistent_state -or
            $agentEnvelope.body.result.writes_durable_audit_log -or
            $agentEnvelope.body.result.installs_rollback_plan -or
            $agentEnvelope.body.result.grants_broad_mutation
        ) {
            throw "Expected agent command envelope to avoid parallel dispatch, provider writes, candidate bytes, persistence, durable audit writes, rollback install, and broad mutation"
        }
        $envelopedDescribe = Get-LastAgentResponseJson -Method "system.describe"
        if ($envelopedDescribe.body.result.schema -ne "system.describe.v0") {
            throw "Expected accepted agent command envelope to route through system.describe"
        }

        $systemSnapshotEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.snapshot requested_capability=cap.system.snapshot.read classification=local_only"
        Send-AgentCommand -Command $systemSnapshotEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END system.snapshot"
        $systemSnapshotEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_system_snapshot_event_id" -Value $systemSnapshotEnvelope.body.result.event_id
        if (-not $systemSnapshotEnvelope.body.result.accepted -or $systemSnapshotEnvelope.body.result.reason -ne "accepted" -or -not $systemSnapshotEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected system.snapshot agent command envelope to dispatch"
        }
        if ($systemSnapshotEnvelope.body.result.target_method -ne "system.snapshot" -or $systemSnapshotEnvelope.body.result.requested_capability -ne "cap.system.snapshot.read") {
            throw "Expected agent command envelope to bind system.snapshot and its read capability"
        }
        $envelopedSystemSnapshot = Get-LastAgentResponseJson -Method "system.snapshot"
        if ($envelopedSystemSnapshot.body.result.schema -ne "system.snapshot.v0") {
            throw "Expected accepted agent command envelope to route through system.snapshot"
        }

        $bootLogEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.boot_log requested_capability=cap.system.boot_log.read classification=local_only"
        Send-AgentCommand -Command $bootLogEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END system.boot_log"
        $bootLogEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_boot_log_event_id" -Value $bootLogEnvelope.body.result.event_id
        if (-not $bootLogEnvelope.body.result.accepted -or $bootLogEnvelope.body.result.reason -ne "accepted" -or -not $bootLogEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected system.boot_log agent command envelope to dispatch"
        }
        if ($bootLogEnvelope.body.result.target_method -ne "system.boot_log" -or $bootLogEnvelope.body.result.requested_capability -ne "cap.system.boot_log.read") {
            throw "Expected agent command envelope to bind system.boot_log and its read capability"
        }
        $envelopedBootLog = Get-LastAgentResponseJson -Method "system.boot_log"
        if ($envelopedBootLog.body.result.schema -ne "system.boot_log.v0") {
            throw "Expected accepted agent command envelope to route through system.boot_log"
        }

        $systemCapabilitiesEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.capabilities requested_capability=cap.system.capabilities.read classification=local_only"
        Send-AgentCommand -Command $systemCapabilitiesEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END system.capabilities"
        $systemCapabilitiesEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_system_capabilities_event_id" -Value $systemCapabilitiesEnvelope.body.result.event_id
        if (-not $systemCapabilitiesEnvelope.body.result.accepted -or $systemCapabilitiesEnvelope.body.result.reason -ne "accepted" -or -not $systemCapabilitiesEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected system.capabilities agent command envelope to dispatch"
        }
        if ($systemCapabilitiesEnvelope.body.result.target_method -ne "system.capabilities" -or $systemCapabilitiesEnvelope.body.result.requested_capability -ne "cap.system.capabilities.read") {
            throw "Expected agent command envelope to bind system.capabilities and its read capability"
        }
        $envelopedSystemCapabilities = Get-LastAgentResponseJson -Method "system.capabilities"
        if ($envelopedSystemCapabilities.body.result.schema -ne "system.capabilities.v0") {
            throw "Expected accepted agent command envelope to route through system.capabilities"
        }

        $deviceGraphEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=device.graph requested_capability=cap.device.graph.read classification=local_only"
        Send-AgentCommand -Command $deviceGraphEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END device.graph"
        $deviceGraphEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_device_graph_event_id" -Value $deviceGraphEnvelope.body.result.event_id
        if (-not $deviceGraphEnvelope.body.result.accepted -or $deviceGraphEnvelope.body.result.reason -ne "accepted" -or -not $deviceGraphEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected device.graph agent command envelope to dispatch"
        }
        if ($deviceGraphEnvelope.body.result.target_method -ne "device.graph" -or $deviceGraphEnvelope.body.result.requested_capability -ne "cap.device.graph.read") {
            throw "Expected agent command envelope to bind device.graph and its read capability"
        }
        $envelopedDeviceGraph = Get-LastAgentResponseJson -Method "device.graph"
        if ($envelopedDeviceGraph.body.result.schema -ne "device.graph.v0") {
            throw "Expected accepted agent command envelope to route through device.graph"
        }

        $serviceInventoryEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.inventory requested_capability=cap.service.inventory.read classification=local_only"
        Send-AgentCommand -Command $serviceInventoryEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END service.inventory"
        $serviceInventoryEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_service_inventory_event_id" -Value $serviceInventoryEnvelope.body.result.event_id
        if (-not $serviceInventoryEnvelope.body.result.accepted -or $serviceInventoryEnvelope.body.result.reason -ne "accepted" -or -not $serviceInventoryEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected service.inventory agent command envelope to dispatch"
        }
        if ($serviceInventoryEnvelope.body.result.target_method -ne "service.inventory" -or $serviceInventoryEnvelope.body.result.requested_capability -ne "cap.service.inventory.read") {
            throw "Expected agent command envelope to bind service.inventory and its read capability"
        }
        $envelopedServiceInventory = Get-LastAgentResponseJson -Method "service.inventory"
        if ($envelopedServiceInventory.body.result.schema -ne "service.inventory.v0") {
            throw "Expected accepted agent command envelope to route through service.inventory"
        }

        $problemListEnvelopeCommand = "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=problem.list requested_capability=cap.problem.list.read classification=local_only"
        Send-AgentCommand -Command $problemListEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END problem.list"
        $problemListEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_problem_list_event_id" -Value $problemListEnvelope.body.result.event_id
        if (-not $problemListEnvelope.body.result.accepted -or $problemListEnvelope.body.result.reason -ne "accepted" -or -not $problemListEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected problem.list agent command envelope to dispatch"
        }
        if ($problemListEnvelope.body.result.target_method -ne "problem.list" -or $problemListEnvelope.body.result.requested_capability -ne "cap.problem.list.read") {
            throw "Expected agent command envelope to bind problem.list and its read capability"
        }
        $envelopedProblemList = Get-LastAgentResponseJson -Method "problem.list"
        if ($envelopedProblemList.body.result.schema -ne "problem.list.v0") {
            throw "Expected accepted agent command envelope to route through problem.list"
        }

        $mismatchEnvelopeOffset = Get-SerialLogOffset
        Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=service.inventory requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
        $mismatchEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_mismatch_event_id" -Value $mismatchEnvelope.body.result.event_id
        if ($mismatchEnvelope.body.result.accepted -or $mismatchEnvelope.body.result.code -ne "capability_denied" -or $mismatchEnvelope.body.result.reason -ne "requested_capability_denied" -or $mismatchEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected target/capability mismatch agent command envelope to be denied before dispatch"
        }
        if ($mismatchEnvelope.body.result.target_method -ne "service.inventory" -or $mismatchEnvelope.body.result.requested_capability -ne "cap.system.describe.read") {
            throw "Expected mismatch envelope to retain submitted target and capability"
        }
        $mismatchLog = Get-SerialLogContent -Path $SerialLog
        $mismatchAfter = if ($mismatchLog.Length -gt $mismatchEnvelopeOffset) { $mismatchLog.Substring([int]$mismatchEnvelopeOffset) } else { "" }
        $mismatchNoDispatch = -not $mismatchAfter.Contains("RAIOS_AGENT_END service.inventory")
        Add-Predicate -Name "quick:agent_command_envelope_mismatch_no_service_dispatch" -Expected "serial_not_contains_after_offset:RAIOS_AGENT_END service.inventory" -Passed $mismatchNoDispatch -Actual $(if ($mismatchNoDispatch) { "absent" } else { "found" })
        if (-not $mismatchNoDispatch) {
            throw "Expected target/capability mismatch to avoid service.inventory dispatch"
        }

        Send-AgentCommand -Command "agent command_envelope schema=bad target_method=system.describe requested_capability=cap.system.describe.read classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
        $badEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_bad_schema_event_id" -Value $badEnvelope.body.result.event_id
        if ($badEnvelope.body.result.accepted -or $badEnvelope.body.result.code -ne "invalid_envelope" -or $badEnvelope.body.result.reason -ne "schema_mismatch" -or $badEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected bad-schema agent command envelope to be denied before dispatch"
        }

        Send-AgentCommand -Command "agent command_envelope schema=raios.agent_command_envelope.v0 target_method=module.load_ephemeral requested_capability=cap.module.load_ephemeral classification=local_only" -ExpectedMarker "RAIOS_AGENT_END agent.command_envelope"
        $overCapEnvelope = Get-LastAgentResponseJson -Method "agent.command_envelope"
        Assert-CurrentBootEventId -Name "quick:agent_command_envelope_over_cap_event_id" -Value $overCapEnvelope.body.result.event_id
        if ($overCapEnvelope.body.result.accepted -or $overCapEnvelope.body.result.code -ne "capability_denied" -or $overCapEnvelope.body.result.reason -ne "target_method_not_allowed" -or $overCapEnvelope.body.result.dispatches_existing_agent_method) {
            throw "Expected over-capable agent command envelope to be denied before dispatch"
        }
        Assert-LogDoesNotContain -Name "quick:agent_command_envelope_denied_no_module_dispatch" -Needle "RAIOS_AGENT_END module.load_ephemeral"

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

        Send-AgentCommand -Command "recovery.rollback_inspect_source_reference_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect_source_reference_selftest"
        $inspectSourceReferenceSelftest = Get-LastAgentResponseJson -Method "recovery.rollback_inspect_source_reference_selftest"
        if ($inspectSourceReferenceSelftest.body.result.schema -ne $HelloRecoveryRollbackInspectSourceReferenceSelftestSchema) {
            throw "Expected recovery rollback inspect source-reference selftest schema"
        }
        if ($inspectSourceReferenceSelftest.body.result.id -ne $HelloRecoveryRollbackInspectSourceReferenceSelftestId) {
            throw "Expected stable recovery rollback inspect source-reference selftest id"
        }
        if (-not $inspectSourceReferenceSelftest.body.result.read_only -or $inspectSourceReferenceSelftest.body.result.mutates_global_event_log -or $inspectSourceReferenceSelftest.body.result.creates_source_reference_events -or $inspectSourceReferenceSelftest.body.result.persistence -ne "none") {
            throw "Recovery rollback inspect source-reference selftest must be read-only, RAM-only, and non-mutating"
        }
        if (-not $inspectSourceReferenceSelftest.body.result.diagnostic_hash -or -not $inspectSourceReferenceSelftest.body.result.diagnostic_hash.StartsWith("sha256:") -or $inspectSourceReferenceSelftest.body.result.validated_binding_schema -ne "raios.recovery_rollback_inspect_source_reference_binding.v0") {
            throw "Expected recovery rollback inspect source-reference selftest to cite diagnostic and binding schema evidence"
        }
        if ($inspectSourceReferenceSelftest.body.result.case_count -ne 7 -or $inspectSourceReferenceSelftest.body.result.passed_count -ne 7 -or -not $inspectSourceReferenceSelftest.body.result.all_passed) {
            throw "Expected all recovery rollback inspect source-reference selftest cases to pass"
        }
        $inspectSourceReferenceCaseNames = @($inspectSourceReferenceSelftest.body.result.cases | ForEach-Object { $_.name })
        foreach ($caseName in @("valid_source_reference", "missing_or_unretained_source_event_id", "wrong_source_read_binding", "missing_or_unretained_audit_event_id", "wrong_audit_event_variant", "substituted_hashes_denied", "authorizing_source_reference_denied")) {
            if ($inspectSourceReferenceCaseNames -notcontains $caseName) {
                throw "Missing recovery rollback inspect source-reference selftest case $caseName"
            }
        }
        $inspectSourceReferenceValidCase = @($inspectSourceReferenceSelftest.body.result.cases | Where-Object { $_.name -eq "valid_source_reference" })[0]
        if (-not $inspectSourceReferenceValidCase.validated -or $inspectSourceReferenceValidCase.actual_status -ne "retained_audit_event_verified" -or $inspectSourceReferenceValidCase.actual_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified") {
            throw "Expected valid recovery rollback inspect source-reference case to verify retained RAM audit evidence"
        }
        foreach ($case in @($inspectSourceReferenceSelftest.body.result.cases | Where-Object { $_.name -ne "valid_source_reference" })) {
            if ($case.actual_status -ne "rejected" -or $case.validated) {
                throw "Expected recovery rollback inspect source-reference negative case $($case.name) to fail closed"
            }
        }
        $inspectSourceReferenceReasons = @($inspectSourceReferenceSelftest.body.result.cases | ForEach-Object { $_.actual_reason })
        foreach ($reason in @("recovery_rollback_inspect_source_event_stale_or_dropped", "recovery_rollback_inspect_source_event_wrong_read_binding", "recovery_rollback_inspect_source_audit_event_stale_or_dropped", "recovery_rollback_inspect_source_audit_wrong_schema_or_variant", "recovery_rollback_inspect_source_audit_substituted_or_mismatched")) {
            if ($inspectSourceReferenceReasons -notcontains $reason) {
                throw "Missing recovery rollback inspect source-reference rejection reason $reason"
            }
        }
        $inspectSourceReferenceFailedCases = @($inspectSourceReferenceSelftest.body.result.cases | Where-Object { -not $_.passed })
        if ($inspectSourceReferenceFailedCases.Count -ne 0) {
            throw "Expected no recovery rollback inspect source-reference selftest failures"
        }
        if ($inspectSourceReferenceSelftest.body.result.denied_surfaces.rollback_apply -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.durable_media_write -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.durable_audit -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.rollback_store_write -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.transaction_append -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.persistence -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.external_artifact_bytes -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.candidate_execution -ne "denied" -or $inspectSourceReferenceSelftest.body.result.denied_surfaces.executable_mapping -ne "denied") {
            throw "Recovery rollback inspect source-reference selftest must keep rollback apply, durable writes, append, persistence, external bytes, execution, and mapping denied"
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
        $helloStateHash = & $AssertHelloState -Name "hello load response" -State $helloLoad.body.result.state -ExpectedCounter 1 -ExpectedVersion "v1"
        $helloServiceStateHash = & $AssertHelloState -Name "hello load service response" -State $helloLoad.body.result.service.state -ExpectedCounter 1 -ExpectedVersion "v1"
        if ($helloServiceStateHash -ne $helloStateHash) {
            throw "Expected hello load response and service object to agree on RAM-only state hash"
        }
        if ($null -ne $helloLoad.body.result.state_migration) {
            throw "Expected initial hello load to have no state migration record"
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
        $helloInventoryStateHash = & $AssertHelloState -Name "svc.demo.hello inventory record" -State $helloInventory[0].state -ExpectedCounter 1 -ExpectedVersion "v1"
        if ($helloInventoryStateHash -ne $helloStateHash) {
            throw "Expected svc.demo.hello inventory to expose the same RAM-only state hash after load"
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
        $helloHealthStateHash = & $AssertHelloState -Name "hello running health response" -State $helloHealthRunning.body.result.state -ExpectedCounter 1 -ExpectedVersion "v1"
        if ($helloHealthStateHash -ne $helloStateHash) {
            throw "Expected hello running health to expose the same RAM-only state hash"
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
        $helloStopStateHash = & $AssertHelloState -Name "hello stop response" -State $helloStop.body.result.state -ExpectedCounter 1 -ExpectedVersion "v1"
        if ($helloStopStateHash -ne $helloStateHash) {
            throw "Expected service.stop to preserve Hello RAM-only state"
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
        $helloHealthStoppedStateHash = & $AssertHelloState -Name "hello stopped health response" -State $helloHealthStopped.body.result.state -ExpectedCounter 1 -ExpectedVersion "v1"
        if ($helloHealthStoppedStateHash -ne $helloStateHash) {
            throw "Expected stopped hello health to preserve Hello RAM-only state"
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

        Send-AgentCommand -Command "service.start svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.start"
        $helloStart = Get-LastAgentResponseJson -Method "service.start"
        Assert-CurrentBootEventId -Name "quick:hello_start_event_id" -Value $helloStart.body.result.event_id
        if (-not $helloStart.body.result.service.loaded -or -not $helloStart.body.result.service.running) {
            throw "Expected service.start to restart the loaded hello service"
        }
        if ($helloStart.body.result.lifecycle.start_event_id -ne $helloStart.body.result.event_id) {
            throw "Expected service.start to record a distinct start event id"
        }
        if ($helloStart.body.result.lifecycle.last_action -ne "start" -or $helloStart.body.result.lifecycle.reason -ne "started_loaded_service") {
            throw "Expected service.start lifecycle to mark the stopped loaded service as started"
        }
        $helloStartStateHash = & $AssertHelloState -Name "hello start response" -State $helloStart.body.result.state -ExpectedCounter 2 -ExpectedVersion "v1"
        $helloStartServiceStateHash = & $AssertHelloState -Name "hello start service response" -State $helloStart.body.result.service.state -ExpectedCounter 2 -ExpectedVersion "v1"
        if ($helloStartServiceStateHash -ne $helloStartStateHash -or $helloStartStateHash -eq $helloStateHash) {
            throw "Expected service.start to advance the tiny Hello RAM-only state value"
        }
        $helloStartActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello start response" `
            -Activation $helloStart.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($helloStartActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello start response to cite the same service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello start service response" -Record $helloStart.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello start loader response" -Record $helloStart.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.restart svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.restart"
        $helloRestart = Get-LastAgentResponseJson -Method "service.restart"
        Assert-CurrentBootEventId -Name "quick:hello_restart_event_id" -Value $helloRestart.body.result.event_id
        if (-not $helloRestart.body.result.service.loaded -or -not $helloRestart.body.result.service.running) {
            throw "Expected service.restart to leave the loaded hello service running"
        }
        if ($helloRestart.body.result.lifecycle.last_action -ne "restart" -or $helloRestart.body.result.lifecycle.reason -ne "restarted_loaded_service") {
            throw "Expected service.restart lifecycle to record a real restart action"
        }
        if ($helloRestart.body.result.lifecycle.start_event_id -ne $helloRestart.body.result.event_id) {
            throw "Expected service.restart to record its own lifecycle event as the latest start event"
        }
        if ($helloRestart.body.result.service.generation -ne $helloStart.body.result.service.generation) {
            throw "Expected service.restart to preserve the loaded hello generation"
        }
        $helloRestartStateHash = & $AssertHelloState -Name "hello restart response" -State $helloRestart.body.result.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($helloRestartStateHash -eq $helloStartStateHash) {
            throw "Expected service.restart to advance the tiny Hello RAM-only state value"
        }
        $helloRestartActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello restart response" `
            -Activation $helloRestart.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($helloRestartActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello restart response to cite the same service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello restart service response" -Record $helloRestart.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello restart loader response" -Record $helloRestart.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.hot_swap svc.demo.hello.reset_state" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
        $resetHelloHotSwap = Get-LastAgentResponseJson -Method "service.hot_swap"
        Assert-CurrentBootEventId -Name "quick:hello_hot_swap_reset_denied_event_id" -Value $resetHelloHotSwap.body.event_id
        if ($resetHelloHotSwap.t -ne "error" -or $resetHelloHotSwap.body.code -ne "capability_denied" -or $resetHelloHotSwap.body.reason -ne "state_migration_would_reset_state") {
            throw "Expected reset-state service.hot_swap target to be denied by the state migration gate"
        }
        $resetHelloStateHash = & $AssertHelloState -Name "hello reset hot-swap denied response" -State $resetHelloHotSwap.body.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($resetHelloStateHash -ne $helloRestartStateHash) {
            throw "Expected denied reset-state service.hot_swap response to cite the active Hello state"
        }
        if (-not $resetHelloHotSwap.body.state_migration -or $resetHelloHotSwap.body.state_migration.schema -ne $HelloStateMigrationSchema) {
            throw "Expected denied reset-state service.hot_swap to expose migration evidence"
        }
        if ($resetHelloHotSwap.body.state_migration.pre_state_counter -ne 3 -or $resetHelloHotSwap.body.state_migration.post_state_counter -ne 0) {
            throw "Expected denied reset-state migration to show the candidate would reset the counter"
        }
        if ($resetHelloHotSwap.body.state_migration.pre_state_hash -ne $helloRestartStateHash -or $resetHelloHotSwap.body.state_migration.post_state_hash -eq $helloRestartStateHash) {
            throw "Expected denied reset-state migration to preserve the pre-state hash and reject the reset post-state"
        }
        if ($resetHelloHotSwap.body.state_migration.state_preserved -or $resetHelloHotSwap.body.state_migration.accepted) {
            throw "Expected denied reset-state migration to be rejected and not state-preserving"
        }
        $resetHelloMigrationHash = $resetHelloHotSwap.body.state_migration.migration_hash

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthAfterResetHotSwap = Get-LastAgentResponseJson -Method "service.health"
        if ($helloHealthAfterResetHotSwap.body.result.service.generation -ne $helloRestart.body.result.service.generation -or -not $helloHealthAfterResetHotSwap.body.result.service.running) {
            throw "Expected denied reset-state service.hot_swap to preserve the running hello generation"
        }
        if ($helloHealthAfterResetHotSwap.body.result.load_descriptor.source.sha256 -ne $helloDescriptorHash) {
            throw "Expected denied reset-state service.hot_swap to preserve the current-image descriptor"
        }
        $helloAfterResetHotSwapStateHash = & $AssertHelloState -Name "hello health after denied reset hot-swap" -State $helloHealthAfterResetHotSwap.body.result.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($helloAfterResetHotSwapStateHash -ne $helloRestartStateHash) {
            throw "Expected denied reset-state service.hot_swap to preserve Hello RAM-only state"
        }

        Send-AgentCommand -Command "service.hot_swap external:svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
        $badHelloHotSwap = Get-LastAgentResponseJson -Method "service.hot_swap"
        Assert-CurrentBootEventId -Name "quick:hello_hot_swap_external_denied_event_id" -Value $badHelloHotSwap.body.event_id
        if ($badHelloHotSwap.t -ne "error" -or $badHelloHotSwap.body.code -ne "capability_denied") {
            throw "Expected external service.hot_swap target to be denied before service mutation"
        }

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthAfterBadHotSwap = Get-LastAgentResponseJson -Method "service.health"
        if ($helloHealthAfterBadHotSwap.body.result.service.generation -ne $helloRestart.body.result.service.generation -or -not $helloHealthAfterBadHotSwap.body.result.service.running) {
            throw "Expected denied service.hot_swap to preserve the running hello generation"
        }
        if ($helloHealthAfterBadHotSwap.body.result.load_descriptor.source.sha256 -ne $helloDescriptorHash) {
            throw "Expected denied service.hot_swap to preserve the current-image descriptor"
        }
        $helloAfterBadHotSwapStateHash = & $AssertHelloState -Name "hello health after denied hot-swap" -State $helloHealthAfterBadHotSwap.body.result.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($helloAfterBadHotSwapStateHash -ne $helloRestartStateHash) {
            throw "Expected denied service.hot_swap to preserve Hello RAM-only state"
        }

        Send-AgentCommand -Command "service.hot_swap svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
        $helloHotSwap = Get-LastAgentResponseJson -Method "service.hot_swap"
        Assert-CurrentBootEventId -Name "quick:hello_hot_swap_event_id" -Value $helloHotSwap.body.result.event_id
        if (-not $helloHotSwap.body.result.service.loaded -or -not $helloHotSwap.body.result.service.running) {
            throw "Expected service.hot_swap to leave the hello service loaded and running"
        }
        if ($helloHotSwap.body.result.lifecycle.last_action -ne "hot_swap" -or $helloHotSwap.body.result.lifecycle.reason -ne "hot_swapped_builtin_service") {
            throw "Expected service.hot_swap lifecycle to record a real hot-swap action"
        }
        if ($helloHotSwap.body.result.lifecycle.hot_swap_event_id -ne $helloHotSwap.body.result.event_id -or $helloHotSwap.body.result.lifecycle.load_event_id -ne $helloHotSwap.body.result.event_id -or $helloHotSwap.body.result.lifecycle.start_event_id -ne $helloHotSwap.body.result.event_id) {
            throw "Expected service.hot_swap to bind its event id as hot-swap/load/start evidence"
        }
        if ($helloHotSwap.body.result.service.generation -ne ($helloRestart.body.result.service.generation + 1)) {
            throw "Expected accepted service.hot_swap to advance the loaded hello generation"
        }
        if ($helloHotSwap.body.result.load_descriptor.source.sha256 -ne $helloDescriptorHash -or $helloHotSwap.body.result.load_descriptor.artifact_identity.sha256 -ne $helloArtifactIdentityHash) {
            throw "Expected service.hot_swap to cite the accepted built-in descriptor and artifact identity"
        }
        $helloHotSwapStateHash = & $AssertHelloState -Name "hello hot-swap response" -State $helloHotSwap.body.result.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($helloHotSwapStateHash -ne $helloRestartStateHash) {
            throw "Expected accepted v1 service.hot_swap to preserve Hello RAM-only state"
        }
        $helloHotSwapMigrationHash = & $AssertHelloStateMigration -Name "hello v1 hot-swap response" -Migration $helloHotSwap.body.result.state_migration -ExpectedFromVersion "v1" -ExpectedToVersion "v1" -ExpectedCounter 3 -ExpectedStateHash $helloRestartStateHash
        $helloHotSwapProbationHash = & $AssertHelloHotSwapProbation `
            -Name "hello v1 hot-swap response" `
            -Probation $helloHotSwap.body.result.hot_swap_probation `
            -ExpectedPreviousVersion "v1" `
            -ExpectedNewVersion "v1" `
            -ExpectedPreviousGeneration $helloRestart.body.result.service.generation `
            -ExpectedNewGeneration $helloHotSwap.body.result.service.generation `
            -ExpectedPreviousStateHash $helloRestartStateHash `
            -ExpectedNewStateHash $helloHotSwapStateHash `
            -ExpectedPreviousArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedNewArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedStateMigrationHash $helloHotSwapMigrationHash
        $helloHotSwapActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello hot-swap response" `
            -Activation $helloHotSwap.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloLoadPlanPreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        if ($helloHotSwapActivationHash -ne $helloServiceSlotActivationHash) {
            throw "Expected hello hot-swap response to cite the same service-slot activation hash"
        }
        & $AssertHelloServiceSlotActivationReference -Name "hello hot-swap service response" -Record $helloHotSwap.body.result.service -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello hot-swap loader response" -Record $helloHotSwap.body.result.loader -ExpectedHash $helloServiceSlotActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.hot_swap svc.demo.hello.v2" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
        $helloHotSwapV2 = Get-LastAgentResponseJson -Method "service.hot_swap"
        Assert-CurrentBootEventId -Name "quick:hello_hot_swap_v2_event_id" -Value $helloHotSwapV2.body.result.event_id
        if ($helloHotSwapV2.body.result.service.version -ne "v2" -or $helloHotSwapV2.body.result.service.artifact_identity_id -ne $HelloArtifactIdentityV2Id) {
            throw "Expected service.hot_swap svc.demo.hello.v2 to select the signed v2 built-in candidate"
        }
        if ($helloHotSwapV2.body.result.service.artifact_identity_hash -eq $helloArtifactIdentityHash) {
            throw "Expected signed v2 candidate to have a distinct artifact identity hash"
        }
        if ($helloHotSwapV2.body.result.service.generation -ne ($helloHotSwap.body.result.service.generation + 1)) {
            throw "Expected accepted v2 service.hot_swap to advance the loaded hello generation"
        }
        $helloHotSwapV2StateHash = & $AssertHelloState -Name "hello v2 hot-swap response" -State $helloHotSwapV2.body.result.state -ExpectedCounter 3 -ExpectedVersion "v2"
        if ($helloHotSwapV2StateHash -ne $helloHotSwapStateHash) {
            throw "Expected v2 service.hot_swap to preserve Hello RAM-only state"
        }
        $helloHotSwapV2MigrationHash = & $AssertHelloStateMigration -Name "hello v2 hot-swap response" -Migration $helloHotSwapV2.body.result.state_migration -ExpectedFromVersion "v1" -ExpectedToVersion "v2" -ExpectedCounter 3 -ExpectedStateHash $helloHotSwapStateHash
        $helloHotSwapV2ProbationHash = & $AssertHelloHotSwapProbation `
            -Name "hello v2 hot-swap response" `
            -Probation $helloHotSwapV2.body.result.hot_swap_probation `
            -ExpectedPreviousVersion "v1" `
            -ExpectedNewVersion "v2" `
            -ExpectedPreviousGeneration $helloHotSwap.body.result.service.generation `
            -ExpectedNewGeneration $helloHotSwapV2.body.result.service.generation `
            -ExpectedPreviousStateHash $helloHotSwapStateHash `
            -ExpectedNewStateHash $helloHotSwapV2StateHash `
            -ExpectedPreviousArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedNewArtifactIdentityHash $helloHotSwapV2.body.result.service.artifact_identity_hash `
            -ExpectedStateMigrationHash $helloHotSwapV2MigrationHash
        $helloHotSwapV2PreflightHash = & $AssertHelloLoadPlanPreflight `
            -Name "hello v2 hot-swap response" `
            -Preflight $helloHotSwapV2.body.result.artifact_load_plan_preflight `
            -DescriptorSourceLocator $helloDescriptorLocator `
            -DescriptorSourceHash $helloDescriptorHash `
            -ArtifactIdentityHash $helloHotSwapV2.body.result.service.artifact_identity_hash `
            -ArtifactContentHash $helloArtifactContentHash `
            -ArtifactReferenceHash $helloArtifactReferenceHash `
            -ArtifactBytesHash $helloArtifactBytesHash `
            -ExpectedArtifactIdentityId $HelloArtifactIdentityV2Id
        $helloHotSwapV2ActivationHash = & $AssertHelloServiceSlotActivation `
            -Name "hello v2 hot-swap response" `
            -Activation $helloHotSwapV2.body.result.service_slot_activation `
            -DescriptorSourceHash $helloDescriptorHash `
            -PreflightHash $helloHotSwapV2PreflightHash `
            -ExpectedStatus $HelloServiceSlotActivationActiveStatus `
            -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello v2 hot-swap service response" -Record $helloHotSwapV2.body.result.service -ExpectedHash $helloHotSwapV2ActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true
        & $AssertHelloServiceSlotActivationReference -Name "hello v2 hot-swap loader response" -Record $helloHotSwapV2.body.result.loader -ExpectedHash $helloHotSwapV2ActivationHash -ExpectedStatus $HelloServiceSlotActivationActiveStatus -ExpectedActive $true

        Send-AgentCommand -Command "service.rollback_preview svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview"
        $helloRollbackPreview = Get-LastAgentResponseJson -Method "service.rollback_preview"
        Assert-CurrentBootEventId -Name "quick:hello_rollback_preview_event_id" -Value $helloRollbackPreview.body.result.event_id
        & $AssertHelloRollbackPreview `
            -Preview $helloRollbackPreview.body.result `
            -ExpectedProbationHash $helloHotSwapV2ProbationHash `
            -ExpectedTargetVersion "v1" `
            -ExpectedCurrentVersion "v2" `
            -ExpectedTargetGeneration $helloHotSwap.body.result.service.generation `
            -ExpectedCurrentGeneration $helloHotSwapV2.body.result.service.generation `
            -ExpectedTargetArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedCurrentArtifactIdentityHash $helloHotSwapV2.body.result.service.artifact_identity_hash `
            -ExpectedStateHash $helloHotSwapStateHash `
            -ExpectedStateMigrationHash $helloHotSwapV2MigrationHash

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthAfterRollbackPreview = Get-LastAgentResponseJson -Method "service.health"
        if ($helloHealthAfterRollbackPreview.body.result.service.version -ne "v2" -or $helloHealthAfterRollbackPreview.body.result.service.generation -ne $helloHotSwapV2.body.result.service.generation) {
            throw "Expected rollback preview to leave the active v2 service unchanged"
        }
        $helloHealthAfterRollbackPreviewStateHash = & $AssertHelloState -Name "hello health after rollback preview" -State $helloHealthAfterRollbackPreview.body.result.state -ExpectedCounter 3 -ExpectedVersion "v2"
        if ($helloHealthAfterRollbackPreviewStateHash -ne $helloHotSwapV2StateHash) {
            throw "Expected rollback preview to preserve Hello RAM-only state"
        }

        Send-AgentCommand -Command "recovery.rollback_materialize_dry_run svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_materialize_dry_run"
        $helloRollbackMaterialize = Get-LastAgentResponseJson -Method "recovery.rollback_materialize_dry_run"
        if ($helloRollbackMaterialize.body.result.schema -ne "raios.recovery_rollback_materialize_dry_run.v0" -or $helloRollbackMaterialize.body.result.status -ne "target_region_sector_materialized_current_boot" -or -not $helloRollbackMaterialize.body.result.materialized_sector_evidence_available -or $helloRollbackMaterialize.body.result.denied_surfaces.appends_rollback_transaction -or $helloRollbackMaterialize.body.result.denied_surfaces.applies_rollback) {
            throw "Expected recovery rollback materialize dry-run to prepare retained target-region evidence without append/apply authority"
        }

        Send-AgentCommand -Command "recovery.rollback_inspect svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_inspect"
        $helloRollbackInspect = Get-LastAgentResponseJson -Method "recovery.rollback_inspect"
        if ($helloRollbackInspect.body.result.schema -ne "raios.recovery_rollback_inspect.v0" -or $helloRollbackInspect.body.result.status -ne "target_region_sector_inspected_current_boot" -or -not $helloRollbackInspect.body.result.materialized_sector_evidence_available -or -not $helloRollbackInspect.body.result.inspection_available -or $helloRollbackInspect.body.result.denied_surfaces.appends_rollback_transaction -or $helloRollbackInspect.body.result.denied_surfaces.applies_rollback) {
            throw "Expected recovery rollback inspect to retain target-region sector inspection evidence without append/apply authority"
        }
        $helloRollbackInspectSector = $helloRollbackInspect.body.result.target_region_sector_inspection
        $helloRollbackInspectSource = $helloRollbackInspect.body.result.retained_recovery_rollback_inspect_source
        if (-not $helloRollbackInspectSource -or $helloRollbackInspectSource.schema -ne $HelloRecoveryRollbackInspectSourceReferenceSchema -or $helloRollbackInspectSource.id -ne $HelloRecoveryRollbackInspectSourceReferenceId -or $helloRollbackInspectSource.status -ne $HelloRecoveryRollbackInspectSourceReferenceStatus -or $helloRollbackInspectSource.reason -ne $HelloRecoveryRollbackInspectSourceReferenceReason -or $helloRollbackInspectSource.source_method -ne "recovery.rollback_inspect" -or $helloRollbackInspectSource.source_event_id -eq $null -or $helloRollbackInspectSource.source_audit_event_id -eq $null -or -not $helloRollbackInspectSource.source_available -or -not $helloRollbackInspectSource.source_matches_sector_inspection -or -not $helloRollbackInspectSource.source_event_retained -or -not $helloRollbackInspectSource.source_audit_event_retained -or $helloRollbackInspectSource.ram_audit_status -ne "retained_audit_event_verified" -or $helloRollbackInspectSource.ram_audit_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified" -or -not $helloRollbackInspectSource.ram_audit_validated -or -not $helloRollbackInspectSource.reference_hash.StartsWith("sha256:") -or $helloRollbackInspectSource.source_inspection_hash -ne $helloRollbackInspectSector.inspection_hash -or $helloRollbackInspectSource.target_region_sector_inspection_hash -ne $helloRollbackInspectSector.inspection_hash -or $helloRollbackInspectSource.source_sector_plan_hash -ne $helloRollbackInspectSector.source_sector_plan_hash -or $helloRollbackInspectSource.source_target_region_write_readback_hash -ne $helloRollbackInspectSector.source_target_region_write_readback_hash -or $helloRollbackInspectSource.authorizes_rollback_apply) {
            throw "Expected recovery rollback inspect to retain a source reference over its verified sector inspection without applying rollback"
        }

        Send-AgentCommand -Command "service.rollback_apply svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply"
        $helloRollbackApply = Get-LastAgentResponseJson -Method "service.rollback_apply"
        Assert-CurrentBootEventId -Name "quick:hello_rollback_apply_event_id" -Value $helloRollbackApply.body.event_id
        $helloRollbackApplyHash = & $AssertHelloRollbackApplyDenied `
            -Apply $helloRollbackApply.body `
            -ExpectedProbationHash $helloHotSwapV2ProbationHash `
            -ExpectedPreviewHash $helloRollbackPreview.body.result.preview_hash `
            -ExpectedTargetVersion "v1" `
            -ExpectedCurrentVersion "v2" `
            -ExpectedTargetGeneration $helloHotSwap.body.result.service.generation `
            -ExpectedCurrentGeneration $helloHotSwapV2.body.result.service.generation `
            -ExpectedTargetArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedCurrentArtifactIdentityHash $helloHotSwapV2.body.result.service.artifact_identity_hash `
            -ExpectedStateHash $helloHotSwapStateHash `
            -ExpectedStateMigrationHash $helloHotSwapV2MigrationHash
        $helloRollbackTransactionPreflightHash = $helloRollbackApply.body.rollback_transaction_preflight.preflight_hash
        $helloRollbackWriteAuthorityGateHash = $helloRollbackApply.body.rollback_write_authority_gate.gate_hash
        $helloRollbackAppendIntentGateHash = $helloRollbackApply.body.rollback_append_intent_gate.gate_hash
        $helloRollbackPayloadEnvelopeGateHash = $helloRollbackApply.body.rollback_payload_envelope_gate.gate_hash
        $helloRollbackPayloadHash = $helloRollbackApply.body.rollback_payload_envelope_gate.proposed_transaction.payload_hash
        $helloRollbackPayloadProvenanceHash = $helloRollbackApply.body.rollback_payload_envelope_gate.proposed_transaction.provenance_hash
        $helloRollbackTransactionWriterStorageAuthorityGateHash = $helloRollbackApply.body.rollback_transaction_writer_storage_authority_gate.gate_hash
        $helloRollbackApplyDurablePreflight = $helloRollbackApply.body.rollback_transaction_writer_storage_authority_gate.writer_storage_foundation.transaction_writer_readiness.scratch_only_writer_dry_run.append_record_dry_run.sector_plan_dry_run.scratch_sector_write_readback_dry_run.durable_append_authority_preflight
        $helloRollbackApplyInspectSource = $helloRollbackApplyDurablePreflight.retained_recovery_rollback_inspect_source
        $helloRollbackApplyPolicyWriteAuthorityDecision = $helloRollbackApplyDurablePreflight.durable_policy_write_authority_decision
        if ($helloRollbackApply.body.source_durable_policy_write_authority_decision_hash -ne $helloRollbackApplyPolicyWriteAuthorityDecision.decision_hash -or $helloRollbackApply.body.source_recovery_rollback_inspect_source_reference_hash -ne $helloRollbackInspectSource.reference_hash -or -not $helloRollbackApply.body.retained_durable_policy_write_authority_decision_verified -or -not $helloRollbackApply.body.retained_recovery_rollback_inspect_source_reference_validated) {
            throw "Expected rollback apply denial hash to bind retained durable-policy decision and recovery inspect-source hashes"
        }
        if (-not $helloRollbackApplyInspectSource -or $helloRollbackApplyInspectSource.reference_hash -ne $helloRollbackInspectSource.reference_hash -or $helloRollbackApplyInspectSource.source_event_id -ne $helloRollbackInspectSource.source_event_id -or $helloRollbackApplyInspectSource.source_audit_event_id -ne $helloRollbackInspectSource.source_audit_event_id -or $helloRollbackApplyInspectSource.source_inspection_hash -ne $helloRollbackApplyDurablePreflight.target_region_sector_inspection.inspection_hash -or -not $helloRollbackApplyInspectSource.source_available -or -not $helloRollbackApplyInspectSource.source_matches_sector_inspection -or -not $helloRollbackApplyInspectSource.source_event_retained -or -not $helloRollbackApplyInspectSource.source_audit_event_retained -or $helloRollbackApplyInspectSource.ram_audit_status -ne "retained_audit_event_verified" -or $helloRollbackApplyInspectSource.ram_audit_reason -ne "recovery_rollback_inspect_source_reference_ram_audit_verified" -or -not $helloRollbackApplyInspectSource.ram_audit_validated -or $helloRollbackApplyInspectSource.authorizes_rollback_apply) {
            throw "Expected rollback apply denial to consume retained recovery inspect source reference without authorizing apply"
        }

        Send-AgentCommand -Command "service.health svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.health"
        $helloHealthAfterRollbackApply = Get-LastAgentResponseJson -Method "service.health"
        if ($helloHealthAfterRollbackApply.body.result.service.version -ne "v2" -or $helloHealthAfterRollbackApply.body.result.service.generation -ne $helloHotSwapV2.body.result.service.generation) {
            throw "Expected rollback apply denial to leave the active v2 service unchanged"
        }
        $helloHealthAfterRollbackApplyStateHash = & $AssertHelloState -Name "hello health after rollback apply denial" -State $helloHealthAfterRollbackApply.body.result.state -ExpectedCounter 3 -ExpectedVersion "v2"
        if ($helloHealthAfterRollbackApplyStateHash -ne $helloHotSwapV2StateHash) {
            throw "Expected rollback apply denial to preserve Hello RAM-only state"
        }

        Send-AgentCommand -Command "service.hot_swap svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.hot_swap"
        $helloHotSwapBack = Get-LastAgentResponseJson -Method "service.hot_swap"
        Assert-CurrentBootEventId -Name "quick:hello_hot_swap_back_event_id" -Value $helloHotSwapBack.body.result.event_id
        if ($helloHotSwapBack.body.result.service.version -ne "v1" -or $helloHotSwapBack.body.result.service.artifact_identity_hash -ne $helloArtifactIdentityHash) {
            throw "Expected service.hot_swap svc.demo.hello to return to the signed v1 built-in candidate"
        }
        if ($helloHotSwapBack.body.result.service.generation -ne ($helloHotSwapV2.body.result.service.generation + 1)) {
            throw "Expected accepted v1 service.hot_swap to advance the loaded hello generation after v2"
        }
        $helloHotSwapBackStateHash = & $AssertHelloState -Name "hello v1 hot-swap back response" -State $helloHotSwapBack.body.result.state -ExpectedCounter 3 -ExpectedVersion "v1"
        if ($helloHotSwapBackStateHash -ne $helloHotSwapV2StateHash) {
            throw "Expected v1 service.hot_swap back to preserve Hello RAM-only state"
        }
        $helloHotSwapBackMigrationHash = & $AssertHelloStateMigration -Name "hello v1 hot-swap back response" -Migration $helloHotSwapBack.body.result.state_migration -ExpectedFromVersion "v2" -ExpectedToVersion "v1" -ExpectedCounter 3 -ExpectedStateHash $helloHotSwapV2StateHash
        $helloHotSwapBackProbationHash = & $AssertHelloHotSwapProbation `
            -Name "hello v1 hot-swap back response" `
            -Probation $helloHotSwapBack.body.result.hot_swap_probation `
            -ExpectedPreviousVersion "v2" `
            -ExpectedNewVersion "v1" `
            -ExpectedPreviousGeneration $helloHotSwapV2.body.result.service.generation `
            -ExpectedNewGeneration $helloHotSwapBack.body.result.service.generation `
            -ExpectedPreviousStateHash $helloHotSwapV2StateHash `
            -ExpectedNewStateHash $helloHotSwapBackStateHash `
            -ExpectedPreviousArtifactIdentityHash $helloHotSwapV2.body.result.service.artifact_identity_hash `
            -ExpectedNewArtifactIdentityHash $helloArtifactIdentityHash `
            -ExpectedStateMigrationHash $helloHotSwapBackMigrationHash

        Send-AgentCommand -Command "service.drop svc.demo.hello" -ExpectedMarker "RAIOS_AGENT_END service.drop"
        $helloDrop = Get-LastAgentResponseJson -Method "service.drop"
        Assert-CurrentBootEventId -Name "quick:hello_drop_event_id" -Value $helloDrop.body.result.event_id
        if ($helloDrop.body.result.service.loaded -or $helloDrop.body.result.service.running) {
            throw "Expected dropped hello service after service.drop"
        }
        $helloDropStateHash = & $AssertHelloState -Name "hello drop response" -State $helloDrop.body.result.state -ExpectedCounter 0 -ExpectedVersion "v1"
        if ($helloDropStateHash -eq $helloHotSwapBackStateHash) {
            throw "Expected service.drop to clear Hello RAM-only state"
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
        & $AssertHelloState -Name "hello missing health response" -State $helloHealthMissing.body.result.state -ExpectedCounter 0 -ExpectedVersion "v1" | Out-Null
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

        Send-AgentCommand -Command "agent audit.events 72" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
        $recentEvents = Get-LastAgentResponseJson -Method "memory.recent_events"
        $envelopeAuditEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.agent_command_envelope.decision" })
        if ($envelopeAuditEvents.Count -ne 10) {
            throw "Expected ten agent command envelope audit events"
        }
        foreach ($event in $envelopeAuditEvents) {
            if ($event.classification -ne "local_only" -or $event.bindings.schema -ne "raios.agent_command_envelope.audit_binding.v0" -or $event.bindings.command_schema -ne "raios.agent_command_envelope.v0") {
                throw "Expected local-only agent command envelope audit binding"
            }
            if (
                $event.bindings.creates_parallel_dispatcher -or
                ($event.bindings.provider_write -ne "not_attempted") -or
                $event.bindings.loads_candidate_bytes -or
                $event.bindings.writes_persistent_state -or
                $event.bindings.writes_durable_audit_log -or
                $event.bindings.installs_rollback_plan -or
                $event.bindings.grants_broad_mutation
            ) {
                throw "Expected agent command envelope audit event to keep unsafe side effects disabled"
            }
        }
        $acceptedEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $agentEnvelope.body.result.audit_event_id } | Select-Object -First 1
        if (-not $acceptedEnvelopeEvent -or $acceptedEnvelopeEvent.outcome -ne "accepted" -or -not $acceptedEnvelopeEvent.bindings.accepted -or -not $acceptedEnvelopeEvent.bindings.dispatches_existing_agent_method -or $acceptedEnvelopeEvent.bindings.target_method -ne "system.describe") {
            throw "Expected accepted agent command envelope audit event to bind system.describe"
        }
        $systemSnapshotEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $systemSnapshotEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $systemSnapshotEnvelopeEvent -or $systemSnapshotEnvelopeEvent.outcome -ne "accepted" -or -not $systemSnapshotEnvelopeEvent.bindings.accepted -or -not $systemSnapshotEnvelopeEvent.bindings.dispatches_existing_agent_method -or $systemSnapshotEnvelopeEvent.bindings.target_method -ne "system.snapshot" -or $systemSnapshotEnvelopeEvent.bindings.requested_capability -ne "cap.system.snapshot.read") {
            throw "Expected accepted agent command envelope audit event to bind system.snapshot"
        }
        $bootLogEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $bootLogEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $bootLogEnvelopeEvent -or $bootLogEnvelopeEvent.outcome -ne "accepted" -or -not $bootLogEnvelopeEvent.bindings.accepted -or -not $bootLogEnvelopeEvent.bindings.dispatches_existing_agent_method -or $bootLogEnvelopeEvent.bindings.target_method -ne "system.boot_log" -or $bootLogEnvelopeEvent.bindings.requested_capability -ne "cap.system.boot_log.read") {
            throw "Expected accepted agent command envelope audit event to bind system.boot_log"
        }
        $systemCapabilitiesEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $systemCapabilitiesEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $systemCapabilitiesEnvelopeEvent -or $systemCapabilitiesEnvelopeEvent.outcome -ne "accepted" -or -not $systemCapabilitiesEnvelopeEvent.bindings.accepted -or -not $systemCapabilitiesEnvelopeEvent.bindings.dispatches_existing_agent_method -or $systemCapabilitiesEnvelopeEvent.bindings.target_method -ne "system.capabilities" -or $systemCapabilitiesEnvelopeEvent.bindings.requested_capability -ne "cap.system.capabilities.read") {
            throw "Expected accepted agent command envelope audit event to bind system.capabilities"
        }
        $deviceGraphEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $deviceGraphEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $deviceGraphEnvelopeEvent -or $deviceGraphEnvelopeEvent.outcome -ne "accepted" -or -not $deviceGraphEnvelopeEvent.bindings.accepted -or -not $deviceGraphEnvelopeEvent.bindings.dispatches_existing_agent_method -or $deviceGraphEnvelopeEvent.bindings.target_method -ne "device.graph" -or $deviceGraphEnvelopeEvent.bindings.requested_capability -ne "cap.device.graph.read") {
            throw "Expected accepted agent command envelope audit event to bind device.graph"
        }
        $serviceInventoryEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $serviceInventoryEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $serviceInventoryEnvelopeEvent -or $serviceInventoryEnvelopeEvent.outcome -ne "accepted" -or -not $serviceInventoryEnvelopeEvent.bindings.accepted -or -not $serviceInventoryEnvelopeEvent.bindings.dispatches_existing_agent_method -or $serviceInventoryEnvelopeEvent.bindings.target_method -ne "service.inventory" -or $serviceInventoryEnvelopeEvent.bindings.requested_capability -ne "cap.service.inventory.read") {
            throw "Expected accepted agent command envelope audit event to bind service.inventory"
        }
        $problemListEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $problemListEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $problemListEnvelopeEvent -or $problemListEnvelopeEvent.outcome -ne "accepted" -or -not $problemListEnvelopeEvent.bindings.accepted -or -not $problemListEnvelopeEvent.bindings.dispatches_existing_agent_method -or $problemListEnvelopeEvent.bindings.target_method -ne "problem.list" -or $problemListEnvelopeEvent.bindings.requested_capability -ne "cap.problem.list.read") {
            throw "Expected accepted agent command envelope audit event to bind problem.list"
        }
        $mismatchEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $mismatchEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $mismatchEnvelopeEvent -or $mismatchEnvelopeEvent.outcome -ne "capability_denied" -or $mismatchEnvelopeEvent.bindings.reason -ne "requested_capability_denied" -or $mismatchEnvelopeEvent.bindings.target_method -ne "service.inventory" -or -not $mismatchEnvelopeEvent.bindings.target_method_allowed -or $mismatchEnvelopeEvent.bindings.requested_capability -ne "cap.system.describe.read" -or $mismatchEnvelopeEvent.bindings.requested_capability_allowed -or $mismatchEnvelopeEvent.bindings.dispatches_existing_agent_method) {
            throw "Expected target/capability mismatch audit event to be denied before dispatch"
        }
        $badEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $badEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $badEnvelopeEvent -or $badEnvelopeEvent.outcome -ne "invalid_envelope" -or $badEnvelopeEvent.bindings.schema_ok -or $badEnvelopeEvent.bindings.reason -ne "schema_mismatch" -or $badEnvelopeEvent.bindings.dispatches_existing_agent_method) {
            throw "Expected bad-schema agent command envelope audit event"
        }
        $overCapEnvelopeEvent = $envelopeAuditEvents | Where-Object { $_.id -eq $overCapEnvelope.body.result.event_id } | Select-Object -First 1
        if (-not $overCapEnvelopeEvent -or $overCapEnvelopeEvent.outcome -ne "capability_denied" -or $overCapEnvelopeEvent.bindings.reason -ne "target_method_not_allowed" -or $overCapEnvelopeEvent.bindings.target_method -ne "module.load_ephemeral" -or $overCapEnvelopeEvent.bindings.dispatches_existing_agent_method) {
            throw "Expected over-capable agent command envelope audit event to be denied before dispatch"
        }
        $helloEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.ram_only_hello_service.lifecycle" -and $_.resource -eq "svc.demo.hello" })
        if ($helloEvents.Count -lt 6) {
            throw "Expected hello load/stop/drop lifecycle events in RAM audit log"
        }
        $helloRestartEvents = @($helloEvents | Where-Object { $_.source_method -eq "service.restart" -and $_.reason -eq "restarted_loaded_service" })
        if ($helloRestartEvents.Count -lt 1) {
            throw "Expected hello lifecycle events to include service.restart"
        }
        $helloHotSwapEvents = @($helloEvents | Where-Object { $_.source_method -eq "service.hot_swap" -and $_.reason -eq "hot_swapped_builtin_service" })
        if ($helloHotSwapEvents.Count -lt 1) {
            throw "Expected hello lifecycle events to include service.hot_swap"
        }
        $helloHotSwapV2Events = @($helloHotSwapEvents | Where-Object { $_.bindings.artifact_identity_id -eq $HelloArtifactIdentityV2Id })
        if ($helloHotSwapV2Events.Count -lt 1) {
            throw "Expected hello lifecycle events to include the signed v2 hot-swap candidate"
        }
        $helloStateEvents = @($helloEvents | Where-Object { $_.bindings.hello_state_id -eq $HelloStateId -and $_.bindings.hello_state_hash -and $_.bindings.hello_state_hash.StartsWith("sha256:") })
        if ($helloStateEvents.Count -lt 6) {
            throw "Expected hello lifecycle events to cite RAM-only Hello state"
        }
        $helloResetDeniedEvents = @($helloEvents | Where-Object { $_.id -eq $resetHelloHotSwap.body.event_id -and $_.outcome -eq "capability_denied" -and $_.reason -eq "state_migration_would_reset_state" -and $_.bindings.state_migration_schema -eq $HelloStateMigrationSchema -and $_.bindings.state_migration_id -eq $HelloStateMigrationId -and $_.bindings.state_migration_hash -eq $resetHelloMigrationHash -and $_.bindings.pre_migration_state_counter -eq 3 -and $_.bindings.post_migration_state_counter -eq 0 -and $_.bindings.pre_migration_state_hash -eq $helloRestartStateHash -and -not $_.bindings.state_migration_preserved -and -not $_.bindings.state_migration_accepted })
        if ($helloResetDeniedEvents.Count -lt 1) {
            throw "Expected reset-state hot-swap denial audit event to bind the rejected state migration"
        }
        $helloHotSwapV2MigrationEvents = @($helloHotSwapV2Events | Where-Object { $_.bindings.state_migration_schema -eq $HelloStateMigrationSchema -and $_.bindings.state_migration_id -eq $HelloStateMigrationId -and $_.bindings.state_migration_hash -eq $helloHotSwapV2MigrationHash -and $_.bindings.migration_from_version -eq "v1" -and $_.bindings.migration_to_version -eq "v2" -and $_.bindings.pre_migration_state_counter -eq 3 -and $_.bindings.post_migration_state_counter -eq 3 -and $_.bindings.pre_migration_state_hash -eq $helloHotSwapStateHash -and $_.bindings.post_migration_state_hash -eq $helloHotSwapStateHash -and $_.bindings.state_migration_preserved -and $_.bindings.state_migration_accepted })
        if ($helloHotSwapV2MigrationEvents.Count -lt 1) {
            throw "Expected v2 hot-swap lifecycle audit event to bind preserved RAM-only state migration"
        }
        $helloHotSwapV2ProbationEvents = @($helloHotSwapV2Events | Where-Object { $_.bindings.hot_swap_probation_schema -eq $HelloHotSwapProbationSchema -and $_.bindings.hot_swap_probation_id -eq $HelloHotSwapProbationId -and $_.bindings.hot_swap_probation_hash -eq $helloHotSwapV2ProbationHash -and $_.bindings.hot_swap_probation_status -eq $HelloHotSwapProbationStatus -and $_.bindings.hot_swap_probation_previous_version -eq "v1" -and $_.bindings.hot_swap_probation_new_version -eq "v2" -and $_.bindings.hot_swap_probation_previous_artifact_identity_hash -eq $helloArtifactIdentityHash -and $_.bindings.hot_swap_probation_new_artifact_identity_hash -eq $helloHotSwapV2.body.result.service.artifact_identity_hash -and $_.bindings.hot_swap_probation_previous_generation -eq $helloHotSwap.body.result.service.generation -and $_.bindings.hot_swap_probation_new_generation -eq $helloHotSwapV2.body.result.service.generation -and $_.bindings.hot_swap_probation_previous_state_hash -eq $helloHotSwapStateHash -and $_.bindings.hot_swap_probation_new_state_hash -eq $helloHotSwapStateHash -and $_.bindings.hot_swap_probation_state_migration_hash -eq $helloHotSwapV2MigrationHash -and $_.bindings.hot_swap_probation_accepted -and -not $_.bindings.hot_swap_probation_writes_persistent_state -and -not $_.bindings.hot_swap_probation_writes_durable_audit_log -and -not $_.bindings.hot_swap_probation_installs_rollback_plan -and -not $_.bindings.hot_swap_probation_applies_rollback })
        if ($helloHotSwapV2ProbationEvents.Count -lt 1) {
            throw "Expected v2 hot-swap lifecycle audit event to bind RAM-only probation evidence"
        }
        $helloRollbackPreviewEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.ram_only_hello_service.rollback_preview" -and $_.id -eq $helloRollbackPreview.body.result.audit_event_id -and $_.outcome -eq "response" -and $_.requested_capability -eq "cap.service.rollback_preview.read" -and $_.bindings.schema -eq "raios.ram_only_hello_service.rollback_preview_binding.v0" -and $_.bindings.rollback_preview_hash -eq $helloRollbackPreview.body.result.preview_hash -and $_.bindings.hot_swap_probation_hash -eq $helloHotSwapV2ProbationHash -and $_.bindings.hot_swap_probation_previous_artifact_identity_hash -eq $helloArtifactIdentityHash -and $_.bindings.hot_swap_probation_new_artifact_identity_hash -eq $helloHotSwapV2.body.result.service.artifact_identity_hash -and $_.bindings.hot_swap_probation_previous_generation -eq $helloHotSwap.body.result.service.generation -and $_.bindings.hot_swap_probation_new_generation -eq $helloHotSwapV2.body.result.service.generation -and $_.bindings.hot_swap_probation_state_migration_hash -eq $helloHotSwapV2MigrationHash -and -not $_.bindings.hot_swap_probation_applies_rollback -and -not $_.bindings.hot_swap_probation_installs_rollback_plan })
        if ($helloRollbackPreviewEvents.Count -lt 1) {
            throw "Expected rollback preview audit event to bind retained hot-swap probation without rollback apply"
        }
        $helloRollbackApplyEvents = @($recentEvents.body.result.events | Where-Object { $_.kind -eq "raios.ram_only_hello_service.rollback_apply" -and $_.id -eq $helloRollbackApply.body.audit_event_id -and $_.outcome -eq "capability_denied" -and $_.requested_capability -eq "cap.service.rollback_apply.current_boot" -and $_.bindings.schema -eq "raios.ram_only_hello_service.rollback_apply_denial_binding.v0" -and $_.bindings.rollback_preview_hash -eq $helloRollbackPreview.body.result.preview_hash -and $_.bindings.rollback_apply_hash -eq $helloRollbackApplyHash -and $_.bindings.rollback_apply_status -eq $HelloRollbackApplyStatus -and -not $_.bindings.rollback_apply_authorized -and -not $_.bindings.rollback_apply_mutates_service_state -and $_.bindings.rollback_transaction_preflight_schema -eq $HelloRollbackTransactionPreflightSchema -and $_.bindings.rollback_transaction_preflight_id -eq $HelloRollbackTransactionPreflightId -and $_.bindings.rollback_transaction_preflight_hash -eq $helloRollbackTransactionPreflightHash -and $_.bindings.rollback_transaction_preflight_status -eq $HelloRollbackTransactionPreflightStatus -and $_.bindings.rollback_transaction_authority_missing -and $_.bindings.rollback_durable_audit_write_authority_missing -and $_.bindings.rollback_persistent_install_authority_missing -and -not $_.bindings.rollback_transaction_writes_durable_audit_log -and -not $_.bindings.rollback_transaction_writes_rollback_store -and -not $_.bindings.rollback_transaction_installs_rollback_plan -and -not $_.bindings.rollback_transaction_applies_rollback -and $_.bindings.rollback_write_authority_gate_schema -eq $HelloRollbackWriteAuthorityGateSchema -and $_.bindings.rollback_write_authority_gate_id -eq $HelloRollbackWriteAuthorityGateId -and $_.bindings.rollback_write_authority_gate_hash -eq $helloRollbackWriteAuthorityGateHash -and $_.bindings.rollback_write_authority_gate_status -eq $HelloRollbackWriteAuthorityGateStatus -and $_.bindings.rollback_write_authority_required_audit_schema -eq "raios.audit_record.v0" -and $_.bindings.rollback_write_authority_required_rollback_schema -eq "raios.rollback_transaction.v0" -and -not $_.bindings.rollback_durable_audit_write_authority_available -and -not $_.bindings.rollback_store_write_authority_available -and -not $_.bindings.rollback_transaction_append_available -and $_.bindings.hot_swap_probation_hash -eq $helloHotSwapV2ProbationHash -and $_.bindings.hot_swap_probation_previous_artifact_identity_hash -eq $helloArtifactIdentityHash -and $_.bindings.hot_swap_probation_new_artifact_identity_hash -eq $helloHotSwapV2.body.result.service.artifact_identity_hash -and $_.bindings.hot_swap_probation_previous_generation -eq $helloHotSwap.body.result.service.generation -and $_.bindings.hot_swap_probation_new_generation -eq $helloHotSwapV2.body.result.service.generation -and $_.bindings.hello_state_hash -eq $helloHotSwapV2StateHash -and $_.bindings.hello_state_counter -eq 3 -and -not $_.bindings.hot_swap_probation_applies_rollback -and -not $_.bindings.hot_swap_probation_installs_rollback_plan })
        if ($helloRollbackApplyEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to bind preview/probation/state evidence without mutating service state"
        }
        $helloRollbackApplyAppendEvents = @($helloRollbackApplyEvents | Where-Object { $_.bindings.rollback_append_intent_gate_schema -eq $HelloRollbackAppendIntentGateSchema -and $_.bindings.rollback_append_intent_gate_id -eq $HelloRollbackAppendIntentGateId -and $_.bindings.rollback_append_intent_gate_hash -eq $helloRollbackAppendIntentGateHash -and $_.bindings.rollback_append_intent_gate_status -eq $HelloRollbackAppendIntentGateStatus -and $_.bindings.rollback_append_intent_required_audit_schema -eq "raios.audit_record.v0" -and $_.bindings.rollback_append_intent_required_rollback_schema -eq "raios.rollback_transaction.v0" -and -not $_.bindings.rollback_append_intent_available -and -not $_.bindings.rollback_append_durable_audit_store_available -and -not $_.bindings.rollback_append_store_available -and -not $_.bindings.rollback_append_transaction_append_available })
        if ($helloRollbackApplyAppendEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to bind append-intent gate evidence without appending a transaction"
        }
        $helloRollbackApplyPayloadEvents = @($helloRollbackApplyAppendEvents | Where-Object { $_.bindings.rollback_payload_envelope_gate_schema -eq $HelloRollbackPayloadEnvelopeGateSchema -and $_.bindings.rollback_payload_envelope_gate_id -eq $HelloRollbackPayloadEnvelopeGateId -and $_.bindings.rollback_payload_envelope_gate_hash -eq $helloRollbackPayloadEnvelopeGateHash -and $_.bindings.rollback_payload_envelope_gate_status -eq $HelloRollbackPayloadEnvelopeGateStatus -and $_.bindings.rollback_payload_envelope_required_audit_schema -eq "raios.audit_record.v0" -and $_.bindings.rollback_payload_envelope_required_rollback_schema -eq "raios.rollback_transaction.v0" -and $_.bindings.rollback_payload_schema -eq $HelloRollbackTransactionPayloadSchema -and $_.bindings.rollback_payload_id -eq $HelloRollbackTransactionPayloadId -and $_.bindings.rollback_payload_hash -eq $helloRollbackPayloadHash -and $_.bindings.rollback_payload_status -eq $HelloRollbackTransactionPayloadStatus -and $_.bindings.rollback_payload_provenance_hash -eq $helloRollbackPayloadProvenanceHash -and -not $_.bindings.rollback_payload_writer_available -and -not $_.bindings.rollback_payload_durable_audit_store_available -and -not $_.bindings.rollback_payload_store_available -and -not $_.bindings.rollback_payload_transaction_append_available })
        if ($helloRollbackApplyPayloadEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to bind payload-envelope evidence without writing a transaction"
        }
        $helloRollbackApplyWriterStorageEvents = @($helloRollbackApplyPayloadEvents | Where-Object { $_.bindings.rollback_transaction_writer_storage_authority_gate_schema -eq $HelloRollbackTransactionWriterStorageAuthorityGateSchema -and $_.bindings.rollback_transaction_writer_storage_authority_gate_id -eq $HelloRollbackTransactionWriterStorageAuthorityGateId -and $_.bindings.rollback_transaction_writer_storage_authority_gate_hash -eq $helloRollbackTransactionWriterStorageAuthorityGateHash -and $_.bindings.rollback_transaction_writer_storage_authority_gate_status -eq $HelloRollbackTransactionWriterStorageAuthorityGateStatus -and $_.bindings.rollback_transaction_writer_storage_required_audit_schema -eq "raios.audit_record.v0" -and $_.bindings.rollback_transaction_writer_storage_required_rollback_schema -eq "raios.rollback_transaction.v0" -and -not $_.bindings.rollback_transaction_writer_storage_transaction_writer_available -and -not $_.bindings.rollback_transaction_writer_storage_durable_audit_store_available -and -not $_.bindings.rollback_transaction_writer_storage_rollback_store_available -and -not $_.bindings.rollback_transaction_writer_storage_append_available })
        if ($helloRollbackApplyWriterStorageEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to bind writer/storage authority gate evidence without writing a transaction"
        }
        $helloRollbackApplyWriterStorageFoundationEvents = @($helloRollbackApplyWriterStorageEvents | Where-Object { $_.bindings.rollback_transaction_writer_storage_foundation_schema -eq $HelloRollbackTransactionWriterStorageFoundationSchema -and $_.bindings.rollback_transaction_writer_storage_foundation_owner -eq $HelloRollbackTransactionWriterStorageFoundationOwner -and $_.bindings.rollback_transaction_writer_storage_authority_id -eq $HelloRollbackTransactionWriterStorageAuthorityId -and $_.bindings.rollback_transaction_writer_storage_authority_owner -eq $HelloRollbackTransactionWriterStorageAuthorityOwner -and $_.bindings.rollback_transaction_writer_storage_audit_target_id -eq $HelloRollbackTransactionWriterStorageAuditTargetId -and $_.bindings.rollback_transaction_writer_storage_audit_target_schema -eq $HelloRollbackTransactionWriterStorageAuditTargetSchema -and $_.bindings.rollback_transaction_writer_storage_append_target_id -eq $HelloRollbackTransactionWriterStorageTargetId -and $_.bindings.rollback_transaction_writer_storage_append_target_schema -eq $HelloRollbackTransactionWriterStorageTargetSchema -and $_.bindings.rollback_transaction_writer_storage_transaction_writer_owner -eq $HelloRollbackTransactionWriterStorageWriterOwner -and $_.bindings.rollback_transaction_writer_storage_layout_status -eq $HelloRollbackTransactionWriterStorageLayoutStatus -and $_.bindings.rollback_transaction_writer_storage_layout_reason -eq $HelloRollbackTransactionWriterStorageLayoutReason -and $_.bindings.rollback_transaction_writer_storage_append_engine_status -eq $HelloRollbackTransactionWriterStorageAppendEngineStatus -and $_.bindings.rollback_transaction_writer_storage_append_engine_reason -eq $HelloRollbackTransactionWriterStorageAppendEngineReason -and $_.bindings.rollback_transaction_writer_storage_append_contract_status -eq $HelloRollbackTransactionWriterStorageAppendContractStatus -and $_.bindings.rollback_transaction_writer_storage_append_contract_reason -eq $HelloRollbackTransactionWriterStorageAppendContractReason -and $_.bindings.rollback_transaction_writer_storage_transaction_envelope_status -eq $HelloRollbackTransactionWriterStorageEnvelopeStatus -and $_.bindings.rollback_transaction_writer_storage_transaction_envelope_reason -eq $HelloRollbackTransactionWriterStorageEnvelopeReason })
        if ($helloRollbackApplyWriterStorageFoundationEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to retain shared writer/storage foundation evidence"
        }
        $helloRollbackApplyWriterReadinessEvents = @($helloRollbackApplyWriterStorageFoundationEvents | Where-Object { $_.bindings.rollback_transaction_writer_storage_append_target_owner_schema -eq $HelloRollbackAppendTargetOwnerSchema -and $_.bindings.rollback_transaction_writer_storage_append_target_owner_id -eq $HelloRollbackAppendTargetOwnerId -and $_.bindings.rollback_transaction_writer_storage_append_target_owner_status -eq $HelloRollbackAppendTargetOwnerStatus -and $_.bindings.rollback_transaction_writer_storage_append_target_owner_reason -eq $HelloRollbackAppendTargetOwnerReason -and $_.bindings.rollback_transaction_writer_storage_transaction_writer_readiness_schema -eq $HelloRollbackTransactionWriterReadinessSchema -and $_.bindings.rollback_transaction_writer_storage_transaction_writer_readiness_id -eq $HelloRollbackTransactionWriterReadinessId -and $_.bindings.rollback_transaction_writer_storage_transaction_writer_status -eq $HelloRollbackTransactionWriterReadinessStatus -and $_.bindings.rollback_transaction_writer_storage_transaction_writer_reason -eq $HelloRollbackTransactionWriterReadinessReason -and -not $_.bindings.rollback_transaction_writer_storage_block_write_path_available })
        if ($helloRollbackApplyWriterReadinessEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to retain denied append target owner and transaction writer readiness evidence"
        }
        & $AssertHelloRollbackScratchWriterDryRunBinding -Name "rollback apply audit event" -Bindings $helloRollbackApplyWriterReadinessEvents[0].bindings
        $helloRollbackApplyWriterWritePathGateEvents = @($helloRollbackApplyWriterReadinessEvents | Where-Object { $_.bindings.rollback_transaction_writer_storage_block_write_path_gate_schema -eq $HelloRollbackBlockWritePathGateSchema -and $_.bindings.rollback_transaction_writer_storage_block_write_path_gate_id -eq $HelloRollbackBlockWritePathGateId -and $_.bindings.rollback_transaction_writer_storage_block_write_path_gate_status -eq $HelloRollbackBlockWritePathGateStatus -and $_.bindings.rollback_transaction_writer_storage_block_write_path_gate_reason -eq $HelloRollbackTransactionWriterStorageLayoutReason -and -not $_.bindings.rollback_transaction_writer_storage_block_write_path_available -and $_.bindings.rollback_transaction_writer_storage_read_only_block_driver_id -eq $HelloRollbackReadOnlyBlockDriverId -and $_.bindings.rollback_transaction_writer_storage_read_only_block_driver_available -and $_.bindings.rollback_transaction_writer_storage_partition_inventory_available -and $_.bindings.rollback_transaction_writer_storage_partition_inventory_scheme -eq $HelloRollbackPartitionInventoryScheme })
        if ($helloRollbackApplyWriterWritePathGateEvents.Count -lt 1) {
            throw "Expected rollback apply audit event to bind the fail-closed block write-path gate evidence"
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
        Assert-LogContains -Name "quick:audit_events_limit" -Needle '"limit": 72' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_module_load_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_recovery_load_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_kind" -Needle '"kind": "raios.ram_only_hello_service.lifecycle"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_health_kind" -Needle '"kind": "raios.ram_only_hello_service.health"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_preview_kind" -Needle '"kind": "raios.ram_only_hello_service.rollback_preview"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_kind" -Needle '"kind": "raios.ram_only_hello_service.rollback_apply"' -TimeoutSeconds 1
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
        Assert-LogContains -Name "quick:audit_events_hello_state_hash" -Needle '"hello_state_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_state_migration_hash" -Needle '"state_migration_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_state_migration_accepted" -Needle '"state_migration_accepted": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_hot_swap_probation_hash" -Needle '"hot_swap_probation_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_hot_swap_probation_status" -Needle '"hot_swap_probation_status": "active_current_boot_probation"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_preview_hash" -Needle '"rollback_preview_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_hash" -Needle '"rollback_apply_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_source_policy_decision" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_source_inspect_reference" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_source_policy_verified" -Needle '"rollback_apply_source_durable_policy_write_authority_decision_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_source_inspect_validated" -Needle '"rollback_apply_source_recovery_rollback_inspect_source_reference_validated": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_apply_denied" -Needle '"rollback_apply_authorized": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_preflight_hash" -Needle '"rollback_transaction_preflight_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_preflight_missing_authority" -Needle '"rollback_transaction_authority_missing": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_preflight_no_durable_write" -Needle '"rollback_transaction_writes_durable_audit_log": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_write_gate_hash" -Needle '"rollback_write_authority_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_write_gate_status" -Needle '"rollback_write_authority_gate_status": "denied_missing_durable_write_authority"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_write_gate_unavailable" -Needle '"rollback_store_write_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_gate_hash" -Needle '"rollback_append_intent_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_gate_status" -Needle '"rollback_append_intent_gate_status": "denied_missing_rollback_transaction_append_authority"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_gate_unavailable" -Needle '"rollback_append_intent_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_payload_gate_hash" -Needle '"rollback_payload_envelope_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_payload_hash" -Needle '"rollback_payload_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_payload_writer_unavailable" -Needle '"rollback_payload_writer_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_gate_schema" -Needle "raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_gate_hash" -Needle '"rollback_transaction_writer_storage_authority_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_foundation" -Needle '"rollback_transaction_writer_storage_foundation_schema": "raios.module_audit_rollback_append_contract.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_authority" -Needle '"rollback_transaction_writer_storage_authority_id": "storage.authority.audit_rollback.current_boot"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_audit_target" -Needle '"rollback_transaction_writer_storage_audit_target_id": "append.audit_ledger.current_boot"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_target" -Needle '"rollback_transaction_writer_storage_append_target_schema": "raios.rollback_transaction.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_target_owner" -Needle '"rollback_transaction_writer_storage_append_target_owner_schema": "raios.audit_rollback_append_target_owner.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_writer_readiness" -Needle '"rollback_transaction_writer_storage_transaction_writer_readiness_schema": "raios.audit_rollback_transaction_writer_readiness.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_scratch_writer_dry_run" -Needle '"rollback_transaction_writer_storage_scratch_dry_run_schema": "raios.audit_rollback_transaction_writer_scratch_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_scratch_target_ready" -Needle '"rollback_transaction_writer_storage_scratch_target_ready": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_record_dry_run" -Needle '"rollback_transaction_writer_storage_append_record_dry_run_schema": "raios.ram_only_hello_service_rollback_append_record_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_record_hash" -Needle '"rollback_transaction_writer_storage_append_record_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_record_total_bytes" -Needle '"rollback_transaction_writer_storage_append_record_total_byte_length": 480' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_sector_plan" -Needle '"rollback_transaction_writer_storage_append_sector_plan_dry_run_schema": "raios.ram_only_hello_service_rollback_append_sector_plan_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_sector_image_hash" -Needle '"rollback_transaction_writer_storage_append_sector_image_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_sector_padding" -Needle '"rollback_transaction_writer_storage_append_sector_padding_byte_length": 32' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_sector_write_readback" -Needle '"rollback_transaction_writer_storage_append_sector_write_readback_dry_run_schema": "raios.ram_only_hello_service_rollback_append_sector_write_readback_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_sector_readback_match" -Needle '"rollback_transaction_writer_storage_append_sector_write_readback_matches_planned_image": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_schema": "raios.ram_only_hello_service_rollback_durable_append_authority_preflight.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight_denied" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_status": "denied_missing_durable_append_authority"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight_target_write_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_source_target_region_write_readback_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight_test_media" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_test_media_write_authority_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight_remaining_reason" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_remaining_denial_reason": "durable_append_authority_missing"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_writer_policy_preflight" -Needle '"rollback_transaction_writer_storage_durable_writer_policy_preflight_schema": "raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_writer_policy_preflight_hash" -Needle '"rollback_transaction_writer_storage_durable_writer_policy_preflight_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_writer_policy_preflight_test_media" -Needle '"rollback_transaction_writer_storage_durable_writer_policy_preflight_test_media_write_authority_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_writer_policy_preflight_denied" -Needle '"rollback_transaction_writer_storage_durable_writer_policy_preflight_status": "candidate_ready_not_append_authority"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_engine_readiness_decision" -Needle '"rollback_transaction_writer_storage_append_engine_readiness_decision_schema": "raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_engine_readiness_decision_hash" -Needle '"rollback_transaction_writer_storage_append_engine_readiness_decision_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_append_engine_readiness_ready_no_authority" -Needle '"rollback_transaction_writer_storage_append_engine_readiness_decision_ready": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_policy_preflight" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema": "raios.audit_rollback_target_region_media_write_policy_preflight.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_policy_preflight_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_policy_denied" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_status": "denied_missing_media_write_authority_and_durable_audit_policy"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_policy_no_write" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_media_write": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_authority_gate" -Needle '"rollback_transaction_writer_storage_media_write_authority_gate_schema": "raios.ram_only_hello_service_rollback_media_write_authority_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_authority_gate_hash" -Needle '"rollback_transaction_writer_storage_media_write_authority_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_authority_gate_denied" -Needle '"rollback_transaction_writer_storage_media_write_authority_gate_status": "denied_missing_durable_audit_policy"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_media_write_authority_gate_target_write_seen" -Needle '"rollback_transaction_writer_storage_media_write_authority_gate_target_region_write_attempted": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_decision" -Needle '"rollback_transaction_writer_storage_durable_append_authority_decision_schema": "raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_decision_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_decision_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_decision_ready" -Needle '"rollback_transaction_writer_storage_durable_append_authority_decision_append_engine_ready": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_decision_denied" -Needle '"rollback_transaction_writer_storage_durable_append_authority_decision_durable_append_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_decision" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_decision_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_decision_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_decision_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_decision_verified" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_decision_media_write_policy_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_decision_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_decision_durable_audit_policy_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_candidate" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_candidate_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_candidate_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_candidate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_candidate_available" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_candidate_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_candidate_not_authority" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_audit_policy_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_acceptance_gate" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_acceptance_gate_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_acceptance_gate_candidate_available" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_candidate_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_acceptance_gate_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_policy_ledger_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_candidate" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_candidate_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_candidate_read_only" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_read_only_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_candidate_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_aware_acceptance_result" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_aware_acceptance_result_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_aware_acceptance_result_verified" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_ledger_evidence_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_ledger_aware_acceptance_result_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_write_authority_availability" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_write_authority_availability.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_write_authority_availability_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_write_authority_availability_verified" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_region_write_readback_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_write_authority_availability_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_schema": "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_hash" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_verified" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_evidence_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_denied" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_policy_ledger_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_dry_run" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_dry_run_denied" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_policy_ledger_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_policy_ledger_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_dry_run" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_dry_run_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_audit_policy_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_schema": "raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_hash" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_verified" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_policy_ledger_availability_evidence_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_audit_policy_availability_denied" -Needle '"rollback_transaction_writer_storage_durable_audit_policy_availability_durable_audit_policy_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_dry_run" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_schema": "raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_dry_run_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_dry_run_gate" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_denial_gate_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_dry_run_denied" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_append_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_dry_run_no_apply" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_applies_rollback": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_schema": "raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_hash" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_verified" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_audit_policy_availability_evidence_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_authority_availability_denied" -Needle '"rollback_transaction_writer_storage_durable_append_authority_availability_durable_append_authority_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_availability_decision" -Needle '"rollback_transaction_writer_storage_transaction_append_availability_decision_schema": "raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_availability_decision_hash" -Needle '"rollback_transaction_writer_storage_transaction_append_availability_decision_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_availability_ready" -Needle '"rollback_transaction_writer_storage_transaction_append_availability_decision_append_engine_ready": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_availability_denied" -Needle '"rollback_transaction_writer_storage_transaction_append_availability_decision_transaction_append_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_authority_denial_gate" -Needle '"rollback_transaction_writer_storage_transaction_append_authority_denial_gate_schema": "raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_authority_denial_gate_hash" -Needle '"rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_authority_denial_gate_missing" -Needle '"rollback_transaction_writer_storage_transaction_append_authority_denial_gate_missing_transaction_append_authority": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_authority_denial_gate_denied" -Needle '"rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_transaction_append": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_dry_run" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_schema": "raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_dry_run_hash" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_dry_run_blocked" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_blocked_by_authority_denial_gate": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_transaction_append_dry_run_no_append" -Needle '"rollback_transaction_writer_storage_transaction_append_dry_run_appends_rollback_transaction": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_sector_inspection" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_schema": "raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_sector_inspection_hash" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_sector_inspection_verified" -Needle '"rollback_transaction_writer_storage_target_region_sector_inspection_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_recovery_inspect_source_reference" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_schema": "raios.recovery_rollback_inspect_source_reference.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_recovery_inspect_source_reference_hash" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_recovery_inspect_source_reference_matches" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_matches_sector_inspection": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_recovery_inspect_source_reference_ram_audit" -Needle '"rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_validated": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_recovery_inspect_source_reference_binding" -Needle '"schema": "raios.recovery_rollback_inspect_source_reference_binding.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_policy_write_consumes_sector_inspection" -Needle '"rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_region_sector_inspection_verified": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_write_readback" -Needle '"rollback_transaction_writer_storage_target_region_write_readback_dry_run_schema": "raios.ram_only_hello_service_rollback_target_region_write_readback_dry_run.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_write_readback_hash" -Needle '"rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_write_completed" -Needle '"rollback_transaction_writer_storage_target_region_write_readback_write_completed": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_readback_match" -Needle '"rollback_transaction_writer_storage_target_region_write_readback_matches_planned_image": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_write_no_append" -Needle '"rollback_transaction_writer_storage_target_region_write_readback_authorizes_append": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_durable_append_preflight_scratch_not_authority" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_used_as_durable_authority": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_discovery" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_schema": "raios.audit_rollback_target_region_discovery.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_read_only" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_reason": "dedicated_audit_rollback_region_discovered_read_only"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_candidate" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_present": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_scratch_rejected" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_scratch_rejected_as_durable_authority": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_target_region_available" -Needle '"rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_durable_region_available": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_block_write_path_gate" -Needle '"rollback_transaction_writer_storage_block_write_path_gate_schema": "raios.block_write_path_authority_gate.v0"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_read_only_block_driver" -Needle '"rollback_transaction_writer_storage_read_only_block_driver_id": "block_driver.ahci.read_only.current_boot"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_partition_inventory" -Needle '"rollback_transaction_writer_storage_partition_inventory_scheme": "mbr_empty"' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_rollback_writer_storage_unavailable" -Needle '"rollback_transaction_writer_storage_transaction_writer_available": false' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_kind" -Needle "current_image_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_source_kind" -Needle "host_bound_descriptor_source" -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_host_bound_binds_hash" -Needle '"binds_source_hash": "sha256:' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_hello_descriptor_source_validated" -Needle '"load_descriptor_source_validated": true' -TimeoutSeconds 1
        Assert-LogContains -Name "quick:audit_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1
