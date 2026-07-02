use spin::Mutex;

use sha2::{Digest, Sha256};

use crate::{
    agent_protocol_support::{
        begin_response, emit_inline_string_array, end_response, json_event_id_option, json_opt_str,
        json_sha256, json_sha256_option, json_str, method_eq, method_head_eq, raw, raw_bool,
        raw_fmt, raw_line,
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
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA: &str =
    "raios.current_boot_artifact_load_plan_preflight.v0";
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_ID: &str =
    "artifact_load_plan_preflight.current_boot.svc.demo.hello.v0";
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS: &str = "accepted_builtin_current_boot_only";
pub(crate) const SERVICE_SLOT_INTENT_SCHEMA: &str = "raios.ram_only_service_slot_intent.v0";
pub(crate) const SERVICE_SLOT_INTENT_ID: &str =
    "service_slot_intent.current_boot.svc.demo.hello.v0";
pub(crate) const RAM_ONLY_SERVICE_SLOT_ID: &str = "ram_only:svc.demo.hello";

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
    pub source_envelope: Option<descriptor_sources::DescriptorSourceEnvelope>,
    pub artifact_identity: descriptor_sources::ArtifactIdentityRecord,
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
    source_envelope: Some(descriptor_sources::HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE),
    artifact_identity: descriptor_sources::hello_builtin_artifact_identity(),
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

pub(crate) fn descriptor_source_signature_verified(descriptor: LoadDescriptor) -> bool {
    descriptor_sources::verify_descriptor_source_envelope_parts(
        descriptor.source_envelope,
        descriptor.source_locator,
        descriptor.source_kind,
        descriptor.source_text,
    )
}

pub(crate) fn artifact_identity_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_identity_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_content_binding_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_content_binding_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_reference_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_reference_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_reference_bytes_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_reference_bytes_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_load_plan_preflight_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA);
    hash_line_str(&mut hash, b"id", ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"status", ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    hash_line_str(&mut hash, b"service_id", descriptor.service_id);
    hash_line_str(&mut hash, b"artifact_id", descriptor.artifact_id);
    hash_line_str(&mut hash, b"load_descriptor_id", descriptor.id);
    hash_line_str(
        &mut hash,
        b"descriptor_source_locator",
        descriptor.source_locator,
    );
    hash_line_hash(
        &mut hash,
        b"descriptor_source_sha256",
        descriptor_source_hash(descriptor),
    );
    hash_line_str(
        &mut hash,
        b"artifact_identity_id",
        descriptor.artifact_identity.id,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_identity_sha256",
        artifact_identity_hash(descriptor),
    );
    hash_line_hash(
        &mut hash,
        b"artifact_content_binding_sha256",
        artifact_content_binding_hash(descriptor),
    );
    hash_line_str(
        &mut hash,
        b"artifact_reference_id",
        descriptor.artifact_identity.artifact_reference_id,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_reference_sha256",
        artifact_reference_hash(descriptor),
    );
    hash_line_hash(
        &mut hash,
        b"artifact_bytes_sha256",
        artifact_reference_bytes_hash(descriptor),
    );
    hash_line_str(&mut hash, b"service_slot_intent_id", SERVICE_SLOT_INTENT_ID);
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        RAM_ONLY_SERVICE_SLOT_ID,
    );
    hash_line_bool(&mut hash, b"accepted", true);
    hash_line_bool(&mut hash, b"authorizes_builtin_current_boot_start", true);
    hash_line_bool(&mut hash, b"authorizes_candidate_artifact_execution", false);
    hash_line_bool(&mut hash, b"accepts_external_artifact_bytes", false);
    hash_line_bool(&mut hash, b"loads_candidate_bytes", false);
    hash_line_bool(&mut hash, b"maps_executable_pages", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"grants_broad_mutation", false);
    finalize_sha256(hash)
}

fn finalize_sha256(hash: Sha256) -> [u8; 32] {
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn hash_line_str(hash: &mut Sha256, key: &'static [u8], value: &str) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value.as_bytes());
    hash.update(b"\n");
}

fn hash_line_hash(hash: &mut Sha256, key: &'static [u8], value: [u8; 32]) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value);
    hash.update(b"\n");
}

fn hash_line_bool(hash: &mut Sha256, key: &'static [u8], value: bool) {
    hash.update(key);
    if value {
        hash.update(b"=true\n");
    } else {
        hash.update(b"=false\n");
    }
}

pub(crate) fn artifact_identity_signature_verified(descriptor: LoadDescriptor) -> bool {
    let identity = descriptor.artifact_identity;
    descriptor_sources::verify_artifact_identity_envelope_parts(
        identity.signed_envelope,
        identity.id,
        identity.artifact_id,
        identity.text,
    )
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

pub(crate) fn is_descriptor_source_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.descriptor_source_trust_selftest")
}

pub(crate) fn is_artifact_reference_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.artifact_reference_trust_selftest")
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

pub(crate) fn emit_descriptor_source_trust_selftest() -> &'static str {
    let method = "service.descriptor_source_trust_selftest";
    let cases = descriptor_sources::hello_descriptor_source_trust_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    begin_response(method);
    raw_line("      \"schema\": \"raios.descriptor_source_trust_selftest.v0\",");
    raw("      \"id\": ");
    json_str(descriptor_sources::HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(descriptor_sources::hello_descriptor_source_trust_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"descriptor_source_locator\": ");
    json_str(LOAD_DESCRIPTOR_SOURCE_LOCATOR);
    raw_line(",");
    raw("      \"descriptor_source_kind\": ");
    json_str(LOAD_DESCRIPTOR_SOURCE_KIND);
    raw_line(",");
    raw("      \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_accept\": ");
        raw_bool(case.expected_accept);
        raw(", \"actual_accept\": ");
        raw_bool(case.actual_accept);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw(", \"reason\": ");
        json_str(case.reason);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"descriptor_bytes_intake\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
    method
}

pub(crate) fn emit_artifact_reference_trust_selftest() -> &'static str {
    let method = "service.artifact_reference_trust_selftest";
    let cases = descriptor_sources::hello_artifact_reference_trust_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    begin_response(method);
    raw_line("      \"schema\": \"raios.builtin_artifact_reference_trust_selftest.v0\",");
    raw("      \"id\": ");
    json_str(descriptor_sources::HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(descriptor_sources::hello_artifact_reference_trust_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"artifact_id\": ");
    json_str(ARTIFACT_ID);
    raw_line(",");
    raw("      \"artifact_reference\": ");
    emit_artifact_reference(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_accept\": ");
        raw_bool(case.expected_accept);
        raw(", \"actual_accept\": ");
        raw_bool(case.actual_accept);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw(", \"reason\": ");
        json_str(case.reason);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"artifact_bytes_intake\": \"denied\",");
    raw_line("        \"artifact_load\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
    method
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
    if !descriptor_sources::validate_builtin_hello_artifact_identity(descriptor.artifact_identity) {
        return None;
    }
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
        source_envelope: source.signed_envelope,
        artifact_identity: descriptor_sources::hello_builtin_artifact_identity(),
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
    let identity = descriptor.artifact_identity;
    let identity_envelope = identity.signed_envelope;
    event_log::HelloServiceLifecycleBinding {
        descriptor_schema: descriptor.schema,
        descriptor_id: descriptor.id,
        descriptor_source_locator: descriptor.source_locator,
        descriptor_source_kind: descriptor.source_kind,
        descriptor_source_hash: descriptor_source_hash(descriptor),
        descriptor_source_envelope_id: descriptor.source_envelope.map(|envelope| envelope.id),
        descriptor_source_envelope_hash: descriptor
            .source_envelope
            .map(|envelope| envelope.envelope_hash),
        descriptor_source_envelope_payload_hash: descriptor
            .source_envelope
            .map(|envelope| envelope.payload_hash),
        descriptor_source_envelope_trust_scope: descriptor
            .source_envelope
            .map(|envelope| envelope.trust_scope),
        descriptor_source_signature_algorithm: descriptor
            .source_envelope
            .map(|envelope| envelope.algorithm),
        descriptor_source_signature_public_key_hash: descriptor
            .source_envelope
            .map(|envelope| envelope.public_key_hash),
        descriptor_source_signature_hash: descriptor
            .source_envelope
            .map(|envelope| envelope.signature_hash),
        descriptor_source_signature_verified: descriptor_source_signature_verified(descriptor),
        artifact_identity_id: identity.id,
        artifact_identity_hash: artifact_identity_hash(descriptor),
        artifact_identity_envelope_id: identity_envelope.id,
        artifact_identity_envelope_hash: identity_envelope.envelope_hash,
        artifact_identity_envelope_payload_hash: identity_envelope.payload_hash,
        artifact_identity_envelope_trust_scope: identity_envelope.trust_scope,
        artifact_identity_signature_algorithm: identity_envelope.algorithm,
        artifact_identity_signature_public_key_hash: identity_envelope.public_key_hash,
        artifact_identity_signature_hash: identity_envelope.signature_hash,
        artifact_identity_signature_verified: artifact_identity_signature_verified(descriptor),
        artifact_identity_validated: descriptor_sources::validate_builtin_hello_artifact_identity(
            identity,
        ),
        artifact_content_binding_id: identity.artifact_content_binding_id,
        artifact_content_binding_hash: artifact_content_binding_hash(descriptor),
        artifact_content_source_locator: identity.artifact_content_source_locator,
        artifact_content_source_hash: identity.artifact_content_source_hash,
        artifact_content_trust_envelope_id: identity_envelope.id,
        artifact_content_trust_envelope_hash: identity_envelope.envelope_hash,
        artifact_content_trust_signature_verified: artifact_identity_signature_verified(descriptor),
        artifact_content_validated: descriptor_sources::validate_builtin_hello_artifact_identity(
            identity,
        ),
        artifact_reference_id: identity.artifact_reference_id,
        artifact_reference_hash: artifact_reference_hash(descriptor),
        artifact_reference_bytes_hash: artifact_reference_bytes_hash(descriptor),
        artifact_reference_content_binding_hash: identity.artifact_reference_content_binding_hash,
        artifact_reference_trust_envelope_id: identity_envelope.id,
        artifact_reference_trust_envelope_hash: identity_envelope.envelope_hash,
        artifact_reference_trust_signature_verified: artifact_identity_signature_verified(
            descriptor,
        ),
        artifact_reference_validated: descriptor_sources::validate_builtin_hello_artifact_identity(
            identity,
        ),
        artifact_load_plan_preflight_id: ARTIFACT_LOAD_PLAN_PREFLIGHT_ID,
        artifact_load_plan_preflight_hash: artifact_load_plan_preflight_hash(descriptor),
        artifact_load_plan_preflight_status: ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS,
        artifact_load_plan_preflight_accepted: true,
        service_slot_intent_id: SERVICE_SLOT_INTENT_ID,
        ram_only_service_slot_id: RAM_ONLY_SERVICE_SLOT_ID,
        binds_source_locator: descriptor.binds_source_locator,
        binds_source_kind: descriptor.binds_source_kind,
        binds_source_hash: descriptor.binds_source_hash,
        descriptor_source_validated: true,
        service_inventory_change,
        persistence: descriptor.persistence,
        accepts_external_artifact_bytes: false,
        loads_external_artifact: false,
        maps_executable_pages: false,
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
    raw_line(",");
    raw("          \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line("");
    raw_line("        },");
    raw("        \"artifact_identity\": ");
    emit_artifact_identity(descriptor);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
    raw_line("");
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
    raw("      \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
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
    raw("        \"load_descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
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
    raw("        \"descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
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
    raw_line("        \"maps_executable_pages\": false,");
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
    raw("        \"descriptor_source_signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw_line(",");
    raw("        \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw_line(",");
    raw("        \"artifact_content_binding_id\": ");
    json_str(descriptor.artifact_identity.artifact_content_binding_id);
    raw_line(",");
    raw("        \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_content_source_hash\": ");
    json_sha256(descriptor.artifact_identity.artifact_content_source_hash);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_content_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw_line(",");
    raw("        \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_reference_content_binding_hash\": ");
    json_sha256(
        descriptor
            .artifact_identity
            .artifact_reference_content_binding_hash,
    );
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_id\": ");
    json_str(descriptor.artifact_identity.signed_envelope.id);
    raw_line(",");
    raw("        \"artifact_reference_trust_envelope_hash\": ");
    json_sha256(descriptor.artifact_identity.signed_envelope.envelope_hash);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw_line(",");
    raw("        \"artifact_load_plan_preflight_status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw_line(",");
    raw("        \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
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
    raw("          \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(descriptor);
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
    raw("        \"artifact_identity\": ");
    emit_artifact_identity(descriptor);
    raw_line(",");
    raw("        \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(descriptor);
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
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"writes_persistent_state\": false");
    raw("      }");
}

pub(crate) fn emit_artifact_load_plan_preflight(descriptor: LoadDescriptor) {
    raw("{");
    raw("\"schema\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA);
    raw(", \"id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_ID);
    raw(", \"scope\": \"current_boot\"");
    raw(", \"classification\": \"local_only\"");
    raw(", \"status\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS);
    raw(", \"preflight_hash\": ");
    json_sha256(artifact_load_plan_preflight_hash(descriptor));
    raw(", \"service_id\": ");
    json_str(descriptor.service_id);
    raw(", \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw(", \"load_descriptor_id\": ");
    json_str(descriptor.id);
    raw(", \"descriptor_source_locator\": ");
    json_str(descriptor.source_locator);
    raw(", \"descriptor_source_hash\": ");
    json_sha256(descriptor_source_hash(descriptor));
    raw(", \"artifact_identity_id\": ");
    json_str(descriptor.artifact_identity.id);
    raw(", \"artifact_identity_hash\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw(", \"artifact_content_binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw(", \"artifact_reference_id\": ");
    json_str(descriptor.artifact_identity.artifact_reference_id);
    raw(", \"artifact_reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw(", \"artifact_bytes_sha256\": ");
    json_sha256(artifact_reference_bytes_hash(descriptor));
    raw(", \"service_slot_intent_schema\": ");
    json_str(SERVICE_SLOT_INTENT_SCHEMA);
    raw(", \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw(", \"accepted\": true");
    raw(", \"authorizes_builtin_current_boot_start\": true");
    raw(", \"authorizes_candidate_artifact_execution\": false");
    raw(", \"accepts_external_artifact_bytes\": false");
    raw(", \"loads_candidate_bytes\": false");
    raw(", \"maps_executable_pages\": false");
    raw(", \"writes_persistent_state\": false");
    raw(", \"writes_durable_audit_log\": false");
    raw(", \"installs_rollback_plan\": false");
    raw(", \"grants_broad_mutation\": false");
    raw("}");
}

pub(crate) fn emit_descriptor_source_signature_envelope(descriptor: LoadDescriptor) {
    let Some(envelope) = descriptor.source_envelope else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(envelope.schema);
    raw(", \"id\": ");
    json_str(envelope.id);
    raw(", \"algorithm\": ");
    json_str(envelope.algorithm);
    raw(", \"verification_phase\": ");
    json_str(envelope.verification_phase);
    raw(", \"trust_scope\": ");
    json_str(envelope.trust_scope);
    raw(", \"envelope_hash\": ");
    json_sha256(envelope.envelope_hash);
    raw(", \"payload_sha256\": ");
    json_sha256(envelope.payload_hash);
    raw(", \"public_key_sha256\": ");
    json_sha256(envelope.public_key_hash);
    raw(", \"signature_sha256\": ");
    json_sha256(envelope.signature_hash);
    raw(", \"signature_verified\": ");
    raw_bool(descriptor_source_signature_verified(descriptor));
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(envelope.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(envelope.authorizes_persistent_install);
    raw("}");
}

pub(crate) fn emit_artifact_identity(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.schema);
    raw(", \"id\": ");
    json_str(identity.id);
    raw(", \"canonicalization\": ");
    json_str(identity.canonicalization);
    raw(", \"sha256\": ");
    json_sha256(artifact_identity_hash(descriptor));
    raw(", \"service_id\": ");
    json_str(identity.service_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"artifact_kind\": ");
    json_str(identity.artifact_kind);
    raw(", \"load_descriptor_id\": ");
    json_str(identity.load_descriptor_id);
    raw(", \"scope\": ");
    json_str(identity.scope);
    raw(", \"classification\": ");
    json_str(identity.classification);
    raw(", \"persistence\": ");
    json_str(identity.persistence);
    raw(", \"content_binding\": ");
    emit_artifact_content_binding(descriptor);
    raw(", \"artifact_reference\": ");
    emit_artifact_reference(descriptor);
    raw(", \"signature_envelope\": ");
    emit_artifact_identity_signature_envelope(descriptor);
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.accepts_external_artifact_bytes);
    raw(", \"loads_external_artifact\": ");
    raw_bool(identity.loads_external_artifact);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.writes_persistent_state);
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(identity.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(identity.authorizes_persistent_install);
    raw(", \"authorizes_rollback_install\": ");
    raw_bool(identity.authorizes_rollback_install);
    raw("}");
}

pub(crate) fn emit_artifact_content_binding(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.artifact_content_binding_schema);
    raw(", \"id\": ");
    json_str(identity.artifact_content_binding_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"content_kind\": ");
    json_str(identity.artifact_content_kind);
    raw(", \"source_locator\": ");
    json_str(identity.artifact_content_source_locator);
    raw(", \"source_sha256\": ");
    json_sha256(identity.artifact_content_source_hash);
    raw(", \"binding_hash\": ");
    json_sha256(artifact_content_binding_hash(descriptor));
    raw(", \"trusted_by_envelope_id\": ");
    json_str(identity.signed_envelope.id);
    raw(", \"trusted_by_envelope_hash\": ");
    json_sha256(identity.signed_envelope.envelope_hash);
    raw(", \"trust_signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.artifact_content_accepts_external_artifact_bytes);
    raw(", \"loads_external_artifact\": ");
    raw_bool(identity.artifact_content_loads_external_artifact);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.artifact_content_maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.artifact_content_writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_artifact_reference(descriptor: LoadDescriptor) {
    let identity = descriptor.artifact_identity;
    raw("{");
    raw("\"schema\": ");
    json_str(identity.artifact_reference_schema);
    raw(", \"id\": ");
    json_str(identity.artifact_reference_id);
    raw(", \"artifact_id\": ");
    json_str(identity.artifact_id);
    raw(", \"service_id\": ");
    json_str(identity.service_id);
    raw(", \"reference_kind\": ");
    json_str(identity.artifact_reference_kind);
    raw(", \"artifact_locator\": ");
    json_str(identity.artifact_reference_locator);
    raw(", \"artifact_bytes_sha256\": ");
    json_sha256(identity.artifact_reference_bytes_hash);
    raw(", \"content_binding_hash\": ");
    json_sha256(identity.artifact_reference_content_binding_hash);
    raw(", \"reference_hash\": ");
    json_sha256(artifact_reference_hash(descriptor));
    raw(", \"trusted_by_envelope_id\": ");
    json_str(identity.signed_envelope.id);
    raw(", \"trusted_by_envelope_hash\": ");
    json_sha256(identity.signed_envelope.envelope_hash);
    raw(", \"trust_signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"validated\": ");
    raw_bool(descriptor_sources::validate_builtin_hello_artifact_identity(identity));
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(identity.artifact_reference_accepts_external_artifact_bytes);
    raw(", \"loads_artifact_as_code\": ");
    raw_bool(identity.artifact_reference_loads_artifact_as_code);
    raw(", \"maps_executable_pages\": ");
    raw_bool(identity.artifact_reference_maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(identity.artifact_reference_writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_artifact_identity_signature_envelope(descriptor: LoadDescriptor) {
    let envelope = descriptor.artifact_identity.signed_envelope;
    raw("{");
    raw("\"schema\": ");
    json_str(envelope.schema);
    raw(", \"id\": ");
    json_str(envelope.id);
    raw(", \"algorithm\": ");
    json_str(envelope.algorithm);
    raw(", \"verification_phase\": ");
    json_str(envelope.verification_phase);
    raw(", \"trust_scope\": ");
    json_str(envelope.trust_scope);
    raw(", \"envelope_hash\": ");
    json_sha256(envelope.envelope_hash);
    raw(", \"payload_sha256\": ");
    json_sha256(envelope.payload_hash);
    raw(", \"public_key_sha256\": ");
    json_sha256(envelope.public_key_hash);
    raw(", \"signature_sha256\": ");
    json_sha256(envelope.signature_hash);
    raw(", \"signature_verified\": ");
    raw_bool(artifact_identity_signature_verified(descriptor));
    raw(", \"authorizes_external_artifact_load\": ");
    raw_bool(envelope.authorizes_external_artifact_load);
    raw(", \"authorizes_persistent_install\": ");
    raw_bool(envelope.authorizes_persistent_install);
    raw(", \"authorizes_rollback_install\": ");
    raw_bool(envelope.authorizes_rollback_install);
    raw("}");
}
