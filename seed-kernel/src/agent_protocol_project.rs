use alloc::{vec, vec::Vec};

use raios_core::{
    project_workspace::{ProjectRevision, TreeEntry},
    record::Value as V,
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    project_workspace,
};

const EMPTY_PROJECT_ID: [u8; 16] = [0; 16];

pub(crate) fn emit_import_begin(input: &str) {
    emit_operation(
        "project.import_begin",
        project_workspace::import_begin(input),
    );
}

pub(crate) fn emit_import_file_begin(input: &str) {
    emit_operation(
        "project.import_file_begin",
        project_workspace::import_file_begin(input),
    );
}

pub(crate) fn emit_import_chunk(input: &str) {
    emit_operation(
        "project.import_chunk",
        project_workspace::import_chunk(input),
    );
}

pub(crate) fn emit_import_file_finalize() {
    emit_operation(
        "project.import_file_finalize",
        project_workspace::import_file_finalize(),
    );
}

pub(crate) fn emit_import_commit() {
    emit_operation("project.import_commit", project_workspace::import_commit());
}

pub(crate) fn emit_inspect(input: &str) {
    let outcome = project_workspace::inspect(input);
    let project_id_bytes = outcome
        .as_ref()
        .ok()
        .and_then(|revision| revision.as_ref())
        .map(|revision| revision.project_id.bytes());
    begin_response("project.inspect");
    let mut fields = posture_fields("project.inspect");
    match &outcome {
        Ok(Some(revision)) => {
            fields.extend([
                f("status", s("present")),
                f("reason", s("project_revision_verified")),
                f("present", b(true)),
            ]);
            fields.extend(revision_fields(
                &revision,
                project_id_bytes.as_ref().unwrap_or(&EMPTY_PROJECT_ID),
            ));
        }
        Ok(None) => fields.extend([
            f("status", s("absent")),
            f("reason", s("project_not_found")),
            f("present", b(false)),
            f("project_id", V::Null),
            f("parent_revision_sha256", V::Null),
            f("tree_sha256", V::Null),
            f("revision_sha256", V::Null),
            f("file_count", V::U64(0)),
            f("total_byte_len", V::U64(0)),
            f("files", V::Array(Vec::new())),
        ]),
        Err(reason) => fields.extend([
            f("status", s("denied")),
            f("reason", s(reason)),
            f("present", b(false)),
            f("project_id", V::Null),
            f("parent_revision_sha256", V::Null),
            f("tree_sha256", V::Null),
            f("revision_sha256", V::Null),
            f("file_count", V::U64(0)),
            f("total_byte_len", V::U64(0)),
            f("files", V::Array(Vec::new())),
        ]),
    }
    fields.extend(non_authority_fields(false, false));
    emit_record_fields(fields, 6);
    end_response("project.inspect");
}

fn emit_operation(method: &'static str, outcome: project_workspace::OperationResult) {
    begin_response(method);
    let mut fields = posture_fields(method);
    fields.extend([
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
            outcome
                .project_id
                .as_ref()
                .map(|value| V::HexBytes(value))
                .unwrap_or(V::Null),
        ),
        f("file_count", V::U64(outcome.file_count as u64)),
        f("total_byte_len", V::U64(outcome.total_byte_len as u64)),
        f(
            "active_path",
            outcome.active_path.as_deref().map(s).unwrap_or(V::Null),
        ),
        f("active_byte_len", V::U64(outcome.active_byte_len as u64)),
        f(
            "expected_byte_len",
            V::U64(outcome.expected_byte_len as u64),
        ),
        f(
            "parent_revision_sha256",
            record_sha_or_null(
                outcome
                    .revision
                    .as_ref()
                    .and_then(|revision| revision.parent_revision_sha256),
            ),
        ),
        f(
            "tree_sha256",
            record_sha_or_null(
                outcome
                    .revision
                    .as_ref()
                    .map(|revision| revision.tree_sha256),
            ),
        ),
        f(
            "revision_sha256",
            record_sha_or_null(
                outcome
                    .revision
                    .as_ref()
                    .map(|revision| revision.revision_sha256),
            ),
        ),
        f(
            "files",
            V::Array(
                outcome
                    .revision
                    .as_ref()
                    .map(|revision| revision.entries.iter().map(file_value).collect())
                    .unwrap_or_default(),
            ),
        ),
    ]);
    fields.extend(non_authority_fields(
        outcome.storage_write_attempted,
        outcome.writes_persistent_state,
    ));
    emit_record_fields(fields, 6);
    end_response(method);
}

fn posture_fields(method: &'static str) -> Vec<raios_core::record::Field<'static>> {
    vec![
        f("method", s(method)),
        f("scope", s("project")),
        f("classification", s("local_only")),
        f("retention", s("durable_when_committed")),
        f(
            "persistence_posture",
            s("qemu_disposable_structured_store_only"),
        ),
        f("qemu_only", b(true)),
        f("physical_media_supported", b(false)),
        f("physical_media_attempted", b(false)),
    ]
}

fn revision_fields<'a>(
    revision: &'a ProjectRevision,
    project_id: &'a [u8; 16],
) -> Vec<raios_core::record::Field<'a>> {
    let total_byte_len: usize = revision.entries.iter().map(|entry| entry.byte_len).sum();
    vec![
        f("project_id", V::HexBytes(project_id)),
        f(
            "parent_revision_sha256",
            record_sha_or_null(revision.parent_revision_sha256),
        ),
        f("tree_sha256", V::Sha256(revision.tree_sha256)),
        f("revision_sha256", V::Sha256(revision.revision_sha256)),
        f("file_count", V::U64(revision.entries.len() as u64)),
        f("total_byte_len", V::U64(total_byte_len as u64)),
        f(
            "files",
            V::Array(revision.entries.iter().map(file_value).collect()),
        ),
    ]
}

fn file_value(entry: &TreeEntry) -> V<'_> {
    V::InlineObject(vec![
        f("path", s(&entry.path)),
        f("classification", s(entry.classification.label())),
        f("media_type", s(&entry.media_type)),
        f("byte_len", V::U64(entry.byte_len as u64)),
        f("blob_sha256", V::Sha256(entry.blob_sha256)),
    ])
}

fn non_authority_fields(
    storage_write_attempted: bool,
    writes_persistent_state: bool,
) -> Vec<raios_core::record::Field<'static>> {
    vec![
        f("storage_write_attempted", b(storage_write_attempted)),
        f("writes_persistent_state", b(writes_persistent_state)),
        f("builder_attempted", b(false)),
        f("build_authorized", b(false)),
        f("provider_export_attempted", b(false)),
        f("provider_export_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
    ]
}
