use crate::{agent_protocol_module_types::*, agent_protocol_support::*, ahci, pci};

pub(crate) const MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD: &str =
    "module.audit_rollback_storage_layout";
pub(crate) const AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA: &str =
    "raios.audit_rollback_storage_authority.v0";
pub(crate) const AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID: &str =
    "storage.authority.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER: &str =
    MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD;
pub(crate) const AUDIT_ROLLBACK_STORAGE_LAYOUT_SCHEMA: &str =
    "raios.audit_rollback_storage_layout.v0";
pub(crate) const AUDIT_ROLLBACK_STORAGE_LAYOUT_ID: &str =
    "storage.audit_rollback_layout.current_boot";
pub(crate) const AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID: &str = "append.audit_ledger.current_boot";
pub(crate) const AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA: &str = "raios.audit_record.v0";
pub(crate) const AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID: &str =
    "append.rollback_store.current_boot";
pub(crate) const AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA: &str =
    "raios.rollback_transaction.v0";
pub(crate) const AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER: &str =
    "module.audit_rollback_append_contract";
pub(crate) const AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA: &str =
    "raios.block_write_path_authority_gate.v0";
pub(crate) const AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID: &str =
    "block_write_path.authority.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SCHEMA: &str =
    "raios.audit_rollback_target_region_discovery.v0";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_ID: &str =
    "target_region.audit_rollback.current_boot";
pub(crate) const AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SOURCE: &str =
    "dedicated_audit_rollback_label_scan";

#[derive(Clone, Copy)]
pub(crate) struct AuditRollbackTargetRegionDiscovery {
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) source: &'static str,
    pub(crate) storage_authority_id: &'static str,
    pub(crate) partition_inventory_available: bool,
    pub(crate) partition_inventory_scheme: &'static str,
    pub(crate) partition_inventory_source_lba: u64,
    pub(crate) partition_entry_count: u8,
    pub(crate) mbr_signature_valid: bool,
    pub(crate) boot_metadata_lba: u64,
    pub(crate) candidate_region_present: bool,
    pub(crate) candidate_region_start_lba: u64,
    pub(crate) candidate_region_lba_count: u64,
    pub(crate) candidate_region_is_scratch: bool,
    pub(crate) candidate_overlaps_boot_metadata: bool,
    pub(crate) candidate_overlaps_scratch: bool,
    pub(crate) scratch_region_id: &'static str,
    pub(crate) scratch_region_available: bool,
    pub(crate) scratch_region_start_lba: u64,
    pub(crate) scratch_region_lba_count: u64,
    pub(crate) scratch_rejected_as_durable_authority: bool,
    pub(crate) durable_region_available: bool,
}

pub(crate) fn audit_rollback_block_write_path_gate_status(available: bool) -> &'static str {
    if available {
        "available"
    } else {
        "missing"
    }
}

pub(crate) fn audit_rollback_target_region_discovery(
    persistence_device: ModuleAuditRollbackPersistenceDeviceFact,
) -> AuditRollbackTargetRegionDiscovery {
    let partition_inventory = persistence_device.ahci_probe.partition_inventory;
    let scratch = persistence_device.ahci_probe.scratch_write_readback;
    let target = persistence_device.ahci_probe.audit_rollback_target_region;
    let scratch_region_available =
        scratch.available || scratch.label_found || scratch.block_write_authority_available;
    let target_available = target.available
        && target.region_within_device_bounds
        && target.no_boot_or_partition_metadata_overlap
        && !target.scratch_region_overlap;
    let reason = if target_available {
        "dedicated_audit_rollback_region_discovered_read_only"
    } else if target.label_found {
        target.reason
    } else if !partition_inventory.available {
        "partition_inventory_missing"
    } else {
        target.reason
    };

    AuditRollbackTargetRegionDiscovery {
        schema: AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SCHEMA,
        id: AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_ID,
        status: if target_available {
            "available"
        } else {
            "missing"
        },
        reason,
        source: AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SOURCE,
        storage_authority_id: AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID,
        partition_inventory_available: partition_inventory.available,
        partition_inventory_scheme: partition_inventory.scheme,
        partition_inventory_source_lba: partition_inventory.source_lba,
        partition_entry_count: partition_inventory.entry_count,
        mbr_signature_valid: partition_inventory.mbr_signature_valid,
        boot_metadata_lba: 0,
        candidate_region_present: target_available,
        candidate_region_start_lba: if target_available {
            target.region_start_lba
        } else {
            0
        },
        candidate_region_lba_count: if target_available {
            target.region_lba_count
        } else {
            0
        },
        candidate_region_is_scratch: false,
        candidate_overlaps_boot_metadata: target.boot_port_overlap || target.metadata_lba_overlap,
        candidate_overlaps_scratch: target.scratch_region_overlap,
        scratch_region_id: scratch.region_id,
        scratch_region_available,
        scratch_region_start_lba: scratch.region_start_lba,
        scratch_region_lba_count: scratch.region_lba_count,
        scratch_rejected_as_durable_authority: scratch_region_available,
        durable_region_available: target_available,
    }
}

pub(crate) fn emit_module_audit_rollback_storage_layout() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);

    begin_response("module.audit_rollback_storage_layout");
    raw_line("      \"schema\": \"raios.module_audit_rollback_storage_layout.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    emit_module_storage_layout_facts(storage, evaluation);
    raw_line(",");
    emit_module_storage_authority_foundation(storage.persistence_device_inventory, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("        \"storage_authority_owner\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER);
    raw_line(",");
    raw("        \"audit_append_target_id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw_line(",");
    raw("        \"rollback_append_target_id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw_line(",");
    raw("        \"transaction_writer_owner\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
    raw_line(",");
    raw("        \"storage_layout_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"storage_layout_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"persistence_device_available\": ");
    raw_bool(evaluation.persistence_device_available);
    raw_line(",");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_storage_authority\": false,");
    raw_line("        \"availability_facts_are_storage_authority\": false,");
    raw_line("        \"write_policy_facts_are_storage_authority\": false,");
    raw_line("        \"storage_layout_facts_are_append_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "persistence_device_inventory",
        evaluation.persistence_device_status,
        evaluation.persistence_device_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_rollback_storage_layout",
        evaluation.storage_layout_status,
        evaluation.storage_layout_reason,
    );
    emit_export_gate(
        &mut wrote,
        "append_engine",
        "missing",
        "append_engine_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_storage_layout");
}

pub(crate) fn emit_module_audit_rollback_storage_layout_selftest() {
    let cases = module_audit_rollback_storage_layout_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_storage_layout_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_storage_layout_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_audit_rollback_storage_layout_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_storage_layout_selftest");
}

pub(crate) fn emit_module_audit_rollback_storage_layout_selftest_case(
    case: &ModuleAuditRollbackStorageLayoutSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"writes_enabled\": false, \"installs_rollback_plan\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_storage_layout_snapshot(
) -> ModuleAuditRollbackStorageLayoutCandidate {
    ModuleAuditRollbackStorageLayoutCandidate {
        persistence_device_inventory: module_audit_rollback_current_boot_persistence_device_fact(),
        audit_rollback_storage_layout: module_audit_rollback_missing_storage_layout_fact(),
    }
}

pub(crate) fn module_audit_rollback_current_boot_persistence_device_fact(
) -> ModuleAuditRollbackPersistenceDeviceFact {
    match pci::find_mass_storage_controller() {
        Some(controller) => {
            let ahci_probe = ahci::probe(controller);
            ModuleAuditRollbackPersistenceDeviceFact {
                present: true,
                schema_ok: true,
                scope: "current_boot",
                provenance_ok: true,
                classification: "local_only",
                storage_controller_observed: true,
                controller_bus: controller.address.bus,
                controller_device: controller.address.device,
                controller_function: controller.address.function,
                controller_vendor_id: controller.vendor_id,
                controller_device_id: controller.device_id,
                controller_subclass: controller.subclass,
                controller_prog_if: controller.prog_if,
                ahci_probe,
                block_driver_available: ahci_probe.block_driver_available,
                stable_identity: true,
                block_device_identity_available: ahci_probe.block_device_identity_available,
                sector_read_available: ahci_probe.sector_read_available,
                partition_inventory_available: ahci_probe.partition_inventory_available,
                write_path_available: ahci_probe.write_path_available,
            }
        }
        None => module_audit_rollback_missing_persistence_device_fact(),
    }
}

pub(crate) fn module_audit_rollback_missing_persistence_device_fact(
) -> ModuleAuditRollbackPersistenceDeviceFact {
    ModuleAuditRollbackPersistenceDeviceFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        storage_controller_observed: false,
        controller_bus: 0,
        controller_device: 0,
        controller_function: 0,
        controller_vendor_id: 0,
        controller_device_id: 0,
        controller_subclass: 0,
        controller_prog_if: 0,
        ahci_probe: ahci::AhciReadOnlyProbe::missing("ahci_controller_not_observed"),
        block_driver_available: false,
        stable_identity: false,
        block_device_identity_available: false,
        sector_read_available: false,
        partition_inventory_available: false,
        write_path_available: false,
    }
}

pub(crate) fn module_audit_rollback_available_persistence_device_fact(
) -> ModuleAuditRollbackPersistenceDeviceFact {
    ModuleAuditRollbackPersistenceDeviceFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        storage_controller_observed: true,
        controller_bus: 0,
        controller_device: 1,
        controller_function: 1,
        controller_vendor_id: 0x8086,
        controller_device_id: 0x7010,
        controller_subclass: 0x01,
        controller_prog_if: 0x80,
        ahci_probe: ahci::AhciReadOnlyProbe::available_for_selftest(),
        block_driver_available: true,
        stable_identity: true,
        block_device_identity_available: true,
        sector_read_available: true,
        partition_inventory_available: true,
        write_path_available: true,
    }
}

pub(crate) fn module_audit_rollback_missing_storage_layout_fact(
) -> ModuleAuditRollbackStorageLayoutFact {
    ModuleAuditRollbackStorageLayoutFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_persistence_device: false,
        has_audit_ledger_region: false,
        has_rollback_store_region: false,
        append_slots_available: false,
        recovery_region_separated: false,
    }
}

pub(crate) fn module_audit_rollback_available_storage_layout_fact(
) -> ModuleAuditRollbackStorageLayoutFact {
    ModuleAuditRollbackStorageLayoutFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_persistence_device: true,
        has_audit_ledger_region: true,
        has_rollback_store_region: true,
        append_slots_available: true,
        recovery_region_separated: true,
    }
}

pub(crate) fn evaluate_module_audit_rollback_storage_layout_candidate(
    candidate: ModuleAuditRollbackStorageLayoutCandidate,
) -> ModuleAuditRollbackStorageLayoutEvaluation {
    let (persistence_device_status, persistence_device_reason) =
        evaluate_module_persistence_device_fact(candidate.persistence_device_inventory);
    let (storage_layout_status, storage_layout_reason) =
        evaluate_module_storage_layout_fact(candidate.audit_rollback_storage_layout);

    let (status, reason) = if method_eq(persistence_device_status, "rejected") {
        ("rejected", persistence_device_reason)
    } else if method_eq(storage_layout_status, "rejected") {
        ("rejected", storage_layout_reason)
    } else if method_eq(persistence_device_status, "missing")
        && method_eq(
            persistence_device_reason,
            "persistence_device_inventory_missing",
        )
        && method_eq(storage_layout_status, "missing")
        && method_eq(
            storage_layout_reason,
            "audit_rollback_storage_layout_missing",
        )
    {
        (
            "missing",
            "persistence_device_inventory_missing_and_storage_layout_missing",
        )
    } else if method_eq(persistence_device_status, "missing") {
        ("missing", persistence_device_reason)
    } else if method_eq(storage_layout_status, "missing") {
        ("missing", storage_layout_reason)
    } else {
        ("available", "audit_rollback_storage_layout_available")
    };

    let persistence_device_available = method_eq(persistence_device_status, "available");
    let storage_layout_available =
        persistence_device_available && method_eq(storage_layout_status, "available");
    let block_write_path_available = candidate.persistence_device_inventory.write_path_available;
    let block_write_path_reason = if block_write_path_available {
        "block_write_path_available"
    } else if !candidate
        .persistence_device_inventory
        .storage_controller_observed
    {
        "storage_controller_unobserved"
    } else if candidate
        .persistence_device_inventory
        .ahci_probe
        .registers_mapped
        && !candidate
            .persistence_device_inventory
            .block_device_identity_available
    {
        "block_device_identity_missing"
    } else if !candidate.persistence_device_inventory.sector_read_available {
        "persistence_sector_read_path_missing"
    } else if !candidate
        .persistence_device_inventory
        .partition_inventory_available
    {
        "persistence_partition_inventory_missing"
    } else if !candidate
        .persistence_device_inventory
        .block_driver_available
    {
        "persistence_block_driver_missing"
    } else {
        "persistence_device_write_path_missing"
    };
    ModuleAuditRollbackStorageLayoutEvaluation {
        status,
        reason,
        persistence_device_status,
        persistence_device_reason,
        storage_layout_status,
        storage_layout_reason,
        storage_controller_observed: candidate
            .persistence_device_inventory
            .storage_controller_observed,
        ahci_register_probe_available: candidate
            .persistence_device_inventory
            .ahci_probe
            .registers_mapped,
        block_driver_available: candidate
            .persistence_device_inventory
            .block_driver_available,
        block_device_identity_available: candidate
            .persistence_device_inventory
            .block_device_identity_available,
        sector_read_available: candidate.persistence_device_inventory.sector_read_available,
        partition_inventory_available: candidate
            .persistence_device_inventory
            .partition_inventory_available,
        block_write_path_available,
        block_write_path_reason,
        persistence_device_available,
        storage_layout_available,
        append_engine_available: false,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_persistence_device_fact(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", "persistence_device_scope_must_be_current_boot");
    }
    if !fact.schema_ok {
        return ("rejected", "persistence_device_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "persistence_device_inventory_missing");
    }
    if !fact.provenance_ok {
        return ("rejected", "persistence_device_provenance_missing");
    }
    if !fact.stable_identity {
        return ("rejected", "persistence_device_stable_identity_missing");
    }
    if fact.ahci_probe.registers_mapped && !fact.block_device_identity_available {
        return ("missing", "block_device_identity_missing");
    }
    if !fact.sector_read_available {
        return ("missing", "persistence_sector_read_path_missing");
    }
    if !fact.partition_inventory_available {
        return ("missing", "persistence_partition_inventory_missing");
    }
    if !fact.block_driver_available {
        return ("missing", "persistence_block_driver_missing");
    }
    if !fact.write_path_available {
        return ("missing", "persistence_device_write_path_missing");
    }
    ("available", "persistence_device_inventory_available")
}

pub(crate) fn evaluate_module_storage_layout_fact(
    fact: ModuleAuditRollbackStorageLayoutFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return (
            "rejected",
            "audit_rollback_storage_layout_scope_must_be_current_boot",
        );
    }
    if !fact.schema_ok {
        return ("rejected", "audit_rollback_storage_layout_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "audit_rollback_storage_layout_missing");
    }
    if !fact.provenance_ok {
        return (
            "rejected",
            "audit_rollback_storage_layout_provenance_missing",
        );
    }
    if !fact.binds_persistence_device {
        return ("rejected", "storage_layout_device_binding_missing");
    }
    if !fact.has_audit_ledger_region {
        return ("missing", "audit_ledger_layout_region_missing");
    }
    if !fact.has_rollback_store_region {
        return ("missing", "rollback_store_layout_region_missing");
    }
    if !fact.append_slots_available {
        return ("missing", "storage_layout_append_slots_missing");
    }
    if !fact.recovery_region_separated {
        return ("rejected", "storage_layout_recovery_boundary_missing");
    }
    ("available", "audit_rollback_storage_layout_available")
}

pub(crate) fn emit_module_storage_layout_facts(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_facts\": {");
    emit_module_persistence_device_fact(
        storage.persistence_device_inventory,
        evaluation.persistence_device_status,
        evaluation.persistence_device_reason,
        true,
    );
    emit_module_storage_layout_fact(
        storage.audit_rollback_storage_layout,
        evaluation.storage_layout_status,
        evaluation.storage_layout_reason,
        false,
    );
    raw_line("      }");
}

pub(crate) fn emit_module_storage_authority_foundation(
    persistence_device: ModuleAuditRollbackPersistenceDeviceFact,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    let target_region_discovery = audit_rollback_target_region_discovery(persistence_device);
    raw_line("      \"storage_authority_foundation\": {");
    raw("        \"schema\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("        \"owner_method\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"persistence\": \"none\",");
    raw("        \"status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw_line("        \"authorizes_append\": false,");
    raw_line("        \"writes_enabled\": false,");
    raw_line("        \"write_attempted\": false,");
    raw_line("        \"append_targets\": {");
    raw("          \"audit_ledger\": {\"id\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID);
    raw(", \"schema\": ");
    json_str(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA);
    raw(", \"available\": false},");
    crlf();
    raw("          \"rollback_store\": {\"id\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID);
    raw(", \"schema\": ");
    json_str(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA);
    raw(", \"available\": false}");
    crlf();
    raw_line("        },");
    raw_line("        \"required_authorities\": {");
    raw_line("          \"persistence_device_inventory\": true,");
    raw_line("          \"audit_rollback_storage_layout\": true,");
    raw_line("          \"append_engine\": true,");
    raw("          \"transaction_writer_owner\": ");
    json_str(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER);
    crlf();
    raw_line("        },");
    raw_line("        \"observed_boot_storage\": {");
    raw("          \"persistence_device_available\": ");
    raw_bool(evaluation.persistence_device_available);
    raw_line(",");
    raw("          \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("          \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("          \"storage_controller_observed\": ");
    raw_bool(evaluation.storage_controller_observed);
    raw_line(",");
    raw("          \"block_driver_available\": ");
    raw_bool(evaluation.block_driver_available);
    raw_line(",");
    raw("          \"ahci_register_probe_available\": ");
    raw_bool(evaluation.ahci_register_probe_available);
    raw_line(",");
    raw("          \"block_device_identity_available\": ");
    raw_bool(evaluation.block_device_identity_available);
    raw_line(",");
    raw("          \"sector_read_available\": ");
    raw_bool(evaluation.sector_read_available);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(evaluation.partition_inventory_available);
    raw_line(",");
    raw("          \"block_write_path_available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("          \"scratch_write_path_available\": ");
    raw_bool(persistence_device.ahci_probe.scratch_write_path_available);
    raw_line(",");
    raw("          \"scratch_block_write_authority_available\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .scratch_write_readback
            .block_write_authority_available,
    );
    raw_line(",");
    raw("          \"scratch_region_within_device_bounds\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .scratch_write_readback
            .region_within_device_bounds,
    );
    raw_line(",");
    raw("          \"scratch_region_no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .scratch_write_readback
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw("          \"scratch_region_authorizes_audit_rollback\": ");
    raw_bool(false);
    raw_line(",");
    raw("          \"audit_rollback_target_region_available\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .audit_rollback_target_region
            .available,
    );
    raw_line(",");
    raw("          \"audit_rollback_target_region_within_device_bounds\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .audit_rollback_target_region
            .region_within_device_bounds,
    );
    raw_line(",");
    raw("          \"audit_rollback_target_region_no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        persistence_device
            .ahci_probe
            .audit_rollback_target_region
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw("          \"block_write_path_reason\": ");
    json_str(evaluation.block_write_path_reason);
    crlf();
    raw_line("        },");
    emit_audit_rollback_target_region_discovery(target_region_discovery);
    raw_line(",");
    emit_module_block_write_path_authority_gate(persistence_device, evaluation);
    raw_line("      }");
}

pub(crate) fn emit_audit_rollback_target_region_discovery(
    discovery: AuditRollbackTargetRegionDiscovery,
) {
    raw_line("        \"audit_rollback_target_region_discovery\": {");
    raw("          \"schema\": ");
    json_str(discovery.schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(discovery.id);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw_line("          \"persistence\": \"none\",");
    raw("          \"status\": ");
    json_str(discovery.status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(discovery.reason);
    raw_line(",");
    raw("          \"source\": ");
    json_str(discovery.source);
    raw_line(",");
    raw("          \"storage_authority_id\": ");
    json_str(discovery.storage_authority_id);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(discovery.partition_inventory_available);
    raw_line(",");
    raw("          \"partition_inventory_scheme\": ");
    json_str(discovery.partition_inventory_scheme);
    raw_line(",");
    raw("          \"partition_inventory_source_lba\": ");
    raw_fmt(format_args!("{}", discovery.partition_inventory_source_lba));
    raw_line(",");
    raw("          \"partition_entry_count\": ");
    raw_fmt(format_args!("{}", discovery.partition_entry_count));
    raw_line(",");
    raw("          \"mbr_signature_valid\": ");
    raw_bool(discovery.mbr_signature_valid);
    raw_line(",");
    raw("          \"boot_metadata_lba\": ");
    raw_fmt(format_args!("{}", discovery.boot_metadata_lba));
    raw_line(",");
    raw("          \"candidate_region_present\": ");
    raw_bool(discovery.candidate_region_present);
    raw_line(",");
    raw("          \"candidate_region_start_lba\": ");
    raw_fmt(format_args!("{}", discovery.candidate_region_start_lba));
    raw_line(",");
    raw("          \"candidate_region_lba_count\": ");
    raw_fmt(format_args!("{}", discovery.candidate_region_lba_count));
    raw_line(",");
    raw("          \"candidate_region_is_scratch\": ");
    raw_bool(discovery.candidate_region_is_scratch);
    raw_line(",");
    raw("          \"candidate_overlaps_boot_metadata\": ");
    raw_bool(discovery.candidate_overlaps_boot_metadata);
    raw_line(",");
    raw("          \"candidate_overlaps_scratch\": ");
    raw_bool(discovery.candidate_overlaps_scratch);
    raw_line(",");
    raw("          \"scratch_region_id\": ");
    json_str(discovery.scratch_region_id);
    raw_line(",");
    raw("          \"scratch_region_available\": ");
    raw_bool(discovery.scratch_region_available);
    raw_line(",");
    raw("          \"scratch_region_start_lba\": ");
    raw_fmt(format_args!("{}", discovery.scratch_region_start_lba));
    raw_line(",");
    raw("          \"scratch_region_lba_count\": ");
    raw_fmt(format_args!("{}", discovery.scratch_region_lba_count));
    raw_line(",");
    raw("          \"scratch_rejected_as_durable_authority\": ");
    raw_bool(discovery.scratch_rejected_as_durable_authority);
    raw_line(",");
    raw("          \"durable_region_available\": ");
    raw_bool(discovery.durable_region_available);
    raw_line(",");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"writes_durable_audit_log\": false,");
    raw_line("          \"writes_rollback_store\": false,");
    raw_line("          \"write_attempted\": false");
    raw_line("        }");
}

pub(crate) fn emit_module_block_write_path_authority_gate(
    persistence_device: ModuleAuditRollbackPersistenceDeviceFact,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    let probe = persistence_device.ahci_probe;
    raw_line("        \"block_write_path_authority_gate\": {");
    raw("          \"schema\": ");
    json_str(AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw("          \"storage_authority_id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID);
    raw_line(",");
    raw("          \"status\": ");
    json_str(audit_rollback_block_write_path_gate_status(
        evaluation.block_write_path_available,
    ));
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.block_write_path_reason);
    raw_line(",");
    raw("          \"available\": ");
    raw_bool(evaluation.block_write_path_available);
    raw_line(",");
    raw("          \"read_only_block_driver_id\": ");
    json_str(probe.block_driver.driver_id);
    raw_line(",");
    raw("          \"read_only_block_driver_available\": ");
    raw_bool(probe.block_driver.available);
    raw_line(",");
    raw("          \"read_only_block_driver_supports_read\": ");
    raw_bool(probe.block_driver.supports_read);
    raw_line(",");
    raw("          \"read_only_block_driver_supports_write\": ");
    raw_bool(probe.block_driver.supports_write);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(probe.partition_inventory.available);
    raw_line(",");
    raw("          \"partition_inventory_scheme\": ");
    json_str(probe.partition_inventory.scheme);
    raw_line(",");
    raw("          \"scratch_write_path_available\": ");
    raw_bool(probe.scratch_write_path_available);
    raw_line(",");
    raw("          \"scratch_block_write_authority_available\": ");
    raw_bool(probe.scratch_write_readback.block_write_authority_available);
    raw_line(",");
    raw("          \"scratch_block_write_authority_id\": ");
    json_str(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID);
    raw_line(",");
    raw("          \"scratch_region_within_device_bounds\": ");
    raw_bool(probe.scratch_write_readback.region_within_device_bounds);
    raw_line(",");
    raw("          \"scratch_region_no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        probe
            .scratch_write_readback
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw("          \"scratch_region_id\": ");
    json_str(probe.scratch_write_readback.region_id);
    raw_line(",");
    raw("          \"scratch_region_status\": ");
    json_str(probe.scratch_write_readback.status);
    raw_line(",");
    raw("          \"scratch_region_reason\": ");
    json_str(probe.scratch_write_readback.reason);
    raw_line(",");
    raw_line("          \"scratch_region_authorizes_audit_rollback\": false,");
    raw_line("          \"requires_media_write_path\": true,");
    raw_line("          \"authorizes_media_write\": false,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"writes_enabled\": false,");
    raw_line("          \"write_attempted\": false");
    raw_line("        }");
}

pub(crate) fn emit_module_persistence_device_fact(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw_line("        \"persistence_device_inventory\": {");
    raw_line("          \"schema\": \"raios.persistence_device_inventory.v0\",");
    raw_line("          \"id\": \"storage.persistence_device_inventory.current_boot\",");
    raw_line("          \"device_class\": \"persistence_device\",");
    raw_line("          \"device_id\": null,");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw_line("          \"boot_storage_probe\": {");
    raw_line("            \"schema\": \"raios.pci_mass_storage_controller_probe.v0\",");
    raw_line("            \"scope\": \"current_boot\",");
    raw_line("            \"classification\": \"local_only\",");
    raw_line("            \"source\": \"pci_config_space\",");
    raw("            \"observed\": ");
    raw_bool(fact.storage_controller_observed);
    raw_line(",");
    raw("            \"controller_address\": ");
    if fact.storage_controller_observed {
        raw("\"");
        raw_fmt(format_args!(
            "{:02x}:{:02x}.{}",
            fact.controller_bus, fact.controller_device, fact.controller_function
        ));
        raw("\"");
    } else {
        raw("null");
    }
    raw_line(",");
    raw("            \"pci_class\": ");
    raw_fmt(format_args!(
        "{}",
        if fact.storage_controller_observed {
            1
        } else {
            0
        }
    ));
    raw_line(",");
    raw("            \"pci_subclass\": ");
    raw_fmt(format_args!("{}", fact.controller_subclass));
    raw_line(",");
    raw("            \"pci_prog_if\": ");
    raw_fmt(format_args!("{}", fact.controller_prog_if));
    raw_line(",");
    raw("            \"vendor_id\": ");
    raw_fmt(format_args!("{}", fact.controller_vendor_id));
    raw_line(",");
    raw("            \"device_id\": ");
    raw_fmt(format_args!("{}", fact.controller_device_id));
    raw_line(",");
    raw("            \"block_driver_available\": ");
    raw_bool(fact.block_driver_available);
    raw_line(",");
    raw("            \"write_path_available\": ");
    raw_bool(fact.write_path_available);
    raw_line(",");
    raw_line("            \"write_attempted\": false");
    raw_line("          },");
    emit_module_ahci_controller_probe(fact.ahci_probe);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"stable_identity\": ");
    raw_bool(fact.stable_identity);
    raw_line(",");
    raw("          \"block_device_identity_available\": ");
    raw_bool(fact.block_device_identity_available);
    raw_line(",");
    raw("          \"sector_read_available\": ");
    raw_bool(fact.sector_read_available);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(fact.partition_inventory_available);
    raw_line(",");
    raw("          \"write_path_available\": ");
    raw_bool(fact.write_path_available);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_layout\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_storage_layout\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_module_ahci_controller_probe(probe: ahci::AhciReadOnlyProbe) {
    raw_line("          \"ahci_controller_probe\": {");
    raw_line("            \"schema\": \"raios.ahci_controller_probe.v0\",");
    raw_line("            \"scope\": \"current_boot\",");
    raw_line("            \"classification\": \"local_only\",");
    raw_line("            \"source\": \"ahci_abar_mmio_identify_sector0_partition_read_only_scratch_write_readback_target_label_scan\",");
    raw("            \"observed\": ");
    raw_bool(probe.observed);
    raw_line(",");
    raw("            \"controller_is_ahci\": ");
    raw_bool(probe.controller_is_ahci);
    raw_line(",");
    raw("            \"abar_available\": ");
    raw_bool(probe.abar_available);
    raw_line(",");
    raw("            \"abar_base\": ");
    raw_fmt(format_args!("{}", probe.abar_base));
    raw_line(",");
    raw("            \"abar_size\": ");
    raw_fmt(format_args!("{}", probe.abar_size));
    raw_line(",");
    raw("            \"registers_mapped\": ");
    raw_bool(probe.registers_mapped);
    raw_line(",");
    raw("            \"ahci_version\": ");
    raw_fmt(format_args!("{}", probe.version));
    raw_line(",");
    raw("            \"capabilities\": ");
    raw_fmt(format_args!("{}", probe.capabilities));
    raw_line(",");
    raw("            \"ports_implemented_mask\": ");
    raw_fmt(format_args!("{}", probe.ports_implemented_mask));
    raw_line(",");
    raw("            \"implemented_port_count\": ");
    raw_fmt(format_args!("{}", probe.implemented_port_count));
    raw_line(",");
    raw("            \"command_slots\": ");
    raw_fmt(format_args!("{}", probe.command_slots));
    raw_line(",");
    raw("            \"supports_64bit\": ");
    raw_bool(probe.supports_64bit);
    raw_line(",");
    raw("            \"supports_ncq\": ");
    raw_bool(probe.supports_ncq);
    raw_line(",");
    raw_line("            \"first_port\": {");
    raw("              \"index\": ");
    if probe.first_port_implemented {
        raw_fmt(format_args!("{}", probe.first_port_index));
    } else {
        raw("null");
    }
    raw_line(",");
    raw("              \"implemented\": ");
    raw_bool(probe.first_port_implemented);
    raw_line(",");
    raw("              \"device_present\": ");
    raw_bool(probe.first_port_device_present);
    raw_line(",");
    raw("              \"interface_active\": ");
    raw_bool(probe.first_port_interface_active);
    raw_line(",");
    raw("              \"signature\": ");
    raw_fmt(format_args!("{}", probe.first_port_signature));
    raw_line(",");
    raw("              \"ssts\": ");
    raw_fmt(format_args!("{}", probe.first_port_ssts));
    raw_line(",");
    raw("              \"tfd\": ");
    raw_fmt(format_args!("{}", probe.first_port_tfd));
    raw_line(",");
    raw("              \"cmd\": ");
    raw_fmt(format_args!("{}", probe.first_port_cmd));
    crlf();
    raw_line("            },");
    raw("            \"block_device_identity_available\": ");
    raw_bool(probe.block_device_identity_available);
    raw_line(",");
    raw("            \"sector_read_available\": ");
    raw_bool(probe.sector_read_available);
    raw_line(",");
    raw("            \"partition_inventory_available\": ");
    raw_bool(probe.partition_inventory_available);
    raw_line(",");
    raw("            \"block_driver_available\": ");
    raw_bool(probe.block_driver_available);
    raw_line(",");
    raw("            \"write_path_available\": ");
    raw_bool(probe.write_path_available);
    raw_line(",");
    raw("            \"identify_command_issued\": ");
    raw_bool(probe.identify_command_issued);
    raw_line(",");
    raw("            \"controller_register_write_attempted\": ");
    raw_bool(probe.controller_register_write_attempted);
    raw_line(",");
    raw("            \"dma_started\": ");
    raw_bool(probe.dma_started);
    raw_line(",");
    raw("            \"identify_completed\": ");
    raw_bool(probe.identify_completed);
    raw_line(",");
    raw("            \"identify_status\": ");
    json_str(probe.identify_status);
    raw_line(",");
    raw("            \"sector_read_attempted\": ");
    raw_bool(probe.sector_read_attempted);
    raw_line(",");
    raw("            \"sector_read_completed\": ");
    raw_bool(probe.sector_read_completed);
    raw_line(",");
    raw("            \"sector_read_status\": ");
    json_str(probe.sector_read_status);
    raw_line(",");
    raw_line("            \"block_device_identity\": {");
    raw("              \"available\": ");
    raw_bool(probe.identity.available);
    raw_line(",");
    raw("              \"logical_sector_size_bytes\": ");
    raw_fmt(format_args!("{}", probe.identity.logical_sector_size_bytes));
    raw_line(",");
    raw("              \"lba28_sector_count\": ");
    raw_fmt(format_args!("{}", probe.identity.lba28_sector_count));
    raw_line(",");
    raw("              \"lba48_sector_count\": ");
    raw_fmt(format_args!("{}", probe.identity.lba48_sector_count));
    raw_line(",");
    raw("              \"serial\": ");
    emit_trimmed_ata_json_string(&probe.identity.serial);
    raw_line(",");
    raw("              \"firmware\": ");
    emit_trimmed_ata_json_string(&probe.identity.firmware);
    raw_line(",");
    raw("              \"model\": ");
    emit_trimmed_ata_json_string(&probe.identity.model);
    crlf();
    raw_line("            },");
    raw_line("            \"sector0_read\": {");
    raw("              \"available\": ");
    raw_bool(probe.sector0.available);
    raw_line(",");
    raw("              \"lba\": ");
    raw_fmt(format_args!("{}", probe.sector0.lba));
    raw_line(",");
    raw("              \"byte_count\": ");
    raw_fmt(format_args!("{}", probe.sector0.byte_count));
    raw_line(",");
    raw("              \"mbr_signature\": ");
    raw_fmt(format_args!("{}", probe.sector0.mbr_signature));
    raw_line(",");
    raw("              \"mbr_signature_valid\": ");
    raw_bool(probe.sector0.mbr_signature == 0xaa55);
    raw_line(",");
    raw("              \"first16_hex\": ");
    emit_hex_json_string(&probe.sector0.first16);
    crlf();
    raw_line("            },");
    raw_line("            \"partition_inventory\": {");
    raw_line("              \"schema\": \"raios.boot_partition_inventory.v0\",");
    raw_line("              \"scope\": \"current_boot\",");
    raw_line("              \"classification\": \"local_only\",");
    raw_line("              \"source\": \"sector0_mbr_partition_table\",");
    raw("              \"available\": ");
    raw_bool(probe.partition_inventory.available);
    raw_line(",");
    raw("              \"source_lba\": ");
    raw_fmt(format_args!("{}", probe.partition_inventory.source_lba));
    raw_line(",");
    raw("              \"scheme\": ");
    json_str(probe.partition_inventory.scheme);
    raw_line(",");
    raw("              \"mbr_signature_valid\": ");
    raw_bool(probe.partition_inventory.mbr_signature_valid);
    raw_line(",");
    raw("              \"entry_count\": ");
    raw_fmt(format_args!("{}", probe.partition_inventory.entry_count));
    raw_line(",");
    raw("              \"bootable_entry_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.partition_inventory.bootable_entry_count
    ));
    raw_line(",");
    raw_line("              \"entries\": [");
    let mut partition_entry_idx = 0usize;
    while partition_entry_idx < probe.partition_inventory.entries.len() {
        emit_module_ahci_partition_entry(
            partition_entry_idx,
            probe.partition_inventory.entries[partition_entry_idx],
            partition_entry_idx + 1 != probe.partition_inventory.entries.len(),
        );
        partition_entry_idx += 1;
    }
    raw_line("              ],");
    raw_line("              \"authorizes_partition_write\": false,");
    raw_line("              \"writes_enabled\": false,");
    raw_line("              \"write_attempted\": false");
    raw_line("            },");
    raw_line("            \"scratch_block_region\": {");
    raw("              \"schema\": ");
    json_str(ahci::SCRATCH_BLOCK_REGION_SCHEMA);
    raw_line(",");
    raw("              \"id\": ");
    json_str(probe.scratch_write_readback.region_id);
    raw_line(",");
    raw_line("              \"scope\": \"current_boot\",");
    raw_line("              \"classification\": \"local_only\",");
    raw_line("              \"test_infrastructure\": true,");
    raw_line("              \"persistence\": \"none\",");
    raw_line("              \"source\": \"vm_harness_labeled_ahci_scratch_region\",");
    raw("              \"available\": ");
    raw_bool(probe.scratch_write_readback.available);
    raw_line(",");
    raw("              \"status\": ");
    json_str(probe.scratch_write_readback.status);
    raw_line(",");
    raw("              \"reason\": ");
    json_str(probe.scratch_write_readback.reason);
    raw_line(",");
    raw("              \"marker\": ");
    json_str("RAIOS_SCRATCH_V0");
    raw_line(",");
    raw("              \"marker_lba\": ");
    raw_fmt(format_args!("{}", probe.scratch_write_readback.marker_lba));
    raw_line(",");
    raw("              \"test_lba\": ");
    raw_fmt(format_args!("{}", probe.scratch_write_readback.test_lba));
    raw_line(",");
    raw("              \"region_start_lba\": ");
    raw_fmt(format_args!(
        "{}",
        probe.scratch_write_readback.region_start_lba
    ));
    raw_line(",");
    raw("              \"region_lba_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.scratch_write_readback.region_lba_count
    ));
    raw_line(",");
    raw("              \"byte_count\": ");
    raw_fmt(format_args!("{}", probe.scratch_write_readback.byte_count));
    raw_line(",");
    raw("              \"port_index\": ");
    raw_fmt(format_args!("{}", probe.scratch_write_readback.port_index));
    raw_line(",");
    raw("              \"device_identity_available\": ");
    raw_bool(probe.scratch_write_readback.device_identity.available);
    raw_line(",");
    raw("              \"logical_sector_size_bytes\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .scratch_write_readback
            .device_identity
            .logical_sector_size_bytes
    ));
    raw_line(",");
    raw("              \"lba28_sector_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .scratch_write_readback
            .device_identity
            .lba28_sector_count
    ));
    raw_line(",");
    raw("              \"lba48_sector_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .scratch_write_readback
            .device_identity
            .lba48_sector_count
    ));
    raw_line(",");
    raw("              \"serial\": ");
    emit_trimmed_ata_json_string(&probe.scratch_write_readback.device_identity.serial);
    raw_line(",");
    raw("              \"model\": ");
    emit_trimmed_ata_json_string(&probe.scratch_write_readback.device_identity.model);
    raw_line(",");
    raw("              \"label_found\": ");
    raw_bool(probe.scratch_write_readback.label_found);
    raw_line(",");
    raw("              \"region_within_device_bounds\": ");
    raw_bool(probe.scratch_write_readback.region_within_device_bounds);
    raw_line(",");
    raw("              \"boot_port_overlap\": ");
    raw_bool(probe.scratch_write_readback.boot_port_overlap);
    raw_line(",");
    raw("              \"metadata_lba_overlap\": ");
    raw_bool(probe.scratch_write_readback.metadata_lba_overlap);
    raw_line(",");
    raw("              \"no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        probe
            .scratch_write_readback
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw("              \"block_write_authority_available\": ");
    raw_bool(probe.scratch_write_readback.block_write_authority_available);
    raw_line(",");
    raw("              \"write_attempted\": ");
    raw_bool(probe.scratch_write_readback.write_attempted);
    raw_line(",");
    raw("              \"write_completed\": ");
    raw_bool(probe.scratch_write_readback.write_completed);
    raw_line(",");
    raw("              \"readback_completed\": ");
    raw_bool(probe.scratch_write_readback.readback_completed);
    raw_line(",");
    raw("              \"readback_matches\": ");
    raw_bool(probe.scratch_write_readback.readback_matches);
    raw_line(",");
    raw("              \"readback_first16_hex\": ");
    emit_hex_json_string(&probe.scratch_write_readback.readback_first16);
    raw_line(",");
    raw_line("              \"authorizes_audit_rollback\": false,");
    raw_line("              \"authorizes_append\": false,");
    raw("              \"test_write_exercised\": ");
    raw_bool(probe.scratch_write_readback.write_attempted);
    raw_line(",");
    raw_line("              \"writes_enabled\": false");
    raw_line("            },");
    raw_line("            \"scratch_block_write_authority\": {");
    raw("              \"schema\": ");
    json_str(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_SCHEMA);
    raw_line(",");
    raw("              \"id\": ");
    json_str(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID);
    raw_line(",");
    raw_line("              \"scope\": \"current_boot\",");
    raw_line("              \"classification\": \"local_only\",");
    raw_line("              \"test_infrastructure\": true,");
    raw_line("              \"persistence\": \"none\",");
    raw("              \"source_region_id\": ");
    json_str(probe.scratch_write_readback.region_id);
    raw_line(",");
    raw_line("              \"owner\": \"vm_harness.scratch_region.current_boot\",");
    raw("              \"available\": ");
    raw_bool(probe.scratch_write_readback.block_write_authority_available);
    raw_line(",");
    raw("              \"status\": ");
    json_str(probe.scratch_write_readback.status);
    raw_line(",");
    raw("              \"reason\": ");
    json_str(probe.scratch_write_readback.reason);
    raw_line(",");
    raw("              \"region_start_lba\": ");
    raw_fmt(format_args!(
        "{}",
        probe.scratch_write_readback.region_start_lba
    ));
    raw_line(",");
    raw("              \"region_lba_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.scratch_write_readback.region_lba_count
    ));
    raw_line(",");
    raw("              \"byte_count\": ");
    raw_fmt(format_args!("{}", probe.scratch_write_readback.byte_count));
    raw_line(",");
    raw("              \"device_identity_available\": ");
    raw_bool(probe.scratch_write_readback.device_identity.available);
    raw_line(",");
    raw("              \"region_within_device_bounds\": ");
    raw_bool(probe.scratch_write_readback.region_within_device_bounds);
    raw_line(",");
    raw("              \"no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        probe
            .scratch_write_readback
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw("              \"authorizes_scratch_write_readback\": ");
    raw_bool(probe.scratch_write_readback.block_write_authority_available);
    raw_line(",");
    raw_line("              \"authorizes_audit_rollback\": false,");
    raw_line("              \"authorizes_append\": false,");
    raw_line("              \"writes_enabled\": false");
    raw_line("            },");
    raw_line("            \"audit_rollback_target_region\": {");
    raw("              \"schema\": ");
    json_str(AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SCHEMA);
    raw_line(",");
    raw("              \"id\": ");
    json_str(probe.audit_rollback_target_region.region_id);
    raw_line(",");
    raw_line("              \"scope\": \"current_boot\",");
    raw_line("              \"classification\": \"local_only\",");
    raw_line("              \"test_infrastructure\": true,");
    raw_line("              \"persistence\": \"none\",");
    raw_line("              \"source\": \"vm_harness_labeled_ahci_audit_rollback_target_region\",");
    raw("              \"available\": ");
    raw_bool(probe.audit_rollback_target_region.available);
    raw_line(",");
    raw("              \"status\": ");
    json_str(probe.audit_rollback_target_region.status);
    raw_line(",");
    raw("              \"reason\": ");
    json_str(probe.audit_rollback_target_region.reason);
    raw_line(",");
    raw("              \"marker\": ");
    json_str(ahci::AUDIT_ROLLBACK_TARGET_REGION_MARKER);
    raw_line(",");
    raw("              \"marker_lba\": ");
    raw_fmt(format_args!(
        "{}",
        probe.audit_rollback_target_region.marker_lba
    ));
    raw_line(",");
    raw("              \"region_start_lba\": ");
    raw_fmt(format_args!(
        "{}",
        probe.audit_rollback_target_region.region_start_lba
    ));
    raw_line(",");
    raw("              \"region_lba_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.audit_rollback_target_region.region_lba_count
    ));
    raw_line(",");
    raw("              \"byte_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.audit_rollback_target_region.byte_count
    ));
    raw_line(",");
    raw("              \"port_index\": ");
    raw_fmt(format_args!(
        "{}",
        probe.audit_rollback_target_region.port_index
    ));
    raw_line(",");
    raw("              \"device_identity_available\": ");
    raw_bool(probe.audit_rollback_target_region.device_identity.available);
    raw_line(",");
    raw("              \"logical_sector_size_bytes\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .audit_rollback_target_region
            .device_identity
            .logical_sector_size_bytes
    ));
    raw_line(",");
    raw("              \"lba28_sector_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .audit_rollback_target_region
            .device_identity
            .lba28_sector_count
    ));
    raw_line(",");
    raw("              \"lba48_sector_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe
            .audit_rollback_target_region
            .device_identity
            .lba48_sector_count
    ));
    raw_line(",");
    raw("              \"serial\": ");
    emit_trimmed_ata_json_string(&probe.audit_rollback_target_region.device_identity.serial);
    raw_line(",");
    raw("              \"model\": ");
    emit_trimmed_ata_json_string(&probe.audit_rollback_target_region.device_identity.model);
    raw_line(",");
    raw("              \"label_found\": ");
    raw_bool(probe.audit_rollback_target_region.label_found);
    raw_line(",");
    raw("              \"read_attempted\": ");
    raw_bool(probe.audit_rollback_target_region.read_attempted);
    raw_line(",");
    raw("              \"read_completed\": ");
    raw_bool(probe.audit_rollback_target_region.read_completed);
    raw_line(",");
    raw("              \"read_first16_hex\": ");
    emit_hex_json_string(&probe.audit_rollback_target_region.read_first16);
    raw_line(",");
    raw("              \"region_within_device_bounds\": ");
    raw_bool(
        probe
            .audit_rollback_target_region
            .region_within_device_bounds,
    );
    raw_line(",");
    raw("              \"boot_port_overlap\": ");
    raw_bool(probe.audit_rollback_target_region.boot_port_overlap);
    raw_line(",");
    raw("              \"metadata_lba_overlap\": ");
    raw_bool(probe.audit_rollback_target_region.metadata_lba_overlap);
    raw_line(",");
    raw("              \"scratch_region_overlap\": ");
    raw_bool(probe.audit_rollback_target_region.scratch_region_overlap);
    raw_line(",");
    raw("              \"no_boot_or_partition_metadata_overlap\": ");
    raw_bool(
        probe
            .audit_rollback_target_region
            .no_boot_or_partition_metadata_overlap,
    );
    raw_line(",");
    raw_line("              \"read_only\": true,");
    raw_line("              \"authorizes_audit_rollback\": false,");
    raw_line("              \"authorizes_append\": false,");
    raw_line("              \"writes_enabled\": false,");
    raw_line("              \"write_attempted\": false");
    raw_line("            },");
    raw_line("            \"block_driver\": {");
    raw_line("              \"schema\": \"raios.read_only_block_driver.v0\",");
    raw_line("              \"scope\": \"current_boot\",");
    raw_line("              \"classification\": \"local_only\",");
    raw("              \"id\": ");
    json_str(probe.block_driver.driver_id);
    raw_line(",");
    raw("              \"source\": ");
    json_str(probe.block_driver.source);
    raw_line(",");
    raw("              \"available\": ");
    raw_bool(probe.block_driver.available);
    raw_line(",");
    raw("              \"logical_sector_size_bytes\": ");
    raw_fmt(format_args!(
        "{}",
        probe.block_driver.logical_sector_size_bytes
    ));
    raw_line(",");
    raw("              \"lba28_sector_count\": ");
    raw_fmt(format_args!("{}", probe.block_driver.lba28_sector_count));
    raw_line(",");
    raw("              \"lba48_sector_count\": ");
    raw_fmt(format_args!("{}", probe.block_driver.lba48_sector_count));
    raw_line(",");
    raw("              \"verified_read_lba\": ");
    raw_fmt(format_args!("{}", probe.block_driver.verified_read_lba));
    raw_line(",");
    raw("              \"verified_read_byte_count\": ");
    raw_fmt(format_args!(
        "{}",
        probe.block_driver.verified_read_byte_count
    ));
    raw_line(",");
    raw("              \"binds_partition_inventory\": ");
    raw_bool(probe.block_driver.binds_partition_inventory);
    raw_line(",");
    raw("              \"supports_read\": ");
    raw_bool(probe.block_driver.supports_read);
    raw_line(",");
    raw("              \"supports_write\": ");
    raw_bool(probe.block_driver.supports_write);
    raw_line(",");
    raw_line("              \"writes_enabled\": false,");
    raw_line("              \"write_attempted\": false");
    raw_line("            },");
    raw("            \"write_attempted\": ");
    raw_bool(probe.media_write_attempted);
    raw_line(",");
    raw("            \"reason\": ");
    json_str(probe.reason);
    crlf();
    raw_line("          }");
}

fn emit_module_ahci_partition_entry(
    slot: usize,
    entry: ahci::AhciPartitionEntryEvidence,
    comma: bool,
) {
    raw("                {\"slot\": ");
    raw_fmt(format_args!("{}", slot));
    raw(", \"available\": ");
    raw_bool(entry.available);
    raw(", \"boot_indicator\": ");
    raw_fmt(format_args!("{}", entry.boot_indicator));
    raw(", \"partition_type\": ");
    raw_fmt(format_args!("{}", entry.partition_type));
    raw(", \"first_lba\": ");
    raw_fmt(format_args!("{}", entry.first_lba));
    raw(", \"sector_count\": ");
    raw_fmt(format_args!("{}", entry.sector_count));
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_hex_json_string(bytes: &[u8]) {
    raw("\"");
    let mut idx = 0usize;
    while idx < bytes.len() {
        raw_fmt(format_args!("{:02x}", bytes[idx]));
        idx += 1;
    }
    raw("\"");
}

fn emit_trimmed_ata_json_string(bytes: &[u8]) {
    let mut start = 0usize;
    let mut end = bytes.len();
    while start < end && (bytes[start] == 0 || bytes[start] == b' ') {
        start += 1;
    }
    while end > start && (bytes[end - 1] == 0 || bytes[end - 1] == b' ') {
        end -= 1;
    }

    raw("\"");
    let mut idx = start;
    while idx < end {
        match bytes[idx] {
            b'"' => raw("\\\""),
            b'\\' => raw("\\\\"),
            0x20..=0x7e => raw_fmt(format_args!("{}", bytes[idx] as char)),
            _ => raw(" "),
        }
        idx += 1;
    }
    raw("\"");
}

pub(crate) fn emit_module_storage_layout_fact(
    fact: ModuleAuditRollbackStorageLayoutFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw_line("        \"audit_rollback_storage_layout\": {");
    raw("          \"schema\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_LAYOUT_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(AUDIT_ROLLBACK_STORAGE_LAYOUT_ID);
    raw_line(",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_persistence_device\": ");
    raw_bool(fact.binds_persistence_device);
    raw_line(",");
    raw("          \"has_audit_ledger_region\": ");
    raw_bool(fact.has_audit_ledger_region);
    raw_line(",");
    raw("          \"has_rollback_store_region\": ");
    raw_bool(fact.has_rollback_store_region);
    raw_line(",");
    raw("          \"append_slots_available\": ");
    raw_bool(fact.append_slots_available);
    raw_line(",");
    raw("          \"recovery_region_separated\": ");
    raw_bool(fact.recovery_region_separated);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_storage_layout\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_storage_layout_selftest_cases(
) -> [ModuleAuditRollbackStorageLayoutSelfTestCase;
       MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_SELFTEST_CASES] {
    let missing = ModuleAuditRollbackStorageLayoutCandidate {
        persistence_device_inventory: module_audit_rollback_missing_persistence_device_fact(),
        audit_rollback_storage_layout: module_audit_rollback_missing_storage_layout_fact(),
    };
    let available_device = module_audit_rollback_available_persistence_device_fact();
    let available_layout = module_audit_rollback_available_storage_layout_fact();
    [
        module_audit_rollback_storage_layout_selftest_case(
            "missing_storage_inputs_current_boot",
            "missing",
            "persistence_device_inventory_missing_and_storage_layout_missing",
            missing,
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_previous_boot",
            "rejected",
            "persistence_device_scope_must_be_current_boot",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    scope: "previous_boot",
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_wrong_schema",
            "rejected",
            "persistence_device_schema_mismatch",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    schema_ok: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_provenance_missing",
            "rejected",
            "persistence_device_provenance_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    provenance_ok: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_stable_identity_missing",
            "rejected",
            "persistence_device_stable_identity_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    stable_identity: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "block_device_identity_missing",
            "missing",
            "block_device_identity_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    block_driver_available: false,
                    block_device_identity_available: false,
                    sector_read_available: false,
                    partition_inventory_available: false,
                    write_path_available: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_block_driver_missing",
            "missing",
            "persistence_block_driver_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    ahci_probe: ahci::AhciReadOnlyProbe::missing("ahci_register_probe_unavailable"),
                    block_driver_available: false,
                    sector_read_available: true,
                    partition_inventory_available: true,
                    write_path_available: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_sector_read_path_missing",
            "missing",
            "persistence_sector_read_path_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    block_driver_available: false,
                    block_device_identity_available: true,
                    sector_read_available: false,
                    partition_inventory_available: false,
                    write_path_available: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_partition_inventory_missing",
            "missing",
            "persistence_partition_inventory_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    partition_inventory_available: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_previous_boot",
            "rejected",
            "audit_rollback_storage_layout_scope_must_be_current_boot",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    scope: "previous_boot",
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_wrong_schema",
            "rejected",
            "audit_rollback_storage_layout_schema_mismatch",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    schema_ok: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_provenance_missing",
            "rejected",
            "audit_rollback_storage_layout_provenance_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    provenance_ok: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_device_binding_missing",
            "rejected",
            "storage_layout_device_binding_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    binds_persistence_device: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_ledger_layout_region_missing",
            "missing",
            "audit_ledger_layout_region_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    has_audit_ledger_region: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "rollback_store_layout_region_missing",
            "missing",
            "rollback_store_layout_region_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    has_rollback_store_region: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_append_slots_missing",
            "missing",
            "storage_layout_append_slots_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    append_slots_available: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_recovery_boundary_missing",
            "rejected",
            "storage_layout_recovery_boundary_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    recovery_region_separated: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "available_storage_layout_still_non_authorizing",
            "available",
            "audit_rollback_storage_layout_available",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: available_layout,
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_storage_layout_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackStorageLayoutCandidate,
) -> ModuleAuditRollbackStorageLayoutSelfTestCase {
    let actual = evaluate_module_audit_rollback_storage_layout_candidate(candidate);
    ModuleAuditRollbackStorageLayoutSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.writes_enabled
            && !actual.installs_rollback_plan
            && !actual.can_load
            && !actual.load_attempted,
    }
}
