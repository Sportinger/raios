//! Fail-closed evidence-v1 projections for read-only system/status responses.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{denied, observed, Blocked, Evidence},
    record::Value,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Classification {
    Public,
    LocalOnly,
    Secret,
    Unknown,
}

impl Classification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly => "local_only",
            Self::Secret => "secret",
            Self::Unknown => "unknown",
        }
    }
}

pub struct EvidenceInput<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub status: &'a str,
    pub reason: &'a str,
    pub source_event_sequence: Option<u64>,
    pub classification: Classification,
    pub facts: Value<'a>,
}

pub struct Projection<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    pub decision: Value<'a>,
    pub classification: Classification,
}

pub fn observed_projection<'a>(
    facts: Value<'a>,
    evidence: Vec<EvidenceInput<'a>>,
    reason: &'a str,
) -> Projection<'a> {
    Projection {
        facts: checked(facts),
        evidence: evidence
            .into_iter()
            .map(|item| {
                let unknown = item.classification == Classification::Unknown;
                Evidence {
                    id: item.id,
                    kind: item.kind,
                    status: if unknown { "rejected" } else { item.status },
                    reason: if unknown {
                        "unknown_classification"
                    } else {
                        item.reason
                    },
                    source_event_sequence: item.source_event_sequence,
                    classification: item.classification.as_str(),
                    facts: checked(item.facts),
                }
            })
            .collect(),
        decision: observed(reason),
        classification: Classification::LocalOnly,
    }
}

pub fn generic_denial<'a>(
    facts: Value<'a>,
    requested_capability: &'a str,
    blocked: &'a [Blocked<'a>],
) -> Projection<'a> {
    let reason = blocked
        .first()
        .map(|item| item.reason)
        .unwrap_or("capability_evidence_missing");
    Projection {
        facts: checked(facts),
        evidence: vec![],
        decision: denied(reason, requested_capability, blocked),
        classification: Classification::LocalOnly,
    }
}

fn checked(value: Value<'_>) -> Value<'_> {
    assert_no_decision_keys(&value);
    value
}

fn assert_no_decision_keys(value: &Value<'_>) {
    match value {
        Value::InlineObject(fields) | Value::Object(fields) => {
            for field in fields {
                assert!(
                    !matches!(field.key, "outcome" | "grants" | "effects" | "blocked_by")
                        && !field.key.starts_with("authorizes_"),
                    "decision key is reserved: {}",
                    field.key
                );
                assert_no_decision_keys(&field.value);
            }
        }
        Value::InlineArray(values) | Value::Array(values) => {
            for value in values {
                assert_no_decision_keys(value);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evidence_response::evidence_value,
        record::{write_json, Field},
    };

    fn render(value: &Value<'_>) -> alloc::string::String {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        alloc::string::String::from_utf8(out).unwrap()
    }

    #[test]
    fn capability_catalog_status_is_observed_and_grants_nothing() {
        let projection = observed_projection(
            Value::InlineObject(vec![Field::new(
                "capabilities",
                Value::InlineArray(vec![Value::InlineObject(vec![
                    Field::new("id", Value::Str("cap.system.describe.read")),
                    Field::new("status", Value::Str("granted")),
                ])]),
            )]),
            vec![],
            "capability_catalog_observed",
        );
        let json = render(&projection.decision);
        assert!(json.contains("\"outcome\": \"observed\""));
        assert!(!json.contains("\"grants\""));
        assert!(!json.contains("\"effects\""));
    }

    #[test]
    fn generic_denial_preserves_evaluator_order_and_first_reason() {
        let required = [
            Blocked {
                evidence_id: "module_manifest",
                status: "missing",
                reason: "manifest_missing",
            },
            Blocked {
                evidence_id: "vm_test_report",
                status: "missing",
                reason: "vm_report_missing",
            },
            Blocked {
                evidence_id: "local_attestation",
                status: "missing",
                reason: "attestation_missing",
            },
            Blocked {
                evidence_id: "computed_capability_grant",
                status: "missing",
                reason: "grant_missing",
            },
            Blocked {
                evidence_id: "local_approval",
                status: "missing",
                reason: "approval_missing",
            },
            Blocked {
                evidence_id: "rollback_plan",
                status: "missing",
                reason: "rollback_missing",
            },
        ];
        let json =
            render(&generic_denial(Value::InlineObject(vec![]), "cap.test", &required).decision);
        assert!(json.contains("\"reason\": \"manifest_missing\""));
        assert!(json.contains("\"grants\": [], \"effects\": []"));
        let mut offset = 0;
        for id in required.map(|item| item.evidence_id) {
            offset += json[offset..].find(id).unwrap() + id.len();
        }
    }

    #[test]
    fn unknown_classification_is_rejected_not_public() {
        let projection = observed_projection(
            Value::InlineObject(vec![]),
            vec![EvidenceInput {
                id: "unknown",
                kind: "fact",
                status: "observed",
                reason: "observed",
                source_event_sequence: None,
                classification: Classification::Unknown,
                facts: Value::InlineObject(vec![]),
            }],
            "read_completed",
        );
        let json = render(&evidence_value(
            projection.evidence.into_iter().next().unwrap(),
        ));
        assert!(json.contains("\"status\": \"rejected\""));
        assert!(json.contains("\"classification\": \"unknown\""));
        assert!(!json.contains("\"classification\": \"public\""));
    }

    #[test]
    #[should_panic(expected = "decision key is reserved: effects")]
    fn facts_reject_reserved_decision_keys() {
        observed_projection(
            Value::InlineObject(vec![Field::new("effects", Value::InlineArray(vec![]))]),
            vec![],
            "read_completed",
        );
    }

    #[test]
    fn source_has_no_positive_constructor() {
        let source = include_str!("system_status_projection.rs");
        assert!(!source.contains(concat!("Grant", "Decision")));
        assert!(!source.contains(concat!("Grant", "Proof")));
    }
}
