use alloc::vec;

use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandEnvelopeReferenceCheck, RecoveryLifelineCommandEnvelopeSelfTestCase,
    },
    agent_protocol_recovery_lifeline::RecoveryLifelineCommandSpec,
    agent_protocol_support::{
        emit_record_object, emit_record_property, emit_selftest_case_fields_split,
        record_bool as b, record_event_or_null, record_false as no, record_field as f,
        record_sha_or_null, record_str as s, record_str_or_null,
        SelftestReportField::{Bool, False, Str},
    },
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_envelope_allowed_command(
    spec: RecoveryLifelineCommandSpec,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("command_id", s(spec.command_id)),
            f("argument_schema", s(spec.argument_schema)),
            f("required_capability", s(spec.required_capability)),
            f("accepts_command_body", no()),
            f("dispatches_command", no()),
            f("command_execution_enabled", no()),
        ],
        8,
        comma,
    );
}

pub(crate) fn emit_recovery_lifeline_command_envelope_reference_object(
    check: &RecoveryLifelineCommandEnvelopeReferenceCheck<'_>,
) {
    emit_record_property(
        "command_envelope_reference",
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f("has_reference", b(check.has_reference)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("command_id", record_str_or_null(check.command_id)),
            f("argument_schema", record_str_or_null(check.argument_schema)),
            f(
                "required_capability",
                record_str_or_null(check.required_capability),
            ),
            f("target_locator", record_str_or_null(check.target_locator)),
            f(
                "command_admission_boundary_id",
                record_str_or_null(check.command_admission_boundary_id),
            ),
            f(
                "retained_recovery_lifeline_request_event_id",
                record_str_or_null(check.retained_lifeline_request_event_id),
            ),
            f(
                "lifeline_request_reference_hash",
                record_sha_or_null(check.lifeline_request_reference_hash),
            ),
            f("argument_hash", record_sha_or_null(check.argument_hash)),
            f(
                "command_envelope_reference_hash",
                record_sha_or_null(check.command_envelope_reference_hash),
            ),
            f(
                "expected_command_envelope_reference_hash",
                record_sha_or_null(check.expected_command_envelope_reference_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_lifeline_command_envelope", no()),
            f("accepts_lifeline_command_body", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("authorizes_recovery_load", no()),
            f("can_move_beyond_denial", no()),
            f("loads_recovery_artifact", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_envelope_retained_reference(
    check: &RecoveryLifelineCommandEnvelopeReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandEnvelopeReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_command_envelope_reference",
        vec![
            f(
                "status",
                s(if check.valid {
                    "retained_hash_reference_command_still_denied"
                } else if retained.is_some() {
                    "previous_retained_hash_reference_present"
                } else {
                    "missing"
                }),
            ),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("authorizes_recovery_load", no()),
            f("loads_recovery_artifact", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "latest_event_id",
                record_event_or_null(retained_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "latest_command_id",
                record_str_or_null(retained_ref.map(|(_, reference)| reference.command_id)),
            ),
            f(
                "latest_command_envelope_reference_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.command_envelope_reference_hash),
                ),
            ),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_envelope_selftest_case(
    case: &RecoveryLifelineCommandEnvelopeSelfTestCase,
    comma: bool,
) {
    emit_selftest_case_fields_split(
        case.name,
        case.expected_status,
        case.expected_reason,
        case.actual_status,
        case.actual_reason,
        &[
            Str("actual_admission_status", case.actual_admission_status),
            Str("actual_admission_reason", case.actual_admission_reason),
            Bool(
                "command_admission_boundary_exposed",
                case.command_admission_boundary_exposed,
            ),
            Bool(
                "command_admission_accepted",
                case.command_admission_accepted,
            ),
            Bool(
                "command_envelope_reference_present",
                case.command_envelope_reference_present,
            ),
            Bool("command_id_supported", case.command_id_supported),
            Bool("argument_schema_matches", case.argument_schema_matches),
            Bool("argument_hash_present", case.argument_hash_present),
            Bool(
                "required_capability_matches",
                case.required_capability_matches,
            ),
            Bool("target_locator_present", case.target_locator_present),
            Bool("reference_hash_matches", case.reference_hash_matches),
        ],
        case.passed,
        &[
            Bool("command_execution_enabled", case.command_execution_enabled),
            Bool(
                "accepts_lifeline_command_envelope",
                case.accepts_lifeline_command_envelope,
            ),
            Bool(
                "dispatches_lifeline_command",
                case.dispatches_lifeline_command,
            ),
            False("memory_writes_enabled"),
            False("provider_export_enabled"),
            False("durable_writes_enabled"),
            False("rollback_replay_enabled"),
            False("rollback_preview_enabled"),
            False("rollback_apply_enabled"),
            Bool("authorizes_recovery_load", case.authorizes_recovery_load),
            Bool("can_move_beyond_denial", case.can_move_beyond_denial),
            Bool("loads_recovery_loader", case.loads_recovery_loader),
            Bool("loads_recovery_artifact", case.loads_recovery_artifact),
            Bool("creates_durable_records", case.creates_durable_records),
            Bool("installs_rollback_plan", case.installs_rollback_plan),
            Bool("allocates_service_slot", case.allocates_service_slot),
            Str("service_inventory_change", case.service_inventory_change),
            Bool("load_attempted", case.load_attempted),
        ],
        comma,
    );
}
