use super::*;

pub(crate) enum RollbackApplyResult {
    Denied {
        snapshot: Snapshot,
        event_id: event_log::EventId,
    },
    Applied {
        pre_apply_snapshot: Snapshot,
        snapshot: Snapshot,
        event_id: event_log::EventId,
        proof: ScopedRollbackApplyProof,
    },
}

pub(crate) fn load_start(source_method: &'static str, descriptor: LoadDescriptor) -> Snapshot {
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
    state.applied_rollback = None;
    state.start_event_id = Some(event_id);
    state.last_action = "load_start";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

pub(crate) fn start(source_method: &'static str) -> Snapshot {
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
        state.applied_rollback = None;
        state.start_event_id = Some(event_id);
    }
    state.last_action = "start";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

pub(crate) fn restart(source_method: &'static str) -> Snapshot {
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
        state.applied_rollback = None;
        state.start_event_id = Some(event_id);
    }
    state.last_action = "restart";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

pub(crate) fn hot_swap(source_method: &'static str, descriptor: LoadDescriptor) -> Snapshot {
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
        state.applied_rollback = None;
    }
    state.last_action = "hot_swap";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

pub(crate) fn denied_reset_state_hot_swap(
) -> (Snapshot, event_log::EventId, HelloStateMigrationRecord) {
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

pub(crate) fn stop(source_method: &'static str) -> Snapshot {
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
        state.applied_rollback = None;
        state.stop_event_id = Some(event_id);
    }
    state.last_action = "stop";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    state.snapshot()
}

pub(crate) fn drop_service(source_method: &'static str) -> Snapshot {
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
    state.applied_rollback = None;
    state.drop_event_id = Some(event_id);
    state.last_action = "drop";
    state.last_reason = reason;
    state.last_inventory_change = inventory_change;
    state.last_event_id = Some(event_id);
    let snapshot = state.snapshot();
    state.load_descriptor = LOAD_DESCRIPTOR;
    snapshot
}

pub(crate) fn health_probe(source_method: &'static str) -> (Snapshot, event_log::EventId) {
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

pub(crate) fn rollback_preview(source_method: &'static str) -> (Snapshot, event_log::EventId) {
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

pub(crate) fn rollback_apply(source_method: &'static str) -> RollbackApplyResult {
    let pre_apply_snapshot = {
        let state = STATE.lock();
        state.snapshot()
    };
    let proof = perform_scoped_rollback_apply_proof(source_method, pre_apply_snapshot);
    if proof.apply_decision.applied {
        if let Some(probation) = pre_apply_snapshot.hot_swap_probation {
            return rollback_apply_verified(source_method, pre_apply_snapshot, probation, proof);
        }
    }
    let (snapshot, event_id) = rollback_apply_denied(source_method);
    RollbackApplyResult::Denied { snapshot, event_id }
}

fn rollback_apply_denied(source_method: &'static str) -> (Snapshot, event_log::EventId) {
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
    bind_rollback_write_authority_gate(&mut binding, snapshot);
    bind_rollback_append_intent_gate(&mut binding, snapshot);
    bind_rollback_payload_envelope_gate(&mut binding, snapshot);
    bind_rollback_transaction_writer_storage_authority_gate(&mut binding, snapshot);
    let event_id = event_log::record_hello_service_rollback_apply(source_method, reason, binding);
    (snapshot, event_id)
}

fn rollback_apply_verified(
    source_method: &'static str,
    pre_apply_snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
    proof: ScopedRollbackApplyProof,
) -> RollbackApplyResult {
    let Some((target_region_write_readback, target_region_sector_inspection)) =
        recovery_rollback_inspection_evidence(pre_apply_snapshot)
    else {
        let (snapshot, event_id) = rollback_apply_denied(source_method);
        return RollbackApplyResult::Denied { snapshot, event_id };
    };
    let target_descriptor = rollback_target_descriptor(pre_apply_snapshot, probation);
    let mut binding = lifecycle_binding(
        target_descriptor,
        "updated_current_boot_service",
        SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS,
        true,
        probation.previous_state_counter,
        pre_apply_snapshot.state_migration,
        pre_apply_snapshot.hot_swap_probation,
    );
    bind_rollback_preview(&mut binding, pre_apply_snapshot);
    bind_rollback_transaction_writer_storage_authority_gate(&mut binding, pre_apply_snapshot);
    bind_rollback_apply_applied(&mut binding, proof);
    let event_id = event_log::record_hello_service_lifecycle(
        source_method,
        "response",
        proof.apply_decision.reason,
        binding,
    );
    let Some(applied_rollback) = applied_rollback_record(
        event_id,
        proof,
        target_region_write_readback,
        target_region_sector_inspection,
    ) else {
        let (snapshot, event_id) = rollback_apply_denied(source_method);
        return RollbackApplyResult::Denied { snapshot, event_id };
    };

    let mut state = STATE.lock();
    state.loaded = true;
    state.running = true;
    state.generation = probation.previous_generation;
    state.state_counter = probation.previous_state_counter;
    state.load_descriptor = target_descriptor;
    state.state_migration = None;
    state.hot_swap_probation = None;
    state.applied_rollback = Some(applied_rollback);
    state.load_event_id = Some(event_id);
    state.start_event_id = Some(event_id);
    state.last_action = "rollback_apply";
    state.last_reason = proof.apply_decision.reason;
    state.last_inventory_change = "updated_current_boot_service";
    state.last_event_id = Some(event_id);
    let snapshot = state.snapshot();

    RollbackApplyResult::Applied {
        pre_apply_snapshot,
        snapshot,
        event_id,
        proof,
    }
}

fn applied_rollback_record(
    event_id: event_log::EventId,
    proof: ScopedRollbackApplyProof,
    target_region_write_readback: RollbackTargetRegionWriteReadbackDryRun,
    target_region_sector_inspection: RollbackTargetRegionSectorInspection,
) -> Option<AppliedRollbackRecord> {
    let rollback_transaction_hash = proof
        .append_input
        .append_record_rollback_transaction_image_hash?;
    let write_readback_hash = proof.append_input.write_readback_hash?;
    let inspection_hash = proof.append_input.inspection_hash?;
    let audit_record_hash = proof.append_input.append_record_audit_record_image_hash?;
    let inspected_rollback_transaction_hash = proof
        .append_input
        .inspected_rollback_transaction_image_hash?;
    if !raios_core::scoped_rollback_apply::applied_rollback_inspect_evidence_retained(
        Some(rollback_transaction_hash),
        Some(write_readback_hash),
        Some(inspection_hash),
        target_region_sector_inspection.rollback_transaction_image_hash,
        target_region_sector_inspection.source_target_region_write_readback_hash,
        target_region_sector_inspection.inspection_hash,
    ) {
        return None;
    }
    if inspected_rollback_transaction_hash
        != target_region_sector_inspection.rollback_transaction_image_hash
        || audit_record_hash != target_region_sector_inspection.audit_record_image_hash
    {
        return None;
    }
    Some(AppliedRollbackRecord {
        event_id,
        audit_event_id: event_id,
        rollback_transaction_hash,
        write_readback_hash,
        inspection_hash,
        audit_record_hash,
        inspected_rollback_transaction_hash,
        authorized_append_hash: proof.append_hash,
        scope_decision_hash: proof.scope_decision_hash,
        target_region_write_readback,
        target_region_sector_inspection,
    })
}

fn rollback_target_descriptor(
    snapshot: Snapshot,
    probation: HelloHotSwapProbationRecord,
) -> LoadDescriptor {
    let artifact_identity = if probation.previous_artifact_identity_id
        == HELLO_SERVICE_DESCRIPTOR.replacement_artifact_identity_id
    {
        descriptor_sources::hello_builtin_artifact_identity_v2()
    } else {
        descriptor_sources::hello_builtin_artifact_identity()
    };
    LoadDescriptor {
        artifact_identity,
        ..snapshot.load_descriptor
    }
}
