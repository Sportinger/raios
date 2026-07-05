use alloc::vec;

use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryDurableAuditRollbackPersistenceCheck, RecoveryMemoryProvenanceCandidate,
        RecoveryMemoryProvenanceCheck, RecoveryMemoryProvenanceSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_inline_record_property_at, emit_record_fields,
        emit_record_property_line, record_bool as b, record_false as no, record_field as f,
        record_non_authorizing_fact_fields, record_null as null, record_object as object,
        record_str as s,
    },
};
use raios_core::record::Value as V;

#[rustfmt::skip]
const MEMORY_FACT_FALSES: &[&str] = &["authorizes_recovery_load", "memory_writes_enabled", "provider_export_enabled", "durable_writes_enabled", "rollback_replay_enabled", "recovery_memory_writes_enabled", "rollback_preview_enabled", "rollback_apply_enabled", "can_move_beyond_denial", "loads_recovery_loader", "loads_recovery_artifact", "creates_durable_records", "installs_rollback_plan", "allocates_service_slot"];

pub(crate) fn emit_recovery_memory_provenance_input_state(
    candidate: &RecoveryMemoryProvenanceCandidate,
    persistence_check: &RecoveryDurableAuditRollbackPersistenceCheck,
    check: &RecoveryMemoryProvenanceCheck,
    comma: bool,
) {
    emit_record_property_line(
        "memory_provenance_inputs",
        vec![f(
            "durable_audit_rollback_persistence",
            object(vec![
                f("schema", s("raios.durable_audit_rollback_persistence.v0")),
                f(
                    "state",
                    s(if candidate.durable_audit_rollback_persistence_available {
                        "defined_read_only_boundary"
                    } else {
                        "missing"
                    }),
                ),
                f("retention", s("current_boot_read_only_diagnostic")),
                f("event_id", null()),
                f(
                    "boundary_exposed",
                    b(check.durable_audit_rollback_persistence_boundary_exposed),
                ),
                f(
                    "accepted_for_memory_readiness",
                    b(check.durable_audit_rollback_persistence_accepted),
                ),
                f(
                    "durable_audit_rollback_persistence_ready",
                    b(persistence_check.durable_audit_rollback_persistence_ready),
                ),
                f(
                    "current_boot",
                    b(candidate.durable_audit_rollback_persistence_current_boot),
                ),
                f(
                    "schema_ok",
                    b(candidate.durable_audit_rollback_persistence_schema_ok),
                ),
                f(
                    "binding_ok",
                    b(candidate.durable_audit_rollback_persistence_binding_ok),
                ),
                f(
                    "reason",
                    s(candidate.durable_audit_rollback_persistence_binding_reason),
                ),
                f("writes_durable_audit_log", no()),
                f("writes_rollback_store", no()),
                f("replays_rollback_transactions", no()),
                f("recovery_memory_writes_enabled", no()),
            ]),
        )],
        comma,
    );
}

pub(crate) fn emit_recovery_memory_provenance_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(
        field,
        record_non_authorizing_fact_fields(schema, present, missing_reason, MEMORY_FACT_FALSES),
        8,
        comma,
    );
}

pub(crate) fn emit_recovery_memory_provenance_boundary(check: &RecoveryMemoryProvenanceCheck) {
    emit_record_fields(
        vec![
            f("schema", s("raios.recovery_memory_provenance.v0")),
            f("state", s("defined_non_executable")),
            f(
                "requirements_exposed",
                b(check.memory_provenance_requirements_exposed),
            ),
            f(
                "recovery_memory_provenance_ready",
                b(check.recovery_memory_provenance_ready),
            ),
            f("memory_writes_enabled", no()),
            f("provider_export_enabled", no()),
            f("durable_writes_enabled", no()),
            f("rollback_replay_enabled", no()),
            f("recovery_memory_writes_enabled", no()),
            f("rollback_preview_enabled", no()),
            f("rollback_apply_enabled", no()),
            f("accepts_memory_record_json", no()),
            f("accepts_provider_context_export", no()),
            f("accepts_rollback_transaction_envelope", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("exports_provider_context", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("loads_recovery_artifact", no()),
            f("service_inventory_change", s("none")),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f(
                "required_before_memory_write",
                V::Array(vec![
                    s("raios.recovery_lifeline_protocol_state.v0"),
                    s("raios.recovery_lifeline_command_vocabulary.v0"),
                    s("raios.recovery_loader_runtime_isolation.v0"),
                    s("raios.recovery_rollback_transaction_engine.v0"),
                    s("raios.durable_audit_rollback_persistence.v0"),
                    s("raios.recovery_memory_source_record_ids.v0"),
                    s("raios.recovery_memory_source_schema_hashes.v0"),
                    s("raios.recovery_memory_classification.v0"),
                    s("raios.recovery_memory_authority_level.v0"),
                    s("raios.recovery_memory_rollback_transaction_binding.v0"),
                    s("raios.recovery_memory_last_good_checkpoint_binding.v0"),
                    s("raios.recovery_memory_export_profile.v0"),
                    s("raios.recovery_memory_redaction_state.v0"),
                    s("raios.recovery_memory_replay_window.v0"),
                    s("raios.recovery_memory_audit_linkage.v0"),
                ]),
            ),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_memory_provenance_check(check: &RecoveryMemoryProvenanceCheck) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f("request_chain_valid", b(check.request_chain_valid)),
            f(
                "command_vocabulary_envelope_exposed",
                b(check.command_vocabulary_envelope_exposed),
            ),
            f(
                "command_vocabulary_accepted",
                b(check.command_vocabulary_accepted),
            ),
            f(
                "loader_runtime_isolation_boundary_exposed",
                b(check.loader_runtime_isolation_boundary_exposed),
            ),
            f(
                "loader_runtime_isolation_accepted",
                b(check.loader_runtime_isolation_accepted),
            ),
            f(
                "rollback_transaction_engine_boundary_exposed",
                b(check.rollback_transaction_engine_boundary_exposed),
            ),
            f(
                "rollback_transaction_engine_accepted",
                b(check.rollback_transaction_engine_accepted),
            ),
            f(
                "durable_audit_rollback_persistence_boundary_exposed",
                b(check.durable_audit_rollback_persistence_boundary_exposed),
            ),
            f(
                "durable_audit_rollback_persistence_accepted",
                b(check.durable_audit_rollback_persistence_accepted),
            ),
            f(
                "memory_provenance_requirements_exposed",
                b(check.memory_provenance_requirements_exposed),
            ),
            f(
                "recovery_memory_provenance_ready",
                b(check.recovery_memory_provenance_ready),
            ),
            f("memory_writes_enabled", b(check.memory_writes_enabled)),
            f("provider_export_enabled", b(check.provider_export_enabled)),
            f("durable_writes_enabled", b(check.durable_writes_enabled)),
            f("rollback_replay_enabled", b(check.rollback_replay_enabled)),
            f(
                "recovery_memory_writes_enabled",
                b(check.recovery_memory_writes_enabled),
            ),
            f(
                "rollback_preview_enabled",
                b(check.rollback_preview_enabled),
            ),
            f("rollback_apply_enabled", b(check.rollback_apply_enabled)),
            f(
                "command_execution_enabled",
                b(check.command_execution_enabled),
            ),
            f(
                "accepts_lifeline_command_envelope",
                b(check.accepts_lifeline_command_envelope),
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
            f("service_slot_allocation_attempted", no()),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_memory_provenance_selftest_case(
    case: &RecoveryMemoryProvenanceSelfTestCase,
    comma: bool,
) {
    #[rustfmt::skip]
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("memory_writes_enabled", no()), f("provider_export_enabled", no()), f("durable_writes_enabled", no()), f("rollback_replay_enabled", no()), f("recovery_memory_writes_enabled", no()), f("rollback_preview_enabled", no()), f("rollback_apply_enabled", no()), f("command_execution_enabled", no()), f("accepts_lifeline_command_envelope", no()), f("authorizes_recovery_load", no()), f("can_move_beyond_denial", no()), f("loads_recovery_loader", no()), f("loads_recovery_artifact", no()), f("creates_durable_records", no()), f("installs_rollback_plan", no()), f("allocates_service_slot", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}
