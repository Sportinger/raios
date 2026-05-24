use crate::{
    agent_protocol_recovery_artifact_types::{
        RecoveryIdentityReferenceCheck, RecoveryLifelineRequestReferenceCheck,
        RecoveryLoaderReferenceCheck, RecoveryLocalApprovalReferenceCheck,
        RecoveryRollbackEvidenceReferenceCheck, RecoveryTrustReferenceCheck,
        RecoveryVmTestReferenceCheck,
    },
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, parse_current_boot_event_id, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_identity_reference_object(check: &RecoveryIdentityReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_identity_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw("        \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("        \"expected_identity_reference_hash\": ");
    json_sha256_option(check.expected_identity_reference_hash);
    raw_line(",");
    raw("        \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    crlf();
    raw_line("      }");
}

pub(crate) fn emit_recovery_identity_retained_reference(
    check: &RecoveryIdentityReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_identity_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_identity_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw_line("        \"hashes\": {");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_identity_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_trust_reference_object(check: &RecoveryTrustReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_trust_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"expected_trust_reference_hash\": ");
    json_sha256_option(check.expected_trust_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_trust_retained_reference(
    check: &RecoveryTrustReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_trust_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_trust_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_trust_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_vm_test_reference_object(check: &RecoveryVmTestReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_vm_test_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw_line("        \"vm_test_schema\": \"raios.recovery_artifact_vm_test.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"expected_vm_test_reference_hash\": ");
    json_sha256_option(check.expected_vm_test_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_vm_test_retained_reference(
    check: &RecoveryVmTestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_vm_test_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_vm_test_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_vm_test.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_vm_test_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_vm_test.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_vm_test_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_local_approval_reference_object(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
) {
    raw_line("      \"recovery_artifact_local_approval_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw_line("        \"local_approval_schema\": \"raios.recovery_artifact_local_approval.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"expected_local_approval_reference_hash\": ");
    json_sha256_option(check.expected_local_approval_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_local_approval_retained_reference(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_local_approval_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_local_approval_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_local_approval.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_local_approval_text\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_local_approval.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line(
            "        \"reason\": \"no_valid_recovery_artifact_local_approval_reference_retained\",",
        );
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_loader_reference_object(check: &RecoveryLoaderReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_loader_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"expected_loader_reference_hash\": ");
    json_sha256_option(check.expected_loader_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_loader_retained_reference(
    check: &RecoveryLoaderReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_loader_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_loader_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_loader.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_loader_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_rollback_evidence_reference_object(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
) {
    raw_line("      \"recovery_artifact_rollback_evidence_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_rollback_evidence_json\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_loader_event_id\": ");
    json_opt_str(check.retained_loader_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"rollback_evidence_reference_hash\": ");
    json_sha256_option(check.rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"expected_rollback_evidence_reference_hash\": ");
    json_sha256_option(check.expected_rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    raw_line(",");
    raw("          \"rollback_evidence_hash\": ");
    json_sha256_option(check.rollback_evidence_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_rollback_evidence_retained_reference(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_rollback_evidence_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_rollback_evidence_reference_matches(
            check, reference,
        ));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_rollback_evidence_json\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_rollback_evidence.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line(
            "        \"reason\": \"no_valid_recovery_artifact_rollback_evidence_reference_retained\",",
        );
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_request_reference_object(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
) {
    raw_line("      \"recovery_lifeline_request_reference\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"hash_reference_only\": true,");
    raw_line("        \"accepts_lifeline_request_json\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_trust_event_id\": ");
    json_opt_str(check.retained_trust_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
    json_opt_str(check.retained_vm_test_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
    json_opt_str(check.retained_local_approval_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_loader_event_id\": ");
    json_opt_str(check.retained_loader_reference_event_id);
    raw_line(",");
    raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
    json_opt_str(check.retained_rollback_evidence_reference_event_id);
    raw_line(",");
    raw_line("        \"hashes\": {");
    raw("          \"lifeline_request_reference_hash\": ");
    json_sha256_option(check.lifeline_request_reference_hash);
    raw_line(",");
    raw("          \"expected_lifeline_request_reference_hash\": ");
    json_sha256_option(check.expected_lifeline_request_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"vm_test_reference_hash\": ");
    json_sha256_option(check.vm_test_reference_hash);
    raw_line(",");
    raw("          \"local_approval_reference_hash\": ");
    json_sha256_option(check.local_approval_reference_hash);
    raw_line(",");
    raw("          \"loader_reference_hash\": ");
    json_sha256_option(check.loader_reference_hash);
    raw_line(",");
    raw("          \"rollback_evidence_reference_hash\": ");
    json_sha256_option(check.rollback_evidence_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    raw_line(",");
    raw("          \"vm_test_hash\": ");
    json_sha256_option(check.vm_test_hash);
    raw_line(",");
    raw("          \"local_approval_hash\": ");
    json_sha256_option(check.local_approval_hash);
    raw_line(",");
    raw("          \"loader_hash\": ");
    json_sha256_option(check.loader_hash);
    raw_line(",");
    raw("          \"rollback_evidence_hash\": ");
    json_sha256_option(check.rollback_evidence_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

pub(crate) fn emit_recovery_lifeline_request_retained_reference(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
) {
    raw_line("      \"retained_recovery_lifeline_request_reference\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(recovery_lifeline_request_reference_matches(
            check, reference,
        ));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_lifeline_request_json\": false,");
        raw_line("        \"accepts_loader_descriptor\": false,");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"loads_recovery_loader\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"creates_durable_records\": false,");
        raw_line("        \"installs_rollback_plan\": false,");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw_line(",");
        raw("        \"retained_recovery_artifact_rollback_evidence_event_id\": ");
        json_event_id(reference.retained_rollback_evidence_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"lifeline_request_reference_hash\": ");
        json_sha256(reference.lifeline_request_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw_line(",");
        raw("          \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw_line(",");
        raw("          \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw_line(",");
        raw("          \"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw_line(",");
        raw("          \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw_line(",");
        raw("          \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw_line(",");
        raw("          \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw_line(",");
        raw("          \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_lifeline_request.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_lifeline_request_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

pub(crate) fn recovery_identity_reference_matches(
    check: &RecoveryIdentityReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactIdentityReference,
) -> bool {
    check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
}

pub(crate) fn recovery_trust_reference_matches(
    check: &RecoveryTrustReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactTrustReference,
) -> bool {
    check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
}

pub(crate) fn recovery_vm_test_reference_matches(
    check: &RecoveryVmTestReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactVmTestReference,
) -> bool {
    check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
}

pub(crate) fn recovery_local_approval_reference_matches(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactLocalApprovalReference,
) -> bool {
    check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
}

pub(crate) fn recovery_loader_reference_matches(
    check: &RecoveryLoaderReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactLoaderReference,
) -> bool {
    check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
}

pub(crate) fn recovery_rollback_evidence_reference_matches(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactRollbackEvidenceReference,
) -> bool {
    check.rollback_evidence_reference_hash == Some(reference.rollback_evidence_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check
            .retained_loader_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_loader_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
        && check.rollback_evidence_hash == Some(reference.rollback_evidence_hash)
}

pub(crate) fn recovery_lifeline_request_reference_matches(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
    reference: event_log::RecoveryLifelineRequestReference,
) -> bool {
    check.lifeline_request_reference_hash == Some(reference.lifeline_request_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check
            .retained_trust_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_trust_reference_event_id)
        && check
            .retained_vm_test_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_vm_test_reference_event_id)
        && check
            .retained_local_approval_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_local_approval_reference_event_id)
        && check
            .retained_loader_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_loader_reference_event_id)
        && check
            .retained_rollback_evidence_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_rollback_evidence_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check.vm_test_reference_hash == Some(reference.vm_test_reference_hash)
        && check.local_approval_reference_hash == Some(reference.local_approval_reference_hash)
        && check.loader_reference_hash == Some(reference.loader_reference_hash)
        && check.rollback_evidence_reference_hash
            == Some(reference.rollback_evidence_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
        && check.vm_test_hash == Some(reference.vm_test_hash)
        && check.local_approval_hash == Some(reference.local_approval_hash)
        && check.loader_hash == Some(reference.loader_hash)
        && check.rollback_evidence_hash == Some(reference.rollback_evidence_hash)
}
