use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_module_reference::emit_evidence_v1_response,
    agent_protocol_support::{record_bool as b, record_field as f, record_str as s},
    ahci, echo_service, granted_candidate_service, hello_service, pci, personal_shell_service,
    project_app_autoload, provider, serial, service_inventory, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, workspace_candidate_service,
};
use raios_core::{
    evidence_response,
    record::Value as V,
    system_status_projection::{observed_projection, Classification, EvidenceInput, Projection},
};
pub(crate) struct Capability {
    pub(crate) id: &'static str,
    pub(crate) risk: &'static str,
    pub(crate) granted: bool,
    pub(crate) scope: &'static str,
    pub(crate) summary: &'static str,
}

pub(crate) const CAPABILITIES: &[Capability] = &[
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
        id: "cap.system.time_authority.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read local CMOS RTC wall-clock evidence without granting time authority",
    },
    Capability {
        id: "cap.system.cert_time_check_selftest.read",
        risk: "observe",
        granted: true,
        scope: "current_boot_test_infrastructure",
        summary: "read unverified-basis certificate validity window sanity selftest",
    },
    Capability {
        id: "cap.system.honesty_report.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read unified local honesty posture report",
    },
    Capability {
        id: "cap.service.inventory.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read static service inventory",
    },
    Capability {
        id: "cap.service.health.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot service health",
    },
    Capability {
        id: "cap.service.rollback_preview.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot rollback preview evidence",
    },
    Capability {
        id: "cap.recovery.rollback_inspect.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot rollback recovery inspection evidence",
    },
    Capability {
        id: "cap.recovery.rollback_materialize_dry_run.current_boot",
        risk: "test_media_write",
        granted: true,
        scope: "current_boot_test_infrastructure",
        summary: "materialize current-boot rollback test-sector evidence",
    },
    Capability {
        id: "cap.service.descriptor_source_trust.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot descriptor-source trust selftests",
    },
    Capability {
        id: "cap.service.artifact_reference_trust.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot artifact-reference trust selftests",
    },
    Capability {
        id: "cap.service.artifact_load_plan_preflight.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot artifact load-plan preflight selftests",
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
        id: "cap.persist.layout.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read current-boot GPT and SEED_DATA layout evidence",
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
        id: "cap.recovery.load_artifact.read",
        risk: "observe",
        granted: true,
        scope: "current_boot",
        summary: "read recovery artifact load binding diagnostics",
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
        id: "cap.service.hello_demo.current_boot",
        risk: "modify_ram",
        granted: true,
        scope: "current_boot_builtin_demo_service_only",
        summary: "load, stop, and drop the built-in hello demo service in RAM",
    },
    Capability {
        id: "cap.service.rollback_apply.current_boot",
        risk: "modify_ram",
        granted: false,
        scope: "denied_until_rollback_apply_authority_durable_audit_and_transaction_evidence",
        summary: "apply a retained current-boot service rollback",
    },
    Capability {
        id: "cap.recovery.load_artifact",
        risk: "recovery_modify_ram",
        granted: false,
        scope: "denied_until_recovery_identity_trust_vm_test_approval_loader_and_rollback_evidence",
        summary: "load a recovery-only artifact through the recovery lifeline",
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

pub(crate) const READ_METHODS: &[&str] = &[
    "system.describe",
    "system.snapshot",
    "system.capabilities",
    "system.boot_log",
    "system.time_authority",
    "system.cert_time_check_selftest",
    "system.honesty_report",
    "device.graph",
    "persist.layout",
    "problem.list",
    "service.inventory",
    "service.health",
    "service.rollback_preview",
    "service.descriptor_source_trust_selftest",
    "service.artifact_reference_trust_selftest",
    "service.artifact_load_plan_preflight_selftest",
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
    "module.attestation_diagnostic",
    "module.attestation_diagnostic_selftest",
    "module.approval_diagnostic",
    "module.approval_diagnostic_selftest",
    "module.grant_diagnostic",
    "module.grant_diagnostic_selftest",
    "module.audit_rollback_diagnostic",
    "module.audit_rollback_diagnostic_selftest",
    "module.service_slot_diagnostic",
    "module.service_slot_diagnostic_selftest",
    "module.service_slot_allocator",
    "module.service_slot_allocator_selftest",
    "module.loader_runtime",
    "module.loader_runtime_selftest",
    "module.loader_identity",
    "module.loader_identity_selftest",
    "module.loader_artifact_hash_binding",
    "module.loader_artifact_hash_binding_selftest",
    "module.loader_entrypoint_abi",
    "module.loader_entrypoint_abi_selftest",
    "module.loader_address_space_boundary",
    "module.loader_address_space_boundary_selftest",
    "module.loader_memory_map_constraints",
    "module.loader_memory_map_constraints_selftest",
    "module.loader_capability_import_table",
    "module.loader_capability_import_table_selftest",
    "module.loader_service_slot_binding",
    "module.loader_service_slot_binding_selftest",
    "module.loader_health_state_hooks",
    "module.loader_health_state_hooks_selftest",
    "module.loader_rollback_hooks",
    "module.loader_rollback_hooks_selftest",
    "module.loader_audit_rollback_write_boundary_binding",
    "module.loader_audit_rollback_write_boundary_binding_selftest",
    "module.audit_rollback_availability",
    "module.audit_rollback_availability_selftest",
    "module.audit_rollback_write_policy",
    "module.audit_rollback_write_policy_selftest",
    "module.audit_rollback_storage_layout",
    "module.audit_rollback_storage_layout_selftest",
    "module.audit_rollback_append_engine",
    "module.audit_rollback_append_engine_selftest",
    "module.audit_rollback_append_contract",
    "module.audit_rollback_append_contract_selftest",
    "module.audit_rollback_append_payload_hash",
    "module.audit_rollback_append_payload_hash_selftest",
    "module.audit_rollback_append_intent",
    "module.audit_rollback_append_intent_selftest",
    "module.audit_rollback_write_boundary",
    "module.audit_rollback_write_boundary_selftest",
    "module.load_gate_manifest_selftest",
    "module.load_gate_artifact_selftest",
    "module.load_gate_vm_report_selftest",
    "module.load_gate_attestation_selftest",
    "module.load_gate_approval_selftest",
    "module.load_gate_retained_selftest",
    "module.load_gate_audit_rollback_selftest",
    "module.load_gate_service_slot_selftest",
    "module.load_gate_loader_runtime_selftest",
    "recovery.identity_diagnostic",
    "recovery.identity_diagnostic_selftest",
    "recovery.trust_diagnostic",
    "recovery.trust_diagnostic_selftest",
    "recovery.vm_test_diagnostic",
    "recovery.vm_test_diagnostic_selftest",
    "recovery.local_approval_diagnostic",
    "recovery.local_approval_diagnostic_selftest",
    "recovery.loader_diagnostic",
    "recovery.loader_diagnostic_selftest",
    "recovery.rollback_evidence_diagnostic",
    "recovery.rollback_evidence_diagnostic_selftest",
    "recovery.rollback_inspect",
    "recovery.lifeline_request_diagnostic",
    "recovery.lifeline_request_diagnostic_selftest",
    "recovery.lifeline_protocol_diagnostic",
    "recovery.lifeline_protocol_diagnostic_selftest",
    "recovery.lifeline_command_vocabulary",
    "recovery.lifeline_command_vocabulary_selftest",
    "recovery.loader_runtime_isolation",
    "recovery.loader_runtime_isolation_selftest",
    "recovery.rollback_transaction_engine",
    "recovery.rollback_transaction_engine_selftest",
    "recovery.durable_audit_rollback_persistence",
    "recovery.durable_audit_rollback_persistence_selftest",
    "recovery.memory_provenance",
    "recovery.memory_provenance_selftest",
    "recovery.lifeline_command_admission",
    "recovery.lifeline_command_admission_selftest",
    "recovery.lifeline_command_envelope_diagnostic",
    "recovery.lifeline_command_envelope_diagnostic_selftest",
    "recovery.lifeline_command_dispatch_diagnostic",
    "recovery.lifeline_command_dispatch_diagnostic_selftest",
    "recovery.lifeline_command_body_canonicalization_diagnostic",
    "recovery.lifeline_command_body_canonicalization_diagnostic_selftest",
    "recovery.lifeline_command_handler_binding_diagnostic",
    "recovery.lifeline_command_handler_binding_diagnostic_selftest",
    "recovery.lifeline_status_read_handler_diagnostic",
    "recovery.lifeline_status_read_handler_diagnostic_selftest",
    "recovery.rollback_preview_authorization_diagnostic",
    "recovery.rollback_preview_authorization_diagnostic_selftest",
    "recovery.rollback_apply_authorization_diagnostic",
    "recovery.rollback_apply_authorization_diagnostic_selftest",
    "recovery.disable_module_target_binding_diagnostic",
    "recovery.disable_module_target_binding_diagnostic_selftest",
    "recovery.restart_last_good_target_binding_diagnostic",
    "recovery.restart_last_good_target_binding_diagnostic_selftest",
    "recovery.load_artifact_by_hash_target_binding_diagnostic",
    "recovery.load_artifact_by_hash_target_binding_diagnostic_selftest",
    "recovery.memory_write_authority_diagnostic",
    "recovery.memory_write_authority_diagnostic_selftest",
    "recovery.durable_audit_rollback_write_authority_diagnostic",
    "recovery.durable_audit_rollback_write_authority_diagnostic_selftest",
    "recovery.service_inventory_side_effect_boundary_diagnostic",
    "recovery.service_inventory_side_effect_boundary_diagnostic_selftest",
    "recovery.lifeline_command_dispatch_behavior_diagnostic",
    "recovery.lifeline_command_dispatch_behavior_diagnostic_selftest",
    "recovery.lifeline_command_executor_capability_table_diagnostic",
    "recovery.lifeline_command_executor_capability_table_diagnostic_selftest",
    "recovery.lifeline_command_side_effect_gate_diagnostic",
    "recovery.lifeline_command_side_effect_gate_diagnostic_selftest",
    "recovery.lifeline_command_execution_enablement_diagnostic",
    "recovery.lifeline_command_execution_enablement_diagnostic_selftest",
    "recovery.lifeline_command_execution_preflight_diagnostic",
    "recovery.lifeline_command_execution_preflight_diagnostic_selftest",
    "recovery.lifeline_command_execution_intent_diagnostic",
    "recovery.lifeline_command_execution_intent_diagnostic_selftest",
    "recovery.lifeline_command_execution_commit_gate_diagnostic",
    "recovery.lifeline_command_execution_commit_gate_diagnostic_selftest",
    "recovery.lifeline_command_execution_result_denial_diagnostic",
    "recovery.lifeline_command_execution_result_denial_diagnostic_selftest",
    "recovery.lifeline_command_execution_audit_denial_diagnostic",
    "recovery.lifeline_command_execution_audit_denial_diagnostic_selftest",
    "recovery.lifeline_command_execution_observation_denial_diagnostic",
    "recovery.lifeline_command_execution_observation_denial_diagnostic_selftest",
    "recovery.lifeline_command_execution_completion_denial_diagnostic",
    "recovery.lifeline_command_execution_completion_denial_diagnostic_selftest",
    "recovery.lifeline_status_execution_result_diagnostic",
    "recovery.lifeline_status_result_read",
    "recovery.lifeline.status",
    "recovery.load_binding",
    "recovery.load_binding_selftest",
];

/// Typed numeric Context fact for the bounded personal-shell packet. It exposes
/// no capability detail and grants nothing; the full capability record remains
/// available only through the existing protocol methods.
pub(crate) fn denied_capability_count() -> u16 {
    CAPABILITIES
        .iter()
        .filter(|capability| !capability.granted)
        .count()
        .min(u16::MAX as usize) as u16
}

pub(crate) const DENIED_METHODS: &[&str] = &[
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
    "module.load_recovery_artifact",
    "recovery.load_artifact",
    "module.persist",
    "module.rollback",
    "service.load_ephemeral",
    "service.restart",
    "service.hot_swap",
    "service.rollback_apply",
    "service.start",
    "service.stop",
    "service.drop",
    "config.apply",
    "apply_config",
    "provider.configure",
    "wifi.configure",
    "draw_text",
    "probe_device",
    "download_signed_module",
    "run_module_test",
];

fn emit_system_observation<'a>(
    method: &'static str,
    family: &'static str,
    facts: V<'a>,
    evidence: Vec<EvidenceInput<'a>>,
    reason: &'static str,
) {
    let Projection {
        facts,
        evidence,
        decision,
        ..
    } = observed_projection(facts, evidence, reason);
    emit_evidence_v1_response(
        method,
        family,
        None,
        facts,
        evidence
            .into_iter()
            .map(evidence_response::evidence_value)
            .collect(),
        decision,
    );
}

fn strings(values: &[&'static str]) -> V<'static> {
    V::Array(values.iter().map(|value| s(value)).collect())
}

fn capability_ids() -> V<'static> {
    V::Array(CAPABILITIES.iter().map(|cap| s(cap.id)).collect())
}

fn status_states(status: &SystemSnapshot) -> V<'_> {
    V::InlineObject(vec![
        f("framebuffer", s(status.framebuffer.state.as_protocol())),
        f("entropy", s(status.entropy.state.as_protocol())),
        f("usb_xhci", s(status.usb_xhci.state.as_protocol())),
        f("wifi", s(status.wifi.state.as_protocol())),
        f("network", s(status.network.state.as_protocol())),
        f("input", s(status.input.state.as_protocol())),
    ])
}

fn status_details(status: &SystemSnapshot) -> V<'_> {
    fn detail(line: &system_status::StatusLine) -> V<'_> {
        V::InlineObject(vec![
            f("state", s(line.state.as_protocol())),
            f("detail", s(line.detail.as_str())),
        ])
    }
    V::InlineObject(vec![
        f("framebuffer", detail(&status.framebuffer)),
        f("entropy", detail(&status.entropy)),
        f("usb_xhci", detail(&status.usb_xhci)),
        f("wifi", detail(&status.wifi)),
        f("network", detail(&status.network)),
        f("input", detail(&status.input)),
    ])
}

fn provider_value(provider: &provider::Snapshot) -> V<'_> {
    V::InlineObject(vec![
        f("selected", s(provider.provider_name)),
        f("route", s(provider.route.as_str())),
        f(
            "api_key_state",
            s(if provider.api_key_set {
                "set"
            } else {
                "missing"
            }),
        ),
        f("direct_phase", s(provider.direct_phase)),
        f("direct_endpoint", s(provider.direct_endpoint)),
        f("direct_model", s(provider.direct_model)),
        f("trust_state", s(provider.trust_state)),
        f(
            "pin_kind",
            provider.trust_pin_kind.map(s).unwrap_or(V::Null),
        ),
        f("pin_id", provider.trust_pin_id.map(s).unwrap_or(V::Null)),
        f(
            "pin_slot",
            provider.trust_pin_slot.map(s).unwrap_or(V::Null),
        ),
        f("pin_rotation_policy", s(provider.trust_pin_rotation_policy)),
        f(
            "pin_rotation_id",
            provider.trust_pin_rotation_id.map(s).unwrap_or(V::Null),
        ),
        f("development_bypass", b(provider.trust_development_bypass)),
        f(
            "verifier_decision",
            V::InlineObject(vec![
                f("schema", s(provider.trust_verifier_decision.schema)),
                f(
                    "verifier_id",
                    s(provider.trust_verifier_decision.verifier_id),
                ),
                f("stage", s(provider.trust_verifier_decision.stage)),
                f("status", s(provider.trust_verifier_decision.outcome)),
                f("reason", s(provider.trust_verifier_decision.reason)),
            ]),
        ),
    ])
}

fn problem_values(status: &SystemSnapshot, provider: &provider::Snapshot) -> Vec<V<'static>> {
    // ponytail: duplicate stable labels until SystemSnapshot carries the captured Wi-Fi fact;
    // calling system_problem_facts::collect here would re-read mutable Wi-Fi state.
    fn problem(id: &'static str, severity: &'static str, summary: &'static str) -> V<'static> {
        V::InlineObject(vec![
            f("id", s(id)),
            f("severity", s(severity)),
            f("summary", s(summary)),
        ])
    }
    let mut values = Vec::new();
    let (trust_id, trust_summary) = match provider.trust_state {
        "tls_certificate_verification_bypassed" => ("provider.tls_unverified", "OpenAI direct transport is using an explicit unverified TLS development override"),
        "pin_config_missing" => ("provider.tls_pin_config_missing", "OpenAI direct transport is fail-closed until a provider pin is configured"),
        "pin_config_invalid" => ("provider.tls_pin_config_invalid", "Configured OpenAI provider pin is invalid"),
        "pin_verifier_unavailable" => ("provider.tls_pin_verifier_unavailable", "Configured OpenAI provider pin cannot be checked until TLS verifier input access exists"),
        "pin_mismatch" => ("provider.tls_pin_mismatch", "OpenAI provider certificate did not match the configured pin"),
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => ("", ""),
        _ => ("provider.tls_unknown", "OpenAI provider trust has not been established"),
    };
    if !trust_id.is_empty() {
        values.push(problem(trust_id, "high", trust_summary));
    }
    if !provider.api_key_set {
        values.push(problem(
            "provider.openai.api_key_missing",
            "info",
            "OpenAI direct requests need a RAM-only API key",
        ));
    }
    for (line, id, severity, summary) in [
        (
            &status.framebuffer,
            "framebuffer.unavailable",
            "error",
            "Limine framebuffer is unavailable",
        ),
        (
            &status.entropy,
            "entropy.not_ready",
            "warning",
            "Entropy is not ready yet",
        ),
        (
            &status.usb_xhci,
            "usb_xhci.unavailable",
            "warning",
            "xHCI USB path is missing or degraded",
        ),
        (
            &status.network,
            "network.unavailable",
            "warning",
            "e1000/IPv4 network path is not configured",
        ),
        (
            &status.input,
            "input.unavailable",
            "warning",
            "keyboard or pointer input is missing",
        ),
    ] {
        if !matches!(
            line.state,
            RowState::Ready | RowState::Configured | RowState::Detected
        ) {
            values.push(problem(id, severity, summary));
        }
    }
    match status.wifi.state {
        RowState::Detected => values.push(problem(
            "wifi.avastar.firmware_todo",
            "info",
            "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        )),
        RowState::Missing => values.push(problem(
            "wifi.avastar.target_absent",
            "info",
            "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        )),
        _ => {}
    }
    if values.is_empty() {
        vec![V::InlineObject(vec![
            f("id", s("none")),
            f("severity", s("info")),
            f("summary", s("no known protocol problems reported")),
        ])]
    } else {
        values
    }
}

pub(crate) fn emit_describe() {
    emit_system_observation(
        "system.describe",
        "system.describe",
        V::InlineObject(vec![
            f(
                "os",
                V::InlineObject(vec![
                    f("name", s("raiOS")),
                    f("product", s("raiOS")),
                    f("stage", s("stage-0")),
                ]),
            ),
            f(
                "protocol",
                V::InlineObject(vec![
                    f("version", s("raios.agent.v0")),
                    f("transport", s("serial-console")),
                    f(
                        "provider_context_injection",
                        s("disabled_until_final_injection_authorization"),
                    ),
                    f("mutation_policy", s("denied_by_default")),
                ]),
            ),
            f("methods", strings(READ_METHODS)),
            f("denied_methods", strings(DENIED_METHODS)),
        ]),
        vec![],
        "system_description_observed",
    );
}

pub(crate) fn emit_snapshot(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();

    let problems = problem_values(&status, &provider);
    emit_system_observation(
        "system.snapshot",
        "system.snapshot",
        V::InlineObject(vec![
            f(
                "os",
                V::InlineObject(vec![
                    f("name", s("raiOS")),
                    f("product", s("raiOS")),
                    f("stage", s("stage-0")),
                ]),
            ),
            f("status", status_states(&status)),
            f("details", status_details(&status)),
            f("provider", provider_value(&provider)),
            f("capabilities", capability_ids()),
            f("problems", V::Array(problems)),
        ]),
        vec![],
        "system_snapshot_observed",
    );
}

pub(crate) fn emit_capabilities() {
    let evidence = CAPABILITIES
        .iter()
        .map(|cap| EvidenceInput {
            id: cap.id,
            kind: "capability_catalog_entry",
            status: if cap.granted { "granted" } else { "denied" },
            reason: "capability_status_observed",
            source_event_sequence: None,
            classification: Classification::LocalOnly,
            facts: V::InlineObject(vec![
                f("risk", s(cap.risk)),
                f("scope", s(cap.scope)),
                f("summary", s(cap.summary)),
            ]),
        })
        .collect();
    emit_system_observation(
        "system.capabilities",
        "system.capabilities",
        V::InlineObject(vec![f("capability_ids", capability_ids())]),
        evidence,
        "capability_catalog_observed",
    );
}

pub(crate) fn emit_boot_log() {
    let log = serial::log_snapshot();
    let lines = log
        .lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !line.as_str().is_empty())
        .map(|(index, line)| {
            V::InlineObject(vec![
                f("index", V::U64(index as u64)),
                f("text", s(line.as_str())),
            ])
        })
        .collect();
    emit_system_observation(
        "system.boot_log",
        "system.boot_log",
        V::InlineObject(vec![
            f("source", s("serial_ring")),
            f("lines", V::Array(lines)),
        ]),
        vec![],
        "boot_log_observed",
    );
}

pub(crate) fn emit_persist_layout() {
    let evidence = match pci::find_mass_storage_controller() {
        Some(controller) => ahci::detect_persist_layout(controller),
        None => ahci::PersistLayoutEvidence::absent("ahci_controller_not_observed"),
    };
    let port = if evidence.port_index == u8::MAX {
        V::Null
    } else {
        V::U64(evidence.port_index as u64)
    };

    emit_system_observation(
        "persist.layout",
        "persist.layout",
        V::InlineObject(vec![
            f("status", s(evidence.status())),
            f("reason", s(evidence.reason)),
            f("source", s(evidence.source)),
            f("source_port_index", port),
            f("controller_present", b(evidence.controller_present)),
            f("read_attempted", b(evidence.read_attempted)),
            f("read_completed", b(evidence.read_completed)),
            f("read_only", b(true)),
            f("write_attempted", b(evidence.write_attempted)),
            f("write_dma_ext_called", b(evidence.write_dma_ext_called)),
            f("writes_enabled", b(evidence.writes_enabled)),
            f("persistence_claimed", b(evidence.persistence_claimed)),
            f(
                "gpt_layout",
                raios_core::gpt_layout::gpt_layout_record(&evidence.gpt),
            ),
            f(
                "data_layout",
                raios_core::seed_data_layout::data_layout_record(&evidence.data),
            ),
        ]),
        vec![],
        "persist_layout_observed",
    );
}

pub(crate) fn emit_device_graph(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    fn device<'a>(
        id: &'static str,
        kind: &'static str,
        line: &'a system_status::StatusLine,
    ) -> V<'a> {
        V::InlineObject(vec![
            f("id", s(id)),
            f("kind", s(kind)),
            f("state", s(line.state.as_protocol())),
            f("detail", s(line.detail.as_str())),
        ])
    }
    emit_system_observation(
        "device.graph",
        "device.graph",
        V::InlineObject(vec![f(
            "devices",
            V::Array(vec![
                device("framebuffer.limine", "framebuffer", &status.framebuffer),
                device("entropy.rdrand", "entropy_source", &status.entropy),
                device("usb.xhci", "bus_controller", &status.usb_xhci),
                device("wifi.avastar_88w8897", "pci_wifi_target", &status.wifi),
                device("net.e1000", "pci_nic", &status.network),
                device("input.console", "input", &status.input),
            ]),
        )]),
        vec![],
        "device_graph_observed",
    );
}

pub(crate) fn emit_problem_list(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let problems = problem_values(&status, &provider);
    emit_system_observation(
        "problem.list",
        "problem.list",
        V::InlineObject(vec![f("problems", V::Array(problems))]),
        vec![],
        "problem_list_observed",
    );
}

pub(crate) fn emit_service_inventory(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let hello = hello_service::loaded_snapshot();
    let echo = echo_service::loaded_snapshot();
    let granted = granted_candidate_service::loaded_snapshot();
    let personal = personal_shell_service::lifecycle_snapshot();
    let autoload = project_app_autoload::snapshot();
    let workspace = (workspace_candidate_service::visible_in_inventory() || autoload.installed)
        .then(workspace_candidate_service::snapshot);
    let mut services = Vec::new();
    for service in service_inventory::SERVICES {
        let health = service_inventory::service_health(service, &status, &provider);
        services.push(V::InlineObject(vec![
            f("id", s(service.id)),
            f("kind", s(service.kind)),
            f("health", s(health.state)),
            f("replaceable", b(service.replaceable)),
            f("core_owned", b(service.core_owned)),
            f("last_error", health.last_error.map(s).unwrap_or(V::Null)),
            f("capabilities", strings(service.capabilities)),
        ]));
    }
    if personal.running {
        services.push(V::InlineObject(vec![
            f("id", s("svc.user.shell")),
            f("kind", s("service")),
            f("health", s("healthy")),
            f("replaceable", b(true)),
            f("core_owned", b(false)),
            f("last_error", V::Null),
            f("scope", s("current_boot")),
            f("persistence", s("none")),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f("host_import_count", V::U64(6)),
            f("running", b(true)),
            f("last_lifecycle_reason", s(personal.last_reason)),
            f("capabilities", V::InlineArray(vec![])),
        ]));
    }
    if let Some(echo) = echo {
        let descriptor = echo_service::ECHO_SERVICE_DESCRIPTOR;
        services.push(V::InlineObject(vec![
            f("id", s(descriptor.service_id)),
            f("kind", s(descriptor.inventory_kind)),
            f("health", s(echo_service::health_state(echo))),
            f("replaceable", b(descriptor.inventory_replaceable)),
            f("core_owned", b(descriptor.inventory_core_owned)),
            f("last_error", V::Null),
            f("scope", s(descriptor.scope)),
            f("persistence", s(descriptor.persistence)),
            f("artifact_id", s(descriptor.artifact_id)),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f("host_import_count", V::U64(2)),
            f("running", b(echo.running)),
            f("generation", V::U64(echo.generation)),
            f("run_count", V::U64(echo.run_count)),
            f("last_action", s(echo.last_action)),
            f("capabilities", strings(echo_service::CAPABILITIES)),
        ]));
    }
    if let Some(granted) = granted {
        let descriptor = granted_candidate_service::GRANTED_CANDIDATE_SERVICE_DESCRIPTOR;
        services.push(V::InlineObject(vec![
            f("id", s(descriptor.service_id)),
            f("kind", s(descriptor.inventory_kind)),
            f(
                "health",
                s(granted_candidate_service::health_state(granted)),
            ),
            f("replaceable", b(descriptor.inventory_replaceable)),
            f("core_owned", b(descriptor.inventory_core_owned)),
            f("last_error", V::Null),
            f("scope", s(descriptor.scope)),
            f("persistence", s(descriptor.persistence)),
            f("classification", s(descriptor.classification)),
            f("trust_tier", s(granted.trust_tier)),
            f(
                "ram_only_service_slot_id",
                s(granted_candidate_service::ram_only_service_slot_id()),
            ),
            f(
                "service_slot_activation_active",
                b(granted_candidate_service::service_slot_activation_active(
                    granted,
                )),
            ),
            f("running", b(granted.running)),
            f("generation", V::U64(granted.generation)),
            f("last_run_outcome", s(granted.last_run_outcome)),
            f("last_action", s(granted.last_action)),
            f(
                "capabilities",
                strings(granted_candidate_service::CAPABILITIES),
            ),
        ]));
    }
    if let Some(workspace) = workspace {
        let binding = workspace.binding;
        services.push(V::InlineObject(vec![
            f("id", s(workspace_candidate_service::service_id())),
            f("kind", s("wasm_service")),
            f(
                "health",
                s(workspace_candidate_service::health_state(workspace)),
            ),
            f("replaceable", b(true)),
            f("core_owned", b(false)),
            f(
                "last_error",
                if workspace.phase == "crashed" {
                    s(workspace.last_reason)
                } else {
                    V::Null
                },
            ),
            f("scope", s("current_boot")),
            f(
                "persistence",
                s(if autoload.installed {
                    "durable_install"
                } else {
                    "none"
                }),
            ),
            f("running", b(workspace.phase == "running")),
            f("generation", V::U64(workspace.generation)),
            f("phase", s(workspace.phase)),
            f("granted_host_imports", V::InlineArray(vec![])),
            f("host_import_count", V::U64(0)),
            f(
                "candidate_sha256",
                binding
                    .map(|item| V::Sha256(item.candidate_sha256))
                    .unwrap_or(V::Null),
            ),
            f(
                "receipt_sha256",
                binding
                    .map(|item| V::Sha256(item.receipt_sha256))
                    .unwrap_or(V::Null),
            ),
            f("run_count", V::U64(workspace.run_count)),
            f(
                "last_return_value_i32",
                workspace
                    .last_return_value
                    .filter(|value| *value >= 0)
                    .map(|value| V::U64(value as u64))
                    .unwrap_or(V::Null),
            ),
            f("last_action", s(workspace.last_run_outcome)),
            f("capabilities", V::InlineArray(vec![])),
        ]));
    }
    if let Some(hello) = hello {
        let descriptor = hello_service::HELLO_SERVICE_DESCRIPTOR;
        services.push(V::InlineObject(vec![
            f("id", s(descriptor.service_id)),
            f("kind", s(descriptor.inventory_kind)),
            f(
                "health",
                s(if hello.running {
                    descriptor.inventory_health_running
                } else {
                    descriptor.inventory_health_stopped
                }),
            ),
            f("replaceable", b(descriptor.inventory_replaceable)),
            f("core_owned", b(descriptor.inventory_core_owned)),
            f("last_error", V::Null),
            f("scope", s(descriptor.scope)),
            f("persistence", s(descriptor.persistence)),
            f(
                "load_descriptor_source_hash",
                V::Sha256(hello_service::descriptor_source_hash(hello.load_descriptor)),
            ),
            f(
                "load_descriptor_source_signature_envelope",
                hello
                    .load_descriptor
                    .source_envelope
                    .map(|envelope| {
                        V::InlineObject(vec![f("envelope_hash", V::Sha256(envelope.envelope_hash))])
                    })
                    .unwrap_or(V::Null),
            ),
            f(
                "artifact_load_plan_preflight_hash",
                V::Sha256(hello_service::artifact_load_plan_preflight_hash(
                    hello.load_descriptor,
                )),
            ),
            f(
                "service_slot_activation_hash",
                V::Sha256(hello_service::service_slot_activation_hash(
                    hello.load_descriptor,
                )),
            ),
            f("running", b(hello.running)),
            f("generation", V::U64(hello.generation)),
            f("last_action", s(hello.last_action)),
            f("capabilities", strings(hello_service::CAPABILITIES)),
        ]));
    }
    emit_system_observation(
        "service.inventory",
        "service.inventory",
        V::InlineObject(vec![f("services", V::Array(services))]),
        vec![],
        "service_inventory_observed",
    );
}
