use core::fmt;

use crate::{
    event_log, provider, serial, service_inventory, system_status,
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
        id: "cap.memory.mutate",
        risk: "persist",
        granted: false,
        scope: "denied_until_event_log_audit_policy_persistence_and_rollback",
        summary: "write, supersede, compact, redact, or persist memory records",
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
    "memory.profile",
    "memory.context",
    "memory.query",
    "memory.trace",
    "memory.recent_events",
    "audit.events",
];

const DENIED_METHODS: &[&str] = &[
    "memory.record_observation",
    "memory.propose_policy",
    "memory.supersede_fact",
    "memory.redact",
    "memory.compact",
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
        record_read("memory.context");
        emit_memory_context(runtime, method);
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
    raw_line("        {\"id\": \"provider_minimal\", \"available\": false, \"target_tokens\": 2000, \"provider_export\": false, \"blocked_by\": \"needs positive provider trust plus provider_minimal redaction projection\"}");
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

fn emit_memory_context(runtime: ui::RuntimeStatus, method: &str) {
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
    raw_line("      \"source_schemas\": [");
    raw_line("        \"system.snapshot.v0\",");
    raw_line("        \"system.capabilities.v0\",");
    raw_line("        \"service.inventory.v0\",");
    raw_line("        \"problem.list.v0\",");
    raw_line("        \"system.boot_log.v0\"");
    raw_line("      ],");
    raw_line("      \"budget\": {\"target_tokens\": 4000, \"estimated_tokens\": 1200},");
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
    raw_line("        {\"kind\": \"provider_export\", \"reason\": \"disabled until positive provider trust and provider_minimal redaction projection exist\"},");
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
    raw("], \"created_at\": {\"clock\": \"sequence_only\", \"millis\": null}, \"persistence\": \"none\"}");
    if comma {
        raw(",");
    }
    crlf();
}

fn json_event_id(event_id: event_log::EventId) {
    json_event_sequence(event_id.sequence());
}

fn json_event_sequence(sequence: u64) {
    raw("\"event.current_boot.");
    raw_fmt(format_args!("{:08}", sequence));
    raw("\"");
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
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
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
    } else {
        "cap.system.describe.read"
    }
}

fn requested_capability_for_denial(method: &str) -> &'static str {
    if memory_mutation_method(method) {
        "cap.memory.mutate"
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
    if method_eq(method, "module.persist")
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

fn method_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn method_head_eq(left: &str, right: &str) -> bool {
    let left = left.trim();
    if left.len() < right.len() {
        return false;
    }
    let (head, rest) = left.split_at(right.len());
    method_eq(head, right) && (rest.is_empty() || rest.as_bytes()[0].is_ascii_whitespace())
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

fn provider_context_block_reason(trust_state: &str) -> &'static str {
    match trust_state {
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => {
            "provider_minimal_redaction_projection_missing"
        }
        _ => "provider_trust_not_positive",
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

fn json_opt_str(value: Option<&str>) {
    match value {
        Some(value) => json_str(value),
        None => raw("null"),
    }
}
