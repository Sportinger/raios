use alloc::vec;

use crate::agent_protocol_support::{
    emit_record_fields_trailing_comma, emit_record_property_line, record_bool as b,
    record_false as no, record_field as f, record_null as null, record_object as object,
    record_str as s,
};
use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_engine::*,
    agent_protocol_module_write_boundary_storage_layout::*, agent_protocol_support::*, ahci,
};
use raios_core::record::Value as V;

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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_append_contract.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", no()),
            f("global_event_log_mutation", s("none")),
            f("writes_enabled", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        6,
    );
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
    emit_record_property_line(
        "policy_result",
        vec![
            f(
                "append_target_owner_status",
                s(evaluation.append_target_owner_status),
            ),
            f(
                "append_target_owner_reason",
                s(evaluation.append_target_owner_reason),
            ),
            f(
                "append_target_owner_available",
                b(evaluation.append_target_owner_available),
            ),
            f(
                "transaction_writer_status",
                s(evaluation.transaction_writer_status),
            ),
            f(
                "transaction_writer_reason",
                s(evaluation.transaction_writer_reason),
            ),
            f(
                "transaction_writer_ready",
                b(evaluation.transaction_writer_ready),
            ),
            f(
                "block_write_path_available",
                b(evaluation.block_write_path_available),
            ),
            f(
                "block_write_path_reason",
                s(evaluation.block_write_path_reason),
            ),
            f("append_contract_status", s(evaluation.status)),
            f("append_contract_reason", s(evaluation.reason)),
            f(
                "audit_append_missing",
                b(!method_eq(evaluation.audit_append_status, "available")),
            ),
            f(
                "rollback_transaction_missing",
                b(!method_eq(
                    evaluation.rollback_transaction_status,
                    "available",
                )),
            ),
            f(
                "storage_layout_available",
                b(evaluation.storage_layout_available),
            ),
            f(
                "append_engine_available",
                b(evaluation.append_engine_available),
            ),
            f(
                "storage_layout_missing",
                b(!evaluation.storage_layout_available),
            ),
            f(
                "append_engine_missing",
                b(!evaluation.append_engine_available),
            ),
            f("append_envelopes_are_durable_authority", no()),
            f("retained_hash_refs_are_append_authority", no()),
            f("policy_facts_are_append_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_append_contract_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
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
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("passed", b(case.passed)),
            f("writes_enabled", no()),
            f("installs_rollback_plan", no()),
            f("can_load", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
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
    emit_record_property_line(
        "append_target_owner",
        vec![
            f("schema", s(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_SCHEMA)),
            f("id", s(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID)),
            f(
                "owner_method",
                s(MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_METHOD),
            ),
            f(
                "storage_authority_id",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID),
            ),
            f(
                "storage_authority_owner",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER),
            ),
            f("status", s(evaluation.append_target_owner_status)),
            f("reason", s(evaluation.append_target_owner_reason)),
            f("available", b(evaluation.append_target_owner_available)),
            f(
                "block_write_path_available",
                b(evaluation.block_write_path_available),
            ),
            f(
                "block_write_path_reason",
                s(evaluation.block_write_path_reason),
            ),
            f("authorizes_append", no()),
            f("writes_enabled", no()),
            f(
                "targets",
                object(vec![
                    f(
                        "audit_ledger",
                        inline_target_owner_record(
                            AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID,
                            AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA,
                            evaluation.append_target_owner_available,
                        ),
                    ),
                    f(
                        "rollback_store",
                        inline_target_owner_record(
                            AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID,
                            AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA,
                            evaluation.append_target_owner_available,
                        ),
                    ),
                ]),
            ),
        ],
        false,
    );
}

fn inline_target_owner_record(
    id: &'static str,
    schema: &'static str,
    owner_bound: bool,
) -> V<'static> {
    record_inline(vec![
        f("id", s(id)),
        f("schema", s(schema)),
        f("owner_bound", b(owner_bound)),
        f("available", no()),
    ])
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
    let target_byte_count =
        target_region_discovery.candidate_region_lba_count * ahci::SECTOR_BYTES as u64;
    let target_contract_status =
        audit_rollback_target_region_writer_contract_status(target_region_contract_ready);
    let target_contract_reason =
        audit_rollback_target_region_writer_contract_reason(target_region_contract_ready);

    emit_record_property_line(
        "transaction_writer_readiness",
        vec![
            f(
                "schema",
                s(AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_SCHEMA),
            ),
            f("id", s(AUDIT_ROLLBACK_TRANSACTION_WRITER_READINESS_ID)),
            f("owner_method", s(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
            f(
                "append_target_owner_id",
                s(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID),
            ),
            f(
                "storage_authority_id",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID),
            ),
            f("status", s(evaluation.transaction_writer_status)),
            f("reason", s(evaluation.transaction_writer_reason)),
            f("ready", b(evaluation.transaction_writer_ready)),
            f(
                "block_write_path_available",
                b(evaluation.block_write_path_available),
            ),
            f(
                "block_write_path_reason",
                s(evaluation.block_write_path_reason),
            ),
            f(
                "scratch_only_writer_dry_run",
                object(vec![
                    f(
                        "schema",
                        s(AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_SCHEMA),
                    ),
                    f(
                        "id",
                        s(AUDIT_ROLLBACK_TRANSACTION_WRITER_SCRATCH_DRY_RUN_ID),
                    ),
                    f("scope", s("current_boot")),
                    f("classification", s("local_only")),
                    f(
                        "status",
                        s(audit_rollback_scratch_writer_dry_run_status(
                            scratch_dry_run_ready,
                        )),
                    ),
                    f(
                        "reason",
                        s(audit_rollback_scratch_writer_dry_run_reason(
                            scratch_dry_run_ready,
                        )),
                    ),
                    f(
                        "source_authority_id",
                        s(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID),
                    ),
                    f("source_region_id", s(scratch.region_id)),
                    f("target_region_start_lba", V::U64(scratch.region_start_lba)),
                    f("target_region_lba_count", V::U64(scratch.region_lba_count)),
                    f("target_byte_count", V::U64(scratch.byte_count as u64)),
                    f(
                        "target_range_scratch_owned",
                        b(scratch.block_write_authority_available),
                    ),
                    f(
                        "target_range_within_device_bounds",
                        b(scratch.region_within_device_bounds),
                    ),
                    f(
                        "target_range_no_boot_or_partition_metadata_overlap",
                        b(scratch.no_boot_or_partition_metadata_overlap),
                    ),
                    f("target_range_ready", b(scratch_dry_run_ready)),
                    f(
                        "audit_ledger_target_id",
                        s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID),
                    ),
                    f(
                        "audit_record_schema",
                        s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA),
                    ),
                    f(
                        "rollback_store_target_id",
                        s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID),
                    ),
                    f(
                        "rollback_transaction_schema",
                        s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA),
                    ),
                    f("dry_run_evaluated", b(true)),
                    f("authorizes_append", no()),
                    f("writes_durable_audit_log", no()),
                    f("writes_rollback_store", no()),
                    f("appends_rollback_transaction", no()),
                    f("write_attempted", no()),
                ]),
            ),
            f(
                "target_region_writer_contract",
                object(vec![
                    f(
                        "schema",
                        s(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA),
                    ),
                    f("id", s(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID)),
                    f("scope", s("current_boot")),
                    f("classification", s("local_only")),
                    f("status", s(target_contract_status)),
                    f("reason", s(target_contract_reason)),
                    f("source_discovery_schema", s(target_region_discovery.schema)),
                    f("source_discovery_id", s(target_region_discovery.id)),
                    f("source_discovery_status", s(target_region_discovery.status)),
                    f("source_discovery_reason", s(target_region_discovery.reason)),
                    f(
                        "target_region_present",
                        b(target_region_discovery.candidate_region_present),
                    ),
                    f(
                        "target_region_start_lba",
                        V::U64(target_region_discovery.candidate_region_start_lba),
                    ),
                    f(
                        "target_region_lba_count",
                        V::U64(target_region_discovery.candidate_region_lba_count),
                    ),
                    f("target_byte_count", V::U64(target_byte_count)),
                    f("owner_method", s(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
                    f(
                        "append_target_owner_id",
                        s(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID),
                    ),
                    f(
                        "storage_authority_id",
                        s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID),
                    ),
                    f(
                        "target_region_is_scratch",
                        b(target_region_discovery.candidate_region_is_scratch),
                    ),
                    f(
                        "target_region_overlaps_boot_metadata",
                        b(target_region_discovery.candidate_overlaps_boot_metadata),
                    ),
                    f(
                        "target_region_overlaps_scratch",
                        b(target_region_discovery.candidate_overlaps_scratch),
                    ),
                    f("target_range_ready", b(target_region_contract_ready)),
                    f(
                        "audit_ledger_target_id",
                        s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID),
                    ),
                    f(
                        "audit_record_schema",
                        s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA),
                    ),
                    f(
                        "rollback_store_target_id",
                        s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID),
                    ),
                    f(
                        "rollback_transaction_schema",
                        s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA),
                    ),
                    f(
                        "media_write_policy_preflight",
                        object(vec![
                            f(
                                "schema",
                                s(AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_SCHEMA),
                            ),
                            f(
                                "id",
                                s(AUDIT_ROLLBACK_TARGET_REGION_MEDIA_WRITE_POLICY_PREFLIGHT_ID),
                            ),
                            f("scope", s("current_boot")),
                            f("classification", s("local_only")),
                            f(
                                "status",
                                s(audit_rollback_target_region_media_write_policy_preflight_status(
                                    target_region_contract_ready,
                                )),
                            ),
                            f(
                                "reason",
                                s(audit_rollback_target_region_media_write_policy_preflight_reason(
                                    target_region_contract_ready,
                                )),
                            ),
                            f(
                                "source_contract_schema",
                                s(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_SCHEMA),
                            ),
                            f(
                                "source_contract_id",
                                s(AUDIT_ROLLBACK_TARGET_REGION_WRITER_CONTRACT_ID),
                            ),
                            f("source_contract_status", s(target_contract_status)),
                            f("source_contract_reason", s(target_contract_reason)),
                            f("owner_method", s(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER)),
                            f(
                                "append_target_owner_id",
                                s(AUDIT_ROLLBACK_APPEND_TARGET_OWNER_ID),
                            ),
                            f("storage_authority_id", s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
                            f("audit_ledger_target_id", s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
                            f(
                                "audit_record_schema",
                                s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA),
                            ),
                            f(
                                "rollback_store_target_id",
                                s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID),
                            ),
                            f(
                                "rollback_transaction_schema",
                                s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA),
                            ),
                            f(
                                "target_region_start_lba",
                                V::U64(target_region_discovery.candidate_region_start_lba),
                            ),
                            f(
                                "target_region_lba_count",
                                V::U64(target_region_discovery.candidate_region_lba_count),
                            ),
                            f("target_byte_count", V::U64(target_byte_count)),
                            f(
                                "source_contract_target_range_ready",
                                b(target_region_contract_ready),
                            ),
                            f("owner_ids_verified", b(true)),
                            f("target_ids_verified", b(true)),
                            f("target_span_verified", b(target_region_contract_ready)),
                            f("schema_ids_verified", b(true)),
                            f("media_write_authority_required", b(true)),
                            f("media_write_authority_available", no()),
                            f("media_write_authority_reason", s("media_write_authority_missing")),
                            f("durable_audit_policy_required", b(true)),
                            f("durable_audit_policy_available", no()),
                            f("durable_audit_policy_reason", s("durable_audit_policy_missing")),
                            f("authorizes_media_write", no()),
                            f("authorizes_append", no()),
                            f("writes_durable_audit_log", no()),
                            f("writes_rollback_store", no()),
                            f("appends_rollback_transaction", no()),
                            f("write_attempted", no()),
                        ]),
                    ),
                    f("write_authority_available", no()),
                    f("durable_audit_policy_available", no()),
                    f("authorizes_append", no()),
                    f("writes_durable_audit_log", no()),
                    f("writes_rollback_store", no()),
                    f("appends_rollback_transaction", no()),
                    f("write_attempted", no()),
                ]),
            ),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("appends_rollback_transaction", no()),
        ],
        false,
    );
}

pub(crate) fn emit_module_append_contract_storage_layout_inputs(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_inputs\": {");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "persistence_device_inventory",
                object(vec![
                    f("schema", s("raios.persistence_device_inventory.v0")),
                    f("status", s(evaluation.persistence_device_status)),
                    f("reason", s(evaluation.persistence_device_reason)),
                    f("present", b(storage.persistence_device_inventory.present)),
                    f(
                        "storage_controller_observed",
                        b(storage
                            .persistence_device_inventory
                            .storage_controller_observed),
                    ),
                    f(
                        "block_driver_available",
                        b(storage.persistence_device_inventory.block_driver_available),
                    ),
                    f(
                        "ahci_register_probe_available",
                        b(storage
                            .persistence_device_inventory
                            .ahci_probe
                            .registers_mapped),
                    ),
                    f(
                        "block_device_identity_available",
                        b(storage
                            .persistence_device_inventory
                            .block_device_identity_available),
                    ),
                    f(
                        "sector_read_available",
                        b(storage.persistence_device_inventory.sector_read_available),
                    ),
                    f(
                        "stable_identity",
                        b(storage.persistence_device_inventory.stable_identity),
                    ),
                    f(
                        "partition_inventory_available",
                        b(storage
                            .persistence_device_inventory
                            .partition_inventory_available),
                    ),
                    f(
                        "write_path_available",
                        b(storage.persistence_device_inventory.write_path_available),
                    ),
                    f(
                        "block_write_path_reason",
                        s(evaluation.block_write_path_reason),
                    ),
                    f("authorizes_append", no()),
                ]),
            ),
            f(
                "audit_rollback_storage_layout",
                object(vec![
                    f("schema", s("raios.audit_rollback_storage_layout.v0")),
                    f("status", s(evaluation.storage_layout_status)),
                    f("reason", s(evaluation.storage_layout_reason)),
                    f("present", b(storage.audit_rollback_storage_layout.present)),
                    f(
                        "binds_persistence_device",
                        b(storage
                            .audit_rollback_storage_layout
                            .binds_persistence_device),
                    ),
                    f(
                        "has_audit_ledger_region",
                        b(storage
                            .audit_rollback_storage_layout
                            .has_audit_ledger_region),
                    ),
                    f(
                        "has_rollback_store_region",
                        b(storage
                            .audit_rollback_storage_layout
                            .has_rollback_store_region),
                    ),
                    f(
                        "append_slots_available",
                        b(storage.audit_rollback_storage_layout.append_slots_available),
                    ),
                    f("authorizes_append", no()),
                ]),
            ),
            f(
                "storage_layout_available",
                b(evaluation.storage_layout_available),
            ),
            f(
                "block_write_path_available",
                b(evaluation.block_write_path_available),
            ),
            f(
                "block_write_path_reason",
                s(evaluation.block_write_path_reason),
            ),
        ],
        8,
    );
    emit_module_block_write_path_authority_gate(storage.persistence_device_inventory, evaluation);
    raw_line(",");
    emit_record_fields(
        vec![f("storage_layout_facts_are_append_authority", no())],
        8,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_append_contract_append_engine_inputs(
    engine: ModuleAuditRollbackAppendEngineCandidate,
    evaluation: ModuleAuditRollbackAppendEngineEvaluation,
) {
    emit_record_property_line(
        "append_engine_inputs",
        vec![
            f(
                "audit_ledger_append_engine",
                object(vec![
                    f("schema", s("raios.audit_ledger_append_engine.v0")),
                    f("status", s(evaluation.audit_engine_status)),
                    f("reason", s(evaluation.audit_engine_reason)),
                    f("present", b(engine.audit_ledger_append_engine.present)),
                    f(
                        "binds_storage_layout",
                        b(engine.audit_ledger_append_engine.binds_storage_layout),
                    ),
                    f(
                        "binds_write_policy",
                        b(engine.audit_ledger_append_engine.binds_write_policy),
                    ),
                    f(
                        "supports_append_only",
                        b(engine.audit_ledger_append_engine.supports_append_only),
                    ),
                    f("authorizes_append_contract", no()),
                ]),
            ),
            f(
                "rollback_store_transaction_engine",
                object(vec![
                    f("schema", s("raios.rollback_store_transaction_engine.v0")),
                    f("status", s(evaluation.rollback_engine_status)),
                    f("reason", s(evaluation.rollback_engine_reason)),
                    f(
                        "present",
                        b(engine.rollback_store_transaction_engine.present),
                    ),
                    f(
                        "binds_storage_layout",
                        b(engine
                            .rollback_store_transaction_engine
                            .binds_storage_layout),
                    ),
                    f(
                        "binds_write_policy",
                        b(engine.rollback_store_transaction_engine.binds_write_policy),
                    ),
                    f(
                        "supports_replay",
                        b(engine.rollback_store_transaction_engine.supports_replay),
                    ),
                    f("authorizes_append_contract", no()),
                ]),
            ),
            f(
                "append_engine_available",
                b(evaluation.append_engine_available),
            ),
            f("append_engine_facts_are_append_authority", no()),
        ],
        false,
    );
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
    emit_record_property_line_at(
        name,
        vec![
            f("schema", s(schema)),
            f("id", s(id)),
            f("target_schema", s(target_schema)),
            f(
                "storage_layout_schema",
                s("raios.audit_rollback_storage_layout.v0"),
            ),
            f("storage_layout_id", s(storage_layout_id)),
            f("append_engine_id", s(append_engine_id)),
            f("write_policy_id", s(write_policy_id)),
            f("availability_id", s(availability_id)),
            f(
                "append_contract_provenance_schema",
                s("raios.append_contract_envelope_provenance.v0"),
            ),
            f("scope", s(fact.scope)),
            f("classification", s(fact.classification)),
            f("status", s(status)),
            f("reason", s(reason)),
            f("present", b(fact.present)),
            f("schema_valid", b(fact.schema_ok)),
            f("provenance_valid", b(fact.provenance_ok)),
            f("binds_write_policy", b(fact.binds_write_policy)),
            f("binds_availability", b(fact.binds_availability)),
            f("binds_storage_layout_id", b(fact.binds_storage_layout_id)),
            f("binds_append_engine_id", b(fact.binds_append_engine_id)),
            f("binds_write_policy_id", b(fact.binds_write_policy_id)),
            f("binds_availability_id", b(fact.binds_availability_id)),
            f(
                "binds_envelope_provenance",
                b(fact.binds_envelope_provenance),
            ),
            f("storage_layout_available", b(fact.storage_layout_available)),
            f("append_engine_available", b(fact.append_engine_available)),
            f(
                "required_bindings",
                object(vec![
                    f("storage_layout_id", s(storage_layout_id)),
                    f("append_engine_id", s(append_engine_id)),
                    f("write_policy_id", s(write_policy_id)),
                    f("availability_id", s(availability_id)),
                    f(
                        "provenance_schema",
                        s("raios.append_contract_envelope_provenance.v0"),
                    ),
                ]),
            ),
            f("authority", s("current_snapshot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("write_attempted", no()),
            f(
                "provenance",
                object(vec![
                    f("source_method", s("module.audit_rollback_append_contract")),
                    f("source_transport", s("serial-console")),
                    f("event_scope", s("current_boot")),
                    f("record_id", null()),
                ]),
            ),
        ],
        8,
        comma,
    );
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
