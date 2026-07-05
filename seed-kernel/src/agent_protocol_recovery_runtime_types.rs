use crate::{
    agent_protocol_recovery_lifeline::RecoveryLifelineCommandSpec,
    agent_protocol_recovery_lifeline_protocol_types::RecoveryLifelineCommandVocabularyCandidate,
    agent_protocol_support::parse_sha256_ref, event_log,
};

#[derive(Clone, Copy)]
pub(crate) struct StageBinding<'a> {
    pub(crate) stage_hash: Option<[u8; 32]>,
    pub(crate) expected_hash: Option<[u8; 32]>,
    pub(crate) retained_previous_event_id: Option<&'a str>,
}

impl<'a> StageBinding<'a> {
    pub(crate) const fn empty() -> Self {
        Self {
            stage_hash: None,
            expected_hash: None,
            retained_previous_event_id: None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CommandBindings<'a> {
    pub(crate) stage: StageBinding<'a>,
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) command_envelope_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_command_envelope_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_lifeline_request_event_id: Option<&'a str>,
    pub(crate) command_id: Option<&'a str>,
    pub(crate) argument_schema: Option<&'a str>,
    pub(crate) argument_hash: Option<[u8; 32]>,
    pub(crate) required_capability: Option<&'a str>,
    pub(crate) target_locator: Option<&'a str>,
    pub(crate) command_admission_boundary_id: Option<&'a str>,
    pub(crate) lifeline_request_reference_hash: Option<[u8; 32]>,
    pub(crate) command_body_canonicalization_hash: Option<[u8; 32]>,
    pub(crate) expected_command_body_canonicalization_hash: Option<[u8; 32]>,
    pub(crate) retained_command_envelope_reference_event_id: Option<&'a str>,
    pub(crate) command_dispatch_boundary_id: Option<&'a str>,
    pub(crate) handler_binding_hash: Option<[u8; 32]>,
    pub(crate) expected_handler_binding_hash: Option<[u8; 32]>,
    pub(crate) retained_command_body_canonicalization_event_id: Option<&'a str>,
    pub(crate) handler_id: Option<&'a str>,
    pub(crate) handler_input_binding_hash: Option<[u8; 32]>,
    pub(crate) status_read_handler_hash: Option<[u8; 32]>,
    pub(crate) expected_status_read_handler_hash: Option<[u8; 32]>,
    pub(crate) retained_command_handler_binding_event_id: Option<&'a str>,
    pub(crate) status_handler_id: Option<&'a str>,
    pub(crate) status_read_projection_hash: Option<[u8; 32]>,
    pub(crate) rollback_preview_authorization_hash: Option<[u8; 32]>,
    pub(crate) expected_rollback_preview_authorization_hash: Option<[u8; 32]>,
    pub(crate) retained_status_read_handler_event_id: Option<&'a str>,
    pub(crate) rollback_preview_authorization_id: Option<&'a str>,
    pub(crate) rollback_preview_projection_hash: Option<[u8; 32]>,
    pub(crate) rollback_apply_authorization_hash: Option<[u8; 32]>,
    pub(crate) expected_rollback_apply_authorization_hash: Option<[u8; 32]>,
    pub(crate) retained_rollback_preview_authorization_event_id: Option<&'a str>,
    pub(crate) rollback_apply_authorization_id: Option<&'a str>,
    pub(crate) rollback_apply_projection_hash: Option<[u8; 32]>,
    pub(crate) disable_module_target_binding_hash: Option<[u8; 32]>,
    pub(crate) expected_disable_module_target_binding_hash: Option<[u8; 32]>,
    pub(crate) retained_rollback_apply_authorization_event_id: Option<&'a str>,
    pub(crate) disable_module_target_id: Option<&'a str>,
    pub(crate) disable_module_target_projection_hash: Option<[u8; 32]>,
    pub(crate) restart_last_good_target_binding_hash: Option<[u8; 32]>,
    pub(crate) expected_restart_last_good_target_binding_hash: Option<[u8; 32]>,
    pub(crate) retained_disable_module_target_binding_event_id: Option<&'a str>,
    pub(crate) restart_last_good_target_id: Option<&'a str>,
    pub(crate) restart_last_good_target_projection_hash: Option<[u8; 32]>,
    pub(crate) load_artifact_by_hash_target_binding_hash: Option<[u8; 32]>,
    pub(crate) expected_load_artifact_by_hash_target_binding_hash: Option<[u8; 32]>,
    pub(crate) retained_restart_last_good_target_binding_event_id: Option<&'a str>,
    pub(crate) load_artifact_by_hash_target_id: Option<&'a str>,
    pub(crate) load_artifact_by_hash_target_artifact_hash: Option<[u8; 32]>,
    pub(crate) load_artifact_by_hash_target_projection_hash: Option<[u8; 32]>,
    pub(crate) recovery_memory_write_authority_hash: Option<[u8; 32]>,
    pub(crate) expected_recovery_memory_write_authority_hash: Option<[u8; 32]>,
    pub(crate) retained_load_artifact_by_hash_target_binding_event_id: Option<&'a str>,
    pub(crate) recovery_memory_write_authority_id: Option<&'a str>,
    pub(crate) recovery_memory_projection_hash: Option<[u8; 32]>,
    pub(crate) durable_audit_rollback_write_authority_hash: Option<[u8; 32]>,
    pub(crate) expected_durable_audit_rollback_write_authority_hash: Option<[u8; 32]>,
    pub(crate) retained_recovery_memory_write_authority_event_id: Option<&'a str>,
    pub(crate) durable_audit_rollback_write_authority_id: Option<&'a str>,
    pub(crate) durable_audit_rollback_projection_hash: Option<[u8; 32]>,
    pub(crate) service_inventory_side_effect_boundary_hash: Option<[u8; 32]>,
    pub(crate) expected_service_inventory_side_effect_boundary_hash: Option<[u8; 32]>,
    pub(crate) retained_durable_audit_rollback_write_authority_event_id: Option<&'a str>,
    pub(crate) service_inventory_side_effect_boundary_id: Option<&'a str>,
    pub(crate) service_inventory_projection_hash: Option<[u8; 32]>,
    pub(crate) command_dispatch_behavior_hash: Option<[u8; 32]>,
    pub(crate) expected_command_dispatch_behavior_hash: Option<[u8; 32]>,
    pub(crate) retained_service_inventory_side_effect_boundary_event_id: Option<&'a str>,
    pub(crate) command_dispatch_behavior_id: Option<&'a str>,
    pub(crate) command_dispatch_behavior_projection_hash: Option<[u8; 32]>,
    pub(crate) executor_capability_table_hash: Option<[u8; 32]>,
    pub(crate) expected_executor_capability_table_hash: Option<[u8; 32]>,
    pub(crate) retained_command_dispatch_behavior_event_id: Option<&'a str>,
    pub(crate) executor_capability_table_id: Option<&'a str>,
    pub(crate) executor_capability_projection_hash: Option<[u8; 32]>,
    pub(crate) side_effect_gate_hash: Option<[u8; 32]>,
    pub(crate) expected_side_effect_gate_hash: Option<[u8; 32]>,
    pub(crate) retained_executor_capability_table_event_id: Option<&'a str>,
    pub(crate) side_effect_gate_id: Option<&'a str>,
    pub(crate) side_effect_projection_hash: Option<[u8; 32]>,
    pub(crate) source_rollback_apply_denial_hash: Option<[u8; 32]>,
    pub(crate) source_durable_policy_write_authority_decision_hash: Option<[u8; 32]>,
    pub(crate) source_recovery_rollback_inspect_source_reference_hash: Option<[u8; 32]>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) retained_loader_reference_event_id: Option<&'a str>,
    pub(crate) retained_rollback_evidence_reference_event_id: Option<&'a str>,
    pub(crate) lifeline_status_admission_present: bool,
    pub(crate) rollback_preview_admission_present: bool,
    pub(crate) rollback_apply_admission_present: bool,
    pub(crate) disable_module_admission_present: bool,
    pub(crate) restart_last_good_admission_present: bool,
    pub(crate) load_recovery_artifact_by_hash_admission_present: bool,
    pub(crate) execution_stage_hash: Option<[u8; 32]>,
    pub(crate) expected_execution_stage_hash: Option<[u8; 32]>,
    pub(crate) retained_previous_stage_event_id: Option<&'a str>,
    pub(crate) execution_enablement_hash: Option<[u8; 32]>,
    pub(crate) execution_preflight_hash: Option<[u8; 32]>,
    pub(crate) execution_intent_hash: Option<[u8; 32]>,
    pub(crate) execution_commit_gate_hash: Option<[u8; 32]>,
    pub(crate) execution_result_denial_hash: Option<[u8; 32]>,
    pub(crate) execution_audit_denial_hash: Option<[u8; 32]>,
    pub(crate) execution_observation_denial_hash: Option<[u8; 32]>,
    pub(crate) execution_stage_id: Option<&'a str>,
    pub(crate) execution_stage_projection_hash: Option<[u8; 32]>,
    pub(crate) normalized_spec: Option<RecoveryLifelineCommandSpec>,
    pub(crate) target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

impl<'a> CommandBindings<'a> {
    pub(crate) const fn empty() -> Self {
        Self {
            stage: StageBinding::empty(),
            has_reference: false,
            arity_valid: false,
            scope: "current_boot",
            command_envelope_reference_hash: None,
            expected_command_envelope_reference_hash: None,
            retained_lifeline_request_event_id: None,
            command_id: None,
            argument_schema: None,
            argument_hash: None,
            required_capability: None,
            target_locator: None,
            command_admission_boundary_id: None,
            lifeline_request_reference_hash: None,
            command_body_canonicalization_hash: None,
            expected_command_body_canonicalization_hash: None,
            retained_command_envelope_reference_event_id: None,
            command_dispatch_boundary_id: None,
            handler_binding_hash: None,
            expected_handler_binding_hash: None,
            retained_command_body_canonicalization_event_id: None,
            handler_id: None,
            handler_input_binding_hash: None,
            status_read_handler_hash: None,
            expected_status_read_handler_hash: None,
            retained_command_handler_binding_event_id: None,
            status_handler_id: None,
            status_read_projection_hash: None,
            rollback_preview_authorization_hash: None,
            expected_rollback_preview_authorization_hash: None,
            retained_status_read_handler_event_id: None,
            rollback_preview_authorization_id: None,
            rollback_preview_projection_hash: None,
            rollback_apply_authorization_hash: None,
            expected_rollback_apply_authorization_hash: None,
            retained_rollback_preview_authorization_event_id: None,
            rollback_apply_authorization_id: None,
            rollback_apply_projection_hash: None,
            disable_module_target_binding_hash: None,
            expected_disable_module_target_binding_hash: None,
            retained_rollback_apply_authorization_event_id: None,
            disable_module_target_id: None,
            disable_module_target_projection_hash: None,
            restart_last_good_target_binding_hash: None,
            expected_restart_last_good_target_binding_hash: None,
            retained_disable_module_target_binding_event_id: None,
            restart_last_good_target_id: None,
            restart_last_good_target_projection_hash: None,
            load_artifact_by_hash_target_binding_hash: None,
            expected_load_artifact_by_hash_target_binding_hash: None,
            retained_restart_last_good_target_binding_event_id: None,
            load_artifact_by_hash_target_id: None,
            load_artifact_by_hash_target_artifact_hash: None,
            load_artifact_by_hash_target_projection_hash: None,
            recovery_memory_write_authority_hash: None,
            expected_recovery_memory_write_authority_hash: None,
            retained_load_artifact_by_hash_target_binding_event_id: None,
            recovery_memory_write_authority_id: None,
            recovery_memory_projection_hash: None,
            durable_audit_rollback_write_authority_hash: None,
            expected_durable_audit_rollback_write_authority_hash: None,
            retained_recovery_memory_write_authority_event_id: None,
            durable_audit_rollback_write_authority_id: None,
            durable_audit_rollback_projection_hash: None,
            service_inventory_side_effect_boundary_hash: None,
            expected_service_inventory_side_effect_boundary_hash: None,
            retained_durable_audit_rollback_write_authority_event_id: None,
            service_inventory_side_effect_boundary_id: None,
            service_inventory_projection_hash: None,
            command_dispatch_behavior_hash: None,
            expected_command_dispatch_behavior_hash: None,
            retained_service_inventory_side_effect_boundary_event_id: None,
            command_dispatch_behavior_id: None,
            command_dispatch_behavior_projection_hash: None,
            executor_capability_table_hash: None,
            expected_executor_capability_table_hash: None,
            retained_command_dispatch_behavior_event_id: None,
            executor_capability_table_id: None,
            executor_capability_projection_hash: None,
            side_effect_gate_hash: None,
            expected_side_effect_gate_hash: None,
            retained_executor_capability_table_event_id: None,
            side_effect_gate_id: None,
            side_effect_projection_hash: None,
            source_rollback_apply_denial_hash: None,
            source_durable_policy_write_authority_decision_hash: None,
            source_recovery_rollback_inspect_source_reference_hash: None,
            identity_reference_hash: None,
            trust_reference_hash: None,
            vm_test_reference_hash: None,
            local_approval_reference_hash: None,
            loader_reference_hash: None,
            rollback_evidence_reference_hash: None,
            artifact_hash: None,
            trust_hash: None,
            vm_test_hash: None,
            local_approval_hash: None,
            loader_hash: None,
            rollback_evidence_hash: None,
            retained_identity_reference_event_id: None,
            retained_trust_reference_event_id: None,
            retained_vm_test_reference_event_id: None,
            retained_local_approval_reference_event_id: None,
            retained_loader_reference_event_id: None,
            retained_rollback_evidence_reference_event_id: None,
            lifeline_status_admission_present: false,
            rollback_preview_admission_present: false,
            rollback_apply_admission_present: false,
            disable_module_admission_present: false,
            restart_last_good_admission_present: false,
            load_recovery_artifact_by_hash_admission_present: false,
            execution_stage_hash: None,
            expected_execution_stage_hash: None,
            retained_previous_stage_event_id: None,
            execution_enablement_hash: None,
            execution_preflight_hash: None,
            execution_intent_hash: None,
            execution_commit_gate_hash: None,
            execution_result_denial_hash: None,
            execution_audit_denial_hash: None,
            execution_observation_denial_hash: None,
            execution_stage_id: None,
            execution_stage_projection_hash: None,
            normalized_spec: None,
            target_locator_value: None,
            status: "",
            reason: "",
            valid: false,
        }
    }

    pub(crate) fn with_reference_check(
        mut self,
        stage_hash_field: CommandReferenceField,
        normalized_spec: Option<RecoveryLifelineCommandSpec>,
        target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
        expected_hash: Option<[u8; 32]>,
        status: &'static str,
        reason: &'static str,
        valid: bool,
    ) -> Self {
        self.stage.expected_hash = expected_hash;
        self.set_expected_hash(stage_hash_field, expected_hash);
        self.normalized_spec = normalized_spec;
        self.target_locator_value = target_locator_value;
        self.status = status;
        self.reason = reason;
        self.valid = valid;
        self
    }

    fn set_expected_hash(&mut self, field: CommandReferenceField, expected: Option<[u8; 32]>) {
        match field {
            CommandReferenceField::CommandEnvelopeReferenceHash => {
                self.expected_command_envelope_reference_hash = expected
            }
            CommandReferenceField::CommandBodyCanonicalizationHash => {
                self.expected_command_body_canonicalization_hash = expected
            }
            CommandReferenceField::HandlerBindingHash => {
                self.expected_handler_binding_hash = expected
            }
            CommandReferenceField::StatusReadHandlerHash => {
                self.expected_status_read_handler_hash = expected
            }
            CommandReferenceField::RollbackPreviewAuthorizationHash => {
                self.expected_rollback_preview_authorization_hash = expected
            }
            CommandReferenceField::RollbackApplyAuthorizationHash => {
                self.expected_rollback_apply_authorization_hash = expected
            }
            CommandReferenceField::DisableModuleTargetBindingHash => {
                self.expected_disable_module_target_binding_hash = expected
            }
            CommandReferenceField::RestartLastGoodTargetBindingHash => {
                self.expected_restart_last_good_target_binding_hash = expected
            }
            CommandReferenceField::LoadArtifactByHashTargetBindingHash => {
                self.expected_load_artifact_by_hash_target_binding_hash = expected
            }
            CommandReferenceField::RecoveryMemoryWriteAuthorityHash => {
                self.expected_recovery_memory_write_authority_hash = expected
            }
            CommandReferenceField::DurableAuditRollbackWriteAuthorityHash => {
                self.expected_durable_audit_rollback_write_authority_hash = expected
            }
            CommandReferenceField::ServiceInventorySideEffectBoundaryHash => {
                self.expected_service_inventory_side_effect_boundary_hash = expected
            }
            CommandReferenceField::CommandDispatchBehaviorHash => {
                self.expected_command_dispatch_behavior_hash = expected
            }
            CommandReferenceField::ExecutorCapabilityTableHash => {
                self.expected_executor_capability_table_hash = expected
            }
            CommandReferenceField::SideEffectGateHash => {
                self.expected_side_effect_gate_hash = expected
            }
            CommandReferenceField::ExecutionStageHash => {
                self.expected_execution_stage_hash = expected
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CommandReferenceField {
    CommandEnvelopeReferenceHash,
    RetainedLifelineRequestEventId,
    CommandId,
    ArgumentSchema,
    ArgumentHash,
    RequiredCapability,
    TargetLocator,
    CommandAdmissionBoundaryId,
    LifelineRequestReferenceHash,
    CommandBodyCanonicalizationHash,
    RetainedCommandEnvelopeReferenceEventId,
    CommandDispatchBoundaryId,
    HandlerBindingHash,
    RetainedCommandBodyCanonicalizationEventId,
    HandlerId,
    HandlerInputBindingHash,
    StatusReadHandlerHash,
    RetainedCommandHandlerBindingEventId,
    StatusHandlerId,
    StatusReadProjectionHash,
    RollbackPreviewAuthorizationHash,
    RetainedStatusReadHandlerEventId,
    RollbackPreviewAuthorizationId,
    RollbackPreviewProjectionHash,
    RollbackApplyAuthorizationHash,
    RetainedRollbackPreviewAuthorizationEventId,
    RollbackApplyAuthorizationId,
    RollbackApplyProjectionHash,
    DisableModuleTargetBindingHash,
    RetainedRollbackApplyAuthorizationEventId,
    DisableModuleTargetId,
    DisableModuleTargetProjectionHash,
    RestartLastGoodTargetBindingHash,
    RetainedDisableModuleTargetBindingEventId,
    RestartLastGoodTargetId,
    RestartLastGoodTargetProjectionHash,
    LoadArtifactByHashTargetBindingHash,
    RetainedRestartLastGoodTargetBindingEventId,
    LoadArtifactByHashTargetId,
    LoadArtifactByHashTargetArtifactHash,
    LoadArtifactByHashTargetProjectionHash,
    RecoveryMemoryWriteAuthorityHash,
    RetainedLoadArtifactByHashTargetBindingEventId,
    RecoveryMemoryWriteAuthorityId,
    RecoveryMemoryProjectionHash,
    DurableAuditRollbackWriteAuthorityHash,
    RetainedRecoveryMemoryWriteAuthorityEventId,
    DurableAuditRollbackWriteAuthorityId,
    DurableAuditRollbackProjectionHash,
    ServiceInventorySideEffectBoundaryHash,
    RetainedDurableAuditRollbackWriteAuthorityEventId,
    ServiceInventorySideEffectBoundaryId,
    ServiceInventoryProjectionHash,
    CommandDispatchBehaviorHash,
    RetainedServiceInventorySideEffectBoundaryEventId,
    CommandDispatchBehaviorId,
    CommandDispatchBehaviorProjectionHash,
    ExecutorCapabilityTableHash,
    RetainedCommandDispatchBehaviorEventId,
    ExecutorCapabilityTableId,
    ExecutorCapabilityProjectionHash,
    SideEffectGateHash,
    RetainedExecutorCapabilityTableEventId,
    SideEffectGateId,
    SideEffectProjectionHash,
    SourceRollbackApplyDenialHash,
    SourceDurablePolicyWriteAuthorityDecisionHash,
    SourceRecoveryRollbackInspectSourceReferenceHash,
    IdentityReferenceHash,
    TrustReferenceHash,
    VmTestReferenceHash,
    LocalApprovalReferenceHash,
    LoaderReferenceHash,
    RollbackEvidenceReferenceHash,
    ArtifactHash,
    TrustHash,
    VmTestHash,
    LocalApprovalHash,
    LoaderHash,
    RollbackEvidenceHash,
    RetainedIdentityReferenceEventId,
    RetainedTrustReferenceEventId,
    RetainedVmTestReferenceEventId,
    RetainedLocalApprovalReferenceEventId,
    RetainedLoaderReferenceEventId,
    RetainedRollbackEvidenceReferenceEventId,
    ExecutionStageHash,
    RetainedPreviousStageEventId,
    ExecutionEnablementHash,
    ExecutionPreflightHash,
    ExecutionIntentHash,
    ExecutionCommitGateHash,
    ExecutionResultDenialHash,
    ExecutionAuditDenialHash,
    ExecutionObservationDenialHash,
    ExecutionStageId,
    ExecutionStageProjectionHash,
}

pub(crate) fn parse_command_reference_args<'a>(
    arg: &'a str,
    fields: &[CommandReferenceField],
) -> CommandBindings<'a> {
    let mut parts = arg.split_whitespace();
    let mut input = CommandBindings::empty();
    let mut all_required_present = true;
    for (index, field) in fields.iter().copied().enumerate() {
        let value = parts.next();
        if index == 0 {
            input.has_reference = value.is_some();
        }
        if value.is_none() {
            all_required_present = false;
        }
        set_command_reference_field(&mut input, field, value);
        if index == 0 {
            input.stage.stage_hash = input_hash_for_field(input, field);
        } else if index == 1 {
            input.stage.retained_previous_event_id = value;
        }
    }
    input.scope = parts.next().unwrap_or("current_boot");
    input.arity_valid = input.has_reference && all_required_present && parts.next().is_none();
    input
}

fn input_hash_for_field(
    input: CommandBindings<'_>,
    field: CommandReferenceField,
) -> Option<[u8; 32]> {
    match field {
        CommandReferenceField::CommandEnvelopeReferenceHash => {
            input.command_envelope_reference_hash
        }
        CommandReferenceField::CommandBodyCanonicalizationHash => {
            input.command_body_canonicalization_hash
        }
        CommandReferenceField::HandlerBindingHash => input.handler_binding_hash,
        CommandReferenceField::StatusReadHandlerHash => input.status_read_handler_hash,
        CommandReferenceField::RollbackPreviewAuthorizationHash => {
            input.rollback_preview_authorization_hash
        }
        CommandReferenceField::RollbackApplyAuthorizationHash => {
            input.rollback_apply_authorization_hash
        }
        CommandReferenceField::DisableModuleTargetBindingHash => {
            input.disable_module_target_binding_hash
        }
        CommandReferenceField::RestartLastGoodTargetBindingHash => {
            input.restart_last_good_target_binding_hash
        }
        CommandReferenceField::LoadArtifactByHashTargetBindingHash => {
            input.load_artifact_by_hash_target_binding_hash
        }
        CommandReferenceField::RecoveryMemoryWriteAuthorityHash => {
            input.recovery_memory_write_authority_hash
        }
        CommandReferenceField::DurableAuditRollbackWriteAuthorityHash => {
            input.durable_audit_rollback_write_authority_hash
        }
        CommandReferenceField::ServiceInventorySideEffectBoundaryHash => {
            input.service_inventory_side_effect_boundary_hash
        }
        CommandReferenceField::CommandDispatchBehaviorHash => input.command_dispatch_behavior_hash,
        CommandReferenceField::ExecutorCapabilityTableHash => input.executor_capability_table_hash,
        CommandReferenceField::SideEffectGateHash => input.side_effect_gate_hash,
        CommandReferenceField::IdentityReferenceHash => input.identity_reference_hash,
        CommandReferenceField::TrustReferenceHash => input.trust_reference_hash,
        CommandReferenceField::VmTestReferenceHash => input.vm_test_reference_hash,
        CommandReferenceField::LocalApprovalReferenceHash => input.local_approval_reference_hash,
        CommandReferenceField::LoaderReferenceHash => input.loader_reference_hash,
        CommandReferenceField::RollbackEvidenceReferenceHash => {
            input.rollback_evidence_reference_hash
        }
        CommandReferenceField::ExecutionStageHash => input.execution_stage_hash,
        _ => None,
    }
}

fn set_command_reference_field<'a>(
    input: &mut CommandBindings<'a>,
    field: CommandReferenceField,
    value: Option<&'a str>,
) {
    match field {
        CommandReferenceField::CommandEnvelopeReferenceHash => {
            input.command_envelope_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedLifelineRequestEventId => {
            input.retained_lifeline_request_event_id = value
        }
        CommandReferenceField::CommandId => input.command_id = value,
        CommandReferenceField::ArgumentSchema => input.argument_schema = value,
        CommandReferenceField::ArgumentHash => {
            input.argument_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RequiredCapability => input.required_capability = value,
        CommandReferenceField::TargetLocator => input.target_locator = value,
        CommandReferenceField::CommandAdmissionBoundaryId => {
            input.command_admission_boundary_id = value
        }
        CommandReferenceField::LifelineRequestReferenceHash => {
            input.lifeline_request_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::CommandBodyCanonicalizationHash => {
            input.command_body_canonicalization_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedCommandEnvelopeReferenceEventId => {
            input.retained_command_envelope_reference_event_id = value
        }
        CommandReferenceField::CommandDispatchBoundaryId => {
            input.command_dispatch_boundary_id = value
        }
        CommandReferenceField::HandlerBindingHash => {
            input.handler_binding_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedCommandBodyCanonicalizationEventId => {
            input.retained_command_body_canonicalization_event_id = value
        }
        CommandReferenceField::HandlerId => input.handler_id = value,
        CommandReferenceField::HandlerInputBindingHash => {
            input.handler_input_binding_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::StatusReadHandlerHash => {
            input.status_read_handler_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedCommandHandlerBindingEventId => {
            input.retained_command_handler_binding_event_id = value
        }
        CommandReferenceField::StatusHandlerId => input.status_handler_id = value,
        CommandReferenceField::StatusReadProjectionHash => {
            input.status_read_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RollbackPreviewAuthorizationHash => {
            input.rollback_preview_authorization_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedStatusReadHandlerEventId => {
            input.retained_status_read_handler_event_id = value
        }
        CommandReferenceField::RollbackPreviewAuthorizationId => {
            input.rollback_preview_authorization_id = value
        }
        CommandReferenceField::RollbackPreviewProjectionHash => {
            input.rollback_preview_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RollbackApplyAuthorizationHash => {
            input.rollback_apply_authorization_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedRollbackPreviewAuthorizationEventId => {
            input.retained_rollback_preview_authorization_event_id = value
        }
        CommandReferenceField::RollbackApplyAuthorizationId => {
            input.rollback_apply_authorization_id = value
        }
        CommandReferenceField::RollbackApplyProjectionHash => {
            input.rollback_apply_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::DisableModuleTargetBindingHash => {
            input.disable_module_target_binding_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedRollbackApplyAuthorizationEventId => {
            input.retained_rollback_apply_authorization_event_id = value
        }
        CommandReferenceField::DisableModuleTargetId => input.disable_module_target_id = value,
        CommandReferenceField::DisableModuleTargetProjectionHash => {
            input.disable_module_target_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RestartLastGoodTargetBindingHash => {
            input.restart_last_good_target_binding_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedDisableModuleTargetBindingEventId => {
            input.retained_disable_module_target_binding_event_id = value
        }
        CommandReferenceField::RestartLastGoodTargetId => input.restart_last_good_target_id = value,
        CommandReferenceField::RestartLastGoodTargetProjectionHash => {
            input.restart_last_good_target_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::LoadArtifactByHashTargetBindingHash => {
            input.load_artifact_by_hash_target_binding_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedRestartLastGoodTargetBindingEventId => {
            input.retained_restart_last_good_target_binding_event_id = value
        }
        CommandReferenceField::LoadArtifactByHashTargetId => {
            input.load_artifact_by_hash_target_id = value
        }
        CommandReferenceField::LoadArtifactByHashTargetArtifactHash => {
            input.load_artifact_by_hash_target_artifact_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::LoadArtifactByHashTargetProjectionHash => {
            input.load_artifact_by_hash_target_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RecoveryMemoryWriteAuthorityHash => {
            input.recovery_memory_write_authority_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedLoadArtifactByHashTargetBindingEventId => {
            input.retained_load_artifact_by_hash_target_binding_event_id = value
        }
        CommandReferenceField::RecoveryMemoryWriteAuthorityId => {
            input.recovery_memory_write_authority_id = value
        }
        CommandReferenceField::RecoveryMemoryProjectionHash => {
            input.recovery_memory_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::DurableAuditRollbackWriteAuthorityHash => {
            input.durable_audit_rollback_write_authority_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedRecoveryMemoryWriteAuthorityEventId => {
            input.retained_recovery_memory_write_authority_event_id = value
        }
        CommandReferenceField::DurableAuditRollbackWriteAuthorityId => {
            input.durable_audit_rollback_write_authority_id = value
        }
        CommandReferenceField::DurableAuditRollbackProjectionHash => {
            input.durable_audit_rollback_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ServiceInventorySideEffectBoundaryHash => {
            input.service_inventory_side_effect_boundary_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedDurableAuditRollbackWriteAuthorityEventId => {
            input.retained_durable_audit_rollback_write_authority_event_id = value
        }
        CommandReferenceField::ServiceInventorySideEffectBoundaryId => {
            input.service_inventory_side_effect_boundary_id = value
        }
        CommandReferenceField::ServiceInventoryProjectionHash => {
            input.service_inventory_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::CommandDispatchBehaviorHash => {
            input.command_dispatch_behavior_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedServiceInventorySideEffectBoundaryEventId => {
            input.retained_service_inventory_side_effect_boundary_event_id = value
        }
        CommandReferenceField::CommandDispatchBehaviorId => {
            input.command_dispatch_behavior_id = value
        }
        CommandReferenceField::CommandDispatchBehaviorProjectionHash => {
            input.command_dispatch_behavior_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutorCapabilityTableHash => {
            input.executor_capability_table_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedCommandDispatchBehaviorEventId => {
            input.retained_command_dispatch_behavior_event_id = value
        }
        CommandReferenceField::ExecutorCapabilityTableId => {
            input.executor_capability_table_id = value
        }
        CommandReferenceField::ExecutorCapabilityProjectionHash => {
            input.executor_capability_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::SideEffectGateHash => {
            input.side_effect_gate_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedExecutorCapabilityTableEventId => {
            input.retained_executor_capability_table_event_id = value
        }
        CommandReferenceField::SideEffectGateId => input.side_effect_gate_id = value,
        CommandReferenceField::SideEffectProjectionHash => {
            input.side_effect_projection_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::SourceRollbackApplyDenialHash => {
            input.source_rollback_apply_denial_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::SourceDurablePolicyWriteAuthorityDecisionHash => {
            input.source_durable_policy_write_authority_decision_hash =
                value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::SourceRecoveryRollbackInspectSourceReferenceHash => {
            input.source_recovery_rollback_inspect_source_reference_hash =
                value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::IdentityReferenceHash => {
            input.identity_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::TrustReferenceHash => {
            input.trust_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::VmTestReferenceHash => {
            input.vm_test_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::LocalApprovalReferenceHash => {
            input.local_approval_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::LoaderReferenceHash => {
            input.loader_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RollbackEvidenceReferenceHash => {
            input.rollback_evidence_reference_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ArtifactHash => {
            input.artifact_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::TrustHash => input.trust_hash = value.and_then(parse_sha256_ref),
        CommandReferenceField::VmTestHash => input.vm_test_hash = value.and_then(parse_sha256_ref),
        CommandReferenceField::LocalApprovalHash => {
            input.local_approval_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::LoaderHash => input.loader_hash = value.and_then(parse_sha256_ref),
        CommandReferenceField::RollbackEvidenceHash => {
            input.rollback_evidence_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedIdentityReferenceEventId => {
            input.retained_identity_reference_event_id = value
        }
        CommandReferenceField::RetainedTrustReferenceEventId => {
            input.retained_trust_reference_event_id = value
        }
        CommandReferenceField::RetainedVmTestReferenceEventId => {
            input.retained_vm_test_reference_event_id = value
        }
        CommandReferenceField::RetainedLocalApprovalReferenceEventId => {
            input.retained_local_approval_reference_event_id = value
        }
        CommandReferenceField::RetainedLoaderReferenceEventId => {
            input.retained_loader_reference_event_id = value
        }
        CommandReferenceField::RetainedRollbackEvidenceReferenceEventId => {
            input.retained_rollback_evidence_reference_event_id = value
        }
        CommandReferenceField::ExecutionStageHash => {
            input.execution_stage_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::RetainedPreviousStageEventId => {
            input.retained_previous_stage_event_id = value
        }
        CommandReferenceField::ExecutionEnablementHash => {
            input.execution_enablement_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionPreflightHash => {
            input.execution_preflight_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionIntentHash => {
            input.execution_intent_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionCommitGateHash => {
            input.execution_commit_gate_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionResultDenialHash => {
            input.execution_result_denial_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionAuditDenialHash => {
            input.execution_audit_denial_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionObservationDenialHash => {
            input.execution_observation_denial_hash = value.and_then(parse_sha256_ref)
        }
        CommandReferenceField::ExecutionStageId => input.execution_stage_id = value,
        CommandReferenceField::ExecutionStageProjectionHash => {
            input.execution_stage_projection_hash = value.and_then(parse_sha256_ref)
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoaderRuntimeIsolationCandidate {
    pub(crate) command_candidate: RecoveryLifelineCommandVocabularyCandidate,
    pub(crate) command_vocabulary_available: bool,
    pub(crate) command_vocabulary_current_boot: bool,
    pub(crate) command_vocabulary_schema_ok: bool,
    pub(crate) command_vocabulary_binding_ok: bool,
    pub(crate) command_vocabulary_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) loader_address_space_boundary_present: bool,
    pub(crate) loader_entrypoint_abi_present: bool,
    pub(crate) loader_memory_map_constraints_present: bool,
    pub(crate) loader_capability_import_table_present: bool,
    pub(crate) loader_artifact_hash_binding_present: bool,
    pub(crate) loader_provider_separation_present: bool,
    pub(crate) loader_normal_module_separation_present: bool,
    pub(crate) rollback_transaction_engine_present: bool,
    pub(crate) durable_audit_rollback_persistence_present: bool,
    pub(crate) recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoaderRuntimeIsolationCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) command_vocabulary_envelope_exposed: bool,
    pub(crate) command_vocabulary_accepted: bool,
    pub(crate) isolation_requirements_exposed: bool,
    pub(crate) loader_runtime_isolation_ready: bool,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryLoaderRuntimeIsolationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryRollbackTransactionEngineCandidate {
    pub(crate) loader_candidate: RecoveryLoaderRuntimeIsolationCandidate,
    pub(crate) loader_runtime_isolation_available: bool,
    pub(crate) loader_runtime_isolation_current_boot: bool,
    pub(crate) loader_runtime_isolation_schema_ok: bool,
    pub(crate) loader_runtime_isolation_binding_ok: bool,
    pub(crate) loader_runtime_isolation_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) rollback_target_selection_present: bool,
    pub(crate) rollback_transaction_id_provenance_present: bool,
    pub(crate) rollback_last_good_binding_present: bool,
    pub(crate) rollback_disabled_module_set_binding_present: bool,
    pub(crate) rollback_artifact_hash_binding_present: bool,
    pub(crate) rollback_replay_preconditions_present: bool,
    pub(crate) rollback_recovery_capability_import_present: bool,
    pub(crate) rollback_atomic_apply_abort_semantics_present: bool,
    pub(crate) durable_audit_rollback_persistence_present: bool,
    pub(crate) recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryRollbackTransactionEngineCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) command_vocabulary_envelope_exposed: bool,
    pub(crate) command_vocabulary_accepted: bool,
    pub(crate) loader_runtime_isolation_boundary_exposed: bool,
    pub(crate) loader_runtime_isolation_accepted: bool,
    pub(crate) transaction_requirements_exposed: bool,
    pub(crate) rollback_transaction_engine_ready: bool,
    pub(crate) rollback_preview_enabled: bool,
    pub(crate) rollback_apply_enabled: bool,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryRollbackTransactionEngineSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryDurableAuditRollbackPersistenceCandidate {
    pub(crate) transaction_candidate: RecoveryRollbackTransactionEngineCandidate,
    pub(crate) rollback_transaction_engine_available: bool,
    pub(crate) rollback_transaction_engine_current_boot: bool,
    pub(crate) rollback_transaction_engine_schema_ok: bool,
    pub(crate) rollback_transaction_engine_binding_ok: bool,
    pub(crate) rollback_transaction_engine_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) persistence_device_inventory_present: bool,
    pub(crate) storage_layout_identity_present: bool,
    pub(crate) audit_append_log_identity_present: bool,
    pub(crate) rollback_store_identity_present: bool,
    pub(crate) transaction_replay_cursor_present: bool,
    pub(crate) last_good_checkpoint_binding_present: bool,
    pub(crate) write_ordering_present: bool,
    pub(crate) crash_consistency_present: bool,
    pub(crate) integrity_root_hash_chain_present: bool,
    pub(crate) recovery_memory_provenance_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryDurableAuditRollbackPersistenceCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) command_vocabulary_envelope_exposed: bool,
    pub(crate) command_vocabulary_accepted: bool,
    pub(crate) loader_runtime_isolation_boundary_exposed: bool,
    pub(crate) loader_runtime_isolation_accepted: bool,
    pub(crate) rollback_transaction_engine_boundary_exposed: bool,
    pub(crate) rollback_transaction_engine_accepted: bool,
    pub(crate) persistence_requirements_exposed: bool,
    pub(crate) durable_audit_rollback_persistence_ready: bool,
    pub(crate) durable_writes_enabled: bool,
    pub(crate) rollback_replay_enabled: bool,
    pub(crate) recovery_memory_writes_enabled: bool,
    pub(crate) rollback_preview_enabled: bool,
    pub(crate) rollback_apply_enabled: bool,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryDurableAuditRollbackPersistenceSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryMemoryProvenanceCandidate {
    pub(crate) persistence_candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
    pub(crate) durable_audit_rollback_persistence_available: bool,
    pub(crate) durable_audit_rollback_persistence_current_boot: bool,
    pub(crate) durable_audit_rollback_persistence_schema_ok: bool,
    pub(crate) durable_audit_rollback_persistence_binding_ok: bool,
    pub(crate) durable_audit_rollback_persistence_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) source_record_ids_present: bool,
    pub(crate) source_schema_hashes_present: bool,
    pub(crate) memory_classification_present: bool,
    pub(crate) memory_authority_level_present: bool,
    pub(crate) memory_rollback_transaction_binding_present: bool,
    pub(crate) memory_last_good_checkpoint_binding_present: bool,
    pub(crate) recovery_only_export_profile_present: bool,
    pub(crate) memory_redaction_state_present: bool,
    pub(crate) memory_replay_window_present: bool,
    pub(crate) memory_audit_linkage_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryMemoryProvenanceCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) request_chain_valid: bool,
    pub(crate) command_vocabulary_envelope_exposed: bool,
    pub(crate) command_vocabulary_accepted: bool,
    pub(crate) loader_runtime_isolation_boundary_exposed: bool,
    pub(crate) loader_runtime_isolation_accepted: bool,
    pub(crate) rollback_transaction_engine_boundary_exposed: bool,
    pub(crate) rollback_transaction_engine_accepted: bool,
    pub(crate) durable_audit_rollback_persistence_boundary_exposed: bool,
    pub(crate) durable_audit_rollback_persistence_accepted: bool,
    pub(crate) memory_provenance_requirements_exposed: bool,
    pub(crate) recovery_memory_provenance_ready: bool,
    pub(crate) memory_writes_enabled: bool,
    pub(crate) provider_export_enabled: bool,
    pub(crate) durable_writes_enabled: bool,
    pub(crate) rollback_replay_enabled: bool,
    pub(crate) recovery_memory_writes_enabled: bool,
    pub(crate) rollback_preview_enabled: bool,
    pub(crate) rollback_apply_enabled: bool,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryMemoryProvenanceSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandAdmissionCandidate {
    pub(crate) memory_candidate: RecoveryMemoryProvenanceCandidate,
    pub(crate) recovery_memory_provenance_available: bool,
    pub(crate) recovery_memory_provenance_current_boot: bool,
    pub(crate) recovery_memory_provenance_schema_ok: bool,
    pub(crate) recovery_memory_provenance_binding_ok: bool,
    pub(crate) recovery_memory_provenance_binding_reason: &'static str,
    pub(crate) direct_openai_recovery_shortcut_used: bool,
    pub(crate) bindings: CommandBindings<'static>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandAdmissionCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) memory_check: RecoveryMemoryProvenanceCheck,
    pub(crate) recovery_memory_provenance_boundary_exposed: bool,
    pub(crate) recovery_memory_provenance_accepted: bool,
    pub(crate) command_admission_requirements_exposed: bool,
    pub(crate) command_admission_ready: bool,
    pub(crate) bindings: CommandBindings<'static>,
    pub(crate) command_execution_enabled: bool,
    pub(crate) accepts_lifeline_command_envelope: bool,
    pub(crate) dispatches_lifeline_command: bool,
    pub(crate) authorizes_recovery_load: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_loader: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

impl core::ops::Deref for RecoveryLifelineCommandAdmissionCandidate {
    type Target = CommandBindings<'static>;

    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl core::ops::DerefMut for RecoveryLifelineCommandAdmissionCandidate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}

impl core::ops::Deref for RecoveryLifelineCommandAdmissionCheck {
    type Target = CommandBindings<'static>;

    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

pub(crate) struct RecoveryLifelineCommandAdmissionSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
