//! Typed evidence-v1 projection for one captured Hello lifecycle snapshot.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{observed, Evidence},
    record::{Field, Value},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleAction {
    Load,
    Start,
    Restart,
    HotSwap,
    Stop,
    Drop,
    Health,
}

impl LifecycleAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Start => "start",
            Self::Restart => "restart",
            Self::HotSwap => "hot_swap",
            Self::Stop => "stop",
            Self::Drop => "drop",
            Self::Health => "health",
        }
    }

    const fn observation_reason(self) -> &'static str {
        match self {
            Self::Load => "load_performed",
            Self::Start => "start_performed",
            Self::Restart => "restart_performed",
            Self::HotSwap => "hot_swap_performed",
            Self::Stop => "stop_performed",
            Self::Drop => "drop_performed",
            Self::Health => "health_performed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceClassification {
    Public,
    LocalOnly,
    Secret,
    Unknown,
}

impl EvidenceClassification {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly | Self::Unknown => "local_only",
            Self::Secret => "secret",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceStatus {
    Verified,
    Rejected,
    Missing,
    NotApplicable,
}

impl EvidenceStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Rejected => "rejected",
            Self::Missing => "missing",
            Self::NotApplicable => "not_applicable",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceMeta<'a> {
    pub status: EvidenceStatus,
    pub reason: &'a str,
    pub classification: EvidenceClassification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptorSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub descriptor_id: &'a str,
    pub source_locator: &'a str,
    pub source_kind: &'a str,
    pub descriptor_hash: [u8; 32],
    pub bound_source_hash: [u8; 32],
    pub signature_hash: [u8; 32],
    pub attestation_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub artifact_id: &'a str,
    pub artifact_hash: [u8; 32],
    pub content_binding_hash: [u8; 32],
    pub reference_hash: [u8; 32],
    pub bytes_hash: [u8; 32],
    pub signature_hash: [u8; 32],
    pub preflight_status: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceSlotSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub slot_id: &'a str,
    pub before_status: &'a str,
    pub after_status: &'a str,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventorySnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub change: &'a str,
    pub present_before: bool,
    pub present_after: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceStateSnapshot<'a> {
    pub version: &'a str,
    pub generation: u64,
    pub loaded: bool,
    pub running: bool,
    pub state_counter: u64,
    pub state_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateTransitionSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub before: ServiceStateSnapshot<'a>,
    pub after: ServiceStateSnapshot<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateMigrationSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub from_version: &'a str,
    pub to_version: &'a str,
    pub state_preserved: bool,
    pub migration_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthSnapshot<'a> {
    pub meta: EvidenceMeta<'a>,
    pub status_detail: &'a str,
    pub loaded: bool,
    pub running: bool,
}

/// All fields are captured under the caller's one lifecycle-state read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelloLifecycleSnapshot<'a> {
    pub action: LifecycleAction,
    pub source_event_sequence: u64,
    pub service_id: &'a str,
    pub descriptor: DescriptorSnapshot<'a>,
    pub artifact: ArtifactSnapshot<'a>,
    pub service_slot: ServiceSlotSnapshot<'a>,
    pub inventory: InventorySnapshot<'a>,
    pub state_transition: StateTransitionSnapshot<'a>,
    pub state_migration: Option<StateMigrationSnapshot<'a>>,
    pub health: Option<HealthSnapshot<'a>>,
}

pub struct HelloLifecycleProjection<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    action: LifecycleAction,
}

impl HelloLifecycleProjection<'_> {
    /// Lifecycle authority is not modeled here: this can only report observation.
    pub fn decision(&self) -> Value<'static> {
        observed(self.action.observation_reason())
    }
}

pub fn project_hello_lifecycle<'a>(
    snapshot: HelloLifecycleSnapshot<'a>,
) -> HelloLifecycleProjection<'a> {
    let event = snapshot.source_event_sequence;
    let mut evidence = vec![
        descriptor_evidence(snapshot.descriptor, event),
        artifact_evidence(snapshot.artifact, event),
        slot_evidence(snapshot.service_slot, event),
        inventory_evidence(snapshot.inventory, event),
        state_evidence(snapshot.state_transition, event),
    ];
    if let Some(migration) = snapshot.state_migration {
        evidence.push(migration_evidence(migration, event));
    }
    if let Some(health) = snapshot.health {
        evidence.push(health_evidence(health, event));
    }

    HelloLifecycleProjection {
        facts: Value::InlineObject(vec![
            Field::new("action", Value::Str(snapshot.action.as_str())),
            Field::new("service_id", Value::Str(snapshot.service_id)),
        ]),
        evidence,
        action: snapshot.action,
    }
}

fn record<'a>(
    id: &'a str,
    kind: &'a str,
    meta: EvidenceMeta<'a>,
    event: u64,
    facts: Vec<Field<'a>>,
) -> Evidence<'a> {
    let unknown = meta.classification == EvidenceClassification::Unknown;
    Evidence {
        id,
        kind,
        status: if unknown {
            "rejected"
        } else {
            meta.status.as_str()
        },
        reason: if unknown {
            "unknown_classification"
        } else {
            meta.reason
        },
        source_event_sequence: Some(event),
        classification: meta.classification.as_str(),
        facts: Value::InlineObject(facts),
    }
}

fn descriptor_evidence<'a>(value: DescriptorSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "descriptor",
        "descriptor",
        value.meta,
        event,
        vec![
            Field::new("descriptor_id", Value::Str(value.descriptor_id)),
            Field::new("source_locator", Value::Str(value.source_locator)),
            Field::new("source_kind", Value::Str(value.source_kind)),
            Field::new("descriptor_hash", Value::Sha256(value.descriptor_hash)),
            Field::new("bound_source_hash", Value::Sha256(value.bound_source_hash)),
            Field::new("signature_hash", Value::Sha256(value.signature_hash)),
            Field::new("attestation_hash", Value::Sha256(value.attestation_hash)),
        ],
    )
}

fn artifact_evidence<'a>(value: ArtifactSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "artifact",
        "artifact",
        value.meta,
        event,
        vec![
            Field::new("artifact_id", Value::Str(value.artifact_id)),
            Field::new("artifact_hash", Value::Sha256(value.artifact_hash)),
            Field::new(
                "content_binding_hash",
                Value::Sha256(value.content_binding_hash),
            ),
            Field::new("reference_hash", Value::Sha256(value.reference_hash)),
            Field::new("bytes_hash", Value::Sha256(value.bytes_hash)),
            Field::new("signature_hash", Value::Sha256(value.signature_hash)),
            Field::new("preflight_status", Value::Str(value.preflight_status)),
        ],
    )
}

fn slot_evidence<'a>(value: ServiceSlotSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "service_slot",
        "service_slot_state",
        value.meta,
        event,
        vec![
            Field::new("slot_id", Value::Str(value.slot_id)),
            Field::new("before_status", Value::Str(value.before_status)),
            Field::new("after_status", Value::Str(value.after_status)),
            Field::new("active", Value::Bool(value.active)),
        ],
    )
}

fn inventory_evidence<'a>(value: InventorySnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "inventory_change",
        "inventory_change",
        value.meta,
        event,
        vec![
            Field::new("change", Value::Str(value.change)),
            Field::new("present_before", Value::Bool(value.present_before)),
            Field::new("present_after", Value::Bool(value.present_after)),
        ],
    )
}

fn state_value<'a>(value: ServiceStateSnapshot<'a>) -> Value<'a> {
    Value::InlineObject(vec![
        Field::new("version", Value::Str(value.version)),
        Field::new("generation", Value::U64(value.generation)),
        Field::new("loaded", Value::Bool(value.loaded)),
        Field::new("running", Value::Bool(value.running)),
        Field::new("state_counter", Value::U64(value.state_counter)),
        Field::new("state_hash", Value::Sha256(value.state_hash)),
    ])
}

fn state_evidence<'a>(value: StateTransitionSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "state_transition",
        "state_transition",
        value.meta,
        event,
        vec![
            Field::new("before", state_value(value.before)),
            Field::new("after", state_value(value.after)),
        ],
    )
}

fn migration_evidence<'a>(value: StateMigrationSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "state_migration",
        "state_migration",
        value.meta,
        event,
        vec![
            Field::new("from_version", Value::Str(value.from_version)),
            Field::new("to_version", Value::Str(value.to_version)),
            Field::new("state_preserved", Value::Bool(value.state_preserved)),
            Field::new("migration_hash", Value::Sha256(value.migration_hash)),
        ],
    )
}

fn health_evidence<'a>(value: HealthSnapshot<'a>, event: u64) -> Evidence<'a> {
    record(
        "health",
        "health_record",
        value.meta,
        event,
        vec![
            Field::new("status_detail", Value::Str(value.status_detail)),
            Field::new("loaded", Value::Bool(value.loaded)),
            Field::new("running", Value::Bool(value.running)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::*;
    use crate::{
        evidence_response::{evidence_value, response_value, Envelope},
        record::write_json,
    };

    const HASH: [u8; 32] = [0x5a; 32];

    fn meta() -> EvidenceMeta<'static> {
        EvidenceMeta {
            status: EvidenceStatus::Verified,
            reason: "captured",
            classification: EvidenceClassification::LocalOnly,
        }
    }

    fn snapshot(action: LifecycleAction) -> HelloLifecycleSnapshot<'static> {
        HelloLifecycleSnapshot {
            action,
            source_event_sequence: 7,
            service_id: "svc.demo.hello",
            descriptor: DescriptorSnapshot {
                meta: meta(),
                descriptor_id: "descriptor.hello.v1",
                source_locator: "seed-kernel/src/hello_service.rs",
                source_kind: "builtin",
                descriptor_hash: HASH,
                bound_source_hash: HASH,
                signature_hash: HASH,
                attestation_hash: HASH,
            },
            artifact: ArtifactSnapshot {
                meta: meta(),
                artifact_id: "artifact.hello.v1",
                artifact_hash: HASH,
                content_binding_hash: HASH,
                reference_hash: HASH,
                bytes_hash: HASH,
                signature_hash: HASH,
                preflight_status: "accepted",
            },
            service_slot: ServiceSlotSnapshot {
                meta: meta(),
                slot_id: "slot.current_boot.hello",
                before_status: "unallocated",
                after_status: "active",
                active: true,
            },
            inventory: InventorySnapshot {
                meta: meta(),
                change: "added",
                present_before: false,
                present_after: true,
            },
            state_transition: StateTransitionSnapshot {
                meta: meta(),
                before: ServiceStateSnapshot {
                    version: "missing",
                    generation: 0,
                    loaded: false,
                    running: false,
                    state_counter: 0,
                    state_hash: HASH,
                },
                after: ServiceStateSnapshot {
                    version: "v1",
                    generation: 1,
                    loaded: true,
                    running: true,
                    state_counter: 0,
                    state_hash: HASH,
                },
            },
            state_migration: None,
            health: None,
        }
    }

    fn render(projection: &HelloLifecycleProjection<'static>) -> String {
        let envelope = Envelope {
            response_sequence: 1,
            family: "hello.lifecycle",
            scope: "current_boot",
            classification: "local_only",
            source_method: "module.load_ephemeral",
            event_sequence: Some(7),
            facts: projection.facts.clone(),
            evidence: projection
                .evidence
                .iter()
                .map(|item| {
                    evidence_value(Evidence {
                        id: item.id,
                        kind: item.kind,
                        status: item.status,
                        reason: item.reason,
                        source_event_sequence: item.source_event_sequence,
                        classification: item.classification,
                        facts: item.facts.clone(),
                    })
                })
                .collect(),
            decision: projection.decision(),
        };
        let mut bytes = Vec::new();
        write_json(&response_value(&envelope), &mut bytes, 0);
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn load_mutation_is_observed_and_slot_inventory_changes_are_evidence() {
        let projection = project_hello_lifecycle(snapshot(LifecycleAction::Load));
        let json = render(&projection);
        assert!(json.contains("\"outcome\": \"observed\", \"reason\": \"load_performed\""));
        assert!(json.contains("\"id\": \"service_slot\""));
        assert!(json.contains("\"before_status\": \"unallocated\", \"after_status\": \"active\""));
        assert!(json.contains("\"id\": \"inventory_change\""));
        assert!(json.contains("\"present_before\": false, \"present_after\": true"));
    }

    #[test]
    fn every_lifecycle_decision_has_no_authorized_effects() {
        for action in [
            LifecycleAction::Load,
            LifecycleAction::Start,
            LifecycleAction::Restart,
            LifecycleAction::HotSwap,
            LifecycleAction::Stop,
            LifecycleAction::Drop,
            LifecycleAction::Health,
        ] {
            let json = render(&project_hello_lifecycle(snapshot(action)));
            assert!(!json.contains("\"effects\""), "{}", action.as_str());
            assert!(!json.contains("\"grants\""), "{}", action.as_str());
        }
    }

    #[test]
    fn projection_source_has_no_positive_decision_constructor() {
        let source = include_str!("hello_lifecycle_projection.rs");
        assert!(!source.contains(concat!("Grant", "Decision")));
        assert!(!source.contains(concat!("Grant", "Proof")));
        assert!(!source.contains(concat!("Granted", "(")));
    }

    #[test]
    fn health_is_observed_and_event_is_provenance_only() {
        let mut input = snapshot(LifecycleAction::Health);
        input.health = Some(HealthSnapshot {
            meta: EvidenceMeta {
                status: EvidenceStatus::NotApplicable,
                reason: "health_observation",
                classification: EvidenceClassification::LocalOnly,
            },
            status_detail: "running",
            loaded: true,
            running: true,
        });
        let projection = project_hello_lifecycle(input);
        let health = projection.evidence.last().unwrap();
        assert_eq!(health.id, "health");
        assert_eq!(health.source_event_sequence, Some(7));
        let json = render(&projection);
        assert!(json.contains("\"reason\": \"health_performed\""));
        assert!(json.contains("\"source_event_id\": \"event.current_boot.00000007\""));
        assert!(!json.contains("\"effects\""));
    }

    #[test]
    fn hashes_pass_through_and_use_sha256_vocabulary() {
        let json = render(&project_hello_lifecycle(snapshot(LifecycleAction::Load)));
        let rendered = "sha256:5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
        assert!(json.contains(&format!("\"descriptor_hash\": \"{rendered}\"")));
        assert!(json.contains(&format!("\"artifact_hash\": \"{rendered}\"")));
    }

    #[test]
    fn classification_is_preserved_per_record() {
        let mut input = snapshot(LifecycleAction::Load);
        input.descriptor.meta.classification = EvidenceClassification::Public;
        input.artifact.meta.classification = EvidenceClassification::Secret;
        let projection = project_hello_lifecycle(input);
        assert_eq!(projection.evidence[0].classification, "public");
        assert_eq!(projection.evidence[1].classification, "secret");
    }

    #[test]
    fn unknown_classification_is_an_explicit_rejected_record() {
        let mut input = snapshot(LifecycleAction::Load);
        input.inventory.meta.classification = EvidenceClassification::Unknown;
        let record = &project_hello_lifecycle(input).evidence[3];
        assert_eq!(record.id, "inventory_change");
        assert_eq!(record.status, "rejected");
        assert_eq!(record.reason, "unknown_classification");
        assert_eq!(record.classification, "local_only");
    }

    #[test]
    fn evidence_order_is_manifest_order() {
        let mut input = snapshot(LifecycleAction::HotSwap);
        input.state_migration = Some(StateMigrationSnapshot {
            meta: meta(),
            from_version: "v1",
            to_version: "v2",
            state_preserved: true,
            migration_hash: HASH,
        });
        input.health = Some(HealthSnapshot {
            meta: meta(),
            status_detail: "running",
            loaded: true,
            running: true,
        });
        assert_eq!(
            project_hello_lifecycle(input)
                .evidence
                .iter()
                .map(|record| record.id)
                .collect::<Vec<_>>(),
            [
                "descriptor",
                "artifact",
                "service_slot",
                "inventory_change",
                "state_transition",
                "state_migration",
                "health",
            ]
        );
    }

    #[test]
    fn all_records_share_the_captured_event() {
        let projection = project_hello_lifecycle(snapshot(LifecycleAction::Load));
        assert!(projection
            .evidence
            .iter()
            .all(|record| record.source_event_sequence == Some(7)));
    }

    #[test]
    fn reserved_decision_keys_are_not_facts() {
        fn check(value: &Value<'_>) {
            match value {
                Value::InlineObject(fields) | Value::Object(fields) => {
                    for field in fields {
                        assert!(!matches!(
                            field.key,
                            "outcome" | "grants" | "effects" | "blocked_by"
                        ));
                        assert!(!field.key.starts_with("authorizes_"));
                        check(&field.value);
                    }
                }
                Value::InlineArray(values) | Value::Array(values) => {
                    for value in values {
                        check(value);
                    }
                }
                _ => {}
            }
        }
        let projection = project_hello_lifecycle(snapshot(LifecycleAction::Load));
        check(&projection.facts);
        for evidence in &projection.evidence {
            check(&evidence.facts);
        }
    }

    #[test]
    fn migration_hash_and_versions_are_evidence_not_effects() {
        let mut input = snapshot(LifecycleAction::HotSwap);
        input.state_migration = Some(StateMigrationSnapshot {
            meta: meta(),
            from_version: "v1",
            to_version: "v2",
            state_preserved: true,
            migration_hash: [0x6b; 32],
        });
        let json = render(&project_hello_lifecycle(input));
        assert!(json.contains("\"from_version\": \"v1\", \"to_version\": \"v2\""));
        assert!(json.contains("\"migration_hash\": \"sha256:6b6b"));
        assert!(!json.contains("\"effects\""));
    }
}
