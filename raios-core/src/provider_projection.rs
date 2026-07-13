//! Typed evidence-v1 projections for provider posture and export denial.

use alloc::{vec, vec::Vec};

use crate::{
    evidence_response::{denied, evidence_value, observed, Blocked, Evidence},
    provider_context::ProviderContextHashes,
    record::{Field, Value},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProviderClassification {
    Public,
    LocalOnly,
    Secret,
    Unknown,
}

impl ProviderClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::LocalOnly => "local_only",
            Self::Secret => "secret",
            Self::Unknown => "unknown",
        }
    }
}

pub struct ProviderProjection<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    pub decision: Value<'a>,
    pub classification: ProviderClassification,
}

pub fn projection_value(projection: ProviderProjection<'_>) -> Value<'_> {
    Value::InlineObject(vec![
        Field::new("facts", projection.facts),
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
        Field::new("decision", projection.decision),
    ])
}

pub struct ProviderTrustInput<'a> {
    pub provider_id: &'a str,
    pub descriptor_id: &'a str,
    pub host: &'a str,
    pub descriptor_sha256: [u8; 32],
    pub trust_state: &'a str,
    pub chain_policy: &'a str,
    pub time_policy: &'a str,
    pub development_bypass: bool,
    pub status: &'a str,
    pub reason: &'a str,
    pub honest: bool,
    pub chain_validated: bool,
    pub time_validated: bool,
    pub claims_chain_validated: bool,
    pub claims_time_validated: bool,
}

pub fn project_trust_honesty<'a>(input: ProviderTrustInput<'a>) -> ProviderProjection<'a> {
    ProviderProjection {
        facts: facts(vec![Field::new(
            "provider_id",
            Value::Str(input.provider_id),
        )]),
        evidence: vec![
            Evidence {
                id: "provider_trust_descriptor",
                kind: "provider_trust_descriptor",
                status: "present",
                reason: "descriptor_observed",
                source_event_sequence: None,
                classification: "local_only",
                facts: facts(vec![
                    Field::new("descriptor_id", Value::Str(input.descriptor_id)),
                    Field::new("host", Value::Str(input.host)),
                    Field::new("descriptor_sha256", Value::Sha256(input.descriptor_sha256)),
                    Field::new("trust_state", Value::Str(input.trust_state)),
                    Field::new("chain_policy", Value::Str(input.chain_policy)),
                    Field::new("time_policy", Value::Str(input.time_policy)),
                    Field::new("development_bypass", Value::Bool(input.development_bypass)),
                ]),
            },
            Evidence {
                id: "provider_trust_honesty",
                kind: "provider_trust_honesty",
                status: input.status,
                reason: input.reason,
                source_event_sequence: None,
                classification: "public",
                facts: facts(vec![
                    Field::new("honest", Value::Bool(input.honest)),
                    Field::new("chain_validated", Value::Bool(input.chain_validated)),
                    Field::new("time_validated", Value::Bool(input.time_validated)),
                    Field::new(
                        "claims_chain_validated",
                        Value::Bool(input.claims_chain_validated),
                    ),
                    Field::new(
                        "claims_time_validated",
                        Value::Bool(input.claims_time_validated),
                    ),
                ]),
            },
        ],
        decision: observed("provider_trust_honesty_evaluated"),
        classification: ProviderClassification::LocalOnly,
    }
}

#[derive(Clone, Copy)]
pub enum KeyPresence {
    Set,
    Missing,
    Unknown,
}

impl KeyPresence {
    fn state(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Missing => "missing",
            Self::Unknown => "unknown",
        }
    }
}

pub struct ProviderProjectionHashes {
    pub projected_packet_hash: [u8; 32],
    pub exported_field_list_hash: [u8; 32],
    pub omitted_field_list_hash: [u8; 32],
    pub redaction_policy_hash: [u8; 32],
    pub field_classification_hash: [u8; 32],
    pub token_budget_hash: [u8; 32],
}

impl From<ProviderContextHashes> for ProviderProjectionHashes {
    fn from(value: ProviderContextHashes) -> Self {
        Self {
            projected_packet_hash: value.projected_packet_hash,
            exported_field_list_hash: value.exported_field_list_hash,
            omitted_field_list_hash: value.omitted_field_list_hash,
            redaction_policy_hash: value.redaction_policy_hash,
            field_classification_hash: value.field_classification_hash,
            token_budget_hash: value.token_budget_hash,
        }
    }
}

pub struct ProviderMinimalInput<'a> {
    pub packet: Value<'a>,
    pub included_fields: Value<'a>,
    pub omitted_fields: Value<'a>,
    pub hashes: ProviderProjectionHashes,
    pub trust_state: &'a str,
    pub trust_positive: bool,
    pub key_presence: KeyPresence,
    pub source_event_sequence: Option<u64>,
}

pub fn project_provider_minimal<'a>(input: ProviderMinimalInput<'a>) -> ProviderProjection<'a> {
    let key_unknown = matches!(input.key_presence, KeyPresence::Unknown);
    let mut blocked = Vec::new();
    if !input.trust_positive {
        blocked.push(Blocked {
            evidence_id: "provider_trust",
            status: input.trust_state,
            reason: "provider_trust_not_positive",
        });
    }
    blocked.push(Blocked {
        evidence_id: "provider_context_export_audit_binding",
        status: "missing",
        reason: "provider_context_export_audit_binding_missing",
    });
    ProviderProjection {
        facts: facts(vec![
            Field::new("profile", Value::Str("provider_minimal")),
            Field::new("mode", Value::Str("local_read_only")),
            Field::new("redaction_projection", Value::Str("present")),
            Field::new("classification_default", Value::Str("local_only")),
            Field::new("unclassified_field_policy", Value::Str("reject")),
            Field::new("included_fields", input.included_fields),
            Field::new("omitted_fields", input.omitted_fields),
            Field::new("packet", input.packet),
        ]),
        evidence: vec![
            Evidence {
                id: "provider_trust",
                kind: "provider_trust",
                status: if input.trust_positive {
                    "verified"
                } else {
                    "rejected"
                },
                reason: if input.trust_positive {
                    "provider_trust_positive"
                } else {
                    "provider_trust_not_positive"
                },
                source_event_sequence: input.source_event_sequence,
                classification: "public",
                facts: facts(vec![Field::new(
                    "trust_state",
                    Value::Str(input.trust_state),
                )]),
            },
            Evidence {
                id: "key_presence",
                kind: "key_presence",
                status: if key_unknown { "rejected" } else { "observed" },
                reason: if key_unknown {
                    "unknown_classification"
                } else {
                    "key_presence_observed"
                },
                source_event_sequence: input.source_event_sequence,
                classification: if key_unknown { "unknown" } else { "public" },
                facts: facts(vec![Field::new(
                    "state",
                    Value::Str(input.key_presence.state()),
                )]),
            },
            provider_projection_binding(input.hashes, input.source_event_sequence),
        ],
        decision: denied(blocked[0].reason, "cap.provider.context_export", &blocked),
        classification: ProviderClassification::LocalOnly,
    }
}

pub struct ProviderGateInput<'a> {
    pub facts: Value<'a>,
    pub evidence: Vec<Evidence<'a>>,
    pub blocked_by: Vec<Blocked<'a>>,
    pub requested_capability: &'a str,
}

pub fn project_export_gate<'a>(input: ProviderGateInput<'a>) -> ProviderProjection<'a> {
    let reason = input
        .blocked_by
        .first()
        .map(|item| item.reason)
        .unwrap_or("provider_export_evidence_missing");
    ProviderProjection {
        facts: checked_value(input.facts),
        evidence: input.evidence,
        decision: denied(reason, input.requested_capability, &input.blocked_by),
        classification: ProviderClassification::LocalOnly,
    }
}

pub fn provider_projection_binding<'a>(
    hashes: ProviderProjectionHashes,
    source_event_sequence: Option<u64>,
) -> Evidence<'a> {
    Evidence {
        id: "provider_projection_binding",
        kind: "provider_projection_binding",
        status: "present",
        reason: "canonical_projection_captured",
        source_event_sequence,
        classification: "local_only",
        facts: facts(vec![
            Field::new(
                "canonicalization",
                Value::Str("raios.provider_minimal.packet.canonical.v0"),
            ),
            Field::new(
                "projected_packet_hash",
                Value::Sha256(hashes.projected_packet_hash),
            ),
            Field::new(
                "exported_field_list_hash",
                Value::Sha256(hashes.exported_field_list_hash),
            ),
            Field::new(
                "omitted_field_list_hash",
                Value::Sha256(hashes.omitted_field_list_hash),
            ),
            Field::new(
                "redaction_policy_hash",
                Value::Sha256(hashes.redaction_policy_hash),
            ),
            Field::new(
                "field_classification_hash",
                Value::Sha256(hashes.field_classification_hash),
            ),
            Field::new("token_budget_hash", Value::Sha256(hashes.token_budget_hash)),
        ]),
    }
}

fn facts(fields: Vec<Field<'_>>) -> Value<'_> {
    checked_value(Value::InlineObject(fields))
}

fn checked_value(value: Value<'_>) -> Value<'_> {
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
    use alloc::{string::String, vec};

    use super::*;
    use crate::{
        evidence_response::evidence_value,
        provider_context::{
            provider_context_hashes, provider_export_packet_hash, ProviderProjectionInput,
            PROVIDER_MINIMAL_INCLUDED_FIELDS, PROVIDER_MINIMAL_OMITTED_FIELDS,
        },
        provider_trust_descriptor::OPENAI_PROVIDER_TRUST_DESCRIPTOR,
        record::write_json,
    };

    fn render(value: &Value<'_>) -> String {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        String::from_utf8(out).unwrap()
    }

    fn empty_hashes() -> ProviderContextHashes {
        provider_context_hashes(&ProviderProjectionInput {
            status: &[],
            provider: &[],
            services: &[],
            capabilities: &[],
            problems: &[],
            records: &[],
        })
    }

    fn minimal(key_presence: KeyPresence) -> ProviderProjection<'static> {
        project_provider_minimal(ProviderMinimalInput {
            packet: Value::InlineObject(vec![]),
            included_fields: Value::InlineArray(vec![]),
            omitted_fields: Value::InlineArray(vec![]),
            hashes: empty_hashes().into(),
            trust_state: "pin_config_missing",
            trust_positive: false,
            key_presence,
            source_event_sequence: Some(7),
        })
    }

    #[test]
    fn provider_hashes_remain_hand_pinned() {
        let hashes = empty_hashes();
        assert_eq!(
            hashes.projected_packet_hash,
            [
                135, 67, 9, 124, 24, 183, 102, 158, 27, 73, 54, 183, 42, 50, 150, 247, 170, 136,
                28, 68, 144, 97, 137, 2, 81, 100, 161, 102, 223, 216, 201, 72
            ]
        );
        assert_eq!(
            hashes.exported_field_list_hash,
            [
                94, 143, 68, 111, 144, 105, 229, 65, 169, 132, 24, 182, 45, 173, 203, 110, 95, 15,
                137, 61, 113, 78, 39, 185, 201, 161, 61, 1, 72, 46, 10, 10
            ]
        );
        assert_eq!(
            hashes.omitted_field_list_hash,
            [
                218, 65, 138, 25, 29, 224, 242, 102, 83, 125, 60, 4, 253, 138, 36, 75, 28, 48, 30,
                186, 121, 225, 66, 72, 42, 90, 110, 11, 171, 233, 189, 254
            ]
        );
        assert_eq!(
            hashes.redaction_policy_hash,
            [
                97, 2, 93, 232, 129, 17, 77, 70, 125, 182, 104, 120, 186, 116, 131, 157, 16, 221,
                104, 143, 146, 189, 123, 118, 21, 172, 129, 21, 90, 148, 146, 52
            ]
        );
        assert_eq!(
            hashes.field_classification_hash,
            [
                214, 249, 45, 110, 47, 198, 152, 53, 210, 196, 60, 188, 85, 17, 169, 200, 31, 29,
                124, 207, 92, 215, 197, 61, 222, 238, 238, 112, 21, 255, 251, 111
            ]
        );
        assert_eq!(
            hashes.token_budget_hash,
            [
                30, 253, 213, 83, 13, 110, 250, 191, 62, 168, 144, 189, 199, 113, 171, 144, 152,
                132, 146, 65, 223, 250, 117, 223, 1, 36, 61, 142, 13, 164, 92, 242
            ]
        );
        assert_eq!(
            provider_export_packet_hash(&[]),
            [
                41, 62, 219, 120, 195, 71, 199, 85, 156, 223, 76, 234, 101, 168, 3, 194, 26, 1,
                131, 240, 112, 237, 86, 36, 140, 91, 157, 44, 109, 19, 131, 253
            ]
        );
        assert_eq!(
            OPENAI_PROVIDER_TRUST_DESCRIPTOR.record_sha256(),
            [
                248, 86, 237, 65, 34, 213, 123, 3, 245, 87, 62, 62, 246, 119, 129, 128, 86, 160,
                249, 175, 46, 216, 205, 208, 180, 118, 19, 50, 53, 125, 181, 93,
            ]
        );
    }

    #[test]
    fn key_presence_is_public_but_bytes_have_no_api_path() {
        let projection = minimal(KeyPresence::Set);
        let key = projection
            .evidence
            .iter()
            .find(|item| item.id == "key_presence")
            .unwrap();
        assert_eq!(key.classification, "public");
        assert!(render(&key.facts).contains("\"state\": \"set\""));
        assert!(!render(&key.facts).contains("api_key"));
    }

    #[test]
    fn key_presence_never_creates_a_secret_locator() {
        let projection = projection_value(minimal(KeyPresence::Set));
        let json = render(&projection);
        assert!(!json.contains("source_locator"));
        assert!(!json.contains("secret_locator"));
        assert!(!json.contains("api_key_bytes"));
    }

    #[test]
    fn unknown_key_classification_is_rejected_not_public() {
        let projection = minimal(KeyPresence::Unknown);
        let key = projection
            .evidence
            .iter()
            .find(|item| item.id == "key_presence")
            .unwrap();
        assert_eq!(
            (key.status, key.reason, key.classification),
            ("rejected", "unknown_classification", "unknown")
        );
    }

    #[test]
    fn trust_honesty_is_observed_and_grants_nothing() {
        let projection = project_trust_honesty(ProviderTrustInput {
            provider_id: "openai",
            descriptor_id: "d",
            host: "api.openai.com",
            descriptor_sha256: [1; 32],
            trust_state: "pinned_spki_verified",
            chain_policy: "not_validated",
            time_policy: "not_validated",
            development_bypass: false,
            status: "verified",
            reason: "honest",
            honest: true,
            chain_validated: false,
            time_validated: false,
            claims_chain_validated: false,
            claims_time_validated: false,
        });
        assert_eq!(
            render(&projection.decision),
            "{\"outcome\": \"observed\", \"reason\": \"provider_trust_honesty_evaluated\"}"
        );
    }

    #[test]
    fn export_gate_denial_preserves_order_and_first_failure() {
        let projection = project_export_gate(ProviderGateInput {
            facts: Value::InlineObject(vec![]),
            evidence: vec![],
            blocked_by: vec![
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
            requested_capability: "cap.provider.context_export",
        });
        let json = render(&projection.decision);
        assert!(json.contains("\"reason\": \"first_missing\""));
        assert!(json.contains("\"grants\": [], \"effects\": []"));
        assert!(json.find("first").unwrap() < json.find("second").unwrap());
    }

    #[test]
    fn facts_have_no_reserved_decision_keys_and_api_never_grants() {
        let projection = minimal(KeyPresence::Missing);
        assert_no_decision_keys(&projection.facts);
        for evidence in projection.evidence {
            assert_no_decision_keys(&evidence.facts);
        }
        let json = render(&projection.decision);
        assert!(json.contains("\"outcome\": \"denied\""));
        assert!(!json.contains("\"outcome\": \"granted\""));
    }

    #[test]
    #[should_panic(expected = "decision key is reserved: blocked_by")]
    fn nested_provider_facts_reject_reserved_decision_keys() {
        project_provider_minimal(ProviderMinimalInput {
            packet: Value::InlineObject(vec![Field::new("blocked_by", Value::InlineArray(vec![]))]),
            included_fields: Value::InlineArray(vec![]),
            omitted_fields: Value::InlineArray(vec![]),
            hashes: empty_hashes().into(),
            trust_state: "pin_config_missing",
            trust_positive: false,
            key_presence: KeyPresence::Missing,
            source_event_sequence: None,
        });
    }

    #[test]
    fn projection_source_has_no_positive_decision_constructor() {
        let source = include_str!("provider_projection.rs");
        assert!(!source.contains(concat!("Grant", "Decision")));
        assert!(!source.contains(concat!("Grant", "Proof")));
    }

    #[test]
    fn field_classifications_remain_fail_closed() {
        assert!(PROVIDER_MINIMAL_INCLUDED_FIELDS
            .iter()
            .all(|spec| spec.classification == "public" && spec.action == "include"));
        assert!(PROVIDER_MINIMAL_OMITTED_FIELDS
            .iter()
            .all(|spec| spec.classification != "public" && spec.action == "omit"));
    }

    #[test]
    fn evidence_renderer_keeps_projection_binding_scoped() {
        let projection = minimal(KeyPresence::Missing);
        let binding = projection
            .evidence
            .into_iter()
            .find(|item| item.id == "provider_projection_binding")
            .unwrap();
        let json = render(&evidence_value(binding));
        assert!(json.contains("\"classification\": \"local_only\""));
        assert!(json.contains("\"projected_packet_hash\""));
    }
}
