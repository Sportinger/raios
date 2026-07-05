use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_record_fields_trailing_comma,
        emit_record_property_line, emit_record_value_property_line, end_response, method_eq,
        method_head_eq, parse_sha256_ref, raw_line, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_sha_or_null, record_static_str_array,
        record_str as s,
    },
    event_log, module_evidence,
};
use raios_core::record::Value as V;

pub(crate) fn module_grant_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.grant_diagnostic")
        || method_head_eq(method, "module.load_gate_diagnostic")
}

pub(crate) fn module_grant_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.grant_diagnostic_selftest")
        || method_head_eq(method, "module.load_gate_diagnostic_selftest")
}

#[rustfmt::skip]
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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_computed_grant_diagnostic.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("accepts_artifact_bytes", no()),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]"),
            ),
            f(
                "request",
                V::Object(vec![
                    f("requested_capability", s("cap.module.load_ephemeral")),
                    f("load_mode", s("ram_only")),
                    f("risk", s("modify_ram")),
                    f("subject", s("agent.session.serial")),
                    f("resource", s("live_service_graph")),
                ]),
            ),
        ],
        6,
    );
    emit_record_property_line("computed_grant_reference", module_grant_reference_fields(&check), true);
    emit_module_grant_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_grant_gate_state(&check, true);
    emit_module_grant_policy_result(&check, true);
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
    end_response("module.grant_diagnostic");
}

#[rustfmt::skip]
pub(crate) fn emit_module_grant_diagnostic_selftest() {
    let cases = module_grant_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases.iter().map(module_grant_selftest_case_record).collect();

    begin_response("module.grant_diagnostic_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_computed_grant_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.grant_diagnostic_selftest");
}

#[rustfmt::skip]
fn module_grant_reference_fields<'a>(
    check: &ModuleGrantReferenceCheck<'a>,
) -> Vec<raios_core::record::Field<'a>> {
    vec![
        f(
            "state",
            s(if check.has_reference { "present" } else { "absent" }),
        ),
        f("validation_status", s(check.status)),
        f("validation_reason", s(check.reason)),
        f("arity_valid", b(check.arity_valid)),
        f("scope", s(check.scope)),
        f("computed_capability_grant_hash", record_sha_or_null(check.grant_hash)),
        f(
            "expected_computed_capability_grant_hash",
            record_sha_or_null(check.expected_grant_hash),
        ),
        f("manifest_hash", record_sha_or_null(check.manifest_hash)),
        f("artifact_hash", record_sha_or_null(check.artifact_hash)),
        f("vm_test_report_hash", record_sha_or_null(check.vm_report_hash)),
        f(
            "local_attestation_hash",
            record_sha_or_null(check.local_attestation_hash),
        ),
    ]
}

#[rustfmt::skip]
fn module_grant_selftest_case_record(case: &ModuleGrantSelfTestCase) -> V<'static> {
    V::InlineObject(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("can_load", no()), f("load_attempted", no())])
}

fn emit_module_grant_gate_state(check: &ModuleGrantReferenceCheck<'_>, comma: bool) {
    let computed_grant = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    emit_record_property_line(
        "gate_state",
        vec![
            f("module_manifest", s("hash_reference_only")),
            f("candidate_artifact", s("hash_reference_only")),
            f("vm_test_report", s("hash_reference_only")),
            f("local_attestation", s("hash_reference_only")),
            f("computed_capability_grant", s(computed_grant)),
            f("local_approval", s("not_received_by_guest")),
            f("rollback_plan", s("missing")),
            f("durable_audit_record", s("missing")),
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
fn emit_module_grant_retained_reference(
    check: &ModuleGrantReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleComputedGrantReference)>,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        vec![
            f("state", s("present")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f(
                "matches_current_reference",
                b(module_grant_reference_matches(check, reference)),
            ),
            f("schema", s("raios.module_computed_grant_reference.v0")),
            f("status", s("retained_hash_reference_load_still_denied")),
            f("grants_capability", no()),
            f("grants_load_now", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f(
                "hashes",
                V::Object(vec![
                    f(
                        "computed_capability_grant_hash",
                        record_sha_or_null(Some(reference.computed_grant_hash)),
                    ),
                    f("manifest_hash", record_sha_or_null(Some(reference.manifest_hash))),
                    f("artifact_hash", record_sha_or_null(Some(reference.artifact_hash))),
                    f(
                        "vm_test_report_hash",
                        record_sha_or_null(Some(reference.vm_report_hash)),
                    ),
                    f(
                        "local_attestation_hash",
                        record_sha_or_null(Some(reference.local_attestation_hash)),
                    ),
                ]),
            ),
        ]
    } else {
        vec![
            f("state", s("missing")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(None)),
            f("recorded_event_id", record_event_or_null(None)),
            f("matches_current_reference", no()),
            f("schema", s("raios.module_computed_grant_reference.v0")),
            f("status", s("missing")),
            f("reason", s("no_valid_computed_grant_reference_retained")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ]
    };
    emit_record_property_line("retained_reference", fields, comma);
}

#[rustfmt::skip]
fn emit_module_grant_policy_result(check: &ModuleGrantReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("computed_candidate_present", b(check.valid)),
            f("grants_capability", no()),
            f("grants_load_now", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("guest_evidence_authority", s("hash_reference_only_no_artifact_bytes")),
            f(
                "required_before_load",
                record_static_str_array(&["in_guest_evidence_retention", "raios.audit_record.v0", "rollback_plan", "module_loader", "ram_only_service_slot"]),
            ),
        ],
        comma,
    );
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
