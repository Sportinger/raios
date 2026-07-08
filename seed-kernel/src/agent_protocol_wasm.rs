use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, raw_line,
        record_bool as b, record_field as f, record_sha, record_static_str_array, record_str as s,
        record_str_or_null,
    },
    memory_store, module_candidate_channel, module_candidate_intake, wasm_runtime,
};
use raios_core::record::Value as V;

pub(crate) fn emit_submit_candidate_chunk(arg: &str) {
    let outcome = module_candidate_channel::submit_candidate_chunk(arg);

    begin_response("module.submit_candidate_chunk");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("module.submit_candidate_chunk")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("chunk_index", V::U64(outcome.chunk_index as u64)),
            f("decoded_byte_len", V::U64(outcome.decoded_byte_len as u64)),
            f("pending_byte_len", V::U64(outcome.pending_byte_len as u64)),
            f(
                "pending_chunk_count",
                V::U64(outcome.pending_chunk_count as u64),
            ),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f(
                "discarded_pending_delivery",
                b(outcome.discarded_pending_delivery),
            ),
            f("reason", s(outcome.reason)),
            f("load_attempted", b(outcome.load_attempted)),
            f("execution_attempted", b(outcome.execution_attempted)),
            f("authorizes_load", b(outcome.authorizes_load)),
            f("authorizes_execution", b(outcome.authorizes_execution)),
            f(
                "writes_persistent_state",
                b(outcome.writes_persistent_state),
            ),
            f(
                "external_delivery_channel",
                s(outcome.external_delivery_channel),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("module.submit_candidate_chunk");
}

pub(crate) fn emit_submit_candidate_finalize() {
    let outcome = module_candidate_channel::submit_candidate_finalize();
    let candidate = &outcome.candidate;

    begin_response("module.submit_candidate_finalize");
    emit_record_fields_trailing_comma(
        vec![
            f("method", s("module.submit_candidate_finalize")),
            f("scope", s(candidate.scope)),
            f("classification", s("local_only")),
            f(
                "delivered_byte_len",
                V::U64(outcome.delivered_byte_len as u64),
            ),
            f(
                "delivered_chunk_count",
                V::U64(outcome.delivered_chunk_count as u64),
            ),
            f("byte_len", V::U64(candidate.byte_len as u64)),
            f("artifact_sha256", record_sha(candidate.artifact_sha256)),
            f("wasm_valid", b(candidate.wasm_valid)),
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
            f(
                "candidate",
                record_candidate_intake_case("serial_console_base64_delivery", candidate),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("module.submit_candidate_finalize");
}

pub(crate) fn emit_wasm_echo_probe() {
    let probe = wasm_runtime::run_echo_probe();
    let candidate_probe = module_candidate_intake::run_candidate_intake_probe();

    begin_response("wasm.echo_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_echo_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(false)),
            f("method", s("wasm.echo_probe")),
            f("service_id", s("svc.demo.echo")),
            f("artifact_id", s("wasm:svc.demo.echo")),
            f("artifact_sha256", record_sha(probe.artifact_hash)),
            f(
                "artifact_identity_descriptor_sha256",
                record_sha(probe.descriptor_hash),
            ),
            f(
                "artifact_signature_envelope_sha256",
                record_sha(probe.signature_envelope_hash),
            ),
            f("validation_ok", b(probe.validation_ok)),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f(
                "granted_host_imports",
                record_static_str_array(&["env.log", "env.counter_get"]),
            ),
            f("host_import_count", V::U64(2)),
            f("instantiation_ok", b(probe.instantiation_ok)),
            f("entrypoint", s("raios_service_main")),
            f("run_outcome", s(probe.run_outcome)),
            f("return_value_i32", record_return_value(probe.return_value)),
            f("fuel_budget", V::U64(probe.fuel_budget)),
            f("fuel_used", V::U64(probe.fuel_used)),
            f("log_prefix", s("WASM_GUEST_LOG")),
            f("log_line_emitted", b(probe.log_line.is_some())),
            f("log_line", record_str_or_null(probe.log_line.as_deref())),
            f("negative_probe", s("forbidden_import_link_failure")),
            f(
                "negative_module_imports",
                record_static_str_array(&["env.forbidden_write"]),
            ),
            f("negative_validation_ok", b(probe.forbidden_validation_ok)),
            f(
                "negative_instantiation_ok",
                b(probe.forbidden_instantiation_ok),
            ),
            f(
                "negative_link_error_kind",
                s(probe.forbidden_link_error_kind),
            ),
            f(
                "negative_missing_import_module",
                record_str_or_null(probe.forbidden_missing_import_module),
            ),
            f(
                "negative_missing_import_name",
                record_str_or_null(probe.forbidden_missing_import_name),
            ),
            f("capability_boundary_held", b(probe.forbidden_boundary_held)),
            f(
                "hardening_case_count",
                V::U64(wasm_runtime::WASM_HARDENING_CASE_COUNT as u64),
            ),
            f(
                "hardening_passed_count",
                V::U64(count_hardening_passed(&probe.hardening_cases)),
            ),
            f(
                "hardening_all_passed",
                b(count_hardening_passed(&probe.hardening_cases)
                    == wasm_runtime::WASM_HARDENING_CASE_COUNT as u64),
            ),
            f(
                "hardening_cases",
                record_hardening_cases(&probe.hardening_cases),
            ),
            f("accepts_external_artifact_bytes", b(false)),
            f("maps_executable_pages", b(false)),
            f("writes_persistent_state", b(false)),
            f("mutates_service_inventory", b(false)),
            f("mutates_global_event_log", b(false)),
            f(
                "candidate_intake",
                record_candidate_intake_probe(&candidate_probe),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.echo_probe");
}

pub(crate) fn emit_wasm_bufecho_probe() {
    let roundtrip = wasm_runtime::run_bufecho_roundtrip(b"raios-m11-bufecho-roundtrip-nonce");
    let negative = wasm_runtime::run_bufecho_unauthorized_probe();
    let audit = memory_store::record_wasm_import_grant_audit(
        wasm_runtime::BUFECHO_SERVICE_ID,
        wasm_runtime::BUFECHO_AUTHORIZED_IMPORTS,
        &roundtrip.run,
    );

    begin_response("wasm.bufecho_probe");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.wasm_bufecho_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("wasm.bufecho_probe")),
            f("service_id", s(wasm_runtime::BUFECHO_SERVICE_ID)),
            f("input_len", V::U64(roundtrip.input_len)),
            f("input_sha256", record_sha(roundtrip.input_sha256)),
            f(
                "captured_output_len",
                V::U64(roundtrip.run.captured_output_len),
            ),
            f(
                "captured_output_sha256",
                record_sha(roundtrip.run.captured_output_sha256),
            ),
            f("run_outcome", s(roundtrip.run.run_outcome)),
            f(
                "authorized_import_count",
                V::U64(roundtrip.run.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(roundtrip.run.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(roundtrip.run.module_imports_within_authorized_list),
            ),
            f("audit_dedupe", s(audit.dedupe)),
            f("audit_record_id", s(audit.record_id)),
            // Provenance for WHY the import-grant audit did/did not durably persist.
            // In this focused current-boot profile the shared durable reclog is not
            // provisioned, so the append is honestly fail-closed to RAM-only; this
            // surfaces the exact underlying reason instead of a bare "denied".
            f(
                "audit_reason",
                s(audit
                    .evidence
                    .as_ref()
                    .map(|evidence| evidence.reason)
                    .unwrap_or("no_append_attempted")),
            ),
            f(
                "negative",
                V::InlineObject(vec![
                    f(
                        "module_imports_within_authorized_list",
                        b(negative.module_imports_within_authorized_list),
                    ),
                    f("run_outcome", s(negative.run_outcome)),
                    f(
                        "missing_import_module",
                        record_str_or_null(negative.missing_import_module.as_deref()),
                    ),
                    f("instantiation_ok", b(negative.instantiation_ok)),
                    f("captured_output_len", V::U64(negative.captured_output_len)),
                ]),
            ),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.bufecho_probe");
}

fn record_return_value(value: Option<i32>) -> V<'static> {
    match value {
        Some(value) if value >= 0 => V::U64(value as u64),
        _ => V::Null,
    }
}

fn count_hardening_passed(cases: &[wasm_runtime::WasmHardeningCase]) -> u64 {
    let mut count = 0u64;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            count += 1;
        }
        idx += 1;
    }
    count
}

fn record_hardening_cases(cases: &[wasm_runtime::WasmHardeningCase]) -> V<'static> {
    let mut values = Vec::new();
    let mut idx = 0usize;
    while idx < cases.len() {
        let case = cases[idx];
        values.push(V::InlineObject(vec![
            f("name", s(case.name)),
            f("mechanism", s(case.mechanism)),
            f("expected_outcome", s(case.expected_outcome)),
            f("actual_outcome", s(case.actual_outcome)),
            f("passed", b(case.passed)),
        ]));
        idx += 1;
    }
    V::Array(values)
}

fn record_candidate_intake_probe(
    probe: &module_candidate_intake::CandidateIntakeProbe,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "max_external_wasm_candidate_bytes",
            V::U64(module_candidate_intake::MAX_EXTERNAL_WASM_CANDIDATE_BYTES as u64),
        ),
        f(
            "external_delivery_channel",
            s(module_candidate_intake::EXTERNAL_WASM_CANDIDATE_DELIVERY_CHANNEL),
        ),
        f("case_count", V::U64(3)),
        f(
            "echo_external_candidate",
            record_candidate_intake_case(
                "echo_external_test_vector",
                &probe.echo_external_candidate,
            ),
        ),
        f(
            "malformed_under_bound_candidate",
            record_candidate_intake_case(
                "malformed_under_bound",
                &probe.malformed_under_bound_candidate,
            ),
        ),
        f(
            "oversize_candidate",
            record_candidate_intake_case("oversize_rejected", &probe.oversize_candidate),
        ),
        f("all_load_denied", b(all_candidate_load_denied(probe))),
        f(
            "all_execution_denied",
            b(all_candidate_execution_denied(probe)),
        ),
        f(
            "all_persistence_denied",
            b(all_candidate_persistence_denied(probe)),
        ),
    ])
}

fn record_candidate_intake_case(
    name: &'static str,
    outcome: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(name)),
        f("byte_len", V::U64(outcome.byte_len as u64)),
        f("artifact_sha256", record_sha(outcome.artifact_sha256)),
        f("wasm_valid", b(outcome.wasm_valid)),
        f("scope", s(outcome.scope)),
        f("retained_in_ram", b(outcome.retained_in_ram)),
        f("rejected", b(outcome.rejected)),
        f("reason", s(outcome.reason)),
        f("load_attempted", b(outcome.load_attempted)),
        f("execution_attempted", b(outcome.execution_attempted)),
        f("authorizes_load", b(outcome.authorizes_load)),
        f("authorizes_execution", b(outcome.authorizes_execution)),
        f(
            "writes_persistent_state",
            b(outcome.writes_persistent_state),
        ),
        f(
            "external_delivery_channel",
            s(outcome.external_delivery_channel),
        ),
    ])
}

fn all_candidate_load_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    candidate_load_denied(&probe.echo_external_candidate)
        && candidate_load_denied(&probe.malformed_under_bound_candidate)
        && candidate_load_denied(&probe.oversize_candidate)
}

fn all_candidate_execution_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    candidate_execution_denied(&probe.echo_external_candidate)
        && candidate_execution_denied(&probe.malformed_under_bound_candidate)
        && candidate_execution_denied(&probe.oversize_candidate)
}

fn all_candidate_persistence_denied(probe: &module_candidate_intake::CandidateIntakeProbe) -> bool {
    !probe.echo_external_candidate.writes_persistent_state
        && !probe
            .malformed_under_bound_candidate
            .writes_persistent_state
        && !probe.oversize_candidate.writes_persistent_state
}

fn candidate_load_denied(outcome: &module_candidate_intake::ExternalWasmCandidateOutcome) -> bool {
    !outcome.load_attempted && !outcome.authorizes_load
}

fn candidate_execution_denied(
    outcome: &module_candidate_intake::ExternalWasmCandidateOutcome,
) -> bool {
    !outcome.execution_attempted && !outcome.authorizes_execution
}
