use crate::{
    agent_protocol_recovery_constants::RECOVERY_LOADER_RUNTIME_ISOLATION_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_command_vocabulary_eval::*,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_lifeline_protocol_types::*, agent_protocol_recovery_runtime_types::*,
    agent_protocol_support::method_eq,
};

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

pub(crate) fn recovery_loader_runtime_isolation_check(
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

pub(crate) fn recovery_loader_runtime_isolation_valid_candidate(
) -> RecoveryLoaderRuntimeIsolationCandidate {
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

pub(crate) fn recovery_loader_runtime_isolation_selftest_case(
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
