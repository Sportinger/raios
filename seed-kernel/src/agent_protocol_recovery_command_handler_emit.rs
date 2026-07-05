use alloc::vec;

use crate::{
    agent_protocol_recovery_command_authorization_types::{
        RecoveryLifelineCommandHandlerBindingReferenceCheck,
        RecoveryLifelineCommandHandlerBindingSelfTestCase,
    },
    agent_protocol_support::{
        emit_record_property, emit_selftest_case, record_event_or_null, record_sha_or_null,
        record_str_or_null, SelftestReportField::False,
    },
    event_log,
};
use raios_core::record::{Field, Value};

pub(crate) fn emit_recovery_lifeline_command_handler_binding_reference_object(
    check: &RecoveryLifelineCommandHandlerBindingReferenceCheck<'_>,
) {
    emit_record_property(
        "command_handler_binding_reference",
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
            Field::new("handler_id", record_str_or_null(check.handler_id)),
            Field::new(
                "retained_recovery_lifeline_command_body_canonicalization_event_id",
                record_str_or_null(check.retained_command_body_canonicalization_event_id),
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
                "handler_input_binding_hash",
                record_sha_or_null(check.handler_input_binding_hash),
            ),
            Field::new(
                "handler_binding_hash",
                record_sha_or_null(check.handler_binding_hash),
            ),
            Field::new(
                "expected_handler_binding_hash",
                record_sha_or_null(check.expected_handler_binding_hash),
            ),
            Field::new("valid_hash_reference", Value::Bool(check.valid)),
            Field::new("accepts_raw_command_body", Value::Bool(false)),
            Field::new("accepts_lifeline_command_body", Value::Bool(false)),
            Field::new("dispatches_lifeline_command", Value::Bool(false)),
            Field::new("command_execution_enabled", Value::Bool(false)),
            Field::new("service_inventory_change", Value::Str("none")),
            Field::new("load_attempted", Value::Bool(false)),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_handler_binding_retained_reference(
    check: &RecoveryLifelineCommandHandlerBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandHandlerBindingReference,
    )>,
) {
    let (latest_event_id, latest_handler_id, latest_handler_binding_hash) =
        if let Some((event_id, reference)) = retained {
            (
                Value::EventSequence(event_id.sequence()),
                Value::Str(reference.handler_id),
                Value::Sha256(reference.handler_binding_hash),
            )
        } else {
            (Value::Null, Value::Null, Value::Null)
        };

    emit_record_property(
        "retained_command_handler_binding_reference",
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
            Field::new("command_execution_enabled", Value::Bool(false)),
            Field::new("load_attempted", Value::Bool(false)),
            Field::new("latest_event_id", latest_event_id),
            Field::new("latest_handler_id", latest_handler_id),
            Field::new("latest_handler_binding_hash", latest_handler_binding_hash),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_handler_binding_selftest_case(
    case: &RecoveryLifelineCommandHandlerBindingSelfTestCase,
    comma: bool,
) {
    emit_selftest_case(
        case,
        &[
            False("accepts_raw_command_body"),
            False("dispatches_lifeline_command"),
            False("command_execution_enabled"),
            False("load_attempted"),
        ],
        comma,
    );
}
