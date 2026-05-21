use crate::{
    agent_protocol_memory::memory_mutation_method,
    agent_protocol_provider::provider_context_export_method,
    agent_protocol_recovery::recovery_artifact_load_method,
    agent_protocol_support::{json_event_id, json_str, method_eq, raw, raw_line},
    agent_protocol_system::DENIED_METHODS,
    event_log, serial,
};
pub(crate) fn record_read(method: &'static str) -> event_log::EventId {
    event_log::record_agent_read(method, requested_capability_for_read(method))
}

pub(crate) fn record_denial(method: &'static str) -> event_log::EventId {
    event_log::record_capability_denied(
        method,
        requested_capability_for_denial(method),
        risk_for_denial(method),
    )
}

pub(crate) fn emit_capability_denied(method: &'static str, event_id: event_log::EventId) {
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"message\": ");
    json_str("mutating agent methods are denied until manifest, VM test report, local attestation, policy grant, approval, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"raios.module_manifest.v0\",");
    raw_line("      \"raios.vm_test_report.v0\",");
    raw_line("      \"local_attestation.v0\",");
    raw_line("      \"computed_capability_grant\",");
    raw_line("      \"local_approval\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ]");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn denied_method(method: &str) -> bool {
    let mut idx = 0usize;
    while idx < DENIED_METHODS.len() {
        if method_eq(method, DENIED_METHODS[idx]) {
            return true;
        }
        idx += 1;
    }
    false
}

pub(crate) fn canonical_denied_method(method: &str) -> &'static str {
    let mut idx = 0usize;
    while idx < DENIED_METHODS.len() {
        if method_eq(method, DENIED_METHODS[idx]) {
            return DENIED_METHODS[idx];
        }
        idx += 1;
    }
    "unknown"
}

pub(crate) fn canonical_module_load_ephemeral_method(method: &str) -> &'static str {
    if method_eq(method, "service.load_ephemeral") {
        "service.load_ephemeral"
    } else {
        "module.load_ephemeral"
    }
}

fn requested_capability_for_read(method: &str) -> &'static str {
    if method_eq(method, "system.describe") {
        "cap.system.describe.read"
    } else if method_eq(method, "system.snapshot") {
        "cap.system.snapshot.read"
    } else if method_eq(method, "system.capabilities") {
        "cap.system.capabilities.read"
    } else if method_eq(method, "system.boot_log") {
        "cap.system.boot_log.read"
    } else if method_eq(method, "device.graph") {
        "cap.device.graph.read"
    } else if method_eq(method, "problem.list") {
        "cap.problem.list.read"
    } else if method_eq(method, "service.inventory") {
        "cap.service.inventory.read"
    } else if method_eq(method, "memory.profile") {
        "cap.memory.profile.read"
    } else if method_eq(method, "memory.context") {
        "cap.memory.context.read"
    } else if method_eq(method, "memory.query") {
        "cap.memory.query.read"
    } else if method_eq(method, "memory.trace") {
        "cap.memory.trace.read"
    } else if method_eq(method, "memory.recent_events") {
        "cap.memory.recent_events.read"
    } else if method_eq(method, "audit.events") {
        "cap.audit.events.read"
    } else if method_eq(method, "provider.context_gate")
        || method_eq(method, "provider.context_gate_selftest")
        || method_eq(method, "provider.context_injection_gate")
        || method_eq(method, "provider.context_injection_gate_selftest")
    {
        if method_eq(method, "provider.context_injection_gate")
            || method_eq(method, "provider.context_injection_gate_selftest")
        {
            "cap.provider.context_injection.read"
        } else {
            "cap.provider.context_export.read"
        }
    } else if method_eq(method, "recovery.load_binding")
        || method_eq(method, "recovery.load_binding_selftest")
        || method_eq(method, "recovery.identity_diagnostic")
        || method_eq(method, "recovery.identity_diagnostic_selftest")
        || method_eq(method, "recovery.trust_diagnostic")
        || method_eq(method, "recovery.trust_diagnostic_selftest")
        || method_eq(method, "recovery.vm_test_diagnostic")
        || method_eq(method, "recovery.vm_test_diagnostic_selftest")
        || method_eq(method, "recovery.local_approval_diagnostic")
        || method_eq(method, "recovery.local_approval_diagnostic_selftest")
    {
        "cap.recovery.load_artifact.read"
    } else if method_eq(method, "module.manifest_diagnostic")
        || method_eq(method, "module.manifest_diagnostic_selftest")
        || method_eq(method, "module.artifact_diagnostic")
        || method_eq(method, "module.artifact_diagnostic_selftest")
        || method_eq(method, "module.vm_report_diagnostic")
        || method_eq(method, "module.vm_report_diagnostic_selftest")
        || method_eq(method, "module.attestation_diagnostic")
        || method_eq(method, "module.attestation_diagnostic_selftest")
        || method_eq(method, "module.approval_diagnostic")
        || method_eq(method, "module.approval_diagnostic_selftest")
        || method_eq(method, "module.grant_diagnostic")
        || method_eq(method, "module.grant_diagnostic_selftest")
        || method_eq(method, "module.audit_rollback_diagnostic")
        || method_eq(method, "module.audit_rollback_diagnostic_selftest")
        || method_eq(method, "module.service_slot_diagnostic")
        || method_eq(method, "module.service_slot_diagnostic_selftest")
        || method_eq(method, "module.audit_rollback_availability")
        || method_eq(method, "module.audit_rollback_availability_selftest")
        || method_eq(method, "module.audit_rollback_write_policy")
        || method_eq(method, "module.audit_rollback_write_policy_selftest")
        || method_eq(method, "module.audit_rollback_storage_layout")
        || method_eq(method, "module.audit_rollback_storage_layout_selftest")
        || method_eq(method, "module.audit_rollback_append_engine")
        || method_eq(method, "module.audit_rollback_append_engine_selftest")
        || method_eq(method, "module.audit_rollback_append_contract")
        || method_eq(method, "module.audit_rollback_append_contract_selftest")
        || method_eq(method, "module.audit_rollback_append_payload_hash")
        || method_eq(method, "module.audit_rollback_append_payload_hash_selftest")
        || method_eq(method, "module.audit_rollback_append_intent")
        || method_eq(method, "module.audit_rollback_append_intent_selftest")
        || method_eq(method, "module.audit_rollback_write_boundary")
        || method_eq(method, "module.audit_rollback_write_boundary_selftest")
        || method_eq(method, "module.load_gate_manifest_selftest")
        || method_eq(method, "module.load_gate_artifact_selftest")
        || method_eq(method, "module.load_gate_vm_report_selftest")
        || method_eq(method, "module.load_gate_attestation_selftest")
        || method_eq(method, "module.load_gate_approval_selftest")
        || method_eq(method, "module.load_gate_retained_selftest")
        || method_eq(method, "module.load_gate_audit_rollback_selftest")
        || method_eq(method, "module.load_gate_service_slot_selftest")
    {
        "cap.module.grant_diagnostic.read"
    } else {
        "cap.system.describe.read"
    }
}

fn requested_capability_for_denial(method: &str) -> &'static str {
    if memory_mutation_method(method) {
        "cap.memory.mutate"
    } else if provider_context_export_method(method) {
        "cap.provider.context_export"
    } else if method_eq(method, "module.propose")
        || method_eq(method, "module.build_result")
        || method_eq(method, "module.test_request")
        || method_eq(method, "module.test_result")
    {
        "cap.module.propose"
    } else if method_eq(method, "module.load_ephemeral")
        || method_eq(method, "service.load_ephemeral")
    {
        "cap.module.load_ephemeral"
    } else if recovery_artifact_load_method(method) {
        "cap.recovery.load_artifact"
    } else if method_eq(method, "module.persist") {
        "cap.module.persist"
    } else if method_eq(method, "module.rollback") {
        "cap.module.rollback"
    } else if method_eq(method, "config.apply")
        || method_eq(method, "apply_config")
        || method_eq(method, "provider.configure")
        || method_eq(method, "wifi.configure")
    {
        "cap.config.apply"
    } else {
        "capability_denied.for_all_mutating_methods"
    }
}

fn risk_for_denial(method: &str) -> &'static str {
    if provider_context_export_method(method) {
        "export"
    } else if recovery_artifact_load_method(method) {
        "recovery_modify_ram"
    } else if method_eq(method, "module.persist")
        || method_eq(method, "module.rollback")
        || method_eq(method, "config.apply")
        || method_eq(method, "apply_config")
        || method_eq(method, "provider.configure")
        || method_eq(method, "wifi.configure")
        || memory_mutation_method(method)
    {
        "persist"
    } else {
        "modify_ram"
    }
}

pub(crate) fn module_load_ephemeral_method(method: &str) -> bool {
    method_eq(method, "module.load_ephemeral") || method_eq(method, "service.load_ephemeral")
}
