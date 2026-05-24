use crate::{agent_protocol_module_types::*, agent_protocol_support::*, event_log};

pub(crate) fn emit_module_write_boundary_input_ref(
    name: &'static str,
    event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    schema: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"event_id\": ");
    json_event_id_option(event_id);
    raw(", \"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_availability_facts(
    availability: ModuleAuditRollbackAvailabilityCandidate,
    evaluation: ModuleAuditRollbackAvailabilityEvaluation,
) {
    raw_line("      \"availability_facts\": {");
    emit_module_availability_fact(
        "durable_audit_ledger",
        "raios.durable_audit_ledger.v0",
        "availability.durable_audit_ledger.current_boot",
        availability.durable_audit_ledger,
        evaluation.durable_audit_ledger_status,
        evaluation.durable_audit_ledger_reason,
        true,
    );
    emit_module_availability_fact(
        "rollback_store",
        "raios.rollback_store.v0",
        "availability.rollback_store.current_boot",
        availability.rollback_store,
        evaluation.rollback_store_status,
        evaluation.rollback_store_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_availability_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"install_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_availability\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}
