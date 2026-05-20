use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, end_response, json_event_id, json_event_id_option,
        json_sha256, json_sha256_option, json_str, method_eq, method_head_eq, parse_sha256_ref,
        raw, raw_bool, raw_fmt, raw_line,
    },
    event_log, module_evidence,
};

pub(crate) fn module_grant_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.grant_diagnostic")
        || method_head_eq(method, "module.load_gate_diagnostic")
}

pub(crate) fn module_grant_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.grant_diagnostic_selftest")
        || method_head_eq(method, "module.load_gate_diagnostic_selftest")
}

pub(crate) fn emit_module_grant_diagnostic(method: &str) {
    let arg = module_grant_diagnostic_arg(method);
    let check = parse_module_grant_reference(arg);
    let recorded_event_id = if check.valid {
        module_grant_binding_from_check(&check)
            .map(event_log::record_module_computed_grant_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_computed_grant_reference();

    begin_response("module.grant_diagnostic");
    raw_line("      \"schema\": \"raios.module_computed_grant_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"risk\": \"modify_ram\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\"");
    raw_line("      },");
    raw_line("      \"computed_grant_reference\": {");
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
    raw("        \"computed_capability_grant_hash\": ");
    json_sha256_option(check.grant_hash);
    raw_line(",");
    raw("        \"expected_computed_capability_grant_hash\": ");
    json_sha256_option(check.expected_grant_hash);
    raw_line(",");
    raw("        \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    raw_line(",");
    raw("        \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("        \"vm_test_report_hash\": ");
    json_sha256_option(check.vm_report_hash);
    raw_line(",");
    raw("        \"local_attestation_hash\": ");
    json_sha256_option(check.local_attestation_hash);
    crlf();
    raw_line("      },");
    emit_module_grant_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_grant_gate_state(&check);
    raw_line(",");
    emit_module_grant_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(
            &mut wrote,
            "computed_capability_grant",
            check.status,
            check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "missing",
        "durable_audit_record_missing",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "missing",
        "rollback_plan_missing",
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
    end_response("module.grant_diagnostic");
}

pub(crate) fn emit_module_grant_diagnostic_selftest() {
    let cases = module_grant_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.grant_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_computed_grant_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
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
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_grant_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.grant_diagnostic_selftest");
}

fn emit_module_grant_selftest_case(case: &ModuleGrantSelfTestCase, comma: bool) {
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

fn emit_module_grant_gate_state(check: &ModuleGrantReferenceCheck<'_>) {
    raw_line("      \"gate_state\": {");
    raw_line("        \"module_manifest\": \"hash_reference_only\",");
    raw_line("        \"candidate_artifact\": \"hash_reference_only\",");
    raw_line("        \"vm_test_report\": \"hash_reference_only\",");
    raw_line("        \"local_attestation\": \"hash_reference_only\",");
    raw("        \"computed_capability_grant\": ");
    json_str(if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"local_approval\": \"not_received_by_guest\",");
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

fn emit_module_grant_retained_reference(
    check: &ModuleGrantReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleComputedGrantReference)>,
) {
    raw_line("      \"retained_reference\": {");
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
        raw_bool(module_grant_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"grants_capability\": false,");
        raw_line("        \"grants_load_now\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false,");
        raw_line("        \"hashes\": {");
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
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_computed_grant_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_grant_policy_result(check: &ModuleGrantReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"computed_candidate_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"grants_capability\": false,");
    raw_line("        \"grants_load_now\": false,");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_artifact_bytes\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"in_guest_evidence_retention\",");
    raw_line("          \"raios.audit_record.v0\",");
    raw_line("          \"rollback_plan\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot\"");
    raw_line("        ]");
    raw("      }");
}

fn module_grant_binding_from_check(
    check: &ModuleGrantReferenceCheck<'_>,
) -> Option<event_log::ModuleComputedGrantReference> {
    let (
        Some(computed_grant_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        check.grant_hash,
        check.manifest_hash,
        check.artifact_hash,
        check.vm_report_hash,
        check.local_attestation_hash,
    )
    else {
        return None;
    };
    Some(event_log::ModuleComputedGrantReference {
        computed_grant_hash,
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    })
}

fn module_grant_reference_matches(
    check: &ModuleGrantReferenceCheck<'_>,
    reference: event_log::ModuleComputedGrantReference,
) -> bool {
    check.grant_hash == Some(reference.computed_grant_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

pub(crate) fn module_computed_grant_reference_matches(
    left: event_log::ModuleComputedGrantReference,
    right: event_log::ModuleComputedGrantReference,
) -> bool {
    left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

pub(crate) fn module_computed_grant_reference_hashes_consistent(
    reference: event_log::ModuleComputedGrantReference,
) -> bool {
    reference.computed_grant_hash
        == computed_module_grant_hash(
            reference.manifest_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

fn parse_module_grant_reference(arg: &str) -> ModuleGrantReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return ModuleGrantReferenceCheck {
            has_reference: false,
            arity_valid: true,
            scope: "current_boot",
            grant_hash: None,
            manifest_hash: None,
            artifact_hash: None,
            vm_report_hash: None,
            local_attestation_hash: None,
            expected_grant_hash: None,
            status: "missing",
            reason: "computed_capability_grant_reference_absent",
            valid: false,
        };
    }

    let mut tokens = arg.split_whitespace();
    let grant_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = grant_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && !extra;

    let grant_hash = grant_token.and_then(parse_sha256_ref);
    let manifest_hash = manifest_token.and_then(parse_sha256_ref);
    let artifact_hash = artifact_token.and_then(parse_sha256_ref);
    let vm_report_hash = report_token.and_then(parse_sha256_ref);
    let local_attestation_hash = attestation_token.and_then(parse_sha256_ref);

    evaluate_module_grant_reference(
        true,
        arity_valid,
        scope,
        grant_hash,
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}

fn evaluate_module_grant_reference<'a>(
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    grant_hash: Option<[u8; 32]>,
    manifest_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    vm_report_hash: Option<[u8; 32]>,
    local_attestation_hash: Option<[u8; 32]>,
) -> ModuleGrantReferenceCheck<'a> {
    if !has_reference {
        return ModuleGrantReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            grant_hash,
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
            expected_grant_hash: None,
            status: "missing",
            reason: "computed_capability_grant_reference_absent",
            valid: false,
        };
    }
    if !arity_valid {
        return ModuleGrantReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            grant_hash,
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
            expected_grant_hash: None,
            status: "invalid_reference_arity",
            reason: "computed_grant_reference_requires_five_hashes_and_optional_scope",
            valid: false,
        };
    }
    let (
        Some(grant_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        grant_hash,
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
    else {
        return ModuleGrantReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            grant_hash,
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
            expected_grant_hash: None,
            status: "invalid_hash_reference",
            reason: "all_module_grant_references_must_be_sha256",
            valid: false,
        };
    };
    let expected_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    if !method_eq(scope, "current_boot") {
        return ModuleGrantReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            grant_hash: Some(grant_hash),
            manifest_hash: Some(manifest_hash),
            artifact_hash: Some(artifact_hash),
            vm_report_hash: Some(vm_report_hash),
            local_attestation_hash: Some(local_attestation_hash),
            expected_grant_hash: Some(expected_grant_hash),
            status: "stale_or_non_current_boot_reference",
            reason: "computed_grant_reference_scope_must_be_current_boot",
            valid: false,
        };
    }
    if grant_hash != expected_grant_hash {
        return ModuleGrantReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            grant_hash: Some(grant_hash),
            manifest_hash: Some(manifest_hash),
            artifact_hash: Some(artifact_hash),
            vm_report_hash: Some(vm_report_hash),
            local_attestation_hash: Some(local_attestation_hash),
            expected_grant_hash: Some(expected_grant_hash),
            status: "mismatched_computed_grant_hash",
            reason: "computed_grant_hash_mismatch",
            valid: false,
        };
    }
    ModuleGrantReferenceCheck {
        has_reference,
        arity_valid,
        scope,
        grant_hash: Some(grant_hash),
        manifest_hash: Some(manifest_hash),
        artifact_hash: Some(artifact_hash),
        vm_report_hash: Some(vm_report_hash),
        local_attestation_hash: Some(local_attestation_hash),
        expected_grant_hash: Some(expected_grant_hash),
        status: "valid_hash_reference_load_still_denied",
        reason: "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        valid: true,
    }
}

fn module_grant_selftest_cases() -> [ModuleGrantSelfTestCase; MODULE_GRANT_SELFTEST_CASES] {
    let valid_grant = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let absent =
        evaluate_module_grant_reference(false, true, "current_boot", None, None, None, None, None);
    let valid = evaluate_module_grant_reference(
        true,
        true,
        "current_boot",
        Some(valid_grant),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    );
    let stale = evaluate_module_grant_reference(
        true,
        true,
        "previous_boot",
        Some(valid_grant),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    );
    let mismatch = evaluate_module_grant_reference(
        true,
        true,
        "current_boot",
        Some(valid_grant),
        Some(MODULE_GRANT_MISMATCH_MANIFEST_HASH),
        Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    );
    let unsafe_policy = evaluate_module_grant_reference(
        true,
        true,
        "current_boot",
        Some([0x66; 32]),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    );
    [
        module_grant_selftest_case(
            "absent_reference",
            "missing",
            "computed_capability_grant_reference_absent",
            absent,
        ),
        module_grant_selftest_case(
            "accepted_current_boot_reference_still_denied",
            "valid_hash_reference_load_still_denied",
            "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
            valid,
        ),
        module_grant_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "computed_grant_reference_scope_must_be_current_boot",
            stale,
        ),
        module_grant_selftest_case(
            "mismatched_manifest_hash_reference",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            mismatch,
        ),
        module_grant_selftest_case(
            "grants_load_now_or_wrong_policy_hash",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            unsafe_policy,
        ),
    ]
}

fn module_grant_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual: ModuleGrantReferenceCheck<'_>,
) -> ModuleGrantSelfTestCase {
    ModuleGrantSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !module_grant_check_can_load(&actual),
    }
}

fn module_grant_check_can_load(_check: &ModuleGrantReferenceCheck<'_>) -> bool {
    false
}

fn module_grant_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.grant_diagnostic") {
        "module.grant_diagnostic".len()
    } else if method_head_eq(method, "module.load_gate_diagnostic") {
        "module.load_gate_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
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
