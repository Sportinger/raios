use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_intent::*,
    agent_protocol_module_write_boundary_emit::*, agent_protocol_support::*, event_log,
    module_evidence,
};

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
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_payload_hash.v0\",");
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
    emit_module_append_payload_retained_inputs(binding);
    raw_line(",");
    emit_module_append_intent_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_append_payload_hash_facts(payload, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"payload_hash_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"payload_hash_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_evidence_available\": ");
    raw_bool(evaluation.retained_evidence_available);
    raw_line(",");
    raw("        \"service_slot_reservation_available\": ");
    raw_bool(evaluation.service_slot_reservation_available);
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
    raw_line("        \"payload_hash_envelopes_are_append_intent_authority\": false,");
    raw_line("        \"retained_hash_refs_are_payload_authority\": false,");
    raw_line("        \"append_contract_facts_are_payload_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
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

    begin_response("module.audit_rollback_append_payload_hash_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_payload_hash_selftest.v0\",");
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
        emit_module_audit_rollback_append_payload_hash_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_payload_hash_selftest");
}

pub(crate) fn emit_module_audit_rollback_append_payload_hash_selftest_case(
    case: &ModuleAuditRollbackAppendPayloadHashSelfTestCase,
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
    binding: event_log::ModuleLoadGateBinding,
) {
    raw_line("      \"retained_payload_inputs\": {");
    emit_module_write_boundary_input_ref(
        "audit_rollback",
        binding.audit_rollback_reference_event_id,
        binding.audit_rollback_reference_status,
        binding.audit_rollback_reference_reason,
        "raios.module_audit_rollback_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "service_slot_reservation",
        binding.service_slot_reservation_event_id,
        binding.service_slot_reservation_status,
        binding.service_slot_reservation_reason,
        "raios.module_service_slot_reservation.v0",
        true,
    );
    raw("        \"audit_record_hash\": ");
    json_sha256_option(
        binding
            .audit_rollback_reference
            .map(|reference| reference.audit_record_hash),
    );
    raw_line(",");
    raw("        \"rollback_plan_hash\": ");
    json_sha256_option(
        binding
            .audit_rollback_reference
            .map(|reference| reference.rollback_plan_hash),
    );
    raw_line(",");
    raw("        \"pre_load_service_inventory_hash\": ");
    json_sha256_option(
        binding
            .audit_rollback_reference
            .map(|reference| reference.pre_load_service_inventory_hash),
    );
    raw_line(",");
    raw("        \"service_slot_reservation_hash\": ");
    json_sha256_option(
        binding
            .service_slot_reservation
            .map(|reservation| reservation.reservation_hash),
    );
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    if let Some(reference) = binding.audit_rollback_reference {
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw("null");
    }
    crlf();
    raw_line("      }");
}

pub(crate) fn emit_module_append_payload_hash_facts(
    payload: ModuleAuditRollbackAppendPayloadHashCandidate,
    evaluation: ModuleAuditRollbackAppendPayloadHashEvaluation,
) {
    raw_line("      \"append_payload_hash_facts\": {");
    emit_module_append_payload_hash_fact(
        "audit_record_append_payload_hash",
        "raios.audit_record_append_payload_hash_envelope.v0",
        "append_payload.audit_record.current_boot",
        "raios.audit_record.v0",
        "append.audit_ledger.current_boot",
        payload.audit_record_payload_hash,
        evaluation.audit_payload_status,
        evaluation.audit_payload_reason,
        true,
    );
    emit_module_append_payload_hash_fact(
        "rollback_transaction_append_payload_hash",
        "raios.rollback_transaction_append_payload_hash_envelope.v0",
        "append_payload.rollback_transaction.current_boot",
        "raios.rollback_plan.v0",
        "append.rollback_store.current_boot",
        payload.rollback_transaction_payload_hash,
        evaluation.rollback_payload_status,
        evaluation.rollback_payload_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_append_payload_hash_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    append_contract_id: &'static str,
    fact: ModuleAuditRollbackAppendPayloadHashFact,
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
    raw_line(
        "          \"canonicalization\": \"raios.append_payload_hash_envelope.canonical.v0\",",
    );
    raw_line(
        "          \"pre_load_write_request_schema\": \"raios.module_pre_load_audit_rollback_write_request.v0\",",
    );
    raw("          \"append_contract_id\": ");
    json_str(append_contract_id);
    raw_line(",");
    raw("          \"retained_audit_rollback_reference_event_id\": ");
    json_event_id_option(fact.retained_audit_rollback_event_id);
    raw_line(",");
    raw("          \"service_slot_reservation_event_id\": ");
    json_event_id_option(fact.service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"payload_hash\": ");
    json_sha256_option(fact.payload_hash);
    raw_line(",");
    raw("          \"source_payload_hash\": ");
    json_sha256_option(fact.source_payload_hash);
    raw_line(",");
    raw("          \"pre_load_service_inventory_hash\": ");
    json_sha256_option(fact.pre_load_service_inventory_hash);
    raw_line(",");
    raw("          \"service_slot_reservation_hash\": ");
    json_sha256_option(fact.service_slot_reservation_hash);
    raw_line(",");
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
    raw("          \"binds_retained_audit_rollback\": ");
    raw_bool(fact.binds_retained_audit_rollback);
    raw_line(",");
    raw("          \"binds_service_slot_reservation\": ");
    raw_bool(fact.binds_service_slot_reservation);
    raw_line(",");
    raw("          \"binds_pre_load_write_request\": ");
    raw_bool(fact.binds_pre_load_write_request);
    raw_line(",");
    raw("          \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw_line(",");
    raw("          \"binds_target_schema\": ");
    raw_bool(fact.binds_target_schema);
    raw_line(",");
    raw("          \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw_line(",");
    raw("          \"binds_payload_provenance\": ");
    raw_bool(fact.binds_payload_provenance);
    raw_line(",");
    raw("          \"retained_audit_rollback_available\": ");
    raw_bool(fact.retained_audit_rollback_available);
    raw_line(",");
    raw("          \"service_slot_reservation_available\": ");
    raw_bool(fact.service_slot_reservation_available);
    raw_line(",");
    raw("          \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_append_intent\": false,");
    raw_line("          \"authorizes_write\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"install_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_payload_hash\",");
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
