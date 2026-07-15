use alloc::vec;

use raios_core::record::Value as V;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        parse_sha256_ref, record_sha_or_null, record_str as s, record_str_or_null,
    },
    event_log, program_persistence, program_workspace, ui,
};

pub(crate) fn emit_submit_chunk(input: &str) {
    let outcome = program_workspace::submit_serial_chunk(input);
    begin_response("program.submit_chunk");
    emit_record_fields(
        vec![
            f("method", s("program.submit_chunk")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f("decoded_byte_len", V::U64(outcome.decoded_byte_len as u64)),
            f("pending_byte_len", V::U64(outcome.pending_byte_len as u64)),
            f(
                "pending_chunk_count",
                V::U64(outcome.pending_chunk_count as u64),
            ),
            f(
                "discarded_pending_delivery",
                b(outcome.discarded_pending_delivery),
            ),
            f("signing_attempted", b(false)),
            f("load_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("writes_persistent_state", b(false)),
        ],
        6,
    );
    end_response("program.submit_chunk");
}

pub(crate) fn emit_finalize() {
    let outcome = program_workspace::finalize_serial();
    begin_response("program.submit_finalize");
    emit_intake(outcome, "program.submit_finalize");
    end_response("program.submit_finalize");
}

pub(crate) fn emit_workspace() {
    begin_response("program.workspace");
    emit_snapshot(program_workspace::snapshot(), "program.workspace", None);
    end_response("program.workspace");
}

pub(crate) fn emit_rollback_preview(input: &str) {
    let program_sha256 = parse_program_hash(input, "program.rollback_preview");
    let result = program_sha256
        .ok_or("ui_program_sha256_required")
        .and_then(program_persistence::rollback_preview);
    emit_rollback_result(
        "program.rollback_preview",
        program_sha256,
        result,
        "program_rollback_ready",
        false,
    );
}

pub(crate) fn emit_rollback_apply(
    _runtime: ui::RuntimeStatus,
    input: &str,
    event_id: event_log::EventId,
) {
    let program_sha256 = parse_program_hash(input, "program.rollback_apply");
    let result = program_sha256
        .ok_or("ui_program_sha256_required")
        .and_then(|hash| program_persistence::rollback_apply(hash, event_id));
    let writes_persistent_state = result.is_ok();
    emit_rollback_result(
        "program.rollback_apply",
        program_sha256,
        result,
        "program_unpromoted",
        writes_persistent_state,
    );
}

fn parse_program_hash(input: &str, method: &str) -> Option<[u8; 32]> {
    let mut parts = input.split_ascii_whitespace();
    if parts.next() != Some(method) {
        return None;
    }
    let hash = parts.next().and_then(parse_sha256_ref)?;
    parts.next().is_none().then_some(hash)
}

fn emit_rollback_result(
    method: &'static str,
    program_sha256: Option<[u8; 32]>,
    result: Result<(), &'static str>,
    accepted_reason: &'static str,
    writes_persistent_state: bool,
) {
    let accepted = result.is_ok();
    begin_response(method);
    emit_record_fields(
        vec![
            f("method", s(method)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("accepted", b(accepted)),
            f("rejected", b(!accepted)),
            f("reason", s(result.err().unwrap_or(accepted_reason))),
            f("program_sha256", record_sha_or_null(program_sha256)),
            f("install_attempted", b(false)),
            f("shell_start_attempted", b(false)),
            f("execution_attempted", b(false)),
            f("writes_persistent_state", b(writes_persistent_state)),
        ],
        6,
    );
    end_response(method);
}

fn emit_intake(outcome: program_workspace::IntakeOutcome, method: &'static str) {
    emit_snapshot(outcome.snapshot, method, Some(outcome));
}

fn emit_snapshot(
    snapshot: program_workspace::Snapshot,
    method: &'static str,
    outcome: Option<program_workspace::IntakeOutcome>,
) {
    let source = snapshot.source;
    let mut fields = vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("retention", s(snapshot.retention)),
        f(
            "status",
            s(if snapshot.present { "ready" } else { "empty" }),
        ),
    ];
    if let Some(outcome) = outcome {
        fields.extend([
            f("accepted", b(outcome.accepted)),
            f("rejected", b(outcome.rejected)),
            f("reason", s(outcome.reason)),
            f(
                "attempted_byte_len",
                V::U64(outcome.attempted_byte_len as u64),
            ),
        ]);
    }
    fields.extend([
        f("present", b(snapshot.present)),
        f("revision", V::U64(snapshot.revision)),
        f("byte_len", V::U64(snapshot.byte_len as u64)),
        f("program_sha256", record_sha_or_null(snapshot.sha256)),
        f(
            "source",
            source.map(|value| s(value.label())).unwrap_or(V::Null),
        ),
        f(
            "provider_request_id",
            source
                .and_then(|value| value.request_id())
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f(
            "serial_chunk_count",
            source
                .and_then(|value| value.chunk_count())
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f("pending_byte_len", V::U64(snapshot.pending_byte_len as u64)),
        f(
            "pending_chunk_count",
            V::U64(snapshot.pending_chunk_count as u64),
        ),
        f(
            "pending_provider_request_id",
            snapshot
                .pending_provider_request_id
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f(
            "original_request_present",
            b(snapshot.original_request_present),
        ),
        f(
            "original_request_byte_len",
            V::U64(snapshot.original_request_byte_len as u64),
        ),
        f(
            "provider_source_spec_present",
            b(snapshot.provider_source_spec_present),
        ),
        f(
            "provider_source_spec_byte_len",
            V::U64(snapshot.provider_source_spec_byte_len as u64),
        ),
        f(
            "provider_source_spec_sha256",
            record_sha_or_null(snapshot.provider_source_spec_sha256),
        ),
        f(
            "parent_program_sha256",
            record_sha_or_null(snapshot.parent_sha256),
        ),
        f(
            "root_program_sha256",
            record_sha_or_null(snapshot.root_sha256),
        ),
        f("lineage_depth", V::U64(snapshot.lineage_depth)),
        f(
            "last_rejection_reason",
            record_str_or_null(snapshot.last_rejection_reason),
        ),
        f(
            "last_rejection_attempted_byte_len",
            V::U64(snapshot.last_rejection_attempted_byte_len as u64),
        ),
        f("signing_attempted", b(false)),
        f("load_attempted", b(false)),
        f("execution_attempted", b(false)),
        f("authorizes_load", b(false)),
        f("authorizes_execution", b(false)),
        f("writes_persistent_state", b(false)),
    ]);
    emit_record_fields(fields, 6);
}
