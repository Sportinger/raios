use alloc::{string::String, vec, vec::Vec};

use raios_core::{
    project_build::{
        build_environment_contract_sha256, build_flags_contract_sha256,
        pinned_toolchain_manifest_sha256, BuildDependencyBundle, BuildDependencyFile, BuildRun,
        BuildSourceFile, ProjectBuildReceipt, ProjectBuildSnapshot, BUILD_CODE_POLICY,
        BUILD_EVIDENCE_QUALITY, BUILD_TARGET, PINNED_TOOLCHAIN_TRUST,
    },
    record::{Field, Value as V},
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    project_build,
};

pub(crate) fn emit_begin(input: &str) {
    emit_build("project.build_begin", project_build::begin(input))
}

pub(crate) fn emit_source_read(input: &str) {
    emit_read(
        "project.build_source_read",
        project_build::source_read(input),
    )
}

pub(crate) fn emit_dependency_read(input: &str) {
    emit_read(
        "project.build_dependency_read",
        project_build::dependency_read(input),
    )
}

pub(crate) fn emit_run(input: &str) {
    emit_build("project.build_run", project_build::record_run(input))
}

pub(crate) fn emit_commit() {
    emit_build("project.build_commit", project_build::commit())
}

pub(crate) fn emit_discard() {
    emit_build("project.build_discard", project_build::discard())
}

pub(crate) fn emit_receipts(input: &str) {
    emit_build("project.build_receipts", project_build::receipts(input))
}

fn emit_build(method: &'static str, view: project_build::BuildView) {
    begin_response(method);
    let snapshot = view
        .receipt
        .as_ref()
        .map(|receipt| &receipt.snapshot)
        .or(view.snapshot.as_ref());
    let receipt = view.receipt.as_ref();
    let project_id = snapshot.map(|item| item.project_id.bytes());
    let mut fields = posture(method);
    fields.extend([
        f(
            "status",
            s(if view.accepted { "accepted" } else { "denied" }),
        ),
        f("reason", s(view.reason)),
        f("accepted", b(view.accepted)),
        f("rejected", b(!view.accepted)),
        f(
            "project_id",
            project_id
                .as_ref()
                .map(|item| V::HexBytes(item))
                .unwrap_or(V::Null),
        ),
        f(
            "project_revision_sha256",
            record_sha_or_null(snapshot.map(|item| item.project_revision_sha256)),
        ),
        f(
            "source_tree_sha256",
            record_sha_or_null(snapshot.map(|item| item.source_tree_sha256)),
        ),
        f(
            "cargo_lock_sha256",
            record_sha_or_null(snapshot.map(|item| item.cargo_lock_sha256)),
        ),
        f(
            "toolchain_manifest_sha256",
            record_sha_or_null(
                receipt
                    .map(|item| item.toolchain_manifest_sha256)
                    .or(view.toolchain_manifest_sha256),
            ),
        ),
        f(
            "input_manifest_sha256",
            record_sha_or_null(snapshot.map(|item| item.input_manifest_sha256)),
        ),
        f(
            "source_file_count",
            V::U64(snapshot.map(|item| item.source_files.len()).unwrap_or(0) as u64),
        ),
        f(
            "dependency_bundle_count",
            V::U64(snapshot.map(|item| item.dependencies.len()).unwrap_or(0) as u64),
        ),
        f(
            "dependency_chunk_count",
            V::U64(snapshot.map(dependency_chunk_count).unwrap_or(0) as u64),
        ),
        f(
            "source_files",
            V::Array(
                snapshot
                    .map(|item| item.source_files.iter().map(source_value).collect())
                    .unwrap_or_default(),
            ),
        ),
        f(
            "dependencies",
            V::Array(
                snapshot
                    .map(|item| item.dependencies.iter().map(dependency_value).collect())
                    .unwrap_or_default(),
            ),
        ),
        f(
            "run_one",
            view.run_one
                .as_ref()
                .or(receipt.map(|item| &item.run_one))
                .map(run_value)
                .unwrap_or(V::Null),
        ),
        f(
            "run_two",
            view.run_two
                .as_ref()
                .or(receipt.map(|item| &item.run_two))
                .map(run_value)
                .unwrap_or(V::Null),
        ),
        f(
            "candidate_sha256",
            record_sha_or_null(receipt.map(|item| item.candidate_sha256)),
        ),
        f(
            "candidate_byte_len",
            V::U64(receipt.map(|item| item.candidate_byte_len).unwrap_or(0) as u64),
        ),
        f(
            "candidate_wasm_valid",
            b(receipt.is_some_and(|item| item.candidate_wasm_valid)),
        ),
        f(
            "receipt_sha256",
            record_sha_or_null(receipt.map(|item| item.receipt_sha256)),
        ),
        f("receipt_present", b(receipt.is_some())),
        f("receipt_count", V::U64(view.receipts.len() as u64)),
        f(
            "receipts",
            V::Array(view.receipts.iter().map(receipt_value).collect()),
        ),
        f(
            "evidence_quality",
            s(if receipt.is_some() {
                BUILD_EVIDENCE_QUALITY
            } else {
                "not_yet_attested"
            }),
        ),
        f("host_build_attested", b(receipt.is_some())),
        f(
            "reproducible",
            b(receipt.is_some_and(|item| item.reproducible)),
        ),
        f(
            "independently_verified",
            b(receipt.is_some_and(|item| item.independently_verified)),
        ),
    ]);
    fields.extend(no_authority());
    emit_record_fields(fields, 6);
    end_response(method)
}

fn emit_read(method: &'static str, view: project_build::ReadView) {
    begin_response(method);
    let bytes_b64 = base64(&view.bytes);
    let mut fields = posture(method);
    fields.extend([
        f(
            "status",
            s(if view.accepted { "accepted" } else { "denied" }),
        ),
        f("reason", s(view.reason)),
        f("accepted", b(view.accepted)),
        f("rejected", b(!view.accepted)),
        f(
            "project_id",
            view.project_id
                .as_ref()
                .map(|item| V::HexBytes(item))
                .unwrap_or(V::Null),
        ),
        f(
            "project_revision_sha256",
            record_sha_or_null(view.revision_sha256),
        ),
        f("path", view.path.as_deref().map(s).unwrap_or(V::Null)),
        f("file_byte_len", V::U64(view.file_byte_len as u64)),
        f("blob_sha256", record_sha_or_null(view.blob_sha256)),
        f("bundle_sha256", record_sha_or_null(view.bundle_sha256)),
        f("tree_sha256", record_sha_or_null(view.tree_sha256)),
        f("whole_sha256", record_sha_or_null(view.whole_sha256)),
        f("chunk_count", V::U64(view.dependency_chunks.len() as u64)),
        f(
            "chunks",
            V::Array(
                view.dependency_chunks
                    .iter()
                    .map(|chunk| {
                        V::InlineObject(vec![
                            f("byte_len", V::U64(chunk.byte_len as u64)),
                            f("sha256", V::Sha256(chunk.sha256)),
                        ])
                    })
                    .collect(),
            ),
        ),
        f("offset", V::U64(view.offset as u64)),
        f("requested_len", V::U64(view.requested_len as u64)),
        f("returned_len", V::U64(view.bytes.len() as u64)),
        f("eof", b(view.eof)),
        f(
            "bytes_b64",
            if view.accepted {
                s(&bytes_b64)
            } else {
                V::Null
            },
        ),
    ]);
    fields.extend(no_authority());
    emit_record_fields(fields, 6);
    end_response(method)
}

fn posture(method: &'static str) -> Vec<Field<'static>> {
    vec![
        f("method", s(method)),
        f("scope", s("project_build")),
        f("classification", s("local_only")),
        f("persistence_posture", s("current_boot_ram_only")),
        f("qemu_only", b(true)),
        f("source_kind", s("owner_workstation_serial")),
        f("target", s(BUILD_TARGET)),
        f(
            "flags_contract_sha256",
            V::Sha256(build_flags_contract_sha256()),
        ),
        f(
            "environment_contract_sha256",
            V::Sha256(build_environment_contract_sha256()),
        ),
        f("build_code_policy", s(BUILD_CODE_POLICY)),
        f("toolchain_trust", s(PINNED_TOOLCHAIN_TRUST)),
        f(
            "pinned_toolchain_manifest_sha256",
            V::Sha256(pinned_toolchain_manifest_sha256()),
        ),
        f("generic_filesystem_exposed", b(false)),
    ]
}

fn no_authority() -> Vec<Field<'static>> {
    vec![
        f("network_attempted", b(false)),
        f("provider_export_attempted", b(false)),
        f("provider_export_authorized", b(false)),
        f("guest_compiler_attempted", b(false)),
        f("guest_build_attempted", b(false)),
        f("guest_build_authorized", b(false)),
        f("storage_write_attempted", b(false)),
        f("writes_persistent_state", b(false)),
        f("promotion_attempted", b(false)),
        f("promotion_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
        f("physical_media_attempted", b(false)),
        f("physical_media_supported", b(false)),
    ]
}

fn source_value(file: &BuildSourceFile) -> V<'_> {
    V::Object(vec![
        f("path", s(&file.path)),
        f("byte_len", V::U64(file.byte_len as u64)),
        f("blob_sha256", V::Sha256(file.blob_sha256)),
    ])
}

fn dependency_value(dependency: &BuildDependencyBundle) -> V<'_> {
    V::Object(vec![
        f("bundle_sha256", V::Sha256(dependency.bundle_sha256)),
        f("tree_sha256", V::Sha256(dependency.tree_sha256)),
        f(
            "files",
            V::Array(dependency.files.iter().map(dependency_file_value).collect()),
        ),
    ])
}

fn dependency_file_value(file: &BuildDependencyFile) -> V<'_> {
    V::Object(vec![
        f("path", s(&file.path)),
        f("whole_byte_len", V::U64(file.whole_byte_len as u64)),
        f("whole_sha256", V::Sha256(file.whole_sha256)),
        f(
            "chunks",
            V::Array(
                file.chunks
                    .iter()
                    .map(|chunk| {
                        V::InlineObject(vec![
                            f("byte_len", V::U64(chunk.byte_len as u64)),
                            f("sha256", V::Sha256(chunk.sha256)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn run_value(run: &BuildRun) -> V<'_> {
    V::Object(vec![
        f("exit_code", V::U64(run.exit_code.unsigned_abs() as u64)),
        f("exit_code_negative", b(run.exit_code < 0)),
        f("stdout_sha256", V::Sha256(run.stdout_sha256)),
        f("stderr_sha256", V::Sha256(run.stderr_sha256)),
        f("output_byte_len", V::U64(run.output_byte_len as u64)),
        f("output_sha256", V::Sha256(run.output_sha256)),
    ])
}

fn receipt_value(receipt: &ProjectBuildReceipt) -> V<'_> {
    V::Object(vec![
        f("receipt_sha256", V::Sha256(receipt.receipt_sha256)),
        f(
            "input_manifest_sha256",
            V::Sha256(receipt.snapshot.input_manifest_sha256),
        ),
        f("candidate_sha256", V::Sha256(receipt.candidate_sha256)),
        f(
            "candidate_byte_len",
            V::U64(receipt.candidate_byte_len as u64),
        ),
        f("reproducible", b(receipt.reproducible)),
        f("independently_verified", b(receipt.independently_verified)),
        f("authorizes_promotion", b(receipt.authorizes_promotion)),
        f("authorizes_install", b(receipt.authorizes_install)),
        f("authorizes_load", b(receipt.authorizes_load)),
        f("authorizes_execution", b(receipt.authorizes_execution)),
    ])
}

fn dependency_chunk_count(snapshot: &ProjectBuildSnapshot) -> usize {
    snapshot
        .dependencies
        .iter()
        .flat_map(|dependency| &dependency.files)
        .map(|file| file.chunks.len())
        .sum()
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(first >> 2) as usize] as char);
        out.push(TABLE[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}
