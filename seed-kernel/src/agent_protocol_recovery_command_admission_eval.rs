use crate::{
    agent_protocol_recovery_constants::RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_memory_provenance_eval::*,
    agent_protocol_recovery_runtime_types::*,
    agent_protocol_support::{method_eq, run_selftest_cases_with, CaseSpec},
};

pub(crate) fn recovery_lifeline_command_admission_candidate_from_memory(
    memory_candidate: RecoveryMemoryProvenanceCandidate,
) -> RecoveryLifelineCommandAdmissionCandidate {
    RecoveryLifelineCommandAdmissionCandidate {
        memory_candidate,
        recovery_memory_provenance_available: true,
        recovery_memory_provenance_current_boot: true,
        recovery_memory_provenance_schema_ok: true,
        recovery_memory_provenance_binding_ok: true,
        recovery_memory_provenance_binding_reason: "recovery_memory_provenance_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        bindings: CommandBindings::empty(),
    }
}

pub(crate) fn evaluate_recovery_lifeline_command_admission(
    candidate: RecoveryLifelineCommandAdmissionCandidate,
) -> RecoveryLifelineCommandAdmissionCheck {
    let memory_check = evaluate_recovery_memory_provenance(candidate.memory_candidate);
    let memory_boundary_exposed = memory_check.memory_provenance_requirements_exposed
        || candidate.recovery_memory_provenance_available;

    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .persistence_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .persistence_candidate
            .transaction_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .memory_candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_lifeline_command_admission_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            memory_check,
            memory_boundary_exposed,
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
    if !memory_check.recovery_memory_provenance_ready {
        return recovery_lifeline_command_admission_check(
            memory_check.status,
            memory_check.reason,
            memory_check,
            memory_boundary_exposed,
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
    if !candidate.recovery_memory_provenance_available {
        return recovery_lifeline_command_admission_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            memory_check,
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
    if !candidate.recovery_memory_provenance_current_boot {
        return recovery_lifeline_command_admission_check(
            "rejected",
            "recovery_memory_provenance_event_id_not_current_boot",
            memory_check,
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
    if !candidate.recovery_memory_provenance_schema_ok {
        return recovery_lifeline_command_admission_check(
            "rejected",
            "recovery_memory_provenance_wrong_schema_or_variant",
            memory_check,
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
    if !candidate.recovery_memory_provenance_binding_ok {
        return recovery_lifeline_command_admission_check(
            "rejected",
            candidate.recovery_memory_provenance_binding_reason,
            memory_check,
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

    if !candidate.lifeline_status_admission_present
        && !candidate.rollback_preview_admission_present
        && !candidate.rollback_apply_admission_present
        && !candidate.disable_module_admission_present
        && !candidate.restart_last_good_admission_present
        && !candidate.load_recovery_artifact_by_hash_admission_present
    {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_lifeline_command_admission_requirements_missing",
            memory_check,
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
    if !candidate.lifeline_status_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_lifeline_status_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            false,
            candidate.rollback_preview_admission_present,
            candidate.rollback_apply_admission_present,
            candidate.disable_module_admission_present,
            candidate.restart_last_good_admission_present,
            candidate.load_recovery_artifact_by_hash_admission_present,
        );
    }
    if !candidate.rollback_preview_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_rollback_preview_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            true,
            false,
            candidate.rollback_apply_admission_present,
            candidate.disable_module_admission_present,
            candidate.restart_last_good_admission_present,
            candidate.load_recovery_artifact_by_hash_admission_present,
        );
    }
    if !candidate.rollback_apply_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_rollback_apply_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            true,
            true,
            false,
            candidate.disable_module_admission_present,
            candidate.restart_last_good_admission_present,
            candidate.load_recovery_artifact_by_hash_admission_present,
        );
    }
    if !candidate.disable_module_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_disable_module_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            false,
            candidate.restart_last_good_admission_present,
            candidate.load_recovery_artifact_by_hash_admission_present,
        );
    }
    if !candidate.restart_last_good_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_restart_last_good_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            false,
            candidate.load_recovery_artifact_by_hash_admission_present,
        );
    }
    if !candidate.load_recovery_artifact_by_hash_admission_present {
        return recovery_lifeline_command_admission_check(
            "denied_missing_lifeline_command_admission",
            "recovery_load_artifact_by_hash_command_admission_missing",
            memory_check,
            true,
            true,
            true,
            false,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }

    recovery_lifeline_command_admission_check(
        "defined_non_executable",
        "recovery_lifeline_command_admission_behavior_not_implemented",
        memory_check,
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

pub(crate) fn recovery_lifeline_command_admission_check(
    status: &'static str,
    reason: &'static str,
    memory_check: RecoveryMemoryProvenanceCheck,
    recovery_memory_provenance_boundary_exposed: bool,
    recovery_memory_provenance_accepted: bool,
    command_admission_requirements_exposed: bool,
    command_admission_ready: bool,
    lifeline_status_admission_present: bool,
    rollback_preview_admission_present: bool,
    rollback_apply_admission_present: bool,
    disable_module_admission_present: bool,
    restart_last_good_admission_present: bool,
    load_recovery_artifact_by_hash_admission_present: bool,
) -> RecoveryLifelineCommandAdmissionCheck {
    RecoveryLifelineCommandAdmissionCheck {
        status,
        reason,
        memory_check,
        recovery_memory_provenance_boundary_exposed,
        recovery_memory_provenance_accepted,
        command_admission_requirements_exposed,
        command_admission_ready,
        bindings: CommandBindings {
            lifeline_status_admission_present,
            rollback_preview_admission_present,
            rollback_apply_admission_present,
            disable_module_admission_present,
            restart_last_good_admission_present,
            load_recovery_artifact_by_hash_admission_present,
            ..CommandBindings::empty()
        },
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
        service_inventory_change: "none",
        load_attempted: false,
    }
}

pub(crate) fn recovery_lifeline_command_admission_valid_candidate(
) -> RecoveryLifelineCommandAdmissionCandidate {
    RecoveryLifelineCommandAdmissionCandidate {
        memory_candidate: recovery_memory_provenance_valid_candidate(),
        recovery_memory_provenance_available: true,
        recovery_memory_provenance_current_boot: true,
        recovery_memory_provenance_schema_ok: true,
        recovery_memory_provenance_binding_ok: true,
        recovery_memory_provenance_binding_reason: "retained_recovery_memory_provenance_valid",
        direct_openai_recovery_shortcut_used: false,
        bindings: CommandBindings {
            lifeline_status_admission_present: true,
            rollback_preview_admission_present: true,
            rollback_apply_admission_present: true,
            disable_module_admission_present: true,
            restart_last_good_admission_present: true,
            load_recovery_artifact_by_hash_admission_present: true,
            ..CommandBindings::empty()
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CommandAdmissionMutation {
    None,
    MissingRequest,
    StaleRequest,
    PreviousRequest,
    WrongSchemaRequest,
    SubstitutedRequest,
    RequestHashMismatch,
    MissingProtocolState,
    PreviousProtocolState,
    WrongSchemaProtocolState,
    SubstitutedProtocolState,
    MissingCommandVocabulary,
    PreviousCommandVocabulary,
    WrongSchemaCommandVocabulary,
    SubstitutedCommandVocabulary,
    DirectProviderShortcut,
    MissingLoaderRuntimeIsolation,
    PreviousLoaderRuntimeIsolation,
    WrongSchemaLoaderRuntimeIsolation,
    SubstitutedLoaderRuntimeIsolation,
    MismatchedLoaderRuntimeIsolation,
    MissingRollbackEngine,
    PreviousRollbackEngine,
    WrongSchemaRollbackEngine,
    SubstitutedRollbackEngine,
    MismatchedRollbackEngine,
    MissingDurablePersistence,
    PreviousDurablePersistence,
    WrongSchemaDurablePersistence,
    SubstitutedDurablePersistence,
    MismatchedDurablePersistence,
    MissingMemoryProvenanceBoundary,
    PreviousMemoryProvenance,
    WrongSchemaMemoryProvenance,
    SubstitutedMemoryProvenance,
    MismatchedMemoryProvenance,
    MemoryFactsMissing,
    MemoryAuditLinkageMissing,
    AllAdmissionMissing,
    StatusAdmissionMissing,
    PreviewAdmissionMissing,
    ApplyAdmissionMissing,
    DisableAdmissionMissing,
    RestartAdmissionMissing,
    LoadByHashAdmissionMissing,
}

const fn admission_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: CommandAdmissionMutation,
) -> CaseSpec<CommandAdmissionMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

fn evaluate_recovery_lifeline_command_admission_case(
    input: RecoveryLifelineCommandAdmissionCandidate,
    _require_live_retained: bool,
) -> RecoveryLifelineCommandAdmissionCheck {
    evaluate_recovery_lifeline_command_admission(input)
}

fn recovery_lifeline_command_admission_selftest_case_from_spec(
    spec: &CaseSpec<CommandAdmissionMutation>,
    check: RecoveryLifelineCommandAdmissionCheck,
) -> RecoveryLifelineCommandAdmissionSelfTestCase {
    recovery_lifeline_command_admission_selftest_case(
        spec.name,
        spec.expected_status,
        spec.expected_reason,
        check,
    )
}

pub(crate) fn apply_command_admission_case(
    input: &mut RecoveryLifelineCommandAdmissionCandidate,
    mutation: CommandAdmissionMutation,
) {
    match mutation {
        CommandAdmissionMutation::None => {}
        CommandAdmissionMutation::MissingRequest => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
        }
        CommandAdmissionMutation::StaleRequest => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
        }
        CommandAdmissionMutation::PreviousRequest => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaRequest => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedRequest => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_reason = "recovery_lifeline_request_substituted_record";
        }
        CommandAdmissionMutation::RequestHashMismatch => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_candidate
                .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";
        }
        CommandAdmissionMutation::MissingProtocolState => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_retained = false;
        }
        CommandAdmissionMutation::PreviousProtocolState => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaProtocolState => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedProtocolState => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason =
                "recovery_lifeline_protocol_state_substituted_record";
        }
        CommandAdmissionMutation::MissingCommandVocabulary => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_available = false;
        }
        CommandAdmissionMutation::PreviousCommandVocabulary => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaCommandVocabulary => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedCommandVocabulary => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_reason =
                "recovery_lifeline_command_vocabulary_substituted_record";
        }
        CommandAdmissionMutation::DirectProviderShortcut => {
            input.direct_openai_recovery_shortcut_used = true;
        }
        CommandAdmissionMutation::MissingLoaderRuntimeIsolation => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_available = false;
        }
        CommandAdmissionMutation::PreviousLoaderRuntimeIsolation => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaLoaderRuntimeIsolation => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedLoaderRuntimeIsolation => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason =
                "recovery_loader_runtime_isolation_substituted_record";
        }
        CommandAdmissionMutation::MismatchedLoaderRuntimeIsolation => {
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason =
                "recovery_loader_runtime_isolation_binding_mismatch";
        }
        CommandAdmissionMutation::MissingRollbackEngine => {
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_available = false;
        }
        CommandAdmissionMutation::PreviousRollbackEngine => {
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaRollbackEngine => {
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedRollbackEngine => {
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_reason =
                "recovery_rollback_transaction_engine_substituted_record";
        }
        CommandAdmissionMutation::MismatchedRollbackEngine => {
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_ok = false;
            input
                .memory_candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_reason =
                "recovery_rollback_transaction_engine_binding_mismatch";
        }
        CommandAdmissionMutation::MissingDurablePersistence => {
            input
                .memory_candidate
                .durable_audit_rollback_persistence_available = false;
        }
        CommandAdmissionMutation::PreviousDurablePersistence => {
            input
                .memory_candidate
                .durable_audit_rollback_persistence_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaDurablePersistence => {
            input
                .memory_candidate
                .durable_audit_rollback_persistence_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedDurablePersistence => {
            input
                .memory_candidate
                .durable_audit_rollback_persistence_binding_ok = false;
            input
                .memory_candidate
                .durable_audit_rollback_persistence_binding_reason =
                "durable_audit_rollback_persistence_substituted_record";
        }
        CommandAdmissionMutation::MismatchedDurablePersistence => {
            input
                .memory_candidate
                .durable_audit_rollback_persistence_binding_ok = false;
            input
                .memory_candidate
                .durable_audit_rollback_persistence_binding_reason =
                "durable_audit_rollback_persistence_binding_mismatch";
        }
        CommandAdmissionMutation::MissingMemoryProvenanceBoundary => {
            input.recovery_memory_provenance_available = false;
        }
        CommandAdmissionMutation::PreviousMemoryProvenance => {
            input.recovery_memory_provenance_current_boot = false;
        }
        CommandAdmissionMutation::WrongSchemaMemoryProvenance => {
            input.recovery_memory_provenance_schema_ok = false;
        }
        CommandAdmissionMutation::SubstitutedMemoryProvenance => {
            input.recovery_memory_provenance_binding_ok = false;
            input.recovery_memory_provenance_binding_reason =
                "recovery_memory_provenance_substituted_record";
        }
        CommandAdmissionMutation::MismatchedMemoryProvenance => {
            input.recovery_memory_provenance_binding_ok = false;
            input.recovery_memory_provenance_binding_reason =
                "recovery_memory_provenance_binding_mismatch";
        }
        CommandAdmissionMutation::MemoryFactsMissing => {
            input.memory_candidate.source_record_ids_present = false;
            input.memory_candidate.source_schema_hashes_present = false;
            input.memory_candidate.memory_classification_present = false;
            input.memory_candidate.memory_authority_level_present = false;
            input
                .memory_candidate
                .memory_rollback_transaction_binding_present = false;
            input
                .memory_candidate
                .memory_last_good_checkpoint_binding_present = false;
            input.memory_candidate.recovery_only_export_profile_present = false;
            input.memory_candidate.memory_redaction_state_present = false;
            input.memory_candidate.memory_replay_window_present = false;
            input.memory_candidate.memory_audit_linkage_present = false;
        }
        CommandAdmissionMutation::MemoryAuditLinkageMissing => {
            input.memory_candidate.memory_audit_linkage_present = false;
        }
        CommandAdmissionMutation::AllAdmissionMissing => {
            input.lifeline_status_admission_present = false;
            input.rollback_preview_admission_present = false;
            input.rollback_apply_admission_present = false;
            input.disable_module_admission_present = false;
            input.restart_last_good_admission_present = false;
            input.load_recovery_artifact_by_hash_admission_present = false;
        }
        CommandAdmissionMutation::StatusAdmissionMissing => {
            input.lifeline_status_admission_present = false;
        }
        CommandAdmissionMutation::PreviewAdmissionMissing => {
            input.rollback_preview_admission_present = false;
        }
        CommandAdmissionMutation::ApplyAdmissionMissing => {
            input.rollback_apply_admission_present = false;
        }
        CommandAdmissionMutation::DisableAdmissionMissing => {
            input.disable_module_admission_present = false;
        }
        CommandAdmissionMutation::RestartAdmissionMissing => {
            input.restart_last_good_admission_present = false;
        }
        CommandAdmissionMutation::LoadByHashAdmissionMissing => {
            input.load_recovery_artifact_by_hash_admission_present = false;
        }
    }
}

const COMMAND_ADMISSION_CASES: [CaseSpec<CommandAdmissionMutation>;
    RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES] = [
    admission_case(
        "missing_lifeline_request_event_id",
        "missing",
        "recovery_lifeline_request_event_id_missing",
        CommandAdmissionMutation::MissingRequest,
    ),
    admission_case(
        "stale_dropped_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_stale_or_dropped",
        CommandAdmissionMutation::StaleRequest,
    ),
    admission_case(
        "previous_boot_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousRequest,
    ),
    admission_case(
        "wrong_schema_lifeline_request_event_id",
        "rejected",
        "recovery_lifeline_request_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaRequest,
    ),
    admission_case(
        "substituted_lifeline_request_record",
        "rejected",
        "recovery_lifeline_request_substituted_record",
        CommandAdmissionMutation::SubstitutedRequest,
    ),
    admission_case(
        "lifeline_request_reference_hash_mismatch",
        "rejected",
        "recovery_lifeline_request_reference_hash_mismatch",
        CommandAdmissionMutation::RequestHashMismatch,
    ),
    admission_case(
        "protocol_state_missing_after_valid_request",
        "denied_missing_lifeline_protocol_state",
        "recovery_lifeline_protocol_state_missing",
        CommandAdmissionMutation::MissingProtocolState,
    ),
    admission_case(
        "previous_boot_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousProtocolState,
    ),
    admission_case(
        "wrong_schema_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaProtocolState,
    ),
    admission_case(
        "substituted_lifeline_protocol_state",
        "rejected",
        "recovery_lifeline_protocol_state_substituted_record",
        CommandAdmissionMutation::SubstitutedProtocolState,
    ),
    admission_case(
        "command_vocabulary_missing_after_protocol_state",
        "denied_missing_lifeline_command_vocabulary",
        "recovery_lifeline_command_vocabulary_missing",
        CommandAdmissionMutation::MissingCommandVocabulary,
    ),
    admission_case(
        "previous_boot_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousCommandVocabulary,
    ),
    admission_case(
        "wrong_schema_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaCommandVocabulary,
    ),
    admission_case(
        "substituted_lifeline_command_vocabulary",
        "rejected",
        "recovery_lifeline_command_vocabulary_substituted_record",
        CommandAdmissionMutation::SubstitutedCommandVocabulary,
    ),
    admission_case(
        "direct_openai_recovery_shortcut_rejected",
        "rejected",
        "direct_openai_provider_path_not_recovery_lifeline",
        CommandAdmissionMutation::DirectProviderShortcut,
    ),
    admission_case(
        "loader_runtime_isolation_missing_after_command_vocabulary",
        "denied_missing_loader_runtime_isolation",
        "recovery_loader_runtime_isolation_missing",
        CommandAdmissionMutation::MissingLoaderRuntimeIsolation,
    ),
    admission_case(
        "previous_boot_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousLoaderRuntimeIsolation,
    ),
    admission_case(
        "wrong_schema_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaLoaderRuntimeIsolation,
    ),
    admission_case(
        "substituted_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_substituted_record",
        CommandAdmissionMutation::SubstitutedLoaderRuntimeIsolation,
    ),
    admission_case(
        "mismatched_loader_runtime_isolation",
        "rejected",
        "recovery_loader_runtime_isolation_binding_mismatch",
        CommandAdmissionMutation::MismatchedLoaderRuntimeIsolation,
    ),
    admission_case(
        "rollback_transaction_engine_missing_after_loader",
        "denied_missing_rollback_transaction_engine",
        "recovery_rollback_transaction_engine_missing",
        CommandAdmissionMutation::MissingRollbackEngine,
    ),
    admission_case(
        "previous_boot_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousRollbackEngine,
    ),
    admission_case(
        "wrong_schema_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaRollbackEngine,
    ),
    admission_case(
        "substituted_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_substituted_record",
        CommandAdmissionMutation::SubstitutedRollbackEngine,
    ),
    admission_case(
        "mismatched_rollback_transaction_engine",
        "rejected",
        "recovery_rollback_transaction_engine_binding_mismatch",
        CommandAdmissionMutation::MismatchedRollbackEngine,
    ),
    admission_case(
        "durable_persistence_boundary_missing_after_rollback_engine",
        "denied_missing_durable_audit_rollback_persistence",
        "durable_audit_rollback_persistence_missing",
        CommandAdmissionMutation::MissingDurablePersistence,
    ),
    admission_case(
        "previous_boot_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousDurablePersistence,
    ),
    admission_case(
        "wrong_schema_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaDurablePersistence,
    ),
    admission_case(
        "substituted_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_substituted_record",
        CommandAdmissionMutation::SubstitutedDurablePersistence,
    ),
    admission_case(
        "mismatched_durable_persistence",
        "rejected",
        "durable_audit_rollback_persistence_binding_mismatch",
        CommandAdmissionMutation::MismatchedDurablePersistence,
    ),
    admission_case(
        "recovery_memory_provenance_boundary_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_provenance_missing",
        CommandAdmissionMutation::MissingMemoryProvenanceBoundary,
    ),
    admission_case(
        "previous_boot_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_event_id_not_current_boot",
        CommandAdmissionMutation::PreviousMemoryProvenance,
    ),
    admission_case(
        "wrong_schema_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_wrong_schema_or_variant",
        CommandAdmissionMutation::WrongSchemaMemoryProvenance,
    ),
    admission_case(
        "substituted_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_substituted_record",
        CommandAdmissionMutation::SubstitutedMemoryProvenance,
    ),
    admission_case(
        "mismatched_recovery_memory_provenance",
        "rejected",
        "recovery_memory_provenance_binding_mismatch",
        CommandAdmissionMutation::MismatchedMemoryProvenance,
    ),
    admission_case(
        "recovery_memory_provenance_facts_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_provenance_missing",
        CommandAdmissionMutation::MemoryFactsMissing,
    ),
    admission_case(
        "recovery_memory_audit_linkage_missing",
        "denied_missing_recovery_memory_provenance",
        "recovery_memory_audit_linkage_missing",
        CommandAdmissionMutation::MemoryAuditLinkageMissing,
    ),
    admission_case(
        "command_admission_requirements_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_lifeline_command_admission_requirements_missing",
        CommandAdmissionMutation::AllAdmissionMissing,
    ),
    admission_case(
        "lifeline_status_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_lifeline_status_command_admission_missing",
        CommandAdmissionMutation::StatusAdmissionMissing,
    ),
    admission_case(
        "rollback_preview_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_rollback_preview_command_admission_missing",
        CommandAdmissionMutation::PreviewAdmissionMissing,
    ),
    admission_case(
        "rollback_apply_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_rollback_apply_command_admission_missing",
        CommandAdmissionMutation::ApplyAdmissionMissing,
    ),
    admission_case(
        "disable_module_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_disable_module_command_admission_missing",
        CommandAdmissionMutation::DisableAdmissionMissing,
    ),
    admission_case(
        "restart_last_good_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_restart_last_good_command_admission_missing",
        CommandAdmissionMutation::RestartAdmissionMissing,
    ),
    admission_case(
        "load_artifact_by_hash_command_admission_missing",
        "denied_missing_lifeline_command_admission",
        "recovery_load_artifact_by_hash_command_admission_missing",
        CommandAdmissionMutation::LoadByHashAdmissionMissing,
    ),
    admission_case(
        "all_inputs_present_command_admission_still_non_executable",
        "defined_non_executable",
        "recovery_lifeline_command_admission_behavior_not_implemented",
        CommandAdmissionMutation::None,
    ),
];

pub(crate) fn recovery_lifeline_command_admission_selftest_cases(
) -> [RecoveryLifelineCommandAdmissionSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES] {
    run_selftest_cases_with(
        recovery_lifeline_command_admission_valid_candidate(),
        &COMMAND_ADMISSION_CASES,
        apply_command_admission_case,
        evaluate_recovery_lifeline_command_admission_case,
        recovery_lifeline_command_admission_selftest_case_from_spec,
    )
}
pub(crate) fn recovery_lifeline_command_admission_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandAdmissionCheck,
) -> RecoveryLifelineCommandAdmissionSelfTestCase {
    RecoveryLifelineCommandAdmissionSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}
