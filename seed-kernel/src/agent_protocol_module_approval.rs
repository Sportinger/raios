use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, emit_export_gate, end_response,
        json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, method_eq, method_head_eq, parse_current_boot_event_id, parse_sha256_ref, raw,
        raw_bool, raw_fmt, raw_line,
    },
    event_log, module_evidence,
};

pub(crate) fn module_approval_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.approval_diagnostic")
}

pub(crate) fn module_approval_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.approval_diagnostic_selftest")
}

pub(crate) fn emit_module_approval_diagnostic(method: &str) {
    let arg = module_approval_diagnostic_arg(method);
    let check = parse_module_local_approval_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_local_approval_binding_from_check(&check)
            .map(event_log::record_module_local_approval_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_local_approval_reference();

    begin_response("module.approval_diagnostic");
    raw_line("      \"schema\": \"raios.module_local_approval_reference_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_local_attestation_json\": false,");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.approval_diagnostic <local_approval_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_local_attestation_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <local_attestation_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"local_approval_schema\": \"raios.local_approval.v0\",");
    raw_line("        \"local_approval_reference_schema\": \"raios.module_local_approval_reference.v0\",");
    raw_line("        \"local_approval_reference_canonicalization\": \"raios.module_local_approval_reference.canonical.v0\"");
    raw_line("      },");
    emit_module_local_approval_reference_object(&check);
    raw_line(",");
    emit_module_local_approval_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_local_approval_gate_state(&check);
    raw_line(",");
    emit_module_local_approval_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "local_approval", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "missing",
        "durable_audit_write_missing",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "missing",
        "rollback_install_missing",
    );
    emit_export_gate(
        &mut wrote,
        "loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    emit_export_gate(
        &mut wrote,
        "service_slot",
        "unallocated",
        "ram_only_service_slot_unallocated",
    );
    crlf();
    raw_line("      ]");
    end_response("module.approval_diagnostic");
}

pub(crate) fn emit_module_approval_diagnostic_selftest() {
    let cases = module_local_approval_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.approval_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_local_approval_reference_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_local_approval_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_local_attestation_json\": false,");
    raw_line("      \"accepts_local_approval_text\": false,");
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
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_local_approval_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.approval_diagnostic_selftest");
}

fn emit_module_local_approval_reference_object(check: &ModuleLocalApprovalReferenceCheck<'_>) {
    raw_line("      \"local_approval_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_manifest_reference_event_id\": ");
    json_opt_str(check.retained_manifest_reference_event_id);
    raw_line(",");
    raw("        \"retained_candidate_artifact_reference_event_id\": ");
    json_opt_str(check.retained_artifact_reference_event_id);
    raw_line(",");
    raw("        \"retained_vm_test_report_reference_event_id\": ");
    json_opt_str(check.retained_vm_report_reference_event_id);
    raw_line(",");
    raw("        \"retained_local_attestation_reference_event_id\": ");
    json_opt_str(check.retained_local_attestation_reference_event_id);
    raw_line(",");
    raw("        \"retained_computed_grant_reference_event_id\": ");
    json_opt_str(check.retained_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.approval_reference_hash);
    raw_line(",");
    raw("          \"expected_local_approval_reference_hash\": ");
    json_sha256_option(check.expected_approval_reference_hash);
    raw_line(",");
    raw("          \"manifest_reference_hash\": ");
    json_sha256_option(check.manifest_reference_hash);
    raw_line(",");
    raw("          \"artifact_reference_hash\": ");
    json_sha256_option(check.artifact_reference_hash);
    raw_line(",");
    raw("          \"vm_test_report_reference_hash\": ");
    json_sha256_option(check.vm_report_reference_hash);
    raw_line(",");
    raw("          \"local_attestation_reference_hash\": ");
    json_sha256_option(check.local_attestation_reference_hash);
    raw_line(",");
    raw("          \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"computed_capability_grant_hash\": ");
    json_sha256_option(check.computed_grant_hash);
    raw_line(",");
    raw("          \"expected_computed_capability_grant_hash\": ");
    json_sha256_option(check.expected_computed_grant_hash);
    raw_line(",");
    raw("          \"vm_test_report_hash\": ");
    json_sha256_option(check.vm_report_hash);
    raw_line(",");
    raw("          \"local_attestation_hash\": ");
    json_sha256_option(check.local_attestation_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    crlf();
    raw_line("        }");
    raw("      }");
}

fn emit_module_local_approval_retained_reference(
    check: &ModuleLocalApprovalReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleLocalApprovalReference)>,
) {
    raw_line("      \"retained_local_approval_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(module_local_approval_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_local_approval_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_local_approval_text\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"accepts_unsigned_service_code\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("        \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("        \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw_line(",");
        raw("        \"retained_local_attestation_reference_event_id\": ");
        json_event_id(reference.retained_local_attestation_reference_event_id);
        raw_line(",");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw_line(",");
        raw("          \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("          \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("          \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw_line(",");
        raw("          \"local_attestation_reference_hash\": ");
        json_sha256(reference.local_attestation_reference_hash);
        raw_line(",");
        raw("          \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("          \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_local_approval_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_local_approval_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_local_approval_gate_state(check: &ModuleLocalApprovalReferenceCheck<'_>) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    raw_line("      \"gate_state\": {");
    raw_line("        \"module_manifest\": \"hash_reference_only\",");
    raw_line("        \"candidate_artifact\": \"hash_reference_only\",");
    raw_line("        \"vm_test_report\": \"hash_reference_only\",");
    raw_line("        \"local_attestation\": \"hash_reference_only\",");
    raw_line("        \"computed_capability_grant\": \"hash_reference_only\",");
    raw("        \"local_approval\": ");
    json_str(state);
    raw_line(",");
    raw_line("        \"rollback_plan\": \"missing\",");
    raw_line("        \"durable_audit_record\": \"missing\",");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"artifact_loaded\": false,");
    raw_line("        \"service_started\": false,");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"can_load\": false");
    raw("      }");
}

fn emit_module_local_approval_policy_result(check: &ModuleLocalApprovalReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"local_approval_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("        \"local_approval_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_approval_text_or_artifact_bytes\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"raios.audit_record.v0\",");
    raw_line("          \"rollback_plan\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot\"");
    raw_line("        ]");
    raw("      }");
}

fn emit_module_local_approval_selftest_case(case: &ModuleLocalApprovalSelfTestCase, comma: bool) {
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

fn parse_module_local_approval_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleLocalApprovalReferenceCheck<'_> {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return module_local_approval_reference_check(
            ModuleLocalApprovalReferenceInput {
                has_reference: false,
                arity_valid: false,
                scope: "current_boot",
                approval_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_artifact_reference_event_id: None,
                retained_vm_report_reference_event_id: None,
                retained_local_attestation_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                artifact_reference_hash: None,
                vm_report_reference_hash: None,
                local_attestation_reference_hash: None,
                manifest_hash: None,
                artifact_hash: None,
                computed_grant_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
                local_approval_hash: None,
            },
            None,
            None,
            "missing",
            "local_approval_reference_absent",
            false,
        );
    }

    let mut tokens = trimmed.split_whitespace();
    let reference_token = tokens.next();
    let manifest_event_token = tokens.next();
    let artifact_event_token = tokens.next();
    let vm_report_event_token = tokens.next();
    let attestation_event_token = tokens.next();
    let retained_event_token = tokens.next();
    let manifest_reference_token = tokens.next();
    let artifact_reference_token = tokens.next();
    let vm_report_reference_token = tokens.next();
    let attestation_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let grant_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let approval_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let arity_valid = reference_token.is_some()
        && manifest_event_token.is_some()
        && artifact_event_token.is_some()
        && vm_report_event_token.is_some()
        && attestation_event_token.is_some()
        && retained_event_token.is_some()
        && manifest_reference_token.is_some()
        && artifact_reference_token.is_some()
        && vm_report_reference_token.is_some()
        && attestation_reference_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && grant_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && approval_token.is_some()
        && tokens.next().is_none();
    let input = ModuleLocalApprovalReferenceInput {
        has_reference: true,
        arity_valid,
        scope,
        approval_reference_hash: reference_token.and_then(parse_sha256_ref),
        retained_manifest_reference_event_id: manifest_event_token,
        retained_artifact_reference_event_id: artifact_event_token,
        retained_vm_report_reference_event_id: vm_report_event_token,
        retained_local_attestation_reference_event_id: attestation_event_token,
        retained_reference_event_id: retained_event_token,
        manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
        artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
        vm_report_reference_hash: vm_report_reference_token.and_then(parse_sha256_ref),
        local_attestation_reference_hash: attestation_reference_token.and_then(parse_sha256_ref),
        manifest_hash: manifest_token.and_then(parse_sha256_ref),
        artifact_hash: artifact_token.and_then(parse_sha256_ref),
        computed_grant_hash: grant_token.and_then(parse_sha256_ref),
        vm_report_hash: report_token.and_then(parse_sha256_ref),
        local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        local_approval_hash: approval_token.and_then(parse_sha256_ref),
    };
    evaluate_module_local_approval_reference(input, require_live_retained)
}

fn evaluate_module_local_approval_reference<'a>(
    input: ModuleLocalApprovalReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleLocalApprovalReferenceCheck<'a> {
    if !input.has_reference {
        return module_local_approval_reference_check(
            input,
            None,
            None,
            "missing",
            "local_approval_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_local_approval_reference_check(
            input,
            None,
            None,
            "invalid_arity",
            "local_approval_reference_requires_hashes_and_event_ids",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return module_local_approval_reference_check(
            input,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "local_approval_reference_scope_must_be_current_boot",
            false,
        );
    }

    let (
        Some(approval_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_artifact_reference_event_id),
        Some(retained_vm_report_reference_event_id),
        Some(retained_local_attestation_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(artifact_reference_hash),
        Some(vm_report_reference_hash),
        Some(local_attestation_reference_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(computed_grant_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
        Some(local_approval_hash),
    ) = (
        input.approval_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_artifact_reference_event_id,
        input.retained_vm_report_reference_event_id,
        input.retained_local_attestation_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.artifact_reference_hash,
        input.vm_report_reference_hash,
        input.local_attestation_reference_hash,
        input.manifest_hash,
        input.artifact_hash,
        input.computed_grant_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
        input.local_approval_hash,
    )
    else {
        return module_local_approval_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "local_approval_reference_hashes_must_be_sha256_refs",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_approval_reference_hash = computed_module_local_approval_reference_hash(
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id,
        retained_local_attestation_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash,
        artifact_reference_hash,
        vm_report_reference_hash,
        local_attestation_reference_hash,
        manifest_hash,
        artifact_hash,
        computed_grant_hash,
        vm_report_hash,
        local_attestation_hash,
        local_approval_hash,
    );

    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_artifact_reference_event_id) {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_vm_report_reference_event_id) {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_vm_report_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_local_attestation_reference_event_id) {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_local_attestation_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if approval_reference_hash != expected_approval_reference_hash {
        return module_local_approval_reference_check(
            input,
            Some(expected_approval_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_local_approval_reference_hash",
            "local_approval_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_local_approval_live_reference_mismatch(&input) {
            return module_local_approval_reference_check(
                input,
                Some(expected_approval_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_local_approval_reference_check(
        input,
        Some(expected_approval_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "local_approval_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

fn module_local_approval_reference_check<'a>(
    input: ModuleLocalApprovalReferenceInput<'a>,
    expected_approval_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleLocalApprovalReferenceCheck<'a> {
    ModuleLocalApprovalReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        approval_reference_hash: input.approval_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_artifact_reference_event_id: input.retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id: input.retained_vm_report_reference_event_id,
        retained_local_attestation_reference_event_id: input
            .retained_local_attestation_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        artifact_reference_hash: input.artifact_reference_hash,
        vm_report_reference_hash: input.vm_report_reference_hash,
        local_attestation_reference_hash: input.local_attestation_reference_hash,
        manifest_hash: input.manifest_hash,
        artifact_hash: input.artifact_hash,
        computed_grant_hash: input.computed_grant_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        local_approval_hash: input.local_approval_hash,
        expected_approval_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_local_approval_live_reference_mismatch(
    input: &ModuleLocalApprovalReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_artifact_reference_event_id =
        parse_current_boot_event_id(input.retained_artifact_reference_event_id?)?;
    let retained_vm_report_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_report_reference_event_id?)?;
    let retained_local_attestation_reference_event_id =
        parse_current_boot_event_id(input.retained_local_attestation_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;

    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("local_approval_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("local_approval_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash
        || Some(manifest_reference.manifest_hash) != input.manifest_hash
    {
        return Some("local_approval_manifest_reference_hash_mismatch");
    }

    let Some((latest_artifact_event_id, artifact_reference)) =
        event_log::latest_module_candidate_artifact_reference()
    else {
        return Some("local_approval_artifact_reference_missing");
    };
    if latest_artifact_event_id != retained_artifact_reference_event_id {
        return Some("local_approval_artifact_reference_mismatch");
    }
    if Some(artifact_reference.artifact_reference_hash) != input.artifact_reference_hash
        || Some(artifact_reference.manifest_reference_hash) != input.manifest_reference_hash
        || Some(artifact_reference.manifest_hash) != input.manifest_hash
        || Some(artifact_reference.artifact_hash) != input.artifact_hash
        || Some(artifact_reference.local_attestation_hash) != input.local_attestation_hash
    {
        return Some("local_approval_artifact_reference_hash_mismatch");
    }

    let Some((latest_report_event_id, report_reference)) =
        event_log::latest_module_vm_test_report_reference()
    else {
        return Some("local_approval_vm_report_reference_missing");
    };
    if latest_report_event_id != retained_vm_report_reference_event_id {
        return Some("local_approval_vm_report_reference_mismatch");
    }
    if Some(report_reference.report_reference_hash) != input.vm_report_reference_hash
        || Some(report_reference.artifact_reference_hash) != input.artifact_reference_hash
        || Some(report_reference.vm_report_hash) != input.vm_report_hash
        || Some(report_reference.local_attestation_hash) != input.local_attestation_hash
    {
        return Some("local_approval_vm_report_reference_hash_mismatch");
    }

    let Some((latest_attestation_event_id, attestation_reference)) =
        event_log::latest_module_local_attestation_reference()
    else {
        return Some("local_approval_local_attestation_reference_missing");
    };
    if latest_attestation_event_id != retained_local_attestation_reference_event_id {
        return Some("local_approval_local_attestation_reference_mismatch");
    }
    if Some(attestation_reference.attestation_reference_hash)
        != input.local_attestation_reference_hash
        || Some(attestation_reference.vm_report_reference_hash) != input.vm_report_reference_hash
        || Some(attestation_reference.local_attestation_hash) != input.local_attestation_hash
    {
        return Some("local_approval_local_attestation_reference_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("local_approval_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("local_approval_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash
        || Some(retained_reference.manifest_hash) != input.manifest_hash
        || Some(retained_reference.artifact_hash) != input.artifact_hash
        || Some(retained_reference.vm_report_hash) != input.vm_report_hash
        || Some(retained_reference.local_attestation_hash) != input.local_attestation_hash
    {
        return Some("local_approval_computed_grant_hash_mismatch");
    }
    None
}

fn module_local_approval_selftest_cases(
) -> [ModuleLocalApprovalSelfTestCase; MODULE_LOCAL_APPROVAL_SELFTEST_CASES] {
    let manifest_reference_hash =
        computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let artifact_reference_hash = computed_module_candidate_artifact_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let vm_report_reference_hash = computed_module_vm_test_report_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        artifact_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let attestation_reference_hash = computed_module_local_attestation_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        artifact_reference_hash,
        vm_report_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let valid_hash = computed_module_local_approval_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID,
        MODULE_LOCAL_APPROVAL_TEST_RETAINED_ATTESTATION_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        artifact_reference_hash,
        vm_report_reference_hash,
        attestation_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
        MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
    );
    let valid_input = ModuleLocalApprovalReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        approval_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_artifact_reference_event_id: Some(
            MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        ),
        retained_vm_report_reference_event_id: Some(
            MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID,
        ),
        retained_local_attestation_reference_event_id: Some(
            MODULE_LOCAL_APPROVAL_TEST_RETAINED_ATTESTATION_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        artifact_reference_hash: Some(artifact_reference_hash),
        vm_report_reference_hash: Some(vm_report_reference_hash),
        local_attestation_reference_hash: Some(attestation_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        local_approval_hash: Some(MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH),
    };
    [
        module_local_approval_selftest_case(
            "absent_reference",
            "missing",
            "local_approval_reference_absent",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    has_reference: false,
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "accepted_current_boot_approval_still_denied",
            "valid_hash_reference_load_still_denied",
            "local_approval_reference_valid_but_loader_and_evidence_missing",
            evaluate_module_local_approval_reference(valid_input, false),
        ),
        module_local_approval_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "local_approval_reference_scope_must_be_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    scope: "previous_boot",
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "local_approval_reference_hash_mismatch",
            "mismatched_local_approval_reference_hash",
            "local_approval_reference_hash_mismatch",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    approval_reference_hash: Some([0x99; 32]),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "computed_grant_hash_mismatch",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    computed_grant_hash: Some([0xaa; 32]),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "retained_manifest_reference_event_not_current_boot",
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "retained_artifact_reference_event_not_current_boot",
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    retained_artifact_reference_event_id: Some("event.previous_boot.00000028"),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "retained_vm_report_reference_event_not_current_boot",
            "rejected",
            "retained_vm_report_reference_event_id_not_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    retained_vm_report_reference_event_id: Some("event.previous_boot.00000029"),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "retained_local_attestation_reference_event_not_current_boot",
            "rejected",
            "retained_local_attestation_reference_event_id_not_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    retained_local_attestation_reference_event_id: Some(
                        "event.previous_boot.00000030",
                    ),
                    ..valid_input
                },
                false,
            ),
        ),
        module_local_approval_selftest_case(
            "retained_reference_event_not_current_boot",
            "rejected",
            "retained_reference_event_id_not_current_boot",
            evaluate_module_local_approval_reference(
                ModuleLocalApprovalReferenceInput {
                    retained_reference_event_id: Some("event.previous_boot.00000027"),
                    ..valid_input
                },
                false,
            ),
        ),
    ]
}

fn module_local_approval_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleLocalApprovalReferenceCheck<'_>,
) -> ModuleLocalApprovalSelfTestCase {
    ModuleLocalApprovalSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn module_local_approval_binding_from_check(
    check: &ModuleLocalApprovalReferenceCheck<'_>,
) -> Option<event_log::ModuleLocalApprovalReference> {
    Some(event_log::ModuleLocalApprovalReference {
        approval_reference_hash: check.approval_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_artifact_reference_event_id: parse_current_boot_event_id(
            check.retained_artifact_reference_event_id?,
        )?,
        retained_vm_report_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_report_reference_event_id?,
        )?,
        retained_local_attestation_reference_event_id: parse_current_boot_event_id(
            check.retained_local_attestation_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        artifact_reference_hash: check.artifact_reference_hash?,
        vm_report_reference_hash: check.vm_report_reference_hash?,
        local_attestation_reference_hash: check.local_attestation_reference_hash?,
        manifest_hash: check.manifest_hash?,
        artifact_hash: check.artifact_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
        local_approval_hash: check.local_approval_hash?,
    })
}

fn module_local_approval_reference_matches(
    check: &ModuleLocalApprovalReferenceCheck<'_>,
    reference: event_log::ModuleLocalApprovalReference,
) -> bool {
    check.approval_reference_hash == Some(reference.approval_reference_hash)
        && check
            .retained_manifest_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_manifest_reference_event_id)
        && check
            .retained_artifact_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_artifact_reference_event_id)
        && check
            .retained_vm_report_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_report_reference_event_id)
        && check
            .retained_local_attestation_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_attestation_reference_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check.vm_report_reference_hash == Some(reference.vm_report_reference_hash)
        && check.local_attestation_reference_hash
            == Some(reference.local_attestation_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
}

fn module_approval_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.approval_diagnostic") {
        "module.approval_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    module_evidence::computed_module_manifest_reference_hash(manifest_hash)
}

fn computed_module_candidate_artifact_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_candidate_artifact_reference_hash(
        module_evidence::ModuleCandidateArtifactReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            manifest_hash,
            computed_grant_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
        },
    )
}

fn computed_module_vm_test_report_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_artifact_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_vm_test_report_reference_hash(
        module_evidence::ModuleVmTestReportReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            artifact_reference_hash,
            manifest_hash,
            artifact_hash,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
        },
    )
}

fn computed_module_local_attestation_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_artifact_reference_event_id: &str,
    retained_vm_report_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_local_attestation_reference_hash(
        module_evidence::ModuleLocalAttestationReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_vm_report_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            artifact_reference_hash,
            vm_report_reference_hash,
            manifest_hash,
            artifact_hash,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
        },
    )
}

fn computed_module_local_approval_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_artifact_reference_event_id: &str,
    retained_vm_report_reference_event_id: &str,
    retained_local_attestation_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    local_attestation_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    local_approval_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_local_approval_reference_hash(
        module_evidence::ModuleLocalApprovalReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_vm_report_reference_event_id,
            retained_local_attestation_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            artifact_reference_hash,
            vm_report_reference_hash,
            local_attestation_reference_hash,
            manifest_hash,
            artifact_hash,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
            local_approval_hash,
        },
    )
}

fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}
