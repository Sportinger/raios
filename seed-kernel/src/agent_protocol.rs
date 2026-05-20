use sha2::{Digest, Sha256};

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
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_inline_string_array, end_response, indent,
        json_current_boot_id, json_event_id, json_event_id_option, json_event_sequence,
        json_opt_str, json_sha256, json_str, method_eq, method_head_eq, raw, raw_bool, raw_fmt,
        raw_line,
    },
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_capability_ids, emit_describe, emit_device_graph,
        emit_problem_list, emit_problem_objects, emit_service_ids, emit_service_inventory,
        emit_snapshot, emit_status_state, emit_status_state_at, CAPABILITIES, DENIED_METHODS,
    },
    event_log, provider, serial, service_inventory, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, wifi,
};

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

#[derive(Clone, Copy)]
struct ProjectionFieldSpec {
    field: &'static str,
    classification: &'static str,
    action: &'static str,
    reason: &'static str,
}

pub(crate) struct ProviderContextEvidence {
    pub projected_packet_hash: [u8; 32],
    pub exported_field_list_hash: [u8; 32],
    pub omitted_field_list_hash: [u8; 32],
}

impl ProviderContextEvidence {
    pub(crate) fn event_hashes(&self) -> event_log::ProviderContextHashes {
        event_log::ProviderContextHashes {
            projected_packet_hash: self.projected_packet_hash,
            exported_field_list_hash: self.exported_field_list_hash,
            omitted_field_list_hash: self.omitted_field_list_hash,
        }
    }
}

pub(crate) fn provider_minimal_context_evidence_for_runtime(
    runtime: ui::RuntimeStatus,
) -> ProviderContextEvidence {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    provider_context_evidence(&status, &provider)
}

const PROVIDER_MINIMAL_INCLUDED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "schema",
        classification: "public",
        action: "include",
        reason: "context schema id is non-secret and required for decoding",
    },
    ProjectionFieldSpec {
        field: "purpose",
        classification: "public",
        action: "include",
        reason: "bounded task purpose, not raw user prompt",
    },
    ProjectionFieldSpec {
        field: "profile",
        classification: "public",
        action: "include",
        reason: "redaction profile id",
    },
    ProjectionFieldSpec {
        field: "scope",
        classification: "public",
        action: "include",
        reason: "current_boot scope marker",
    },
    ProjectionFieldSpec {
        field: "budget.target_tokens",
        classification: "public",
        action: "include",
        reason: "token budget target for the projection",
    },
    ProjectionFieldSpec {
        field: "budget.estimated_tokens",
        classification: "public",
        action: "include",
        reason: "bounded estimate only",
    },
    ProjectionFieldSpec {
        field: "authority_order[]",
        classification: "public",
        action: "include",
        reason: "authority labels from ADR 0004",
    },
    ProjectionFieldSpec {
        field: "included.*[]",
        classification: "public",
        action: "include",
        reason: "stable record ids only",
    },
    ProjectionFieldSpec {
        field: "current.os.*",
        classification: "public",
        action: "include",
        reason: "product identity and stage",
    },
    ProjectionFieldSpec {
        field: "current.status.*",
        classification: "public",
        action: "include",
        reason: "coarse subsystem states only",
    },
    ProjectionFieldSpec {
        field: "current.provider.selected",
        classification: "public",
        action: "include",
        reason: "provider family only",
    },
    ProjectionFieldSpec {
        field: "current.provider.route",
        classification: "public",
        action: "include",
        reason: "canonical route name without credentials",
    },
    ProjectionFieldSpec {
        field: "current.provider.api_key_state",
        classification: "public",
        action: "include",
        reason: "state marker only; never the key",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_phase",
        classification: "public",
        action: "include",
        reason: "coarse provider phase",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_endpoint",
        classification: "public",
        action: "include",
        reason: "canonical provider endpoint",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_model",
        classification: "public",
        action: "include",
        reason: "model id",
    },
    ProjectionFieldSpec {
        field: "current.provider.trust_state",
        classification: "public",
        action: "include",
        reason: "required for fail-closed provider policy",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_kind",
        classification: "public",
        action: "include",
        reason: "pin type only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_id",
        classification: "public",
        action: "include",
        reason: "short non-secret pin identifier only",
    },
    ProjectionFieldSpec {
        field: "current.provider.development_bypass",
        classification: "public",
        action: "include",
        reason: "must be visible because it blocks trusted export",
    },
    ProjectionFieldSpec {
        field: "current.services[]",
        classification: "public",
        action: "include",
        reason: "stable service ids only",
    },
    ProjectionFieldSpec {
        field: "current.capabilities[]",
        classification: "public",
        action: "include",
        reason: "capability ids and denied mutation posture",
    },
    ProjectionFieldSpec {
        field: "current.problems[].id",
        classification: "public",
        action: "include",
        reason: "stable problem ids",
    },
    ProjectionFieldSpec {
        field: "current.problems[].severity",
        classification: "public",
        action: "include",
        reason: "coarse severity",
    },
    ProjectionFieldSpec {
        field: "current.problems[].summary",
        classification: "public",
        action: "include",
        reason: "stable scrubbed summaries only",
    },
    ProjectionFieldSpec {
        field: "records[].id",
        classification: "public",
        action: "include",
        reason: "stable locators",
    },
    ProjectionFieldSpec {
        field: "records[].kind",
        classification: "public",
        action: "include",
        reason: "record kind labels",
    },
    ProjectionFieldSpec {
        field: "records[].authority",
        classification: "public",
        action: "include",
        reason: "authority labels",
    },
    ProjectionFieldSpec {
        field: "records[].classification",
        classification: "public",
        action: "include",
        reason: "classification labels",
    },
    ProjectionFieldSpec {
        field: "records[].summary",
        classification: "public",
        action: "include",
        reason: "only summaries for records classified public",
    },
];

const PROVIDER_MINIMAL_OMITTED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "source_schemas",
        classification: "local_only",
        action: "omit",
        reason: "local provenance list; the provider packet names only its projection schema",
    },
    ProjectionFieldSpec {
        field: "system.snapshot.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw system.snapshot is never attached to provider context",
    },
    ProjectionFieldSpec {
        field: "details.*.detail",
        classification: "local_only",
        action: "omit",
        reason: "detail strings may contain IPs, PCI data, topology, request ids, or hashes",
    },
    ProjectionFieldSpec {
        field: "network.ip",
        classification: "local_only",
        action: "omit",
        reason: "local network address",
    },
    ProjectionFieldSpec {
        field: "network.gateway",
        classification: "local_only",
        action: "omit",
        reason: "local network topology",
    },
    ProjectionFieldSpec {
        field: "network.dns",
        classification: "local_only",
        action: "omit",
        reason: "local resolver topology",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_prompt",
        classification: "secret",
        action: "omit",
        reason: "raw prompt text is not context memory",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_error",
        classification: "local_only",
        action: "omit",
        reason: "error text may contain request ids or provider diagnostics",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_event",
        classification: "local_only",
        action: "omit",
        reason: "free-form event text remains local",
    },
    ProjectionFieldSpec {
        field: "provider.direct_pending_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_request_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.tcp.*",
        classification: "local_only",
        action: "omit",
        reason: "TCP diagnostics include local transport details",
    },
    ProjectionFieldSpec {
        field: "wifi.ssid",
        classification: "secret",
        action: "omit",
        reason: "raw SSID is user-local configuration",
    },
    ProjectionFieldSpec {
        field: "wifi.passphrase",
        classification: "secret",
        action: "omit",
        reason: "raw Wi-Fi passphrase is never exported",
    },
    ProjectionFieldSpec {
        field: "system.boot_log.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw boot log remains behind local-only methods",
    },
    ProjectionFieldSpec {
        field: "boot_log.summary.current",
        classification: "local_only",
        action: "omit",
        reason: "boot-log summary is a local locator, not provider context",
    },
    ProjectionFieldSpec {
        field: "records[].source",
        classification: "local_only",
        action: "omit",
        reason: "source file paths and local method names remain trace-only locators",
    },
];

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

fn emit_provider_minimal_projection(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    event_id: event_log::EventId,
) {
    let trust_positive = provider_trust_positive(provider.trust_state);
    let evidence = provider_context_evidence(status, provider);

    raw_line("        \"schema\": \"raios.provider_context_projection.v0\",");
    raw_line("        \"mode\": \"local_read_only\",");
    raw_line("        \"profile\": \"provider_minimal\",");
    raw_line("        \"provider_export\": \"disabled\",");
    raw_line("        \"redaction_projection\": \"present\",");
    raw_line("        \"classification_default\": \"local_only\",");
    raw_line("        \"unclassified_field_policy\": \"omit\",");
    raw_line("        \"packet_evidence\": {");
    raw_line("          \"canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",");
    raw("          \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("          \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("          \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line("");
    raw_line("        },");
    raw("        \"local_projection_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("        \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw_line("        \"can_export\": false,");
    raw_line("        \"blocked_by\": [");
    if !trust_positive {
        raw("          {\"gate\": \"provider_trust\", \"state\": ");
        json_str(provider.trust_state);
        raw_line(", \"reason\": \"provider_trust_not_positive\"},");
    }
    raw_line("          {\"gate\": \"provider_context_export_audit_binding\", \"state\": \"missing\", \"reason\": \"provider_context_export_audit_binding_missing\"}");
    raw_line("        ],");
    raw_line("        \"included_fields\": [");
    emit_projection_field_specs(PROVIDER_MINIMAL_INCLUDED_FIELDS, 10);
    raw_line("        ],");
    raw_line("        \"omitted_fields\": [");
    emit_projection_field_specs(PROVIDER_MINIMAL_OMITTED_FIELDS, 10);
    raw_line("        ],");
    raw_line("        \"packet\": {");
    emit_provider_minimal_packet(status, provider);
    raw_line("        }");
}

fn emit_provider_minimal_packet(status: &SystemSnapshot, provider: &provider::Snapshot) {
    raw_line("          \"schema\": \"raios.agent_context.v0\",");
    raw_line("          \"purpose\": \"current_boot_provider_context\",");
    raw_line("          \"profile\": \"provider_minimal\",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"budget\": {\"target_tokens\": 2000, \"estimated_tokens\": 900},");
    raw_line("          \"authority_order\": [");
    raw_line("            \"current_snapshot\",");
    raw_line("            \"decision\",");
    raw_line("            \"service_state\",");
    raw_line("            \"summary\"");
    raw_line("          ],");
    raw_line("          \"included\": {");
    raw_line("            \"identity\": [\"mem.fact.identity.stage0\"],");
    raw_line("            \"policy\": [\"adr.0001\", \"adr.0004\"],");
    raw_line("            \"current\": [\"snapshot.current.provider_minimal\", \"capabilities.current_boot\", \"service.inventory.current\", \"problem.list.current\"]");
    raw_line("          },");
    raw_line("          \"current\": {");
    raw_line("            \"os\": {\"name\": \"raiOS\", \"product\": \"raiOS\", \"stage\": \"stage-0\"},");
    raw_line("            \"status\": {");
    emit_status_state_at("framebuffer", status.framebuffer.state, true, 14);
    emit_status_state_at("entropy", status.entropy.state, true, 14);
    emit_status_state_at("usb_xhci", status.usb_xhci.state, true, 14);
    emit_status_state_at("wifi", status.wifi.state, true, 14);
    emit_status_state_at("network", status.network.state, true, 14);
    emit_status_state_at("input", status.input.state, false, 14);
    raw_line("            },");
    raw_line("            \"provider\": {");
    raw("              \"selected\": ");
    json_str(provider.provider_name);
    raw_line(",");
    raw("              \"route\": ");
    json_str(provider.route.as_str());
    raw_line(",");
    raw("              \"api_key_state\": ");
    json_str(if provider.api_key_set {
        "set"
    } else {
        "missing"
    });
    raw_line(",");
    raw("              \"direct_phase\": ");
    json_str(provider.direct_phase);
    raw_line(",");
    raw("              \"direct_endpoint\": ");
    json_str(provider.direct_endpoint);
    raw_line(",");
    raw("              \"direct_model\": ");
    json_str(provider.direct_model);
    raw_line(",");
    raw("              \"trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("              \"pin_kind\": ");
    json_opt_str(provider.trust_pin_kind);
    raw_line(",");
    raw("              \"pin_id\": ");
    json_opt_str(provider.trust_pin_id);
    raw_line(",");
    raw("              \"development_bypass\": ");
    raw_bool(provider.trust_development_bypass);
    crlf();
    raw_line("            },");
    raw_line("            \"services\": [");
    emit_service_ids(14);
    raw_line("            ],");
    raw_line("            \"capabilities\": [");
    emit_capability_ids(14);
    raw_line("            ],");
    raw_line("            \"problems\": [");
    emit_problem_objects(status, provider, 14);
    raw_line("            ]");
    raw_line("          },");
    raw_line("          \"records\": [");
    emit_projection_record(
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 identity",
        true,
    );
    emit_projection_record(
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "current_snapshot",
        "public",
        "provider_minimal projection of current status and provider trust",
        true,
    );
    emit_projection_record(
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
        true,
    );
    emit_projection_record(
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "stable current-boot service ids",
        true,
    );
    emit_projection_record(
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current stable problem ids and severities",
        true,
    );
    emit_projection_record(
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI",
        true,
    );
    emit_projection_record(
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory uses typed local facts and budgeted task-scoped projections",
        false,
    );
    raw_line("          ],");
    raw_line("          \"omitted\": [");
    emit_projection_omission(
        "system.snapshot.raw",
        "local_only",
        "only the provider_minimal projection of selected snapshot fields is included",
        true,
    );
    emit_projection_omission(
        "system.boot_log.raw",
        "local_only",
        "raw boot log is not included",
        true,
    );
    emit_projection_omission(
        "unclassified.memory_context",
        "local_only",
        "unclassified context fields are omitted by default",
        false,
    );
    raw_line("          ]");
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

fn emit_provider_context_gate(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let evidence = provider_context_evidence(&status, &provider);
    let event_hashes = evidence.event_hashes();
    let check = event_log::check_provider_context_binding_gate(event_hashes);

    begin_response("provider.context_gate");
    raw_line("      \"schema\": \"raios.provider_context_export_gate_state.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"gate_state\": {");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("        \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&check, "request"));
    raw_line(",");
    raw("        \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&check, "export"));
    raw_line(",");
    raw("        \"binding_validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"binding_validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"binding_retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    raw("        \"binding_consumed\": ");
    raw_bool(check.consumed);
    raw_line(",");
    raw_line("        \"satisfies_current_boot_export_gate\": false,");
    raw_line("        \"can_export\": false");
    raw_line("      },");
    raw_line("      \"candidate\": {");
    emit_provider_binding_candidate(&check, 8);
    raw_line("      },");
    raw_line("      \"evidence\": {");
    raw_line(
        "        \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
    );
    raw("        \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("        \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("        \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw_line("        \"provider_write_path\": \"disabled\"");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if check.status != "valid" {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            check.status,
            check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("      ]");
    end_response("provider.context_gate");
}

fn emit_provider_context_gate_selftest(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let cases = event_log::provider_context_binding_gate_selftest(evidence.event_hashes());
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("provider.context_gate_selftest");
    raw_line("      \"schema\": \"raios.provider_context_gate_negative_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_provider_request_envelope\": false,");
    raw_line("      \"creates_positive_binding_records\": false,");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"covered_rejections\": [");
    raw_line("        \"missing_export_audit_binding\",");
    raw_line("        \"stale_dropped_request_binding_event_id\",");
    raw_line("        \"stale_dropped_envelope_event_id\",");
    raw_line("        \"previous_boot_or_unretained_event_id\",");
    raw_line("        \"denial_schema_substitution\",");
    raw_line("        \"positive_record_substitution\",");
    raw_line("        \"request_envelope_wrong_variant\",");
    raw_line("        \"request_envelope_event_id_mismatch\",");
    raw_line("        \"request_id_mismatch\",");
    raw_line("        \"request_body_hash_mismatch\",");
    raw_line("        \"request_envelope_hash_mismatch\",");
    raw_line("        \"request_binding_hash_mismatch\",");
    raw_line("        \"provider_minimal_packet_hash_mismatch\",");
    raw_line("        \"exported_field_list_hash_mismatch\",");
    raw_line("        \"omitted_field_list_hash_mismatch\",");
    raw_line("        \"trust_bypass_record\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    let mut case_idx = 0usize;
    while case_idx < cases.len() {
        emit_provider_context_gate_selftest_case(&cases[case_idx], case_idx + 1 != cases.len());
        case_idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_gate_selftest");
}

fn emit_provider_context_gate_selftest_case(
    case: &event_log::ProviderBindingGateSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_provider_context_injection_gate(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let projection_present = profile_supported;
    let evidence = provider_context_evidence(&status, &provider);
    let check = event_log::check_provider_context_binding_gate(evidence.event_hashes());
    let injection_check = event_log::check_provider_context_injection_gate(
        evidence.event_hashes(),
        provider.trust_state,
    );

    begin_response("provider.context_injection_gate");
    raw_line("      \"schema\": \"raios.provider_context_injection_gate.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"gate_state\": {");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("        \"redaction_projection\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"packet_evidence_binding\": \"present\",");
    raw_line("        \"exported_field_list_binding\": \"present\",");
    raw_line("        \"omitted_field_list_binding\": \"present\",");
    raw("        \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&check, "request"));
    raw_line(",");
    raw("        \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&check, "export"));
    raw_line(",");
    raw("        \"binding_validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"binding_validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"binding_retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    raw("        \"binding_consumed\": ");
    raw_bool(check.consumed);
    raw_line(",");
    raw_line(
        "        \"final_authorization_schema\": \"raios.provider_context_injection_authorization.v0\",",
    );
    raw("        \"final_authorization\": ");
    json_str(provider_injection_authorization_state(&injection_check));
    raw_line(",");
    raw("        \"final_authorization_validation_status\": ");
    json_str(injection_check.status);
    raw_line(",");
    raw("        \"final_authorization_validation_reason\": ");
    json_str(injection_check.reason);
    raw_line(",");
    raw("        \"final_authorization_event_id\": ");
    json_event_id_option(injection_check.authorization_event_id);
    raw_line(",");
    raw("        \"binding_consumption_event_id\": ");
    json_event_id_option(injection_check.binding_consumption_event_id);
    raw_line(",");
    raw("        \"final_authorization_retained\": ");
    raw_bool(injection_check.retained);
    raw_line(",");
    raw("        \"final_prewrite_body_check\": ");
    json_str(provider_injection_body_check_state(&injection_check));
    raw_line(",");
    raw("        \"satisfies_current_boot_export_gate\": ");
    raw_bool(injection_check.satisfies_current_boot_export_gate);
    raw_line(",");
    raw("        \"can_attach_context\": ");
    raw_bool(injection_check.can_attach_context);
    crlf();
    raw_line("      },");
    raw_line("      \"candidate\": {");
    emit_provider_binding_candidate(&check, 8);
    raw_line("      },");
    raw_line("      \"evidence\": {");
    raw_line(
        "        \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
    );
    raw("        \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("        \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("        \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw_line("        \"provider_request_body_attachment\": \"blocked_until_final_authorization\"");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if !projection_present {
        emit_export_gate(
            &mut wrote,
            "redaction_projection",
            "missing",
            "provider_minimal_projection_missing",
        );
    }
    if check.status != "valid" {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            check.status,
            check.reason,
        );
    }
    if injection_check.status != "blocked" {
        emit_export_gate(
            &mut wrote,
            "provider_context_injection_authorization",
            injection_check.status,
            injection_check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("      ],");
    raw_line("      \"required\": [");
    raw_line("        \"positive_provider_trust\",");
    raw_line("        \"raios.provider_context_projection.v0\",");
    raw_line("        \"raios.provider_request_binding.v0\",");
    raw_line("        \"raios.provider_context_export_audit_binding.v0\",");
    raw_line("        \"raios.provider_context_binding_consumption.v0\",");
    raw_line("        \"raios.provider_context_injection_authorization.v0\",");
    raw_line("        \"final_prewrite_body_hash_check\"");
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_injection_gate");
}

fn emit_provider_context_injection_gate_selftest(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let cases = event_log::provider_context_injection_gate_selftest(evidence.event_hashes());
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("provider.context_injection_gate_selftest");
    raw_line("      \"schema\": \"raios.provider_context_injection_gate_negative_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_provider_request_envelope\": false,");
    raw_line("      \"creates_positive_binding_records\": false,");
    raw_line("      \"creates_final_authorization_records\": false,");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"can_attach_context\": false,");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"covered_rejections\": [");
    raw_line("        \"missing_final_authorization\",");
    raw_line("        \"stale_dropped_final_authorization_event_id\",");
    raw_line("        \"final_authorization_schema_substitution\",");
    raw_line("        \"substituted_positive_final_authorization_record\",");
    raw_line("        \"final_authorization_body_hash_mismatch\",");
    raw_line("        \"final_authorization_trust_downgrade\",");
    raw_line("        \"body_attachment_without_final_authorization\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    let mut case_idx = 0usize;
    while case_idx < cases.len() {
        emit_provider_context_injection_gate_selftest_case(
            &cases[case_idx],
            case_idx + 1 != cases.len(),
        );
        case_idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_injection_gate_selftest");
}

fn emit_provider_context_injection_gate_selftest_case(
    case: &event_log::ProviderContextInjectionGateSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
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

fn emit_provider_context_export_denied(
    runtime: ui::RuntimeStatus,
    request: &str,
    denial_event_id: event_log::EventId,
) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let projection_present = profile_supported;
    let evidence = provider_context_evidence(&status, &provider);
    let event_hashes = evidence.event_hashes();
    let (binding_check, binding_consumption_event_id) =
        event_log::consume_provider_context_binding_gate(event_hashes);
    let binding_consumed = binding_check.status == "valid";
    let request_binding_denial_event_id = if binding_consumed {
        None
    } else {
        Some(event_log::record_provider_request_binding_denied(
            event_hashes,
        ))
    };
    let export_denial_audit_event_id =
        event_log::record_provider_context_export_denial_audit(event_hashes);

    serial::write_raw_fmt(format_args!(
        "RAIOS_AGENT_BEGIN provider.context_export\r\n"
    ));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw_line("    \"method\": \"provider.context_export\",");
    raw("    \"event_id\": ");
    json_event_id(denial_event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.provider_context_export.v0\",");
    raw("    \"message\": ");
    json_str("provider context export is denied until positive provider trust, a provider_minimal projection, packet evidence, provider request binding, and a provider export audit binding exist");
    raw_line(",");
    raw_line("    \"request\": {");
    raw("      \"provider\": ");
    json_str(provider.provider_name);
    raw_line(",");
    raw("      \"route\": ");
    json_str(provider.route.as_str());
    raw_line(",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"context_schema\": \"raios.agent_context.v0\",");
    raw_line("      \"projection_schema\": \"raios.provider_context_projection.v0\",");
    raw_line("      \"export_schema\": \"raios.provider_context_export.v0\",");
    raw_line("      \"requested_capability\": \"cap.provider.context_export\",");
    raw_line("      \"export_scope\": \"single_provider_request\"");
    raw_line("    },");
    raw_line("    \"gate_state\": {");
    raw("      \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("      \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("      \"redaction_projection\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw("      \"field_classification\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("      \"packet_evidence_binding\": \"present\",");
    raw_line("      \"exported_field_list_binding\": \"present\",");
    raw_line("      \"omitted_field_list_binding\": \"present\",");
    raw("      \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&binding_check, "request"));
    raw_line(",");
    raw("      \"provider_request_binding_denial\": ");
    json_str(if binding_consumed {
        "not_created_positive_binding_consumed"
    } else {
        "present_denied_not_bound"
    });
    raw_line(",");
    raw("      \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&binding_check, "export"));
    raw_line(",");
    raw("      \"provider_binding_consumption\": ");
    json_str(if binding_consumed {
        "consumed_for_gate_evaluation"
    } else {
        "not_consumed"
    });
    raw_line(",");
    raw("      \"binding_validation_status\": ");
    json_str(binding_check.status);
    raw_line(",");
    raw("      \"binding_validation_reason\": ");
    json_str(binding_check.reason);
    raw_line(",");
    raw_line("      \"provider_export_denial_audit\": \"present_denied_no_provider_write\",");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"can_export\": false");
    raw_line("    },");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        raw_line("    \"provider_request_binding_denial\": {");
        raw_line("      \"schema\": \"raios.provider_request_binding_denial.v0\",");
        raw("      \"id\": ");
        json_current_boot_id(
            "provider_request_binding_denial.current_boot",
            request_binding_denial_event_id,
        );
        raw_line(",");
        raw("      \"attempted_request_id\": ");
        json_current_boot_id(
            "provider_request_attempt.current_boot",
            request_binding_denial_event_id,
        );
        raw_line(",");
        raw("      \"event_id\": ");
        json_event_id(request_binding_denial_event_id);
        raw_line(",");
        raw_line("      \"status\": \"denied_not_bound\",");
        raw_line("      \"satisfies_export_gate\": false,");
        raw("      \"provider\": ");
        json_str(provider.provider_name);
        raw_line(",");
        raw("      \"route\": ");
        json_str(provider.route.as_str());
        raw_line(",");
        raw("      \"profile\": ");
        json_str(profile);
        raw_line(",");
        raw_line("      \"classification\": \"public\",");
        raw_line("      \"context_schema\": \"raios.agent_context.v0\",");
        raw_line(
            "      \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
        );
        raw("      \"projected_packet_hash\": ");
        json_sha256(evidence.projected_packet_hash);
        raw_line(",");
        raw("      \"exported_field_list_hash\": ");
        json_sha256(evidence.exported_field_list_hash);
        raw_line(",");
        raw("      \"omitted_field_list_hash\": ");
        json_sha256(evidence.omitted_field_list_hash);
        raw_line(",");
        raw_line("      \"provider_write\": \"not_attempted\"");
        raw_line("    },");
    } else {
        raw_line("    \"provider_request_binding_denial\": null,");
    }
    raw_line("    \"provider_binding_consumption\": {");
    raw_line("      \"schema\": \"raios.provider_context_binding_consumption.v0\",");
    raw("      \"event_id\": ");
    json_event_id_option(binding_consumption_event_id);
    raw_line(",");
    raw("      \"status\": ");
    json_str(if binding_consumed {
        "consumed_for_gate_evaluation"
    } else {
        "not_consumed"
    });
    raw_line(",");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"reason\": ");
    json_str(if binding_consumed {
        "provider_binding_consumed_without_body_attachment"
    } else {
        binding_check.reason
    });
    crlf();
    raw_line("    },");
    raw_line("    \"export_denial_audit\": {");
    raw_line("      \"schema\": \"raios.provider_context_export_denial_audit.v0\",");
    raw("      \"id\": ");
    json_current_boot_id(
        "provider_context_export_denial_audit.current_boot",
        export_denial_audit_event_id,
    );
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("      \"status\": \"denied_no_provider_write\",");
    raw_line("      \"satisfies_export_gate\": false,");
    raw_line("      \"classification\": \"public\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"positive_export_authorization\": false,");
    raw("      \"denial_event_id\": ");
    json_event_id(denial_event_id);
    raw_line(",");
    raw_line("      \"provider_write\": \"not_attempted\"");
    raw_line("    },");
    raw_line("    \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if !projection_present {
        emit_export_gate(
            &mut wrote,
            "redaction_projection",
            "missing",
            "provider_minimal_projection_missing",
        );
    }
    if !binding_consumed && binding_check.status == "missing" {
        emit_export_gate(
            &mut wrote,
            "provider_request_binding",
            "missing",
            "provider_request_binding_missing",
        );
        emit_export_gate(
            &mut wrote,
            "provider_context_export_audit_binding",
            "missing",
            "provider_context_export_audit_binding_missing",
        );
    } else if !binding_consumed {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            binding_check.status,
            binding_check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"positive_provider_trust\",");
    raw_line("      \"raios.provider_context_projection.v0\",");
    raw_line("      \"raios.provider_context_export.v0\",");
    raw_line("      \"projected_packet_hash\",");
    raw_line("      \"exported_field_list_hash\",");
    raw_line("      \"omitted_field_list_hash\",");
    raw_line("      \"provider_request_binding\",");
    raw_line("      \"provider_context_export_audit_binding\",");
    raw_line("      \"checked_current_boot_binding_consumption\",");
    raw_line("      \"audit.event.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw_line("      \"local_projection_method\": \"memory.context provider_minimal\",");
    raw_line("      \"local_projection_locator\": \"snapshot.current.provider_minimal\",");
    raw_line("      \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",");
    raw("      \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("      \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("      \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw("      \"provider_request_binding_status\": ");
    json_str(provider_binding_gate_state(&binding_check, "request"));
    raw_line(",");
    raw("      \"provider_request_binding_event_id\": ");
    json_event_id_option(binding_check.request_binding_event_id);
    raw_line(",");
    raw("      \"provider_request_binding_denial_id\": ");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        json_current_boot_id(
            "provider_request_binding_denial.current_boot",
            request_binding_denial_event_id,
        );
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"provider_request_binding_denial_event_id\": ");
    json_event_id_option(request_binding_denial_event_id);
    raw_line(",");
    raw("      \"provider_request_attempt_id\": ");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        json_current_boot_id(
            "provider_request_attempt.current_boot",
            request_binding_denial_event_id,
        );
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"export_audit_binding_status\": ");
    json_str(provider_binding_gate_state(&binding_check, "export"));
    raw_line(",");
    raw("      \"export_audit_binding_event_id\": ");
    json_event_id_option(binding_check.export_audit_binding_event_id);
    raw_line(",");
    raw("      \"binding_consumption_event_id\": ");
    json_event_id_option(binding_consumption_event_id);
    raw_line(",");
    raw("      \"export_denial_audit_id\": ");
    json_current_boot_id(
        "provider_context_export_denial_audit.current_boot",
        export_denial_audit_event_id,
    );
    raw_line(",");
    raw("      \"export_denial_audit_event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("      \"export_denial_audit_satisfies_export_gate\": false,");
    raw_line("      \"denial_event_is_export_binding\": false,");
    raw("      \"denial_event_id\": ");
    json_event_id(denial_event_id);
    crlf();
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END provider.context_export\r\n"));
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

fn emit_provider_context_hashes(hashes: event_log::ProviderContextHashes) {
    raw("{\"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\", \"projected_packet_hash\": ");
    json_sha256(hashes.projected_packet_hash);
    raw(", \"exported_field_list_hash\": ");
    json_sha256(hashes.exported_field_list_hash);
    raw(", \"omitted_field_list_hash\": ");
    json_sha256(hashes.omitted_field_list_hash);
    raw("}");
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

fn emit_projection_record(
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
    comma: bool,
) {
    indent(12);
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
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_projection_omission(
    field: &'static str,
    classification: &'static str,
    reason: &'static str,
    comma: bool,
) {
    indent(10);
    raw("{\"field\": ");
    json_str(field);
    raw(", \"classification\": ");
    json_str(classification);
    raw(", \"action\": \"omit\", \"reason\": ");
    json_str(reason);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_projection_field_specs(fields: &[ProjectionFieldSpec], spaces: usize) {
    let mut idx = 0usize;
    while idx < fields.len() {
        emit_projection_field_spec(&fields[idx], idx + 1 != fields.len(), spaces);
        idx += 1;
    }
}

fn emit_projection_field_spec(spec: &ProjectionFieldSpec, comma: bool, spaces: usize) {
    indent(spaces);
    raw("{\"field\": ");
    json_str(spec.field);
    raw(", \"classification\": ");
    json_str(spec.classification);
    raw(", \"action\": ");
    json_str(spec.action);
    raw(", \"reason\": ");
    json_str(spec.reason);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn provider_binding_gate_state(
    check: &event_log::ProviderBindingGateCheck,
    _kind: &'static str,
) -> &'static str {
    if check.status == "valid" {
        "present_validated"
    } else if check.reason == "binding_already_consumed" {
        "consumed"
    } else if check.retained {
        "rejected"
    } else {
        "missing"
    }
}

fn provider_injection_authorization_state(
    check: &event_log::ProviderContextInjectionGateCheck,
) -> &'static str {
    if check.authorization_event_id.is_none() {
        "missing"
    } else if check.status == "blocked" {
        "present_blocked"
    } else {
        "present_rejected"
    }
}

fn provider_injection_body_check_state(
    check: &event_log::ProviderContextInjectionGateCheck,
) -> &'static str {
    if check.reason == "final_prewrite_body_hash_mismatch" {
        "mismatch"
    } else if check.authorization_event_id.is_none() || check.status != "blocked" {
        "not_attempted"
    } else {
        "matched"
    }
}

fn emit_provider_binding_candidate(check: &event_log::ProviderBindingGateCheck, spaces: usize) {
    indent(spaces);
    raw("\"request_binding_event_id\": ");
    json_event_id_option(check.request_binding_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"export_audit_binding_event_id\": ");
    json_event_id_option(check.export_audit_binding_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"request_envelope_event_id\": ");
    json_event_id_option(check.request_envelope_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"request_id\": ");
    if let Some(binding) = check.export_audit_binding {
        raw_fmt(format_args!("{}", binding.request_id));
    } else if let Some(binding) = check.request_binding {
        raw_fmt(format_args!("{}", binding.request_id));
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"request_binding_hash\": ");
    if let Some(binding) = check.request_binding {
        json_sha256(binding.request_binding_hash);
    } else if let Some(binding) = check.export_audit_binding {
        json_sha256(binding.request_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"export_audit_binding_hash\": ");
    if let Some(binding) = check.export_audit_binding {
        json_sha256(binding.export_audit_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    indent(spaces);
    raw("\"consumed\": ");
    raw_bool(check.consumed);
    crlf();
}

fn provider_context_evidence(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> ProviderContextEvidence {
    ProviderContextEvidence {
        projected_packet_hash: provider_minimal_packet_hash(status, provider),
        exported_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.exported_fields.canonical.v0",
            PROVIDER_MINIMAL_INCLUDED_FIELDS,
        ),
        omitted_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.omitted_fields.canonical.v0",
            PROVIDER_MINIMAL_OMITTED_FIELDS,
        ),
    }
}

fn provider_minimal_packet_hash(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> [u8; 32] {
    let mut hash = EvidenceHash::new("raios.provider_minimal.packet.canonical.v0");
    hash.field("schema", "raios.agent_context.v0");
    hash.field("purpose", "current_boot_provider_context");
    hash.field("profile", "provider_minimal");
    hash.field("scope", "current_boot");
    hash.field("budget.target_tokens", "2000");
    hash.field("budget.estimated_tokens", "900");
    hash.array(
        "authority_order",
        &["current_snapshot", "decision", "service_state", "summary"],
    );
    hash.array("included.identity", &["mem.fact.identity.stage0"]);
    hash.array("included.policy", &["adr.0001", "adr.0004"]);
    hash.array(
        "included.current",
        &[
            "snapshot.current.provider_minimal",
            "capabilities.current_boot",
            "service.inventory.current",
            "problem.list.current",
        ],
    );
    hash.field("current.os.name", "raiOS");
    hash.field("current.os.product", "raiOS");
    hash.field("current.os.stage", "stage-0");
    hash_status(&mut hash, status);
    hash_provider(&mut hash, provider);
    hash_services(&mut hash);
    hash_capabilities(&mut hash);
    hash_problems(&mut hash, status, provider);
    hash_projection_records(&mut hash);
    hash.array(
        "packet.omitted",
        &[
            "system.snapshot.raw",
            "system.boot_log.raw",
            "unclassified.memory_context",
        ],
    );
    hash.finish()
}

fn projection_field_list_hash(domain: &'static str, fields: &[ProjectionFieldSpec]) -> [u8; 32] {
    let mut hash = EvidenceHash::new(domain);
    let mut idx = 0usize;
    while idx < fields.len() {
        hash.field("field", fields[idx].field);
        hash.field("classification", fields[idx].classification);
        hash.field("action", fields[idx].action);
        hash.field("reason", fields[idx].reason);
        hash.separator();
        idx += 1;
    }
    hash.finish()
}

fn hash_status(hash: &mut EvidenceHash, status: &SystemSnapshot) {
    hash.field(
        "current.status.framebuffer",
        status.framebuffer.state.as_protocol(),
    );
    hash.field("current.status.entropy", status.entropy.state.as_protocol());
    hash.field(
        "current.status.usb_xhci",
        status.usb_xhci.state.as_protocol(),
    );
    hash.field("current.status.wifi", status.wifi.state.as_protocol());
    hash.field("current.status.network", status.network.state.as_protocol());
    hash.field("current.status.input", status.input.state.as_protocol());
}

fn hash_provider(hash: &mut EvidenceHash, provider: &provider::Snapshot) {
    hash.field("current.provider.selected", provider.provider_name);
    hash.field("current.provider.route", provider.route.as_str());
    hash.field(
        "current.provider.api_key_state",
        if provider.api_key_set {
            "set"
        } else {
            "missing"
        },
    );
    hash.field("current.provider.direct_phase", provider.direct_phase);
    hash.field("current.provider.direct_endpoint", provider.direct_endpoint);
    hash.field("current.provider.direct_model", provider.direct_model);
    hash.field("current.provider.trust_state", provider.trust_state);
    hash.opt_field("current.provider.pin_kind", provider.trust_pin_kind);
    hash.opt_field("current.provider.pin_id", provider.trust_pin_id);
    hash.bool_field(
        "current.provider.development_bypass",
        provider.trust_development_bypass,
    );
}

fn hash_services(hash: &mut EvidenceHash) {
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        hash.field("current.services[]", service_inventory::SERVICES[idx].id);
        idx += 1;
    }
}

fn hash_capabilities(hash: &mut EvidenceHash) {
    let mut idx = 0usize;
    while idx < CAPABILITIES.len() {
        if CAPABILITIES[idx].granted {
            hash.field("current.capabilities[]", CAPABILITIES[idx].id);
        }
        idx += 1;
    }
    hash.field(
        "current.capabilities[]",
        "capability_denied.for_all_mutating_methods",
    );
}

fn hash_problems(hash: &mut EvidenceHash, status: &SystemSnapshot, provider: &provider::Snapshot) {
    let mut wrote = false;
    hash_provider_trust_problem(hash, &mut wrote, provider);
    if !provider.api_key_set {
        hash_problem(
            hash,
            &mut wrote,
            "provider.openai.api_key_missing",
            "info",
            "OpenAI direct requests need a RAM-only API key",
        );
    }
    hash_status_problem(
        hash,
        &mut wrote,
        "framebuffer.unavailable",
        "error",
        "Limine framebuffer is unavailable",
        &status.framebuffer,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "entropy.not_ready",
        "warning",
        "Entropy is not ready yet",
        &status.entropy,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "usb_xhci.unavailable",
        "warning",
        "xHCI USB path is missing or degraded",
        &status.usb_xhci,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "network.unavailable",
        "warning",
        "e1000/IPv4 network path is not configured",
        &status.network,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "input.unavailable",
        "warning",
        "keyboard or pointer input is missing",
        &status.input,
    );
    match wifi::snapshot().state {
        wifi::WifiState::Detected => hash_problem(
            hash,
            &mut wrote,
            "wifi.avastar.firmware_todo",
            "info",
            "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        ),
        wifi::WifiState::Missing => hash_problem(
            hash,
            &mut wrote,
            "wifi.avastar.target_absent",
            "info",
            "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        ),
        wifi::WifiState::NotProbed => {}
    }
    if !wrote {
        hash_problem(
            hash,
            &mut wrote,
            "none",
            "info",
            "no known protocol problems reported",
        );
    }
}

fn hash_provider_trust_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    provider: &provider::Snapshot,
) {
    let (id, summary) = match provider.trust_state {
        "unknown" => (
            "provider.tls_unknown",
            "OpenAI provider trust has not been established",
        ),
        "tls_certificate_verification_bypassed" => (
            "provider.tls_unverified",
            "OpenAI direct transport is using an explicit unverified TLS development override",
        ),
        "pin_config_missing" => (
            "provider.tls_pin_config_missing",
            "OpenAI direct transport is fail-closed until a provider pin is configured",
        ),
        "pin_config_invalid" => (
            "provider.tls_pin_config_invalid",
            "Configured OpenAI provider pin is invalid",
        ),
        "pin_verifier_unavailable" => (
            "provider.tls_pin_verifier_unavailable",
            "Configured OpenAI provider pin cannot be checked until TLS verifier input access exists",
        ),
        "pin_mismatch" => (
            "provider.tls_pin_mismatch",
            "OpenAI provider certificate did not match the configured pin",
        ),
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => return,
        _ => (
            "provider.tls_unknown",
            "OpenAI provider trust state is not recognized by this protocol build",
        ),
    };
    hash_problem(hash, wrote, id, "high", summary);
}

fn hash_status_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    id: &'static str,
    severity: &'static str,
    summary: &'static str,
    line: &system_status::StatusLine,
) {
    if matches!(
        line.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        return;
    }
    hash_problem(hash, wrote, id, severity, summary);
}

fn hash_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    id: &'static str,
    severity: &'static str,
    summary: &'static str,
) {
    hash.field("current.problems[].id", id);
    hash.field("current.problems[].severity", severity);
    hash.field("current.problems[].summary", summary);
    hash.separator();
    *wrote = true;
}

fn hash_projection_records(hash: &mut EvidenceHash) {
    hash_projection_record(
        hash,
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 identity",
    );
    hash_projection_record(
        hash,
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "current_snapshot",
        "public",
        "provider_minimal projection of current status and provider trust",
    );
    hash_projection_record(
        hash,
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
    );
    hash_projection_record(
        hash,
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "stable current-boot service ids",
    );
    hash_projection_record(
        hash,
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current stable problem ids and severities",
    );
    hash_projection_record(
        hash,
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI",
    );
    hash_projection_record(
        hash,
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory uses typed local facts and budgeted task-scoped projections",
    );
}

fn hash_projection_record(
    hash: &mut EvidenceHash,
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
) {
    hash.field("records[].id", id);
    hash.field("records[].kind", kind);
    hash.field("records[].authority", authority);
    hash.field("records[].classification", classification);
    hash.field("records[].summary", summary);
    hash.separator();
}

struct EvidenceHash {
    hasher: Sha256,
}

impl EvidenceHash {
    fn new(domain: &'static str) -> Self {
        let mut value = Self {
            hasher: Sha256::new(),
        };
        value.field("domain", domain);
        value
    }

    fn field(&mut self, name: &str, value: &str) {
        self.hasher.update(name.as_bytes());
        self.hasher.update(b"=");
        self.hasher.update(value.as_bytes());
        self.hasher.update(b"\n");
    }

    fn bool_field(&mut self, name: &str, value: bool) {
        self.field(name, if value { "true" } else { "false" });
    }

    fn opt_field(&mut self, name: &str, value: Option<&str>) {
        self.field(name, value.unwrap_or("null"));
    }

    fn array(&mut self, name: &str, values: &[&str]) {
        let mut idx = 0usize;
        while idx < values.len() {
            self.field(name, values[idx]);
            idx += 1;
        }
        self.separator();
    }

    fn separator(&mut self) {
        self.hasher.update(b"--\n");
    }

    fn finish(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
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

fn provider_context_export_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_export")
        || method_head_eq(method, "provider.export_context")
}

fn provider_context_gate_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_gate")
        || method_head_eq(method, "provider.context_export_status")
}

fn provider_context_gate_selftest_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_gate_selftest")
}

fn provider_context_injection_gate_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_injection_gate")
}

fn provider_context_injection_gate_selftest_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_injection_gate_selftest")
}

fn provider_context_export_profile(method: &str) -> &'static str {
    let arg = provider_context_export_arg(method);
    if method_eq(arg, "provider_minimal") || arg.is_empty() {
        "provider_minimal"
    } else {
        "unsupported"
    }
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

fn provider_trust_positive(trust_state: &str) -> bool {
    matches!(
        trust_state,
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified"
    )
}

fn provider_context_block_reason(trust_state: &str) -> &'static str {
    if provider_trust_positive(trust_state) {
        "provider_context_export_audit_binding_missing"
    } else {
        "provider_trust_not_positive"
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

fn provider_context_export_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "provider.context_export") {
        "provider.context_export".len()
    } else if method_head_eq(method, "provider.export_context") {
        "provider.export_context".len()
    } else if method_head_eq(method, "provider.context_gate") {
        "provider.context_gate".len()
    } else if method_head_eq(method, "provider.context_export_status") {
        "provider.context_export_status".len()
    } else if method_head_eq(method, "provider.context_gate_selftest") {
        "provider.context_gate_selftest".len()
    } else if method_head_eq(method, "provider.context_injection_gate") {
        "provider.context_injection_gate".len()
    } else if method_head_eq(method, "provider.context_injection_gate_selftest") {
        "provider.context_injection_gate_selftest".len()
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
