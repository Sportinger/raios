use crate::event_log;

#[derive(Clone, Copy)]
pub(crate) struct ModuleManifestReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) expected_manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleManifestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleArtifactReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleArtifactReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleArtifactSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleVmReportReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) report_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleVmReportReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) report_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_report_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleVmReportSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalAttestationReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalAttestationReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleLocalAttestationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalApprovalReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_attestation_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalApprovalReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_attestation_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) expected_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleLocalApprovalSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleGrantReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleGrantSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) audit_schema_ok: bool,
    pub(crate) rollback_schema_ok: bool,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) cleanup_actions_hash: Option<[u8; 32]>,
    pub(crate) denial_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) cleanup_actions_hash: Option<[u8; 32]>,
    pub(crate) denial_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) expected_rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) expected_audit_record_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleAuditRollbackSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotReservationInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) reservation_hash: Option<[u8; 32]>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) retained_audit_rollback_reference_event_id: Option<&'a str>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotReservationCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) reservation_hash: Option<[u8; 32]>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) retained_audit_rollback_reference_event_id: Option<&'a str>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
    pub(crate) expected_reservation_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleServiceSlotSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_reservation: bool,
    pub(crate) binds_allocator_runtime: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorFactSource {
    pub(crate) name: &'static str,
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) missing_reason: &'static str,
    pub(crate) source_method: &'static str,
    pub(crate) source_fact_locator: &'static str,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_missing_reason: &'static str,
}

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCE_COUNT: usize = 4;

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES:
    [ModuleServiceSlotAllocatorFactSource; MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCE_COUNT] = [
    ModuleServiceSlotAllocatorFactSource {
        name: "service_slot_allocator_runtime",
        schema: "raios.ram_only_service_slot_allocator.v0",
        id: "module.service_slot_allocator.runtime.current_boot",
        missing_reason: "service_slot_allocator_runtime_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.service_slot_allocator_runtime",
        source_evidence_schema: "raios.service_slot_allocator_runtime_source_evidence.v0",
        source_evidence_missing_reason: "service_slot_allocator_runtime_source_evidence_missing",
    },
    ModuleServiceSlotAllocatorFactSource {
        name: "service_slot_registry_binding",
        schema: "raios.service_slot_registry_binding.v0",
        id: "module.service_slot_registry.binding.current_boot",
        missing_reason: "service_slot_registry_binding_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.service_slot_registry_binding",
        source_evidence_schema: "raios.service_slot_registry_binding_source_evidence.v0",
        source_evidence_missing_reason: "service_slot_registry_binding_source_evidence_missing",
    },
    ModuleServiceSlotAllocatorFactSource {
        name: "service_health_state_model",
        schema: "raios.service_health_state_model.v0",
        id: "module.service_health_state.model.current_boot",
        missing_reason: "service_health_state_model_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.service_health_state_model",
        source_evidence_schema: "raios.service_health_state_model_source_evidence.v0",
        source_evidence_missing_reason: "service_health_state_model_source_evidence_missing",
    },
    ModuleServiceSlotAllocatorFactSource {
        name: "service_unload_cleanup_plan",
        schema: "raios.service_unload_cleanup_plan.v0",
        id: "module.service_unload.cleanup.current_boot",
        missing_reason: "service_unload_cleanup_plan_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.service_unload_cleanup_plan",
        source_evidence_schema: "raios.service_unload_cleanup_plan_source_evidence.v0",
        source_evidence_missing_reason: "service_unload_cleanup_plan_source_evidence_missing",
    },
];

pub(crate) fn module_service_slot_allocator_source_fact_map_complete() -> bool {
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCE_COUNT {
        let source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[idx];
        if source.name.is_empty()
            || source.schema.is_empty()
            || source.id.is_empty()
            || source.missing_reason.is_empty()
            || source.source_method.is_empty()
            || source.source_fact_locator.is_empty()
            || source.source_evidence_schema.is_empty()
            || source.source_evidence_missing_reason.is_empty()
        {
            return false;
        }
        idx += 1;
    }
    true
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorPrerequisite {
    pub(crate) available: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorPrerequisiteSource {
    pub(crate) name: &'static str,
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) missing_status: &'static str,
    pub(crate) missing_reason: &'static str,
    pub(crate) source_method: &'static str,
    pub(crate) source_fact_locator: &'static str,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_missing_reason: &'static str,
}

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCE_COUNT: usize = 3;

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES:
    [ModuleServiceSlotAllocatorPrerequisiteSource;
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCE_COUNT] = [
    ModuleServiceSlotAllocatorPrerequisiteSource {
        name: "durable_audit_write",
        schema: "raios.audit_record.v0",
        id: "module.service_slot_allocator.durable_audit_write.current_boot",
        missing_status: "missing",
        missing_reason: "durable_audit_write_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.durable_audit_write",
        source_evidence_schema:
            "raios.service_slot_allocator_durable_audit_write_source_evidence.v0",
        source_evidence_missing_reason:
            "service_slot_allocator_durable_audit_write_source_evidence_missing",
    },
    ModuleServiceSlotAllocatorPrerequisiteSource {
        name: "rollback_plan_install",
        schema: "raios.rollback_plan.v0",
        id: "module.service_slot_allocator.rollback_plan_install.current_boot",
        missing_status: "missing",
        missing_reason: "rollback_install_missing",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.rollback_plan_install",
        source_evidence_schema: "raios.service_slot_allocator_rollback_install_source_evidence.v0",
        source_evidence_missing_reason:
            "service_slot_allocator_rollback_install_source_evidence_missing",
    },
    ModuleServiceSlotAllocatorPrerequisiteSource {
        name: "module_loader",
        schema: "raios.module_loader.v0",
        id: "module.service_slot_allocator.module_loader.current_boot",
        missing_status: "unavailable",
        missing_reason: "module_loader_unimplemented",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.module_loader",
        source_evidence_schema: "raios.service_slot_allocator_module_loader_source_evidence.v0",
        source_evidence_missing_reason:
            "service_slot_allocator_module_loader_source_evidence_missing",
    },
];

pub(crate) fn module_service_slot_allocator_prerequisite_source_map_complete() -> bool {
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCE_COUNT {
        let source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[idx];
        if source.name.is_empty()
            || source.schema.is_empty()
            || source.id.is_empty()
            || source.missing_status.is_empty()
            || source.missing_reason.is_empty()
            || source.source_method.is_empty()
            || source.source_fact_locator.is_empty()
            || source.source_evidence_schema.is_empty()
            || source.source_evidence_missing_reason.is_empty()
        {
            return false;
        }
        idx += 1;
    }
    true
}

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA: &str =
    "raios.module_service_slot_allocator_authority.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_ID: &str =
    "module.service_slot_allocator.authority.current_boot";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_service_slot_allocator_authority_source_evidence.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD: &str =
    "module.service_slot_allocator";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR: &str =
    "module.service_slot_allocator.allocator_authority";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS: &str =
    "denied_allocator_authority_not_granted";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON: &str =
    "service_slot_allocator_authority_boundary_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS: &str =
    "denied_missing_service_slot_allocator_authority";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "service_slot_allocator_authority_source_evidence_missing";

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA: &str =
    "raios.service_slot_allocation_intent.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID: &str =
    "module.service_slot_allocator.allocation_intent.current_boot";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.service_slot_allocation_intent_source_evidence.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD: &str =
    "module.service_slot_allocator";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR: &str =
    "module.service_slot_allocator.allocation_intent";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON: &str =
    "service_slot_allocation_intent_defined_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "service_slot_allocation_intent_source_evidence_missing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "service_slot_allocation_intent_source_chain_incomplete";

pub(crate) const MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT: usize = 5;

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAuthorityInputSpec {
    pub(crate) name: &'static str,
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_method: &'static str,
    pub(crate) source_fact_locator: &'static str,
    pub(crate) available_status: &'static str,
    pub(crate) available_reason: &'static str,
    pub(crate) missing_status: &'static str,
    pub(crate) source_evidence_missing_reason: &'static str,
    pub(crate) source_chain_incomplete_reason: &'static str,
}

pub(crate) const MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES:
    [ModuleServiceSlotAuthorityInputSpec; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] = [
    ModuleServiceSlotAuthorityInputSpec {
        name: "policy_decision",
        schema: "raios.service_slot_allocator_policy_decision.v0",
        id: "module.service_slot_allocator.policy_decision.current_boot",
        source_evidence_schema: "raios.service_slot_allocator_policy_decision_source_evidence.v0",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.policy_decision",
        available_status: "defined_non_authorizing",
        available_reason: "service_slot_allocator_policy_decision_defined_non_authorizing",
        missing_status: "missing",
        source_evidence_missing_reason:
            "service_slot_allocator_policy_decision_source_evidence_missing",
        source_chain_incomplete_reason:
            "service_slot_allocator_policy_decision_source_chain_incomplete",
    },
    ModuleServiceSlotAuthorityInputSpec {
        name: "registry_write_authority",
        schema: "raios.service_slot_registry_write_authority.v0",
        id: "module.service_slot_allocator.registry_write_authority.current_boot",
        source_evidence_schema: "raios.service_slot_registry_write_authority_source_evidence.v0",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.registry_write_authority",
        available_status: "defined_non_authorizing",
        available_reason: "service_slot_registry_write_authority_defined_non_authorizing",
        missing_status: "missing",
        source_evidence_missing_reason:
            "service_slot_registry_write_authority_source_evidence_missing",
        source_chain_incomplete_reason:
            "service_slot_registry_write_authority_source_chain_incomplete",
    },
    ModuleServiceSlotAuthorityInputSpec {
        name: "loader_runtime_contract",
        schema: "raios.module_loader_runtime_contract.v0",
        id: "module.service_slot_allocator.loader_runtime_contract.current_boot",
        source_evidence_schema: "raios.module_loader_runtime_contract_source_evidence.v0",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.loader_runtime_contract",
        available_status: "defined_non_authorizing",
        available_reason: "module_loader_runtime_contract_defined_non_authorizing",
        missing_status: "missing",
        source_evidence_missing_reason: "module_loader_runtime_contract_source_evidence_missing",
        source_chain_incomplete_reason: "module_loader_runtime_contract_source_chain_incomplete",
    },
    ModuleServiceSlotAuthorityInputSpec {
        name: "health_monitor_binding",
        schema: "raios.service_health_monitor_binding.v0",
        id: "module.service_slot_allocator.health_monitor_binding.current_boot",
        source_evidence_schema: "raios.service_health_monitor_binding_source_evidence.v0",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.health_monitor_binding",
        available_status: "defined_non_authorizing",
        available_reason: "service_health_monitor_binding_defined_non_authorizing",
        missing_status: "missing",
        source_evidence_missing_reason: "service_health_monitor_binding_source_evidence_missing",
        source_chain_incomplete_reason: "service_health_monitor_binding_source_chain_incomplete",
    },
    ModuleServiceSlotAuthorityInputSpec {
        name: "unload_cleanup_authority",
        schema: "raios.service_unload_cleanup_authority.v0",
        id: "module.service_slot_allocator.unload_cleanup_authority.current_boot",
        source_evidence_schema: "raios.service_unload_cleanup_authority_source_evidence.v0",
        source_method: "module.service_slot_allocator",
        source_fact_locator: "module.service_slot_allocator.unload_cleanup_authority",
        available_status: "defined_non_authorizing",
        available_reason: "service_unload_cleanup_authority_defined_non_authorizing",
        missing_status: "missing",
        source_evidence_missing_reason: "service_unload_cleanup_authority_source_evidence_missing",
        source_chain_incomplete_reason: "service_unload_cleanup_authority_source_chain_incomplete",
    },
];

pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA: &str =
    "raios.module_service_slot_allocator_authority_decision.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID: &str =
    "module.service_slot_allocator.authority_decision.current_boot";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_service_slot_allocator_authority_decision_source_evidence.v0";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD: &str =
    "module.service_slot_allocator";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR: &str =
    "module.service_slot_allocator.authority_decision";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON: &str =
    "service_slot_allocator_authority_decision_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "service_slot_allocator_authority_decision_source_evidence_missing";
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "service_slot_allocator_authority_decision_source_chain_incomplete";

pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA: &str =
    "raios.service_slot_registry_write_commit_gate.v0";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID: &str =
    "module.service_slot_allocator.registry_write_commit_gate.current_boot";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.service_slot_registry_write_commit_gate_source_evidence.v0";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD: &str =
    "module.service_slot_allocator";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR: &str =
    "module.service_slot_allocator.registry_write_commit_gate";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON: &str =
    "service_slot_registry_write_commit_gate_non_authorizing";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "service_slot_registry_write_commit_gate_source_evidence_missing";
pub(crate) const MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "service_slot_registry_write_commit_gate_source_chain_incomplete";

pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA: &str =
    "raios.module_loader_runtime_execution_commit_gate.v0";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID: &str =
    "module.loader_runtime.execution_commit_gate.current_boot";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_runtime_execution_commit_gate_source_evidence.v0";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.execution_commit_gate";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON: &str =
    "module_loader_runtime_execution_commit_gate_non_authorizing";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_runtime_execution_commit_gate_source_evidence_missing";
pub(crate) const MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_runtime_execution_commit_gate_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_intake_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_intake_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_descriptor_intake_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_intake_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON: &str =
    "module_loader_descriptor_intake_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_descriptor_intake_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_descriptor_intake_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_artifact_byte_intake_boundary.v0";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID: &str =
    "module.loader_runtime.artifact_byte_intake_boundary.current_boot";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_artifact_byte_intake_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.artifact_byte_intake_boundary";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON: &str =
    "module_loader_artifact_byte_intake_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_artifact_byte_intake_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_artifact_byte_intake_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_execution_authorization_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID: &str =
    "module.loader_runtime.execution_authorization_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_execution_authorization_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.execution_authorization_boundary";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON: &str =
    "module_loader_execution_authorization_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_execution_authorization_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_execution_authorization_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_registry_mutation_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID: &str =
    "module.loader_runtime.service_registry_mutation_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_registry_mutation_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_registry_mutation_boundary";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON: &str =
    "module_loader_service_registry_mutation_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_service_registry_mutation_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_service_registry_mutation_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_load_attempt_boundary.v0";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID: &str =
    "module.loader_runtime.load_attempt_boundary.current_boot";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_load_attempt_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD: &str = "module.loader_runtime";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.load_attempt_boundary";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON: &str =
    "module_loader_load_attempt_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_load_attempt_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_load_attempt_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_artifact_load_boundary.v0";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID: &str =
    "module.loader_runtime.artifact_load_boundary.current_boot";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_artifact_load_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD: &str = "module.loader_runtime";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.artifact_load_boundary";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON: &str =
    "module_loader_artifact_load_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_artifact_load_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_artifact_load_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_mapping_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_mapping_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_executable_mapping_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_mapping_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON: &str =
    "module_loader_executable_mapping_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_executable_mapping_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_executable_mapping_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_entrypoint_transfer_boundary.v0";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID: &str =
    "module.loader_runtime.entrypoint_transfer_boundary.current_boot";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_entrypoint_transfer_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.entrypoint_transfer_boundary";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON: &str =
    "module_loader_entrypoint_transfer_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_entrypoint_transfer_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_entrypoint_transfer_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_start_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_ID: &str =
    "module.loader_runtime.service_start_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_start_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD: &str = "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_start_boundary";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_REASON: &str =
    "module_loader_service_start_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_service_start_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_service_start_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_health_binding_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID: &str =
    "module.loader_runtime.service_health_binding_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_health_binding_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_health_binding_boundary";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON: &str =
    "module_loader_service_health_binding_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_service_health_binding_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_service_health_binding_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_running_state_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID: &str =
    "module.loader_runtime.service_running_state_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_running_state_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_running_state_boundary";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON: &str =
    "module_loader_service_running_state_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_service_running_state_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_service_running_state_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_start_audit_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID: &str =
    "module.loader_runtime.service_start_audit_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_start_audit_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_start_audit_boundary";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON: &str =
    "module_loader_service_start_audit_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_service_start_audit_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_service_start_audit_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_service_unload_cleanup_boundary.v0";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID: &str =
    "module.loader_runtime.service_unload_cleanup_boundary.current_boot";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_service_unload_cleanup_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.service_unload_cleanup_boundary";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON: &str =
    "module_loader_service_unload_cleanup_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_service_unload_cleanup_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_service_unload_cleanup_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_live_load_commit_boundary.v0";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID: &str =
    "module.loader_runtime.live_load_commit_boundary.current_boot";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_live_load_commit_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.live_load_commit_boundary";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON: &str =
    "module_loader_live_load_commit_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_live_load_commit_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_live_load_commit_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_commit_audit_boundary.v0";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID: &str =
    "module.loader_runtime.commit_audit_boundary.current_boot";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_commit_audit_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD: &str = "module.loader_runtime";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.commit_audit_boundary";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON: &str =
    "module_loader_commit_audit_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_commit_audit_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_commit_audit_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_commit_rollback_boundary.v0";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID: &str =
    "module.loader_runtime.commit_rollback_boundary.current_boot";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_commit_rollback_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.commit_rollback_boundary";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON: &str =
    "module_loader_commit_rollback_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_commit_rollback_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_commit_rollback_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_commit_result_boundary.v0";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID: &str =
    "module.loader_runtime.commit_result_boundary.current_boot";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_commit_result_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD: &str = "module.loader_runtime";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.commit_result_boundary";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS: &str = "defined_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON: &str =
    "module_loader_commit_result_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_commit_result_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_commit_result_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_acceptance_authority_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_acceptance_authority_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA:
    &str = "raios.module_loader_descriptor_acceptance_authority_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_acceptance_authority_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON: &str =
    "module_loader_descriptor_acceptance_authority_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_descriptor_acceptance_authority_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_descriptor_acceptance_authority_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_parser_contract_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_parser_contract_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_descriptor_parser_contract_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_parser_contract_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON: &str =
    "module_loader_descriptor_parser_contract_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_descriptor_parser_contract_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_descriptor_parser_contract_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_parser_result_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_parser_result_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_descriptor_parser_result_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_parser_result_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON: &str =
    "module_loader_descriptor_parser_result_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_descriptor_parser_result_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_descriptor_parser_result_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_schema_validation_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_schema_validation_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_descriptor_schema_validation_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_schema_validation_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON: &str =
    "module_loader_descriptor_schema_validation_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_descriptor_schema_validation_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_descriptor_schema_validation_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_capability_validation_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_capability_validation_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA:
    &str = "raios.module_loader_descriptor_capability_validation_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_capability_validation_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON: &str =
    "module_loader_descriptor_capability_validation_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_descriptor_capability_validation_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_descriptor_capability_validation_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_descriptor_load_plan_boundary.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID: &str =
    "module.loader_runtime.descriptor_load_plan_boundary.current_boot";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_descriptor_load_plan_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.descriptor_load_plan_boundary";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON: &str =
    "module_loader_descriptor_load_plan_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON: &str =
    "module_loader_descriptor_load_plan_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON: &str =
    "module_loader_descriptor_load_plan_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_load_plan_authority_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_load_plan_authority_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA:
    &str = "raios.module_loader_executable_load_plan_authority_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_load_plan_authority_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON: &str =
    "module_loader_executable_load_plan_authority_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_executable_load_plan_authority_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_executable_load_plan_authority_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_load_plan_result_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_load_plan_result_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_executable_load_plan_result_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_load_plan_result_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON: &str =
    "module_loader_executable_load_plan_result_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_executable_load_plan_result_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_executable_load_plan_result_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_image_layout_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_image_layout_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_executable_image_layout_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_image_layout_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON: &str =
    "module_loader_executable_image_layout_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_executable_image_layout_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_executable_image_layout_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_page_mapping_plan_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_page_mapping_plan_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_executable_page_mapping_plan_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_page_mapping_plan_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON: &str =
    "module_loader_executable_page_mapping_plan_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS: &str =
    "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_executable_page_mapping_plan_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_executable_page_mapping_plan_boundary_source_chain_incomplete";

pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA: &str =
    "raios.module_loader_executable_page_mapping_boundary.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID: &str =
    "module.loader_runtime.executable_page_mapping_boundary.current_boot";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA: &str =
    "raios.module_loader_executable_page_mapping_boundary_source_evidence.v0";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD: &str =
    "module.loader_runtime";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR: &str =
    "module.loader_runtime.executable_page_mapping_boundary";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS: &str =
    "defined_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON: &str =
    "module_loader_executable_page_mapping_boundary_non_authorizing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS: &str = "missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON:
    &str = "module_loader_executable_page_mapping_boundary_source_evidence_missing";
pub(crate) const MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON:
    &str = "module_loader_executable_page_mapping_boundary_source_chain_incomplete";

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorAuthority {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocationIntent {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAuthorityInput {
    pub(crate) spec: ModuleServiceSlotAuthorityInputSpec,
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) dependency_source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_chain_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorAuthorityDecision {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) input_chain_complete: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotRegistryWriteCommitGate {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) authority_decision_present: bool,
    pub(crate) registry_write_authority_present: bool,
    pub(crate) registry_binding_available: bool,
    pub(crate) durable_audit_write_available: bool,
    pub(crate) rollback_plan_install_available: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorCandidate {
    pub(crate) retained_reservation_present: bool,
    pub(crate) allocator_runtime: ModuleServiceSlotAllocatorFact,
    pub(crate) registry_binding: ModuleServiceSlotAllocatorFact,
    pub(crate) health_state: ModuleServiceSlotAllocatorFact,
    pub(crate) unload_cleanup: ModuleServiceSlotAllocatorFact,
    pub(crate) durable_audit_write: ModuleServiceSlotAllocatorPrerequisite,
    pub(crate) rollback_plan_install: ModuleServiceSlotAllocatorPrerequisite,
    pub(crate) module_loader: ModuleServiceSlotAllocatorPrerequisite,
    pub(crate) allocator_authority: ModuleServiceSlotAllocatorAuthority,
    pub(crate) allocation_intent: ModuleServiceSlotAllocationIntent,
    pub(crate) authority_inputs:
        [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    pub(crate) authority_decision: ModuleServiceSlotAllocatorAuthorityDecision,
    pub(crate) registry_write_commit_gate: ModuleServiceSlotRegistryWriteCommitGate,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotAllocatorEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) retained_reservation_status: &'static str,
    pub(crate) retained_reservation_reason: &'static str,
    pub(crate) allocator_runtime_status: &'static str,
    pub(crate) allocator_runtime_reason: &'static str,
    pub(crate) registry_binding_status: &'static str,
    pub(crate) registry_binding_reason: &'static str,
    pub(crate) health_state_status: &'static str,
    pub(crate) health_state_reason: &'static str,
    pub(crate) unload_cleanup_status: &'static str,
    pub(crate) unload_cleanup_reason: &'static str,
    pub(crate) durable_audit_status: &'static str,
    pub(crate) durable_audit_reason: &'static str,
    pub(crate) rollback_status: &'static str,
    pub(crate) rollback_reason: &'static str,
    pub(crate) module_loader_status: &'static str,
    pub(crate) module_loader_reason: &'static str,
    pub(crate) authority_status: &'static str,
    pub(crate) authority_reason: &'static str,
    pub(crate) allocation_intent_status: &'static str,
    pub(crate) allocation_intent_reason: &'static str,
    pub(crate) authority_input_statuses: [&'static str; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    pub(crate) authority_input_reasons: [&'static str; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    pub(crate) authority_decision_status: &'static str,
    pub(crate) authority_decision_reason: &'static str,
    pub(crate) registry_write_commit_gate_status: &'static str,
    pub(crate) registry_write_commit_gate_reason: &'static str,
    pub(crate) allocates_service_slot: bool,
    pub(crate) creates_service_inventory_records: bool,
    pub(crate) can_allocate: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleServiceSlotAllocatorSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_allocator_runtime_source_evidence_present: bool,
    pub(crate) actual_allocator_runtime_source_evidence_state: &'static str,
    pub(crate) actual_allocator_runtime_source_evidence_status: &'static str,
    pub(crate) actual_allocator_runtime_source_evidence_reason: &'static str,
    pub(crate) actual_registry_binding_source_evidence_present: bool,
    pub(crate) actual_registry_binding_source_evidence_state: &'static str,
    pub(crate) actual_registry_binding_source_evidence_status: &'static str,
    pub(crate) actual_registry_binding_source_evidence_reason: &'static str,
    pub(crate) actual_health_state_source_evidence_present: bool,
    pub(crate) actual_health_state_source_evidence_state: &'static str,
    pub(crate) actual_health_state_source_evidence_status: &'static str,
    pub(crate) actual_health_state_source_evidence_reason: &'static str,
    pub(crate) actual_unload_cleanup_source_evidence_present: bool,
    pub(crate) actual_unload_cleanup_source_evidence_state: &'static str,
    pub(crate) actual_unload_cleanup_source_evidence_status: &'static str,
    pub(crate) actual_unload_cleanup_source_evidence_reason: &'static str,
    pub(crate) actual_durable_audit_source_evidence_present: bool,
    pub(crate) actual_durable_audit_source_evidence_state: &'static str,
    pub(crate) actual_durable_audit_source_evidence_status: &'static str,
    pub(crate) actual_durable_audit_source_evidence_reason: &'static str,
    pub(crate) actual_rollback_install_source_evidence_present: bool,
    pub(crate) actual_rollback_install_source_evidence_state: &'static str,
    pub(crate) actual_rollback_install_source_evidence_status: &'static str,
    pub(crate) actual_rollback_install_source_evidence_reason: &'static str,
    pub(crate) actual_module_loader_source_evidence_present: bool,
    pub(crate) actual_module_loader_source_evidence_state: &'static str,
    pub(crate) actual_module_loader_source_evidence_status: &'static str,
    pub(crate) actual_module_loader_source_evidence_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderRuntimeFactSource {
    pub(crate) name: &'static str,
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) missing_reason: &'static str,
    pub(crate) source_method: &'static str,
    pub(crate) source_fact_locator: &'static str,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_missing_reason: &'static str,
}

pub(crate) const MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT: usize = 10;

pub(crate) const MODULE_LOADER_RUNTIME_FACT_SOURCES: [ModuleLoaderRuntimeFactSource;
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT] = [
    ModuleLoaderRuntimeFactSource {
        name: "loader_identity",
        schema: "raios.module_loader_identity.v0",
        id: "module.loader_runtime.identity.current_boot",
        missing_reason: "module_loader_identity_missing",
        source_method: "module.loader_identity",
        source_fact_locator: "module.loader_identity.loader_identity",
        source_evidence_schema: "raios.module_loader_identity_source_evidence.v0",
        source_evidence_missing_reason: "module_loader_identity_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "artifact_hash_binding",
        schema: "raios.module_loader_artifact_hash_binding.v0",
        id: "module.loader_runtime.artifact_hash_binding.current_boot",
        missing_reason: "module_loader_artifact_hash_binding_missing",
        source_method: "module.loader_artifact_hash_binding",
        source_fact_locator: "module.loader_artifact_hash_binding.artifact_hash_binding",
        source_evidence_schema: "raios.module_loader_artifact_hash_binding_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_artifact_hash_binding_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "entrypoint_abi",
        schema: "raios.module_loader_entrypoint_abi.v0",
        id: "module.loader_runtime.entrypoint_abi.current_boot",
        missing_reason: "module_loader_entrypoint_abi_missing",
        source_method: "module.loader_entrypoint_abi",
        source_fact_locator: "module.loader_entrypoint_abi.entrypoint_abi",
        source_evidence_schema: "raios.module_loader_entrypoint_abi_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_entrypoint_abi_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "address_space_boundary",
        schema: "raios.module_loader_address_space_boundary.v0",
        id: "module.loader_runtime.address_space_boundary.current_boot",
        missing_reason: "module_loader_address_space_boundary_missing",
        source_method: "module.loader_address_space_boundary",
        source_fact_locator: "module.loader_address_space_boundary.address_space_boundary",
        source_evidence_schema:
            "raios.module_loader_address_space_boundary_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_address_space_boundary_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "memory_map_constraints",
        schema: "raios.module_loader_memory_map_constraints.v0",
        id: "module.loader_runtime.memory_map_constraints.current_boot",
        missing_reason: "module_loader_memory_map_constraints_missing",
        source_method: "module.loader_memory_map_constraints",
        source_fact_locator: "module.loader_memory_map_constraints.memory_map_constraints",
        source_evidence_schema:
            "raios.module_loader_memory_map_constraints_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_memory_map_constraints_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "capability_import_table",
        schema: "raios.module_loader_capability_import_table.v0",
        id: "module.loader_runtime.capability_import_table.current_boot",
        missing_reason: "module_loader_capability_import_table_missing",
        source_method: "module.loader_capability_import_table",
        source_fact_locator: "module.loader_capability_import_table.capability_import_table",
        source_evidence_schema:
            "raios.module_loader_capability_import_table_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_capability_import_table_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "service_slot_binding",
        schema: "raios.module_loader_service_slot_binding.v0",
        id: "module.loader_runtime.service_slot_binding.current_boot",
        missing_reason: "module_loader_service_slot_binding_missing",
        source_method: "module.loader_service_slot_binding",
        source_fact_locator: "module.loader_service_slot_binding.service_slot_binding",
        source_evidence_schema: "raios.module_loader_service_slot_binding_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_service_slot_binding_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "health_state_hooks",
        schema: "raios.module_loader_health_state_hooks.v0",
        id: "module.loader_runtime.health_state_hooks.current_boot",
        missing_reason: "module_loader_health_state_hooks_missing",
        source_method: "module.loader_health_state_hooks",
        source_fact_locator: "module.loader_health_state_hooks.health_state_hooks",
        source_evidence_schema: "raios.module_loader_health_state_hooks_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_health_state_hooks_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "rollback_hooks",
        schema: "raios.module_loader_rollback_hooks.v0",
        id: "module.loader_runtime.rollback_hooks.current_boot",
        missing_reason: "module_loader_rollback_hooks_missing",
        source_method: "module.loader_rollback_hooks",
        source_fact_locator: "module.loader_rollback_hooks.rollback_hooks",
        source_evidence_schema: "raios.module_loader_rollback_hooks_source_evidence.v0",
        source_evidence_missing_reason: "module_loader_rollback_hooks_source_evidence_missing",
    },
    ModuleLoaderRuntimeFactSource {
        name: "audit_rollback_write_boundary_binding",
        schema: "raios.module_loader_audit_rollback_write_boundary_binding.v0",
        id: "module.loader_runtime.audit_rollback_write_boundary_binding.current_boot",
        missing_reason: "module_loader_audit_rollback_write_boundary_binding_missing",
        source_method: "module.loader_audit_rollback_write_boundary_binding",
        source_fact_locator:
            "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding",
        source_evidence_schema:
            "raios.module_loader_audit_rollback_write_boundary_binding_source_evidence.v0",
        source_evidence_missing_reason:
            "module_loader_audit_rollback_write_boundary_binding_source_evidence_missing",
    },
];

pub(crate) fn module_loader_runtime_source_fact_map_complete() -> bool {
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        if source.name.is_empty()
            || source.schema.is_empty()
            || source.id.is_empty()
            || source.missing_reason.is_empty()
            || source.source_method.is_empty()
            || source.source_fact_locator.is_empty()
            || source.source_evidence_schema.is_empty()
            || source.source_evidence_missing_reason.is_empty()
        {
            return false;
        }
        idx += 1;
    }
    true
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderRuntimeFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_module_evidence: bool,
    pub(crate) binds_service_slot_allocator: bool,
    pub(crate) binds_audit_rollback_write_boundary: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderRuntimeExecutionCommitGate {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) authority_decision_present: bool,
    pub(crate) loader_runtime_contract_present: bool,
    pub(crate) loader_runtime_source_evidence_complete: bool,
    pub(crate) service_slot_binding_source_evidence_present: bool,
    pub(crate) service_slot_binding_fact_present: bool,
    pub(crate) audit_rollback_write_boundary_source_evidence_present: bool,
    pub(crate) audit_rollback_write_boundary_fact_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderDescriptorIntakeBoundary {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) registry_write_commit_gate_present: bool,
    pub(crate) execution_commit_gate_present: bool,
    pub(crate) loader_runtime_source_evidence_complete: bool,
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderArtifactByteIntakeBoundary {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) descriptor_intake_boundary_present: bool,
    pub(crate) descriptor_intake_boundary_source_chain_complete: bool,
    pub(crate) execution_commit_gate_present: bool,
    pub(crate) artifact_hash_binding_present: bool,
    pub(crate) retained_artifact_reference_present: bool,
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderExecutionAuthorizationBoundary {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) artifact_byte_intake_boundary_present: bool,
    pub(crate) artifact_byte_intake_boundary_source_chain_complete: bool,
    pub(crate) descriptor_intake_boundary_present: bool,
    pub(crate) descriptor_intake_boundary_source_chain_complete: bool,
    pub(crate) execution_commit_gate_present: bool,
    pub(crate) entrypoint_abi_source_evidence_present: bool,
    pub(crate) address_space_source_evidence_present: bool,
    pub(crate) memory_map_source_evidence_present: bool,
    pub(crate) audit_rollback_write_boundary_source_evidence_present: bool,
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderServiceRegistryMutationBoundary {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) execution_authorization_boundary_present: bool,
    pub(crate) execution_authorization_boundary_source_chain_complete: bool,
    pub(crate) registry_write_commit_gate_present: bool,
    pub(crate) service_slot_binding_source_evidence_present: bool,
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderLiveLoadBoundary {
    pub(crate) present: bool,
    pub(crate) source_evidence_event_id: Option<event_log::EventId>,
    pub(crate) source_evidence_schema: &'static str,
    pub(crate) source_evidence_state: &'static str,
    pub(crate) source_evidence_status: &'static str,
    pub(crate) source_evidence_reason: &'static str,
    pub(crate) source_evidence_method: &'static str,
    pub(crate) source_evidence_fact_locator: &'static str,
    pub(crate) source_chain_complete: bool,
    pub(crate) load_attempt_boundary_present: bool,
    pub(crate) load_attempt_boundary_source_chain_complete: bool,
    pub(crate) artifact_load_boundary_present: bool,
    pub(crate) artifact_load_boundary_source_chain_complete: bool,
    pub(crate) executable_mapping_boundary_present: bool,
    pub(crate) executable_mapping_boundary_source_chain_complete: bool,
    pub(crate) entrypoint_transfer_boundary_present: bool,
    pub(crate) entrypoint_transfer_boundary_source_chain_complete: bool,
    pub(crate) service_start_boundary_present: bool,
    pub(crate) service_start_boundary_source_chain_complete: bool,
    pub(crate) service_health_binding_boundary_present: bool,
    pub(crate) service_health_binding_boundary_source_chain_complete: bool,
    pub(crate) service_running_state_boundary_present: bool,
    pub(crate) service_running_state_boundary_source_chain_complete: bool,
    pub(crate) service_start_audit_boundary_present: bool,
    pub(crate) service_start_audit_boundary_source_chain_complete: bool,
    pub(crate) service_unload_cleanup_boundary_present: bool,
    pub(crate) service_unload_cleanup_boundary_source_chain_complete: bool,
    pub(crate) live_load_commit_boundary_present: bool,
    pub(crate) live_load_commit_boundary_source_chain_complete: bool,
    pub(crate) commit_audit_boundary_present: bool,
    pub(crate) commit_audit_boundary_source_chain_complete: bool,
    pub(crate) commit_rollback_boundary_present: bool,
    pub(crate) commit_rollback_boundary_source_chain_complete: bool,
    pub(crate) commit_result_boundary_present: bool,
    pub(crate) commit_result_boundary_source_chain_complete: bool,
    pub(crate) descriptor_acceptance_authority_boundary_present: bool,
    pub(crate) descriptor_acceptance_authority_boundary_source_chain_complete: bool,
    pub(crate) descriptor_parser_contract_boundary_present: bool,
    pub(crate) descriptor_parser_contract_boundary_source_chain_complete: bool,
    pub(crate) descriptor_parser_result_boundary_present: bool,
    pub(crate) descriptor_parser_result_boundary_source_chain_complete: bool,
    pub(crate) descriptor_schema_validation_boundary_present: bool,
    pub(crate) descriptor_schema_validation_boundary_source_chain_complete: bool,
    pub(crate) descriptor_capability_validation_boundary_present: bool,
    pub(crate) descriptor_capability_validation_boundary_source_chain_complete: bool,
    pub(crate) descriptor_load_plan_boundary_present: bool,
    pub(crate) descriptor_load_plan_boundary_source_chain_complete: bool,
    pub(crate) executable_load_plan_authority_boundary_present: bool,
    pub(crate) executable_load_plan_authority_boundary_source_chain_complete: bool,
    pub(crate) executable_load_plan_result_boundary_present: bool,
    pub(crate) executable_load_plan_result_boundary_source_chain_complete: bool,
    pub(crate) executable_image_layout_boundary_present: bool,
    pub(crate) executable_image_layout_boundary_source_chain_complete: bool,
    pub(crate) executable_page_mapping_plan_boundary_present: bool,
    pub(crate) executable_page_mapping_plan_boundary_source_chain_complete: bool,
    pub(crate) artifact_byte_intake_boundary_present: bool,
    pub(crate) artifact_byte_intake_boundary_source_chain_complete: bool,
    pub(crate) execution_authorization_boundary_present: bool,
    pub(crate) execution_authorization_boundary_source_chain_complete: bool,
    pub(crate) service_registry_mutation_boundary_present: bool,
    pub(crate) service_registry_mutation_boundary_source_chain_complete: bool,
    pub(crate) service_slot_binding_source_evidence_present: bool,
    pub(crate) health_state_hooks_source_evidence_present: bool,
    pub(crate) artifact_hash_binding_present: bool,
    pub(crate) entrypoint_abi_source_evidence_present: bool,
    pub(crate) address_space_source_evidence_present: bool,
    pub(crate) memory_map_source_evidence_present: bool,
    pub(crate) capability_import_table_source_evidence_present: bool,
    pub(crate) audit_rollback_write_boundary_source_evidence_present: bool,
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) retained_artifact_reference_present: bool,
    pub(crate) retained_service_slot_reservation_present: bool,
    pub(crate) loader_runtime_source_evidence_event_ids:
        [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_source_evidence_present:
        [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    pub(crate) loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderRuntimeCandidate {
    pub(crate) manifest_reference_present: bool,
    pub(crate) artifact_reference_present: bool,
    pub(crate) vm_report_reference_present: bool,
    pub(crate) local_attestation_reference_present: bool,
    pub(crate) local_approval_reference_present: bool,
    pub(crate) computed_grant_reference_present: bool,
    pub(crate) audit_rollback_reference_present: bool,
    pub(crate) service_slot_reservation_present: bool,
    pub(crate) service_slot_allocator_readiness_present: bool,
    pub(crate) service_slot_allocator_ready: bool,
    pub(crate) service_slot_allocator_unready_status: &'static str,
    pub(crate) service_slot_allocator_unready_reason: &'static str,
    pub(crate) loader_identity: ModuleLoaderRuntimeFact,
    pub(crate) artifact_hash_binding: ModuleLoaderRuntimeFact,
    pub(crate) entrypoint_abi: ModuleLoaderRuntimeFact,
    pub(crate) address_space_boundary: ModuleLoaderRuntimeFact,
    pub(crate) memory_map_constraints: ModuleLoaderRuntimeFact,
    pub(crate) capability_import_table: ModuleLoaderRuntimeFact,
    pub(crate) service_slot_binding: ModuleLoaderRuntimeFact,
    pub(crate) health_state_hooks: ModuleLoaderRuntimeFact,
    pub(crate) rollback_hooks: ModuleLoaderRuntimeFact,
    pub(crate) audit_rollback_write_boundary_binding: ModuleLoaderRuntimeFact,
    pub(crate) execution_commit_gate: ModuleLoaderRuntimeExecutionCommitGate,
    pub(crate) descriptor_intake_boundary: ModuleLoaderDescriptorIntakeBoundary,
    pub(crate) artifact_byte_intake_boundary: ModuleLoaderArtifactByteIntakeBoundary,
    pub(crate) execution_authorization_boundary: ModuleLoaderExecutionAuthorizationBoundary,
    pub(crate) service_registry_mutation_boundary: ModuleLoaderServiceRegistryMutationBoundary,
    pub(crate) load_attempt_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) artifact_load_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_mapping_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) entrypoint_transfer_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) service_start_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) service_health_binding_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) service_running_state_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) service_start_audit_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) service_unload_cleanup_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) live_load_commit_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) commit_audit_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) commit_rollback_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) commit_result_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_acceptance_authority_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_parser_contract_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_parser_result_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_schema_validation_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_capability_validation_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) descriptor_load_plan_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_load_plan_authority_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_load_plan_result_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_image_layout_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_page_mapping_plan_boundary: ModuleLoaderLiveLoadBoundary,
    pub(crate) executable_page_mapping_boundary: ModuleLoaderLiveLoadBoundary,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderRuntimeEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) manifest_reference_status: &'static str,
    pub(crate) manifest_reference_reason: &'static str,
    pub(crate) artifact_reference_status: &'static str,
    pub(crate) artifact_reference_reason: &'static str,
    pub(crate) vm_report_reference_status: &'static str,
    pub(crate) vm_report_reference_reason: &'static str,
    pub(crate) local_attestation_reference_status: &'static str,
    pub(crate) local_attestation_reference_reason: &'static str,
    pub(crate) local_approval_reference_status: &'static str,
    pub(crate) local_approval_reference_reason: &'static str,
    pub(crate) computed_grant_reference_status: &'static str,
    pub(crate) computed_grant_reference_reason: &'static str,
    pub(crate) audit_rollback_reference_status: &'static str,
    pub(crate) audit_rollback_reference_reason: &'static str,
    pub(crate) service_slot_reservation_status: &'static str,
    pub(crate) service_slot_reservation_reason: &'static str,
    pub(crate) service_slot_allocator_readiness_status: &'static str,
    pub(crate) service_slot_allocator_readiness_reason: &'static str,
    pub(crate) service_slot_allocator_runtime_status: &'static str,
    pub(crate) service_slot_allocator_runtime_reason: &'static str,
    pub(crate) loader_identity_status: &'static str,
    pub(crate) loader_identity_reason: &'static str,
    pub(crate) artifact_hash_binding_status: &'static str,
    pub(crate) artifact_hash_binding_reason: &'static str,
    pub(crate) entrypoint_abi_status: &'static str,
    pub(crate) entrypoint_abi_reason: &'static str,
    pub(crate) address_space_boundary_status: &'static str,
    pub(crate) address_space_boundary_reason: &'static str,
    pub(crate) memory_map_constraints_status: &'static str,
    pub(crate) memory_map_constraints_reason: &'static str,
    pub(crate) capability_import_table_status: &'static str,
    pub(crate) capability_import_table_reason: &'static str,
    pub(crate) service_slot_binding_status: &'static str,
    pub(crate) service_slot_binding_reason: &'static str,
    pub(crate) health_state_hooks_status: &'static str,
    pub(crate) health_state_hooks_reason: &'static str,
    pub(crate) rollback_hooks_status: &'static str,
    pub(crate) rollback_hooks_reason: &'static str,
    pub(crate) audit_rollback_write_boundary_binding_status: &'static str,
    pub(crate) audit_rollback_write_boundary_binding_reason: &'static str,
    pub(crate) execution_commit_gate_status: &'static str,
    pub(crate) execution_commit_gate_reason: &'static str,
    pub(crate) descriptor_intake_boundary_status: &'static str,
    pub(crate) descriptor_intake_boundary_reason: &'static str,
    pub(crate) artifact_byte_intake_boundary_status: &'static str,
    pub(crate) artifact_byte_intake_boundary_reason: &'static str,
    pub(crate) execution_authorization_boundary_status: &'static str,
    pub(crate) execution_authorization_boundary_reason: &'static str,
    pub(crate) service_registry_mutation_boundary_status: &'static str,
    pub(crate) service_registry_mutation_boundary_reason: &'static str,
    pub(crate) load_attempt_boundary_status: &'static str,
    pub(crate) load_attempt_boundary_reason: &'static str,
    pub(crate) artifact_load_boundary_status: &'static str,
    pub(crate) artifact_load_boundary_reason: &'static str,
    pub(crate) executable_mapping_boundary_status: &'static str,
    pub(crate) executable_mapping_boundary_reason: &'static str,
    pub(crate) entrypoint_transfer_boundary_status: &'static str,
    pub(crate) entrypoint_transfer_boundary_reason: &'static str,
    pub(crate) service_start_boundary_status: &'static str,
    pub(crate) service_start_boundary_reason: &'static str,
    pub(crate) service_health_binding_boundary_status: &'static str,
    pub(crate) service_health_binding_boundary_reason: &'static str,
    pub(crate) service_running_state_boundary_status: &'static str,
    pub(crate) service_running_state_boundary_reason: &'static str,
    pub(crate) service_start_audit_boundary_status: &'static str,
    pub(crate) service_start_audit_boundary_reason: &'static str,
    pub(crate) service_unload_cleanup_boundary_status: &'static str,
    pub(crate) service_unload_cleanup_boundary_reason: &'static str,
    pub(crate) live_load_commit_boundary_status: &'static str,
    pub(crate) live_load_commit_boundary_reason: &'static str,
    pub(crate) commit_audit_boundary_status: &'static str,
    pub(crate) commit_audit_boundary_reason: &'static str,
    pub(crate) commit_rollback_boundary_status: &'static str,
    pub(crate) commit_rollback_boundary_reason: &'static str,
    pub(crate) commit_result_boundary_status: &'static str,
    pub(crate) commit_result_boundary_reason: &'static str,
    pub(crate) descriptor_acceptance_authority_boundary_status: &'static str,
    pub(crate) descriptor_acceptance_authority_boundary_reason: &'static str,
    pub(crate) descriptor_parser_contract_boundary_status: &'static str,
    pub(crate) descriptor_parser_contract_boundary_reason: &'static str,
    pub(crate) descriptor_parser_result_boundary_status: &'static str,
    pub(crate) descriptor_parser_result_boundary_reason: &'static str,
    pub(crate) descriptor_schema_validation_boundary_status: &'static str,
    pub(crate) descriptor_schema_validation_boundary_reason: &'static str,
    pub(crate) descriptor_capability_validation_boundary_status: &'static str,
    pub(crate) descriptor_capability_validation_boundary_reason: &'static str,
    pub(crate) descriptor_load_plan_boundary_status: &'static str,
    pub(crate) descriptor_load_plan_boundary_reason: &'static str,
    pub(crate) executable_load_plan_authority_boundary_status: &'static str,
    pub(crate) executable_load_plan_authority_boundary_reason: &'static str,
    pub(crate) executable_load_plan_result_boundary_status: &'static str,
    pub(crate) executable_load_plan_result_boundary_reason: &'static str,
    pub(crate) executable_image_layout_boundary_status: &'static str,
    pub(crate) executable_image_layout_boundary_reason: &'static str,
    pub(crate) executable_page_mapping_plan_boundary_status: &'static str,
    pub(crate) executable_page_mapping_plan_boundary_reason: &'static str,
    pub(crate) executable_page_mapping_boundary_status: &'static str,
    pub(crate) executable_page_mapping_boundary_reason: &'static str,
    pub(crate) loads_artifact: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) creates_service_inventory_records: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoaderRuntimeSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_loader_identity_source_evidence_present: bool,
    pub(crate) actual_loader_identity_source_evidence_state: &'static str,
    pub(crate) actual_loader_identity_source_evidence_status: &'static str,
    pub(crate) actual_loader_identity_source_evidence_reason: &'static str,
    pub(crate) actual_artifact_hash_source_evidence_present: bool,
    pub(crate) actual_artifact_hash_source_evidence_state: &'static str,
    pub(crate) actual_artifact_hash_source_evidence_status: &'static str,
    pub(crate) actual_artifact_hash_source_evidence_reason: &'static str,
    pub(crate) actual_entrypoint_abi_source_evidence_present: bool,
    pub(crate) actual_entrypoint_abi_source_evidence_state: &'static str,
    pub(crate) actual_entrypoint_abi_source_evidence_status: &'static str,
    pub(crate) actual_entrypoint_abi_source_evidence_reason: &'static str,
    pub(crate) actual_address_space_source_evidence_present: bool,
    pub(crate) actual_address_space_source_evidence_state: &'static str,
    pub(crate) actual_address_space_source_evidence_status: &'static str,
    pub(crate) actual_address_space_source_evidence_reason: &'static str,
    pub(crate) actual_memory_map_source_evidence_present: bool,
    pub(crate) actual_memory_map_source_evidence_state: &'static str,
    pub(crate) actual_memory_map_source_evidence_status: &'static str,
    pub(crate) actual_memory_map_source_evidence_reason: &'static str,
    pub(crate) actual_capability_table_source_evidence_present: bool,
    pub(crate) actual_capability_table_source_evidence_state: &'static str,
    pub(crate) actual_capability_table_source_evidence_status: &'static str,
    pub(crate) actual_capability_table_source_evidence_reason: &'static str,
    pub(crate) actual_service_slot_source_evidence_present: bool,
    pub(crate) actual_service_slot_source_evidence_state: &'static str,
    pub(crate) actual_service_slot_source_evidence_status: &'static str,
    pub(crate) actual_service_slot_source_evidence_reason: &'static str,
    pub(crate) actual_health_source_evidence_present: bool,
    pub(crate) actual_health_source_evidence_state: &'static str,
    pub(crate) actual_health_source_evidence_status: &'static str,
    pub(crate) actual_health_source_evidence_reason: &'static str,
    pub(crate) actual_rollback_source_evidence_present: bool,
    pub(crate) actual_rollback_source_evidence_state: &'static str,
    pub(crate) actual_rollback_source_evidence_status: &'static str,
    pub(crate) actual_rollback_source_evidence_reason: &'static str,
    pub(crate) actual_write_boundary_source_evidence_present: bool,
    pub(crate) actual_write_boundary_source_evidence_state: &'static str,
    pub(crate) actual_write_boundary_source_evidence_status: &'static str,
    pub(crate) actual_write_boundary_source_evidence_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderIdentityCandidate {
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) service_slot_allocator_readiness_present: bool,
    pub(crate) service_slot_allocator_ready: bool,
    pub(crate) service_slot_allocator_unready_status: &'static str,
    pub(crate) service_slot_allocator_unready_reason: &'static str,
    pub(crate) audit_rollback_write_boundary_present: bool,
    pub(crate) identity: ModuleLoaderRuntimeFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderIdentityEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) retained_module_evidence_status: &'static str,
    pub(crate) retained_module_evidence_reason: &'static str,
    pub(crate) service_slot_allocator_readiness_status: &'static str,
    pub(crate) service_slot_allocator_readiness_reason: &'static str,
    pub(crate) service_slot_allocator_runtime_status: &'static str,
    pub(crate) service_slot_allocator_runtime_reason: &'static str,
    pub(crate) audit_rollback_write_boundary_status: &'static str,
    pub(crate) audit_rollback_write_boundary_reason: &'static str,
    pub(crate) identity_status: &'static str,
    pub(crate) identity_reason: &'static str,
    pub(crate) loads_artifact: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) creates_service_inventory_records: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoaderIdentitySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_identity_status: &'static str,
    pub(crate) actual_identity_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderArtifactHashBindingFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_module_evidence: bool,
    pub(crate) binds_service_slot_allocator: bool,
    pub(crate) binds_audit_rollback_write_boundary: bool,
    pub(crate) binds_loader_identity: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderArtifactHashBindingCandidate {
    pub(crate) retained_module_evidence_present: bool,
    pub(crate) service_slot_allocator_readiness_present: bool,
    pub(crate) service_slot_allocator_ready: bool,
    pub(crate) service_slot_allocator_unready_status: &'static str,
    pub(crate) service_slot_allocator_unready_reason: &'static str,
    pub(crate) audit_rollback_write_boundary_present: bool,
    pub(crate) loader_identity_present: bool,
    pub(crate) artifact_hash_binding: ModuleLoaderArtifactHashBindingFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoaderArtifactHashBindingEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) retained_module_evidence_status: &'static str,
    pub(crate) retained_module_evidence_reason: &'static str,
    pub(crate) service_slot_allocator_readiness_status: &'static str,
    pub(crate) service_slot_allocator_readiness_reason: &'static str,
    pub(crate) service_slot_allocator_runtime_status: &'static str,
    pub(crate) service_slot_allocator_runtime_reason: &'static str,
    pub(crate) audit_rollback_write_boundary_status: &'static str,
    pub(crate) audit_rollback_write_boundary_reason: &'static str,
    pub(crate) loader_identity_status: &'static str,
    pub(crate) loader_identity_reason: &'static str,
    pub(crate) artifact_hash_binding_status: &'static str,
    pub(crate) artifact_hash_binding_reason: &'static str,
    pub(crate) loads_artifact: bool,
    pub(crate) allocates_service_slot: bool,
    pub(crate) creates_service_inventory_records: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoaderArtifactHashBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_artifact_hash_binding_status: &'static str,
    pub(crate) actual_artifact_hash_binding_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityCandidate {
    pub(crate) durable_audit_ledger: ModuleAuditRollbackAvailabilityFact,
    pub(crate) rollback_store: ModuleAuditRollbackAvailabilityFact,
    pub(crate) durable_write_policy_available: bool,
    pub(crate) rollback_install_policy_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_audit_ledger_status: &'static str,
    pub(crate) durable_audit_ledger_reason: &'static str,
    pub(crate) rollback_store_status: &'static str,
    pub(crate) rollback_store_reason: &'static str,
    pub(crate) durable_write_policy_available: bool,
    pub(crate) rollback_install_policy_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAvailabilitySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_evidence: bool,
    pub(crate) binds_availability: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyCandidate {
    pub(crate) durable_write_policy: ModuleAuditRollbackWritePolicyFact,
    pub(crate) rollback_install_policy: ModuleAuditRollbackWritePolicyFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_write_policy_status: &'static str,
    pub(crate) durable_write_policy_reason: &'static str,
    pub(crate) rollback_install_policy_status: &'static str,
    pub(crate) rollback_install_policy_reason: &'static str,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackWritePolicySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackPersistenceDeviceFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) stable_identity: bool,
    pub(crate) partition_inventory_available: bool,
    pub(crate) write_path_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_persistence_device: bool,
    pub(crate) has_audit_ledger_region: bool,
    pub(crate) has_rollback_store_region: bool,
    pub(crate) append_slots_available: bool,
    pub(crate) recovery_region_separated: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutCandidate {
    pub(crate) persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact,
    pub(crate) audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) persistence_device_status: &'static str,
    pub(crate) persistence_device_reason: &'static str,
    pub(crate) storage_layout_status: &'static str,
    pub(crate) storage_layout_reason: &'static str,
    pub(crate) persistence_device_available: bool,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackStorageLayoutSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_storage_layout: bool,
    pub(crate) binds_write_policy: bool,
    pub(crate) supports_append_only: bool,
    pub(crate) supports_flush: bool,
    pub(crate) supports_replay: bool,
    pub(crate) recovery_separation_respected: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineCandidate {
    pub(crate) audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact,
    pub(crate) rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_engine_status: &'static str,
    pub(crate) audit_engine_reason: &'static str,
    pub(crate) rollback_engine_status: &'static str,
    pub(crate) rollback_engine_reason: &'static str,
    pub(crate) audit_engine_available: bool,
    pub(crate) rollback_engine_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendEngineSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_write_policy: bool,
    pub(crate) binds_availability: bool,
    pub(crate) binds_storage_layout_id: bool,
    pub(crate) binds_append_engine_id: bool,
    pub(crate) binds_write_policy_id: bool,
    pub(crate) binds_availability_id: bool,
    pub(crate) binds_envelope_provenance: bool,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractCandidate {
    pub(crate) audit_append_envelope: ModuleAuditRollbackAppendContractFact,
    pub(crate) rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_append_status: &'static str,
    pub(crate) audit_append_reason: &'static str,
    pub(crate) rollback_transaction_status: &'static str,
    pub(crate) rollback_transaction_reason: &'static str,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendContractSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_audit_rollback: bool,
    pub(crate) binds_service_slot_reservation: bool,
    pub(crate) binds_pre_load_write_request: bool,
    pub(crate) binds_append_contract_id: bool,
    pub(crate) binds_target_schema: bool,
    pub(crate) binds_payload_hash: bool,
    pub(crate) binds_payload_provenance: bool,
    pub(crate) retained_audit_rollback_available: bool,
    pub(crate) service_slot_reservation_available: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) retained_audit_rollback_event_id: Option<event_log::EventId>,
    pub(crate) service_slot_reservation_event_id: Option<event_log::EventId>,
    pub(crate) payload_hash: Option<[u8; 32]>,
    pub(crate) source_payload_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) service_slot_reservation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashCandidate {
    pub(crate) audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact,
    pub(crate) rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_payload_status: &'static str,
    pub(crate) audit_payload_reason: &'static str,
    pub(crate) rollback_payload_status: &'static str,
    pub(crate) rollback_payload_reason: &'static str,
    pub(crate) retained_evidence_available: bool,
    pub(crate) service_slot_reservation_available: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendPayloadHashSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_append_contract: bool,
    pub(crate) binds_append_contract_id: bool,
    pub(crate) binds_append_engine_id: bool,
    pub(crate) binds_storage_layout_id: bool,
    pub(crate) binds_write_policy_id: bool,
    pub(crate) binds_availability_id: bool,
    pub(crate) binds_payload_hash: bool,
    pub(crate) binds_intent_provenance: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentCandidate {
    pub(crate) audit_record_append_intent: ModuleAuditRollbackAppendIntentFact,
    pub(crate) rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_intent_status: &'static str,
    pub(crate) audit_intent_reason: &'static str,
    pub(crate) rollback_intent_status: &'static str,
    pub(crate) rollback_intent_reason: &'static str,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
    pub(crate) append_intent_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendIntentSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWriteBoundaryCandidate {
    pub(crate) scope: &'static str,
    pub(crate) request_schema_ok: bool,
    pub(crate) manifest_status: &'static str,
    pub(crate) manifest_reason: &'static str,
    pub(crate) artifact_status: &'static str,
    pub(crate) artifact_reason: &'static str,
    pub(crate) vm_report_status: &'static str,
    pub(crate) vm_report_reason: &'static str,
    pub(crate) computed_grant_status: &'static str,
    pub(crate) computed_grant_reason: &'static str,
    pub(crate) local_attestation_status: &'static str,
    pub(crate) local_attestation_reason: &'static str,
    pub(crate) local_approval_status: &'static str,
    pub(crate) local_approval_reason: &'static str,
    pub(crate) audit_rollback_status: &'static str,
    pub(crate) audit_rollback_reason: &'static str,
    pub(crate) service_slot_status: &'static str,
    pub(crate) service_slot_reason: &'static str,
    pub(crate) manifest_hash_matches_grant: bool,
    pub(crate) artifact_hash_matches_grant: bool,
    pub(crate) vm_report_hash_matches_grant: bool,
    pub(crate) local_attestation_hash_matches_grant: bool,
    pub(crate) local_approval_hash_matches_audit: bool,
    pub(crate) audit_record_hash_matches_service_slot: bool,
    pub(crate) rollback_plan_hash_matches_service_slot: bool,
    pub(crate) service_slot_binds_audit_rollback: bool,
    pub(crate) durable_audit_ledger_status: &'static str,
    pub(crate) durable_audit_ledger_reason: &'static str,
    pub(crate) rollback_store_status: &'static str,
    pub(crate) rollback_store_reason: &'static str,
    pub(crate) durable_write_policy_status: &'static str,
    pub(crate) durable_write_policy_reason: &'static str,
    pub(crate) rollback_install_policy_status: &'static str,
    pub(crate) rollback_install_policy_reason: &'static str,
    pub(crate) audit_append_status: &'static str,
    pub(crate) audit_append_reason: &'static str,
    pub(crate) rollback_transaction_status: &'static str,
    pub(crate) rollback_transaction_reason: &'static str,
    pub(crate) audit_append_intent_status: &'static str,
    pub(crate) audit_append_intent_reason: &'static str,
    pub(crate) rollback_transaction_append_intent_status: &'static str,
    pub(crate) rollback_transaction_append_intent_reason: &'static str,
    pub(crate) recovery_artifact_loader_requested: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWriteBoundaryEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_audit_write_state: &'static str,
    pub(crate) durable_audit_write_reason: &'static str,
    pub(crate) rollback_install_state: &'static str,
    pub(crate) rollback_install_reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackWriteBoundarySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateRetainedCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateManifestReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleManifestReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateManifestEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) module_manifest_state: &'static str,
    pub(crate) accepted_manifest_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateManifestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_module_manifest_state: &'static str,
    pub(crate) accepted_manifest_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateArtifactReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateArtifactEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) candidate_artifact_state: &'static str,
    pub(crate) accepted_artifact_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateArtifactSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_candidate_artifact_state: &'static str,
    pub(crate) accepted_artifact_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateVmReportReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateVmReportEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) vm_test_report_state: &'static str,
    pub(crate) accepted_vm_report_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateVmReportSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_vm_test_report_state: &'static str,
    pub(crate) accepted_vm_report_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalAttestationReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) vm_report_event_id: Option<event_log::EventId>,
    pub(crate) vm_report_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalAttestationEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) local_attestation_state: &'static str,
    pub(crate) accepted_local_attestation_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateLocalAttestationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_local_attestation_state: &'static str,
    pub(crate) accepted_local_attestation_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalApprovalReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleLocalApprovalReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleLocalApprovalReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) vm_report_event_id: Option<event_log::EventId>,
    pub(crate) vm_report_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) attestation_event_id: Option<event_log::EventId>,
    pub(crate) attestation_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalApprovalEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) local_approval_state: &'static str,
    pub(crate) accepted_local_approval_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateLocalApprovalSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_local_approval_state: &'static str,
    pub(crate) accepted_local_approval_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateRetainedCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleComputedGrantReference>,
}

pub(crate) struct ModuleLoadGateRetainedSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleAuditRollbackReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackCandidate {
    pub(crate) retained_reference: bool,
    pub(crate) retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate,
    pub(crate) durable_audit_record: bool,
    pub(crate) rollback_plan: bool,
    pub(crate) audit_schema_ok: bool,
    pub(crate) rollback_schema_ok: bool,
    pub(crate) audit_binds_retained_grant: bool,
    pub(crate) audit_binds_manifest: bool,
    pub(crate) audit_binds_artifact: bool,
    pub(crate) audit_binds_vm_report: bool,
    pub(crate) audit_binds_local_attestation: bool,
    pub(crate) audit_binds_local_approval: bool,
    pub(crate) audit_binds_rollback_plan: bool,
    pub(crate) rollback_binds_artifact: bool,
    pub(crate) rollback_binds_service_slot: bool,
    pub(crate) ram_only_service_slot_allocated: bool,
    pub(crate) loader_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateAuditRollbackSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotReservationCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) grant_event_schema_ok: bool,
    pub(crate) audit_event_schema_ok: bool,
    pub(crate) grant_event_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) audit_event_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) event_reservation: Option<event_log::ModuleServiceSlotReservation>,
    pub(crate) candidate_reservation: Option<event_log::ModuleServiceSlotReservation>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotCandidate {
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) audit_rollback_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) audit_rollback_valid: bool,
    pub(crate) service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) service_slot_state: &'static str,
    pub(crate) accepted_service_slot_reservation_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateServiceSlotSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_service_slot_state: &'static str,
    pub(crate) accepted_service_slot_reservation_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLoaderRuntimeCandidate {
    pub(crate) manifest_reference_state: &'static str,
    pub(crate) manifest_reference_reason: &'static str,
    pub(crate) artifact_reference_state: &'static str,
    pub(crate) artifact_reference_reason: &'static str,
    pub(crate) vm_report_reference_state: &'static str,
    pub(crate) vm_report_reference_reason: &'static str,
    pub(crate) local_attestation_reference_state: &'static str,
    pub(crate) local_attestation_reference_reason: &'static str,
    pub(crate) local_approval_reference_state: &'static str,
    pub(crate) local_approval_reference_reason: &'static str,
    pub(crate) computed_grant_reference_state: &'static str,
    pub(crate) computed_grant_reference_reason: &'static str,
    pub(crate) audit_rollback_reference_state: &'static str,
    pub(crate) audit_rollback_reference_reason: &'static str,
    pub(crate) service_slot_reservation_state: &'static str,
    pub(crate) service_slot_reservation_reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLoaderRuntimeEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) retained_module_evidence_state: &'static str,
    pub(crate) retained_module_evidence_reason: &'static str,
    pub(crate) service_slot_allocator_state: &'static str,
    pub(crate) service_slot_allocator_status: &'static str,
    pub(crate) service_slot_allocator_reason: &'static str,
    pub(crate) loader_runtime_state: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateLoaderRuntimeSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) expected_retained_module_evidence_state: &'static str,
    pub(crate) expected_service_slot_allocator_state: &'static str,
    pub(crate) expected_loader_runtime_state: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_retained_module_evidence_state: &'static str,
    pub(crate) actual_retained_module_evidence_reason: &'static str,
    pub(crate) actual_service_slot_allocator_state: &'static str,
    pub(crate) actual_service_slot_allocator_status: &'static str,
    pub(crate) actual_service_slot_allocator_reason: &'static str,
    pub(crate) actual_loader_runtime_state: &'static str,
    pub(crate) passed: bool,
}

pub(crate) const MODULE_MANIFEST_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_ARTIFACT_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_VM_REPORT_SELFTEST_CASES: usize = 8;
pub(crate) const MODULE_LOCAL_ATTESTATION_SELFTEST_CASES: usize = 9;
pub(crate) const MODULE_LOCAL_APPROVAL_SELFTEST_CASES: usize = 10;
pub(crate) const MODULE_GRANT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 10;
pub(crate) const MODULE_SERVICE_SLOT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_SERVICE_SLOT_ALLOCATOR_SELFTEST_CASES: usize = 29;
pub(crate) const MODULE_LOADER_RUNTIME_SELFTEST_CASES: usize = 66;
pub(crate) const MODULE_LOADER_IDENTITY_SELFTEST_CASES: usize = 12;
pub(crate) const MODULE_LOADER_ARTIFACT_HASH_BINDING_SELFTEST_CASES: usize = 14;
pub(crate) const MODULE_AUDIT_ROLLBACK_AVAILABILITY_SELFTEST_CASES: usize = 8;
pub(crate) const MODULE_AUDIT_ROLLBACK_WRITE_POLICY_SELFTEST_CASES: usize = 12;
pub(crate) const MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_SELFTEST_CASES: usize = 15;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_ENGINE_SELFTEST_CASES: usize = 16;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_SELFTEST_CASES: usize = 24;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_PAYLOAD_HASH_SELFTEST_CASES: usize = 20;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES: usize = 20;
pub(crate) const MODULE_AUDIT_ROLLBACK_WRITE_BOUNDARY_SELFTEST_CASES: usize = 22;
pub(crate) const MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES: usize = 9;
pub(crate) const MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES: usize = 11;
pub(crate) const MODULE_LOAD_GATE_LOCAL_ATTESTATION_SELFTEST_CASES: usize = 11;
pub(crate) const MODULE_LOAD_GATE_LOCAL_APPROVAL_SELFTEST_CASES: usize = 12;
pub(crate) const MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 23;
pub(crate) const MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES: usize = 13;
pub(crate) const MODULE_LOAD_GATE_LOADER_RUNTIME_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_GRANT_TEST_MANIFEST_HASH: [u8; 32] = [0x11; 32];
pub(crate) const MODULE_GRANT_TEST_ARTIFACT_HASH: [u8; 32] = [0x22; 32];
pub(crate) const MODULE_GRANT_TEST_VM_REPORT_HASH: [u8; 32] = [0x33; 32];
pub(crate) const MODULE_GRANT_TEST_ATTESTATION_HASH: [u8; 32] = [0x44; 32];
pub(crate) const MODULE_GRANT_MISMATCH_MANIFEST_HASH: [u8; 32] = [0x55; 32];
pub(crate) const MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH: [u8; 32] = [0x66; 32];
pub(crate) const MODULE_AUDIT_TEST_PRE_INVENTORY_HASH: [u8; 32] = [0x77; 32];
pub(crate) const MODULE_AUDIT_TEST_CLEANUP_HASH: [u8; 32] = [0x88; 32];
pub(crate) const MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000026";
pub(crate) const MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000028";
pub(crate) const MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000029";
pub(crate) const MODULE_LOCAL_APPROVAL_TEST_RETAINED_ATTESTATION_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000030";
pub(crate) const MODULE_AUDIT_TEST_DENIAL_EVENT_ID: &str = "event.current_boot.00000031";
pub(crate) const MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000027";
pub(crate) const MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID: &str = "ram_only:svc.test.0001";
pub(crate) const MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID: &str =
    "event.current_boot.00000033";
