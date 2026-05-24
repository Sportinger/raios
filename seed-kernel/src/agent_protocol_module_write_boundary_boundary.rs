use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_intent::*,
    agent_protocol_module_write_boundary_append_payload_hash::*,
    agent_protocol_module_write_boundary_availability::*,
    agent_protocol_module_write_boundary_emit::*,
    agent_protocol_module_write_boundary_write_policy::*, agent_protocol_support::*, event_log,
};

pub(crate) fn emit_module_audit_rollback_write_boundary() {
    let binding = event_log::module_load_gate_binding_snapshot();
    let availability = module_audit_rollback_availability_snapshot();
    let availability_evaluation =
        evaluate_module_audit_rollback_availability_candidate(availability);
    let policy = module_audit_rollback_write_policy_snapshot();
    let policy_evaluation = evaluate_module_audit_rollback_write_policy_candidate(policy);
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let append_payload =
        module_audit_rollback_append_payload_hash_snapshot_from_binding_and_append_contract(
            binding,
            append_evaluation,
        );
    let append_payload_evaluation =
        evaluate_module_audit_rollback_append_payload_hash_candidate(append_payload);
    let append_intent =
        module_audit_rollback_append_intent_snapshot_from_append_contract_and_payload(
            append_evaluation,
            append_payload_evaluation,
        );
    let append_intent_evaluation =
        evaluate_module_audit_rollback_append_intent_candidate(append_intent);
    let candidate = module_audit_rollback_write_boundary_candidate_from_binding(
        binding,
        availability_evaluation,
        policy_evaluation,
        append_evaluation,
        append_intent_evaluation,
    );
    let evaluation = evaluate_module_audit_rollback_write_boundary_candidate(candidate);

    begin_response("module.audit_rollback_write_boundary");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_boundary.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"pre_load_write_request\": {");
    raw_line("        \"schema\": \"raios.module_pre_load_audit_rollback_write_request.v0\",");
    raw_line(
        "        \"canonicalization\": \"raios.module_pre_load_audit_rollback_write_request.canonical.v0\",",
    );
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"requested_writes\": [");
    raw_line(
        "          {\"target\": \"durable_audit_ledger\", \"schema\": \"raios.audit_record.v0\"},",
    );
    raw_line("          {\"target\": \"rollback_store\", \"schema\": \"raios.rollback_plan.v0\"}");
    raw_line("        ],");
    raw_line("        \"required_retained_references\": [");
    raw_line("          \"raios.module_manifest_reference.v0\",");
    raw_line("          \"raios.module_candidate_artifact_reference.v0\",");
    raw_line("          \"raios.module_vm_test_report_reference.v0\",");
    raw_line("          \"raios.computed_capability_grant.v0\",");
    raw_line("          \"raios.module_local_attestation_reference.v0\",");
    raw_line("          \"raios.module_local_approval_reference.v0\",");
    raw_line("          \"raios.module_audit_rollback_reference.v0\",");
    raw_line("          \"raios.module_service_slot_reservation.v0\"");
    raw_line("        ],");
    raw_line("        \"recovery_artifact_loading\": \"separate_capability\"");
    raw_line("      },");
    emit_module_write_boundary_inputs(binding);
    raw_line(",");
    emit_module_write_boundary_availability_inputs(availability, availability_evaluation);
    raw_line(",");
    emit_module_write_boundary_policy_inputs(policy, policy_evaluation);
    raw_line(",");
    emit_module_write_boundary_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_write_boundary_append_payload_hash_inputs(
        append_payload,
        append_payload_evaluation,
    );
    raw_line(",");
    emit_module_write_boundary_append_intent_inputs(append_intent, append_intent_evaluation);
    raw_line(",");
    emit_module_write_boundary_denial_evidence(evaluation);
    raw_line(",");
    emit_module_write_boundary_policy_result(
        evaluation,
        policy_evaluation,
        append_evaluation,
        append_payload_evaluation,
        append_intent_evaluation,
    );
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !method_eq(evaluation.status, "denied_missing_durable_write_boundary") {
        emit_export_gate(
            &mut wrote,
            "write_boundary_preconditions",
            evaluation.status,
            evaluation.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "durable_audit_write",
        evaluation.durable_audit_write_state,
        evaluation.durable_audit_write_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_install",
        evaluation.rollback_install_state,
        evaluation.rollback_install_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_append_envelope",
        append_evaluation.audit_append_status,
        append_evaluation.audit_append_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_envelope",
        append_evaluation.rollback_transaction_status,
        append_evaluation.rollback_transaction_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_payload_hash",
        append_payload_evaluation.audit_payload_status,
        append_payload_evaluation.audit_payload_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_payload_hash",
        append_payload_evaluation.rollback_payload_status,
        append_payload_evaluation.rollback_payload_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_intent",
        append_intent_evaluation.audit_intent_status,
        append_intent_evaluation.audit_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_intent",
        append_intent_evaluation.rollback_intent_status,
        append_intent_evaluation.rollback_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "module_loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_write_boundary");
}

pub(crate) fn emit_module_write_boundary_inputs(binding: event_log::ModuleLoadGateBinding) {
    raw_line("      \"retained_reference_inputs\": {");
    emit_module_write_boundary_input_ref(
        "module_manifest",
        binding.manifest_reference_event_id,
        binding.manifest_reference_status,
        binding.manifest_reference_reason,
        "raios.module_manifest_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "candidate_artifact",
        binding.artifact_reference_event_id,
        binding.artifact_reference_status,
        binding.artifact_reference_reason,
        "raios.module_candidate_artifact_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "vm_test_report",
        binding.vm_report_reference_event_id,
        binding.vm_report_reference_status,
        binding.vm_report_reference_reason,
        "raios.module_vm_test_report_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "computed_capability_grant",
        binding.retained_reference_event_id,
        computed_grant_status(binding),
        computed_grant_reason(binding),
        "raios.computed_capability_grant.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "local_attestation",
        binding.attestation_reference_event_id,
        binding.attestation_reference_status,
        binding.attestation_reference_reason,
        "raios.module_local_attestation_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "local_approval",
        binding.approval_reference_event_id,
        binding.approval_reference_status,
        binding.approval_reference_reason,
        "raios.module_local_approval_reference.v0",
        true,
    );
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
        false,
    );
    raw_line("      },");
    raw_line("      \"hash_inputs\": {");
    let retained = binding.retained_reference;
    let approval = binding.approval_reference;
    let audit = binding.audit_rollback_reference;
    let service_slot = binding.service_slot_reservation;
    raw("        \"manifest_hash\": ");
    json_sha256_option(retained.map(|reference| reference.manifest_hash));
    raw_line(",");
    raw("        \"candidate_artifact_hash\": ");
    json_sha256_option(retained.map(|reference| reference.artifact_hash));
    raw_line(",");
    raw("        \"vm_test_report_hash\": ");
    json_sha256_option(retained.map(|reference| reference.vm_report_hash));
    raw_line(",");
    raw("        \"local_attestation_hash\": ");
    json_sha256_option(retained.map(|reference| reference.local_attestation_hash));
    raw_line(",");
    raw("        \"local_approval_hash\": ");
    json_sha256_option(approval.map(|reference| reference.local_approval_hash));
    raw_line(",");
    raw("        \"computed_capability_grant_hash\": ");
    json_sha256_option(retained.map(|reference| reference.computed_grant_hash));
    raw_line(",");
    raw("        \"audit_record_hash\": ");
    json_sha256_option(audit.map(|reference| reference.audit_record_hash));
    raw_line(",");
    raw("        \"rollback_plan_hash\": ");
    json_sha256_option(audit.map(|reference| reference.rollback_plan_hash));
    raw_line(",");
    raw("        \"pre_load_service_inventory_hash\": ");
    json_sha256_option(audit.map(|reference| reference.pre_load_service_inventory_hash));
    raw_line(",");
    raw("        \"cleanup_actions_hash\": ");
    json_sha256_option(audit.map(|reference| reference.cleanup_actions_hash));
    raw_line(",");
    raw("        \"service_slot_reservation_hash\": ");
    json_sha256_option(service_slot.map(|reservation| reservation.reservation_hash));
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    if let Some(reference) = audit {
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw("null");
    }
    crlf();
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_availability_inputs(
    availability: ModuleAuditRollbackAvailabilityCandidate,
    evaluation: ModuleAuditRollbackAvailabilityEvaluation,
) {
    raw_line("      \"availability_inputs\": {");
    emit_module_write_boundary_availability_input(
        "durable_audit_ledger",
        "raios.durable_audit_ledger.v0",
        availability.durable_audit_ledger,
        evaluation.durable_audit_ledger_status,
        evaluation.durable_audit_ledger_reason,
        true,
    );
    emit_module_write_boundary_availability_input(
        "rollback_store",
        "raios.rollback_store.v0",
        availability.rollback_store,
        evaluation.rollback_store_status,
        evaluation.rollback_store_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_availability_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
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
    raw(", \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_write_boundary_policy_inputs(
    policy: ModuleAuditRollbackWritePolicyCandidate,
    evaluation: ModuleAuditRollbackWritePolicyEvaluation,
) {
    raw_line("      \"policy_inputs\": {");
    emit_module_write_boundary_policy_input(
        "durable_write_policy",
        "raios.durable_audit_write_policy.v0",
        policy.durable_write_policy,
        evaluation.durable_write_policy_status,
        evaluation.durable_write_policy_reason,
        true,
    );
    emit_module_write_boundary_policy_input(
        "rollback_install_policy",
        "raios.rollback_install_policy.v0",
        policy.rollback_install_policy,
        evaluation.rollback_install_policy_status,
        evaluation.rollback_install_policy_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_policy_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackWritePolicyFact,
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
    raw(", \"binds_retained_evidence\": ");
    raw_bool(fact.binds_retained_evidence);
    raw(", \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_write_boundary_append_contract_inputs(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_inputs\": {");
    emit_module_write_boundary_append_contract_input(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_write_boundary_append_contract_input(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_append_contract_input(
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
    raw(", \"binds_write_policy\": ");
    raw_bool(fact.binds_write_policy);
    raw(", \"binds_availability\": ");
    raw_bool(fact.binds_availability);
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
    raw(", \"storage_layout_available\": ");
    raw_bool(fact.storage_layout_available);
    raw(", \"append_engine_available\": ");
    raw_bool(fact.append_engine_available);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_write_boundary_append_payload_hash_inputs(
    payload: ModuleAuditRollbackAppendPayloadHashCandidate,
    evaluation: ModuleAuditRollbackAppendPayloadHashEvaluation,
) {
    raw_line("      \"append_payload_hash_inputs\": {");
    emit_module_write_boundary_append_payload_hash_input(
        "audit_record_append_payload_hash",
        "raios.audit_record_append_payload_hash_envelope.v0",
        payload.audit_record_payload_hash,
        evaluation.audit_payload_status,
        evaluation.audit_payload_reason,
        true,
    );
    emit_module_write_boundary_append_payload_hash_input(
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
    raw_line("        \"payload_hash_envelopes_are_writer_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_append_payload_hash_input(
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
    raw(", \"binds_pre_load_write_request\": ");
    raw_bool(fact.binds_pre_load_write_request);
    raw(", \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw(", \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw(", \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_write_boundary_append_intent_inputs(
    intent: ModuleAuditRollbackAppendIntentCandidate,
    evaluation: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"append_intent_inputs\": {");
    emit_module_write_boundary_append_intent_input(
        "audit_record_append_intent",
        "raios.audit_record_append_intent.v0",
        intent.audit_record_append_intent,
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
        true,
    );
    emit_module_write_boundary_append_intent_input(
        "rollback_transaction_append_intent",
        "raios.rollback_transaction_append_intent.v0",
        intent.rollback_transaction_append_intent,
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
        true,
    );
    raw("        \"append_contract_available\": ");
    raw_bool(evaluation.append_contract_available);
    raw_line(",");
    raw("        \"payload_hash_available\": ");
    raw_bool(evaluation.payload_hash_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(evaluation.append_intent_available);
    raw_line(",");
    raw_line("        \"append_intent_facts_are_writer_authority\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_append_intent_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendIntentFact,
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
    raw(", \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw(", \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw(", \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw(", \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw(", \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw(", \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw(", \"binds_intent_provenance\": ");
    raw_bool(fact.binds_intent_provenance);
    raw(", \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw(", \"payload_hash_available\": ");
    raw_bool(fact.payload_hash_available);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_write_boundary_denial_evidence(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
) {
    raw_line("      \"denial_evidence\": {");
    raw_line("        \"schema\": \"raios.module_audit_rollback_write_denial_evidence.v0\",");
    raw("        \"validation_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw_line("        \"durable_audit_write\": {");
    raw_line("          \"schema\": \"raios.audit_record.v0\",");
    raw("          \"state\": ");
    json_str(evaluation.durable_audit_write_state);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.durable_audit_write_reason);
    raw_line(",");
    raw("          \"ledger\": ");
    json_str(evaluation.durable_audit_write_state);
    raw_line(",");
    raw_line("          \"write_attempted\": false");
    raw_line("        },");
    raw_line("        \"rollback_install\": {");
    raw_line("          \"schema\": \"raios.rollback_plan.v0\",");
    raw("          \"state\": ");
    json_str(evaluation.rollback_install_state);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.rollback_install_reason);
    raw_line(",");
    raw("          \"store\": ");
    json_str(evaluation.rollback_install_state);
    raw_line(",");
    raw_line("          \"install_attempted\": false");
    raw_line("        },");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"recovery_artifact_loading\": \"separate_capability\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

pub(crate) fn emit_module_write_boundary_policy_result(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
    policy: ModuleAuditRollbackWritePolicyEvaluation,
    append: ModuleAuditRollbackAppendContractEvaluation,
    append_payload: ModuleAuditRollbackAppendPayloadHashEvaluation,
    append_intent: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"policy_result\": {");
    raw("        \"preconditions_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"preconditions_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"durable_write_policy_status\": ");
    json_str(policy.durable_write_policy_status);
    raw_line(",");
    raw("        \"durable_write_policy_reason\": ");
    json_str(policy.durable_write_policy_reason);
    raw_line(",");
    raw("        \"rollback_install_policy_status\": ");
    json_str(policy.rollback_install_policy_status);
    raw_line(",");
    raw("        \"rollback_install_policy_reason\": ");
    json_str(policy.rollback_install_policy_reason);
    raw_line(",");
    raw("        \"audit_append_status\": ");
    json_str(append.audit_append_status);
    raw_line(",");
    raw("        \"audit_append_reason\": ");
    json_str(append.audit_append_reason);
    raw_line(",");
    raw("        \"rollback_transaction_status\": ");
    json_str(append.rollback_transaction_status);
    raw_line(",");
    raw("        \"rollback_transaction_reason\": ");
    json_str(append.rollback_transaction_reason);
    raw_line(",");
    raw("        \"audit_append_payload_hash_status\": ");
    json_str(append_payload.audit_payload_status);
    raw_line(",");
    raw("        \"audit_append_payload_hash_reason\": ");
    json_str(append_payload.audit_payload_reason);
    raw_line(",");
    raw("        \"rollback_transaction_append_payload_hash_status\": ");
    json_str(append_payload.rollback_payload_status);
    raw_line(",");
    raw("        \"rollback_transaction_append_payload_hash_reason\": ");
    json_str(append_payload.rollback_payload_reason);
    raw_line(",");
    raw("        \"audit_append_intent_status\": ");
    json_str(append_intent.audit_intent_status);
    raw_line(",");
    raw("        \"audit_append_intent_reason\": ");
    json_str(append_intent.audit_intent_reason);
    raw_line(",");
    raw("        \"rollback_transaction_append_intent_status\": ");
    json_str(append_intent.rollback_intent_status);
    raw_line(",");
    raw("        \"rollback_transaction_append_intent_reason\": ");
    json_str(append_intent.rollback_intent_reason);
    raw_line(",");
    raw_line("        \"durable_audit_written\": false,");
    raw_line("        \"rollback_plan_installed\": false,");
    raw("        \"durable_audit_write_missing\": ");
    raw_bool(method_eq(evaluation.durable_audit_write_state, "missing"));
    raw_line(",");
    raw("        \"rollback_install_missing\": ");
    raw_bool(method_eq(evaluation.rollback_install_state, "missing"));
    raw_line(",");
    raw("        \"durable_write_policy_missing\": ");
    raw_bool(!method_eq(policy.durable_write_policy_status, "available"));
    raw_line(",");
    raw("        \"rollback_install_policy_missing\": ");
    raw_bool(!method_eq(
        policy.rollback_install_policy_status,
        "available",
    ));
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!append.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!append.append_engine_available);
    raw_line(",");
    raw("        \"append_contract_available\": ");
    raw_bool(append_intent.append_contract_available);
    raw_line(",");
    raw("        \"payload_hash_available\": ");
    raw_bool(append_payload.payload_hash_available);
    raw_line(",");
    raw("        \"payload_hash_missing\": ");
    raw_bool(!append_payload.payload_hash_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(append_intent.append_intent_available);
    raw_line(",");
    raw("        \"append_intent_missing\": ");
    raw_bool(!append_intent.append_intent_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_durable_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_authority\": false,");
    raw_line("        \"retained_hash_refs_are_payload_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_intent_authority\": false,");
    raw_line("        \"policy_facts_are_append_authority\": false,");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false,");
    raw_line("        \"payload_hash_envelopes_are_writer_authority\": false,");
    raw_line("        \"append_intent_facts_are_writer_authority\": false,");
    raw_line("        \"recovery_artifact_loading_separate\": true,");
    raw_line("        \"grants_capability\": false,");
    raw_line("        \"grants_load_now\": false,");
    raw_line("        \"authorizes_guest_load\": false,");
    raw("        \"can_load_now\": ");
    raw_bool(evaluation.can_load);
    raw_line(",");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw("        \"load_attempted\": ");
    raw_bool(evaluation.load_attempted);
    crlf();
    raw_line("      }");
}

pub(crate) fn emit_module_audit_rollback_write_boundary_selftest() {
    let cases = module_audit_rollback_write_boundary_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_write_boundary_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_boundary_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
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
        emit_module_audit_rollback_write_boundary_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_write_boundary_selftest");
}

pub(crate) fn emit_module_audit_rollback_write_boundary_selftest_case(
    case: &ModuleAuditRollbackWriteBoundarySelfTestCase,
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
    raw(", \"creates_durable_audit_records\": false, \"installs_rollback_plan\": false, \"loads_artifact\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_write_boundary_candidate_from_binding(
    binding: event_log::ModuleLoadGateBinding,
    availability: ModuleAuditRollbackAvailabilityEvaluation,
    policy: ModuleAuditRollbackWritePolicyEvaluation,
    append: ModuleAuditRollbackAppendContractEvaluation,
    append_intent: ModuleAuditRollbackAppendIntentEvaluation,
) -> ModuleAuditRollbackWriteBoundaryCandidate {
    ModuleAuditRollbackWriteBoundaryCandidate {
        scope: "current_boot",
        request_schema_ok: true,
        manifest_status: binding.manifest_reference_status,
        manifest_reason: binding.manifest_reference_reason,
        artifact_status: binding.artifact_reference_status,
        artifact_reason: binding.artifact_reference_reason,
        vm_report_status: binding.vm_report_reference_status,
        vm_report_reason: binding.vm_report_reference_reason,
        computed_grant_status: computed_grant_status(binding),
        computed_grant_reason: computed_grant_reason(binding),
        local_attestation_status: binding.attestation_reference_status,
        local_attestation_reason: binding.attestation_reference_reason,
        local_approval_status: binding.approval_reference_status,
        local_approval_reason: binding.approval_reference_reason,
        audit_rollback_status: binding.audit_rollback_reference_status,
        audit_rollback_reason: binding.audit_rollback_reference_reason,
        service_slot_status: binding.service_slot_reservation_status,
        service_slot_reason: binding.service_slot_reservation_reason,
        manifest_hash_matches_grant: manifest_hash_matches_grant(binding),
        artifact_hash_matches_grant: artifact_hash_matches_grant(binding),
        vm_report_hash_matches_grant: vm_report_hash_matches_grant(binding),
        local_attestation_hash_matches_grant: local_attestation_hash_matches_grant(binding),
        local_approval_hash_matches_audit: local_approval_hash_matches_audit(binding),
        audit_record_hash_matches_service_slot: audit_record_hash_matches_service_slot(binding),
        rollback_plan_hash_matches_service_slot: rollback_plan_hash_matches_service_slot(binding),
        service_slot_binds_audit_rollback: service_slot_binds_audit_rollback(binding),
        durable_audit_ledger_status: availability.durable_audit_ledger_status,
        durable_audit_ledger_reason: availability.durable_audit_ledger_reason,
        rollback_store_status: availability.rollback_store_status,
        rollback_store_reason: availability.rollback_store_reason,
        durable_write_policy_status: policy.durable_write_policy_status,
        durable_write_policy_reason: policy.durable_write_policy_reason,
        rollback_install_policy_status: policy.rollback_install_policy_status,
        rollback_install_policy_reason: policy.rollback_install_policy_reason,
        audit_append_status: append.audit_append_status,
        audit_append_reason: append.audit_append_reason,
        rollback_transaction_status: append.rollback_transaction_status,
        rollback_transaction_reason: append.rollback_transaction_reason,
        audit_append_intent_status: append_intent.audit_intent_status,
        audit_append_intent_reason: append_intent.audit_intent_reason,
        rollback_transaction_append_intent_status: append_intent.rollback_intent_status,
        rollback_transaction_append_intent_reason: append_intent.rollback_intent_reason,
        recovery_artifact_loader_requested: false,
    }
}

pub(crate) fn evaluate_module_audit_rollback_write_boundary_candidate(
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundaryEvaluation {
    if !method_eq(candidate.scope, "current_boot") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_scope_must_be_current_boot",
            candidate,
        );
    }
    if !candidate.request_schema_ok {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "pre_load_audit_rollback_write_request_schema_mismatch",
            candidate,
        );
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.manifest_status,
        candidate.manifest_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.artifact_status,
        candidate.artifact_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.vm_report_status,
        candidate.vm_report_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.computed_grant_status,
        candidate.computed_grant_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.local_attestation_status,
        candidate.local_attestation_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.local_approval_status,
        candidate.local_approval_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.audit_rollback_status,
        candidate.audit_rollback_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.service_slot_status,
        candidate.service_slot_reason,
        "retained_hash_reference_only_not_allocated",
        candidate,
    ) {
        return evaluation;
    }
    if !candidate.manifest_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_manifest_hash_mismatch",
            candidate,
        );
    }
    if !candidate.artifact_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_artifact_hash_mismatch",
            candidate,
        );
    }
    if !candidate.vm_report_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_vm_test_report_hash_mismatch",
            candidate,
        );
    }
    if !candidate.local_attestation_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_local_attestation_hash_mismatch",
            candidate,
        );
    }
    if !candidate.local_approval_hash_matches_audit {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_local_approval_hash_mismatch",
            candidate,
        );
    }
    if !candidate.audit_record_hash_matches_service_slot {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_audit_record_hash_mismatch",
            candidate,
        );
    }
    if !candidate.rollback_plan_hash_matches_service_slot {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_rollback_plan_hash_mismatch",
            candidate,
        );
    }
    if !candidate.service_slot_binds_audit_rollback {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_service_slot_reference_mismatch",
            candidate,
        );
    }
    if candidate.recovery_artifact_loader_requested {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "recovery_artifact_loading_is_separate",
            candidate,
        );
    }
    if method_eq(candidate.durable_audit_ledger_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.durable_audit_ledger_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_store_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_store_reason,
            candidate,
        );
    }

    let durable_audit_ledger_available =
        method_eq(candidate.durable_audit_ledger_status, "available");
    let rollback_store_available = method_eq(candidate.rollback_store_status, "available");
    if !durable_audit_ledger_available && !rollback_store_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing_and_rollback_install_missing",
            candidate,
        );
    }
    if !durable_audit_ledger_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing",
            candidate,
        );
    }
    if !rollback_store_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "rollback_install_missing",
            candidate,
        );
    }
    if method_eq(candidate.durable_write_policy_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.durable_write_policy_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_install_policy_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_install_policy_reason,
            candidate,
        );
    }
    if !method_eq(candidate.durable_write_policy_status, "available") {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_policy",
            candidate.durable_write_policy_reason,
            candidate,
        );
    }
    if !method_eq(candidate.rollback_install_policy_status, "available") {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_rollback_install_policy",
            candidate.rollback_install_policy_reason,
            candidate,
        );
    }
    if method_eq(candidate.audit_append_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.audit_append_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_transaction_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_transaction_reason,
            candidate,
        );
    }

    let audit_append_available = method_eq(candidate.audit_append_status, "available");
    let rollback_transaction_available =
        method_eq(candidate.rollback_transaction_status, "available");
    if !audit_append_available
        && method_eq(
            candidate.audit_append_reason,
            "audit_append_envelope_missing",
        )
        && !rollback_transaction_available
        && method_eq(
            candidate.rollback_transaction_reason,
            "rollback_transaction_envelope_missing",
        )
    {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            candidate,
        );
    }
    if !audit_append_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            candidate.audit_append_reason,
            candidate,
        );
    }
    if !rollback_transaction_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            candidate.rollback_transaction_reason,
            candidate,
        );
    }
    if method_eq(candidate.audit_append_intent_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.audit_append_intent_reason,
            candidate,
        );
    }
    if method_eq(
        candidate.rollback_transaction_append_intent_status,
        "rejected",
    ) {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_transaction_append_intent_reason,
            candidate,
        );
    }

    let audit_append_intent_available =
        method_eq(candidate.audit_append_intent_status, "available");
    let rollback_transaction_append_intent_available = method_eq(
        candidate.rollback_transaction_append_intent_status,
        "available",
    );
    if !audit_append_intent_available
        && method_eq(
            candidate.audit_append_intent_reason,
            "audit_record_append_intent_missing",
        )
        && !rollback_transaction_append_intent_available
        && method_eq(
            candidate.rollback_transaction_append_intent_reason,
            "rollback_transaction_append_intent_missing",
        )
    {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            candidate,
        );
    }
    if !audit_append_intent_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            candidate.audit_append_intent_reason,
            candidate,
        );
    }
    if !rollback_transaction_append_intent_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            candidate.rollback_transaction_append_intent_reason,
            candidate,
        );
    }
    module_audit_rollback_write_boundary_evaluation(
        "denied_write_path_unimplemented",
        "durable_audit_rollback_writer_unimplemented",
        candidate,
    )
}

pub(crate) fn write_boundary_require_status(
    status: &'static str,
    reason: &'static str,
    expected: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> Option<ModuleAuditRollbackWriteBoundaryEvaluation> {
    if method_eq(status, expected) {
        None
    } else {
        Some(module_audit_rollback_write_boundary_evaluation(
            if method_eq(status, "missing") {
                "missing"
            } else {
                "rejected"
            },
            reason,
            candidate,
        ))
    }
}

pub(crate) fn module_audit_rollback_write_boundary_evaluation(
    status: &'static str,
    reason: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundaryEvaluation {
    let durable_audit_ledger_available =
        method_eq(candidate.durable_audit_ledger_status, "available");
    let rollback_store_available = method_eq(candidate.rollback_store_status, "available");
    let durable_write_policy_available =
        method_eq(candidate.durable_write_policy_status, "available");
    let rollback_install_policy_available =
        method_eq(candidate.rollback_install_policy_status, "available");
    let audit_append_available = method_eq(candidate.audit_append_status, "available");
    let rollback_transaction_available =
        method_eq(candidate.rollback_transaction_status, "available");
    let audit_append_intent_available =
        method_eq(candidate.audit_append_intent_status, "available");
    let rollback_transaction_append_intent_available = method_eq(
        candidate.rollback_transaction_append_intent_status,
        "available",
    );
    ModuleAuditRollbackWriteBoundaryEvaluation {
        status,
        reason,
        durable_audit_write_state: if durable_audit_ledger_available {
            "available_but_write_disabled"
        } else {
            "missing"
        },
        durable_audit_write_reason: if durable_audit_ledger_available
            && !durable_write_policy_available
        {
            candidate.durable_write_policy_reason
        } else if durable_audit_ledger_available && !audit_append_available {
            candidate.audit_append_reason
        } else if durable_audit_ledger_available && !audit_append_intent_available {
            candidate.audit_append_intent_reason
        } else if durable_audit_ledger_available {
            "durable_audit_write_disabled_until_writer_exists"
        } else {
            "durable_audit_write_missing"
        },
        rollback_install_state: if rollback_store_available {
            "available_but_install_disabled"
        } else {
            "missing"
        },
        rollback_install_reason: if rollback_store_available && !rollback_install_policy_available {
            candidate.rollback_install_policy_reason
        } else if rollback_store_available && !rollback_transaction_available {
            candidate.rollback_transaction_reason
        } else if rollback_store_available && !rollback_transaction_append_intent_available {
            candidate.rollback_transaction_append_intent_reason
        } else if rollback_store_available {
            "rollback_install_disabled_until_installer_exists"
        } else {
            "rollback_install_missing"
        },
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn module_audit_rollback_write_boundary_selftest_cases(
) -> [ModuleAuditRollbackWriteBoundarySelfTestCase;
       MODULE_AUDIT_ROLLBACK_WRITE_BOUNDARY_SELFTEST_CASES] {
    let valid = module_audit_rollback_write_boundary_valid_candidate();
    [
        module_audit_rollback_write_boundary_selftest_case(
            "missing_manifest_reference",
            "missing",
            "retained_module_manifest_reference_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                manifest_status: "missing",
                manifest_reason: "retained_module_manifest_reference_missing",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "stale_artifact_reference",
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
            ModuleAuditRollbackWriteBoundaryCandidate {
                artifact_status: "rejected",
                artifact_reason: "retained_candidate_artifact_reference_stale_or_dropped_event_id",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "substituted_vm_report_reference",
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
            ModuleAuditRollbackWriteBoundaryCandidate {
                vm_report_status: "rejected",
                vm_report_reason: "retained_vm_test_report_reference_substituted_record",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "previous_boot_write_request",
            "rejected",
            "write_boundary_scope_must_be_current_boot",
            ModuleAuditRollbackWriteBoundaryCandidate {
                scope: "previous_boot",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "write_request_schema_mismatch",
            "rejected",
            "pre_load_audit_rollback_write_request_schema_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                request_schema_ok: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "missing_computed_grant_reference",
            "missing",
            "retained_computed_grant_reference_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                computed_grant_status: "missing",
                computed_grant_reason: "retained_computed_grant_reference_missing",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "local_attestation_hash_mismatch",
            "rejected",
            "write_boundary_local_attestation_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_attestation_hash_matches_grant: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "local_approval_hash_mismatch",
            "rejected",
            "write_boundary_local_approval_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_approval_hash_matches_audit: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "audit_record_service_slot_hash_mismatch",
            "rejected",
            "write_boundary_audit_record_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                audit_record_hash_matches_service_slot: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "rollback_plan_service_slot_hash_mismatch",
            "rejected",
            "write_boundary_rollback_plan_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_plan_hash_matches_service_slot: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "substituted_service_slot_reference",
            "rejected",
            "write_boundary_service_slot_reference_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                service_slot_binds_audit_rollback: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "recovery_artifact_loader_requested",
            "rejected",
            "recovery_artifact_loading_is_separate",
            ModuleAuditRollbackWriteBoundaryCandidate {
                recovery_artifact_loader_requested: true,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "durable_audit_ledger_available_rollback_store_missing",
            "denied_missing_durable_write_boundary",
            "rollback_install_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "rollback_store_available_durable_audit_ledger_missing",
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "availability_facts_present_policy_still_denied",
            "denied_missing_durable_write_policy",
            "durable_write_policy_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "durable_write_policy_available_rollback_policy_missing",
            "denied_missing_rollback_install_policy",
            "rollback_install_policy_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "policy_facts_available_append_contract_missing",
            "denied_missing_append_contract",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "audit_append_available_rollback_transaction_missing",
            "denied_missing_append_contract",
            "rollback_transaction_envelope_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "append_contract_available_append_intent_missing",
            "denied_missing_append_intent",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                rollback_transaction_status: "available",
                rollback_transaction_reason: "rollback_transaction_envelope_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "append_intent_payload_hash_envelope_missing",
            "denied_missing_append_intent",
            "audit_record_append_payload_hash_envelope_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                rollback_transaction_status: "available",
                rollback_transaction_reason: "rollback_transaction_envelope_available",
                audit_append_intent_status: "missing",
                audit_append_intent_reason: "audit_record_append_payload_hash_envelope_missing",
                rollback_transaction_append_intent_status: "available",
                rollback_transaction_append_intent_reason:
                    "rollback_transaction_append_intent_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "append_intents_available_writer_still_denied",
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                rollback_transaction_status: "available",
                rollback_transaction_reason: "rollback_transaction_envelope_available",
                audit_append_intent_status: "available",
                audit_append_intent_reason: "audit_record_append_intent_available",
                rollback_transaction_append_intent_status: "available",
                rollback_transaction_append_intent_reason:
                    "rollback_transaction_append_intent_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "accepted_current_boot_preconditions_write_still_denied",
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing_and_rollback_install_missing",
            valid,
        ),
    ]
}

pub(crate) fn module_audit_rollback_write_boundary_valid_candidate(
) -> ModuleAuditRollbackWriteBoundaryCandidate {
    ModuleAuditRollbackWriteBoundaryCandidate {
        scope: "current_boot",
        request_schema_ok: true,
        manifest_status: "retained_hash_reference_only",
        manifest_reason: "retained_module_manifest_reference_not_authorizing",
        artifact_status: "retained_hash_reference_only",
        artifact_reason: "retained_candidate_artifact_reference_not_authorizing",
        vm_report_status: "retained_hash_reference_only",
        vm_report_reason: "retained_vm_test_report_reference_not_authorizing",
        computed_grant_status: "retained_hash_reference_only",
        computed_grant_reason: "retained_computed_grant_reference_not_authorizing",
        local_attestation_status: "retained_hash_reference_only",
        local_attestation_reason: "retained_local_attestation_reference_not_authorizing",
        local_approval_status: "retained_hash_reference_only",
        local_approval_reason: "retained_local_approval_reference_not_authorizing",
        audit_rollback_status: "retained_hash_reference_only",
        audit_rollback_reason: "retained_audit_rollback_reference_not_authorizing",
        service_slot_status: "retained_hash_reference_only_not_allocated",
        service_slot_reason: "retained_service_slot_reservation_not_allocated",
        manifest_hash_matches_grant: true,
        artifact_hash_matches_grant: true,
        vm_report_hash_matches_grant: true,
        local_attestation_hash_matches_grant: true,
        local_approval_hash_matches_audit: true,
        audit_record_hash_matches_service_slot: true,
        rollback_plan_hash_matches_service_slot: true,
        service_slot_binds_audit_rollback: true,
        durable_audit_ledger_status: "missing",
        durable_audit_ledger_reason: "durable_audit_ledger_missing",
        rollback_store_status: "missing",
        rollback_store_reason: "rollback_store_missing",
        durable_write_policy_status: "missing",
        durable_write_policy_reason: "durable_write_policy_missing",
        rollback_install_policy_status: "missing",
        rollback_install_policy_reason: "rollback_install_policy_missing",
        audit_append_status: "missing",
        audit_append_reason: "audit_append_envelope_missing",
        rollback_transaction_status: "missing",
        rollback_transaction_reason: "rollback_transaction_envelope_missing",
        audit_append_intent_status: "missing",
        audit_append_intent_reason: "audit_record_append_intent_missing",
        rollback_transaction_append_intent_status: "missing",
        rollback_transaction_append_intent_reason: "rollback_transaction_append_intent_missing",
        recovery_artifact_loader_requested: false,
    }
}

pub(crate) fn module_audit_rollback_write_boundary_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundarySelfTestCase {
    let actual = evaluate_module_audit_rollback_write_boundary_candidate(candidate);
    ModuleAuditRollbackWriteBoundarySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

pub(crate) fn computed_grant_status(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_hash_reference_only"
    } else {
        "missing"
    }
}

pub(crate) fn computed_grant_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_computed_grant_reference_not_authorizing"
    } else {
        "retained_computed_grant_reference_missing"
    }
}

pub(crate) fn manifest_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.manifest_reference, binding.retained_reference) {
        (Some(manifest), Some(grant)) => manifest.manifest_hash == grant.manifest_hash,
        _ => false,
    }
}

pub(crate) fn artifact_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.artifact_reference, binding.retained_reference) {
        (Some(artifact), Some(grant)) => artifact.artifact_hash == grant.artifact_hash,
        _ => false,
    }
}

pub(crate) fn vm_report_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.vm_report_reference, binding.retained_reference) {
        (Some(report), Some(grant)) => report.vm_report_hash == grant.vm_report_hash,
        _ => false,
    }
}

pub(crate) fn local_attestation_hash_matches_grant(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    match (binding.attestation_reference, binding.retained_reference) {
        (Some(attestation), Some(grant)) => {
            attestation.local_attestation_hash == grant.local_attestation_hash
        }
        _ => false,
    }
}

pub(crate) fn local_approval_hash_matches_audit(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.approval_reference, binding.audit_rollback_reference) {
        (Some(approval), Some(audit)) => approval.local_approval_hash == audit.local_approval_hash,
        _ => false,
    }
}

pub(crate) fn audit_record_hash_matches_service_slot(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    match (
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit), Some(reservation)) => {
            audit.audit_record_hash == reservation.audit_record_hash
        }
        _ => false,
    }
}

pub(crate) fn rollback_plan_hash_matches_service_slot(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    match (
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit), Some(reservation)) => {
            audit.rollback_plan_hash == reservation.rollback_plan_hash
        }
        _ => false,
    }
}

pub(crate) fn service_slot_binds_audit_rollback(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (
        binding.audit_rollback_reference_event_id,
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit_event_id), Some(audit), Some(reservation)) => {
            reservation.retained_audit_rollback_reference_event_id == audit_event_id
                && reservation.computed_grant_hash == audit.computed_grant_hash
                && reservation.audit_record_hash == audit.audit_record_hash
                && reservation.rollback_plan_hash == audit.rollback_plan_hash
                && reservation.pre_load_service_inventory_hash
                    == audit.pre_load_service_inventory_hash
                && reservation.ram_only_service_slot_id.as_str()
                    == audit.ram_only_service_slot_id.as_str()
        }
        _ => false,
    }
}
