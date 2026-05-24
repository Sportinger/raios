use crate::{
    agent_protocol_recovery_lifeline_protocol_types::{
        RecoveryLifelineProtocolCheck, RecoveryLifelineProtocolSelfTestCase,
    },
    agent_protocol_support::{crlf, json_event_id, json_sha256, json_str, raw, raw_bool, raw_line},
    event_log,
};

pub(crate) fn emit_recovery_lifeline_protocol_request_state(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    check: &RecoveryLifelineProtocolCheck,
    comma: bool,
) {
    raw_line("      \"retained_recovery_lifeline_request\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw("        \"status\": ");
        json_str(if check.request_chain_valid {
            "retained_current_boot_hash_reference_only"
        } else {
            "rejected"
        });
        raw_line(",");
        raw("        \"reason\": ");
        json_str(check.reason);
        raw_line(",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"hash_reference_only\": true,");
        raw("        \"request_chain_valid\": ");
        raw_bool(check.request_chain_valid);
        raw_line(",");
        raw("        \"can_report_protocol_gaps\": ");
        raw_bool(check.can_report_protocol_gaps);
        raw_line(",");
        raw_line("        \"accepts_lifeline_request_json\": false,");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
        json_event_id(reference.retained_rollback_evidence_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"lifeline_request_reference_hash\": ");
        json_sha256(reference.lifeline_request_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"recovery_lifeline_request_event_id_missing\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"request_chain_valid\": false,");
        raw_line("        \"can_report_protocol_gaps\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_protocol_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_protocol_check(check: &RecoveryLifelineProtocolCheck) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"can_report_protocol_gaps\": ");
    raw_bool(check.can_report_protocol_gaps);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

pub(crate) fn emit_recovery_lifeline_protocol_selftest_case(
    case: &RecoveryLifelineProtocolSelfTestCase,
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
