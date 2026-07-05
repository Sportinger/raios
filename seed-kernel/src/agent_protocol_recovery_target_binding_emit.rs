use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_recovery_command_authorization_types::{
        RecoveryDisableModuleTargetBindingReferenceCheck,
        RecoveryDisableModuleTargetBindingSelfTestCase,
        RecoveryLoadArtifactByHashTargetBindingReferenceCheck,
        RecoveryLoadArtifactByHashTargetBindingSelfTestCase,
        RecoveryRestartLastGoodTargetBindingReferenceCheck,
        RecoveryRestartLastGoodTargetBindingSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_property, extend_false_fields, record_bool as b,
        record_event_or_null, record_false as no, record_field as f, record_null as null,
        record_object as object, record_sha_or_null, record_str as s, record_str_or_null,
    },
    event_log,
};
use raios_core::record::Field;

fn extend_hash_fields<'a>(fields: &mut Vec<Field<'a>>, pairs: &[(&'a str, Option<[u8; 32]>)]) {
    let mut idx = 0usize;
    while idx < pairs.len() {
        fields.push(f(pairs[idx].0, record_sha_or_null(pairs[idx].1)));
        idx += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_target_reference_object<'a>(
    property: &'static str,
    status: &'a str,
    reason: &'a str,
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    command_id: Option<&'a str>,
    argument_schema: Option<&'a str>,
    target_locator: Option<&'a str>,
    command_dispatch_boundary_id: Option<&'a str>,
    id_key: &'static str,
    id: Option<&'a str>,
    retained_key: &'static str,
    retained_event_id: Option<&'a str>,
    hash_fields: &[(&'a str, Option<[u8; 32]>)],
    valid: bool,
    action_false_fields: &'static [&'static str],
) {
    let mut fields = vec![
        f("status", s(status)),
        f("reason", s(reason)),
        f("has_reference", b(has_reference)),
        f("arity_valid", b(arity_valid)),
        f("scope", s(scope)),
        f("command_id", record_str_or_null(command_id)),
        f("argument_schema", record_str_or_null(argument_schema)),
        f("target_locator", record_str_or_null(target_locator)),
        f(
            "command_dispatch_boundary_id",
            record_str_or_null(command_dispatch_boundary_id),
        ),
        f(id_key, record_str_or_null(id)),
        f(retained_key, record_str_or_null(retained_event_id)),
    ];
    extend_hash_fields(&mut fields, hash_fields);
    fields.push(f("valid_hash_reference", b(valid)));
    extend_false_fields(&mut fields, action_false_fields);
    fields.push(f("service_inventory_change", s("none")));
    fields.push(f("load_attempted", no()));

    emit_record_property(property, fields);
}

fn emit_target_retained_reference<'a>(
    property: &'static str,
    valid: bool,
    recorded_event_id: Option<event_log::EventId>,
    retained_present: bool,
    false_fields: &'static [&'static str],
    latest_event_id: Option<event_log::EventId>,
    latest_id_key: &'static str,
    latest_id: Option<&'a str>,
    latest_hash_key: &'static str,
    latest_hash: Option<[u8; 32]>,
    latest_source_rollback_apply_denial_hash: Option<[u8; 32]>,
    latest_source_durable_policy_write_authority_decision_hash: Option<[u8; 32]>,
    latest_source_recovery_rollback_inspect_source_reference_hash: Option<[u8; 32]>,
) {
    let mut fields = vec![
        f(
            "status",
            s(if valid {
                "retained_hash_reference_command_still_denied"
            } else if retained_present {
                "previous_retained_hash_reference_present"
            } else {
                "missing"
            }),
        ),
        f("recorded_event_id", record_event_or_null(recorded_event_id)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
    ];
    extend_false_fields(&mut fields, false_fields);
    fields.extend([
        f("latest_event_id", record_event_or_null(latest_event_id)),
        f(latest_id_key, record_str_or_null(latest_id)),
        f(latest_hash_key, record_sha_or_null(latest_hash)),
        f(
            "latest_source_rollback_apply_denial_hash",
            record_sha_or_null(latest_source_rollback_apply_denial_hash),
        ),
        f(
            "latest_source_durable_policy_write_authority_decision_hash",
            record_sha_or_null(latest_source_durable_policy_write_authority_decision_hash),
        ),
        f(
            "latest_source_recovery_rollback_inspect_source_reference_hash",
            record_sha_or_null(latest_source_recovery_rollback_inspect_source_reference_hash),
        ),
    ]);

    emit_record_property(property, fields);
}

pub(crate) fn emit_recovery_artifact_load_denial_source_evidence() {
    let latest = event_log::latest_recovery_artifact_load_denied_binding();
    let latest_ref = latest.as_ref();

    emit_record_property(
        "recovery_artifact_load_denial_source",
        vec![
            f("schema", s("raios.recovery_artifact_load_denial_source.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f(
                "status",
                s(latest_ref
                    .map(|(_, binding)| binding.recovery_load_binding_status)
                    .unwrap_or("missing")),
            ),
            f(
                "reason",
                s(latest_ref
                    .map(|(_, binding)| binding.recovery_load_binding_reason)
                    .unwrap_or("recovery_artifact_load_denial_event_missing")),
            ),
            f("source_evidence_present", b(latest_ref.is_some())),
            f("source_method", s("recovery.load_artifact")),
            f("read_only", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_records", no()),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("loads_recovery_artifact", no()),
            f("authorizes_recovery_load", no()),
            f("writes_recovery_memory", no()),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "retained_recovery_artifact_load_denied_event_id",
                record_event_or_null(latest_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "recovery_load_binding",
                object(vec![
                    f(
                        "status",
                        record_str_or_null(
                            latest_ref.map(|(_, binding)| binding.recovery_load_binding_status),
                        ),
                    ),
                    f(
                        "reason",
                        record_str_or_null(
                            latest_ref.map(|(_, binding)| binding.recovery_load_binding_reason),
                        ),
                    ),
                    f(
                        "retained_execution_completion_denial_event_id",
                        latest_ref
                            .and_then(|(_, binding)| {
                                binding.retained_execution_completion_denial_event_id
                            })
                            .map_or(null(), |event_id| record_event_or_null(Some(event_id))),
                    ),
                    f(
                        "execution_completion_denial_hash",
                        record_sha_or_null(
                            latest_ref
                                .and_then(|(_, binding)| binding.execution_completion_denial_hash),
                        ),
                    ),
                    f(
                        "side_effect_gate_hash",
                        record_sha_or_null(
                            latest_ref.and_then(|(_, binding)| binding.side_effect_gate_hash),
                        ),
                    ),
                    f(
                        "source_rollback_apply_denial_hash",
                        record_sha_or_null(
                            latest_ref
                                .and_then(|(_, binding)| binding.source_rollback_apply_denial_hash),
                        ),
                    ),
                    f(
                        "source_durable_policy_write_authority_decision_hash",
                        record_sha_or_null(latest_ref.and_then(|(_, binding)| {
                            binding.source_durable_policy_write_authority_decision_hash
                        })),
                    ),
                    f(
                        "source_recovery_rollback_inspect_source_reference_hash",
                        record_sha_or_null(latest_ref.and_then(|(_, binding)| {
                            binding.source_recovery_rollback_inspect_source_reference_hash
                        })),
                    ),
                    f("loads_recovery_artifact", no()),
                    f("dispatches_lifeline_command", no()),
                    f("command_execution_enabled", no()),
                    f("load_attempted", no()),
                ]),
            ),
        ],
    );
}

pub(crate) fn emit_recovery_disable_module_target_binding_reference_object(
    check: &RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
) {
    emit_target_reference_object(
        "disable_module_target_binding_reference",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        check.command_id,
        check.argument_schema,
        check.target_locator,
        check.command_dispatch_boundary_id,
        "disable_module_target_id",
        check.disable_module_target_id,
        "retained_recovery_rollback_apply_authorization_event_id",
        check.retained_rollback_apply_authorization_event_id,
        &[
            ("argument_hash", check.argument_hash),
            (
                "command_envelope_reference_hash",
                check.command_envelope_reference_hash,
            ),
            (
                "command_body_canonicalization_hash",
                check.command_body_canonicalization_hash,
            ),
            ("handler_binding_hash", check.handler_binding_hash),
            ("status_read_handler_hash", check.status_read_handler_hash),
            (
                "rollback_preview_authorization_hash",
                check.rollback_preview_authorization_hash,
            ),
            (
                "rollback_apply_authorization_hash",
                check.rollback_apply_authorization_hash,
            ),
            (
                "source_rollback_apply_denial_hash",
                check.source_rollback_apply_denial_hash,
            ),
            (
                "source_durable_policy_write_authority_decision_hash",
                check.source_durable_policy_write_authority_decision_hash,
            ),
            (
                "source_recovery_rollback_inspect_source_reference_hash",
                check.source_recovery_rollback_inspect_source_reference_hash,
            ),
            (
                "disable_module_target_projection_hash",
                check.disable_module_target_projection_hash,
            ),
            (
                "disable_module_target_binding_hash",
                check.disable_module_target_binding_hash,
            ),
            (
                "expected_disable_module_target_binding_hash",
                check.expected_disable_module_target_binding_hash,
            ),
        ],
        check.valid,
        &[
            "accepts_raw_command_body",
            "accepts_lifeline_command_body",
            "dispatches_lifeline_command",
            "disables_module",
            "command_execution_enabled",
        ],
    );
}

pub(crate) fn emit_recovery_disable_module_target_binding_retained_reference(
    check: &RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryDisableModuleTargetBindingReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_target_retained_reference(
        "retained_disable_module_target_binding_reference",
        check.valid,
        recorded_event_id,
        retained_ref.is_some(),
        &[
            "dispatches_lifeline_command",
            "disables_module",
            "command_execution_enabled",
            "load_attempted",
        ],
        retained_ref.map(|(event_id, _)| *event_id),
        "latest_disable_module_target_id",
        retained_ref.map(|(_, reference)| reference.disable_module_target_id),
        "latest_disable_module_target_binding_hash",
        retained_ref.map(|(_, reference)| reference.disable_module_target_binding_hash),
        retained_ref.map(|(_, reference)| reference.source_rollback_apply_denial_hash),
        retained_ref
            .map(|(_, reference)| reference.source_durable_policy_write_authority_decision_hash),
        retained_ref
            .map(|(_, reference)| reference.source_recovery_rollback_inspect_source_reference_hash),
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_disable_module_target_binding_selftest_case(case: &RecoveryDisableModuleTargetBindingSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("accepts_raw_command_body", no()), f("dispatches_lifeline_command", no()), f("disables_module", no()), f("command_execution_enabled", no()), f("load_attempted", no())], comma);
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_reference_object(
    check: &RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
) {
    emit_target_reference_object(
        "restart_last_good_target_binding_reference",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        check.command_id,
        check.argument_schema,
        check.target_locator,
        check.command_dispatch_boundary_id,
        "restart_last_good_target_id",
        check.restart_last_good_target_id,
        "retained_recovery_disable_module_target_binding_event_id",
        check.retained_disable_module_target_binding_event_id,
        &[
            ("argument_hash", check.argument_hash),
            (
                "command_envelope_reference_hash",
                check.command_envelope_reference_hash,
            ),
            (
                "command_body_canonicalization_hash",
                check.command_body_canonicalization_hash,
            ),
            ("handler_binding_hash", check.handler_binding_hash),
            ("status_read_handler_hash", check.status_read_handler_hash),
            (
                "rollback_preview_authorization_hash",
                check.rollback_preview_authorization_hash,
            ),
            (
                "rollback_apply_authorization_hash",
                check.rollback_apply_authorization_hash,
            ),
            (
                "disable_module_target_binding_hash",
                check.disable_module_target_binding_hash,
            ),
            (
                "source_rollback_apply_denial_hash",
                check.source_rollback_apply_denial_hash,
            ),
            (
                "source_durable_policy_write_authority_decision_hash",
                check.source_durable_policy_write_authority_decision_hash,
            ),
            (
                "source_recovery_rollback_inspect_source_reference_hash",
                check.source_recovery_rollback_inspect_source_reference_hash,
            ),
            (
                "restart_last_good_target_projection_hash",
                check.restart_last_good_target_projection_hash,
            ),
            (
                "restart_last_good_target_binding_hash",
                check.restart_last_good_target_binding_hash,
            ),
            (
                "expected_restart_last_good_target_binding_hash",
                check.expected_restart_last_good_target_binding_hash,
            ),
        ],
        check.valid,
        &[
            "accepts_raw_command_body",
            "accepts_lifeline_command_body",
            "dispatches_lifeline_command",
            "restarts_last_good",
            "command_execution_enabled",
        ],
    );
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_retained_reference(
    check: &RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryRestartLastGoodTargetBindingReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_target_retained_reference(
        "retained_restart_last_good_target_binding_reference",
        check.valid,
        recorded_event_id,
        retained_ref.is_some(),
        &[
            "dispatches_lifeline_command",
            "restarts_last_good",
            "command_execution_enabled",
            "load_attempted",
        ],
        retained_ref.map(|(event_id, _)| *event_id),
        "latest_restart_last_good_target_id",
        retained_ref.map(|(_, reference)| reference.restart_last_good_target_id),
        "latest_restart_last_good_target_binding_hash",
        retained_ref.map(|(_, reference)| reference.restart_last_good_target_binding_hash),
        retained_ref.map(|(_, reference)| reference.source_rollback_apply_denial_hash),
        retained_ref
            .map(|(_, reference)| reference.source_durable_policy_write_authority_decision_hash),
        retained_ref
            .map(|(_, reference)| reference.source_recovery_rollback_inspect_source_reference_hash),
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_restart_last_good_target_binding_selftest_case(case: &RecoveryRestartLastGoodTargetBindingSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("accepts_raw_command_body", no()), f("dispatches_lifeline_command", no()), f("restarts_last_good", no()), f("command_execution_enabled", no()), f("load_attempted", no())], comma);
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_reference_object(
    check: &RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
) {
    emit_target_reference_object(
        "load_artifact_by_hash_target_binding_reference",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        check.command_id,
        check.argument_schema,
        check.target_locator,
        check.command_dispatch_boundary_id,
        "load_artifact_by_hash_target_id",
        check.load_artifact_by_hash_target_id,
        "retained_recovery_restart_last_good_target_binding_event_id",
        check.retained_restart_last_good_target_binding_event_id,
        &[
            ("argument_hash", check.argument_hash),
            (
                "command_envelope_reference_hash",
                check.command_envelope_reference_hash,
            ),
            (
                "command_body_canonicalization_hash",
                check.command_body_canonicalization_hash,
            ),
            ("handler_binding_hash", check.handler_binding_hash),
            ("status_read_handler_hash", check.status_read_handler_hash),
            (
                "rollback_preview_authorization_hash",
                check.rollback_preview_authorization_hash,
            ),
            (
                "rollback_apply_authorization_hash",
                check.rollback_apply_authorization_hash,
            ),
            (
                "disable_module_target_binding_hash",
                check.disable_module_target_binding_hash,
            ),
            (
                "restart_last_good_target_binding_hash",
                check.restart_last_good_target_binding_hash,
            ),
            (
                "source_rollback_apply_denial_hash",
                check.source_rollback_apply_denial_hash,
            ),
            (
                "source_durable_policy_write_authority_decision_hash",
                check.source_durable_policy_write_authority_decision_hash,
            ),
            (
                "source_recovery_rollback_inspect_source_reference_hash",
                check.source_recovery_rollback_inspect_source_reference_hash,
            ),
            (
                "load_artifact_by_hash_target_artifact_hash",
                check.load_artifact_by_hash_target_artifact_hash,
            ),
            (
                "load_artifact_by_hash_target_projection_hash",
                check.load_artifact_by_hash_target_projection_hash,
            ),
            (
                "load_artifact_by_hash_target_binding_hash",
                check.load_artifact_by_hash_target_binding_hash,
            ),
            (
                "expected_load_artifact_by_hash_target_binding_hash",
                check.expected_load_artifact_by_hash_target_binding_hash,
            ),
        ],
        check.valid,
        &[
            "accepts_raw_command_body",
            "accepts_lifeline_command_body",
            "dispatches_lifeline_command",
            "loads_recovery_artifact",
            "authorizes_recovery_load",
            "command_execution_enabled",
        ],
    );
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_retained_reference(
    check: &RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLoadArtifactByHashTargetBindingReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_target_retained_reference(
        "retained_load_artifact_by_hash_target_binding_reference",
        check.valid,
        recorded_event_id,
        retained_ref.is_some(),
        &[
            "dispatches_lifeline_command",
            "loads_recovery_artifact",
            "authorizes_recovery_load",
            "command_execution_enabled",
            "load_attempted",
        ],
        retained_ref.map(|(event_id, _)| *event_id),
        "latest_load_artifact_by_hash_target_id",
        retained_ref.map(|(_, reference)| reference.load_artifact_by_hash_target_id),
        "latest_load_artifact_by_hash_target_binding_hash",
        retained_ref.map(|(_, reference)| reference.load_artifact_by_hash_target_binding_hash),
        retained_ref.map(|(_, reference)| reference.source_rollback_apply_denial_hash),
        retained_ref
            .map(|(_, reference)| reference.source_durable_policy_write_authority_decision_hash),
        retained_ref
            .map(|(_, reference)| reference.source_recovery_rollback_inspect_source_reference_hash),
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_selftest_case(case: &RecoveryLoadArtifactByHashTargetBindingSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("accepts_raw_command_body", no()), f("dispatches_lifeline_command", no()), f("loads_recovery_artifact", no()), f("authorizes_recovery_load", no()), f("command_execution_enabled", no()), f("load_attempted", no())], comma);
}
