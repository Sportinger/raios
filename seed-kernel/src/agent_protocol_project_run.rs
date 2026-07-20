use alloc::{vec, vec::Vec};

use raios_core::record::{Field, Value as V};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, parse_sha256_ref, record_bool as b,
        record_field as f, record_sha_or_null, record_str as s,
    },
    workspace_candidate_service::{self, Outcome, Snapshot},
};

pub(crate) fn emit_prepare(input: &str) {
    let outcome = match prepare_args(input) {
        Ok((project, revision, receipt, candidate)) => {
            workspace_candidate_service::prepare(project, revision, receipt, candidate)
        }
        Err(reason) => malformed(reason),
    };
    emit("project.run_prepare", outcome);
}

pub(crate) fn emit_status() {
    emit_snapshot(
        "project.run_status",
        true,
        "workspace_status_read",
        workspace_candidate_service::snapshot(),
    );
}

pub(crate) fn emit_cancel() {
    emit("project.run_cancel", workspace_candidate_service::cancel());
}

pub(crate) fn emit_serial_approval_denied(_input: &str) {
    emit_snapshot(
        "project.run_approve",
        false,
        "workspace_physical_pointer_approval_required",
        workspace_candidate_service::snapshot(),
    );
}

pub(crate) fn emit_health(_input: &str) -> &'static str {
    emit_snapshot(
        "service.health",
        true,
        "workspace_health_read",
        workspace_candidate_service::snapshot(),
    );
    "accepted"
}

pub(crate) fn emit_start(_input: &str) -> &'static str {
    let outcome = workspace_candidate_service::start();
    let response = if outcome.accepted {
        "accepted"
    } else {
        "denied"
    };
    emit("service.start", outcome);
    response
}

pub(crate) fn emit_stop(_input: &str) -> &'static str {
    let outcome = workspace_candidate_service::stop();
    let response = if outcome.accepted {
        "accepted"
    } else {
        "denied"
    };
    emit("service.stop", outcome);
    response
}

pub(crate) fn emit_drop(_input: &str) -> &'static str {
    let outcome = workspace_candidate_service::drop_service("workspace_service_dropped");
    let response = if outcome.accepted {
        "accepted"
    } else {
        "denied"
    };
    emit("service.drop", outcome);
    response
}

fn emit(method: &'static str, outcome: Outcome) {
    emit_snapshot(method, outcome.accepted, outcome.reason, outcome.snapshot)
}

fn emit_snapshot(method: &'static str, accepted: bool, reason: &'static str, snapshot: Snapshot) {
    begin_response(method);
    emit_record_fields(fields(method, accepted, reason, &snapshot), 6);
    end_response(method);
}

fn fields<'a>(
    method: &'static str,
    accepted: bool,
    reason: &'static str,
    snapshot: &'a Snapshot,
) -> Vec<Field<'a>> {
    let binding = snapshot.binding.as_ref();
    let approval = snapshot.approval.as_ref();
    let mut fields = vec![
        f("method", s(method)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("persistence_posture", s("ram_only")),
        f("status", s(if accepted { "accepted" } else { "denied" })),
        f("reason", s(reason)),
        f("accepted", b(accepted)),
        f("rejected", b(!accepted)),
        f("service_id", s(workspace_candidate_service::service_id())),
        f("phase", s(snapshot.phase)),
        f(
            "health",
            s(workspace_candidate_service::health_state(*snapshot)),
        ),
        f(
            "project_id",
            binding
                .map(|item| V::HexBytes(&item.project_id))
                .unwrap_or(V::Null),
        ),
        f(
            "project_revision_sha256",
            record_sha_or_null(binding.map(|item| item.project_revision_sha256)),
        ),
        f(
            "source_tree_sha256",
            record_sha_or_null(binding.map(|item| item.source_tree_sha256)),
        ),
        f(
            "cargo_lock_sha256",
            record_sha_or_null(binding.map(|item| item.cargo_lock_sha256)),
        ),
        f(
            "input_manifest_sha256",
            record_sha_or_null(binding.map(|item| item.input_manifest_sha256)),
        ),
        f(
            "receipt_sha256",
            record_sha_or_null(binding.map(|item| item.receipt_sha256)),
        ),
        f(
            "candidate_sha256",
            record_sha_or_null(binding.map(|item| item.candidate_sha256)),
        ),
        f(
            "candidate_byte_len",
            V::U64(binding.map(|item| item.candidate_byte_len).unwrap_or(0) as u64),
        ),
        f("import_list_observed", b(binding.is_some())),
        f(
            "import_count",
            V::U64(binding.map(|item| item.import_count).unwrap_or(0) as u64),
        ),
        f(
            "import_list_sha256",
            record_sha_or_null(binding.map(|item| item.import_list_sha256)),
        ),
        f(
            "entrypoint",
            s(raios_core::project_runtime::WORKSPACE_ENTRYPOINT),
        ),
        f(
            "fuel_budget",
            V::U64(binding.map(|item| item.fuel_budget).unwrap_or(0)),
        ),
        f(
            "memory_limit_bytes",
            V::U64(binding.map(|item| item.memory_limit_bytes).unwrap_or(0) as u64),
        ),
        f(
            "instance_limit",
            V::U64(binding.map(|item| item.instance_limit).unwrap_or(0) as u64),
        ),
        f(
            "memory_count_limit",
            V::U64(binding.map(|item| item.memory_count_limit).unwrap_or(0) as u64),
        ),
        f(
            "table_limit",
            V::U64(binding.map(|item| item.table_limit).unwrap_or(0) as u64),
        ),
        f(
            "approval_challenge_sha256",
            record_sha_or_null(binding.map(|item| item.approval_challenge_sha256)),
        ),
        f("physical_approval_present", b(approval.is_some())),
        f(
            "approval_source",
            approval.map(|item| s(item.source)).unwrap_or(V::Null),
        ),
        f(
            "approval_sha256",
            record_sha_or_null(approval.map(|item| item.approval_sha256)),
        ),
        f("generation", V::U64(snapshot.generation)),
        f("run_count", V::U64(snapshot.run_count)),
        f("run_outcome", s(snapshot.last_run_outcome)),
        f(
            "return_value_i32",
            snapshot
                .last_return_value
                .and_then(|value| (value >= 0).then_some(V::U64(value as u64)))
                .unwrap_or(V::Null),
        ),
        f(
            "return_value_i32_bits",
            snapshot
                .last_return_value
                .map(|value| V::U64(value as u32 as u64))
                .unwrap_or(V::Null),
        ),
        f("fuel_used", V::U64(snapshot.last_fuel_used)),
        f("trust_tier", s(workspace_candidate_service::trust_tier())),
        f("serial_approval_authorized", b(false)),
    ];
    for name in [
        "writes_persistent_state",
        "storage_write",
        "authorizes_promotion",
        "authorizes_install",
        "authorizes_native_load",
        "authorizes_persistence",
    ] {
        fields.push(f(name, b(false)));
    }
    fields
}

fn malformed(reason: &'static str) -> Outcome {
    Outcome {
        accepted: false,
        reason,
        snapshot: workspace_candidate_service::snapshot(),
    }
}

fn prepare_args(input: &str) -> Result<([u8; 16], [u8; 32], [u8; 32], [u8; 32]), &'static str> {
    let mut parts = input.split_ascii_whitespace();
    if parts.next() != Some("project.run_prepare") {
        return Err("workspace_prepare_method_invalid");
    }
    let project = parse_hex_16(parts.next().ok_or("workspace_project_id_missing")?)
        .ok_or("workspace_project_id_invalid")?;
    let revision = parse_sha256_ref(parts.next().ok_or("workspace_revision_missing")?)
        .ok_or("workspace_revision_invalid")?;
    let receipt = parse_sha256_ref(parts.next().ok_or("workspace_receipt_missing")?)
        .ok_or("workspace_receipt_invalid")?;
    let candidate = parse_sha256_ref(parts.next().ok_or("workspace_candidate_missing")?)
        .ok_or("workspace_candidate_invalid")?;
    if parts.next().is_some() {
        return Err("workspace_prepare_argument_count_invalid");
    }
    Ok((project, revision, receipt, candidate))
}

fn parse_hex_16(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut out = [0u8; 16];
    let mut idx = 0usize;
    while idx < out.len() {
        out[idx] = (hex(bytes[idx * 2])? << 4) | hex(bytes[idx * 2 + 1])?;
        idx += 1;
    }
    Some(out)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
