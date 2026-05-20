use sha2::{Digest, Sha256};

use crate::{
    agent_protocol_module_audit::{
        emit_module_audit_rollback_diagnostic, emit_module_audit_rollback_diagnostic_selftest,
        module_audit_rollback_diagnostic_method, module_audit_rollback_diagnostic_selftest_method,
    },
    agent_protocol_module_grant::{
        emit_module_grant_diagnostic, emit_module_grant_diagnostic_selftest,
        module_computed_grant_reference_hashes_consistent, module_computed_grant_reference_matches,
        module_grant_diagnostic_method, module_grant_diagnostic_selftest_method,
    },
    agent_protocol_module_service_slot::{
        emit_module_service_slot_diagnostic, emit_module_service_slot_diagnostic_selftest,
        module_service_slot_diagnostic_method, module_service_slot_diagnostic_selftest_method,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, emit_export_gate,
        emit_inline_string_array, emit_static_string_array, end_response, indent,
        json_current_boot_id, json_event_id, json_event_id_option, json_event_sequence,
        json_opt_str, json_sha256, json_sha256_option, json_str, method_eq, method_head_eq,
        parse_current_boot_event_id, parse_sha256_ref, raw, raw_bool, raw_fmt, raw_line,
    },
    event_log,
    module_evidence::{
        self, ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput,
        ModuleVmTestReportReferenceHashInput,
    },
    provider, serial, service_inventory, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, wifi,
};

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

struct Capability {
    id: &'static str,
    risk: &'static str,
    granted: bool,
    scope: &'static str,
    summary: &'static str,
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

const CAPABILITIES: &[Capability] = &[
    Capability {
        id: "cap.system.describe.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read protocol and OS identity",
    },
    Capability {
        id: "cap.system.snapshot.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read typed Stage-0 status facts",
    },
    Capability {
        id: "cap.system.boot_log.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read the local serial boot log ring",
    },
    Capability {
        id: "cap.system.capabilities.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read the current capability catalog",
    },
    Capability {
        id: "cap.service.inventory.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read static service inventory",
    },
    Capability {
        id: "cap.device.graph.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read known device graph",
    },
    Capability {
        id: "cap.problem.list.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read known local problems and gaps",
    },
    Capability {
        id: "cap.memory.profile.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read available memory context profiles",
    },
    Capability {
        id: "cap.memory.context.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read bounded current-boot agent context",
    },
    Capability {
        id: "cap.memory.query.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "query current-boot memory record ids",
    },
    Capability {
        id: "cap.memory.trace.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "trace current-boot memory records to source evidence",
    },
    Capability {
        id: "cap.memory.recent_events.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read bounded current-boot memory event records",
    },
    Capability {
        id: "cap.audit.events.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read bounded current-boot audit event records",
    },
    Capability {
        id: "cap.provider.context_export.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read provider context gate diagnostics and local selftests",
    },
    Capability {
        id: "cap.provider.context_injection.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read final provider context injection gate diagnostics",
    },
    Capability {
        id: "cap.module.grant_diagnostic.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read module grant, audit, rollback, and denied-load diagnostics",
    },
    Capability {
        id: "cap.memory.mutate",
        risk: "persist",
        granted: false,
        scope: "denied_until_event_log_audit_policy_persistence_and_rollback",
        summary: "write, supersede, compact, redact, or persist memory records",
    },
    Capability {
        id: "cap.provider.context_export",
        risk: "export",
        granted: false,
        scope: "denied_until_positive_provider_trust_projection_and_export_audit_binding",
        summary: "attach redacted system context to a provider request",
    },
    Capability {
        id: "cap.module.propose",
        risk: "modify_ram",
        granted: false,
        scope: "denied_until_manifest_v0",
        summary: "submit module manifest proposal",
    },
    Capability {
        id: "cap.module.load_ephemeral",
        risk: "modify_ram",
        granted: false,
        scope: "denied_until_vm_test_report_and_attestation",
        summary: "load current-boot-only artifact",
    },
    Capability {
        id: "cap.module.persist",
        risk: "persist",
        granted: false,
        scope: "denied_until_layout_vm_test_report_attestation_approval",
        summary: "change persistent boot/module state",
    },
    Capability {
        id: "cap.module.rollback",
        risk: "persist",
        granted: false,
        scope: "denied_until_rollback_state_exists",
        summary: "roll back persistent or live module state",
    },
    Capability {
        id: "cap.config.apply",
        risk: "persist",
        granted: false,
        scope: "denied_until_policy_v0",
        summary: "apply persistent device or provider configuration",
    },
];

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

const READ_METHODS: &[&str] = &[
    "system.describe",
    "system.snapshot",
    "system.capabilities",
    "system.boot_log",
    "device.graph",
    "problem.list",
    "service.inventory",
    "memory.profile",
    "memory.context",
    "memory.query",
    "memory.trace",
    "memory.recent_events",
    "audit.events",
    "provider.context_gate",
    "provider.context_gate_selftest",
    "provider.context_injection_gate",
    "provider.context_injection_gate_selftest",
    "module.manifest_diagnostic",
    "module.manifest_diagnostic_selftest",
    "module.artifact_diagnostic",
    "module.artifact_diagnostic_selftest",
    "module.vm_report_diagnostic",
    "module.vm_report_diagnostic_selftest",
    "module.grant_diagnostic",
    "module.grant_diagnostic_selftest",
    "module.audit_rollback_diagnostic",
    "module.audit_rollback_diagnostic_selftest",
    "module.service_slot_diagnostic",
    "module.service_slot_diagnostic_selftest",
    "module.load_gate_manifest_selftest",
    "module.load_gate_artifact_selftest",
    "module.load_gate_vm_report_selftest",
    "module.load_gate_retained_selftest",
    "module.load_gate_audit_rollback_selftest",
    "module.load_gate_service_slot_selftest",
];

const DENIED_METHODS: &[&str] = &[
    "memory.record_observation",
    "memory.propose_policy",
    "memory.supersede_fact",
    "memory.redact",
    "memory.compact",
    "provider.context_export",
    "module.propose",
    "module.build_result",
    "module.test_request",
    "module.test_result",
    "module.load_ephemeral",
    "module.persist",
    "module.rollback",
    "service.load_ephemeral",
    "service.restart",
    "service.start",
    "service.stop",
    "config.apply",
    "apply_config",
    "provider.configure",
    "wifi.configure",
    "draw_text",
    "probe_device",
    "download_signed_module",
    "run_module_test",
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

fn emit_describe() {
    begin_response("system.describe");
    raw_line("      \"schema\": \"system.describe.v0\",");
    raw_line(
        "      \"os\": {\"name\": \"raiOS\", \"product\": \"raiOS\", \"stage\": \"stage-0\"},",
    );
    raw_line("      \"protocol\": {");
    raw_line("        \"version\": \"raios.agent.v0\",");
    raw_line("        \"transport\": \"serial-console\",");
    raw_line(
        "        \"provider_context_injection\": \"disabled_until_final_injection_authorization\",",
    );
    raw_line("        \"mutation_policy\": \"denied_by_default\"");
    raw_line("      },");
    raw_line("      \"methods\": [");
    emit_static_string_array(READ_METHODS, 8);
    raw_line("      ],");
    raw_line("      \"denied_methods\": [");
    emit_static_string_array(DENIED_METHODS, 8);
    raw_line("      ]");
    end_response("system.describe");
}

fn emit_snapshot(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();

    begin_response("system.snapshot");
    raw_line("      \"schema\": \"system.snapshot.v0\",");
    raw_line(
        "      \"os\": {\"name\": \"raiOS\", \"product\": \"raiOS\", \"stage\": \"stage-0\"},",
    );
    raw_line("      \"status\": {");
    emit_status_state("framebuffer", status.framebuffer.state, true);
    emit_status_state("entropy", status.entropy.state, true);
    emit_status_state("usb_xhci", status.usb_xhci.state, true);
    emit_status_state("wifi", status.wifi.state, true);
    emit_status_state("network", status.network.state, true);
    emit_status_state("input", status.input.state, false);
    raw_line("      },");
    raw_line("      \"details\": {");
    emit_status_detail("framebuffer", &status.framebuffer, true);
    emit_status_detail("entropy", &status.entropy, true);
    emit_status_detail("usb_xhci", &status.usb_xhci, true);
    emit_status_detail("wifi", &status.wifi, true);
    emit_status_detail("network", &status.network, true);
    emit_status_detail("input", &status.input, false);
    raw_line("      },");
    emit_provider_object(&provider, true);
    raw_line("      \"capabilities\": [");
    emit_capability_ids(8);
    raw_line("      ],");
    raw_line("      \"problems\": [");
    emit_problem_objects(&status, &provider, 8);
    raw_line("      ]");
    end_response("system.snapshot");
}

fn emit_capabilities() {
    begin_response("system.capabilities");
    raw_line("      \"schema\": \"system.capabilities.v0\",");
    raw_line("      \"capabilities\": [");
    let mut idx = 0usize;
    while idx < CAPABILITIES.len() {
        let cap = &CAPABILITIES[idx];
        indent(8);
        raw("{");
        raw("\"id\": ");
        json_str(cap.id);
        raw(", \"risk\": ");
        json_str(cap.risk);
        raw(", \"granted\": ");
        raw_bool(cap.granted);
        raw(", \"scope\": ");
        json_str(cap.scope);
        raw(", \"summary\": ");
        json_str(cap.summary);
        raw("}");
        if idx + 1 != CAPABILITIES.len() {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("      ]");
    end_response("system.capabilities");
}

fn emit_boot_log() {
    let log = serial::log_snapshot();
    begin_response("system.boot_log");
    raw_line("      \"schema\": \"system.boot_log.v0\",");
    raw_line("      \"source\": \"serial_ring\",");
    raw_line("      \"lines\": [");
    let mut wrote = false;
    let mut idx = 0usize;
    while idx < log.lines.len() {
        let line = log.lines[idx].as_str();
        if !line.is_empty() {
            if wrote {
                raw_line(",");
            }
            indent(8);
            raw("{\"index\": ");
            raw_fmt(format_args!("{}", idx));
            raw(", \"text\": ");
            json_str(line);
            raw("}");
            wrote = true;
        }
        idx += 1;
    }
    if wrote {
        crlf();
    }
    raw_line("      ]");
    end_response("system.boot_log");
}

fn emit_device_graph(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    begin_response("device.graph");
    raw_line("      \"schema\": \"device.graph.v0\",");
    raw_line("      \"devices\": [");
    emit_device(
        "framebuffer.limine",
        "framebuffer",
        &status.framebuffer,
        true,
    );
    emit_device("entropy.rdrand", "entropy_source", &status.entropy, true);
    emit_device("usb.xhci", "bus_controller", &status.usb_xhci, true);
    emit_device(
        "wifi.avastar_88w8897",
        "pci_wifi_target",
        &status.wifi,
        true,
    );
    emit_device("net.e1000", "pci_nic", &status.network, true);
    emit_device("input.console", "input", &status.input, false);
    raw_line("      ]");
    end_response("device.graph");
}

fn emit_problem_list(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    begin_response("problem.list");
    raw_line("      \"schema\": \"problem.list.v0\",");
    raw_line("      \"problems\": [");
    emit_problem_objects(&status, &provider, 8);
    raw_line("      ]");
    end_response("problem.list");
}

fn emit_service_inventory(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    begin_response("service.inventory");
    raw_line("      \"schema\": \"service.inventory.v0\",");
    raw_line("      \"services\": [");
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        let service = &service_inventory::SERVICES[idx];
        let health = service_inventory::service_health(service, &status, &provider);
        indent(8);
        raw("{");
        raw("\"id\": ");
        json_str(service.id);
        raw(", \"kind\": ");
        json_str(service.kind);
        raw(", \"health\": ");
        json_str(health.state);
        raw(", \"replaceable\": ");
        raw_bool(service.replaceable);
        raw(", \"core_owned\": ");
        raw_bool(service.core_owned);
        raw(", \"last_error\": ");
        match health.last_error {
            Some(error) => json_str(error),
            None => raw("null"),
        }
        raw(", \"capabilities\": [");
        emit_inline_string_array(service.capabilities);
        raw("]}");
        if idx + 1 != service_inventory::SERVICES.len() {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("      ]");
    end_response("service.inventory");
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

fn emit_module_manifest_diagnostic(method: &str) {
    let arg = module_manifest_diagnostic_arg(method);
    let check = parse_module_manifest_reference(arg);
    let recorded_event_id = if check.valid {
        module_manifest_binding_from_check(&check).map(event_log::record_module_manifest_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_manifest_reference();

    begin_response("module.manifest_diagnostic");
    raw_line("      \"schema\": \"raios.module_manifest_reference_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.manifest_diagnostic <manifest_reference_hash> <manifest_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"manifest_schema\": \"raios.module_manifest.v0\",");
    raw_line("        \"manifest_reference_schema\": \"raios.module_manifest_reference.v0\",");
    raw_line("        \"manifest_reference_canonicalization\": \"raios.module_manifest_reference.canonical.v0\"");
    raw_line("      },");
    emit_module_manifest_reference_object(&check);
    raw_line(",");
    emit_module_manifest_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_manifest_gate_state(&check);
    raw_line(",");
    emit_module_manifest_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "module_manifest", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "candidate_artifact",
        "missing",
        "candidate_artifact_missing",
    );
    emit_export_gate(
        &mut wrote,
        "vm_test_report",
        "missing",
        "vm_test_report_missing",
    );
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
    emit_export_gate(
        &mut wrote,
        "computed_capability_grant",
        "missing",
        "computed_capability_grant_missing",
    );
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "missing",
        "durable_audit_record_missing",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "missing",
        "rollback_plan_missing",
    );
    emit_export_gate(
        &mut wrote,
        "loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.manifest_diagnostic");
}

fn emit_module_manifest_reference_object(check: &ModuleManifestReferenceCheck<'_>) {
    raw_line("      \"module_manifest_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"manifest_schema\": \"raios.module_manifest.v0\",");
    raw("        \"manifest_reference_hash\": ");
    json_sha256_option(check.manifest_reference_hash);
    raw_line(",");
    raw("        \"expected_manifest_reference_hash\": ");
    json_sha256_option(check.expected_manifest_reference_hash);
    raw_line(",");
    raw("        \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    crlf();
    raw_line("      }");
}

fn emit_module_manifest_retained_reference(
    check: &ModuleManifestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleManifestReference)>,
) {
    raw_line("      \"retained_manifest_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(module_manifest_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_manifest_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_manifest_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"accepts_unsigned_service_code\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw_line("        \"hashes\": {");
        raw("          \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("          \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_manifest_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_module_manifest_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_manifest_gate_state(check: &ModuleManifestReferenceCheck<'_>) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    raw_line("      \"gate_state\": {");
    raw("        \"module_manifest\": ");
    json_str(state);
    raw_line(",");
    raw_line("        \"candidate_artifact\": \"missing\",");
    raw_line("        \"vm_test_report\": \"missing\",");
    raw_line("        \"local_attestation\": \"missing\",");
    raw_line("        \"computed_capability_grant\": \"missing\",");
    raw_line("        \"local_approval\": \"missing\",");
    raw_line("        \"rollback_plan\": \"missing\",");
    raw_line("        \"durable_audit_record\": \"missing\",");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"artifact_loaded\": false,");
    raw_line("        \"service_started\": false,");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"can_load\": false");
    raw("      }");
}

fn emit_module_manifest_policy_result(check: &ModuleManifestReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"manifest_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_manifest_json_or_artifact_bytes\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"candidate_artifact_sha256\",");
    raw_line("          \"raios.vm_test_report.v0\",");
    raw_line("          \"raios.local_attestation.v0\",");
    raw_line("          \"raios.computed_capability_grant.v0\",");
    raw_line("          \"raios.audit_record.v0\",");
    raw_line("          \"rollback_plan\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot\"");
    raw_line("        ]");
    raw("      }");
}

fn emit_module_manifest_diagnostic_selftest() {
    let cases = module_manifest_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.manifest_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_manifest_reference_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_manifest_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_manifest_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.manifest_diagnostic_selftest");
}

fn emit_module_manifest_selftest_case(case: &ModuleManifestSelfTestCase, comma: bool) {
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_artifact_diagnostic(method: &str) {
    let arg = module_artifact_diagnostic_arg(method);
    let check = parse_module_artifact_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_artifact_binding_from_check(&check)
            .map(event_log::record_module_candidate_artifact_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_candidate_artifact_reference();

    begin_response("module.artifact_diagnostic");
    raw_line("      \"schema\": \"raios.module_candidate_artifact_reference_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.artifact_diagnostic <artifact_reference_hash> <retained_manifest_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <manifest_hash> <computed_grant_hash> <artifact_hash> <vm_report_hash> <local_attestation_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line(
        "        \"artifact_reference_schema\": \"raios.module_candidate_artifact_reference.v0\",",
    );
    raw_line("        \"artifact_reference_canonicalization\": \"raios.module_candidate_artifact_reference.canonical.v0\"");
    raw_line("      },");
    emit_module_artifact_reference_object(&check);
    raw_line(",");
    emit_module_artifact_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_artifact_gate_state(&check);
    raw_line(",");
    emit_module_artifact_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "candidate_artifact", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "vm_test_report",
        "missing",
        "vm_test_report_missing",
    );
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "missing",
        "durable_audit_record_missing",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "missing",
        "rollback_plan_missing",
    );
    emit_export_gate(
        &mut wrote,
        "loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.artifact_diagnostic");
}

fn emit_module_artifact_reference_object(check: &ModuleArtifactReferenceCheck<'_>) {
    raw_line("      \"candidate_artifact_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_manifest_reference_event_id\": ");
    json_opt_str(check.retained_manifest_reference_event_id);
    raw_line(",");
    raw("        \"retained_computed_grant_reference_event_id\": ");
    json_opt_str(check.retained_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"artifact_reference_hash\": ");
    json_sha256_option(check.artifact_reference_hash);
    raw_line(",");
    raw("          \"expected_artifact_reference_hash\": ");
    json_sha256_option(check.expected_artifact_reference_hash);
    raw_line(",");
    raw("          \"manifest_reference_hash\": ");
    json_sha256_option(check.manifest_reference_hash);
    raw_line(",");
    raw("          \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    raw_line(",");
    raw("          \"computed_capability_grant_hash\": ");
    json_sha256_option(check.computed_grant_hash);
    raw_line(",");
    raw("          \"expected_computed_capability_grant_hash\": ");
    json_sha256_option(check.expected_computed_grant_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"vm_test_report_hash\": ");
    json_sha256_option(check.vm_report_hash);
    raw_line(",");
    raw("          \"local_attestation_hash\": ");
    json_sha256_option(check.local_attestation_hash);
    crlf();
    raw_line("        }");
    raw("      }");
}

fn emit_module_artifact_retained_reference(
    check: &ModuleArtifactReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::ModuleCandidateArtifactReference,
    )>,
) {
    raw_line("      \"retained_candidate_artifact_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(module_artifact_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_manifest_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"accepts_unsigned_service_code\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("          \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("          \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("          \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_candidate_artifact_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_artifact_gate_state(check: &ModuleArtifactReferenceCheck<'_>) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    raw_line("      \"gate_state\": {");
    raw_line("        \"module_manifest\": \"hash_reference_only\",");
    raw("        \"candidate_artifact\": ");
    json_str(state);
    raw_line(",");
    raw_line("        \"vm_test_report\": \"missing\",");
    raw_line("        \"local_attestation\": \"missing\",");
    raw_line("        \"computed_capability_grant\": \"hash_reference_only\",");
    raw_line("        \"local_approval\": \"missing\",");
    raw_line("        \"rollback_plan\": \"missing\",");
    raw_line("        \"durable_audit_record\": \"missing\",");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"artifact_loaded\": false,");
    raw_line("        \"service_started\": false,");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"can_load\": false");
    raw("      }");
}

fn emit_module_artifact_policy_result(check: &ModuleArtifactReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"candidate_artifact_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_artifact_bytes\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"raios.vm_test_report.v0\",");
    raw_line("          \"raios.local_attestation.v0\",");
    raw_line("          \"raios.audit_record.v0\",");
    raw_line("          \"rollback_plan\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot\"");
    raw_line("        ]");
    raw("      }");
}

fn emit_module_artifact_diagnostic_selftest() {
    let cases = module_artifact_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.artifact_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.module_candidate_artifact_reference_diagnostic_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_candidate_artifact_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_artifact_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.artifact_diagnostic_selftest");
}

fn emit_module_artifact_selftest_case(case: &ModuleArtifactSelfTestCase, comma: bool) {
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_manifest_selftest() {
    let cases = module_load_gate_manifest_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_manifest_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_manifest_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_manifest_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"manifest_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_manifest_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_manifest_selftest");
}

fn emit_module_load_gate_manifest_selftest_case(
    case: &ModuleLoadGateManifestSelfTestCase,
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
    raw(", \"actual_module_manifest_state\": ");
    json_str(case.actual_module_manifest_state);
    raw(", \"accepted_manifest_hash\": ");
    raw_bool(case.accepted_manifest_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_artifact_selftest() {
    let cases = module_load_gate_artifact_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_artifact_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_artifact_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_candidate_artifact_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"artifact_reference_hash\",");
    raw_line("        \"retained_manifest_reference_event_id\",");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_artifact_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_artifact_selftest");
}

fn emit_module_load_gate_artifact_selftest_case(
    case: &ModuleLoadGateArtifactSelfTestCase,
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
    raw(", \"actual_candidate_artifact_state\": ");
    json_str(case.actual_candidate_artifact_state);
    raw(", \"accepted_artifact_hash\": ");
    raw_bool(case.accepted_artifact_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_vm_report_selftest() {
    let cases = module_load_gate_vm_report_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_vm_report_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_vm_report_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_vm_test_report_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"vm_test_report_reference_hash\",");
    raw_line("        \"retained_manifest_reference_event_id\",");
    raw_line("        \"retained_candidate_artifact_reference_event_id\",");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"manifest_reference_hash\",");
    raw_line("        \"artifact_reference_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_vm_report_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_vm_report_selftest");
}

fn emit_module_load_gate_vm_report_selftest_case(
    case: &ModuleLoadGateVmReportSelfTestCase,
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
    raw(", \"actual_vm_test_report_state\": ");
    json_str(case.actual_vm_test_report_state);
    raw(", \"accepted_vm_test_report_hash\": ");
    raw_bool(case.accepted_vm_report_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_retained_selftest() {
    let cases = module_load_gate_retained_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_retained_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_retained_reference_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_reference_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"unallocated\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_retained_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_retained_selftest");
}

fn emit_module_load_gate_retained_selftest_case(
    case: &ModuleLoadGateRetainedSelfTestCase,
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_audit_rollback_selftest() {
    let cases = module_load_gate_audit_rollback_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_audit_rollback_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_audit_rollback_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"unallocated\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"retained_audit_rollback_reference_event_id\",");
    raw_line("        \"audit_record_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"manifest_hash\",");
    raw_line("        \"artifact_hash\",");
    raw_line("        \"vm_test_report_hash\",");
    raw_line("        \"local_attestation_hash\",");
    raw_line("        \"local_approval\",");
    raw_line("        \"rollback_plan_hash\",");
    raw_line("        \"ram_only_service_slot_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_audit_rollback_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_audit_rollback_selftest");
}

fn emit_module_load_gate_audit_rollback_selftest_case(
    case: &ModuleLoadGateAuditRollbackSelfTestCase,
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_service_slot_selftest() {
    let cases = module_load_gate_service_slot_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.load_gate_service_slot_selftest");
    raw_line("      \"schema\": \"raios.module_load_gate_service_slot_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"service_slot\": \"non_authorizing\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_bindings\": [");
    raw_line("        \"retained_computed_grant_reference_event_id\",");
    raw_line("        \"retained_audit_rollback_reference_event_id\",");
    raw_line("        \"reservation_hash\",");
    raw_line("        \"computed_capability_grant_hash\",");
    raw_line("        \"audit_record_hash\",");
    raw_line("        \"rollback_plan_hash\",");
    raw_line("        \"pre_load_service_inventory_hash\",");
    raw_line("        \"ram_only_service_slot_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_load_gate_service_slot_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.load_gate_service_slot_selftest");
}

fn emit_module_load_gate_service_slot_selftest_case(
    case: &ModuleLoadGateServiceSlotSelfTestCase,
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
    raw(", \"actual_service_slot_state\": ");
    json_str(case.actual_service_slot_state);
    raw(", \"accepted_service_slot_reservation_hash\": ");
    raw_bool(case.accepted_service_slot_reservation_hash);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"allocates_service_slot\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn parse_module_manifest_reference(arg: &str) -> ModuleManifestReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_manifest_reference(false, true, "current_boot", None, None);
    }

    let mut tokens = arg.split_whitespace();
    let manifest_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = manifest_reference_token.is_some() && manifest_token.is_some() && !extra;

    let manifest_reference_hash = manifest_reference_token.and_then(parse_sha256_ref);
    let manifest_hash = manifest_token.and_then(parse_sha256_ref);

    evaluate_module_manifest_reference(
        true,
        arity_valid,
        scope,
        manifest_reference_hash,
        manifest_hash,
    )
}

fn evaluate_module_manifest_reference<'a>(
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    manifest_reference_hash: Option<[u8; 32]>,
    manifest_hash: Option<[u8; 32]>,
) -> ModuleManifestReferenceCheck<'a> {
    if !has_reference {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "missing",
            reason: "module_manifest_reference_absent",
            valid: false,
        };
    }
    if !arity_valid {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "invalid_reference_arity",
            reason: "module_manifest_reference_requires_two_hashes_and_optional_scope",
            valid: false,
        };
    }
    let (Some(manifest_reference_hash), Some(manifest_hash)) =
        (manifest_reference_hash, manifest_hash)
    else {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash,
            manifest_hash,
            expected_manifest_reference_hash: None,
            status: "invalid_hash_reference",
            reason: "all_module_manifest_references_must_be_sha256",
            valid: false,
        };
    };
    let expected_manifest_reference_hash = computed_module_manifest_reference_hash(manifest_hash);
    if !method_eq(scope, "current_boot") {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash: Some(manifest_reference_hash),
            manifest_hash: Some(manifest_hash),
            expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
            status: "stale_or_non_current_boot_reference",
            reason: "module_manifest_reference_scope_must_be_current_boot",
            valid: false,
        };
    }
    if manifest_reference_hash != expected_manifest_reference_hash {
        return ModuleManifestReferenceCheck {
            has_reference,
            arity_valid,
            scope,
            manifest_reference_hash: Some(manifest_reference_hash),
            manifest_hash: Some(manifest_hash),
            expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
            status: "mismatched_manifest_reference_hash",
            reason: "module_manifest_reference_hash_mismatch",
            valid: false,
        };
    }
    ModuleManifestReferenceCheck {
        has_reference,
        arity_valid,
        scope,
        manifest_reference_hash: Some(manifest_reference_hash),
        manifest_hash: Some(manifest_hash),
        expected_manifest_reference_hash: Some(expected_manifest_reference_hash),
        status: "valid_hash_reference_load_still_denied",
        reason: "module_manifest_reference_valid_but_loader_and_evidence_missing",
        valid: true,
    }
}

fn module_manifest_selftest_cases() -> [ModuleManifestSelfTestCase; MODULE_MANIFEST_SELFTEST_CASES]
{
    let valid_hash = computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let absent = evaluate_module_manifest_reference(false, true, "current_boot", None, None);
    let valid = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    let stale = evaluate_module_manifest_reference(
        true,
        true,
        "previous_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    let mismatch = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some(valid_hash),
        Some(MODULE_GRANT_MISMATCH_MANIFEST_HASH),
    );
    let invalid_hash = evaluate_module_manifest_reference(
        true,
        true,
        "current_boot",
        Some([0x99; 32]),
        Some(MODULE_GRANT_TEST_MANIFEST_HASH),
    );
    [
        module_manifest_selftest_case(
            "absent_reference",
            "missing",
            "module_manifest_reference_absent",
            absent,
        ),
        module_manifest_selftest_case(
            "accepted_current_boot_manifest_still_denied",
            "valid_hash_reference_load_still_denied",
            "module_manifest_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_manifest_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "module_manifest_reference_scope_must_be_current_boot",
            stale,
        ),
        module_manifest_selftest_case(
            "mismatched_manifest_hash_reference",
            "mismatched_manifest_reference_hash",
            "module_manifest_reference_hash_mismatch",
            mismatch,
        ),
        module_manifest_selftest_case(
            "invalid_manifest_reference_hash",
            "mismatched_manifest_reference_hash",
            "module_manifest_reference_hash_mismatch",
            invalid_hash,
        ),
    ]
}

fn module_manifest_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleManifestReferenceCheck<'_>,
) -> ModuleManifestSelfTestCase {
    ModuleManifestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn module_manifest_binding_from_check(
    check: &ModuleManifestReferenceCheck<'_>,
) -> Option<event_log::ModuleManifestReference> {
    Some(event_log::ModuleManifestReference {
        manifest_reference_hash: check.manifest_reference_hash?,
        manifest_hash: check.manifest_hash?,
    })
}

fn module_manifest_reference_matches(
    check: &ModuleManifestReferenceCheck<'_>,
    reference: event_log::ModuleManifestReference,
) -> bool {
    check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
}

fn parse_module_artifact_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleArtifactReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_artifact_reference(
            ModuleArtifactReferenceInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                artifact_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                manifest_hash: None,
                computed_grant_hash: None,
                artifact_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
            },
            require_live_retained,
        );
    }

    let mut tokens = arg.split_whitespace();
    let artifact_reference_token = tokens.next();
    let retained_manifest_reference_event_id = tokens.next();
    let retained_reference_event_id = tokens.next();
    let manifest_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let grant_token = tokens.next();
    let artifact_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = artifact_reference_token.is_some()
        && retained_manifest_reference_event_id.is_some()
        && retained_reference_event_id.is_some()
        && manifest_reference_token.is_some()
        && manifest_token.is_some()
        && grant_token.is_some()
        && artifact_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && !extra;

    evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            has_reference: true,
            arity_valid,
            scope,
            artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
            retained_manifest_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
            manifest_hash: manifest_token.and_then(parse_sha256_ref),
            computed_grant_hash: grant_token.and_then(parse_sha256_ref),
            artifact_hash: artifact_token.and_then(parse_sha256_ref),
            vm_report_hash: report_token.and_then(parse_sha256_ref),
            local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        },
        require_live_retained,
    )
}

fn evaluate_module_artifact_reference<'a>(
    input: ModuleArtifactReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleArtifactReferenceCheck<'a> {
    if !input.has_reference {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "missing",
            "candidate_artifact_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "invalid_reference_arity",
            "candidate_artifact_reference_requires_hashes_events_and_optional_scope",
            false,
        );
    }

    let (
        Some(artifact_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(manifest_hash),
        Some(computed_grant_hash),
        Some(artifact_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        input.artifact_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.manifest_hash,
        input.computed_grant_hash,
        input.artifact_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
    )
    else {
        return module_artifact_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "all_candidate_artifact_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_artifact_reference_hash = computed_module_candidate_artifact_reference_hash(
        retained_manifest_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash,
        manifest_hash,
        expected_computed_grant_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );

    if !method_eq(input.scope, "current_boot") {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "stale_or_non_current_boot_reference",
            "candidate_artifact_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if artifact_reference_hash != expected_artifact_reference_hash {
        return module_artifact_reference_check(
            input,
            Some(expected_artifact_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_candidate_artifact_reference_hash",
            "candidate_artifact_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_artifact_live_reference_mismatch(&input) {
            return module_artifact_reference_check(
                input,
                Some(expected_artifact_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_artifact_reference_check(
        input,
        Some(expected_artifact_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "candidate_artifact_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

fn module_artifact_reference_check<'a>(
    input: ModuleArtifactReferenceInput<'a>,
    expected_artifact_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleArtifactReferenceCheck<'a> {
    ModuleArtifactReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        artifact_reference_hash: input.artifact_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        manifest_hash: input.manifest_hash,
        computed_grant_hash: input.computed_grant_hash,
        artifact_hash: input.artifact_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        expected_artifact_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_artifact_live_reference_mismatch(
    input: &ModuleArtifactReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;
    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("candidate_artifact_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("candidate_artifact_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("candidate_artifact_manifest_reference_hash_mismatch");
    }
    if Some(manifest_reference.manifest_hash) != input.manifest_hash {
        return Some("candidate_artifact_manifest_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("candidate_artifact_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("candidate_artifact_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("candidate_artifact_computed_grant_hash_mismatch");
    }
    if Some(retained_reference.manifest_hash) != input.manifest_hash {
        return Some("candidate_artifact_manifest_hash_mismatch");
    }
    if Some(retained_reference.artifact_hash) != input.artifact_hash {
        return Some("candidate_artifact_hash_mismatch");
    }
    if Some(retained_reference.vm_report_hash) != input.vm_report_hash {
        return Some("candidate_artifact_vm_report_hash_mismatch");
    }
    if Some(retained_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("candidate_artifact_local_attestation_hash_mismatch");
    }
    None
}

fn module_artifact_selftest_cases() -> [ModuleArtifactSelfTestCase; MODULE_ARTIFACT_SELFTEST_CASES]
{
    let manifest_reference_hash =
        computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let valid_hash = computed_module_candidate_artifact_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let base = ModuleArtifactReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        artifact_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    };
    let absent = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            has_reference: false,
            arity_valid: true,
            scope: "current_boot",
            artifact_reference_hash: None,
            retained_manifest_reference_event_id: None,
            retained_reference_event_id: None,
            manifest_reference_hash: None,
            manifest_hash: None,
            computed_grant_hash: None,
            artifact_hash: None,
            vm_report_hash: None,
            local_attestation_hash: None,
        },
        false,
    );
    let valid = evaluate_module_artifact_reference(base, false);
    let stale = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            scope: "previous_boot",
            ..base
        },
        false,
    );
    let mismatch = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            artifact_reference_hash: Some([0x99; 32]),
            ..base
        },
        false,
    );
    let invalid_hash = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            artifact_reference_hash: None,
            ..base
        },
        false,
    );
    let grant_mismatch = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            computed_grant_hash: Some([0xaa; 32]),
            ..base
        },
        false,
    );
    let bad_event_id = evaluate_module_artifact_reference(
        ModuleArtifactReferenceInput {
            retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
            ..base
        },
        false,
    );
    [
        module_artifact_selftest_case(
            "absent_reference",
            "missing",
            "candidate_artifact_reference_absent",
            absent,
        ),
        module_artifact_selftest_case(
            "accepted_current_boot_artifact_still_denied",
            "valid_hash_reference_load_still_denied",
            "candidate_artifact_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_artifact_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "candidate_artifact_reference_scope_must_be_current_boot",
            stale,
        ),
        module_artifact_selftest_case(
            "mismatched_artifact_reference_hash",
            "mismatched_candidate_artifact_reference_hash",
            "candidate_artifact_reference_hash_mismatch",
            mismatch,
        ),
        module_artifact_selftest_case(
            "invalid_artifact_reference_hash",
            "invalid_hash_reference",
            "all_candidate_artifact_references_must_be_sha256_or_current_boot_ids",
            invalid_hash,
        ),
        module_artifact_selftest_case(
            "computed_grant_hash_mismatch",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            grant_mismatch,
        ),
        module_artifact_selftest_case(
            "retained_manifest_reference_event_id_not_current_boot",
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            bad_event_id,
        ),
    ]
}

fn module_artifact_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleArtifactReferenceCheck<'_>,
) -> ModuleArtifactSelfTestCase {
    ModuleArtifactSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn module_artifact_binding_from_check(
    check: &ModuleArtifactReferenceCheck<'_>,
) -> Option<event_log::ModuleCandidateArtifactReference> {
    Some(event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash: check.artifact_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        manifest_hash: check.manifest_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        artifact_hash: check.artifact_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
    })
}

fn module_artifact_reference_matches(
    check: &ModuleArtifactReferenceCheck<'_>,
    reference: event_log::ModuleCandidateArtifactReference,
) -> bool {
    check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check
            .retained_manifest_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_manifest_reference_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

fn emit_module_vm_report_diagnostic(method: &str) {
    let arg = module_vm_report_diagnostic_arg(method);
    let check = parse_module_vm_report_reference(arg, true);
    let recorded_event_id = if check.valid {
        module_vm_report_binding_from_check(&check)
            .map(event_log::record_module_vm_test_report_reference)
    } else {
        None
    };
    let retained = event_log::latest_module_vm_test_report_reference();

    begin_response("module.vm_report_diagnostic");
    raw_line("      \"schema\": \"raios.module_vm_test_report_reference_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"module.vm_report_diagnostic <report_reference_hash> <retained_manifest_reference_event_id> <retained_artifact_reference_event_id> <retained_reference_event_id> <manifest_reference_hash> <artifact_reference_hash> <manifest_hash> <artifact_hash> <computed_grant_hash> <vm_report_hash> <local_attestation_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"vm_test_report_schema\": \"raios.vm_test_report.v0\",");
    raw_line("        \"vm_test_report_reference_schema\": \"raios.module_vm_test_report_reference.v0\",");
    raw_line("        \"vm_test_report_reference_canonicalization\": \"raios.module_vm_test_report_reference.canonical.v0\"");
    raw_line("      },");
    emit_module_vm_report_reference_object(&check);
    raw_line(",");
    emit_module_vm_report_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_vm_report_gate_state(&check);
    raw_line(",");
    emit_module_vm_report_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !check.valid {
        emit_export_gate(&mut wrote, "vm_test_report", check.status, check.reason);
    }
    emit_export_gate(
        &mut wrote,
        "local_attestation",
        "missing",
        "local_attestation_missing",
    );
    emit_export_gate(
        &mut wrote,
        "durable_audit_record",
        "missing",
        "durable_audit_record_missing",
    );
    emit_export_gate(
        &mut wrote,
        "rollback_plan",
        "missing",
        "rollback_plan_missing",
    );
    emit_export_gate(
        &mut wrote,
        "loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.vm_report_diagnostic");
}

fn emit_module_vm_report_reference_object(check: &ModuleVmReportReferenceCheck<'_>) {
    raw_line("      \"vm_test_report_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_manifest_reference_event_id\": ");
    json_opt_str(check.retained_manifest_reference_event_id);
    raw_line(",");
    raw("        \"retained_candidate_artifact_reference_event_id\": ");
    json_opt_str(check.retained_artifact_reference_event_id);
    raw_line(",");
    raw("        \"retained_computed_grant_reference_event_id\": ");
    json_opt_str(check.retained_reference_event_id);
    raw_line(",");
    raw_line("        \"vm_test_report_schema\": \"raios.vm_test_report.v0\",");
    raw("        \"vm_test_report_reference_hash\": ");
    json_sha256_option(check.report_reference_hash);
    raw_line(",");
    raw("        \"expected_vm_test_report_reference_hash\": ");
    json_sha256_option(check.expected_report_reference_hash);
    raw_line(",");
    raw("        \"expected_computed_capability_grant_hash\": ");
    json_sha256_option(check.expected_computed_grant_hash);
    raw_line(",");
    raw("        \"manifest_reference_hash\": ");
    json_sha256_option(check.manifest_reference_hash);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256_option(check.artifact_reference_hash);
    raw_line(",");
    raw("        \"manifest_hash\": ");
    json_sha256_option(check.manifest_hash);
    raw_line(",");
    raw("        \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("        \"computed_capability_grant_hash\": ");
    json_sha256_option(check.computed_grant_hash);
    raw_line(",");
    raw("        \"vm_test_report_hash\": ");
    json_sha256_option(check.vm_report_hash);
    raw_line(",");
    raw("        \"local_attestation_hash\": ");
    json_sha256_option(check.local_attestation_hash);
    crlf();
    raw_line("      }");
}

fn emit_module_vm_report_retained_reference(
    check: &ModuleVmReportReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleVmTestReportReference)>,
) {
    raw_line("      \"retained_vm_test_report_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(module_vm_report_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_manifest_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"accepts_vm_report_json\": false,");
        raw_line("        \"accepts_unsigned_service_code\": false,");
        raw_line("        \"authorizes_guest_load\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("        \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("          \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("          \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("          \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("          \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_vm_test_report_reference_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_vm_report_gate_state(check: &ModuleVmReportReferenceCheck<'_>) {
    let state = if check.valid {
        "hash_reference_valid"
    } else if check.has_reference {
        "hash_reference_invalid"
    } else {
        "missing"
    };
    raw_line("      \"gate_state\": {");
    raw_line("        \"module_manifest\": \"retained_hash_reference_only\",");
    raw_line("        \"candidate_artifact\": \"retained_hash_reference_only\",");
    raw("        \"vm_test_report\": ");
    json_str(state);
    raw_line(",");
    raw_line("        \"local_attestation\": \"missing\",");
    raw_line("        \"computed_capability_grant\": \"retained_hash_reference_only\",");
    raw_line("        \"local_approval\": \"missing\",");
    raw_line("        \"rollback_plan\": \"missing\",");
    raw_line("        \"durable_audit_record\": \"missing\",");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_slot\": \"unallocated\",");
    raw_line("        \"artifact_loaded\": false,");
    raw_line("        \"service_started\": false,");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"can_load\": false");
    raw("      }");
}

fn emit_module_vm_report_policy_result(check: &ModuleVmReportReferenceCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"vm_test_report_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_guest_load\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"guest_evidence_authority\": \"hash_reference_only_no_vm_report_json_or_artifact_bytes\",");
    raw_line("        \"required_before_load\": [");
    raw_line("          \"raios.local_attestation.v0\",");
    raw_line("          \"local_approval\",");
    raw_line("          \"raios.audit_record.v0\",");
    raw_line("          \"rollback_plan\",");
    raw_line("          \"module_loader\",");
    raw_line("          \"ram_only_service_slot\"");
    raw_line("        ]");
    raw("      }");
}

fn emit_module_vm_report_diagnostic_selftest() {
    let cases = module_vm_report_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.vm_report_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_vm_test_report_reference_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_vm_test_report_reference_records\": false,");
    raw_line("      \"accepts_manifest_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_vm_report_json\": false,");
    raw_line("      \"accepts_unsigned_service_code\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_vm_report_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.vm_report_diagnostic_selftest");
}

fn emit_module_vm_report_selftest_case(case: &ModuleVmReportSelfTestCase, comma: bool) {
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
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn parse_module_vm_report_reference(
    arg: &str,
    require_live_retained: bool,
) -> ModuleVmReportReferenceCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_vm_report_reference(
            ModuleVmReportReferenceInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                report_reference_hash: None,
                retained_manifest_reference_event_id: None,
                retained_artifact_reference_event_id: None,
                retained_reference_event_id: None,
                manifest_reference_hash: None,
                artifact_reference_hash: None,
                manifest_hash: None,
                artifact_hash: None,
                computed_grant_hash: None,
                vm_report_hash: None,
                local_attestation_hash: None,
            },
            require_live_retained,
        );
    }

    let mut tokens = arg.split_whitespace();
    let report_reference_token = tokens.next();
    let retained_manifest_reference_event_id = tokens.next();
    let retained_artifact_reference_event_id = tokens.next();
    let retained_reference_event_id = tokens.next();
    let manifest_reference_token = tokens.next();
    let artifact_reference_token = tokens.next();
    let manifest_token = tokens.next();
    let artifact_token = tokens.next();
    let grant_token = tokens.next();
    let report_token = tokens.next();
    let attestation_token = tokens.next();
    let scope = tokens.next().unwrap_or("current_boot");
    let extra = tokens.next().is_some();
    let arity_valid = report_reference_token.is_some()
        && retained_manifest_reference_event_id.is_some()
        && retained_artifact_reference_event_id.is_some()
        && retained_reference_event_id.is_some()
        && manifest_reference_token.is_some()
        && artifact_reference_token.is_some()
        && manifest_token.is_some()
        && artifact_token.is_some()
        && grant_token.is_some()
        && report_token.is_some()
        && attestation_token.is_some()
        && !extra;

    evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            has_reference: true,
            arity_valid,
            scope,
            report_reference_hash: report_reference_token.and_then(parse_sha256_ref),
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash: manifest_reference_token.and_then(parse_sha256_ref),
            artifact_reference_hash: artifact_reference_token.and_then(parse_sha256_ref),
            manifest_hash: manifest_token.and_then(parse_sha256_ref),
            artifact_hash: artifact_token.and_then(parse_sha256_ref),
            computed_grant_hash: grant_token.and_then(parse_sha256_ref),
            vm_report_hash: report_token.and_then(parse_sha256_ref),
            local_attestation_hash: attestation_token.and_then(parse_sha256_ref),
        },
        require_live_retained,
    )
}

fn evaluate_module_vm_report_reference<'a>(
    input: ModuleVmReportReferenceInput<'a>,
    require_live_retained: bool,
) -> ModuleVmReportReferenceCheck<'a> {
    if !input.has_reference {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "missing",
            "vm_test_report_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "invalid_reference_arity",
            "vm_test_report_reference_requires_hashes_events_and_optional_scope",
            false,
        );
    }

    let (
        Some(report_reference_hash),
        Some(retained_manifest_reference_event_id),
        Some(retained_artifact_reference_event_id),
        Some(retained_reference_event_id),
        Some(manifest_reference_hash),
        Some(artifact_reference_hash),
        Some(manifest_hash),
        Some(artifact_hash),
        Some(computed_grant_hash),
        Some(vm_report_hash),
        Some(local_attestation_hash),
    ) = (
        input.report_reference_hash,
        input.retained_manifest_reference_event_id,
        input.retained_artifact_reference_event_id,
        input.retained_reference_event_id,
        input.manifest_reference_hash,
        input.artifact_reference_hash,
        input.manifest_hash,
        input.artifact_hash,
        input.computed_grant_hash,
        input.vm_report_hash,
        input.local_attestation_hash,
    )
    else {
        return module_vm_report_reference_check(
            input,
            None,
            None,
            "invalid_hash_reference",
            "all_vm_test_report_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    let expected_report_reference_hash = computed_module_vm_test_report_reference_hash(
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash,
        artifact_reference_hash,
        manifest_hash,
        artifact_hash,
        expected_computed_grant_hash,
        vm_report_hash,
        local_attestation_hash,
    );

    if !method_eq(input.scope, "current_boot") {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "stale_or_non_current_boot_reference",
            "vm_test_report_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_manifest_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_artifact_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if computed_grant_hash != expected_computed_grant_hash {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            false,
        );
    }
    if report_reference_hash != expected_report_reference_hash {
        return module_vm_report_reference_check(
            input,
            Some(expected_report_reference_hash),
            Some(expected_computed_grant_hash),
            "mismatched_vm_test_report_reference_hash",
            "vm_test_report_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_vm_report_live_reference_mismatch(&input) {
            return module_vm_report_reference_check(
                input,
                Some(expected_report_reference_hash),
                Some(expected_computed_grant_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_vm_report_reference_check(
        input,
        Some(expected_report_reference_hash),
        Some(expected_computed_grant_hash),
        "valid_hash_reference_load_still_denied",
        "vm_test_report_reference_valid_but_loader_and_evidence_missing",
        true,
    )
}

fn module_vm_report_reference_check<'a>(
    input: ModuleVmReportReferenceInput<'a>,
    expected_report_reference_hash: Option<[u8; 32]>,
    expected_computed_grant_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleVmReportReferenceCheck<'a> {
    ModuleVmReportReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        report_reference_hash: input.report_reference_hash,
        retained_manifest_reference_event_id: input.retained_manifest_reference_event_id,
        retained_artifact_reference_event_id: input.retained_artifact_reference_event_id,
        retained_reference_event_id: input.retained_reference_event_id,
        manifest_reference_hash: input.manifest_reference_hash,
        artifact_reference_hash: input.artifact_reference_hash,
        manifest_hash: input.manifest_hash,
        artifact_hash: input.artifact_hash,
        computed_grant_hash: input.computed_grant_hash,
        vm_report_hash: input.vm_report_hash,
        local_attestation_hash: input.local_attestation_hash,
        expected_report_reference_hash,
        expected_computed_grant_hash,
        status,
        reason,
        valid,
    }
}

fn module_vm_report_live_reference_mismatch(
    input: &ModuleVmReportReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_manifest_reference_event_id =
        parse_current_boot_event_id(input.retained_manifest_reference_event_id?)?;
    let retained_artifact_reference_event_id =
        parse_current_boot_event_id(input.retained_artifact_reference_event_id?)?;
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;
    let Some((latest_manifest_event_id, manifest_reference)) =
        event_log::latest_module_manifest_reference()
    else {
        return Some("vm_test_report_manifest_reference_missing");
    };
    if latest_manifest_event_id != retained_manifest_reference_event_id {
        return Some("vm_test_report_manifest_reference_mismatch");
    }
    if Some(manifest_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("vm_test_report_manifest_reference_hash_mismatch");
    }
    if Some(manifest_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }

    let Some((latest_artifact_event_id, artifact_reference)) =
        event_log::latest_module_candidate_artifact_reference()
    else {
        return Some("vm_test_report_artifact_reference_missing");
    };
    if latest_artifact_event_id != retained_artifact_reference_event_id {
        return Some("vm_test_report_artifact_reference_mismatch");
    }
    if Some(artifact_reference.artifact_reference_hash) != input.artifact_reference_hash {
        return Some("vm_test_report_artifact_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_reference_hash) != input.manifest_reference_hash {
        return Some("vm_test_report_manifest_reference_hash_mismatch");
    }
    if Some(artifact_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }
    if Some(artifact_reference.artifact_hash) != input.artifact_hash {
        return Some("vm_test_report_artifact_hash_mismatch");
    }
    if Some(artifact_reference.vm_report_hash) != input.vm_report_hash {
        return Some("vm_test_report_hash_mismatch");
    }
    if Some(artifact_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("vm_test_report_local_attestation_hash_mismatch");
    }

    let Some((latest_retained_event_id, retained_reference)) =
        event_log::latest_module_computed_grant_reference()
    else {
        return Some("vm_test_report_computed_grant_reference_missing");
    };
    if latest_retained_event_id != retained_reference_event_id {
        return Some("vm_test_report_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("vm_test_report_computed_grant_hash_mismatch");
    }
    if Some(retained_reference.manifest_hash) != input.manifest_hash {
        return Some("vm_test_report_manifest_hash_mismatch");
    }
    if Some(retained_reference.artifact_hash) != input.artifact_hash {
        return Some("vm_test_report_artifact_hash_mismatch");
    }
    if Some(retained_reference.vm_report_hash) != input.vm_report_hash {
        return Some("vm_test_report_hash_mismatch");
    }
    if Some(retained_reference.local_attestation_hash) != input.local_attestation_hash {
        return Some("vm_test_report_local_attestation_hash_mismatch");
    }
    None
}

fn module_vm_report_selftest_cases() -> [ModuleVmReportSelfTestCase; MODULE_VM_REPORT_SELFTEST_CASES]
{
    let manifest_reference_hash =
        computed_module_manifest_reference_hash(MODULE_GRANT_TEST_MANIFEST_HASH);
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let artifact_reference_hash = computed_module_candidate_artifact_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let valid_hash = computed_module_vm_test_report_reference_hash(
        MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        manifest_reference_hash,
        artifact_reference_hash,
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        computed_grant_hash,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let valid_input = ModuleVmReportReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        report_reference_hash: Some(valid_hash),
        retained_manifest_reference_event_id: Some(
            MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID,
        ),
        retained_artifact_reference_event_id: Some(
            MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID,
        ),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        manifest_reference_hash: Some(manifest_reference_hash),
        artifact_reference_hash: Some(artifact_reference_hash),
        manifest_hash: Some(MODULE_GRANT_TEST_MANIFEST_HASH),
        artifact_hash: Some(MODULE_GRANT_TEST_ARTIFACT_HASH),
        computed_grant_hash: Some(computed_grant_hash),
        vm_report_hash: Some(MODULE_GRANT_TEST_VM_REPORT_HASH),
        local_attestation_hash: Some(MODULE_GRANT_TEST_ATTESTATION_HASH),
    };
    let absent = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            has_reference: false,
            ..valid_input
        },
        false,
    );
    let valid = evaluate_module_vm_report_reference(valid_input, false);
    let stale = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            scope: "previous_boot",
            ..valid_input
        },
        false,
    );
    let mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            report_reference_hash: Some([0x99; 32]),
            ..valid_input
        },
        false,
    );
    let grant_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            computed_grant_hash: Some([0xaa; 32]),
            ..valid_input
        },
        false,
    );
    let manifest_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_manifest_reference_event_id: Some("event.previous_boot.00000026"),
            ..valid_input
        },
        false,
    );
    let artifact_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_artifact_reference_event_id: Some("event.previous_boot.00000028"),
            ..valid_input
        },
        false,
    );
    let grant_event_mismatch = evaluate_module_vm_report_reference(
        ModuleVmReportReferenceInput {
            retained_reference_event_id: Some("event.previous_boot.00000027"),
            ..valid_input
        },
        false,
    );
    [
        module_vm_report_selftest_case(
            "absent_reference",
            "missing",
            "vm_test_report_reference_absent",
            absent,
        ),
        module_vm_report_selftest_case(
            "accepted_current_boot_report_still_denied",
            "valid_hash_reference_load_still_denied",
            "vm_test_report_reference_valid_but_loader_and_evidence_missing",
            valid,
        ),
        module_vm_report_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "vm_test_report_reference_scope_must_be_current_boot",
            stale,
        ),
        module_vm_report_selftest_case(
            "vm_report_reference_hash_mismatch",
            "mismatched_vm_test_report_reference_hash",
            "vm_test_report_reference_hash_mismatch",
            mismatch,
        ),
        module_vm_report_selftest_case(
            "computed_grant_hash_mismatch",
            "mismatched_computed_grant_hash",
            "computed_grant_hash_mismatch",
            grant_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_manifest_reference_event_not_current_boot",
            "rejected",
            "retained_manifest_reference_event_id_not_current_boot",
            manifest_event_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_artifact_reference_event_not_current_boot",
            "rejected",
            "retained_artifact_reference_event_id_not_current_boot",
            artifact_event_mismatch,
        ),
        module_vm_report_selftest_case(
            "retained_reference_event_not_current_boot",
            "rejected",
            "retained_reference_event_id_not_current_boot",
            grant_event_mismatch,
        ),
    ]
}

fn module_vm_report_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ModuleVmReportReferenceCheck<'_>,
) -> ModuleVmReportSelfTestCase {
    ModuleVmReportSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn module_vm_report_binding_from_check(
    check: &ModuleVmReportReferenceCheck<'_>,
) -> Option<event_log::ModuleVmTestReportReference> {
    Some(event_log::ModuleVmTestReportReference {
        report_reference_hash: check.report_reference_hash?,
        retained_manifest_reference_event_id: parse_current_boot_event_id(
            check.retained_manifest_reference_event_id?,
        )?,
        retained_artifact_reference_event_id: parse_current_boot_event_id(
            check.retained_artifact_reference_event_id?,
        )?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        manifest_reference_hash: check.manifest_reference_hash?,
        artifact_reference_hash: check.artifact_reference_hash?,
        manifest_hash: check.manifest_hash?,
        artifact_hash: check.artifact_hash?,
        computed_grant_hash: check.computed_grant_hash?,
        vm_report_hash: check.vm_report_hash?,
        local_attestation_hash: check.local_attestation_hash?,
    })
}

fn module_vm_report_reference_matches(
    check: &ModuleVmReportReferenceCheck<'_>,
    reference: event_log::ModuleVmTestReportReference,
) -> bool {
    check.report_reference_hash == Some(reference.report_reference_hash)
        && check
            .retained_manifest_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_manifest_reference_event_id)
        && check
            .retained_artifact_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_artifact_reference_event_id)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check.manifest_reference_hash == Some(reference.manifest_reference_hash)
        && check.artifact_reference_hash == Some(reference.artifact_reference_hash)
        && check.manifest_hash == Some(reference.manifest_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.vm_report_hash == Some(reference.vm_report_hash)
        && check.local_attestation_hash == Some(reference.local_attestation_hash)
}

fn module_load_gate_manifest_selftest_cases(
) -> [ModuleLoadGateManifestSelfTestCase; MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES] {
    let valid_reference = module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let substituted_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_MISMATCH_MANIFEST_HASH);
    let mismatched_hash_reference = event_log::ModuleManifestReference {
        manifest_reference_hash: [0x99; 32],
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
    };

    [
        module_load_gate_manifest_selftest_case(
            "missing_retained_manifest_reference",
            "missing",
            "retained_module_manifest_reference_missing",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
            },
        ),
        module_load_gate_manifest_selftest_case(
            "accepted_current_boot_manifest_still_denied",
            "retained_hash_reference_only",
            "retained_module_manifest_reference_not_authorizing",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "stale_dropped_manifest_reference_event_id",
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "previous_boot_or_unretained_manifest_reference",
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "substituted_manifest_reference_record",
            "rejected",
            "retained_module_manifest_reference_substituted_record",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "manifest_reference_hash_mismatch",
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
            },
        ),
    ]
}

fn module_load_gate_manifest_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestSelfTestCase {
    let actual = evaluate_module_load_gate_manifest_candidate(candidate);
    ModuleLoadGateManifestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_module_manifest_state: actual.module_manifest_state,
        accepted_manifest_hash: actual.accepted_manifest_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_manifest_reference(
    manifest_hash: [u8; 32],
) -> event_log::ModuleManifestReference {
    event_log::ModuleManifestReference {
        manifest_reference_hash: computed_module_manifest_reference_hash(manifest_hash),
        manifest_hash,
    }
}

fn module_load_gate_artifact_selftest_cases(
) -> [ModuleLoadGateArtifactSelfTestCase; MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES] {
    let valid_manifest_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let valid_retained_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let manifest_event_id = module_load_gate_test_event_id(26);
    let retained_event_id = module_load_gate_test_event_id(27);
    let valid_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_retained_reference,
    );
    let substituted_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        module_load_gate_test_reference(
            MODULE_GRANT_TEST_MANIFEST_HASH,
            [0xbb; 32],
            MODULE_GRANT_TEST_VM_REPORT_HASH,
            MODULE_GRANT_TEST_ATTESTATION_HASH,
        ),
    );
    let mismatched_hash_reference = event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash: [0x99; 32],
        ..valid_reference
    };

    [
        module_load_gate_artifact_selftest_case(
            "missing_retained_candidate_artifact_reference",
            "missing",
            "retained_candidate_artifact_reference_missing",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "accepted_current_boot_artifact_still_denied",
            "retained_hash_reference_only",
            "retained_candidate_artifact_reference_not_authorizing",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "stale_dropped_retained_artifact_reference_event_id",
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "previous_boot_or_unretained_artifact_reference",
            "rejected",
            "retained_candidate_artifact_reference_previous_boot_or_unretained",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_candidate_artifact_reference_wrong_schema_or_variant",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "substituted_artifact_reference_record",
            "rejected",
            "retained_candidate_artifact_reference_substituted_record",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "artifact_reference_hash_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "manifest_reference_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(module_load_gate_test_event_id(99)),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "computed_grant_reference_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(module_load_gate_test_event_id(98)),
                retained_reference: Some(valid_retained_reference),
            },
        ),
    ]
}

fn module_load_gate_artifact_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactSelfTestCase {
    let actual = evaluate_module_load_gate_artifact_candidate(candidate);
    ModuleLoadGateArtifactSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_candidate_artifact_state: actual.candidate_artifact_state,
        accepted_artifact_hash: actual.accepted_artifact_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_event_id(sequence: u64) -> event_log::EventId {
    let mut candidate = sequence;
    loop {
        if let Some(event_id) = event_log::EventId::from_sequence(candidate) {
            return event_id;
        }
        candidate = 1;
    }
}

fn module_load_gate_test_artifact_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> event_log::ModuleCandidateArtifactReference {
    let artifact_reference_hash =
        module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            manifest_reference.manifest_hash,
            retained_reference.computed_grant_hash,
            retained_reference.artifact_hash,
            retained_reference.vm_report_hash,
            retained_reference.local_attestation_hash,
        );
    event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash,
        retained_manifest_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        artifact_hash: retained_reference.artifact_hash,
        vm_report_hash: retained_reference.vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
    }
}

fn module_load_gate_vm_report_selftest_cases(
) -> [ModuleLoadGateVmReportSelfTestCase; MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES] {
    let valid_manifest_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let valid_retained_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let manifest_event_id = module_load_gate_test_event_id(26);
    let artifact_event_id = module_load_gate_test_event_id(28);
    let retained_event_id = module_load_gate_test_event_id(27);
    let valid_artifact_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_retained_reference,
    );
    let valid_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_artifact_reference,
        valid_retained_reference,
        None,
    );
    let substituted_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        event_log::ModuleCandidateArtifactReference {
            artifact_hash: [0xbb; 32],
            ..valid_artifact_reference
        },
        valid_retained_reference,
        None,
    );
    let mismatched_hash_reference = event_log::ModuleVmTestReportReference {
        report_reference_hash: [0x99; 32],
        ..valid_reference
    };
    let mismatched_report_hash_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_artifact_reference,
        valid_retained_reference,
        Some([0xbb; 32]),
    );

    [
        module_load_gate_vm_report_selftest_case(
            "missing_retained_vm_test_report_reference",
            "missing",
            "retained_vm_test_report_reference_missing",
            module_load_gate_vm_report_candidate(
                false,
                true,
                "current_boot",
                None,
                None,
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "accepted_current_boot_report_still_denied",
            "retained_hash_reference_only",
            "retained_vm_test_report_reference_not_authorizing",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "stale_dropped_retained_vm_test_report_reference_event_id",
            "rejected",
            "retained_vm_test_report_reference_stale_or_dropped_event_id",
            module_load_gate_vm_report_candidate(
                false,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "previous_boot_or_unretained_vm_test_report_reference",
            "rejected",
            "retained_vm_test_report_reference_previous_boot_or_unretained",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "previous_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_vm_test_report_reference_wrong_schema_or_variant",
            module_load_gate_vm_report_candidate(
                true,
                false,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "substituted_vm_test_report_reference_record",
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(substituted_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "vm_test_report_reference_hash_mismatch",
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(mismatched_hash_reference),
                Some(mismatched_hash_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "manifest_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                module_load_gate_test_event_id(99),
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "artifact_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                module_load_gate_test_event_id(98),
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "computed_grant_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                module_load_gate_test_event_id(97),
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "vm_test_report_hash_mismatch",
            "rejected",
            "retained_vm_test_report_hash_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(mismatched_report_hash_reference),
                Some(mismatched_report_hash_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
    ]
}

fn module_load_gate_vm_report_candidate(
    retained: bool,
    schema_ok: bool,
    scope: &'static str,
    event_reference: Option<event_log::ModuleVmTestReportReference>,
    candidate_reference: Option<event_log::ModuleVmTestReportReference>,
    manifest_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_event_id: event_log::EventId,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    retained_event_id: event_log::EventId,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> ModuleLoadGateVmReportReferenceCandidate {
    ModuleLoadGateVmReportReferenceCandidate {
        scope,
        retained,
        schema_ok,
        event_reference,
        candidate_reference,
        manifest_event_id: Some(manifest_event_id),
        manifest_reference: Some(manifest_reference),
        artifact_event_id: Some(artifact_event_id),
        artifact_reference: Some(artifact_reference),
        retained_event_id: Some(retained_event_id),
        retained_reference: Some(retained_reference),
    }
}

fn module_load_gate_vm_report_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportSelfTestCase {
    let actual = evaluate_module_load_gate_vm_report_candidate(candidate);
    ModuleLoadGateVmReportSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_vm_test_report_state: actual.vm_test_report_state,
        accepted_vm_report_hash: actual.accepted_vm_report_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_vm_report_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_artifact_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    retained_reference: event_log::ModuleComputedGrantReference,
    vm_report_hash_override: Option<[u8; 32]>,
) -> event_log::ModuleVmTestReportReference {
    let vm_report_hash = vm_report_hash_override.unwrap_or(retained_reference.vm_report_hash);
    let report_reference_hash =
        module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_artifact_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            artifact_reference.artifact_reference_hash,
            manifest_reference.manifest_hash,
            artifact_reference.artifact_hash,
            retained_reference.computed_grant_hash,
            vm_report_hash,
            retained_reference.local_attestation_hash,
        );
    event_log::ModuleVmTestReportReference {
        report_reference_hash,
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        artifact_reference_hash: artifact_reference.artifact_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        artifact_hash: artifact_reference.artifact_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
    }
}

fn module_load_gate_retained_selftest_cases(
) -> [ModuleLoadGateRetainedSelfTestCase; MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES] {
    let valid_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let substituted_reference = module_load_gate_test_reference(
        MODULE_GRANT_MISMATCH_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let mismatched_hash_reference = event_log::ModuleComputedGrantReference {
        computed_grant_hash: [0x66; 32],
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
    };

    [
        module_load_gate_retained_selftest_case(
            "missing_retained_reference",
            "missing",
            "computed_capability_grant_reference_missing",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
            },
        ),
        module_load_gate_retained_selftest_case(
            "accepted_current_boot_reference_still_denied",
            "retained_hash_reference_only",
            "retained_computed_grant_reference_not_authorizing",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "stale_dropped_retained_reference_event_id",
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "previous_boot_or_unretained_reference",
            "rejected",
            "retained_reference_previous_boot_or_unretained",
            ModuleLoadGateRetainedCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "wrong_schema_or_variant_substitution",
            "rejected",
            "retained_reference_wrong_schema_or_variant",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "substituted_retained_reference_record",
            "rejected",
            "retained_reference_substituted_record",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "mismatched_computed_grant_hash",
            "rejected",
            "retained_reference_hash_mismatch",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
            },
        ),
    ]
}

fn module_load_gate_audit_rollback_selftest_cases(
) -> [ModuleLoadGateAuditRollbackSelfTestCase; MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES] {
    let valid_requirements = module_load_gate_test_audit_rollback_candidate();
    let valid_audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let substituted_audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference_with_manifest(
            MODULE_GRANT_MISMATCH_MANIFEST_HASH,
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
        );
    let computed_grant_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            Some([0x99; 32]),
            None,
            None,
        );
    let audit_hash_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            None,
            None,
            Some([0xaa; 32]),
        );
    let rollback_hash_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            None,
            Some([0xbb; 32]),
            None,
        );
    let service_slot_mismatch_reference =
        module_load_gate_test_audit_rollback_reference("ram_only:svc.test.other");
    [
        module_load_gate_audit_rollback_selftest_case(
            "missing_retained_audit_rollback_reference",
            "missing",
            "retained_audit_rollback_reference_missing",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: false,
                    schema_ok: true,
                    event_reference: None,
                    candidate_reference: None,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "stale_dropped_retained_audit_rollback_reference_event_id",
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: false,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "previous_boot_or_unretained_audit_rollback_reference",
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "previous_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_wrong_schema_or_variant",
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: false,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "substituted_retained_audit_rollback_reference",
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: substituted_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_computed_grant_hash_mismatch",
            "rejected",
            "retained_audit_rollback_computed_grant_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: computed_grant_mismatch_reference,
                    candidate_reference: computed_grant_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_record_hash_mismatch",
            "rejected",
            "retained_audit_record_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: audit_hash_mismatch_reference,
                    candidate_reference: audit_hash_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_rollback_plan_hash_mismatch",
            "rejected",
            "retained_rollback_plan_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: rollback_hash_mismatch_reference,
                    candidate_reference: rollback_hash_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_service_slot_mismatch",
            "rejected",
            "retained_audit_rollback_service_slot_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: service_slot_mismatch_reference,
                    candidate_reference: service_slot_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "missing_durable_audit_record",
            "missing",
            "durable_audit_record_missing",
            ModuleLoadGateAuditRollbackCandidate {
                durable_audit_record: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "missing_rollback_plan",
            "missing",
            "rollback_plan_missing",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_plan: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "durable_audit_record_schema_mismatch",
            "rejected",
            "durable_audit_record_schema_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_schema_ok: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_plan_schema_mismatch",
            "rejected",
            "rollback_plan_schema_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_schema_ok: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "valid_audit_and_rollback_still_denied",
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
            valid_requirements,
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_retained_grant_hash_mismatch",
            "rejected",
            "audit_retained_grant_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_retained_grant: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_manifest_hash_mismatch",
            "rejected",
            "audit_manifest_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_manifest: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_artifact_hash_mismatch",
            "rejected",
            "audit_artifact_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_artifact: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_vm_report_hash_mismatch",
            "rejected",
            "audit_vm_test_report_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_vm_report: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_local_attestation_hash_mismatch",
            "rejected",
            "audit_local_attestation_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_local_attestation: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "local_approval_mismatch",
            "rejected",
            "local_approval_missing_or_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_local_approval: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_rollback_plan_hash_mismatch",
            "rejected",
            "audit_rollback_plan_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_rollback_plan: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_artifact_hash_mismatch",
            "rejected",
            "rollback_artifact_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_binds_artifact: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_service_slot_mismatch",
            "rejected",
            "rollback_service_slot_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_binds_service_slot: false,
                ..valid_requirements
            },
        ),
    ]
}

fn module_load_gate_service_slot_selftest_cases(
) -> [ModuleLoadGateServiceSlotSelfTestCase; MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES] {
    let valid_gate = module_load_gate_test_service_slot_candidate();
    let valid_reservation = module_load_gate_test_service_slot_reservation();
    let substituted_reservation = module_load_gate_test_service_slot_reservation_with_override(
        Some([0x91; 32]),
        None,
        None,
        None,
        None,
        None,
    );
    let computed_grant_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            Some([0x92; 32]),
            None,
            None,
            None,
            None,
            None,
        );
    let audit_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            Some([0x93; 32]),
            None,
            None,
            None,
            None,
        );
    let rollback_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            Some([0x94; 32]),
            None,
            None,
            None,
        );
    let inventory_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            None,
            Some([0x95; 32]),
            None,
            None,
        );
    let service_slot_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            None,
            None,
            Some("ram_only:svc.test.other"),
            None,
        );
    let reservation_hash_mismatch = module_load_gate_test_service_slot_reservation_with_override(
        None,
        None,
        None,
        None,
        None,
        Some([0x96; 32]),
    );

    [
        module_load_gate_service_slot_selftest_case(
            "missing_retained_service_slot_reservation",
            "missing",
            "retained_service_slot_reservation_missing",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: None,
                    candidate_reservation: None,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "accepted_current_boot_reservation_still_denied",
            "retained_hash_reference_only_not_allocated",
            "retained_service_slot_reservation_not_allocated",
            valid_gate,
        ),
        module_load_gate_service_slot_selftest_case(
            "stale_dropped_retained_service_slot_reservation_event_id",
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    retained: false,
                    event_reservation: valid_reservation,
                    candidate_reservation: valid_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "substituted_retained_service_slot_reservation",
            "rejected",
            "retained_service_slot_reservation_substituted_record",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: valid_reservation,
                    candidate_reservation: substituted_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_grant_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    grant_event_schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_audit_rollback_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    audit_event_schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_computed_grant_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: computed_grant_mismatch_reservation,
                    candidate_reservation: computed_grant_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_audit_record_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: audit_hash_mismatch_reservation,
                    candidate_reservation: audit_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_rollback_plan_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: rollback_hash_mismatch_reservation,
                    candidate_reservation: rollback_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_inventory_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: inventory_hash_mismatch_reservation,
                    candidate_reservation: inventory_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_service_slot_mismatch",
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: service_slot_mismatch_reservation,
                    candidate_reservation: service_slot_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_reservation_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: reservation_hash_mismatch,
                    candidate_reservation: reservation_hash_mismatch,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
    ]
}

fn module_load_gate_retained_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedSelfTestCase {
    let actual = evaluate_module_load_gate_retained_candidate(candidate);
    ModuleLoadGateRetainedSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_audit_rollback_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackSelfTestCase {
    let actual = evaluate_module_load_gate_audit_rollback_candidate(candidate);
    ModuleLoadGateAuditRollbackSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_service_slot_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotSelfTestCase {
    let actual = evaluate_module_load_gate_service_slot_candidate(candidate);
    let expected_hash_exposed = method_eq(
        expected_status,
        "retained_hash_reference_only_not_allocated",
    );
    ModuleLoadGateServiceSlotSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_service_slot_state: actual.service_slot_state,
        accepted_service_slot_reservation_hash: actual.accepted_service_slot_reservation_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && actual.accepted_service_slot_reservation_hash == expected_hash_exposed
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_audit_rollback_candidate() -> ModuleLoadGateAuditRollbackCandidate {
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    ModuleLoadGateAuditRollbackCandidate {
        retained_reference: true,
        retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            event_reference: audit_rollback_reference,
            candidate_reference: audit_rollback_reference,
        },
        durable_audit_record: true,
        rollback_plan: true,
        audit_schema_ok: true,
        rollback_schema_ok: true,
        audit_binds_retained_grant: true,
        audit_binds_manifest: true,
        audit_binds_artifact: true,
        audit_binds_vm_report: true,
        audit_binds_local_attestation: true,
        audit_binds_local_approval: true,
        audit_binds_rollback_plan: true,
        rollback_binds_artifact: true,
        rollback_binds_service_slot: true,
        ram_only_service_slot_allocated: false,
        loader_available: false,
    }
}

fn module_load_gate_test_audit_rollback_reference(
    ram_only_service_slot_id: &'static str,
) -> Option<event_log::ModuleAuditRollbackReference> {
    module_load_gate_test_audit_rollback_reference_with_manifest(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        ram_only_service_slot_id,
    )
}

fn module_load_gate_test_audit_rollback_reference_with_manifest(
    manifest_hash: [u8; 32],
    ram_only_service_slot_id: &'static str,
) -> Option<event_log::ModuleAuditRollbackReference> {
    module_load_gate_test_audit_rollback_reference_with_override(
        ram_only_service_slot_id,
        None,
        None,
        None,
    )
    .map(|mut reference| {
        reference.manifest_hash = manifest_hash;
        reference.computed_grant_hash = computed_module_grant_hash(
            manifest_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        );
        reference.audit_record_hash =
            computed_module_audit_record_hash(ModuleAuditRecordHashInput {
                denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
                retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
                computed_grant_hash: reference.computed_grant_hash,
                manifest_hash: reference.manifest_hash,
                artifact_hash: reference.artifact_hash,
                vm_report_hash: reference.vm_report_hash,
                local_attestation_hash: reference.local_attestation_hash,
                local_approval_hash: reference.local_approval_hash,
                rollback_plan_hash: reference.rollback_plan_hash,
                ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
            });
        reference
    })
}

fn module_load_gate_test_audit_rollback_reference_with_override(
    ram_only_service_slot_id: &'static str,
    computed_grant_hash_override: Option<[u8; 32]>,
    rollback_plan_hash_override: Option<[u8; 32]>,
    audit_record_hash_override: Option<[u8; 32]>,
) -> Option<event_log::ModuleAuditRollbackReference> {
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let rollback_plan_hash = computed_module_rollback_plan_hash(
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
        ram_only_service_slot_id,
        MODULE_AUDIT_TEST_CLEANUP_HASH,
    );
    let audit_record_hash = computed_module_audit_record_hash(ModuleAuditRecordHashInput {
        denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
        retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        computed_grant_hash,
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        rollback_plan_hash,
        ram_only_service_slot_id,
    });

    Some(event_log::ModuleAuditRollbackReference {
        audit_record_hash: audit_record_hash_override.unwrap_or(audit_record_hash),
        rollback_plan_hash: rollback_plan_hash_override.unwrap_or(rollback_plan_hash),
        computed_grant_hash: computed_grant_hash_override.unwrap_or(computed_grant_hash),
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        pre_load_service_inventory_hash: MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
        cleanup_actions_hash: MODULE_AUDIT_TEST_CLEANUP_HASH,
        denial_event_id: parse_current_boot_event_id(MODULE_AUDIT_TEST_DENIAL_EVENT_ID)?,
        retained_reference_event_id: parse_current_boot_event_id(
            MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        )?,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(ram_only_service_slot_id)?,
    })
}

fn module_load_gate_test_service_slot_candidate() -> ModuleLoadGateServiceSlotCandidate {
    let retained_reference = Some(module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    ));
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let service_slot_reservation = module_load_gate_test_service_slot_reservation();

    ModuleLoadGateServiceSlotCandidate {
        retained_reference,
        audit_rollback_reference,
        audit_rollback_valid: true,
        service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            grant_event_schema_ok: true,
            audit_event_schema_ok: true,
            grant_event_reference: retained_reference,
            audit_event_reference: audit_rollback_reference,
            event_reservation: service_slot_reservation,
            candidate_reservation: service_slot_reservation,
        },
    }
}

fn module_load_gate_test_service_slot_reservation(
) -> Option<event_log::ModuleServiceSlotReservation> {
    module_load_gate_test_service_slot_reservation_with_override(None, None, None, None, None, None)
}

fn module_load_gate_test_service_slot_reservation_with_override(
    computed_grant_hash_override: Option<[u8; 32]>,
    audit_record_hash_override: Option<[u8; 32]>,
    rollback_plan_hash_override: Option<[u8; 32]>,
    pre_load_service_inventory_hash_override: Option<[u8; 32]>,
    ram_only_service_slot_id_override: Option<&'static str>,
    reservation_hash_override: Option<[u8; 32]>,
) -> Option<event_log::ModuleServiceSlotReservation> {
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID)?;
    let ram_only_service_slot_id =
        ram_only_service_slot_id_override.unwrap_or(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let computed_grant_hash =
        computed_grant_hash_override.unwrap_or(audit_rollback_reference.computed_grant_hash);
    let audit_record_hash =
        audit_record_hash_override.unwrap_or(audit_rollback_reference.audit_record_hash);
    let rollback_plan_hash =
        rollback_plan_hash_override.unwrap_or(audit_rollback_reference.rollback_plan_hash);
    let pre_load_service_inventory_hash = pre_load_service_inventory_hash_override
        .unwrap_or(audit_rollback_reference.pre_load_service_inventory_hash);
    let reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash,
            audit_record_hash,
            rollback_plan_hash,
            pre_load_service_inventory_hash,
            ram_only_service_slot_id,
        });

    Some(event_log::ModuleServiceSlotReservation {
        reservation_hash: reservation_hash_override.unwrap_or(reservation_hash),
        retained_reference_event_id: parse_current_boot_event_id(
            MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        )?,
        retained_audit_rollback_reference_event_id: parse_current_boot_event_id(
            MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
        )?,
        computed_grant_hash,
        audit_record_hash,
        rollback_plan_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(ram_only_service_slot_id)?,
    })
}

fn module_load_gate_test_reference(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> event_log::ModuleComputedGrantReference {
    event_log::ModuleComputedGrantReference {
        computed_grant_hash: computed_module_grant_hash(
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
        ),
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    }
}

fn evaluate_module_load_gate_manifest_candidate(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestEvaluation {
    if candidate.candidate_reference.is_none() {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    }
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_substituted_record",
        );
    }
    let Some(reference) = candidate.candidate_reference else {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    };
    if reference.manifest_reference_hash
        != computed_module_manifest_reference_hash(reference.manifest_hash)
    {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
        );
    }
    module_load_gate_manifest_check(
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
    )
}

fn module_load_gate_manifest_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateManifestEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateManifestEvaluation {
        status,
        reason,
        module_manifest_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_manifest_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_artifact_candidate(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_artifact_check(
            "missing",
            "retained_candidate_artifact_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_substituted_record",
        );
    }
    if candidate_reference.artifact_reference_hash
        != module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.artifact_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.vm_report_hash != retained_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.artifact_hash != retained_reference.artifact_hash {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_hash_mismatch",
        );
    }

    module_load_gate_artifact_check(
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
    )
}

fn evaluate_module_load_gate_vm_report_candidate(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_vm_report_check(
            "missing",
            "retained_vm_test_report_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
        );
    }
    if candidate_reference.report_reference_hash
        != module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference
                .retained_artifact_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.artifact_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.artifact_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    }

    let (Some(artifact_event_id), Some(artifact_reference)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    };
    if candidate_reference.retained_artifact_reference_event_id != artifact_event_id
        || candidate_reference.artifact_reference_hash != artifact_reference.artifact_reference_hash
        || candidate_reference.manifest_reference_hash != artifact_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != artifact_reference.manifest_hash
        || candidate_reference.artifact_hash != artifact_reference.artifact_hash
        || candidate_reference.local_attestation_hash != artifact_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != artifact_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.artifact_hash != retained_reference.artifact_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != retained_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    module_load_gate_vm_report_check(
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
    )
}

fn module_load_gate_artifact_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateArtifactEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateArtifactEvaluation {
        status,
        reason,
        candidate_artifact_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_artifact_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_vm_report_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateVmReportEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateVmReportEvaluation {
        status,
        reason,
        vm_test_report_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_vm_report_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "computed_capability_grant_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_substituted_record",
        );
    }
    if !module_computed_grant_reference_hashes_consistent(candidate_reference) {
        return module_load_gate_retained_check("rejected", "retained_reference_hash_mismatch");
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
    )
}

fn module_load_gate_retained_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateRetainedCheck {
    ModuleLoadGateRetainedCheck {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_audit_rollback_candidate(
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackEvaluation {
    if !candidate.retained_reference {
        return module_load_gate_audit_rollback_check(
            "missing",
            "retained_computed_grant_reference_missing",
        );
    }
    let retained_audit_rollback_check =
        evaluate_module_load_gate_audit_rollback_reference_candidate(
            candidate.retained_audit_rollback_reference,
        );
    if !method_eq(
        retained_audit_rollback_check.status,
        "retained_hash_reference_only",
    ) {
        return module_load_gate_audit_rollback_check(
            retained_audit_rollback_check.status,
            retained_audit_rollback_check.reason,
        );
    }
    if !candidate.durable_audit_record {
        return module_load_gate_audit_rollback_check("missing", "durable_audit_record_missing");
    }
    if !candidate.rollback_plan {
        return module_load_gate_audit_rollback_check("missing", "rollback_plan_missing");
    }
    if !candidate.audit_schema_ok {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "durable_audit_record_schema_mismatch",
        );
    }
    if !candidate.rollback_schema_ok {
        return module_load_gate_audit_rollback_check("rejected", "rollback_plan_schema_mismatch");
    }
    if !candidate.audit_binds_retained_grant {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_retained_grant_hash_mismatch",
        );
    }
    if !candidate.audit_binds_manifest {
        return module_load_gate_audit_rollback_check("rejected", "audit_manifest_hash_mismatch");
    }
    if !candidate.audit_binds_artifact {
        return module_load_gate_audit_rollback_check("rejected", "audit_artifact_hash_mismatch");
    }
    if !candidate.audit_binds_vm_report {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_vm_test_report_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_attestation {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_local_attestation_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_approval {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "local_approval_missing_or_mismatch",
        );
    }
    if !candidate.audit_binds_rollback_plan {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_rollback_plan_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_artifact {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "rollback_artifact_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_service_slot {
        return module_load_gate_audit_rollback_check("rejected", "rollback_service_slot_mismatch");
    }
    if !candidate.ram_only_service_slot_allocated && !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
        );
    }
    if !candidate.ram_only_service_slot_allocated {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "ram_only_service_slot_unallocated",
        );
    }
    if !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "module_loader_unimplemented",
        );
    }
    module_load_gate_audit_rollback_check("rejected", "positive_loader_path_unimplemented")
}

fn evaluate_module_load_gate_audit_rollback_reference_candidate(
    candidate: ModuleLoadGateAuditRollbackReferenceCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
        );
    }
    if candidate_reference.ram_only_service_slot_id.as_str()
        != MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID
    {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_audit_rollback_reference_hash_mismatch(candidate_reference) {
        return module_load_gate_retained_check("rejected", reason);
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_audit_rollback_reference_not_authorizing",
    )
}

fn evaluate_module_load_gate_service_slot_candidate(
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotEvaluation {
    let Some(reservation) = candidate.service_slot_reservation.candidate_reservation else {
        return module_load_gate_service_slot_check(
            "missing",
            "retained_service_slot_reservation_missing",
        );
    };
    let Some(retained_reference) = candidate.retained_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_computed_grant_reference_missing",
        );
    };
    let Some(audit_rollback_reference) = candidate.audit_rollback_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !candidate.audit_rollback_valid {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_not_valid_for_service_slot",
        );
    }

    let Some(retained_reference_event_id) =
        parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    };
    let Some(audit_rollback_event_id) =
        parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    };

    if reservation.retained_reference_event_id != retained_reference_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    }
    if reservation.retained_audit_rollback_reference_event_id != audit_rollback_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    }

    let service_slot_candidate = candidate.service_slot_reservation;
    if !method_eq(service_slot_candidate.scope, "current_boot") || !service_slot_candidate.retained
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    }
    if !service_slot_candidate.schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(event_reservation) = service_slot_candidate.event_reservation else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_service_slot_reservation_matches(event_reservation, reservation) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.grant_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(grant_event_reference) = service_slot_candidate.grant_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(retained_reference, grant_event_reference) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.audit_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(audit_event_reference) = service_slot_candidate.audit_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(
        audit_rollback_reference,
        audit_event_reference,
    ) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if reservation.computed_grant_hash != retained_reference.computed_grant_hash
        || reservation.computed_grant_hash != audit_rollback_reference.computed_grant_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
        );
    }
    if reservation.audit_record_hash != audit_rollback_reference.audit_record_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
        );
    }
    if reservation.rollback_plan_hash != audit_rollback_reference.rollback_plan_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
        );
    }
    if reservation.pre_load_service_inventory_hash
        != audit_rollback_reference.pre_load_service_inventory_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
        );
    }
    if reservation.ram_only_service_slot_id.as_str()
        != audit_rollback_reference.ram_only_service_slot_id.as_str()
        || !module_evidence::ram_only_service_slot_id_valid(
            reservation.ram_only_service_slot_id.as_str(),
        )
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_service_slot_reservation_hash_mismatch(reservation) {
        return module_load_gate_service_slot_check("rejected", reason);
    }

    module_load_gate_service_slot_check(
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
    )
}

fn module_audit_rollback_event_reference_matches(
    event_reference: event_log::ModuleAuditRollbackReference,
    candidate_reference: event_log::ModuleAuditRollbackReference,
) -> bool {
    event_reference.audit_record_hash == candidate_reference.audit_record_hash
        && event_reference.rollback_plan_hash == candidate_reference.rollback_plan_hash
        && event_reference.computed_grant_hash == candidate_reference.computed_grant_hash
        && event_reference.manifest_hash == candidate_reference.manifest_hash
        && event_reference.artifact_hash == candidate_reference.artifact_hash
        && event_reference.vm_report_hash == candidate_reference.vm_report_hash
        && event_reference.local_attestation_hash == candidate_reference.local_attestation_hash
        && event_reference.local_approval_hash == candidate_reference.local_approval_hash
        && event_reference.pre_load_service_inventory_hash
            == candidate_reference.pre_load_service_inventory_hash
        && event_reference.cleanup_actions_hash == candidate_reference.cleanup_actions_hash
        && event_reference.denial_event_id == candidate_reference.denial_event_id
        && event_reference.retained_reference_event_id
            == candidate_reference.retained_reference_event_id
        && event_reference.ram_only_service_slot_id.as_str()
            == candidate_reference.ram_only_service_slot_id.as_str()
}

fn module_audit_rollback_reference_hash_mismatch(
    reference: event_log::ModuleAuditRollbackReference,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_DENIAL_EVENT_ID)
        != Some(reference.denial_event_id)
        || parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
            != Some(reference.retained_reference_event_id)
    {
        return Some("retained_audit_rollback_reference_substituted_record");
    }

    let expected_computed_grant_hash = computed_module_grant_hash(
        reference.manifest_hash,
        reference.artifact_hash,
        reference.vm_report_hash,
        reference.local_attestation_hash,
    );
    if reference.computed_grant_hash != expected_computed_grant_hash {
        return Some("retained_audit_rollback_computed_grant_hash_mismatch");
    }

    let expected_rollback_plan_hash = computed_module_rollback_plan_hash(
        reference.artifact_hash,
        reference.pre_load_service_inventory_hash,
        reference.ram_only_service_slot_id.as_str(),
        reference.cleanup_actions_hash,
    );
    if reference.rollback_plan_hash != expected_rollback_plan_hash {
        return Some("retained_rollback_plan_hash_mismatch");
    }

    let expected_audit_record_hash =
        computed_module_audit_record_hash(ModuleAuditRecordHashInput {
            denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            computed_grant_hash: reference.computed_grant_hash,
            manifest_hash: reference.manifest_hash,
            artifact_hash: reference.artifact_hash,
            vm_report_hash: reference.vm_report_hash,
            local_attestation_hash: reference.local_attestation_hash,
            local_approval_hash: reference.local_approval_hash,
            rollback_plan_hash: reference.rollback_plan_hash,
            ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
        });
    if reference.audit_record_hash != expected_audit_record_hash {
        return Some("retained_audit_record_hash_mismatch");
    }

    None
}

fn module_service_slot_reservation_matches(
    left: event_log::ModuleServiceSlotReservation,
    right: event_log::ModuleServiceSlotReservation,
) -> bool {
    left.reservation_hash == right.reservation_hash
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.retained_audit_rollback_reference_event_id
            == right.retained_audit_rollback_reference_event_id
        && left.computed_grant_hash == right.computed_grant_hash
        && left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.ram_only_service_slot_id.as_str() == right.ram_only_service_slot_id.as_str()
}

fn module_service_slot_reservation_hash_mismatch(
    reservation: event_log::ModuleServiceSlotReservation,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
        != Some(reservation.retained_reference_event_id)
        || parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
            != Some(reservation.retained_audit_rollback_reference_event_id)
    {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    let expected_reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash: reservation.computed_grant_hash,
            audit_record_hash: reservation.audit_record_hash,
            rollback_plan_hash: reservation.rollback_plan_hash,
            pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
            ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
        });
    if reservation.reservation_hash != expected_reservation_hash {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    None
}

fn module_load_gate_service_slot_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateServiceSlotEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only_not_allocated");
    let service_slot_state = if accepted {
        "retained_hash_reference_only_not_allocated"
    } else if method_eq(status, "rejected") {
        "rejected_retained_reference"
    } else {
        "unallocated"
    };
    ModuleLoadGateServiceSlotEvaluation {
        status,
        reason,
        service_slot_state,
        accepted_service_slot_reservation_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_audit_rollback_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateAuditRollbackEvaluation {
    ModuleLoadGateAuditRollbackEvaluation {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}

fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    module_evidence::computed_module_manifest_reference_hash(manifest_hash)
}

fn computed_module_candidate_artifact_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_candidate_artifact_reference_hash(
        module_evidence::ModuleCandidateArtifactReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            manifest_hash,
            computed_grant_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
        },
    )
}

fn computed_module_vm_test_report_reference_hash(
    retained_manifest_reference_event_id: &str,
    retained_artifact_reference_event_id: &str,
    retained_reference_event_id: &str,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_vm_test_report_reference_hash(
        ModuleVmTestReportReferenceHashInput {
            retained_manifest_reference_event_id,
            retained_artifact_reference_event_id,
            retained_reference_event_id,
            manifest_reference_hash,
            artifact_reference_hash,
            manifest_hash,
            artifact_hash,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
        },
    )
}

fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}

fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    )
}

fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    module_evidence::computed_module_audit_record_hash(input)
}

fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    module_evidence::computed_module_service_slot_reservation_hash(input)
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

fn module_load_gate_manifest_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_manifest_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_module_manifest_reference_not_authorizing"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        binding.manifest_reference_reason
    } else {
        "module_manifest_missing"
    }
}

fn module_load_gate_candidate_artifact_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_candidate_artifact_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_candidate_artifact_reference_not_authorizing"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        binding.artifact_reference_reason
    } else {
        "candidate_artifact_missing"
    }
}

fn module_load_gate_vm_test_report_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_vm_test_report_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_vm_test_report_reference_not_authorizing"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        binding.vm_report_reference_reason
    } else {
        "vm_test_report_missing"
    }
}

fn module_load_gate_computed_grant_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_hash_reference_only"
    } else {
        "missing"
    }
}

fn module_load_gate_computed_grant_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_computed_grant_reference_not_authorizing"
    } else {
        "computed_capability_grant_missing"
    }
}

fn module_load_gate_durable_audit_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_durable"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_durable_audit_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_audit_record_reference_not_durable"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "durable_audit_record_missing"
    }
}

fn module_load_gate_rollback_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_installed"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_rollback_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_rollback_plan_reference_not_installed"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "rollback_plan_missing"
    }
}

fn module_load_gate_service_slot_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_hash_reference_only_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "unallocated"
    }
}

fn module_load_gate_service_slot_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_service_slot_reservation_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        binding.service_slot_reservation_reason
    } else {
        "ram_only_service_slot_unallocated"
    }
}

fn module_load_gate_audit_rollback_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.audit_rollback_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_audit_rollback_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.audit_rollback_reference_status, "rejected")
}

fn module_load_gate_service_slot_reservation_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.service_slot_reservation_status,
        "retained_hash_reference_only_not_allocated",
    )
}

fn module_load_gate_service_slot_reservation_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.service_slot_reservation_status, "rejected")
}

fn module_load_gate_manifest_reference_valid(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(
        binding.manifest_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_manifest_reference_rejected(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(binding.manifest_reference_status, "rejected")
}

fn module_load_gate_candidate_artifact_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.artifact_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_candidate_artifact_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.artifact_reference_status, "rejected")
}

fn module_load_gate_vm_test_report_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.vm_report_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_vm_test_report_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.vm_report_reference_status, "rejected")
}

fn emit_module_load_gate_manifest_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_module_manifest_reference\": {");
    if let Some(reference) = binding.manifest_reference {
        if module_load_gate_manifest_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.manifest_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.manifest_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.manifest_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.manifest_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw_line("      \"hashes\": {");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.manifest_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.manifest_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_artifact_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_candidate_artifact_reference\": {");
    if let Some(reference) = binding.artifact_reference {
        if module_load_gate_candidate_artifact_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.artifact_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.artifact_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.artifact_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.artifact_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.artifact_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.artifact_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_vm_report_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_vm_test_report_reference\": {");
    if let Some(reference) = binding.vm_report_reference {
        if module_load_gate_vm_test_report_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.vm_report_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.vm_report_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.vm_report_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.vm_report_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_vm_report_json\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.vm_report_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.vm_report_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_retained_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_computed_grant_reference\": {");
    if let Some(reference) = binding.retained_reference {
        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw_line("      \"hashes\": {");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("      \"status\": \"missing\",");
        raw_line("      \"reason\": \"no_valid_computed_grant_reference_retained\",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_audit_rollback_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_audit_rollback_reference\": {");
    if let Some(reference) = binding.audit_rollback_reference {
        if module_load_gate_audit_rollback_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.audit_rollback_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.audit_rollback_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.audit_rollback_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"durable_audit_written\": false,");
            raw_line("      \"rollback_plan_installed\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.audit_rollback_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"durable_audit_written\": false,");
        raw_line("      \"rollback_plan_installed\": false,");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw("      \"denial_event_id\": ");
        json_event_id(reference.denial_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
        raw("        \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("        \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.audit_rollback_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.audit_rollback_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_service_slot_reservation(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_service_slot_reservation\": {");
    if let Some(reservation) = binding.service_slot_reservation {
        if module_load_gate_service_slot_reservation_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.service_slot_reservation_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
            raw("      \"status\": ");
            json_str(binding.service_slot_reservation_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.service_slot_reservation_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"allocates_service_slot\": false,");
            raw_line("      \"creates_service_inventory_records\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.service_slot_reservation_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_only_not_allocated\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"allocates_service_slot\": false,");
        raw_line("      \"creates_service_inventory_records\": false,");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reservation.retained_reference_event_id);
        raw_line(",");
        raw("      \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reservation.retained_audit_rollback_reference_event_id);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reservation.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reservation.computed_grant_hash);
        raw_line(",");
        raw("        \"audit_record_hash\": ");
        json_sha256(reservation.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reservation.rollback_plan_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reservation.pre_load_service_inventory_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw("      \"status\": ");
        json_str(binding.service_slot_reservation_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.service_slot_reservation_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_evidence_hashes(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("      \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("      \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
    } else {
        raw_line("      \"computed_capability_grant_hash\": null,");
        raw_line("      \"local_attestation_hash\": null,");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw("      \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("      \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
    } else {
        raw_line("      \"vm_test_report_reference_hash\": null,");
        raw_line("      \"vm_test_report_hash\": null,");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw("      \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("      \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
    } else {
        raw_line("      \"artifact_reference_hash\": null,");
        raw_line("      \"artifact_hash\": null,");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw("      \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("      \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
    } else {
        raw_line("      \"manifest_reference_hash\": null,");
        raw_line("      \"manifest_hash\": null,");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw("      \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("      \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("      \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("      \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("      \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
    } else {
        raw_line("      \"audit_record_hash\": null,");
        raw_line("      \"rollback_plan_hash\": null,");
        raw_line("      \"local_approval_hash\": null,");
        raw_line("      \"pre_load_service_inventory_hash\": null,");
        raw_line("      \"cleanup_actions_hash\": null,");
        raw_line("      \"ram_only_service_slot_id\": null,");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw("      \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw_line(",");
    } else {
        raw_line("      \"service_slot_reservation_hash\": null,");
    }
}

fn emit_module_load_gate_audit_rollback_requirements(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"audit_rollback_requirements\": {");
    raw_line("      \"schema\": \"raios.module_load_gate_audit_rollback_requirements.v0\",");
    raw_line("      \"classification\": \"public\",");
    raw_line("      \"status\": \"required_missing\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"durable_audit_record\": {");
    raw_line("        \"schema\": \"raios.audit_record.v0\",");
    raw("        \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw_line(",");
    raw_line("        \"durability\": \"required_before_load\",");
    raw_line("        \"required_bindings\": [");
    raw_line("          \"denial_event_id\",");
    raw_line("          \"retained_computed_grant_reference_event_id\",");
    raw_line("          \"computed_capability_grant_hash\",");
    raw_line("          \"manifest_hash\",");
    raw_line("          \"artifact_hash\",");
    raw_line("          \"vm_test_report_hash\",");
    raw_line("          \"local_attestation_hash\",");
    raw_line("          \"local_approval\",");
    raw_line("          \"rollback_plan_hash\",");
    raw_line("          \"ram_only_service_slot_id\"");
    raw_line("        ]");
    raw_line("      },");
    raw_line("      \"rollback_plan\": {");
    raw_line("        \"schema\": \"raios.rollback_plan.v0\",");
    raw("        \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw_line(",");
    raw_line("        \"must_preexist_load\": true,");
    raw_line("        \"required_bindings\": [");
    raw_line("          \"artifact_hash\",");
    raw_line("          \"pre_load_service_inventory_hash\",");
    raw_line("          \"ram_only_service_slot_id\",");
    raw_line("          \"cleanup_actions_hash\"");
    raw_line("        ]");
    raw_line("      },");
    raw_line("      \"required_hashes\": {");
    emit_module_load_gate_required_hashes(binding);
    raw_line("      },");
    raw("      \"retained_reference_event_id\": ");
    json_event_id_option(binding.retained_reference_event_id);
    raw_line(",");
    raw("      \"retained_manifest_reference_event_id\": ");
    json_event_id_option(binding.manifest_reference_event_id);
    raw_line(",");
    raw("      \"retained_audit_rollback_reference_event_id\": ");
    json_event_id_option(binding.audit_rollback_reference_event_id);
    raw_line(",");
    raw("      \"retained_service_slot_reservation_event_id\": ");
    json_event_id_option(binding.service_slot_reservation_event_id);
    raw_line(",");
    raw_line("      \"local_approval\": {\"state\": \"missing\", \"required\": true},");
    raw("      \"ram_only_service_slot\": {\"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw_line(", \"required\": true, \"allocates_service_slot\": false},");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load\": false");
    raw("    }");
}

fn emit_module_load_gate_required_hashes(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
    } else {
        raw_line("        \"computed_capability_grant_hash\": null,");
        raw_line("        \"local_attestation_hash\": null,");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
    } else {
        raw_line("        \"vm_test_report_reference_hash\": null,");
        raw_line("        \"vm_test_report_hash\": null,");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
    } else {
        raw_line("        \"artifact_reference_hash\": null,");
        raw_line("        \"artifact_hash\": null,");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
    } else {
        raw_line("        \"manifest_reference_hash\": null,");
        raw_line("        \"manifest_hash\": null,");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw("        \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("        \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("        \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw_line(",");
        raw("        \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
    } else {
        raw_line("        \"audit_record_hash\": null,");
        raw_line("        \"rollback_plan_hash\": null,");
        raw_line("        \"local_approval_hash\": null,");
        raw_line("        \"pre_load_service_inventory_hash\": null,");
        raw_line("        \"cleanup_actions_hash\": null,");
        raw_line("        \"ram_only_service_slot_id\": null,");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw("        \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        crlf();
    } else {
        raw_line("        \"service_slot_reservation_hash\": null");
    }
}

fn emit_module_load_ephemeral_denied(
    method: &'static str,
    event_id: event_log::EventId,
    gate_binding: event_log::ModuleLoadGateBinding,
) {
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
    raw_line("    \"schema\": \"raios.module_load_gate.v0\",");
    raw("    \"message\": ");
    json_str("ephemeral module loading is denied until a manifest, exact artifact, VM test report, local attestation, computed capability grant, audit record, and rollback plan are bound");
    raw_line(",");
    raw_line("    \"request\": {");
    raw_line("      \"load_mode\": \"ram_only\",");
    raw_line("      \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("      \"risk\": \"modify_ram\",");
    raw_line("      \"target\": \"live_service_graph\",");
    raw_line("      \"subject\": \"agent.session.serial\"");
    raw_line("    },");
    raw_line("    \"gate_state\": {");
    raw("      \"module_manifest\": ");
    json_str(module_load_gate_manifest_state(gate_binding));
    raw_line(",");
    raw("      \"candidate_artifact\": ");
    json_str(module_load_gate_candidate_artifact_state(gate_binding));
    raw_line(",");
    raw("      \"vm_test_report\": ");
    json_str(module_load_gate_vm_test_report_state(gate_binding));
    raw_line(",");
    raw_line("      \"local_attestation\": \"missing\",");
    raw("      \"computed_capability_grant\": ");
    json_str(module_load_gate_computed_grant_state(gate_binding));
    raw_line(",");
    raw_line("      \"local_approval\": \"missing\",");
    raw("      \"rollback_plan\": ");
    json_str(module_load_gate_rollback_state(gate_binding));
    raw_line(",");
    raw("      \"durable_audit_record\": ");
    json_str(module_load_gate_durable_audit_state(gate_binding));
    raw_line(",");
    raw_line("      \"loader\": \"unavailable\",");
    raw("      \"service_slot\": ");
    json_str(module_load_gate_service_slot_state(gate_binding));
    raw_line(",");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"can_load\": false");
    raw_line("    },");
    emit_module_load_gate_manifest_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_artifact_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_vm_report_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_retained_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_audit_rollback_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_service_slot_reservation(gate_binding);
    raw_line(",");
    emit_module_load_gate_audit_rollback_requirements(gate_binding);
    raw_line(",");
    raw_line("    \"blocked_by\": [");
    raw("      {\"gate\": \"module_manifest\", \"state\": ");
    json_str(module_load_gate_manifest_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_manifest_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"candidate_artifact\", \"state\": ");
    json_str(module_load_gate_candidate_artifact_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_candidate_artifact_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"vm_test_report\", \"state\": ");
    json_str(module_load_gate_vm_test_report_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_vm_test_report_reason(gate_binding));
    raw_line("},");
    raw_line(
        "      {\"gate\": \"local_attestation\", \"state\": \"missing\", \"reason\": \"local_attestation_missing\"},",
    );
    raw("      {\"gate\": \"computed_capability_grant\", \"state\": ");
    json_str(module_load_gate_computed_grant_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_computed_grant_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"durable_audit_record\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_durable_audit_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"rollback_plan\", \"state\": ");
    json_str(module_load_gate_rollback_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_rollback_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"service_slot\", \"state\": ");
    json_str(module_load_gate_service_slot_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(gate_binding));
    raw_line("},");
    raw_line(
        "      {\"gate\": \"loader\", \"state\": \"unavailable\", \"reason\": \"module_loader_unimplemented\"}",
    );
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"raios.module_manifest.v0\",");
    raw_line("      \"candidate_artifact_sha256\",");
    raw_line("      \"raios.vm_test_report.v0\",");
    raw_line("      \"raios.local_attestation.v0\",");
    raw_line("      \"computed_capability_grant\",");
    raw_line("      \"local_approval\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"rollback_plan\",");
    raw_line("      \"ram_only_service_slot\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw("      \"denial_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"event_scope\": \"current_boot\",");
    emit_module_load_gate_evidence_hashes(gate_binding);
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
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

fn emit_module_load_gate_event_binding(binding: event_log::ModuleLoadGateBinding) {
    raw(", \"bindings\": {\"schema\": \"raios.module_load_gate.v0\", \"status\": \"denied_missing_evidence\", \"load_mode\": \"ram_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"risk\": \"modify_ram\", \"target\": \"live_service_graph\", \"subject\": \"agent.session.serial\", \"gate_state\": {\"module_manifest\": ");
    json_str(module_load_gate_manifest_state(binding));
    raw(", \"candidate_artifact\": ");
    json_str(module_load_gate_candidate_artifact_state(binding));
    raw(", \"vm_test_report\": ");
    json_str(module_load_gate_vm_test_report_state(binding));
    raw(", \"local_attestation\": \"missing\", \"computed_capability_grant\": ");
    json_str(module_load_gate_computed_grant_state(binding));
    raw(", \"local_approval\": \"missing\", \"rollback_plan\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"durable_audit_record\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"loader\": \"unavailable\", \"service_slot\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"artifact_loaded\": false, \"service_started\": false, \"persistence\": \"none\", \"can_load\": false}, \"retained_module_manifest_reference\": ");
    emit_module_load_gate_manifest_reference_compact(binding);
    raw(", \"retained_candidate_artifact_reference\": ");
    emit_module_load_gate_artifact_reference_compact(binding);
    raw(", \"retained_vm_test_report_reference\": ");
    emit_module_load_gate_vm_report_reference_compact(binding);
    raw(", \"retained_computed_grant_reference\": ");
    emit_module_load_gate_retained_reference_compact(binding);
    raw(", \"retained_audit_rollback_reference\": ");
    emit_module_load_gate_audit_rollback_reference_compact(binding);
    raw(", \"retained_service_slot_reservation\": ");
    emit_module_load_gate_service_slot_reservation_compact(binding);
    raw(", \"audit_rollback_requirements\": ");
    emit_module_load_gate_audit_rollback_requirements_compact(binding);
    raw(", \"blocked_by\": [{\"gate\": \"module_manifest\", \"state\": ");
    json_str(module_load_gate_manifest_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_manifest_reason(binding));
    raw("}, {\"gate\": \"candidate_artifact\", \"state\": ");
    json_str(module_load_gate_candidate_artifact_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_candidate_artifact_reason(binding));
    raw("}, {\"gate\": \"vm_test_report\", \"state\": ");
    json_str(module_load_gate_vm_test_report_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_vm_test_report_reason(binding));
    raw("}, {\"gate\": \"local_attestation\", \"state\": \"missing\", \"reason\": \"local_attestation_missing\"}, {\"gate\": \"computed_capability_grant\", \"state\": ");
    json_str(module_load_gate_computed_grant_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_computed_grant_reason(binding));
    raw("}, {\"gate\": \"durable_audit_record\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_durable_audit_reason(binding));
    raw("}, {\"gate\": \"rollback_plan\", \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_rollback_reason(binding));
    raw("}, {\"gate\": \"service_slot\", \"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw("}, {\"gate\": \"loader\", \"state\": \"unavailable\", \"reason\": \"module_loader_unimplemented\"}], \"required\": [\"raios.module_manifest.v0\", \"candidate_artifact_sha256\", \"raios.vm_test_report.v0\", \"raios.local_attestation.v0\", \"raios.computed_capability_grant.v0\", \"local_approval\", \"raios.audit_record.v0\", \"rollback_plan\", \"ram_only_service_slot\"], \"evidence\": {\"event_scope\": \"current_boot\", ");
    emit_module_load_gate_evidence_hashes_compact(binding);
    raw(", \"service_inventory_change\": \"none\", \"load_attempted\": false}}");
}

fn emit_module_load_gate_manifest_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.manifest_reference {
        if module_load_gate_manifest_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.manifest_reference_event_id);
            raw(", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
            json_str(binding.manifest_reference_status);
            raw(", \"reason\": ");
            json_str(binding.manifest_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.manifest_reference_event_id);
        raw(", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
        json_str(binding.manifest_reference_status);
        raw(", \"reason\": ");
        json_str(binding.manifest_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_artifact_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.artifact_reference {
        if module_load_gate_candidate_artifact_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.artifact_reference_event_id);
            raw(", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
            json_str(binding.artifact_reference_status);
            raw(", \"reason\": ");
            json_str(binding.artifact_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.artifact_reference_event_id);
        raw(", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
        json_str(binding.artifact_reference_status);
        raw(", \"reason\": ");
        json_str(binding.artifact_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_vm_report_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.vm_report_reference {
        if module_load_gate_vm_test_report_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.vm_report_reference_event_id);
            raw(", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
            json_str(binding.vm_report_reference_status);
            raw(", \"reason\": ");
            json_str(binding.vm_report_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.vm_report_reference_event_id);
        raw(", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_vm_report_json\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
        json_str(binding.vm_report_reference_status);
        raw(", \"reason\": ");
        json_str(binding.vm_report_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_retained_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.retained_reference_event_id);
        raw(", \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"hashes\": {\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"missing\", \"reason\": \"no_valid_computed_grant_reference_retained\", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_audit_rollback_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.audit_rollback_reference {
        if module_load_gate_audit_rollback_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.audit_rollback_reference_event_id);
            raw(", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
            json_str(binding.audit_rollback_reference_status);
            raw(", \"reason\": ");
            json_str(binding.audit_rollback_reference_reason);
            raw(", \"classification\": \"local_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.audit_rollback_reference_event_id);
        raw(", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"denial_event_id\": ");
        json_event_id(reference.denial_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
        json_str(binding.audit_rollback_reference_status);
        raw(", \"reason\": ");
        json_str(binding.audit_rollback_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_service_slot_reservation_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reservation) = binding.service_slot_reservation {
        if module_load_gate_service_slot_reservation_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.service_slot_reservation_event_id);
            raw(", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
            json_str(binding.service_slot_reservation_status);
            raw(", \"reason\": ");
            json_str(binding.service_slot_reservation_reason);
            raw(", \"classification\": \"local_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.service_slot_reservation_event_id);
        raw(", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": \"retained_hash_reference_only_not_allocated\", \"classification\": \"local_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reservation.retained_reference_event_id);
        raw(", \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reservation.retained_audit_rollback_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reservation.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reservation.computed_grant_hash);
        raw(", \"audit_record_hash\": ");
        json_sha256(reservation.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reservation.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reservation.pre_load_service_inventory_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
        json_str(binding.service_slot_reservation_status);
        raw(", \"reason\": ");
        json_str(binding.service_slot_reservation_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_evidence_hashes_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
    } else {
        raw("\"computed_capability_grant_hash\": null, \"local_attestation_hash\": null");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
    } else {
        raw(", \"vm_test_report_reference_hash\": null, \"vm_test_report_hash\": null");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
    } else {
        raw(", \"artifact_reference_hash\": null, \"artifact_hash\": null");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
    } else {
        raw(", \"manifest_reference_hash\": null, \"manifest_hash\": null");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw(", \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw(", \"audit_record_hash\": null, \"rollback_plan_hash\": null, \"local_approval_hash\": null, \"pre_load_service_inventory_hash\": null, \"cleanup_actions_hash\": null, \"ram_only_service_slot_id\": null");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw(", \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
    } else {
        raw(", \"service_slot_reservation_hash\": null");
    }
}

fn emit_module_load_gate_audit_rollback_requirements_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw("{\"schema\": \"raios.module_load_gate_audit_rollback_requirements.v0\", \"classification\": \"public\", \"status\": \"required_missing\", \"writes_enabled\": false, \"creates_durable_audit_records\": false, \"creates_rollback_plans\": false, \"durable_audit_record\": {\"schema\": \"raios.audit_record.v0\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"durability\": \"required_before_load\", \"required_bindings\": [\"denial_event_id\", \"retained_computed_grant_reference_event_id\", \"computed_capability_grant_hash\", \"manifest_hash\", \"artifact_hash\", \"vm_test_report_hash\", \"local_attestation_hash\", \"local_approval\", \"rollback_plan_hash\", \"ram_only_service_slot_id\"]}, \"rollback_plan\": {\"schema\": \"raios.rollback_plan.v0\", \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"must_preexist_load\": true, \"required_bindings\": [\"artifact_hash\", \"pre_load_service_inventory_hash\", \"ram_only_service_slot_id\", \"cleanup_actions_hash\"]}, \"required_hashes\": {");
    emit_module_load_gate_required_hashes_compact(binding);
    raw("}, \"retained_reference_event_id\": ");
    json_event_id_option(binding.retained_reference_event_id);
    raw(", \"retained_manifest_reference_event_id\": ");
    json_event_id_option(binding.manifest_reference_event_id);
    raw(", \"retained_audit_rollback_reference_event_id\": ");
    json_event_id_option(binding.audit_rollback_reference_event_id);
    raw(", \"retained_service_slot_reservation_event_id\": ");
    json_event_id_option(binding.service_slot_reservation_event_id);
    raw(", \"local_approval\": {\"state\": \"missing\", \"required\": true}, \"ram_only_service_slot\": {\"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw(", \"required\": true, \"allocates_service_slot\": false}, \"load_attempted\": false, \"service_inventory_change\": \"none\", \"can_load\": false}");
}

fn emit_module_load_gate_required_hashes_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
    } else {
        raw("\"computed_capability_grant_hash\": null, \"local_attestation_hash\": null");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
    } else {
        raw(", \"vm_test_report_reference_hash\": null, \"vm_test_report_hash\": null");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
    } else {
        raw(", \"artifact_reference_hash\": null, \"artifact_hash\": null");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
    } else {
        raw(", \"manifest_reference_hash\": null, \"manifest_hash\": null");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw(", \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw(", \"audit_record_hash\": null, \"rollback_plan_hash\": null, \"local_approval_hash\": null, \"pre_load_service_inventory_hash\": null, \"cleanup_actions_hash\": null, \"ram_only_service_slot_id\": null");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw(", \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
    } else {
        raw(", \"service_slot_reservation_hash\": null");
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

fn emit_provider_object(provider: &provider::Snapshot, comma: bool) {
    raw_line("      \"provider\": {");
    raw("        \"selected\": ");
    json_str(provider.provider_name);
    raw_line(",");
    raw("        \"route\": ");
    json_str(provider.route.as_str());
    raw_line(",");
    raw("        \"api_key_state\": ");
    json_str(if provider.api_key_set {
        "set"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"direct_phase\": ");
    json_str(provider.direct_phase);
    raw_line(",");
    raw("        \"direct_endpoint\": ");
    json_str(provider.direct_endpoint);
    raw_line(",");
    raw("        \"direct_model\": ");
    json_str(provider.direct_model);
    raw_line(",");
    raw("        \"trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"pin_kind\": ");
    json_opt_str(provider.trust_pin_kind);
    raw_line(",");
    raw("        \"pin_id\": ");
    json_opt_str(provider.trust_pin_id);
    raw_line(",");
    raw("        \"development_bypass\": ");
    raw_bool(provider.trust_development_bypass);
    crlf();
    raw("      }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_status_state(name: &str, state: RowState, comma: bool) {
    emit_status_state_at(name, state, comma, 8);
}

fn emit_status_state_at(name: &str, state: RowState, comma: bool, spaces: usize) {
    indent(spaces);
    json_str(name);
    raw(": ");
    json_str(state.as_protocol());
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_status_detail(name: &str, line: &system_status::StatusLine, comma: bool) {
    indent(8);
    json_str(name);
    raw(": {\"state\": ");
    json_str(line.state.as_protocol());
    raw(", \"detail\": ");
    json_str(line.detail.as_str());
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_device(id: &str, kind: &str, line: &system_status::StatusLine, comma: bool) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"kind\": ");
    json_str(kind);
    raw(", \"state\": ");
    json_str(line.state.as_protocol());
    raw(", \"detail\": ");
    json_str(line.detail.as_str());
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_problem_objects(status: &SystemSnapshot, provider: &provider::Snapshot, spaces: usize) {
    let mut wrote = false;
    emit_provider_trust_problem(&mut wrote, spaces, provider);
    if !provider.api_key_set {
        emit_problem(
            &mut wrote,
            spaces,
            "provider.openai.api_key_missing",
            "info",
            "OpenAI direct requests need a RAM-only API key",
        );
    }
    emit_status_problem(
        &mut wrote,
        spaces,
        "framebuffer.unavailable",
        "error",
        "Limine framebuffer is unavailable",
        &status.framebuffer,
    );
    emit_status_problem(
        &mut wrote,
        spaces,
        "entropy.not_ready",
        "warning",
        "Entropy is not ready yet",
        &status.entropy,
    );
    emit_status_problem(
        &mut wrote,
        spaces,
        "usb_xhci.unavailable",
        "warning",
        "xHCI USB path is missing or degraded",
        &status.usb_xhci,
    );
    emit_status_problem(
        &mut wrote,
        spaces,
        "network.unavailable",
        "warning",
        "e1000/IPv4 network path is not configured",
        &status.network,
    );
    emit_status_problem(
        &mut wrote,
        spaces,
        "input.unavailable",
        "warning",
        "keyboard or pointer input is missing",
        &status.input,
    );
    match wifi::snapshot().state {
        wifi::WifiState::Detected => emit_problem(
            &mut wrote,
            spaces,
            "wifi.avastar.firmware_todo",
            "info",
            "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        ),
        wifi::WifiState::Missing => emit_problem(
            &mut wrote,
            spaces,
            "wifi.avastar.target_absent",
            "info",
            "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        ),
        wifi::WifiState::NotProbed => {}
    }
    if !wrote {
        indent(spaces);
        raw("{\"id\": \"none\", \"severity\": \"info\", \"summary\": \"no known protocol problems reported\"}");
        crlf();
    } else {
        crlf();
    }
}

fn emit_provider_trust_problem(wrote: &mut bool, spaces: usize, provider: &provider::Snapshot) {
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

    emit_problem(wrote, spaces, id, "high", summary);
}

fn emit_status_problem(
    wrote: &mut bool,
    spaces: usize,
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
    emit_problem(wrote, spaces, id, severity, summary);
}

fn emit_problem(
    wrote: &mut bool,
    spaces: usize,
    id: &'static str,
    severity: &'static str,
    summary: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    indent(spaces);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"severity\": ");
    json_str(severity);
    raw(", \"summary\": ");
    json_str(summary);
    raw("}");
    *wrote = true;
}

fn emit_service_ids(spaces: usize) {
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        indent(spaces);
        json_str(service_inventory::SERVICES[idx].id);
        if idx + 1 != service_inventory::SERVICES.len() {
            raw(",");
        }
        crlf();
        idx += 1;
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

fn emit_capability_ids(spaces: usize) {
    let mut idx = 0usize;
    while idx < CAPABILITIES.len() {
        if CAPABILITIES[idx].granted {
            indent(spaces);
            json_str(CAPABILITIES[idx].id);
            raw(",");
            crlf();
        }
        idx += 1;
    }
    indent(spaces);
    json_str("capability_denied.for_all_mutating_methods");
    crlf();
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

fn module_manifest_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.manifest_diagnostic")
}

fn module_manifest_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.manifest_diagnostic_selftest")
}

fn module_artifact_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.artifact_diagnostic")
}

fn module_artifact_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.artifact_diagnostic_selftest")
}

fn module_vm_report_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.vm_report_diagnostic")
}

fn module_vm_report_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.vm_report_diagnostic_selftest")
}

fn module_load_gate_manifest_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_manifest_selftest")
        || method_head_eq(method, "module.manifest_gate_selftest")
}

fn module_load_gate_artifact_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_artifact_selftest")
        || method_head_eq(method, "module.artifact_gate_selftest")
}

fn module_load_gate_vm_report_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_vm_report_selftest")
        || method_head_eq(method, "module.vm_report_gate_selftest")
}

fn module_load_gate_retained_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_retained_selftest")
        || method_head_eq(method, "module.retained_grant_gate_selftest")
}

fn module_load_gate_audit_rollback_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_audit_rollback_selftest")
        || method_head_eq(method, "module.audit_rollback_gate_selftest")
}

fn module_load_gate_service_slot_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_service_slot_selftest")
        || method_head_eq(method, "module.service_slot_gate_selftest")
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

fn module_manifest_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.manifest_diagnostic") {
        "module.manifest_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn module_artifact_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.artifact_diagnostic") {
        "module.artifact_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn module_vm_report_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.vm_report_diagnostic") {
        "module.vm_report_diagnostic".len()
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
