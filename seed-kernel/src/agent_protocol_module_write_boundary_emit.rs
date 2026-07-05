use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        emit_record_property_line, record_bool as b, record_false as no, record_field as f,
        record_null as null, record_object as object, record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_module_availability_facts(
    availability: ModuleAuditRollbackAvailabilityCandidate,
    evaluation: ModuleAuditRollbackAvailabilityEvaluation,
) {
    emit_record_property_line(
        "availability_facts",
        vec![
            f(
                "durable_audit_ledger",
                module_availability_fact_record(
                    "raios.durable_audit_ledger.v0",
                    "availability.durable_audit_ledger.current_boot",
                    availability.durable_audit_ledger,
                    evaluation.durable_audit_ledger_status,
                    evaluation.durable_audit_ledger_reason,
                ),
            ),
            f(
                "rollback_store",
                module_availability_fact_record(
                    "raios.rollback_store.v0",
                    "availability.rollback_store.current_boot",
                    availability.rollback_store,
                    evaluation.rollback_store_status,
                    evaluation.rollback_store_reason,
                ),
            ),
        ],
        false,
    );
}

fn module_availability_fact_record(
    schema: &'static str,
    id: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    object(vec![
        f("schema", s(schema)),
        f("id", s(id)),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("present", b(fact.present)),
        f("schema_valid", b(fact.schema_ok)),
        f("provenance_valid", b(fact.provenance_ok)),
        f("authority", s("current_snapshot")),
        f("persistence", s("none")),
        f("durable", no()),
        f("write_attempted", no()),
        f("install_attempted", no()),
        f(
            "provenance",
            object(vec![
                f("source_method", s("module.audit_rollback_availability")),
                f("source_transport", s("serial-console")),
                f("event_scope", s("current_boot")),
                f("record_id", null()),
            ]),
        ),
    ])
}
