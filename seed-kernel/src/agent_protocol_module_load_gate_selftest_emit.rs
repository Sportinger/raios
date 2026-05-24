use crate::{
    agent_protocol_module_load_gate_selftest::{
        module_load_gate_audit_rollback_selftest_cases,
        module_load_gate_loader_runtime_selftest_cases, module_load_gate_retained_selftest_cases,
        module_load_gate_service_slot_selftest_cases,
    },
    agent_protocol_module_load_gate_selftest_reference_cases::{
        module_load_gate_approval_selftest_cases, module_load_gate_artifact_selftest_cases,
        module_load_gate_attestation_selftest_cases, module_load_gate_manifest_selftest_cases,
        module_load_gate_vm_report_selftest_cases,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, end_response, json_str, raw, raw_bool, raw_fmt, raw_line,
    },
};

pub(crate) fn emit_module_load_gate_manifest_selftest() {
    let cases = module_load_gate_manifest_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_manifest_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_manifest_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_manifest_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"manifest_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_manifest_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_manifest_selftest");
}

fn emit_module_load_gate_manifest_selftest_case(
    case: &ModuleLoadGateManifestSelfTestCase,
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
    raw(", \"actual_module_manifest_state\": ");
    json_str(case.actual_module_manifest_state);
    raw(", \"accepted_manifest_hash\": ");
    raw_bool(case.accepted_manifest_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_artifact_selftest() {
    let cases = module_load_gate_artifact_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_artifact_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_artifact_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_candidate_artifact_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"artifact_reference_hash\",");
    raw_line("        \"retained_manifest_reference_event_id\",");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_artifact_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_artifact_selftest");
}

fn emit_module_load_gate_artifact_selftest_case(
    case: &ModuleLoadGateArtifactSelfTestCase,
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
    raw(", \"actual_candidate_artifact_state\": ");
    json_str(case.actual_candidate_artifact_state);
    raw(", \"accepted_artifact_hash\": ");
    raw_bool(case.accepted_artifact_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_vm_report_selftest() {
    let cases = module_load_gate_vm_report_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_vm_report_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_vm_report_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_vm_test_report_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"vm_test_report_reference_hash\",");
    raw_line("        \"retained_manifest_reference_event_id\",");
    raw_line("        \"retained_candidate_artifact_reference_event_id\",");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"artifact_reference_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_vm_report_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_vm_report_selftest");
}

pub(crate) fn emit_module_load_gate_attestation_selftest() {
    let cases = module_load_gate_attestation_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_attestation_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_local_attestation_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_local_attestation_reference_records\": false,");
    raw_line("      \"accepts_local_attestation_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
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
        emit_module_load_gate_attestation_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_attestation_selftest");
}

pub(crate) fn emit_module_load_gate_approval_selftest() {
    let cases = module_load_gate_approval_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_approval_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_local_approval_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_local_approval_reference_records\": false,");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
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
        emit_module_load_gate_approval_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_approval_selftest");
}

fn emit_module_load_gate_attestation_selftest_case(
    case: &ModuleLoadGateLocalAttestationSelfTestCase,
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
    raw(", \"actual_local_attestation_state\": ");
    json_str(case.actual_local_attestation_state);
    raw(", \"accepted_local_attestation_hash\": ");
    raw_bool(case.accepted_local_attestation_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_approval_selftest_case(
    case: &ModuleLoadGateLocalApprovalSelfTestCase,
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
    raw(", \"actual_local_approval_state\": ");
    json_str(case.actual_local_approval_state);
    raw(", \"accepted_local_approval_hash\": ");
    raw_bool(case.accepted_local_approval_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_vm_report_selftest_case(
    case: &ModuleLoadGateVmReportSelfTestCase,
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
    raw(", \"actual_vm_test_report_state\": ");
    json_str(case.actual_vm_test_report_state);
    raw(", \"accepted_vm_test_report_hash\": ");
    raw_bool(case.accepted_vm_report_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_retained_selftest() {
    let cases = module_load_gate_retained_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_retained_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_retained_reference_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_reference_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"unallocated\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_retained_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_retained_selftest");
}

fn emit_module_load_gate_retained_selftest_case(
    case: &ModuleLoadGateRetainedSelfTestCase,
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_audit_rollback_selftest() {
    let cases = module_load_gate_audit_rollback_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_audit_rollback_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_audit_rollback_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"unallocated\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"retained_audit_rollback_reference_event_id\",");
    raw_line("        \"audit_record_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\",");
    raw_line("        \"local_approval\",");
    raw_line("        \"rollback_plan_hash\",");
    raw_line("        \"ram_only_service_slot_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_audit_rollback_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_audit_rollback_selftest");
}

fn emit_module_load_gate_audit_rollback_selftest_case(
    case: &ModuleLoadGateAuditRollbackSelfTestCase,
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_service_slot_selftest() {
    let cases = module_load_gate_service_slot_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_service_slot_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_service_slot_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"non_authorizing\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"retained_audit_rollback_reference_event_id\",");
    raw_line("        \"reservation_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"audit_record_hash\",");
    raw_line("        \"rollback_plan_hash\",");
    raw_line("        \"pre_load_service_inventory_hash\",");
    raw_line("        \"ram_only_service_slot_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_service_slot_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_service_slot_selftest");
}

fn emit_module_load_gate_service_slot_selftest_case(
    case: &ModuleLoadGateServiceSlotSelfTestCase,
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
    raw(", \"actual_service_slot_state\": ");
    json_str(case.actual_service_slot_state);
    raw(", \"accepted_service_slot_reservation_hash\": ");
    raw_bool(case.accepted_service_slot_reservation_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"allocates_service_slot\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_load_gate_loader_runtime_selftest() {
    let cases = module_load_gate_loader_runtime_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_loader_runtime_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_loader_runtime_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"service_slot_allocator_ready\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"retained_module_evidence\",");
    raw_line("        \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("        \"raios.module_loader_identity.v0\",");
    raw_line("        \"raios.module_loader_artifact_hash_binding.v0\",");
    raw_line("        \"raios.module_loader_entrypoint_abi.v0\",");
    raw_line("        \"raios.module_loader_address_space_boundary.v0\",");
    raw_line("        \"raios.module_loader_memory_map_constraints.v0\",");
    raw_line("        \"raios.module_loader_capability_import_table.v0\",");
    raw_line("        \"raios.module_loader_service_slot_binding.v0\",");
    raw_line("        \"raios.module_loader_health_state_hooks.v0\",");
    raw_line("        \"raios.module_loader_rollback_hooks.v0\",");
    raw_line("        \"raios.module_loader_audit_rollback_write_boundary_binding.v0\"");
    raw_line("      ],");
    raw_line("      \"missing_runtime_facts\": [");
    raw_line("        \"raios.module_loader_identity.v0\",");
    raw_line("        \"raios.module_loader_artifact_hash_binding.v0\",");
    raw_line("        \"raios.module_loader_entrypoint_abi.v0\",");
    raw_line("        \"raios.module_loader_address_space_boundary.v0\",");
    raw_line("        \"raios.module_loader_memory_map_constraints.v0\",");
    raw_line("        \"raios.module_loader_capability_import_table.v0\",");
    raw_line("        \"raios.module_loader_service_slot_binding.v0\",");
    raw_line("        \"raios.module_loader_health_state_hooks.v0\",");
    raw_line("        \"raios.module_loader_rollback_hooks.v0\",");
    raw_line("        \"raios.module_loader_audit_rollback_write_boundary_binding.v0\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_loader_runtime_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_loader_runtime_selftest");
}

fn emit_module_load_gate_loader_runtime_selftest_case(
    case: &ModuleLoadGateLoaderRuntimeSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"expected_retained_module_evidence_state\": ");
    json_str(case.expected_retained_module_evidence_state);
    raw(", \"expected_service_slot_allocator_state\": ");
    json_str(case.expected_service_slot_allocator_state);
    raw(", \"expected_loader_runtime_state\": ");
    json_str(case.expected_loader_runtime_state);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"actual_retained_module_evidence_state\": ");
    json_str(case.actual_retained_module_evidence_state);
    raw(", \"actual_retained_module_evidence_reason\": ");
    json_str(case.actual_retained_module_evidence_reason);
    raw(", \"actual_service_slot_allocator_state\": ");
    json_str(case.actual_service_slot_allocator_state);
    raw(", \"actual_service_slot_allocator_status\": ");
    json_str(case.actual_service_slot_allocator_status);
    raw(", \"actual_service_slot_allocator_reason\": ");
    json_str(case.actual_service_slot_allocator_reason);
    raw(", \"actual_loader_runtime_state\": ");
    json_str(case.actual_loader_runtime_state);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
