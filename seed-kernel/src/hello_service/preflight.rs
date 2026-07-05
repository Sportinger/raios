use super::*;

pub(crate) fn artifact_load_plan_preflight_record(
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

pub(crate) fn artifact_load_plan_preflight_record_hash(
    record: ArtifactLoadPlanPreflightRecord,
) -> [u8; 32] {
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

pub(crate) fn service_slot_activation_record(
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

pub(crate) fn service_slot_activation_record_hash(record: ServiceSlotActivationRecord) -> [u8; 32] {
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

pub(crate) fn validate_artifact_load_plan_preflight_record(
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

pub(crate) fn artifact_load_plan_preflight_selftest_hash() -> [u8; 32] {
    sha256_bytes(
        b"schema=raios.current_boot_artifact_load_plan_preflight_selftest.v0\n\
id=artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0\n\
cases=valid_preflight,tampered_descriptor_source_hash,tampered_artifact_identity_hash,tampered_content_binding_hash,tampered_artifact_reference_hash,tampered_artifact_bytes_hash,tampered_service_slot_intent,tampered_denial_flags",
    )
}

pub(crate) fn recovery_rollback_inspect_source_reference_selftest_hash() -> [u8; 32] {
    sha256_bytes(
        b"schema=raios.recovery_rollback_inspect_source_reference_selftest.v0\n\
id=recovery_rollback_inspect_source_reference_selftest.current_boot.svc.demo.hello.v0\n\
cases=valid_source_reference,missing_or_unretained_source_event_id,wrong_source_read_binding,missing_or_unretained_audit_event_id,wrong_audit_event_variant,substituted_hashes_denied,authorizing_source_reference_denied",
    )
}

pub(crate) fn artifact_load_plan_preflight_selftest_cases(
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

pub(crate) fn rehash_preflight_record(
    mut record: ArtifactLoadPlanPreflightRecord,
) -> ArtifactLoadPlanPreflightRecord {
    record.preflight_hash = artifact_load_plan_preflight_record_hash(record);
    record
}

pub(crate) fn preflight_case(
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
