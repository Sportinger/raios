use crate::{
    agent_protocol_recovery_command_dispatch_types::*,
    agent_protocol_recovery_constants::*,
    agent_protocol_recovery_execution::{
        recovery_lifeline_command_execution_stage_chain_presence_from_retained,
        RecoveryLifelineCommandExecutionStageRetainedChain,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE,
    },
    agent_protocol_recovery_lifeline::RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    agent_protocol_recovery_lifeline_eval::{
        apply_command_admission_case, evaluate_recovery_lifeline_command_admission,
        recovery_lifeline_command_admission_valid_candidate, CommandAdmissionMutation,
    },
    agent_protocol_recovery_runtime_types::RecoveryLifelineCommandAdmissionCheck,
    agent_protocol_support::{method_eq, run_selftest_cases_with, CaseSpec},
    event_log,
};

pub(crate) fn evaluate_recovery_lifeline_command_envelope(
    candidate: RecoveryLifelineCommandEnvelopeCandidate,
) -> RecoveryLifelineCommandEnvelopeCheck {
    let admission_check =
        evaluate_recovery_lifeline_command_admission(candidate.admission_candidate);
    let admission_boundary_exposed = admission_check.command_admission_requirements_exposed
        || candidate.command_admission_available;
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            admission_check,
            admission_boundary_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !admission_check.command_admission_ready {
        return recovery_lifeline_command_envelope_check(
            admission_check.status,
            admission_check.reason,
            admission_check,
            admission_boundary_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_admission_available {
        return recovery_lifeline_command_envelope_check(
            "denied_missing_lifeline_command_admission",
            "recovery_lifeline_command_admission_missing",
            admission_check,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_admission_current_boot {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "recovery_lifeline_command_admission_event_id_not_current_boot",
            admission_check,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_admission_schema_ok {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "recovery_lifeline_command_admission_wrong_schema_or_variant",
            admission_check,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_admission_binding_ok {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            candidate.command_admission_binding_reason,
            admission_check,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_id_supported {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            admission_check,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.argument_schema_matches {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            admission_check,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.argument_hash_present {
        return recovery_lifeline_command_envelope_check(
            "invalid_reference",
            "recovery_lifeline_command_argument_hash_missing",
            admission_check,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.required_capability_matches {
        return recovery_lifeline_command_envelope_check(
            "rejected",
            "recovery_lifeline_command_required_capability_mismatch",
            admission_check,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.target_locator_present {
        return recovery_lifeline_command_envelope_check(
            "invalid_reference",
            "recovery_lifeline_command_target_locator_missing",
            admission_check,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.reference_hash_matches {
        return recovery_lifeline_command_envelope_check(
            "mismatched_command_envelope_reference_hash",
            "recovery_lifeline_command_envelope_reference_hash_mismatch",
            admission_check,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    recovery_lifeline_command_envelope_check(
        "defined_non_executable",
        "recovery_lifeline_command_envelope_reference_behavior_not_implemented",
        admission_check,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

pub(crate) fn recovery_lifeline_command_envelope_check(
    status: &'static str,
    reason: &'static str,
    admission_check: RecoveryLifelineCommandAdmissionCheck,
    command_admission_boundary_exposed: bool,
    command_admission_accepted: bool,
    command_envelope_reference_present: bool,
    command_id_supported: bool,
    argument_schema_matches: bool,
    argument_hash_present: bool,
    required_capability_matches: bool,
    target_locator_present: bool,
    reference_hash_matches: bool,
    all_inputs_valid: bool,
) -> RecoveryLifelineCommandEnvelopeCheck {
    RecoveryLifelineCommandEnvelopeCheck {
        status,
        reason,
        admission_check,
        command_admission_boundary_exposed,
        command_admission_accepted,
        command_envelope_reference_present,
        command_id_supported,
        argument_schema_matches,
        argument_hash_present,
        required_capability_matches,
        target_locator_present,
        reference_hash_matches,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
        dispatches_lifeline_command: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: if all_inputs_valid { "none" } else { "none" },
        load_attempted: false,
    }
}

pub(crate) fn recovery_lifeline_command_envelope_valid_candidate(
) -> RecoveryLifelineCommandEnvelopeCandidate {
    RecoveryLifelineCommandEnvelopeCandidate {
        admission_candidate: recovery_lifeline_command_admission_valid_candidate(),
        command_admission_available: true,
        command_admission_current_boot: true,
        command_admission_schema_ok: true,
        command_admission_binding_ok: true,
        command_admission_binding_reason: "retained_recovery_lifeline_command_admission_valid",
        direct_openai_recovery_shortcut_used: false,
        command_id_supported: true,
        argument_schema_matches: true,
        argument_hash_present: true,
        required_capability_matches: true,
        target_locator_present: true,
        reference_hash_matches: true,
    }
}

#[derive(Clone, Copy)]
enum CommandEnvelopeMutation {
    None,
    Admission(CommandAdmissionMutation),
    DirectProviderShortcut,
    MissingAdmission,
    PreviousAdmission,
    WrongSchemaAdmission,
    SubstitutedAdmission,
    MismatchedAdmission,
    UnsupportedCommand,
    ArgumentSchemaMismatch,
    RequiredCapabilityMismatch,
    ArgumentHashMissing,
    TargetLocatorMissing,
    ReferenceHashMismatch,
}

const fn envelope_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CommandEnvelopeMutation,
) -> CaseSpec<CommandEnvelopeMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

fn evaluate_recovery_lifeline_command_envelope_case(
    input: RecoveryLifelineCommandEnvelopeCandidate,
    _require_live_retained: bool,
) -> RecoveryLifelineCommandEnvelopeCheck {
    evaluate_recovery_lifeline_command_envelope(input)
}

fn recovery_lifeline_command_envelope_selftest_case_from_spec(
    spec: &CaseSpec<CommandEnvelopeMutation>,
    check: RecoveryLifelineCommandEnvelopeCheck,
) -> RecoveryLifelineCommandEnvelopeSelfTestCase {
    recovery_lifeline_command_envelope_selftest_case(
        spec.name,
        spec.expected_status,
        spec.expected_reason,
        check,
    )
}

fn apply_command_envelope_case(
    input: &mut RecoveryLifelineCommandEnvelopeCandidate,
    mutation: CommandEnvelopeMutation,
) {
    match mutation {
        CommandEnvelopeMutation::None => {}
        CommandEnvelopeMutation::Admission(mutation) => {
            apply_command_admission_case(&mut input.admission_candidate, mutation)
        }
        CommandEnvelopeMutation::DirectProviderShortcut => {
            input.direct_openai_recovery_shortcut_used = true;
        }
        CommandEnvelopeMutation::MissingAdmission => input.command_admission_available = false,
        CommandEnvelopeMutation::PreviousAdmission => input.command_admission_current_boot = false,
        CommandEnvelopeMutation::WrongSchemaAdmission => input.command_admission_schema_ok = false,
        CommandEnvelopeMutation::SubstitutedAdmission => {
            input.command_admission_binding_ok = false;
            input.command_admission_binding_reason =
                "recovery_lifeline_command_admission_substituted_record";
        }
        CommandEnvelopeMutation::MismatchedAdmission => {
            input.command_admission_binding_ok = false;
            input.command_admission_binding_reason =
                "recovery_lifeline_command_admission_binding_mismatch";
        }
        CommandEnvelopeMutation::UnsupportedCommand => input.command_id_supported = false,
        CommandEnvelopeMutation::ArgumentSchemaMismatch => input.argument_schema_matches = false,
        CommandEnvelopeMutation::RequiredCapabilityMismatch => {
            input.required_capability_matches = false;
        }
        CommandEnvelopeMutation::ArgumentHashMissing => input.argument_hash_present = false,
        CommandEnvelopeMutation::TargetLocatorMissing => input.target_locator_present = false,
        CommandEnvelopeMutation::ReferenceHashMismatch => input.reference_hash_matches = false,
    }
}

const COMMAND_ENVELOPE_CASES: [CaseSpec<CommandEnvelopeMutation>;
    RECOVERY_LIFELINE_COMMAND_ENVELOPE_SELFTEST_CASES] = [
    envelope_case(
        "missing_lifeline_request_event_id",
        "missing",
        "recovery_lifeline_request_event_id_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingRequest),
    ),
    envelope_case(
        "stale_dropped_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_stale_or_dropped",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::StaleRequest),
    ),
    envelope_case(
        "previous_boot_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousRequest),
    ),
    envelope_case(
        "wrong_schema_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaRequest),
    ),
    envelope_case(
        "substituted_lifeline_request_record",
        "rejected",
        "recovery_lifeline_request_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedRequest),
    ),
    envelope_case(
        "lifeline_request_reference_hash_mismatch",
        "rejected",
        "recovery_lifeline_request_reference_hash_mismatch",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::RequestHashMismatch),
    ),
    envelope_case(
        "protocol_state_missing_after_valid_request",
        "denied_missing_lifeline_protocol_state",
        "recovery_lifeline_protocol_state_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingProtocolState),
    ),
    envelope_case(
        "previous_boot_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousProtocolState),
    ),
    envelope_case(
        "wrong_schema_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaProtocolState),
    ),
    envelope_case(
        "substituted_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedProtocolState),
    ),
    envelope_case(
        "command_vocabulary_missing_after_protocol_state",
        "denied_missing_lifeline_command_vocabulary",
        "recovery_lifeline_command_vocabulary_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingCommandVocabulary),
    ),
    envelope_case(
        "previous_boot_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousCommandVocabulary),
    ),
    envelope_case(
        "wrong_schema_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaCommandVocabulary),
    ),
    envelope_case(
        "substituted_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedCommandVocabulary),
    ),
    envelope_case(
        "direct_openai_recovery_shortcut_rejected",
        "rejected",
        "direct_openai_provider_path_not_recovery_lifeline",
        CommandEnvelopeMutation::DirectProviderShortcut,
    ),
    envelope_case(
        "loader_runtime_isolation_missing_after_command_vocabulary",
        "denied_missing_loader_runtime_isolation",
        "recovery_loader_runtime_isolation_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingLoaderRuntimeIsolation),
    ),
    envelope_case(
        "previous_boot_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::PreviousLoaderRuntimeIsolation,
        ),
    ),
    envelope_case(
        "wrong_schema_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::WrongSchemaLoaderRuntimeIsolation,
        ),
    ),
    envelope_case(
        "substituted_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_substituted_record",
        CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::SubstitutedLoaderRuntimeIsolation,
        ),
    ),
    envelope_case(
        "mismatched_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_binding_mismatch",
        CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MismatchedLoaderRuntimeIsolation,
        ),
    ),
    envelope_case(
        "rollback_transaction_engine_missing_after_loader",
        "denied_missing_rollback_transaction_engine",
        "recovery_rollback_transaction_engine_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingRollbackEngine),
    ),
    envelope_case(
        "previous_boot_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousRollbackEngine),
    ),
    envelope_case(
        "wrong_schema_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaRollbackEngine),
    ),
    envelope_case(
        "substituted_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedRollbackEngine),
    ),
    envelope_case(
        "mismatched_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_binding_mismatch",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MismatchedRollbackEngine),
    ),
    envelope_case(
        "durable_persistence_boundary_missing_after_rollback_engine",
        "denied_missing_durable_audit_rollback_persistence",
        "durable_audit_rollback_persistence_missing",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingDurablePersistence),
    ),
    envelope_case(
        "previous_boot_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousDurablePersistence),
    ),
    envelope_case(
        "wrong_schema_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaDurablePersistence),
    ),
    envelope_case(
        "substituted_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedDurablePersistence),
    ),
    envelope_case(
        "mismatched_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_binding_mismatch",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MismatchedDurablePersistence),
    ),
    envelope_case(
        "recovery_memory_provenance_boundary_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_provenance_missing",
        CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingMemoryProvenanceBoundary,
        ),
    ),
    envelope_case(
        "previous_boot_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_event_id_not_current_boot",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousMemoryProvenance),
    ),
    envelope_case(
        "wrong_schema_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_wrong_schema_or_variant",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaMemoryProvenance),
    ),
    envelope_case(
        "substituted_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_substituted_record",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedMemoryProvenance),
    ),
    envelope_case(
        "mismatched_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_binding_mismatch",
        CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MismatchedMemoryProvenance),
    ),
    envelope_case(
        "recovery_lifeline_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_lifeline_command_admission_missing",
        CommandEnvelopeMutation::MissingAdmission,
    ),
    envelope_case(
        "previous_boot_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_event_id_not_current_boot",
        CommandEnvelopeMutation::PreviousAdmission,
    ),
    envelope_case(
        "wrong_schema_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_wrong_schema_or_variant",
        CommandEnvelopeMutation::WrongSchemaAdmission,
    ),
    envelope_case(
        "substituted_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_substituted_record",
        CommandEnvelopeMutation::SubstitutedAdmission,
    ),
    envelope_case(
        "mismatched_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_binding_mismatch",
        CommandEnvelopeMutation::MismatchedAdmission,
    ),
    envelope_case(
        "unsupported_lifeline_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandEnvelopeMutation::UnsupportedCommand,
    ),
    envelope_case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandEnvelopeMutation::ArgumentSchemaMismatch,
    ),
    envelope_case(
        "required_capability_mismatch",
        "rejected",
        "recovery_lifeline_command_required_capability_mismatch",
        CommandEnvelopeMutation::RequiredCapabilityMismatch,
    ),
    envelope_case(
        "argument_hash_missing",
        "invalid_reference",
        "recovery_lifeline_command_argument_hash_missing",
        CommandEnvelopeMutation::ArgumentHashMissing,
    ),
    envelope_case(
        "target_locator_missing",
        "invalid_reference",
        "recovery_lifeline_command_target_locator_missing",
        CommandEnvelopeMutation::TargetLocatorMissing,
    ),
    envelope_case(
        "command_envelope_reference_hash_mismatch",
        "mismatched_command_envelope_reference_hash",
        "recovery_lifeline_command_envelope_reference_hash_mismatch",
        CommandEnvelopeMutation::ReferenceHashMismatch,
    ),
    envelope_case(
        "all_inputs_present_command_envelope_still_non_executable",
        "defined_non_executable",
        "recovery_lifeline_command_envelope_reference_behavior_not_implemented",
        CommandEnvelopeMutation::None,
    ),
];

pub(crate) fn recovery_lifeline_command_envelope_selftest_cases(
) -> [RecoveryLifelineCommandEnvelopeSelfTestCase; RECOVERY_LIFELINE_COMMAND_ENVELOPE_SELFTEST_CASES]
{
    run_selftest_cases_with(
        recovery_lifeline_command_envelope_valid_candidate(),
        &COMMAND_ENVELOPE_CASES,
        apply_command_envelope_case,
        evaluate_recovery_lifeline_command_envelope_case,
        recovery_lifeline_command_envelope_selftest_case_from_spec,
    )
}
pub(crate) fn recovery_lifeline_command_envelope_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandEnvelopeCheck,
) -> RecoveryLifelineCommandEnvelopeSelfTestCase {
    RecoveryLifelineCommandEnvelopeSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        actual_admission_status: check.admission_check.status,
        actual_admission_reason: check.admission_check.reason,
        command_admission_boundary_exposed: check.command_admission_boundary_exposed,
        command_admission_accepted: check.command_admission_accepted,
        command_envelope_reference_present: check.command_envelope_reference_present,
        command_id_supported: check.command_id_supported,
        argument_schema_matches: check.argument_schema_matches,
        argument_hash_present: check.argument_hash_present,
        required_capability_matches: check.required_capability_matches,
        target_locator_present: check.target_locator_present,
        reference_hash_matches: check.reference_hash_matches,
        command_execution_enabled: check.command_execution_enabled,
        accepts_lifeline_command_envelope: check.accepts_lifeline_command_envelope,
        dispatches_lifeline_command: check.dispatches_lifeline_command,
        authorizes_recovery_load: check.authorizes_recovery_load,
        can_move_beyond_denial: check.can_move_beyond_denial,
        loads_recovery_loader: check.loads_recovery_loader,
        loads_recovery_artifact: check.loads_recovery_artifact,
        creates_durable_records: check.creates_durable_records,
        installs_rollback_plan: check.installs_rollback_plan,
        allocates_service_slot: check.allocates_service_slot,
        service_inventory_change: check.service_inventory_change,
        load_attempted: check.load_attempted,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_lifeline_command_dispatch_candidate_from_retained(
    retained_envelope: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandEnvelopeReference,
    )>,
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    retained_body: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandBodyCanonicalizationReference,
    )>,
    retained_handler: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandHandlerBindingReference,
    )>,
    retained_status_handler: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineStatusReadHandlerReference,
    )>,
    retained_preview_authorization: Option<(
        event_log::EventId,
        event_log::RecoveryRollbackPreviewAuthorizationReference,
    )>,
    retained_apply_authorization: Option<(
        event_log::EventId,
        event_log::RecoveryRollbackApplyAuthorizationReference,
    )>,
    retained_disable_module_target_binding: Option<(
        event_log::EventId,
        event_log::RecoveryDisableModuleTargetBindingReference,
    )>,
    retained_restart_last_good_target_binding: Option<(
        event_log::EventId,
        event_log::RecoveryRestartLastGoodTargetBindingReference,
    )>,
    retained_load_artifact_by_hash_target_binding: Option<(
        event_log::EventId,
        event_log::RecoveryLoadArtifactByHashTargetBindingReference,
    )>,
    retained_recovery_memory_write_authority: Option<(
        event_log::EventId,
        event_log::RecoveryMemoryWriteAuthorityReference,
    )>,
    retained_durable_audit_rollback_write_authority: Option<(
        event_log::EventId,
        event_log::DurableAuditRollbackWriteAuthorityReference,
    )>,
    retained_service_inventory_side_effect_boundary: Option<(
        event_log::EventId,
        event_log::RecoveryServiceInventorySideEffectBoundaryReference,
    )>,
    retained_command_dispatch_behavior: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandDispatchBehaviorReference,
    )>,
    retained_executor_capability_table: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutorCapabilityTableReference,
    )>,
    retained_side_effect_gate: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandSideEffectGateReference,
    )>,
    retained_execution_enablement: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_preflight: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_intent: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_commit_gate: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_result_denial: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_audit_denial: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_observation_denial: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
    retained_execution_completion_denial: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutionStageReference,
    )>,
) -> RecoveryLifelineCommandDispatchCandidate {
    let mut candidate = recovery_lifeline_command_dispatch_valid_candidate();
    candidate.command_body_canonicalization_present = false;
    candidate.command_handler_binding_present = false;
    candidate.status_read_handler_present = false;
    candidate.rollback_preview_authorization_present = false;
    candidate.rollback_apply_authorization_present = false;
    candidate.disable_module_target_binding_present = false;
    candidate.restart_last_good_target_binding_present = false;
    candidate.load_artifact_by_hash_target_binding_present = false;
    candidate.recovery_memory_write_authority_present = false;
    candidate.durable_audit_rollback_write_authority_present = false;
    candidate.service_inventory_side_effect_boundary_present = false;
    candidate.command_dispatch_behavior_present = false;
    candidate.executor_capability_table_present = false;
    candidate.side_effect_gate_present = false;
    candidate.execution_enablement_present = false;
    candidate.execution_preflight_present = false;
    candidate.execution_intent_present = false;
    candidate.execution_commit_gate_present = false;
    candidate.execution_result_denial_present = false;
    candidate.execution_audit_denial_present = false;
    candidate.execution_observation_denial_present = false;
    candidate.execution_completion_denial_present = false;
    candidate.execution_completion_denial_reference = None;

    let Some((envelope_event_id, envelope)) = retained_envelope else {
        candidate.command_envelope_reference_available = false;
        return candidate;
    };
    let Some((request_event_id, request_reference)) = retained_request else {
        candidate.command_envelope_reference_binding_ok = false;
        candidate.command_envelope_reference_binding_reason =
            "retained_recovery_lifeline_request_missing";
        return candidate;
    };
    if envelope.retained_lifeline_request_event_id != request_event_id {
        candidate.command_envelope_reference_binding_ok = false;
        candidate.command_envelope_reference_binding_reason =
            "retained_recovery_lifeline_command_envelope_event_id_stale_or_dropped";
        return candidate;
    }
    if envelope.lifeline_request_reference_hash != request_reference.lifeline_request_reference_hash
    {
        candidate.command_envelope_reference_binding_ok = false;
        candidate.command_envelope_reference_binding_reason =
            "retained_recovery_lifeline_command_envelope_request_hash_mismatch";
    }
    let mut accepted_body = None;
    if let Some((body_event_id, body)) = retained_body {
        if body.retained_command_envelope_reference_event_id == envelope_event_id
            && method_eq(body.command_id, envelope.command_id)
            && method_eq(body.argument_schema, envelope.argument_schema)
            && body.argument_hash == envelope.argument_hash
            && body.target_locator == envelope.target_locator
            && body.command_envelope_reference_hash == envelope.command_envelope_reference_hash
            && method_eq(
                body.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
        {
            candidate.command_body_canonicalization_present = true;
            accepted_body = Some((body_event_id, body));
        }
    }
    let mut accepted_handler = None;
    if let (Some((body_event_id, body)), Some((handler_event_id, handler))) =
        (accepted_body, retained_handler)
    {
        if handler.retained_command_body_canonicalization_event_id == body_event_id
            && method_eq(handler.command_id, body.command_id)
            && method_eq(handler.argument_schema, body.argument_schema)
            && handler.argument_hash == body.argument_hash
            && handler.target_locator == body.target_locator
            && handler.command_envelope_reference_hash == body.command_envelope_reference_hash
            && handler.command_body_canonicalization_hash == body.command_body_canonicalization_hash
            && method_eq(
                handler.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                handler.handler_id,
                RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID,
            )
        {
            candidate.command_handler_binding_present = true;
            accepted_handler = Some((handler_event_id, handler));
        }
    }
    let mut accepted_status_handler = None;
    if let (Some((handler_event_id, handler)), Some((status_event_id, status_handler))) =
        (accepted_handler, retained_status_handler)
    {
        if status_handler.retained_command_handler_binding_event_id == handler_event_id
            && method_eq(status_handler.command_id, handler.command_id)
            && method_eq(status_handler.argument_schema, handler.argument_schema)
            && status_handler.argument_hash == handler.argument_hash
            && status_handler.target_locator == handler.target_locator
            && status_handler.command_envelope_reference_hash
                == handler.command_envelope_reference_hash
            && status_handler.command_body_canonicalization_hash
                == handler.command_body_canonicalization_hash
            && status_handler.handler_binding_hash == handler.handler_binding_hash
            && method_eq(
                status_handler.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                status_handler.status_handler_id,
                RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID,
            )
        {
            candidate.status_read_handler_present = true;
            accepted_status_handler = Some((status_event_id, status_handler));
        }
    }
    let mut accepted_preview_authorization = None;
    if let (
        Some((status_event_id, status_handler)),
        Some((preview_event_id, preview_authorization)),
    ) = (accepted_status_handler, retained_preview_authorization)
    {
        if preview_authorization.retained_status_read_handler_event_id == status_event_id
            && method_eq(preview_authorization.command_id, status_handler.command_id)
            && method_eq(
                preview_authorization.argument_schema,
                status_handler.argument_schema,
            )
            && preview_authorization.argument_hash == status_handler.argument_hash
            && preview_authorization.target_locator == status_handler.target_locator
            && preview_authorization.command_envelope_reference_hash
                == status_handler.command_envelope_reference_hash
            && preview_authorization.command_body_canonicalization_hash
                == status_handler.command_body_canonicalization_hash
            && preview_authorization.handler_binding_hash == status_handler.handler_binding_hash
            && preview_authorization.status_read_handler_hash
                == status_handler.status_read_handler_hash
            && method_eq(
                preview_authorization.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                preview_authorization.rollback_preview_authorization_id,
                RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID,
            )
        {
            candidate.rollback_preview_authorization_present = true;
            accepted_preview_authorization = Some((preview_event_id, preview_authorization));
        }
    }
    let mut accepted_apply_authorization = None;
    if let (
        Some((preview_event_id, preview_authorization)),
        Some((apply_event_id, apply_authorization)),
    ) = (accepted_preview_authorization, retained_apply_authorization)
    {
        if apply_authorization.retained_rollback_preview_authorization_event_id == preview_event_id
            && method_eq(
                apply_authorization.command_id,
                preview_authorization.command_id,
            )
            && method_eq(
                apply_authorization.argument_schema,
                preview_authorization.argument_schema,
            )
            && apply_authorization.argument_hash == preview_authorization.argument_hash
            && apply_authorization.target_locator == preview_authorization.target_locator
            && apply_authorization.command_envelope_reference_hash
                == preview_authorization.command_envelope_reference_hash
            && apply_authorization.command_body_canonicalization_hash
                == preview_authorization.command_body_canonicalization_hash
            && apply_authorization.handler_binding_hash
                == preview_authorization.handler_binding_hash
            && apply_authorization.status_read_handler_hash
                == preview_authorization.status_read_handler_hash
            && apply_authorization.rollback_preview_authorization_hash
                == preview_authorization.rollback_preview_authorization_hash
            && method_eq(
                apply_authorization.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                apply_authorization.rollback_apply_authorization_id,
                RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID,
            )
        {
            candidate.rollback_apply_authorization_present = true;
            accepted_apply_authorization = Some((apply_event_id, apply_authorization));
        }
    }
    let mut accepted_disable_module_target_binding = None;
    if let (
        Some((apply_event_id, apply_authorization)),
        Some((disable_event_id, disable_module_target_binding)),
    ) = (
        accepted_apply_authorization,
        retained_disable_module_target_binding,
    ) {
        candidate.disable_module_target_binding_present = disable_module_target_binding
            .retained_rollback_apply_authorization_event_id
            == apply_event_id
            && method_eq(
                disable_module_target_binding.command_id,
                apply_authorization.command_id,
            )
            && method_eq(
                disable_module_target_binding.argument_schema,
                apply_authorization.argument_schema,
            )
            && disable_module_target_binding.argument_hash == apply_authorization.argument_hash
            && disable_module_target_binding.target_locator == apply_authorization.target_locator
            && disable_module_target_binding.command_envelope_reference_hash
                == apply_authorization.command_envelope_reference_hash
            && disable_module_target_binding.command_body_canonicalization_hash
                == apply_authorization.command_body_canonicalization_hash
            && disable_module_target_binding.handler_binding_hash
                == apply_authorization.handler_binding_hash
            && disable_module_target_binding.status_read_handler_hash
                == apply_authorization.status_read_handler_hash
            && disable_module_target_binding.rollback_preview_authorization_hash
                == apply_authorization.rollback_preview_authorization_hash
            && disable_module_target_binding.rollback_apply_authorization_hash
                == apply_authorization.rollback_apply_authorization_hash
            && method_eq(
                disable_module_target_binding.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                disable_module_target_binding.disable_module_target_id,
                RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID,
            );
        if candidate.disable_module_target_binding_present {
            accepted_disable_module_target_binding =
                Some((disable_event_id, disable_module_target_binding));
        }
    }
    let mut accepted_restart_last_good_target_binding = None;
    if let (
        Some((disable_event_id, disable_module_target_binding)),
        Some((restart_event_id, restart_last_good_target_binding)),
    ) = (
        accepted_disable_module_target_binding,
        retained_restart_last_good_target_binding,
    ) {
        candidate.restart_last_good_target_binding_present = restart_last_good_target_binding
            .retained_disable_module_target_binding_event_id
            == disable_event_id
            && method_eq(
                restart_last_good_target_binding.command_id,
                disable_module_target_binding.command_id,
            )
            && method_eq(
                restart_last_good_target_binding.argument_schema,
                disable_module_target_binding.argument_schema,
            )
            && restart_last_good_target_binding.argument_hash
                == disable_module_target_binding.argument_hash
            && restart_last_good_target_binding.target_locator
                == disable_module_target_binding.target_locator
            && restart_last_good_target_binding.command_envelope_reference_hash
                == disable_module_target_binding.command_envelope_reference_hash
            && restart_last_good_target_binding.command_body_canonicalization_hash
                == disable_module_target_binding.command_body_canonicalization_hash
            && restart_last_good_target_binding.handler_binding_hash
                == disable_module_target_binding.handler_binding_hash
            && restart_last_good_target_binding.status_read_handler_hash
                == disable_module_target_binding.status_read_handler_hash
            && restart_last_good_target_binding.rollback_preview_authorization_hash
                == disable_module_target_binding.rollback_preview_authorization_hash
            && restart_last_good_target_binding.rollback_apply_authorization_hash
                == disable_module_target_binding.rollback_apply_authorization_hash
            && restart_last_good_target_binding.disable_module_target_binding_hash
                == disable_module_target_binding.disable_module_target_binding_hash
            && method_eq(
                restart_last_good_target_binding.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                restart_last_good_target_binding.restart_last_good_target_id,
                RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID,
            );
        if candidate.restart_last_good_target_binding_present {
            accepted_restart_last_good_target_binding =
                Some((restart_event_id, restart_last_good_target_binding));
        }
    }
    let mut accepted_load_artifact_by_hash_target_binding = None;
    if let (
        Some((restart_event_id, restart_last_good_target_binding)),
        Some((load_event_id, load_artifact_by_hash_target_binding)),
    ) = (
        accepted_restart_last_good_target_binding,
        retained_load_artifact_by_hash_target_binding,
    ) {
        candidate.load_artifact_by_hash_target_binding_present =
            load_artifact_by_hash_target_binding.retained_restart_last_good_target_binding_event_id
                == restart_event_id
                && method_eq(
                    load_artifact_by_hash_target_binding.command_id,
                    restart_last_good_target_binding.command_id,
                )
                && method_eq(
                    load_artifact_by_hash_target_binding.argument_schema,
                    restart_last_good_target_binding.argument_schema,
                )
                && load_artifact_by_hash_target_binding.argument_hash
                    == restart_last_good_target_binding.argument_hash
                && load_artifact_by_hash_target_binding.target_locator
                    == restart_last_good_target_binding.target_locator
                && load_artifact_by_hash_target_binding.command_envelope_reference_hash
                    == restart_last_good_target_binding.command_envelope_reference_hash
                && load_artifact_by_hash_target_binding.command_body_canonicalization_hash
                    == restart_last_good_target_binding.command_body_canonicalization_hash
                && load_artifact_by_hash_target_binding.handler_binding_hash
                    == restart_last_good_target_binding.handler_binding_hash
                && load_artifact_by_hash_target_binding.status_read_handler_hash
                    == restart_last_good_target_binding.status_read_handler_hash
                && load_artifact_by_hash_target_binding.rollback_preview_authorization_hash
                    == restart_last_good_target_binding.rollback_preview_authorization_hash
                && load_artifact_by_hash_target_binding.rollback_apply_authorization_hash
                    == restart_last_good_target_binding.rollback_apply_authorization_hash
                && load_artifact_by_hash_target_binding.disable_module_target_binding_hash
                    == restart_last_good_target_binding.disable_module_target_binding_hash
                && load_artifact_by_hash_target_binding.restart_last_good_target_binding_hash
                    == restart_last_good_target_binding.restart_last_good_target_binding_hash
                && method_eq(
                    load_artifact_by_hash_target_binding.command_dispatch_boundary_id,
                    RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                )
                && method_eq(
                    load_artifact_by_hash_target_binding.load_artifact_by_hash_target_id,
                    RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID,
                );
        if candidate.load_artifact_by_hash_target_binding_present {
            accepted_load_artifact_by_hash_target_binding =
                Some((load_event_id, load_artifact_by_hash_target_binding));
        }
    }
    let mut accepted_recovery_memory_write_authority = None;
    if let (
        Some((load_event_id, load_artifact_by_hash_target_binding)),
        Some((memory_event_id, recovery_memory_write_authority)),
    ) = (
        accepted_load_artifact_by_hash_target_binding,
        retained_recovery_memory_write_authority,
    ) {
        candidate.recovery_memory_write_authority_present = recovery_memory_write_authority
            .retained_load_artifact_by_hash_target_binding_event_id
            == load_event_id
            && method_eq(
                recovery_memory_write_authority.command_id,
                load_artifact_by_hash_target_binding.command_id,
            )
            && method_eq(
                recovery_memory_write_authority.argument_schema,
                load_artifact_by_hash_target_binding.argument_schema,
            )
            && recovery_memory_write_authority.argument_hash
                == load_artifact_by_hash_target_binding.argument_hash
            && recovery_memory_write_authority.target_locator
                == load_artifact_by_hash_target_binding.target_locator
            && recovery_memory_write_authority.command_envelope_reference_hash
                == load_artifact_by_hash_target_binding.command_envelope_reference_hash
            && recovery_memory_write_authority.command_body_canonicalization_hash
                == load_artifact_by_hash_target_binding.command_body_canonicalization_hash
            && recovery_memory_write_authority.handler_binding_hash
                == load_artifact_by_hash_target_binding.handler_binding_hash
            && recovery_memory_write_authority.status_read_handler_hash
                == load_artifact_by_hash_target_binding.status_read_handler_hash
            && recovery_memory_write_authority.rollback_preview_authorization_hash
                == load_artifact_by_hash_target_binding.rollback_preview_authorization_hash
            && recovery_memory_write_authority.rollback_apply_authorization_hash
                == load_artifact_by_hash_target_binding.rollback_apply_authorization_hash
            && recovery_memory_write_authority.disable_module_target_binding_hash
                == load_artifact_by_hash_target_binding.disable_module_target_binding_hash
            && recovery_memory_write_authority.restart_last_good_target_binding_hash
                == load_artifact_by_hash_target_binding.restart_last_good_target_binding_hash
            && recovery_memory_write_authority.load_artifact_by_hash_target_binding_hash
                == load_artifact_by_hash_target_binding.load_artifact_by_hash_target_binding_hash
            && method_eq(
                recovery_memory_write_authority.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                recovery_memory_write_authority.recovery_memory_write_authority_id,
                RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID,
            );
        if candidate.recovery_memory_write_authority_present {
            accepted_recovery_memory_write_authority =
                Some((memory_event_id, recovery_memory_write_authority));
        }
    }
    let mut accepted_durable_audit_rollback_write_authority = None;
    if let (
        Some((memory_event_id, recovery_memory_write_authority)),
        Some((durable_event_id, durable_audit_rollback_write_authority)),
    ) = (
        accepted_recovery_memory_write_authority,
        retained_durable_audit_rollback_write_authority,
    ) {
        candidate.durable_audit_rollback_write_authority_present =
            durable_audit_rollback_write_authority
                .retained_recovery_memory_write_authority_event_id
                == memory_event_id
                && method_eq(
                    durable_audit_rollback_write_authority.command_id,
                    recovery_memory_write_authority.command_id,
                )
                && method_eq(
                    durable_audit_rollback_write_authority.argument_schema,
                    recovery_memory_write_authority.argument_schema,
                )
                && durable_audit_rollback_write_authority.argument_hash
                    == recovery_memory_write_authority.argument_hash
                && durable_audit_rollback_write_authority.target_locator
                    == recovery_memory_write_authority.target_locator
                && durable_audit_rollback_write_authority.command_envelope_reference_hash
                    == recovery_memory_write_authority.command_envelope_reference_hash
                && durable_audit_rollback_write_authority.command_body_canonicalization_hash
                    == recovery_memory_write_authority.command_body_canonicalization_hash
                && durable_audit_rollback_write_authority.handler_binding_hash
                    == recovery_memory_write_authority.handler_binding_hash
                && durable_audit_rollback_write_authority.status_read_handler_hash
                    == recovery_memory_write_authority.status_read_handler_hash
                && durable_audit_rollback_write_authority.rollback_preview_authorization_hash
                    == recovery_memory_write_authority.rollback_preview_authorization_hash
                && durable_audit_rollback_write_authority.rollback_apply_authorization_hash
                    == recovery_memory_write_authority.rollback_apply_authorization_hash
                && durable_audit_rollback_write_authority.disable_module_target_binding_hash
                    == recovery_memory_write_authority.disable_module_target_binding_hash
                && durable_audit_rollback_write_authority.restart_last_good_target_binding_hash
                    == recovery_memory_write_authority.restart_last_good_target_binding_hash
                && durable_audit_rollback_write_authority.load_artifact_by_hash_target_binding_hash
                    == recovery_memory_write_authority.load_artifact_by_hash_target_binding_hash
                && durable_audit_rollback_write_authority.recovery_memory_write_authority_hash
                    == recovery_memory_write_authority.recovery_memory_write_authority_hash
                && method_eq(
                    durable_audit_rollback_write_authority.command_dispatch_boundary_id,
                    RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                )
                && method_eq(
                    durable_audit_rollback_write_authority
                        .durable_audit_rollback_write_authority_id,
                    DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID,
                );
        if candidate.durable_audit_rollback_write_authority_present {
            accepted_durable_audit_rollback_write_authority =
                Some((durable_event_id, durable_audit_rollback_write_authority));
        }
    }
    let mut accepted_service_inventory_side_effect_boundary = None;
    if let (
        Some((durable_event_id, durable_audit_rollback_write_authority)),
        Some((service_event_id, service_inventory_side_effect_boundary)),
    ) = (
        accepted_durable_audit_rollback_write_authority,
        retained_service_inventory_side_effect_boundary,
    ) {
        candidate.service_inventory_side_effect_boundary_present =
            service_inventory_side_effect_boundary
                .retained_durable_audit_rollback_write_authority_event_id
                == durable_event_id
                && method_eq(
                    service_inventory_side_effect_boundary.command_id,
                    durable_audit_rollback_write_authority.command_id,
                )
                && method_eq(
                    service_inventory_side_effect_boundary.argument_schema,
                    durable_audit_rollback_write_authority.argument_schema,
                )
                && service_inventory_side_effect_boundary.argument_hash
                    == durable_audit_rollback_write_authority.argument_hash
                && service_inventory_side_effect_boundary.target_locator
                    == durable_audit_rollback_write_authority.target_locator
                && service_inventory_side_effect_boundary.command_envelope_reference_hash
                    == durable_audit_rollback_write_authority.command_envelope_reference_hash
                && service_inventory_side_effect_boundary.command_body_canonicalization_hash
                    == durable_audit_rollback_write_authority.command_body_canonicalization_hash
                && service_inventory_side_effect_boundary.handler_binding_hash
                    == durable_audit_rollback_write_authority.handler_binding_hash
                && service_inventory_side_effect_boundary.status_read_handler_hash
                    == durable_audit_rollback_write_authority.status_read_handler_hash
                && service_inventory_side_effect_boundary.rollback_preview_authorization_hash
                    == durable_audit_rollback_write_authority.rollback_preview_authorization_hash
                && service_inventory_side_effect_boundary.rollback_apply_authorization_hash
                    == durable_audit_rollback_write_authority.rollback_apply_authorization_hash
                && service_inventory_side_effect_boundary.disable_module_target_binding_hash
                    == durable_audit_rollback_write_authority.disable_module_target_binding_hash
                && service_inventory_side_effect_boundary.restart_last_good_target_binding_hash
                    == durable_audit_rollback_write_authority.restart_last_good_target_binding_hash
                && service_inventory_side_effect_boundary.load_artifact_by_hash_target_binding_hash
                    == durable_audit_rollback_write_authority
                        .load_artifact_by_hash_target_binding_hash
                && service_inventory_side_effect_boundary.recovery_memory_write_authority_hash
                    == durable_audit_rollback_write_authority.recovery_memory_write_authority_hash
                && service_inventory_side_effect_boundary
                    .durable_audit_rollback_write_authority_hash
                    == durable_audit_rollback_write_authority
                        .durable_audit_rollback_write_authority_hash
                && method_eq(
                    service_inventory_side_effect_boundary.command_dispatch_boundary_id,
                    RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                )
                && method_eq(
                    service_inventory_side_effect_boundary
                        .service_inventory_side_effect_boundary_id,
                    RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID,
                );
        if candidate.service_inventory_side_effect_boundary_present {
            accepted_service_inventory_side_effect_boundary =
                Some((service_event_id, service_inventory_side_effect_boundary));
        }
    }
    let mut accepted_command_dispatch_behavior = None;
    if let (
        Some((service_event_id, service_inventory_side_effect_boundary)),
        Some((behavior_event_id, command_dispatch_behavior)),
    ) = (
        accepted_service_inventory_side_effect_boundary,
        retained_command_dispatch_behavior,
    ) {
        candidate.command_dispatch_behavior_present = command_dispatch_behavior
            .retained_service_inventory_side_effect_boundary_event_id
            == service_event_id
            && method_eq(
                command_dispatch_behavior.command_id,
                service_inventory_side_effect_boundary.command_id,
            )
            && method_eq(
                command_dispatch_behavior.argument_schema,
                service_inventory_side_effect_boundary.argument_schema,
            )
            && command_dispatch_behavior.argument_hash
                == service_inventory_side_effect_boundary.argument_hash
            && command_dispatch_behavior.target_locator
                == service_inventory_side_effect_boundary.target_locator
            && command_dispatch_behavior.command_envelope_reference_hash
                == service_inventory_side_effect_boundary.command_envelope_reference_hash
            && command_dispatch_behavior.command_body_canonicalization_hash
                == service_inventory_side_effect_boundary.command_body_canonicalization_hash
            && command_dispatch_behavior.handler_binding_hash
                == service_inventory_side_effect_boundary.handler_binding_hash
            && command_dispatch_behavior.status_read_handler_hash
                == service_inventory_side_effect_boundary.status_read_handler_hash
            && command_dispatch_behavior.rollback_preview_authorization_hash
                == service_inventory_side_effect_boundary.rollback_preview_authorization_hash
            && command_dispatch_behavior.rollback_apply_authorization_hash
                == service_inventory_side_effect_boundary.rollback_apply_authorization_hash
            && command_dispatch_behavior.disable_module_target_binding_hash
                == service_inventory_side_effect_boundary.disable_module_target_binding_hash
            && command_dispatch_behavior.restart_last_good_target_binding_hash
                == service_inventory_side_effect_boundary.restart_last_good_target_binding_hash
            && command_dispatch_behavior.load_artifact_by_hash_target_binding_hash
                == service_inventory_side_effect_boundary.load_artifact_by_hash_target_binding_hash
            && command_dispatch_behavior.recovery_memory_write_authority_hash
                == service_inventory_side_effect_boundary.recovery_memory_write_authority_hash
            && command_dispatch_behavior.durable_audit_rollback_write_authority_hash
                == service_inventory_side_effect_boundary
                    .durable_audit_rollback_write_authority_hash
            && command_dispatch_behavior.service_inventory_side_effect_boundary_hash
                == service_inventory_side_effect_boundary
                    .service_inventory_side_effect_boundary_hash
            && method_eq(
                command_dispatch_behavior.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                command_dispatch_behavior.command_dispatch_behavior_id,
                RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID,
            );
        if candidate.command_dispatch_behavior_present {
            accepted_command_dispatch_behavior =
                Some((behavior_event_id, command_dispatch_behavior));
        }
    }
    let mut accepted_executor_capability_table = None;
    if let (
        Some((behavior_event_id, command_dispatch_behavior)),
        Some((executor_event_id, executor_capability_table)),
    ) = (
        accepted_command_dispatch_behavior,
        retained_executor_capability_table,
    ) {
        candidate.executor_capability_table_present = executor_capability_table
            .retained_command_dispatch_behavior_event_id
            == behavior_event_id
            && method_eq(
                executor_capability_table.command_id,
                command_dispatch_behavior.command_id,
            )
            && method_eq(
                executor_capability_table.argument_schema,
                command_dispatch_behavior.argument_schema,
            )
            && executor_capability_table.argument_hash == command_dispatch_behavior.argument_hash
            && executor_capability_table.target_locator == command_dispatch_behavior.target_locator
            && executor_capability_table.command_envelope_reference_hash
                == command_dispatch_behavior.command_envelope_reference_hash
            && executor_capability_table.command_body_canonicalization_hash
                == command_dispatch_behavior.command_body_canonicalization_hash
            && executor_capability_table.handler_binding_hash
                == command_dispatch_behavior.handler_binding_hash
            && executor_capability_table.status_read_handler_hash
                == command_dispatch_behavior.status_read_handler_hash
            && executor_capability_table.rollback_preview_authorization_hash
                == command_dispatch_behavior.rollback_preview_authorization_hash
            && executor_capability_table.rollback_apply_authorization_hash
                == command_dispatch_behavior.rollback_apply_authorization_hash
            && executor_capability_table.disable_module_target_binding_hash
                == command_dispatch_behavior.disable_module_target_binding_hash
            && executor_capability_table.restart_last_good_target_binding_hash
                == command_dispatch_behavior.restart_last_good_target_binding_hash
            && executor_capability_table.load_artifact_by_hash_target_binding_hash
                == command_dispatch_behavior.load_artifact_by_hash_target_binding_hash
            && executor_capability_table.recovery_memory_write_authority_hash
                == command_dispatch_behavior.recovery_memory_write_authority_hash
            && executor_capability_table.durable_audit_rollback_write_authority_hash
                == command_dispatch_behavior.durable_audit_rollback_write_authority_hash
            && executor_capability_table.service_inventory_side_effect_boundary_hash
                == command_dispatch_behavior.service_inventory_side_effect_boundary_hash
            && executor_capability_table.command_dispatch_behavior_hash
                == command_dispatch_behavior.command_dispatch_behavior_hash
            && method_eq(
                executor_capability_table.command_dispatch_boundary_id,
                RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            )
            && method_eq(
                executor_capability_table.executor_capability_table_id,
                RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID,
            );
        if candidate.executor_capability_table_present {
            accepted_executor_capability_table =
                Some((executor_event_id, executor_capability_table));
        }
    }
    let mut accepted_side_effect_gate = None;
    if let (
        Some((executor_event_id, executor_capability_table)),
        Some((side_effect_event_id, side_effect_gate)),
    ) = (
        accepted_executor_capability_table,
        retained_side_effect_gate,
    ) {
        candidate.side_effect_gate_present =
            side_effect_gate.retained_executor_capability_table_event_id == executor_event_id
                && method_eq(
                    side_effect_gate.command_id,
                    executor_capability_table.command_id,
                )
                && method_eq(
                    side_effect_gate.argument_schema,
                    executor_capability_table.argument_schema,
                )
                && side_effect_gate.argument_hash == executor_capability_table.argument_hash
                && side_effect_gate.target_locator == executor_capability_table.target_locator
                && side_effect_gate.command_envelope_reference_hash
                    == executor_capability_table.command_envelope_reference_hash
                && side_effect_gate.command_body_canonicalization_hash
                    == executor_capability_table.command_body_canonicalization_hash
                && side_effect_gate.handler_binding_hash
                    == executor_capability_table.handler_binding_hash
                && side_effect_gate.status_read_handler_hash
                    == executor_capability_table.status_read_handler_hash
                && side_effect_gate.rollback_preview_authorization_hash
                    == executor_capability_table.rollback_preview_authorization_hash
                && side_effect_gate.rollback_apply_authorization_hash
                    == executor_capability_table.rollback_apply_authorization_hash
                && side_effect_gate.disable_module_target_binding_hash
                    == executor_capability_table.disable_module_target_binding_hash
                && side_effect_gate.restart_last_good_target_binding_hash
                    == executor_capability_table.restart_last_good_target_binding_hash
                && side_effect_gate.load_artifact_by_hash_target_binding_hash
                    == executor_capability_table.load_artifact_by_hash_target_binding_hash
                && side_effect_gate.recovery_memory_write_authority_hash
                    == executor_capability_table.recovery_memory_write_authority_hash
                && side_effect_gate.durable_audit_rollback_write_authority_hash
                    == executor_capability_table.durable_audit_rollback_write_authority_hash
                && side_effect_gate.service_inventory_side_effect_boundary_hash
                    == executor_capability_table.service_inventory_side_effect_boundary_hash
                && side_effect_gate.command_dispatch_behavior_hash
                    == executor_capability_table.command_dispatch_behavior_hash
                && side_effect_gate.executor_capability_table_hash
                    == executor_capability_table.executor_capability_table_hash
                && method_eq(
                    side_effect_gate.command_dispatch_boundary_id,
                    RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
                )
                && method_eq(
                    side_effect_gate.side_effect_gate_id,
                    RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID,
                );
        if candidate.side_effect_gate_present {
            accepted_side_effect_gate = Some((side_effect_event_id, side_effect_gate));
        }
    }
    let execution_stage_presence =
        recovery_lifeline_command_execution_stage_chain_presence_from_retained(
            RecoveryLifelineCommandExecutionStageRetainedChain {
                side_effect_gate: accepted_side_effect_gate,
                execution_enablement: retained_execution_enablement,
                execution_preflight: retained_execution_preflight,
                execution_intent: retained_execution_intent,
                execution_commit_gate: retained_execution_commit_gate,
                execution_result_denial: retained_execution_result_denial,
                execution_audit_denial: retained_execution_audit_denial,
                execution_observation_denial: retained_execution_observation_denial,
                execution_completion_denial: retained_execution_completion_denial,
            },
        );
    candidate.execution_enablement_present = execution_stage_presence.execution_enablement_present;
    candidate.execution_preflight_present = execution_stage_presence.execution_preflight_present;
    candidate.execution_intent_present = execution_stage_presence.execution_intent_present;
    candidate.execution_commit_gate_present =
        execution_stage_presence.execution_commit_gate_present;
    candidate.execution_result_denial_present =
        execution_stage_presence.execution_result_denial_present;
    candidate.execution_audit_denial_present =
        execution_stage_presence.execution_audit_denial_present;
    candidate.execution_observation_denial_present =
        execution_stage_presence.execution_observation_denial_present;
    candidate.execution_completion_denial_present =
        execution_stage_presence.execution_completion_denial_present;
    candidate.execution_completion_denial_reference =
        if execution_stage_presence.execution_completion_denial_present {
            retained_execution_completion_denial
        } else {
            None
        };
    candidate
}

pub(crate) fn evaluate_recovery_lifeline_command_dispatch(
    candidate: RecoveryLifelineCommandDispatchCandidate,
) -> RecoveryLifelineCommandDispatchCheck {
    let envelope_check = evaluate_recovery_lifeline_command_envelope(candidate.envelope_candidate);
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_command_dispatch_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            envelope_check,
            candidate,
            false,
        );
    }
    if !method_eq(envelope_check.status, "defined_non_executable") {
        return recovery_lifeline_command_dispatch_check(
            envelope_check.status,
            envelope_check.reason,
            envelope_check,
            candidate,
            false,
        );
    }
    if !candidate.command_envelope_reference_available {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_envelope_reference",
            "recovery_lifeline_command_envelope_reference_missing",
            envelope_check,
            candidate,
            false,
        );
    }
    if !candidate.command_envelope_reference_current_boot {
        return recovery_lifeline_command_dispatch_check(
            "rejected",
            "recovery_lifeline_command_envelope_reference_event_id_not_current_boot",
            envelope_check,
            candidate,
            false,
        );
    }
    if !candidate.command_envelope_reference_schema_ok {
        return recovery_lifeline_command_dispatch_check(
            "rejected",
            "recovery_lifeline_command_envelope_reference_wrong_schema_or_variant",
            envelope_check,
            candidate,
            false,
        );
    }
    if !candidate.command_envelope_reference_binding_ok {
        return recovery_lifeline_command_dispatch_check(
            "rejected",
            candidate.command_envelope_reference_binding_reason,
            envelope_check,
            candidate,
            false,
        );
    }
    if !candidate.command_body_canonicalization_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_lifeline_command_body_canonicalization_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.command_handler_binding_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_lifeline_command_handler_binding_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.status_read_handler_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_lifeline_status_read_handler_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.rollback_preview_authorization_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_rollback_preview_authorization_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.rollback_apply_authorization_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_rollback_apply_authorization_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.disable_module_target_binding_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_disable_module_target_binding_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.restart_last_good_target_binding_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_restart_last_good_target_binding_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.load_artifact_by_hash_target_binding_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_load_artifact_by_hash_target_binding_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.recovery_memory_write_authority_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_memory_write_authority_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.durable_audit_rollback_write_authority_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "durable_audit_rollback_write_authority_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.service_inventory_side_effect_boundary_present {
        return recovery_lifeline_command_dispatch_check(
            "denied_missing_lifeline_command_dispatch_boundary",
            "recovery_service_inventory_side_effect_boundary_missing",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.command_dispatch_behavior_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            "recovery_lifeline_command_dispatch_behavior_not_implemented",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.executor_capability_table_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            "recovery_lifeline_command_executor_capability_table_not_implemented",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.side_effect_gate_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            "recovery_lifeline_command_side_effect_gate_not_implemented",
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_enablement_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_preflight_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_intent_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_commit_gate_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_result_denial_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_audit_denial_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_observation_denial_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    if !candidate.execution_completion_denial_present {
        return recovery_lifeline_command_dispatch_check(
            "defined_non_executable",
            RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE.not_implemented_reason,
            envelope_check,
            candidate,
            true,
        );
    }
    recovery_lifeline_command_dispatch_check(
        "defined_non_executable",
        "recovery_lifeline_command_dispatch_execution_disabled",
        envelope_check,
        candidate,
        true,
    )
}

pub(crate) fn recovery_lifeline_command_dispatch_check(
    status: &'static str,
    reason: &'static str,
    envelope_check: RecoveryLifelineCommandEnvelopeCheck,
    candidate: RecoveryLifelineCommandDispatchCandidate,
    command_envelope_reference_accepted: bool,
) -> RecoveryLifelineCommandDispatchCheck {
    RecoveryLifelineCommandDispatchCheck {
        status,
        reason,
        envelope_check,
        command_envelope_reference_available: candidate.command_envelope_reference_available,
        command_envelope_reference_accepted,
        command_body_canonicalization_present: candidate.command_body_canonicalization_present,
        command_handler_binding_present: candidate.command_handler_binding_present,
        status_read_handler_present: candidate.status_read_handler_present,
        rollback_preview_authorization_present: candidate.rollback_preview_authorization_present,
        rollback_apply_authorization_present: candidate.rollback_apply_authorization_present,
        disable_module_target_binding_present: candidate.disable_module_target_binding_present,
        restart_last_good_target_binding_present: candidate
            .restart_last_good_target_binding_present,
        load_artifact_by_hash_target_binding_present: candidate
            .load_artifact_by_hash_target_binding_present,
        recovery_memory_write_authority_present: candidate.recovery_memory_write_authority_present,
        durable_audit_rollback_write_authority_present: candidate
            .durable_audit_rollback_write_authority_present,
        service_inventory_side_effect_boundary_present: candidate
            .service_inventory_side_effect_boundary_present,
        command_dispatch_behavior_present: candidate.command_dispatch_behavior_present,
        executor_capability_table_present: candidate.executor_capability_table_present,
        side_effect_gate_present: candidate.side_effect_gate_present,
        execution_enablement_present: candidate.execution_enablement_present,
        execution_preflight_present: candidate.execution_preflight_present,
        execution_intent_present: candidate.execution_intent_present,
        execution_commit_gate_present: candidate.execution_commit_gate_present,
        execution_result_denial_present: candidate.execution_result_denial_present,
        execution_audit_denial_present: candidate.execution_audit_denial_present,
        execution_observation_denial_present: candidate.execution_observation_denial_present,
        execution_completion_denial_present: candidate.execution_completion_denial_present,
        execution_completion_denial_reference: candidate.execution_completion_denial_reference,
        accepts_lifeline_command_body: false,
        accepts_lifeline_command_envelope: false,
        dispatches_lifeline_command: false,
        command_execution_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
        recovery_memory_writes_enabled: false,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        provider_export_enabled: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

const RECOVERY_LIFELINE_STATUS_READ_READY_REASON: &str =
    "recovery_lifeline_status_read_ready_command_execution_disabled";

pub(crate) fn recovery_lifeline_status_execution_readiness_ready(
    check: &RecoveryLifelineCommandDispatchCheck,
    retained_status_handler_present: bool,
) -> bool {
    check.command_envelope_reference_accepted
        && check.command_body_canonicalization_present
        && check.command_handler_binding_present
        && check.status_read_handler_present
        && check.rollback_preview_authorization_present
        && check.rollback_apply_authorization_present
        && check.disable_module_target_binding_present
        && check.restart_last_good_target_binding_present
        && check.load_artifact_by_hash_target_binding_present
        && check.recovery_memory_write_authority_present
        && check.durable_audit_rollback_write_authority_present
        && check.service_inventory_side_effect_boundary_present
        && check.command_dispatch_behavior_present
        && check.executor_capability_table_present
        && check.side_effect_gate_present
        && check.execution_enablement_present
        && check.execution_preflight_present
        && check.execution_intent_present
        && check.execution_commit_gate_present
        && check.execution_result_denial_present
        && check.execution_audit_denial_present
        && check.execution_observation_denial_present
        && check.execution_completion_denial_present
        && retained_status_handler_present
}

pub(crate) fn recovery_lifeline_status_execution_readiness_reason(
    check: &RecoveryLifelineCommandDispatchCheck,
    retained_status_handler_present: bool,
) -> &'static str {
    if recovery_lifeline_status_execution_readiness_ready(check, retained_status_handler_present) {
        RECOVERY_LIFELINE_STATUS_READ_READY_REASON
    } else {
        check.reason
    }
}

pub(crate) fn recovery_lifeline_command_dispatch_valid_candidate(
) -> RecoveryLifelineCommandDispatchCandidate {
    RecoveryLifelineCommandDispatchCandidate {
        envelope_candidate: recovery_lifeline_command_envelope_valid_candidate(),
        command_envelope_reference_available: true,
        command_envelope_reference_current_boot: true,
        command_envelope_reference_schema_ok: true,
        command_envelope_reference_binding_ok: true,
        command_envelope_reference_binding_reason:
            "retained_recovery_lifeline_command_envelope_reference_valid",
        direct_openai_recovery_shortcut_used: false,
        command_body_canonicalization_present: true,
        command_handler_binding_present: true,
        status_read_handler_present: true,
        rollback_preview_authorization_present: true,
        rollback_apply_authorization_present: true,
        disable_module_target_binding_present: true,
        restart_last_good_target_binding_present: true,
        load_artifact_by_hash_target_binding_present: true,
        recovery_memory_write_authority_present: true,
        durable_audit_rollback_write_authority_present: true,
        service_inventory_side_effect_boundary_present: true,
        command_dispatch_behavior_present: true,
        executor_capability_table_present: true,
        side_effect_gate_present: true,
        execution_enablement_present: true,
        execution_preflight_present: true,
        execution_intent_present: true,
        execution_commit_gate_present: true,
        execution_result_denial_present: true,
        execution_audit_denial_present: true,
        execution_observation_denial_present: true,
        execution_completion_denial_present: true,
        execution_completion_denial_reference: None,
    }
}

#[derive(Clone, Copy)]
enum CommandDispatchMutation {
    None,
    Envelope(CommandEnvelopeMutation),
    MissingEnvelope,
    PreviousEnvelope,
    WrongEnvelope,
    SubstitutedEnvelope,
    MismatchedEnvelope,
    BodyMissing,
    HandlerMissing,
    StatusHandlerMissing,
    PreviewAuthMissing,
    ApplyAuthMissing,
    DisableTargetMissing,
    RestartTargetMissing,
    LoadHashTargetMissing,
    MemoryWriteMissing,
    DurableWriteMissing,
    ServiceSideEffectMissing,
    BehaviorMissing,
    ExecutorMissing,
    SideEffectMissing,
    ExecutionEnablementMissing,
    ExecutionPreflightMissing,
    ExecutionIntentMissing,
    ExecutionCommitGateMissing,
    ExecutionResultDenialMissing,
    ExecutionAuditDenialMissing,
    ExecutionObservationDenialMissing,
    ExecutionCompletionDenialMissing,
}

const fn dispatch_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CommandDispatchMutation,
) -> CaseSpec<CommandDispatchMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

fn evaluate_recovery_lifeline_command_dispatch_case(
    input: RecoveryLifelineCommandDispatchCandidate,
    _require_live_retained: bool,
) -> RecoveryLifelineCommandDispatchCheck {
    evaluate_recovery_lifeline_command_dispatch(input)
}

fn recovery_lifeline_command_dispatch_selftest_case_from_spec(
    spec: &CaseSpec<CommandDispatchMutation>,
    check: RecoveryLifelineCommandDispatchCheck,
) -> RecoveryLifelineCommandDispatchSelfTestCase {
    recovery_lifeline_command_dispatch_selftest_case(
        spec.name,
        spec.expected_status,
        spec.expected_reason,
        check,
    )
}

fn apply_command_dispatch_case(
    input: &mut RecoveryLifelineCommandDispatchCandidate,
    mutation: CommandDispatchMutation,
) {
    match mutation {
        CommandDispatchMutation::None => {}
        CommandDispatchMutation::Envelope(mutation) => {
            apply_command_envelope_case(&mut input.envelope_candidate, mutation)
        }
        CommandDispatchMutation::MissingEnvelope => {
            input.command_envelope_reference_available = false
        }
        CommandDispatchMutation::PreviousEnvelope => {
            input.command_envelope_reference_current_boot = false;
        }
        CommandDispatchMutation::WrongEnvelope => {
            input.command_envelope_reference_schema_ok = false
        }
        CommandDispatchMutation::SubstitutedEnvelope => {
            input.command_envelope_reference_binding_ok = false;
            input.command_envelope_reference_binding_reason =
                "recovery_lifeline_command_envelope_reference_substituted_record";
        }
        CommandDispatchMutation::MismatchedEnvelope => {
            input.command_envelope_reference_binding_ok = false;
            input.command_envelope_reference_binding_reason =
                "recovery_lifeline_command_envelope_reference_binding_mismatch";
        }
        CommandDispatchMutation::BodyMissing => input.command_body_canonicalization_present = false,
        CommandDispatchMutation::HandlerMissing => input.command_handler_binding_present = false,
        CommandDispatchMutation::StatusHandlerMissing => input.status_read_handler_present = false,
        CommandDispatchMutation::PreviewAuthMissing => {
            input.rollback_preview_authorization_present = false;
        }
        CommandDispatchMutation::ApplyAuthMissing => {
            input.rollback_apply_authorization_present = false;
        }
        CommandDispatchMutation::DisableTargetMissing => {
            input.disable_module_target_binding_present = false;
        }
        CommandDispatchMutation::RestartTargetMissing => {
            input.restart_last_good_target_binding_present = false;
        }
        CommandDispatchMutation::LoadHashTargetMissing => {
            input.load_artifact_by_hash_target_binding_present = false;
        }
        CommandDispatchMutation::MemoryWriteMissing => {
            input.recovery_memory_write_authority_present = false;
        }
        CommandDispatchMutation::DurableWriteMissing => {
            input.durable_audit_rollback_write_authority_present = false;
        }
        CommandDispatchMutation::ServiceSideEffectMissing => {
            input.service_inventory_side_effect_boundary_present = false;
        }
        CommandDispatchMutation::BehaviorMissing => input.command_dispatch_behavior_present = false,
        CommandDispatchMutation::ExecutorMissing => input.executor_capability_table_present = false,
        CommandDispatchMutation::SideEffectMissing => input.side_effect_gate_present = false,
        CommandDispatchMutation::ExecutionEnablementMissing => {
            input.execution_enablement_present = false;
        }
        CommandDispatchMutation::ExecutionPreflightMissing => {
            input.execution_preflight_present = false
        }
        CommandDispatchMutation::ExecutionIntentMissing => input.execution_intent_present = false,
        CommandDispatchMutation::ExecutionCommitGateMissing => {
            input.execution_commit_gate_present = false;
        }
        CommandDispatchMutation::ExecutionResultDenialMissing => {
            input.execution_result_denial_present = false;
        }
        CommandDispatchMutation::ExecutionAuditDenialMissing => {
            input.execution_audit_denial_present = false;
        }
        CommandDispatchMutation::ExecutionObservationDenialMissing => {
            input.execution_observation_denial_present = false;
        }
        CommandDispatchMutation::ExecutionCompletionDenialMissing => {
            input.execution_completion_denial_present = false;
        }
    }
}

const COMMAND_DISPATCH_CASES: [CaseSpec<CommandDispatchMutation>;
    RECOVERY_LIFELINE_COMMAND_DISPATCH_SELFTEST_CASES] = [
    dispatch_case(
        "missing_lifeline_request_event_id",
        "missing",
        "recovery_lifeline_request_event_id_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingRequest,
        )),
    ),
    dispatch_case(
        "stale_dropped_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_stale_or_dropped",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::StaleRequest,
        )),
    ),
    dispatch_case(
        "previous_boot_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_not_current_boot",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::PreviousRequest,
        )),
    ),
    dispatch_case(
        "wrong_schema_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_wrong_schema_or_variant",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::WrongSchemaRequest,
        )),
    ),
    dispatch_case(
        "substituted_lifeline_request_record",
        "rejected",
        "recovery_lifeline_request_substituted_record",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::SubstitutedRequest,
        )),
    ),
    dispatch_case(
        "lifeline_request_reference_hash_mismatch",
        "rejected",
        "recovery_lifeline_request_reference_hash_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::RequestHashMismatch,
        )),
    ),
    dispatch_case(
        "protocol_state_missing_after_valid_request",
        "denied_missing_lifeline_protocol_state",
        "recovery_lifeline_protocol_state_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingProtocolState,
        )),
    ),
    dispatch_case(
        "previous_boot_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_event_id_not_current_boot",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::PreviousProtocolState,
        )),
    ),
    dispatch_case(
        "wrong_schema_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_wrong_schema_or_variant",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::WrongSchemaProtocolState,
        )),
    ),
    dispatch_case(
        "substituted_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_substituted_record",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::SubstitutedProtocolState,
        )),
    ),
    dispatch_case(
        "command_vocabulary_missing_after_protocol_state",
        "denied_missing_lifeline_command_vocabulary",
        "recovery_lifeline_command_vocabulary_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingCommandVocabulary,
        )),
    ),
    dispatch_case(
        "previous_boot_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::PreviousCommandVocabulary,
        )),
    ),
    dispatch_case(
        "substituted_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_substituted_record",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::SubstitutedCommandVocabulary,
        )),
    ),
    dispatch_case(
        "loader_runtime_isolation_missing_after_command_vocabulary",
        "denied_missing_loader_runtime_isolation",
        "recovery_loader_runtime_isolation_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingLoaderRuntimeIsolation,
        )),
    ),
    dispatch_case(
        "mismatched_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_binding_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MismatchedLoaderRuntimeIsolation,
        )),
    ),
    dispatch_case(
        "rollback_transaction_engine_missing_after_loader",
        "denied_missing_rollback_transaction_engine",
        "recovery_rollback_transaction_engine_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingRollbackEngine,
        )),
    ),
    dispatch_case(
        "mismatched_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_binding_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MismatchedRollbackEngine,
        )),
    ),
    dispatch_case(
        "durable_persistence_boundary_missing_after_rollback_engine",
        "denied_missing_durable_audit_rollback_persistence",
        "durable_audit_rollback_persistence_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingDurablePersistence,
        )),
    ),
    dispatch_case(
        "mismatched_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_binding_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MismatchedDurablePersistence,
        )),
    ),
    dispatch_case(
        "recovery_memory_provenance_boundary_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_provenance_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MissingMemoryProvenanceBoundary,
        )),
    ),
    dispatch_case(
        "mismatched_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_binding_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::Admission(
            CommandAdmissionMutation::MismatchedMemoryProvenance,
        )),
    ),
    dispatch_case(
        "recovery_lifeline_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_lifeline_command_admission_missing",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::MissingAdmission),
    ),
    dispatch_case(
        "mismatched_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_binding_mismatch",
        CommandDispatchMutation::Envelope(CommandEnvelopeMutation::MismatchedAdmission),
    ),
    dispatch_case(
        "command_envelope_reference_missing",
        "denied_missing_lifeline_command_envelope_reference",
        "recovery_lifeline_command_envelope_reference_missing",
        CommandDispatchMutation::MissingEnvelope,
    ),
    dispatch_case(
        "previous_boot_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_event_id_not_current_boot",
        CommandDispatchMutation::PreviousEnvelope,
    ),
    dispatch_case(
        "wrong_schema_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_wrong_schema_or_variant",
        CommandDispatchMutation::WrongEnvelope,
    ),
    dispatch_case(
        "substituted_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_substituted_record",
        CommandDispatchMutation::SubstitutedEnvelope,
    ),
    dispatch_case(
        "mismatched_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_binding_mismatch",
        CommandDispatchMutation::MismatchedEnvelope,
    ),
    dispatch_case(
        "command_body_canonicalization_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_lifeline_command_body_canonicalization_missing",
        CommandDispatchMutation::BodyMissing,
    ),
    dispatch_case(
        "command_handler_binding_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_lifeline_command_handler_binding_missing",
        CommandDispatchMutation::HandlerMissing,
    ),
    dispatch_case(
        "status_read_handler_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_lifeline_status_read_handler_missing",
        CommandDispatchMutation::StatusHandlerMissing,
    ),
    dispatch_case(
        "rollback_preview_authorization_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_rollback_preview_authorization_missing",
        CommandDispatchMutation::PreviewAuthMissing,
    ),
    dispatch_case(
        "rollback_apply_authorization_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_rollback_apply_authorization_missing",
        CommandDispatchMutation::ApplyAuthMissing,
    ),
    dispatch_case(
        "disable_module_target_binding_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_disable_module_target_binding_missing",
        CommandDispatchMutation::DisableTargetMissing,
    ),
    dispatch_case(
        "restart_last_good_target_binding_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_restart_last_good_target_binding_missing",
        CommandDispatchMutation::RestartTargetMissing,
    ),
    dispatch_case(
        "load_artifact_by_hash_target_binding_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_load_artifact_by_hash_target_binding_missing",
        CommandDispatchMutation::LoadHashTargetMissing,
    ),
    dispatch_case(
        "recovery_memory_write_authority_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_memory_write_authority_missing",
        CommandDispatchMutation::MemoryWriteMissing,
    ),
    dispatch_case(
        "durable_audit_rollback_write_authority_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "durable_audit_rollback_write_authority_missing",
        CommandDispatchMutation::DurableWriteMissing,
    ),
    dispatch_case(
        "service_inventory_side_effect_boundary_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_service_inventory_side_effect_boundary_missing",
        CommandDispatchMutation::ServiceSideEffectMissing,
    ),
    dispatch_case(
        "command_dispatch_behavior_missing",
        "defined_non_executable",
        "recovery_lifeline_command_dispatch_behavior_not_implemented",
        CommandDispatchMutation::BehaviorMissing,
    ),
    dispatch_case(
        "executor_capability_table_missing",
        "defined_non_executable",
        "recovery_lifeline_command_executor_capability_table_not_implemented",
        CommandDispatchMutation::ExecutorMissing,
    ),
    dispatch_case(
        "side_effect_gate_missing",
        "defined_non_executable",
        "recovery_lifeline_command_side_effect_gate_not_implemented",
        CommandDispatchMutation::SideEffectMissing,
    ),
    dispatch_case(
        "execution_enablement_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_enablement_not_implemented",
        CommandDispatchMutation::ExecutionEnablementMissing,
    ),
    dispatch_case(
        "execution_preflight_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_preflight_not_implemented",
        CommandDispatchMutation::ExecutionPreflightMissing,
    ),
    dispatch_case(
        "execution_intent_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_intent_not_implemented",
        CommandDispatchMutation::ExecutionIntentMissing,
    ),
    dispatch_case(
        "execution_commit_gate_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_commit_gate_not_implemented",
        CommandDispatchMutation::ExecutionCommitGateMissing,
    ),
    dispatch_case(
        "execution_result_denial_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_result_denial_not_implemented",
        CommandDispatchMutation::ExecutionResultDenialMissing,
    ),
    dispatch_case(
        "execution_audit_denial_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_audit_denial_not_implemented",
        CommandDispatchMutation::ExecutionAuditDenialMissing,
    ),
    dispatch_case(
        "execution_observation_denial_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_observation_denial_not_implemented",
        CommandDispatchMutation::ExecutionObservationDenialMissing,
    ),
    dispatch_case(
        "execution_completion_denial_missing",
        "defined_non_executable",
        "recovery_lifeline_command_execution_completion_denial_not_implemented",
        CommandDispatchMutation::ExecutionCompletionDenialMissing,
    ),
    dispatch_case(
        "all_inputs_present_command_dispatch_still_non_executable",
        "defined_non_executable",
        "recovery_lifeline_command_dispatch_execution_disabled",
        CommandDispatchMutation::None,
    ),
];

pub(crate) fn recovery_lifeline_command_dispatch_selftest_cases(
) -> [RecoveryLifelineCommandDispatchSelfTestCase; RECOVERY_LIFELINE_COMMAND_DISPATCH_SELFTEST_CASES]
{
    run_selftest_cases_with(
        recovery_lifeline_command_dispatch_valid_candidate(),
        &COMMAND_DISPATCH_CASES,
        apply_command_dispatch_case,
        evaluate_recovery_lifeline_command_dispatch_case,
        recovery_lifeline_command_dispatch_selftest_case_from_spec,
    )
}
pub(crate) fn recovery_lifeline_command_dispatch_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandDispatchCheck,
) -> RecoveryLifelineCommandDispatchSelfTestCase {
    RecoveryLifelineCommandDispatchSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        actual_envelope_status: check.envelope_check.status,
        actual_envelope_reason: check.envelope_check.reason,
        command_envelope_reference_accepted: check.command_envelope_reference_accepted,
        command_body_canonicalization_present: check.command_body_canonicalization_present,
        command_handler_binding_present: check.command_handler_binding_present,
        dispatches_lifeline_command: check.dispatches_lifeline_command,
        command_execution_enabled: check.command_execution_enabled,
        load_attempted: check.load_attempted,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn evaluate_recovery_lifeline_command_body_canonicalization(
    candidate: RecoveryLifelineCommandBodyCanonicalizationCandidate,
) -> RecoveryLifelineCommandBodyCanonicalizationCheck {
    let dispatch_check = evaluate_recovery_lifeline_command_dispatch(candidate.dispatch_candidate);
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !method_eq(
        dispatch_check.status,
        "denied_missing_lifeline_command_dispatch_boundary",
    ) || !method_eq(
        dispatch_check.reason,
        "recovery_lifeline_command_body_canonicalization_missing",
    ) {
        return recovery_lifeline_command_body_canonicalization_check(
            dispatch_check.status,
            dispatch_check.reason,
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_body_reference_present {
        return recovery_lifeline_command_body_canonicalization_check(
            "missing",
            "recovery_lifeline_command_body_canonicalization_missing",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_body_reference_current_boot {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "retained_recovery_lifeline_command_body_canonicalization_event_id_not_current_boot",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_body_reference_schema_ok {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "recovery_lifeline_command_body_canonicalization_wrong_schema_or_variant",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_body_reference_binding_ok {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            candidate.command_body_reference_binding_reason,
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_id_supported {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.argument_schema_matches {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.argument_hash_present {
        return recovery_lifeline_command_body_canonicalization_check(
            "invalid_reference",
            "recovery_lifeline_command_body_canonicalization_invalid_hash",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.target_locator_present {
        return recovery_lifeline_command_body_canonicalization_check(
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_envelope_reference_hash_matches {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "recovery_lifeline_command_envelope_reference_hash_mismatch",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.command_dispatch_boundary_id_matches {
        return recovery_lifeline_command_body_canonicalization_check(
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            dispatch_check,
            candidate,
            false,
        );
    }
    if !candidate.body_hash_matches {
        return recovery_lifeline_command_body_canonicalization_check(
            "mismatched_command_body_canonicalization_hash",
            "recovery_lifeline_command_body_canonicalization_hash_mismatch",
            dispatch_check,
            candidate,
            false,
        );
    }
    recovery_lifeline_command_body_canonicalization_check(
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_body_canonicalization_valid_but_command_dispatch_disabled",
        dispatch_check,
        candidate,
        true,
    )
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_check(
    status: &'static str,
    reason: &'static str,
    dispatch_check: RecoveryLifelineCommandDispatchCheck,
    candidate: RecoveryLifelineCommandBodyCanonicalizationCandidate,
    command_body_reference_accepted: bool,
) -> RecoveryLifelineCommandBodyCanonicalizationCheck {
    RecoveryLifelineCommandBodyCanonicalizationCheck {
        status,
        reason,
        dispatch_check,
        command_body_reference_present: candidate.command_body_reference_present,
        command_body_reference_accepted,
        command_id_supported: candidate.command_id_supported,
        argument_schema_matches: candidate.argument_schema_matches,
        argument_hash_present: candidate.argument_hash_present,
        target_locator_present: candidate.target_locator_present,
        command_envelope_reference_hash_matches: candidate.command_envelope_reference_hash_matches,
        command_dispatch_boundary_id_matches: candidate.command_dispatch_boundary_id_matches,
        body_hash_matches: candidate.body_hash_matches,
        accepts_raw_command_body: false,
        accepts_lifeline_command_body: false,
        accepts_lifeline_command_envelope: false,
        dispatches_lifeline_command: false,
        command_execution_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
        recovery_memory_writes_enabled: false,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        provider_export_enabled: false,
        authorizes_recovery_load: false,
        can_move_beyond_denial: false,
        loads_recovery_loader: false,
        loads_recovery_artifact: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        allocates_service_slot: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_valid_candidate(
) -> RecoveryLifelineCommandBodyCanonicalizationCandidate {
    let mut dispatch_candidate = recovery_lifeline_command_dispatch_valid_candidate();
    dispatch_candidate.command_body_canonicalization_present = false;
    dispatch_candidate.command_handler_binding_present = false;
    dispatch_candidate.status_read_handler_present = false;
    dispatch_candidate.rollback_preview_authorization_present = false;
    dispatch_candidate.rollback_apply_authorization_present = false;
    dispatch_candidate.disable_module_target_binding_present = false;
    dispatch_candidate.restart_last_good_target_binding_present = false;
    dispatch_candidate.load_artifact_by_hash_target_binding_present = false;
    dispatch_candidate.recovery_memory_write_authority_present = false;
    dispatch_candidate.durable_audit_rollback_write_authority_present = false;
    dispatch_candidate.service_inventory_side_effect_boundary_present = false;
    RecoveryLifelineCommandBodyCanonicalizationCandidate {
        dispatch_candidate,
        command_body_reference_present: true,
        command_body_reference_current_boot: true,
        command_body_reference_schema_ok: true,
        command_body_reference_binding_ok: true,
        command_body_reference_binding_reason:
            "retained_recovery_lifeline_command_body_canonicalization_valid",
        command_id_supported: true,
        argument_schema_matches: true,
        argument_hash_present: true,
        target_locator_present: true,
        command_envelope_reference_hash_matches: true,
        command_dispatch_boundary_id_matches: true,
        body_hash_matches: true,
        direct_openai_recovery_shortcut_used: false,
    }
}

#[derive(Clone, Copy)]
enum CommandBodyMutation {
    None,
    Dispatch(CommandDispatchMutation),
    DirectProviderShortcut,
    DispatchMovedPastBody,
    BodyMissing,
    BodyPrevious,
    BodyWrongSchema,
    BodySubstituted,
    BodyMismatched,
    UnsupportedCommand,
    ArgumentSchemaMismatch,
    ArgumentHashMissing,
    TargetLocatorMissing,
    EnvelopeHashMismatch,
    DispatchBoundaryMismatch,
    BodyHashMismatch,
}

const fn body_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CommandBodyMutation,
) -> CaseSpec<CommandBodyMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

fn evaluate_recovery_lifeline_command_body_canonicalization_case(
    input: RecoveryLifelineCommandBodyCanonicalizationCandidate,
    _require_live_retained: bool,
) -> RecoveryLifelineCommandBodyCanonicalizationCheck {
    evaluate_recovery_lifeline_command_body_canonicalization(input)
}

fn recovery_lifeline_command_body_canonicalization_selftest_case_from_spec(
    spec: &CaseSpec<CommandBodyMutation>,
    check: RecoveryLifelineCommandBodyCanonicalizationCheck,
) -> RecoveryLifelineCommandBodyCanonicalizationSelfTestCase {
    recovery_lifeline_command_body_canonicalization_selftest_case(
        spec.name,
        spec.expected_status,
        spec.expected_reason,
        check,
    )
}

fn apply_command_body_case(
    input: &mut RecoveryLifelineCommandBodyCanonicalizationCandidate,
    mutation: CommandBodyMutation,
) {
    match mutation {
        CommandBodyMutation::None => {}
        CommandBodyMutation::Dispatch(mutation) => {
            apply_command_dispatch_case(&mut input.dispatch_candidate, mutation)
        }
        CommandBodyMutation::DirectProviderShortcut => {
            input.direct_openai_recovery_shortcut_used = true;
        }
        CommandBodyMutation::DispatchMovedPastBody => {
            input
                .dispatch_candidate
                .command_body_canonicalization_present = true;
        }
        CommandBodyMutation::BodyMissing => input.command_body_reference_present = false,
        CommandBodyMutation::BodyPrevious => input.command_body_reference_current_boot = false,
        CommandBodyMutation::BodyWrongSchema => input.command_body_reference_schema_ok = false,
        CommandBodyMutation::BodySubstituted => {
            input.command_body_reference_binding_ok = false;
            input.command_body_reference_binding_reason =
                "recovery_lifeline_command_body_canonicalization_substituted_record";
        }
        CommandBodyMutation::BodyMismatched => {
            input.command_body_reference_binding_ok = false;
            input.command_body_reference_binding_reason =
                "recovery_lifeline_command_body_canonicalization_binding_mismatch";
        }
        CommandBodyMutation::UnsupportedCommand => input.command_id_supported = false,
        CommandBodyMutation::ArgumentSchemaMismatch => input.argument_schema_matches = false,
        CommandBodyMutation::ArgumentHashMissing => input.argument_hash_present = false,
        CommandBodyMutation::TargetLocatorMissing => input.target_locator_present = false,
        CommandBodyMutation::EnvelopeHashMismatch => {
            input.command_envelope_reference_hash_matches = false;
        }
        CommandBodyMutation::DispatchBoundaryMismatch => {
            input.command_dispatch_boundary_id_matches = false;
        }
        CommandBodyMutation::BodyHashMismatch => input.body_hash_matches = false,
    }
}

const COMMAND_BODY_CASES: [CaseSpec<CommandBodyMutation>;
    RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_SELFTEST_CASES] = [
    body_case(
        "direct_openai_provider_path_rejected",
        "rejected",
        "direct_openai_provider_path_not_recovery_lifeline",
        CommandBodyMutation::DirectProviderShortcut,
    ),
    body_case(
        "missing_lifeline_request_event_id",
        "missing",
        "recovery_lifeline_request_event_id_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingRequest),
        )),
    ),
    body_case(
        "stale_dropped_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_stale_or_dropped",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::StaleRequest),
        )),
    ),
    body_case(
        "previous_boot_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_not_current_boot",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousRequest),
        )),
    ),
    body_case(
        "wrong_schema_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_wrong_schema_or_variant",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaRequest),
        )),
    ),
    body_case(
        "substituted_lifeline_request_record",
        "rejected",
        "recovery_lifeline_request_substituted_record",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedRequest),
        )),
    ),
    body_case(
        "lifeline_request_reference_hash_mismatch",
        "rejected",
        "recovery_lifeline_request_reference_hash_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::RequestHashMismatch),
        )),
    ),
    body_case(
        "protocol_state_missing_after_valid_request",
        "denied_missing_lifeline_protocol_state",
        "recovery_lifeline_protocol_state_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingProtocolState),
        )),
    ),
    body_case(
        "previous_boot_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_event_id_not_current_boot",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousProtocolState),
        )),
    ),
    body_case(
        "wrong_schema_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_wrong_schema_or_variant",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::WrongSchemaProtocolState),
        )),
    ),
    body_case(
        "substituted_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_substituted_record",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::SubstitutedProtocolState),
        )),
    ),
    body_case(
        "command_vocabulary_missing_after_protocol_state",
        "denied_missing_lifeline_command_vocabulary",
        "recovery_lifeline_command_vocabulary_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingCommandVocabulary),
        )),
    ),
    body_case(
        "previous_boot_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::PreviousCommandVocabulary),
        )),
    ),
    body_case(
        "substituted_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_substituted_record",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::SubstitutedCommandVocabulary,
            ),
        )),
    ),
    body_case(
        "loader_runtime_isolation_missing_after_command_vocabulary",
        "denied_missing_loader_runtime_isolation",
        "recovery_loader_runtime_isolation_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::MissingLoaderRuntimeIsolation,
            ),
        )),
    ),
    body_case(
        "mismatched_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::MismatchedLoaderRuntimeIsolation,
            ),
        )),
    ),
    body_case(
        "rollback_transaction_engine_missing_after_loader",
        "denied_missing_rollback_transaction_engine",
        "recovery_rollback_transaction_engine_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingRollbackEngine),
        )),
    ),
    body_case(
        "mismatched_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MismatchedRollbackEngine),
        )),
    ),
    body_case(
        "durable_persistence_boundary_missing_after_rollback_engine",
        "denied_missing_durable_audit_rollback_persistence",
        "durable_audit_rollback_persistence_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(CommandAdmissionMutation::MissingDurablePersistence),
        )),
    ),
    body_case(
        "mismatched_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::MismatchedDurablePersistence,
            ),
        )),
    ),
    body_case(
        "recovery_memory_provenance_boundary_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_provenance_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::MissingMemoryProvenanceBoundary,
            ),
        )),
    ),
    body_case(
        "mismatched_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::Admission(
                CommandAdmissionMutation::MismatchedMemoryProvenance,
            ),
        )),
    ),
    body_case(
        "recovery_lifeline_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_lifeline_command_admission_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::MissingAdmission,
        )),
    ),
    body_case(
        "mismatched_recovery_lifeline_command_admission",
        "rejected",
        "recovery_lifeline_command_admission_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::Envelope(
            CommandEnvelopeMutation::MismatchedAdmission,
        )),
    ),
    body_case(
        "command_envelope_reference_missing",
        "denied_missing_lifeline_command_envelope_reference",
        "recovery_lifeline_command_envelope_reference_missing",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::MissingEnvelope),
    ),
    body_case(
        "previous_boot_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_event_id_not_current_boot",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::PreviousEnvelope),
    ),
    body_case(
        "wrong_schema_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_wrong_schema_or_variant",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::WrongEnvelope),
    ),
    body_case(
        "substituted_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_substituted_record",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::SubstitutedEnvelope),
    ),
    body_case(
        "mismatched_command_envelope_reference",
        "rejected",
        "recovery_lifeline_command_envelope_reference_binding_mismatch",
        CommandBodyMutation::Dispatch(CommandDispatchMutation::MismatchedEnvelope),
    ),
    body_case(
        "dispatch_boundary_not_body_missing",
        "denied_missing_lifeline_command_dispatch_boundary",
        "recovery_lifeline_command_handler_binding_missing",
        CommandBodyMutation::DispatchMovedPastBody,
    ),
    body_case(
        "command_body_canonicalization_missing",
        "missing",
        "recovery_lifeline_command_body_canonicalization_missing",
        CommandBodyMutation::BodyMissing,
    ),
    body_case(
        "previous_boot_command_body_canonicalization_reference",
        "rejected",
        "retained_recovery_lifeline_command_body_canonicalization_event_id_not_current_boot",
        CommandBodyMutation::BodyPrevious,
    ),
    body_case(
        "wrong_schema_command_body_canonicalization_reference",
        "rejected",
        "recovery_lifeline_command_body_canonicalization_wrong_schema_or_variant",
        CommandBodyMutation::BodyWrongSchema,
    ),
    body_case(
        "substituted_command_body_canonicalization_reference",
        "rejected",
        "recovery_lifeline_command_body_canonicalization_substituted_record",
        CommandBodyMutation::BodySubstituted,
    ),
    body_case(
        "mismatched_command_body_canonicalization_reference",
        "rejected",
        "recovery_lifeline_command_body_canonicalization_binding_mismatch",
        CommandBodyMutation::BodyMismatched,
    ),
    body_case(
        "unsupported_command_id",
        "rejected",
        "recovery_lifeline_command_id_unsupported",
        CommandBodyMutation::UnsupportedCommand,
    ),
    body_case(
        "argument_schema_mismatch",
        "rejected",
        "recovery_lifeline_command_argument_schema_mismatch",
        CommandBodyMutation::ArgumentSchemaMismatch,
    ),
    body_case(
        "argument_hash_missing",
        "invalid_reference",
        "recovery_lifeline_command_body_canonicalization_invalid_hash",
        CommandBodyMutation::ArgumentHashMissing,
    ),
    body_case(
        "target_locator_missing",
        "invalid_reference",
        "recovery_lifeline_command_target_locator_invalid",
        CommandBodyMutation::TargetLocatorMissing,
    ),
    body_case(
        "command_envelope_reference_hash_mismatch",
        "rejected",
        "recovery_lifeline_command_envelope_reference_hash_mismatch",
        CommandBodyMutation::EnvelopeHashMismatch,
    ),
    body_case(
        "command_dispatch_boundary_id_mismatch",
        "rejected",
        "recovery_lifeline_command_dispatch_boundary_mismatch",
        CommandBodyMutation::DispatchBoundaryMismatch,
    ),
    body_case(
        "command_body_canonicalization_hash_mismatch",
        "mismatched_command_body_canonicalization_hash",
        "recovery_lifeline_command_body_canonicalization_hash_mismatch",
        CommandBodyMutation::BodyHashMismatch,
    ),
    body_case(
        "all_inputs_present_command_body_canonicalization_still_non_executable",
        "valid_hash_reference_command_still_denied",
        "recovery_lifeline_command_body_canonicalization_valid_but_command_dispatch_disabled",
        CommandBodyMutation::None,
    ),
];

pub(crate) fn recovery_lifeline_command_body_canonicalization_selftest_cases(
) -> [RecoveryLifelineCommandBodyCanonicalizationSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_SELFTEST_CASES] {
    run_selftest_cases_with(
        recovery_lifeline_command_body_canonicalization_valid_candidate(),
        &COMMAND_BODY_CASES,
        apply_command_body_case,
        evaluate_recovery_lifeline_command_body_canonicalization_case,
        recovery_lifeline_command_body_canonicalization_selftest_case_from_spec,
    )
}
pub(crate) fn recovery_lifeline_command_body_canonicalization_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandBodyCanonicalizationCheck,
) -> RecoveryLifelineCommandBodyCanonicalizationSelfTestCase {
    let _input_flags = (
        check.command_body_reference_present,
        check.command_id_supported,
        check.argument_schema_matches,
        check.argument_hash_present,
        check.target_locator_present,
        check.command_envelope_reference_hash_matches,
        check.command_dispatch_boundary_id_matches,
    );
    let non_authorizing = !check.accepts_raw_command_body
        && !check.accepts_lifeline_command_body
        && !check.accepts_lifeline_command_envelope
        && !check.rollback_preview_enabled
        && !check.rollback_apply_enabled
        && !check.recovery_memory_writes_enabled
        && !check.durable_writes_enabled
        && !check.rollback_replay_enabled
        && !check.provider_export_enabled
        && !check.authorizes_recovery_load
        && !check.can_move_beyond_denial
        && !check.loads_recovery_loader
        && !check.loads_recovery_artifact
        && !check.creates_durable_records
        && !check.installs_rollback_plan
        && !check.allocates_service_slot
        && method_eq(check.service_inventory_change, "none");
    RecoveryLifelineCommandBodyCanonicalizationSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        actual_dispatch_status: check.dispatch_check.status,
        actual_dispatch_reason: check.dispatch_check.reason,
        command_body_reference_accepted: check.command_body_reference_accepted,
        body_hash_matches: check.body_hash_matches,
        dispatches_lifeline_command: check.dispatches_lifeline_command,
        command_execution_enabled: check.command_execution_enabled,
        load_attempted: check.load_attempted,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && non_authorizing,
    }
}
