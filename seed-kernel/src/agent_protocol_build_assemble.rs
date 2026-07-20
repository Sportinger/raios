use alloc::{vec, vec::Vec};

use raios_core::{
    host_import_abi_v1::{host_import_abi_ordered_list_sha256, HOST_IMPORT_ABI_V1},
    project_workspace::{Classification, ProjectId},
    record::Value as V,
    sha256_bytes,
};
use raios_wasm_ir::{assemble, RETURN_42_IR};
use spin::Mutex;
use wasmi::{core::ValueType, Engine, Module};

use crate::{
    agent_build_loop,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha, record_sha_or_null, record_str as s,
    },
    project_store, wasm_runtime, workspace_candidate_service,
};

const BUILD_ASSEMBLER_FUEL_BUDGET: u64 = 1_000_000;
const BUILD_ASSEMBLER_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;
const IR_PATH: &str = "main.rwir";
const IR_MEDIA_TYPE: &str = "text/raios-wasm-ir";

#[derive(Clone)]
struct ProducedArtifact {
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    tree_sha256: [u8; 32],
    input_sha256: [u8; 32],
    input_byte_len: usize,
    output_sha256: [u8; 32],
    bytes: Vec<u8>,
}

struct BoundSource {
    project_id: [u8; 16],
    revision_sha256: [u8; 32],
    tree_sha256: [u8; 32],
    input_sha256: [u8; 32],
    bytes: Vec<u8>,
}

struct AssembleEvidence {
    outcome: &'static str,
    source: Option<BoundSource>,
    signed_guest_valid: bool,
    run_one: Option<wasm_runtime::EchoRunEvidence>,
    run_two: Option<wasm_runtime::EchoRunEvidence>,
    kernel_recompute_sha256: [u8; 32],
    guest_kernel_byte_identical: bool,
    fresh_builds_byte_identical: bool,
    wasmi_module_valid: bool,
    produced_import_count: usize,
    exact_entrypoint_export: bool,
    stored_current_boot: bool,
}

static PRODUCED: Mutex<Option<ProducedArtifact>> = Mutex::new(None);

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

pub(crate) fn emit_build_assemble_revision() {
    let evidence = assemble_latest_revision();
    let source = evidence.source.as_ref();
    let run_one = evidence.run_one.as_ref();
    let run_two = evidence.run_two.as_ref();
    let run_one_sha256 = run_one
        .map(|run| sha256_bytes(&run.raw_captured_output))
        .unwrap_or([0; 32]);
    let run_two_sha256 = run_two
        .map(|run| sha256_bytes(&run.raw_captured_output))
        .unwrap_or([0; 32]);
    let byte_identical = evidence.guest_kernel_byte_identical
        && evidence.fresh_builds_byte_identical
        && run_two.is_some();

    begin_response("build.assemble_revision");
    emit_record_fields(
        vec![
            f("method", s("build.assemble_revision")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f(
                "status",
                s(if evidence.outcome == "passed" {
                    "accepted"
                } else {
                    "denied"
                }),
            ),
            f("assemble_outcome", s(evidence.outcome)),
            f("source_path", s(IR_PATH)),
            f("source_media_type", s(IR_MEDIA_TYPE)),
            f("source_classification", s("local_only")),
            f(
                "project_id",
                source
                    .map(|source| V::HexBytes(&source.project_id))
                    .unwrap_or(V::Null),
            ),
            f(
                "revision_sha256",
                record_sha_or_null(source.map(|source| source.revision_sha256)),
            ),
            f(
                "tree_sha256",
                record_sha_or_null(source.map(|source| source.tree_sha256)),
            ),
            f(
                "input_byte_len",
                V::U64(source.map(|source| source.bytes.len()).unwrap_or(0) as u64),
            ),
            f(
                "input_sha256",
                record_sha_or_null(source.map(|source| source.input_sha256)),
            ),
            f(
                "input_binding_verified_before_assembly",
                b(source.is_some()),
            ),
            f("service_id", s(wasm_runtime::BUILD_ASSEMBLER_SERVICE_ID)),
            f(
                "guest_artifact_sha256",
                record_sha(wasm_runtime::BUILD_ASSEMBLER_WASM_ARTIFACT_BYTES_HASH),
            ),
            f(
                "guest_descriptor_sha256",
                record_sha(wasm_runtime::BUILD_ASSEMBLER_WASM_ARTIFACT_IDENTITY_DESCRIPTOR_HASH),
            ),
            f(
                "guest_signature_envelope_sha256",
                record_sha(wasm_runtime::BUILD_ASSEMBLER_WASM_ARTIFACT_SIGNATURE_ENVELOPE_HASH),
            ),
            f("signed_guest_valid", b(evidence.signed_guest_valid)),
            f(
                "assembler_guest_run_outcome",
                s(run_one.map(|run| run.run_outcome).unwrap_or("not_run")),
            ),
            f(
                "assembler_guest_executed",
                b(run_one.is_some_and(assembler_run_succeeded)),
            ),
            f(
                "assembler_guest_run2_outcome",
                s(run_two.map(|run| run.run_outcome).unwrap_or("not_run")),
            ),
            f(
                "assembler_guest_run2_executed",
                b(run_two.is_some_and(assembler_run_succeeded)),
            ),
            f(
                "authorized_host_import_count",
                V::U64(run_one.map(|run| run.authorized_import_count).unwrap_or(0)),
            ),
            f("authorized_host_import_abi", s(HOST_IMPORT_ABI_V1)),
            f(
                "authorized_host_import_list_sha256",
                record_sha(host_import_abi_ordered_list_sha256(
                    HOST_IMPORT_ABI_V1,
                    wasm_runtime::BUILD_ASSEMBLER_AUTHORIZED_IMPORTS,
                )),
            ),
            f(
                "linked_host_import_count",
                V::U64(run_one.map(|run| run.linked_host_import_count).unwrap_or(0)),
            ),
            f(
                "module_imports_within_authorized_list",
                b(run_one.is_some_and(|run| run.module_imports_within_authorized_list)),
            ),
            f("fuel_budget", V::U64(BUILD_ASSEMBLER_FUEL_BUDGET)),
            f(
                "run1_fuel_used",
                V::U64(run_one.map(|run| run.fuel_used).unwrap_or(0)),
            ),
            f(
                "run2_fuel_used",
                V::U64(run_two.map(|run| run.fuel_used).unwrap_or(0)),
            ),
            f(
                "memory_limit_bytes",
                V::U64(BUILD_ASSEMBLER_MAX_MEMORY_BYTES as u64),
            ),
            f("guest_output_sha256", record_sha(run_one_sha256)),
            f(
                "run1_input_sha256",
                record_sha_or_null(source.map(|source| source.input_sha256)),
            ),
            f(
                "run2_input_sha256",
                record_sha_or_null(run_two.and_then(|_| source.map(|source| source.input_sha256))),
            ),
            f("run1_output_sha256", record_sha(run_one_sha256)),
            f("run2_output_sha256", record_sha(run_two_sha256)),
            f(
                "kernel_recompute_sha256",
                record_sha(evidence.kernel_recompute_sha256),
            ),
            f(
                "guest_kernel_byte_identical",
                b(evidence.guest_kernel_byte_identical),
            ),
            f(
                "fresh_builds_byte_identical",
                b(evidence.fresh_builds_byte_identical),
            ),
            f("byte_identical", b(byte_identical)),
            f(
                "output_sha256",
                if evidence.stored_current_boot {
                    record_sha(run_one_sha256)
                } else {
                    V::Null
                },
            ),
            f(
                "output_byte_len",
                V::U64(
                    run_one
                        .map(|run| run.raw_captured_output.len())
                        .unwrap_or(0) as u64,
                ),
            ),
            f("wasmi_module_valid", b(evidence.wasmi_module_valid)),
            f(
                "produced_module_import_count",
                V::U64(evidence.produced_import_count as u64),
            ),
            f(
                "produced_module_imports_empty",
                b(evidence.wasmi_module_valid && evidence.produced_import_count == 0),
            ),
            f(
                "entrypoint",
                s(raios_core::project_runtime::WORKSPACE_ENTRYPOINT),
            ),
            f(
                "exact_entrypoint_export",
                b(evidence.exact_entrypoint_export),
            ),
            f(
                "double_build_scope",
                s("two_independent_guest_instantiations_same_boot"),
            ),
            f(
                "two_fresh_guest_instantiations",
                b(run_one.is_some() && run_two.is_some()),
            ),
            f("fresh_store_double_build_across_reboot", b(false)),
            f("output_retained_in_ram", b(evidence.stored_current_boot)),
            f("retention_scope", s("current_boot")),
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
    end_response("build.assemble_revision");
}

pub(crate) fn emit_build_run_prepare() {
    let produced = PRODUCED.lock().clone();
    let outcome = match produced.as_ref() {
        Some(produced) => workspace_candidate_service::prepare_assembled(
            produced.project_id,
            produced.revision_sha256,
            produced.tree_sha256,
            produced.input_sha256,
            produced.input_byte_len,
            produced.output_sha256,
            produced.bytes.clone(),
        ),
        None => workspace_candidate_service::Outcome {
            accepted: false,
            reason: "build_assembled_output_missing",
            snapshot: workspace_candidate_service::snapshot(),
        },
    };
    let preview = outcome
        .snapshot
        .assembled_binding
        .as_ref()
        .filter(|binding| {
            produced
                .as_ref()
                .is_some_and(|produced| binding.candidate_sha256 == produced.output_sha256)
        });
    let executed = preview.is_some() && outcome.snapshot.run_count != 0;

    begin_response("build.run_prepare");
    emit_record_fields(
        vec![
            f("method", s("build.run_prepare")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("persistence_posture", s("ram_only")),
            f(
                "status",
                s(if outcome.accepted {
                    "accepted"
                } else {
                    "denied"
                }),
            ),
            f("reason", s(outcome.reason)),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(!outcome.accepted)),
            f(
                "project_id",
                produced
                    .as_ref()
                    .map(|produced| V::HexBytes(&produced.project_id))
                    .unwrap_or(V::Null),
            ),
            f(
                "revision_sha256",
                record_sha_or_null(produced.as_ref().map(|item| item.revision_sha256)),
            ),
            f(
                "tree_sha256",
                record_sha_or_null(produced.as_ref().map(|item| item.tree_sha256)),
            ),
            f(
                "input_sha256",
                record_sha_or_null(produced.as_ref().map(|item| item.input_sha256)),
            ),
            f(
                "input_byte_len",
                V::U64(
                    produced
                        .as_ref()
                        .map(|item| item.input_byte_len)
                        .unwrap_or(0) as u64,
                ),
            ),
            f(
                "output_sha256",
                record_sha_or_null(produced.as_ref().map(|item| item.output_sha256)),
            ),
            f(
                "output_byte_len",
                V::U64(produced.as_ref().map(|item| item.bytes.len()).unwrap_or(0) as u64),
            ),
            f(
                "entrypoint",
                s(raios_core::project_runtime::WORKSPACE_ENTRYPOINT),
            ),
            f(
                "fuel_budget",
                V::U64(
                    preview
                        .map(|binding| binding.fuel_budget)
                        .unwrap_or(raios_core::project_runtime::WORKSPACE_FUEL_BUDGET),
                ),
            ),
            f(
                "memory_limit_bytes",
                V::U64(
                    preview
                        .map(|binding| binding.memory_limit_bytes)
                        .unwrap_or(raios_core::project_runtime::WORKSPACE_MEMORY_LIMIT_BYTES)
                        as u64,
                ),
            ),
            f(
                "instance_limit",
                V::U64(
                    preview
                        .map(|binding| binding.instance_limit)
                        .unwrap_or(raios_core::project_runtime::WORKSPACE_INSTANCE_LIMIT)
                        as u64,
                ),
            ),
            f(
                "memory_count_limit",
                V::U64(
                    preview
                        .map(|binding| binding.memory_count_limit)
                        .unwrap_or(raios_core::project_runtime::WORKSPACE_MEMORY_COUNT_LIMIT)
                        as u64,
                ),
            ),
            f(
                "table_limit",
                V::U64(
                    preview
                        .map(|binding| binding.table_limit)
                        .unwrap_or(raios_core::project_runtime::WORKSPACE_TABLE_LIMIT)
                        as u64,
                ),
            ),
            f(
                "import_count",
                V::U64(preview.map(|binding| binding.import_count).unwrap_or(0) as u64),
            ),
            f("produced_module_imports_empty", b(preview.is_some())),
            f(
                "import_list_sha256",
                record_sha_or_null(preview.map(|binding| binding.import_list_sha256)),
            ),
            f(
                "approval_challenge_sha256",
                record_sha_or_null(preview.map(|binding| binding.approval_challenge_sha256)),
            ),
            f("phase", s(outcome.snapshot.phase)),
            f(
                "w5_preview_created",
                b(outcome.accepted && preview.is_some()),
            ),
            f(
                "physical_approval_present",
                b(preview.is_some() && outcome.snapshot.approval.is_some()),
            ),
            f(
                "approval_source",
                if preview.is_some() {
                    s("genesis_pointer")
                } else {
                    V::Null
                },
            ),
            f(
                "approval_sha256",
                record_sha_or_null(preview.and_then(|_| {
                    outcome
                        .snapshot
                        .approval
                        .map(|approval| approval.approval_sha256)
                })),
            ),
            f("serial_approval_authorized", b(false)),
            f(
                "serial_approval_reason",
                s("workspace_physical_pointer_approval_required"),
            ),
            f(
                "run_count",
                V::U64(if preview.is_some() {
                    outcome.snapshot.run_count
                } else {
                    0
                }),
            ),
            f(
                "run_outcome",
                s(if preview.is_some() {
                    outcome.snapshot.last_run_outcome
                } else {
                    "not_run"
                }),
            ),
            f(
                "return_value_i32_bits",
                if preview.is_some() {
                    outcome
                        .snapshot
                        .last_return_value
                        .map(|value| V::U64(value as u32 as u64))
                        .unwrap_or(V::Null)
                } else {
                    V::Null
                },
            ),
            f(
                "fuel_used",
                V::U64(if preview.is_some() {
                    outcome.snapshot.last_fuel_used
                } else {
                    0
                }),
            ),
            f("executed", b(executed)),
            f("one_shot_requires_fresh_prepare", b(true)),
            f("install_attempted", b(false)),
            f("install_authorized", b(false)),
            f("promotion_attempted", b(false)),
            f("promotion_authorized", b(false)),
            f("w6_preview_created", b(false)),
            f("reclog_executable_record_written", b(false)),
            f("artstor_executable_record_written", b(false)),
            f("writes_persistent_state", b(false)),
            f("network_access", b(false)),
            f("secret_access", b(false)),
            f("rollback_effect", b(false)),
        ],
        6,
    );
    end_response("build.run_prepare");
}

fn assemble_latest_revision() -> AssembleEvidence {
    let source = match load_bound_source() {
        Ok(source) => source,
        Err(reason) => return denied_evidence(reason),
    };
    let signed_guest_valid = wasm_runtime::validate_build_assembler_wasm_artifact();
    let run_one =
        wasm_runtime::run_build_assembler_roundtrip(&source.bytes, BUILD_ASSEMBLER_FUEL_BUDGET);
    let kernel = assemble(&source.bytes);
    let kernel_recompute_sha256 = kernel
        .as_ref()
        .map(|wasm| sha256_bytes(wasm.as_slice()))
        .unwrap_or([0; 32]);
    let guest_kernel_byte_identical = kernel
        .as_ref()
        .is_ok_and(|wasm| wasm.as_slice() == run_one.raw_captured_output.as_slice());
    let (wasmi_module_valid, produced_import_count, exact_entrypoint_export) =
        inspect_produced_module(&run_one.raw_captured_output);
    let first_stage_error = if !signed_guest_valid {
        Some("signed_guest_invalid")
    } else if run_one.run_outcome != "success" {
        Some(run_one.run_outcome)
    } else if run_one.return_value != Some(0) {
        Some("guest_return_nonzero")
    } else if let Err(reason) = kernel.as_ref() {
        Some(*reason)
    } else if !guest_kernel_byte_identical {
        Some("guest_kernel_byte_mismatch")
    } else if !wasmi_module_valid {
        Some("produced_module_invalid")
    } else if produced_import_count != 0 {
        Some("produced_module_imports_not_empty")
    } else if !exact_entrypoint_export {
        Some("produced_module_entrypoint_invalid")
    } else {
        None
    };
    let run_two = first_stage_error.is_none().then(|| {
        wasm_runtime::run_build_assembler_roundtrip(&source.bytes, BUILD_ASSEMBLER_FUEL_BUDGET)
    });
    let fresh_builds_byte_identical = run_two
        .as_ref()
        .is_some_and(|run_two| run_one.raw_captured_output == run_two.raw_captured_output);
    let outcome = match (first_stage_error, run_two.as_ref()) {
        (Some(reason), _) => reason,
        (None, Some(run_two)) if run_two.run_outcome != "success" => run_two.run_outcome,
        (None, Some(run_two)) if run_two.return_value != Some(0) => "guest_return_nonzero",
        (None, Some(_)) if !fresh_builds_byte_identical => "fresh_build_output_mismatch",
        (None, Some(_)) => "passed",
        (None, None) => "fresh_build_missing",
    };
    let stored_current_boot = outcome == "passed";
    if stored_current_boot {
        let bytes = run_one.raw_captured_output.clone();
        *PRODUCED.lock() = Some(ProducedArtifact {
            project_id: source.project_id,
            revision_sha256: source.revision_sha256,
            tree_sha256: source.tree_sha256,
            input_sha256: source.input_sha256,
            input_byte_len: source.bytes.len(),
            output_sha256: sha256_bytes(&bytes),
            bytes,
        });
    }
    AssembleEvidence {
        outcome,
        source: Some(source),
        signed_guest_valid,
        run_one: Some(run_one),
        run_two,
        kernel_recompute_sha256,
        guest_kernel_byte_identical,
        fresh_builds_byte_identical,
        wasmi_module_valid,
        produced_import_count,
        exact_entrypoint_export,
        stored_current_boot,
    }
}

fn load_bound_source() -> Result<BoundSource, &'static str> {
    let snapshot = agent_build_loop::snapshot();
    let project_id = snapshot
        .project_id
        .ok_or("build_assemble_project_missing")?;
    let revision = snapshot
        .latest_revision
        .ok_or("build_assemble_revision_missing")?;
    if revision.project_id.bytes() != project_id {
        return Err("build_assemble_revision_binding_mismatch");
    }
    let loaded = project_store::load_revision(ProjectId::new(project_id), revision.revision_sha256)
        .map_err(|error| error.reason())?
        .ok_or("build_assemble_revision_not_readable")?;
    if loaded.revision != revision {
        return Err("build_assemble_revision_binding_mismatch");
    }
    let mut matching = loaded
        .files
        .into_iter()
        .filter(|file| file.entry.path == IR_PATH);
    let file = matching.next().ok_or("build_assemble_main_rwir_missing")?;
    if matching.next().is_some() {
        return Err("build_assemble_main_rwir_file_count_invalid");
    }
    if file.entry.path != IR_PATH
        || file.entry.media_type != IR_MEDIA_TYPE
        || file.entry.classification != Classification::LocalOnly
    {
        return Err("build_assemble_main_rwir_binding_invalid");
    }
    let input_sha256 = sha256_bytes(&file.bytes);
    if input_sha256 != file.entry.blob_sha256 || file.bytes.len() != file.entry.byte_len {
        return Err("build_assemble_input_hash_mismatch");
    }
    Ok(BoundSource {
        project_id,
        revision_sha256: revision.revision_sha256,
        tree_sha256: revision.tree_sha256,
        input_sha256,
        bytes: file.bytes,
    })
}

fn inspect_produced_module(bytes: &[u8]) -> (bool, usize, bool) {
    if bytes.is_empty() {
        return (false, 0, false);
    }
    let engine = Engine::default();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => return (false, 0, false),
    };
    let import_count = module.imports().count();
    let mut exports = module.exports();
    let exact_entrypoint_export = exports.next().is_some_and(|export| {
        export.name() == raios_core::project_runtime::WORKSPACE_ENTRYPOINT
            && export
                .ty()
                .func()
                .is_some_and(|ty| ty.params().is_empty() && ty.results() == [ValueType::I32])
    }) && exports.next().is_none();
    (true, import_count, exact_entrypoint_export)
}

fn assembler_run_succeeded(run: &wasm_runtime::EchoRunEvidence) -> bool {
    run.run_outcome == "success" && run.return_value == Some(0)
}

fn denied_evidence(reason: &'static str) -> AssembleEvidence {
    AssembleEvidence {
        outcome: reason,
        source: None,
        signed_guest_valid: false,
        run_one: None,
        run_two: None,
        kernel_recompute_sha256: [0; 32],
        guest_kernel_byte_identical: false,
        fresh_builds_byte_identical: false,
        wasmi_module_valid: false,
        produced_import_count: 0,
        exact_entrypoint_export: false,
        stored_current_boot: false,
    }
}
