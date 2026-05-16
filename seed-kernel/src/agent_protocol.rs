use core::fmt;

use crate::{
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

const READ_METHODS: &[&str] = &[
    "system.describe",
    "system.snapshot",
    "system.capabilities",
    "system.boot_log",
    "device.graph",
    "problem.list",
    "service.inventory",
];

const DENIED_METHODS: &[&str] = &[
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

pub fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> DispatchOutcome {
    let method = method.trim();
    if method.is_empty() {
        return DispatchOutcome::Unknown;
    }

    if method_eq(method, "system.describe") || method_eq(method, "describe") {
        emit_describe();
        return DispatchOutcome::Response("system.describe");
    }
    if method_eq(method, "system.snapshot") || method_eq(method, "snapshot") {
        emit_snapshot(runtime);
        return DispatchOutcome::Response("system.snapshot");
    }
    if method_eq(method, "system.capabilities")
        || method_eq(method, "capabilities")
        || method_eq(method, "caps")
    {
        emit_capabilities();
        return DispatchOutcome::Response("system.capabilities");
    }
    if method_eq(method, "system.boot_log")
        || method_eq(method, "system.bootlog")
        || method_eq(method, "bootlog")
    {
        emit_boot_log();
        return DispatchOutcome::Response("system.boot_log");
    }
    if method_eq(method, "device.graph") || method_eq(method, "devicegraph") {
        emit_device_graph(runtime);
        return DispatchOutcome::Response("device.graph");
    }
    if method_eq(method, "problem.list") || method_eq(method, "problems") {
        emit_problem_list(runtime);
        return DispatchOutcome::Response("problem.list");
    }
    if method_eq(method, "service.inventory") || method_eq(method, "services") {
        emit_service_inventory(runtime);
        return DispatchOutcome::Response("service.inventory");
    }

    if denied_method(method) {
        emit_capability_denied(canonical_denied_method(method));
        return DispatchOutcome::Denied(canonical_denied_method(method));
    }

    DispatchOutcome::Unknown
}

fn emit_describe() {
    begin_response("system.describe");
    raw_line("      \"schema\": \"system.describe.v0\",");
    raw_line(
        "      \"os\": {\"name\": \"SeedOS\", \"product\": \"RaiOS2\", \"stage\": \"stage-0\"},",
    );
    raw_line("      \"protocol\": {");
    raw_line("        \"version\": \"seedos.agent.v0\",");
    raw_line("        \"transport\": \"serial-console\",");
    raw_line("        \"provider_context_injection\": \"disabled_until_tls_trust_gate\",");
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
        "      \"os\": {\"name\": \"SeedOS\", \"product\": \"RaiOS2\", \"stage\": \"stage-0\"},",
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

fn emit_capability_denied(method: &'static str) {
    serial::write_raw_fmt(format_args!("SEEDOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"seedos.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"message\": ");
    json_str("mutating agent methods are denied until manifest, VM test report, local attestation, policy grant, approval, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"seedos.module_manifest.v0\",");
    raw_line("      \"seedos.vm_test_report.v0\",");
    raw_line("      \"local_attestation.v0\",");
    raw_line("      \"computed_capability_grant\",");
    raw_line("      \"local_approval\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ]");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("SEEDOS_AGENT_END {}\r\n", method));
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
    raw_line("        \"trust_state\": \"tls_certificate_verification_bypassed\"");
    raw("      }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_status_state(name: &str, state: RowState, comma: bool) {
    indent(8);
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
    emit_problem(
        &mut wrote,
        spaces,
        "provider.tls_unverified",
        "high",
        "OpenAI direct transport currently bypasses certificate verification",
    );
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

fn emit_static_string_array(values: &[&str], spaces: usize) {
    let mut idx = 0usize;
    while idx < values.len() {
        indent(spaces);
        json_str(values[idx]);
        if idx + 1 != values.len() {
            raw(",");
        }
        crlf();
        idx += 1;
    }
}

fn emit_inline_string_array(values: &[&str]) {
    let mut idx = 0usize;
    while idx < values.len() {
        if idx != 0 {
            raw(", ");
        }
        json_str(values[idx]);
        idx += 1;
    }
}

fn begin_response(method: &'static str) {
    serial::write_raw_fmt(format_args!("SEEDOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"seedos.agent.v0\",");
    raw_line("  \"t\": \"response\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw_line("    \"result\": {");
}

fn end_response(method: &'static str) {
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("SEEDOS_AGENT_END {}\r\n", method));
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

fn method_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn raw(value: &str) {
    serial::write_raw_str(value);
}

fn raw_line(value: &str) {
    serial::write_raw_line(value);
}

fn raw_fmt(args: fmt::Arguments<'_>) {
    serial::write_raw_fmt(args);
}

fn raw_bool(value: bool) {
    raw(if value { "true" } else { "false" });
}

fn crlf() {
    raw("\r\n");
}

fn indent(spaces: usize) {
    let mut idx = 0usize;
    while idx < spaces {
        raw(" ");
        idx += 1;
    }
}

fn json_str(value: &str) {
    raw("\"");
    for byte in value.bytes() {
        match byte {
            b'"' => raw("\\\""),
            b'\\' => raw("\\\\"),
            b'\n' => raw("\\n"),
            b'\r' => raw("\\r"),
            b'\t' => raw("\\t"),
            0x20..=0x7e => serial::write_byte(byte),
            _ => raw(" "),
        }
    }
    raw("\"");
}
