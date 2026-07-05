use alloc::vec;

use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryLifelineCommandAdmissionCandidate, RecoveryLifelineCommandAdmissionCheck,
        RecoveryLifelineCommandAdmissionSelfTestCase, RecoveryMemoryProvenanceCheck,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_fields, emit_record_object,
        emit_record_property_line, record_bool as b, record_false as no, record_field as f,
        record_null as null, record_object as object, record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_recovery_lifeline_command_admission_input_state(
    candidate: &RecoveryLifelineCommandAdmissionCandidate,
    memory_check: &RecoveryMemoryProvenanceCheck,
    check: &RecoveryLifelineCommandAdmissionCheck,
    comma: bool,
) {
    emit_record_property_line(
        "command_admission_inputs",
        vec![f(
            "recovery_memory_provenance",
            object(vec![
                f("schema", s("raios.recovery_memory_provenance.v0")),
                f(
                    "state",
                    s(if candidate.recovery_memory_provenance_available {
                        "defined_read_only_boundary"
                    } else {
                        "missing"
                    }),
                ),
                f("retention", s("current_boot_read_only_diagnostic")),
                f("event_id", null()),
                f(
                    "boundary_exposed",
                    b(check.recovery_memory_provenance_boundary_exposed),
                ),
                f(
                    "accepted_for_command_admission",
                    b(check.recovery_memory_provenance_accepted),
                ),
                f(
                    "recovery_memory_provenance_ready",
                    b(memory_check.recovery_memory_provenance_ready),
                ),
                f(
                    "current_boot",
                    b(candidate.recovery_memory_provenance_current_boot),
                ),
                f(
                    "schema_ok",
                    b(candidate.recovery_memory_provenance_schema_ok),
                ),
                f(
                    "binding_ok",
                    b(candidate.recovery_memory_provenance_binding_ok),
                ),
                f(
                    "reason",
                    s(candidate.recovery_memory_provenance_binding_reason),
                ),
                f("writes_recovery_memory", no()),
                f("exports_provider_context", no()),
                f("accepts_lifeline_command_envelope", no()),
            ]),
        )],
        comma,
    );
}

pub(crate) fn emit_recovery_lifeline_command_admission_requirement(
    command_id: &'static str,
    args_schema: &'static str,
    required_capability: &'static str,
    readiness_schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    check: &RecoveryLifelineCommandAdmissionCheck,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("command_id", s(command_id)),
            f("argument_schema", s(args_schema)),
            f("required_capability", s(required_capability)),
            f("admission_schema", s(readiness_schema)),
            f("status", s(if present { "present" } else { "missing" })),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("required", b(true)),
            f("event_id", null()),
            f(
                "reason",
                s(if present {
                    "command_admission_requirement_defined"
                } else {
                    missing_reason
                }),
            ),
            f(
                "admission_ready",
                b(check.command_admission_ready && present),
            ),
            f("accepts_envelope", no()),
            f("dispatches_command", no()),
            f("command_execution_enabled", no()),
            f("rollback_preview_enabled", no()),
            f("rollback_apply_enabled", no()),
            f("memory_writes_enabled", no()),
            f("provider_export_enabled", no()),
            f("durable_writes_enabled", no()),
            f("loads_recovery_artifact", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

pub(crate) fn emit_recovery_lifeline_command_admission_boundary(
    check: &RecoveryLifelineCommandAdmissionCheck,
) {
    emit_record_fields(
        vec![
            f("schema", s("raios.recovery_lifeline_command_admission.v0")),
            f("state", s("defined_non_executable")),
            f(
                "requirements_exposed",
                b(check.command_admission_requirements_exposed),
            ),
            f("command_admission_ready", b(check.command_admission_ready)),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_lifeline_command", no()),
            f("command_execution_enabled", no()),
            f("rollback_preview_enabled", no()),
            f("rollback_apply_enabled", no()),
            f("memory_writes_enabled", no()),
            f("provider_export_enabled", no()),
            f("durable_writes_enabled", no()),
            f("rollback_replay_enabled", no()),
            f("recovery_memory_writes_enabled", no()),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("loads_recovery_loader", no()),
            f("loads_recovery_artifact", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f(
                "required_before_admission",
                V::Array(vec![
                    s("raios.recovery_lifeline_request.v0"),
                    s("raios.recovery_lifeline_protocol_state.v0"),
                    s("raios.recovery_lifeline_command_vocabulary.v0"),
                    s("raios.recovery_loader_runtime_isolation.v0"),
                    s("raios.recovery_rollback_transaction_engine.v0"),
                    s("raios.durable_audit_rollback_persistence.v0"),
                    s("raios.recovery_memory_provenance.v0"),
                ]),
            ),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_lifeline_command_admission_check(
    check: &RecoveryLifelineCommandAdmissionCheck,
) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f(
                "request_chain_valid",
                b(check.memory_check.request_chain_valid),
            ),
            f(
                "command_vocabulary_envelope_exposed",
                b(check.memory_check.command_vocabulary_envelope_exposed),
            ),
            f(
                "command_vocabulary_accepted",
                b(check.memory_check.command_vocabulary_accepted),
            ),
            f(
                "loader_runtime_isolation_boundary_exposed",
                b(check.memory_check.loader_runtime_isolation_boundary_exposed),
            ),
            f(
                "loader_runtime_isolation_accepted",
                b(check.memory_check.loader_runtime_isolation_accepted),
            ),
            f(
                "rollback_transaction_engine_boundary_exposed",
                b(check
                    .memory_check
                    .rollback_transaction_engine_boundary_exposed),
            ),
            f(
                "rollback_transaction_engine_accepted",
                b(check.memory_check.rollback_transaction_engine_accepted),
            ),
            f(
                "durable_audit_rollback_persistence_boundary_exposed",
                b(check
                    .memory_check
                    .durable_audit_rollback_persistence_boundary_exposed),
            ),
            f(
                "durable_audit_rollback_persistence_accepted",
                b(check
                    .memory_check
                    .durable_audit_rollback_persistence_accepted),
            ),
            f(
                "recovery_memory_provenance_boundary_exposed",
                b(check.recovery_memory_provenance_boundary_exposed),
            ),
            f(
                "recovery_memory_provenance_accepted",
                b(check.recovery_memory_provenance_accepted),
            ),
            f(
                "command_admission_requirements_exposed",
                b(check.command_admission_requirements_exposed),
            ),
            f("command_admission_ready", b(check.command_admission_ready)),
            f(
                "lifeline_status_admission_present",
                b(check.lifeline_status_admission_present),
            ),
            f(
                "rollback_preview_admission_present",
                b(check.rollback_preview_admission_present),
            ),
            f(
                "rollback_apply_admission_present",
                b(check.rollback_apply_admission_present),
            ),
            f(
                "disable_module_admission_present",
                b(check.disable_module_admission_present),
            ),
            f(
                "restart_last_good_admission_present",
                b(check.restart_last_good_admission_present),
            ),
            f(
                "load_recovery_artifact_by_hash_admission_present",
                b(check.load_recovery_artifact_by_hash_admission_present),
            ),
            f(
                "command_execution_enabled",
                b(check.command_execution_enabled),
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
            f("durable_audit_write_attempted", no()),
            f("rollback_install_attempted", no()),
            f("rollback_replay_attempted", no()),
            f("recovery_memory_write_attempted", no()),
            f("provider_export_attempted", no()),
            f("lifeline_command_dispatch_attempted", no()),
            f("service_slot_allocation_attempted", no()),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_lifeline_command_admission_selftest_case(case: &RecoveryLifelineCommandAdmissionSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("command_execution_enabled", no()), f("accepts_lifeline_command_envelope", no()), f("dispatches_lifeline_command", no()), f("memory_writes_enabled", no()), f("provider_export_enabled", no()), f("durable_writes_enabled", no()), f("rollback_replay_enabled", no()), f("recovery_memory_writes_enabled", no()), f("rollback_preview_enabled", no()), f("rollback_apply_enabled", no()), f("authorizes_recovery_load", no()), f("can_move_beyond_denial", no()), f("loads_recovery_loader", no()), f("loads_recovery_artifact", no()), f("creates_durable_records", no()), f("installs_rollback_plan", no()), f("allocates_service_slot", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}
