use crate::{
    agent_protocol_recovery_constants::RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_memory_provenance_eval::*, agent_protocol_recovery_runtime_types::*,
    agent_protocol_support::method_eq,
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
        lifeline_status_admission_present: false,
        rollback_preview_admission_present: false,
        rollback_apply_admission_present: false,
        disable_module_admission_present: false,
        restart_last_good_admission_present: false,
        load_recovery_artifact_by_hash_admission_present: false,
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
        lifeline_status_admission_present,
        rollback_preview_admission_present,
        rollback_apply_admission_present,
        disable_module_admission_present,
        restart_last_good_admission_present,
        load_recovery_artifact_by_hash_admission_present,
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
        lifeline_status_admission_present: true,
        rollback_preview_admission_present: true,
        rollback_apply_admission_present: true,
        disable_module_admission_present: true,
        restart_last_good_admission_present: true,
        load_recovery_artifact_by_hash_admission_present: true,
    }
}

pub(crate) fn recovery_lifeline_command_admission_selftest_cases(
) -> [RecoveryLifelineCommandAdmissionSelfTestCase;
       RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES] {
    let valid = recovery_lifeline_command_admission_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";
    let mut mismatched_loader_runtime_isolation = valid;
    mismatched_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    mismatched_loader_runtime_isolation
        .memory_candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_binding_mismatch";

    let mut missing_rollback_engine = valid;
    missing_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_available = false;
    let mut previous_rollback_engine = valid;
    previous_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_current_boot = false;
    let mut wrong_schema_rollback_engine = valid;
    wrong_schema_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_schema_ok = false;
    let mut substituted_rollback_engine = valid;
    substituted_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    substituted_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_substituted_record";
    let mut mismatched_rollback_engine = valid;
    mismatched_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    mismatched_rollback_engine
        .memory_candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_binding_mismatch";

    let mut missing_durable_persistence = valid;
    missing_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_available = false;
    let mut previous_durable_persistence = valid;
    previous_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_current_boot = false;
    let mut wrong_schema_durable_persistence = valid;
    wrong_schema_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_schema_ok = false;
    let mut substituted_durable_persistence = valid;
    substituted_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_binding_ok = false;
    substituted_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_substituted_record";
    let mut mismatched_durable_persistence = valid;
    mismatched_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_binding_ok = false;
    mismatched_durable_persistence
        .memory_candidate
        .durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_binding_mismatch";

    let mut missing_memory_provenance_boundary = valid;
    missing_memory_provenance_boundary.recovery_memory_provenance_available = false;
    let mut previous_memory_provenance = valid;
    previous_memory_provenance.recovery_memory_provenance_current_boot = false;
    let mut wrong_schema_memory_provenance = valid;
    wrong_schema_memory_provenance.recovery_memory_provenance_schema_ok = false;
    let mut substituted_memory_provenance = valid;
    substituted_memory_provenance.recovery_memory_provenance_binding_ok = false;
    substituted_memory_provenance.recovery_memory_provenance_binding_reason =
        "recovery_memory_provenance_substituted_record";
    let mut mismatched_memory_provenance = valid;
    mismatched_memory_provenance.recovery_memory_provenance_binding_ok = false;
    mismatched_memory_provenance.recovery_memory_provenance_binding_reason =
        "recovery_memory_provenance_binding_mismatch";

    let mut memory_facts_missing = valid;
    memory_facts_missing
        .memory_candidate
        .source_record_ids_present = false;
    memory_facts_missing
        .memory_candidate
        .source_schema_hashes_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_classification_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_authority_level_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_rollback_transaction_binding_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_last_good_checkpoint_binding_present = false;
    memory_facts_missing
        .memory_candidate
        .recovery_only_export_profile_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_redaction_state_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_replay_window_present = false;
    memory_facts_missing
        .memory_candidate
        .memory_audit_linkage_present = false;
    let mut memory_audit_linkage_missing = valid;
    memory_audit_linkage_missing
        .memory_candidate
        .memory_audit_linkage_present = false;

    let mut all_admission_missing = valid;
    all_admission_missing.lifeline_status_admission_present = false;
    all_admission_missing.rollback_preview_admission_present = false;
    all_admission_missing.rollback_apply_admission_present = false;
    all_admission_missing.disable_module_admission_present = false;
    all_admission_missing.restart_last_good_admission_present = false;
    all_admission_missing.load_recovery_artifact_by_hash_admission_present = false;
    let mut status_admission_missing = valid;
    status_admission_missing.lifeline_status_admission_present = false;
    let mut preview_admission_missing = valid;
    preview_admission_missing.rollback_preview_admission_present = false;
    let mut apply_admission_missing = valid;
    apply_admission_missing.rollback_apply_admission_present = false;
    let mut disable_admission_missing = valid;
    disable_admission_missing.disable_module_admission_present = false;
    let mut restart_admission_missing = valid;
    restart_admission_missing.restart_last_good_admission_present = false;
    let mut load_by_hash_admission_missing = valid;
    load_by_hash_admission_missing.load_recovery_artifact_by_hash_admission_present = false;

    [
        recovery_lifeline_command_admission_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_lifeline_command_admission(missing_request),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_lifeline_command_admission(stale_request),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_request),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_request),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_request),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_command_admission(request_hash_mismatch),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_lifeline_command_admission(missing_protocol_state),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_protocol_state),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_protocol_state),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_protocol_state),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_lifeline_command_admission(missing_command_vocabulary),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_command_vocabulary),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_command_vocabulary),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_command_vocabulary),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_lifeline_command_admission(direct_provider_shortcut),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_lifeline_command_admission(missing_loader_runtime_isolation),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_loader_runtime_isolation),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_loader_runtime_isolation),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_loader_runtime_isolation),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "mismatched_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_binding_mismatch",
            evaluate_recovery_lifeline_command_admission(mismatched_loader_runtime_isolation),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "rollback_transaction_engine_missing_after_loader",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_lifeline_command_admission(missing_rollback_engine),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_rollback_engine),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_rollback_engine),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_rollback_engine),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "mismatched_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_binding_mismatch",
            evaluate_recovery_lifeline_command_admission(mismatched_rollback_engine),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "durable_persistence_boundary_missing_after_rollback_engine",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_lifeline_command_admission(missing_durable_persistence),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_durable_persistence),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_durable_persistence),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_durable_persistence),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "mismatched_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_binding_mismatch",
            evaluate_recovery_lifeline_command_admission(mismatched_durable_persistence),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "recovery_memory_provenance_boundary_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_command_admission(missing_memory_provenance_boundary),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "previous_boot_recovery_memory_provenance",
            "rejected",
            "recovery_memory_provenance_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_admission(previous_memory_provenance),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "wrong_schema_recovery_memory_provenance",
            "rejected",
            "recovery_memory_provenance_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_admission(wrong_schema_memory_provenance),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "substituted_recovery_memory_provenance",
            "rejected",
            "recovery_memory_provenance_substituted_record",
            evaluate_recovery_lifeline_command_admission(substituted_memory_provenance),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "mismatched_recovery_memory_provenance",
            "rejected",
            "recovery_memory_provenance_binding_mismatch",
            evaluate_recovery_lifeline_command_admission(mismatched_memory_provenance),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "recovery_memory_provenance_facts_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_command_admission(memory_facts_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "recovery_memory_audit_linkage_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_audit_linkage_missing",
            evaluate_recovery_lifeline_command_admission(memory_audit_linkage_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "command_admission_requirements_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_lifeline_command_admission_requirements_missing",
            evaluate_recovery_lifeline_command_admission(all_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "lifeline_status_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_lifeline_status_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(status_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "rollback_preview_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_rollback_preview_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(preview_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "rollback_apply_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_rollback_apply_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(apply_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "disable_module_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_disable_module_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(disable_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "restart_last_good_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_restart_last_good_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(restart_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "load_artifact_by_hash_command_admission_missing",
            "denied_missing_lifeline_command_admission",
            "recovery_load_artifact_by_hash_command_admission_missing",
            evaluate_recovery_lifeline_command_admission(load_by_hash_admission_missing),
        ),
        recovery_lifeline_command_admission_selftest_case(
            "all_inputs_present_command_admission_still_non_executable",
            "defined_non_executable",
            "recovery_lifeline_command_admission_behavior_not_implemented",
            evaluate_recovery_lifeline_command_admission(valid),
        ),
    ]
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
