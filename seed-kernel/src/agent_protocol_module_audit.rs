use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, emit_export_gate,
        emit_record_fields_trailing_comma, emit_record_property_line,
        emit_record_value_property_line, end_response, method_eq, method_head_eq,
        parse_current_boot_event_id, parse_sha256_ref, raw_line, record_bool as b,
        record_event_or_null, record_false as no, record_field as f, record_sha_fields,
        record_sha_or_null_fields, record_static_str_array, record_str as s, record_str_or_null,
    },
    event_log,
    module_evidence::{self, ram_only_service_slot_id_valid, ModuleAuditRecordHashInput},
};
use raios_core::record::Value as V;
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_reference_diagnostic.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", b(check.valid)),
            f(
                "global_event_log_mutation",
                s(if check.valid {
                    "valid_hash_reference_retention_only"
                } else {
                    "none"
                }),
            ),
            f("accepts_artifact_bytes", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]"),
            ),
            f(
                "request",
                V::Object(vec![
                    f("requested_capability", s("cap.module.load_ephemeral")),
                    f("load_mode", s("ram_only")),
                    f("subject", s("agent.session.serial")),
                    f("resource", s("live_service_graph")),
                    f("audit_record_schema", s("raios.audit_record.v0")),
                    f(
                        "audit_record_canonicalization",
                        s("raios.audit_record.canonical.v0"),
                    ),
                    f("rollback_plan_schema", s("raios.rollback_plan.v0")),
                    f(
                        "rollback_plan_canonicalization",
                        s("raios.rollback_plan.canonical.v0"),
                    ),
                ]),
            ),
        ],
        6,
    );
    emit_module_audit_rollback_reference_object(&check, true);
    emit_module_audit_rollback_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_audit_rollback_gate_state(&check, true);
    emit_module_audit_rollback_policy_result(&check, true);
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

#[rustfmt::skip]
fn emit_module_audit_rollback_reference_object(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
    comma: bool,
) {
    emit_record_property_line(
        "audit_rollback_reference",
        vec![
            f(
                "state",
                s(if check.has_reference { "present" } else { "absent" }),
            ),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("denial_event_id", record_str_or_null(check.denial_event_id)),
            f(
                "retained_reference_event_id",
                record_str_or_null(check.retained_reference_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(check.ram_only_service_slot_id),
            ),
            f(
                "hashes",
                V::Object(record_sha_or_null_fields(&[
                    ("audit_record_hash", check.audit_record_hash),
                    ("expected_audit_record_hash", check.expected_audit_record_hash),
                    ("rollback_plan_hash", check.rollback_plan_hash),
                    ("expected_rollback_plan_hash", check.expected_rollback_plan_hash),
                    ("computed_capability_grant_hash", check.computed_grant_hash),
                    (
                        "expected_computed_capability_grant_hash",
                        check.expected_computed_grant_hash,
                    ),
                    ("manifest_hash", check.manifest_hash),
                    ("artifact_hash", check.artifact_hash),
                    ("vm_test_report_hash", check.vm_report_hash),
                    ("local_attestation_hash", check.local_attestation_hash),
                    ("local_approval_hash", check.local_approval_hash),
                    (
                        "pre_load_service_inventory_hash",
                        check.pre_load_service_inventory_hash,
                    ),
                    ("cleanup_actions_hash", check.cleanup_actions_hash),
                ])),
            ),
        ],
        comma,
    );
}

#[rustfmt::skip]
fn emit_module_audit_rollback_retained_reference(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleAuditRollbackReference)>,
    comma: bool,
) {
    let fields = if let Some((event_id, ref reference)) = retained {
        vec![
            f("state", s("present")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f(
                "matches_current_reference",
                b(module_audit_rollback_reference_matches(check, *reference)),
            ),
            f("schema", s("raios.module_audit_rollback_reference.v0")),
            f("status", s("retained_hash_reference_load_still_denied")),
            f("classification", s("local_only")),
            f("durable_audit_written", no()),
            f("rollback_plan_installed", no()),
            f("grants_capability", no()),
            f("grants_load_now", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("denial_event_id", record_event_or_null(Some(reference.denial_event_id))),
            f(
                "retained_computed_grant_reference_event_id",
                record_event_or_null(Some(reference.retained_reference_event_id)),
            ),
            f("ram_only_service_slot_id", s(reference.ram_only_service_slot_id.as_str())),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    ("audit_record_hash", reference.audit_record_hash),
                    ("rollback_plan_hash", reference.rollback_plan_hash),
                    ("computed_capability_grant_hash", reference.computed_grant_hash),
                    ("manifest_hash", reference.manifest_hash),
                    ("artifact_hash", reference.artifact_hash),
                    ("vm_test_report_hash", reference.vm_report_hash),
                    ("local_attestation_hash", reference.local_attestation_hash),
                    ("local_approval_hash", reference.local_approval_hash),
                    (
                        "pre_load_service_inventory_hash",
                        reference.pre_load_service_inventory_hash,
                    ),
                    ("cleanup_actions_hash", reference.cleanup_actions_hash),
                ])),
            ),
        ]
    } else {
        vec![
            f("state", s("missing")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(None)),
            f("recorded_event_id", record_event_or_null(None)),
            f("matches_current_reference", no()),
            f("schema", s("raios.module_audit_rollback_reference.v0")),
            f("status", s("missing")),
            f("reason", s("no_valid_audit_rollback_reference_retained")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ]
    };
    emit_record_property_line("retained_audit_rollback_reference", fields, comma);
}

#[rustfmt::skip]
fn emit_module_audit_rollback_gate_state(check: &ModuleAuditRollbackReferenceCheck<'_>, comma: bool) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    emit_record_property_line(
        "gate_state",
        vec![
            f(
                "retained_computed_grant_reference",
                s(if check.valid {
                    "hash_reference_supplied"
                } else {
                    state
                }),
            ),
            f("computed_capability_grant", s(state)),
            f("module_manifest", s("hash_reference_only")),
            f("candidate_artifact", s("hash_reference_only")),
            f("vm_test_report", s("hash_reference_only")),
            f("local_attestation", s("hash_reference_only")),
            f(
                "durable_audit_record",
                s(if check.valid {
                    "hash_reference_valid_not_durable"
                } else {
                    state
                }),
            ),
            f(
                "rollback_plan",
                s(if check.valid {
                    "hash_reference_valid_not_installed"
                } else {
                    state
                }),
            ),
            f("local_approval", s("hash_reference_only")),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("persistence", s("none")),
            f("can_load", no()),
        ],
        comma,
    );
}

#[rustfmt::skip]
fn emit_module_audit_rollback_policy_result(
    check: &ModuleAuditRollbackReferenceCheck<'_>,
    comma: bool,
) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("audit_record_hash_reference_present", b(check.valid)),
            f("rollback_plan_hash_reference_present", b(check.valid)),
            f("guest_evidence_authority", s("hash_reference_only_no_artifact_bytes")),
            f("durable_audit_written", no()),
            f("rollback_plan_installed", no()),
            f("grants_capability", no()),
            f("grants_load_now", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f(
                "required_before_load",
                record_static_str_array(&[
                    "durable_audit_record_write",
                    "rollback_plan_installation",
                    "module_loader",
                    "ram_only_service_slot_allocation",
                ]),
            ),
        ],
        comma,
    );
}

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_diagnostic_selftest() {
    let cases = module_audit_rollback_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases.iter().map(module_audit_rollback_selftest_case_record).collect();

    begin_response("module.audit_rollback_diagnostic_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_reference_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_audit_rollback_reference_records", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("allocates_service_slot", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                record_static_str_array(&[
                    "audit_record_hash",
                    "rollback_plan_hash",
                    "computed_capability_grant_hash",
                    "retained_reference_event_id",
                    "denial_event_id",
                    "manifest_hash",
                    "artifact_hash",
                    "vm_test_report_hash",
                    "local_attestation_hash",
                    "local_approval_hash",
                    "pre_load_service_inventory_hash",
                    "cleanup_actions_hash",
                    "ram_only_service_slot_id",
                ]),
            ),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.audit_rollback_diagnostic_selftest");
}

#[rustfmt::skip]
fn module_audit_rollback_selftest_case_record(case: &ModuleAuditRollbackSelfTestCase) -> V<'static> {
    V::InlineObject(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("can_load", no()), f("load_attempted", no())])
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
