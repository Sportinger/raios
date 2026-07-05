use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_payload_hash::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_record_fields_trailing_comma,
        emit_record_property_line, emit_selftest_case_fields, end_response, method_eq, raw_line,
        record_bool as b, record_false as no, record_field as f, record_inline as inline,
        record_object as object, record_sha_or_null, record_str as s, run_selftest_cases_with,
        CaseSpec, SelftestReportField::False,
    },
    event_log,
};
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum AppendIntentSelftestMutation {
    MissingPair,
    AuditPreviousBoot,
    AuditWrongSchema,
    AuditProvenanceMissing,
    AuditProvenanceBindingMissing,
    AuditAppendContractBindingMissing,
    AuditAppendEngineBindingMissing,
    AuditStorageLayoutBindingMissing,
    AuditWritePolicyBindingMissing,
    AuditAvailabilityBindingMissing,
    AuditPayloadHashMissing,
    AuditAppendContractMissing,
    AuditPayloadHashEnvelopeMissing,
    RollbackPreviousBoot,
    RollbackWrongSchema,
    RollbackProvenanceMissing,
    RollbackAppendContractBindingMissing,
    RollbackPayloadHashMissing,
    RollbackPayloadHashEnvelopeMissing,
    AvailableIntents,
}

const fn append_intent_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: AppendIntentSelftestMutation,
) -> CaseSpec<AppendIntentSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const APPEND_INTENT_CASES: [CaseSpec<AppendIntentSelftestMutation>;
    MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES] = [
    append_intent_case(
        "missing_append_intent_pair_current_boot",
        "missing",
        "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
        AppendIntentSelftestMutation::MissingPair,
    ),
    append_intent_case(
        "audit_record_append_intent_previous_boot",
        "rejected",
        "audit_record_append_intent_scope_must_be_current_boot",
        AppendIntentSelftestMutation::AuditPreviousBoot,
    ),
    append_intent_case(
        "audit_record_append_intent_wrong_schema",
        "rejected",
        "audit_record_append_intent_schema_mismatch",
        AppendIntentSelftestMutation::AuditWrongSchema,
    ),
    append_intent_case(
        "audit_record_append_intent_provenance_missing",
        "rejected",
        "audit_record_append_intent_provenance_missing",
        AppendIntentSelftestMutation::AuditProvenanceMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_provenance_binding_missing",
        "rejected",
        "audit_record_append_intent_provenance_binding_missing",
        AppendIntentSelftestMutation::AuditProvenanceBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_append_contract_binding_missing",
        "rejected",
        "audit_record_append_intent_append_contract_binding_missing",
        AppendIntentSelftestMutation::AuditAppendContractBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_append_engine_binding_missing",
        "rejected",
        "audit_record_append_intent_append_engine_binding_missing",
        AppendIntentSelftestMutation::AuditAppendEngineBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_storage_layout_binding_missing",
        "rejected",
        "audit_record_append_intent_storage_layout_binding_missing",
        AppendIntentSelftestMutation::AuditStorageLayoutBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_write_policy_binding_missing",
        "rejected",
        "audit_record_append_intent_write_policy_binding_missing",
        AppendIntentSelftestMutation::AuditWritePolicyBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_availability_binding_missing",
        "rejected",
        "audit_record_append_intent_availability_binding_missing",
        AppendIntentSelftestMutation::AuditAvailabilityBindingMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_payload_hash_missing",
        "rejected",
        "audit_record_append_intent_payload_hash_missing",
        AppendIntentSelftestMutation::AuditPayloadHashMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_append_contract_missing",
        "missing",
        "audit_record_append_contract_missing",
        AppendIntentSelftestMutation::AuditAppendContractMissing,
    ),
    append_intent_case(
        "audit_record_append_intent_payload_hash_envelope_missing",
        "missing",
        "audit_record_append_payload_hash_envelope_missing",
        AppendIntentSelftestMutation::AuditPayloadHashEnvelopeMissing,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_previous_boot",
        "rejected",
        "rollback_transaction_append_intent_scope_must_be_current_boot",
        AppendIntentSelftestMutation::RollbackPreviousBoot,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_wrong_schema",
        "rejected",
        "rollback_transaction_append_intent_schema_mismatch",
        AppendIntentSelftestMutation::RollbackWrongSchema,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_provenance_missing",
        "rejected",
        "rollback_transaction_append_intent_provenance_missing",
        AppendIntentSelftestMutation::RollbackProvenanceMissing,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_append_contract_binding_missing",
        "rejected",
        "rollback_transaction_append_intent_append_contract_binding_missing",
        AppendIntentSelftestMutation::RollbackAppendContractBindingMissing,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_payload_hash_missing",
        "rejected",
        "rollback_transaction_append_intent_payload_hash_missing",
        AppendIntentSelftestMutation::RollbackPayloadHashMissing,
    ),
    append_intent_case(
        "rollback_transaction_append_intent_payload_hash_envelope_missing",
        "missing",
        "rollback_transaction_append_payload_hash_envelope_missing",
        AppendIntentSelftestMutation::RollbackPayloadHashEnvelopeMissing,
    ),
    append_intent_case(
        "available_append_intents_still_non_authorizing",
        "available",
        "audit_rollback_append_intent_available",
        AppendIntentSelftestMutation::AvailableIntents,
    ),
];

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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_append_intent.v0")),
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
    emit_module_append_intent_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_append_intent_payload_hash_inputs(payload, payload_evaluation);
    raw_line(",");
    emit_module_append_intent_facts(intent, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            f("append_intent_status", s(evaluation.status)),
            f("append_intent_reason", s(evaluation.reason)),
            f(
                "audit_intent_missing",
                b(!method_eq(evaluation.audit_intent_status, "available")),
            ),
            f(
                "rollback_intent_missing",
                b(!method_eq(evaluation.rollback_intent_status, "available")),
            ),
            f(
                "append_contract_available",
                b(evaluation.append_contract_available),
            ),
            f(
                "append_contract_missing",
                b(!evaluation.append_contract_available),
            ),
            f(
                "payload_hash_available",
                b(evaluation.payload_hash_available),
            ),
            f(
                "payload_hash_missing",
                b(!evaluation.payload_hash_available),
            ),
            f(
                "append_intent_available",
                b(evaluation.append_intent_available),
            ),
            f(
                "append_intent_missing",
                b(!evaluation.append_intent_available),
            ),
            f("append_intent_facts_are_writer_authority", no()),
            f("append_contract_facts_are_append_intent_authority", no()),
            f("retained_hash_refs_are_append_intent_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_append_intent_selftest.v0"),
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
    emit_selftest_case_fields(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
        &[
            False("writes_enabled"),
            False("installs_rollback_plan"),
            False("can_load"),
            False("load_attempted"),
        ],
        comma,
    );
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
    emit_record_property_line(
        "append_contract_inputs",
        vec![
            f(
                "audit_append_envelope",
                module_append_intent_append_contract_input_record(
                    "raios.audit_ledger_append_envelope.v0",
                    append.audit_append_envelope,
                    evaluation.audit_append_status,
                    evaluation.audit_append_reason,
                ),
            ),
            f(
                "rollback_transaction_envelope",
                module_append_intent_append_contract_input_record(
                    "raios.rollback_store_transaction_envelope.v0",
                    append.rollback_transaction_envelope,
                    evaluation.rollback_transaction_status,
                    evaluation.rollback_transaction_reason,
                ),
            ),
            f(
                "append_contract_available",
                b(method_eq(evaluation.audit_append_status, "available")
                    && method_eq(evaluation.rollback_transaction_status, "available")),
            ),
            f("append_contract_facts_are_append_intent_authority", no()),
        ],
        false,
    );
}

fn module_append_intent_append_contract_input_record(
    schema: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    inline(vec![
        f("schema", s(schema)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("present", b(fact.present)),
        f("binds_storage_layout_id", b(fact.binds_storage_layout_id)),
        f("binds_append_engine_id", b(fact.binds_append_engine_id)),
        f("binds_write_policy_id", b(fact.binds_write_policy_id)),
        f("binds_availability_id", b(fact.binds_availability_id)),
        f(
            "binds_envelope_provenance",
            b(fact.binds_envelope_provenance),
        ),
        f("authorizes_append_intent", no()),
    ])
}

pub(crate) fn emit_module_append_intent_payload_hash_inputs(
    payload: ModuleAuditRollbackAppendPayloadHashCandidate,
    evaluation: ModuleAuditRollbackAppendPayloadHashEvaluation,
) {
    emit_record_property_line(
        "append_payload_hash_inputs",
        vec![
            f(
                "audit_record_append_payload_hash",
                module_append_intent_payload_hash_input_record(
                    "raios.audit_record_append_payload_hash_envelope.v0",
                    payload.audit_record_payload_hash,
                    evaluation.audit_payload_status,
                    evaluation.audit_payload_reason,
                ),
            ),
            f(
                "rollback_transaction_append_payload_hash",
                module_append_intent_payload_hash_input_record(
                    "raios.rollback_transaction_append_payload_hash_envelope.v0",
                    payload.rollback_transaction_payload_hash,
                    evaluation.rollback_payload_status,
                    evaluation.rollback_payload_reason,
                ),
            ),
            f(
                "payload_hash_available",
                b(evaluation.payload_hash_available),
            ),
            f("payload_hash_envelopes_are_append_intent_authority", no()),
        ],
        false,
    );
}

fn module_append_intent_payload_hash_input_record(
    schema: &'static str,
    fact: ModuleAuditRollbackAppendPayloadHashFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    inline(vec![
        f("schema", s(schema)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("present", b(fact.present)),
        f("payload_hash", record_sha_or_null(fact.payload_hash)),
        f(
            "source_payload_hash",
            record_sha_or_null(fact.source_payload_hash),
        ),
        f(
            "binds_retained_audit_rollback",
            b(fact.binds_retained_audit_rollback),
        ),
        f(
            "binds_service_slot_reservation",
            b(fact.binds_service_slot_reservation),
        ),
        f(
            "binds_pre_load_write_request",
            b(fact.binds_pre_load_write_request),
        ),
        f("binds_append_contract_id", b(fact.binds_append_contract_id)),
        f("binds_payload_hash", b(fact.binds_payload_hash)),
        f(
            "append_contract_available",
            b(fact.append_contract_available),
        ),
        f("authorizes_append_intent", no()),
    ])
}

pub(crate) fn emit_module_append_intent_facts(
    intent: ModuleAuditRollbackAppendIntentCandidate,
    evaluation: ModuleAuditRollbackAppendIntentEvaluation,
) {
    emit_record_property_line(
        "append_intent_facts",
        vec![
            f(
                "audit_record_append_intent",
                module_append_intent_fact_record(
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
                ),
            ),
            f(
                "rollback_transaction_append_intent",
                module_append_intent_fact_record(
                    "raios.rollback_transaction_append_intent.v0",
                    "append_intent.rollback_transaction.current_boot",
                    "raios.rollback_transaction.v0",
                    "append.rollback_store.current_boot",
                    "append_engine.rollback_store.current_boot",
                    "storage.audit_rollback_layout.current_boot",
                    "policy.rollback_install.current_boot",
                    "availability.rollback_store.current_boot",
                    intent.rollback_transaction_append_intent,
                    evaluation.rollback_intent_status,
                    evaluation.rollback_intent_reason,
                ),
            ),
        ],
        false,
    );
}

fn module_append_intent_fact_record(
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
) -> V<'static> {
    object(vec![
        f("schema", s(schema)),
        f("id", s(id)),
        f("target_schema", s(target_schema)),
        f("append_contract_id", s(append_contract_id)),
        f("append_engine_id", s(append_engine_id)),
        f("storage_layout_id", s(storage_layout_id)),
        f("write_policy_id", s(write_policy_id)),
        f("availability_id", s(availability_id)),
        f(
            "intent_provenance_schema",
            s("raios.append_intent_provenance.v0"),
        ),
        f(
            "payload_hash_schema",
            s("raios.append_intent_payload_hash.v0"),
        ),
        f(
            "payload_hash_envelope_schema",
            s("raios.append_payload_hash_envelope.canonical.v0"),
        ),
        f("payload_hash", V::Null),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("present", b(fact.present)),
        f("schema_valid", b(fact.schema_ok)),
        f("provenance_valid", b(fact.provenance_ok)),
        f("binds_append_contract", b(fact.binds_append_contract)),
        f("binds_append_contract_id", b(fact.binds_append_contract_id)),
        f("binds_append_engine_id", b(fact.binds_append_engine_id)),
        f("binds_storage_layout_id", b(fact.binds_storage_layout_id)),
        f("binds_write_policy_id", b(fact.binds_write_policy_id)),
        f("binds_availability_id", b(fact.binds_availability_id)),
        f("binds_payload_hash", b(fact.binds_payload_hash)),
        f("binds_intent_provenance", b(fact.binds_intent_provenance)),
        f(
            "append_contract_available",
            b(fact.append_contract_available),
        ),
        f("payload_hash_available", b(fact.payload_hash_available)),
        f(
            "required_bindings",
            object(vec![
                f("append_contract_id", s(append_contract_id)),
                f("append_engine_id", s(append_engine_id)),
                f("storage_layout_id", s(storage_layout_id)),
                f("write_policy_id", s(write_policy_id)),
                f("availability_id", s(availability_id)),
                f("payload_hash_envelope", s("required")),
                f("provenance_schema", s("raios.append_intent_provenance.v0")),
            ]),
        ),
        f("authority", s("current_snapshot")),
        f("persistence", s("none")),
        f("durable", no()),
        f("write_attempted", no()),
        f("install_attempted", no()),
        f(
            "provenance",
            object(vec![
                f("source_method", s("module.audit_rollback_append_intent")),
                f("source_transport", s("serial-console")),
                f("event_scope", s("current_boot")),
                f("record_id", V::Null),
            ]),
        ),
    ])
}

pub(crate) fn module_audit_rollback_append_intent_selftest_cases(
) -> [ModuleAuditRollbackAppendIntentSelfTestCase; MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES]
{
    run_selftest_cases_with(
        module_audit_rollback_append_intent_snapshot(),
        &APPEND_INTENT_CASES,
        apply_append_intent_selftest_case,
        evaluate_append_intent_selftest_case,
        module_audit_rollback_append_intent_selftest_case_from_spec,
    )
}

fn apply_append_intent_selftest_case(
    candidate: &mut ModuleAuditRollbackAppendIntentCandidate,
    mutation: AppendIntentSelftestMutation,
) {
    *candidate = module_audit_rollback_append_intent_selftest_candidate(mutation);
}

fn evaluate_append_intent_selftest_case(
    candidate: ModuleAuditRollbackAppendIntentCandidate,
    _require_live_retained: bool,
) -> ModuleAuditRollbackAppendIntentEvaluation {
    evaluate_module_audit_rollback_append_intent_candidate(candidate)
}

fn module_audit_rollback_append_intent_selftest_candidate(
    mutation: AppendIntentSelftestMutation,
) -> ModuleAuditRollbackAppendIntentCandidate {
    let missing = module_audit_rollback_append_intent_snapshot();
    let available = module_audit_rollback_available_append_intent_fact();
    match mutation {
        AppendIntentSelftestMutation::MissingPair => missing,
        AppendIntentSelftestMutation::AuditPreviousBoot => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditWrongSchema => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditProvenanceMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditProvenanceBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_intent_provenance: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditAppendContractBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditAppendEngineBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_engine_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditStorageLayoutBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_storage_layout_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditWritePolicyBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_write_policy_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditAvailabilityBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_availability_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditPayloadHashMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditAppendContractMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    append_contract_available: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::AuditPayloadHashEnvelopeMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    payload_hash_available: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            }
        }
        AppendIntentSelftestMutation::RollbackPreviousBoot => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::RollbackWrongSchema => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::RollbackProvenanceMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::RollbackAppendContractBindingMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract_id: false,
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::RollbackPayloadHashMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::RollbackPayloadHashEnvelopeMissing => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    payload_hash_available: false,
                    ..available
                },
            }
        }
        AppendIntentSelftestMutation::AvailableIntents => {
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: available,
            }
        }
    }
}

fn module_audit_rollback_append_intent_selftest_case_from_spec(
    spec: &CaseSpec<AppendIntentSelftestMutation>,
    actual: ModuleAuditRollbackAppendIntentEvaluation,
) -> ModuleAuditRollbackAppendIntentSelfTestCase {
    ModuleAuditRollbackAppendIntentSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && !actual.writes_enabled
            && !actual.installs_rollback_plan
            && !actual.can_load
            && !actual.load_attempted,
    }
}
