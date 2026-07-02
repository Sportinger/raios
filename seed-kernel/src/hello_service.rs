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
pub(crate) const SERVICE_SLOT_ACTIVATION_SCHEMA: &str = "raios.ram_only_service_slot_activation.v0";
pub(crate) const SERVICE_SLOT_ACTIVATION_ID: &str =
    "service_slot_activation.current_boot.svc.demo.hello.v0";
pub(crate) const SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS: &str = "active_current_boot";
pub(crate) const SERVICE_SLOT_ACTIVATION_STOPPED_STATUS: &str = "stopped_current_boot";
pub(crate) const SERVICE_SLOT_ACTIVATION_CLEARED_STATUS: &str = "cleared_current_boot";
pub(crate) const SERVICE_SLOT_ACTIVATION_MISSING_STATUS: &str = "missing_current_boot";
pub(crate) const HELLO_STATE_SCHEMA: &str = "raios.ram_only_hello_service_state.v0";
pub(crate) const HELLO_STATE_ID: &str = "hello_state.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_STATE_MIGRATION_SCHEMA: &str =
    "raios.ram_only_hello_service_state_migration.v0";
pub(crate) const HELLO_STATE_MIGRATION_ID: &str =
    "hello_state_migration.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_SCHEMA: &str =
    "raios.ram_only_hello_service_hot_swap_probation.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_ID: &str =
    "hello_hot_swap_probation.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_STATUS: &str = "active_current_boot_probation";
const HELLO_ROLLBACK_PREVIEW_SCHEMA: &str = "raios.ram_only_hello_service_rollback_preview.v0";
const HELLO_ROLLBACK_PREVIEW_ID: &str = "hello_rollback_preview.current_boot.svc.demo.hello.v0";
const HELLO_ROLLBACK_PREVIEW_STATUS: &str = "preview_only_current_boot";
const HELLO_ROLLBACK_APPLY_SCHEMA: &str = "raios.ram_only_hello_service_rollback_apply.v0";
const HELLO_ROLLBACK_APPLY_ID: &str = "hello_rollback_apply.current_boot.svc.demo.hello.v0";
const HELLO_ROLLBACK_APPLY_STATUS: &str = "denied_missing_rollback_apply_authority";
const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_preflight.v0";
const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID: &str =
    "hello_rollback_transaction_preflight.current_boot.svc.demo.hello.v0";
const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS: &str = "denied_missing_write_authorities";
const HELLO_ROLLBACK_APPLY_CAPABILITY: &str = "cap.service.rollback_apply.current_boot";
const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_SCHEMA: &str =
    "raios.current_boot_artifact_load_plan_preflight_selftest.v0";
const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_ID: &str =
    "artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0";
const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_CASES: usize = 8;

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
struct ArtifactLoadPlanPreflightRecord {
    schema: &'static str,
    id: &'static str,
    scope: &'static str,
    classification: &'static str,
    status: &'static str,
    preflight_hash: [u8; 32],
    service_id: &'static str,
    artifact_id: &'static str,
    load_descriptor_id: &'static str,
    descriptor_source_locator: &'static str,
    descriptor_source_hash: [u8; 32],
    artifact_identity_id: &'static str,
    artifact_identity_hash: [u8; 32],
    artifact_content_binding_hash: [u8; 32],
    artifact_reference_id: &'static str,
    artifact_reference_hash: [u8; 32],
    artifact_bytes_sha256: [u8; 32],
    service_slot_intent_schema: &'static str,
    service_slot_intent_id: &'static str,
    ram_only_service_slot_id: &'static str,
    accepted: bool,
    authorizes_builtin_current_boot_start: bool,
    authorizes_candidate_artifact_execution: bool,
    accepts_external_artifact_bytes: bool,
    loads_candidate_bytes: bool,
    maps_executable_pages: bool,
    writes_persistent_state: bool,
    writes_durable_audit_log: bool,
    installs_rollback_plan: bool,
    grants_broad_mutation: bool,
}

#[derive(Clone, Copy)]
struct ServiceSlotActivationRecord {
    schema: &'static str,
    id: &'static str,
    scope: &'static str,
    classification: &'static str,
    persistence: &'static str,
    status: &'static str,
    activation_hash: [u8; 32],
    service_id: &'static str,
    artifact_id: &'static str,
    load_descriptor_id: &'static str,
    descriptor_source_hash: [u8; 32],
    artifact_load_plan_preflight_id: &'static str,
    artifact_load_plan_preflight_hash: [u8; 32],
    artifact_load_plan_preflight_status: &'static str,
    service_slot_intent_id: &'static str,
    ram_only_service_slot_id: &'static str,
    active: bool,
    accepted_preflight: bool,
    authorizes_builtin_current_boot_start: bool,
    authorizes_candidate_artifact_execution: bool,
    writes_persistent_state: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct HelloStateMigrationRecord {
    schema: &'static str,
    id: &'static str,
    scope: &'static str,
    classification: &'static str,
    persistence: &'static str,
    migration_hash: [u8; 32],
    service_id: &'static str,
    ram_only_service_slot_id: &'static str,
    from_version: &'static str,
    to_version: &'static str,
    pre_state_hash: [u8; 32],
    post_state_hash: [u8; 32],
    pre_state_counter: u64,
    post_state_counter: u64,
    state_preserved: bool,
    accepted: bool,
    writes_persistent_state: bool,
    writes_durable_audit_log: bool,
    installs_rollback_plan: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct HelloHotSwapProbationRecord {
    schema: &'static str,
    id: &'static str,
    scope: &'static str,
    classification: &'static str,
    persistence: &'static str,
    status: &'static str,
    probation_hash: [u8; 32],
    service_id: &'static str,
    ram_only_service_slot_id: &'static str,
    previous_version: &'static str,
    new_version: &'static str,
    previous_descriptor_id: &'static str,
    new_descriptor_id: &'static str,
    previous_descriptor_source_hash: [u8; 32],
    new_descriptor_source_hash: [u8; 32],
    previous_artifact_identity_id: &'static str,
    new_artifact_identity_id: &'static str,
    previous_artifact_identity_hash: [u8; 32],
    new_artifact_identity_hash: [u8; 32],
    previous_generation: u64,
    new_generation: u64,
    previous_state_hash: [u8; 32],
    new_state_hash: [u8; 32],
    previous_state_counter: u64,
    new_state_counter: u64,
    state_migration_hash: [u8; 32],
    accepted: bool,
    loads_candidate_bytes: bool,
    maps_executable_pages: bool,
    writes_persistent_state: bool,
    writes_durable_audit_log: bool,
    installs_rollback_plan: bool,
    applies_rollback: bool,
}

#[derive(Clone, Copy)]
struct ArtifactLoadPlanPreflightSelftestCase {
    name: &'static str,
    expected_accept: bool,
    actual_accept: bool,
    passed: bool,
    reason: &'static str,
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

pub(crate) fn service_version(descriptor: LoadDescriptor) -> &'static str {
    if descriptor.artifact_identity.id == descriptor_sources::HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ID
    {
        "v2"
    } else {
        "v1"
    }
}

pub(crate) fn artifact_load_plan_preflight_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    artifact_load_plan_preflight_record(descriptor).preflight_hash
}

pub(crate) fn service_slot_activation_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    service_slot_activation_record(descriptor, SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS, true)
        .activation_hash
}

pub(crate) fn hello_state_hash(state_counter: u64) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", HELLO_STATE_SCHEMA);
    hash_line_str(&mut hash, b"id", HELLO_STATE_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        RAM_ONLY_SERVICE_SLOT_ID,
    );
    hash_line_u64(&mut hash, b"state_counter", state_counter);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    finalize_sha256(hash)
}

fn hello_state_migration_record(
    from_descriptor: LoadDescriptor,
    to_descriptor: LoadDescriptor,
    pre_state_counter: u64,
    post_state_counter: u64,
    accepted: bool,
) -> HelloStateMigrationRecord {
    let pre_state_hash = hello_state_hash(pre_state_counter);
    let post_state_hash = hello_state_hash(post_state_counter);
    let mut record = HelloStateMigrationRecord {
        schema: HELLO_STATE_MIGRATION_SCHEMA,
        id: HELLO_STATE_MIGRATION_ID,
        scope: "current_boot",
        classification: "local_only",
        persistence: "none",
        migration_hash: [0; 32],
        service_id: SERVICE_ID,
        ram_only_service_slot_id: RAM_ONLY_SERVICE_SLOT_ID,
        from_version: service_version(from_descriptor),
        to_version: service_version(to_descriptor),
        pre_state_hash,
        post_state_hash,
        pre_state_counter,
        post_state_counter,
        state_preserved: pre_state_hash == post_state_hash
            && pre_state_counter == post_state_counter,
        accepted,
        writes_persistent_state: false,
        writes_durable_audit_log: false,
        installs_rollback_plan: false,
    };
    record.migration_hash = hello_state_migration_record_hash(record);
    record
}

fn hello_state_migration_record_hash(record: HelloStateMigrationRecord) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", record.schema);
    hash_line_str(&mut hash, b"id", record.id);
    hash_line_str(&mut hash, b"scope", record.scope);
    hash_line_str(&mut hash, b"classification", record.classification);
    hash_line_str(&mut hash, b"persistence", record.persistence);
    hash_line_str(&mut hash, b"service_id", record.service_id);
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        record.ram_only_service_slot_id,
    );
    hash_line_str(&mut hash, b"from_version", record.from_version);
    hash_line_str(&mut hash, b"to_version", record.to_version);
    hash_line_hash(&mut hash, b"pre_state_sha256", record.pre_state_hash);
    hash_line_hash(&mut hash, b"post_state_sha256", record.post_state_hash);
    hash_line_u64(&mut hash, b"pre_state_counter", record.pre_state_counter);
    hash_line_u64(&mut hash, b"post_state_counter", record.post_state_counter);
    hash_line_bool(&mut hash, b"state_preserved", record.state_preserved);
    hash_line_bool(&mut hash, b"accepted", record.accepted);
    hash_line_bool(
        &mut hash,
        b"writes_persistent_state",
        record.writes_persistent_state,
    );
    hash_line_bool(
        &mut hash,
        b"writes_durable_audit_log",
        record.writes_durable_audit_log,
    );
    hash_line_bool(
        &mut hash,
        b"installs_rollback_plan",
        record.installs_rollback_plan,
    );
    finalize_sha256(hash)
}

fn hello_hot_swap_probation_record(
    previous_descriptor: LoadDescriptor,
    new_descriptor: LoadDescriptor,
    previous_generation: u64,
    new_generation: u64,
    state_counter: u64,
    migration: HelloStateMigrationRecord,
) -> HelloHotSwapProbationRecord {
    let state_hash = hello_state_hash(state_counter);
    let mut record = HelloHotSwapProbationRecord {
        schema: HELLO_HOT_SWAP_PROBATION_SCHEMA,
        id: HELLO_HOT_SWAP_PROBATION_ID,
        scope: "current_boot",
        classification: "local_only",
        persistence: "none",
        status: HELLO_HOT_SWAP_PROBATION_STATUS,
        probation_hash: [0; 32],
        service_id: SERVICE_ID,
        ram_only_service_slot_id: RAM_ONLY_SERVICE_SLOT_ID,
        previous_version: service_version(previous_descriptor),
        new_version: service_version(new_descriptor),
        previous_descriptor_id: previous_descriptor.id,
        new_descriptor_id: new_descriptor.id,
        previous_descriptor_source_hash: descriptor_source_hash(previous_descriptor),
        new_descriptor_source_hash: descriptor_source_hash(new_descriptor),
        previous_artifact_identity_id: previous_descriptor.artifact_identity.id,
        new_artifact_identity_id: new_descriptor.artifact_identity.id,
        previous_artifact_identity_hash: artifact_identity_hash(previous_descriptor),
        new_artifact_identity_hash: artifact_identity_hash(new_descriptor),
        previous_generation,
        new_generation,
        previous_state_hash: state_hash,
        new_state_hash: state_hash,
        previous_state_counter: state_counter,
        new_state_counter: state_counter,
        state_migration_hash: migration.migration_hash,
        accepted: true,
        loads_candidate_bytes: false,
        maps_executable_pages: false,
        writes_persistent_state: false,
        writes_durable_audit_log: false,
        installs_rollback_plan: false,
        applies_rollback: false,
    };
    record.probation_hash = hello_hot_swap_probation_record_hash(record);
    record
}

fn hello_hot_swap_probation_record_hash(record: HelloHotSwapProbationRecord) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", record.schema);
    hash_line_str(&mut hash, b"id", record.id);
    hash_line_str(&mut hash, b"scope", record.scope);
    hash_line_str(&mut hash, b"classification", record.classification);
    hash_line_str(&mut hash, b"persistence", record.persistence);
    hash_line_str(&mut hash, b"status", record.status);
    hash_line_str(&mut hash, b"service_id", record.service_id);
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        record.ram_only_service_slot_id,
    );
    hash_line_str(&mut hash, b"previous_version", record.previous_version);
    hash_line_str(&mut hash, b"new_version", record.new_version);
    hash_line_str(
        &mut hash,
        b"previous_descriptor_id",
        record.previous_descriptor_id,
    );
    hash_line_str(&mut hash, b"new_descriptor_id", record.new_descriptor_id);
    hash_line_hash(
        &mut hash,
        b"previous_descriptor_source_sha256",
        record.previous_descriptor_source_hash,
    );
    hash_line_hash(
        &mut hash,
        b"new_descriptor_source_sha256",
        record.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"previous_artifact_identity_id",
        record.previous_artifact_identity_id,
    );
    hash_line_str(
        &mut hash,
        b"new_artifact_identity_id",
        record.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"previous_artifact_identity_sha256",
        record.previous_artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"new_artifact_identity_sha256",
        record.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"previous_generation",
        record.previous_generation,
    );
    hash_line_u64(&mut hash, b"new_generation", record.new_generation);
    hash_line_hash(
        &mut hash,
        b"previous_state_sha256",
        record.previous_state_hash,
    );
    hash_line_hash(&mut hash, b"new_state_sha256", record.new_state_hash);
    hash_line_u64(
        &mut hash,
        b"previous_state_counter",
        record.previous_state_counter,
    );
    hash_line_u64(&mut hash, b"new_state_counter", record.new_state_counter);
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        record.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"accepted", record.accepted);
    hash_line_bool(
        &mut hash,
        b"loads_candidate_bytes",
        record.loads_candidate_bytes,
    );
    hash_line_bool(
        &mut hash,
        b"maps_executable_pages",
        record.maps_executable_pages,
    );
    hash_line_bool(
        &mut hash,
        b"writes_persistent_state",
        record.writes_persistent_state,
    );
    hash_line_bool(
        &mut hash,
        b"writes_durable_audit_log",
        record.writes_durable_audit_log,
    );
    hash_line_bool(
        &mut hash,
        b"installs_rollback_plan",
        record.installs_rollback_plan,
    );
    hash_line_bool(&mut hash, b"applies_rollback", record.applies_rollback);
    finalize_sha256(hash)
}

fn hello_rollback_preview_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", HELLO_ROLLBACK_PREVIEW_SCHEMA);
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_PREVIEW_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", HELLO_ROLLBACK_PREVIEW_STATUS);
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_artifact_identity_id",
        probation.previous_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_artifact_identity_id",
        probation.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_generation",
        probation.previous_generation,
    );
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_hash(
        &mut hash,
        b"rollback_target_state_sha256",
        probation.previous_state_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_state_counter",
        probation.previous_state_counter,
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"read_only", true);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    finalize_sha256(hash)
}

fn hello_rollback_apply_denial_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", HELLO_ROLLBACK_APPLY_SCHEMA);
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_APPLY_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(&mut hash, b"status", HELLO_ROLLBACK_APPLY_STATUS);
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_str(
        &mut hash,
        b"current_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_bool(&mut hash, b"authorized", false);
    hash_line_bool(&mut hash, b"mutates_service_state", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    finalize_sha256(hash)
}

fn hello_rollback_transaction_preflight_hash(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(
        &mut hash,
        b"schema",
        HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA,
    );
    hash_line_str(&mut hash, b"id", HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID);
    hash_line_str(&mut hash, b"scope", "current_boot");
    hash_line_str(&mut hash, b"classification", "local_only");
    hash_line_str(&mut hash, b"persistence", "none");
    hash_line_str(
        &mut hash,
        b"status",
        HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS,
    );
    hash_line_str(&mut hash, b"service_id", SERVICE_ID);
    hash_line_str(
        &mut hash,
        b"requested_capability",
        HELLO_ROLLBACK_APPLY_CAPABILITY,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_apply_sha256",
        hello_rollback_apply_denial_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"rollback_preview_sha256",
        hello_rollback_preview_hash(snapshot, probation),
    );
    hash_line_hash(
        &mut hash,
        b"source_probation_sha256",
        probation.probation_hash,
    );
    hash_line_hash(
        &mut hash,
        b"current_state_sha256",
        hello_state_hash(snapshot.state_counter),
    );
    hash_line_u64(&mut hash, b"current_state_counter", snapshot.state_counter);
    hash_line_u64(&mut hash, b"current_generation", snapshot.generation);
    hash_line_str(
        &mut hash,
        b"rollback_target_descriptor_id",
        probation.previous_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_descriptor_source_sha256",
        probation.previous_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"rollback_target_artifact_identity_id",
        probation.previous_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_artifact_identity_sha256",
        probation.previous_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_generation",
        probation.previous_generation,
    );
    hash_line_hash(
        &mut hash,
        b"rollback_target_state_sha256",
        probation.previous_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"rollback_target_state_counter",
        probation.previous_state_counter,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_descriptor_id",
        probation.new_descriptor_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_descriptor_source_sha256",
        probation.new_descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"current_candidate_artifact_identity_id",
        probation.new_artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_artifact_identity_sha256",
        probation.new_artifact_identity_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_generation",
        probation.new_generation,
    );
    hash_line_hash(
        &mut hash,
        b"current_candidate_state_sha256",
        probation.new_state_hash,
    );
    hash_line_u64(
        &mut hash,
        b"current_candidate_state_counter",
        probation.new_state_counter,
    );
    hash_line_hash(
        &mut hash,
        b"state_migration_sha256",
        probation.state_migration_hash,
    );
    hash_line_bool(&mut hash, b"rollback_apply_authority_missing", true);
    hash_line_bool(&mut hash, b"rollback_transaction_authority_missing", true);
    hash_line_bool(&mut hash, b"durable_audit_write_authority_missing", true);
    hash_line_bool(&mut hash, b"persistent_install_authority_missing", true);
    hash_line_bool(&mut hash, b"mutates_service_state", false);
    hash_line_bool(&mut hash, b"applies_rollback", false);
    hash_line_bool(&mut hash, b"writes_persistent_state", false);
    hash_line_bool(&mut hash, b"writes_durable_audit_log", false);
    hash_line_bool(&mut hash, b"writes_rollback_store", false);
    hash_line_bool(&mut hash, b"installs_rollback_plan", false);
    hash_line_bool(&mut hash, b"accepts_external_artifact_bytes", false);
    hash_line_bool(&mut hash, b"loads_candidate_bytes", false);
    hash_line_bool(&mut hash, b"maps_executable_pages", false);
    hash_line_bool(&mut hash, b"provider_auto_load", false);
    hash_line_bool(&mut hash, b"grants_broad_mutation", false);
    finalize_sha256(hash)
}

fn artifact_load_plan_preflight_record(
    descriptor: LoadDescriptor,
) -> ArtifactLoadPlanPreflightRecord {
    let mut record = ArtifactLoadPlanPreflightRecord {
        schema: ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA,
        id: ARTIFACT_LOAD_PLAN_PREFLIGHT_ID,
        scope: "current_boot",
        classification: "local_only",
        status: ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS,
        preflight_hash: [0; 32],
        service_id: descriptor.service_id,
        artifact_id: descriptor.artifact_id,
        load_descriptor_id: descriptor.id,
        descriptor_source_locator: descriptor.source_locator,
        descriptor_source_hash: descriptor_source_hash(descriptor),
        artifact_identity_id: descriptor.artifact_identity.id,
        artifact_identity_hash: artifact_identity_hash(descriptor),
        artifact_content_binding_hash: artifact_content_binding_hash(descriptor),
        artifact_reference_id: descriptor.artifact_identity.artifact_reference_id,
        artifact_reference_hash: artifact_reference_hash(descriptor),
        artifact_bytes_sha256: artifact_reference_bytes_hash(descriptor),
        service_slot_intent_schema: SERVICE_SLOT_INTENT_SCHEMA,
        service_slot_intent_id: SERVICE_SLOT_INTENT_ID,
        ram_only_service_slot_id: RAM_ONLY_SERVICE_SLOT_ID,
        accepted: true,
        authorizes_builtin_current_boot_start: true,
        authorizes_candidate_artifact_execution: false,
        accepts_external_artifact_bytes: false,
        loads_candidate_bytes: false,
        maps_executable_pages: false,
        writes_persistent_state: false,
        writes_durable_audit_log: false,
        installs_rollback_plan: false,
        grants_broad_mutation: false,
    };
    record.preflight_hash = artifact_load_plan_preflight_record_hash(record);
    record
}

fn artifact_load_plan_preflight_record_hash(record: ArtifactLoadPlanPreflightRecord) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", record.schema);
    hash_line_str(&mut hash, b"id", record.id);
    hash_line_str(&mut hash, b"scope", record.scope);
    hash_line_str(&mut hash, b"classification", record.classification);
    hash_line_str(&mut hash, b"status", record.status);
    hash_line_str(&mut hash, b"service_id", record.service_id);
    hash_line_str(&mut hash, b"artifact_id", record.artifact_id);
    hash_line_str(&mut hash, b"load_descriptor_id", record.load_descriptor_id);
    hash_line_str(
        &mut hash,
        b"descriptor_source_locator",
        record.descriptor_source_locator,
    );
    hash_line_hash(
        &mut hash,
        b"descriptor_source_sha256",
        record.descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"artifact_identity_id",
        record.artifact_identity_id,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_identity_sha256",
        record.artifact_identity_hash,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_content_binding_sha256",
        record.artifact_content_binding_hash,
    );
    hash_line_str(
        &mut hash,
        b"artifact_reference_id",
        record.artifact_reference_id,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_reference_sha256",
        record.artifact_reference_hash,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_bytes_sha256",
        record.artifact_bytes_sha256,
    );
    hash_line_str(
        &mut hash,
        b"service_slot_intent_id",
        record.service_slot_intent_id,
    );
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        record.ram_only_service_slot_id,
    );
    hash_line_bool(&mut hash, b"accepted", record.accepted);
    hash_line_bool(
        &mut hash,
        b"authorizes_builtin_current_boot_start",
        record.authorizes_builtin_current_boot_start,
    );
    hash_line_bool(
        &mut hash,
        b"authorizes_candidate_artifact_execution",
        record.authorizes_candidate_artifact_execution,
    );
    hash_line_bool(
        &mut hash,
        b"accepts_external_artifact_bytes",
        record.accepts_external_artifact_bytes,
    );
    hash_line_bool(
        &mut hash,
        b"loads_candidate_bytes",
        record.loads_candidate_bytes,
    );
    hash_line_bool(
        &mut hash,
        b"maps_executable_pages",
        record.maps_executable_pages,
    );
    hash_line_bool(
        &mut hash,
        b"writes_persistent_state",
        record.writes_persistent_state,
    );
    hash_line_bool(
        &mut hash,
        b"writes_durable_audit_log",
        record.writes_durable_audit_log,
    );
    hash_line_bool(
        &mut hash,
        b"installs_rollback_plan",
        record.installs_rollback_plan,
    );
    hash_line_bool(
        &mut hash,
        b"grants_broad_mutation",
        record.grants_broad_mutation,
    );
    finalize_sha256(hash)
}

fn service_slot_activation_record(
    descriptor: LoadDescriptor,
    status: &'static str,
    active: bool,
) -> ServiceSlotActivationRecord {
    let preflight = artifact_load_plan_preflight_record(descriptor);
    let mut record = ServiceSlotActivationRecord {
        schema: SERVICE_SLOT_ACTIVATION_SCHEMA,
        id: SERVICE_SLOT_ACTIVATION_ID,
        scope: "current_boot",
        classification: "local_only",
        persistence: "none",
        status,
        activation_hash: [0; 32],
        service_id: descriptor.service_id,
        artifact_id: descriptor.artifact_id,
        load_descriptor_id: descriptor.id,
        descriptor_source_hash: descriptor_source_hash(descriptor),
        artifact_load_plan_preflight_id: preflight.id,
        artifact_load_plan_preflight_hash: preflight.preflight_hash,
        artifact_load_plan_preflight_status: preflight.status,
        service_slot_intent_id: preflight.service_slot_intent_id,
        ram_only_service_slot_id: preflight.ram_only_service_slot_id,
        active,
        accepted_preflight: preflight.accepted,
        authorizes_builtin_current_boot_start: preflight.authorizes_builtin_current_boot_start,
        authorizes_candidate_artifact_execution: preflight.authorizes_candidate_artifact_execution,
        writes_persistent_state: preflight.writes_persistent_state,
    };
    record.activation_hash = service_slot_activation_record_hash(record);
    record
}

fn service_slot_activation_record_hash(record: ServiceSlotActivationRecord) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_line_str(&mut hash, b"schema", record.schema);
    hash_line_str(&mut hash, b"id", record.id);
    hash_line_str(&mut hash, b"scope", record.scope);
    hash_line_str(&mut hash, b"classification", record.classification);
    hash_line_str(&mut hash, b"persistence", record.persistence);
    hash_line_str(&mut hash, b"service_id", record.service_id);
    hash_line_str(&mut hash, b"artifact_id", record.artifact_id);
    hash_line_str(&mut hash, b"load_descriptor_id", record.load_descriptor_id);
    hash_line_hash(
        &mut hash,
        b"descriptor_source_sha256",
        record.descriptor_source_hash,
    );
    hash_line_str(
        &mut hash,
        b"artifact_load_plan_preflight_id",
        record.artifact_load_plan_preflight_id,
    );
    hash_line_hash(
        &mut hash,
        b"artifact_load_plan_preflight_sha256",
        record.artifact_load_plan_preflight_hash,
    );
    hash_line_str(
        &mut hash,
        b"artifact_load_plan_preflight_status",
        record.artifact_load_plan_preflight_status,
    );
    hash_line_str(
        &mut hash,
        b"service_slot_intent_id",
        record.service_slot_intent_id,
    );
    hash_line_str(
        &mut hash,
        b"ram_only_service_slot_id",
        record.ram_only_service_slot_id,
    );
    hash_line_bool(&mut hash, b"accepted_preflight", record.accepted_preflight);
    hash_line_bool(
        &mut hash,
        b"authorizes_builtin_current_boot_start",
        record.authorizes_builtin_current_boot_start,
    );
    hash_line_bool(
        &mut hash,
        b"authorizes_candidate_artifact_execution",
        record.authorizes_candidate_artifact_execution,
    );
    hash_line_bool(
        &mut hash,
        b"writes_persistent_state",
        record.writes_persistent_state,
    );
    finalize_sha256(hash)
}

fn validate_artifact_load_plan_preflight_record(
    record: ArtifactLoadPlanPreflightRecord,
    descriptor: LoadDescriptor,
) -> bool {
    record.schema == ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA
        && record.id == ARTIFACT_LOAD_PLAN_PREFLIGHT_ID
        && record.scope == "current_boot"
        && record.classification == "local_only"
        && record.status == ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS
        && record.preflight_hash == artifact_load_plan_preflight_record_hash(record)
        && record.service_id == descriptor.service_id
        && record.artifact_id == descriptor.artifact_id
        && record.load_descriptor_id == descriptor.id
        && record.descriptor_source_locator == descriptor.source_locator
        && record.descriptor_source_hash == descriptor_source_hash(descriptor)
        && record.artifact_identity_id == descriptor.artifact_identity.id
        && record.artifact_identity_hash == artifact_identity_hash(descriptor)
        && record.artifact_content_binding_hash == artifact_content_binding_hash(descriptor)
        && record.artifact_reference_id == descriptor.artifact_identity.artifact_reference_id
        && record.artifact_reference_hash == artifact_reference_hash(descriptor)
        && record.artifact_bytes_sha256 == artifact_reference_bytes_hash(descriptor)
        && record.service_slot_intent_schema == SERVICE_SLOT_INTENT_SCHEMA
        && record.service_slot_intent_id == SERVICE_SLOT_INTENT_ID
        && record.ram_only_service_slot_id == RAM_ONLY_SERVICE_SLOT_ID
        && record.accepted
        && record.authorizes_builtin_current_boot_start
        && !record.authorizes_candidate_artifact_execution
        && !record.accepts_external_artifact_bytes
        && !record.loads_candidate_bytes
        && !record.maps_executable_pages
        && !record.writes_persistent_state
        && !record.writes_durable_audit_log
        && !record.installs_rollback_plan
        && !record.grants_broad_mutation
}

fn artifact_load_plan_preflight_selftest_hash() -> [u8; 32] {
    sha256_bytes(
        b"schema=raios.current_boot_artifact_load_plan_preflight_selftest.v0\n\
id=artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0\n\
cases=valid_preflight,tampered_descriptor_source_hash,tampered_artifact_identity_hash,tampered_content_binding_hash,tampered_artifact_reference_hash,tampered_artifact_bytes_hash,tampered_service_slot_intent,tampered_denial_flags",
    )
}

fn artifact_load_plan_preflight_selftest_cases(
) -> [ArtifactLoadPlanPreflightSelftestCase; ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_CASES] {
    let valid = artifact_load_plan_preflight_record(LOAD_DESCRIPTOR);
    let mut bad_descriptor_source = valid;
    bad_descriptor_source.descriptor_source_hash = [0x11; 32];
    let mut bad_artifact_identity = valid;
    bad_artifact_identity.artifact_identity_hash = [0x22; 32];
    let mut bad_content_binding = valid;
    bad_content_binding.artifact_content_binding_hash = [0x33; 32];
    let mut bad_artifact_reference = valid;
    bad_artifact_reference.artifact_reference_hash = [0x44; 32];
    let mut bad_artifact_bytes = valid;
    bad_artifact_bytes.artifact_bytes_sha256 = [0x55; 32];
    let mut bad_service_slot = valid;
    bad_service_slot.service_slot_intent_id = "service_slot_intent.current_boot.svc.demo.bad.v0";
    bad_service_slot.ram_only_service_slot_id = "ram_only:svc.demo.bad";
    let mut bad_denial_flags = valid;
    bad_denial_flags.authorizes_candidate_artifact_execution = true;
    bad_denial_flags.maps_executable_pages = true;

    [
        preflight_case(
            "valid_current_boot_load_plan_preflight",
            true,
            validate_artifact_load_plan_preflight_record(valid, LOAD_DESCRIPTOR),
            "accepted_current_boot_load_plan_preflight",
        ),
        preflight_case(
            "tampered_descriptor_source_hash_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_descriptor_source),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_selected_descriptor_source_hash",
        ),
        preflight_case(
            "tampered_artifact_identity_hash_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_artifact_identity),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_artifact_identity_hash",
        ),
        preflight_case(
            "tampered_content_binding_hash_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_content_binding),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_content_binding_hash",
        ),
        preflight_case(
            "tampered_artifact_reference_hash_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_artifact_reference),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_artifact_reference_hash",
        ),
        preflight_case(
            "tampered_artifact_bytes_hash_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_artifact_bytes),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_artifact_bytes_hash",
        ),
        preflight_case(
            "tampered_service_slot_intent_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_service_slot),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_bind_ram_only_service_slot_intent",
        ),
        preflight_case(
            "tampered_denial_flags_denied",
            false,
            validate_artifact_load_plan_preflight_record(
                rehash_preflight_record(bad_denial_flags),
                LOAD_DESCRIPTOR,
            ),
            "preflight_must_keep_candidate_execution_and_mapping_denied",
        ),
    ]
}

fn rehash_preflight_record(
    mut record: ArtifactLoadPlanPreflightRecord,
) -> ArtifactLoadPlanPreflightRecord {
    record.preflight_hash = artifact_load_plan_preflight_record_hash(record);
    record
}

fn preflight_case(
    name: &'static str,
    expected_accept: bool,
    actual_accept: bool,
    reason: &'static str,
) -> ArtifactLoadPlanPreflightSelftestCase {
    ArtifactLoadPlanPreflightSelftestCase {
        name,
        expected_accept,
        actual_accept,
        passed: expected_accept == actual_accept,
        reason,
    }
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
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

fn hash_line_u64(hash: &mut Sha256, key: &'static [u8], value: u64) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value.to_le_bytes());
    hash.update(b"\n");
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
    pub state_counter: u64,
    pub state_migration: Option<HelloStateMigrationRecord>,
    pub hot_swap_probation: Option<HelloHotSwapProbationRecord>,
    pub load_descriptor: LoadDescriptor,
    pub last_action: &'static str,
    pub last_reason: &'static str,
    pub last_inventory_change: &'static str,
    pub last_event_id: Option<event_log::EventId>,
    pub load_event_id: Option<event_log::EventId>,
    pub start_event_id: Option<event_log::EventId>,
    pub hot_swap_event_id: Option<event_log::EventId>,
    pub stop_event_id: Option<event_log::EventId>,
    pub drop_event_id: Option<event_log::EventId>,
}

#[derive(Clone, Copy)]
struct State {
    loaded: bool,
    running: bool,
    generation: u64,
    state_counter: u64,
    state_migration: Option<HelloStateMigrationRecord>,
    hot_swap_probation: Option<HelloHotSwapProbationRecord>,
    load_descriptor: LoadDescriptor,
    last_action: &'static str,
    last_reason: &'static str,
    last_inventory_change: &'static str,
    last_event_id: Option<event_log::EventId>,
    load_event_id: Option<event_log::EventId>,
    start_event_id: Option<event_log::EventId>,
    hot_swap_event_id: Option<event_log::EventId>,
    stop_event_id: Option<event_log::EventId>,
    drop_event_id: Option<event_log::EventId>,
}

impl State {
    const fn new() -> Self {
        Self {
            loaded: false,
            running: false,
            generation: 0,
            state_counter: 0,
            state_migration: None,
            hot_swap_probation: None,
            load_descriptor: LOAD_DESCRIPTOR,
            last_action: "none",
            last_reason: "not_loaded",
            last_inventory_change: "none",
            last_event_id: None,
            load_event_id: None,
            start_event_id: None,
            hot_swap_event_id: None,
            stop_event_id: None,
            drop_event_id: None,
        }
    }

    fn snapshot(self) -> Snapshot {
        Snapshot {
            loaded: self.loaded,
            running: self.running,
            generation: self.generation,
            state_counter: self.state_counter,
            state_migration: self.state_migration,
            hot_swap_probation: self.hot_swap_probation,
            load_descriptor: self.load_descriptor,
            last_action: self.last_action,
            last_reason: self.last_reason,
            last_inventory_change: self.last_inventory_change,
            last_event_id: self.last_event_id,
            load_event_id: self.load_event_id,
            start_event_id: self.start_event_id,
            hot_swap_event_id: self.hot_swap_event_id,
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

pub(crate) fn is_start_method(method: &str) -> bool {
    target_arg_matches(method, "service.start")
}

pub(crate) fn is_restart_method(method: &str) -> bool {
    target_arg_matches(method, "service.restart")
}

pub(crate) fn is_hot_swap_method(method: &str) -> bool {
    hot_swap_request(method).is_some() || reset_state_hot_swap_target(method)
}

pub(crate) fn is_drop_method(method: &str) -> bool {
    target_arg_matches(method, "service.drop")
}

pub(crate) fn is_health_method(method: &str) -> bool {
    target_arg_matches(method, "service.health")
}

pub(crate) fn is_rollback_preview_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_preview")
}

pub(crate) fn is_rollback_apply_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_apply")
}

pub(crate) fn is_descriptor_source_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.descriptor_source_trust_selftest")
}

pub(crate) fn is_artifact_reference_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.artifact_reference_trust_selftest")
}

pub(crate) fn is_artifact_load_plan_preflight_selftest_method(method: &str) -> bool {
    method_eq(method, "service.artifact_load_plan_preflight_selftest")
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

pub(crate) fn emit_start(_method: &str) -> &'static str {
    let snapshot = start("service.start");
    emit_response("service.start", "start", snapshot, snapshot.load_descriptor);
    "service.start"
}

pub(crate) fn emit_restart(_method: &str) -> &'static str {
    let snapshot = restart("service.restart");
    emit_response(
        "service.restart",
        "restart",
        snapshot,
        snapshot.load_descriptor,
    );
    "service.restart"
}

pub(crate) fn emit_hot_swap(method: &str) -> &'static str {
    if let Some(request) = hot_swap_request(method) {
        let snapshot = hot_swap(request.source_method, request.descriptor);
        emit_response(
            request.source_method,
            "hot_swap",
            snapshot,
            request.descriptor,
        );
        request.source_method
    } else if reset_state_hot_swap_target(method) {
        let (snapshot, event_id, migration) = denied_reset_state_hot_swap();
        emit_hot_swap_state_migration_denied("service.hot_swap", snapshot, event_id, migration);
        "service.hot_swap"
    } else {
        "service.hot_swap"
    }
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

pub(crate) fn emit_rollback_preview(_method: &str) -> &'static str {
    let (snapshot, event_id) = rollback_preview("service.rollback_preview");
    emit_rollback_preview_response("service.rollback_preview", snapshot, event_id);
    "service.rollback_preview"
}

pub(crate) fn emit_rollback_apply(_method: &str) -> &'static str {
    let (snapshot, event_id) = rollback_apply("service.rollback_apply");
    emit_rollback_apply_denied("service.rollback_apply", snapshot, event_id);
    "service.rollback_apply"
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

pub(crate) fn emit_artifact_load_plan_preflight_selftest() -> &'static str {
    let method = "service.artifact_load_plan_preflight_selftest";
    let cases = artifact_load_plan_preflight_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    begin_response(method);
    raw("      \"schema\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(artifact_load_plan_preflight_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"artifact_id\": ");
    json_str(ARTIFACT_ID);
    raw_line(",");
    raw("      \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("      \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw_line(",");
    raw("      \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(LOAD_DESCRIPTOR);
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
    raw_line("        \"external_artifact_bytes\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
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
    let state_counter = if state.loaded { state.state_counter } else { 1 };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS,
            true,
            state_counter,
            None,
            None,
        ),
    );

    if !state.loaded {
        state.generation = state.generation.saturating_add(1);
        state.state_counter = state_counter;
        state.load_event_id = Some(event_id);
    }
    state.loaded = true;
    state.running = true;
    state.load_descriptor = descriptor;
    state.state_migration = None;
    state.hot_swap_probation = None;
    state.start_event_id = Some(event_id);
    state.last_action = "load_start";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn start(source_method: &'static str) -> Snapshot {
    let mut state = STATE.lock();
    let descriptor = state.load_descriptor;
    let reason = if state.loaded && state.running {
        "already_running"
    } else if state.loaded {
        "started_loaded_service"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    let activation_status = if state.loaded {
        SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    };
    let state_counter = if state.loaded && !state.running {
        state.state_counter.saturating_add(1)
    } else {
        state.state_counter
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            activation_status,
            state.loaded,
            state_counter,
            None,
            None,
        ),
    );

    if state.loaded {
        state.running = true;
        state.state_counter = state_counter;
        state.state_migration = None;
        state.hot_swap_probation = None;
        state.start_event_id = Some(event_id);
    }
    state.last_action = "start";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn restart(source_method: &'static str) -> Snapshot {
    let mut state = STATE.lock();
    let descriptor = state.load_descriptor;
    let reason = if state.loaded {
        "restarted_loaded_service"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    let activation_status = if state.loaded {
        SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    };
    let state_counter = if state.loaded {
        state.state_counter.saturating_add(1)
    } else {
        state.state_counter
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            activation_status,
            state.loaded,
            state_counter,
            None,
            None,
        ),
    );

    if state.loaded {
        state.running = true;
        state.state_counter = state_counter;
        state.state_migration = None;
        state.hot_swap_probation = None;
        state.start_event_id = Some(event_id);
    }
    state.last_action = "restart";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn hot_swap(source_method: &'static str, descriptor: LoadDescriptor) -> Snapshot {
    let mut state = STATE.lock();
    let reason = if state.loaded {
        "hot_swapped_builtin_service"
    } else {
        "not_loaded"
    };
    let inventory_change = if state.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    let activation_status = if state.loaded {
        SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    };
    let state_counter = state.state_counter;
    let previous_descriptor = state.load_descriptor;
    let previous_generation = state.generation;
    let new_generation = previous_generation.saturating_add(1);
    let (migration, probation) = if state.loaded {
        let migration = hello_state_migration_record(
            previous_descriptor,
            descriptor,
            state_counter,
            state_counter,
            true,
        );
        let probation = hello_hot_swap_probation_record(
            previous_descriptor,
            descriptor,
            previous_generation,
            new_generation,
            state_counter,
            migration,
        );
        (Some(migration), Some(probation))
    } else {
        (None, None)
    };
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            activation_status,
            state.loaded,
            state_counter,
            migration,
            probation,
        ),
    );

    if state.loaded {
        state.generation = new_generation;
        state.running = true;
        state.load_descriptor = descriptor;
        state.load_event_id = Some(event_id);
        state.start_event_id = Some(event_id);
        state.hot_swap_event_id = Some(event_id);
        state.state_migration = migration;
        state.hot_swap_probation = probation;
    }
    state.last_action = "hot_swap";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

fn denied_reset_state_hot_swap() -> (Snapshot, event_log::EventId, HelloStateMigrationRecord) {
    let snapshot = STATE.lock().snapshot();
    let migration = hello_state_migration_record(
        snapshot.load_descriptor,
        snapshot.load_descriptor,
        snapshot.state_counter,
        0,
        false,
    );
    let event_id = event_log::record_hello_service_lifecycle(
        "service.hot_swap",
        "capability_denied",
        "state_migration_would_reset_state",
        lifecycle_binding(
            snapshot.load_descriptor,
            "none",
            service_slot_activation_status(snapshot),
            service_slot_activation_active(snapshot),
            snapshot.state_counter,
            Some(migration),
            None,
        ),
    );
    (snapshot, event_id, migration)
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
    let activation_status = if state.loaded {
        SERVICE_SLOT_ACTIVATION_STOPPED_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    };
    let state_counter = state.state_counter;
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            activation_status,
            state.loaded,
            state_counter,
            None,
            None,
        ),
    );

    if state.loaded {
        state.running = false;
        state.state_migration = None;
        state.hot_swap_probation = None;
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
    let activation_status = if state.loaded {
        SERVICE_SLOT_ACTIVATION_CLEARED_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    };
    let state_counter = state.state_counter;
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        reason,
        lifecycle_binding(
            descriptor,
            inventory_change,
            activation_status,
            false,
            state_counter,
            None,
            None,
        ),
    );

    state.loaded = false;
    state.running = false;
    state.state_counter = 0;
    state.state_migration = None;
    state.hot_swap_probation = None;
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
        lifecycle_binding(
            snapshot.load_descriptor,
            "none",
            service_slot_activation_status(snapshot),
            service_slot_activation_active(snapshot),
            snapshot.state_counter,
            snapshot.state_migration,
            snapshot.hot_swap_probation,
        ),
    );
    (snapshot, event_id)
}

fn rollback_preview(source_method: &'static str) -> (Snapshot, event_log::EventId) {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    let reason = if snapshot.hot_swap_probation.is_some() {
        "rollback_preview_ready"
    } else if snapshot.loaded {
        "hot_swap_probation_missing"
    } else {
        "service_not_loaded"
    };
    let mut binding = lifecycle_binding(
        snapshot.load_descriptor,
        "none",
        service_slot_activation_status(snapshot),
        service_slot_activation_active(snapshot),
        snapshot.state_counter,
        snapshot.state_migration,
        snapshot.hot_swap_probation,
    );
    bind_rollback_preview(&mut binding, snapshot);
    let event_id = event_log::record_hello_service_rollback_preview(
        source_method,
        if snapshot.hot_swap_probation.is_some() {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        binding,
    );
    (snapshot, event_id)
}

fn rollback_apply(source_method: &'static str) -> (Snapshot, event_log::EventId) {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    let reason = if snapshot.hot_swap_probation.is_some() {
        "rollback_apply_authority_missing"
    } else if snapshot.loaded {
        "rollback_preview_or_probation_missing"
    } else {
        "service_not_loaded"
    };
    let mut binding = lifecycle_binding(
        snapshot.load_descriptor,
        "none",
        service_slot_activation_status(snapshot),
        service_slot_activation_active(snapshot),
        snapshot.state_counter,
        snapshot.state_migration,
        snapshot.hot_swap_probation,
    );
    bind_rollback_preview(&mut binding, snapshot);
    bind_rollback_apply_denial(&mut binding, snapshot);
    bind_rollback_transaction_preflight(&mut binding, snapshot);
    let event_id = event_log::record_hello_service_rollback_apply(source_method, reason, binding);
    (snapshot, event_id)
}

fn bind_rollback_preview(
    binding: &mut event_log::HelloServiceLifecycleBinding,
    snapshot: Snapshot,
) {
    if let Some(probation) = snapshot.hot_swap_probation {
        binding.rollback_preview_schema = Some(HELLO_ROLLBACK_PREVIEW_SCHEMA);
        binding.rollback_preview_id = Some(HELLO_ROLLBACK_PREVIEW_ID);
        binding.rollback_preview_hash = Some(hello_rollback_preview_hash(snapshot, probation));
        binding.rollback_preview_status = Some(HELLO_ROLLBACK_PREVIEW_STATUS);
    }
}

fn bind_rollback_apply_denial(
    binding: &mut event_log::HelloServiceLifecycleBinding,
    snapshot: Snapshot,
) {
    binding.rollback_apply_schema = Some(HELLO_ROLLBACK_APPLY_SCHEMA);
    binding.rollback_apply_id = Some(HELLO_ROLLBACK_APPLY_ID);
    binding.rollback_apply_status = Some(HELLO_ROLLBACK_APPLY_STATUS);
    if let Some(probation) = snapshot.hot_swap_probation {
        binding.rollback_apply_hash = Some(hello_rollback_apply_denial_hash(snapshot, probation));
    }
}

fn bind_rollback_transaction_preflight(
    binding: &mut event_log::HelloServiceLifecycleBinding,
    snapshot: Snapshot,
) {
    if let Some(probation) = snapshot.hot_swap_probation {
        binding.rollback_transaction_preflight_schema =
            Some(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA);
        binding.rollback_transaction_preflight_id = Some(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID);
        binding.rollback_transaction_preflight_hash = Some(
            hello_rollback_transaction_preflight_hash(snapshot, probation),
        );
        binding.rollback_transaction_preflight_status =
            Some(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS);
        binding.rollback_transaction_authority_missing = true;
        binding.rollback_durable_audit_write_authority_missing = true;
        binding.rollback_persistent_install_authority_missing = true;
    }
}

fn load_request(method: &str) -> Option<LoadRequest> {
    load_request_for_head(method, "module.load_ephemeral")
        .or_else(|| load_request_for_head(method, "service.load_ephemeral"))
}

fn hot_swap_request(method: &str) -> Option<LoadRequest> {
    load_request_for_head(method, "service.hot_swap")
}

fn load_request_for_head(method: &str, head: &'static str) -> Option<LoadRequest> {
    let method = method.trim();
    if !method_head_eq(method, head) {
        return None;
    }
    let target = method[head.len()..].trim();
    let descriptor = verified_load_descriptor_for_target(target, head == "service.hot_swap")?;
    Some(LoadRequest {
        source_method: head,
        descriptor,
    })
}

fn verified_load_descriptor_for_target(
    target: &str,
    allow_replacement: bool,
) -> Option<LoadDescriptor> {
    let source = descriptor_source_for_target(target)?;
    if !descriptor_sources::validate_descriptor_source(source) {
        return None;
    }
    let replacement_target = replacement_target_matches(target);
    let artifact_identity = if allow_replacement && replacement_target {
        descriptor_sources::hello_builtin_artifact_identity_v2()
    } else {
        descriptor_sources::hello_builtin_artifact_identity()
    };
    let descriptor = load_descriptor_from_source(source, artifact_identity);
    if !descriptor_sources::validate_builtin_hello_artifact_identity(descriptor.artifact_identity) {
        return None;
    }
    if descriptor_target_matches(target, descriptor)
        || descriptor_source_target_matches(target, source)
        || (allow_replacement && replacement_target)
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
    artifact_identity: descriptor_sources::ArtifactIdentityRecord,
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
        artifact_identity,
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

fn replacement_target_matches(target: &str) -> bool {
    target.eq_ignore_ascii_case("svc.demo.hello.v2")
        || target.eq_ignore_ascii_case("hello.v2")
        || target.eq_ignore_ascii_case(descriptor_sources::HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ID)
}

fn reset_state_hot_swap_target(method: &str) -> bool {
    let method = method.trim();
    if !method_head_eq(method, "service.hot_swap") {
        return false;
    }
    let target = method["service.hot_swap".len()..].trim();
    target.eq_ignore_ascii_case("svc.demo.hello.reset_state")
        || target.eq_ignore_ascii_case("hello.reset_state")
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

pub(crate) fn service_slot_activation_status(snapshot: Snapshot) -> &'static str {
    if snapshot.running {
        SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS
    } else if snapshot.loaded {
        SERVICE_SLOT_ACTIVATION_STOPPED_STATUS
    } else if snapshot.last_action == "drop" && snapshot.last_reason == "dropped" {
        SERVICE_SLOT_ACTIVATION_CLEARED_STATUS
    } else {
        SERVICE_SLOT_ACTIVATION_MISSING_STATUS
    }
}

pub(crate) fn service_slot_activation_active(snapshot: Snapshot) -> bool {
    snapshot.loaded
}

fn lifecycle_binding(
    descriptor: LoadDescriptor,
    service_inventory_change: &'static str,
    service_slot_activation_status: &'static str,
    service_slot_activation_active: bool,
    state_counter: u64,
    state_migration: Option<HelloStateMigrationRecord>,
    hot_swap_probation: Option<HelloHotSwapProbationRecord>,
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
        service_slot_activation_id: SERVICE_SLOT_ACTIVATION_ID,
        service_slot_activation_hash: service_slot_activation_hash(descriptor),
        service_slot_activation_status,
        service_slot_activation_active,
        hello_state_schema: HELLO_STATE_SCHEMA,
        hello_state_id: HELLO_STATE_ID,
        hello_state_hash: hello_state_hash(state_counter),
        hello_state_counter: state_counter,
        state_migration_schema: state_migration.map(|record| record.schema),
        state_migration_id: state_migration.map(|record| record.id),
        state_migration_hash: state_migration.map(|record| record.migration_hash),
        migration_from_version: state_migration.map(|record| record.from_version),
        migration_to_version: state_migration.map(|record| record.to_version),
        pre_migration_state_hash: state_migration.map(|record| record.pre_state_hash),
        post_migration_state_hash: state_migration.map(|record| record.post_state_hash),
        pre_migration_state_counter: state_migration.map(|record| record.pre_state_counter),
        post_migration_state_counter: state_migration.map(|record| record.post_state_counter),
        state_migration_preserved: state_migration
            .map(|record| record.state_preserved)
            .unwrap_or(false),
        state_migration_accepted: state_migration
            .map(|record| record.accepted)
            .unwrap_or(false),
        hot_swap_probation_schema: hot_swap_probation.map(|record| record.schema),
        hot_swap_probation_id: hot_swap_probation.map(|record| record.id),
        hot_swap_probation_hash: hot_swap_probation.map(|record| record.probation_hash),
        hot_swap_probation_status: hot_swap_probation.map(|record| record.status),
        hot_swap_probation_previous_version: hot_swap_probation
            .map(|record| record.previous_version),
        hot_swap_probation_new_version: hot_swap_probation.map(|record| record.new_version),
        hot_swap_probation_previous_descriptor_source_hash: hot_swap_probation
            .map(|record| record.previous_descriptor_source_hash),
        hot_swap_probation_new_descriptor_source_hash: hot_swap_probation
            .map(|record| record.new_descriptor_source_hash),
        hot_swap_probation_previous_artifact_identity_hash: hot_swap_probation
            .map(|record| record.previous_artifact_identity_hash),
        hot_swap_probation_new_artifact_identity_hash: hot_swap_probation
            .map(|record| record.new_artifact_identity_hash),
        hot_swap_probation_previous_generation: hot_swap_probation
            .map(|record| record.previous_generation),
        hot_swap_probation_new_generation: hot_swap_probation.map(|record| record.new_generation),
        hot_swap_probation_previous_state_hash: hot_swap_probation
            .map(|record| record.previous_state_hash),
        hot_swap_probation_new_state_hash: hot_swap_probation.map(|record| record.new_state_hash),
        hot_swap_probation_previous_state_counter: hot_swap_probation
            .map(|record| record.previous_state_counter),
        hot_swap_probation_new_state_counter: hot_swap_probation
            .map(|record| record.new_state_counter),
        hot_swap_probation_state_migration_hash: hot_swap_probation
            .map(|record| record.state_migration_hash),
        hot_swap_probation_accepted: hot_swap_probation
            .map(|record| record.accepted)
            .unwrap_or(false),
        hot_swap_probation_writes_persistent_state: hot_swap_probation
            .map(|record| record.writes_persistent_state)
            .unwrap_or(false),
        hot_swap_probation_writes_durable_audit_log: hot_swap_probation
            .map(|record| record.writes_durable_audit_log)
            .unwrap_or(false),
        hot_swap_probation_installs_rollback_plan: hot_swap_probation
            .map(|record| record.installs_rollback_plan)
            .unwrap_or(false),
        hot_swap_probation_applies_rollback: hot_swap_probation
            .map(|record| record.applies_rollback)
            .unwrap_or(false),
        rollback_preview_schema: None,
        rollback_preview_id: None,
        rollback_preview_hash: None,
        rollback_preview_status: None,
        rollback_apply_schema: None,
        rollback_apply_id: None,
        rollback_apply_hash: None,
        rollback_apply_status: None,
        rollback_apply_authorized: false,
        rollback_apply_mutates_service_state: false,
        rollback_transaction_preflight_schema: None,
        rollback_transaction_preflight_id: None,
        rollback_transaction_preflight_hash: None,
        rollback_transaction_preflight_status: None,
        rollback_transaction_authority_missing: false,
        rollback_durable_audit_write_authority_missing: false,
        rollback_persistent_install_authority_missing: false,
        rollback_transaction_writes_durable_audit_log: false,
        rollback_transaction_writes_rollback_store: false,
        rollback_transaction_installs_rollback_plan: false,
        rollback_transaction_applies_rollback: false,
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
    let activation_status = service_slot_activation_status(snapshot);
    let activation_active = service_slot_activation_active(snapshot);
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
    raw("      \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
    raw_line(",");
    raw("      \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw("      \"hot_swap_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw_line("        \"kind\": \"service\",");
    raw("        \"version\": ");
    json_str(service_version(descriptor));
    raw_line(",");
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
    raw("        \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("        \"last_action\": ");
    json_str(snapshot.last_action);
    raw_line(",");
    raw("        \"last_reason\": ");
    json_str(snapshot.last_reason);
    raw_line(",");
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
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
    raw_line(",");
    raw("        \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
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

fn emit_hot_swap_state_migration_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
    migration: HelloStateMigrationRecord,
) {
    raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"reason\": ");
    json_str("state_migration_would_reset_state");
    raw_line(",");
    raw("    \"message\": ");
    json_str("hello hot-swap denied because the candidate would reset RAM-only service state before a rollback-capable migrator exists");
    raw_line(",");
    raw("    \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("    \"target\": ");
    json_str("svc.demo.hello.reset_state");
    raw_line(",");
    raw("    \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("    \"active_descriptor_id\": ");
    json_str(snapshot.load_descriptor.id);
    raw_line(",");
    raw("    \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("    \"state_migration\": ");
    emit_hello_state_migration_option(Some(migration));
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"state_preserving_migrator\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ],");
    raw_line("    \"denied_surfaces\": {");
    raw_line("      \"descriptor_mutation\": \"not_attempted\",");
    raw_line("      \"state_reset\": \"denied\",");
    raw_line("      \"external_artifact_load\": \"denied\",");
    raw_line("      \"candidate_artifact_execution\": \"denied\",");
    raw_line("      \"executable_mapping\": \"denied\",");
    raw_line("      \"persistent_install\": \"denied\",");
    raw_line("      \"durable_audit\": \"denied\",");
    raw_line("      \"rollback_install\": \"denied\",");
    raw_line("      \"broad_mutation\": \"denied\"");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

fn emit_rollback_transaction_preflight(
    snapshot: Snapshot,
    probation: Option<HelloHotSwapProbationRecord>,
) {
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID);
        raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\", \"status\": ");
        json_str(HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS);
        raw(", \"service_id\": ");
        json_str(SERVICE_ID);
        raw(", \"requested_capability\": ");
        json_str(HELLO_ROLLBACK_APPLY_CAPABILITY);
        raw(", \"preflight_hash\": ");
        json_sha256(hello_rollback_transaction_preflight_hash(
            snapshot, probation,
        ));
        raw(", \"rollback_apply_hash\": ");
        json_sha256(hello_rollback_apply_denial_hash(snapshot, probation));
        raw(", \"rollback_preview_hash\": ");
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
        raw(", \"source_probation_hash\": ");
        json_sha256(probation.probation_hash);
        raw(", \"current_state_hash\": ");
        json_sha256(hello_state_hash(snapshot.state_counter));
        raw(", \"current_state_counter\": ");
        raw_fmt(format_args!("{}", snapshot.state_counter));
        raw(", \"rollback_target_descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"rollback_target_descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"rollback_target_artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"rollback_target_artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"rollback_target_generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"rollback_target_state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"rollback_target_state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw(", \"current_candidate_descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"current_candidate_descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"current_candidate_artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"current_candidate_artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"current_candidate_generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"current_candidate_state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"current_candidate_state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw(", \"state_migration_hash\": ");
        json_sha256(probation.state_migration_hash);
        raw(", \"missing_authorities\": {\"rollback_apply_authority\": true, \"rollback_transaction_authority\": true, \"durable_audit_write_authority\": true, \"persistent_install_authority\": true}");
        raw(", \"side_effects\": {\"mutates_service_state\": false, \"applies_rollback\": false, \"writes_persistent_state\": false, \"writes_durable_audit_log\": false, \"writes_rollback_store\": false, \"installs_rollback_plan\": false, \"accepts_external_artifact_bytes\": false, \"loads_candidate_bytes\": false, \"maps_executable_pages\": false, \"provider_auto_load\": false, \"grants_broad_mutation\": false}");
        raw("}");
    } else {
        raw("null");
    }
}

fn emit_rollback_apply_denied(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw("    \"schema\": ");
    json_str(HELLO_ROLLBACK_APPLY_SCHEMA);
    raw_line(",");
    raw("    \"id\": ");
    json_str(HELLO_ROLLBACK_APPLY_ID);
    raw_line(",");
    raw_line("    \"scope\": \"current_boot\",");
    raw_line("    \"classification\": \"local_only\",");
    raw_line("    \"persistence\": \"none\",");
    raw("    \"status\": ");
    json_str(HELLO_ROLLBACK_APPLY_STATUS);
    raw_line(",");
    raw("    \"reason\": ");
    json_str(if probation.is_some() {
        "rollback_apply_authority_missing"
    } else if snapshot.loaded {
        "rollback_preview_or_probation_missing"
    } else {
        "service_not_loaded"
    });
    raw_line(",");
    raw("    \"message\": ");
    json_str("hello rollback apply is denied until rollback apply authority, durable audit, and rollback transaction evidence exist");
    raw_line(",");
    raw("    \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("    \"active_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("    \"active_descriptor_id\": ");
    json_str(snapshot.load_descriptor.id);
    raw_line(",");
    raw("    \"current_state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("    \"source_probation\": ");
    emit_hello_hot_swap_probation_option(probation);
    raw_line(",");
    raw("    \"required_preview\": ");
    if let Some(probation) = probation {
        raw("{\"schema\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_SCHEMA);
        raw(", \"id\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_ID);
        raw(", \"status\": ");
        json_str(HELLO_ROLLBACK_PREVIEW_STATUS);
        raw(", \"preview_hash\": ");
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"rollback_apply_hash\": ");
    if let Some(probation) = probation {
        json_sha256(hello_rollback_apply_denial_hash(snapshot, probation));
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"rollback_transaction_preflight\": ");
    emit_rollback_transaction_preflight(snapshot, probation);
    raw_line(",");
    raw("    \"rollback_target\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.previous_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"current_candidate\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.new_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("    \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"rollback_apply_authority\",");
    raw_line("      \"rollback_transaction_authority\",");
    raw_line("      \"durable_audit_write_authority\",");
    raw_line("      \"persistent_install_authority\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"raios.rollback_transaction.v0\",");
    raw_line("      \"durable_audit_rollback_store\"");
    raw_line("    ],");
    raw_line("    \"denied_surfaces\": {");
    raw_line("      \"mutates_service_state\": false,");
    raw_line("      \"applies_rollback\": false,");
    raw_line("      \"descriptor_mutation\": \"not_attempted\",");
    raw_line("      \"generation_mutation\": \"not_attempted\",");
    raw_line("      \"running_state_mutation\": \"not_attempted\",");
    raw_line("      \"ram_only_state_mutation\": \"not_attempted\",");
    raw_line("      \"persistent_install\": \"denied\",");
    raw_line("      \"durable_audit_write\": \"denied\",");
    raw_line("      \"external_artifact_load\": \"denied\",");
    raw_line("      \"candidate_artifact_execution\": \"denied\",");
    raw_line("      \"executable_mapping\": \"denied\",");
    raw_line("      \"provider_auto_load\": \"denied\",");
    raw_line("      \"broad_mutation\": \"denied\"");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

fn emit_rollback_preview_response(
    method: &'static str,
    snapshot: Snapshot,
    event_id: event_log::EventId,
) {
    let probation = snapshot.hot_swap_probation;
    begin_response(method);
    raw_line("      \"schema\": \"raios.ram_only_hello_service_rollback_preview.v0\",");
    raw("      \"id\": ");
    json_str(HELLO_ROLLBACK_PREVIEW_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw("      \"status\": ");
    json_str(if probation.is_some() {
        HELLO_ROLLBACK_PREVIEW_STATUS
    } else {
        "missing_hot_swap_probation"
    });
    raw_line(",");
    raw("      \"preview_available\": ");
    raw_bool(probation.is_some());
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"current_generation\": ");
    raw_fmt(format_args!("{}", snapshot.generation));
    raw_line(",");
    raw("      \"current_state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"source_probation\": ");
    emit_hello_hot_swap_probation_option(probation);
    raw_line(",");
    raw("      \"preview_hash\": ");
    if let Some(probation) = probation {
        json_sha256(hello_rollback_preview_hash(snapshot, probation));
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"rollback_target\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.previous_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.previous_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.previous_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.previous_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.previous_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.previous_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.previous_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.previous_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"current_candidate\": ");
    if let Some(probation) = probation {
        raw("{\"version\": ");
        json_str(probation.new_version);
        raw(", \"descriptor_id\": ");
        json_str(probation.new_descriptor_id);
        raw(", \"descriptor_source_hash\": ");
        json_sha256(probation.new_descriptor_source_hash);
        raw(", \"artifact_identity_id\": ");
        json_str(probation.new_artifact_identity_id);
        raw(", \"artifact_identity_hash\": ");
        json_sha256(probation.new_artifact_identity_hash);
        raw(", \"generation\": ");
        raw_fmt(format_args!("{}", probation.new_generation));
        raw(", \"state_hash\": ");
        json_sha256(probation.new_state_hash);
        raw(", \"state_counter\": ");
        raw_fmt(format_args!("{}", probation.new_state_counter));
        raw("}");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"mutates_service_state\": false,");
    raw_line("        \"applies_rollback\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit_write\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"provider_auto_load\": \"denied\",");
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
    let activation_status = service_slot_activation_status(snapshot);
    let activation_active = service_slot_activation_active(snapshot);
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
    raw("      \"service_slot_activation\": ");
    emit_service_slot_activation(descriptor, activation_status, activation_active);
    raw_line(",");
    raw("      \"state\": ");
    emit_hello_state(snapshot);
    raw_line(",");
    raw("      \"state_migration\": ");
    emit_hello_state_migration_option(snapshot.state_migration);
    raw_line(",");
    raw("      \"hot_swap_probation\": ");
    emit_hello_hot_swap_probation_option(snapshot.hot_swap_probation);
    raw_line(",");
    raw_line("      \"service\": {");
    raw("        \"id\": ");
    json_str(descriptor.service_id);
    raw_line(",");
    raw("        \"artifact_id\": ");
    json_str(descriptor.artifact_id);
    raw_line(",");
    raw("        \"version\": ");
    json_str(service_version(descriptor));
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
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
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
    raw("        \"state\": ");
    emit_hello_state(snapshot);
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
    raw("        \"hot_swap_event_id\": ");
    json_event_id_option(snapshot.hot_swap_event_id);
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
    raw("        \"service_slot_activation_id\": ");
    json_str(SERVICE_SLOT_ACTIVATION_ID);
    raw_line(",");
    raw("        \"service_slot_activation_hash\": ");
    json_sha256(service_slot_activation_hash(descriptor));
    raw_line(",");
    raw("        \"service_slot_activation_status\": ");
    json_str(activation_status);
    raw_line(",");
    raw("        \"service_slot_activation_active\": ");
    raw_bool(activation_active);
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
    let record = artifact_load_plan_preflight_record(descriptor);
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"preflight_hash\": ");
    json_sha256(record.preflight_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"artifact_id\": ");
    json_str(record.artifact_id);
    raw(", \"load_descriptor_id\": ");
    json_str(record.load_descriptor_id);
    raw(", \"descriptor_source_locator\": ");
    json_str(record.descriptor_source_locator);
    raw(", \"descriptor_source_hash\": ");
    json_sha256(record.descriptor_source_hash);
    raw(", \"artifact_identity_id\": ");
    json_str(record.artifact_identity_id);
    raw(", \"artifact_identity_hash\": ");
    json_sha256(record.artifact_identity_hash);
    raw(", \"artifact_content_binding_hash\": ");
    json_sha256(record.artifact_content_binding_hash);
    raw(", \"artifact_reference_id\": ");
    json_str(record.artifact_reference_id);
    raw(", \"artifact_reference_hash\": ");
    json_sha256(record.artifact_reference_hash);
    raw(", \"artifact_bytes_sha256\": ");
    json_sha256(record.artifact_bytes_sha256);
    raw(", \"service_slot_intent_schema\": ");
    json_str(record.service_slot_intent_schema);
    raw(", \"service_slot_intent_id\": ");
    json_str(record.service_slot_intent_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"authorizes_builtin_current_boot_start\": ");
    raw_bool(record.authorizes_builtin_current_boot_start);
    raw(", \"authorizes_candidate_artifact_execution\": ");
    raw_bool(record.authorizes_candidate_artifact_execution);
    raw(", \"accepts_external_artifact_bytes\": ");
    raw_bool(record.accepts_external_artifact_bytes);
    raw(", \"loads_candidate_bytes\": ");
    raw_bool(record.loads_candidate_bytes);
    raw(", \"maps_executable_pages\": ");
    raw_bool(record.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw(", \"grants_broad_mutation\": ");
    raw_bool(record.grants_broad_mutation);
    raw("}");
}

pub(crate) fn emit_service_slot_activation(
    descriptor: LoadDescriptor,
    status: &'static str,
    active: bool,
) {
    let record = service_slot_activation_record(descriptor, status, active);
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"activation_hash\": ");
    json_sha256(record.activation_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"artifact_id\": ");
    json_str(record.artifact_id);
    raw(", \"load_descriptor_id\": ");
    json_str(record.load_descriptor_id);
    raw(", \"descriptor_source_hash\": ");
    json_sha256(record.descriptor_source_hash);
    raw(", \"artifact_load_plan_preflight_id\": ");
    json_str(record.artifact_load_plan_preflight_id);
    raw(", \"artifact_load_plan_preflight_hash\": ");
    json_sha256(record.artifact_load_plan_preflight_hash);
    raw(", \"artifact_load_plan_preflight_status\": ");
    json_str(record.artifact_load_plan_preflight_status);
    raw(", \"service_slot_intent_id\": ");
    json_str(record.service_slot_intent_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"active\": ");
    raw_bool(record.active);
    raw(", \"accepted_preflight\": ");
    raw_bool(record.accepted_preflight);
    raw(", \"authorizes_builtin_current_boot_start\": ");
    raw_bool(record.authorizes_builtin_current_boot_start);
    raw(", \"authorizes_candidate_artifact_execution\": ");
    raw_bool(record.authorizes_candidate_artifact_execution);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw("}");
}

pub(crate) fn emit_hello_state(snapshot: Snapshot) {
    raw("{");
    raw("\"schema\": ");
    json_str(HELLO_STATE_SCHEMA);
    raw(", \"id\": ");
    json_str(HELLO_STATE_ID);
    raw(", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"persistence\": \"none\"");
    raw(", \"service_id\": ");
    json_str(SERVICE_ID);
    raw(", \"version\": ");
    json_str(service_version(snapshot.load_descriptor));
    raw(", \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw(", \"state_counter\": ");
    raw_fmt(format_args!("{}", snapshot.state_counter));
    raw(", \"state_hash\": ");
    json_sha256(hello_state_hash(snapshot.state_counter));
    raw(", \"loaded\": ");
    raw_bool(snapshot.loaded);
    raw(", \"running\": ");
    raw_bool(snapshot.running);
    raw(", \"writes_persistent_state\": false");
    raw("}");
}

fn emit_hello_state_migration_option(record: Option<HelloStateMigrationRecord>) {
    let Some(record) = record else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"migration_hash\": ");
    json_sha256(record.migration_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"from_version\": ");
    json_str(record.from_version);
    raw(", \"to_version\": ");
    json_str(record.to_version);
    raw(", \"pre_state_hash\": ");
    json_sha256(record.pre_state_hash);
    raw(", \"post_state_hash\": ");
    json_sha256(record.post_state_hash);
    raw(", \"pre_state_counter\": ");
    raw_fmt(format_args!("{}", record.pre_state_counter));
    raw(", \"post_state_counter\": ");
    raw_fmt(format_args!("{}", record.post_state_counter));
    raw(", \"state_preserved\": ");
    raw_bool(record.state_preserved);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw("}");
}

pub(crate) fn emit_hello_hot_swap_probation_option(record: Option<HelloHotSwapProbationRecord>) {
    let Some(record) = record else {
        raw("null");
        return;
    };
    raw("{");
    raw("\"schema\": ");
    json_str(record.schema);
    raw(", \"id\": ");
    json_str(record.id);
    raw(", \"scope\": ");
    json_str(record.scope);
    raw(", \"classification\": ");
    json_str(record.classification);
    raw(", \"persistence\": ");
    json_str(record.persistence);
    raw(", \"status\": ");
    json_str(record.status);
    raw(", \"probation_hash\": ");
    json_sha256(record.probation_hash);
    raw(", \"service_id\": ");
    json_str(record.service_id);
    raw(", \"ram_only_service_slot_id\": ");
    json_str(record.ram_only_service_slot_id);
    raw(", \"previous_version\": ");
    json_str(record.previous_version);
    raw(", \"new_version\": ");
    json_str(record.new_version);
    raw(", \"previous_descriptor_id\": ");
    json_str(record.previous_descriptor_id);
    raw(", \"new_descriptor_id\": ");
    json_str(record.new_descriptor_id);
    raw(", \"previous_descriptor_source_hash\": ");
    json_sha256(record.previous_descriptor_source_hash);
    raw(", \"new_descriptor_source_hash\": ");
    json_sha256(record.new_descriptor_source_hash);
    raw(", \"previous_artifact_identity_id\": ");
    json_str(record.previous_artifact_identity_id);
    raw(", \"new_artifact_identity_id\": ");
    json_str(record.new_artifact_identity_id);
    raw(", \"previous_artifact_identity_hash\": ");
    json_sha256(record.previous_artifact_identity_hash);
    raw(", \"new_artifact_identity_hash\": ");
    json_sha256(record.new_artifact_identity_hash);
    raw(", \"previous_generation\": ");
    raw_fmt(format_args!("{}", record.previous_generation));
    raw(", \"new_generation\": ");
    raw_fmt(format_args!("{}", record.new_generation));
    raw(", \"previous_state_hash\": ");
    json_sha256(record.previous_state_hash);
    raw(", \"new_state_hash\": ");
    json_sha256(record.new_state_hash);
    raw(", \"previous_state_counter\": ");
    raw_fmt(format_args!("{}", record.previous_state_counter));
    raw(", \"new_state_counter\": ");
    raw_fmt(format_args!("{}", record.new_state_counter));
    raw(", \"state_migration_hash\": ");
    json_sha256(record.state_migration_hash);
    raw(", \"accepted\": ");
    raw_bool(record.accepted);
    raw(", \"loads_candidate_bytes\": ");
    raw_bool(record.loads_candidate_bytes);
    raw(", \"maps_executable_pages\": ");
    raw_bool(record.maps_executable_pages);
    raw(", \"writes_persistent_state\": ");
    raw_bool(record.writes_persistent_state);
    raw(", \"writes_durable_audit_log\": ");
    raw_bool(record.writes_durable_audit_log);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(record.installs_rollback_plan);
    raw(", \"applies_rollback\": ");
    raw_bool(record.applies_rollback);
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
