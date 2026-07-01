use spin::Mutex;

use crate::{
    agent_protocol_support::{
        begin_response, emit_inline_string_array, end_response, json_event_id_option, json_opt_str,
        json_sha256, json_sha256_option, json_str, method_head_eq, raw, raw_bool, raw_fmt,
        raw_line,
    },
    descriptor_sources, event_log,
};

pub(crate) const SERVICE_ID: &str = descriptor_sources::HELLO_SERVICE_ID;
pub(crate) const ARTIFACT_ID: &str = descriptor_sources::HELLO_ARTIFACT_ID;
pub(crate) const CAPABILITIES: &[&str] = &["cap.service.hello_demo.current_boot"];
pub(crate) const LOAD_DESCRIPTOR_SCHEMA: &str = descriptor_sources::HELLO_LOAD_DESCRIPTOR_SCHEMA;
pub(crate) const LOAD_DESCRIPTOR_ID: &str = descriptor_sources::HELLO_LOAD_DESCRIPTOR_ID;
pub(crate) const LOAD_DESCRIPTOR_CANONICALIZATION: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_CANONICALIZATION;
pub(crate) const LOAD_DESCRIPTOR_SOURCE_LOCATOR: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR;
pub(crate) const LOAD_DESCRIPTOR_SOURCE_KIND: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE_KIND;

#[derive(Clone, Copy)]
pub(crate) struct LoadDescriptor {
    pub schema: &'static str,
    pub id: &'static str,
    pub canonicalization: &'static str,
    pub source_locator: &'static str,
    pub source_kind: &'static str,
    pub binds_source_locator: Option<&'static str>,
    pub binds_source_kind: Option<&'static str>,
    pub binds_source_hash: Option<[u8; 32]>,
    pub source_text: &'static str,
    pub service_id: &'static str,
    pub artifact_id: &'static str,
    pub artifact_kind: &'static str,
    pub scope: &'static str,
    pub classification: &'static str,
    pub persistence: &'static str,
}

#[derive(Clone, Copy)]
struct LoadRequest {
    source_method: &'static str,
    descriptor: LoadDescriptor,
}

pub(crate) const LOAD_DESCRIPTOR: LoadDescriptor = LoadDescriptor {
    schema: LOAD_DESCRIPTOR_SCHEMA,
    id: LOAD_DESCRIPTOR_ID,
    canonicalization: LOAD_DESCRIPTOR_CANONICALIZATION,
    source_locator: LOAD_DESCRIPTOR_SOURCE_LOCATOR,
    source_kind: LOAD_DESCRIPTOR_SOURCE_KIND,
    binds_source_locator: None,
    binds_source_kind: None,
    binds_source_hash: None,
    source_text: descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE,
    service_id: SERVICE_ID,
    artifact_id: ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
};

pub(crate) fn load_descriptor_source_hash() -> [u8; 32] {
    descriptor_sources::hello_load_descriptor_source_hash()
}

pub(crate) fn descriptor_source_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    if let Some(hash) =
        descriptor_sources::descriptor_source_hash_for_locator(descriptor.source_locator)
    {
        hash
    } else {
        load_descriptor_source_hash()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub loaded: bool,
    pub running: bool,
    pub generation: u64,
    pub load_descriptor: LoadDescriptor,
    pub last_action: &'static str,
    pub last_reason: &'static str,
    pub last_inventory_change: &'static str,
    pub last_event_id: Option<event_log::EventId>,
    pub load_event_id: Option<event_log::EventId>,
    pub start_event_id: Option<event_log::EventId>,
    pub stop_event_id: Option<event_log::EventId>,
    pub drop_event_id: Option<event_log::EventId>,
}

#[derive(Clone, Copy)]
struct State {
    loaded: bool,
    running: bool,
    generation: u64,
    load_descriptor: LoadDescriptor,
    last_action: &'static str,
    last_reason: &'static str,
    last_inventory_change: &'static str,
    last_event_id: Option<event_log::EventId>,
    load_event_id: Option<event_log::EventId>,
    start_event_id: Option<event_log::EventId>,
    stop_event_id: Option<event_log::EventId>,
    drop_event_id: Option<event_log::EventId>,
}

impl State {
    const fn new() -> Self {
        Self {
            loaded: false,
            running: false,
            generation: 0,
            load_descriptor: LOAD_DESCRIPTOR,
            last_action: "none",
            last_reason: "not_loaded",
            last_inventory_change: "none",
            last_event_id: None,
            load_event_id: None,
            start_event_id: None,
            stop_event_id: None,
            drop_event_id: None,
        }
    }

    fn snapshot(self) -> Snapshot {
        Snapshot {
            loaded: self.loaded,
            running: self.running,
            generation: self.generation,
            load_descriptor: self.load_descriptor,
            last_action: self.last_action,
            last_reason: self.last_reason,
            last_inventory_change: self.last_inventory_change,
            last_event_id: self.last_event_id,
            load_event_id: self.load_event_id,
            start_event_id: self.start_event_id,
            stop_event_id: self.stop_event_id,
            drop_event_id: self.drop_event_id,
        }
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn loaded_snapshot() -> Option<Snapshot> {
    let state = STATE.lock();
    if state.loaded {
        Some(state.snapshot())
    } else {
        None
    }
}

pub(crate) fn is_load_start_method(method: &str) -> bool {
    load_request(method).is_some()
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

pub(crate) fn emit_load_start(method: &str) -> &'static str {
    let Some(request) = load_request(method) else {
        return "module.load_ephemeral";
    };
    let snapshot = load_start(request.source_method, request.descriptor);
    emit_response(
        request.source_method,
        "load_start",
        snapshot,
        request.descriptor,
    );
    request.source_method
}

pub(crate) fn emit_stop(_method: &str) -> &'static str {
    let snapshot = stop("service.stop");
    emit_response("service.stop", "stop", snapshot, snapshot.load_descriptor);
    "service.stop"
}

pub(crate) fn emit_drop(_method: &str) -> &'static str {
    let snapshot = drop_service("service.drop");
    emit_response("service.drop", "drop", snapshot, snapshot.load_descriptor);
    "service.drop"
}

pub(crate) fn emit_health(_method: &str) -> &'static str {
    let (snapshot, event_id) = health_probe("service.health");
    emit_health_response("service.health", snapshot, event_id);
    "service.health"
}

fn load_start(source_method: &'static str, descriptor: LoadDescriptor) -> Snapshot {
    let mut state = STATE.lock();
    let reason = if state.loaded && state.running {
        "already_running"
    } else if state.loaded {
        "started_loaded_service"
    } else {
        "loaded_and_started_builtin_service"
    };
    let inventory_change = if state.loaded {
        "updated_current_boot_service"
    } else {
        "upserted_current_boot_service"
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(descriptor, inventory_change),
    );

    if !state.loaded {
        state.generation = state.generation.saturating_add(1);
        state.load_event_id = Some(event_id);
    }
    state.loaded = true;
    state.running = true;
    state.load_descriptor = descriptor;
    state.start_event_id = Some(event_id);
    state.last_action = "load_start";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn stop(source_method: &'static str) -> Snapshot {
    let mut state = STATE.lock();
    let descriptor = state.load_descriptor;
    let reason = if state.loaded && state.running {
        "stopped"
    } else if state.loaded {
        "already_stopped"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(descriptor, inventory_change),
    );

    if state.loaded {
        state.running = false;
        state.stop_event_id = Some(event_id);
    }
    state.last_action = "stop";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn drop_service(source_method: &'static str) -> Snapshot {
    let mut state = STATE.lock();
    let descriptor = state.load_descriptor;
    let reason = if state.loaded {
        "dropped"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.loaded {
        "removed_current_boot_service"
    } else {
        "none"
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(descriptor, inventory_change),
    );

    state.loaded = false;
    state.running = false;
    state.drop_event_id = Some(event_id);
    state.last_action = "drop";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    let snapshot = state.snapshot();
    state.load_descriptor = LOAD_DESCRIPTOR;
    snapshot
}

fn health_probe(source_method: &'static str) -> (Snapshot, event_log::EventId) {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    let health = health_state(snapshot);
    let reason = if snapshot.running {
        "health_probe_healthy"
    } else if snapshot.loaded {
        "health_probe_stopped"
    } else {
        "health_probe_missing"
    };
    let event_id = event_log::record_hello_service_health(
        source_method,
        health,
        reason,
        lifecycle_binding(snapshot.load_descriptor, "none"),
    );
    (snapshot, event_id)
}

fn load_request(method: &str) -> Option<LoadRequest> {
    load_request_for_head(method, "module.load_ephemeral")
        .or_else(|| load_request_for_head(method, "service.load_ephemeral"))
}

fn load_request_for_head(method: &str, head: &'static str) -> Option<LoadRequest> {
    let method = method.trim();
    if !method_head_eq(method, head) {
        return None;
    }
    let target = method[head.len()..].trim();
    let descriptor = verified_load_descriptor_for_target(target)?;
    Some(LoadRequest {
        source_method: head,
        descriptor,
    })
}

fn verified_load_descriptor_for_target(target: &str) -> Option<LoadDescriptor> {
    let source = descriptor_source_for_target(target)?;
    if !descriptor_sources::validate_descriptor_source(source) {
        return None;
    }
    let descriptor = load_descriptor_from_source(source);
    if descriptor_target_matches(target, descriptor)
        || descriptor_source_target_matches(target, source)
    {
        Some(descriptor)
    } else {
        None
    }
}

fn descriptor_source_for_target(
    target: &str,
) -> Option<descriptor_sources::DescriptorSourceRecord> {
    if descriptor_source_target_matches_locator(
        target,
        descriptor_sources::HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR,
    ) || target.eq_ignore_ascii_case("host_bound:svc.demo.hello")
    {
        descriptor_sources::lookup_host_bound_descriptor_source(LOAD_DESCRIPTOR_ID)
    } else {
        descriptor_sources::lookup_current_image_descriptor_source(LOAD_DESCRIPTOR_ID)
    }
}

fn load_descriptor_from_source(
    source: descriptor_sources::DescriptorSourceRecord,
) -> LoadDescriptor {
    LoadDescriptor {
        schema: source.schema,
        id: source.id,
        canonicalization: source.canonicalization,
        source_locator: source.locator,
        source_kind: source.kind,
        binds_source_locator: source.binds_source_locator,
        binds_source_kind: source.binds_source_kind,
        binds_source_hash: source.binds_source_hash,
        source_text: source.text,
        service_id: source.service_id,
        artifact_id: source.artifact_id,
        artifact_kind: source.artifact_kind,
        scope: source.scope,
        classification: source.classification,
        persistence: source.persistence,
    }
}

fn descriptor_source_target_matches(
    target: &str,
    source: descriptor_sources::DescriptorSourceRecord,
) -> bool {
    descriptor_source_target_matches_locator(target, source.locator)
        || (source.locator == descriptor_sources::HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR
            && target.eq_ignore_ascii_case("host_bound:svc.demo.hello"))
}

fn descriptor_source_target_matches_locator(target: &str, locator: &str) -> bool {
    target.eq_ignore_ascii_case(locator)
}

fn target_arg_matches(method: &str, head: &str) -> bool {
    let method = method.trim();
    if !method_head_eq(method, head) {
        return false;
    }
    let target = method[head.len()..].trim();
    descriptor_target_matches(target, LOAD_DESCRIPTOR)
}

fn descriptor_target_matches(target: &str, descriptor: LoadDescriptor) -> bool {
    target.eq_ignore_ascii_case(descriptor.service_id)
        || target.eq_ignore_ascii_case("hello")
        || target.eq_ignore_ascii_case(descriptor.artifact_id)
        || target.eq_ignore_ascii_case(descriptor.id)
}

fn health_state(snapshot: Snapshot) -> &'static str {
    if snapshot.running {
        "healthy"
    } else if snapshot.loaded {
        "stopped"
    } else {
        "missing"
    }
}

fn lifecycle_binding(
    descriptor: LoadDescriptor,
    service_inventory_change: &'static str,
) -> event_log::HelloServiceLifecycleBinding {
    event_log::HelloServiceLifecycleBinding {
        descriptor_schema: descriptor.schema,
        descriptor_id: descriptor.id,
        descriptor_source_locator: descriptor.source_locator,
        descriptor_source_kind: descriptor.source_kind,
        descriptor_source_hash: descriptor_source_hash(descriptor),
        binds_source_locator: descriptor.binds_source_locator,
        binds_source_kind: descriptor.binds_source_kind,
        binds_source_hash: descriptor.binds_source_hash,
        descriptor_source_validated: true,
        service_inventory_change,
        persistence: descriptor.persistence,
        accepts_external_artifact_bytes: false,
        loads_external_artifact: false,
        writes_persistent_state: false,
    }
}

fn emit_health_response(method: &'static str, snapshot: Snapshot, event_id: event_log::EventId) {
    let descriptor = snapshot.load_descriptor;
    begin_response(method);
    raw_line("      \"schema\": \"raios.ram_only_hello_service.health.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"action\": \"health_probe\",");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw_line("        \"kind\": \"service\",");
    raw("        \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw_line(",");
    raw("        \"running\": ");
    raw_bool(snapshot.running);
    raw_line(",");
    raw("        \"health\": ");
    json_str(health_state(snapshot));
    raw_line(",");
    raw("        \"generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("        \"last_action\": ");
    json_str(snapshot.last_action);
    raw_line(",");
    raw("        \"last_reason\": ");
    json_str(snapshot.last_reason);
    raw_line(",");
    raw("        \"capabilities\": [");
    emit_inline_string_array(CAPABILITIES);
    raw_line("]");
    raw_line("      },");
    raw_line("      \"load_descriptor\": {");
    raw("        \"schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw_line("        \"source\": {");
    raw("          \"locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("          \"kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("          \"validated\": true,");
    raw("          \"sha256\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("          \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("          \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("          \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line("");
    raw_line("        }");
    raw_line("      },");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
}

fn emit_response(
    method: &'static str,
    action: &'static str,
    snapshot: Snapshot,
    descriptor: LoadDescriptor,
) {
    begin_response(method);
    raw_line("      \"schema\": \"raios.ram_only_hello_service.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw("      \"action\": ");
    json_str(action);
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id_option(snapshot.last_event_id);
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(snapshot.last_event_id);
    raw_line(",");
    emit_load_request(descriptor);
    raw_line(",");
    emit_load_descriptor(descriptor);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"load_descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"load_descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"load_descriptor_source_validated\": true,");
    raw("        \"load_descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw_line("        \"kind\": \"service\",");
    raw("        \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw_line(",");
    raw("        \"running\": ");
    raw_bool(snapshot.running);
    raw_line(",");
    raw("        \"generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("        \"health\": ");
    json_str(if snapshot.running {
        "healthy"
    } else if snapshot.loaded {
        "stopped"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"capabilities\": [");
    emit_inline_string_array(CAPABILITIES);
    raw_line("]");
    raw_line("      },");
    raw_line("      \"lifecycle\": {");
    raw("        \"last_action\": ");
    json_str(snapshot.last_action);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(snapshot.last_reason);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(snapshot.last_inventory_change);
    raw_line(",");
    raw("        \"load_event_id\": ");
    json_event_id_option(snapshot.load_event_id);
    raw_line(",");
    raw("        \"start_event_id\": ");
    json_event_id_option(snapshot.start_event_id);
    raw_line(",");
    raw("        \"stop_event_id\": ");
    json_event_id_option(snapshot.stop_event_id);
    raw_line(",");
    raw("        \"drop_event_id\": ");
    json_event_id_option(snapshot.drop_event_id);
    raw_line("");
    raw_line("      },");
    raw_line("      \"loader\": {");
    raw("        \"kind\": ");
    json_str(descriptor.artifact_kind);
    raw_line(",");
    raw("        \"descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"descriptor_source_locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("        \"descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"descriptor_source_validated\": true,");
    raw("        \"descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("        \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("        \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("        \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw_line("        \"accepts_external_artifact_bytes\": false,");
    raw_line("        \"loads_external_artifact\": false,");
    raw_line("        \"writes_persistent_state\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"grants_broad_mutation\": false");
    raw_line("      },");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"general_module_load\": \"unchanged_denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
}

fn emit_load_request(descriptor: LoadDescriptor) {
    raw_line("      \"load_request\": {");
    raw_line("        \"schema\": \"raios.current_boot_load_request.v0\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"descriptor_schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"descriptor_id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw("        \"descriptor_source_locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("        \"descriptor_source_kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("        \"descriptor_source_validated\": true,");
    raw("        \"descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("        \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("        \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("        \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw("        \"service_id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw_line("        \"accepted\": true");
    raw("      }");
}

fn emit_load_descriptor(descriptor: LoadDescriptor) {
    raw_line("      \"load_descriptor\": {");
    raw("        \"schema\": ");
    json_str(descriptor.schema);
    raw_line(",");
    raw("        \"id\": ");
    json_str(descriptor.id);
    raw_line(",");
    raw_line("        \"source\": {");
    raw("          \"canonicalization\": ");
    json_str(descriptor.canonicalization);
    raw_line(",");
    raw("          \"locator\": ");
    json_str(descriptor.source_locator);
    raw_line(",");
    raw("          \"kind\": ");
    json_str(descriptor.source_kind);
    raw_line(",");
    raw_line("          \"validated\": true,");
    raw("          \"sha256\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw_line(",");
    raw("          \"binds_source_locator\": ");
    json_opt_str(descriptor.binds_source_locator);
    raw_line(",");
    raw("          \"binds_source_kind\": ");
    json_opt_str(descriptor.binds_source_kind);
    raw_line(",");
    raw("          \"binds_source_hash\": ");
    json_sha256_option(descriptor.binds_source_hash);
    raw_line(",");
    raw("          \"text\": ");
    json_str(descriptor.source_text);
    raw_line("");
    raw_line("        },");
    raw("        \"service_id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"artifact_kind\": ");
    json_str(descriptor.artifact_kind);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(descriptor.scope);
    raw_line(",");
    raw("        \"classification\": ");
    json_str(descriptor.classification);
    raw_line(",");
    raw("        \"persistence\": ");
    json_str(descriptor.persistence);
    raw_line(",");
    raw_line("        \"accepts_external_artifact_bytes\": false,");
    raw_line("        \"loads_external_artifact\": false,");
    raw_line("        \"writes_persistent_state\": false");
    raw("      }");
}
