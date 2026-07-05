use alloc::vec;

use crate::{
    agent_protocol_recovery_lifeline_protocol_types::{
        RecoveryLifelineCommandVocabularyCheck, RecoveryLifelineCommandVocabularySelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_record_fields, emit_record_property_line, record_bool as b,
        record_false as no, record_field as f, record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_recovery_lifeline_command_vocabulary_object(
    check: &RecoveryLifelineCommandVocabularyCheck,
    comma: bool,
) {
    let commands = if check.command_vocabulary_exposed {
        V::Array(vec![
            command_definition(
                "recovery.lifeline.status",
                "raios.recovery_lifeline_command.status_args.v0",
                "cap.recovery.load_artifact.read",
                "observe",
                "read recovery lifeline status",
                check.reason,
            ),
            command_definition(
                "recovery.lifeline.rollback_preview",
                "raios.recovery_lifeline_command.rollback_preview_args.v0",
                "cap.recovery.rollback.read",
                "observe",
                "preview rollback target and required evidence",
                check.reason,
            ),
            command_definition(
                "recovery.lifeline.rollback_apply",
                "raios.recovery_lifeline_command.rollback_apply_args.v0",
                "cap.recovery.rollback",
                "persist",
                "apply a rollback transaction",
                check.reason,
            ),
            command_definition(
                "recovery.lifeline.disable_module",
                "raios.recovery_lifeline_command.disable_module_args.v0",
                "cap.recovery.module.disable",
                "persist",
                "disable a bad retained module id",
                check.reason,
            ),
            command_definition(
                "recovery.lifeline.restart_last_good",
                "raios.recovery_lifeline_command.restart_last_good_args.v0",
                "cap.recovery.service.restart",
                "recovery_modify_ram",
                "restart the last-good service set",
                check.reason,
            ),
            command_definition(
                "recovery.lifeline.load_artifact_by_hash",
                "raios.recovery_lifeline_command.load_artifact_by_hash_args.v0",
                "cap.recovery.load_artifact",
                "recovery_modify_ram",
                "load a recovery artifact by retained hash evidence",
                check.reason,
            ),
        ])
    } else {
        V::Array(vec![])
    };

    emit_record_property_line(
        "command_vocabulary",
        vec![
            f("schema", s("raios.recovery_lifeline_command_vocabulary.v0")),
            f("state", s("defined_non_executable")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("exposed", b(check.command_vocabulary_exposed)),
            f("authority", no()),
            f("command_execution_enabled", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("dispatches_commands", no()),
            f(
                "argument_envelope_schema",
                s("raios.recovery_lifeline_command_envelope.v0"),
            ),
            f("primary_denial_reason", s(check.reason)),
            f(
                "command_count",
                V::U64(if check.command_vocabulary_exposed {
                    6
                } else {
                    0
                }),
            ),
            f("commands", commands),
            f(
                "required_before_execution",
                V::Array(vec![
                    s("raios.recovery_lifeline_protocol_state.v0"),
                    s("raios.recovery_loader_runtime_isolation.v0"),
                    s("raios.recovery_rollback_transaction_engine.v0"),
                    s("raios.durable_audit_rollback_persistence.v0"),
                    s("raios.recovery_memory_provenance.v0"),
                ]),
            ),
        ],
        comma,
    );
}

fn command_definition(
    command_id: &'static str,
    args_schema: &'static str,
    required_capability: &'static str,
    risk: &'static str,
    summary: &'static str,
    denial_reason: &'static str,
) -> V<'static> {
    V::Object(vec![
        f("id", s(command_id)),
        f("argument_schema", s(args_schema)),
        f("required_capability", s(required_capability)),
        f("risk", s(risk)),
        f("summary", s(summary)),
        f("status", s("defined_non_executable")),
        f("accepts_envelope", no()),
        f("dispatches_command", no()),
        f("authorizes_recovery_load", no()),
        f("creates_durable_records", no()),
        f("installs_rollback_plan", no()),
        f("allocates_service_slot", no()),
        f("denial_reason", s(denial_reason)),
    ])
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary_check(
    check: &RecoveryLifelineCommandVocabularyCheck,
) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f("request_chain_valid", b(check.request_chain_valid)),
            f(
                "command_vocabulary_exposed",
                b(check.command_vocabulary_exposed),
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

pub(crate) fn emit_recovery_lifeline_command_vocabulary_selftest_case(
    case: &RecoveryLifelineCommandVocabularySelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("passed", b(case.passed)),
            f("command_execution_enabled", no()),
            f("accepts_lifeline_command_envelope", no()),
            f("authorizes_recovery_load", no()),
            f("can_move_beyond_denial", no()),
            f("loads_recovery_loader", no()),
            f("loads_recovery_artifact", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        comma,
    );
}
