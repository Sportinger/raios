use crate::{
    agent_protocol_recovery_constants::RECOVERY_LIFELINE_COMMAND_VOCABULARY_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_lifeline_protocol_types::*, agent_protocol_support::method_eq,
};

pub(crate) fn recovery_lifeline_command_vocabulary_candidate_from_protocol(
    protocol_candidate: RecoveryLifelineProtocolCandidate,
) -> RecoveryLifelineCommandVocabularyCandidate {
    RecoveryLifelineCommandVocabularyCandidate {
        protocol_candidate,
        protocol_state_retained: false,
        protocol_state_current_boot: false,
        protocol_state_schema_ok: false,
        protocol_state_binding_ok: false,
        protocol_state_binding_reason: "recovery_lifeline_protocol_state_missing",
        direct_openai_recovery_shortcut_used: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn evaluate_recovery_lifeline_command_vocabulary(
    candidate: RecoveryLifelineCommandVocabularyCandidate,
) -> RecoveryLifelineCommandVocabularyCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(candidate.protocol_candidate);
    if !protocol_check.request_chain_valid {
        return recovery_lifeline_command_vocabulary_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            false,
            false,
        );
    }
    if !candidate.protocol_state_retained {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            true,
        );
    }
    if !candidate.protocol_state_current_boot {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            false,
            false,
        );
    }
    if !candidate.protocol_state_schema_ok {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            false,
            false,
        );
    }
    if !candidate.protocol_state_binding_ok {
        return recovery_lifeline_command_vocabulary_check(
            "rejected",
            candidate.protocol_state_binding_reason,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_lifeline_command_vocabulary_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
        );
    }
    recovery_lifeline_command_vocabulary_check(
        "defined_non_executable",
        "recovery_lifeline_command_behavior_not_implemented",
        true,
        true,
    )
}

pub(crate) fn recovery_lifeline_command_vocabulary_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_exposed: bool,
) -> RecoveryLifelineCommandVocabularyCheck {
    RecoveryLifelineCommandVocabularyCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_exposed,
        command_execution_enabled: false,
        accepts_lifeline_command_envelope: false,
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

pub(crate) fn recovery_lifeline_command_vocabulary_selftest_cases(
) -> [RecoveryLifelineCommandVocabularySelfTestCase;
       RECOVERY_LIFELINE_COMMAND_VOCABULARY_SELFTEST_CASES] {
    let valid_protocol = recovery_lifeline_protocol_valid_request_candidate();
    let valid = RecoveryLifelineCommandVocabularyCandidate {
        protocol_candidate: valid_protocol,
        protocol_state_retained: true,
        protocol_state_current_boot: true,
        protocol_state_schema_ok: true,
        protocol_state_binding_ok: true,
        protocol_state_binding_reason: "retained_recovery_lifeline_protocol_state_valid",
        direct_openai_recovery_shortcut_used: false,
        loader_runtime_isolation_present: true,
        rollback_transaction_engine_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    };

    let mut missing_request = valid;
    missing_request.protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request.protocol_candidate.request_binding_ok = false;
    stale_request.protocol_candidate.request_binding_reason =
        "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request.protocol_candidate.request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request.protocol_candidate.request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request.protocol_candidate.request_binding_ok = false;
    substituted_request
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch.protocol_candidate.request_binding_ok = false;
    request_hash_mismatch
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state.protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state.protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state.protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state.protocol_state_binding_ok = false;
    substituted_protocol_state.protocol_state_binding_reason =
        "recovery_lifeline_protocol_state_substituted_record";
    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;
    let mut isolation_missing = valid;
    isolation_missing.loader_runtime_isolation_present = false;
    let mut rollback_engine_missing = valid;
    rollback_engine_missing.rollback_transaction_engine_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_lifeline_command_vocabulary_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_lifeline_command_vocabulary(missing_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_lifeline_command_vocabulary(stale_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_vocabulary(previous_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_vocabulary(wrong_schema_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_lifeline_command_vocabulary(substituted_request),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_command_vocabulary(request_hash_mismatch),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_lifeline_command_vocabulary(missing_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_lifeline_command_vocabulary(previous_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_lifeline_command_vocabulary(wrong_schema_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_lifeline_command_vocabulary(substituted_protocol_state),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_lifeline_command_vocabulary(direct_provider_shortcut),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "loader_runtime_isolation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_lifeline_command_vocabulary(isolation_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_lifeline_command_vocabulary(rollback_engine_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_lifeline_command_vocabulary(durable_persistence_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_command_vocabulary(memory_provenance_missing),
        ),
        recovery_lifeline_command_vocabulary_selftest_case(
            "all_inputs_present_commands_still_non_executable",
            "defined_non_executable",
            "recovery_lifeline_command_behavior_not_implemented",
            evaluate_recovery_lifeline_command_vocabulary(valid),
        ),
    ]
}

pub(crate) fn recovery_lifeline_command_vocabulary_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineCommandVocabularyCheck,
) -> RecoveryLifelineCommandVocabularySelfTestCase {
    RecoveryLifelineCommandVocabularySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}
