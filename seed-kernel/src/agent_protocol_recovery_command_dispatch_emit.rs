use alloc::vec;

use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandDispatchCheck, RecoveryLifelineCommandDispatchSelfTestCase,
    },
    agent_protocol_recovery_command_eval::{
        recovery_lifeline_status_execution_readiness_ready,
        recovery_lifeline_status_execution_readiness_reason,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_fields, emit_record_property,
        emit_selftest_case_fields_split, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_inline as inline, record_null as null,
        record_object as object, record_sha as sha, record_sha_or_null, record_str as s,
        record_str_or_null,
        SelftestReportField::{Bool, False, Str},
    },
    event_log,
};
use raios_core::record::Value as V;

pub(crate) fn emit_recovery_lifeline_command_dispatch_retained_envelope(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandEnvelopeReference,
    )>,
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    let matches_latest_lifeline_request =
        if let (Some((_, reference)), Some((request_event_id, request))) =
            (retained_ref, retained_request.as_ref())
        {
            reference.retained_lifeline_request_event_id == *request_event_id
                && reference.lifeline_request_reference_hash
                    == request.lifeline_request_reference_hash
        } else {
            false
        };

    emit_record_property(
        "retained_command_envelope_reference",
        vec![
            f(
                "status",
                s(if retained_ref.is_some() {
                    "retained_hash_reference_command_still_denied"
                } else {
                    "missing"
                }),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("accepts_lifeline_command_body", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f(
                "event_id",
                record_event_or_null(retained_ref.map(|(id, _)| *id)),
            ),
            f(
                "command_id",
                record_str_or_null(retained_ref.map(|(_, reference)| reference.command_id)),
            ),
            f(
                "argument_schema",
                record_str_or_null(retained_ref.map(|(_, reference)| reference.argument_schema)),
            ),
            f(
                "required_capability",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.required_capability),
                ),
            ),
            f(
                "target_locator",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.target_locator.as_str()),
                ),
            ),
            f(
                "command_admission_boundary_id",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.command_admission_boundary_id),
                ),
            ),
            f(
                "retained_recovery_lifeline_request_event_id",
                retained_ref
                    .map(|(_, reference)| {
                        V::EventSequence(reference.retained_lifeline_request_event_id.sequence())
                    })
                    .unwrap_or(null()),
            ),
            f(
                "matches_latest_lifeline_request",
                b(matches_latest_lifeline_request),
            ),
            f(
                "command_envelope_reference_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.command_envelope_reference_hash),
                ),
            ),
            f(
                "argument_hash",
                record_sha_or_null(retained_ref.map(|(_, reference)| reference.argument_hash)),
            ),
            f(
                "lifeline_request_reference_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.lifeline_request_reference_hash),
                ),
            ),
        ],
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_lifeline_command_dispatch_requirement(name: &'static str, schema: &'static str, present: bool, reason: &'static str, check: &RecoveryLifelineCommandDispatchCheck, comma: bool) {
    emit_inline_record_object(vec![f("fact", s(name)), f("schema", s(schema)), f("status", s(if present { "present" } else { "missing" })), f("required", b(true)), f("scope", s("current_boot")), f("classification", s("local_only")), f("reason", s(if present { "present_for_selftest_only" } else { reason })), f("command_envelope_reference_accepted", b(check.command_envelope_reference_accepted)), f("accepts_lifeline_command_body", no()), f("dispatches_lifeline_command", no()), f("command_execution_enabled", no()), f("rollback_preview_enabled", no()), f("rollback_apply_enabled", no()), f("recovery_memory_writes_enabled", no()), f("durable_writes_enabled", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}

fn execution_completion_denial_reference_value(
    check: &RecoveryLifelineCommandDispatchCheck,
) -> V<'_> {
    if let Some((_, reference)) = check.execution_completion_denial_reference.as_ref() {
        object(vec![
            f("status", s("retained_hash_reference_command_still_denied")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("schema", s(reference.schema)),
            f("stage_name", s(reference.stage_name)),
            f("execution_stage_id", s(reference.execution_stage_id)),
            f("accepts_lifeline_command_body", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f(
                "hashes",
                inline(vec![
                    f("execution_stage_hash", sha(reference.execution_stage_hash)),
                    f(
                        "execution_completion_denial_hash",
                        sha(reference.execution_stage_hash),
                    ),
                    f(
                        "side_effect_gate_hash",
                        sha(reference.side_effect_gate_hash),
                    ),
                    f(
                        "source_rollback_apply_denial_hash",
                        sha(reference.source_rollback_apply_denial_hash),
                    ),
                    f(
                        "source_durable_policy_write_authority_decision_hash",
                        sha(reference.source_durable_policy_write_authority_decision_hash),
                    ),
                    f(
                        "source_recovery_rollback_inspect_source_reference_hash",
                        sha(reference.source_recovery_rollback_inspect_source_reference_hash),
                    ),
                    f(
                        "execution_enablement_hash",
                        record_sha_or_null(reference.execution_enablement_hash),
                    ),
                    f(
                        "execution_preflight_hash",
                        record_sha_or_null(reference.execution_preflight_hash),
                    ),
                    f(
                        "execution_intent_hash",
                        record_sha_or_null(reference.execution_intent_hash),
                    ),
                    f(
                        "execution_commit_gate_hash",
                        record_sha_or_null(reference.execution_commit_gate_hash),
                    ),
                    f(
                        "execution_result_denial_hash",
                        record_sha_or_null(reference.execution_result_denial_hash),
                    ),
                    f(
                        "execution_audit_denial_hash",
                        record_sha_or_null(reference.execution_audit_denial_hash),
                    ),
                    f(
                        "execution_observation_denial_hash",
                        record_sha_or_null(reference.execution_observation_denial_hash),
                    ),
                ]),
            ),
        ])
    } else {
        object(vec![
            f("status", s("missing")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
        ])
    }
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_boundary(
    check: &RecoveryLifelineCommandDispatchCheck,
) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f(
                "command_envelope_reference_available",
                b(check.command_envelope_reference_available),
            ),
            f(
                "command_envelope_reference_accepted",
                b(check.command_envelope_reference_accepted),
            ),
            f(
                "command_body_canonicalization_present",
                b(check.command_body_canonicalization_present),
            ),
            f(
                "command_handler_binding_present",
                b(check.command_handler_binding_present),
            ),
            f(
                "status_read_handler_present",
                b(check.status_read_handler_present),
            ),
            f(
                "rollback_preview_authorization_present",
                b(check.rollback_preview_authorization_present),
            ),
            f(
                "rollback_apply_authorization_present",
                b(check.rollback_apply_authorization_present),
            ),
            f(
                "disable_module_target_binding_present",
                b(check.disable_module_target_binding_present),
            ),
            f(
                "restart_last_good_target_binding_present",
                b(check.restart_last_good_target_binding_present),
            ),
            f(
                "load_artifact_by_hash_target_binding_present",
                b(check.load_artifact_by_hash_target_binding_present),
            ),
            f(
                "recovery_memory_write_authority_present",
                b(check.recovery_memory_write_authority_present),
            ),
            f(
                "durable_audit_rollback_write_authority_present",
                b(check.durable_audit_rollback_write_authority_present),
            ),
            f(
                "service_inventory_side_effect_boundary_present",
                b(check.service_inventory_side_effect_boundary_present),
            ),
            f(
                "command_dispatch_behavior_present",
                b(check.command_dispatch_behavior_present),
            ),
            f(
                "executor_capability_table_present",
                b(check.executor_capability_table_present),
            ),
            f(
                "side_effect_gate_present",
                b(check.side_effect_gate_present),
            ),
            f(
                "execution_enablement_present",
                b(check.execution_enablement_present),
            ),
            f(
                "execution_preflight_present",
                b(check.execution_preflight_present),
            ),
            f(
                "execution_intent_present",
                b(check.execution_intent_present),
            ),
            f(
                "execution_commit_gate_present",
                b(check.execution_commit_gate_present),
            ),
            f(
                "execution_result_denial_present",
                b(check.execution_result_denial_present),
            ),
            f(
                "execution_audit_denial_present",
                b(check.execution_audit_denial_present),
            ),
            f(
                "execution_observation_denial_present",
                b(check.execution_observation_denial_present),
            ),
            f(
                "execution_completion_denial_present",
                b(check.execution_completion_denial_present),
            ),
            f(
                "retained_execution_completion_denial_event_id",
                record_event_or_null(
                    check
                        .execution_completion_denial_reference
                        .as_ref()
                        .map(|(event_id, _)| *event_id),
                ),
            ),
            f(
                "retained_execution_completion_denial_reference",
                execution_completion_denial_reference_value(check),
            ),
            f(
                "accepts_lifeline_command_body",
                b(check.accepts_lifeline_command_body),
            ),
            f(
                "accepts_lifeline_command_envelope",
                b(check.accepts_lifeline_command_envelope),
            ),
            f(
                "dispatches_lifeline_command",
                b(check.dispatches_lifeline_command),
            ),
            f(
                "command_execution_enabled",
                b(check.command_execution_enabled),
            ),
            f(
                "rollback_preview_enabled",
                b(check.rollback_preview_enabled),
            ),
            f("rollback_apply_enabled", b(check.rollback_apply_enabled)),
            f(
                "recovery_memory_writes_enabled",
                b(check.recovery_memory_writes_enabled),
            ),
            f("durable_writes_enabled", b(check.durable_writes_enabled)),
            f("rollback_replay_enabled", b(check.rollback_replay_enabled)),
            f("provider_export_enabled", b(check.provider_export_enabled)),
            f(
                "authorizes_recovery_load",
                b(check.authorizes_recovery_load),
            ),
            f("can_move_beyond_denial", b(check.can_move_beyond_denial)),
            f("loads_recovery_loader", b(check.loads_recovery_loader)),
            f("loads_recovery_artifact", b(check.loads_recovery_artifact)),
            f("creates_durable_records", b(check.creates_durable_records)),
            f("installs_rollback_plan", b(check.installs_rollback_plan)),
            f("allocates_service_slot", b(check.allocates_service_slot)),
            f(
                "service_inventory_change",
                s(check.service_inventory_change),
            ),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_lifeline_status_execution_readiness(
    check: &RecoveryLifelineCommandDispatchCheck,
    retained_status_handler: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineStatusReadHandlerReference,
    )>,
) {
    let retained_status_handler_ref = retained_status_handler.as_ref();
    let ready = recovery_lifeline_status_execution_readiness_ready(
        check,
        retained_status_handler_ref.is_some(),
    );
    let reason = recovery_lifeline_status_execution_readiness_reason(
        check,
        retained_status_handler_ref.is_some(),
    );

    emit_record_property(
        "status_execution_readiness",
        vec![
            f(
                "schema",
                s("raios.recovery_lifeline_status_execution_readiness.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f(
                "status",
                s(if ready {
                    "available_read_only_non_authorizing"
                } else {
                    "blocked_missing_evidence"
                }),
            ),
            f("reason", s(reason)),
            f("command_id", s("recovery.lifeline.status")),
            f(
                "argument_schema",
                s("raios.recovery_lifeline_command.status_args.v0"),
            ),
            f(
                "retained_status_read_handler_event_id",
                record_event_or_null(retained_status_handler_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "status_read_handler_hash",
                record_sha_or_null(
                    retained_status_handler_ref
                        .map(|(_, reference)| reference.status_read_handler_hash),
                ),
            ),
            f(
                "status_read_projection_hash",
                record_sha_or_null(
                    retained_status_handler_ref
                        .map(|(_, reference)| reference.status_read_projection_hash),
                ),
            ),
            f(
                "status_handler_id",
                record_str_or_null(
                    retained_status_handler_ref.map(|(_, reference)| reference.status_handler_id),
                ),
            ),
            f(
                "execution_completion_denial_present",
                b(check.execution_completion_denial_present),
            ),
            f("would_execute_lifeline_status_read", b(ready)),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("executes_lifeline_status", no()),
            f("writes_recovery_memory", no()),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("loads_recovery_artifact", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_selftest_case(
    case: &RecoveryLifelineCommandDispatchSelfTestCase,
    comma: bool,
) {
    emit_selftest_case_fields_split(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        &[
            Str("actual_envelope_status", case.actual_envelope_status),
            Str("actual_envelope_reason", case.actual_envelope_reason),
            Bool(
                "command_envelope_reference_accepted",
                case.command_envelope_reference_accepted,
            ),
            Bool(
                "command_body_canonicalization_present",
                case.command_body_canonicalization_present,
            ),
            Bool(
                "command_handler_binding_present",
                case.command_handler_binding_present,
            ),
        ],
        case.passed,
        &[
            False("accepts_lifeline_command_body"),
            False("accepts_lifeline_command_envelope"),
            Bool(
                "dispatches_lifeline_command",
                case.dispatches_lifeline_command,
            ),
            Bool("command_execution_enabled", case.command_execution_enabled),
            False("memory_writes_enabled"),
            False("provider_export_enabled"),
            False("durable_writes_enabled"),
            False("rollback_replay_enabled"),
            False("rollback_preview_enabled"),
            False("rollback_apply_enabled"),
            False("authorizes_recovery_load"),
            False("can_move_beyond_denial"),
            False("loads_recovery_loader"),
            False("loads_recovery_artifact"),
            False("creates_durable_records"),
            False("installs_rollback_plan"),
            False("allocates_service_slot"),
            Str("service_inventory_change", "none"),
            Bool("load_attempted", case.load_attempted),
        ],
        comma,
    );
}
