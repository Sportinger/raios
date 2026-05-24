use crate::{
    agent_protocol_recovery_constants::*, agent_protocol_recovery_lifeline_protocol_types::*,
    agent_protocol_recovery_runtime_types::*, agent_protocol_support::method_eq,
};
pub(crate) fn recovery_lifeline_protocol_missing_candidate() -> RecoveryLifelineProtocolCandidate {
    RecoveryLifelineProtocolCandidate {
        request_retained: false,
        request_current_boot: false,
        request_schema_ok: false,
        request_binding_ok: false,
        request_binding_reason: "recovery_lifeline_request_event_id_missing",
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn recovery_lifeline_protocol_valid_request_candidate(
) -> RecoveryLifelineProtocolCandidate {
    RecoveryLifelineProtocolCandidate {
        request_retained: true,
        request_current_boot: true,
        request_schema_ok: true,
        request_binding_ok: true,
        request_binding_reason: "retained_recovery_lifeline_request_valid",
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn evaluate_recovery_lifeline_protocol(
    candidate: RecoveryLifelineProtocolCandidate,
) -> RecoveryLifelineProtocolCheck {
    if !candidate.request_retained {
        return recovery_lifeline_protocol_check(
            "missing",
            "recovery_lifeline_request_event_id_missing",
            false,
            false,
        );
    }
    if !candidate.request_current_boot {
        return recovery_lifeline_protocol_check(
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            false,
            false,
        );
    }
    if !candidate.request_schema_ok {
        return recovery_lifeline_protocol_check(
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            false,
            false,
        );
    }
    if !candidate.request_binding_ok {
        return recovery_lifeline_protocol_check(
            "rejected",
            candidate.request_binding_reason,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used {
        return recovery_lifeline_protocol_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            false,
            false,
        );
    }
    if !candidate.lifeline_protocol_state_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            true,
        );
    }
    if !candidate.command_vocabulary_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            true,
        );
    }
    if !candidate.loader_runtime_isolation_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_lifeline_protocol_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
        );
    }
    recovery_lifeline_protocol_check(
        "denied_lifeline_protocol_behavior_unimplemented",
        "recovery_lifeline_protocol_behavior_not_implemented",
        true,
        true,
    )
}

fn recovery_lifeline_protocol_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    can_report_protocol_gaps: bool,
) -> RecoveryLifelineProtocolCheck {
    RecoveryLifelineProtocolCheck {
        status,
        reason,
        request_chain_valid,
        can_report_protocol_gaps,
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

pub(crate) fn recovery_lifeline_protocol_selftest_cases(
) -> [RecoveryLifelineProtocolSelfTestCase; RECOVERY_LIFELINE_PROTOCOL_SELFTEST_CASES] {
    let valid = recovery_lifeline_protocol_valid_request_candidate();

    let mut stale = valid;
    stale.request_binding_ok = false;
    stale.request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";

    let mut previous_boot = valid;
    previous_boot.request_current_boot = false;

    let mut wrong_schema = valid;
    wrong_schema.request_schema_ok = false;

    let mut substituted = valid;
    substituted.request_binding_ok = false;
    substituted.request_binding_reason = "recovery_lifeline_request_substituted_record";

    let mut request_hash_mismatch = valid;
    request_hash_mismatch.request_binding_ok = false;
    request_hash_mismatch.request_binding_reason =
        "recovery_lifeline_request_reference_hash_mismatch";

    let mut identity_event_mismatch = valid;
    identity_event_mismatch.request_binding_ok = false;
    identity_event_mismatch.request_binding_reason =
        "recovery_lifeline_request_identity_event_id_mismatch";

    let mut rollback_reference_mismatch = valid;
    rollback_reference_mismatch.request_binding_ok = false;
    rollback_reference_mismatch.request_binding_reason =
        "recovery_artifact_rollback_evidence_reference_hash_mismatch";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut command_vocabulary_missing = valid;
    command_vocabulary_missing.lifeline_protocol_state_present = true;

    let mut loader_isolation_missing = command_vocabulary_missing;
    loader_isolation_missing.command_vocabulary_present = true;

    let mut rollback_engine_missing = loader_isolation_missing;
    rollback_engine_missing.loader_runtime_isolation_present = true;

    let mut durable_persistence_missing = rollback_engine_missing;
    durable_persistence_missing.rollback_transaction_engine_present = true;

    let mut memory_provenance_missing = durable_persistence_missing;
    memory_provenance_missing.durable_audit_rollback_persistence_present = true;

    [
        recovery_lifeline_protocol_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_lifeline_protocol(recovery_lifeline_protocol_missing_candidate()),
        ),
        recovery_lifeline_protocol_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_lifeline_protocol(stale),
        ),
        recovery_lifeline_protocol_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_lifeline_protocol(previous_boot),
        ),
        recovery_lifeline_protocol_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_lifeline_protocol(wrong_schema),
        ),
        recovery_lifeline_protocol_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_lifeline_protocol(substituted),
        ),
        recovery_lifeline_protocol_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_protocol(request_hash_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "retained_identity_event_id_mismatch",
            "rejected",
            "recovery_lifeline_request_identity_event_id_mismatch",
            evaluate_recovery_lifeline_protocol(identity_event_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            evaluate_recovery_lifeline_protocol(rollback_reference_mismatch),
        ),
        recovery_lifeline_protocol_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_lifeline_protocol(direct_provider_shortcut),
        ),
        recovery_lifeline_protocol_selftest_case(
            "accepted_current_boot_request_protocol_state_missing",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_lifeline_protocol(valid),
        ),
        recovery_lifeline_protocol_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_lifeline_protocol(command_vocabulary_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_lifeline_protocol(loader_isolation_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "rollback_transaction_engine_missing_after_isolation",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_lifeline_protocol(rollback_engine_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "durable_audit_rollback_persistence_missing_after_engine",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_lifeline_protocol(durable_persistence_missing),
        ),
        recovery_lifeline_protocol_selftest_case(
            "recovery_memory_provenance_missing_after_persistence",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_lifeline_protocol(memory_provenance_missing),
        ),
    ]
}

fn recovery_lifeline_protocol_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineProtocolCheck,
) -> RecoveryLifelineProtocolSelfTestCase {
    RecoveryLifelineProtocolSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

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

fn recovery_lifeline_command_vocabulary_check(
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

fn recovery_lifeline_command_vocabulary_selftest_case(
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

pub(crate) fn recovery_loader_runtime_isolation_candidate_from_command_vocabulary(
    command_candidate: RecoveryLifelineCommandVocabularyCandidate,
) -> RecoveryLoaderRuntimeIsolationCandidate {
    RecoveryLoaderRuntimeIsolationCandidate {
        command_candidate,
        command_vocabulary_available: true,
        command_vocabulary_current_boot: true,
        command_vocabulary_schema_ok: true,
        command_vocabulary_binding_ok: true,
        command_vocabulary_binding_reason: "recovery_lifeline_command_vocabulary_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        loader_address_space_boundary_present: false,
        loader_entrypoint_abi_present: false,
        loader_memory_map_constraints_present: false,
        loader_capability_import_table_present: false,
        loader_artifact_hash_binding_present: false,
        loader_provider_separation_present: false,
        loader_normal_module_separation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn evaluate_recovery_loader_runtime_isolation(
    candidate: RecoveryLoaderRuntimeIsolationCandidate,
) -> RecoveryLoaderRuntimeIsolationCheck {
    let protocol_check =
        evaluate_recovery_lifeline_protocol(candidate.command_candidate.protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(candidate.command_candidate);
    if !protocol_check.request_chain_valid {
        return recovery_loader_runtime_isolation_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
        );
    }

    let vocabulary_envelope_exposed =
        command_check.command_vocabulary_exposed || candidate.command_vocabulary_available;
    if !candidate.command_candidate.protocol_state_retained {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            vocabulary_envelope_exposed,
            false,
            vocabulary_envelope_exposed,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_current_boot {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_schema_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_candidate.protocol_state_binding_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            candidate.command_candidate.protocol_state_binding_reason,
            true,
            vocabulary_envelope_exposed,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_available || !command_check.command_vocabulary_exposed {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_current_boot {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_schema_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.command_vocabulary_binding_ok {
        return recovery_loader_runtime_isolation_check(
            "rejected",
            candidate.command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
        );
    }

    if !candidate.loader_address_space_boundary_present
        && !candidate.loader_entrypoint_abi_present
        && !candidate.loader_memory_map_constraints_present
        && !candidate.loader_capability_import_table_present
        && !candidate.loader_artifact_hash_binding_present
        && !candidate.loader_provider_separation_present
        && !candidate.loader_normal_module_separation_present
    {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_address_space_boundary_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_entrypoint_abi_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_memory_map_constraints_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_capability_import_table_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_artifact_hash_binding_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_provider_separation_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.loader_normal_module_separation_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_transaction_engine_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_loader_runtime_isolation_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_loader_runtime_isolation_check(
        "defined_non_executable",
        "recovery_loader_runtime_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_loader_runtime_isolation_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    isolation_requirements_exposed: bool,
    loader_runtime_isolation_ready: bool,
) -> RecoveryLoaderRuntimeIsolationCheck {
    RecoveryLoaderRuntimeIsolationCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        isolation_requirements_exposed,
        loader_runtime_isolation_ready,
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

fn recovery_loader_runtime_isolation_valid_candidate() -> RecoveryLoaderRuntimeIsolationCandidate {
    let valid_protocol = recovery_lifeline_protocol_valid_request_candidate();
    let valid_command = RecoveryLifelineCommandVocabularyCandidate {
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
    RecoveryLoaderRuntimeIsolationCandidate {
        command_candidate: valid_command,
        command_vocabulary_available: true,
        command_vocabulary_current_boot: true,
        command_vocabulary_schema_ok: true,
        command_vocabulary_binding_ok: true,
        command_vocabulary_binding_reason: "retained_recovery_lifeline_command_vocabulary_valid",
        direct_openai_recovery_shortcut_used: false,
        loader_address_space_boundary_present: true,
        loader_entrypoint_abi_present: true,
        loader_memory_map_constraints_present: true,
        loader_capability_import_table_present: true,
        loader_artifact_hash_binding_present: true,
        loader_provider_separation_present: true,
        loader_normal_module_separation_present: true,
        rollback_transaction_engine_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    }
}

pub(crate) fn recovery_loader_runtime_isolation_selftest_cases(
) -> [RecoveryLoaderRuntimeIsolationSelfTestCase; RECOVERY_LOADER_RUNTIME_ISOLATION_SELFTEST_CASES]
{
    let valid = recovery_loader_runtime_isolation_valid_candidate();

    let mut missing_request = valid;
    missing_request.command_candidate.protocol_candidate =
        recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary.command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary.command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary.command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary.command_vocabulary_binding_ok = false;
    substituted_command_vocabulary.command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut all_isolation_missing = valid;
    all_isolation_missing.loader_address_space_boundary_present = false;
    all_isolation_missing.loader_entrypoint_abi_present = false;
    all_isolation_missing.loader_memory_map_constraints_present = false;
    all_isolation_missing.loader_capability_import_table_present = false;
    all_isolation_missing.loader_artifact_hash_binding_present = false;
    all_isolation_missing.loader_provider_separation_present = false;
    all_isolation_missing.loader_normal_module_separation_present = false;

    let mut address_space_missing = valid;
    address_space_missing.loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing.loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing.loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing.loader_capability_import_table_present = false;
    let mut artifact_hash_binding_missing = valid;
    artifact_hash_binding_missing.loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing.loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing.loader_normal_module_separation_present = false;

    let mut rollback_engine_missing = valid;
    rollback_engine_missing.rollback_transaction_engine_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_loader_runtime_isolation_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_loader_runtime_isolation(missing_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_loader_runtime_isolation(stale_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_request),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_loader_runtime_isolation(request_hash_mismatch),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_loader_runtime_isolation(missing_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_protocol_state),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_loader_runtime_isolation(missing_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_loader_runtime_isolation(previous_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_loader_runtime_isolation(wrong_schema_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_loader_runtime_isolation(substituted_command_vocabulary),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_loader_runtime_isolation(direct_provider_shortcut),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_runtime_isolation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_loader_runtime_isolation(all_isolation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_loader_runtime_isolation(address_space_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_loader_runtime_isolation(entrypoint_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_loader_runtime_isolation(memory_map_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_loader_runtime_isolation(import_table_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_loader_runtime_isolation(artifact_hash_binding_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_loader_runtime_isolation(provider_separation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_loader_runtime_isolation(normal_module_separation_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_loader_runtime_isolation(rollback_engine_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_loader_runtime_isolation(durable_persistence_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_loader_runtime_isolation(memory_provenance_missing),
        ),
        recovery_loader_runtime_isolation_selftest_case(
            "all_inputs_present_loader_still_non_executable",
            "defined_non_executable",
            "recovery_loader_runtime_behavior_not_implemented",
            evaluate_recovery_loader_runtime_isolation(valid),
        ),
    ]
}

fn recovery_loader_runtime_isolation_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLoaderRuntimeIsolationCheck,
) -> RecoveryLoaderRuntimeIsolationSelfTestCase {
    RecoveryLoaderRuntimeIsolationSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_rollback_transaction_engine_candidate_from_loader(
    loader_candidate: RecoveryLoaderRuntimeIsolationCandidate,
) -> RecoveryRollbackTransactionEngineCandidate {
    RecoveryRollbackTransactionEngineCandidate {
        loader_candidate,
        loader_runtime_isolation_available: true,
        loader_runtime_isolation_current_boot: true,
        loader_runtime_isolation_schema_ok: true,
        loader_runtime_isolation_binding_ok: true,
        loader_runtime_isolation_binding_reason:
            "recovery_loader_runtime_isolation_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        rollback_target_selection_present: false,
        rollback_transaction_id_provenance_present: false,
        rollback_last_good_binding_present: false,
        rollback_disabled_module_set_binding_present: false,
        rollback_artifact_hash_binding_present: false,
        rollback_replay_preconditions_present: false,
        rollback_recovery_capability_import_present: false,
        rollback_atomic_apply_abort_semantics_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn evaluate_recovery_rollback_transaction_engine(
    candidate: RecoveryRollbackTransactionEngineCandidate,
) -> RecoveryRollbackTransactionEngineCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check =
        evaluate_recovery_lifeline_command_vocabulary(candidate.loader_candidate.command_candidate);
    let loader_check = evaluate_recovery_loader_runtime_isolation(candidate.loader_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_rollback_transaction_engine_check(
            protocol_check.status,
            protocol_check.reason,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate.loader_candidate.command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed =
        loader_check.isolation_requirements_exposed || candidate.loader_runtime_isolation_available;
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_current_boot {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_schema_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_candidate.command_vocabulary_binding_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate.loader_candidate.command_vocabulary_binding_reason,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        );
    }

    if !candidate.loader_runtime_isolation_available {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_current_boot {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_schema_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !candidate.loader_runtime_isolation_binding_ok {
        return recovery_rollback_transaction_engine_check(
            "rejected",
            candidate.loader_runtime_isolation_binding_reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_rollback_transaction_engine_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        );
    }

    if !candidate.rollback_target_selection_present
        && !candidate.rollback_transaction_id_provenance_present
        && !candidate.rollback_last_good_binding_present
        && !candidate.rollback_disabled_module_set_binding_present
        && !candidate.rollback_artifact_hash_binding_present
        && !candidate.rollback_replay_preconditions_present
        && !candidate.rollback_recovery_capability_import_present
        && !candidate.rollback_atomic_apply_abort_semantics_present
    {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_target_selection_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_transaction_id_provenance_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_last_good_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_disabled_module_set_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_artifact_hash_binding_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_replay_preconditions_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_recovery_capability_import_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.rollback_atomic_apply_abort_semantics_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    if !candidate.recovery_memory_provenance_present {
        return recovery_rollback_transaction_engine_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_rollback_transaction_engine_check(
        "defined_non_executable",
        "recovery_rollback_transaction_behavior_not_implemented",
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

fn recovery_rollback_transaction_engine_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    transaction_requirements_exposed: bool,
    rollback_transaction_engine_ready: bool,
) -> RecoveryRollbackTransactionEngineCheck {
    RecoveryRollbackTransactionEngineCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        transaction_requirements_exposed,
        rollback_transaction_engine_ready,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
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

fn recovery_rollback_transaction_engine_valid_candidate(
) -> RecoveryRollbackTransactionEngineCandidate {
    RecoveryRollbackTransactionEngineCandidate {
        loader_candidate: recovery_loader_runtime_isolation_valid_candidate(),
        loader_runtime_isolation_available: true,
        loader_runtime_isolation_current_boot: true,
        loader_runtime_isolation_schema_ok: true,
        loader_runtime_isolation_binding_ok: true,
        loader_runtime_isolation_binding_reason: "retained_recovery_loader_runtime_isolation_valid",
        direct_openai_recovery_shortcut_used: false,
        rollback_target_selection_present: true,
        rollback_transaction_id_provenance_present: true,
        rollback_last_good_binding_present: true,
        rollback_disabled_module_set_binding_present: true,
        rollback_artifact_hash_binding_present: true,
        rollback_replay_preconditions_present: true,
        rollback_recovery_capability_import_present: true,
        rollback_atomic_apply_abort_semantics_present: true,
        durable_audit_rollback_persistence_present: true,
        recovery_memory_provenance_present: true,
    }
}

pub(crate) fn recovery_rollback_transaction_engine_selftest_cases(
) -> [RecoveryRollbackTransactionEngineSelfTestCase;
       RECOVERY_ROLLBACK_TRANSACTION_ENGINE_SELFTEST_CASES] {
    let valid = recovery_rollback_transaction_engine_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation.loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation.loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation.loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation.loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation.loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut all_transaction_missing = valid;
    all_transaction_missing.rollback_target_selection_present = false;
    all_transaction_missing.rollback_transaction_id_provenance_present = false;
    all_transaction_missing.rollback_last_good_binding_present = false;
    all_transaction_missing.rollback_disabled_module_set_binding_present = false;
    all_transaction_missing.rollback_artifact_hash_binding_present = false;
    all_transaction_missing.rollback_replay_preconditions_present = false;
    all_transaction_missing.rollback_recovery_capability_import_present = false;
    all_transaction_missing.rollback_atomic_apply_abort_semantics_present = false;

    let mut target_selection_missing = valid;
    target_selection_missing.rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing.rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing.rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing.rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing.rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing.rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing.rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing.rollback_atomic_apply_abort_semantics_present = false;
    let mut durable_persistence_missing = valid;
    durable_persistence_missing.durable_audit_rollback_persistence_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_rollback_transaction_engine_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_rollback_transaction_engine(missing_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_rollback_transaction_engine(stale_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_request),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_rollback_transaction_engine(request_hash_mismatch),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_rollback_transaction_engine(missing_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_protocol_state),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_rollback_transaction_engine(missing_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_command_vocabulary),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_rollback_transaction_engine(direct_provider_shortcut),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_rollback_transaction_engine(missing_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_rollback_transaction_engine(previous_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_rollback_transaction_engine(wrong_schema_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_rollback_transaction_engine(substituted_loader_runtime_isolation),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_rollback_transaction_engine(address_space_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_rollback_transaction_engine(entrypoint_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_rollback_transaction_engine(memory_map_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_rollback_transaction_engine(import_table_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_rollback_transaction_engine(artifact_hash_isolation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_rollback_transaction_engine(provider_separation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_rollback_transaction_engine(normal_module_separation_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_transaction_engine_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_rollback_transaction_engine(all_transaction_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_rollback_transaction_engine(target_selection_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_rollback_transaction_engine(transaction_id_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_rollback_transaction_engine(last_good_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_rollback_transaction_engine(disabled_set_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_rollback_transaction_engine(artifact_hash_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_rollback_transaction_engine(replay_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_rollback_transaction_engine(capability_import_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_rollback_transaction_engine(atomic_semantics_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_rollback_transaction_engine(durable_persistence_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_rollback_transaction_engine(memory_provenance_missing),
        ),
        recovery_rollback_transaction_engine_selftest_case(
            "all_inputs_present_rollback_still_non_executable",
            "defined_non_executable",
            "recovery_rollback_transaction_behavior_not_implemented",
            evaluate_recovery_rollback_transaction_engine(valid),
        ),
    ]
}

fn recovery_rollback_transaction_engine_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackTransactionEngineCheck,
) -> RecoveryRollbackTransactionEngineSelfTestCase {
    RecoveryRollbackTransactionEngineSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_durable_audit_rollback_persistence_candidate_from_transaction(
    transaction_candidate: RecoveryRollbackTransactionEngineCandidate,
) -> RecoveryDurableAuditRollbackPersistenceCandidate {
    RecoveryDurableAuditRollbackPersistenceCandidate {
        transaction_candidate,
        rollback_transaction_engine_available: true,
        rollback_transaction_engine_current_boot: true,
        rollback_transaction_engine_schema_ok: true,
        rollback_transaction_engine_binding_ok: true,
        rollback_transaction_engine_binding_reason:
            "recovery_rollback_transaction_engine_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        persistence_device_inventory_present: false,
        storage_layout_identity_present: false,
        audit_append_log_identity_present: false,
        rollback_store_identity_present: false,
        transaction_replay_cursor_present: false,
        last_good_checkpoint_binding_present: false,
        write_ordering_present: false,
        crash_consistency_present: false,
        integrity_root_hash_chain_present: false,
        recovery_memory_provenance_present: false,
    }
}

pub(crate) fn evaluate_recovery_durable_audit_rollback_persistence(
    candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
) -> RecoveryDurableAuditRollbackPersistenceCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check = evaluate_recovery_lifeline_command_vocabulary(
        candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate,
    );
    let loader_check = evaluate_recovery_loader_runtime_isolation(
        candidate.transaction_candidate.loader_candidate,
    );
    let transaction_check =
        evaluate_recovery_rollback_transaction_engine(candidate.transaction_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_durable_audit_rollback_persistence_check(
            protocol_check.status,
            protocol_check.reason,
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
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            true,
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

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate
            .transaction_candidate
            .loader_candidate
            .command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed = loader_check.isolation_requirements_exposed
        || candidate
            .transaction_candidate
            .loader_runtime_isolation_available;
    let rollback_transaction_engine_boundary_exposed = transaction_check
        .transaction_requirements_exposed
        || candidate.rollback_transaction_engine_available;

    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        );
    }
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            true,
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
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
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
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
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
    if !candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_reason,
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

    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_available
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
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
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
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
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
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
    if !candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok
    {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason,
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
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_durable_audit_rollback_persistence_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate.rollback_transaction_engine_available {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
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
    if !candidate.rollback_transaction_engine_current_boot {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
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
    if !candidate.rollback_transaction_engine_schema_ok {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
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
    if !candidate.rollback_transaction_engine_binding_ok {
        return recovery_durable_audit_rollback_persistence_check(
            "rejected",
            candidate.rollback_transaction_engine_binding_reason,
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
    if !transaction_check.rollback_transaction_engine_ready {
        return recovery_durable_audit_rollback_persistence_check(
            transaction_check.status,
            transaction_check.reason,
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

    if !candidate.persistence_device_inventory_present
        && !candidate.storage_layout_identity_present
        && !candidate.audit_append_log_identity_present
        && !candidate.rollback_store_identity_present
        && !candidate.transaction_replay_cursor_present
        && !candidate.last_good_checkpoint_binding_present
        && !candidate.write_ordering_present
        && !candidate.crash_consistency_present
        && !candidate.integrity_root_hash_chain_present
    {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
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
    if !candidate.persistence_device_inventory_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
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
    if !candidate.storage_layout_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
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
    if !candidate.audit_append_log_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
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
    if !candidate.rollback_store_identity_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
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
    if !candidate.transaction_replay_cursor_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
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
    if !candidate.last_good_checkpoint_binding_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
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
    if !candidate.write_ordering_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
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
    if !candidate.crash_consistency_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
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
    if !candidate.integrity_root_hash_chain_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
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
    if !candidate.recovery_memory_provenance_present {
        return recovery_durable_audit_rollback_persistence_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        );
    }
    recovery_durable_audit_rollback_persistence_check(
        "defined_non_executable",
        "durable_audit_rollback_persistence_behavior_not_implemented",
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

fn recovery_durable_audit_rollback_persistence_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    persistence_requirements_exposed: bool,
    durable_audit_rollback_persistence_ready: bool,
) -> RecoveryDurableAuditRollbackPersistenceCheck {
    RecoveryDurableAuditRollbackPersistenceCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        rollback_transaction_engine_boundary_exposed,
        rollback_transaction_engine_accepted,
        persistence_requirements_exposed,
        durable_audit_rollback_persistence_ready,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        recovery_memory_writes_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
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

fn recovery_durable_audit_rollback_persistence_valid_candidate(
) -> RecoveryDurableAuditRollbackPersistenceCandidate {
    RecoveryDurableAuditRollbackPersistenceCandidate {
        transaction_candidate: recovery_rollback_transaction_engine_valid_candidate(),
        rollback_transaction_engine_available: true,
        rollback_transaction_engine_current_boot: true,
        rollback_transaction_engine_schema_ok: true,
        rollback_transaction_engine_binding_ok: true,
        rollback_transaction_engine_binding_reason:
            "retained_recovery_rollback_transaction_engine_valid",
        direct_openai_recovery_shortcut_used: false,
        persistence_device_inventory_present: true,
        storage_layout_identity_present: true,
        audit_append_log_identity_present: true,
        rollback_store_identity_present: true,
        transaction_replay_cursor_present: true,
        last_good_checkpoint_binding_present: true,
        write_ordering_present: true,
        crash_consistency_present: true,
        integrity_root_hash_chain_present: true,
        recovery_memory_provenance_present: true,
    }
}

pub(crate) fn recovery_durable_audit_rollback_persistence_selftest_cases(
) -> [RecoveryDurableAuditRollbackPersistenceSelfTestCase;
       RECOVERY_DURABLE_AUDIT_ROLLBACK_PERSISTENCE_SELFTEST_CASES] {
    let valid = recovery_durable_audit_rollback_persistence_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .transaction_candidate
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .transaction_candidate
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .transaction_candidate
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .transaction_candidate
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .transaction_candidate
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .transaction_candidate
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .transaction_candidate
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut missing_rollback_engine_boundary = valid;
    missing_rollback_engine_boundary.rollback_transaction_engine_available = false;
    let mut previous_rollback_engine = valid;
    previous_rollback_engine.rollback_transaction_engine_current_boot = false;
    let mut wrong_schema_rollback_engine = valid;
    wrong_schema_rollback_engine.rollback_transaction_engine_schema_ok = false;
    let mut substituted_rollback_engine = valid;
    substituted_rollback_engine.rollback_transaction_engine_binding_ok = false;
    substituted_rollback_engine.rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_substituted_record";
    let mut mismatched_rollback_engine = valid;
    mismatched_rollback_engine.rollback_transaction_engine_binding_ok = false;
    mismatched_rollback_engine.rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_binding_mismatch";

    let mut target_selection_missing = valid;
    target_selection_missing
        .transaction_candidate
        .rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing
        .transaction_candidate
        .rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing
        .transaction_candidate
        .rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing
        .transaction_candidate
        .rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing
        .transaction_candidate
        .rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing
        .transaction_candidate
        .rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing
        .transaction_candidate
        .rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing
        .transaction_candidate
        .rollback_atomic_apply_abort_semantics_present = false;

    let mut all_persistence_missing = valid;
    all_persistence_missing.persistence_device_inventory_present = false;
    all_persistence_missing.storage_layout_identity_present = false;
    all_persistence_missing.audit_append_log_identity_present = false;
    all_persistence_missing.rollback_store_identity_present = false;
    all_persistence_missing.transaction_replay_cursor_present = false;
    all_persistence_missing.last_good_checkpoint_binding_present = false;
    all_persistence_missing.write_ordering_present = false;
    all_persistence_missing.crash_consistency_present = false;
    all_persistence_missing.integrity_root_hash_chain_present = false;
    let mut persistence_device_missing = valid;
    persistence_device_missing.persistence_device_inventory_present = false;
    let mut storage_layout_missing = valid;
    storage_layout_missing.storage_layout_identity_present = false;
    let mut audit_log_missing = valid;
    audit_log_missing.audit_append_log_identity_present = false;
    let mut rollback_store_missing = valid;
    rollback_store_missing.rollback_store_identity_present = false;
    let mut replay_cursor_missing = valid;
    replay_cursor_missing.transaction_replay_cursor_present = false;
    let mut checkpoint_missing = valid;
    checkpoint_missing.last_good_checkpoint_binding_present = false;
    let mut write_ordering_missing = valid;
    write_ordering_missing.write_ordering_present = false;
    let mut crash_consistency_missing = valid;
    crash_consistency_missing.crash_consistency_present = false;
    let mut integrity_root_missing = valid;
    integrity_root_missing.integrity_root_hash_chain_present = false;
    let mut memory_provenance_missing = valid;
    memory_provenance_missing.recovery_memory_provenance_present = false;

    [
        recovery_durable_audit_rollback_persistence_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_durable_audit_rollback_persistence(stale_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_request),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_durable_audit_rollback_persistence(request_hash_mismatch),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_protocol_state),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_command_vocabulary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_durable_audit_rollback_persistence(direct_provider_shortcut),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_loader_runtime_isolation),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_loader_runtime_isolation),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(
                wrong_schema_loader_runtime_isolation,
            ),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(
                substituted_loader_runtime_isolation,
            ),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_durable_audit_rollback_persistence(address_space_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_durable_audit_rollback_persistence(entrypoint_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_durable_audit_rollback_persistence(memory_map_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_durable_audit_rollback_persistence(import_table_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(artifact_hash_isolation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(provider_separation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_durable_audit_rollback_persistence(normal_module_separation_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_transaction_engine_boundary_missing_after_loader",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_durable_audit_rollback_persistence(missing_rollback_engine_boundary),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "previous_boot_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            evaluate_recovery_durable_audit_rollback_persistence(previous_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "wrong_schema_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            evaluate_recovery_durable_audit_rollback_persistence(wrong_schema_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "substituted_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_substituted_record",
            evaluate_recovery_durable_audit_rollback_persistence(substituted_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "mismatched_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_binding_mismatch",
            evaluate_recovery_durable_audit_rollback_persistence(mismatched_rollback_engine),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_durable_audit_rollback_persistence(target_selection_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_durable_audit_rollback_persistence(transaction_id_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(last_good_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(disabled_set_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(artifact_hash_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_durable_audit_rollback_persistence(replay_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_durable_audit_rollback_persistence(capability_import_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_durable_audit_rollback_persistence(atomic_semantics_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "durable_audit_rollback_persistence_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_durable_audit_rollback_persistence(all_persistence_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "persistence_device_inventory_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
            evaluate_recovery_durable_audit_rollback_persistence(persistence_device_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "storage_layout_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(storage_layout_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "audit_append_log_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(audit_log_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "rollback_store_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
            evaluate_recovery_durable_audit_rollback_persistence(rollback_store_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "transaction_replay_cursor_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
            evaluate_recovery_durable_audit_rollback_persistence(replay_cursor_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "last_good_checkpoint_binding_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
            evaluate_recovery_durable_audit_rollback_persistence(checkpoint_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "write_ordering_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
            evaluate_recovery_durable_audit_rollback_persistence(write_ordering_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "crash_consistency_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
            evaluate_recovery_durable_audit_rollback_persistence(crash_consistency_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "integrity_root_hash_chain_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
            evaluate_recovery_durable_audit_rollback_persistence(integrity_root_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_durable_audit_rollback_persistence(memory_provenance_missing),
        ),
        recovery_durable_audit_rollback_persistence_selftest_case(
            "all_inputs_present_persistence_still_non_executable",
            "defined_non_executable",
            "durable_audit_rollback_persistence_behavior_not_implemented",
            evaluate_recovery_durable_audit_rollback_persistence(valid),
        ),
    ]
}

fn recovery_durable_audit_rollback_persistence_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryDurableAuditRollbackPersistenceCheck,
) -> RecoveryDurableAuditRollbackPersistenceSelfTestCase {
    RecoveryDurableAuditRollbackPersistenceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

pub(crate) fn recovery_memory_provenance_candidate_from_persistence(
    persistence_candidate: RecoveryDurableAuditRollbackPersistenceCandidate,
) -> RecoveryMemoryProvenanceCandidate {
    RecoveryMemoryProvenanceCandidate {
        persistence_candidate,
        durable_audit_rollback_persistence_available: true,
        durable_audit_rollback_persistence_current_boot: true,
        durable_audit_rollback_persistence_schema_ok: true,
        durable_audit_rollback_persistence_binding_ok: true,
        durable_audit_rollback_persistence_binding_reason:
            "durable_audit_rollback_persistence_defined_read_only",
        direct_openai_recovery_shortcut_used: false,
        source_record_ids_present: false,
        source_schema_hashes_present: false,
        memory_classification_present: false,
        memory_authority_level_present: false,
        memory_rollback_transaction_binding_present: false,
        memory_last_good_checkpoint_binding_present: false,
        recovery_only_export_profile_present: false,
        memory_redaction_state_present: false,
        memory_replay_window_present: false,
        memory_audit_linkage_present: false,
    }
}

pub(crate) fn evaluate_recovery_memory_provenance(
    candidate: RecoveryMemoryProvenanceCandidate,
) -> RecoveryMemoryProvenanceCheck {
    let protocol_check = evaluate_recovery_lifeline_protocol(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate,
    );
    let command_check = evaluate_recovery_lifeline_command_vocabulary(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate,
    );
    let loader_check = evaluate_recovery_loader_runtime_isolation(
        candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate,
    );
    let transaction_check = evaluate_recovery_rollback_transaction_engine(
        candidate.persistence_candidate.transaction_candidate,
    );
    let persistence_check =
        evaluate_recovery_durable_audit_rollback_persistence(candidate.persistence_candidate);

    if !protocol_check.request_chain_valid {
        return recovery_memory_provenance_check(
            protocol_check.status,
            protocol_check.reason,
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
            false,
        );
    }
    if candidate.direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .direct_openai_recovery_shortcut_used
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_candidate
            .protocol_candidate
            .direct_openai_recovery_shortcut_used
    {
        return recovery_memory_provenance_check(
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
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
            false,
        );
    }

    let command_vocabulary_envelope_exposed = command_check.command_vocabulary_exposed
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_candidate
            .command_vocabulary_available;
    let loader_runtime_isolation_boundary_exposed = loader_check.isolation_requirements_exposed
        || candidate
            .persistence_candidate
            .transaction_candidate
            .loader_runtime_isolation_available;
    let rollback_transaction_engine_boundary_exposed = transaction_check
        .transaction_requirements_exposed
        || candidate
            .persistence_candidate
            .rollback_transaction_engine_available;
    let durable_audit_rollback_persistence_boundary_exposed = persistence_check
        .persistence_requirements_exposed
        || candidate.durable_audit_rollback_persistence_available;

    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained
    {
        return recovery_memory_provenance_check(
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            true,
            command_vocabulary_envelope_exposed,
            false,
            loader_runtime_isolation_boundary_exposed,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            true,
            command_vocabulary_envelope_exposed,
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
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            true,
            command_vocabulary_envelope_exposed,
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
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_candidate
                .protocol_state_binding_reason,
            true,
            command_vocabulary_envelope_exposed,
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
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available
        || !command_check.command_vocabulary_exposed
    {
        return recovery_memory_provenance_check(
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            true,
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
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            true,
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
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_candidate
                .command_vocabulary_binding_reason,
            true,
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

    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_available
    {
        return recovery_memory_provenance_check(
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .transaction_candidate
                .loader_runtime_isolation_binding_reason,
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
            false,
        );
    }
    if !loader_check.loader_runtime_isolation_ready {
        return recovery_memory_provenance_check(
            loader_check.status,
            loader_check.reason,
            true,
            true,
            true,
            true,
            false,
            rollback_transaction_engine_boundary_exposed,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_available
    {
        return recovery_memory_provenance_check(
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_current_boot
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_schema_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
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
            false,
        );
    }
    if !candidate
        .persistence_candidate
        .rollback_transaction_engine_binding_ok
    {
        return recovery_memory_provenance_check(
            "rejected",
            candidate
                .persistence_candidate
                .rollback_transaction_engine_binding_reason,
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
            false,
        );
    }
    if !transaction_check.rollback_transaction_engine_ready {
        return recovery_memory_provenance_check(
            transaction_check.status,
            transaction_check.reason,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            durable_audit_rollback_persistence_boundary_exposed,
            false,
            false,
            false,
        );
    }

    if !candidate.durable_audit_rollback_persistence_available {
        return recovery_memory_provenance_check(
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
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
            false,
        );
    }
    if !candidate.durable_audit_rollback_persistence_current_boot {
        return recovery_memory_provenance_check(
            "rejected",
            "durable_audit_rollback_persistence_event_id_not_current_boot",
            true,
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
    if !candidate.durable_audit_rollback_persistence_schema_ok {
        return recovery_memory_provenance_check(
            "rejected",
            "durable_audit_rollback_persistence_wrong_schema_or_variant",
            true,
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
    if !candidate.durable_audit_rollback_persistence_binding_ok {
        return recovery_memory_provenance_check(
            "rejected",
            candidate.durable_audit_rollback_persistence_binding_reason,
            true,
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
    if !persistence_check.durable_audit_rollback_persistence_ready {
        return recovery_memory_provenance_check(
            persistence_check.status,
            persistence_check.reason,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            persistence_check.persistence_requirements_exposed,
            false,
        );
    }

    if !candidate.source_record_ids_present
        && !candidate.source_schema_hashes_present
        && !candidate.memory_classification_present
        && !candidate.memory_authority_level_present
        && !candidate.memory_rollback_transaction_binding_present
        && !candidate.memory_last_good_checkpoint_binding_present
        && !candidate.recovery_only_export_profile_present
        && !candidate.memory_redaction_state_present
        && !candidate.memory_replay_window_present
        && !candidate.memory_audit_linkage_present
    {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
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
            false,
        );
    }
    if !candidate.source_record_ids_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_record_ids_missing",
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
            false,
        );
    }
    if !candidate.source_schema_hashes_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_schema_hashes_missing",
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
            false,
        );
    }
    if !candidate.memory_classification_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_classification_missing",
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
            false,
        );
    }
    if !candidate.memory_authority_level_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_authority_level_missing",
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
            false,
        );
    }
    if !candidate.memory_rollback_transaction_binding_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_rollback_transaction_binding_missing",
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
            false,
        );
    }
    if !candidate.memory_last_good_checkpoint_binding_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_last_good_checkpoint_binding_missing",
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
            false,
        );
    }
    if !candidate.recovery_only_export_profile_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_only_export_profile_missing",
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
            false,
        );
    }
    if !candidate.memory_redaction_state_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_redaction_state_missing",
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
            false,
        );
    }
    if !candidate.memory_replay_window_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_replay_window_missing",
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
            false,
        );
    }
    if !candidate.memory_audit_linkage_present {
        return recovery_memory_provenance_check(
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_audit_linkage_missing",
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
            false,
        );
    }

    recovery_memory_provenance_check(
        "defined_non_executable",
        "recovery_memory_provenance_behavior_not_implemented",
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
        true,
    )
}

fn recovery_memory_provenance_check(
    status: &'static str,
    reason: &'static str,
    request_chain_valid: bool,
    command_vocabulary_envelope_exposed: bool,
    command_vocabulary_accepted: bool,
    loader_runtime_isolation_boundary_exposed: bool,
    loader_runtime_isolation_accepted: bool,
    rollback_transaction_engine_boundary_exposed: bool,
    rollback_transaction_engine_accepted: bool,
    durable_audit_rollback_persistence_boundary_exposed: bool,
    durable_audit_rollback_persistence_accepted: bool,
    memory_provenance_requirements_exposed: bool,
    recovery_memory_provenance_ready: bool,
) -> RecoveryMemoryProvenanceCheck {
    RecoveryMemoryProvenanceCheck {
        status,
        reason,
        request_chain_valid,
        command_vocabulary_envelope_exposed,
        command_vocabulary_accepted,
        loader_runtime_isolation_boundary_exposed,
        loader_runtime_isolation_accepted,
        rollback_transaction_engine_boundary_exposed,
        rollback_transaction_engine_accepted,
        durable_audit_rollback_persistence_boundary_exposed,
        durable_audit_rollback_persistence_accepted,
        memory_provenance_requirements_exposed,
        recovery_memory_provenance_ready,
        memory_writes_enabled: false,
        provider_export_enabled: false,
        durable_writes_enabled: false,
        rollback_replay_enabled: false,
        recovery_memory_writes_enabled: false,
        rollback_preview_enabled: false,
        rollback_apply_enabled: false,
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

fn recovery_memory_provenance_valid_candidate() -> RecoveryMemoryProvenanceCandidate {
    RecoveryMemoryProvenanceCandidate {
        persistence_candidate: recovery_durable_audit_rollback_persistence_valid_candidate(),
        durable_audit_rollback_persistence_available: true,
        durable_audit_rollback_persistence_current_boot: true,
        durable_audit_rollback_persistence_schema_ok: true,
        durable_audit_rollback_persistence_binding_ok: true,
        durable_audit_rollback_persistence_binding_reason:
            "retained_durable_audit_rollback_persistence_valid",
        direct_openai_recovery_shortcut_used: false,
        source_record_ids_present: true,
        source_schema_hashes_present: true,
        memory_classification_present: true,
        memory_authority_level_present: true,
        memory_rollback_transaction_binding_present: true,
        memory_last_good_checkpoint_binding_present: true,
        recovery_only_export_profile_present: true,
        memory_redaction_state_present: true,
        memory_replay_window_present: true,
        memory_audit_linkage_present: true,
    }
}

pub(crate) fn recovery_memory_provenance_selftest_cases(
) -> [RecoveryMemoryProvenanceSelfTestCase; RECOVERY_MEMORY_PROVENANCE_SELFTEST_CASES] {
    let valid = recovery_memory_provenance_valid_candidate();

    let mut missing_request = valid;
    missing_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate = recovery_lifeline_protocol_missing_candidate();
    let mut stale_request = valid;
    stale_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    stale_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_event_id_stale_or_dropped";
    let mut previous_request = valid;
    previous_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_current_boot = false;
    let mut wrong_schema_request = valid;
    wrong_schema_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_schema_ok = false;
    let mut substituted_request = valid;
    substituted_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    substituted_request
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_substituted_record";
    let mut request_hash_mismatch = valid;
    request_hash_mismatch
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_ok = false;
    request_hash_mismatch
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_candidate
        .request_binding_reason = "recovery_lifeline_request_reference_hash_mismatch";

    let mut missing_protocol_state = valid;
    missing_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_retained = false;
    let mut previous_protocol_state = valid;
    previous_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_current_boot = false;
    let mut wrong_schema_protocol_state = valid;
    wrong_schema_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_schema_ok = false;
    let mut substituted_protocol_state = valid;
    substituted_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_ok = false;
    substituted_protocol_state
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_candidate
        .protocol_state_binding_reason = "recovery_lifeline_protocol_state_substituted_record";

    let mut missing_command_vocabulary = valid;
    missing_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_available = false;
    let mut previous_command_vocabulary = valid;
    previous_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_current_boot = false;
    let mut wrong_schema_command_vocabulary = valid;
    wrong_schema_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_schema_ok = false;
    let mut substituted_command_vocabulary = valid;
    substituted_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_ok = false;
    substituted_command_vocabulary
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .command_vocabulary_binding_reason =
        "recovery_lifeline_command_vocabulary_substituted_record";

    let mut direct_provider_shortcut = valid;
    direct_provider_shortcut.direct_openai_recovery_shortcut_used = true;

    let mut missing_loader_runtime_isolation = valid;
    missing_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_available = false;
    let mut previous_loader_runtime_isolation = valid;
    previous_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_current_boot = false;
    let mut wrong_schema_loader_runtime_isolation = valid;
    wrong_schema_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_schema_ok = false;
    let mut substituted_loader_runtime_isolation = valid;
    substituted_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_ok = false;
    substituted_loader_runtime_isolation
        .persistence_candidate
        .transaction_candidate
        .loader_runtime_isolation_binding_reason =
        "recovery_loader_runtime_isolation_substituted_record";

    let mut address_space_missing = valid;
    address_space_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_address_space_boundary_present = false;
    let mut entrypoint_missing = valid;
    entrypoint_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_entrypoint_abi_present = false;
    let mut memory_map_missing = valid;
    memory_map_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_memory_map_constraints_present = false;
    let mut import_table_missing = valid;
    import_table_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_capability_import_table_present = false;
    let mut artifact_hash_isolation_missing = valid;
    artifact_hash_isolation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_artifact_hash_binding_present = false;
    let mut provider_separation_missing = valid;
    provider_separation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_provider_separation_present = false;
    let mut normal_module_separation_missing = valid;
    normal_module_separation_missing
        .persistence_candidate
        .transaction_candidate
        .loader_candidate
        .loader_normal_module_separation_present = false;

    let mut missing_rollback_engine_boundary = valid;
    missing_rollback_engine_boundary
        .persistence_candidate
        .rollback_transaction_engine_available = false;
    let mut previous_rollback_engine = valid;
    previous_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_current_boot = false;
    let mut wrong_schema_rollback_engine = valid;
    wrong_schema_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_schema_ok = false;
    let mut substituted_rollback_engine = valid;
    substituted_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    substituted_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_substituted_record";
    let mut mismatched_rollback_engine = valid;
    mismatched_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_ok = false;
    mismatched_rollback_engine
        .persistence_candidate
        .rollback_transaction_engine_binding_reason =
        "recovery_rollback_transaction_engine_binding_mismatch";

    let mut target_selection_missing = valid;
    target_selection_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_target_selection_present = false;
    let mut transaction_id_missing = valid;
    transaction_id_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_transaction_id_provenance_present = false;
    let mut last_good_missing = valid;
    last_good_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_last_good_binding_present = false;
    let mut disabled_set_missing = valid;
    disabled_set_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_disabled_module_set_binding_present = false;
    let mut artifact_hash_missing = valid;
    artifact_hash_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_artifact_hash_binding_present = false;
    let mut replay_missing = valid;
    replay_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_replay_preconditions_present = false;
    let mut capability_import_missing = valid;
    capability_import_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_recovery_capability_import_present = false;
    let mut atomic_semantics_missing = valid;
    atomic_semantics_missing
        .persistence_candidate
        .transaction_candidate
        .rollback_atomic_apply_abort_semantics_present = false;

    let mut durable_boundary_missing = valid;
    durable_boundary_missing.durable_audit_rollback_persistence_available = false;
    let mut previous_durable_persistence = valid;
    previous_durable_persistence.durable_audit_rollback_persistence_current_boot = false;
    let mut wrong_schema_durable_persistence = valid;
    wrong_schema_durable_persistence.durable_audit_rollback_persistence_schema_ok = false;
    let mut substituted_durable_persistence = valid;
    substituted_durable_persistence.durable_audit_rollback_persistence_binding_ok = false;
    substituted_durable_persistence.durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_substituted_record";
    let mut mismatched_durable_persistence = valid;
    mismatched_durable_persistence.durable_audit_rollback_persistence_binding_ok = false;
    mismatched_durable_persistence.durable_audit_rollback_persistence_binding_reason =
        "durable_audit_rollback_persistence_binding_mismatch";

    let mut persistence_device_missing = valid;
    persistence_device_missing
        .persistence_candidate
        .persistence_device_inventory_present = false;
    let mut storage_layout_missing = valid;
    storage_layout_missing
        .persistence_candidate
        .storage_layout_identity_present = false;
    let mut audit_log_missing = valid;
    audit_log_missing
        .persistence_candidate
        .audit_append_log_identity_present = false;
    let mut rollback_store_missing = valid;
    rollback_store_missing
        .persistence_candidate
        .rollback_store_identity_present = false;
    let mut replay_cursor_missing = valid;
    replay_cursor_missing
        .persistence_candidate
        .transaction_replay_cursor_present = false;
    let mut checkpoint_missing = valid;
    checkpoint_missing
        .persistence_candidate
        .last_good_checkpoint_binding_present = false;
    let mut write_ordering_missing = valid;
    write_ordering_missing
        .persistence_candidate
        .write_ordering_present = false;
    let mut crash_consistency_missing = valid;
    crash_consistency_missing
        .persistence_candidate
        .crash_consistency_present = false;
    let mut integrity_root_missing = valid;
    integrity_root_missing
        .persistence_candidate
        .integrity_root_hash_chain_present = false;
    let mut all_memory_provenance_missing = valid;
    all_memory_provenance_missing.source_record_ids_present = false;
    all_memory_provenance_missing.source_schema_hashes_present = false;
    all_memory_provenance_missing.memory_classification_present = false;
    all_memory_provenance_missing.memory_authority_level_present = false;
    all_memory_provenance_missing.memory_rollback_transaction_binding_present = false;
    all_memory_provenance_missing.memory_last_good_checkpoint_binding_present = false;
    all_memory_provenance_missing.recovery_only_export_profile_present = false;
    all_memory_provenance_missing.memory_redaction_state_present = false;
    all_memory_provenance_missing.memory_replay_window_present = false;
    all_memory_provenance_missing.memory_audit_linkage_present = false;

    let mut source_record_ids_missing = valid;
    source_record_ids_missing.source_record_ids_present = false;
    let mut source_schema_hashes_missing = valid;
    source_schema_hashes_missing.source_schema_hashes_present = false;
    let mut memory_classification_missing = valid;
    memory_classification_missing.memory_classification_present = false;
    let mut memory_authority_missing = valid;
    memory_authority_missing.memory_authority_level_present = false;
    let mut memory_rollback_binding_missing = valid;
    memory_rollback_binding_missing.memory_rollback_transaction_binding_present = false;
    let mut memory_checkpoint_binding_missing = valid;
    memory_checkpoint_binding_missing.memory_last_good_checkpoint_binding_present = false;
    let mut export_profile_missing = valid;
    export_profile_missing.recovery_only_export_profile_present = false;
    let mut redaction_state_missing = valid;
    redaction_state_missing.memory_redaction_state_present = false;
    let mut replay_window_missing = valid;
    replay_window_missing.memory_replay_window_present = false;
    let mut audit_linkage_missing = valid;
    audit_linkage_missing.memory_audit_linkage_present = false;

    [
        recovery_memory_provenance_selftest_case(
            "missing_lifeline_request_event_id",
            "missing",
            "recovery_lifeline_request_event_id_missing",
            evaluate_recovery_memory_provenance(missing_request),
        ),
        recovery_memory_provenance_selftest_case(
            "stale_dropped_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_stale_or_dropped",
            evaluate_recovery_memory_provenance(stale_request),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_request),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_request_event_id",
            "rejected",
            "recovery_lifeline_request_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_request),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_request_record",
            "rejected",
            "recovery_lifeline_request_substituted_record",
            evaluate_recovery_memory_provenance(substituted_request),
        ),
        recovery_memory_provenance_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "rejected",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_memory_provenance(request_hash_mismatch),
        ),
        recovery_memory_provenance_selftest_case(
            "protocol_state_missing_after_valid_request",
            "denied_missing_lifeline_protocol_state",
            "recovery_lifeline_protocol_state_missing",
            evaluate_recovery_memory_provenance(missing_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_protocol_state",
            "rejected",
            "recovery_lifeline_protocol_state_substituted_record",
            evaluate_recovery_memory_provenance(substituted_protocol_state),
        ),
        recovery_memory_provenance_selftest_case(
            "command_vocabulary_missing_after_protocol_state",
            "denied_missing_lifeline_command_vocabulary",
            "recovery_lifeline_command_vocabulary_missing",
            evaluate_recovery_memory_provenance(missing_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_lifeline_command_vocabulary",
            "rejected",
            "recovery_lifeline_command_vocabulary_substituted_record",
            evaluate_recovery_memory_provenance(substituted_command_vocabulary),
        ),
        recovery_memory_provenance_selftest_case(
            "direct_openai_recovery_shortcut_rejected",
            "rejected",
            "direct_openai_provider_path_not_recovery_lifeline",
            evaluate_recovery_memory_provenance(direct_provider_shortcut),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_runtime_isolation_missing_after_command_vocabulary",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_runtime_isolation_missing",
            evaluate_recovery_memory_provenance(missing_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_loader_runtime_isolation",
            "rejected",
            "recovery_loader_runtime_isolation_substituted_record",
            evaluate_recovery_memory_provenance(substituted_loader_runtime_isolation),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_address_space_boundary_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_address_space_boundary_missing",
            evaluate_recovery_memory_provenance(address_space_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_entrypoint_abi_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_entrypoint_abi_missing",
            evaluate_recovery_memory_provenance(entrypoint_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_memory_map_constraints_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_memory_map_constraints_missing",
            evaluate_recovery_memory_provenance(memory_map_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_capability_import_table_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_capability_import_table_missing",
            evaluate_recovery_memory_provenance(import_table_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_artifact_hash_binding_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_artifact_hash_binding_missing",
            evaluate_recovery_memory_provenance(artifact_hash_isolation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_provider_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_provider_separation_missing",
            evaluate_recovery_memory_provenance(provider_separation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "loader_normal_module_separation_missing",
            "denied_missing_loader_runtime_isolation",
            "recovery_loader_normal_module_separation_missing",
            evaluate_recovery_memory_provenance(normal_module_separation_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_transaction_engine_boundary_missing_after_loader",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_engine_missing",
            evaluate_recovery_memory_provenance(missing_rollback_engine_boundary),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_substituted_record",
            evaluate_recovery_memory_provenance(substituted_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "mismatched_rollback_transaction_engine",
            "rejected",
            "recovery_rollback_transaction_engine_binding_mismatch",
            evaluate_recovery_memory_provenance(mismatched_rollback_engine),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_target_selection_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_target_selection_missing",
            evaluate_recovery_memory_provenance(target_selection_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_transaction_id_provenance_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_transaction_id_provenance_missing",
            evaluate_recovery_memory_provenance(transaction_id_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_last_good_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_last_good_binding_missing",
            evaluate_recovery_memory_provenance(last_good_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_disabled_module_set_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_disabled_module_set_binding_missing",
            evaluate_recovery_memory_provenance(disabled_set_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_artifact_hash_binding_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_artifact_hash_binding_missing",
            evaluate_recovery_memory_provenance(artifact_hash_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_replay_preconditions_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_replay_preconditions_missing",
            evaluate_recovery_memory_provenance(replay_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_recovery_capability_import_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_recovery_capability_import_missing",
            evaluate_recovery_memory_provenance(capability_import_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_atomic_apply_abort_semantics_missing",
            "denied_missing_rollback_transaction_engine",
            "recovery_rollback_atomic_apply_abort_semantics_missing",
            evaluate_recovery_memory_provenance(atomic_semantics_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "durable_persistence_boundary_missing_after_rollback_engine",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_rollback_persistence_missing",
            evaluate_recovery_memory_provenance(durable_boundary_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "previous_boot_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_event_id_not_current_boot",
            evaluate_recovery_memory_provenance(previous_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "wrong_schema_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_wrong_schema_or_variant",
            evaluate_recovery_memory_provenance(wrong_schema_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "substituted_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_substituted_record",
            evaluate_recovery_memory_provenance(substituted_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "mismatched_durable_persistence",
            "rejected",
            "durable_audit_rollback_persistence_binding_mismatch",
            evaluate_recovery_memory_provenance(mismatched_durable_persistence),
        ),
        recovery_memory_provenance_selftest_case(
            "persistence_device_inventory_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "persistence_device_inventory_missing",
            evaluate_recovery_memory_provenance(persistence_device_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "storage_layout_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_storage_layout_identity_missing",
            evaluate_recovery_memory_provenance(storage_layout_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "audit_append_log_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_audit_append_log_identity_missing",
            evaluate_recovery_memory_provenance(audit_log_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "rollback_store_identity_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_store_identity_missing",
            evaluate_recovery_memory_provenance(rollback_store_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "transaction_replay_cursor_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "rollback_transaction_replay_cursor_missing",
            evaluate_recovery_memory_provenance(replay_cursor_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "last_good_checkpoint_binding_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "recovery_last_good_checkpoint_binding_missing",
            evaluate_recovery_memory_provenance(checkpoint_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "write_ordering_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_write_ordering_missing",
            evaluate_recovery_memory_provenance(write_ordering_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "crash_consistency_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_crash_consistency_missing",
            evaluate_recovery_memory_provenance(crash_consistency_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "integrity_root_hash_chain_missing",
            "denied_missing_durable_audit_rollback_persistence",
            "durable_integrity_root_hash_chain_missing",
            evaluate_recovery_memory_provenance(integrity_root_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "recovery_memory_provenance_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_provenance_missing",
            evaluate_recovery_memory_provenance(all_memory_provenance_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "source_record_ids_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_record_ids_missing",
            evaluate_recovery_memory_provenance(source_record_ids_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "source_schema_hashes_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_source_schema_hashes_missing",
            evaluate_recovery_memory_provenance(source_schema_hashes_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_classification_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_classification_missing",
            evaluate_recovery_memory_provenance(memory_classification_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_authority_level_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_authority_level_missing",
            evaluate_recovery_memory_provenance(memory_authority_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_rollback_transaction_binding_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_rollback_transaction_binding_missing",
            evaluate_recovery_memory_provenance(memory_rollback_binding_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_last_good_checkpoint_binding_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_last_good_checkpoint_binding_missing",
            evaluate_recovery_memory_provenance(memory_checkpoint_binding_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "recovery_only_export_profile_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_only_export_profile_missing",
            evaluate_recovery_memory_provenance(export_profile_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_redaction_state_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_redaction_state_missing",
            evaluate_recovery_memory_provenance(redaction_state_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_replay_window_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_replay_window_missing",
            evaluate_recovery_memory_provenance(replay_window_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "memory_audit_linkage_missing",
            "denied_missing_recovery_memory_provenance",
            "recovery_memory_audit_linkage_missing",
            evaluate_recovery_memory_provenance(audit_linkage_missing),
        ),
        recovery_memory_provenance_selftest_case(
            "all_inputs_present_memory_still_non_executable",
            "defined_non_executable",
            "recovery_memory_provenance_behavior_not_implemented",
            evaluate_recovery_memory_provenance(valid),
        ),
    ]
}

fn recovery_memory_provenance_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryMemoryProvenanceCheck,
) -> RecoveryMemoryProvenanceSelfTestCase {
    RecoveryMemoryProvenanceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason),
    }
}

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

fn recovery_lifeline_command_admission_check(
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

fn recovery_lifeline_command_admission_selftest_case(
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
