use alloc::{format, string::String, vec, vec::Vec};

use crate::agent_protocol_support::{
    emit_record_fields_trailing_comma, emit_record_property_line, record_bool as b,
    record_false as no, record_field as f, record_null as null, record_object as object,
    record_str as s, record_str_or_null,
};
use crate::{agent_protocol_module_types::*, agent_protocol_support::*, ahci, pci};
use raios_core::record::Value as V;

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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_storage_layout.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", no()),
            f("global_event_log_mutation", s("none")),
            f("writes_enabled", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        6,
    );
    emit_module_storage_layout_facts(storage, evaluation);
    raw_line(",");
    emit_module_storage_authority_foundation(storage.persistence_device_inventory, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            f(
                "storage_authority_id",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID),
            ),
            f(
                "storage_authority_owner",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER),
            ),
            f(
                "audit_append_target_id",
                s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID),
            ),
            f(
                "rollback_append_target_id",
                s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID),
            ),
            f(
                "transaction_writer_owner",
                s(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER),
            ),
            f("storage_layout_status", s(evaluation.status)),
            f("storage_layout_reason", s(evaluation.reason)),
            f(
                "persistence_device_available",
                b(evaluation.persistence_device_available),
            ),
            f(
                "storage_layout_available",
                b(evaluation.storage_layout_available),
            ),
            f(
                "storage_layout_missing",
                b(!evaluation.storage_layout_available),
            ),
            f(
                "append_engine_available",
                b(evaluation.append_engine_available),
            ),
            f(
                "append_engine_missing",
                b(!evaluation.append_engine_available),
            ),
            f("retained_hash_refs_are_storage_authority", no()),
            f("availability_facts_are_storage_authority", no()),
            f("write_policy_facts_are_storage_authority", no()),
            f("storage_layout_facts_are_append_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_storage_layout_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
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
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("passed", b(case.passed)),
            f("writes_enabled", no()),
            f("installs_rollback_plan", no()),
            f("can_load", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_SCHEMA)),
            f("id", s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID)),
            f("owner_method", s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_OWNER)),
            f(
                "source_method",
                s(MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_METHOD),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("persistence", s("none")),
            f("status", s(evaluation.status)),
            f("reason", s(evaluation.reason)),
            f("available", b(evaluation.storage_layout_available)),
            f("authorizes_append", no()),
            f("writes_enabled", no()),
            f("write_attempted", no()),
            f(
                "append_targets",
                object(vec![
                    f(
                        "audit_ledger",
                        record_inline(vec![
                            f("id", s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_ID)),
                            f("schema", s(AUDIT_ROLLBACK_AUDIT_APPEND_TARGET_SCHEMA)),
                            f("available", no()),
                        ]),
                    ),
                    f(
                        "rollback_store",
                        record_inline(vec![
                            f("id", s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID)),
                            f("schema", s(AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_SCHEMA)),
                            f("available", no()),
                        ]),
                    ),
                ]),
            ),
            f(
                "required_authorities",
                object(vec![
                    f("persistence_device_inventory", b(true)),
                    f("audit_rollback_storage_layout", b(true)),
                    f("append_engine", b(true)),
                    f(
                        "transaction_writer_owner",
                        s(AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER),
                    ),
                ]),
            ),
            f(
                "observed_boot_storage",
                object(vec![
                    f(
                        "persistence_device_available",
                        b(evaluation.persistence_device_available),
                    ),
                    f(
                        "storage_layout_available",
                        b(evaluation.storage_layout_available),
                    ),
                    f(
                        "append_engine_available",
                        b(evaluation.append_engine_available),
                    ),
                    f(
                        "storage_controller_observed",
                        b(evaluation.storage_controller_observed),
                    ),
                    f(
                        "block_driver_available",
                        b(evaluation.block_driver_available),
                    ),
                    f(
                        "ahci_register_probe_available",
                        b(evaluation.ahci_register_probe_available),
                    ),
                    f(
                        "block_device_identity_available",
                        b(evaluation.block_device_identity_available),
                    ),
                    f("sector_read_available", b(evaluation.sector_read_available)),
                    f(
                        "partition_inventory_available",
                        b(evaluation.partition_inventory_available),
                    ),
                    f(
                        "block_write_path_available",
                        b(evaluation.block_write_path_available),
                    ),
                    f(
                        "scratch_write_path_available",
                        b(persistence_device.ahci_probe.scratch_write_path_available),
                    ),
                    f(
                        "scratch_block_write_authority_available",
                        b(persistence_device
                            .ahci_probe
                            .scratch_write_readback
                            .block_write_authority_available),
                    ),
                    f(
                        "scratch_region_within_device_bounds",
                        b(persistence_device
                            .ahci_probe
                            .scratch_write_readback
                            .region_within_device_bounds),
                    ),
                    f(
                        "scratch_region_no_boot_or_partition_metadata_overlap",
                        b(persistence_device
                            .ahci_probe
                            .scratch_write_readback
                            .no_boot_or_partition_metadata_overlap),
                    ),
                    f("scratch_region_authorizes_audit_rollback", no()),
                    f(
                        "audit_rollback_target_region_available",
                        b(persistence_device
                            .ahci_probe
                            .audit_rollback_target_region
                            .available),
                    ),
                    f(
                        "audit_rollback_target_region_within_device_bounds",
                        b(persistence_device
                            .ahci_probe
                            .audit_rollback_target_region
                            .region_within_device_bounds),
                    ),
                    f(
                        "audit_rollback_target_region_no_boot_or_partition_metadata_overlap",
                        b(persistence_device
                            .ahci_probe
                            .audit_rollback_target_region
                            .no_boot_or_partition_metadata_overlap),
                    ),
                    f(
                        "block_write_path_reason",
                        s(evaluation.block_write_path_reason),
                    ),
                ]),
            ),
        ],
        8,
    );
    emit_audit_rollback_target_region_discovery(target_region_discovery);
    raw_line(",");
    emit_module_block_write_path_authority_gate(persistence_device, evaluation);
    raw_line("      }");
}

pub(crate) fn emit_audit_rollback_target_region_discovery(
    discovery: AuditRollbackTargetRegionDiscovery,
) {
    emit_record_property_line_at(
        "audit_rollback_target_region_discovery",
        vec![
            f("schema", s(discovery.schema)),
            f("id", s(discovery.id)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("persistence", s("none")),
            f("status", s(discovery.status)),
            f("reason", s(discovery.reason)),
            f("source", s(discovery.source)),
            f("storage_authority_id", s(discovery.storage_authority_id)),
            f(
                "partition_inventory_available",
                b(discovery.partition_inventory_available),
            ),
            f(
                "partition_inventory_scheme",
                s(discovery.partition_inventory_scheme),
            ),
            f(
                "partition_inventory_source_lba",
                V::U64(discovery.partition_inventory_source_lba),
            ),
            f(
                "partition_entry_count",
                V::U64(discovery.partition_entry_count as u64),
            ),
            f("mbr_signature_valid", b(discovery.mbr_signature_valid)),
            f("boot_metadata_lba", V::U64(discovery.boot_metadata_lba)),
            f(
                "candidate_region_present",
                b(discovery.candidate_region_present),
            ),
            f(
                "candidate_region_start_lba",
                V::U64(discovery.candidate_region_start_lba),
            ),
            f(
                "candidate_region_lba_count",
                V::U64(discovery.candidate_region_lba_count),
            ),
            f(
                "candidate_region_is_scratch",
                b(discovery.candidate_region_is_scratch),
            ),
            f(
                "candidate_overlaps_boot_metadata",
                b(discovery.candidate_overlaps_boot_metadata),
            ),
            f(
                "candidate_overlaps_scratch",
                b(discovery.candidate_overlaps_scratch),
            ),
            f("scratch_region_id", s(discovery.scratch_region_id)),
            f(
                "scratch_region_available",
                b(discovery.scratch_region_available),
            ),
            f(
                "scratch_region_start_lba",
                V::U64(discovery.scratch_region_start_lba),
            ),
            f(
                "scratch_region_lba_count",
                V::U64(discovery.scratch_region_lba_count),
            ),
            f(
                "scratch_rejected_as_durable_authority",
                b(discovery.scratch_rejected_as_durable_authority),
            ),
            f(
                "durable_region_available",
                b(discovery.durable_region_available),
            ),
            f("authorizes_append", no()),
            f("writes_durable_audit_log", no()),
            f("writes_rollback_store", no()),
            f("write_attempted", no()),
        ],
        8,
        false,
    );
}

pub(crate) fn emit_module_block_write_path_authority_gate(
    persistence_device: ModuleAuditRollbackPersistenceDeviceFact,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    let probe = persistence_device.ahci_probe;
    emit_record_property_line_at(
        "block_write_path_authority_gate",
        vec![
            f(
                "schema",
                s(AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_SCHEMA),
            ),
            f("id", s(AUDIT_ROLLBACK_BLOCK_WRITE_PATH_AUTHORITY_GATE_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f(
                "storage_authority_id",
                s(AUDIT_ROLLBACK_STORAGE_AUTHORITY_ID),
            ),
            f(
                "status",
                s(audit_rollback_block_write_path_gate_status(
                    evaluation.block_write_path_available,
                )),
            ),
            f("reason", s(evaluation.block_write_path_reason)),
            f("available", b(evaluation.block_write_path_available)),
            f("read_only_block_driver_id", s(probe.block_driver.driver_id)),
            f(
                "read_only_block_driver_available",
                b(probe.block_driver.available),
            ),
            f(
                "read_only_block_driver_supports_read",
                b(probe.block_driver.supports_read),
            ),
            f(
                "read_only_block_driver_supports_write",
                b(probe.block_driver.supports_write),
            ),
            f(
                "partition_inventory_available",
                b(probe.partition_inventory.available),
            ),
            f(
                "partition_inventory_scheme",
                s(probe.partition_inventory.scheme),
            ),
            f(
                "scratch_write_path_available",
                b(probe.scratch_write_path_available),
            ),
            f(
                "scratch_block_write_authority_available",
                b(probe.scratch_write_readback.block_write_authority_available),
            ),
            f(
                "scratch_block_write_authority_id",
                s(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID),
            ),
            f(
                "scratch_region_within_device_bounds",
                b(probe.scratch_write_readback.region_within_device_bounds),
            ),
            f(
                "scratch_region_no_boot_or_partition_metadata_overlap",
                b(probe
                    .scratch_write_readback
                    .no_boot_or_partition_metadata_overlap),
            ),
            f(
                "scratch_region_id",
                s(probe.scratch_write_readback.region_id),
            ),
            f(
                "scratch_region_status",
                s(probe.scratch_write_readback.status),
            ),
            f(
                "scratch_region_reason",
                s(probe.scratch_write_readback.reason),
            ),
            f("scratch_region_authorizes_audit_rollback", no()),
            f("requires_media_write_path", b(true)),
            f("authorizes_media_write", no()),
            f("authorizes_append", no()),
            f("writes_enabled", no()),
            f("write_attempted", no()),
        ],
        8,
        false,
    );
}

pub(crate) fn emit_module_persistence_device_fact(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    let controller_address: Option<String> = if fact.storage_controller_observed {
        Some(format!(
            "{:02x}:{:02x}.{}",
            fact.controller_bus, fact.controller_device, fact.controller_function
        ))
    } else {
        None
    };

    emit_record_property_line_at(
        "persistence_device_inventory",
        vec![
            f("schema", s("raios.persistence_device_inventory.v0")),
            f("id", s("storage.persistence_device_inventory.current_boot")),
            f("device_class", s("persistence_device")),
            f("device_id", null()),
            f("scope", s(fact.scope)),
            f("classification", s(fact.classification)),
            f("status", s(status)),
            f("reason", s(reason)),
            f("present", b(fact.present)),
            f("schema_valid", b(fact.schema_ok)),
            f(
                "boot_storage_probe",
                module_boot_storage_probe_record(fact, controller_address.as_deref()),
            ),
            f(
                "ahci_controller_probe",
                module_ahci_controller_probe_record(&fact.ahci_probe),
            ),
            f("provenance_valid", b(fact.provenance_ok)),
            f("stable_identity", b(fact.stable_identity)),
            f(
                "block_device_identity_available",
                b(fact.block_device_identity_available),
            ),
            f("sector_read_available", b(fact.sector_read_available)),
            f(
                "partition_inventory_available",
                b(fact.partition_inventory_available),
            ),
            f("write_path_available", b(fact.write_path_available)),
            f("authority", s("current_snapshot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("authorizes_layout", no()),
            f("write_attempted", no()),
            f(
                "provenance",
                object(vec![
                    f("source_method", s("module.audit_rollback_storage_layout")),
                    f("source_transport", s("serial-console")),
                    f("event_scope", s("current_boot")),
                    f("record_id", null()),
                ]),
            ),
        ],
        8,
        comma,
    );
}

fn module_boot_storage_probe_record<'a>(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
    controller_address: Option<&'a str>,
) -> V<'a> {
    object(vec![
        f("schema", s("raios.pci_mass_storage_controller_probe.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("source", s("pci_config_space")),
        f("observed", b(fact.storage_controller_observed)),
        f("controller_address", record_str_or_null(controller_address)),
        f(
            "pci_class",
            V::U64(if fact.storage_controller_observed {
                1
            } else {
                0
            }),
        ),
        f("pci_subclass", V::U64(fact.controller_subclass as u64)),
        f("pci_prog_if", V::U64(fact.controller_prog_if as u64)),
        f("vendor_id", V::U64(fact.controller_vendor_id as u64)),
        f("device_id", V::U64(fact.controller_device_id as u64)),
        f("block_driver_available", b(fact.block_driver_available)),
        f("write_path_available", b(fact.write_path_available)),
        f("write_attempted", no()),
    ])
}

fn module_ahci_controller_probe_record(probe: &ahci::AhciReadOnlyProbe) -> V<'_> {
    let mut partition_entries = Vec::new();
    let mut idx = 0usize;
    while idx < probe.partition_inventory.entries.len() {
        partition_entries.push(module_ahci_partition_entry_record(
            idx,
            probe.partition_inventory.entries[idx],
        ));
        idx += 1;
    }

    object(vec![
        f("schema", s("raios.ahci_controller_probe.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "source",
            s("ahci_abar_mmio_identify_sector0_partition_read_only_scratch_write_readback_target_label_scan"),
        ),
        f("observed", b(probe.observed)),
        f("controller_is_ahci", b(probe.controller_is_ahci)),
        f("abar_available", b(probe.abar_available)),
        f("abar_base", V::U64(probe.abar_base)),
        f("abar_size", V::U64(probe.abar_size)),
        f("registers_mapped", b(probe.registers_mapped)),
        f("ahci_version", V::U64(probe.version as u64)),
        f("capabilities", V::U64(probe.capabilities as u64)),
        f("ports_implemented_mask", V::U64(probe.ports_implemented_mask as u64)),
        f("implemented_port_count", V::U64(probe.implemented_port_count as u64)),
        f("command_slots", V::U64(probe.command_slots as u64)),
        f("supports_64bit", b(probe.supports_64bit)),
        f("supports_ncq", b(probe.supports_ncq)),
        f(
            "first_port",
            object(vec![
                f(
                    "index",
                    if probe.first_port_implemented {
                        V::U64(probe.first_port_index as u64)
                    } else {
                        null()
                    },
                ),
                f("implemented", b(probe.first_port_implemented)),
                f("device_present", b(probe.first_port_device_present)),
                f("interface_active", b(probe.first_port_interface_active)),
                f("signature", V::U64(probe.first_port_signature as u64)),
                f("ssts", V::U64(probe.first_port_ssts as u64)),
                f("tfd", V::U64(probe.first_port_tfd as u64)),
                f("cmd", V::U64(probe.first_port_cmd as u64)),
            ]),
        ),
        f(
            "block_device_identity_available",
            b(probe.block_device_identity_available),
        ),
        f("sector_read_available", b(probe.sector_read_available)),
        f(
            "partition_inventory_available",
            b(probe.partition_inventory_available),
        ),
        f("block_driver_available", b(probe.block_driver_available)),
        f("write_path_available", b(probe.write_path_available)),
        f("identify_command_issued", b(probe.identify_command_issued)),
        f(
            "controller_register_write_attempted",
            b(probe.controller_register_write_attempted),
        ),
        f("dma_started", b(probe.dma_started)),
        f("identify_completed", b(probe.identify_completed)),
        f("identify_status", s(probe.identify_status)),
        f("sector_read_attempted", b(probe.sector_read_attempted)),
        f("sector_read_completed", b(probe.sector_read_completed)),
        f("sector_read_status", s(probe.sector_read_status)),
        f(
            "block_device_identity",
            object(vec![
                f("available", b(probe.identity.available)),
                f(
                    "logical_sector_size_bytes",
                    V::U64(probe.identity.logical_sector_size_bytes as u64),
                ),
                f("lba28_sector_count", V::U64(probe.identity.lba28_sector_count as u64)),
                f("lba48_sector_count", V::U64(probe.identity.lba48_sector_count)),
                f("serial", V::TrimmedAsciiBytes(&probe.identity.serial)),
                f("firmware", V::TrimmedAsciiBytes(&probe.identity.firmware)),
                f("model", V::TrimmedAsciiBytes(&probe.identity.model)),
            ]),
        ),
        f(
            "sector0_read",
            object(vec![
                f("available", b(probe.sector0.available)),
                f("lba", V::U64(probe.sector0.lba)),
                f("byte_count", V::U64(probe.sector0.byte_count as u64)),
                f("mbr_signature", V::U64(probe.sector0.mbr_signature as u64)),
                f(
                    "mbr_signature_valid",
                    b(probe.sector0.mbr_signature == 0xaa55),
                ),
                f("first16_hex", V::HexBytes(&probe.sector0.first16)),
            ]),
        ),
        f(
            "partition_inventory",
            object(vec![
                f("schema", s("raios.boot_partition_inventory.v0")),
                f("scope", s("current_boot")),
                f("classification", s("local_only")),
                f("source", s("sector0_mbr_partition_table")),
                f("available", b(probe.partition_inventory.available)),
                f("source_lba", V::U64(probe.partition_inventory.source_lba)),
                f("scheme", s(probe.partition_inventory.scheme)),
                f(
                    "mbr_signature_valid",
                    b(probe.partition_inventory.mbr_signature_valid),
                ),
                f("entry_count", V::U64(probe.partition_inventory.entry_count as u64)),
                f(
                    "bootable_entry_count",
                    V::U64(probe.partition_inventory.bootable_entry_count as u64),
                ),
                f("entries", V::Array(partition_entries)),
                f("authorizes_partition_write", no()),
                f("writes_enabled", no()),
                f("write_attempted", no()),
            ]),
        ),
        f("scratch_block_region", scratch_block_region_record(probe)),
        f(
            "scratch_block_write_authority",
            scratch_block_write_authority_record(probe),
        ),
        f(
            "audit_rollback_target_region",
            audit_rollback_target_region_record(probe),
        ),
        f("block_driver", block_driver_record(probe)),
        f("write_attempted", b(probe.media_write_attempted)),
        f("reason", s(probe.reason)),
    ])
}

fn module_ahci_partition_entry_record(
    slot: usize,
    entry: ahci::AhciPartitionEntryEvidence,
) -> V<'static> {
    record_inline(vec![
        f("slot", V::U64(slot as u64)),
        f("available", b(entry.available)),
        f("boot_indicator", V::U64(entry.boot_indicator as u64)),
        f("partition_type", V::U64(entry.partition_type as u64)),
        f("first_lba", V::U64(entry.first_lba as u64)),
        f("sector_count", V::U64(entry.sector_count as u64)),
        f("authorizes_write", no()),
    ])
}

fn scratch_block_region_record(probe: &ahci::AhciReadOnlyProbe) -> V<'_> {
    let scratch = &probe.scratch_write_readback;
    object(vec![
        f("schema", s(ahci::SCRATCH_BLOCK_REGION_SCHEMA)),
        f("id", s(scratch.region_id)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("test_infrastructure", b(true)),
        f("persistence", s("none")),
        f("source", s("vm_harness_labeled_ahci_scratch_region")),
        f("available", b(scratch.available)),
        f("status", s(scratch.status)),
        f("reason", s(scratch.reason)),
        f("marker", s("RAIOS_SCRATCH_V0")),
        f("marker_lba", V::U64(scratch.marker_lba)),
        f("test_lba", V::U64(scratch.test_lba)),
        f("region_start_lba", V::U64(scratch.region_start_lba)),
        f("region_lba_count", V::U64(scratch.region_lba_count)),
        f("byte_count", V::U64(scratch.byte_count as u64)),
        f("port_index", V::U64(scratch.port_index as u64)),
        f(
            "device_identity_available",
            b(scratch.device_identity.available),
        ),
        f(
            "logical_sector_size_bytes",
            V::U64(scratch.device_identity.logical_sector_size_bytes as u64),
        ),
        f(
            "lba28_sector_count",
            V::U64(scratch.device_identity.lba28_sector_count as u64),
        ),
        f(
            "lba48_sector_count",
            V::U64(scratch.device_identity.lba48_sector_count),
        ),
        f(
            "serial",
            V::TrimmedAsciiBytes(&scratch.device_identity.serial),
        ),
        f(
            "model",
            V::TrimmedAsciiBytes(&scratch.device_identity.model),
        ),
        f("label_found", b(scratch.label_found)),
        f(
            "region_within_device_bounds",
            b(scratch.region_within_device_bounds),
        ),
        f("boot_port_overlap", b(scratch.boot_port_overlap)),
        f("metadata_lba_overlap", b(scratch.metadata_lba_overlap)),
        f(
            "no_boot_or_partition_metadata_overlap",
            b(scratch.no_boot_or_partition_metadata_overlap),
        ),
        f(
            "block_write_authority_available",
            b(scratch.block_write_authority_available),
        ),
        f("write_attempted", b(scratch.write_attempted)),
        f("write_completed", b(scratch.write_completed)),
        f("readback_completed", b(scratch.readback_completed)),
        f("readback_matches", b(scratch.readback_matches)),
        f(
            "readback_first16_hex",
            V::HexBytes(&scratch.readback_first16),
        ),
        f("authorizes_audit_rollback", no()),
        f("authorizes_append", no()),
        f("test_write_exercised", b(scratch.write_attempted)),
        f("writes_enabled", no()),
    ])
}

fn scratch_block_write_authority_record(probe: &ahci::AhciReadOnlyProbe) -> V<'_> {
    let scratch = &probe.scratch_write_readback;
    object(vec![
        f("schema", s(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_SCHEMA)),
        f("id", s(ahci::SCRATCH_BLOCK_WRITE_AUTHORITY_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("test_infrastructure", b(true)),
        f("persistence", s("none")),
        f("source_region_id", s(scratch.region_id)),
        f("owner", s("vm_harness.scratch_region.current_boot")),
        f("available", b(scratch.block_write_authority_available)),
        f("status", s(scratch.status)),
        f("reason", s(scratch.reason)),
        f("region_start_lba", V::U64(scratch.region_start_lba)),
        f("region_lba_count", V::U64(scratch.region_lba_count)),
        f("byte_count", V::U64(scratch.byte_count as u64)),
        f(
            "device_identity_available",
            b(scratch.device_identity.available),
        ),
        f(
            "region_within_device_bounds",
            b(scratch.region_within_device_bounds),
        ),
        f(
            "no_boot_or_partition_metadata_overlap",
            b(scratch.no_boot_or_partition_metadata_overlap),
        ),
        f(
            "authorizes_scratch_write_readback",
            b(scratch.block_write_authority_available),
        ),
        f("authorizes_audit_rollback", no()),
        f("authorizes_append", no()),
        f("writes_enabled", no()),
    ])
}

fn audit_rollback_target_region_record(probe: &ahci::AhciReadOnlyProbe) -> V<'_> {
    let target = &probe.audit_rollback_target_region;
    object(vec![
        f("schema", s(AUDIT_ROLLBACK_TARGET_REGION_DISCOVERY_SCHEMA)),
        f("id", s(target.region_id)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("test_infrastructure", b(true)),
        f("persistence", s("none")),
        f(
            "source",
            s("vm_harness_labeled_ahci_audit_rollback_target_region"),
        ),
        f("available", b(target.available)),
        f("status", s(target.status)),
        f("reason", s(target.reason)),
        f("marker", s(ahci::AUDIT_ROLLBACK_TARGET_REGION_MARKER)),
        f("marker_lba", V::U64(target.marker_lba)),
        f("region_start_lba", V::U64(target.region_start_lba)),
        f("region_lba_count", V::U64(target.region_lba_count)),
        f("byte_count", V::U64(target.byte_count as u64)),
        f("port_index", V::U64(target.port_index as u64)),
        f(
            "device_identity_available",
            b(target.device_identity.available),
        ),
        f(
            "logical_sector_size_bytes",
            V::U64(target.device_identity.logical_sector_size_bytes as u64),
        ),
        f(
            "lba28_sector_count",
            V::U64(target.device_identity.lba28_sector_count as u64),
        ),
        f(
            "lba48_sector_count",
            V::U64(target.device_identity.lba48_sector_count),
        ),
        f(
            "serial",
            V::TrimmedAsciiBytes(&target.device_identity.serial),
        ),
        f("model", V::TrimmedAsciiBytes(&target.device_identity.model)),
        f("label_found", b(target.label_found)),
        f("read_attempted", b(target.read_attempted)),
        f("read_completed", b(target.read_completed)),
        f("read_first16_hex", V::HexBytes(&target.read_first16)),
        f(
            "region_within_device_bounds",
            b(target.region_within_device_bounds),
        ),
        f("boot_port_overlap", b(target.boot_port_overlap)),
        f("metadata_lba_overlap", b(target.metadata_lba_overlap)),
        f("scratch_region_overlap", b(target.scratch_region_overlap)),
        f(
            "no_boot_or_partition_metadata_overlap",
            b(target.no_boot_or_partition_metadata_overlap),
        ),
        f("read_only", b(true)),
        f("authorizes_audit_rollback", no()),
        f("authorizes_append", no()),
        f("writes_enabled", no()),
        f("write_attempted", no()),
    ])
}

fn block_driver_record(probe: &ahci::AhciReadOnlyProbe) -> V<'_> {
    let block = &probe.block_driver;
    object(vec![
        f("schema", s("raios.read_only_block_driver.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("id", s(block.driver_id)),
        f("source", s(block.source)),
        f("available", b(block.available)),
        f(
            "logical_sector_size_bytes",
            V::U64(block.logical_sector_size_bytes as u64),
        ),
        f(
            "lba28_sector_count",
            V::U64(block.lba28_sector_count as u64),
        ),
        f("lba48_sector_count", V::U64(block.lba48_sector_count)),
        f("verified_read_lba", V::U64(block.verified_read_lba)),
        f(
            "verified_read_byte_count",
            V::U64(block.verified_read_byte_count as u64),
        ),
        f(
            "binds_partition_inventory",
            b(block.binds_partition_inventory),
        ),
        f("supports_read", b(block.supports_read)),
        f("supports_write", b(block.supports_write)),
        f("writes_enabled", no()),
        f("write_attempted", no()),
    ])
}

pub(crate) fn emit_module_storage_layout_fact(
    fact: ModuleAuditRollbackStorageLayoutFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_record_property_line_at(
        "audit_rollback_storage_layout",
        vec![
            f("schema", s(AUDIT_ROLLBACK_STORAGE_LAYOUT_SCHEMA)),
            f("id", s(AUDIT_ROLLBACK_STORAGE_LAYOUT_ID)),
            f("scope", s(fact.scope)),
            f("classification", s(fact.classification)),
            f("status", s(status)),
            f("reason", s(reason)),
            f("present", b(fact.present)),
            f("schema_valid", b(fact.schema_ok)),
            f("provenance_valid", b(fact.provenance_ok)),
            f("binds_persistence_device", b(fact.binds_persistence_device)),
            f("has_audit_ledger_region", b(fact.has_audit_ledger_region)),
            f(
                "has_rollback_store_region",
                b(fact.has_rollback_store_region),
            ),
            f("append_slots_available", b(fact.append_slots_available)),
            f(
                "recovery_region_separated",
                b(fact.recovery_region_separated),
            ),
            f("authority", s("current_snapshot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("authorizes_append", no()),
            f("write_attempted", no()),
            f(
                "provenance",
                object(vec![
                    f("source_method", s("module.audit_rollback_storage_layout")),
                    f("source_transport", s("serial-console")),
                    f("event_scope", s("current_boot")),
                    f("record_id", null()),
                ]),
            ),
        ],
        8,
        comma,
    );
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
