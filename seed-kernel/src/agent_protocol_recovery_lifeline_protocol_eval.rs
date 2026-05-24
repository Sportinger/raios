use crate::{
    agent_protocol_recovery_constants::*, agent_protocol_recovery_lifeline_protocol_types::*,
    agent_protocol_support::method_eq,
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

pub(crate) fn recovery_lifeline_protocol_check(
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

pub(crate) fn recovery_lifeline_protocol_selftest_case(
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
