use alloc::vec;

use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandEnvelopeReferenceCheck, RecoveryLifelineCommandEnvelopeSelfTestCase,
    },
    agent_protocol_recovery_lifeline::RecoveryLifelineCommandSpec,
    agent_protocol_support::{
        emit_inline_record_object, emit_record_object, emit_record_property, record_bool as b,
        record_event_or_null, record_false as no, record_field as f, record_sha_or_null,
        record_str as s, record_str_or_null,
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
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("actual_admission_status", s(case.actual_admission_status)),
            f("actual_admission_reason", s(case.actual_admission_reason)),
            f(
                "command_admission_boundary_exposed",
                b(case.command_admission_boundary_exposed),
            ),
            f(
                "command_admission_accepted",
                b(case.command_admission_accepted),
            ),
            f(
                "command_envelope_reference_present",
                b(case.command_envelope_reference_present),
            ),
            f("command_id_supported", b(case.command_id_supported)),
            f("argument_schema_matches", b(case.argument_schema_matches)),
            f("argument_hash_present", b(case.argument_hash_present)),
            f(
                "required_capability_matches",
                b(case.required_capability_matches),
            ),
            f("target_locator_present", b(case.target_locator_present)),
            f("reference_hash_matches", b(case.reference_hash_matches)),
            f("passed", b(case.passed)),
            f(
                "command_execution_enabled",
                b(case.command_execution_enabled),
            ),
            f(
                "accepts_lifeline_command_envelope",
                b(case.accepts_lifeline_command_envelope),
            ),
            f(
                "dispatches_lifeline_command",
                b(case.dispatches_lifeline_command),
            ),
            f("memory_writes_enabled", no()),
            f("provider_export_enabled", no()),
            f("durable_writes_enabled", no()),
            f("rollback_replay_enabled", no()),
            f("rollback_preview_enabled", no()),
            f("rollback_apply_enabled", no()),
            f("authorizes_recovery_load", b(case.authorizes_recovery_load)),
            f("can_move_beyond_denial", b(case.can_move_beyond_denial)),
            f("loads_recovery_loader", b(case.loads_recovery_loader)),
            f("loads_recovery_artifact", b(case.loads_recovery_artifact)),
            f("creates_durable_records", b(case.creates_durable_records)),
            f("installs_rollback_plan", b(case.installs_rollback_plan)),
            f("allocates_service_slot", b(case.allocates_service_slot)),
            f("service_inventory_change", s(case.service_inventory_change)),
            f("load_attempted", b(case.load_attempted)),
        ],
        comma,
    );
}
