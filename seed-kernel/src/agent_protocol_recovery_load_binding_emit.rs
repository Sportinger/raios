use crate::{
    agent_protocol_recovery::{
        recovery_load_binding_retained_loader_mismatch,
        recovery_load_binding_retained_local_approval_mismatch,
        recovery_load_binding_retained_rollback_evidence_mismatch,
        recovery_load_binding_retained_trust_mismatch,
        recovery_load_binding_retained_vm_test_mismatch,
    },
    agent_protocol_recovery_load_binding::{
        RecoveryLoadBindingCheck, RecoveryLoadBindingSelfTestCase,
    },
    agent_protocol_support::{crlf, json_event_id, json_sha256, json_str, raw, raw_bool, raw_line},
    event_log,
};

pub(crate) fn emit_recovery_artifact_load_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("      \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"authorizes_recovery_load\": false, \"loads_recovery_artifact\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_identity_binding_fact(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_identity_event_id\": {\"schema\": \"raios.recovery_artifact_identity.v0\"");
    if let Some((event_id, reference)) = retained {
        raw(", \"status\": \"retained_hash_reference_only\", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"retained_recovery_artifact_identity_reference_not_authorizing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_identity_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_trust_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_trust_event_id\": {\"schema\": \"raios.recovery_artifact_trust.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_trust_mismatch(retained_identity, retained);
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(mismatch.unwrap_or("retained_recovery_artifact_trust_reference_not_authorizing"));
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"hashes\": {\"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_trust_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_vm_test_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_vm_test_event_id\": {\"schema\": \"raios.recovery_artifact_vm_test.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_vm_test_mismatch(
            retained_identity,
            retained_trust,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch.unwrap_or("retained_recovery_artifact_vm_test_reference_not_authorizing"),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"hashes\": {\"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_vm_test_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_local_approval_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_local_approval_event_id\": {\"schema\": \"raios.recovery_artifact_local_approval.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_local_approval_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch
                .unwrap_or("retained_recovery_artifact_local_approval_reference_not_authorizing"),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"hashes\": {\"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_local_approval_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_loader_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_loader_event_id\": {\"schema\": \"raios.recovery_artifact_loader.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_loader_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(mismatch.unwrap_or("retained_recovery_artifact_loader_reference_not_authorizing"));
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw(", \"hashes\": {\"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_loader_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_rollback_evidence_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_rollback_evidence_event_id\": {\"schema\": \"raios.recovery_artifact_rollback_evidence.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_rollback_evidence_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
            retained,
        );
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(
            mismatch.unwrap_or(
                "retained_recovery_artifact_rollback_evidence_reference_not_authorizing",
            ),
        );
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"retained_recovery_artifact_trust_event_id\": ");
        json_event_id(reference.retained_trust_reference_event_id);
        raw(", \"retained_recovery_artifact_vm_test_event_id\": ");
        json_event_id(reference.retained_vm_test_reference_event_id);
        raw(", \"retained_recovery_artifact_local_approval_event_id\": ");
        json_event_id(reference.retained_local_approval_reference_event_id);
        raw(", \"retained_recovery_artifact_loader_event_id\": ");
        json_event_id(reference.retained_loader_reference_event_id);
        raw(", \"hashes\": {\"rollback_evidence_reference_hash\": ");
        json_sha256(reference.rollback_evidence_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"vm_test_reference_hash\": ");
        json_sha256(reference.vm_test_reference_hash);
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.local_approval_reference_hash);
        raw(", \"loader_reference_hash\": ");
        json_sha256(reference.loader_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw(", \"vm_test_hash\": ");
        json_sha256(reference.vm_test_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"loader_hash\": ");
        json_sha256(reference.loader_hash);
        raw(", \"rollback_evidence_hash\": ");
        json_sha256(reference.rollback_evidence_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_rollback_evidence_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_load_blocker(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    raw("        {\"gate\": ");
    json_str(gate);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw("}");
    *wrote = true;
}

pub(crate) fn emit_recovery_load_binding_check(
    check: &RecoveryLoadBindingCheck,
    _spaces: usize,
    _include_status: bool,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"recovery_only_capability_used\": ");
    raw_bool(check.recovery_only_capability_used);
    raw_line(",");
    raw("        \"accepts_normal_module_authority\": ");
    raw_bool(check.accepts_normal_module_authority);
    raw_line(",");
    raw("        \"append_payload_hash_authority\": ");
    raw_bool(check.append_payload_hash_authority);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"loads_normal_module\": ");
    raw_bool(check.loads_normal_module);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

pub(crate) fn emit_recovery_load_binding_selftest_case(
    case: &RecoveryLoadBindingSelfTestCase,
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
    raw(", \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"load_attempted\": false, \"normal_module_capability_accepted\": false, \"append_payload_hash_authority\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
