use crate::{
    agent_protocol_memory::{
        canonical_memory_mutation_method, emit_memory_capability_denied, emit_memory_context,
        emit_memory_profile, emit_memory_query, emit_memory_trace, emit_recent_events,
        memory_mutation_method,
    },
    agent_protocol_module_audit::{
        emit_module_audit_rollback_diagnostic, emit_module_audit_rollback_diagnostic_selftest,
        module_audit_rollback_diagnostic_method, module_audit_rollback_diagnostic_selftest_method,
    },
    agent_protocol_module_grant::{
        emit_module_grant_diagnostic, emit_module_grant_diagnostic_selftest,
        module_grant_diagnostic_method, module_grant_diagnostic_selftest_method,
    },
    agent_protocol_module_load_gate::{
        emit_module_load_ephemeral_denied, emit_module_load_gate_artifact_selftest,
        emit_module_load_gate_audit_rollback_selftest, emit_module_load_gate_manifest_selftest,
        emit_module_load_gate_retained_selftest, emit_module_load_gate_service_slot_selftest,
        emit_module_load_gate_vm_report_selftest, module_load_gate_artifact_selftest_method,
        module_load_gate_audit_rollback_selftest_method, module_load_gate_manifest_selftest_method,
        module_load_gate_retained_selftest_method, module_load_gate_service_slot_selftest_method,
        module_load_gate_vm_report_selftest_method,
    },
    agent_protocol_module_reference::{
        emit_module_artifact_diagnostic, emit_module_artifact_diagnostic_selftest,
        emit_module_manifest_diagnostic, emit_module_manifest_diagnostic_selftest,
        emit_module_vm_report_diagnostic, emit_module_vm_report_diagnostic_selftest,
        module_artifact_diagnostic_method, module_artifact_diagnostic_selftest_method,
        module_manifest_diagnostic_method, module_manifest_diagnostic_selftest_method,
        module_vm_report_diagnostic_method, module_vm_report_diagnostic_selftest_method,
    },
    agent_protocol_module_service_slot::{
        emit_module_service_slot_diagnostic, emit_module_service_slot_diagnostic_selftest,
        module_service_slot_diagnostic_method, module_service_slot_diagnostic_selftest_method,
    },
    agent_protocol_provider::{
        emit_provider_context_export_denied, emit_provider_context_gate,
        emit_provider_context_gate_selftest, emit_provider_context_injection_gate,
        emit_provider_context_injection_gate_selftest, provider_context_export_method,
        provider_context_gate_method, provider_context_gate_selftest_method,
        provider_context_injection_gate_method, provider_context_injection_gate_selftest_method,
    },
    agent_protocol_support::{json_event_id, json_str, method_eq, method_head_eq, raw, raw_line},
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_describe, emit_device_graph, emit_problem_list,
        emit_service_inventory, emit_snapshot, DENIED_METHODS,
    },
    event_log, serial, ui,
};

pub(crate) use crate::agent_protocol_provider::provider_minimal_context_evidence_for_runtime;

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

pub fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> DispatchOutcome {
    let method = method.trim();
    if method.is_empty() {
        return DispatchOutcome::Unknown;
    }

    if method_eq(method, "system.describe") || method_eq(method, "describe") {
        record_read("system.describe");
        emit_describe();
        return DispatchOutcome::Response("system.describe");
    }
    if method_eq(method, "system.snapshot") || method_eq(method, "snapshot") {
        record_read("system.snapshot");
        emit_snapshot(runtime);
        return DispatchOutcome::Response("system.snapshot");
    }
    if method_eq(method, "system.capabilities")
        || method_eq(method, "capabilities")
        || method_eq(method, "caps")
    {
        record_read("system.capabilities");
        emit_capabilities();
        return DispatchOutcome::Response("system.capabilities");
    }
    if method_eq(method, "system.boot_log")
        || method_eq(method, "system.bootlog")
        || method_eq(method, "bootlog")
    {
        record_read("system.boot_log");
        emit_boot_log();
        return DispatchOutcome::Response("system.boot_log");
    }
    if method_eq(method, "device.graph") || method_eq(method, "devicegraph") {
        record_read("device.graph");
        emit_device_graph(runtime);
        return DispatchOutcome::Response("device.graph");
    }
    if method_eq(method, "problem.list") || method_eq(method, "problems") {
        record_read("problem.list");
        emit_problem_list(runtime);
        return DispatchOutcome::Response("problem.list");
    }
    if method_eq(method, "service.inventory") || method_eq(method, "services") {
        record_read("service.inventory");
        emit_service_inventory(runtime);
        return DispatchOutcome::Response("service.inventory");
    }
    if method_eq(method, "memory.profile") || method_eq(method, "memprofile") {
        record_read("memory.profile");
        emit_memory_profile();
        return DispatchOutcome::Response("memory.profile");
    }
    if method_head_eq(method, "memory.context") || method_head_eq(method, "memctx") {
        let event_id = record_read("memory.context");
        emit_memory_context(runtime, method, event_id);
        return DispatchOutcome::Response("memory.context");
    }
    if method_head_eq(method, "memory.query") || method_head_eq(method, "memquery") {
        record_read("memory.query");
        emit_memory_query();
        return DispatchOutcome::Response("memory.query");
    }
    if method_head_eq(method, "memory.trace") || method_head_eq(method, "memtrace") {
        record_read("memory.trace");
        emit_memory_trace(method);
        return DispatchOutcome::Response("memory.trace");
    }
    if method_head_eq(method, "memory.recent_events")
        || method_head_eq(method, "audit.events")
        || method_head_eq(method, "events")
    {
        record_read("memory.recent_events");
        emit_recent_events(method);
        return DispatchOutcome::Response("memory.recent_events");
    }
    if provider_context_gate_method(method) {
        record_read("provider.context_gate");
        emit_provider_context_gate(runtime, method);
        return DispatchOutcome::Response("provider.context_gate");
    }
    if provider_context_gate_selftest_method(method) {
        record_read("provider.context_gate_selftest");
        emit_provider_context_gate_selftest(runtime, method);
        return DispatchOutcome::Response("provider.context_gate_selftest");
    }
    if provider_context_injection_gate_method(method) {
        record_read("provider.context_injection_gate");
        emit_provider_context_injection_gate(runtime, method);
        return DispatchOutcome::Response("provider.context_injection_gate");
    }
    if provider_context_injection_gate_selftest_method(method) {
        record_read("provider.context_injection_gate_selftest");
        emit_provider_context_injection_gate_selftest(runtime, method);
        return DispatchOutcome::Response("provider.context_injection_gate_selftest");
    }
    if module_manifest_diagnostic_method(method) {
        record_read("module.manifest_diagnostic");
        emit_module_manifest_diagnostic(method);
        return DispatchOutcome::Response("module.manifest_diagnostic");
    }
    if module_manifest_diagnostic_selftest_method(method) {
        record_read("module.manifest_diagnostic_selftest");
        emit_module_manifest_diagnostic_selftest();
        return DispatchOutcome::Response("module.manifest_diagnostic_selftest");
    }
    if module_artifact_diagnostic_method(method) {
        record_read("module.artifact_diagnostic");
        emit_module_artifact_diagnostic(method);
        return DispatchOutcome::Response("module.artifact_diagnostic");
    }
    if module_artifact_diagnostic_selftest_method(method) {
        record_read("module.artifact_diagnostic_selftest");
        emit_module_artifact_diagnostic_selftest();
        return DispatchOutcome::Response("module.artifact_diagnostic_selftest");
    }
    if module_vm_report_diagnostic_method(method) {
        record_read("module.vm_report_diagnostic");
        emit_module_vm_report_diagnostic(method);
        return DispatchOutcome::Response("module.vm_report_diagnostic");
    }
    if module_vm_report_diagnostic_selftest_method(method) {
        record_read("module.vm_report_diagnostic_selftest");
        emit_module_vm_report_diagnostic_selftest();
        return DispatchOutcome::Response("module.vm_report_diagnostic_selftest");
    }
    if module_grant_diagnostic_method(method) {
        record_read("module.grant_diagnostic");
        emit_module_grant_diagnostic(method);
        return DispatchOutcome::Response("module.grant_diagnostic");
    }
    if module_grant_diagnostic_selftest_method(method) {
        record_read("module.grant_diagnostic_selftest");
        emit_module_grant_diagnostic_selftest();
        return DispatchOutcome::Response("module.grant_diagnostic_selftest");
    }
    if module_audit_rollback_diagnostic_method(method) {
        record_read("module.audit_rollback_diagnostic");
        emit_module_audit_rollback_diagnostic(method);
        return DispatchOutcome::Response("module.audit_rollback_diagnostic");
    }
    if module_audit_rollback_diagnostic_selftest_method(method) {
        record_read("module.audit_rollback_diagnostic_selftest");
        emit_module_audit_rollback_diagnostic_selftest();
        return DispatchOutcome::Response("module.audit_rollback_diagnostic_selftest");
    }
    if module_service_slot_diagnostic_method(method) {
        record_read("module.service_slot_diagnostic");
        emit_module_service_slot_diagnostic(method);
        return DispatchOutcome::Response("module.service_slot_diagnostic");
    }
    if module_service_slot_diagnostic_selftest_method(method) {
        record_read("module.service_slot_diagnostic_selftest");
        emit_module_service_slot_diagnostic_selftest();
        return DispatchOutcome::Response("module.service_slot_diagnostic_selftest");
    }
    if module_load_gate_manifest_selftest_method(method) {
        record_read("module.load_gate_manifest_selftest");
        emit_module_load_gate_manifest_selftest();
        return DispatchOutcome::Response("module.load_gate_manifest_selftest");
    }
    if module_load_gate_artifact_selftest_method(method) {
        record_read("module.load_gate_artifact_selftest");
        emit_module_load_gate_artifact_selftest();
        return DispatchOutcome::Response("module.load_gate_artifact_selftest");
    }
    if module_load_gate_vm_report_selftest_method(method) {
        record_read("module.load_gate_vm_report_selftest");
        emit_module_load_gate_vm_report_selftest();
        return DispatchOutcome::Response("module.load_gate_vm_report_selftest");
    }
    if module_load_gate_retained_selftest_method(method) {
        record_read("module.load_gate_retained_selftest");
        emit_module_load_gate_retained_selftest();
        return DispatchOutcome::Response("module.load_gate_retained_selftest");
    }
    if module_load_gate_audit_rollback_selftest_method(method) {
        record_read("module.load_gate_audit_rollback_selftest");
        emit_module_load_gate_audit_rollback_selftest();
        return DispatchOutcome::Response("module.load_gate_audit_rollback_selftest");
    }
    if module_load_gate_service_slot_selftest_method(method) {
        record_read("module.load_gate_service_slot_selftest");
        emit_module_load_gate_service_slot_selftest();
        return DispatchOutcome::Response("module.load_gate_service_slot_selftest");
    }

    if provider_context_export_method(method) {
        let event_id = record_denial("provider.context_export");
        emit_provider_context_export_denied(runtime, method, event_id);
        return DispatchOutcome::Denied("provider.context_export");
    }

    if module_load_ephemeral_method(method) {
        let method = canonical_module_load_ephemeral_method(method);
        let (event_id, gate_binding) = event_log::record_module_load_ephemeral_denied(method);
        emit_module_load_ephemeral_denied(method, event_id, gate_binding);
        return DispatchOutcome::Denied(method);
    }

    if memory_mutation_method(method) {
        let method = canonical_memory_mutation_method(method);
        let event_id = record_denial(method);
        emit_memory_capability_denied(method, event_id);
        return DispatchOutcome::Denied(method);
    }

    if denied_method(method) {
        let method = canonical_denied_method(method);
        let event_id = record_denial(method);
        emit_capability_denied(method, event_id);
        return DispatchOutcome::Denied(method);
    }

    DispatchOutcome::Unknown
}

fn record_read(method: &'static str) -> event_log::EventId {
    event_log::record_agent_read(method, requested_capability_for_read(method))
}

fn record_denial(method: &'static str) -> event_log::EventId {
    event_log::record_capability_denied(
        method,
        requested_capability_for_denial(method),
        risk_for_denial(method),
    )
}

fn emit_capability_denied(method: &'static str, event_id: event_log::EventId) {
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

fn denied_method(method: &str) -> bool {
    let mut idx = 0usize;
    while idx < DENIED_METHODS.len() {
        if method_eq(method, DENIED_METHODS[idx]) {
            return true;
        }
        idx += 1;
    }
    false
}

fn canonical_denied_method(method: &str) -> &'static str {
    let mut idx = 0usize;
    while idx < DENIED_METHODS.len() {
        if method_eq(method, DENIED_METHODS[idx]) {
            return DENIED_METHODS[idx];
        }
        idx += 1;
    }
    "unknown"
}

fn canonical_module_load_ephemeral_method(method: &str) -> &'static str {
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
    } else if method_eq(method, "module.manifest_diagnostic")
        || method_eq(method, "module.manifest_diagnostic_selftest")
        || method_eq(method, "module.artifact_diagnostic")
        || method_eq(method, "module.artifact_diagnostic_selftest")
        || method_eq(method, "module.vm_report_diagnostic")
        || method_eq(method, "module.vm_report_diagnostic_selftest")
        || method_eq(method, "module.grant_diagnostic")
        || method_eq(method, "module.grant_diagnostic_selftest")
        || method_eq(method, "module.audit_rollback_diagnostic")
        || method_eq(method, "module.audit_rollback_diagnostic_selftest")
        || method_eq(method, "module.service_slot_diagnostic")
        || method_eq(method, "module.service_slot_diagnostic_selftest")
        || method_eq(method, "module.load_gate_manifest_selftest")
        || method_eq(method, "module.load_gate_artifact_selftest")
        || method_eq(method, "module.load_gate_vm_report_selftest")
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

fn module_load_ephemeral_method(method: &str) -> bool {
    method_eq(method, "module.load_ephemeral") || method_eq(method, "service.load_ephemeral")
}
