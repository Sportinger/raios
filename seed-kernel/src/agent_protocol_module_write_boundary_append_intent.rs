use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_payload_hash::*, agent_protocol_support::*,
    event_log,
};

pub(crate) fn emit_module_audit_rollback_append_intent() {
    let binding = event_log::module_load_gate_binding_snapshot();
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let payload =
        module_audit_rollback_append_payload_hash_snapshot_from_binding_and_append_contract(
            binding,
            append_evaluation,
        );
    let payload_evaluation = evaluate_module_audit_rollback_append_payload_hash_candidate(payload);
    let intent = module_audit_rollback_append_intent_snapshot_from_append_contract_and_payload(
        append_evaluation,
        payload_evaluation,
    );
    let evaluation = evaluate_module_audit_rollback_append_intent_candidate(intent);

    begin_response("module.audit_rollback_append_intent");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_intent.v0\",");
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
    emit_module_append_intent_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_append_intent_payload_hash_inputs(payload, payload_evaluation);
    raw_line(",");
    emit_module_append_intent_facts(intent, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"append_intent_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"append_intent_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"audit_intent_missing\": ");
    raw_bool(!method_eq(evaluation.audit_intent_status, "available"));
    raw_line(",");
    raw("        \"rollback_intent_missing\": ");
    raw_bool(!method_eq(evaluation.rollback_intent_status, "available"));
    raw_line(",");
    raw("        \"append_contract_available\": ");
    raw_bool(evaluation.append_contract_available);
    raw_line(",");
    raw("        \"append_contract_missing\": ");
    raw_bool(!evaluation.append_contract_available);
    raw_line(",");
    raw("        \"payload_hash_available\": ");
    raw_bool(evaluation.payload_hash_available);
    raw_line(",");
    raw("        \"payload_hash_missing\": ");
    raw_bool(!evaluation.payload_hash_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(evaluation.append_intent_available);
    raw_line(",");
    raw("        \"append_intent_missing\": ");
    raw_bool(!evaluation.append_intent_available);
    raw_line(",");
    raw_line("        \"append_intent_facts_are_writer_authority\": false,");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_intent_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    let append_contract_available = evaluation.append_contract_available;
    emit_export_gate(
        &mut wrote,
        "append_contract",
        if append_contract_available {
            "available"
        } else {
            append_evaluation.status
        },
        if append_contract_available {
            "append_contract_available"
        } else {
            append_evaluation.reason
        },
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_payload_hash",
        payload_evaluation.audit_payload_status,
        payload_evaluation.audit_payload_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_payload_hash",
        payload_evaluation.rollback_payload_status,
        payload_evaluation.rollback_payload_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_intent",
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_intent",
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_intent");
}

pub(crate) fn emit_module_audit_rollback_append_intent_selftest() {
    let cases = module_audit_rollback_append_intent_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_append_intent_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_intent_selftest.v0\",");
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
        emit_module_audit_rollback_append_intent_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_intent_selftest");
}

pub(crate) fn emit_module_audit_rollback_append_intent_selftest_case(
    case: &ModuleAuditRollbackAppendIntentSelfTestCase,
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

pub(crate) fn module_audit_rollback_append_intent_snapshot(
) -> ModuleAuditRollbackAppendIntentCandidate {
    let binding = event_log::module_load_gate_binding_snapshot();
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let payload =
        module_audit_rollback_append_payload_hash_snapshot_from_binding_and_append_contract(
            binding,
            append_evaluation,
        );
    let payload_evaluation = evaluate_module_audit_rollback_append_payload_hash_candidate(payload);
    module_audit_rollback_append_intent_snapshot_from_append_contract_and_payload(
        append_evaluation,
        payload_evaluation,
    )
}

pub(crate) fn module_audit_rollback_append_intent_snapshot_from_append_contract_and_payload(
    append: ModuleAuditRollbackAppendContractEvaluation,
    payload: ModuleAuditRollbackAppendPayloadHashEvaluation,
) -> ModuleAuditRollbackAppendIntentCandidate {
    ModuleAuditRollbackAppendIntentCandidate {
        audit_record_append_intent: module_audit_rollback_missing_append_intent_fact(
            method_eq(append.audit_append_status, "available"),
            method_eq(payload.audit_payload_status, "available"),
        ),
        rollback_transaction_append_intent: module_audit_rollback_missing_append_intent_fact(
            method_eq(append.rollback_transaction_status, "available"),
            method_eq(payload.rollback_payload_status, "available"),
        ),
    }
}

pub(crate) fn module_audit_rollback_missing_append_intent_fact(
    append_contract_available: bool,
    payload_hash_available: bool,
) -> ModuleAuditRollbackAppendIntentFact {
    ModuleAuditRollbackAppendIntentFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_append_contract: false,
        binds_append_contract_id: false,
        binds_append_engine_id: false,
        binds_storage_layout_id: false,
        binds_write_policy_id: false,
        binds_availability_id: false,
        binds_payload_hash: false,
        binds_intent_provenance: false,
        append_contract_available,
        payload_hash_available,
    }
}

pub(crate) fn module_audit_rollback_available_append_intent_fact(
) -> ModuleAuditRollbackAppendIntentFact {
    ModuleAuditRollbackAppendIntentFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_append_contract: true,
        binds_append_contract_id: true,
        binds_append_engine_id: true,
        binds_storage_layout_id: true,
        binds_write_policy_id: true,
        binds_availability_id: true,
        binds_payload_hash: true,
        binds_intent_provenance: true,
        append_contract_available: true,
        payload_hash_available: true,
    }
}

pub(crate) fn evaluate_module_audit_rollback_append_intent_candidate(
    candidate: ModuleAuditRollbackAppendIntentCandidate,
) -> ModuleAuditRollbackAppendIntentEvaluation {
    let (audit_intent_status, audit_intent_reason) = evaluate_module_append_intent_fact(
        candidate.audit_record_append_intent,
        "audit_record_append_intent_scope_must_be_current_boot",
        "audit_record_append_intent_schema_mismatch",
        "audit_record_append_intent_missing",
        "audit_record_append_intent_provenance_missing",
        "audit_record_append_intent_provenance_binding_missing",
        "audit_record_append_intent_append_contract_binding_missing",
        "audit_record_append_intent_append_engine_binding_missing",
        "audit_record_append_intent_storage_layout_binding_missing",
        "audit_record_append_intent_write_policy_binding_missing",
        "audit_record_append_intent_availability_binding_missing",
        "audit_record_append_intent_payload_hash_missing",
        "audit_record_append_contract_missing",
        "audit_record_append_payload_hash_envelope_missing",
        "audit_record_append_intent_available",
    );
    let (rollback_intent_status, rollback_intent_reason) = evaluate_module_append_intent_fact(
        candidate.rollback_transaction_append_intent,
        "rollback_transaction_append_intent_scope_must_be_current_boot",
        "rollback_transaction_append_intent_schema_mismatch",
        "rollback_transaction_append_intent_missing",
        "rollback_transaction_append_intent_provenance_missing",
        "rollback_transaction_append_intent_provenance_binding_missing",
        "rollback_transaction_append_intent_append_contract_binding_missing",
        "rollback_transaction_append_intent_append_engine_binding_missing",
        "rollback_transaction_append_intent_storage_layout_binding_missing",
        "rollback_transaction_append_intent_write_policy_binding_missing",
        "rollback_transaction_append_intent_availability_binding_missing",
        "rollback_transaction_append_intent_payload_hash_missing",
        "rollback_transaction_append_contract_missing",
        "rollback_transaction_append_payload_hash_envelope_missing",
        "rollback_transaction_append_intent_available",
    );

    let (status, reason) = if method_eq(audit_intent_status, "rejected") {
        ("rejected", audit_intent_reason)
    } else if method_eq(rollback_intent_status, "rejected") {
        ("rejected", rollback_intent_reason)
    } else if method_eq(audit_intent_status, "missing")
        && method_eq(audit_intent_reason, "audit_record_append_intent_missing")
        && method_eq(rollback_intent_status, "missing")
        && method_eq(
            rollback_intent_reason,
            "rollback_transaction_append_intent_missing",
        )
    {
        (
            "missing",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
        )
    } else if method_eq(audit_intent_status, "missing") {
        ("missing", audit_intent_reason)
    } else if method_eq(rollback_intent_status, "missing") {
        ("missing", rollback_intent_reason)
    } else {
        ("available", "audit_rollback_append_intent_available")
    };

    let append_contract_available = candidate
        .audit_record_append_intent
        .append_contract_available
        && candidate
            .rollback_transaction_append_intent
            .append_contract_available;
    let payload_hash_available = candidate.audit_record_append_intent.payload_hash_available
        && candidate
            .rollback_transaction_append_intent
            .payload_hash_available;
    let append_intent_available = method_eq(audit_intent_status, "available")
        && method_eq(rollback_intent_status, "available");
    ModuleAuditRollbackAppendIntentEvaluation {
        status,
        reason,
        audit_intent_status,
        audit_intent_reason,
        rollback_intent_status,
        rollback_intent_reason,
        append_contract_available,
        payload_hash_available,
        append_intent_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_append_intent_fact(
    fact: ModuleAuditRollbackAppendIntentFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    provenance_binding_reason: &'static str,
    append_contract_binding_reason: &'static str,
    append_engine_binding_reason: &'static str,
    storage_layout_binding_reason: &'static str,
    write_policy_binding_reason: &'static str,
    availability_binding_reason: &'static str,
    payload_hash_reason: &'static str,
    append_contract_reason: &'static str,
    payload_hash_envelope_reason: &'static str,
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
    if !fact.binds_intent_provenance {
        return ("rejected", provenance_binding_reason);
    }
    if !fact.binds_append_contract || !fact.binds_append_contract_id {
        return ("rejected", append_contract_binding_reason);
    }
    if !fact.binds_append_engine_id {
        return ("rejected", append_engine_binding_reason);
    }
    if !fact.binds_storage_layout_id {
        return ("rejected", storage_layout_binding_reason);
    }
    if !fact.binds_write_policy_id {
        return ("rejected", write_policy_binding_reason);
    }
    if !fact.binds_availability_id {
        return ("rejected", availability_binding_reason);
    }
    if !fact.binds_payload_hash {
        return ("rejected", payload_hash_reason);
    }
    if !fact.append_contract_available {
        return ("missing", append_contract_reason);
    }
    if !fact.payload_hash_available {
        return ("missing", payload_hash_envelope_reason);
    }
    ("available", available_reason)
}

pub(crate) fn emit_module_append_intent_append_contract_inputs(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_inputs\": {");
    emit_module_append_intent_append_contract_input(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_append_intent_append_contract_input(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        true,
    );
    raw("        \"append_contract_available\": ");
    raw_bool(
        method_eq(evaluation.audit_append_status, "available")
            && method_eq(evaluation.rollback_transaction_status, "available"),
    );
    raw_line(",");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_append_intent_append_contract_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw(", \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw(", \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw(", \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw(", \"binds_envelope_provenance\": ");
    raw_bool(fact.binds_envelope_provenance);
    raw(", \"authorizes_append_intent\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_append_intent_payload_hash_inputs(
    payload: ModuleAuditRollbackAppendPayloadHashCandidate,
    evaluation: ModuleAuditRollbackAppendPayloadHashEvaluation,
) {
    raw_line("      \"append_payload_hash_inputs\": {");
    emit_module_append_intent_payload_hash_input(
        "audit_record_append_payload_hash",
        "raios.audit_record_append_payload_hash_envelope.v0",
        payload.audit_record_payload_hash,
        evaluation.audit_payload_status,
        evaluation.audit_payload_reason,
        true,
    );
    emit_module_append_intent_payload_hash_input(
        "rollback_transaction_append_payload_hash",
        "raios.rollback_transaction_append_payload_hash_envelope.v0",
        payload.rollback_transaction_payload_hash,
        evaluation.rollback_payload_status,
        evaluation.rollback_payload_reason,
        true,
    );
    raw("        \"payload_hash_available\": ");
    raw_bool(evaluation.payload_hash_available);
    raw_line(",");
    raw_line("        \"payload_hash_envelopes_are_append_intent_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_append_intent_payload_hash_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendPayloadHashFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"payload_hash\": ");
    json_sha256_option(fact.payload_hash);
    raw(", \"source_payload_hash\": ");
    json_sha256_option(fact.source_payload_hash);
    raw(", \"binds_retained_audit_rollback\": ");
    raw_bool(fact.binds_retained_audit_rollback);
    raw(", \"binds_service_slot_reservation\": ");
    raw_bool(fact.binds_service_slot_reservation);
    raw(", \"binds_pre_load_write_request\": ");
    raw_bool(fact.binds_pre_load_write_request);
    raw(", \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw(", \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw(", \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw(", \"authorizes_append_intent\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_append_intent_facts(
    intent: ModuleAuditRollbackAppendIntentCandidate,
    evaluation: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"append_intent_facts\": {");
    emit_module_append_intent_fact(
        "audit_record_append_intent",
        "raios.audit_record_append_intent.v0",
        "append_intent.audit_record.current_boot",
        "raios.audit_record.v0",
        "append.audit_ledger.current_boot",
        "append_engine.audit_ledger.current_boot",
        "storage.audit_rollback_layout.current_boot",
        "policy.durable_audit_write.current_boot",
        "availability.durable_audit_ledger.current_boot",
        intent.audit_record_append_intent,
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
        true,
    );
    emit_module_append_intent_fact(
        "rollback_transaction_append_intent",
        "raios.rollback_transaction_append_intent.v0",
        "append_intent.rollback_transaction.current_boot",
        "raios.rollback_plan.v0",
        "append.rollback_store.current_boot",
        "append_engine.rollback_store.current_boot",
        "storage.audit_rollback_layout.current_boot",
        "policy.rollback_install.current_boot",
        "availability.rollback_store.current_boot",
        intent.rollback_transaction_append_intent,
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_append_intent_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    append_contract_id: &'static str,
    append_engine_id: &'static str,
    storage_layout_id: &'static str,
    write_policy_id: &'static str,
    availability_id: &'static str,
    fact: ModuleAuditRollbackAppendIntentFact,
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
    raw("          \"append_contract_id\": ");
    json_str(append_contract_id);
    raw_line(",");
    raw("          \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("          \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("          \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("          \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("          \"intent_provenance_schema\": \"raios.append_intent_provenance.v0\",");
    raw_line("          \"payload_hash_schema\": \"raios.append_intent_payload_hash.v0\",");
    raw_line("          \"payload_hash_envelope_schema\": \"raios.append_payload_hash_envelope.canonical.v0\",");
    raw_line("          \"payload_hash\": null,");
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
    raw("          \"binds_append_contract\": ");
    raw_bool(fact.binds_append_contract);
    raw_line(",");
    raw("          \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw_line(",");
    raw("          \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw_line(",");
    raw("          \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw_line(",");
    raw("          \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw_line(",");
    raw("          \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw_line(",");
    raw("          \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw_line(",");
    raw("          \"binds_intent_provenance\": ");
    raw_bool(fact.binds_intent_provenance);
    raw_line(",");
    raw("          \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw_line(",");
    raw("          \"payload_hash_available\": ");
    raw_bool(fact.payload_hash_available);
    raw_line(",");
    raw_line("          \"required_bindings\": {");
    raw("            \"append_contract_id\": ");
    json_str(append_contract_id);
    raw_line(",");
    raw("            \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("            \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("            \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("            \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("            \"payload_hash_envelope\": \"required\",");
    raw_line("            \"provenance_schema\": \"raios.append_intent_provenance.v0\"");
    raw_line("          },");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"install_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_intent\",");
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

pub(crate) fn module_audit_rollback_append_intent_selftest_cases(
) -> [ModuleAuditRollbackAppendIntentSelfTestCase; MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES]
{
    let missing = module_audit_rollback_append_intent_snapshot();
    let available = module_audit_rollback_available_append_intent_fact();
    [
        module_audit_rollback_append_intent_selftest_case(
            "missing_append_intent_pair_current_boot",
            "missing",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            missing,
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_previous_boot",
            "rejected",
            "audit_record_append_intent_scope_must_be_current_boot",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_wrong_schema",
            "rejected",
            "audit_record_append_intent_schema_mismatch",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_provenance_missing",
            "rejected",
            "audit_record_append_intent_provenance_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_provenance_binding_missing",
            "rejected",
            "audit_record_append_intent_provenance_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_intent_provenance: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_contract_binding_missing",
            "rejected",
            "audit_record_append_intent_append_contract_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_engine_binding_missing",
            "rejected",
            "audit_record_append_intent_append_engine_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_engine_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_storage_layout_binding_missing",
            "rejected",
            "audit_record_append_intent_storage_layout_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_storage_layout_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_write_policy_binding_missing",
            "rejected",
            "audit_record_append_intent_write_policy_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_write_policy_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_availability_binding_missing",
            "rejected",
            "audit_record_append_intent_availability_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_availability_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_payload_hash_missing",
            "rejected",
            "audit_record_append_intent_payload_hash_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_contract_missing",
            "missing",
            "audit_record_append_contract_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    append_contract_available: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_payload_hash_envelope_missing",
            "missing",
            "audit_record_append_payload_hash_envelope_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    payload_hash_available: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_previous_boot",
            "rejected",
            "rollback_transaction_append_intent_scope_must_be_current_boot",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_wrong_schema",
            "rejected",
            "rollback_transaction_append_intent_schema_mismatch",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_provenance_missing",
            "rejected",
            "rollback_transaction_append_intent_provenance_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_append_contract_binding_missing",
            "rejected",
            "rollback_transaction_append_intent_append_contract_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_payload_hash_missing",
            "rejected",
            "rollback_transaction_append_intent_payload_hash_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_payload_hash_envelope_missing",
            "missing",
            "rollback_transaction_append_payload_hash_envelope_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    payload_hash_available: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "available_append_intents_still_non_authorizing",
            "available",
            "audit_rollback_append_intent_available",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: available,
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_append_intent_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendIntentCandidate,
) -> ModuleAuditRollbackAppendIntentSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_intent_candidate(candidate);
    ModuleAuditRollbackAppendIntentSelfTestCase {
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
