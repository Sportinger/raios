use crate::{
    agent_protocol_recovery_constants::RECOVERY_DURABLE_AUDIT_ROLLBACK_PERSISTENCE_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_command_vocabulary_eval::*,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_loader_runtime_eval::*,
    agent_protocol_recovery_rollback_transaction_eval::*, agent_protocol_recovery_runtime_types::*,
    agent_protocol_support::method_eq,
};

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

pub(crate) fn recovery_durable_audit_rollback_persistence_check(
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

pub(crate) fn recovery_durable_audit_rollback_persistence_valid_candidate(
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

pub(crate) fn recovery_durable_audit_rollback_persistence_selftest_case(
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
