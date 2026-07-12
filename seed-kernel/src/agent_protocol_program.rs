use alloc::vec;

use raios_core::record::Value as V;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    program_workspace,
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
        f("retention", s("current_boot_ram_only")),
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
        f("signing_attempted", b(false)),
        f("load_attempted", b(false)),
        f("execution_attempted", b(false)),
        f("authorizes_load", b(false)),
        f("authorizes_execution", b(false)),
        f("writes_persistent_state", b(false)),
    ]);
    emit_record_fields(fields, 6);
}
