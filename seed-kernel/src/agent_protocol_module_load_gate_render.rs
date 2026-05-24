use crate::{
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_sha256, json_str, method_eq, raw, raw_line,
    },
    event_log, serial,
};
fn module_load_gate_manifest_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_manifest_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_module_manifest_reference_not_authorizing"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        binding.manifest_reference_reason
    } else {
        "module_manifest_missing"
    }
}

fn module_load_gate_candidate_artifact_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_candidate_artifact_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_candidate_artifact_reference_not_authorizing"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        binding.artifact_reference_reason
    } else {
        "candidate_artifact_missing"
    }
}

fn module_load_gate_vm_test_report_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_vm_test_report_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_vm_test_report_reference_not_authorizing"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        binding.vm_report_reference_reason
    } else {
        "vm_test_report_missing"
    }
}

fn module_load_gate_local_attestation_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_attestation_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_local_attestation_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_local_attestation_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_attestation_reference_valid(binding) {
        "retained_local_attestation_reference_not_authorizing"
    } else if module_load_gate_local_attestation_reference_rejected(binding) {
        binding.attestation_reference_reason
    } else {
        "local_attestation_missing"
    }
}

fn module_load_gate_local_approval_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_approval_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_local_approval_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_local_approval_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_approval_reference_valid(binding) {
        "retained_local_approval_reference_not_authorizing"
    } else if module_load_gate_local_approval_reference_rejected(binding) {
        binding.approval_reference_reason
    } else {
        "local_approval_missing"
    }
}

fn module_load_gate_computed_grant_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_hash_reference_only"
    } else {
        "missing"
    }
}

fn module_load_gate_computed_grant_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_computed_grant_reference_not_authorizing"
    } else {
        "computed_capability_grant_missing"
    }
}

fn module_load_gate_durable_audit_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_durable"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_durable_audit_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "durable_audit_write_missing"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "durable_audit_write_missing"
    }
}

fn module_load_gate_rollback_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_installed"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_rollback_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "rollback_install_missing"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "rollback_install_missing"
    }
}

fn module_load_gate_service_slot_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_hash_reference_only_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "unallocated"
    }
}

fn module_load_gate_service_slot_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_service_slot_reservation_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        binding.service_slot_reservation_reason
    } else {
        "ram_only_service_slot_unallocated"
    }
}

fn module_load_gate_retained_module_evidence_complete(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    module_load_gate_manifest_reference_valid(binding)
        && module_load_gate_candidate_artifact_reference_valid(binding)
        && module_load_gate_vm_test_report_reference_valid(binding)
        && module_load_gate_local_attestation_reference_valid(binding)
        && module_load_gate_local_approval_reference_valid(binding)
        && binding.retained_reference.is_some()
        && module_load_gate_audit_rollback_reference_valid(binding)
        && module_load_gate_service_slot_reservation_valid(binding)
}

fn module_load_gate_retained_module_evidence_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    module_load_gate_manifest_reference_rejected(binding)
        || module_load_gate_candidate_artifact_reference_rejected(binding)
        || module_load_gate_vm_test_report_reference_rejected(binding)
        || module_load_gate_local_attestation_reference_rejected(binding)
        || module_load_gate_local_approval_reference_rejected(binding)
        || module_load_gate_audit_rollback_reference_rejected(binding)
        || module_load_gate_service_slot_reservation_rejected(binding)
}

fn module_load_gate_retained_module_evidence_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        "available"
    } else if module_load_gate_retained_module_evidence_rejected(binding) {
        "rejected"
    } else {
        "missing"
    }
}

fn module_load_gate_retained_module_evidence_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if !module_load_gate_manifest_reference_valid(binding) {
        return module_load_gate_manifest_reason(binding);
    }
    if !module_load_gate_candidate_artifact_reference_valid(binding) {
        return module_load_gate_candidate_artifact_reason(binding);
    }
    if !module_load_gate_vm_test_report_reference_valid(binding) {
        return module_load_gate_vm_test_report_reason(binding);
    }
    if !module_load_gate_local_attestation_reference_valid(binding) {
        return module_load_gate_local_attestation_reason(binding);
    }
    if !module_load_gate_local_approval_reference_valid(binding) {
        return module_load_gate_local_approval_reason(binding);
    }
    if binding.retained_reference.is_none() {
        return module_load_gate_computed_grant_reason(binding);
    }
    if !module_load_gate_audit_rollback_reference_valid(binding) {
        return module_load_gate_rollback_reason(binding);
    }
    if !module_load_gate_service_slot_reservation_valid(binding) {
        return module_load_gate_service_slot_reason(binding);
    }
    "retained_module_evidence_available"
}

fn module_load_gate_service_slot_allocator_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "missing_runtime"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        "blocked_by_rejected_service_slot_reservation"
    } else {
        "blocked_by_service_slot_reservation"
    }
}

fn module_load_gate_service_slot_allocator_status(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "missing"
    } else {
        "blocked"
    }
}

fn module_load_gate_service_slot_allocator_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "service_slot_allocator_runtime_missing"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        binding.service_slot_reservation_reason
    } else {
        "retained_service_slot_reservation_missing"
    }
}

fn module_load_gate_loader_runtime_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if !module_load_gate_retained_module_evidence_complete(binding) {
        "blocked_by_retained_module_evidence"
    } else {
        "blocked_by_service_slot_allocator_runtime"
    }
}

fn module_load_gate_loader_runtime_status(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        "denied_missing_service_slot_allocator_runtime"
    } else {
        "denied_missing_retained_module_evidence"
    }
}

fn module_load_gate_loader_runtime_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        "service_slot_allocator_runtime_missing"
    } else {
        module_load_gate_retained_module_evidence_reason(binding)
    }
}

fn module_load_gate_audit_rollback_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.audit_rollback_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_audit_rollback_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.audit_rollback_reference_status, "rejected")
}

fn module_load_gate_service_slot_reservation_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.service_slot_reservation_status,
        "retained_hash_reference_only_not_allocated",
    )
}

fn module_load_gate_service_slot_reservation_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.service_slot_reservation_status, "rejected")
}

fn module_load_gate_manifest_reference_valid(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(
        binding.manifest_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_manifest_reference_rejected(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(binding.manifest_reference_status, "rejected")
}

fn module_load_gate_candidate_artifact_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.artifact_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_candidate_artifact_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.artifact_reference_status, "rejected")
}

fn module_load_gate_vm_test_report_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.vm_report_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_vm_test_report_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.vm_report_reference_status, "rejected")
}

fn module_load_gate_local_attestation_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.attestation_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_local_attestation_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.attestation_reference_status, "rejected")
}

fn module_load_gate_local_approval_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.approval_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_local_approval_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.approval_reference_status, "rejected")
}

fn emit_module_load_gate_manifest_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_module_manifest_reference\": {");
    if let Some(reference) = binding.manifest_reference {
        if module_load_gate_manifest_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.manifest_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.manifest_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.manifest_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.manifest_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw_line("      \"hashes\": {");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_manifest_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.manifest_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.manifest_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_artifact_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_candidate_artifact_reference\": {");
    if let Some(reference) = binding.artifact_reference {
        if module_load_gate_candidate_artifact_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.artifact_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.artifact_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.artifact_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.artifact_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_candidate_artifact_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.artifact_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.artifact_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_vm_report_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_vm_test_report_reference\": {");
    if let Some(reference) = binding.vm_report_reference {
        if module_load_gate_vm_test_report_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.vm_report_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.vm_report_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.vm_report_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.vm_report_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_manifest_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_vm_report_json\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_vm_test_report_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.vm_report_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.vm_report_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_local_attestation_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_local_attestation_reference\": {");
    if let Some(reference) = binding.attestation_reference {
        if module_load_gate_local_attestation_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.attestation_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_local_attestation_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.attestation_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.attestation_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.attestation_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_local_attestation_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_local_attestation_json\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("      \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_local_attestation_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.attestation_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.attestation_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_local_approval_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_local_approval_reference\": {");
    if let Some(reference) = binding.approval_reference {
        if module_load_gate_local_approval_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.approval_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_local_approval_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.approval_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.approval_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"authorizes_guest_load\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.approval_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_local_approval_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"accepts_local_approval_text\": false,");
        raw_line("      \"accepts_artifact_bytes\": false,");
        raw_line("      \"accepts_unsigned_service_code\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"service_inventory_change\": \"none\",");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw_line(",");
        raw("      \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw_line(",");
        raw("      \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw_line(",");
        raw("      \"retained_local_attestation_reference_event_id\": ");
        json_event_id(reference.retained_local_attestation_reference_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw_line(",");
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw_line(",");
        raw("        \"local_attestation_reference_hash\": ");
        json_sha256(reference.local_attestation_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
        raw("        \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_local_approval_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.approval_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.approval_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_retained_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_computed_grant_reference\": {");
    if let Some(reference) = binding.retained_reference {
        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.retained_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw_line("      \"hashes\": {");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_computed_grant_reference.v0\",");
        raw_line("      \"status\": \"missing\",");
        raw_line("      \"reason\": \"no_valid_computed_grant_reference_retained\",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_audit_rollback_reference(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_audit_rollback_reference\": {");
    if let Some(reference) = binding.audit_rollback_reference {
        if module_load_gate_audit_rollback_reference_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.audit_rollback_reference_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
            raw("      \"status\": ");
            json_str(binding.audit_rollback_reference_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.audit_rollback_reference_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"durable_audit_written\": false,");
            raw_line("      \"rollback_plan_installed\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.audit_rollback_reference_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"durable_audit_written\": false,");
        raw_line("      \"rollback_plan_installed\": false,");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw("      \"denial_event_id\": ");
        json_event_id(reference.denial_event_id);
        raw_line(",");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
        raw("        \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("        \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_audit_rollback_reference.v0\",");
        raw("      \"status\": ");
        json_str(binding.audit_rollback_reference_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.audit_rollback_reference_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_service_slot_reservation(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"retained_service_slot_reservation\": {");
    if let Some(reservation) = binding.service_slot_reservation {
        if module_load_gate_service_slot_reservation_rejected(binding) {
            raw_line("      \"state\": \"rejected\",");
            raw_line("      \"retention\": \"current_boot_ram_event_log\",");
            raw("      \"event_id\": ");
            json_event_id_option(binding.service_slot_reservation_event_id);
            raw_line(",");
            raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
            raw("      \"status\": ");
            json_str(binding.service_slot_reservation_status);
            raw_line(",");
            raw("      \"reason\": ");
            json_str(binding.service_slot_reservation_reason);
            raw_line(",");
            raw_line("      \"classification\": \"local_only\",");
            raw_line("      \"allocates_service_slot\": false,");
            raw_line("      \"creates_service_inventory_records\": false,");
            raw_line("      \"can_load_now\": false,");
            raw_line("      \"load_attempted\": false");
            raw("    }");
            return;
        }

        raw_line("      \"state\": \"present\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw("      \"event_id\": ");
        json_event_id_option(binding.service_slot_reservation_event_id);
        raw_line(",");
        raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw_line("      \"status\": \"retained_hash_reference_only_not_allocated\",");
        raw_line("      \"classification\": \"local_only\",");
        raw_line("      \"allocates_service_slot\": false,");
        raw_line("      \"creates_service_inventory_records\": false,");
        raw_line("      \"grants_capability\": false,");
        raw_line("      \"grants_load_now\": false,");
        raw_line("      \"authorizes_guest_load\": false,");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false,");
        raw("      \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reservation.retained_reference_event_id);
        raw_line(",");
        raw("      \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reservation.retained_audit_rollback_reference_event_id);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reservation.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("      \"hashes\": {");
        raw("        \"reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw_line(",");
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reservation.computed_grant_hash);
        raw_line(",");
        raw("        \"audit_record_hash\": ");
        json_sha256(reservation.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reservation.rollback_plan_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reservation.pre_load_service_inventory_hash);
        crlf();
        raw_line("      }");
    } else {
        raw_line("      \"state\": \"missing\",");
        raw_line("      \"retention\": \"current_boot_ram_event_log\",");
        raw_line("      \"event_id\": null,");
        raw_line("      \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw("      \"status\": ");
        json_str(binding.service_slot_reservation_status);
        raw_line(",");
        raw("      \"reason\": ");
        json_str(binding.service_slot_reservation_reason);
        raw_line(",");
        raw_line("      \"can_load_now\": false,");
        raw_line("      \"load_attempted\": false");
    }
    raw("    }");
}

fn emit_module_load_gate_service_slot_allocator_readiness(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw_line("    \"service_slot_allocator_readiness\": {");
    raw_line("      \"schema\": \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"source_method\": \"module.service_slot_allocator\",");
    raw("      \"state\": ");
    json_str(module_load_gate_service_slot_allocator_state(binding));
    raw_line(",");
    raw("      \"readiness_status\": ");
    json_str(module_load_gate_service_slot_allocator_status(binding));
    raw_line(",");
    raw("      \"readiness_reason\": ");
    json_str(module_load_gate_service_slot_allocator_reason(binding));
    raw_line(",");
    raw("      \"retained_service_slot_reservation_status\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw_line(",");
    raw("      \"retained_service_slot_reservation_reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw_line(",");
    raw_line("      \"service_slot_allocator_ready\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
}

fn emit_module_load_gate_loader_runtime_readiness(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"loader_runtime_readiness\": {");
    raw_line("      \"schema\": \"raios.module_loader_runtime_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"state\": ");
    json_str(module_load_gate_loader_runtime_state(binding));
    raw_line(",");
    raw("      \"readiness_status\": ");
    json_str(module_load_gate_loader_runtime_status(binding));
    raw_line(",");
    raw("      \"readiness_reason\": ");
    json_str(module_load_gate_loader_runtime_reason(binding));
    raw_line(",");
    raw("      \"retained_module_evidence_state\": ");
    json_str(module_load_gate_retained_module_evidence_state(binding));
    raw_line(",");
    raw("      \"retained_module_evidence_reason\": ");
    json_str(module_load_gate_retained_module_evidence_reason(binding));
    raw_line(",");
    raw_line("      \"service_slot_allocator_ready\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"loader_runtime_facts\": {");
    emit_module_load_gate_loader_runtime_fact(
        "loader_identity",
        "raios.module_loader_identity.v0",
        "module_loader_identity_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "artifact_hash_binding",
        "raios.module_loader_artifact_hash_binding.v0",
        "module_loader_artifact_hash_binding_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "entrypoint_abi",
        "raios.module_loader_entrypoint_abi.v0",
        "module_loader_entrypoint_abi_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "address_space_boundary",
        "raios.module_loader_address_space_boundary.v0",
        "module_loader_address_space_boundary_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "memory_map_constraints",
        "raios.module_loader_memory_map_constraints.v0",
        "module_loader_memory_map_constraints_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "capability_import_table",
        "raios.module_loader_capability_import_table.v0",
        "module_loader_capability_import_table_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "service_slot_binding",
        "raios.module_loader_service_slot_binding.v0",
        "module_loader_service_slot_binding_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "health_state_hooks",
        "raios.module_loader_health_state_hooks.v0",
        "module_loader_health_state_hooks_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "rollback_hooks",
        "raios.module_loader_rollback_hooks.v0",
        "module_loader_rollback_hooks_missing",
        true,
    );
    emit_module_load_gate_loader_runtime_fact(
        "audit_rollback_write_boundary_binding",
        "raios.module_loader_audit_rollback_write_boundary_binding.v0",
        "module_loader_audit_rollback_write_boundary_binding_missing",
        false,
    );
    raw_line("      }");
    raw_line("    }");
}

fn emit_module_load_gate_loader_runtime_fact(
    name: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw_line("          \"status\": \"missing\",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw_line("          \"present\": false,");
    raw_line("          \"authorizes_load\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_load_gate_evidence_hashes(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("      \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
    } else {
        raw_line("      \"computed_capability_grant_hash\": null,");
    }
    if let Some(reference) = binding
        .attestation_reference
        .filter(|_| module_load_gate_local_attestation_reference_valid(binding))
    {
        raw("      \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw_line(",");
        raw("      \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
    } else {
        raw_line("      \"local_attestation_reference_hash\": null,");
        raw_line("      \"local_attestation_hash\": null,");
    }
    if let Some(reference) = binding
        .approval_reference
        .filter(|_| module_load_gate_local_approval_reference_valid(binding))
    {
        raw("      \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw_line(",");
        raw("      \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
    } else {
        raw_line("      \"local_approval_reference_hash\": null,");
        raw_line("      \"local_approval_hash\": null,");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw("      \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("      \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
    } else {
        raw_line("      \"vm_test_report_reference_hash\": null,");
        raw_line("      \"vm_test_report_hash\": null,");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw("      \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("      \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
    } else {
        raw_line("      \"artifact_reference_hash\": null,");
        raw_line("      \"artifact_hash\": null,");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw("      \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("      \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
    } else {
        raw_line("      \"manifest_reference_hash\": null,");
        raw_line("      \"manifest_hash\": null,");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw("      \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("      \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("      \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("      \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw_line(",");
        raw("      \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
    } else {
        raw_line("      \"audit_record_hash\": null,");
        raw_line("      \"rollback_plan_hash\": null,");
        raw_line("      \"pre_load_service_inventory_hash\": null,");
        raw_line("      \"cleanup_actions_hash\": null,");
        raw_line("      \"ram_only_service_slot_id\": null,");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw("      \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw_line(",");
    } else {
        raw_line("      \"service_slot_reservation_hash\": null,");
    }
}

fn emit_module_load_gate_audit_rollback_requirements(binding: event_log::ModuleLoadGateBinding) {
    raw_line("    \"audit_rollback_requirements\": {");
    raw_line("      \"schema\": \"raios.module_load_gate_audit_rollback_requirements.v0\",");
    raw_line("      \"classification\": \"public\",");
    raw_line("      \"status\": \"required_missing\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"durable_audit_record\": {");
    raw_line("        \"schema\": \"raios.audit_record.v0\",");
    raw("        \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw_line(",");
    raw_line("        \"durability\": \"required_before_load\",");
    raw_line("        \"required_bindings\": [");
    raw_line("          \"denial_event_id\",");
    raw_line("          \"retained_computed_grant_reference_event_id\",");
    raw_line("          \"computed_capability_grant_hash\",");
    raw_line("          \"manifest_hash\",");
    raw_line("          \"artifact_hash\",");
    raw_line("          \"vm_test_report_hash\",");
    raw_line("          \"local_attestation_hash\",");
    raw_line("          \"local_approval_hash\",");
    raw_line("          \"rollback_plan_hash\",");
    raw_line("          \"ram_only_service_slot_id\"");
    raw_line("        ]");
    raw_line("      },");
    raw_line("      \"rollback_plan\": {");
    raw_line("        \"schema\": \"raios.rollback_plan.v0\",");
    raw("        \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw_line(",");
    raw_line("        \"must_preexist_load\": true,");
    raw_line("        \"required_bindings\": [");
    raw_line("          \"artifact_hash\",");
    raw_line("          \"pre_load_service_inventory_hash\",");
    raw_line("          \"ram_only_service_slot_id\",");
    raw_line("          \"cleanup_actions_hash\"");
    raw_line("        ]");
    raw_line("      },");
    raw_line("      \"required_hashes\": {");
    emit_module_load_gate_required_hashes(binding);
    raw_line("      },");
    raw("      \"retained_reference_event_id\": ");
    json_event_id_option(binding.retained_reference_event_id);
    raw_line(",");
    raw("      \"retained_manifest_reference_event_id\": ");
    json_event_id_option(binding.manifest_reference_event_id);
    raw_line(",");
    raw("      \"retained_local_attestation_reference_event_id\": ");
    json_event_id_option(binding.attestation_reference_event_id);
    raw_line(",");
    raw("      \"retained_local_approval_reference_event_id\": ");
    json_event_id_option(binding.approval_reference_event_id);
    raw_line(",");
    raw("      \"retained_audit_rollback_reference_event_id\": ");
    json_event_id_option(binding.audit_rollback_reference_event_id);
    raw_line(",");
    raw("      \"retained_service_slot_reservation_event_id\": ");
    json_event_id_option(binding.service_slot_reservation_event_id);
    raw_line(",");
    raw("      \"local_approval\": {\"state\": ");
    json_str(module_load_gate_local_approval_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_approval_reason(binding));
    raw_line(", \"required\": true, \"authorizes_guest_load\": false},");
    raw("      \"ram_only_service_slot\": {\"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw_line(", \"required\": true, \"allocates_service_slot\": false},");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load\": false");
    raw("    }");
}

fn emit_module_load_gate_required_hashes(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("        \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
    } else {
        raw_line("        \"computed_capability_grant_hash\": null,");
    }
    if let Some(reference) = binding
        .attestation_reference
        .filter(|_| module_load_gate_local_attestation_reference_valid(binding))
    {
        raw("        \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw_line(",");
        raw("        \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw_line(",");
    } else {
        raw_line("        \"local_attestation_reference_hash\": null,");
        raw_line("        \"local_attestation_hash\": null,");
    }
    if let Some(reference) = binding
        .approval_reference
        .filter(|_| module_load_gate_local_approval_reference_valid(binding))
    {
        raw("        \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw_line(",");
        raw("        \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
    } else {
        raw_line("        \"local_approval_reference_hash\": null,");
        raw_line("        \"local_approval_hash\": null,");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw("        \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw_line(",");
        raw("        \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw_line(",");
    } else {
        raw_line("        \"vm_test_report_reference_hash\": null,");
        raw_line("        \"vm_test_report_hash\": null,");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw("        \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw_line(",");
        raw("        \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
    } else {
        raw_line("        \"artifact_reference_hash\": null,");
        raw_line("        \"artifact_hash\": null,");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw("        \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw_line(",");
        raw("        \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw_line(",");
    } else {
        raw_line("        \"manifest_reference_hash\": null,");
        raw_line("        \"manifest_hash\": null,");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw("        \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("        \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("        \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw_line(",");
        raw("        \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw_line(",");
        raw("        \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
    } else {
        raw_line("        \"audit_record_hash\": null,");
        raw_line("        \"rollback_plan_hash\": null,");
        raw_line("        \"pre_load_service_inventory_hash\": null,");
        raw_line("        \"cleanup_actions_hash\": null,");
        raw_line("        \"ram_only_service_slot_id\": null,");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw("        \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        crlf();
    } else {
        raw_line("        \"service_slot_reservation_hash\": null");
    }
}

pub(crate) fn emit_module_load_ephemeral_denied(
    method: &'static str,
    event_id: event_log::EventId,
    gate_binding: event_log::ModuleLoadGateBinding,
) {
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.module_load_gate.v0\",");
    raw("    \"message\": ");
    json_str("ephemeral module loading is denied until a manifest, exact artifact, VM test report, local attestation, local approval, computed capability grant, audit record, and rollback plan are bound");
    raw_line(",");
    raw_line("    \"request\": {");
    raw_line("      \"load_mode\": \"ram_only\",");
    raw_line("      \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("      \"risk\": \"modify_ram\",");
    raw_line("      \"target\": \"live_service_graph\",");
    raw_line("      \"subject\": \"agent.session.serial\"");
    raw_line("    },");
    raw_line("    \"gate_state\": {");
    raw("      \"module_manifest\": ");
    json_str(module_load_gate_manifest_state(gate_binding));
    raw_line(",");
    raw("      \"candidate_artifact\": ");
    json_str(module_load_gate_candidate_artifact_state(gate_binding));
    raw_line(",");
    raw("      \"vm_test_report\": ");
    json_str(module_load_gate_vm_test_report_state(gate_binding));
    raw_line(",");
    raw("      \"local_attestation\": ");
    json_str(module_load_gate_local_attestation_state(gate_binding));
    raw_line(",");
    raw("      \"computed_capability_grant\": ");
    json_str(module_load_gate_computed_grant_state(gate_binding));
    raw_line(",");
    raw("      \"local_approval\": ");
    json_str(module_load_gate_local_approval_state(gate_binding));
    raw_line(",");
    raw("      \"rollback_plan\": ");
    json_str(module_load_gate_rollback_state(gate_binding));
    raw_line(",");
    raw("      \"durable_audit_record\": ");
    json_str(module_load_gate_durable_audit_state(gate_binding));
    raw_line(",");
    raw("      \"service_slot\": ");
    json_str(module_load_gate_service_slot_state(gate_binding));
    raw_line(",");
    raw("      \"service_slot_allocator\": ");
    json_str(module_load_gate_service_slot_allocator_state(gate_binding));
    raw_line(",");
    raw("      \"loader_runtime\": ");
    json_str(module_load_gate_loader_runtime_state(gate_binding));
    raw_line(",");
    raw_line("      \"loader\": \"unavailable\",");
    raw_line("      \"artifact_loaded\": false,");
    raw_line("      \"service_started\": false,");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"can_load\": false");
    raw_line("    },");
    emit_module_load_gate_manifest_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_artifact_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_vm_report_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_local_attestation_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_local_approval_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_retained_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_audit_rollback_reference(gate_binding);
    raw_line(",");
    emit_module_load_gate_service_slot_reservation(gate_binding);
    raw_line(",");
    emit_module_load_gate_service_slot_allocator_readiness(gate_binding);
    raw_line(",");
    emit_module_load_gate_loader_runtime_readiness(gate_binding);
    raw_line(",");
    emit_module_load_gate_audit_rollback_requirements(gate_binding);
    raw_line(",");
    raw_line("    \"blocked_by\": [");
    raw("      {\"gate\": \"module_manifest\", \"state\": ");
    json_str(module_load_gate_manifest_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_manifest_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"candidate_artifact\", \"state\": ");
    json_str(module_load_gate_candidate_artifact_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_candidate_artifact_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"vm_test_report\", \"state\": ");
    json_str(module_load_gate_vm_test_report_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_vm_test_report_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"local_attestation\", \"state\": ");
    json_str(module_load_gate_local_attestation_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_attestation_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"local_approval\", \"state\": ");
    json_str(module_load_gate_local_approval_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_approval_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"computed_capability_grant\", \"state\": ");
    json_str(module_load_gate_computed_grant_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_computed_grant_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"durable_audit_record\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_durable_audit_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"rollback_plan\", \"state\": ");
    json_str(module_load_gate_rollback_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_rollback_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"service_slot\", \"state\": ");
    json_str(module_load_gate_service_slot_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"service_slot_allocator\", \"state\": ");
    json_str(module_load_gate_service_slot_allocator_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_allocator_reason(gate_binding));
    raw_line("},");
    raw("      {\"gate\": \"loader_runtime\", \"state\": ");
    json_str(module_load_gate_loader_runtime_state(gate_binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_loader_runtime_reason(gate_binding));
    raw_line("},");
    raw_line(
        "      {\"gate\": \"loader\", \"state\": \"unavailable\", \"reason\": \"module_loader_unimplemented\"}",
    );
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"raios.module_manifest.v0\",");
    raw_line("      \"candidate_artifact_sha256\",");
    raw_line("      \"raios.vm_test_report.v0\",");
    raw_line("      \"raios.local_attestation.v0\",");
    raw_line("      \"computed_capability_grant\",");
    raw_line("      \"local_approval\",");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"rollback_plan\",");
    raw_line("      \"ram_only_service_slot\",");
    raw_line("      \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("      \"raios.module_loader_runtime_readiness.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw("      \"denial_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"event_scope\": \"current_boot\",");
    emit_module_load_gate_evidence_hashes(gate_binding);
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_module_load_gate_event_binding(binding: event_log::ModuleLoadGateBinding) {
    raw(", \"bindings\": {\"schema\": \"raios.module_load_gate.v0\", \"status\": \"denied_missing_evidence\", \"load_mode\": \"ram_only\", \"requested_capability\": \"cap.module.load_ephemeral\", \"risk\": \"modify_ram\", \"target\": \"live_service_graph\", \"subject\": \"agent.session.serial\", \"gate_state\": {\"module_manifest\": ");
    json_str(module_load_gate_manifest_state(binding));
    raw(", \"candidate_artifact\": ");
    json_str(module_load_gate_candidate_artifact_state(binding));
    raw(", \"vm_test_report\": ");
    json_str(module_load_gate_vm_test_report_state(binding));
    raw(", \"local_attestation\": ");
    json_str(module_load_gate_local_attestation_state(binding));
    raw(", \"computed_capability_grant\": ");
    json_str(module_load_gate_computed_grant_state(binding));
    raw(", \"local_approval\": ");
    json_str(module_load_gate_local_approval_state(binding));
    raw(", \"rollback_plan\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"durable_audit_record\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"service_slot\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"service_slot_allocator\": ");
    json_str(module_load_gate_service_slot_allocator_state(binding));
    raw(", \"loader_runtime\": ");
    json_str(module_load_gate_loader_runtime_state(binding));
    raw(", \"loader\": \"unavailable\"");
    raw(", \"artifact_loaded\": false, \"service_started\": false, \"persistence\": \"none\", \"can_load\": false}, \"retained_module_manifest_reference\": ");
    emit_module_load_gate_manifest_reference_compact(binding);
    raw(", \"retained_candidate_artifact_reference\": ");
    emit_module_load_gate_artifact_reference_compact(binding);
    raw(", \"retained_vm_test_report_reference\": ");
    emit_module_load_gate_vm_report_reference_compact(binding);
    raw(", \"retained_local_attestation_reference\": ");
    emit_module_load_gate_local_attestation_reference_compact(binding);
    raw(", \"retained_local_approval_reference\": ");
    emit_module_load_gate_local_approval_reference_compact(binding);
    raw(", \"retained_computed_grant_reference\": ");
    emit_module_load_gate_retained_reference_compact(binding);
    raw(", \"retained_audit_rollback_reference\": ");
    emit_module_load_gate_audit_rollback_reference_compact(binding);
    raw(", \"retained_service_slot_reservation\": ");
    emit_module_load_gate_service_slot_reservation_compact(binding);
    raw(", \"service_slot_allocator_readiness\": ");
    emit_module_load_gate_service_slot_allocator_readiness_compact(binding);
    raw(", \"loader_runtime_readiness\": ");
    emit_module_load_gate_loader_runtime_readiness_compact(binding);
    raw(", \"audit_rollback_requirements\": ");
    emit_module_load_gate_audit_rollback_requirements_compact(binding);
    raw(", \"blocked_by\": [{\"gate\": \"module_manifest\", \"state\": ");
    json_str(module_load_gate_manifest_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_manifest_reason(binding));
    raw("}, {\"gate\": \"candidate_artifact\", \"state\": ");
    json_str(module_load_gate_candidate_artifact_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_candidate_artifact_reason(binding));
    raw("}, {\"gate\": \"vm_test_report\", \"state\": ");
    json_str(module_load_gate_vm_test_report_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_vm_test_report_reason(binding));
    raw("}, {\"gate\": \"local_attestation\", \"state\": ");
    json_str(module_load_gate_local_attestation_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_attestation_reason(binding));
    raw("}, {\"gate\": \"local_approval\", \"state\": ");
    json_str(module_load_gate_local_approval_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_approval_reason(binding));
    raw("}, {\"gate\": \"computed_capability_grant\", \"state\": ");
    json_str(module_load_gate_computed_grant_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_computed_grant_reason(binding));
    raw("}, {\"gate\": \"durable_audit_record\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_durable_audit_reason(binding));
    raw("}, {\"gate\": \"rollback_plan\", \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_rollback_reason(binding));
    raw("}, {\"gate\": \"service_slot\", \"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw("}, {\"gate\": \"service_slot_allocator\", \"state\": ");
    json_str(module_load_gate_service_slot_allocator_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_allocator_reason(binding));
    raw("}, {\"gate\": \"loader_runtime\", \"state\": ");
    json_str(module_load_gate_loader_runtime_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_loader_runtime_reason(binding));
    raw("}, {\"gate\": \"loader\", \"state\": \"unavailable\", \"reason\": \"module_loader_unimplemented\"}], \"required\": [\"raios.module_manifest.v0\", \"candidate_artifact_sha256\", \"raios.vm_test_report.v0\", \"raios.local_attestation.v0\", \"raios.computed_capability_grant.v0\", \"local_approval\", \"raios.audit_record.v0\", \"rollback_plan\", \"ram_only_service_slot\", \"raios.module_service_slot_allocator_readiness.v0\", \"raios.module_loader_runtime_readiness.v0\"], \"evidence\": {\"event_scope\": \"current_boot\", ");
    emit_module_load_gate_evidence_hashes_compact(binding);
    raw(", \"service_inventory_change\": \"none\", \"load_attempted\": false}}");
}

fn emit_module_load_gate_manifest_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.manifest_reference {
        if module_load_gate_manifest_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.manifest_reference_event_id);
            raw(", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
            json_str(binding.manifest_reference_status);
            raw(", \"reason\": ");
            json_str(binding.manifest_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.manifest_reference_event_id);
        raw(", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
        json_str(binding.manifest_reference_status);
        raw(", \"reason\": ");
        json_str(binding.manifest_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_artifact_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.artifact_reference {
        if module_load_gate_candidate_artifact_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.artifact_reference_event_id);
            raw(", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
            json_str(binding.artifact_reference_status);
            raw(", \"reason\": ");
            json_str(binding.artifact_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.artifact_reference_event_id);
        raw(", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
        json_str(binding.artifact_reference_status);
        raw(", \"reason\": ");
        json_str(binding.artifact_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_vm_report_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.vm_report_reference {
        if module_load_gate_vm_test_report_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.vm_report_reference_event_id);
            raw(", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
            json_str(binding.vm_report_reference_status);
            raw(", \"reason\": ");
            json_str(binding.vm_report_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.vm_report_reference_event_id);
        raw(", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_vm_report_json\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
        json_str(binding.vm_report_reference_status);
        raw(", \"reason\": ");
        json_str(binding.vm_report_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_local_attestation_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.attestation_reference {
        if module_load_gate_local_attestation_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.attestation_reference_event_id);
            raw(", \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": ");
            json_str(binding.attestation_reference_status);
            raw(", \"reason\": ");
            json_str(binding.attestation_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.attestation_reference_event_id);
        raw(", \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_local_attestation_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": ");
        json_str(binding.attestation_reference_status);
        raw(", \"reason\": ");
        json_str(binding.attestation_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_local_approval_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.approval_reference {
        if module_load_gate_local_approval_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.approval_reference_event_id);
            raw(", \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": ");
            json_str(binding.approval_reference_status);
            raw(", \"reason\": ");
            json_str(binding.approval_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.approval_reference_event_id);
        raw(", \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"accepts_local_approval_text\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ");
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw(", \"retained_local_attestation_reference_event_id\": ");
        json_event_id(reference.retained_local_attestation_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw(", \"local_attestation_reference_hash\": ");
        json_sha256(reference.local_attestation_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": ");
        json_str(binding.approval_reference_status);
        raw(", \"reason\": ");
        json_str(binding.approval_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_retained_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.retained_reference_event_id);
        raw(", \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"hashes\": {\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"missing\", \"reason\": \"no_valid_computed_grant_reference_retained\", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_audit_rollback_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.audit_rollback_reference {
        if module_load_gate_audit_rollback_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.audit_rollback_reference_event_id);
            raw(", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
            json_str(binding.audit_rollback_reference_status);
            raw(", \"reason\": ");
            json_str(binding.audit_rollback_reference_reason);
            raw(", \"classification\": \"local_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.audit_rollback_reference_event_id);
        raw(", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"denial_event_id\": ");
        json_event_id(reference.denial_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
        json_str(binding.audit_rollback_reference_status);
        raw(", \"reason\": ");
        json_str(binding.audit_rollback_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_service_slot_reservation_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reservation) = binding.service_slot_reservation {
        if module_load_gate_service_slot_reservation_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.service_slot_reservation_event_id);
            raw(", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
            json_str(binding.service_slot_reservation_status);
            raw(", \"reason\": ");
            json_str(binding.service_slot_reservation_reason);
            raw(", \"classification\": \"local_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.service_slot_reservation_event_id);
        raw(", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": \"retained_hash_reference_only_not_allocated\", \"classification\": \"local_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reservation.retained_reference_event_id);
        raw(", \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reservation.retained_audit_rollback_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reservation.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reservation.computed_grant_hash);
        raw(", \"audit_record_hash\": ");
        json_sha256(reservation.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reservation.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reservation.pre_load_service_inventory_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
        json_str(binding.service_slot_reservation_status);
        raw(", \"reason\": ");
        json_str(binding.service_slot_reservation_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_service_slot_allocator_readiness_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw("{\"schema\": \"raios.module_service_slot_allocator_readiness.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"source_method\": \"module.service_slot_allocator\", \"state\": ");
    json_str(module_load_gate_service_slot_allocator_state(binding));
    raw(", \"readiness_status\": ");
    json_str(module_load_gate_service_slot_allocator_status(binding));
    raw(", \"readiness_reason\": ");
    json_str(module_load_gate_service_slot_allocator_reason(binding));
    raw(", \"retained_service_slot_reservation_status\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"service_slot_allocator_ready\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"can_load_now\": false, \"load_attempted\": false}");
}

fn emit_module_load_gate_loader_runtime_readiness_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw("{\"schema\": \"raios.module_loader_runtime_readiness.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"state\": ");
    json_str(module_load_gate_loader_runtime_state(binding));
    raw(", \"readiness_status\": ");
    json_str(module_load_gate_loader_runtime_status(binding));
    raw(", \"readiness_reason\": ");
    json_str(module_load_gate_loader_runtime_reason(binding));
    raw(", \"retained_module_evidence_state\": ");
    json_str(module_load_gate_retained_module_evidence_state(binding));
    raw(", \"retained_module_evidence_reason\": ");
    json_str(module_load_gate_retained_module_evidence_reason(binding));
    raw(", \"service_slot_allocator_ready\": false, \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"can_load_now\": false, \"load_attempted\": false, \"missing_facts\": [\"raios.module_loader_identity.v0\", \"raios.module_loader_artifact_hash_binding.v0\", \"raios.module_loader_entrypoint_abi.v0\", \"raios.module_loader_address_space_boundary.v0\", \"raios.module_loader_memory_map_constraints.v0\", \"raios.module_loader_capability_import_table.v0\", \"raios.module_loader_service_slot_binding.v0\", \"raios.module_loader_health_state_hooks.v0\", \"raios.module_loader_rollback_hooks.v0\", \"raios.module_loader_audit_rollback_write_boundary_binding.v0\"]}");
}

fn emit_module_load_gate_evidence_hashes_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
    } else {
        raw("\"computed_capability_grant_hash\": null");
    }
    if let Some(reference) = binding
        .attestation_reference
        .filter(|_| module_load_gate_local_attestation_reference_valid(binding))
    {
        raw(", \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
    } else {
        raw(", \"local_attestation_reference_hash\": null, \"local_attestation_hash\": null");
    }
    if let Some(reference) = binding
        .approval_reference
        .filter(|_| module_load_gate_local_approval_reference_valid(binding))
    {
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
    } else {
        raw(", \"local_approval_reference_hash\": null, \"local_approval_hash\": null");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
    } else {
        raw(", \"vm_test_report_reference_hash\": null, \"vm_test_report_hash\": null");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
    } else {
        raw(", \"artifact_reference_hash\": null, \"artifact_hash\": null");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
    } else {
        raw(", \"manifest_reference_hash\": null, \"manifest_hash\": null");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw(", \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw(", \"audit_record_hash\": null, \"rollback_plan_hash\": null, \"pre_load_service_inventory_hash\": null, \"cleanup_actions_hash\": null, \"ram_only_service_slot_id\": null");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw(", \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
    } else {
        raw(", \"service_slot_reservation_hash\": null");
    }
}

fn emit_module_load_gate_audit_rollback_requirements_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw("{\"schema\": \"raios.module_load_gate_audit_rollback_requirements.v0\", \"classification\": \"public\", \"status\": \"required_missing\", \"writes_enabled\": false, \"creates_durable_audit_records\": false, \"creates_rollback_plans\": false, \"durable_audit_record\": {\"schema\": \"raios.audit_record.v0\", \"state\": ");
    json_str(module_load_gate_durable_audit_state(binding));
    raw(", \"durability\": \"required_before_load\", \"required_bindings\": [\"denial_event_id\", \"retained_computed_grant_reference_event_id\", \"computed_capability_grant_hash\", \"manifest_hash\", \"artifact_hash\", \"vm_test_report_hash\", \"local_attestation_hash\", \"local_approval_hash\", \"rollback_plan_hash\", \"ram_only_service_slot_id\"]}, \"rollback_plan\": {\"schema\": \"raios.rollback_plan.v0\", \"state\": ");
    json_str(module_load_gate_rollback_state(binding));
    raw(", \"must_preexist_load\": true, \"required_bindings\": [\"artifact_hash\", \"pre_load_service_inventory_hash\", \"ram_only_service_slot_id\", \"cleanup_actions_hash\"]}, \"required_hashes\": {");
    emit_module_load_gate_required_hashes_compact(binding);
    raw("}, \"retained_reference_event_id\": ");
    json_event_id_option(binding.retained_reference_event_id);
    raw(", \"retained_manifest_reference_event_id\": ");
    json_event_id_option(binding.manifest_reference_event_id);
    raw(", \"retained_local_attestation_reference_event_id\": ");
    json_event_id_option(binding.attestation_reference_event_id);
    raw(", \"retained_local_approval_reference_event_id\": ");
    json_event_id_option(binding.approval_reference_event_id);
    raw(", \"retained_audit_rollback_reference_event_id\": ");
    json_event_id_option(binding.audit_rollback_reference_event_id);
    raw(", \"retained_service_slot_reservation_event_id\": ");
    json_event_id_option(binding.service_slot_reservation_event_id);
    raw(", \"local_approval\": {\"state\": ");
    json_str(module_load_gate_local_approval_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_local_approval_reason(binding));
    raw(", \"required\": true, \"authorizes_guest_load\": false}, \"ram_only_service_slot\": {\"state\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"reason\": ");
    json_str(module_load_gate_service_slot_reason(binding));
    raw(", \"required\": true, \"allocates_service_slot\": false}, \"load_attempted\": false, \"service_inventory_change\": \"none\", \"can_load\": false}");
}

fn emit_module_load_gate_required_hashes_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
    } else {
        raw("\"computed_capability_grant_hash\": null");
    }
    if let Some(reference) = binding
        .attestation_reference
        .filter(|_| module_load_gate_local_attestation_reference_valid(binding))
    {
        raw(", \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
    } else {
        raw(", \"local_attestation_reference_hash\": null, \"local_attestation_hash\": null");
    }
    if let Some(reference) = binding
        .approval_reference
        .filter(|_| module_load_gate_local_approval_reference_valid(binding))
    {
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
    } else {
        raw(", \"local_approval_reference_hash\": null, \"local_approval_hash\": null");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
    } else {
        raw(", \"vm_test_report_reference_hash\": null, \"vm_test_report_hash\": null");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
    } else {
        raw(", \"artifact_reference_hash\": null, \"artifact_hash\": null");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
    } else {
        raw(", \"manifest_reference_hash\": null, \"manifest_hash\": null");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw(", \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw(", \"audit_record_hash\": null, \"rollback_plan_hash\": null, \"pre_load_service_inventory_hash\": null, \"cleanup_actions_hash\": null, \"ram_only_service_slot_id\": null");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw(", \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
    } else {
        raw(", \"service_slot_reservation_hash\": null");
    }
}
