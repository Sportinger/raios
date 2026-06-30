use crate::{agent_protocol_module_types::*, event_log};

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAuthorityInputProjection {
    pub(crate) schema: &'static str,
    pub(crate) name: &'static str,
    pub(crate) present: bool,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorReadinessProjection {
    pub(crate) readiness_present: bool,
    pub(crate) ready: bool,
    pub(crate) unready_status: &'static str,
    pub(crate) unready_reason: &'static str,
    pub(crate) authority_present: bool,
    pub(crate) authority_status: &'static str,
    pub(crate) authority_reason: &'static str,
    pub(crate) authority_source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) allocation_intent_present: bool,
    pub(crate) allocation_intent_status: &'static str,
    pub(crate) allocation_intent_reason: &'static str,
    pub(crate) allocation_intent_source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) authority_inputs:
        [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    pub(crate) authority_decision_present: bool,
    pub(crate) authority_decision_status: &'static str,
    pub(crate) authority_decision_reason: &'static str,
    pub(crate) authority_decision_source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) registry_write_commit_gate_present: bool,
    pub(crate) registry_write_commit_gate_status: &'static str,
    pub(crate) registry_write_commit_gate_reason: &'static str,
    pub(crate) registry_write_commit_gate_source_evidence_event_id: Option<event_log::EventId>,
}

fn module_service_slot_authority_input_projection_missing(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInputProjection {
    ModuleServiceSlotAuthorityInputProjection {
        schema: source.schema,
        name: source.name,
        present: false,
        status: source.missing_status,
        reason: source.source_evidence_missing_reason,
        source_evidence_event_id: None,
    }
}

fn module_service_slot_authority_input_projections_missing(
) -> [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_service_slot_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        ),
        module_service_slot_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
        ),
        module_service_slot_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
        ),
        module_service_slot_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
        ),
        module_service_slot_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
        ),
    ]
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
    let Some((durable_audit_event_id, durable_audit)) = durable_audit else {
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
    let Some((rollback_install_event_id, rollback_install)) = rollback_install else {
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
    let Some((module_loader_event_id, module_loader)) = module_loader else {
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

    let authority = event_log::latest_module_service_slot_allocator_authority_source_evidence();
    let Some((authority_event_id, authority)) = authority else {
        return module_service_slot_allocator_authority_unavailable(
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            None,
            "missing",
        );
    };
    if !module_service_slot_allocator_authority_source_available(
        authority,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
        durable_audit_event_id,
        rollback_install_event_id,
        module_loader_event_id,
    ) {
        return module_service_slot_allocator_authority_unavailable(
            module_service_slot_allocator_authority_source_reason(
                authority,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
                durable_audit_event_id,
                rollback_install_event_id,
                module_loader_event_id,
            ),
            Some(authority_event_id),
            authority.authority_status,
        );
    }

    let allocation_intent =
        event_log::latest_module_service_slot_allocation_intent_source_evidence();
    let Some((allocation_intent_event_id, allocation_intent)) = allocation_intent else {
        return module_service_slot_allocator_allocation_intent_unavailable(
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            None,
            authority_event_id,
            authority.authority_status,
            authority.authority_reason,
        );
    };
    if !module_service_slot_allocation_intent_source_available(
        allocation_intent,
        retained_service_slot_reservation_event_id,
        authority_event_id,
    ) {
        return module_service_slot_allocator_allocation_intent_unavailable(
            module_service_slot_allocation_intent_source_reason(
                allocation_intent,
                retained_service_slot_reservation_event_id,
                authority_event_id,
            ),
            Some(allocation_intent_event_id),
            authority_event_id,
            authority.authority_status,
            authority.authority_reason,
        );
    }

    let authority_inputs = module_service_slot_authority_input_projections(
        allocation_intent_event_id,
        allocation_intent,
    );
    if !module_service_slot_authority_input_projections_complete(authority_inputs) {
        return ModuleServiceSlotAllocatorReadinessProjection {
            readiness_present: true,
            ready: false,
            unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_present: true,
            authority_status: authority.authority_status,
            authority_reason: authority.authority_reason,
            authority_source_evidence_event_id: Some(authority_event_id),
            allocation_intent_present: true,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
            allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
            allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
            authority_inputs,
            authority_decision_present: false,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON,
            authority_decision_source_evidence_event_id: None,
            registry_write_commit_gate_present: false,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
            registry_write_commit_gate_source_evidence_event_id: None,
        };
    }

    let authority_decision =
        event_log::latest_module_service_slot_allocator_authority_decision_source_evidence();
    let Some((authority_decision_event_id, authority_decision)) = authority_decision else {
        return ModuleServiceSlotAllocatorReadinessProjection {
            readiness_present: true,
            ready: false,
            unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_present: true,
            authority_status: authority.authority_status,
            authority_reason: authority.authority_reason,
            authority_source_evidence_event_id: Some(authority_event_id),
            allocation_intent_present: true,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
            allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
            allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
            authority_inputs,
            authority_decision_present: false,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_source_evidence_event_id: None,
            registry_write_commit_gate_present: false,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_source_evidence_event_id: None,
        };
    };
    if !module_service_slot_allocator_authority_decision_source_available(
        authority_decision,
        authority_event_id,
        allocation_intent_event_id,
        authority_inputs,
    ) {
        return ModuleServiceSlotAllocatorReadinessProjection {
            readiness_present: true,
            ready: false,
            unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_present: true,
            authority_status: authority.authority_status,
            authority_reason: authority.authority_reason,
            authority_source_evidence_event_id: Some(authority_event_id),
            allocation_intent_present: true,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
            allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
            allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
            authority_inputs,
            authority_decision_present: false,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                module_service_slot_allocator_authority_decision_source_reason(
                    authority_decision,
                    authority_event_id,
                    allocation_intent_event_id,
                    authority_inputs,
                ),
            authority_decision_source_evidence_event_id: Some(authority_decision_event_id),
            registry_write_commit_gate_present: false,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
            registry_write_commit_gate_source_evidence_event_id: None,
        };
    }

    let registry_write_commit_gate =
        event_log::latest_module_service_slot_registry_write_commit_gate_source_evidence();
    let Some((registry_write_commit_gate_event_id, registry_write_commit_gate)) =
        registry_write_commit_gate
    else {
        return ModuleServiceSlotAllocatorReadinessProjection {
            readiness_present: true,
            ready: false,
            unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_present: true,
            authority_status: "defined_non_authorizing",
            authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_source_evidence_event_id: Some(authority_event_id),
            allocation_intent_present: true,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
            allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
            allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
            authority_inputs,
            authority_decision_present: true,
            authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS,
            authority_decision_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON,
            authority_decision_source_evidence_event_id: Some(authority_decision_event_id),
            registry_write_commit_gate_present: false,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_source_evidence_event_id: None,
        };
    };
    let registry_write_authority_event_id = authority_inputs[1].source_evidence_event_id;
    if !module_service_slot_registry_write_commit_gate_source_available(
        registry_write_commit_gate,
        authority_decision_event_id,
        registry_write_authority_event_id,
        registry_binding_event_id,
        durable_audit_event_id,
        rollback_install_event_id,
        retained_service_slot_reservation_event_id,
    ) {
        return ModuleServiceSlotAllocatorReadinessProjection {
            readiness_present: true,
            ready: false,
            unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_present: true,
            authority_status: "defined_non_authorizing",
            authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            authority_source_evidence_event_id: Some(authority_event_id),
            allocation_intent_present: true,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
            allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
            allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
            authority_inputs,
            authority_decision_present: true,
            authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS,
            authority_decision_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON,
            authority_decision_source_evidence_event_id: Some(authority_decision_event_id),
            registry_write_commit_gate_present: false,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                module_service_slot_registry_write_commit_gate_source_reason(
                    registry_write_commit_gate,
                    authority_decision_event_id,
                    registry_write_authority_event_id,
                    registry_binding_event_id,
                    durable_audit_event_id,
                    rollback_install_event_id,
                    retained_service_slot_reservation_event_id,
                ),
            registry_write_commit_gate_source_evidence_event_id: Some(
                registry_write_commit_gate_event_id,
            ),
        };
    }

    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
        unready_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        authority_present: true,
        authority_status: "defined_non_authorizing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        authority_source_evidence_event_id: Some(authority_event_id),
        allocation_intent_present: true,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
        allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
        allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
        authority_inputs,
        authority_decision_present: true,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS,
        authority_decision_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON,
        authority_decision_source_evidence_event_id: Some(authority_decision_event_id),
        registry_write_commit_gate_present: true,
        registry_write_commit_gate_status: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS,
        registry_write_commit_gate_reason: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON,
        registry_write_commit_gate_source_evidence_event_id: Some(
            registry_write_commit_gate_event_id,
        ),
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
        authority_present: false,
        authority_status: "missing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        authority_source_evidence_event_id: None,
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
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
        authority_present: false,
        authority_status: "missing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        authority_source_evidence_event_id: None,
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
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
        authority_present: false,
        authority_status: "missing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        authority_source_evidence_event_id: None,
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
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
        authority_present: false,
        authority_status: "missing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        authority_source_evidence_event_id: None,
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
    }
}

fn module_service_slot_allocator_authority_unavailable(
    reason: &'static str,
    authority_source_evidence_event_id: Option<event_log::EventId>,
    authority_status: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS,
        unready_reason: reason,
        authority_present: false,
        authority_status,
        authority_reason: reason,
        authority_source_evidence_event_id,
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
    }
}

fn module_service_slot_allocator_allocation_intent_unavailable(
    reason: &'static str,
    allocation_intent_source_evidence_event_id: Option<event_log::EventId>,
    authority_source_evidence_event_id: event_log::EventId,
    authority_status: &'static str,
    authority_reason: &'static str,
) -> ModuleServiceSlotAllocatorReadinessProjection {
    ModuleServiceSlotAllocatorReadinessProjection {
        readiness_present: true,
        ready: false,
        unready_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        unready_reason: reason,
        authority_present: true,
        authority_status,
        authority_reason,
        authority_source_evidence_event_id: Some(authority_source_evidence_event_id),
        allocation_intent_present: false,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason: reason,
        allocation_intent_source_evidence_event_id,
        authority_inputs: module_service_slot_authority_input_projections_missing(),
        authority_decision_present: false,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_source_evidence_event_id: None,
        registry_write_commit_gate_present: false,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_source_evidence_event_id: None,
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

fn module_service_slot_allocator_authority_source_available(
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
    durable_audit_event_id: event_log::EventId,
    rollback_install_event_id: event_log::EventId,
    module_loader_event_id: event_log::EventId,
) -> bool {
    evidence.authority_present
        && evidence.authority_schema_ok
        && evidence.authority_provenance_ok
        && evidence.source_chain_complete
        && evidence.allocator_runtime_source_evidence_event_id == Some(allocator_runtime_event_id)
        && evidence.registry_binding_source_evidence_event_id == Some(registry_binding_event_id)
        && evidence.health_state_source_evidence_event_id == Some(health_state_event_id)
        && evidence.unload_cleanup_source_evidence_event_id == Some(unload_cleanup_event_id)
        && evidence.durable_audit_source_evidence_event_id == Some(durable_audit_event_id)
        && evidence.rollback_install_source_evidence_event_id == Some(rollback_install_event_id)
        && evidence.module_loader_source_evidence_event_id == Some(module_loader_event_id)
}

fn module_service_slot_allocator_authority_source_reason(
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
    durable_audit_event_id: event_log::EventId,
    rollback_install_event_id: event_log::EventId,
    module_loader_event_id: event_log::EventId,
) -> &'static str {
    if evidence.allocator_runtime_source_evidence_event_id != Some(allocator_runtime_event_id)
        || evidence.registry_binding_source_evidence_event_id != Some(registry_binding_event_id)
        || evidence.health_state_source_evidence_event_id != Some(health_state_event_id)
        || evidence.unload_cleanup_source_evidence_event_id != Some(unload_cleanup_event_id)
        || evidence.durable_audit_source_evidence_event_id != Some(durable_audit_event_id)
        || evidence.rollback_install_source_evidence_event_id != Some(rollback_install_event_id)
        || evidence.module_loader_source_evidence_event_id != Some(module_loader_event_id)
    {
        "service_slot_allocator_authority_source_evidence_binding_mismatch"
    } else if !evidence.source_chain_complete {
        "service_slot_allocator_authority_source_chain_incomplete"
    } else {
        evidence.authority_reason
    }
}

fn module_service_slot_allocation_intent_source_available(
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    retained_service_slot_reservation_event_id: event_log::EventId,
    authority_source_evidence_event_id: event_log::EventId,
) -> bool {
    evidence.intent_present
        && evidence.intent_schema_ok
        && evidence.intent_provenance_ok
        && evidence.source_chain_complete
        && evidence.retained_module_evidence_present
        && evidence.retained_service_slot_reservation_present
        && evidence.allocator_authority_present
        && evidence.intent_scope == "current_boot"
        && evidence.intent_classification == "local_only"
        && evidence.requested_capability == "cap.module.load_ephemeral"
        && evidence.load_mode == "ram_only"
        && evidence.target == "live_service_graph"
        && evidence.service_slot_reservation_event_id
            == Some(retained_service_slot_reservation_event_id)
        && evidence.allocator_authority_source_evidence_event_id
            == Some(authority_source_evidence_event_id)
}

fn module_service_slot_allocation_intent_source_reason(
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    retained_service_slot_reservation_event_id: event_log::EventId,
    authority_source_evidence_event_id: event_log::EventId,
) -> &'static str {
    if evidence.service_slot_reservation_event_id
        != Some(retained_service_slot_reservation_event_id)
        || evidence.allocator_authority_source_evidence_event_id
            != Some(authority_source_evidence_event_id)
    {
        "service_slot_allocation_intent_source_evidence_binding_mismatch"
    } else if !evidence.source_chain_complete {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
    } else if evidence.intent_scope != "current_boot" {
        "service_slot_allocation_intent_scope_must_be_current_boot"
    } else if !evidence.intent_schema_ok {
        "service_slot_allocation_intent_schema_mismatch"
    } else if !evidence.intent_provenance_ok {
        "service_slot_allocation_intent_provenance_missing"
    } else {
        evidence.intent_reason
    }
}

fn module_service_slot_authority_input_projections(
    allocation_intent_event_id: event_log::EventId,
    allocation_intent: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
) -> [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    let policy_decision = module_service_slot_authority_input_projection(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
        Some(allocation_intent_event_id),
        allocation_intent.intent_present && allocation_intent.source_chain_complete,
    );
    let registry_write = module_service_slot_authority_input_projection(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].schema,
        policy_decision.source_evidence_event_id,
        policy_decision.present,
    );
    let loader_contract = module_service_slot_authority_input_projection(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1].schema,
        registry_write.source_evidence_event_id,
        registry_write.present,
    );
    let health_monitor = module_service_slot_authority_input_projection(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2].schema,
        loader_contract.source_evidence_event_id,
        loader_contract.present,
    );
    let unload_cleanup = module_service_slot_authority_input_projection(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3].schema,
        health_monitor.source_evidence_event_id,
        health_monitor.present,
    );
    [
        policy_decision,
        registry_write,
        loader_contract,
        health_monitor,
        unload_cleanup,
    ]
}

fn module_service_slot_authority_input_projection(
    source: ModuleServiceSlotAuthorityInputSpec,
    dependency_schema: &'static str,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
    dependency_present: bool,
) -> ModuleServiceSlotAuthorityInputProjection {
    let latest = event_log::latest_module_service_slot_authority_input_source_evidence(
        source.source_fact_locator,
    );
    let Some((event_id, evidence)) = latest else {
        return module_service_slot_authority_input_projection_missing(source);
    };
    if module_service_slot_authority_input_source_available(
        source,
        evidence,
        dependency_schema,
        dependency_source_evidence_event_id,
        dependency_present,
    ) {
        return ModuleServiceSlotAuthorityInputProjection {
            schema: source.schema,
            name: source.name,
            present: true,
            status: source.available_status,
            reason: source.available_reason,
            source_evidence_event_id: Some(event_id),
        };
    }
    ModuleServiceSlotAuthorityInputProjection {
        schema: source.schema,
        name: source.name,
        present: false,
        status: source.missing_status,
        reason: module_service_slot_authority_input_source_reason(
            source,
            evidence,
            dependency_schema,
            dependency_source_evidence_event_id,
            dependency_present,
        ),
        source_evidence_event_id: Some(event_id),
    }
}

fn module_service_slot_authority_input_source_available(
    source: ModuleServiceSlotAuthorityInputSpec,
    evidence: event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    dependency_schema: &'static str,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
    dependency_present: bool,
) -> bool {
    evidence.input_present
        && evidence.input_schema_ok
        && evidence.input_provenance_ok
        && evidence.source_chain_complete
        && evidence.input_scope == "current_boot"
        && evidence.input_classification == "local_only"
        && evidence.input_schema == source.schema
        && evidence.input_id == source.id
        && evidence.dependency_schema == dependency_schema
        && evidence.dependency_source_evidence_event_id == dependency_source_evidence_event_id
        && evidence.dependency_present == dependency_present
        && evidence.requested_capability == "cap.module.load_ephemeral"
        && evidence.load_mode == "ram_only"
        && evidence.target == "live_service_graph"
}

fn module_service_slot_authority_input_source_reason(
    source: ModuleServiceSlotAuthorityInputSpec,
    evidence: event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    dependency_schema: &'static str,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
    dependency_present: bool,
) -> &'static str {
    if evidence.input_schema != source.schema || evidence.input_id != source.id {
        "service_slot_authority_input_identity_mismatch"
    } else if evidence.dependency_schema != dependency_schema
        || evidence.dependency_source_evidence_event_id != dependency_source_evidence_event_id
        || evidence.dependency_present != dependency_present
    {
        "service_slot_authority_input_dependency_binding_mismatch"
    } else if !evidence.source_chain_complete {
        source.source_chain_incomplete_reason
    } else if evidence.input_scope != "current_boot" {
        "service_slot_authority_input_scope_must_be_current_boot"
    } else if !evidence.input_schema_ok {
        "service_slot_authority_input_schema_mismatch"
    } else if !evidence.input_provenance_ok {
        "service_slot_authority_input_provenance_missing"
    } else {
        evidence.input_reason
    }
}

fn module_service_slot_authority_input_projections_complete(
    inputs: [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> bool {
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        if !inputs[idx].present {
            return false;
        }
        idx += 1;
    }
    true
}

fn module_service_slot_allocator_authority_decision_source_available(
    evidence: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    allocator_authority_source_evidence_event_id: event_log::EventId,
    allocation_intent_source_evidence_event_id: event_log::EventId,
    authority_inputs: [ModuleServiceSlotAuthorityInputProjection;
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> bool {
    evidence.decision_present
        && evidence.decision_schema_ok
        && evidence.decision_provenance_ok
        && evidence.source_chain_complete
        && evidence.authority_inputs_complete
        && evidence.decision_scope == "current_boot"
        && evidence.decision_classification == "local_only"
        && evidence.decision_schema == MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA
        && evidence.decision_id == MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID
        && evidence.allocator_authority_source_evidence_event_id
            == Some(allocator_authority_source_evidence_event_id)
        && evidence.allocation_intent_source_evidence_event_id
            == Some(allocation_intent_source_evidence_event_id)
        && module_service_slot_allocator_authority_decision_inputs_bound(evidence, authority_inputs)
        && evidence.requested_capability == "cap.module.load_ephemeral"
        && evidence.load_mode == "ram_only"
        && evidence.target == "live_service_graph"
}

fn module_service_slot_allocator_authority_decision_source_reason(
    evidence: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    allocator_authority_source_evidence_event_id: event_log::EventId,
    allocation_intent_source_evidence_event_id: event_log::EventId,
    authority_inputs: [ModuleServiceSlotAuthorityInputProjection;
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> &'static str {
    if evidence.decision_schema != MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA
        || evidence.decision_id != MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID
    {
        "service_slot_allocator_authority_decision_identity_mismatch"
    } else if evidence.allocator_authority_source_evidence_event_id
        != Some(allocator_authority_source_evidence_event_id)
        || evidence.allocation_intent_source_evidence_event_id
            != Some(allocation_intent_source_evidence_event_id)
        || !module_service_slot_allocator_authority_decision_inputs_bound(
            evidence,
            authority_inputs,
        )
    {
        "service_slot_allocator_authority_decision_source_binding_mismatch"
    } else if !evidence.source_chain_complete || !evidence.authority_inputs_complete {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON
    } else if evidence.decision_scope != "current_boot" {
        "service_slot_allocator_authority_decision_scope_must_be_current_boot"
    } else if !evidence.decision_schema_ok {
        "service_slot_allocator_authority_decision_schema_mismatch"
    } else if !evidence.decision_provenance_ok {
        "service_slot_allocator_authority_decision_provenance_missing"
    } else {
        evidence.decision_reason
    }
}

fn module_service_slot_allocator_authority_decision_inputs_bound(
    evidence: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    authority_inputs: [ModuleServiceSlotAuthorityInputProjection;
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> bool {
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        if evidence.authority_input_source_evidence_event_ids[idx]
            != authority_inputs[idx].source_evidence_event_id
            || evidence.authority_input_present[idx] != authority_inputs[idx].present
            || !authority_inputs[idx].present
        {
            return false;
        }
        idx += 1;
    }
    true
}

fn module_service_slot_registry_write_commit_gate_source_available(
    evidence: event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    authority_decision_source_evidence_event_id: event_log::EventId,
    registry_write_authority_source_evidence_event_id: Option<event_log::EventId>,
    registry_binding_source_evidence_event_id: event_log::EventId,
    durable_audit_source_evidence_event_id: event_log::EventId,
    rollback_install_source_evidence_event_id: event_log::EventId,
    retained_service_slot_reservation_event_id: event_log::EventId,
) -> bool {
    evidence.gate_present
        && evidence.gate_schema_ok
        && evidence.gate_provenance_ok
        && evidence.source_chain_complete
        && evidence.gate_scope == "current_boot"
        && evidence.gate_classification == "local_only"
        && evidence.gate_schema == MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA
        && evidence.gate_id == MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID
        && evidence.authority_decision_present
        && evidence.registry_write_authority_present
        && evidence.registry_binding_available
        && evidence.durable_audit_write_available
        && evidence.rollback_plan_install_available
        && evidence.retained_service_slot_reservation_present
        && evidence.authority_decision_source_evidence_event_id
            == Some(authority_decision_source_evidence_event_id)
        && evidence.registry_write_authority_source_evidence_event_id
            == registry_write_authority_source_evidence_event_id
        && evidence.registry_binding_source_evidence_event_id
            == Some(registry_binding_source_evidence_event_id)
        && evidence.durable_audit_source_evidence_event_id
            == Some(durable_audit_source_evidence_event_id)
        && evidence.rollback_install_source_evidence_event_id
            == Some(rollback_install_source_evidence_event_id)
        && evidence.service_slot_reservation_event_id
            == Some(retained_service_slot_reservation_event_id)
        && evidence.requested_capability == "cap.module.load_ephemeral"
        && evidence.load_mode == "ram_only"
        && evidence.target == "live_service_graph"
        && !evidence.authorizes_registry_write
        && !evidence.mutates_service_registry
        && !evidence.writes_durable_audit_state
        && !evidence.installs_rollback_state
        && !evidence.allocates_service_slot
        && !evidence.loads_artifact
}

fn module_service_slot_registry_write_commit_gate_source_reason(
    evidence: event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    authority_decision_source_evidence_event_id: event_log::EventId,
    registry_write_authority_source_evidence_event_id: Option<event_log::EventId>,
    registry_binding_source_evidence_event_id: event_log::EventId,
    durable_audit_source_evidence_event_id: event_log::EventId,
    rollback_install_source_evidence_event_id: event_log::EventId,
    retained_service_slot_reservation_event_id: event_log::EventId,
) -> &'static str {
    if evidence.gate_schema != MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA
        || evidence.gate_id != MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID
    {
        "service_slot_registry_write_commit_gate_identity_mismatch"
    } else if evidence.authority_decision_source_evidence_event_id
        != Some(authority_decision_source_evidence_event_id)
        || evidence.registry_write_authority_source_evidence_event_id
            != registry_write_authority_source_evidence_event_id
        || evidence.registry_binding_source_evidence_event_id
            != Some(registry_binding_source_evidence_event_id)
        || evidence.durable_audit_source_evidence_event_id
            != Some(durable_audit_source_evidence_event_id)
        || evidence.rollback_install_source_evidence_event_id
            != Some(rollback_install_source_evidence_event_id)
        || evidence.service_slot_reservation_event_id
            != Some(retained_service_slot_reservation_event_id)
    {
        "service_slot_registry_write_commit_gate_source_binding_mismatch"
    } else if !evidence.source_chain_complete
        || !evidence.authority_decision_present
        || !evidence.registry_write_authority_present
        || !evidence.registry_binding_available
        || !evidence.durable_audit_write_available
        || !evidence.rollback_plan_install_available
        || !evidence.retained_service_slot_reservation_present
    {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
    } else if evidence.gate_scope != "current_boot" {
        "service_slot_registry_write_commit_gate_scope_must_be_current_boot"
    } else if !evidence.gate_schema_ok {
        "service_slot_registry_write_commit_gate_schema_mismatch"
    } else if !evidence.gate_provenance_ok {
        "service_slot_registry_write_commit_gate_provenance_missing"
    } else if evidence.authorizes_registry_write
        || evidence.mutates_service_registry
        || evidence.writes_durable_audit_state
        || evidence.installs_rollback_state
        || evidence.allocates_service_slot
        || evidence.loads_artifact
    {
        "service_slot_registry_write_commit_gate_must_be_non_authorizing"
    } else {
        evidence.gate_reason
    }
}
