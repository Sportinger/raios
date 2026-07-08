use alloc::{vec, vec::Vec};

use raios_core::{
    distribution_provenance::PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    distribution_registry::{
        evaluate_distribution_registry_selection, ChunkedDeliveryError,
        ChunkedDistributionChunkInput, ChunkedDistributionDelivery, ChunkedDistributionTarget,
        DistributionRegistryEntry, RegistrySelectionDecision, RegistrySelectionReason,
    },
    parse_sha256_ref,
    record::Value as V,
    sha256_bytes,
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, method_head_eq, record_bool as b,
        record_field as f, record_sha, record_sha_or_null, record_str as s,
    },
    distribution_candidate, module_candidate_intake, wasm_runtime,
};

use super::distribution_registry;

#[derive(Clone, Copy)]
struct RegistrySelectionRun<'a> {
    parse_ok: bool,
    requested_artifact_sha256: Option<[u8; 32]>,
    registry_entry_count: usize,
    registry_capacity: usize,
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
            registry_entry_count: 0,
            registry_capacity: 0,
            selection: None,
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    };

    let registry = match distribution_registry::builtin_registry() {
        Ok(registry) => registry,
        Err(reason) => {
            return RegistrySelectionRun {
                parse_ok: true,
                requested_artifact_sha256: Some(requested_artifact_sha256),
                registry_entry_count: 0,
                registry_capacity: 0,
                selection: None,
                entry_reason: Some(reason),
                staged_candidate: None,
                retained_provenance: None,
            }
        }
    };
    let registry_entry_count = registry.len();
    let registry_capacity = registry.capacity();

    let selection = match registry.select_by_hash(requested_artifact_sha256) {
        Ok(selection) => selection,
        Err(reason) => {
            return RegistrySelectionRun {
                parse_ok: true,
                requested_artifact_sha256: Some(requested_artifact_sha256),
                registry_entry_count,
                registry_capacity,
                selection: None,
                entry_reason: Some(reason),
                staged_candidate: None,
                retained_provenance: None,
            }
        }
    };
    if !stage_valid || !selection.selected_for_candidate_intake {
        return RegistrySelectionRun {
            parse_ok: true,
            requested_artifact_sha256: Some(requested_artifact_sha256),
            registry_entry_count,
            registry_capacity,
            selection: Some(selection),
            entry_reason: None,
            staged_candidate: None,
            retained_provenance: None,
        };
    }

    let Some(entry) = registry_entry_for_selection(&registry, &selection) else {
        return RegistrySelectionRun {
            parse_ok: true,
            requested_artifact_sha256: Some(requested_artifact_sha256),
            registry_entry_count,
            registry_capacity,
            selection: Some(selection),
            entry_reason: Some(RegistrySelectionReason::RegistryEntryNotFound),
            staged_candidate: None,
            retained_provenance: None,
        };
    };
    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );

    RegistrySelectionRun {
        parse_ok: true,
        requested_artifact_sha256: Some(requested_artifact_sha256),
        registry_entry_count,
        registry_capacity,
        selection: Some(selection),
        entry_reason: None,
        staged_candidate: Some(staged_candidate),
        retained_provenance: Some(retained_provenance),
    }
}

fn registry_entry_for_selection<'a>(
    registry: &'a raios_core::distribution_registry::DistributionRegistry<'a>,
    selection: &RegistrySelectionDecision<'_>,
) -> Option<DistributionRegistryEntry<'a>> {
    let mut idx = 0usize;
    while idx < registry.len() {
        if let Some(entry) = registry.get(idx) {
            if entry.artifact_sha256 == selection.artifact_sha256 {
                return Some(*entry);
            }
        }
        idx += 1;
    }
    None
}

fn registry_selection_selftest_cases() -> [SelftestCase; 5] {
    let valid_echo = run_registry_selection(
        "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2",
        true,
    );
    let valid_bufecho = run_registry_selection(
        "sha256:1983797d9ecc6f3f85deedc0c82a8651062f01dc80710ee699e834a51c52e544",
        true,
    );
    let chunked_bufecho = chunked_bufecho_selftest_case();
    let wrong_hash = run_registry_selection(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        true,
    );
    let invalid = run_registry_selection("not-a-sha256", true);

    [
        selftest_case(
            "valid_echo_registry_selection_stages_inert_candidate",
            &valid_echo,
            "selected",
            "registry_entry_selected_for_inert_candidate_intake",
            true,
        ),
        selftest_case(
            "valid_bufecho_registry_selection_stages_inert_candidate",
            &valid_bufecho,
            "selected",
            "registry_entry_selected_for_inert_candidate_intake",
            true,
        ),
        chunked_bufecho,
        selftest_case(
            "wrong_hash_denied_without_staging",
            &wrong_hash,
            "denied",
            "registry_entry_not_found",
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

fn chunked_bufecho_selftest_case() -> SelftestCase {
    match run_chunked_bufecho_selection() {
        Ok((selection, staged_candidate, retained_provenance)) => {
            let staged = staged_candidate.retained_in_ram
                && staged_candidate.wasm_valid
                && !staged_candidate.rejected;
            let retained_provenance_verified = retained_provenance.provenance_verified;
            SelftestCase {
                name: "chunked_bufecho_delivery_stages_inert_candidate",
                passed: selection.status == "selected"
                    && selection.reason.as_str()
                        == "registry_entry_selected_for_inert_candidate_intake"
                    && selection.selected_for_candidate_intake
                    && staged
                    && retained_provenance_verified
                    && !selection.authorizes_load
                    && !selection.authorizes_execute
                    && !selection.authorizes_persist,
                status: selection.status,
                reason: selection.reason.as_str(),
                selected_for_candidate_intake: selection.selected_for_candidate_intake,
                staged,
                retained_provenance_verified,
                authorizes_load: selection.authorizes_load,
                authorizes_execute: selection.authorizes_execute,
                authorizes_persist: selection.authorizes_persist,
            }
        }
        Err(reason) => SelftestCase {
            name: "chunked_bufecho_delivery_stages_inert_candidate",
            passed: false,
            status: "denied",
            reason,
            selected_for_candidate_intake: false,
            staged: false,
            retained_provenance_verified: false,
            authorizes_load: false,
            authorizes_execute: false,
            authorizes_persist: false,
        },
    }
}

fn run_chunked_bufecho_selection() -> Result<
    (
        RegistrySelectionDecision<'static>,
        module_candidate_intake::ExternalWasmCandidateOutcome,
        distribution_candidate::DistributionCandidateOutcome,
    ),
    &'static str,
> {
    let bytes = wasm_runtime::BUFECHO_WASM_ARTIFACT_BYTES;
    let first_end = bytes.len() / 3;
    let second_end = (bytes.len() * 2) / 3;
    let mut delivery = ChunkedDistributionDelivery::new(ChunkedDistributionTarget {
        entry_id: distribution_registry::BUILTIN_BUFECHO_REGISTRY_ENTRY_ID,
        content_sha256: wasm_runtime::BUFECHO_WASM_ARTIFACT_BYTES_HASH,
        total_length: bytes.len(),
        chunk_count: 3,
        provenance_signature_der: Some(
            distribution_registry::BUILTIN_BUFECHO_PROVENANCE_SIGNATURE_DER,
        ),
        publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
        classification: "local_only",
    });
    for (index, chunk) in [
        (2usize, &bytes[second_end..]),
        (0usize, &bytes[..first_end]),
        (1usize, &bytes[first_end..second_end]),
    ] {
        delivery
            .accept_chunk(ChunkedDistributionChunkInput {
                index,
                bytes: chunk,
                claimed_chunk_sha256: sha256_bytes(chunk),
            })
            .map_err(ChunkedDeliveryError::as_str)?;
    }

    let mut reassembled = vec![0u8; bytes.len()];
    let entry = delivery
        .try_finalize(&mut reassembled)
        .map_err(ChunkedDeliveryError::as_str)?;
    let selection = evaluate_distribution_registry_selection(&entry, entry.artifact_sha256);
    if !selection.selected_for_candidate_intake {
        return Err(selection.reason.as_str());
    }
    let staged_candidate = module_candidate_intake::intake_and_retain_external_wasm_candidate(
        Vec::from(entry.artifact_bytes),
    );
    let retained_provenance = distribution_candidate::verify_retained_candidate_provenance(
        entry.provenance_signature_der,
    );

    Ok((
        RegistrySelectionDecision {
            entry_id: distribution_registry::BUILTIN_BUFECHO_REGISTRY_ENTRY_ID,
            ..selection
        },
        staged_candidate,
        retained_provenance,
    ))
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
        f(
            "registry_entry_count",
            V::U64(run.registry_entry_count as u64),
        ),
        f("registry_capacity", V::U64(run.registry_capacity as u64)),
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
            s(selection.map(|s| s.entry_id).unwrap_or("none")),
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
