use crate::{
    agent_protocol_recovery_artifact_types::{
        RecoveryIdentitySelfTestCase, RecoveryLifelineRequestSelfTestCase,
        RecoveryLoaderSelfTestCase, RecoveryLocalApprovalSelfTestCase,
        RecoveryRollbackEvidenceSelfTestCase, RecoveryTrustSelfTestCase,
        RecoveryVmTestSelfTestCase,
    },
    agent_protocol_support::{crlf, json_str, raw, raw_bool},
};

pub(crate) fn emit_recovery_identity_selftest_case(
    case: &RecoveryIdentitySelfTestCase,
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_trust_selftest_case(case: &RecoveryTrustSelfTestCase, comma: bool) {
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_vm_test_selftest_case(case: &RecoveryVmTestSelfTestCase, comma: bool) {
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_local_approval_selftest_case(
    case: &RecoveryLocalApprovalSelfTestCase,
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_loader_selftest_case(case: &RecoveryLoaderSelfTestCase, comma: bool) {
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_rollback_evidence_selftest_case(
    case: &RecoveryRollbackEvidenceSelfTestCase,
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_lifeline_request_selftest_case(
    case: &RecoveryLifelineRequestSelfTestCase,
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
