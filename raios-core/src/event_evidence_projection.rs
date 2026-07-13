//! Typed evidence-v1 projection for captured current-boot event snapshots.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{evidence_value, observed, Evidence},
    module_load_gate_projection::{project_load_gate_denial, LoadGateProjectionInput},
    parse_sha256_ref,
    record::{Field, Value},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventClassification {
    Public,
    LocalOnly,
    Secret,
}

impl EventClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly => "local_only",
            Self::Secret => "secret",
        }
    }
}

/// The projection classes inventoried by the P4-4 manifest. `name` on
/// [`EventKind::Known`] retains the exact evaluator-owned event kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventProjectionClass {
    Unbound,
    HelloLifecycle,
    HelloInspectSource,
    CommandEnvelope,
    Provider,
    ModuleRetainedReference,
    AllocatorSourceEvidence,
    LoaderSourceEvidence,
    LoaderLiveBoundary,
    RecoveryArtifactLifeline,
    ModuleLoadGate,
    PromotionSignatureReference,
}

impl EventProjectionClass {
    const fn evidence_id(self) -> &'static str {
        match self {
            Self::Unbound => "event",
            Self::HelloLifecycle => "hello_lifecycle",
            Self::HelloInspectSource => "hello_inspect_source",
            Self::CommandEnvelope => "command_envelope",
            Self::Provider => "provider",
            Self::ModuleRetainedReference => "module_retained_reference",
            Self::AllocatorSourceEvidence => "allocator_source_evidence",
            Self::LoaderSourceEvidence => "loader_source_evidence",
            Self::LoaderLiveBoundary => "loader_live_boundary",
            Self::RecoveryArtifactLifeline => "recovery_artifact_lifeline",
            Self::ModuleLoadGate => "module_load_gate",
            Self::PromotionSignatureReference => "promotion_signature_reference",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind<'a> {
    Known {
        class: EventProjectionClass,
        name: &'a str,
    },
    Unknown {
        name: &'a str,
    },
}

impl<'a> EventKind<'a> {
    fn name(self) -> &'a str {
        match self {
            Self::Known { name, .. } | Self::Unknown { name } => name,
        }
    }

    fn evidence_id(self) -> &'static str {
        match self {
            Self::Known { class, .. } => class.evidence_id(),
            Self::Unknown { .. } => "unknown_event",
        }
    }

    fn renderable(self) -> bool {
        matches!(self, Self::Known { name, .. } if !name.is_empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalEventOutcome<'a> {
    Observed(&'a str),
    Denied(&'a str),
    Unknown(&'a str),
}

impl<'a> HistoricalEventOutcome<'a> {
    fn status_detail(self) -> &'a str {
        match self {
            Self::Observed(value) | Self::Denied(value) | Self::Unknown(value) => value,
        }
    }

    fn evidence_status(self) -> &'static str {
        match self {
            Self::Observed(_) => "verified",
            Self::Denied(_) | Self::Unknown(_) => "rejected",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifiedEventFacts<'a> {
    pub classification: EventClassification,
    pub fields: Vec<Field<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedEvent<'a> {
    pub sequence: u64,
    pub kind: EventKind<'a>,
    pub source_method: &'a str,
    pub source_transport: &'a str,
    pub classification: EventClassification,
    pub outcome: HistoricalEventOutcome<'a>,
    pub requested_capability: &'a str,
    pub risk: &'a str,
    pub subject: &'a str,
    pub resource: &'a str,
    pub reason: &'a str,
    pub source_labels: &'a [&'a str],
    pub binding: Option<ClassifiedEventFacts<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSnapshotInput<'a> {
    pub retention: &'a str,
    pub persistence: &'a str,
    pub provider_export: &'a str,
    pub bounded: bool,
    pub limit: u64,
    pub capacity: u64,
    pub total_count: u64,
    pub dropped_before_sequence: u64,
    pub events: Vec<CapturedEvent<'a>>,
}

pub struct EventEvidenceProjection<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    pub decision: Value<'static>,
}

/// Projects one immutable kernel-captured snapshot without sorting or deduping.
pub fn project_recent_events<'a>(input: EventSnapshotInput<'a>) -> EventEvidenceProjection<'a> {
    let returned_count = input.events.len() as u64;
    let evidence = input.events.into_iter().map(project_event).collect();
    let ring = Value::InlineObject(vec![
        Field::new("retention", Value::Str(input.retention)),
        Field::new("persistence", Value::Str(input.persistence)),
        Field::new("provider_export", Value::Str(input.provider_export)),
        Field::new("bounded", Value::Bool(input.bounded)),
        Field::new("limit", Value::U64(input.limit)),
        Field::new("capacity", Value::U64(input.capacity)),
        Field::new("total_count", Value::U64(input.total_count)),
        Field::new("returned_count", Value::U64(returned_count)),
        Field::new(
            "dropped_before_sequence",
            Value::U64(input.dropped_before_sequence),
        ),
    ]);

    EventEvidenceProjection {
        facts: Value::InlineObject(vec![Field::new("ring", ring)]),
        evidence,
        decision: observed("recent_events_read"),
    }
}

fn project_event<'a>(event: CapturedEvent<'a>) -> Evidence<'a> {
    let mut classification = event.classification;
    let classification_mismatch = event
        .binding
        .as_ref()
        .is_some_and(|binding| binding.classification > event.classification);
    if let Some(binding) = &event.binding {
        classification = classification.max(binding.classification);
        assert_fact_keys(&binding.fields);
    }

    let renderable = event.kind.renderable();
    let reason = if !renderable {
        "unknown_event_kind"
    } else if classification_mismatch {
        "classification_mismatch"
    } else {
        event.reason
    };
    let status = if renderable && !classification_mismatch {
        event.outcome.evidence_status()
    } else {
        "rejected"
    };
    let mut facts = vec![
        Field::new("sequence", Value::U64(event.sequence)),
        Field::new("kind", Value::Str(event.kind.name())),
        Field::new("source_method", Value::Str(event.source_method)),
        Field::new("source_transport", Value::Str(event.source_transport)),
        Field::new("status_detail", Value::Str(event.outcome.status_detail())),
        Field::new("status_reason", Value::Str(event.reason)),
        Field::new(
            "requested_capability",
            Value::Str(event.requested_capability),
        ),
        Field::new("risk", Value::Str(event.risk)),
        Field::new("subject", Value::Str(event.subject)),
        Field::new("resource", Value::Str(event.resource)),
        Field::new(
            "labels",
            Value::InlineArray(
                event
                    .source_labels
                    .iter()
                    .copied()
                    .map(Value::Str)
                    .collect(),
            ),
        ),
    ];
    if let Some(binding) = event.binding {
        facts.push(Field::new("binding", Value::InlineObject(binding.fields)));
    }
    assert_fact_keys(&facts);

    Evidence {
        id: event.kind.evidence_id(),
        kind: event.kind.name(),
        status,
        reason,
        source_event_sequence: Some(event.sequence),
        classification: classification.as_str(),
        facts: Value::InlineObject(facts),
    }
}

/// Converts the existing typed load-gate projection into event binding facts.
/// Historical blocker state stays evidence and cannot become a new decision.
pub fn project_load_gate_event_binding<'a>(
    input: LoadGateProjectionInput<'a>,
) -> ClassifiedEventFacts<'a> {
    let projection = project_load_gate_denial(input);
    ClassifiedEventFacts {
        classification: EventClassification::LocalOnly,
        fields: vec![
            Field::new("record_schema", Value::Str("raios.module_load_gate.v0")),
            Field::new(
                "evidence",
                Value::InlineArray(
                    projection
                        .evidence
                        .into_iter()
                        .map(evidence_value)
                        .collect(),
                ),
            ),
        ],
    }
}

/// Extracts a canonical serialized hash leaf; this never hashes event JSON.
pub fn extract_serialized_sha256(value: &str) -> Option<[u8; 32]> {
    parse_sha256_ref(value)
}

fn assert_fact_keys(fields: &[Field<'_>]) {
    for field in fields {
        debug_assert!(
            !matches!(field.key, "outcome" | "grants" | "effects" | "blocked_by")
                && !field.key.starts_with("authorizes_"),
            "decision key is reserved: {}",
            field.key
        );
        assert_value_keys(&field.value);
    }
}

fn assert_value_keys(value: &Value<'_>) {
    match value {
        Value::InlineObject(fields) | Value::Object(fields) => assert_fact_keys(fields),
        Value::InlineArray(values) | Value::Array(values) => {
            for value in values {
                assert_value_keys(value);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec};

    use super::*;
    use crate::{
        evidence_response::{response_value, Envelope},
        module_load_gate_projection::{
            LoadGateDisposition, LoadGateEvidenceStatus, LoadGatePrerequisite,
        },
        record::write_json,
    };

    const LABELS: &[&str] = &["second", "first"];
    const DIGEST: [u8; 32] = [0x5a; 32];

    fn event(
        sequence: u64,
        outcome: HistoricalEventOutcome<'static>,
        classification: EventClassification,
    ) -> CapturedEvent<'static> {
        CapturedEvent {
            sequence,
            kind: EventKind::Known {
                class: EventProjectionClass::Unbound,
                name: if sequence == 2 { "second" } else { "first" },
            },
            source_method: "memory.test",
            source_transport: "serial",
            classification,
            outcome,
            requested_capability: "none",
            risk: "read_only",
            subject: "agent.session.serial",
            resource: "event_log",
            reason: "recorded",
            source_labels: LABELS,
            binding: None,
        }
    }

    fn snapshot(events: Vec<CapturedEvent<'static>>) -> EventSnapshotInput<'static> {
        EventSnapshotInput {
            retention: "current_boot",
            persistence: "none",
            provider_export: "denied",
            bounded: true,
            limit: 32,
            capacity: 256,
            total_count: 7,
            dropped_before_sequence: 3,
            events,
        }
    }

    fn render(projection: EventEvidenceProjection<'static>) -> String {
        let envelope = Envelope {
            response_sequence: 1,
            family: "event",
            scope: "current_boot",
            classification: "local_only",
            source_method: "memory.recent_events",
            event_sequence: None,
            facts: projection.facts,
            evidence: projection
                .evidence
                .into_iter()
                .map(evidence_value)
                .collect(),
            decision: projection.decision,
        };
        let mut json = Vec::new();
        write_json(&response_value(&envelope), &mut json, 0);
        String::from_utf8(json).unwrap()
    }

    #[test]
    fn ordered_projection_preserves_ring_and_label_order() {
        let projection = project_recent_events(snapshot(vec![
            event(
                2,
                HistoricalEventOutcome::Observed("ok"),
                EventClassification::Public,
            ),
            event(
                1,
                HistoricalEventOutcome::Observed("ok"),
                EventClassification::Public,
            ),
        ]));
        assert_eq!(
            projection
                .evidence
                .iter()
                .map(|item| (item.source_event_sequence, item.kind))
                .collect::<Vec<_>>(),
            [(Some(2), "second"), (Some(1), "first")]
        );
        let Value::InlineObject(fields) = &projection.evidence[0].facts else {
            panic!("event facts must be inline")
        };
        assert_eq!(
            fields.last().unwrap().value,
            Value::InlineArray(vec![Value::Str("second"), Value::Str("first")])
        );
    }

    #[test]
    fn historical_denial_is_rejected_evidence_while_read_is_observed() {
        let mut denied_event = event(
            1,
            HistoricalEventOutcome::Denied("capability_denied"),
            EventClassification::LocalOnly,
        );
        denied_event.reason = "missing_evidence";
        let json = render(project_recent_events(snapshot(vec![denied_event])));
        assert!(json.contains("\"status\": \"rejected\", \"reason\": \"missing_evidence\""));
        assert!(json.contains("\"status_detail\": \"capability_denied\""));
        assert!(json.contains(
            "\"decision\": {\"outcome\": \"observed\", \"reason\": \"recent_events_read\"}"
        ));
    }

    #[test]
    fn unknown_kind_is_explicitly_rejected_not_omitted() {
        let mut unknown = event(
            4,
            HistoricalEventOutcome::Unknown("unrenderable"),
            EventClassification::LocalOnly,
        );
        unknown.kind = EventKind::Unknown {
            name: "future.event",
        };
        let projection = project_recent_events(snapshot(vec![unknown]));
        assert_eq!(projection.evidence.len(), 1);
        assert_eq!(projection.evidence[0].id, "unknown_event");
        assert_eq!(projection.evidence[0].kind, "future.event");
        assert_eq!(projection.evidence[0].status, "rejected");
        assert_eq!(projection.evidence[0].reason, "unknown_event_kind");
    }

    #[test]
    fn known_empty_kind_is_unrenderable_and_rejected() {
        let mut unrenderable = event(
            5,
            HistoricalEventOutcome::Observed("recorded"),
            EventClassification::Public,
        );
        unrenderable.kind = EventKind::Known {
            class: EventProjectionClass::Unbound,
            name: "",
        };
        assert_eq!(
            project_recent_events(snapshot(vec![unrenderable])).evidence[0].status,
            "rejected"
        );
    }

    #[test]
    fn classification_is_preserved_per_record() {
        let projection = project_recent_events(snapshot(vec![
            event(
                1,
                HistoricalEventOutcome::Observed("ok"),
                EventClassification::Public,
            ),
            event(
                2,
                HistoricalEventOutcome::Observed("ok"),
                EventClassification::Secret,
            ),
        ]));
        assert_eq!(projection.evidence[0].classification, "public");
        assert_eq!(projection.evidence[1].classification, "secret");
    }

    #[test]
    fn secret_binding_cannot_render_as_public_evidence() {
        let mut public = event(
            1,
            HistoricalEventOutcome::Observed("ok"),
            EventClassification::Public,
        );
        public.binding = Some(ClassifiedEventFacts {
            classification: EventClassification::Secret,
            fields: vec![Field::new("secret_token", Value::Str("redacted"))],
        });
        let record = project_recent_events(snapshot(vec![public]))
            .evidence
            .pop()
            .unwrap();
        assert_eq!(record.classification, "secret");
        assert_eq!(record.status, "rejected");
        assert_eq!(record.reason, "classification_mismatch");
    }

    #[test]
    fn sha256_fact_passes_through_byte_identically() {
        let mut with_hash = event(
            1,
            HistoricalEventOutcome::Observed("ok"),
            EventClassification::LocalOnly,
        );
        with_hash.binding = Some(ClassifiedEventFacts {
            classification: EventClassification::LocalOnly,
            fields: vec![Field::new("artifact_hash", Value::Sha256(DIGEST))],
        });
        let json = render(project_recent_events(snapshot(vec![with_hash])));
        assert!(json.contains(
            "\"artifact_hash\": \"sha256:5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\""
        ));
    }

    #[test]
    fn serialized_hash_extractor_consumes_sha256_prefix() {
        let hash = "sha256:5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
        assert_eq!(extract_serialized_sha256(hash), Some(DIGEST));
    }

    #[test]
    fn observational_response_has_null_event_and_no_grants_or_effects() {
        let json = render(project_recent_events(snapshot(vec![])));
        assert!(json.contains("\"event_id\": null"));
        assert!(!json.contains("\"grants\""));
        assert!(!json.contains("\"effects\""));
    }

    #[test]
    fn ring_facts_use_manifest_names_and_derive_returned_count() {
        let projection = project_recent_events(snapshot(vec![event(
            1,
            HistoricalEventOutcome::Observed("ok"),
            EventClassification::Public,
        )]));
        assert_eq!(
            projection.facts,
            Value::InlineObject(vec![Field::new(
                "ring",
                Value::InlineObject(vec![
                    Field::new("retention", Value::Str("current_boot")),
                    Field::new("persistence", Value::Str("none")),
                    Field::new("provider_export", Value::Str("denied")),
                    Field::new("bounded", Value::Bool(true)),
                    Field::new("limit", Value::U64(32)),
                    Field::new("capacity", Value::U64(256)),
                    Field::new("total_count", Value::U64(7)),
                    Field::new("returned_count", Value::U64(1)),
                    Field::new("dropped_before_sequence", Value::U64(3)),
                ])
            )])
        );
    }

    fn prerequisite(index: usize) -> LoadGatePrerequisite<'static> {
        LoadGatePrerequisite {
            status: LoadGateEvidenceStatus::Missing,
            status_detail: "missing",
            reason: if index == 0 {
                "manifest_missing"
            } else {
                "missing"
            },
            source_event_sequence: None,
            facts: vec![Field::new("index", Value::U64(index as u64))],
            disposition: LoadGateDisposition::Blocked,
        }
    }

    #[test]
    fn load_gate_binding_reuses_core_evidence_without_a_nested_decision() {
        let mut items = (0..12).map(prerequisite);
        let binding = project_load_gate_event_binding(LoadGateProjectionInput {
            module_manifest: items.next().unwrap(),
            candidate_artifact: items.next().unwrap(),
            vm_test_report: items.next().unwrap(),
            local_attestation: items.next().unwrap(),
            local_approval: items.next().unwrap(),
            computed_capability_grant: items.next().unwrap(),
            durable_audit_record: items.next().unwrap(),
            rollback_plan: items.next().unwrap(),
            service_slot: items.next().unwrap(),
            service_slot_allocator: items.next().unwrap(),
            loader_runtime: items.next().unwrap(),
            loader: items.next().unwrap(),
        });
        let Value::InlineArray(evidence) = &binding.fields[1].value else {
            panic!("load-gate evidence must be inline")
        };
        assert_eq!(evidence.len(), 12);
        assert_eq!(
            evidence[0],
            evidence_value(Evidence {
                id: "module_manifest",
                kind: "retained_reference",
                status: "missing",
                reason: "manifest_missing",
                source_event_sequence: None,
                classification: "local_only",
                facts: Value::InlineObject(vec![
                    Field::new("status_detail", Value::Str("missing")),
                    Field::new("index", Value::U64(0)),
                ]),
            })
        );
        assert!(!binding.fields.iter().any(|field| field.key == "decision"));
    }

    #[test]
    fn promotion_signature_class_is_never_silently_retired() {
        let mut signature = event(
            8,
            HistoricalEventOutcome::Observed("retained"),
            EventClassification::LocalOnly,
        );
        signature.kind = EventKind::Known {
            class: EventProjectionClass::PromotionSignatureReference,
            name: "module.promotion_signature_reference.retained",
        };
        let projection = project_recent_events(snapshot(vec![signature]));
        assert_eq!(projection.evidence.len(), 1);
        assert_eq!(projection.evidence[0].id, "promotion_signature_reference");
    }

    #[test]
    #[should_panic(expected = "decision key is reserved: authorizes_load")]
    fn binding_facts_reject_reserved_decision_keys() {
        let mut bad = event(
            1,
            HistoricalEventOutcome::Observed("ok"),
            EventClassification::LocalOnly,
        );
        bad.binding = Some(ClassifiedEventFacts {
            classification: EventClassification::LocalOnly,
            fields: vec![Field::new("authorizes_load", Value::Bool(false))],
        });
        project_recent_events(snapshot(vec![bad]));
    }
}
