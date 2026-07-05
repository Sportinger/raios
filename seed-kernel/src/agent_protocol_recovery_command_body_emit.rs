use alloc::vec;

use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandBodyCanonicalizationReferenceCheck,
        RecoveryLifelineCommandBodyCanonicalizationSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_property, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_sha_or_null, record_str as s,
        record_str_or_null,
    },
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_reference_object(
    check: &RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_>,
) {
    emit_record_property(
        "command_body_canonicalization_reference",
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
                "retained_recovery_lifeline_command_envelope_event_id",
                record_str_or_null(check.retained_command_envelope_reference_event_id),
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
                "expected_command_body_canonicalization_hash",
                record_sha_or_null(check.expected_command_body_canonicalization_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("accepts_lifeline_command_envelope", no()),
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

#[rustfmt::skip]
pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_requirement(name: &'static str, schema: &'static str, reason: &'static str, comma: bool) {
    emit_inline_record_object(vec![f("fact", s(name)), f("schema", s(schema)), f("status", s("missing")), f("required", b(true)), f("scope", s("current_boot")), f("classification", s("local_only")), f("reason", s(reason)), f("accepts_raw_command_body", no()), f("accepts_lifeline_command_body", no()), f("dispatches_lifeline_command", no()), f("command_execution_enabled", no()), f("rollback_preview_enabled", no()), f("rollback_apply_enabled", no()), f("recovery_memory_writes_enabled", no()), f("durable_writes_enabled", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_retained_reference(
    check: &RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandBodyCanonicalizationReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_command_body_canonicalization_reference",
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
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
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
                "latest_command_body_canonicalization_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.command_body_canonicalization_hash),
                ),
            ),
        ],
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_selftest_case(case: &RecoveryLifelineCommandBodyCanonicalizationSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("actual_dispatch_status", s(case.actual_dispatch_status)), f("actual_dispatch_reason", s(case.actual_dispatch_reason)), f("command_body_reference_accepted", b(case.command_body_reference_accepted)), f("body_hash_matches", b(case.body_hash_matches)), f("passed", b(case.passed)), f("accepts_raw_command_body", no()), f("accepts_lifeline_command_body", no()), f("accepts_lifeline_command_envelope", no()), f("dispatches_lifeline_command", b(case.dispatches_lifeline_command)), f("command_execution_enabled", b(case.command_execution_enabled)), f("memory_writes_enabled", no()), f("provider_export_enabled", no()), f("durable_writes_enabled", no()), f("rollback_replay_enabled", no()), f("rollback_preview_enabled", no()), f("rollback_apply_enabled", no()), f("authorizes_recovery_load", no()), f("can_move_beyond_denial", no()), f("loads_recovery_loader", no()), f("loads_recovery_artifact", no()), f("creates_durable_records", no()), f("installs_rollback_plan", no()), f("allocates_service_slot", no()), f("service_inventory_change", s("none")), f("load_attempted", b(case.load_attempted))], comma);
}
