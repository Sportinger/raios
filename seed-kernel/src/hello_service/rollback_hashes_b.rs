use super::*;

pub(crate) fn hello_rollback_media_write_authority_gate(
    durable_append_preflight: RollbackDurableAppendAuthorityPreflight,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackMediaWriteAuthorityGate {
    let policy = durable_append_preflight.target_region_media_write_policy_preflight;
    let test_media_write_authority_available =
        target_region_write.test_infrastructure_media_write_authority_available;
    let media_write_authority_reason = if test_media_write_authority_available {
        HELLO_ROLLBACK_TEST_MEDIA_WRITE_AUTHORITY_REASON
    } else {
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_MISSING_REASON
    };
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_REASON,
    );
    hash_line_str(
        &mut hash,
        b"source_durable_append_authority_preflight_schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"source_durable_append_authority_preflight_id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_preflight_sha256",
        durable_append_preflight.preflight_hash,
    );
    hash_line_str(
        &mut hash,
        b"source_policy_preflight_schema",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"source_policy_preflight_id",
        rollback_append_contract::AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID,
    );
    hash_line_str(
        &mut hash,
        b"source_policy_preflight_status",
        rollback_append_contract::audit_rollback_target_region_media_write_policy_preflight_status(
            policy.source_contract_target_range_ready,
        ),
    );
    hash_line_str(
        &mut hash,
        b"source_policy_preflight_reason",
        rollback_append_contract::audit_rollback_target_region_media_write_policy_preflight_reason(
            policy.source_contract_target_range_ready,
        ),
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        policy.preflight_hash,
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
        policy.source_contract_status,
    );
    hash_line_str(
        &mut hash,
        b"source_contract_reason",
        policy.source_contract_reason,
    );
    hash_line_bool(
        &mut hash,
        b"source_contract_target_range_ready",
        policy.source_contract_target_range_ready,
    );
    hash_line_bool(&mut hash, b"owner_ids_verified", policy.owner_ids_verified);
    hash_line_bool(
        &mut hash,
        b"target_ids_verified",
        policy.target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        policy.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"schema_ids_verified",
        policy.schema_ids_verified,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_start_lba",
        policy.target_region_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_region_lba_count",
        policy.target_region_lba_count,
    );
    hash_line_u64(&mut hash, b"target_byte_count", policy.target_byte_count);
    hash_line_bool(&mut hash, b"media_write_authority_required", true);
    hash_line_bool(
        &mut hash,
        b"media_write_authority_available",
        test_media_write_authority_available,
    );
    hash_line_str(
        &mut hash,
        b"media_write_authority_reason",
        media_write_authority_reason,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_media_write_authority_available,
    );
    hash_line_bool(&mut hash, b"durable_audit_policy_required", true);
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        policy.durable_audit_policy_available,
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
    hash_line_bool(
        &mut hash,
        b"target_region_write_attempted",
        target_region_write.write_attempted,
    );
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackMediaWriteAuthorityGate {
        gate_hash: finalize_sha256(hash),
        source_durable_append_authority_preflight_hash: durable_append_preflight.preflight_hash,
        source_policy_preflight_hash: policy.preflight_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        source_contract_status: policy.source_contract_status,
        source_contract_reason: policy.source_contract_reason,
        source_contract_target_range_ready: policy.source_contract_target_range_ready,
        owner_ids_verified: policy.owner_ids_verified,
        target_ids_verified: policy.target_ids_verified,
        target_span_verified: policy.target_span_verified,
        schema_ids_verified: policy.schema_ids_verified,
        target_region_start_lba: policy.target_region_start_lba,
        target_region_lba_count: policy.target_region_lba_count,
        target_byte_count: policy.target_byte_count,
        media_write_authority_available: test_media_write_authority_available,
        test_infrastructure_media_write_authority_available: test_media_write_authority_available,
        durable_audit_policy_available: policy.durable_audit_policy_available,
        target_region_write_attempted: target_region_write.write_attempted,
    }
}

pub(crate) fn hello_rollback_durable_append_authority_decision(
    durable_append_preflight: RollbackDurableAppendAuthorityPreflight,
    media_write_authority_gate: RollbackMediaWriteAuthorityGate,
    append_engine_readiness_decision: RollbackAppendEngineReadinessDecision,
) -> RollbackDurableAppendAuthorityDecision {
    let writer_policy = durable_append_preflight.durable_writer_policy_preflight;
    let writer_policy_ready = writer_policy.target_range_ready
        && writer_policy.test_infrastructure_media_write_authority_available
        && writer_policy.durable_audit_writer_available
        && writer_policy.rollback_store_writer_available
        && writer_policy.transaction_append_writer_available;
    let media_write_gate_ready = media_write_authority_gate.source_contract_target_range_ready
        && media_write_authority_gate.owner_ids_verified
        && media_write_authority_gate.target_ids_verified
        && media_write_authority_gate.target_span_verified
        && media_write_authority_gate.schema_ids_verified
        && media_write_authority_gate.test_infrastructure_media_write_authority_available
        && media_write_authority_gate.target_region_write_attempted;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_preflight_sha256",
        durable_append_preflight.preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_writer_policy_preflight_sha256",
        writer_policy.preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_engine_readiness_decision_sha256",
        append_engine_readiness_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_media_write_authority_gate_sha256",
        media_write_authority_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        media_write_authority_gate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        media_write_authority_gate.source_target_region_write_readback_hash,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        append_engine_readiness_decision.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        append_engine_readiness_decision.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        append_engine_readiness_decision.target_byte_count,
    );
    hash_line_bool(&mut hash, b"writer_policy_ready", writer_policy_ready);
    hash_line_bool(
        &mut hash,
        b"append_engine_ready",
        append_engine_readiness_decision.ready,
    );
    hash_line_bool(&mut hash, b"media_write_gate_ready", media_write_gate_ready);
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        media_write_authority_gate.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        media_write_authority_gate.durable_audit_policy_available,
    );
    hash_line_bool(&mut hash, b"durable_append_authority_available", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAppendAuthorityDecision {
        decision_hash: finalize_sha256(hash),
        source_durable_append_authority_preflight_hash: durable_append_preflight.preflight_hash,
        source_writer_policy_preflight_hash: writer_policy.preflight_hash,
        source_append_engine_readiness_decision_hash: append_engine_readiness_decision
            .decision_hash,
        source_media_write_authority_gate_hash: media_write_authority_gate.gate_hash,
        source_policy_preflight_hash: media_write_authority_gate.source_policy_preflight_hash,
        source_target_region_write_readback_hash: media_write_authority_gate
            .source_target_region_write_readback_hash,
        target_start_lba: append_engine_readiness_decision.target_start_lba,
        target_lba_count: append_engine_readiness_decision.target_lba_count,
        target_byte_count: append_engine_readiness_decision.target_byte_count,
        writer_policy_ready,
        append_engine_ready: append_engine_readiness_decision.ready,
        media_write_gate_ready,
        test_infrastructure_media_write_authority_available: media_write_authority_gate
            .test_infrastructure_media_write_authority_available,
        durable_audit_policy_available: media_write_authority_gate.durable_audit_policy_available,
        durable_append_authority_available: false,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_decision(
    durable_append_authority_decision: RollbackDurableAppendAuthorityDecision,
) -> RollbackDurableAuditPolicyDecision {
    let media_write_policy_verified = durable_append_authority_decision.media_write_gate_ready
        && durable_append_authority_decision.test_infrastructure_media_write_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_decision_sha256",
        durable_append_authority_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        durable_append_authority_decision.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_media_write_authority_gate_sha256",
        durable_append_authority_decision.source_media_write_authority_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        durable_append_authority_decision.source_target_region_write_readback_hash,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        durable_append_authority_decision.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        durable_append_authority_decision.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        durable_append_authority_decision.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_ready",
        durable_append_authority_decision.append_engine_ready,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        durable_append_authority_decision.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_append_authority_decision.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_append_authority_decision.durable_audit_policy_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyDecision {
        decision_hash: finalize_sha256(hash),
        source_durable_append_authority_decision_hash: durable_append_authority_decision
            .decision_hash,
        source_policy_preflight_hash: durable_append_authority_decision
            .source_policy_preflight_hash,
        source_media_write_authority_gate_hash: durable_append_authority_decision
            .source_media_write_authority_gate_hash,
        source_target_region_write_readback_hash: durable_append_authority_decision
            .source_target_region_write_readback_hash,
        target_start_lba: durable_append_authority_decision.target_start_lba,
        target_lba_count: durable_append_authority_decision.target_lba_count,
        target_byte_count: durable_append_authority_decision.target_byte_count,
        append_engine_ready: durable_append_authority_decision.append_engine_ready,
        media_write_policy_verified,
        test_infrastructure_media_write_authority_available: durable_append_authority_decision
            .test_infrastructure_media_write_authority_available,
        durable_append_authority_available: durable_append_authority_decision
            .durable_append_authority_available,
        durable_audit_policy_available: durable_append_authority_decision
            .durable_audit_policy_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_candidate(
    durable_audit_policy_decision: RollbackDurableAuditPolicyDecision,
    append_record: RollbackAppendRecordDryRun,
) -> RollbackDurableAuditPolicyCandidate {
    let durable_audit_policy_candidate_available =
        durable_audit_policy_decision.media_write_policy_verified;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_decision_sha256",
        durable_audit_policy_decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_record_image_sha256",
        append_record.audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        durable_audit_policy_decision.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        durable_audit_policy_decision.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        durable_audit_policy_decision.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        durable_audit_policy_decision.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        durable_audit_policy_decision.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        durable_audit_policy_decision.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_candidate_available",
        durable_audit_policy_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_audit_policy_decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_audit_policy_decision.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyCandidate {
        candidate_hash: finalize_sha256(hash),
        source_durable_audit_policy_decision_hash: durable_audit_policy_decision.decision_hash,
        source_audit_record_image_hash: append_record.audit_record_image_hash,
        source_policy_preflight_hash: durable_audit_policy_decision.source_policy_preflight_hash,
        source_target_region_write_readback_hash: durable_audit_policy_decision
            .source_target_region_write_readback_hash,
        target_start_lba: durable_audit_policy_decision.target_start_lba,
        target_lba_count: durable_audit_policy_decision.target_lba_count,
        target_byte_count: durable_audit_policy_decision.target_byte_count,
        media_write_policy_verified: durable_audit_policy_decision.media_write_policy_verified,
        durable_audit_policy_candidate_available,
        durable_audit_policy_available: durable_audit_policy_decision
            .durable_audit_policy_available,
        durable_append_authority_available: durable_audit_policy_decision
            .durable_append_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_acceptance_gate(
    durable_audit_policy_candidate: RollbackDurableAuditPolicyCandidate,
) -> RollbackDurableAuditPolicyAcceptanceGate {
    let durable_policy_ledger_available = false;
    let write_authority_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_candidate_sha256",
        durable_audit_policy_candidate.candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_decision_sha256",
        durable_audit_policy_candidate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_record_image_sha256",
        durable_audit_policy_candidate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        durable_audit_policy_candidate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        durable_audit_policy_candidate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        durable_audit_policy_candidate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        durable_audit_policy_candidate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        durable_audit_policy_candidate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"candidate_available",
        durable_audit_policy_candidate.durable_audit_policy_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        durable_audit_policy_candidate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_audit_policy_candidate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_audit_policy_candidate.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyAcceptanceGate {
        gate_hash: finalize_sha256(hash),
        source_durable_audit_policy_candidate_hash: durable_audit_policy_candidate.candidate_hash,
        source_durable_audit_policy_decision_hash: durable_audit_policy_candidate
            .source_durable_audit_policy_decision_hash,
        source_audit_record_image_hash: durable_audit_policy_candidate
            .source_audit_record_image_hash,
        source_policy_preflight_hash: durable_audit_policy_candidate.source_policy_preflight_hash,
        source_target_region_write_readback_hash: durable_audit_policy_candidate
            .source_target_region_write_readback_hash,
        target_start_lba: durable_audit_policy_candidate.target_start_lba,
        target_lba_count: durable_audit_policy_candidate.target_lba_count,
        target_byte_count: durable_audit_policy_candidate.target_byte_count,
        candidate_available: durable_audit_policy_candidate
            .durable_audit_policy_candidate_available,
        media_write_policy_verified: durable_audit_policy_candidate.media_write_policy_verified,
        durable_policy_ledger_available,
        write_authority_available,
        durable_audit_policy_available: durable_audit_policy_candidate
            .durable_audit_policy_available,
        durable_append_authority_available: durable_audit_policy_candidate
            .durable_append_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_ledger_candidate(
    durable_audit_policy_acceptance_gate: RollbackDurableAuditPolicyAcceptanceGate,
) -> RollbackDurableAuditPolicyLedgerCandidate {
    let read_only_ledger_candidate_available = durable_audit_policy_acceptance_gate
        .candidate_available
        && durable_audit_policy_acceptance_gate.media_write_policy_verified;
    let durable_policy_ledger_available = false;
    let write_authority_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"access", "read_only");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_acceptance_gate_sha256",
        durable_audit_policy_acceptance_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_candidate_sha256",
        durable_audit_policy_acceptance_gate.source_durable_audit_policy_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_decision_sha256",
        durable_audit_policy_acceptance_gate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_record_image_sha256",
        durable_audit_policy_acceptance_gate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        durable_audit_policy_acceptance_gate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        durable_audit_policy_acceptance_gate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        durable_audit_policy_acceptance_gate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        durable_audit_policy_acceptance_gate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        durable_audit_policy_acceptance_gate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"read_only_ledger_candidate_available",
        read_only_ledger_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"candidate_available",
        durable_audit_policy_acceptance_gate.candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        durable_audit_policy_acceptance_gate.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_audit_policy_acceptance_gate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_audit_policy_acceptance_gate.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyLedgerCandidate {
        ledger_candidate_hash: finalize_sha256(hash),
        source_acceptance_gate_hash: durable_audit_policy_acceptance_gate.gate_hash,
        source_durable_audit_policy_candidate_hash: durable_audit_policy_acceptance_gate
            .source_durable_audit_policy_candidate_hash,
        source_durable_audit_policy_decision_hash: durable_audit_policy_acceptance_gate
            .source_durable_audit_policy_decision_hash,
        source_audit_record_image_hash: durable_audit_policy_acceptance_gate
            .source_audit_record_image_hash,
        source_policy_preflight_hash: durable_audit_policy_acceptance_gate
            .source_policy_preflight_hash,
        source_target_region_write_readback_hash: durable_audit_policy_acceptance_gate
            .source_target_region_write_readback_hash,
        target_start_lba: durable_audit_policy_acceptance_gate.target_start_lba,
        target_lba_count: durable_audit_policy_acceptance_gate.target_lba_count,
        target_byte_count: durable_audit_policy_acceptance_gate.target_byte_count,
        read_only_ledger_candidate_available,
        candidate_available: durable_audit_policy_acceptance_gate.candidate_available,
        media_write_policy_verified: durable_audit_policy_acceptance_gate
            .media_write_policy_verified,
        durable_policy_ledger_available,
        write_authority_available,
        durable_audit_policy_available: durable_audit_policy_acceptance_gate
            .durable_audit_policy_available,
        durable_append_authority_available: durable_audit_policy_acceptance_gate
            .durable_append_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_ledger_aware_acceptance_result(
    ledger_candidate: RollbackDurableAuditPolicyLedgerCandidate,
) -> RollbackDurableAuditPolicyLedgerAwareAcceptanceResult {
    let ledger_evidence_verified = ledger_candidate.read_only_ledger_candidate_available
        && ledger_candidate.candidate_available
        && ledger_candidate.media_write_policy_verified;
    let write_authority_available = false;
    let durable_policy_ledger_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        ledger_candidate.ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_acceptance_gate_sha256",
        ledger_candidate.source_acceptance_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_candidate_sha256",
        ledger_candidate.source_durable_audit_policy_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_audit_policy_decision_sha256",
        ledger_candidate.source_durable_audit_policy_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_record_image_sha256",
        ledger_candidate.source_audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        ledger_candidate.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        ledger_candidate.source_target_region_write_readback_hash,
    );
    hash_line_str(
        &mut hash,
        b"audit_record_schema",
        rollback_storage_layout::AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        ledger_candidate.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        ledger_candidate.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        ledger_candidate.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"read_only_ledger_candidate_available",
        ledger_candidate.read_only_ledger_candidate_available,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        ledger_candidate.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        ledger_candidate.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyLedgerAwareAcceptanceResult {
        result_hash: finalize_sha256(hash),
        source_ledger_candidate_hash: ledger_candidate.ledger_candidate_hash,
        source_acceptance_gate_hash: ledger_candidate.source_acceptance_gate_hash,
        source_durable_audit_policy_candidate_hash: ledger_candidate
            .source_durable_audit_policy_candidate_hash,
        source_durable_audit_policy_decision_hash: ledger_candidate
            .source_durable_audit_policy_decision_hash,
        source_audit_record_image_hash: ledger_candidate.source_audit_record_image_hash,
        source_policy_preflight_hash: ledger_candidate.source_policy_preflight_hash,
        source_target_region_write_readback_hash: ledger_candidate
            .source_target_region_write_readback_hash,
        target_start_lba: ledger_candidate.target_start_lba,
        target_lba_count: ledger_candidate.target_lba_count,
        target_byte_count: ledger_candidate.target_byte_count,
        read_only_ledger_candidate_available: ledger_candidate.read_only_ledger_candidate_available,
        ledger_evidence_verified,
        write_authority_available,
        durable_policy_ledger_available,
        durable_audit_policy_available: ledger_candidate.durable_audit_policy_available,
        durable_append_authority_available: ledger_candidate.durable_append_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_write_authority_availability(
    ledger_aware_result: RollbackDurableAuditPolicyLedgerAwareAcceptanceResult,
    ledger_candidate: RollbackDurableAuditPolicyLedgerCandidate,
    target_region_media_write_policy_preflight: TargetRegionMediaWritePolicyPreflight,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurableAuditPolicyWriteAuthorityAvailability {
    let media_write_policy_verified = target_region_media_write_policy_preflight
        .source_contract_target_range_ready
        && target_region_media_write_policy_preflight.owner_ids_verified
        && target_region_media_write_policy_preflight.target_ids_verified
        && target_region_media_write_policy_preflight.target_span_verified
        && target_region_media_write_policy_preflight.schema_ids_verified;
    let target_region_write_readback_verified = target_region_write.label_found
        && target_region_write.target_range_ready
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let target_span_verified = ledger_aware_result.target_start_lba
        == ledger_candidate.target_start_lba
        && ledger_aware_result.target_lba_count == ledger_candidate.target_lba_count
        && ledger_aware_result.target_byte_count == ledger_candidate.target_byte_count
        && ledger_aware_result.target_start_lba
            == target_region_media_write_policy_preflight.target_region_start_lba
        && ledger_aware_result.target_lba_count
            == target_region_media_write_policy_preflight.target_region_lba_count
        && ledger_aware_result.target_byte_count
            == target_region_media_write_policy_preflight.target_byte_count
        && ledger_aware_result.target_start_lba == target_region_write.target_start_lba
        && ledger_aware_result.target_lba_count == target_region_write.target_lba_count
        && ledger_aware_result.target_byte_count == target_region_write.target_byte_count;
    let audit_rollback_target_ids_verified = target_region_media_write_policy_preflight
        .target_ids_verified
        && target_region_media_write_policy_preflight.schema_ids_verified;
    let write_authority_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        ledger_aware_result.result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        ledger_candidate.ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        target_region_media_write_policy_preflight.preflight_hash,
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
        ledger_aware_result.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        ledger_aware_result.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        ledger_aware_result.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        ledger_aware_result.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        target_region_write.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        ledger_aware_result.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        ledger_aware_result.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        ledger_aware_result.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyWriteAuthorityAvailability {
        availability_hash: finalize_sha256(hash),
        source_ledger_aware_acceptance_result_hash: ledger_aware_result.result_hash,
        source_ledger_candidate_hash: ledger_candidate.ledger_candidate_hash,
        source_policy_preflight_hash: target_region_media_write_policy_preflight.preflight_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        target_start_lba: ledger_aware_result.target_start_lba,
        target_lba_count: ledger_aware_result.target_lba_count,
        target_byte_count: ledger_aware_result.target_byte_count,
        ledger_evidence_verified: ledger_aware_result.ledger_evidence_verified,
        media_write_policy_verified,
        target_region_write_readback_verified,
        target_span_verified,
        audit_rollback_target_ids_verified,
        write_authority_available,
        durable_policy_ledger_available: ledger_aware_result.durable_policy_ledger_available,
        durable_audit_policy_available: ledger_aware_result.durable_audit_policy_available,
        durable_append_authority_available: ledger_aware_result.durable_append_authority_available,
        test_infrastructure_media_write_authority_available: target_region_write
            .test_infrastructure_media_write_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_policy_ledger_availability(
    write_authority_availability: RollbackDurableAuditPolicyWriteAuthorityAvailability,
) -> RollbackDurablePolicyLedgerAvailability {
    let write_authority_evidence_verified = write_authority_availability.ledger_evidence_verified
        && write_authority_availability.media_write_policy_verified
        && write_authority_availability.target_region_write_readback_verified
        && write_authority_availability.target_span_verified
        && write_authority_availability.audit_rollback_target_ids_verified;
    let durable_policy_ledger_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        write_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        write_authority_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        write_authority_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        write_authority_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        write_authority_availability.source_target_region_write_readback_hash,
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
        write_authority_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        write_authority_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        write_authority_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        write_authority_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        write_authority_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        write_authority_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        write_authority_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        write_authority_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        write_authority_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        write_authority_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        write_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurablePolicyLedgerAvailability {
        availability_hash: finalize_sha256(hash),
        source_write_authority_availability_hash: write_authority_availability.availability_hash,
        source_ledger_aware_acceptance_result_hash: write_authority_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: write_authority_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: write_authority_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: write_authority_availability
            .source_target_region_write_readback_hash,
        target_start_lba: write_authority_availability.target_start_lba,
        target_lba_count: write_authority_availability.target_lba_count,
        target_byte_count: write_authority_availability.target_byte_count,
        write_authority_evidence_verified,
        ledger_evidence_verified: write_authority_availability.ledger_evidence_verified,
        media_write_policy_verified: write_authority_availability.media_write_policy_verified,
        target_region_write_readback_verified: write_authority_availability
            .target_region_write_readback_verified,
        target_span_verified: write_authority_availability.target_span_verified,
        audit_rollback_target_ids_verified: write_authority_availability
            .audit_rollback_target_ids_verified,
        write_authority_available: write_authority_availability.write_authority_available,
        durable_policy_ledger_available,
        durable_audit_policy_available: write_authority_availability.durable_audit_policy_available,
        durable_append_authority_available: write_authority_availability
            .durable_append_authority_available,
        test_infrastructure_media_write_authority_available: write_authority_availability
            .test_infrastructure_media_write_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_policy_ledger_availability_dry_run(
    policy_ledger_availability: RollbackDurablePolicyLedgerAvailability,
    write_authority_availability: RollbackDurableAuditPolicyWriteAuthorityAvailability,
    authority_denial_gate: RollbackTransactionAppendAuthorityDenialGate,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurablePolicyLedgerAvailabilityDryRun {
    let target_span_verified = policy_ledger_availability.target_span_verified
        && write_authority_availability.target_span_verified
        && authority_denial_gate.target_span_verified
        && target_region_write.target_range_ready
        && policy_ledger_availability.target_start_lba
            == write_authority_availability.target_start_lba
        && policy_ledger_availability.target_lba_count
            == write_authority_availability.target_lba_count
        && policy_ledger_availability.target_byte_count
            == write_authority_availability.target_byte_count
        && policy_ledger_availability.target_start_lba == authority_denial_gate.target_start_lba
        && policy_ledger_availability.target_lba_count == authority_denial_gate.target_lba_count
        && policy_ledger_availability.target_byte_count == authority_denial_gate.target_byte_count
        && policy_ledger_availability.target_start_lba == target_region_write.target_start_lba
        && policy_ledger_availability.target_lba_count == target_region_write.target_lba_count
        && policy_ledger_availability.target_byte_count == target_region_write.target_byte_count;
    let policy_ledger_availability_evidence_verified = policy_ledger_availability
        .write_authority_evidence_verified
        && policy_ledger_availability.ledger_evidence_verified
        && policy_ledger_availability.media_write_policy_verified
        && policy_ledger_availability.target_region_write_readback_verified
        && !policy_ledger_availability.durable_policy_ledger_available;
    let write_authority_evidence_verified = policy_ledger_availability
        .source_write_authority_availability_hash
        == write_authority_availability.availability_hash
        && write_authority_availability.ledger_evidence_verified
        && write_authority_availability.media_write_policy_verified
        && !write_authority_availability.write_authority_available;
    let ledger_evidence_verified = policy_ledger_availability.ledger_evidence_verified
        && write_authority_availability.ledger_evidence_verified
        && policy_ledger_availability.source_ledger_aware_acceptance_result_hash
            == write_authority_availability.source_ledger_aware_acceptance_result_hash
        && policy_ledger_availability.source_ledger_candidate_hash
            == write_authority_availability.source_ledger_candidate_hash;
    let media_write_policy_verified = policy_ledger_availability.media_write_policy_verified
        && write_authority_availability.media_write_policy_verified
        && policy_ledger_availability.source_policy_preflight_hash
            == target_region_write.source_policy_preflight_hash;
    let target_region_write_readback_verified = policy_ledger_availability
        .target_region_write_readback_verified
        && write_authority_availability.target_region_write_readback_verified
        && policy_ledger_availability.source_target_region_write_readback_hash
            == target_region_write.dry_run_hash
        && target_region_write.readback_matches_planned_image
        && target_region_write.write_attempted
        && target_region_write.write_completed
        && target_region_write.readback_completed;
    let transaction_append_denial_gate_verified = authority_denial_gate
        .missing_transaction_append_authority
        && !authority_denial_gate.transaction_append_available
        && authority_denial_gate.source_target_region_write_readback_hash
            == policy_ledger_availability.source_target_region_write_readback_hash
        && authority_denial_gate.source_policy_preflight_hash
            == policy_ledger_availability.source_policy_preflight_hash
        && authority_denial_gate.media_write_policy_verified
        && authority_denial_gate.target_region_write_readback_verified;
    let audit_rollback_target_ids_verified = policy_ledger_availability
        .audit_rollback_target_ids_verified
        && write_authority_availability.audit_rollback_target_ids_verified
        && authority_denial_gate.audit_rollback_target_ids_verified;
    let test_infrastructure_media_write_authority_available = policy_ledger_availability
        .test_infrastructure_media_write_authority_available
        && write_authority_availability.test_infrastructure_media_write_authority_available
        && authority_denial_gate.test_infrastructure_media_write_authority_available
        && target_region_write.test_infrastructure_media_write_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_sha256",
        policy_ledger_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        policy_ledger_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        policy_ledger_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        policy_ledger_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        policy_ledger_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        policy_ledger_availability.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_authority_denial_gate_sha256",
        authority_denial_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        authority_denial_gate.source_transaction_append_availability_decision_hash,
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
        policy_ledger_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        policy_ledger_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        policy_ledger_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_availability_evidence_verified",
        policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_denial_gate_verified",
        transaction_append_denial_gate_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        policy_ledger_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        policy_ledger_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        policy_ledger_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        policy_ledger_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        authority_denial_gate.transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"installs_rollback_state", false);
    RollbackDurablePolicyLedgerAvailabilityDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_policy_ledger_availability_hash: policy_ledger_availability.availability_hash,
        source_write_authority_availability_hash: policy_ledger_availability
            .source_write_authority_availability_hash,
        source_ledger_aware_acceptance_result_hash: policy_ledger_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: policy_ledger_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: policy_ledger_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: policy_ledger_availability
            .source_target_region_write_readback_hash,
        source_authority_denial_gate_hash: authority_denial_gate.gate_hash,
        source_transaction_append_availability_decision_hash: authority_denial_gate
            .source_transaction_append_availability_decision_hash,
        target_start_lba: policy_ledger_availability.target_start_lba,
        target_lba_count: policy_ledger_availability.target_lba_count,
        target_byte_count: policy_ledger_availability.target_byte_count,
        policy_ledger_availability_evidence_verified,
        write_authority_evidence_verified,
        ledger_evidence_verified,
        media_write_policy_verified,
        target_region_write_readback_verified,
        transaction_append_denial_gate_verified,
        target_span_verified,
        audit_rollback_target_ids_verified,
        test_infrastructure_media_write_authority_available,
        write_authority_available: policy_ledger_availability.write_authority_available,
        durable_policy_ledger_available: policy_ledger_availability.durable_policy_ledger_available,
        durable_audit_policy_available: policy_ledger_availability.durable_audit_policy_available,
        durable_append_authority_available: policy_ledger_availability
            .durable_append_authority_available,
        transaction_append_available: authority_denial_gate.transaction_append_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_availability(
    policy_ledger_availability: RollbackDurablePolicyLedgerAvailability,
) -> RollbackDurableAuditPolicyAvailability {
    let policy_ledger_availability_evidence_verified = policy_ledger_availability
        .write_authority_evidence_verified
        && policy_ledger_availability.ledger_evidence_verified
        && policy_ledger_availability.media_write_policy_verified
        && policy_ledger_availability.target_region_write_readback_verified
        && policy_ledger_availability.target_span_verified
        && policy_ledger_availability.audit_rollback_target_ids_verified;
    let durable_audit_policy_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_sha256",
        policy_ledger_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        policy_ledger_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        policy_ledger_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        policy_ledger_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        policy_ledger_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        policy_ledger_availability.source_target_region_write_readback_hash,
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
        policy_ledger_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        policy_ledger_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        policy_ledger_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_availability_evidence_verified",
        policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        policy_ledger_availability.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        policy_ledger_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        policy_ledger_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        policy_ledger_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        policy_ledger_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        policy_ledger_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        policy_ledger_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        policy_ledger_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        policy_ledger_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        policy_ledger_availability.durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAuditPolicyAvailability {
        availability_hash: finalize_sha256(hash),
        source_policy_ledger_availability_hash: policy_ledger_availability.availability_hash,
        source_write_authority_availability_hash: policy_ledger_availability
            .source_write_authority_availability_hash,
        source_ledger_aware_acceptance_result_hash: policy_ledger_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: policy_ledger_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: policy_ledger_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: policy_ledger_availability
            .source_target_region_write_readback_hash,
        target_start_lba: policy_ledger_availability.target_start_lba,
        target_lba_count: policy_ledger_availability.target_lba_count,
        target_byte_count: policy_ledger_availability.target_byte_count,
        policy_ledger_availability_evidence_verified,
        write_authority_evidence_verified: policy_ledger_availability
            .write_authority_evidence_verified,
        ledger_evidence_verified: policy_ledger_availability.ledger_evidence_verified,
        media_write_policy_verified: policy_ledger_availability.media_write_policy_verified,
        target_region_write_readback_verified: policy_ledger_availability
            .target_region_write_readback_verified,
        target_span_verified: policy_ledger_availability.target_span_verified,
        audit_rollback_target_ids_verified: policy_ledger_availability
            .audit_rollback_target_ids_verified,
        write_authority_available: policy_ledger_availability.write_authority_available,
        durable_policy_ledger_available: policy_ledger_availability.durable_policy_ledger_available,
        durable_audit_policy_available,
        durable_append_authority_available: policy_ledger_availability
            .durable_append_authority_available,
        test_infrastructure_media_write_authority_available: policy_ledger_availability
            .test_infrastructure_media_write_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_audit_policy_availability_dry_run(
    audit_policy_availability: RollbackDurableAuditPolicyAvailability,
    policy_ledger_dry_run: RollbackDurablePolicyLedgerAvailabilityDryRun,
    authority_denial_gate: RollbackTransactionAppendAuthorityDenialGate,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurableAuditPolicyAvailabilityDryRun {
    let target_span_verified = audit_policy_availability.target_span_verified
        && policy_ledger_dry_run.target_span_verified
        && authority_denial_gate.target_span_verified
        && target_region_write.target_range_ready
        && audit_policy_availability.target_start_lba == policy_ledger_dry_run.target_start_lba
        && audit_policy_availability.target_lba_count == policy_ledger_dry_run.target_lba_count
        && audit_policy_availability.target_byte_count == policy_ledger_dry_run.target_byte_count
        && audit_policy_availability.target_start_lba == authority_denial_gate.target_start_lba
        && audit_policy_availability.target_lba_count == authority_denial_gate.target_lba_count
        && audit_policy_availability.target_byte_count == authority_denial_gate.target_byte_count
        && audit_policy_availability.target_start_lba == target_region_write.target_start_lba
        && audit_policy_availability.target_lba_count == target_region_write.target_lba_count
        && audit_policy_availability.target_byte_count == target_region_write.target_byte_count;
    let audit_policy_availability_evidence_verified = audit_policy_availability
        .policy_ledger_availability_evidence_verified
        && audit_policy_availability.write_authority_evidence_verified
        && audit_policy_availability.ledger_evidence_verified
        && audit_policy_availability.media_write_policy_verified
        && audit_policy_availability.target_region_write_readback_verified
        && audit_policy_availability.audit_rollback_target_ids_verified
        && !audit_policy_availability.durable_audit_policy_available;
    let policy_ledger_dry_run_evidence_verified = policy_ledger_dry_run
        .policy_ledger_availability_evidence_verified
        && policy_ledger_dry_run.transaction_append_denial_gate_verified
        && !policy_ledger_dry_run.durable_policy_ledger_available
        && !policy_ledger_dry_run.durable_audit_policy_available;
    let policy_ledger_availability_evidence_verified = audit_policy_availability
        .source_policy_ledger_availability_hash
        == policy_ledger_dry_run.source_policy_ledger_availability_hash
        && audit_policy_availability.policy_ledger_availability_evidence_verified
        && policy_ledger_dry_run.policy_ledger_availability_evidence_verified;
    let write_authority_evidence_verified = audit_policy_availability
        .source_write_authority_availability_hash
        == policy_ledger_dry_run.source_write_authority_availability_hash
        && audit_policy_availability.write_authority_evidence_verified
        && policy_ledger_dry_run.write_authority_evidence_verified
        && !audit_policy_availability.write_authority_available;
    let ledger_evidence_verified = audit_policy_availability.ledger_evidence_verified
        && policy_ledger_dry_run.ledger_evidence_verified
        && audit_policy_availability.source_ledger_aware_acceptance_result_hash
            == policy_ledger_dry_run.source_ledger_aware_acceptance_result_hash
        && audit_policy_availability.source_ledger_candidate_hash
            == policy_ledger_dry_run.source_ledger_candidate_hash;
    let media_write_policy_verified = audit_policy_availability.media_write_policy_verified
        && policy_ledger_dry_run.media_write_policy_verified
        && audit_policy_availability.source_policy_preflight_hash
            == policy_ledger_dry_run.source_policy_preflight_hash
        && audit_policy_availability.source_policy_preflight_hash
            == target_region_write.source_policy_preflight_hash;
    let target_region_write_readback_verified = audit_policy_availability
        .target_region_write_readback_verified
        && policy_ledger_dry_run.target_region_write_readback_verified
        && audit_policy_availability.source_target_region_write_readback_hash
            == policy_ledger_dry_run.source_target_region_write_readback_hash
        && audit_policy_availability.source_target_region_write_readback_hash
            == target_region_write.dry_run_hash
        && target_region_write.readback_matches_planned_image
        && target_region_write.write_attempted
        && target_region_write.write_completed
        && target_region_write.readback_completed;
    let transaction_append_denial_gate_verified =
        policy_ledger_dry_run.source_authority_denial_gate_hash == authority_denial_gate.gate_hash
            && policy_ledger_dry_run.source_transaction_append_availability_decision_hash
                == authority_denial_gate.source_transaction_append_availability_decision_hash
            && policy_ledger_dry_run.transaction_append_denial_gate_verified
            && authority_denial_gate.missing_transaction_append_authority
            && !authority_denial_gate.transaction_append_available;
    let audit_rollback_target_ids_verified = audit_policy_availability
        .audit_rollback_target_ids_verified
        && policy_ledger_dry_run.audit_rollback_target_ids_verified
        && authority_denial_gate.audit_rollback_target_ids_verified;
    let test_infrastructure_media_write_authority_available = audit_policy_availability
        .test_infrastructure_media_write_authority_available
        && policy_ledger_dry_run.test_infrastructure_media_write_authority_available
        && authority_denial_gate.test_infrastructure_media_write_authority_available
        && target_region_write.test_infrastructure_media_write_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        audit_policy_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_dry_run_sha256",
        policy_ledger_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_sha256",
        audit_policy_availability.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        audit_policy_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        audit_policy_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        audit_policy_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        audit_policy_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        audit_policy_availability.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_authority_denial_gate_sha256",
        authority_denial_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        authority_denial_gate.source_transaction_append_availability_decision_hash,
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
        audit_policy_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        audit_policy_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        audit_policy_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_availability_evidence_verified",
        audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_dry_run_evidence_verified",
        policy_ledger_dry_run_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_availability_evidence_verified",
        policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_denial_gate_verified",
        transaction_append_denial_gate_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        audit_policy_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        audit_policy_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        audit_policy_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        audit_policy_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        authority_denial_gate.transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"installs_rollback_state", false);
    RollbackDurableAuditPolicyAvailabilityDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_audit_policy_availability_hash: audit_policy_availability.availability_hash,
        source_policy_ledger_availability_dry_run_hash: policy_ledger_dry_run.dry_run_hash,
        source_policy_ledger_availability_hash: audit_policy_availability
            .source_policy_ledger_availability_hash,
        source_write_authority_availability_hash: audit_policy_availability
            .source_write_authority_availability_hash,
        source_ledger_aware_acceptance_result_hash: audit_policy_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: audit_policy_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: audit_policy_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: audit_policy_availability
            .source_target_region_write_readback_hash,
        source_authority_denial_gate_hash: authority_denial_gate.gate_hash,
        source_transaction_append_availability_decision_hash: authority_denial_gate
            .source_transaction_append_availability_decision_hash,
        target_start_lba: audit_policy_availability.target_start_lba,
        target_lba_count: audit_policy_availability.target_lba_count,
        target_byte_count: audit_policy_availability.target_byte_count,
        audit_policy_availability_evidence_verified,
        policy_ledger_dry_run_evidence_verified,
        policy_ledger_availability_evidence_verified,
        write_authority_evidence_verified,
        ledger_evidence_verified,
        media_write_policy_verified,
        target_region_write_readback_verified,
        transaction_append_denial_gate_verified,
        target_span_verified,
        audit_rollback_target_ids_verified,
        test_infrastructure_media_write_authority_available,
        write_authority_available: audit_policy_availability.write_authority_available,
        durable_policy_ledger_available: audit_policy_availability.durable_policy_ledger_available,
        durable_audit_policy_available: audit_policy_availability.durable_audit_policy_available,
        durable_append_authority_available: audit_policy_availability
            .durable_append_authority_available,
        transaction_append_available: authority_denial_gate.transaction_append_available,
    }
}

pub(crate) fn hello_rollback_durable_append_authority_availability(
    audit_policy_availability: RollbackDurableAuditPolicyAvailability,
) -> RollbackDurableAppendAuthorityAvailability {
    let audit_policy_availability_evidence_verified = audit_policy_availability
        .policy_ledger_availability_evidence_verified
        && audit_policy_availability.write_authority_evidence_verified
        && audit_policy_availability.ledger_evidence_verified
        && audit_policy_availability.media_write_policy_verified
        && audit_policy_availability.target_region_write_readback_verified
        && audit_policy_availability.target_span_verified
        && audit_policy_availability.audit_rollback_target_ids_verified;
    let durable_append_authority_available = false;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        audit_policy_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_sha256",
        audit_policy_availability.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        audit_policy_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        audit_policy_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        audit_policy_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        audit_policy_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        audit_policy_availability.source_target_region_write_readback_hash,
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
        audit_policy_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        audit_policy_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        audit_policy_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_availability_evidence_verified",
        audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_availability_evidence_verified",
        audit_policy_availability.policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        audit_policy_availability.write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        audit_policy_availability.ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        audit_policy_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        audit_policy_availability.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        audit_policy_availability.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        audit_policy_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        audit_policy_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        audit_policy_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        audit_policy_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        audit_policy_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_append_authority_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackDurableAppendAuthorityAvailability {
        availability_hash: finalize_sha256(hash),
        source_audit_policy_availability_hash: audit_policy_availability.availability_hash,
        source_policy_ledger_availability_hash: audit_policy_availability
            .source_policy_ledger_availability_hash,
        source_write_authority_availability_hash: audit_policy_availability
            .source_write_authority_availability_hash,
        source_ledger_aware_acceptance_result_hash: audit_policy_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: audit_policy_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: audit_policy_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: audit_policy_availability
            .source_target_region_write_readback_hash,
        target_start_lba: audit_policy_availability.target_start_lba,
        target_lba_count: audit_policy_availability.target_lba_count,
        target_byte_count: audit_policy_availability.target_byte_count,
        audit_policy_availability_evidence_verified,
        policy_ledger_availability_evidence_verified: audit_policy_availability
            .policy_ledger_availability_evidence_verified,
        write_authority_evidence_verified: audit_policy_availability
            .write_authority_evidence_verified,
        ledger_evidence_verified: audit_policy_availability.ledger_evidence_verified,
        media_write_policy_verified: audit_policy_availability.media_write_policy_verified,
        target_region_write_readback_verified: audit_policy_availability
            .target_region_write_readback_verified,
        target_span_verified: audit_policy_availability.target_span_verified,
        audit_rollback_target_ids_verified: audit_policy_availability
            .audit_rollback_target_ids_verified,
        write_authority_available: audit_policy_availability.write_authority_available,
        durable_policy_ledger_available: audit_policy_availability.durable_policy_ledger_available,
        durable_audit_policy_available: audit_policy_availability.durable_audit_policy_available,
        durable_append_authority_available,
        test_infrastructure_media_write_authority_available: audit_policy_availability
            .test_infrastructure_media_write_authority_available,
    }
}

pub(crate) fn hello_rollback_durable_append_authority_availability_dry_run(
    append_authority_availability: RollbackDurableAppendAuthorityAvailability,
    audit_policy_dry_run: RollbackDurableAuditPolicyAvailabilityDryRun,
    authority_denial_gate: RollbackTransactionAppendAuthorityDenialGate,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackDurableAppendAuthorityAvailabilityDryRun {
    let target_span_verified = append_authority_availability.target_span_verified
        && audit_policy_dry_run.target_span_verified
        && authority_denial_gate.target_span_verified
        && target_region_write.target_range_ready
        && append_authority_availability.target_start_lba == audit_policy_dry_run.target_start_lba
        && append_authority_availability.target_lba_count == audit_policy_dry_run.target_lba_count
        && append_authority_availability.target_byte_count
            == audit_policy_dry_run.target_byte_count
        && append_authority_availability.target_start_lba == authority_denial_gate.target_start_lba
        && append_authority_availability.target_lba_count == authority_denial_gate.target_lba_count
        && append_authority_availability.target_byte_count
            == authority_denial_gate.target_byte_count
        && append_authority_availability.target_start_lba == target_region_write.target_start_lba
        && append_authority_availability.target_lba_count == target_region_write.target_lba_count
        && append_authority_availability.target_byte_count == target_region_write.target_byte_count;
    let append_authority_availability_evidence_verified = append_authority_availability
        .audit_policy_availability_evidence_verified
        && append_authority_availability.policy_ledger_availability_evidence_verified
        && append_authority_availability.write_authority_evidence_verified
        && append_authority_availability.ledger_evidence_verified
        && append_authority_availability.media_write_policy_verified
        && append_authority_availability.target_region_write_readback_verified
        && append_authority_availability.audit_rollback_target_ids_verified
        && !append_authority_availability.durable_append_authority_available;
    let audit_policy_dry_run_evidence_verified = audit_policy_dry_run
        .audit_policy_availability_evidence_verified
        && audit_policy_dry_run.policy_ledger_dry_run_evidence_verified
        && audit_policy_dry_run.transaction_append_denial_gate_verified
        && !audit_policy_dry_run.durable_audit_policy_available
        && !audit_policy_dry_run.durable_append_authority_available;
    let audit_policy_availability_evidence_verified = append_authority_availability
        .source_audit_policy_availability_hash
        == audit_policy_dry_run.source_audit_policy_availability_hash
        && append_authority_availability.audit_policy_availability_evidence_verified
        && audit_policy_dry_run.audit_policy_availability_evidence_verified;
    let policy_ledger_dry_run_evidence_verified =
        audit_policy_dry_run.policy_ledger_dry_run_evidence_verified;
    let policy_ledger_availability_evidence_verified = append_authority_availability
        .source_policy_ledger_availability_hash
        == audit_policy_dry_run.source_policy_ledger_availability_hash
        && append_authority_availability.policy_ledger_availability_evidence_verified
        && audit_policy_dry_run.policy_ledger_availability_evidence_verified;
    let write_authority_evidence_verified = append_authority_availability
        .source_write_authority_availability_hash
        == audit_policy_dry_run.source_write_authority_availability_hash
        && append_authority_availability.write_authority_evidence_verified
        && audit_policy_dry_run.write_authority_evidence_verified
        && !append_authority_availability.write_authority_available;
    let ledger_evidence_verified = append_authority_availability.ledger_evidence_verified
        && audit_policy_dry_run.ledger_evidence_verified
        && append_authority_availability.source_ledger_aware_acceptance_result_hash
            == audit_policy_dry_run.source_ledger_aware_acceptance_result_hash
        && append_authority_availability.source_ledger_candidate_hash
            == audit_policy_dry_run.source_ledger_candidate_hash;
    let media_write_policy_verified = append_authority_availability.media_write_policy_verified
        && audit_policy_dry_run.media_write_policy_verified
        && append_authority_availability.source_policy_preflight_hash
            == audit_policy_dry_run.source_policy_preflight_hash
        && append_authority_availability.source_policy_preflight_hash
            == target_region_write.source_policy_preflight_hash;
    let target_region_write_readback_verified = append_authority_availability
        .target_region_write_readback_verified
        && audit_policy_dry_run.target_region_write_readback_verified
        && append_authority_availability.source_target_region_write_readback_hash
            == audit_policy_dry_run.source_target_region_write_readback_hash
        && append_authority_availability.source_target_region_write_readback_hash
            == target_region_write.dry_run_hash
        && target_region_write.readback_matches_planned_image
        && target_region_write.write_attempted
        && target_region_write.write_completed
        && target_region_write.readback_completed;
    let transaction_append_denial_gate_verified =
        audit_policy_dry_run.source_authority_denial_gate_hash == authority_denial_gate.gate_hash
            && audit_policy_dry_run.source_transaction_append_availability_decision_hash
                == authority_denial_gate.source_transaction_append_availability_decision_hash
            && authority_denial_gate.source_durable_append_authority_availability_hash
                == append_authority_availability.availability_hash
            && authority_denial_gate.source_audit_policy_availability_hash
                == append_authority_availability.source_audit_policy_availability_hash
            && authority_denial_gate.missing_transaction_append_authority
            && !authority_denial_gate.transaction_append_available;
    let audit_rollback_target_ids_verified = append_authority_availability
        .audit_rollback_target_ids_verified
        && audit_policy_dry_run.audit_rollback_target_ids_verified
        && authority_denial_gate.audit_rollback_target_ids_verified;
    let test_infrastructure_media_write_authority_available = append_authority_availability
        .test_infrastructure_media_write_authority_available
        && audit_policy_dry_run.test_infrastructure_media_write_authority_available
        && authority_denial_gate.test_infrastructure_media_write_authority_available
        && target_region_write.test_infrastructure_media_write_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_authority_availability_sha256",
        append_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_dry_run_sha256",
        audit_policy_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        append_authority_availability.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_dry_run_sha256",
        audit_policy_dry_run.source_policy_ledger_availability_dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_ledger_availability_sha256",
        append_authority_availability.source_policy_ledger_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        append_authority_availability.source_write_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_aware_acceptance_result_sha256",
        append_authority_availability.source_ledger_aware_acceptance_result_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_ledger_candidate_sha256",
        append_authority_availability.source_ledger_candidate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        append_authority_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        append_authority_availability.source_target_region_write_readback_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_authority_denial_gate_sha256",
        authority_denial_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        authority_denial_gate.source_transaction_append_availability_decision_hash,
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
        append_authority_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        append_authority_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        append_authority_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"append_authority_availability_evidence_verified",
        append_authority_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_dry_run_evidence_verified",
        audit_policy_dry_run_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_availability_evidence_verified",
        audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_dry_run_evidence_verified",
        policy_ledger_dry_run_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"policy_ledger_availability_evidence_verified",
        policy_ledger_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"ledger_evidence_verified",
        ledger_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_denial_gate_verified",
        transaction_append_denial_gate_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        append_authority_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        append_authority_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        append_authority_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        append_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        authority_denial_gate.transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"installs_rollback_state", false);
    RollbackDurableAppendAuthorityAvailabilityDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_append_authority_availability_hash: append_authority_availability.availability_hash,
        source_audit_policy_availability_dry_run_hash: audit_policy_dry_run.dry_run_hash,
        source_audit_policy_availability_hash: append_authority_availability
            .source_audit_policy_availability_hash,
        source_policy_ledger_availability_dry_run_hash: audit_policy_dry_run
            .source_policy_ledger_availability_dry_run_hash,
        source_policy_ledger_availability_hash: append_authority_availability
            .source_policy_ledger_availability_hash,
        source_write_authority_availability_hash: append_authority_availability
            .source_write_authority_availability_hash,
        source_ledger_aware_acceptance_result_hash: append_authority_availability
            .source_ledger_aware_acceptance_result_hash,
        source_ledger_candidate_hash: append_authority_availability.source_ledger_candidate_hash,
        source_policy_preflight_hash: append_authority_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: append_authority_availability
            .source_target_region_write_readback_hash,
        source_authority_denial_gate_hash: authority_denial_gate.gate_hash,
        source_transaction_append_availability_decision_hash: authority_denial_gate
            .source_transaction_append_availability_decision_hash,
        target_start_lba: append_authority_availability.target_start_lba,
        target_lba_count: append_authority_availability.target_lba_count,
        target_byte_count: append_authority_availability.target_byte_count,
        append_authority_availability_evidence_verified,
        audit_policy_dry_run_evidence_verified,
        audit_policy_availability_evidence_verified,
        policy_ledger_dry_run_evidence_verified,
        policy_ledger_availability_evidence_verified,
        write_authority_evidence_verified,
        ledger_evidence_verified,
        media_write_policy_verified,
        target_region_write_readback_verified,
        transaction_append_denial_gate_verified,
        target_span_verified,
        audit_rollback_target_ids_verified,
        test_infrastructure_media_write_authority_available,
        write_authority_available: append_authority_availability.write_authority_available,
        durable_policy_ledger_available: append_authority_availability
            .durable_policy_ledger_available,
        durable_audit_policy_available: append_authority_availability
            .durable_audit_policy_available,
        durable_append_authority_available: append_authority_availability
            .durable_append_authority_available,
        transaction_append_available: authority_denial_gate.transaction_append_available,
    }
}

pub(crate) fn hello_rollback_transaction_append_availability_decision(
    append_authority_availability: RollbackDurableAppendAuthorityAvailability,
    append_engine_readiness: RollbackAppendEngineReadinessDecision,
    writer_policy: RollbackDurableWriterPolicyPreflight,
) -> RollbackTransactionAppendAvailabilityDecision {
    let target_span_verified = append_authority_availability.target_span_verified
        && append_authority_availability.target_start_lba
            == append_engine_readiness.target_start_lba
        && append_authority_availability.target_lba_count
            == append_engine_readiness.target_lba_count
        && append_authority_availability.target_byte_count
            == append_engine_readiness.target_byte_count
        && append_authority_availability.target_start_lba == writer_policy.target_start_lba
        && append_authority_availability.target_lba_count == writer_policy.target_lba_count
        && append_authority_availability.target_byte_count == writer_policy.target_byte_count;
    let writer_policy_ready = writer_policy.target_range_ready
        && writer_policy.durable_audit_writer_available
        && writer_policy.rollback_store_writer_available
        && writer_policy.transaction_append_writer_available;
    let durable_append_authority_availability_evidence_verified = append_authority_availability
        .audit_policy_availability_evidence_verified
        && append_authority_availability.policy_ledger_availability_evidence_verified
        && append_authority_availability.write_authority_evidence_verified
        && append_authority_availability.ledger_evidence_verified
        && append_authority_availability.media_write_policy_verified
        && append_authority_availability.target_region_write_readback_verified
        && target_span_verified
        && append_authority_availability.audit_rollback_target_ids_verified;
    let transaction_append_available = append_engine_readiness.ready
        && writer_policy_ready
        && append_authority_availability.durable_append_authority_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_availability_sha256",
        append_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        append_authority_availability.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_engine_readiness_decision_sha256",
        append_engine_readiness.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_writer_policy_preflight_sha256",
        writer_policy.preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        append_authority_availability.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        append_authority_availability.source_target_region_write_readback_hash,
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
        append_authority_availability.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        append_authority_availability.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        append_authority_availability.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_evidence_verified",
        durable_append_authority_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_availability_evidence_verified",
        append_authority_availability.audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_ready",
        append_engine_readiness.ready,
    );
    hash_line_bool(&mut hash, b"writer_policy_ready", writer_policy_ready);
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        append_authority_availability.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        append_authority_availability.target_region_write_readback_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        append_authority_availability.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        append_authority_availability.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        append_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        append_authority_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackTransactionAppendAvailabilityDecision {
        decision_hash: finalize_sha256(hash),
        source_durable_append_authority_availability_hash: append_authority_availability
            .availability_hash,
        source_audit_policy_availability_hash: append_authority_availability
            .source_audit_policy_availability_hash,
        source_append_engine_readiness_decision_hash: append_engine_readiness.decision_hash,
        source_writer_policy_preflight_hash: writer_policy.preflight_hash,
        source_policy_preflight_hash: append_authority_availability.source_policy_preflight_hash,
        source_target_region_write_readback_hash: append_authority_availability
            .source_target_region_write_readback_hash,
        target_start_lba: append_authority_availability.target_start_lba,
        target_lba_count: append_authority_availability.target_lba_count,
        target_byte_count: append_authority_availability.target_byte_count,
        durable_append_authority_availability_evidence_verified,
        audit_policy_availability_evidence_verified: append_authority_availability
            .audit_policy_availability_evidence_verified,
        append_engine_ready: append_engine_readiness.ready,
        writer_policy_ready,
        media_write_policy_verified: append_authority_availability.media_write_policy_verified,
        target_region_write_readback_verified: append_authority_availability
            .target_region_write_readback_verified,
        target_span_verified,
        audit_rollback_target_ids_verified: append_authority_availability
            .audit_rollback_target_ids_verified,
        test_infrastructure_media_write_authority_available: append_authority_availability
            .test_infrastructure_media_write_authority_available,
        durable_append_authority_available: append_authority_availability
            .durable_append_authority_available,
        durable_audit_policy_available: append_authority_availability
            .durable_audit_policy_available,
        transaction_append_available,
    }
}

pub(crate) fn hello_rollback_transaction_append_authority_denial_gate(
    decision: RollbackTransactionAppendAvailabilityDecision,
) -> RollbackTransactionAppendAuthorityDenialGate {
    let availability_decision_evidence_verified = decision
        .durable_append_authority_availability_evidence_verified
        && decision.audit_policy_availability_evidence_verified
        && decision.append_engine_ready
        && decision.writer_policy_ready
        && decision.media_write_policy_verified
        && decision.target_region_write_readback_verified
        && decision.target_span_verified
        && decision.audit_rollback_target_ids_verified;
    let missing_transaction_append_authority = !decision.transaction_append_available
        && (!decision.durable_append_authority_available
            || !decision.durable_audit_policy_available);
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        decision.decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_availability_sha256",
        decision.source_durable_append_authority_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        decision.source_audit_policy_availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_append_engine_readiness_decision_sha256",
        decision.source_append_engine_readiness_decision_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_writer_policy_preflight_sha256",
        decision.source_writer_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        decision.source_policy_preflight_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        decision.source_target_region_write_readback_hash,
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
    hash_line_u64(&mut hash, b"target_start_lba", decision.target_start_lba);
    hash_line_u64(&mut hash, b"target_lba_count", decision.target_lba_count);
    hash_line_u64(&mut hash, b"target_byte_count", decision.target_byte_count);
    hash_line_bool(
        &mut hash,
        b"availability_decision_evidence_verified",
        availability_decision_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"append_engine_ready",
        decision.append_engine_ready,
    );
    hash_line_bool(
        &mut hash,
        b"writer_policy_ready",
        decision.writer_policy_ready,
    );
    hash_line_bool(
        &mut hash,
        b"media_write_policy_verified",
        decision.media_write_policy_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        decision.target_region_write_readback_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_span_verified",
        decision.target_span_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_rollback_target_ids_verified",
        decision.audit_rollback_target_ids_verified,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        decision.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        decision.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        decision.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        decision.transaction_append_available,
    );
    hash_line_bool(
        &mut hash,
        b"missing_transaction_append_authority",
        missing_transaction_append_authority,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    RollbackTransactionAppendAuthorityDenialGate {
        gate_hash: finalize_sha256(hash),
        source_transaction_append_availability_decision_hash: decision.decision_hash,
        source_durable_append_authority_availability_hash: decision
            .source_durable_append_authority_availability_hash,
        source_audit_policy_availability_hash: decision.source_audit_policy_availability_hash,
        source_append_engine_readiness_decision_hash: decision
            .source_append_engine_readiness_decision_hash,
        source_writer_policy_preflight_hash: decision.source_writer_policy_preflight_hash,
        source_policy_preflight_hash: decision.source_policy_preflight_hash,
        source_target_region_write_readback_hash: decision.source_target_region_write_readback_hash,
        target_start_lba: decision.target_start_lba,
        target_lba_count: decision.target_lba_count,
        target_byte_count: decision.target_byte_count,
        availability_decision_evidence_verified,
        append_engine_ready: decision.append_engine_ready,
        writer_policy_ready: decision.writer_policy_ready,
        media_write_policy_verified: decision.media_write_policy_verified,
        target_region_write_readback_verified: decision.target_region_write_readback_verified,
        target_span_verified: decision.target_span_verified,
        audit_rollback_target_ids_verified: decision.audit_rollback_target_ids_verified,
        test_infrastructure_media_write_authority_available: decision
            .test_infrastructure_media_write_authority_available,
        durable_append_authority_available: decision.durable_append_authority_available,
        durable_audit_policy_available: decision.durable_audit_policy_available,
        transaction_append_available: decision.transaction_append_available,
        missing_transaction_append_authority,
    }
}

pub(crate) fn hello_rollback_transaction_append_dry_run(
    authority_denial_gate: RollbackTransactionAppendAuthorityDenialGate,
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackTransactionAppendDryRun {
    let target_span_verified = authority_denial_gate.target_span_verified
        && append_record.target_start_lba == sector_plan.target_start_lba
        && append_record.target_lba_count == sector_plan.target_lba_count
        && append_record.target_byte_count == sector_plan.target_byte_count
        && sector_plan.target_start_lba == target_region_write.target_start_lba
        && sector_plan.target_lba_count == target_region_write.target_lba_count
        && sector_plan.target_byte_count == target_region_write.target_byte_count
        && authority_denial_gate.target_start_lba == target_region_write.target_start_lba
        && authority_denial_gate.target_lba_count == target_region_write.target_lba_count
        && authority_denial_gate.target_byte_count == target_region_write.target_byte_count;
    let authority_denial_gate_verified = authority_denial_gate
        .availability_decision_evidence_verified
        && authority_denial_gate.missing_transaction_append_authority
        && !authority_denial_gate.transaction_append_available;
    let target_region_write_readback_verified = target_region_write.label_found
        && target_region_write.target_range_ready
        && target_region_write.test_infrastructure_media_write_authority_available
        && target_region_write.write_completed
        && target_region_write.readback_completed
        && target_region_write.readback_matches_planned_image;
    let append_image_ready = target_span_verified
        && sector_plan.target_range_ready
        && append_record.target_range_ready
        && target_region_write_readback_verified
        && target_region_write.planned_sector_image_hash == sector_plan.sector_image_hash
        && target_region_write.readback_sector_image_hash == sector_plan.sector_image_hash;
    let blocked_by_authority_denial_gate = authority_denial_gate_verified
        && authority_denial_gate.missing_transaction_append_authority;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_authority_denial_gate_sha256",
        authority_denial_gate.gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        authority_denial_gate.source_transaction_append_availability_decision_hash,
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
    hash_line_hash(
        &mut hash,
        b"planned_sector_image_sha256",
        target_region_write.planned_sector_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"readback_sector_image_sha256",
        target_region_write.readback_sector_image_hash,
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
    hash_line_bool(
        &mut hash,
        b"authority_denial_gate_verified",
        authority_denial_gate_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(&mut hash, b"append_image_ready", append_image_ready);
    hash_line_bool(
        &mut hash,
        b"blocked_by_authority_denial_gate",
        blocked_by_authority_denial_gate,
    );
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        target_region_write.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        authority_denial_gate.transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"transaction_append_attempted", false);
    RollbackTransactionAppendDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_authority_denial_gate_hash: authority_denial_gate.gate_hash,
        source_transaction_append_availability_decision_hash: authority_denial_gate
            .source_transaction_append_availability_decision_hash,
        source_append_record_hash: append_record.dry_run_hash,
        source_sector_plan_hash: sector_plan.plan_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        planned_sector_image_hash: target_region_write.planned_sector_image_hash,
        readback_sector_image_hash: target_region_write.readback_sector_image_hash,
        target_start_lba: target_region_write.target_start_lba,
        target_lba_count: target_region_write.target_lba_count,
        target_byte_count: target_region_write.target_byte_count,
        authority_denial_gate_verified,
        target_span_verified,
        target_region_write_readback_verified,
        append_image_ready,
        blocked_by_authority_denial_gate,
        test_infrastructure_media_write_authority_available: target_region_write
            .test_infrastructure_media_write_authority_available,
        transaction_append_available: authority_denial_gate.transaction_append_available,
    }
}

pub(crate) fn hello_rollback_durable_policy_write_authority_decision(
    durable_append_authority_availability_dry_run: RollbackDurableAppendAuthorityAvailabilityDryRun,
    write_authority_availability: RollbackDurableAuditPolicyWriteAuthorityAvailability,
    audit_policy_availability: RollbackDurableAuditPolicyAvailability,
    durable_append_authority_availability: RollbackDurableAppendAuthorityAvailability,
    transaction_append_dry_run: RollbackTransactionAppendDryRun,
    target_region_sector_inspection: RollbackTargetRegionSectorInspection,
) -> RollbackDurablePolicyWriteAuthorityDecision {
    let target_span_verified = transaction_append_dry_run.target_span_verified
        && target_region_sector_inspection.target_span_verified
        && durable_append_authority_availability_dry_run.target_span_verified
        && write_authority_availability.target_span_verified
        && audit_policy_availability.target_span_verified
        && durable_append_authority_availability.target_span_verified
        && transaction_append_dry_run.target_start_lba
            == durable_append_authority_availability_dry_run.target_start_lba
        && transaction_append_dry_run.target_lba_count
            == durable_append_authority_availability_dry_run.target_lba_count
        && transaction_append_dry_run.target_byte_count
            == durable_append_authority_availability_dry_run.target_byte_count
        && transaction_append_dry_run.target_start_lba
            == target_region_sector_inspection.target_start_lba
        && transaction_append_dry_run.target_lba_count
            == target_region_sector_inspection.target_lba_count
        && transaction_append_dry_run.target_byte_count
            == target_region_sector_inspection.target_byte_count
        && transaction_append_dry_run.target_start_lba
            == write_authority_availability.target_start_lba
        && transaction_append_dry_run.target_lba_count
            == write_authority_availability.target_lba_count
        && transaction_append_dry_run.target_byte_count
            == write_authority_availability.target_byte_count
        && transaction_append_dry_run.target_start_lba
            == audit_policy_availability.target_start_lba
        && transaction_append_dry_run.target_lba_count
            == audit_policy_availability.target_lba_count
        && transaction_append_dry_run.target_byte_count
            == audit_policy_availability.target_byte_count
        && transaction_append_dry_run.target_start_lba
            == durable_append_authority_availability.target_start_lba
        && transaction_append_dry_run.target_lba_count
            == durable_append_authority_availability.target_lba_count
        && transaction_append_dry_run.target_byte_count
            == durable_append_authority_availability.target_byte_count;
    let transaction_append_dry_run_verified = transaction_append_dry_run
        .authority_denial_gate_verified
        && transaction_append_dry_run.target_region_write_readback_verified
        && transaction_append_dry_run.append_image_ready
        && transaction_append_dry_run.blocked_by_authority_denial_gate
        && transaction_append_dry_run.test_infrastructure_media_write_authority_available
        && !transaction_append_dry_run.transaction_append_available
        && target_span_verified;
    let target_region_sector_inspection_verified = target_region_sector_inspection
        .inspection_verified
        && target_region_sector_inspection.target_region_write_readback_verified
        && target_region_sector_inspection.sector_hash_verified
        && target_region_sector_inspection.audit_record_hash_verified
        && target_region_sector_inspection.rollback_transaction_hash_verified
        && target_region_sector_inspection.offsets_verified
        && target_region_sector_inspection.padding_zeroed
        && target_span_verified;
    let write_authority_evidence_verified = write_authority_availability.ledger_evidence_verified
        && write_authority_availability.media_write_policy_verified
        && write_authority_availability.target_region_write_readback_verified
        && write_authority_availability.audit_rollback_target_ids_verified
        && !write_authority_availability.write_authority_available;
    let audit_policy_availability_evidence_verified = audit_policy_availability
        .policy_ledger_availability_evidence_verified
        && audit_policy_availability.write_authority_evidence_verified
        && audit_policy_availability.ledger_evidence_verified
        && audit_policy_availability.media_write_policy_verified
        && audit_policy_availability.target_region_write_readback_verified
        && audit_policy_availability.audit_rollback_target_ids_verified
        && !audit_policy_availability.durable_audit_policy_available;
    let durable_append_authority_availability_evidence_verified =
        durable_append_authority_availability.audit_policy_availability_evidence_verified
            && durable_append_authority_availability.write_authority_evidence_verified
            && durable_append_authority_availability.ledger_evidence_verified
            && durable_append_authority_availability.media_write_policy_verified
            && durable_append_authority_availability.target_region_write_readback_verified
            && durable_append_authority_availability.audit_rollback_target_ids_verified
            && !durable_append_authority_availability.durable_append_authority_available;
    let durable_append_authority_availability_dry_run_verified =
        durable_append_authority_availability_dry_run
            .append_authority_availability_evidence_verified
            && durable_append_authority_availability_dry_run
                .transaction_append_denial_gate_verified
            && durable_append_authority_availability_dry_run
                .source_append_authority_availability_hash
                == durable_append_authority_availability.availability_hash
            && durable_append_authority_availability_dry_run.source_authority_denial_gate_hash
                == transaction_append_dry_run.source_authority_denial_gate_hash
            && durable_append_authority_availability_dry_run
                .source_transaction_append_availability_decision_hash
                == transaction_append_dry_run.source_transaction_append_availability_decision_hash
            && !durable_append_authority_availability_dry_run.durable_append_authority_available
            && !durable_append_authority_availability_dry_run.transaction_append_available;
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS,
    );
    hash_line_str(
        &mut hash,
        b"reason",
        HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_availability_dry_run_sha256",
        durable_append_authority_availability_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_dry_run_sha256",
        transaction_append_dry_run.dry_run_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_sector_inspection_sha256",
        target_region_sector_inspection.inspection_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_write_authority_availability_sha256",
        write_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_audit_policy_availability_sha256",
        audit_policy_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_durable_append_authority_availability_sha256",
        durable_append_authority_availability.availability_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_authority_denial_gate_sha256",
        transaction_append_dry_run.source_authority_denial_gate_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_transaction_append_availability_decision_sha256",
        transaction_append_dry_run.source_transaction_append_availability_decision_hash,
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
        transaction_append_dry_run.target_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        transaction_append_dry_run.target_lba_count,
    );
    hash_line_u64(
        &mut hash,
        b"target_byte_count",
        transaction_append_dry_run.target_byte_count,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_dry_run_verified",
        transaction_append_dry_run_verified,
    );
    hash_line_bool(
        &mut hash,
        b"target_region_sector_inspection_verified",
        target_region_sector_inspection_verified,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_evidence_verified",
        write_authority_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"audit_policy_availability_evidence_verified",
        audit_policy_availability_evidence_verified,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_availability_evidence_verified",
        durable_append_authority_availability_evidence_verified
            && durable_append_authority_availability_dry_run_verified,
    );
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        transaction_append_dry_run.test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"write_authority_available",
        write_authority_availability.write_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_policy_ledger_available",
        write_authority_availability.durable_policy_ledger_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_audit_policy_available",
        audit_policy_availability.durable_audit_policy_available,
    );
    hash_line_bool(
        &mut hash,
        b"durable_append_authority_available",
        durable_append_authority_availability.durable_append_authority_available,
    );
    hash_line_bool(
        &mut hash,
        b"transaction_append_available",
        transaction_append_dry_run.transaction_append_available,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"authorizes_transaction_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"write_attempted", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    RollbackDurablePolicyWriteAuthorityDecision {
        decision_hash: finalize_sha256(hash),
        source_durable_append_authority_availability_dry_run_hash:
            durable_append_authority_availability_dry_run.dry_run_hash,
        source_transaction_append_dry_run_hash: transaction_append_dry_run.dry_run_hash,
        source_target_region_sector_inspection_hash: target_region_sector_inspection
            .inspection_hash,
        source_write_authority_availability_hash: write_authority_availability.availability_hash,
        source_audit_policy_availability_hash: audit_policy_availability.availability_hash,
        source_durable_append_authority_availability_hash: durable_append_authority_availability
            .availability_hash,
        source_authority_denial_gate_hash: transaction_append_dry_run
            .source_authority_denial_gate_hash,
        source_transaction_append_availability_decision_hash: transaction_append_dry_run
            .source_transaction_append_availability_decision_hash,
        target_start_lba: transaction_append_dry_run.target_start_lba,
        target_lba_count: transaction_append_dry_run.target_lba_count,
        target_byte_count: transaction_append_dry_run.target_byte_count,
        transaction_append_dry_run_verified,
        target_region_sector_inspection_verified,
        write_authority_evidence_verified,
        audit_policy_availability_evidence_verified,
        durable_append_authority_availability_evidence_verified:
            durable_append_authority_availability_evidence_verified
                && durable_append_authority_availability_dry_run_verified,
        target_span_verified,
        test_infrastructure_media_write_authority_available: transaction_append_dry_run
            .test_infrastructure_media_write_authority_available,
        write_authority_available: write_authority_availability.write_authority_available,
        durable_policy_ledger_available: write_authority_availability
            .durable_policy_ledger_available,
        durable_audit_policy_available: audit_policy_availability.durable_audit_policy_available,
        durable_append_authority_available: durable_append_authority_availability
            .durable_append_authority_available,
        transaction_append_available: transaction_append_dry_run.transaction_append_available,
    }
}

pub(crate) fn hello_rollback_target_region_write_readback_dry_run(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    sector_plan: RollbackAppendSectorPlanDryRun,
    foundation: RollbackWriterStorageFoundation,
    target_region_media_write_policy_preflight: TargetRegionMediaWritePolicyPreflight,
) -> RollbackTargetRegionWriteReadbackDryRun {
    let planned_image = hello_rollback_append_sector_image(snapshot, probation);
    let evidence = pci::find_mass_storage_controller().map(|controller| {
        ahci::write_readback_audit_rollback_target_sector_image(
            controller,
            &planned_image,
            sector_plan.sector_image_hash,
        )
    });
    hello_rollback_target_region_write_readback_dry_run_from_evidence(
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
        evidence,
        "pci_mass_storage_controller_missing",
    )
}

pub(crate) fn hello_rollback_target_region_write_readback_dry_run_from_materializer(
    sector_plan: RollbackAppendSectorPlanDryRun,
    foundation: RollbackWriterStorageFoundation,
    target_region_media_write_policy_preflight: TargetRegionMediaWritePolicyPreflight,
) -> RollbackTargetRegionWriteReadbackDryRun {
    hello_rollback_target_region_write_readback_dry_run_from_evidence(
        sector_plan,
        foundation,
        target_region_media_write_policy_preflight,
        ahci::cached_audit_rollback_target_sector_write_readback(sector_plan.sector_image_hash),
        HELLO_ROLLBACK_TARGET_REGION_MATERIALIZER_MISSING_REASON,
    )
}

pub(crate) fn hello_rollback_target_region_write_readback_dry_run_from_evidence(
    sector_plan: RollbackAppendSectorPlanDryRun,
    foundation: RollbackWriterStorageFoundation,
    target_region_media_write_policy_preflight: TargetRegionMediaWritePolicyPreflight,
    evidence: Option<ahci::AhciAuditRollbackTargetSectorWriteReadbackEvidence>,
    missing_reason: &'static str,
) -> RollbackTargetRegionWriteReadbackDryRun {
    let target_region = foundation.target_region_discovery;
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
                && target_region.durable_region_available
                && evidence.available
                && evidence.region_id == ahci::AUDIT_ROLLBACK_TARGET_REGION_ID
                && evidence.write_readback_id
                    == ahci::AUDIT_ROLLBACK_TARGET_SECTOR_WRITE_READBACK_ID
                && evidence.target_lba == target_region.candidate_region_start_lba
                && evidence.byte_count as u64
                    == target_region.candidate_region_lba_count * ahci::SECTOR_BYTES as u64
                && evidence.region_within_device_bounds
                && evidence.no_boot_or_partition_metadata_overlap
                && !evidence.scratch_region_overlap,
            evidence.write_attempted,
            evidence.write_completed,
            evidence.readback_completed,
            evidence.readback_matches_planned_image,
            if evidence.available {
                HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_STATUS
            } else {
                "missing"
            },
            if evidence.available {
                HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_REASON
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
            missing_reason,
        ),
    };
    let test_infrastructure_media_write_authority_available = target_range_ready
        && write_completed
        && readback_completed
        && readback_matches_planned_image;
    let target_byte_count = target_region.candidate_region_lba_count * ahci::SECTOR_BYTES as u64;

    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", status);
    hash_line_str(&mut hash, b"reason", reason);
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        sector_plan.plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_policy_preflight_sha256",
        target_region_media_write_policy_preflight.preflight_hash,
    );
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
    hash_line_str(
        &mut hash,
        b"target_region_id",
        ahci::AUDIT_ROLLBACK_TARGET_REGION_ID,
    );
    hash_line_u64(
        &mut hash,
        b"target_start_lba",
        target_region.candidate_region_start_lba,
    );
    hash_line_u64(
        &mut hash,
        b"target_lba_count",
        target_region.candidate_region_lba_count,
    );
    hash_line_u64(&mut hash, b"target_byte_count", target_byte_count);
    hash_line_bool(&mut hash, b"label_found", label_found);
    hash_line_bool(&mut hash, b"target_range_ready", target_range_ready);
    hash_line_bool(
        &mut hash,
        b"test_infrastructure_media_write_authority_available",
        test_infrastructure_media_write_authority_available,
    );
    hash_line_bool(&mut hash, b"target_region_write_attempted", write_attempted);
    hash_line_bool(&mut hash, b"target_region_write_completed", write_completed);
    hash_line_bool(
        &mut hash,
        b"target_region_readback_completed",
        readback_completed,
    );
    hash_line_bool(
        &mut hash,
        b"readback_matches_planned_image",
        readback_matches_planned_image,
    );
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_state", false);

    RollbackTargetRegionWriteReadbackDryRun {
        dry_run_hash: finalize_sha256(hash),
        source_sector_plan_hash: sector_plan.plan_hash,
        source_policy_preflight_hash: target_region_media_write_policy_preflight.preflight_hash,
        planned_sector_image_hash,
        readback_sector_image_hash,
        target_start_lba: target_region.candidate_region_start_lba,
        target_lba_count: target_region.candidate_region_lba_count,
        target_byte_count,
        label_found,
        target_range_ready,
        test_infrastructure_media_write_authority_available,
        write_attempted,
        write_completed,
        readback_completed,
        readback_matches_planned_image,
        status,
        reason,
    }
}

pub(crate) fn hello_rollback_target_region_sector_inspection(
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackTargetRegionSectorInspection {
    let evidence = pci::find_mass_storage_controller().map(|controller| {
        ahci::inspect_audit_rollback_target_sector_image(
            controller,
            sector_plan.sector_image_hash,
            sector_plan.audit_record_offset as usize,
            sector_plan.audit_record_byte_length as usize,
            sector_plan.rollback_transaction_offset as usize,
            sector_plan.rollback_transaction_byte_length as usize,
            sector_plan.padding_offset as usize,
            sector_plan.padding_byte_length as usize,
        )
    });
    hello_rollback_target_region_sector_inspection_from_evidence(
        append_record,
        sector_plan,
        target_region_write,
        evidence,
        "pci_mass_storage_controller_missing",
    )
}

pub(crate) fn hello_rollback_target_region_sector_inspection_from_retained_inspect(
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
) -> RollbackTargetRegionSectorInspection {
    hello_rollback_target_region_sector_inspection_from_evidence(
        append_record,
        sector_plan,
        target_region_write,
        ahci::cached_audit_rollback_target_sector_inspection(sector_plan.sector_image_hash),
        HELLO_ROLLBACK_TARGET_REGION_INSPECTION_MISSING_REASON,
    )
}

pub(crate) fn hello_rollback_target_region_sector_inspection_from_evidence(
    append_record: RollbackAppendRecordDryRun,
    sector_plan: RollbackAppendSectorPlanDryRun,
    target_region_write: RollbackTargetRegionWriteReadbackDryRun,
    evidence: Option<ahci::AhciAuditRollbackTargetSectorInspectionEvidence>,
    missing_reason: &'static str,
) -> RollbackTargetRegionSectorInspection {
    let (
        expected_sector_image_hash,
        sector_image_hash,
        audit_record_image_hash,
        rollback_transaction_image_hash,
        label_found,
        read_attempted,
        read_completed,
        sector_hash_verified,
        audit_record_hash_verified,
        rollback_transaction_hash_verified,
        offsets_verified,
        padding_zeroed,
        target_span_verified,
        target_region_write_readback_verified,
        inspection_verified,
        status,
        reason,
    ) = match evidence {
        Some(evidence) => {
            let target_span_verified = sector_plan.target_range_ready
                && evidence.region_id == ahci::AUDIT_ROLLBACK_TARGET_REGION_ID
                && evidence.inspection_id == ahci::AUDIT_ROLLBACK_TARGET_SECTOR_INSPECTION_ID
                && evidence.target_lba == target_region_write.target_start_lba
                && evidence.byte_count as u64 == target_region_write.target_byte_count
                && evidence.region_within_device_bounds
                && evidence.no_boot_or_partition_metadata_overlap
                && !evidence.scratch_region_overlap;
            let sector_hash_verified = evidence.read_matches_expected_image
                && evidence.expected_sector_image_hash == sector_plan.sector_image_hash
                && evidence.sector_image_hash == sector_plan.sector_image_hash;
            let audit_record_hash_verified =
                evidence.audit_record_image_hash == append_record.audit_record_image_hash;
            let rollback_transaction_hash_verified = evidence.rollback_transaction_image_hash
                == append_record.rollback_transaction_image_hash;
            let offsets_verified = evidence.offsets_within_sector
                && sector_plan.audit_record_offset == 0
                && sector_plan.audit_record_byte_length == append_record.audit_record_byte_length
                && sector_plan.rollback_transaction_offset
                    == append_record.audit_record_byte_length
                && sector_plan.rollback_transaction_byte_length
                    == append_record.rollback_transaction_byte_length
                && sector_plan.padding_offset == append_record.total_record_byte_length
                && sector_plan.padding_byte_length
                    == sector_plan
                        .target_byte_count
                        .saturating_sub(append_record.total_record_byte_length);
            let target_region_write_readback_verified = target_region_write
                .test_infrastructure_media_write_authority_available
                && target_region_write.readback_matches_planned_image
                && target_region_write.readback_sector_image_hash == evidence.sector_image_hash
                && target_region_write.planned_sector_image_hash
                    == evidence.expected_sector_image_hash;
            let inspection_verified = evidence.available
                && target_span_verified
                && target_region_write_readback_verified
                && sector_hash_verified
                && audit_record_hash_verified
                && rollback_transaction_hash_verified
                && offsets_verified
                && evidence.padding_zeroed;
            (
                evidence.expected_sector_image_hash,
                evidence.sector_image_hash,
                evidence.audit_record_image_hash,
                evidence.rollback_transaction_image_hash,
                evidence.label_found,
                evidence.read_attempted,
                evidence.read_completed,
                sector_hash_verified,
                audit_record_hash_verified,
                rollback_transaction_hash_verified,
                offsets_verified,
                evidence.padding_zeroed,
                target_span_verified,
                target_region_write_readback_verified,
                inspection_verified,
                if inspection_verified {
                    HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_STATUS
                } else {
                    "missing"
                },
                if inspection_verified {
                    HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_REASON
                } else {
                    evidence.reason
                },
            )
        }
        None => (
            sector_plan.sector_image_hash,
            [0; 32],
            [0; 32],
            [0; 32],
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
            "missing",
            missing_reason,
        ),
    };
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", status);
    hash_line_str(&mut hash, b"reason", reason);
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
    hash_line_hash(
        &mut hash,
        b"expected_sector_image_sha256",
        expected_sector_image_hash,
    );
    hash_line_hash(&mut hash, b"sector_image_sha256", sector_image_hash);
    hash_line_hash(
        &mut hash,
        b"audit_record_image_sha256",
        audit_record_image_hash,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_transaction_image_sha256",
        rollback_transaction_image_hash,
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
    hash_line_u64(
        &mut hash,
        b"audit_record_offset",
        sector_plan.audit_record_offset,
    );
    hash_line_u64(
        &mut hash,
        b"audit_record_byte_length",
        sector_plan.audit_record_byte_length,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_transaction_offset",
        sector_plan.rollback_transaction_offset,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_transaction_byte_length",
        sector_plan.rollback_transaction_byte_length,
    );
    hash_line_u64(&mut hash, b"padding_offset", sector_plan.padding_offset);
    hash_line_u64(
        &mut hash,
        b"padding_byte_length",
        sector_plan.padding_byte_length,
    );
    hash_line_bool(&mut hash, b"label_found", label_found);
    hash_line_bool(&mut hash, b"target_region_read_attempted", read_attempted);
    hash_line_bool(&mut hash, b"target_region_read_completed", read_completed);
    hash_line_bool(&mut hash, b"sector_hash_verified", sector_hash_verified);
    hash_line_bool(
        &mut hash,
        b"audit_record_hash_verified",
        audit_record_hash_verified,
    );
    hash_line_bool(
        &mut hash,
        b"rollback_transaction_hash_verified",
        rollback_transaction_hash_verified,
    );
    hash_line_bool(&mut hash, b"offsets_verified", offsets_verified);
    hash_line_bool(&mut hash, b"padding_zeroed", padding_zeroed);
    hash_line_bool(&mut hash, b"target_span_verified", target_span_verified);
    hash_line_bool(
        &mut hash,
        b"target_region_write_readback_verified",
        target_region_write_readback_verified,
    );
    hash_line_bool(&mut hash, b"inspection_verified", inspection_verified);
    hash_line_bool(&mut hash, b"authorizes_media_write", false);
    hash_line_bool(&mut hash, b"authorizes_append", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_state", false);

    RollbackTargetRegionSectorInspection {
        inspection_hash: finalize_sha256(hash),
        source_sector_plan_hash: sector_plan.plan_hash,
        source_target_region_write_readback_hash: target_region_write.dry_run_hash,
        expected_sector_image_hash,
        sector_image_hash,
        audit_record_image_hash,
        rollback_transaction_image_hash,
        target_start_lba: target_region_write.target_start_lba,
        target_lba_count: target_region_write.target_lba_count,
        target_byte_count: target_region_write.target_byte_count,
        audit_record_offset: sector_plan.audit_record_offset,
        audit_record_byte_length: sector_plan.audit_record_byte_length,
        rollback_transaction_offset: sector_plan.rollback_transaction_offset,
        rollback_transaction_byte_length: sector_plan.rollback_transaction_byte_length,
        padding_offset: sector_plan.padding_offset,
        padding_byte_length: sector_plan.padding_byte_length,
        label_found,
        read_attempted,
        read_completed,
        sector_hash_verified,
        audit_record_hash_verified,
        rollback_transaction_hash_verified,
        offsets_verified,
        padding_zeroed,
        target_span_verified,
        target_region_write_readback_verified,
        inspection_verified,
        status,
        reason,
    }
}

pub(crate) fn recovery_rollback_inspect_source_reference_hash(
    event_id: event_log::EventId,
    inspection: RollbackTargetRegionSectorInspection,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"id",
        HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_ID,
    );
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"source_method", "recovery.rollback_inspect");
    hash_line_u64(&mut hash, b"source_event_sequence", event_id.sequence());
    hash_line_hash(
        &mut hash,
        b"sector_inspection_sha256",
        inspection.inspection_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_sector_plan_sha256",
        inspection.source_sector_plan_hash,
    );
    hash_line_hash(
        &mut hash,
        b"source_target_region_write_readback_sha256",
        inspection.source_target_region_write_readback_hash,
    );
    hash_line_bool(
        &mut hash,
        b"sector_inspection_verified",
        inspection.inspection_verified,
    );
    hash_line_bool(&mut hash, b"authorizes_rollback_apply", false);
    finalize_sha256(hash)
}

pub(crate) fn retain_recovery_rollback_inspect_source_reference(
    event_id: event_log::EventId,
    inspection: RollbackTargetRegionSectorInspection,
) {
    if !inspection.inspection_verified {
        return;
    }
    let reference_hash = recovery_rollback_inspect_source_reference_hash(event_id, inspection);
    let audit_event_id = event_log::record_hello_recovery_rollback_inspect_source_reference(
        event_log::HelloRecoveryRollbackInspectSourceReferenceBinding {
            source_event_id: event_id,
            reference_hash,
            inspection_hash: inspection.inspection_hash,
            source_sector_plan_hash: inspection.source_sector_plan_hash,
            source_target_region_write_readback_hash: inspection
                .source_target_region_write_readback_hash,
            authorizes_rollback_apply: false,
        },
    );
    let reference = RecoveryRollbackInspectSourceReference {
        audit_event_id,
        event_id,
        reference_hash,
        inspection_hash: inspection.inspection_hash,
        source_sector_plan_hash: inspection.source_sector_plan_hash,
        source_target_region_write_readback_hash: inspection
            .source_target_region_write_readback_hash,
    };
    *RETAINED_RECOVERY_ROLLBACK_INSPECT_SOURCE.lock() = Some(reference);
}

pub(crate) fn recovery_rollback_inspect_source_reference_state(
    inspection: RollbackTargetRegionSectorInspection,
) -> RecoveryRollbackInspectSourceReferenceState {
    let Some(reference) = *RETAINED_RECOVERY_ROLLBACK_INSPECT_SOURCE.lock() else {
        return RecoveryRollbackInspectSourceReferenceState {
            reference: None,
            status: "missing",
            reason: HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_MISSING_REASON,
            ram_audit_status: "missing",
            ram_audit_reason: HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_MISSING_REASON,
            source_event_retained: false,
            audit_event_retained: false,
            ram_audit_validated: false,
        };
    };
    if !inspection.inspection_verified
        || reference.inspection_hash != inspection.inspection_hash
        || reference.source_sector_plan_hash != inspection.source_sector_plan_hash
        || reference.source_target_region_write_readback_hash
            != inspection.source_target_region_write_readback_hash
    {
        return RecoveryRollbackInspectSourceReferenceState {
            reference: Some(reference),
            status: "rejected",
            reason: "recovery_rollback_inspect_source_mismatched_sector_inspection",
            ram_audit_status: "rejected",
            ram_audit_reason: "recovery_rollback_inspect_source_mismatched_sector_inspection",
            source_event_retained: false,
            audit_event_retained: false,
            ram_audit_validated: false,
        };
    }
    let check = event_log::check_hello_recovery_rollback_inspect_source_reference(
        reference.audit_event_id,
        event_log::HelloRecoveryRollbackInspectSourceReferenceBinding {
            source_event_id: reference.event_id,
            reference_hash: reference.reference_hash,
            inspection_hash: reference.inspection_hash,
            source_sector_plan_hash: reference.source_sector_plan_hash,
            source_target_region_write_readback_hash: reference
                .source_target_region_write_readback_hash,
            authorizes_rollback_apply: false,
        },
    );
    if check.validated {
        RecoveryRollbackInspectSourceReferenceState {
            reference: Some(reference),
            status: HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_STATUS,
            reason: HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_REASON,
            ram_audit_status: check.status,
            ram_audit_reason: check.reason,
            source_event_retained: check.source_event_retained,
            audit_event_retained: check.audit_event_retained,
            ram_audit_validated: true,
        }
    } else {
        RecoveryRollbackInspectSourceReferenceState {
            reference: Some(reference),
            status: "rejected",
            reason: check.reason,
            ram_audit_status: check.status,
            ram_audit_reason: check.reason,
            source_event_retained: check.source_event_retained,
            audit_event_retained: check.audit_event_retained,
            ram_audit_validated: false,
        }
    }
}

pub(crate) fn hello_rollback_writer_storage_foundation() -> RollbackWriterStorageFoundation {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let engine_evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);
    let append_contract = module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
        storage_evaluation,
        engine_evaluation,
    );
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let rollback_transaction_envelope_available =
        method_eq(append_evaluation.rollback_transaction_status, "available");
    let scratch = storage
        .persistence_device_inventory
        .ahci_probe
        .scratch_write_readback;
    let target_region_discovery = rollback_storage_layout::audit_rollback_target_region_discovery(
        storage.persistence_device_inventory,
    );
    let target_region_writer_contract_ready =
        rollback_append_contract::audit_rollback_target_region_writer_contract_ready(
            target_region_discovery,
        );
    let scratch_writer_dry_run_ready =
        rollback_append_contract::audit_rollback_scratch_writer_dry_run_ready(scratch);

    RollbackWriterStorageFoundation {
        storage_layout_status: storage_evaluation.status,
        storage_layout_reason: storage_evaluation.reason,
        storage_layout_available: storage_evaluation.storage_layout_available,
        append_engine_status: engine_evaluation.status,
        append_engine_reason: engine_evaluation.reason,
        append_engine_available: engine_evaluation.append_engine_available,
        append_contract_status: append_evaluation.status,
        append_contract_reason: append_evaluation.reason,
        rollback_transaction_envelope_status: append_evaluation.rollback_transaction_status,
        rollback_transaction_envelope_reason: append_evaluation.rollback_transaction_reason,
        rollback_transaction_envelope_available,
        append_target_owner_status: append_evaluation.append_target_owner_status,
        append_target_owner_reason: append_evaluation.append_target_owner_reason,
        append_target_owner_available: append_evaluation.append_target_owner_available,
        transaction_writer_status: append_evaluation.transaction_writer_status,
        transaction_writer_reason: append_evaluation.transaction_writer_reason,
        transaction_writer_ready: append_evaluation.transaction_writer_ready,
        block_write_path_available: append_evaluation.block_write_path_available,
        block_write_path_reason: append_evaluation.block_write_path_reason,
        block_write_path_gate_status:
            rollback_storage_layout::audit_rollback_block_write_path_gate_status(
                append_evaluation.block_write_path_available,
            ),
        read_only_block_driver_id: storage
            .persistence_device_inventory
            .ahci_probe
            .block_driver
            .driver_id,
        read_only_block_driver_available: storage
            .persistence_device_inventory
            .ahci_probe
            .block_driver
            .available,
        partition_inventory_available: storage
            .persistence_device_inventory
            .ahci_probe
            .partition_inventory
            .available,
        partition_inventory_scheme: storage
            .persistence_device_inventory
            .ahci_probe
            .partition_inventory
            .scheme,
        target_region_discovery,
        target_region_writer_contract_status:
            rollback_append_contract::audit_rollback_target_region_writer_contract_status(
                target_region_writer_contract_ready,
            ),
        target_region_writer_contract_reason:
            rollback_append_contract::audit_rollback_target_region_writer_contract_reason(
                target_region_writer_contract_ready,
            ),
        target_region_writer_contract_ready,
        target_region_media_write_policy_preflight_status:
            rollback_append_contract::audit_rollback_target_region_media_write_policy_preflight_status(
                target_region_writer_contract_ready,
            ),
        target_region_media_write_policy_preflight_reason:
            rollback_append_contract::audit_rollback_target_region_media_write_policy_preflight_reason(
                target_region_writer_contract_ready,
            ),
        scratch_block_write_authority_id: ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID,
        scratch_block_write_authority_available: scratch.block_write_authority_available,
        scratch_region_within_device_bounds: scratch.region_within_device_bounds,
        scratch_region_no_boot_or_partition_metadata_overlap: scratch
            .no_boot_or_partition_metadata_overlap,
        scratch_region_id: scratch.region_id,
        scratch_region_start_lba: scratch.region_start_lba,
        scratch_region_lba_count: scratch.region_lba_count,
        scratch_region_byte_count: scratch.byte_count,
        scratch_writer_dry_run_status:
            rollback_append_contract::audit_rollback_scratch_writer_dry_run_status(
                scratch_writer_dry_run_ready,
            ),
        scratch_writer_dry_run_reason:
            rollback_append_contract::audit_rollback_scratch_writer_dry_run_reason(
                scratch_writer_dry_run_ready,
            ),
        scratch_writer_dry_run_ready,
        transaction_writer_available: rollback_transaction_envelope_available
            && append_evaluation.transaction_writer_ready
            && append_evaluation.writes_enabled,
        durable_audit_store_available: storage_evaluation.storage_layout_available
            && engine_evaluation.audit_engine_available
            && append_evaluation.writes_enabled,
        rollback_store_available: storage_evaluation.storage_layout_available
            && engine_evaluation.rollback_engine_available
            && append_evaluation.writes_enabled,
        rollback_transaction_append_available: rollback_transaction_envelope_available
            && append_evaluation.writes_enabled,
    }
}

pub(crate) fn hello_rollback_payload_envelope_gate_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_STATUS,
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
    hash_line_str(
        &mut hash,
        b"payload_schema",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA,
    );
    hash_line_str(
        &mut hash,
        b"payload_id",
        HELLO_ROLLBACK_TRANSACTION_PAYLOAD_ID,
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
        b"rollback_append_intent_gate_sha256",
        hello_rollback_append_intent_gate_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_write_authority_gate_sha256",
        hello_rollback_write_authority_gate_hash(snapshot, probation),
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
    hash_line_bool(&mut hash, b"transaction_writer_available", false);
    hash_line_bool(&mut hash, b"durable_audit_store_available", false);
    hash_line_bool(&mut hash, b"rollback_store_available", false);
    hash_line_bool(&mut hash, b"rollback_transaction_append_available", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"appends_rollback_transaction", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    finalize_sha256(hash)
}
