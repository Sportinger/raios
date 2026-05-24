use crate::{
    agent_protocol_module_grant::{
        module_computed_grant_reference_hashes_consistent, module_computed_grant_reference_matches,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{method_eq, parse_current_boot_event_id},
    event_log,
    module_evidence::{self, ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput},
};

pub(crate) fn evaluate_module_load_gate_manifest_candidate(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestEvaluation {
    if candidate.candidate_reference.is_none() {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    }
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_substituted_record",
        );
    }
    let Some(reference) = candidate.candidate_reference else {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    };
    if reference.manifest_reference_hash
        != computed_module_manifest_reference_hash(reference.manifest_hash)
    {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
        );
    }
    module_load_gate_manifest_check(
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
    )
}

fn module_load_gate_manifest_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateManifestEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateManifestEvaluation {
        status,
        reason,
        module_manifest_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_manifest_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_load_gate_artifact_candidate(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_artifact_check(
            "missing",
            "retained_candidate_artifact_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_substituted_record",
        );
    }
    if candidate_reference.artifact_reference_hash
        != module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.artifact_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.vm_report_hash != retained_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.artifact_hash != retained_reference.artifact_hash {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_hash_mismatch",
        );
    }

    module_load_gate_artifact_check(
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
    )
}

pub(crate) fn evaluate_module_load_gate_vm_report_candidate(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_vm_report_check(
            "missing",
            "retained_vm_test_report_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
        );
    }
    if candidate_reference.report_reference_hash
        != module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference
                .retained_artifact_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.artifact_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.artifact_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    }

    let (Some(artifact_event_id), Some(artifact_reference)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    };
    if candidate_reference.retained_artifact_reference_event_id != artifact_event_id
        || candidate_reference.artifact_reference_hash != artifact_reference.artifact_reference_hash
        || candidate_reference.manifest_reference_hash != artifact_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != artifact_reference.manifest_hash
        || candidate_reference.artifact_hash != artifact_reference.artifact_hash
        || candidate_reference.local_attestation_hash != artifact_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != artifact_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.artifact_hash != retained_reference.artifact_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != retained_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    module_load_gate_vm_report_check(
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
    )
}

pub(crate) fn evaluate_module_load_gate_local_attestation_candidate(
    candidate: ModuleLoadGateLocalAttestationReferenceCandidate,
) -> ModuleLoadGateLocalAttestationEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_local_attestation_check(
            "missing",
            "retained_local_attestation_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_substituted_record",
        );
    }
    if candidate_reference.attestation_reference_hash
        != module_evidence::computed_module_local_attestation_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference
                .retained_artifact_reference_event_id
                .sequence(),
            candidate_reference
                .retained_vm_report_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.artifact_reference_hash,
            candidate_reference.vm_report_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.artifact_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_manifest_reference_mismatch",
        );
    }

    let (Some(artifact_event_id), Some(artifact_reference)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_artifact_reference_mismatch",
        );
    };
    if candidate_reference.retained_artifact_reference_event_id != artifact_event_id
        || candidate_reference.artifact_reference_hash != artifact_reference.artifact_reference_hash
        || candidate_reference.manifest_reference_hash != artifact_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != artifact_reference.manifest_hash
        || candidate_reference.artifact_hash != artifact_reference.artifact_hash
        || candidate_reference.local_attestation_hash != artifact_reference.local_attestation_hash
    {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_artifact_reference_mismatch",
        );
    }

    let (Some(vm_report_event_id), Some(vm_report_reference)) =
        (candidate.vm_report_event_id, candidate.vm_report_reference)
    else {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_vm_report_reference_mismatch",
        );
    };
    if candidate_reference.retained_vm_report_reference_event_id != vm_report_event_id
        || candidate_reference.vm_report_reference_hash != vm_report_reference.report_reference_hash
        || candidate_reference.artifact_reference_hash
            != vm_report_reference.artifact_reference_hash
        || candidate_reference.vm_report_hash != vm_report_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != vm_report_reference.local_attestation_hash
    {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_vm_report_reference_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.artifact_hash != retained_reference.artifact_hash
        || candidate_reference.vm_report_hash != retained_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_local_attestation_check(
            "rejected",
            "retained_local_attestation_reference_computed_grant_reference_mismatch",
        );
    }

    module_load_gate_local_attestation_check(
        "retained_hash_reference_only",
        "retained_local_attestation_reference_not_authorizing",
    )
}

pub(crate) fn evaluate_module_load_gate_local_approval_candidate(
    candidate: ModuleLoadGateLocalApprovalReferenceCandidate,
) -> ModuleLoadGateLocalApprovalEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_local_approval_check(
            "missing",
            "retained_local_approval_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_substituted_record",
        );
    }
    if candidate_reference.approval_reference_hash
        != module_evidence::computed_module_local_approval_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference
                .retained_artifact_reference_event_id
                .sequence(),
            candidate_reference
                .retained_vm_report_reference_event_id
                .sequence(),
            candidate_reference
                .retained_local_attestation_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.artifact_reference_hash,
            candidate_reference.vm_report_reference_hash,
            candidate_reference.local_attestation_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.artifact_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
            candidate_reference.local_approval_hash,
        )
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_manifest_reference_mismatch",
        );
    }

    let (Some(artifact_event_id), Some(artifact_reference)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_artifact_reference_mismatch",
        );
    };
    if candidate_reference.retained_artifact_reference_event_id != artifact_event_id
        || candidate_reference.artifact_reference_hash != artifact_reference.artifact_reference_hash
        || candidate_reference.manifest_reference_hash != artifact_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != artifact_reference.manifest_hash
        || candidate_reference.artifact_hash != artifact_reference.artifact_hash
        || candidate_reference.local_attestation_hash != artifact_reference.local_attestation_hash
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_artifact_reference_mismatch",
        );
    }

    let (Some(vm_report_event_id), Some(vm_report_reference)) =
        (candidate.vm_report_event_id, candidate.vm_report_reference)
    else {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_vm_report_reference_mismatch",
        );
    };
    if candidate_reference.retained_vm_report_reference_event_id != vm_report_event_id
        || candidate_reference.vm_report_reference_hash != vm_report_reference.report_reference_hash
        || candidate_reference.artifact_reference_hash
            != vm_report_reference.artifact_reference_hash
        || candidate_reference.vm_report_hash != vm_report_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != vm_report_reference.local_attestation_hash
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_vm_report_reference_mismatch",
        );
    }

    let (Some(attestation_event_id), Some(attestation_reference)) = (
        candidate.attestation_event_id,
        candidate.attestation_reference,
    ) else {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_local_attestation_reference_mismatch",
        );
    };
    if candidate_reference.retained_local_attestation_reference_event_id != attestation_event_id
        || candidate_reference.local_attestation_reference_hash
            != attestation_reference.attestation_reference_hash
        || candidate_reference.vm_report_reference_hash
            != attestation_reference.vm_report_reference_hash
        || candidate_reference.local_attestation_hash
            != attestation_reference.local_attestation_hash
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_local_attestation_reference_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.artifact_hash != retained_reference.artifact_hash
        || candidate_reference.vm_report_hash != retained_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_local_approval_check(
            "rejected",
            "retained_local_approval_reference_computed_grant_reference_mismatch",
        );
    }

    module_load_gate_local_approval_check(
        "retained_hash_reference_only",
        "retained_local_approval_reference_not_authorizing",
    )
}

fn module_load_gate_artifact_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateArtifactEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateArtifactEvaluation {
        status,
        reason,
        candidate_artifact_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_artifact_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_vm_report_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateVmReportEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateVmReportEvaluation {
        status,
        reason,
        vm_test_report_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_vm_report_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_local_attestation_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateLocalAttestationEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateLocalAttestationEvaluation {
        status,
        reason,
        local_attestation_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_local_attestation_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_local_approval_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateLocalApprovalEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateLocalApprovalEvaluation {
        status,
        reason,
        local_approval_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_local_approval_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_load_gate_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "computed_capability_grant_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_substituted_record",
        );
    }
    if !module_computed_grant_reference_hashes_consistent(candidate_reference) {
        return module_load_gate_retained_check("rejected", "retained_reference_hash_mismatch");
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
    )
}

fn module_load_gate_retained_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateRetainedCheck {
    ModuleLoadGateRetainedCheck {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_load_gate_audit_rollback_candidate(
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackEvaluation {
    if !candidate.retained_reference {
        return module_load_gate_audit_rollback_check(
            "missing",
            "retained_computed_grant_reference_missing",
        );
    }
    let retained_audit_rollback_check =
        evaluate_module_load_gate_audit_rollback_reference_candidate(
            candidate.retained_audit_rollback_reference,
        );
    if !method_eq(
        retained_audit_rollback_check.status,
        "retained_hash_reference_only",
    ) {
        return module_load_gate_audit_rollback_check(
            retained_audit_rollback_check.status,
            retained_audit_rollback_check.reason,
        );
    }
    if !candidate.durable_audit_record {
        return module_load_gate_audit_rollback_check("missing", "durable_audit_write_missing");
    }
    if !candidate.rollback_plan {
        return module_load_gate_audit_rollback_check("missing", "rollback_install_missing");
    }
    if !candidate.audit_schema_ok {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "durable_audit_record_schema_mismatch",
        );
    }
    if !candidate.rollback_schema_ok {
        return module_load_gate_audit_rollback_check("rejected", "rollback_plan_schema_mismatch");
    }
    if !candidate.audit_binds_retained_grant {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_retained_grant_hash_mismatch",
        );
    }
    if !candidate.audit_binds_manifest {
        return module_load_gate_audit_rollback_check("rejected", "audit_manifest_hash_mismatch");
    }
    if !candidate.audit_binds_artifact {
        return module_load_gate_audit_rollback_check("rejected", "audit_artifact_hash_mismatch");
    }
    if !candidate.audit_binds_vm_report {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_vm_test_report_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_attestation {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_local_attestation_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_approval {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "local_approval_missing_or_mismatch",
        );
    }
    if !candidate.audit_binds_rollback_plan {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_rollback_plan_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_artifact {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "rollback_artifact_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_service_slot {
        return module_load_gate_audit_rollback_check("rejected", "rollback_service_slot_mismatch");
    }
    if !candidate.ram_only_service_slot_allocated && !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
        );
    }
    if !candidate.ram_only_service_slot_allocated {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "ram_only_service_slot_unallocated",
        );
    }
    if !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "module_loader_unimplemented",
        );
    }
    module_load_gate_audit_rollback_check("rejected", "positive_loader_path_unimplemented")
}

fn evaluate_module_load_gate_audit_rollback_reference_candidate(
    candidate: ModuleLoadGateAuditRollbackReferenceCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
        );
    }
    if candidate_reference.ram_only_service_slot_id.as_str()
        != MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID
    {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_audit_rollback_reference_hash_mismatch(candidate_reference) {
        return module_load_gate_retained_check("rejected", reason);
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_audit_rollback_reference_not_authorizing",
    )
}

pub(crate) fn evaluate_module_load_gate_service_slot_candidate(
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotEvaluation {
    let Some(reservation) = candidate.service_slot_reservation.candidate_reservation else {
        return module_load_gate_service_slot_check(
            "missing",
            "retained_service_slot_reservation_missing",
        );
    };
    let Some(retained_reference) = candidate.retained_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_computed_grant_reference_missing",
        );
    };
    let Some(audit_rollback_reference) = candidate.audit_rollback_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !candidate.audit_rollback_valid {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_not_valid_for_service_slot",
        );
    }

    let Some(retained_reference_event_id) =
        parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    };
    let Some(audit_rollback_event_id) =
        parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    };

    if reservation.retained_reference_event_id != retained_reference_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    }
    if reservation.retained_audit_rollback_reference_event_id != audit_rollback_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    }

    let service_slot_candidate = candidate.service_slot_reservation;
    if !method_eq(service_slot_candidate.scope, "current_boot") || !service_slot_candidate.retained
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    }
    if !service_slot_candidate.schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(event_reservation) = service_slot_candidate.event_reservation else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_service_slot_reservation_matches(event_reservation, reservation) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.grant_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(grant_event_reference) = service_slot_candidate.grant_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(retained_reference, grant_event_reference) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.audit_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(audit_event_reference) = service_slot_candidate.audit_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(
        audit_rollback_reference,
        audit_event_reference,
    ) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if reservation.computed_grant_hash != retained_reference.computed_grant_hash
        || reservation.computed_grant_hash != audit_rollback_reference.computed_grant_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
        );
    }
    if reservation.audit_record_hash != audit_rollback_reference.audit_record_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
        );
    }
    if reservation.rollback_plan_hash != audit_rollback_reference.rollback_plan_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
        );
    }
    if reservation.pre_load_service_inventory_hash
        != audit_rollback_reference.pre_load_service_inventory_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
        );
    }
    if reservation.ram_only_service_slot_id.as_str()
        != audit_rollback_reference.ram_only_service_slot_id.as_str()
        || !module_evidence::ram_only_service_slot_id_valid(
            reservation.ram_only_service_slot_id.as_str(),
        )
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_service_slot_reservation_hash_mismatch(reservation) {
        return module_load_gate_service_slot_check("rejected", reason);
    }

    module_load_gate_service_slot_check(
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
    )
}

pub(crate) fn evaluate_module_load_gate_loader_runtime_candidate(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> ModuleLoadGateLoaderRuntimeEvaluation {
    let retained_module_evidence_complete =
        module_load_gate_loader_runtime_retained_evidence_complete(candidate);
    let retained_module_evidence_state = if retained_module_evidence_complete {
        "available"
    } else if module_load_gate_loader_runtime_retained_evidence_rejected(candidate) {
        "rejected"
    } else {
        "missing"
    };
    let retained_module_evidence_reason =
        module_load_gate_loader_runtime_retained_evidence_reason(candidate);
    let service_slot_allocator_state = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        "missing_runtime"
    } else if module_load_gate_loader_runtime_reference_rejected(
        candidate.service_slot_reservation_state,
    ) {
        "blocked_by_rejected_service_slot_reservation"
    } else {
        "blocked_by_service_slot_reservation"
    };
    let service_slot_allocator_status = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        "missing"
    } else {
        "blocked"
    };
    let service_slot_allocator_reason = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        "service_slot_allocator_runtime_missing"
    } else if module_load_gate_loader_runtime_reference_rejected(
        candidate.service_slot_reservation_state,
    ) {
        candidate.service_slot_reservation_reason
    } else {
        "retained_service_slot_reservation_missing"
    };
    let (loader_runtime_state, status, reason) = if retained_module_evidence_complete {
        (
            "blocked_by_service_slot_allocator_runtime",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
        )
    } else {
        (
            "blocked_by_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            retained_module_evidence_reason,
        )
    };

    ModuleLoadGateLoaderRuntimeEvaluation {
        status,
        reason,
        retained_module_evidence_state,
        retained_module_evidence_reason,
        service_slot_allocator_state,
        service_slot_allocator_status,
        service_slot_allocator_reason,
        loader_runtime_state,
        can_load: false,
        load_attempted: false,
    }
}

fn module_audit_rollback_event_reference_matches(
    event_reference: event_log::ModuleAuditRollbackReference,
    candidate_reference: event_log::ModuleAuditRollbackReference,
) -> bool {
    event_reference.audit_record_hash == candidate_reference.audit_record_hash
        && event_reference.rollback_plan_hash == candidate_reference.rollback_plan_hash
        && event_reference.computed_grant_hash == candidate_reference.computed_grant_hash
        && event_reference.manifest_hash == candidate_reference.manifest_hash
        && event_reference.artifact_hash == candidate_reference.artifact_hash
        && event_reference.vm_report_hash == candidate_reference.vm_report_hash
        && event_reference.local_attestation_hash == candidate_reference.local_attestation_hash
        && event_reference.local_approval_hash == candidate_reference.local_approval_hash
        && event_reference.pre_load_service_inventory_hash
            == candidate_reference.pre_load_service_inventory_hash
        && event_reference.cleanup_actions_hash == candidate_reference.cleanup_actions_hash
        && event_reference.denial_event_id == candidate_reference.denial_event_id
        && event_reference.retained_reference_event_id
            == candidate_reference.retained_reference_event_id
        && event_reference.ram_only_service_slot_id.as_str()
            == candidate_reference.ram_only_service_slot_id.as_str()
}

fn module_audit_rollback_reference_hash_mismatch(
    reference: event_log::ModuleAuditRollbackReference,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_DENIAL_EVENT_ID)
        != Some(reference.denial_event_id)
        || parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
            != Some(reference.retained_reference_event_id)
    {
        return Some("retained_audit_rollback_reference_substituted_record");
    }

    let expected_computed_grant_hash = computed_module_grant_hash(
        reference.manifest_hash,
        reference.artifact_hash,
        reference.vm_report_hash,
        reference.local_attestation_hash,
    );
    if reference.computed_grant_hash != expected_computed_grant_hash {
        return Some("retained_audit_rollback_computed_grant_hash_mismatch");
    }

    let expected_rollback_plan_hash = computed_module_rollback_plan_hash(
        reference.artifact_hash,
        reference.pre_load_service_inventory_hash,
        reference.ram_only_service_slot_id.as_str(),
        reference.cleanup_actions_hash,
    );
    if reference.rollback_plan_hash != expected_rollback_plan_hash {
        return Some("retained_rollback_plan_hash_mismatch");
    }

    let expected_audit_record_hash =
        computed_module_audit_record_hash(ModuleAuditRecordHashInput {
            denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            computed_grant_hash: reference.computed_grant_hash,
            manifest_hash: reference.manifest_hash,
            artifact_hash: reference.artifact_hash,
            vm_report_hash: reference.vm_report_hash,
            local_attestation_hash: reference.local_attestation_hash,
            local_approval_hash: reference.local_approval_hash,
            rollback_plan_hash: reference.rollback_plan_hash,
            ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
        });
    if reference.audit_record_hash != expected_audit_record_hash {
        return Some("retained_audit_record_hash_mismatch");
    }

    None
}

fn module_service_slot_reservation_matches(
    left: event_log::ModuleServiceSlotReservation,
    right: event_log::ModuleServiceSlotReservation,
) -> bool {
    left.reservation_hash == right.reservation_hash
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.retained_audit_rollback_reference_event_id
            == right.retained_audit_rollback_reference_event_id
        && left.computed_grant_hash == right.computed_grant_hash
        && left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.ram_only_service_slot_id.as_str() == right.ram_only_service_slot_id.as_str()
}

fn module_service_slot_reservation_hash_mismatch(
    reservation: event_log::ModuleServiceSlotReservation,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
        != Some(reservation.retained_reference_event_id)
        || parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
            != Some(reservation.retained_audit_rollback_reference_event_id)
    {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    let expected_reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash: reservation.computed_grant_hash,
            audit_record_hash: reservation.audit_record_hash,
            rollback_plan_hash: reservation.rollback_plan_hash,
            pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
            ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
        });
    if reservation.reservation_hash != expected_reservation_hash {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    None
}

fn module_load_gate_loader_runtime_retained_evidence_complete(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> bool {
    module_load_gate_loader_runtime_reference_available(candidate.manifest_reference_state)
        && module_load_gate_loader_runtime_reference_available(candidate.artifact_reference_state)
        && module_load_gate_loader_runtime_reference_available(candidate.vm_report_reference_state)
        && module_load_gate_loader_runtime_reference_available(
            candidate.local_attestation_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.local_approval_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.computed_grant_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.audit_rollback_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.service_slot_reservation_state,
        )
}

fn module_load_gate_loader_runtime_retained_evidence_rejected(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> bool {
    module_load_gate_loader_runtime_reference_rejected(candidate.manifest_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(candidate.artifact_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(candidate.vm_report_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.local_attestation_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.local_approval_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.computed_grant_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.audit_rollback_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.service_slot_reservation_state,
        )
}

fn module_load_gate_loader_runtime_retained_evidence_reason(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> &'static str {
    if !module_load_gate_loader_runtime_reference_available(candidate.manifest_reference_state) {
        return candidate.manifest_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(candidate.artifact_reference_state) {
        return candidate.artifact_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(candidate.vm_report_reference_state) {
        return candidate.vm_report_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.local_attestation_reference_state,
    ) {
        return candidate.local_attestation_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.local_approval_reference_state,
    ) {
        return candidate.local_approval_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.computed_grant_reference_state,
    ) {
        return candidate.computed_grant_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.audit_rollback_reference_state,
    ) {
        return candidate.audit_rollback_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        return candidate.service_slot_reservation_reason;
    }
    "retained_module_evidence_available"
}

fn module_load_gate_loader_runtime_reference_available(state: &'static str) -> bool {
    method_eq(state, "available")
}

fn module_load_gate_loader_runtime_reference_rejected(state: &'static str) -> bool {
    method_eq(state, "rejected")
}

fn module_load_gate_service_slot_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateServiceSlotEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only_not_allocated");
    let service_slot_state = if accepted {
        "retained_hash_reference_only_not_allocated"
    } else if method_eq(status, "rejected") {
        "rejected_retained_reference"
    } else {
        "unallocated"
    };
    ModuleLoadGateServiceSlotEvaluation {
        status,
        reason,
        service_slot_state,
        accepted_service_slot_reservation_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_audit_rollback_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateAuditRollbackEvaluation {
    ModuleLoadGateAuditRollbackEvaluation {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    module_evidence::computed_module_manifest_reference_hash(manifest_hash)
}

pub(crate) fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}

pub(crate) fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    )
}

pub(crate) fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    module_evidence::computed_module_audit_record_hash(input)
}

pub(crate) fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    module_evidence::computed_module_service_slot_reservation_hash(input)
}
