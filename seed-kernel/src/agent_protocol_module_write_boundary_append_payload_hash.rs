use alloc::vec;

use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_intent::*, agent_protocol_support::*, event_log,
    module_evidence,
};
use raios_core::record::Value as V;

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_append_payload_hash() {
    let binding = event_log::module_load_gate_binding_snapshot();
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let payload =
        module_audit_rollback_append_payload_hash_snapshot_from_binding_and_append_contract(
            binding,
            append_evaluation,
        );
    let evaluation = evaluate_module_audit_rollback_append_payload_hash_candidate(payload);

    begin_response("module.audit_rollback_append_payload_hash");
    emit_record_fields_trailing_comma(
        vec![
            record_field(
                "schema",
                record_str("raios.module_audit_rollback_append_payload_hash.v0"),
            ),
            record_field("scope", record_str("current_boot")),
            record_field("classification", record_str("local_only")),
            record_field("test_infrastructure", record_false()),
            record_field("mutates_global_event_log", record_false()),
            record_field("global_event_log_mutation", record_str("none")),
            record_field("writes_enabled", record_false()),
            record_field("creates_durable_audit_records", record_false()),
            record_field("creates_rollback_plans", record_false()),
            record_field("installs_rollback_plan", record_false()),
            record_field("service_inventory_change", record_str("none")),
            record_field("load_attempted", record_false()),
        ],
        6,
    );
    emit_module_append_payload_retained_inputs(&binding);
    raw_line(",");
    emit_module_append_intent_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_append_payload_hash_facts(payload, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            record_field("payload_hash_status", record_str(evaluation.status)),
            record_field("payload_hash_reason", record_str(evaluation.reason)),
            record_field(
                "retained_evidence_available",
                record_bool(evaluation.retained_evidence_available),
            ),
            record_field(
                "service_slot_reservation_available",
                record_bool(evaluation.service_slot_reservation_available),
            ),
            record_field(
                "append_contract_available",
                record_bool(evaluation.append_contract_available),
            ),
            record_field(
                "append_contract_missing",
                record_bool(!evaluation.append_contract_available),
            ),
            record_field(
                "payload_hash_available",
                record_bool(evaluation.payload_hash_available),
            ),
            record_field(
                "payload_hash_missing",
                record_bool(!evaluation.payload_hash_available),
            ),
            record_field("payload_hash_envelopes_are_append_intent_authority", record_false()),
            record_field("retained_hash_refs_are_payload_authority", record_false()),
            record_field("append_contract_facts_are_payload_authority", record_false()),
            record_field("can_load_now", record_false()),
            record_field("load_attempted", record_false()),
        ],
        true,
    );
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "audit_record_append_payload_hash",
        evaluation.audit_payload_status,
        evaluation.audit_payload_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_payload_hash",
        evaluation.rollback_payload_status,
        evaluation.rollback_payload_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_payload_hash");
}

pub(crate) fn emit_module_audit_rollback_append_payload_hash_selftest() {
    let cases = module_audit_rollback_append_payload_hash_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases
        .iter()
        .map(module_audit_rollback_append_payload_hash_selftest_case_record)
        .collect();

    begin_response("module.audit_rollback_append_payload_hash_selftest");
    emit_record_fields_trailing_comma(
        vec![
            record_field(
                "schema",
                record_str("raios.module_audit_rollback_append_payload_hash_selftest.v0"),
            ),
            record_field("scope", record_str("current_boot")),
            record_field("classification", record_str("local_only")),
            record_field("test_infrastructure", record_bool(true)),
            record_field("mutates_global_event_log", record_false()),
            record_field("creates_durable_audit_records", record_false()),
            record_field("creates_rollback_plans", record_false()),
            record_field("installs_rollback_plan", record_false()),
            record_field("service_inventory_change", record_str("none")),
            record_field("load_attempted", record_false()),
            record_field("case_count", V::U64(cases.len() as u64)),
            record_field("passed", record_bool(passed)),
            record_field("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", record_false(), false);
    end_response("module.audit_rollback_append_payload_hash_selftest");
}

#[rustfmt::skip]
fn module_audit_rollback_append_payload_hash_selftest_case_record(case: &ModuleAuditRollbackAppendPayloadHashSelfTestCase) -> V<'static> {
    V::InlineObject(vec![record_field("case", record_str(case.name)), record_field("expected_status", record_str(case.expected_status)), record_field("expected_reason", record_str(case.expected_reason)), record_field("actual_status", record_str(case.actual_status)), record_field("actual_reason", record_str(case.actual_reason)), record_field("passed", record_bool(case.passed)), record_field("writes_enabled", record_false()), record_field("installs_rollback_plan", record_false()), record_field("can_load", record_false()), record_field("load_attempted", record_false())])
}

pub(crate) fn module_audit_rollback_append_payload_hash_snapshot_from_binding_and_append_contract(
    binding: event_log::ModuleLoadGateBinding,
    append: ModuleAuditRollbackAppendContractEvaluation,
) -> ModuleAuditRollbackAppendPayloadHashCandidate {
    ModuleAuditRollbackAppendPayloadHashCandidate {
        audit_record_payload_hash: module_audit_rollback_append_payload_hash_fact_from_binding(
            binding,
            true,
            method_eq(append.audit_append_status, "available"),
        ),
        rollback_transaction_payload_hash:
            module_audit_rollback_append_payload_hash_fact_from_binding(
                binding,
                false,
                method_eq(append.rollback_transaction_status, "available"),
            ),
    }
}

pub(crate) fn module_audit_rollback_append_payload_hash_fact_from_binding(
    binding: event_log::ModuleLoadGateBinding,
    audit_payload: bool,
    append_contract_available: bool,
) -> ModuleAuditRollbackAppendPayloadHashFact {
    let retained_available = method_eq(
        binding.audit_rollback_reference_status,
        "retained_hash_reference_only",
    ) && binding.audit_rollback_reference.is_some();
    let service_slot_available = method_eq(
        binding.service_slot_reservation_status,
        "retained_hash_reference_only_not_allocated",
    ) && binding.service_slot_reservation.is_some();
    let present = retained_available && service_slot_available;
    let payload_hash = if present {
        match (
            binding.audit_rollback_reference_event_id,
            binding.service_slot_reservation_event_id,
            binding.audit_rollback_reference,
            binding.service_slot_reservation,
        ) {
            (Some(audit_event_id), Some(slot_event_id), Some(audit), Some(slot)) => {
                if audit_payload {
                    Some(
                        module_evidence::computed_module_audit_append_payload_hash_from_sequences(
                            audit_event_id.sequence(),
                            slot_event_id.sequence(),
                            audit.audit_record_hash,
                            audit.rollback_plan_hash,
                            audit.pre_load_service_inventory_hash,
                            slot.reservation_hash,
                            audit.ram_only_service_slot_id.as_str(),
                        ),
                    )
                } else {
                    Some(
                        module_evidence::computed_module_rollback_append_payload_hash_from_sequences(
                            audit_event_id.sequence(),
                            slot_event_id.sequence(),
                            audit.audit_record_hash,
                            audit.rollback_plan_hash,
                            audit.pre_load_service_inventory_hash,
                            slot.reservation_hash,
                            audit.ram_only_service_slot_id.as_str(),
                        ),
                    )
                }
            }
            _ => None,
        }
    } else {
        None
    };
    let audit = binding.audit_rollback_reference;
    let slot = binding.service_slot_reservation;
    ModuleAuditRollbackAppendPayloadHashFact {
        present,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: present,
        classification: "local_only",
        binds_retained_audit_rollback: present,
        binds_service_slot_reservation: present,
        binds_pre_load_write_request: present,
        binds_append_contract_id: present,
        binds_target_schema: present,
        binds_payload_hash: payload_hash.is_some(),
        binds_payload_provenance: present,
        retained_audit_rollback_available: retained_available,
        service_slot_reservation_available: service_slot_available,
        append_contract_available,
        retained_audit_rollback_event_id: if retained_available {
            binding.audit_rollback_reference_event_id
        } else {
            None
        },
        service_slot_reservation_event_id: if service_slot_available {
            binding.service_slot_reservation_event_id
        } else {
            None
        },
        payload_hash,
        source_payload_hash: if present {
            if audit_payload {
                audit.map(|reference| reference.audit_record_hash)
            } else {
                audit.map(|reference| reference.rollback_plan_hash)
            }
        } else {
            None
        },
        pre_load_service_inventory_hash: if present {
            audit.map(|reference| reference.pre_load_service_inventory_hash)
        } else {
            None
        },
        service_slot_reservation_hash: if present {
            slot.map(|reservation| reservation.reservation_hash)
        } else {
            None
        },
    }
}

pub(crate) fn module_audit_rollback_missing_append_payload_hash_fact(
) -> ModuleAuditRollbackAppendPayloadHashFact {
    ModuleAuditRollbackAppendPayloadHashFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_audit_rollback: false,
        binds_service_slot_reservation: false,
        binds_pre_load_write_request: false,
        binds_append_contract_id: false,
        binds_target_schema: false,
        binds_payload_hash: false,
        binds_payload_provenance: false,
        retained_audit_rollback_available: false,
        service_slot_reservation_available: false,
        append_contract_available: false,
        retained_audit_rollback_event_id: None,
        service_slot_reservation_event_id: None,
        payload_hash: None,
        source_payload_hash: None,
        pre_load_service_inventory_hash: None,
        service_slot_reservation_hash: None,
    }
}

pub(crate) fn module_audit_rollback_available_append_payload_hash_fact(
) -> ModuleAuditRollbackAppendPayloadHashFact {
    ModuleAuditRollbackAppendPayloadHashFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_audit_rollback: true,
        binds_service_slot_reservation: true,
        binds_pre_load_write_request: true,
        binds_append_contract_id: true,
        binds_target_schema: true,
        binds_payload_hash: true,
        binds_payload_provenance: true,
        retained_audit_rollback_available: true,
        service_slot_reservation_available: true,
        append_contract_available: true,
        retained_audit_rollback_event_id: None,
        service_slot_reservation_event_id: None,
        payload_hash: Some([0x51; 32]),
        source_payload_hash: Some([0x52; 32]),
        pre_load_service_inventory_hash: Some([0x53; 32]),
        service_slot_reservation_hash: Some([0x54; 32]),
    }
}

pub(crate) fn evaluate_module_audit_rollback_append_payload_hash_candidate(
    candidate: ModuleAuditRollbackAppendPayloadHashCandidate,
) -> ModuleAuditRollbackAppendPayloadHashEvaluation {
    let (audit_payload_status, audit_payload_reason) = evaluate_module_append_payload_hash_fact(
        candidate.audit_record_payload_hash,
        "audit_record_append_payload_hash_scope_must_be_current_boot",
        "audit_record_append_payload_hash_schema_mismatch",
        "audit_record_append_payload_hash_missing",
        "audit_record_append_payload_hash_provenance_missing",
        "audit_record_append_payload_hash_retained_binding_missing",
        "audit_record_append_payload_hash_service_slot_binding_missing",
        "audit_record_append_payload_hash_write_request_binding_missing",
        "audit_record_append_payload_hash_append_contract_binding_missing",
        "audit_record_append_payload_hash_target_schema_binding_missing",
        "audit_record_append_payload_hash_missing",
        "audit_record_append_payload_hash_provenance_binding_missing",
        "audit_record_retained_audit_rollback_missing",
        "audit_record_service_slot_reservation_missing",
        "audit_record_append_contract_missing",
        "audit_record_append_payload_hash_available",
    );
    let (rollback_payload_status, rollback_payload_reason) =
        evaluate_module_append_payload_hash_fact(
            candidate.rollback_transaction_payload_hash,
            "rollback_transaction_append_payload_hash_scope_must_be_current_boot",
            "rollback_transaction_append_payload_hash_schema_mismatch",
            "rollback_transaction_append_payload_hash_missing",
            "rollback_transaction_append_payload_hash_provenance_missing",
            "rollback_transaction_append_payload_hash_retained_binding_missing",
            "rollback_transaction_append_payload_hash_service_slot_binding_missing",
            "rollback_transaction_append_payload_hash_write_request_binding_missing",
            "rollback_transaction_append_payload_hash_append_contract_binding_missing",
            "rollback_transaction_append_payload_hash_target_schema_binding_missing",
            "rollback_transaction_append_payload_hash_missing",
            "rollback_transaction_append_payload_hash_provenance_binding_missing",
            "rollback_transaction_retained_audit_rollback_missing",
            "rollback_transaction_service_slot_reservation_missing",
            "rollback_transaction_append_contract_missing",
            "rollback_transaction_append_payload_hash_available",
        );

    let (status, reason) = if method_eq(audit_payload_status, "rejected") {
        ("rejected", audit_payload_reason)
    } else if method_eq(rollback_payload_status, "rejected") {
        ("rejected", rollback_payload_reason)
    } else if method_eq(audit_payload_status, "missing")
        && method_eq(
            audit_payload_reason,
            "audit_record_append_payload_hash_missing",
        )
        && method_eq(rollback_payload_status, "missing")
        && method_eq(
            rollback_payload_reason,
            "rollback_transaction_append_payload_hash_missing",
        )
    {
        (
            "missing",
            "audit_record_append_payload_hash_missing_and_rollback_transaction_append_payload_hash_missing",
        )
    } else if method_eq(audit_payload_status, "missing") {
        ("missing", audit_payload_reason)
    } else if method_eq(rollback_payload_status, "missing") {
        ("missing", rollback_payload_reason)
    } else {
        ("available", "audit_rollback_append_payload_hash_available")
    };

    let retained_evidence_available = candidate
        .audit_record_payload_hash
        .retained_audit_rollback_available
        && candidate
            .rollback_transaction_payload_hash
            .retained_audit_rollback_available;
    let service_slot_reservation_available = candidate
        .audit_record_payload_hash
        .service_slot_reservation_available
        && candidate
            .rollback_transaction_payload_hash
            .service_slot_reservation_available;
    let append_contract_available = candidate
        .audit_record_payload_hash
        .append_contract_available
        && candidate
            .rollback_transaction_payload_hash
            .append_contract_available;
    let payload_hash_available = method_eq(audit_payload_status, "available")
        && method_eq(rollback_payload_status, "available");
    ModuleAuditRollbackAppendPayloadHashEvaluation {
        status,
        reason,
        audit_payload_status,
        audit_payload_reason,
        rollback_payload_status,
        rollback_payload_reason,
        retained_evidence_available,
        service_slot_reservation_available,
        append_contract_available,
        payload_hash_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_append_payload_hash_fact(
    fact: ModuleAuditRollbackAppendPayloadHashFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_binding_reason: &'static str,
    service_slot_binding_reason: &'static str,
    write_request_binding_reason: &'static str,
    append_contract_binding_reason: &'static str,
    target_schema_binding_reason: &'static str,
    payload_hash_reason: &'static str,
    payload_provenance_reason: &'static str,
    retained_missing_reason: &'static str,
    service_slot_missing_reason: &'static str,
    append_contract_missing_reason: &'static str,
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
    if !fact.binds_payload_provenance {
        return ("rejected", payload_provenance_reason);
    }
    if !fact.binds_retained_audit_rollback {
        return ("rejected", retained_binding_reason);
    }
    if !fact.binds_service_slot_reservation {
        return ("rejected", service_slot_binding_reason);
    }
    if !fact.binds_pre_load_write_request {
        return ("rejected", write_request_binding_reason);
    }
    if !fact.binds_append_contract_id {
        return ("rejected", append_contract_binding_reason);
    }
    if !fact.binds_target_schema {
        return ("rejected", target_schema_binding_reason);
    }
    if !fact.binds_payload_hash || fact.payload_hash.is_none() {
        return ("rejected", payload_hash_reason);
    }
    if !fact.retained_audit_rollback_available {
        return ("missing", retained_missing_reason);
    }
    if !fact.service_slot_reservation_available {
        return ("missing", service_slot_missing_reason);
    }
    if !fact.append_contract_available {
        return ("missing", append_contract_missing_reason);
    }
    ("available", available_reason)
}

pub(crate) fn emit_module_append_payload_retained_inputs(
    binding: &event_log::ModuleLoadGateBinding,
) {
    emit_record_property_line(
        "retained_payload_inputs",
        vec![
            record_field(
                "audit_rollback",
                module_write_boundary_input_ref_record(
                    binding.audit_rollback_reference_event_id,
                    binding.audit_rollback_reference_status,
                    binding.audit_rollback_reference_reason,
                    "raios.module_audit_rollback_reference.v0",
                ),
            ),
            record_field(
                "service_slot_reservation",
                module_write_boundary_input_ref_record(
                    binding.service_slot_reservation_event_id,
                    binding.service_slot_reservation_status,
                    binding.service_slot_reservation_reason,
                    "raios.module_service_slot_reservation.v0",
                ),
            ),
            record_field(
                "audit_record_hash",
                record_sha_or_null(
                    binding
                        .audit_rollback_reference
                        .map(|reference| reference.audit_record_hash),
                ),
            ),
            record_field(
                "rollback_plan_hash",
                record_sha_or_null(
                    binding
                        .audit_rollback_reference
                        .map(|reference| reference.rollback_plan_hash),
                ),
            ),
            record_field(
                "pre_load_service_inventory_hash",
                record_sha_or_null(
                    binding
                        .audit_rollback_reference
                        .map(|reference| reference.pre_load_service_inventory_hash),
                ),
            ),
            record_field(
                "service_slot_reservation_hash",
                record_sha_or_null(
                    binding
                        .service_slot_reservation
                        .map(|reservation| reservation.reservation_hash),
                ),
            ),
            record_field(
                "ram_only_service_slot_id",
                match binding.audit_rollback_reference.as_ref() {
                    Some(reference) => record_str(reference.ram_only_service_slot_id.as_str()),
                    None => record_null(),
                },
            ),
        ],
        false,
    );
}

pub(crate) fn emit_module_append_payload_hash_facts(
    payload: ModuleAuditRollbackAppendPayloadHashCandidate,
    evaluation: ModuleAuditRollbackAppendPayloadHashEvaluation,
) {
    emit_record_property_line(
        "append_payload_hash_facts",
        vec![
            record_field(
                "audit_record_append_payload_hash",
                module_append_payload_hash_fact_record(
                    "raios.audit_record_append_payload_hash_envelope.v0",
                    "append_payload.audit_record.current_boot",
                    "raios.audit_record.v0",
                    "append.audit_ledger.current_boot",
                    payload.audit_record_payload_hash,
                    evaluation.audit_payload_status,
                    evaluation.audit_payload_reason,
                ),
            ),
            record_field(
                "rollback_transaction_append_payload_hash",
                module_append_payload_hash_fact_record(
                    "raios.rollback_transaction_append_payload_hash_envelope.v0",
                    "append_payload.rollback_transaction.current_boot",
                    "raios.rollback_transaction.v0",
                    "append.rollback_store.current_boot",
                    payload.rollback_transaction_payload_hash,
                    evaluation.rollback_payload_status,
                    evaluation.rollback_payload_reason,
                ),
            ),
        ],
        false,
    );
}

#[rustfmt::skip]
fn module_write_boundary_input_ref_record(event_id: Option<event_log::EventId>, status: &'static str, reason: &'static str, schema: &'static str) -> V<'static> {
    V::InlineObject(vec![record_field("event_id", record_event_or_null(event_id)), record_field("schema", record_str(schema)), record_field("status", record_str(status)), record_field("reason", record_str(reason)), record_field("classification", record_str("local_only")), record_field("authorizes_guest_load", record_false())])
}

#[rustfmt::skip]
fn module_append_payload_hash_fact_record(
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    append_contract_id: &'static str,
    fact: ModuleAuditRollbackAppendPayloadHashFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    V::Object(vec![
        record_field("schema", record_str(schema)),
        record_field("id", record_str(id)),
        record_field("target_schema", record_str(target_schema)),
        record_field(
            "canonicalization",
            record_str("raios.append_payload_hash_envelope.canonical.v0"),
        ),
        record_field(
            "pre_load_write_request_schema",
            record_str("raios.module_pre_load_audit_rollback_write_request.v0"),
        ),
        record_field("append_contract_id", record_str(append_contract_id)),
        record_field(
            "retained_audit_rollback_reference_event_id",
            record_event_or_null(fact.retained_audit_rollback_event_id),
        ),
        record_field(
            "service_slot_reservation_event_id",
            record_event_or_null(fact.service_slot_reservation_event_id),
        ),
        record_field("payload_hash", record_sha_or_null(fact.payload_hash)),
        record_field("source_payload_hash", record_sha_or_null(fact.source_payload_hash)),
        record_field(
            "pre_load_service_inventory_hash",
            record_sha_or_null(fact.pre_load_service_inventory_hash),
        ),
        record_field(
            "service_slot_reservation_hash",
            record_sha_or_null(fact.service_slot_reservation_hash),
        ),
        record_field("scope", record_str(fact.scope)),
        record_field("classification", record_str(fact.classification)),
        record_field("status", record_str(status)),
        record_field("reason", record_str(reason)),
        record_field("present", record_bool(fact.present)),
        record_field("schema_valid", record_bool(fact.schema_ok)),
        record_field("provenance_valid", record_bool(fact.provenance_ok)),
        record_field(
            "binds_retained_audit_rollback",
            record_bool(fact.binds_retained_audit_rollback),
        ),
        record_field(
            "binds_service_slot_reservation",
            record_bool(fact.binds_service_slot_reservation),
        ),
        record_field(
            "binds_pre_load_write_request",
            record_bool(fact.binds_pre_load_write_request),
        ),
        record_field("binds_append_contract_id", record_bool(fact.binds_append_contract_id)),
        record_field("binds_target_schema", record_bool(fact.binds_target_schema)),
        record_field("binds_payload_hash", record_bool(fact.binds_payload_hash)),
        record_field(
            "binds_payload_provenance",
            record_bool(fact.binds_payload_provenance),
        ),
        record_field(
            "retained_audit_rollback_available",
            record_bool(fact.retained_audit_rollback_available),
        ),
        record_field(
            "service_slot_reservation_available",
            record_bool(fact.service_slot_reservation_available),
        ),
        record_field("append_contract_available", record_bool(fact.append_contract_available)),
        record_field("authority", record_str("current_snapshot")),
        record_field("persistence", record_str("none")),
        record_field("durable", record_false()),
        record_field("authorizes_append_intent", record_false()),
        record_field("authorizes_write", record_false()),
        record_field("write_attempted", record_false()),
        record_field("install_attempted", record_false()),
        record_field(
            "provenance",
            V::Object(vec![
                record_field(
                    "source_method",
                    record_str("module.audit_rollback_append_payload_hash"),
                ),
                record_field("source_transport", record_str("serial-console")),
                record_field("event_scope", record_str("current_boot")),
                record_field("record_id", record_null()),
            ]),
        ),
    ])
}

pub(crate) fn module_audit_rollback_append_payload_hash_selftest_cases(
) -> [ModuleAuditRollbackAppendPayloadHashSelfTestCase;
       MODULE_AUDIT_ROLLBACK_APPEND_PAYLOAD_HASH_SELFTEST_CASES] {
    let missing_fact = module_audit_rollback_missing_append_payload_hash_fact();
    let missing = ModuleAuditRollbackAppendPayloadHashCandidate {
        audit_record_payload_hash: missing_fact,
        rollback_transaction_payload_hash: missing_fact,
    };
    let available = module_audit_rollback_available_append_payload_hash_fact();
    [
        module_audit_rollback_append_payload_hash_selftest_case(
            "missing_payload_hash_pair_current_boot",
            "missing",
            "audit_record_append_payload_hash_missing_and_rollback_transaction_append_payload_hash_missing",
            missing,
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_previous_boot",
            "rejected",
            "audit_record_append_payload_hash_scope_must_be_current_boot",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_wrong_schema",
            "rejected",
            "audit_record_append_payload_hash_schema_mismatch",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_provenance_missing",
            "rejected",
            "audit_record_append_payload_hash_provenance_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_retained_binding_missing",
            "rejected",
            "audit_record_append_payload_hash_retained_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_retained_audit_rollback: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_service_slot_binding_missing",
            "rejected",
            "audit_record_append_payload_hash_service_slot_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_service_slot_reservation: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_write_request_binding_missing",
            "rejected",
            "audit_record_append_payload_hash_write_request_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_pre_load_write_request: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_append_contract_binding_missing",
            "rejected",
            "audit_record_append_payload_hash_append_contract_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_append_contract_id: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_target_schema_binding_missing",
            "rejected",
            "audit_record_append_payload_hash_target_schema_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_target_schema: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_payload_hash_missing",
            "rejected",
            "audit_record_append_payload_hash_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    payload_hash: None,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_retained_audit_rollback_missing",
            "missing",
            "audit_record_retained_audit_rollback_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    retained_audit_rollback_available: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_service_slot_reservation_missing",
            "missing",
            "audit_record_service_slot_reservation_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    service_slot_reservation_available: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "audit_record_append_contract_missing",
            "missing",
            "audit_record_append_contract_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    append_contract_available: false,
                    ..available
                },
                rollback_transaction_payload_hash: available,
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_payload_hash_previous_boot",
            "rejected",
            "rollback_transaction_append_payload_hash_scope_must_be_current_boot",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_payload_hash_wrong_schema",
            "rejected",
            "rollback_transaction_append_payload_hash_schema_mismatch",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_payload_hash_provenance_missing",
            "rejected",
            "rollback_transaction_append_payload_hash_provenance_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_payload_hash_append_contract_binding_missing",
            "rejected",
            "rollback_transaction_append_payload_hash_append_contract_binding_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    binds_append_contract_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_payload_hash_missing",
            "rejected",
            "rollback_transaction_append_payload_hash_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    payload_hash: None,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "rollback_transaction_append_contract_missing",
            "missing",
            "rollback_transaction_append_contract_missing",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact {
                    append_contract_available: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_payload_hash_selftest_case(
            "available_payload_hashes_still_non_authorizing",
            "available",
            "audit_rollback_append_payload_hash_available",
            ModuleAuditRollbackAppendPayloadHashCandidate {
                audit_record_payload_hash: available,
                rollback_transaction_payload_hash: available,
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_append_payload_hash_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendPayloadHashCandidate,
) -> ModuleAuditRollbackAppendPayloadHashSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_payload_hash_candidate(candidate);
    ModuleAuditRollbackAppendPayloadHashSelfTestCase {
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
