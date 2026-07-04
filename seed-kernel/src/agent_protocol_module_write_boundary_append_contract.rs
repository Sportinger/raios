use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_engine::*,
    agent_protocol_module_write_boundary_storage_layout::*, agent_protocol_support::*, ahci,
};

pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_METHOD: &str =
    "module.audit_rollback_append_contract";
pub(crate) const AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA: &str =
    "raios.audit_rollback_append_target_owner.v0";
pub(crate) const AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID: &str =
    "append_target_owner.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA: &str =
    "raios.audit_rollback_transaction_writer_readiness.v0";
pub(crate) const AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID: &str =
    "transaction_writer.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA: &str =
    "raios.audit_rollback_transaction_writer_scratch_dry_run.v0";
pub(crate) const AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID: &str =
    "transaction_writer.scratch_dry_run.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA: &str =
    "raios.audit_rollback_target_region_writer_contract.v0";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID: &str =
    "target_region_writer_contract.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA: &str =
    "raios.audit_rollback_target_region_media_write_policy_preflight.v0";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID: &str =
    "target_region_media_write_policy_preflight.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_WRITER_READINESS_MISSING_STATUS: &str = "missing";
pub(crate) const AUDIT_ROLLBACK_WRITER_READINESS_MISSING_REASON: &str =
    "block_write_path_unavailable";

pub(crate) fn emit_module_audit_rollback_append_contract() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let engine_evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);
    let append = module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
        storage_evaluation,
        engine_evaluation,
    );
    let evaluation = evaluate_module_audit_rollback_append_contract_candidate(append);

    begin_response("module.audit_rollback_append_contract");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_contract.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    emit_module_append_contract_storage_layout_inputs(storage, storage_evaluation);
    raw_line(",");
    emit_module_append_contract_append_engine_inputs(engine, engine_evaluation);
    raw_line(",");
    emit_module_append_contract_facts(append, evaluation);
    raw_line(",");
    emit_module_append_target_owner(evaluation);
    raw_line(",");
    emit_module_transaction_writer_readiness(storage, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"append_target_owner_status\": ");
    json_str(evaluation.append_target_owner_status);
    raw_line(",");
    raw("        \"append_target_owner_reason\": ");
    json_str(evaluation.append_target_owner_reason);
    raw_line(",");
    raw("        \"append_target_owner_available\": ");
    raw_bool(evaluation.append_target_owner_available);
    raw_line(",");
    raw("        \"transaction_writer_status\": ");
    json_str(evaluation.transaction_writer_status);
    raw_line(",");
    raw("        \"transaction_writer_reason\": ");
    json_str(evaluation.transaction_writer_reason);
    raw_line(",");
    raw("        \"transaction_writer_ready\": ");
    raw_bool(evaluation.transaction_writer_ready);
    raw_line(",");
    raw("        \"block_write_path_available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("        \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    raw("        \"append_contract_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"append_contract_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"audit_append_missing\": ");
    raw_bool(!method_eq(evaluation.audit_append_status, "available"));
    raw_line(",");
    raw("        \"rollback_transaction_missing\": ");
    raw_bool(!method_eq(
        evaluation.rollback_transaction_status,
        "available",
    ));
    raw_line(",");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"append_envelopes_are_durable_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_authority\": false,");
    raw_line("        \"policy_facts_are_append_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "audit_append_envelope",
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_envelope",
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_contract");
}

pub(crate) fn emit_module_audit_rollback_append_contract_selftest() {
    let cases = module_audit_rollback_append_contract_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_append_contract_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_contract_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_audit_rollback_append_contract_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_contract_selftest");
}

pub(crate) fn emit_module_audit_rollback_append_contract_selftest_case(
    case: &ModuleAuditRollbackAppendContractSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"writes_enabled\": false, \"installs_rollback_plan\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_append_contract_snapshot(
) -> ModuleAuditRollbackAppendContractCandidate {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let engine_evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);
    module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
        storage_evaluation,
        engine_evaluation,
    )
}

pub(crate) fn module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
    storage: ModuleAuditRollbackStorageLayoutEvaluation,
    engine: ModuleAuditRollbackAppendEngineEvaluation,
) -> ModuleAuditRollbackAppendContractCandidate {
    ModuleAuditRollbackAppendContractCandidate {
        audit_append_envelope: module_audit_rollback_missing_append_contract_fact(storage, engine),
        rollback_transaction_envelope: module_audit_rollback_missing_append_contract_fact(
            storage, engine,
        ),
    }
}

pub(crate) fn module_audit_rollback_missing_append_contract_fact(
    storage: ModuleAuditRollbackStorageLayoutEvaluation,
    engine: ModuleAuditRollbackAppendEngineEvaluation,
) -> ModuleAuditRollbackAppendContractFact {
    ModuleAuditRollbackAppendContractFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_write_policy: false,
        binds_availability: false,
        binds_storage_layout_id: false,
        binds_append_engine_id: false,
        binds_write_policy_id: false,
        binds_availability_id: false,
        binds_envelope_provenance: false,
        storage_layout_available: storage.storage_layout_available,
        append_engine_available: engine.append_engine_available,
        block_write_path_available: storage.block_write_path_available,
        block_write_path_reason: storage.block_write_path_reason,
    }
}

pub(crate) fn module_audit_rollback_available_append_contract_fact(
) -> ModuleAuditRollbackAppendContractFact {
    ModuleAuditRollbackAppendContractFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_write_policy: true,
        binds_availability: true,
        binds_storage_layout_id: true,
        binds_append_engine_id: true,
        binds_write_policy_id: true,
        binds_availability_id: true,
        binds_envelope_provenance: true,
        storage_layout_available: true,
        append_engine_available: true,
        block_write_path_available: true,
        block_write_path_reason: "block_write_path_available",
    }
}

pub(crate) fn evaluate_module_audit_rollback_append_contract_candidate(
    candidate: ModuleAuditRollbackAppendContractCandidate,
) -> ModuleAuditRollbackAppendContractEvaluation {
    let (audit_append_status, audit_append_reason) = evaluate_module_append_contract_fact(
        candidate.audit_append_envelope,
        "audit_append_envelope_scope_must_be_current_boot",
        "audit_append_envelope_schema_mismatch",
        "audit_append_envelope_missing",
        "audit_append_envelope_provenance_missing",
        "audit_append_envelope_write_policy_binding_missing",
        "audit_append_envelope_availability_binding_missing",
        "audit_append_envelope_storage_layout_binding_missing",
        "audit_append_envelope_append_engine_binding_missing",
        "audit_append_envelope_provenance_binding_missing",
        "audit_ledger_storage_layout_missing",
        "audit_ledger_append_engine_missing",
        "audit_append_envelope_available",
    );
    let (rollback_transaction_status, rollback_transaction_reason) =
        evaluate_module_append_contract_fact(
            candidate.rollback_transaction_envelope,
            "rollback_transaction_envelope_scope_must_be_current_boot",
            "rollback_transaction_envelope_schema_mismatch",
            "rollback_transaction_envelope_missing",
            "rollback_transaction_envelope_provenance_missing",
            "rollback_transaction_envelope_write_policy_binding_missing",
            "rollback_transaction_envelope_availability_binding_missing",
            "rollback_transaction_envelope_storage_layout_binding_missing",
            "rollback_transaction_envelope_append_engine_binding_missing",
            "rollback_transaction_envelope_provenance_binding_missing",
            "rollback_store_storage_layout_missing",
            "rollback_store_append_engine_missing",
            "rollback_transaction_envelope_available",
        );

    let (status, reason) = if method_eq(audit_append_status, "rejected") {
        ("rejected", audit_append_reason)
    } else if method_eq(rollback_transaction_status, "rejected") {
        ("rejected", rollback_transaction_reason)
    } else if method_eq(audit_append_status, "missing")
        && method_eq(audit_append_reason, "audit_append_envelope_missing")
        && method_eq(rollback_transaction_status, "missing")
        && method_eq(
            rollback_transaction_reason,
            "rollback_transaction_envelope_missing",
        )
    {
        (
            "missing",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
        )
    } else if method_eq(audit_append_status, "missing") {
        ("missing", audit_append_reason)
    } else if method_eq(rollback_transaction_status, "missing") {
        ("missing", rollback_transaction_reason)
    } else {
        (
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
        )
    };

    let storage_layout_available = candidate.audit_append_envelope.storage_layout_available
        && candidate
            .rollback_transaction_envelope
            .storage_layout_available;
    let append_engine_available = candidate.audit_append_envelope.append_engine_available
        && candidate
            .rollback_transaction_envelope
            .append_engine_available;
    let block_write_path_available = candidate.audit_append_envelope.block_write_path_available
        && candidate
            .rollback_transaction_envelope
            .block_write_path_available;
    let block_write_path_reason = if block_write_path_available {
        "block_write_path_available"
    } else {
        candidate.audit_append_envelope.block_write_path_reason
    };
    let (append_target_owner_status, append_target_owner_reason, append_target_owner_available) =
        evaluate_append_target_owner_status(
            storage_layout_available,
            append_engine_available,
            block_write_path_available,
            block_write_path_reason,
            audit_append_status,
            rollback_transaction_status,
        );
    let (transaction_writer_status, transaction_writer_reason, transaction_writer_ready) =
        evaluate_transaction_writer_readiness(
            append_target_owner_available,
            false,
            append_target_owner_reason,
        );

    ModuleAuditRollbackAppendContractEvaluation {
        status,
        reason,
        audit_append_status,
        audit_append_reason,
        rollback_transaction_status,
        rollback_transaction_reason,
        storage_layout_available,
        append_engine_available,
        append_target_owner_status,
        append_target_owner_reason,
        append_target_owner_available,
        transaction_writer_status,
        transaction_writer_reason,
        transaction_writer_ready,
        block_write_path_available,
        block_write_path_reason,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_append_target_owner_status(
    storage_layout_available: bool,
    append_engine_available: bool,
    block_write_path_available: bool,
    block_write_path_reason: &'static str,
    audit_append_status: &'static str,
    rollback_transaction_status: &'static str,
) -> (&'static str, &'static str, bool) {
    if !block_write_path_available {
        return (
            AUDIT_ROLLBACK_WRITER_READINESS_MISSING_STATUS,
            block_write_path_reason,
            false,
        );
    }
    if !storage_layout_available {
        return (
            AUDIT_ROLLBACK_WRITER_READINESS_MISSING_STATUS,
            AUDIT_ROLLBACK_WRITER_READINESS_MISSING_REASON,
            false,
        );
    }
    if !append_engine_available {
        return ("missing", "append_engine_unavailable", false);
    }
    if !method_eq(audit_append_status, "available")
        || !method_eq(rollback_transaction_status, "available")
    {
        return ("missing", "append_contract_envelope_missing", false);
    }
    ("available", "append_target_owner_ready", true)
}

pub(crate) fn evaluate_transaction_writer_readiness(
    append_target_owner_available: bool,
    writes_enabled: bool,
    missing_reason: &'static str,
) -> (&'static str, &'static str, bool) {
    if !append_target_owner_available {
        return (
            AUDIT_ROLLBACK_WRITER_READINESS_MISSING_STATUS,
            missing_reason,
            false,
        );
    }
    if !writes_enabled {
        return ("missing", "append_engine_write_path_disabled", false);
    }
    ("available", "transaction_writer_ready", true)
}

pub(crate) fn audit_rollback_scratch_writer_dry_run_ready(
    scratch: ahci::AhciScratchWriteReadbackEvidence,
) -> bool {
    scratch.block_write_authority_available
        && scratch.region_within_device_bounds
        && scratch.no_boot_or_partition_metadata_overlap
}

pub(crate) fn audit_rollback_scratch_writer_dry_run_status(ready: bool) -> &'static str {
    if ready {
        "scratch_range_ready_not_durable_authority"
    } else {
        "missing"
    }
}

pub(crate) fn audit_rollback_scratch_writer_dry_run_reason(ready: bool) -> &'static str {
    if ready {
        "scratch_write_authority_verified_current_boot"
    } else {
        "scratch_block_write_authority_missing"
    }
}

pub(crate) fn audit_rollback_target_region_writer_contract_ready(
    discovery: AuditRollbackTargetRegionDiscovery,
) -> bool {
    discovery.durable_region_available
        && discovery.candidate_region_present
        && !discovery.candidate_region_is_scratch
        && !discovery.candidate_overlaps_boot_metadata
        && !discovery.candidate_overlaps_scratch
}

pub(crate) fn audit_rollback_target_region_writer_contract_status(ready: bool) -> &'static str {
    if ready {
        "target_region_ready_not_write_authority"
    } else {
        "missing"
    }
}

pub(crate) fn audit_rollback_target_region_writer_contract_reason(ready: bool) -> &'static str {
    if ready {
        "target_region_read_only_missing_media_write_authority"
    } else {
        "target_region_discovery_missing"
    }
}

pub(crate) fn audit_rollback_target_region_media_write_policy_preflight_status(
    contract_ready: bool,
) -> &'static str {
    if contract_ready {
        "denied_missing_media_write_authority_and_durable_audit_policy"
    } else {
        "missing"
    }
}

pub(crate) fn audit_rollback_target_region_media_write_policy_preflight_reason(
    contract_ready: bool,
) -> &'static str {
    if contract_ready {
        "target_region_contract_ready_policy_or_write_authority_missing"
    } else {
        "target_region_writer_contract_missing"
    }
}

pub(crate) fn evaluate_module_append_contract_fact(
    fact: ModuleAuditRollbackAppendContractFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    write_policy_reason: &'static str,
    availability_reason: &'static str,
    storage_layout_binding_reason: &'static str,
    append_engine_binding_reason: &'static str,
    provenance_binding_reason: &'static str,
    storage_layout_reason: &'static str,
    append_engine_reason: &'static str,
    available_reason: &'static str,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", scope_reason);
    }
    if !fact.schema_ok {
        return ("rejected", schema_reason);
    }
    if !fact.present {
        return ("missing", missing_reason);
    }
    if !fact.provenance_ok {
        return ("rejected", provenance_reason);
    }
    if !fact.binds_envelope_provenance {
        return ("rejected", provenance_binding_reason);
    }
    if !fact.binds_write_policy {
        return ("rejected", write_policy_reason);
    }
    if !fact.binds_availability {
        return ("rejected", availability_reason);
    }
    if !fact.binds_write_policy_id {
        return ("rejected", write_policy_reason);
    }
    if !fact.binds_availability_id {
        return ("rejected", availability_reason);
    }
    if !fact.binds_storage_layout_id {
        return ("rejected", storage_layout_binding_reason);
    }
    if !fact.binds_append_engine_id {
        return ("rejected", append_engine_binding_reason);
    }
    if !fact.storage_layout_available {
        return ("missing", storage_layout_reason);
    }
    if !fact.append_engine_available {
        return ("missing", append_engine_reason);
    }
    ("available", available_reason)
}

pub(crate) fn emit_module_append_contract_facts(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_facts\": {");
    emit_module_append_contract_fact(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        "append.audit_ledger.current_boot",
        "raios.audit_record.v0",
        "storage.audit_rollback_layout.current_boot",
        "append_engine.audit_ledger.current_boot",
        "policy.durable_audit_write.current_boot",
        "availability.durable_audit_ledger.current_boot",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_append_contract_fact(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        "append.rollback_store.current_boot",
        "raios.rollback_transaction.v0",
        "storage.audit_rollback_layout.current_boot",
        "append_engine.rollback_store.current_boot",
        "policy.rollback_install.current_boot",
        "availability.rollback_store.current_boot",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_append_target_owner(
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_target_owner\": {");
    raw("        \"schema\": ");
    json_str(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
    raw_line(",");
    raw("        \"owner_method\": ");
    json_str(MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_METHOD);
    raw_line(",");
    raw("        \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("        \"storage_authority_owner\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.append_target_owner_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.append_target_owner_reason);
    raw_line(",");
    raw("        \"available\": ");
    raw_bool(evaluation.append_target_owner_available);
    raw_line(",");
    raw("        \"block_write_path_available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("        \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    raw_line("        \"authorizes_append\": false,");
    raw_line("        \"writes_enabled\": false,");
    raw_line("        \"targets\": {");
    raw("          \"audit_ledger\": {\"id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw(", \"schema\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw(", \"owner_bound\": ");
    raw_bool(evaluation.append_target_owner_available);
    raw(", \"available\": false},");
    crlf();
    raw("          \"rollback_store\": {\"id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw(", \"schema\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw(", \"owner_bound\": ");
    raw_bool(evaluation.append_target_owner_available);
    raw(", \"available\": false}");
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_module_transaction_writer_readiness(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    let scratch = storage
        .persistence_device_inventory
        .ahci_probe
        .scratch_write_readback;
    let scratch_dry_run_ready = audit_rollback_scratch_writer_dry_run_ready(scratch);
    let target_region_discovery =
        audit_rollback_target_region_discovery(storage.persistence_device_inventory);
    let target_region_contract_ready =
        audit_rollback_target_region_writer_contract_ready(target_region_discovery);
    raw_line("      \"transaction_writer_readiness\": {");
    raw("        \"schema\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID);
    raw_line(",");
    raw("        \"owner_method\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
    raw_line(",");
    raw("        \"append_target_owner_id\": ");
    json_str(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
    raw_line(",");
    raw("        \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.transaction_writer_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.transaction_writer_reason);
    raw_line(",");
    raw("        \"ready\": ");
    raw_bool(evaluation.transaction_writer_ready);
    raw_line(",");
    raw("        \"block_write_path_available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("        \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    raw_line("        \"scratch_only_writer_dry_run\": {");
    raw("          \"schema\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw("          \"status\": ");
    json_str(audit_rollback_scratch_writer_dry_run_status(
        scratch_dry_run_ready,
    ));
    raw_line(",");
    raw("          \"reason\": ");
    json_str(audit_rollback_scratch_writer_dry_run_reason(
        scratch_dry_run_ready,
    ));
    raw_line(",");
    raw("          \"source_authority_id\": ");
    json_str(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID);
    raw_line(",");
    raw("          \"source_region_id\": ");
    json_str(scratch.region_id);
    raw_line(",");
    raw("          \"target_region_start_lba\": ");
    raw_fmt(format_args!("{}", scratch.region_start_lba));
    raw_line(",");
    raw("          \"target_region_lba_count\": ");
    raw_fmt(format_args!("{}", scratch.region_lba_count));
    raw_line(",");
    raw("          \"target_byte_count\": ");
    raw_fmt(format_args!("{}", scratch.byte_count));
    raw_line(",");
    raw("          \"target_range_scratch_owned\": ");
    raw_bool(scratch.block_write_authority_available);
    raw_line(",");
    raw("          \"target_range_within_device_bounds\": ");
    raw_bool(scratch.region_within_device_bounds);
    raw_line(",");
    raw("          \"target_range_no_boot_or_partition_metadata_overlap\": ");
    raw_bool(scratch.no_boot_or_partition_metadata_overlap);
    raw_line(",");
    raw("          \"target_range_ready\": ");
    raw_bool(scratch_dry_run_ready);
    raw_line(",");
    raw("          \"audit_ledger_target_id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw_line(",");
    raw("          \"audit_record_schema\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw("          \"rollback_store_target_id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw_line(",");
    raw("          \"rollback_transaction_schema\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw_line("          \"dry_run_evaluated\": true,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"writes_durable_audit_log\": false,");
    raw_line("          \"writes_rollback_store\": false,");
    raw_line("          \"appends_rollback_transaction\": false,");
    raw_line("          \"write_attempted\": false");
    raw_line("        },");
    raw_line("        \"target_region_writer_contract\": {");
    raw("          \"schema\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw("          \"status\": ");
    json_str(audit_rollback_target_region_writer_contract_status(
        target_region_contract_ready,
    ));
    raw_line(",");
    raw("          \"reason\": ");
    json_str(audit_rollback_target_region_writer_contract_reason(
        target_region_contract_ready,
    ));
    raw_line(",");
    raw("          \"source_discovery_schema\": ");
    json_str(target_region_discovery.schema);
    raw_line(",");
    raw("          \"source_discovery_id\": ");
    json_str(target_region_discovery.id);
    raw_line(",");
    raw("          \"source_discovery_status\": ");
    json_str(target_region_discovery.status);
    raw_line(",");
    raw("          \"source_discovery_reason\": ");
    json_str(target_region_discovery.reason);
    raw_line(",");
    raw("          \"target_region_present\": ");
    raw_bool(target_region_discovery.candidate_region_present);
    raw_line(",");
    raw("          \"target_region_start_lba\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_start_lba
    ));
    raw_line(",");
    raw("          \"target_region_lba_count\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_lba_count
    ));
    raw_line(",");
    raw("          \"target_byte_count\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_lba_count * ahci::SECTOR_BYTES as u64
    ));
    raw_line(",");
    raw("          \"owner_method\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
    raw_line(",");
    raw("          \"append_target_owner_id\": ");
    json_str(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
    raw_line(",");
    raw("          \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("          \"target_region_is_scratch\": ");
    raw_bool(target_region_discovery.candidate_region_is_scratch);
    raw_line(",");
    raw("          \"target_region_overlaps_boot_metadata\": ");
    raw_bool(target_region_discovery.candidate_overlaps_boot_metadata);
    raw_line(",");
    raw("          \"target_region_overlaps_scratch\": ");
    raw_bool(target_region_discovery.candidate_overlaps_scratch);
    raw_line(",");
    raw("          \"target_range_ready\": ");
    raw_bool(target_region_contract_ready);
    raw_line(",");
    raw("          \"audit_ledger_target_id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw_line(",");
    raw("          \"audit_record_schema\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw("          \"rollback_store_target_id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw_line(",");
    raw("          \"rollback_transaction_schema\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw_line("          \"media_write_policy_preflight\": {");
    raw("            \"schema\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA);
    raw_line(",");
    raw("            \"id\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID);
    raw_line(",");
    raw_line("            \"scope\": \"current_boot\",");
    raw_line("            \"classification\": \"local_only\",");
    raw("            \"status\": ");
    json_str(
        audit_rollback_target_region_media_write_policy_preflight_status(
            target_region_contract_ready,
        ),
    );
    raw_line(",");
    raw("            \"reason\": ");
    json_str(
        audit_rollback_target_region_media_write_policy_preflight_reason(
            target_region_contract_ready,
        ),
    );
    raw_line(",");
    raw("            \"source_contract_schema\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA);
    raw_line(",");
    raw("            \"source_contract_id\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID);
    raw_line(",");
    raw("            \"source_contract_status\": ");
    json_str(audit_rollback_target_region_writer_contract_status(
        target_region_contract_ready,
    ));
    raw_line(",");
    raw("            \"source_contract_reason\": ");
    json_str(audit_rollback_target_region_writer_contract_reason(
        target_region_contract_ready,
    ));
    raw_line(",");
    raw("            \"owner_method\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
    raw_line(",");
    raw("            \"append_target_owner_id\": ");
    json_str(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID);
    raw_line(",");
    raw("            \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("            \"audit_ledger_target_id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw_line(",");
    raw("            \"audit_record_schema\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw("            \"rollback_store_target_id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw_line(",");
    raw("            \"rollback_transaction_schema\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw_line(",");
    raw("            \"target_region_start_lba\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_start_lba
    ));
    raw_line(",");
    raw("            \"target_region_lba_count\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_lba_count
    ));
    raw_line(",");
    raw("            \"target_byte_count\": ");
    raw_fmt(format_args!(
        "{}",
        target_region_discovery.candidate_region_lba_count * ahci::SECTOR_BYTES as u64
    ));
    raw_line(",");
    raw("            \"source_contract_target_range_ready\": ");
    raw_bool(target_region_contract_ready);
    raw_line(",");
    raw_line("            \"owner_ids_verified\": true,");
    raw_line("            \"target_ids_verified\": true,");
    raw("            \"target_span_verified\": ");
    raw_bool(target_region_contract_ready);
    raw_line(",");
    raw_line("            \"schema_ids_verified\": true,");
    raw_line("            \"media_write_authority_required\": true,");
    raw_line("            \"media_write_authority_available\": false,");
    raw_line("            \"media_write_authority_reason\": \"media_write_authority_missing\",");
    raw_line("            \"durable_audit_policy_required\": true,");
    raw_line("            \"durable_audit_policy_available\": false,");
    raw_line("            \"durable_audit_policy_reason\": \"durable_audit_policy_missing\",");
    raw_line("            \"authorizes_media_write\": false,");
    raw_line("            \"authorizes_append\": false,");
    raw_line("            \"writes_durable_audit_log\": false,");
    raw_line("            \"writes_rollback_store\": false,");
    raw_line("            \"appends_rollback_transaction\": false,");
    raw_line("            \"write_attempted\": false");
    raw_line("          },");
    raw_line("          \"write_authority_available\": false,");
    raw_line("          \"durable_audit_policy_available\": false,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"writes_durable_audit_log\": false,");
    raw_line("          \"writes_rollback_store\": false,");
    raw_line("          \"appends_rollback_transaction\": false,");
    raw_line("          \"write_attempted\": false");
    raw_line("        },");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"appends_rollback_transaction\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_append_contract_storage_layout_inputs(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_inputs\": {");
    raw_line("        \"persistence_device_inventory\": {");
    raw_line("          \"schema\": \"raios.persistence_device_inventory.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.persistence_device_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.persistence_device_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.persistence_device_inventory.present);
    raw_line(",");
    raw("          \"storage_controller_observed\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .storage_controller_observed,
    );
    raw_line(",");
    raw("          \"block_driver_available\": ");
    raw_bool(storage.persistence_device_inventory.block_driver_available);
    raw_line(",");
    raw("          \"ahci_register_probe_available\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .ahci_probe
            .registers_mapped,
    );
    raw_line(",");
    raw("          \"block_device_identity_available\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .block_device_identity_available,
    );
    raw_line(",");
    raw("          \"sector_read_available\": ");
    raw_bool(storage.persistence_device_inventory.sector_read_available);
    raw_line(",");
    raw("          \"stable_identity\": ");
    raw_bool(storage.persistence_device_inventory.stable_identity);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .partition_inventory_available,
    );
    raw_line(",");
    raw("          \"write_path_available\": ");
    raw_bool(storage.persistence_device_inventory.write_path_available);
    raw_line(",");
    raw("          \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    raw_line("          \"authorizes_append\": false");
    raw_line("        },");
    raw_line("        \"audit_rollback_storage_layout\": {");
    raw_line("          \"schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.storage_layout_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.storage_layout_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.audit_rollback_storage_layout.present);
    raw_line(",");
    raw("          \"binds_persistence_device\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .binds_persistence_device,
    );
    raw_line(",");
    raw("          \"has_audit_ledger_region\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .has_audit_ledger_region,
    );
    raw_line(",");
    raw("          \"has_rollback_store_region\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .has_rollback_store_region,
    );
    raw_line(",");
    raw("          \"append_slots_available\": ");
    raw_bool(storage.audit_rollback_storage_layout.append_slots_available);
    raw_line(",");
    raw("          \"authorizes_append\": false");
    crlf();
    raw_line("        },");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"block_write_path_available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("        \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    emit_module_block_write_path_authority_gate(storage.persistence_device_inventory, evaluation);
    raw_line(",");
    raw_line("        \"storage_layout_facts_are_append_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_append_contract_append_engine_inputs(
    engine: ModuleAuditRollbackAppendEngineCandidate,
    evaluation: ModuleAuditRollbackAppendEngineEvaluation,
) {
    raw_line("      \"append_engine_inputs\": {");
    raw_line("        \"audit_ledger_append_engine\": {");
    raw_line("          \"schema\": \"raios.audit_ledger_append_engine.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.audit_engine_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.audit_engine_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(engine.audit_ledger_append_engine.present);
    raw_line(",");
    raw("          \"binds_storage_layout\": ");
    raw_bool(engine.audit_ledger_append_engine.binds_storage_layout);
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(engine.audit_ledger_append_engine.binds_write_policy);
    raw_line(",");
    raw("          \"supports_append_only\": ");
    raw_bool(engine.audit_ledger_append_engine.supports_append_only);
    raw_line(",");
    raw_line("          \"authorizes_append_contract\": false");
    raw_line("        },");
    raw_line("        \"rollback_store_transaction_engine\": {");
    raw_line("          \"schema\": \"raios.rollback_store_transaction_engine.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.rollback_engine_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.rollback_engine_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(engine.rollback_store_transaction_engine.present);
    raw_line(",");
    raw("          \"binds_storage_layout\": ");
    raw_bool(
        engine
            .rollback_store_transaction_engine
            .binds_storage_layout,
    );
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(engine.rollback_store_transaction_engine.binds_write_policy);
    raw_line(",");
    raw("          \"supports_replay\": ");
    raw_bool(engine.rollback_store_transaction_engine.supports_replay);
    raw_line(",");
    raw_line("          \"authorizes_append_contract\": false");
    raw_line("        },");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"append_engine_facts_are_append_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_append_contract_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    storage_layout_id: &'static str,
    append_engine_id: &'static str,
    write_policy_id: &'static str,
    availability_id: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"target_schema\": ");
    json_str(target_schema);
    raw_line(",");
    raw_line("          \"storage_layout_schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw("          \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("          \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("          \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("          \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("          \"append_contract_provenance_schema\": \"raios.append_contract_envelope_provenance.v0\",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(fact.binds_write_policy);
    raw_line(",");
    raw("          \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw_line(",");
    raw("          \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw_line(",");
    raw("          \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw_line(",");
    raw("          \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw_line(",");
    raw("          \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw_line(",");
    raw("          \"binds_envelope_provenance\": ");
    raw_bool(fact.binds_envelope_provenance);
    raw_line(",");
    raw("          \"storage_layout_available\": ");
    raw_bool(fact.storage_layout_available);
    raw_line(",");
    raw("          \"append_engine_available\": ");
    raw_bool(fact.append_engine_available);
    raw_line(",");
    raw_line("          \"required_bindings\": {");
    raw("            \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("            \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("            \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("            \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("            \"provenance_schema\": \"raios.append_contract_envelope_provenance.v0\"");
    raw_line("          },");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_contract\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_append_contract_selftest_cases(
) -> [ModuleAuditRollbackAppendContractSelfTestCase;
       MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_SELFTEST_CASES] {
    let missing = module_audit_rollback_append_contract_snapshot();
    let available = module_audit_rollback_available_append_contract_fact();
    [
        module_audit_rollback_append_contract_selftest_case(
            "missing_append_envelope_pair_current_boot",
            "missing",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            missing,
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_previous_boot",
            "rejected",
            "audit_append_envelope_scope_must_be_current_boot",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_wrong_schema",
            "rejected",
            "audit_append_envelope_schema_mismatch",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_provenance_missing",
            "rejected",
            "audit_append_envelope_provenance_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_provenance_binding_missing",
            "rejected",
            "audit_append_envelope_provenance_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_envelope_provenance: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_policy_binding_missing",
            "rejected",
            "audit_append_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_write_policy_id_missing",
            "rejected",
            "audit_append_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_availability_binding_missing",
            "rejected",
            "audit_append_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_availability_id_missing",
            "rejected",
            "audit_append_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_storage_layout_id_missing",
            "rejected",
            "audit_append_envelope_storage_layout_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_storage_layout_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_append_engine_id_missing",
            "rejected",
            "audit_append_envelope_append_engine_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_append_engine_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_ledger_storage_layout_missing",
            "missing",
            "audit_ledger_storage_layout_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    storage_layout_available: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_previous_boot",
            "rejected",
            "rollback_transaction_envelope_scope_must_be_current_boot",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_wrong_schema",
            "rejected",
            "rollback_transaction_envelope_schema_mismatch",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_provenance_missing",
            "rejected",
            "rollback_transaction_envelope_provenance_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_provenance_binding_missing",
            "rejected",
            "rollback_transaction_envelope_provenance_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_envelope_provenance: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_policy_binding_missing",
            "rejected",
            "rollback_transaction_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_write_policy_id_missing",
            "rejected",
            "rollback_transaction_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_availability_binding_missing",
            "rejected",
            "rollback_transaction_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_availability_id_missing",
            "rejected",
            "rollback_transaction_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_storage_layout_id_missing",
            "rejected",
            "rollback_transaction_envelope_storage_layout_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_storage_layout_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_append_engine_id_missing",
            "rejected",
            "rollback_transaction_envelope_append_engine_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_append_engine_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_store_storage_layout_missing",
            "missing",
            "rollback_store_storage_layout_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    storage_layout_available: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "available_envelopes_append_engine_still_missing",
            "missing",
            "audit_ledger_append_engine_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    append_engine_available: false,
                    ..available
                },
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    append_engine_available: false,
                    ..available
                },
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_append_contract_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendContractCandidate,
) -> ModuleAuditRollbackAppendContractSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_contract_candidate(candidate);
    ModuleAuditRollbackAppendContractSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.writes_enabled
            && !actual.installs_rollback_plan
            && !actual.can_load
            && !actual.load_attempted,
    }
}
