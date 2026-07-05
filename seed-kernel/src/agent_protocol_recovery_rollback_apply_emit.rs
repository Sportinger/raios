use alloc::vec;

use crate::{
    agent_protocol_recovery_command_authorization_types::{
        RecoveryRollbackApplyAuthorizationReferenceCheck,
        RecoveryRollbackApplyAuthorizationSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_property, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_sha_or_null, record_str as s,
        record_str_or_null,
    },
    event_log,
};

pub(crate) fn emit_recovery_rollback_apply_authorization_reference_object(
    check: &RecoveryRollbackApplyAuthorizationReferenceCheck<'_>,
) {
    emit_record_property(
        "rollback_apply_authorization_reference",
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f("has_reference", b(check.has_reference)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("command_id", record_str_or_null(check.command_id)),
            f("argument_schema", record_str_or_null(check.argument_schema)),
            f("target_locator", record_str_or_null(check.target_locator)),
            f(
                "command_dispatch_boundary_id",
                record_str_or_null(check.command_dispatch_boundary_id),
            ),
            f(
                "rollback_apply_authorization_id",
                record_str_or_null(check.rollback_apply_authorization_id),
            ),
            f(
                "retained_recovery_rollback_preview_authorization_event_id",
                record_str_or_null(check.retained_rollback_preview_authorization_event_id),
            ),
            f("argument_hash", record_sha_or_null(check.argument_hash)),
            f(
                "command_envelope_reference_hash",
                record_sha_or_null(check.command_envelope_reference_hash),
            ),
            f(
                "command_body_canonicalization_hash",
                record_sha_or_null(check.command_body_canonicalization_hash),
            ),
            f(
                "handler_binding_hash",
                record_sha_or_null(check.handler_binding_hash),
            ),
            f(
                "status_read_handler_hash",
                record_sha_or_null(check.status_read_handler_hash),
            ),
            f(
                "rollback_preview_authorization_hash",
                record_sha_or_null(check.rollback_preview_authorization_hash),
            ),
            f(
                "rollback_apply_projection_hash",
                record_sha_or_null(check.rollback_apply_projection_hash),
            ),
            f(
                "source_rollback_apply_denial_hash",
                record_sha_or_null(check.source_rollback_apply_denial_hash),
            ),
            f(
                "source_durable_policy_write_authority_decision_hash",
                record_sha_or_null(check.source_durable_policy_write_authority_decision_hash),
            ),
            f(
                "source_recovery_rollback_inspect_source_reference_hash",
                record_sha_or_null(check.source_recovery_rollback_inspect_source_reference_hash),
            ),
            f(
                "rollback_apply_authorization_hash",
                record_sha_or_null(check.rollback_apply_authorization_hash),
            ),
            f(
                "expected_rollback_apply_authorization_hash",
                record_sha_or_null(check.expected_rollback_apply_authorization_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("dispatches_lifeline_command", no()),
            f("executes_rollback_apply", no()),
            f("command_execution_enabled", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_rollback_apply_authorization_retained_reference(
    check: &RecoveryRollbackApplyAuthorizationReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryRollbackApplyAuthorizationReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_rollback_apply_authorization_reference",
        vec![
            f(
                "status",
                s(if check.valid {
                    "retained_hash_reference_command_still_denied"
                } else if retained_ref.is_some() {
                    "previous_retained_hash_reference_present"
                } else {
                    "missing"
                }),
            ),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("dispatches_lifeline_command", no()),
            f("executes_rollback_apply", no()),
            f("command_execution_enabled", no()),
            f("load_attempted", no()),
            f(
                "latest_event_id",
                record_event_or_null(retained_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "latest_rollback_apply_authorization_id",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.rollback_apply_authorization_id),
                ),
            ),
            f(
                "latest_rollback_apply_authorization_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.rollback_apply_authorization_hash),
                ),
            ),
            f(
                "latest_source_rollback_apply_denial_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.source_rollback_apply_denial_hash),
                ),
            ),
            f(
                "latest_source_durable_policy_write_authority_decision_hash",
                record_sha_or_null(retained_ref.map(|(_, reference)| {
                    reference.source_durable_policy_write_authority_decision_hash
                })),
            ),
            f(
                "latest_source_recovery_rollback_inspect_source_reference_hash",
                record_sha_or_null(retained_ref.map(|(_, reference)| {
                    reference.source_recovery_rollback_inspect_source_reference_hash
                })),
            ),
        ],
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_rollback_apply_authorization_selftest_case(case: &RecoveryRollbackApplyAuthorizationSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("accepts_raw_command_body", no()), f("dispatches_lifeline_command", no()), f("executes_rollback_apply", no()), f("command_execution_enabled", no()), f("load_attempted", no())], comma);
}
