use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, emit_export_gate,
        emit_record_fields_trailing_comma, emit_record_property_line,
        emit_record_value_property_line, end_response, method_eq, method_head_eq,
        parse_current_boot_event_id, parse_sha256_ref, raw_line, record_bool as b, record_event,
        record_false as no, record_field as f, record_missing_retained_reference_fields,
        record_present_absent, record_retained_reference_present_fields, record_selftest_case,
        record_sha_fields, record_sha_or_null, record_static_str_array, record_str as s,
        record_str_or_null,
    },
    event_log, module_evidence,
};
use raios_core::record::Value as V;
pub(crate) fn emit_module_manifest_diagnostic(method: &str) {
    let arg = module_manifest_diagnostic_arg(method);
    let check = parse_module_manifest_reference(arg);
    let recorded_event_id = if check.valid {
        module_manifest_binding_from_check(&check).map(event_log::record_module_manifest_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_manifest_reference();

    begin_response("module.manifest_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_manifest_reference_diagnostic.v0")),
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
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.manifest_diagnostic <manifest_reference_hash> <manifest_hash> [current_boot]"),
            ),
        ],
        6,
    );
    emit_record_property_line(
        "request",
        vec![
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("subject", s("agent.session.serial")),
            f("resource", s("live_service_graph")),
            f("manifest_schema", s("raios.module_manifest.v0")),
            f(
                "manifest_reference_schema",
                s("raios.module_manifest_reference.v0"),
            ),
            f(
                "manifest_reference_canonicalization",
                s("raios.module_manifest_reference.canonical.v0"),
            ),
        ],
        true,
    );
    emit_module_manifest_reference_object(&check, true);
    emit_module_manifest_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_manifest_gate_state(&check, true);
    emit_module_manifest_policy_result(&check, true);
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "module_manifest", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "candidate_artifact",
        "missing",
        "candidate_artifact_missing",
    );
    emit_export_gate(
        &mut wrote,
        "vm_test_report",
        "missing",
        "vm_test_report_missing",
    );
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
    emit_export_gate(
        &mut wrote,
        "computed_capability_grant",
        "missing",
        "computed_capability_grant_missing",
    );
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
    crlf();
    raw_line("      ]");
    end_response("module.manifest_diagnostic");
}

fn emit_module_manifest_reference_object(check: &ModuleManifestReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "module_manifest_reference",
        vec![
            f("state", record_present_absent(check.has_reference)),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("manifest_schema", s("raios.module_manifest.v0")),
            f(
                "manifest_reference_hash",
                record_sha_or_null(check.manifest_reference_hash),
            ),
            f(
                "expected_manifest_reference_hash",
                record_sha_or_null(check.expected_manifest_reference_hash),
            ),
            f("manifest_hash", record_sha_or_null(check.manifest_hash)),
        ],
        comma,
    );
}

fn emit_module_manifest_retained_reference(
    check: &ModuleManifestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleManifestReference)>,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        let mut fields = record_retained_reference_present_fields(
            event_id,
            recorded_event_id,
            module_manifest_reference_matches(check, reference),
            "raios.module_manifest_reference.v0",
        );
        fields.extend(vec![
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    ("manifest_reference_hash", reference.manifest_reference_hash),
                    ("manifest_hash", reference.manifest_hash),
                ])),
            ),
        ]);
        fields
    } else {
        record_missing_retained_reference_fields(
            "raios.module_manifest_reference.v0",
            "no_valid_module_manifest_reference_retained",
        )
    };
    emit_record_property_line("retained_manifest_reference", fields, comma);
}

fn emit_module_manifest_gate_state(check: &ModuleManifestReferenceCheck<'_>, comma: bool) {
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
            f("module_manifest", s(state)),
            f("candidate_artifact", s("missing")),
            f("vm_test_report", s("missing")),
            f("local_attestation", s("missing")),
            f("computed_capability_grant", s("missing")),
            f("local_approval", s("missing")),
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

fn emit_module_manifest_policy_result(check: &ModuleManifestReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("manifest_reference_present", b(check.valid)),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "guest_evidence_authority",
                s("hash_reference_only_no_manifest_json_or_artifact_bytes"),
            ),
            f(
                "required_before_load",
                record_static_str_array(&[
                    "candidate_artifact_sha256",
                    "raios.vm_test_report.v0",
                    "raios.local_attestation.v0",
                    "raios.computed_capability_grant.v0",
                    "raios.audit_record.v0",
                    "rollback_plan",
                    "module_loader",
                    "ram_only_service_slot",
                ]),
            ),
        ],
        comma,
    );
}

pub(crate) fn emit_module_manifest_diagnostic_selftest() {
    let cases = module_manifest_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.manifest_diagnostic_selftest");
    let case_records = cases
        .iter()
        .map(module_manifest_selftest_case_record)
        .collect();
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_manifest_reference_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_manifest_reference_records", no()),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.manifest_diagnostic_selftest");
}

fn module_manifest_selftest_case_record(case: &ModuleManifestSelfTestCase) -> V<'static> {
    record_selftest_case(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
    )
}

pub(crate) fn emit_module_artifact_diagnostic(method: &str) {
    let arg = module_artifact_diagnostic_arg(method);
    let check = parse_module_artifact_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_artifact_binding_from_check(&check)
            .map(event_log::record_module_candidate_artifact_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_candidate_artifact_reference();

    begin_response("module.artifact_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_candidate_artifact_reference_diagnostic.v0"),
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
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.artifact_diagnostic <artifact_reference_hash> <retained_manifest_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <manifest_hash> <computed_grant_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]"),
            ),
        ],
        6,
    );
    emit_record_property_line(
        "request",
        vec![
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("subject", s("agent.session.serial")),
            f("resource", s("live_service_graph")),
            f(
                "artifact_reference_schema",
                s("raios.module_candidate_artifact_reference.v0"),
            ),
            f(
                "artifact_reference_canonicalization",
                s("raios.module_candidate_artifact_reference.canonical.v0"),
            ),
        ],
        true,
    );
    emit_module_artifact_reference_object(&check, true);
    emit_module_artifact_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_artifact_gate_state(&check, true);
    emit_module_artifact_policy_result(&check, true);
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "candidate_artifact", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "vm_test_report",
        "missing",
        "vm_test_report_missing",
    );
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
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
    crlf();
    raw_line("      ]");
    end_response("module.artifact_diagnostic");
}

fn emit_module_artifact_reference_object(check: &ModuleArtifactReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "candidate_artifact_reference",
        vec![
            f("state", record_present_absent(check.has_reference)),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f(
                "retained_manifest_reference_event_id",
                record_str_or_null(check.retained_manifest_reference_event_id),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_str_or_null(check.retained_reference_event_id),
            ),
            f(
                "hashes",
                V::Object(vec![
                    f(
                        "artifact_reference_hash",
                        record_sha_or_null(check.artifact_reference_hash),
                    ),
                    f(
                        "expected_artifact_reference_hash",
                        record_sha_or_null(check.expected_artifact_reference_hash),
                    ),
                    f(
                        "manifest_reference_hash",
                        record_sha_or_null(check.manifest_reference_hash),
                    ),
                    f("manifest_hash", record_sha_or_null(check.manifest_hash)),
                    f(
                        "computed_capability_grant_hash",
                        record_sha_or_null(check.computed_grant_hash),
                    ),
                    f(
                        "expected_computed_capability_grant_hash",
                        record_sha_or_null(check.expected_computed_grant_hash),
                    ),
                    f("artifact_hash", record_sha_or_null(check.artifact_hash)),
                    f(
                        "vm_test_report_hash",
                        record_sha_or_null(check.vm_report_hash),
                    ),
                    f(
                        "local_attestation_hash",
                        record_sha_or_null(check.local_attestation_hash),
                    ),
                ]),
            ),
        ],
        comma,
    );
}

fn emit_module_artifact_retained_reference(
    check: &ModuleArtifactReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::ModuleCandidateArtifactReference,
    )>,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        let mut fields = record_retained_reference_present_fields(
            event_id,
            recorded_event_id,
            module_artifact_reference_matches(check, reference),
            "raios.module_candidate_artifact_reference.v0",
        );
        fields.extend(vec![
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "retained_manifest_reference_event_id",
                record_event(reference.retained_manifest_reference_event_id),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_event(reference.retained_reference_event_id),
            ),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    ("artifact_reference_hash", reference.artifact_reference_hash),
                    ("manifest_reference_hash", reference.manifest_reference_hash),
                    ("manifest_hash", reference.manifest_hash),
                    (
                        "computed_capability_grant_hash",
                        reference.computed_grant_hash,
                    ),
                    ("artifact_hash", reference.artifact_hash),
                    ("vm_test_report_hash", reference.vm_report_hash),
                    ("local_attestation_hash", reference.local_attestation_hash),
                ])),
            ),
        ]);
        fields
    } else {
        record_missing_retained_reference_fields(
            "raios.module_candidate_artifact_reference.v0",
            "no_valid_candidate_artifact_reference_retained",
        )
    };
    emit_record_property_line("retained_candidate_artifact_reference", fields, comma);
}

fn emit_module_artifact_gate_state(check: &ModuleArtifactReferenceCheck<'_>, comma: bool) {
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
            f("module_manifest", s("hash_reference_only")),
            f("candidate_artifact", s(state)),
            f("vm_test_report", s("missing")),
            f("local_attestation", s("missing")),
            f("computed_capability_grant", s("hash_reference_only")),
            f("local_approval", s("missing")),
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

fn emit_module_artifact_policy_result(check: &ModuleArtifactReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("candidate_artifact_reference_present", b(check.valid)),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "guest_evidence_authority",
                s("hash_reference_only_no_artifact_bytes"),
            ),
            f(
                "required_before_load",
                record_static_str_array(&[
                    "raios.vm_test_report.v0",
                    "raios.local_attestation.v0",
                    "raios.audit_record.v0",
                    "rollback_plan",
                    "module_loader",
                    "ram_only_service_slot",
                ]),
            ),
        ],
        comma,
    );
}

pub(crate) fn emit_module_artifact_diagnostic_selftest() {
    let cases = module_artifact_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.artifact_diagnostic_selftest");
    let case_records = cases
        .iter()
        .map(module_artifact_selftest_case_record)
        .collect();
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_candidate_artifact_reference_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f(
                "creates_retained_candidate_artifact_reference_records",
                no(),
            ),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.artifact_diagnostic_selftest");
}

fn module_artifact_selftest_case_record(case: &ModuleArtifactSelfTestCase) -> V<'static> {
    record_selftest_case(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
    )
}

fn parse_module_manifest_reference(arg: &str) -> ModuleManifestReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_manifest_reference(false, true, "current_boot", None, None);
    }

    let mut tokens = arg.split_whitespace();
    let manifest_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = manifest_reference_token.is_some() && manifest_token.is_some() && !extra;

    let manifest_reference_hash = manifest_reference_token.and_then(parse_sha256_ref);
    let manifest_hash = manifest_token.and_then(parse_sha256_ref);

    evaluate_module_manifest_reference(
        true,
        arity_valid,
        scope,
        manifest_reference_hash,
        manifest_hash,
    )
}

fn evaluate_module_manifest_reference<'a>(
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    manifest_reference_hash: Option<[u8; 32]>,
    manifest_hash: Option<[u8; 32]>,
) -> ModuleManifestReferenceCheck<'a> {
    if !has_reference {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "missing",
            reason: "module_manifest_reference_absent",
            valid: false,
        };
    }
    if !arity_valid {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "invalid_reference_arity",
            reason: "module_manifest_reference_requires_two_hashes_and_optional_scope",
            valid: false,
        };
    }
    let (Some(manifest_reference_hash), Some(manifest_hash)) =
        (manifest_reference_hash, manifest_hash)
    else {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "invalid_hash_reference",
            reason: "all_module_manifest_references_must_be_sha256",
            valid: false,
        };
    };
    let expected_manifest_reference_hash = computed_module_manifest_reference_hash(manifest_hash);
    if !method_eq(scope, "current_boot") {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash: Some(manifest_reference_hash),
            manifest_hash: Some(manifest_hash),
            expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
            status: "stale_or_non_current_boot_reference",
            reason: "module_manifest_reference_scope_must_be_current_boot",
            valid: false,
        };
    }
    if manifest_reference_hash != expected_manifest_reference_hash {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash: Some(manifest_reference_hash),
            manifest_hash: Some(manifest_hash),
            expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
            status: "mismatched_manifest_reference_hash",
            reason: "module_manifest_reference_hash_mismatch",
            valid: false,
        };
    }
    ModuleManifestReferenceCheck {
        has_reference,
        arity_valid,
        scope,
        manifest_reference_hash: Some(manifest_reference_hash),
        manifest_hash: Some(manifest_hash),
        expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
        status: "valid_hash_reference_load_still_denied",
        reason: "module_manifest_reference_valid_but_loader_and_evidence_missing",
        valid: true,
    }
}

fn module_manifest_selftest_cases() -> [ModuleManifestSelfTestCase; MODULE_MANIFEST_SELFTEST_CASES]
{
    let valid_hash = computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let absent = evaluate_module_manifest_reference(false, true, "current_boot", None, None);
    let valid = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    let stale = evaluate_module_manifest_reference(
        true,
        true,
        "previous_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    let mismatch = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_MISMATCH_MANIFEST_HASH),
    );
    let invalid_hash = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some([0x99; 32]),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    [
        module_manifest_selftest_case(
            "absent_reference",
            "missing",
            "module_manifest_reference_absent",
            absent,
        ),
        module_manifest_selftest_case(
            "accepted_current_boot_manifest_still_denied",
            "valid_hash_reference_load_still_denied",
            "module_manifest_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_manifest_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "module_manifest_reference_scope_must_be_current_boot",
            stale,
        ),
        module_manifest_selftest_case(
            "mismatched_manifest_hash_reference",
            "mismatched_manifest_reference_hash",
            "module_manifest_reference_hash_mismatch",
            mismatch,
        ),
        module_manifest_selftest_case(
            "invalid_manifest_reference_hash",
            "mismatched_manifest_reference_hash",
            "module_manifest_reference_hash_mismatch",
            invalid_hash,
        ),
    ]
}

fn module_manifest_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleManifestReferenceCheck<'_>,
) -> ModuleManifestSelfTestCase {
    ModuleManifestSelfTestCase {
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

fn module_manifest_binding_from_check(
    check: &ModuleManifestReferenceCheck<'_>,
) -> Option<event_log::ModuleManifestReference> {
    Some(event_log::ModuleManifestReference {
        manifest_reference_hash: check.manifest_reference_hash?,
        manifest_hash: check.manifest_hash?,
    })
}

fn module_manifest_reference_matches(
    check: &ModuleManifestReferenceCheck<'_>,
    reference: event_log::ModuleManifestReference,
) -> bool {
    check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
}

fn parse_module_artifact_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleArtifactReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_artifact_reference(
            ModuleArtifactReferenceInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                artifact_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                manifest_hash: None,
                computed_grant_hash: None,
                artifact_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
            },
            require_live_retained,
        );
    }

    let mut tokens = arg.split_whitespace();
    let artifact_reference_token = tokens.next();
    let retained_manifest_reference_event_id = tokens.next();
    let retained_reference_event_id = tokens.next();
    let manifest_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let grant_token = tokens.next();
    let artifact_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = artifact_reference_token.is_some()
        && retained_manifest_reference_event_id.is_some()
        && retained_reference_event_id.is_some()
        && manifest_reference_token.is_some()
        && manifest_token.is_some()
        && grant_token.is_some()
        && artifact_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && !extra;

    evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            has_reference: true,
            arity_valid,
            scope,
            artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
            retained_manifest_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
            manifest_hash: manifest_token.and_then(parse_sha256_ref),
            computed_grant_hash: grant_token.and_then(parse_sha256_ref),
            artifact_hash: artifact_token.and_then(parse_sha256_ref),
            vm_report_hash: report_token.and_then(parse_sha256_ref),
            local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        },
        require_live_retained,
    )
}

fn evaluate_module_artifact_reference<'a>(
    input: ModuleArtifactReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleArtifactReferenceCheck<'a> {
    if !input.has_reference {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "missing",
            "candidate_artifact_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "invalid_reference_arity",
            "candidate_artifact_reference_requires_hashes_events_and_optional_scope",
            false,
        );
    }

    let (
        Some(artifact_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(manifest_hash),
        Some(computed_grant_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        input.artifact_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.manifest_hash,
        input.computed_grant_hash,
        input.artifact_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
    )
    else {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "all_candidate_artifact_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_artifact_reference_hash = computed_module_candidate_artifact_reference_hash(
        retained_manifest_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash,
        manifest_hash,
        expected_computed_grant_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );

    if !method_eq(input.scope, "current_boot") {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "stale_or_non_current_boot_reference",
            "candidate_artifact_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if artifact_reference_hash != expected_artifact_reference_hash {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_candidate_artifact_reference_hash",
            "candidate_artifact_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_artifact_live_reference_mismatch(&input) {
            return module_artifact_reference_check(
                input,
                Some(expected_artifact_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_artifact_reference_check(
        input,
        Some(expected_artifact_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "candidate_artifact_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

fn module_artifact_reference_check<'a>(
    input: ModuleArtifactReferenceInput<'a>,
    expected_artifact_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleArtifactReferenceCheck<'a> {
    ModuleArtifactReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        artifact_reference_hash: input.artifact_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        manifest_hash: input.manifest_hash,
        computed_grant_hash: input.computed_grant_hash,
        artifact_hash: input.artifact_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        expected_artifact_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_artifact_live_reference_mismatch(
    input: &ModuleArtifactReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;
    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("candidate_artifact_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("candidate_artifact_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("candidate_artifact_manifest_reference_hash_mismatch");
    }
    if Some(manifest_reference.manifest_hash) != input.manifest_hash {
        return Some("candidate_artifact_manifest_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("candidate_artifact_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("candidate_artifact_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("candidate_artifact_computed_grant_hash_mismatch");
    }
    if Some(retained_reference.manifest_hash) != input.manifest_hash {
        return Some("candidate_artifact_manifest_hash_mismatch");
    }
    if Some(retained_reference.artifact_hash) != input.artifact_hash {
        return Some("candidate_artifact_hash_mismatch");
    }
    if Some(retained_reference.vm_report_hash) != input.vm_report_hash {
        return Some("candidate_artifact_vm_report_hash_mismatch");
    }
    if Some(retained_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("candidate_artifact_local_attestation_hash_mismatch");
    }
    None
}

fn module_artifact_selftest_cases() -> [ModuleArtifactSelfTestCase; MODULE_ARTIFACT_SELFTEST_CASES]
{
    let manifest_reference_hash =
        computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let valid_hash = computed_module_candidate_artifact_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let base = ModuleArtifactReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        artifact_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    };
    let absent = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            has_reference: false,
            arity_valid: true,
            scope: "current_boot",
            artifact_reference_hash: None,
            retained_manifest_reference_event_id: None,
            retained_reference_event_id: None,
            manifest_reference_hash: None,
            manifest_hash: None,
            computed_grant_hash: None,
            artifact_hash: None,
            vm_report_hash: None,
            local_attestation_hash: None,
        },
        false,
    );
    let valid = evaluate_module_artifact_reference(base, false);
    let stale = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            scope: "previous_boot",
            ..base
        },
        false,
    );
    let mismatch = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            artifact_reference_hash: Some([0x99; 32]),
            ..base
        },
        false,
    );
    let invalid_hash = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            artifact_reference_hash: None,
            ..base
        },
        false,
    );
    let grant_mismatch = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            computed_grant_hash: Some([0xaa; 32]),
            ..base
        },
        false,
    );
    let bad_event_id = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
            ..base
        },
        false,
    );
    [
        module_artifact_selftest_case(
            "absent_reference",
            "missing",
            "candidate_artifact_reference_absent",
            absent,
        ),
        module_artifact_selftest_case(
            "accepted_current_boot_artifact_still_denied",
            "valid_hash_reference_load_still_denied",
            "candidate_artifact_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_artifact_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "candidate_artifact_reference_scope_must_be_current_boot",
            stale,
        ),
        module_artifact_selftest_case(
            "mismatched_artifact_reference_hash",
            "mismatched_candidate_artifact_reference_hash",
            "candidate_artifact_reference_hash_mismatch",
            mismatch,
        ),
        module_artifact_selftest_case(
            "invalid_artifact_reference_hash",
            "invalid_hash_reference",
            "all_candidate_artifact_references_must_be_sha256_or_current_boot_ids",
            invalid_hash,
        ),
        module_artifact_selftest_case(
            "computed_grant_hash_mismatch",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            grant_mismatch,
        ),
        module_artifact_selftest_case(
            "retained_manifest_reference_event_id_not_current_boot",
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            bad_event_id,
        ),
    ]
}

fn module_artifact_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleArtifactReferenceCheck<'_>,
) -> ModuleArtifactSelfTestCase {
    ModuleArtifactSelfTestCase {
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

fn module_artifact_binding_from_check(
    check: &ModuleArtifactReferenceCheck<'_>,
) -> Option<event_log::ModuleCandidateArtifactReference> {
    Some(event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash: check.artifact_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        manifest_hash: check.manifest_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        artifact_hash: check.artifact_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
    })
}

fn module_artifact_reference_matches(
    check: &ModuleArtifactReferenceCheck<'_>,
    reference: event_log::ModuleCandidateArtifactReference,
) -> bool {
    check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check
            .retained_manifest_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_manifest_reference_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

pub(crate) fn emit_module_vm_report_diagnostic(method: &str) {
    let arg = module_vm_report_diagnostic_arg(method);
    let check = parse_module_vm_report_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_vm_report_binding_from_check(&check)
            .map(event_log::record_module_vm_test_report_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_vm_test_report_reference();

    begin_response("module.vm_report_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_vm_test_report_reference_diagnostic.v0"),
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
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_unsigned_service_code", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]"),
            ),
        ],
        6,
    );
    emit_record_property_line(
        "request",
        vec![
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("subject", s("agent.session.serial")),
            f("resource", s("live_service_graph")),
            f("vm_test_report_schema", s("raios.vm_test_report.v0")),
            f(
                "vm_test_report_reference_schema",
                s("raios.module_vm_test_report_reference.v0"),
            ),
            f(
                "vm_test_report_reference_canonicalization",
                s("raios.module_vm_test_report_reference.canonical.v0"),
            ),
        ],
        true,
    );
    emit_module_vm_report_reference_object(&check, true);
    emit_module_vm_report_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_vm_report_gate_state(&check, true);
    emit_module_vm_report_policy_result(&check, true);
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "vm_test_report", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
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
    crlf();
    raw_line("      ]");
    end_response("module.vm_report_diagnostic");
}

fn emit_module_vm_report_reference_object(check: &ModuleVmReportReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "vm_test_report_reference",
        vec![
            f("state", record_present_absent(check.has_reference)),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f(
                "retained_manifest_reference_event_id",
                record_str_or_null(check.retained_manifest_reference_event_id),
            ),
            f(
                "retained_candidate_artifact_reference_event_id",
                record_str_or_null(check.retained_artifact_reference_event_id),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_str_or_null(check.retained_reference_event_id),
            ),
            f("vm_test_report_schema", s("raios.vm_test_report.v0")),
            f(
                "vm_test_report_reference_hash",
                record_sha_or_null(check.report_reference_hash),
            ),
            f(
                "expected_vm_test_report_reference_hash",
                record_sha_or_null(check.expected_report_reference_hash),
            ),
            f(
                "expected_computed_capability_grant_hash",
                record_sha_or_null(check.expected_computed_grant_hash),
            ),
            f(
                "manifest_reference_hash",
                record_sha_or_null(check.manifest_reference_hash),
            ),
            f(
                "artifact_reference_hash",
                record_sha_or_null(check.artifact_reference_hash),
            ),
            f("manifest_hash", record_sha_or_null(check.manifest_hash)),
            f("artifact_hash", record_sha_or_null(check.artifact_hash)),
            f(
                "computed_capability_grant_hash",
                record_sha_or_null(check.computed_grant_hash),
            ),
            f(
                "vm_test_report_hash",
                record_sha_or_null(check.vm_report_hash),
            ),
            f(
                "local_attestation_hash",
                record_sha_or_null(check.local_attestation_hash),
            ),
        ],
        comma,
    );
}

fn emit_module_vm_report_retained_reference(
    check: &ModuleVmReportReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleVmTestReportReference)>,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        let mut fields = record_retained_reference_present_fields(
            event_id,
            recorded_event_id,
            module_vm_report_reference_matches(check, reference),
            "raios.module_vm_test_report_reference.v0",
        );
        fields.extend(vec![
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_unsigned_service_code", no()),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "retained_manifest_reference_event_id",
                record_event(reference.retained_manifest_reference_event_id),
            ),
            f(
                "retained_candidate_artifact_reference_event_id",
                record_event(reference.retained_artifact_reference_event_id),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_event(reference.retained_reference_event_id),
            ),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    (
                        "vm_test_report_reference_hash",
                        reference.report_reference_hash,
                    ),
                    ("manifest_reference_hash", reference.manifest_reference_hash),
                    ("artifact_reference_hash", reference.artifact_reference_hash),
                    ("manifest_hash", reference.manifest_hash),
                    ("artifact_hash", reference.artifact_hash),
                    (
                        "computed_capability_grant_hash",
                        reference.computed_grant_hash,
                    ),
                    ("vm_test_report_hash", reference.vm_report_hash),
                    ("local_attestation_hash", reference.local_attestation_hash),
                ])),
            ),
        ]);
        fields
    } else {
        record_missing_retained_reference_fields(
            "raios.module_vm_test_report_reference.v0",
            "no_valid_vm_test_report_reference_retained",
        )
    };
    emit_record_property_line("retained_vm_test_report_reference", fields, comma);
}

fn emit_module_vm_report_gate_state(check: &ModuleVmReportReferenceCheck<'_>, comma: bool) {
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
            f("module_manifest", s("retained_hash_reference_only")),
            f("candidate_artifact", s("retained_hash_reference_only")),
            f("vm_test_report", s(state)),
            f("local_attestation", s("missing")),
            f(
                "computed_capability_grant",
                s("retained_hash_reference_only"),
            ),
            f("local_approval", s("missing")),
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

fn emit_module_vm_report_policy_result(check: &ModuleVmReportReferenceCheck<'_>, comma: bool) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("vm_test_report_reference_present", b(check.valid)),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "guest_evidence_authority",
                s("hash_reference_only_no_vm_report_json_or_artifact_bytes"),
            ),
            f(
                "required_before_load",
                record_static_str_array(&[
                    "raios.local_attestation.v0",
                    "local_approval",
                    "raios.audit_record.v0",
                    "rollback_plan",
                    "module_loader",
                    "ram_only_service_slot",
                ]),
            ),
        ],
        comma,
    );
}

pub(crate) fn emit_module_vm_report_diagnostic_selftest() {
    let cases = module_vm_report_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.vm_report_diagnostic_selftest");
    let case_records = cases
        .iter()
        .map(module_vm_report_selftest_case_record)
        .collect();
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_vm_test_report_reference_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_vm_test_report_reference_records", no()),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.vm_report_diagnostic_selftest");
}

fn module_vm_report_selftest_case_record(case: &ModuleVmReportSelfTestCase) -> V<'static> {
    record_selftest_case(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
    )
}

fn parse_module_vm_report_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleVmReportReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_vm_report_reference(
            ModuleVmReportReferenceInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                report_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_artifact_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                artifact_reference_hash: None,
                manifest_hash: None,
                artifact_hash: None,
                computed_grant_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
            },
            require_live_retained,
        );
    }

    let mut tokens = arg.split_whitespace();
    let report_reference_token = tokens.next();
    let retained_manifest_reference_event_id = tokens.next();
    let retained_artifact_reference_event_id = tokens.next();
    let retained_reference_event_id = tokens.next();
    let manifest_reference_token = tokens.next();
    let artifact_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let grant_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = report_reference_token.is_some()
        && retained_manifest_reference_event_id.is_some()
        && retained_artifact_reference_event_id.is_some()
        && retained_reference_event_id.is_some()
        && manifest_reference_token.is_some()
        && artifact_reference_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && grant_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && !extra;

    evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            has_reference: true,
            arity_valid,
            scope,
            report_reference_hash: report_reference_token.and_then(parse_sha256_ref),
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
            artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
            manifest_hash: manifest_token.and_then(parse_sha256_ref),
            artifact_hash: artifact_token.and_then(parse_sha256_ref),
            computed_grant_hash: grant_token.and_then(parse_sha256_ref),
            vm_report_hash: report_token.and_then(parse_sha256_ref),
            local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        },
        require_live_retained,
    )
}

fn evaluate_module_vm_report_reference<'a>(
    input: ModuleVmReportReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleVmReportReferenceCheck<'a> {
    if !input.has_reference {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "missing",
            "vm_test_report_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "invalid_reference_arity",
            "vm_test_report_reference_requires_hashes_events_and_optional_scope",
            false,
        );
    }

    let (
        Some(report_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_artifact_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(artifact_reference_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(computed_grant_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        input.report_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_artifact_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.artifact_reference_hash,
        input.manifest_hash,
        input.artifact_hash,
        input.computed_grant_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
    )
    else {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "all_vm_test_report_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_report_reference_hash = computed_module_vm_test_report_reference_hash(
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash,
        artifact_reference_hash,
        manifest_hash,
        artifact_hash,
        expected_computed_grant_hash,
        vm_report_hash,
        local_attestation_hash,
    );

    if !method_eq(input.scope, "current_boot") {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "stale_or_non_current_boot_reference",
            "vm_test_report_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_artifact_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if report_reference_hash != expected_report_reference_hash {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_vm_test_report_reference_hash",
            "vm_test_report_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_vm_report_live_reference_mismatch(&input) {
            return module_vm_report_reference_check(
                input,
                Some(expected_report_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_vm_report_reference_check(
        input,
        Some(expected_report_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "vm_test_report_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

fn module_vm_report_reference_check<'a>(
    input: ModuleVmReportReferenceInput<'a>,
    expected_report_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleVmReportReferenceCheck<'a> {
    ModuleVmReportReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        report_reference_hash: input.report_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_artifact_reference_event_id: input.retained_artifact_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        artifact_reference_hash: input.artifact_reference_hash,
        manifest_hash: input.manifest_hash,
        artifact_hash: input.artifact_hash,
        computed_grant_hash: input.computed_grant_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        expected_report_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_vm_report_live_reference_mismatch(
    input: &ModuleVmReportReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_artifact_reference_event_id =
        parse_current_boot_event_id(input.retained_artifact_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;
    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("vm_test_report_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("vm_test_report_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("vm_test_report_manifest_reference_hash_mismatch");
    }
    if Some(manifest_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }

    let Some((latest_artifact_event_id, artifact_reference)) =
        event_log::latest_module_candidate_artifact_reference()
    else {
        return Some("vm_test_report_artifact_reference_missing");
    };
    if latest_artifact_event_id != retained_artifact_reference_event_id {
        return Some("vm_test_report_artifact_reference_mismatch");
    }
    if Some(artifact_reference.artifact_reference_hash) != input.artifact_reference_hash {
        return Some("vm_test_report_artifact_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("vm_test_report_manifest_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }
    if Some(artifact_reference.artifact_hash) != input.artifact_hash {
        return Some("vm_test_report_artifact_hash_mismatch");
    }
    if Some(artifact_reference.vm_report_hash) != input.vm_report_hash {
        return Some("vm_test_report_hash_mismatch");
    }
    if Some(artifact_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("vm_test_report_local_attestation_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("vm_test_report_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("vm_test_report_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("vm_test_report_computed_grant_hash_mismatch");
    }
    if Some(retained_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }
    if Some(retained_reference.artifact_hash) != input.artifact_hash {
        return Some("vm_test_report_artifact_hash_mismatch");
    }
    if Some(retained_reference.vm_report_hash) != input.vm_report_hash {
        return Some("vm_test_report_hash_mismatch");
    }
    if Some(retained_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("vm_test_report_local_attestation_hash_mismatch");
    }
    None
}

fn module_vm_report_selftest_cases() -> [ModuleVmReportSelfTestCase; MODULE_VM_REPORT_SELFTEST_CASES]
{
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
    let valid_hash = computed_module_vm_test_report_reference_hash(
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
    let valid_input = ModuleVmReportReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        report_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_artifact_reference_event_id: Some(
            MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        artifact_reference_hash: Some(artifact_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    };
    let absent = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            has_reference: false,
            ..valid_input
        },
        false,
    );
    let valid = evaluate_module_vm_report_reference(valid_input, false);
    let stale = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            scope: "previous_boot",
            ..valid_input
        },
        false,
    );
    let mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            report_reference_hash: Some([0x99; 32]),
            ..valid_input
        },
        false,
    );
    let grant_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            computed_grant_hash: Some([0xaa; 32]),
            ..valid_input
        },
        false,
    );
    let manifest_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
            ..valid_input
        },
        false,
    );
    let artifact_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_artifact_reference_event_id: Some("event.previous_boot.00000028"),
            ..valid_input
        },
        false,
    );
    let grant_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_reference_event_id: Some("event.previous_boot.00000027"),
            ..valid_input
        },
        false,
    );
    [
        module_vm_report_selftest_case(
            "absent_reference",
            "missing",
            "vm_test_report_reference_absent",
            absent,
        ),
        module_vm_report_selftest_case(
            "accepted_current_boot_report_still_denied",
            "valid_hash_reference_load_still_denied",
            "vm_test_report_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_vm_report_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "vm_test_report_reference_scope_must_be_current_boot",
            stale,
        ),
        module_vm_report_selftest_case(
            "vm_report_reference_hash_mismatch",
            "mismatched_vm_test_report_reference_hash",
            "vm_test_report_reference_hash_mismatch",
            mismatch,
        ),
        module_vm_report_selftest_case(
            "computed_grant_hash_mismatch",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            grant_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_manifest_reference_event_not_current_boot",
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            manifest_event_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_artifact_reference_event_not_current_boot",
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            artifact_event_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_reference_event_not_current_boot",
            "rejected",
            "retained_reference_event_id_not_current_boot",
            grant_event_mismatch,
        ),
    ]
}

fn module_vm_report_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleVmReportReferenceCheck<'_>,
) -> ModuleVmReportSelfTestCase {
    ModuleVmReportSelfTestCase {
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

fn module_vm_report_binding_from_check(
    check: &ModuleVmReportReferenceCheck<'_>,
) -> Option<event_log::ModuleVmTestReportReference> {
    Some(event_log::ModuleVmTestReportReference {
        report_reference_hash: check.report_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_artifact_reference_event_id: parse_current_boot_event_id(
            check.retained_artifact_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        artifact_reference_hash: check.artifact_reference_hash?,
        manifest_hash: check.manifest_hash?,
        artifact_hash: check.artifact_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
    })
}

fn module_vm_report_reference_matches(
    check: &ModuleVmReportReferenceCheck<'_>,
    reference: event_log::ModuleVmTestReportReference,
) -> bool {
    check.report_reference_hash == Some(reference.report_reference_hash)
        && check
            .retained_manifest_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_manifest_reference_event_id)
        && check
            .retained_artifact_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_artifact_reference_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

pub(crate) fn module_manifest_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.manifest_diagnostic")
}

pub(crate) fn module_manifest_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.manifest_diagnostic_selftest")
}

pub(crate) fn module_artifact_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.artifact_diagnostic")
}

pub(crate) fn module_artifact_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.artifact_diagnostic_selftest")
}

pub(crate) fn module_vm_report_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.vm_report_diagnostic")
}

pub(crate) fn module_vm_report_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.vm_report_diagnostic_selftest")
}

fn module_manifest_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.manifest_diagnostic") {
        "module.manifest_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn module_artifact_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.artifact_diagnostic") {
        "module.artifact_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn module_vm_report_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.vm_report_diagnostic") {
        "module.vm_report_diagnostic".len()
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
