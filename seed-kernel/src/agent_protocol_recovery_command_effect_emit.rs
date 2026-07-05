use alloc::vec;

use crate::{
    agent_protocol_recovery_command_effect_types::{
        RecoveryLifelineCommandDispatchBehaviorReferenceCheck,
        RecoveryLifelineCommandDispatchBehaviorSelfTestCase,
        RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck,
        RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase,
        RecoveryLifelineCommandSideEffectGateReferenceCheck,
        RecoveryLifelineCommandSideEffectGateSelfTestCase,
    },
    agent_protocol_support::{
        emit_record_property, emit_selftest_case, record_bool as b, record_event_or_null,
        record_false as no, record_field as f, record_sha_or_null, record_str as s,
        record_str_or_null,
        SelftestReportField::{False, Str},
    },
    event_log,
};
pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_reference_object(
    check: &RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
) {
    emit_record_property(
        "command_dispatch_behavior_reference",
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
                "command_dispatch_behavior_id",
                record_str_or_null(check.command_dispatch_behavior_id),
            ),
            f(
                "retained_service_inventory_side_effect_boundary_event_id",
                record_str_or_null(check.retained_service_inventory_side_effect_boundary_event_id),
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
                "rollback_apply_authorization_hash",
                record_sha_or_null(check.rollback_apply_authorization_hash),
            ),
            f(
                "disable_module_target_binding_hash",
                record_sha_or_null(check.disable_module_target_binding_hash),
            ),
            f(
                "restart_last_good_target_binding_hash",
                record_sha_or_null(check.restart_last_good_target_binding_hash),
            ),
            f(
                "load_artifact_by_hash_target_binding_hash",
                record_sha_or_null(check.load_artifact_by_hash_target_binding_hash),
            ),
            f(
                "recovery_memory_write_authority_hash",
                record_sha_or_null(check.recovery_memory_write_authority_hash),
            ),
            f(
                "durable_audit_rollback_write_authority_hash",
                record_sha_or_null(check.durable_audit_rollback_write_authority_hash),
            ),
            f(
                "service_inventory_side_effect_boundary_hash",
                record_sha_or_null(check.service_inventory_side_effect_boundary_hash),
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
                "command_dispatch_behavior_projection_hash",
                record_sha_or_null(check.command_dispatch_behavior_projection_hash),
            ),
            f(
                "command_dispatch_behavior_hash",
                record_sha_or_null(check.command_dispatch_behavior_hash),
            ),
            f(
                "expected_command_dispatch_behavior_hash",
                record_sha_or_null(check.expected_command_dispatch_behavior_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_retained_reference(
    check: &RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandDispatchBehaviorReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_recovery_lifeline_command_dispatch_behavior_reference",
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
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "latest_event_id",
                record_event_or_null(retained_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "latest_command_dispatch_behavior_id",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.command_dispatch_behavior_id),
                ),
            ),
            f(
                "latest_command_dispatch_behavior_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.command_dispatch_behavior_hash),
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

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_selftest_case(
    case: &RecoveryLifelineCommandDispatchBehaviorSelfTestCase,
    comma: bool,
) {
    emit_selftest_case(case, EFFECT_SELFTEST_FIELDS, comma);
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_reference_object(
    check: &RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
) {
    emit_record_property(
        "executor_capability_table_reference",
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
                "executor_capability_table_id",
                record_str_or_null(check.executor_capability_table_id),
            ),
            f(
                "retained_command_dispatch_behavior_event_id",
                record_str_or_null(check.retained_command_dispatch_behavior_event_id),
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
                "rollback_apply_authorization_hash",
                record_sha_or_null(check.rollback_apply_authorization_hash),
            ),
            f(
                "disable_module_target_binding_hash",
                record_sha_or_null(check.disable_module_target_binding_hash),
            ),
            f(
                "restart_last_good_target_binding_hash",
                record_sha_or_null(check.restart_last_good_target_binding_hash),
            ),
            f(
                "load_artifact_by_hash_target_binding_hash",
                record_sha_or_null(check.load_artifact_by_hash_target_binding_hash),
            ),
            f(
                "recovery_memory_write_authority_hash",
                record_sha_or_null(check.recovery_memory_write_authority_hash),
            ),
            f(
                "durable_audit_rollback_write_authority_hash",
                record_sha_or_null(check.durable_audit_rollback_write_authority_hash),
            ),
            f(
                "service_inventory_side_effect_boundary_hash",
                record_sha_or_null(check.service_inventory_side_effect_boundary_hash),
            ),
            f(
                "command_dispatch_behavior_hash",
                record_sha_or_null(check.command_dispatch_behavior_hash),
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
                "executor_capability_projection_hash",
                record_sha_or_null(check.executor_capability_projection_hash),
            ),
            f(
                "executor_capability_table_hash",
                record_sha_or_null(check.executor_capability_table_hash),
            ),
            f(
                "expected_executor_capability_table_hash",
                record_sha_or_null(check.expected_executor_capability_table_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_retained_reference(
    check: &RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutorCapabilityTableReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_recovery_lifeline_command_executor_capability_table_reference",
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
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "latest_event_id",
                record_event_or_null(retained_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "latest_executor_capability_table_id",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.executor_capability_table_id),
                ),
            ),
            f(
                "latest_executor_capability_table_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.executor_capability_table_hash),
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

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_selftest_case(
    case: &RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase,
    comma: bool,
) {
    emit_selftest_case(case, EFFECT_SELFTEST_FIELDS, comma);
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_reference_object(
    check: &RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
) {
    emit_record_property(
        "side_effect_gate_reference",
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
                "side_effect_gate_id",
                record_str_or_null(check.side_effect_gate_id),
            ),
            f(
                "retained_executor_capability_table_event_id",
                record_str_or_null(check.retained_executor_capability_table_event_id),
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
                "rollback_apply_authorization_hash",
                record_sha_or_null(check.rollback_apply_authorization_hash),
            ),
            f(
                "disable_module_target_binding_hash",
                record_sha_or_null(check.disable_module_target_binding_hash),
            ),
            f(
                "restart_last_good_target_binding_hash",
                record_sha_or_null(check.restart_last_good_target_binding_hash),
            ),
            f(
                "load_artifact_by_hash_target_binding_hash",
                record_sha_or_null(check.load_artifact_by_hash_target_binding_hash),
            ),
            f(
                "recovery_memory_write_authority_hash",
                record_sha_or_null(check.recovery_memory_write_authority_hash),
            ),
            f(
                "durable_audit_rollback_write_authority_hash",
                record_sha_or_null(check.durable_audit_rollback_write_authority_hash),
            ),
            f(
                "service_inventory_side_effect_boundary_hash",
                record_sha_or_null(check.service_inventory_side_effect_boundary_hash),
            ),
            f(
                "command_dispatch_behavior_hash",
                record_sha_or_null(check.command_dispatch_behavior_hash),
            ),
            f(
                "executor_capability_table_hash",
                record_sha_or_null(check.executor_capability_table_hash),
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
                "side_effect_projection_hash",
                record_sha_or_null(check.side_effect_projection_hash),
            ),
            f(
                "side_effect_gate_hash",
                record_sha_or_null(check.side_effect_gate_hash),
            ),
            f(
                "expected_side_effect_gate_hash",
                record_sha_or_null(check.expected_side_effect_gate_hash),
            ),
            f("valid_hash_reference", b(check.valid)),
            f("accepts_raw_command_body", no()),
            f("accepts_lifeline_command_body", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
    );
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_retained_reference(
    check: &RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandSideEffectGateReference,
    )>,
) {
    let retained_ref = retained.as_ref();

    emit_record_property(
        "retained_recovery_lifeline_command_side_effect_gate_reference",
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
            f("command_execution_enabled", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "latest_event_id",
                record_event_or_null(retained_ref.map(|(event_id, _)| *event_id)),
            ),
            f(
                "latest_side_effect_gate_id",
                record_str_or_null(
                    retained_ref.map(|(_, reference)| reference.side_effect_gate_id),
                ),
            ),
            f(
                "latest_side_effect_gate_hash",
                record_sha_or_null(
                    retained_ref.map(|(_, reference)| reference.side_effect_gate_hash),
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

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_selftest_case(
    case: &RecoveryLifelineCommandSideEffectGateSelfTestCase,
    comma: bool,
) {
    emit_selftest_case(case, EFFECT_SELFTEST_FIELDS, comma);
}

const EFFECT_SELFTEST_FIELDS: &[crate::agent_protocol_support::SelftestReportField] = &[
    False("accepts_raw_command_body"),
    False("accepts_lifeline_command_body"),
    False("accepts_lifeline_command_envelope"),
    False("dispatches_lifeline_command"),
    False("command_execution_enabled"),
    False("allocates_service_slot"),
    False("creates_service_inventory_records"),
    Str("service_inventory_change", "none"),
    False("load_attempted"),
];
