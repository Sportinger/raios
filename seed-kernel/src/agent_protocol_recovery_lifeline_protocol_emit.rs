use alloc::vec;

use crate::{
    agent_protocol_recovery_lifeline_protocol_types::{
        RecoveryLifelineProtocolCheck, RecoveryLifelineProtocolSelfTestCase,
    },
    agent_protocol_support::{
        emit_inline_record_object, emit_inline_record_property_at, emit_record_fields,
        emit_record_property_line, record_bool as b, record_event as ev, record_false as no,
        record_field as f, record_non_authorizing_fact_fields, record_null as null,
        record_object as object, record_sha_fields as shas, record_str as s,
    },
    event_log,
};

#[rustfmt::skip]
const PROTOCOL_MISSING_FALSES: &[&str] = &["authorizes_recovery_load", "can_move_beyond_denial", "loads_recovery_loader", "loads_recovery_artifact", "creates_durable_records", "installs_rollback_plan", "allocates_service_slot"];

pub(crate) fn emit_recovery_lifeline_protocol_request_state(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    check: &RecoveryLifelineProtocolCheck,
    comma: bool,
) {
    let fields = if let Some((event_id, reference)) = retained {
        vec![
            f("state", s("present")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", ev(event_id)),
            f("schema", s("raios.recovery_lifeline_request.v0")),
            f(
                "status",
                s(if check.request_chain_valid {
                    "retained_current_boot_hash_reference_only"
                } else {
                    "rejected"
                }),
            ),
            f("reason", s(check.reason)),
            f("classification", s("local_only")),
            f("hash_reference_only", b(true)),
            f("request_chain_valid", b(check.request_chain_valid)),
            f(
                "can_report_protocol_gaps",
                b(check.can_report_protocol_gaps),
            ),
            f("accepts_lifeline_request_json", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_recovery_loader", no()),
            f("loads_recovery_artifact", no()),
            f("authorizes_recovery_load", no()),
            f("creates_durable_records", no()),
            f("installs_rollback_plan", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f(
                "retained_recovery_artifact_identity_event_id",
                ev(reference.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                ev(reference.retained_trust_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_vm_test_event_id",
                ev(reference.retained_vm_test_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_local_approval_event_id",
                ev(reference.retained_local_approval_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_loader_event_id",
                ev(reference.retained_loader_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_rollback_evidence_event_id",
                ev(reference.retained_rollback_evidence_reference_event_id),
            ),
            f(
                "hashes",
                object(shas(&[
                    (
                        "lifeline_request_reference_hash",
                        reference.lifeline_request_reference_hash,
                    ),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("vm_test_reference_hash", reference.vm_test_reference_hash),
                    (
                        "local_approval_reference_hash",
                        reference.local_approval_reference_hash,
                    ),
                    ("loader_reference_hash", reference.loader_reference_hash),
                    (
                        "rollback_evidence_reference_hash",
                        reference.rollback_evidence_reference_hash,
                    ),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                    ("vm_test_hash", reference.vm_test_hash),
                    ("local_approval_hash", reference.local_approval_hash),
                    ("loader_hash", reference.loader_hash),
                    ("rollback_evidence_hash", reference.rollback_evidence_hash),
                ])),
            ),
        ]
    } else {
        vec![
            f("state", s("missing")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", null()),
            f("schema", s("raios.recovery_lifeline_request.v0")),
            f("status", s("missing")),
            f("reason", s("recovery_lifeline_request_event_id_missing")),
            f("classification", s("local_only")),
            f("request_chain_valid", no()),
            f("can_report_protocol_gaps", no()),
            f("can_move_beyond_denial", no()),
            f("load_attempted", no()),
        ]
    };

    emit_record_property_line("retained_recovery_lifeline_request", fields, comma);
}

pub(crate) fn emit_recovery_lifeline_protocol_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    emit_inline_record_property_at(
        field,
        record_non_authorizing_fact_fields(schema, false, reason, PROTOCOL_MISSING_FALSES),
        8,
        comma,
    );
}

pub(crate) fn emit_recovery_lifeline_protocol_check(check: &RecoveryLifelineProtocolCheck) {
    emit_record_fields(
        vec![
            f("status", s(check.status)),
            f("reason", s(check.reason)),
            f("request_chain_valid", b(check.request_chain_valid)),
            f(
                "can_report_protocol_gaps",
                b(check.can_report_protocol_gaps),
            ),
            f(
                "authorizes_recovery_load",
                b(check.authorizes_recovery_load),
            ),
            f("can_move_beyond_denial", b(check.can_move_beyond_denial)),
            f("loads_recovery_loader", b(check.loads_recovery_loader)),
            f("loads_recovery_artifact", b(check.loads_recovery_artifact)),
            f("creates_durable_records", b(check.creates_durable_records)),
            f("installs_rollback_plan", b(check.installs_rollback_plan)),
            f("allocates_service_slot", b(check.allocates_service_slot)),
            f(
                "service_inventory_change",
                s(check.service_inventory_change),
            ),
            f("durable_audit_write_attempted", no()),
            f("rollback_install_attempted", no()),
            f("service_slot_allocation_attempted", no()),
            f("direct_openai_recovery_shortcut_accepted", no()),
            f("load_attempted", b(check.load_attempted)),
        ],
        8,
    );
}

#[rustfmt::skip]
pub(crate) fn emit_recovery_lifeline_protocol_selftest_case(case: &RecoveryLifelineProtocolSelfTestCase, comma: bool) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("authorizes_recovery_load", no()), f("can_move_beyond_denial", no()), f("loads_recovery_loader", no()), f("loads_recovery_artifact", no()), f("creates_durable_records", no()), f("installs_rollback_plan", no()), f("allocates_service_slot", no()), f("service_inventory_change", s("none")), f("load_attempted", no())], comma);
}
