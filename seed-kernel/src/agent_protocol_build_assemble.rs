use alloc::vec;

use raios_core::{record::Value as V, sha256_bytes};
use raios_wasm_ir::{assemble, RETURN_42_IR};
use wasmi::{Engine, Module};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha, record_str as s,
    },
    wasm_runtime,
};

const BUILD_ASSEMBLER_FUEL_BUDGET: u64 = 1_000_000;
const BUILD_ASSEMBLER_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;

pub(crate) fn emit_build_assemble_probe() {
    let input_sha256 = sha256_bytes(RETURN_42_IR);
    let signed_guest_valid = wasm_runtime::validate_build_assembler_wasm_artifact();
    let guest =
        wasm_runtime::run_build_assembler_roundtrip(RETURN_42_IR, BUILD_ASSEMBLER_FUEL_BUDGET);
    let guest_output = &guest.raw_captured_output;
    let guest_output_sha256 = sha256_bytes(guest_output);
    let kernel = assemble(RETURN_42_IR);
    let kernel_recompute_sha256 = kernel
        .as_ref()
        .map(|wasm| sha256_bytes(wasm.as_slice()))
        .unwrap_or([0; 32]);
    let byte_identical = kernel
        .as_ref()
        .map(|wasm| wasm.as_slice() == guest_output.as_slice())
        .unwrap_or(false);
    let validation_engine = Engine::default();
    let wasmi_module_valid = !guest_output.is_empty()
        && Module::new(&validation_engine, guest_output.as_slice()).is_ok();
    let assembler_guest_executed = guest.run_outcome == "success" && guest.return_value == Some(0);
    let probe_outcome = if !signed_guest_valid {
        "signed_guest_invalid"
    } else if guest.run_outcome != "success" {
        guest.run_outcome
    } else if !assembler_guest_executed {
        "guest_return_nonzero"
    } else if kernel.is_err() {
        "kernel_recompute_failed"
    } else if !byte_identical {
        "guest_kernel_byte_mismatch"
    } else if !wasmi_module_valid {
        "produced_module_invalid"
    } else {
        "passed"
    };

    begin_response("build.assemble_probe");
    emit_record_fields(
        vec![
            f("schema", s("raios.build_assemble_probe.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s("build.assemble_probe")),
            f("service_id", s(wasm_runtime::BUILD_ASSEMBLER_SERVICE_ID)),
            f("probe_outcome", s(probe_outcome)),
            f("input_byte_len", V::U64(RETURN_42_IR.len() as u64)),
            f("input_sha256", record_sha(input_sha256)),
            f(
                "guest_artifact_sha256",
                record_sha(wasm_runtime::BUILD_ASSEMBLER_WASM_ARTIFACT_BYTES_HASH),
            ),
            f("signed_guest_valid", b(signed_guest_valid)),
            f("assembler_guest_run_outcome", s(guest.run_outcome)),
            f("assembler_guest_executed", b(assembler_guest_executed)),
            f(
                "authorized_host_import_count",
                V::U64(guest.authorized_import_count),
            ),
            f(
                "linked_host_import_count",
                V::U64(guest.linked_host_import_count),
            ),
            f(
                "module_imports_within_authorized_list",
                b(guest.module_imports_within_authorized_list),
            ),
            f("fuel_budget", V::U64(BUILD_ASSEMBLER_FUEL_BUDGET)),
            f("fuel_used", V::U64(guest.fuel_used)),
            f(
                "memory_limit_bytes",
                V::U64(BUILD_ASSEMBLER_MAX_MEMORY_BYTES as u64),
            ),
            f("guest_output_sha256", record_sha(guest_output_sha256)),
            f(
                "kernel_recompute_sha256",
                record_sha(kernel_recompute_sha256),
            ),
            f("byte_identical", b(byte_identical)),
            f("output_byte_len", V::U64(guest_output.len() as u64)),
            f("wasmi_module_valid", b(wasmi_module_valid)),
            f("guest_output_inert", b(true)),
            f("executed", b(false)),
            f("accepts_external_artifact_bytes", b(false)),
            f("candidate_intake_attempted", b(false)),
            f("executable_candidate_created", b(false)),
            f("maps_produced_artifact_executable_pages", b(false)),
            f("load_attempted", b(false)),
            f("load_authorized", b(false)),
            f("execution_attempted", b(false)),
            f("execution_authorized", b(false)),
            f("install_attempted", b(false)),
            f("install_authorized", b(false)),
            f("promotion_attempted", b(false)),
            f("promotion_authorized", b(false)),
            f("service_started", b(false)),
            f("w5_preview_created", b(false)),
            f("w6_preview_created", b(false)),
            f("reclog_executable_record_written", b(false)),
            f("artstor_executable_record_written", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_access", b(false)),
            f("secret_access", b(false)),
            f("rollback_effect", b(false)),
            f("service_inventory_mutation", s("none")),
        ],
        6,
    );
    end_response("build.assemble_probe");
}
