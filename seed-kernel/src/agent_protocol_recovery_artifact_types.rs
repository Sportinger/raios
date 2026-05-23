#[derive(Clone, Copy)]
pub(crate) struct RecoveryIdentityReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) expected_identity_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryIdentitySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryTrustReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryTrustReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) expected_trust_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryTrustSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryVmTestReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryVmTestReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) expected_vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryVmTestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLocalApprovalReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLocalApprovalReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) expected_local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryLocalApprovalSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoaderReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoaderReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) expected_loader_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryLoaderSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryRollbackEvidenceReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) retained_loader_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryRollbackEvidenceReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) retained_loader_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_hash: Option<[u8; 32]>,
    pub(crate) expected_rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryRollbackEvidenceSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineRequestReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) lifeline_request_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) retained_loader_reference_event_id: Option<&'a str>,
    pub(crate) retained_rollback_evidence_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineRequestReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) lifeline_request_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_identity_reference_event_id: Option<&'a str>,
    pub(crate) retained_trust_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_test_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_approval_reference_event_id: Option<&'a str>,
    pub(crate) retained_loader_reference_event_id: Option<&'a str>,
    pub(crate) retained_rollback_evidence_reference_event_id: Option<&'a str>,
    pub(crate) identity_reference_hash: Option<[u8; 32]>,
    pub(crate) trust_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_test_reference_hash: Option<[u8; 32]>,
    pub(crate) local_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) loader_reference_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) trust_hash: Option<[u8; 32]>,
    pub(crate) vm_test_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) loader_hash: Option<[u8; 32]>,
    pub(crate) rollback_evidence_hash: Option<[u8; 32]>,
    pub(crate) expected_lifeline_request_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct RecoveryLifelineRequestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}
