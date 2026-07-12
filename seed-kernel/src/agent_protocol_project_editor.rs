use alloc::{vec, vec::Vec};

use raios_core::{
    project_overlay::{DiffEntry, DiffKind},
    project_workspace::{ProjectRevision, TreeEntry},
    record::{Field, Value as V},
};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    project_editor,
};

pub(crate) fn emit_edit_begin(input: &str) {
    emit(
        "project.edit_begin",
        project_editor::edit_begin(input),
        false,
    );
}

pub(crate) fn emit_edit_file_begin(input: &str) {
    emit(
        "project.edit_file_begin",
        project_editor::edit_file_begin(input),
        false,
    );
}

pub(crate) fn emit_edit_chunk(input: &str) {
    emit(
        "project.edit_chunk",
        project_editor::edit_chunk(input),
        false,
    );
}

pub(crate) fn emit_edit_file_finalize() {
    emit(
        "project.edit_file_finalize",
        project_editor::edit_file_finalize(),
        false,
    );
}

pub(crate) fn emit_edit_delete(input: &str) {
    emit(
        "project.edit_delete",
        project_editor::edit_delete(input),
        false,
    );
}

pub(crate) fn emit_edit_diff() {
    emit("project.edit_diff", project_editor::edit_diff(), true);
}

pub(crate) fn emit_edit_commit() {
    emit("project.edit_commit", project_editor::edit_commit(), false);
}

pub(crate) fn emit_edit_discard() {
    emit(
        "project.edit_discard",
        project_editor::edit_discard(),
        false,
    );
}

fn emit(method: &'static str, result: project_editor::EditResult, diff_response: bool) {
    begin_response(method);
    let mut fields = posture(method);
    fields.extend([
        f(
            "status",
            s(if diff_response && result.accepted {
                "present"
            } else if result.accepted {
                "accepted"
            } else {
                "denied"
            }),
        ),
        f("reason", s(result.reason)),
        f("accepted", b(result.accepted)),
        f("rejected", b(!result.accepted)),
        f("action", s("agent_overlay_commit")),
        f(
            "project_id",
            result
                .project_id
                .as_ref()
                .map(|value| V::HexBytes(value))
                .unwrap_or(V::Null),
        ),
        f(
            "base_revision_sha256",
            record_sha_or_null(result.base_revision_sha256),
        ),
        f("file_count", V::U64(result.file_count as u64)),
        f("total_byte_len", V::U64(result.total_byte_len as u64)),
        f(
            "active_path",
            result.active_path.as_deref().map(s).unwrap_or(V::Null),
        ),
        f("active_byte_len", V::U64(result.active_byte_len as u64)),
        f("expected_byte_len", V::U64(result.expected_byte_len as u64)),
        f("diff_count", V::U64(result.diff.len() as u64)),
        f(
            "diff",
            V::Array(result.diff.iter().map(diff_value).collect()),
        ),
        f(
            "proposed_tree_sha256",
            record_sha_or_null(
                result
                    .proposed_revision
                    .as_ref()
                    .map(|revision| revision.tree_sha256),
            ),
        ),
        f(
            "proposed_revision_sha256",
            record_sha_or_null(
                result
                    .proposed_revision
                    .as_ref()
                    .map(|revision| revision.revision_sha256),
            ),
        ),
    ]);
    fields.extend(committed_fields(result.committed_revision.as_ref()));
    fields.extend(no_authority(
        result.storage_write_attempted,
        result.writes_persistent_state,
    ));
    emit_record_fields(fields, 6);
    end_response(method);
}

fn diff_value(entry: &DiffEntry) -> V<'_> {
    V::InlineObject(vec![
        f(
            "kind",
            s(match entry.kind {
                DiffKind::Add => "add",
                DiffKind::Replace => "replace",
                DiffKind::Delete => "delete",
            }),
        ),
        f("path", s(&entry.path)),
        f(
            "old_blob_sha256",
            record_sha_or_null(entry.old.as_ref().map(|value| value.blob_sha256)),
        ),
        f(
            "old_classification",
            entry
                .old
                .as_ref()
                .map(|value| s(value.classification.label()))
                .unwrap_or(V::Null),
        ),
        f(
            "old_media_type",
            entry
                .old
                .as_ref()
                .map(|value| s(&value.media_type))
                .unwrap_or(V::Null),
        ),
        f(
            "old_byte_len",
            entry
                .old
                .as_ref()
                .map(|value| V::U64(value.byte_len as u64))
                .unwrap_or(V::Null),
        ),
        f(
            "new_blob_sha256",
            record_sha_or_null(entry.new.as_ref().map(|value| value.blob_sha256)),
        ),
        f(
            "new_classification",
            entry
                .new
                .as_ref()
                .map(|value| s(value.classification.label()))
                .unwrap_or(V::Null),
        ),
        f(
            "new_media_type",
            entry
                .new
                .as_ref()
                .map(|value| s(&value.media_type))
                .unwrap_or(V::Null),
        ),
        f(
            "new_byte_len",
            entry
                .new
                .as_ref()
                .map(|value| V::U64(value.byte_len as u64))
                .unwrap_or(V::Null),
        ),
    ])
}

fn committed_fields(revision: Option<&ProjectRevision>) -> Vec<Field<'_>> {
    vec![
        f(
            "parent_revision_sha256",
            record_sha_or_null(revision.and_then(|value| value.parent_revision_sha256)),
        ),
        f(
            "tree_sha256",
            record_sha_or_null(revision.map(|value| value.tree_sha256)),
        ),
        f(
            "revision_sha256",
            record_sha_or_null(revision.map(|value| value.revision_sha256)),
        ),
        f(
            "files",
            V::Array(
                revision
                    .map(|value| value.entries.iter().map(file_value).collect())
                    .unwrap_or_default(),
            ),
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

fn posture(method: &'static str) -> Vec<Field<'static>> {
    vec![
        f("method", s(method)),
        f("scope", s("project")),
        f("classification", s("local_only")),
        f(
            "persistence_posture",
            s("qemu_disposable_structured_store_only"),
        ),
        f("qemu_only", b(true)),
        f("physical_media_supported", b(false)),
        f("physical_media_attempted", b(false)),
    ]
}

fn no_authority(
    storage_write_attempted: bool,
    writes_persistent_state: bool,
) -> Vec<Field<'static>> {
    vec![
        f("storage_write_attempted", b(storage_write_attempted)),
        f("writes_persistent_state", b(writes_persistent_state)),
        f("provider_export_attempted", b(false)),
        f("provider_export_authorized", b(false)),
        f("builder_attempted", b(false)),
        f("build_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
    ]
}
