use alloc::vec;

use crate::agent_protocol_support::{
    emit_selftest_case_fields, record_bool as b, record_event_or_null, record_false as no,
    record_field as f, record_inline as inline, record_object as object, record_sha_or_null,
    record_str as s, record_str_or_null, run_selftest_cases_with, CaseSpec,
    SelftestReportField::False,
};
use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_append_contract::*,
    agent_protocol_module_write_boundary_append_intent::*,
    agent_protocol_module_write_boundary_append_payload_hash::*,
    agent_protocol_module_write_boundary_availability::*,
    agent_protocol_module_write_boundary_write_policy::*, agent_protocol_support::*, event_log,
};
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum WriteBoundarySelftestMutation {
    MissingManifestReference,
    StaleArtifactReference,
    SubstitutedVmReportReference,
    PreviousBootWriteRequest,
    WriteRequestSchemaMismatch,
    MissingComputedGrantReference,
    LocalAttestationHashMismatch,
    LocalApprovalHashMismatch,
    AuditRecordServiceSlotHashMismatch,
    RollbackPlanServiceSlotHashMismatch,
    SubstitutedServiceSlotReference,
    RecoveryArtifactLoaderRequested,
    DurableAuditLedgerAvailableRollbackStoreMissing,
    RollbackStoreAvailableDurableAuditLedgerMissing,
    AvailabilityFactsPresentPolicyDenied,
    DurableWritePolicyAvailableRollbackPolicyMissing,
    PolicyFactsAvailableAppendContractMissing,
    AuditAppendAvailableRollbackTransactionMissing,
    AppendContractAvailableAppendIntentMissing,
    AppendIntentPayloadHashEnvelopeMissing,
    AppendIntentsAvailableWriterDenied,
    AcceptedCurrentBootPreconditionsWriteDenied,
}

const fn write_boundary_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: WriteBoundarySelftestMutation,
) -> CaseSpec<WriteBoundarySelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const WRITE_BOUNDARY_CASES: [CaseSpec<WriteBoundarySelftestMutation>;
    MODULE_AUDIT_ROLLBACK_WRITE_BOUNDARY_SELFTEST_CASES] = [
    write_boundary_case(
        "missing_manifest_reference",
        "missing",
        "retained_module_manifest_reference_missing",
        WriteBoundarySelftestMutation::MissingManifestReference,
    ),
    write_boundary_case(
        "stale_artifact_reference",
        "rejected",
        "retained_candidate_artifact_reference_stale_or_dropped_event_id",
        WriteBoundarySelftestMutation::StaleArtifactReference,
    ),
    write_boundary_case(
        "substituted_vm_report_reference",
        "rejected",
        "retained_vm_test_report_reference_substituted_record",
        WriteBoundarySelftestMutation::SubstitutedVmReportReference,
    ),
    write_boundary_case(
        "previous_boot_write_request",
        "rejected",
        "write_boundary_scope_must_be_current_boot",
        WriteBoundarySelftestMutation::PreviousBootWriteRequest,
    ),
    write_boundary_case(
        "write_request_schema_mismatch",
        "rejected",
        "pre_load_audit_rollback_write_request_schema_mismatch",
        WriteBoundarySelftestMutation::WriteRequestSchemaMismatch,
    ),
    write_boundary_case(
        "missing_computed_grant_reference",
        "missing",
        "retained_computed_grant_reference_missing",
        WriteBoundarySelftestMutation::MissingComputedGrantReference,
    ),
    write_boundary_case(
        "local_attestation_hash_mismatch",
        "rejected",
        "write_boundary_local_attestation_hash_mismatch",
        WriteBoundarySelftestMutation::LocalAttestationHashMismatch,
    ),
    write_boundary_case(
        "local_approval_hash_mismatch",
        "rejected",
        "write_boundary_local_approval_hash_mismatch",
        WriteBoundarySelftestMutation::LocalApprovalHashMismatch,
    ),
    write_boundary_case(
        "audit_record_service_slot_hash_mismatch",
        "rejected",
        "write_boundary_audit_record_hash_mismatch",
        WriteBoundarySelftestMutation::AuditRecordServiceSlotHashMismatch,
    ),
    write_boundary_case(
        "rollback_plan_service_slot_hash_mismatch",
        "rejected",
        "write_boundary_rollback_plan_hash_mismatch",
        WriteBoundarySelftestMutation::RollbackPlanServiceSlotHashMismatch,
    ),
    write_boundary_case(
        "substituted_service_slot_reference",
        "rejected",
        "write_boundary_service_slot_reference_mismatch",
        WriteBoundarySelftestMutation::SubstitutedServiceSlotReference,
    ),
    write_boundary_case(
        "recovery_artifact_loader_requested",
        "rejected",
        "recovery_artifact_loading_is_separate",
        WriteBoundarySelftestMutation::RecoveryArtifactLoaderRequested,
    ),
    write_boundary_case(
        "durable_audit_ledger_available_rollback_store_missing",
        "denied_missing_durable_write_boundary",
        "rollback_install_missing",
        WriteBoundarySelftestMutation::DurableAuditLedgerAvailableRollbackStoreMissing,
    ),
    write_boundary_case(
        "rollback_store_available_durable_audit_ledger_missing",
        "denied_missing_durable_write_boundary",
        "durable_audit_write_missing",
        WriteBoundarySelftestMutation::RollbackStoreAvailableDurableAuditLedgerMissing,
    ),
    write_boundary_case(
        "availability_facts_present_policy_still_denied",
        "denied_missing_durable_write_policy",
        "durable_write_policy_missing",
        WriteBoundarySelftestMutation::AvailabilityFactsPresentPolicyDenied,
    ),
    write_boundary_case(
        "durable_write_policy_available_rollback_policy_missing",
        "denied_missing_rollback_install_policy",
        "rollback_install_policy_missing",
        WriteBoundarySelftestMutation::DurableWritePolicyAvailableRollbackPolicyMissing,
    ),
    write_boundary_case(
        "policy_facts_available_append_contract_missing",
        "denied_missing_append_contract",
        "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
        WriteBoundarySelftestMutation::PolicyFactsAvailableAppendContractMissing,
    ),
    write_boundary_case(
        "audit_append_available_rollback_transaction_missing",
        "denied_missing_append_contract",
        "rollback_transaction_envelope_missing",
        WriteBoundarySelftestMutation::AuditAppendAvailableRollbackTransactionMissing,
    ),
    write_boundary_case(
        "append_contract_available_append_intent_missing",
        "denied_missing_append_intent",
        "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
        WriteBoundarySelftestMutation::AppendContractAvailableAppendIntentMissing,
    ),
    write_boundary_case(
        "append_intent_payload_hash_envelope_missing",
        "denied_missing_append_intent",
        "audit_record_append_payload_hash_envelope_missing",
        WriteBoundarySelftestMutation::AppendIntentPayloadHashEnvelopeMissing,
    ),
    write_boundary_case(
        "append_intents_available_writer_still_denied",
        "denied_write_path_unimplemented",
        "durable_audit_rollback_writer_unimplemented",
        WriteBoundarySelftestMutation::AppendIntentsAvailableWriterDenied,
    ),
    write_boundary_case(
        "accepted_current_boot_preconditions_write_still_denied",
        "denied_missing_durable_write_boundary",
        "durable_audit_write_missing_and_rollback_install_missing",
        WriteBoundarySelftestMutation::AcceptedCurrentBootPreconditionsWriteDenied,
    ),
];

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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_write_boundary.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", no()),
            f("global_event_log_mutation", s("none")),
            f("writes_enabled", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("loads_recovery_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        6,
    );
    emit_record_property_line(
        "pre_load_write_request",
        vec![
            f(
                "schema",
                s("raios.module_pre_load_audit_rollback_write_request.v0"),
            ),
            f(
                "canonicalization",
                s("raios.module_pre_load_audit_rollback_write_request.canonical.v0"),
            ),
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("subject", s("agent.session.serial")),
            f("resource", s("live_service_graph")),
            f(
                "requested_writes",
                V::Array(vec![
                    inline(vec![
                        f("target", s("durable_audit_ledger")),
                        f("schema", s("raios.audit_record.v0")),
                    ]),
                    inline(vec![
                        f("target", s("rollback_store")),
                        f("schema", s("raios.rollback_transaction.v0")),
                    ]),
                ]),
            ),
            f(
                "required_retained_references",
                V::Array(vec![
                    s("raios.module_manifest_reference.v0"),
                    s("raios.module_candidate_artifact_reference.v0"),
                    s("raios.module_vm_test_report_reference.v0"),
                    s("raios.computed_capability_grant.v0"),
                    s("raios.module_local_attestation_reference.v0"),
                    s("raios.module_local_approval_reference.v0"),
                    s("raios.module_audit_rollback_reference.v0"),
                    s("raios.module_service_slot_reservation.v0"),
                ]),
            ),
            f("recovery_artifact_loading", s("separate_capability")),
        ],
        true,
    );
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
    let retained = binding.retained_reference;
    let approval = binding.approval_reference;
    let audit = binding.audit_rollback_reference;
    let service_slot = binding.service_slot_reservation;
    let ram_only_service_slot_id = audit.map(|reference| reference.ram_only_service_slot_id);

    emit_record_property_line(
        "retained_reference_inputs",
        vec![
            f(
                "module_manifest",
                module_write_boundary_input_ref_record(
                    binding.manifest_reference_event_id,
                    binding.manifest_reference_status,
                    binding.manifest_reference_reason,
                    "raios.module_manifest_reference.v0",
                ),
            ),
            f(
                "candidate_artifact",
                module_write_boundary_input_ref_record(
                    binding.artifact_reference_event_id,
                    binding.artifact_reference_status,
                    binding.artifact_reference_reason,
                    "raios.module_candidate_artifact_reference.v0",
                ),
            ),
            f(
                "vm_test_report",
                module_write_boundary_input_ref_record(
                    binding.vm_report_reference_event_id,
                    binding.vm_report_reference_status,
                    binding.vm_report_reference_reason,
                    "raios.module_vm_test_report_reference.v0",
                ),
            ),
            f(
                "computed_capability_grant",
                module_write_boundary_input_ref_record(
                    binding.retained_reference_event_id,
                    computed_grant_status(binding),
                    computed_grant_reason(binding),
                    "raios.computed_capability_grant.v0",
                ),
            ),
            f(
                "local_attestation",
                module_write_boundary_input_ref_record(
                    binding.attestation_reference_event_id,
                    binding.attestation_reference_status,
                    binding.attestation_reference_reason,
                    "raios.module_local_attestation_reference.v0",
                ),
            ),
            f(
                "local_approval",
                module_write_boundary_input_ref_record(
                    binding.approval_reference_event_id,
                    binding.approval_reference_status,
                    binding.approval_reference_reason,
                    "raios.module_local_approval_reference.v0",
                ),
            ),
            f(
                "audit_rollback",
                module_write_boundary_input_ref_record(
                    binding.audit_rollback_reference_event_id,
                    binding.audit_rollback_reference_status,
                    binding.audit_rollback_reference_reason,
                    "raios.module_audit_rollback_reference.v0",
                ),
            ),
            f(
                "service_slot_reservation",
                module_write_boundary_input_ref_record(
                    binding.service_slot_reservation_event_id,
                    binding.service_slot_reservation_status,
                    binding.service_slot_reservation_reason,
                    "raios.module_service_slot_reservation.v0",
                ),
            ),
        ],
        true,
    );
    emit_record_property_line(
        "hash_inputs",
        vec![
            f(
                "manifest_hash",
                record_sha_or_null(retained.map(|reference| reference.manifest_hash)),
            ),
            f(
                "candidate_artifact_hash",
                record_sha_or_null(retained.map(|reference| reference.artifact_hash)),
            ),
            f(
                "vm_test_report_hash",
                record_sha_or_null(retained.map(|reference| reference.vm_report_hash)),
            ),
            f(
                "local_attestation_hash",
                record_sha_or_null(retained.map(|reference| reference.local_attestation_hash)),
            ),
            f(
                "local_approval_hash",
                record_sha_or_null(approval.map(|reference| reference.local_approval_hash)),
            ),
            f(
                "computed_capability_grant_hash",
                record_sha_or_null(retained.map(|reference| reference.computed_grant_hash)),
            ),
            f(
                "audit_record_hash",
                record_sha_or_null(audit.map(|reference| reference.audit_record_hash)),
            ),
            f(
                "rollback_plan_hash",
                record_sha_or_null(audit.map(|reference| reference.rollback_plan_hash)),
            ),
            f(
                "pre_load_service_inventory_hash",
                record_sha_or_null(
                    audit.map(|reference| reference.pre_load_service_inventory_hash),
                ),
            ),
            f(
                "cleanup_actions_hash",
                record_sha_or_null(audit.map(|reference| reference.cleanup_actions_hash)),
            ),
            f(
                "service_slot_reservation_hash",
                record_sha_or_null(service_slot.map(|reservation| reservation.reservation_hash)),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(ram_only_service_slot_id.as_ref().map(|id| id.as_str())),
            ),
        ],
        false,
    );
}

fn module_write_boundary_input_ref_record(
    event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    schema: &'static str,
) -> V<'static> {
    inline(vec![
        f("event_id", record_event_or_null(event_id)),
        f("schema", s(schema)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("classification", s("local_only")),
        f("authorizes_guest_load", no()),
    ])
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

#[rustfmt::skip]
pub(crate) fn emit_module_write_boundary_availability_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(name, vec![f("schema", s(schema)), f("status", s(status)), f("reason", s(reason)), f("scope", s(fact.scope)), f("classification", s(fact.classification)), f("present", b(fact.present)), f("provenance_valid", b(fact.provenance_ok)), f("authorizes_write", no())], 8, comma);
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

#[rustfmt::skip]
pub(crate) fn emit_module_write_boundary_policy_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackWritePolicyFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(name, vec![f("schema", s(schema)), f("status", s(status)), f("reason", s(reason)), f("scope", s(fact.scope)), f("classification", s(fact.classification)), f("present", b(fact.present)), f("binds_retained_evidence", b(fact.binds_retained_evidence)), f("binds_availability", b(fact.binds_availability)), f("authorizes_write", no())], 8, comma);
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

#[rustfmt::skip]
pub(crate) fn emit_module_write_boundary_append_contract_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(name, vec![f("schema", s(schema)), f("status", s(status)), f("reason", s(reason)), f("scope", s(fact.scope)), f("classification", s(fact.classification)), f("present", b(fact.present)), f("binds_write_policy", b(fact.binds_write_policy)), f("binds_availability", b(fact.binds_availability)), f("binds_storage_layout_id", b(fact.binds_storage_layout_id)), f("binds_append_engine_id", b(fact.binds_append_engine_id)), f("binds_write_policy_id", b(fact.binds_write_policy_id)), f("binds_availability_id", b(fact.binds_availability_id)), f("binds_envelope_provenance", b(fact.binds_envelope_provenance)), f("storage_layout_available", b(fact.storage_layout_available)), f("append_engine_available", b(fact.append_engine_available)), f("authorizes_write", no())], 8, comma);
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
    emit_record_fields(
        vec![
            f(
                "payload_hash_available",
                b(evaluation.payload_hash_available),
            ),
            f("payload_hash_envelopes_are_writer_authority", no()),
        ],
        8,
    );
    raw_line("      }");
}

#[rustfmt::skip]
pub(crate) fn emit_module_write_boundary_append_payload_hash_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendPayloadHashFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(name, vec![f("schema", s(schema)), f("status", s(status)), f("reason", s(reason)), f("scope", s(fact.scope)), f("classification", s(fact.classification)), f("present", b(fact.present)), f("payload_hash", record_sha_or_null(fact.payload_hash)), f("source_payload_hash", record_sha_or_null(fact.source_payload_hash)), f("binds_pre_load_write_request", b(fact.binds_pre_load_write_request)), f("binds_append_contract_id", b(fact.binds_append_contract_id)), f("binds_payload_hash", b(fact.binds_payload_hash)), f("append_contract_available", b(fact.append_contract_available)), f("authorizes_write", no())], 8, comma);
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
    emit_record_fields(
        vec![
            f(
                "append_contract_available",
                b(evaluation.append_contract_available),
            ),
            f(
                "payload_hash_available",
                b(evaluation.payload_hash_available),
            ),
            f(
                "append_intent_available",
                b(evaluation.append_intent_available),
            ),
            f("append_intent_facts_are_writer_authority", no()),
        ],
        8,
    );
    raw_line("      }");
}

#[rustfmt::skip]
pub(crate) fn emit_module_write_boundary_append_intent_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendIntentFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(name, vec![f("schema", s(schema)), f("status", s(status)), f("reason", s(reason)), f("scope", s(fact.scope)), f("classification", s(fact.classification)), f("present", b(fact.present)), f("binds_append_contract_id", b(fact.binds_append_contract_id)), f("binds_append_engine_id", b(fact.binds_append_engine_id)), f("binds_storage_layout_id", b(fact.binds_storage_layout_id)), f("binds_write_policy_id", b(fact.binds_write_policy_id)), f("binds_availability_id", b(fact.binds_availability_id)), f("binds_payload_hash", b(fact.binds_payload_hash)), f("binds_intent_provenance", b(fact.binds_intent_provenance)), f("append_contract_available", b(fact.append_contract_available)), f("payload_hash_available", b(fact.payload_hash_available)), f("authorizes_write", no())], 8, comma);
}

pub(crate) fn emit_module_write_boundary_denial_evidence(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
) {
    emit_record_property_line(
        "denial_evidence",
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_write_denial_evidence.v0"),
            ),
            f("validation_status", s(evaluation.status)),
            f("validation_reason", s(evaluation.reason)),
            f(
                "durable_audit_write",
                object(vec![
                    f("schema", s("raios.audit_record.v0")),
                    f("state", s(evaluation.durable_audit_write_state)),
                    f("reason", s(evaluation.durable_audit_write_reason)),
                    f("ledger", s(evaluation.durable_audit_write_state)),
                    f("write_attempted", no()),
                ]),
            ),
            f(
                "rollback_install",
                object(vec![
                    f("schema", s("raios.rollback_plan.v0")),
                    f("state", s(evaluation.rollback_install_state)),
                    f("reason", s(evaluation.rollback_install_reason)),
                    f("store", s(evaluation.rollback_install_state)),
                    f("install_attempted", no()),
                ]),
            ),
            f("loads_recovery_artifact", no()),
            f("recovery_artifact_loading", s("separate_capability")),
            f("load_attempted", no()),
        ],
        false,
    );
}

pub(crate) fn emit_module_write_boundary_policy_result(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
    policy: ModuleAuditRollbackWritePolicyEvaluation,
    append: ModuleAuditRollbackAppendContractEvaluation,
    append_payload: ModuleAuditRollbackAppendPayloadHashEvaluation,
    append_intent: ModuleAuditRollbackAppendIntentEvaluation,
) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("preconditions_status", s(evaluation.status)),
            f("preconditions_reason", s(evaluation.reason)),
            f(
                "durable_write_policy_status",
                s(policy.durable_write_policy_status),
            ),
            f(
                "durable_write_policy_reason",
                s(policy.durable_write_policy_reason),
            ),
            f(
                "rollback_install_policy_status",
                s(policy.rollback_install_policy_status),
            ),
            f(
                "rollback_install_policy_reason",
                s(policy.rollback_install_policy_reason),
            ),
            f("audit_append_status", s(append.audit_append_status)),
            f("audit_append_reason", s(append.audit_append_reason)),
            f(
                "rollback_transaction_status",
                s(append.rollback_transaction_status),
            ),
            f(
                "rollback_transaction_reason",
                s(append.rollback_transaction_reason),
            ),
            f(
                "audit_append_payload_hash_status",
                s(append_payload.audit_payload_status),
            ),
            f(
                "audit_append_payload_hash_reason",
                s(append_payload.audit_payload_reason),
            ),
            f(
                "rollback_transaction_append_payload_hash_status",
                s(append_payload.rollback_payload_status),
            ),
            f(
                "rollback_transaction_append_payload_hash_reason",
                s(append_payload.rollback_payload_reason),
            ),
            f(
                "audit_append_intent_status",
                s(append_intent.audit_intent_status),
            ),
            f(
                "audit_append_intent_reason",
                s(append_intent.audit_intent_reason),
            ),
            f(
                "rollback_transaction_append_intent_status",
                s(append_intent.rollback_intent_status),
            ),
            f(
                "rollback_transaction_append_intent_reason",
                s(append_intent.rollback_intent_reason),
            ),
            f("durable_audit_written", no()),
            f("rollback_plan_installed", no()),
            f(
                "durable_audit_write_missing",
                b(method_eq(evaluation.durable_audit_write_state, "missing")),
            ),
            f(
                "rollback_install_missing",
                b(method_eq(evaluation.rollback_install_state, "missing")),
            ),
            f(
                "durable_write_policy_missing",
                b(!method_eq(policy.durable_write_policy_status, "available")),
            ),
            f(
                "rollback_install_policy_missing",
                b(!method_eq(
                    policy.rollback_install_policy_status,
                    "available",
                )),
            ),
            f(
                "storage_layout_missing",
                b(!append.storage_layout_available),
            ),
            f("append_engine_missing", b(!append.append_engine_available)),
            f(
                "append_contract_available",
                b(append_intent.append_contract_available),
            ),
            f(
                "payload_hash_available",
                b(append_payload.payload_hash_available),
            ),
            f(
                "payload_hash_missing",
                b(!append_payload.payload_hash_available),
            ),
            f(
                "append_intent_available",
                b(append_intent.append_intent_available),
            ),
            f(
                "append_intent_missing",
                b(!append_intent.append_intent_available),
            ),
            f("retained_hash_refs_are_durable_authority", no()),
            f("retained_hash_refs_are_append_authority", no()),
            f("retained_hash_refs_are_payload_authority", no()),
            f("retained_hash_refs_are_append_intent_authority", no()),
            f("policy_facts_are_append_authority", no()),
            f("append_contract_facts_are_append_intent_authority", no()),
            f("payload_hash_envelopes_are_writer_authority", no()),
            f("append_intent_facts_are_writer_authority", no()),
            f("recovery_artifact_loading_separate", b(true)),
            f("grants_capability", no()),
            f("grants_load_now", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", b(evaluation.can_load)),
            f("service_inventory_change", s("none")),
            f("load_attempted", b(evaluation.load_attempted)),
        ],
        false,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_write_boundary_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("loads_artifact", no()),
            f("loads_recovery_artifact", no()),
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
    emit_selftest_case_fields(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
        &[
            False("creates_durable_audit_records"),
            False("installs_rollback_plan"),
            False("loads_artifact"),
            False("can_load"),
            False("load_attempted"),
        ],
        comma,
    );
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
    run_selftest_cases_with(
        module_audit_rollback_write_boundary_valid_candidate(),
        &WRITE_BOUNDARY_CASES,
        apply_write_boundary_selftest_case,
        evaluate_write_boundary_selftest_case,
        module_audit_rollback_write_boundary_selftest_case_from_spec,
    )
}

fn apply_write_boundary_selftest_case(
    candidate: &mut ModuleAuditRollbackWriteBoundaryCandidate,
    mutation: WriteBoundarySelftestMutation,
) {
    let valid = module_audit_rollback_write_boundary_valid_candidate();
    *candidate = match mutation {
        WriteBoundarySelftestMutation::MissingManifestReference => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                manifest_status: "missing",
                manifest_reason: "retained_module_manifest_reference_missing",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::StaleArtifactReference => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                artifact_status: "rejected",
                artifact_reason: "retained_candidate_artifact_reference_stale_or_dropped_event_id",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::SubstitutedVmReportReference => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                vm_report_status: "rejected",
                vm_report_reason: "retained_vm_test_report_reference_substituted_record",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::PreviousBootWriteRequest => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                scope: "previous_boot",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::WriteRequestSchemaMismatch => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                request_schema_ok: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::MissingComputedGrantReference => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                computed_grant_status: "missing",
                computed_grant_reason: "retained_computed_grant_reference_missing",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::LocalAttestationHashMismatch => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_attestation_hash_matches_grant: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::LocalApprovalHashMismatch => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_approval_hash_matches_audit: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::AuditRecordServiceSlotHashMismatch => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                audit_record_hash_matches_service_slot: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::RollbackPlanServiceSlotHashMismatch => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_plan_hash_matches_service_slot: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::SubstitutedServiceSlotReference => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                service_slot_binds_audit_rollback: false,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::RecoveryArtifactLoaderRequested => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                recovery_artifact_loader_requested: true,
                ..valid
            }
        }
        WriteBoundarySelftestMutation::DurableAuditLedgerAvailableRollbackStoreMissing => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::RollbackStoreAvailableDurableAuditLedgerMissing => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::AvailabilityFactsPresentPolicyDenied => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::DurableWritePolicyAvailableRollbackPolicyMissing => {
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                ..valid
            }
        }
        WriteBoundarySelftestMutation::PolicyFactsAvailableAppendContractMissing => {
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
            }
        }
        WriteBoundarySelftestMutation::AuditAppendAvailableRollbackTransactionMissing => {
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
            }
        }
        WriteBoundarySelftestMutation::AppendContractAvailableAppendIntentMissing => {
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
            }
        }
        WriteBoundarySelftestMutation::AppendIntentPayloadHashEnvelopeMissing => {
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
            }
        }
        WriteBoundarySelftestMutation::AppendIntentsAvailableWriterDenied => {
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
            }
        }
        WriteBoundarySelftestMutation::AcceptedCurrentBootPreconditionsWriteDenied => valid,
    };
}

fn evaluate_write_boundary_selftest_case(
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
    _require_live_retained: bool,
) -> ModuleAuditRollbackWriteBoundaryEvaluation {
    evaluate_module_audit_rollback_write_boundary_candidate(candidate)
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

fn module_audit_rollback_write_boundary_selftest_case_from_spec(
    spec: &CaseSpec<WriteBoundarySelftestMutation>,
    actual: ModuleAuditRollbackWriteBoundaryEvaluation,
) -> ModuleAuditRollbackWriteBoundarySelfTestCase {
    ModuleAuditRollbackWriteBoundarySelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
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
