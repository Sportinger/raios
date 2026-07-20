use super::*;

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

pub(crate) fn hello_state_migration_record(
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

pub(crate) fn hello_state_migration_record_hash(record: HelloStateMigrationRecord) -> [u8; 32] {
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

pub(crate) fn hello_hot_swap_probation_record(
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

pub(crate) fn hello_hot_swap_probation_record_hash(
    record: HelloHotSwapProbationRecord,
) -> [u8; 32] {
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
