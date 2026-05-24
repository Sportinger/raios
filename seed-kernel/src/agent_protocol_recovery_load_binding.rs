use crate::{
    agent_protocol_recovery_constants::{
        RECOVERY_ARTIFACT_LOAD_CAPABILITY, RECOVERY_LOAD_BINDING_SELFTEST_CASES,
    },
    agent_protocol_support::method_eq,
    event_log,
};

#[derive(Clone, Copy)]
pub(crate) struct RecoveryEvidenceCandidate {
    pub(crate) retained: bool,
    pub(crate) current_boot: bool,
    pub(crate) schema_ok: bool,
    pub(crate) binding_ok: bool,
    pub(crate) binding_reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadBindingCandidate {
    pub(crate) requested_capability: &'static str,
    pub(crate) identity: RecoveryEvidenceCandidate,
    pub(crate) trust: RecoveryEvidenceCandidate,
    pub(crate) vm_test: RecoveryEvidenceCandidate,
    pub(crate) local_approval: RecoveryEvidenceCandidate,
    pub(crate) loader: RecoveryEvidenceCandidate,
    pub(crate) rollback_evidence: RecoveryEvidenceCandidate,
    pub(crate) normal_module_capability_substituted: bool,
    pub(crate) normal_module_append_intent_substituted: bool,
    pub(crate) append_payload_hash_claimed_authority: bool,
    pub(crate) normal_module_writer_facts_substituted: bool,
    pub(crate) normal_module_service_slot_substituted: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadBindingCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) recovery_only_capability_used: bool,
    pub(crate) accepts_normal_module_authority: bool,
    pub(crate) append_payload_hash_authority: bool,
    pub(crate) can_move_beyond_denial: bool,
    pub(crate) loads_recovery_artifact: bool,
    pub(crate) loads_normal_module: bool,
    pub(crate) creates_durable_records: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) service_inventory_change: &'static str,
    pub(crate) load_attempted: bool,
}

pub(crate) struct RecoveryLoadBindingSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

pub(crate) fn recovery_load_binding_selftest_cases(
) -> [RecoveryLoadBindingSelfTestCase; RECOVERY_LOAD_BINDING_SELFTEST_CASES] {
    let valid = recovery_load_binding_available_candidate();

    let mut identity_missing = valid;
    identity_missing.identity = recovery_evidence_missing();
    let mut identity_previous_boot = valid;
    identity_previous_boot.identity.current_boot = false;
    let mut identity_wrong_schema = valid;
    identity_wrong_schema.identity.schema_ok = false;

    let mut trust_missing = valid;
    trust_missing.trust = recovery_evidence_missing();
    let mut vm_test_missing = valid;
    vm_test_missing.vm_test = recovery_evidence_missing();
    let mut local_approval_missing = valid;
    local_approval_missing.local_approval = recovery_evidence_missing();
    let mut loader_missing = valid;
    loader_missing.loader = recovery_evidence_missing();
    let mut rollback_missing = valid;
    rollback_missing.rollback_evidence = recovery_evidence_missing();

    let mut module_capability = valid;
    module_capability.requested_capability = "cap.module.load_ephemeral";
    module_capability.normal_module_capability_substituted = true;
    let mut module_append_intent = valid;
    module_append_intent.normal_module_append_intent_substituted = true;
    let mut append_payload_hash = valid;
    append_payload_hash.append_payload_hash_claimed_authority = true;
    let mut module_writer_facts = valid;
    module_writer_facts.normal_module_writer_facts_substituted = true;
    let mut module_service_slot = valid;
    module_service_slot.normal_module_service_slot_substituted = true;

    [
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
            identity_missing,
        ),
        recovery_load_binding_selftest_case(
            "previous_boot_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_event_id_not_current_boot",
            identity_previous_boot,
        ),
        recovery_load_binding_selftest_case(
            "wrong_schema_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_schema_mismatch",
            identity_wrong_schema,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
            trust_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_vm_test_event_id",
            "missing",
            "recovery_vm_test_event_id_missing",
            vm_test_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_local_approval_event_id",
            "missing",
            "recovery_local_approval_event_id_missing",
            local_approval_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_loader_event_id",
            "missing",
            "recovery_loader_event_id_missing",
            loader_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_rollback_evidence_event_id",
            "missing",
            "recovery_rollback_evidence_event_id_missing",
            rollback_missing,
        ),
        recovery_load_binding_selftest_case(
            "module_load_ephemeral_capability_substituted",
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
            module_capability,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_append_intent_substituted",
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
            module_append_intent,
        ),
        recovery_load_binding_selftest_case(
            "append_payload_hash_claimed_as_authority",
            "rejected",
            "append_payload_hash_not_recovery_authority",
            append_payload_hash,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_writer_facts_substituted",
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
            module_writer_facts,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_service_slot_substituted",
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
            module_service_slot,
        ),
        recovery_load_binding_selftest_case(
            "available_recovery_binding_still_denied",
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
            valid,
        ),
    ]
}

fn recovery_load_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingSelfTestCase {
    let actual = evaluate_recovery_load_binding(candidate);
    RecoveryLoadBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_move_beyond_denial
            && !actual.accepts_normal_module_authority
            && !actual.append_payload_hash_authority
            && !actual.loads_recovery_artifact
            && !actual.loads_normal_module
            && !actual.creates_durable_records
            && !actual.installs_rollback_plan
            && method_eq(actual.service_inventory_change, "none")
            && !actual.load_attempted,
    }
}

pub(crate) fn evaluate_recovery_load_binding(
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingCheck {
    let (status, reason) = if !method_eq(
        candidate.requested_capability,
        RECOVERY_ARTIFACT_LOAD_CAPABILITY,
    ) || candidate.normal_module_capability_substituted
    {
        (
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
        )
    } else if candidate.normal_module_append_intent_substituted {
        (
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
        )
    } else if candidate.append_payload_hash_claimed_authority {
        ("rejected", "append_payload_hash_not_recovery_authority")
    } else if candidate.normal_module_writer_facts_substituted {
        (
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
        )
    } else if candidate.normal_module_service_slot_substituted {
        (
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
        )
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.identity,
        "missing",
        "recovery_artifact_identity_event_id_missing",
        "rejected",
        "recovery_artifact_identity_event_id_not_current_boot",
        "recovery_artifact_identity_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.trust,
        "missing",
        "recovery_artifact_trust_event_id_missing",
        "rejected",
        "recovery_artifact_trust_event_id_not_current_boot",
        "recovery_artifact_trust_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.vm_test,
        "missing",
        "recovery_vm_test_event_id_missing",
        "rejected",
        "recovery_vm_test_event_id_not_current_boot",
        "recovery_vm_test_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.local_approval,
        "missing",
        "recovery_local_approval_event_id_missing",
        "rejected",
        "recovery_local_approval_event_id_not_current_boot",
        "recovery_local_approval_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.loader,
        "missing",
        "recovery_loader_event_id_missing",
        "rejected",
        "recovery_loader_event_id_not_current_boot",
        "recovery_loader_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.rollback_evidence,
        "missing",
        "recovery_rollback_evidence_event_id_missing",
        "rejected",
        "recovery_rollback_evidence_event_id_not_current_boot",
        "recovery_rollback_evidence_schema_mismatch",
    ) {
        result
    } else {
        (
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
        )
    };

    RecoveryLoadBindingCheck {
        status,
        reason,
        recovery_only_capability_used: method_eq(
            candidate.requested_capability,
            RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        ) && !candidate.normal_module_capability_substituted,
        accepts_normal_module_authority: false,
        append_payload_hash_authority: false,
        can_move_beyond_denial: false,
        loads_recovery_artifact: false,
        loads_normal_module: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn evaluate_recovery_evidence(
    evidence: RecoveryEvidenceCandidate,
    missing_status: &'static str,
    missing_reason: &'static str,
    rejected_status: &'static str,
    stale_reason: &'static str,
    schema_reason: &'static str,
) -> Option<(&'static str, &'static str)> {
    if !evidence.retained {
        Some((missing_status, missing_reason))
    } else if !evidence.current_boot {
        Some((rejected_status, stale_reason))
    } else if !evidence.schema_ok {
        Some((rejected_status, schema_reason))
    } else if !evidence.binding_ok {
        Some((rejected_status, evidence.binding_reason))
    } else {
        None
    }
}

pub(crate) fn recovery_load_binding_candidate_from_retained(
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
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> RecoveryLoadBindingCandidate {
    let mut candidate = recovery_load_binding_missing_candidate();
    if retained_identity.is_some() {
        candidate.identity = recovery_evidence_available();
    }
    if retained_trust.is_some() {
        candidate.trust = if let Some(reason) =
            recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
        {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_vm_test.is_some() {
        candidate.vm_test = if let Some(reason) = recovery_load_binding_retained_vm_test_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
        ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_local_approval.is_some() {
        candidate.local_approval = if let Some(reason) =
            recovery_load_binding_retained_local_approval_mismatch(
                retained_identity,
                retained_trust,
                retained_vm_test,
                retained_local_approval,
            ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_loader.is_some() {
        candidate.loader = if let Some(reason) = recovery_load_binding_retained_loader_mismatch(
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
        ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    if retained_rollback_evidence.is_some() {
        candidate.rollback_evidence = if let Some(reason) =
            recovery_load_binding_retained_rollback_evidence_mismatch(
                retained_identity,
                retained_trust,
                retained_vm_test,
                retained_local_approval,
                retained_loader,
                retained_rollback_evidence,
            ) {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    candidate
}

pub(crate) fn recovery_load_binding_retained_trust_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((_trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    if trust_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if trust_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if trust_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    None
}

pub(crate) fn recovery_load_binding_retained_vm_test_mismatch(
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
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((_vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    if vm_test_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_vm_test_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_vm_test_trust_event_id_mismatch");
    }
    if vm_test_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_vm_test_identity_reference_hash_mismatch");
    }
    if vm_test_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_vm_test_trust_reference_hash_mismatch");
    }
    if vm_test_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_vm_test_artifact_hash_mismatch");
    }
    if vm_test_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_vm_test_trust_artifact_hash_mismatch");
    }
    if vm_test_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_vm_test_trust_hash_mismatch");
    }
    None
}

pub(crate) fn recovery_load_binding_retained_local_approval_mismatch(
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
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    if approval_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_local_approval_identity_event_id_mismatch");
    }
    if approval_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_local_approval_trust_event_id_mismatch");
    }
    if approval_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_local_approval_vm_test_event_id_mismatch");
    }
    if approval_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_local_approval_identity_reference_hash_mismatch");
    }
    if approval_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_local_approval_trust_reference_hash_mismatch");
    }
    if approval_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_local_approval_vm_test_reference_hash_mismatch");
    }
    if approval_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_artifact_hash_mismatch");
    }
    if approval_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_trust_artifact_hash_mismatch");
    }
    if approval_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_local_approval_vm_test_artifact_hash_mismatch");
    }
    if approval_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_local_approval_trust_hash_mismatch");
    }
    if approval_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_local_approval_vm_test_trust_hash_mismatch");
    }
    if approval_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_local_approval_vm_test_hash_mismatch");
    }
    None
}

pub(crate) fn recovery_load_binding_retained_loader_mismatch(
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
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((local_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    let Some((_loader_event_id, loader_reference)) = retained_loader else {
        return None;
    };
    if loader_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_loader_identity_event_id_mismatch");
    }
    if loader_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_loader_trust_event_id_mismatch");
    }
    if loader_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_loader_vm_test_event_id_mismatch");
    }
    if loader_reference.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_artifact_loader_local_approval_event_id_mismatch");
    }
    if loader_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_loader_identity_reference_hash_mismatch");
    }
    if loader_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_loader_trust_reference_hash_mismatch");
    }
    if loader_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_loader_vm_test_reference_hash_mismatch");
    }
    if loader_reference.local_approval_reference_hash
        != approval_reference.local_approval_reference_hash
    {
        return Some("recovery_artifact_loader_local_approval_reference_hash_mismatch");
    }
    if loader_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_loader_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_loader_trust_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_loader_vm_test_artifact_hash_mismatch");
    }
    if loader_reference.artifact_hash != approval_reference.artifact_hash {
        return Some("recovery_artifact_loader_local_approval_artifact_hash_mismatch");
    }
    if loader_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_loader_trust_hash_mismatch");
    }
    if loader_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_loader_vm_test_trust_hash_mismatch");
    }
    if loader_reference.trust_hash != approval_reference.trust_hash {
        return Some("recovery_artifact_loader_local_approval_trust_hash_mismatch");
    }
    if loader_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_loader_vm_test_hash_mismatch");
    }
    if loader_reference.vm_test_hash != approval_reference.vm_test_hash {
        return Some("recovery_artifact_loader_local_approval_vm_test_hash_mismatch");
    }
    if loader_reference.local_approval_hash != approval_reference.local_approval_hash {
        return Some("recovery_artifact_loader_local_approval_hash_mismatch");
    }
    None
}

pub(crate) fn recovery_load_binding_retained_rollback_evidence_mismatch(
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
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    let Some((vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return None;
    };
    let Some((local_approval_event_id, approval_reference)) = retained_local_approval else {
        return None;
    };
    let Some((loader_event_id, loader_reference)) = retained_loader else {
        return None;
    };
    let Some((_rollback_event_id, rollback_reference)) = retained_rollback_evidence else {
        return None;
    };
    if rollback_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_rollback_evidence_identity_event_id_mismatch");
    }
    if rollback_reference.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_artifact_rollback_evidence_trust_event_id_mismatch");
    }
    if rollback_reference.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_artifact_rollback_evidence_vm_test_event_id_mismatch");
    }
    if rollback_reference.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_artifact_rollback_evidence_local_approval_event_id_mismatch");
    }
    if rollback_reference.retained_loader_reference_event_id != loader_event_id {
        return Some("recovery_artifact_rollback_evidence_loader_event_id_mismatch");
    }
    if rollback_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_rollback_evidence_identity_reference_hash_mismatch");
    }
    if rollback_reference.trust_reference_hash != trust_reference.trust_reference_hash {
        return Some("recovery_artifact_rollback_evidence_trust_reference_hash_mismatch");
    }
    if rollback_reference.vm_test_reference_hash != vm_test_reference.vm_test_reference_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_reference_hash_mismatch");
    }
    if rollback_reference.local_approval_reference_hash
        != approval_reference.local_approval_reference_hash
    {
        return Some("recovery_artifact_rollback_evidence_local_approval_reference_hash_mismatch");
    }
    if rollback_reference.loader_reference_hash != loader_reference.loader_reference_hash {
        return Some("recovery_artifact_rollback_evidence_loader_reference_hash_mismatch");
    }
    if rollback_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != trust_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_trust_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != vm_test_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != approval_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_artifact_hash_mismatch");
    }
    if rollback_reference.artifact_hash != loader_reference.artifact_hash {
        return Some("recovery_artifact_rollback_evidence_loader_artifact_hash_mismatch");
    }
    if rollback_reference.trust_hash != trust_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != vm_test_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != approval_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_trust_hash_mismatch");
    }
    if rollback_reference.trust_hash != loader_reference.trust_hash {
        return Some("recovery_artifact_rollback_evidence_loader_trust_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != vm_test_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_vm_test_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != approval_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_vm_test_hash_mismatch");
    }
    if rollback_reference.vm_test_hash != loader_reference.vm_test_hash {
        return Some("recovery_artifact_rollback_evidence_loader_vm_test_hash_mismatch");
    }
    if rollback_reference.local_approval_hash != approval_reference.local_approval_hash {
        return Some("recovery_artifact_rollback_evidence_local_approval_hash_mismatch");
    }
    if rollback_reference.local_approval_hash != loader_reference.local_approval_hash {
        return Some("recovery_artifact_rollback_evidence_loader_local_approval_hash_mismatch");
    }
    if rollback_reference.loader_hash != loader_reference.loader_hash {
        return Some("recovery_artifact_rollback_evidence_loader_hash_mismatch");
    }
    None
}

fn recovery_load_binding_missing_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_missing(),
        trust: recovery_evidence_missing(),
        vm_test: recovery_evidence_missing(),
        local_approval: recovery_evidence_missing(),
        loader: recovery_evidence_missing(),
        rollback_evidence: recovery_evidence_missing(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_load_binding_available_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_available(),
        trust: recovery_evidence_available(),
        vm_test: recovery_evidence_available(),
        local_approval: recovery_evidence_available(),
        loader: recovery_evidence_available(),
        rollback_evidence: recovery_evidence_available(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_evidence_available() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_missing() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: false,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_rejected(reason: &'static str) -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: false,
        binding_reason: reason,
    }
}
