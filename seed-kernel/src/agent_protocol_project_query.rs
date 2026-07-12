use alloc::{vec, vec::Vec};

use raios_core::record::{Field, Value as V};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_str as s,
    },
    project_query,
};

pub(crate) fn emit_read(input: &str) {
    let outcome = project_query::read(input);
    begin_response("project.read");
    let mut fields = posture("project.read");
    match &outcome {
        Ok(read) => fields.extend([
            f("status", s("present")),
            f("reason", s("project_read_verified")),
            f("project_id", V::HexBytes(&read.project_id)),
            f("revision_sha256", V::Sha256(read.revision_sha256)),
            f("path", s(&read.path)),
            f("file_classification", s(read.classification.label())),
            f("media_type", s(&read.media_type)),
            f("blob_sha256", V::Sha256(read.blob_sha256)),
            f("file_byte_len", V::U64(read.file_byte_len as u64)),
            f("offset", V::U64(read.offset as u64)),
            f("requested_len", V::U64(read.requested_len as u64)),
            f("returned_len", V::U64(read.bytes.len() as u64)),
            f("eof", b(read.eof)),
            f("bytes_hex", V::HexBytes(&read.bytes)),
        ]),
        Err(reason) => fields.extend([
            f("status", s("denied")),
            f("reason", s(reason)),
            f("project_id", V::Null),
            f("revision_sha256", V::Null),
            f("path", V::Null),
            f("file_classification", V::Null),
            f("media_type", V::Null),
            f("blob_sha256", V::Null),
            f("file_byte_len", V::U64(0)),
            f("offset", V::U64(0)),
            f("requested_len", V::U64(0)),
            f("returned_len", V::U64(0)),
            f("eof", b(false)),
            f("bytes_hex", V::Null),
        ]),
    }
    fields.extend(no_authority());
    emit_record_fields(fields, 6);
    end_response("project.read");
}

pub(crate) fn emit_search(input: &str) {
    let outcome = project_query::search(input);
    begin_response("project.search");
    let mut fields = posture("project.search");
    match &outcome {
        Ok(search) => fields.extend([
            f("status", s("present")),
            f("reason", s("project_search_verified")),
            f("project_id", V::HexBytes(&search.project_id)),
            f("revision_sha256", V::Sha256(search.revision_sha256)),
            f("query_sha256", V::Sha256(search.query_sha256)),
            f("query_byte_len", V::U64(search.query_byte_len as u64)),
            f(
                "searched_file_count",
                V::U64(search.searched_file_count as u64),
            ),
            f("limit", V::U64(search.limit as u64)),
            f("match_count", V::U64(search.matches.len() as u64)),
            f("truncated", b(search.truncated)),
            f(
                "matches",
                V::Array(
                    search
                        .matches
                        .iter()
                        .map(|item| {
                            V::InlineObject(vec![
                                f("path", s(&item.path)),
                                f("byte_offset", V::U64(item.byte_offset as u64)),
                                f("match_len", V::U64(item.match_len as u64)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        Err(reason) => fields.extend([
            f("status", s("denied")),
            f("reason", s(reason)),
            f("project_id", V::Null),
            f("revision_sha256", V::Null),
            f("query_sha256", V::Null),
            f("query_byte_len", V::U64(0)),
            f("searched_file_count", V::U64(0)),
            f("limit", V::U64(0)),
            f("match_count", V::U64(0)),
            f("truncated", b(false)),
            f("matches", V::Array(Vec::new())),
        ]),
    }
    fields.extend(no_authority());
    emit_record_fields(fields, 6);
    end_response("project.search");
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

fn no_authority() -> Vec<Field<'static>> {
    vec![
        f("storage_write_attempted", b(false)),
        f("writes_persistent_state", b(false)),
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
