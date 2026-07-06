use alloc::vec;

use raios_core::record::Value as V;
use spin::Mutex;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, raw_line,
        record_bool as b, record_event_or_null, record_field as f, record_sha,
        record_static_str_array, record_str as s, record_str_or_null,
    },
    current_boot_service::{self, ServiceDescriptor, ServiceState},
    event_log, wasm_runtime,
};

include!(concat!(
    env!("OUT_DIR"),
    "/echo_current_boot_load_descriptor.rs"
));

pub(crate) const ECHO_SERVICE_DESCRIPTOR: ServiceDescriptor = ServiceDescriptor {
    service_id: "svc.demo.echo",
    artifact_id: "wasm:svc.demo.echo",
    artifact_kind: "wasm32_unknown_unknown_service_module",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    service_capability: "cap.service.echo_demo.current_boot",
    health_capability: "cap.service.health.read",
    rollback_preview_capability: "cap.service.echo.rollback_preview.read",
    rollback_apply_capability: "cap.service.echo.rollback_apply.current_boot",
    rollback_materialize_capability: "cap.recovery.echo.rollback_materialize_dry_run.current_boot",
    rollback_inspect_capability: "cap.recovery.echo.rollback_inspect.read",
    primary_alias: "echo",
    host_bound_alias: "host_bound:svc.demo.echo",
    replacement_service_id: "svc.demo.echo.v2",
    replacement_alias: "echo.v2",
    replacement_artifact_identity_id: "builtin_artifact_identity.svc.demo.echo.wasm.v2",
    reset_state_service_id: "svc.demo.echo.reset_state",
    reset_state_alias: "echo.reset_state",
    artifact_load_plan_preflight_id: "artifact_load_plan_preflight.current_boot.svc.demo.echo.v0",
    artifact_load_plan_preflight_status: "accepted_wasm_current_boot_only",
    service_slot_intent_id: "service_slot_intent.current_boot.svc.demo.echo.v0",
    ram_only_service_slot_id: "ram_only:svc.demo.echo",
    service_slot_activation_id: "service_slot_activation.current_boot.svc.demo.echo.v0",
    service_slot_activation_active_status: "active_current_boot",
    service_slot_activation_stopped_status: "stopped_current_boot",
    service_slot_activation_cleared_status: "cleared_current_boot",
    service_slot_activation_missing_status: "missing_current_boot",
    inventory_kind: "service",
    inventory_replaceable: true,
    inventory_core_owned: false,
    inventory_health_running: "healthy",
    inventory_health_stopped: "stopped",
    inventory_health_missing: "missing",
    event_lifecycle_kind: "raios.ram_only_echo_service.lifecycle",
    event_health_kind: "raios.ram_only_echo_service.health",
    event_rollback_preview_kind: "raios.ram_only_echo_service.rollback_preview",
    event_rollback_apply_kind: "raios.ram_only_echo_service.rollback_apply",
};

const ECHO_LOAD_DESCRIPTOR_SCHEMA: &str = "raios.current_boot_load_descriptor.v0";
const ECHO_LOAD_DESCRIPTOR_ID: &str = "load_descriptor.current_boot.svc.demo.echo.v0";
const ECHO_LOAD_DESCRIPTOR_SOURCE_LOCATOR: &str =
    "current_boot.service_load_descriptor.svc.demo.echo.v0";
const ECHO_LOAD_DESCRIPTOR_SOURCE_KIND: &str = "current_boot_wasm_service_load_descriptor_source";
const ECHO_LIFECYCLE_RESPONSE_SCHEMA: &str = "raios.ram_only_echo_service.lifecycle_response.v0";
const ECHO_HEALTH_RESPONSE_SCHEMA: &str = "raios.ram_only_echo_service.health_response.v0";
const ECHO_SERVICE_SLOT_ACTIVATION_SCHEMA: &str = "raios.ram_only_service_slot_activation.v0";
const ECHO_SERVICE_VERSION: &str = "v0";
const ECHO_ENTRYPOINT: &str = "raios_service_main";
pub(crate) const CAPABILITIES: &[&str] = &[
    "cap.service.echo_demo.current_boot",
    "cap.service.health.read",
];
const ECHO_SERVICE_LIFECYCLE_EVIDENCE: &[&str] = &[
    "current_boot.service_load_descriptor.svc.demo.echo.v0",
    "seed-kernel/artifacts/svc.demo.echo.wasm",
    "wasmi_linker_import_surface",
];
const ECHO_SERVICE_HEALTH_EVIDENCE: &[&str] = &[
    "current_boot.service_load_descriptor.svc.demo.echo.v0",
    "ram_only:svc.demo.echo",
];

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) loaded: bool,
    pub(crate) running: bool,
    pub(crate) generation: u64,
    pub(crate) run_count: u64,
    pub(crate) last_run_outcome: &'static str,
    pub(crate) last_return_value: Option<i32>,
    pub(crate) last_fuel_used: u64,
    pub(crate) last_log_line_emitted: bool,
    pub(crate) last_action: &'static str,
    pub(crate) last_reason: &'static str,
    pub(crate) last_inventory_change: &'static str,
    pub(crate) load_event_id: Option<event_log::EventId>,
    pub(crate) start_event_id: Option<event_log::EventId>,
    pub(crate) stop_event_id: Option<event_log::EventId>,
    pub(crate) drop_event_id: Option<event_log::EventId>,
}

#[derive(Clone, Copy)]
struct State {
    service: ServiceState,
    last_run_outcome: &'static str,
    last_return_value: Option<i32>,
    last_fuel_used: u64,
    last_log_line_emitted: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            service: ServiceState::new(),
            last_run_outcome: "not_run",
            last_return_value: None,
            last_fuel_used: 0,
            last_log_line_emitted: false,
        }
    }

    fn snapshot(self) -> Snapshot {
        let service = self.service;
        Snapshot {
            loaded: service.loaded,
            running: service.running,
            generation: service.generation,
            run_count: service.state_counter,
            last_run_outcome: self.last_run_outcome,
            last_return_value: self.last_return_value,
            last_fuel_used: self.last_fuel_used,
            last_log_line_emitted: self.last_log_line_emitted,
            last_action: service.last_action,
            last_reason: service.last_reason,
            last_inventory_change: service.last_inventory_change,
            load_event_id: service.load_event_id,
            start_event_id: service.start_event_id,
            stop_event_id: service.stop_event_id,
            drop_event_id: service.drop_event_id,
        }
    }
}

struct ActionResult {
    snapshot: Snapshot,
    event_id: event_log::EventId,
    run: Option<wasm_runtime::EchoRunEvidence>,
}

static STATE: Mutex<State> = Mutex::new(State::new());

#[used]
static ECHO_SERVICE_DESCRIPTOR_PROOF: fn() -> bool = validate_echo_service_descriptor;

pub(crate) fn validate_echo_service_descriptor() -> bool {
    let descriptor = ECHO_SERVICE_DESCRIPTOR;
    descriptor.service_id == ECHO_LOAD_DESCRIPTOR_SERVICE_ID
        && descriptor.artifact_id == ECHO_LOAD_DESCRIPTOR_ARTIFACT_ID
        && descriptor.artifact_kind == ECHO_LOAD_DESCRIPTOR_ARTIFACT_KIND
        && descriptor.scope == "current_boot"
        && descriptor.classification == "local_only"
        && descriptor.persistence == "none"
        && descriptor.service_capability == ECHO_LOAD_DESCRIPTOR_SERVICE_CAPABILITY
        && descriptor.ram_only_service_slot_id == ECHO_LOAD_DESCRIPTOR_SERVICE_SLOT_ID
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_ID
            == "builtin_artifact_identity.svc.demo.echo.wasm.v0"
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_HASH
            == [
                0xb3, 0xf7, 0xf8, 0x5a, 0xf2, 0x25, 0xb9, 0x1a, 0x31, 0x53, 0xc4, 0xc5, 0xdd, 0xd3,
                0x61, 0x4f, 0x36, 0x9d, 0x58, 0x30, 0x45, 0x8c, 0xaa, 0x91, 0xc4, 0x25, 0x31, 0xb1,
                0x99, 0x9f, 0x86, 0xad,
            ]
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_BYTES_HASH
            == [
                0xf8, 0x1f, 0x94, 0x42, 0xde, 0x37, 0x29, 0xf5, 0x8f, 0x9d, 0x5c, 0x43, 0xb1, 0x86,
                0xa4, 0x22, 0x3e, 0x3f, 0x0e, 0xd0, 0xbd, 0xde, 0x20, 0xe9, 0x47, 0x22, 0xda, 0x8d,
                0x57, 0x33, 0xab, 0xd2,
            ]
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORTS == "env.log,env.counter_get"
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORT_COUNT == 2
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZES_CURRENT_BOOT_WASM_EXECUTION
        && ECHO_LOAD_DESCRIPTOR_VALIDATES_WITH_WASMI_MODULE_NEW
        && !ECHO_LOAD_DESCRIPTOR_ACCEPTS_EXTERNAL_ARTIFACT_BYTES
        && !ECHO_LOAD_DESCRIPTOR_LOADS_EXTERNAL_ARTIFACT
        && !ECHO_LOAD_DESCRIPTOR_MAPS_EXECUTABLE_PAGES
        && !ECHO_LOAD_DESCRIPTOR_WRITES_PERSISTENT_STATE
        && !ECHO_LOAD_DESCRIPTOR_AUTHORIZES_PERSISTENT_INSTALL
        && !ECHO_LOAD_DESCRIPTOR_AUTHORIZES_ROLLBACK_INSTALL
        && raios_core::sha256_bytes(ECHO_LOAD_DESCRIPTOR_SOURCE.as_bytes())
            == ECHO_LOAD_DESCRIPTOR_HASH
        && raios_core::sha256_bytes(ECHO_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == ECHO_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH
}

pub(crate) fn loaded_snapshot() -> Option<Snapshot> {
    let state = STATE.lock();
    if state.service.loaded {
        Some(state.snapshot())
    } else {
        None
    }
}

pub(crate) fn is_load_method(method: &str) -> bool {
    load_source_method(method).is_some()
}

pub(crate) fn is_start_method(method: &str) -> bool {
    target_arg_matches(method, "service.start")
}

pub(crate) fn is_stop_method(method: &str) -> bool {
    target_arg_matches(method, "service.stop")
}

pub(crate) fn is_drop_method(method: &str) -> bool {
    target_arg_matches(method, "service.drop")
}

pub(crate) fn is_health_method(method: &str) -> bool {
    target_arg_matches(method, "service.health")
}

pub(crate) fn emit_load(method: &str) -> &'static str {
    let source_method = load_source_method(method).unwrap_or("module.load_ephemeral");
    let result = load(source_method);
    emit_response(source_method, "load", result);
    source_method
}

pub(crate) fn emit_start(_method: &str) -> &'static str {
    let result = start("service.start");
    emit_response("service.start", "start", result);
    "service.start"
}

pub(crate) fn emit_stop(_method: &str) -> &'static str {
    let result = stop("service.stop");
    emit_response("service.stop", "stop", result);
    "service.stop"
}

pub(crate) fn emit_drop(_method: &str) -> &'static str {
    let result = drop_service("service.drop");
    emit_response("service.drop", "drop", result);
    "service.drop"
}

pub(crate) fn emit_health(_method: &str) -> &'static str {
    let result = health("service.health");
    emit_response("service.health", "health", result);
    "service.health"
}

fn load_source_method(method: &str) -> Option<&'static str> {
    if target_arg_matches(method, "module.load_ephemeral") {
        Some("module.load_ephemeral")
    } else if target_arg_matches(method, "service.load_ephemeral") {
        Some("service.load_ephemeral")
    } else {
        None
    }
}

fn target_arg_matches(method: &str, head: &str) -> bool {
    current_boot_service::target_arg_matches(
        method,
        head,
        current_boot_service::LoadTarget {
            service_id: ECHO_SERVICE_DESCRIPTOR.service_id,
            artifact_id: ECHO_SERVICE_DESCRIPTOR.artifact_id,
            descriptor_id: ECHO_LOAD_DESCRIPTOR_ID,
        },
        ECHO_SERVICE_DESCRIPTOR,
    ) && validate_current_boot_load()
}

fn validate_current_boot_load() -> bool {
    validate_echo_service_descriptor() && wasm_runtime::validate_echo_wasm_artifact()
}

fn load(source_method: &'static str) -> ActionResult {
    let validation_ok = validate_current_boot_load();
    let mut state = STATE.lock();
    let reason = if !validation_ok {
        "load_descriptor_or_wasm_validation_failed"
    } else if state.service.loaded {
        "already_loaded"
    } else {
        "loaded_wasm_current_boot_service"
    };
    let inventory_change = if !validation_ok {
        "none"
    } else if state.service.loaded {
        "updated_current_boot_service"
    } else {
        "upserted_current_boot_service"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &ECHO_SERVICE_DESCRIPTOR,
        source_method,
        if validation_ok {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ECHO_SERVICE_LIFECYCLE_EVIDENCE,
    );

    if validation_ok {
        if !state.service.loaded {
            state.service.generation = state.service.generation.saturating_add(1);
            state.service.load_event_id = Some(event_id);
        }
        state.service.loaded = true;
        state.service.running = false;
    }
    state.service.last_action = "load";
    state.service.last_reason = reason;
    state.service.last_inventory_change = inventory_change;
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run: None,
    }
}

fn start(source_method: &'static str) -> ActionResult {
    let was_loaded = STATE.lock().service.loaded;
    let run = was_loaded.then(wasm_runtime::run_echo_service);
    let run_ok = run.as_ref().map(echo_run_succeeded).unwrap_or(false);
    let reason = if !was_loaded {
        "not_loaded"
    } else if run_ok {
        "wasm_run_success"
    } else {
        "wasm_run_failed"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &ECHO_SERVICE_DESCRIPTOR,
        source_method,
        if was_loaded {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ECHO_SERVICE_LIFECYCLE_EVIDENCE,
    );

    let mut state = STATE.lock();
    if was_loaded {
        state.service.running = run_ok;
        state.service.start_event_id = Some(event_id);
        if let Some(run) = run.as_ref() {
            if run_ok {
                state.service.state_counter = state.service.state_counter.saturating_add(1);
            }
            state.last_run_outcome = run.run_outcome;
            state.last_return_value = run.return_value;
            state.last_fuel_used = run.fuel_used;
            state.last_log_line_emitted = run.log_line.is_some();
        }
    }
    state.service.last_action = "start";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if was_loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run,
    }
}

fn stop(source_method: &'static str) -> ActionResult {
    let mut state = STATE.lock();
    let reason = if state.service.loaded && state.service.running {
        "stopped"
    } else if state.service.loaded {
        "already_stopped"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.service.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &ECHO_SERVICE_DESCRIPTOR,
        source_method,
        if state.service.loaded {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ECHO_SERVICE_LIFECYCLE_EVIDENCE,
    );

    if state.service.loaded {
        state.service.running = false;
        state.service.stop_event_id = Some(event_id);
    }
    state.service.last_action = "stop";
    state.service.last_reason = reason;
    state.service.last_inventory_change = inventory_change;
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run: None,
    }
}

fn drop_service(source_method: &'static str) -> ActionResult {
    let mut state = STATE.lock();
    let was_loaded = state.service.loaded;
    let reason = if was_loaded { "dropped" } else { "not_loaded" };
    let event_id = event_log::record_service_lifecycle_unbound(
        &ECHO_SERVICE_DESCRIPTOR,
        source_method,
        if was_loaded {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ECHO_SERVICE_LIFECYCLE_EVIDENCE,
    );

    state.service.loaded = false;
    state.service.running = false;
    state.service.state_counter = 0;
    state.service.drop_event_id = Some(event_id);
    state.service.last_action = "drop";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if was_loaded {
        "removed_current_boot_service"
    } else {
        "none"
    };
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run: None,
    }
}

fn health(source_method: &'static str) -> ActionResult {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    drop(state);
    let health = health_state(snapshot);
    let reason = if snapshot.running {
        "health_probe_healthy"
    } else if snapshot.loaded {
        "health_probe_stopped"
    } else {
        "health_probe_missing"
    };
    let event_id = event_log::record_service_health_unbound(
        &ECHO_SERVICE_DESCRIPTOR,
        source_method,
        health,
        reason,
        ECHO_SERVICE_HEALTH_EVIDENCE,
    );
    ActionResult {
        snapshot,
        event_id,
        run: None,
    }
}

pub(crate) fn health_state(snapshot: Snapshot) -> &'static str {
    current_boot_service::health_state(snapshot.loaded, snapshot.running, ECHO_SERVICE_DESCRIPTOR)
}

pub(crate) fn service_slot_activation_status(snapshot: Snapshot) -> &'static str {
    current_boot_service::service_slot_activation_status(
        snapshot.loaded,
        snapshot.running,
        snapshot.last_action,
        snapshot.last_reason,
        ECHO_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn service_slot_activation_active(snapshot: Snapshot) -> bool {
    current_boot_service::service_slot_activation_active(snapshot.loaded)
}

pub(crate) fn service_slot_activation_hash() -> [u8; 32] {
    raios_core::sha256_bytes(
        ECHO_SERVICE_DESCRIPTOR
            .service_slot_activation_id
            .as_bytes(),
    )
}

fn echo_run_succeeded(run: &wasm_runtime::EchoRunEvidence) -> bool {
    run.validation_ok
        && run.instantiation_ok
        && run.run_outcome == "success"
        && run.return_value == Some(0)
}

fn emit_response(method: &'static str, action: &'static str, result: ActionResult) {
    let snapshot = result.snapshot;
    begin_response(method);
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s(if action == "health" {
                    ECHO_HEALTH_RESPONSE_SCHEMA
                } else {
                    ECHO_LIFECYCLE_RESPONSE_SCHEMA
                }),
            ),
            f("scope", s(ECHO_SERVICE_DESCRIPTOR.scope)),
            f("classification", s(ECHO_SERVICE_DESCRIPTOR.classification)),
            f("method", s(method)),
            f("action", s(action)),
            f("service_id", s(ECHO_SERVICE_DESCRIPTOR.service_id)),
            f("artifact_id", s(ECHO_SERVICE_DESCRIPTOR.artifact_id)),
            f("artifact_kind", s(ECHO_SERVICE_DESCRIPTOR.artifact_kind)),
            f("version", s(ECHO_SERVICE_VERSION)),
            f("health", s(health_state(snapshot))),
            f("loaded", b(snapshot.loaded)),
            f("running", b(snapshot.running)),
            f("generation", V::U64(snapshot.generation)),
            f("run_count", V::U64(snapshot.run_count)),
            f("last_action", s(snapshot.last_action)),
            f("reason", s(snapshot.last_reason)),
            f(
                "service_inventory_change",
                s(snapshot.last_inventory_change),
            ),
            f("event_id", record_event_or_null(Some(result.event_id))),
            f(
                "load_event_id",
                record_event_or_null(snapshot.load_event_id),
            ),
            f(
                "start_event_id",
                record_event_or_null(snapshot.start_event_id),
            ),
            f(
                "stop_event_id",
                record_event_or_null(snapshot.stop_event_id),
            ),
            f(
                "drop_event_id",
                record_event_or_null(snapshot.drop_event_id),
            ),
            f("load_descriptor", record_load_descriptor()),
            f(
                "artifact_identity_id",
                s(ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_ID),
            ),
            f(
                "artifact_identity_hash",
                record_sha(ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_HASH),
            ),
            f(
                "artifact_sha256",
                record_sha(ECHO_LOAD_DESCRIPTOR_ARTIFACT_BYTES_HASH),
            ),
            f(
                "artifact_load_plan_preflight_id",
                s(ECHO_SERVICE_DESCRIPTOR.artifact_load_plan_preflight_id),
            ),
            f(
                "artifact_load_plan_preflight_hash",
                record_sha(ECHO_LOAD_DESCRIPTOR_HASH),
            ),
            f(
                "artifact_load_plan_preflight_status",
                s(ECHO_SERVICE_DESCRIPTOR.artifact_load_plan_preflight_status),
            ),
            f(
                "service_slot_activation",
                record_service_slot_activation(snapshot),
            ),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f(
                "granted_host_imports",
                record_static_str_array(&["env.log", "env.counter_get"]),
            ),
            f("host_import_count", V::U64(2)),
            f("entrypoint", s(ECHO_ENTRYPOINT)),
            f("run_evidence", record_run_evidence(result.run.as_ref())),
            f("last_run_evidence", record_last_run(snapshot)),
            f("accepts_external_artifact_bytes", b(false)),
            f("loads_external_artifact", b(false)),
            f("maps_executable_pages", b(false)),
            f("writes_persistent_state", b(false)),
            f("durable_writes_enabled", b(false)),
            f("rollback_apply_authorized", b(false)),
            f("broad_mutation_authorized", b(false)),
        ],
        6,
    );
    raw_line("      \"evidence_complete\": true");
    end_response(method);
}

fn record_load_descriptor() -> V<'static> {
    V::Object(vec![
        f("schema", s(ECHO_LOAD_DESCRIPTOR_SCHEMA)),
        f("id", s(ECHO_LOAD_DESCRIPTOR_ID)),
        f("source_locator", s(ECHO_LOAD_DESCRIPTOR_SOURCE_LOCATOR)),
        f("source_kind", s(ECHO_LOAD_DESCRIPTOR_SOURCE_KIND)),
        f("source_hash", record_sha(ECHO_LOAD_DESCRIPTOR_HASH)),
        f(
            "signature_envelope_hash",
            record_sha(ECHO_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH),
        ),
        f("source_validated", b(validate_echo_service_descriptor())),
        f("authorizes_current_boot_wasm_execution", b(true)),
        f("validates_with_wasmi_module_new", b(true)),
    ])
}

fn record_service_slot_activation(snapshot: Snapshot) -> V<'static> {
    V::Object(vec![
        f("schema", s(ECHO_SERVICE_SLOT_ACTIVATION_SCHEMA)),
        f("id", s(ECHO_SERVICE_DESCRIPTOR.service_slot_activation_id)),
        f("hash", record_sha(service_slot_activation_hash())),
        f("status", s(service_slot_activation_status(snapshot))),
        f("active", b(service_slot_activation_active(snapshot))),
        f(
            "ram_only_service_slot_id",
            s(ECHO_SERVICE_DESCRIPTOR.ram_only_service_slot_id),
        ),
    ])
}

fn record_run_evidence(run: Option<&wasm_runtime::EchoRunEvidence>) -> V<'_> {
    match run {
        Some(run) => V::Object(vec![
            f("present", b(true)),
            f("validation_ok", b(run.validation_ok)),
            f("instantiation_ok", b(run.instantiation_ok)),
            f("run_outcome", s(run.run_outcome)),
            f("return_value_i32", record_return_value(run.return_value)),
            f("fuel_budget", V::U64(run.fuel_budget)),
            f("fuel_used", V::U64(run.fuel_used)),
            f("log_prefix", s("WASM_GUEST_LOG")),
            f("log_line_emitted", b(run.log_line.is_some())),
            f("log_line", record_str_or_null(run.log_line.as_deref())),
        ]),
        None => V::Object(vec![
            f("present", b(false)),
            f("run_outcome", s("not_run")),
            f("return_value_i32", V::Null),
            f("fuel_budget", V::U64(wasm_runtime::ECHO_WASM_FUEL_BUDGET)),
            f("fuel_used", V::U64(0)),
            f("log_line", V::Null),
        ]),
    }
}

fn record_last_run(snapshot: Snapshot) -> V<'static> {
    V::Object(vec![
        f("run_outcome", s(snapshot.last_run_outcome)),
        f(
            "return_value_i32",
            record_return_value(snapshot.last_return_value),
        ),
        f("fuel_used", V::U64(snapshot.last_fuel_used)),
        f("log_line_emitted", b(snapshot.last_log_line_emitted)),
    ])
}

fn record_return_value(value: Option<i32>) -> V<'static> {
    match value {
        Some(value) if value >= 0 => V::U64(value as u64),
        _ => V::Null,
    }
}
