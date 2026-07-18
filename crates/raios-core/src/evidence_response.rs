//! Evidence-vocabulary-v1 response construction shared by kernel families.

use alloc::{vec, vec::Vec};

use crate::{
    field,
    record::{Field, Value},
    record_table::{
        decision_value, object_from, object_from_owned, Cell, DenialDecision, DeniedBy,
        EvaluatedDecision, FieldSpec, GrantDecision, GrantProof, Observation, OwnedFieldSpec,
    },
};

pub const SCHEMA: &str = "raios.evidence_response.v1";

pub fn reference_status(valid: bool, has_reference: bool) -> &'static str {
    if valid {
        "verified"
    } else if has_reference {
        "rejected"
    } else {
        "missing"
    }
}

pub struct Envelope<'a> {
    pub response_sequence: u64,
    pub family: &'a str,
    pub scope: &'a str,
    pub classification: &'a str,
    pub source_method: &'a str,
    pub event_sequence: Option<u64>,
    pub facts: Value<'a>,
    pub evidence: Vec<Value<'a>>,
    pub decision: Value<'a>,
}

impl Envelope<'_> {
    fn schema(&self) -> &str {
        SCHEMA
    }
    fn response_sequence(&self) -> u64 {
        self.response_sequence
    }
    fn family(&self) -> &str {
        self.family
    }
    fn scope(&self) -> &str {
        self.scope
    }
    fn classification(&self) -> &str {
        self.classification
    }
    fn source_method(&self) -> &str {
        self.source_method
    }
    fn event_id(&self) -> Value<'static> {
        self.event_sequence
            .map(Value::EventSequence)
            .unwrap_or(Value::Null)
    }
    fn facts(&self) -> Value<'_> {
        self.facts.clone()
    }
    fn evidence(&self) -> Value<'_> {
        Value::Array(self.evidence.clone())
    }
    fn decision(&self) -> Value<'_> {
        self.decision.clone()
    }
}

fn envelope_fields<'a>() -> [FieldSpec<Envelope<'a>>; 10] {
    [
        field!("schema", str <- Envelope::schema),
        field!("id", response_sequence <- Envelope::response_sequence),
        field!("family", str <- Envelope::family),
        field!("scope", str <- Envelope::scope),
        field!("classification", str <- Envelope::classification),
        field!("source_method", str <- Envelope::source_method),
        field!("event_id", value <- Envelope::event_id),
        field!("facts", value <- Envelope::facts),
        field!("evidence", value <- Envelope::evidence),
        field!("decision", value <- Envelope::decision),
    ]
}

pub fn response_value<'a>(envelope: &'a Envelope<'a>) -> Value<'a> {
    match object_from(&envelope_fields(), envelope) {
        Value::InlineObject(fields) => Value::Object(fields),
        _ => unreachable!(),
    }
}

pub struct DiagnosticFacts<'a> {
    pub test_infrastructure: bool,
    pub reference_format: &'a str,
    pub request: Value<'a>,
    pub intake: Value<'a>,
    pub guest_evidence_authority: &'a str,
    pub required_before_load: Value<'a>,
    pub trust_tier: Value<'a>,
    pub runtime: Value<'a>,
}

pub fn diagnostic_facts_value<'a>(facts: DiagnosticFacts<'a>) -> Value<'a> {
    let fields = [
        OwnedFieldSpec {
            key: "test_infrastructure",
            get: |value: &DiagnosticFacts<'a>| Cell::Bool(value.test_infrastructure),
        },
        OwnedFieldSpec {
            key: "reference_format",
            get: |value: &DiagnosticFacts<'a>| Cell::Str(value.reference_format),
        },
        OwnedFieldSpec {
            key: "request",
            get: |value: &DiagnosticFacts<'a>| Cell::Value(value.request.clone()),
        },
        OwnedFieldSpec {
            key: "intake",
            get: |value: &DiagnosticFacts<'a>| Cell::Value(value.intake.clone()),
        },
        OwnedFieldSpec {
            key: "guest_evidence_authority",
            get: |value: &DiagnosticFacts<'a>| Cell::Str(value.guest_evidence_authority),
        },
        OwnedFieldSpec {
            key: "required_before_load",
            get: |value: &DiagnosticFacts<'a>| Cell::Value(value.required_before_load.clone()),
        },
        OwnedFieldSpec {
            key: "trust_tier",
            get: |value: &DiagnosticFacts<'a>| Cell::Value(value.trust_tier.clone()),
        },
        OwnedFieldSpec {
            key: "runtime",
            get: |value: &DiagnosticFacts<'a>| Cell::Value(value.runtime.clone()),
        },
    ];
    object_from_owned(&fields, &facts)
}

pub struct SelftestFacts<'a> {
    pub case_count: u64,
    pub passed: bool,
    pub safety: Value<'a>,
    pub cases: Value<'a>,
}

pub fn selftest_facts_value<'a>(facts: SelftestFacts<'a>) -> Value<'a> {
    let fields = [
        OwnedFieldSpec {
            key: "test_infrastructure",
            get: |_value: &SelftestFacts<'a>| Cell::Bool(true),
        },
        OwnedFieldSpec {
            key: "case_count",
            get: |value: &SelftestFacts<'a>| Cell::U64(value.case_count),
        },
        OwnedFieldSpec {
            key: "passed",
            get: |value: &SelftestFacts<'a>| Cell::Bool(value.passed),
        },
        OwnedFieldSpec {
            key: "safety",
            get: |value: &SelftestFacts<'a>| Cell::Value(value.safety.clone()),
        },
        OwnedFieldSpec {
            key: "cases",
            get: |value: &SelftestFacts<'a>| Cell::Value(value.cases.clone()),
        },
    ];
    object_from_owned(&fields, &facts)
}

pub fn selftest_safety_value() -> Value<'static> {
    struct Safety;
    let fields = [
        OwnedFieldSpec {
            key: "event_log_write_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "external_artifact_intake_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "artifact_load_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "load_authorization_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "load_attempt_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "retained_record_create_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "durable_audit_record_create_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "rollback_plan_create_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "service_slot_allocation_count",
            get: |_value: &Safety| Cell::U64(0),
        },
        OwnedFieldSpec {
            key: "service_inventory_change",
            get: |_value: &Safety| Cell::Str("none"),
        },
    ];
    object_from_owned(&fields, &Safety)
}

pub struct Evidence<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub status: &'a str,
    pub reason: &'a str,
    pub source_event_sequence: Option<u64>,
    pub classification: &'a str,
    pub facts: Value<'a>,
}

impl<'a> Evidence<'a> {
    fn id(&self) -> &'a str {
        self.id
    }
    fn kind(&self) -> &'a str {
        self.kind
    }
    fn status(&self) -> &'a str {
        self.status
    }
    fn reason(&self) -> &'a str {
        self.reason
    }
    fn source_event_id(&self) -> Value<'static> {
        self.source_event_sequence
            .map(Value::EventSequence)
            .unwrap_or(Value::Null)
    }
    fn classification(&self) -> &'a str {
        self.classification
    }
    fn facts(&self) -> Value<'a> {
        self.facts.clone()
    }
}

fn evidence_fields<'a>() -> [OwnedFieldSpec<'a, Evidence<'a>>; 7] {
    [
        OwnedFieldSpec {
            key: "id",
            get: |value| Cell::Str(value.id()),
        },
        OwnedFieldSpec {
            key: "kind",
            get: |value| Cell::Str(value.kind()),
        },
        OwnedFieldSpec {
            key: "status",
            get: |value| Cell::Str(value.status()),
        },
        OwnedFieldSpec {
            key: "reason",
            get: |value| Cell::Str(value.reason()),
        },
        OwnedFieldSpec {
            key: "source_event_id",
            get: |value| Cell::Value(value.source_event_id()),
        },
        OwnedFieldSpec {
            key: "classification",
            get: |value| Cell::Str(value.classification()),
        },
        OwnedFieldSpec {
            key: "facts",
            get: |value| Cell::Value(value.facts()),
        },
    ]
}

pub fn evidence_value<'a>(evidence: Evidence<'a>) -> Value<'a> {
    object_from_owned(&evidence_fields(), &evidence)
}

pub fn observed(reason: &str) -> Value<'_> {
    let observation = Observation::new(reason);
    decision_value(&EvaluatedDecision::Observed(observation))
}

pub struct Blocked<'a> {
    pub evidence_id: &'a str,
    pub status: &'a str,
    pub reason: &'a str,
}

#[derive(Clone, Copy)]
pub enum ModuleReferenceFamily {
    Manifest,
    Grant,
    Artifact,
    VmReport,
    Attestation,
    Approval,
    AuditRollback,
    ServiceSlot,
}

pub fn module_reference_denial<'a>(
    family: ModuleReferenceFamily,
    primary: Option<Blocked<'a>>,
) -> Value<'a> {
    let mut blocked = Vec::new();
    if let Some(primary) = primary {
        blocked.push(primary);
    }
    let required: &[Blocked<'static>] = match family {
        ModuleReferenceFamily::Manifest => &[
            Blocked {
                evidence_id: "candidate_artifact",
                status: "missing",
                reason: "candidate_artifact_missing",
            },
            Blocked {
                evidence_id: "vm_test_report",
                status: "missing",
                reason: "vm_test_report_missing",
            },
            Blocked {
                evidence_id: "local_attestation",
                status: "missing",
                reason: "local_attestation_missing",
            },
            Blocked {
                evidence_id: "computed_capability_grant",
                status: "missing",
                reason: "computed_capability_grant_missing",
            },
            Blocked {
                evidence_id: "durable_audit_record",
                status: "missing",
                reason: "durable_audit_write_missing",
            },
            Blocked {
                evidence_id: "rollback_plan",
                status: "missing",
                reason: "rollback_install_missing",
            },
            Blocked {
                evidence_id: "loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
        ],
        ModuleReferenceFamily::Artifact => &[
            Blocked {
                evidence_id: "vm_test_report",
                status: "missing",
                reason: "vm_test_report_missing",
            },
            Blocked {
                evidence_id: "local_attestation",
                status: "missing",
                reason: "local_attestation_missing",
            },
            Blocked {
                evidence_id: "durable_audit_record",
                status: "missing",
                reason: "durable_audit_write_missing",
            },
            Blocked {
                evidence_id: "rollback_plan",
                status: "missing",
                reason: "rollback_install_missing",
            },
            Blocked {
                evidence_id: "loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
        ],
        ModuleReferenceFamily::VmReport => &[
            Blocked {
                evidence_id: "local_attestation",
                status: "missing",
                reason: "local_attestation_missing",
            },
            Blocked {
                evidence_id: "durable_audit_record",
                status: "missing",
                reason: "durable_audit_write_missing",
            },
            Blocked {
                evidence_id: "rollback_plan",
                status: "missing",
                reason: "rollback_install_missing",
            },
            Blocked {
                evidence_id: "loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
        ],
        ModuleReferenceFamily::Grant
        | ModuleReferenceFamily::Attestation
        | ModuleReferenceFamily::Approval => &[
            Blocked {
                evidence_id: "durable_audit_record",
                status: "missing",
                reason: "durable_audit_write_missing",
            },
            Blocked {
                evidence_id: "rollback_plan",
                status: "missing",
                reason: "rollback_install_missing",
            },
            Blocked {
                evidence_id: "loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
            Blocked {
                evidence_id: "service_slot",
                status: "unallocated",
                reason: "ram_only_service_slot_unallocated",
            },
        ],
        ModuleReferenceFamily::AuditRollback => &[
            Blocked {
                evidence_id: "loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
            Blocked {
                evidence_id: "service_slot",
                status: "unallocated",
                reason: "ram_only_service_slot_unallocated",
            },
        ],
        ModuleReferenceFamily::ServiceSlot => &[
            Blocked {
                evidence_id: "service_slot_allocator",
                status: "unavailable",
                reason: "ram_only_service_slot_allocator_unimplemented",
            },
            Blocked {
                evidence_id: "module_loader",
                status: "unavailable",
                reason: "module_loader_unimplemented",
            },
        ],
    };
    blocked.extend(required.iter().map(|item| Blocked {
        evidence_id: item.evidence_id,
        status: item.status,
        reason: item.reason,
    }));
    let reason = blocked
        .first()
        .map(|item| item.reason)
        .unwrap_or("capability_evidence_missing");
    denied(reason, "cap.module.load_ephemeral", &blocked)
}

pub fn denied<'a>(
    reason: &'a str,
    requested_capability: &'a str,
    blocked: &[Blocked<'a>],
) -> Value<'a> {
    let denied_by: Vec<_> = blocked
        .iter()
        .map(|item| DeniedBy::new(item.evidence_id, item.status, item.reason))
        .collect();
    let denial = DenialDecision::new(reason, requested_capability, denied_by);
    decision_value(&EvaluatedDecision::Denied(denial))
}

/// The only public positive constructor: callers provide evaluator facts, and
/// raios-core creates the proof only when every positive predicate is true.
pub fn module_grant_decision<'a>(
    reference_valid: bool,
    signature_verified: bool,
    hashes_match: bool,
    candidate_bytes_present: bool,
    slot_allocatable: bool,
    loader_available: bool,
    denied_value: Value<'a>,
) -> Value<'a> {
    if !(reference_valid
        && signature_verified
        && hashes_match
        && candidate_bytes_present
        && slot_allocatable
        && loader_available)
    {
        return denied_value;
    }
    const GRANTS: &[&str] = &["cap.module.load_ephemeral"];
    const EFFECTS: &[&str] = &["guest_load_authorized"];
    let grant = GrantDecision::new(
        GrantProof::new(),
        "dev_key_attestation_and_runtime_ready",
        "cap.module.load_ephemeral",
        GRANTS,
        EFFECTS,
    );
    decision_value(&EvaluatedDecision::Granted(grant))
}

pub fn selftest_case<'a>(
    name: &'a str,
    expected_status: &'a str,
    expected_reason: &'a str,
    actual_status: &'a str,
    actual_reason: &'a str,
    passed: bool,
) -> Value<'a> {
    Value::InlineObject(vec![
        Field::new("case", Value::Str(name)),
        Field::new(
            "expected",
            Value::InlineObject(vec![
                Field::new("status", Value::Str(expected_status)),
                Field::new("reason", Value::Str(expected_reason)),
            ]),
        ),
        Field::new(
            "actual",
            Value::InlineObject(vec![
                Field::new("status", Value::Str(actual_status)),
                Field::new("reason", Value::Str(actual_reason)),
            ]),
        ),
        Field::new("passed", Value::Bool(passed)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::write_json;

    fn render(value: &Value<'_>) -> std::vec::Vec<u8> {
        let mut out = std::vec::Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    #[test]
    fn envelope_has_v1_order_and_typed_ids() {
        let envelope = Envelope {
            response_sequence: 7,
            family: "module.manifest_reference",
            scope: "current_boot",
            classification: "local_only",
            source_method: "module.manifest_diagnostic",
            event_sequence: Some(3),
            facts: Value::InlineObject(vec![]),
            evidence: vec![],
            decision: observed("selftest_completed"),
        };
        let value = response_value(&envelope);
        let json = std::str::from_utf8(&render(&value)).unwrap().to_owned();
        assert!(json.starts_with("{\r\n  \"schema\": \"raios.evidence_response.v1\",\r\n  \"id\": \"response.current_boot.00000007\""));
        assert!(json.contains("\"event_id\": \"event.current_boot.00000003\""));
    }

    #[test]
    fn denial_is_absolute_and_preserves_evaluator_order() {
        let value = denied(
            "first_missing",
            "cap.module.load_ephemeral",
            &[
                Blocked {
                    evidence_id: "first",
                    status: "missing",
                    reason: "first_missing",
                },
                Blocked {
                    evidence_id: "second",
                    status: "rejected",
                    reason: "second_rejected",
                },
            ],
        );
        let json = std::str::from_utf8(&render(&value)).unwrap().to_owned();
        assert!(json.contains("\"grants\": [], \"effects\": []"));
        assert!(json.find("\"first\"").unwrap() < json.find("\"second\"").unwrap());
    }

    #[test]
    fn module_family_denial_order_is_evaluator_owned() {
        let value = module_reference_denial(
            ModuleReferenceFamily::Manifest,
            Some(Blocked {
                evidence_id: "module_manifest",
                status: "rejected",
                reason: "substituted_manifest",
            }),
        );
        let json = std::str::from_utf8(&render(&value)).unwrap().to_owned();
        let ordered = [
            "module_manifest",
            "candidate_artifact",
            "vm_test_report",
            "local_attestation",
            "computed_capability_grant",
            "durable_audit_record",
            "rollback_plan",
            "loader",
        ];
        let mut prior = 0;
        for evidence_id in ordered {
            let offset = json[prior..].find(evidence_id).unwrap() + prior;
            assert!(offset >= prior);
            prior = offset + evidence_id.len();
        }
        assert!(json.contains("\"grants\": [], \"effects\": []"));
    }

    #[test]
    fn grant_requires_all_evaluator_facts() {
        let denial = denied("loader_missing", "cap.module.load_ephemeral", &[]);
        for missing in 0..6 {
            let mut facts = [true; 6];
            facts[missing] = false;
            let value = module_grant_decision(
                facts[0],
                facts[1],
                facts[2],
                facts[3],
                facts[4],
                facts[5],
                denial.clone(),
            );
            assert!(std::str::from_utf8(&render(&value))
                .unwrap()
                .contains("\"outcome\": \"denied\""));
        }
        let granted = module_grant_decision(true, true, true, true, true, true, denial);
        assert!(std::str::from_utf8(&render(&granted))
            .unwrap()
            .contains("\"outcome\": \"granted\""));
    }

    #[test]
    fn selftest_shape_exposes_typed_zero_effect_counts() {
        let value = selftest_case("present", "verified", "ok", "verified", "ok", true);
        let json = std::str::from_utf8(&render(&value)).unwrap().to_owned();
        assert!(json.contains("\"expected\": {\"status\": \"verified\", \"reason\": \"ok\"}"));
        assert!(!json.contains("can_load"));
        assert!(!json.contains("load_attempted"));

        let safety = std::str::from_utf8(&render(&selftest_safety_value()))
            .unwrap()
            .to_owned();
        assert!(safety.contains("\"event_log_write_count\": 0"));
        assert!(safety.contains("\"load_authorization_count\": 0"));
        assert!(safety.contains("\"service_inventory_change\": \"none\""));
    }

    #[test]
    fn every_module_reference_family_maps_every_legacy_semantic_group() {
        #[derive(Clone, Copy)]
        #[allow(dead_code)] // The manifest currently retires leaf fields, not a whole semantic group.
        enum Carrier {
            V1Needle(&'static str),
            ConstantInvariant(&'static str),
            Retired(&'static str),
        }

        const MANIFEST_GROUP_COUNT: usize = 20;
        const MAPPING: &[(&str, Carrier)] = &[
            (
                "schema",
                Carrier::ConstantInvariant(
                    "all module-reference responses use raios.evidence_response.v1",
                ),
            ),
            (
                "scope",
                Carrier::V1Needle(
                    "\"family\": \"module.manifest_reference\",\r\n  \"scope\": \"current_boot\"",
                ),
            ),
            (
                "classification",
                Carrier::V1Needle(
                    "\"scope\": \"current_boot\",\r\n  \"classification\": \"local_only\"",
                ),
            ),
            (
                "test_infrastructure",
                Carrier::V1Needle("\"test_infrastructure\": false"),
            ),
            (
                "intake",
                Carrier::V1Needle("\"intake\": {\"accepts_manifest_json\": false"),
            ),
            (
                "reference_format",
                Carrier::V1Needle("\"reference_format\": \"module.manifest_diagnostic"),
            ),
            (
                "request",
                Carrier::V1Needle(
                    "\"request\": {\"requested_capability\": \"cap.module.load_ephemeral\"",
                ),
            ),
            (
                "reference_state",
                Carrier::V1Needle(
                    "\"facts\": {\"state\": \"present\", \"status_detail\": \"hash_reference_valid_load_still_denied\"",
                ),
            ),
            (
                "validation_status",
                Carrier::V1Needle(
                    "\"status_detail\": \"hash_reference_valid_load_still_denied\"",
                ),
            ),
            (
                "validation_reason",
                Carrier::V1Needle(
                    "\"reason\": \"hash_reference_valid_load_still_denied\"",
                ),
            ),
            ("arity_valid", Carrier::V1Needle("\"arity_valid\": true")),
            (
                "reference_hashes",
                Carrier::V1Needle("\"expected_manifest_reference_hash\": \"sha256:5a5a"),
            ),
            (
                "retained_state",
                Carrier::V1Needle(
                    "\"facts\": {\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\"",
                ),
            ),
            (
                "retained_provenance",
                Carrier::V1Needle(
                    "\"source_event_id\": \"event.current_boot.00000003\"",
                ),
            ),
            (
                "retained_match",
                Carrier::V1Needle("\"matches_current_reference\": false"),
            ),
            (
                "retained_hashes",
                Carrier::V1Needle("\"manifest_reference_hash\": \"sha256:6b6b"),
            ),
            (
                "gate_state",
                Carrier::V1Needle(
                    "\"status\": \"verified\", \"reason\": \"hash_reference_valid_load_still_denied\"",
                ),
            ),
            (
                "policy_result",
                Carrier::V1Needle(
                    "\"guest_evidence_authority\": \"hash_reference_only_no_manifest_json_or_artifact_bytes\"",
                ),
            ),
            ("blocked_by", Carrier::V1Needle("\"blocked_by\": [")),
            (
                "authority_effects",
                Carrier::V1Needle(
                    "\"outcome\": \"denied\", \"reason\": \"candidate_artifact_missing\", \"requested_capability\": \"cap.module.load_ephemeral\", \"grants\": [], \"effects\": []",
                ),
            ),
        ];

        let facts = diagnostic_facts_value(DiagnosticFacts {
            test_infrastructure: false,
            reference_format:
                "module.manifest_diagnostic <manifest_reference_hash> <manifest_hash> [current_boot]",
            request: Value::InlineObject(vec![
                Field::new(
                    "requested_capability",
                    Value::Str("cap.module.load_ephemeral"),
                ),
                Field::new("load_mode", Value::Str("ram_only")),
                Field::new("subject", Value::Str("agent.session.serial")),
                Field::new("resource", Value::Str("live_service_graph")),
            ]),
            intake: Value::InlineObject(vec![
                Field::new("accepts_manifest_json", Value::Bool(false)),
                Field::new("accepts_artifact_bytes", Value::Bool(false)),
                Field::new("accepts_unsigned_service_code", Value::Bool(false)),
            ]),
            guest_evidence_authority: "hash_reference_only_no_manifest_json_or_artifact_bytes",
            required_before_load: Value::InlineArray(vec![Value::Str("candidate_artifact_sha256")]),
            trust_tier: Value::Null,
            runtime: Value::InlineObject(vec![
                Field::new("loader", Value::Str("unavailable")),
                Field::new("service_slot", Value::Str("unallocated")),
            ]),
        });
        let evidence = vec![
            evidence_value(Evidence {
                id: "module_manifest",
                kind: "reference",
                status: "verified",
                reason: "hash_reference_valid_load_still_denied",
                source_event_sequence: None,
                classification: "local_only",
                facts: Value::InlineObject(vec![
                    Field::new("state", Value::Str("present")),
                    Field::new(
                        "status_detail",
                        Value::Str("hash_reference_valid_load_still_denied"),
                    ),
                    Field::new("arity_valid", Value::Bool(true)),
                    Field::new("scope", Value::Str("current_boot")),
                    Field::new("manifest_schema", Value::Str("raios.module_manifest.v0")),
                    Field::new("manifest_reference_hash", Value::Sha256([0x5a; 32])),
                    Field::new(
                        "expected_manifest_reference_hash",
                        Value::Sha256([0x5a; 32]),
                    ),
                    Field::new("manifest_hash", Value::Sha256([0x5a; 32])),
                ]),
            }),
            evidence_value(Evidence {
                id: "module_manifest_retained",
                kind: "retained_reference",
                status: "verified",
                reason: "retained_hash_reference_load_still_denied",
                source_event_sequence: Some(3),
                classification: "local_only",
                facts: Value::InlineObject(vec![
                    Field::new("state", Value::Str("present")),
                    Field::new("retention", Value::Str("current_boot_ram_event_log")),
                    Field::new("matches_current_reference", Value::Bool(false)),
                    Field::new(
                        "record_schema",
                        Value::Str("raios.module_manifest_reference.v0"),
                    ),
                    Field::new(
                        "status_detail",
                        Value::Str("retained_hash_reference_load_still_denied"),
                    ),
                    Field::new("manifest_reference_hash", Value::Sha256([0x6b; 32])),
                    Field::new("manifest_hash", Value::Sha256([0x6b; 32])),
                ]),
            }),
        ];
        let envelope = Envelope {
            response_sequence: 7,
            family: "module.manifest_reference",
            scope: "current_boot",
            classification: "local_only",
            source_method: "module.manifest_diagnostic",
            event_sequence: Some(3),
            facts,
            evidence,
            decision: module_reference_denial(ModuleReferenceFamily::Manifest, None),
        };
        let value = response_value(&envelope);
        let json = std::str::from_utf8(&render(&value)).unwrap().to_owned();

        assert_eq!(MAPPING.len(), MANIFEST_GROUP_COUNT);
        for &(group, carrier) in MAPPING {
            match carrier {
                Carrier::V1Needle(needle) => assert!(
                    json.contains(needle),
                    "legacy semantic group {group} missing v1 carrier {needle:?}"
                ),
                Carrier::ConstantInvariant(reason) | Carrier::Retired(reason) => assert!(
                    !reason.is_empty(),
                    "legacy semantic group {group} has no mapping reason"
                ),
            }
        }
    }

    #[test]
    fn present_missing_rejected_and_substituted_statuses_are_explicit() {
        assert_eq!(reference_status(true, true), "verified");
        assert_eq!(reference_status(false, false), "missing");
        assert_eq!(reference_status(false, true), "rejected");
        assert_eq!(
            reference_status(false, true),
            "rejected",
            "substituted reference"
        );
    }

    #[test]
    fn module_authority_hash_grammar_is_byte_identical() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../seed-kernel/src/module_evidence.rs");
        let bytes = std::fs::read(path).unwrap();
        let normalized = String::from_utf8(bytes).unwrap().replace("\r\n", "\n");
        let digest = crate::sha256_bytes(normalized.as_bytes());
        assert_eq!(
            &crate::sha256_hex(&digest),
            b"ee2cf2323b685f77be9e76685cfb19ae024cedbf4584e251a05ecd807575ad15"
        );
    }
}
