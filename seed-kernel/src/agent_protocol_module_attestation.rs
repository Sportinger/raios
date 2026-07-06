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
        record_sha_fields, record_sha_or_null_fields, record_static_str_array, record_str as s,
        record_str_or_null, run_selftest_cases_with, CaseSpec,
    },
    event_log, module_evidence,
};
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum LocalAttestationSelftestMutation {
    Absent,
    Accepted,
    SignatureInvalid,
    Stale,
    ReferenceHashMismatch,
    ComputedGrantHashMismatch,
    ManifestEventPreviousBoot,
    ArtifactEventPreviousBoot,
    VmReportEventPreviousBoot,
    RetainedEventPreviousBoot,
}

const fn local_attestation_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: LocalAttestationSelftestMutation,
) -> CaseSpec<LocalAttestationSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const LOCAL_ATTESTATION_CASES: [CaseSpec<LocalAttestationSelftestMutation>;
    MODULE_LOCAL_ATTESTATION_SELFTEST_CASES] = [
    local_attestation_case(
        "absent_reference",
        "missing",
        "local_attestation_reference_absent",
        LocalAttestationSelftestMutation::Absent,
    ),
    local_attestation_case(
        "accepted_current_boot_attestation_still_denied",
        "valid_hash_reference_load_still_denied",
        "local_attestation_reference_valid_but_loader_and_evidence_missing",
        LocalAttestationSelftestMutation::Accepted,
    ),
    local_attestation_case(
        "promotion_signature_invalid",
        "mismatched_local_attestation_signature",
        "local_attestation_promotion_signature_invalid",
        LocalAttestationSelftestMutation::SignatureInvalid,
    ),
    local_attestation_case(
        "stale_previous_boot_reference",
        "stale_or_non_current_boot_reference",
        "local_attestation_reference_scope_must_be_current_boot",
        LocalAttestationSelftestMutation::Stale,
    ),
    local_attestation_case(
        "local_attestation_reference_hash_mismatch",
        "mismatched_local_attestation_reference_hash",
        "local_attestation_reference_hash_mismatch",
        LocalAttestationSelftestMutation::ReferenceHashMismatch,
    ),
    local_attestation_case(
        "computed_grant_hash_mismatch",
        "mismatched_computed_grant_hash",
        "computed_grant_hash_mismatch",
        LocalAttestationSelftestMutation::ComputedGrantHashMismatch,
    ),
    local_attestation_case(
        "retained_manifest_reference_event_not_current_boot",
        "rejected",
        "retained_manifest_reference_event_id_not_current_boot",
        LocalAttestationSelftestMutation::ManifestEventPreviousBoot,
    ),
    local_attestation_case(
        "retained_artifact_reference_event_not_current_boot",
        "rejected",
        "retained_artifact_reference_event_id_not_current_boot",
        LocalAttestationSelftestMutation::ArtifactEventPreviousBoot,
    ),
    local_attestation_case(
        "retained_vm_report_reference_event_not_current_boot",
        "rejected",
        "retained_vm_report_reference_event_id_not_current_boot",
        LocalAttestationSelftestMutation::VmReportEventPreviousBoot,
    ),
    local_attestation_case(
        "retained_reference_event_not_current_boot",
        "rejected",
        "retained_reference_event_id_not_current_boot",
        LocalAttestationSelftestMutation::RetainedEventPreviousBoot,
    ),
];

pub(crate) fn emit_module_attestation_diagnostic(method: &str) {
    let arg = module_attestation_diagnostic_arg(method);
    let check = parse_module_local_attestation_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_local_attestation_binding_from_check(&check)
            .map(event_log::record_module_local_attestation_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_local_attestation_reference();

    begin_response("module.attestation_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_local_attestation_reference_diagnostic.v0"),
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
            f("accepts_local_attestation_json", no()),
            f("accepts_unsigned_service_code", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("artifact_loaded", no()),
            f("service_started", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "reference_format",
                s("module.attestation_diagnostic <local_attestation_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_vm_report_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <vm_test_report_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]"),
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
            f("local_attestation_schema", s("raios.local_attestation.v0")),
            f(
                "local_attestation_reference_schema",
                s("raios.module_local_attestation_reference.v0"),
            ),
            f(
                "local_attestation_reference_canonicalization",
                s("raios.module_local_attestation_reference.canonical.v0"),
            ),
        ],
        true,
    );
    emit_module_local_attestation_reference_object(&check, true);
    emit_module_local_attestation_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_local_attestation_gate_state(&check, true);
    emit_module_local_attestation_policy_result(&check, true);
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "local_attestation", check.status, check.reason);
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
    end_response("module.attestation_diagnostic");
}

pub(crate) fn emit_module_attestation_diagnostic_selftest() {
    let cases = module_local_attestation_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.attestation_diagnostic_selftest");
    let case_records = cases
        .iter()
        .map(module_local_attestation_selftest_case_record)
        .collect();
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_local_attestation_reference_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_local_attestation_reference_records", no()),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_local_attestation_json", no()),
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
    end_response("module.attestation_diagnostic_selftest");
}

fn emit_module_local_attestation_reference_object(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
    comma: bool,
) {
    let mut fields = vec![
        f("state", record_present_absent(check.has_reference)),
        f("validation_status", s(check.status)),
        f("validation_reason", s(check.reason)),
    ];
    if check.signature_verified {
        fields.push(f("signature_verified", b(true)));
    }
    fields.extend(vec![
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
            "retained_vm_test_report_reference_event_id",
            record_str_or_null(check.retained_vm_report_reference_event_id),
        ),
        f(
            "retained_computed_grant_reference_event_id",
            record_str_or_null(check.retained_reference_event_id),
        ),
        f(
            "hashes",
            V::Object(record_sha_or_null_fields(&[
                (
                    "local_attestation_reference_hash",
                    check.attestation_reference_hash,
                ),
                (
                    "expected_local_attestation_reference_hash",
                    check.expected_attestation_reference_hash,
                ),
                ("manifest_reference_hash", check.manifest_reference_hash),
                ("artifact_reference_hash", check.artifact_reference_hash),
                (
                    "vm_test_report_reference_hash",
                    check.vm_report_reference_hash,
                ),
                ("manifest_hash", check.manifest_hash),
                ("artifact_hash", check.artifact_hash),
                ("computed_capability_grant_hash", check.computed_grant_hash),
                (
                    "expected_computed_capability_grant_hash",
                    check.expected_computed_grant_hash,
                ),
                ("vm_test_report_hash", check.vm_report_hash),
                ("local_attestation_hash", check.local_attestation_hash),
            ])),
        ),
    ]);
    emit_record_property_line("local_attestation_reference", fields, comma);
}

fn emit_module_local_attestation_retained_reference(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::ModuleLocalAttestationReference,
    )>,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        let mut fields = record_retained_reference_present_fields(
            event_id,
            recorded_event_id,
            module_local_attestation_reference_matches(check, reference),
            "raios.module_local_attestation_reference.v0",
        );
        fields.extend(vec![
            f("accepts_local_attestation_json", no()),
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
                "retained_candidate_artifact_reference_event_id",
                record_event(reference.retained_artifact_reference_event_id),
            ),
            f(
                "retained_vm_test_report_reference_event_id",
                record_event(reference.retained_vm_report_reference_event_id),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_event(reference.retained_reference_event_id),
            ),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    (
                        "local_attestation_reference_hash",
                        reference.attestation_reference_hash,
                    ),
                    ("manifest_reference_hash", reference.manifest_reference_hash),
                    ("artifact_reference_hash", reference.artifact_reference_hash),
                    (
                        "vm_test_report_reference_hash",
                        reference.vm_report_reference_hash,
                    ),
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
            "raios.module_local_attestation_reference.v0",
            "no_valid_local_attestation_reference_retained",
        )
    };
    emit_record_property_line("retained_local_attestation_reference", fields, comma);
}

fn emit_module_local_attestation_gate_state(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
    comma: bool,
) {
    let state = if check.signature_verified {
        "signature_verified"
    } else if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    let mut fields = vec![
        f("module_manifest", s("hash_reference_only")),
        f("candidate_artifact", s("hash_reference_only")),
        f("vm_test_report", s("hash_reference_only")),
        f("local_attestation", s(state)),
    ];
    if check.signature_verified {
        fields.push(f("signature_verified", b(true)));
    }
    fields.extend(vec![
        f("computed_capability_grant", s("hash_reference_only")),
        f("local_approval", s("not_received_by_guest")),
        f("rollback_plan", s("missing")),
        f("durable_audit_record", s("missing")),
        f("loader", s("unavailable")),
        f("service_slot", s("unallocated")),
        f("artifact_loaded", no()),
        f("service_started", no()),
        f("persistence", s("none")),
        f("can_load", no()),
    ]);
    emit_record_property_line("gate_state", fields, comma);
}

fn emit_module_local_attestation_policy_result(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
    comma: bool,
) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("local_attestation_reference_present", b(check.valid)),
            f("local_attestation_present", b(check.valid)),
            f("authorizes_guest_load", no()),
            f("can_load_now", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "guest_evidence_authority",
                s("hash_reference_only_no_attestation_json_or_artifact_bytes"),
            ),
            f(
                "required_before_load",
                record_static_str_array(&[
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

fn module_local_attestation_selftest_case_record(
    case: &ModuleLocalAttestationSelfTestCase,
) -> V<'static> {
    record_selftest_case(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        case.passed,
    )
}

fn parse_module_local_attestation_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleLocalAttestationReferenceCheck<'_> {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return module_local_attestation_reference_check(
            ModuleLocalAttestationReferenceInput {
                has_reference: false,
                arity_valid: false,
                scope: "current_boot",
                attestation_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_artifact_reference_event_id: None,
                retained_vm_report_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                artifact_reference_hash: None,
                vm_report_reference_hash: None,
                manifest_hash: None,
                artifact_hash: None,
                computed_grant_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
                promotion_signature_der: None,
                signature_verified: false,
            },
            None,
            None,
            "missing",
            "local_attestation_reference_absent",
            false,
        );
    }

    let mut tokens = trimmed.split_whitespace();
    let reference_token = tokens.next();
    let manifest_event_token = tokens.next();
    let artifact_event_token = tokens.next();
    let vm_report_event_token = tokens.next();
    let retained_event_token = tokens.next();
    let manifest_reference_token = tokens.next();
    let artifact_reference_token = tokens.next();
    let vm_report_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let grant_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope_or_signature_token = tokens.next();
    let trailing_signature_token = tokens.next();
    let (scope, promotion_signature_token) =
        match (scope_or_signature_token, trailing_signature_token) {
            (Some(scope), Some(signature)) => (scope, Some(signature)),
            (Some(signature), None) if promotion_signature_hex_candidate(signature.as_bytes()) => {
                ("current_boot", Some(signature))
            }
            (Some(scope), None) => (scope, None),
            (None, None) => ("current_boot", None),
            (None, Some(_)) => ("current_boot", None),
        };
    let arity_valid = reference_token.is_some()
        && manifest_event_token.is_some()
        && artifact_event_token.is_some()
        && vm_report_event_token.is_some()
        && retained_event_token.is_some()
        && manifest_reference_token.is_some()
        && artifact_reference_token.is_some()
        && vm_report_reference_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && grant_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && tokens.next().is_none();
    let input = ModuleLocalAttestationReferenceInput {
        has_reference: true,
        arity_valid,
        scope,
        attestation_reference_hash: reference_token.and_then(parse_sha256_ref),
        retained_manifest_reference_event_id: manifest_event_token,
        retained_artifact_reference_event_id: artifact_event_token,
        retained_vm_report_reference_event_id: vm_report_event_token,
        retained_reference_event_id: retained_event_token,
        manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
        artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
        vm_report_reference_hash: vm_report_reference_token.and_then(parse_sha256_ref),
        manifest_hash: manifest_token.and_then(parse_sha256_ref),
        artifact_hash: artifact_token.and_then(parse_sha256_ref),
        computed_grant_hash: grant_token.and_then(parse_sha256_ref),
        vm_report_hash: report_token.and_then(parse_sha256_ref),
        local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        promotion_signature_der: promotion_signature_token.map(str::as_bytes),
        signature_verified: false,
    };
    evaluate_module_local_attestation_reference(input, require_live_retained)
}

fn evaluate_module_local_attestation_reference<'a>(
    input: ModuleLocalAttestationReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleLocalAttestationReferenceCheck<'a> {
    if !input.has_reference {
        return module_local_attestation_reference_check(
            input,
            None,
            None,
            "missing",
            "local_attestation_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_local_attestation_reference_check(
            input,
            None,
            None,
            "invalid_arity",
            "local_attestation_reference_requires_hashes_and_event_ids",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return module_local_attestation_reference_check(
            input,
            None,
            None,
            "stale_or_non_current_boot_reference",
            "local_attestation_reference_scope_must_be_current_boot",
            false,
        );
    }

    let (
        Some(attestation_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_artifact_reference_event_id),
        Some(retained_vm_report_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(artifact_reference_hash),
        Some(vm_report_reference_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(computed_grant_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        input.attestation_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_artifact_reference_event_id,
        input.retained_vm_report_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.artifact_reference_hash,
        input.vm_report_reference_hash,
        input.manifest_hash,
        input.artifact_hash,
        input.computed_grant_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
    )
    else {
        return module_local_attestation_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "local_attestation_reference_hashes_must_be_sha256_refs",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_attestation_reference_hash = computed_module_local_attestation_reference_hash(
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
    );

    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_artifact_reference_event_id) {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_vm_report_reference_event_id) {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_vm_report_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if attestation_reference_hash != expected_attestation_reference_hash {
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_local_attestation_reference_hash",
            "local_attestation_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_local_attestation_live_reference_mismatch(&input) {
            return module_local_attestation_reference_check(
                input,
                Some(expected_attestation_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    if let Some(signature_hex) = input.promotion_signature_der {
        if !verify_promotion_signature_hex(signature_hex, &expected_attestation_reference_hash) {
            return module_local_attestation_reference_check(
                input,
                Some(expected_attestation_reference_hash),
                Some(expected_computed_grant_hash),
                "mismatched_local_attestation_signature",
                "local_attestation_promotion_signature_invalid",
                false,
            );
        }
        return module_local_attestation_reference_check(
            input,
            Some(expected_attestation_reference_hash),
            Some(expected_computed_grant_hash),
            "local_attestation_signature_verified_load_still_denied",
            "local_attestation_promotion_signature_verified_but_loader_and_evidence_missing",
            true,
        );
    }

    module_local_attestation_reference_check(
        input,
        Some(expected_attestation_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "local_attestation_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

const MAX_PROMOTION_SIGNATURE_DER_LEN: usize = 80;

fn verify_promotion_signature_hex(signature_hex: &[u8], payload: &[u8; 32]) -> bool {
    let Some((signature_der, signature_len)) = decode_promotion_signature_der_hex(signature_hex)
    else {
        return false;
    };
    raios_core::promotion_attestation::verify_promotion_authority_signature(
        &signature_der[..signature_len],
        payload,
    )
}

fn decode_promotion_signature_der_hex(
    signature_hex: &[u8],
) -> Option<([u8; MAX_PROMOTION_SIGNATURE_DER_LEN], usize)> {
    if !promotion_signature_hex_candidate(signature_hex) {
        return None;
    }

    let mut out = [0u8; MAX_PROMOTION_SIGNATURE_DER_LEN];
    let mut idx = 0usize;
    while idx < signature_hex.len() / 2 {
        let high = hex_nibble(signature_hex[idx * 2])?;
        let low = hex_nibble(signature_hex[idx * 2 + 1])?;
        out[idx] = (high << 4) | low;
        idx += 1;
    }
    Some((out, idx))
}

fn promotion_signature_hex_candidate(signature_hex: &[u8]) -> bool {
    !signature_hex.is_empty()
        && signature_hex.len() % 2 == 0
        && signature_hex.len() / 2 <= MAX_PROMOTION_SIGNATURE_DER_LEN
        && signature_hex.iter().all(|byte| hex_nibble(*byte).is_some())
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn module_local_attestation_reference_check<'a>(
    input: ModuleLocalAttestationReferenceInput<'a>,
    expected_attestation_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleLocalAttestationReferenceCheck<'a> {
    ModuleLocalAttestationReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        attestation_reference_hash: input.attestation_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_artifact_reference_event_id: input.retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id: input.retained_vm_report_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        artifact_reference_hash: input.artifact_reference_hash,
        vm_report_reference_hash: input.vm_report_reference_hash,
        manifest_hash: input.manifest_hash,
        artifact_hash: input.artifact_hash,
        computed_grant_hash: input.computed_grant_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        promotion_signature_der: input.promotion_signature_der,
        signature_verified: input.signature_verified
            || method_eq(
                status,
                "local_attestation_signature_verified_load_still_denied",
            ),
        expected_attestation_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_local_attestation_live_reference_mismatch(
    input: &ModuleLocalAttestationReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_artifact_reference_event_id =
        parse_current_boot_event_id(input.retained_artifact_reference_event_id?)?;
    let retained_vm_report_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_report_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;

    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("local_attestation_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("local_attestation_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("local_attestation_manifest_reference_hash_mismatch");
    }
    if Some(manifest_reference.manifest_hash) != input.manifest_hash {
        return Some("local_attestation_manifest_hash_mismatch");
    }

    let Some((latest_artifact_event_id, artifact_reference)) =
        event_log::latest_module_candidate_artifact_reference()
    else {
        return Some("local_attestation_artifact_reference_missing");
    };
    if latest_artifact_event_id != retained_artifact_reference_event_id {
        return Some("local_attestation_artifact_reference_mismatch");
    }
    if Some(artifact_reference.artifact_reference_hash) != input.artifact_reference_hash {
        return Some("local_attestation_artifact_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("local_attestation_manifest_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_hash) != input.manifest_hash {
        return Some("local_attestation_manifest_hash_mismatch");
    }
    if Some(artifact_reference.artifact_hash) != input.artifact_hash {
        return Some("local_attestation_artifact_hash_mismatch");
    }
    if Some(artifact_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("local_attestation_hash_mismatch");
    }

    let Some((latest_report_event_id, report_reference)) =
        event_log::latest_module_vm_test_report_reference()
    else {
        return Some("local_attestation_vm_report_reference_missing");
    };
    if latest_report_event_id != retained_vm_report_reference_event_id {
        return Some("local_attestation_vm_report_reference_mismatch");
    }
    if Some(report_reference.report_reference_hash) != input.vm_report_reference_hash {
        return Some("local_attestation_vm_report_reference_hash_mismatch");
    }
    if Some(report_reference.artifact_reference_hash) != input.artifact_reference_hash {
        return Some("local_attestation_artifact_reference_hash_mismatch");
    }
    if Some(report_reference.vm_report_hash) != input.vm_report_hash {
        return Some("local_attestation_vm_report_hash_mismatch");
    }
    if Some(report_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("local_attestation_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("local_attestation_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("local_attestation_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("local_attestation_computed_grant_hash_mismatch");
    }
    if Some(retained_reference.manifest_hash) != input.manifest_hash {
        return Some("local_attestation_manifest_hash_mismatch");
    }
    if Some(retained_reference.artifact_hash) != input.artifact_hash {
        return Some("local_attestation_artifact_hash_mismatch");
    }
    if Some(retained_reference.vm_report_hash) != input.vm_report_hash {
        return Some("local_attestation_vm_report_hash_mismatch");
    }
    if Some(retained_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("local_attestation_hash_mismatch");
    }
    None
}

fn module_local_attestation_selftest_cases(
) -> [ModuleLocalAttestationSelfTestCase; MODULE_LOCAL_ATTESTATION_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_local_attestation_valid_input(),
        &LOCAL_ATTESTATION_CASES,
        apply_local_attestation_selftest_case,
        evaluate_local_attestation_selftest_case,
        module_local_attestation_selftest_case_from_spec,
    )
}

fn module_local_attestation_valid_input<'a>() -> ModuleLocalAttestationReferenceInput<'a> {
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
    let valid_hash = computed_module_local_attestation_reference_hash(
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
    ModuleLocalAttestationReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        attestation_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_artifact_reference_event_id: Some(
            MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        ),
        retained_vm_report_reference_event_id: Some(
            MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        artifact_reference_hash: Some(artifact_reference_hash),
        vm_report_reference_hash: Some(vm_report_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        promotion_signature_der: None,
        signature_verified: false,
    }
}

fn apply_local_attestation_selftest_case(
    candidate: &mut ModuleLocalAttestationReferenceInput<'static>,
    mutation: LocalAttestationSelftestMutation,
) {
    let valid_input = module_local_attestation_valid_input();
    *candidate = match mutation {
        LocalAttestationSelftestMutation::Absent => ModuleLocalAttestationReferenceInput {
            has_reference: false,
            ..valid_input
        },
        LocalAttestationSelftestMutation::Accepted => valid_input,
        LocalAttestationSelftestMutation::SignatureInvalid => {
            ModuleLocalAttestationReferenceInput {
                promotion_signature_der: Some(b"3006020100020100"),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::Stale => ModuleLocalAttestationReferenceInput {
            scope: "previous_boot",
            ..valid_input
        },
        LocalAttestationSelftestMutation::ReferenceHashMismatch => {
            ModuleLocalAttestationReferenceInput {
                attestation_reference_hash: Some([0x99; 32]),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::ComputedGrantHashMismatch => {
            ModuleLocalAttestationReferenceInput {
                computed_grant_hash: Some([0xaa; 32]),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::ManifestEventPreviousBoot => {
            ModuleLocalAttestationReferenceInput {
                retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::ArtifactEventPreviousBoot => {
            ModuleLocalAttestationReferenceInput {
                retained_artifact_reference_event_id: Some("event.previous_boot.00000028"),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::VmReportEventPreviousBoot => {
            ModuleLocalAttestationReferenceInput {
                retained_vm_report_reference_event_id: Some("event.previous_boot.00000029"),
                ..valid_input
            }
        }
        LocalAttestationSelftestMutation::RetainedEventPreviousBoot => {
            ModuleLocalAttestationReferenceInput {
                retained_reference_event_id: Some("event.previous_boot.00000027"),
                ..valid_input
            }
        }
    }
}

fn evaluate_local_attestation_selftest_case(
    candidate: ModuleLocalAttestationReferenceInput<'_>,
    _require_live_retained: bool,
) -> ModuleLocalAttestationReferenceCheck<'_> {
    evaluate_module_local_attestation_reference(candidate, false)
}

fn module_local_attestation_selftest_case_from_spec(
    spec: &CaseSpec<LocalAttestationSelftestMutation>,
    check: ModuleLocalAttestationReferenceCheck<'_>,
) -> ModuleLocalAttestationSelfTestCase {
    let expected_valid = method_eq(
        spec.expected_status,
        "valid_hash_reference_load_still_denied",
    ) || method_eq(
        spec.expected_status,
        "local_attestation_signature_verified_load_still_denied",
    );
    let can_load = false;
    ModuleLocalAttestationSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, spec.expected_status)
            && method_eq(check.reason, spec.expected_reason)
            && check.valid == expected_valid
            && !can_load,
    }
}

fn module_local_attestation_binding_from_check(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
) -> Option<event_log::ModuleLocalAttestationReference> {
    Some(event_log::ModuleLocalAttestationReference {
        attestation_reference_hash: check.attestation_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_artifact_reference_event_id: parse_current_boot_event_id(
            check.retained_artifact_reference_event_id?,
        )?,
        retained_vm_report_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_report_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        artifact_reference_hash: check.artifact_reference_hash?,
        vm_report_reference_hash: check.vm_report_reference_hash?,
        manifest_hash: check.manifest_hash?,
        artifact_hash: check.artifact_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
    })
}

fn module_local_attestation_reference_matches(
    check: &ModuleLocalAttestationReferenceCheck<'_>,
    reference: event_log::ModuleLocalAttestationReference,
) -> bool {
    check.attestation_reference_hash == Some(reference.attestation_reference_hash)
        && (!check.signature_verified || check.promotion_signature_der.is_some())
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
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check.vm_report_reference_hash == Some(reference.vm_report_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

fn module_attestation_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.attestation_diagnostic") {
        "module.attestation_diagnostic".len()
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
