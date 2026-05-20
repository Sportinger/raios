use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, emit_export_gate, end_response,
        json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, method_eq, method_head_eq, parse_current_boot_event_id, parse_sha256_ref, raw,
        raw_bool, raw_fmt, raw_line,
    },
    event_log,
    module_evidence::{self, ram_only_service_slot_id_valid, ModuleAuditRecordHashInput},
};
pub(crate) fn module_audit_rollback_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_diagnostic")
        || method_head_eq(method, "module.audit_rollback_gate_diagnostic")
}

pub(crate) fn module_audit_rollback_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_diagnostic_selftest")
}

fn module_audit_rollback_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.audit_rollback_diagnostic") {
        "module.audit_rollback_diagnostic".len()
    } else if method_head_eq(method, "module.audit_rollback_gate_diagnostic") {
        "module.audit_rollback_gate_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

pub(crate) fn emit_module_audit_rollback_diagnostic(method: &str) {
    let arg = module_audit_rollback_diagnostic_arg(method);
    let check = parse_module_audit_rollback_reference(arg);
    let recorded_event_id = if check.valid {
        module_audit_rollback_binding_from_check(&check)
            .map(event_log::record_module_audit_rollback_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_audit_rollback_reference();

    begin_response("module.audit_rollback_diagnostic");
    raw_line("      \"schema\": \"raios.module_audit_rollback_reference_diagnostic.v0\",");
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
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"audit_record_schema\": \"raios.audit_record.v0\",");
    raw_line("        \"audit_record_canonicalization\": \"raios.audit_record.canonical.v0\",");
    raw_line("        \"rollback_plan_schema\": \"raios.rollback_plan.v0\",");
    raw_line("        \"rollback_plan_canonicalization\": \"raios.rollback_plan.canonical.v0\"");
    raw_line("      },");
    emit_module_audit_rollback_reference_object(&check);
    raw_line(",");
    emit_module_audit_rollback_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_audit_rollback_gate_state(&check);
    raw_line(",");
    emit_module_audit_rollback_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(
            &mut wrote,
            "audit_rollback_reference",
            check.status,
            check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "hash_reference_only_not_durable",
        "durable_audit_write_not_enabled",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "hash_reference_only_not_installed",
        "rollback_plan_install_not_enabled",
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
    end_response("module.audit_rollback_diagnostic");
}

fn emit_module_audit_rollback_reference_object(check: &ModuleAuditRollbackReferenceCheck<'_>) {
    raw_line("      \"audit_rollback_reference\": {");
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
    raw("        \"denial_event_id\": ");
    json_opt_str(check.denial_event_id);
    raw_line(",");
    raw("        \"retained_reference_event_id\": ");
    json_opt_str(check.retained_reference_event_id);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_opt_str(check.ram_only_service_slot_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"audit_record_hash\": ");
    json_sha256_option(check.audit_record_hash);
    raw_line(",");
    raw("          \"expected_audit_record_hash\": ");
    json_sha256_option(check.expected_audit_record_hash);
    raw_line(",");
    raw("          \"rollback_plan_hash\": ");
    json_sha256_option(check.rollback_plan_hash);
    raw_line(",");
    raw("          \"expected_rollback_plan_hash\": ");
    json_sha256_option(check.expected_rollback_plan_hash);
    raw_line(",");
    raw("          \"computed_capability_grant_hash\": ");
    json_sha256_option(check.computed_grant_hash);
    raw_line(",");
    raw("          \"expected_computed_capability_grant_hash\": ");
    json_sha256_option(check.expected_computed_grant_hash);
    raw_line(",");
    raw("          \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"vm_test_report_hash\": ");
    json_sha256_option(check.vm_report_hash);
    raw_line(",");
    raw("          \"local_attestation_hash\": ");
    json_sha256_option(check.local_attestation_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"pre_load_service_inventory_hash\": ");
    json_sha256_option(check.pre_load_service_inventory_hash);
    raw_line(",");
    raw("          \"cleanup_actions_hash\": ");
    json_sha256_option(check.cleanup_actions_hash);
    crlf();
    raw_line("        }");
    raw("      }");
}

fn emit_module_audit_rollback_retained_reference(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleAuditRollbackReference)>,
) {
    raw_line("      \"retained_audit_rollback_reference\": {");
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
        raw_bool(module_audit_rollback_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"durable_audit_written\": false,");
        raw_line("        \"rollback_plan_installed\": false,");
        raw_line("        \"grants_capability\": false,");
        raw_line("        \"grants_load_now\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false,");
        raw("        \"denial_event_id\": ");
        json_event_id(reference.denial_event_id);
        raw_line(",");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw("        \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("          \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("          \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("          \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_audit_rollback_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_audit_rollback_gate_state(check: &ModuleAuditRollbackReferenceCheck<'_>) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    raw_line("      \"gate_state\": {");
    raw("        \"retained_computed_grant_reference\": ");
    json_str(if check.valid {
        "hash_reference_supplied"
    } else {
        state
    });
    raw_line(",");
    raw("        \"computed_capability_grant\": ");
    json_str(state);
    raw_line(",");
    raw_line("        \"module_manifest\": \"hash_reference_only\",");
    raw_line("        \"candidate_artifact\": \"hash_reference_only\",");
    raw_line("        \"vm_test_report\": \"hash_reference_only\",");
    raw_line("        \"local_attestation\": \"hash_reference_only\",");
    raw("        \"durable_audit_record\": ");
    json_str(if check.valid {
        "hash_reference_valid_not_durable"
    } else {
        state
    });
    raw_line(",");
    raw("        \"rollback_plan\": ");
    json_str(if check.valid {
        "hash_reference_valid_not_installed"
    } else {
        state
    });
    raw_line(",");
    raw_line("        \"local_approval\": \"hash_reference_only\",");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"artifact_loaded\": false,");
    raw_line("        \"service_started\": false,");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"can_load\": false");
    raw("      }");
}

fn emit_module_audit_rollback_policy_result(check: &ModuleAuditRollbackReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"audit_record_hash_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("        \"rollback_plan_hash_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_artifact_bytes\",");
    raw_line("        \"durable_audit_written\": false,");
    raw_line("        \"rollback_plan_installed\": false,");
    raw_line("        \"grants_capability\": false,");
    raw_line("        \"grants_load_now\": false,");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"durable_audit_record_write\",");
    raw_line("          \"rollback_plan_installation\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot_allocation\"");
    raw_line("        ]");
    raw("      }");
}

pub(crate) fn emit_module_audit_rollback_diagnostic_selftest() {
    let cases = module_audit_rollback_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_reference_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_audit_rollback_reference_records\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
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
    raw_line("        \"audit_record_hash\",");
    raw_line("        \"rollback_plan_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"retained_reference_event_id\",");
    raw_line("        \"denial_event_id\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\",");
    raw_line("        \"local_approval_hash\",");
    raw_line("        \"pre_load_service_inventory_hash\",");
    raw_line("        \"cleanup_actions_hash\",");
    raw_line("        \"ram_only_service_slot_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_audit_rollback_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_diagnostic_selftest");
}

fn emit_module_audit_rollback_selftest_case(case: &ModuleAuditRollbackSelfTestCase, comma: bool) {
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

fn parse_module_audit_rollback_reference(arg: &str) -> ModuleAuditRollbackReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_audit_rollback_reference(ModuleAuditRollbackReferenceInput {
            has_reference: false,
            arity_valid: true,
            scope: "current_boot",
            audit_schema_ok: true,
            rollback_schema_ok: true,
            audit_record_hash: None,
            rollback_plan_hash: None,
            computed_grant_hash: None,
            manifest_hash: None,
            artifact_hash: None,
            vm_report_hash: None,
            local_attestation_hash: None,
            local_approval_hash: None,
            pre_load_service_inventory_hash: None,
            cleanup_actions_hash: None,
            denial_event_id: None,
            retained_reference_event_id: None,
            ram_only_service_slot_id: None,
        });
    }

    let mut tokens = arg.split_whitespace();
    let audit_token = tokens.next();
    let rollback_token = tokens.next();
    let grant_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let approval_token = tokens.next();
    let inventory_token = tokens.next();
    let cleanup_token = tokens.next();
    let denial_event_id = tokens.next();
    let retained_reference_event_id = tokens.next();
    let ram_only_service_slot_id = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = audit_token.is_some()
        && rollback_token.is_some()
        && grant_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && approval_token.is_some()
        && inventory_token.is_some()
        && cleanup_token.is_some()
        && denial_event_id.is_some()
        && retained_reference_event_id.is_some()
        && ram_only_service_slot_id.is_some()
        && !extra;

    evaluate_module_audit_rollback_reference(ModuleAuditRollbackReferenceInput {
        has_reference: true,
        arity_valid,
        scope,
        audit_schema_ok: true,
        rollback_schema_ok: true,
        audit_record_hash: audit_token.and_then(parse_sha256_ref),
        rollback_plan_hash: rollback_token.and_then(parse_sha256_ref),
        computed_grant_hash: grant_token.and_then(parse_sha256_ref),
        manifest_hash: manifest_token.and_then(parse_sha256_ref),
        artifact_hash: artifact_token.and_then(parse_sha256_ref),
        vm_report_hash: report_token.and_then(parse_sha256_ref),
        local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        local_approval_hash: approval_token.and_then(parse_sha256_ref),
        pre_load_service_inventory_hash: inventory_token.and_then(parse_sha256_ref),
        cleanup_actions_hash: cleanup_token.and_then(parse_sha256_ref),
        denial_event_id,
        retained_reference_event_id,
        ram_only_service_slot_id,
    })
}

fn evaluate_module_audit_rollback_reference<'a>(
    input: ModuleAuditRollbackReferenceInput<'a>,
) -> ModuleAuditRollbackReferenceCheck<'a> {
    if !input.has_reference {
        return module_audit_rollback_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            "audit_rollback_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_audit_rollback_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference_arity",
            "audit_rollback_reference_requires_hashes_events_slot_and_optional_scope",
            false,
        );
    }

    let (
        Some(audit_record_hash),
        Some(rollback_plan_hash),
        Some(computed_grant_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
        Some(local_approval_hash),
        Some(pre_load_service_inventory_hash),
        Some(cleanup_actions_hash),
        Some(denial_event_id),
        Some(retained_reference_event_id),
        Some(ram_only_service_slot_id),
    ) = (
        input.audit_record_hash,
        input.rollback_plan_hash,
        input.computed_grant_hash,
        input.manifest_hash,
        input.artifact_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
        input.local_approval_hash,
        input.pre_load_service_inventory_hash,
        input.cleanup_actions_hash,
        input.denial_event_id,
        input.retained_reference_event_id,
        input.ram_only_service_slot_id,
    )
    else {
        return module_audit_rollback_reference_check(
            input,
            None,
            None,
            None,
            "invalid_hash_reference",
            "all_audit_rollback_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_rollback_plan_hash = computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    );
    let expected_audit_record_hash =
        computed_module_audit_record_hash(ModuleAuditRecordHashInput {
            denial_event_id,
            retained_reference_event_id,
            computed_grant_hash: expected_computed_grant_hash,
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
            local_approval_hash,
            rollback_plan_hash: expected_rollback_plan_hash,
            ram_only_service_slot_id,
        });

    if !method_eq(input.scope, "current_boot") {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "stale_or_non_current_boot_reference",
            "audit_rollback_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(denial_event_id) {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "rejected",
            "denial_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if !ram_only_service_slot_id_valid(ram_only_service_slot_id) {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "rejected",
            "ram_only_service_slot_id_invalid",
            false,
        );
    }
    if !input.audit_schema_ok {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "rejected",
            "audit_record_schema_mismatch",
            false,
        );
    }
    if !input.rollback_schema_ok {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "rejected",
            "rollback_plan_schema_mismatch",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if rollback_plan_hash != expected_rollback_plan_hash {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "mismatched_rollback_plan_hash",
            "rollback_plan_hash_mismatch",
            false,
        );
    }
    if audit_record_hash != expected_audit_record_hash {
        return module_audit_rollback_reference_check(
            input,
            Some(expected_computed_grant_hash),
            Some(expected_rollback_plan_hash),
            Some(expected_audit_record_hash),
            "mismatched_audit_record_hash",
            "audit_record_hash_mismatch",
            false,
        );
    }

    module_audit_rollback_reference_check(
        input,
        Some(expected_computed_grant_hash),
        Some(expected_rollback_plan_hash),
        Some(expected_audit_record_hash),
        "valid_hash_reference_load_still_denied",
        "audit_rollback_reference_valid_but_loader_and_slot_missing",
        true,
    )
}

fn module_audit_rollback_reference_check<'a>(
    input: ModuleAuditRollbackReferenceInput<'a>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    expected_rollback_plan_hash: Option<[u8; 32]>,
    expected_audit_record_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleAuditRollbackReferenceCheck<'a> {
    ModuleAuditRollbackReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        audit_record_hash: input.audit_record_hash,
        rollback_plan_hash: input.rollback_plan_hash,
        computed_grant_hash: input.computed_grant_hash,
        manifest_hash: input.manifest_hash,
        artifact_hash: input.artifact_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        local_approval_hash: input.local_approval_hash,
        pre_load_service_inventory_hash: input.pre_load_service_inventory_hash,
        cleanup_actions_hash: input.cleanup_actions_hash,
        denial_event_id: input.denial_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        ram_only_service_slot_id: input.ram_only_service_slot_id,
        expected_computed_grant_hash,
        expected_rollback_plan_hash,
        expected_audit_record_hash,
        status,
        reason,
        valid,
    }
}

fn module_audit_rollback_selftest_cases(
) -> [ModuleAuditRollbackSelfTestCase; MODULE_AUDIT_ROLLBACK_SELFTEST_CASES] {
    let valid = module_audit_rollback_valid_input();
    [
        module_audit_rollback_selftest_case(
            "absent_reference",
            "missing",
            "audit_rollback_reference_absent",
            ModuleAuditRollbackReferenceInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                audit_schema_ok: true,
                rollback_schema_ok: true,
                audit_record_hash: None,
                rollback_plan_hash: None,
                computed_grant_hash: None,
                manifest_hash: None,
                artifact_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
                local_approval_hash: None,
                pre_load_service_inventory_hash: None,
                cleanup_actions_hash: None,
                denial_event_id: None,
                retained_reference_event_id: None,
                ram_only_service_slot_id: None,
            },
        ),
        module_audit_rollback_selftest_case(
            "accepted_current_boot_reference_still_denied",
            "valid_hash_reference_load_still_denied",
            "audit_rollback_reference_valid_but_loader_and_slot_missing",
            valid,
        ),
        module_audit_rollback_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "audit_rollback_reference_scope_must_be_current_boot",
            ModuleAuditRollbackReferenceInput {
                scope: "previous_boot",
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "previous_boot_denial_event_id",
            "rejected",
            "denial_event_id_not_current_boot",
            ModuleAuditRollbackReferenceInput {
                denial_event_id: Some("event.previous_boot.00000031"),
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "audit_record_schema_mismatch",
            "rejected",
            "audit_record_schema_mismatch",
            ModuleAuditRollbackReferenceInput {
                audit_schema_ok: false,
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "rollback_plan_schema_mismatch",
            "rejected",
            "rollback_plan_schema_mismatch",
            ModuleAuditRollbackReferenceInput {
                rollback_schema_ok: false,
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "substituted_audit_record_hash",
            "mismatched_audit_record_hash",
            "audit_record_hash_mismatch",
            ModuleAuditRollbackReferenceInput {
                audit_record_hash: Some([0x99; 32]),
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "mismatched_rollback_plan_hash",
            "mismatched_rollback_plan_hash",
            "rollback_plan_hash_mismatch",
            ModuleAuditRollbackReferenceInput {
                rollback_plan_hash: Some([0xaa; 32]),
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "mismatched_computed_grant_hash",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            ModuleAuditRollbackReferenceInput {
                computed_grant_hash: Some([0xbb; 32]),
                ..valid
            },
        ),
        module_audit_rollback_selftest_case(
            "invalid_ram_only_service_slot",
            "rejected",
            "ram_only_service_slot_id_invalid",
            ModuleAuditRollbackReferenceInput {
                ram_only_service_slot_id: Some("svc.test.0001"),
                ..valid
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_valid_input<'a>() -> ModuleAuditRollbackReferenceInput<'a> {
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let rollback_plan_hash = computed_module_rollback_plan_hash(
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
        MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
        MODULE_AUDIT_TEST_CLEANUP_HASH,
    );
    let audit_record_hash = computed_module_audit_record_hash(ModuleAuditRecordHashInput {
        denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
        retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        computed_grant_hash,
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        rollback_plan_hash,
        ram_only_service_slot_id: MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
    });
    ModuleAuditRollbackReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        audit_schema_ok: true,
        rollback_schema_ok: true,
        audit_record_hash: Some(audit_record_hash),
        rollback_plan_hash: Some(rollback_plan_hash),
        computed_grant_hash: Some(computed_grant_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        local_approval_hash: Some(MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH),
        pre_load_service_inventory_hash: Some(MODULE_AUDIT_TEST_PRE_INVENTORY_HASH),
        cleanup_actions_hash: Some(MODULE_AUDIT_TEST_CLEANUP_HASH),
        denial_event_id: Some(MODULE_AUDIT_TEST_DENIAL_EVENT_ID),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        ram_only_service_slot_id: Some(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID),
    }
}

fn module_audit_rollback_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackReferenceInput<'_>,
) -> ModuleAuditRollbackSelfTestCase {
    let actual = evaluate_module_audit_rollback_reference(candidate);
    ModuleAuditRollbackSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !module_audit_rollback_check_can_load(&actual),
    }
}

fn module_audit_rollback_check_can_load(_check: &ModuleAuditRollbackReferenceCheck<'_>) -> bool {
    false
}

fn module_audit_rollback_binding_from_check(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
) -> Option<event_log::ModuleAuditRollbackReference> {
    let (
        Some(audit_record_hash),
        Some(rollback_plan_hash),
        Some(computed_grant_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
        Some(local_approval_hash),
        Some(pre_load_service_inventory_hash),
        Some(cleanup_actions_hash),
        Some(denial_event_id),
        Some(retained_reference_event_id),
        Some(ram_only_service_slot_id),
    ) = (
        check.audit_record_hash,
        check.rollback_plan_hash,
        check.computed_grant_hash,
        check.manifest_hash,
        check.artifact_hash,
        check.vm_report_hash,
        check.local_attestation_hash,
        check.local_approval_hash,
        check.pre_load_service_inventory_hash,
        check.cleanup_actions_hash,
        check.denial_event_id,
        check.retained_reference_event_id,
        check.ram_only_service_slot_id,
    )
    else {
        return None;
    };
    Some(event_log::ModuleAuditRollbackReference {
        audit_record_hash,
        rollback_plan_hash,
        computed_grant_hash,
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
        local_approval_hash,
        pre_load_service_inventory_hash,
        cleanup_actions_hash,
        denial_event_id: parse_current_boot_event_id(denial_event_id)?,
        retained_reference_event_id: parse_current_boot_event_id(retained_reference_event_id)?,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(ram_only_service_slot_id)?,
    })
}

fn module_audit_rollback_reference_matches(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
    reference: event_log::ModuleAuditRollbackReference,
) -> bool {
    check.audit_record_hash == Some(reference.audit_record_hash)
        && check.rollback_plan_hash == Some(reference.rollback_plan_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.pre_load_service_inventory_hash == Some(reference.pre_load_service_inventory_hash)
        && check.cleanup_actions_hash == Some(reference.cleanup_actions_hash)
        && check.denial_event_id.and_then(parse_current_boot_event_id)
            == Some(reference.denial_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.ram_only_service_slot_id == Some(reference.ram_only_service_slot_id.as_str())
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

fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    )
}

fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    module_evidence::computed_module_audit_record_hash(input)
}
