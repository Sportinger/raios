use crate::{
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
        emit_module_load_gate_audit_rollback_selftest, emit_module_load_gate_event_binding,
        emit_module_load_gate_manifest_selftest, emit_module_load_gate_retained_selftest,
        emit_module_load_gate_service_slot_selftest, emit_module_load_gate_vm_report_selftest,
        module_load_gate_artifact_selftest_method, module_load_gate_audit_rollback_selftest_method,
        module_load_gate_manifest_selftest_method, module_load_gate_retained_selftest_method,
        module_load_gate_service_slot_selftest_method, module_load_gate_vm_report_selftest_method,
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
        emit_provider_context_gate_selftest, emit_provider_context_hashes,
        emit_provider_context_injection_gate, emit_provider_context_injection_gate_selftest,
        emit_provider_minimal_projection, provider_context_block_reason,
        provider_context_export_method, provider_context_gate_method,
        provider_context_gate_selftest_method, provider_context_injection_gate_method,
        provider_context_injection_gate_selftest_method,
    },
    agent_protocol_support::{
        begin_response, crlf, emit_inline_string_array, end_response, indent, json_event_id,
        json_event_sequence, json_sha256, json_str, method_eq, method_head_eq, raw, raw_bool,
        raw_fmt, raw_line,
    },
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_describe, emit_device_graph, emit_problem_list,
        emit_problem_objects, emit_service_ids, emit_service_inventory, emit_snapshot,
        emit_status_state, DENIED_METHODS,
    },
    event_log, provider, serial,
    system_status::SystemSnapshot,
    ui,
};

pub(crate) use crate::agent_protocol_provider::provider_minimal_context_evidence_for_runtime;

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

const MEMORY_MUTATION_METHODS: &[&str] = &[
    "memory.record_observation",
    "memory.propose_policy",
    "memory.supersede_fact",
    "memory.redact",
    "memory.compact",
];

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

fn emit_memory_profile() {
    begin_response("memory.profile");
    raw_line("      \"schema\": \"memory.profile.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"profiles\": [");
    raw_line("        {\"id\": \"diagnostic\", \"available\": true, \"target_tokens\": 4000, \"provider_export\": false, \"summary\": \"local current-boot facts for one diagnostic task\"},");
    raw_line("        {\"id\": \"planning\", \"available\": true, \"target_tokens\": 8000, \"provider_export\": false, \"summary\": \"local architecture and status handoff context\"},");
    raw_line("        {\"id\": \"provider_minimal\", \"available\": true, \"local_projection\": true, \"target_tokens\": 2000, \"provider_export\": false, \"blocked_by\": \"provider export requires positive provider trust plus current-boot export audit binding\", \"summary\": \"local read-only redaction projection for future provider context\"}");
    raw_line("      ],");
    raw_line("      \"read_methods\": [");
    raw_line("        \"memory.context\",");
    raw_line("        \"memory.query\",");
    raw_line("        \"memory.trace\",");
    raw_line("        \"memory.recent_events\",");
    raw_line("        \"audit.events\"");
    raw_line("      ],");
    raw_line("      \"mutation_policy\": \"denied_until_event_log_audit_policy_persistence_and_rollback_exist\"");
    end_response("memory.profile");
}

fn emit_memory_context(runtime: ui::RuntimeStatus, method: &str, event_id: event_log::EventId) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = memory_context_profile(method);

    begin_response("memory.context");
    raw_line("      \"schema\": \"raios.agent_context.v0\",");
    raw_line("      \"purpose\": \"current_boot_system_context\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw("      \"context_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"source_schemas\": [");
    raw_line("        \"system.snapshot.v0\",");
    raw_line("        \"system.capabilities.v0\",");
    raw_line("        \"service.inventory.v0\",");
    raw_line("        \"problem.list.v0\",");
    raw_line("        \"system.boot_log.v0\"");
    raw_line("      ],");
    raw("      \"budget\": {\"target_tokens\": ");
    raw_fmt(format_args!("{}", memory_context_target_tokens(profile)));
    raw(", \"estimated_tokens\": ");
    raw_fmt(format_args!("{}", memory_context_estimated_tokens(profile)));
    raw_line("},");
    raw_line("      \"authority_order\": [");
    raw_line("        \"current_snapshot\",");
    raw_line("        \"decision\",");
    raw_line("        \"service_state\",");
    raw_line("        \"summary\"");
    raw_line("      ],");
    raw_line("      \"included\": {");
    raw_line("        \"identity\": [\"mem.fact.identity.stage0\"],");
    raw_line("        \"policy\": [\"adr.0001\", \"adr.0004\"],");
    raw_line("        \"current\": [\"snapshot.current\", \"capabilities.current_boot\", \"service.inventory.current\", \"problem.list.current\"],");
    raw_line("        \"summaries\": [\"boot_log.summary.current\"]");
    raw_line("      },");
    raw_line("      \"current\": {");
    raw_line("        \"snapshot_id\": \"snapshot.current\",");
    raw_line("        \"status\": {");
    emit_status_state("framebuffer", status.framebuffer.state, true);
    emit_status_state("entropy", status.entropy.state, true);
    emit_status_state("usb_xhci", status.usb_xhci.state, true);
    emit_status_state("wifi", status.wifi.state, true);
    emit_status_state("network", status.network.state, true);
    emit_status_state("input", status.input.state, false);
    raw_line("        },");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_api_key_state\": ");
    json_str(if provider.api_key_set {
        "set"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"capability_posture\": \"observe_only_mutations_denied\",");
    raw_line("        \"services\": [");
    emit_service_ids(10);
    raw_line("        ],");
    raw_line("        \"problems\": [");
    emit_problem_objects(&status, &provider, 10);
    raw_line("        ]");
    raw_line("      },");
    if method_eq(profile, "provider_minimal") {
        raw_line("      \"provider_projection\": {");
        emit_provider_minimal_projection(&status, &provider, event_id);
        raw_line("      },");
    }
    raw_line("      \"records\": [");
    emit_memory_record(
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 booted kernel identity",
        "system.describe",
        true,
    );
    emit_memory_record(
        "snapshot.current",
        "current_snapshot",
        "current_snapshot",
        "local_only",
        "current framebuffer, entropy, USB, Wi-Fi, network, input, provider, capabilities, and problems",
        "system.snapshot",
        true,
    );
    emit_memory_record(
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
        "system.capabilities",
        true,
    );
    emit_memory_record(
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "static current-boot service inventory for the monolithic Stage-0 kernel",
        "service.inventory",
        true,
    );
    emit_memory_record(
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current known local problems and explicit gaps",
        "problem.list",
        true,
    );
    emit_memory_record(
        "boot_log.summary.current",
        "summary",
        "summary",
        "local_only",
        "serial boot log summary locator; raw lines remain local",
        "system.boot_log",
        true,
    );
    emit_memory_record(
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI into Stage-0",
        "docs/architecture-decisions/0001-raios-agent-protocol.md",
        true,
    );
    emit_memory_record(
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory is typed local facts with bounded task-scoped agent_context packets, not prompt stuffing",
        "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
        false,
    );
    raw_line("      ],");
    raw_line("      \"omitted\": [");
    raw_line("        {\"kind\": \"raw_boot_log\", \"reason\": \"memory.context includes only a summary locator; use system.boot_log or memory.trace locally for raw lines\"},");
    raw_line("        {\"kind\": \"local_only_details\", \"reason\": \"details strings may contain IPs, PCI data, topology, request ids, or hashes\"},");
    raw_line("        {\"kind\": \"secret_values\", \"reason\": \"API keys, Wi-Fi passphrases, and raw secret values are never included\"},");
    raw_line("        {\"kind\": \"provider_export\", \"reason\": \"disabled until positive provider trust and current-boot provider export audit binding exist\"},");
    raw("        {\"kind\": \"provider_minimal\", \"reason\": ");
    json_str(provider_context_block_reason(provider.trust_state));
    raw_line("}");
    raw_line("      ]");
    end_response("memory.context");
}

fn emit_memory_query() {
    begin_response("memory.query");
    raw_line("      \"schema\": \"memory.query.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"query\": \"static_current_boot_index\",");
    raw_line("      \"records\": [");
    emit_memory_candidate(
        "mem.fact.identity.stage0",
        "fact",
        "public",
        "raiOS Stage-0 identity",
        true,
    );
    emit_memory_candidate(
        "snapshot.current",
        "current_snapshot",
        "local_only",
        "current typed system snapshot",
        true,
    );
    emit_memory_candidate(
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "public",
        "provider_minimal projection of current status and provider trust",
        true,
    );
    emit_memory_candidate(
        "capabilities.current_boot",
        "capability_index",
        "public",
        "observe-only capability posture",
        true,
    );
    emit_memory_candidate(
        "service.inventory.current",
        "service_state",
        "public",
        "static service inventory",
        true,
    );
    emit_memory_candidate(
        "problem.list.current",
        "problem_index",
        "public",
        "current problem ids and severities",
        true,
    );
    emit_memory_candidate(
        "boot_log.summary.current",
        "summary",
        "local_only",
        "serial boot log summary locator",
        true,
    );
    emit_memory_candidate(
        "adr.0001",
        "decision",
        "public",
        "raiOS Agent Protocol instead of Codex CLI",
        true,
    );
    emit_memory_candidate(
        "adr.0004",
        "decision",
        "public",
        "System memory and agent context selection",
        false,
    );
    raw_line("      ],");
    raw_line("      \"semantic_index\": \"not_implemented_locator_only\"");
    end_response("memory.query");
}

fn emit_memory_trace(method: &str) {
    let id = memory_method_arg(method, "memory.trace");

    begin_response("memory.trace");
    raw_line("      \"schema\": \"memory.trace.v0\",");
    raw("      \"requested_id\": ");
    if id.is_empty() {
        raw("null");
    } else {
        json_str(id);
    }
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"records\": [");
    if id.is_empty() {
        emit_trace_record(
            "snapshot.current",
            "system.snapshot",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "snapshot.current.provider_minimal",
            "memory.context provider_minimal",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "service.inventory.current",
            "service.inventory",
            "seed-kernel/src/service_inventory.rs",
            true,
        );
        emit_trace_record(
            "problem.list.current",
            "problem.list",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "adr.0001",
            "decision",
            "docs/architecture-decisions/0001-raios-agent-protocol.md",
            true,
        );
        emit_trace_record(
            "adr.0004",
            "decision",
            "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
            false,
        );
    } else {
        emit_single_trace_record(id);
    }
    raw_line("      ]");
    end_response("memory.trace");
}

fn emit_recent_events(method: &str) {
    let limit = event_limit_arg(method);
    let snapshot = event_log::snapshot_recent(limit);

    begin_response("memory.recent_events");
    raw_line("      \"schema\": \"event.log.v0\",");
    raw_line("      \"record_schema\": \"audit.event.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"retention\": \"ram_ring\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"bounded\": true,");
    raw("      \"limit\": ");
    raw_fmt(format_args!("{}", snapshot.limit));
    raw_line(",");
    raw("      \"capacity\": ");
    raw_fmt(format_args!("{}", snapshot.capacity));
    raw_line(",");
    raw("      \"event_count\": ");
    raw_fmt(format_args!("{}", snapshot.total_count));
    raw_line(",");
    raw("      \"returned\": ");
    raw_fmt(format_args!("{}", snapshot.len));
    raw_line(",");
    raw("      \"dropped_before_sequence\": ");
    raw_fmt(format_args!("{}", snapshot.dropped_before_sequence));
    raw_line(",");
    raw_line("      \"events\": [");

    let mut idx = 0usize;
    while idx < snapshot.len {
        if let Some(event) = snapshot.events[idx] {
            emit_event(&event, idx + 1 != snapshot.len);
        }
        idx += 1;
    }

    raw_line("      ]");
    end_response("memory.recent_events");
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

fn emit_memory_capability_denied(method: &'static str, event_id: event_log::EventId) {
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
    json_str("memory mutation is denied until durable audit, policy, source retention, persistence, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"policy_ledger\",");
    raw_line("      \"source_retention\",");
    raw_line("      \"redaction_transaction\",");
    raw_line("      \"raios.memory_persistence.v0\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ]");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

fn emit_event(event: &event_log::Event, comma: bool) {
    indent(8);
    raw("{\"schema\": \"audit.event.v0\", \"id\": ");
    json_event_sequence(event.sequence);
    raw(", \"scope\": \"current_boot\", \"sequence\": ");
    raw_fmt(format_args!("{}", event.sequence));
    raw(", \"kind\": ");
    json_str(event.kind);
    raw(", \"source_method\": ");
    json_str(event.source_method);
    raw(", \"source_transport\": ");
    json_str(event.source_transport);
    raw(", \"classification\": ");
    json_str(event.classification);
    raw(", \"outcome\": ");
    json_str(event.outcome);
    raw(", \"requested_capability\": ");
    json_str(event.requested_capability);
    raw(", \"risk\": ");
    json_str(event.risk);
    raw(", \"subject\": ");
    json_str(event.subject);
    raw(", \"resource\": ");
    json_str(event.resource);
    raw(", \"reason\": ");
    json_str(event.reason);
    raw(", \"evidence\": [");
    emit_inline_string_array(event.evidence);
    raw("], \"created_at\": {\"clock\": \"sequence_only\", \"millis\": null}");
    emit_event_bindings(event.bindings);
    raw(", \"persistence\": \"none\"}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_event_bindings(bindings: event_log::EventBindings) {
    match bindings {
        event_log::EventBindings::None => {}
        event_log::EventBindings::ProviderRequestEnvelope(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_request_envelope.v0\", \"status\": \"local_prewrite_envelope\", \"satisfies_current_boot_export_gate\": false, \"provider_write\": \"not_attempted\", \"context_attached_to_provider_body\": false, \"request_id\": ");
            raw_fmt(format_args!("{}", binding.request_id));
            raw(", \"request_body_hash\": ");
            json_sha256(binding.request_body_hash);
            raw(", \"envelope_hash\": ");
            json_sha256(binding.envelope_hash);
            raw(", \"trust_snapshot\": {\"provider_trust_state\": ");
            json_str(binding.provider_trust_state);
            raw(", \"provider_trust_positive\": ");
            raw_bool(binding.provider_trust_positive);
            raw(", \"development_tls_bypass\": ");
            raw_bool(binding.development_tls_bypass);
            raw("}}");
        }
        event_log::EventBindings::ProviderRequestBound(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_request_binding.v0\", \"status\": \"bound\", \"satisfies_request_binding_gate\": true, \"satisfies_current_boot_export_gate\": false, \"provider_write_at_binding\": \"not_attempted\", \"context_attached_to_provider_body\": false, \"request_id\": ");
            raw_fmt(format_args!("{}", binding.request_id));
            raw(", \"request_envelope_event_id\": ");
            json_event_id(binding.request_envelope_event_id);
            raw(", \"request_body_hash\": ");
            json_sha256(binding.request_body_hash);
            raw(", \"request_envelope_hash\": ");
            json_sha256(binding.request_envelope_hash);
            raw(", \"request_binding_hash\": ");
            json_sha256(binding.request_binding_hash);
            raw(", \"trust_snapshot\": {\"provider_trust_state\": ");
            json_str(binding.provider_trust_state);
            raw(", \"provider_trust_positive\": true, \"development_tls_bypass\": ");
            raw_bool(binding.development_tls_bypass);
            raw("}, \"hashes\": ");
            emit_provider_context_hashes(binding.context);
            raw("}");
        }
        event_log::EventBindings::ProviderExportAuditBound(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_context_export_audit_binding.v0\", \"status\": \"authorized_for_single_provider_request\", \"satisfies_export_audit_binding_gate\": true, \"satisfies_current_boot_export_gate\": false, \"positive_export_authorization\": true, \"automatic_context_injection\": \"disabled\", \"provider_write_at_binding\": \"not_attempted\", \"context_attached_to_provider_body\": ");
            raw_bool(binding.context_attached_to_provider_body);
            raw(", \"request_id\": ");
            raw_fmt(format_args!("{}", binding.request_id));
            raw(", \"request_envelope_event_id\": ");
            json_event_id(binding.request_envelope_event_id);
            raw(", \"request_binding_event_id\": ");
            json_event_id(binding.request_binding_event_id);
            raw(", \"request_body_hash\": ");
            json_sha256(binding.request_body_hash);
            raw(", \"request_envelope_hash\": ");
            json_sha256(binding.request_envelope_hash);
            raw(", \"request_binding_hash\": ");
            json_sha256(binding.request_binding_hash);
            raw(", \"export_audit_binding_hash\": ");
            json_sha256(binding.export_audit_binding_hash);
            raw(", \"trust_snapshot\": {\"provider_trust_state\": ");
            json_str(binding.provider_trust_state);
            raw(", \"provider_trust_positive\": true, \"development_tls_bypass\": false}, \"hashes\": ");
            emit_provider_context_hashes(binding.context);
            raw("}");
        }
        event_log::EventBindings::ProviderBindingConsumption(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_context_binding_consumption.v0\", \"status\": \"consumed_for_gate_evaluation\", \"satisfies_current_boot_export_gate\": false, \"automatic_context_injection\": \"disabled\", \"provider_write\": \"not_attempted\", \"context_attached_to_provider_body\": false, \"request_id\": ");
            raw_fmt(format_args!("{}", binding.request_id));
            raw(", \"request_envelope_event_id\": ");
            json_event_id(binding.request_envelope_event_id);
            raw(", \"request_binding_event_id\": ");
            json_event_id(binding.request_binding_event_id);
            raw(", \"export_audit_binding_event_id\": ");
            json_event_id(binding.export_audit_binding_event_id);
            raw(", \"request_binding_hash\": ");
            json_sha256(binding.request_binding_hash);
            raw(", \"export_audit_binding_hash\": ");
            json_sha256(binding.export_audit_binding_hash);
            raw(", \"hashes\": ");
            emit_provider_context_hashes(binding.context);
            raw("}");
        }
        event_log::EventBindings::ProviderContextInjectionAuthorization(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_context_injection_authorization.v0\", \"status\": \"authorized_for_single_provider_request\", \"satisfies_current_boot_export_gate\": false, \"automatic_context_injection\": \"disabled\", \"provider_write\": \"not_attempted\", \"context_attached_to_provider_body\": ");
            raw_bool(binding.context_attached_to_provider_body);
            raw(", \"request_id\": ");
            raw_fmt(format_args!("{}", binding.request_id));
            raw(", \"request_envelope_event_id\": ");
            json_event_id(binding.request_envelope_event_id);
            raw(", \"request_binding_event_id\": ");
            json_event_id(binding.request_binding_event_id);
            raw(", \"export_audit_binding_event_id\": ");
            json_event_id(binding.export_audit_binding_event_id);
            raw(", \"binding_consumption_event_id\": ");
            json_event_id(binding.binding_consumption_event_id);
            raw(", \"request_body_hash\": ");
            json_sha256(binding.request_body_hash);
            raw(", \"request_envelope_hash\": ");
            json_sha256(binding.request_envelope_hash);
            raw(", \"request_binding_hash\": ");
            json_sha256(binding.request_binding_hash);
            raw(", \"export_audit_binding_hash\": ");
            json_sha256(binding.export_audit_binding_hash);
            raw(", \"final_authorization_hash\": ");
            json_sha256(binding.final_authorization_hash);
            raw(", \"trust_snapshot\": {\"provider_trust_state\": ");
            json_str(binding.provider_trust_state);
            raw(", \"provider_trust_positive\": true}, \"hashes\": ");
            emit_provider_context_hashes(binding.context);
            raw("}");
        }
        event_log::EventBindings::ProviderRequestBindingDenied(hashes) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_request_binding_denial.v0\", \"status\": \"denied_not_bound\", \"satisfies_current_boot_export_gate\": false, \"provider_write\": \"not_attempted\", \"hashes\": ");
            emit_provider_context_hashes(hashes);
            raw("}");
        }
        event_log::EventBindings::ProviderExportDenialAudit(hashes) => {
            raw(", \"bindings\": {\"schema\": \"raios.provider_context_export_denial_audit.v0\", \"status\": \"denied_no_provider_write\", \"satisfies_current_boot_export_gate\": false, \"positive_export_authorization\": false, \"provider_write\": \"not_attempted\", \"hashes\": ");
            emit_provider_context_hashes(hashes);
            raw("}");
        }
        event_log::EventBindings::ModuleManifestReference(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_manifest_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"manifest_schema\": \"raios.module_manifest.v0\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"manifest_reference_hash\": ");
            json_sha256(binding.manifest_reference_hash);
            raw(", \"manifest_hash\": ");
            json_sha256(binding.manifest_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleCandidateArtifactReference(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
            json_event_id(binding.retained_manifest_reference_event_id);
            raw(", \"retained_computed_grant_reference_event_id\": ");
            json_event_id(binding.retained_reference_event_id);
            raw(", \"hashes\": {\"artifact_reference_hash\": ");
            json_sha256(binding.artifact_reference_hash);
            raw(", \"manifest_reference_hash\": ");
            json_sha256(binding.manifest_reference_hash);
            raw(", \"manifest_hash\": ");
            json_sha256(binding.manifest_hash);
            raw(", \"computed_capability_grant_hash\": ");
            json_sha256(binding.computed_grant_hash);
            raw(", \"artifact_hash\": ");
            json_sha256(binding.artifact_hash);
            raw(", \"vm_test_report_hash\": ");
            json_sha256(binding.vm_report_hash);
            raw(", \"local_attestation_hash\": ");
            json_sha256(binding.local_attestation_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleVmTestReportReference(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"vm_test_report_schema\": \"raios.vm_test_report.v0\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_vm_report_json\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
            json_event_id(binding.retained_manifest_reference_event_id);
            raw(", \"retained_candidate_artifact_reference_event_id\": ");
            json_event_id(binding.retained_artifact_reference_event_id);
            raw(", \"retained_computed_grant_reference_event_id\": ");
            json_event_id(binding.retained_reference_event_id);
            raw(", \"hashes\": {\"vm_test_report_reference_hash\": ");
            json_sha256(binding.report_reference_hash);
            raw(", \"manifest_reference_hash\": ");
            json_sha256(binding.manifest_reference_hash);
            raw(", \"artifact_reference_hash\": ");
            json_sha256(binding.artifact_reference_hash);
            raw(", \"manifest_hash\": ");
            json_sha256(binding.manifest_hash);
            raw(", \"artifact_hash\": ");
            json_sha256(binding.artifact_hash);
            raw(", \"computed_capability_grant_hash\": ");
            json_sha256(binding.computed_grant_hash);
            raw(", \"vm_test_report_hash\": ");
            json_sha256(binding.vm_report_hash);
            raw(", \"local_attestation_hash\": ");
            json_sha256(binding.local_attestation_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleComputedGrantReference(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"computed_capability_grant_hash\": ");
            json_sha256(binding.computed_grant_hash);
            raw(", \"manifest_hash\": ");
            json_sha256(binding.manifest_hash);
            raw(", \"artifact_hash\": ");
            json_sha256(binding.artifact_hash);
            raw(", \"vm_test_report_hash\": ");
            json_sha256(binding.vm_report_hash);
            raw(", \"local_attestation_hash\": ");
            json_sha256(binding.local_attestation_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleAuditRollbackReference(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"denial_event_id\": ");
            json_event_id(binding.denial_event_id);
            raw(", \"retained_computed_grant_reference_event_id\": ");
            json_event_id(binding.retained_reference_event_id);
            raw(", \"ram_only_service_slot_id\": ");
            json_str(binding.ram_only_service_slot_id.as_str());
            raw(", \"hashes\": {\"audit_record_hash\": ");
            json_sha256(binding.audit_record_hash);
            raw(", \"rollback_plan_hash\": ");
            json_sha256(binding.rollback_plan_hash);
            raw(", \"computed_capability_grant_hash\": ");
            json_sha256(binding.computed_grant_hash);
            raw(", \"manifest_hash\": ");
            json_sha256(binding.manifest_hash);
            raw(", \"artifact_hash\": ");
            json_sha256(binding.artifact_hash);
            raw(", \"vm_test_report_hash\": ");
            json_sha256(binding.vm_report_hash);
            raw(", \"local_attestation_hash\": ");
            json_sha256(binding.local_attestation_hash);
            raw(", \"local_approval_hash\": ");
            json_sha256(binding.local_approval_hash);
            raw(", \"pre_load_service_inventory_hash\": ");
            json_sha256(binding.pre_load_service_inventory_hash);
            raw(", \"cleanup_actions_hash\": ");
            json_sha256(binding.cleanup_actions_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleServiceSlotReservation(binding) => {
            raw(", \"bindings\": {\"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_computed_grant_reference_event_id\": ");
            json_event_id(binding.retained_reference_event_id);
            raw(", \"retained_audit_rollback_reference_event_id\": ");
            json_event_id(binding.retained_audit_rollback_reference_event_id);
            raw(", \"ram_only_service_slot_id\": ");
            json_str(binding.ram_only_service_slot_id.as_str());
            raw(", \"hashes\": {\"reservation_hash\": ");
            json_sha256(binding.reservation_hash);
            raw(", \"computed_capability_grant_hash\": ");
            json_sha256(binding.computed_grant_hash);
            raw(", \"audit_record_hash\": ");
            json_sha256(binding.audit_record_hash);
            raw(", \"rollback_plan_hash\": ");
            json_sha256(binding.rollback_plan_hash);
            raw(", \"pre_load_service_inventory_hash\": ");
            json_sha256(binding.pre_load_service_inventory_hash);
            raw("}}");
        }
        event_log::EventBindings::ModuleLoadGate(binding) => {
            emit_module_load_gate_event_binding(binding);
        }
    }
}

fn emit_memory_record(
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
    source: &'static str,
    comma: bool,
) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"kind\": ");
    json_str(kind);
    raw(", \"authority\": ");
    json_str(authority);
    raw(", \"classification\": ");
    json_str(classification);
    raw(", \"summary\": ");
    json_str(summary);
    raw(", \"source\": ");
    json_str(source);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_memory_candidate(
    id: &'static str,
    kind: &'static str,
    classification: &'static str,
    summary: &'static str,
    comma: bool,
) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"kind\": ");
    json_str(kind);
    raw(", \"classification\": ");
    json_str(classification);
    raw(", \"summary\": ");
    json_str(summary);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_trace_record(
    id: &'static str,
    source_method: &'static str,
    source: &'static str,
    comma: bool,
) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"found\": true, \"source_method\": ");
    json_str(source_method);
    raw(", \"source\": ");
    json_str(source);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_single_trace_record(id: &str) {
    if method_eq(id, "mem.fact.identity.stage0") {
        emit_trace_record(
            "mem.fact.identity.stage0",
            "system.describe",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "snapshot.current") {
        emit_trace_record(
            "snapshot.current",
            "system.snapshot",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "snapshot.current.provider_minimal") {
        emit_trace_record(
            "snapshot.current.provider_minimal",
            "memory.context provider_minimal",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "capabilities.current_boot") {
        emit_trace_record(
            "capabilities.current_boot",
            "system.capabilities",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "service.inventory.current") {
        emit_trace_record(
            "service.inventory.current",
            "service.inventory",
            "seed-kernel/src/service_inventory.rs",
            false,
        );
    } else if method_eq(id, "problem.list.current") {
        emit_trace_record(
            "problem.list.current",
            "problem.list",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "boot_log.summary.current") {
        emit_trace_record(
            "boot_log.summary.current",
            "system.boot_log",
            "seed-kernel/src/serial.rs",
            false,
        );
    } else if method_eq(id, "adr.0001") {
        emit_trace_record(
            "adr.0001",
            "decision",
            "docs/architecture-decisions/0001-raios-agent-protocol.md",
            false,
        );
    } else if method_eq(id, "adr.0004") {
        emit_trace_record(
            "adr.0004",
            "decision",
            "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
            false,
        );
    } else {
        indent(8);
        raw("{\"id\": ");
        json_str(id);
        raw(", \"found\": false, \"reason\": \"record id is not in the current_boot memory index\"}");
        crlf();
    }
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

fn memory_mutation_method(method: &str) -> bool {
    let mut idx = 0usize;
    while idx < MEMORY_MUTATION_METHODS.len() {
        if method_eq(method, MEMORY_MUTATION_METHODS[idx]) {
            return true;
        }
        idx += 1;
    }
    false
}

fn canonical_memory_mutation_method(method: &str) -> &'static str {
    let mut idx = 0usize;
    while idx < MEMORY_MUTATION_METHODS.len() {
        if method_eq(method, MEMORY_MUTATION_METHODS[idx]) {
            return MEMORY_MUTATION_METHODS[idx];
        }
        idx += 1;
    }
    "unknown"
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

fn memory_context_profile(method: &str) -> &'static str {
    let arg = memory_method_arg(method, "memory.context");
    if method_eq(arg, "planning") {
        "planning"
    } else if method_eq(arg, "provider_minimal") {
        "provider_minimal"
    } else {
        "diagnostic"
    }
}

fn memory_context_target_tokens(profile: &str) -> u16 {
    if method_eq(profile, "planning") {
        8000
    } else if method_eq(profile, "provider_minimal") {
        2000
    } else {
        4000
    }
}

fn memory_context_estimated_tokens(profile: &str) -> u16 {
    if method_eq(profile, "planning") {
        1600
    } else if method_eq(profile, "provider_minimal") {
        900
    } else {
        1200
    }
}

fn memory_method_arg<'a>(method: &'a str, canonical: &str) -> &'a str {
    let method = method.trim();
    let head_len = if method_head_eq(method, canonical) {
        canonical.len()
    } else if method_head_eq(method, "memctx") {
        "memctx".len()
    } else if method_head_eq(method, "memtrace") {
        "memtrace".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn event_limit_arg(method: &str) -> usize {
    let method = method.trim();
    let head_len = if method_head_eq(method, "memory.recent_events") {
        "memory.recent_events".len()
    } else if method_head_eq(method, "audit.events") {
        "audit.events".len()
    } else if method_head_eq(method, "events") {
        "events".len()
    } else {
        return event_log::DEFAULT_EVENT_LIMIT;
    };

    parse_usize_arg(method[head_len..].trim())
}

fn parse_usize_arg(value: &str) -> usize {
    let mut parsed = 0usize;
    let mut saw_digit = false;
    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            if saw_digit {
                break;
            }
            continue;
        }
        if !byte.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        parsed = parsed
            .saturating_mul(10)
            .saturating_add((byte - b'0') as usize);
    }

    if saw_digit {
        parsed
    } else {
        event_log::DEFAULT_EVENT_LIMIT
    }
}
