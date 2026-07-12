use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    project_dependency,
};
use alloc::{vec, vec::Vec};
use raios_core::{
    project_dependency::{ChunkRef, DependencyBundle, DependencyFileEntry},
    record::{Field, Value as V},
};

pub(crate) fn emit_begin(i: &str) {
    emit("project.dependency_begin", project_dependency::begin(i))
}
pub(crate) fn emit_file_begin(i: &str) {
    emit(
        "project.dependency_file_begin",
        project_dependency::file_begin(i),
    )
}
pub(crate) fn emit_chunk(i: &str) {
    emit("project.dependency_chunk", project_dependency::chunk(i))
}
pub(crate) fn emit_file_finalize() {
    emit(
        "project.dependency_file_finalize",
        project_dependency::file_finalize(),
    )
}
pub(crate) fn emit_commit() {
    emit("project.dependency_commit", project_dependency::commit())
}
pub(crate) fn emit_discard() {
    emit(
        "project.dependency_discard",
        project_dependency::discard_session(),
    )
}
pub(crate) fn emit_dependencies(i: &str) {
    emit("project.dependencies", project_dependency::inspect(i))
}

fn emit(method: &'static str, v: project_dependency::ResultView) {
    begin_response(method);
    let mut fields = posture(method);
    let primary = v.bundle.as_ref();
    fields.extend([
        f("status", s(if v.accepted { "accepted" } else { "denied" })),
        f("reason", s(v.reason)),
        f("accepted", b(v.accepted)),
        f("rejected", b(!v.accepted)),
        f(
            "project_id",
            v.project_id
                .as_ref()
                .map(|x| V::HexBytes(x))
                .unwrap_or(V::Null),
        ),
        f(
            "project_revision_sha256",
            record_sha_or_null(v.revision_sha256),
        ),
        f("cargo_lock_sha256", record_sha_or_null(v.cargo_lock_sha256)),
        f("name", v.name.as_deref().map(s).unwrap_or(V::Null)),
        f("version", v.version.as_deref().map(s).unwrap_or(V::Null)),
        f("origin", v.origin.as_deref().map(s).unwrap_or(V::Null)),
        f("origin_verified", b(false)),
        f("origin_evidence", s("owner_declared_unverified")),
        f(
            "license_expression",
            v.license_expression.as_deref().map(s).unwrap_or(V::Null),
        ),
        f(
            "license_path",
            v.license_path.as_deref().map(s).unwrap_or(V::Null),
        ),
        f("license_sha256", record_sha_or_null(v.license_sha256)),
        f("license_verified", b(false)),
        f("license_evidence", s("owner_declared_unverified")),
        f("file_count", V::U64(v.file_count as u64)),
        f("chunk_count", V::U64(v.chunk_count as u64)),
        f(
            "active_path",
            v.active_path.as_deref().map(s).unwrap_or(V::Null),
        ),
        f("active_byte_len", V::U64(v.active_byte_len as u64)),
        f("expected_byte_len", V::U64(v.expected_byte_len as u64)),
        f("chunk_sha256", record_sha_or_null(v.chunk_sha256)),
        f("chunk_byte_len", V::U64(v.chunk_byte_len as u64)),
        f("chunk_persisted", b(v.chunk_persisted)),
        f("bundle_visible", b(v.bundle_visible)),
        f("orphan_chunk_count", V::U64(v.orphan_chunk_count as u64)),
        f(
            "tree_sha256",
            record_sha_or_null(primary.map(|x| x.tree_sha256)),
        ),
        f(
            "bundle_sha256",
            record_sha_or_null(primary.map(|x| x.bundle_sha256)),
        ),
        f(
            "build_script_present",
            b(primary.is_some_and(|x| x.build_script_present)),
        ),
        f(
            "files",
            V::Array(
                primary
                    .map(|x| x.files.iter().map(file_value).collect())
                    .unwrap_or_default(),
            ),
        ),
        f("bundle_count", V::U64(v.bundles.len() as u64)),
        f(
            "bundles",
            V::Array(v.bundles.iter().map(bundle_value).collect()),
        ),
    ]);
    fields.extend(authority(
        v.storage_write_attempted,
        v.writes_persistent_state,
    ));
    emit_record_fields(fields, 6);
    end_response(method)
}

fn bundle_value(x: &DependencyBundle) -> V<'_> {
    V::Object(vec![
        f("name", s(&x.name)),
        f("version", s(&x.version)),
        f("origin", s(&x.origin)),
        f("license_expression", s(&x.license_expression)),
        f("license_path", s(&x.license_path)),
        f("license_sha256", V::Sha256(x.license_sha256)),
        f("tree_sha256", V::Sha256(x.tree_sha256)),
        f("bundle_sha256", V::Sha256(x.bundle_sha256)),
        f("build_script_present", b(x.build_script_present)),
        f("files", V::Array(x.files.iter().map(file_value).collect())),
    ])
}
fn file_value(x: &DependencyFileEntry) -> V<'_> {
    V::Object(vec![
        f("path", s(&x.path)),
        f("classification", s("local_only")),
        f("media_type", s(&x.media_type)),
        f("whole_byte_len", V::U64(x.whole_byte_len as u64)),
        f("whole_sha256", V::Sha256(x.whole_sha256)),
        f("chunk_count", V::U64(x.chunks.len() as u64)),
        f(
            "chunks",
            V::Array(x.chunks.iter().map(chunk_value).collect()),
        ),
    ])
}
fn chunk_value(x: &ChunkRef) -> V<'_> {
    V::InlineObject(vec![
        f("byte_len", V::U64(x.byte_len as u64)),
        f("sha256", V::Sha256(x.sha256)),
    ])
}
fn posture(m: &'static str) -> Vec<Field<'static>> {
    vec![
        f("method", s(m)),
        f("scope", s("project_dependency")),
        f("classification", s("local_only")),
        f(
            "persistence_posture",
            s("qemu_disposable_structured_store_only"),
        ),
        f("qemu_only", b(true)),
        f("source_kind", s("owner_local_serial")),
    ]
}
fn authority(write: bool, writes: bool) -> Vec<Field<'static>> {
    vec![
        f("storage_write_attempted", b(write)),
        f("writes_persistent_state", b(writes)),
        f("network_fetch_attempted", b(false)),
        f("archive_parse_attempted", b(false)),
        f("provider_export_attempted", b(false)),
        f("provider_export_authorized", b(false)),
        f("compiler_attempted", b(false)),
        f("build_attempted", b(false)),
        f("build_authorized", b(false)),
        f("build_script_execution_attempted", b(false)),
        f("build_script_execution_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
        f("secret_access_attempted", b(false)),
        f("physical_media_attempted", b(false)),
        f("physical_media_supported", b(false)),
    ]
}
