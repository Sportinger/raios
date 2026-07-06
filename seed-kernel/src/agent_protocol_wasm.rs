use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, raw_line,
        record_bool as b, record_field as f, record_sha, record_static_str_array, record_str as s,
        record_str_or_null,
    },
    wasm_runtime,
};
use raios_core::record::Value as V;

pub(crate) fn emit_wasm_echo_probe() {
    let probe = wasm_runtime::run_echo_probe();

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
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response("wasm.echo_probe");
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
