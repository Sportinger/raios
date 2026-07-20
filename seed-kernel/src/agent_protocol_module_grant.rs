use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_module_reference::{
        common_evidence_status, diagnostic_facts, emit_evidence_v1_response, evidence_record,
        selftest_case, selftest_facts,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        method_eq, method_head_eq, parse_sha256_ref, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_sha_or_null, record_static_str_array,
        record_str as s, run_selftest_cases_with, CaseSpec,
    },
    event_log, granted_candidate_service, module_candidate_intake, module_evidence, wasm_runtime,
};
use raios_core::evidence_response as ev;
use raios_core::evidence_response::Blocked;
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum GrantSelftestMutation {
    Absent,
    Accepted,
    Stale,
    MismatchedManifestHash,
    UnsafePolicyHash,
    SignedFullyBoundAttestation,
    NoRetainedAttestation,
    UnsignedFullyBoundAttestation,
    SignedDifferentGrantAttestation,
    SignedShadowedByUnsignedAttestation,
}

const fn grant_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: GrantSelftestMutation,
) -> CaseSpec<GrantSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const MODULE_GRANT_AUTHORITY_SELFTEST_CASES: usize = MODULE_GRANT_SELFTEST_CASES + 5;

const GRANT_CASES: [CaseSpec<GrantSelftestMutation>; MODULE_GRANT_AUTHORITY_SELFTEST_CASES] = [
    grant_case(
        "absent_reference",
        "missing",
        "computed_capability_grant_reference_absent",
        GrantSelftestMutation::Absent,
    ),
    grant_case(
        "accepted_current_boot_reference_still_denied",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::Accepted,
    ),
    grant_case(
        "stale_previous_boot_reference",
        "stale_or_non_current_boot_reference",
        "computed_grant_reference_scope_must_be_current_boot",
        GrantSelftestMutation::Stale,
    ),
    grant_case(
        "mismatched_manifest_hash_reference",
        "mismatched_computed_grant_hash",
        "computed_grant_hash_mismatch",
        GrantSelftestMutation::MismatchedManifestHash,
    ),
    grant_case(
        "grants_load_now_or_wrong_policy_hash",
        "mismatched_computed_grant_hash",
        "computed_grant_hash_mismatch",
        GrantSelftestMutation::UnsafePolicyHash,
    ),
    grant_case(
        "signed_fully_bound_attestation_grants_capability",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::SignedFullyBoundAttestation,
    ),
    grant_case(
        "no_retained_attestation_no_grant",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::NoRetainedAttestation,
    ),
    grant_case(
        "unsigned_hash_valid_attestation_no_grant",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::UnsignedFullyBoundAttestation,
    ),
    grant_case(
        "signed_different_grant_attestation_no_grant",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::SignedDifferentGrantAttestation,
    ),
    grant_case(
        "signed_record_shadowed_by_unsigned_retain_no_grant",
        "valid_hash_reference_load_still_denied",
        "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        GrantSelftestMutation::SignedShadowedByUnsignedAttestation,
    ),
];

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
    let retained_attestation =
        event_log::latest_module_local_attestation_reference().map(|(_, reference)| reference);
    let readiness = module_grant_live_runtime_readiness();
    let authority =
        module_grant_authority_from_attestation(&check, retained_attestation, readiness);
    let facts = diagnostic_facts(
        "module.grant_diagnostic <computed_grant_hash> <manifest_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]",
        V::InlineObject(vec![f("requested_capability", s("cap.module.load_ephemeral")), f("load_mode", s("ram_only")), f("risk", s("modify_ram")), f("subject", s("agent.session.serial")), f("resource", s("live_service_graph"))]),
        if authority.can_load_now { "dev_key_attestation_plus_retained_artifact_bytes" } else { "hash_reference_only_no_artifact_bytes" },
        record_static_str_array(&["in_guest_evidence_retention", "raios.audit_record.v0", "rollback_plan", "module_loader", "ram_only_service_slot"]),
        s(authority.trust_tier),
        V::InlineObject(vec![f("candidate_bytes_present", b(readiness.retained_candidate_bytes_present)), f("slot_allocatable", b(readiness.slot_allocatable)), f("loader_available", b(readiness.loader_available))]),
    );
    let mut evidence = vec![evidence_record(
        "computed_capability_grant",
        "reference",
        common_evidence_status(check.valid, check.has_reference),
        check.reason,
        None,
        V::InlineObject(
            module_grant_reference_fields(&check)
                .into_iter()
                .map(|mut field| {
                    if field.key == "validation_status" {
                        field.key = "status_detail";
                    }
                    field
                })
                .filter(|field| field.key != "validation_reason")
                .collect(),
        ),
    )];
    if let Some((event_id, reference)) = retained {
        evidence.push(evidence_record(
            "computed_capability_grant_retained",
            "retained_reference",
            "verified",
            "retained_hash_reference_load_still_denied",
            Some(event_id),
            V::InlineObject(vec![
                f("state", s("present")),
                f("retention", s("current_boot_ram_event_log")),
                f(
                    "matches_current_reference",
                    b(module_grant_reference_matches(&check, reference)),
                ),
                f(
                    "record_schema",
                    s("raios.module_computed_grant_reference.v0"),
                ),
                f(
                    "status_detail",
                    s("retained_hash_reference_load_still_denied"),
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
            ]),
        ));
    } else {
        evidence.push(evidence_record(
            "computed_capability_grant_retained",
            "retained_reference",
            "missing",
            "no_valid_computed_grant_reference_retained",
            None,
            V::InlineObject(vec![
                f("state", s("missing")),
                f("retention", s("current_boot_ram_event_log")),
                f("matches_current_reference", no()),
                f(
                    "record_schema",
                    s("raios.module_computed_grant_reference.v0"),
                ),
                f("status_detail", s("missing")),
            ]),
        ));
    }
    let primary = (!check.valid).then_some(Blocked {
        evidence_id: "computed_capability_grant",
        status: common_evidence_status(false, check.has_reference),
        reason: check.reason,
    });
    let denial = ev::module_reference_denial(ev::ModuleReferenceFamily::Grant, primary);
    let attestation_hashes_match = retained_attestation
        .map(|attestation| {
            Some(attestation.computed_grant_hash) == check.grant_hash
                && Some(attestation.manifest_hash) == check.manifest_hash
                && Some(attestation.artifact_hash) == check.artifact_hash
                && Some(attestation.vm_report_hash) == check.vm_report_hash
                && Some(attestation.local_attestation_hash) == check.local_attestation_hash
        })
        .unwrap_or(false);
    let decision = ev::module_grant_decision(
        check.valid,
        retained_attestation
            .map(|value| value.signature_verified)
            .unwrap_or(false),
        attestation_hashes_match,
        readiness.retained_candidate_bytes_present,
        readiness.slot_allocatable,
        readiness.loader_available,
        denial,
    );
    emit_evidence_v1_response(
        "module.grant_diagnostic",
        "module.computed_grant",
        recorded_event_id,
        facts,
        evidence,
        decision,
    );
}

pub(crate) fn emit_module_grant_diagnostic_selftest() {
    let cases = module_grant_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases
        .iter()
        .map(module_grant_selftest_case_record)
        .collect();
    let mut facts = selftest_facts(V::Array(case_records), cases.len(), passed);
    if let V::InlineObject(fields) = &mut facts {
        fields.push(f(
            "co_emission_invariant",
            s("can_load_now_true_implies_trust_tier_dev_key_not_owner_sealed_and_grants_capability"),
        ));
    }

    emit_evidence_v1_response(
        "module.grant_diagnostic_selftest",
        "module.computed_grant.selftest",
        None,
        facts,
        vec![],
        ev::observed("selftest_completed"),
    );
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
    selftest_case(case.name, case.expected_status, case.expected_reason, case.actual_status, case.actual_reason, case.passed)
}

#[rustfmt::skip]
#[derive(Clone, Copy)]
struct ModuleGrantAuthorityResult {
    grants_capability: bool,
    trust_tier: &'static str,
    can_load_now: bool,
}

#[derive(Clone, Copy)]
struct ModuleGrantRuntimeReadiness {
    retained_candidate_bytes_present: bool,
    slot_allocatable: bool,
    loader_available: bool,
}

fn module_grant_authority_from_attestation(
    check: &ModuleGrantReferenceCheck<'_>,
    attestation: Option<event_log::ModuleLocalAttestationReference>,
    readiness: ModuleGrantRuntimeReadiness,
) -> ModuleGrantAuthorityResult {
    let grants_capability = module_grant_grants_capability(check, attestation);
    ModuleGrantAuthorityResult {
        grants_capability,
        trust_tier: module_grant_trust_tier(grants_capability),
        can_load_now: grants_capability
            && readiness.retained_candidate_bytes_present
            && readiness.slot_allocatable
            && readiness.loader_available,
    }
}

pub(crate) fn module_grant_trust_tier(grants_capability: bool) -> &'static str {
    if grants_capability {
        "dev_key_not_owner_sealed"
    } else {
        "unsealed_no_grant"
    }
}

pub(crate) fn module_grant_grants_capability(
    check: &ModuleGrantReferenceCheck<'_>,
    attestation: Option<event_log::ModuleLocalAttestationReference>,
) -> bool {
    let Some(attestation) = attestation else {
        return false;
    };
    check.valid
        && attestation.signature_verified
        && Some(attestation.computed_grant_hash) == check.grant_hash
        && Some(attestation.manifest_hash) == check.manifest_hash
        && Some(attestation.artifact_hash) == check.artifact_hash
        && Some(attestation.vm_report_hash) == check.vm_report_hash
        && Some(attestation.local_attestation_hash) == check.local_attestation_hash
}

pub(crate) fn module_grant_check_from_retained(
    retained: Option<event_log::ModuleComputedGrantReference>,
) -> ModuleGrantReferenceCheck<'static> {
    let Some(reference) = retained else {
        return evaluate_module_grant_reference(
            false,
            true,
            "current_boot",
            None,
            None,
            None,
            None,
            None,
        );
    };
    evaluate_module_grant_reference(
        true,
        true,
        "current_boot",
        Some(reference.computed_grant_hash),
        Some(reference.manifest_hash),
        Some(reference.artifact_hash),
        Some(reference.vm_report_hash),
        Some(reference.local_attestation_hash),
    )
}

fn module_grant_live_runtime_readiness() -> ModuleGrantRuntimeReadiness {
    ModuleGrantRuntimeReadiness {
        retained_candidate_bytes_present: module_candidate_intake::retained().is_some(),
        slot_allocatable: granted_candidate_service::slot_allocatable(),
        loader_available: wasm_runtime::loader_available(),
    }
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

fn module_grant_selftest_cases() -> [ModuleGrantSelfTestCase; MODULE_GRANT_AUTHORITY_SELFTEST_CASES]
{
    run_selftest_cases_with(
        module_grant_selftest_check(GrantSelftestMutation::Absent),
        &GRANT_CASES,
        apply_module_grant_selftest_case,
        evaluate_module_grant_selftest_case,
        module_grant_selftest_case_from_spec,
    )
}

fn apply_module_grant_selftest_case(
    actual: &mut ModuleGrantReferenceCheck<'static>,
    mutation: GrantSelftestMutation,
) {
    *actual = module_grant_selftest_check(mutation);
}

fn evaluate_module_grant_selftest_case(
    actual: ModuleGrantReferenceCheck<'static>,
    _require_live_retained: bool,
) -> ModuleGrantReferenceCheck<'static> {
    actual
}

fn module_grant_selftest_check(
    mutation: GrantSelftestMutation,
) -> ModuleGrantReferenceCheck<'static> {
    let valid_grant = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    match mutation {
        GrantSelftestMutation::Absent => evaluate_module_grant_reference(
            false,
            true,
            "current_boot",
            None,
            None,
            None,
            None,
            None,
        ),
        GrantSelftestMutation::Accepted => evaluate_module_grant_reference(
            true,
            true,
            "current_boot",
            Some(valid_grant),
            Some(MODULE_GRANT_TEST_MANIFEST_HASH),
            Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
            Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
            Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        ),
        GrantSelftestMutation::Stale => evaluate_module_grant_reference(
            true,
            true,
            "previous_boot",
            Some(valid_grant),
            Some(MODULE_GRANT_TEST_MANIFEST_HASH),
            Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
            Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
            Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        ),
        GrantSelftestMutation::MismatchedManifestHash => evaluate_module_grant_reference(
            true,
            true,
            "current_boot",
            Some(valid_grant),
            Some(MODULE_GRANT_MISMATCH_MANIFEST_HASH),
            Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
            Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
            Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        ),
        GrantSelftestMutation::UnsafePolicyHash => evaluate_module_grant_reference(
            true,
            true,
            "current_boot",
            Some([0x66; 32]),
            Some(MODULE_GRANT_TEST_MANIFEST_HASH),
            Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
            Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
            Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
        ),
        GrantSelftestMutation::SignedFullyBoundAttestation
        | GrantSelftestMutation::NoRetainedAttestation
        | GrantSelftestMutation::UnsignedFullyBoundAttestation
        | GrantSelftestMutation::SignedDifferentGrantAttestation
        | GrantSelftestMutation::SignedShadowedByUnsignedAttestation => {
            evaluate_module_grant_reference(
                true,
                true,
                "current_boot",
                Some(valid_grant),
                Some(MODULE_GRANT_TEST_MANIFEST_HASH),
                Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
                Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
                Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
            )
        }
    }
}

fn module_grant_selftest_case_from_spec(
    spec: &CaseSpec<GrantSelftestMutation>,
    actual: ModuleGrantReferenceCheck<'_>,
) -> ModuleGrantSelfTestCase {
    let authority = module_grant_authority_from_attestation(
        &actual,
        module_grant_selftest_attestation(spec.mutation),
        module_grant_selftest_runtime_readiness(spec.mutation),
    );
    let expected_grants_capability =
        module_grant_selftest_expected_grants_capability(spec.mutation);
    let expected_trust_tier = module_grant_trust_tier(expected_grants_capability);
    let expected_can_load_now = expected_grants_capability;
    let co_emission_invariant = !authority.can_load_now
        || (authority.grants_capability
            && method_eq(authority.trust_tier, "dev_key_not_owner_sealed"));
    ModuleGrantSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && authority.grants_capability == expected_grants_capability
            && method_eq(authority.trust_tier, expected_trust_tier)
            && authority.can_load_now == expected_can_load_now
            && !module_grant_check_can_load(&actual)
            && co_emission_invariant,
    }
}

fn module_grant_selftest_runtime_readiness(
    mutation: GrantSelftestMutation,
) -> ModuleGrantRuntimeReadiness {
    ModuleGrantRuntimeReadiness {
        retained_candidate_bytes_present: matches!(
            mutation,
            GrantSelftestMutation::SignedFullyBoundAttestation
        ),
        slot_allocatable: true,
        loader_available: true,
    }
}

fn module_grant_selftest_expected_grants_capability(mutation: GrantSelftestMutation) -> bool {
    matches!(mutation, GrantSelftestMutation::SignedFullyBoundAttestation)
}

fn module_grant_selftest_attestation(
    mutation: GrantSelftestMutation,
) -> Option<event_log::ModuleLocalAttestationReference> {
    match mutation {
        GrantSelftestMutation::SignedFullyBoundAttestation => {
            Some(module_grant_selftest_attestation_reference(true))
        }
        GrantSelftestMutation::UnsignedFullyBoundAttestation
        | GrantSelftestMutation::SignedShadowedByUnsignedAttestation => {
            Some(module_grant_selftest_attestation_reference(false))
        }
        GrantSelftestMutation::SignedDifferentGrantAttestation => {
            let mut reference = module_grant_selftest_attestation_reference(true);
            reference.computed_grant_hash = [0x66; 32];
            reference.manifest_hash = [0x77; 32];
            Some(reference)
        }
        _ => None,
    }
}

fn module_grant_selftest_attestation_reference(
    signature_verified: bool,
) -> event_log::ModuleLocalAttestationReference {
    event_log::ModuleLocalAttestationReference {
        attestation_reference_hash: [0xaa; 32],
        retained_manifest_reference_event_id: module_grant_selftest_event_id(26),
        retained_artifact_reference_event_id: module_grant_selftest_event_id(28),
        retained_vm_report_reference_event_id: module_grant_selftest_event_id(29),
        retained_reference_event_id: module_grant_selftest_event_id(27),
        manifest_reference_hash: [0x55; 32],
        artifact_reference_hash: [0x56; 32],
        vm_report_reference_hash: [0x57; 32],
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        computed_grant_hash: computed_module_grant_hash(
            MODULE_GRANT_TEST_MANIFEST_HASH,
            MODULE_GRANT_TEST_ARTIFACT_HASH,
            MODULE_GRANT_TEST_VM_REPORT_HASH,
            MODULE_GRANT_TEST_ATTESTATION_HASH,
        ),
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        signature_verified,
    }
}

fn module_grant_selftest_event_id(sequence: u64) -> event_log::EventId {
    let mut candidate = sequence;
    loop {
        if let Some(event_id) = event_log::EventId::from_sequence(candidate) {
            return event_id;
        }
        candidate = 1;
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
