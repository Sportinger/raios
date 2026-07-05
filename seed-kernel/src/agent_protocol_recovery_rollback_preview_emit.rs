use alloc::vec;

use crate::{
    agent_protocol_recovery_command_authorization_types::{
        RecoveryRollbackPreviewAuthorizationReferenceCheck,
        RecoveryRollbackPreviewAuthorizationSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_property, record_event_or_null, record_sha_or_null,
        record_str_or_null,
    },
    event_log,
};
use raios_core::record::{Field, Value};

pub(crate) fn emit_recovery_rollback_preview_authorization_reference_object(
    check: &RecoveryRollbackPreviewAuthorizationReferenceCheck<'_>,
) {
    emit_record_property(
        "rollback_preview_authorization_reference",
        vec![
            Field::new("status", Value::Str(check.status)),
            Field::new("reason", Value::Str(check.reason)),
            Field::new("has_reference", Value::Bool(check.has_reference)),
            Field::new("arity_valid", Value::Bool(check.arity_valid)),
            Field::new("scope", Value::Str(check.scope)),
            Field::new("command_id", record_str_or_null(check.command_id)),
            Field::new("argument_schema", record_str_or_null(check.argument_schema)),
            Field::new("target_locator", record_str_or_null(check.target_locator)),
            Field::new(
                "command_dispatch_boundary_id",
                record_str_or_null(check.command_dispatch_boundary_id),
            ),
            Field::new(
                "rollback_preview_authorization_id",
                record_str_or_null(check.rollback_preview_authorization_id),
            ),
            Field::new(
                "retained_recovery_lifeline_status_read_handler_event_id",
                record_str_or_null(check.retained_status_read_handler_event_id),
            ),
            Field::new("argument_hash", record_sha_or_null(check.argument_hash)),
            Field::new(
                "command_envelope_reference_hash",
                record_sha_or_null(check.command_envelope_reference_hash),
            ),
            Field::new(
                "command_body_canonicalization_hash",
                record_sha_or_null(check.command_body_canonicalization_hash),
            ),
            Field::new(
                "handler_binding_hash",
                record_sha_or_null(check.handler_binding_hash),
            ),
            Field::new(
                "status_read_handler_hash",
                record_sha_or_null(check.status_read_handler_hash),
            ),
            Field::new(
                "rollback_preview_projection_hash",
                record_sha_or_null(check.rollback_preview_projection_hash),
            ),
            Field::new(
                "rollback_preview_authorization_hash",
                record_sha_or_null(check.rollback_preview_authorization_hash),
            ),
            Field::new(
                "expected_rollback_preview_authorization_hash",
                record_sha_or_null(check.expected_rollback_preview_authorization_hash),
            ),
            Field::new("valid_hash_reference", Value::Bool(check.valid)),
            Field::new("accepts_raw_command_body", Value::Bool(false)),
            Field::new("accepts_lifeline_command_body", Value::Bool(false)),
            Field::new("dispatches_lifeline_command", Value::Bool(false)),
            Field::new("executes_rollback_preview", Value::Bool(false)),
            Field::new("command_execution_enabled", Value::Bool(false)),
            Field::new("service_inventory_change", Value::Str("none")),
            Field::new("load_attempted", Value::Bool(false)),
        ],
    );
}

pub(crate) fn emit_recovery_rollback_preview_authorization_retained_reference(
    check: &RecoveryRollbackPreviewAuthorizationReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryRollbackPreviewAuthorizationReference,
    )>,
) {
    let (
        latest_event_id,
        latest_rollback_preview_authorization_id,
        latest_rollback_preview_authorization_hash,
    ) = if let Some((event_id, reference)) = retained {
        (
            Value::EventSequence(event_id.sequence()),
            Value::Str(reference.rollback_preview_authorization_id),
            Value::Sha256(reference.rollback_preview_authorization_hash),
        )
    } else {
        (Value::Null, Value::Null, Value::Null)
    };

    emit_record_property(
        "retained_rollback_preview_authorization_reference",
        vec![
            Field::new(
                "status",
                Value::Str(if check.valid {
                    "retained_hash_reference_command_still_denied"
                } else if retained.is_some() {
                    "previous_retained_hash_reference_present"
                } else {
                    "missing"
                }),
            ),
            Field::new("recorded_event_id", record_event_or_null(recorded_event_id)),
            Field::new("scope", Value::Str("current_boot")),
            Field::new("classification", Value::Str("local_only")),
            Field::new("dispatches_lifeline_command", Value::Bool(false)),
            Field::new("executes_rollback_preview", Value::Bool(false)),
            Field::new("command_execution_enabled", Value::Bool(false)),
            Field::new("load_attempted", Value::Bool(false)),
            Field::new("latest_event_id", latest_event_id),
            Field::new(
                "latest_rollback_preview_authorization_id",
                latest_rollback_preview_authorization_id,
            ),
            Field::new(
                "latest_rollback_preview_authorization_hash",
                latest_rollback_preview_authorization_hash,
            ),
        ],
    );
}

pub(crate) fn emit_recovery_rollback_preview_authorization_selftest_case(
    case: &RecoveryRollbackPreviewAuthorizationSelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(
        vec![
            Field::new("case", Value::Str(case.name)),
            Field::new("expected_status", Value::Str(case.expected_status)),
            Field::new("expected_reason", Value::Str(case.expected_reason)),
            Field::new("actual_status", Value::Str(case.actual_status)),
            Field::new("actual_reason", Value::Str(case.actual_reason)),
            Field::new("passed", Value::Bool(case.passed)),
            Field::new("accepts_raw_command_body", Value::Bool(false)),
            Field::new("dispatches_lifeline_command", Value::Bool(false)),
            Field::new("executes_rollback_preview", Value::Bool(false)),
            Field::new("command_execution_enabled", Value::Bool(false)),
            Field::new("load_attempted", Value::Bool(false)),
        ],
        comma,
    );
}
