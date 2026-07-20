use super::*;

pub(crate) fn hello_rollback_preview_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", HELLO_ROLLBACK_PREVIEW_SCHEMA);
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_PREVIEW_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", HELLO_ROLLBACK_PREVIEW_STATUS);
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_artifact_identity_id",
        probation.previous_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_artifact_identity_id",
        probation.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_generation",
        probation.previous_generation,
    );
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_hash(
        &mut hash,
        b"rollback_target_state_sha256",
        probation.previous_state_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_state_counter",
        probation.previous_state_counter,
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"read_only", true);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_apply_denial_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    hello_rollback_apply_denial_hash_with_retained_sources(snapshot, probation, None, None)
}

pub(crate) fn hello_rollback_apply_denial_hash_with_retained_sources(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    durable_policy_write_authority_decision: Option<RollbackDurablePolicyWriteAuthorityDecision>,
    recovery_rollback_inspect_source_reference_state: Option<
        RecoveryRollbackInspectSourceReferenceState,
    >,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", HELLO_ROLLBACK_APPLY_SCHEMA);
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_APPLY_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", HELLO_ROLLBACK_APPLY_STATUS);
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_str(
        &mut hash,
        b"current_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_bool(&mut hash, b"authorized", false);
    hash_line_bool(&mut hash, b"mutates_service_state", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    if let Some(decision) = durable_policy_write_authority_decision {
        hash_line_str(
            &mut hash,
            b"source_durable_policy_write_authority_decision_schema",
            HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA,
        );
        hash_line_str(
            &mut hash,
            b"source_durable_policy_write_authority_decision_id",
            HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID,
        );
        hash_line_str(
            &mut hash,
            b"source_durable_policy_write_authority_decision_status",
            HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS,
        );
        hash_line_str(
            &mut hash,
            b"source_durable_policy_write_authority_decision_reason",
            HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON,
        );
        hash_line_hash(
            &mut hash,
            b"source_durable_policy_write_authority_decision_sha256",
            decision.decision_hash,
        );
        hash_line_hash(
            &mut hash,
            b"source_durable_policy_write_authority_decision_target_region_sector_inspection_sha256",
            decision.source_target_region_sector_inspection_hash,
        );
        hash_line_bool(
            &mut hash,
            b"source_durable_policy_write_authority_decision_verified",
            decision.transaction_append_dry_run_verified
                && decision.target_region_sector_inspection_verified
                && decision.write_authority_evidence_verified
                && decision.audit_policy_availability_evidence_verified
                && decision.durable_append_authority_availability_evidence_verified
                && decision.target_span_verified,
        );
        hash_line_bool(
            &mut hash,
            b"source_durable_policy_write_authority_decision_authorizes_rollback_apply",
            false,
        );
    }
    if let Some(state) = recovery_rollback_inspect_source_reference_state {
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_schema",
            HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SCHEMA,
        );
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_id",
            HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_ID,
        );
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_status",
            state.status,
        );
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_reason",
            state.reason,
        );
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_ram_audit_status",
            state.ram_audit_status,
        );
        hash_line_str(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_ram_audit_reason",
            state.ram_audit_reason,
        );
        if let Some(reference) = state.reference {
            hash_line_u64(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_event_sequence",
                reference.event_id.sequence(),
            );
            hash_line_u64(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_audit_event_sequence",
                reference.audit_event_id.sequence(),
            );
            hash_line_hash(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_sha256",
                reference.reference_hash,
            );
            hash_line_hash(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_inspection_sha256",
                reference.inspection_hash,
            );
            hash_line_hash(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_sector_plan_sha256",
                reference.source_sector_plan_hash,
            );
            hash_line_hash(
                &mut hash,
                b"source_recovery_rollback_inspect_source_reference_target_region_write_readback_sha256",
                reference.source_target_region_write_readback_hash,
            );
        }
        hash_line_bool(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_source_event_retained",
            state.source_event_retained,
        );
        hash_line_bool(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_audit_event_retained",
            state.audit_event_retained,
        );
        hash_line_bool(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_ram_audit_validated",
            state.ram_audit_validated,
        );
        hash_line_bool(
            &mut hash,
            b"source_recovery_rollback_inspect_source_reference_authorizes_rollback_apply",
            false,
        );
    }
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_apply_retained_denial_sources(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> (
    RollbackDurablePolicyWriteAuthorityDecision,
    RecoveryRollbackInspectSourceReferenceState,
) {
    let foundation = rollback_writer_storage_foundation();
    let append_record = hello_rollback_append_record_dry_run(snapshot, probation, foundation);
    let sector_plan = hello_rollback_append_sector_plan_dry_run(snapshot, probation, append_record);
    let sector_write =
        hello_rollback_append_sector_write_readback_dry_run(snapshot, probation, sector_plan);
    let target_region_media_write_policy_preflight =
        hello_target_region_media_write_policy_preflight(foundation);
    let target_region_write = hello_rollback_target_region_write_readback_dry_run_from_materializer(
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
    );
    let durable_writer_policy_preflight = hello_rollback_durable_writer_policy_preflight(
        foundation,
        append_record,
        sector_plan,
        target_region_write,
    );
    let durable_append_preflight = hello_rollback_durable_append_authority_preflight(
        foundation,
        append_record,
        sector_plan,
        sector_write,
        target_region_media_write_policy_preflight,
        target_region_write,
        durable_writer_policy_preflight,
    );
    let media_write_authority_gate =
        hello_rollback_media_write_authority_gate(durable_append_preflight, target_region_write);
    let durable_append_transaction_authorization_gate =
        hello_rollback_durable_append_transaction_authorization_gate(
            durable_writer_policy_preflight,
            append_record,
            sector_plan,
            target_region_write,
        );
    let append_engine_readiness_decision = hello_rollback_append_engine_readiness_decision(
        durable_append_transaction_authorization_gate,
    );
    let durable_append_authority_decision = hello_rollback_durable_append_authority_decision(
        durable_append_preflight,
        media_write_authority_gate,
        append_engine_readiness_decision,
    );
    let durable_audit_policy_decision =
        hello_rollback_durable_audit_policy_decision(durable_append_authority_decision);
    let durable_audit_policy_candidate =
        hello_rollback_durable_audit_policy_candidate(durable_audit_policy_decision, append_record);
    let durable_audit_policy_acceptance_gate =
        hello_rollback_durable_audit_policy_acceptance_gate(durable_audit_policy_candidate);
    let durable_audit_policy_ledger_candidate =
        hello_rollback_durable_audit_policy_ledger_candidate(durable_audit_policy_acceptance_gate);
    let durable_audit_policy_ledger_aware_acceptance_result =
        hello_rollback_durable_audit_policy_ledger_aware_acceptance_result(
            durable_audit_policy_ledger_candidate,
        );
    let durable_audit_policy_write_authority_availability =
        hello_rollback_durable_audit_policy_write_authority_availability(
            durable_audit_policy_ledger_aware_acceptance_result,
            durable_audit_policy_ledger_candidate,
            target_region_media_write_policy_preflight,
            target_region_write,
        );
    let durable_policy_ledger_availability = hello_rollback_durable_policy_ledger_availability(
        durable_audit_policy_write_authority_availability,
    );
    let durable_audit_policy_availability =
        hello_rollback_durable_audit_policy_availability(durable_policy_ledger_availability);
    let durable_append_authority_availability =
        hello_rollback_durable_append_authority_availability(durable_audit_policy_availability);
    let transaction_append_availability_decision =
        hello_rollback_transaction_append_availability_decision(
            durable_append_authority_availability,
            append_engine_readiness_decision,
            durable_writer_policy_preflight,
        );
    let transaction_append_authority_denial_gate =
        hello_rollback_transaction_append_authority_denial_gate(
            transaction_append_availability_decision,
        );
    let durable_policy_ledger_availability_dry_run =
        hello_rollback_durable_policy_ledger_availability_dry_run(
            durable_policy_ledger_availability,
            durable_audit_policy_write_authority_availability,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let durable_audit_policy_availability_dry_run =
        hello_rollback_durable_audit_policy_availability_dry_run(
            durable_audit_policy_availability,
            durable_policy_ledger_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let durable_append_authority_availability_dry_run =
        hello_rollback_durable_append_authority_availability_dry_run(
            durable_append_authority_availability,
            durable_audit_policy_availability_dry_run,
            transaction_append_authority_denial_gate,
            target_region_write,
        );
    let transaction_append_dry_run = hello_rollback_transaction_append_dry_run(
        transaction_append_authority_denial_gate,
        append_record,
        sector_plan,
        target_region_write,
    );
    let target_region_sector_inspection =
        hello_rollback_target_region_sector_inspection_from_retained_inspect(
            append_record,
            sector_plan,
            target_region_write,
        );
    let recovery_rollback_inspect_source_reference_state =
        recovery_rollback_inspect_source_reference_state(target_region_sector_inspection);
    let durable_policy_write_authority_decision =
        hello_rollback_durable_policy_write_authority_decision(
            durable_append_authority_availability_dry_run,
            durable_audit_policy_write_authority_availability,
            durable_audit_policy_availability,
            durable_append_authority_availability,
            transaction_append_dry_run,
            target_region_sector_inspection,
        );
    (
        durable_policy_write_authority_decision,
        recovery_rollback_inspect_source_reference_state,
    )
}

pub(crate) fn hello_rollback_transaction_preflight_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_artifact_identity_id",
        probation.previous_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_generation",
        probation.previous_generation,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_state_sha256",
        probation.previous_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_state_counter",
        probation.previous_state_counter,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_artifact_identity_id",
        probation.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_generation",
        probation.new_generation,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_state_sha256",
        probation.new_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_state_counter",
        probation.new_state_counter,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"rollback_apply_authority_missing", true);
    hash_line_bool(&mut hash, b"rollback_transaction_authority_missing", true);
    hash_line_bool(&mut hash, b"durable_audit_write_authority_missing", true);
    hash_line_bool(&mut hash, b"persistent_install_authority_missing", true);
    hash_line_bool(&mut hash, b"mutates_service_state", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"accepts_external_artifact_bytes", false);
    hash_line_bool(&mut hash, b"loads_candidate_bytes", false);
    hash_line_bool(&mut hash, b"maps_executable_pages", false);
    hash_line_bool(&mut hash, b"provider_auto_load", false);
    hash_line_bool(&mut hash, b"grants_broad_mutation", false);
    finalize_sha256(hash)
}

pub(crate) fn rollback_write_authority_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_str(&mut hash, b"required_audit_schema", "raios.audit_record.v0");
    hash_line_str(
        &mut hash,
        b"required_rollback_schema",
        "raios.rollback_transaction.v0",
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_preflight_sha256",
        hello_rollback_transaction_preflight_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"durable_audit_write_authority_available", false);
    hash_line_bool(
        &mut hash,
        b"rollback_store_write_authority_available",
        false,
    );
    hash_line_bool(&mut hash, b"rollback_transaction_append_available", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    finalize_sha256(hash)
}

pub(crate) fn rollback_append_intent_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_APPEND_INTENT_GATE_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_APPEND_INTENT_GATE_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_APPEND_INTENT_GATE_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_str(&mut hash, b"required_audit_schema", "raios.audit_record.v0");
    hash_line_str(
        &mut hash,
        b"required_rollback_schema",
        "raios.rollback_transaction.v0",
    );
    hash_line_hash(
        &mut hash,
        b"rollback_write_authority_gate_sha256",
        rollback_write_authority_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_preflight_sha256",
        hello_rollback_transaction_preflight_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"append_intent_available", false);
    hash_line_bool(&mut hash, b"rollback_transaction_append_available", false);
    hash_line_bool(&mut hash, b"durable_audit_store_available", false);
    hash_line_bool(&mut hash, b"rollback_store_available", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_transaction_payload_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_TRANSACTION_PAYLOAD_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_append_intent_gate_sha256",
        rollback_append_intent_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_write_authority_gate_sha256",
        rollback_write_authority_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_preflight_sha256",
        hello_rollback_transaction_preflight_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_artifact_identity_id",
        probation.previous_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_generation",
        probation.previous_generation,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_state_sha256",
        probation.previous_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_state_counter",
        probation.previous_state_counter,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_artifact_identity_id",
        probation.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_generation",
        probation.new_generation,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_state_sha256",
        probation.new_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_state_counter",
        probation.new_state_counter,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"proposed_only", true);
    hash_line_bool(&mut hash, b"appended_to_rollback_log", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_transaction_payload_provenance_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        "raios.rollback_transaction_provenance.v0",
    );
    hash_line_str(
        &mut hash,
        b"id",
        "rollback_transaction_provenance.current_boot.svc.demo.hello.v0",
    );
    hash_line_str(&mut hash, b"source_method", "service.rollback_apply");
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_hash(
        &mut hash,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_append_intent_gate_sha256",
        rollback_append_intent_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_write_authority_gate_sha256",
        rollback_write_authority_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_preflight_sha256",
        hello_rollback_transaction_preflight_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_bool(&mut hash, b"derived_from_current_boot_ram_evidence", true);
    hash_line_bool(&mut hash, b"durable_record_created", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_audit_append_record_image_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", "raios.audit_record.v0");
    hash_line_str(
        &mut hash,
        b"append_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_hash(
        &mut hash,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_bool(&mut hash, b"write_attempted", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_audit_append_record_image_byte_length() -> u64 {
    canonical_line_str_len(b"schema", "raios.audit_record.v0")
        + canonical_line_str_len(
            b"append_target_id",
            rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
        )
        + canonical_line_hash_len(b"payload_sha256")
        + canonical_line_hash_len(b"provenance_sha256")
        + canonical_line_hash_len(b"rollback_apply_sha256")
        + canonical_line_bool_len(b"write_attempted", false)
}

pub(crate) fn hello_rollback_transaction_append_record_image_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"append_target_id",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID,
    );
    hash_line_hash(
        &mut hash,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    hash_line_bool(&mut hash, b"appended", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    finalize_sha256(hash)
}

pub(crate) fn hello_rollback_transaction_append_record_image_byte_length() -> u64 {
    canonical_line_str_len(b"schema", HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA)
        + canonical_line_str_len(
            b"append_target_id",
            HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID,
        )
        + canonical_line_hash_len(b"payload_sha256")
        + canonical_line_hash_len(b"provenance_sha256")
        + canonical_line_bool_len(b"appended", false)
        + canonical_line_bool_len(b"write_attempted", false)
}

pub(crate) fn hello_rollback_append_sector_image(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; ahci::SECTOR_BYTES] {
    let mut image = [0u8; ahci::SECTOR_BYTES];
    let mut offset = 0usize;
    write_canonical_line_str(&mut image, &mut offset, b"schema", "raios.audit_record.v0");
    write_canonical_line_str(
        &mut image,
        &mut offset,
        b"append_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    write_canonical_line_hash(
        &mut image,
        &mut offset,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    write_canonical_line_hash(
        &mut image,
        &mut offset,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    write_canonical_line_hash(
        &mut image,
        &mut offset,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    write_canonical_line_bool(&mut image, &mut offset, b"write_attempted", false);
    write_canonical_line_str(
        &mut image,
        &mut offset,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA,
    );
    write_canonical_line_str(
        &mut image,
        &mut offset,
        b"append_target_id",
        HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID,
    );
    write_canonical_line_hash(
        &mut image,
        &mut offset,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    write_canonical_line_hash(
        &mut image,
        &mut offset,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    write_canonical_line_bool(&mut image, &mut offset, b"appended", false);
    write_canonical_line_bool(&mut image, &mut offset, b"write_attempted", false);
    image
}

pub(crate) fn hello_rollback_append_record_dry_run(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    foundation: RollbackWriterStorageFoundation,
) -> RollbackAppendRecordDryRun {
    let audit_record_byte_length = hello_rollback_audit_append_record_image_byte_length();
    let rollback_transaction_byte_length =
        hello_rollback_transaction_append_record_image_byte_length();
    let total_record_byte_length = audit_record_byte_length + rollback_transaction_byte_length;
    let target_byte_count = foundation.scratch_region_byte_count as u64;
    let target_lba_count = if target_byte_count == 0 {
        0
    } else {
        (total_record_byte_length + target_byte_count - 1) / target_byte_count
    };
    let target_range_ready = foundation.scratch_writer_dry_run_ready
        && target_lba_count <= foundation.scratch_region_lba_count;
    let audit_record_image_hash =
        hello_rollback_audit_append_record_image_hash(snapshot, probation);
    let rollback_transaction_image_hash =
        hello_rollback_transaction_append_record_image_hash(snapshot, probation);
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"canonicalization",
        HELLO_ROLLBACK_APPEND_RECORD_CANONICALIZATION,
    );
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"audit_record_image_sha256",
        audit_record_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"audit_record_byte_length",
        audit_record_byte_length,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_image_sha256",
        rollback_transaction_image_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_transaction_byte_length",
        rollback_transaction_byte_length,
    );
    hash_line_u64(
        &mut hash,
        b"total_record_byte_length",
        total_record_byte_length,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        foundation.scratch_region_start_lba,
    );
    hash_line_u64(&mut hash, b"target_lba_count", target_lba_count);
    hash_line_u64(&mut hash, b"target_byte_count", target_byte_count);
    hash_line_bool(&mut hash, b"target_range_ready", target_range_ready);
    hash_line_hash(
        &mut hash,
        b"payload_sha256",
        hello_rollback_transaction_payload_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"provenance_sha256",
        hello_rollback_transaction_payload_provenance_hash(snapshot, probation),
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackAppendRecordDryRun {
        dry_run_hash: finalize_sha256(hash),
        audit_record_image_hash,
        rollback_transaction_image_hash,
        audit_record_byte_length,
        rollback_transaction_byte_length,
        total_record_byte_length,
        target_start_lba: foundation.scratch_region_start_lba,
        target_lba_count,
        target_byte_count,
        target_range_ready,
    }
}

pub(crate) fn hello_rollback_append_sector_plan_dry_run(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    append_record: RollbackAppendRecordDryRun,
) -> RollbackAppendSectorPlanDryRun {
    let audit_record_offset = 0;
    let rollback_transaction_offset = append_record.audit_record_byte_length;
    let padding_offset = append_record.total_record_byte_length;
    let padding_byte_length = append_record
        .target_byte_count
        .saturating_sub(append_record.total_record_byte_length);
    let sector_size_bytes = append_record.target_byte_count;
    let target_range_ready = append_record.target_range_ready
        && append_record.target_lba_count == 1
        && append_record.total_record_byte_length <= sector_size_bytes;

    let sector_image = hello_rollback_append_sector_image(snapshot, probation);
    let sector_image_hash = sha256_bytes(&sector_image);

    let mut plan_hash = Sha256::new();
    hash_line_str(
        &mut plan_hash,
        b"schema",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut plan_hash,
        b"id",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID,
    );
    hash_line_str(&mut plan_hash, b"scope", "current_boot");
    hash_line_str(&mut plan_hash, b"classification", "local_only");
    hash_line_str(&mut plan_hash, b"persistence", "none");
    hash_line_str(
        &mut plan_hash,
        b"canonicalization",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_CANONICALIZATION,
    );
    hash_line_str(
        &mut plan_hash,
        b"status",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut plan_hash,
        b"reason",
        HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut plan_hash,
        b"append_record_dry_run_sha256",
        append_record.dry_run_hash,
    );
    hash_line_hash(&mut plan_hash, b"sector_image_sha256", sector_image_hash);
    hash_line_u64(&mut plan_hash, b"sector_size_bytes", sector_size_bytes);
    hash_line_u64(&mut plan_hash, b"audit_record_offset", audit_record_offset);
    hash_line_u64(
        &mut plan_hash,
        b"rollback_transaction_offset",
        rollback_transaction_offset,
    );
    hash_line_str(
        &mut plan_hash,
        b"padding_policy",
        HELLO_ROLLBACK_APPEND_SECTOR_PADDING_POLICY,
    );
    hash_line_u64(&mut plan_hash, b"padding_offset", padding_offset);
    hash_line_u64(&mut plan_hash, b"padding_byte_length", padding_byte_length);
    hash_line_u64(
        &mut plan_hash,
        b"target_start_lba",
        append_record.target_start_lba,
    );
    hash_line_u64(
        &mut plan_hash,
        b"target_lba_count",
        append_record.target_lba_count,
    );
    hash_line_u64(
        &mut plan_hash,
        b"target_byte_count",
        append_record.target_byte_count,
    );
    hash_line_bool(&mut plan_hash, b"target_range_ready", target_range_ready);
    hash_line_bool(&mut plan_hash, b"authorizes_append", false);
    hash_line_bool(&mut plan_hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut plan_hash, b"writes_rollback_store", false);
    hash_line_bool(&mut plan_hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut plan_hash, b"write_attempted", false);

    RollbackAppendSectorPlanDryRun {
        plan_hash: finalize_sha256(plan_hash),
        sector_image_hash,
        sector_size_bytes,
        audit_record_offset,
        audit_record_byte_length: append_record.audit_record_byte_length,
        rollback_transaction_offset,
        rollback_transaction_byte_length: append_record.rollback_transaction_byte_length,
        padding_offset,
        padding_byte_length,
        target_start_lba: append_record.target_start_lba,
        target_lba_count: append_record.target_lba_count,
        target_byte_count: append_record.target_byte_count,
        target_range_ready,
    }
}

pub(crate) fn hello_rollback_append_sector_write_readback_dry_run(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    sector_plan: RollbackAppendSectorPlanDryRun,
) -> RollbackAppendSectorWriteReadbackDryRun {
    let planned_image = hello_rollback_append_sector_image(snapshot, probation);
    let evidence = pci::find_mass_storage_controller().map(|controller| {
        ahci::write_readback_scratch_sector_image(
            controller,
            &planned_image,
            sector_plan.sector_image_hash,
        )
    });
    let (
        planned_sector_image_hash,
        readback_sector_image_hash,
        label_found,
        target_range_ready,
        write_attempted,
        write_completed,
        readback_completed,
        readback_matches_planned_image,
        status,
        reason,
    ) = match evidence {
        Some(evidence) => (
            evidence.planned_image_hash,
            evidence.readback_image_hash,
            evidence.label_found,
            sector_plan.target_range_ready
                && evidence.available
                && evidence.region_id == ahci::SCRATCH_BLOCK_REGION_ID
                && evidence.target_lba == sector_plan.target_start_lba
                && evidence.byte_count as u64 == sector_plan.target_byte_count
                && evidence.region_within_device_bounds
                && evidence.no_boot_or_partition_metadata_overlap,
            evidence.write_attempted,
            evidence.write_completed,
            evidence.readback_completed,
            evidence.readback_matches_planned_image,
            if evidence.available {
                HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_STATUS
            } else {
                "missing"
            },
            if evidence.available {
                HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_REASON
            } else {
                evidence.reason
            },
        ),
        None => (
            sector_plan.sector_image_hash,
            [0; 32],
            false,
            false,
            false,
            false,
            false,
            false,
            "missing",
            "pci_mass_storage_controller_missing",
        ),
    };
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", status);
    hash_line_str(&mut hash, b"reason", reason);
    hash_line_hash(&mut hash, b"source_plan_sha256", sector_plan.plan_hash);
    hash_line_hash(
        &mut hash,
        b"planned_sector_image_sha256",
        planned_sector_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"readback_sector_image_sha256",
        readback_sector_image_hash,
    );
    hash_line_u64(&mut hash, b"target_start_lba", sector_plan.target_start_lba);
    hash_line_u64(&mut hash, b"target_lba_count", sector_plan.target_lba_count);
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        sector_plan.target_byte_count,
    );
    hash_line_bool(&mut hash, b"label_found", label_found);
    hash_line_bool(&mut hash, b"target_range_ready", target_range_ready);
    hash_line_bool(&mut hash, b"scratch_write_attempted", write_attempted);
    hash_line_bool(&mut hash, b"scratch_write_completed", write_completed);
    hash_line_bool(&mut hash, b"scratch_readback_completed", readback_completed);
    hash_line_bool(
        &mut hash,
        b"readback_matches_planned_image",
        readback_matches_planned_image,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    RollbackAppendSectorWriteReadbackDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_plan_hash: sector_plan.plan_hash,
        planned_sector_image_hash,
        readback_sector_image_hash,
        target_start_lba: sector_plan.target_start_lba,
        target_lba_count: sector_plan.target_lba_count,
        target_byte_count: sector_plan.target_byte_count,
        label_found,
        target_range_ready,
        write_attempted,
        write_completed,
        readback_completed,
        readback_matches_planned_image,
        status,
        reason,
    }
}

pub(crate) fn hello_target_region_media_write_policy_preflight(
    foundation: RollbackWriterStorageFoundation,
) -> TargetRegionMediaWritePolicyPreflight {
    let target_region = foundation.target_region_discovery;
    let target_byte_count = target_region.candidate_region_lba_count * ahci::SECTOR_BYTES as u64;
    let media_write_authority_available = false;
    let durable_audit_policy_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        foundation.target_region_media_write_policy_preflight_status,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        foundation.target_region_media_write_policy_preflight_reason,
    );
    hash_line_str(
        &mut hash,
        b"source_contract_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"source_contract_id",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID,
    );
    hash_line_str(
        &mut hash,
        b"source_contract_status",
        foundation.target_region_writer_contract_status,
    );
    hash_line_str(
        &mut hash,
        b"source_contract_reason",
        foundation.target_region_writer_contract_reason,
    );
    hash_line_str(
        &mut hash,
        b"owner_method",
        rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_id",
        rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID,
    );
    hash_line_str(
        &mut hash,
        b"storage_authority_id",
        rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_start_lba",
        target_region.candidate_region_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_lba_count",
        target_region.candidate_region_lba_count,
    );
    hash_line_u64(&mut hash, b"target_byte_count", target_byte_count);
    hash_line_bool(
        &mut hash,
        b"source_contract_target_range_ready",
        foundation.target_region_writer_contract_ready,
    );
    hash_line_bool(&mut hash, b"owner_ids_verified", true);
    hash_line_bool(&mut hash, b"target_ids_verified", true);
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        foundation.target_region_writer_contract_ready,
    );
    hash_line_bool(&mut hash, b"schema_ids_verified", true);
    hash_line_bool(&mut hash, b"media_write_authority_required", true);
    hash_line_bool(
        &mut hash,
        b"media_write_authority_available",
        media_write_authority_available,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_reason",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_MISSING_REASON,
    );
    hash_line_bool(&mut hash, b"durable_audit_policy_required", true);
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_audit_policy_available,
    );
    hash_line_str(
        &mut hash,
        b"durable_audit_policy_reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    TargetRegionMediaWritePolicyPreflight {
        preflight_hash: finalize_sha256(hash),
        source_contract_status: foundation.target_region_writer_contract_status,
        source_contract_reason: foundation.target_region_writer_contract_reason,
        source_contract_target_range_ready: foundation.target_region_writer_contract_ready,
        owner_ids_verified: true,
        target_ids_verified: true,
        target_span_verified: foundation.target_region_writer_contract_ready,
        schema_ids_verified: true,
        target_region_start_lba: target_region.candidate_region_start_lba,
        target_region_lba_count: target_region.candidate_region_lba_count,
        target_byte_count,
        media_write_authority_available,
        durable_audit_policy_available,
    }
}

pub(crate) fn hello_rollback_durable_writer_policy_preflight(
    _foundation: RollbackWriterStorageFoundation,
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurableWriterPolicyPreflight {
    let target_range_ready = append_record.target_range_ready
        && sector_plan.target_range_ready
        && target_region_write.target_range_ready
        && append_record.target_start_lba == target_region_write.target_start_lba
        && append_record.target_lba_count == target_region_write.target_lba_count
        && append_record.target_byte_count == target_region_write.target_byte_count
        && sector_plan.target_start_lba == target_region_write.target_start_lba
        && sector_plan.target_lba_count == target_region_write.target_lba_count
        && sector_plan.target_byte_count == target_region_write.target_byte_count;
    let durable_audit_writer_available = target_range_ready
        && append_record.audit_record_byte_length > 0
        && append_record.audit_record_byte_length <= target_region_write.target_byte_count
        && target_region_write.test_infrastructure_media_write_authority_available
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let rollback_store_writer_available = target_range_ready
        && durable_audit_writer_available
        && append_record.rollback_transaction_byte_length > 0
        && append_record.rollback_transaction_byte_length <= target_region_write.target_byte_count
        && sector_plan.rollback_transaction_byte_length
            == append_record.rollback_transaction_byte_length
        && target_region_write.test_infrastructure_media_write_authority_available
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let transaction_append_writer_available = target_range_ready
        && durable_audit_writer_available
        && rollback_store_writer_available
        && append_record.total_record_byte_length > 0
        && append_record.total_record_byte_length <= target_region_write.target_byte_count
        && sector_plan.audit_record_offset == 0
        && sector_plan.audit_record_byte_length == append_record.audit_record_byte_length
        && sector_plan.rollback_transaction_offset == append_record.audit_record_byte_length
        && sector_plan.rollback_transaction_byte_length
            == append_record.rollback_transaction_byte_length
        && sector_plan.padding_offset == append_record.total_record_byte_length
        && target_region_write.test_infrastructure_media_write_authority_available
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_record_sha256",
        append_record.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        sector_plan.plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        target_region_write.dry_run_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        target_region_write.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        target_region_write.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        target_region_write.target_byte_count,
    );
    hash_line_bool(&mut hash, b"target_range_ready", target_range_ready);
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        target_region_write.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_writer_available",
        durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_store_writer_available",
        rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_writer_available",
        transaction_append_writer_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableWriterPolicyPreflight {
        preflight_hash: finalize_sha256(hash),
        source_append_record_hash: append_record.dry_run_hash,
        source_sector_plan_hash: sector_plan.plan_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        target_start_lba: target_region_write.target_start_lba,
        target_lba_count: target_region_write.target_lba_count,
        target_byte_count: target_region_write.target_byte_count,
        target_range_ready,
        test_infrastructure_media_write_authority_available: target_region_write
            .test_infrastructure_media_write_authority_available,
        durable_audit_writer_available,
        rollback_store_writer_available,
        transaction_append_writer_available,
    }
}

pub(crate) fn hello_rollback_durable_append_transaction_authorization_gate(
    writer_policy: RollbackDurableWriterPolicyPreflight,
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurableAppendTransactionAuthorizationGate {
    let append_engine_available = writer_policy.target_range_ready
        && writer_policy.test_infrastructure_media_write_authority_available
        && append_record.target_range_ready
        && sector_plan.target_range_ready
        && target_region_write.target_range_ready
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_writer_policy_preflight_sha256",
        writer_policy.preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_record_sha256",
        append_record.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        sector_plan.plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        target_region_write.dry_run_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_ledger_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"rollback_store_target_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"rollback_transaction_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        writer_policy.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        writer_policy.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        writer_policy.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"target_range_ready",
        writer_policy.target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        writer_policy.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_available",
        append_engine_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_writer_available",
        writer_policy.durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_store_writer_available",
        writer_policy.rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_writer_available",
        writer_policy.transaction_append_writer_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAppendTransactionAuthorizationGate {
        gate_hash: finalize_sha256(hash),
        source_writer_policy_preflight_hash: writer_policy.preflight_hash,
        source_append_record_hash: append_record.dry_run_hash,
        source_sector_plan_hash: sector_plan.plan_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        target_start_lba: writer_policy.target_start_lba,
        target_lba_count: writer_policy.target_lba_count,
        target_byte_count: writer_policy.target_byte_count,
        target_range_ready: writer_policy.target_range_ready,
        test_infrastructure_media_write_authority_available: writer_policy
            .test_infrastructure_media_write_authority_available,
        append_engine_available,
        durable_audit_writer_available: writer_policy.durable_audit_writer_available,
        rollback_store_writer_available: writer_policy.rollback_store_writer_available,
        transaction_append_writer_available: writer_policy.transaction_append_writer_available,
    }
}

pub(crate) fn hello_rollback_append_engine_readiness_decision(
    authorization_gate: RollbackDurableAppendTransactionAuthorizationGate,
) -> RollbackAppendEngineReadinessDecision {
    let (status, reason, ready) = if !authorization_gate.target_range_ready {
        (
            "missing",
            "authorization_gate_target_range_not_ready",
            false,
        )
    } else if !authorization_gate.test_infrastructure_media_write_authority_available {
        (
            "missing",
            "authorization_gate_test_media_authority_missing",
            false,
        )
    } else if !authorization_gate.append_engine_available {
        ("missing", "append_engine_candidate_not_ready", false)
    } else if !authorization_gate.durable_audit_writer_available {
        ("missing", "durable_audit_writer_missing", false)
    } else if !authorization_gate.rollback_store_writer_available {
        ("missing", "rollback_store_writer_missing", false)
    } else if !authorization_gate.transaction_append_writer_available {
        ("missing", "transaction_append_writer_missing", false)
    } else {
        ("available", "transaction_append_engine_ready", true)
    };
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", status);
    hash_line_str(&mut hash, b"reason", reason);
    hash_line_hash(
        &mut hash,
        b"source_authorization_gate_sha256",
        authorization_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_writer_policy_preflight_sha256",
        authorization_gate.source_writer_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_record_sha256",
        authorization_gate.source_append_record_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        authorization_gate.source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        authorization_gate.source_target_region_write_readback_hash,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        authorization_gate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        authorization_gate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        authorization_gate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"target_range_ready",
        authorization_gate.target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        authorization_gate.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_available",
        authorization_gate.append_engine_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_writer_available",
        authorization_gate.durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_store_writer_available",
        authorization_gate.rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_writer_available",
        authorization_gate.transaction_append_writer_available,
    );
    hash_line_bool(&mut hash, b"ready", ready);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackAppendEngineReadinessDecision {
        decision_hash: finalize_sha256(hash),
        source_authorization_gate_hash: authorization_gate.gate_hash,
        source_writer_policy_preflight_hash: authorization_gate.source_writer_policy_preflight_hash,
        source_append_record_hash: authorization_gate.source_append_record_hash,
        source_sector_plan_hash: authorization_gate.source_sector_plan_hash,
        source_target_region_write_readback_hash: authorization_gate
            .source_target_region_write_readback_hash,
        target_start_lba: authorization_gate.target_start_lba,
        target_lba_count: authorization_gate.target_lba_count,
        target_byte_count: authorization_gate.target_byte_count,
        target_range_ready: authorization_gate.target_range_ready,
        test_infrastructure_media_write_authority_available: authorization_gate
            .test_infrastructure_media_write_authority_available,
        status,
        reason,
        append_engine_available: authorization_gate.append_engine_available,
        durable_audit_writer_available: authorization_gate.durable_audit_writer_available,
        rollback_store_writer_available: authorization_gate.rollback_store_writer_available,
        transaction_append_writer_available: authorization_gate.transaction_append_writer_available,
        ready,
    }
}

pub(crate) fn hello_rollback_durable_append_authority_preflight(
    foundation: RollbackWriterStorageFoundation,
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    sector_write: RollbackAppendSectorWriteReadbackDryRun,
    target_region_media_write_policy_preflight: TargetRegionMediaWritePolicyPreflight,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
    durable_writer_policy_preflight: RollbackDurableWriterPolicyPreflight,
) -> RollbackDurableAppendAuthorityPreflight {
    let scratch_write_readback_verified = sector_write.target_range_ready
        && sector_write.write_completed
        && sector_write.readback_completed
        && sector_write.readback_matches_planned_image;
    let target_region_discovery = foundation.target_region_discovery;
    let test_infrastructure_media_write_authority_available =
        target_region_write.test_infrastructure_media_write_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_readback_sha256",
        sector_write.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_record_sha256",
        append_record.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        sector_plan.plan_hash,
    );
    hash_line_str(
        &mut hash,
        b"source_target_region_write_readback_schema",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"source_target_region_write_readback_id",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID,
    );
    hash_line_str(
        &mut hash,
        b"source_target_region_write_readback_status",
        target_region_write.status,
    );
    hash_line_str(
        &mut hash,
        b"source_target_region_write_readback_reason",
        target_region_write.reason,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        target_region_write.dry_run_hash,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_infrastructure_media_write_authority_available,
    );
    hash_line_str(
        &mut hash,
        b"remaining_denial_reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_REMAINING_DENIAL_REASON,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_schema",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_id",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_status",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"durable_writer_policy_preflight_reason",
        HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"durable_writer_policy_preflight_sha256",
        durable_writer_policy_preflight.preflight_hash,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_test_media_write_authority_available",
        durable_writer_policy_preflight.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_durable_audit_writer_available",
        durable_writer_policy_preflight.durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_rollback_store_writer_available",
        durable_writer_policy_preflight.rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_writer_policy_preflight_transaction_append_writer_available",
        durable_writer_policy_preflight.transaction_append_writer_available,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_id",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_status",
        foundation.target_region_media_write_policy_preflight_status,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_reason",
        foundation.target_region_media_write_policy_preflight_reason,
    );
    hash_line_hash(
        &mut hash,
        b"target_region_media_write_policy_preflight_sha256",
        target_region_media_write_policy_preflight.preflight_hash,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_source_contract_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_source_contract_id",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_source_contract_status",
        target_region_media_write_policy_preflight.source_contract_status,
    );
    hash_line_str(
        &mut hash,
        b"target_region_media_write_policy_preflight_source_contract_reason",
        target_region_media_write_policy_preflight.source_contract_reason,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_source_contract_target_range_ready",
        target_region_media_write_policy_preflight.source_contract_target_range_ready,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_owner_ids_verified",
        target_region_media_write_policy_preflight.owner_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_target_ids_verified",
        target_region_media_write_policy_preflight.target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_target_span_verified",
        target_region_media_write_policy_preflight.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_schema_ids_verified",
        target_region_media_write_policy_preflight.schema_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_media_write_authority_available",
        target_region_media_write_policy_preflight.media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_durable_audit_policy_available",
        target_region_media_write_policy_preflight.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_authorizes_media_write",
        false,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_media_write_policy_preflight_write_attempted",
        false,
    );
    hash_line_str(
        &mut hash,
        b"target_region_discovery_schema",
        target_region_discovery.schema,
    );
    hash_line_str(
        &mut hash,
        b"target_region_discovery_id",
        target_region_discovery.id,
    );
    hash_line_str(
        &mut hash,
        b"target_region_discovery_status",
        target_region_discovery.status,
    );
    hash_line_str(
        &mut hash,
        b"target_region_discovery_reason",
        target_region_discovery.reason,
    );
    hash_line_str(
        &mut hash,
        b"target_region_discovery_source",
        target_region_discovery.source,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_partition_inventory_available",
        target_region_discovery.partition_inventory_available,
    );
    hash_line_str(
        &mut hash,
        b"target_region_partition_inventory_scheme",
        target_region_discovery.partition_inventory_scheme,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_partition_entry_count",
        target_region_discovery.partition_entry_count as u64,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_mbr_signature_valid",
        target_region_discovery.mbr_signature_valid,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_candidate_present",
        target_region_discovery.candidate_region_present,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_candidate_start_lba",
        target_region_discovery.candidate_region_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_candidate_lba_count",
        target_region_discovery.candidate_region_lba_count,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_candidate_is_scratch",
        target_region_discovery.candidate_region_is_scratch,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_candidate_overlaps_boot_metadata",
        target_region_discovery.candidate_overlaps_boot_metadata,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_candidate_overlaps_scratch",
        target_region_discovery.candidate_overlaps_scratch,
    );
    hash_line_str(
        &mut hash,
        b"target_region_scratch_region_id",
        target_region_discovery.scratch_region_id,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_scratch_rejected_as_durable_authority",
        target_region_discovery.scratch_rejected_as_durable_authority,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_durable_region_available",
        target_region_discovery.durable_region_available,
    );
    hash_line_str(
        &mut hash,
        b"storage_authority_id",
        rollback_storage_layout::AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID,
    );
    hash_line_str(
        &mut hash,
        b"append_target_owner_id",
        rollback_append_contract::AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID,
    );
    hash_line_str(
        &mut hash,
        b"transaction_writer_readiness_id",
        rollback_append_contract::AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID,
    );
    hash_line_str(
        &mut hash,
        b"audit_ledger_writer_fact_id",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
    );
    hash_line_str(
        &mut hash,
        b"rollback_store_writer_fact_id",
        rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
    );
    hash_line_bool(
        &mut hash,
        b"scratch_write_readback_verified",
        scratch_write_readback_verified,
    );
    hash_line_bool(&mut hash, b"scratch_used_as_durable_authority", false);
    hash_line_bool(
        &mut hash,
        b"durable_audit_writer_available",
        durable_writer_policy_preflight.durable_audit_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_store_writer_available",
        durable_writer_policy_preflight.rollback_store_writer_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_writer_available",
        durable_writer_policy_preflight.transaction_append_writer_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    RollbackDurableAppendAuthorityPreflight {
        preflight_hash: finalize_sha256(hash),
        source_write_readback_hash: sector_write.dry_run_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        durable_writer_policy_preflight,
        target_region_discovery,
        target_region_media_write_policy_preflight,
        scratch_write_readback_verified,
        test_infrastructure_media_write_authority_available,
        remaining_denial_reason: HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_REMAINING_DENIAL_REASON,
        durable_audit_writer_available: durable_writer_policy_preflight
            .durable_audit_writer_available,
        rollback_store_writer_available: durable_writer_policy_preflight
            .rollback_store_writer_available,
        transaction_append_writer_available: durable_writer_policy_preflight
            .transaction_append_writer_available,
    }
}

#[allow(dead_code)]
pub(crate) fn hello_rollback_write_authority_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    rollback_write_authority_gate_hash(snapshot, probation)
}

#[allow(dead_code)]
pub(crate) fn hello_rollback_append_intent_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    rollback_append_intent_gate_hash(snapshot, probation)
}
