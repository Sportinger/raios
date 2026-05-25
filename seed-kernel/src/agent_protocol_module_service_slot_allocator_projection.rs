use crate::{agent_protocol_module_types::*, event_log};

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorReadinessProjection {
    pub(crate) readiness_present: bool,
    pub(crate) ready: bool,
    pub(crate) unready_status: &'static str,
    pub(crate) unready_reason: &'static str,
}

pub(crate) fn latest_module_service_slot_allocator_readiness_projection(
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    let Some(retained_service_slot_reservation_event_id) =
        retained_service_slot_reservation_event_id
    else {
        return module_service_slot_allocator_readiness_missing(
            "service_slot_allocator_readiness_missing",
        );
    };

    let allocator_runtime_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0];
    let allocator_runtime = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        allocator_runtime_source.source_fact_locator,
    );
    let Some((allocator_runtime_event_id, allocator_runtime)) = allocator_runtime else {
        return module_service_slot_allocator_runtime_missing(
            allocator_runtime_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_fact_source_available(
        retained_service_slot_reservation_event_id,
        allocator_runtime,
        None,
    ) {
        return module_service_slot_allocator_runtime_missing(
            module_service_slot_allocator_fact_source_reason(
                retained_service_slot_reservation_event_id,
                allocator_runtime,
                None,
            ),
        );
    }

    let registry_binding_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1];
    let registry_binding = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        registry_binding_source.source_fact_locator,
    );
    let Some((registry_binding_event_id, registry_binding)) = registry_binding else {
        return module_service_slot_allocator_readiness_unavailable(
            registry_binding_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_fact_source_available(
        retained_service_slot_reservation_event_id,
        registry_binding,
        Some(allocator_runtime_event_id),
    ) {
        return module_service_slot_allocator_readiness_unavailable(
            module_service_slot_allocator_fact_source_reason(
                retained_service_slot_reservation_event_id,
                registry_binding,
                Some(allocator_runtime_event_id),
            ),
        );
    }

    let health_state_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2];
    let health_state = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        health_state_source.source_fact_locator,
    );
    let Some((health_state_event_id, health_state)) = health_state else {
        return module_service_slot_allocator_readiness_unavailable(
            health_state_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_fact_source_available(
        retained_service_slot_reservation_event_id,
        health_state,
        Some(allocator_runtime_event_id),
    ) {
        return module_service_slot_allocator_readiness_unavailable(
            module_service_slot_allocator_fact_source_reason(
                retained_service_slot_reservation_event_id,
                health_state,
                Some(allocator_runtime_event_id),
            ),
        );
    }

    let unload_cleanup_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3];
    let unload_cleanup = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        unload_cleanup_source.source_fact_locator,
    );
    let Some((unload_cleanup_event_id, unload_cleanup)) = unload_cleanup else {
        return module_service_slot_allocator_readiness_unavailable(
            unload_cleanup_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_fact_source_available(
        retained_service_slot_reservation_event_id,
        unload_cleanup,
        Some(allocator_runtime_event_id),
    ) {
        return module_service_slot_allocator_readiness_unavailable(
            module_service_slot_allocator_fact_source_reason(
                retained_service_slot_reservation_event_id,
                unload_cleanup,
                Some(allocator_runtime_event_id),
            ),
        );
    }

    let durable_audit_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0];
    let durable_audit =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            durable_audit_source.source_fact_locator,
        );
    let Some((_, durable_audit)) = durable_audit else {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_missing_durable_audit_write",
            durable_audit_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_prerequisite_source_available(
        durable_audit,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_missing_durable_audit_write",
            module_service_slot_allocator_prerequisite_source_reason(
                durable_audit,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
        );
    }

    let rollback_install_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1];
    let rollback_install =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            rollback_install_source.source_fact_locator,
        );
    let Some((_, rollback_install)) = rollback_install else {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_missing_rollback_install",
            rollback_install_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_prerequisite_source_available(
        rollback_install,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_missing_rollback_install",
            module_service_slot_allocator_prerequisite_source_reason(
                rollback_install,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
        );
    }

    let module_loader_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2];
    let module_loader =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            module_loader_source.source_fact_locator,
        );
    let Some((_, module_loader)) = module_loader else {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_loader_unimplemented",
            module_loader_source.source_evidence_missing_reason,
        );
    };
    if !module_service_slot_allocator_prerequisite_source_available(
        module_loader,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return module_service_slot_allocator_prerequisite_unavailable(
            "denied_loader_unimplemented",
            module_service_slot_allocator_prerequisite_source_reason(
                module_loader,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
        );
    }

    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: "denied_allocator_authority_unimplemented",
        unready_reason: "service_slot_allocator_authority_unimplemented",
    }
}

fn module_service_slot_allocator_readiness_missing(
    reason: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: false,
        ready: false,
        unready_status: "denied_missing_service_slot_allocator_readiness",
        unready_reason: reason,
    }
}

fn module_service_slot_allocator_runtime_missing(
    reason: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: "denied_missing_service_slot_allocator_runtime",
        unready_reason: reason,
    }
}

fn module_service_slot_allocator_readiness_unavailable(
    reason: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: "denied_missing_service_slot_allocator_readiness",
        unready_reason: reason,
    }
}

fn module_service_slot_allocator_prerequisite_unavailable(
    status: &'static str,
    reason: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: status,
        unready_reason: reason,
    }
}

fn module_service_slot_allocator_fact_source_available(
    retained_service_slot_reservation_event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    expected_allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> bool {
    evidence.fact_present
        && evidence.fact_schema_ok
        && evidence.fact_provenance_ok
        && evidence.binds_retained_service_slot_reservation
        && evidence.binds_allocator_runtime
        && evidence.retained_service_slot_reservation_event_id
            == Some(retained_service_slot_reservation_event_id)
        && evidence.allocator_runtime_source_evidence_event_id
            == expected_allocator_runtime_source_evidence_event_id
}

fn module_service_slot_allocator_fact_source_reason(
    retained_service_slot_reservation_event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    expected_allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> &'static str {
    if evidence.retained_service_slot_reservation_event_id
        != Some(retained_service_slot_reservation_event_id)
    {
        "service_slot_allocator_retained_reservation_binding_mismatch"
    } else if evidence.allocator_runtime_source_evidence_event_id
        != expected_allocator_runtime_source_evidence_event_id
    {
        "service_slot_allocator_runtime_source_evidence_binding_mismatch"
    } else {
        evidence.fact_reason
    }
}

fn module_service_slot_allocator_prerequisite_source_available(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
) -> bool {
    evidence.prerequisite_available
        && evidence.allocator_runtime_source_evidence_event_id == Some(allocator_runtime_event_id)
        && evidence.registry_binding_source_evidence_event_id == Some(registry_binding_event_id)
        && evidence.health_state_source_evidence_event_id == Some(health_state_event_id)
        && evidence.unload_cleanup_source_evidence_event_id == Some(unload_cleanup_event_id)
}

fn module_service_slot_allocator_prerequisite_source_reason(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
) -> &'static str {
    if evidence.allocator_runtime_source_evidence_event_id != Some(allocator_runtime_event_id)
        || evidence.registry_binding_source_evidence_event_id != Some(registry_binding_event_id)
        || evidence.health_state_source_evidence_event_id != Some(health_state_event_id)
        || evidence.unload_cleanup_source_evidence_event_id != Some(unload_cleanup_event_id)
    {
        "service_slot_allocator_prerequisite_source_evidence_binding_mismatch"
    } else {
        evidence.prerequisite_reason
    }
}
