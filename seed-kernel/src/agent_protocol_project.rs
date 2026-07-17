use alloc::{vec, vec::Vec};

use raios_core::{
    project_workspace::{ProjectRevision, TreeEntry},
    record::Value as V,
};

use crate::{
    agent_build_loop,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s, record_str_or_null,
    },
    project_build, project_workspace,
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

pub(crate) fn emit_agent_answer_fixture() {
    let outcome = agent_build_loop::accept_test_fixture();
    begin_response("project.agent_answer_fixture");
    emit_record_fields(
        agent_workspace_fields(
            "project.agent_answer_fixture",
            &outcome.snapshot,
            Some(&outcome),
        ),
        6,
    );
    end_response("project.agent_answer_fixture");
}

pub(crate) fn emit_verify_revision() {
    let snapshot = agent_build_loop::snapshot();
    let result = snapshot
        .latest_revision
        .as_ref()
        .ok_or("source_preflight_revision_not_tracked")
        .and_then(|revision| {
            agent_build_loop::record_verifier_result(
                revision,
                project_build::verify_source_revision(revision),
            )
        });
    begin_response("project.verify_revision");
    let mut fields = source_preflight_posture("project.verify_revision", &snapshot);
    match result {
        Ok(result) => fields.extend([
            f("status", s("recorded")),
            f("accepted", b(true)),
            f("reason", s(result.reason)),
            f("check_id", s(result.check_id)),
            f("outcome", s(result.outcome())),
            f("revision_sha256", V::Sha256(result.revision_sha256)),
            f("tree_sha256", V::Sha256(result.tree_sha256)),
            f("system_computed", b(true)),
            f("provider_supplied", b(false)),
        ]),
        Err(reason) => fields.extend([
            f("status", s("denied")),
            f("accepted", b(false)),
            f("reason", s(reason)),
            f("check_id", V::Null),
            f("outcome", V::Null),
            f("revision_sha256", V::Null),
            f("tree_sha256", V::Null),
            f("system_computed", b(false)),
            f("provider_supplied", b(false)),
        ]),
    }
    fields.extend(source_preflight_non_authority_fields());
    emit_record_fields(fields, 6);
    end_response("project.verify_revision");
}

pub(crate) fn emit_feedback_packet() {
    let result = agent_build_loop::build_feedback_packet();
    let snapshot = agent_build_loop::snapshot();
    begin_response("project.feedback_packet");
    let mut fields = source_preflight_posture("project.feedback_packet", &snapshot);
    match result {
        Ok(packet) => fields.extend([
            f("status", s("ready")),
            f("accepted", b(true)),
            f("reason", s(packet.reason)),
            f("feedback_packet", feedback_packet_value(packet)),
            f("packet_field_count", V::U64(4)),
        ]),
        Err(reason) => fields.extend([
            f("status", s("denied")),
            f("accepted", b(false)),
            f("reason", s(reason)),
            f("feedback_packet", V::Null),
            f("packet_field_count", V::U64(0)),
        ]),
    }
    fields.extend([
        f("system_computed", b(result.is_ok())),
        f("provider_supplied", b(false)),
        f("source_bytes_included", b(false)),
        f("secret_bytes_included", b(false)),
        f("log_bytes_included", b(false)),
        f("unclassified_text_included", b(false)),
    ]);
    fields.extend(source_preflight_non_authority_fields());
    emit_record_fields(fields, 6);
    end_response("project.feedback_packet");
}

pub(crate) fn emit_revision_answer_fixture() {
    let outcome = agent_build_loop::accept_revision_answer_fixture();
    begin_response("project.revision_answer_fixture");
    emit_record_fields(
        agent_workspace_fields(
            "project.revision_answer_fixture",
            &outcome.snapshot,
            Some(&outcome),
        ),
        6,
    );
    end_response("project.revision_answer_fixture");
}

pub(crate) fn emit_workspace() {
    let snapshot = agent_build_loop::snapshot();
    begin_response("project.workspace");
    emit_record_fields(
        agent_workspace_fields("project.workspace", &snapshot, None),
        6,
    );
    end_response("project.workspace");
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
            f("revision_action", V::Null),
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
            f("revision_action", V::Null),
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
            "revision_action",
            outcome
                .revision
                .as_ref()
                .map(|revision| s(revision.action.label()))
                .unwrap_or(V::Null),
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

fn agent_workspace_fields<'a>(
    method: &'static str,
    snapshot: &'a agent_build_loop::Snapshot,
    outcome: Option<&agent_build_loop::AnswerOutcome>,
) -> Vec<raios_core::record::Field<'a>> {
    let revision = snapshot.latest_revision.as_ref();
    let total_byte_len = revision
        .map(|revision| revision.entries.iter().map(|entry| entry.byte_len).sum())
        .unwrap_or(0usize);
    let answer_sha256 = outcome
        .map(|outcome| Some(outcome.answer_sha256))
        .unwrap_or(snapshot.answer_sha256);
    let answer_byte_len = outcome
        .map(|outcome| Some(outcome.answer_byte_len))
        .unwrap_or(snapshot.answer_byte_len);
    let answer_origin = outcome
        .map(|outcome| Some(outcome.answer_origin))
        .unwrap_or(snapshot.answer_origin);
    let provider_trust_positive = outcome
        .map(|outcome| outcome.provider_trust_positive)
        .unwrap_or(snapshot.provider_trust_positive);
    let test_infrastructure = outcome
        .map(|outcome| outcome.test_infrastructure)
        .unwrap_or(snapshot.test_infrastructure);
    let parent_revision_readback_verified = outcome
        .filter(|outcome| outcome.accepted)
        .map(|outcome| outcome.parent_revision_readback_verified)
        .unwrap_or(snapshot.parent_revision_readback_verified);
    let verifier_result = snapshot.verifier_result.as_ref();
    let mut fields = vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("phase", s(snapshot.phase.label())),
        f(
            "pending_request_id",
            snapshot
                .pending_request_id
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f(
            "latest_request_id",
            snapshot
                .latest_request_id
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f(
            "project_id",
            snapshot
                .project_id
                .as_ref()
                .map(|value| V::HexBytes(value))
                .unwrap_or(V::Null),
        ),
        f(
            "request_sha256",
            record_sha_or_null(snapshot.request_sha256),
        ),
        f("answer_sha256", record_sha_or_null(answer_sha256)),
        f(
            "answer_byte_len",
            answer_byte_len
                .map(|value| V::U64(value as u64))
                .unwrap_or(V::Null),
        ),
        f("latest_revision_present", b(revision.is_some())),
        f(
            "parent_revision_sha256",
            record_sha_or_null(revision.and_then(|revision| revision.parent_revision_sha256)),
        ),
        f(
            "revision_action",
            revision
                .map(|revision| s(revision.action.label()))
                .unwrap_or(V::Null),
        ),
        f(
            "tree_sha256",
            record_sha_or_null(revision.map(|revision| revision.tree_sha256)),
        ),
        f(
            "revision_sha256",
            record_sha_or_null(revision.map(|revision| revision.revision_sha256)),
        ),
        f(
            "file_count",
            V::U64(revision.map_or(0, |revision| revision.entries.len()) as u64),
        ),
        f("total_byte_len", V::U64(total_byte_len as u64)),
        f(
            "files",
            V::Array(
                revision
                    .map(|revision| revision.entries.iter().map(file_value).collect())
                    .unwrap_or_default(),
            ),
        ),
        f("revision_lineage", revision_lineage_value(snapshot)),
        f(
            "parent_revision_readback_verified",
            b(parent_revision_readback_verified),
        ),
        f("answer_origin", record_str_or_null(answer_origin)),
        f(
            "source_authority",
            revision
                .map(|_| s("untrusted_agent_candidate"))
                .unwrap_or(V::Null),
        ),
        f("provider_trust_positive", b(provider_trust_positive)),
        f("test_infrastructure", b(test_infrastructure)),
        f(
            "verifier_check_id",
            record_str_or_null(verifier_result.map(|result| result.check_id)),
        ),
        f(
            "verifier_revision_sha256",
            record_sha_or_null(verifier_result.map(|result| result.revision_sha256)),
        ),
        f(
            "verifier_tree_sha256",
            record_sha_or_null(verifier_result.map(|result| result.tree_sha256)),
        ),
        f(
            "verifier_reason",
            record_str_or_null(verifier_result.map(|result| result.reason)),
        ),
        f(
            "verifier_outcome",
            record_str_or_null(verifier_result.map(|result| result.outcome())),
        ),
        f("verifier_system_computed", b(verifier_result.is_some())),
        f("verifier_provider_supplied", b(false)),
        f(
            "feedback_packet",
            snapshot
                .feedback_packet
                .map(feedback_packet_value)
                .unwrap_or(V::Null),
        ),
        f("last_reason", record_str_or_null(snapshot.last_reason)),
        f("job_state_retention", s("current_boot_ram_only")),
        f(
            "committed_files_retention",
            s("qemu_disposable_structured_store"),
        ),
        f(
            "persistence_posture",
            s("qemu_disposable_structured_store_only"),
        ),
        f("qemu_only", b(true)),
        f("physical_media_supported", b(false)),
        f("physical_media_attempted", b(false)),
    ];
    if let Some(outcome) = outcome {
        fields.extend([
            f(
                "status",
                s(if outcome.accepted {
                    "accepted"
                } else {
                    "denied"
                }),
            ),
            f("accepted", b(outcome.accepted)),
            f("rejected", b(!outcome.accepted)),
            f("reason", s(outcome.reason)),
            f(
                "storage_write_attempted",
                b(outcome.storage_write_attempted),
            ),
            f(
                "writes_persistent_state",
                b(outcome.writes_persistent_state),
            ),
        ]);
    } else {
        fields.extend([
            f("status", s(snapshot.phase.label())),
            f("reason", record_str_or_null(snapshot.last_reason)),
            f("storage_write_attempted", b(false)),
            f("writes_persistent_state", b(false)),
        ]);
    }
    fields.extend([
        f("signing_attempted", b(false)),
        f("build_session_opened", b(false)),
        f("builder_attempted", b(false)),
        f("compiler_attempted", b(false)),
        f("test_attempted", b(false)),
        f("run_attempted", b(false)),
        f("build_authorized", b(false)),
        f("candidate_intake_attempted", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("promotion_attempted", b(false)),
        f("promotion_authorized", b(false)),
        f("wasm_instance_created", b(false)),
        f("w6_preview_created", b(false)),
        f("reclog_executable_record_written", b(false)),
        f("artstor_executable_record_written", b(false)),
        f("service_inventory_mutation", s("none")),
    ]);
    fields
}

fn source_preflight_posture<'a>(
    method: &'static str,
    snapshot: &'a agent_build_loop::Snapshot,
) -> Vec<raios_core::record::Field<'a>> {
    vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("check_kind", s("deterministic_local_source_preflight")),
        f("answer_origin", record_str_or_null(snapshot.answer_origin)),
        f(
            "provider_trust_positive",
            b(snapshot.provider_trust_positive),
        ),
        f("test_infrastructure", b(snapshot.test_infrastructure)),
    ]
}

fn source_preflight_non_authority_fields() -> Vec<raios_core::record::Field<'static>> {
    vec![
        f("storage_write_attempted", b(false)),
        f("writes_persistent_state", b(false)),
        f("signing_attempted", b(false)),
        f("build_session_opened", b(false)),
        f("builder_attempted", b(false)),
        f("compiler_attempted", b(false)),
        f("test_attempted", b(false)),
        f("run_attempted", b(false)),
        f("build_authorized", b(false)),
        f("candidate_intake_attempted", b(false)),
        f("provider_export_attempted", b(false)),
        f("provider_export_authorized", b(false)),
        f("load_attempted", b(false)),
        f("load_authorized", b(false)),
        f("execution_attempted", b(false)),
        f("execution_authorized", b(false)),
        f("install_attempted", b(false)),
        f("install_authorized", b(false)),
        f("promotion_attempted", b(false)),
        f("promotion_authorized", b(false)),
        f("wasm_instance_created", b(false)),
        f("w6_preview_created", b(false)),
        f("reclog_executable_record_written", b(false)),
        f("artstor_executable_record_written", b(false)),
        f("service_inventory_mutation", s("none")),
    ]
}

fn feedback_packet_value(packet: agent_build_loop::FeedbackPacket) -> V<'static> {
    V::InlineObject(vec![
        f("check_id", s(packet.check_id)),
        f("revision_sha256", V::Sha256(packet.revision_sha256)),
        f("tree_sha256", V::Sha256(packet.tree_sha256)),
        f("reason", s(packet.reason)),
    ])
}

fn revision_lineage_value(snapshot: &agent_build_loop::Snapshot) -> V<'_> {
    let mut lineage = Vec::new();
    if let Some(revision) = snapshot.parent_revision.as_ref() {
        lineage.push(lineage_revision_value(revision));
    }
    if let Some(revision) = snapshot.latest_revision.as_ref() {
        lineage.push(lineage_revision_value(revision));
    }
    V::Array(lineage)
}

fn lineage_revision_value(revision: &ProjectRevision) -> V<'_> {
    V::InlineObject(vec![
        f("revision_sha256", V::Sha256(revision.revision_sha256)),
        f("tree_sha256", V::Sha256(revision.tree_sha256)),
        f(
            "parent_revision_sha256",
            record_sha_or_null(revision.parent_revision_sha256),
        ),
    ])
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
        f("revision_action", s(revision.action.label())),
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
