use alloc::vec;

use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryLoaderRuntimeIsolationCandidate, RecoveryLoaderRuntimeIsolationCheck,
        RecoveryLoaderRuntimeIsolationSelfTestCase,
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
const LOADER_FACT_FALSES: &[&str] = &["authorizes_recovery_load", "can_move_beyond_denial", "loads_recovery_loader", "loads_recovery_artifact", "creates_durable_records", "installs_rollback_plan", "allocates_service_slot"];

pub(crate) fn emit_recovery_loader_runtime_isolation_input_state(
    candidate: &RecoveryLoaderRuntimeIsolationCandidate,
    check: &RecoveryLoaderRuntimeIsolationCheck,
    comma: bool,
) {
    emit_record_property_line(
        "loader_isolation_inputs",
        vec![
            f(
                "lifeline_protocol_state",
                object(vec![
                    f("schema", s("raios.recovery_lifeline_protocol_state.v0")),
                    f(
                        "state",
                        s(if candidate.command_candidate.protocol_state_retained {
                            "present"
                        } else {
                            "missing"
                        }),
                    ),
                    f("retention", s("current_boot_ram_event_log")),
                    f("event_id", null()),
                    f(
                        "current_boot",
                        b(candidate.command_candidate.protocol_state_current_boot),
                    ),
                    f(
                        "schema_ok",
                        b(candidate.command_candidate.protocol_state_schema_ok),
                    ),
                    f(
                        "binding_ok",
                        b(candidate.command_candidate.protocol_state_binding_ok),
                    ),
                    f(
                        "reason",
                        s(candidate.command_candidate.protocol_state_binding_reason),
                    ),
                    f("authorizes_recovery_load", no()),
                ]),
            ),
            f(
                "lifeline_command_vocabulary",
                object(vec![
                    f("schema", s("raios.recovery_lifeline_command_vocabulary.v0")),
                    f(
                        "state",
                        s(if candidate.command_vocabulary_available {
                            "defined_read_only_envelope"
                        } else {
                            "missing"
                        }),
                    ),
                    f("retention", s("current_boot_read_only_diagnostic")),
                    f("event_id", null()),
                    f(
                        "envelope_exposed",
                        b(check.command_vocabulary_envelope_exposed),
                    ),
                    f(
                        "accepted_for_loader_readiness",
                        b(check.command_vocabulary_accepted),
                    ),
                    f("current_boot", b(candidate.command_vocabulary_current_boot)),
                    f("schema_ok", b(candidate.command_vocabulary_schema_ok)),
                    f("binding_ok", b(candidate.command_vocabulary_binding_ok)),
                    f("reason", s(candidate.command_vocabulary_binding_reason)),
                    f("accepts_lifeline_command_envelope", no()),
                    f("dispatches_commands", no()),
                    f("authorizes_recovery_load", no()),
                ]),
            ),
        ],
        comma,
    );
}

pub(crate) fn emit_recovery_loader_runtime_isolation_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(
        field,
        record_non_authorizing_fact_fields(schema, present, missing_reason, LOADER_FACT_FALSES),
        8,
        comma,
    );
}

pub(crate) fn emit_recovery_loader_runtime_isolation_boundary(
    check: &RecoveryLoaderRuntimeIsolationCheck,
) {
    emit_record_fields(
        vec![
            f("schema", s("raios.recovery_loader_runtime_isolation.v0")),
            f("state", s("defined_non_executable")),
            f(
                "requirements_exposed",
                b(check.isolation_requirements_exposed),
            ),
            f(
                "loader_runtime_isolation_ready",
                b(check.loader_runtime_isolation_ready),
            ),
            f("loader_execution_enabled", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("loads_recovery_loader", no()),
            f("loads_recovery_artifact", no()),
            f("normal_module_load_path_used", no()),
            f("recovery_lifeline_command_dispatch_enabled", no()),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f(
                "required_before_execution",
                V::Array(vec![
                    s("raios.recovery_lifeline_protocol_state.v0"),
                    s("raios.recovery_lifeline_command_vocabulary.v0"),
                    s("raios.recovery_loader_address_space_boundary.v0"),
                    s("raios.recovery_loader_entrypoint_abi.v0"),
                    s("raios.recovery_loader_memory_map_constraints.v0"),
                    s("raios.recovery_loader_capability_import_table.v0"),
                    s("raios.recovery_loader_artifact_hash_binding.v0"),
                    s("raios.recovery_loader_provider_separation.v0"),
                    s("raios.recovery_loader_normal_module_separation.v0"),
                    s("raios.recovery_rollback_transaction_engine.v0"),
                    s("raios.durable_audit_rollback_persistence.v0"),
                    s("raios.recovery_memory_provenance.v0"),
                ]),
            ),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_loader_runtime_isolation_check(
    check: &RecoveryLoaderRuntimeIsolationCheck,
) {
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
                "isolation_requirements_exposed",
                b(check.isolation_requirements_exposed),
            ),
            f(
                "loader_runtime_isolation_ready",
                b(check.loader_runtime_isolation_ready),
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
            f("service_slot_allocation_attempted", no()),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

pub(crate) fn emit_recovery_loader_runtime_isolation_selftest_case(
    case: &RecoveryLoaderRuntimeIsolationSelfTestCase,
    comma: bool,
) {
    #[rustfmt::skip]
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("loader_execution_enabled", no()), f("command_execution_enabled", no()), f("accepts_lifeline_command_envelope", no()), f("authorizes_recovery_load", no()), f("can_move_beyond_denial", no()), f("loads_recovery_loader", no()), f("loads_recovery_artifact", no()), f("creates_durable_records", no()), f("installs_rollback_plan", no()), f("allocates_service_slot", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}
