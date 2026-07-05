use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_recovery_load_binding::{
        recovery_load_binding_retained_execution_completion_denial_mismatch,
        recovery_load_binding_retained_loader_mismatch,
        recovery_load_binding_retained_local_approval_mismatch,
        recovery_load_binding_retained_rollback_evidence_mismatch,
        recovery_load_binding_retained_trust_mismatch,
        recovery_load_binding_retained_vm_test_mismatch, RecoveryLoadBindingCheck,
        RecoveryLoadBindingSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_inline_record_object_fragment, emit_inline_record_property,
        emit_record_fields, extend_false_fields, raw_line, record_bool as b, record_event as ev,
        record_false as no, record_field as f, record_inline as inline, record_null as null,
        record_sha_fields as shas, record_sha_or_null, record_str as s,
    },
    event_log,
};
use raios_core::record::Field as F;

const LOAD_DENIED_FALSES: &[&str] = &[
    "authorizes_recovery_load",
    "can_move_beyond_denial",
    "loads_recovery_artifact",
    "loads_normal_module",
];
const LOADER_DENIED_FALSES: &[&str] = &[
    "authorizes_recovery_load",
    "can_move_beyond_denial",
    "loads_recovery_loader",
    "loads_recovery_artifact",
    "loads_normal_module",
];
const ROLLBACK_DENIED_FALSES: &[&str] = &[
    "authorizes_recovery_load",
    "can_move_beyond_denial",
    "loads_recovery_artifact",
    "loads_normal_module",
    "creates_durable_records",
    "installs_rollback_plan",
];
const EXEC_DENIED_FALSES: &[&str] = &[
    "authorizes_recovery_load",
    "can_move_beyond_denial",
    "accepts_lifeline_command_body",
    "dispatches_lifeline_command",
    "command_execution_enabled",
    "loads_recovery_artifact",
    "loads_normal_module",
    "creates_durable_records",
    "installs_rollback_plan",
];
const EXEC_MISSING_FALSES: &[&str] = &[
    "accepts_lifeline_command_body",
    "dispatches_lifeline_command",
    "command_execution_enabled",
];

#[derive(Clone, Copy)]
enum DeniedTail {
    Load,
    Loader,
    Rollback,
    Exec,
}

fn missing_binding_fields(schema: &'static str, reason: &'static str) -> Vec<F<'static>> {
    vec![
        f("schema", s(schema)),
        f("status", s("missing")),
        f("event_id", null()),
        f("retained", no()),
        f("required", b(true)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("reason", s(reason)),
        f("authorizes_recovery_load", no()),
        f("can_move_beyond_denial", no()),
        f("loads_recovery_artifact", no()),
    ]
}

fn retained_binding_fields<'a>(
    schema: &'a str,
    status: &'a str,
    event_id: event_log::EventId,
    reason: &'a str,
) -> Vec<F<'a>> {
    vec![
        f("schema", s(schema)),
        f("status", s(status)),
        f("event_id", ev(event_id)),
        f("retained", b(true)),
        f("required", b(true)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("reason", s(reason)),
    ]
}

fn add_inventory_result<'a>(fields: &mut Vec<F<'a>>) {
    fields.push(f("service_inventory_change", s("none")));
    fields.push(f("load_attempted", no()));
}

fn add_denied_tail<'a>(fields: &mut Vec<F<'a>>, tail: DeniedTail) {
    match tail {
        DeniedTail::Load => extend_false_fields(fields, LOAD_DENIED_FALSES),
        DeniedTail::Loader => extend_false_fields(fields, LOADER_DENIED_FALSES),
        DeniedTail::Rollback => extend_false_fields(fields, ROLLBACK_DENIED_FALSES),
        DeniedTail::Exec => extend_false_fields(fields, EXEC_DENIED_FALSES),
    }
    add_inventory_result(fields);
}

fn emit_missing_binding(
    property: &'static str,
    schema: &'static str,
    reason: &'static str,
    extra_false_fields: &'static [&'static str],
    comma: bool,
) {
    let mut fields = missing_binding_fields(schema, reason);
    extend_false_fields(&mut fields, extra_false_fields);
    emit_inline_record_property(property, fields, comma);
}

fn emit_retained_binding<'a>(
    property: &'static str,
    schema: &'a str,
    status: &'a str,
    event_id: event_log::EventId,
    reason: &'a str,
    tail: DeniedTail,
    mut extra: Vec<F<'a>>,
    hashes: &[(&'a str, [u8; 32])],
    comma: bool,
) {
    let mut fields = retained_binding_fields(schema, status, event_id, reason);
    add_denied_tail(&mut fields, tail);
    fields.append(&mut extra);
    if !hashes.is_empty() {
        fields.push(f("hashes", inline(shas(hashes))));
    }
    emit_inline_record_property(property, fields, comma);
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_identity_binding_fact(retained: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        emit_retained_binding("recovery_artifact_identity_event_id", "raios.recovery_artifact_identity.v0", "retained_hash_reference_only", event_id, "retained_recovery_artifact_identity_reference_not_authorizing", DeniedTail::Load, vec![], &[("identity_reference_hash", reference.identity_reference_hash), ("artifact_hash", reference.artifact_hash)], comma);
    } else {
        emit_missing_binding("recovery_artifact_identity_event_id", "raios.recovery_artifact_identity.v0", "recovery_artifact_identity_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_trust_binding_fact(retained_identity: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, retained: Option<(event_log::EventId, event_log::RecoveryArtifactTrustReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_trust_mismatch(retained_identity, retained);
        emit_retained_binding("recovery_artifact_trust_event_id", "raios.recovery_artifact_trust.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_only" }, event_id, mismatch.unwrap_or("retained_recovery_artifact_trust_reference_not_authorizing"), DeniedTail::Load, vec![f("retained_recovery_artifact_identity_event_id", ev(reference.retained_identity_reference_event_id))], &[("trust_reference_hash", reference.trust_reference_hash), ("identity_reference_hash", reference.identity_reference_hash), ("artifact_hash", reference.artifact_hash), ("trust_hash", reference.trust_hash)], comma);
    } else {
        emit_missing_binding("recovery_artifact_trust_event_id", "raios.recovery_artifact_trust.v0", "recovery_artifact_trust_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_vm_test_binding_fact(retained_identity: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, retained_trust: Option<(event_log::EventId, event_log::RecoveryArtifactTrustReference)>, retained: Option<(event_log::EventId, event_log::RecoveryArtifactVmTestReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_vm_test_mismatch(retained_identity, retained_trust, retained);
        emit_retained_binding("recovery_vm_test_event_id", "raios.recovery_artifact_vm_test.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_only" }, event_id, mismatch.unwrap_or("retained_recovery_artifact_vm_test_reference_not_authorizing"), DeniedTail::Load, vec![f("retained_recovery_artifact_identity_event_id", ev(reference.retained_identity_reference_event_id)), f("retained_recovery_artifact_trust_event_id", ev(reference.retained_trust_reference_event_id))], &[("vm_test_reference_hash", reference.vm_test_reference_hash), ("identity_reference_hash", reference.identity_reference_hash), ("trust_reference_hash", reference.trust_reference_hash), ("artifact_hash", reference.artifact_hash), ("trust_hash", reference.trust_hash), ("vm_test_hash", reference.vm_test_hash)], comma);
    } else {
        emit_missing_binding("recovery_vm_test_event_id", "raios.recovery_artifact_vm_test.v0", "recovery_vm_test_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_local_approval_binding_fact(retained_identity: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, retained_trust: Option<(event_log::EventId, event_log::RecoveryArtifactTrustReference)>, retained_vm_test: Option<(event_log::EventId, event_log::RecoveryArtifactVmTestReference)>, retained: Option<(event_log::EventId, event_log::RecoveryArtifactLocalApprovalReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_local_approval_mismatch(retained_identity, retained_trust, retained_vm_test, retained);
        emit_retained_binding("recovery_local_approval_event_id", "raios.recovery_artifact_local_approval.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_only" }, event_id, mismatch.unwrap_or("retained_recovery_artifact_local_approval_reference_not_authorizing"), DeniedTail::Load, vec![f("retained_recovery_artifact_identity_event_id", ev(reference.retained_identity_reference_event_id)), f("retained_recovery_artifact_trust_event_id", ev(reference.retained_trust_reference_event_id)), f("retained_recovery_artifact_vm_test_event_id", ev(reference.retained_vm_test_reference_event_id))], &[("local_approval_reference_hash", reference.local_approval_reference_hash), ("identity_reference_hash", reference.identity_reference_hash), ("trust_reference_hash", reference.trust_reference_hash), ("vm_test_reference_hash", reference.vm_test_reference_hash), ("artifact_hash", reference.artifact_hash), ("trust_hash", reference.trust_hash), ("vm_test_hash", reference.vm_test_hash), ("local_approval_hash", reference.local_approval_hash)], comma);
    } else {
        emit_missing_binding("recovery_local_approval_event_id", "raios.recovery_artifact_local_approval.v0", "recovery_local_approval_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_loader_binding_fact(retained_identity: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, retained_trust: Option<(event_log::EventId, event_log::RecoveryArtifactTrustReference)>, retained_vm_test: Option<(event_log::EventId, event_log::RecoveryArtifactVmTestReference)>, retained_local_approval: Option<(event_log::EventId, event_log::RecoveryArtifactLocalApprovalReference)>, retained: Option<(event_log::EventId, event_log::RecoveryArtifactLoaderReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_loader_mismatch(retained_identity, retained_trust, retained_vm_test, retained_local_approval, retained);
        emit_retained_binding("recovery_loader_event_id", "raios.recovery_artifact_loader.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_only" }, event_id, mismatch.unwrap_or("retained_recovery_artifact_loader_reference_not_authorizing"), DeniedTail::Loader, vec![f("retained_recovery_artifact_identity_event_id", ev(reference.retained_identity_reference_event_id)), f("retained_recovery_artifact_trust_event_id", ev(reference.retained_trust_reference_event_id)), f("retained_recovery_artifact_vm_test_event_id", ev(reference.retained_vm_test_reference_event_id)), f("retained_recovery_artifact_local_approval_event_id", ev(reference.retained_local_approval_reference_event_id))], &[("loader_reference_hash", reference.loader_reference_hash), ("identity_reference_hash", reference.identity_reference_hash), ("trust_reference_hash", reference.trust_reference_hash), ("vm_test_reference_hash", reference.vm_test_reference_hash), ("local_approval_reference_hash", reference.local_approval_reference_hash), ("artifact_hash", reference.artifact_hash), ("trust_hash", reference.trust_hash), ("vm_test_hash", reference.vm_test_hash), ("local_approval_hash", reference.local_approval_hash), ("loader_hash", reference.loader_hash)], comma);
    } else {
        emit_missing_binding("recovery_loader_event_id", "raios.recovery_artifact_loader.v0", "recovery_loader_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_rollback_evidence_binding_fact(retained_identity: Option<(event_log::EventId, event_log::RecoveryArtifactIdentityReference)>, retained_trust: Option<(event_log::EventId, event_log::RecoveryArtifactTrustReference)>, retained_vm_test: Option<(event_log::EventId, event_log::RecoveryArtifactVmTestReference)>, retained_local_approval: Option<(event_log::EventId, event_log::RecoveryArtifactLocalApprovalReference)>, retained_loader: Option<(event_log::EventId, event_log::RecoveryArtifactLoaderReference)>, retained: Option<(event_log::EventId, event_log::RecoveryArtifactRollbackEvidenceReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_rollback_evidence_mismatch(retained_identity, retained_trust, retained_vm_test, retained_local_approval, retained_loader, retained);
        emit_retained_binding("recovery_rollback_evidence_event_id", "raios.recovery_artifact_rollback_evidence.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_only" }, event_id, mismatch.unwrap_or("retained_recovery_artifact_rollback_evidence_reference_not_authorizing"), DeniedTail::Rollback, vec![f("retained_recovery_artifact_identity_event_id", ev(reference.retained_identity_reference_event_id)), f("retained_recovery_artifact_trust_event_id", ev(reference.retained_trust_reference_event_id)), f("retained_recovery_artifact_vm_test_event_id", ev(reference.retained_vm_test_reference_event_id)), f("retained_recovery_artifact_local_approval_event_id", ev(reference.retained_local_approval_reference_event_id)), f("retained_recovery_artifact_loader_event_id", ev(reference.retained_loader_reference_event_id))], &[("rollback_evidence_reference_hash", reference.rollback_evidence_reference_hash), ("identity_reference_hash", reference.identity_reference_hash), ("trust_reference_hash", reference.trust_reference_hash), ("vm_test_reference_hash", reference.vm_test_reference_hash), ("local_approval_reference_hash", reference.local_approval_reference_hash), ("loader_reference_hash", reference.loader_reference_hash), ("artifact_hash", reference.artifact_hash), ("trust_hash", reference.trust_hash), ("vm_test_hash", reference.vm_test_hash), ("local_approval_hash", reference.local_approval_hash), ("loader_hash", reference.loader_hash), ("rollback_evidence_hash", reference.rollback_evidence_hash)], comma);
    } else {
        emit_missing_binding("recovery_rollback_evidence_event_id", "raios.recovery_artifact_rollback_evidence.v0", "recovery_rollback_evidence_event_id_missing", &[], comma);
    }
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_execution_completion_denial_binding_fact(retained: Option<(event_log::EventId, event_log::RecoveryLifelineCommandExecutionStageReference)>, comma: bool) {
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_execution_completion_denial_mismatch(retained);
        let mut hash_fields = shas(&[("execution_stage_hash", reference.execution_stage_hash), ("execution_completion_denial_hash", reference.execution_stage_hash), ("side_effect_gate_hash", reference.side_effect_gate_hash), ("source_rollback_apply_denial_hash", reference.source_rollback_apply_denial_hash), ("source_durable_policy_write_authority_decision_hash", reference.source_durable_policy_write_authority_decision_hash), ("source_recovery_rollback_inspect_source_reference_hash", reference.source_recovery_rollback_inspect_source_reference_hash)]);
        hash_fields.extend([f("execution_enablement_hash", record_sha_or_null(reference.execution_enablement_hash)), f("execution_preflight_hash", record_sha_or_null(reference.execution_preflight_hash)), f("execution_intent_hash", record_sha_or_null(reference.execution_intent_hash)), f("execution_commit_gate_hash", record_sha_or_null(reference.execution_commit_gate_hash)), f("execution_result_denial_hash", record_sha_or_null(reference.execution_result_denial_hash)), f("execution_audit_denial_hash", record_sha_or_null(reference.execution_audit_denial_hash)), f("execution_observation_denial_hash", record_sha_or_null(reference.execution_observation_denial_hash))]);
        emit_retained_binding("recovery_lifeline_command_execution_completion_denial_event_id", "raios.recovery_lifeline_command_execution_completion_denial.v0", if mismatch.is_some() { "rejected_retained_reference" } else { "retained_hash_reference_command_still_denied" }, event_id, mismatch.unwrap_or("retained_recovery_lifeline_command_execution_completion_denial_reference_not_authorizing"), DeniedTail::Exec, vec![f("command_dispatch_boundary_id", s(reference.command_dispatch_boundary_id)), f("hashes", inline(hash_fields))], &[], comma);
    } else {
        emit_missing_binding("recovery_lifeline_command_execution_completion_denial_event_id", "raios.recovery_lifeline_command_execution_completion_denial.v0", "recovery_lifeline_command_execution_completion_denial_event_id_missing", EXEC_MISSING_FALSES, comma);
    }
}

pub(crate) fn emit_recovery_load_blocker(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    emit_inline_record_object_fragment(
        vec![
            f("gate", s(gate)),
            f("state", s(state)),
            f("reason", s(reason)),
        ],
        8,
    );
    *wrote = true;
}

pub(crate) fn emit_recovery_load_binding_check(
    check: &RecoveryLoadBindingCheck,
    _spaces: usize,
    _include_status: bool,
) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f(
                "recovery_only_capability_used",
                b(check.recovery_only_capability_used),
            ),
            f(
                "accepts_normal_module_authority",
                b(check.accepts_normal_module_authority),
            ),
            f(
                "append_payload_hash_authority",
                b(check.append_payload_hash_authority),
            ),
            f("can_move_beyond_denial", b(check.can_move_beyond_denial)),
            f("loads_recovery_artifact", b(check.loads_recovery_artifact)),
            f("loads_normal_module", b(check.loads_normal_module)),
            f("creates_durable_records", b(check.creates_durable_records)),
            f("installs_rollback_plan", b(check.installs_rollback_plan)),
            f(
                "service_inventory_change",
                s(check.service_inventory_change),
            ),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_load_binding_selftest_case(
    case: &RecoveryLoadBindingSelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("passed", b(case.passed)),
            f("can_move_beyond_denial", no()),
            f("loads_recovery_artifact", no()),
            f("loads_normal_module", no()),
            f("load_attempted", no()),
            f("normal_module_capability_accepted", no()),
            f("append_payload_hash_authority", no()),
        ],
        comma,
    );
}
