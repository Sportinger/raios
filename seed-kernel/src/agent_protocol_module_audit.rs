use alloc::vec;

use crate::{
    agent_protocol_module_reference::{
        common_evidence_status, diagnostic_facts, emit_evidence_v1_response, evidence_record,
        selftest_case, selftest_facts,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        current_boot_event_id_str, method_eq, method_head_eq, parse_current_boot_event_id,
        parse_sha256_ref, raw_line, record_bool as b, record_event_or_null, record_false as no,
        record_field as f, record_present_absent, record_sha_fields, record_sha_or_null,
        record_sha_or_null_fields, record_static_str_array, record_str as s, record_str_or_null,
        run_selftest_cases_with, CaseSpec,
    },
    event_log,
    module_evidence::{self, ram_only_service_slot_id_valid, ModuleAuditRecordHashInput},
};
use raios_core::evidence_response as ev;
use raios_core::evidence_response::Blocked;
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum AuditRollbackSelftestMutation {
    Absent,
    Accepted,
    Stale,
    PreviousBootDenialEventId,
    AuditSchemaMismatch,
    RollbackSchemaMismatch,
    SubstitutedAuditRecordHash,
    MismatchedRollbackPlanHash,
    MismatchedComputedGrantHash,
    InvalidRamOnlyServiceSlot,
}

const fn audit_rollback_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: AuditRollbackSelftestMutation,
) -> CaseSpec<AuditRollbackSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const AUDIT_ROLLBACK_CASES: [CaseSpec<AuditRollbackSelftestMutation>;
    MODULE_AUDIT_ROLLBACK_SELFTEST_CASES] = [
    audit_rollback_case(
        "absent_reference",
        "missing",
        "audit_rollback_reference_absent",
        AuditRollbackSelftestMutation::Absent,
    ),
    audit_rollback_case(
        "accepted_current_boot_reference_still_denied",
        "valid_hash_reference_load_still_denied",
        "audit_rollback_reference_valid_but_loader_and_slot_missing",
        AuditRollbackSelftestMutation::Accepted,
    ),
    audit_rollback_case(
        "stale_previous_boot_reference",
        "stale_or_non_current_boot_reference",
        "audit_rollback_reference_scope_must_be_current_boot",
        AuditRollbackSelftestMutation::Stale,
    ),
    audit_rollback_case(
        "previous_boot_denial_event_id",
        "rejected",
        "denial_event_id_not_current_boot",
        AuditRollbackSelftestMutation::PreviousBootDenialEventId,
    ),
    audit_rollback_case(
        "audit_record_schema_mismatch",
        "rejected",
        "audit_record_schema_mismatch",
        AuditRollbackSelftestMutation::AuditSchemaMismatch,
    ),
    audit_rollback_case(
        "rollback_plan_schema_mismatch",
        "rejected",
        "rollback_plan_schema_mismatch",
        AuditRollbackSelftestMutation::RollbackSchemaMismatch,
    ),
    audit_rollback_case(
        "substituted_audit_record_hash",
        "mismatched_audit_record_hash",
        "audit_record_hash_mismatch",
        AuditRollbackSelftestMutation::SubstitutedAuditRecordHash,
    ),
    audit_rollback_case(
        "mismatched_rollback_plan_hash",
        "mismatched_rollback_plan_hash",
        "rollback_plan_hash_mismatch",
        AuditRollbackSelftestMutation::MismatchedRollbackPlanHash,
    ),
    audit_rollback_case(
        "mismatched_computed_grant_hash",
        "mismatched_computed_grant_hash",
        "computed_grant_hash_mismatch",
        AuditRollbackSelftestMutation::MismatchedComputedGrantHash,
    ),
    audit_rollback_case(
        "invalid_ram_only_service_slot",
        "rejected",
        "ram_only_service_slot_id_invalid",
        AuditRollbackSelftestMutation::InvalidRamOnlyServiceSlot,
    ),
];

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

    let facts = diagnostic_facts("module.audit_rollback_diagnostic <audit_record_hash> <rollback_plan_hash> <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> <local_approval_hash> <pre_load_service_inventory_hash> <cleanup_actions_hash> <denial_event_id> <retained_reference_event_id> <ram_only_service_slot_id> [current_boot]",
        V::InlineObject(vec![f("requested_capability", s("cap.module.load_ephemeral")), f("load_mode", s("ram_only")), f("subject", s("agent.session.serial")), f("resource", s("live_service_graph"))]),
        "hash_reference_only_no_durable_audit_or_rollback_write", record_static_str_array(&["durable_audit_record_write", "rollback_plan_installation", "module_loader", "ram_only_service_slot_allocation"]), V::Null, V::InlineObject(vec![f("loader", s("unavailable")), f("service_slot", s("unallocated"))]));
    let mut evidence = vec![evidence_record(
        "audit_rollback_reference",
        "reference",
        common_evidence_status(check.valid, check.has_reference),
        check.reason,
        None,
        V::InlineObject(vec![
            f("state", record_present_absent(check.has_reference)),
            f("status_detail", s(check.status)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("denial_event_id", record_str_or_null(check.denial_event_id)),
            f(
                "retained_computed_grant_reference_event_id",
                record_str_or_null(check.retained_reference_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(check.ram_only_service_slot_id),
            ),
            f(
                "audit_record_hash",
                record_sha_or_null(check.audit_record_hash),
            ),
            f(
                "expected_audit_record_hash",
                record_sha_or_null(check.expected_audit_record_hash),
            ),
            f(
                "rollback_plan_hash",
                record_sha_or_null(check.rollback_plan_hash),
            ),
            f(
                "expected_rollback_plan_hash",
                record_sha_or_null(check.expected_rollback_plan_hash),
            ),
            f(
                "computed_capability_grant_hash",
                record_sha_or_null(check.computed_grant_hash),
            ),
            f(
                "expected_computed_capability_grant_hash",
                record_sha_or_null(check.expected_computed_grant_hash),
            ),
            f("manifest_hash", record_sha_or_null(check.manifest_hash)),
            f("artifact_hash", record_sha_or_null(check.artifact_hash)),
            f(
                "vm_test_report_hash",
                record_sha_or_null(check.vm_report_hash),
            ),
            f(
                "local_attestation_hash",
                record_sha_or_null(check.local_attestation_hash),
            ),
            f(
                "local_approval_hash",
                record_sha_or_null(check.local_approval_hash),
            ),
            f(
                "pre_load_service_inventory_hash",
                record_sha_or_null(check.pre_load_service_inventory_hash),
            ),
            f(
                "cleanup_actions_hash",
                record_sha_or_null(check.cleanup_actions_hash),
            ),
        ]),
    )];
    if let Some((event_id, reference)) = retained {
        evidence.push(evidence_record(
            "audit_rollback_reference_retained",
            "retained_reference",
            "verified",
            "retained_hash_reference_load_still_denied",
            Some(event_id),
            V::InlineObject(vec![
                f("state", s("present")),
                f("retention", s("current_boot_ram_event_log")),
                f(
                    "matches_current_reference",
                    b(module_audit_rollback_reference_matches(&check, reference)),
                ),
                f(
                    "record_schema",
                    s("raios.module_audit_rollback_reference.v0"),
                ),
                f(
                    "status_detail",
                    s("retained_hash_reference_load_still_denied"),
                ),
                f("audit_record_hash", V::Sha256(reference.audit_record_hash)),
                f(
                    "rollback_plan_hash",
                    V::Sha256(reference.rollback_plan_hash),
                ),
                f(
                    "computed_capability_grant_hash",
                    V::Sha256(reference.computed_grant_hash),
                ),
                f("manifest_hash", V::Sha256(reference.manifest_hash)),
                f("artifact_hash", V::Sha256(reference.artifact_hash)),
                f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
                f(
                    "local_attestation_hash",
                    V::Sha256(reference.local_attestation_hash),
                ),
                f(
                    "local_approval_hash",
                    V::Sha256(reference.local_approval_hash),
                ),
                f(
                    "pre_load_service_inventory_hash",
                    V::Sha256(reference.pre_load_service_inventory_hash),
                ),
                f(
                    "cleanup_actions_hash",
                    V::Sha256(reference.cleanup_actions_hash),
                ),
            ]),
        ));
    } else {
        evidence.push(evidence_record(
            "audit_rollback_reference_retained",
            "retained_reference",
            "missing",
            "no_valid_audit_rollback_reference_retained",
            None,
            V::InlineObject(vec![
                f("state", s("missing")),
                f("retention", s("current_boot_ram_event_log")),
                f("matches_current_reference", no()),
                f(
                    "record_schema",
                    s("raios.module_audit_rollback_reference.v0"),
                ),
                f("status_detail", s("missing")),
            ]),
        ));
    }
    let primary = (!check.valid).then_some(Blocked {
        evidence_id: "audit_rollback_reference",
        status: common_evidence_status(false, check.has_reference),
        reason: check.reason,
    });
    emit_evidence_v1_response(
        "module.audit_rollback_diagnostic",
        "module.audit_rollback_reference",
        recorded_event_id,
        facts,
        evidence,
        ev::module_reference_denial(ev::ModuleReferenceFamily::AuditRollback, primary),
    );
    return;
}

pub(crate) fn emit_module_audit_rollback_diagnostic_selftest() {
    let cases = module_audit_rollback_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases
        .iter()
        .map(module_audit_rollback_selftest_case_record)
        .collect();

    emit_evidence_v1_response(
        "module.audit_rollback_diagnostic_selftest",
        "module.audit_rollback_reference.selftest",
        None,
        selftest_facts(V::Array(case_records), cases.len(), passed),
        vec![],
        ev::observed("selftest_completed"),
    );
}

#[rustfmt::skip]
fn module_audit_rollback_selftest_case_record(case: &ModuleAuditRollbackSelfTestCase) -> V<'static> {
    selftest_case(case.name, case.expected_status, case.expected_reason, case.actual_status, case.actual_reason, case.passed)
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
    run_selftest_cases_with(
        module_audit_rollback_valid_input(),
        &AUDIT_ROLLBACK_CASES,
        apply_audit_rollback_selftest_case,
        evaluate_audit_rollback_selftest_case,
        module_audit_rollback_selftest_case_from_spec,
    )
}

fn apply_audit_rollback_selftest_case(
    candidate: &mut ModuleAuditRollbackReferenceInput<'static>,
    mutation: AuditRollbackSelftestMutation,
) {
    let valid = module_audit_rollback_valid_input();
    *candidate = match mutation {
        AuditRollbackSelftestMutation::Absent => ModuleAuditRollbackReferenceInput {
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
        AuditRollbackSelftestMutation::Accepted => valid,
        AuditRollbackSelftestMutation::Stale => ModuleAuditRollbackReferenceInput {
            scope: "previous_boot",
            ..valid
        },
        AuditRollbackSelftestMutation::PreviousBootDenialEventId => {
            ModuleAuditRollbackReferenceInput {
                denial_event_id: Some("event.previous_boot.00000031"),
                ..valid
            }
        }
        AuditRollbackSelftestMutation::AuditSchemaMismatch => ModuleAuditRollbackReferenceInput {
            audit_schema_ok: false,
            ..valid
        },
        AuditRollbackSelftestMutation::RollbackSchemaMismatch => {
            ModuleAuditRollbackReferenceInput {
                rollback_schema_ok: false,
                ..valid
            }
        }
        AuditRollbackSelftestMutation::SubstitutedAuditRecordHash => {
            ModuleAuditRollbackReferenceInput {
                audit_record_hash: Some([0x99; 32]),
                ..valid
            }
        }
        AuditRollbackSelftestMutation::MismatchedRollbackPlanHash => {
            ModuleAuditRollbackReferenceInput {
                rollback_plan_hash: Some([0xaa; 32]),
                ..valid
            }
        }
        AuditRollbackSelftestMutation::MismatchedComputedGrantHash => {
            ModuleAuditRollbackReferenceInput {
                computed_grant_hash: Some([0xbb; 32]),
                ..valid
            }
        }
        AuditRollbackSelftestMutation::InvalidRamOnlyServiceSlot => {
            ModuleAuditRollbackReferenceInput {
                ram_only_service_slot_id: Some("svc.test.0001"),
                ..valid
            }
        }
    }
}

fn evaluate_audit_rollback_selftest_case(
    candidate: ModuleAuditRollbackReferenceInput<'_>,
    _require_live_retained: bool,
) -> ModuleAuditRollbackReferenceCheck<'_> {
    evaluate_module_audit_rollback_reference(candidate)
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

fn module_audit_rollback_selftest_case_from_spec(
    spec: &CaseSpec<AuditRollbackSelftestMutation>,
    actual: ModuleAuditRollbackReferenceCheck<'_>,
) -> ModuleAuditRollbackSelfTestCase {
    ModuleAuditRollbackSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
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
