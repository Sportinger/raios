use crate::event_log::{
    Event, EventBindings, EventId, EventLog, HelloRecoveryRollbackInspectSourceReferenceBinding,
    HelloRecoveryRollbackInspectSourceReferenceCheck,
    HelloRecoveryRollbackInspectSourceReferenceSelfTestCase,
    HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_CASES,
};
use crate::event_log_evidence::HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_EVIDENCE;
use spin::Mutex;

static SELFTEST_LOG: Mutex<EventLog> = Mutex::new(EventLog::new());

pub(crate) fn hello_recovery_rollback_inspect_source_reference_selftest(
) -> [HelloRecoveryRollbackInspectSourceReferenceSelfTestCase;
       HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_CASES] {
    [
        with_selftest_log(selftest_valid_source_reference),
        with_selftest_log(selftest_missing_source_event_id),
        with_selftest_log(selftest_wrong_source_read_binding),
        with_selftest_log(selftest_missing_audit_event_id),
        with_selftest_log(selftest_wrong_audit_event_variant),
        with_selftest_log(selftest_substituted_hashes),
        with_selftest_log(selftest_authorizing_source_reference_denied),
    ]
}

fn with_selftest_log(
    test: fn(&mut EventLog) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let mut log = SELFTEST_LOG.lock();
    log.clear_for_selftest();
    test(&mut log)
}

fn selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: HelloRecoveryRollbackInspectSourceReferenceCheck,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        source_event_retained: check.source_event_retained,
        audit_event_retained: check.audit_event_retained,
        validated: check.validated,
        passed: check.status == expected_status && check.reason == expected_reason,
    }
}

fn selftest_valid_source_reference(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let audit_event_id = record_source_reference_audit(log, expected);
    selftest_case(
        "valid_source_reference",
        "retained_audit_event_verified",
        "recovery_rollback_inspect_source_reference_ram_audit_verified",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn selftest_missing_source_event_id(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let missing_source_event_id = EventId { sequence: u64::MAX };
    let expected = source_reference_binding(missing_source_event_id, false);
    let audit_event_id = record_source_reference_audit(log, expected);
    selftest_case(
        "missing_or_unretained_source_event_id",
        "rejected",
        "recovery_rollback_inspect_source_event_stale_or_dropped",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn selftest_wrong_source_read_binding(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_wrong_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let audit_event_id = record_source_reference_audit(log, expected);
    selftest_case(
        "wrong_source_read_binding",
        "rejected",
        "recovery_rollback_inspect_source_event_wrong_read_binding",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn selftest_missing_audit_event_id(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let missing_audit_event_id = EventId { sequence: u64::MAX };
    selftest_case(
        "missing_or_unretained_audit_event_id",
        "rejected",
        "recovery_rollback_inspect_source_audit_event_stale_or_dropped",
        log.check_hello_recovery_rollback_inspect_source_reference(
            missing_audit_event_id,
            expected,
        ),
    )
}

fn selftest_wrong_audit_event_variant(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let audit_event_id = record_wrong_audit_variant(log);
    selftest_case(
        "wrong_audit_event_variant",
        "rejected",
        "recovery_rollback_inspect_source_audit_wrong_schema_or_variant",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn selftest_substituted_hashes(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let mut actual = expected;
    actual.reference_hash = tagged_hash(0xaa);
    let audit_event_id = record_source_reference_audit(log, actual);
    selftest_case(
        "substituted_hashes_denied",
        "rejected",
        "recovery_rollback_inspect_source_audit_substituted_or_mismatched",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn selftest_authorizing_source_reference_denied(
    log: &mut EventLog,
) -> HelloRecoveryRollbackInspectSourceReferenceSelfTestCase {
    let source_event_id = record_source_read(log);
    let expected = source_reference_binding(source_event_id, false);
    let actual = source_reference_binding(source_event_id, true);
    let audit_event_id = record_source_reference_audit(log, actual);
    selftest_case(
        "authorizing_source_reference_denied",
        "rejected",
        "recovery_rollback_inspect_source_audit_substituted_or_mismatched",
        log.check_hello_recovery_rollback_inspect_source_reference(audit_event_id, expected),
    )
}

fn source_reference_binding(
    source_event_id: EventId,
    authorizes_rollback_apply: bool,
) -> HelloRecoveryRollbackInspectSourceReferenceBinding {
    HelloRecoveryRollbackInspectSourceReferenceBinding {
        source_event_id,
        reference_hash: tagged_hash(1),
        inspection_hash: tagged_hash(2),
        source_sector_plan_hash: tagged_hash(3),
        source_target_region_write_readback_hash: tagged_hash(4),
        authorizes_rollback_apply,
    }
}

fn record_source_read(log: &mut EventLog) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.recovery_rollback_inspect.read_response",
        source_method: "recovery.rollback_inspect",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "response",
        requested_capability: "cap.recovery.rollback_inspect.read",
        risk: "observe",
        subject: "selftest",
        resource: "svc.demo.hello",
        reason: "synthetic_source_read_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::None,
    })
}

fn record_wrong_source_read(log: &mut EventLog) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.recovery_rollback_inspect.wrong_read_response",
        source_method: "system.describe",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "response",
        requested_capability: "cap.system.describe.read",
        risk: "observe",
        subject: "selftest",
        resource: "svc.demo.hello",
        reason: "synthetic_wrong_source_read",
        evidence: &[],
        bindings: EventBindings::None,
    })
}

fn record_source_reference_audit(
    log: &mut EventLog,
    binding: HelloRecoveryRollbackInspectSourceReferenceBinding,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.recovery_rollback_inspect_source_reference",
        source_method: "recovery.rollback_inspect_source_reference_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_global_evidence",
        requested_capability: "cap.recovery.rollback_inspect.read",
        risk: "observe",
        subject: "selftest",
        resource: "svc.demo.hello",
        reason: "synthetic_source_reference_not_global_evidence",
        evidence: HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_EVIDENCE,
        bindings: EventBindings::HelloRecoveryRollbackInspectSourceReference(binding),
    })
}

fn record_wrong_audit_variant(log: &mut EventLog) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.recovery_rollback_inspect_source_reference_wrong_variant",
        source_method: "recovery.rollback_inspect_source_reference_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_global_evidence",
        requested_capability: "cap.recovery.rollback_inspect.read",
        risk: "observe",
        subject: "selftest",
        resource: "svc.demo.hello",
        reason: "synthetic_wrong_audit_variant",
        evidence: &[],
        bindings: EventBindings::None,
    })
}

fn tagged_hash(tag: u8) -> [u8; 32] {
    [tag; 32]
}
