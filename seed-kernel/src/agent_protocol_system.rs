use crate::{
    agent_protocol_support::{
        begin_response, crlf, emit_inline_string_array, emit_static_string_array, end_response,
        indent, json_opt_str, json_str, raw, raw_bool, raw_fmt, raw_line,
    },
    provider, serial, service_inventory, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, wifi,
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
    "recovery.load_binding",
    "recovery.load_binding_selftest",
];

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

pub(crate) fn emit_describe() {
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

pub(crate) fn emit_snapshot(runtime: ui::RuntimeStatus) {
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

pub(crate) fn emit_capabilities() {
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

pub(crate) fn emit_boot_log() {
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

pub(crate) fn emit_device_graph(runtime: ui::RuntimeStatus) {
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

pub(crate) fn emit_problem_list(runtime: ui::RuntimeStatus) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    begin_response("problem.list");
    raw_line("      \"schema\": \"problem.list.v0\",");
    raw_line("      \"problems\": [");
    emit_problem_objects(&status, &provider, 8);
    raw_line("      ]");
    end_response("problem.list");
}

pub(crate) fn emit_service_inventory(runtime: ui::RuntimeStatus) {
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

pub(crate) fn emit_provider_object(provider: &provider::Snapshot, comma: bool) {
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

pub(crate) fn emit_status_state(name: &str, state: RowState, comma: bool) {
    emit_status_state_at(name, state, comma, 8);
}

pub(crate) fn emit_status_state_at(name: &str, state: RowState, comma: bool, spaces: usize) {
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

pub(crate) fn emit_problem_objects(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    spaces: usize,
) {
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

pub(crate) fn emit_service_ids(spaces: usize) {
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

pub(crate) fn emit_capability_ids(spaces: usize) {
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
