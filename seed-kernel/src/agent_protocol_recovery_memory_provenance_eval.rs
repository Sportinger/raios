use crate::{
    agent_protocol_recovery_constants::RECOVERY_MEMORY_PROVENANCE_SELFTEST_CASES,
    agent_protocol_recovery_lifeline_command_vocabulary_eval::*,
    agent_protocol_recovery_lifeline_protocol_eval::*,
    agent_protocol_recovery_loader_runtime_eval::*, agent_protocol_recovery_persistence_eval::*,
    agent_protocol_recovery_rollback_transaction_eval::*, agent_protocol_recovery_runtime_types::*,
    agent_protocol_support::method_eq,
};

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

pub(crate) fn recovery_memory_provenance_check(
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

pub(crate) fn recovery_memory_provenance_valid_candidate() -> RecoveryMemoryProvenanceCandidate {
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

pub(crate) fn recovery_memory_provenance_selftest_case(
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
