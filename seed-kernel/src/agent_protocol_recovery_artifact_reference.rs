use crate::{
    agent_protocol_recovery::{
        recovery_load_binding_retained_loader_mismatch,
        recovery_load_binding_retained_local_approval_mismatch,
        recovery_load_binding_retained_rollback_evidence_mismatch,
    },
    agent_protocol_recovery_artifact_types::{
        RecoveryIdentityReferenceCheck, RecoveryIdentitySelfTestCase,
        RecoveryLifelineRequestReferenceCheck, RecoveryLifelineRequestReferenceInput,
        RecoveryLifelineRequestSelfTestCase, RecoveryLoaderReferenceCheck,
        RecoveryLoaderReferenceInput, RecoveryLoaderSelfTestCase,
        RecoveryLocalApprovalReferenceCheck, RecoveryLocalApprovalReferenceInput,
        RecoveryLocalApprovalSelfTestCase, RecoveryRollbackEvidenceReferenceCheck,
        RecoveryRollbackEvidenceReferenceInput, RecoveryRollbackEvidenceSelfTestCase,
        RecoveryTrustReferenceCheck, RecoveryTrustReferenceInput, RecoveryTrustSelfTestCase,
        RecoveryVmTestReferenceCheck, RecoveryVmTestReferenceInput, RecoveryVmTestSelfTestCase,
    },
    agent_protocol_recovery_constants::{
        RECOVERY_IDENTITY_SELFTEST_CASES, RECOVERY_LIFELINE_REQUEST_SELFTEST_CASES,
        RECOVERY_LOADER_SELFTEST_CASES, RECOVERY_LOCAL_APPROVAL_SELFTEST_CASES,
        RECOVERY_ROLLBACK_EVIDENCE_SELFTEST_CASES, RECOVERY_TRUST_SELFTEST_CASES,
        RECOVERY_VM_TEST_SELFTEST_CASES,
    },
    agent_protocol_support::{
        current_boot_event_id_str, method_eq, parse_current_boot_event_id, parse_sha256_ref,
    },
    event_log, module_evidence,
};

pub(crate) fn parse_recovery_identity_reference(arg: &str) -> RecoveryIdentityReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryIdentityReferenceCheck {
        has_reference: identity_reference_hash.is_some(),
        arity_valid: identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && extra.is_none(),
        scope,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "recovery_artifact_identity_reference_absent",
        valid: false,
    };
    evaluate_recovery_identity_reference(input)
}

pub(crate) fn evaluate_recovery_identity_reference(
    input: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentityReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_identity_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_identity_reference_absent",
            false,
        );
    }
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_identity_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_identity_reference_invalid_hash"
            } else {
                "recovery_artifact_identity_reference_absent"
            },
            false,
        );
    };
    if !input.arity_valid {
        return recovery_identity_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_identity_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_identity_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            false,
        );
    }
    let expected =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    if input.identity_reference_hash != Some(expected) {
        return recovery_identity_reference_check(
            input,
            Some(expected),
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            false,
        );
    }
    recovery_identity_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
        true,
    )
}

pub(crate) fn recovery_identity_reference_check<'a>(
    input: RecoveryIdentityReferenceCheck<'a>,
    expected_identity_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryIdentityReferenceCheck<'a> {
    RecoveryIdentityReferenceCheck {
        expected_identity_reference_hash,
        status,
        reason,
        valid,
        ..input
    }
}

pub(crate) fn parse_recovery_trust_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let trust_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryTrustReferenceInput {
        has_reference: trust_reference_hash.is_some(),
        arity_valid: trust_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && extra.is_none(),
        scope,
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_trust_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_trust_reference(
    input: RecoveryTrustReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_trust_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_trust_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_trust_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_trust_reference_invalid_hash"
            } else {
                "recovery_artifact_trust_reference_absent"
            },
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_trust_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_trust_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    if input.trust_reference_hash != Some(expected) {
        return recovery_trust_reference_check(
            input,
            Some(expected),
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_trust_live_identity_mismatch(&input) {
            return recovery_trust_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_trust_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
        true,
    )
}

pub(crate) fn recovery_trust_reference_check<'a>(
    input: RecoveryTrustReferenceInput<'a>,
    expected_trust_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryTrustReferenceCheck<'a> {
    RecoveryTrustReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        trust_reference_hash: input.trust_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        expected_trust_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_trust_live_identity_mismatch(
    input: &RecoveryTrustReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let Some((latest_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    if latest_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    None
}

pub(crate) fn parse_recovery_vm_test_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryVmTestReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let vm_test_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryVmTestReferenceInput {
        has_reference: vm_test_reference_hash.is_some(),
        arity_valid: vm_test_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && extra.is_none(),
        scope,
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_vm_test_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_vm_test_reference(
    input: RecoveryVmTestReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryVmTestReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_vm_test_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_vm_test_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_vm_test_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_vm_test_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_vm_test_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_vm_test_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_vm_test_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_vm_test_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    if input.vm_test_reference_hash != Some(expected) {
        return recovery_vm_test_reference_check(
            input,
            Some(expected),
            "mismatched_vm_test_reference_hash",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_vm_test_live_chain_mismatch(&input) {
            return recovery_vm_test_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_vm_test_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing",
        true,
    )
}

pub(crate) fn recovery_vm_test_reference_check<'a>(
    input: RecoveryVmTestReferenceInput<'a>,
    expected_vm_test_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryVmTestReferenceCheck<'a> {
    RecoveryVmTestReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        vm_test_reference_hash: input.vm_test_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        expected_vm_test_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_vm_test_live_chain_mismatch(
    input: &RecoveryVmTestReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let Some((latest_identity_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) =
        event_log::latest_recovery_artifact_trust_reference()
    else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if trust_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_trust_reference_hash_mismatch");
    }
    if Some(trust_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if Some(trust_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_trust_hash_mismatch");
    }
    None
}

pub(crate) fn parse_recovery_local_approval_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLocalApprovalReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let local_approval_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLocalApprovalReferenceInput {
        has_reference: local_approval_reference_hash.is_some(),
        arity_valid: local_approval_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && extra.is_none(),
        scope,
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_local_approval_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_local_approval_reference(
    input: RecoveryLocalApprovalReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLocalApprovalReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_local_approval_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_local_approval_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_local_approval_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_local_approval_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_local_approval_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_local_approval_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_local_approval_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_local_approval_reference_hash(
        module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
        },
    );
    if input.local_approval_reference_hash != Some(expected) {
        return recovery_local_approval_reference_check(
            input,
            Some(expected),
            "mismatched_local_approval_reference_hash",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_local_approval_live_chain_mismatch(&input) {
            return recovery_local_approval_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_local_approval_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_local_approval_reference_valid_but_loader_missing",
        true,
    )
}

pub(crate) fn recovery_local_approval_reference_check<'a>(
    input: RecoveryLocalApprovalReferenceInput<'a>,
    expected_local_approval_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLocalApprovalReferenceCheck<'a> {
    RecoveryLocalApprovalReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        local_approval_reference_hash: input.local_approval_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        expected_local_approval_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_local_approval_live_chain_mismatch(
    input: &RecoveryLocalApprovalReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let Some((latest_identity_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) =
        event_log::latest_recovery_artifact_trust_reference()
    else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, vm_test_reference)) =
        event_log::latest_recovery_artifact_vm_test_reference()
    else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if trust_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_identity_reference_event_id != latest_identity_event_id {
        return Some("recovery_artifact_vm_test_identity_event_id_mismatch");
    }
    if vm_test_reference.retained_trust_reference_event_id != latest_trust_event_id {
        return Some("recovery_artifact_vm_test_trust_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_trust_reference_hash_mismatch");
    }
    if Some(trust_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if Some(trust_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    if Some(trust_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_trust_hash_mismatch");
    }
    if Some(vm_test_reference.vm_test_reference_hash) != input.vm_test_reference_hash {
        return Some("recovery_artifact_vm_test_reference_hash_mismatch");
    }
    if Some(vm_test_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_vm_test_identity_reference_hash_mismatch");
    }
    if Some(vm_test_reference.trust_reference_hash) != input.trust_reference_hash {
        return Some("recovery_artifact_vm_test_trust_reference_hash_mismatch");
    }
    if Some(vm_test_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_vm_test_artifact_hash_mismatch");
    }
    if Some(vm_test_reference.trust_hash) != input.trust_hash {
        return Some("recovery_artifact_vm_test_trust_hash_mismatch");
    }
    if Some(vm_test_reference.vm_test_hash) != input.vm_test_hash {
        return Some("recovery_artifact_vm_test_hash_mismatch");
    }
    None
}

pub(crate) fn parse_recovery_loader_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLoaderReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let loader_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLoaderReferenceInput {
        has_reference: loader_reference_hash.is_some(),
        arity_valid: loader_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && extra.is_none(),
        scope,
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_loader_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_loader_reference(
    input: RecoveryLoaderReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLoaderReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_loader_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_loader_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_loader_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_loader_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_loader_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_loader_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_loader_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    if input.loader_reference_hash != Some(expected) {
        return recovery_loader_reference_check(
            input,
            Some(expected),
            "mismatched_loader_reference_hash",
            "recovery_artifact_loader_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_loader_live_chain_mismatch(&input) {
            return recovery_loader_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_loader_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing",
        true,
    )
}

pub(crate) fn recovery_loader_reference_check<'a>(
    input: RecoveryLoaderReferenceInput<'a>,
    expected_loader_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLoaderReferenceCheck<'a> {
    RecoveryLoaderReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        loader_reference_hash: input.loader_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        expected_loader_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_loader_live_chain_mismatch(
    input: &RecoveryLoaderReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let Some((latest_identity_event_id, identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, approval_reference)) = retained_local_approval else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_local_approval_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
    )
    .or_else(|| {
        if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
            Some("recovery_artifact_identity_reference_hash_mismatch")
        } else if Some(trust_reference.trust_reference_hash) != input.trust_reference_hash {
            Some("recovery_artifact_trust_reference_hash_mismatch")
        } else if Some(vm_test_reference.vm_test_reference_hash) != input.vm_test_reference_hash {
            Some("recovery_artifact_vm_test_reference_hash_mismatch")
        } else if Some(approval_reference.local_approval_reference_hash)
            != input.local_approval_reference_hash
        {
            Some("recovery_artifact_local_approval_reference_hash_mismatch")
        } else if Some(approval_reference.local_approval_hash) != input.local_approval_hash {
            Some("recovery_artifact_local_approval_hash_mismatch")
        } else {
            None
        }
    })
}

pub(crate) fn parse_recovery_rollback_evidence_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let rollback_evidence_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let retained_loader_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let loader_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let rollback_evidence_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryRollbackEvidenceReferenceInput {
        has_reference: rollback_evidence_reference_hash.is_some(),
        arity_valid: rollback_evidence_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && retained_loader_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && loader_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && rollback_evidence_hash.is_some()
            && extra.is_none(),
        scope,
        rollback_evidence_reference_hash: rollback_evidence_reference_hash
            .and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        retained_loader_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
        rollback_evidence_hash: rollback_evidence_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_rollback_evidence_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_rollback_evidence_reference(
    input: RecoveryRollbackEvidenceReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_rollback_evidence_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_event_id) = input.retained_loader_reference_event_id else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_reference_hash) = input.loader_reference_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    let Some(rollback_evidence_hash) = input.rollback_evidence_hash else {
        return recovery_rollback_evidence_invalid(input);
    };
    if !input.arity_valid {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_rollback_evidence_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_rollback_evidence_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(loader_event_id) {
        return recovery_rollback_evidence_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
        module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    if input.rollback_evidence_reference_hash != Some(expected) {
        return recovery_rollback_evidence_reference_check(
            input,
            Some(expected),
            "mismatched_rollback_evidence_reference_hash",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_rollback_evidence_live_chain_mismatch(&input) {
            return recovery_rollback_evidence_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_rollback_evidence_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing",
        true,
    )
}

pub(crate) fn recovery_rollback_evidence_invalid(
    input: RecoveryRollbackEvidenceReferenceInput<'_>,
) -> RecoveryRollbackEvidenceReferenceCheck<'_> {
    recovery_rollback_evidence_reference_check(
        input,
        None,
        "invalid_reference",
        "recovery_artifact_rollback_evidence_reference_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_rollback_evidence_reference_check<'a>(
    input: RecoveryRollbackEvidenceReferenceInput<'a>,
    expected_rollback_evidence_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryRollbackEvidenceReferenceCheck<'a> {
    RecoveryRollbackEvidenceReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        rollback_evidence_reference_hash: input.rollback_evidence_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        retained_loader_reference_event_id: input.retained_loader_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        loader_reference_hash: input.loader_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        rollback_evidence_hash: input.rollback_evidence_hash,
        expected_rollback_evidence_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_rollback_evidence_live_chain_mismatch(
    input: &RecoveryRollbackEvidenceReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_loader_reference_event_id =
        parse_current_boot_event_id(input.retained_loader_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let Some((latest_identity_event_id, _identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, _approval_reference)) = retained_local_approval
    else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((latest_loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    if latest_loader_event_id != retained_loader_reference_event_id {
        return Some("recovery_artifact_loader_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_loader_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
    )
    .or_else(|| {
        if let Some((_loader_event_id, loader_reference)) = retained_loader {
            if Some(loader_reference.loader_reference_hash) != input.loader_reference_hash {
                Some("recovery_artifact_loader_reference_hash_mismatch")
            } else if Some(loader_reference.loader_hash) != input.loader_hash {
                Some("recovery_artifact_loader_hash_mismatch")
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub(crate) fn parse_recovery_lifeline_request_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let lifeline_request_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let retained_trust_reference_event_id = parts.next();
    let retained_vm_test_reference_event_id = parts.next();
    let retained_local_approval_reference_event_id = parts.next();
    let retained_loader_reference_event_id = parts.next();
    let retained_rollback_evidence_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let trust_reference_hash = parts.next();
    let vm_test_reference_hash = parts.next();
    let local_approval_reference_hash = parts.next();
    let loader_reference_hash = parts.next();
    let rollback_evidence_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let vm_test_hash = parts.next();
    let local_approval_hash = parts.next();
    let loader_hash = parts.next();
    let rollback_evidence_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineRequestReferenceInput {
        has_reference: lifeline_request_reference_hash.is_some(),
        arity_valid: lifeline_request_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && retained_trust_reference_event_id.is_some()
            && retained_vm_test_reference_event_id.is_some()
            && retained_local_approval_reference_event_id.is_some()
            && retained_loader_reference_event_id.is_some()
            && retained_rollback_evidence_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && trust_reference_hash.is_some()
            && vm_test_reference_hash.is_some()
            && local_approval_reference_hash.is_some()
            && loader_reference_hash.is_some()
            && rollback_evidence_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && vm_test_hash.is_some()
            && local_approval_hash.is_some()
            && loader_hash.is_some()
            && rollback_evidence_hash.is_some()
            && extra.is_none(),
        scope,
        lifeline_request_reference_hash: lifeline_request_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        retained_trust_reference_event_id,
        retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id,
        retained_loader_reference_event_id,
        retained_rollback_evidence_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        vm_test_reference_hash: vm_test_reference_hash.and_then(parse_sha256_ref),
        local_approval_reference_hash: local_approval_reference_hash.and_then(parse_sha256_ref),
        loader_reference_hash: loader_reference_hash.and_then(parse_sha256_ref),
        rollback_evidence_reference_hash: rollback_evidence_reference_hash
            .and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
        vm_test_hash: vm_test_hash.and_then(parse_sha256_ref),
        local_approval_hash: local_approval_hash.and_then(parse_sha256_ref),
        loader_hash: loader_hash.and_then(parse_sha256_ref),
        rollback_evidence_hash: rollback_evidence_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_request_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_request_reference(
    input: RecoveryLifelineRequestReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "missing",
            "recovery_lifeline_request_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_event_id) = input.retained_trust_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_event_id) = input.retained_vm_test_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_event_id) = input.retained_local_approval_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_event_id) = input.retained_loader_reference_event_id else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_event_id) = input.retained_rollback_evidence_reference_event_id
    else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_reference_hash) = input.trust_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_reference_hash) = input.vm_test_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_reference_hash) = input.local_approval_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_reference_hash) = input.loader_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_reference_hash) = input.rollback_evidence_reference_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(vm_test_hash) = input.vm_test_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(local_approval_hash) = input.local_approval_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(loader_hash) = input.loader_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    let Some(rollback_evidence_hash) = input.rollback_evidence_hash else {
        return recovery_lifeline_request_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_lifeline_request_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_request_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(trust_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(vm_test_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(local_approval_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(loader_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(rollback_evidence_event_id) {
        return recovery_lifeline_request_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_rollback_evidence_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            retained_rollback_evidence_reference_event_id: rollback_evidence_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            rollback_evidence_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    if input.lifeline_request_reference_hash != Some(expected) {
        return recovery_lifeline_request_reference_check(
            input,
            Some(expected),
            "mismatched_lifeline_request_reference_hash",
            "recovery_lifeline_request_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_request_live_chain_mismatch(&input) {
            return recovery_lifeline_request_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_request_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing",
        true,
    )
}

pub(crate) fn recovery_lifeline_request_invalid(
    input: RecoveryLifelineRequestReferenceInput<'_>,
) -> RecoveryLifelineRequestReferenceCheck<'_> {
    recovery_lifeline_request_reference_check(
        input,
        None,
        "invalid_reference",
        "recovery_lifeline_request_reference_invalid_hash",
        false,
    )
}

pub(crate) fn recovery_lifeline_request_reference_check<'a>(
    input: RecoveryLifelineRequestReferenceInput<'a>,
    expected_lifeline_request_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineRequestReferenceCheck<'a> {
    RecoveryLifelineRequestReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        lifeline_request_reference_hash: input.lifeline_request_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        retained_trust_reference_event_id: input.retained_trust_reference_event_id,
        retained_vm_test_reference_event_id: input.retained_vm_test_reference_event_id,
        retained_local_approval_reference_event_id: input
            .retained_local_approval_reference_event_id,
        retained_loader_reference_event_id: input.retained_loader_reference_event_id,
        retained_rollback_evidence_reference_event_id: input
            .retained_rollback_evidence_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        trust_reference_hash: input.trust_reference_hash,
        vm_test_reference_hash: input.vm_test_reference_hash,
        local_approval_reference_hash: input.local_approval_reference_hash,
        loader_reference_hash: input.loader_reference_hash,
        rollback_evidence_reference_hash: input.rollback_evidence_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        vm_test_hash: input.vm_test_hash,
        local_approval_hash: input.local_approval_hash,
        loader_hash: input.loader_hash,
        rollback_evidence_hash: input.rollback_evidence_hash,
        expected_lifeline_request_reference_hash,
        status,
        reason,
        valid,
    }
}

pub(crate) fn recovery_lifeline_request_live_chain_mismatch(
    input: &RecoveryLifelineRequestReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let retained_trust_reference_event_id =
        parse_current_boot_event_id(input.retained_trust_reference_event_id?)?;
    let retained_vm_test_reference_event_id =
        parse_current_boot_event_id(input.retained_vm_test_reference_event_id?)?;
    let retained_local_approval_reference_event_id =
        parse_current_boot_event_id(input.retained_local_approval_reference_event_id?)?;
    let retained_loader_reference_event_id =
        parse_current_boot_event_id(input.retained_loader_reference_event_id?)?;
    let retained_rollback_evidence_reference_event_id =
        parse_current_boot_event_id(input.retained_rollback_evidence_reference_event_id?)?;
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let Some((latest_identity_event_id, _identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((latest_trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((latest_vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((latest_local_approval_event_id, _approval_reference)) = retained_local_approval
    else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((latest_loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    let Some((latest_rollback_event_id, _rollback_reference)) = retained_rollback_evidence else {
        return Some("recovery_artifact_rollback_evidence_reference_missing");
    };
    if latest_identity_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if latest_trust_event_id != retained_trust_reference_event_id {
        return Some("recovery_artifact_trust_reference_event_id_mismatch");
    }
    if latest_vm_test_event_id != retained_vm_test_reference_event_id {
        return Some("recovery_artifact_vm_test_reference_event_id_mismatch");
    }
    if latest_local_approval_event_id != retained_local_approval_reference_event_id {
        return Some("recovery_artifact_local_approval_reference_event_id_mismatch");
    }
    if latest_loader_event_id != retained_loader_reference_event_id {
        return Some("recovery_artifact_loader_reference_event_id_mismatch");
    }
    if latest_rollback_event_id != retained_rollback_evidence_reference_event_id {
        return Some("recovery_artifact_rollback_evidence_reference_event_id_mismatch");
    }
    recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    )
    .or_else(|| {
        if let Some((_rollback_event_id, rollback_reference)) = retained_rollback_evidence {
            if Some(rollback_reference.rollback_evidence_reference_hash)
                != input.rollback_evidence_reference_hash
            {
                Some("recovery_artifact_rollback_evidence_reference_hash_mismatch")
            } else if Some(rollback_reference.rollback_evidence_hash)
                != input.rollback_evidence_hash
            {
                Some("recovery_artifact_rollback_evidence_hash_mismatch")
            } else if Some(rollback_reference.loader_reference_hash) != input.loader_reference_hash
            {
                Some("recovery_artifact_loader_reference_hash_mismatch")
            } else if Some(rollback_reference.loader_hash) != input.loader_hash {
                Some("recovery_artifact_loader_hash_mismatch")
            } else if Some(rollback_reference.local_approval_reference_hash)
                != input.local_approval_reference_hash
            {
                Some("recovery_artifact_local_approval_reference_hash_mismatch")
            } else if Some(rollback_reference.local_approval_hash) != input.local_approval_hash {
                Some("recovery_artifact_local_approval_hash_mismatch")
            } else if Some(rollback_reference.vm_test_reference_hash)
                != input.vm_test_reference_hash
            {
                Some("recovery_artifact_vm_test_reference_hash_mismatch")
            } else if Some(rollback_reference.vm_test_hash) != input.vm_test_hash {
                Some("recovery_artifact_vm_test_hash_mismatch")
            } else if Some(rollback_reference.trust_reference_hash) != input.trust_reference_hash {
                Some("recovery_artifact_trust_reference_hash_mismatch")
            } else if Some(rollback_reference.trust_hash) != input.trust_hash {
                Some("recovery_artifact_trust_hash_mismatch")
            } else if Some(rollback_reference.identity_reference_hash)
                != input.identity_reference_hash
            {
                Some("recovery_artifact_identity_reference_hash_mismatch")
            } else if Some(rollback_reference.artifact_hash) != input.artifact_hash {
                Some("recovery_artifact_identity_artifact_hash_mismatch")
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub(crate) fn recovery_identity_selftest_cases(
) -> [RecoveryIdentitySelfTestCase; RECOVERY_IDENTITY_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let valid_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let valid = RecoveryIdentityReferenceCheck {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        identity_reference_hash: Some(valid_hash),
        artifact_hash: Some(artifact_hash),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "missing",
        valid: false,
    };
    [
        recovery_identity_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_identity_reference_absent",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                has_reference: false,
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "accepted_current_boot_identity_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
            evaluate_recovery_identity_reference(valid),
        ),
        recovery_identity_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                scope: "previous_boot",
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "wrong_schema_identity_reference",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_wrong_schema_or_variant",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_substituted_record",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "identity_reference_hash_mismatch",
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                identity_reference_hash: Some([0x92; 32]),
                ..valid
            }),
        ),
    ]
}

pub(crate) fn recovery_identity_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentitySelfTestCase {
    RecoveryIdentitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_trust_selftest_cases(
) -> [RecoveryTrustSelfTestCase; RECOVERY_TRUST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let valid_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let valid = RecoveryTrustReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        trust_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
    };
    [
        recovery_trust_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_trust_reference_absent",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "accepted_current_boot_trust_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
            evaluate_recovery_trust_reference(valid, false),
        ),
        recovery_trust_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_substituted_record",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "trust_reference_hash_mismatch",
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    trust_reference_hash: Some([0x94; 32]),
                    ..valid
                },
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_trust_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryTrustReferenceCheck<'_>,
) -> RecoveryTrustSelfTestCase {
    RecoveryTrustSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_vm_test_selftest_cases(
) -> [RecoveryVmTestSelfTestCase; RECOVERY_VM_TEST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let valid = RecoveryVmTestReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        vm_test_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
    };
    [
        recovery_vm_test_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_vm_test_reference_absent",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "accepted_current_boot_vm_test_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing",
            evaluate_recovery_vm_test_reference(valid, false),
        ),
        recovery_vm_test_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_vm_test_reference_scope_must_be_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_trust_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_trust_event_id_not_current_boot",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    retained_trust_reference_event_id: Some("event.previous_boot.00000032"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "retained_trust_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_trust_reference_wrong_schema_or_variant",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "substituted_trust_reference_record",
            "rejected",
            "recovery_artifact_trust_reference_substituted_record",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_substituted_record",
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "vm_test_reference_hash_mismatch",
            "mismatched_vm_test_reference_hash",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            evaluate_recovery_vm_test_reference(
                RecoveryVmTestReferenceInput {
                    vm_test_reference_hash: Some([0x96; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_vm_test_selftest_case(
            "trust_binding_mismatch",
            "rejected",
            "recovery_artifact_trust_identity_event_id_mismatch",
            recovery_vm_test_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_identity_event_id_mismatch",
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_vm_test_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryVmTestReferenceCheck<'_>,
) -> RecoveryVmTestSelfTestCase {
    RecoveryVmTestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_local_approval_selftest_cases(
) -> [RecoveryLocalApprovalSelfTestCase; RECOVERY_LOCAL_APPROVAL_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_local_approval_reference_hash(
        module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
        },
    );
    let valid = RecoveryLocalApprovalReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        local_approval_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
    };
    [
        recovery_local_approval_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_local_approval_reference_absent",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "accepted_current_boot_local_approval_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_local_approval_reference_valid_but_loader_missing",
            evaluate_recovery_local_approval_reference(valid, false),
        ),
        recovery_local_approval_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_local_approval_reference_scope_must_be_current_boot",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_vm_test_event_id_not_current_boot",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    retained_vm_test_reference_event_id: Some("event.previous_boot.00000033"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_missing",
            "rejected",
            "recovery_artifact_vm_test_reference_missing",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_missing",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_vm_test_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_vm_test_reference_wrong_schema_or_variant",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "substituted_vm_test_reference_record",
            "rejected",
            "recovery_artifact_vm_test_reference_substituted_record",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_substituted_record",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "local_approval_reference_hash_mismatch",
            "mismatched_local_approval_reference_hash",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            evaluate_recovery_local_approval_reference(
                RecoveryLocalApprovalReferenceInput {
                    local_approval_reference_hash: Some([0x98; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "vm_test_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_vm_test_reference_hash_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_vm_test_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "trust_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_trust_reference_hash_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_trust_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_local_approval_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_local_approval_vm_test_event_id_mismatch",
            recovery_local_approval_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_vm_test_event_id_mismatch",
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_local_approval_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLocalApprovalReferenceCheck<'_>,
) -> RecoveryLocalApprovalSelfTestCase {
    RecoveryLocalApprovalSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_loader_selftest_cases(
) -> [RecoveryLoaderSelfTestCase; RECOVERY_LOADER_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let valid_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let valid = RecoveryLoaderReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        loader_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
    };
    [
        recovery_loader_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_loader_reference_absent",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "accepted_current_boot_loader_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing",
            evaluate_recovery_loader_reference(valid, false),
        ),
        recovery_loader_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_loader_reference_scope_must_be_current_boot",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_local_approval_event_id_not_current_boot",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    retained_local_approval_reference_event_id: Some(
                        "event.previous_boot.00000034",
                    ),
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_missing",
            "rejected",
            "recovery_artifact_local_approval_reference_missing",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_missing",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_local_approval_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_local_approval_reference_wrong_schema_or_variant",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "substituted_local_approval_reference_record",
            "rejected",
            "recovery_artifact_local_approval_reference_substituted_record",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_substituted_record",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "loader_reference_hash_mismatch",
            "mismatched_loader_reference_hash",
            "recovery_artifact_loader_reference_hash_mismatch",
            evaluate_recovery_loader_reference(
                RecoveryLoaderReferenceInput {
                    loader_reference_hash: Some([0x9a; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "local_approval_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_local_approval_reference_hash_mismatch",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_local_approval_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_loader_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_loader_local_approval_event_id_mismatch",
            recovery_loader_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_local_approval_event_id_mismatch",
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_loader_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLoaderReferenceCheck<'_>,
) -> RecoveryLoaderSelfTestCase {
    RecoveryLoaderSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_rollback_evidence_selftest_cases(
) -> [RecoveryRollbackEvidenceSelfTestCase; RECOVERY_ROLLBACK_EVIDENCE_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let rollback_evidence_hash = [0x9b; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let loader_event_id = "event.current_boot.00000035";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let loader_reference_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let valid_hash = module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
        module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    let valid = RecoveryRollbackEvidenceReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        rollback_evidence_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        retained_loader_reference_event_id: Some(loader_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        loader_reference_hash: Some(loader_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
        rollback_evidence_hash: Some(rollback_evidence_hash),
    };
    [
        recovery_rollback_evidence_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_rollback_evidence_reference_absent",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "accepted_current_boot_rollback_evidence_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing",
            evaluate_recovery_rollback_evidence_reference(valid, false),
        ),
        recovery_rollback_evidence_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_rollback_evidence_reference_scope_must_be_current_boot",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_loader_event_id_not_current_boot",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    retained_loader_reference_event_id: Some("event.previous_boot.00000035"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_missing",
            "rejected",
            "recovery_artifact_loader_reference_missing",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_missing",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_loader_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_loader_reference_wrong_schema_or_variant",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "substituted_loader_reference_record",
            "rejected",
            "recovery_artifact_loader_reference_substituted_record",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_substituted_record",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "mismatched_rollback_evidence_reference_hash",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            evaluate_recovery_rollback_evidence_reference(
                RecoveryRollbackEvidenceReferenceInput {
                    rollback_evidence_reference_hash: Some([0x9c; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "loader_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_loader_reference_hash_mismatch",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_loader_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_rollback_evidence_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
            recovery_rollback_evidence_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_rollback_evidence_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryRollbackEvidenceReferenceCheck<'_>,
) -> RecoveryRollbackEvidenceSelfTestCase {
    RecoveryRollbackEvidenceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_lifeline_request_selftest_cases(
) -> [RecoveryLifelineRequestSelfTestCase; RECOVERY_LIFELINE_REQUEST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let vm_test_hash = [0x95; 32];
    let local_approval_hash = [0x97; 32];
    let loader_hash = [0x99; 32];
    let rollback_evidence_hash = [0x9b; 32];
    let identity_event_id = "event.current_boot.00000031";
    let trust_event_id = "event.current_boot.00000032";
    let vm_test_event_id = "event.current_boot.00000033";
    let local_approval_event_id = "event.current_boot.00000034";
    let loader_event_id = "event.current_boot.00000035";
    let rollback_evidence_event_id = "event.current_boot.00000036";
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let trust_reference_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let vm_test_reference_hash = module_evidence::computed_recovery_artifact_vm_test_reference_hash(
        module_evidence::RecoveryArtifactVmTestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            identity_reference_hash,
            trust_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
        },
    );
    let local_approval_reference_hash =
        module_evidence::computed_recovery_artifact_local_approval_reference_hash(
            module_evidence::RecoveryArtifactLocalApprovalReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
            },
        );
    let loader_reference_hash = module_evidence::computed_recovery_artifact_loader_reference_hash(
        module_evidence::RecoveryArtifactLoaderReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
        },
    );
    let rollback_evidence_reference_hash =
        module_evidence::computed_recovery_artifact_rollback_evidence_reference_hash(
            module_evidence::RecoveryArtifactRollbackEvidenceReferenceHashInput {
                retained_identity_reference_event_id: identity_event_id,
                retained_trust_reference_event_id: trust_event_id,
                retained_vm_test_reference_event_id: vm_test_event_id,
                retained_local_approval_reference_event_id: local_approval_event_id,
                retained_loader_reference_event_id: loader_event_id,
                identity_reference_hash,
                trust_reference_hash,
                vm_test_reference_hash,
                local_approval_reference_hash,
                loader_reference_hash,
                artifact_hash,
                trust_hash,
                vm_test_hash,
                local_approval_hash,
                loader_hash,
                rollback_evidence_hash,
            },
        );
    let valid_hash = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            retained_trust_reference_event_id: trust_event_id,
            retained_vm_test_reference_event_id: vm_test_event_id,
            retained_local_approval_reference_event_id: local_approval_event_id,
            retained_loader_reference_event_id: loader_event_id,
            retained_rollback_evidence_reference_event_id: rollback_evidence_event_id,
            identity_reference_hash,
            trust_reference_hash,
            vm_test_reference_hash,
            local_approval_reference_hash,
            loader_reference_hash,
            rollback_evidence_reference_hash,
            artifact_hash,
            trust_hash,
            vm_test_hash,
            local_approval_hash,
            loader_hash,
            rollback_evidence_hash,
        },
    );
    let valid = RecoveryLifelineRequestReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        lifeline_request_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        retained_trust_reference_event_id: Some(trust_event_id),
        retained_vm_test_reference_event_id: Some(vm_test_event_id),
        retained_local_approval_reference_event_id: Some(local_approval_event_id),
        retained_loader_reference_event_id: Some(loader_event_id),
        retained_rollback_evidence_reference_event_id: Some(rollback_evidence_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        trust_reference_hash: Some(trust_reference_hash),
        vm_test_reference_hash: Some(vm_test_reference_hash),
        local_approval_reference_hash: Some(local_approval_reference_hash),
        loader_reference_hash: Some(loader_reference_hash),
        rollback_evidence_reference_hash: Some(rollback_evidence_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
        vm_test_hash: Some(vm_test_hash),
        local_approval_hash: Some(local_approval_hash),
        loader_hash: Some(loader_hash),
        rollback_evidence_hash: Some(rollback_evidence_hash),
    };
    [
        recovery_lifeline_request_selftest_case(
            "absent_reference",
            "missing",
            "recovery_lifeline_request_reference_absent",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "accepted_current_boot_lifeline_request_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing",
            evaluate_recovery_lifeline_request_reference(valid, false),
        ),
        recovery_lifeline_request_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_lifeline_request_reference_scope_must_be_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_rollback_evidence_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_rollback_evidence_event_id_not_current_boot",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    retained_rollback_evidence_reference_event_id: Some(
                        "event.previous_boot.00000036",
                    ),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_rollback_evidence_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_wrong_schema_or_variant",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "substituted_rollback_evidence_reference_record",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_substituted_record",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_substituted_record",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "lifeline_request_reference_hash_mismatch",
            "mismatched_lifeline_request_reference_hash",
            "recovery_lifeline_request_reference_hash_mismatch",
            evaluate_recovery_lifeline_request_reference(
                RecoveryLifelineRequestReferenceInput {
                    lifeline_request_reference_hash: Some([0x9d; 32]),
                    ..valid
                },
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "rollback_evidence_reference_hash_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_reference_hash_mismatch",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_reference_hash_mismatch",
                false,
            ),
        ),
        recovery_lifeline_request_selftest_case(
            "retained_chain_mismatch",
            "rejected",
            "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
            recovery_lifeline_request_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_rollback_evidence_loader_event_id_mismatch",
                false,
            ),
        ),
    ]
}

pub(crate) fn recovery_lifeline_request_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryLifelineRequestReferenceCheck<'_>,
) -> RecoveryLifelineRequestSelfTestCase {
    RecoveryLifelineRequestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

pub(crate) fn recovery_identity_binding_from_check(
    check: &RecoveryIdentityReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactIdentityReference> {
    Some(event_log::RecoveryArtifactIdentityReference {
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
    })
}

pub(crate) fn recovery_trust_binding_from_check(
    check: &RecoveryTrustReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactTrustReference> {
    Some(event_log::RecoveryArtifactTrustReference {
        trust_reference_hash: check.trust_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
    })
}

pub(crate) fn recovery_vm_test_binding_from_check(
    check: &RecoveryVmTestReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactVmTestReference> {
    Some(event_log::RecoveryArtifactVmTestReference {
        vm_test_reference_hash: check.vm_test_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
    })
}

pub(crate) fn recovery_local_approval_binding_from_check(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactLocalApprovalReference> {
    Some(event_log::RecoveryArtifactLocalApprovalReference {
        local_approval_reference_hash: check.local_approval_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
    })
}

pub(crate) fn recovery_loader_binding_from_check(
    check: &RecoveryLoaderReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactLoaderReference> {
    Some(event_log::RecoveryArtifactLoaderReference {
        loader_reference_hash: check.loader_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
    })
}

pub(crate) fn recovery_rollback_evidence_binding_from_check(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactRollbackEvidenceReference> {
    Some(event_log::RecoveryArtifactRollbackEvidenceReference {
        rollback_evidence_reference_hash: check.rollback_evidence_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        retained_loader_reference_event_id: parse_current_boot_event_id(
            check.retained_loader_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        loader_reference_hash: check.loader_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
        rollback_evidence_hash: check.rollback_evidence_hash?,
    })
}

pub(crate) fn recovery_lifeline_request_binding_from_check(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineRequestReference> {
    Some(event_log::RecoveryLifelineRequestReference {
        lifeline_request_reference_hash: check.lifeline_request_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        retained_trust_reference_event_id: parse_current_boot_event_id(
            check.retained_trust_reference_event_id?,
        )?,
        retained_vm_test_reference_event_id: parse_current_boot_event_id(
            check.retained_vm_test_reference_event_id?,
        )?,
        retained_local_approval_reference_event_id: parse_current_boot_event_id(
            check.retained_local_approval_reference_event_id?,
        )?,
        retained_loader_reference_event_id: parse_current_boot_event_id(
            check.retained_loader_reference_event_id?,
        )?,
        retained_rollback_evidence_reference_event_id: parse_current_boot_event_id(
            check.retained_rollback_evidence_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        trust_reference_hash: check.trust_reference_hash?,
        vm_test_reference_hash: check.vm_test_reference_hash?,
        local_approval_reference_hash: check.local_approval_reference_hash?,
        loader_reference_hash: check.loader_reference_hash?,
        rollback_evidence_reference_hash: check.rollback_evidence_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
        vm_test_hash: check.vm_test_hash?,
        local_approval_hash: check.local_approval_hash?,
        loader_hash: check.loader_hash?,
        rollback_evidence_hash: check.rollback_evidence_hash?,
    })
}
