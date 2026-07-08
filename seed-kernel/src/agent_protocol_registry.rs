use alloc::{vec, vec::Vec};

use raios_core::{
    distribution_registry::{
        evaluate_distribution_registry_selection, RegistrySelectionDecision,
        RegistrySelectionReason,
    },
    parse_sha256_ref,
    record::Value as V,
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, method_head_eq, record_bool as b,
        record_field as f, record_sha, record_sha_or_null, record_str as s,
    },
    distribution_candidate, module_candidate_intake,
};

use super::distribution_registry;

#[derive(Clone, Copy)]
struct RegistrySelectionRun<'a> {
    parse_ok: bool,
    requested_artifact_sha256: Option<[u8; 32]>,
    selection: Option<RegistrySelectionDecision<'a>>,
    entry_reason: Option<RegistrySelectionReason>,
    staged_candidate: Option<module_candidate_intake::ExternalWasmCandidateOutcome>,
    retained_provenance: Option<distribution_candidate::DistributionCandidateOutcome>,
}

#[derive(Clone, Copy)]
struct SelftestCase {
    name: &'static str,
    passed: bool,
    status: &'static str,
    reason: &'static str,
    selected_for_candidate_intake: bool,
    staged: bool,
    retained_provenance_verified: bool,
    authorizes_load: bool,
    authorizes_execute: bool,
    authorizes_persist: bool,
}

pub(crate) fn emit_registry_selection_diagnostic(arg: &str) {
    let run = run_registry_selection(registry_selection_hash_arg(arg), true);

    begin_response("module.registry_selection_diagnostic");
    emit_record_fields(record_run("module.registry_selection_diagnostic", &run), 6);
    end_response("module.registry_selection_diagnostic");
}

pub(crate) fn emit_registry_selection_diagnostic_selftest() {
    let cases = registry_selection_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);

    begin_response("module.registry_selection_diagnostic_selftest");
    emit_record_fields(
        vec![
            f("method", s("module.registry_selection_diagnostic_selftest")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("passed", b(passed)),
            f("case_count", V::U64(cases.len() as u64)),
            f(
                "cases",
                V::Array(cases.iter().map(record_selftest_case).collect()),
            ),
            f("read_only", b(true)),
            f("durable_write", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
            f("install_authorized", b(false)),
            f("load_authorized", b(false)),
            f("execute_authorized", b(false)),
            f("persist_authorized", b(false)),
        ],
        6,
    );
    end_response("module.registry_selection_diagnostic_selftest");
}

fn run_registry_selection(selector: &str, stage_valid: bool) -> RegistrySelectionRun<'static> {
    let Some(requested_artifact_sha256) = parse_sha256_ref(selector) else {
        return RegistrySelectionRun {
            parse_ok: false,
            requested_artifact_sha256: None,
            selection: None,
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    };

    let entry = match distribution_registry::builtin_echo_entry() {
        Ok(entry) => entry,
        Err(reason) => {
            return RegistrySelectionRun {
                parse_ok: true,
                requested_artifact_sha256: Some(requested_artifact_sha256),
                selection: None,
                entry_reason: Some(reason),
                staged_candidate: None,
                retained_provenance: None,
            }
        }
    };

    let selection = evaluate_distribution_registry_selection(&entry, requested_artifact_sha256);
    if !stage_valid || !selection.selected_for_candidate_intake {
        return RegistrySelectionRun {
            parse_ok: true,
            requested_artifact_sha256: Some(requested_artifact_sha256),
            selection: Some(selection),
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    }

    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );

    RegistrySelectionRun {
        parse_ok: true,
        requested_artifact_sha256: Some(requested_artifact_sha256),
        selection: Some(selection),
        entry_reason: None,
        staged_candidate: Some(staged_candidate),
        retained_provenance: Some(retained_provenance),
    }
}

fn registry_selection_selftest_cases() -> [SelftestCase; 3] {
    let valid = run_registry_selection(
        "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2",
        true,
    );
    let wrong_hash = run_registry_selection(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        true,
    );
    let invalid = run_registry_selection("not-a-sha256", true);

    [
        selftest_case(
            "valid_registry_selection_stages_inert_candidate",
            &valid,
            "selected",
            "registry_entry_selected_for_inert_candidate_intake",
            true,
        ),
        selftest_case(
            "wrong_hash_denied_without_staging",
            &wrong_hash,
            "denied",
            "selection_hash_mismatch",
            false,
        ),
        selftest_case(
            "invalid_selector_denied_without_staging",
            &invalid,
            "denied",
            "invalid_sha256_selector",
            false,
        ),
    ]
}

fn selftest_case(
    name: &'static str,
    run: &RegistrySelectionRun<'_>,
    expected_status: &'static str,
    expected_reason: &'static str,
    expect_staged: bool,
) -> SelftestCase {
    let status = run_status(run);
    let reason = run_reason(run);
    let selected = run
        .selection
        .map(|selection| selection.selected_for_candidate_intake)
        .unwrap_or(false);
    let staged = run
        .staged_candidate
        .map(|candidate| candidate.retained_in_ram && candidate.wasm_valid && !candidate.rejected)
        .unwrap_or(false);
    let retained_provenance_verified = run
        .retained_provenance
        .map(|provenance| provenance.provenance_verified)
        .unwrap_or(false);
    let authorizes_load = run
        .selection
        .map(|selection| selection.authorizes_load)
        .unwrap_or(false);
    let authorizes_execute = run
        .selection
        .map(|selection| selection.authorizes_execute)
        .unwrap_or(false);
    let authorizes_persist = run
        .selection
        .map(|selection| selection.authorizes_persist)
        .unwrap_or(false);

    SelftestCase {
        name,
        passed: status == expected_status
            && reason == expected_reason
            && staged == expect_staged
            && (!expect_staged || retained_provenance_verified)
            && !authorizes_load
            && !authorizes_execute
            && !authorizes_persist,
        status,
        reason,
        selected_for_candidate_intake: selected,
        staged,
        retained_provenance_verified,
        authorizes_load,
        authorizes_execute,
        authorizes_persist,
    }
}

fn record_run<'a>(
    method: &'static str,
    run: &'a RegistrySelectionRun<'a>,
) -> Vec<raios_core::record::Field<'a>> {
    let selection = run.selection;
    vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("parse_ok", b(run.parse_ok)),
        f(
            "requested_artifact_sha256",
            record_sha_or_null(run.requested_artifact_sha256),
        ),
        f("status", s(run_status(run))),
        f("reason", s(run_reason(run))),
        f(
            "selection",
            selection
                .map(|selection| selection.as_record_value())
                .unwrap_or(V::Null),
        ),
        f(
            "entry_id",
            s(distribution_registry::BUILTIN_ECHO_REGISTRY_ENTRY_ID),
        ),
        f(
            "staged_candidate",
            run.staged_candidate
                .as_ref()
                .map(record_candidate_outcome)
                .unwrap_or(V::Null),
        ),
        f(
            "retained_provenance",
            run.retained_provenance
                .as_ref()
                .map(record_retained_provenance)
                .unwrap_or(V::Null),
        ),
        f(
            "staged_only_after_valid_selection",
            b(run.staged_candidate.is_some()
                == selection
                    .map(|selection| selection.selected_for_candidate_intake)
                    .unwrap_or(false)),
        ),
        f(
            "recomputed_sha256_matches_selection",
            b(recomputed_sha256_matches_selection(run)),
        ),
        f("provenance_is_origin_evidence_only", b(true)),
        f("requires_m6_reverify_for_load", b(true)),
        f("load_attempted", b(false)),
        f("execution_attempted", b(false)),
        f("durable_write_attempted", b(false)),
        f("authorizes_acquisition", b(false)),
        f("authorizes_install", b(false)),
        f("authorizes_load", b(false)),
        f("authorizes_execute", b(false)),
        f("authorizes_persist", b(false)),
        f("writes_persistent_state", b(false)),
        f("owner_sealed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("network_attempted", b(false)),
        f("host_import_added", b(false)),
        f("durable_write", b(false)),
    ]
}

fn record_candidate_outcome(
    candidate: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("byte_len", V::U64(candidate.byte_len as u64)),
        f("artifact_sha256", record_sha(candidate.artifact_sha256)),
        f("wasm_valid", b(candidate.wasm_valid)),
        f("scope", s(candidate.scope)),
        f("retained_in_ram", b(candidate.retained_in_ram)),
        f("rejected", b(candidate.rejected)),
        f("reason", s(candidate.reason)),
        f("load_attempted", b(candidate.load_attempted)),
        f("execution_attempted", b(candidate.execution_attempted)),
        f("authorizes_load", b(candidate.authorizes_load)),
        f("authorizes_execution", b(candidate.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(candidate.writes_persistent_state),
        ),
        f(
            "external_delivery_channel",
            s(candidate.external_delivery_channel),
        ),
    ])
}

fn record_retained_provenance(
    provenance: &distribution_candidate::DistributionCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("source_kind", s(provenance.source_kind)),
        f("retained_present", b(provenance.retained_present)),
        f("retained_wasm_valid", b(provenance.retained_wasm_valid)),
        f("artifact_sha256", record_sha(provenance.artifact_sha256)),
        f(
            "provenance_signature_present",
            b(provenance.provenance_signature_present),
        ),
        f("provenance_verified", b(provenance.provenance_verified)),
        f(
            "publisher_key_sha256",
            record_sha(provenance.publisher_key_sha256),
        ),
        f("status", s(provenance.status)),
        f("reason", s(provenance.reason)),
        f("honest", b(provenance.honest)),
        f("load_authorized", b(provenance.load_authorized)),
        f("install_authorized", b(provenance.install_authorized)),
        f("owner_sealed", b(provenance.owner_sealed)),
        f(
            "requires_m6_reverify_for_load",
            b(provenance.requires_m6_reverify_for_load),
        ),
        f("trust_tier", s(provenance.trust_tier)),
        f("load_attempted", b(provenance.load_attempted)),
        f("execution_attempted", b(provenance.execution_attempted)),
        f("authorizes_load", b(provenance.authorizes_load)),
        f("authorizes_execution", b(provenance.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(provenance.writes_persistent_state),
        ),
    ])
}

fn record_selftest_case(case: &SelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("status", s(case.status)),
        f("reason", s(case.reason)),
        f(
            "selected_for_candidate_intake",
            b(case.selected_for_candidate_intake),
        ),
        f("staged", b(case.staged)),
        f(
            "retained_provenance_verified",
            b(case.retained_provenance_verified),
        ),
        f("authorizes_load", b(case.authorizes_load)),
        f("authorizes_execute", b(case.authorizes_execute)),
        f("authorizes_persist", b(case.authorizes_persist)),
        f("passed", b(case.passed)),
    ])
}

fn registry_selection_hash_arg(arg: &str) -> &str {
    let trimmed = arg.trim();
    let payload = if method_head_eq(trimmed, "module.registry_selection_diagnostic") {
        trimmed
            .strip_prefix("module.registry_selection_diagnostic")
            .unwrap_or(trimmed)
    } else {
        trimmed
    };
    payload.split_whitespace().next().unwrap_or("")
}

fn run_status(run: &RegistrySelectionRun<'_>) -> &'static str {
    if !run.parse_ok {
        return "denied";
    }
    if run.entry_reason.is_some() {
        return "denied";
    }
    run.selection
        .map(|selection| selection.status)
        .unwrap_or("denied")
}

fn run_reason(run: &RegistrySelectionRun<'_>) -> &'static str {
    if !run.parse_ok {
        return "invalid_sha256_selector";
    }
    if let Some(reason) = run.entry_reason {
        return reason.as_str();
    }
    run.selection
        .map(|selection| selection.reason.as_str())
        .unwrap_or("registry_entry_unavailable")
}

fn recomputed_sha256_matches_selection(run: &RegistrySelectionRun<'_>) -> bool {
    let Some(candidate) = run.staged_candidate else {
        return false;
    };
    let Some(selection) = run.selection else {
        return false;
    };
    candidate.artifact_sha256 == selection.artifact_sha256
}
