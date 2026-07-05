use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_recovery_artifact_types::{
        RecoveryIdentityReferenceCheck, RecoveryLifelineRequestReferenceCheck,
        RecoveryLoaderReferenceCheck, RecoveryLocalApprovalReferenceCheck,
        RecoveryRollbackEvidenceReferenceCheck, RecoveryTrustReferenceCheck,
        RecoveryVmTestReferenceCheck,
    },
    agent_protocol_support::{
        emit_record_property, emit_record_property_line, extend_false_fields,
        parse_current_boot_event_id, record_bool as b, record_event as ev, record_event_or_null,
        record_false as no, record_field as f, record_null as null, record_object as object,
        record_sha_fields as shas, record_sha_or_null, record_str as s, record_str_or_null,
    },
    event_log,
};
use raios_core::record::Field;

fn option_hash_fields<'a>(pairs: &[(&'a str, Option<[u8; 32]>)]) -> Vec<Field<'a>> {
    let mut fields = Vec::new();
    let mut idx = 0usize;
    while idx < pairs.len() {
        fields.push(f(pairs[idx].0, record_sha_or_null(pairs[idx].1)));
        idx += 1;
    }
    fields
}

fn add_retained_denials<'a>(fields: &mut Vec<Field<'a>>, names: &'static [&'static str]) {
    extend_false_fields(fields, names);
    fields.push(f("service_inventory_change", s("none")));
    fields.push(f("load_attempted", no()));
}

fn emit_state_reference_object<'a>(
    property: &'static str,
    schema_key: &'static str,
    schema: &'static str,
    has_reference: bool,
    status: &'a str,
    reason: &'a str,
    arity_valid: bool,
    scope: &'a str,
    retained_event_fields: Vec<Field<'a>>,
    hash_fields: Vec<Field<'a>>,
) {
    let mut fields = vec![
        f("state", s(if has_reference { "present" } else { "absent" })),
        f("validation_status", s(status)),
        f("validation_reason", s(reason)),
        f("arity_valid", b(arity_valid)),
        f("scope", s(scope)),
    ];
    fields.extend(retained_event_fields);
    fields.push(f(schema_key, s(schema)));
    fields.push(f("hashes", object(hash_fields)));
    emit_record_property_line(property, fields, false);
}

fn emit_chain_reference_object<'a>(
    property: &'static str,
    schema: &'static str,
    status: &'a str,
    reason: &'a str,
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    false_fields: &'static [&'static str],
    retained_event_fields: Vec<Field<'a>>,
    hash_fields: Vec<Field<'a>>,
) {
    let mut fields = vec![
        f("schema", s(schema)),
        f("validation_status", s(status)),
        f("validation_reason", s(reason)),
        f("present", b(has_reference)),
        f("arity_valid", b(arity_valid)),
        f("scope", s(scope)),
        f("classification", s("local_only")),
        f("hash_reference_only", b(true)),
    ];
    add_retained_denials(&mut fields, false_fields);
    fields.extend(retained_event_fields);
    fields.push(f("hashes", object(hash_fields)));
    emit_record_property_line(property, fields, false);
}

#[allow(clippy::too_many_arguments)]
fn emit_retained_reference<'a>(
    property: &'static str,
    schema: &'static str,
    missing_reason: &'static str,
    recorded_event_id: Option<event_log::EventId>,
    retained_event_id: Option<event_log::EventId>,
    matches_current_reference: bool,
    false_fields: &'static [&'static str],
    retained_event_fields: Vec<Field<'a>>,
    hash_fields: Vec<Field<'a>>,
) {
    let fields = if let Some(event_id) = retained_event_id {
        let mut fields = vec![
            f("state", s("present")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", ev(event_id)),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f("matches_current_reference", b(matches_current_reference)),
            f("schema", s(schema)),
            f("status", s("retained_hash_reference_load_still_denied")),
            f("classification", s("local_only")),
        ];
        add_retained_denials(&mut fields, false_fields);
        fields.extend(retained_event_fields);
        fields.push(f("hashes", object(hash_fields)));
        fields
    } else {
        vec![
            f("state", s("missing")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", null()),
            f("recorded_event_id", null()),
            f("matches_current_reference", no()),
            f("schema", s(schema)),
            f("status", s("missing")),
            f("reason", s(missing_reason)),
            f("can_move_beyond_denial", no()),
            f("load_attempted", no()),
        ]
    };

    emit_record_property(property, fields);
}

pub(crate) fn emit_recovery_identity_reference_object(check: &RecoveryIdentityReferenceCheck<'_>) {
    emit_record_property_line(
        "recovery_artifact_identity_reference",
        vec![
            f(
                "state",
                s(if check.has_reference {
                    "present"
                } else {
                    "absent"
                }),
            ),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("identity_schema", s("raios.recovery_artifact_identity.v0")),
            f(
                "identity_reference_hash",
                record_sha_or_null(check.identity_reference_hash),
            ),
            f(
                "expected_identity_reference_hash",
                record_sha_or_null(check.expected_identity_reference_hash),
            ),
            f("artifact_hash", record_sha_or_null(check.artifact_hash)),
        ],
        false,
    );
}

pub(crate) fn emit_recovery_identity_retained_reference(
    check: &RecoveryIdentityReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_identity_reference",
        "raios.recovery_artifact_identity.v0",
        "no_valid_recovery_artifact_identity_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_identity_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        vec![],
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("artifact_hash", reference.artifact_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_trust_reference_object(check: &RecoveryTrustReferenceCheck<'_>) {
    emit_state_reference_object(
        "recovery_artifact_trust_reference",
        "trust_schema",
        "raios.recovery_artifact_trust.v0",
        check.has_reference,
        check.status,
        check.reason,
        check.arity_valid,
        check.scope,
        vec![f(
            "retained_recovery_artifact_identity_event_id",
            record_str_or_null(check.retained_identity_reference_event_id),
        )],
        option_hash_fields(&[
            ("trust_reference_hash", check.trust_reference_hash),
            (
                "expected_trust_reference_hash",
                check.expected_trust_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_trust_retained_reference(
    check: &RecoveryTrustReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_trust_reference",
        "raios.recovery_artifact_trust.v0",
        "no_valid_recovery_artifact_trust_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_trust_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![f(
                    "retained_recovery_artifact_identity_event_id",
                    ev(reference.retained_identity_reference_event_id),
                )]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_vm_test_reference_object(check: &RecoveryVmTestReferenceCheck<'_>) {
    emit_state_reference_object(
        "recovery_artifact_vm_test_reference",
        "vm_test_schema",
        "raios.recovery_artifact_vm_test.v0",
        check.has_reference,
        check.status,
        check.reason,
        check.arity_valid,
        check.scope,
        vec![
            f(
                "retained_recovery_artifact_identity_event_id",
                record_str_or_null(check.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                record_str_or_null(check.retained_trust_reference_event_id),
            ),
        ],
        option_hash_fields(&[
            ("vm_test_reference_hash", check.vm_test_reference_hash),
            (
                "expected_vm_test_reference_hash",
                check.expected_vm_test_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("trust_reference_hash", check.trust_reference_hash),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
            ("vm_test_hash", check.vm_test_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_vm_test_retained_reference(
    check: &RecoveryVmTestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_vm_test_reference",
        "raios.recovery_artifact_vm_test.v0",
        "no_valid_recovery_artifact_vm_test_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_vm_test_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_vm_test_json",
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![
                    f(
                        "retained_recovery_artifact_identity_event_id",
                        ev(reference.retained_identity_reference_event_id),
                    ),
                    f(
                        "retained_recovery_artifact_trust_event_id",
                        ev(reference.retained_trust_reference_event_id),
                    ),
                ]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    ("vm_test_reference_hash", reference.vm_test_reference_hash),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                    ("vm_test_hash", reference.vm_test_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_local_approval_reference_object(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
) {
    emit_state_reference_object(
        "recovery_artifact_local_approval_reference",
        "local_approval_schema",
        "raios.recovery_artifact_local_approval.v0",
        check.has_reference,
        check.status,
        check.reason,
        check.arity_valid,
        check.scope,
        vec![
            f(
                "retained_recovery_artifact_identity_event_id",
                record_str_or_null(check.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                record_str_or_null(check.retained_trust_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_vm_test_event_id",
                record_str_or_null(check.retained_vm_test_reference_event_id),
            ),
        ],
        option_hash_fields(&[
            (
                "local_approval_reference_hash",
                check.local_approval_reference_hash,
            ),
            (
                "expected_local_approval_reference_hash",
                check.expected_local_approval_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("trust_reference_hash", check.trust_reference_hash),
            ("vm_test_reference_hash", check.vm_test_reference_hash),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
            ("vm_test_hash", check.vm_test_hash),
            ("local_approval_hash", check.local_approval_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_local_approval_retained_reference(
    check: &RecoveryLocalApprovalReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_local_approval_reference",
        "raios.recovery_artifact_local_approval.v0",
        "no_valid_recovery_artifact_local_approval_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_local_approval_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_local_approval_text",
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![
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
                ]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    (
                        "local_approval_reference_hash",
                        reference.local_approval_reference_hash,
                    ),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("vm_test_reference_hash", reference.vm_test_reference_hash),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                    ("vm_test_hash", reference.vm_test_hash),
                    ("local_approval_hash", reference.local_approval_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_loader_reference_object(check: &RecoveryLoaderReferenceCheck<'_>) {
    emit_chain_reference_object(
        "recovery_artifact_loader_reference",
        "raios.recovery_artifact_loader.v0",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        &[
            "accepts_loader_descriptor",
            "accepts_artifact_bytes",
            "loads_recovery_loader",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        vec![
            f(
                "retained_recovery_artifact_identity_event_id",
                record_str_or_null(check.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                record_str_or_null(check.retained_trust_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_vm_test_event_id",
                record_str_or_null(check.retained_vm_test_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_local_approval_event_id",
                record_str_or_null(check.retained_local_approval_reference_event_id),
            ),
        ],
        option_hash_fields(&[
            ("loader_reference_hash", check.loader_reference_hash),
            (
                "expected_loader_reference_hash",
                check.expected_loader_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("trust_reference_hash", check.trust_reference_hash),
            ("vm_test_reference_hash", check.vm_test_reference_hash),
            (
                "local_approval_reference_hash",
                check.local_approval_reference_hash,
            ),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
            ("vm_test_hash", check.vm_test_hash),
            ("local_approval_hash", check.local_approval_hash),
            ("loader_hash", check.loader_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_loader_retained_reference(
    check: &RecoveryLoaderReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_loader_reference",
        "raios.recovery_artifact_loader.v0",
        "no_valid_recovery_artifact_loader_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_loader_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_loader_descriptor",
            "accepts_artifact_bytes",
            "loads_recovery_loader",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![
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
                ]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    ("loader_reference_hash", reference.loader_reference_hash),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("vm_test_reference_hash", reference.vm_test_reference_hash),
                    (
                        "local_approval_reference_hash",
                        reference.local_approval_reference_hash,
                    ),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                    ("vm_test_hash", reference.vm_test_hash),
                    ("local_approval_hash", reference.local_approval_hash),
                    ("loader_hash", reference.loader_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_rollback_evidence_reference_object(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
) {
    emit_chain_reference_object(
        "recovery_artifact_rollback_evidence_reference",
        "raios.recovery_artifact_rollback_evidence.v0",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        &[
            "accepts_rollback_evidence_json",
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
            "creates_durable_records",
            "installs_rollback_plan",
        ],
        vec![
            f(
                "retained_recovery_artifact_identity_event_id",
                record_str_or_null(check.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                record_str_or_null(check.retained_trust_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_vm_test_event_id",
                record_str_or_null(check.retained_vm_test_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_local_approval_event_id",
                record_str_or_null(check.retained_local_approval_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_loader_event_id",
                record_str_or_null(check.retained_loader_reference_event_id),
            ),
        ],
        option_hash_fields(&[
            (
                "rollback_evidence_reference_hash",
                check.rollback_evidence_reference_hash,
            ),
            (
                "expected_rollback_evidence_reference_hash",
                check.expected_rollback_evidence_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("trust_reference_hash", check.trust_reference_hash),
            ("vm_test_reference_hash", check.vm_test_reference_hash),
            (
                "local_approval_reference_hash",
                check.local_approval_reference_hash,
            ),
            ("loader_reference_hash", check.loader_reference_hash),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
            ("vm_test_hash", check.vm_test_hash),
            ("local_approval_hash", check.local_approval_hash),
            ("loader_hash", check.loader_hash),
            ("rollback_evidence_hash", check.rollback_evidence_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_rollback_evidence_retained_reference(
    check: &RecoveryRollbackEvidenceReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_artifact_rollback_evidence_reference",
        "raios.recovery_artifact_rollback_evidence.v0",
        "no_valid_recovery_artifact_rollback_evidence_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_rollback_evidence_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_rollback_evidence_json",
            "accepts_artifact_bytes",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
            "creates_durable_records",
            "installs_rollback_plan",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![
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
                ]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
                    (
                        "rollback_evidence_reference_hash",
                        reference.rollback_evidence_reference_hash,
                    ),
                    ("identity_reference_hash", reference.identity_reference_hash),
                    ("trust_reference_hash", reference.trust_reference_hash),
                    ("vm_test_reference_hash", reference.vm_test_reference_hash),
                    (
                        "local_approval_reference_hash",
                        reference.local_approval_reference_hash,
                    ),
                    ("loader_reference_hash", reference.loader_reference_hash),
                    ("artifact_hash", reference.artifact_hash),
                    ("trust_hash", reference.trust_hash),
                    ("vm_test_hash", reference.vm_test_hash),
                    ("local_approval_hash", reference.local_approval_hash),
                    ("loader_hash", reference.loader_hash),
                    ("rollback_evidence_hash", reference.rollback_evidence_hash),
                ])
            })
            .unwrap_or_else(Vec::new),
    );
}

pub(crate) fn emit_recovery_lifeline_request_reference_object(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
) {
    emit_chain_reference_object(
        "recovery_lifeline_request_reference",
        "raios.recovery_lifeline_request.v0",
        check.status,
        check.reason,
        check.has_reference,
        check.arity_valid,
        check.scope,
        &[
            "accepts_lifeline_request_json",
            "accepts_loader_descriptor",
            "accepts_artifact_bytes",
            "loads_recovery_loader",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
            "creates_durable_records",
            "installs_rollback_plan",
            "allocates_service_slot",
        ],
        vec![
            f(
                "retained_recovery_artifact_identity_event_id",
                record_str_or_null(check.retained_identity_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_trust_event_id",
                record_str_or_null(check.retained_trust_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_vm_test_event_id",
                record_str_or_null(check.retained_vm_test_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_local_approval_event_id",
                record_str_or_null(check.retained_local_approval_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_loader_event_id",
                record_str_or_null(check.retained_loader_reference_event_id),
            ),
            f(
                "retained_recovery_artifact_rollback_evidence_event_id",
                record_str_or_null(check.retained_rollback_evidence_reference_event_id),
            ),
        ],
        option_hash_fields(&[
            (
                "lifeline_request_reference_hash",
                check.lifeline_request_reference_hash,
            ),
            (
                "expected_lifeline_request_reference_hash",
                check.expected_lifeline_request_reference_hash,
            ),
            ("identity_reference_hash", check.identity_reference_hash),
            ("trust_reference_hash", check.trust_reference_hash),
            ("vm_test_reference_hash", check.vm_test_reference_hash),
            (
                "local_approval_reference_hash",
                check.local_approval_reference_hash,
            ),
            ("loader_reference_hash", check.loader_reference_hash),
            (
                "rollback_evidence_reference_hash",
                check.rollback_evidence_reference_hash,
            ),
            ("artifact_hash", check.artifact_hash),
            ("trust_hash", check.trust_hash),
            ("vm_test_hash", check.vm_test_hash),
            ("local_approval_hash", check.local_approval_hash),
            ("loader_hash", check.loader_hash),
            ("rollback_evidence_hash", check.rollback_evidence_hash),
        ]),
    );
}

pub(crate) fn emit_recovery_lifeline_request_retained_reference(
    check: &RecoveryLifelineRequestReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
) {
    let retained_ref = retained.as_ref();
    emit_retained_reference(
        "retained_recovery_lifeline_request_reference",
        "raios.recovery_lifeline_request.v0",
        "no_valid_recovery_lifeline_request_reference_retained",
        recorded_event_id,
        retained_ref.map(|(event_id, _)| *event_id),
        retained_ref
            .map(|(_, reference)| recovery_lifeline_request_reference_matches(check, *reference))
            .unwrap_or(false),
        &[
            "accepts_lifeline_request_json",
            "accepts_loader_descriptor",
            "accepts_artifact_bytes",
            "loads_recovery_loader",
            "authorizes_recovery_load",
            "can_move_beyond_denial",
            "loads_recovery_artifact",
            "creates_durable_records",
            "installs_rollback_plan",
            "allocates_service_slot",
        ],
        retained_ref
            .map(|(_, reference)| {
                vec![
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
                ]
            })
            .unwrap_or_else(Vec::new),
        retained_ref
            .map(|(_, reference)| {
                shas(&[
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
                ])
            })
            .unwrap_or_else(Vec::new),
    );
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
